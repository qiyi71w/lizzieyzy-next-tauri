use app_model::{
    admits_analysis_publication, AnalysisJobEventDto, AnalysisJobOutcomeDto, AnalysisPublicationScopeDto,
    EngineBackend, EngineCapabilitySnapshotDto, EngineFailureKind, EngineProfileDto,
    ForegroundEngineEventDto, ForegroundEngineLifecycleDto, NodePath,
};
use engine_manager::{
    AnalysisCancelToken, AnalysisJobCancel, AnalysisJobLane, EngineProfileCatalog, ForegroundEngineConfig,
    ForegroundEngineManager, InMemoryEngineProfileCatalog, SavedEngineProfile, SelectedNodeJobRequest,
};
use katago_protocol::AnalysisQuery;
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

fn wait_job(
    events: &Receiver<ForegroundEngineEventDto>,
    timeout: Duration,
    mut predicate: impl FnMut(&AnalysisJobEventDto) -> bool,
) -> AnalysisJobEventDto {
    let deadline = Instant::now() + timeout;
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        let event = events
            .recv_timeout(remaining)
            .expect("timed out waiting for analysis job event");
        if let ForegroundEngineEventDto::Job { job } = event {
            if predicate(&job) {
                return job;
            }
        }
    }
}

fn sample_query() -> AnalysisQuery {
    AnalysisQuery {
        id: "placeholder".into(),
        moves: Vec::new(),
        initial_stones: Vec::new(),
        rules: "chinese".into(),
        komi: 7.5,
        board_x_size: 9,
        board_y_size: 9,
        analyze_turns: Some(vec![0]),
        max_visits: Some(2),
        include_ownership: None,
        include_policy: None,
    }
}

fn selected_request(run_id: &str, generation: u64, indices: Vec<u32>) -> SelectedNodeJobRequest {
    SelectedNodeJobRequest {
        run_id: run_id.into(),
        generation,
        node_path: NodePath { indices },
        query: sample_query(),
        board_size: 9,
    }
}

#[cfg(unix)]
fn hold_after_probe_script() -> String {
    r#"
first=1
holding=0
while IFS= read -r line; do
  if [ -n "$ENGINE_LOG" ]; then
    printf '%s\n' "$line" >> "$ENGINE_LOG"
  fi
  id=$(printf '%s' "$line" | sed -n 's/.*"id":"\([^"]*\)".*/\1/p')
  [ -z "$id" ] && id="ok"
  if printf '%s' "$line" | grep -q '"action":"terminate"'; then
    holding=0
    continue
  fi
  if [ "$first" = 1 ]; then
    printf '{"id":"%s","turnNumber":0}\n' "$id"
    first=0
    continue
  fi
  holding=1
done
"#
    .into()
}

#[cfg(unix)]
fn hold_then_echo_script() -> String {
    r#"
mode=probe
pending=""
while IFS= read -r line; do
  if [ -n "$ENGINE_LOG" ]; then
    printf '%s\n' "$line" >> "$ENGINE_LOG"
  fi
  id=$(printf '%s' "$line" | sed -n 's/.*"id":"\([^"]*\)".*/\1/p')
  [ -z "$id" ] && id="ok"
  if printf '%s' "$line" | grep -q '"action":"terminate"'; then
    if [ -n "$pending" ] && [ "$pending" != "$id" ]; then
      printf '{"id":"%s","turnNumber":0,"rootInfo":{"visits":4,"winrate":0.6,"scoreMean":1.25}}\n' "$pending"
    fi
    pending=""
    mode=echo
    continue
  fi
  if [ "$mode" = probe ]; then
    printf '{"id":"%s","turnNumber":0}\n' "$id"
    mode=hold
    continue
  fi
  if [ "$mode" = hold ]; then
    pending="$id"
    continue
  fi
  printf '{"id":"%s","turnNumber":0,"rootInfo":{"visits":4,"winrate":0.6,"scoreMean":1.25}}\n' "$id"
done
"#
    .into()
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
    assert_eq!(failure.profile_id.as_deref(), Some("missing"));
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
    assert_eq!(failure.profile_id.as_deref(), Some("profile-1"));
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
            assert_eq!(failure.run_id.as_deref(), Some(run_id.as_str()));
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
#[test]
fn selected_node_start_completes_with_identity_and_keeps_ready() {
    let temp = TestTempDir::new("selected-complete");
    let (manager, _, events, run_id) = ready_manager(&temp, &resident_echo_script());
    let started = manager
        .start_selected_node_job(selected_request(&run_id, 3, vec![0, 1]))
        .unwrap();
    assert_eq!(started.run_id, run_id);
    assert_eq!(started.generation, 3);
    assert_eq!(started.node_path.indices, vec![0, 1]);
    let started_event = wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == started.job_id && job.outcome == AnalysisJobOutcomeDto::Started
    });
    let completed = wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == started.job_id && job.outcome == AnalysisJobOutcomeDto::Completed
    });
    assert_eq!(started_event.run_id, run_id);
    assert_eq!(completed.generation, 3);
    assert_eq!(completed.node_path.indices, vec![0, 1]);
    assert!(completed.frame.is_some());
    let current = AnalysisPublicationScopeDto {
        run_id: started.run_id.clone(),
        job_id: started.job_id.clone(),
        generation: started.generation,
        node_path: started.node_path.clone(),
    };
    assert!(admits_analysis_publication(&completed, &current));
    assert!(!admits_analysis_publication(
        &completed,
        &AnalysisPublicationScopeDto {
            generation: 4,
            ..current.clone()
        }
    ));
    assert!(!admits_analysis_publication(
        &completed,
        &AnalysisPublicationScopeDto {
            node_path: NodePath { indices: vec![2] },
            ..current
        }
    ));
    assert!(matches!(
        manager.snapshot().lifecycle,
        ForegroundEngineLifecycleDto::Ready { .. }
    ));
}

#[cfg(unix)]
#[test]
fn selected_node_explicit_cancel_writes_terminate_and_keeps_ready() {
    let temp = TestTempDir::new("selected-cancel");
    let log = temp.path().join("engine.log");
    let mut script = format!("ENGINE_LOG='{}'\n", log.display());
    script.push_str(&hold_after_probe_script());
    let (manager, _, events, run_id) = ready_manager(&temp, &script);
    let started = manager
        .start_selected_node_job(selected_request(&run_id, 1, vec![]))
        .unwrap();
    manager.cancel_job(&run_id, &started.job_id).unwrap();
    let cancelled = wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == started.job_id && job.outcome == AnalysisJobOutcomeDto::Cancelled
    });
    assert!(cancelled.frame.is_none());
    let deadline = Instant::now() + Duration::from_secs(1);
    let logged = loop {
        let logged = std::fs::read_to_string(&log).unwrap_or_default();
        if logged.contains(r#""action":"terminate"#) {
            break logged;
        }
        if Instant::now() >= deadline {
            panic!("engine log never recorded protocol terminate: {logged}");
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    assert!(logged.contains(&started.job_id), "{logged}");
    assert!(matches!(
        manager.snapshot().lifecycle,
        ForegroundEngineLifecycleDto::Ready { .. }
    ));
}

#[cfg(unix)]
#[test]
fn selected_node_supersede_cancels_only_that_lane_and_completes_latest() {
    let temp = TestTempDir::new("selected-supersede");
    let log = temp.path().join("engine.log");
    let cancel_marker = temp.path().join("whole-game-cancelled");
    let mut script = format!("ENGINE_LOG='{}'\n", log.display());
    script.push_str(&hold_then_echo_script());
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
            AnalysisJobLane::WholeGame,
            Arc::new(FileCancel(cancel_marker.clone())),
        )
        .unwrap();
    let first = manager
        .start_selected_node_job(selected_request(&run_id, 2, vec![0]))
        .unwrap();
    let second = manager
        .start_selected_node_job(selected_request(&run_id, 2, vec![1]))
        .unwrap();
    let superseded = wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == first.job_id && job.outcome == AnalysisJobOutcomeDto::Superseded
    });
    assert!(superseded.frame.is_none());
    let completed = wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == second.job_id && job.outcome == AnalysisJobOutcomeDto::Completed
    });
    assert!(completed.frame.is_some());
    assert!(!cancel_marker.exists());
    let mut saw_first_completed = false;
    let deadline = Instant::now() + Duration::from_millis(200);
    while Instant::now() < deadline {
        if let Ok(ForegroundEngineEventDto::Job { job }) = events.recv_timeout(Duration::from_millis(50)) {
            if job.job_id == first.job_id && job.outcome == AnalysisJobOutcomeDto::Completed {
                saw_first_completed = true;
            }
        }
    }
    assert!(!saw_first_completed);
    assert!(matches!(
        manager.snapshot().lifecycle,
        ForegroundEngineLifecycleDto::Ready { .. }
    ));
}

#[cfg(unix)]
#[test]
fn selected_node_timeout_is_terminal_and_keeps_ready() {
    let temp = TestTempDir::new("selected-timeout");
    let log = temp.path().join("engine.log");
    let mut script = format!("ENGINE_LOG='{}'\n", log.display());
    script.push_str(&hold_after_probe_script());
    let (manager, _, events, run_id) = ready_manager(&temp, &script);
    let started = manager
        .start_selected_node_job(selected_request(&run_id, 8, vec![0]))
        .unwrap();
    let timed_out = wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == started.job_id && job.outcome == AnalysisJobOutcomeDto::Timeout
    });
    assert!(timed_out.frame.is_none());
    let logged = std::fs::read_to_string(&log).unwrap();
    assert!(logged.contains(r#""action":"terminate"#), "{logged}");
    assert!(matches!(
        manager.snapshot().lifecycle,
        ForegroundEngineLifecycleDto::Ready { .. }
    ));
}

#[cfg(unix)]
#[test]
fn selected_node_rejects_stale_run_and_job_without_protocol_io() {
    let temp = TestTempDir::new("selected-stale");
    let log = temp.path().join("engine.log");
    let mut script = format!("ENGINE_LOG='{}'\n", log.display());
    script.push_str(&resident_echo_script());
    let (manager, _, _, run_id) = ready_manager(&temp, &script);
    let before = std::fs::read_to_string(&log).unwrap_or_default();
    let stale_run = manager
        .start_selected_node_job(selected_request("missing-run", 1, vec![]))
        .unwrap_err();
    assert_eq!(stale_run.kind, EngineFailureKind::InvalidState);
    let stale_cancel = manager.cancel_job(&run_id, "missing-job").unwrap_err();
    assert_eq!(stale_cancel.kind, EngineFailureKind::InvalidState);
    assert_eq!(stale_cancel.job_id.as_deref(), Some("missing-job"));
    let after = std::fs::read_to_string(&log).unwrap_or_default();
    assert_eq!(before, after);
}

#[cfg(unix)]
#[test]
fn selected_node_unsupported_capability_does_not_write_protocol() {
    let temp = TestTempDir::new("selected-unsupported");
    let log = temp.path().join("engine.log");
    let mut script = format!("ENGINE_LOG='{}'\n", log.display());
    script.push_str(&resident_echo_script());
    let (manager, _, _, run_id) = ready_manager(&temp, &script);
    let before = std::fs::read_to_string(&log).unwrap_or_default();
    manager.set_capability_snapshot_for_tests(EngineCapabilitySnapshotDto {
        adapter_kind: EngineBackend::KataGoAnalysis,
        selected_node_analysis: false,
        whole_game_analysis: true,
        protocol_cancel: true,
    });
    let error = manager
        .start_selected_node_job(selected_request(&run_id, 1, vec![]))
        .unwrap_err();
    assert_eq!(error.kind, EngineFailureKind::UnsupportedCapability);
    let after = std::fs::read_to_string(&log).unwrap_or_default();
    assert_eq!(before, after);
}

#[cfg(unix)]
#[test]
fn stop_and_restart_make_selected_node_job_terminal_without_late_completion() {
    let temp = TestTempDir::new("selected-stop");
    let log = temp.path().join("engine.log");
    let mut script = format!("ENGINE_LOG='{}'\n", log.display());
    script.push_str(&hold_after_probe_script());
    let (manager, _, events, run_id) = ready_manager(&temp, &script);
    let started = manager
        .start_selected_node_job(selected_request(&run_id, 1, vec![]))
        .unwrap();
    manager.stop().unwrap();
    wait_snapshot(&events, Duration::from_secs(3), |lifecycle| {
        matches!(lifecycle, ForegroundEngineLifecycleDto::NoEngine)
    });
    let mut saw_completed = false;
    let deadline = Instant::now() + Duration::from_millis(300);
    while Instant::now() < deadline {
        if let Ok(ForegroundEngineEventDto::Job { job }) = events.recv_timeout(Duration::from_millis(50)) {
            if job.job_id == started.job_id && job.outcome == AnalysisJobOutcomeDto::Completed {
                saw_completed = true;
            }
        }
    }
    assert!(!saw_completed);

    let temp_restart = TestTempDir::new("selected-restart");
    let (manager, _, events, run_id) = ready_manager(&temp_restart, &hold_after_probe_script());
    let started = manager
        .start_selected_node_job(selected_request(&run_id, 5, vec![0]))
        .unwrap();
    manager.restart().unwrap();
    wait_snapshot(
        &events,
        Duration::from_secs(4),
        |lifecycle| matches!(lifecycle, ForegroundEngineLifecycleDto::Ready { run } if run.run_id != run_id),
    );
    let mut saw_completed = false;
    let deadline = Instant::now() + Duration::from_millis(300);
    while Instant::now() < deadline {
        if let Ok(ForegroundEngineEventDto::Job { job }) = events.recv_timeout(Duration::from_millis(50)) {
            if job.job_id == started.job_id && job.outcome == AnalysisJobOutcomeDto::Completed {
                saw_completed = true;
            }
        }
    }
    assert!(!saw_completed);
}
