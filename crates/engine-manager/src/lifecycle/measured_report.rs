use super::*;
use crate::measured_report::{self as report, fingerprint};
use app_model::{AcceptedMeasuredSceneDto, MeasuredReviewDto, MeasuredSceneDto};
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Default)]
pub(super) struct Owner {
    generation: u64,
    operation: Option<String>,
    cancelled: Option<Arc<AtomicBool>>,
    receipt: Option<MeasuredReviewDto>,
    worker: bool,
    scene_run: Option<String>,
    scene_pending: bool,
    adopted: Vec<(String, NodePath, u64, Instant)>,
    cleanup_failed: bool,
    measurement: Option<(
        String,
        app_model::ApplicationMeasurementProgressDto,
        std::collections::BTreeMap<String, u32>,
    )>,
    event_loop_samples: Vec<f64>,
    event_loop_times: Vec<[f64; 2]>,
    row_clock: Option<Instant>,
    clock_exchanges: Vec<[f64; 3]>,
    sample_window: Option<[f64; 2]>,
    first_adopted: Option<Instant>,
    probe_finished: bool,
    search_progress: Option<(String, u64)>,
}
/// Nonqueued setup ownership. Cancellation retires publication immediately; Drop proves worker exit.
pub struct SetupLease {
    manager: ForegroundEngineManager,
    id: String,
    generation: u64,
    cancelled: Arc<AtomicBool>,
}
impl SetupLease {
    pub fn begin_row(
        &self,
        target: &str,
        scene: MeasuredSceneDto,
        round: u32,
        candidate: bool,
        parameters: std::collections::BTreeMap<String, u32>,
    ) -> Result<app_model::ApplicationMeasurementProgressDto, String> {
        self.check()?;
        report::parameters(
            &serde_json::to_value(&parameters).map_err(|e| e.to_string())?,
            scene,
        )?;
        let progress = app_model::ApplicationMeasurementProgressDto {
            operation_id: self.id.clone(),
            row_id: Uuid::new_v4().to_string(),
            scene,
            round,
            candidate,
            sampling: false,
            closing: false,
            backend_clock_ms: 0.0,
        };
        let mut owner = self.manager.inner.measured.lock().expect("setup owner");
        if owner.generation != self.generation {
            return Err("Measurement retired".into());
        }
        owner.measurement = Some((target.to_owned(), progress.clone(), parameters));
        owner.event_loop_samples.clear();
        owner.event_loop_times.clear();
        owner.row_clock = Some(Instant::now());
        owner.clock_exchanges.clear();
        owner.sample_window = None;
        owner.probe_finished = false;
        Ok(progress)
    }
    pub fn begin_sampling(&self) -> Result<(), String> {
        self.check()?;
        {
            let mut owner = self.manager.inner.measured.lock().expect("setup owner");
            owner.measurement.as_mut().ok_or("No measurement row")?.1.sampling = true;
            owner.event_loop_samples.clear();
            owner.event_loop_times.clear();
        }
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            self.check()?;
            let owner = self.manager.inner.measured.lock().expect("setup owner");
            if !owner.event_loop_samples.is_empty() {
                // Retain pre-t0 observations to prove the eventual window was covered.
                return Ok(());
            }
            drop(owner);
            if Instant::now() >= deadline {
                return Err("Actual WebView probe did not acknowledge measurement window".into());
            }
            thread::sleep(Duration::from_millis(5));
        }
    }
    pub fn finish_sampling(&self) -> Result<serde_json::Value, String> {
        self.check()?;
        {
            let mut owner = self.manager.inner.measured.lock().expect("setup owner");
            let progress = &mut owner.measurement.as_mut().ok_or("No measurement row")?.1;
            progress.sampling = false;
            progress.closing = true;
        }
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            self.check()?;
            let owner = self.manager.inner.measured.lock().expect("setup owner");
            if owner.probe_finished {
                return report::measurement::window_samples(
                    &owner.event_loop_times,
                    &owner.clock_exchanges,
                    owner
                        .sample_window
                        .ok_or("Actual measurement window was not recorded")?,
                );
            }
            drop(owner);
            if Instant::now() >= deadline {
                return Err("WebView did not confirm final sampling flush".into());
            }
            thread::sleep(Duration::from_millis(5));
        }
    }
    pub fn mark_sample_window(&self, start: Instant, end: Instant) -> Result<(), String> {
        self.check()?;
        let mut owner = self.manager.inner.measured.lock().expect("setup owner");
        let origin = owner.row_clock.ok_or("No measurement clock")?;
        owner.sample_window = Some([
            start.duration_since(origin).as_secs_f64() * 1000.0,
            end.duration_since(origin).as_secs_f64() * 1000.0,
        ]);
        Ok(())
    }
    pub fn check(&self) -> Result<(), String> {
        if self.cancelled.load(Ordering::Acquire) {
            return Err("Setup operation retired".into());
        }
        let owner = self.manager.inner.measured.lock().expect("setup owner");
        if owner.operation.as_deref() != Some(&self.id) || owner.generation != self.generation {
            return Err("Setup generation changed".into());
        }
        Ok(())
    }
    pub fn commit<T>(&self, persist: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
        let owner = self.manager.inner.measured.lock().expect("setup owner");
        if owner.operation.as_deref() != Some(&self.id)
            || owner.generation != self.generation
            || self.cancelled.load(Ordering::Acquire)
        {
            return Err("Setup operation retired before commit".into());
        }
        persist()
    }
}
impl Drop for SetupLease {
    fn drop(&mut self) {
        let mut owner = self.manager.inner.measured.lock().expect("setup owner");
        if owner.operation.as_deref() == Some(&self.id) {
            owner.worker = false;
            owner.operation = None;
            owner.cancelled = None;
            owner.measurement = None;
            owner.scene_pending = false;
            owner.event_loop_samples.clear();
            owner.event_loop_times.clear();
        }
    }
}
impl ForegroundEngineManager {
    pub(super) fn measured_reservation(&self) -> bool {
        let owner = self.inner.measured.lock().expect("setup owner");
        owner.scene_pending
            || owner.scene_run.is_some()
            || owner.measurement.is_some()
            || owner.cleanup_failed
    }
    pub fn retire_setup_for_foreground(&self) -> Result<(), String> {
        self.retire_setup_operation()?;
        let deadline = Instant::now() + self.inner.config.stop_drain_timeout + Duration::from_secs(35);
        while self.setup_operation_status()? {
            if Instant::now() >= deadline {
                return Err("Setup worker is still retiring; retry explicitly after actual cleanup".into());
            }
            thread::sleep(Duration::from_millis(10));
        }
        Ok(())
    }
    pub fn measured_begin_observation(&self, lease: &SetupLease) -> Result<(), String> {
        lease.check()?;
        let mut owner = self.inner.measured.lock().expect("setup owner");
        owner.adopted.clear();
        owner.first_adopted = None;
        owner.search_progress = None;
        Ok(())
    }
    pub fn measured_first_result(&self, started: Instant) -> Option<f64> {
        self.inner
            .measured
            .lock()
            .expect("setup owner")
            .first_adopted
            .map(|time| time.saturating_duration_since(started).as_secs_f64())
    }
    pub fn measured_search_progress(&self, run_id: &str, job_id: &str, generation: u64) -> bool {
        let owner = self.inner.measured.lock().expect("setup owner");
        owner.scene_run.as_deref() == Some(run_id)
            && owner
                .cancelled
                .as_ref()
                .is_some_and(|flag| !flag.load(Ordering::Acquire))
            && owner
                .search_progress
                .as_ref()
                .is_some_and(|(job, document)| job == job_id && *document == generation)
    }
    pub fn measured_process_count(&self) -> usize {
        let state = self.lock();
        usize::from(state.live.is_some())
            + usize::from(state.candidate.is_some())
            + state.retiring.len()
            + state.match_residents.len()
            + state.preloads.iter().filter(|slot| slot.live.is_some()).count()
    }
    pub fn measurement_inputs(
        &self,
        target: &str,
        scene: MeasuredSceneDto,
        lease: &SetupLease,
    ) -> Result<(serde_json::Value, std::collections::BTreeMap<String, u32>), String> {
        let saved = self.report_saved(target)?;
        let spec = self.measured_baseline(&saved, scene)?;
        let gpu = fingerprint::sample_gpu()?;
        let result = fingerprint::capture(&spec, scene, &gpu, &|| lease.check().is_err())?;
        self.report_current(&saved, lease)?;
        Ok(result)
    }
    pub fn measurement_progress(&self) -> Option<app_model::ApplicationMeasurementProgressDto> {
        let owner = self.inner.measured.lock().expect("setup owner");
        owner.measurement.as_ref().map(|(_, progress, _)| {
            let mut progress = progress.clone();
            progress.backend_clock_ms = owner.row_clock.expect("row clock").elapsed().as_secs_f64() * 1000.0;
            progress
        })
    }
    pub fn measurement_probe(&self, probe: app_model::ApplicationMeasurementProbeDto) -> Result<(), String> {
        let mut owner = self.inner.measured.lock().expect("setup owner");
        let (_, progress, _) = owner.measurement.as_ref().ok_or("No active measurement row")?;
        if progress.operation_id != probe.operation_id
            || progress.row_id != probe.row_id
            || owner
                .cancelled
                .as_ref()
                .is_none_or(|flag| flag.load(Ordering::Acquire))
        {
            return Err("Measurement probe retired".into());
        }
        if !progress.sampling && !progress.closing {
            return Err("Measurement probe is outside the warm window".into());
        }
        if probe.finished && !progress.closing {
            return Err("Measurement probe closed before response completion".into());
        }
        if probe.samples_seconds.len() > 1000
            || probe.samples_seconds.iter().any(|v| !v.is_finite() || *v < 0.0)
            || owner.event_loop_samples.len() + probe.samples_seconds.len() > 360000
            || owner.clock_exchanges.len() >= 360000
            || probe.clock_exchange_ms.iter().any(|v| !v.is_finite() || *v < 0.0)
            || probe.clock_exchange_ms[2] < probe.clock_exchange_ms[0]
            || probe.sample_times_ms.len() != probe.samples_seconds.len()
            || probe
                .sample_times_ms
                .iter()
                .zip(&probe.samples_seconds)
                .any(|([expected, actual], delay)| {
                    !expected.is_finite()
                        || !actual.is_finite()
                        || *expected < 0.0
                        || *actual < 0.0
                        || (((actual - expected).max(0.0) / 1000.0) - delay).abs() > 1e-9
                })
        {
            return Err("Invalid or excessive event-loop samples".into());
        }
        owner.event_loop_samples.extend(probe.samples_seconds);
        owner.event_loop_times.extend(probe.sample_times_ms);
        owner.clock_exchanges.push(probe.clock_exchange_ms);
        owner.probe_finished |= probe.finished;
        Ok(())
    }
    pub fn measured_observing(&self, run_id: &str) -> bool {
        self.inner
            .measured
            .lock()
            .expect("setup owner")
            .scene_run
            .as_deref()
            == Some(run_id)
    }
    pub fn measured_note_adopted(&self, event: &AnalysisJobEventDto) {
        let mut owner = self.inner.measured.lock().expect("setup owner");
        if owner.scene_run.as_deref() != Some(&event.run_id) {
            return;
        }
        if let Some(frame) = &event.frame {
            owner.first_adopted.get_or_insert_with(Instant::now);
            if let Some(record) = owner
                .adopted
                .iter_mut()
                .find(|(job, path, _, _)| job == &event.job_id && path == &event.node_path)
            {
                record.2 = frame.visits as u64;
                record.3 = Instant::now();
            } else {
                owner.adopted.push((
                    event.job_id.clone(),
                    event.node_path.clone(),
                    frame.visits as u64,
                    Instant::now(),
                ));
            }
        }
    }
    pub fn measured_adopted(&self, run_id: &str, job_id: &str) -> Vec<(NodePath, u64)> {
        let owner = self.inner.measured.lock().expect("setup owner");
        if owner.scene_run.as_deref() != Some(run_id) {
            return Vec::new();
        }
        owner
            .adopted
            .iter()
            .filter(|(job, _, _, _)| job == job_id)
            .map(|(_, path, visits, _)| (path.clone(), *visits))
            .collect()
    }
    pub fn start_independent_measured_scene(
        &self,
        target: &str,
        scene: MeasuredSceneDto,
        lease: &SetupLease,
    ) -> Result<EngineRunDto, String> {
        lease.check()?;
        let saved = self.report_saved(target)?;
        self.measured_baseline(&saved, scene)?;
        {
            let state = self.lock();
            if !matches!(state.phase, Phase::NoEngine { .. })
                || state.live.is_some()
                || state.candidate.is_some()
                || !state.retiring.is_empty()
                || !state.match_residents.is_empty()
                || state.match_reservation.is_some()
                || preload::occupied(&state)
                || self.inner.evaluation_worker.load(Ordering::Acquire)
                || continuation::occupied(&state)
                || batch::occupied(&state.batch)
                || state.finite_admission_pending
                || state.analysis_task.as_ref().is_some_and(|task| {
                    !matches!(
                        task.state,
                        app_model::AnalysisTaskStateDto::Completed
                            | app_model::AnalysisTaskStateDto::Failed
                            | app_model::AnalysisTaskStateDto::Cancelled
                            | app_model::AnalysisTaskStateDto::Invalidated
                    )
                })
            {
                return Err("Independent scene requires NoEngine and reaped preload/evaluation work without another Task/Match/batch/continuation owner. Explicitly Stop, then retry.".into());
            }
            self.inner.measured.lock().expect("setup owner").scene_pending = true;
        }
        let launch = self.begin_start_scene(EngineOperationDto::Start, target, Some(scene));
        if let Err(error) = launch {
            self.inner.measured.lock().expect("setup owner").scene_pending = false;
            return Err(error.to_string());
        }
        let run_id = {
            let state = self.lock();
            current_run_id(&state.phase).ok_or("Independent scene did not reserve a Run")?
        };
        {
            let mut owner = self.inner.measured.lock().expect("setup owner");
            owner.scene_run = Some(run_id.clone());
            owner.scene_pending = false;
            owner.adopted.clear();
        }
        let deadline = Instant::now() + self.inner.config.readiness_timeout + Duration::from_secs(390);
        loop {
            if lease.check().is_err() {
                self.stop_measured_scene(&run_id)?;
                return Err("Scene launch retired and owned process reaped".into());
            }
            let state = self.lock();
            match &state.phase {
                Phase::Ready(run) if run.profile_id == target && run.measured_scene == Some(scene) => {
                    let run = run.clone();
                    drop(state);
                    if let Err(error) = lease.check() {
                        self.stop_measured_scene(&run_id)?;
                        return Err(error);
                    }
                    return Ok(run);
                }
                Phase::Error { failure, .. }
                | Phase::NoEngine {
                    failure: Some(failure),
                } => {
                    let error = failure.to_string();
                    drop(state);
                    self.stop_measured_scene(&run_id)?;
                    return Err(error);
                }
                Phase::Starting(_) => {}
                _ => {
                    drop(state);
                    self.stop_measured_scene(&run_id)?;
                    return Err("Independent scene lost its launch authority".into());
                }
            }
            drop(state);
            if Instant::now() >= deadline {
                self.stop_measured_scene(&run_id)?;
                return Err("Independent scene readiness timed out".into());
            }
            thread::sleep(Duration::from_millis(10));
        }
    }
    pub fn stop_measured_scene(&self, run_id: &str) -> Result<(), String> {
        {
            let state = self.lock();
            if current_run_id(&state.phase)
                .as_deref()
                .is_some_and(|id| id != run_id)
            {
                return Err("Scene Run was replaced; refusing to stop another owner".into());
            }
        }
        if let Err(error) = self.stop() {
            self.inner.measured.lock().expect("setup owner").cleanup_failed = true;
            return Err(error.to_string());
        }
        let deadline = Instant::now() + self.inner.config.stop_drain_timeout + Duration::from_secs(3);
        loop {
            let state = self.lock();
            if matches!(state.phase, Phase::NoEngine { .. })
                && state.live.is_none()
                && state.candidate.is_none()
                && state.retiring.is_empty()
            {
                drop(state);
                let mut owner = self.inner.measured.lock().expect("setup owner");
                if owner.scene_run.as_deref() == Some(run_id) {
                    owner.scene_run = None;
                    owner.cleanup_failed = false;
                }
                return Ok(());
            }
            drop(state);
            if Instant::now() >= deadline {
                self.inner.measured.lock().expect("setup owner").cleanup_failed = true;
                return Err("Scene cleanup is still occupied; explicitly retry Stop".into());
            }
            thread::sleep(Duration::from_millis(10));
        }
    }
    pub fn begin_setup_operation(&self) -> Result<SetupLease, String> {
        let mut owner = self.inner.measured.lock().expect("setup owner");
        if owner.worker || owner.cleanup_failed {
            return Err(
                "Another setup/model/discovery/benchmark/report operation is still running or retiring"
                    .into(),
            );
        }
        owner.generation = owner
            .generation
            .checked_add(1)
            .ok_or("Setup generation exhausted")?;
        let id = Uuid::new_v4().to_string();
        let cancelled = Arc::new(AtomicBool::new(false));
        owner.operation = Some(id.clone());
        owner.cancelled = Some(cancelled.clone());
        owner.receipt = None;
        owner.worker = true;
        Ok(SetupLease {
            manager: self.clone(),
            id,
            generation: owner.generation,
            cancelled,
        })
    }
    pub fn retire_setup_operation(&self) -> Result<(), String> {
        let mut owner = self.inner.measured.lock().expect("setup owner");
        owner.generation = owner
            .generation
            .checked_add(1)
            .ok_or("Setup generation exhausted")?;
        owner.receipt = None;
        if let Some(cancelled) = &owner.cancelled {
            cancelled.store(true, Ordering::Release);
        }
        let retry = (!owner.worker && owner.cleanup_failed)
            .then(|| owner.scene_run.clone())
            .flatten();
        drop(owner);
        if self.inner.evaluation_worker.load(Ordering::Acquire) {
            let evaluation_id = self.evaluation_snapshot().evaluation_id;
            if let Some(id) = evaluation_id {
                self.cancel_evaluation(&id).map_err(|error| error.to_string())?;
            }
        }
        if let Some(run) = retry {
            self.stop_measured_scene(&run)?;
        }
        Ok(())
    }
    pub fn setup_operation_busy(&self) -> bool {
        let owner = self.inner.measured.lock().expect("setup owner");
        owner.worker || owner.cleanup_failed
    }
    pub fn setup_operation_status(&self) -> Result<bool, String> {
        let owner = self.inner.measured.lock().expect("setup owner");
        if owner.cleanup_failed {
            return Err("Owned engine cleanup remains occupied. Explicitly Cancel again to retry Stop; no new operation is allowed.".into());
        }
        Ok(owner.worker)
    }
    fn report_saved(&self, id: &str) -> Result<SavedEngineProfile, String> {
        self.inner
            .catalog
            .get(id)
            .ok_or_else(|| "Saved report target no longer exists".into())
    }
    fn report_current(&self, saved: &SavedEngineProfile, lease: &SetupLease) -> Result<(), String> {
        lease.check()?;
        if self.inner.catalog.get(&saved.profile_id).as_ref() != Some(saved) {
            return Err("Saved target, policy or scene revision changed".into());
        }
        Ok(())
    }
    pub fn review_measured_report(&self, target: &str, bytes: &[u8]) -> Result<MeasuredReviewDto, String> {
        let lease = self.begin_setup_operation()?;
        let saved = self.report_saved(target)?;
        let parsed = report::parse(bytes)?;
        let started = Instant::now();
        let mut reasons = Vec::new();
        let qualification = (|| {
            let spec = self.measured_baseline(&saved, parsed.scene)?;
            let gpu = fingerprint::sample_gpu()?;
            let (fp, baseline) = fingerprint::capture(&spec, parsed.scene, &gpu, &|| lease.check().is_err())?;
            fingerprint::matches(&parsed, &fp, &baseline)
        })();
        if let Err(reason) = qualification {
            reasons.push(reason);
        }
        self.report_current(&saved, &lease)?;
        let review = MeasuredReviewDto {
            operation_id: lease.id.clone(),
            generation: lease.generation,
            target_id: saved.profile_id.clone(),
            target_name: saved.profile.name.clone(),
            scene: parsed.scene,
            profile_revision: crate::thread_policy::input_revision(&saved.profile),
            policy_revision: saved.thread_policy.revision,
            overlay_revision: saved.measured_scenes.revision,
            baseline_parameters: parsed.baseline.clone(),
            candidate_parameters: parsed.candidate.clone(),
            assessment: parsed.assess(),
            qualification_reasons: reasons,
            cold_verification_seconds: started.elapsed().as_secs_f64(),
            report: parsed.raw,
        };
        let mut owner = self.inner.measured.lock().expect("setup owner");
        if owner.operation.as_deref() != Some(&lease.id)
            || owner.generation != lease.generation
            || lease.cancelled.load(Ordering::Acquire)
        {
            return Err("Review retired before publication".into());
        }
        owner.receipt = Some(review.clone());
        drop(owner);
        Ok(review)
    }
    /// Called before taking ENGINE_CATALOG_WRITES. Never trusts a renderer-supplied report.
    pub fn validate_measured_apply(
        &self,
        id: &str,
        generation: u64,
    ) -> Result<(SetupLease, MeasuredReviewDto), String> {
        let review = {
            let owner = self.inner.measured.lock().expect("setup owner");
            let receipt = owner
                .receipt
                .as_ref()
                .ok_or("Review receipt retired; import again")?;
            if receipt.operation_id != id
                || receipt.generation != generation
                || owner.generation != generation
            {
                return Err("Review receipt generation changed".into());
            }
            receipt.clone()
        };
        if !review.assessment.eligible || !review.qualification_reasons.is_empty() {
            return Err("Report is not eligible to Apply".into());
        }
        let lease = self.begin_setup_operation()?;
        let saved = self.report_saved(&review.target_id)?;
        if crate::thread_policy::input_revision(&saved.profile) != review.profile_revision
            || saved.thread_policy.revision != review.policy_revision
            || saved.measured_scenes.revision != review.overlay_revision
        {
            return Err("Saved inputs changed after Review".into());
        }
        let parsed = report::parse(&serde_json::to_vec(&review.report).map_err(|e| e.to_string())?)?;
        let spec = self.measured_baseline(&saved, review.scene)?;
        let gpu = fingerprint::sample_gpu()?;
        let (fp, baseline) = fingerprint::capture(&spec, review.scene, &gpu, &|| lease.check().is_err())?;
        fingerprint::matches(&parsed, &fp, &baseline)?;
        self.report_current(&saved, &lease)?;
        Ok((lease, review))
    }
    pub fn measured_baseline(
        &self,
        saved: &SavedEngineProfile,
        scene: MeasuredSceneDto,
    ) -> Result<crate::CommandSpec, String> {
        let mut profile = saved.profile.clone();
        match (&profile.adapter,scene){
            (app_model::EngineAdapterSettings::KataGoGtp(settings),MeasuredSceneDto::WholeGame)=>{profile.adapter=app_model::EngineAdapterSettings::KataGoAnalysis(settings.clone());}
            (app_model::EngineAdapterSettings::KataGoGtp(_),MeasuredSceneDto::Live)|(app_model::EngineAdapterSettings::KataGoAnalysis(_),MeasuredSceneDto::WholeGame)=>{},
            _=>return Err("Unsupported report scene: live requires local GTP; whole-game requires a new independent local JSONL Run, never reuse-current or SSH".into()),
        }
        let mut spec = build_command_spec(&profile).map_err(|e| e.to_string())?;
        let mut run = starting_run(saved);
        run.profile_snapshot = profile;
        run.adapter_kind = run.profile_snapshot.adapter_kind();
        crate::thread_policy::apply(
            &run,
            &mut spec,
            self.inner.config.managed_resources_root.as_deref(),
        )?;
        Ok(spec)
    }
}

pub fn apply_to_catalog(
    settings: &mut crate::EngineProfilesSettings,
    review: &MeasuredReviewDto,
    lease: &SetupLease,
) -> Result<(), String> {
    lease.check()?;
    let record = settings
        .profiles
        .iter_mut()
        .find(|r| r.id == review.target_id)
        .ok_or("Report target deleted")?;
    if crate::thread_policy::input_revision(&record.profile) != review.profile_revision
        || record.thread_policy.revision != review.policy_revision
        || record.measured_scenes.revision != review.overlay_revision
    {
        return Err("Report target changed before atomic Apply".into());
    }
    let accepted = AcceptedMeasuredSceneDto {
        report: review.report.clone(),
        profile_revision: review.profile_revision.clone(),
        policy_revision: review.policy_revision,
        accepted_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_secs()
            .to_string(),
    };
    record.measured_scenes.revision = record
        .measured_scenes
        .revision
        .checked_add(1)
        .ok_or("Scene revision exhausted")?;
    match review.scene {
        MeasuredSceneDto::Live => record.measured_scenes.live = Some(accepted),
        MeasuredSceneDto::WholeGame => record.measured_scenes.whole_game = Some(accepted),
    }
    Ok(())
}
pub fn restore_catalog(
    settings: &mut crate::EngineProfilesSettings,
    target: &str,
    expected_revision: u64,
    lease: &SetupLease,
) -> Result<(), String> {
    lease.check()?;
    let record = settings
        .profiles
        .iter_mut()
        .find(|r| r.id == target)
        .ok_or("Report target deleted")?;
    if record.measured_scenes.revision != expected_revision {
        return Err("Scene revision changed before Restore".into());
    }
    record.measured_scenes.revision = record
        .measured_scenes
        .revision
        .checked_add(1)
        .ok_or("Scene revision exhausted")?;
    record.measured_scenes.live = None;
    record.measured_scenes.whole_game = None;
    Ok(())
}

pub(super) fn apply_launch(
    inner: &Arc<Inner>,
    run: &EngineRunDto,
    spec: &mut crate::CommandSpec,
    capture: &AttemptCapture,
    operation: u64,
    background: bool,
) {
    let Some(scene) = run.measured_scene else {
        return;
    };
    {
        let owner = inner.measured.lock().expect("setup owner");
        if let Some((target, progress, parameters)) = &owner.measurement {
            if target == &run.profile_id
                && progress.scene == scene
                && owner.worker
                && owner
                    .cancelled
                    .as_ref()
                    .is_some_and(|flag| !flag.load(Ordering::Acquire))
            {
                overlay(spec, scene, parameters);
                capture.record(
                    "measured-report",
                    "Explicit leased application measurement row; accepted overlays excluded",
                );
                return;
            }
        }
    }
    let accepted = match scene {
        MeasuredSceneDto::Live => &run.measured_scenes_snapshot.live,
        MeasuredSceneDto::WholeGame => &run.measured_scenes_snapshot.whole_game,
    };
    let Some(accepted) = accepted else {
        return;
    };
    let started = Instant::now();
    let result = (|| -> Result<report::Report, String> {
        if crate::thread_policy::input_revision(&run.profile_snapshot) != accepted.profile_revision
            || run.thread_policy_snapshot.revision != accepted.policy_revision
        {
            return Err("Manual profile or policy edit invalidated accepted report".into());
        }
        let parsed = report::parse(&serde_json::to_vec(&accepted.report).map_err(|e| e.to_string())?)?;
        if parsed.scene != scene || !parsed.assess().eligible {
            return Err("Accepted report does not qualify for this launch scene".into());
        }
        let gpu = fingerprint::sample_gpu()?;
        let (fp, baseline) = fingerprint::capture(spec, scene, &gpu, &|| {
            !inner.preparation_current(operation, run, background)
        })?;
        fingerprint::matches(&parsed, &fp, &baseline)?;
        let latest = inner
            .catalog
            .get(&run.profile_id)
            .ok_or("Saved report target deleted during launch")?;
        if latest.profile != run.profile_snapshot
            || latest.thread_policy != run.thread_policy_snapshot
            || latest.measured_scenes != run.measured_scenes_snapshot
        {
            return Err("Saved report target changed during launch verification".into());
        }
        Ok(parsed)
    })();
    capture.record(
        "measured-report",
        &format!("coldVerificationSeconds={:.6}", started.elapsed().as_secs_f64()),
    );
    match result {
        Ok(parsed) => {
            overlay(spec, scene, &parsed.candidate);
            capture.record("measured-report","Accepted matching scene overlay applied; experimental, sampled total GPU is not peak/per-process/RAM");
        }
        Err(reason) => capture.record(
            "measured-report",
            &format!("Overlay not applied; original launch policy retained: {reason}"),
        ),
    }
}

fn overlay(
    spec: &mut crate::CommandSpec,
    scene: MeasuredSceneDto,
    parameters: &std::collections::BTreeMap<String, u32>,
) {
    let mut overrides = parameters
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>();
    if scene == MeasuredSceneDto::WholeGame {
        overrides.push("numSearchThreads=".into());
    }
    if let Some(index) = spec
        .args
        .iter()
        .rposition(|arg| matches!(arg.as_str(), "-override-config" | "--override-config"))
    {
        if let Some(value) = spec.args.get_mut(index + 1) {
            if !value.is_empty() {
                value.push(',');
            }
            value.push_str(&overrides.join(","));
        }
    } else {
        spec.args.extend(["-override-config".into(), overrides.join(",")]);
    }
}

impl Inner {
    pub(super) fn measured_note_search_progress(&self, run_id: &str, job_id: &str, generation: u64) {
        let mut owner = self.measured.lock().expect("setup owner");
        if owner.scene_run.as_deref() == Some(run_id)
            && owner
                .cancelled
                .as_ref()
                .is_some_and(|flag| !flag.load(Ordering::Acquire))
            && !owner
                .search_progress
                .as_ref()
                .is_some_and(|(job, document)| job == job_id && *document == generation)
        {
            owner.search_progress = Some((job_id.to_owned(), generation));
        }
    }
}
