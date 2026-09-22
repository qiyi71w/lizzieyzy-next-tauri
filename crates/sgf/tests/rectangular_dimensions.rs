use app_model::{MoveDto, MoveVertex, NodePath, PlayerColor, PointDto};
use sgf::{
    encode_analysis_payload, parse_analysis_payload, parse_sgf, replay_sgf_positions, serialize_sgf_document,
    AnalysisSlot, CurrentSgfDocument, SgfAnalysisPayload, SgfDocument,
};

#[test]
fn parses_missing_square_and_rectangular_dimensions() {
    let defaulted = parse_sgf("(;GM[1]FF[4])").unwrap();
    assert_eq!((defaulted.board_width, defaulted.board_height), (19, 19));

    let square = parse_sgf("(;SZ[9])").unwrap();
    assert_eq!((square.board_width, square.board_height), (9, 9));

    let rectangular = parse_sgf("(;SZ[25:2];B[yb])").unwrap();
    assert_eq!((rectangular.board_width, rectangular.board_height), (25, 2));
    assert_eq!(
        rectangular.moves[0].vertex,
        MoveVertex::Point(PointDto { x: 24, y: 1 })
    );
}

#[test]
fn rejects_malformed_out_of_range_and_far_axis_coordinates() {
    for size in [
        "", "1", "26", "19x13", "19:", ":13", "19:1", "26:13", "19:26", "19:13:9",
    ] {
        assert!(
            parse_sgf(&format!("(;SZ[{size}])")).is_err(),
            "accepted SZ[{size}]"
        );
    }

    assert!(parse_sgf("(;SZ[25:2];B[yc])").is_err());
    assert!(parse_sgf("(;SZ[2:25];B[by])").is_ok());
    assert!(parse_sgf("(;SZ[2:25];B[cy])").is_err());
}

#[test]
fn current_document_rejects_invalid_properties_anywhere_in_the_tree() {
    for sgf in [
        "(;SZ[2:3]AB[ca])",
        "(;SZ[2:3](;B[aa])(;B[ca]))",
        "(;SZ[2:3](;B[aa])(;PL[X]))",
        "(;SZ[2:3](;B[aa])(;AB[aa:ca]))",
        "(;SZ[2:3];B[aa][ca])",
        "(;SZ[2:3];PL[W][X])",
    ] {
        assert!(CurrentSgfDocument::open(sgf).is_err(), "accepted {sgf}");
    }
}

#[test]
fn current_document_rejects_ambiguous_root_dimensions() {
    assert!(CurrentSgfDocument::open("(;SZ[9][13])").is_err());
    assert!(CurrentSgfDocument::open("(;SZ[9]SZ[13])").is_err());
    let defaulted = CurrentSgfDocument::open("(;GM[1])").unwrap();
    assert_eq!((defaulted.board_width(), defaulted.board_height()), (19, 19));
}

#[test]
fn current_document_accepts_rectangular_compressed_setup_and_variations() {
    let document = CurrentSgfDocument::open("(;SZ[2:3]AB[aa:bb]PL[W](;W[ac])(;W[bc]))").unwrap();
    let snapshot = document.snapshot(&NodePath { indices: vec![1] }).unwrap();

    assert_eq!(
        (snapshot.position.board_width, snapshot.position.board_height),
        (2, 3)
    );
    assert_eq!(snapshot.position.to_play, PlayerColor::Black);
    assert_eq!(snapshot.position.stones.len(), 5);
    assert!(snapshot
        .position
        .stones
        .iter()
        .any(|stone| stone.x == 1 && stone.y == 2 && stone.color == PlayerColor::White));
}

#[test]
fn exact_node_markup_uses_both_axes_and_survives_reopen() {
    use app_model::SgfMarkupDto;

    let document = CurrentSgfDocument::open("(;SZ[2:25]CR[ax:by](;B[aa]LB[ay:A])(;B[ba]LB[by:B]))").unwrap();
    let root = document.snapshot(&NodePath::default()).unwrap();
    let mut circles: Vec<_> = root
        .markup
        .iter()
        .map(|mark| match mark {
            SgfMarkupDto::Circle { point } => (point.x, point.y),
            _ => panic!("unexpected root markup"),
        })
        .collect();
    circles.sort();
    assert_eq!(circles, vec![(0, 23), (0, 24), (1, 23), (1, 24)]);

    let branch = NodePath { indices: vec![1] };
    let selected = document.snapshot(&branch).unwrap();
    assert_eq!(
        selected.markup,
        vec![SgfMarkupDto::Label {
            point: PointDto { x: 1, y: 24 },
            text: "B".to_string(),
        }]
    );
    let reopened = CurrentSgfDocument::open(&document.serialize().unwrap()).unwrap();
    assert_eq!(reopened.snapshot(&branch).unwrap().markup, selected.markup);
}

#[test]
fn replays_compressed_setup_and_far_axis_moves_on_rectangles() {
    let positions = replay_sgf_positions("(;SZ[25:2]AB[xa:yb];W[wa])").unwrap();
    let root = &positions[0];
    assert_eq!((root.board_width, root.board_height), (25, 2));
    assert_eq!(root.stones.len(), 4);
    assert!(root.stones.iter().any(|stone| stone.x == 24 && stone.y == 1));

    let moved = &positions[1];
    assert!(moved.errors.is_empty());
    assert!(moved.stones.iter().any(|stone| stone.x == 22 && stone.y == 0));

    let tall = replay_sgf_positions("(;SZ[2:25];B[by])").unwrap();
    assert!(tall[1].errors.is_empty());
    assert!(tall[1].stones.iter().any(|stone| stone.x == 1 && stone.y == 24));
}

#[test]
fn serializes_canonical_square_and_rectangular_sz() {
    let rectangular = SgfDocument {
        board_width: 25,
        board_height: 2,
        komi: 7.5,
        handicap: None,
        black_name: None,
        white_name: None,
        result: None,
        moves: vec![MoveDto {
            color: PlayerColor::Black,
            vertex: MoveVertex::Point(PointDto { x: 24, y: 1 }),
            move_number: 1,
        }],
        root: None,
    };
    assert_eq!(
        serialize_sgf_document(&rectangular).unwrap(),
        "(;FF[4]GM[1]SZ[25:2]KM[7.5];B[yb])"
    );

    let square = SgfDocument {
        board_width: 9,
        board_height: 9,
        moves: Vec::new(),
        ..rectangular
    };
    assert!(serialize_sgf_document(&square).unwrap().contains("SZ[9]"));

    let parsed_without_size = parse_sgf("(;GM[1]FF[4]C[keep])").unwrap();
    let canonical = serialize_sgf_document(&parsed_without_size).unwrap();
    assert!(canonical.contains("SZ[19]"));
    assert!(canonical.contains("C[keep]"));
}

#[test]
fn current_document_plays_and_serializes_far_axis_moves() {
    let mut document = CurrentSgfDocument::open("(;SZ[25:2])").unwrap();
    assert_eq!((document.board_width(), document.board_height()), (25, 2));

    let snapshot = document
        .play(&NodePath::default(), MoveVertex::Point(PointDto { x: 24, y: 1 }))
        .unwrap();
    assert_eq!(
        (snapshot.position.board_width, snapshot.position.board_height),
        (25, 2)
    );
    assert!(document.serialize().unwrap().contains(";B[yb]"));
}

#[test]
fn analysis_gtp_coordinates_use_width_and_height() {
    let parsed = parse_analysis_payload(
        "KataGo 50.0 10\nmove Z2 visits 10 winrate 5000 prior 1000 pv Z2",
        25,
        2,
    )
    .unwrap();
    assert_eq!(
        parsed.candidates[0].vertex,
        MoveVertex::Point(PointDto { x: 24, y: 0 })
    );

    let encoded = encode_analysis_payload(
        &SgfAnalysisPayload {
            engine_name: "KataGo".to_string(),
            visits: 10,
            winrate_black: 0.5,
            score_mean_black: None,
            score_stdev: None,
            pda: None,
            candidates: parsed.candidates,
            ownership: Some(vec![0.0; 50]),
        },
        AnalysisSlot::Primary { root: true },
        25,
        2,
    );
    assert!(encoded.values[0].contains("move Z2"));
}
