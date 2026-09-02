use app_model::{
    admits_analysis_publication, AnalysisJobEventDto, AnalysisJobOutcomeDto, AnalysisPublicationScopeDto,
    EngineBackend, EngineCapabilitySnapshotDto, EngineFailureKind, EngineOperationDto, EngineProfileDto,
    ForegroundEngineEventDto, ForegroundEngineLifecycleDto, NodePath,
};
use engine_manager::{
    AnalysisCancelToken, AnalysisJobCancel, AnalysisJobLane, EngineProfileCatalog, ForegroundEngineConfig,
    ForegroundEngineManager, InMemoryEngineProfileCatalog, SavedEngineProfile, SelectedNodeJobRequest,
    WholeGameJobRequest,
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

fn wait_lifecycle(
    manager: &ForegroundEngineManager,
    timeout: Duration,
    mut predicate: impl FnMut(&ForegroundEngineLifecycleDto) -> bool,
) -> app_model::ForegroundEngineSnapshotDto {
    let deadline = Instant::now() + timeout;
    loop {
        let snapshot = manager.snapshot();
        if predicate(&snapshot.lifecycle) {
            return snapshot;
        }
        if Instant::now() >= deadline {
            panic!(
                "timed out waiting for foreground engine snapshot, last={:?}",
                snapshot.lifecycle
            );
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn no_engine_failure(lifecycle: &ForegroundEngineLifecycleDto) -> Option<&app_model::EngineFailureDto> {
    match lifecycle {
        ForegroundEngineLifecycleDto::NoEngine { failure } => failure.as_ref(),
        _ => None,
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
        ForegroundEngineLifecycleDto::Switching { primary, .. } => Some(primary),
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

fn whole_game_request(run_id: &str, generation: u64, expected_responses: usize) -> WholeGameJobRequest {
    WholeGameJobRequest {
        run_id: run_id.into(),
        generation,
        node_path: NodePath { indices: Vec::new() },
        query_jsonl: whole_game_query_jsonl(),
        expected_responses,
    }
}

fn collect_job_events(
    events: &Receiver<ForegroundEngineEventDto>,
    until: Instant,
) -> Vec<AnalysisJobEventDto> {
    let mut jobs = Vec::new();
    while Instant::now() < until {
        let remaining = until
            .saturating_duration_since(Instant::now())
            .min(Duration::from_millis(50));
        match events.recv_timeout(remaining) {
            Ok(ForegroundEngineEventDto::Job { job }) => jobs.push(job),
            Ok(_) => {}
            Err(_) => {}
        }
    }
    jobs
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
        ForegroundEngineLifecycleDto::NoEngine { .. }
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
    wait_snapshot(&events, Duration::from_secs(2), |lifecycle| {
        no_engine_failure(lifecycle).is_some_and(|failure| failure.kind == EngineFailureKind::Asset)
    });
    let snapshot = manager.snapshot();
    let failure = no_engine_failure(&snapshot.lifecycle).expect("start miss must publish snapshot failure");
    assert_eq!(failure.operation, EngineOperationDto::Start);
    assert_eq!(failure.kind, EngineFailureKind::Asset);
    assert_eq!(failure.profile_id.as_deref(), Some("missing"));
    assert!(!failure.message.is_empty());
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
        ForegroundEngineLifecycleDto::NoEngine { .. }
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
        matches!(lifecycle, ForegroundEngineLifecycleDto::NoEngine { .. })
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
        ForegroundEngineLifecycleDto::NoEngine { .. }
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
    let deadline = Instant::now() + Duration::from_secs(1);
    let logged = loop {
        let logged = std::fs::read_to_string(&log).unwrap_or_default();
        if logged.contains(r#""action":"terminate""#) {
            break logged;
        }
        if Instant::now() >= deadline {
            panic!("engine log never recorded protocol terminate: {logged}");
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    assert!(logged.contains(r#""action":"terminate""#), "{logged}");
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
        matches!(lifecycle, ForegroundEngineLifecycleDto::NoEngine { .. })
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

#[cfg(unix)]
fn job_was_torn_down(lifecycle: &ForegroundEngineLifecycleDto, started_run_id: &str) -> bool {
    match lifecycle {
        ForegroundEngineLifecycleDto::NoEngine { .. } | ForegroundEngineLifecycleDto::Stopping { .. } => true,
        ForegroundEngineLifecycleDto::Ready { run } if run.run_id != started_run_id => true,
        ForegroundEngineLifecycleDto::Switching { primary, .. } if primary.run_id != started_run_id => true,
        _ => false,
    }
}

#[cfg(unix)]
fn timeout_must_not_follow_teardown(
    events: &Receiver<ForegroundEngineEventDto>,
    job_id: &str,
    started_run_id: &str,
    until: Instant,
) -> Vec<AnalysisJobOutcomeDto> {
    let mut outcomes = Vec::new();
    let mut torn_down = false;
    let mut timeout_after_teardown = false;
    while Instant::now() < until {
        let remaining = until
            .saturating_duration_since(Instant::now())
            .min(Duration::from_millis(50));
        match events.recv_timeout(remaining) {
            Ok(ForegroundEngineEventDto::Job { job }) if job.job_id == job_id => {
                if job.outcome == AnalysisJobOutcomeDto::Cancelled
                    || job.outcome == AnalysisJobOutcomeDto::Superseded
                {
                    torn_down = true;
                }
                if job.outcome == AnalysisJobOutcomeDto::Timeout && torn_down {
                    timeout_after_teardown = true;
                }
                outcomes.push(job.outcome);
            }
            Ok(ForegroundEngineEventDto::Snapshot { snapshot }) => {
                if job_was_torn_down(&snapshot.lifecycle, started_run_id) {
                    torn_down = true;
                }
            }
            Ok(_) => {}
            Err(_) => {}
        }
    }
    assert!(
        !timeout_after_teardown,
        "selected-node Timeout must not publish after Stop/Switch teardown: {outcomes:?}"
    );
    outcomes
}

#[cfg(unix)]
#[test]
fn stop_before_selected_node_timeout_never_publishes_timeout() {
    let temp = TestTempDir::new("selected-stop-before-timeout");
    let (manager, _, events, run_id) = ready_manager(&temp, &hold_after_probe_script());
    let started = manager
        .start_selected_node_job(selected_request(&run_id, 1, vec![]))
        .unwrap();
    manager.stop().unwrap();
    let outcomes = timeout_must_not_follow_teardown(
        &events,
        &started.job_id,
        &started.run_id,
        Instant::now() + Duration::from_millis(1200),
    );
    assert!(
        outcomes.contains(&AnalysisJobOutcomeDto::Cancelled),
        "Stop should publish Cancelled for the live selected-node job: {outcomes:?}"
    );
}

#[cfg(unix)]
#[test]
fn switch_before_selected_node_timeout_never_publishes_timeout() {
    let temp = TestTempDir::new("selected-switch-before-timeout");
    let (manager, _, events, run_id) =
        ready_two_profiles(&temp, &hold_after_probe_script(), &resident_echo_script());
    let started = manager
        .start_selected_node_job(selected_request(&run_id, 1, vec![]))
        .unwrap();
    manager.switch_to("profile-b").unwrap();
    timeout_must_not_follow_teardown(
        &events,
        &started.job_id,
        &started.run_id,
        Instant::now() + Duration::from_secs(4),
    );
    let ready = manager.snapshot().lifecycle;
    assert!(
        matches!(
            ready,
            ForegroundEngineLifecycleDto::Ready { ref run } if run.profile_id == "profile-b"
        ),
        "expected Ready B after switch: {ready:?}"
    );
}

fn whole_game_query_jsonl() -> String {
    r#"{"id":"caller","rules":"chinese","komi":7.5,"boardXSize":19,"boardYSize":19,"moves":[],"analyzeTurns":[0,1]}"#.into()
}

#[cfg(unix)]
fn resident_whole_game_script() -> String {
    r#"
while IFS= read -r line; do
  if [ -n "$ENGINE_LOG" ]; then
    printf '%s\n' "$line" >> "$ENGINE_LOG"
  fi
  printf '%s' "$line" | grep -q '"action":"terminate"' && continue
  id=$(printf '%s' "$line" | sed -n 's/.*"id":"\([^"]*\)".*/\1/p')
  [ -z "$id" ] && id="ok"
  case "$id" in
    lifecycle-readiness-*)
      printf '{"id":"%s","turnNumber":0}\n' "$id"
      ;;
    *)
      printf '{"id":"%s","turnNumber":0}\n' "$id"
      printf '{"id":"%s","turnNumber":1}\n' "$id"
      ;;
  esac
done
"#
    .into()
}

#[cfg(unix)]
#[test]
fn whole_game_job_completes_on_ready_run_without_spawning_another_process() {
    let temp = TestTempDir::new("whole-game-complete");
    let log = temp.path().join("engine.log");
    let mut script = format!("ENGINE_LOG='{}'\n", log.display());
    script.push_str(&resident_whole_game_script());
    let (manager, _, events, run_id) = ready_manager(&temp, &script);
    let started = manager
        .start_whole_game_analysis(whole_game_request(&run_id, 4, 2))
        .unwrap();
    assert_eq!(started.run_id, run_id);
    assert_eq!(started.lane, AnalysisJobLane::WholeGame);
    assert_eq!(started.generation, 4);
    assert_eq!(started.node_path.indices, Vec::<u32>::new());
    let job_id = started.job_id.clone();
    wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == job_id && job.outcome == AnalysisJobOutcomeDto::Started
    });
    let progress = wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == job_id && job.outcome == AnalysisJobOutcomeDto::Progress && job.completed == Some(1)
    });
    assert_eq!(progress.run_id, run_id);
    assert_eq!(progress.lane, AnalysisJobLane::WholeGame);
    assert_eq!(progress.generation, 4);
    assert_eq!(progress.node_path.indices, Vec::<u32>::new());
    assert_eq!(progress.expected, Some(2));
    assert!(progress.frame.is_none());
    let progress_done = wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == job_id && job.outcome == AnalysisJobOutcomeDto::Progress && job.completed == Some(2)
    });
    assert_eq!(progress_done.expected, Some(2));
    let completed = wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == job_id && job.outcome == AnalysisJobOutcomeDto::Completed
    });
    assert_eq!(completed.run_id, run_id);
    assert_eq!(completed.generation, 4);
    assert_eq!(completed.completed, None);
    assert_eq!(completed.expected, None);
    assert!(completed.frame.is_none());
    assert!(manager.snapshot().whole_game_job.is_none());
    assert!(matches!(
        manager.snapshot().lifecycle,
        ForegroundEngineLifecycleDto::Ready { .. }
    ));
    let logged = std::fs::read_to_string(&log).unwrap();
    assert!(logged.contains("lifecycle-readiness-"));
    assert!(logged.contains(&job_id));
    assert!(!logged.contains("\"id\":\"caller\""));
}

#[cfg(unix)]
#[test]
fn whole_game_user_cancel_keeps_run_ready_and_does_not_cancel_selected_node() {
    let temp = TestTempDir::new("whole-game-cancel");
    let log = temp.path().join("engine.log");
    let release = temp.path().join("release");
    let selected_cancelled = temp.path().join("selected-cancelled");
    let script = format!(
        r#"
ENGINE_LOG='{log}'
while IFS= read -r line; do
  printf '%s\n' "$line" >> "$ENGINE_LOG"
  printf '%s' "$line" | grep -q '"action":"terminate"' && continue
  id=$(printf '%s' "$line" | sed -n 's/.*"id":"\([^"]*\)".*/\1/p')
  [ -z "$id" ] && id="ok"
  case "$id" in
    lifecycle-readiness-*)
      printf '{{"id":"%s","turnNumber":0}}\n' "$id"
      ;;
    *)
      printf '{{"id":"%s","turnNumber":0}}\n' "$id"
      while [ ! -f '{release}' ]; do sleep 0.01; done
      printf '{{"id":"%s","turnNumber":1}}\n' "$id"
      ;;
  esac
done
"#,
        log = log.display(),
        release = release.display(),
    );
    let (manager, _, events, run_id) = ready_manager(&temp, &script);
    struct FileCancel(std::path::PathBuf);
    impl AnalysisJobCancel for FileCancel {
        fn cancel(&self) {
            std::fs::write(&self.0, "cancelled").unwrap();
        }
    }
    manager
        .register_job(
            &run_id,
            AnalysisJobLane::SelectedNode,
            Arc::new(FileCancel(selected_cancelled.clone())),
        )
        .unwrap();
    let started = manager
        .start_whole_game_analysis(whole_game_request(&run_id, 6, 2))
        .unwrap();
    wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == started.job_id
            && job.outcome == AnalysisJobOutcomeDto::Progress
            && job.completed == Some(1)
    });
    manager.cancel_job(&run_id, &started.job_id).unwrap();
    wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == started.job_id && job.outcome == AnalysisJobOutcomeDto::Cancelled
    });
    std::fs::write(&release, "go").unwrap();
    let later = collect_job_events(&events, Instant::now() + Duration::from_millis(200));
    assert!(later
        .iter()
        .all(|job| { job.job_id != started.job_id || job.outcome != AnalysisJobOutcomeDto::Completed }));
    assert!(!selected_cancelled.exists());
    let snapshot = manager.snapshot();
    assert!(snapshot.selected_node_job.is_some());
    assert!(snapshot.whole_game_job.is_none());
    assert!(matches!(
        snapshot.lifecycle,
        ForegroundEngineLifecycleDto::Ready { .. }
    ));
    let logged = std::fs::read_to_string(&log).unwrap();
    assert!(logged.contains("\"action\":\"terminate\""));
}

#[cfg(unix)]
#[test]
fn stop_cancels_whole_game_and_rejects_stale_completion() {
    let temp = TestTempDir::new("whole-game-stop");
    let release = temp.path().join("release");
    let script = format!(
        r#"
while IFS= read -r line; do
  printf '%s' "$line" | grep -q '"action":"terminate"' && continue
  id=$(printf '%s' "$line" | sed -n 's/.*"id":"\([^"]*\)".*/\1/p')
  [ -z "$id" ] && id="ok"
  case "$id" in
    lifecycle-readiness-*)
      printf '{{"id":"%s","turnNumber":0}}\n' "$id"
      ;;
    *)
      printf '{{"id":"%s","turnNumber":0}}\n' "$id"
      while [ ! -f '{release}' ]; do sleep 0.01; done
      printf '{{"id":"%s","turnNumber":1}}\n' "$id"
      ;;
  esac
done
"#,
        release = release.display(),
    );
    let (manager, _, events, run_id) = ready_manager(&temp, &script);
    let started = manager
        .start_whole_game_analysis(whole_game_request(&run_id, 7, 2))
        .unwrap();
    wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == started.job_id
            && job.outcome == AnalysisJobOutcomeDto::Progress
            && job.completed == Some(1)
    });
    manager.stop().unwrap();
    wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == started.job_id && job.outcome == AnalysisJobOutcomeDto::Cancelled
    });
    std::fs::write(&release, "go").unwrap();
    wait_snapshot(&events, Duration::from_secs(3), |lifecycle| {
        matches!(lifecycle, ForegroundEngineLifecycleDto::NoEngine { .. })
    });
    let later = collect_job_events(&events, Instant::now() + Duration::from_millis(200));
    assert!(later
        .iter()
        .all(|job| { job.job_id != started.job_id || job.outcome != AnalysisJobOutcomeDto::Completed }));
}

#[cfg(unix)]
#[test]
fn process_loss_cancels_whole_game_and_keeps_run_in_error() {
    let temp = TestTempDir::new("whole-game-crash");
    let crash = temp.path().join("crash");
    let script = format!(
        r#"
while IFS= read -r line; do
  id=$(printf '%s' "$line" | sed -n 's/.*"id":"\([^"]*\)".*/\1/p')
  [ -z "$id" ] && id="ok"
  case "$id" in
    lifecycle-readiness-*)
      printf '{{"id":"%s","turnNumber":0}}\n' "$id"
      ;;
    *)
      printf '{{"id":"%s","turnNumber":0}}\n' "$id"
      while [ ! -f '{crash}' ]; do sleep 0.01; done
      exit 9
      ;;
  esac
done
"#,
        crash = crash.display(),
    );
    let (manager, _, events, run_id) = ready_manager(&temp, &script);
    let started = manager
        .start_whole_game_analysis(whole_game_request(&run_id, 8, 2))
        .unwrap();
    wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == started.job_id
            && job.outcome == AnalysisJobOutcomeDto::Progress
            && job.completed == Some(1)
    });
    std::fs::write(&crash, "now").unwrap();
    let terminal = wait_job(&events, Duration::from_secs(3), |job| {
        job.job_id == started.job_id
            && matches!(
                job.outcome,
                AnalysisJobOutcomeDto::Cancelled | AnalysisJobOutcomeDto::Failed
            )
    });
    assert_ne!(terminal.outcome, AnalysisJobOutcomeDto::Completed);
    wait_snapshot(&events, Duration::from_secs(3), |lifecycle| {
        matches!(lifecycle, ForegroundEngineLifecycleDto::Error { .. })
    });
    std::thread::sleep(Duration::from_millis(200));
    assert!(matches!(
        manager.snapshot().lifecycle,
        ForegroundEngineLifecycleDto::Error { .. }
    ));
}

#[cfg(unix)]
#[test]
fn unsupported_whole_game_capability_is_rejected_before_protocol_io() {
    let temp = TestTempDir::new("whole-game-unsupported");
    let log = temp.path().join("engine.log");
    let mut script = format!("ENGINE_LOG='{}'\n", log.display());
    script.push_str(&resident_echo_script());
    let catalog = Arc::new(InMemoryEngineProfileCatalog::new());
    catalog.upsert(SavedEngineProfile {
        profile_id: "profile-1".into(),
        profile: setup_profile(&temp, &script),
    });
    let manager = ForegroundEngineManager::new(
        catalog,
        ForegroundEngineConfig {
            admit_whole_game_analysis: false,
            ..ForegroundEngineConfig::for_tests()
        },
    );
    let events = manager.subscribe();
    manager.start("profile-1").unwrap();
    let ready = wait_snapshot(&events, Duration::from_secs(3), |lifecycle| {
        matches!(lifecycle, ForegroundEngineLifecycleDto::Ready { .. })
    });
    let run_id = run_from_ready(&ready.lifecycle).run_id.clone();
    assert!(
        !run_from_ready(&ready.lifecycle)
            .capability_snapshot
            .as_ref()
            .unwrap()
            .whole_game_analysis
    );
    let logged_before = std::fs::read_to_string(&log).unwrap();
    let err = manager
        .start_whole_game_analysis(whole_game_request(&run_id, 9, 2))
        .unwrap_err();
    assert_eq!(err.kind, EngineFailureKind::UnsupportedCapability);
    assert_eq!(err.operation, app_model::EngineOperationDto::Job);
    assert_eq!(err.run_id.as_deref(), Some(run_id.as_str()));
    std::thread::sleep(Duration::from_millis(100));
    let logged_after = std::fs::read_to_string(&log).unwrap();
    assert_eq!(logged_before, logged_after);
}

#[cfg(unix)]
fn hold_both_lanes_manager(
    temp: &TestTempDir,
) -> (
    ForegroundEngineManager,
    Receiver<ForegroundEngineEventDto>,
    String,
    PathBuf,
) {
    let log = temp.path().join("engine.log");
    let mut script = format!("ENGINE_LOG='{}'\n", log.display());
    script.push_str(&hold_after_probe_script());
    let (manager, _, events, run_id) = ready_manager(temp, &script);
    (manager, events, run_id, log)
}

#[cfg(unix)]
fn start_both_lanes(
    manager: &ForegroundEngineManager,
    run_id: &str,
    generation: u64,
) -> (app_model::AnalysisJobStartedDto, app_model::AnalysisJobStartedDto) {
    let selected = manager
        .start_selected_node_job(selected_request(run_id, generation, vec![0]))
        .unwrap();
    let whole = manager
        .start_whole_game_analysis(whole_game_request(run_id, generation, 2))
        .unwrap();
    let snapshot = manager.snapshot();
    assert_eq!(
        snapshot.selected_node_job.as_ref().map(|job| job.job_id.as_str()),
        Some(selected.job_id.as_str())
    );
    assert_eq!(
        snapshot.whole_game_job.as_ref().map(|job| job.job_id.as_str()),
        Some(whole.job_id.as_str())
    );
    (selected, whole)
}

#[cfg(unix)]
#[test]
fn selected_node_and_whole_game_jobs_run_concurrently() {
    let temp = TestTempDir::new("lanes-concurrent");
    let (manager, _, run_id, _) = hold_both_lanes_manager(&temp);
    let (selected, whole) = start_both_lanes(&manager, &run_id, 11);
    let again = manager
        .start_selected_node_job(selected_request(&run_id, 12, vec![1]))
        .unwrap();
    let snapshot = manager.snapshot();
    assert_eq!(
        snapshot.selected_node_job.as_ref().map(|job| job.job_id.as_str()),
        Some(again.job_id.as_str())
    );
    assert_eq!(
        snapshot.whole_game_job.as_ref().map(|job| job.job_id.as_str()),
        Some(whole.job_id.as_str())
    );
    assert_ne!(again.job_id, selected.job_id);
    assert!(matches!(
        snapshot.lifecycle,
        ForegroundEngineLifecycleDto::Ready { .. }
    ));
}

#[cfg(unix)]
#[test]
fn second_whole_game_start_while_occupied_returns_occupied() {
    let temp = TestTempDir::new("lanes-occupied");
    let (manager, events, run_id, log) = hold_both_lanes_manager(&temp);
    let whole = manager
        .start_whole_game_analysis(whole_game_request(&run_id, 13, 2))
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let logged = std::fs::read_to_string(&log).unwrap_or_default();
        if logged.contains(&whole.job_id) {
            break;
        }
        if Instant::now() >= deadline {
            panic!("engine log never recorded the first whole-game query: {logged}");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    let selected = manager
        .start_selected_node_job(selected_request(&run_id, 13, vec![0]))
        .unwrap();
    let selected_deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let logged = std::fs::read_to_string(&log).unwrap_or_default();
        if logged.contains(&selected.job_id) {
            break;
        }
        if Instant::now() >= selected_deadline {
            panic!("engine log never recorded the selected-node query: {logged}");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    let logged_before = std::fs::read_to_string(&log).unwrap();
    let err = manager
        .start_whole_game_analysis(whole_game_request(&run_id, 99, 2))
        .unwrap_err();
    assert_eq!(err.kind, EngineFailureKind::Occupied);
    assert_eq!(err.operation, EngineOperationDto::Job);
    assert_eq!(err.run_id.as_deref(), Some(run_id.as_str()));
    assert_eq!(err.job_id.as_deref(), Some(whole.job_id.as_str()));
    assert!(!err.message.is_empty());
    std::thread::sleep(Duration::from_millis(50));
    let logged_after = std::fs::read_to_string(&log).unwrap();
    assert_eq!(logged_before, logged_after);
    let snapshot = manager.snapshot();
    assert_eq!(
        snapshot.selected_node_job.as_ref().map(|job| job.job_id.as_str()),
        Some(selected.job_id.as_str())
    );
    assert_eq!(
        snapshot.whole_game_job.as_ref().map(|job| job.job_id.as_str()),
        Some(whole.job_id.as_str())
    );
    let later = collect_job_events(&events, Instant::now() + Duration::from_millis(150));
    assert!(later.iter().all(|job| {
        job.job_id != selected.job_id
            || !matches!(
                job.outcome,
                AnalysisJobOutcomeDto::Cancelled | AnalysisJobOutcomeDto::Completed
            )
    }));
    assert!(later.iter().all(|job| {
        job.job_id != whole.job_id
            || !matches!(
                job.outcome,
                AnalysisJobOutcomeDto::Cancelled | AnalysisJobOutcomeDto::Completed
            )
    }));
}

#[cfg(unix)]
#[test]
fn cancel_whole_game_leaves_selected_node_job_running() {
    let temp = TestTempDir::new("cancel-whole-keeps-selected");
    let (manager, events, run_id, _) = hold_both_lanes_manager(&temp);
    let (selected, whole) = start_both_lanes(&manager, &run_id, 14);
    manager.cancel_job(&run_id, &whole.job_id).unwrap();
    wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == whole.job_id && job.outcome == AnalysisJobOutcomeDto::Cancelled
    });
    let snapshot = manager.snapshot();
    assert_eq!(
        snapshot.selected_node_job.as_ref().map(|job| job.job_id.as_str()),
        Some(selected.job_id.as_str())
    );
    assert!(snapshot.whole_game_job.is_none());
    let later = collect_job_events(&events, Instant::now() + Duration::from_millis(150));
    assert!(later.iter().all(|job| {
        job.job_id != selected.job_id
            || !matches!(
                job.outcome,
                AnalysisJobOutcomeDto::Cancelled | AnalysisJobOutcomeDto::Completed
            )
    }));
    assert!(matches!(
        snapshot.lifecycle,
        ForegroundEngineLifecycleDto::Ready { .. }
    ));
}

#[cfg(unix)]
#[test]
fn cancel_selected_node_leaves_whole_game_job_running() {
    let temp = TestTempDir::new("cancel-selected-keeps-whole");
    let (manager, events, run_id, _) = hold_both_lanes_manager(&temp);
    let (selected, whole) = start_both_lanes(&manager, &run_id, 15);
    manager.cancel_job(&run_id, &selected.job_id).unwrap();
    wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == selected.job_id && job.outcome == AnalysisJobOutcomeDto::Cancelled
    });
    let snapshot = manager.snapshot();
    assert_eq!(
        snapshot.whole_game_job.as_ref().map(|job| job.job_id.as_str()),
        Some(whole.job_id.as_str())
    );
    assert!(snapshot.selected_node_job.is_none());
    let later = collect_job_events(&events, Instant::now() + Duration::from_millis(150));
    assert!(later.iter().all(|job| {
        job.job_id != whole.job_id
            || !matches!(
                job.outcome,
                AnalysisJobOutcomeDto::Cancelled | AnalysisJobOutcomeDto::Completed
            )
    }));
}

#[cfg(unix)]
#[test]
fn selected_node_supersession_does_not_cancel_whole_game_job() {
    let temp = TestTempDir::new("supersede-keeps-whole");
    let (manager, events, run_id, _) = hold_both_lanes_manager(&temp);
    let whole = manager
        .start_whole_game_analysis(whole_game_request(&run_id, 16, 2))
        .unwrap();
    let first = manager
        .start_selected_node_job(selected_request(&run_id, 16, vec![0]))
        .unwrap();
    let second = manager
        .start_selected_node_job(selected_request(&run_id, 16, vec![1]))
        .unwrap();
    wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == first.job_id && job.outcome == AnalysisJobOutcomeDto::Superseded
    });
    let snapshot = manager.snapshot();
    assert_eq!(
        snapshot.whole_game_job.as_ref().map(|job| job.job_id.as_str()),
        Some(whole.job_id.as_str())
    );
    assert_eq!(
        snapshot.selected_node_job.as_ref().map(|job| job.job_id.as_str()),
        Some(second.job_id.as_str())
    );
    let later = collect_job_events(&events, Instant::now() + Duration::from_millis(150));
    assert!(later.iter().all(|job| {
        job.job_id != whole.job_id
            || !matches!(
                job.outcome,
                AnalysisJobOutcomeDto::Cancelled
                    | AnalysisJobOutcomeDto::Completed
                    | AnalysisJobOutcomeDto::Superseded
            )
    }));
}

#[cfg(unix)]
#[test]
fn stop_and_restart_cancel_both_analysis_job_lanes() {
    let temp = TestTempDir::new("stop-both-lanes");
    let (manager, events, run_id, _) = hold_both_lanes_manager(&temp);
    let (selected, whole) = start_both_lanes(&manager, &run_id, 17);
    manager.stop().unwrap();
    let jobs = collect_job_events(&events, Instant::now() + Duration::from_secs(2));
    assert!(
        jobs.iter()
            .any(|job| job.job_id == selected.job_id && job.outcome == AnalysisJobOutcomeDto::Cancelled),
        "Stop should cancel selected-node: {jobs:?}"
    );
    assert!(
        jobs.iter()
            .any(|job| job.job_id == whole.job_id && job.outcome == AnalysisJobOutcomeDto::Cancelled),
        "Stop should cancel whole-game: {jobs:?}"
    );
    assert!(jobs
        .iter()
        .all(|job| job.outcome != AnalysisJobOutcomeDto::Completed));
    wait_lifecycle(&manager, Duration::from_secs(3), |lifecycle| {
        matches!(lifecycle, ForegroundEngineLifecycleDto::NoEngine { .. })
    });

    let temp_restart = TestTempDir::new("restart-both-lanes");
    let (manager, events, run_id, _) = hold_both_lanes_manager(&temp_restart);
    let (selected, whole) = start_both_lanes(&manager, &run_id, 18);
    manager.restart().unwrap();
    let jobs = collect_job_events(&events, Instant::now() + Duration::from_secs(2));
    assert!(
        jobs.iter()
            .any(|job| job.job_id == selected.job_id && job.outcome == AnalysisJobOutcomeDto::Cancelled),
        "Restart should cancel selected-node: {jobs:?}"
    );
    assert!(
        jobs.iter()
            .any(|job| job.job_id == whole.job_id && job.outcome == AnalysisJobOutcomeDto::Cancelled),
        "Restart should cancel whole-game: {jobs:?}"
    );
    assert!(jobs
        .iter()
        .all(|job| job.outcome != AnalysisJobOutcomeDto::Completed));
    wait_lifecycle(
        &manager,
        Duration::from_secs(4),
        |lifecycle| matches!(lifecycle, ForegroundEngineLifecycleDto::Ready { run } if run.run_id != run_id),
    );
}

#[cfg(unix)]
#[test]
fn mismatched_job_identity_does_not_complete_the_other_lane() {
    let temp = TestTempDir::new("mismatched-lane-identity");
    let (manager, events, run_id, _) = hold_both_lanes_manager(&temp);
    let (selected, whole) = start_both_lanes(&manager, &run_id, 19);
    manager.cancel_job(&run_id, &selected.job_id).unwrap();
    wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == selected.job_id && job.outcome == AnalysisJobOutcomeDto::Cancelled
    });
    let snapshot = manager.snapshot();
    assert_eq!(
        snapshot.whole_game_job.as_ref().map(|job| job.job_id.as_str()),
        Some(whole.job_id.as_str())
    );
    assert_eq!(snapshot.whole_game_job.as_ref().unwrap().generation, 19);
    let stale = manager.cancel_job(&run_id, &selected.job_id).unwrap_err();
    assert_eq!(stale.kind, EngineFailureKind::InvalidState);
    let later = collect_job_events(&events, Instant::now() + Duration::from_millis(150));
    assert!(later.iter().all(|job| {
        job.job_id != whole.job_id
            || !matches!(
                job.outcome,
                AnalysisJobOutcomeDto::Cancelled | AnalysisJobOutcomeDto::Completed
            )
    }));
}

#[cfg(unix)]
#[cfg(unix)]
fn missing_assets_profile(name: &str, temp: &TestTempDir) -> EngineProfileDto {
    EngineProfileDto {
        name: name.into(),
        engine_path: format!("/definitely/missing/{name}-katago"),
        model_path: Some(format!("/definitely/missing/{name}-model.bin")),
        config_path: Some(format!("/definitely/missing/{name}.cfg")),
        working_dir: Some(temp.path().to_string_lossy().into_owned()),
        backend: EngineBackend::KataGoAnalysis,
    }
}

#[cfg(unix)]
fn spawn_fail_profile(temp: &TestTempDir, stem: &str) -> EngineProfileDto {
    let not_binary = temp.path().join(format!("{stem}-not-a-binary"));
    std::fs::create_dir_all(&not_binary).unwrap();
    let model_path = temp.path().join(format!("{stem}.bin"));
    let config_path = temp.path().join(format!("{stem}.cfg"));
    std::fs::write(&model_path, "").unwrap();
    std::fs::write(&config_path, "").unwrap();
    EngineProfileDto {
        name: stem.into(),
        engine_path: not_binary.to_string_lossy().into_owned(),
        model_path: Some(model_path.to_string_lossy().into_owned()),
        config_path: Some(config_path.to_string_lossy().into_owned()),
        working_dir: Some(temp.path().to_string_lossy().into_owned()),
        backend: EngineBackend::KataGoAnalysis,
    }
}

#[cfg(unix)]
fn exit_when_file_exists_script(kill_path: &Path) -> String {
    format!(
        r#"
killfile='{kill}'
( while [ ! -f "$killfile" ]; do sleep 0.02; done; kill $$ ) &
while IFS= read -r line; do
  id=$(printf '%s' "$line" | sed -n 's/.*"id":"\([^"]*\)".*/\1/p')
  [ -z "$id" ] && id="ok"
  printf '{{"id":"%s","turnNumber":0}}\n' "$id"
done
"#,
        kill = kill_path.display()
    )
}

#[cfg(unix)]
fn protocol_fail_script() -> String {
    r#"
while IFS= read -r line; do
  id=$(printf '%s' "$line" | sed -n 's/.*"id":"\([^"]*\)".*/\1/p')
  printf 'not-json %s\n' "$id"
done
"#
    .into()
}

#[cfg(unix)]
fn assert_ready_a(lifecycle: &ForegroundEngineLifecycleDto, run_a: &str, before: &app_model::EngineRunDto) {
    match lifecycle {
        ForegroundEngineLifecycleDto::Ready { run } => {
            assert_eq!(run.run_id, run_a);
            assert_eq!(run.profile_id, "profile-a");
            assert_eq!(run.profile_snapshot, before.profile_snapshot);
            assert_eq!(run.capability_snapshot, before.capability_snapshot);
        }
        other => panic!("expected Ready A, got {other:?}"),
    }
}

#[cfg(unix)]
fn setup_named_profile(temp: &TestTempDir, stem: &str, script: &str) -> EngineProfileDto {
    let engine_path = temp.path().join(format!("{stem}.sh"));
    write_executable(&engine_path, script);
    let model_path = temp.path().join(format!("{stem}.bin"));
    let config_path = temp.path().join(format!("{stem}.cfg"));
    std::fs::write(&model_path, "").unwrap();
    std::fs::write(&config_path, "").unwrap();
    EngineProfileDto {
        name: stem.into(),
        engine_path: engine_path.to_string_lossy().into_owned(),
        model_path: Some(model_path.to_string_lossy().into_owned()),
        config_path: Some(config_path.to_string_lossy().into_owned()),
        working_dir: Some(temp.path().to_string_lossy().into_owned()),
        backend: EngineBackend::KataGoAnalysis,
    }
}

#[cfg(unix)]
fn hold_probe_until_release_script(release_path: &Path) -> String {
    format!(
        r#"
release='{release}'
first=1
while IFS= read -r line; do
  id=$(printf '%s' "$line" | sed -n 's/.*"id":"\([^"]*\)".*/\1/p')
  [ -z "$id" ] && id="ok"
  if printf '%s' "$line" | grep -q '"action":"terminate"'; then
    continue
  fi
  if [ "$first" = 1 ]; then
    while [ ! -f "$release" ]; do
      sleep 0.02
    done
    first=0
    printf '{{"id":"%s","turnNumber":0}}\n' "$id"
    continue
  fi
  printf '{{"id":"%s","turnNumber":0}}\n' "$id"
  printf '{{"id":"%s","turnNumber":1}}\n' "$id"
done
"#,
        release = release_path.display()
    )
}

#[cfg(unix)]
fn ready_two_profiles(
    temp: &TestTempDir,
    profile_a_script: &str,
    profile_b_script: &str,
) -> (
    ForegroundEngineManager,
    Arc<InMemoryEngineProfileCatalog>,
    Receiver<ForegroundEngineEventDto>,
    String,
) {
    let catalog = Arc::new(InMemoryEngineProfileCatalog::new());
    catalog.upsert(SavedEngineProfile {
        profile_id: "profile-a".into(),
        profile: setup_named_profile(temp, "engine-a", profile_a_script),
    });
    catalog.upsert(SavedEngineProfile {
        profile_id: "profile-b".into(),
        profile: setup_named_profile(temp, "engine-b", profile_b_script),
    });
    let manager = ForegroundEngineManager::new(catalog.clone(), ForegroundEngineConfig::for_tests());
    let events = manager.subscribe();
    manager.start("profile-a").unwrap();
    let ready = wait_snapshot(
        &events,
        Duration::from_secs(3),
        |lifecycle| matches!(lifecycle, ForegroundEngineLifecycleDto::Ready { run } if run.profile_id == "profile-a"),
    );
    let run_id = run_from_ready(&ready.lifecycle).run_id.clone();
    (manager, catalog, events, run_id)
}

#[cfg(unix)]
#[test]
fn selecting_current_primary_does_not_switch_or_restart() {
    let temp = TestTempDir::new("switch-same");
    let (manager, _, events, run_id) =
        ready_two_profiles(&temp, &resident_echo_script(), &resident_echo_script());
    let before = manager.snapshot();
    manager.switch_to("profile-a").unwrap();
    assert_eq!(manager.snapshot().revision, before.revision);
    assert!(matches!(
        manager.snapshot().lifecycle,
        ForegroundEngineLifecycleDto::Ready { ref run } if run.run_id == run_id && run.profile_id == "profile-a"
    ));
    assert!(events.try_recv().is_err());
}

#[test]
fn apply_autoload_without_mark_stays_no_engine() {
    let catalog = Arc::new(InMemoryEngineProfileCatalog::new());
    catalog.upsert(SavedEngineProfile {
        profile_id: "profile-1".into(),
        profile: EngineProfileDto {
            name: "Unused".into(),
            engine_path: "/bin/unused".into(),
            model_path: None,
            config_path: None,
            working_dir: None,
            backend: EngineBackend::KataGoAnalysis,
        },
    });
    let manager = ForegroundEngineManager::new(catalog, ForegroundEngineConfig::for_tests());
    manager.apply_autoload().unwrap();
    assert!(matches!(
        manager.snapshot().lifecycle,
        ForegroundEngineLifecycleDto::NoEngine { .. }
    ));
}

#[cfg(unix)]
#[test]
fn apply_autoload_starts_only_the_marked_profile_with_a_new_run_identity() {
    let temp = TestTempDir::new("autoload-ready");
    let catalog = Arc::new(InMemoryEngineProfileCatalog::new());
    catalog.upsert(SavedEngineProfile {
        profile_id: "other".into(),
        profile: setup_profile(&temp, "exit 1"),
    });
    catalog.upsert(SavedEngineProfile {
        profile_id: "marked".into(),
        profile: setup_profile(&temp, &resident_echo_script()),
    });
    catalog.set_autoload_profile_id(Some("marked".into()));
    let manager = ForegroundEngineManager::new(catalog, ForegroundEngineConfig::for_tests());
    let events = manager.subscribe();
    manager.apply_autoload().unwrap();
    let ready = wait_snapshot(&events, Duration::from_secs(3), |lifecycle| {
        matches!(lifecycle, ForegroundEngineLifecycleDto::Ready { .. })
    });
    let run = run_from_ready(&ready.lifecycle);
    assert_eq!(run.profile_id, "marked");
    assert!(!run.run_id.is_empty());
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
    catalog.set_autoload_profile_id(Some("bad".into()));
    let manager = ForegroundEngineManager::new(catalog, ForegroundEngineConfig::for_tests());
    manager.apply_autoload().unwrap();
    let snapshot = wait_lifecycle(&manager, Duration::from_secs(2), |lifecycle| {
        no_engine_failure(lifecycle).is_some_and(|failure| failure.kind == EngineFailureKind::Asset)
    });
    let failure =
        no_engine_failure(&snapshot.lifecycle).expect("autoload miss must publish snapshot failure");
    assert_eq!(failure.operation, EngineOperationDto::Autoload);
    assert_eq!(failure.kind, EngineFailureKind::Asset);
    assert_eq!(failure.profile_id.as_deref(), Some("bad"));
    assert!(!failure.message.is_empty());
    let events = manager.subscribe();
    let after_subscribe = manager.snapshot();
    let persisted = no_engine_failure(&after_subscribe.lifecycle)
        .expect("subscribe-after-miss must read snapshot payload");
    assert_eq!(persisted.operation, EngineOperationDto::Autoload);
    assert_eq!(persisted.kind, EngineFailureKind::Asset);
    assert_eq!(persisted.message, failure.message);
    assert!(
        events.try_recv().is_err(),
        "banner must not depend on a post-subscribe failure event"
    );
}

#[cfg(unix)]
#[test]
fn successful_start_after_autoload_miss_then_stop_does_not_revive_failure() {
    let temp = TestTempDir::new("autoload-then-stop");
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
    catalog.set_autoload_profile_id(Some("bad".into()));
    let manager = ForegroundEngineManager::new(catalog, ForegroundEngineConfig::for_tests());
    manager.apply_autoload().unwrap();
    wait_lifecycle(&manager, Duration::from_secs(2), |lifecycle| {
        no_engine_failure(lifecycle).is_some_and(|failure| failure.operation == EngineOperationDto::Autoload)
    });
    manager.start("good").unwrap();
    wait_lifecycle(&manager, Duration::from_secs(3), |lifecycle| {
        matches!(lifecycle, ForegroundEngineLifecycleDto::Ready { .. })
    });
    assert!(no_engine_failure(&manager.snapshot().lifecycle).is_none());
    manager.stop().unwrap();
    let stopped = wait_lifecycle(&manager, Duration::from_secs(3), |lifecycle| {
        matches!(lifecycle, ForegroundEngineLifecycleDto::NoEngine { .. })
    });
    assert!(no_engine_failure(&stopped.lifecycle).is_none());
}

#[cfg(unix)]
#[test]
fn later_start_miss_replaces_autoload_snapshot_failure() {
    let _temp = TestTempDir::new("last-attempt-wins");
    let catalog = Arc::new(InMemoryEngineProfileCatalog::new());
    catalog.upsert(SavedEngineProfile {
        profile_id: "bad-autoload".into(),
        profile: EngineProfileDto {
            name: "Broken Autoload".into(),
            engine_path: "/definitely/missing/katago".into(),
            model_path: Some("/definitely/missing/model.bin".into()),
            config_path: Some("/definitely/missing/analysis.cfg".into()),
            working_dir: None,
            backend: EngineBackend::KataGoAnalysis,
        },
    });
    catalog.upsert(SavedEngineProfile {
        profile_id: "bad-start".into(),
        profile: EngineProfileDto {
            name: "Broken Start".into(),
            engine_path: "/definitely/missing/katago-2".into(),
            model_path: Some("/definitely/missing/model-2.bin".into()),
            config_path: Some("/definitely/missing/analysis-2.cfg".into()),
            working_dir: None,
            backend: EngineBackend::KataGoAnalysis,
        },
    });
    catalog.set_autoload_profile_id(Some("bad-autoload".into()));
    let manager = ForegroundEngineManager::new(catalog, ForegroundEngineConfig::for_tests());
    manager.apply_autoload().unwrap();
    wait_lifecycle(&manager, Duration::from_secs(2), |lifecycle| {
        no_engine_failure(lifecycle).is_some_and(|failure| failure.operation == EngineOperationDto::Autoload)
    });
    manager.start("bad-start").unwrap();
    let snapshot = wait_lifecycle(&manager, Duration::from_secs(2), |lifecycle| {
        no_engine_failure(lifecycle).is_some_and(|failure| failure.operation == EngineOperationDto::Start)
    });
    let failure = no_engine_failure(&snapshot.lifecycle).unwrap();
    assert_eq!(failure.kind, EngineFailureKind::Asset);
    assert_eq!(failure.profile_id.as_deref(), Some("bad-start"));
}

#[test]
fn start_unsupported_from_no_engine_publishes_snapshot_failure() {
    let catalog = Arc::new(InMemoryEngineProfileCatalog::new());
    catalog.upsert(SavedEngineProfile {
        profile_id: "gtp".into(),
        profile: EngineProfileDto {
            name: "GTP".into(),
            engine_path: "/bin/gtp".into(),
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
    let snapshot = manager.snapshot();
    let published =
        no_engine_failure(&snapshot.lifecycle).expect("sync start miss must publish snapshot failure");
    assert_eq!(published.kind, EngineFailureKind::UnsupportedCapability);
    assert_eq!(published.operation, EngineOperationDto::Start);
    assert_eq!(published.profile_id.as_deref(), Some("gtp"));
    assert_eq!(published.message, failure.message);
}

#[cfg(unix)]
#[test]
fn switch_keeps_primary_a_until_ready_b_promotes() {
    let temp = TestTempDir::new("switch-success");
    let release = temp.path().join("release-b");
    let (manager, _, events, run_a) = ready_two_profiles(
        &temp,
        &resident_echo_script(),
        &hold_probe_until_release_script(&release),
    );
    manager.assert_profile_deletable("profile-a").unwrap_err();
    manager.assert_profile_deletable("profile-b").unwrap();

    manager.switch_to("profile-b").unwrap();
    let switching = wait_snapshot(&events, Duration::from_secs(2), |lifecycle| {
        matches!(lifecycle, ForegroundEngineLifecycleDto::Switching { .. })
    });
    let ForegroundEngineLifecycleDto::Switching {
        primary,
        candidate,
        switch_id,
    } = switching.lifecycle
    else {
        panic!("expected switching snapshot");
    };
    assert_eq!(primary.run_id, run_a);
    assert_eq!(primary.profile_id, "profile-a");
    assert!(primary.capability_snapshot.is_some());
    assert_eq!(candidate.profile_id, "profile-b");
    assert_ne!(candidate.run_id, run_a);
    assert!(candidate.capability_snapshot.is_none());
    assert!(!switch_id.is_empty());
    manager.assert_profile_deletable("profile-a").unwrap_err();
    manager.assert_profile_deletable("profile-b").unwrap();

    let rejected_b = manager
        .start_selected_node_job(selected_request(&candidate.run_id, 9, vec![]))
        .unwrap_err();
    assert_eq!(rejected_b.kind, EngineFailureKind::InvalidState);

    let started_on_a = manager
        .start_selected_node_job(selected_request(&run_a, 3, vec![0]))
        .unwrap();
    assert_eq!(started_on_a.run_id, run_a);
    wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == started_on_a.job_id && job.outcome == AnalysisJobOutcomeDto::Completed
    });

    std::fs::write(&release, b"go").unwrap();
    let promoted = wait_snapshot(
        &events,
        Duration::from_secs(3),
        |lifecycle| matches!(lifecycle, ForegroundEngineLifecycleDto::Ready { run } if run.profile_id == "profile-b"),
    );
    let run_b = run_from_ready(&promoted.lifecycle);
    assert_eq!(run_b.profile_id, "profile-b");
    assert_ne!(run_b.run_id, run_a);
    assert!(run_b.capability_snapshot.is_some());
    manager.assert_profile_deletable("profile-a").unwrap();
    manager.assert_profile_deletable("profile-b").unwrap_err();

    let started_on_b = manager
        .start_selected_node_job(selected_request(&run_b.run_id, 4, vec![]))
        .unwrap();
    assert_eq!(started_on_b.run_id, run_b.run_id);
    wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == started_on_b.job_id && job.outcome == AnalysisJobOutcomeDto::Completed
    });
    let rejected = manager
        .start_selected_node_job(selected_request(&run_a, 5, vec![]))
        .unwrap_err();
    assert_eq!(rejected.kind, EngineFailureKind::InvalidState);
}

#[cfg(unix)]
#[test]
fn promotion_cancels_a_jobs_before_stopping_a_and_rejects_late_a_results() {
    let temp = TestTempDir::new("switch-cancel-a");
    let a_log = temp.path().join("engine-a.log");
    let mut a_script = format!("ENGINE_LOG='{}'\n", a_log.display());
    a_script.push_str(&hold_after_probe_script());
    let (manager, _, events, run_a) = ready_two_profiles(&temp, &a_script, &resident_echo_script());
    let started = manager
        .start_selected_node_job(selected_request(&run_a, 8, vec![]))
        .unwrap();
    manager.switch_to("profile-b").unwrap();
    let mut saw_cancel = false;
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        let event = events
            .recv_timeout(remaining)
            .expect("timed out waiting for promotion snapshot");
        match event {
            ForegroundEngineEventDto::Job { job }
                if job.job_id == started.job_id
                    && matches!(
                        job.outcome,
                        AnalysisJobOutcomeDto::Cancelled | AnalysisJobOutcomeDto::Superseded
                    ) =>
            {
                saw_cancel = true;
            }
            ForegroundEngineEventDto::Snapshot { ref snapshot }
                if matches!(
                    snapshot.lifecycle,
                    ForegroundEngineLifecycleDto::Ready { ref run } if run.profile_id == "profile-b"
                ) =>
            {
                assert!(
                    saw_cancel,
                    "A-owned jobs must be cancelled before the Ready B snapshot is observable"
                );
                break;
            }
            _ => {}
        }
    }
    let deadline = Instant::now() + Duration::from_secs(1);
    let _log = loop {
        let log = std::fs::read_to_string(&a_log).unwrap_or_default();
        if log.contains(r#""action":"terminate""#) {
            break log;
        }
        if Instant::now() >= deadline {
            panic!("A must receive protocol cancel before stop: {log}");
        }
        std::thread::sleep(Duration::from_millis(20));
    };
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

#[cfg(unix)]
#[test]
fn switch_rebinding_covers_whole_game_jobs() {
    let temp = TestTempDir::new("switch-whole-game");
    let release = temp.path().join("release-b");
    let (manager, _, events, run_a) = ready_two_profiles(
        &temp,
        &resident_whole_game_script(),
        &hold_probe_until_release_script(&release),
    );
    let started_a = manager
        .start_whole_game_analysis(whole_game_request(&run_a, 1, 2))
        .unwrap();
    wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == started_a.job_id && job.outcome == AnalysisJobOutcomeDto::Completed
    });
    manager.switch_to("profile-b").unwrap();
    wait_snapshot(&events, Duration::from_secs(2), |lifecycle| {
        matches!(lifecycle, ForegroundEngineLifecycleDto::Switching { .. })
    });
    std::fs::write(&release, b"go").unwrap();
    let promoted = wait_snapshot(
        &events,
        Duration::from_secs(3),
        |lifecycle| matches!(lifecycle, ForegroundEngineLifecycleDto::Ready { run } if run.profile_id == "profile-b"),
    );
    let run_b = run_from_ready(&promoted.lifecycle).run_id.clone();
    let started_b = manager
        .start_whole_game_analysis(whole_game_request(&run_b, 2, 2))
        .unwrap();
    wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == started_b.job_id && job.outcome == AnalysisJobOutcomeDto::Completed
    });
    let err = manager
        .start_whole_game_analysis(whole_game_request(&run_a, 3, 2))
        .unwrap_err();
    assert_eq!(err.kind, EngineFailureKind::InvalidState);
}

#[cfg(unix)]
#[test]
fn switch_asset_failure_keeps_ready_a_and_publishes_switch_scoped_failure() {
    let temp = TestTempDir::new("switch-asset");
    let (manager, catalog, events, run_a) =
        ready_two_profiles(&temp, &resident_echo_script(), &resident_echo_script());
    catalog.set_autoload_profile_id(Some("profile-a".into()));
    catalog.upsert(SavedEngineProfile {
        profile_id: "profile-b".into(),
        profile: EngineProfileDto {
            name: "Broken B".into(),
            engine_path: "/definitely/missing/katago-b".into(),
            model_path: Some("/definitely/missing/model-b.bin".into()),
            config_path: Some("/definitely/missing/b.cfg".into()),
            working_dir: Some(temp.path().to_string_lossy().into_owned()),
            backend: EngineBackend::KataGoAnalysis,
        },
    });
    let before = manager.snapshot();
    let before_run = run_from_ready(&before.lifecycle).clone();
    manager.switch_to("profile-b").unwrap();
    let switching = wait_snapshot(&events, Duration::from_secs(2), |lifecycle| {
        matches!(lifecycle, ForegroundEngineLifecycleDto::Switching { .. })
    });
    let ForegroundEngineLifecycleDto::Switching {
        primary,
        candidate,
        switch_id,
    } = switching.lifecycle
    else {
        panic!("expected switching snapshot");
    };
    assert_eq!(primary.run_id, run_a);
    assert_eq!(primary.profile_snapshot, before_run.profile_snapshot);
    assert_eq!(primary.capability_snapshot, before_run.capability_snapshot);
    let failure = wait_failure(&events, Duration::from_secs(2), |failure| {
        failure.kind == EngineFailureKind::Asset
    });
    assert_eq!(failure.operation, EngineOperationDto::Switch);
    assert_eq!(failure.profile_id.as_deref(), Some("profile-b"));
    assert_eq!(failure.run_id.as_deref(), Some(candidate.run_id.as_str()));
    assert_eq!(failure.switch_id.as_deref(), Some(switch_id.as_str()));
    let after_snapshot = manager.snapshot();
    let after = run_from_ready(&after_snapshot.lifecycle);
    assert_eq!(after.run_id, run_a);
    assert_eq!(after.profile_id, "profile-a");
    assert_eq!(after.profile_snapshot, before_run.profile_snapshot);
    assert_eq!(after.capability_snapshot, before_run.capability_snapshot);
    manager.assert_profile_deletable("profile-a").unwrap_err();
    manager.assert_profile_deletable("profile-b").unwrap();
    assert_eq!(catalog.autoload_profile_id().as_deref(), Some("profile-a"));
    let started = manager
        .start_selected_node_job(selected_request(&run_a, 11, vec![]))
        .unwrap();
    wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == started.job_id && job.outcome == AnalysisJobOutcomeDto::Completed
    });
}

#[cfg(unix)]
#[test]
fn switch_spawn_failure_keeps_ready_a_with_start_kind() {
    let temp = TestTempDir::new("switch-spawn");
    let (manager, catalog, events, run_a) =
        ready_two_profiles(&temp, &resident_echo_script(), &resident_echo_script());
    catalog.upsert(SavedEngineProfile {
        profile_id: "profile-b".into(),
        profile: spawn_fail_profile(&temp, "engine-b-spawn"),
    });
    let before_run = run_from_ready(&manager.snapshot().lifecycle).clone();
    manager.switch_to("profile-b").unwrap();
    let switching = wait_snapshot(&events, Duration::from_secs(2), |lifecycle| {
        matches!(lifecycle, ForegroundEngineLifecycleDto::Switching { .. })
    });
    let ForegroundEngineLifecycleDto::Switching {
        candidate, switch_id, ..
    } = switching.lifecycle
    else {
        panic!("expected switching snapshot");
    };
    let failure = wait_failure(&events, Duration::from_secs(2), |failure| {
        failure.kind == EngineFailureKind::Start
    });
    assert_eq!(failure.operation, EngineOperationDto::Switch);
    assert_eq!(failure.profile_id.as_deref(), Some("profile-b"));
    assert_eq!(failure.run_id.as_deref(), Some(candidate.run_id.as_str()));
    assert_eq!(failure.switch_id.as_deref(), Some(switch_id.as_str()));
    assert_ready_a(&manager.snapshot().lifecycle, &run_a, &before_run);
}

#[cfg(unix)]
#[test]
fn switch_protocol_failure_keeps_ready_a() {
    let temp = TestTempDir::new("switch-protocol");
    let (manager, catalog, events, run_a) =
        ready_two_profiles(&temp, &resident_echo_script(), &resident_echo_script());
    catalog.upsert(SavedEngineProfile {
        profile_id: "profile-b".into(),
        profile: setup_named_profile(&temp, "engine-b-protocol", &protocol_fail_script()),
    });
    let before_run = run_from_ready(&manager.snapshot().lifecycle).clone();
    manager.switch_to("profile-b").unwrap();
    let switching = wait_snapshot(&events, Duration::from_secs(2), |lifecycle| {
        matches!(lifecycle, ForegroundEngineLifecycleDto::Switching { .. })
    });
    let ForegroundEngineLifecycleDto::Switching {
        candidate, switch_id, ..
    } = switching.lifecycle
    else {
        panic!("expected switching snapshot");
    };
    let failure = wait_failure(&events, Duration::from_secs(3), |failure| {
        failure.kind == EngineFailureKind::Protocol
    });
    assert_eq!(failure.operation, EngineOperationDto::Switch);
    assert_eq!(failure.run_id.as_deref(), Some(candidate.run_id.as_str()));
    assert_eq!(failure.switch_id.as_deref(), Some(switch_id.as_str()));
    assert_ready_a(&manager.snapshot().lifecycle, &run_a, &before_run);
}

#[cfg(unix)]
#[test]
fn switch_readiness_exit_keeps_ready_a() {
    let temp = TestTempDir::new("switch-exit");
    let (manager, catalog, events, run_a) =
        ready_two_profiles(&temp, &resident_echo_script(), &resident_echo_script());
    catalog.upsert(SavedEngineProfile {
        profile_id: "profile-b".into(),
        profile: setup_named_profile(&temp, "engine-b-exit", "read line\nexit 7"),
    });
    let before_run = run_from_ready(&manager.snapshot().lifecycle).clone();
    manager.switch_to("profile-b").unwrap();
    let switching = wait_snapshot(&events, Duration::from_secs(2), |lifecycle| {
        matches!(lifecycle, ForegroundEngineLifecycleDto::Switching { .. })
    });
    let ForegroundEngineLifecycleDto::Switching {
        candidate, switch_id, ..
    } = switching.lifecycle
    else {
        panic!("expected switching snapshot");
    };
    let failure = wait_failure(&events, Duration::from_secs(3), |failure| {
        matches!(
            failure.kind,
            EngineFailureKind::Readiness | EngineFailureKind::NonzeroExit | EngineFailureKind::Start
        )
    });
    assert_eq!(failure.operation, EngineOperationDto::Switch);
    assert_eq!(failure.profile_id.as_deref(), Some("profile-b"));
    assert_eq!(failure.run_id.as_deref(), Some(candidate.run_id.as_str()));
    assert_eq!(failure.switch_id.as_deref(), Some(switch_id.as_str()));
    assert_ready_a(&manager.snapshot().lifecycle, &run_a, &before_run);
}

#[cfg(unix)]
#[test]
fn switch_timeout_keeps_ready_a() {
    let temp = TestTempDir::new("switch-timeout");
    let release = temp.path().join("never-release-b");
    let (manager, _, events, run_a) = ready_two_profiles(
        &temp,
        &resident_echo_script(),
        &hold_probe_until_release_script(&release),
    );
    let before_run = run_from_ready(&manager.snapshot().lifecycle).clone();
    manager.switch_to("profile-b").unwrap();
    let switching = wait_snapshot(&events, Duration::from_secs(2), |lifecycle| {
        matches!(lifecycle, ForegroundEngineLifecycleDto::Switching { .. })
    });
    let ForegroundEngineLifecycleDto::Switching {
        switch_id, candidate, ..
    } = switching.lifecycle
    else {
        panic!("expected switching snapshot");
    };
    let failure = wait_failure(&events, Duration::from_secs(4), |failure| {
        failure.kind == EngineFailureKind::Timeout
    });
    assert_eq!(failure.operation, EngineOperationDto::Switch);
    assert_eq!(failure.run_id.as_deref(), Some(candidate.run_id.as_str()));
    assert_eq!(failure.switch_id.as_deref(), Some(switch_id.as_str()));
    assert_ready_a(&manager.snapshot().lifecycle, &run_a, &before_run);
}

#[cfg(unix)]
#[test]
fn later_switch_to_c_wins_and_rejects_superseded_b() {
    let temp = TestTempDir::new("switch-b-then-c");
    let release_b = temp.path().join("release-b");
    let (manager, catalog, events, run_a) = ready_two_profiles(
        &temp,
        &resident_echo_script(),
        &hold_probe_until_release_script(&release_b),
    );
    catalog.upsert(SavedEngineProfile {
        profile_id: "profile-c".into(),
        profile: setup_named_profile(&temp, "engine-c", &resident_echo_script()),
    });
    manager.switch_to("profile-b").unwrap();
    let switching_b = wait_snapshot(
        &events,
        Duration::from_secs(2),
        |lifecycle| matches!(lifecycle, ForegroundEngineLifecycleDto::Switching { candidate, .. } if candidate.profile_id == "profile-b"),
    );
    let ForegroundEngineLifecycleDto::Switching {
        switch_id: switch_b,
        candidate: b,
        ..
    } = switching_b.lifecycle
    else {
        panic!("expected switching B");
    };
    manager.switch_to("profile-c").unwrap();
    let promoted = wait_snapshot(
        &events,
        Duration::from_secs(3),
        |lifecycle| matches!(lifecycle, ForegroundEngineLifecycleDto::Ready { run } if run.profile_id == "profile-c"),
    );
    let run_c = run_from_ready(&promoted.lifecycle).clone();
    std::fs::write(&release_b, b"go").unwrap();
    std::thread::sleep(Duration::from_millis(400));
    let current = manager.snapshot();
    match current.lifecycle {
        ForegroundEngineLifecycleDto::Ready { run } => {
            assert_eq!(run.run_id, run_c.run_id);
            assert_eq!(run.profile_id, "profile-c");
        }
        other => panic!("superseded B must not change primary C, got {other:?}"),
    }
    let mut late_b_failure = false;
    let deadline = Instant::now() + Duration::from_millis(300);
    while Instant::now() < deadline {
        if let Ok(ForegroundEngineEventDto::Failure { failure }) =
            events.recv_timeout(Duration::from_millis(50))
        {
            if failure.switch_id.as_deref() == Some(switch_b.as_str())
                || failure.run_id.as_deref() == Some(b.run_id.as_str())
            {
                late_b_failure = true;
            }
        }
    }
    assert!(!late_b_failure, "superseded B completion must stay private");
    manager
        .start_selected_node_job(selected_request(&run_a, 12, vec![]))
        .unwrap_err();
    let started = manager
        .start_selected_node_job(selected_request(&run_c.run_id, 13, vec![]))
        .unwrap();
    wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == started.job_id && job.outcome == AnalysisJobOutcomeDto::Completed
    });
}

#[cfg(unix)]
#[test]
fn stop_during_switch_then_b_failure_is_no_engine_without_promoting_b() {
    let temp = TestTempDir::new("switch-stop-a");
    let release_b = temp.path().join("release-b");
    let (manager, catalog, events, _run_a) = ready_two_profiles(
        &temp,
        &resident_echo_script(),
        &hold_probe_until_release_script(&release_b),
    );
    catalog.upsert(SavedEngineProfile {
        profile_id: "profile-b".into(),
        profile: setup_named_profile(
            &temp,
            "engine-b-stop",
            &hold_probe_until_release_script(&release_b),
        ),
    });
    manager.switch_to("profile-b").unwrap();
    wait_snapshot(&events, Duration::from_secs(2), |lifecycle| {
        matches!(lifecycle, ForegroundEngineLifecycleDto::Switching { .. })
    });
    manager.stop().unwrap();
    wait_snapshot(&events, Duration::from_secs(3), |lifecycle| {
        matches!(lifecycle, ForegroundEngineLifecycleDto::NoEngine { .. })
    });
    std::fs::write(&release_b, b"go").unwrap();
    std::thread::sleep(Duration::from_millis(400));
    assert!(matches!(
        manager.snapshot().lifecycle,
        ForegroundEngineLifecycleDto::NoEngine { .. }
    ));
}

#[cfg(unix)]
#[test]
fn primary_crash_during_switch_enters_error_and_cleans_candidate() {
    let temp = TestTempDir::new("switch-a-crash");
    let kill_a = temp.path().join("kill-a");
    let release_b = temp.path().join("release-b");
    let (manager, _, events, run_a) = ready_two_profiles(
        &temp,
        &exit_when_file_exists_script(&kill_a),
        &hold_probe_until_release_script(&release_b),
    );
    manager.switch_to("profile-b").unwrap();
    wait_snapshot(&events, Duration::from_secs(2), |lifecycle| {
        matches!(lifecycle, ForegroundEngineLifecycleDto::Switching { .. })
    });
    std::fs::write(&kill_a, b"die").unwrap();
    let snapshot = wait_snapshot(&events, Duration::from_secs(3), |lifecycle| {
        matches!(lifecycle, ForegroundEngineLifecycleDto::Error { .. })
    });
    match snapshot.lifecycle {
        ForegroundEngineLifecycleDto::Error { run, failure } => {
            assert_eq!(run.run_id, run_a);
            assert_eq!(run.profile_id, "profile-a");
            assert!(run.capability_snapshot.is_some());
            assert_eq!(failure.kind, EngineFailureKind::NonzeroExit);
            assert_eq!(failure.operation, EngineOperationDto::UnexpectedExit);
            assert_eq!(failure.run_id.as_deref(), Some(run_a.as_str()));
        }
        other => panic!("expected Error from A crash, got {other:?}"),
    }
    std::fs::write(&release_b, b"go").unwrap();
    std::thread::sleep(Duration::from_millis(400));
    match manager.snapshot().lifecycle {
        ForegroundEngineLifecycleDto::Error { run, .. } => {
            assert_eq!(run.run_id, run_a);
        }
        other => panic!("B must not promote after A crash, got {other:?}"),
    }
    manager.assert_profile_deletable("profile-a").unwrap_err();
}

#[cfg(unix)]
#[test]
fn stale_a_job_after_failed_switch_does_not_bind_to_b() {
    let temp = TestTempDir::new("switch-stale-job");
    let (manager, catalog, events, run_a) =
        ready_two_profiles(&temp, &resident_echo_script(), &resident_echo_script());
    catalog.upsert(SavedEngineProfile {
        profile_id: "profile-b".into(),
        profile: missing_assets_profile("Broken B", &temp),
    });
    manager.switch_to("profile-b").unwrap();
    let switching = wait_snapshot(&events, Duration::from_secs(2), |lifecycle| {
        matches!(lifecycle, ForegroundEngineLifecycleDto::Switching { .. })
    });
    let ForegroundEngineLifecycleDto::Switching { candidate, .. } = switching.lifecycle else {
        panic!("expected switching");
    };
    wait_failure(&events, Duration::from_secs(2), |failure| {
        failure.kind == EngineFailureKind::Asset
    });
    manager
        .start_selected_node_job(selected_request(&candidate.run_id, 14, vec![]))
        .unwrap_err();
    manager
        .start_whole_game_analysis(whole_game_request(&candidate.run_id, 14, 2))
        .unwrap_err();
    let started = manager
        .start_selected_node_job(selected_request(&run_a, 14, vec![]))
        .unwrap();
    wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == started.job_id && job.outcome == AnalysisJobOutcomeDto::Completed
    });
}

#[cfg(unix)]
#[test]
fn autoload_after_teardown_does_not_restore_old_run_snapshot_or_jobs() {
    let temp = TestTempDir::new("autoload-session");
    let catalog = Arc::new(InMemoryEngineProfileCatalog::new());
    catalog.upsert(SavedEngineProfile {
        profile_id: "marked".into(),
        profile: setup_profile(&temp, &resident_echo_script()),
    });
    catalog.set_autoload_profile_id(Some("marked".into()));
    let first = ForegroundEngineManager::new(catalog.clone(), ForegroundEngineConfig::for_tests());
    let first_events = first.subscribe();
    first.start("marked").unwrap();
    let first_ready = wait_snapshot(&first_events, Duration::from_secs(3), |lifecycle| {
        matches!(lifecycle, ForegroundEngineLifecycleDto::Ready { .. })
    });
    let old_run_id = run_from_ready(&first_ready.lifecycle).run_id.clone();
    first
        .register_job(&old_run_id, AnalysisJobLane::SelectedNode, Arc::new(NoopCancel))
        .unwrap();
    first.teardown().unwrap();
    assert!(matches!(
        first.snapshot().lifecycle,
        ForegroundEngineLifecycleDto::NoEngine { .. }
    ));

    let second = ForegroundEngineManager::new(catalog, ForegroundEngineConfig::for_tests());
    assert!(matches!(
        second.snapshot().lifecycle,
        ForegroundEngineLifecycleDto::NoEngine { .. }
    ));
    let second_events = second.subscribe();
    second.apply_autoload().unwrap();
    let second_ready = wait_snapshot(&second_events, Duration::from_secs(3), |lifecycle| {
        matches!(lifecycle, ForegroundEngineLifecycleDto::Ready { .. })
    });
    let new_run = run_from_ready(&second_ready.lifecycle);
    assert_ne!(new_run.run_id, old_run_id);
    assert_eq!(new_run.profile_id, "marked");
    second
        .register_job(&old_run_id, AnalysisJobLane::SelectedNode, Arc::new(NoopCancel))
        .expect_err("old run identity must not admit jobs on a new session");
}

#[cfg(unix)]
struct NoopCancel;

#[cfg(unix)]
impl AnalysisJobCancel for NoopCancel {
    fn cancel(&self) {}
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
        ForegroundEngineLifecycleDto::NoEngine { .. }
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
        ForegroundEngineLifecycleDto::NoEngine { .. }
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
        ForegroundEngineLifecycleDto::NoEngine { .. }
    ));
}

#[cfg(unix)]
#[test]
fn stdout_close_before_probe_stays_no_engine_with_readiness_kind() {
    let temp = TestTempDir::new("readiness");
    let catalog = Arc::new(InMemoryEngineProfileCatalog::new());
    catalog.upsert(SavedEngineProfile {
        profile_id: "profile-1".into(),
        profile: setup_profile(&temp, "read line\nexit 0"),
    });
    let manager = ForegroundEngineManager::new(catalog, ForegroundEngineConfig::for_tests());
    let events = manager.subscribe();
    manager.start("profile-1").unwrap();
    let failure = wait_failure(&events, Duration::from_secs(3), |_| true);
    assert_eq!(failure.kind, EngineFailureKind::Readiness);
    assert_eq!(failure.operation, EngineOperationDto::Start);
    assert_eq!(failure.profile_id.as_deref(), Some("profile-1"));
    assert!(failure.run_id.is_some());
    assert!(matches!(
        manager.snapshot().lifecycle,
        ForegroundEngineLifecycleDto::NoEngine { .. }
    ));
    assert_ne!(failure.kind, EngineFailureKind::Cancellation);
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
        matches!(lifecycle, ForegroundEngineLifecycleDto::NoEngine { .. })
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
    wait_snapshot(&events, Duration::from_secs(4), |lifecycle| {
        no_engine_failure(lifecycle).is_some_and(|failure| failure.kind == EngineFailureKind::Asset)
    });
    let snapshot = manager.snapshot();
    let failure = no_engine_failure(&snapshot.lifecycle)
        .expect("failed recovery Restart must publish snapshot failure");
    assert_eq!(failure.operation, EngineOperationDto::Start);
    assert_eq!(failure.profile_id.as_deref(), Some("profile-1"));
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
        ForegroundEngineLifecycleDto::NoEngine { .. }
    ));
    second
        .register_job(
            &old_run_id,
            AnalysisJobLane::SelectedNode,
            Arc::new(AnalysisCancelToken::new()),
        )
        .expect_err("a new session must not restore the previous Error Run identity");
}
