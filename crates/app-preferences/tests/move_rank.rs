use app_model::{MoveRankMode, MoveRankPreferencesDto};
use app_preferences::{default_app_preferences, load_from_path, save_to_path, AppPreferencesDto};

#[test]
fn move_rank_defaults_roundtrip_validation_and_failed_atomic_save() {
    let directory = std::env::temp_dir().join(format!("r13-rank-{}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    std::fs::create_dir(&directory).unwrap();
    let path = directory.join("preferences.json");
    let defaults = default_app_preferences();
    assert_eq!(defaults.move_rank, MoveRankPreferencesDto::default());
    let mut old = serde_json::to_value(&defaults).unwrap();
    old.as_object_mut().unwrap().remove("moveRank");
    assert_eq!(serde_json::from_value::<AppPreferencesDto>(old).unwrap().move_rank, defaults.move_rank);
    let mut settings = defaults.clone();
    settings.move_rank = MoveRankPreferencesDto { mode: MoveRankMode::Combined, winrate_thresholds_pp: [-1.0, 0.0, 0.0, 101.123456789, 200.0], score_thresholds: [-2.0, -1.0, 0.0, 0.0, 1000.0] };
    save_to_path(&path, settings.clone()).unwrap();
    assert_eq!(load_from_path(&path).unwrap().preferences.move_rank, settings.move_rank);
    let committed = std::fs::read(&path).unwrap();
    for thresholds in [[1.0, 0.0, 3.0, 4.0, 5.0], [0.0, 1.0, 2.0, 3.0, f64::INFINITY], [f64::NAN; 5]] {
        let mut invalid = settings.clone();
        invalid.move_rank.score_thresholds = thresholds;
        assert!(save_to_path(&path, invalid).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), committed);
    }
    let blocked_temporary_path = directory.join("preferences.json.tmp");
    std::fs::create_dir(&blocked_temporary_path).unwrap();
    assert!(save_to_path(&path, defaults.clone()).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), committed);
    std::fs::remove_dir(&blocked_temporary_path).unwrap();
    assert_eq!(load_from_path(&path).unwrap().preferences.move_rank, settings.move_rank);
    save_to_path(&path, defaults.clone()).unwrap();
    assert_eq!(load_from_path(&path).unwrap().preferences.move_rank, defaults.move_rank);
    std::fs::remove_dir_all(directory).unwrap();
}
