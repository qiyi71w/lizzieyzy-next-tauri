use crate::current_game_state::CurrentGameState;
use crate::save_as;
use app_model::{
    AnalysisJobStartedDto, CurrentGameError, DocumentDepartureActionDto, DocumentDepartureAdmissionDto,
    DocumentDepartureOutcomeDto, ForegroundEngineSnapshotDto, NodePath,
};
use engine_manager::ForegroundEngineManager;
use save_as_dialog::persist_save_as;
use tauri::{AppHandle, State};

pub fn jobs_from_snapshot(snapshot: &ForegroundEngineSnapshotDto) -> Vec<AnalysisJobStartedDto> {
    let mut jobs = Vec::new();
    if let Some(job) = snapshot.selected_node_job.clone() {
        jobs.push(job);
    }
    if let Some(job) = snapshot.whole_game_job.clone() {
        jobs.push(job);
    }
    jobs
}

pub fn confirm_departure(
    state: &CurrentGameState,
    departure_id: u64,
    closed_jobs: &[AnalysisJobStartedDto],
    cancel_job: impl Fn(&AnalysisJobStartedDto),
) -> Result<(), CurrentGameError> {
    state.begin_protected_commit(departure_id, closed_jobs)?;
    for job in closed_jobs {
        cancel_job(job);
    }
    Ok(())
}

pub fn complete_save(
    state: &CurrentGameState,
    departure_id: u64,
    selected_path: NodePath,
    destination: Result<Option<String>, String>,
) -> Result<DocumentDepartureOutcomeDto, CurrentGameError> {
    match destination {
        Ok(None) => state.abort_protected_commit(departure_id, selected_path),
        Ok(Some(path)) => match state.save_sealed_departure(departure_id, path, selected_path.clone()) {
            Ok(_) => state.commit_replacement(departure_id),
            Err(error) => abort_save_failure(state, departure_id, selected_path, error),
        },
        Err(error) => abort_save_failure(state, departure_id, selected_path, error),
    }
}

pub fn resolve_replacement(
    state: &CurrentGameState,
    departure_id: u64,
    action: DocumentDepartureActionDto,
    selected_path: NodePath,
    closed_jobs: &[AnalysisJobStartedDto],
    cancel_job: impl Fn(&AnalysisJobStartedDto),
    save_destination: impl FnOnce() -> Result<Option<String>, String>,
) -> Result<DocumentDepartureOutcomeDto, CurrentGameError> {
    match action {
        DocumentDepartureActionDto::Cancel => state.cancel_replacement(departure_id),
        DocumentDepartureActionDto::Discard => {
            confirm_departure(state, departure_id, closed_jobs, cancel_job)?;
            state.commit_replacement(departure_id)
        }
        DocumentDepartureActionDto::Save => {
            confirm_departure(state, departure_id, closed_jobs, cancel_job)?;
            complete_save(state, departure_id, selected_path, save_destination())
        }
    }
}

fn abort_save_failure(
    state: &CurrentGameState,
    departure_id: u64,
    selected_path: NodePath,
    error: String,
) -> Result<DocumentDepartureOutcomeDto, CurrentGameError> {
    let mut outcome = state.abort_protected_commit(departure_id, selected_path)?;
    outcome.message = format!("{error} Analysis is stopped; restart it explicitly.");
    Ok(outcome)
}

#[tauri::command]
pub fn prepare_document_replacement(
    state: State<CurrentGameState>,
    sgf_text: String,
    native_path: Option<String>,
) -> Result<DocumentDepartureAdmissionDto, CurrentGameError> {
    state.prepare_replacement(&sgf_text, native_path)
}

#[tauri::command]
pub async fn resolve_document_replacement(
    app: AppHandle,
    state: State<'_, CurrentGameState>,
    manager: State<'_, ForegroundEngineManager>,
    departure_id: u64,
    action: DocumentDepartureActionDto,
    selected_path: NodePath,
    default_file_name: Option<String>,
) -> Result<DocumentDepartureOutcomeDto, CurrentGameError> {
    let closed_jobs = jobs_from_snapshot(&manager.snapshot());
    let cancel_job = |job: &AnalysisJobStartedDto| {
        let _ = manager.cancel_job(&job.run_id, &job.job_id);
    };
    if matches!(action, DocumentDepartureActionDto::Save) && state.native_path().is_none() {
        confirm_departure(&state, departure_id, &closed_jobs, cancel_job)?;
        let destination = pick_untitled_save_destination(app, default_file_name).await;
        return complete_save(&state, departure_id, selected_path, destination);
    }
    let save_path = state.native_path();
    resolve_replacement(
        &state,
        departure_id,
        action,
        selected_path,
        &closed_jobs,
        cancel_job,
        || Ok(save_path),
    )
}

async fn pick_untitled_save_destination(
    app: AppHandle,
    default_file_name: Option<String>,
) -> Result<Option<String>, String> {
    let default_file_name = default_file_name.unwrap_or_else(|| "review.sgf".to_string());
    let outcome =
        tauri::async_runtime::spawn_blocking(move || save_as::pick_save_as_outcome(&app, &default_file_name))
            .await
            .map_err(|error| error.to_string())??;
    persist_save_as(outcome, Ok::<String, String>)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::current_game_state::CurrentGameState;
    use app_model::{
        AnalysisJobLaneDto, AnalysisJobStartedDto, CurrentGameErrorKind, DocumentDepartureActionDto,
        DocumentDepartureAdmissionDto, ForegroundEngineLifecycleDto, ForegroundEngineSnapshotDto, MoveVertex,
        NodePath,
    };
    use std::cell::RefCell;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    const BRANCHING: &str = include_str!("../../../../tests/golden/editable-workspace-branching.sgf");
    const EMPTY: &str = "(;GM[1]FF[4]SZ[19]KM[7.5]PB[黑]PW[白])";

    fn unique_path(label: &str) -> std::path::PathBuf {
        let unique = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        std::env::temp_dir().join(format!("lizzieyzy-departure-coord-{label}-{unique}.sgf"))
    }

    fn job(run_id: &str, job_id: &str, lane: AnalysisJobLaneDto) -> AnalysisJobStartedDto {
        AnalysisJobStartedDto {
            run_id: run_id.to_string(),
            job_id: job_id.to_string(),
            lane,
            generation: 1,
            node_path: NodePath { indices: Vec::new() },
        }
    }

    fn departure_id(admission: DocumentDepartureAdmissionDto) -> u64 {
        match admission {
            DocumentDepartureAdmissionDto::NeedsDecision { departure_id }
            | DocumentDepartureAdmissionDto::Ready { departure_id } => departure_id,
        }
    }

    #[test]
    fn jobs_from_snapshot_collects_both_lanes() {
        let mut snapshot = ForegroundEngineSnapshotDto::with_lifecycle(
            1,
            ForegroundEngineLifecycleDto::NoEngine { failure: None },
        );
        snapshot.selected_node_job = Some(job("run-1", "selected", AnalysisJobLaneDto::SelectedNode));
        snapshot.whole_game_job = Some(job("run-1", "whole", AnalysisJobLaneDto::WholeGame));
        let jobs = jobs_from_snapshot(&snapshot);
        assert_eq!(jobs.len(), 2);
        assert_eq!(jobs[0].job_id, "selected");
        assert_eq!(jobs[1].job_id, "whole");
    }

    #[test]
    fn cancel_does_not_stop_jobs_or_commit() {
        let state = CurrentGameState::default();
        let opened = state
            .replace(BRANCHING, Some("/tmp/branching.sgf".to_string()))
            .unwrap();
        state.force_dirty();
        let id = departure_id(state.prepare_replacement(EMPTY, None).unwrap());
        let cancelled = RefCell::new(Vec::new());
        let outcome = resolve_replacement(
            &state,
            id,
            DocumentDepartureActionDto::Cancel,
            opened.selected_path.clone(),
            &[job("run-1", "job-1", AnalysisJobLaneDto::SelectedNode)],
            |job| cancelled.borrow_mut().push(job.job_id.clone()),
            || panic!("cancel must not save"),
        )
        .unwrap();
        assert!(!outcome.committed);
        assert!(!outcome.analysis_stopped);
        assert!(cancelled.borrow().is_empty());
        assert!(state.play(opened.selected_path, MoveVertex::Pass).is_ok());
    }

    #[test]
    fn discard_stops_jobs_and_commits_candidate() {
        let state = CurrentGameState::default();
        let opened = state
            .replace(BRANCHING, Some("/tmp/branching.sgf".to_string()))
            .unwrap();
        state.force_dirty();
        let id = departure_id(state.prepare_replacement(EMPTY, None).unwrap());
        let cancelled = RefCell::new(Vec::new());
        let outcome = resolve_replacement(
            &state,
            id,
            DocumentDepartureActionDto::Discard,
            opened.selected_path,
            &[job("run-1", "job-1", AnalysisJobLaneDto::SelectedNode)],
            |job| cancelled.borrow_mut().push(job.job_id.clone()),
            || panic!("discard must not save"),
        )
        .unwrap();
        assert!(outcome.committed);
        assert!(outcome.analysis_stopped);
        assert_eq!(cancelled.borrow().as_slice(), ["job-1"]);
        let current = outcome.current.unwrap();
        assert!(current.selected_path.indices.is_empty());
        assert!(current.native_path.is_none());
        assert!(!current.dirty);
    }

    #[test]
    fn save_as_cancel_keeps_document_and_leaves_analysis_stopped() {
        let state = CurrentGameState::default();
        let opened = state
            .replace(BRANCHING, Some("/tmp/branching.sgf".to_string()))
            .unwrap();
        state.force_dirty();
        let generation = opened.generation;
        let id = departure_id(state.prepare_replacement(EMPTY, None).unwrap());
        let cancelled = RefCell::new(Vec::new());
        let outcome = resolve_replacement(
            &state,
            id,
            DocumentDepartureActionDto::Save,
            opened.selected_path.clone(),
            &[job("run-1", "job-1", AnalysisJobLaneDto::SelectedNode)],
            |job| cancelled.borrow_mut().push(job.job_id.clone()),
            || Ok(None),
        )
        .unwrap();
        assert!(!outcome.committed);
        assert!(outcome.analysis_stopped);
        assert_eq!(cancelled.borrow().as_slice(), ["job-1"]);
        let current = outcome.current.unwrap();
        assert_eq!(current.generation, generation);
        assert!(current.dirty);
        assert_eq!(current.native_path.as_deref(), Some("/tmp/branching.sgf"));
        assert!(state.play(opened.selected_path, MoveVertex::Pass).is_ok());
    }

    #[test]
    fn failed_save_restores_editing_with_restart_feedback() {
        let state = CurrentGameState::default();
        let opened = state
            .replace(BRANCHING, Some("/tmp/branching.sgf".to_string()))
            .unwrap();
        state.force_dirty();
        let id = departure_id(state.prepare_replacement(EMPTY, None).unwrap());
        let outcome = resolve_replacement(
            &state,
            id,
            DocumentDepartureActionDto::Save,
            opened.selected_path.clone(),
            &[],
            |_| {},
            || Err("disk full".to_string()),
        )
        .unwrap();
        assert!(!outcome.committed);
        assert!(outcome.analysis_stopped);
        assert!(outcome.message.contains("disk full"));
        assert!(outcome.message.contains("restart"));
        assert!(state.play(opened.selected_path, MoveVertex::Pass).is_ok());
    }

    #[test]
    fn save_writes_sealed_snapshot_then_commits_candidate() {
        let state = CurrentGameState::default();
        let source = unique_path("sealed-source");
        let opened = state
            .replace(BRANCHING, Some(source.to_string_lossy().into_owned()))
            .unwrap();
        state.force_dirty();
        let id = departure_id(state.prepare_replacement(EMPTY, None).unwrap());
        let outcome = resolve_replacement(
            &state,
            id,
            DocumentDepartureActionDto::Save,
            opened.selected_path,
            &[],
            |_| {},
            || Ok(Some(source.to_string_lossy().into_owned())),
        )
        .unwrap();
        let written = fs::read_to_string(&source).unwrap();
        let _ = fs::remove_file(&source);
        assert!(outcome.committed);
        assert!(written.contains("mainline pass"));
        assert!(!written.contains("PB[黑]"));
        let current = outcome.current.unwrap();
        assert!(current.native_path.is_none());
        assert!(current.selected_path.indices.is_empty());
    }

    #[test]
    fn force_replace_is_rejected_while_departure_is_open() {
        let state = CurrentGameState::default();
        state.replace(BRANCHING, None).unwrap();
        state.force_dirty();
        let _ = state.prepare_replacement(EMPTY, None).unwrap();
        let error = state.replace(EMPTY, None).unwrap_err();
        assert_eq!(error.kind, CurrentGameErrorKind::DepartureInProgress);
    }
}
