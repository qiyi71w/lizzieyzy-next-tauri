#![allow(clippy::result_large_err)]
use crate::catalog::{EngineProfileCatalog, SavedEngineProfile};
use crate::{
    build_command_spec, build_process_command, check_assets, kill_timed_out_child, spawn_stderr_reader,
    spawn_stdout_lines_reader, write_jsonl, AnalysisCancelToken,
};
use app_model::{
    AnalysisJobLaneDto, AnalysisJobOutcomeDto, AnalysisJobStartedDto, EngineBackend,
    EngineCapabilitySnapshotDto, EngineFailureDto, EngineFailureKind, EngineOperationDto, EngineRunDto,
    ForegroundEngineEventDto, ForegroundEngineLifecycleDto, ForegroundEngineSnapshotDto, NodePath,
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
    Superseded,
    Cancelled,
}

pub struct SelectedNodeJobRequest {
    pub run_id: String,
    pub generation: u64,
    pub node_path: NodePath,
    pub query: AnalysisQuery,
    pub board_size: u8,
}

pub struct WholeGameJobRequest {
    pub run_id: String,
    pub generation: u64,
    pub node_path: NodePath,
    pub query_jsonl: String,
    pub expected_responses: usize,
}

struct RegisteredJob {
    job_id: String,
    run_id: String,
    lane: AnalysisJobLane,
    generation: u64,
    node_path: NodePath,
    board_size: u8,
    cancel: Arc<dyn AnalysisJobCancel>,
    disposition: JobDisposition,
    expected: Option<usize>,
    received_lines: Vec<String>,
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
    jobs: Vec<RegisteredJob>,
    subscribers: Vec<Sender<ForegroundEngineEventDto>>,
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
                    subscribers: Vec::new(),
                }),
            }),
        }
    }

    pub fn snapshot(&self) -> ForegroundEngineSnapshotDto {
        snapshot_from(&self.lock())
    }

    pub fn subscribe(&self) -> Receiver<ForegroundEngineEventDto> {
        let (tx, rx) = mpsc::channel();
        self.lock().subscribers.push(tx);
        rx
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
            inner.terminate_live(operation);
            inner.force_no_engine(operation);
        });
        Ok(())
    }

    pub fn teardown(&self) -> Result<(), EngineFailureDto> {
        let Some(operation) = self.begin_stop(EngineOperationDto::Teardown)? else {
            return Ok(());
        };
        self.inner.terminate_live(operation);
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
            cancel_jobs_for_current(&mut state);
            publish_snapshot(&mut state);
            (state.operation, profile_id)
        };
        let manager = self.clone();
        thread::spawn(move || {
            manager.inner.terminate_live(operation);
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
            run_id: run_id.to_string(),
            lane,
            generation: 0,
            node_path: NodePath { indices: Vec::new() },
            board_size: 19,
            cancel,
            disposition: JobDisposition::Running,
            expected: None,
            received_lines: Vec::new(),
            terminal: false,
        });
        Ok(job_id)
    }

    pub fn start_selected_node_job(
        &self,
        mut request: SelectedNodeJobRequest,
    ) -> Result<AnalysisJobStartedDto, EngineFailureDto> {
        let cancel = AnalysisCancelToken::new();
        let (started, bound_query) = {
            let mut state = self.lock();
            let run = match admitting_run(&state.phase, &request.run_id) {
                Some(run) => run,
                None => {
                    return Err(failure(
                        EngineOperationDto::Job,
                        EngineFailureKind::InvalidState,
                        "selected-node analysis requires the current Ready Foreground Engine Run".into(),
                        Some(request.run_id.as_str()),
                        None,
                        None,
                    ));
                }
            };
            let selected = run
                .capability_snapshot
                .as_ref()
                .map(|snapshot| snapshot.selected_node_analysis)
                .unwrap_or(false);
            if !selected {
                return Err(failure(
                    EngineOperationDto::Job,
                    EngineFailureKind::UnsupportedCapability,
                    "current Foreground Engine Run does not advertise selected-node analysis".into(),
                    Some(run.run_id.as_str()),
                    Some(run.profile_id.as_str()),
                    None,
                ));
            }
            supersede_selected_node_jobs(&mut state, &request.run_id);
            let job_id = Uuid::new_v4().to_string();
            request.query.id = job_id.clone();
            let started = AnalysisJobStartedDto {
                run_id: request.run_id.clone(),
                job_id: job_id.clone(),
                lane: AnalysisJobLaneDto::SelectedNode,
                generation: request.generation,
                node_path: request.node_path.clone(),
            };
            state.jobs.push(RegisteredJob {
                job_id: job_id.clone(),
                run_id: request.run_id.clone(),
                lane: AnalysisJobLane::SelectedNode,
                generation: request.generation,
                node_path: request.node_path.clone(),
                board_size: request.board_size,
                cancel: Arc::new(cancel.clone()),
                disposition: JobDisposition::Running,
                expected: Some(1),
                received_lines: Vec::new(),
                terminal: false,
            });
            publish_event(
                &mut state,
                ForegroundEngineEventDto::Job {
                    job: selected_node_job_event(&started, AnalysisJobOutcomeDto::Started, None, None),
                },
            );
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
            (started, bound_query)
        };
        if let Err(published) = self.write_live_jsonl(&started.run_id, &bound_query) {
            self.abandon_job(&started.run_id, &started.job_id, published.clone());
            return Err(published);
        }
        let inner = self.inner.clone();
        let timed = started.clone();
        let timeout = self.inner.config.job_timeout;
        thread::spawn(move || {
            thread::sleep(timeout);
            inner.timeout_selected_node_job(&timed);
        });
        Ok(started)
    }

    pub fn start_whole_game_analysis(
        &self,
        request: WholeGameJobRequest,
    ) -> Result<AnalysisJobStartedDto, EngineFailureDto> {
        let cancel = AnalysisCancelToken::new();
        let (started, bound_query) = {
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
            let job_id = Uuid::new_v4().to_string();
            let bound_query = bind_query_id(&request.query_jsonl, &job_id)?;
            let started = AnalysisJobStartedDto {
                run_id: request.run_id.clone(),
                job_id: job_id.clone(),
                lane: AnalysisJobLaneDto::WholeGame,
                generation: request.generation,
                node_path: request.node_path.clone(),
            };
            state.jobs.push(RegisteredJob {
                job_id: job_id.clone(),
                run_id: request.run_id.clone(),
                lane: AnalysisJobLane::WholeGame,
                generation: request.generation,
                node_path: request.node_path.clone(),
                board_size: 19,
                cancel: Arc::new(cancel),
                disposition: JobDisposition::Running,
                expected: Some(request.expected_responses),
                received_lines: Vec::new(),
                terminal: false,
            });
            publish_event(
                &mut state,
                ForegroundEngineEventDto::Job {
                    job: selected_node_job_event(&started, AnalysisJobOutcomeDto::Started, None, None),
                },
            );
            (started, bound_query)
        };
        if let Err(published) = self.write_live_jsonl(&started.run_id, &bound_query) {
            self.abandon_job(&started.run_id, &started.job_id, published.clone());
            return Err(published);
        }
        Ok(started)
    }

    pub fn cancel_job(&self, run_id: &str, job_id: &str) -> Result<(), EngineFailureDto> {
        let started = {
            let mut state = self.lock();
            let Some(job) = state
                .jobs
                .iter_mut()
                .find(|job| job.job_id == job_id && job.run_id == run_id)
            else {
                return Err(failure(
                    EngineOperationDto::Job,
                    EngineFailureKind::InvalidState,
                    format!("analysis job not found: {job_id}"),
                    Some(run_id),
                    None,
                    None,
                )
                .with_job_id(job_id));
            };
            if job.disposition != JobDisposition::Running || job.terminal {
                return Err(failure(
                    EngineOperationDto::Job,
                    EngineFailureKind::InvalidState,
                    "cancel requires the current run identity and a non-terminal job".into(),
                    Some(run_id),
                    None,
                    None,
                )
                .with_job_id(job_id));
            }
            let started = started_from(job);
            job.cancel.cancel();
            if job.lane == AnalysisJobLane::SelectedNode {
                job.disposition = JobDisposition::Cancelled;
            }
            mark_job_cancelled(job);
            started
        };
        self.write_terminate(run_id, job_id);
        let mut state = self.lock();
        finish_selected_node_job(
            &mut state,
            &started,
            Some(AnalysisJobOutcomeDto::Cancelled),
            None,
            None,
        );
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
        finish_selected_node_job(
            &mut state,
            &started,
            Some(AnalysisJobOutcomeDto::Failed),
            None,
            Some(published),
        );
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

    fn write_terminate(&self, run_id: &str, job_id: &str) {
        let stdin = {
            let state = self.lock();
            let Some(stdin) = engine_stdin(&state, run_id) else {
                return;
            };
            stdin
        };
        if let Ok(mut guard) = stdin.lock() {
            if let Some(stdin) = guard.as_mut() {
                let payload = format!(r#"{{"id":"{job_id}","action":"terminate"}}"#);
                let _ = write_jsonl(stdin, &payload);
            }
        };
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
        state.phase = Phase::Ready(ready);
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
            cancel_jobs_for_run(&mut state, &primary_run_id);
            let retiring = state.live.take();
            state.live = state.candidate.take();
            state.phase = Phase::Ready(ready);
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

    fn terminate_live(&self, operation: u64) {
        thread::sleep(self.config.stop_drain_timeout);
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
    }

    fn force_no_engine(&self, operation: u64) {
        let mut state = self.lock();
        if state.operation != operation {
            return;
        }
        if let Some(mut live) = state.candidate.take() {
            close_live_stdin(&live);
            let _ = kill_timed_out_child(&mut live.child);
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
            let owned = {
                let state = self.lock();
                state.live.as_ref().is_some_and(|live| live.run_id == run_id)
                    || state.candidate.as_ref().is_some_and(|live| live.run_id == run_id)
            };
            if !owned {
                return;
            }
            match stdout_rx.recv_timeout(Duration::from_millis(50)) {
                Ok(Ok(Some(line))) => self.route_stdout_line(&run_id, line),
                Ok(Ok(None)) | Ok(Err(_)) => return,
                Err(mpsc::RecvTimeoutError::Timeout) => continue,
                Err(mpsc::RecvTimeoutError::Disconnected) => return,
            }
        }
    }

    fn route_stdout_line(&self, run_id: &str, line: String) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return;
        }
        let Some(response_id) = extract_response_id(trimmed) else {
            return;
        };
        let mut state = self.lock();
        let selected = state.jobs.iter().find(|job| {
            job.job_id == response_id
                && job.run_id == run_id
                && !job.terminal
                && job.lane == AnalysisJobLane::SelectedNode
        });
        if let Some(job) = selected {
            if job.disposition != JobDisposition::Running {
                return;
            }
            let started = AnalysisJobStartedDto {
                run_id: job.run_id.clone(),
                job_id: job.job_id.clone(),
                lane: job.lane,
                generation: job.generation,
                node_path: job.node_path.clone(),
            };
            let board_size = job.board_size;
            match parse_response_line(trimmed) {
                Ok(response) if response.id == started.job_id => {
                    let job_uuid = Uuid::parse_str(&started.job_id).unwrap_or_else(|_| Uuid::nil());
                    let frame = normalize_response(job_uuid, response, board_size);
                    finish_selected_node_job(
                        &mut state,
                        &started,
                        Some(AnalysisJobOutcomeDto::Completed),
                        Some(frame),
                        None,
                    );
                }
                Ok(_) => {}
                Err(error) if trimmed.contains(&started.job_id) => {
                    finish_selected_node_job(
                        &mut state,
                        &started,
                        Some(AnalysisJobOutcomeDto::Failed),
                        None,
                        Some(
                            failure(
                                EngineOperationDto::Job,
                                EngineFailureKind::Protocol,
                                format!("selected-node response was not parseable: {error}"),
                                Some(started.run_id.as_str()),
                                None,
                                None,
                            )
                            .with_job_id(&started.job_id),
                        ),
                    );
                }
                Err(_) => {}
            }
            return;
        }
        let Some((started, completed, expected)) = state.jobs.iter_mut().find_map(|job| {
            if job.job_id == response_id
                && job.run_id == run_id
                && !job.terminal
                && job.lane == AnalysisJobLane::WholeGame
            {
                let expected = job.expected?;
                job.received_lines.push(trimmed.to_string());
                let completed = job.received_lines.len();
                Some((started_from(job), completed, expected))
            } else {
                None
            }
        }) else {
            return;
        };
        publish_event(
            &mut state,
            ForegroundEngineEventDto::Job {
                job: AnalysisJobEventDto {
                    run_id: started.run_id.clone(),
                    job_id: started.job_id.clone(),
                    lane: started.lane,
                    generation: started.generation,
                    node_path: started.node_path.clone(),
                    outcome: AnalysisJobOutcomeDto::Progress,
                    completed: Some(completed),
                    expected: Some(expected),
                    frame: None,
                    failure: None,
                },
            },
        );
        if completed >= expected {
            finish_selected_node_job(
                &mut state,
                &started,
                Some(AnalysisJobOutcomeDto::Completed),
                None,
                None,
            );
        }
    }

    fn timeout_selected_node_job(&self, started: &AnalysisJobStartedDto) {
        let mut state = self.lock();
        if admitting_run(&state.phase, &started.run_id).is_none() {
            return;
        }
        {
            let Some(job) = state.jobs.iter_mut().find(|job| {
                job.job_id == started.job_id
                    && job.run_id == started.run_id
                    && job.disposition == JobDisposition::Running
                    && !job.terminal
            }) else {
                return;
            };
            job.disposition = JobDisposition::Cancelled;
            job.cancel.cancel();
            mark_job_cancelled(job);
        }
        if let Some(live) = state.live.as_ref() {
            write_terminate_to_live(live, &started.job_id);
        }
        finish_selected_node_job(
            &mut state,
            started,
            Some(AnalysisJobOutcomeDto::Timeout),
            None,
            None,
        );
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
    let job_ids: Vec<String> = state
        .jobs
        .iter()
        .filter(|job| job.run_id == run_id && !job.terminal)
        .map(|job| job.job_id.clone())
        .collect();
    let mut to_finish = Vec::new();
    for job in &mut state.jobs {
        if job.run_id == run_id && !job.terminal {
            job.cancel.cancel();
            to_finish.push(started_from(job));
            mark_job_cancelled(job);
        }
    }
    if let Some(live) = state.live.as_ref() {
        if live.run_id == run_id {
            for job_id in &job_ids {
                write_terminate_to_live(live, job_id);
            }
        }
    }
    if let Some(live) = state.candidate.as_ref() {
        if live.run_id == run_id {
            for job_id in &job_ids {
                write_terminate_to_live(live, job_id);
            }
        }
    }
    for started in to_finish {
        finish_selected_node_job(
            state,
            &started,
            Some(AnalysisJobOutcomeDto::Cancelled),
            None,
            None,
        );
    }
    state.jobs.retain(|job| job.run_id != run_id);
}

fn mark_job_cancelled(job: &mut RegisteredJob) {
    job.terminal = true;
    if job.lane == AnalysisJobLane::SelectedNode && job.disposition == JobDisposition::Running {
        job.disposition = JobDisposition::Cancelled;
    }
}

fn supersede_selected_node_jobs(state: &mut ManagerState, run_id: &str) {
    let mut events = Vec::new();
    let mut terminate = Vec::new();
    for job in &mut state.jobs {
        if job.run_id == run_id
            && job.lane == AnalysisJobLane::SelectedNode
            && job.disposition == JobDisposition::Running
            && !job.terminal
        {
            job.disposition = JobDisposition::Superseded;
            job.cancel.cancel();
            mark_job_cancelled(job);
            terminate.push(job.job_id.clone());
            events.push(selected_node_job_event(
                &AnalysisJobStartedDto {
                    run_id: job.run_id.clone(),
                    job_id: job.job_id.clone(),
                    lane: job.lane,
                    generation: job.generation,
                    node_path: job.node_path.clone(),
                },
                AnalysisJobOutcomeDto::Superseded,
                None,
                None,
            ));
        }
    }
    if let Some(live) = state.live.as_ref() {
        for job_id in terminate {
            write_terminate_to_live(live, &job_id);
        }
    }
    for event in events {
        publish_event(state, ForegroundEngineEventDto::Job { job: event });
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
        generation: started.generation,
        node_path: started.node_path.clone(),
        outcome,
        completed: None,
        expected: None,
        frame,
        failure,
    }
}

fn started_from(job: &RegisteredJob) -> AnalysisJobStartedDto {
    AnalysisJobStartedDto {
        run_id: job.run_id.clone(),
        job_id: job.job_id.clone(),
        lane: job.lane,
        generation: job.generation,
        node_path: job.node_path.clone(),
    }
}

fn current_non_terminal_job(state: &ManagerState, lane: AnalysisJobLane) -> Option<AnalysisJobStartedDto> {
    state
        .jobs
        .iter()
        .rev()
        .find(|job| job.lane == lane && !job.terminal)
        .map(started_from)
}

fn finish_selected_node_job(
    state: &mut ManagerState,
    started: &AnalysisJobStartedDto,
    outcome: Option<AnalysisJobOutcomeDto>,
    frame: Option<app_model::AnalysisFrameDto>,
    failure: Option<EngineFailureDto>,
) {
    let owned = state
        .jobs
        .iter()
        .any(|job| job.job_id == started.job_id && job.run_id == started.run_id);
    if !owned {
        return;
    }
    state.jobs.retain(|job| job.job_id != started.job_id);
    let Some(outcome) = outcome else {
        return;
    };
    publish_event(
        state,
        ForegroundEngineEventDto::Job {
            job: selected_node_job_event(started, outcome, frame, failure),
        },
    );
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
            let payload = format!(r#"{{"id":"{job_id}","action":"terminate"}}"#);
            let _ = write_jsonl(stdin, &payload);
        }
    }
}

fn bind_query_id(query_jsonl: &str, job_id: &str) -> Result<String, EngineFailureDto> {
    let mut value: serde_json::Value = serde_json::from_str(query_jsonl.trim()).map_err(|error| {
        failure(
            EngineOperationDto::Job,
            EngineFailureKind::Protocol,
            format!("analysis query was not valid JSON: {error}"),
            None,
            None,
            None,
        )
    })?;
    let Some(object) = value.as_object_mut() else {
        return Err(failure(
            EngineOperationDto::Job,
            EngineFailureKind::Protocol,
            "analysis query must be a JSON object".into(),
            None,
            None,
            None,
        ));
    };
    object.insert("id".into(), serde_json::Value::String(job_id.to_string()));
    let encoded = serde_json::to_string(&value).map_err(|error| {
        failure(
            EngineOperationDto::Job,
            EngineFailureKind::Protocol,
            format!("failed to bind analysis job identity: {error}"),
            None,
            None,
            None,
        )
    })?;
    Ok(format!("{encoded}\n"))
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
