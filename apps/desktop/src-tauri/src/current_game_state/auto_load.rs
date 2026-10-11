use super::*;

impl CurrentGameState {
    /// Both immediate commit and the ordered manager event consumer use this admission seam.
    /// Snapshot notifications are only wakeups; authority is re-read under both owner locks.
    pub fn reconcile_auto_load(&self) {
        let Some(manager) = self.analysis_manager.get() else {
            return;
        };
        let holder = self.holder.lock().expect("current game state");
        let Some(intent) = manager.pending_auto_load() else {
            return;
        };
        let admit = || -> Result<(), String> {
            holder.ensure_review_access().map_err(|error| error.message)?;
            if holder.generation != intent.generation || holder.document_identity != intent.load_id {
                return Err("Automatic load belongs to a retired document.".into());
            }
            let document = holder.document.as_ref().ok_or("No current game")?;
            let nodes = document
                .missing_primary_mainline_snapshots()
                .map_err(|error| error.message)?;
            if nodes.is_empty() {
                manager.finish_auto_load_admission(intent.load_id, None);
                return Ok(());
            }
            let run_id = intent.run_id.clone().ok_or("No captured Run")?;
            let admission = WholeGameAdmission {
                generation: holder.generation,
                board_width: document.board_width(),
                board_height: document.board_height(),
                komi: document.komi(),
                rules: document.rules(),
                requested: nodes.iter().map(|node| node.path.clone()).collect(),
                nodes,
                supporting: Vec::new(),
                swing_comparisons: Vec::new(),
                swing_criteria: None,
            };
            let work_items = crate::whole_game_work_items(&admission, intent.total_visits, &run_id)
                .map_err(|error| error.message)?;
            manager
                .start_auto_load_task(
                    intent.clone(),
                    engine_manager::WholeGameJobRequest {
                        run_id,
                        generation: holder.generation,
                        work_items,
                    },
                    app_model::AnalysisScopeDto {
                        mode: app_model::AnalysisScopeModeDto::FirstChildMainline,
                        current_node: NodePath::default(),
                        branch_choices: Vec::new(),
                        interval: None,
                        to_play: None,
                    },
                )
                .map_err(|error| error.message)?;
            Ok(())
        };
        if let Err(reason) = admit() {
            manager.finish_auto_load_admission(intent.load_id, Some(reason));
        }
    }
}
