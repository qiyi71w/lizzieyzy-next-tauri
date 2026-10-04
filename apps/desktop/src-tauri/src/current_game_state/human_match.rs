use super::{CurrentGameHolder, CurrentGameState};
use crate::EngineCommandResult;
use app_model::*;
use engine_manager::{ForegroundEngineManager, GameMoveHandle, GameMoveRequest};
use match_core::HumanMatch;
use sgf::CurrentSgfDocument;
use std::path::Path;

mod analysis;

#[derive(Default)]
pub(super) struct MatchState {
    pub snapshot: MatchSnapshotDto,
    turns: Option<HumanMatch>,
    budget: Option<ComputeBudgetDto>,
    pk_budgets: Option<[ComputeBudgetDto; 2]>,
    analysis_work: Option<analysis::AnalysisWork>,
    analysis_completed: Option<(u64, u64)>,
    analysis_transition: bool,
}

impl MatchState {
    pub(super) fn blocks(&self) -> bool {
        self.snapshot.resources_held
            || matches!(
                self.snapshot.phase,
                MatchPhaseDto::Starting
                    | MatchPhaseDto::Playing
                    | MatchPhaseDto::Paused
                    | MatchPhaseDto::Ending
            )
    }
    fn changed(&mut self) {
        self.snapshot.revision += 1;
    }
    fn owns(&self, id: &str) -> bool {
        self.snapshot.session_id.as_deref() == Some(id)
    }
    pub(super) fn seal(&mut self, reason: MatchEndDto, failure: Option<EngineFailureDto>) {
        self.snapshot.analysis.epoch += 1;
        self.snapshot.analysis.frame = None;
        self.analysis_work = None;
        if let Some(turns) = self.turns.as_mut() {
            turns.seal(reason);
        }
        self.snapshot.phase = MatchPhaseDto::Ending;
        self.snapshot.end = Some(reason);
        self.snapshot.pause_pending = false;
        self.snapshot.resume_pending = false;
        self.snapshot.rebuild_sides.clear();
        self.record_failure(failure);
        self.snapshot.job = None;
        self.changed();
    }
    fn record_failure(&mut self, failure: Option<EngineFailureDto>) {
        let side = failure
            .as_ref()
            .and_then(|failure| {
                self.snapshot.pk_runs.as_ref().and_then(|runs| {
                    runs.iter()
                        .position(|run| Some(&run.run_id) == failure.run_id.as_ref())
                })
            })
            .map(|index| {
                if index == 0 {
                    PlayerColor::Black
                } else {
                    PlayerColor::White
                }
            })
            .or(self.snapshot.failed_side)
            .or(self.snapshot.to_play);
        self.snapshot.failed_side = failure.as_ref().and(side);
        self.snapshot.failure = failure.map(|mut failure| {
            let run = self
                .snapshot
                .pk_runs
                .as_ref()
                .map(|runs| &runs[usize::from(side == Some(PlayerColor::White))]);
            if failure.profile_id.is_none() {
                failure.profile_id = run.map(|run| run.profile_id.clone()).or_else(|| {
                    self.snapshot
                        .settings
                        .as_ref()
                        .and_then(|settings| settings.profile_id.clone())
                });
            }
            if failure.run_id.is_none() {
                failure.run_id = run
                    .map(|run| run.run_id.clone())
                    .or_else(|| self.snapshot.run_id.clone());
            }
            if failure.job_id.is_none() {
                failure.job_id = self.snapshot.job.as_ref().map(|job| job.job_id.clone());
            }
            failure
        });
    }
}

pub(crate) fn match_failure(kind: EngineFailureKind, message: impl Into<String>) -> EngineFailureDto {
    EngineFailureDto {
        operation: EngineOperationDto::Job,
        run_id: None,
        switch_id: None,
        job_id: None,
        profile_id: None,
        kind,
        message: message.into(),
        diagnostic_summary: None,
    }
}
fn invalid(message: impl Into<String>) -> Box<EngineFailureDto> {
    Box::new(match_failure(EngineFailureKind::InvalidState, message))
}

impl CurrentGameHolder {
    fn match_update(&self) -> MatchUpdateDto {
        MatchUpdateDto {
            match_state: self.human_match.snapshot.clone(),
            current: self.current_result().ok(),
        }
    }
    fn admit_match_turn(&self, token: &MatchTurnDto, human: bool) -> EngineCommandResult<()> {
        if self.edits_blocked || (human && self.departure.is_some()) {
            return Err(invalid("This match turn is no longer current."));
        }
        self.admit_match_identity(token, human)
    }
    // Session-only analysis may finish during a departure; it cannot mutate the protected document.
    fn admit_match_identity(&self, token: &MatchTurnDto, human: bool) -> EngineCommandResult<()> {
        if self.human_match.analysis_transition
            || !self.human_match.owns(&token.session_id)
            || self.human_match.snapshot.phase != MatchPhaseDto::Playing
            || self.generation != token.generation
            || self.selected_path != token.node_path
            || !self
                .human_match
                .turns
                .as_ref()
                .is_some_and(|turns| turns.admits(token.turn, human))
        {
            return Err(invalid("This match turn is no longer current."));
        }
        Ok(())
    }
    fn apply_match_move(&mut self, action: GameMoveDto) -> EngineCommandResult<()> {
        let document = self
            .document
            .as_ref()
            .ok_or_else(|| invalid("Match document is unavailable."))?;
        let turns = self
            .human_match
            .turns
            .as_ref()
            .expect("playing has turn authority");
        let (prepared, pass, resign) = match action {
            GameMoveDto::Move { vertex } => {
                let pass = vertex == MoveVertex::Pass;
                (document.prepare_play(&self.selected_path, vertex), pass, false)
            }
            GameMoveDto::Resign => (
                document.prepare_result(&self.selected_path, turns.resignation_result()),
                false,
                true,
            ),
        };
        let prepared =
            prepared.map_err(|error| match_failure(EngineFailureKind::Command, error.to_string()))?;
        let outcome = self
            .document
            .as_mut()
            .expect("validated document")
            .apply_prepared(prepared);
        let selected = outcome.snapshot.path.clone();
        if let Some(edit) = outcome.edit {
            self.commit_edit(edit);
        }
        self.selected_path = selected;
        self.generation += 1;
        self.human_match.snapshot.analysis.epoch += 1;
        self.human_match.snapshot.analysis.frame = None;
        self.human_match.analysis_work = None;
        self.bump_snapshot();
        let turns = self
            .human_match
            .turns
            .as_mut()
            .expect("playing has turn authority");
        if resign {
            turns.seal(MatchEndDto::Resigned);
        } else {
            turns.played(pass);
        }
        self.human_match.snapshot.turn = turns.turn();
        self.human_match.snapshot.to_play = Some(turns.to_play());
        self.human_match.snapshot.committed_moves = turns.committed_moves();
        if let Some(reason) = turns.end() {
            self.human_match.seal(reason, None);
        } else {
            self.human_match.changed();
        }
        Ok(())
    }
}

impl CurrentGameState {
    pub fn human_match_snapshot(&self) -> MatchUpdateDto {
        self.holder.lock().expect("current game state").match_update()
    }

    /// Starts a New game or an exact Continue as one transaction: nothing in the document,
    /// history, selection or saved defaults changes unless engine preparation and the defaults
    /// write both succeed.
    pub fn start_human_match(
        &self,
        manager: &ForegroundEngineManager,
        preferences: &crate::continuous_analysis::PreferencesState,
        preferences_path: &Path,
        request: HumanMatchStartDto,
        profile: EngineProfileDto,
        publish: impl Fn(MatchUpdateDto),
    ) -> EngineCommandResult<MatchUpdateDto> {
        self.start_match(
            manager,
            preferences,
            preferences_path,
            request,
            (profile, None),
            publish,
        )
    }

    pub fn start_pk_match(
        &self,
        manager: &ForegroundEngineManager,
        preferences: &crate::continuous_analysis::PreferencesState,
        preferences_path: &Path,
        request: HumanMatchStartDto,
        profiles: [EngineProfileDto; 2],
        publish: impl Fn(MatchUpdateDto),
    ) -> EngineCommandResult<MatchUpdateDto> {
        let [black, white] = profiles;
        self.start_match(
            manager,
            preferences,
            preferences_path,
            request,
            (black, Some(white)),
            publish,
        )
    }

    fn start_match(
        &self,
        manager: &ForegroundEngineManager,
        preferences: &crate::continuous_analysis::PreferencesState,
        preferences_path: &Path,
        request: HumanMatchStartDto,
        profiles: (EngineProfileDto, Option<EngineProfileDto>),
        publish: impl Fn(MatchUpdateDto),
    ) -> EngineCommandResult<MatchUpdateDto> {
        let (profile, opponent) = profiles;
        let mode = if opponent.is_some() {
            MatchModeDto::Pk
        } else {
            MatchModeDto::Human
        };
        let settings = &request.settings;
        let unsupported = |error: String| match_failure(EngineFailureKind::UnsupportedCapability, error);
        match &request.start {
            MatchStartDto::New { .. } => {
                settings.validate().map_err(invalid)?;
                if settings.rules.is_none() {
                    return Err(invalid("Choose exact rules before starting a new game."));
                }
            }
            MatchStartDto::Continue {
                root_metadata_confirmed,
                ..
            } => {
                settings.validate_participants().map_err(invalid)?;
                if !root_metadata_confirmed {
                    return Err(invalid("Confirm that continuing replaces the whole document's players and clears its result."));
                }
            }
        }
        let names = if let Some(opponent) = &opponent {
            (profile.name.as_str(), opponent.name.as_str())
        } else if settings.human_color == PlayerColor::Black {
            ("Human", profile.name.as_str())
        } else {
            (profile.name.as_str(), "Human")
        };
        // A New game is staged before taking the lock; a Continue must stage from the locked current document.
        let new_game = match request.start {
            MatchStartDto::New { .. } => Some(
                CurrentSgfDocument::stage_new_game(
                    settings.board_size,
                    settings.komi,
                    settings.handicap,
                    settings.rules.expect("validated rules"),
                    names.0,
                    names.1,
                )
                .map_err(|error| unsupported(error.to_string()))?,
            ),
            MatchStartDto::Continue { .. } => None,
        };
        let empty = CurrentSgfDocument::open("(;GM[1]FF[4]SZ[19]KM[7.5])")
            .map_err(|error| invalid(error.to_string()))?;
        let budget = ComputeBudgetDto {
            deadline_ms: settings.deadline_ms,
            max_visits: (profile.adapter_kind() == EngineBackend::KataGoAnalysis)
                .then_some(settings.kata_max_visits),
        };
        // GenericGtp cannot enforce visits; its side is budgeted by the hard deadline alone.
        let side_budget = |side: &PkSideSettingsDto, profile: &EngineProfileDto| ComputeBudgetDto {
            deadline_ms: side.deadline_ms,
            max_visits: (profile.adapter_kind() == EngineBackend::KataGoAnalysis)
                .then_some(side.kata_max_visits),
        };
        let pk_budgets = opponent.as_ref().map(|white| {
            [
                side_budget(&settings.pk_black, &profile),
                side_budget(&settings.pk_white, white),
            ]
        });
        let (id, baseline, staged, endpoint, position, turns) = {
            let mut holder = self.holder.lock().expect("current game state");
            holder
                .ensure_editable()
                .map_err(|error| invalid(error.to_string()))?;
            // Accepted analysis advances snapshot_seq without moving a position, so Continue is pinned by
            // generation plus the exact NodePath below; New keeps the snapshot check behind its discard confirmation.
            let continuing = matches!(request.start, MatchStartDto::Continue { .. });
            if holder.generation != request.generation
                || (!continuing && holder.snapshot_seq != request.snapshot_seq)
            {
                return Err(invalid(
                    "Document changed while choosing match settings; start again.",
                ));
            }
            let (staged, endpoint, position) = match (&request.start, new_game) {
                (MatchStartDto::New { discard_confirmed }, Some(staged)) => {
                    if holder.dirty && !discard_confirmed {
                        return Err(invalid(
                            "Save, discard or cancel the current document before starting.",
                        ));
                    }
                    let position = staged.exact_position(&NodePath::default()).map_err(unsupported)?;
                    (staged.into(), NodePath::default(), position)
                }
                (MatchStartDto::Continue { node_path, .. }, None) => {
                    if *node_path != holder.selected_path {
                        return Err(invalid(
                            "The selected node changed while choosing match settings; start again.",
                        ));
                    }
                    holder
                        .document
                        .as_ref()
                        .ok_or_else(|| invalid("Open a document before continuing it."))?
                        .stage_continuation(node_path, names.0, names.1)
                        .map_err(unsupported)?
                }
                _ => unreachable!("New games are staged exactly when the request is New"),
            };
            let turns = if mode == MatchModeDto::Pk {
                HumanMatch::new_pk(settings, position.dto())
            } else {
                HumanMatch::new(settings, position.dto())
            }
            .map_err(invalid)?;
            holder.next_match_id += 1;
            let id = format!(
                "{}-{}",
                if mode == MatchModeDto::Pk { "pk" } else { "human" },
                holder.next_match_id
            );
            manager.reserve_match(&id)?;
            let revision = holder.human_match.snapshot.revision + 1;
            holder.human_match = MatchState {
                snapshot: MatchSnapshotDto {
                    revision,
                    mode,
                    phase: MatchPhaseDto::Starting,
                    session_id: Some(id.clone()),
                    settings: Some(settings.clone()),
                    resources_held: true,
                    ..MatchSnapshotDto::default()
                },
                ..MatchState::default()
            };
            let baseline = (
                holder.document_identity,
                holder.generation,
                holder.snapshot_seq,
                holder.content_version(),
                holder.selected_path.clone(),
            );
            (id, baseline, staged, endpoint, position, turns)
        };
        publish(self.human_match_snapshot());
        let prepared = if let Some(budgets) = pk_budgets {
            manager
                .prepare_reserved_pk(
                    &id,
                    [
                        (
                            settings
                                .pk_black
                                .profile_id
                                .as_deref()
                                .expect("validated black profile"),
                            budgets[0],
                        ),
                        (
                            settings
                                .pk_white
                                .profile_id
                                .as_deref()
                                .expect("validated white profile"),
                            budgets[1],
                        ),
                    ],
                    &position,
                )
                .map(|runs| {
                    (
                        runs[usize::from(position.dto().to_play == PlayerColor::White)].clone(),
                        Some(runs),
                    )
                })
                .map_err(|(side, error)| {
                    let mut holder = self.holder.lock().expect("current game state");
                    if holder.human_match.owns(&id) {
                        holder.human_match.snapshot.failed_side = Some(side);
                    }
                    error
                })
        } else {
            manager
                .prepare_reserved_match(
                    &id,
                    settings.profile_id.as_deref().expect("validated profile"),
                    &position,
                    budget,
                )
                .map(|run| (run, None))
        };
        let result = prepared.map_err(Box::new).and_then(|(run, pk_runs)| {
            // Complete fallible projection before durable defaults can change.
            let tree = staged
                .document()
                .tree()
                .map_err(|error| invalid(error.to_string()))?;
            let snapshot = staged
                .document()
                .snapshot(&endpoint)
                .map_err(|error| invalid(error.to_string()))?;
            let mut holder = self.holder.lock().expect("current game state");
            if holder.human_match.owns(&id)
                && holder.human_match.snapshot.phase == MatchPhaseDto::Starting
                && holder.departure.is_some()
            {
                // Departure refuses this prepared candidate, rather than failing the engine.
                holder.human_match.seal(MatchEndDto::Stopped, None);
            }
            if !holder.human_match.owns(&id)
                || holder.human_match.snapshot.phase != MatchPhaseDto::Starting
                || holder.departure.is_some()
                || baseline
                    != (
                        holder.document_identity,
                        holder.generation,
                        holder.snapshot_seq,
                        holder.content_version(),
                        holder.selected_path.clone(),
                    )
            {
                return Err(Box::new(match_failure(
                    EngineFailureKind::Cancellation,
                    "Match start was cancelled or its document changed.",
                )));
            }
            let profiles_match = if let Some(runs) = &pk_runs {
                runs[0].profile_snapshot == profile && Some(&runs[1].profile_snapshot) == opponent.as_ref()
            } else {
                run.profile_snapshot == profile
            };
            if !profiles_match {
                return Err(invalid(
                    "Saved engine profile changed while preparing the match; start again.",
                ));
            }
            holder.human_match.snapshot.pk_runs = pk_runs.clone();
            let new_document = matches!(request.start, MatchStartDto::New { .. });
            let defaults = |saved: &MatchDefaultsDto| {
                if new_document {
                    settings.clone()
                } else {
                    saved.with_participants_of(settings)
                }
            };
            let current = preferences.commit_match(preferences_path, manager, &id, defaults, |saved| {
                let old_path = holder.selected_path.clone();
                let document = holder.document.get_or_insert(empty);
                let edit = document.replace_with_history(staged, &old_path, endpoint.clone());
                holder.commit_edit(edit);
                holder.selected_path = endpoint.clone();
                if new_document {
                    holder.native_path = None;
                    holder.document_seq += 1;
                    holder.document_identity = holder.document_seq;
                }
                holder.generation += 1;
                holder.bump_snapshot();
                holder.analysis_target = None;
                holder.human_match.turns = Some(turns);
                holder.human_match.snapshot.committed = true;
                holder.human_match.snapshot.settings = Some(saved.clone());
                holder.human_match.budget = Some(budget);
                holder.human_match.pk_budgets = pk_budgets;
                holder.human_match.snapshot.pk_runs = pk_runs;
                holder.human_match.snapshot.phase = MatchPhaseDto::Playing;
                holder.human_match.snapshot.analysis.supported = mode == MatchModeDto::Human
                    && run.adapter_kind == EngineBackend::KataGoAnalysis
                    && run
                        .capability_snapshot
                        .as_ref()
                        .and_then(|caps| caps.analysis.as_ref())
                        .is_some_and(|caps| {
                            caps.selected_node_analysis
                                && caps.candidates
                                && caps.visits_limit
                                && caps.protocol_cancel
                        });
                holder.human_match.snapshot.run_id = Some(run.run_id);
                holder.human_match.snapshot.turn = 1;
                holder.human_match.snapshot.to_play = Some(position.dto().to_play);
                holder.human_match.changed();
                CurrentGameResultDto {
                    tree,
                    selected_path: endpoint,
                    snapshot,
                    generation: holder.generation,
                    snapshot_seq: holder.snapshot_seq,
                    dirty: holder.dirty,
                    can_undo: true,
                    can_redo: false,
                    native_path: holder.native_path.clone(),
                }
            })?;
            self.note_recovery(&holder);
            Ok(MatchUpdateDto {
                match_state: holder.human_match.snapshot.clone(),
                current: Some(current),
            })
        });
        match result {
            Ok(update) => Ok(update),
            Err(error) => {
                let cleanup = manager.abort_reserved_match(&id);
                let mut holder = self.holder.lock().expect("current game state");
                if holder.human_match.owns(&id) && !holder.human_match.snapshot.committed {
                    let cleanup_failed = cleanup.is_err();
                    let cleanup_error = cleanup
                        .err()
                        .or_else(|| holder.human_match.snapshot.failure.clone());
                    let intentionally_stopped = holder.human_match.snapshot.end == Some(MatchEndDto::Stopped);

                    if intentionally_stopped && cleanup_error.is_none() {
                        holder.human_match.snapshot.phase = MatchPhaseDto::Idle;
                        holder.human_match.snapshot.resources_held = false;
                        holder.human_match.snapshot.failure = None;
                        holder.human_match.snapshot.failed_side = None;
                        holder.human_match.changed();
                        let update = holder.match_update();
                        drop(holder);
                        publish(update.clone());
                        return Ok(update);
                    }

                    let failure = cleanup_error.map(Box::new).unwrap_or(error);
                    holder.human_match.snapshot.phase = MatchPhaseDto::Error;
                    holder.human_match.snapshot.resources_held = cleanup_failed;
                    holder.human_match.record_failure(Some((*failure).clone()));
                    holder.human_match.changed();
                    let update = holder.match_update();
                    drop(holder);
                    publish(update);
                    return Err(failure);
                }
                let update = holder.match_update();
                drop(holder);
                publish(update);
                Err(error)
            }
        }
    }

    pub fn human_match_action(
        &self,
        manager: &ForegroundEngineManager,
        turn: MatchTurnDto,
        action: HumanMatchActionDto,
    ) -> EngineCommandResult<MatchUpdateDto> {
        let mut holder = self.drain_match_analysis(manager, &turn, true)?;
        holder.admit_match_turn(&turn, true)?;
        let action = match action {
            HumanMatchActionDto::Play { vertex } => GameMoveDto::Move { vertex },
            HumanMatchActionDto::Resign => GameMoveDto::Resign,
        };
        holder.apply_match_move(action)?;
        self.note_recovery(&holder);
        Ok(holder.match_update())
    }

    #[cfg(test)]
    pub fn take_match_engine_turn(
        &self,
        manager: &ForegroundEngineManager,
    ) -> EngineCommandResult<Option<(MatchTurnDto, GameMoveHandle)>> {
        self.take_match_work(manager)
            .map(|work| work.map(|work| (work.turn, work.handle)))
    }

    pub fn accept_match_engine_result(
        &self,
        manager: &ForegroundEngineManager,
        token: &MatchTurnDto,
        result: Result<GameMoveResultDto, EngineFailureDto>,
    ) -> MatchUpdateDto {
        let mut holder = self.holder.lock().expect("current game state");
        while holder.edits_blocked && holder.human_match.snapshot.phase == MatchPhaseDto::Playing {
            holder = self.match_departure.wait(holder).expect("match departure gate");
        }
        if holder.admit_match_turn(token, false).is_err() {
            return holder.match_update();
        }
        let result = result.map_err(Box::new).and_then(|result| {
            if !holder.human_match.snapshot.job.as_ref().is_some_and(|job| {
                job.run_id == result.run_id
                    && job.job_id == result.job_id
                    && job.generation == result.generation
                    && job.node_path == result.node_path
            }) {
                return Err(invalid("Engine move does not match the reserved turn."));
            }
            manager.claim_reserved_game_move_result(&token.session_id, &result)?;
            holder.apply_match_move(result.result)
        });
        holder.human_match.snapshot.job = None;
        if let Err(error) = result {
            holder.human_match.seal(MatchEndDto::Failed, Some(*error));
        }
        self.note_recovery(&holder);
        holder.match_update()
    }

    /// The document lock seals publication before cancellation touches the process. Paused remains
    /// exclusive, but is not resumable until the manager confirms KataGo drain or GTP kill/reap.
    pub fn pause_pk_match(
        &self,
        manager: &ForegroundEngineManager,
        id: &str,
        publish: impl Fn(MatchUpdateDto),
    ) -> EngineCommandResult<MatchUpdateDto> {
        {
            let mut holder = self.holder.lock().expect("current game state");
            if !holder.human_match.owns(id) || holder.human_match.snapshot.mode != MatchModeDto::Pk {
                return Err(invalid("PK session is no longer current."));
            }
            if holder.human_match.snapshot.phase == MatchPhaseDto::Paused {
                return Ok(holder.match_update());
            }
            if holder.human_match.snapshot.phase != MatchPhaseDto::Playing {
                return Err(invalid("Only a playing PK can pause."));
            }
            let turns = holder.human_match.turns.as_mut().expect("playing authority");
            turns.invalidate_turn();
            holder.human_match.snapshot.turn = turns.turn();
            holder.human_match.snapshot.phase = MatchPhaseDto::Paused;
            holder.human_match.snapshot.pause_pending = true;
            holder.human_match.snapshot.job = None;
            holder.human_match.changed();
            self.match_departure.notify_all();
            let update = holder.match_update();
            drop(holder);
            publish(update);
        }
        let cleanup = manager.pause_reserved_match(id);
        let mut holder = self.holder.lock().expect("current game state");
        if holder.human_match.owns(id) && holder.human_match.snapshot.phase == MatchPhaseDto::Paused {
            holder.human_match.snapshot.pause_pending = false;
            match cleanup {
                Err(error) => holder.human_match.seal(MatchEndDto::Failed, Some(error)),
                Ok(retired) => {
                    let runs = holder.human_match.snapshot.pk_runs.clone().expect("PK runs");
                    holder.human_match.snapshot.rebuild_sides = [PlayerColor::Black, PlayerColor::White]
                        .into_iter()
                        .zip(runs.iter())
                        .filter(|(_, run)| retired.contains(&run.run_id))
                        .map(|(side, _)| side)
                        .collect();
                    holder.human_match.changed();
                }
            }
        }
        Ok(holder.match_update())
    }

    /// Explicit Resume continues the same session, branch, side and count. Sides whose GTP run Pause
    /// reaped are rebuilt from the session's immutable snapshots; repeated Resume joins the one attempt.
    /// Stop or exit during reconstruction wins: a late result never returns the session to Playing.
    pub fn resume_pk_match(
        &self,
        manager: &ForegroundEngineManager,
        id: &str,
        publish: impl Fn(MatchUpdateDto),
    ) -> EngineCommandResult<MatchUpdateDto> {
        let (position, budgets) = {
            let mut holder = self.holder.lock().expect("current game state");
            if !holder.human_match.owns(id) || holder.human_match.snapshot.mode != MatchModeDto::Pk {
                return Err(invalid("PK session is no longer current."));
            }
            if holder.human_match.snapshot.phase == MatchPhaseDto::Playing
                || holder.human_match.snapshot.resume_pending
            {
                return Ok(holder.match_update());
            }
            if holder.edits_blocked
                || holder.departure.is_some()
                || holder.human_match.snapshot.phase != MatchPhaseDto::Paused
                || holder.human_match.snapshot.pause_pending
                || !holder.human_match.snapshot.resources_held
            {
                return Err(invalid(
                    "PK cannot resume until pause cleanup and document departure have finished.",
                ));
            }
            if holder.human_match.snapshot.rebuild_sides.is_empty() {
                holder.human_match.snapshot.phase = MatchPhaseDto::Playing;
                holder.human_match.changed();
                return Ok(holder.match_update());
            }
            let position = holder
                .document
                .as_ref()
                .expect("paused document")
                .exact_position(&holder.selected_path)
                .map_err(|error| match_failure(EngineFailureKind::UnsupportedCapability, error))?;
            let budgets = holder.human_match.pk_budgets.expect("PK budgets");
            holder.human_match.snapshot.resume_pending = true;
            holder.human_match.changed();
            let update = holder.match_update();
            drop(holder);
            publish(update);
            (position, budgets)
        };
        let rebuilt = manager.resume_reserved_pk(id, &position, budgets);
        let mut holder = self.holder.lock().expect("current game state");
        if !holder.human_match.owns(id)
            || holder.human_match.snapshot.phase != MatchPhaseDto::Paused
            || !holder.human_match.snapshot.resume_pending
        {
            return Ok(holder.match_update());
        }
        holder.human_match.snapshot.resume_pending = false;
        match rebuilt {
            Ok(runs) => {
                holder.human_match.snapshot.pk_runs = Some(runs);
                holder.human_match.snapshot.rebuild_sides.clear();
                holder.human_match.snapshot.phase = MatchPhaseDto::Playing;
                holder.human_match.changed();
            }
            Err((side, error)) => {
                holder.human_match.snapshot.failed_side = Some(side);
                holder.human_match.seal(MatchEndDto::Failed, Some(error));
            }
        }
        Ok(holder.match_update())
    }

    pub fn stop_human_match(
        &self,
        manager: &ForegroundEngineManager,
        id: &str,
    ) -> EngineCommandResult<MatchUpdateDto> {
        let committed = {
            let mut holder = self.holder.lock().expect("current game state");
            if !holder.human_match.owns(id) {
                return Err(invalid("Match session is no longer current."));
            }
            if !holder.human_match.snapshot.resources_held {
                return Ok(holder.match_update());
            }
            if holder.human_match.snapshot.phase != MatchPhaseDto::Ending {
                holder.human_match.seal(MatchEndDto::Stopped, None);
            }
            self.match_departure.notify_all();
            holder.human_match.snapshot.committed
        };
        let cleanup = if committed {
            manager.stop_reserved_match(id)
        } else {
            manager.abort_reserved_match(id)
        };
        let mut holder = self.holder.lock().expect("current game state");
        if holder.human_match.owns(id) {
            holder.human_match.snapshot.resources_held = cleanup.is_err();
            if let Err(error) = cleanup {
                holder.human_match.record_failure(Some(error));
            }
            holder.human_match.snapshot.phase = if holder.human_match.snapshot.failure.is_some() {
                MatchPhaseDto::Error
            } else {
                MatchPhaseDto::Idle
            };
            if !committed && holder.human_match.snapshot.failure.is_none() {
                holder.human_match.snapshot.failed_side = None;
            }
            holder.human_match.changed();
        }
        Ok(holder.match_update())
    }
}

impl CurrentGameState {
    pub fn observe_match_failure(&self, failure: &EngineFailureDto) -> Option<MatchUpdateDto> {
        let mut holder = self.holder.lock().expect("current game state");
        let snapshot = &holder.human_match.snapshot;
        let owns_run = failure.run_id.is_some()
            && (failure.run_id == snapshot.run_id
                || snapshot.pk_runs.as_ref().is_some_and(|runs| {
                    runs.iter()
                        .any(|run| Some(&run.run_id) == failure.run_id.as_ref())
                }));
        if !matches!(snapshot.phase, MatchPhaseDto::Playing | MatchPhaseDto::Paused)
            || !owns_run
            || !matches!(failure.operation, EngineOperationDto::UnexpectedExit)
        {
            return None;
        }
        holder
            .human_match
            .seal(MatchEndDto::Failed, Some(failure.clone()));
        Some(holder.match_update())
    }
}

#[cfg(all(test, unix))]
mod tests;
