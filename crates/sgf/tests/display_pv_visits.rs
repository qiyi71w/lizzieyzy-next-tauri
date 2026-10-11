use sgf::{encode_analysis_payload, parse_analysis_payload, AnalysisSlot};
#[test]
fn display_pv_visits_survive_roundtrip_without_truncating_pv() {
    let raw = "KataGo 40.0 900\nmove A9 visits 900 winrate 6000 pv A9 pass B8 pvVisits 12 8 3";
    let payload = parse_analysis_payload(raw, 9, 9).unwrap();
    assert_eq!(
        payload.candidates[0].pv_visits.as_deref(),
        Some([12, 8, 3].as_slice())
    );
    assert_eq!(payload.candidates[0].pv.len(), 3);
    let encoded = encode_analysis_payload(&payload, AnalysisSlot::Primary { root: false }, 9, 9);
    let reloaded = parse_analysis_payload(&encoded.values[0], 9, 9).unwrap();
    assert_eq!(reloaded.candidates, payload.candidates);
    let malformed = parse_analysis_payload(&raw.replace("12 8 3", "900"), 9, 9).unwrap();
    assert_eq!(malformed.candidates[0].pv_visits, None);
    assert_eq!(malformed.candidates[0].pv.len(), 3);
}
