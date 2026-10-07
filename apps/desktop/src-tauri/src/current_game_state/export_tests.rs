use super::*;

#[test]
fn selected_line_capture_is_a_read_and_later_edits_do_not_refresh_it() {
    let state = CurrentGameState::default();
    let opened = state
        .replace(
            "(;SZ[9]PB[黑]C[root];B[aa](;W[bb])(;W[]C[leaf]))",
            Some("original.sgf".into()),
        )
        .unwrap();
    let selected = NodePath { indices: vec![0] };
    let current = state.select_path(selected.clone(), opened.generation).unwrap();
    let edited = state
        .set_personal_comment(selected.clone(), "personal".into())
        .unwrap();
    let before = state.inspect();
    let leaf = NodePath { indices: vec![0, 1] };
    let captured = state
        .capture_selected_line(current.generation, &selected, &leaf)
        .unwrap();
    assert_eq!(state.inspect(), before);
    assert_eq!(state.native_path().as_deref(), Some("original.sgf"));
    assert!(edited.dirty);
    assert!(captured.contains("C[personal]"));
    assert!(!captured.contains("W[bb]"));
    state
        .set_personal_comment(selected.clone(), "later".into())
        .unwrap();
    assert!(!captured.contains("later"));
    assert!(state
        .capture_selected_line(current.generation + 1, &selected, &leaf)
        .is_err());
    assert!(state
        .capture_selected_line(current.generation, &selected, &NodePath::default())
        .is_err());
}
