use app_model::{CandidateDelayMode, CandidateScoreMode, CandidateTextField, OwnershipStyle, PvVisitsMode};
use app_preferences::{default_app_preferences, load_from_path, save_to_path, AppPreferencesDto};
#[test]
fn granular_display_roundtrip_invalid_input_and_real_write_failure_preserve_committed_bytes() {
    let directory = std::env::temp_dir().join(format!(
        "r13-display-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&directory).unwrap();
    let path = directory.join("preferences.json");
    let mut preferences = default_app_preferences();
    preferences.allow_drag = true;
    let display = &mut preferences.granular_display;
    display.delay_mode = CandidateDelayMode::Timed;
    display.delay_seconds = 0.125;
    display.show_black_candidates = false;
    display.pv_visits_mode = PvVisitsMode::All;
    display.pv_visits_threshold = i32::MAX;
    display.pv_length = i32::MIN;
    display.show_winrate = false;
    display.show_visits = false;
    display.show_score = false;
    display.score_mode = CandidateScoreMode::Board;
    display.show_order = false;
    display.emphasize_max = true;
    display.text_order = [
        CandidateTextField::Score,
        CandidateTextField::Winrate,
        CandidateTextField::Visits,
    ];
    display.ownership_style = OwnershipStyle::BySize;
    display.use_candidate_ownership = false;
    save_to_path(&path, preferences.clone()).unwrap();
    assert_eq!(load_from_path(&path).unwrap().preferences, preferences);
    let bytes = std::fs::read(&path).unwrap();
    let mut invalid = preferences.clone();
    invalid.granular_display.delay_seconds = f64::INFINITY;
    assert!(save_to_path(&path, invalid).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    assert!(save_to_path(&directory, preferences.clone()).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    let mut value = serde_json::to_value(&preferences).unwrap();
    value["granularDisplay"]["showScore"] = serde_json::json!("false");
    assert!(serde_json::from_value::<AppPreferencesDto>(value).is_err());
    preferences.granular_display.delay_mode = CandidateDelayMode::Manual;
    save_to_path(&path, preferences.clone()).unwrap();
    assert_eq!(load_from_path(&path).unwrap().preferences, preferences);
    assert_eq!(preferences.granular_display.delay_seconds, 0.125);
    std::fs::remove_dir_all(directory).unwrap();
}
