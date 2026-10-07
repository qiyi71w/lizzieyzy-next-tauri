use app_model::NodePath;
use sgf::CurrentSgfDocument;

#[test]
fn selected_line_export_preserves_properties_and_source_without_siblings() {
    let source = "(;SZ[9:7]KM[6.5]PB[黑]PW[白]RE[B+R]AB[aa]C[root\\]\\\\text]XX[opaque];PL[W]AW[bc]C[setup](;W[]C[main];B[dd])(;W[cc]TR[cc]C[personal]LZ[opaque persisted analysis](;B[ee])(;B[]LB[cc:标签]C[chosen leaf])))";
    let document = CurrentSgfDocument::open(source).unwrap();
    let before = document.serialize().unwrap();
    let leaf = NodePath { indices: vec![0, 1, 1] };
    let exported = document.serialize_selected_line(&leaf).unwrap();
    let reopened = CurrentSgfDocument::open(&exported).unwrap();
    let mut node = reopened.tree().unwrap();
    assert!(exported.contains("PB[黑]"));
    assert!(exported.contains("LZ[opaque persisted analysis]"));
    assert!(!exported.contains("C[main]"));
    for _ in 0..3 { assert_eq!(node.children.len(), 1); node = node.children.remove(0); }
    assert!(node.children.is_empty());
    assert_eq!(reopened.snapshot(&NodePath { indices: vec![0, 0, 0] }).unwrap().personal_comment, "chosen leaf");
    assert_eq!(document.serialize().unwrap(), before);
    let root = document.serialize_selected_line(&NodePath::default()).unwrap();
    assert!(CurrentSgfDocument::open(&root).unwrap().tree().unwrap().children.is_empty());
    assert!(document.serialize_selected_line(&NodePath { indices: vec![9] }).is_err());
}
