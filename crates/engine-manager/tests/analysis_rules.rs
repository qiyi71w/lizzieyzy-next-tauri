use app_model::NodePath;
use katago_protocol::{analysis_query_from_position, AnalysisQueryOptions};
use sgf::CurrentSgfDocument;

#[test]
fn yike_cn_analysis_uses_chinese_without_rewriting_the_saved_game() {
    // Public Yike room 196315, captured during SPEC-F01 native acceptance.
    let mut document = CurrentSgfDocument::open(include_str!("fixtures/yike-cn.sgf")).unwrap();
    let path = NodePath { indices: vec![0, 0] };
    document.set_personal_comment(&path, "Personal review").unwrap();
    let before = document.serialize().unwrap();
    let snapshot = document.snapshot(&path).unwrap();
    let query = analysis_query_from_position(
        document.board_width(),
        document.board_height(),
        document.komi(),
        &snapshot.position.stones,
        snapshot.position.to_play,
        AnalysisQueryOptions {
            id: "yike-cn".to_string(),
            rules: document.rules(),
            turn: 0,
            max_visits: Some(8),
            include_ownership: None,
            include_policy: None,
        },
    )
    .unwrap();
    let wire: serde_json::Value = serde_json::from_str(&query.to_jsonl().unwrap()).unwrap();
    assert_eq!(wire["rules"], "chinese");
    assert_eq!(query.rules, "cn");
    assert_eq!(document.serialize().unwrap(), before);
    let reopened = CurrentSgfDocument::open(&before).unwrap();
    assert_eq!(reopened.rules(), "cn");
    assert_eq!(
        reopened.snapshot(&path).unwrap().personal_comment,
        "Personal review"
    );
    assert!(before.contains("RU[cn]"));
    assert!(before.contains("PB[翟群智]PW[汪鹏飞]"));
}
