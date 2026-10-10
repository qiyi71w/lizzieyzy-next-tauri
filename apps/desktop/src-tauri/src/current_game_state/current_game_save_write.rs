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
            score_mean_black: Some(2.0),
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
        .unwrap()
        .current_game
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
        .unwrap()
        .current_game
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
        .unwrap()
        .current_game
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
        .unwrap()
        .current_game
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
    assert!(current.current_game.is_none());
    assert_eq!(current.saved_path, stale_path.to_string_lossy());
    assert_eq!(current.captured_generation, saved.generation);
    assert_eq!(current.captured_snapshot_seq, saved.snapshot_seq);
    assert_eq!(state.native_path().as_deref(), Some("/tmp/replacement.sgf"));
    assert!(!state.inspect().1);
    let _ = fs::remove_file(analysis_path);
    let _ = fs::remove_file(stale_path);
}

#[test]
fn captured_full_tree_keeps_properties_personal_comments_and_valid_analysis_before_late_edit() {
    let state = CurrentGameState::default();
    let opened = state.replace(
        "(;SZ[9]KM[6.5]DT[2020-03,04]RE[W+R]C[root]XY[unknown]AB[aa](;W[bb]C[main];B[])(;PL[B]C[setup]AW[cc];B[dd]C[branch]LB[dd:label]))",
        Some("source.sgf".into()),
    ).unwrap();
    let analyzed = state
        .attach_primary_analysis(opened.generation, NodePath::default(), analysis_payload(777))
        .unwrap();
    let captured_tree = analyzed.tree.clone();
    let captured = state
        .capture_save_snapshot(analyzed.selected_path.clone())
        .unwrap();
    let later = state
        .set_personal_comment(NodePath::default(), "later comment".into())
        .unwrap();
    let path = std::env::temp_dir().join(format!("lizzieyzy-full-snapshot-{}.sgf", uuid::Uuid::new_v4()));
    fs::write(&path, b"protected previous bytes").unwrap();
    let saved = state
        .persist_save_snapshot(path.to_string_lossy().into_owned(), captured)
        .unwrap();
    assert_eq!(saved.captured_generation, analyzed.generation);
    assert_eq!(saved.captured_snapshot_seq, analyzed.snapshot_seq);
    let saved = saved.current_game.unwrap();
    let reopened = CurrentSgfDocument::open(&fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(reopened.tree().unwrap(), captured_tree);
    assert_eq!(
        reopened
            .snapshot(&NodePath::default())
            .unwrap()
            .primary_analysis
            .unwrap()
            .visits,
        777
    );
    assert_eq!(reopened.result(), Some("W+R"));
    assert_eq!(
        reopened.snapshot(&NodePath::default()).unwrap().personal_comment,
        "root"
    );
    assert_eq!(saved.tree, later.tree);
    assert!(saved.dirty);
    assert_eq!(saved.generation, later.generation);
    fs::remove_file(path).unwrap();
}

#[test]
fn off_ui_captured_save_allows_new_document_and_returns_only_the_saved_receipt() {
    let state = std::sync::Arc::new(CurrentGameState::default());
    let opened = state.replace(BRANCHING, Some("original.sgf".into())).unwrap();
    let snapshot = state.capture_save_snapshot(opened.selected_path.clone()).unwrap();
    let worker_state = state.clone();
    let path = std::env::temp_dir().join(format!("worker-save-{}.SGF", uuid::Uuid::new_v4()));
    let worker_path = path.clone();
    let calling_thread = std::thread::current().id();
    let (entered, entered_rx) = std::sync::mpsc::channel();
    let (release, release_rx) = std::sync::mpsc::channel();
    let worker = tauri::async_runtime::spawn_blocking(move || {
        entered.send(std::thread::current().id()).unwrap();
        release_rx.recv().unwrap();
        worker_state.persist_save_snapshot(worker_path.to_string_lossy().into_owned(), snapshot)
    });
    assert_ne!(entered_rx.recv().unwrap(), calling_thread);
    let new_game = state
        .replace("(;SZ[9]C[new document])", Some("replacement.sgf".into()))
        .unwrap();
    let before = state.inspect();
    release.send(()).unwrap();
    let receipt = tauri::async_runtime::block_on(worker).unwrap().unwrap();
    assert!(receipt.current_game.is_none());
    assert_eq!(receipt.saved_path, path.to_string_lossy());
    assert_eq!(receipt.captured_generation, opened.generation);
    assert_eq!(receipt.captured_snapshot_seq, opened.snapshot_seq);
    assert_eq!(state.inspect(), before);
    assert_eq!(new_game.native_path.as_deref(), Some("replacement.sgf"));
    let reopened = CurrentSgfDocument::open(&fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(reopened.tree().unwrap(), opened.tree);
    fs::remove_file(path).unwrap();
}

#[test]
fn cancelled_save_as_preserves_protected_target_and_uppercase_target_is_written_exactly() {
    let state = CurrentGameState::default();
    let opened = state.replace(BRANCHING, Some("original.sgf".into())).unwrap();
    let edited = state
        .set_personal_comment(NodePath::default(), "personal edit".into())
        .unwrap();
    let directory = std::env::temp_dir().join(format!("save-target-{}", uuid::Uuid::new_v4()));
    fs::create_dir(&directory).unwrap();
    let target = directory.join("protected-sgf-中文.sgf");
    fs::write(&target, b"protected\0original bytes").unwrap();
    let before = state.inspect();
    let cancel = crate::save_as::persist_current_game_save_as(
        &state,
        save_as_dialog::SaveAsDialogOutcome::Cancelled,
        state.capture_save_snapshot(edited.selected_path.clone()).unwrap(),
    )
    .unwrap();
    assert!(cancel.is_none());
    assert_eq!(state.inspect(), before);
    assert_eq!(fs::read(&target).unwrap(), b"protected\0original bytes");
    let uppercase = directory.join("game.SGF");
    fs::write(&uppercase, b"previous game").unwrap();
    let receipt = crate::save_as::persist_current_game_save_as(
        &state,
        save_as_dialog::SaveAsDialogOutcome::Chosen(uppercase.to_string_lossy().into_owned()),
        state.capture_save_snapshot(edited.selected_path).unwrap(),
    )
    .unwrap()
    .unwrap();
    assert_eq!(receipt.saved_path, uppercase.to_string_lossy());
    assert!(!receipt.current_game.unwrap().dirty);
    assert!(!directory.join("game.SGF.sgf").exists());
    assert_eq!(
        CurrentSgfDocument::open(&fs::read_to_string(&uppercase).unwrap())
            .unwrap()
            .snapshot(&NodePath::default())
            .unwrap()
            .personal_comment,
        "personal edit"
    );
    assert_eq!(opened.native_path.as_deref(), Some("original.sgf"));
    fs::remove_dir_all(directory).unwrap();
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
