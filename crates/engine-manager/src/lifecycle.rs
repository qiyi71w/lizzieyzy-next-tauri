#![allow(clippy::result_large_err)]
use crate::catalog::{EngineProfileCatalog, SavedEngineProfile};
use crate::{
    build_command_spec, build_process_command, check_assets, kill_timed_out_child, spawn_stderr_reader,
    spawn_stdout_lines_reader, write_jsonl, AnalysisCancelToken,
};
use app_model::{
    AnalysisJobEventDto, AnalysisJobLaneDto, AnalysisJobOutcomeDto, AnalysisJobStartedDto, EngineBackend,
    EngineCapabilitySnapshotDto, EngineFailureDto, EngineFailureKind, EngineOperationDto, EngineRunDto,
    ForegroundEngineEventDto, ForegroundEngineLifecycleDto, ForegroundEngineSnapshotDto, NodePath,
};
use katago_protocol::{normalize_response, parse_response_line, terminate_action_jsonl, AnalysisQuery};

pub use app_model::AnalysisJobLaneDto as AnalysisJobLane;
use std::io;
use std::process::{Child, ChildStdin};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use uuid::Uuid;

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
}

impl Default for ForegroundEngineConfig {
    fn default() -> Self {
        Self {
            readiness_timeout: Duration::from_secs(30),
            stop_drain_timeout: Duration::from_secs(2),
            job_timeout: Duration::from_secs(60),
        }
    }
}

impl ForegroundEngineConfig {
    pub fn for_tests() -> Self {
        Self {
            readiness_timeout: Duration::from_secs(2),
            stop_drain_timeout: Duration::from_millis(400),
            job_timeout: Duration::from_millis(800),
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

struct RegisteredJob {
    job_id: String,
    run_id: String,
    lane: AnalysisJobLane,
    generation: u64,
    node_path: NodePath,
    cancel: Arc<dyn AnalysisJobCancel>,
    disposition: JobDisposition,
}

struct LiveEngine {
    child: Child,
    stdin: Option<ChildStdin>,
    stdout_rx: Receiver<io::Result<Option<String>>>,
    #[allow(dead_code)]
    stderr_rx: Receiver<io::Result<String>>,
    run_id: String,
}

enum Phase {
    NoEngine,
    Starting(EngineRunDto),
    Ready(EngineRunDto),
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
    live: Option<LiveEngine>,
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
                    phase: Phase::NoEngine,
                    operation: 0,
                    live: None,
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
        let saved = self.inner.catalog.get(profile_id).ok_or_else(|| {
            failure(
                EngineOperationDto::Start,
                EngineFailureKind::ProfileNotFound,
                format!("saved engine profile was not found: {profile_id}"),
                None,
                Some(profile_id),
                None,
            )
        })?;
        if saved.profile.backend != EngineBackend::KataGoAnalysis {
            return Err(failure(
                EngineOperationDto::Start,
                EngineFailureKind::UnsupportedCapability,
                "R3 Foreground Engine Run only proves KataGoAnalysis".into(),
                None,
                Some(saved.profile_id.as_str()),
                None,
            ));
        }

        let (operation, run) = {
            let mut state = self.lock();
            if !matches!(state.phase, Phase::NoEngine) {
                return Err(failure(
                    EngineOperationDto::Start,
                    EngineFailureKind::InvalidState,
                    "Start requires an authoritative No-engine snapshot".into(),
                    current_run_id(&state.phase).as_deref(),
                    Some(profile_id),
                    None,
                ));
            }
            state.operation += 1;
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
            if let Err(published) = manager.start(&profile_id) {
                manager.inner.publish_failure(published);
            }
        });
        Ok(())
    }

    pub fn register_job(
        &self,
        run_id: &str,
        lane: AnalysisJobLane,
        cancel: Arc<dyn AnalysisJobCancel>,
    ) -> Result<String, EngineFailureDto> {
        let mut state = self.lock();
        let admitted = matches!(
            &state.phase,
            Phase::Ready(run) if run.run_id == run_id && run.capability_snapshot.is_some()
        );
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
            cancel,
            disposition: JobDisposition::Running,
        });
        Ok(job_id)
    }

    pub fn start_selected_node_job(
        &self,
        mut request: SelectedNodeJobRequest,
    ) -> Result<AnalysisJobStartedDto, EngineFailureDto> {
        let cancel = AnalysisCancelToken::new();
        let started = {
            let mut state = self.lock();
            let run = match &state.phase {
                Phase::Ready(run) if run.run_id == request.run_id => run.clone(),
                _ => {
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
                cancel: Arc::new(cancel.clone()),
                disposition: JobDisposition::Running,
            });
            publish_event(
                &mut state,
                ForegroundEngineEventDto::Job {
                    job: job_event(&started, AnalysisJobOutcomeDto::Started, None, None),
                },
            );
            started
        };
        let inner = self.inner.clone();
        let operation = inner.current_operation();
        let worker_started = started.clone();
        thread::spawn(move || {
            inner.run_selected_node_job(operation, worker_started, request, cancel);
        });
        Ok(started)
    }

    pub fn cancel_job(&self, run_id: &str, job_id: &str) -> Result<(), EngineFailureDto> {
        let mut state = self.lock();
        let Some(job) = state.jobs.iter_mut().find(|job| job.job_id == job_id) else {
            return Err(failure(
                EngineOperationDto::Job,
                EngineFailureKind::InvalidState,
                format!("analysis job was not found: {job_id}"),
                Some(run_id),
                None,
                None,
            )
            .with_job_id(job_id));
        };
        if job.run_id != run_id || job.disposition != JobDisposition::Running {
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
        job.disposition = JobDisposition::Cancelled;
        job.cancel.cancel();
        Ok(())
    }

    #[doc(hidden)]
    pub fn set_capability_snapshot_for_tests(&self, snapshot: EngineCapabilitySnapshotDto) {
        let mut state = self.lock();
        if let Phase::Ready(run) = &mut state.phase {
            run.capability_snapshot = Some(snapshot);
        }
    }

    pub fn assert_profile_deletable(&self, profile_id: &str) -> Result<(), EngineFailureDto> {
        let state = self.lock();
        let blocked = match &state.phase {
            Phase::Starting(run) | Phase::Ready(run) | Phase::Stopping(run) => Some(run),
            Phase::Error { run, .. } => Some(run),
            Phase::NoEngine => None,
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
            Phase::NoEngine => return Ok(None),
            Phase::Starting(run) | Phase::Ready(run) | Phase::Stopping(run) => run.clone(),
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
        if let Err(published) = self.start_resident(operation, &run) {
            self.fail_attempt(operation, published);
        }
    }

    fn start_resident(self: &Arc<Self>, operation: u64, run: &EngineRunDto) -> Result<(), EngineFailureDto> {
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
                EngineOperationDto::Start,
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
                EngineOperationDto::Start,
                EngineFailureKind::Start,
                error.to_string(),
                Some(run.run_id.as_str()),
                Some(run.profile_id.as_str()),
                None,
            )
        })?;
        let mut child = build_process_command(&spec).spawn().map_err(|error| {
            failure(
                EngineOperationDto::Start,
                EngineFailureKind::Start,
                format!("failed to spawn engine process: {error}"),
                Some(run.run_id.as_str()),
                Some(run.profile_id.as_str()),
                None,
            )
        })?;
        let mut stdin = child.stdin.take().ok_or_else(|| {
            failure(
                EngineOperationDto::Start,
                EngineFailureKind::Start,
                "engine process stdin was not piped".into(),
                Some(run.run_id.as_str()),
                Some(run.profile_id.as_str()),
                None,
            )
        })?;
        let stdout = child.stdout.take().ok_or_else(|| {
            failure(
                EngineOperationDto::Start,
                EngineFailureKind::Start,
                "engine process stdout was not piped".into(),
                Some(run.run_id.as_str()),
                Some(run.profile_id.as_str()),
                None,
            )
        })?;
        let stderr = child.stderr.take().ok_or_else(|| {
            failure(
                EngineOperationDto::Start,
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
                EngineOperationDto::Start,
                EngineFailureKind::Protocol,
                format!("failed to serialize readiness probe: {error}"),
                Some(run.run_id.as_str()),
                Some(run.profile_id.as_str()),
                None,
            )
        })?;
        write_jsonl(&mut stdin, &query_jsonl).map_err(|error| {
            failure(
                EngineOperationDto::Start,
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
            state.live = Some(LiveEngine {
                child,
                stdin: Some(stdin),
                stdout_rx,
                stderr_rx,
                run_id: run.run_id.clone(),
            });
        }

        self.await_readiness(operation, run, &probe_id)?;
        self.admit_ready(operation, run);
        Ok(())
    }

    fn await_readiness(
        &self,
        operation: u64,
        run: &EngineRunDto,
        probe_id: &str,
    ) -> Result<(), EngineFailureDto> {
        let deadline = Instant::now() + self.config.readiness_timeout;
        loop {
            if self.current_operation() != operation {
                return Ok(());
            }
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Err(failure(
                    EngineOperationDto::Start,
                    EngineFailureKind::Timeout,
                    "engine readiness probe timed out".into(),
                    Some(run.run_id.as_str()),
                    Some(run.profile_id.as_str()),
                    None,
                ));
            }
            let line = {
                let state = self.lock();
                let Some(live) = state.live.as_ref() else {
                    return Err(failure(
                        EngineOperationDto::Start,
                        EngineFailureKind::Start,
                        "engine process was lost before readiness".into(),
                        Some(run.run_id.as_str()),
                        Some(run.profile_id.as_str()),
                        None,
                    ));
                };
                match live.stdout_rx.try_recv() {
                    Ok(Ok(Some(line))) => Some(line),
                    Ok(Ok(None)) => {
                        return Err(failure(
                            EngineOperationDto::Start,
                            EngineFailureKind::Readiness,
                            "engine closed stdout before answering the readiness probe".into(),
                            Some(run.run_id.as_str()),
                            Some(run.profile_id.as_str()),
                            None,
                        ));
                    }
                    Ok(Err(error)) => {
                        return Err(failure(
                            EngineOperationDto::Start,
                            EngineFailureKind::Protocol,
                            format!("failed to read engine stdout: {error}"),
                            Some(run.run_id.as_str()),
                            Some(run.profile_id.as_str()),
                            None,
                        ));
                    }
                    Err(_) => None,
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
                            EngineOperationDto::Start,
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
            whole_game_analysis: true,
            protocol_cancel: true,
        });
        state.phase = Phase::Ready(ready);
        publish_snapshot(&mut state);
        drop(state);
        let inner = self.clone();
        let run_id = run.run_id.clone();
        thread::spawn(move || inner.watch_exit(operation, run_id));
    }

    fn run_selected_node_job(
        self: Arc<Self>,
        operation: u64,
        started: AnalysisJobStartedDto,
        request: SelectedNodeJobRequest,
        cancel: AnalysisCancelToken,
    ) {
        let query_jsonl = match request.query.to_jsonl() {
            Ok(line) => line,
            Err(error) => {
                self.fail_ready_run(
                    operation,
                    &started,
                    failure(
                        EngineOperationDto::Job,
                        EngineFailureKind::Protocol,
                        format!("failed to serialize selected-node query: {error}"),
                        Some(started.run_id.as_str()),
                        None,
                        None,
                    )
                    .with_job_id(&started.job_id),
                );
                return;
            }
        };
        if let Err(published) = self.write_live_jsonl(operation, &started, &query_jsonl) {
            self.fail_ready_run(operation, &started, published);
            return;
        }
        let deadline = Instant::now() + self.config.job_timeout;
        loop {
            if self.current_operation() != operation {
                return;
            }
            let disposition = {
                let state = self.lock();
                state
                    .jobs
                    .iter()
                    .find(|job| job.job_id == started.job_id)
                    .map(|job| job.disposition)
            };
            let Some(disposition) = disposition else {
                return;
            };
            if matches!(
                disposition,
                JobDisposition::Cancelled | JobDisposition::Superseded
            ) || cancel.is_cancelled()
            {
                if let Err(published) =
                    self.write_live_jsonl(operation, &started, &terminate_action_jsonl(&started.job_id))
                {
                    self.fail_ready_run(operation, &started, published);
                    return;
                }
                let outcome = if disposition == JobDisposition::Superseded {
                    None
                } else {
                    Some(AnalysisJobOutcomeDto::Cancelled)
                };
                self.finish_job(&started, outcome, None, None);
                return;
            }
            if deadline.saturating_duration_since(Instant::now()).is_zero() {
                if let Err(published) =
                    self.write_live_jsonl(operation, &started, &terminate_action_jsonl(&started.job_id))
                {
                    self.fail_ready_run(operation, &started, published);
                    return;
                }
                self.finish_job(&started, Some(AnalysisJobOutcomeDto::Timeout), None, None);
                return;
            }
            let line = {
                let state = self.lock();
                let Some(live) = state.live.as_ref() else {
                    drop(state);
                    self.fail_ready_run(
                        operation,
                        &started,
                        failure(
                            EngineOperationDto::Job,
                            EngineFailureKind::Protocol,
                            "engine process was lost during selected-node analysis".into(),
                            Some(started.run_id.as_str()),
                            None,
                            None,
                        )
                        .with_job_id(&started.job_id),
                    );
                    return;
                };
                match live.stdout_rx.try_recv() {
                    Ok(Ok(Some(line))) => Some(line),
                    Ok(Ok(None)) => {
                        drop(state);
                        self.fail_ready_run(
                            operation,
                            &started,
                            failure(
                                EngineOperationDto::Job,
                                EngineFailureKind::Protocol,
                                "engine closed stdout during selected-node analysis".into(),
                                Some(started.run_id.as_str()),
                                None,
                                None,
                            )
                            .with_job_id(&started.job_id),
                        );
                        return;
                    }
                    Ok(Err(error)) => {
                        drop(state);
                        self.fail_ready_run(
                            operation,
                            &started,
                            failure(
                                EngineOperationDto::Job,
                                EngineFailureKind::Protocol,
                                format!("failed to read engine stdout: {error}"),
                                Some(started.run_id.as_str()),
                                None,
                                None,
                            )
                            .with_job_id(&started.job_id),
                        );
                        return;
                    }
                    Err(_) => None,
                }
            };
            let Some(line) = line else {
                thread::sleep(Duration::from_millis(20));
                continue;
            };
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            match parse_response_line(trimmed) {
                Ok(response) if response.id == started.job_id => {
                    let still_running = {
                        let state = self.lock();
                        state.jobs.iter().any(|job| {
                            job.job_id == started.job_id && job.disposition == JobDisposition::Running
                        })
                    };
                    if !still_running {
                        return;
                    }
                    let job_uuid = Uuid::parse_str(&started.job_id).unwrap_or_else(|_| Uuid::nil());
                    let frame = normalize_response(job_uuid, response, request.board_size);
                    self.finish_job(
                        &started,
                        Some(AnalysisJobOutcomeDto::Completed),
                        Some(frame),
                        None,
                    );
                    return;
                }
                Ok(_) => continue,
                Err(error) => {
                    if trimmed.contains(&started.job_id) {
                        self.finish_job(
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
                        return;
                    }
                }
            }
        }
    }

    fn write_live_jsonl(
        &self,
        operation: u64,
        started: &AnalysisJobStartedDto,
        payload: &str,
    ) -> Result<(), EngineFailureDto> {
        let mut state = self.lock();
        if state.operation != operation {
            return Ok(());
        }
        write_live_line(&mut state, payload).map_err(|error| {
            failure(
                EngineOperationDto::Job,
                EngineFailureKind::Protocol,
                format!("failed to write engine protocol: {error}"),
                Some(started.run_id.as_str()),
                None,
                None,
            )
            .with_job_id(&started.job_id)
        })
    }

    fn finish_job(
        &self,
        started: &AnalysisJobStartedDto,
        outcome: Option<AnalysisJobOutcomeDto>,
        frame: Option<app_model::AnalysisFrameDto>,
        failure: Option<EngineFailureDto>,
    ) {
        let mut state = self.lock();
        state.jobs.retain(|job| job.job_id != started.job_id);
        if let Some(outcome) = outcome {
            publish_event(
                &mut state,
                ForegroundEngineEventDto::Job {
                    job: job_event(started, outcome, frame, failure),
                },
            );
        }
    }

    fn fail_ready_run(&self, operation: u64, started: &AnalysisJobStartedDto, published: EngineFailureDto) {
        let mut state = self.lock();
        if state.operation != operation {
            return;
        }
        let run = match &state.phase {
            Phase::Ready(run) if run.run_id == started.run_id => run.clone(),
            _ => return,
        };
        cancel_jobs_for_current(&mut state);
        if let Some(mut live) = state.live.take() {
            drop(live.stdin.take());
            let _ = kill_timed_out_child(&mut live.child);
        }
        publish_event(
            &mut state,
            ForegroundEngineEventDto::Job {
                job: job_event(
                    started,
                    AnalysisJobOutcomeDto::Failed,
                    None,
                    Some(published.clone()),
                ),
            },
        );
        state.phase = Phase::Error {
            run,
            failure: published.clone(),
        };
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
            drop(live.stdin.take());
            let _ = kill_timed_out_child(&mut live.child);
        }
        state.jobs.clear();
        state.phase = Phase::NoEngine;
        publish_snapshot(&mut state);
        publish_event(
            &mut state,
            ForegroundEngineEventDto::Failure { failure: published },
        );
    }

    fn publish_failure(&self, published: EngineFailureDto) {
        let mut state = self.lock();
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
            drop(live.stdin.take());
            let _ = kill_timed_out_child(&mut live.child);
        }
    }

    fn force_no_engine(&self, operation: u64) {
        let mut state = self.lock();
        if state.operation != operation {
            return;
        }
        state.jobs.clear();
        state.phase = Phase::NoEngine;
        publish_snapshot(&mut state);
    }

    fn watch_exit(self: Arc<Self>, operation: u64, run_id: String) {
        loop {
            thread::sleep(Duration::from_millis(30));
            let status = {
                let mut state = self.lock();
                if state.operation != operation {
                    return;
                }
                let Some(live) = state.live.as_mut() else {
                    return;
                };
                if live.run_id != run_id {
                    return;
                }
                match live.child.try_wait() {
                    Ok(Some(status)) => Some(status.code()),
                    Ok(None) => None,
                    Err(_) => Some(None),
                }
            };
            if let Some(exit_code) = status {
                self.handle_unexpected_exit(operation, &run_id, exit_code);
                return;
            }
        }
    }

    fn handle_unexpected_exit(&self, operation: u64, run_id: &str, exit_code: Option<i32>) {
        let mut state = self.lock();
        if state.operation != operation {
            return;
        }
        match &state.phase {
            Phase::Starting(run) if run.run_id == run_id => {
                let profile_id = run.profile_id.clone();
                drop(state);
                self.fail_attempt(
                    operation,
                    failure(
                        EngineOperationDto::Start,
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
                return;
            }
            Phase::Ready(run) if run.run_id == run_id => {}
            _ => return,
        }
        let run = match &state.phase {
            Phase::Ready(run) => run.clone(),
            _ => return,
        };
        cancel_jobs_for_current(&mut state);
        if let Some(mut live) = state.live.take() {
            let _ = live.child.wait();
        }
        let published = failure(
            EngineOperationDto::UnexpectedExit,
            EngineFailureKind::NonzeroExit,
            format!("engine process exited unexpectedly; exit_code={exit_code:?}"),
            Some(run_id),
            Some(run.profile_id.as_str()),
            None,
        );
        state.phase = Phase::Error {
            run,
            failure: published.clone(),
        };
        publish_snapshot(&mut state);
        publish_event(
            &mut state,
            ForegroundEngineEventDto::Failure { failure: published },
        );
    }
}

fn supersede_selected_node_jobs(state: &mut ManagerState, run_id: &str) {
    let mut events = Vec::new();
    for job in &mut state.jobs {
        if job.run_id == run_id
            && job.lane == AnalysisJobLane::SelectedNode
            && job.disposition == JobDisposition::Running
        {
            job.disposition = JobDisposition::Superseded;
            job.cancel.cancel();
            events.push(AnalysisJobEventDto {
                run_id: job.run_id.clone(),
                job_id: job.job_id.clone(),
                lane: job.lane,
                generation: job.generation,
                node_path: job.node_path.clone(),
                outcome: AnalysisJobOutcomeDto::Superseded,
                frame: None,
                failure: None,
            });
        }
    }
    for event in events {
        publish_event(state, ForegroundEngineEventDto::Job { job: event });
    }
}

fn job_event(
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
        frame,
        failure,
    }
}

fn write_live_line(state: &mut ManagerState, payload: &str) -> io::Result<()> {
    let live = state
        .live
        .as_mut()
        .ok_or_else(|| io::Error::new(io::ErrorKind::BrokenPipe, "no live engine"))?;
    let stdin = live
        .stdin
        .as_mut()
        .ok_or_else(|| io::Error::new(io::ErrorKind::BrokenPipe, "engine stdin was closed"))?;
    write_jsonl(stdin, payload)
}

fn snapshot_from(state: &ManagerState) -> ForegroundEngineSnapshotDto {
    let lifecycle = match &state.phase {
        Phase::NoEngine => ForegroundEngineLifecycleDto::NoEngine,
        Phase::Starting(run) => ForegroundEngineLifecycleDto::Starting { run: run.clone() },
        Phase::Ready(run) => ForegroundEngineLifecycleDto::Ready { run: run.clone() },
        Phase::Stopping(run) => ForegroundEngineLifecycleDto::Stopping { run: run.clone() },
        Phase::Error { run, failure } => ForegroundEngineLifecycleDto::Error {
            run: run.clone(),
            failure: failure.clone(),
        },
    };
    ForegroundEngineSnapshotDto {
        revision: state.revision,
        lifecycle,
    }
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
    let mut remaining = Vec::new();
    for job in state.jobs.drain(..) {
        if job.run_id == run_id {
            job.cancel.cancel();
        } else {
            remaining.push(job);
        }
    }
    state.jobs = remaining;
}

fn current_run_id(phase: &Phase) -> Option<String> {
    match phase {
        Phase::NoEngine => None,
        Phase::Starting(run) | Phase::Ready(run) | Phase::Stopping(run) => Some(run.run_id.clone()),
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
