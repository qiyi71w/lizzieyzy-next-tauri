use super::*;
use app_model::ComputeBudgetDto;

pub(super) struct MatchReservation {
    owner: String,
    pub(super) runs: Vec<EngineRunDto>,
    pub(super) ready: bool,
    pub(super) committed: bool,
    pub(super) sealed: bool,
    preparing: bool,
    pub(super) pausing: bool,
    /// Run IDs whose in-flight GTP move Pause cancelled by confirmed kill/reap. They stay listed
    /// in `runs` (reservation and profile protection) until an explicit Resume replaces them.
    pub(super) retired: Vec<String>,
    pub(super) failure: Option<EngineFailureDto>,
}

fn reservation_failure(kind: EngineFailureKind, message: &str) -> EngineFailureDto {
    failure(EngineOperationDto::Job, kind, message.into(), None, None, None)
}

fn run_failure(run: &EngineRunDto, kind: EngineFailureKind, message: &str) -> EngineFailureDto {
    failure(
        EngineOperationDto::Job,
        kind,
        message.into(),
        Some(&run.run_id),
        Some(&run.profile_id),
        None,
    )
}

/// Adapter budget shape needs no process: KataGo enforces positive visits within its komi range;
/// GenericGtp cannot enforce visits, so a visits request is refused rather than ignored.
fn budget_refusal(
    run: &EngineRunDto,
    budget: ComputeBudgetDto,
    position: &sgf::ExactPosition,
) -> Option<&'static str> {
    if budget.deadline_ms == 0 {
        return Some("Match moves require a positive hard deadline.");
    }
    match run.adapter_kind {
        EngineBackend::KataGoAnalysis
            if budget.max_visits.is_none_or(|visits| visits == 0) || position.dto().komi.abs() > 400.0 =>
        {
            Some("KataGo match moves require positive max_visits and komi in [-400,400].")
        }
        EngineBackend::GenericGtp if budget.max_visits.is_some() => {
            Some("GTP cannot enforce a visits budget; GTP sides use only the hard deadline.")
        }
        _ => None,
    }
}

pub(super) fn require_unreserved(state: &ManagerState) -> Result<(), EngineFailureDto> {
    if state.match_reservation.is_some() {
        return Err(reservation_failure(EngineFailureKind::Occupied,
            "A match owns the foreground engine; stop the match before changing engines or starting analysis."));
    }
    Ok(())
}

fn require_owner<'a>(state: &'a ManagerState, owner: &str) -> Result<&'a MatchReservation, EngineFailureDto> {
    state
        .match_reservation
        .as_ref()
        .filter(|reservation| reservation.owner == owner)
        .ok_or_else(|| {
            reservation_failure(
                EngineFailureKind::InvalidState,
                "The match reservation is no longer owned by this session.",
            )
        })
}

pub(super) fn require_move_owner(
    state: &ManagerState,
    owner: Option<&str>,
    run_id: &str,
) -> Result<(), EngineFailureDto> {
    let Some(owner) = owner else {
        return require_unreserved(state);
    };
    let reservation = require_owner(state, owner)?;
    if !reservation.committed
        || reservation.sealed
        || reservation.pausing
        || !reservation.runs.iter().any(|run| run.run_id == run_id)
    {
        return Err(reservation.failure.clone().unwrap_or_else(|| {
            reservation_failure(
                EngineFailureKind::Cancellation,
                "The match move belongs to a retired or uncommitted engine run.",
            )
        }));
    }
    Ok(())
}

pub(super) fn assert_reserved_profile_deletable(
    state: &ManagerState,
    profile_id: &str,
) -> Result<(), EngineFailureDto> {
    if state
        .match_reservation
        .as_ref()
        .is_some_and(|reservation| reservation.runs.iter().any(|run| run.profile_id == profile_id))
        || state.retiring.iter().any(|(run, _)| run.profile_id == profile_id)
    {
        return Err(reservation_failure(
            EngineFailureKind::Occupied,
            "This profile is owned by a match candidate or a retiring engine process.",
        ));
    }
    Ok(())
}

// Exit monitoring follows owned run identity, including the idle PK side and staged candidates.
pub(super) fn handle_reserved_exit(state: &mut ManagerState, run_id: &str, exit_code: Option<i32>) -> bool {
    let Some(reservation) = state.match_reservation.as_ref() else {
        return false;
    };
    let Some(run) = reservation.runs.iter().find(|run| run.run_id == run_id).cloned() else {
        return false;
    };
    let committed = reservation.committed;
    let mut published = run_failure(
        &run,
        EngineFailureKind::ProcessExit,
        &format!("engine process exited unexpectedly; exit_code={exit_code:?}"),
    );
    published.operation = EngineOperationDto::UnexpectedExit;
    if let Some(slot) = state
        .game_move
        .as_ref()
        .filter(|slot| slot.identity.run_id == run_id)
    {
        published.job_id = Some(slot.identity.job_id.clone());
    }
    let reservation = state.match_reservation.as_mut().unwrap();
    reservation.sealed = true;
    reservation.failure.get_or_insert_with(|| published.clone());
    state.game_move_publication = None;
    if let Some(active) = state.game_move.as_ref().map(|slot| slot.identity.run_id.clone()) {
        if active == run_id {
            game_move::fail_move_for_run(state, &published);
        } else {
            game_move::seal_move_for_run(state, &active);
        }
    }
    // try_wait already confirmed exit; removing this process does not release the reservation.
    take_owned_live(state, run_id);
    if committed {
        state.phase = Phase::Error {
            run,
            failure: published.clone(),
        };
    }
    publish_snapshot(state);
    publish_event(state, ForegroundEngineEventDto::Failure { failure: published });
    true
}

pub(super) fn handle_reserved_stdout_failure(
    state: &mut ManagerState,
    run_id: &str,
    deadline: Instant,
    kind: EngineFailureKind,
    message: &str,
) -> bool {
    let Some(reservation) = state.match_reservation.as_ref() else {
        return false;
    };
    if !reservation.committed {
        return false;
    }
    let Some(run) = reservation.runs.iter().find(|run| run.run_id == run_id).cloned() else {
        return false;
    };
    if owned_live(state, run_id).is_none() {
        return true;
    }
    if let Some(Ok(Some(status))) = owned_live_mut(state, run_id).map(|live| live.child.try_wait()) {
        return handle_reserved_exit(state, run_id, status.code());
    }
    let mut published = run_failure(&run, kind, message);
    published.operation = EngineOperationDto::UnexpectedExit;
    if let Some(slot) = state
        .game_move
        .as_ref()
        .filter(|slot| slot.identity.run_id == run_id)
    {
        published.job_id = Some(slot.identity.job_id.clone());
    }
    state.game_move_publication = None;
    if let Some(active) = state.game_move.as_ref().map(|slot| slot.identity.run_id.clone()) {
        if active == run_id {
            game_move::fail_move_for_run(state, &published);
        } else {
            game_move::seal_move_for_run(state, &active);
        }
    }
    let cleanup = owned_live_mut(state, run_id).map(|live| terminate_process(live, deadline));
    if matches!(&cleanup, Some(Ok(()))) {
        take_owned_live(state, run_id);
    }
    if let Some(Err(error)) = cleanup {
        published.message.push_str(&format!(
            "; process cleanup was not confirmed: {error}; retry Stop"
        ));
    }
    let reservation = state.match_reservation.as_mut().unwrap();
    reservation.sealed = true;
    reservation.failure.get_or_insert_with(|| published.clone());
    state.phase = Phase::Error {
        run,
        failure: published.clone(),
    };
    publish_snapshot(state);
    publish_event(state, ForegroundEngineEventDto::Failure { failure: published });
    true
}

impl ForegroundEngineManager {
    /// Reserves admission only. Process IO and cancellation happen during prepare.
    pub fn reserve_match(&self, owner: &str) -> Result<(), EngineFailureDto> {
        let mut state = self.lock();
        require_unreserved(&state)?;
        game_move::require_idle_move(&state)?;
        super::runtime_control::require_idle(&state)?;
        if owner.is_empty()
            || !matches!(state.phase, Phase::NoEngine { .. } | Phase::Ready(_))
            || state.candidate.is_some()
            || !state.match_residents.is_empty()
            || state.finite_admission_pending
            || state.continuous_departing
        {
            return Err(reservation_failure(
                EngineFailureKind::InvalidState,
                "Match reservation requires a nonempty session and a stable Ready or unloaded foreground.",
            ));
        }
        evaluation::yield_to_foreground(&mut state)?;
        self.yield_preloads_locked(&mut state);
        state.match_reservation = Some(MatchReservation {
            owner: owner.into(),
            runs: Vec::new(),
            ready: false,
            committed: false,
            sealed: false,
            preparing: false,
            pausing: false,
            retired: Vec::new(),
            failure: None,
        });
        state.game_move_publication = None;
        state.continuous_safety_hold = true;
        publish_snapshot(&mut state);
        Ok(())
    }

    pub fn match_reservation_owner(&self) -> Option<String> {
        self.lock()
            .match_reservation
            .as_ref()
            .map(|reservation| reservation.owner.clone())
    }

    /// Always stages isolated, fully qualified candidates; foreground changes only at commit.
    pub fn prepare_reserved_match(
        &self,
        owner: &str,
        profile_id: &str,
        position: &sgf::ExactPosition,
        budget: ComputeBudgetDto,
    ) -> Result<EngineRunDto, EngineFailureDto> {
        self.prepare_reserved_participants(owner, &[(profile_id, budget)], position, &mut 0)
            .map(|mut runs| runs.remove(0))
    }

    /// Returns Black then White. Equal profile IDs still spawn independently owned processes.
    pub fn prepare_reserved_pk(
        &self,
        owner: &str,
        participants: [(&str, ComputeBudgetDto); 2],
        position: &sgf::ExactPosition,
    ) -> Result<[EngineRunDto; 2], (app_model::PlayerColor, EngineFailureDto)> {
        let mut failed_side = 0;
        self.prepare_reserved_participants(owner, &participants, position, &mut failed_side)
            .map(|runs| runs.try_into().expect("PK has exactly two participants"))
            .map_err(|error| {
                (
                    if failed_side == 0 {
                        app_model::PlayerColor::Black
                    } else {
                        app_model::PlayerColor::White
                    },
                    error,
                )
            })
    }

    fn prepare_reserved_participants(
        &self,
        owner: &str,
        participants: &[(&str, ComputeBudgetDto)],
        position: &sgf::ExactPosition,
        failed_side: &mut usize,
    ) -> Result<Vec<EngineRunDto>, EngineFailureDto> {
        let (operation, candidates) = {
            let mut state = self.lock();
            let reservation = require_owner(&state, owner)?;
            if reservation.sealed
                || reservation.committed
                || reservation.preparing
                || !reservation.runs.is_empty()
            {
                return Err(reservation_failure(
                    EngineFailureKind::InvalidState,
                    "The match reservation already has a candidate or is stopping.",
                ));
            }
            let mut runs = Vec::with_capacity(participants.len());
            let mut refusal = None;
            // Capture and validate every profile before cancellation, assets checks or spawning.
            for (index, (profile_id, budget)) in participants.iter().enumerate() {
                *failed_side = index;
                let Some(saved) = self.inner.catalog.get(profile_id) else {
                    refusal = Some(failure(
                        EngineOperationDto::Start,
                        EngineFailureKind::InvalidState,
                        "The selected saved engine profile no longer exists.".into(),
                        None,
                        Some(profile_id),
                        None,
                    ));
                    break;
                };
                let run = starting_run(&saved);
                // Adapter-level budget shape is pure; position qualification needs the Ready capability snapshot.
                if let Some(message) = budget_refusal(&run, *budget, position) {
                    refusal = Some(run_failure(
                        &run,
                        EngineFailureKind::UnsupportedCapability,
                        message,
                    ));
                    break;
                }
                runs.push(run);
            }
            if let Some(mut error) = refusal {
                // Validation failures release the empty reservation before returning.
                drop(state);
                if let Err(cleanup) = self.abort_reserved_match(owner) {
                    error
                        .message
                        .push_str(&format!("; reservation cleanup: {}", cleanup.message));
                }
                return Err(error);
            }
            state.operation += 1;
            state.operation_kind = EngineOperationDto::Start;
            let operation = state.operation;
            let reservation = state.match_reservation.as_mut().unwrap();
            reservation.preparing = true;
            reservation.runs = runs.clone();
            (operation, runs)
        };
        let prepared = (|| {
            *failed_side = 0;
            self.drain_reserved_analysis(owner)?;
            for (index, candidate) in candidates.iter().enumerate() {
                *failed_side = index;
                self.inner.start_resident(operation, candidate, true, false)?;
                let mut state = self.lock();
                let reservation = require_owner(&state, owner)?;
                if let Some(error) = &reservation.failure {
                    return Err(error.clone());
                }
                let run = reservation.runs[index].clone();
                if reservation.sealed || run.capability_snapshot.is_none() {
                    return Err(run_failure(
                        &run,
                        EngineFailureKind::Cancellation,
                        "Match preparation was stopped before the candidate became Ready.",
                    ));
                }
                game_move::validate_move_position(&run, position, participants[index].1)?;
                let process_id = owned_live(&state, &run.run_id)
                    .ok_or_else(|| {
                        run_failure(
                            &run,
                            EngineFailureKind::ProcessExit,
                            "The match candidate process is no longer owned.",
                        )
                    })?
                    .process_id;
                // Free the staging slot for the next independent process.
                if index + 1 < candidates.len() {
                    let live = state.candidate.take().expect("qualified candidate is owned");
                    state.match_residents.push(live);
                }
                drop(state);
                let inner = self.inner.clone();
                thread::spawn(move || inner.watch_exit(operation, run.run_id, process_id));
            }
            let state = self.lock();
            let reservation = require_owner(&state, owner)?;
            if let Some(error) = &reservation.failure {
                return Err(error.clone());
            }
            if reservation.sealed || !reservation.ready {
                return Err(reservation_failure(
                    EngineFailureKind::Cancellation,
                    "Match preparation was stopped.",
                ));
            }
            Ok(reservation.runs.clone())
        })();
        {
            let mut state = self.lock();
            if let Some(reservation) = state
                .match_reservation
                .as_mut()
                .filter(|reservation| reservation.owner == owner)
            {
                reservation.preparing = false;
                if prepared.is_err() {
                    reservation.ready = false;
                }
            }
        }
        match prepared {
            Ok(runs) => Ok(runs),
            Err(mut error) => {
                if let Some(index) = candidates
                    .iter()
                    .position(|run| error.run_id.as_ref() == Some(&run.run_id))
                {
                    *failed_side = index;
                }
                if let Err(cleanup) = self.abort_reserved_match(owner) {
                    error
                        .message
                        .push_str(&format!("; candidate cleanup: {}", cleanup.message));
                }
                Err(error)
            }
        }
    }

    fn drain_reserved_analysis(&self, owner: &str) -> Result<(), EngineFailureDto> {
        {
            let mut state = self.lock();
            require_owner(&state, owner)?;
            let jobs: Vec<_> = state
                .jobs
                .iter()
                .filter(|job| !job.terminal)
                .map(|job| (job.run_id.clone(), job.job_id.clone()))
                .collect();
            for (run_id, job_id) in jobs {
                self.request_job_stop_locked(&mut state, &run_id, &job_id, JobDisposition::Cancelled)?;
            }
            invalidate_analysis_task_locked(&mut state, "A match reserved the foreground engine.");
        }
        let deadline = Instant::now() + self.inner.config.stop_drain_timeout;
        loop {
            {
                let state = self.lock();
                if require_owner(&state, owner)?.sealed {
                    return Err(reservation_failure(
                        EngineFailureKind::Cancellation,
                        "Match preparation was stopped.",
                    ));
                }
                if state.jobs.iter().all(|job| job.terminal) {
                    return Ok(());
                }
            }
            if Instant::now() >= deadline {
                // Failed transactions must not let an analysis watchdog retire the old foreground.
                for job in &mut self.lock().jobs {
                    if !job.terminal && job.state == AnalysisJobStateDto::Stopping {
                        job.cancel_deadline = None;
                        job.cleanup_deadline = None;
                    }
                }
                return Err(reservation_failure(EngineFailureKind::Timeout,
                    "Existing analysis did not acknowledge cancellation; the old foreground was preserved. Stop analysis and retry."));
            }
            thread::sleep(Duration::from_millis(5));
        }
    }

    /// Checks every owned process and immutable profile under admission lock before installation.
    pub fn commit_reserved_match<T>(
        &self,
        owner: &str,
        install: impl FnOnce() -> Result<T, String>,
    ) -> Result<T, EngineFailureDto> {
        let mut state = self.lock();
        let reservation = require_owner(&state, owner)?;
        if let Some(error) = &reservation.failure {
            return Err(error.clone());
        }
        if reservation.sealed || reservation.preparing || reservation.committed || !reservation.ready {
            return Err(reservation_failure(
                EngineFailureKind::InvalidState,
                "Match commit requires its qualified Ready candidates.",
            ));
        }
        let runs = reservation.runs.clone();
        if !matches!(state.phase, Phase::Ready(_) | Phase::NoEngine { .. }) {
            return Err(reservation_failure(
                EngineFailureKind::InvalidState,
                "The foreground failed during match preparation; stop it and prepare again.",
            ));
        }
        for run in &runs {
            if !self
                .inner
                .catalog
                .get(&run.profile_id)
                .is_some_and(|saved| saved.profile == run.profile_snapshot)
            {
                return Err(run_failure(
                    run,
                    EngineFailureKind::InvalidState,
                    "The selected engine profile changed during preparation; prepare the match again.",
                ));
            }
            let live = owned_live_mut(&mut state, &run.run_id).ok_or_else(|| {
                run_failure(
                    run,
                    EngineFailureKind::ProcessExit,
                    "The match candidate process is no longer owned.",
                )
            })?;
            match live.child.try_wait() {
                Ok(None) => {}
                Ok(Some(_)) => {
                    return Err(run_failure(
                        run,
                        EngineFailureKind::ProcessExit,
                        "The match candidate exited before commit.",
                    ))
                }
                Err(error) => {
                    return Err(run_failure(
                        run,
                        EngineFailureKind::ProcessExit,
                        &format!("Cannot observe match candidate: {error}"),
                    ))
                }
            }
        }
        let result = install().map_err(|message| {
            reservation_failure(
                EngineFailureKind::InvalidState,
                &format!("Match installation failed before engine swap: {message}"),
            )
        })?;
        if let Some(old) = state.live.take() {
            let old_run = current_admitting_run(&state.phase).expect("live foreground has a run");
            state.retiring.push((old_run, Arc::new(Mutex::new(old))));
        }
        let primary = take_owned_live(&mut state, &runs[0].run_id).expect("validated primary process");
        if let Some(extra) = state.candidate.take() {
            state.match_residents.push(extra);
        }
        state.live = Some(primary);
        state.phase = Phase::Ready(runs[0].clone());
        state.match_reservation.as_mut().unwrap().committed = true;
        state.continuous_safety_hold = true;
        state.continuous_limited = None;
        state.continuous_paused = None;
        let operation = state.operation;
        let readers: Vec<_> = runs
            .iter()
            .filter_map(|run| {
                owned_live_mut(&mut state, &run.run_id)
                    .and_then(|live| live.stdout_rx.take())
                    .map(|reader| (run.clone(), reader))
            })
            .collect();
        publish_snapshot(&mut state);
        drop(state);
        for (run, stdout_rx) in readers {
            let inner = self.inner.clone();
            thread::spawn(move || match run.adapter_kind {
                EngineBackend::GenericGtp | EngineBackend::KataGoGtp => {
                    inner.pump_gtp_stdout(run.run_id, stdout_rx)
                }
                EngineBackend::KataGoAnalysis => inner.pump_stdout(operation, run.run_id, stdout_rx),
            });
        }
        let manager = self.clone();
        thread::spawn(move || {
            let _ = manager.reap_retiring_runs();
        });
        Ok(result)
    }

    /// Seals the current job, not the session. KataGo must acknowledge its targeted drain; a GTP
    /// move cannot be cancelled by protocol, so its run is killed and reaped and listed as retired.
    /// Returns the retired run IDs that only an explicit Resume may rebuild.
    pub fn pause_reserved_match(&self, owner: &str) -> Result<Vec<String>, EngineFailureDto> {
        {
            let mut state = self.lock();
            let reservation = require_owner(&state, owner)?;
            if let Some(error) = &reservation.failure {
                return Err(error.clone());
            }
            if !reservation.committed
                || reservation.sealed
                || reservation.preparing
                || reservation.runs.len() != 2
            {
                return Err(reservation_failure(
                    EngineFailureKind::InvalidState,
                    "Pause requires a committed PK match that is not rebuilding or stopping.",
                ));
            }
            state.match_reservation.as_mut().unwrap().pausing = true;
            state.game_move_publication = None;
            if let Some(run_id) = state.game_move.as_ref().map(|slot| slot.identity.run_id.clone()) {
                game_move::seal_move_for_run(&mut state, &run_id);
            }
        }
        let deadline = Instant::now() + self.inner.config.stop_drain_timeout * 3 + Duration::from_secs(1);
        loop {
            let mut state = self.lock();
            let reservation = require_owner(&state, owner)?;
            if let Some(error) = &reservation.failure {
                return Err(error.clone());
            }
            if reservation.sealed {
                return Err(reservation_failure(
                    EngineFailureKind::Cancellation,
                    "The match was stopped while pausing.",
                ));
            }
            if state.game_move.is_none() {
                let retired = reservation.retired.clone();
                let runs = reservation.runs.clone();
                // A side without an active move is never reclaimed; it must still own a living process.
                for run in runs.iter().filter(|run| !retired.contains(&run.run_id)) {
                    let live = owned_live_mut(&mut state, &run.run_id).ok_or_else(|| {
                        run_failure(
                            run,
                            EngineFailureKind::ProcessExit,
                            "The paused match no longer owns its retained live process.",
                        )
                    })?;
                    if !matches!(live.child.try_wait(), Ok(None)) {
                        return Err(run_failure(
                            run,
                            EngineFailureKind::ProcessExit,
                            "A paused match process exited.",
                        ));
                    }
                }
                state.match_reservation.as_mut().unwrap().pausing = false;
                publish_snapshot(&mut state);
                return Ok(retired);
            }
            if Instant::now() >= deadline {
                let active = state.game_move.as_ref().unwrap().identity.clone();
                let mut error = failure(
                    EngineOperationDto::Job,
                    EngineFailureKind::Timeout,
                    "Move cancellation/drain was not confirmed; the match remains reserved. Stop the match."
                        .into(),
                    Some(&active.run_id),
                    None,
                    None,
                )
                .with_job_id(&active.job_id);
                error.profile_id = reservation
                    .runs
                    .iter()
                    .find(|run| run.run_id == active.run_id)
                    .map(|run| run.profile_id.clone());
                let reservation = state.match_reservation.as_mut().unwrap();
                reservation.sealed = true;
                reservation.failure = Some(error.clone());
                return Err(error);
            }
            drop(state);
            thread::sleep(Duration::from_millis(5));
        }
    }

    /// Explicit Match Resume. Each run Pause retired is rebuilt from its immutable profile snapshot
    /// (never the saved catalog or a draft) under a new run identity, then must pass readiness,
    /// capability and exact-position admission again. Retained Ready runs keep their identity.
    /// The reservation stays exclusive; Stop/exit seal it, and a late Ready is never installed.
    /// Failure seals the reservation without retry; the owner ends the match through Stop.
    pub fn resume_reserved_pk(
        &self,
        owner: &str,
        position: &sgf::ExactPosition,
        budgets: [ComputeBudgetDto; 2],
    ) -> Result<[EngineRunDto; 2], (app_model::PlayerColor, EngineFailureDto)> {
        let side = |index: usize| {
            if index == 0 {
                app_model::PlayerColor::Black
            } else {
                app_model::PlayerColor::White
            }
        };
        let (operation, rebuild) = {
            let mut state = self.lock();
            let reservation = require_owner(&state, owner).map_err(|error| (side(0), error))?;
            let first_retired = reservation
                .runs
                .iter()
                .position(|run| reservation.retired.contains(&run.run_id))
                .unwrap_or(0);
            if let Some(error) = &reservation.failure {
                return Err((side(first_retired), error.clone()));
            }
            if !reservation.committed
                || reservation.sealed
                || reservation.pausing
                || reservation.preparing
                || reservation.runs.len() != 2
                || state.game_move.is_some()
                || state.candidate.is_some()
            {
                return Err((
                    side(first_retired),
                    reservation_failure(
                        EngineFailureKind::InvalidState,
                        "Resume requires a paused PK match whose cleanup has finished.",
                    ),
                ));
            }
            let rebuild: Vec<(usize, EngineRunDto)> = reservation
                .runs
                .iter()
                .enumerate()
                .filter(|(_, run)| reservation.retired.contains(&run.run_id))
                .map(|(index, old)| {
                    (
                        index,
                        EngineRunDto {
                            run_id: Uuid::new_v4().to_string(),
                            profile_id: old.profile_id.clone(),
                            adapter_kind: old.adapter_kind,
                            profile_snapshot: old.profile_snapshot.clone(),
                            capability_snapshot: None,
                            qualified_resource: None,
                        },
                    )
                })
                .collect();
            for (index, run) in &rebuild {
                if let Some(message) = budget_refusal(run, budgets[*index], position) {
                    return Err((
                        side(*index),
                        run_failure(run, EngineFailureKind::UnsupportedCapability, message),
                    ));
                }
            }
            if rebuild.is_empty() {
                return Ok(reservation
                    .runs
                    .clone()
                    .try_into()
                    .expect("PK has exactly two participants"));
            }
            state.operation += 1;
            state.operation_kind = EngineOperationDto::Start;
            let operation = state.operation;
            let reservation = state.match_reservation.as_mut().unwrap();
            reservation.preparing = true;
            reservation.retired.clear();
            for (index, run) in &rebuild {
                reservation.runs[*index] = run.clone();
            }
            publish_snapshot(&mut state);
            (operation, rebuild)
        };
        let mut failed = rebuild[0].0;
        let rebuilt = (|| {
            for (index, candidate) in &rebuild {
                failed = *index;
                self.inner.start_resident(operation, candidate, true, false)?;
                let mut state = self.lock();
                let reservation = require_owner(&state, owner)?;
                if let Some(error) = &reservation.failure {
                    return Err(error.clone());
                }
                let run = reservation.runs[*index].clone();
                if reservation.sealed || state.operation != operation || run.capability_snapshot.is_none() {
                    return Err(run_failure(
                        &run,
                        EngineFailureKind::Cancellation,
                        "Match Resume was stopped before the rebuilt engine became Ready.",
                    ));
                }
                game_move::validate_move_position(&run, position, budgets[*index])?;
                match owned_live_mut(&mut state, &run.run_id).map(|live| live.child.try_wait()) {
                    Some(Ok(None)) => {}
                    _ => {
                        return Err(run_failure(
                            &run,
                            EngineFailureKind::ProcessExit,
                            "The rebuilt match engine exited before it could be installed.",
                        ))
                    }
                }
                let mut live = take_owned_live(&mut state, &run.run_id).expect("observed rebuilt process");
                let stdout_rx = live.stdout_rx.take();
                let process_id = live.process_id;
                // The reaped side may have been the primary; restore a Ready foreground only then.
                if state.live.is_none() {
                    state.live = Some(live);
                    state.phase = Phase::Ready(run.clone());
                } else {
                    state.match_residents.push(live);
                }
                publish_snapshot(&mut state);
                drop(state);
                let inner = self.inner.clone();
                let run_id = run.run_id.clone();
                thread::spawn(move || inner.watch_exit(operation, run_id, process_id));
                if let Some(stdout_rx) = stdout_rx {
                    let inner = self.inner.clone();
                    thread::spawn(move || match run.adapter_kind {
                        EngineBackend::GenericGtp | EngineBackend::KataGoGtp => {
                            inner.pump_gtp_stdout(run.run_id, stdout_rx)
                        }
                        EngineBackend::KataGoAnalysis => inner.pump_stdout(operation, run.run_id, stdout_rx),
                    });
                }
            }
            let state = self.lock();
            let reservation = require_owner(&state, owner)?;
            if let Some(error) = &reservation.failure {
                return Err(error.clone());
            }
            if reservation.sealed {
                return Err(reservation_failure(
                    EngineFailureKind::Cancellation,
                    "Match Resume was stopped.",
                ));
            }
            Ok(reservation
                .runs
                .clone()
                .try_into()
                .expect("PK has exactly two participants"))
        })();
        let mut state = self.lock();
        if let Some(reservation) = state
            .match_reservation
            .as_mut()
            .filter(|reservation| reservation.owner == owner)
        {
            reservation.preparing = false;
            if let Err(error) = &rebuilt {
                reservation.sealed = true;
                reservation.failure.get_or_insert_with(|| error.clone());
            }
        }
        publish_snapshot(&mut state);
        rebuilt.map_err(|error| (side(failed), error))
    }

    pub fn abort_reserved_match(&self, owner: &str) -> Result<(), EngineFailureDto> {
        self.release_reserved_match(owner, false)
    }

    pub fn stop_reserved_match(&self, owner: &str) -> Result<(), EngineFailureDto> {
        self.release_reserved_match(owner, true)
    }

    fn release_reserved_match(&self, owner: &str, allow_committed: bool) -> Result<(), EngineFailureDto> {
        {
            let mut state = self.lock();
            if state.match_reservation.is_none() {
                return Ok(());
            }
            let reservation = require_owner(&state, owner)?;
            if reservation.committed && !allow_committed {
                return Err(reservation_failure(
                    EngineFailureKind::InvalidState,
                    "A committed match must be stopped, not aborted.",
                ));
            }
            state.match_reservation.as_mut().unwrap().sealed = true;
            state.game_move_publication = None;
            state.continuous_safety_hold = true;
            state.operation += 1;
            if let Some(run_id) = state.game_move.as_ref().map(|slot| slot.identity.run_id.clone()) {
                game_move::seal_move_for_run(&mut state, &run_id);
            }
        }
        let deadline =
            Instant::now() + self.inner.config.readiness_timeout + self.inner.config.stop_drain_timeout;
        loop {
            let mut state = self.lock();
            if state.match_reservation.is_none() {
                return Ok(());
            }
            let reservation = require_owner(&state, owner)?;
            if !reservation.preparing {
                let runs = reservation.runs.clone();
                for run in runs {
                    if let Some(live) = take_owned_live(&mut state, &run.run_id) {
                        state.retiring.push((run, Arc::new(Mutex::new(live))));
                    }
                }
                break;
            }
            drop(state);
            if Instant::now() >= deadline {
                return Err(reservation_failure(EngineFailureKind::Timeout,
                    "Candidate preparation has not released process ownership; the match remains reserved. Retry Stop."));
            }
            thread::sleep(Duration::from_millis(5));
        }
        self.reap_retiring_runs()?;
        let deadline = Instant::now() + self.inner.config.stop_drain_timeout * 3 + Duration::from_secs(1);
        loop {
            let mut state = self.lock();
            if state.match_reservation.is_none() {
                return Ok(());
            }
            let committed = require_owner(&state, owner)?.committed;
            // A worker whose earlier reap failed may have finished while retaining occupancy.
            // All retiring processes were just confirmed reaped; its sealed slot can now retire.
            if state.game_move.as_ref().is_some_and(|slot| slot.finished) {
                state.game_move = None;
            }
            if state.game_move.is_none() {
                if committed {
                    state.phase = Phase::NoEngine { failure: None };
                }
                state.match_reservation = None;
                if !committed {
                    let deadline = Instant::now() + Duration::from_secs(5);
                    for job in &mut state.jobs {
                        if !job.terminal
                            && job.state == AnalysisJobStateDto::Stopping
                            && job.cancel_deadline.is_none()
                        {
                            job.cancel_deadline = Some(deadline);
                        }
                    }
                }
                state.continuous_safety_hold = true;
                publish_snapshot(&mut state);
                return Ok(());
            }
            drop(state);
            if Instant::now() >= deadline {
                return Err(reservation_failure(EngineFailureKind::Timeout,
                    "The match move worker has not released ownership; the match remains reserved. Retry Stop."));
            }
            thread::sleep(Duration::from_millis(5));
        }
    }

    pub(super) fn reap_retiring_runs(&self) -> Result<(), EngineFailureDto> {
        let retiring = self.lock().retiring.clone();
        let deadline = Instant::now() + self.inner.config.stop_drain_timeout;
        for (run, live) in retiring {
            loop {
                match live.try_lock() {
                    Ok(mut live) => {
                        terminate_process(&mut live, deadline).map_err(|error| {
                            run_failure(
                                &run,
                                EngineFailureKind::Timeout,
                                &format!(
                                    "Engine {} cleanup was not confirmed: {error}; retry Stop.",
                                    run.run_id
                                ),
                            )
                        })?;
                        break;
                    }
                    Err(std::sync::TryLockError::Poisoned(_)) => {
                        return Err(run_failure(
                            &run,
                            EngineFailureKind::InvalidState,
                            "Retiring engine ownership lock was poisoned.",
                        ))
                    }
                    Err(std::sync::TryLockError::WouldBlock) => {
                        if Instant::now() >= deadline {
                            return Err(run_failure(
                                &run,
                                EngineFailureKind::Timeout,
                                "Engine retirement is still running; retry Stop.",
                            ));
                        }
                        thread::sleep(Duration::from_millis(5));
                    }
                }
            }
            self.lock()
                .retiring
                .retain(|(owned, _)| owned.run_id != run.run_id);
        }
        Ok(())
    }
}
