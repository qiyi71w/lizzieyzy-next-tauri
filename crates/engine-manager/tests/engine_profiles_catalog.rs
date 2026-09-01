use app_model::{EngineBackend, EngineProfileDto};
use engine_manager::{
    default_engine_profiles_settings, load_engine_profiles, parse_engine_profiles, save_engine_profiles,
    EngineProfileRecord, EngineProfilesSettings, DEFAULT_ENGINE_PROFILE_ID,
};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TEST_TEMP_ID: AtomicU64 = AtomicU64::new(0);

struct TestTempDir {
    path: PathBuf,
}

impl TestTempDir {
    fn new(label: &str) -> Self {
        let unique = format!(
            "{}-{}-{}",
            std::process::id(),
            NEXT_TEST_TEMP_ID.fetch_add(1, Ordering::Relaxed),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let path = std::env::temp_dir()
            .join("lizzieyzy-engine-profiles-catalog-tests")
            .join(format!("{label}-{unique}"));
        std::fs::create_dir_all(&path).unwrap();
        Self { path }
    }

    fn catalog_path(&self) -> PathBuf {
        self.path.join("lizzieyzy-next-engine-profile.json")
    }
}

impl Drop for TestTempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

fn profile(id: &str, name: &str) -> EngineProfileRecord {
    EngineProfileRecord {
        id: id.to_string(),
        profile: EngineProfileDto {
            name: name.to_string(),
            engine_path: format!("/bin/{id}"),
            model_path: None,
            config_path: None,
            working_dir: None,
            backend: EngineBackend::KataGoAnalysis,
        },
        max_visits: 800,
    }
}

fn settings(
    selected: &str,
    autoload: Option<&str>,
    profiles: Vec<EngineProfileRecord>,
) -> EngineProfilesSettings {
    EngineProfilesSettings {
        selected_profile_id: selected.to_string(),
        autoload_profile_id: autoload.map(str::to_string),
        profiles,
    }
}

#[test]
fn first_use_and_missing_file_have_no_autoload_mark() {
    let temp = TestTempDir::new("first-use");
    let path = temp.catalog_path();
    assert!(!path.exists());

    let loaded = load_engine_profiles(&path).unwrap();
    assert_eq!(loaded.autoload_profile_id, None);
    assert_eq!(loaded, default_engine_profiles_settings());
    assert!(!path.exists());
}

#[test]
fn old_format_without_autoload_field_is_off() {
    let collection = r#"{
        "selected_profile_id": "profile-a",
        "profiles": [
            {
                "id": "default",
                "profile": {
                    "name": "Local KataGo",
                    "engine_path": "",
                    "model_path": null,
                    "config_path": null,
                    "working_dir": null,
                    "backend": "kata_go_analysis"
                },
                "max_visits": 800
            },
            {
                "id": "profile-a",
                "profile": {
                    "name": "Alpha",
                    "engine_path": "/bin/a",
                    "model_path": null,
                    "config_path": null,
                    "working_dir": null,
                    "backend": "kata_go_analysis"
                },
                "max_visits": 500
            }
        ]
    }"#;
    let parsed = parse_engine_profiles(collection).unwrap();
    assert_eq!(parsed.selected_profile_id, "profile-a");
    assert_eq!(parsed.autoload_profile_id, None);

    let legacy = r#"{
        "profile": {
            "name": "Legacy",
            "engine_path": "/bin/legacy",
            "model_path": null,
            "config_path": null,
            "working_dir": null,
            "backend": "kata_go_analysis"
        },
        "max_visits": 400
    }"#;
    let migrated = parse_engine_profiles(legacy).unwrap();
    assert_eq!(migrated.selected_profile_id, DEFAULT_ENGINE_PROFILE_ID);
    assert_eq!(migrated.autoload_profile_id, None);
    assert_eq!(migrated.profiles[0].profile.name, "Legacy");
}

#[test]
fn set_replace_clear_and_reload_keep_a_single_autoload_mark() {
    let temp = TestTempDir::new("set-replace-clear");
    let path = temp.catalog_path();
    let catalog = settings(
        "default",
        None,
        vec![
            profile("default", "Local KataGo"),
            profile("alpha", "Alpha"),
            profile("beta", "Beta"),
        ],
    );

    let saved = save_engine_profiles(&path, catalog.clone()).unwrap();
    assert_eq!(saved.autoload_profile_id, None);
    assert_eq!(load_engine_profiles(&path).unwrap().autoload_profile_id, None);

    let marked = save_engine_profiles(
        &path,
        EngineProfilesSettings {
            autoload_profile_id: Some("alpha".into()),
            ..catalog.clone()
        },
    )
    .unwrap();
    assert_eq!(marked.autoload_profile_id.as_deref(), Some("alpha"));
    assert_eq!(
        load_engine_profiles(&path)
            .unwrap()
            .autoload_profile_id
            .as_deref(),
        Some("alpha")
    );

    let replaced = save_engine_profiles(
        &path,
        EngineProfilesSettings {
            autoload_profile_id: Some("beta".into()),
            ..catalog.clone()
        },
    )
    .unwrap();
    assert_eq!(replaced.autoload_profile_id.as_deref(), Some("beta"));
    let reloaded = load_engine_profiles(&path).unwrap();
    assert_eq!(reloaded.autoload_profile_id.as_deref(), Some("beta"));
    assert_eq!(reloaded.selected_profile_id, "default");

    let cleared = save_engine_profiles(
        &path,
        EngineProfilesSettings {
            autoload_profile_id: None,
            ..catalog
        },
    )
    .unwrap();
    assert_eq!(cleared.autoload_profile_id, None);
    assert_eq!(load_engine_profiles(&path).unwrap().autoload_profile_id, None);
}

#[test]
fn normalize_enforces_zero_or_one_autoload_and_drops_dangling_marks() {
    let parsed = parse_engine_profiles(
        r#"{
            "selected_profile_id": "alpha",
            "autoload_profile_id": "missing",
            "profiles": [
                {
                    "id": "default",
                    "profile": {
                        "name": "Local KataGo",
                        "engine_path": "",
                        "model_path": null,
                        "config_path": null,
                        "working_dir": null,
                        "backend": "kata_go_analysis"
                    },
                    "max_visits": 800
                },
                {
                    "id": "alpha",
                    "profile": {
                        "name": "Alpha",
                        "engine_path": "/bin/a",
                        "model_path": null,
                        "config_path": null,
                        "working_dir": null,
                        "backend": "kata_go_analysis"
                    },
                    "max_visits": 800
                }
            ]
        }"#,
    )
    .unwrap();
    assert_eq!(parsed.autoload_profile_id, None);

    let blank = parse_engine_profiles(
        r#"{
            "selected_profile_id": "default",
            "autoload_profile_id": "   ",
            "profiles": [{
                "id": "default",
                "profile": {
                    "name": "Local KataGo",
                    "engine_path": "",
                    "model_path": null,
                    "config_path": null,
                    "working_dir": null,
                    "backend": "kata_go_analysis"
                },
                "max_visits": 800
            }]
        }"#,
    )
    .unwrap();
    assert_eq!(blank.autoload_profile_id, None);
}

#[test]
fn deleting_inactive_marked_profile_clears_mark_in_the_same_document() {
    let temp = TestTempDir::new("delete-marked");
    let path = temp.catalog_path();
    save_engine_profiles(
        &path,
        settings(
            "default",
            Some("alpha"),
            vec![profile("default", "Local KataGo"), profile("alpha", "Alpha")],
        ),
    )
    .unwrap();

    let after_delete = save_engine_profiles(
        &path,
        settings("default", Some("alpha"), vec![profile("default", "Local KataGo")]),
    )
    .unwrap();
    assert_eq!(after_delete.autoload_profile_id, None);
    assert!(after_delete.profiles.iter().all(|record| record.id != "alpha"));

    let reloaded = load_engine_profiles(&path).unwrap();
    assert_eq!(reloaded.autoload_profile_id, None);
    assert!(reloaded.profiles.iter().all(|record| record.id != "alpha"));
}

#[test]
fn catalog_wire_keeps_snake_case_autoload_identity() {
    let saved = save_engine_profiles(
        &TestTempDir::new("wire").catalog_path(),
        settings(
            "default",
            Some("alpha"),
            vec![profile("default", "Local KataGo"), profile("alpha", "Alpha")],
        ),
    )
    .unwrap();
    let json = serde_json::to_string(&saved).unwrap();
    assert!(json.contains("autoload_profile_id"));
    assert!(json.contains("selected_profile_id"));
    assert!(!json.contains("autoloadProfileId"));
    let parsed: EngineProfilesSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.autoload_profile_id.as_deref(), Some("alpha"));
}
