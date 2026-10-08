use super::{trial, CurrentGameState};
use app_model::{
    EngineFailureDto, EngineFailureKind, EngineOperationDto, GameMoveRequestDto, GameMoveResultDto,
};
use engine_manager::{ForegroundEngineManager, GameMoveHandle, GameMoveRequest};

#[cfg(all(test, unix))]
mod tests;

fn failure(run_id: &str, kind: EngineFailureKind, message: String) -> Box<EngineFailureDto> {
    Box::new(EngineFailureDto {
        operation: EngineOperationDto::Job,
        run_id: Some(run_id.to_owned()),
        switch_id: None,
        job_id: None,
        profile_id: None,
        kind,
        message,
        diagnostic_summary: None,
    })
}

impl CurrentGameState {
    pub fn start_ordinary_rules(
        &self,
        manager: &ForegroundEngineManager,
        request: app_model::OrdinaryRulesRequestDto,
    ) -> crate::EngineCommandResult<engine_manager::OrdinaryRulesHandle> {
        let holder = self.holder.lock().expect("current game state");
        if holder.departure.is_some()
            || holder.human_match.blocks()
            || !matches!(holder.trial_mode, trial::TrialMode::Review)
            || holder.generation != request.generation
            || holder.selected_path != request.node_path
        {
            return Err(failure(
                &request.run_id,
                EngineFailureKind::InvalidState,
                "Rules confirmation requires the current review position outside departure or Match".into(),
            ));
        }
        let document = holder.document.as_ref().ok_or_else(|| {
            failure(
                &request.run_id,
                EngineFailureKind::InvalidState,
                "Rules confirmation requires a current game".into(),
            )
        })?;
        let position = document
            .exact_position(&request.node_path)
            .map_err(|message| failure(&request.run_id, EngineFailureKind::UnsupportedCapability, message))?;
        manager
            .start_ordinary_rules(GameMoveRequest {
                identity: GameMoveRequestDto {
                    run_id: request.run_id,
                    generation: request.generation,
                    node_path: request.node_path,
                    budget: app_model::ComputeBudgetDto {
                        deadline_ms: 30000,
                        max_visits: None,
                    },
                },
                position,
            })
            .map_err(Box::new)
    }

    pub fn publish_ordinary_rules(
        &self,
        manager: &ForegroundEngineManager,
        result: app_model::OrdinaryRulesSnapshotDto,
    ) -> crate::EngineCommandResult<app_model::OrdinaryRulesSnapshotDto> {
        let holder = self.holder.lock().expect("current game state");
        if holder.departure.is_some()
            || holder.human_match.blocks()
            || !matches!(holder.trial_mode, trial::TrialMode::Review)
            || holder.generation != result.identity.generation
            || holder.selected_path != result.identity.node_path
        {
            return Err(failure(
                &result.identity.run_id,
                EngineFailureKind::Cancellation,
                "Rules confirmation belongs to a retired position".into(),
            ));
        }
        manager.claim_ordinary_rules(&result).map_err(Box::new)?;
        Ok(result)
    }

    pub fn start_game_move(
        &self,
        manager: &ForegroundEngineManager,
        request: GameMoveRequestDto,
    ) -> crate::EngineCommandResult<GameMoveHandle> {
        // Admission and accepted cursor changes share this lock. No document mutation is needed.
        let holder = self.holder.lock().expect("current game state");
        if holder.departure.is_some()
            || holder.human_match.blocks()
            || !matches!(holder.trial_mode, trial::TrialMode::Review)
            || holder.generation != request.generation
            || holder.selected_path != request.node_path
        {
            return Err(failure(
                &request.run_id,
                EngineFailureKind::InvalidState,
                "move requires the current review generation and selected path outside departure".into(),
            ));
        }
        let document = holder.document.as_ref().ok_or_else(|| {
            failure(
                &request.run_id,
                EngineFailureKind::InvalidState,
                "move requires a current game".into(),
            )
        })?;
        let position = document
            .exact_position(&request.node_path)
            .map_err(|message| failure(&request.run_id, EngineFailureKind::UnsupportedCapability, message))?;
        manager
            .start_game_move(GameMoveRequest {
                identity: request,
                position,
            })
            .map_err(Box::new)
    }

    pub fn publish_game_move(
        &self,
        manager: &ForegroundEngineManager,
        result: GameMoveResultDto,
    ) -> crate::EngineCommandResult<GameMoveResultDto> {
        let holder = self.holder.lock().expect("current game state");
        let snapshot = manager.snapshot();
        if holder.departure.is_some()
            || holder.human_match.blocks()
            || !matches!(holder.trial_mode, trial::TrialMode::Review)
            || holder.generation != result.generation
            || holder.selected_path != result.node_path
            || !matches!(snapshot.lifecycle, app_model::ForegroundEngineLifecycleDto::Ready { ref run }
                if run.run_id == result.run_id)
        {
            let mut error = failure(
                &result.run_id,
                EngineFailureKind::Cancellation,
                "move result belongs to a retired run or position".into(),
            );
            error.job_id = Some(result.job_id);
            return Err(error);
        }
        manager.claim_game_move_result(&result).map_err(Box::new)?;
        Ok(result)
    }
}
