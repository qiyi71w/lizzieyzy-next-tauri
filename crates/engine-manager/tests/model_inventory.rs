use app_model::ModelInspectionStatusDto as Status;
use engine_manager::models::inspect_model;
use std::{fs, io::Write};

#[test]
fn content_header_not_filename_and_corrupt_gzip_are_distinct() {
    let dir = std::env::temp_dir().join(format!("model-identity-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("misleading-b11.bin.gz");
    // Real b20 sample header; the payload is deliberately only a header fixture.
    let bytes = b"g170-b20c256x2-s5303129600-d1228401921\n8\n22\n19\ntrunk\n20\n256\n256\n192\n64\n64\nconv1\n5\n5\n22\n256\n1\n1\n@BIN@\0\0\0\0";
    let mut gzip = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    gzip.write_all(bytes).unwrap();
    let encoded = gzip.finish().unwrap();
    fs::write(&path, &encoded).unwrap();
    let first = inspect_model(&path);
    assert_eq!(first.status, Status::HeaderRecognized);
    assert_eq!(
        first.model_name.as_deref(),
        Some("g170-b20c256x2-s5303129600-d1228401921")
    );
    assert_eq!(first.format_version, Some(8));
    assert_eq!(first.format.as_deref(), Some("katago_binary_gzip"));
    fs::write(&path, &encoded[..encoded.len() - 3]).unwrap();
    assert_eq!(inspect_model(&path).status, Status::Corrupt);
    fs::write(&path, b"not a model").unwrap();
    assert_eq!(inspect_model(&path).status, Status::Unknown);
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn retained_paths_survive_reopen_and_same_size_time_replacement_rejects_selection() {
    use app_model::{ModelOriginDto, ModelPathDto, ModelSelectionRequestDto};
    use engine_manager::models::ModelInventory;
    let dir = std::env::temp_dir().join(format!("model-retention-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&dir).unwrap();
    let b11 = dir.join("b11.bin");
    let b10 = dir.join("b10.bin");
    fs::write(&b11, b"real-b11\n8\n22\n19\ntrunk\n@BIN@1234").unwrap();
    fs::write(&b10, b"real-b10\n8\n22\n19\ntrunk\n@BIN@1234").unwrap();
    let model = |path: &std::path::Path| ModelPathDto {
        path: path.to_string_lossy().into_owned(),
        working_dir: None,
    };
    let inventory = ModelInventory::new(dir.join("inventory.json"));
    let first = inventory.refresh(&[model(&b11)]).unwrap();
    let digest = first.models[0].inspection.sha256.clone().unwrap();
    inventory.retain_managed(&b11, "b11-11750M", &digest).unwrap();
    let second = inventory.refresh(&[model(&b10)]).unwrap();
    assert_eq!(second.models.len(), 2);
    let original = &second.models[0];
    assert!(matches!(original.origin, ModelOriginDto::Managed { .. }));
    let request = ModelSelectionRequestDto {
        revision: second.revision.clone(),
        model_id: original.id.clone(),
        sha256: digest,
    };
    assert_eq!(inventory.select(&request).unwrap(), b11.to_string_lossy());
    let modified = fs::metadata(&b11).unwrap().modified().unwrap();
    fs::write(&b11, b"real-new\n8\n22\n19\ntrunk\n@BIN@1234").unwrap();
    fs::File::options()
        .write(true)
        .open(&b11)
        .unwrap()
        .set_times(fs::FileTimes::new().set_modified(modified))
        .unwrap();
    assert!(inventory.select(&request).is_err());
    drop(inventory);
    let reopened = ModelInventory::new(dir.join("inventory.json"));
    let snapshot = reopened.snapshot().unwrap();
    assert_eq!(snapshot.models.len(), 2);
    assert_eq!(snapshot.models[0].id, original.id);
    assert!(matches!(
        snapshot.models[0].origin,
        ModelOriginDto::Managed { .. }
    ));
    assert!(snapshot
        .models
        .iter()
        .all(|model| model.inspection.status == Status::Unchecked));
    assert!(reopened.select(&request).is_err());
    fs::remove_dir_all(dir).unwrap();
}

#[test]
#[ignore = "requires named local model samples; set LIZZIEYZY_MODEL_SAMPLES"]
fn named_real_model_headers() {
    let directory = std::path::PathBuf::from(std::env::var("LIZZIEYZY_MODEL_SAMPLES").unwrap());
    for (filename, name, version) in [
        (
            "kata1-b20c256x2-s5303129600-d1228401921.bin.gz",
            "g170-b20c256x2-s5303129600-d1228401921",
            8,
        ),
        (
            "kata1-tf3-b11c768-s11001M-d5973M.bin.gz",
            "b11c768h12nbt3tflrs-fson-silu",
            17,
        ),
    ] {
        let inspected = inspect_model(&directory.join(filename));
        println!(
            "sample={filename} inspection={}",
            serde_json::to_string(&inspected).unwrap()
        );
        assert_eq!(inspected.status, Status::HeaderRecognized);
        assert_eq!(inspected.model_name.as_deref(), Some(name));
        assert_eq!(inspected.format_version, Some(version));
    }
}

#[test]
#[ignore = "requires real engine/model/config; set LIZZIEYZY_KATAGO_ENGINE, MODEL, CONFIG, WORKDIR"]
fn real_retained_selection_launches_without_rewriting_profile_inputs() {
    use app_model::{
        EngineAdapterSettings, EngineProfileDto, ForegroundEngineEventDto, ForegroundEngineLifecycleDto,
        KataGoSettings, ModelPathDto, ModelSelectionRequestDto,
    };
    use engine_manager::{
        models::ModelInventory, ForegroundEngineConfig, ForegroundEngineManager,
        InMemoryEngineProfileCatalog, SavedEngineProfile,
    };
    use std::{
        sync::Arc,
        time::{Duration, Instant},
    };
    let env = |name: &str| std::env::var(format!("LIZZIEYZY_KATAGO_{name}")).unwrap();
    let dir = std::env::temp_dir().join(format!("model-real-{}", uuid::Uuid::new_v4()));
    let inventory = ModelInventory::new(dir.join("inventory.json"));
    let snapshot = inventory
        .refresh(&[ModelPathDto {
            path: env("MODEL"),
            working_dir: Some(env("WORKDIR")),
        }])
        .unwrap();
    let retained = &snapshot.models[0];
    let selected = inventory
        .select(&ModelSelectionRequestDto {
            revision: snapshot.revision.clone(),
            model_id: retained.id.clone(),
            sha256: retained.inspection.sha256.clone().unwrap(),
        })
        .unwrap();
    let profile = EngineProfileDto {
        name: "renamed local model".into(),
        program: env("ENGINE"),
        argv: vec!["-override-config".into(), "numSearchThreads=1".into()],
        working_dir: Some(env("WORKDIR")),
        adapter: EngineAdapterSettings::KataGoAnalysis(KataGoSettings {
            model_path: Some(selected),
            config_path: Some(env("CONFIG")),
            max_visits: 1,
        }),
    };
    let catalog = Arc::new(InMemoryEngineProfileCatalog::new());
    catalog.upsert(SavedEngineProfile {
        profile_id: "local-sample".into(),
        profile: profile.clone(),
    });
    let manager = ForegroundEngineManager::new(
        catalog,
        ForegroundEngineConfig {
            readiness_timeout: Duration::from_secs(300),
            ..Default::default()
        },
    );
    let events = manager.subscribe();
    manager.start("local-sample").unwrap();
    let deadline = Instant::now() + Duration::from_secs(310);
    loop {
        match events
            .recv_timeout(deadline.saturating_duration_since(Instant::now()))
            .unwrap()
        {
            ForegroundEngineEventDto::Snapshot { snapshot } => {
                if let ForegroundEngineLifecycleDto::Ready { run } = snapshot.lifecycle {
                    assert_eq!(run.profile_snapshot, profile);
                    println!(
                        "real selection SHA256={} run={}",
                        retained.inspection.sha256.as_ref().unwrap(),
                        serde_json::to_string(&run).unwrap()
                    );
                    break;
                }
            }
            ForegroundEngineEventDto::Failure { failure } => {
                panic!("real selection launch failed: {failure:?}")
            }
            _ => (),
        }
    }
    manager.teardown().unwrap();
    assert_eq!(
        ModelInventory::new(dir.join("inventory.json"))
            .snapshot()
            .unwrap()
            .models[0]
            .id,
        retained.id
    );
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn unknown_custom_paths_and_failed_refresh_keep_durable_entries_but_retire_actions() {
    use app_model::{ModelPathDto, ModelSelectionRequestDto};
    use engine_manager::models::ModelInventory;
    let dir = std::env::temp_dir().join(format!("model-failure-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&dir).unwrap();
    let unknown = dir.join("custom.onnx");
    fs::write(&unknown, b"unknown actual content").unwrap();
    let known = dir.join("known.bin");
    fs::write(&known, b"real-b11\n8\n22\n19\ntrunk\n@BIN@1234").unwrap();
    let path = dir.join("inventory.json");
    let store = ModelInventory::new(path.clone());
    let model_path = |path: &std::path::Path| ModelPathDto {
        path: path.to_string_lossy().into_owned(),
        working_dir: None,
    };
    let snapshot = store
        .refresh(&[model_path(&unknown), model_path(&known)])
        .unwrap();
    assert_eq!(snapshot.models[0].inspection.status, Status::Unknown);
    let model = &snapshot.models[1];
    let request = ModelSelectionRequestDto {
        revision: snapshot.revision.clone(),
        model_id: model.id.clone(),
        sha256: model.inspection.sha256.clone().unwrap(),
    };
    let original_bytes = fs::read(&path).unwrap();
    let excessive = (0..129)
        .map(|index| model_path(&dir.join(format!("extra-{index}.bin"))))
        .collect::<Vec<_>>();
    assert!(store.refresh(&excessive).is_err());
    assert_eq!(fs::read(&path).unwrap(), original_bytes);
    assert!(store.select(&request).is_err());
    fs::remove_file(&unknown).unwrap();
    let refreshed = store.refresh(&[]).unwrap();
    assert_eq!(refreshed.models.len(), 2);
    assert_eq!(refreshed.models[0].inspection.status, Status::Unavailable);
    fs::write(&path, b"corrupt inventory").unwrap();
    assert!(store.refresh(&[]).is_err());
    assert_eq!(fs::read(&path).unwrap(), b"corrupt inventory");
    fs::remove_dir_all(dir).unwrap();
}
