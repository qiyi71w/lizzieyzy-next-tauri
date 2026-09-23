use app_model::{NodePath, PlayerColor, StoneDto};
use sgf::{CurrentSgfDocument, DocumentHistory};

fn path(indices: &[u32]) -> NodePath {
    NodePath {
        indices: indices.to_vec(),
    }
}

#[test]
fn childless_root_setup_is_one_reversible_edit_and_preserves_unrelated_properties() {
    let mut document =
        CurrentSgfDocument::open("(;FF[4]SZ[2:3]C[personal]XY[keep]AB[aa]AW[bb]AE[ba]PL[W]LZOP[stale])")
            .unwrap();
    let original = document.serialize().unwrap();
    let mut history = DocumentHistory::default();
    let root = path(&[]);
    let draft = vec![
        StoneDto {
            x: 1,
            y: 0,
            color: PlayerColor::White,
        },
        StoneDto {
            x: 0,
            y: 2,
            color: PlayerColor::Black,
        },
    ];
    let edit = document
        .apply_root_setup_with_history(&root, &draft, PlayerColor::Black)
        .unwrap();
    assert!(edit.edit.is_some());
    history.commit(edit.edit.unwrap());
    let snapshot = document.snapshot(&root).unwrap();
    assert_eq!(snapshot.position.stones, draft);
    assert_eq!(snapshot.position.to_play, PlayerColor::Black);
    assert_eq!(snapshot.position.move_number, 0);
    assert!(snapshot.primary_analysis.is_none());
    let saved = document.serialize().unwrap();
    assert!(saved.contains("XY[keep]"));
    assert!(saved.contains("C[personal]"));
    assert!(!saved.contains("LZOP[stale]"));
    assert!(document
        .apply_root_setup_with_history(&root, &draft, PlayerColor::Black)
        .unwrap()
        .edit
        .is_none());
    history.undo(&mut document).unwrap().unwrap();
    assert_eq!(document.serialize().unwrap(), original);
    history.redo(&mut document).unwrap().unwrap();
    assert_eq!(document.serialize().unwrap(), saved);
    assert_eq!(
        CurrentSgfDocument::open(&saved)
            .unwrap()
            .snapshot(&root)
            .unwrap()
            .position
            .stones,
        draft
    );
}

#[test]
fn setup_rejects_child_roots_and_out_of_bounds_without_mutation() {
    let mut document = CurrentSgfDocument::open("(;SZ[2:3]XY[keep];B[aa])").unwrap();
    let original = document.serialize().unwrap();
    let root = path(&[]);
    assert!(document
        .apply_root_setup_with_history(&root, &[], PlayerColor::White)
        .is_err());
    assert!(document
        .apply_root_setup_with_history(&path(&[0]), &[], PlayerColor::White)
        .is_err());
    assert!(document
        .apply_root_setup_with_history(
            &root,
            &[StoneDto {
                x: 2,
                y: 0,
                color: PlayerColor::Black
            }],
            PlayerColor::White
        )
        .is_err());
    assert_eq!(document.serialize().unwrap(), original);
}

#[test]
fn conversion_replays_selected_branch_and_restores_entire_tree_and_cursor() {
    let mut document = CurrentSgfDocument::open("(;FF[4]SZ[3:4]PB[black]C[root]XY[keep]AB[aa]PL[W]LZOP[stale];W[bb](;B[cc];W[];AB[ca]AE[aa]PL[B]C[branch])(;B[bc]ZZ[other]))").unwrap();
    let selected = path(&[0, 0, 0, 0]);
    let before = document.snapshot(&selected).unwrap().position;
    let original = document.serialize().unwrap();
    let mut history = DocumentHistory::default();
    let edit = document.convert_to_root_setup_with_history(&selected).unwrap();
    history.commit(edit.edit.unwrap());
    let converted = document.snapshot(&path(&[])).unwrap();
    assert_eq!(converted.position.stones, before.stones);
    assert_eq!(converted.position.to_play, before.to_play);
    assert_eq!(converted.position.move_number, 0);
    assert!(converted.primary_analysis.is_none());
    let tree = document.tree().unwrap();
    assert!(tree.children.is_empty());
    let result = document.serialize().unwrap();
    assert!(result.contains("PB[black]"));
    assert!(result.contains("C[root]"));
    assert!(result.contains("XY[keep]"));
    assert!(!result.contains("LZOP[stale]"));
    let undone = history.undo(&mut document).unwrap().unwrap();
    assert_eq!(undone.selected_path, selected);
    assert_eq!(document.serialize().unwrap(), original);
    let redone = history.redo(&mut document).unwrap().unwrap();
    assert_eq!(redone.selected_path, path(&[]));
    assert_eq!(document.serialize().unwrap(), result);
}

#[test]
fn conversion_keeps_root_analysis_when_position_is_unchanged() {
    let mut document = CurrentSgfDocument::open("(;SZ[3]AB[aa]PL[B]LZOP[still-applicable];B[];W[])").unwrap();
    let mut history = DocumentHistory::default();
    let original = document.serialize().unwrap();
    let root = path(&[]);
    let selected = path(&[0, 0]);
    assert_eq!(
        document.snapshot(&root).unwrap().position.stones,
        document.snapshot(&selected).unwrap().position.stones
    );
    assert_eq!(
        document.snapshot(&root).unwrap().position.to_play,
        document.snapshot(&selected).unwrap().position.to_play
    );
    history.commit(
        document
            .convert_to_root_setup_with_history(&selected)
            .unwrap()
            .edit
            .unwrap(),
    );
    let converted = document.serialize().unwrap();
    assert!(converted.contains("LZOP[still-applicable]"));
    assert!(document.tree().unwrap().children.is_empty());
    history.undo(&mut document).unwrap().unwrap();
    assert_eq!(document.serialize().unwrap(), original);
    history.redo(&mut document).unwrap().unwrap();
    assert_eq!(document.serialize().unwrap(), converted);
}
