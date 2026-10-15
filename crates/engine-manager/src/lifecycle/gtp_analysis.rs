//! Named KataGo GTP analysis on the existing Run, reader, command IDs and exact restore.
use super::*;
use app_model::{AnalysisFrameDto, ComputeBudgetDto, EngineGtpFactsDto, GameMoveRequestDto};
use std::sync::mpsc::{RecvTimeoutError, TryRecvError};

mod focus;

pub(super) fn capabilities(
    run: &EngineRunDto,
    facts: &EngineGtpFactsDto,
) -> Option<EngineAnalysisCapabilitiesDto> {
    // Discovery alone is not implementation support: this codec is qualified against 1.18.2.
    let version = facts.version.split('+').next()?;
    if run.adapter_kind != EngineBackend::KataGoGtp
        || facts.name != "KataGo"
        || version != "1.18.2"
        || !["kata-analyze", "stop"]
            .iter()
            .all(|command| facts.commands.iter().any(|value| value == command))
    {
        return None;
    }
    Some(EngineAnalysisCapabilitiesDto {
        selected_node_analysis: true,
        continuous_analysis: true,
        whole_game_analysis: false,
        candidates: true,
        pv: true,
        winrate: true,
        root_score: true,
        ownership: true,
        policy: false,
        visits_limit: true,
        protocol_cancel: true,
    })
}

struct Command {
    id: u32,
    response: Receiver<String>,
    written: Receiver<Result<(), EngineFailureDto>>,
}

struct AnalysisError {
    kind: EngineFailureKind,
    message: AnalysisFailureText,
}
enum AnalysisFailureText {
    Public(FailureText),
    Dynamic(String),
    Context(&'static str, String),
}
impl From<String> for AnalysisError {
    fn from(message: String) -> Self {
        Self {
            kind: EngineFailureKind::Protocol,
            message: AnalysisFailureText::Dynamic(message),
        }
    }
}
impl From<&'static str> for AnalysisError {
    fn from(message: &'static str) -> Self {
        Self {
            kind: EngineFailureKind::Protocol,
            message: AnalysisFailureText::Public(message.into()),
        }
    }
}
fn response_error(error: RecvTimeoutError, context: &'static str) -> AnalysisError {
    AnalysisError {
        kind: if error == RecvTimeoutError::Timeout {
            EngineFailureKind::Timeout
        } else {
            EngineFailureKind::Protocol
        },
        message: AnalysisFailureText::Context(context, error.to_string()),
    }
}

fn retain_same_tree_maps(frame: &mut AnalysisFrameDto, previous: Option<&AnalysisFrameDto>) {
    let Some(previous) = previous else { return; };
    if frame.ownership.is_none() { frame.ownership.clone_from(&previous.ownership); }
    if frame.policy.is_none() { frame.policy.clone_from(&previous.policy); }
    for candidate in &mut frame.candidates {
        if candidate.ownership.is_none() {
            if let Some(old) = previous.candidates.iter().find(|old| old.vertex == candidate.vertex) {
                candidate.ownership.clone_from(&old.ownership);
            }
        }
    }
}

impl ForegroundEngineManager {
    pub(super) fn lock_gtp_writer<'a>(&'a self, stdin: &'a Mutex<Option<std::process::ChildStdin>>, deadline: Instant) -> Result<(std::sync::MutexGuard<'a, ManagerState>, std::sync::MutexGuard<'a, Option<std::process::ChildStdin>>), String> {
        loop {
            if Instant::now() >= deadline { return Err("GTP write boundary timed out".into()); }
            let state = self.lock();
            match stdin.try_lock() {
                Ok(writer) => return Ok((state, writer)),
                Err(std::sync::TryLockError::Poisoned(_)) => return Err("GTP stdin lock poisoned".into()),
                Err(std::sync::TryLockError::WouldBlock) => drop(state),
            }
            thread::sleep(Duration::from_millis(1));
        }
    }

    // Run IDs are fresh per process and have precisely one immutable stdout pump. A retired
    // Run cannot register again: both registration and writing require that live incarnation.
    fn begin_analysis_command(
        &self,
        started: &AnalysisJobStartedDto,
        body: &str,
        stream: bool,
    ) -> Result<Command, String> {
        let (id, response, stdin) = {
            let mut state = self.lock();
            if owned_live(&state, &started.run_id).is_none() {
                return Err("analysis Run retired".into());
            }
            state.gtp_command_seq = state
                .gtp_command_seq
                .checked_add(1)
                .ok_or("GTP command IDs exhausted")?;
            let id = state.gtp_command_seq;
            let response = state
                .gtp_dispatch
                .register(&started.run_id, id, &started.job_id, stream)?;
            let stdin = engine_stdin(&state, &started.run_id).ok_or("analysis stdin retired")?;
            (id, response, stdin)
        };
        let (sender, written) = mpsc::sync_channel(1);
        let manager = self.clone();
        let run_id = started.run_id.clone();
        let job_id = started.job_id.clone();
        let payload = format!("{id} {body}");
        let deadline = Instant::now() + Duration::from_secs(5);
        thread::spawn(move || {
            let (mut state, mut guard) = match manager.lock_gtp_writer(&stdin, deadline) {
                Ok(guards) => guards,
                Err(message) => {
                    let _ = sender.send(Err(failure(EngineOperationDto::Job, EngineFailureKind::Timeout, FailureText::unscoped(&message), Some(&run_id), None, None)));
                    return;
                }
            };
            let valid = owned_live(&state, &run_id).is_some()
                && state.jobs.iter().any(|job| {
                    job.job_id == job_id
                        && job.run_id == run_id
                        && !job.terminal
                        && (!stream || job.disposition == JobDisposition::Running)
                });
            let focus_write = stream && state.local_focus.owns_job(&run_id, &job_id);
            let payload = if focus_write {
                format!("{id} {}", state.local_focus.command(state.local_focus.color))
            } else { payload };
            let result = if !valid {
                Err(failure(
                    EngineOperationDto::Job,
                    EngineFailureKind::Cancellation,
                    "queued analysis command retired before write".into(),
                    Some(&run_id),
                    None,
                    None,
                ))
            } else if let Some(writer) = guard.as_mut() {
                match write_jsonl(writer, &payload) {
                    Ok(()) => {
                        if focus_write {
                            let revision = state.local_focus.revision;
                            state.local_focus.applied(id, revision);
                        }
                        Ok(())
                    }
                    Err(_) => Err(failure(
                        EngineOperationDto::Job, EngineFailureKind::Protocol,
                        "analysis write failed".into(), Some(&run_id), None, None,
                    )),
                }
            } else {
                Err(failure(
                    EngineOperationDto::Job,
                    EngineFailureKind::Protocol,
                    "analysis stdin closed".into(),
                    Some(&run_id),
                    None,
                    None,
                ))
            };
            let _ = sender.send(result);
        });
        Ok(Command {
            id,
            response,
            written,
        })
    }

    pub(super) fn spawn_gtp_analysis(&self, started: AnalysisJobStartedDto, request: SelectedNodeJobRequest) {
        let manager = self.clone();
        thread::spawn(move || {
            let result = manager.run_gtp_analysis(&started, &request);
            if let Err(error) = result {
                if error.kind == EngineFailureKind::Cancellation {
                    let mut state = manager.lock();
                    let outcome = state
                        .jobs
                        .iter()
                        .find(|job| job.job_id == started.job_id)
                        .map(|job| stopped_outcome(job.disposition));
                    finish_selected_node_job(&mut state, &started, outcome, None, None);
                    publish_snapshot(&mut state);
                } else {
                    let message = match error.message {
                        AnalysisFailureText::Public(text) => text,
                        AnalysisFailureText::Dynamic(text) => {
                            manager.inner.diagnostic_text(&started.run_id, &text)
                        }
                        AnalysisFailureText::Context(context, text) => FailureText::from(context)
                            .then(": ")
                            .then(manager.inner.diagnostic_text(&started.run_id, &text)),
                    };
                    manager
                        .inner
                        .fail_analysis_run(&started.run_id, error.kind, message);
                }
            }
            manager
                .lock()
                .gtp_dispatch
                .retire(&started.run_id, &started.job_id);
            manager.reconcile_continuous();
        });
    }

    fn run_gtp_analysis(
        &self,
        started: &AnalysisJobStartedDto,
        request: &SelectedNodeJobRequest,
    ) -> Result<(), AnalysisError> {
        let position = request.exact_position.clone()?;
        let rules_request = GameMoveRequest {
            identity: GameMoveRequestDto {
                run_id: started.run_id.clone(),
                generation: started.generation,
                node_path: started.node_path.clone(),
                budget: ComputeBudgetDto {
                    deadline_ms: 30_000,
                    max_visits: None,
                },
            },
            position: position.clone(),
        };
        // The selected job owns admission throughout this existing ticket09 transaction and
        // the following stream. Other positioning transactions cannot interleave.
        let confirmed = self.confirm_analysis_rules(rules_request, &started.job_id);
        if let Err(error) = confirmed {
            let mut state = self.lock();
            if let Some(job) = state.jobs.iter().find(|job| job.job_id == started.job_id) {
                if job.disposition != JobDisposition::Running {
                    let outcome = stopped_outcome(job.disposition);
                    finish_selected_node_job(&mut state, started, Some(outcome), None, None);
                    publish_snapshot(&mut state);
                    return Ok(());
                }
            }
            state.continuous_safety_hold = true;
            finish_failed_job(&mut state, started, error);
            publish_snapshot(&mut state);
            return Ok(());
        }
        self.probe_focus(started, request)?;
        {
            let mut state = self.lock();
            let Some(job) = state
                .jobs
                .iter()
                .find(|job| job.job_id == started.job_id && !job.terminal)
            else {
                return Ok(());
            };
            if job.disposition != JobDisposition::Running {
                let outcome = stopped_outcome(job.disposition);
                finish_selected_node_job(&mut state, started, Some(outcome), None, None);
                publish_snapshot(&mut state);
                return Ok(());
            }
        }
        let color = match position.dto().to_play {
            app_model::PlayerColor::Black => "B",
            app_model::PlayerColor::White => "W",
        };
        // Freeze ownership for this stream invocation. A retired ordinary owner
        // must keep rejecting output rather than fall back to an unfenced stream.
        // Explicit leased Continuous jobs do not own the ordinary target.
        let focus_owned = {
            let mut state = self.lock();
            let eligible = started.mode == AnalysisJobModeDto::Continuous
                && !state.continuous_departing
                && admitting_run(&state.phase, &started.run_id).is_some()
                && state.continuous_target.as_ref().is_some_and(|target|
                    target.generation == started.generation && target.node_path == started.node_path);
            if eligible {
                state.local_focus.bind(started);
                state.local_focus.height = request.board_height;
                state.local_focus.color = color;
                state.local_focus.refresh_eligible = admitting_run(&state.phase, &started.run_id)
                    .is_some_and(|run| run.qualified_resource.as_deref()
                        .is_some_and(crate::thread_policy::direct_local));
                if state.local_focus.refresh_eligible {
                    state.local_focus.interval = state.analysis_refresh_interval;
                }
            }
            eligible
        };
        let began = Instant::now();
        let mut last: Option<AnalysisFrameDto> = None;
        let outcome = 'ordinary: loop {
            let stream = self.begin_analysis_command(
                started,
                &format!("kata-analyze {color} interval 10 rootInfo true ownership true"),
                true,
            )?;
            wait_written(&stream, Instant::now() + Duration::from_secs(5))?;
            {
                let mut state = self.lock();
                if let Some(job) = state.jobs.iter_mut().find(|job| job.job_id == started.job_id) {
                    if !job.submitted { job.submitted = true; job.submitted_at = began; }
                }
            }
            let header_deadline = Instant::now() + Duration::from_secs(5);
            let mut header = false;
            let ending = loop {
                let mut state = self.lock();
                let Some(job) = state.jobs.iter().find(|job| job.job_id == started.job_id && !job.terminal) else {
                    return Err("analysis job retired before its stream drain".into());
                };
                if job.disposition != JobDisposition::Running { break Some(stopped_outcome(job.disposition)); }
                let visits = last.as_ref().map_or(0, |frame| frame.visits);
                if let Some(budget) = job.continuous_budget {
                    if budget.continuous_visits_limit_enabled && visits >= budget.continuous_visits_limit { break Some(AnalysisJobOutcomeDto::VisitsLimited); }
                    if budget.continuous_time_limit_enabled && began.elapsed() >= Duration::from_secs(u64::from(budget.continuous_time_limit_seconds)) { break Some(AnalysisJobOutcomeDto::TimeLimited); }
                } else if request.query.max_visits.is_some_and(|limit| visits >= limit) { break Some(AnalysisJobOutcomeDto::Completed); }
                if state.local_focus.matches(started) {
                    if state.local_focus.observe(None, Instant::now()) { publish_snapshot(&mut state); }
                    if state.local_focus.revision != state.local_focus.applied { break None; }
                }
                drop(state);
                // unpark retains a token when cancellation/data races this empty check.
                // Every wake returns through the authoritative state/budget/focus checks above.
                match stream.response.try_recv() {
                    Ok(line) => {
                        if !header { require_stream_header(stream.id, &line)?; header = true; continue; }
                        if line.is_empty() { return Err("KataGo analysis ended without the requested stop barrier".into()); }
                        // Check the output owner before parsing and again before publishing.
                        {
                            let state = self.lock();
                            if focus_owned && (!state.local_focus.matches(started) || state.local_focus.output.map(|output| output.command) != Some(stream.id)) { continue; }
                        }
                        let mut frame = katago_protocol::parse_gtp_analysis(
                            &line, Uuid::parse_str(&started.job_id).map_err(|error| error.to_string())?,
                            request.board_width, request.board_height, position.dto().moves.len() as u32,
                        )?;
                        let mut state = self.lock();
                        if admitting_run(&state.phase, &started.run_id).is_none()
                            || (focus_owned && (!state.local_focus.matches(started)
                            || state.local_focus.output.map(|output| output.command) != Some(stream.id))) { continue; }
                        retain_same_tree_maps(&mut frame, last.as_ref());
                        let Some(job) = state.jobs.iter_mut().find(|job| job.job_id == started.job_id && !job.terminal && job.disposition == JobDisposition::Running) else { continue; };
                        job.state = AnalysisJobStateDto::Searching;
                        state.last_activity = Instant::now();
                        publish_event(&mut state, ForegroundEngineEventDto::Job { job: selected_node_job_event(started, AnalysisJobOutcomeDto::Progress, Some(frame.clone()), None) });
                        if focus_owned { state.local_focus.observe(Some(&frame), Instant::now()); }
                        publish_snapshot(&mut state);
                        last = Some(frame);
                    }
                    Err(TryRecvError::Empty) => {
                        if !header && Instant::now() >= header_deadline { return Err(response_error(RecvTimeoutError::Timeout, "KataGo analysis header")); }
                        thread::park_timeout(Duration::from_millis(20));
                    }
                    Err(TryRecvError::Disconnected) => return Err("KataGo analysis response disconnected".into()),
                }
            };
            let final_line = self.drain_analysis_stream(started, &stream, header)?;
            let mut state = self.lock();
            let final_eligible = focus_owned && state.local_focus.matches(started) && state.local_focus.output.is_some_and(|output| output.command == stream.id && output.lineage > 0);
            if final_eligible && state.jobs.iter().any(|job| job.job_id == started.job_id && job.disposition == JobDisposition::Running && !job.terminal) {
                if let Some(line) = final_line {
                    let mut frame = katago_protocol::parse_gtp_analysis(&line, Uuid::parse_str(&started.job_id).map_err(|error| error.to_string())?, request.board_width, request.board_height, position.dto().moves.len() as u32)?;
                    retain_same_tree_maps(&mut frame, last.as_ref());
                    publish_event(&mut state, ForegroundEngineEventDto::Job { job: selected_node_job_event(started, AnalysisJobOutcomeDto::Progress, Some(frame.clone()), None) });
                    last = Some(frame);
                }
            }
            if state.local_focus.matches(started) { state.local_focus.drained(); }
            let Some(job) = state.jobs.iter().find(|job| job.job_id == started.job_id && !job.terminal) else { return Ok(()); };
            if job.disposition != JobDisposition::Running { break 'ordinary stopped_outcome(job.disposition); }
            if let Some(outcome) = ending { break 'ordinary outcome; }
            if let Some(budget) = job.continuous_budget {
                if budget.continuous_visits_limit_enabled && last.as_ref().is_some_and(|frame| frame.visits >= budget.continuous_visits_limit) { break 'ordinary AnalysisJobOutcomeDto::VisitsLimited; }
                if budget.continuous_time_limit_enabled && began.elapsed() >= Duration::from_secs(u64::from(budget.continuous_time_limit_seconds)) { break 'ordinary AnalysisJobOutcomeDto::TimeLimited; }
            }
            // Only the next ordinary analyze is sent. Position, rules, job, start and
            // cumulative budget remain owned by this invocation, including drain time.
        };
        let mut state = self.lock();
        let frame = state.jobs.iter().find(|job| job.job_id == started.job_id)
            .filter(|job| job.disposition == JobDisposition::Running).and(last);
        finish_selected_node_job(&mut state, started, Some(outcome), frame, None);
        state.last_activity = Instant::now();
        publish_snapshot(&mut state);
        Ok(())
    }

    fn drain_analysis_stream(&self, started: &AnalysisJobStartedDto, stream: &Command, mut header: bool) -> Result<Option<String>, AnalysisError> {
        let deadline = Instant::now() + Duration::from_secs(5);
        let stop = self.begin_analysis_command(started, "stop", false)?;
        wait_written(&stop, deadline)?;
        let mut final_line = None;
        loop {
            let line = stream.response.recv_timeout(deadline.saturating_duration_since(Instant::now()))
                .map_err(|error| response_error(error, "KataGo stream drain"))?;
            if !header { require_stream_header(stream.id, &line)?; header = true; }
            else if line.is_empty() { break; }
            else if line.starts_with("info ") { final_line = Some(line); }
        }
        let mut decoder = crate::gtp::ResponseDecoder::new(stop.id);
        loop {
            let line = stop.response.recv_timeout(deadline.saturating_duration_since(Instant::now()))
                .map_err(|error| response_error(error, "KataGo Stop acknowledgement"))?;
            if let Some(reply) = decoder.push(&line)? {
                if !reply.success || !reply.body.trim().is_empty() { return Err("KataGo rejected the numbered Stop request".into()); }
                break;
            }
        }
        Ok(final_line)
    }
}

fn wait_written(command: &Command, deadline: Instant) -> Result<(), AnalysisError> {
    command
        .written
        .recv_timeout(deadline.saturating_duration_since(Instant::now()))
        .map_err(|error| response_error(error, "GTP analysis write"))?
        .map_err(|error| AnalysisError {
            kind: error.kind,
            message: AnalysisFailureText::Public(FailureText::from_failure(&error)),
        })
}
fn require_stream_header(id: u32, line: &str) -> Result<(), String> {
    if line.trim_end() == format!("={id}") {
        Ok(())
    } else {
        Err("KataGo analysis command did not acknowledge its numbered request".into())
    }
}
fn stopped_outcome(disposition: JobDisposition) -> AnalysisJobOutcomeDto {
    match disposition {
        JobDisposition::Superseded => AnalysisJobOutcomeDto::Superseded,
        JobDisposition::TimedOut => AnalysisJobOutcomeDto::Timeout,
        _ => AnalysisJobOutcomeDto::Cancelled,
    }
}

#[cfg(all(test, unix))]
mod writer_tests {
    use super::*;

    fn ready_fixture() -> (std::path::PathBuf, ForegroundEngineManager, EngineRunDto) {
        let directory = std::env::temp_dir().join(format!("focus-writer-{}", Uuid::new_v4()));
        std::fs::create_dir(&directory).unwrap();
        std::fs::write(directory.join("mode"), "analysis").unwrap();
        std::fs::write(directory.join("model"), "").unwrap();
        std::fs::write(directory.join("config"), "numSearchThreads=1\n").unwrap();
        use std::os::unix::fs::PermissionsExt;
        std::fs::write(directory.join("engine.py"), concat!("#!/usr/bin/env python3\n", include_str!("../../tests/fixtures/rules_process.py"))).unwrap();
        std::fs::set_permissions(directory.join("engine.py"), std::fs::Permissions::from_mode(0o755)).unwrap();
        let catalog = Arc::new(crate::InMemoryEngineProfileCatalog::new());
        catalog.upsert(SavedEngineProfile { profile_id: "queued".into(), thread_policy: Default::default(), measured_scenes: Default::default(), profile: app_model::EngineProfileDto {
            name: "queued writer".into(), program: directory.join("engine.py").to_string_lossy().into(), argv: vec![], working_dir: Some(directory.to_string_lossy().into()),
            adapter: app_model::EngineAdapterSettings::KataGoGtp(app_model::KataGoSettings { model_path: Some("model".into()), config_path: Some("config".into()), max_visits: 32 }),
        } });
        let manager = ForegroundEngineManager::new(catalog, ForegroundEngineConfig::for_tests());
        manager.start("queued").unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        let run = loop {
            if let ForegroundEngineLifecycleDto::Ready { run } = manager.snapshot().lifecycle { break run; }
            assert!(Instant::now() < deadline, "{:?}", manager.snapshot());
            thread::sleep(Duration::from_millis(2));
        };
        (directory, manager, run)
    }

    #[test]
    fn cancellation_at_wait_entry_wakes_registered_stream_without_protocol_data() {
        let (directory, manager, run) = ready_fixture();
        let job = manager.register_job(&run.run_id, AnalysisJobLane::SelectedNode, Arc::new(AnalysisCancelToken::new())).unwrap();
        let (registered, registered_rx) = mpsc::sync_channel(1);
        let enter = Arc::new(std::sync::Barrier::new(2));
        let worker_enter = enter.clone();
        let (done, done_rx) = mpsc::sync_channel(1);
        let worker_manager = manager.clone();
        let worker_run = run.run_id.clone();
        let worker_job = job.clone();
        let worker = thread::spawn(move || {
            let response = worker_manager.lock().gtp_dispatch.register(&worker_run, 900, &worker_job, true).unwrap();
            registered.send(()).unwrap();
            worker_enter.wait();
            // No fallback timeout: cancellation must leave a wake token even before wait entry.
            thread::park();
            let state = worker_manager.lock();
            let cancelled = state.jobs.iter().any(|job| job.job_id == worker_job && job.disposition == JobDisposition::Cancelled);
            done.send((cancelled, response.try_recv())).unwrap();
        });
        registered_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        assert!(manager.cancel_job("other-run", &job).is_err());
        assert!(manager.cancel_job(&run.run_id, "other-job").is_err());
        manager.cancel_job(&run.run_id, &job).unwrap();
        enter.wait();
        let result = done_rx.recv_timeout(Duration::from_secs(2));
        worker.thread().unpark(); // Cleanup even on the unfixed implementation.
        worker.join().unwrap();
        manager.teardown().unwrap();
        std::fs::remove_dir_all(directory).unwrap();
        let (cancelled, line) = result.expect("committed cancellation did not wake the registered stream");
        assert!(cancelled);
        assert_eq!(line, Err(mpsc::TryRecvError::Empty), "wake must not be a protocol response");
    }

    #[test]
    fn contended_writer_releases_state_and_retired_queued_analyze_never_emits() {
        let (directory, manager, run) = ready_fixture();
        let id = manager.register_job(&run.run_id, AnalysisJobLane::SelectedNode, Arc::new(AnalysisCancelToken::new())).unwrap();
        let started = { let state = manager.lock(); started_from(state.jobs.iter().find(|job| job.job_id == id).unwrap()) };
        let stdin = engine_stdin(&manager.lock(), &run.run_id).unwrap();
        let held_stdin = stdin.lock().unwrap();
        let command = manager.begin_analysis_command(&started, "kata-analyze B interval 10 rootInfo true", true).unwrap();
        let (entered, entered_rx) = mpsc::sync_channel(1);
        let (finished, finished_rx) = mpsc::sync_channel(1);
        let cancelling = manager.clone();
        let run_id = run.run_id.clone();
        let cancellation = thread::spawn(move || {
            let mut state = cancelling.lock();
            entered.send(()).unwrap();
            cancel_jobs_for_run(&mut state, &run_id);
            finished.send(()).unwrap();
        });
        // State remains available while the writer is contending on stdin.
        entered_rx.recv_timeout(Duration::from_secs(1)).unwrap();
        drop(held_stdin);
        finished_rx.recv_timeout(Duration::from_secs(1)).unwrap();
        assert!(command.written.recv_timeout(Duration::from_secs(1)).unwrap().is_err());
        cancellation.join().unwrap();
        manager.teardown().unwrap();
        let trace = std::fs::read_to_string(directory.join("trace")).unwrap();
        assert!(!trace.contains("kata-analyze B"), "{trace}");
        std::fs::remove_dir_all(directory).unwrap();
    }
}
