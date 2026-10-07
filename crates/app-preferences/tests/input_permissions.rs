use app_preferences::{default_app_preferences, load_from_path, save_to_path, AppPreferencesDto};

#[test]
fn java_input_permission_defaults_are_independent_and_survive_restart_and_failed_write() {
    let mut value = serde_json::to_value(default_app_preferences()).unwrap();
    value.as_object_mut().unwrap().remove("allowDrag");
    value.as_object_mut().unwrap().remove("allowDoubleClick");
    value["enableClickReview"] = serde_json::json!(true);
    let loaded: AppPreferencesDto = serde_json::from_value(value).unwrap();
    assert!(!loaded.allow_drag);
    assert!(loaded.allow_double_click);
    let directory = std::env::temp_dir().join(format!(
        "r11-input-permissions-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&directory).unwrap();
    let path = directory.join("preferences.json");
    let mut changed = loaded;
    changed.allow_drag = true;
    changed.allow_double_click = false;
    save_to_path(&path, changed.clone()).unwrap();
    let restarted = load_from_path(&path).unwrap().preferences;
    assert!(restarted.allow_drag);
    assert!(!restarted.allow_double_click);
    let bytes = std::fs::read(&path).unwrap();
    assert!(save_to_path(&directory, changed).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    let reread = load_from_path(&path).unwrap().preferences;
    assert!(reread.allow_drag);
    assert!(!reread.allow_double_click);
    std::fs::remove_dir_all(directory).unwrap();
}
