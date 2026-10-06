use super::*;
use app_model::{CandidateMoveDto, MoveVertex, NodePath, PointDto};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

const BRANCHING: &str = include_str!("../../../../../tests/golden/editable-workspace-branching.sgf");

fn analysis_payload(visits: u32) -> SgfAnalysisPayload {
    SgfAnalysisPayload {
        engine_name: "KataGo".to_string(),
        visits,
        winrate_black: 0.6,
        score_mean_black: Some(2.0),
        score_stdev: None,
        pda: None,
        candidates: vec![CandidateMoveDto {
            vertex: MoveVertex::Pass,
            visits,
            winrate_black: 0.6,
            score_mean_black: 2.0,
            policy_prior: None,
            pv: vec![MoveVertex::Pass],
        }],
        ownership: None,
    }
}

#[test]
fn current_game_save_write_failure_leaves_path_and_dirty_unchanged() {
    let state = CurrentGameState::default();
    let opened = state
        .replace(BRANCHING, Some("/tmp/branching.sgf".to_string()))
        .unwrap();
    let parent = NodePath { indices: vec![0] };
    let edited = state
        .play(parent, MoveVertex::Point(PointDto { x: 1, y: 1 }))
        .unwrap();
    assert!(edited.dirty);
    assert_eq!(edited.generation, opened.generation + 1);
    let before = state.inspect();

    let unique = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let dir = std::env::temp_dir().join(format!("lizzieyzy-save-failure-{unique}"));
    fs::create_dir_all(&dir).unwrap();
    let error = state
        .save_to_path(dir.to_string_lossy().into_owned(), edited.selected_path.clone())
        .unwrap_err();
    let _ = fs::remove_dir_all(&dir);

    assert!(error.contains("failed to write"), "{error}");
    assert_eq!(state.inspect(), before);
    assert_eq!(edited.native_path.as_deref(), Some("/tmp/branching.sgf"));
}

#[test]
fn current_game_save_clears_dirty_without_changing_cursor_or_generation() {
    let state = CurrentGameState::default();
    let opened = state
        .replace(BRANCHING, Some("/tmp/branching.sgf".to_string()))
        .unwrap();
    let edited = state
        .play(
            NodePath { indices: vec![0] },
            MoveVertex::Point(PointDto { x: 1, y: 1 }),
        )
        .unwrap();
    let unique = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let path = std::env::temp_dir().join(format!("lizzieyzy-save-success-{unique}.sgf"));

    let saved = state
        .save_to_path(path.to_string_lossy().into_owned(), edited.selected_path.clone())
        .unwrap();
    let written = fs::read_to_string(&path).unwrap();
    let _ = fs::remove_file(&path);

    assert_eq!(saved.generation, edited.generation);
    assert!(!saved.dirty);
    assert_eq!(saved.selected_path, edited.selected_path);
    assert_eq!(saved.snapshot.path, edited.selected_path);
    assert_eq!(
        saved.native_path.as_deref(),
        Some(path.to_string_lossy().as_ref())
    );
    assert_eq!(written, state.serialize().unwrap());
    assert_eq!(
        state.inspect(),
        (
            edited.generation,
            false,
            Some(path.to_string_lossy().into_owned()),
            Some(written)
        )
    );
    assert_eq!(opened.native_path.as_deref(), Some("/tmp/branching.sgf"));
}

#[test]
fn current_game_save_as_updates_path_and_keeps_prior_state_on_failure() {
    let state = CurrentGameState::default();
    let edited = {
        state
            .replace(BRANCHING, Some("/tmp/branching.sgf".to_string()))
            .unwrap();
        state
            .play(
                NodePath { indices: vec![0] },
                MoveVertex::Point(PointDto { x: 1, y: 1 }),
            )
            .unwrap()
    };
    let unique = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let path = std::env::temp_dir().join(format!("lizzieyzy-save-as-{unique}.sgf"));
    let saved = state
        .save_to_path(path.to_string_lossy().into_owned(), edited.selected_path.clone())
        .unwrap();
    let written = fs::read_to_string(&path).unwrap();
    let _ = fs::remove_file(&path);

    assert_eq!(saved.generation, edited.generation);
    assert!(!saved.dirty);
    assert_eq!(saved.selected_path, edited.selected_path);
    assert_eq!(
        saved.native_path.as_deref(),
        Some(path.to_string_lossy().as_ref())
    );
    assert!(written.contains("B[bb]"));
    assert_ne!(saved.native_path.as_deref(), Some("/tmp/branching.sgf"));
}

#[test]
fn current_game_save_as_redirect_does_not_write_or_adopt() {
    let state = CurrentGameState::default();
    let edited = {
        state
            .replace(BRANCHING, Some("/tmp/branching.sgf".to_string()))
            .unwrap();
        state
            .play(
                NodePath { indices: vec![0] },
                MoveVertex::Point(PointDto { x: 1, y: 1 }),
            )
            .unwrap()
    };
    let before = state.inspect();
    let unique = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let redirected = std::env::temp_dir().join(format!("lizzieyzy-save-as-redirect-{unique}.sgf"));

    let error = crate::save_as::persist_current_game_save_as(
        &state,
        save_as_dialog::SaveAsDialogOutcome::Redirected {
            requested: "/denied-acl/denied.sgf".to_string(),
            redirected: redirected.to_string_lossy().into_owned(),
        },
        state.capture_save_snapshot(edited.selected_path.clone()).unwrap(),
    )
    .unwrap_err();

    assert!(error.contains("failed to write"), "{error}");
    assert!(error.contains("/denied-acl/denied.sgf"), "{error}");
    assert!(!redirected.exists());
    assert_eq!(state.inspect(), before);
    assert_eq!(edited.native_path.as_deref(), Some("/tmp/branching.sgf"));
}

#[test]
fn savepoint_tracks_history_state_and_independent_analysis_content() {
    let state = CurrentGameState::default();
    let opened = state
        .replace(BRANCHING, Some("/tmp/source.sgf".to_string()))
        .unwrap();
    let edited = state
        .set_personal_comment(NodePath::default(), "saved comment".to_string())
        .unwrap();
    let unique = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let path = std::env::temp_dir().join(format!("lizzieyzy-history-savepoint-{unique}.sgf"));
    let saved = state
        .save_to_path(path.to_string_lossy().into_owned(), edited.selected_path.clone())
        .unwrap();
    assert!(!saved.dirty);
    assert!(saved.can_undo);

    let undone = state.undo(saved.generation).unwrap();
    assert!(undone.dirty);
    assert_eq!(undone.native_path, saved.native_path);
    let redone = state.redo(undone.generation).unwrap();
    assert!(!redone.dirty);
    assert_eq!(redone.native_path, saved.native_path);

    let analyzed = state
        .attach_primary_analysis(redone.generation, NodePath::default(), analysis_payload(777))
        .unwrap();
    assert!(analyzed.dirty);
    let after_history_change = state.undo(analyzed.generation).unwrap();
    assert!(after_history_change.dirty);
    let root = state
        .select_path(NodePath::default(), after_history_change.generation)
        .unwrap();
    assert_eq!(root.snapshot.primary_analysis.unwrap().visits, 777);
    let _ = fs::remove_file(path);
    assert_eq!(opened.native_path.as_deref(), Some("/tmp/source.sgf"));
}

#[test]
fn concurrent_analysis_keeps_save_dirty_and_replacement_does_not_adopt_stale_save_as_path() {
    let state = CurrentGameState::default();
    let opened = state
        .replace(BRANCHING, Some("/tmp/source.sgf".to_string()))
        .unwrap();
    let unique = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let analysis_path = std::env::temp_dir().join(format!("lizzieyzy-concurrent-analysis-{unique}.sgf"));
    let saved = state
        .save_to_path_after_hook(
            analysis_path.to_string_lossy().into_owned(),
            opened.selected_path.clone(),
            || {
                state
                    .attach_primary_analysis(opened.generation, NodePath::default(), analysis_payload(888))
                    .unwrap();
            },
        )
        .unwrap();
    assert!(saved.dirty);
    assert_eq!(
        saved.native_path.as_deref(),
        Some(analysis_path.to_string_lossy().as_ref())
    );

    let stale_path = std::env::temp_dir().join(format!("lizzieyzy-stale-save-as-{unique}.sgf"));
    let current = state
        .save_to_path_after_hook(
            stale_path.to_string_lossy().into_owned(),
            saved.selected_path.clone(),
            || {
                state
                    .replace("(;GM[1]FF[4]SZ[9])", Some("/tmp/replacement.sgf".to_string()))
                    .unwrap();
            },
        )
        .unwrap();
    assert_eq!(current.native_path.as_deref(), Some("/tmp/replacement.sgf"));
    assert!(!current.can_undo);
    assert!(!current.can_redo);
    let _ = fs::remove_file(analysis_path);
    let _ = fs::remove_file(stale_path);
}

#[test]
fn structural_undo_rejects_results_from_the_previous_generation() {
    let state = CurrentGameState::default();
    state.replace(BRANCHING, None).unwrap();
    let played = state
        .play(
            NodePath { indices: vec![0] },
            MoveVertex::Point(PointDto { x: 1, y: 1 }),
        )
        .unwrap();
    let undone = state.undo(played.generation).unwrap();
    assert!(undone.generation > played.generation);
    let stale = state
        .attach_primary_analysis(played.generation, undone.selected_path, analysis_payload(999))
        .unwrap_err();
    assert_eq!(stale.kind, app_model::CurrentGameErrorKind::NoCurrentGame);
}
