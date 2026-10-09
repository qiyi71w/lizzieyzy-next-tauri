use app_model::{EngineAdapterSettings, EngineProfileDto, EngineStartupPolicyDto, GenericGtpSettings, KataGoSettings};
use engine_manager::{
    default_engine_profiles_settings, load_engine_profiles, parse_engine_profiles, save_engine_profiles,
    EngineProfileRecord, EngineProfilesSettings, DEFAULT_ENGINE_PROFILE_ID, ENGINE_PROFILES_VERSION,
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
            program: format!("/bin/{id}"),
            argv: vec![],
            working_dir: None,
            adapter: EngineAdapterSettings::KataGoAnalysis(KataGoSettings {
                model_path: None,
                config_path: None,
                max_visits: 800,
            }),
        },
    }
}

fn settings(
    selected: &str,
    autoload: Option<&str>,
    profiles: Vec<EngineProfileRecord>,
) -> EngineProfilesSettings {
    EngineProfilesSettings {
        version: ENGINE_PROFILES_VERSION,
        selected_profile_id: selected.to_string(),
        startup: autoload.map_or(EngineStartupPolicyDto::Off, |id| EngineStartupPolicyDto::Fixed { profile_id: id.into() }),
        last_primary_profile_id: None,
        profiles,
    }
}

#[test]
fn first_use_and_missing_file_have_no_autoload_mark() {
    let temp = TestTempDir::new("first-use");
    let path = temp.catalog_path();
    assert!(!path.exists());

    let loaded = load_engine_profiles(&path).unwrap();
    assert_eq!(loaded.startup_profile_id(), None);
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
    assert_eq!(parsed.startup_profile_id(), None);

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
    assert_eq!(migrated.startup_profile_id(), None);
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
    assert_eq!(saved.startup_profile_id(), None);
    assert_eq!(load_engine_profiles(&path).unwrap().startup_profile_id(), None);

    let marked = save_engine_profiles(
        &path,
        EngineProfilesSettings {
            startup: EngineStartupPolicyDto::Fixed { profile_id: "alpha".into() },
            ..catalog.clone()
        },
    )
    .unwrap();
    assert_eq!(marked.startup_profile_id(), Some("alpha"));
    assert_eq!(load_engine_profiles(&path).unwrap().startup_profile_id(), Some("alpha"));

    let replaced = save_engine_profiles(
        &path,
        EngineProfilesSettings {
            startup: EngineStartupPolicyDto::Fixed { profile_id: "beta".into() },
            ..catalog.clone()
        },
    )
    .unwrap();
    assert_eq!(replaced.startup_profile_id(), Some("beta"));
    let reloaded = load_engine_profiles(&path).unwrap();
    assert_eq!(reloaded.startup_profile_id(), Some("beta"));
    assert_eq!(reloaded.selected_profile_id, "default");

    let cleared = save_engine_profiles(
        &path,
        EngineProfilesSettings {
            startup: EngineStartupPolicyDto::Off,
            ..catalog
        },
    )
    .unwrap();
    assert_eq!(cleared.startup_profile_id(), None);
    assert_eq!(load_engine_profiles(&path).unwrap().startup_profile_id(), None);
}

#[test]
fn corrupt_identities_are_rejected_without_replacing_storage() {
    let temp = TestTempDir::new("dangling-identities");
    let path = temp.catalog_path();
    let valid = settings("alpha", Some("alpha"), vec![profile("alpha", "Alpha")]);
    save_engine_profiles(&path, valid.clone()).unwrap();
    let before = std::fs::read(&path).unwrap();
    for candidate in [
        EngineProfilesSettings {
            selected_profile_id: "missing".into(),
            ..valid.clone()
        },
        EngineProfilesSettings {
            startup: EngineStartupPolicyDto::Fixed { profile_id: "bad\0id".into() },
            ..valid.clone()
        },
        EngineProfilesSettings {
            startup: EngineStartupPolicyDto::Fixed { profile_id: "   ".into() },
            ..valid.clone()
        },
    ] {
        assert!(save_engine_profiles(&path, candidate).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), before);
        assert_eq!(load_engine_profiles(&path).unwrap(), valid);
    }
}

#[test]
fn deleting_inactive_fixed_profile_retains_missing_identity_without_fallback() {
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
    assert_eq!(after_delete.startup_profile_id(), Some("alpha"));
    assert!(after_delete.profiles.iter().all(|record| record.id != "alpha"));

    let reloaded = load_engine_profiles(&path).unwrap();
    assert_eq!(reloaded.startup_profile_id(), Some("alpha"));
    assert!(reloaded.profiles.iter().all(|record| record.id != "alpha"));
}

#[test]
fn legacy_read_preserves_every_value_and_never_rewrites_until_explicit_save() {
    let temp = TestTempDir::new("legacy-preservation");
    let path = temp.catalog_path();
    let legacy = serde_json::json!({
        "selected_profile_id": "second", "autoload_profile_id": "first",
        "profiles": [
            {"id":"first", "max_visits":123, "profile": {
                "name":"  引擎 一  ", "engine_path":"  D:\\围棋 引擎\\katago.exe  ",
                "model_path":"模型 空格.bin", "config_path":" config spaced.cfg ",
                "working_dir":" D:\\工作 目录 ", "backend":"kata_go_analysis"
            }},
            {"id":"second", "max_visits":456, "profile": {
                "name":"Second", "engine_path":"relative engine", "model_path":null,
                "config_path":null, "working_dir":null, "backend":"kata_go_analysis"
            }}
        ]
    });
    let original = serde_json::to_string_pretty(&legacy).unwrap();
    std::fs::write(&path, &original).unwrap();
    let loaded = load_engine_profiles(&path).unwrap();
    assert_eq!(std::fs::read_to_string(&path).unwrap(), original);
    assert_eq!(loaded.version, 2);
    assert_eq!(loaded.selected_profile_id, "second");
    assert_eq!(loaded.startup_profile_id(), Some("first"));
    assert_eq!(
        loaded
            .profiles
            .iter()
            .map(|record| record.id.as_str())
            .collect::<Vec<_>>(),
        ["first", "second"]
    );
    assert_eq!(
        loaded.profiles[0].profile,
        EngineProfileDto {
            name: "  引擎 一  ".into(),
            program: "  D:\\围棋 引擎\\katago.exe  ".into(),
            argv: vec![],
            working_dir: Some(" D:\\工作 目录 ".into()),
            adapter: EngineAdapterSettings::KataGoAnalysis(KataGoSettings {
                model_path: Some("模型 空格.bin".into()),
                config_path: Some(" config spaced.cfg ".into()),
                max_visits: 123,
            }),
        }
    );
    assert_eq!(
        loaded.profiles[1].profile.adapter,
        EngineAdapterSettings::KataGoAnalysis(KataGoSettings {
            model_path: None,
            config_path: None,
            max_visits: 456,
        })
    );
    save_engine_profiles(&path, loaded.clone()).unwrap();
    let persisted: serde_json::Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(persisted["version"], 2);
    assert_eq!(persisted["profiles"][0]["profile"]["settings"]["max_visits"], 123);
    assert!(persisted["profiles"][0].get("max_visits").is_none());
    assert!(persisted["profiles"][0]["profile"].get("engine_path").is_none());
    assert_eq!(load_engine_profiles(&path).unwrap(), loaded);

    let single = serde_json::json!({"profile": legacy["profiles"][0]["profile"], "max_visits": 123});
    std::fs::write(&path, single.to_string()).unwrap();
    let migrated = load_engine_profiles(&path).unwrap();
    assert_eq!(migrated.profiles[0].profile, loaded.profiles[0].profile);
    assert_eq!(migrated.profiles[0].id, DEFAULT_ENGINE_PROFILE_ID);
    assert_eq!(std::fs::read_to_string(&path).unwrap(), single.to_string());
}

#[test]
fn malformed_version_adapter_and_settings_remain_observable_without_storage_changes() {
    let temp = TestTempDir::new("invalid-documents");
    let path = temp.catalog_path();
    let base = serde_json::to_value(settings("alpha", None, vec![profile("alpha", "Alpha")])).unwrap();
    let mut cases = Vec::new();
    for version in [
        serde_json::json!(3),
        serde_json::Value::Null,
        serde_json::json!("1"),
        serde_json::json!(1.5),
    ] {
        let mut value = base.clone();
        value["version"] = version;
        cases.push(value);
    }
    for (field, invalid) in [
        ("settings", serde_json::Value::Null),
        (
            "settings",
            serde_json::json!({"model_path":null,"config_path":null,"max_visits":0}),
        ),
        (
            "settings",
            serde_json::json!({"model_path":null,"config_path":null,"max_visits":800,"command":"gtp"}),
        ),
        ("adapter_kind", serde_json::json!("unknown")),
        ("adapter_kind", serde_json::json!("generic_gtp")),
    ] {
        let mut value = base.clone();
        value["profiles"][0]["profile"][field] = invalid;
        cases.push(value);
    }
    let mut absent = base.clone();
    absent["profiles"][0]["profile"]
        .as_object_mut()
        .unwrap()
        .remove("settings");
    cases.push(absent);
    cases.push(serde_json::json!({"version":1,"profile":{"name":"old","engine_path":"/bin/old","backend":"kata_go_analysis"},"max_visits":800}));
    cases.push(serde_json::json!({"profile":{"name":"old","engine_path":"/bin/old","backend":"leela_zero_gtp"},"max_visits":800}));
    for value in cases {
        let text = value.to_string();
        std::fs::write(&path, &text).unwrap();
        assert!(load_engine_profiles(&path).is_err(), "accepted {text}");
        assert_eq!(std::fs::read_to_string(&path).unwrap(), text);
    }
}

#[test]
fn invalid_save_preserves_effective_catalog_and_previous_bytes() {
    let temp = TestTempDir::new("invalid-save");
    let path = temp.catalog_path();
    let valid = settings("alpha", Some("alpha"), vec![profile("alpha", "Alpha")]);
    save_engine_profiles(&path, valid.clone()).unwrap();
    let previous = std::fs::read(&path).unwrap();
    let mut invalid_profiles = Vec::new();
    for field in 0..6 {
        let mut record = profile("alpha", "Alpha");
        match field {
            0 => record.profile.program = "bad\0program".into(),
            1 => record.profile.argv = vec!["bad\0argument".into()],
            2 => record.profile.working_dir = Some("bad\0cwd".into()),
            3 => record.profile.name = "  ".into(),
            4 | 5 => {
                if let EngineAdapterSettings::KataGoAnalysis(settings) = &mut record.profile.adapter {
                    if field == 4 {
                        settings.model_path = Some("bad\0model".into());
                    } else {
                        settings.config_path = Some("bad\0config".into());
                    }
                }
            }
            _ => unreachable!(),
        }
        invalid_profiles.push(record);
    }
    for token in [
        "analysis",
        "gtp",
        "benchmark",
        "selfplay",
        "-model",
        "--model=other",
        "-config=other",
        "--config",
    ] {
        let mut record = profile("alpha", "Alpha");
        record.profile.argv.push(token.into());
        invalid_profiles.push(record);
    }
    for record in invalid_profiles {
        assert!(save_engine_profiles(&path, settings("alpha", Some("alpha"), vec![record])).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), previous);
        assert_eq!(load_engine_profiles(&path).unwrap(), valid);
    }
    let mut duplicate = valid.clone();
    duplicate.profiles.push(duplicate.profiles[0].clone());
    assert!(save_engine_profiles(&path, duplicate).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), previous);
}

#[test]
fn generic_settings_roundtrip_preserves_unrestricted_argv() {
    let temp = TestTempDir::new("generic-roundtrip");
    let mut record = profile("generic", "通用 引擎");
    record.profile.argv = vec!["gtp".into(), "--model=x y".into(), "".into(), "中文".into()];
    record.profile.adapter = EngineAdapterSettings::GenericGtp(GenericGtpSettings {});
    let catalog = settings("generic", Some("generic"), vec![record]);
    save_engine_profiles(&temp.catalog_path(), catalog.clone()).unwrap();
    assert_eq!(load_engine_profiles(&temp.catalog_path()).unwrap(), catalog);
}

#[test]
fn katago_layered_config_arguments_roundtrip_in_source_order() {
    let temp = TestTempDir::new("layered-config-roundtrip");
    let mut record = profile("layered", "Layered KataGo");
    record.profile.argv = vec!["-config".into(), "配置, space.cfg".into(), "-config".into(), "second.cfg".into(), "-override-config".into(), "homeDataDir=custom".into()];
    let catalog = settings("layered", Some("layered"), vec![record]);
    save_engine_profiles(&temp.catalog_path(), catalog.clone()).unwrap();
    assert_eq!(load_engine_profiles(&temp.catalog_path()).unwrap(), catalog);
}

#[test]
fn reorder_persists_stable_ids_without_changing_catalog_identities_or_records() {
    let temp = TestTempDir::new("reorder");
    let path = temp.catalog_path();
    let current = settings(
        "beta",
        Some("alpha"),
        vec![
            profile("alpha", "Alpha"),
            profile("beta", "Beta"),
            profile("gamma", "Gamma"),
        ],
    );
    save_engine_profiles(&path, current.clone()).unwrap();
    let request = app_model::EngineProfileOrderRequestDto {
        expected_profile_ids: vec!["alpha".into(), "beta".into(), "gamma".into()],
        profile_ids: vec!["gamma".into(), "alpha".into(), "beta".into()],
    };
    let reordered = engine_manager::reorder_engine_profiles(&path, current.clone(), &request).unwrap();
    assert_eq!(reordered.selected_profile_id, "beta");
    assert_eq!(reordered.startup_profile_id(), Some("alpha"));
    assert_eq!(
        reordered.profiles,
        vec![
            current.profiles[2].clone(),
            current.profiles[0].clone(),
            current.profiles[1].clone()
        ]
    );
    assert_eq!(load_engine_profiles(&path).unwrap(), reordered);
}

#[test]
fn stale_invalid_and_boundary_orders_never_replace_durable_catalog() {
    let temp = TestTempDir::new("invalid-orders");
    let path = temp.catalog_path();
    let current = settings(
        "beta",
        Some("alpha"),
        vec![profile("alpha", "Alpha"), profile("beta", "Beta")],
    );
    save_engine_profiles(&path, current.clone()).unwrap();
    let before = std::fs::read(&path).unwrap();
    for (expected, ordered) in [
        (vec!["beta", "alpha"], vec!["beta", "alpha"]),
        (vec!["alpha", "beta", "gamma"], vec!["beta", "alpha"]),
        (vec!["alpha", "beta"], vec!["alpha"]),
        (vec!["alpha", "beta"], vec!["beta", "beta"]),
        (vec!["alpha", "beta"], vec!["beta", "missing"]),
        (vec!["alpha", "beta"], vec!["alpha", "beta", "gamma"]),
    ] {
        let request = app_model::EngineProfileOrderRequestDto {
            expected_profile_ids: expected.into_iter().map(str::to_string).collect(),
            profile_ids: ordered.into_iter().map(str::to_string).collect(),
        };
        assert!(engine_manager::reorder_engine_profiles(&path, current.clone(), &request).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), before);
        assert_eq!(load_engine_profiles(&path).unwrap(), current);
    }
    std::fs::create_dir(path.with_extension("json.tmp")).unwrap();
    let no_op = app_model::EngineProfileOrderRequestDto {
        expected_profile_ids: vec!["alpha".into(), "beta".into()],
        profile_ids: vec!["alpha".into(), "beta".into()],
    };
    assert_eq!(
        engine_manager::reorder_engine_profiles(&path, current.clone(), &no_op).unwrap(),
        current
    );
    let reverse = app_model::EngineProfileOrderRequestDto {
        profile_ids: vec!["beta".into(), "alpha".into()],
        ..no_op
    };
    assert!(
        engine_manager::reorder_engine_profiles(&path, current.clone(), &reverse)
            .unwrap_err()
            .contains("write")
    );
    assert_eq!(std::fs::read(&path).unwrap(), before);
    assert_eq!(load_engine_profiles(&path).unwrap(), current);
}

#[test]
fn version_one_migrates_off_and_fixed_without_rewriting_bytes() {
    let temp = TestTempDir::new("v1-startup");
    let path = temp.catalog_path();
    for autoload in [None, Some("alpha")] {
        let original = settings("beta", autoload, vec![profile("alpha", "A"), profile("beta", "B")]);
        let mut old = serde_json::to_value(&original).unwrap();
        let object = old.as_object_mut().unwrap();
        object.remove("startup");
        object.remove("last_primary_profile_id");
        object.insert("version".into(), serde_json::json!(1));
        object.insert("autoload_profile_id".into(), serde_json::json!(autoload));
        let bytes = serde_json::to_vec(&old).unwrap();
        std::fs::write(&path, &bytes).unwrap();
        assert_eq!(load_engine_profiles(&path).unwrap(), original);
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        save_engine_profiles(&path, original).unwrap();
        let saved: serde_json::Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        assert_eq!(saved["version"], 2);
        assert!(saved.get("autoload_profile_id").is_none());
    }
}

#[test]
fn last_primary_reopens_independently_of_editor_and_retains_durable_identity_on_failed_write() {
    let temp = TestTempDir::new("last-primary");
    let path = temp.catalog_path();
    let mut current = settings("beta", None, vec![profile("alpha", "A"), profile("beta", "B")]);
    current.startup = EngineStartupPolicyDto::LastPrimary;
    save_engine_profiles(&path, current.clone()).unwrap();
    assert_eq!(load_engine_profiles(&path).unwrap().startup_profile_id(), None);
    current = engine_manager::persist_last_primary(&path, current, Some("alpha".into())).unwrap();
    assert_eq!(load_engine_profiles(&path).unwrap().startup_profile_id(), Some("alpha"));
    let mut stale_form = current.clone();
    stale_form.last_primary_profile_id = Some("beta".into());
    let prepared = engine_manager::prepare_engine_profiles_save(&current, stale_form).unwrap();
    assert_eq!(prepared.last_primary_profile_id.as_deref(), Some("alpha"));
    let reordered = engine_manager::reorder_engine_profiles(&path, current.clone(), &app_model::EngineProfileOrderRequestDto {
        expected_profile_ids: vec!["alpha".into(), "beta".into()],
        profile_ids: vec!["beta".into(), "alpha".into()],
    }).unwrap();
    assert_eq!(reordered.startup_profile_id(), Some("alpha"));
    let before = std::fs::read(&path).unwrap();
    std::fs::create_dir(path.with_extension("json.tmp")).unwrap();
    assert!(engine_manager::persist_last_primary(&path, reordered.clone(), Some("beta".into())).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), before);
    assert_eq!(load_engine_profiles(&path).unwrap(), reordered);
    // A session with no established primary is not a request to clear last-good.
    assert_eq!(engine_manager::persist_last_primary(&path, reordered.clone(), None).unwrap(), reordered);
}

#[test]
fn missing_deleted_and_corrupt_last_primary_never_select_the_editor() {
    let mut current = settings("beta", None, vec![profile("beta", "B")]);
    current.startup = EngineStartupPolicyDto::LastPrimary;
    assert_eq!(current.startup_profile_id(), None);
    current.last_primary_profile_id = Some("deleted".into());
    let parsed = parse_engine_profiles(&serde_json::to_string(&current).unwrap()).unwrap();
    assert_eq!(parsed.startup_profile_id(), Some("deleted"));
    for invalid in [serde_json::json!(12), serde_json::json!(""), serde_json::json!("bad\0id")] {
        let mut value = serde_json::to_value(&current).unwrap();
        value["last_primary_profile_id"] = invalid;
        assert!(parse_engine_profiles(&value.to_string()).is_err());
    }
}
