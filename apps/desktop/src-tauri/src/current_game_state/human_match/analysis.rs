use super::*;
use std::sync::MutexGuard;

#[derive(Clone)]
pub(super) struct AnalysisWork {
    turn: MatchTurnDto,
    epoch: u64,
    job: GameMoveJobDto,
    human: bool,
}

pub(crate) struct MatchWork {
    pub turn: MatchTurnDto,
    pub epoch: u64,
    pub human: bool,
    pub handle: GameMoveHandle,
}

impl CurrentGameHolder {
    fn analysis_update(&self) -> MatchUpdateDto {
        MatchUpdateDto { match_state: self.human_match.snapshot.clone(), current: None }
    }
}

impl CurrentGameState {
    /// Seal presentation before draining; the document lock is never held over process IO.
    pub(super) fn drain_match_analysis<'a>(
        &'a self, manager: &ForegroundEngineManager, turn: &MatchTurnDto, human: bool,
    ) -> Result<MutexGuard<'a, CurrentGameHolder>, EngineFailureDto> {
        let mut holder = self.holder.lock().expect("current game state");
        holder.admit_match_turn(turn, human)?;
        let Some(work) = holder.human_match.analysis_work.as_ref().filter(|work| work.human).cloned() else {
            return Ok(holder);
        };
        holder.human_match.analysis_transition = true;
        holder.human_match.snapshot.analysis.epoch += 1;
        holder.human_match.snapshot.analysis.frame = None;
        holder.human_match.analysis_work = None;
        holder.human_match.changed();
        drop(holder);
        let drained = manager.cancel_reserved_analysis(&turn.session_id, &work.job);
        let mut holder = self.holder.lock().expect("current game state");
        if !holder.human_match.owns(&turn.session_id) {
            return Err(invalid("The analysis session was replaced during cleanup."));
        }
        holder.human_match.analysis_transition = false;
        if holder.human_match.snapshot.job.as_ref() == Some(&work.job) {
            holder.human_match.snapshot.job = None;
            holder.human_match.changed();
        }
        if let Err(error) = drained {
            if holder.human_match.snapshot.phase == MatchPhaseDto::Playing {
                holder.human_match.seal(MatchEndDto::Failed, Some(error.clone()));
            }
            return Err(error);
        }
        holder.admit_match_turn(turn, human)?;
        Ok(holder)
    }

    pub fn human_match_analysis_policy(
        &self, manager: &ForegroundEngineManager, turn: MatchTurnDto, policy: MatchAnalysisPolicyDto,
    ) -> Result<MatchUpdateDto, EngineFailureDto> {
        let human = {
            let holder = self.holder.lock().expect("current game state");
            if holder.human_match.snapshot.mode != MatchModeDto::Human {
                return Err(invalid("Session analysis is only available in a Human match."));
            }
            let human = holder.human_match.turns.as_ref().is_some_and(HumanMatch::human_turn);
            holder.admit_match_turn(&turn, human)?;
            if policy != MatchAnalysisPolicyDto::Off && !holder.human_match.snapshot.analysis.supported {
                return Err(match_failure(EngineFailureKind::UnsupportedCapability,
                    "This verified engine does not provide Human session candidate analysis."));
            }
            if holder.human_match.snapshot.analysis.policy == policy { return Ok(holder.analysis_update()); }
            human
        };
        let mut holder = self.drain_match_analysis(manager, &turn, human)?;
        holder.human_match.snapshot.analysis.epoch += 1;
        holder.human_match.snapshot.analysis.frame = None;
        holder.human_match.snapshot.analysis.policy = policy;
        holder.human_match.changed();
        Ok(holder.analysis_update())
    }

    pub(crate) fn take_match_work(&self, manager: &ForegroundEngineManager) -> Result<Option<MatchWork>, EngineFailureDto> {
        let mut holder = self.holder.lock().expect("current game state");
        while holder.edits_blocked && holder.human_match.snapshot.phase == MatchPhaseDto::Playing {
            holder = self.match_departure.wait(holder).expect("match departure gate");
        }
        if holder.human_match.snapshot.phase != MatchPhaseDto::Playing
            || holder.human_match.analysis_transition || holder.human_match.snapshot.job.is_some() {
            return Ok(None);
        }
        let human = holder.human_match.turns.as_ref().expect("playing turns").human_turn();
        let snapshot = &holder.human_match.snapshot;
        let enabled = snapshot.analysis.supported && snapshot.analysis.policy.includes(human);
        let epoch = snapshot.analysis.epoch;
        if human && (!enabled || holder.human_match.analysis_completed == Some((snapshot.turn, epoch))) {
            return Ok(None);
        }
        let turn = MatchTurnDto { session_id: snapshot.session_id.clone().expect("playing session"),
            turn: snapshot.turn, generation: holder.generation, node_path: holder.selected_path.clone() };
        let position = holder.document.as_ref().expect("playing document").exact_position(&turn.node_path)
            .map_err(|error| match_failure(EngineFailureKind::UnsupportedCapability, error))?;
        let (run_id, budget) = if let Some(runs) = &snapshot.pk_runs {
            let side = usize::from(snapshot.to_play == Some(PlayerColor::White));
            (runs[side].run_id.clone(), holder.human_match.pk_budgets.expect("PK budgets")[side])
        } else {
            (snapshot.run_id.clone().expect("playing run"), holder.human_match.budget.expect("playing budget"))
        };
        holder.human_match.snapshot.run_id = Some(run_id.clone());
        let request = GameMoveRequest { position, identity: GameMoveRequestDto {
            run_id, generation: turn.generation, node_path: turn.node_path.clone(), budget } };
        let started = if human { manager.start_reserved_analysis(&turn.session_id, request) }
            else if enabled { manager.start_reserved_game_move_with_analysis(&turn.session_id, request) }
            else { manager.start_reserved_game_move(&turn.session_id, request) };
        match started {
            Ok(handle) => {
                holder.human_match.snapshot.job = Some(handle.identity.clone());
                holder.human_match.analysis_work = enabled.then(|| AnalysisWork {
                    turn: turn.clone(), epoch, job: handle.identity.clone(), human });
                holder.human_match.changed();
                Ok(Some(MatchWork { turn, epoch, human, handle }))
            }
            Err(error) => {
                holder.human_match.seal(MatchEndDto::Failed, Some(error.clone()));
                Err(error)
            }
        }
    }

    pub(crate) fn publish_match_analysis(&self, turn: &MatchTurnDto, epoch: u64,
        job: &GameMoveJobDto, frame: AnalysisFrameDto) -> Option<MatchUpdateDto> {
        let mut holder = self.holder.lock().expect("current game state");
        let work = holder.human_match.analysis_work.as_ref()?;
        if work.turn != *turn || work.epoch != epoch || work.job != *job
            || holder.admit_match_identity(turn, work.human).is_err()
            || holder.human_match.snapshot.analysis.epoch != epoch
            || !holder.human_match.snapshot.analysis.policy.includes(work.human)
            || holder.human_match.snapshot.run_id.as_deref() != Some(job.run_id.as_str())
            || job.generation != turn.generation || job.node_path != turn.node_path
            || frame.job_id.to_string() != job.job_id {
            return None;
        }
        holder.human_match.snapshot.analysis.frame = Some(MatchAnalysisFrameDto {
            turn: turn.clone(), epoch, job: job.clone(), frame });
        holder.human_match.changed();
        Some(holder.analysis_update())
    }

    pub(crate) fn finish_match_analysis(&self, turn: &MatchTurnDto, epoch: u64,
        result: Result<GameMoveResultDto, EngineFailureDto>) -> MatchUpdateDto {
        let mut holder = self.holder.lock().expect("current game state");
        if !holder.human_match.analysis_work.as_ref().is_some_and(|work|
            work.human && work.turn == *turn && work.epoch == epoch)
            || holder.admit_match_identity(turn, true).is_err() {
            return holder.analysis_update();
        }
        holder.human_match.analysis_work = None;
        holder.human_match.analysis_completed = Some((turn.turn, epoch));
        holder.human_match.snapshot.job = None;
        match result {
            Ok(_) => holder.human_match.changed(),
            Err(error) => holder.human_match.seal(MatchEndDto::Failed, Some(error)),
        }
        holder.analysis_update()
    }
}
