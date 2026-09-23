use app_model::{CandidateMoveDto, MoveVertex, NodePath, PointDto};
use sgf::{CurrentSgfDocument, DocumentHistory, SgfAnalysisPayload};

fn path(indices: &[u32]) -> NodePath {
    NodePath {
        indices: indices.to_vec(),
    }
}

fn payload(visits: u32) -> SgfAnalysisPayload {
    SgfAnalysisPayload {
        engine_name: "KataGo".to_string(),
        visits,
        winrate_black: 0.55,
        score_mean_black: Some(1.5),
        score_stdev: None,
        pda: None,
        candidates: vec![CandidateMoveDto {
            vertex: MoveVertex::Pass,
            visits,
            winrate_black: 0.55,
            score_mean_black: 1.5,
            policy_prior: None,
            pv: vec![MoveVertex::Pass],
        }],
        ownership: None,
    }
}

#[test]
fn mixed_history_restores_exact_cursors_and_preserves_new_analysis_and_unknown_properties() {
    let mut document =
        CurrentSgfDocument::open("(;GM[1]FF[4]SZ[9]XY[root];B[aa]C[before]ZZ[keep](;W[bb])(;W[cc]))")
            .unwrap();
    let mut history = DocumentHistory::default();
    let original_cursor = path(&[0, 1]);

    let comment = document
        .set_personal_comment_with_history(&original_cursor, &path(&[0]), "after")
        .unwrap();
    history.commit(comment.edit.unwrap());

    let played = document
        .play_with_history(
            &path(&[0]),
            &path(&[0]),
            MoveVertex::Point(PointDto { x: 3, y: 3 }),
        )
        .unwrap();
    let played_path = played.snapshot.path.clone();
    history.commit(played.edit.unwrap());
    document
        .replace_primary_analysis(&played_path, &payload(321))
        .unwrap();
    document
        .replace_primary_analysis(&NodePath::default(), &payload(654))
        .unwrap();

    let removed = document
        .remove_variation_with_history(&played_path, &path(&[0, 0]))
        .unwrap();
    history.commit(removed.edit.unwrap());

    assert_eq!(
        history.undo(&mut document).unwrap().unwrap().selected_path,
        played_path
    );
    assert_eq!(
        history.undo(&mut document).unwrap().unwrap().selected_path,
        path(&[0])
    );
    assert_eq!(
        history.undo(&mut document).unwrap().unwrap().selected_path,
        original_cursor
    );
    assert_eq!(document.snapshot(&path(&[0])).unwrap().personal_comment, "before");
    assert_eq!(
        document
            .snapshot(&NodePath::default())
            .unwrap()
            .primary_analysis
            .unwrap()
            .visits,
        654
    );
    let serialized = document.serialize().unwrap();
    assert!(serialized.contains("XY[root]"));
    assert!(serialized.contains("ZZ[keep]"));

    history.redo(&mut document).unwrap().unwrap();
    let redone_play = history.redo(&mut document).unwrap().unwrap();
    assert_eq!(redone_play.selected_path, played_path);
    assert_eq!(
        document
            .snapshot(&played_path)
            .unwrap()
            .primary_analysis
            .unwrap()
            .visits,
        321
    );
    let redone_remove = history.redo(&mut document).unwrap().unwrap();
    assert_eq!(redone_remove.selected_path, path(&[0]));
    assert!(!history.can_redo());
}

#[test]
fn history_keeps_only_the_latest_hundred_effective_edits() {
    let mut document = CurrentSgfDocument::open("(;GM[1]FF[4]SZ[9])").unwrap();
    let mut history = DocumentHistory::default();
    let mut selected = NodePath::default();

    for _ in 0..101 {
        let outcome = document
            .play_with_history(&selected, &selected, MoveVertex::Pass)
            .unwrap();
        selected = outcome.snapshot.path;
        history.commit(outcome.edit.unwrap());
    }

    for _ in 0..100 {
        history.undo(&mut document).unwrap().unwrap();
    }
    assert!(!history.can_undo());
    assert_eq!(document.snapshot(&path(&[0])).unwrap().position.move_number, 1);
}

#[test]
fn noops_and_failures_do_not_commit_and_branching_clears_redo() {
    let mut document = CurrentSgfDocument::open("(;GM[1]FF[4]SZ[9];B[aa])").unwrap();
    let mut history = DocumentHistory::default();

    let existing = document
        .play_with_history(
            &NodePath::default(),
            &NodePath::default(),
            MoveVertex::Point(PointDto { x: 0, y: 0 }),
        )
        .unwrap();
    assert!(existing.edit.is_none());
    let noop = document
        .set_personal_comment_with_history(&path(&[0]), &path(&[0]), "")
        .unwrap();
    assert!(noop.edit.is_none());
    assert!(document
        .play_with_history(
            &path(&[0]),
            &path(&[0]),
            MoveVertex::Point(PointDto { x: 0, y: 0 })
        )
        .is_err());
    assert!(!history.can_undo());

    let first = document
        .set_personal_comment_with_history(&path(&[0]), &path(&[0]), "first")
        .unwrap();
    history.commit(first.edit.unwrap());
    history.undo(&mut document).unwrap().unwrap();
    assert!(history.can_redo());

    let branch = document
        .set_personal_comment_with_history(&path(&[0]), &path(&[0]), "branch")
        .unwrap();
    history.commit(branch.edit.unwrap());
    assert!(!history.can_redo());
}

#[test]
fn promoting_nested_variation_preserves_nodes_analysis_and_reversible_cursor() {
    let source = "(;GM[1]FF[4]SZ[9]XY[root](;B[aa]C[first])(;B[bb]ZZ[branch](;W[cc]C[side])(;W[dd]C[chosen]TR[ee];B[ff])))";
    let mut document = CurrentSgfDocument::open(source).unwrap();
    let mut history = DocumentHistory::default();
    let original = path(&[1, 1, 0]);
    document
        .replace_primary_analysis(&original, &payload(321))
        .unwrap();
    document
        .replace_primary_analysis(&path(&[0]), &payload(654))
        .unwrap();
    let before = document.serialize().unwrap();

    let promoted = document
        .promote_to_main_with_history(&original, &original)
        .unwrap();
    assert_eq!(promoted.snapshot.path, path(&[0, 0, 0]));
    history.commit(promoted.edit.unwrap());
    assert_eq!(
        document.tree().unwrap().children[1].properties[0].values,
        vec!["aa"]
    );
    assert_eq!(
        document.tree().unwrap().children[0].children[1].properties[0].values,
        vec!["cc"]
    );
    assert_eq!(
        document
            .snapshot(&path(&[0, 0, 0]))
            .unwrap()
            .primary_analysis
            .unwrap()
            .visits,
        321
    );
    assert_eq!(
        document
            .snapshot(&path(&[1]))
            .unwrap()
            .primary_analysis
            .unwrap()
            .visits,
        654
    );
    assert!(document.serialize().unwrap().contains("ZZ[branch]"));
    assert!(document.serialize().unwrap().contains("TR[ee]"));

    let undone = history.undo(&mut document).unwrap().unwrap();
    assert_eq!(undone.selected_path, original);
    assert!(undone.structural);
    assert_eq!(document.serialize().unwrap(), before);
    let redone = history.redo(&mut document).unwrap().unwrap();
    assert_eq!(redone.selected_path, path(&[0, 0, 0]));
    assert!(redone.structural);
    assert_eq!(
        document
            .snapshot(&redone.selected_path)
            .unwrap()
            .primary_analysis
            .unwrap()
            .visits,
        321
    );
    assert!(document
        .promote_to_main_with_history(&redone.selected_path, &redone.selected_path)
        .unwrap()
        .edit
        .is_none());
}
