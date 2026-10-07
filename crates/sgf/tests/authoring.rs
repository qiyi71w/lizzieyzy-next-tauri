use app_model::{NodePath, PlayerColor, PointDto, SgfAuthoringActionDto};
use sgf::{CurrentSgfDocument, DocumentHistory, SgfAnalysisPayload};

#[test]
fn insertion_preserves_source_tree_and_exact_cursor_in_one_history_unit() {
    let before = "(;SZ[19];B[aa]C[keep]XX[v](;W[bb];B[cc])(;W[dd]))";
    let after = "(;SZ[19];B[aa]C[keep]XX[v];W[ee](;W[bb];B[cc])(;W[dd]))";
    let selected = NodePath { indices: vec![0] };
    let mut doc = CurrentSgfDocument::open(before).unwrap();
    let outcome = doc
        .author_with_history(
            &selected,
            SgfAuthoringActionDto::Add {
                point: PointDto { x: 4, y: 4 },
                color: Some(PlayerColor::White),
                insert: true,
            },
        )
        .unwrap();
    assert_eq!(outcome.snapshot.path.indices, vec![0, 0]);
    assert_eq!(doc.serialize().unwrap(), after);
    let mut history = DocumentHistory::default();
    history.commit(outcome.edit.unwrap());
    assert_eq!(history.undo(&mut doc).unwrap().unwrap().selected_path, selected);
    assert_eq!(doc.serialize().unwrap(), before);
    assert!(!history.can_undo());
    assert_eq!(
        history.redo(&mut doc).unwrap().unwrap().selected_path.indices,
        vec![0, 0]
    );
    assert_eq!(doc.serialize().unwrap(), after);
    assert_eq!(
        CurrentSgfDocument::open(after).unwrap().tree().unwrap(),
        doc.tree().unwrap()
    );
}

fn point(x: u8, y: u8) -> PointDto {
    PointDto { x, y }
}
fn path(indices: &[u32]) -> NodePath {
    NodePath {
        indices: indices.into(),
    }
}
fn roundtrip(before: &str, selected: &[u32], action: SgfAuthoringActionDto, after: &str, cursor: &[u32]) {
    let mut doc = CurrentSgfDocument::open(before).unwrap();
    let outcome = doc.author_with_history(&path(selected), action).unwrap();
    assert_eq!(outcome.snapshot.path, path(cursor));
    assert!(outcome.snapshot.position.errors.is_empty());
    assert_eq!(doc.serialize().unwrap(), after);
    assert_eq!(
        CurrentSgfDocument::open(after).unwrap().tree().unwrap(),
        doc.tree().unwrap()
    );
    let mut history = DocumentHistory::default();
    history.commit(outcome.edit.unwrap());
    assert_eq!(
        history.undo(&mut doc).unwrap().unwrap().selected_path,
        path(selected)
    );
    assert_eq!(doc.serialize().unwrap(), before);
    assert!(!history.can_undo());
    assert_eq!(
        history.redo(&mut doc).unwrap().unwrap().selected_path,
        path(cursor)
    );
    assert_eq!(doc.serialize().unwrap(), after);
    assert!(!history.can_redo());
}

#[test]
fn forced_black_white_and_alternate_are_actual_moves_not_setup_or_pl() {
    for (color, key) in [
        (Some(PlayerColor::Black), "B"),
        (Some(PlayerColor::White), "W"),
        (None, "W"),
    ] {
        roundtrip(
            "(;SZ[19];B[aa])",
            &[0],
            SgfAuthoringActionDto::Add {
                point: point(1, 1),
                color,
                insert: false,
            },
            &format!("(;SZ[19];B[aa];{key}[bb])"),
            &[0, 0],
        );
    }
}

#[test]
fn source_root_pass_and_approved_nonmove_insertion_anchors_are_preserved() {
    for (before, selected, after, cursor, color, p) in [
        (
            "(;SZ[19])",
            vec![],
            "(;SZ[19];B[aa])",
            vec![0],
            PlayerColor::Black,
            point(0, 0),
        ),
        (
            "(;SZ[19];B[aa];W[bb])",
            vec![],
            "(;SZ[19];B[aa];W[ee];W[bb])",
            vec![0, 0],
            PlayerColor::White,
            point(4, 4),
        ),
        (
            "(;SZ[19];B[aa];W[];B[cc])",
            vec![0, 0],
            "(;SZ[19];B[aa];W[];W[ee];B[cc])",
            vec![0, 0, 0],
            PlayerColor::White,
            point(4, 4),
        ),
        (
            "(;SZ[19];B[aa];AB[bb]C[setup]PL[W];W[cc])",
            vec![0, 0],
            "(;SZ[19];B[aa];AB[bb]C[setup]PL[W];W[ee];W[cc])",
            vec![0, 0, 0],
            PlayerColor::White,
            point(4, 4),
        ),
        (
            "(;SZ[19];B[aa];C[note]PL[B](;W[bb])(;W[dd]))",
            vec![0, 0],
            "(;SZ[19];B[aa];C[note]PL[B];B[ee](;W[bb])(;W[dd]))",
            vec![0, 0, 0],
            PlayerColor::Black,
            point(4, 4),
        ),
        (
            "(;SZ[19];B[aa];AB[bb]C[setup]PL[W])",
            vec![0, 0],
            "(;SZ[19];B[aa];AB[bb]C[setup]PL[W];W[ee])",
            vec![0, 0, 0],
            PlayerColor::White,
            point(4, 4),
        ),
    ] {
        roundtrip(
            before,
            &selected,
            SgfAuthoringActionDto::Add {
                point: p,
                color: Some(color),
                insert: true,
            },
            after,
            &cursor,
        );
    }
}

#[test]
fn recorded_and_starting_stone_drag_keep_original_node_color_properties_and_cursor() {
    roundtrip(
        "(;SZ[19];B[aa]C[x];W[bb])",
        &[0, 0],
        SgfAuthoringActionDto::Drag {
            from: point(0, 0),
            to: point(2, 2),
        },
        "(;SZ[19];B[cc]C[x];W[bb])",
        &[0, 0],
    );
    roundtrip(
        "(;SZ[19]AB[aa]PL[W]C[x];W[bb])",
        &[0],
        SgfAuthoringActionDto::Drag {
            from: point(0, 0),
            to: point(2, 2),
        },
        "(;SZ[19]AB[cc]PL[W]C[x];W[bb])",
        &[0],
    );
    roundtrip(
        "(;SZ[5:3]AB[aa:ba]PL[W]XX[v];W[cc])",
        &[0],
        SgfAuthoringActionDto::Drag {
            from: point(0, 0),
            to: point(4, 2),
        },
        "(;SZ[5:3]AB[ec][ba]PL[W]XX[v];W[cc])",
        &[0],
    );
}

#[test]
fn occupied_bounds_and_any_invalid_descendant_refuse_without_tree_or_analysis_loss() {
    for (before, selected, action) in [
        (
            "(;SZ[5:3];B[aa]C[x]LZ[opaque](;W[bb])(;W[cc]XX[v]))",
            vec![0],
            SgfAuthoringActionDto::Add {
                point: point(2, 2),
                color: Some(PlayerColor::Black),
                insert: true,
            },
        ),
        (
            "(;SZ[5:3];B[aa];W[bb])",
            vec![0, 0],
            SgfAuthoringActionDto::Drag {
                from: point(0, 0),
                to: point(1, 1),
            },
        ),
        (
            "(;SZ[5:3];B[aa](;W[bb])(;W[cc]))",
            vec![0],
            SgfAuthoringActionDto::Drag {
                from: point(0, 0),
                to: point(2, 2),
            },
        ),
        (
            "(;SZ[5:3];B[aa])",
            vec![0],
            SgfAuthoringActionDto::Add {
                point: point(1, 3),
                color: None,
                insert: false,
            },
        ),
        (
            "(;SZ[5:3];B[aa])",
            vec![0],
            SgfAuthoringActionDto::Drag {
                from: point(0, 0),
                to: point(5, 0),
            },
        ),
    ] {
        let mut doc = CurrentSgfDocument::open(before).unwrap();
        let snapshot = doc.snapshot(&path(&selected)).unwrap();
        assert!(doc.author_with_history(&path(&selected), action).is_err());
        assert_eq!(doc.serialize().unwrap(), before);
        assert_eq!(doc.snapshot(&path(&selected)).unwrap(), snapshot);
    }
    let mut doc = CurrentSgfDocument::open("(;SZ[5:3]AB[aa])").unwrap();
    assert!(doc
        .author_with_history(
            &path(&[]),
            SgfAuthoringActionDto::Drag {
                from: point(0, 0),
                to: point(0, 0)
            }
        )
        .unwrap()
        .edit
        .is_none());
}

#[test]
fn all_branch_geometry_handles_supported_compressed_points_labels_lines_and_first_white_pass() {
    use app_model::SgfTransformDto::*;
    let before = "(;SZ[5]AB[aa:bb]AW[cc]AE[dd]LB[aa:B:W]CR[bb:cc]SQ[dd]MA[ee]TR[ab]SL[ba]DD[cb]VW[aa:ee]TB[de]TW[ed]AR[aa:bc]LN[cd:de]XX[B W]C[black white](;W[];B[ce])(;W[ea]))";
    let after = "(;SZ[5]AB[da:eb]AW[cc]AE[bd]LB[ea:B:W]CR[cb:dc]SQ[bd]MA[ae]TR[da]SL[eb]DD[dc]VW[aa:ee]TB[ad]TW[be]AR[ea:cb]LN[bc:ad]XX[B W]C[black white](;W[];B[ac])(;W[ee]))";
    roundtrip(
        before,
        &[1],
        SgfAuthoringActionDto::Transform {
            transform: RotateClockwise,
        },
        after,
        &[1],
    );
    roundtrip(
        "(;SZ[5:3]AB[aa:bb]LB[aa:x](;W[];B[ec])(;B[ca]))",
        &[0, 0],
        SgfAuthoringActionDto::Transform {
            transform: MirrorHorizontal,
        },
        "(;SZ[5:3]AB[da:eb]LB[ea:x](;W[];B[ac])(;B[ca]))",
        &[0, 0],
    );
    roundtrip(
        "(;SZ[5:3];W[];B[ec])",
        &[0, 0],
        SgfAuthoringActionDto::Transform {
            transform: MirrorVertical,
        },
        "(;SZ[5:3];W[];B[ea])",
        &[0, 0],
    );
    roundtrip(
        "(;SZ[5];W[];B[ce])",
        &[0, 0],
        SgfAuthoringActionDto::Transform {
            transform: RotateCounterclockwise,
        },
        "(;SZ[5];W[];B[ec])",
        &[0, 0],
    );
}

#[test]
fn color_exchange_swaps_owned_metadata_and_result_without_rewriting_comment_or_unknown_text() {
    roundtrip("(;SZ[5]PB[B name]PW[W name]BR[1d]WR[2d]BT[one]WT[two]AB[aa]AW[bb]PL[W]RE[B+R]C[B+R W]XX[B W](;W[];B[cc])(;W[dd]BL[30]WL[20]OB[2]OW[3]TB[aa]TW[bb]))", &[0], SgfAuthoringActionDto::Transform {transform:app_model::SgfTransformDto::SwapColors}, "(;SZ[5]PW[B name]PB[W name]WR[1d]BR[2d]WT[one]BT[two]AW[aa]AB[bb]PL[B]RE[W+R]C[B+R W]XX[B W](;B[];W[cc])(;B[dd]WL[30]BL[20]OW[2]OB[3]TW[aa]TB[bb]))", &[0]);
}

#[test]
fn malformed_geometry_results_and_rectangular_rotation_are_atomic_refusals() {
    for (before, transform) in [
        ("(;SZ[5:3];B[aa])", app_model::SgfTransformDto::RotateClockwise),
        (
            "(;SZ[5]LB[bad]C[keep];B[aa])",
            app_model::SgfTransformDto::MirrorHorizontal,
        ),
        (
            "(;SZ[5]AR[aa:zz];B[aa])",
            app_model::SgfTransformDto::MirrorVertical,
        ),
        (
            "(;SZ[5]RE[unknown];B[aa])",
            app_model::SgfTransformDto::SwapColors,
        ),
    ] {
        let mut doc = CurrentSgfDocument::open(before).unwrap();
        assert!(doc
            .author_with_history(&path(&[0]), SgfAuthoringActionDto::Transform { transform })
            .is_err());
        assert_eq!(doc.serialize().unwrap(), before);
    }
}

#[test]
fn ladder_uses_source_five_moves_period_four_three_empty_points_and_commits_legal_prefix_once() {
    roundtrip(
        "(;SZ[5];B[aa]C[keep](;W[ba];B[ab];W[ac];B[bb]XX[v])(;W[ee]))",
        &[0, 0, 0, 0, 0],
        SgfAuthoringActionDto::ContinueLadder,
        "(;SZ[5];B[aa]C[keep](;W[ba];B[ab];W[ac];B[bb]XX[v];W[cb];B[bc];W[bd];B[cc];W[dc];B[cd])(;W[ee]))",
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    );
}

#[test]
fn ladder_below_threshold_pass_non_diagonal_blocked_and_offboard_are_nondestructive() {
    for before in [
        "(;SZ[5];B[aa];W[ba])",
        "(;SZ[5];B[aa];W[ba];B[];W[ac];B[bb])",
        "(;SZ[5];B[aa];W[ba];B[ab];W[ac];B[ca])",
        "(;SZ[5]AW[cb];B[aa];W[ba];B[ab];W[ac];B[bb])",
        "(;SZ[5];B[cc];W[dc];B[cd];W[ce];B[dd])",
    ] {
        let mut doc = CurrentSgfDocument::open(before).unwrap();
        let selected = doc.default_selected_path();
        assert!(
            doc.author_with_history(&selected, SgfAuthoringActionDto::ContinueLadder)
                .is_err(),
            "{before}"
        );
        assert_eq!(doc.serialize().unwrap(), before);
    }
}

#[test]
fn affected_analysis_is_invalidated_and_restored_by_the_same_history_unit() {
    roundtrip(
        "(;SZ[5];B[aa]LZ[old]C[keep](;W[bb]LZ2[child])(;W[cc]XX[v]))",
        &[0],
        SgfAuthoringActionDto::Drag {
            from: point(0, 0),
            to: point(4, 4),
        },
        "(;SZ[5];B[ee]C[keep](;W[bb])(;W[cc]XX[v]))",
        &[0],
    );
    roundtrip(
        "(;SZ[5];B[aa];W[bb]LZ[old])",
        &[0],
        SgfAuthoringActionDto::Add {
            point: point(4, 4),
            color: Some(PlayerColor::White),
            insert: true,
        },
        "(;SZ[5];B[aa];W[ee];W[bb])",
        &[0, 0],
    );
}

fn analysis(visits: u32) -> SgfAnalysisPayload {
    SgfAnalysisPayload {
        engine_name: "KataGo".into(),
        visits,
        winrate_black: 0.55,
        score_mean_black: Some(1.5),
        score_stdev: None,
        pda: None,
        candidates: vec![],
        ownership: None,
    }
}

fn assert_analysis_roundtrip(doc: &CurrentSgfDocument, selected: &NodePath, visits: Option<u32>) {
    let reopened = CurrentSgfDocument::open(&doc.serialize().unwrap()).unwrap();
    for document in [doc, &reopened] {
        let snapshot = document.snapshot(selected).unwrap();
        assert_eq!(snapshot.primary_analysis.map(|payload| payload.visits), visits);
    }
}

#[test]
fn drag_history_exchanges_analysis_attached_to_an_originally_empty_slot() {
    let before = "(;SZ[5];B[aa]C[keep])";
    let selected = path(&[0]);
    let mut doc = CurrentSgfDocument::open(before).unwrap();
    let edit = doc
        .author_with_history(
            &selected,
            SgfAuthoringActionDto::Drag {
                from: point(0, 0),
                to: point(4, 4),
            },
        )
        .unwrap();
    let mut history = DocumentHistory::default();
    history.commit(edit.edit.unwrap());
    doc.replace_primary_analysis(&selected, &analysis(100)).unwrap();
    assert_analysis_roundtrip(&doc, &selected, Some(100));
    let analyzed_edit = doc.serialize().unwrap();

    assert_eq!(history.undo(&mut doc).unwrap().unwrap().selected_path, selected);
    assert_analysis_roundtrip(&doc, &selected, None);
    assert_eq!(doc.serialize().unwrap(), before);
    assert!(!history.can_undo());

    doc.replace_primary_analysis(&selected, &analysis(200)).unwrap();
    let analyzed_original = doc.serialize().unwrap();
    assert_eq!(history.redo(&mut doc).unwrap().unwrap().selected_path, selected);
    assert_analysis_roundtrip(&doc, &selected, Some(100));
    assert_eq!(doc.serialize().unwrap(), analyzed_edit);
    assert!(!history.can_redo());

    history.undo(&mut doc).unwrap().unwrap();
    assert_analysis_roundtrip(&doc, &selected, Some(200));
    assert_eq!(doc.serialize().unwrap(), analyzed_original);
}

fn analysis_history_cycle(
    action: SgfAuthoringActionDto,
    selected: &[u32],
    affected: &[&[u32]],
    unaffected: &[&[u32]],
) {
    let mut doc = CurrentSgfDocument::open(
        "(;SZ[5]PB[Black]PW[White]XX[root];B[aa]C[keep](;W[bb]C[edited];B[cc]XX[leaf]LZ2[secondary])(;W[dd]C[sibling]))",
    )
    .unwrap();
    let selected = path(selected);
    let existing = path(affected[affected.len() - 1]);
    doc.replace_primary_analysis(&existing, &analysis(50)).unwrap();
    let before = doc.serialize().unwrap();
    let edit = doc.author_with_history(&selected, action).unwrap();
    let mut history = DocumentHistory::default();
    history.commit(edit.edit.unwrap());
    for (index, indices) in affected.iter().enumerate() {
        assert_analysis_roundtrip(&doc, &path(indices), None);
        doc.replace_primary_analysis(&path(indices), &analysis(100 + index as u32))
            .unwrap();
    }
    for indices in unaffected {
        doc.replace_primary_analysis(&path(indices), &analysis(900)).unwrap();
    }
    let analyzed_edit = doc.serialize().unwrap();
    assert_eq!(history.undo(&mut doc).unwrap().unwrap().selected_path, selected);
    assert!(!history.can_undo());
    // The original gains only legitimate later analysis on unaffected positions.
    let mut original = CurrentSgfDocument::open(&before).unwrap();
    for indices in unaffected {
        original.replace_primary_analysis(&path(indices), &analysis(900)).unwrap();
    }
    assert_eq!(doc.serialize().unwrap(), original.serialize().unwrap());
    for (index, indices) in affected.iter().enumerate() {
        let node = path(indices);
        assert_analysis_roundtrip(&doc, &node, (node == existing).then_some(50));
        doc.replace_primary_analysis(&node, &analysis(200 + index as u32)).unwrap();
    }
    let analyzed_original = doc.serialize().unwrap();
    assert_eq!(history.redo(&mut doc).unwrap().unwrap().selected_path, selected);
    assert!(!history.can_redo());
    assert_eq!(doc.serialize().unwrap(), analyzed_edit);
    for (index, indices) in affected.iter().enumerate() {
        assert_analysis_roundtrip(&doc, &path(indices), Some(100 + index as u32));
    }
    history.undo(&mut doc).unwrap().unwrap();
    assert_eq!(doc.serialize().unwrap(), analyzed_original);
    for (index, indices) in affected.iter().enumerate() {
        assert_analysis_roundtrip(&doc, &path(indices), Some(200 + index as u32));
    }
    for indices in unaffected {
        assert_analysis_roundtrip(&doc, &path(indices), Some(900));
    }
}

#[test]
fn drag_analysis_inverse_covers_empty_descendants_but_keeps_later_unrelated_analysis() {
    analysis_history_cycle(
        SgfAuthoringActionDto::Drag {
            from: point(1, 1),
            to: point(4, 4),
        },
        &[0, 0, 0],
        &[&[0, 0], &[0, 0, 0]],
        &[&[], &[0], &[0, 1]],
    );
}

#[test]
fn every_transform_exchanges_root_and_all_branch_analysis_in_both_directions() {
    use app_model::SgfTransformDto::*;
    for transform in [RotateClockwise, RotateCounterclockwise, MirrorHorizontal, MirrorVertical, SwapColors] {
        analysis_history_cycle(
            SgfAuthoringActionDto::Transform { transform },
            &[0, 1],
            &[&[], &[0], &[0, 0], &[0, 1], &[0, 0, 0]],
            &[],
        );
    }
}

#[test]
fn redo_clears_analysis_attached_after_undo_when_edited_slots_remained_empty() {
    let selected = path(&[0]);
    let mut doc = CurrentSgfDocument::open("(;SZ[5];B[aa]C[keep])").unwrap();
    let edit = doc
        .author_with_history(
            &selected,
            SgfAuthoringActionDto::Transform {
                transform: app_model::SgfTransformDto::MirrorHorizontal,
            },
        )
        .unwrap();
    let mut history = DocumentHistory::default();
    history.commit(edit.edit.unwrap());
    history.undo(&mut doc).unwrap().unwrap();
    doc.replace_primary_analysis(&selected, &analysis(200)).unwrap();
    let analyzed_original = doc.serialize().unwrap();
    history.redo(&mut doc).unwrap().unwrap();
    assert_analysis_roundtrip(&doc, &selected, None);
    assert_eq!(doc.serialize().unwrap(), "(;SZ[5];B[ea]C[keep])");
    history.undo(&mut doc).unwrap().unwrap();
    assert_analysis_roundtrip(&doc, &selected, Some(200));
    assert_eq!(doc.serialize().unwrap(), analyzed_original);
}

#[test]
fn insertion_exchanges_analysis_at_rebased_descendant_paths_and_retains_new_node_payload() {
    let selected = path(&[0]);
    let mut doc = CurrentSgfDocument::open(
        "(;SZ[5]XX[root];B[aa]C[keep](;W[bb];B[cc]XX[leaf]LZ2[secondary])(;W[dd]C[sibling]))",
    )
    .unwrap();
    doc.replace_primary_analysis(&path(&[0, 0, 0]), &analysis(50)).unwrap();
    let before = doc.serialize().unwrap();
    let edit = doc
        .author_with_history(
            &selected,
            SgfAuthoringActionDto::Add {
                point: point(4, 4),
                color: Some(PlayerColor::White),
                insert: true,
            },
        )
        .unwrap();
    let inserted = path(&[0, 0]);
    assert_eq!(edit.snapshot.path, inserted);
    let mut history = DocumentHistory::default();
    history.commit(edit.edit.unwrap());
    doc.replace_primary_analysis(&selected, &analysis(900)).unwrap();
    let rebased: &[&[u32]] = &[&[0, 0, 0], &[0, 0, 0, 0], &[0, 0, 1]];
    let original: &[&[u32]] = &[&[0, 0], &[0, 0, 0], &[0, 1]];
    doc.replace_primary_analysis(&inserted, &analysis(700)).unwrap();
    for (index, indices) in rebased.iter().enumerate() {
        doc.replace_primary_analysis(&path(indices), &analysis(100 + index as u32)).unwrap();
    }
    let analyzed_edit = doc.serialize().unwrap();
    assert_eq!(history.undo(&mut doc).unwrap().unwrap().selected_path, selected);
    assert!(!history.can_undo());
    let mut expected = CurrentSgfDocument::open(&before).unwrap();
    expected.replace_primary_analysis(&selected, &analysis(900)).unwrap();
    assert_eq!(doc.serialize().unwrap(), expected.serialize().unwrap());
    for (index, indices) in original.iter().enumerate() {
        assert_analysis_roundtrip(&doc, &path(indices), (index == 1).then_some(50));
        doc.replace_primary_analysis(&path(indices), &analysis(200 + index as u32)).unwrap();
    }
    let analyzed_original = doc.serialize().unwrap();
    assert_eq!(history.redo(&mut doc).unwrap().unwrap().selected_path, inserted);
    assert!(!history.can_redo());
    assert_eq!(doc.serialize().unwrap(), analyzed_edit);
    assert_analysis_roundtrip(&doc, &inserted, Some(700));
    for (index, indices) in rebased.iter().enumerate() {
        assert_analysis_roundtrip(&doc, &path(indices), Some(100 + index as u32));
    }
    history.undo(&mut doc).unwrap().unwrap();
    assert_eq!(doc.serialize().unwrap(), analyzed_original);
    for (index, indices) in original.iter().enumerate() {
        assert_analysis_roundtrip(&doc, &path(indices), Some(200 + index as u32));
    }
    assert_analysis_roundtrip(&doc, &selected, Some(900));
}

#[test]
fn rule_errors_and_extreme_coordinates_reject_the_entire_candidate() {
    for (before, action) in [
        (
            "(;SZ[5];B[aa];W[aa];B[ab];W[ac];B[bb])",
            SgfAuthoringActionDto::ContinueLadder,
        ),
        (
            "(;SZ[5];B[aa])",
            SgfAuthoringActionDto::Add {
                point: point(255, 255),
                color: None,
                insert: false,
            },
        ),
        (
            "(;SZ[5];B[aa])",
            SgfAuthoringActionDto::Drag {
                from: point(0, 0),
                to: point(255, 255),
            },
        ),
    ] {
        let mut doc = CurrentSgfDocument::open(before).unwrap();
        let selected = doc.default_selected_path();
        assert!(doc.author_with_history(&selected, action).is_err());
        assert_eq!(doc.serialize().unwrap(), before);
    }
}
