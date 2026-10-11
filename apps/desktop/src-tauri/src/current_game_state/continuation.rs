use super::*;
use app_model::{ContinuationPreferencesDto, ContinuationSnapshotDto};
use engine_manager::{ContinuationAction, ForegroundEngineManager, SelectedNodeJobRequest};
use std::time::Instant;

impl CurrentGameState {
    pub fn start_continuation(
        &self,
        manager: &ForegroundEngineManager,
        generation: u64,
        path: NodePath,
        choices: Vec<app_model::AnalysisBranchChoiceDto>,
        preferences: ContinuationPreferencesDto,
        epoch: u64,
    ) -> Result<ContinuationSnapshotDto, String> {
        let holder = self.holder.lock().expect("current game state");
        holder.ensure_editable().map_err(|error| error.message)?;
        holder
            .ensure_generation(generation)
            .map_err(|error| error.message)?;
        if holder.selected_path != path {
            return Err("Selected position changed before Start.".into());
        }
        let document = holder.document.as_ref().ok_or("No current game.")?;
        let line = document
            .continuation_line(&path, &choices)
            .map_err(|error| error.message)?;
        let terminal = document
            .continuation_terminal(&path)
            .map_err(|error| error.message)?;
        let run = match manager.snapshot().lifecycle {
            app_model::ForegroundEngineLifecycleDto::Ready { run } => run.run_id,
            _ => return Err("Engine Continuation requires a qualified Ready review Run.".into()),
        };
        let request = continuation_request(&holder, run)?;
        manager
            .start_continuation(request, preferences, line, terminal, Instant::now(), epoch)
            .map_err(|error| error.message)
    }

    pub fn stop_continuation(
        &self,
        manager: &ForegroundEngineManager,
        reason: &str,
    ) -> ContinuationSnapshotDto {
        let holder = self.holder.lock().expect("current game state");
        let mut snapshot = manager.stop_continuation(reason);
        snapshot.current_game = holder.current_result().ok();
        snapshot
    }

    pub fn continuation_beat(
        &self,
        manager: &ForegroundEngineManager,
        session: &str,
    ) -> ContinuationSnapshotDto {
        self.continuation_beat_at(manager, session, Instant::now())
    }

    pub(super) fn continuation_beat_at(
        &self,
        manager: &ForegroundEngineManager,
        session: &str,
        now: Instant,
    ) -> ContinuationSnapshotDto {
        let mut holder = self.holder.lock().expect("current game state");
        let before = manager.continuation_snapshot();
        if before.session_id.as_deref() != Some(session) {
            return before;
        }
        if holder.departure.is_some() && !holder.edits_blocked {
            return before;
        }
        if holder.ensure_editable().is_err()
            || holder.generation != before.generation
            || holder.selected_path != before.node_path
        {
            return manager.stop_continuation("Current-game authority or selected position changed.");
        }
        let mut changed = false;
        let mut snapshot = manager.advance_continuation(session, now, |action| {
            match action {
                ContinuationAction::Navigate(path) => {
                    holder.select_path(path).map_err(|error| error.message)?;
                    holder.bump_snapshot();
                }
                ContinuationAction::Play(vertex) => {
                    let path = holder.selected_path.clone();
                    // Exact GTP positions enforce complete ko history; other JSONL rules retain
                    // the authoritative SGF play validator, not a UI/policy fallback.
                    if let Ok(exact) = holder
                        .document
                        .as_ref()
                        .expect("admitted document")
                        .exact_position(&path)
                    {
                        exact.validate_move(&vertex)?;
                    }
                    holder.play(path, vertex).map_err(|error| error.message)?;
                }
            }
            changed = true;
            let terminal = holder
                .document
                .as_ref()
                .expect("admitted document")
                .continuation_terminal(&holder.selected_path)
                .map_err(|error| error.message)?;
            let request =
                continuation_request(&holder, before.run_id.clone().ok_or("Continuation Run missing")?)?;
            holder.analysis_target = Some((holder.generation, holder.selected_path.clone()));
            Ok((request, terminal))
        });
        if changed {
            self.note_recovery(&holder);
            snapshot.current_game = holder.current_result().ok();
        }
        snapshot
    }

    pub(super) fn interrupt_continuation(&self) {
        if let Some(manager) = self.analysis_manager.get() {
            manager.stop_continuation("Stopped by a review action.");
        }
    }
}

fn continuation_request(
    holder: &CurrentGameHolder,
    run_id: String,
) -> Result<SelectedNodeJobRequest, String> {
    let document = holder.document.as_ref().ok_or("No current game")?;
    let snapshot = document
        .snapshot(&holder.selected_path)
        .map_err(|error| error.message)?;
    if !snapshot.position.errors.is_empty() {
        return Err(snapshot.position.errors.join("; "));
    }
    let mut request = crate::continuous_analysis::position_request(
        holder.generation,
        holder.selected_path.clone(),
        snapshot,
        document.board_width(),
        document.board_height(),
        document.komi(),
        document.rules(),
    )?;
    request.run_id = run_id;
    request.mode = AnalysisJobModeDto::Continuous;
    request.exact_position = document.exact_position(&holder.selected_path);
    Ok(request)
}
