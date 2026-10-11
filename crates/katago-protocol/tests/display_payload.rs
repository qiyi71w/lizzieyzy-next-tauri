use app_model::AnalysisJobId;
use katago_protocol::{normalize_response, AnalysisResponse};

// Captured from KataGo 1.18.2 Eigen, b20c256 model, 9x9 Chinese, 7.5 komi,
// maxVisits=8, one search thread. Query requested includePVVisits and includeMovesOwnership.
const REAL: &str = include_str!("fixtures/katago-1.18.2-analysis-display.jsonl");
#[test]
fn real_display_fields_remain_paired_to_their_candidate() {
    let response: AnalysisResponse = serde_json::from_str(REAL).unwrap();
    assert!(response.has_valid_search_result(9, 9));
    let frame = normalize_response(AnalysisJobId::nil(), response, 9, 9);
    assert_eq!(frame.candidates[0].pv_visits.as_deref(), Some([7, 2, 1].as_slice()));
    assert_eq!(frame.candidates[0].pv.len(), 3);
    assert_eq!(frame.candidates[0].ownership.as_ref().unwrap().len(), 81);
    assert_ne!(frame.candidates[0].ownership, frame.ownership);
}
#[test]
fn real_unrequested_fields_remain_absent() {
    let response: AnalysisResponse = serde_json::from_str(include_str!("fixtures/katago-1.18.2-analysis-display-absent.jsonl")).unwrap();
    assert!(response.has_valid_search_result(9, 9));
    let frame = normalize_response(AnalysisJobId::nil(), response, 9, 9);
    assert!(frame.candidates.iter().all(|candidate| candidate.pv_visits.is_none() && candidate.ownership.is_none()));
}
#[test]
fn missing_and_mismatched_optional_fields_do_not_become_zero_or_root_ownership() {
    let mut value: serde_json::Value = serde_json::from_str(REAL).unwrap();
    let first = value["moveInfos"][0].as_object_mut().unwrap();
    first.remove("scoreMean");
    first.remove("ownership");
    first.insert("pvVisits".into(), serde_json::json!([999]));
    let response: AnalysisResponse = serde_json::from_value(value).unwrap();
    assert!(response.has_valid_search_result(9, 9));
    let frame = normalize_response(AnalysisJobId::nil(), response, 9, 9);
    assert_eq!(frame.candidates[0].score_mean_black, None);
    assert_eq!(frame.candidates[0].pv_visits, None);
    assert_eq!(frame.candidates[0].ownership, None);
    assert!(frame.ownership.is_some());
}
