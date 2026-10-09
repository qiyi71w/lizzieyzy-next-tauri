#![allow(clippy::result_large_err)]
use crate::catalog::{EngineProfileCatalog, SavedEngineProfile};
use crate::gtp::{self, ResponseDecoder};
use crate::resources::{ResourceSnapshot, StartupOutput};
use crate::diagnostics::{AttemptCapture, ObservedRead};
use crate::{
    build_command_spec, build_process_command, check_assets, kill_timed_out_child, spawn_stdout_lines_reader,
    write_jsonl, AnalysisCancelToken,
};
use app_model::{
    AnalysisJobLaneDto, AnalysisJobModeDto, AnalysisJobOutcomeDto, AnalysisJobStartedDto,
    AnalysisJobStateDto, AnalysisScopeDto, AnalysisScopeModeDto, AnalysisStageConditionsDto,
    AnalysisSwingComparisonDto, AnalysisSwingCriteriaDto, AnalysisTaskDto, AnalysisTaskLimitDto,
    AnalysisTaskOverviewDto, AnalysisTaskStageDto, AnalysisTaskStateDto, AnalysisTaskStrategyDto,
    ContinuousAnalysisBudgetDto, ContinuousAnalysisPhaseDto, ContinuousAnalysisSnapshotDto,
    EngineAnalysisCapabilitiesDto, EngineBackend, EngineCapabilitySnapshotDto, EngineFailureDto,
    EngineFailureKind, EngineGtpFactsDto, EngineOperationDto, EngineRunDto, ForegroundEngineEventDto,
    ForegroundEngineLifecycleDto, ForegroundEngineSnapshotDto, NodePath, PlayerColor,
};
use katago_protocol::{normalize_response, parse_response_line, AnalysisQuery, ProtocolError};
use std::io;
use std::process::{Child, ChildStdin};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use uuid::Uuid;

pub use app_model::AnalysisJobEventDto;
pub use app_model::AnalysisJobLaneDto as AnalysisJobLane;

mod game_move;
mod evaluation;
mod match_reservation;
mod ordinary_rules;
mod preload;
mod gtp_control;
mod gtp_analysis;
pub use game_move::{GameMoveHandle, GameMoveRequest, OrdinaryRulesHandle};

pub trait AnalysisJobCancel: Send + Sync {
    fn cancel(&self);
}

impl AnalysisJobCancel for AnalysisCancelToken {
    fn cancel(&self) {
        AnalysisCancelToken::cancel(self);
    }
}

#[derive(Clone, Debug)]
pub struct ForegroundEngineConfig {
    pub readiness_timeout: Duration,
    pub stop_drain_timeout: Duration,
    pub job_timeout: Duration,
    pub admit_whole_game_analysis: bool,
    pub managed_resources_root: Option<std::path::PathBuf>,
}

impl Default for ForegroundEngineConfig {
    fn default() -> Self {
        Self {
            readiness_timeout: Duration::from_secs(30),
            stop_drain_timeout: Duration::from_secs(2),
            job_timeout: Duration::from_secs(60),
            admit_whole_game_analysis: true,
            managed_resources_root: None,
        }
    }
}

impl ForegroundEngineConfig {
    pub fn for_tests() -> Self {
        Self {
            readiness_timeout: Duration::from_secs(2),
            stop_drain_timeout: Duration::from_millis(400),
            job_timeout: Duration::from_millis(800),
            admit_whole_game_analysis: true,
            managed_resources_root: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum JobDisposition {
    Running,
    BudgetReached,
    Superseded,
    Paused,
    Cancelled,
    TimedOut,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContinuousPrimaryAction {
    Start,
    Stop,
    Resume,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ContinuousAdmission {
    run_id: String,
    generation: u64,
    node_path: NodePath,
}

#[derive(Clone)]
pub struct SelectedNodeJobRequest {
    pub run_id: String,
    pub mode: AnalysisJobModeDto,
    pub generation: u64,
    pub node_path: NodePath,
    pub query: AnalysisQuery,
    pub board_width: u8,
    pub board_height: u8,
    pub position_empty: bool,
    pub exact_position: Result<sgf::ExactPosition, String>,
}

#[derive(Clone)]
pub struct WholeGameWorkItem {
    pub node_path: NodePath,
    pub query: AnalysisQuery,
    pub board_width: u8,
    pub board_height: u8,
    pub move_number: u32,
}

struct SelectedSubmission {
    started: AnalysisJobStartedDto,
    command: SelectedCommand,
}

enum SelectedCommand {
    Jsonl(String),
    Gtp(SelectedNodeJobRequest),
}

enum ContinuousReconcileWork {
    Cancel(AnalysisJobStartedDto),
    Submit(SelectedSubmission),
}

pub struct WholeGameJobRequest {
    pub run_id: String,
    pub generation: u64,
    pub work_items: Vec<WholeGameWorkItem>,
}

pub struct SwingAnalysisTaskRequest {
    pub job: WholeGameJobRequest,
    pub scope: AnalysisScopeDto,
    pub requested: Vec<NodePath>,
    pub supporting: Vec<NodePath>,
    pub swing_comparisons: Vec<AnalysisSwingComparisonDto>,
    pub swing_criteria: AnalysisSwingCriteriaDto,
    pub overview_conditions: AnalysisStageConditionsDto,
    pub deep_conditions: AnalysisStageConditionsDto,
}

#[derive(Default)]
struct AnalysisTaskTargets {
    requested: Option<Vec<NodePath>>,
    supporting: Vec<NodePath>,
    swing_comparisons: Vec<AnalysisSwingComparisonDto>,
    swing_criteria: Option<AnalysisSwingCriteriaDto>,
}

enum WholeGameAdvance {
    Submit {
        started: AnalysisJobStartedDto,
        jsonl: String,
    },
    Terminate {
        run_id: String,
        job_id: String,
        query_id: String,
    },
}

struct RegisteredJob {
    job_id: String,
    run_id: String,
    query_id: String,
    lane: AnalysisJobLane,
    mode: AnalysisJobModeDto,
    continuous_budget: Option<ContinuousAnalysisBudgetDto>,
    state: AnalysisJobStateDto,
    cancel_deadline: Option<Instant>,
    cleanup_deadline: Option<Instant>,
    submitted: bool,
    submitted_at: Instant,
    generation: u64,
    node_path: NodePath,
    board_width: u8,
    board_height: u8,
    cancel: Arc<dyn AnalysisJobCancel>,
    disposition: JobDisposition,
    expected: Option<usize>,
    work_items: Vec<WholeGameWorkItem>,
    current_index: usize,
    pending_ending_conditions: Vec<String>,
    terminal: bool,
}

struct LiveEngine {
    child: Child,
    stdin: Arc<Mutex<Option<ChildStdin>>>,
    stdout_rx: Option<Receiver<io::Result<Option<String>>>>,
    startup_output: Arc<Mutex<StartupOutput>>,
    run_id: String,
    process_id: u32,
    capture: AttemptCapture,
}

enum Phase {
    NoEngine {
        failure: Option<EngineFailureDto>,
    },
    Starting(EngineRunDto),
    Ready(EngineRunDto),
    Switching {
        primary: EngineRunDto,
        candidate: EngineRunDto,
        switch_id: String,
    },
    Stopping(EngineRunDto),
    Error {
        run: EngineRunDto,
        failure: EngineFailureDto,
    },
}

struct ManagerState {
    revision: u64,
    phase: Phase,
    last_primary_profile_id: Option<String>,
    operation: u64,
    operation_kind: EngineOperationDto,
    live: Option<LiveEngine>,
    candidate: Option<LiveEngine>,
    match_residents: Vec<LiveEngine>,
    preloads: Vec<preload::PreloadSlot>,
    preload_shutdown: bool,
    switch_seq: u64,
    last_activity: Instant,
    jobs: Vec<RegisteredJob>,
    analysis_task: Option<AnalysisTaskDto>,
    subscribers: Vec<Sender<ForegroundEngineEventDto>>,
    continuous_intent: Option<bool>,
    continuous_budget: ContinuousAnalysisBudgetDto,
    continuous_target: Option<SelectedNodeJobRequest>,
    continuous_limited: Option<(ContinuousAdmission, ContinuousAnalysisPhaseDto)>,
    continuous_paused: Option<ContinuousAdmission>,
    continuous_error: bool,
    continuous_safety_hold: bool,
    continuous_departing: bool,
    finite_admission_pending: bool,
    game_move: Option<game_move::MoveSlot>,
    game_move_publication: Option<app_model::GameMoveJobDto>,
    gtp_command_seq: u32,
    gtp_dispatch: gtp_control::Dispatch,
    match_reservation: Option<match_reservation::MatchReservation>,
    retiring: Vec<(EngineRunDto, Arc<Mutex<LiveEngine>>)>,
    evaluation: Option<evaluation::EvaluationSlot>,
}

struct Inner {
    catalog: Arc<dyn EngineProfileCatalog>,
    config: ForegroundEngineConfig,
    state: Mutex<ManagerState>,
    diagnostics: Mutex<std::collections::VecDeque<AttemptCapture>>,
    evaluation_worker: Arc<std::sync::atomic::AtomicBool>,
}

#[derive(Clone)]
pub struct ForegroundEngineManager {
    inner: Arc<Inner>,
}

impl ForegroundEngineManager {
    pub fn new(catalog: Arc<dyn EngineProfileCatalog>, config: ForegroundEngineConfig) -> Self {
        Self {
            inner: Arc::new(Inner {
                catalog,
                config,
                diagnostics: Mutex::new(std::collections::VecDeque::new()),
                evaluation_worker: Arc::new(std::sync::atomic::AtomicBool::new(false)),
                state: Mutex::new(ManagerState {
                    revision: 0,
                    phase: Phase::NoEngine { failure: None },
                    last_primary_profile_id: None,
                    operation: 0,
                    operation_kind: EngineOperationDto::Start,
                    live: None,
                    candidate: None,
                    match_residents: Vec::new(),
                    preloads: Vec::new(),
                    preload_shutdown: false,
                    switch_seq: 0,
                    jobs: Vec::new(),
                    analysis_task: None,
                    last_activity: Instant::now(),
                    subscribers: Vec::new(),
                    continuous_intent: None,
                    continuous_budget: ContinuousAnalysisBudgetDto::default(),
                    continuous_target: None,
                    continuous_limited: None,
                    continuous_paused: None,
                    continuous_error: false,
                    continuous_safety_hold: false,
                    continuous_departing: false,
                    finite_admission_pending: false,
                    game_move: None,
                    game_move_publication: None,
                    gtp_command_seq: 100,
                    gtp_dispatch: gtp_control::Dispatch::default(),
                    match_reservation: None,
                    retiring: Vec::new(),
                    evaluation: None,
                }),
            }),
        }
    }

    pub fn snapshot(&self) -> ForegroundEngineSnapshotDto {
        snapshot_from(&self.lock())
    }

    pub fn diagnostic_snapshots(&self) -> Vec<app_model::EngineDiagnosticSnapshotDto> {
        self.inner.diagnostics.lock().expect("diagnostic capture list").iter().map(AttemptCapture::snapshot).collect()
    }

    pub fn set_diagnostic_trace(&self, attempt_id: &str, enabled: bool) -> Result<(), String> {
        let attempts = self.inner.diagnostics.lock().expect("diagnostic capture list");
        let attempt = attempts.iter().find(|capture| capture.matches_run(attempt_id)).ok_or("diagnostic attempt expired")?;
        attempt.set_trace(enabled);
        Ok(())
    }

    pub fn diagnostic_trace_enabled(&self, run_id: &str) -> bool {
        self.inner.diagnostics.lock().expect("diagnostic capture list").iter()
            .find(|capture| capture.matches_run(run_id)).is_some_and(AttemptCapture::trace_enabled)
    }

    pub fn trace_analysis_adoption(&self, run_id: &str, detail: impl FnOnce() -> String) {
        if let Some(capture) = self.inner.diagnostics.lock().expect("diagnostic capture list").iter()
            .find(|capture| capture.matches_run(run_id)) {
            capture.trace(detail);
        }
    }

    /// Last successfully established primary in this application session, retained after Stop.
    pub fn last_primary_profile_id(&self) -> Option<String> {
        self.lock().last_primary_profile_id.clone()
    }

    pub fn analysis_task_snapshot(&self) -> Option<AnalysisTaskDto> {
        self.lock().analysis_task.clone()
    }

    pub fn subscribe(&self) -> Receiver<ForegroundEngineEventDto> {
        let (tx, rx) = mpsc::channel();
        self.lock().subscribers.push(tx);
        rx
    }
    pub fn set_continuous_preferences(
        &self,
        enabled: bool,
        budget: ContinuousAnalysisBudgetDto,
    ) -> Result<(), String> {
        self.commit_continuous_preferences(enabled, budget, || Ok(()))
    }

    /// Holds admission through persistence and intent installation. The callback must not
    /// call back into the manager; acquire any preference lock before entering this method.
    pub fn commit_continuous_preferences<T>(
        &self,
        enabled: bool,
        budget: ContinuousAnalysisBudgetDto,
        persist: impl FnOnce() -> Result<T, String>,
    ) -> Result<T, String> {
        budget.validate()?;
        let saved = {
            let mut state = self.lock();
            let budget_changed = state.continuous_budget != budget;
            if state.continuous_intent == Some(enabled) && !budget_changed {
                return persist();
            }
            match_reservation::require_unreserved(&state).map_err(|error| error.message)?;
            let saved = persist()?;
            state.continuous_intent = Some(enabled);
            state.continuous_budget = budget;
            if budget_changed {
                clear_weak_holds_for_new_position(&mut state);
            }
            if let Some(job) = current_selected_job(&state).filter(|job| {
                job.mode == AnalysisJobModeDto::Continuous
                    && !job.state.is_limited()
                    && (!enabled || budget_changed)
            }) {
                let disposition = if enabled {
                    JobDisposition::Superseded
                } else {
                    JobDisposition::Cancelled
                };
                // Seal under the same lock as the new settings: an old final cannot re-latch its limit.
                let _ = self.request_job_stop_locked(&mut state, &job.run_id, &job.job_id, disposition);
            }
            publish_snapshot(&mut state);
            saved
        };
        self.reconcile_continuous();
        Ok(saved)
    }

    pub fn follow_continuous_position(&self, mut request: SelectedNodeJobRequest) {
        request.mode = AnalysisJobModeDto::Continuous;
        let cancel = {
            let mut state = self.lock();
            if state.match_reservation.is_some() {
                return;
            }
            let changed = state.continuous_target.as_ref().is_none_or(|old| {
                !same_position(
                    old.generation,
                    &old.node_path,
                    request.generation,
                    &request.node_path,
                )
            });
            state.continuous_target = Some(request);
            if changed {
                clear_weak_holds_for_new_position(&mut state);
            }
            let cancel = current_selected_job(&state).filter(|job| {
                state.continuous_target.as_ref().is_some_and(|target| {
                    !same_position(
                        job.generation,
                        &job.node_path,
                        target.generation,
                        &target.node_path,
                    )
                })
            });
            if changed || cancel.is_some() {
                publish_snapshot(&mut state);
            }
            cancel
        };
        if let Some(job) = cancel {
            let _ = self.request_job_stop(&job.run_id, &job.job_id, JobDisposition::Superseded);
        }
        self.reconcile_continuous();
    }

    pub fn clear_continuous_position(&self) {
        let cancel = {
            let mut state = self.lock();
            if state.match_reservation.is_some() {
                return;
            }
            let changed = state.continuous_target.take().is_some();
            if changed {
                clear_weak_holds_for_new_position(&mut state);
            }
            let cancel = current_selected_job(&state);
            if changed || cancel.is_some() {
                publish_snapshot(&mut state);
            }
            cancel
        };
        if let Some(job) = cancel {
            let _ = self.request_job_stop(&job.run_id, &job.job_id, JobDisposition::Superseded);
        }
    }

    pub fn check_analysis_task_admission(
        &self,
        swing_criteria: Option<&AnalysisSwingCriteriaDto>,
    ) -> Result<(), EngineFailureDto> {
        let state = self.lock();
        match_reservation::require_unreserved(&state)?;
        let run = current_admitting_run(&state.phase).ok_or_else(|| {
            continuous_invalid_state(&state, "Analysis task requires a Ready Foreground Engine Run")
        })?;
        validate_whole_game_capabilities(&run, swing_criteria)
    }

    pub fn continuous_primary_action(&self) -> Result<ContinuousPrimaryAction, EngineFailureDto> {
        let state = self.lock();
        match_reservation::require_unreserved(&state)?;
        if state.continuous_departing
            || state.finite_admission_pending
            || matches!(state.phase, Phase::Stopping(_))
            || current_selected_job(&state).is_some_and(|job| job.state == AnalysisJobStateDto::Stopping)
        {
            return Err(continuous_invalid_state(
                &state,
                "continuous analysis is waiting for cancellation or departure cleanup",
            ));
        }
        if let Phase::Error { failure, .. } = &state.phase {
            return Err(failure.clone());
        }
        let enabled = state.continuous_intent.ok_or_else(|| {
            continuous_invalid_state(&state, "continuous analysis preference is still loading")
        })?;
        let stopping_owned_continuous = enabled
            && current_selected_job(&state)
                .is_some_and(|job| job.mode == AnalysisJobModeDto::Continuous && !job.state.is_limited());
        if !stopping_owned_continuous {
            validate_continuous_capabilities(&state)?;
        }
        if current_selected_job(&state).is_some_and(|job| job.mode == AnalysisJobModeDto::Finite) {
            return Ok(if enabled {
                ContinuousPrimaryAction::Stop
            } else {
                ContinuousPrimaryAction::Start
            });
        }
        if !enabled {
            return Ok(ContinuousPrimaryAction::Start);
        }
        if state.continuous_limited.is_some()
            || state.continuous_paused.is_some()
            || state.continuous_error
            || state.continuous_safety_hold
        {
            return Ok(ContinuousPrimaryAction::Resume);
        }
        Ok(ContinuousPrimaryAction::Stop)
    }

    pub fn authorize_continuous_start(&self) {
        {
            let mut state = self.lock();
            if state.match_reservation.is_some() {
                return;
            }
            if state.continuous_departing
                || state.finite_admission_pending
                || current_selected_job(&state).is_some_and(|job| job.state == AnalysisJobStateDto::Stopping)
                || validate_continuous_capabilities(&state).is_err()
            {
                return;
            }
            clear_resumable_holds(&mut state);
            publish_snapshot(&mut state);
        }
        self.reconcile_continuous();
    }

    pub fn resume_continuous(&self) -> Result<(), EngineFailureDto> {
        {
            let mut state = self.lock();
            validate_continuous_capabilities(&state)?;
            if state.continuous_intent != Some(true)
                || state.continuous_departing
                || current_selected_job(&state).is_some_and(|job| !job.state.is_limited())
                || state.continuous_target.is_none()
                || continuous_empty_board(&state)
            {
                return Err(continuous_invalid_state(
                    &state,
                    "continuous analysis Resume requires an enabled intent, Ready Run, current position, and an idle selected-node lane",
                ));
            }
            clear_resumable_holds(&mut state);
            publish_snapshot(&mut state);
        }
        self.reconcile_continuous();
        Ok(())
    }

    pub fn begin_continuous_departure(&self) {
        let mut state = self.lock();
        if state.match_reservation.is_some() {
            return;
        }
        if !state.continuous_departing {
            state.continuous_departing = true;
            state.game_move_publication = None;
            if let Some(identity) = state.game_move.as_ref().map(|slot| slot.identity.clone()) {
                game_move::seal_move_for_run(&mut state, &identity.run_id);
            }
            if let Some(job_id) = state.analysis_task.as_ref().map(|task| task.job_id.clone()) {
                cancel_analysis_task_locked(&mut state, &job_id);
                state.jobs.retain(|job| !(job.job_id == job_id && job.terminal));
            }
            publish_snapshot(&mut state);
        }
    }

    pub fn finish_continuous_departure(&self, committed_replacement: bool) {
        {
            let mut state = self.lock();
            if state.match_reservation.is_some() {
                return;
            }
            state.continuous_departing = false;
            if committed_replacement {
                // Import changes the position, not the user's authorization after a failure.
                clear_weak_holds_for_new_position(&mut state);
            } else {
                state.continuous_safety_hold = true;
            }
            publish_snapshot(&mut state);
        }
        if committed_replacement {
            self.reconcile_continuous();
        }
    }

    pub fn start(&self, profile_id: &str) -> Result<(), EngineFailureDto> {
        self.begin_start(EngineOperationDto::Start, profile_id)
    }

    pub fn autoload(&self, profile_id: &str) -> Result<(), EngineFailureDto> {
        self.begin_start(EngineOperationDto::Autoload, profile_id)
    }

    pub fn apply_autoload(&self) -> Result<(), EngineFailureDto> {
        match_reservation::require_unreserved(&self.lock())?;
        let Some(profile_id) = self.inner.catalog.autoload_profile_id() else {
            return Ok(());
        };
        self.autoload(&profile_id)
    }
    fn begin_start(
        &self,
        operation_kind: EngineOperationDto,
        profile_id: &str,
    ) -> Result<(), EngineFailureDto> {
        match_reservation::require_unreserved(&self.lock())?;
        let saved = match self.inner.catalog.get(profile_id) {
            Some(saved) => saved,
            None => {
                let published = failure(
                    operation_kind,
                    EngineFailureKind::ProfileNotFound,
                    format!("saved engine profile was not found: {profile_id}"),
                    None,
                    Some(profile_id),
                    None,
                );
                self.inner.record_no_engine_failure(published.clone());
                return Err(published);
            }
        };

        let (operation, run) = {
            let mut state = self.lock();
            match_reservation::require_unreserved(&state)?;
            if !matches!(state.phase, Phase::NoEngine { .. }) {
                return Err(failure(
                    operation_kind,
                    EngineFailureKind::InvalidState,
                    "Start requires an authoritative No-engine snapshot".into(),
                    current_run_id(&state.phase).as_deref(),
                    Some(profile_id),
                    None,
                ));
            }
            self.yield_preloads_locked(&mut state);
            state.operation += 1;
            state.operation_kind = operation_kind;
            let run = starting_run(&saved);
            state.phase = Phase::Starting(run.clone());
            publish_snapshot(&mut state);
            (state.operation, run)
        };
        let inner = self.inner.clone();
        thread::spawn(move || inner.run_start(operation, run));
        Ok(())
    }

    pub fn stop(&self) -> Result<(), EngineFailureDto> {
        let Some(operation) = self.begin_stop(EngineOperationDto::Stop)? else {
            return Ok(());
        };
        let inner = self.inner.clone();
        thread::spawn(move || {
            if inner.terminate_live(operation).is_ok() {
                inner.force_no_engine(operation);
            }
        });
        Ok(())
    }

    pub fn teardown(&self) -> Result<(), EngineFailureDto> {
        {
            let mut state = self.lock();
            evaluation::close(&mut state)?;
        }
        self.cancel_all_preloads(true)?;
        if let Some(owner) = self.match_reservation_owner() {
            self.stop_reserved_match(&owner)?;
        }
        self.reap_retiring_runs()?;
        let Some(operation) = self.begin_stop(EngineOperationDto::Teardown)? else {
            return Ok(());
        };
        self.inner.terminate_live(operation)?;
        self.inner.force_no_engine(operation);
        Ok(())
    }

    pub fn restart(&self) -> Result<(), EngineFailureDto> {
        let (operation, profile_id) = {
            let mut state = self.lock();
            match_reservation::require_unreserved(&state)?;
            let run = match &state.phase {
                Phase::Ready(run) | Phase::Error { run, .. } => run.clone(),
                _ => {
                    return Err(failure(
                        EngineOperationDto::Restart,
                        EngineFailureKind::InvalidState,
                        "Restart requires a Ready or Error Foreground Engine Run".into(),
                        current_run_id(&state.phase).as_deref(),
                        None,
                        None,
                    ));
                }
            };
            self.inner.catalog.get(&run.profile_id).ok_or_else(|| {
                failure(
                    EngineOperationDto::Restart,
                    EngineFailureKind::ProfileNotFound,
                    format!("saved engine profile was not found: {}", run.profile_id),
                    Some(&run.run_id),
                    Some(&run.profile_id),
                    None,
                )
            })?;
            self.yield_preloads_locked(&mut state);
            state.operation += 1;
            let profile_id = run.profile_id.clone();
            state.phase = Phase::Stopping(run);
            invalidate_analysis_task_locked(
                &mut state,
                "The Foreground Engine Run restarted; start a new analysis task.",
            );
            cancel_jobs_for_current(&mut state);
            publish_snapshot(&mut state);
            (state.operation, profile_id)
        };
        let manager = self.clone();
        thread::spawn(move || {
            if manager.inner.terminate_live(operation).is_err() {
                return;
            }
            if manager.inner.current_operation() != operation {
                return;
            }
            manager.inner.force_no_engine(operation);
            if manager.inner.current_operation() != operation {
                return;
            }
            let _ = manager.start(&profile_id);
        });
        Ok(())
    }

    pub fn switch_to(&self, profile_id: &str) -> Result<(), EngineFailureDto> {
        match_reservation::require_unreserved(&self.lock())?;
        if self.switch_preload(profile_id)? {
            return Ok(());
        }
        let saved = self.inner.catalog.get(profile_id).ok_or_else(|| {
            failure(
                EngineOperationDto::Switch,
                EngineFailureKind::ProfileNotFound,
                format!("saved engine profile was not found: {profile_id}"),
                None,
                Some(profile_id),
                None,
            )
        })?;
        if self.finish_move_before_switch()? {
            return self.start(profile_id);
        }
        let (operation, candidate, switch_id) = {
            let mut state = self.lock();
            match_reservation::require_unreserved(&state)?;
            game_move::require_idle_move(&state)?;
            let primary = match &state.phase {
                Phase::Ready(run) => run.clone(),
                Phase::Switching { primary, .. } => primary.clone(),
                _ => {
                    return Err(failure(
                        EngineOperationDto::Switch,
                        EngineFailureKind::InvalidState,
                        "Switch requires a Ready Foreground Engine Run".into(),
                        current_run_id(&state.phase).as_deref(),
                        Some(profile_id),
                        None,
                    ));
                }
            };
            if primary.profile_id == profile_id {
                return Ok(());
            }
            if let Some(mut previous) = state.candidate.take() {
                close_live_stdin(&previous);
                let _ = kill_timed_out_child(&mut previous.child);
            }
            self.yield_preloads_locked(&mut state);
            state.operation += 1;
            state.operation_kind = EngineOperationDto::Switch;
            state.switch_seq += 1;
            let switch_id = state.switch_seq.to_string();
            let candidate = starting_run(&saved);
            state.phase = Phase::Switching {
                primary,
                candidate: candidate.clone(),
                switch_id: switch_id.clone(),
            };
            publish_snapshot(&mut state);
            (state.operation, candidate, switch_id)
        };
        let inner = self.inner.clone();
        thread::spawn(move || inner.run_switch_candidate(operation, candidate, switch_id));
        Ok(())
    }

    pub fn register_job(
        &self,
        run_id: &str,
        lane: AnalysisJobLane,
        cancel: Arc<dyn AnalysisJobCancel>,
    ) -> Result<String, EngineFailureDto> {
        let mut state = self.lock();
        match_reservation::require_unreserved(&state)?;
        game_move::require_idle_move(&state)?;
        let run = admitting_run(&state.phase, run_id).ok_or_else(|| {
            continuous_invalid_state(
                &state,
                "Analysis Job admission requires a Ready Foreground Engine Run",
            )
        })?;
        let capabilities = analysis_capabilities(&run)?;
        match lane {
            AnalysisJobLane::SelectedNode => require_capability(
                &run,
                capabilities.selected_node_analysis,
                "selected-node analysis",
            )?,
            AnalysisJobLane::WholeGame => {
                require_capability(&run, capabilities.whole_game_analysis, "whole-game analysis")?
            }
        }
        evaluation::yield_to_foreground(&mut state)?;
        let job_id = Uuid::new_v4().to_string();
        self.yield_preloads_locked(&mut state);
        state.jobs.push(RegisteredJob {
            job_id: job_id.clone(),
            query_id: job_id.clone(),
            submitted: true,
            run_id: run_id.to_string(),
            lane,
            generation: 0,
            mode: AnalysisJobModeDto::Finite,
            continuous_budget: None,
            state: AnalysisJobStateDto::Queued,
            cancel_deadline: None,
            cleanup_deadline: None,
            submitted_at: Instant::now(),
            node_path: NodePath { indices: Vec::new() },
            board_width: 19,
            board_height: 19,
            cancel,
            disposition: JobDisposition::Running,
            expected: None,
            work_items: Vec::new(),
            current_index: 0,
            pending_ending_conditions: Vec::new(),
            terminal: false,
        });
        Ok(job_id)
    }

    pub fn start_selected_node_job(
        &self,
        request: SelectedNodeJobRequest,
    ) -> Result<AnalysisJobStartedDto, EngineFailureDto> {
        let old = {
            let mut state = self.lock();
            validate_selected_admission(&state, &request)?;
            if state.finite_admission_pending {
                return Err(continuous_invalid_state(
                    &state,
                    "selected-node admission is already pending",
                ));
            }
            let old = current_selected_job(&state).filter(|job| !job.state.is_limited());
            if old
                .as_ref()
                .is_some_and(|job| job.state == AnalysisJobStateDto::Stopping)
            {
                return Err(failure(
                    EngineOperationDto::Job,
                    EngineFailureKind::Occupied,
                    "selected-node cancellation is still pending".into(),
                    Some(&request.run_id),
                    None,
                    None,
                ));
            }
            state.finite_admission_pending = true;
            publish_snapshot(&mut state);
            old
        };

        if let Some(old) = old {
            if let Err(error) = self
                .request_job_stop(&old.run_id, &old.job_id, JobDisposition::Superseded)
                .and_then(|()| {
                    self.wait_for_job_cancellation(&old.run_id, &old.job_id, Duration::from_secs(5))
                })
            {
                {
                    let mut state = self.lock();
                    state.finite_admission_pending = false;
                    publish_snapshot(&mut state);
                }
                self.reconcile_continuous();
                return Err(error);
            }
        }

        let result = (|| {
            let mut state = self.lock();
            state.finite_admission_pending = false;
            validate_selected_admission(&state, &request)?;
            if current_selected_job(&state).is_some_and(|job| !job.state.is_limited()) {
                return Err(continuous_invalid_state(
                    &state,
                    "selected-node lane changed during admission",
                ));
            }
            self.register_selected_locked(&mut state, request)
        })();
        match result {
            Ok(submission) => self.submit_registered_selected(submission),
            Err(error) => {
                let mut state = self.lock();
                publish_snapshot(&mut state);
                drop(state);
                self.reconcile_continuous();
                Err(error)
            }
        }
    }

    fn register_selected_locked(
        &self,
        state: &mut ManagerState,
        mut request: SelectedNodeJobRequest,
    ) -> Result<SelectedSubmission, EngineFailureDto> {
        let run = validate_selected_admission(state, &request)?;
        evaluation::yield_to_foreground(state)?;
        state
            .jobs
            .retain(|job| !(job.lane == AnalysisJobLane::SelectedNode && job.terminal));
        if request.mode == AnalysisJobModeDto::Continuous {
            request.query.continuous(state.continuous_budget);
        } else {
            request.query.report_during_search_every = Some(0.1);
        }
        let job_id = Uuid::new_v4().to_string();
        request.query.id = job_id.clone();
        let started = AnalysisJobStartedDto {
            run_id: request.run_id.clone(),
            job_id: job_id.clone(),
            lane: AnalysisJobLaneDto::SelectedNode,
            mode: request.mode,
            state: AnalysisJobStateDto::Queued,
            generation: request.generation,
            node_path: request.node_path.clone(),
        };
        let command = if run.adapter_kind == EngineBackend::KataGoGtp {
            SelectedCommand::Gtp(request.clone())
        } else {
            SelectedCommand::Jsonl(request.query.to_jsonl().map_err(|error| {
            failure(
                EngineOperationDto::Job,
                EngineFailureKind::Protocol,
                format!("failed to serialize selected-node query: {error}"),
                Some(started.run_id.as_str()),
                None,
                None,
            )
            .with_job_id(&started.job_id)
            })?)
        };
        self.yield_preloads_locked(state);
        state.jobs.push(RegisteredJob {
            query_id: job_id.clone(),
            job_id,
            run_id: request.run_id,
            lane: AnalysisJobLane::SelectedNode,
            mode: request.mode,
            continuous_budget: (request.mode == AnalysisJobModeDto::Continuous)
                .then_some(state.continuous_budget),
            state: AnalysisJobStateDto::Queued,
            cancel_deadline: None,
            cleanup_deadline: None,
            submitted: false,
            submitted_at: Instant::now(),
            generation: request.generation,
            node_path: request.node_path,
            board_width: request.board_width,
            board_height: request.board_height,
            cancel: Arc::new(AnalysisCancelToken::new()),
            disposition: JobDisposition::Running,
            expected: Some(1),
            work_items: Vec::new(),
            current_index: 0,
            pending_ending_conditions: Vec::new(),
            terminal: false,
        });
        publish_event(
            state,
            ForegroundEngineEventDto::Job {
                job: selected_node_job_event(&started, AnalysisJobOutcomeDto::Started, None, None),
            },
        );
        publish_snapshot(state);
        Ok(SelectedSubmission {
            started,
            command,
        })
    }

    fn submit_registered_selected(
        &self,
        submission: SelectedSubmission,
    ) -> Result<AnalysisJobStartedDto, EngineFailureDto> {
        let jsonl = match submission.command {
            SelectedCommand::Gtp(request) => {
                self.spawn_gtp_analysis(submission.started.clone(), request);
                return Ok(submission.started);
            }
            SelectedCommand::Jsonl(jsonl) => jsonl,
        };
        if let Err(error) = self.write_live_jsonl(&submission.started.run_id, &jsonl) {
            let published = error.with_job_id(&submission.started.job_id);
            self.abandon_job(
                &submission.started.run_id,
                &submission.started.job_id,
                published.clone(),
            );
            return Err(published);
        }
        {
            let mut state = self.lock();
            if let Some(job) = state.jobs.iter_mut().find(|job| {
                job.run_id == submission.started.run_id && job.job_id == submission.started.job_id
            }) {
                job.submitted = true;
            }
        }
        Ok(submission.started)
    }

    pub fn start_whole_game_analysis(
        &self,
        request: WholeGameJobRequest,
    ) -> Result<AnalysisJobStartedDto, EngineFailureDto> {
        let Some(first) = request.work_items.first() else {
            return Err(empty_whole_game_failure(&request.run_id));
        };
        let total_visits = first.query.max_visits.unwrap_or(0);
        let scope = AnalysisScopeDto {
            mode: AnalysisScopeModeDto::FirstChildMainline,
            current_node: first.node_path.clone(),
            branch_choices: Vec::new(),
            interval: None,
            to_play: None,
        };
        let conditions = single_stage_conditions(total_visits);
        let task = self.start_analysis_task(request, scope, conditions)?;
        let node_path = task
            .requested
            .first()
            .cloned()
            .unwrap_or(NodePath { indices: Vec::new() });
        Ok(AnalysisJobStartedDto {
            run_id: task.run_id,
            job_id: task.job_id,
            lane: AnalysisJobLaneDto::WholeGame,
            mode: AnalysisJobModeDto::Finite,
            state: AnalysisJobStateDto::Queued,
            generation: task.generation,
            node_path,
        })
    }

    pub fn start_analysis_task(
        &self,
        request: WholeGameJobRequest,
        scope: AnalysisScopeDto,
        conditions: AnalysisStageConditionsDto,
    ) -> Result<AnalysisTaskDto, EngineFailureDto> {
        conditions
            .validate_single_stage()
            .map_err(|message| invalid_task_conditions(&request.run_id, message))?;
        self.start_analysis_task_with_strategy(
            request,
            scope,
            AnalysisTaskStrategyDto::SingleStage,
            None,
            conditions,
            AnalysisTaskTargets::default(),
        )
    }

    pub fn start_all_positions_analysis_task(
        &self,
        request: WholeGameJobRequest,
        scope: AnalysisScopeDto,
        overview_conditions: AnalysisStageConditionsDto,
        deep_conditions: AnalysisStageConditionsDto,
    ) -> Result<AnalysisTaskDto, EngineFailureDto> {
        AnalysisStageConditionsDto::validate_all_positions_two_stage(&overview_conditions, &deep_conditions)
            .map_err(|message| invalid_task_conditions(&request.run_id, message))?;
        self.start_analysis_task_with_strategy(
            request,
            scope,
            AnalysisTaskStrategyDto::AllPositionsTwoStage,
            Some(overview_conditions),
            deep_conditions,
            AnalysisTaskTargets::default(),
        )
    }

    pub fn start_swing_analysis_task(
        &self,
        request: SwingAnalysisTaskRequest,
    ) -> Result<AnalysisTaskDto, EngineFailureDto> {
        let SwingAnalysisTaskRequest {
            job: request,
            scope,
            requested,
            supporting,
            swing_comparisons,
            swing_criteria,
            overview_conditions,
            deep_conditions,
        } = request;
        overview_conditions
            .validate_single_stage()
            .and_then(|_| deep_conditions.validate_single_stage())
            .and_then(|_| swing_criteria.validate())
            .map_err(|message| invalid_task_conditions(&request.run_id, message))?;
        self.start_analysis_task_with_strategy(
            request,
            scope,
            AnalysisTaskStrategyDto::SwingSelectedTwoStage,
            Some(overview_conditions),
            deep_conditions,
            AnalysisTaskTargets {
                requested: Some(requested),
                supporting,
                swing_comparisons,
                swing_criteria: Some(swing_criteria),
            },
        )
    }

    fn start_analysis_task_with_strategy(
        &self,
        mut request: WholeGameJobRequest,
        scope: AnalysisScopeDto,
        strategy: AnalysisTaskStrategyDto,
        overview_conditions: Option<AnalysisStageConditionsDto>,
        conditions: AnalysisStageConditionsDto,
        targets: AnalysisTaskTargets,
    ) -> Result<AnalysisTaskDto, EngineFailureDto> {
        if request.work_items.is_empty() {
            return Err(empty_whole_game_failure(&request.run_id));
        }
        let initial_conditions = overview_conditions.as_ref().unwrap_or(&conditions);
        for item in &mut request.work_items {
            item.query.task(initial_conditions);
        }

        let cancel = AnalysisCancelToken::new();
        let (task, bound_query) = {
            let mut state = self.lock();
            match_reservation::require_unreserved(&state)?;
            game_move::require_idle_move(&state)?;
            let run = match admitting_run(&state.phase, &request.run_id) {
                Some(run) => run,
                None => {
                    return Err(failure(
                        EngineOperationDto::Job,
                        EngineFailureKind::InvalidState,
                        "Analysis Job admission requires a Ready Foreground Engine Run".into(),
                        Some(request.run_id.as_str()),
                        None,
                        None,
                    ));
                }
            };
            validate_whole_game_capabilities(&run, targets.swing_criteria.as_ref())?;
            validate_task_conditions_capabilities(&run, &conditions)?;
            if let Some(overview) = overview_conditions.as_ref() {
                validate_task_conditions_capabilities(&run, overview)?;
            }
            for item in &request.work_items {
                validate_query_capabilities(&run, &item.query, false)?;
            }
            if state.analysis_task.as_ref().is_some_and(|task| {
                matches!(
                    task.state,
                    AnalysisTaskStateDto::Pausing | AnalysisTaskStateDto::Paused
                )
            }) {
                return Err(failure(
                    EngineOperationDto::Job,
                    EngineFailureKind::Occupied,
                    "The paused analysis task still owns this lane; Continue or Cancel it.".into(),
                    Some(&request.run_id),
                    None,
                    None,
                ));
            }
            if let Some(occupied) = state.jobs.iter().find(|job| {
                job.run_id == request.run_id && job.lane == AnalysisJobLane::WholeGame && !job.terminal
            }) {
                return Err(failure(
                    EngineOperationDto::Job,
                    EngineFailureKind::Occupied,
                    "whole-game analysis is already running on this Foreground Engine Run".into(),
                    Some(request.run_id.as_str()),
                    Some(run.profile_id.as_str()),
                    None,
                )
                .with_job_id(&occupied.job_id));
            }

            let task_id = Uuid::new_v4().to_string();
            let job_id = Uuid::new_v4().to_string();
            let query_id = target_query_id(&job_id);
            let first_node_path = request.work_items[0].node_path.clone();
            let first_board_width = request.work_items[0].board_width;
            let first_board_height = request.work_items[0].board_height;
            let bound_query = bound_work_item_query(&request.work_items[0], &query_id, &job_id)?;
            evaluation::yield_to_foreground(&mut state)?;
            let started = AnalysisJobStartedDto {
                run_id: request.run_id.clone(),
                job_id: job_id.clone(),
                lane: AnalysisJobLaneDto::WholeGame,
                mode: AnalysisJobModeDto::Finite,
                state: AnalysisJobStateDto::Queued,
                generation: request.generation,
                node_path: first_node_path.clone(),
            };
            let requested = targets.requested.unwrap_or_else(|| {
                request
                    .work_items
                    .iter()
                    .map(|item| item.node_path.clone())
                    .collect()
            });
            let task = AnalysisTaskDto {
                task_id,
                run_id: request.run_id.clone(),
                job_id: job_id.clone(),
                generation: request.generation,
                scope,
                strategy,
                stage: if strategy == AnalysisTaskStrategyDto::SingleStage {
                    AnalysisTaskStageDto::SingleStage
                } else {
                    AnalysisTaskStageDto::Overview
                },
                conditions,
                overview_conditions,
                requested,
                supporting: targets.supporting,
                swing_comparisons: targets.swing_comparisons,
                swing_criteria: targets.swing_criteria,
                selected_for_deep: None,
                overview_completed: Vec::new(),
                completed: Vec::new(),
                overview_summaries: Vec::new(),
                ending_conditions: Vec::new(),
                state: AnalysisTaskStateDto::Queued,
                reason: None,
            };
            let expected = request.work_items.len();
            self.yield_preloads_locked(&mut state);
            state.jobs.push(RegisteredJob {
                job_id: job_id.clone(),
                query_id,
                run_id: request.run_id.clone(),
                lane: AnalysisJobLane::WholeGame,
                mode: AnalysisJobModeDto::Finite,
                continuous_budget: None,
                state: AnalysisJobStateDto::Queued,
                cancel_deadline: None,
                cleanup_deadline: None,
                submitted: false,
                submitted_at: Instant::now(),
                generation: request.generation,
                node_path: first_node_path,
                board_width: first_board_width,
                board_height: first_board_height,
                cancel: Arc::new(cancel),
                disposition: JobDisposition::Running,
                expected: Some(expected),
                work_items: request.work_items,
                current_index: 0,
                pending_ending_conditions: Vec::new(),
                terminal: false,
            });
            state.analysis_task = Some(task.clone());
            publish_event(
                &mut state,
                ForegroundEngineEventDto::Job {
                    job: whole_game_job_event(
                        &started,
                        AnalysisJobOutcomeDto::Started,
                        started.node_path.clone(),
                        Some(0),
                        Some(expected),
                        Some(expected),
                        None,
                        None,
                    ),
                },
            );
            publish_snapshot(&mut state);
            (task, bound_query)
        };
        self.dispatch_whole_game_query(task.run_id.clone(), task.job_id.clone(), bound_query);
        Ok(task)
    }

    pub fn pause_analysis_task(
        &self,
        run_id: &str,
        task_id: &str,
    ) -> Result<AnalysisTaskDto, EngineFailureDto> {
        let mut state = self.lock();
        match_reservation::require_unreserved(&state)?;
        let task = state
            .analysis_task
            .as_ref()
            .filter(|task| task.run_id == run_id && task.task_id == task_id)
            .ok_or_else(|| {
                continuous_invalid_state(&state, "Analysis task identity is no longer current.")
            })?;
        if matches!(
            task.state,
            AnalysisTaskStateDto::Pausing | AnalysisTaskStateDto::Paused
        ) {
            return Ok(task.clone());
        }
        if !matches!(
            task.state,
            AnalysisTaskStateDto::Queued | AnalysisTaskStateDto::Searching
        ) {
            return Err(continuous_invalid_state(
                &state,
                "Pause requires an active analysis task.",
            ));
        }
        let job_id = task.job_id.clone();
        self.request_job_stop_locked(&mut state, run_id, &job_id, JobDisposition::Paused)?;
        Ok(state.analysis_task.as_ref().unwrap().clone())
    }

    pub fn continue_analysis_task(
        &self,
        run_id: &str,
        task_id: &str,
        generation: u64,
    ) -> Result<AnalysisTaskDto, EngineFailureDto> {
        let (task, jsonl) = {
            let mut state = self.lock();
            match_reservation::require_unreserved(&state)?;
            let task = state
                .analysis_task
                .as_ref()
                .filter(|task| {
                    task.run_id == run_id
                        && task.task_id == task_id
                        && task.generation == generation
                        && task.state == AnalysisTaskStateDto::Paused
                })
                .ok_or_else(|| {
                    continuous_invalid_state(&state, "Continue requires the same paused task and game.")
                })?;
            if admitting_run(&state.phase, run_id).is_none()
                || state.continuous_departing
                || state.continuous_safety_hold
            {
                return Err(continuous_invalid_state(
                    &state,
                    "Continue is blocked by Run or departure state.",
                ));
            }
            let run = admitting_run(&state.phase, run_id).expect("Ready Run checked above");
            validate_whole_game_capabilities(&run, task.swing_criteria.as_ref())?;
            validate_task_conditions_capabilities(&run, &task.conditions)?;
            if let Some(overview) = task.overview_conditions.as_ref() {
                validate_task_conditions_capabilities(&run, overview)?;
            }
            let old_job_id = task.job_id.clone();
            let index = state
                .jobs
                .iter()
                .position(|job| job.job_id == old_job_id && job.run_id == run_id && job.terminal)
                .ok_or_else(|| continuous_invalid_state(&state, "Paused task cleanup has not finished."))?;
            for item in &state.jobs[index].work_items[state.jobs[index].current_index..] {
                validate_query_capabilities(&run, &item.query, false)?;
            }
            evaluation::yield_to_foreground(&mut state)?;
            let job_id = Uuid::new_v4().to_string();
            let query_id = target_query_id(&job_id);
            let job = &mut state.jobs[index];
            let jsonl = bound_work_item_query(&job.work_items[job.current_index], &query_id, &job_id)?;
            job.job_id = job_id.clone();
            job.query_id = query_id;
            job.state = AnalysisJobStateDto::Queued;
            job.submitted = false;
            job.submitted_at = Instant::now();
            job.cancel_deadline = None;
            job.cleanup_deadline = None;
            job.cancel = Arc::new(AnalysisCancelToken::new());
            job.disposition = JobDisposition::Running;
            job.pending_ending_conditions.clear();
            job.terminal = false;
            let started = started_from(job);
            let counts = whole_game_progress_counts(job);
            let task = state.analysis_task.as_mut().unwrap();
            task.job_id = job_id;
            task.state = AnalysisTaskStateDto::Queued;
            task.reason = None;
            let task = task.clone();
            publish_event(
                &mut state,
                ForegroundEngineEventDto::Job {
                    job: whole_game_job_event(
                        &started,
                        AnalysisJobOutcomeDto::Started,
                        started.node_path.clone(),
                        counts.0,
                        counts.1,
                        counts.2,
                        None,
                        None,
                    ),
                },
            );
            publish_snapshot(&mut state);
            (task, jsonl)
        };
        self.dispatch_whole_game_query(task.run_id.clone(), task.job_id.clone(), jsonl);
        Ok(task)
    }

    fn dispatch_whole_game_query(&self, run_id: String, job_id: String, jsonl: String) {
        let manager = self.clone();
        // Neither the current-game holder nor the stdout deadline pump waits for pipe IO.
        thread::spawn(move || {
            if let Err(error) = manager.submit_whole_game_query(&run_id, &job_id, &jsonl) {
                manager.abandon_job(&run_id, &job_id, error.with_job_id(&job_id));
            }
        });
    }

    fn submit_whole_game_query(
        &self,
        run_id: &str,
        job_id: &str,
        jsonl: &str,
    ) -> Result<(), EngineFailureDto> {
        loop {
            let mut state = self.lock();
            let Some(index) = state.jobs.iter().position(|job| {
                job.run_id == run_id
                    && job.job_id == job_id
                    && !job.terminal
                    && job.disposition == JobDisposition::Running
                    && !job.submitted
            }) else {
                return Ok(());
            };
            let stdin = engine_stdin(&state, run_id)
                .ok_or_else(|| continuous_invalid_state(&state, "Run process is not available."))?;
            match stdin.try_lock() {
                Ok(mut guard) => {
                    let writer = guard
                        .as_mut()
                        .ok_or_else(|| continuous_invalid_state(&state, "Run stdin is closed."))?;
                    // Commit delivery atomically with cancellation admission. The stdin guard
                    // orders this query before its terminate; blocking IO holds no owner lock.
                    state.jobs[index].submitted = true;
                    drop(state);
                    return write_jsonl(writer, jsonl).map_err(|error| {
                        failure(
                            EngineOperationDto::Job,
                            EngineFailureKind::Protocol,
                            format!("failed to write analysis query: {error}"),
                            Some(run_id),
                            None,
                            None,
                        )
                    });
                }
                Err(std::sync::TryLockError::Poisoned(_)) => {
                    return Err(continuous_invalid_state(&state, "Run stdin lock is poisoned."));
                }
                Err(std::sync::TryLockError::WouldBlock) => {}
            }
            drop(state);
            thread::sleep(Duration::from_millis(1));
        }
    }

    pub fn invalidate_analysis_task(&self, generation: u64) {
        let mut state = self.lock();
        let Some(task) = state.analysis_task.as_ref() else {
            return;
        };
        if task.generation == generation
            || matches!(
                task.state,
                AnalysisTaskStateDto::Cancelled | AnalysisTaskStateDto::Invalidated
            )
        {
            return;
        }
        let run_id = task.run_id.clone();
        let job_id = task.job_id.clone();
        invalidate_analysis_task_locked(&mut state, "The current game changed; start a new analysis task.");
        if state
            .jobs
            .iter()
            .any(|job| job.run_id == run_id && job.job_id == job_id && !job.terminal)
        {
            let _ = self.request_job_stop_locked(&mut state, &run_id, &job_id, JobDisposition::Cancelled);
        } else {
            publish_snapshot(&mut state);
        }
    }

    pub fn wait_for_job_cancellation(
        &self,
        run_id: &str,
        job_id: &str,
        budget: Duration,
    ) -> Result<(), EngineFailureDto> {
        let deadline = Instant::now() + budget.min(Duration::from_secs(5));
        {
            let mut state = self.lock();
            if let Some(job) = state
                .jobs
                .iter_mut()
                .find(|job| job.run_id == run_id && job.job_id == job_id)
            {
                let cleanup_deadline = Instant::now() + budget;
                job.cleanup_deadline = Some(
                    job.cleanup_deadline
                        .map_or(cleanup_deadline, |old| old.min(cleanup_deadline)),
                );
                if let Some(old) = job.cancel_deadline {
                    job.cancel_deadline = Some(old.min(deadline));
                }
            }
        }
        loop {
            {
                let state = self.lock();
                if let Phase::Error { failure, .. } = &state.phase {
                    return Err(failure.clone());
                }
                if !state
                    .jobs
                    .iter()
                    .any(|job| job.run_id == run_id && job.job_id == job_id && !job.terminal)
                {
                    return Ok(());
                }
            }
            if Instant::now() >= deadline {
                return Err(self.inner.fail_unresponsive_run(
                    run_id,
                    "target cancellation did not finish within its deadline",
                ));
            }
            thread::sleep(Duration::from_millis(10));
        }
    }

    pub fn cancel_job(&self, run_id: &str, job_id: &str) -> Result<(), EngineFailureDto> {
        let mut state = self.lock();
        match_reservation::require_unreserved(&state)?;
        if state.analysis_task.as_ref().is_some_and(|task| {
            task.run_id == run_id && task.job_id == job_id && task.state == AnalysisTaskStateDto::Paused
        }) {
            cancel_analysis_task_locked(&mut state, job_id);
            state.jobs.retain(|job| job.job_id != job_id);
            publish_snapshot(&mut state);
            return Ok(());
        }
        self.request_job_stop_locked(&mut state, run_id, job_id, JobDisposition::Cancelled)
    }

    fn request_job_stop(
        &self,
        run_id: &str,
        job_id: &str,
        disposition: JobDisposition,
    ) -> Result<(), EngineFailureDto> {
        self.request_job_stop_locked(&mut self.lock(), run_id, job_id, disposition)
    }

    fn request_job_stop_locked(
        &self,
        state: &mut ManagerState,
        run_id: &str,
        job_id: &str,
        disposition: JobDisposition,
    ) -> Result<(), EngineFailureDto> {
        let query_id = {
            let Some(index) = state
                .jobs
                .iter()
                .position(|job| job.job_id == job_id && job.run_id == run_id && !job.terminal)
            else {
                return Err(failure(
                    EngineOperationDto::Job,
                    EngineFailureKind::InvalidState,
                    "cancel requires a non-terminal job on this Run".into(),
                    Some(run_id),
                    None,
                    None,
                )
                .with_job_id(job_id));
            };
            if disposition == JobDisposition::Cancelled {
                cancel_analysis_task_locked(state, job_id);
            }
            let job = &mut state.jobs[index];
            if job.lane == AnalysisJobLane::WholeGame && !job.submitted {
                let started = started_from(job);
                let counts = whole_game_progress_counts(job);
                if disposition == JobDisposition::Paused {
                    state.analysis_task.as_mut().unwrap().state = AnalysisTaskStateDto::Pausing;
                }
                finish_whole_game_job(
                    state,
                    &started,
                    match disposition {
                        JobDisposition::TimedOut => AnalysisJobOutcomeDto::Timeout,
                        JobDisposition::Superseded => AnalysisJobOutcomeDto::Superseded,
                        _ => AnalysisJobOutcomeDto::Cancelled,
                    },
                    counts.0,
                    counts.1,
                    counts.2,
                    None,
                );
                publish_snapshot(state);
                return Ok(());
            }
            if job.state == AnalysisJobStateDto::Stopping {
                if job.disposition == JobDisposition::BudgetReached {
                    job.disposition = disposition;
                    job.pending_ending_conditions.clear();
                    if disposition == JobDisposition::Paused {
                        let task = state.analysis_task.as_mut().unwrap();
                        task.state = AnalysisTaskStateDto::Pausing;
                        task.reason = None;
                    }
                    publish_snapshot(state);
                }
                return Ok(());
            }
            job.cancel.cancel();
            job.disposition = disposition;
            job.state = AnalysisJobStateDto::Stopping;
            job.cancel_deadline = Some(Instant::now() + Duration::from_secs(5));
            let started = started_from(job);
            let counts = whole_game_progress_counts(job);
            let query_id = job.query_id.clone();
            let event = if job.lane == AnalysisJobLane::SelectedNode {
                selected_node_job_event(&started, AnalysisJobOutcomeDto::Stopping, None, None)
            } else {
                whole_game_job_event(
                    &started,
                    AnalysisJobOutcomeDto::Stopping,
                    started.node_path.clone(),
                    counts.0,
                    counts.1,
                    counts.2,
                    None,
                    None,
                )
            };
            if disposition == JobDisposition::Paused {
                state.analysis_task.as_mut().unwrap().state = AnalysisTaskStateDto::Pausing;
            }
            publish_event(state, ForegroundEngineEventDto::Job { job: event });
            publish_snapshot(state);
            query_id
        };
        if admitting_run(&state.phase, run_id).is_some_and(|run| run.adapter_kind == EngineBackend::KataGoGtp) {
            // The registered stream worker owns Stop and both terminal barriers.
            return Ok(());
        }
        let manager = self.clone();
        let run_id = run_id.to_string();
        let job_id = job_id.to_string();
        thread::spawn(move || {
            manager.write_terminate_after_submission(&run_id, &job_id, &query_id);
        });
        Ok(())
    }

    pub fn set_capability_snapshot_for_tests(&self, snapshot: Option<EngineCapabilitySnapshotDto>) {
        let mut state = self.lock();
        if state.match_reservation.is_some() {
            return;
        }
        if let Phase::Ready(run) = &mut state.phase {
            run.capability_snapshot = snapshot;
        }
    }

    fn abandon_job(&self, run_id: &str, job_id: &str, published: EngineFailureDto) {
        let mut state = self.lock();
        let Some(started) = state
            .jobs
            .iter()
            .find(|job| job.job_id == job_id && job.run_id == run_id && !job.terminal)
            .map(started_from)
        else {
            state
                .jobs
                .retain(|job| !(job.job_id == job_id && job.run_id == run_id));
            return;
        };
        if started.lane == AnalysisJobLane::WholeGame {
            let (completed, expected, remaining) = state
                .jobs
                .iter()
                .find(|job| job.job_id == started.job_id && job.run_id == started.run_id)
                .map(whole_game_progress_counts)
                .unwrap_or((None, None, None));
            finish_whole_game_job(
                &mut state,
                &started,
                AnalysisJobOutcomeDto::Failed,
                completed,
                expected,
                remaining,
                Some(published),
            );
        } else {
            finish_selected_node_job(
                &mut state,
                &started,
                Some(AnalysisJobOutcomeDto::Failed),
                None,
                Some(published),
            );
        }
        publish_snapshot(&mut state);
    }

    fn write_terminate_after_submission(&self, run_id: &str, job_id: &str, query_id: &str) {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            let submitted_query_id = {
                let state = self.lock();
                let Some(job) = state.jobs.iter().find(|job| {
                    job.run_id == run_id && job.job_id == job_id && job.query_id == query_id && !job.terminal
                }) else {
                    return;
                };
                job.submitted.then(|| job.query_id.clone())
            };
            if let Some(query_id) = submitted_query_id {
                let jsonl = katago_protocol::terminate_action_jsonl(&Uuid::new_v4().to_string(), &query_id);
                if self.write_live_jsonl(run_id, &jsonl).is_err() {
                    self.inner
                        .fail_unresponsive_run(run_id, "target cancellation could not be delivered");
                }
                return;
            }
            if Instant::now() >= deadline {
                self.inner
                    .fail_unresponsive_run(run_id, "target cancellation waited for an unsubmitted query");
                return;
            }
            thread::sleep(Duration::from_millis(1));
        }
    }

    fn write_live_jsonl(&self, run_id: &str, jsonl: &str) -> Result<(), EngineFailureDto> {
        let stdin = {
            let state = self.lock();
            engine_stdin(&state, run_id).ok_or_else(|| {
                failure(
                    EngineOperationDto::Job,
                    EngineFailureKind::InvalidState,
                    "Foreground Engine Run process is not available".into(),
                    Some(run_id),
                    None,
                    None,
                )
            })?
        };
        let mut guard = stdin.lock().map_err(|_| {
            failure(
                EngineOperationDto::Job,
                EngineFailureKind::Protocol,
                "engine stdin lock was poisoned".into(),
                Some(run_id),
                None,
                None,
            )
        })?;
        let stdin = guard.as_mut().ok_or_else(|| {
            failure(
                EngineOperationDto::Job,
                EngineFailureKind::Protocol,
                "engine process stdin was already closed".into(),
                Some(run_id),
                None,
                None,
            )
        })?;
        write_jsonl(stdin, jsonl).map_err(|error| {
            failure(
                EngineOperationDto::Job,
                EngineFailureKind::Protocol,
                format!("failed to write analysis query: {error}"),
                Some(run_id),
                None,
                None,
            )
        })
    }

    fn reconcile_continuous(&self) {
        let work = {
            let mut state = self.lock();
            if state.match_reservation.is_some() {
                return;
            }
            if state.continuous_departing
                || state.finite_admission_pending
                || state.game_move.is_some()
                || state.continuous_target.is_none()
            {
                return;
            }
            if let Some(active) = current_selected_job(&state).filter(|job| !job.state.is_limited()) {
                let target = state
                    .continuous_target
                    .as_ref()
                    .expect("target presence checked before reconciliation");
                if same_position(
                    active.generation,
                    &active.node_path,
                    target.generation,
                    &target.node_path,
                ) {
                    return;
                }
                ContinuousReconcileWork::Cancel(active)
            } else {
                if validate_continuous_capabilities(&state).is_err() {
                    return;
                }
                let current_run_id = current_admitting_run(&state.phase).map(|run| run.run_id);
                let limited_matches = state.continuous_limited.as_ref().is_some_and(|(hold, _)| {
                    hold.matches_target(current_run_id.as_deref(), state.continuous_target.as_ref())
                });
                let paused_matches = state.continuous_paused.as_ref().is_some_and(|hold| {
                    hold.matches_target(current_run_id.as_deref(), state.continuous_target.as_ref())
                });
                if state.continuous_limited.is_some() && !limited_matches {
                    state.continuous_limited = None;
                }
                if state.continuous_paused.is_some() && !paused_matches {
                    state.continuous_paused = None;
                }
                let limited = state.continuous_limited.as_ref().map(|(hold, _)| hold.clone());
                state.jobs.retain(|job| {
                    !(job.lane == AnalysisJobLane::SelectedNode
                        && job.terminal
                        && limited.as_ref().is_none_or(|hold| !hold.matches_job(job)))
                });
                if state.continuous_intent != Some(true)
                    || state.continuous_error
                    || state.continuous_safety_hold
                    || state.continuous_limited.is_some()
                    || state.continuous_paused.is_some()
                    || continuous_empty_board(&state)
                {
                    return;
                }
                let Some(mut target) = state.continuous_target.clone() else {
                    return;
                };
                let Some(run) = current_admitting_run(&state.phase) else {
                    return;
                };
                target.run_id = run.run_id;
                match self.register_selected_locked(&mut state, target) {
                    Ok(submission) => ContinuousReconcileWork::Submit(submission),
                    Err(published) => {
                        state.continuous_error = true;
                        publish_event(
                            &mut state,
                            ForegroundEngineEventDto::Failure { failure: published },
                        );
                        publish_snapshot(&mut state);
                        return;
                    }
                }
            }
        };
        match work {
            ContinuousReconcileWork::Cancel(job) => {
                let _ = self.request_job_stop(&job.run_id, &job.job_id, JobDisposition::Superseded);
            }
            ContinuousReconcileWork::Submit(submission) => {
                if let Err(published) = self.submit_registered_selected(submission) {
                    let mut state = self.lock();
                    state.continuous_error = true;
                    publish_event(
                        &mut state,
                        ForegroundEngineEventDto::Failure { failure: published },
                    );
                    publish_snapshot(&mut state);
                }
            }
        }
    }

    pub fn assert_profile_deletable(&self, profile_id: &str) -> Result<(), EngineFailureDto> {
        let state = self.lock();
        match_reservation::assert_reserved_profile_deletable(&state, profile_id)?;
        let blocked = match &state.phase {
            Phase::Starting(run) | Phase::Ready(run) | Phase::Stopping(run) => Some(run),
            Phase::Switching {
                primary, candidate, ..
            } => {
                if primary.profile_id == profile_id {
                    Some(primary)
                } else {
                    Some(candidate)
                }
            }
            Phase::Error { run, .. } => Some(run),
            Phase::NoEngine { .. } => None,
        };
        if let Some(run) = blocked {
            if run.profile_id == profile_id {
                return Err(failure(
                    EngineOperationDto::DeleteProfile,
                    EngineFailureKind::ProfileInUse,
                    "an active or candidate Foreground Engine Run still holds this profile identity".into(),
                    Some(run.run_id.as_str()),
                    Some(profile_id),
                    None,
                ));
            }
        }
        Ok(())
    }

    fn begin_stop(&self, _operation_kind: EngineOperationDto) -> Result<Option<u64>, EngineFailureDto> {
        let mut state = self.lock();
        match_reservation::require_unreserved(&state)?;
        self.yield_preloads_locked(&mut state);
        let run = match &state.phase {
            Phase::NoEngine { .. } => return Ok(None),
            Phase::Starting(run) | Phase::Ready(run) | Phase::Stopping(run) => run.clone(),
            Phase::Switching { primary, .. } => primary.clone(),
            Phase::Error { run, .. } => run.clone(),
        };
        state.operation += 1;
        state.phase = Phase::Stopping(run);
        invalidate_analysis_task_locked(
            &mut state,
            "The Foreground Engine Run stopped; start a new analysis task.",
        );
        cancel_jobs_for_current(&mut state);
        publish_snapshot(&mut state);
        Ok(Some(state.operation))
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, ManagerState> {
        self.inner.state.lock().expect("foreground engine manager lock")
    }
}

impl Inner {
    fn lock(&self) -> std::sync::MutexGuard<'_, ManagerState> {
        self.state.lock().expect("foreground engine manager lock")
    }

    fn current_operation(&self) -> u64 {
        self.lock().operation
    }

    fn run_start(self: Arc<Self>, operation: u64, run: EngineRunDto) {
        if self.current_operation() != operation {
            return;
        }
        if let Err(mut published) = self.start_resident(operation, &run, false, false) {
            identify_failed_executable(&run, &mut published);
            self.fail_attempt(operation, published);
        }
    }

    fn run_switch_candidate(self: Arc<Self>, operation: u64, run: EngineRunDto, switch_id: String) {
        if self.current_operation() != operation {
            return;
        }
        if let Err(mut published) = self.start_resident(operation, &run, true, false) {
            identify_failed_executable(&run, &mut published);
            self.fail_switch_candidate(operation, &run, &switch_id, published);
        }
    }

    fn record_diagnostic_failure(&self, failure: &EngineFailureDto) {
        let attempts = self.diagnostics.lock().expect("diagnostic capture list");
        if let Some(capture) = attempts.iter().find(|capture| failure.run_id.as_deref().is_some_and(|id| capture.matches_run(id))) {
            capture.record("failure", &format!("{:?}: {}", failure.kind, failure.message));
        }
    }

    fn start_resident(
        self: &Arc<Self>,
        operation: u64,
        run: &EngineRunDto,
        as_candidate: bool,
        background: bool,
    ) -> Result<(), EngineFailureDto> {
        let kind = if background { EngineOperationDto::Start } else { self.lock().operation_kind };
        let capture = AttemptCapture::new(run);
        let protected_run = match &self.lock().phase {
            Phase::Ready(run) | Phase::Switching { primary: run, .. } => Some(run.run_id.clone()),
            _ => None,
        };
        {
            let mut attempts = self.diagnostics.lock().expect("diagnostic capture list");
            if attempts.len() == 4 {
                let oldest_retired = attempts.iter().position(|attempt| !protected_run.as_deref().is_some_and(|id| attempt.matches_run(id))).unwrap_or(0);
                attempts.remove(oldest_retired);
            }
            attempts.push_back(capture.clone());
        }
        let missing: Vec<_> = check_assets(&run.profile_snapshot)
            .into_iter()
            .filter(|check| check.required && !check.exists)
            .collect();
        if !missing.is_empty() {
            let summary = missing
                .iter()
                .map(|check| {
                    if check.path.is_empty() {
                        check.label.clone()
                    } else {
                        format!("{} ({})", check.label, check.path)
                    }
                })
                .collect::<Vec<_>>()
                .join(", ");
            return Err(failure(
                kind,
                EngineFailureKind::Asset,
                format!("required engine assets are missing: {summary}"),
                Some(run.run_id.as_str()),
                Some(run.profile_id.as_str()),
                Some(summary),
            ));
        }
        if !self.preparation_current(operation, run, background) {
            return Ok(());
        }

        let spec = build_command_spec(&run.profile_snapshot).map_err(|error| {
            failure(
                kind,
                EngineFailureKind::Start,
                error.to_string(),
                Some(run.run_id.as_str()),
                Some(run.profile_id.as_str()),
                None,
            )
        })?;
        let resource_deadline = Instant::now() + Duration::from_secs(30);
        let resources =
            ResourceSnapshot::capture(run, &spec, resource_deadline, capture.clone(), self.config.managed_resources_root.as_deref()).map_err(|(cause, message)| {
                failure(
                    kind,
                    cause,
                    message,
                    Some(&run.run_id),
                    Some(&run.profile_id),
                    None,
                )
            })?;
        if !self.preparation_current(operation, run, background) {
            return Ok(());
        }
        capture.command(&spec);
        let mut child = build_process_command(&spec).spawn().map_err(|error| {
            failure(
                kind,
                EngineFailureKind::Start,
                format!("failed to spawn engine process: {error}"),
                Some(run.run_id.as_str()),
                Some(run.profile_id.as_str()),
                None,
            )
        })?;
        let stdin = child.stdin.take().ok_or_else(|| {
            failure(
                kind,
                EngineFailureKind::Start,
                "engine process stdin was not piped".into(),
                Some(run.run_id.as_str()),
                Some(run.profile_id.as_str()),
                None,
            )
        })?;
        let stdout = child.stdout.take().ok_or_else(|| {
            failure(
                kind,
                EngineFailureKind::Start,
                "engine process stdout was not piped".into(),
                Some(run.run_id.as_str()),
                Some(run.profile_id.as_str()),
                None,
            )
        })?;
        let stderr = child.stderr.take().ok_or_else(|| {
            failure(
                kind,
                EngineFailureKind::Start,
                "engine process stderr was not piped".into(),
                Some(run.run_id.as_str()),
                Some(run.profile_id.as_str()),
                None,
            )
        })?;
        let generic = run.adapter_kind != EngineBackend::KataGoAnalysis;
        let stdout = ObservedRead::new(stdout, capture.clone(), "stdout");
        let stderr = ObservedRead::new(stderr, capture.clone(), "stderr");
        let stdout_rx = if generic {
            gtp::spawn_stdout_reader(stdout)
        } else {
            spawn_stdout_lines_reader(stdout)
        };
        let startup_output = StartupOutput::capture(stderr);
        {
            let mut state = self.lock();
            let process_id = child.id();
            let engine = LiveEngine {
                child,
                stdin: Arc::new(Mutex::new(Some(stdin))),
                stdout_rx: Some(stdout_rx),
                startup_output: startup_output.clone(),
                run_id: run.run_id.clone(),
                process_id,
                capture: capture.clone(),
            };
            if state.operation != operation || (background && !preload::preparing(&state, &run.run_id)) {
                // Keep even a late-spawned child manager-owned until its exit is confirmed.
                state.retiring.push((run.clone(), Arc::new(Mutex::new(engine))));
                drop(state);
                let manager = ForegroundEngineManager {
                    inner: Arc::clone(self),
                };
                thread::spawn(move || {
                    let _ = manager.reap_retiring_runs();
                });
                return Ok(());
            }
            if background {
                state
                    .preloads
                    .iter_mut()
                    .find(|slot| slot.dto.run.run_id == run.run_id)
                    .expect("admitted preload")
                    .live = Some(engine);
            } else if as_candidate {
                state.candidate = Some(engine);
            } else {
                state.live = Some(engine);
            }
        }
        let probe_id = format!("lifecycle-readiness-{}", run.run_id);
        if !generic {
            let query = AnalysisQuery {
                id: probe_id.clone(),
                moves: Vec::new(),
                initial_stones: Vec::new(),
                rules: "chinese".into(),
                komi: 7.5,
                board_x_size: 19,
                board_y_size: 19,
                analyze_turns: Some(vec![0]),
                max_visits: Some(2),
                include_ownership: None,
                include_policy: None,
                report_during_search_every: None,
                override_settings: None,
            };
            let result = query
                .to_jsonl()
                .map_err(|error| error.to_string())
                .and_then(|jsonl| {
                    let stdin = engine_stdin(&self.lock(), &run.run_id)
                        .ok_or_else(|| "readiness process was retired".to_owned())?;
                    let mut guard = stdin
                        .lock()
                        .map_err(|_| "engine stdin lock poisoned".to_owned())?;
                    let input = guard.as_mut().ok_or_else(|| "engine stdin closed".to_owned())?;
                    write_jsonl(input, &jsonl).map_err(|error| error.to_string())
                });
            if let Err(error) = result {
                return Err(failure(
                    kind,
                    EngineFailureKind::Protocol,
                    format!("failed to send readiness probe: {error}"),
                    Some(&run.run_id),
                    Some(&run.profile_id),
                    None,
                ));
            }
        }

        let gtp = if generic {
            let Some(facts) = self.await_gtp_readiness(operation, run, kind)? else {
                return Ok(());
            };
            Some(facts)
        } else {
            self.await_readiness(operation, run, &probe_id, kind)?;
            None
        };
        capture.readiness_confirmed();
        let capabilities = EngineCapabilitySnapshotDto {
            adapter_kind: run.adapter_kind,
            game_move: !generic
                || gtp.as_ref().is_some_and(|facts| {
                    crate::game_move_protocol::qualified_gtp_launch(&run.profile_snapshot, facts)
                }),
            analysis: if generic { gtp.as_ref().and_then(|facts| gtp_analysis::capabilities(run, facts)) } else { Some(EngineAnalysisCapabilitiesDto {
                selected_node_analysis: true,
                continuous_analysis: true,
                whole_game_analysis: self.config.admit_whole_game_analysis,
                candidates: true,
                pv: true,
                winrate: true,
                root_score: true,
                ownership: true,
                policy: true,
                visits_limit: true,
                protocol_cancel: true,
            }) },
            gtp,
        };
        resources
            .revalidate(Instant::now() + Duration::from_secs(30))
            .map_err(|(cause, message)| {
                failure(
                    kind,
                    cause,
                    message,
                    Some(&run.run_id),
                    Some(&run.profile_id),
                    None,
                )
            })?;
        // Match resume intentionally uses the reservation's immutable launch snapshot.
        if self.lock().match_reservation.is_none()
            && !self
                .catalog
                .get(&run.profile_id)
                .is_some_and(|saved| saved.profile == run.profile_snapshot)
        {
            return Err(failure(
                kind,
                EngineFailureKind::ResourceChanged,
                "Saved profile changed during qualification; Start again explicitly".into(),
                Some(&run.run_id),
                Some(&run.profile_id),
                None,
            ));
        }
        let (version, backend) = startup_output
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .identity();
        let version = capabilities
            .gtp
            .as_ref()
            .map(|facts| facts.version.clone())
            .or(version);
        let mut qualified_run = run.clone();
        let preload_resources = background.then(|| resources.clone());
        qualified_run.qualified_resource = Some(resources.qualified(version, backend));
        let run = &qualified_run;
        if background {
            return self.finish_preload(
                operation,
                qualified_run,
                capabilities,
                preload_resources.expect("background resources"),
            );
        }
        if as_candidate {
            let mut state = self.lock();
            if state.match_reservation.is_some() {
                if state.operation == operation {
                    if let Some(reservation) = state.match_reservation.as_mut() {
                        if !reservation.sealed {
                            if let Some(ready) = reservation
                                .runs
                                .iter_mut()
                                .find(|ready| ready.run_id == run.run_id)
                            {
                                ready.capability_snapshot = Some(capabilities);
                                ready.qualified_resource = run.qualified_resource.clone();
                                reservation.ready = reservation
                                    .runs
                                    .iter()
                                    .all(|run| run.capability_snapshot.is_some());
                            }
                        }
                    }
                }
            } else {
                drop(state);
                self.promote_candidate(operation, run, capabilities)?;
            }
        } else {
            self.admit_ready(operation, run, capabilities);
        }
        Ok(())
    }
    fn reconcile_continuous(self: &Arc<Self>) {
        ForegroundEngineManager {
            inner: Arc::clone(self),
        }
        .reconcile_continuous();
    }

    fn await_gtp_readiness(
        &self,
        operation: u64,
        run: &EngineRunDto,
        kind: EngineOperationDto,
    ) -> Result<Option<EngineGtpFactsDto>, EngineFailureDto> {
        let fail = |failure_kind, message| {
            failure(
                kind,
                failure_kind,
                message,
                Some(&run.run_id),
                Some(&run.profile_id),
                None,
            )
        };
        // One deadline for the whole handshake; traffic never extends it.
        let deadline = Instant::now() + self.config.readiness_timeout;
        let mut bodies = Vec::with_capacity(4);
        for (index, command) in ["protocol_version", "name", "version", "list_commands"]
            .iter()
            .enumerate()
        {
            let id = (index + 1) as u32;
            {
                let mut state = self.lock();
                if state.operation != operation {
                    return Ok(None);
                }
                let Some(live) = owned_live_mut(&mut state, &run.run_id) else {
                    return Ok(None);
                };
                let mut guard = live.stdin.lock().expect("engine stdin lock");
                let stdin = guard
                    .as_mut()
                    .ok_or_else(|| fail(EngineFailureKind::ProcessExit, "GTP stdin closed".into()))?;
                // Four short serial commands fit in the OS pipe even if the child never reads.
                write_jsonl(stdin, &format!("{id} {command}\n")).map_err(|error| {
                    fail(
                        EngineFailureKind::ProcessExit,
                        format!("GTP command write failed: {error}"),
                    )
                })?;
            }
            let mut decoder = ResponseDecoder::new(id);
            loop {
                if Instant::now() >= deadline {
                    return Err(fail(
                        EngineFailureKind::Timeout,
                        format!("GTP readiness timed out waiting for {command}"),
                    ));
                }
                let received = {
                    let mut state = self.lock();
                    if state.operation != operation {
                        return Ok(None);
                    }
                    let Some(live) = owned_live_mut(&mut state, &run.run_id) else {
                        return Ok(None);
                    };
                    if let Some(status) = live.child.try_wait().map_err(|error| {
                        fail(
                            EngineFailureKind::ProcessExit,
                            format!("GTP process observation failed: {error}"),
                        )
                    })? {
                        return Err(fail(
                            EngineFailureKind::ProcessExit,
                            format!("GTP exited during {command}: {status}"),
                        ));
                    }
                    live.stdout_rx.as_ref().expect("readiness owns stdout").try_recv()
                };
                match received {
                    Ok(Ok(Some(line))) => {
                        if let Some(response) = decoder
                            .push(&line)
                            .map_err(|message| fail(EngineFailureKind::Protocol, message))?
                        {
                            if !response.success {
                                return Err(fail(
                                    EngineFailureKind::Command,
                                    format!("GTP {command} rejected: {}", response.body),
                                ));
                            }
                            bodies.push(response.body);
                            break;
                        }
                    }
                    Ok(Err(error)) => {
                        return Err(fail(
                            EngineFailureKind::Protocol,
                            format!("GTP response read failed: {error}"),
                        ))
                    }
                    Ok(Ok(None)) | Err(mpsc::TryRecvError::Disconnected) => {
                        return Err(fail(
                            EngineFailureKind::ProcessExit,
                            format!("GTP stdout closed during {command}"),
                        ))
                    }
                    Err(mpsc::TryRecvError::Empty) => thread::sleep(Duration::from_millis(5)),
                }
            }
            if index == 0 && bodies[0].trim() != "2" {
                return Err(fail(
                    EngineFailureKind::UnsupportedCapability,
                    "GenericGtp requires protocol version 2".into(),
                ));
            }
        }
        let mut bodies = bodies.into_iter();
        let _protocol_version = bodies.next();
        let name = bodies.next().expect("name response");
        let version = bodies.next().expect("version response");
        if name.trim().is_empty() || version.trim().is_empty() {
            return Err(fail(
                EngineFailureKind::Protocol,
                "GTP name/version must not be empty".into(),
            ));
        }
        let command_body = bodies.next().expect("commands response");
        let mut commands = Vec::new();
        for line in command_body.lines() {
            let command = line.trim();
            if command.is_empty() || !command.bytes().all(|b| b.is_ascii_graphic()) {
                return Err(fail(
                    EngineFailureKind::Protocol,
                    "GTP list_commands contains an invalid command name".into(),
                ));
            }
            if !commands.iter().any(|existing| existing == command) {
                commands.push(command.to_owned());
            }
        }
        for required in ["boardsize", "clear_board", "komi", "play", "genmove", "quit"] {
            if !commands.iter().any(|command| command == required) {
                return Err(fail(
                    EngineFailureKind::UnsupportedCapability,
                    format!("GTP required command is missing: {required}"),
                ));
            }
        }
        if run.adapter_kind == EngineBackend::KataGoGtp
            && (name.trim() != "KataGo"
                || !version.trim().split('+').next().is_some_and(|v| v == "1.18.2")
                || [
                    "loadsgf",
                    "showboard",
                    "printsgf",
                    "kata-get-rules",
                    "kata-set-rules",
                    "get_komi",
                ]
                .iter()
                .any(|required| !commands.iter().any(|command| command == required)))
        {
            return Err(fail(EngineFailureKind::UnsupportedCapability,
                "KataGo GTP rules confirmation requires the qualified 1.18.2 protocol and exact-position commands".into()));
        }
        Ok(Some(EngineGtpFactsDto {
            protocol_version: 2,
            name,
            version,
            commands,
        }))
    }

    fn await_readiness(
        &self,
        operation: u64,
        run: &EngineRunDto,
        probe_id: &str,
        kind: EngineOperationDto,
    ) -> Result<(), EngineFailureDto> {
        let deadline = Instant::now() + self.config.readiness_timeout;
        loop {
            if self.current_operation() != operation {
                return Ok(());
            }
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Err(failure(
                    kind,
                    EngineFailureKind::Timeout,
                    "engine readiness probe timed out".into(),
                    Some(run.run_id.as_str()),
                    Some(run.profile_id.as_str()),
                    None,
                ));
            }
            let line = {
                let state = self.lock();
                let live = owned_live(&state, &run.run_id);
                let Some(live) = live else {
                    return Err(failure(
                        kind,
                        EngineFailureKind::Start,
                        "engine process was lost before readiness".into(),
                        Some(run.run_id.as_str()),
                        Some(run.profile_id.as_str()),
                        None,
                    ));
                };
                let Some(stdout_rx) = live.stdout_rx.as_ref() else {
                    return Err(failure(
                        kind,
                        EngineFailureKind::Start,
                        "engine process was lost before readiness".into(),
                        Some(run.run_id.as_str()),
                        Some(run.profile_id.as_str()),
                        None,
                    ));
                };
                match stdout_rx.try_recv() {
                    Ok(Ok(Some(line))) => Some(line),
                    Ok(Ok(None)) => {
                        return Err(failure(
                            kind,
                            EngineFailureKind::Readiness,
                            "engine closed stdout before answering the readiness probe".into(),
                            Some(run.run_id.as_str()),
                            Some(run.profile_id.as_str()),
                            None,
                        ));
                    }
                    Ok(Err(error)) => {
                        return Err(failure(
                            kind,
                            EngineFailureKind::Protocol,
                            format!("failed to read engine stdout: {error}"),
                            Some(run.run_id.as_str()),
                            Some(run.profile_id.as_str()),
                            None,
                        ));
                    }
                    Err(mpsc::TryRecvError::Empty) => None,
                    Err(mpsc::TryRecvError::Disconnected) => {
                        return Err(failure(
                            kind,
                            EngineFailureKind::Readiness,
                            "engine closed stdout before answering the readiness probe".into(),
                            Some(run.run_id.as_str()),
                            Some(run.profile_id.as_str()),
                            None,
                        ));
                    }
                }
            };
            if line.is_none() {
                thread::sleep(Duration::from_millis(20));
            }
            let Some(line) = line else {
                continue;
            };
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            match parse_response_line(trimmed) {
                Ok(response) if response.id == probe_id => return Ok(()),
                Ok(_) => continue,
                Err(error) => {
                    if trimmed.contains(probe_id) {
                        return Err(failure(
                            kind,
                            EngineFailureKind::Protocol,
                            format!("readiness probe response was not parseable: {error}"),
                            Some(run.run_id.as_str()),
                            Some(run.profile_id.as_str()),
                            Some(trimmed.to_string()),
                        ));
                    }
                }
            }
        }
    }

    fn admit_ready(
        self: &Arc<Self>,
        operation: u64,
        run: &EngineRunDto,
        capabilities: EngineCapabilitySnapshotDto,
    ) {
        let mut state = self.lock();
        if state.operation != operation {
            return;
        }
        if !matches!(&state.phase, Phase::Starting(current) if current.run_id == run.run_id) {
            return;
        }
        let mut ready = run.clone();
        ready.capability_snapshot = Some(capabilities);
        invalidate_analysis_task_locked(
            &mut state,
            "A new Foreground Engine Run is ready; start a new analysis task.",
        );
        state.last_primary_profile_id = Some(ready.profile_id.clone());
        state.phase = Phase::Ready(ready);
        clear_holds_for_new_run(&mut state);
        publish_snapshot(&mut state);
        let process_id = state.live.as_ref().map(|live| live.process_id);
        let stdout_rx = state.live.as_mut().and_then(|live| live.stdout_rx.take());
        drop(state);
        let Some(process_id) = process_id else {
            return;
        };
        let inner = self.clone();
        let run_id = run.run_id.clone();
        thread::spawn(move || inner.watch_exit(operation, run_id, process_id));
        if let Some(stdout_rx) = stdout_rx {
            let inner = self.clone();
            let run_id = run.run_id.clone();
            let generic = run.adapter_kind != EngineBackend::KataGoAnalysis;
            thread::spawn(move || {
                if generic {
                    inner.pump_gtp_stdout(run_id, stdout_rx)
                } else {
                    inner.pump_stdout(operation, run_id, stdout_rx)
                }
            });
        }
    }

    fn promote_candidate(
        self: &Arc<Self>,
        operation: u64,
        run: &EngineRunDto,
        capabilities: EngineCapabilitySnapshotDto,
    ) -> Result<(), EngineFailureDto> {
        let retiring = {
            let mut state = self.lock();
            if state.operation != operation {
                return Ok(());
            }
            let Phase::Switching {
                primary,
                candidate,
                switch_id: _,
            } = &state.phase
            else {
                return Ok(());
            };
            if candidate.run_id != run.run_id {
                return Ok(());
            }
            let primary_run_id = primary.run_id.clone();
            if !state.live.as_mut().is_some_and(|live| {
                live.run_id == primary_run_id && matches!(live.child.try_wait(), Ok(None))
            }) || !state
                .candidate
                .as_mut()
                .is_some_and(|live| live.run_id == run.run_id && matches!(live.child.try_wait(), Ok(None)))
            {
                return Err(failure(EngineOperationDto::Switch, EngineFailureKind::ProcessExit,
                    "The captured primary or candidate exited before promotion; no alternate engine was selected".into(),
                    Some(&run.run_id), Some(&run.profile_id), None));
            }
            let mut ready = run.clone();
            ready.capability_snapshot = Some(capabilities);
            invalidate_analysis_task_locked(
                &mut state,
                "The Foreground Engine Run switched; start a new analysis task.",
            );
            cancel_jobs_for_run(&mut state, &primary_run_id);
            let retiring = state.live.take();
            state.live = state.candidate.take();
            state.last_primary_profile_id = Some(ready.profile_id.clone());
            state.phase = Phase::Ready(ready);
            clear_holds_for_new_run(&mut state);
            publish_snapshot(&mut state);
            let process_id = state.live.as_ref().map(|live| live.process_id);
            let stdout_rx = state.live.as_mut().and_then(|live| live.stdout_rx.take());
            drop(state);
            if let Some(stdout_rx) = stdout_rx {
                let inner = self.clone();
                let run_id = run.run_id.clone();
                let generic = run.adapter_kind != EngineBackend::KataGoAnalysis;
                thread::spawn(move || {
                    if generic {
                        inner.pump_gtp_stdout(run_id, stdout_rx)
                    } else {
                        inner.pump_stdout(operation, run_id, stdout_rx)
                    }
                });
            }
            if let Some(process_id) = process_id {
                let inner = self.clone();
                let run_id = run.run_id.clone();
                thread::spawn(move || inner.watch_exit(operation, run_id, process_id));
            }
            retiring
        };
        if let Some(mut live) = retiring {
            thread::sleep(self.config.stop_drain_timeout);
            close_live_stdin(&live);
            let _ = kill_timed_out_child(&mut live.child);
        }
        Ok(())
    }

    fn fail_switch_candidate(
        &self,
        operation: u64,
        run: &EngineRunDto,
        switch_id: &str,
        published: EngineFailureDto,
    ) {
        let mut state = self.lock();
        if state.operation != operation {
            return;
        }
        let Phase::Switching {
            primary,
            candidate,
            switch_id: current_switch,
        } = &state.phase
        else {
            return;
        };
        if candidate.run_id != run.run_id || current_switch != switch_id {
            return;
        }
        let primary = primary.clone();
        let mut published = published.with_switch_id(switch_id);
        if let Some(live) = state.candidate.as_mut() {
            if let Err(error) = terminate_process(live, Instant::now() + self.config.stop_drain_timeout) {
                published
                    .message
                    .push_str(&format!("; candidate cleanup failed: {error}"));
                self.record_diagnostic_failure(&published);
                state.phase = Phase::Error {
                    run: primary,
                    failure: published.clone(),
                };
                publish_snapshot(&mut state);
                publish_event(
                    &mut state,
                    ForegroundEngineEventDto::Failure { failure: published },
                );
                return;
            }
            attach_startup_failure(&mut published, &live.startup_output, &live.capture);
            state.candidate = None;
        }
        self.record_diagnostic_failure(&published);
        state.phase =
            if state.live.as_mut().is_some_and(|live| {
                live.run_id == primary.run_id && matches!(live.child.try_wait(), Ok(None))
            }) {
                Phase::Ready(primary)
            } else {
                Phase::NoEngine {
                    failure: Some(published.clone()),
                }
            };
        publish_snapshot(&mut state);
        publish_event(
            &mut state,
            ForegroundEngineEventDto::Failure { failure: published },
        );
    }

    fn fail_attempt(&self, operation: u64, mut published: EngineFailureDto) {
        let mut state = self.lock();
        if state.operation != operation {
            return;
        }
        let deadline = Instant::now() + self.config.stop_drain_timeout;
        let ManagerState { live, candidate, .. } = &mut *state;
        for slot in [live, candidate] {
            if let Some(process) = slot.as_mut() {
                match terminate_process(process, deadline) {
                    Ok(()) => {
                        attach_startup_failure(&mut published, &process.startup_output, &process.capture);
                        *slot = None;
                    }
                    Err(error) => published
                        .message
                        .push_str(&format!("; process cleanup failed: {error}")),
                }
            }
        }
        self.record_diagnostic_failure(&published);
        cancel_jobs_for_current(&mut state);
        state.jobs.clear();
        state.phase = Phase::NoEngine {
            failure: Some(published.clone()),
        };
        publish_snapshot(&mut state);
        publish_event(
            &mut state,
            ForegroundEngineEventDto::Failure { failure: published },
        );
    }

    fn record_no_engine_failure(&self, published: EngineFailureDto) {
        let mut state = self.lock();
        if !matches!(state.phase, Phase::NoEngine { .. }) {
            return;
        }
        state.phase = Phase::NoEngine {
            failure: Some(published.clone()),
        };
        publish_snapshot(&mut state);
        publish_event(
            &mut state,
            ForegroundEngineEventDto::Failure { failure: published },
        );
    }

    fn terminate_live(&self, operation: u64) -> Result<(), EngineFailureDto> {
        thread::sleep(self.config.stop_drain_timeout);
        let mut state = self.lock();
        if state.operation != operation {
            return Ok(());
        }
        let deadline = Instant::now() + self.config.stop_drain_timeout;
        let mut cleanup_error = None;
        let ManagerState { live, candidate, .. } = &mut *state;
        for slot in [live, candidate] {
            if let Some(process) = slot.as_mut() {
                match terminate_process(process, deadline) {
                    Ok(()) => {
                        *slot = None;
                    }
                    Err(error) => {
                        cleanup_error = Some(error.to_string());
                    }
                }
            }
        }
        if let Some(error) = cleanup_error {
            if let Phase::Stopping(run) = &state.phase {
                let run = run.clone();
                let published = failure(
                    EngineOperationDto::Teardown,
                    EngineFailureKind::Timeout,
                    format!("engine process cleanup failed: {error}"),
                    Some(&run.run_id),
                    None,
                    None,
                );
                state.phase = Phase::Error {
                    run,
                    failure: published.clone(),
                };
                publish_snapshot(&mut state);
                publish_event(
                    &mut state,
                    ForegroundEngineEventDto::Failure {
                        failure: published.clone(),
                    },
                );
                return Err(published);
            }
        }
        Ok(())
    }

    fn force_no_engine(&self, operation: u64) {
        let mut state = self.lock();
        if state.operation != operation {
            return;
        }
        if state.live.is_some() || state.candidate.is_some() {
            return;
        }
        state.jobs.clear();
        state.game_move = None;
        state.game_move_publication = None;
        state.phase = Phase::NoEngine { failure: None };
        publish_snapshot(&mut state);
    }

    fn watch_exit(self: Arc<Self>, operation: u64, run_id: String, process_id: u32) {
        loop {
            thread::sleep(Duration::from_millis(30));
            let status = {
                let mut state = self.lock();
                let Some(live) = owned_live_mut(&mut state, &run_id) else {
                    return;
                };
                if live.process_id != process_id {
                    return;
                }
                match live.child.try_wait() {
                    Ok(Some(status)) => {
                        live.capture.exited(status.code());
                        Some(status.code())
                    }
                    Ok(None) => None,
                    Err(_) => Some(None),
                }
            };
            if let Some(exit_code) = status {
                self.handle_unexpected_exit(operation, &run_id, process_id, exit_code);
                return;
            }
        }
    }

    fn handle_unexpected_exit(&self, operation: u64, run_id: &str, process_id: u32, exit_code: Option<i32>) {
        let mut state = self.lock();
        let Some(live) = owned_live(&state, run_id) else {
            return;
        };
        if live.process_id != process_id {
            return;
        }
        live.capture.record("failure", &format!("process exited; exit_code={exit_code:?}"));
        if match_reservation::handle_reserved_exit(&mut state, run_id, exit_code) {
            return;
        }
        match &state.phase {
            Phase::Starting(run) if run.run_id == run_id => {
                if state.operation != operation {
                    return;
                }
                let profile_id = run.profile_id.clone();
                let operation_kind = state.operation_kind;
                drop(state);
                self.fail_attempt(
                    operation,
                    failure(
                        operation_kind,
                        if exit_code.unwrap_or(0) == 0 {
                            EngineFailureKind::Readiness
                        } else {
                            EngineFailureKind::NonzeroExit
                        },
                        format!("engine exited during start; exit_code={exit_code:?}"),
                        Some(run_id),
                        Some(profile_id.as_str()),
                        None,
                    ),
                );
            }
            Phase::Ready(run) if run.run_id == run_id => {
                let run = run.clone();
                self.enter_primary_error(&mut state, run, exit_code);
            }
            Phase::Switching { primary, .. } if primary.run_id == run_id => {
                let run = primary.clone();
                state.operation += 1;
                if let Some(mut candidate) = state.candidate.take() {
                    close_live_stdin(&candidate);
                    let _ = kill_timed_out_child(&mut candidate.child);
                }
                self.enter_primary_error(&mut state, run, exit_code);
            }
            _ => {}
        }
    }

    fn enter_primary_error(&self, state: &mut ManagerState, run: EngineRunDto, exit_code: Option<i32>) {
        let published = failure(
            EngineOperationDto::UnexpectedExit,
            if run.adapter_kind != EngineBackend::KataGoAnalysis {
                EngineFailureKind::ProcessExit
            } else {
                EngineFailureKind::NonzeroExit
            },
            format!("engine process exited unexpectedly; exit_code={exit_code:?}"),
            Some(run.run_id.as_str()),
            Some(run.profile_id.as_str()),
            None,
        );
        game_move::fail_move_for_run(state, &published);
        fail_analysis_task_locked(state, None, "The Foreground Engine Run exited unexpectedly.");
        state.continuous_safety_hold = true;
        cancel_jobs_for_current(state);
        if let Some(mut live) = state.live.take() {
            let _ = live.child.wait();
        }
        state.phase = Phase::Error {
            run,
            failure: published.clone(),
        };
        publish_snapshot(state);
        publish_event(state, ForegroundEngineEventDto::Failure { failure: published });
    }
    /// Match residents are owned outside `live`, so ownership, not the primary slot, keeps the pump.
    fn pump_gtp_stdout(self: Arc<Self>, run_id: String, stdout_rx: Receiver<io::Result<Option<String>>>) {
        self.reconcile_continuous();
        loop {
            if owned_live(&self.lock(), &run_id).is_none() {
                return;
            }
            let (kind, message) = match stdout_rx.recv_timeout(Duration::from_millis(50)) {
                Err(mpsc::RecvTimeoutError::Timeout) => continue,
                Ok(Ok(Some(line))) => {
                    let routed = self.lock().gtp_dispatch.route(&run_id, &line);
                    match routed {
                        Ok(()) => continue,
                        Err(message) => (EngineFailureKind::Protocol, message),
                    }
                }
                Ok(Err(error)) => (EngineFailureKind::Protocol, format!("GTP stdout failed: {error}")),
                Ok(Ok(None)) | Err(mpsc::RecvTimeoutError::Disconnected) => {
                    (EngineFailureKind::ProcessExit, "GTP stdout closed".into())
                }
            };
            let mut state = self.lock();
            if match_reservation::handle_reserved_stdout_failure(
                &mut state,
                &run_id,
                Instant::now() + self.config.stop_drain_timeout,
                kind,
                &message,
            ) {
                return;
            }
            let Some(run) = admitting_run(&state.phase, &run_id) else {
                return;
            };
            // A failed primary seals its candidate too; stale readers cannot touch a new run.
            state.operation += 1;
            let mut published = failure(
                EngineOperationDto::UnexpectedExit,
                kind,
                message,
                Some(&run_id),
                Some(&run.profile_id),
                None,
            );
            game_move::fail_move_for_run(&mut state, &published);
            state.continuous_safety_hold = true;
            fail_analysis_task_locked(&mut state, None, &published.message);
            let jobs: Vec<_> = state.jobs.iter()
                .filter(|job| job.run_id == run_id && !job.terminal)
                .map(started_from)
                .collect();
            for started in jobs {
                finish_failed_job(&mut state, &started, published.clone().with_job_id(&started.job_id));
            }
            let deadline = Instant::now() + self.config.stop_drain_timeout;
            let ManagerState { live, candidate, gtp_dispatch, .. } = &mut *state;
            for slot in [live, candidate] {
                if let Some(process) = slot.as_mut() {
                    gtp_dispatch.retire_run(&process.run_id);
                    match terminate_process(process, deadline) {
                        Ok(()) => *slot = None,
                        Err(error) => published.message.push_str(&format!("; cleanup failed: {error}")),
                    }
                }
            }
            state.phase = Phase::Error {
                run,
                failure: published.clone(),
            };
            publish_snapshot(&mut state);
            publish_event(
                &mut state,
                ForegroundEngineEventDto::Failure { failure: published },
            );
            return;
        }
    }

    fn pump_stdout(
        self: Arc<Self>,
        _operation: u64,
        run_id: String,
        stdout_rx: Receiver<io::Result<Option<String>>>,
    ) {
        loop {
            self.check_job_deadlines(&run_id);
            let owned = {
                let state = self.lock();
                owned_live(&state, &run_id).is_some()
            };
            if !owned {
                return;
            }
            match stdout_rx.recv_timeout(Duration::from_millis(50)) {
                Ok(Ok(Some(line))) => {
                    if self.route_game_move_line(&run_id, &line) {
                        continue;
                    }
                    if let Some(pending) = self.route_stdout_line(&run_id, line) {
                        self.advance_whole_game(pending);
                    }
                }
                Ok(Ok(None)) | Ok(Err(_)) | Err(mpsc::RecvTimeoutError::Disconnected) => {
                    let mut state = self.lock();
                    if match_reservation::handle_reserved_stdout_failure(
                        &mut state,
                        &run_id,
                        Instant::now() + self.config.stop_drain_timeout,
                        EngineFailureKind::ProcessExit,
                        "The owned match engine stdout closed or failed unexpectedly.",
                    ) {
                        return;
                    }
                    drop(state);
                    thread::sleep(Duration::from_millis(50));
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
            }
            self.reconcile_continuous();
        }
    }

    fn route_stdout_line(&self, run_id: &str, line: String) -> Option<WholeGameAdvance> {
        let trimmed = line.trim();
        let response_id = extract_response_id(trimmed)?;
        let mut state = self.lock();
        let index = state
            .jobs
            .iter()
            .position(|job| job.query_id == response_id && job.run_id == run_id && !job.terminal)?;
        let started = started_from(&state.jobs[index]);
        let response = match parse_response_line(trimmed) {
            Ok(response) => response,
            Err(error) => {
                if started.state == AnalysisJobStateDto::Stopping
                    && state.jobs[index].disposition != JobDisposition::BudgetReached
                {
                    return None;
                }
                let job = &state.jobs[index];
                let (board_width, board_height) = job
                    .work_items
                    .get(job.current_index)
                    .map(|item| (item.board_width, item.board_height))
                    .unwrap_or((job.board_width, job.board_height));
                let capability_refusal = board_dimension_capability_refusal(&error);
                let (kind, message) = match capability_refusal {
                    Some(reason) => (
                        EngineFailureKind::UnsupportedCapability,
                        format!(
                            "The current engine does not support {board_width}×{board_height} boards: {reason}"
                        ),
                    ),
                    None => (
                        EngineFailureKind::Protocol,
                        format!("analysis response was not parseable: {error}"),
                    ),
                };
                let published = failure(EngineOperationDto::Job, kind, message, Some(run_id), None, None)
                    .with_job_id(&started.job_id);
                if capability_refusal.is_none()
                    && (started.mode == AnalysisJobModeDto::Continuous
                        || state.jobs[index].disposition == JobDisposition::BudgetReached)
                {
                    drop(state);
                    self.fail_unresponsive_run(run_id, &published.message);
                    return None;
                }
                finish_failed_job(&mut state, &started, published);
                publish_snapshot(&mut state);
                return None;
            }
        };
        if response.action.is_some() {
            return None;
        }
        if started.state == AnalysisJobStateDto::Stopping
            && state.jobs[index].disposition != JobDisposition::BudgetReached
        {
            if response.is_during_search == Some(false) {
                let outcome = match state.jobs[index].disposition {
                    JobDisposition::Superseded => AnalysisJobOutcomeDto::Superseded,
                    JobDisposition::TimedOut => AnalysisJobOutcomeDto::Timeout,
                    _ => AnalysisJobOutcomeDto::Cancelled,
                };
                let counts = whole_game_progress_counts(&state.jobs[index]);
                if started.lane == AnalysisJobLane::SelectedNode {
                    finish_selected_node_job(&mut state, &started, Some(outcome), None, None);
                } else {
                    finish_whole_game_job(&mut state, &started, outcome, counts.0, counts.1, counts.2, None);
                }
                state.last_activity = Instant::now();
                publish_snapshot(&mut state);
            }
            return None;
        }
        if let Some(warning) = response.warning.as_ref() {
            let ignored_required_setting = [
                "reportDuringSearchEvery",
                "overrideSettings",
                "maxTime",
                "maxVisits",
                "maxPlayouts",
            ]
            .iter()
            .any(|field| {
                response
                    .field
                    .as_deref()
                    .is_some_and(|value| value.contains(field))
                    || warning.contains(field)
            });
            if ignored_required_setting
                && (started.lane == AnalysisJobLane::WholeGame
                    || started.mode == AnalysisJobModeDto::Continuous)
            {
                let message = format!("required analysis setting was ignored: {warning}");
                drop(state);
                self.fail_unresponsive_run(run_id, &message);
            }
            return None;
        }
        if started.lane == AnalysisJobLane::WholeGame {
            let job = &state.jobs[index];
            let invalid_budget_final = job.disposition == JobDisposition::BudgetReached
                && response.is_during_search == Some(false)
                && !response.has_valid_search_result(
                    job.work_items[job.current_index].board_width,
                    job.work_items[job.current_index].board_height,
                );
            if response.is_during_search.is_none() || invalid_budget_final {
                drop(state);
                self.fail_unresponsive_run(run_id, "task response did not establish a valid target final");
                return None;
            }
            return self.route_whole_game_line(&mut state, run_id, response_id, trimmed);
        }
        let during = response.is_during_search == Some(true);
        let visits = response.root_info.as_ref().map_or(0, |root| root.visits);
        let has_result = visits > 0 && !response.no_results;
        let has_data =
            response.has_valid_search_result(state.jobs[index].board_width, state.jobs[index].board_height);
        if has_result && response.is_during_search.is_none() {
            drop(state);
            self.fail_unresponsive_run(
                run_id,
                "selected-node response omitted the required search-progress marker",
            );
            return None;
        }
        if has_result && !has_data {
            drop(state);
            self.fail_unresponsive_run(
                run_id,
                "selected-node response contained invalid analysis values or geometry",
            );
            return None;
        }
        if started.mode == AnalysisJobModeDto::Continuous && !during && !has_result {
            let published = failure(
                EngineOperationDto::Job,
                EngineFailureKind::Protocol,
                "continuous search ended without valid analysis".into(),
                Some(run_id),
                None,
                None,
            )
            .with_job_id(&started.job_id);
            finish_failed_job(&mut state, &started, published);
            publish_snapshot(&mut state);
            return None;
        }
        if during && !has_data {
            return None;
        }
        let frame = has_data.then(|| {
            normalize_response(
                Uuid::parse_str(&started.job_id).expect("manager job UUID"),
                response,
                state.jobs[index].board_width,
                state.jobs[index].board_height,
            )
        });
        if has_data {
            state.last_activity = Instant::now();
            state.jobs[index].state = AnalysisJobStateDto::Searching;
        }
        if during {
            if started.mode == AnalysisJobModeDto::Continuous {
                publish_event(
                    &mut state,
                    ForegroundEngineEventDto::Job {
                        job: selected_node_job_event(&started, AnalysisJobOutcomeDto::Progress, frame, None),
                    },
                );
            }
            publish_snapshot(&mut state);
        } else {
            let outcome = if started.mode == AnalysisJobModeDto::Continuous {
                let budget = state.jobs[index]
                    .continuous_budget
                    .expect("continuous job budget");
                if budget.continuous_visits_limit_enabled && visits >= budget.continuous_visits_limit {
                    AnalysisJobOutcomeDto::VisitsLimited
                } else if budget.continuous_time_limit_enabled {
                    AnalysisJobOutcomeDto::TimeLimited
                } else {
                    let published = failure(
                        EngineOperationDto::Job,
                        EngineFailureKind::Protocol,
                        "continuous search ended before an enabled limit".into(),
                        Some(run_id),
                        None,
                        None,
                    )
                    .with_job_id(&started.job_id);
                    finish_failed_job(&mut state, &started, published);
                    publish_snapshot(&mut state);
                    return None;
                }
            } else {
                AnalysisJobOutcomeDto::Completed
            };
            finish_selected_node_job(&mut state, &started, Some(outcome), frame, None);
            publish_snapshot(&mut state);
        }
        None
    }

    fn route_whole_game_line(
        &self,
        state: &mut ManagerState,
        run_id: &str,
        response_id: String,
        trimmed: &str,
    ) -> Option<WholeGameAdvance> {
        let job_index = state.jobs.iter().position(|job| {
            job.query_id == response_id
                && job.run_id == run_id
                && !job.terminal
                && matches!(
                    job.disposition,
                    JobDisposition::Running | JobDisposition::BudgetReached
                )
                && job.lane == AnalysisJobLane::WholeGame
        })?;
        let started = started_from(&state.jobs[job_index]);
        let current_index = state.jobs[job_index].current_index;
        let item = state.jobs[job_index].work_items.get(current_index)?.clone();
        let expected = state.jobs[job_index].expected?;
        let conditions = state
            .analysis_task
            .as_ref()
            .filter(|task| task.job_id == started.job_id)
            .and_then(active_task_conditions)
            .cloned()?;
        match parse_response_line(trimmed) {
            Ok(response) if response.id == response_id => {
                if response.action.is_some() {
                    return None;
                }
                let during = response.is_during_search == Some(true);
                let has_search_data = response.has_valid_search_result(item.board_width, item.board_height);
                if has_search_data {
                    state.last_activity = Instant::now();
                    if state.jobs[job_index].disposition == JobDisposition::Running {
                        state.jobs[job_index].state = AnalysisJobStateDto::Searching;
                        mark_analysis_task_searching(state, &started.job_id);
                    }
                }

                let observed = if has_search_data {
                    response.observed_task_ending_conditions(&conditions, !during)
                } else {
                    Vec::new()
                };
                if during {
                    if !observed.is_empty() && state.jobs[job_index].disposition == JobDisposition::Running {
                        let job = &mut state.jobs[job_index];
                        job.cancel.cancel();
                        job.disposition = JobDisposition::BudgetReached;
                        job.state = AnalysisJobStateDto::Stopping;
                        job.cancel_deadline = Some(Instant::now() + Duration::from_secs(5));
                        job.pending_ending_conditions = observed.clone();
                        if let Some(task) = state.analysis_task.as_mut() {
                            task.reason =
                                Some(format!("Stopping after reaching {}.", observed.join(" and ")));
                        }
                        let counts = whole_game_progress_counts(job);
                        publish_event(
                            state,
                            ForegroundEngineEventDto::Job {
                                job: whole_game_job_event(
                                    &started,
                                    AnalysisJobOutcomeDto::Stopping,
                                    item.node_path,
                                    counts.0,
                                    counts.1,
                                    counts.2,
                                    None,
                                    None,
                                ),
                            },
                        );
                        publish_snapshot(state);
                        return Some(WholeGameAdvance::Terminate {
                            run_id: started.run_id,
                            job_id: started.job_id,
                            query_id: state.jobs[job_index].query_id.clone(),
                        });
                    }
                    publish_snapshot(state);
                    return None;
                }
                let ending_conditions = if state.jobs[job_index].pending_ending_conditions.is_empty() {
                    observed
                } else {
                    state.jobs[job_index].pending_ending_conditions.clone()
                };
                if !has_search_data || ending_conditions.is_empty() {
                    let published = failure(
                        EngineOperationDto::Job,
                        EngineFailureKind::Protocol,
                        "whole-game search ended without valid analysis reaching an enabled condition".into(),
                        Some(run_id),
                        None,
                        None,
                    )
                    .with_job_id(&started.job_id);
                    finish_failed_job(state, &started, published);
                    publish_snapshot(state);
                    return None;
                }

                let job_uuid = Uuid::parse_str(&started.job_id).unwrap_or_else(|_| Uuid::nil());
                let mut frame = normalize_response(job_uuid, response, item.board_width, item.board_height);
                frame.turn = item.move_number;
                let requires_root_score = state.analysis_task.as_ref().is_some_and(|task| {
                    task.job_id == started.job_id
                        && task.stage == AnalysisTaskStageDto::Overview
                        && task
                            .swing_criteria
                            .as_ref()
                            .is_some_and(|criteria| criteria.score_change_points.enabled)
                });
                if requires_root_score && frame.score_mean_black.is_none() {
                    let published = failure(
                        EngineOperationDto::Job,
                        EngineFailureKind::Protocol,
                        "swing overview result omitted the required root score".into(),
                        Some(run_id),
                        None,
                        None,
                    )
                    .with_job_id(&started.job_id);
                    finish_failed_job(state, &started, published);
                    publish_snapshot(state);
                    return None;
                }
                let completed = current_index + 1;
                let remaining = expected.saturating_sub(completed);
                state.jobs[job_index].current_index = completed;
                state.jobs[job_index].pending_ending_conditions.clear();
                state.jobs[job_index].cancel_deadline = None;
                state.jobs[job_index].disposition = JobDisposition::Running;
                record_analysis_task_completion(
                    state,
                    &started.job_id,
                    &item.node_path,
                    &frame,
                    ending_conditions,
                );
                publish_event(
                    state,
                    ForegroundEngineEventDto::Job {
                        job: whole_game_job_event(
                            &started,
                            AnalysisJobOutcomeDto::Progress,
                            item.node_path.clone(),
                            Some(completed),
                            Some(expected),
                            Some(remaining),
                            Some(frame),
                            None,
                        ),
                    },
                );
                if completed >= expected {
                    let begins_deep = state.analysis_task.as_ref().is_some_and(|task| {
                        task.job_id == started.job_id
                            && task.strategy != AnalysisTaskStrategyDto::SingleStage
                            && task.stage == AnalysisTaskStageDto::Overview
                    });
                    if !begins_deep {
                        finish_whole_game_job(
                            state,
                            &started,
                            AnalysisJobOutcomeDto::Completed,
                            Some(completed),
                            Some(expected),
                            Some(0),
                            None,
                        );
                        return None;
                    }

                    let (deep_conditions, selected, selection_reason) = {
                        let task = state.analysis_task.as_ref().expect("two-stage task");
                        let (selected, reason) =
                            if task.strategy == AnalysisTaskStrategyDto::SwingSelectedTwoStage {
                                let selected = select_swing_deep_paths(task);
                                let reason = selected.is_empty().then(|| swing_zero_selection_reason(task));
                                (selected, reason)
                            } else {
                                (task.requested.clone(), None)
                            };
                        (task.conditions.clone(), selected, reason)
                    };
                    let selected_work_items = selected
                        .iter()
                        .filter_map(|path| {
                            state.jobs[job_index]
                                .work_items
                                .iter()
                                .find(|item| item.node_path == *path)
                                .cloned()
                        })
                        .collect::<Vec<_>>();
                    if selected_work_items.len() != selected.len() {
                        let published = failure(
                            EngineOperationDto::Job,
                            EngineFailureKind::InvalidState,
                            "Frozen swing selection did not match the admitted overview positions.".into(),
                            Some(run_id),
                            None,
                            None,
                        )
                        .with_job_id(&started.job_id);
                        finish_failed_job(state, &started, published);
                        publish_snapshot(state);
                        return None;
                    }
                    {
                        let task = state.analysis_task.as_mut().expect("two-stage task");
                        task.selected_for_deep = Some(selected.clone());
                        task.stage = AnalysisTaskStageDto::Deep;
                        task.state = AnalysisTaskStateDto::Queued;
                        task.reason = selection_reason;
                        task.ending_conditions.clear();
                    }
                    {
                        let job = &mut state.jobs[job_index];
                        job.work_items = selected_work_items;
                        job.expected = Some(selected.len());
                        job.current_index = 0;
                        for work_item in &mut job.work_items {
                            work_item.query.task(&deep_conditions);
                        }
                    }
                    if selected.is_empty() {
                        finish_whole_game_job(
                            state,
                            &started,
                            AnalysisJobOutcomeDto::Completed,
                            Some(0),
                            Some(0),
                            Some(0),
                            None,
                        );
                        return None;
                    }
                }

                let next_index = state.jobs[job_index].current_index;
                let expected = state.jobs[job_index].expected.unwrap_or_default();
                let next = state.jobs[job_index].work_items[next_index].clone();
                let query_id = target_query_id(&started.job_id);
                let jsonl = match bound_work_item_query(&next, &query_id, &started.job_id) {
                    Ok(jsonl) => jsonl,
                    Err(error) => {
                        finish_whole_game_job(
                            state,
                            &started,
                            AnalysisJobOutcomeDto::Failed,
                            Some(next_index),
                            Some(expected),
                            Some(expected.saturating_sub(next_index)),
                            Some(error),
                        );
                        return None;
                    }
                };
                state.jobs[job_index].query_id = query_id;
                state.jobs[job_index].node_path = next.node_path;
                state.jobs[job_index].state = AnalysisJobStateDto::Queued;
                state.jobs[job_index].submitted = false;
                state.jobs[job_index].cancel = Arc::new(AnalysisCancelToken::new());
                state.jobs[job_index].submitted_at = Instant::now();
                if let Some(task) = state.analysis_task.as_mut() {
                    task.state = AnalysisTaskStateDto::Queued;
                    task.reason = None;
                }
                publish_snapshot(state);
                Some(WholeGameAdvance::Submit { started, jsonl })
            }
            Ok(_) => None,
            Err(error) if trimmed.contains(&response_id) => {
                let completed = current_index;
                let remaining = expected.saturating_sub(completed);
                finish_whole_game_job(
                    state,
                    &started,
                    AnalysisJobOutcomeDto::Failed,
                    Some(completed),
                    Some(expected),
                    Some(remaining),
                    Some(
                        failure(
                            EngineOperationDto::Job,
                            EngineFailureKind::Protocol,
                            format!("whole-game response was not parseable: {error}"),
                            Some(started.run_id.as_str()),
                            None,
                            None,
                        )
                        .with_job_id(&started.job_id),
                    ),
                );
                None
            }
            Err(_) => None,
        }
    }

    fn advance_whole_game(self: &Arc<Self>, pending: WholeGameAdvance) {
        let manager = ForegroundEngineManager {
            inner: Arc::clone(self),
        };
        match pending {
            WholeGameAdvance::Submit { started, jsonl } => {
                manager.dispatch_whole_game_query(started.run_id, started.job_id, jsonl);
            }
            WholeGameAdvance::Terminate {
                run_id,
                job_id,
                query_id,
            } => {
                thread::spawn(move || manager.write_terminate_after_submission(&run_id, &job_id, &query_id));
            }
        }
    }

    fn check_job_deadlines(self: &Arc<Self>, run_id: &str) {
        let state = self.lock();
        if state
            .match_reservation
            .as_ref()
            .is_some_and(|reservation| !reservation.committed)
        {
            return;
        }
        if admitting_run(&state.phase, run_id).is_none() {
            return;
        }
        let now = Instant::now();
        let expired_cancel = state.jobs.iter().any(|job| {
            job.run_id == run_id
                && !job.terminal
                && job.cancel_deadline.is_some_and(|deadline| now >= deadline)
        });
        let expired_activity: Vec<_> = state
            .jobs
            .iter()
            .filter(|job| {
                job.run_id == run_id
                    && !job.terminal
                    && job.state != AnalysisJobStateDto::Stopping
                    && now.duration_since(state.last_activity.max(job.submitted_at))
                        >= self.config.job_timeout
            })
            .map(started_from)
            .collect();
        drop(state);
        if expired_cancel {
            self.fail_unresponsive_run(run_id, "target cancellation timed out without a final response");
        } else {
            let manager = ForegroundEngineManager {
                inner: Arc::clone(self),
            };
            for job in expired_activity {
                let _ = manager.request_job_stop(&job.run_id, &job.job_id, JobDisposition::TimedOut);
            }
        }
    }

    fn fail_unresponsive_run(&self, run_id: &str, message: &str) -> EngineFailureDto {
        self.fail_analysis_run(run_id, EngineFailureKind::Timeout, message)
    }

    fn fail_analysis_run(&self, run_id: &str, kind: EngineFailureKind, message: &str) -> EngineFailureDto {
        let mut state = self.lock();
        if let Phase::Error { failure, .. } = &state.phase {
            return failure.clone();
        }
        let mut published = failure(
            EngineOperationDto::Job,
            kind,
            message.into(),
            Some(run_id),
            None,
            None,
        );
        if state
            .match_reservation
            .as_ref()
            .is_some_and(|reservation| !reservation.committed)
        {
            state.continuous_safety_hold = true;
            return published;
        }
        let Some(run) = admitting_run(&state.phase, run_id) else {
            return published;
        };
        state.continuous_safety_hold = true;
        fail_analysis_task_locked(&mut state, None, message);
        state.operation += 1;
        let deadline = state
            .jobs
            .iter()
            .filter(|job| job.run_id == run_id)
            .filter_map(|job| job.cleanup_deadline)
            .fold(Instant::now() + self.config.stop_drain_timeout, Instant::min);
        let jobs: Vec<_> = state
            .jobs
            .iter()
            .filter(|job| job.run_id == run_id && !job.terminal)
            .map(started_from)
            .collect();
        for started in jobs {
            finish_failed_job(
                &mut state,
                &started,
                published.clone().with_job_id(&started.job_id),
            );
        }
        let ManagerState { live, candidate, gtp_dispatch, .. } = &mut *state;
        for slot in [live, candidate] {
            if let Some(live) = slot.as_mut() {
                gtp_dispatch.retire_run(&live.run_id);
                match terminate_process(live, deadline) {
                    Ok(()) => {
                        *slot = None;
                    }
                    Err(error) => {
                        published
                            .message
                            .push_str(&format!("; process cleanup failed: {error}"));
                    }
                }
            }
        }
        state.phase = Phase::Error {
            run,
            failure: published.clone(),
        };
        publish_snapshot(&mut state);
        publish_event(
            &mut state,
            ForegroundEngineEventDto::Failure {
                failure: published.clone(),
            },
        );
        published
    }
}

fn snapshot_from(state: &ManagerState) -> ForegroundEngineSnapshotDto {
    let lifecycle = match &state.phase {
        Phase::NoEngine { failure } => ForegroundEngineLifecycleDto::NoEngine {
            failure: failure.clone(),
        },
        Phase::Starting(run) => ForegroundEngineLifecycleDto::Starting { run: run.clone() },
        Phase::Ready(run) => ForegroundEngineLifecycleDto::Ready { run: run.clone() },
        Phase::Switching {
            primary,
            candidate,
            switch_id,
        } => ForegroundEngineLifecycleDto::Switching {
            primary: primary.clone(),
            candidate: candidate.clone(),
            switch_id: switch_id.clone(),
        },
        Phase::Stopping(run) => ForegroundEngineLifecycleDto::Stopping { run: run.clone() },
        Phase::Error { run, failure } => ForegroundEngineLifecycleDto::Error {
            run: run.clone(),
            failure: failure.clone(),
        },
    };
    let mut snapshot = ForegroundEngineSnapshotDto::with_lifecycle(state.revision, lifecycle);
    snapshot.continuous = continuous_snapshot(state);
    snapshot.selected_node_job = current_non_terminal_job(state, AnalysisJobLane::SelectedNode);
    snapshot.whole_game_job = current_non_terminal_job(state, AnalysisJobLane::WholeGame);
    snapshot.game_move_job = state.game_move.as_ref().map(|slot| slot.identity.clone());
    snapshot
}

fn publish_snapshot(state: &mut ManagerState) {
    state.revision = state.revision.saturating_add(1);
    let snapshot = snapshot_from(state);
    publish_event(state, ForegroundEngineEventDto::Snapshot { snapshot });
}

fn publish_event(state: &mut ManagerState, event: ForegroundEngineEventDto) {
    state
        .subscribers
        .retain(|subscriber| subscriber.send(event.clone()).is_ok());
}

fn cancel_jobs_for_current(state: &mut ManagerState) {
    let Some(run_id) = current_run_id(&state.phase) else {
        return;
    };
    cancel_jobs_for_run(state, &run_id);
}

fn cancel_jobs_for_run(state: &mut ManagerState, run_id: &str) {
    game_move::seal_move_for_run(state, run_id);
    let query_ids: Vec<String> = state
        .jobs
        .iter()
        .filter(|job| job.run_id == run_id && !job.terminal)
        .map(|job| job.query_id.clone())
        .collect();
    let mut to_finish = Vec::new();
    for job in &mut state.jobs {
        if job.run_id == run_id && !job.terminal {
            job.cancel.cancel();
            let counts = (job.lane == AnalysisJobLane::WholeGame).then(|| whole_game_progress_counts(job));
            to_finish.push((started_from(job), counts));
            mark_job_cancelled(job);
        }
    }
    if let Some(live) = state.live.as_ref() {
        if live.run_id == run_id {
            for query_id in &query_ids {
                write_terminate_to_live(live, query_id);
            }
        }
    }
    if let Some(live) = state.candidate.as_ref() {
        if live.run_id == run_id {
            for query_id in &query_ids {
                write_terminate_to_live(live, query_id);
            }
        }
    }
    for (started, counts) in to_finish {
        if let Some((completed, expected, remaining)) = counts {
            finish_whole_game_job(
                state,
                &started,
                AnalysisJobOutcomeDto::Cancelled,
                completed,
                expected,
                remaining,
                None,
            );
        } else {
            finish_selected_node_job(
                state,
                &started,
                Some(AnalysisJobOutcomeDto::Cancelled),
                None,
                None,
            );
        }
    }
    state.jobs.retain(|job| job.run_id != run_id);
}

fn mark_job_cancelled(job: &mut RegisteredJob) {
    job.terminal = true;
    if job.lane == AnalysisJobLane::SelectedNode && job.disposition == JobDisposition::Running {
        job.disposition = JobDisposition::Cancelled;
    }
}

fn selected_node_job_event(
    started: &AnalysisJobStartedDto,
    outcome: AnalysisJobOutcomeDto,
    frame: Option<app_model::AnalysisFrameDto>,
    failure: Option<EngineFailureDto>,
) -> AnalysisJobEventDto {
    AnalysisJobEventDto {
        run_id: started.run_id.clone(),
        job_id: started.job_id.clone(),
        lane: started.lane,
        mode: started.mode,
        generation: started.generation,
        node_path: started.node_path.clone(),
        outcome,
        completed: None,
        expected: None,
        remaining: None,
        frame,
        failure,
        current_game: None,
    }
}

fn whole_game_progress_counts(job: &RegisteredJob) -> (Option<usize>, Option<usize>, Option<usize>) {
    let expected = job.expected;
    let completed = Some(job.current_index);
    let remaining = expected.map(|expected| expected.saturating_sub(job.current_index));
    (completed, expected, remaining)
}

#[allow(clippy::too_many_arguments)]
fn whole_game_job_event(
    started: &AnalysisJobStartedDto,
    outcome: AnalysisJobOutcomeDto,
    node_path: NodePath,
    completed: Option<usize>,
    expected: Option<usize>,
    remaining: Option<usize>,
    frame: Option<app_model::AnalysisFrameDto>,
    failure: Option<EngineFailureDto>,
) -> AnalysisJobEventDto {
    AnalysisJobEventDto {
        run_id: started.run_id.clone(),
        job_id: started.job_id.clone(),
        lane: started.lane,
        mode: started.mode,
        generation: started.generation,
        node_path,
        outcome,
        completed,
        expected,
        remaining,
        frame,
        failure,
        current_game: None,
    }
}

fn target_query_id(job_id: &str) -> String {
    format!("{job_id}:{}", Uuid::new_v4())
}

fn bound_work_item_query(
    item: &WholeGameWorkItem,
    query_id: &str,
    job_id: &str,
) -> Result<String, EngineFailureDto> {
    let mut query = item.query.clone();
    query.id = query_id.to_string();
    query.report_during_search_every = Some(0.1);
    query.to_jsonl().map_err(|error| {
        failure(
            EngineOperationDto::Job,
            EngineFailureKind::Protocol,
            format!("failed to serialize whole-game query: {error}"),
            None,
            None,
            None,
        )
        .with_job_id(job_id)
    })
}

fn finish_whole_game_job(
    state: &mut ManagerState,
    started: &AnalysisJobStartedDto,
    outcome: AnalysisJobOutcomeDto,
    completed: Option<usize>,
    expected: Option<usize>,
    remaining: Option<usize>,
    failure: Option<EngineFailureDto>,
) {
    let owned = state
        .jobs
        .iter()
        .any(|job| job.job_id == started.job_id && job.run_id == started.run_id);
    if !owned {
        return;
    }
    let node_path = state
        .jobs
        .iter()
        .find(|job| job.job_id == started.job_id && job.run_id == started.run_id)
        .map(|job| {
            if job.work_items.is_empty() {
                return job.node_path.clone();
            }
            let index = job.current_index.min(job.work_items.len() - 1);
            job.work_items
                .get(index)
                .map(|item| item.node_path.clone())
                .unwrap_or_else(|| job.node_path.clone())
        })
        .unwrap_or_else(|| started.node_path.clone());
    finish_analysis_task_locked(state, &started.job_id, outcome, failure.as_ref());
    if state
        .analysis_task
        .as_ref()
        .is_some_and(|task| task.job_id == started.job_id && task.state == AnalysisTaskStateDto::Paused)
    {
        let job = state
            .jobs
            .iter_mut()
            .find(|job| job.job_id == started.job_id)
            .unwrap();
        job.terminal = true;
        job.cancel_deadline = None;
    } else {
        state.jobs.retain(|job| job.job_id != started.job_id);
    }
    publish_event(
        state,
        ForegroundEngineEventDto::Job {
            job: whole_game_job_event(
                started, outcome, node_path, completed, expected, remaining, None, failure,
            ),
        },
    );
}

fn mark_analysis_task_searching(state: &mut ManagerState, job_id: &str) {
    if let Some(task) = state
        .analysis_task
        .as_mut()
        .filter(|task| task.job_id == job_id && task.state == AnalysisTaskStateDto::Queued)
    {
        task.state = AnalysisTaskStateDto::Searching;
    }
}

fn active_task_conditions(task: &AnalysisTaskDto) -> Option<&AnalysisStageConditionsDto> {
    match task.stage {
        AnalysisTaskStageDto::Overview => task.overview_conditions.as_ref(),
        AnalysisTaskStageDto::SingleStage | AnalysisTaskStageDto::Deep => Some(&task.conditions),
    }
}

fn select_swing_deep_paths(task: &AnalysisTaskDto) -> Vec<NodePath> {
    let criteria = task.swing_criteria.as_ref().expect("swing task criteria");
    let mut selected = Vec::new();
    for comparison in &task.swing_comparisons {
        if !criteria.move_actors.admits(comparison.move_actor) {
            continue;
        }
        let Some(before) = task
            .overview_summaries
            .iter()
            .find(|summary| summary.node_path == comparison.before)
        else {
            continue;
        };
        let Some(after) = task
            .overview_summaries
            .iter()
            .find(|summary| summary.node_path == comparison.after)
        else {
            continue;
        };
        let player_winrate = |winrate_black: f32| match comparison.move_actor {
            PlayerColor::Black => winrate_black,
            PlayerColor::White => 1.0 - winrate_black,
        };
        let winrate_change =
            (player_winrate(after.frame.winrate_black) - player_winrate(before.frame.winrate_black)).abs()
                * 100.0;
        let mut qualifies = criteria.winrate_change_percentage_points.enabled
            && meets_swing_threshold(winrate_change, criteria.winrate_change_percentage_points.value);
        if criteria.score_change_points.enabled {
            let (before_score, after_score) = (
                before
                    .frame
                    .score_mean_black
                    .expect("validated overview root score"),
                after
                    .frame
                    .score_mean_black
                    .expect("validated overview root score"),
            );
            let player_score = |score_black: f32| match comparison.move_actor {
                PlayerColor::Black => score_black,
                PlayerColor::White => -score_black,
            };
            qualifies |= meets_swing_threshold(
                (player_score(after_score) - player_score(before_score)).abs(),
                criteria.score_change_points.value,
            );
        }
        if qualifies {
            for path in [&comparison.before, &comparison.after] {
                if !selected.contains(path) {
                    selected.push(path.clone());
                }
            }
        }
    }
    selected
}

fn meets_swing_threshold(observed: f32, threshold: f32) -> bool {
    observed >= threshold || (observed - threshold).abs() <= f32::EPSILON * threshold.abs().max(1.0) * 4.0
}

fn swing_zero_selection_reason(task: &AnalysisTaskDto) -> String {
    if task.swing_comparisons.is_empty() {
        return "Overview completed; no played-move predecessor comparisons were available in the requested scope."
            .into();
    }
    "Overview completed; no moves met the swing thresholds.".into()
}

fn record_analysis_task_completion(
    state: &mut ManagerState,
    job_id: &str,
    node_path: &NodePath,
    frame: &app_model::AnalysisFrameDto,
    ending_conditions: Vec<String>,
) {
    if let Some(task) = state.analysis_task.as_mut().filter(|task| {
        task.job_id == job_id
            && matches!(
                task.state,
                AnalysisTaskStateDto::Queued | AnalysisTaskStateDto::Searching
            )
    }) {
        let admitted = if task.stage == AnalysisTaskStageDto::Overview {
            task.requested.contains(node_path) || task.supporting.contains(node_path)
        } else {
            task.selected_for_deep
                .as_ref()
                .unwrap_or(&task.requested)
                .contains(node_path)
        };
        if !admitted {
            return;
        }
        if task.stage == AnalysisTaskStageDto::Overview {
            if !task.overview_completed.contains(node_path) {
                task.overview_completed.push(node_path.clone());
                task.overview_summaries.push(AnalysisTaskOverviewDto {
                    node_path: node_path.clone(),
                    frame: frame.clone(),
                });
            }
        } else if !task.completed.contains(node_path) {
            task.completed.push(node_path.clone());
        }
        task.ending_conditions = ending_conditions;
        task.reason = None;
    }
}

fn finish_analysis_task_locked(
    state: &mut ManagerState,
    job_id: &str,
    outcome: AnalysisJobOutcomeDto,
    failure: Option<&EngineFailureDto>,
) {
    let Some(task) = state.analysis_task.as_mut().filter(|task| task.job_id == job_id) else {
        return;
    };
    if matches!(
        task.state,
        AnalysisTaskStateDto::Cancelled | AnalysisTaskStateDto::Failed | AnalysisTaskStateDto::Invalidated
    ) {
        return;
    }
    match outcome {
        AnalysisJobOutcomeDto::Completed
            if task.completed.len() == task.selected_for_deep.as_ref().unwrap_or(&task.requested).len() =>
        {
            task.state = AnalysisTaskStateDto::Completed;
        }
        AnalysisJobOutcomeDto::Cancelled if task.state == AnalysisTaskStateDto::Pausing => {
            task.state = AnalysisTaskStateDto::Paused;
            task.reason = None;
        }
        AnalysisJobOutcomeDto::Cancelled => {
            task.state = AnalysisTaskStateDto::Cancelled;
            task.reason = Some("Analysis task was cancelled.".into());
        }
        AnalysisJobOutcomeDto::Completed => {
            task.state = AnalysisTaskStateDto::Failed;
            task.reason =
                Some("Analysis task ended before every requested position produced a result.".into());
        }
        AnalysisJobOutcomeDto::Failed | AnalysisJobOutcomeDto::Timeout => {
            task.state = AnalysisTaskStateDto::Failed;
            task.reason = Some(
                failure
                    .map(|failure| failure.message.clone())
                    .unwrap_or_else(|| "Analysis task failed.".into()),
            );
        }
        AnalysisJobOutcomeDto::Superseded => {
            task.state = AnalysisTaskStateDto::Invalidated;
            task.reason = Some("Analysis task was superseded.".into());
        }
        _ => {}
    }
}

fn cancel_analysis_task_locked(state: &mut ManagerState, job_id: &str) {
    if let Some(task) = state.analysis_task.as_mut().filter(|task| {
        task.job_id == job_id
            && matches!(
                task.state,
                AnalysisTaskStateDto::Queued
                    | AnalysisTaskStateDto::Searching
                    | AnalysisTaskStateDto::Pausing
                    | AnalysisTaskStateDto::Paused
            )
    }) {
        task.state = AnalysisTaskStateDto::Cancelled;
        task.reason = Some("Analysis task was cancelled.".into());
    }
}

fn invalidate_analysis_task_locked(state: &mut ManagerState, reason: &str) {
    if let Some(task) = state.analysis_task.as_mut().filter(|task| {
        matches!(
            task.state,
            AnalysisTaskStateDto::Queued
                | AnalysisTaskStateDto::Searching
                | AnalysisTaskStateDto::Pausing
                | AnalysisTaskStateDto::Paused
                | AnalysisTaskStateDto::Completed
                | AnalysisTaskStateDto::Failed
        )
    }) {
        task.state = AnalysisTaskStateDto::Invalidated;
        task.reason = Some(reason.into());
        state
            .jobs
            .retain(|job| !(job.job_id == task.job_id && job.terminal));
    }
}

fn fail_analysis_task_locked(state: &mut ManagerState, job_id: Option<&str>, reason: &str) {
    if let Some(task) = state.analysis_task.as_mut().filter(|task| {
        job_id.is_none_or(|job_id| task.job_id == job_id)
            && matches!(
                task.state,
                AnalysisTaskStateDto::Queued
                    | AnalysisTaskStateDto::Searching
                    | AnalysisTaskStateDto::Pausing
                    | AnalysisTaskStateDto::Paused
            )
    }) {
        task.state = AnalysisTaskStateDto::Failed;
        task.reason = Some(reason.into());
        state
            .jobs
            .retain(|job| !(job.job_id == task.job_id && job.terminal));
    }
}

fn started_from(job: &RegisteredJob) -> AnalysisJobStartedDto {
    AnalysisJobStartedDto {
        run_id: job.run_id.clone(),
        job_id: job.job_id.clone(),
        lane: job.lane,
        mode: job.mode,
        state: job.state,
        generation: job.generation,
        node_path: job.node_path.clone(),
    }
}

fn current_non_terminal_job(state: &ManagerState, lane: AnalysisJobLane) -> Option<AnalysisJobStartedDto> {
    state
        .jobs
        .iter()
        .rev()
        .find(|job| job.lane == lane && (!job.terminal || job.state.is_limited()))
        .map(started_from)
}

fn finish_selected_node_job(
    state: &mut ManagerState,
    started: &AnalysisJobStartedDto,
    outcome: Option<AnalysisJobOutcomeDto>,
    frame: Option<app_model::AnalysisFrameDto>,
    failure: Option<EngineFailureDto>,
) {
    let Some(job) = state
        .jobs
        .iter()
        .find(|job| job.job_id == started.job_id && job.run_id == started.run_id)
    else {
        return;
    };
    let admission = ContinuousAdmission::from_job(job);
    let mode = job.mode;
    let Some(outcome) = outcome else {
        return;
    };
    match (mode, outcome) {
        (
            AnalysisJobModeDto::Continuous,
            AnalysisJobOutcomeDto::TimeLimited | AnalysisJobOutcomeDto::VisitsLimited,
        ) => {
            state.continuous_limited = Some((
                admission,
                if outcome == AnalysisJobOutcomeDto::VisitsLimited {
                    ContinuousAnalysisPhaseDto::VisitsLimited
                } else {
                    ContinuousAnalysisPhaseDto::TimeLimited
                },
            ));
            if let Some(job) = state.jobs.iter_mut().find(|job| job.job_id == started.job_id) {
                job.terminal = true;
                job.state = if outcome == AnalysisJobOutcomeDto::VisitsLimited {
                    AnalysisJobStateDto::VisitsLimited
                } else {
                    AnalysisJobStateDto::TimeLimited
                };
            }
        }
        (AnalysisJobModeDto::Continuous, AnalysisJobOutcomeDto::Failed) => {
            state.continuous_error = true;
            state.jobs.retain(|job| job.job_id != started.job_id);
        }
        (AnalysisJobModeDto::Finite, AnalysisJobOutcomeDto::Failed | AnalysisJobOutcomeDto::Timeout) => {
            state.continuous_error = true;
            state.jobs.retain(|job| job.job_id != started.job_id);
        }
        (AnalysisJobModeDto::Finite, AnalysisJobOutcomeDto::Cancelled) => {
            state.continuous_paused = Some(admission);
            state.jobs.retain(|job| job.job_id != started.job_id);
        }
        _ => {
            state.jobs.retain(|job| job.job_id != started.job_id);
        }
    }
    publish_event(
        state,
        ForegroundEngineEventDto::Job {
            job: selected_node_job_event(started, outcome, frame, failure),
        },
    );
}
impl ContinuousAdmission {
    fn from_job(job: &RegisteredJob) -> Self {
        Self {
            run_id: job.run_id.clone(),
            generation: job.generation,
            node_path: job.node_path.clone(),
        }
    }

    fn matches_job(&self, job: &RegisteredJob) -> bool {
        self.run_id == job.run_id && self.generation == job.generation && self.node_path == job.node_path
    }

    fn matches_target(&self, run_id: Option<&str>, target: Option<&SelectedNodeJobRequest>) -> bool {
        run_id == Some(self.run_id.as_str())
            && target.is_some_and(|target| {
                same_position(
                    self.generation,
                    &self.node_path,
                    target.generation,
                    &target.node_path,
                )
            })
    }
}

fn same_position(
    left_generation: u64,
    left_path: &NodePath,
    right_generation: u64,
    right_path: &NodePath,
) -> bool {
    left_generation == right_generation && left_path == right_path
}

fn current_selected_job(state: &ManagerState) -> Option<AnalysisJobStartedDto> {
    state
        .jobs
        .iter()
        .rev()
        .find(|job| job.lane == AnalysisJobLane::SelectedNode && (!job.terminal || job.state.is_limited()))
        .map(started_from)
}

fn current_admitting_run(phase: &Phase) -> Option<EngineRunDto> {
    match phase {
        Phase::Ready(run) => Some(run.clone()),
        Phase::Switching { primary, .. } => Some(primary.clone()),
        _ => None,
    }
}

fn require_capability(run: &EngineRunDto, supported: bool, capability: &str) -> Result<(), EngineFailureDto> {
    if supported {
        Ok(())
    } else {
        Err(failure(
            EngineOperationDto::Job,
            EngineFailureKind::UnsupportedCapability,
            format!("current Ready Run does not admit {capability}"),
            Some(&run.run_id),
            Some(&run.profile_id),
            None,
        ))
    }
}

fn analysis_capabilities(run: &EngineRunDto) -> Result<&EngineAnalysisCapabilitiesDto, EngineFailureDto> {
    run.capability_snapshot
        .as_ref()
        .and_then(|snapshot| snapshot.analysis.as_ref())
        .ok_or_else(|| {
            failure(
                EngineOperationDto::Job,
                EngineFailureKind::UnsupportedCapability,
                "current Ready Run has no verified analysis capabilities".into(),
                Some(&run.run_id),
                Some(&run.profile_id),
                None,
            )
        })
}

fn validate_query_capabilities(
    run: &EngineRunDto,
    query: &AnalysisQuery,
    check_visits: bool,
) -> Result<(), EngineFailureDto> {
    let capabilities = analysis_capabilities(run)?;
    if query.include_ownership == Some(true) {
        require_capability(run, capabilities.ownership, "ownership analysis")?;
    }
    if query.include_policy == Some(true) && run.adapter_kind != EngineBackend::KataGoGtp {
        require_capability(run, capabilities.policy, "policy analysis")?;
    }
    if check_visits
        && (query.max_visits.is_some()
            || query.override_settings.as_ref().is_some_and(|settings| {
                settings.max_visits != katago_protocol::UNBOUNDED_MAX_VISITS
                    || settings.max_playouts != katago_protocol::UNBOUNDED_MAX_VISITS
            }))
    {
        require_capability(run, capabilities.visits_limit, "visits limits")?;
    }
    Ok(())
}

fn validate_whole_game_capabilities(
    run: &EngineRunDto,
    swing_criteria: Option<&AnalysisSwingCriteriaDto>,
) -> Result<(), EngineFailureDto> {
    let capabilities = analysis_capabilities(run)?;
    require_capability(run, capabilities.whole_game_analysis, "whole-game analysis")?;
    if let Some(criteria) = swing_criteria {
        if criteria.score_change_points.enabled {
            require_capability(run, capabilities.root_score, "root-score comparisons")?;
        }
        if criteria.winrate_change_percentage_points.enabled {
            require_capability(run, capabilities.winrate, "winrate comparisons")?;
        }
    }
    Ok(())
}

fn validate_task_conditions_capabilities(
    run: &EngineRunDto,
    conditions: &AnalysisStageConditionsDto,
) -> Result<(), EngineFailureDto> {
    let capabilities = analysis_capabilities(run)?;
    if conditions.total_visits.enabled || conditions.leading_candidate_visits.enabled {
        require_capability(run, capabilities.visits_limit, "visits limits")?;
    }
    if conditions.leading_candidate_visits.enabled {
        require_capability(run, capabilities.candidates, "candidate visits")?;
    }
    Ok(())
}

fn validate_continuous_run_capabilities(
    state: &ManagerState,
    run: &EngineRunDto,
) -> Result<(), EngineFailureDto> {
    let capabilities = analysis_capabilities(run)?;
    require_capability(run, capabilities.continuous_analysis, "continuous analysis")?;
    if state.continuous_budget.continuous_visits_limit_enabled {
        require_capability(run, capabilities.visits_limit, "visits limits")?;
    }
    if let Some(target) = state.continuous_target.as_ref() {
        validate_query_capabilities(run, &target.query, false)?;
    }
    Ok(())
}

fn validate_continuous_capabilities(state: &ManagerState) -> Result<(), EngineFailureDto> {
    match_reservation::require_unreserved(state)?;
    let run = current_admitting_run(&state.phase).ok_or_else(|| {
        continuous_invalid_state(
            state,
            "continuous analysis requires a Ready Foreground Engine Run",
        )
    })?;
    validate_continuous_run_capabilities(state, &run)
}

fn validate_selected_admission(
    state: &ManagerState,
    request: &SelectedNodeJobRequest,
) -> Result<EngineRunDto, EngineFailureDto> {
    match_reservation::require_unreserved(state)?;
    game_move::require_idle_move(state)?;
    let run_id = request.run_id.as_str();
    if state.continuous_departing {
        return Err(continuous_invalid_state(
            state,
            "document departure is in progress",
        ));
    }
    if state.continuous_target.as_ref().is_some_and(|target| {
        !same_position(
            request.generation,
            &request.node_path,
            target.generation,
            &target.node_path,
        )
    }) {
        return Err(continuous_invalid_state(
            state,
            "selected-node position changed during admission",
        ));
    }
    let run = admitting_run(&state.phase, run_id).ok_or_else(|| {
        failure(
            EngineOperationDto::Job,
            EngineFailureKind::InvalidState,
            "selected-node analysis requires the current Ready Foreground Engine Run".into(),
            Some(run_id),
            None,
            None,
        )
    })?;
    if run.adapter_kind == EngineBackend::KataGoGtp {
        if request.mode == AnalysisJobModeDto::Finite && request.query.max_visits.is_none_or(|visits| visits == 0) {
            return Err(failure(EngineOperationDto::Job, EngineFailureKind::UnsupportedCapability,
                "finite KataGo GTP analysis requires positive max visits".into(), Some(run_id), Some(&run.profile_id), None));
        }
        let exact = request.exact_position.as_ref().map_err(|message| failure(
            EngineOperationDto::Job, EngineFailureKind::UnsupportedCapability, message.clone(),
            Some(run_id), Some(&run.profile_id), None,
        ))?;
        ordinary_rules::admit(&run, exact).map_err(|message| failure(
            EngineOperationDto::Job, EngineFailureKind::UnsupportedCapability, message,
            Some(run_id), Some(&run.profile_id), None,
        ))?;
    }
    let capabilities = analysis_capabilities(&run)?;
    let continuous = request.mode == AnalysisJobModeDto::Continuous;
    require_capability(
        &run,
        if continuous {
            capabilities.continuous_analysis
        } else {
            capabilities.selected_node_analysis
        },
        if continuous {
            "continuous analysis"
        } else {
            "selected-node analysis"
        },
    )?;
    validate_query_capabilities(&run, &request.query, !continuous)?;
    if continuous && state.continuous_budget.continuous_visits_limit_enabled {
        require_capability(&run, capabilities.visits_limit, "visits limits")?;
    }
    Ok(run)
}

fn clear_weak_holds_for_new_position(state: &mut ManagerState) {
    state.continuous_limited = None;
    state.continuous_paused = None;
    state
        .jobs
        .retain(|job| !(job.lane == AnalysisJobLane::SelectedNode && job.terminal));
}

fn clear_resumable_holds(state: &mut ManagerState) {
    state.continuous_limited = None;
    state.continuous_paused = None;
    state.continuous_error = false;
    state.continuous_safety_hold = false;
    state
        .jobs
        .retain(|job| !(job.lane == AnalysisJobLane::SelectedNode && job.terminal));
}

fn clear_holds_for_new_run(state: &mut ManagerState) {
    state.continuous_limited = None;
    state.continuous_paused = None;
    state.continuous_error = false;
    state
        .jobs
        .retain(|job| !(job.lane == AnalysisJobLane::SelectedNode && job.terminal));
}

fn continuous_invalid_state(state: &ManagerState, message: &str) -> EngineFailureDto {
    failure(
        EngineOperationDto::Job,
        EngineFailureKind::InvalidState,
        message.into(),
        current_run_id(&state.phase).as_deref(),
        None,
        None,
    )
}

fn continuous_empty_board(state: &ManagerState) -> bool {
    state.continuous_budget.continuous_stop_on_empty_board
        && state
            .continuous_target
            .as_ref()
            .is_some_and(|target| target.position_empty)
}

fn continuous_snapshot(state: &ManagerState) -> ContinuousAnalysisSnapshotDto {
    let enabled = state.continuous_intent;
    let phase = if state.continuous_departing {
        ContinuousAnalysisPhaseDto::Departing
    } else if matches!(state.phase, Phase::Error { .. }) || state.continuous_error {
        ContinuousAnalysisPhaseDto::Error
    } else if state.game_move.is_some() {
        ContinuousAnalysisPhaseDto::Waiting
    } else if current_selected_job(state).is_some_and(|job| job.state == AnalysisJobStateDto::Stopping) {
        ContinuousAnalysisPhaseDto::Stopping
    } else if current_selected_job(state)
        .is_some_and(|job| job.mode == AnalysisJobModeDto::Finite && !job.state.is_limited())
    {
        ContinuousAnalysisPhaseDto::Finite
    } else if enabled.is_none() {
        ContinuousAnalysisPhaseDto::Loading
    } else if enabled == Some(false) {
        ContinuousAnalysisPhaseDto::Off
    } else if matches!(state.phase, Phase::Stopping(_)) {
        ContinuousAnalysisPhaseDto::Stopping
    } else if state.continuous_safety_hold {
        ContinuousAnalysisPhaseDto::SafetyHold
    } else if let Some(job) =
        current_selected_job(state).filter(|job| job.mode == AnalysisJobModeDto::Continuous)
    {
        match job.state {
            AnalysisJobStateDto::Queued => ContinuousAnalysisPhaseDto::Queued,
            AnalysisJobStateDto::Searching => ContinuousAnalysisPhaseDto::Searching,
            AnalysisJobStateDto::Stopping => ContinuousAnalysisPhaseDto::Stopping,
            AnalysisJobStateDto::TimeLimited => ContinuousAnalysisPhaseDto::TimeLimited,
            AnalysisJobStateDto::VisitsLimited => ContinuousAnalysisPhaseDto::VisitsLimited,
        }
    } else if let Some((_, phase)) = state.continuous_limited.as_ref() {
        *phase
    } else if state.continuous_paused.is_some() {
        ContinuousAnalysisPhaseDto::Paused
    } else if continuous_empty_board(state) {
        ContinuousAnalysisPhaseDto::EmptyBoard
    } else {
        match &state.phase {
            Phase::Ready(run) | Phase::Switching { primary: run, .. } => {
                if validate_continuous_run_capabilities(state, run).is_err() {
                    ContinuousAnalysisPhaseDto::Unavailable
                } else {
                    ContinuousAnalysisPhaseDto::Waiting
                }
            }
            Phase::NoEngine { failure: Some(_) } => ContinuousAnalysisPhaseDto::Unavailable,
            Phase::Error { .. } => ContinuousAnalysisPhaseDto::Error,
            _ => ContinuousAnalysisPhaseDto::Waiting,
        }
    };
    ContinuousAnalysisSnapshotDto { enabled, phase }
}

fn admitting_run(phase: &Phase, run_id: &str) -> Option<EngineRunDto> {
    match phase {
        Phase::Ready(run) if run.run_id == run_id => Some(run.clone()),
        Phase::Switching { primary, .. } if primary.run_id == run_id => Some(primary.clone()),
        _ => None,
    }
}

fn owned_live<'a>(state: &'a ManagerState, run_id: &str) -> Option<&'a LiveEngine> {
    state
        .live
        .iter()
        .chain(state.candidate.iter())
        .chain(state.match_residents.iter())
        .chain(state.preloads.iter().filter_map(|slot| slot.live.as_ref()))
        .find(|live| live.run_id == run_id)
}

fn owned_live_mut<'a>(state: &'a mut ManagerState, run_id: &str) -> Option<&'a mut LiveEngine> {
    state
        .live
        .iter_mut()
        .chain(state.candidate.iter_mut())
        .chain(state.match_residents.iter_mut())
        .chain(state.preloads.iter_mut().filter_map(|slot| slot.live.as_mut()))
        .find(|live| live.run_id == run_id)
}

fn take_owned_live(state: &mut ManagerState, run_id: &str) -> Option<LiveEngine> {
    state.gtp_dispatch.retire_run(run_id);
    if state.live.as_ref().is_some_and(|live| live.run_id == run_id) {
        state.live.take()
    } else if state.candidate.as_ref().is_some_and(|live| live.run_id == run_id) {
        state.candidate.take()
    } else {
        state
            .match_residents
            .iter()
            .position(|live| live.run_id == run_id)
            .map(|index| state.match_residents.remove(index))
    }
}

fn ready_move_run(state: &ManagerState, run_id: &str) -> Option<EngineRunDto> {
    if let Some(reservation) = state.match_reservation.as_ref() {
        if reservation.committed && !reservation.sealed {
            return reservation
                .runs
                .iter()
                .find(|run| run.run_id == run_id)
                .filter(|_| owned_live(state, run_id).is_some())
                .cloned();
        }
        return None;
    }
    match &state.phase {
        Phase::Ready(run) if run.run_id == run_id => Some(run.clone()),
        _ => None,
    }
}

fn engine_stdin(state: &ManagerState, run_id: &str) -> Option<Arc<Mutex<Option<ChildStdin>>>> {
    owned_live(state, run_id).map(|live| live.stdin.clone())
}

fn close_live_stdin(live: &LiveEngine) {
    if let Ok(mut guard) = live.stdin.lock() {
        drop(guard.take());
    }
}

fn write_terminate_to_live(live: &LiveEngine, job_id: &str) {
    if let Ok(mut guard) = live.stdin.lock() {
        if let Some(stdin) = guard.as_mut() {
            let payload = katago_protocol::terminate_action_jsonl(&Uuid::new_v4().to_string(), job_id);
            let _ = write_jsonl(stdin, &payload);
        }
    }
}

fn terminate_process(live: &mut LiveEngine, deadline: Instant) -> io::Result<()> {
    if let Some(status) = live.child.try_wait()? {
        live.capture.exited(status.code());
        return Ok(());
    }
    live.child.kill()?;
    if Instant::now() >= deadline {
        return Err(io::Error::new(
            io::ErrorKind::TimedOut,
            "engine cleanup observation budget exhausted",
        ));
    }
    loop {
        if let Some(status) = live.child.try_wait()? {
            live.capture.exited(status.code());
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "engine process did not exit after kill",
            ));
        }
        thread::sleep(Duration::from_millis(10));
    }
}

fn finish_failed_job(state: &mut ManagerState, started: &AnalysisJobStartedDto, published: EngineFailureDto) {
    if started.lane == AnalysisJobLane::SelectedNode {
        finish_selected_node_job(
            state,
            started,
            Some(AnalysisJobOutcomeDto::Failed),
            None,
            Some(published),
        );
    } else {
        let counts = state
            .jobs
            .iter()
            .find(|job| job.job_id == started.job_id)
            .map(whole_game_progress_counts)
            .unwrap_or((None, None, None));
        finish_whole_game_job(
            state,
            started,
            AnalysisJobOutcomeDto::Failed,
            counts.0,
            counts.1,
            counts.2,
            Some(published),
        );
    }
}

fn board_dimension_capability_refusal(error: &ProtocolError) -> Option<&str> {
    let ProtocolError::EngineField { message, field } = error else {
        return None;
    };
    matches!(field.as_str(), "boardXSize" | "boardYSize").then_some(message.as_str())
}

fn extract_response_id(line: &str) -> Option<String> {
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(line) {
        return value.get("id").and_then(|id| id.as_str()).map(str::to_string);
    }
    let marker = "\"id\":\"";
    let start = line.find(marker)? + marker.len();
    let rest = &line[start..];
    let end = rest.find('"')?;
    let id = &rest[..end];
    if id.is_empty() {
        None
    } else {
        Some(id.to_string())
    }
}

fn current_run_id(phase: &Phase) -> Option<String> {
    match phase {
        Phase::NoEngine { .. } => None,
        Phase::Starting(run) | Phase::Ready(run) | Phase::Stopping(run) => Some(run.run_id.clone()),
        Phase::Switching { primary, .. } => Some(primary.run_id.clone()),
        Phase::Error { run, .. } => Some(run.run_id.clone()),
    }
}

fn starting_run(saved: &SavedEngineProfile) -> EngineRunDto {
    EngineRunDto {
        run_id: Uuid::new_v4().to_string(),
        profile_id: saved.profile_id.clone(),
        adapter_kind: saved.profile.adapter_kind(),
        profile_snapshot: saved.profile.clone(),
        capability_snapshot: None,
        qualified_resource: None,
    }
}

fn empty_whole_game_failure(run_id: &str) -> EngineFailureDto {
    failure(
        EngineOperationDto::Job,
        EngineFailureKind::InvalidState,
        "whole-game analysis requires a captured first-child mainline worklist".into(),
        Some(run_id),
        None,
        None,
    )
}

fn invalid_task_conditions(run_id: &str, message: String) -> EngineFailureDto {
    failure(
        EngineOperationDto::Job,
        EngineFailureKind::InvalidState,
        message,
        Some(run_id),
        None,
        None,
    )
}

fn single_stage_conditions(total_visits: u32) -> AnalysisStageConditionsDto {
    AnalysisStageConditionsDto {
        time_seconds: AnalysisTaskLimitDto {
            enabled: false,
            value: 10,
        },
        total_visits: AnalysisTaskLimitDto {
            enabled: true,
            value: total_visits,
        },
        leading_candidate_visits: AnalysisTaskLimitDto {
            enabled: false,
            value: 500,
        },
    }
}

fn identify_failed_executable(run: &EngineRunDto, failure: &mut EngineFailureDto) {
    let path = std::path::Path::new(&run.profile_snapshot.program);
    let name = path.file_name().unwrap_or_default().to_string_lossy();
    failure.message = format!(
        "executable {}: {}",
        crate::diagnostics::DiagnosticSanitizer::default().sanitize(&name),
        failure.message
    );
}

fn attach_startup_failure(failure: &mut EngineFailureDto, output: &Arc<Mutex<StartupOutput>>, capture: &AttemptCapture) {
    let output = output.lock().unwrap_or_else(|e| e.into_inner());
    if matches!(
        failure.kind,
        EngineFailureKind::Start
            | EngineFailureKind::Readiness
            | EngineFailureKind::ProcessExit
            | EngineFailureKind::NonzeroExit
    ) {
        if let Some(kind) = output.failure_kind() {
            failure.kind = kind;
        }
    }
    let summary = output.summary(capture);
    if !summary.is_empty() {
        failure.diagnostic_summary = Some(summary);
    }
    failure.message = capture.sanitize(&failure.message);
}

fn failure(
    operation: EngineOperationDto,
    kind: EngineFailureKind,
    message: String,
    run_id: Option<&str>,
    profile_id: Option<&str>,
    diagnostic_summary: Option<String>,
) -> EngineFailureDto {
    let mut sanitizer = crate::diagnostics::DiagnosticSanitizer::default();
    EngineFailureDto {
        operation,
        run_id: run_id.map(str::to_string),
        switch_id: None,
        job_id: None,
        profile_id: profile_id.map(str::to_string),
        kind,
        message: sanitizer.sanitize(&message),
        diagnostic_summary: diagnostic_summary.map(|text| sanitizer.sanitize(&text)),
    }
}

#[cfg(test)]
mod swing_selection_tests {
    use super::*;
    use app_model::{AnalysisMoveActorFilterDto, AnalysisSwingThresholdDto};

    fn path(indices: &[u32]) -> NodePath {
        NodePath {
            indices: indices.to_vec(),
        }
    }

    fn summary(
        indices: &[u32],
        winrate_black: f32,
        score_mean_black: Option<f32>,
    ) -> AnalysisTaskOverviewDto {
        AnalysisTaskOverviewDto {
            node_path: path(indices),
            frame: app_model::AnalysisFrameDto {
                job_id: Uuid::nil(),
                game_id: None,
                node_id: None,
                turn: indices.len() as u32,
                visits: 32,
                winrate_black,
                score_mean_black,
                score_stdev: None,
                candidates: Vec::new(),
                ownership: None,
                policy: None,
            },
        }
    }

    fn task(criteria: AnalysisSwingCriteriaDto) -> AnalysisTaskDto {
        let requested = vec![path(&[0]), path(&[0, 0])];
        AnalysisTaskDto {
            task_id: "task".into(),
            run_id: "run".into(),
            job_id: "job".into(),
            generation: 1,
            scope: AnalysisScopeDto {
                mode: AnalysisScopeModeDto::FirstChildMainline,
                current_node: NodePath::default(),
                branch_choices: Vec::new(),
                interval: None,
                to_play: None,
            },
            strategy: AnalysisTaskStrategyDto::SwingSelectedTwoStage,
            stage: AnalysisTaskStageDto::Overview,
            conditions: AnalysisStageConditionsDto::default(),
            overview_conditions: Some(AnalysisStageConditionsDto::default()),
            requested: requested.clone(),
            supporting: vec![NodePath::default()],
            swing_comparisons: vec![
                AnalysisSwingComparisonDto {
                    before: NodePath::default(),
                    after: requested[0].clone(),
                    move_actor: PlayerColor::Black,
                },
                AnalysisSwingComparisonDto {
                    before: requested[0].clone(),
                    after: requested[1].clone(),
                    move_actor: PlayerColor::White,
                },
            ],
            swing_criteria: Some(criteria),
            selected_for_deep: None,
            overview_completed: vec![NodePath::default(), requested[0].clone(), requested[1].clone()],
            completed: Vec::new(),
            overview_summaries: vec![
                summary(&[], 0.5, Some(0.0)),
                summary(&[0], 0.4, Some(5.0)),
                summary(&[0, 0], 0.5, Some(4.0)),
            ],
            state: AnalysisTaskStateDto::Searching,
            reason: None,
            ending_conditions: Vec::new(),
        }
    }

    #[test]
    fn swing_predicates_cover_equality_both_directions_score_only_and_or() {
        let criteria = AnalysisSwingCriteriaDto {
            move_actors: AnalysisMoveActorFilterDto::Both,
            winrate_change_percentage_points: AnalysisSwingThresholdDto {
                enabled: true,
                value: 10.0,
            },
            score_change_points: AnalysisSwingThresholdDto {
                enabled: false,
                value: 5.0,
            },
        };
        let winrate_only = select_swing_deep_paths(&task(criteria.clone()));
        assert_eq!(winrate_only, vec![path(&[]), path(&[0]), path(&[0, 0])]);

        let mut score_only = criteria.clone();
        score_only.winrate_change_percentage_points.enabled = false;
        score_only.score_change_points.enabled = true;
        let selected = select_swing_deep_paths(&task(score_only));
        assert_eq!(selected, vec![path(&[]), path(&[0])]);

        let mut either = criteria;
        either.winrate_change_percentage_points.value = 100.0;
        either.score_change_points.enabled = true;
        let selected = select_swing_deep_paths(&task(either));
        assert_eq!(selected, vec![path(&[]), path(&[0])]);

        let mut unformable = task(AnalysisSwingCriteriaDto::default());
        unformable.swing_comparisons.clear();
        assert!(swing_zero_selection_reason(&unformable)
            .contains("no played-move predecessor comparisons were available"));
    }
}
