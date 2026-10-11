use super::*;
use app_model::{AutoLoadPhaseDto as PhaseDto, AutoLoadPreferencesDto, AutoLoadSnapshotDto};

#[derive(Default)]
pub(super) struct Owner {
    pub preferences: AutoLoadPreferencesDto,
    pub snapshot: AutoLoadSnapshotDto,
    pub handback_pending: bool,
    pub recovery: super::auto_load_recovery::Recovery,
}

fn active(state: &ManagerState) -> bool {
    matches!(state.auto_load.snapshot.phase, PhaseDto::Waiting | PhaseDto::Running)
}

fn allowed(state: &ManagerState) -> bool {
    !state.continuous_safety_hold && !state.continuous_error
        && state.continuous_paused.is_none() && !state.continuous_departing
        && state.match_reservation.is_none() && state.game_move.is_none()
        && state.runtime_control.is_none()
}

impl ForegroundEngineManager {
    /// Only durable publication changes these values. Existing load identities retain their budget.
    pub fn set_auto_load_preferences(&self, preferences: AutoLoadPreferencesDto) -> Result<(), String> {
        preferences.validate()?;
        self.lock().auto_load.preferences = preferences;
        Ok(())
    }

    /// Called under the current-document lock, after a successful independent replacement.
    pub fn commit_auto_load(&self, load_id: u64, generation: u64) {
        let mut state = self.lock();
        let preferences = state.auto_load.preferences;
        let run_id = match &state.phase {
            Phase::Ready(run) | Phase::Starting(run) => Some(run.run_id.clone()),
            _ => None,
        };
        let busy = state.analysis_task.as_ref().is_some_and(|task| matches!(task.state,
            AnalysisTaskStateDto::Queued | AnalysisTaskStateDto::Searching |
            AnalysisTaskStateDto::Pausing | AnalysisTaskStateDto::Paused))
            || state.jobs.iter().any(|job| !job.terminal && job.mode == AnalysisJobModeDto::Finite);
        let phase = if !preferences.enabled { PhaseDto::Idle }
            else if run_id.is_none() || !allowed(&state) || busy { PhaseDto::Unavailable }
            else { PhaseDto::Waiting };
        state.auto_load.snapshot = AutoLoadSnapshotDto {
            phase, load_id, generation, run_id, task_id: None,
            total_visits: preferences.total_visits,
            reason: (phase == PhaseDto::Unavailable).then(|| "No eligible foreground authority at load; use explicit analysis.".into()),
        };
        publish_snapshot(&mut state);
    }

    pub fn pending_auto_load(&self) -> Option<AutoLoadSnapshotDto> {
        let state = self.lock();
        let intent = &state.auto_load.snapshot;
        (intent.phase == PhaseDto::Waiting && allowed(&state)
            && matches!(&state.phase, Phase::Ready(run) if intent.run_id.as_deref() == Some(&run.run_id)))
            .then(|| intent.clone())
    }

    pub fn finish_auto_load_admission(&self, load_id: u64, reason: Option<String>) {
        let mut state = self.lock();
        if state.auto_load.snapshot.load_id != load_id || state.auto_load.snapshot.phase != PhaseDto::Waiting { return; }
        state.auto_load.snapshot.phase = if reason.is_some() { PhaseDto::Failed } else { PhaseDto::NotNeeded };
        state.auto_load.snapshot.reason = reason;
        publish_snapshot(&mut state);
    }

    /// Explicit consumers (including future batch/continuation) retire, then drain, before admission.
    pub fn retire_auto_load(&self) -> Result<(), EngineFailureDto> {
        let job = {
            let mut state = self.lock();
            if !active(&state) && state.auto_load.recovery.receipt.is_none() { return Ok(()); }
            let mut job = retire_locked(self, &mut state)?;
            if let Some(receipt) = state.auto_load.recovery.receipt.clone() {
                super::auto_load_recovery::invalidate(&mut state);
                if state.jobs.iter().any(|job| job.run_id == receipt.run_id && job.job_id == receipt.job_id && !job.terminal) {
                    self.request_job_stop_locked(&mut state, &receipt.run_id, &receipt.job_id, JobDisposition::Cancelled)?;
                    job = Some((receipt.run_id, receipt.job_id));
                }
            }
            publish_snapshot(&mut state);
            job
        };
        if let Some((run_id, job_id)) = job {
            self.wait_for_job_cancellation(&run_id, &job_id, self.inner.config.stop_drain_timeout)?;
        }
        Ok(())
    }

    pub fn start_auto_load_task(&self, intent: AutoLoadSnapshotDto, request: WholeGameJobRequest,
        scope: AnalysisScopeDto) -> Result<AnalysisTaskDto, EngineFailureDto> {
        let conditions = AnalysisStageConditionsDto {
            time_seconds: app_model::AnalysisTaskLimitDto { enabled: false, value: 1 },
            total_visits: app_model::AnalysisTaskLimitDto { enabled: true, value: intent.total_visits },
            leading_candidate_visits: app_model::AnalysisTaskLimitDto { enabled: false, value: 1 },
        };
        self.start_analysis_task_with_strategy(request, scope, AnalysisTaskStrategyDto::SingleStage,
            None, conditions, AnalysisTaskTargets::default(), Some(intent))
    }
}

pub(super) fn admit(state: &ManagerState, intent: &AutoLoadSnapshotDto) -> Result<(), EngineFailureDto> {
    if state.auto_load.snapshot != *intent || intent.phase != PhaseDto::Waiting || !allowed(state) {
        return Err(continuous_invalid_state(state, "Automatic load authorization retired."));
    }
    Ok(())
}

pub(super) fn retire_locked(manager: &ForegroundEngineManager, state: &mut ManagerState)
    -> Result<Option<(String, String)>, EngineFailureDto> {
    if !active(state) { return Ok(None); }
    let job = state.analysis_task.as_ref().filter(|task|
        state.auto_load.snapshot.task_id.as_deref() == Some(&task.task_id))
        .map(|task| (task.run_id.clone(), task.job_id.clone()));
    state.auto_load.snapshot.phase = PhaseDto::Cancelled;
    if let Some((run_id, job_id)) = &job {
        manager.request_job_stop_locked(state, run_id, job_id, JobDisposition::Cancelled)?;
    }
    Ok(job)
}

pub(super) fn refresh(state: &mut ManagerState) {
    if !active(state) { return; }
    let intent = &state.auto_load.snapshot;
    let same_run = matches!(&state.phase, Phase::Starting(run) | Phase::Ready(run)
        if intent.run_id.as_deref() == Some(&run.run_id));
    if !same_run || !allowed(state) {
        state.auto_load.snapshot.phase = if state.continuous_safety_hold || state.continuous_error {
            PhaseDto::Failed
        } else { PhaseDto::Cancelled };
        return;
    }
    if let Some(task) = state.analysis_task.as_ref().filter(|task|
        intent.task_id.as_deref() == Some(&task.task_id)) {
        state.auto_load.snapshot.phase = match task.state {
            AnalysisTaskStateDto::Completed => PhaseDto::Complete,
            AnalysisTaskStateDto::Failed => PhaseDto::Failed,
            AnalysisTaskStateDto::Cancelled | AnalysisTaskStateDto::Invalidated => PhaseDto::Cancelled,
            _ => PhaseDto::Running,
        };
        state.auto_load.snapshot.reason = task.reason.clone();
    }
}
