use app_model::{
    EngineBackend, EngineFailureKind, EngineOperationDto, EngineProfileDto, ForegroundEngineEventDto,
    ForegroundEngineLifecycleDto,
};
use engine_manager::{
    AnalysisCancelToken, AnalysisJobCancel, AnalysisJobLane, EngineProfileCatalog, ForegroundEngineConfig,
    ForegroundEngineManager, InMemoryEngineProfileCatalog, SavedEngineProfile,
};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::time::{Duration, Instant};

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
            .join("lizzieyzy-foreground-engine-run-tests")
            .join(format!("{label}-{unique}"));
        std::fs::create_dir_all(&path).unwrap();
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TestTempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

fn wait_snapshot(
    events: &Receiver<ForegroundEngineEventDto>,
    timeout: Duration,
    mut predicate: impl FnMut(&ForegroundEngineLifecycleDto) -> bool,
) -> app_model::ForegroundEngineSnapshotDto {
    let deadline = Instant::now() + timeout;
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        let event = events
            .recv_timeout(remaining)
            .expect("timed out waiting for foreground engine snapshot");
        if let ForegroundEngineEventDto::Snapshot { snapshot } = event {
            if predicate(&snapshot.lifecycle) {
                return snapshot;
            }
        }
    }
}

fn wait_failure(
    events: &Receiver<ForegroundEngineEventDto>,
    timeout: Duration,
    mut predicate: impl FnMut(&app_model::EngineFailureDto) -> bool,
) -> app_model::EngineFailureDto {
    let deadline = Instant::now() + timeout;
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        let event = events
            .recv_timeout(remaining)
            .expect("timed out waiting for foreground engine failure");
        if let ForegroundEngineEventDto::Failure { failure } = event {
            if predicate(&failure) {
                return failure;
            }
        }
    }
}

fn run_from_ready(lifecycle: &ForegroundEngineLifecycleDto) -> &app_model::EngineRunDto {
    match lifecycle {
        ForegroundEngineLifecycleDto::Ready { run } => run,
        other => panic!("expected Ready, got {other:?}"),
    }
}

fn lifecycle_run(lifecycle: &ForegroundEngineLifecycleDto) -> Option<&app_model::EngineRunDto> {
    match lifecycle {
        ForegroundEngineLifecycleDto::Starting { run }
        | ForegroundEngineLifecycleDto::Ready { run }
        | ForegroundEngineLifecycleDto::Stopping { run }
        | ForegroundEngineLifecycleDto::Error { run, .. } => Some(run),
        _ => None,
    }
}

#[test]
fn initial_snapshot_is_no_engine() {
    let catalog = Arc::new(InMemoryEngineProfileCatalog::new());
    let manager = ForegroundEngineManager::new(catalog, ForegroundEngineConfig::for_tests());
    let snapshot = manager.snapshot();
    assert_eq!(snapshot.revision, 0);
    assert!(matches!(
        snapshot.lifecycle,
        ForegroundEngineLifecycleDto::NoEngine
    ));
}

#[cfg(unix)]
fn write_executable(path: &Path, script: &str) {
    std::fs::write(path, format!("#!/bin/sh\n{script}\n")).unwrap();
    use std::os::unix::fs::PermissionsExt;
    let mut permissions = std::fs::metadata(path).unwrap().permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(path, permissions).unwrap();
}

#[cfg(unix)]
fn resident_echo_script() -> String {
    r#"
while IFS= read -r line; do
  if [ -n "$ENGINE_LOG" ]; then
    printf '%s\n' "$line" >> "$ENGINE_LOG"
  fi
  id=$(printf '%s' "$line" | sed -n 's/.*"id":"\([^"]*\)".*/\1/p')
  [ -z "$id" ] && id="ok"
  printf '{"id":"%s","turnNumber":0}\n' "$id"
  if [ "${EXIT_AFTER:-}" = "first" ]; then
    exit "${EXIT_CODE:-1}"
  fi
done
"#
    .into()
}

#[cfg(unix)]
fn setup_profile(temp: &TestTempDir, script: &str) -> EngineProfileDto {
    let engine_path = temp.path().join("fake-engine.sh");
    write_executable(&engine_path, script);
    let model_path = temp.path().join("model.bin");
    let config_path = temp.path().join("analysis.cfg");
    std::fs::write(&model_path, "").unwrap();
    std::fs::write(&config_path, "").unwrap();
    EngineProfileDto {
        name: "Fixture KataGo".into(),
        engine_path: engine_path.to_string_lossy().into_owned(),
        model_path: Some(model_path.to_string_lossy().into_owned()),
        config_path: Some(config_path.to_string_lossy().into_owned()),
        working_dir: Some(temp.path().to_string_lossy().into_owned()),
        backend: EngineBackend::KataGoAnalysis,
    }
}

#[cfg(unix)]
fn ready_manager(
    temp: &TestTempDir,
    script: &str,
) -> (
    ForegroundEngineManager,
    Arc<InMemoryEngineProfileCatalog>,
    Receiver<ForegroundEngineEventDto>,
    String,
) {
    let catalog = Arc::new(InMemoryEngineProfileCatalog::new());
    let profile = setup_profile(temp, script);
    catalog.upsert(SavedEngineProfile {
        profile_id: "profile-1".into(),
        profile,
    });
    let manager = ForegroundEngineManager::new(catalog.clone(), ForegroundEngineConfig::for_tests());
    let events = manager.subscribe();
    manager.start("profile-1").unwrap();
    let starting = wait_snapshot(&events, Duration::from_secs(2), |lifecycle| {
        matches!(lifecycle, ForegroundEngineLifecycleDto::Starting { .. })
    });
    assert!(lifecycle_run(&starting.lifecycle)
        .unwrap()
        .capability_snapshot
        .is_none());
    let ready = wait_snapshot(&events, Duration::from_secs(3), |lifecycle| {
        matches!(lifecycle, ForegroundEngineLifecycleDto::Ready { .. })
    });
    assert!(ready.revision > starting.revision);
    let run_id = run_from_ready(&ready.lifecycle).run_id.clone();
    (manager, catalog, events, run_id)
}

#[cfg(unix)]
#[test]
fn start_reaches_ready_only_after_jsonl_readiness_probe() {
    let temp = TestTempDir::new("ready-probe");
    let log = temp.path().join("engine.log");
    let mut script = format!("ENGINE_LOG='{}'\n", log.display());
    script.push_str(&resident_echo_script());
    let (manager, _, _, run_id) = ready_manager(&temp, &script);
    let snapshot = manager.snapshot();
    let run = run_from_ready(&snapshot.lifecycle);
    assert_eq!(run.run_id, run_id);
    assert_eq!(run.profile_id, "profile-1");
    assert_eq!(run.adapter_kind, EngineBackend::KataGoAnalysis);
    assert!(run.capability_snapshot.is_some());
    let logged = std::fs::read_to_string(&log).unwrap();
    assert!(logged.contains("lifecycle-readiness-"));
    assert!(logged.contains(&run_id));
}

#[cfg(unix)]
#[test]
fn start_failure_without_assets_stays_no_engine_with_typed_failure() {
    let catalog = Arc::new(InMemoryEngineProfileCatalog::new());
    catalog.upsert(SavedEngineProfile {
        profile_id: "missing".into(),
        profile: EngineProfileDto {
            name: "Missing".into(),
            engine_path: "/definitely/missing/katago".into(),
            model_path: Some("/definitely/missing/model.bin".into()),
            config_path: Some("/definitely/missing/analysis.cfg".into()),
            working_dir: None,
            backend: EngineBackend::KataGoAnalysis,
        },
    });
    let manager = ForegroundEngineManager::new(catalog, ForegroundEngineConfig::for_tests());
    let events = manager.subscribe();
    manager.start("missing").unwrap();
    wait_snapshot(&events, Duration::from_secs(2), |lifecycle| {
        matches!(lifecycle, ForegroundEngineLifecycleDto::Starting { .. })
    });
    let failure = wait_failure(&events, Duration::from_secs(2), |failure| {
        failure.kind == EngineFailureKind::Asset
    });
    assert_eq!(failure.operation, EngineOperationDto::Start);
    assert_eq!(failure.profile_id.as_deref(), Some("missing"));
    assert!(failure.run_id.is_some());
    assert!(matches!(
        manager.snapshot().lifecycle,
        ForegroundEngineLifecycleDto::NoEngine
    ));
}

#[cfg(unix)]
#[test]
fn spawn_without_probe_response_does_not_become_ready() {
    let temp = TestTempDir::new("no-probe");
    let catalog = Arc::new(InMemoryEngineProfileCatalog::new());
    catalog.upsert(SavedEngineProfile {
        profile_id: "profile-1".into(),
        profile: setup_profile(&temp, "sleep 2"),
    });
    let manager = ForegroundEngineManager::new(catalog, ForegroundEngineConfig::for_tests());
    let events = manager.subscribe();
    manager.start("profile-1").unwrap();
    wait_snapshot(&events, Duration::from_secs(2), |lifecycle| {
        matches!(lifecycle, ForegroundEngineLifecycleDto::Starting { .. })
    });
    let failure = wait_failure(&events, Duration::from_secs(4), |failure| {
        failure.kind == EngineFailureKind::Timeout
    });
    assert_eq!(failure.operation, EngineOperationDto::Start);
    assert_eq!(failure.profile_id.as_deref(), Some("profile-1"));
    assert!(failure.run_id.is_some());
    assert!(matches!(
        manager.snapshot().lifecycle,
        ForegroundEngineLifecycleDto::NoEngine
    ));
}

#[cfg(unix)]
#[test]
fn stop_cancels_run_owned_jobs_then_returns_no_engine() {
    let temp = TestTempDir::new("stop-cancel");
    let cancel_marker = temp.path().join("cancelled");
    let saw_cancel = temp.path().join("saw-cancel");
    let survived = temp.path().join("survived");
    let script = format!(
        "read line\nid=$(printf '%s' \"$line\" | sed -n 's/.*\"id\":\"\\([^\\\"]*\\)\".*/\\1/p')\nprintf '{{\"id\":\"%s\",\"turnNumber\":0}}\\n' \"$id\"\nwhile [ ! -f '{cancel}' ]; do sleep 0.01; done\nprintf seen > '{saw}'\nsleep 2\nprintf survived > '{survived}'\n",
        cancel = cancel_marker.display(),
        saw = saw_cancel.display(),
        survived = survived.display(),
    );
    let (manager, _, events, run_id) = ready_manager(&temp, &script);
    struct FileCancel(PathBuf);
    impl AnalysisJobCancel for FileCancel {
        fn cancel(&self) {
            std::fs::write(&self.0, "cancelled").unwrap();
        }
    }
    manager
        .register_job(
            &run_id,
            AnalysisJobLane::SelectedNode,
            Arc::new(FileCancel(cancel_marker.clone())),
        )
        .unwrap();
    manager.stop().unwrap();
    wait_snapshot(&events, Duration::from_secs(2), |lifecycle| {
        matches!(lifecycle, ForegroundEngineLifecycleDto::Stopping { .. })
    });
    wait_snapshot(&events, Duration::from_secs(3), |lifecycle| {
        matches!(lifecycle, ForegroundEngineLifecycleDto::NoEngine)
    });
    assert!(saw_cancel.exists());
    assert!(!survived.exists());
    manager
        .assert_profile_deletable("profile-1")
        .expect("Stop should release the profile delete guard");
}

#[cfg(unix)]
#[test]
fn restart_creates_new_run_identity_from_saved_record() {
    let temp = TestTempDir::new("restart");
    let (manager, catalog, events, old_run_id) = ready_manager(&temp, &resident_echo_script());
    let mut updated = catalog.get("profile-1").unwrap();
    updated.profile.name = "Updated KataGo".into();
    catalog.upsert(updated);
    assert_eq!(
        lifecycle_run(&manager.snapshot().lifecycle)
            .unwrap()
            .profile_snapshot
            .name,
        "Fixture KataGo"
    );
    manager
        .assert_profile_deletable("profile-1")
        .expect_err("Ready must block deleting the active profile");
    manager.restart().unwrap();
    wait_snapshot(&events, Duration::from_secs(2), |lifecycle| {
        matches!(lifecycle, ForegroundEngineLifecycleDto::Stopping { .. })
    });
    let ready = wait_snapshot(&events, Duration::from_secs(4), |lifecycle| {
        matches!(lifecycle, ForegroundEngineLifecycleDto::Ready { .. })
    });
    let run = run_from_ready(&ready.lifecycle);
    assert_ne!(run.run_id, old_run_id);
    assert_eq!(run.profile_id, "profile-1");
    assert_eq!(run.profile_snapshot.name, "Updated KataGo");
}

#[cfg(unix)]
#[test]
fn unexpected_exit_enters_error_and_does_not_autorestart() {
    let temp = TestTempDir::new("crash");
    let script = r#"
read line
id=$(printf '%s' "$line" | sed -n 's/.*"id":"\([^"]*\)".*/\1/p')
printf '{"id":"%s","turnNumber":0}\n' "$id"
exit 9
"#;
    let (manager, _, events, run_id) = ready_manager(&temp, script);
    let snapshot = wait_snapshot(&events, Duration::from_secs(3), |lifecycle| {
        matches!(lifecycle, ForegroundEngineLifecycleDto::Error { .. })
    });
    match snapshot.lifecycle {
        ForegroundEngineLifecycleDto::Error { run, failure } => {
            assert_eq!(run.run_id, run_id);
            assert!(run.capability_snapshot.is_some());
            assert_eq!(failure.kind, EngineFailureKind::NonzeroExit);
            assert_eq!(failure.operation, EngineOperationDto::UnexpectedExit);
            assert_eq!(failure.run_id.as_deref(), Some(run_id.as_str()));
            assert_eq!(failure.profile_id.as_deref(), Some("profile-1"));
            assert_ne!(failure.kind, EngineFailureKind::Cancellation);
        }
        other => panic!("expected Error, got {other:?}"),
    }
    std::thread::sleep(Duration::from_millis(200));
    assert!(matches!(
        manager.snapshot().lifecycle,
        ForegroundEngineLifecycleDto::Error { .. }
    ));
    manager
        .assert_profile_deletable("profile-1")
        .expect_err("Error still holding the run identity must block delete");
}

#[cfg(unix)]
#[test]
fn register_job_is_rejected_until_ready() {
    let catalog = Arc::new(InMemoryEngineProfileCatalog::new());
    let manager = ForegroundEngineManager::new(catalog, ForegroundEngineConfig::for_tests());
    let err = manager
        .register_job(
            "missing-run",
            AnalysisJobLane::WholeGame,
            Arc::new(AnalysisCancelToken::new()),
        )
        .unwrap_err();
    assert_eq!(err.kind, EngineFailureKind::InvalidState);
}

#[cfg(unix)]
#[test]
fn teardown_from_ready_reaches_no_engine_and_lifts_delete_guard() {
    let temp = TestTempDir::new("teardown");
    let (manager, _, _, _) = ready_manager(&temp, &resident_echo_script());
    manager
        .assert_profile_deletable("profile-1")
        .expect_err("Ready must block deleting the active profile");
    manager.teardown().unwrap();
    assert!(matches!(
        manager.snapshot().lifecycle,
        ForegroundEngineLifecycleDto::NoEngine
    ));
    manager
        .assert_profile_deletable("profile-1")
        .expect("teardown should release the profile delete guard");
}

#[cfg(unix)]
struct FileCancel(PathBuf);

#[cfg(unix)]
impl AnalysisJobCancel for FileCancel {
    fn cancel(&self) {
        std::fs::write(&self.0, "cancelled").unwrap();
    }
}

#[cfg(unix)]
#[test]
fn unsupported_backend_stays_no_engine_with_typed_failure() {
    let catalog = Arc::new(InMemoryEngineProfileCatalog::new());
    catalog.upsert(SavedEngineProfile {
        profile_id: "gtp".into(),
        profile: EngineProfileDto {
            name: "GTP".into(),
            engine_path: "/bin/katago".into(),
            model_path: None,
            config_path: None,
            working_dir: None,
            backend: EngineBackend::KataGoGtp,
        },
    });
    let manager = ForegroundEngineManager::new(catalog, ForegroundEngineConfig::for_tests());
    let failure = manager.start("gtp").unwrap_err();
    assert_eq!(failure.kind, EngineFailureKind::UnsupportedCapability);
    assert_eq!(failure.operation, EngineOperationDto::Start);
    assert_eq!(failure.profile_id.as_deref(), Some("gtp"));
    assert!(matches!(
        manager.snapshot().lifecycle,
        ForegroundEngineLifecycleDto::NoEngine
    ));
}

#[cfg(unix)]
#[test]
fn spawn_failure_stays_no_engine_with_start_kind() {
    let temp = TestTempDir::new("spawn-start");
    let catalog = Arc::new(InMemoryEngineProfileCatalog::new());
    let mut profile = setup_profile(&temp, &resident_echo_script());
    let not_binary = temp.path().join("not-a-binary");
    std::fs::create_dir_all(&not_binary).unwrap();
    profile.engine_path = not_binary.to_string_lossy().into_owned();
    catalog.upsert(SavedEngineProfile {
        profile_id: "profile-1".into(),
        profile,
    });
    let manager = ForegroundEngineManager::new(catalog, ForegroundEngineConfig::for_tests());
    let events = manager.subscribe();
    manager.start("profile-1").unwrap();
    let failure = wait_failure(&events, Duration::from_secs(2), |failure| {
        failure.kind == EngineFailureKind::Start
    });
    assert_eq!(failure.operation, EngineOperationDto::Start);
    assert_eq!(failure.profile_id.as_deref(), Some("profile-1"));
    assert!(failure.run_id.is_some());
    assert!(matches!(
        manager.snapshot().lifecycle,
        ForegroundEngineLifecycleDto::NoEngine
    ));
}

#[cfg(unix)]
#[test]
fn unparseable_readiness_probe_stays_no_engine_with_protocol_kind() {
    let temp = TestTempDir::new("protocol");
    let catalog = Arc::new(InMemoryEngineProfileCatalog::new());
    catalog.upsert(SavedEngineProfile {
        profile_id: "profile-1".into(),
        profile: setup_profile(
            &temp,
            r#"
while IFS= read -r line; do
  id=$(printf '%s' "$line" | sed -n 's/.*"id":"\([^"]*\)".*/\1/p')
  printf 'not-json %s\n' "$id"
done
"#,
        ),
    });
    let manager = ForegroundEngineManager::new(catalog, ForegroundEngineConfig::for_tests());
    let events = manager.subscribe();
    manager.start("profile-1").unwrap();
    let failure = wait_failure(&events, Duration::from_secs(3), |failure| {
        failure.kind == EngineFailureKind::Protocol
    });
    assert_eq!(failure.operation, EngineOperationDto::Start);
    assert_eq!(failure.profile_id.as_deref(), Some("profile-1"));
    assert!(failure.run_id.is_some());
    assert!(matches!(
        manager.snapshot().lifecycle,
        ForegroundEngineLifecycleDto::NoEngine
    ));
}

#[cfg(unix)]
#[test]
fn stdout_close_before_probe_stays_no_engine_with_readiness_kind() {
    let temp = TestTempDir::new("readiness");
    let catalog = Arc::new(InMemoryEngineProfileCatalog::new());
    catalog.upsert(SavedEngineProfile {
        profile_id: "profile-1".into(),
        profile: setup_profile(&temp, "exit 0"),
    });
    let manager = ForegroundEngineManager::new(catalog, ForegroundEngineConfig::for_tests());
    let events = manager.subscribe();
    manager.start("profile-1").unwrap();
    let failure = wait_failure(&events, Duration::from_secs(3), |failure| {
        failure.kind == EngineFailureKind::Readiness
    });
    assert_eq!(failure.operation, EngineOperationDto::Start);
    assert_eq!(failure.profile_id.as_deref(), Some("profile-1"));
    assert!(failure.run_id.is_some());
    assert!(matches!(
        manager.snapshot().lifecycle,
        ForegroundEngineLifecycleDto::NoEngine
    ));
    assert_ne!(failure.kind, EngineFailureKind::Cancellation);
}

#[cfg(unix)]
#[test]
fn autoload_asset_failure_stays_no_engine_without_falling_back() {
    let temp = TestTempDir::new("autoload-fail");
    let catalog = Arc::new(InMemoryEngineProfileCatalog::new());
    catalog.upsert(SavedEngineProfile {
        profile_id: "good".into(),
        profile: setup_profile(&temp, &resident_echo_script()),
    });
    catalog.upsert(SavedEngineProfile {
        profile_id: "bad".into(),
        profile: EngineProfileDto {
            name: "Broken".into(),
            engine_path: "/definitely/missing/katago".into(),
            model_path: Some("/definitely/missing/model.bin".into()),
            config_path: Some("/definitely/missing/analysis.cfg".into()),
            working_dir: None,
            backend: EngineBackend::KataGoAnalysis,
        },
    });
    let manager = ForegroundEngineManager::new(catalog, ForegroundEngineConfig::for_tests());
    let events = manager.subscribe();
    manager.autoload("bad").unwrap();
    let failure = wait_failure(&events, Duration::from_secs(2), |failure| {
        failure.kind == EngineFailureKind::Asset
    });
    assert_eq!(failure.operation, EngineOperationDto::Autoload);
    assert_eq!(failure.profile_id.as_deref(), Some("bad"));
    assert!(matches!(
        manager.snapshot().lifecycle,
        ForegroundEngineLifecycleDto::NoEngine
    ));
}

#[cfg(unix)]
#[test]
fn unexpected_exit_cancels_run_owned_jobs_before_error() {
    let temp = TestTempDir::new("crash-cancel");
    let cancel_marker = temp.path().join("cancelled");
    let script = r#"
read line
id=$(printf '%s' "$line" | sed -n 's/.*"id":"\([^"]*\)".*/\1/p')
printf '{"id":"%s","turnNumber":0}\n' "$id"
exit 9
"#;
    let (manager, _, events, run_id) = ready_manager(&temp, script);
    manager
        .register_job(
            &run_id,
            AnalysisJobLane::SelectedNode,
            Arc::new(FileCancel(cancel_marker.clone())),
        )
        .unwrap();
    let snapshot = wait_snapshot(&events, Duration::from_secs(3), |lifecycle| {
        matches!(lifecycle, ForegroundEngineLifecycleDto::Error { .. })
    });
    match snapshot.lifecycle {
        ForegroundEngineLifecycleDto::Error { run, failure } => {
            assert_eq!(run.run_id, run_id);
            assert_eq!(failure.kind, EngineFailureKind::NonzeroExit);
            assert_eq!(failure.operation, EngineOperationDto::UnexpectedExit);
            assert!(run.capability_snapshot.is_some());
        }
        other => panic!("expected Error, got {other:?}"),
    }
    assert!(cancel_marker.exists());
}

#[cfg(unix)]
#[test]
fn stop_does_not_publish_crash_failure() {
    let temp = TestTempDir::new("stop-not-crash");
    let (manager, _, events, _) = ready_manager(&temp, &resident_echo_script());
    manager.stop().unwrap();
    wait_snapshot(&events, Duration::from_secs(3), |lifecycle| {
        matches!(lifecycle, ForegroundEngineLifecycleDto::NoEngine)
    });
    let deadline = Instant::now() + Duration::from_millis(200);
    while let Ok(event) = events.recv_timeout(deadline.saturating_duration_since(Instant::now())) {
        if let ForegroundEngineEventDto::Failure { failure } = event {
            panic!("Stop must not publish a crash failure, got {failure:?}");
        }
    }
}

#[cfg(unix)]
#[test]
fn restart_after_error_uses_saved_record_and_new_run_identity() {
    let temp = TestTempDir::new("repair-restart");
    let script = r#"
read line
id=$(printf '%s' "$line" | sed -n 's/.*"id":"\([^"]*\)".*/\1/p')
printf '{"id":"%s","turnNumber":0}\n' "$id"
exit 9
"#;
    let (manager, catalog, events, old_run_id) = ready_manager(&temp, script);
    wait_snapshot(&events, Duration::from_secs(3), |lifecycle| {
        matches!(lifecycle, ForegroundEngineLifecycleDto::Error { .. })
    });
    let mut repaired = catalog.get("profile-1").unwrap();
    repaired.profile = setup_profile(&temp, &resident_echo_script());
    repaired.profile.name = "Repaired KataGo".into();
    catalog.upsert(repaired);
    manager.restart().unwrap();
    let ready = wait_snapshot(&events, Duration::from_secs(4), |lifecycle| {
        matches!(lifecycle, ForegroundEngineLifecycleDto::Ready { .. })
    });
    let run = run_from_ready(&ready.lifecycle);
    assert_ne!(run.run_id, old_run_id);
    assert_eq!(run.profile_snapshot.name, "Repaired KataGo");
}

#[cfg(unix)]
#[test]
fn failed_recovery_restart_stays_operable_and_does_not_fallback() {
    let temp = TestTempDir::new("failed-restart");
    let script = r#"
read line
id=$(printf '%s' "$line" | sed -n 's/.*"id":"\([^"]*\)".*/\1/p')
printf '{"id":"%s","turnNumber":0}\n' "$id"
exit 9
"#;
    let (manager, catalog, events, _) = ready_manager(&temp, script);
    wait_snapshot(&events, Duration::from_secs(3), |lifecycle| {
        matches!(lifecycle, ForegroundEngineLifecycleDto::Error { .. })
    });
    catalog.upsert(SavedEngineProfile {
        profile_id: "other".into(),
        profile: setup_profile(&temp, &resident_echo_script()),
    });
    let mut broken = catalog.get("profile-1").unwrap();
    broken.profile.engine_path = "/definitely/missing/katago".into();
    broken.profile.model_path = Some("/definitely/missing/model.bin".into());
    broken.profile.config_path = Some("/definitely/missing/analysis.cfg".into());
    catalog.upsert(broken);
    manager.restart().unwrap();
    let failure = wait_failure(&events, Duration::from_secs(4), |failure| {
        failure.kind == EngineFailureKind::Asset
    });
    assert_eq!(failure.profile_id.as_deref(), Some("profile-1"));
    assert!(matches!(
        manager.snapshot().lifecycle,
        ForegroundEngineLifecycleDto::NoEngine
    ));
    manager
        .assert_profile_deletable("profile-1")
        .expect("failed recovery Restart must leave an operable No-engine boundary");
    manager.start("other").unwrap();
    let ready = wait_snapshot(&events, Duration::from_secs(3), |lifecycle| {
        matches!(lifecycle, ForegroundEngineLifecycleDto::Ready { .. })
    });
    assert_eq!(run_from_ready(&ready.lifecycle).profile_id, "other");
}

#[cfg(unix)]
#[test]
fn restart_does_not_enter_error_from_the_replaced_process() {
    let temp = TestTempDir::new("stale-exit");
    let (manager, _, events, old_run_id) = ready_manager(&temp, &resident_echo_script());
    manager.restart().unwrap();
    let ready = wait_snapshot(&events, Duration::from_secs(4), |lifecycle| {
        matches!(lifecycle, ForegroundEngineLifecycleDto::Ready { .. })
    });
    let run = run_from_ready(&ready.lifecycle);
    assert_ne!(run.run_id, old_run_id);
    std::thread::sleep(Duration::from_millis(300));
    match manager.snapshot().lifecycle {
        ForegroundEngineLifecycleDto::Ready { run: current } => {
            assert_eq!(current.run_id, run.run_id);
        }
        other => panic!("replaced process exit must not overwrite the current Ready run, got {other:?}"),
    }
}

#[cfg(unix)]
#[test]
fn new_manager_does_not_restore_error_run_or_jobs() {
    let temp = TestTempDir::new("session-only");
    let script = r#"
read line
id=$(printf '%s' "$line" | sed -n 's/.*"id":"\([^"]*\)".*/\1/p')
printf '{"id":"%s","turnNumber":0}\n' "$id"
exit 9
"#;
    let (first, catalog, events, old_run_id) = ready_manager(&temp, script);
    wait_snapshot(&events, Duration::from_secs(3), |lifecycle| {
        matches!(lifecycle, ForegroundEngineLifecycleDto::Error { .. })
    });
    first
        .register_job(
            &old_run_id,
            AnalysisJobLane::SelectedNode,
            Arc::new(FileCancel(temp.path().join("unused"))),
        )
        .expect_err("Error must refuse new Analysis Job admission");
    first.teardown().unwrap();
    let second = ForegroundEngineManager::new(catalog, ForegroundEngineConfig::for_tests());
    assert!(matches!(
        second.snapshot().lifecycle,
        ForegroundEngineLifecycleDto::NoEngine
    ));
    second
        .register_job(
            &old_run_id,
            AnalysisJobLane::SelectedNode,
            Arc::new(AnalysisCancelToken::new()),
        )
        .expect_err("a new session must not restore the previous Error Run identity");
}
