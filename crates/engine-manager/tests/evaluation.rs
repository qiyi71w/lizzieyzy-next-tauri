//! Production-build admission (no cfg(test) fixture trust) and opt-in real policy acceptance.
use app_model::*;
use engine_manager::*;
use std::{
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

struct DiskCatalog(PathBuf);
impl EngineProfileCatalog for DiskCatalog {
    fn get(&self, id: &str) -> Option<SavedEngineProfile> {
        load_engine_profiles(&self.0)
            .ok()?
            .profiles
            .into_iter()
            .find(|r| r.id == id)
            .map(|r| SavedEngineProfile {
                measured_scenes: Default::default(),
                profile_id: r.id,
                profile: r.profile,
                thread_policy: r.thread_policy,
            })
    }
    fn preload_profiles(&self) -> Vec<SavedEngineProfile> {
        load_engine_profiles(&self.0)
            .unwrap()
            .profiles
            .into_iter()
            .filter(|r| r.preload)
            .map(|r| SavedEngineProfile {
                measured_scenes: Default::default(),
                profile_id: r.id,
                profile: r.profile,
                thread_policy: r.thread_policy,
            })
            .collect()
    }
}
struct Owned(ForegroundEngineManager);
impl Drop for Owned {
    fn drop(&mut self) {
        let _ = self.0.teardown();
    }
}
fn wait_evaluation(manager: &ForegroundEngineManager) -> (EvaluationSnapshotDto, Vec<u32>) {
    let deadline = Instant::now() + Duration::from_secs(150);
    let mut pids = Vec::new();
    loop {
        let value = manager.evaluation_snapshot();
        if let Some(pid) = value.process_id {
            if !pids.contains(&pid) {
                pids.push(pid);
            }
        }
        if !matches!(
            value.phase,
            EvaluationPhaseDto::Qualifying | EvaluationPhaseDto::Running
        ) {
            return (value, pids);
        }
        assert!(Instant::now() < deadline, "{value:?}");
        std::thread::sleep(Duration::from_millis(5));
    }
}
fn wait_ready(manager: &ForegroundEngineManager) -> EngineRunDto {
    let deadline = Instant::now() + Duration::from_secs(90);
    loop {
        match manager.snapshot().lifecycle {
            ForegroundEngineLifecycleDto::Ready { run } => return run,
            ForegroundEngineLifecycleDto::Error { failure, .. }
            | ForegroundEngineLifecycleDto::NoEngine {
                failure: Some(failure),
            } => panic!("{failure:?}"),
            phase => assert!(Instant::now() < deadline, "{phase:?}"),
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}
fn read_threads(manager: &ForegroundEngineManager) -> RuntimeThreadsSnapshotDto {
    let state = manager.runtime_threads_snapshot();
    manager
        .runtime_threads(RuntimeThreadsRequestDto {
            identity: RuntimeControlIdentityDto {
                run_id: state.run_id.unwrap(),
                profile_revision: state.profile_revision.unwrap(),
                request_id: uuid::Uuid::new_v4().to_string(),
            },
            action: RuntimeThreadsActionDto::Read,
            value: None,
        })
        .unwrap()
}

#[test]
#[cfg(unix)]
fn unknown_fixture_and_wrapper_never_spawn_even_a_version_probe() {
    use std::os::unix::fs::PermissionsExt;
    let dir = std::env::temp_dir().join(format!("policy-admission-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("model.bin"), "model").unwrap();
    std::fs::write(dir.join("config.cfg"), "numSearchThreads=1\n").unwrap();
    let wrapper = dir.join("katago");
    std::fs::write(&wrapper, "#!/bin/sh\nprintf spawned > forbidden-spawn\nexit 0\n").unwrap();
    std::fs::set_permissions(&wrapper, std::fs::Permissions::from_mode(0o700)).unwrap();
    for program in [
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/evaluation_process.py"),
        wrapper,
    ] {
        let catalog = Arc::new(InMemoryEngineProfileCatalog::new());
        catalog.upsert(SavedEngineProfile {
            measured_scenes: Default::default(),
            profile_id: "target".into(),
            thread_policy: Default::default(),
            profile: EngineProfileDto {
                name: "unknown local-looking target".into(),
                program: program.to_string_lossy().into_owned(),
                argv: vec![],
                working_dir: Some(dir.to_string_lossy().into_owned()),
                adapter: EngineAdapterSettings::KataGoGtp(KataGoSettings {
                    model_path: Some("model.bin".into()),
                    config_path: Some("config.cfg".into()),
                    max_visits: 4,
                }),
            },
        });
        let owner = Owned(ForegroundEngineManager::new(
            catalog,
            ForegroundEngineConfig::for_tests(),
        ));
        owner.0.start_evaluation("target").unwrap();
        let (done, pids) = wait_evaluation(&owner.0);
        assert_eq!(done.phase, EvaluationPhaseDto::Failed);
        assert_eq!(done.message.as_deref(), Some("policy_direct_local_unknown"));
        assert!(pids.is_empty());
        assert!(
            !dir.join("spawned.json").exists(),
            "test-only byte trust must not enter production builds"
        );
        assert!(
            !dir.join("forbidden-spawn").exists(),
            "basename does not establish direct-local execution"
        );
    }
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn legacy_policy_defaults_without_eager_write_and_cfg_write_failure_preserves_catalog() {
    let dir = std::env::temp_dir().join(format!("policy-catalog-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("catalog.json");
    let mut json = serde_json::to_value(default_engine_profiles_settings()).unwrap();
    json["profiles"][0]
        .as_object_mut()
        .unwrap()
        .remove("thread_policy");
    let legacy = serde_json::to_vec(&json).unwrap();
    std::fs::write(&path, &legacy).unwrap();
    let current = load_engine_profiles(&path).unwrap();
    assert_eq!(current.profiles[0].thread_policy.strategy, ThreadStrategyDto::Cfg);
    assert_eq!(std::fs::read(&path).unwrap(), legacy);
    let owner = Owned(ForegroundEngineManager::new(
        Arc::new(DiskCatalog(path.clone())),
        ForegroundEngineConfig::for_tests(),
    ));
    let status = owner.0.thread_policy_status("default", None).unwrap();
    assert_eq!(status.validity, RecommendationValidityDto::None);
    let blocked = dir.join("blocked");
    std::fs::create_dir(&blocked).unwrap();
    let request = ThreadPolicySaveRequestDto {
        target_id: "default".into(),
        expected_revision: 0,
        input_revision: status.input_revision,
        strategy: ThreadStrategyDto::Cfg,
        evaluation_id: None,
    };
    assert!(owner
        .0
        .save_thread_policy_at_path(&blocked, current.clone(), request.clone())
        .is_err());
    assert_eq!(std::fs::read(&path).unwrap(), legacy);
    let saved = owner
        .0
        .save_thread_policy_at_path(&path, current, request)
        .unwrap();
    assert_eq!(saved.profiles[0].thread_policy.revision, 1);
    assert_eq!(load_engine_profiles(&path).unwrap(), saved);
    drop(owner);
    std::fs::remove_dir_all(dir).unwrap();
}

#[cfg(target_os = "linux")]
fn owned_pid(engine: &str) -> (u32, String) {
    let executable = std::fs::canonicalize(engine).unwrap();
    // Ready promotion precedes the existing two-second old-resident drain.
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let mut found = Vec::new();
        for task in std::fs::read_dir("/proc/self/task").unwrap() {
            if let Ok(children) = std::fs::read_to_string(task.unwrap().path().join("children")) {
                for pid in children.split_whitespace() {
                    if std::fs::read_link(format!("/proc/{pid}/exe")).ok().as_ref() == Some(&executable) {
                        let stat = std::fs::read_to_string(format!("/proc/{pid}/stat")).unwrap();
                        found.push((
                            pid.parse().unwrap(),
                            stat.rsplit_once(')')
                                .unwrap()
                                .1
                                .split_whitespace()
                                .nth(19)
                                .unwrap()
                                .to_owned(),
                        ));
                    }
                }
            }
        }
        found.sort();
        found.dedup();
        if found.len() == 1 {
            return found.pop().unwrap();
        }
        assert!(
            Instant::now() < deadline,
            "expected one manager-owned resident after drain, found {found:?}"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
#[cfg(target_os = "linux")]
#[ignore = "requires pinned real CPU engine/model and a private evidence directory; runs production manager"]
fn real_two_saved_ids_recommend_save_launch_and_effective_thread_readback() {
    let engine = std::env::var("LIZZIEYZY_KATAGO_ENGINE").unwrap();
    let model = std::env::var("LIZZIEYZY_KATAGO_MODEL").unwrap();
    let dir = PathBuf::from(std::env::var("LIZZIEYZY_POLICY_RUN_DIR").unwrap());
    std::fs::create_dir_all(&dir).unwrap();
    let cwd = dir.join(" cwd exact ");
    std::fs::create_dir(&cwd).unwrap();
    let config = b"numSearchThreads=1\nnumAnalysisThreads=1\nnumNNServerThreadsPerModel=1\nnnMaxBatchSize=1\nnnCacheSizePowerOfTwo=16\nnnMutexPoolSizePowerOfTwo=12\nmaxVisits=8\nlogToStderr=true\nlogSearchInfo=false\nlogAllGTPCommunication=false\nlogSearchInfoForChosenMove=false\nrules=tromp-taylor\nhomeDataDir=.\n";
    std::fs::write(cwd.join("engine.cfg"), config).unwrap();
    let mut settings = default_engine_profiles_settings();
    settings.profiles.clear();
    settings.selected_profile_id = "a".into();
    for id in ["a", "b"] {
        settings.profiles.push(EngineProfileRecord {
            measured_scenes: Default::default(),
            id: id.into(),
            preload: false,
            thread_policy: Default::default(),
            profile: EngineProfileDto {
                name: format!("Policy {id}"),
                program: engine.clone(),
                argv: vec![],
                working_dir: Some(cwd.to_string_lossy().into_owned()),
                adapter: EngineAdapterSettings::KataGoGtp(KataGoSettings {
                    model_path: Some(model.clone()),
                    config_path: Some("engine.cfg".into()),
                    max_visits: 8,
                }),
            },
        });
    }
    let originals = settings
        .profiles
        .iter()
        .map(|r| r.profile.clone())
        .collect::<Vec<_>>();
    let path = dir.join("catalog.json");
    save_engine_profiles(&path, settings).unwrap();
    let owner = Owned(ForegroundEngineManager::new(
        Arc::new(DiskCatalog(path.clone())),
        ForegroundEngineConfig::default(),
    ));
    let manager = &owner.0;
    manager.start("a").unwrap();
    let initial = wait_ready(manager);
    let initial_pid = owned_pid(&engine);
    assert_eq!(read_threads(manager).actual, Some(1));
    let mut evidence = Vec::new();
    for (index, id) in ["a", "b"].into_iter().enumerate() {
        let before_run = wait_ready(manager);
        let before_pid = owned_pid(&engine);
        let before_threads = read_threads(manager).actual;
        manager.start_evaluation(id).unwrap();
        let (done, evaluation_pids) = wait_evaluation(manager);
        assert_eq!(done.phase, EvaluationPhaseDto::Completed, "{done:?}");
        let result = done.result.as_ref().unwrap();
        let recommendation = result
            .recommendation
            .as_ref()
            .expect("actual compatible multirow upstream recommendation");
        assert_eq!(recommendation.target_id, id);
        assert!(result.measurements.len() >= 2);
        assert!(!evaluation_pids.is_empty());
        assert!(evaluation_pids.iter().all(|pid| *pid != before_pid.0));
        let status = manager
            .thread_policy_status(id, Some(&result.evaluation_id))
            .unwrap();
        assert_eq!(status.validity, RecommendationValidityDto::Valid, "{status:?}");
        let current = load_engine_profiles(&path).unwrap();
        let saved = manager
            .save_thread_policy_at_path(
                &path,
                current,
                ThreadPolicySaveRequestDto {
                    target_id: id.into(),
                    expected_revision: status.policy.revision,
                    input_revision: status.input_revision,
                    strategy: ThreadStrategyDto::Benchmark,
                    evaluation_id: Some(result.evaluation_id.clone()),
                },
            )
            .unwrap();
        assert_eq!(load_engine_profiles(&path).unwrap(), saved);
        assert_eq!(wait_ready(manager).run_id, before_run.run_id);
        assert_eq!(owned_pid(&engine), before_pid);
        assert_eq!(read_threads(manager).actual, before_threads);
        if index == 0 {
            assert_eq!(saved.profiles[1].thread_policy.strategy, ThreadStrategyDto::Cfg);
            // Separate processes model reopening from a different launcher without mutating
            // the test runner's process-global environment while manager threads are alive.
            for (key, value, expected) in [
                ("PR25_UNRELATED_LAUNCHER_SESSION", "session-b", "valid"),
                ("LD_LIBRARY_PATH", "/policy-smoke-loader-path", "changed"),
                ("LD_PRELOAD", "/policy-smoke-absent-injected-library", "unknown"),
            ] {
                let output = std::process::Command::new(std::env::current_exe().unwrap())
                    .args([
                        "real_saved_policy_environment_launch",
                        "--ignored",
                        "--exact",
                        "--nocapture",
                        "--test-threads=1",
                    ])
                    .env("LIZZIEYZY_POLICY_CATALOG", &path)
                    .env("LIZZIEYZY_POLICY_EXPECTED", expected)
                    .env(key, value)
                    .output()
                    .unwrap();
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                println!("POLICY_ENVIRONMENT_CASE {key} expected={expected}\n{stdout}\n{stderr}");
                assert!(output.status.success(), "{key}: {stdout}\n{stderr}");
            }
            manager.restart().unwrap();
        } else {
            manager.switch_to(id).unwrap();
        }
        let launched = wait_ready(manager);
        assert_eq!(launched.profile_id, id);
        assert_ne!(launched.run_id, before_run.run_id);
        let pid = owned_pid(&engine);
        assert_ne!(pid, before_pid);
        let observed = read_threads(manager);
        assert_eq!(observed.status, RuntimeThreadsStatusDto::Confirmed);
        assert_eq!(observed.actual, Some(recommendation.threads));
        assert_eq!(observed.run_id.as_deref(), Some(launched.run_id.as_str()));
        assert_eq!(launched.profile_snapshot, originals[index]);
        assert_eq!(std::fs::read(cwd.join("engine.cfg")).unwrap(), config);
        assert_eq!(
            load_engine_profiles(&path).unwrap().profiles[index].profile,
            originals[index]
        );
        evidence.push(serde_json::json!({ "target": id, "evaluation": done, "evaluation_pids": evaluation_pids, "pid": pid, "run": launched, "effective_readback": observed }));
    }
    let result = serde_json::json!({ "candidate": std::env::var("LIZZIEYZY_CANDIDATE_SHA").unwrap_or_default(), "initial_run": initial, "initial_pid": initial_pid, "entries": evidence, "catalog": load_engine_profiles(&path).unwrap(), "cwd": cwd });
    std::fs::write(
        dir.join("result.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
    println!("REAL_SAVED_POLICY {}", serde_json::to_string(&result).unwrap());
}

#[test]
#[cfg(target_os = "linux")]
#[ignore = "isolated subprocess of the real two-ID policy acceptance test"]
fn real_saved_policy_environment_launch() {
    let path = PathBuf::from(std::env::var("LIZZIEYZY_POLICY_CATALOG").unwrap());
    let expected = std::env::var("LIZZIEYZY_POLICY_EXPECTED").unwrap();
    let bytes = std::fs::read(&path).unwrap();
    let owner = Owned(ForegroundEngineManager::new(
        Arc::new(DiskCatalog(path.clone())),
        ForegroundEngineConfig::default(),
    ));
    let manager = &owner.0;
    let status = manager.thread_policy_status("a", None).unwrap();
    let validity = match expected.as_str() {
        "valid" => RecommendationValidityDto::Valid,
        "changed" => RecommendationValidityDto::Changed,
        "unknown" => RecommendationValidityDto::Unknown,
        _ => panic!("unexpected policy expectation"),
    };
    assert_eq!(status.validity, validity, "{status:?}");
    manager.start("a").unwrap();
    if validity == RecommendationValidityDto::Valid {
        let run = wait_ready(manager);
        let observed = read_threads(manager);
        assert_eq!(observed.status, RuntimeThreadsStatusDto::Confirmed);
        assert_eq!(
            observed.actual,
            Some(status.policy.recommendation.unwrap().threads)
        );
        assert_eq!(observed.run_id.as_deref(), Some(run.run_id.as_str()));
        let engine = std::env::var("LIZZIEYZY_KATAGO_ENGINE").unwrap();
        println!(
            "POLICY_ENVIRONMENT_LAUNCH run={} pid={:?} readback={observed:?}",
            run.run_id,
            owned_pid(&engine)
        );
    } else {
        let deadline = Instant::now() + Duration::from_secs(30);
        loop {
            match manager.snapshot().lifecycle {
                ForegroundEngineLifecycleDto::Error { failure, .. }
                | ForegroundEngineLifecycleDto::NoEngine {
                    failure: Some(failure),
                } => {
                    assert!(failure.message.contains(&status.reason));
                    println!("POLICY_ENVIRONMENT_REJECT validity={validity:?} failure={failure:?}");
                    break;
                }
                phase => {
                    assert!(!matches!(phase, ForegroundEngineLifecycleDto::Ready { .. }));
                    assert!(Instant::now() < deadline, "{phase:?}");
                }
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
}
