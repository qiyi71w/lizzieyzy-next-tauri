#![allow(clippy::result_large_err)]
use crate::catalog::{EngineProfileCatalog, SavedEngineProfile};
use crate::{
    build_command_spec, build_process_command, check_assets, kill_timed_out_child, spawn_stderr_reader,
    spawn_stdout_lines_reader, write_jsonl, AnalysisCancelToken,
};
use app_model::{
    AnalysisJobLaneDto, AnalysisJobModeDto, AnalysisJobOutcomeDto, AnalysisJobStartedDto,
    AnalysisJobStateDto, AnalysisScopeDto, AnalysisScopeModeDto, AnalysisStageConditionsDto, AnalysisTaskDto,
    AnalysisTaskLimitDto, AnalysisTaskOverviewDto, AnalysisTaskStageDto, AnalysisTaskStateDto,
    AnalysisTaskStrategyDto, ContinuousAnalysisBudgetDto, ContinuousAnalysisPhaseDto,
    ContinuousAnalysisSnapshotDto, EngineBackend, EngineCapabilitySnapshotDto, EngineFailureDto,
    EngineFailureKind, EngineOperationDto, EngineRunDto, ForegroundEngineEventDto,
    ForegroundEngineLifecycleDto, ForegroundEngineSnapshotDto, NodePath,
};
use katago_protocol::{normalize_response, parse_response_line, AnalysisQuery};
use std::io;
use std::process::{Child, ChildStdin};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use uuid::Uuid;

pub use app_model::AnalysisJobEventDto;
pub use app_model::AnalysisJobLaneDto as AnalysisJobLane;

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
}

impl Default for ForegroundEngineConfig {
    fn default() -> Self {
        Self {
            readiness_timeout: Duration::from_secs(30),
            stop_drain_timeout: Duration::from_secs(2),
            job_timeout: Duration::from_secs(60),
            admit_whole_game_analysis: true,
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
    pub board_size: u8,
    pub position_empty: bool,
}

#[derive(Clone)]
pub struct WholeGameWorkItem {
    pub node_path: NodePath,
    pub query: AnalysisQuery,
    pub board_size: u8,
    pub move_number: u32,
}

struct SelectedSubmission {
    started: AnalysisJobStartedDto,
    jsonl: String,
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
    board_size: u8,
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
    #[allow(dead_code)]
    stderr_rx: Receiver<io::Result<String>>,
    run_id: String,
    process_id: u32,
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
    operation: u64,
    operation_kind: EngineOperationDto,
    live: Option<LiveEngine>,
    candidate: Option<LiveEngine>,
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
}

struct Inner {
    catalog: Arc<dyn EngineProfileCatalog>,
    config: ForegroundEngineConfig,
    state: Mutex<ManagerState>,
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
                state: Mutex::new(ManagerState {
                    revision: 0,
                    phase: Phase::NoEngine { failure: None },
                    operation: 0,
                    operation_kind: EngineOperationDto::Start,
                    live: None,
                    candidate: None,
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
                }),
            }),
        }
    }

    pub fn snapshot(&self) -> ForegroundEngineSnapshotDto {
        snapshot_from(&self.lock())
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
        budget.validate()?;
        {
            let mut state = self.lock();
            let budget_changed = state.continuous_budget != budget;
            if state.continuous_intent == Some(enabled) && !budget_changed {
                return Ok(());
            }
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
        }
        self.reconcile_continuous();
        Ok(())
    }

    pub fn follow_continuous_position(&self, mut request: SelectedNodeJobRequest) {
        request.mode = AnalysisJobModeDto::Continuous;
        let cancel = {
            let mut state = self.lock();
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

    pub fn continuous_primary_action(&self) -> Result<ContinuousPrimaryAction, EngineFailureDto> {
        let state = self.lock();
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
            if state.continuous_departing
                || state.finite_admission_pending
                || current_selected_job(&state).is_some_and(|job| job.state == AnalysisJobStateDto::Stopping)
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
            if state.continuous_intent != Some(true)
                || state.continuous_departing
                || !current_admitting_run(&state.phase).is_some_and(|run| {
                    run.capability_snapshot
                        .as_ref()
                        .is_some_and(|capability| capability.selected_node_analysis)
                })
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
        if !state.continuous_departing {
            state.continuous_departing = true;
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
            state.continuous_departing = false;
            if committed_replacement {
                clear_resumable_holds(&mut state);
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
        if saved.profile.backend != EngineBackend::KataGoAnalysis {
            let published = failure(
                operation_kind,
                EngineFailureKind::UnsupportedCapability,
                "R3 Foreground Engine Run only proves KataGoAnalysis".into(),
                None,
                Some(saved.profile_id.as_str()),
                None,
            );
            self.inner.record_no_engine_failure(published.clone());
            return Err(published);
        }

        let (operation, run) = {
            let mut state = self.lock();
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
        if saved.profile.backend != EngineBackend::KataGoAnalysis {
            return Err(failure(
                EngineOperationDto::Switch,
                EngineFailureKind::UnsupportedCapability,
                "R3 Foreground Engine Run only proves KataGoAnalysis".into(),
                None,
                Some(saved.profile_id.as_str()),
                None,
            ));
        }
        let (operation, candidate, switch_id) = {
            let mut state = self.lock();
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
        let admitted =
            admitting_run(&state.phase, run_id).is_some_and(|run| run.capability_snapshot.is_some());
        if !admitted {
            return Err(failure(
                EngineOperationDto::Job,
                EngineFailureKind::InvalidState,
                "Analysis Job admission requires a Ready Foreground Engine Run".into(),
                Some(run_id),
                None,
                None,
            ));
        }
        let job_id = Uuid::new_v4().to_string();
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
            board_size: 19,
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
        validate_selected_admission(state, &request)?;
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
        let bound_query = request.query.to_jsonl().map_err(|error| {
            failure(
                EngineOperationDto::Job,
                EngineFailureKind::Protocol,
                format!("failed to serialize selected-node query: {error}"),
                Some(started.run_id.as_str()),
                None,
                None,
            )
            .with_job_id(&started.job_id)
        })?;
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
            board_size: request.board_size,
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
            jsonl: bound_query,
        })
    }

    fn submit_registered_selected(
        &self,
        submission: SelectedSubmission,
    ) -> Result<AnalysisJobStartedDto, EngineFailureDto> {
        if let Err(error) = self.write_live_jsonl(&submission.started.run_id, &submission.jsonl) {
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
        )
    }

    fn start_analysis_task_with_strategy(
        &self,
        mut request: WholeGameJobRequest,
        scope: AnalysisScopeDto,
        strategy: AnalysisTaskStrategyDto,
        overview_conditions: Option<AnalysisStageConditionsDto>,
        conditions: AnalysisStageConditionsDto,
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
            let supported = run
                .capability_snapshot
                .as_ref()
                .is_some_and(|snapshot| snapshot.whole_game_analysis);
            if !supported {
                return Err(failure(
                    EngineOperationDto::Job,
                    EngineFailureKind::UnsupportedCapability,
                    "current Ready Run does not admit whole-game analysis".into(),
                    Some(request.run_id.as_str()),
                    Some(run.profile_id.as_str()),
                    None,
                ));
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
            let first_board_size = request.work_items[0].board_size;
            let bound_query = bound_work_item_query(&request.work_items[0], &query_id, &job_id)?;
            let started = AnalysisJobStartedDto {
                run_id: request.run_id.clone(),
                job_id: job_id.clone(),
                lane: AnalysisJobLaneDto::WholeGame,
                mode: AnalysisJobModeDto::Finite,
                state: AnalysisJobStateDto::Queued,
                generation: request.generation,
                node_path: first_node_path.clone(),
            };
            let requested = request
                .work_items
                .iter()
                .map(|item| item.node_path.clone())
                .collect();
            let task = AnalysisTaskDto {
                task_id,
                run_id: request.run_id.clone(),
                job_id: job_id.clone(),
                generation: request.generation,
                scope,
                strategy,
                stage: if strategy == AnalysisTaskStrategyDto::AllPositionsTwoStage {
                    AnalysisTaskStageDto::Overview
                } else {
                    AnalysisTaskStageDto::SingleStage
                },
                conditions,
                overview_conditions,
                requested,
                overview_completed: Vec::new(),
                completed: Vec::new(),
                overview_summaries: Vec::new(),
                ending_conditions: Vec::new(),
                state: AnalysisTaskStateDto::Queued,
                reason: None,
            };
            let expected = request.work_items.len();
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
                board_size: first_board_size,
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
            let old_job_id = task.job_id.clone();
            let index = state
                .jobs
                .iter()
                .position(|job| job.job_id == old_job_id && job.run_id == run_id && job.terminal)
                .ok_or_else(|| continuous_invalid_state(&state, "Paused task cleanup has not finished."))?;
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
        let manager = self.clone();
        let run_id = run_id.to_string();
        let job_id = job_id.to_string();
        thread::spawn(move || {
            manager.write_terminate_after_submission(&run_id, &job_id, &query_id);
        });
        Ok(())
    }

    pub fn set_capability_snapshot_for_tests(&self, snapshot: EngineCapabilitySnapshotDto) {
        let mut state = self.lock();
        if let Phase::Ready(run) = &mut state.phase {
            run.capability_snapshot = Some(snapshot);
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
            if state.continuous_departing
                || state.finite_admission_pending
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
                if !run
                    .capability_snapshot
                    .as_ref()
                    .is_some_and(|capability| capability.selected_node_analysis)
                {
                    return;
                }
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
        let blocked = match &state.phase {
            Phase::Starting(run) | Phase::Ready(run) | Phase::Stopping(run) => Some(run),
            Phase::Switching { primary, .. } => Some(primary),
            Phase::Error { run, .. } => Some(run),
            Phase::NoEngine { .. } => None,
        };
        if let Some(run) = blocked {
            if run.profile_id == profile_id {
                return Err(failure(
                    EngineOperationDto::DeleteProfile,
                    EngineFailureKind::ProfileInUse,
                    "an active Foreground Engine Run still holds this profile identity".into(),
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
        if let Err(published) = self.start_resident(operation, &run, false) {
            self.fail_attempt(operation, published);
        }
    }

    fn run_switch_candidate(self: Arc<Self>, operation: u64, run: EngineRunDto, switch_id: String) {
        if self.current_operation() != operation {
            return;
        }
        if let Err(published) = self.start_resident(operation, &run, true) {
            self.fail_switch_candidate(operation, &run, &switch_id, published);
        }
    }

    fn start_resident(
        self: &Arc<Self>,
        operation: u64,
        run: &EngineRunDto,
        as_candidate: bool,
    ) -> Result<(), EngineFailureDto> {
        let kind = self.lock().operation_kind;
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
        if self.current_operation() != operation {
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
        let mut stdin = child.stdin.take().ok_or_else(|| {
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
        let stdout_rx = spawn_stdout_lines_reader(stdout);
        let stderr_rx = spawn_stderr_reader(stderr);

        let probe_id = format!("lifecycle-readiness-{}", run.run_id);
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
        let query_jsonl = query.to_jsonl().map_err(|error| {
            failure(
                kind,
                EngineFailureKind::Protocol,
                format!("failed to serialize readiness probe: {error}"),
                Some(run.run_id.as_str()),
                Some(run.profile_id.as_str()),
                None,
            )
        })?;
        write_jsonl(&mut stdin, &query_jsonl).map_err(|error| {
            failure(
                kind,
                EngineFailureKind::Protocol,
                format!("failed to write readiness probe: {error}"),
                Some(run.run_id.as_str()),
                Some(run.profile_id.as_str()),
                None,
            )
        })?;

        {
            let mut state = self.lock();
            if state.operation != operation {
                drop(state);
                let _ = kill_timed_out_child(&mut child);
                return Ok(());
            }
            let process_id = child.id();
            let engine = LiveEngine {
                child,
                stdin: Arc::new(Mutex::new(Some(stdin))),
                stdout_rx: Some(stdout_rx),
                stderr_rx,
                run_id: run.run_id.clone(),
                process_id,
            };
            if as_candidate {
                state.candidate = Some(engine);
            } else {
                state.live = Some(engine);
            }
        }

        self.await_readiness(operation, run, &probe_id, as_candidate)?;
        if as_candidate {
            self.promote_candidate(operation, run);
        } else {
            self.admit_ready(operation, run);
        }
        Ok(())
    }
    fn reconcile_continuous(self: &Arc<Self>) {
        ForegroundEngineManager {
            inner: Arc::clone(self),
        }
        .reconcile_continuous();
    }

    fn await_readiness(
        &self,
        operation: u64,
        run: &EngineRunDto,
        probe_id: &str,
        as_candidate: bool,
    ) -> Result<(), EngineFailureDto> {
        let kind = self.lock().operation_kind;
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
                let live = if as_candidate {
                    state.candidate.as_ref()
                } else {
                    state.live.as_ref()
                };
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

    fn admit_ready(self: &Arc<Self>, operation: u64, run: &EngineRunDto) {
        let mut state = self.lock();
        if state.operation != operation {
            return;
        }
        if !matches!(&state.phase, Phase::Starting(current) if current.run_id == run.run_id) {
            return;
        }
        let mut ready = run.clone();
        ready.capability_snapshot = Some(EngineCapabilitySnapshotDto {
            adapter_kind: EngineBackend::KataGoAnalysis,
            selected_node_analysis: true,
            whole_game_analysis: self.config.admit_whole_game_analysis,
            protocol_cancel: true,
        });
        invalidate_analysis_task_locked(
            &mut state,
            "A new Foreground Engine Run is ready; start a new analysis task.",
        );
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
            thread::spawn(move || inner.pump_stdout(operation, run_id, stdout_rx));
        }
    }

    fn promote_candidate(self: &Arc<Self>, operation: u64, run: &EngineRunDto) {
        let retiring = {
            let mut state = self.lock();
            if state.operation != operation {
                return;
            }
            let Phase::Switching {
                primary,
                candidate,
                switch_id: _,
            } = &state.phase
            else {
                return;
            };
            if candidate.run_id != run.run_id {
                return;
            }
            let primary_run_id = primary.run_id.clone();
            let mut ready = run.clone();
            ready.capability_snapshot = Some(EngineCapabilitySnapshotDto {
                adapter_kind: EngineBackend::KataGoAnalysis,
                selected_node_analysis: true,
                whole_game_analysis: self.config.admit_whole_game_analysis,
                protocol_cancel: true,
            });
            invalidate_analysis_task_locked(
                &mut state,
                "The Foreground Engine Run switched; start a new analysis task.",
            );
            cancel_jobs_for_run(&mut state, &primary_run_id);
            let retiring = state.live.take();
            state.live = state.candidate.take();
            state.phase = Phase::Ready(ready);
            clear_holds_for_new_run(&mut state);
            publish_snapshot(&mut state);
            let process_id = state.live.as_ref().map(|live| live.process_id);
            let stdout_rx = state.live.as_mut().and_then(|live| live.stdout_rx.take());
            drop(state);
            if let Some(stdout_rx) = stdout_rx {
                let inner = self.clone();
                let run_id = run.run_id.clone();
                thread::spawn(move || inner.pump_stdout(operation, run_id, stdout_rx));
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
        let published = published.with_switch_id(switch_id);
        if let Some(mut live) = state.candidate.take() {
            close_live_stdin(&live);
            let _ = kill_timed_out_child(&mut live.child);
        }
        state.phase = Phase::Ready(primary);
        publish_snapshot(&mut state);
        publish_event(
            &mut state,
            ForegroundEngineEventDto::Failure { failure: published },
        );
    }

    fn fail_attempt(&self, operation: u64, published: EngineFailureDto) {
        let mut state = self.lock();
        if state.operation != operation {
            return;
        }
        if let Some(mut live) = state.live.take() {
            close_live_stdin(&live);
            let _ = kill_timed_out_child(&mut live.child);
        }
        if let Some(mut live) = state.candidate.take() {
            close_live_stdin(&live);
            let _ = kill_timed_out_child(&mut live.child);
        }
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
        state.phase = Phase::NoEngine { failure: None };
        publish_snapshot(&mut state);
    }

    fn watch_exit(self: Arc<Self>, operation: u64, run_id: String, process_id: u32) {
        loop {
            thread::sleep(Duration::from_millis(30));
            let status = {
                let mut state = self.lock();
                let Some(live) = state.live.as_mut() else {
                    return;
                };
                if live.run_id != run_id || live.process_id != process_id {
                    return;
                }
                match live.child.try_wait() {
                    Ok(Some(status)) => Some(status.code()),
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
        let Some(live) = state.live.as_ref() else {
            return;
        };
        if live.run_id != run_id || live.process_id != process_id {
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
        fail_analysis_task_locked(state, None, "The Foreground Engine Run exited unexpectedly.");
        cancel_jobs_for_current(state);
        if let Some(mut live) = state.live.take() {
            let _ = live.child.wait();
        }
        let published = failure(
            EngineOperationDto::UnexpectedExit,
            EngineFailureKind::NonzeroExit,
            format!("engine process exited unexpectedly; exit_code={exit_code:?}"),
            Some(run.run_id.as_str()),
            Some(run.profile_id.as_str()),
            None,
        );
        state.phase = Phase::Error {
            run,
            failure: published.clone(),
        };
        publish_snapshot(state);
        publish_event(state, ForegroundEngineEventDto::Failure { failure: published });
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
                state.live.as_ref().is_some_and(|live| live.run_id == run_id)
                    || state.candidate.as_ref().is_some_and(|live| live.run_id == run_id)
            };
            if !owned {
                return;
            }
            match stdout_rx.recv_timeout(Duration::from_millis(50)) {
                Ok(Ok(Some(line))) => {
                    if let Some(pending) = self.route_stdout_line(&run_id, line) {
                        self.advance_whole_game(pending);
                    }
                }
                Ok(Ok(None)) | Ok(Err(_)) | Err(mpsc::RecvTimeoutError::Disconnected) => {
                    thread::sleep(Duration::from_millis(50))
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
                let published = failure(
                    EngineOperationDto::Job,
                    EngineFailureKind::Protocol,
                    format!("analysis response was not parseable: {error}"),
                    Some(run_id),
                    None,
                    None,
                )
                .with_job_id(&started.job_id);
                if started.mode == AnalysisJobModeDto::Continuous
                    || state.jobs[index].disposition == JobDisposition::BudgetReached
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
                && (response.no_results
                    || response.move_infos.is_empty()
                    || !response.root_info.as_ref().is_some_and(|root| root.visits > 0)
                    || !response.has_valid_search_result(job.work_items[job.current_index].board_size));
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
        let has_data = has_result && !response.move_infos.is_empty();
        if started.mode == AnalysisJobModeDto::Continuous && has_result && response.is_during_search.is_none()
        {
            drop(state);
            self.fail_unresponsive_run(
                run_id,
                "continuous response omitted the required search-progress marker",
            );
            return None;
        }
        if started.mode == AnalysisJobModeDto::Continuous {
            if has_result && !response.has_valid_search_result(state.jobs[index].board_size) {
                drop(state);
                self.fail_unresponsive_run(
                    run_id,
                    "continuous response contained invalid analysis values or geometry",
                );
                return None;
            }
            if !during && !has_result {
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
        }
        if during && !has_data {
            return None;
        }
        let frame = has_data.then(|| {
            normalize_response(
                Uuid::parse_str(&started.job_id).expect("manager job UUID"),
                response,
                state.jobs[index].board_size,
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
                let has_search_data = !response.no_results
                    && !response.move_infos.is_empty()
                    && response.root_info.as_ref().is_some_and(|root| root.visits > 0)
                    && response.has_valid_search_result(item.board_size);
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
                let mut frame = normalize_response(job_uuid, response, item.board_size);
                frame.turn = item.move_number;
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
                            && task.strategy == AnalysisTaskStrategyDto::AllPositionsTwoStage
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

                    let deep_conditions = state
                        .analysis_task
                        .as_ref()
                        .expect("two-stage task")
                        .conditions
                        .clone();
                    let job = &mut state.jobs[job_index];
                    for work_item in &mut job.work_items {
                        work_item.query.task(&deep_conditions);
                    }
                    job.current_index = 0;
                    if let Some(task) = state.analysis_task.as_mut() {
                        task.stage = AnalysisTaskStageDto::Deep;
                        task.state = AnalysisTaskStateDto::Queued;
                        task.reason = None;
                        task.ending_conditions.clear();
                    }
                }

                let next_index = state.jobs[job_index].current_index;
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
        let mut state = self.lock();
        if let Phase::Error { failure, .. } = &state.phase {
            return failure.clone();
        }
        let mut published = failure(
            EngineOperationDto::Job,
            EngineFailureKind::Timeout,
            message.into(),
            Some(run_id),
            None,
            None,
        );
        let Some(run) = admitting_run(&state.phase, run_id) else {
            return published;
        };
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
        let ManagerState { live, candidate, .. } = &mut *state;
        for slot in [live, candidate] {
            if let Some(live) = slot.as_mut() {
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
        if !task.requested.contains(node_path) {
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
        AnalysisJobOutcomeDto::Completed if task.completed.len() == task.requested.len() => {
            task.state = AnalysisTaskStateDto::Completed;
            task.reason = None;
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

fn validate_selected_admission(
    state: &ManagerState,
    request: &SelectedNodeJobRequest,
) -> Result<EngineRunDto, EngineFailureDto> {
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
    if !run
        .capability_snapshot
        .as_ref()
        .is_some_and(|snapshot| snapshot.selected_node_analysis)
    {
        return Err(failure(
            EngineOperationDto::Job,
            EngineFailureKind::UnsupportedCapability,
            "current Foreground Engine Run does not advertise selected-node analysis".into(),
            Some(run_id),
            Some(&run.profile_id),
            None,
        ));
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
                if !run
                    .capability_snapshot
                    .as_ref()
                    .is_some_and(|capability| capability.selected_node_analysis)
                {
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

fn engine_stdin(state: &ManagerState, run_id: &str) -> Option<Arc<Mutex<Option<ChildStdin>>>> {
    if let Some(live) = state.live.as_ref() {
        if live.run_id == run_id {
            return Some(live.stdin.clone());
        }
    }
    if let Some(live) = state.candidate.as_ref() {
        if live.run_id == run_id {
            return Some(live.stdin.clone());
        }
    }
    None
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
    if live.child.try_wait()?.is_some() {
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
        if live.child.try_wait()?.is_some() {
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
        adapter_kind: saved.profile.backend,
        profile_snapshot: saved.profile.clone(),
        capability_snapshot: None,
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

fn failure(
    operation: EngineOperationDto,
    kind: EngineFailureKind,
    message: String,
    run_id: Option<&str>,
    profile_id: Option<&str>,
    diagnostic_summary: Option<String>,
) -> EngineFailureDto {
    EngineFailureDto {
        operation,
        run_id: run_id.map(str::to_string),
        switch_id: None,
        job_id: None,
        profile_id: profile_id.map(str::to_string),
        kind,
        message,
        diagnostic_summary,
    }
}
