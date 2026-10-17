use app_preferences::{default_app_preferences, load_from_path, save_to_path, AppPreferencesDto};

#[test]
fn sgf_landing_defaults_and_durable_values_preserve_other_preferences() {
    assert!(!default_app_preferences().load_sgf_last);
    let mut legacy = serde_json::to_value(default_app_preferences()).unwrap();
    legacy.as_object_mut().unwrap().remove("loadSgfLast");
    let mut preferences: AppPreferencesDto = serde_json::from_value(legacy).unwrap();
    assert!(!preferences.load_sgf_last);
    preferences.sound_enabled = false;
    preferences.recent_game_paths = vec!["/tmp/retained.sgf".into()];
    let directory = std::env::temp_dir().join(format!("sgf-landing-{}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    std::fs::create_dir(&directory).unwrap();
    let path = directory.join("preferences.json");
    for value in [true, false] {
        preferences.load_sgf_last = value;
        save_to_path(&path, preferences.clone()).unwrap();
        assert_eq!(load_from_path(&path).unwrap().preferences, preferences);
    }
    let before = std::fs::read(&path).unwrap();
    preferences.load_sgf_last = true;
    assert!(save_to_path(&directory, preferences).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), before);
    assert!(!load_from_path(&path).unwrap().preferences.load_sgf_last);
    let mut invalid = serde_json::from_slice::<serde_json::Value>(&before).unwrap();
    invalid["loadSgfLast"] = serde_json::json!("true");
    std::fs::write(&path, serde_json::to_vec(&invalid).unwrap()).unwrap();
    let recovered = load_from_path(&path).unwrap();
    assert!(recovered.recovery.is_some());
    assert!(!recovered.preferences.load_sgf_last);
    std::fs::remove_dir_all(directory).unwrap();
}
