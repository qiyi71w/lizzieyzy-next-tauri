use app_model::{MoveVertex, NodePath, PlayerColor, SgfTreeNodeDto};
use encoding_rs::GB18030;
use sgf::{import_gib, CurrentSgfDocument, GibError};

const NAMED_PASS: &[u8] = include_bytes!("../../../tests/fixtures/tygem-named-pass-lf.gib");
const NATIVE_REPRESENTATIVE: &[u8] =
    include_bytes!("../../../tests/fixtures/tygem-native-representative.gib");

#[test]
fn imports_named_moves_and_pass_into_reopenable_sgf() {
    let imported = import_gib(NAMED_PASS).unwrap();
    let document = CurrentSgfDocument::open(&imported).unwrap();
    let tree = document.tree().unwrap();

    assert_eq!(property(&tree, "PB"), Some("黑方 九段"));
    assert_eq!(property(&tree, "PW"), Some("白方 初段"));
    assert_eq!(property(&tree, "KM"), Some("6.5"));
    assert_eq!(document.default_selected_path().indices, vec![0, 0, 0]);

    let black = document.snapshot(&NodePath { indices: vec![0] }).unwrap();
    assert!(matches!(
        &black.position.last_move.as_ref().unwrap().vertex,
        MoveVertex::Point(point) if point.x == 3 && point.y == 3
    ));
    assert_eq!(
        black.position.last_move.as_ref().unwrap().color,
        PlayerColor::Black
    );

    let white = document.snapshot(&NodePath { indices: vec![0, 0] }).unwrap();
    assert!(matches!(
        &white.position.last_move.as_ref().unwrap().vertex,
        MoveVertex::Point(point) if point.x == 15 && point.y == 15
    ));
    assert_eq!(
        white.position.last_move.as_ref().unwrap().color,
        PlayerColor::White
    );

    let pass = document
        .snapshot(&NodePath {
            indices: vec![0, 0, 0],
        })
        .unwrap();
    assert!(matches!(
        pass.position.last_move.as_ref().unwrap().vertex,
        MoveVertex::Pass
    ));
    assert_eq!(
        pass.position.last_move.as_ref().unwrap().color,
        PlayerColor::Black
    );
}

#[test]
fn accepts_crlf_and_gb18030_while_preserving_defaults_and_names() {
    let defaults = import_gib(b"STO 0 1 1 3 3\r\nSKI 0 2\r\n").unwrap();
    let default_document = CurrentSgfDocument::open(&defaults).unwrap();
    let default_tree = default_document.tree().unwrap();
    assert_eq!(property(&default_tree, "PB"), Some("Player 2"));
    assert_eq!(property(&default_tree, "PW"), Some("Player 1"));
    assert_eq!(property(&default_tree, "KM"), Some("7.5"));

    let (encoded, _, had_errors) = GB18030.encode(std::str::from_utf8(NAMED_PASS).unwrap());
    assert!(!had_errors);
    let imported = import_gib(&encoded).unwrap();
    let document = CurrentSgfDocument::open(&imported).unwrap();
    let serialized = document.serialize().unwrap();
    let reopened = CurrentSgfDocument::open(&serialized).unwrap();
    let tree = reopened.tree().unwrap();
    assert_eq!(property(&tree, "PB"), Some("黑方 九段"));
    assert_eq!(property(&tree, "PW"), Some("白方 初段"));
    assert_eq!(property(&tree, "KM"), Some("6.5"));
    assert_eq!(reopened.default_selected_path().indices, vec![0, 0, 0]);
}

#[test]
fn maps_five_seven_and_nine_stone_handicap_positions() {
    let cases = [
        (5_u8, vec![(3, 3), (3, 15), (9, 9), (15, 3), (15, 15)]),
        (
            7,
            vec![(3, 3), (3, 9), (3, 15), (9, 9), (15, 3), (15, 9), (15, 15)],
        ),
        (
            9,
            vec![
                (3, 3),
                (3, 9),
                (3, 15),
                (9, 3),
                (9, 9),
                (9, 15),
                (15, 3),
                (15, 9),
                (15, 15),
            ],
        ),
    ];

    for (handicap, expected) in cases {
        let imported = import_gib(format!("INI 0 0 {handicap}\n").as_bytes()).unwrap();
        let document = CurrentSgfDocument::open(&imported).unwrap();
        let tree = document.tree().unwrap();
        let snapshot = document.snapshot(&NodePath::default()).unwrap();
        let mut actual: Vec<(u8, u8)> = snapshot
            .position
            .stones
            .iter()
            .map(|stone| {
                assert_eq!(stone.color, PlayerColor::Black);
                (stone.x, stone.y)
            })
            .collect();
        actual.sort_unstable();

        assert_eq!(property(&tree, "HA"), Some(handicap.to_string().as_str()));
        assert_eq!(property(&tree, "PL"), Some("W"));
        assert_eq!(snapshot.position.to_play, PlayerColor::White);
        assert_eq!(actual, expected);
        assert!(document.default_selected_path().indices.is_empty());
    }
}

#[test]
fn representative_handicap_game_roundtrips_through_current_sgf_document() {
    let imported = import_gib(NATIVE_REPRESENTATIVE).unwrap();
    let document = CurrentSgfDocument::open(&imported).unwrap();
    let serialized = document.serialize().unwrap();
    let reopened = CurrentSgfDocument::open(&serialized).unwrap();
    let tree = reopened.tree().unwrap();
    let root = reopened.snapshot(&NodePath::default()).unwrap();
    let final_position = reopened
        .snapshot(&NodePath {
            indices: vec![0, 0, 0],
        })
        .unwrap();

    assert_eq!(property(&tree, "PB"), Some("黑方 九段"));
    assert_eq!(property(&tree, "PW"), Some("白方 初段"));
    assert_eq!(property(&tree, "KM"), Some("6.5"));
    assert_eq!(property(&tree, "HA"), Some("5"));
    assert_eq!(root.position.stones.len(), 5);
    assert_eq!(root.position.to_play, PlayerColor::White);
    assert_eq!(final_position.position.move_number, 3);
    assert_eq!(
        final_position.position.last_move.as_ref().unwrap().color,
        PlayerColor::White
    );
    assert!(matches!(
        final_position.position.last_move.as_ref().unwrap().vertex,
        MoveVertex::Pass
    ));
    assert_eq!(reopened.default_selected_path().indices, vec![0, 0, 0]);
}

#[test]
fn rejects_empty_malformed_and_invalid_coordinate_input() {
    assert_eq!(import_gib(b"").unwrap_err(), GibError::Empty);
    assert_eq!(import_gib(b" \r\n").unwrap_err(), GibError::Empty);
    assert_eq!(
        import_gib(b"not a GIB record\n").unwrap_err(),
        GibError::Malformed
    );
    assert_eq!(
        import_gib(br"\[GAMEINFOMAIN=GONGJE:65\]").unwrap_err(),
        GibError::Malformed
    );
    assert_eq!(import_gib(b"STO 0 1 1 19 0\n").unwrap_err(), GibError::Malformed);
    assert_eq!(
        import_gib(b"STO 0 1 1 3 3\nSTO 0 2 2 3 3\n").unwrap_err(),
        GibError::Malformed
    );
    assert_eq!(import_gib(&[0xff]).unwrap_err(), GibError::InvalidEncoding);
}

fn property<'a>(node: &'a SgfTreeNodeDto, key: &str) -> Option<&'a str> {
    node.properties
        .iter()
        .find(|property| property.key == key)
        .and_then(|property| property.values.first())
        .map(String::as_str)
}
