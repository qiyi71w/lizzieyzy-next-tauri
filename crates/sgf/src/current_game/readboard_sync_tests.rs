use super::*;

const FOLLOW: ReadboardViewPreferences = ReadboardViewPreferences {
    always_sync: true,
    jump_to_last: false,
};

/// Rows top to bottom, one digit per column: 0 empty, 1 black, 2 white, 3/4 black/white last move.
fn frame(rows: &[&str], context: ReadboardRemoteContext) -> ReadboardFrame {
    ReadboardFrame {
        width: rows[0].len() as u8,
        height: rows.len() as u8,
        codes: rows
            .iter()
            .flat_map(|row| row.bytes().map(|byte| byte - b'0'))
            .collect(),
        context,
    }
}

fn generic() -> ReadboardRemoteContext {
    ReadboardRemoteContext::generic(false)
}

fn fox(number: u32) -> ReadboardRemoteContext {
    fox_room("room-1", number)
}

fn fox_room(room: &str, number: u32) -> ReadboardRemoteContext {
    generic()
        .with_platform(ReadboardPlatform::Fox)
        .with_room_token(room)
        .with_fox_move_number(Some(number))
}

fn open(sgf: &str) -> CurrentSgfDocument {
    CurrentSgfDocument::open(sgf).unwrap()
}

fn path(indices: &[u32]) -> NodePath {
    NodePath {
        indices: indices.to_vec(),
    }
}

struct Accepted {
    document: Option<CurrentSgfDocument>,
    selected: NodePath,
    source: NodePath,
    move_number: u32,
    rebuilt: bool,
}

fn accepted(outcome: ReadboardSyncOutcome) -> Accepted {
    match outcome {
        ReadboardSyncOutcome::Accepted {
            document,
            selected,
            source,
            source_move_number,
            rebuilt,
        } => Accepted {
            document: document.map(|document| *document),
            selected,
            source,
            move_number: source_move_number,
            rebuilt,
        },
        other => panic!("expected an accepted frame, got {other:?}"),
    }
}

fn apply(
    sync: &mut ReadboardSync,
    document: &CurrentSgfDocument,
    frame: &ReadboardFrame,
) -> ReadboardSyncOutcome {
    sync.apply(Some(document), &document.default_selected_path(), frame, FOLLOW)
        .unwrap()
}

fn root_property(document: &CurrentSgfDocument, key: &str) -> Option<String> {
    document
        .root()
        .unwrap()
        .properties
        .iter()
        .find(|p| p.key == key)
        .and_then(|p| p.values.first().cloned())
}

fn assert_no_history(document: &CurrentSgfDocument) {
    let root = document.root().unwrap();
    assert!(root.children.is_empty(), "a static snapshot never creates nodes");
    assert!(
        !root.properties.iter().any(|p| p.key == "B" || p.key == "W"),
        "no fabricated move or PASS"
    );
}

#[test]
fn marker_move_appends_a_real_move_in_the_frame_colour_without_pass() {
    let document = open("(;SZ[3];B[aa])");
    let mut sync = ReadboardSync::new();
    let result = accepted(apply(
        &mut sync,
        &document,
        &frame(&["100", "040", "000"], generic()),
    ));
    let updated = result.document.unwrap();
    assert_eq!(updated.serialize().unwrap(), "(;SZ[3];B[aa];W[bb])");
    assert_eq!(
        (
            result.source.clone(),
            result.selected,
            result.move_number,
            result.rebuilt
        ),
        (path(&[0, 0]), path(&[0, 0]), 2, false)
    );

    // Java places the marker colour even out of turn; it never inserts a PASS.
    let result = accepted(apply(
        &mut sync,
        &updated,
        &frame(&["100", "020", "004"], generic()),
    ));
    assert_eq!(
        result.document.unwrap().serialize().unwrap(),
        "(;SZ[3];B[aa];W[bb];W[cc])"
    );

    // An unchanged frame is steady: no new content, same source.
    let mut steady = ReadboardSync::new();
    let unchanged = accepted(apply(
        &mut steady,
        &document,
        &frame(&["100", "000", "000"], generic()),
    ));
    assert!(unchanged.document.is_none());
    assert_eq!(unchanged.source, path(&[0]));
}

#[test]
fn conflict_holds_once_and_rebuilds_on_the_same_key_while_a_new_key_restarts() {
    let document = open("(;SZ[3];B[aa];W[ba])");
    let conflict = frame(&["000", "000", "112"], generic());
    let other = frame(&["000", "000", "221"], generic());

    let mut sync = ReadboardSync::new();
    assert!(matches!(
        apply(&mut sync, &document, &conflict),
        ReadboardSyncOutcome::Hold
    ));
    let rebuilt = accepted(apply(&mut sync, &document, &conflict));
    assert!(rebuilt.rebuilt);
    assert_no_history(rebuilt.document.as_ref().unwrap());

    let mut sync = ReadboardSync::new();
    assert!(matches!(
        apply(&mut sync, &document, &conflict),
        ReadboardSyncOutcome::Hold
    ));
    assert!(matches!(
        apply(&mut sync, &document, &other),
        ReadboardSyncOutcome::Hold
    ));
    assert!(accepted(apply(&mut sync, &document, &other)).rebuilt);
}

#[test]
fn a_sync_start_without_history_rebuilds_immediately() {
    let document = open("(;SZ[3])");
    let result = accepted(apply(
        &mut ReadboardSync::new(),
        &document,
        &frame(&["110", "000", "000"], generic()),
    ));
    assert!(result.rebuilt);
    let rebuilt = result.document.unwrap();
    assert_no_history(&rebuilt);
    assert_eq!(rebuilt.snapshot(&path(&[])).unwrap().position.stones.len(), 2);
}

#[test]
fn single_move_recovery_applies_a_legal_capture() {
    let document = open("(;SZ[3];B[ba];W[aa])");
    let result = accepted(apply(
        &mut ReadboardSync::new(),
        &document,
        &frame(&["010", "300", "000"], generic()),
    ));
    assert!(!result.rebuilt);
    assert_eq!(
        result.document.unwrap().serialize().unwrap(),
        "(;SZ[3];B[ba];W[aa];B[ab])"
    );
    assert_eq!(result.source, path(&[0, 0, 0]));
}

#[test]
fn single_move_recovery_refuses_a_ko_recapture_and_holds() {
    // White just took the ko at bb; the frame shows Black retaking it immediately.
    let document = open("(;SZ[4]AB[ba][ab][bc][cb]AW[ca][db][cc]PL[W];W[bb])");
    let retake = frame(&["0120", "1032", "0120", "0000"], generic());
    assert!(matches!(
        apply(&mut ReadboardSync::new(), &document, &retake),
        ReadboardSyncOutcome::Hold
    ));
}

#[test]
fn generic_recovery_never_matches_an_arbitrary_same_board_variation() {
    let document = open("(;SZ[3];B[aa](;W[ba])(;W[ca]))");
    let variation_board = frame(&["104", "000", "000"], generic());
    assert!(matches!(
        apply(&mut ReadboardSync::new(), &document, &variation_board),
        ReadboardSyncOutcome::Hold
    ));
}

#[test]
fn fox_number_is_snapshot_metadata_and_does_not_rebuild_again() {
    let document = open("(;SZ[3])");
    let board = frame(&["120", "000", "000"], fox(58));
    let mut sync = ReadboardSync::new();
    let first = accepted(apply(&mut sync, &document, &board));
    assert!(first.rebuilt);
    assert_eq!(first.move_number, 58);
    let rebuilt = first.document.unwrap();
    assert_no_history(&rebuilt);

    let again = accepted(apply(&mut sync, &rebuilt, &board));
    assert!(
        again.document.is_none(),
        "an unchanged Fox frame must not rebuild every time"
    );
    assert_eq!(again.move_number, 58);

    // The next Fox move is a real move numbered from the snapshot metadata.
    let next = accepted(apply(
        &mut sync,
        &rebuilt,
        &frame(&["120", "000", "001"], fox(59)),
    ));
    assert_eq!(
        next.document
            .unwrap()
            .serialize()
            .unwrap()
            .matches(";B[cc]")
            .count(),
        1
    );
    assert_eq!(next.move_number, 59);
}

#[test]
fn fox_zero_handicap_turn_depends_on_the_marker_source() {
    let document = open("(;SZ[3])");
    let handicap = |source| frame(&["100", "000", "001"], fox(0).with_last_move_source(source));
    let untrusted = accepted(apply(
        &mut ReadboardSync::new(),
        &document,
        &handicap(ReadboardLastMoveSource::Unknown),
    ));
    assert_eq!(
        root_property(untrusted.document.as_ref().unwrap(), "PL").as_deref(),
        Some("W")
    );
    assert_eq!(untrusted.move_number, 0);
    // A trusted token without an actual marker keeps the baseline (black on the empty start).
    let trusted = accepted(apply(
        &mut ReadboardSync::new(),
        &document,
        &handicap(ReadboardLastMoveSource::RedBlueMarker),
    ));
    assert_eq!(
        root_property(trusted.document.as_ref().unwrap(), "PL").as_deref(),
        Some("B")
    );
}

#[test]
fn only_a_trusted_visual_marker_decides_the_side_to_play() {
    let document = open("(;SZ[3])");
    let marked = |source| frame(&["320", "000", "000"], generic().with_last_move_source(source));
    let trusted = accepted(apply(
        &mut ReadboardSync::new(),
        &document,
        &marked(ReadboardLastMoveSource::FoxCornerFlip),
    ));
    assert_eq!(
        root_property(trusted.document.as_ref().unwrap(), "PL").as_deref(),
        Some("W")
    );
    let heuristic = accepted(apply(
        &mut ReadboardSync::new(),
        &document,
        &marked(ReadboardLastMoveSource::Deviation),
    ));
    assert_eq!(
        root_property(heuristic.document.as_ref().unwrap(), "PL").as_deref(),
        Some("B")
    );
    // Marker with a baseline: pure additions accumulate onto the sync-start move number.
    assert_eq!(heuristic.move_number, 2);
}

#[test]
fn room_change_rebuild_takes_the_side_to_play_from_a_trusted_marker_after_a_colour_flip() {
    // The old room ends with White's move (black to play). In the new room the first point differs in
    // colour before the trusted marker, which shows Black's last move: white is to play there.
    let document = open("(;SZ[3];B[aa];W[ba])");
    let new_room = frame(
        &["200", "000", "003"],
        fox_room("room-2", 10).with_last_move_source(ReadboardLastMoveSource::FoxCornerFlip),
    );
    let result = accepted(apply(&mut ReadboardSync::new(), &document, &new_room));
    assert!(result.rebuilt);
    assert_eq!(result.move_number, 10);
    let rebuilt = result.document.unwrap();
    assert_no_history(&rebuilt);
    assert_eq!(root_property(&rebuilt, "PL").as_deref(), Some("W"));
    // An untrusted marker source still keeps the old baseline.
    let heuristic = frame(
        &["200", "000", "003"],
        fox_room("room-2", 10).with_last_move_source(ReadboardLastMoveSource::Deviation),
    );
    let kept = accepted(apply(&mut ReadboardSync::new(), &document, &heuristic));
    assert_eq!(
        root_property(kept.document.as_ref().unwrap(), "PL").as_deref(),
        Some("B")
    );
}

#[test]
fn ordinary_fox_parity_fallback_requires_matching_colour_and_no_setup_risk() {
    let pl = |sgf: &str, rows: &[&str], number| {
        let result = accepted(apply(
            &mut ReadboardSync::new(),
            &open(sgf),
            &frame(rows, fox(number)),
        ));
        root_property(result.document.as_ref().unwrap(), "PL")
    };
    let added_white = ["120", "000", "000"];
    // Even number and a white single addition: parity says black to play.
    assert_eq!(pl("(;SZ[3];B[aa])", &added_white, 6).as_deref(), Some("B"));
    // Odd number expects a black addition; the white addition keeps the baseline (white to play).
    assert_eq!(pl("(;SZ[3];B[aa])", &added_white, 5).as_deref(), Some("W"));
    // Root setup is a handicap/setup risk: the same single addition keeps the baseline.
    assert_eq!(
        pl("(;SZ[3]AB[cc];B[aa])", &["120", "000", "001"], 6).as_deref(),
        Some("W")
    );
}

#[test]
fn rebuild_keeps_game_info_anchor_comment_explicit_pl_and_mn_across_save_and_reopen() {
    let document = open("(;SZ[3]PB[Alice]PW[Bob]KM[6.5]PL[W]MN[93]C[root note];B[aa]C[move note])");
    let result = accepted(apply(
        &mut ReadboardSync::new(),
        &document,
        &frame(&["110", "000", "000"], fox(58)),
    ));
    assert!(result.rebuilt);
    assert_eq!(result.move_number, 58);
    let reopened = open(&result.document.unwrap().serialize().unwrap());
    assert_no_history(&reopened);
    for (key, value) in [
        ("PB", "Alice"),
        ("PW", "Bob"),
        ("KM", "6.5"),
        ("PL", "W"),
        ("MN", "93"),
        ("C", "root note"),
    ] {
        assert_eq!(root_property(&reopened, key).as_deref(), Some(value), "{key}");
    }
    assert!(!reopened.serialize().unwrap().contains("move note"));
    assert_eq!(
        reopened.snapshot(&path(&[])).unwrap().position.to_play,
        PlayerColor::White
    );
}

#[test]
fn a_pl_materialised_by_a_rebuild_is_not_treated_as_explicit() {
    let mut sync = ReadboardSync::new();
    let first = accepted(apply(
        &mut sync,
        &open("(;SZ[3])"),
        &frame(&["110", "000", "000"], generic()),
    ));
    let rebuilt = first.document.unwrap();
    assert_eq!(root_property(&rebuilt, "PL").as_deref(), Some("B"));
    // Same board reported as a Fox zero-move handicap: the metadata change rebuilds with white.
    let second = accepted(apply(&mut sync, &rebuilt, &frame(&["110", "000", "000"], fox(0))));
    assert!(second.rebuilt);
    assert_eq!(
        root_property(second.document.as_ref().unwrap(), "PL").as_deref(),
        Some("W")
    );
}

#[test]
fn fox_history_match_navigates_to_the_existing_ancestor_without_rebuilding() {
    let document = open("(;SZ[3];B[aa];W[ba];B[ca])");
    let result = accepted(apply(
        &mut ReadboardSync::new(),
        &document,
        &frame(&["100", "000", "000"], fox(1)),
    ));
    assert!(result.document.is_none());
    assert_eq!(
        (result.source, result.selected, result.move_number),
        (path(&[0]), path(&[0]), 1)
    );
}

#[test]
fn a_new_room_does_not_reuse_the_previous_recovery_context() {
    let document = open("(;SZ[3];B[aa];W[ba])");
    let earlier = |room| frame(&["100", "000", "000"], fox_room(room, 1));
    let mut sync = ReadboardSync::new();
    assert!(accepted(apply(&mut sync, &document, &earlier("room-1")))
        .document
        .is_none());
    assert!(accepted(apply(&mut sync, &document, &earlier("room-1")))
        .document
        .is_none());
    let switched = accepted(apply(&mut sync, &document, &earlier("room-2")));
    assert!(
        switched.rebuilt,
        "a room change forces a rebuild instead of matching old history"
    );
}

#[test]
fn cursor_follows_only_when_requested() {
    let document = open("(;SZ[3];B[aa];W[ba])");
    let next = frame(&["120", "000", "003"], generic());
    let steady = frame(&["120", "000", "000"], generic());
    let run = |view, cursor: &[u32]| {
        let mut sync = ReadboardSync::new();
        sync.apply(Some(&document), &path(&[0, 0]), &steady, view)
            .unwrap();
        accepted(sync.apply(Some(&document), &path(cursor), &next, view).unwrap()).selected
    };
    let browse = ReadboardViewPreferences {
        always_sync: true,
        jump_to_last: false,
    };
    let jump = ReadboardViewPreferences {
        always_sync: false,
        jump_to_last: true,
    };
    let still = ReadboardViewPreferences {
        always_sync: false,
        jump_to_last: false,
    };
    assert_eq!(
        run(browse, &[0, 0]),
        path(&[0, 0, 0]),
        "always-sync follows from the source end"
    );
    assert_eq!(
        run(browse, &[0]),
        path(&[0]),
        "browsing elsewhere keeps the cursor"
    );
    assert_eq!(run(jump, &[0]), path(&[0, 0, 0]), "jump-to-last always moves");
    assert_eq!(
        run(still, &[0, 0]),
        path(&[0, 0]),
        "no follow without always-sync"
    );
}

#[test]
fn clear_board_and_size_change_start_from_an_empty_board() {
    let document = open("(;SZ[3]PB[Alice];B[aa];W[ba])");
    let mut sync = ReadboardSync::new();
    sync.control(ReadboardControl::ClearBoard);
    let cleared = accepted(apply(
        &mut sync,
        &document,
        &frame(&["300", "000", "000"], generic()),
    ));
    let cleared = cleared.document.unwrap();
    assert_eq!(cleared.serialize().unwrap(), "(;GM[1]FF[4]SZ[3];B[aa])");

    let resized = accepted(apply(
        &mut ReadboardSync::new(),
        &document,
        &frame(&["1100", "0000", "0000", "0000"], generic()),
    ));
    let resized = resized.document.unwrap();
    assert_eq!((resized.board_width(), resized.board_height()), (4, 4));
    assert_no_history(&resized);
}

#[test]
fn yike_frames_are_ignored_and_failures_leave_state_untouched() {
    let document = open("(;SZ[3];B[aa];W[ba])");
    let mut sync = ReadboardSync::new();
    let yike = frame(
        &["000", "000", "112"],
        generic().with_platform(ReadboardPlatform::Yike),
    );
    assert!(matches!(
        apply(&mut sync, &document, &yike),
        ReadboardSyncOutcome::Ignored
    ));
    let conflict = frame(&["000", "000", "112"], generic());
    assert!(matches!(
        apply(&mut sync, &document, &conflict),
        ReadboardSyncOutcome::Hold
    ));
    let mut broken = conflict.clone();
    broken.codes.pop();
    assert!(sync
        .apply(Some(&document), &path(&[0, 0]), &broken, FOLLOW)
        .is_err());
    assert!(
        accepted(apply(&mut sync, &document, &conflict)).rebuilt,
        "the held key survives a failed frame"
    );
}

#[test]
fn a_non_root_pl_setup_node_is_the_rebuild_anchor() {
    let document = open("(;SZ[3]PB[Alice]C[root];B[aa];PL[W]MN[93]C[turn anchor];W[bb])");
    let forced = frame(&["000", "000", "112"], fox(58).with_force_rebuild(true));
    let result = accepted(apply(&mut ReadboardSync::new(), &document, &forced));
    assert!(result.rebuilt);
    let reopened = open(&result.document.unwrap().serialize().unwrap());
    assert_no_history(&reopened);
    for (key, value) in [("PB", "Alice"), ("C", "turn anchor"), ("MN", "93"), ("PL", "W")] {
        assert_eq!(root_property(&reopened, key).as_deref(), Some(value), "{key}");
    }
    assert_eq!(
        reopened.snapshot(&path(&[])).unwrap().position.to_play,
        PlayerColor::White
    );
}

#[test]
fn a_rebuild_never_carries_analysis_of_the_previous_position() {
    let document = open("(;SZ[3]C[keep]LZOP[KataGo 40.0 700]LZOP2[KataGo 40.0 600];B[aa])");
    let forced = frame(&["000", "000", "112"], generic().with_force_rebuild(true));
    let result = accepted(apply(&mut ReadboardSync::new(), &document, &forced));
    assert!(result.rebuilt);
    let reopened = open(&result.document.unwrap().serialize().unwrap());
    assert_eq!(root_property(&reopened, "C").as_deref(), Some("keep"));
    for key in ["LZ", "LZ2", "LZOP", "LZOP2"] {
        assert_eq!(root_property(&reopened, key), None, "{key}");
    }
}

#[test]
fn force_rebuild_skips_the_conflict_hold() {
    let document = open("(;SZ[3];B[aa])");
    let conflict = frame(&["000", "000", "112"], generic());
    assert!(matches!(
        apply(&mut ReadboardSync::new(), &document, &conflict),
        ReadboardSyncOutcome::Hold
    ));
    let forced = frame(&["000", "000", "112"], generic().with_force_rebuild(true));
    let result = accepted(apply(&mut ReadboardSync::new(), &document, &forced));
    assert!(result.rebuilt);
    assert_no_history(result.document.as_ref().unwrap());
}

#[test]
fn a_changed_record_identity_rebuilds_while_the_same_record_keeps_history() {
    let document = open("(;SZ[3];B[aa];W[ba];B[ca])");
    let record = |fingerprint, total| {
        frame(
            &["100", "000", "000"],
            generic()
                .with_platform(ReadboardPlatform::Fox)
                .with_fox_move_number(Some(1))
                .with_title_fingerprint(fingerprint)
                .with_record_total_move(Some(total))
                .with_record_current_move(Some(1)),
        )
    };
    let mut sync = ReadboardSync::new();
    let first = accepted(apply(&mut sync, &document, &record("game A", 3)));
    assert!(!first.rebuilt);
    assert_eq!(first.source, path(&[0]));
    assert!(
        !accepted(apply(&mut sync, &document, &record("game A", 3))).rebuilt,
        "same record source"
    );
    assert!(
        accepted(apply(&mut sync, &document, &record("game A", 4))).rebuilt,
        "total move count changed"
    );
    let mut sync = ReadboardSync::new();
    accepted(apply(&mut sync, &document, &record("game A", 3)));
    assert!(
        accepted(apply(&mut sync, &document, &record("game B", 3))).rebuilt,
        "fingerprint changed"
    );
}

#[test]
fn adjacent_recovery_follows_the_last_resolved_node_when_the_view_is_on_a_variation() {
    let document = open("(;SZ[3];B[aa](;W[ba];B[ca])(;W[cc]))");
    let mut sync = ReadboardSync::new();
    let first = sync
        .apply(
            Some(&document),
            &path(&[0]),
            &frame(&["100", "000", "000"], fox(1)),
            FOLLOW,
        )
        .unwrap();
    assert_eq!(accepted(first).source, path(&[0]));
    // The view moved to the W[cc] variation; the next Fox move is the main-line child of the
    // last resolved node, not a rebuild.
    let next = accepted(
        sync.apply(
            Some(&document),
            &path(&[0, 1]),
            &frame(&["120", "000", "000"], fox(2)),
            FOLLOW,
        )
        .unwrap(),
    );
    assert!(!next.rebuilt);
    assert!(next.document.is_none());
    assert_eq!(
        (next.source, next.selected, next.move_number),
        (path(&[0, 0]), path(&[0, 1]), 2)
    );
}

#[test]
fn a_trusted_marker_supersedes_an_earlier_session_root_pl_without_rebuilding_again() {
    // A previous session's rebuild left a root PL in the document. A later session cannot tell it
    // from an explicit PL; the frame's trusted marker (Black just moved) still decides white to play.
    let earlier = accepted(apply(
        &mut ReadboardSync::new(),
        &open("(;SZ[3];B[aa];W[ba])"),
        &frame(&["110", "000", "000"], fox_room("room-1", 2)),
    ));
    let document = earlier.document.unwrap();
    assert_eq!(root_property(&document, "PL").as_deref(), Some("B"));
    let new_room = frame(
        &["200", "000", "003"],
        fox_room("room-2", 10).with_last_move_source(ReadboardLastMoveSource::FoxCornerFlip),
    );
    let mut sync = ReadboardSync::new();
    let rebuilt = accepted(apply(&mut sync, &document, &new_room)).document.unwrap();
    assert_eq!(root_property(&rebuilt, "PL").as_deref(), Some("W"));
    // The same frame again is steady: no repeated metadata rebuild.
    let again = accepted(apply(&mut sync, &rebuilt, &new_room));
    assert!(!again.rebuilt);
    assert!(again.document.is_none());
}
