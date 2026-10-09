use app_model::{
    CandidateMoveDto, MoveVertex, NodePath, PointDto, SgfMarkupActionDto, SgfMarkupDto, SgfMarkupToolDto,
};
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
            score_mean_black: Some(1.5),
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

#[test]
fn markup_replaces_compressed_points_and_roundtrips_without_clobbering_analysis() {
    let source = "(;GM[1]FF[4]SZ[5:4]XY[root];B[aa]C[note]ZZ[keep]CR[aa:cc]TR[dd]LB[ab:old]QQ[foreign](;W[bb])(;W[cb]))";
    let mut document = CurrentSgfDocument::open(source).unwrap();
    let node = path(&[0]);
    let branch = path(&[0, 1]);
    let mut history = DocumentHistory::default();
    let action = |x, y, tool| SgfMarkupActionDto::Point {
        point: PointDto { x, y },
        tool,
    };

    let first = document
        .edit_markup_with_history(
            &branch,
            &node,
            action(
                1,
                1,
                SgfMarkupToolDto::Label {
                    text: "a]\\b:中".into(),
                },
            ),
        )
        .unwrap();
    assert_eq!(
        first
            .snapshot
            .markup
            .iter()
            .filter(
                |mark| matches!(mark, SgfMarkupDto::Circle { point } if *point == PointDto { x: 1, y: 1 })
            )
            .count(),
        0
    );
    history.commit(first.edit.unwrap());
    let duplicate = document
        .edit_markup_with_history(
            &node,
            &node,
            action(
                1,
                1,
                SgfMarkupToolDto::Label {
                    text: "a]\\b:中".into(),
                },
            ),
        )
        .unwrap();
    assert!(duplicate.edit.is_none());
    let letter = document
        .edit_markup_with_history(&node, &node, action(2, 1, SgfMarkupToolDto::Letters))
        .unwrap();
    history.commit(letter.edit.unwrap());
    let number = document
        .edit_markup_with_history(&node, &node, action(3, 1, SgfMarkupToolDto::Numbers))
        .unwrap();
    history.commit(number.edit.unwrap());
    assert!(document.snapshot(&node).unwrap().markup.iter().any(|mark| matches!(mark, SgfMarkupDto::Label { point, text } if *point == PointDto { x: 2, y: 1 } && text == "A")));

    document.replace_primary_analysis(&node, &payload(73)).unwrap();
    let cleared = document
        .edit_markup_with_history(&node, &node, SgfMarkupActionDto::Clear)
        .unwrap();
    assert!(cleared.snapshot.markup.is_empty());
    history.commit(cleared.edit.unwrap());
    let text = document.serialize().unwrap();
    assert!(text.contains("C[note]"));
    assert!(text.contains("ZZ[keep]"));
    assert!(text.contains("QQ[foreign]"));
    assert_eq!(document.snapshot(&branch).unwrap().markup.len(), 0);

    let undone = history.undo(&mut document).unwrap().unwrap();
    assert!(!undone.structural);
    assert_eq!(undone.selected_path, node);
    let restored = document.snapshot(&node).unwrap();
    assert!(restored
        .markup
        .iter()
        .any(|mark| matches!(mark, SgfMarkupDto::Label { text, .. } if text == "a]\\b:中")));
    assert_eq!(restored.primary_analysis.unwrap().visits, 73);
    let reopened = CurrentSgfDocument::open(&document.serialize().unwrap()).unwrap();
    assert_eq!(reopened.snapshot(&node).unwrap().markup, restored.markup);
    history.redo(&mut document).unwrap().unwrap();
    assert!(document.snapshot(&node).unwrap().markup.is_empty());
}

#[test]
fn markup_rejects_out_of_bounds_without_edit_or_serialization_change() {
    let mut document = CurrentSgfDocument::open("(;SZ[5:4]C[keep]XY[keep])").unwrap();
    let before = document.serialize().unwrap();
    assert!(document
        .edit_markup_with_history(
            &path(&[]),
            &path(&[]),
            SgfMarkupActionDto::Point {
                point: PointDto { x: 4, y: 4 },
                tool: SgfMarkupToolDto::Circle
            }
        )
        .is_err());
    assert_eq!(document.serialize().unwrap(), before);
    assert!(document
        .edit_markup_with_history(&path(&[]), &path(&[]), SgfMarkupActionDto::Clear)
        .unwrap()
        .edit
        .is_none());
}

#[test]
fn shape_tools_replace_at_one_point_and_erase_is_idempotent() {
    let mut document = CurrentSgfDocument::open("(;SZ[5:4]C[leave]TR[aa]SQ[dd]AB[bb])").unwrap();
    let root = path(&[]);
    let point = PointDto { x: 0, y: 0 };
    for (tool, kind) in [
        (SgfMarkupToolDto::Square, "square"),
        (SgfMarkupToolDto::Cross, "cross"),
        (SgfMarkupToolDto::Triangle, "triangle"),
        (SgfMarkupToolDto::Circle, "circle"),
    ] {
        let action = SgfMarkupActionDto::Point { point, tool };
        let updated = document
            .edit_markup_with_history(&root, &root, action.clone())
            .unwrap();
        assert!(updated.edit.is_some());
        assert_eq!(
            updated
                .snapshot
                .markup
                .iter()
                .filter(|mark| match mark {
                    SgfMarkupDto::Label { point: at, .. }
                    | SgfMarkupDto::Circle { point: at }
                    | SgfMarkupDto::Square { point: at }
                    | SgfMarkupDto::Cross { point: at }
                    | SgfMarkupDto::Triangle { point: at } => *at == point,
                })
                .count(),
            1,
            "{kind}"
        );
        assert!(
            document
                .edit_markup_with_history(&root, &root, action)
                .unwrap()
                .edit
                .is_none(),
            "{kind}"
        );
    }
    let erase = SgfMarkupActionDto::Point {
        point,
        tool: SgfMarkupToolDto::Erase,
    };
    assert!(document
        .edit_markup_with_history(&root, &root, erase.clone())
        .unwrap()
        .edit
        .is_some());
    assert!(document
        .edit_markup_with_history(&root, &root, erase)
        .unwrap()
        .edit
        .is_none());
    let snapshot = document.snapshot(&root).unwrap();
    assert_eq!(snapshot.markup.len(), 1);
    assert_eq!(snapshot.personal_comment, "leave");
    assert!(document.serialize().unwrap().contains("AB[bb]"));
}

#[test]
fn sequential_tools_replace_imported_labels_but_repeat_is_noop() {
    for (imported, tool, expected) in [
        ("HELLO", SgfMarkupToolDto::Letters, "A"),
        ("999", SgfMarkupToolDto::Numbers, "1"),
    ] {
        let mut document = CurrentSgfDocument::open(&format!("(;SZ[5]C[note]LB[aa:{imported}])")).unwrap();
        let root = path(&[]);
        let action = SgfMarkupActionDto::Point {
            point: PointDto { x: 0, y: 0 },
            tool,
        };
        let replaced = document
            .edit_markup_with_history(&root, &root, action.clone())
            .unwrap();
        assert_eq!(
            replaced.snapshot.markup,
            vec![SgfMarkupDto::Label {
                point: PointDto { x: 0, y: 0 },
                text: expected.into()
            }]
        );
        assert!(replaced.edit.is_some());
        assert!(document
            .edit_markup_with_history(&root, &root, action)
            .unwrap()
            .edit
            .is_none());
        assert_eq!(document.snapshot(&root).unwrap().personal_comment, "note");
    }
}
