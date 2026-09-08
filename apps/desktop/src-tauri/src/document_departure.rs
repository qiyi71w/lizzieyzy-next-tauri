use crate::current_game_state::{recovery, CurrentGameState};
use crate::save_as;
use app_model::{
    AnalysisJobStartedDto, ApplicationExitActionDto, ApplicationExitDispositionDto,
    ApplicationExitOutcomeDto, ApplicationTeardownAttemptDto, CurrentGameError, DocumentDepartureActionDto,
    DocumentDepartureAdmissionDto, DocumentDepartureOutcomeDto, ForegroundEngineSnapshotDto, NodePath,
};
use current_game_recovery::FileRecoveryStore;
use engine_manager::ForegroundEngineManager;
use save_as_dialog::persist_save_as;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, State};

pub fn jobs_from_snapshot(snapshot: &ForegroundEngineSnapshotDto) -> Vec<AnalysisJobStartedDto> {
    let mut jobs = Vec::new();
    if let Some(job) = snapshot
        .selected_node_job
        .clone()
        .filter(|job| job.state != app_model::AnalysisJobStateDto::TimeLimited)
    {
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
    cancel_job: impl Fn(&AnalysisJobStartedDto) -> Result<(), String>,
    wait_for_job: impl Fn(&AnalysisJobStartedDto, Duration) -> Result<(), String>,
    budget: Duration,
) -> Result<(), String> {
    state
        .begin_protected_commit(departure_id, closed_jobs)
        .map_err(|error| error.to_string())?;
    // Suppression is established by begin_protected_commit before taking this snapshot.
    // Include jobs admitted after the caller's earlier snapshot, before the cutoff.
    let owned_jobs = state.analysis_jobs();
    let closed_jobs = owned_jobs.as_deref().unwrap_or(closed_jobs);
    for job in closed_jobs {
        state.seal_job(job);
    }
    let deadline = Instant::now() + budget;
    let mut delivery_error = None;
    for job in closed_jobs {
        if let Err(error) = cancel_job(job) {
            delivery_error.get_or_insert(error);
        }
    }
    if let Some(error) = delivery_error {
        return Err(error);
    }
    for job in closed_jobs {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(format!("timed out while stopping analysis job {}", job.job_id));
        }
        wait_for_job(job, remaining)?;
    }
    Ok(())
}

fn abort_cancellation_failure(
    state: &CurrentGameState,
    departure_id: u64,
    selected_path: NodePath,
    error: String,
) -> Result<DocumentDepartureOutcomeDto, CurrentGameError> {
    let mut outcome = state.abort_protected_commit(departure_id, selected_path)?;
    outcome.message = format!("Failed to stop analysis: {error}");
    Ok(outcome)
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
    cancel_job: impl Fn(&AnalysisJobStartedDto) -> Result<(), String>,
    wait_for_job: impl Fn(&AnalysisJobStartedDto, Duration) -> Result<(), String>,
    save_destination: impl FnOnce() -> Result<Option<String>, String>,
) -> Result<DocumentDepartureOutcomeDto, CurrentGameError> {
    match action {
        DocumentDepartureActionDto::Cancel => state.cancel_replacement(departure_id),
        DocumentDepartureActionDto::Discard | DocumentDepartureActionDto::Save => {
            if let Err(error) = confirm_departure(
                state,
                departure_id,
                closed_jobs,
                &cancel_job,
                &wait_for_job,
                APPLICATION_TEARDOWN_BUDGET,
            ) {
                return abort_cancellation_failure(state, departure_id, selected_path, error);
            }
            if matches!(action, DocumentDepartureActionDto::Discard) {
                state.commit_replacement(departure_id)
            } else {
                complete_save(state, departure_id, selected_path, save_destination())
            }
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

pub const APPLICATION_TEARDOWN_BUDGET: Duration = Duration::from_secs(10);
pub const APPLICATION_EXIT_REQUESTED_EVENT: &str = "application-exit-requested";

pub fn attempt_teardown(
    budget: Duration,
    stop_owned: impl FnOnce(Duration) -> Vec<String>,
) -> ApplicationTeardownAttemptDto {
    let outstanding = stop_owned(budget);
    if outstanding.is_empty() {
        ApplicationTeardownAttemptDto::Completed
    } else {
        ApplicationTeardownAttemptDto::TimedOut { outstanding }
    }
}

pub fn stop_foreground_resources(manager: &ForegroundEngineManager, budget: Duration) -> Vec<String> {
    let manager = manager.clone();
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let teardown = manager.teardown();
        let snapshot = manager.snapshot();
        let mut outstanding = Vec::new();
        if snapshot.selected_node_job.is_some() {
            outstanding.push("selected-node analysis".to_string());
        }
        if snapshot.whole_game_job.is_some() {
            outstanding.push("whole-game analysis".to_string());
        }
        if !matches!(
            snapshot.lifecycle,
            app_model::ForegroundEngineLifecycleDto::NoEngine { .. }
        ) {
            outstanding.push("foreground engine".to_string());
        }
        if let Err(failure) = teardown {
            outstanding.push(format!("foreground engine cleanup failed: {}", failure.message));
        }
        let _ = tx.send(outstanding);
    });
    match rx.recv_timeout(budget) {
        Ok(outstanding) => outstanding,
        Err(_) => vec!["foreground engine".to_string()],
    }
}

fn exit_from_departure(
    outcome: DocumentDepartureOutcomeDto,
    disposition: Option<ApplicationExitDispositionDto>,
    teardown: Option<ApplicationTeardownAttemptDto>,
) -> ApplicationExitOutcomeDto {
    ApplicationExitOutcomeDto {
        committed: outcome.committed,
        analysis_stopped: outcome.analysis_stopped,
        current: outcome.current,
        message: outcome.message,
        disposition,
        teardown,
        recovery_persist_error: None,
    }
}

fn complete_exit(
    state: &CurrentGameState,
    departure_id: u64,
    selected_path: NodePath,
    disposition: ApplicationExitDispositionDto,
    teardown: impl FnOnce(Duration) -> Vec<String>,
    budget: Duration,
) -> Result<ApplicationExitOutcomeDto, CurrentGameError> {
    state.begin_application_teardown(departure_id, selected_path.clone(), disposition)?;
    let attempt = attempt_teardown(budget, teardown);
    state.finish_application_teardown(departure_id, selected_path, attempt, false)
}

fn finish_exit_save(
    state: &CurrentGameState,
    departure_id: u64,
    selected_path: NodePath,
    destination: Result<Option<String>, String>,
    teardown: impl FnOnce(Duration) -> Vec<String>,
    budget: Duration,
) -> Result<ApplicationExitOutcomeDto, CurrentGameError> {
    match destination {
        Ok(None) => Ok(exit_from_departure(
            state.abort_protected_commit(departure_id, selected_path)?,
            None,
            None,
        )),
        Ok(Some(path)) => match state.save_sealed_departure(departure_id, path, selected_path.clone()) {
            Ok(_) => complete_exit(
                state,
                departure_id,
                selected_path,
                ApplicationExitDispositionDto::ExitIncomplete,
                teardown,
                budget,
            ),
            Err(error) => Ok(exit_from_departure(
                abort_save_failure(state, departure_id, selected_path, error)?,
                None,
                None,
            )),
        },
        Err(error) => Ok(exit_from_departure(
            abort_save_failure(state, departure_id, selected_path, error)?,
            None,
            None,
        )),
    }
}

pub fn resolve_exit(
    state: &CurrentGameState,
    departure_id: u64,
    action: ApplicationExitActionDto,
    selected_path: NodePath,
    closed_jobs: &[AnalysisJobStartedDto],
    cancel_job: impl Fn(&AnalysisJobStartedDto) -> Result<(), String>,
    wait_for_job: impl Fn(&AnalysisJobStartedDto, Duration) -> Result<(), String>,
    save_destination: impl FnOnce() -> Result<Option<String>, String>,
    teardown: impl FnOnce(Duration) -> Vec<String>,
    budget: Duration,
) -> Result<ApplicationExitOutcomeDto, CurrentGameError> {
    if matches!(action, ApplicationExitActionDto::Cancel) {
        let cancelled = state.cancel_replacement(departure_id)?;
        return Ok(ApplicationExitOutcomeDto {
            committed: false,
            analysis_stopped: false,
            current: cancelled.current,
            message: "Exit cancelled.".to_string(),
            disposition: None,
            teardown: None,
            recovery_persist_error: None,
        });
    }

    let deadline = Instant::now() + budget;
    if let Err(error) = confirm_departure(
        state,
        departure_id,
        closed_jobs,
        &cancel_job,
        &wait_for_job,
        deadline.saturating_duration_since(Instant::now()),
    ) {
        return Ok(exit_from_departure(
            abort_cancellation_failure(state, departure_id, selected_path, error)?,
            None,
            None,
        ));
    }
    let remaining = || deadline.saturating_duration_since(Instant::now());
    match action {
        ApplicationExitActionDto::Discard => complete_exit(
            state,
            departure_id,
            selected_path,
            ApplicationExitDispositionDto::ExplicitDiscard,
            teardown,
            remaining(),
        ),
        ApplicationExitActionDto::Continue => complete_exit(
            state,
            departure_id,
            selected_path,
            ApplicationExitDispositionDto::ExitIncomplete,
            teardown,
            remaining(),
        ),
        ApplicationExitActionDto::Save => finish_exit_save(
            state,
            departure_id,
            selected_path,
            save_destination(),
            teardown,
            remaining(),
        ),
        ApplicationExitActionDto::Cancel => unreachable!(),
    }
}

pub fn retry_exit_teardown(
    state: &CurrentGameState,
    departure_id: u64,
    selected_path: NodePath,
    teardown: impl FnOnce(Duration) -> Vec<String>,
    budget: Duration,
) -> Result<ApplicationExitOutcomeDto, CurrentGameError> {
    let attempt = attempt_teardown(budget, teardown);
    state.finish_application_teardown(departure_id, selected_path, attempt, false)
}

pub fn exit_anyway(
    state: &CurrentGameState,
    departure_id: u64,
    selected_path: NodePath,
    outstanding: Vec<String>,
) -> Result<ApplicationExitOutcomeDto, CurrentGameError> {
    state.finish_application_teardown(
        departure_id,
        selected_path,
        ApplicationTeardownAttemptDto::TimedOut { outstanding },
        true,
    )
}

fn persist_exit_outcome(
    state: &CurrentGameState,
    store: &Mutex<FileRecoveryStore>,
    outcome: ApplicationExitOutcomeDto,
) -> ApplicationExitOutcomeDto {
    let store = store.lock().expect("recovery store");
    recovery::persist_committed_exit(state, &store, outcome)
}

fn persist_replacement_outcome(
    app: &AppHandle,
    state: &CurrentGameState,
    store: &Mutex<FileRecoveryStore>,
    outcome: DocumentDepartureOutcomeDto,
) -> DocumentDepartureOutcomeDto {
    if outcome.committed {
        let store = store.lock().expect("recovery store");
        let _ = recovery::persist_replacement_snapshot(state, &store);
        let _ = app.emit(
            crate::session_recovery::RECOVERY_PROTECTION_EVENT,
            state.recovery_protection(),
        );
    }
    outcome
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
    store: State<'_, Mutex<FileRecoveryStore>>,
    departure_id: u64,
    action: DocumentDepartureActionDto,
    selected_path: NodePath,
    default_file_name: Option<String>,
) -> Result<DocumentDepartureOutcomeDto, CurrentGameError> {
    let closed_jobs = jobs_from_snapshot(&manager.snapshot());
    let cancel_job = |job: &AnalysisJobStartedDto| {
        manager
            .cancel_job(&job.run_id, &job.job_id)
            .map_err(|failure| failure.message)
    };
    let wait_for_job = |job: &AnalysisJobStartedDto, budget: Duration| {
        manager
            .wait_for_job_cancellation(&job.run_id, &job.job_id, budget)
            .map_err(|failure| failure.message)
    };
    let outcome = if matches!(action, DocumentDepartureActionDto::Save) && state.native_path().is_none() {
        if let Err(error) = confirm_departure(
            &state,
            departure_id,
            &closed_jobs,
            cancel_job,
            wait_for_job,
            APPLICATION_TEARDOWN_BUDGET,
        ) {
            abort_cancellation_failure(&state, departure_id, selected_path, error)?
        } else {
            let destination = pick_untitled_save_destination(app.clone(), default_file_name).await;
            complete_save(&state, departure_id, selected_path, destination)?
        }
    } else {
        let save_path = state.native_path();
        resolve_replacement(
            &state,
            departure_id,
            action,
            selected_path,
            &closed_jobs,
            cancel_job,
            wait_for_job,
            || Ok(save_path),
        )?
    };
    Ok(persist_replacement_outcome(&app, &state, &store, outcome))
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

#[tauri::command]
pub fn prepare_application_exit(
    state: State<CurrentGameState>,
) -> Result<DocumentDepartureAdmissionDto, CurrentGameError> {
    state.prepare_exit()
}

#[tauri::command]
pub async fn resolve_application_exit(
    app: AppHandle,
    state: State<'_, CurrentGameState>,
    manager: State<'_, ForegroundEngineManager>,
    store: State<'_, Mutex<FileRecoveryStore>>,
    departure_id: u64,
    action: ApplicationExitActionDto,
    selected_path: NodePath,
    default_file_name: Option<String>,
) -> Result<ApplicationExitOutcomeDto, CurrentGameError> {
    let closed_jobs = jobs_from_snapshot(&manager.snapshot());
    let cancel_job = |job: &AnalysisJobStartedDto| {
        manager
            .cancel_job(&job.run_id, &job.job_id)
            .map_err(|failure| failure.message)
    };
    let wait_for_job = |job: &AnalysisJobStartedDto, budget: Duration| {
        manager
            .wait_for_job_cancellation(&job.run_id, &job.job_id, budget)
            .map_err(|failure| failure.message)
    };
    let outcome = if matches!(action, ApplicationExitActionDto::Save) && state.native_path().is_none() {
        let deadline = Instant::now() + APPLICATION_TEARDOWN_BUDGET;
        if let Err(error) = confirm_departure(
            &state,
            departure_id,
            &closed_jobs,
            cancel_job,
            wait_for_job,
            deadline.saturating_duration_since(Instant::now()),
        ) {
            exit_from_departure(
                abort_cancellation_failure(&state, departure_id, selected_path, error)?,
                None,
                None,
            )
        } else {
            let destination = pick_untitled_save_destination(app.clone(), default_file_name).await;
            finish_exit_save(
                &state,
                departure_id,
                selected_path,
                destination,
                |budget| stop_foreground_resources(&manager, budget),
                deadline.saturating_duration_since(Instant::now()),
            )?
        }
    } else {
        let save_path = state.native_path();
        resolve_exit(
            &state,
            departure_id,
            action,
            selected_path,
            &closed_jobs,
            cancel_job,
            wait_for_job,
            || Ok(save_path),
            |budget| stop_foreground_resources(&manager, budget),
            APPLICATION_TEARDOWN_BUDGET,
        )?
    };
    Ok(persist_exit_outcome(&state, &store, outcome))
}

#[tauri::command]
pub fn retry_application_teardown(
    state: State<CurrentGameState>,
    manager: State<ForegroundEngineManager>,
    store: State<Mutex<FileRecoveryStore>>,
    departure_id: u64,
    selected_path: NodePath,
) -> Result<ApplicationExitOutcomeDto, CurrentGameError> {
    let outcome = retry_exit_teardown(
        &state,
        departure_id,
        selected_path,
        |budget| stop_foreground_resources(&manager, budget),
        APPLICATION_TEARDOWN_BUDGET,
    )?;
    Ok(persist_exit_outcome(&state, &store, outcome))
}

#[tauri::command]
pub fn confirm_application_exit_anyway(
    state: State<CurrentGameState>,
    store: State<Mutex<FileRecoveryStore>>,
    departure_id: u64,
    selected_path: NodePath,
    outstanding: Vec<String>,
) -> Result<ApplicationExitOutcomeDto, CurrentGameError> {
    let outcome = exit_anyway(&state, departure_id, selected_path, outstanding)?;
    Ok(persist_exit_outcome(&state, &store, outcome))
}

#[tauri::command]
pub fn confirm_native_exit(app: AppHandle) {
    app.exit(0);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::current_game_state::CurrentGameState;
    use app_model::{
        AnalysisJobLaneDto, AnalysisJobModeDto, AnalysisJobStartedDto, AnalysisJobStateDto,
        ApplicationExitActionDto, ApplicationExitDispositionDto, ApplicationTeardownAttemptDto,
        CurrentGameErrorKind, DocumentDepartureActionDto, DocumentDepartureAdmissionDto,
        ForegroundEngineLifecycleDto, ForegroundEngineSnapshotDto, MoveVertex, NodePath,
    };
    use std::cell::RefCell;
    use std::fs;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    const BRANCHING: &str = include_str!("../../../../tests/golden/editable-workspace-branching.sgf");
    const EMPTY: &str = "(;GM[1]FF[4]SZ[19]KM[7.5]PB[黑]PW[白])";

    fn unique_path(label: &str) -> std::path::PathBuf {
        let unique = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        std::env::temp_dir().join(format!("lizzieyzy-departure-coord-{label}-{unique}.sgf"))
    }

    fn wait_succeeds(_: &AnalysisJobStartedDto, _: Duration) -> Result<(), String> {
        Ok(())
    }

    fn job(run_id: &str, job_id: &str, lane: AnalysisJobLaneDto) -> AnalysisJobStartedDto {
        AnalysisJobStartedDto {
            run_id: run_id.to_string(),
            job_id: job_id.to_string(),
            lane,
            mode: if lane == AnalysisJobLaneDto::SelectedNode {
                AnalysisJobModeDto::Continuous
            } else {
                AnalysisJobModeDto::Finite
            },
            state: AnalysisJobStateDto::Searching,
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
            |job| {
                cancelled.borrow_mut().push(job.job_id.clone());
                Ok(())
            },
            wait_succeeds,
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
            |job| {
                cancelled.borrow_mut().push(job.job_id.clone());
                Ok(())
            },
            wait_succeeds,
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
            |job| {
                cancelled.borrow_mut().push(job.job_id.clone());
                Ok(())
            },
            wait_succeeds,
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
            |_| Ok(()),
            wait_succeeds,
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
            |_| Ok(()),
            wait_succeeds,
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

    #[test]
    fn exit_cancel_does_not_stop_jobs_or_teardown() {
        let state = CurrentGameState::default();
        let opened = state
            .replace(BRANCHING, Some("/tmp/branching.sgf".to_string()))
            .unwrap();
        state.force_dirty();
        let id = departure_id(state.prepare_exit().unwrap());
        let cancelled = RefCell::new(Vec::new());
        let torn_down = RefCell::new(false);
        let outcome = resolve_exit(
            &state,
            id,
            ApplicationExitActionDto::Cancel,
            opened.selected_path.clone(),
            &[job("run-1", "job-1", AnalysisJobLaneDto::SelectedNode)],
            |job| {
                cancelled.borrow_mut().push(job.job_id.clone());
                Ok(())
            },
            wait_succeeds,
            || panic!("cancel must not save"),
            |_| {
                *torn_down.borrow_mut() = true;
                Vec::new()
            },
            Duration::from_secs(10),
        )
        .unwrap();
        assert!(!outcome.committed);
        assert!(!outcome.analysis_stopped);
        assert!(outcome.disposition.is_none());
        assert!(cancelled.borrow().is_empty());
        assert!(!*torn_down.borrow());
        assert!(state.play(opened.selected_path, MoveVertex::Pass).is_ok());
    }

    #[test]
    fn exit_save_as_cancel_keeps_window_and_does_not_teardown() {
        let state = CurrentGameState::default();
        let opened = state
            .replace(BRANCHING, Some("/tmp/branching.sgf".to_string()))
            .unwrap();
        state.force_dirty();
        let id = departure_id(state.prepare_exit().unwrap());
        let cancelled = RefCell::new(Vec::new());
        let torn_down = RefCell::new(false);
        let outcome = resolve_exit(
            &state,
            id,
            ApplicationExitActionDto::Save,
            opened.selected_path.clone(),
            &[job("run-1", "job-1", AnalysisJobLaneDto::SelectedNode)],
            |job| {
                cancelled.borrow_mut().push(job.job_id.clone());
                Ok(())
            },
            wait_succeeds,
            || Ok(None),
            |_| {
                *torn_down.borrow_mut() = true;
                Vec::new()
            },
            Duration::from_secs(10),
        )
        .unwrap();
        assert!(!outcome.committed);
        assert!(outcome.analysis_stopped);
        assert!(outcome.disposition.is_none());
        assert_eq!(cancelled.borrow().as_slice(), ["job-1"]);
        assert!(!*torn_down.borrow());
        let current = outcome.current.unwrap();
        assert!(current.dirty);
        assert!(state.play(opened.selected_path, MoveVertex::Pass).is_ok());
    }

    #[test]
    fn exit_failed_save_does_not_teardown_engine() {
        let state = CurrentGameState::default();
        let opened = state
            .replace(BRANCHING, Some("/tmp/branching.sgf".to_string()))
            .unwrap();
        state.force_dirty();
        let id = departure_id(state.prepare_exit().unwrap());
        let torn_down = RefCell::new(false);
        let outcome = resolve_exit(
            &state,
            id,
            ApplicationExitActionDto::Save,
            opened.selected_path.clone(),
            &[],
            |_| Ok(()),
            wait_succeeds,
            || Err("disk full".to_string()),
            |_| {
                *torn_down.borrow_mut() = true;
                Vec::new()
            },
            Duration::from_secs(10),
        )
        .unwrap();
        assert!(!outcome.committed);
        assert!(outcome.analysis_stopped);
        assert!(outcome.disposition.is_none());
        assert!(!*torn_down.borrow());
        assert!(outcome.message.contains("disk full"));
        assert!(state.play(opened.selected_path, MoveVertex::Pass).is_ok());
    }

    #[test]
    fn clean_exit_and_discard_report_distinct_dispositions() {
        let clean = CurrentGameState::default();
        let opened = clean
            .replace(BRANCHING, Some("/tmp/branching.sgf".to_string()))
            .unwrap();
        let clean_id = departure_id(clean.prepare_exit().unwrap());
        let clean_outcome = resolve_exit(
            &clean,
            clean_id,
            ApplicationExitActionDto::Continue,
            opened.selected_path.clone(),
            &[job("run-1", "job-1", AnalysisJobLaneDto::SelectedNode)],
            |_| Ok(()),
            wait_succeeds,
            || panic!("clean exit must not save"),
            |_| Vec::new(),
            Duration::from_secs(10),
        )
        .unwrap();
        assert!(clean_outcome.committed);
        assert_eq!(
            clean_outcome.disposition,
            Some(ApplicationExitDispositionDto::CleanCompleted)
        );
        assert_eq!(
            clean_outcome.teardown,
            Some(ApplicationTeardownAttemptDto::Completed)
        );
        assert_eq!(
            clean.application_exit_disposition(),
            Some(ApplicationExitDispositionDto::CleanCompleted)
        );

        let dirty = CurrentGameState::default();
        let source = unique_path("exit-discard-coord");
        fs::write(&source, BRANCHING).unwrap();
        let opened = dirty
            .replace(BRANCHING, Some(source.to_string_lossy().into_owned()))
            .unwrap();
        dirty.force_dirty();
        let before = fs::read_to_string(&source).unwrap();
        let dirty_id = departure_id(dirty.prepare_exit().unwrap());
        let discarded = resolve_exit(
            &dirty,
            dirty_id,
            ApplicationExitActionDto::Discard,
            opened.selected_path,
            &[job("run-1", "job-1", AnalysisJobLaneDto::SelectedNode)],
            |_| Ok(()),
            wait_succeeds,
            || panic!("discard must not save"),
            |_| Vec::new(),
            Duration::from_secs(10),
        )
        .unwrap();
        assert_eq!(fs::read_to_string(&source).unwrap(), before);
        let _ = fs::remove_file(&source);
        assert_eq!(
            discarded.disposition,
            Some(ApplicationExitDispositionDto::ExplicitDiscard)
        );
        assert_eq!(
            dirty.application_exit_disposition(),
            Some(ApplicationExitDispositionDto::ExplicitDiscard)
        );
    }

    #[test]
    fn duplicate_exit_does_not_start_a_second_transaction() {
        let state = CurrentGameState::default();
        state.replace(BRANCHING, None).unwrap();
        state.force_dirty();
        let first = state.prepare_exit().unwrap();
        let second = state.prepare_exit().unwrap_err();
        assert_eq!(second.kind, CurrentGameErrorKind::DepartureInProgress);
        let replacement = state.prepare_replacement(EMPTY, None).unwrap_err();
        assert_eq!(replacement.kind, CurrentGameErrorKind::DepartureInProgress);
        let _ = departure_id(first);
    }

    #[test]
    fn teardown_timeout_names_resources_and_retry_is_another_bounded_attempt() {
        let state = CurrentGameState::default();
        let opened = state.replace(BRANCHING, None).unwrap();
        let id = departure_id(state.prepare_exit().unwrap());
        let attempts = RefCell::new(0_u8);
        let outcome = resolve_exit(
            &state,
            id,
            ApplicationExitActionDto::Continue,
            opened.selected_path.clone(),
            &[],
            |_| Ok(()),
            wait_succeeds,
            || panic!("clean exit must not save"),
            |_| {
                *attempts.borrow_mut() += 1;
                vec!["foreground engine".to_string()]
            },
            Duration::from_secs(10),
        )
        .unwrap();
        assert_eq!(
            outcome.disposition,
            Some(ApplicationExitDispositionDto::ExitIncomplete)
        );
        match outcome.teardown {
            Some(ApplicationTeardownAttemptDto::TimedOut { outstanding }) => {
                assert_eq!(outstanding, ["foreground engine"]);
            }
            other => panic!("expected timeout, got {other:?}"),
        }
        assert_eq!(*attempts.borrow(), 1);
        assert_eq!(
            state.prepare_exit().unwrap_err().kind,
            CurrentGameErrorKind::DepartureInProgress
        );

        let retried = retry_exit_teardown(
            &state,
            id,
            opened.selected_path.clone(),
            |_| {
                *attempts.borrow_mut() += 1;
                Vec::new()
            },
            Duration::from_secs(10),
        )
        .unwrap();
        assert_eq!(*attempts.borrow(), 2);
        assert_eq!(
            retried.disposition,
            Some(ApplicationExitDispositionDto::CleanCompleted)
        );
        assert_eq!(retried.teardown, Some(ApplicationTeardownAttemptDto::Completed));
    }

    #[test]
    fn exit_anyway_keeps_incomplete_disposition_unless_discarded() {
        let incomplete = CurrentGameState::default();
        let opened = incomplete.replace(BRANCHING, None).unwrap();
        let id = departure_id(incomplete.prepare_exit().unwrap());
        resolve_exit(
            &incomplete,
            id,
            ApplicationExitActionDto::Continue,
            opened.selected_path.clone(),
            &[],
            |_| Ok(()),
            wait_succeeds,
            || panic!("clean exit must not save"),
            |_| vec!["foreground engine".to_string()],
            Duration::from_secs(10),
        )
        .unwrap();
        let forced = exit_anyway(
            &incomplete,
            id,
            opened.selected_path,
            vec!["foreground engine".to_string()],
        )
        .unwrap();
        assert_eq!(
            forced.disposition,
            Some(ApplicationExitDispositionDto::ExitIncomplete)
        );
        assert_eq!(
            incomplete.application_exit_disposition(),
            Some(ApplicationExitDispositionDto::ExitIncomplete)
        );
    }
}

#[cfg(test)]
mod cancellation_failure_tests {
    use super::*;
    use crate::current_game_state::CurrentGameState;
    use app_model::{
        AnalysisJobLaneDto, AnalysisJobModeDto, AnalysisJobStartedDto, AnalysisJobStateDto,
        ApplicationExitActionDto, DocumentDepartureActionDto, DocumentDepartureAdmissionDto, NodePath,
    };
    use std::cell::{Cell, RefCell};

    const SOURCE: &str = "(;GM[1]FF[4]SZ[19]KM[7.5]C[personal])";
    const CANDIDATE: &str = "(;GM[1]FF[4]SZ[19]KM[7.5]PB[new])";

    fn departure_id(admission: DocumentDepartureAdmissionDto) -> u64 {
        match admission {
            DocumentDepartureAdmissionDto::NeedsDecision { departure_id }
            | DocumentDepartureAdmissionDto::Ready { departure_id } => departure_id,
        }
    }

    fn continuous_job() -> AnalysisJobStartedDto {
        AnalysisJobStartedDto {
            run_id: "run-1".to_string(),
            job_id: "job-1".to_string(),
            lane: AnalysisJobLaneDto::SelectedNode,
            mode: AnalysisJobModeDto::Continuous,
            state: AnalysisJobStateDto::Searching,
            generation: 1,
            node_path: NodePath { indices: Vec::new() },
        }
    }

    #[test]
    fn cancellation_failure_aborts_replacement_without_save_or_install() {
        let state = CurrentGameState::default();
        let opened = state
            .replace(SOURCE, Some("/tmp/source.sgf".to_string()))
            .unwrap();
        state.force_dirty();
        let id = departure_id(state.prepare_replacement(CANDIDATE, None).unwrap());
        let save_called = Cell::new(false);

        let outcome = resolve_replacement(
            &state,
            id,
            DocumentDepartureActionDto::Save,
            opened.selected_path.clone(),
            &[continuous_job()],
            |_| Ok(()),
            |_, _| Err("target final did not arrive".to_string()),
            || {
                save_called.set(true);
                Ok(Some("/tmp/must-not-write.sgf".to_string()))
            },
        )
        .unwrap();

        assert!(!outcome.committed);
        assert!(outcome.analysis_stopped);
        assert!(outcome.message.contains("target final did not arrive"));
        assert!(!save_called.get());
        assert!(state.serialize().unwrap().contains("C[personal]"));
        assert!(!state.serialize().unwrap().contains("PB[new]"));
    }

    #[test]
    fn confirmed_exit_waits_for_stop_before_save_and_failure_skips_all_finalization() {
        let state = CurrentGameState::default();
        let opened = state
            .replace(SOURCE, Some("/tmp/source.sgf".to_string()))
            .unwrap();
        state.force_dirty();
        let id = departure_id(state.prepare_exit().unwrap());
        let order = RefCell::new(Vec::new());

        let outcome = resolve_exit(
            &state,
            id,
            ApplicationExitActionDto::Save,
            opened.selected_path,
            &[continuous_job()],
            |_| {
                order.borrow_mut().push("cancel");
                Ok(())
            },
            |_, _| {
                order.borrow_mut().push("wait");
                Err("cleanup failed".to_string())
            },
            || {
                order.borrow_mut().push("save");
                Ok(Some("/tmp/must-not-write.sgf".to_string()))
            },
            |_| {
                order.borrow_mut().push("teardown");
                Vec::new()
            },
            APPLICATION_TEARDOWN_BUDGET,
        )
        .unwrap();

        assert_eq!(order.into_inner(), ["cancel", "wait"]);
        assert!(!outcome.committed);
        assert!(outcome.analysis_stopped);
        assert!(outcome.message.contains("cleanup failed"));
        assert!(outcome.disposition.is_none());
        assert!(outcome.teardown.is_none());
        assert!(state.serialize().unwrap().contains("C[personal]"));
    }

    #[test]
    fn confirmed_replacement_waits_for_stopped_job_before_saving() {
        let state = CurrentGameState::default();
        let opened = state.replace(SOURCE, None).unwrap();
        state.force_dirty();
        let id = departure_id(state.prepare_replacement(CANDIDATE, None).unwrap());
        let order = RefCell::new(Vec::new());
        let path = std::env::temp_dir().join(format!(
            "lizzieyzy-stopped-before-save-{}.sgf",
            uuid::Uuid::new_v4()
        ));

        let outcome = resolve_replacement(
            &state,
            id,
            DocumentDepartureActionDto::Save,
            opened.selected_path,
            &[continuous_job()],
            |_| {
                order.borrow_mut().push("cancel");
                Ok(())
            },
            |_, _| {
                order.borrow_mut().push("stopped");
                Ok(())
            },
            || {
                order.borrow_mut().push("save");
                Ok(Some(path.to_string_lossy().into_owned()))
            },
        )
        .unwrap();
        let _ = std::fs::remove_file(path);

        assert_eq!(order.into_inner(), ["cancel", "stopped", "save"]);
        assert!(outcome.committed);
    }

    #[test]
    fn cancellation_delivery_failure_still_dispatches_all_jobs_and_aborts() {
        let state = CurrentGameState::default();
        let opened = state.replace(SOURCE, None).unwrap();
        state.force_dirty();
        let id = departure_id(state.prepare_replacement(CANDIDATE, None).unwrap());
        let mut whole = continuous_job();
        whole.job_id = "job-2".to_string();
        whole.lane = AnalysisJobLaneDto::WholeGame;
        whole.mode = AnalysisJobModeDto::Finite;
        let dispatched = RefCell::new(Vec::new());
        let wait_called = Cell::new(false);
        let save_called = Cell::new(false);

        let outcome = resolve_replacement(
            &state,
            id,
            DocumentDepartureActionDto::Save,
            opened.selected_path,
            &[continuous_job(), whole],
            |job| {
                dispatched.borrow_mut().push(job.job_id.clone());
                if job.job_id == "job-1" {
                    Err("cancel delivery failed".to_string())
                } else {
                    Ok(())
                }
            },
            |_, _| {
                wait_called.set(true);
                Ok(())
            },
            || {
                save_called.set(true);
                Ok(None)
            },
        )
        .unwrap();

        assert_eq!(dispatched.into_inner(), ["job-1", "job-2"]);
        assert!(!wait_called.get());
        assert!(!save_called.get());
        assert!(!outcome.committed);
        assert!(outcome.message.contains("cancel delivery failed"));
    }
}
