use app_model::{MoveVertex, NodePath, PlayerColor, PointDto, PositionDto, SgfTreeNodeDto};
use sgf::CurrentSgfDocument;

const BRANCHING: &str = include_str!("../../../tests/golden/editable-workspace-branching.sgf");

#[test]
fn exact_navigation_preserves_distinct_same_position_nodes_and_markup() {
    let document =
        CurrentSgfDocument::open(include_str!("../../../tests/golden/r6-exact-navigation.sgf")).unwrap();
    let before = document.serialize().unwrap();
    let a = document
        .snapshot(&NodePath {
            indices: vec![0, 0, 0, 0, 0],
        })
        .unwrap();
    let b = document
        .snapshot(&NodePath {
            indices: vec![0, 0, 0, 1, 0],
        })
        .unwrap();
    assert_eq!(a.position, b.position);
    assert_eq!(a.position.move_number, 2);
    assert_ne!(a.path, b.path);
    assert_eq!(a.personal_comment, "branch A move");
    assert_eq!(b.personal_comment, "branch B same move");
    assert_eq!(
        a.markup,
        vec![app_model::SgfMarkupDto::Label {
            point: PointDto { x: 3, y: 3 },
            text: "A".into()
        }]
    );
    assert_eq!(
        b.markup,
        vec![app_model::SgfMarkupDto::Label {
            point: PointDto { x: 3, y: 3 },
            text: "B".into()
        }]
    );
    let pass = document
        .snapshot(&NodePath {
            indices: vec![0, 0, 0],
        })
        .unwrap();
    assert_eq!(pass.position.move_number, 1);
    assert!(matches!(
        pass.position.last_move.unwrap().vertex,
        MoveVertex::Pass
    ));
    let comment = document.snapshot(&NodePath { indices: vec![0, 0] }).unwrap();
    assert_eq!(comment.position.move_number, 0);
    assert!(comment.markup.is_empty());
    assert_eq!(document.serialize().unwrap(), before);
    let reopened = CurrentSgfDocument::open(&before).unwrap();
    assert_eq!(reopened.tree().unwrap(), document.tree().unwrap());
    assert_eq!(reopened.snapshot(&b.path).unwrap(), b);
}

#[test]
fn editable_workspace_roundtrip_retains_edit_and_drops_removed_sibling() {
    let mut document = CurrentSgfDocument::open(BRANCHING).unwrap();
    let parent = NodePath { indices: vec![0] };
    let first_sibling = NodePath { indices: vec![0, 0] };
    let second_sibling = NodePath { indices: vec![0, 1] };

    let added = document
        .play(&parent, MoveVertex::Point(PointDto { x: 1, y: 1 }))
        .unwrap();
    assert_eq!(added.path.indices, vec![0, 2]);
    let added_path = added.path;
    let commented = document
        .set_personal_comment(&added_path, "retained note")
        .unwrap();
    assert_eq!(commented.personal_comment, "retained note");
    assert!(matches!(
        commented.position.last_move.as_ref().unwrap().vertex,
        MoveVertex::Point(PointDto { x: 1, y: 1 })
    ));

    let after_remove = document.remove_variation(&second_sibling).unwrap();
    assert_eq!(after_remove.indices, vec![0]);
    let retained_path = NodePath { indices: vec![0, 1] };

    let serialized = document.serialize().unwrap();
    let reopened = CurrentSgfDocument::open(&serialized).unwrap();
    let tree = reopened.tree().unwrap();
    let root_snapshot = reopened.snapshot(&NodePath { indices: Vec::new() }).unwrap();
    let first = reopened.snapshot(&first_sibling).unwrap();
    let retained = reopened.snapshot(&retained_path).unwrap();
    let first_leaf = reopened
        .snapshot(&NodePath {
            indices: vec![0, 0, 0],
        })
        .unwrap();

    assert_eq!(tree.children[0].children.len(), 2);
    assert_eq!(comment(&tree.children[0].children[0]), Some("first continuation"));
    assert_eq!(comment(&tree.children[0].children[1]), Some("retained note"));
    assert!(tree.children[0]
        .children
        .iter()
        .all(|child| comment(child) != Some("second continuation")));
    assert!(!serialized.contains("second continuation"));
    assert!(has_unknown_property(&tree));
    assert_eq!(property(&tree, "SZ"), Some("5"));
    assert_eq!(property(&tree, "KM"), Some("0.5"));
    assert_eq!(property(&tree, "HA"), Some("2"));
    assert_eq!(property(&tree, "PB"), Some("Black"));
    assert_eq!(property(&tree, "PW"), Some("White"));
    assert_eq!(property(&tree, "RE"), Some("B+R"));
    assert_eq!(property(&tree, "DT"), Some("2026-08-29"));
    assert_eq!(property(&tree, "PL"), Some("W"));
    assert_eq!(root_snapshot.personal_comment, "root personal");
    assert_eq!(root_snapshot.position.board_size, 5);
    assert_eq!(root_snapshot.position.to_play, PlayerColor::White);
    assert!(has_stone(&root_snapshot.position, 0, 0, PlayerColor::Black));
    assert!(has_stone(&root_snapshot.position, 2, 2, PlayerColor::White));
    assert!(!has_stone(&root_snapshot.position, 1, 1, PlayerColor::Black));
    assert_eq!(first.personal_comment, "first continuation");
    assert!(has_stone(&first.position, 4, 4, PlayerColor::Black));
    assert!(!has_stone(&first.position, 1, 1, PlayerColor::Black));
    assert!(matches!(
        first_leaf.position.last_move.as_ref().unwrap().vertex,
        MoveVertex::Pass
    ));
    assert_eq!(retained.personal_comment, "retained note");
    assert_eq!(retained.position.move_number, 2);
    assert_eq!(retained.position.to_play, PlayerColor::White);
    assert!(has_stone(&retained.position, 1, 1, PlayerColor::Black));
    assert!(!has_stone(&retained.position, 0, 3, PlayerColor::Black));
    assert!(reopened.snapshot(&NodePath { indices: vec![0, 2] }).is_err());
}

fn comment(node: &SgfTreeNodeDto) -> Option<&str> {
    node.properties
        .iter()
        .find(|property| property.key == "C")
        .and_then(|property| property.values.first())
        .map(String::as_str)
}

fn property<'a>(node: &'a SgfTreeNodeDto, key: &str) -> Option<&'a str> {
    node.properties
        .iter()
        .find(|property| property.key == key)
        .and_then(|property| property.values.first())
        .map(String::as_str)
}

fn has_stone(position: &PositionDto, x: u8, y: u8, color: PlayerColor) -> bool {
    position
        .stones
        .iter()
        .any(|stone| stone.x == x && stone.y == y && stone.color == color)
}

fn has_unknown_property(node: &SgfTreeNodeDto) -> bool {
    node.properties
        .iter()
        .any(|property| property.key == "XY" && property.values == ["keep-me"])
}
