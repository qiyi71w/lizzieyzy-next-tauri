use app_model::{MoveVertex, NodePath, PointDto};
use sgf::{encode_analysis_payload, parse_analysis_payload, AnalysisSlot, CurrentSgfDocument};

const ROOT_LZOP: &str = include_str!("../../../tests/golden/java-analysis-root-lzop.sgf");
const BRANCHING: &str = include_str!("../../../tests/golden/java-analysis-branching.sgf");

fn assert_close(actual: f32, expected: f32) {
    assert!((actual - expected).abs() < 1e-4, "{actual} != {expected}");
}

fn point(x: u8, y: u8) -> MoveVertex {
    MoveVertex::Point(PointDto { x, y })
}

fn payload_value<'a>(sgf: &'a str, key: &str) -> &'a str {
    let marker = format!("{key}[");
    let start = sgf
        .find(&marker)
        .unwrap_or_else(|| panic!("{key} property missing"));
    let body_start = start + marker.len();
    let body = &sgf[body_start..];
    let mut escaped = false;
    for (index, ch) in body.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == ']' {
            return &body[..index];
        }
    }
    panic!("{key} property was not closed");
}

#[test]
fn frozen_java_root_lzop_parses_engine_visits_winrate_candidates_and_pv() {
    let payload =
        parse_analysis_payload(payload_value(ROOT_LZOP, "LZOP"), 19).expect("root LZOP should parse");

    assert_eq!(payload.engine_name, "MainEngine");
    assert_eq!(payload.visits, 120);
    assert_close(payload.winrate_black, 0.56);
    assert!(payload.score_mean_black.is_none());
    assert_eq!(payload.candidates.len(), 1);
    let candidate = &payload.candidates[0];
    assert_eq!(candidate.vertex, point(3, 15));
    assert_eq!(candidate.visits, 120);
    assert_close(candidate.winrate_black, 0.56);
    assert_eq!(candidate.policy_prior, Some(0.5));
    assert_eq!(candidate.pv, vec![point(3, 15), point(2, 15)]);
}

#[test]
fn frozen_java_root_lzop2_parses_secondary_payload() {
    let payload =
        parse_analysis_payload(payload_value(ROOT_LZOP, "LZOP2"), 19).expect("root LZOP2 should parse");

    assert_eq!(payload.engine_name, "SubEngine");
    assert_eq!(payload.visits, 150);
    assert_close(payload.winrate_black, 0.59);
    assert_eq!(payload.candidates[0].vertex, point(15, 3));
}

#[test]
fn frozen_java_payload_parses_score_ownership_and_playout_shorthand() {
    let payload =
        parse_analysis_payload(payload_value(BRANCHING, "LZOP"), 19).expect("kata LZOP should parse");

    assert_eq!(payload.visits, 1500);
    assert_close(payload.score_mean_black.unwrap(), 3.5);
    assert_close(payload.score_stdev.unwrap(), 0.7);
    assert_close(payload.pda.unwrap(), 0.9);
    assert_eq!(payload.ownership.as_deref(), Some(&[0.1, -0.2, 0.3][..]));
    assert_close(payload.candidates[0].score_mean_black, 3.5);
    assert_eq!(payload.candidates[0].policy_prior, Some(0.1828));
}

#[test]
fn malformed_and_zero_visit_payloads_are_not_projectable_analysis() {
    assert!(parse_analysis_payload("not-analysis", 19).is_none());
    assert!(parse_analysis_payload("Engine", 19).is_none());
    assert!(parse_analysis_payload("", 19).is_none());

    let header_only = parse_analysis_payload("Main 44.0 120", 19).expect("header parses");
    assert!(header_only.is_projectable());

    let zero_visits = parse_analysis_payload("Main 44.0 0 3.5 0.7", 19).expect("zero visits parses");
    assert!(!zero_visits.is_projectable());
}

#[test]
fn canonical_encode_uses_lzop_at_root_and_lz_elsewhere() {
    let parsed = parse_analysis_payload(payload_value(ROOT_LZOP, "LZOP"), 19).unwrap();

    let root = encode_analysis_payload(&parsed, AnalysisSlot::Primary { root: true }, 19);
    let node = encode_analysis_payload(&parsed, AnalysisSlot::Primary { root: false }, 19);
    let secondary = encode_analysis_payload(&parsed, AnalysisSlot::Secondary { root: true }, 19);

    assert_eq!(root.key, "LZOP");
    assert_eq!(node.key, "LZ");
    assert_eq!(secondary.key, "LZOP2");
    assert_eq!(root.values.len(), 1);

    let reparsed = parse_analysis_payload(&root.values[0], 19).unwrap();
    assert_eq!(reparsed.engine_name, parsed.engine_name);
    assert_eq!(reparsed.visits, parsed.visits);
    assert_close(reparsed.winrate_black, parsed.winrate_black);
    assert_eq!(reparsed.candidates[0].vertex, parsed.candidates[0].vertex);
    assert_eq!(reparsed.candidates[0].pv, parsed.candidates[0].pv);
    assert_eq!(
        reparsed.candidates[0].policy_prior,
        parsed.candidates[0].policy_prior
    );
    assert_eq!(reparsed.to_frame(0).score_mean_black, None);
}

#[test]
fn encode_does_not_emit_next_companion_context_or_comment_stats() {
    let parsed = parse_analysis_payload(payload_value(ROOT_LZOP, "LZOP"), 19).unwrap();
    let encoded = encode_analysis_payload(&parsed, AnalysisSlot::Primary { root: true }, 19);
    let body = &encoded.values[0];
    assert!(!body.contains("AnalysisContext"));
    assert!(!body.contains("visits:"));
    assert_eq!(encoded.key, "LZOP");
}

fn path(indices: &[u32]) -> NodePath {
    NodePath {
        indices: indices.to_vec(),
    }
}

#[test]
fn current_game_projects_java_primary_and_secondary_on_exact_nodes() {
    let document = CurrentSgfDocument::open(BRANCHING).unwrap();
    let root = document.snapshot(&path(&[])).unwrap();
    let primary = root.primary_analysis.expect("root primary");
    let secondary = root.secondary_analysis.expect("root secondary");
    assert_eq!(primary.visits, 1500);
    assert_close(primary.winrate_black, 0.56);
    assert_close(primary.score_mean_black.unwrap(), 3.5);
    assert_eq!(primary.ownership.as_deref(), Some(&[0.1, -0.2, 0.3][..]));
    assert_eq!(primary.candidates[0].vertex, point(3, 3));
    assert_eq!(secondary.visits, 150);
    assert_eq!(secondary.candidates[0].vertex, point(15, 3));
    assert!(primary.policy.is_none());

    let move_node = document.snapshot(&path(&[0])).unwrap();
    let move_primary = move_node.primary_analysis.expect("move primary");
    assert_eq!(move_primary.visits, 800);
    assert_eq!(move_node.personal_comment, "move comment");
    assert_eq!(move_primary.candidates[0].pv.last(), Some(&MoveVertex::Pass));

    let pass_node = document.snapshot(&path(&[0, 0, 0])).unwrap();
    let pass_primary = pass_node.primary_analysis.expect("pass primary");
    assert_eq!(pass_primary.candidates[0].vertex, MoveVertex::Pass);
    assert_eq!(pass_node.personal_comment, "mainline pass");

    let malformed = document.snapshot(&path(&[0, 0, 1])).unwrap();
    assert!(malformed.primary_analysis.is_none());
    assert!(malformed.secondary_analysis.is_none());
    assert_eq!(malformed.personal_comment, "branch leaf");
}

#[test]
fn save_reopen_preserves_secondary_unknown_comment_and_malformed_payloads() {
    let opened = CurrentSgfDocument::open(BRANCHING).unwrap();
    let serialized = opened.serialize().unwrap();

    assert!(serialized.contains("C[root personal]"));
    assert!(serialized.contains("XY[keep-me]"));
    assert!(serialized.contains("ZZ[unknown-stay]"));
    assert!(serialized.contains("C[move comment]"));
    assert!(serialized.contains("C[branch leaf]"));
    assert!(serialized.contains("LZ[not-analysis]"));
    assert!(serialized.contains("LZ2[incomplete]"));
    assert!(serialized.contains("LZOP2[SubEngine 41.0 150"));
    assert!(!serialized.contains("AnalysisContext"));
    assert_eq!(serialized.matches("C[root personal]").count(), 1);

    let reopened = CurrentSgfDocument::open(&serialized).unwrap();
    let original_root = opened.snapshot(&path(&[])).unwrap();
    let reopened_root = reopened.snapshot(&path(&[])).unwrap();
    let original = original_root.primary_analysis.unwrap();
    let restored = reopened_root.primary_analysis.unwrap();
    assert_eq!(restored.visits, original.visits);
    assert_close(restored.winrate_black, original.winrate_black);
    assert_eq!(restored.score_mean_black, original.score_mean_black);
    assert_eq!(restored.candidates[0].vertex, original.candidates[0].vertex);
    assert_eq!(reopened_root.secondary_analysis.unwrap().visits, 150);
    assert_eq!(reopened_root.personal_comment, "root personal");
    assert!(reopened
        .snapshot(&path(&[0, 0, 1]))
        .unwrap()
        .primary_analysis
        .is_none());
}

#[test]
fn encoded_root_and_node_payloads_reopen_as_equivalent_primary_analysis() {
    let parsed = parse_analysis_payload(payload_value(ROOT_LZOP, "LZOP"), 19).unwrap();
    let root_prop = encode_analysis_payload(&parsed, AnalysisSlot::Primary { root: true }, 19);
    let node_prop = encode_analysis_payload(&parsed, AnalysisSlot::Primary { root: false }, 19);
    let root_sgf = format!(
        "(;FF[4]GM[1]SZ[19]KM[7.5]{}[{}])",
        root_prop.key, root_prop.values[0]
    );
    let node_sgf = format!(
        "(;FF[4]GM[1]SZ[19]KM[7.5];B[dd]{}[{}])",
        node_prop.key, node_prop.values[0]
    );

    let root = CurrentSgfDocument::open(&root_sgf)
        .unwrap()
        .snapshot(&path(&[]))
        .unwrap()
        .primary_analysis
        .unwrap();
    let node = CurrentSgfDocument::open(&node_sgf)
        .unwrap()
        .snapshot(&path(&[0]))
        .unwrap()
        .primary_analysis
        .unwrap();
    assert_eq!(root.visits, parsed.visits);
    assert_eq!(node.visits, parsed.visits);
    assert_eq!(root.candidates[0].vertex, parsed.candidates[0].vertex);
    assert_eq!(node.candidates[0].vertex, parsed.candidates[0].vertex);
    assert!(root_sgf.contains("LZOP["));
    assert!(node_sgf.contains("LZ["));
    assert!(!node_sgf.contains("LZOP["));
}
