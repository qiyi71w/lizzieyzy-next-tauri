use super::*;
use app_model::{ContinuationModeDto, ContinuationPhaseDto as PhaseDto, ContinuationPreferencesDto, ContinuationSnapshotDto, MoveVertex};
use std::collections::VecDeque;

#[derive(Default)]
pub(super) struct Owner {
    epoch: u64,
    snapshot: ContinuationSnapshotDto,
    reserved: bool,
    interval: Duration,
    origin: Option<Instant>,
    line: VecDeque<NodePath>,
    candidate: Option<MoveVertex>,
}

/// A single decision; the current-game owner alone applies it to its authoritative document.
pub enum ContinuationAction {
    Navigate(NodePath),
    Play(MoveVertex),
}

pub(super) fn occupied(state: &ManagerState) -> bool { state.continuation.reserved }
pub(super) fn require_idle(state: &ManagerState) -> Result<(), EngineFailureDto> {
    if occupied(state) { Err(continuous_invalid_state(state, "Engine Continuation owns the current position; stop it first.")) } else { Ok(()) }
}

pub(super) fn invalidate(state: &mut ManagerState, reason: &str) {
    let owner = &mut state.continuation;
    owner.epoch = owner.epoch.wrapping_add(1);
    if owner.reserved {
        owner.reserved = false;
        owner.candidate = None;
        owner.snapshot.phase = PhaseDto::Stopped;
        owner.snapshot.reason = Some(reason.into());
    }
}

pub(super) fn observe(state: &mut ManagerState, event: &ForegroundEngineEventDto) {
    let owner = &mut state.continuation;
    if !owner.reserved { return; }
    if let ForegroundEngineEventDto::Job { job } = event {
        if owner.snapshot.run_id.as_ref() != Some(&job.run_id)
            || owner.snapshot.request_id.as_ref() != Some(&job.job_id)
            || owner.snapshot.generation != job.generation || owner.snapshot.node_path != job.node_path { return; }
        if let Some(failure) = &job.failure {
            owner.snapshot.phase = PhaseDto::Failed;
            owner.snapshot.reason = Some(failure.message.clone());
            owner.candidate = None;
        } else if app_model::admits_analysis_attachment(job) {
            // The sole reader has already validated framing, query and (for GTP) position readback.
            if let Some(frame) = &job.frame {
                match frame.candidates.first() {
                    Some(candidate) => owner.candidate = Some(candidate.vertex.clone()),
                    None => {
                        owner.snapshot.phase = PhaseDto::Failed;
                        owner.snapshot.reason = Some("Engine returned no legal candidate.".into());
                    }
                }
            }
        }
    }
}

impl ForegroundEngineManager {
    /// Capture before dispatching asynchronous Start work; every Stop invalidates it.
    pub fn continuation_epoch(&self) -> u64 { self.lock().continuation.epoch }

    pub fn continuation_snapshot(&self) -> ContinuationSnapshotDto {
        self.lock().continuation.snapshot.clone()
    }

    /// Reserves before draining the old query, then registers a genuinely new request.
    /// Never rebinds a pre-Start frame, even for the same Run and exact position.
    pub fn start_continuation(&self, request: SelectedNodeJobRequest, preferences: ContinuationPreferencesDto,
        line: Vec<NodePath>, terminal: bool, now: Instant, epoch: u64) -> Result<ContinuationSnapshotDto, EngineFailureDto> {
        let interval = preferences.interval_millis().map_err(|message| failure(EngineOperationDto::Job,
            EngineFailureKind::InvalidState, FailureText::unscoped(&message), Some(&request.run_id), None, None))?;
        self.retire_auto_load()?;
        let old = {
            let mut state = self.lock();
            if state.continuation.epoch != epoch || request.mode != AnalysisJobModeDto::Continuous {
                return Err(continuous_invalid_state(&state, "Continuation Start was canceled or is not a continuous request."));
            }
            require_idle(&state)?;
            batch::admit(&state, None)?;
            validate_selected_admission(&state, &request)?;
            if state.continuous_safety_hold || state.continuous_error || state.continuous_paused.is_some()
                || state.finite_admission_pending || state.analysis_task.as_ref().is_some_and(|task|
                    matches!(task.state, AnalysisTaskStateDto::Queued | AnalysisTaskStateDto::Searching | AnalysisTaskStateDto::Pausing | AnalysisTaskStateDto::Paused))
                || state.jobs.iter().any(|job| !job.terminal && job.mode == AnalysisJobModeDto::Finite) {
                return Err(continuous_invalid_state(&state, "Resolve the current Pause, safety hold or explicit analysis owner before starting continuation."));
            }
            let old = current_selected_job(&state).filter(|job| !job.state.is_limited());
            state.continuation = Owner {
                epoch,
                snapshot: ContinuationSnapshotDto {
                    session_id: Some(Uuid::new_v4().to_string()),
                    phase: if terminal { PhaseDto::Terminal } else { PhaseDto::Waiting },
                    mode: preferences.mode,
                    run_id: Some(request.run_id.clone()), request_id: None,
                    generation: request.generation, node_path: request.node_path.clone(),
                    reason: terminal.then(|| "The selected position is already terminal.".into()), current_game: None,
                }, reserved: !terminal, interval: Duration::from_millis(interval), origin: Some(now),
                line: line.into(), candidate: None,
            };
            if terminal { return Ok(state.continuation.snapshot.clone()); }
            old
        };
        let result = (|| {
            if let Some(old) = old {
                self.request_job_stop(&old.run_id, &old.job_id, JobDisposition::Superseded)?;
                self.wait_for_job_cancellation(&old.run_id, &old.job_id, Duration::from_secs(5))?;
            }
            self.submit_continuation_position(request)
        })();
        if let Err(error) = result {
            self.seal_continuation(PhaseDto::Failed, &error.message);
            return Err(error);
        }
        Ok(self.continuation_snapshot())
    }

    fn submit_continuation_position(&self, request: SelectedNodeJobRequest) -> Result<(), EngineFailureDto> {
        let submission = {
            let mut state = self.lock();
            if !occupied(&state) { return Err(continuous_invalid_state(&state, "Continuation was stopped before its next request.")); }
            if state.continuation.snapshot.run_id.as_ref() != Some(&request.run_id) {
                return Err(continuous_invalid_state(&state, "Continuation Run changed."));
            }
            state.continuous_target = Some(request.clone());
            let submission = self.register_selected_locked(&mut state, request)?;
            state.continuation.snapshot.request_id = Some(submission.started.job_id.clone());
            state.continuation.snapshot.generation = submission.started.generation;
            state.continuation.snapshot.node_path = submission.started.node_path.clone();
            state.continuation.candidate = None;
            submission
        };
        self.submit_registered_selected(submission)?;
        Ok(())
    }

    /// Caller holds the current-game lock. This manager lock fences Run/reader/job retirement
    /// across the legal document commit; no event or timer can commit a second action.
    pub fn advance_continuation<F>(&self, session: &str, now: Instant, commit: F) -> ContinuationSnapshotDto
    where F: FnOnce(ContinuationAction) -> Result<(SelectedNodeJobRequest, bool), String> {
        let next = {
            let mut state = self.lock();
            if !occupied(&state) || state.continuation.snapshot.session_id.as_deref() != Some(session) {
                return state.continuation.snapshot.clone();
            }
            let run = current_admitting_run(&state.phase);
            if run.as_ref().map(|run| &run.run_id) != state.continuation.snapshot.run_id.as_ref()
                || state.continuous_safety_hold || state.continuous_error || state.continuous_paused.is_some()
                || !state.jobs.iter().any(|job| state.continuation.snapshot.request_id.as_ref() == Some(&job.job_id)
                    && state.continuation.snapshot.run_id.as_ref() == Some(&job.run_id)
                    && job.state != AnalysisJobStateDto::Stopping
                    && job.generation == state.continuation.snapshot.generation
                    && job.node_path == state.continuation.snapshot.node_path)
                || state.continuous_departing || state.match_reservation.is_some() {
                state.continuation.snapshot.phase = PhaseDto::Failed;
                state.continuation.snapshot.reason = Some("Continuation authority or Run is no longer available.".into());
            }
            if state.continuation.snapshot.phase == PhaseDto::Failed {
                None
            } else if now.saturating_duration_since(state.continuation.origin.expect("active clock")) < state.continuation.interval {
                return state.continuation.snapshot.clone();
            } else {
                if state.continuation.candidate.is_none() {
                    state.continuation.snapshot.phase = PhaseDto::Waiting;
                    return state.continuation.snapshot.clone();
                }
                let action = if state.continuation.snapshot.mode == ContinuationModeDto::LeafOnly && !state.continuation.line.is_empty() {
                    Some(ContinuationAction::Navigate(state.continuation.line.pop_front().expect("line present")))
                } else { state.continuation.candidate.take().map(ContinuationAction::Play) };
                let Some(action) = action else {
                    state.continuation.snapshot.phase = PhaseDto::Waiting;
                    return state.continuation.snapshot.clone();
                };
                match commit(action) {
                    Ok((request, terminal)) => {
                        state.continuation.origin = Some(now);
                        state.continuation.snapshot.generation = request.generation;
                        state.continuation.snapshot.node_path = request.node_path.clone();
                        state.continuous_target = Some(request.clone());
                        state.continuation.snapshot.phase = if terminal { PhaseDto::Terminal } else { PhaseDto::Running };
                        state.continuation.snapshot.reason = terminal.then(|| "The selected position is terminal.".into());
                        Some((request, terminal))
                    }
                    Err(reason) => {
                        state.continuation.snapshot.phase = PhaseDto::Failed;
                        state.continuation.snapshot.reason = Some(reason);
                        None
                    }
                }
            }
        };
        // The old query is sealed before waiting for its target-final. The document lock
        // remains with the caller; Stop and external edits linearize before or after this beat.
        let old = { let state = self.lock(); current_selected_job(&state).filter(|job| !job.state.is_limited()) };
        if let Some(job) = old {
            if let Err(error) = self.request_job_stop(&job.run_id, &job.job_id, JobDisposition::Superseded)
                .and_then(|()| self.wait_for_job_cancellation(&job.run_id, &job.job_id, Duration::from_secs(5))) {
                self.seal_continuation(PhaseDto::Failed, &error.message);
                return self.continuation_snapshot();
            }
        }
        match next {
            Some((request, false)) => {
                if let Err(error) = self.submit_continuation_position(request) { self.seal_continuation(PhaseDto::Failed, &error.message); }
            }
            _ => {
                self.lock().continuation.reserved = false;
                self.reconcile_continuous();
            }
        }
        self.continuation_snapshot()
    }

    pub fn stop_continuation(&self, reason: &str) -> ContinuationSnapshotDto {
        self.seal_continuation(PhaseDto::Stopped, reason);
        self.continuation_snapshot()
    }

    fn seal_continuation(&self, phase: PhaseDto, reason: &str) {
        let old = {
            let mut state = self.lock();
            state.continuation.epoch = state.continuation.epoch.wrapping_add(1);
            if !occupied(&state) { return; }
            state.continuation.reserved = false;
            state.continuation.candidate = None;
            state.continuation.snapshot.phase = phase;
            state.continuation.snapshot.reason = Some(reason.into());
            current_selected_job(&state).filter(|job| state.continuation.snapshot.request_id.as_ref() == Some(&job.job_id) && !job.state.is_limited())
        };
        if let Some(job) = old {
            let _ = self.request_job_stop(&job.run_id, &job.job_id, JobDisposition::Superseded);
        }
        self.reconcile_continuous();
    }
}
