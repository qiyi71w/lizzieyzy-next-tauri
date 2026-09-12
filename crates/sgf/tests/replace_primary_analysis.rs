use app_model::{CandidateMoveDto, MoveVertex, NodePath, PointDto};
use sgf::{CurrentSgfDocument, SgfAnalysisPayload};

const BRANCHING: &str = include_str!("../../../tests/golden/java-analysis-branching.sgf");

fn path(indices: &[u32]) -> NodePath {
    NodePath {
        indices: indices.to_vec(),
    }
}

fn replacement_payload(engine_name: &str, visits: u32, x: u8, y: u8) -> SgfAnalysisPayload {
    SgfAnalysisPayload {
        engine_name: engine_name.to_string(),
        visits,
        winrate_black: 0.61,
        score_mean_black: Some(2.25),
        score_stdev: Some(0.5),
        pda: None,
        candidates: vec![CandidateMoveDto {
            vertex: MoveVertex::Point(PointDto { x, y }),
            visits,
            winrate_black: 0.61,
            score_mean_black: 2.25,
            policy_prior: Some(0.4),
            pv: vec![MoveVertex::Point(PointDto { x, y })],
        }],
        ownership: None,
    }
}

#[test]
fn replace_primary_on_root_writes_lzop_and_preserves_secondary_unknown_and_comment() {
    let mut document = CurrentSgfDocument::open(BRANCHING).unwrap();
    let payload = replacement_payload("KataGo", 400, 3, 3);

    let (snapshot, changed) = document.replace_primary_analysis(&path(&[]), &payload).unwrap();

    assert!(changed);
    let primary = snapshot.primary_analysis.expect("root primary");
    assert_eq!(primary.visits, 400);
    assert_eq!(
        primary.candidates[0].vertex,
        MoveVertex::Point(PointDto { x: 3, y: 3 })
    );
    assert_eq!(snapshot.secondary_analysis.unwrap().visits, 150);
    assert_eq!(snapshot.personal_comment, "root personal");

    let serialized = document.serialize().unwrap();
    assert!(serialized.contains("LZOP[KataGo"));
    assert!(serialized.contains("LZOP2[SubEngine 41.0 150"));
    assert!(serialized.contains("C[root personal]"));
    assert!(serialized.contains("XY[keep-me]"));
    assert!(!serialized.contains("AnalysisContext"));
    assert_eq!(serialized.matches("C[root personal]").count(), 1);
}

#[test]
fn save_reopen_preserves_unavailable_root_score() {
    let mut document = CurrentSgfDocument::open(BRANCHING).unwrap();
    let mut payload = replacement_payload("KataGo", 400, 3, 3);
    payload.score_mean_black = None;
    document.replace_primary_analysis(&path(&[]), &payload).unwrap();

    let serialized = document.serialize().unwrap();
    let reopened = CurrentSgfDocument::open(&serialized).unwrap();
    let primary = reopened
        .snapshot(&path(&[]))
        .unwrap()
        .primary_analysis
        .expect("root primary");
    assert_eq!(primary.score_mean_black, None);
    assert_eq!(primary.candidates[0].score_mean_black, 2.25);
}

#[test]
fn replace_primary_on_move_node_writes_lz_and_preserves_lz2_comment_and_unknown() {
    let mut document = CurrentSgfDocument::open(BRANCHING).unwrap();
    let payload = replacement_payload("KataGo", 900, 15, 3);

    let (snapshot, changed) = document.replace_primary_analysis(&path(&[0]), &payload).unwrap();

    assert!(changed);
    let primary = snapshot.primary_analysis.expect("move primary");
    assert_eq!(primary.visits, 900);
    assert_eq!(snapshot.personal_comment, "move comment");
    assert_eq!(snapshot.secondary_analysis.unwrap().visits, 120);

    let serialized = document.serialize().unwrap();
    assert!(serialized.contains("LZ[KataGo"));
    assert!(serialized.contains("LZ2[SubEngine 41.0 120"));
    assert!(serialized.contains("C[move comment]"));
    assert!(serialized.contains("ZZ[unknown-stay]"));
    assert!(!serialized.contains("LZOP[KataGo"));
}

#[test]
fn replace_primary_is_idempotent_for_the_same_canonical_payload() {
    let mut document = CurrentSgfDocument::open(BRANCHING).unwrap();
    let payload = replacement_payload("KataGo", 400, 3, 3);
    let first = document.replace_primary_analysis(&path(&[]), &payload).unwrap();
    let serialized = document.serialize().unwrap();
    let second = document.replace_primary_analysis(&path(&[]), &payload).unwrap();

    assert!(first.1);
    assert!(!second.1);
    assert_eq!(document.serialize().unwrap(), serialized);
    assert_eq!(second.0.personal_comment, "root personal");
    assert_eq!(second.0.secondary_analysis.unwrap().visits, 150);
}

#[test]
fn non_projectable_payload_and_invalid_path_do_not_rewrite_the_tree() {
    let mut document = CurrentSgfDocument::open(BRANCHING).unwrap();
    let before = document.serialize().unwrap();
    let empty = SgfAnalysisPayload {
        engine_name: "KataGo".to_string(),
        visits: 0,
        winrate_black: 0.5,
        score_mean_black: None,
        score_stdev: None,
        pda: None,
        candidates: Vec::new(),
        ownership: None,
    };

    let (snapshot, changed) = document.replace_primary_analysis(&path(&[]), &empty).unwrap();
    assert!(!changed);
    assert_eq!(snapshot.primary_analysis.unwrap().visits, 1500);
    assert_eq!(document.serialize().unwrap(), before);

    let error = document
        .replace_primary_analysis(&path(&[9]), &replacement_payload("KataGo", 400, 3, 3))
        .unwrap_err();
    assert_eq!(error.kind, app_model::CurrentGameErrorKind::InvalidNodePath);
    assert_eq!(document.serialize().unwrap(), before);
}
