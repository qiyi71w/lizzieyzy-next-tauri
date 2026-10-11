use super::*;

#[derive(Default)]
pub(super) struct Recovery {
    pub required: bool,
    pub run_id: Option<String>,
    pub receipt: Option<AnalysisJobStartedDto>,
    pub attempted: bool,
    pub confirmed: bool,
}

pub(super) fn phase(state: &ManagerState) -> Option<app_model::AutoLoadRecoveryPhaseDto> {
    let recovery = &state.auto_load.recovery;
    recovery.required.then_some(if recovery.confirmed {
        app_model::AutoLoadRecoveryPhaseDto::Confirmed
    } else if recovery.receipt.is_some() && !recovery.confirmed {
        app_model::AutoLoadRecoveryPhaseDto::Confirming
    } else {
        app_model::AutoLoadRecoveryPhaseDto::RestartRequired
    })
}

pub(super) fn require_confirmation(state: &ManagerState) -> Result<(), EngineFailureDto> {
    if state.auto_load.recovery.required && !state.auto_load.recovery.confirmed {
        return Err(continuous_invalid_state(state,
            "Automatic handback failed: explicitly Restart and wait for the new reader's current-position confirmation before Continue."));
    }
    Ok(())
}

pub(super) fn invalidate(state: &mut ManagerState) {
    if state.auto_load.recovery.required {
        state.auto_load.recovery = Recovery {
            required: true,
            ..Recovery::default()
        };
    }
}

pub(super) fn started(state: &mut ManagerState, run: &EngineRunDto, operation: EngineOperationDto) {
    invalidate(state);
    if state.auto_load.recovery.required && operation == EngineOperationDto::Restart {
        state.auto_load.recovery.run_id = Some(run.run_id.clone());
    }
}

pub(super) fn observe_failure(state: &mut ManagerState) {
    if state.auto_load.handback_pending && (state.continuous_safety_hold || state.continuous_error) {
        state.auto_load.handback_pending = false;
        state.auto_load.recovery = Recovery {
            required: true,
            ..Recovery::default()
        };
    }
    let recovery = &state.auto_load.recovery;
    if let Some(run_id) = &recovery.run_id {
        if !matches!(&state.phase, Phase::Ready(run) | Phase::Starting(run) if run.run_id == *run_id) {
            invalidate(state);
        }
    }
}

/// Only the manager's existing immutable Run reader can call the finite terminal path.
/// The displayed SGF result and successful submission are deliberately not receipts.
pub(super) fn terminal(
    state: &mut ManagerState,
    started: &AnalysisJobStartedDto,
    outcome: AnalysisJobOutcomeDto,
    has_frame: bool,
) -> bool {
    if started.mode == AnalysisJobModeDto::Continuous
        && has_frame
        && state.auto_load.snapshot.phase != app_model::AutoLoadPhaseDto::Running
    {
        state.auto_load.handback_pending = false;
    }
    let Some(receipt) = &state.auto_load.recovery.receipt else {
        return false;
    };
    if receipt.run_id != started.run_id || receipt.job_id != started.job_id {
        return false;
    }
    let valid = outcome == AnalysisJobOutcomeDto::Completed
        && has_frame
        && matches!(&state.phase, Phase::Ready(run) if run.run_id == started.run_id)
        && state.continuous_target.as_ref().is_some_and(|target| {
            target.generation == started.generation && target.node_path == started.node_path
        });
    state.auto_load.recovery.confirmed = valid;
    if !valid {
        state.auto_load.recovery.receipt = None;
    }
    true
}

impl ForegroundEngineManager {
    /// Confirmation is a bounded finite request through the existing selected-node lane.
    /// It neither clears safety/Pause nor attaches its output to the document.
    pub(super) fn reconcile_auto_load_recovery(&self) {
        let submission = {
            let mut state = self.lock();
            if !state.auto_load.recovery.required
                || state.auto_load.recovery.attempted
                || state.continuous_departing
                || state.finite_admission_pending
                || state.match_reservation.is_some()
                || state.game_move.is_some()
                || current_selected_job(&state).is_some()
            {
                return;
            }
            let Phase::Ready(run) = &state.phase else {
                return;
            };
            if run.adapter_kind != EngineBackend::KataGoAnalysis
                || state.auto_load.recovery.run_id.as_deref() != Some(&run.run_id)
            {
                return;
            }
            let run_id = run.run_id.clone();
            let Some(mut request) = state.continuous_target.clone() else {
                return;
            };
            request.run_id = run_id;
            request.mode = AnalysisJobModeDto::Finite;
            request.query.task(&AnalysisStageConditionsDto {
                time_seconds: app_model::AnalysisTaskLimitDto {
                    enabled: true,
                    value: 10,
                },
                total_visits: app_model::AnalysisTaskLimitDto {
                    enabled: true,
                    value: 1,
                },
                leading_candidate_visits: app_model::AnalysisTaskLimitDto {
                    enabled: false,
                    value: 1,
                },
            });
            state.auto_load.recovery.attempted = true;
            match self.register_selected_locked(&mut state, request) {
                Ok(submission) => {
                    state.auto_load.recovery.receipt = Some(submission.started.clone());
                    publish_snapshot(&mut state);
                    Some(submission)
                }
                Err(_) => {
                    publish_snapshot(&mut state);
                    None
                }
            }
        };
        if let Some(submission) = submission {
            let _ = self.submit_registered_selected(submission);
        }
    }
}
