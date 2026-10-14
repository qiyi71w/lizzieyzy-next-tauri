use app_preferences::{default_app_preferences, load_from_path, save_to_path, AppPreferencesDto};

#[test]
fn explicit_flash_budget_is_independent_positive_and_atomic() {
    let original = default_app_preferences();
    let mut value = serde_json::to_value(&original).unwrap();
    value.as_object_mut().unwrap().remove("flashAnalysisVisits");
    let mut preferences: AppPreferencesDto = serde_json::from_value(value).unwrap();
    assert_eq!(preferences.flash_analysis_visits, 1);
    let directory = std::env::temp_dir().join(format!(
        "r13-flash-preferences-{}-{}", std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
    ));
    std::fs::create_dir(&directory).unwrap();
    let path = directory.join("preferences.json");
    preferences.flash_analysis_visits = 17;
    save_to_path(&path, preferences.clone()).unwrap();
    let loaded = load_from_path(&path).unwrap().preferences;
    assert_eq!(loaded.flash_analysis_visits, 17);
    assert_eq!(loaded.auto_load, original.auto_load);
    assert_eq!(loaded.task_single_stage_conditions, original.task_single_stage_conditions);
    let bytes = std::fs::read(&path).unwrap();
    preferences.flash_analysis_visits = 0;
    assert!(save_to_path(&path, preferences.clone()).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    preferences.flash_analysis_visits = u32::MAX;
    assert!(save_to_path(&directory, preferences.clone()).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    save_to_path(&path, preferences).unwrap();
    assert_eq!(load_from_path(&path).unwrap().preferences.flash_analysis_visits, u32::MAX);
    std::fs::remove_dir_all(directory).unwrap();
}
