use app_model::MeasuredSceneDto;
use engine_manager::measured_report::{
    exact_count,
    fingerprint::{capture, semantic_equal, GpuSample},
    parse,
};
use engine_manager::CommandSpec;
use serde_json::{json, Value};
use std::time::{Duration, Instant};

fn valid(whole: bool, pairs: u32) -> Value {
    let hash = "a".repeat(64);
    let mut runs = Vec::new();
    for round in 1..=pairs {
        for candidate in if round % 2 == 1 {
            [false, true]
        } else {
            [true, false]
        } {
            let mut row = json!({"profile":if candidate{"candidate"}else{"baseline"},"round":round,"phase":"warm","seconds":if candidate{1.0}else{1.04},"responseSeconds":0.01,"edtP95Seconds":0.002,"maxMemoryMiB":1024,"maxEngineProcesses":1});
            if whole {
                row["rootVisitsByTurn"] = json!([100, 100]);
            } else {
                row["firstResultSeconds"] = json!(0.05);
                row["observedRootVisits"] = json!(100);
            }
            runs.push(row);
        }
    }
    json!({"schemaVersion":1,"scene":if whole{"whole-game"}else{"live"},"measurementMode":"application","metricScope":"totalGpu","fingerprint":{"engineSha256":hash,"modelSha256":hash,"configSha256":hash,"configIncludes":[],"gpuName":"Named GPU","driverVersion":"591.66","memoryMiB":12227,"commandSemantics":{}},"fixtureSha256":hash,"budget":100,"positions":if whole{2}else{1},"baselineParameters":if whole{json!({"numAnalysisThreads":1,"numSearchThreadsPerAnalysisThread":4,"nnMaxBatchSize":8})}else{json!({"numSearchThreads":4,"nnMaxBatchSize":8})},"candidateParameters":if whole{json!({"numAnalysisThreads":2,"numSearchThreadsPerAnalysisThread":4,"nnMaxBatchSize":8})}else{json!({"numSearchThreads":8,"nnMaxBatchSize":8})},"runs":runs})
}
fn parsed(value: &Value) -> engine_manager::measured_report::Report {
    parse(&serde_json::to_vec(value).unwrap()).unwrap()
}
#[test]
fn exact_integer_lexemes_never_round_through_double() {
    for (text, expected) in [
        ("1.0", 1),
        ("1e2", 100),
        ("2147483647.0", 2147483647),
        ("100e-2", 1),
        ("-0.0", 0),
    ] {
        assert_eq!(
            exact_count(&serde_json::from_str(text).unwrap(), 0).unwrap(),
            expected,
            "{text}"
        );
    }
    for text in [
        "1.00000000000000000001",
        "2147483647.0000000001",
        "2147483648",
        "1e1000",
        "1e-1000",
        "true",
        "\"1\"",
        "null",
        "-1",
    ] {
        assert!(
            exact_count(&serde_json::from_str(text).unwrap(), 0).is_err(),
            "{text}"
        );
    }
}
#[test]
fn schema_compatibility_and_cross_scene_null_fields() {
    for whole in [false, true] {
        for n in [3, 5] {
            assert!(parsed(&valid(whole, n)).assess().eligible);
        }
    }
    let mut v = valid(false, 3);
    v["extra"] = json!({"permitted":true});
    v["fingerprint"]["extra"] = json!(7);
    v["fingerprint"]["commandSemantics"]["external"] = json!([true]);
    v["runs"][0]["extra"] = json!(1);
    assert!(parsed(&v).assess().eligible);
    for whole in [false, true] {
        let mut v = valid(whole, 3);
        v["runs"][0][if whole {
            "firstResultSeconds"
        } else {
            "rootVisitsByTurn"
        }] = Value::Null;
        assert!(parse(&serde_json::to_vec(&v).unwrap()).is_err());
    }
    let mut v = valid(false, 3);
    v["candidateParameters"]["other"] = json!(1);
    assert!(parse(&serde_json::to_vec(&v).unwrap()).is_err());
    let text = serde_json::to_string(&valid(false, 3)).unwrap().replacen(
        "\"schemaVersion\":1",
        "\"schemaVersion\":1,\"schemaVersion\":1",
        1,
    );
    assert!(parse(text.as_bytes()).is_err());
    assert!(parse(&vec![b' '; 1024 * 1024 + 1]).is_err());
}
#[test]
fn ordered_pairs_and_each_budget_are_required() {
    for change in 0..7 {
        let mut v = valid(false, 3);
        match change {
            0 => v["runs"][0]["round"] = json!(2),
            1 => v["runs"][2]["profile"] = json!("baseline"),
            2 => v["runs"][0]["seconds"] = json!(1),
            3 => v["runs"][0]["observedRootVisits"] = json!(99),
            4 => v["runs"][0]["maxEngineProcesses"] = json!(0),
            5 => v["candidateParameters"] = v["baselineParameters"].clone(),
            _ => {
                v["runs"].as_array_mut().unwrap().pop();
            }
        }
        assert!(!parsed(&v).assess().eligible, "case {change}");
    }
    let mut v = valid(true, 3);
    v["runs"][1]["rootVisitsByTurn"][1] = json!(99);
    assert!(!parsed(&v).assess().eligible);
}
#[test]
fn numerical_quality_boundaries_include_tail_and_both_memory_bounds() {
    let mut v = valid(false, 3);
    for row in v["runs"].as_array_mut().unwrap() {
        if row["profile"] == "baseline" {
            row["seconds"] = json!(1.03);
        }
    }
    assert!(parsed(&v).assess().eligible);
    v["runs"][1]["responseSeconds"] = json!(0.04);
    assert!(!parsed(&v).assess().eligible);
    let mut v = valid(false, 3);
    v["runs"][1]["maxMemoryMiB"] = json!(1434.0);
    assert!(!parsed(&v).assess().eligible);
    let mut v = valid(false, 3);
    v["fingerprint"]["memoryMiB"] = json!(1100);
    assert!(!parsed(&v).assess().eligible);
    let mut v = valid(false, 3);
    v["runs"][4]["seconds"] = json!(2.0);
    v["runs"][5]["seconds"] = json!(1.9);
    assert!(parsed(&v).assess().needs_five_pairs);
    let mut v = valid(false, 5);
    v["runs"][8]["seconds"] = json!(3.0);
    v["runs"][9]["seconds"] = json!(2.9);
    assert!(!parsed(&v).assess().eligible);
}
#[test]
fn full_fingerprint_equality_is_semantic_but_never_ignores_extra_keys() {
    assert!(semantic_equal(
        &json!({"memoryMiB":12227}),
        &json!({"memoryMiB":12227.0})
    ));
    assert!(!semantic_equal(
        &json!({"memoryMiB":12227,"extra":0}),
        &json!({"memoryMiB":12227.0})
    ));
    assert!(!semantic_equal(
        &serde_json::from_str("1.00000000000000000001").unwrap(),
        &json!(1)
    ));
}
#[test]
fn fingerprint_preserves_order_and_detects_equal_size_replacement_and_retirement() {
    let dir = std::env::temp_dir().join(format!("measured-fingerprint-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("engine"), b"engine-a").unwrap();
    std::fs::write(dir.join("model"), b"model-a").unwrap();
    std::fs::write(
        dir.join("root.cfg"),
        "@include child.cfg\nnumSearchThreads = 4\nnnMaxBatchSize = 8\n",
    )
    .unwrap();
    std::fs::write(dir.join("child.cfg"), "maxVisits = 100\n").unwrap();
    let spec = CommandSpec {
        program: dir.join("engine").to_string_lossy().into(),
        args: vec![
            "gtp".into(),
            "--model".into(),
            "model".into(),
            "--config".into(),
            "root.cfg".into(),
            "--override-config".into(),
            "maxVisits=100,logDir=private".into(),
        ],
        working_dir: Some(dir.to_string_lossy().into()),
        env: vec![],
    };
    let gpu = GpuSample {
        uuid: "GPU-test".into(),
        name: "Named GPU".into(),
        driver: "591.66".into(),
        physical_mib: 12227.0,
        used_mib: 1000.0,
    };
    let (before, baseline) = capture(&spec, MeasuredSceneDto::Live, &gpu, &|| false).unwrap();
    assert_eq!(baseline["numSearchThreads"], 4);
    assert_eq!(before["configIncludes"].as_array().unwrap().len(), 1);
    assert_eq!(before["commandSemantics"], json!({"maxVisits":"100"}));
    let modified = std::fs::metadata(dir.join("engine")).unwrap().modified().unwrap();
    std::fs::write(dir.join("engine"), b"engine-b").unwrap();
    std::fs::File::options()
        .write(true)
        .open(dir.join("engine"))
        .unwrap()
        .set_modified(modified)
        .unwrap();
    let (after, _) = capture(&spec, MeasuredSceneDto::Live, &gpu, &|| false).unwrap();
    assert_ne!(before["engineSha256"], after["engineSha256"]);
    assert!(capture(&spec, MeasuredSceneDto::Live, &gpu, &|| true).is_err());
    std::fs::write(dir.join("child.cfg"), "@include root.cfg\n").unwrap();
    assert!(capture(&spec, MeasuredSceneDto::Live, &gpu, &|| false).is_err());
    std::fs::remove_file(dir.join("child.cfg")).unwrap();
    assert!(capture(&spec, MeasuredSceneDto::Live, &gpu, &|| false).is_err());
    assert!(engine_manager::measured_report::fingerprint::hash_file(
        &dir.join("model"),
        Instant::now() - Duration::from_secs(1),
        &|| false
    )
    .is_err());
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn fingerprint_enforces_unique_file_and_byte_bounds() {
    let dir = std::env::temp_dir().join(format!("measured-bounds-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("engine"), b"engine").unwrap();
    std::fs::write(dir.join("model"), b"model").unwrap();
    let spec = CommandSpec {
        program: dir.join("engine").to_string_lossy().into(),
        args: vec![
            "gtp".into(),
            "-model".into(),
            "model".into(),
            "-config".into(),
            "root.cfg".into(),
        ],
        working_dir: Some(dir.to_string_lossy().into()),
        env: vec![],
    };
    let gpu = GpuSample {
        uuid: "GPU-test".into(),
        name: "GPU".into(),
        driver: "driver".into(),
        physical_mib: 12227.0,
        used_mib: 1.0,
    };
    let mut root = String::from("numSearchThreads=2\nnnMaxBatchSize=8\n");
    for index in 0..63 {
        let name = format!("include-{index}.cfg");
        std::fs::write(dir.join(&name), "# included\n").unwrap();
        root.push_str(&format!("@include {name}\n"));
    }
    std::fs::write(dir.join("root.cfg"), &root).unwrap();
    assert_eq!(
        capture(&spec, MeasuredSceneDto::Live, &gpu, &|| false).unwrap().0["configIncludes"]
            .as_array()
            .unwrap()
            .len(),
        63
    );
    std::fs::write(dir.join("extra.cfg"), "# extra\n").unwrap();
    root.push_str("@include extra.cfg\n");
    std::fs::write(dir.join("root.cfg"), root).unwrap();
    assert!(capture(&spec, MeasuredSceneDto::Live, &gpu, &|| false).is_err());
    let mut boundary = b"numSearchThreads=2\nnnMaxBatchSize=8\n#".to_vec();
    boundary.resize(4 * 1024 * 1024, b'x');
    std::fs::write(dir.join("root.cfg"), &boundary).unwrap();
    assert!(capture(&spec, MeasuredSceneDto::Live, &gpu, &|| false).is_ok());
    boundary.push(b'x');
    std::fs::write(dir.join("root.cfg"), boundary).unwrap();
    assert!(capture(&spec, MeasuredSceneDto::Live, &gpu, &|| false).is_err());
    for depth in 0..=33 {
        std::fs::write(
            dir.join(format!("depth-{depth}.cfg")),
            if depth == 33 {
                "numSearchThreads=2\nnnMaxBatchSize=8\n".into()
            } else {
                format!("@include depth-{}.cfg\n", depth + 1)
            },
        )
        .unwrap();
    }
    std::fs::write(dir.join("root.cfg"), "@include depth-0.cfg\n").unwrap();
    assert!(capture(&spec, MeasuredSceneDto::Live, &gpu, &|| false).is_err());
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn setup_retirement_fences_publication_until_worker_exit() {
    let manager = engine_manager::ForegroundEngineManager::new(
        std::sync::Arc::new(engine_manager::InMemoryEngineProfileCatalog::new()),
        engine_manager::ForegroundEngineConfig::for_tests(),
    );
    let lease = manager.begin_setup_operation().unwrap();
    assert!(manager.begin_setup_operation().is_err());
    manager.retire_setup_operation().unwrap();
    assert!(lease.check().is_err());
    assert!(lease.commit(|| Ok(())).is_err());
    assert!(manager.setup_operation_busy());
    assert!(manager.begin_setup_operation().is_err());
    drop(lease);
    assert!(!manager.setup_operation_busy());
    assert!(manager.begin_setup_operation().unwrap().check().is_ok());
}

#[test]
fn fingerprint_missing_config_is_rejected_without_panicking() {
    let spec = CommandSpec {
        program: "missing".into(),
        args: vec!["gtp".into(), "-model".into(), "missing".into()],
        working_dir: None,
        env: vec![],
    };
    let gpu = GpuSample {
        uuid: "GPU-test".into(),
        name: "GPU".into(),
        driver: "driver".into(),
        physical_mib: 1.0,
        used_mib: 0.0,
    };
    assert_eq!(
        capture(&spec, MeasuredSceneDto::Live, &gpu, &|| false).unwrap_err(),
        "report configuration is invalid"
    );
}

#[test]
fn scene_apply_restore_and_full_saves_preserve_manual_bytes_and_other_ids() {
    use sha2::{Digest, Sha256};
    let manager = engine_manager::ForegroundEngineManager::new(
        std::sync::Arc::new(engine_manager::InMemoryEngineProfileCatalog::new()),
        engine_manager::ForegroundEngineConfig::for_tests(),
    );
    let lease = manager.begin_setup_operation().unwrap();
    let mut settings = engine_manager::default_engine_profiles_settings();
    settings.profiles[0].profile.argv = vec![
        "-override-config".into(),
        "  numSearchThreads=3,nnMaxBatchSize=7,appleUseFP16=true  ".into(),
    ];
    let original = settings.profiles[0].profile.clone();
    let mut other = settings.profiles[0].clone();
    other.id = "unrelated".into();
    settings.profiles.push(other.clone());
    let mut review = app_model::MeasuredReviewDto {
        operation_id: "fixture".into(),
        generation: 1,
        target_id: settings.profiles[0].id.clone(),
        target_name: original.name.clone(),
        scene: MeasuredSceneDto::Live,
        profile_revision: format!("{:x}", Sha256::digest(serde_json::to_vec(&original).unwrap())),
        policy_revision: settings.profiles[0].thread_policy.revision,
        overlay_revision: 0,
        baseline_parameters: parsed(&valid(false, 3)).baseline,
        candidate_parameters: parsed(&valid(false, 3)).candidate,
        assessment: parsed(&valid(false, 3)).assess(),
        qualification_reasons: vec![],
        cold_verification_seconds: 0.0,
        report: valid(false, 3),
    };
    engine_manager::apply_measured_to_catalog(&mut settings, &review, &lease).unwrap();
    let live = settings.profiles[0].measured_scenes.live.clone();
    review.scene = MeasuredSceneDto::WholeGame;
    review.report = valid(true, 3);
    review.overlay_revision = 1;
    engine_manager::apply_measured_to_catalog(&mut settings, &review, &lease).unwrap();
    assert_eq!(settings.profiles[0].measured_scenes.live, live);
    assert!(settings.profiles[0].measured_scenes.whole_game.is_some());
    assert_eq!(settings.profiles[1], other);
    let mut full_save = settings.clone();
    full_save.profiles[0].measured_scenes = Default::default();
    let prepared = engine_manager::prepare_engine_profiles_save(&settings, full_save).unwrap();
    assert_eq!(
        prepared.profiles[0].measured_scenes,
        settings.profiles[0].measured_scenes
    );
    assert!(engine_manager::restore_measured_catalog(&mut settings, &review.target_id, 1, &lease).is_err());
    engine_manager::restore_measured_catalog(&mut settings, &review.target_id, 2, &lease).unwrap();
    assert!(settings.profiles[0].measured_scenes.live.is_none());
    assert!(settings.profiles[0].measured_scenes.whole_game.is_none());
    assert_eq!(settings.profiles[0].profile, original);
    assert_eq!(settings.profiles[1], other);
    manager.retire_setup_operation().unwrap();
    assert!(engine_manager::restore_measured_catalog(&mut settings, &review.target_id, 3, &lease).is_err());
}

#[test]
fn failed_atomic_catalog_replace_does_not_change_prior_file() {
    let dir = std::env::temp_dir().join(format!("measured-atomic-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("catalog.json");
    let settings = engine_manager::default_engine_profiles_settings();
    engine_manager::save_engine_profiles(&path, settings.clone()).unwrap();
    let before = std::fs::read(&path).unwrap();
    assert!(engine_manager::save_engine_profiles(&path.join("child"), settings).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), before);
    std::fs::remove_dir_all(dir).unwrap();
}

#[cfg(unix)]
#[test]
fn fingerprint_rejects_fifo_without_waiting_for_a_writer() {
    use std::os::unix::ffi::OsStrExt;
    let path = std::env::temp_dir().join(format!("measured-fifo-{}", uuid::Uuid::new_v4()));
    let name = std::ffi::CString::new(path.as_os_str().as_bytes()).unwrap();
    assert_eq!(unsafe { libc::mkfifo(name.as_ptr(), 0o600) }, 0);
    let result = engine_manager::measured_report::fingerprint::hash_file(
        &path,
        Instant::now() + Duration::from_secs(1),
        &|| false,
    );
    std::fs::remove_file(&path).unwrap();
    assert_eq!(result.unwrap_err(), "report resource must be a regular file");
}

#[cfg(unix)]
#[test]
fn hash_rejects_same_size_and_time_replacement_during_read() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    let directory = std::env::temp_dir().join(format!("measured-hash-race-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("model");
    let replacement = directory.join("replacement");
    std::fs::write(&path, vec![b'a'; 131072]).unwrap();
    std::fs::write(&replacement, vec![b'b'; 131072]).unwrap();
    let modified = std::fs::metadata(&path).unwrap().modified().unwrap();
    std::fs::OpenOptions::new()
        .write(true)
        .open(&replacement)
        .unwrap()
        .set_modified(modified)
        .unwrap();
    let chunks = AtomicUsize::new(0);
    let result = engine_manager::measured_report::fingerprint::hash_file(
        &path,
        Instant::now() + Duration::from_secs(5),
        &|| {
            if chunks.fetch_add(1, Ordering::Relaxed) == 1 {
                std::fs::rename(&replacement, &path).unwrap();
            }
            false
        },
    );
    std::fs::remove_dir_all(directory).unwrap();
    assert_eq!(result.unwrap_err(), "report resource changed during hashing");
}
