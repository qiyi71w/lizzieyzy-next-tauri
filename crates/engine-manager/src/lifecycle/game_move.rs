use super::*;
use crate::game_move_protocol::{color_text, gtp_sync_plan, katago_query, katago_result, parse_gtp_move};
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
    completion: Receiver<Result<GameMoveResultDto, EngineFailureDto>>,
}

impl GameMoveHandle {
    pub fn wait(self) -> Result<GameMoveResultDto, EngineFailureDto> {
        self.completion.recv().unwrap_or_else(|_| {
            Err(move_failure(
                &self.identity,
                EngineFailureKind::Protocol,
                "move worker disconnected",
            ))
        })
    }
}

pub(super) struct MoveSlot {
    pub(super) identity: GameMoveJobDto,
    events: SyncSender<MoveInput>,
    sealed: Option<EngineFailureDto>,
    adapter: EngineBackend,
}

enum MoveInput {
    Line(String),
    Written(Result<(), String>),
}

struct MoveWorker {
    manager: ForegroundEngineManager,
    request: GameMoveRequest,
    identity: GameMoveJobDto,
    run: EngineRunDto,
    deadline: Instant,
    events: Receiver<MoveInput>,
    writes: Sender<String>,
    submitted: bool,
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
            move_failure(
                &slot.identity,
                EngineFailureKind::Cancellation,
                "the foreground run was retired",
            )
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
        slot.sealed
            .get_or_insert_with(|| move_failure(&slot.identity, failure.kind, &failure.message));
    }
}

impl ForegroundEngineManager {
    pub fn start_game_move(&self, request: GameMoveRequest) -> Result<GameMoveHandle, EngineFailureDto> {
        let identity = GameMoveJobDto {
            run_id: request.identity.run_id.clone(),
            job_id: Uuid::new_v4().to_string(),
            generation: request.identity.generation,
            node_path: request.identity.node_path.clone(),
        };
        let fail = |kind, message: &str| move_failure(&identity, kind, message);
        let (events_tx, events) = mpsc::sync_channel(64);
        let (writes, writes_rx) = mpsc::channel::<String>();
        let (completed, completion) = mpsc::channel();
        let (run, plan, stdin, deadline) = {
            let mut state = self.lock();
            // Complete pure admission precedes any cancellation, process write or slot mutation.
            let run = match &state.phase {
                Phase::Ready(run) if run.run_id == identity.run_id => run.clone(),
                _ => {
                    return Err(fail(
                        EngineFailureKind::InvalidState,
                        "move requires the current Ready run",
                    ))
                }
            };
            if !run
                .capability_snapshot
                .as_ref()
                .is_some_and(|caps| caps.game_move)
            {
                return Err(fail(
                    EngineFailureKind::UnsupportedCapability,
                    "run does not admit game moves",
                ));
            }
            let budget = request.identity.budget;
            if budget.deadline_ms == 0 {
                return Err(fail(
                    EngineFailureKind::UnsupportedCapability,
                    "move deadline must be positive",
                ));
            }
            let plan = match run.adapter_kind {
                EngineBackend::KataGoAnalysis => {
                    let capabilities = analysis_capabilities(&run)?;
                    if !capabilities.visits_limit
                        || !capabilities.protocol_cancel
                        || budget.max_visits.is_none_or(|visits| visits == 0)
                        || request.position.dto().komi.abs() > 400.0
                    {
                        return Err(fail(EngineFailureKind::UnsupportedCapability,
                            "KataGo move requires positive max_visits, target cancellation and komi in [-400,400]"));
                    }
                    Vec::new()
                }
                EngineBackend::GenericGtp => {
                    if budget.max_visits.is_some() {
                        return Err(fail(
                            EngineFailureKind::UnsupportedCapability,
                            "GTP cannot enforce a visits budget",
                        ));
                    }
                    gtp_sync_plan(&run, request.position.dto())
                        .map_err(|message| fail(EngineFailureKind::UnsupportedCapability, &message))?
                }
            };
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
            state.game_move_publication = None;
            state.game_move = Some(MoveSlot {
                identity: identity.clone(),
                events: events_tx.clone(),
                sealed: None,
                adapter: run.adapter_kind,
            });
            publish_snapshot(&mut state);
            (run, plan, stdin, deadline)
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
                run,
                deadline,
                events,
                writes,
                submitted: false,
            };
            let result = worker.compute(plan);
            let result = worker.finish(result);
            // Drop the receiver before joining, so a saturated event channel cannot hold the writer.
            drop(worker);
            let _ = writer.join();
            let _ = completed.send(result);
        });
        Ok(GameMoveHandle { identity, completion })
    }

    pub fn cancel_game_move(&self, run_id: &str, job_id: &str) -> Result<(), EngineFailureDto> {
        let mut state = self.lock();
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
        let mut state = self.lock();
        if !state.game_move_publication.as_ref().is_some_and(|identity| {
            identity.run_id == result.run_id
                && identity.job_id == result.job_id
                && identity.generation == result.generation
                && identity.node_path == result.node_path
        }) || !matches!(&state.phase, Phase::Ready(run) if run.run_id == result.run_id)
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
        self.lock().game_move_publication = None;
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
        move_failure(&self.identity, kind, message)
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
        if !matches!(&state.phase, Phase::Ready(run) if run.run_id == self.identity.run_id) {
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
        loop {
            match self.receive()? {
                MoveInput::Written(Ok(())) => {}
                MoveInput::Written(Err(message)) => {
                    return Err(self.error(EngineFailureKind::Protocol, &message))
                }
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
                        return Ok(response.body);
                    }
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

    fn compute(&mut self, plan: Vec<String>) -> Result<GameMoveResultDto, EngineFailureDto> {
        let mut mapped = false;
        let result = match self.run.adapter_kind {
            EngineBackend::KataGoAnalysis => {
                let payload = katago_query(
                    self.request.position.dto(),
                    &self.identity.job_id,
                    self.request.identity.budget.max_visits.expect("admitted visits"),
                )
                .to_string();
                self.write(payload)?;
                loop {
                    match self.receive()? {
                        MoveInput::Written(Ok(())) => {}
                        MoveInput::Written(Err(message)) => {
                            return Err(self.error(EngineFailureKind::Protocol, &message))
                        }
                        MoveInput::Line(line) => {
                            let position = self.request.position.dto();
                            if let Some(result) = katago_result(
                                &line,
                                &self.identity.job_id,
                                position.moves.len(),
                                position.board_width,
                                position.board_height,
                            )
                            .map_err(|message| self.error(EngineFailureKind::Protocol, &message))?
                            {
                                break result;
                            }
                        }
                    }
                }
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
        Ok(GameMoveResultDto {
            run_id: self.identity.run_id.clone(),
            job_id: self.identity.job_id.clone(),
            generation: self.identity.generation,
            node_path: self.identity.node_path.clone(),
            result,
            engine_time_mapped: mapped,
        })
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
        mut result: Result<GameMoveResultDto, EngineFailureDto>,
    ) -> Result<GameMoveResultDto, EngineFailureDto> {
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
                state.game_move_publication = Some(self.identity.clone());
                state.game_move = None;
                publish_snapshot(&mut state);
                drop(state);
                self.manager.reconcile_continuous();
                return result;
            }
        }
        let clean_katago = self.run.adapter_kind == EngineBackend::KataGoAnalysis
            && (!self.submitted || self.drain_katago());
        let mut state = self.manager.lock();
        // Only this run can be retired. Stop/restart may already have reaped it and started another.
        if !clean_katago
            && state
                .live
                .as_ref()
                .is_some_and(|live| live.run_id == self.identity.run_id)
        {
            let deadline = Instant::now() + self.manager.inner.config.stop_drain_timeout;
            let cleanup = terminate_process(state.live.as_mut().expect("owned process"), deadline);
            if cleanup.is_ok() {
                state.live = None;
            }
            let mut error = result.as_ref().expect_err("failed result").clone();
            if let Err(cleanup) = cleanup {
                error
                    .message
                    .push_str(&format!("; process cleanup failed: {cleanup}"));
                error.kind = EngineFailureKind::Timeout;
                result = Err(error.clone());
            }
            if matches!(&state.phase, Phase::Ready(run) if run.run_id == self.identity.run_id) {
                state.phase = if error.kind == EngineFailureKind::Cancellation
                    && state.live.is_none()
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
        }
        if state
            .game_move
            .as_ref()
            .is_some_and(|slot| slot.identity == self.identity)
        {
            // A failed reap must retain the exclusive slot until explicit lifecycle retirement.
            if clean_katago
                || state
                    .live
                    .as_ref()
                    .is_none_or(|live| live.run_id != self.identity.run_id)
            {
                state.game_move = None;
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
