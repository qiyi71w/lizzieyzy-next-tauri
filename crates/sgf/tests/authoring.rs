use app_model::{NodePath, PlayerColor, PointDto, SgfAuthoringActionDto};
use sgf::{CurrentSgfDocument, DocumentHistory};

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
