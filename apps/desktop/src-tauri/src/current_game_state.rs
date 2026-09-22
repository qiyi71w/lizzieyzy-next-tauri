use ::current_game_recovery::RecoveryCoordinator;
use app_model::{
    admits_analysis_attachment, AnalysisJobEventDto, AnalysisJobModeDto, AnalysisJobStartedDto,
    ApplicationExitDispositionDto, CurrentGameError, CurrentGameErrorKind, CurrentGameResultDto, GameDto,
    MoveVertex, NodePath, RecoveryEnvelopeDto, RecoveryProtectionDto, SelectedNodeSnapshotDto, TrialSessionDto,
};
use sgf::{CurrentSgfDocument, DocumentHistory, SgfAnalysisPayload, SgfDocumentEdit, TrialLine};
use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};

#[cfg(test)]
mod continuous_intent;
#[cfg(test)]
mod current_game_analysis_attach;
#[cfg(test)]
mod current_game_departure;
#[cfg(test)]
mod current_game_save_write;
#[cfg(test)]
mod current_game_session_recovery;
mod departure;
mod trial;
pub(crate) mod recovery;

#[derive(Debug, Clone)]
pub struct WholeGameAdmission {
    pub generation: u64,
    pub board_width: u8,
    pub board_height: u8,
    pub komi: f32,
    pub rules: String,
    pub nodes: Vec<SelectedNodeSnapshotDto>,
    pub requested: Vec<NodePath>,
    pub supporting: Vec<NodePath>,
    pub swing_comparisons: Vec<app_model::AnalysisSwingComparisonDto>,
    pub swing_criteria: Option<app_model::AnalysisSwingCriteriaDto>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct ContentVersion {
    edit: u64,
    nonhistory: u64,
}
#[derive(Default)]
pub struct CurrentGameState {
    holder: Mutex<CurrentGameHolder>,
    recovery: Mutex<RecoveryCoordinator>,
    analysis_manager: OnceLock<engine_manager::ForegroundEngineManager>,
}

#[derive(Default)]
struct CurrentGameHolder {
    document: Option<CurrentSgfDocument>,
    generation: u64,
    dirty: bool,
    history: DocumentHistory,
    undo_revisions: Vec<(u64, u64)>,
    redo_revisions: Vec<(u64, u64)>,
    next_edit_revision: u64,
    edit_revision: u64,
    nonhistory_revision: u64,
    saved_version: ContentVersion,
    native_path: Option<String>,
    departure: Option<departure::DepartureSession>,
    next_departure_id: u64,
    edits_blocked: bool,
    closed_jobs: HashSet<(String, String)>,
    exit_disposition: Option<ApplicationExitDispositionDto>,
    selected_path: NodePath,
    document_seq: u64,
    document_identity: u64,
    snapshot_seq: u64,
    analysis_target: Option<(u64, NodePath)>,
    trial_mode: trial::TrialMode,
    next_trial_id: u64,
    next_trial_revision: u64,
}

impl CurrentGameState {
    pub fn connect_analysis_manager(&self, manager: engine_manager::ForegroundEngineManager) {
        assert!(
            self.analysis_manager.set(manager).is_ok(),
            "analysis manager already connected"
        );
        let mut holder = self.holder.lock().expect("current game state");
        self.follow_continuous_position(&mut holder);
    }

    // Called with the holder locked: accepted cursor/edit order is also target order.
    fn follow_continuous_position(&self, holder: &mut CurrentGameHolder) {
        if !matches!(holder.trial_mode, trial::TrialMode::Review) {
            return;
        }
        let Some(manager) = self.analysis_manager.get() else {
            return;
        };
        manager.invalidate_analysis_task(holder.generation);
        if holder.analysis_target.as_ref().is_some_and(|(generation, path)| {
            *generation == holder.generation && *path == holder.selected_path
        }) {
            return;
        }
        if let Some(job) = manager
            .snapshot()
            .selected_node_job
            .filter(|job| job.generation != holder.generation || job.node_path != holder.selected_path)
        {
            holder.closed_jobs.insert((job.run_id, job.job_id));
        }
        let Some(document) = holder.document.as_ref() else {
            manager.clear_continuous_position();
            return;
        };
        let request = document
            .snapshot(&holder.selected_path)
            .map_err(|error| error.to_string())
            .and_then(|snapshot| {
                crate::continuous_analysis::position_request(
                    holder.generation,
                    holder.selected_path.clone(),
                    snapshot,
                    document.board_width(),
                    document.board_height(),
                    document.komi(),
                    document.rules(),
                )
            });
        match request {
            Ok(request) => {
                holder.analysis_target = Some((holder.generation, holder.selected_path.clone()));
                manager.follow_continuous_position(request);
            }
            Err(_) => manager.clear_continuous_position(),
        }
    }

    pub fn analysis_jobs(&self) -> Option<Vec<AnalysisJobStartedDto>> {
        self.analysis_manager
            .get()
            .map(|manager| crate::document_departure::jobs_from_snapshot(&manager.snapshot()))
    }

    #[cfg(test)]
    pub fn replace(
        &self,
        sgf_text: &str,
        native_path: Option<String>,
    ) -> Result<CurrentGameResultDto, CurrentGameError> {
        self.replace_unless_discarded(sgf_text, native_path, true)?
            .ok_or_else(no_current_game)
    }

    #[cfg(test)]
    pub fn replace_unless_discarded(
        &self,
        sgf_text: &str,
        native_path: Option<String>,
        discard_confirmed: bool,
    ) -> Result<Option<CurrentGameResultDto>, CurrentGameError> {
        let mut holder = self.holder.lock().expect("current game state");
        if holder.departure.is_some() {
            return Err(departure::departure_in_progress());
        }
        if holder.dirty && !discard_confirmed {
            return Ok(None);
        }
        let result = holder.replace(sgf_text, native_path)?;
        self.note_recovery(&holder);
        self.follow_continuous_position(&mut holder);
        Ok(Some(result))
    }

    pub fn native_path(&self) -> Option<String> {
        self.holder
            .lock()
            .expect("current game state")
            .native_path
            .as_ref()
            .map(|path| path.trim())
            .filter(|path| !path.is_empty())
            .map(|path| path.to_string())
    }

    pub fn serialize(&self) -> Result<String, CurrentGameError> {
        self.with_document(|document| document.serialize())
    }

    pub fn save_to_path(
        &self,
        path: String,
        selected_path: NodePath,
    ) -> Result<CurrentGameResultDto, String> {
        self.save_to_path_with(path, selected_path, || {})
    }

    fn save_to_path_with(
        &self,
        path: String,
        selected_path: NodePath,
        after_snapshot: impl FnOnce(),
    ) -> Result<CurrentGameResultDto, String> {
        self.save_to_path_allowing_departure(path, selected_path, after_snapshot, false)
    }

    fn save_to_path_allowing_departure(
        &self,
        path: String,
        selected_path: NodePath,
        after_snapshot: impl FnOnce(),
        allow_departure: bool,
    ) -> Result<CurrentGameResultDto, String> {
        let trimmed = path.trim();
        if trimmed.is_empty() {
            return Err("path must not be empty".to_string());
        }
        let target = std::path::PathBuf::from(trimmed);
        let (serialized, version, document_identity) = {
            let holder = self.holder.lock().expect("current game state");
            if holder.edits_blocked && !allow_departure {
                return Err("cannot save while a document departure is in progress".to_string());
            }
            let document = holder
                .document
                .as_ref()
                .ok_or_else(|| no_current_game().to_string())?;
            document
                .snapshot(&selected_path)
                .map_err(|error| error.to_string())?;
            (
                document.serialize().map_err(|error| error.to_string())?,
                holder.content_version(),
                holder.document_identity,
            )
        };
        after_snapshot();
        std::fs::write(&target, &serialized)
            .map_err(|err| format!("failed to write SGF file {}: {err}", target.display()))?;
        let mut holder = self.holder.lock().expect("current game state");
        if holder.document_identity != document_identity {
            return holder.current_result().map_err(|error| error.to_string());
        }
        holder.native_path = Some(trimmed.to_string());
        holder.saved_version = version;
        holder.refresh_dirty();
        holder.bump_snapshot();
        self.note_recovery(&holder);
        holder.current_result().map_err(|error| error.to_string())
    }

    pub fn mainline_projection(&self) -> Result<GameDto, CurrentGameError> {
        self.with_document(|document| Ok(document.mainline_projection()))
    }

    pub fn admit_whole_game(&self, generation: u64) -> Result<WholeGameAdmission, CurrentGameError> {
        let holder = self.holder.lock().expect("current game state");
        holder.analysis_scope_admission(
            generation,
            &app_model::AnalysisScopeDto {
                mode: app_model::AnalysisScopeModeDto::FirstChildMainline,
                current_node: holder.selected_path.clone(),
                branch_choices: Vec::new(),
                interval: None,
                to_play: None,
            },
            None,
        )
    }

    pub fn start_first_child_analysis(
        &self,
        manager: &engine_manager::ForegroundEngineManager,
        run_id: String,
        generation: u64,
        max_visits: u32,
    ) -> Result<AnalysisJobStartedDto, app_model::EngineFailureDto> {
        let holder = self.holder.lock().expect("current game state");
        let scope = app_model::AnalysisScopeDto {
            mode: app_model::AnalysisScopeModeDto::FirstChildMainline,
            current_node: holder.selected_path.clone(),
            branch_choices: Vec::new(),
            interval: None,
            to_play: None,
        };
        let admitted = holder
            .analysis_scope_admission(generation, &scope, None)
            .map_err(|error| {
                crate::job_failure(&run_id, app_model::EngineFailureKind::InvalidState, error.message)
            })?;
        let work_items = crate::whole_game_work_items(&admitted, max_visits, &run_id)?;
        manager.start_whole_game_analysis(engine_manager::WholeGameJobRequest {
            run_id,
            generation,
            work_items,
        })
    }

    pub fn preview_analysis_scope(
        &self,
        generation: u64,
        scope: app_model::AnalysisScopeDto,
        swing_criteria: Option<app_model::AnalysisSwingCriteriaDto>,
    ) -> Result<app_model::AnalysisScopePreviewDto, CurrentGameError> {
        let holder = self.holder.lock().expect("current game state");
        let admission = holder.analysis_scope_admission(generation, &scope, swing_criteria.as_ref())?;
        Ok(scope_preview(&admission, scope))
    }

    pub fn start_analysis_task(
        &self,
        manager: &engine_manager::ForegroundEngineManager,
        run_id: String,
        preview: app_model::AnalysisScopePreviewDto,
        conditions: app_model::AnalysisStageConditionsDto,
    ) -> Result<app_model::AnalysisTaskDto, app_model::EngineFailureDto> {
        let invalid =
            |message| crate::job_failure(&run_id, app_model::EngineFailureKind::InvalidState, message);
        conditions.validate_single_stage().map_err(&invalid)?;
        // Keep semantic revalidation and manager admission under the same owner lock.
        let holder = self.holder.lock().expect("current game state");
        let admitted = holder
            .analysis_scope_admission(preview.generation, &preview.scope, None)
            .map_err(|error| invalid(error.message))?;
        if scope_preview(&admitted, preview.scope.clone()) != preview {
            return Err(invalid(
                "Analysis scope preview no longer matches the current game.".into(),
            ));
        }
        let work_items = crate::whole_game_work_items(&admitted, conditions.total_visits.value, &run_id)?;
        manager.start_analysis_task(
            engine_manager::WholeGameJobRequest {
                run_id,
                generation: admitted.generation,
                work_items,
            },
            preview.scope,
            conditions,
        )
    }

    pub fn start_all_positions_analysis_task(
        &self,
        manager: &engine_manager::ForegroundEngineManager,
        run_id: String,
        preview: app_model::AnalysisScopePreviewDto,
        overview_conditions: app_model::AnalysisStageConditionsDto,
        deep_conditions: app_model::AnalysisStageConditionsDto,
    ) -> Result<app_model::AnalysisTaskDto, app_model::EngineFailureDto> {
        let invalid =
            |message| crate::job_failure(&run_id, app_model::EngineFailureKind::InvalidState, message);
        app_model::AnalysisStageConditionsDto::validate_all_positions_two_stage(
            &overview_conditions,
            &deep_conditions,
        )
        .map_err(&invalid)?;
        let holder = self.holder.lock().expect("current game state");
        let admitted = holder
            .analysis_scope_admission(preview.generation, &preview.scope, None)
            .map_err(|error| invalid(error.message))?;
        if scope_preview(&admitted, preview.scope.clone()) != preview {
            return Err(invalid(
                "Analysis scope preview no longer matches the current game.".into(),
            ));
        }
        let work_items =
            crate::whole_game_work_items(&admitted, deep_conditions.total_visits.value, &run_id)?;
        manager.start_all_positions_analysis_task(
            engine_manager::WholeGameJobRequest {
                run_id,
                generation: admitted.generation,
                work_items,
            },
            preview.scope,
            overview_conditions,
            deep_conditions,
        )
    }

    pub fn start_swing_analysis_task(
        &self,
        manager: &engine_manager::ForegroundEngineManager,
        run_id: String,
        preview: app_model::AnalysisScopePreviewDto,
        overview_conditions: app_model::AnalysisStageConditionsDto,
        deep_conditions: app_model::AnalysisStageConditionsDto,
    ) -> Result<app_model::AnalysisTaskDto, app_model::EngineFailureDto> {
        let invalid =
            |message| crate::job_failure(&run_id, app_model::EngineFailureKind::InvalidState, message);
        overview_conditions.validate_single_stage().map_err(&invalid)?;
        deep_conditions.validate_single_stage().map_err(&invalid)?;
        let criteria = preview
            .swing_criteria
            .clone()
            .ok_or_else(|| invalid("Swing-selected analysis requires swing criteria.".into()))?;
        criteria.validate().map_err(&invalid)?;
        let holder = self.holder.lock().expect("current game state");
        let admitted = holder
            .analysis_scope_admission(preview.generation, &preview.scope, Some(&criteria))
            .map_err(|error| invalid(error.message))?;
        if scope_preview(&admitted, preview.scope.clone()) != preview {
            return Err(invalid(
                "Analysis scope preview no longer matches the current game.".into(),
            ));
        }
        let work_items =
            crate::whole_game_work_items(&admitted, deep_conditions.total_visits.value, &run_id)?;
        manager.start_swing_analysis_task(
            engine_manager::WholeGameJobRequest {
                run_id,
                generation: admitted.generation,
                work_items,
            },
            preview.scope,
            admitted.requested,
            admitted.supporting,
            admitted.swing_comparisons,
            criteria,
            overview_conditions,
            deep_conditions,
        )
    }

    pub fn pause_analysis_task(
        &self,
        manager: &engine_manager::ForegroundEngineManager,
        run_id: &str,
        task_id: &str,
    ) -> Result<app_model::AnalysisTaskDto, app_model::EngineFailureDto> {
        // The holder lock orders Pause against attachment of already queued events.
        let mut holder = self.holder.lock().expect("current game state");
        let task = manager.pause_analysis_task(run_id, task_id)?;
        holder
            .closed_jobs
            .insert((task.run_id.clone(), task.job_id.clone()));
        Ok(task)
    }

    pub fn continue_analysis_task(
        &self,
        manager: &engine_manager::ForegroundEngineManager,
        run_id: &str,
        task_id: &str,
    ) -> Result<app_model::AnalysisTaskDto, app_model::EngineFailureDto> {
        let holder = self.holder.lock().expect("current game state");
        holder.ensure_editable().map_err(|error| {
            crate::job_failure(run_id, app_model::EngineFailureKind::InvalidState, error.message)
        })?;
        manager.continue_analysis_task(run_id, task_id, holder.generation)
    }

    pub fn admit_selected_node(
        &self,
        generation: u64,
        path: &NodePath,
    ) -> Result<(SelectedNodeSnapshotDto, u8, u8, f32, String), CurrentGameError> {
        let holder = self.holder.lock().expect("current game state");
        holder.ensure_editable()?;
        let document = holder.document.as_ref().ok_or_else(no_current_game)?;
        if holder.generation != generation {
            return Err(CurrentGameError {
                kind: CurrentGameErrorKind::NoCurrentGame,
                message: "current game generation does not match".to_string(),
            });
        }
        let snapshot = document.snapshot(path)?;
        Ok((
            snapshot,
            document.board_width(),
            document.board_height(),
            document.komi(),
            document.rules(),
        ))
    }

    #[allow(dead_code)]
    pub fn discard_confirmation_required(&self) -> bool {
        self.holder.lock().expect("current game state").dirty
    }

    pub fn select_path(
        &self,
        path: NodePath,
        generation: u64,
    ) -> Result<CurrentGameResultDto, CurrentGameError> {
        let mut holder = self.holder.lock().expect("current game state");
        holder.ensure_editable()?;
        if holder.document.is_none() {
            return Err(no_current_game());
        }
        if holder.generation != generation {
            return Err(CurrentGameError {
                kind: CurrentGameErrorKind::InvalidNodePath,
                message: "Current game semantics changed; select a node from the current tree.".into(),
            });
        }
        let changed = holder.selected_path != path;
        let mut result = holder.select_path(path)?;
        if changed {
            holder.bump_snapshot();
            self.note_recovery(&holder);
        }
        self.follow_continuous_position(&mut holder);
        result.snapshot_seq = holder.snapshot_seq;
        Ok(result)
    }

    pub fn play(&self, path: NodePath, vertex: MoveVertex) -> Result<CurrentGameResultDto, CurrentGameError> {
        let mut holder = self.holder.lock().expect("current game state");
        let result = holder.play(path, vertex)?;
        self.note_recovery(&holder);
        self.follow_continuous_position(&mut holder);
        Ok(result)
    }

    pub fn set_personal_comment(
        &self,
        path: NodePath,
        comment: String,
    ) -> Result<CurrentGameResultDto, CurrentGameError> {
        let mut holder = self.holder.lock().expect("current game state");
        let result = holder.set_personal_comment(path, &comment)?;
        self.note_recovery(&holder);
        self.follow_continuous_position(&mut holder);
        Ok(result)
    }

    pub fn remove_variation(&self, path: NodePath) -> Result<CurrentGameResultDto, CurrentGameError> {
        let mut holder = self.holder.lock().expect("current game state");
        let result = holder.remove_variation(path)?;
        self.note_recovery(&holder);
        self.follow_continuous_position(&mut holder);
        Ok(result)
    }

    pub fn undo(&self, generation: u64) -> Result<CurrentGameResultDto, CurrentGameError> {
        let mut holder = self.holder.lock().expect("current game state");
        let result = holder.undo(generation)?;
        self.note_recovery(&holder);
        self.follow_continuous_position(&mut holder);
        Ok(result)
    }

    pub fn redo(&self, generation: u64) -> Result<CurrentGameResultDto, CurrentGameError> {
        let mut holder = self.holder.lock().expect("current game state");
        let result = holder.redo(generation)?;
        self.note_recovery(&holder);
        self.follow_continuous_position(&mut holder);
        Ok(result)
    }

    #[cfg(test)]
    pub fn attach_primary_analysis(
        &self,
        generation: u64,
        path: NodePath,
        payload: SgfAnalysisPayload,
    ) -> Result<CurrentGameResultDto, CurrentGameError> {
        let mut holder = self.holder.lock().expect("current game state");
        let result = holder.attach_primary_analysis(generation, path, payload)?;
        self.note_recovery(&holder);
        Ok(result)
    }

    pub fn run_analysis_action<T>(&self, action: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
        let holder = self.holder.lock().expect("current game state");
        if holder.departure.is_some() {
            return Err("A document departure is still pending.".to_string());
        }
        // Keep explicit authorization and its durable write before any later safety cutoff.
        action()
    }

    pub fn attach_from_job_event(&self, event: &AnalysisJobEventDto) -> Option<CurrentGameResultDto> {
        if !admits_analysis_attachment(event) {
            return None;
        }
        let frame = event.frame.as_ref()?;
        if event.lane == app_model::AnalysisJobLaneDto::WholeGame {
            if let Some(task) = self
                .analysis_manager
                .get()
                .and_then(engine_manager::ForegroundEngineManager::analysis_task_snapshot)
            {
                if task.run_id != event.run_id
                    || task.job_id != event.job_id
                    || task.generation != event.generation
                    || !matches!(
                        task.state,
                        app_model::AnalysisTaskStateDto::Queued
                            | app_model::AnalysisTaskStateDto::Searching
                            | app_model::AnalysisTaskStateDto::Completed
                    )
                {
                    return None;
                }
            }
        }
        let payload = SgfAnalysisPayload::from_frame(frame, "KataGo");
        let mut holder = self.holder.lock().expect("current game state");
        if !matches!(holder.trial_mode, trial::TrialMode::Review) {
            return None;
        }
        if holder.rejects_job(&event.run_id, &event.job_id)
            || (event.mode == AnalysisJobModeDto::Continuous && holder.selected_path != event.node_path)
        {
            return None;
        }
        if event.mode == AnalysisJobModeDto::Continuous {
            if let Some(manager) = self.analysis_manager.get() {
                let snapshot = manager.snapshot();
                if snapshot.continuous.enabled == Some(false)
                    || !snapshot.selected_node_job.is_some_and(|job| {
                        job.run_id == event.run_id
                            && job.job_id == event.job_id
                            && job.state != app_model::AnalysisJobStateDto::Stopping
                    })
                {
                    return None;
                }
            }
        }
        let mut result = holder
            .attach_primary_analysis(event.generation, event.node_path.clone(), payload)
            .ok()?;
        if let Some(analysis) = result.snapshot.primary_analysis.as_mut() {
            // Global policy is a live search output; Java LZ stores candidates and ownership.
            analysis.policy = frame.policy.clone();
        }
        self.note_recovery(&holder);
        Some(result)
    }

    pub fn seal_job(&self, job: &AnalysisJobStartedDto) {
        self.holder
            .lock()
            .expect("current game state")
            .closed_jobs
            .insert((job.run_id.clone(), job.job_id.clone()));
    }

    fn with_document<T>(
        &self,
        f: impl FnOnce(&CurrentSgfDocument) -> Result<T, CurrentGameError>,
    ) -> Result<T, CurrentGameError> {
        let holder = self.holder.lock().expect("current game state");
        f(holder.document.as_ref().ok_or_else(no_current_game)?)
    }
}

fn scope_preview(
    admitted: &WholeGameAdmission,
    scope: app_model::AnalysisScopeDto,
) -> app_model::AnalysisScopePreviewDto {
    let target = |snapshot: &SelectedNodeSnapshotDto| app_model::AnalysisScopeTargetDto {
        node_path: snapshot.path.clone(),
        move_number: snapshot.position.move_number,
        to_play: snapshot.position.to_play,
    };
    let requested = admitted
        .requested
        .iter()
        .map(|path| &path.indices)
        .collect::<HashSet<_>>();
    let supporting = admitted
        .supporting
        .iter()
        .map(|path| &path.indices)
        .collect::<HashSet<_>>();
    app_model::AnalysisScopePreviewDto {
        generation: admitted.generation,
        scope,
        targets: admitted
            .nodes
            .iter()
            .filter(|snapshot| requested.contains(&snapshot.path.indices))
            .map(target)
            .collect(),
        supporting_targets: admitted
            .nodes
            .iter()
            .filter(|snapshot| supporting.contains(&snapshot.path.indices))
            .map(target)
            .collect(),
        swing_comparisons: admitted.swing_comparisons.clone(),
        swing_criteria: admitted.swing_criteria.clone(),
    }
}

fn no_current_game() -> CurrentGameError {
    CurrentGameError {
        kind: CurrentGameErrorKind::NoCurrentGame,
        message: "no current game".to_string(),
    }
}

impl CurrentGameHolder {
    fn analysis_scope_admission(
        &self,
        generation: u64,
        scope: &app_model::AnalysisScopeDto,
        swing_criteria: Option<&app_model::AnalysisSwingCriteriaDto>,
    ) -> Result<WholeGameAdmission, CurrentGameError> {
        self.ensure_editable()?;
        let document = self.document.as_ref().ok_or_else(no_current_game)?;
        if self.generation != generation {
            return Err(CurrentGameError {
                kind: CurrentGameErrorKind::InvalidNodePath,
                message: "Current game semantics changed; preview a new analysis task.".into(),
            });
        }
        let (nodes, requested, supporting, swing_comparisons) = if let Some(criteria) = swing_criteria {
            criteria.validate().map_err(|message| CurrentGameError {
                kind: CurrentGameErrorKind::InvalidNodePath,
                message,
            })?;
            let resolved = document.swing_analysis_scope(scope, criteria.move_actors)?;
            let requested = resolved
                .requested
                .iter()
                .map(|snapshot| snapshot.path.clone())
                .collect();
            let supporting = resolved
                .supporting
                .iter()
                .map(|snapshot| snapshot.path.clone())
                .collect();
            let mut nodes = resolved.requested;
            nodes.extend(resolved.supporting);
            (nodes, requested, supporting, resolved.comparisons)
        } else {
            let nodes = document.analysis_scope_snapshots(scope)?;
            let requested = nodes.iter().map(|snapshot| snapshot.path.clone()).collect();
            (nodes, requested, Vec::new(), Vec::new())
        };
        Ok(WholeGameAdmission {
            generation: self.generation,
            board_width: document.board_width(),
            board_height: document.board_height(),
            komi: document.komi(),
            rules: document.rules(),
            nodes,
            requested,
            supporting,
            swing_comparisons,
            swing_criteria: swing_criteria.cloned(),
        })
    }

    #[cfg(test)]
    fn replace(
        &mut self,
        sgf_text: &str,
        native_path: Option<String>,
    ) -> Result<CurrentGameResultDto, CurrentGameError> {
        let document = CurrentSgfDocument::open(sgf_text)?;
        self.install_document(document, native_path)
    }

    #[cfg(test)]
    fn snapshot_state(&self) -> (u64, bool, Option<String>, Option<String>) {
        (
            self.generation,
            self.dirty,
            self.native_path.clone(),
            self.document
                .as_ref()
                .and_then(|document| document.serialize().ok()),
        )
    }

    fn play(&mut self, path: NodePath, vertex: MoveVertex) -> Result<CurrentGameResultDto, CurrentGameError> {
        self.ensure_editable()?;
        let outcome = self
            .document
            .as_mut()
            .ok_or_else(no_current_game)?
            .play_with_history(&self.selected_path, &path, vertex)?;
        let changed = outcome.edit.is_some();
        if let Some(edit) = outcome.edit {
            self.commit_edit(edit);
            self.generation = self.generation.saturating_add(1);
        }
        if changed || self.selected_path != outcome.snapshot.path {
            self.selected_path = outcome.snapshot.path;
            self.bump_snapshot();
        }
        self.current_result()
    }

    fn select_path(&mut self, path: NodePath) -> Result<CurrentGameResultDto, CurrentGameError> {
        self.document
            .as_ref()
            .ok_or_else(no_current_game)?
            .snapshot(&path)?;
        self.selected_path = path;
        self.current_result()
    }

    fn set_personal_comment(
        &mut self,
        path: NodePath,
        comment: &str,
    ) -> Result<CurrentGameResultDto, CurrentGameError> {
        self.ensure_editable()?;
        let outcome = self
            .document
            .as_mut()
            .ok_or_else(no_current_game)?
            .set_personal_comment_with_history(&self.selected_path, &path, comment)?;
        if let Some(edit) = outcome.edit {
            self.commit_edit(edit);
            self.selected_path = outcome.snapshot.path;
            self.bump_snapshot();
        }
        self.current_result()
    }

    fn remove_variation(&mut self, path: NodePath) -> Result<CurrentGameResultDto, CurrentGameError> {
        self.ensure_editable()?;
        let outcome = self
            .document
            .as_mut()
            .ok_or_else(no_current_game)?
            .remove_variation_with_history(&self.selected_path, &path)?;
        self.commit_edit(
            outcome
                .edit
                .expect("variation removal always changes the document"),
        );
        self.generation = self.generation.saturating_add(1);
        self.selected_path = outcome.snapshot.path;
        self.bump_snapshot();
        self.current_result()
    }

    fn undo(&mut self, generation: u64) -> Result<CurrentGameResultDto, CurrentGameError> {
        self.ensure_editable()?;
        self.ensure_generation(generation)?;
        let Some(outcome) = self
            .history
            .undo(self.document.as_mut().ok_or_else(no_current_game)?)?
        else {
            return self.current_result();
        };
        let (before, after) = self
            .undo_revisions
            .pop()
            .expect("history revisions follow SGF history");
        self.redo_revisions.push((before, after));
        self.edit_revision = before;
        if outcome.structural {
            self.generation = self.generation.saturating_add(1);
        }
        self.selected_path = outcome.selected_path;
        self.refresh_dirty();
        self.bump_snapshot();
        self.current_result()
    }

    fn redo(&mut self, generation: u64) -> Result<CurrentGameResultDto, CurrentGameError> {
        self.ensure_editable()?;
        self.ensure_generation(generation)?;
        let Some(outcome) = self
            .history
            .redo(self.document.as_mut().ok_or_else(no_current_game)?)?
        else {
            return self.current_result();
        };
        let (before, after) = self
            .redo_revisions
            .pop()
            .expect("history revisions follow SGF history");
        self.undo_revisions.push((before, after));
        self.edit_revision = after;
        if outcome.structural {
            self.generation = self.generation.saturating_add(1);
        }
        self.selected_path = outcome.selected_path;
        self.refresh_dirty();
        self.bump_snapshot();
        self.current_result()
    }

    fn attach_primary_analysis(
        &mut self,
        generation: u64,
        path: NodePath,
        payload: SgfAnalysisPayload,
    ) -> Result<CurrentGameResultDto, CurrentGameError> {
        self.ensure_editable()?;
        if self.generation != generation {
            return Err(CurrentGameError {
                kind: CurrentGameErrorKind::NoCurrentGame,
                message: "current game generation does not match".to_string(),
            });
        }
        let changed = self
            .document
            .as_mut()
            .ok_or_else(no_current_game)?
            .replace_primary_analysis(&path, &payload)?
            .1;
        if changed {
            self.mark_nonhistory_change();
            self.bump_snapshot();
        }
        self.result_at(&path)
    }

    fn current_result(&self) -> Result<CurrentGameResultDto, CurrentGameError> {
        self.result_at(&self.selected_path)
    }

    fn result_at(&self, selected_path: &NodePath) -> Result<CurrentGameResultDto, CurrentGameError> {
        let document = self.document.as_ref().ok_or_else(no_current_game)?;
        Ok(CurrentGameResultDto {
            tree: document.tree()?,
            selected_path: selected_path.clone(),
            snapshot: document.snapshot(selected_path)?,
            generation: self.generation,
            snapshot_seq: self.snapshot_seq,
            dirty: self.dirty,
            can_undo: self.history.can_undo(),
            can_redo: self.history.can_redo(),
            native_path: self.native_path.clone(),
        })
    }

    fn ensure_generation(&self, generation: u64) -> Result<(), CurrentGameError> {
        if self.generation == generation {
            Ok(())
        } else {
            Err(CurrentGameError {
                kind: CurrentGameErrorKind::InvalidNodePath,
                message: "Current game semantics changed; use the latest document state.".to_string(),
            })
        }
    }

    fn commit_edit(&mut self, edit: SgfDocumentEdit) {
        let before = self.edit_revision;
        self.next_edit_revision = self.next_edit_revision.saturating_add(1);
        let after = self.next_edit_revision;
        self.history.commit(edit);
        self.redo_revisions.clear();
        self.undo_revisions.push((before, after));
        if self.undo_revisions.len() > 100 {
            self.undo_revisions.remove(0);
        }
        self.edit_revision = after;
        self.refresh_dirty();
    }

    fn bump_snapshot(&mut self) {
        self.snapshot_seq = self.snapshot_seq.saturating_add(1);
    }

    fn mark_nonhistory_change(&mut self) {
        self.nonhistory_revision = self.nonhistory_revision.saturating_add(1);
        self.refresh_dirty();
    }

    fn mark_dirty(&mut self) {
        self.mark_nonhistory_change();
    }

    fn content_version(&self) -> ContentVersion {
        ContentVersion {
            edit: self.edit_revision,
            nonhistory: self.nonhistory_revision,
        }
    }

    fn refresh_dirty(&mut self) {
        self.dirty = self.content_version() != self.saved_version;
    }
}

#[cfg(test)]
mod current_game_replacement {
    use super::*;
    use app_model::{CurrentGameErrorKind, MoveVertex, NodePath};

    const BRANCHING: &str = include_str!("../../../../tests/golden/editable-workspace-branching.sgf");
    const EMPTY: &str = "(;GM[1]FF[4]SZ[13:9]KM[7.5]PB[黑]PW[白])";

    #[test]
    fn current_game_replacement_installs_shared_result_and_preserves_state_on_cancel_or_failure() {
        let state = CurrentGameState::default();

        let opened = state
            .replace(BRANCHING, Some("/tmp/branching.sgf".to_string()))
            .unwrap();
        assert_eq!(opened.generation, 1);
        assert!(!opened.dirty);
        assert_eq!(opened.native_path.as_deref(), Some("/tmp/branching.sgf"));
        assert_eq!(opened.selected_path.indices, vec![0, 0, 0]);
        assert_eq!(opened.snapshot.personal_comment, "mainline pass");
        assert_eq!(opened.snapshot.position.move_number, 3);

        let serialized = state.serialize().unwrap();
        let projection = state.mainline_projection().unwrap();
        assert_eq!(projection.moves.len(), 3);
        assert!(matches!(projection.moves[2].vertex, MoveVertex::Pass));
        assert_eq!(
            CurrentSgfDocument::open(&serialized)
                .unwrap()
                .default_selected_path()
                .indices,
            vec![0, 0, 0]
        );

        let imported = state.replace(EMPTY, None).unwrap();
        assert_eq!(imported.generation, 2);
        assert!(!imported.dirty);
        assert!(imported.native_path.is_none());
        assert!(imported.selected_path.indices.is_empty());
        assert_eq!(imported.snapshot.position.move_number, 0);
        assert_eq!(imported.snapshot.position.board_width, 13);
        assert_eq!(imported.snapshot.position.board_height, 9);
        assert!(state.mainline_projection().unwrap().moves.is_empty());

        state.force_dirty();
        let before_cancel = state.inspect();
        assert!(state.discard_confirmation_required());
        let cancelled = state.replace_unless_discarded(EMPTY, None, false).unwrap();
        assert!(cancelled.is_none());
        assert_eq!(state.inspect(), before_cancel);

        let before_failure = state.inspect();
        let error = state
            .replace("not an sgf", Some("/tmp/bad.sgf".to_string()))
            .unwrap_err();
        assert_eq!(error.kind, CurrentGameErrorKind::MalformedSgf);
        assert_eq!(state.inspect(), before_failure);

        let before_unsupported = state.inspect();
        let unsupported = state.replace("(;GM[1]FF[4]SZ[99])", None).unwrap_err();
        assert_eq!(unsupported.kind, CurrentGameErrorKind::UnsupportedBoardSize);
        assert_eq!(state.inspect(), before_unsupported);
    }

    #[test]
    fn current_game_replacement_reports_no_current_game_before_install() {
        let state = CurrentGameState::default();
        assert_eq!(
            state.serialize().unwrap_err().kind,
            CurrentGameErrorKind::NoCurrentGame
        );
        assert_eq!(
            state.mainline_projection().unwrap_err().kind,
            CurrentGameErrorKind::NoCurrentGame
        );
        assert_eq!(
            state.admit_whole_game(1).unwrap_err().kind,
            CurrentGameErrorKind::NoCurrentGame
        );
    }

    #[test]
    fn select_path_returns_snapshot_without_mutating_document_identity() {
        let state = CurrentGameState::default();
        let missing = state
            .select_path(NodePath { indices: Vec::new() }, state.inspect().0)
            .unwrap_err();
        assert_eq!(missing.kind, CurrentGameErrorKind::NoCurrentGame);

        let opened = state
            .replace(BRANCHING, Some("/tmp/branching.sgf".to_string()))
            .unwrap();
        let before = state.holder.lock().expect("current game state").snapshot_state();

        let second = state
            .select_path(NodePath { indices: vec![0, 1] }, state.inspect().0)
            .unwrap();
        assert_eq!(
            state.holder.lock().expect("current game state").snapshot_state(),
            before
        );
        assert_eq!(second.generation, opened.generation);
        assert_eq!(second.dirty, opened.dirty);
        assert_eq!(second.native_path, opened.native_path);
        assert_eq!(second.tree, opened.tree);
        assert_eq!(second.selected_path.indices, vec![0, 1]);
        assert_eq!(second.snapshot.path.indices, vec![0, 1]);
        assert_eq!(second.snapshot.personal_comment, "second continuation");
        assert_eq!(second.snapshot.position.move_number, 2);
        assert_eq!(second.snapshot.position.to_play, app_model::PlayerColor::White);

        let invalid = state
            .select_path(NodePath { indices: vec![0, 2] }, state.inspect().0)
            .unwrap_err();
        assert_eq!(invalid.kind, CurrentGameErrorKind::InvalidNodePath);
        assert_eq!(
            state.holder.lock().expect("current game state").snapshot_state(),
            before
        );
        assert_eq!(opened.selected_path.indices, vec![0, 0, 0]);
    }

    #[test]
    fn exact_navigation_rejects_old_document_and_invalid_paths_without_dirtying() {
        let state = CurrentGameState::default();
        let old = state.replace(BRANCHING, None).unwrap();
        let current = state
            .replace(
                include_str!("../../../../tests/golden/r6-exact-navigation.sgf"),
                None,
            )
            .unwrap();
        let before = state.serialize().unwrap();
        assert!(state.select_path(NodePath::default(), old.generation).is_err());
        assert!(state
            .select_path(NodePath { indices: vec![9] }, current.generation)
            .is_err());
        let selected = state
            .select_path(current.selected_path.clone(), current.generation)
            .unwrap();
        assert_eq!(selected, current);
        for path in [vec![], vec![0], vec![0, 0], vec![0, 0, 0], vec![0, 0, 0, 1, 0]] {
            let result = state
                .select_path(NodePath { indices: path }, current.generation)
                .unwrap();
            assert!(!result.dirty);
            assert_eq!(result.generation, current.generation);
            assert_eq!(state.serialize().unwrap(), before);
        }
    }

    #[test]
    fn admit_selected_node_requires_matching_generation_and_existing_path() {
        let state = CurrentGameState::default();
        let opened = state
            .replace("(;GM[1]FF[4]SZ[13:9]KM[7.5]RU[Chinese])", None)
            .unwrap();
        let (snapshot, board_width, board_height, komi, rules) = state
            .admit_selected_node(opened.generation, &opened.selected_path)
            .unwrap();
        assert_eq!(snapshot.path, opened.selected_path);
        assert_eq!(board_width, 13);
        assert_eq!(board_height, 9);
        assert_eq!(komi, 7.5);
        assert_eq!(rules, "chinese");

        let stale = state
            .admit_selected_node(opened.generation + 1, &opened.selected_path)
            .unwrap_err();
        assert_eq!(stale.message, "current game generation does not match");

        let missing = state
            .admit_selected_node(opened.generation, &NodePath { indices: vec![9] })
            .unwrap_err();
        assert_eq!(missing.kind, CurrentGameErrorKind::InvalidNodePath);
    }

    #[test]
    fn admit_whole_game_captures_first_child_mainline_and_rejects_stale_generation() {
        let state = CurrentGameState::default();
        assert_eq!(
            state.admit_whole_game(1).unwrap_err().kind,
            CurrentGameErrorKind::NoCurrentGame
        );

        let opened = state
            .replace(
                "(;GM[1]FF[4]SZ[5:7]KM[0.5]RU[Chinese];B[aa](;W[bb];B[cc])(;W[dd];B[]))",
                Some("/tmp/branching.sgf".to_string()),
            )
            .unwrap();
        let sibling = state
            .select_path(NodePath { indices: vec![0, 1] }, state.inspect().0)
            .unwrap();
        assert_eq!(sibling.generation, opened.generation);
        assert_eq!(sibling.selected_path.indices, vec![0, 1]);

        let admitted = state.admit_whole_game(opened.generation).unwrap();
        assert_eq!(admitted.generation, opened.generation);
        assert_eq!(admitted.board_width, 5);
        assert_eq!(admitted.board_height, 7);
        assert_eq!(admitted.komi, 0.5);
        assert_eq!(admitted.rules, "chinese");
        let paths: Vec<Vec<u32>> = admitted
            .nodes
            .iter()
            .map(|node| node.path.indices.clone())
            .collect();
        assert_eq!(paths, vec![Vec::new(), vec![0], vec![0, 0], vec![0, 0, 0]]);
        assert_eq!(
            admitted.nodes[3].position.last_move.as_ref().unwrap().vertex,
            MoveVertex::Point(app_model::PointDto { x: 2, y: 2 })
        );

        assert!(state.admit_whole_game(opened.generation + 1).is_err());
    }
}

#[cfg(test)]
mod current_game_remove_variation {
    use super::*;
    use app_model::{CurrentGameErrorKind, PlayerColor};

    const BRANCHING: &str = include_str!("../../../../tests/golden/editable-workspace-branching.sgf");

    #[test]
    fn current_game_remove_variation_returns_parent_and_preserves_state_on_rejection() {
        let state = CurrentGameState::default();
        assert_eq!(
            state
                .remove_variation(NodePath { indices: vec![0] })
                .unwrap_err()
                .kind,
            CurrentGameErrorKind::NoCurrentGame
        );

        let opened = state
            .replace(BRANCHING, Some("/tmp/branching.sgf".to_string()))
            .unwrap();
        let before = state.inspect();
        let before_projection = state.mainline_projection().unwrap();
        let before_serialize = state.serialize().unwrap();

        let root = state
            .remove_variation(NodePath { indices: Vec::new() })
            .unwrap_err();
        assert_eq!(root.kind, CurrentGameErrorKind::RootRemoval);
        assert_eq!(state.inspect(), before);
        assert_eq!(state.serialize().unwrap(), before_serialize);
        assert_eq!(
            state.mainline_projection().unwrap().moves.len(),
            before_projection.moves.len()
        );

        let invalid = state
            .remove_variation(NodePath { indices: vec![0, 2] })
            .unwrap_err();
        assert_eq!(invalid.kind, CurrentGameErrorKind::InvalidNodePath);
        assert_eq!(state.inspect(), before);
        assert_eq!(state.serialize().unwrap(), before_serialize);

        let removed = state.remove_variation(NodePath { indices: vec![0, 1] }).unwrap();
        assert_eq!(removed.selected_path.indices, vec![0]);
        assert_eq!(removed.snapshot.path.indices, vec![0]);
        assert_eq!(removed.snapshot.personal_comment, "main move");
        assert_eq!(removed.snapshot.position.to_play, PlayerColor::Black);
        assert_eq!(removed.generation, opened.generation + 1);
        assert!(removed.dirty);
        assert_eq!(removed.native_path, opened.native_path);
        assert_eq!(removed.tree.children[0].children.len(), 1);
        assert!(state.serialize().unwrap().contains("first continuation"));
        assert!(!state.serialize().unwrap().contains("second continuation"));
        assert_eq!(
            state.inspect(),
            (
                removed.generation,
                true,
                opened.native_path.clone(),
                Some(state.serialize().unwrap())
            )
        );
        assert_eq!(state.mainline_projection().unwrap().moves.len(), 3);
    }
}

#[cfg(test)]
impl CurrentGameState {
    pub(crate) fn force_dirty(&self) {
        self.holder.lock().expect("current game state").mark_dirty();
    }

    fn save_to_path_after_hook(
        &self,
        path: String,
        selected_path: NodePath,
        hook: impl FnOnce(),
    ) -> Result<CurrentGameResultDto, String> {
        self.save_to_path_with(path, selected_path, hook)
    }

    fn inspect(&self) -> (u64, bool, Option<String>, Option<String>) {
        self.holder.lock().expect("current game state").snapshot_state()
    }
}

#[cfg(test)]
mod current_game_mutation_edit {
    use super::*;
    use app_model::{CurrentGameErrorKind, MoveVertex, NodePath, PointDto};

    const BRANCHING: &str = include_str!("../../../../tests/golden/editable-workspace-branching.sgf");

    #[test]
    fn current_game_move_edit_updates_generation_dirty_and_stays_atomic() {
        let state = CurrentGameState::default();
        assert_eq!(
            state
                .play(NodePath { indices: Vec::new() }, MoveVertex::Pass)
                .unwrap_err()
                .kind,
            CurrentGameErrorKind::NoCurrentGame
        );

        let opened = state
            .replace(BRANCHING, Some("/tmp/branching.sgf".to_string()))
            .unwrap();
        assert_eq!(opened.generation, 1);
        assert!(!opened.dirty);

        let parent = NodePath { indices: vec![0] };
        let played = state
            .play(parent.clone(), MoveVertex::Point(PointDto { x: 1, y: 1 }))
            .unwrap();
        assert_eq!(played.generation, 2);
        assert!(played.dirty);
        assert_eq!(played.selected_path.indices, vec![0, 2]);
        assert_eq!(
            played.snapshot.position.last_move.as_ref().unwrap().color,
            app_model::PlayerColor::Black
        );
        assert_eq!(state.mainline_projection().unwrap().moves.len(), 3);
        assert_eq!(
            CurrentSgfDocument::open(&state.serialize().unwrap())
                .unwrap()
                .tree()
                .unwrap()
                .children[0]
                .children
                .len(),
            3
        );

        let existing = state
            .play(parent.clone(), MoveVertex::Point(PointDto { x: 4, y: 4 }))
            .unwrap();
        assert_eq!(existing.generation, 2);
        assert!(existing.dirty);
        assert_eq!(existing.selected_path.indices, vec![0, 0]);
        assert_eq!(existing.snapshot.personal_comment, "first continuation");

        let before = state.inspect();
        let occupied = state
            .play(parent, MoveVertex::Point(PointDto { x: 3, y: 3 }))
            .unwrap_err();
        assert_eq!(occupied.kind, CurrentGameErrorKind::OccupiedPoint);
        assert_eq!(state.inspect(), before);

        let invalid = state
            .play(NodePath { indices: vec![9] }, MoveVertex::Pass)
            .unwrap_err();
        assert_eq!(invalid.kind, CurrentGameErrorKind::InvalidNodePath);
        assert_eq!(state.inspect(), before);
    }
}

#[cfg(test)]
mod current_game_comment_edit {
    use super::*;
    use app_model::{CurrentGameErrorKind, NodePath};

    const BRANCHING: &str = include_str!("../../../../tests/golden/editable-workspace-branching.sgf");

    #[test]
    fn current_game_comment_edit_marks_dirty_and_leaves_noop_unchanged() {
        let state = CurrentGameState::default();
        let opened = state
            .replace(BRANCHING, Some("/tmp/branching.sgf".to_string()))
            .unwrap();
        let root = NodePath { indices: Vec::new() };
        let move_node = NodePath { indices: vec![0] };
        let second_sibling = NodePath { indices: vec![0, 1] };

        let edited = state
            .set_personal_comment(second_sibling.clone(), "sibling edited".to_string())
            .unwrap();
        assert_eq!(edited.selected_path.indices, second_sibling.indices);
        assert_eq!(edited.snapshot.path.indices, second_sibling.indices);
        assert_eq!(edited.snapshot.personal_comment, "sibling edited");
        assert!(edited.snapshot.generated_information.is_none());
        assert!(edited.dirty);
        assert_eq!(edited.native_path, opened.native_path);
        assert!(state.serialize().unwrap().contains("C[sibling edited]"));
        assert_eq!(
            state.mainline_projection().unwrap().moves.len(),
            opened.snapshot.position.move_number as usize
        );

        let before_noop = state.inspect();
        let serialized = state.serialize().unwrap();
        let projection = state.mainline_projection().unwrap();
        let noop = state
            .set_personal_comment(second_sibling.clone(), "sibling edited".to_string())
            .unwrap();
        assert_eq!(noop.dirty, edited.dirty);
        assert_eq!(noop.snapshot.personal_comment, "sibling edited");
        assert_eq!(state.inspect(), before_noop);
        assert_eq!(state.serialize().unwrap(), serialized);
        assert_eq!(state.mainline_projection().unwrap().moves, projection.moves);

        let root_edit = state
            .set_personal_comment(root.clone(), "root edited".to_string())
            .unwrap();
        assert_eq!(root_edit.selected_path.indices, root.indices);
        assert_eq!(root_edit.snapshot.personal_comment, "root edited");
        assert!(root_edit.dirty);

        let move_edit = state
            .set_personal_comment(move_node.clone(), "".to_string())
            .unwrap();
        assert_eq!(move_edit.selected_path.indices, move_node.indices);
        assert_eq!(move_edit.snapshot.personal_comment, "");

        let before_invalid = state.inspect();
        let error = state
            .set_personal_comment(NodePath { indices: vec![9] }, "nope".to_string())
            .unwrap_err();
        assert_eq!(error.kind, CurrentGameErrorKind::InvalidNodePath);
        assert_eq!(state.inspect(), before_invalid);
    }

    #[test]
    fn current_game_comment_edit_preserves_loaded_empty_comment_on_empty_submit() {
        let state = CurrentGameState::default();
        let opened = state.replace("(;GM[1]FF[4]SZ[5]C[])", None).unwrap();
        let root = NodePath { indices: Vec::new() };
        assert_eq!(opened.snapshot.personal_comment, "");
        let before = state.inspect();
        let serialized = state.serialize().unwrap();
        let noop = state.set_personal_comment(root, "".to_string()).unwrap();
        assert_eq!(noop.generation, opened.generation);
        assert!(!noop.dirty);
        assert_eq!(noop.snapshot.personal_comment, "");
        assert_eq!(state.inspect(), before);
        assert_eq!(state.serialize().unwrap(), serialized);
    }

    #[test]
    fn current_game_comment_edit_reports_no_current_game() {
        let state = CurrentGameState::default();
        let error = state
            .set_personal_comment(NodePath { indices: Vec::new() }, "note".to_string())
            .unwrap_err();
        assert_eq!(error.kind, CurrentGameErrorKind::NoCurrentGame);
    }
}

#[cfg(test)]
mod current_game_document_history {
    use super::*;
    use app_model::{CurrentGameErrorKind, MoveVertex, NodePath, PointDto};

    const BRANCHING: &str = include_str!("../../../../tests/golden/editable-workspace-branching.sgf");

    #[test]
    fn mixed_undo_redo_restores_exact_cursor_and_fences_stale_generations() {
        let state = CurrentGameState::default();
        let opened = state
            .replace(BRANCHING, Some("/tmp/source.sgf".to_string()))
            .unwrap();
        assert!(!opened.can_undo);
        assert!(!opened.can_redo);

        let comment_path = NodePath { indices: vec![0, 1] };
        let commented = state
            .set_personal_comment(comment_path.clone(), "history comment".to_string())
            .unwrap();
        assert_eq!(commented.generation, opened.generation);
        assert!(commented.can_undo);

        let played = state
            .play(
                NodePath { indices: vec![0] },
                MoveVertex::Point(PointDto { x: 1, y: 1 }),
            )
            .unwrap();
        let generation_before_undo = played.generation;
        let undone_play = state.undo(generation_before_undo).unwrap();
        assert!(undone_play.generation > generation_before_undo);
        assert_eq!(undone_play.selected_path, comment_path);
        assert!(undone_play.can_redo);

        let before_stale = state.inspect();
        let stale = state.undo(generation_before_undo).unwrap_err();
        assert_eq!(stale.kind, CurrentGameErrorKind::InvalidNodePath);
        assert_eq!(state.inspect(), before_stale);

        let undone_comment = state.undo(undone_play.generation).unwrap();
        assert_eq!(undone_comment.generation, undone_play.generation);
        assert_eq!(undone_comment.selected_path, opened.selected_path);
        assert!(!undone_comment.can_undo);
        assert!(undone_comment.can_redo);

        let redone_comment = state.redo(undone_comment.generation).unwrap();
        assert_eq!(redone_comment.generation, undone_comment.generation);
        assert_eq!(redone_comment.selected_path, comment_path);
        let redone_play = state.redo(redone_comment.generation).unwrap();
        assert!(redone_play.generation > redone_comment.generation);
        assert_eq!(redone_play.selected_path, played.selected_path);
        assert!(!redone_play.can_redo);
    }

    #[test]
    fn empty_history_is_unchanged_and_new_branch_or_replacement_clears_redo() {
        let state = CurrentGameState::default();
        let opened = state.replace(BRANCHING, None).unwrap();
        let empty = state.undo(opened.generation).unwrap();
        assert_eq!(empty, opened);

        let edited = state
            .set_personal_comment(NodePath::default(), "first".to_string())
            .unwrap();
        let undone = state.undo(edited.generation).unwrap();
        assert!(undone.can_redo);
        let branched = state
            .set_personal_comment(NodePath::default(), "branch".to_string())
            .unwrap();
        assert!(!branched.can_redo);

        let replaced = state.replace("(;GM[1]FF[4]SZ[9])", None).unwrap();
        assert!(!replaced.can_undo);
        assert!(!replaced.can_redo);
        assert_eq!(state.undo(replaced.generation).unwrap(), replaced);
    }

    #[test]
    fn pending_departure_rejects_history_without_mutation() {
        let state = CurrentGameState::default();
        let opened = state.replace(BRANCHING, None).unwrap();
        let edited = state
            .set_personal_comment(NodePath::default(), "pending".to_string())
            .unwrap();
        let before = state.serialize().unwrap();
        let admission = state.prepare_replacement("(;GM[1]FF[4]SZ[9])", None).unwrap();
        let departure_id = match admission {
            app_model::DocumentDepartureAdmissionDto::NeedsDecision { departure_id }
            | app_model::DocumentDepartureAdmissionDto::Ready { departure_id } => departure_id,
        };

        let error = state.undo(edited.generation).unwrap_err();
        assert_eq!(error.kind, CurrentGameErrorKind::DepartureInProgress);
        assert_eq!(state.serialize().unwrap(), before);
        state.cancel_replacement(departure_id).unwrap();
        let undone = state.undo(opened.generation).unwrap();
        assert_eq!(undone.snapshot.personal_comment, opened.snapshot.personal_comment);
    }
}
