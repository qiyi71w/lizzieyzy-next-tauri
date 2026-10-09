use engine_manager::managed::{ManagedResources, catalog_snapshot};
use provider_core::network::NetworkState;
use app_model::{ManagedAcquireRequestDto, ManagedPhaseDto};

#[test]
fn explicit_operation_is_nonreentrant_and_cancel_has_one_terminal_without_installing() {
    let root = std::env::temp_dir().join(format!("managed-{}", uuid::Uuid::new_v4()));
    let resources = ManagedResources::new(root.clone(), root.join("trt"));
    let network = NetworkState::default();
    let profile = engine_manager::default_engine_profiles_settings().profiles[0].profile.clone();
    let request = ManagedAcquireRequestDto {
        profile_id: "default".into(), profile, target_id: if cfg!(windows) { "windows-cpu" } else { "linux-cpu" }.into(),
        model_id: "b11-flagship".into(), policy_revision: 0,
    };
    let id = resources.begin(request.clone(), &network).unwrap();
    assert!(resources.begin(request, &network).is_err());
    resources.cancel(&id).unwrap();
    assert!(resources.prepare(&id).is_err());
    let snapshot = resources.snapshot();
    assert_eq!(snapshot.operation.unwrap().phase, ManagedPhaseDto::Cancelled);
    assert!(!root.join("installed").exists());
    assert_eq!(catalog_snapshot().targets.len(), 15);
    if root.exists() { std::fs::remove_dir_all(root).unwrap(); }
}

#[test]
fn failed_operation_uses_canonical_bounded_diagnostics() {
    let root = std::env::temp_dir().join(format!("managed-{}", uuid::Uuid::new_v4()));
    let resources = ManagedResources::new(root.clone(), root.join("trt"));
    let request = ManagedAcquireRequestDto {
        profile_id: "default".into(),
        profile: engine_manager::default_engine_profiles_settings().profiles[0].profile.clone(),
        target_id: if cfg!(windows) { "windows-cpu" } else { "linux-cpu" }.into(),
        model_id: "b11-flagship".into(), policy_revision: 0,
    };
    let id = resources.begin(request, &NetworkState::default()).unwrap();
    resources.fail(&id, "token=private-token\nfailed at /home/private-user/resource\u{1b}");
    let operation = resources.snapshot().operation.unwrap();
    assert_eq!(operation.phase, ManagedPhaseDto::Failed);
    let message = operation.message.unwrap();
    assert!(!message.contains("private-token"));
    assert!(!message.contains("private-user"));
    assert!(!message.contains('\u{1b}'));
    assert!(message.chars().count() <= 256);
    assert!(!root.exists());
}

#[test]
#[ignore = "downloads frozen public CPU archive and 262 MB model into LIZZIEYZY_ACQUISITION_ROOT, then explicitly starts the real engine"]
fn real_public_acquisition_preserves_selection_and_qualifies_only_on_explicit_start() {
    use engine_manager::{models::ModelInventory, ForegroundEngineConfig, ForegroundEngineManager, InMemoryEngineProfileCatalog, SavedEngineProfile};
    use app_model::{EngineAdapterSettings, ForegroundEngineEventDto, ForegroundEngineLifecycleDto, KataGoSettings, ModelOriginDto, ModelPathDto};
    use std::{sync::Arc, time::{Duration, Instant}};
    let root = std::path::PathBuf::from(std::env::var("LIZZIEYZY_ACQUISITION_ROOT").unwrap());
    std::fs::create_dir_all(&root).unwrap();
    let resources = ManagedResources::new(root.join("resources"), root.join("trt"));
    let inventory = ModelInventory::new(root.join("inventory.json"));
    inventory.remember_saved(&[ModelPathDto { path: root.join("preserved-custom.bin.gz").to_string_lossy().into_owned(), working_dir: None }]).unwrap();
    let original = engine_manager::default_engine_profiles_settings();
    let profile_path = root.join("profiles.json");
    engine_manager::save_engine_profiles(&profile_path, original.clone()).unwrap();
    let saved_bytes = std::fs::read(&profile_path).unwrap();
    let network = NetworkState::default();
    let request = ManagedAcquireRequestDto {
        profile_id: "default".into(), profile: original.profiles[0].profile.clone(),
        target_id: if cfg!(windows) { "windows-cpu" } else { "linux-cpu" }.into(),
        model_id: "b11-flagship".into(), policy_revision: 0,
    };
    let catalog = Arc::new(InMemoryEngineProfileCatalog::new());
    let manager = ForegroundEngineManager::new(catalog.clone(), ForegroundEngineConfig {
        readiness_timeout: Duration::from_secs(300), managed_resources_root: Some(root.join("resources")), ..Default::default()
    });
    let before = manager.snapshot();
    let id = resources.begin(request.clone(), &network).unwrap();
    let prepared = resources.prepare(&id).unwrap();
    assert_eq!(std::fs::read(&profile_path).unwrap(), saved_bytes);
    let installed = resources.publish(prepared, &inventory).unwrap();
    assert_eq!(manager.snapshot(), before);
    assert_eq!(manager.last_primary_profile_id(), None);
    assert_eq!(std::fs::read(&profile_path).unwrap(), saved_bytes);
    let inventory_snapshot = inventory.snapshot().unwrap();
    assert!(inventory_snapshot.models.iter().any(|model| model.path.ends_with("preserved-custom.bin.gz")));
    assert!(inventory_snapshot.models.iter().any(|model| matches!(&model.origin, ModelOriginDto::Managed { catalog_id, installed_sha256 } if catalog_id == "b11-flagship" && installed_sha256 == "4a6312e80faadee7b7dd28689a2e87a1efb4640c10132f16290da7a17b4c6d9e")));
    println!("acquisition={}", serde_json::to_string(&resources.snapshot()).unwrap());
    let mut profile = original.profiles[0].profile.clone();
    profile.program = installed.program.clone();
    profile.working_dir = Some(root.to_string_lossy().into_owned());
    profile.argv = vec!["-override-config".into(), "numAnalysisThreads=1,numSearchThreadsPerAnalysisThread=1,nnMaxBatchSize=1,nnCacheSizePowerOfTwo=16,nnMutexPoolSizePowerOfTwo=10,logToStderr=true".into()];
    profile.adapter = EngineAdapterSettings::KataGoAnalysis(KataGoSettings { model_path: Some(installed.model_path.clone()), config_path: Some(installed.config_path.clone()), max_visits: 1 });
    catalog.upsert(SavedEngineProfile { profile_id: "default".into(), profile: profile.clone() });
    let events = manager.subscribe();
    manager.start("default").unwrap();
    let deadline = Instant::now() + Duration::from_secs(310);
    loop {
        match events.recv_timeout(deadline.saturating_duration_since(Instant::now())).unwrap() {
            ForegroundEngineEventDto::Snapshot { snapshot } => if let ForegroundEngineLifecycleDto::Ready { run } = snapshot.lifecycle {
                assert_eq!(run.profile_snapshot, profile);
                let qualified = run.qualified_resource.as_ref().unwrap();
                assert_eq!(qualified.origin, "project-source-build");
                assert_eq!(qualified.source_commit.as_deref(), Some("47aadc08518b3e121f22539796c911002f699584"));
                println!("qualified-run={}", serde_json::to_string(&run).unwrap());
                break;
            },
            ForegroundEngineEventDto::Failure { failure } => panic!("acquired engine failed qualification: {failure:?}"),
            _ => (),
        }
    }
    let ready = manager.snapshot();
    assert_eq!(manager.last_primary_profile_id().as_deref(), Some("default"));
    let cancel = resources.begin(request, &network).unwrap();
    resources.cancel(&cancel).unwrap();
    assert!(resources.prepare(&cancel).is_err());
    assert_eq!(manager.snapshot(), ready);
    assert!(std::path::Path::new(&installed.program).is_file());
    assert_eq!(std::fs::read(&profile_path).unwrap(), saved_bytes);
    let config_bytes = std::fs::read(&installed.config_path).unwrap();
    std::fs::write(&installed.config_path, b"numAnalysisThreads = 999\n").unwrap();
    catalog.upsert(SavedEngineProfile { profile_id: "altered".into(), profile });
    manager.switch_to("altered").unwrap();
    let deadline = Instant::now() + Duration::from_secs(90);
    loop {
        if let ForegroundEngineEventDto::Failure { failure } = events.recv_timeout(deadline.saturating_duration_since(Instant::now())).unwrap() {
            println!("modified-managed-switch={failure:?}");
            assert_eq!(failure.kind, app_model::EngineFailureKind::ResourceChanged);
            break;
        }
    }
    assert_eq!(manager.snapshot().lifecycle, ready.lifecycle);
    assert_eq!(manager.last_primary_profile_id().as_deref(), Some("default"));
    std::fs::write(&installed.config_path, config_bytes).unwrap();
    manager.teardown().unwrap();
    assert_eq!(manager.last_primary_profile_id().as_deref(), Some("default"));
    std::fs::write(root.join("installation.json"), serde_json::to_vec_pretty(&installed).unwrap()).unwrap();
}
