use super::*;
use app_model::{MoveVertex, NodePath, PointDto};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

const BRANCHING: &str = include_str!("../../../../../tests/golden/editable-workspace-branching.sgf");

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

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
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
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
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
    assert_eq!(saved.native_path.as_deref(), Some(path.to_string_lossy().as_ref()));
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
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("lizzieyzy-save-as-{unique}.sgf"));
    let saved = state
        .save_to_path(path.to_string_lossy().into_owned(), edited.selected_path.clone())
        .unwrap();
    let written = fs::read_to_string(&path).unwrap();
    let _ = fs::remove_file(&path);

    assert_eq!(saved.generation, edited.generation);
    assert!(!saved.dirty);
    assert_eq!(saved.selected_path, edited.selected_path);
    assert_eq!(saved.native_path.as_deref(), Some(path.to_string_lossy().as_ref()));
    assert!(written.contains("B[bb]"));
    assert_ne!(saved.native_path.as_deref(), Some("/tmp/branching.sgf"));
}
