use app_model::NodePath;
use sgf::{CurrentSgfDocument, SgfAnalysisPayload};

fn path(indices: &[u32]) -> NodePath {
    NodePath {
        indices: indices.to_vec(),
    }
}

fn analysis(visits: u32) -> SgfAnalysisPayload {
    SgfAnalysisPayload {
        engine_name: "KataGo".into(),
        visits,
        winrate_black: 0.61,
        score_mean_black: Some(2.25),
        score_stdev: Some(0.5),
        pda: None,
        candidates: Vec::new(),
        ownership: None,
    }
}

#[test]
fn extension_preserves_browsed_cursor_and_local_comment_and_analysis() {
    let previous = CurrentSgfDocument::open("(;SZ[9]KM[6.5]RU[Chinese];B[aa]C[source];W[bb])").unwrap();
    let mut current = previous.clone();
    let selected = path(&[0]);
    current.set_personal_comment(&selected, "my note").unwrap();
    current
        .replace_primary_analysis(&selected, &analysis(400))
        .unwrap();
    let incoming =
        CurrentSgfDocument::open("(;SZ[9]KM[6.5]RU[Chinese];B[aa]C[new source];W[bb];B[cc])").unwrap();
    let before = current.serialize().unwrap();

    let (merged, relocated) = current
        .reconcile_external_source(&previous, &incoming, &selected, false)
        .unwrap();
    let reopened = CurrentSgfDocument::open(&merged.serialize().unwrap()).unwrap();
    let snapshot = reopened.snapshot(&relocated).unwrap();
    assert_eq!(relocated, selected);
    assert_eq!(snapshot.position.move_number, 1);
    assert_eq!(snapshot.personal_comment, "my note");
    assert_eq!(snapshot.primary_analysis.unwrap().visits, 400);
    assert_eq!(reopened.default_selected_path(), path(&[0, 0, 0]));
    assert_eq!(current.serialize().unwrap(), before);
    assert_eq!(
        incoming.snapshot(&selected).unwrap().personal_comment,
        "new source"
    );
}

#[test]
fn same_count_divergence_drops_node_and_descendant_overlays_but_keeps_ancestors() {
    let previous = CurrentSgfDocument::open("(;SZ[9]KM[6.5]RU[Chinese];B[aa];W[bb];B[cc])").unwrap();
    let mut current = previous.clone();
    for (indices, note, visits) in [
        (&[0][..], "ancestor", 100),
        (&[0, 0][..], "diverged", 200),
        (&[0, 0, 0][..], "descendant", 300),
    ] {
        current.set_personal_comment(&path(indices), note).unwrap();
        current
            .replace_primary_analysis(&path(indices), &analysis(visits))
            .unwrap();
    }
    let incoming =
        CurrentSgfDocument::open("(;SZ[9]KM[6.5]RU[Chinese];B[aa];W[dd]C[incoming];B[cc])").unwrap();
    assert_eq!(
        previous.snapshot(&path(&[0, 0, 0])).unwrap().position.move_number,
        3
    );
    assert_eq!(
        incoming.snapshot(&path(&[0, 0, 0])).unwrap().position.move_number,
        3
    );

    let (merged, relocated) = current
        .reconcile_external_source(&previous, &incoming, &path(&[0, 0, 0]), false)
        .unwrap();
    assert_eq!(relocated, path(&[0]));
    let ancestor = merged.snapshot(&path(&[0])).unwrap();
    assert_eq!(ancestor.personal_comment, "ancestor");
    assert_eq!(ancestor.primary_analysis.unwrap().visits, 100);
    assert_eq!(
        merged.snapshot(&path(&[0, 0])).unwrap().personal_comment,
        "incoming"
    );
    assert!(merged
        .snapshot(&path(&[0, 0]))
        .unwrap()
        .primary_analysis
        .is_none());
    let descendant = merged.snapshot(&path(&[0, 0, 0])).unwrap();
    assert_eq!(descendant.personal_comment, "");
    assert!(descendant.primary_analysis.is_none());
}

#[test]
fn root_semantic_revisions_invalidate_overlays_everywhere_and_relocate_to_tip() {
    let previous = CurrentSgfDocument::open("(;SZ[9]KM[6.5]RU[Chinese]HA[0];B[cc];W[dd])").unwrap();
    let mut current = previous.clone();
    for indices in [&[][..], &[0][..], &[0, 0][..]] {
        current.set_personal_comment(&path(indices), "local").unwrap();
        current
            .replace_primary_analysis(&path(indices), &analysis(400))
            .unwrap();
    }
    for root in [
        "SZ[9]KM[7.5]RU[Chinese]HA[0]",
        "SZ[9]KM[6.5]RU[Japanese]HA[0]",
        "SZ[9]KM[6.5]RU[Chinese]HA[2]",
        "SZ[13]KM[6.5]RU[Chinese]HA[0]",
        "SZ[9]KM[6.5]RU[Chinese]HA[0]AB[aa][bb]",
        "SZ[9]KM[6.5]RU[Chinese]HA[0]AW[aa]",
        "SZ[9]KM[6.5]RU[Chinese]HA[0]AE[aa]",
        "SZ[9]KM[6.5]RU[Chinese]HA[0]PL[W]",
    ] {
        let incoming =
            CurrentSgfDocument::open(&format!("(;{root}C[source root];B[cc]C[source move];W[dd])")).unwrap();
        let (merged, relocated) = current
            .reconcile_external_source(&previous, &incoming, &path(&[0]), false)
            .unwrap();
        assert_eq!(relocated, path(&[0, 0]), "{root}");
        assert_eq!(
            merged.serialize().unwrap(),
            incoming.serialize().unwrap(),
            "{root}"
        );
        assert!(
            merged.snapshot(&path(&[0])).unwrap().primary_analysis.is_none(),
            "{root}"
        );
        assert!(
            merged.snapshot(&path(&[])).unwrap().primary_analysis.is_none(),
            "{root}"
        );
    }
}

#[test]
fn source_results_comments_metadata_and_all_branches_win_when_not_locally_changed() {
    let previous = CurrentSgfDocument::open(
        "(;SZ[9]KM[6.5]RU[Chinese]RE[W+R]C[old root];B[aa]C[old move](;W[bb])(;W[cc]))",
    )
    .unwrap();
    let current = previous.clone();
    let incoming = CurrentSgfDocument::open("(;SZ[9]KM[6.5]RU[Chinese]RE[B+2.5]PB[New black]C[new root]XX[keep];B[aa]C[new move](;W[bb]C[main];B[dd])(;W[cc]C[branch])(;W[ee]C[new branch]))").unwrap();

    let (merged, relocated) = current
        .reconcile_external_source(&previous, &incoming, &path(&[0, 1]), false)
        .unwrap();
    let reopened = CurrentSgfDocument::open(&merged.serialize().unwrap()).unwrap();
    assert_eq!(relocated, path(&[0, 1]));
    assert_eq!(reopened.result(), Some("B+2.5"));
    assert_eq!(reopened.serialize().unwrap(), incoming.serialize().unwrap());
    assert_eq!(
        reopened.snapshot(&path(&[])).unwrap().personal_comment,
        "new root"
    );
    assert_eq!(
        reopened.snapshot(&path(&[0])).unwrap().personal_comment,
        "new move"
    );
    assert_eq!(
        reopened.snapshot(&path(&[0, 1])).unwrap().personal_comment,
        "branch"
    );
    assert_eq!(
        reopened.snapshot(&path(&[0, 2])).unwrap().personal_comment,
        "new branch"
    );
    assert_eq!(reopened.tree().unwrap().children[0].children.len(), 3);
}

#[test]
fn explicitly_cleared_local_comment_survives_a_source_comment_revision() {
    let previous = CurrentSgfDocument::open("(;SZ[9];B[aa]C[old source])").unwrap();
    let mut current = previous.clone();
    current.set_personal_comment(&path(&[0]), "").unwrap();
    let incoming = CurrentSgfDocument::open("(;SZ[9];B[aa]C[new source];W[bb])").unwrap();

    let (merged, _) = current
        .reconcile_external_source(&previous, &incoming, &path(&[0]), false)
        .unwrap();
    let reopened = CurrentSgfDocument::open(&merged.serialize().unwrap()).unwrap();
    assert_eq!(reopened.snapshot(&path(&[0])).unwrap().personal_comment, "");
    assert!(!merged.serialize().unwrap().contains("C["));
}

#[test]
fn removed_selected_branch_relocates_to_nearest_matching_ancestor() {
    let previous = CurrentSgfDocument::open("(;SZ[9];B[aa](;W[bb])(;W[cc];B[dd]))").unwrap();
    let incoming = CurrentSgfDocument::open("(;SZ[9];B[aa];W[bb];B[ee])").unwrap();

    let (merged, relocated) = previous
        .reconcile_external_source(&previous, &incoming, &path(&[0, 1, 0]), false)
        .unwrap();
    assert_eq!(relocated, path(&[0]));
    assert_eq!(merged.snapshot(&relocated).unwrap().position.move_number, 1);
    assert_eq!(merged.serialize().unwrap(), incoming.serialize().unwrap());
}

#[test]
fn jump_to_last_uses_incoming_tip_even_when_browsed_path_still_matches() {
    let previous = CurrentSgfDocument::open("(;SZ[9];B[aa];W[bb])").unwrap();
    let incoming = CurrentSgfDocument::open("(;SZ[9];B[aa];W[bb];B[cc])").unwrap();

    let (merged, relocated) = previous
        .reconcile_external_source(&previous, &incoming, &path(&[0]), true)
        .unwrap();
    assert_eq!(relocated, path(&[0, 0, 0]));
    assert_eq!(merged.snapshot(&relocated).unwrap().position.move_number, 3);
}

#[test]
fn unchanged_source_analysis_is_not_overlaid_and_local_secondary_is_preserved_independently() {
    let previous = CurrentSgfDocument::open("(;SZ[9];B[aa]LZ[Main 39.0 100]LZ2[Sub 39.0 100])").unwrap();
    let current = CurrentSgfDocument::open("(;SZ[9];B[aa]LZ[Main 39.0 100]LZ2[Sub 39.0 400])").unwrap();
    let incoming =
        CurrentSgfDocument::open("(;SZ[9];B[aa]LZ[Main 39.0 200]LZ2[Sub 39.0 200];W[bb])").unwrap();

    let (merged, _) = current
        .reconcile_external_source(&previous, &incoming, &path(&[0]), false)
        .unwrap();
    let reopened = CurrentSgfDocument::open(&merged.serialize().unwrap()).unwrap();
    let snapshot = reopened.snapshot(&path(&[0])).unwrap();
    assert_eq!(snapshot.primary_analysis.unwrap().visits, 200);
    assert_eq!(snapshot.secondary_analysis.unwrap().visits, 400);
}

#[test]
fn locally_attached_root_analysis_survives_a_matching_source_extension() {
    let previous = CurrentSgfDocument::open("(;SZ[9]KM[6.5]RU[Chinese];B[aa])").unwrap();
    let mut current = previous.clone();
    current
        .replace_primary_analysis(&path(&[]), &analysis(400))
        .unwrap();
    let incoming = CurrentSgfDocument::open("(;SZ[9]KM[6.5]RU[Chinese];B[aa];W[bb])").unwrap();

    let (merged, relocated) = current
        .reconcile_external_source(&previous, &incoming, &path(&[]), false)
        .unwrap();
    assert_eq!(relocated, path(&[]));
    assert_eq!(
        merged
            .snapshot(&relocated)
            .unwrap()
            .primary_analysis
            .unwrap()
            .visits,
        400
    );
    assert!(merged.serialize().unwrap().contains("LZOP[KataGo"));
}

#[test]
fn intermediate_setup_and_player_changes_invalidate_only_the_changed_ancestry() {
    let previous = CurrentSgfDocument::open("(;SZ[9];B[aa];W[bb];B[cc])").unwrap();
    let mut current = previous.clone();
    current.set_personal_comment(&path(&[0]), "ancestor").unwrap();
    current
        .set_personal_comment(&path(&[0, 0]), "changed node")
        .unwrap();
    current
        .replace_primary_analysis(&path(&[0, 0, 0]), &analysis(400))
        .unwrap();
    for semantic_property in ["AB[ee]", "AW[ee]", "AE[aa]", "PL[W]"] {
        let incoming =
            CurrentSgfDocument::open(&format!("(;SZ[9];B[aa];W[bb]{semantic_property};B[cc])")).unwrap();
        let (merged, relocated) = current
            .reconcile_external_source(&previous, &incoming, &path(&[0, 0, 0]), false)
            .unwrap();
        assert_eq!(relocated, path(&[0]), "{semantic_property}");
        assert_eq!(merged.snapshot(&relocated).unwrap().personal_comment, "ancestor");
        assert_eq!(merged.snapshot(&path(&[0, 0])).unwrap().personal_comment, "");
        assert!(merged
            .snapshot(&path(&[0, 0, 0]))
            .unwrap()
            .primary_analysis
            .is_none());
    }
}

#[test]
fn equal_boards_and_counts_with_different_history_do_not_preserve_overlays() {
    let previous = CurrentSgfDocument::open("(;SZ[9];B[];W[])").unwrap();
    let mut current = previous.clone();
    current
        .set_personal_comment(&path(&[0, 0]), "old history")
        .unwrap();
    current
        .replace_primary_analysis(&path(&[0, 0]), &analysis(400))
        .unwrap();
    let incoming = CurrentSgfDocument::open("(;SZ[9];W[];B[])").unwrap();
    let old_position = previous.snapshot(&path(&[0, 0])).unwrap().position;
    let new_position = incoming.snapshot(&path(&[0, 0])).unwrap().position;
    assert_eq!(old_position.stones, new_position.stones);
    assert_eq!(old_position.move_number, new_position.move_number);

    let (merged, relocated) = current
        .reconcile_external_source(&previous, &incoming, &path(&[0, 0]), false)
        .unwrap();
    assert_eq!(relocated, path(&[]));
    assert_eq!(merged.serialize().unwrap(), incoming.serialize().unwrap());
}

#[test]
fn local_position_changes_cannot_masquerade_as_an_overlay_on_the_source_position() {
    let previous = CurrentSgfDocument::open("(;SZ[9];B[aa];W[bb])").unwrap();
    let current = CurrentSgfDocument::open("(;SZ[9];B[cc]C[local move];W[bb]LZ[Main 39.0 400])").unwrap();
    let incoming = CurrentSgfDocument::open("(;SZ[9];B[aa];W[bb];B[dd])").unwrap();

    let (merged, relocated) = current
        .reconcile_external_source(&previous, &incoming, &path(&[0, 0]), false)
        .unwrap();
    assert_eq!(relocated, path(&[]));
    assert_eq!(merged.serialize().unwrap(), incoming.serialize().unwrap());
}

#[test]
fn invalid_live_cursor_returns_an_error_instead_of_silently_jumping() {
    let previous = CurrentSgfDocument::open("(;SZ[9];B[aa])").unwrap();
    let incoming = CurrentSgfDocument::open("(;SZ[9];B[aa];W[bb])").unwrap();
    for jump_to_last in [false, true] {
        let error = previous
            .reconcile_external_source(&previous, &incoming, &path(&[7]), jump_to_last)
            .unwrap_err();
        assert_eq!(error.kind, app_model::CurrentGameErrorKind::InvalidNodePath);
    }
}
