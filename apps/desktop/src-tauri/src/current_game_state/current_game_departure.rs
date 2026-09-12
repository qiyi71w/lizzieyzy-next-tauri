use super::*;
use app_model::{
    AnalysisJobEventDto, AnalysisJobLaneDto, AnalysisJobModeDto, AnalysisJobOutcomeDto,
    AnalysisJobStartedDto, AnalysisJobStateDto, ApplicationExitDispositionDto, ApplicationTeardownAttemptDto,
    CurrentGameErrorKind, DocumentDepartureAdmissionDto, MoveVertex, NodePath, PointDto,
};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

const BRANCHING: &str = include_str!("../../../../../tests/golden/editable-workspace-branching.sgf");
const EMPTY: &str = "(;GM[1]FF[4]SZ[19]KM[7.5]PB[黑]PW[白])";

fn path(indices: &[u32]) -> NodePath {
    NodePath {
        indices: indices.to_vec(),
    }
}

fn unique_path(label: &str) -> std::path::PathBuf {
    let unique = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    std::env::temp_dir().join(format!("lizzieyzy-departure-{label}-{unique}.sgf"))
}

fn projectable_frame(visits: u32, x: u8, y: u8) -> app_model::AnalysisFrameDto {
    app_model::AnalysisFrameDto {
        job_id: uuid::Uuid::nil(),
        game_id: None,
        node_id: None,
        turn: 0,
        visits,
        winrate_black: 0.61,
        score_mean_black: 2.25,
        score_stdev: None,
        candidates: vec![app_model::CandidateMoveDto {
            vertex: MoveVertex::Point(PointDto { x, y }),
            visits,
            winrate_black: 0.61,
            score_mean_black: 2.25,
            policy_prior: Some(0.4),
            pv: vec![MoveVertex::Point(PointDto { x, y })],
        }],
        ownership: None,
        policy: None,
    }
}

fn job_event(
    job_id: &str,
    lane: AnalysisJobLaneDto,
    outcome: AnalysisJobOutcomeDto,
    generation: u64,
    indices: &[u32],
    frame: Option<app_model::AnalysisFrameDto>,
) -> AnalysisJobEventDto {
    AnalysisJobEventDto {
        run_id: "run-1".into(),
        job_id: job_id.into(),
        lane,
        mode: AnalysisJobModeDto::Finite,
        generation,
        node_path: path(indices),
        outcome,
        completed: None,
        expected: None,
        remaining: None,
        frame,
        failure: None,
        current_game: None,
    }
}

fn continuous_event(
    job_id: &str,
    outcome: AnalysisJobOutcomeDto,
    generation: u64,
    indices: &[u32],
    frame: Option<app_model::AnalysisFrameDto>,
) -> AnalysisJobEventDto {
    let mut event = job_event(
        job_id,
        AnalysisJobLaneDto::SelectedNode,
        outcome,
        generation,
        indices,
        frame,
    );
    event.mode = AnalysisJobModeDto::Continuous;
    event
}

fn started(job_id: &str, generation: u64) -> AnalysisJobStartedDto {
    AnalysisJobStartedDto {
        run_id: "run-1".into(),
        job_id: job_id.into(),
        lane: AnalysisJobLaneDto::SelectedNode,
        mode: AnalysisJobModeDto::Continuous,
        state: AnalysisJobStateDto::Searching,
        generation,
        node_path: path(&[]),
    }
}

fn departure_id(admission: DocumentDepartureAdmissionDto) -> u64 {
    match admission {
        DocumentDepartureAdmissionDto::NeedsDecision { departure_id }
        | DocumentDepartureAdmissionDto::Ready { departure_id } => departure_id,
    }
}

#[test]
fn replacement_validates_candidate_before_dirty_prompt() {
    let state = CurrentGameState::default();
    state
        .replace(BRANCHING, Some("/tmp/branching.sgf".to_string()))
        .unwrap();
    state.force_dirty();
    let before = state.inspect();

    let malformed = state
        .prepare_replacement("not an sgf", Some("/tmp/bad.sgf".to_string()))
        .unwrap_err();
    assert_eq!(malformed.kind, CurrentGameErrorKind::MalformedSgf);
    assert_eq!(state.inspect(), before);

    let unsupported = state
        .prepare_replacement("(;GM[1]FF[4]SZ[99])", None)
        .unwrap_err();
    assert_eq!(unsupported.kind, CurrentGameErrorKind::UnsupportedBoardSize);
    assert_eq!(state.inspect(), before);

    let admitted = state.prepare_replacement(EMPTY, None).unwrap();
    assert!(matches!(
        admitted,
        DocumentDepartureAdmissionDto::NeedsDecision { departure_id } if departure_id == 1
    ));
    assert_eq!(state.inspect(), before);
}

#[test]
fn dirty_replacement_cannot_commit_before_a_departure_decision() {
    let state = CurrentGameState::default();
    state
        .replace(BRANCHING, Some("/tmp/branching.sgf".to_string()))
        .unwrap();
    state
        .set_personal_comment(path(&[]), "unsaved personal comment".to_string())
        .unwrap();
    let before = state.inspect();
    let admission = state.prepare_replacement(EMPTY, None).unwrap();
    let DocumentDepartureAdmissionDto::NeedsDecision { departure_id } = admission else {
        panic!("dirty replacement must require a departure decision");
    };

    let error = state.commit_replacement(departure_id).unwrap_err();
    assert_eq!(error.kind, CurrentGameErrorKind::DepartureBlocked);
    assert_eq!(state.inspect(), before);

    state.cancel_replacement(departure_id).unwrap();
    assert_eq!(state.inspect(), before);
}

#[test]
fn duplicate_replacement_is_rejected_and_initial_cancel_does_not_seal_jobs() {
    let state = CurrentGameState::default();
    let opened = state
        .replace(BRANCHING, Some("/tmp/branching.sgf".to_string()))
        .unwrap();
    state.force_dirty();
    let before = state.inspect();
    let first = departure_id(state.prepare_replacement(EMPTY, None).unwrap());
    let busy = state.prepare_replacement(EMPTY, None).unwrap_err();
    assert_eq!(busy.kind, CurrentGameErrorKind::DepartureInProgress);
    assert_eq!(state.inspect(), before);

    let cancelled = state.cancel_replacement(first).unwrap();
    assert!(!cancelled.committed);
    assert!(!cancelled.analysis_stopped);
    assert_eq!(state.inspect(), before);

    let attached = state
        .attach_from_job_event(&continuous_event(
            "job-live",
            AnalysisJobOutcomeDto::Progress,
            opened.generation,
            &opened.selected_path.indices,
            Some(projectable_frame(400, 3, 3)),
        ))
        .unwrap();
    assert!(attached.dirty);
    assert_eq!(attached.snapshot.primary_analysis.unwrap().visits, 400);
}

#[test]
fn save_and_leave_seals_accepted_results_and_commits_candidate() {
    let state = CurrentGameState::default();
    let opened = state
        .replace(BRANCHING, Some("/tmp/branching.sgf".to_string()))
        .unwrap();
    let attached = state
        .attach_from_job_event(&continuous_event(
            "job-accepted",
            AnalysisJobOutcomeDto::Progress,
            opened.generation,
            &opened.selected_path.indices,
            Some(projectable_frame(400, 3, 3)),
        ))
        .unwrap();
    assert!(attached.dirty);

    let id = departure_id(state.prepare_replacement(EMPTY, None).unwrap());
    state
        .begin_protected_commit(id, &[started("job-late", opened.generation)])
        .unwrap();

    let before_save = state.inspect();
    assert!(state
        .attach_from_job_event(&continuous_event(
            "job-late",
            AnalysisJobOutcomeDto::Progress,
            opened.generation,
            &opened.selected_path.indices,
            Some(projectable_frame(50, 1, 1)),
        ))
        .is_none());
    assert_eq!(state.inspect(), before_save);
    assert_eq!(
        state.play(path(&[]), MoveVertex::Pass).unwrap_err().kind,
        CurrentGameErrorKind::DepartureBlocked
    );
    assert!(state
        .save_to_path("/tmp/ignored.sgf".to_string(), attached.selected_path.clone())
        .unwrap_err()
        .contains("departure is in progress"));
    assert_eq!(
        state.prepare_replacement(EMPTY, None).unwrap_err().kind,
        CurrentGameErrorKind::DepartureInProgress
    );

    let saved_path = unique_path("save-and-leave");
    let saved = state
        .save_sealed_departure(
            id,
            saved_path.to_string_lossy().into_owned(),
            attached.selected_path.clone(),
        )
        .unwrap();
    let written = fs::read_to_string(&saved_path).unwrap();
    let _ = fs::remove_file(&saved_path);
    assert!(written.contains("PB[") || written.contains("SZ["));
    assert_eq!(saved.generation, attached.generation);
    assert!(!saved.dirty);

    let committed = state.commit_replacement(id).unwrap();
    assert!(committed.committed);
    assert!(committed.analysis_stopped);
    let current = committed.current.unwrap();
    assert!(current.selected_path.indices.is_empty());
    assert_eq!(current.snapshot.position.move_number, 0);
    assert!(current.native_path.is_none());
    assert!(!state.serialize().unwrap().contains("first continuation"));
}

#[test]
fn failed_departure_save_restores_editing_and_keeps_late_jobs_rejected() {
    let state = CurrentGameState::default();
    let opened = state
        .replace(BRANCHING, Some("/tmp/branching.sgf".to_string()))
        .unwrap();
    let edited = state
        .play(path(&[0]), MoveVertex::Point(PointDto { x: 1, y: 1 }))
        .unwrap();
    let id = departure_id(state.prepare_replacement(EMPTY, None).unwrap());
    state
        .begin_protected_commit(id, &[started("job-old", edited.generation)])
        .unwrap();

    let dir = unique_path("save-fail-dir");
    fs::create_dir_all(&dir).unwrap();
    let error = state
        .save_sealed_departure(
            id,
            dir.to_string_lossy().into_owned(),
            edited.selected_path.clone(),
        )
        .unwrap_err();
    let _ = fs::remove_dir_all(&dir);
    assert!(error.contains("failed to write"), "{error}");

    let aborted = state
        .abort_protected_commit(id, edited.selected_path.clone())
        .unwrap();
    assert!(!aborted.committed);
    assert!(aborted.analysis_stopped);
    let current = aborted.current.unwrap();
    assert_eq!(current.generation, edited.generation);
    assert!(current.dirty);
    assert_eq!(current.native_path, opened.native_path);
    assert_eq!(current.selected_path, edited.selected_path);

    assert!(state
        .attach_from_job_event(&job_event(
            "job-old",
            AnalysisJobLaneDto::SelectedNode,
            AnalysisJobOutcomeDto::Completed,
            edited.generation,
            &[],
            Some(projectable_frame(50, 1, 1)),
        ))
        .is_none());

    let restarted = state
        .attach_from_job_event(&job_event(
            "job-new",
            AnalysisJobLaneDto::SelectedNode,
            AnalysisJobOutcomeDto::Completed,
            edited.generation,
            &[],
            Some(projectable_frame(900, 15, 3)),
        ))
        .unwrap();
    assert_eq!(restarted.snapshot.primary_analysis.unwrap().visits, 900);

    let saved_path = unique_path("ordinary-after-abort");
    let saved = state
        .save_to_path(
            saved_path.to_string_lossy().into_owned(),
            edited.selected_path.clone(),
        )
        .unwrap();
    let _ = fs::remove_file(&saved_path);
    assert!(!saved.dirty);
    assert_eq!(saved.generation, edited.generation);
}

#[test]
fn discard_does_not_write_source_and_installs_candidate() {
    let source = unique_path("discard-source");
    fs::write(&source, BRANCHING).unwrap();
    let state = CurrentGameState::default();
    state
        .replace(BRANCHING, Some(source.to_string_lossy().into_owned()))
        .unwrap();
    state
        .play(path(&[0]), MoveVertex::Point(PointDto { x: 1, y: 1 }))
        .unwrap();
    let before_source = fs::read_to_string(&source).unwrap();

    let id = departure_id(state.prepare_replacement(EMPTY, None).unwrap());
    state
        .begin_protected_commit(id, &[started("job-discard", 2)])
        .unwrap();
    let committed = state.commit_replacement(id).unwrap();
    let after_source = fs::read_to_string(&source).unwrap();
    let _ = fs::remove_file(&source);

    assert_eq!(before_source, after_source);
    assert!(committed.committed);
    let current = committed.current.unwrap();
    assert!(current.selected_path.indices.is_empty());
    assert!(current.native_path.is_none());
    assert!(!current.dirty);
}

#[test]
fn clean_document_enters_protected_commit_without_prompt() {
    let state = CurrentGameState::default();
    state
        .replace(BRANCHING, Some("/tmp/branching.sgf".to_string()))
        .unwrap();
    let admitted = state.prepare_replacement(EMPTY, None).unwrap();
    assert!(matches!(admitted, DocumentDepartureAdmissionDto::Ready { .. }));
    let id = departure_id(admitted);
    state.begin_protected_commit(id, &[]).unwrap();
    let committed = state.commit_replacement(id).unwrap();
    assert!(committed.committed);
    assert!(committed.current.unwrap().selected_path.indices.is_empty());
}

#[test]
fn ordinary_save_during_analysis_still_uses_invocation_snapshot() {
    let state = CurrentGameState::default();
    let opened = state.replace(BRANCHING, None).unwrap();
    let attached = state
        .attach_from_job_event(&job_event(
            "job-1",
            AnalysisJobLaneDto::SelectedNode,
            AnalysisJobOutcomeDto::Completed,
            opened.generation,
            &[],
            Some(projectable_frame(400, 3, 3)),
        ))
        .unwrap();
    let saved_path = unique_path("ordinary-save");
    let saved = state
        .save_to_path_after_hook(
            saved_path.to_string_lossy().into_owned(),
            attached.selected_path.clone(),
            || {
                let _ = state.attach_from_job_event(&job_event(
                    "job-2",
                    AnalysisJobLaneDto::WholeGame,
                    AnalysisJobOutcomeDto::Progress,
                    opened.generation,
                    &[0],
                    Some(projectable_frame(900, 15, 3)),
                ));
            },
        )
        .unwrap();
    let written = fs::read_to_string(&saved_path).unwrap();
    let _ = fs::remove_file(&saved_path);
    assert!(saved.dirty);
    assert_eq!(saved.generation, opened.generation);
    assert_ne!(written, state.serialize().unwrap());
}

#[test]
fn exit_is_serial_and_initial_cancel_does_not_seal_jobs() {
    let state = CurrentGameState::default();
    let opened = state
        .replace(BRANCHING, Some("/tmp/branching.sgf".to_string()))
        .unwrap();
    state.force_dirty();
    let before = state.inspect();
    let first = departure_id(state.prepare_exit().unwrap());
    let busy = state.prepare_exit().unwrap_err();
    assert_eq!(busy.kind, CurrentGameErrorKind::DepartureInProgress);
    assert_eq!(
        state.prepare_replacement(EMPTY, None).unwrap_err().kind,
        CurrentGameErrorKind::DepartureInProgress
    );
    assert_eq!(state.inspect(), before);

    let cancelled = state.cancel_replacement(first).unwrap();
    assert!(!cancelled.committed);
    assert!(!cancelled.analysis_stopped);
    assert!(state.application_exit_disposition().is_none());
    assert_eq!(state.inspect(), before);

    let attached = state
        .attach_from_job_event(&job_event(
            "job-live",
            AnalysisJobLaneDto::SelectedNode,
            AnalysisJobOutcomeDto::Completed,
            opened.generation,
            &[],
            Some(projectable_frame(400, 3, 3)),
        ))
        .unwrap();
    assert!(attached.dirty);
    assert_eq!(attached.snapshot.primary_analysis.unwrap().visits, 400);
}

#[test]
fn save_and_leave_exit_seals_keeps_document_and_starts_incomplete_until_teardown() {
    let state = CurrentGameState::default();
    let source = unique_path("exit-save-source");
    let opened = state
        .replace(BRANCHING, Some(source.to_string_lossy().into_owned()))
        .unwrap();
    let attached = state
        .attach_from_job_event(&job_event(
            "job-accepted",
            AnalysisJobLaneDto::SelectedNode,
            AnalysisJobOutcomeDto::Completed,
            opened.generation,
            &[],
            Some(projectable_frame(400, 3, 3)),
        ))
        .unwrap();
    assert!(attached.dirty);

    let id = departure_id(state.prepare_exit().unwrap());
    state
        .begin_protected_commit(id, &[started("job-late", opened.generation)])
        .unwrap();
    assert!(state
        .attach_from_job_event(&job_event(
            "job-late",
            AnalysisJobLaneDto::SelectedNode,
            AnalysisJobOutcomeDto::Completed,
            opened.generation,
            &[],
            Some(projectable_frame(50, 1, 1)),
        ))
        .is_none());

    let saved = state
        .save_sealed_departure(
            id,
            source.to_string_lossy().into_owned(),
            attached.selected_path.clone(),
        )
        .unwrap();
    let written = fs::read_to_string(&source).unwrap();
    assert!(written.contains("first continuation"));
    assert!(!saved.dirty);

    let leaving = state
        .begin_application_teardown(
            id,
            attached.selected_path.clone(),
            ApplicationExitDispositionDto::ExitIncomplete,
        )
        .unwrap();
    assert!(leaving.committed);
    assert_eq!(
        leaving.disposition,
        Some(ApplicationExitDispositionDto::ExitIncomplete)
    );
    let current = leaving.current.unwrap();
    assert_eq!(current.generation, attached.generation);
    assert_eq!(current.native_path.as_deref(), source.to_str());
    assert!(state.serialize().unwrap().contains("first continuation"));

    let finished = state
        .finish_application_teardown(
            id,
            attached.selected_path,
            ApplicationTeardownAttemptDto::Completed,
            false,
        )
        .unwrap();
    let _ = fs::remove_file(&source);
    assert_eq!(
        finished.disposition,
        Some(ApplicationExitDispositionDto::CleanCompleted)
    );
    assert_eq!(
        state.application_exit_disposition(),
        Some(ApplicationExitDispositionDto::CleanCompleted)
    );
}

#[test]
fn failed_exit_save_restores_window_and_keeps_late_jobs_rejected() {
    let state = CurrentGameState::default();
    let opened = state
        .replace(BRANCHING, Some("/tmp/branching.sgf".to_string()))
        .unwrap();
    let edited = state
        .play(path(&[0]), MoveVertex::Point(PointDto { x: 1, y: 1 }))
        .unwrap();
    let id = departure_id(state.prepare_exit().unwrap());
    state
        .begin_protected_commit(id, &[started("job-old", edited.generation)])
        .unwrap();

    let dir = unique_path("exit-save-fail-dir");
    fs::create_dir_all(&dir).unwrap();
    let error = state
        .save_sealed_departure(
            id,
            dir.to_string_lossy().into_owned(),
            edited.selected_path.clone(),
        )
        .unwrap_err();
    let _ = fs::remove_dir_all(&dir);
    assert!(error.contains("failed to write"), "{error}");

    let aborted = state
        .abort_protected_commit(id, edited.selected_path.clone())
        .unwrap();
    assert!(!aborted.committed);
    assert!(aborted.analysis_stopped);
    assert!(state.application_exit_disposition().is_none());
    let current = aborted.current.unwrap();
    assert_eq!(current.generation, edited.generation);
    assert!(current.dirty);
    assert_eq!(current.native_path, opened.native_path);
    assert!(state.play(edited.selected_path.clone(), MoveVertex::Pass).is_ok());
    assert!(state
        .attach_from_job_event(&job_event(
            "job-old",
            AnalysisJobLaneDto::SelectedNode,
            AnalysisJobOutcomeDto::Completed,
            edited.generation,
            &[],
            Some(projectable_frame(50, 1, 1)),
        ))
        .is_none());
}

#[test]
fn discard_exit_does_not_write_source_and_keeps_discard_after_timeout() {
    let source = unique_path("exit-discard-source");
    fs::write(&source, BRANCHING).unwrap();
    let state = CurrentGameState::default();
    let opened = state
        .replace(BRANCHING, Some(source.to_string_lossy().into_owned()))
        .unwrap();
    state
        .play(path(&[0]), MoveVertex::Point(PointDto { x: 1, y: 1 }))
        .unwrap();
    let before_source = fs::read_to_string(&source).unwrap();

    let id = departure_id(state.prepare_exit().unwrap());
    state
        .begin_protected_commit(id, &[started("job-discard", opened.generation)])
        .unwrap();
    let leaving = state
        .begin_application_teardown(
            id,
            opened.selected_path.clone(),
            ApplicationExitDispositionDto::ExplicitDiscard,
        )
        .unwrap();
    let after_source = fs::read_to_string(&source).unwrap();
    assert_eq!(before_source, after_source);
    assert_eq!(
        leaving.disposition,
        Some(ApplicationExitDispositionDto::ExplicitDiscard)
    );
    assert!(state
        .attach_from_job_event(&job_event(
            "job-discard",
            AnalysisJobLaneDto::SelectedNode,
            AnalysisJobOutcomeDto::Completed,
            opened.generation,
            &[],
            Some(projectable_frame(50, 1, 1)),
        ))
        .is_none());

    let timed_out = state
        .finish_application_teardown(
            id,
            opened.selected_path,
            ApplicationTeardownAttemptDto::TimedOut {
                outstanding: vec!["foreground engine".to_string()],
            },
            true,
        )
        .unwrap();
    let _ = fs::remove_file(&source);
    assert_eq!(
        timed_out.disposition,
        Some(ApplicationExitDispositionDto::ExplicitDiscard)
    );
    assert_eq!(
        state.application_exit_disposition(),
        Some(ApplicationExitDispositionDto::ExplicitDiscard)
    );
}

#[test]
fn clean_exit_rejects_late_jobs_and_timeout_stays_incomplete_until_retry() {
    let state = CurrentGameState::default();
    let opened = state
        .replace(BRANCHING, Some("/tmp/branching.sgf".to_string()))
        .unwrap();
    let admitted = state.prepare_exit().unwrap();
    assert!(matches!(admitted, DocumentDepartureAdmissionDto::Ready { .. }));
    let id = departure_id(admitted);
    state
        .begin_protected_commit(id, &[started("job-clean", opened.generation)])
        .unwrap();
    state
        .begin_application_teardown(
            id,
            opened.selected_path.clone(),
            ApplicationExitDispositionDto::ExitIncomplete,
        )
        .unwrap();
    assert!(state
        .attach_from_job_event(&job_event(
            "job-clean",
            AnalysisJobLaneDto::SelectedNode,
            AnalysisJobOutcomeDto::Completed,
            opened.generation,
            &[],
            Some(projectable_frame(50, 1, 1)),
        ))
        .is_none());
    assert_eq!(
        state.prepare_exit().unwrap_err().kind,
        CurrentGameErrorKind::DepartureInProgress
    );

    let timed_out = state
        .finish_application_teardown(
            id,
            opened.selected_path.clone(),
            ApplicationTeardownAttemptDto::TimedOut {
                outstanding: vec!["foreground engine".to_string()],
            },
            false,
        )
        .unwrap();
    assert_eq!(
        timed_out.disposition,
        Some(ApplicationExitDispositionDto::ExitIncomplete)
    );
    assert_eq!(
        state.application_exit_disposition(),
        Some(ApplicationExitDispositionDto::ExitIncomplete)
    );
    assert_eq!(
        state.prepare_exit().unwrap_err().kind,
        CurrentGameErrorKind::DepartureInProgress
    );

    let retried = state
        .finish_application_teardown(
            id,
            opened.selected_path,
            ApplicationTeardownAttemptDto::Completed,
            false,
        )
        .unwrap();
    assert_eq!(
        retried.disposition,
        Some(ApplicationExitDispositionDto::CleanCompleted)
    );
    assert_eq!(
        state.application_exit_disposition(),
        Some(ApplicationExitDispositionDto::CleanCompleted)
    );
}
