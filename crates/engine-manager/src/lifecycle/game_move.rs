use super::*;
use crate::game_move_protocol::{
    color_text, gtp_sync_plan, katago_query, katago_result, parse_gtp_move, KataGoMoveResponse,
};
use app_model::{GameMoveDto, GameMoveJobDto, GameMoveRequestDto, GameMoveResultDto};
use std::sync::mpsc::SyncSender;

#[cfg(test)]
mod real_smoke;

/// The exact position can only be obtained by strict SGF projection.
#[derive(Clone, Debug)]
pub struct GameMoveRequest {
    pub identity: GameMoveRequestDto,
    pub position: sgf::ExactPosition,
}

pub struct GameMoveHandle {
    pub identity: GameMoveJobDto,
    completion: Receiver<MoveCompletion>,
    frames: Option<Receiver<app_model::AnalysisFrameDto>>,
}

pub struct OrdinaryRulesHandle(GameMoveHandle);
impl OrdinaryRulesHandle {
    pub fn wait(self) -> Result<app_model::OrdinaryRulesSnapshotDto, EngineFailureDto> {
        let handle = self.0;
        let (result, _, snapshot) = handle.completion.recv().map_err(|_| {
            move_failure(
                &handle.identity,
                EngineFailureKind::Protocol,
                "rules worker disconnected",
            )
        })?;
        result?;
        snapshot.ok_or_else(|| {
            move_failure(
                &handle.identity,
                EngineFailureKind::Protocol,
                "rules confirmation absent",
            )
        })
    }
}

type MoveCompletion = (
    Result<Option<GameMoveResultDto>, EngineFailureDto>,
    Option<app_model::AnalysisFrameDto>,
    Option<app_model::OrdinaryRulesSnapshotDto>,
);

impl GameMoveHandle {
    pub fn wait(self) -> Result<GameMoveResultDto, EngineFailureDto> {
        self.completion
            .recv()
            .map(|(result, _, _)| {
                result.and_then(|result| {
                    result.ok_or_else(|| {
                        move_failure(
                            &self.identity,
                            EngineFailureKind::InvalidState,
                            "not a move operation",
                        )
                    })
                })
            })
            .unwrap_or_else(|_| {
                Err(move_failure(
                    &self.identity,
                    EngineFailureKind::Protocol,
                    "move worker disconnected",
                ))
            })
    }

    /// Frames share this move's exact identity. A slow consumer never blocks the worker.
    pub fn wait_with_analysis(
        self,
        mut on_frame: impl FnMut(&GameMoveJobDto, app_model::AnalysisFrameDto),
    ) -> Result<GameMoveResultDto, EngineFailureDto> {
        let Some(frames) = self.frames.as_ref() else {
            return self.wait();
        };
        loop {
            match self.completion.try_recv() {
                Ok((result, final_frame, _)) => {
                    while let Ok(frame) = frames.try_recv() {
                        on_frame(&self.identity, frame);
                    }
                    if let Some(frame) = final_frame {
                        on_frame(&self.identity, frame);
                    }
                    return result.and_then(|result| {
                        result.ok_or_else(|| {
                            move_failure(
                                &self.identity,
                                EngineFailureKind::InvalidState,
                                "not a move operation",
                            )
                        })
                    });
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    return Err(move_failure(
                        &self.identity,
                        EngineFailureKind::Protocol,
                        "move worker disconnected",
                    ));
                }
                Err(mpsc::TryRecvError::Empty) => {}
            }
            match frames.recv_timeout(Duration::from_millis(5)) {
                Ok(frame) => on_frame(&self.identity, frame),
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    // Completion follows worker cleanup, which may still be joining its writer.
                    match self.completion.recv() {
                        Ok((result, final_frame, _)) => {
                            if let Some(frame) = final_frame {
                                on_frame(&self.identity, frame);
                            }
                            return result.and_then(|result| {
                                result.ok_or_else(|| {
                                    move_failure(
                                        &self.identity,
                                        EngineFailureKind::InvalidState,
                                        "not a move operation",
                                    )
                                })
                            });
                        }
                        Err(_) => {
                            return Err(move_failure(
                                &self.identity,
                                EngineFailureKind::Protocol,
                                "move worker disconnected",
                            ))
                        }
                    }
                }
            }
        }
    }
}

pub(super) struct MoveSlot {
    pub(super) identity: GameMoveJobDto,
    events: SyncSender<MoveInput>,
    sealed: Option<EngineFailureDto>,
    adapter: EngineBackend,
    profile_id: String,
    pub(super) finished: bool,
    analysis_only: bool,
}

enum MoveInput {
    Line(String),
    Written(Result<(), String>),
}

struct MoveWorker {
    manager: ForegroundEngineManager,
    request: GameMoveRequest,
    identity: GameMoveJobDto,
    job_uuid: Uuid,
    run: EngineRunDto,
    deadline: Instant,
    events: Receiver<MoveInput>,
    writes: Sender<String>,
    submitted: bool,
    query_rejected: bool,
    mode: MoveMode,
    frames: Option<SyncSender<app_model::AnalysisFrameDto>>,
    final_frame: Option<app_model::AnalysisFrameDto>,
    rules_snapshot: Option<app_model::OrdinaryRulesSnapshotDto>,
    restore_file: Option<super::ordinary_rules::RestoreFile>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum MoveMode {
    Move,
    MoveWithAnalysis,
    AnalysisOnly,
    ConfirmRules,
}

fn move_failure(identity: &GameMoveJobDto, kind: EngineFailureKind, message: &str) -> EngineFailureDto {
    failure(
        EngineOperationDto::Job,
        kind,
        message.into(),
        Some(&identity.run_id),
        None,
        None,
    )
    .with_job_id(&identity.job_id)
}

pub(super) fn require_idle_move(state: &ManagerState) -> Result<(), EngineFailureDto> {
    if let Some(slot) = &state.game_move {
        return Err(move_failure(
            &slot.identity,
            EngineFailureKind::Occupied,
            "the foreground move operation still owns this run",
        ));
    }
    Ok(())
}

pub(super) fn seal_move_for_run(state: &mut ManagerState, run_id: &str) {
    if state
        .game_move_publication
        .as_ref()
        .is_some_and(|job| job.run_id == run_id)
    {
        state.game_move_publication = None;
    }
    if let Some(slot) = state
        .game_move
        .as_mut()
        .filter(|slot| slot.identity.run_id == run_id)
    {
        slot.sealed.get_or_insert_with(|| {
            let mut error = move_failure(
                &slot.identity,
                EngineFailureKind::Cancellation,
                "the foreground run was retired",
            );
            error.profile_id = Some(slot.profile_id.clone());
            error
        });
    }
}

pub(super) fn fail_move_for_run(state: &mut ManagerState, failure: &EngineFailureDto) {
    if state
        .game_move_publication
        .as_ref()
        .is_some_and(|job| Some(&job.run_id) == failure.run_id.as_ref())
    {
        state.game_move_publication = None;
    }
    if let Some(slot) = state
        .game_move
        .as_mut()
        .filter(|slot| failure.run_id.as_ref() == Some(&slot.identity.run_id))
    {
        slot.sealed.get_or_insert_with(|| {
            let mut error = failure.clone().with_job_id(&slot.identity.job_id);
            error.profile_id = Some(slot.profile_id.clone());
            error
        });
    }
}

pub(super) fn validate_move_position(
    run: &EngineRunDto,
    position: &sgf::ExactPosition,
    budget: app_model::ComputeBudgetDto,
) -> Result<Vec<String>, EngineFailureDto> {
    let fail = |message: &str| {
        failure(
            EngineOperationDto::Job,
            EngineFailureKind::UnsupportedCapability,
            message.into(),
            Some(&run.run_id),
            Some(&run.profile_id),
            None,
        )
    };
    if !run
        .capability_snapshot
        .as_ref()
        .is_some_and(|caps| caps.game_move)
    {
        return Err(fail("run does not admit game moves"));
    }
    if budget.deadline_ms == 0 {
        return Err(fail("move deadline must be positive"));
    }
    match run.adapter_kind {
        EngineBackend::KataGoAnalysis => {
            let capabilities = analysis_capabilities(run)?;
            if !capabilities.visits_limit
                || !capabilities.protocol_cancel
                || budget.max_visits.is_none_or(|visits| visits == 0)
                || position.dto().komi.abs() > 400.0
            {
                return Err(fail(
                    "KataGo move requires positive max_visits, target cancellation and komi in [-400,400]",
                ));
            }
            Ok(Vec::new())
        }
        EngineBackend::KataGoGtp => Err(fail("KataGo GTP move computation is not admitted")),
        EngineBackend::GenericGtp => {
            if budget.max_visits.is_some() {
                return Err(fail("GTP cannot enforce a visits budget"));
            }
            gtp_sync_plan(run, position.dto()).map_err(|message| fail(&message))
        }
    }
}

impl ForegroundEngineManager {
    pub fn start_ordinary_rules(
        &self,
        request: GameMoveRequest,
    ) -> Result<OrdinaryRulesHandle, EngineFailureDto> {
        self.start_game_move_owned(None, request, MoveMode::ConfirmRules)
            .map(OrdinaryRulesHandle)
    }

    pub fn confirm_ordinary_rules(
        &self,
        request: GameMoveRequest,
    ) -> Result<app_model::OrdinaryRulesSnapshotDto, EngineFailureDto> {
        self.start_ordinary_rules(request)?.wait()
    }

    pub fn claim_ordinary_rules(
        &self,
        snapshot: &app_model::OrdinaryRulesSnapshotDto,
    ) -> Result<(), EngineFailureDto> {
        let mut state = self.lock();
        if state.game_move_publication.as_ref() != Some(&snapshot.identity)
            || ready_move_run(&state, &snapshot.identity.run_id).is_none()
        {
            return Err(move_failure(
                &snapshot.identity,
                EngineFailureKind::Cancellation,
                "rules confirmation was retired",
            ));
        }
        state.game_move_publication = None;
        Ok(())
    }

    pub fn start_game_move(&self, request: GameMoveRequest) -> Result<GameMoveHandle, EngineFailureDto> {
        self.start_game_move_owned(None, request, MoveMode::Move)
    }

    pub fn start_reserved_game_move(
        &self,
        owner: &str,
        request: GameMoveRequest,
    ) -> Result<GameMoveHandle, EngineFailureDto> {
        self.start_game_move_owned(Some(owner), request, MoveMode::Move)
    }

    /// Computes session-only candidates without granting permission to publish a move.
    pub fn start_reserved_analysis(
        &self,
        owner: &str,
        request: GameMoveRequest,
    ) -> Result<GameMoveHandle, EngineFailureDto> {
        self.start_game_move_owned(Some(owner), request, MoveMode::AnalysisOnly)
    }

    /// Reports candidates from the actual engine move query, never a second analysis lane.
    pub fn start_reserved_game_move_with_analysis(
        &self,
        owner: &str,
        request: GameMoveRequest,
    ) -> Result<GameMoveHandle, EngineFailureDto> {
        self.start_game_move_owned(Some(owner), request, MoveMode::MoveWithAnalysis)
    }

    fn start_game_move_owned(
        &self,
        owner: Option<&str>,
        request: GameMoveRequest,
        mode: MoveMode,
    ) -> Result<GameMoveHandle, EngineFailureDto> {
        let job_uuid = Uuid::new_v4();
        let identity = GameMoveJobDto {
            run_id: request.identity.run_id.clone(),
            job_id: job_uuid.to_string(),
            generation: request.identity.generation,
            node_path: request.identity.node_path.clone(),
        };
        let fail = |kind, message: &str| move_failure(&identity, kind, message);
        let (events_tx, events) = mpsc::sync_channel(64);
        let (writes, writes_rx) = mpsc::channel::<String>();
        let (completed, completion) = mpsc::channel();
        let (frames_tx, frames) = if matches!(mode, MoveMode::Move | MoveMode::ConfirmRules) {
            (None, None)
        } else {
            let (sender, receiver) = mpsc::sync_channel(8);
            (Some(sender), Some(receiver))
        };
        let (run, plan, stdin, deadline, restore_file) = {
            let mut state = self.lock();
            match_reservation::require_move_owner(&state, owner, &identity.run_id)?;
            // Complete pure admission precedes any cancellation, process write or slot mutation.
            let run = ready_move_run(&state, &identity.run_id).ok_or_else(|| {
                fail(
                    EngineFailureKind::InvalidState,
                    "move requires a living Ready owned run",
                )
            })?;
            let budget = request.identity.budget;
            let plan = if mode == MoveMode::ConfirmRules {
                super::ordinary_rules::admit(&run, &request.position)
                    .map_err(|message| fail(EngineFailureKind::UnsupportedCapability, &message))?;
                Vec::new()
            } else {
                validate_move_position(&run, &request.position, budget)?
            };
            if matches!(mode, MoveMode::MoveWithAnalysis | MoveMode::AnalysisOnly) {
                if run.adapter_kind != EngineBackend::KataGoAnalysis {
                    return Err(fail(
                        EngineFailureKind::UnsupportedCapability,
                        "reserved analysis requires a verified KataGo analysis run",
                    ));
                }
                let capabilities = analysis_capabilities(&run)?;
                if !capabilities.candidates
                    || !capabilities.selected_node_analysis
                    || !capabilities.visits_limit
                    || !capabilities.protocol_cancel
                {
                    return Err(fail(EngineFailureKind::UnsupportedCapability,
                        "reserved analysis requires verified candidates, selected-node analysis, visits limits and target cancellation"));
                }
            }
            require_idle_move(&state)?;
            if state.finite_admission_pending
                || state.continuous_departing
                || state
                    .jobs
                    .iter()
                    .any(|job| job.run_id == run.run_id && !job.terminal)
                || state.analysis_task.as_ref().is_some_and(|task| {
                    matches!(
                        task.state,
                        AnalysisTaskStateDto::Queued
                            | AnalysisTaskStateDto::Searching
                            | AnalysisTaskStateDto::Pausing
                            | AnalysisTaskStateDto::Paused
                    )
                })
            {
                return Err(fail(
                    EngineFailureKind::Occupied,
                    "an analysis lane or task still owns this run",
                ));
            }
            let stdin = engine_stdin(&state, &run.run_id)
                .ok_or_else(|| fail(EngineFailureKind::InvalidState, "engine stdin unavailable"))?;
            let deadline = Instant::now() + Duration::from_millis(u64::from(budget.deadline_ms));
            let restore_file = if mode == MoveMode::ConfirmRules {
                Some(
                    super::ordinary_rules::RestoreFile::create(&run, request.position.dto())
                        .map_err(|message| fail(EngineFailureKind::UnsupportedCapability, &message))?,
                )
            } else {
                None
            };
            state.game_move_publication = None;
            state.game_move = Some(MoveSlot {
                identity: identity.clone(),
                events: events_tx.clone(),
                sealed: None,
                adapter: run.adapter_kind,
                profile_id: run.profile_id.clone(),
                finished: false,
                analysis_only: mode == MoveMode::AnalysisOnly,
            });
            publish_snapshot(&mut state);
            (run, plan, stdin, deadline, restore_file)
        };
        // A blocked pipe must not block the deadline owner. Retirement kills the child before
        // taking/closing its stdin; the writer then observes the broken pipe and exits.
        let writer = thread::spawn(move || {
            for payload in writes_rx {
                #[cfg(test)]
                real_smoke::record("stdin", &payload);
                let result = stdin
                    .lock()
                    .map_err(|_| "engine stdin lock poisoned".to_owned())
                    .and_then(|mut guard| {
                        let input = guard.as_mut().ok_or_else(|| "engine stdin closed".to_owned())?;
                        write_jsonl(input, &payload).map_err(|error| error.to_string())
                    });
                if events_tx.send(MoveInput::Written(result)).is_err() {
                    break;
                }
            }
        });
        let manager = self.clone();
        let worker_identity = identity.clone();
        thread::spawn(move || {
            let mut worker = MoveWorker {
                manager,
                request,
                identity: worker_identity,
                job_uuid,
                run,
                deadline,
                events,
                writes,
                submitted: false,
                query_rejected: false,
                mode,
                frames: frames_tx,
                final_frame: None,
                rules_snapshot: None,
                restore_file,
            };
            let result = worker.compute(plan);
            let result = worker.finish(result);
            let final_frame = if result.is_ok() {
                worker.final_frame.take()
            } else {
                None
            };
            let rules_snapshot = if result.is_ok() {
                worker.rules_snapshot.take()
            } else {
                None
            };
            // Drop the receiver before joining, so a saturated event channel cannot hold the writer.
            drop(worker);
            let _ = writer.join();
            let _ = completed.send((result, final_frame, rules_snapshot));
        });
        Ok(GameMoveHandle {
            identity,
            completion,
            frames,
        })
    }

    /// Seals only session analysis, then waits for the existing worker's target drain.
    pub fn cancel_reserved_analysis(
        &self,
        owner: &str,
        job: &GameMoveJobDto,
    ) -> Result<(), EngineFailureDto> {
        {
            let mut state = self.lock();
            match_reservation::require_move_owner(&state, Some(owner), &job.run_id)?;
            if !matches!(&state.phase, Phase::Ready(run) if run.run_id == job.run_id) {
                return Err(move_failure(
                    job,
                    EngineFailureKind::InvalidState,
                    "reserved analysis cancellation requires the current Ready run",
                ));
            }
            let Some(slot) = state
                .game_move
                .as_mut()
                .filter(|slot| slot.identity.run_id == job.run_id && slot.identity.job_id == job.job_id)
            else {
                // A completed target cannot cancel a subsequent move or consume its permit.
                return Ok(());
            };
            if slot.identity != *job || !slot.analysis_only {
                return Err(move_failure(
                    job,
                    EngineFailureKind::InvalidState,
                    "reserved analysis cancellation cannot stop an engine move or mismatched identity",
                ));
            }
            slot.sealed.get_or_insert_with(|| {
                move_failure(
                    job,
                    EngineFailureKind::Cancellation,
                    "reserved analysis cancelled by owner",
                )
            });
        }
        let deadline = Instant::now() + self.inner.config.stop_drain_timeout * 3 + Duration::from_secs(1);
        loop {
            let state = self.lock();
            match_reservation::require_move_owner(&state, Some(owner), &job.run_id)?;
            if state.game_move.as_ref().is_none_or(|slot| slot.identity != *job) {
                return match &state.phase {
                    Phase::Ready(run) if run.run_id == job.run_id => Ok(()),
                    Phase::Error { failure, .. } => Err(failure.clone()),
                    _ => Err(move_failure(
                        job,
                        EngineFailureKind::InvalidState,
                        "reserved analysis run changed during cleanup",
                    )),
                };
            }
            if Instant::now() >= deadline {
                return Err(move_failure(
                    job,
                    EngineFailureKind::Timeout,
                    "reserved analysis cleanup did not release the move slot",
                ));
            }
            drop(state);
            thread::sleep(Duration::from_millis(5));
        }
    }

    pub fn cancel_game_move(&self, run_id: &str, job_id: &str) -> Result<(), EngineFailureDto> {
        let mut state = self.lock();
        match_reservation::require_unreserved(&state)?;
        if state
            .game_move_publication
            .as_ref()
            .is_some_and(|job| job.run_id == run_id && job.job_id == job_id)
        {
            state.game_move_publication = None;
            return Ok(());
        }
        let slot = state
            .game_move
            .as_mut()
            .filter(|slot| slot.identity.run_id == run_id && slot.identity.job_id == job_id)
            .ok_or_else(|| {
                failure(
                    EngineOperationDto::Job,
                    EngineFailureKind::InvalidState,
                    "move identity is no longer active".into(),
                    Some(run_id),
                    None,
                    None,
                )
                .with_job_id(job_id)
            })?;
        slot.sealed.get_or_insert_with(|| {
            move_failure(
                &slot.identity,
                EngineFailureKind::Cancellation,
                "move cancelled by caller",
            )
        });
        Ok(())
    }

    /// Consumes the latest completed result's publication permit exactly once.
    /// Position/mode/lifecycle invalidation remains effective after compute cleanup.
    pub fn claim_game_move_result(&self, result: &GameMoveResultDto) -> Result<(), EngineFailureDto> {
        self.claim_game_move_result_owned(None, result)
    }

    pub fn claim_reserved_game_move_result(
        &self,
        owner: &str,
        result: &GameMoveResultDto,
    ) -> Result<(), EngineFailureDto> {
        self.claim_game_move_result_owned(Some(owner), result)
    }

    fn claim_game_move_result_owned(
        &self,
        owner: Option<&str>,
        result: &GameMoveResultDto,
    ) -> Result<(), EngineFailureDto> {
        let mut state = self.lock();
        match_reservation::require_move_owner(&state, owner, &result.run_id)?;
        if !state.game_move_publication.as_ref().is_some_and(|identity| {
            identity.run_id == result.run_id
                && identity.job_id == result.job_id
                && identity.generation == result.generation
                && identity.node_path == result.node_path
        }) || ready_move_run(&state, &result.run_id).is_none()
        {
            return Err(failure(
                EngineOperationDto::Job,
                EngineFailureKind::Cancellation,
                "move publication was retired".into(),
                Some(&result.run_id),
                None,
                None,
            )
            .with_job_id(&result.job_id));
        }
        state.game_move_publication = None;
        Ok(())
    }

    pub fn cancel_current_game_move(&self) {
        let mut state = self.lock();
        if state.match_reservation.is_some() {
            return;
        }
        state.game_move_publication = None;
        if let Some(slot) = state.game_move.as_mut() {
            slot.sealed.get_or_insert_with(|| {
                move_failure(
                    &slot.identity,
                    EngineFailureKind::Cancellation,
                    "move cancelled by current-game transition",
                )
            });
        }
    }

    pub fn invalidate_game_move_position(&self, generation: u64, path: &NodePath) {
        let mut state = self.lock();
        if state
            .game_move_publication
            .as_ref()
            .is_some_and(|job| job.generation != generation || job.node_path != *path)
        {
            state.game_move_publication = None;
        }
        if let Some(slot) = state
            .game_move
            .as_mut()
            .filter(|slot| slot.identity.generation != generation || slot.identity.node_path != *path)
        {
            slot.sealed.get_or_insert_with(|| {
                move_failure(
                    &slot.identity,
                    EngineFailureKind::Cancellation,
                    "current game generation or selected path changed",
                )
            });
        }
    }

    pub(super) fn finish_move_before_switch(&self) -> Result<bool, EngineFailureDto> {
        {
            let mut state = self.lock();
            match_reservation::require_unreserved(&state)?;
            state.game_move_publication = None;
        }
        let active = self.snapshot().game_move_job;
        let Some(active) = active else {
            return Ok(false);
        };
        self.cancel_game_move(&active.run_id, &active.job_id)?;
        let deadline = Instant::now() + self.inner.config.stop_drain_timeout * 3 + Duration::from_secs(1);
        loop {
            let state = self.lock();
            if state
                .game_move
                .as_ref()
                .is_none_or(|slot| slot.identity != active)
            {
                return match &state.phase {
                    Phase::NoEngine { .. } => Ok(true),
                    Phase::Error { failure, .. } => Err(failure.clone()),
                    _ => Ok(false),
                };
            }
            if Instant::now() >= deadline {
                return Err(move_failure(
                    &active,
                    EngineFailureKind::Timeout,
                    "move cleanup did not finish before switch",
                ));
            }
            drop(state);
            thread::sleep(Duration::from_millis(5));
        }
    }
}

impl Inner {
    pub(super) fn route_game_move_line(&self, run_id: &str, line: &str) -> bool {
        let mut state = self.lock();
        let Some(slot) = state
            .game_move
            .as_mut()
            .filter(|slot| slot.identity.run_id == run_id)
        else {
            return false;
        };
        if slot.adapter == EngineBackend::KataGoGtp && gtp::is_analysis_stream_record(line) {
            return true;
        }
        #[cfg(test)]
        real_smoke::record("stdout", line);
        if slot.adapter == EngineBackend::KataGoAnalysis {
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(line) {
                // Old query responses and terminate acknowledgements are not this move's result.
                if value
                    .get("id")
                    .and_then(|id| id.as_str())
                    .is_some_and(|id| id != slot.identity.job_id)
                {
                    return true;
                }
            }
        }
        if slot.events.try_send(MoveInput::Line(line.to_owned())).is_err() {
            slot.sealed.get_or_insert_with(|| {
                move_failure(
                    &slot.identity,
                    EngineFailureKind::Protocol,
                    "move stdout exceeded the bounded event queue",
                )
            });
        }
        true
    }
}

impl MoveWorker {
    fn error(&self, kind: EngineFailureKind, message: &str) -> EngineFailureDto {
        let mut error = move_failure(&self.identity, kind, message);
        error.profile_id = Some(self.run.profile_id.clone());
        error
    }

    fn write_error(&self, message: &str) -> EngineFailureDto {
        let mut error = self.error(EngineFailureKind::Protocol, message);
        if self
            .manager
            .lock()
            .match_reservation
            .as_ref()
            .is_some_and(|reservation| reservation.runs.len() == 2)
        {
            error.operation = EngineOperationDto::UnexpectedExit;
        }
        error
    }

    fn check(&self) -> Result<(), EngineFailureDto> {
        let state = self.manager.lock();
        let slot = state
            .game_move
            .as_ref()
            .filter(|slot| slot.identity == self.identity)
            .ok_or_else(|| self.error(EngineFailureKind::Cancellation, "move slot was retired"))?;
        if let Some(error) = &slot.sealed {
            return Err(error.clone());
        }
        if ready_move_run(&state, &self.identity.run_id).is_none() {
            return Err(self.error(EngineFailureKind::Cancellation, "foreground run changed"));
        }
        if Instant::now() >= self.deadline {
            return Err(self.error(EngineFailureKind::Timeout, "move hard deadline expired"));
        }
        Ok(())
    }

    fn receive(&self) -> Result<MoveInput, EngineFailureDto> {
        loop {
            self.check()?;
            let remaining = self.deadline.saturating_duration_since(Instant::now());
            match self.events.recv_timeout(remaining.min(Duration::from_millis(5))) {
                Ok(event) => {
                    self.check()?;
                    return Ok(event);
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(_) => return Err(self.error(EngineFailureKind::Protocol, "move event stream closed")),
            }
        }
    }

    fn write(&mut self, payload: String) -> Result<(), EngineFailureDto> {
        self.check()?;
        self.submitted = true;
        self.writes
            .send(payload)
            .map_err(|_| self.error(EngineFailureKind::Protocol, "move writer closed"))
    }

    fn gtp_command(&mut self, command: String) -> Result<String, EngineFailureDto> {
        self.check()?;
        let id = {
            let mut state = self.manager.lock();
            state.gtp_command_seq = state
                .gtp_command_seq
                .checked_add(1)
                .ok_or_else(|| self.error(EngineFailureKind::Protocol, "GTP command ID exhausted"))?;
            state.gtp_command_seq
        };
        self.write(format!("{id} {command}"))?;
        let mut decoder = ResponseDecoder::new(id);
        let mut written = false;
        let mut acknowledged = None;
        loop {
            match self.receive()? {
                MoveInput::Written(Ok(())) => written = true,
                MoveInput::Written(Err(message)) => return Err(self.write_error(&message)),
                MoveInput::Line(line) => {
                    if let Some(response) = decoder
                        .push(&line)
                        .map_err(|message| self.error(EngineFailureKind::Protocol, &message))?
                    {
                        if !response.success {
                            return Err(self.error(
                                EngineFailureKind::Command,
                                &format!("GTP {command} rejected: {}", response.body),
                            ));
                        }
                        acknowledged = Some(response.body);
                    }
                }
            }
            if written {
                if let Some(body) = acknowledged.take() {
                    return Ok(body);
                }
            }
        }
    }

    fn empty_command(&mut self, command: String) -> Result<(), EngineFailureDto> {
        if !self.gtp_command(command)?.trim().is_empty() {
            return Err(self.error(
                EngineFailureKind::Protocol,
                "GTP state command returned an unexpected body",
            ));
        }
        Ok(())
    }

    fn whole_seconds(&self) -> Result<u64, EngineFailureDto> {
        self.check()?;
        let seconds = self.deadline.saturating_duration_since(Instant::now()).as_secs();
        if seconds == 0 {
            return Err(self.error(
                EngineFailureKind::Timeout,
                "less than one positive GTP budget second remains",
            ));
        }
        Ok(seconds)
    }

    fn confirm_rules(&mut self) -> Result<app_model::OrdinaryRulesSnapshotDto, EngineFailureDto> {
        let restore = self.restore_file.take().expect("admitted restore file");
        self.empty_command(format!("loadsgf {}", restore.basename))?;
        restore
            .remove()
            .map_err(|message| self.error(EngineFailureKind::Protocol, &message))?;
        let rules = self.gtp_command("kata-get-rules".into())?;
        let komi = self.gtp_command("get_komi".into())?;
        let board = self.gtp_command("showboard".into())?;
        let history = self.gtp_command("printsgf".into())?;
        let confirmed_rules =
            super::ordinary_rules::verify(&self.request.position, &rules, &komi, &board, &history)
                .map_err(|message| self.error(EngineFailureKind::Protocol, &message))?;
        self.check()?;
        Ok(app_model::OrdinaryRulesSnapshotDto {
            identity: self.identity.clone(),
            reader_id: self.run.run_id.clone(),
            profile_revision: self
                .run
                .qualified_resource
                .as_ref()
                .expect("qualified Run")
                .profile_revision
                .clone(),
            position: self.request.position.dto().clone(),
            confirmed_rules: confirmed_rules.to_string(),
            stones: self.request.position.stones(),
            true_final_move: self.request.position.dto().moves.last().cloned(),
        })
    }

    fn compute(&mut self, plan: Vec<String>) -> Result<Option<GameMoveResultDto>, EngineFailureDto> {
        if self.mode == MoveMode::ConfirmRules {
            self.rules_snapshot = Some(self.confirm_rules()?);
            return Ok(None);
        }
        let mut mapped = false;
        let result = match self.run.adapter_kind {
            EngineBackend::KataGoAnalysis => {
                let mut query = katago_query(
                    self.request.position.dto(),
                    &self.identity.job_id,
                    self.request.identity.budget.max_visits.expect("admitted visits"),
                );
                if self.mode != MoveMode::Move {
                    query["reportDuringSearchEvery"] = serde_json::json!(0.05);
                    if analysis_capabilities(&self.run)?.ownership {
                        query["includeOwnership"] = serde_json::json!(true);
                    }
                }
                self.write(query.to_string())?;
                loop {
                    match self.receive()? {
                        MoveInput::Written(Ok(())) => {}
                        MoveInput::Written(Err(message)) => return Err(self.write_error(&message)),
                        MoveInput::Line(line) => {
                            let position = self.request.position.dto();
                            let result = match katago_result(
                                &line,
                                &self.identity.job_id,
                                position.moves.len(),
                                position.board_width,
                                position.board_height,
                            )
                            .map_err(|message| self.error(EngineFailureKind::Protocol, &message))?
                            {
                                KataGoMoveResponse::Searching => None,
                                KataGoMoveResponse::Complete(result) => Some(result),
                                KataGoMoveResponse::Rejected(message) => {
                                    self.query_rejected = true;
                                    return Err(self.error(EngineFailureKind::Protocol, &message));
                                }
                            };
                            if self.mode != MoveMode::Move {
                                // The strict move parser above fences id/turn and every warning/error
                                // before the common rich-analysis parser and normalizer are used.
                                let response =
                                    katago_protocol::parse_response_line(&line).map_err(|error| {
                                        self.error(EngineFailureKind::Protocol, &error.to_string())
                                    })?;
                                if !response
                                    .has_valid_search_result(position.board_width, position.board_height)
                                {
                                    return Err(self.error(EngineFailureKind::Protocol,
                                        "KataGo reserved analysis response has invalid search accounting or geometry"));
                                }
                                let frame = katago_protocol::normalize_response(
                                    self.job_uuid,
                                    response,
                                    position.board_width,
                                    position.board_height,
                                );
                                self.check()?;
                                if result.is_some() {
                                    self.final_frame = Some(frame);
                                } else {
                                    // Dropping a saturated intermediate sample cannot stall the deadline.
                                    let _ = self
                                        .frames
                                        .as_ref()
                                        .expect("enabled analysis stream")
                                        .try_send(frame);
                                }
                            }
                            if let Some(result) = result {
                                break result;
                            }
                        }
                    }
                }
            }
            EngineBackend::KataGoGtp => {
                return Err(self.error(
                    EngineFailureKind::UnsupportedCapability,
                    "KataGo GTP move computation is not admitted",
                ))
            }
            EngineBackend::GenericGtp => {
                for command in plan {
                    self.empty_command(command)?;
                }
                let facts = self
                    .run
                    .capability_snapshot
                    .as_ref()
                    .and_then(|caps| caps.gtp.as_ref())
                    .expect("admitted GTP facts");
                mapped = ["time_settings", "time_left"]
                    .iter()
                    .all(|required| facts.commands.iter().any(|command| command == required));
                if mapped {
                    self.empty_command(format!("time_settings 0 {} 1", self.whole_seconds()?))?;
                    self.empty_command(format!(
                        "time_left {} {} 1",
                        color_text(self.request.position.dto().to_play),
                        self.whole_seconds()?
                    ))?;
                }
                self.whole_seconds()?;
                let reply = self.gtp_command(format!(
                    "genmove {}",
                    color_text(self.request.position.dto().to_play)
                ))?;
                let position = self.request.position.dto();
                parse_gtp_move(&reply, position.board_width, position.board_height, true)
                    .map_err(|message| self.error(EngineFailureKind::Protocol, &message))?
            }
        };
        if let GameMoveDto::Move { vertex } = &result {
            self.request.position.validate_move(vertex).map_err(|message| {
                self.error(
                    EngineFailureKind::Protocol,
                    &format!("illegal engine move: {message}"),
                )
            })?;
        }
        self.check()?;
        Ok(Some(GameMoveResultDto {
            run_id: self.identity.run_id.clone(),
            job_id: self.identity.job_id.clone(),
            generation: self.identity.generation,
            node_path: self.identity.node_path.clone(),
            result,
            engine_time_mapped: mapped,
        }))
    }

    fn drain_katago(&self) -> bool {
        let payload =
            katago_protocol::terminate_action_jsonl(&Uuid::new_v4().to_string(), &self.identity.job_id);
        if self.writes.send(payload).is_err() {
            return false;
        }
        let deadline = Instant::now() + self.manager.inner.config.stop_drain_timeout;
        while Instant::now() < deadline {
            match self
                .events
                .recv_timeout(deadline.saturating_duration_since(Instant::now()))
            {
                Ok(MoveInput::Line(line)) => {
                    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&line) {
                        if value["id"].as_str() == Some(&self.identity.job_id)
                            && value["turnNumber"].as_u64()
                                == Some(self.request.position.dto().moves.len() as u64)
                            && value["isDuringSearch"].as_bool() == Some(false)
                        {
                            return true;
                        }
                    }
                }
                Ok(MoveInput::Written(Ok(()))) => {}
                _ => return false,
            }
        }
        false
    }

    fn finish(
        &mut self,
        mut result: Result<Option<GameMoveResultDto>, EngineFailureDto>,
    ) -> Result<Option<GameMoveResultDto>, EngineFailureDto> {
        // Seal publication under the same lock as cancellation before beginning any cleanup.
        {
            let mut state = self.manager.lock();
            if let Some(slot) = state
                .game_move
                .as_mut()
                .filter(|slot| slot.identity == self.identity)
            {
                if let Some(error) = &slot.sealed {
                    result = Err(error.clone());
                } else if Instant::now() >= self.deadline {
                    result = Err(self.error(EngineFailureKind::Timeout, "move hard deadline expired"));
                }
                if let Err(error) = &result {
                    slot.sealed = Some(error.clone());
                }
            } else {
                return Err(self.error(
                    EngineFailureKind::Cancellation,
                    "move identity retired before publication",
                ));
            }
            if result.is_ok() {
                if self.mode != MoveMode::AnalysisOnly {
                    state.game_move_publication = Some(self.identity.clone());
                }
                state.game_move = None;
                publish_snapshot(&mut state);
                drop(state);
                self.manager.reconcile_continuous();
                return result;
            }
        }
        let clean_katago = self.run.adapter_kind == EngineBackend::KataGoAnalysis
            && (!self.submitted || self.query_rejected || self.drain_katago());
        let mut state = self.manager.lock();
        // Retirement follows identity, never whichever side happens to be foreground.
        if !clean_katago {
            let deadline = Instant::now() + self.manager.inner.config.stop_drain_timeout;
            let cleanup = owned_live_mut(&mut state, &self.identity.run_id)
                .map(|live| terminate_process(live, deadline));
            let reaped = matches!(&cleanup, Some(Ok(())));
            if reaped {
                take_owned_live(&mut state, &self.identity.run_id);
            }
            let mut error = result.as_ref().expect_err("failed result").clone();
            error.profile_id = Some(self.run.profile_id.clone());
            if let Some(Err(cleanup)) = cleanup {
                error
                    .message
                    .push_str(&format!("; process cleanup failed: {cleanup}"));
                error.kind = EngineFailureKind::Timeout;
            } else if self.run.adapter_kind == EngineBackend::KataGoAnalysis
                && state
                    .match_reservation
                    .as_ref()
                    .is_some_and(|reservation| reservation.pausing)
            {
                error
                    .message
                    .push_str("; KataGo cancellation/drain was not acknowledged; the run cannot be resumed");
                error.kind = EngineFailureKind::Timeout;
            }
            result = Err(error.clone());
            if let Some(reservation) = state.match_reservation.as_mut() {
                if reservation
                    .runs
                    .iter()
                    .any(|run| run.run_id == self.identity.run_id)
                {
                    // Pause's own cancellation of a GTP move ends at a confirmed reap. The session
                    // survives; only an explicit Resume may rebuild this side. Anything else fails.
                    if reaped
                        && self.run.adapter_kind == EngineBackend::GenericGtp
                        && error.kind == EngineFailureKind::Cancellation
                        && reservation.pausing
                        && !reservation.sealed
                    {
                        reservation.retired.push(self.identity.run_id.clone());
                    } else {
                        reservation.failure.get_or_insert_with(|| error.clone());
                        reservation.sealed = true;
                    }
                }
            }
            if matches!(&state.phase, Phase::Ready(run) if run.run_id == self.identity.run_id) {
                state.phase = if error.kind == EngineFailureKind::Cancellation
                    && owned_live(&state, &self.identity.run_id).is_none()
                    && self.run.adapter_kind == EngineBackend::GenericGtp
                {
                    Phase::NoEngine { failure: None }
                } else {
                    Phase::Error {
                        run: self.run.clone(),
                        failure: error,
                    }
                };
            }
        } else if let Err(error) = &result {
            if error.kind != EngineFailureKind::Cancellation {
                if let Some(reservation) = state.match_reservation.as_mut() {
                    reservation.failure.get_or_insert_with(|| error.clone());
                    reservation.sealed = true;
                }
            }
        }
        if state
            .game_move
            .as_ref()
            .is_some_and(|slot| slot.identity == self.identity)
        {
            // A failed reap retains the exclusive slot until explicit lifecycle retirement.
            if clean_katago || owned_live(&state, &self.identity.run_id).is_none() {
                state.game_move = None;
            } else if let Some(slot) = state.game_move.as_mut() {
                slot.finished = true;
            }
        }
        publish_snapshot(&mut state);
        if let Err(error) = &result {
            publish_event(
                &mut state,
                ForegroundEngineEventDto::Failure {
                    failure: error.clone(),
                },
            );
        }
        drop(state);
        self.manager.reconcile_continuous();
        result
    }
}
