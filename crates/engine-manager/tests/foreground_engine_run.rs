use app_model::{
    admits_analysis_publication, AnalysisJobEventDto, AnalysisJobOutcomeDto, AnalysisPositionIntervalDto,
    AnalysisPublicationScopeDto, AnalysisScopeDto, AnalysisScopeModeDto, AnalysisStageConditionsDto,
    AnalysisTaskLimitDto, AnalysisTaskStageDto, AnalysisTaskStateDto, AnalysisTaskStrategyDto,
    EngineAdapterSettings, EngineAnalysisCapabilitiesDto, EngineBackend, EngineCapabilitySnapshotDto,
    EngineFailureKind, EngineOperationDto, EngineProfileDto, ForegroundEngineEventDto,
    ForegroundEngineLifecycleDto, GenericGtpSettings, KataGoSettings, MoveVertex, NodePath, PointDto,
};
use engine_manager::{
    AnalysisCancelToken, AnalysisJobCancel, AnalysisJobLane, EngineProfileCatalog, ForegroundEngineConfig,
    ForegroundEngineManager, InMemoryEngineProfileCatalog, SavedEngineProfile, SelectedNodeJobRequest,
    WholeGameJobRequest, WholeGameWorkItem,
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
        report_during_search_every: None,
        override_settings: None,
    }
}

fn selected_request(run_id: &str, generation: u64, indices: Vec<u32>) -> SelectedNodeJobRequest {
    SelectedNodeJobRequest {
        exact_position: Err("JSONL fixture does not require exact GTP history".into()),
        run_id: run_id.into(),
        mode: app_model::AnalysisJobModeDto::Finite,
        generation,
        node_path: NodePath { indices },
        query: sample_query(),
        board_width: 9,
        board_height: 9,
        position_empty: true,
    }
}

fn whole_game_request(run_id: &str, generation: u64, expected_responses: usize) -> WholeGameJobRequest {
    WholeGameJobRequest {
        run_id: run_id.into(),
        generation,
        work_items: (0..expected_responses)
            .map(|depth| WholeGameWorkItem {
                node_path: NodePath {
                    indices: vec![0; depth],
                },
                query: sample_query(),
                board_width: 9,
                board_height: 9,
                move_number: depth as u32,
            })
            .collect(),
    }
}

fn analysis_scope() -> AnalysisScopeDto {
    AnalysisScopeDto {
        mode: AnalysisScopeModeDto::AllBranches,
        current_node: NodePath { indices: vec![1] },
        branch_choices: Vec::new(),
        interval: Some(AnalysisPositionIntervalDto { start: 0, end: 10 }),
        to_play: None,
    }
}

fn task_conditions(total_visits: u32) -> AnalysisStageConditionsDto {
    AnalysisStageConditionsDto {
        time_seconds: AnalysisTaskLimitDto {
            enabled: false,
            value: 10,
        },
        total_visits: AnalysisTaskLimitDto {
            enabled: true,
            value: total_visits,
        },
        leading_candidate_visits: AnalysisTaskLimitDto {
            enabled: false,
            value: 500,
        },
    }
}

fn count_job_queries(logged: &str, job_id: &str) -> usize {
    logged
        .lines()
        .filter(|line| {
            line.contains(job_id) && !line.contains("terminate") && !line.contains("lifecycle-readiness-")
        })
        .count()
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
    target=$(printf '%s' "$line" | sed -n 's/.*"terminateId":"\([^"]*\)".*/\1/p')
    [ -n "$target" ] && [ "$target" != "$id" ] || exit 20
    printf '%s\n' "$line"
    printf '{"id":"%s","turnNumber":0,"isDuringSearch":false,"noResults":true}\n' "$target"
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
    target=$(printf '%s' "$line" | sed -n 's/.*"terminateId":"\([^"]*\)".*/\1/p')
    [ -n "$target" ] && [ "$target" != "$id" ] || exit 20
    printf '%s\n' "$line"
    printf '{"id":"%s","turnNumber":0,"isDuringSearch":false,"noResults":true}\n' "$target"
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
  printf '{"id":"%s","turnNumber":0,"isDuringSearch":false,"rootInfo":{"visits":4,"winrate":0.6,"scoreMean":1.25},"moveInfos":[{"move":"D4","visits":4,"winrate":0.6,"scoreMean":1.25}]}\n' "$id"
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
  if printf '%s' "$line" | grep -q '"action":"terminate"'; then
    target=$(printf '%s' "$line" | sed -n 's/.*"terminateId":"\([^"]*\)".*/\1/p')
    [ -n "$target" ] && [ "$target" != "$id" ] || exit 20
    printf '%s\n' "$line"
    printf '{"id":"%s","turnNumber":0,"isDuringSearch":false,"noResults":true}\n' "$target"
    continue
  fi
  printf '{"id":"%s","turnNumber":0,"isDuringSearch":false,"rootInfo":{"visits":4,"winrate":0.6,"scoreMean":1.25},"moveInfos":[{"move":"D4","visits":4,"winrate":0.6,"scoreMean":1.25}]}\n' "$id"
  if [ "${EXIT_AFTER:-}" = "first" ]; then
    exit "${EXIT_CODE:-1}"
  fi
done
"#
    .into()
}

#[cfg(unix)]
fn selected_node_final_script(final_response: &str) -> String {
    r#"
first=1
while IFS= read -r line; do
  if [ -n "$ENGINE_LOG" ]; then
    printf '%s\n' "$line" >> "$ENGINE_LOG"
  fi
  id=$(printf '%s' "$line" | sed -n 's/.*"id":"\([^"]*\)".*/\1/p')
  [ -z "$id" ] && id="ok"
  if [ "$first" = 1 ]; then
    printf '{"id":"%s","turnNumber":0}\n' "$id"
    first=0
    continue
  fi
  FINAL_RESPONSE
done
"#
    .replace("FINAL_RESPONSE", final_response)
}

#[cfg(unix)]
fn resident_selected_node_result_script() -> String {
    selected_node_final_script(
        r#"printf '{"id":"%s","turnNumber":2,"isDuringSearch":false,"rootInfo":{"visits":8,"winrate":0.61,"scoreMean":2.5,"scoreStdev":4.0},"moveInfos":[{"move":"B2","visits":5,"winrate":0.62,"scoreMean":2.7,"prior":0.4,"pv":["B2","A1"]}],"ownership":[0.1,-0.2,0.3,0.0],"policy":[0.01,0.02,0.03,0.04,-1.0]}\n' "$id""#,
    )
}

#[cfg(unix)]
fn selected_node_protocol_error_script() -> String {
    r#"
first=1
while IFS= read -r line; do
  if [ -n "$ENGINE_LOG" ]; then
    printf '%s\n' "$line" >> "$ENGINE_LOG"
  fi
  id=$(printf '%s' "$line" | sed -n 's/.*"id":"\([^"]*\)".*/\1/p')
  [ -z "$id" ] && id="ok"
  if [ "$first" = 1 ]; then
    printf '{"id":"%s","turnNumber":0}\n' "$id"
    first=0
    continue
  fi
  printf 'engine stderr failure for %s\n' "$id" >&2
  printf '{"id":"%s","error":"stderr boom"}\n' "$id"
done
"#
    .into()
}

#[cfg(unix)]
fn selected_node_malformed_response_script() -> String {
    r#"
first=1
while IFS= read -r line; do
  if [ -n "$ENGINE_LOG" ]; then
    printf '%s\n' "$line" >> "$ENGINE_LOG"
  fi
  id=$(printf '%s' "$line" | sed -n 's/.*"id":"\([^"]*\)".*/\1/p')
  [ -z "$id" ] && id="ok"
  if [ "$first" = 1 ]; then
    printf '{"id":"%s","turnNumber":0}\n' "$id"
    first=0
    continue
  fi
  printf '{"id":"%s","broken"\n' "$id"
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
        program: engine_path.to_string_lossy().into_owned(),
        argv: vec![],
        working_dir: Some(temp.path().to_string_lossy().into_owned()),
        adapter: EngineAdapterSettings::KataGoAnalysis(KataGoSettings {
            model_path: Some(model_path.to_string_lossy().into_owned()),
            config_path: Some(config_path.to_string_lossy().into_owned()),
            max_visits: 800,
        }),
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
            program: "/definitely/missing/katago".into(),
            argv: vec![],
            working_dir: None,
            adapter: EngineAdapterSettings::KataGoAnalysis(KataGoSettings {
                model_path: Some("/definitely/missing/model.bin".into()),
                config_path: Some("/definitely/missing/analysis.cfg".into()),
                max_visits: 800,
            }),
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
fn selected_node_completion_publishes_normalized_candidates_pv_ownership_policy_and_score() {
    let temp = TestTempDir::new("selected-normalized");
    let log = temp.path().join("engine.log");
    let mut script = format!(
        "ENGINE_LOG='{}'
",
        log.display()
    );
    script.push_str(&resident_selected_node_result_script());
    let (manager, _, events, run_id) = ready_manager(&temp, &script);
    let mut query = sample_query();
    query.rules = "japanese".into();
    query.komi = 6.5;
    query.include_ownership = Some(true);
    query.include_policy = Some(true);
    query.initial_stones = vec![("B".into(), "B1".into())];
    query.board_x_size = 2;
    query.board_y_size = 2;
    let started = manager
        .start_selected_node_job(SelectedNodeJobRequest {
            exact_position: Err("JSONL fixture does not require exact GTP history".into()),
            run_id: run_id.clone(),
            mode: app_model::AnalysisJobModeDto::Finite,
            generation: 4,
            node_path: NodePath { indices: vec![0, 1] },
            query,
            board_width: 2,
            board_height: 2,
            position_empty: true,
        })
        .unwrap();
    let completed = wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == started.job_id && job.outcome == AnalysisJobOutcomeDto::Completed
    });
    let frame = completed.frame.as_ref().expect("normalized frame");
    assert_eq!(completed.run_id, run_id);
    assert_eq!(completed.generation, 4);
    assert_eq!(completed.node_path.indices, vec![0, 1]);
    assert_eq!(frame.visits, 8);
    assert!((frame.winrate_black - 0.61).abs() < f32::EPSILON);
    assert_eq!(frame.score_mean_black, Some(2.5));
    assert_eq!(frame.score_stdev, Some(4.0));
    assert_eq!(frame.candidates.len(), 1);
    assert_eq!(
        frame.candidates[0].vertex,
        MoveVertex::Point(PointDto { x: 1, y: 0 })
    );
    assert_eq!(
        frame.candidates[0].pv,
        vec![
            MoveVertex::Point(PointDto { x: 1, y: 0 }),
            MoveVertex::Point(PointDto { x: 0, y: 1 }),
        ]
    );
    assert_eq!(frame.candidates[0].policy_prior, Some(0.4));
    assert_eq!(frame.ownership, Some(vec![0.1, -0.2, 0.3, 0.0]));
    assert_eq!(frame.policy, Some(vec![0.01, 0.02, 0.03, 0.04, -1.0]));
    let current = started.publication_scope();
    assert!(admits_analysis_publication(&completed, &current));
    let logged = std::fs::read_to_string(&log).unwrap();
    assert!(logged.contains(r#""rules":"japanese""#), "{logged}");
    assert!(logged.contains(r#""komi":6.5"#), "{logged}");
    assert!(logged.contains(r#""includeOwnership":true"#), "{logged}");
    assert!(logged.contains(r#""includePolicy":true"#), "{logged}");
    assert!(logged.contains(r#"["B","B1"]"#), "{logged}");
    assert!(matches!(
        manager.snapshot().lifecycle,
        ForegroundEngineLifecycleDto::Ready { .. }
    ));
}

#[cfg(unix)]
#[test]
fn selected_node_invalid_or_unmarked_final_fails_run_without_a_frame() {
    for (variant, response) in [
        (
            "invalid-root",
            r#"printf '{"id":"%s","turnNumber":2,"isDuringSearch":false,"rootInfo":{"visits":1,"winrate":1.5},"moveInfos":[]}\n' "$id""#,
        ),
        (
            "missing-marker",
            r#"printf '{"id":"%s","turnNumber":2,"rootInfo":{"visits":1,"winrate":0.5},"moveInfos":[]}\n' "$id""#,
        ),
    ] {
        let temp = TestTempDir::new(&format!("selected-{variant}"));
        let (manager, _, events, run_id) = ready_manager(&temp, &selected_node_final_script(response));
        let started = manager
            .start_selected_node_job(selected_request(&run_id, 1, vec![]))
            .unwrap();
        let failed = wait_job(&events, Duration::from_secs(2), |job| {
            job.job_id == started.job_id && job.outcome == AnalysisJobOutcomeDto::Failed
        });
        assert!(failed.frame.is_none(), "{variant}");
        assert!(matches!(
            manager.snapshot().lifecycle,
            ForegroundEngineLifecycleDto::Error { .. }
        ));
    }
}

#[cfg(unix)]
#[test]
fn selected_node_protocol_stderr_failure_is_typed_and_publishes_no_frame() {
    let temp = TestTempDir::new("selected-protocol");
    let log = temp.path().join("engine.log");
    let mut script = format!(
        "ENGINE_LOG='{}'
",
        log.display()
    );
    script.push_str(&selected_node_protocol_error_script());
    let (manager, _, events, run_id) = ready_manager(&temp, &script);
    let started = manager
        .start_selected_node_job(selected_request(&run_id, 3, vec![0]))
        .unwrap();
    let failed = wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == started.job_id && job.outcome == AnalysisJobOutcomeDto::Failed
    });
    assert!(failed.frame.is_none());
    let failure = failed.failure.as_ref().expect("typed protocol failure");
    assert_eq!(failure.kind, EngineFailureKind::Protocol);
    assert_eq!(failure.job_id.as_deref(), Some(started.job_id.as_str()));
    assert!(failure.message.contains("stderr boom"), "{}", failure.message);
    let current = started.publication_scope();
    assert!(!admits_analysis_publication(&failed, &current));
    assert!(matches!(
        manager.snapshot().lifecycle,
        ForegroundEngineLifecycleDto::Ready { .. }
    ));
}

#[cfg(unix)]
#[test]
fn continuous_board_size_refusal_fails_only_the_job_and_keeps_exact_ready_run() {
    let temp = TestTempDir::new("continuous-board-size-refusal");
    let script = selected_node_final_script(
        r#"printf '{"id":"%s","error":"Must provide an integer from 2 to 19","field":"boardXSize"}\n' "$id""#,
    );
    let (manager, _, events, run_id) = ready_manager(&temp, &script);
    let mut request = continuous_request(&run_id, 4, vec![]);
    request.query.board_x_size = 25;
    request.query.board_y_size = 2;
    request.board_width = 25;
    request.board_height = 2;
    let started = manager.start_selected_node_job(request).unwrap();

    let failed = wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == started.job_id && job.outcome == AnalysisJobOutcomeDto::Failed
    });
    let failure = failed.failure.expect("typed capability refusal");
    assert_eq!(failure.kind, EngineFailureKind::UnsupportedCapability);
    assert!(failure.message.contains("25×2"), "{}", failure.message);
    assert!(failure.message.contains("2 to 19"), "{}", failure.message);
    assert!(matches!(
        manager.snapshot().lifecycle,
        ForegroundEngineLifecycleDto::Ready { ref run } if run.run_id == run_id
    ));
}

#[cfg(unix)]
#[test]
fn finite_board_size_refusal_fails_only_the_job_and_keeps_exact_ready_run() {
    let temp = TestTempDir::new("finite-board-size-refusal");
    let script = selected_node_final_script(
        r#"printf '{"id":"%s","error":"Must provide an integer from 2 to 19","field":"boardYSize"}\n' "$id""#,
    );
    let (manager, _, events, run_id) = ready_manager(&temp, &script);
    let mut request = selected_request(&run_id, 4, vec![]);
    request.query.board_x_size = 2;
    request.query.board_y_size = 25;
    request.board_width = 2;
    request.board_height = 25;
    let started = manager.start_selected_node_job(request).unwrap();

    let failed = wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == started.job_id && job.outcome == AnalysisJobOutcomeDto::Failed
    });
    let failure = failed.failure.expect("typed capability refusal");
    assert_eq!(failure.kind, EngineFailureKind::UnsupportedCapability);
    assert!(failure.message.contains("2×25"), "{}", failure.message);
    assert!(failure.message.contains("2 to 19"), "{}", failure.message);
    assert!(matches!(
        manager.snapshot().lifecycle,
        ForegroundEngineLifecycleDto::Ready { ref run } if run.run_id == run_id
    ));
}

#[cfg(unix)]
#[test]
fn generic_integer_error_for_another_field_remains_a_protocol_failure() {
    let temp = TestTempDir::new("non-board-integer-refusal");
    let script = selected_node_final_script(
        r#"printf '{"id":"%s","error":"Must provide an integer from 2 to 19","field":"maxVisits"}\n' "$id""#,
    );
    let (manager, _, events, run_id) = ready_manager(&temp, &script);
    let started = manager
        .start_selected_node_job(selected_request(&run_id, 4, vec![]))
        .unwrap();

    let failed = wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == started.job_id && job.outcome == AnalysisJobOutcomeDto::Failed
    });
    let failure = failed.failure.expect("typed protocol failure");
    assert_eq!(failure.kind, EngineFailureKind::Protocol);
    assert!(matches!(
        manager.snapshot().lifecycle,
        ForegroundEngineLifecycleDto::Ready { ref run } if run.run_id == run_id
    ));
}

#[cfg(unix)]
#[test]
fn selected_node_malformed_response_is_typed_failure_without_a_frame() {
    let temp = TestTempDir::new("selected-malformed");
    let mut script = String::new();
    script.push_str(&selected_node_malformed_response_script());
    let (manager, _, events, run_id) = ready_manager(&temp, &script);
    let started = manager
        .start_selected_node_job(selected_request(&run_id, 1, vec![]))
        .unwrap();
    let failed = wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == started.job_id && job.outcome == AnalysisJobOutcomeDto::Failed
    });
    assert!(failed.frame.is_none());
    let failure = failed.failure.as_ref().expect("typed protocol failure");
    assert_eq!(failure.kind, EngineFailureKind::Protocol);
    assert!(!admits_analysis_publication(
        &failed,
        &started.publication_scope()
    ));
    assert!(matches!(
        manager.snapshot().lifecycle,
        ForegroundEngineLifecycleDto::Ready { .. }
    ));
}

#[cfg(unix)]
#[test]
fn selected_node_protocol_failure_does_not_cancel_whole_game() {
    let temp = TestTempDir::new("selected-fail-whole-game");
    let cancel_marker = temp.path().join("whole-game-cancelled");
    let script = selected_node_protocol_error_script();
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
    let started = manager
        .start_selected_node_job(selected_request(&run_id, 2, vec![0]))
        .unwrap();
    let failed = wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == started.job_id && job.outcome == AnalysisJobOutcomeDto::Failed
    });
    assert!(failed.frame.is_none());
    assert!(!cancel_marker.exists());
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
    manager.set_capability_snapshot_for_tests(Some(EngineCapabilitySnapshotDto {
        adapter_kind: EngineBackend::KataGoAnalysis,
        game_move: true,
        gtp: None,
        analysis: Some(EngineAnalysisCapabilitiesDto {
            selected_node_analysis: false,
            continuous_analysis: true,
            candidates: true,
            pv: true,
            winrate: true,
            ownership: true,
            policy: true,
            visits_limit: true,
            whole_game_analysis: true,
            root_score: true,
            protocol_cancel: true,
        }),
    }));
    let error = manager
        .start_selected_node_job(selected_request(&run_id, 1, vec![]))
        .unwrap_err();
    assert_eq!(error.kind, EngineFailureKind::UnsupportedCapability);
    let after = std::fs::read_to_string(&log).unwrap_or_default();
    assert_eq!(before, after);
}

#[cfg(unix)]
#[test]
fn admission_missing_analysis_preserves_active_lanes_and_protocol() {
    for missing in [
        None,
        Some(EngineCapabilitySnapshotDto {
            adapter_kind: EngineBackend::GenericGtp,
            game_move: false,
            gtp: None,
            analysis: None,
        }),
    ] {
        let temp = TestTempDir::new("admission-missing-analysis");
        let (manager, _, run_id, log) = hold_both_lanes_manager(&temp);
        let (selected, whole) = start_both_lanes(&manager, &run_id, 42);
        let deadline = Instant::now() + Duration::from_secs(2);
        let before_log = loop {
            let logged = std::fs::read_to_string(&log).unwrap();
            if count_job_queries(&logged, &selected.job_id) == 1
                && count_job_queries(&logged, &whole.job_id) == 1
            {
                break logged;
            }
            assert!(
                Instant::now() < deadline,
                "both admitted jobs must reach the engine"
            );
            std::thread::sleep(Duration::from_millis(10));
        };
        manager.set_capability_snapshot_for_tests(missing);
        let before = manager.snapshot();
        let task_before = manager.analysis_task_snapshot();
        for request in [
            selected_request(&run_id, 42, vec![1]),
            continuous_request(&run_id, 42, vec![1]),
        ] {
            assert_eq!(
                manager.start_selected_node_job(request).unwrap_err().kind,
                EngineFailureKind::UnsupportedCapability
            );
        }
        for lane in [AnalysisJobLane::SelectedNode, AnalysisJobLane::WholeGame] {
            assert_eq!(
                manager
                    .register_job(&run_id, lane, Arc::new(AnalysisCancelToken::new()))
                    .unwrap_err()
                    .kind,
                EngineFailureKind::UnsupportedCapability
            );
        }
        assert_eq!(
            manager
                .start_analysis_task(
                    whole_game_request(&run_id, 42, 1),
                    analysis_scope(),
                    task_conditions(32)
                )
                .unwrap_err()
                .kind,
            EngineFailureKind::UnsupportedCapability
        );
        assert_eq!(
            manager.check_analysis_task_admission(None).unwrap_err().kind,
            EngineFailureKind::UnsupportedCapability
        );
        assert_eq!(manager.snapshot(), before);
        assert_eq!(manager.analysis_task_snapshot(), task_before);
        std::thread::sleep(Duration::from_millis(100));
        assert_eq!(std::fs::read_to_string(&log).unwrap(), before_log);
        manager.teardown().unwrap();
    }
}

#[cfg(unix)]
#[test]
fn admission_continue_rechecks_capabilities_without_replacing_paused_task() {
    let temp = TestTempDir::new("admission-paused-task");
    std::fs::write(temp.path().join("cancel-final"), "").unwrap();
    let (manager, _, events, run_id) = ready_manager(&temp, &task_engine_script(temp.path()));
    let verified = lifecycle_run(&manager.snapshot().lifecycle)
        .unwrap()
        .capability_snapshot
        .clone();
    let task = manager
        .start_analysis_task(
            whole_game_request(&run_id, 42, 2),
            analysis_scope(),
            task_conditions(32),
        )
        .unwrap();
    wait_job(&events, Duration::from_secs(2), |job| {
        job.outcome == AnalysisJobOutcomeDto::Progress
    });
    manager.pause_analysis_task(&run_id, &task.task_id).unwrap();
    let paused = wait_task(&manager, AnalysisTaskStateDto::Paused);
    let before_log = std::fs::read_to_string(temp.path().join("queries.jsonl")).unwrap();
    let mut whole_disabled = verified.clone().unwrap();
    whole_disabled.analysis.as_mut().unwrap().whole_game_analysis = false;
    for missing in [
        None,
        Some(EngineCapabilitySnapshotDto {
            adapter_kind: EngineBackend::GenericGtp,
            game_move: false,
            analysis: None,
            gtp: None,
        }),
        Some(whole_disabled),
    ] {
        manager.set_capability_snapshot_for_tests(missing);
        let before = manager.snapshot();
        assert_eq!(
            manager
                .continue_analysis_task(&run_id, &task.task_id, 42)
                .unwrap_err()
                .kind,
            EngineFailureKind::UnsupportedCapability
        );
        assert_eq!(manager.snapshot(), before);
        assert_eq!(manager.analysis_task_snapshot().unwrap(), paused);
        assert_eq!(
            std::fs::read_to_string(temp.path().join("queries.jsonl")).unwrap(),
            before_log
        );
    }
    manager.set_capability_snapshot_for_tests(verified);
    let continued = manager
        .continue_analysis_task(&run_id, &task.task_id, 42)
        .unwrap();
    assert_eq!(continued.task_id, paused.task_id);
    assert_ne!(continued.job_id, paused.job_id);
    assert_eq!(continued.completed, paused.completed);
    manager.teardown().unwrap();
}

#[cfg(unix)]
#[test]
fn admission_finite_continuous_and_whole_game_are_independent() {
    for admitted in ["finite", "continuous", "whole"] {
        let temp = TestTempDir::new("admission-independent");
        let (manager, events, run_id, log) = hold_both_lanes_manager(&temp);
        let mut verified = lifecycle_run(&manager.snapshot().lifecycle)
            .unwrap()
            .capability_snapshot
            .clone()
            .unwrap();
        let analysis = verified.analysis.as_mut().unwrap();
        analysis.selected_node_analysis = admitted == "finite";
        analysis.continuous_analysis = admitted == "continuous";
        analysis.whole_game_analysis = admitted == "whole";
        manager.set_capability_snapshot_for_tests(Some(verified));
        manager
            .set_continuous_preferences(false, app_model::ContinuousAnalysisBudgetDto::default())
            .unwrap();
        for requested in ["finite", "continuous", "whole"] {
            if requested == admitted {
                continue;
            }
            let before = manager.snapshot();
            let error = match requested {
                "finite" => manager
                    .start_selected_node_job(selected_request(&run_id, 42, vec![0]))
                    .unwrap_err(),
                "continuous" => manager
                    .start_selected_node_job(continuous_request(&run_id, 42, vec![0]))
                    .unwrap_err(),
                _ => manager
                    .start_whole_game_analysis(whole_game_request(&run_id, 42, 1))
                    .unwrap_err(),
            };
            assert_eq!(
                error.kind,
                EngineFailureKind::UnsupportedCapability,
                "{admitted}/{requested}"
            );
            assert_eq!(manager.snapshot(), before);
        }
        let job = match admitted {
            "finite" => manager
                .start_selected_node_job(selected_request(&run_id, 42, vec![0]))
                .unwrap(),
            "continuous" => manager
                .start_selected_node_job(continuous_request(&run_id, 42, vec![0]))
                .unwrap(),
            _ => manager
                .start_whole_game_analysis(whole_game_request(&run_id, 42, 1))
                .unwrap(),
        };
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            let logged = std::fs::read_to_string(&log).unwrap();
            if count_job_queries(&logged, &job.job_id) == 1 {
                break;
            }
            assert!(
                Instant::now() < deadline,
                "{admitted} query must reach the engine"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
        manager.cancel_job(&run_id, &job.job_id).unwrap();
        wait_job(&events, Duration::from_secs(2), |event| {
            event.job_id == job.job_id && event.outcome == AnalysisJobOutcomeDto::Cancelled
        });
        manager.teardown().unwrap();
    }
}

#[cfg(unix)]
#[test]
fn admission_query_features_reject_before_selected_takeover_or_task_creation() {
    for feature in ["ownership", "policy", "visits"] {
        let temp = TestTempDir::new("admission-query-features");
        let (manager, _, run_id, log) = hold_both_lanes_manager(&temp);
        let active = manager
            .start_selected_node_job(selected_request(&run_id, 42, vec![0]))
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        let before_log = loop {
            let logged = std::fs::read_to_string(&log).unwrap();
            if count_job_queries(&logged, &active.job_id) == 1 {
                break logged;
            }
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(10));
        };
        let mut verified = lifecycle_run(&manager.snapshot().lifecycle)
            .unwrap()
            .capability_snapshot
            .clone()
            .unwrap();
        let caps = verified.analysis.as_mut().unwrap();
        match feature {
            "ownership" => caps.ownership = false,
            "policy" => caps.policy = false,
            _ => caps.visits_limit = false,
        }
        manager.set_capability_snapshot_for_tests(Some(verified));
        let mut request = selected_request(&run_id, 42, vec![1]);
        request.query.include_ownership = Some(feature == "ownership");
        request.query.include_policy = Some(feature == "policy");
        let mut whole = whole_game_request(&run_id, 42, 1);
        whole.work_items[0].query = request.query.clone();
        let before = manager.snapshot();
        assert_eq!(
            manager.start_selected_node_job(request).unwrap_err().kind,
            EngineFailureKind::UnsupportedCapability
        );
        assert_eq!(
            manager
                .start_analysis_task(whole, analysis_scope(), task_conditions(32))
                .unwrap_err()
                .kind,
            EngineFailureKind::UnsupportedCapability
        );
        assert_eq!(manager.snapshot(), before);
        assert!(manager.analysis_task_snapshot().is_none());
        std::thread::sleep(Duration::from_millis(100));
        assert_eq!(std::fs::read_to_string(&log).unwrap(), before_log);
        manager.teardown().unwrap();
    }
}

#[cfg(unix)]
#[test]
fn admission_continuous_unavailable_preserves_intent_and_safety_hold() {
    let temp = TestTempDir::new("admission-continuous-hold");
    let (manager, _, run_id, log) = hold_both_lanes_manager(&temp);
    let mut verified = lifecycle_run(&manager.snapshot().lifecycle)
        .unwrap()
        .capability_snapshot
        .clone()
        .unwrap();
    verified.analysis.as_mut().unwrap().selected_node_analysis = false;
    let mut unavailable = verified.clone();
    unavailable.analysis.as_mut().unwrap().continuous_analysis = false;
    manager.set_capability_snapshot_for_tests(Some(unavailable));
    let budget = app_model::ContinuousAnalysisBudgetDto::default();
    manager.set_continuous_preferences(true, budget).unwrap();
    let mut target = continuous_request(&run_id, 42, vec![0]);
    target.position_empty = false;
    manager.follow_continuous_position(target.clone());
    let unavailable = manager.snapshot();
    assert_eq!(unavailable.continuous.enabled, Some(true));
    assert_eq!(
        unavailable.continuous.phase,
        app_model::ContinuousAnalysisPhaseDto::Unavailable
    );
    let before_log = std::fs::read_to_string(&log).unwrap();
    assert_eq!(
        manager.continuous_primary_action().unwrap_err().kind,
        EngineFailureKind::UnsupportedCapability
    );
    assert_eq!(
        manager.resume_continuous().unwrap_err().kind,
        EngineFailureKind::UnsupportedCapability
    );
    manager.authorize_continuous_start();
    assert_eq!(manager.snapshot(), unavailable);
    manager.begin_continuous_departure();
    manager.finish_continuous_departure(false);
    let held = manager.snapshot();
    assert_eq!(
        held.continuous.phase,
        app_model::ContinuousAnalysisPhaseDto::SafetyHold
    );
    assert_eq!(
        manager.resume_continuous().unwrap_err().kind,
        EngineFailureKind::UnsupportedCapability
    );
    manager.authorize_continuous_start();
    manager.follow_continuous_position(target);
    assert_eq!(manager.snapshot(), held);
    assert_eq!(std::fs::read_to_string(&log).unwrap(), before_log);
    manager.set_capability_snapshot_for_tests(Some(verified));
    manager.set_continuous_preferences(true, budget).unwrap();
    assert_eq!(
        manager.snapshot().continuous.phase,
        app_model::ContinuousAnalysisPhaseDto::SafetyHold
    );
    assert!(manager.snapshot().selected_node_job.is_none());
    manager.resume_continuous().unwrap();
    let admitted = manager.snapshot().selected_node_job.unwrap();
    assert_eq!(admitted.mode, app_model::AnalysisJobModeDto::Continuous);
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        if count_job_queries(&std::fs::read_to_string(&log).unwrap(), &admitted.job_id) == 1 {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "continuous resume must submit with finite disabled"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    manager.teardown().unwrap();
}

#[cfg(unix)]
#[test]
fn admission_swing_preview_and_start_require_requested_metrics() {
    for metric in ["score", "winrate"] {
        let temp = TestTempDir::new("admission-swing-metrics");
        let (manager, _, run_id, log) = hold_both_lanes_manager(&temp);
        let mut verified = lifecycle_run(&manager.snapshot().lifecycle)
            .unwrap()
            .capability_snapshot
            .clone()
            .unwrap();
        verified.analysis.as_mut().unwrap().root_score = metric != "score";
        verified.analysis.as_mut().unwrap().winrate = metric != "winrate";
        manager.set_capability_snapshot_for_tests(Some(verified));
        let criteria = app_model::AnalysisSwingCriteriaDto {
            move_actors: app_model::AnalysisMoveActorFilterDto::Both,
            score_change_points: app_model::AnalysisSwingThresholdDto {
                enabled: metric == "score",
                value: 3.0,
            },
            winrate_change_percentage_points: app_model::AnalysisSwingThresholdDto {
                enabled: metric == "winrate",
                value: 10.0,
            },
        };
        let before = manager.snapshot();
        let before_log = std::fs::read_to_string(&log).unwrap();
        assert_eq!(
            manager
                .check_analysis_task_admission(Some(&criteria))
                .unwrap_err()
                .kind,
            EngineFailureKind::UnsupportedCapability
        );
        let job = whole_game_request(&run_id, 42, 2);
        let requested = job.work_items.iter().map(|item| item.node_path.clone()).collect();
        let error = manager
            .start_swing_analysis_task(engine_manager::SwingAnalysisTaskRequest {
                job,
                scope: analysis_scope(),
                requested,
                supporting: Vec::new(),
                swing_comparisons: Vec::new(),
                swing_criteria: criteria,
                overview_conditions: task_conditions(8),
                deep_conditions: task_conditions(32),
            })
            .unwrap_err();
        assert_eq!(error.kind, EngineFailureKind::UnsupportedCapability);
        assert_eq!(manager.snapshot(), before);
        assert!(manager.analysis_task_snapshot().is_none());
        assert_eq!(std::fs::read_to_string(&log).unwrap(), before_log);
        manager.check_analysis_task_admission(None).unwrap();
        manager.teardown().unwrap();
    }
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

#[cfg(unix)]
fn resident_whole_game_script() -> String {
    resident_echo_script()
}

#[cfg(unix)]
fn echo_first_then_hold_script(release: &Path) -> String {
    r#"
export ENGINE_LOG
exec python3 -u -c '
import json, os, sys, threading, time
cancelled = set()
first = True
def emit(identity):
    print(json.dumps(dict(id=identity, turnNumber=0, isDuringSearch=False,
        rootInfo=dict(visits=4, winrate=0.6, scoreMean=1.25),
        moveInfos=[dict(move="D4", visits=4, winrate=0.6, scoreMean=1.25)])), flush=True)
def held(identity):
    while not os.path.isfile("$RELEASE_PATH"):
        if identity in cancelled: return
        time.sleep(0.01)
    if identity not in cancelled: emit(identity)
for line in sys.stdin:
    log = os.environ.get("ENGINE_LOG")
    if log:
        with open(log, "a") as output: output.write(line)
    q = json.loads(line)
    if q.get("action") == "terminate":
        assert "terminateId" in q and q["id"] != q["terminateId"]
        cancelled.add(q["terminateId"])
        print(json.dumps(q), flush=True)
        print(json.dumps(dict(id=q["terminateId"], turnNumber=0, isDuringSearch=False, noResults=True)), flush=True)
    elif q["id"].startswith("lifecycle-readiness-"):
        emit(q["id"])
    elif first:
        first = False
        emit(q["id"])
    else:
        threading.Thread(target=held, args=(q["id"],), daemon=True).start()
'
"#.replace("$RELEASE_PATH", &release.to_string_lossy())
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
    assert_eq!(progress.remaining, Some(1));
    assert_eq!(progress.frame.as_ref().map(|frame| frame.turn), Some(0));
    let progress_done = wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == job_id && job.outcome == AnalysisJobOutcomeDto::Progress && job.completed == Some(2)
    });
    assert_eq!(progress_done.node_path.indices, vec![0]);
    assert_eq!(progress_done.expected, Some(2));
    assert_eq!(progress_done.remaining, Some(0));
    assert_eq!(progress_done.frame.as_ref().map(|frame| frame.turn), Some(1));
    let completed = wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == job_id && job.outcome == AnalysisJobOutcomeDto::Completed
    });
    assert_eq!(completed.run_id, run_id);
    assert_eq!(completed.generation, 4);
    assert_eq!(completed.completed, Some(2));
    assert_eq!(completed.expected, Some(2));
    assert_eq!(completed.remaining, Some(0));
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
    assert_eq!(count_job_queries(&logged, &job_id), 2);
}

#[cfg(unix)]
#[test]
fn analysis_task_freezes_scope_budget_and_legitimate_completions() {
    let temp = TestTempDir::new("analysis-task-complete");
    let log = temp.path().join("engine.log");
    let mut script = format!("ENGINE_LOG='{}'\n", log.display());
    script.push_str(&resident_whole_game_script());
    let (manager, _, events, run_id) = ready_manager(&temp, &script);
    let request = whole_game_request(&run_id, 40, 2);
    let task = manager
        .start_analysis_task(request, analysis_scope(), task_conditions(4))
        .unwrap();

    assert_ne!(task.task_id, task.job_id);
    assert_eq!(task.state, AnalysisTaskStateDto::Queued);
    assert_eq!(
        task.requested,
        vec![NodePath { indices: vec![] }, NodePath { indices: vec![0] }]
    );
    assert!(task.completed.is_empty());
    wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == task.job_id
            && job.outcome == AnalysisJobOutcomeDto::Progress
            && job.completed == Some(1)
    });
    let searching = manager.analysis_task_snapshot().unwrap();
    assert_eq!(searching.completed, vec![NodePath { indices: vec![] }]);
    wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == task.job_id && job.outcome == AnalysisJobOutcomeDto::Completed
    });
    let completed = manager.analysis_task_snapshot().unwrap();
    assert_eq!(completed.state, AnalysisTaskStateDto::Completed);
    assert_eq!(completed.completed, completed.requested);
    assert_eq!(completed.scope, analysis_scope());
    assert_eq!(completed.conditions, task_conditions(4));
    assert_eq!(completed.ending_conditions, vec!["total_visits"]);
    let logged = std::fs::read_to_string(log).unwrap();
    assert_eq!(count_job_queries(&logged, &task.job_id), 2);
    assert_eq!(logged.matches("\"maxVisits\":4").count(), 2);
    assert_eq!(logged.matches("\"maxTime\":1e+20").count(), 2);
}

#[cfg(unix)]
#[test]
fn all_positions_two_stage_completes_overview_before_deep_and_retains_summaries() {
    let temp = TestTempDir::new("task-two-stage-complete");
    std::fs::write(temp.path().join("finish"), "go").unwrap();
    let (manager, _, _, run_id) = ready_manager(&temp, &task_engine_script(temp.path()));
    let task = manager
        .start_all_positions_analysis_task(
            whole_game_request(&run_id, 80, 2),
            analysis_scope(),
            task_conditions(32),
            task_conditions(500),
        )
        .unwrap();
    assert_eq!(task.strategy, AnalysisTaskStrategyDto::AllPositionsTwoStage);
    assert_eq!(task.stage, AnalysisTaskStageDto::Overview);

    let completed = wait_task(&manager, AnalysisTaskStateDto::Completed);
    assert_eq!(completed.stage, AnalysisTaskStageDto::Deep);
    assert_eq!(completed.overview_completed, completed.requested);
    assert_eq!(completed.completed, completed.requested);
    assert_eq!(completed.overview_summaries.len(), 2);
    assert!(completed
        .overview_summaries
        .iter()
        .all(|summary| summary.frame.visits == 32));

    let queries: Vec<serde_json::Value> = std::fs::read_to_string(temp.path().join("queries.jsonl"))
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(queries.len(), 4);
    assert_eq!(
        queries
            .iter()
            .map(|query| query["overrideSettings"]["maxVisits"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        vec![32, 32, 500, 500]
    );
    manager.teardown().unwrap();
}

#[cfg(unix)]
#[test]
fn all_positions_two_stage_applies_time_and_leading_conditions_in_both_stages() {
    for (mode, conditions, expected_ending) in [
        (
            "time_seconds",
            budget_conditions(Some(1), None, None),
            vec!["time_seconds"],
        ),
        (
            "leading_candidate_visits",
            budget_conditions(Some(10), Some(800), Some(5)),
            vec!["leading_candidate_visits"],
        ),
    ] {
        let temp = TestTempDir::new(&format!("task-two-stage-{mode}"));
        std::fs::write(temp.path().join("budget-smoke"), mode).unwrap();
        std::fs::write(temp.path().join("cancel-final"), "go").unwrap();
        let (manager, _, _, run_id) = ready_manager(&temp, &task_engine_script(temp.path()));
        let task = manager
            .start_all_positions_analysis_task(
                whole_game_request(&run_id, 82, 1),
                analysis_scope(),
                conditions.clone(),
                conditions.clone(),
            )
            .unwrap();
        let completed = wait_task(&manager, AnalysisTaskStateDto::Completed);
        assert_eq!(completed.stage, AnalysisTaskStageDto::Deep, "{mode}");
        assert_eq!(completed.overview_completed, completed.requested, "{mode}");
        assert_eq!(completed.completed, completed.requested, "{mode}");
        assert_eq!(
            completed.overview_conditions.as_ref(),
            Some(&conditions),
            "{mode}"
        );
        assert_eq!(completed.conditions, conditions, "{mode}");
        assert_eq!(completed.ending_conditions, expected_ending, "{mode}");
        let queries = std::fs::read_to_string(temp.path().join("queries.jsonl")).unwrap();
        assert_eq!(count_job_queries(&queries, &task.job_id), 2, "{mode}");
        manager.teardown().unwrap();
    }
}

#[cfg(unix)]
#[test]
fn all_positions_two_stage_pause_continue_restarts_only_interrupted_stage_work() {
    for (label, pause_stage) in [
        ("overview", AnalysisTaskStageDto::Overview),
        ("deep", AnalysisTaskStageDto::Deep),
    ] {
        let temp = TestTempDir::new(&format!("task-two-stage-pause-{label}"));
        if pause_stage == AnalysisTaskStageDto::Overview {
            std::fs::write(temp.path().join("hold-first"), "go").unwrap();
        } else {
            std::fs::write(temp.path().join("finish"), "go").unwrap();
            std::fs::write(temp.path().join("hold-deep"), "go").unwrap();
        }
        let (manager, _, _, run_id) = ready_manager(&temp, &task_engine_script(temp.path()));
        let task = manager
            .start_all_positions_analysis_task(
                whole_game_request(&run_id, 81, 2),
                analysis_scope(),
                task_conditions(32),
                task_conditions(500),
            )
            .unwrap();
        let searching = wait_task_stage(&manager, pause_stage, AnalysisTaskStateDto::Searching);
        if pause_stage == AnalysisTaskStageDto::Deep {
            assert_eq!(searching.overview_completed, searching.requested);
            assert_eq!(searching.overview_summaries.len(), 2);
            assert!(searching.completed.is_empty());
        }
        let pausing = manager.pause_analysis_task(&run_id, &task.task_id).unwrap();
        assert_eq!(pausing.stage, pause_stage);
        assert_eq!(pausing.state, AnalysisTaskStateDto::Pausing);
        std::fs::write(temp.path().join("cancel-final"), "go").unwrap();
        let paused = wait_task_stage(&manager, pause_stage, AnalysisTaskStateDto::Paused);
        assert_eq!(paused.overview_completed, searching.overview_completed);
        assert_eq!(paused.completed, searching.completed);

        let continued = manager
            .continue_analysis_task(&run_id, &task.task_id, 81)
            .unwrap();
        assert_eq!(continued.task_id, task.task_id);
        assert_ne!(continued.job_id, task.job_id);
        assert_eq!(continued.stage, pause_stage);
        std::fs::write(temp.path().join("finish"), "go").unwrap();
        let _ = std::fs::remove_file(temp.path().join("hold-deep"));
        let completed = wait_task(&manager, AnalysisTaskStateDto::Completed);
        assert_eq!(completed.overview_completed, completed.requested);
        assert_eq!(completed.completed, completed.requested);

        let queries: Vec<serde_json::Value> = std::fs::read_to_string(temp.path().join("queries.jsonl"))
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        assert_eq!(
            queries.len(),
            5,
            "{label} interrupted one position and must repeat only that stage target"
        );
        let visits = queries
            .iter()
            .map(|query| query["overrideSettings"]["maxVisits"].as_u64().unwrap())
            .collect::<Vec<_>>();
        if pause_stage == AnalysisTaskStageDto::Overview {
            assert_eq!(visits, vec![32, 32, 32, 500, 500]);
        } else {
            assert_eq!(visits, vec![32, 32, 500, 500, 500]);
        }
        manager.teardown().unwrap();
    }
}

#[cfg(unix)]
#[test]
fn all_positions_two_stage_boundary_pause_blocks_unsent_deep_and_late_overview() {
    let temp = TestTempDir::new("task-two-stage-boundary-pause");
    let script = format!(
        r#"exec python3 -u -c '
import json,pathlib,sys,threading,time
root=pathlib.Path("{}")
lock=threading.Lock()
def emit(q):
    visits=q.get("overrideSettings",{{}}).get("maxVisits",q.get("maxVisits",2))
    with lock:
        print(json.dumps(dict(id=q["id"],isDuringSearch=False,turnNumber=0,
            rootInfo=dict(visits=visits,winrate=0.6),
            moveInfos=[dict(move="D4",order=0,visits=visits,winrate=0.6)])),flush=True)
emit(json.loads(sys.stdin.readline()))
overview=json.loads(sys.stdin.readline())
with lock:
    print(json.dumps(dict(id=overview["id"],isDuringSearch=True,turnNumber=0,
        rootInfo=dict(visits=16,winrate=0.6),
        moveInfos=[dict(move="D4",order=0,visits=16,winrate=0.6)])),flush=True)
def release_overview():
    while not (root/"release-overview").exists(): time.sleep(0.005)
    emit(overview)
    while not (root/"late-overview").exists(): time.sleep(0.005)
    emit(overview)
threading.Thread(target=release_overview,daemon=True).start()
while not (root/"release-reader").exists(): time.sleep(0.005)
for line in sys.stdin:
    with (root/"delivered.jsonl").open("a") as log: log.write(line)
    emit(json.loads(line))
'"#,
        temp.path().display()
    );
    let (manager, _, _, run_id) = ready_manager(&temp, &script);
    let task = manager
        .start_all_positions_analysis_task(
            whole_game_request(&run_id, 83, 1),
            analysis_scope(),
            task_conditions(32),
            task_conditions(500),
        )
        .unwrap();
    wait_task_stage(
        &manager,
        AnalysisTaskStageDto::Overview,
        AnalysisTaskStateDto::Searching,
    );

    let mut selected = selected_request(&run_id, 83, vec![]);
    selected.query.moves = (0..20000)
        .map(|index| (if index % 2 == 0 { "B" } else { "W" }.into(), "pass".into()))
        .collect();
    let writer = manager.clone();
    let selected_thread = std::thread::spawn(move || {
        writer.start_selected_node_job(selected).unwrap();
    });
    std::thread::sleep(Duration::from_millis(100));
    std::fs::write(temp.path().join("release-overview"), "go").unwrap();
    let boundary = wait_task_stage(&manager, AnalysisTaskStageDto::Deep, AnalysisTaskStateDto::Queued);
    assert_eq!(boundary.overview_completed, boundary.requested);
    assert!(boundary.completed.is_empty());
    let paused = manager.pause_analysis_task(&run_id, &task.task_id).unwrap();
    assert_eq!(paused.stage, AnalysisTaskStageDto::Deep);
    assert_eq!(paused.state, AnalysisTaskStateDto::Paused);
    std::fs::write(temp.path().join("late-overview"), "go").unwrap();
    std::thread::sleep(Duration::from_millis(100));
    let after_late = manager.analysis_task_snapshot().unwrap();
    assert_eq!(after_late, paused);

    let continued = manager
        .continue_analysis_task(&run_id, &task.task_id, 83)
        .unwrap();
    assert_eq!(continued.stage, AnalysisTaskStageDto::Deep);
    assert_ne!(continued.job_id, task.job_id);
    std::fs::write(temp.path().join("release-reader"), "go").unwrap();
    selected_thread.join().unwrap();
    let completed = wait_task(&manager, AnalysisTaskStateDto::Completed);
    assert_eq!(completed.overview_completed, completed.requested);
    assert_eq!(completed.completed, completed.requested);
    assert_eq!(completed.overview_summaries.len(), 1);
    let delivered = std::fs::read_to_string(temp.path().join("delivered.jsonl")).unwrap();
    assert!(!delivered.contains(&task.job_id));
    assert_eq!(count_job_queries(&delivered, &continued.job_id), 1);
    manager.teardown().unwrap();
}

#[cfg(unix)]
#[test]
fn analysis_task_rejects_invalid_conditions_without_replacing_active_task() {
    let temp = TestTempDir::new("analysis-task-admission");
    let release = temp.path().join("release");
    let (manager, _, _, run_id) = ready_manager(&temp, &echo_first_then_hold_script(&release));
    let active = manager
        .start_analysis_task(
            whole_game_request(&run_id, 41, 2),
            analysis_scope(),
            task_conditions(4),
        )
        .unwrap();
    let mut unsupported = task_conditions(4);
    unsupported.total_visits.enabled = false;
    let error = manager
        .start_analysis_task(whole_game_request(&run_id, 41, 2), analysis_scope(), unsupported)
        .unwrap_err();
    assert_eq!(error.kind, EngineFailureKind::InvalidState);
    assert_eq!(manager.analysis_task_snapshot().unwrap().task_id, active.task_id);
    let error = manager
        .start_all_positions_analysis_task(
            whole_game_request(&run_id, 41, 2),
            analysis_scope(),
            task_conditions(32),
            task_conditions(499),
        )
        .unwrap_err();
    assert_eq!(error.kind, EngineFailureKind::InvalidState);
    assert_eq!(manager.analysis_task_snapshot().unwrap().task_id, active.task_id);
}

#[cfg(unix)]
#[test]
fn analysis_task_cancel_is_terminal_and_late_final_cannot_complete_it() {
    let temp = TestTempDir::new("analysis-task-cancel");
    let release = temp.path().join("release");
    let (manager, _, events, run_id) = ready_manager(&temp, &echo_first_then_hold_script(&release));
    let task = manager
        .start_analysis_task(
            whole_game_request(&run_id, 42, 2),
            analysis_scope(),
            task_conditions(4),
        )
        .unwrap();
    wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == task.job_id && job.outcome == AnalysisJobOutcomeDto::Progress
    });
    manager.cancel_job(&run_id, &task.job_id).unwrap();
    assert_eq!(
        manager.analysis_task_snapshot().unwrap().state,
        AnalysisTaskStateDto::Cancelled
    );
    wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == task.job_id && job.outcome == AnalysisJobOutcomeDto::Cancelled
    });
    std::fs::write(release, "go").unwrap();
    std::thread::sleep(Duration::from_millis(100));
    assert_eq!(
        manager.analysis_task_snapshot().unwrap().state,
        AnalysisTaskStateDto::Cancelled
    );
}

#[cfg(unix)]
#[test]
fn analysis_task_invalidates_only_when_generation_changes() {
    let temp = TestTempDir::new("analysis-task-invalidate");
    let release = temp.path().join("release");
    let (manager, _, events, run_id) = ready_manager(&temp, &echo_first_then_hold_script(&release));
    let task = manager
        .start_analysis_task(
            whole_game_request(&run_id, 43, 2),
            analysis_scope(),
            task_conditions(4),
        )
        .unwrap();
    wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == task.job_id && job.outcome == AnalysisJobOutcomeDto::Progress
    });
    manager.invalidate_analysis_task(43);
    assert_eq!(manager.analysis_task_snapshot().unwrap().task_id, task.task_id);
    assert!(matches!(
        manager.analysis_task_snapshot().unwrap().state,
        AnalysisTaskStateDto::Queued | AnalysisTaskStateDto::Searching
    ));
    manager.invalidate_analysis_task(44);
    let invalidated = manager.analysis_task_snapshot().unwrap();
    assert_eq!(invalidated.state, AnalysisTaskStateDto::Invalidated);
    assert_eq!(invalidated.completed, vec![NodePath { indices: vec![] }]);
}

#[cfg(unix)]
fn task_engine_script(directory: &Path) -> String {
    format!(
        "export TASK_ENGINE_DIR='{}'\nexec python3 -u '{}'\n",
        directory.display(),
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/analysis_task_engine.py")
            .display()
    )
}

#[cfg(unix)]
fn wait_task(
    manager: &ForegroundEngineManager,
    expected: AnalysisTaskStateDto,
) -> app_model::AnalysisTaskDto {
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        let task = manager.analysis_task_snapshot().unwrap();
        if task.state == expected {
            return task;
        }
        assert!(Instant::now() < deadline, "expected {expected:?}, got {task:?}");
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[cfg(unix)]
fn wait_task_stage(
    manager: &ForegroundEngineManager,
    stage: AnalysisTaskStageDto,
    state: AnalysisTaskStateDto,
) -> app_model::AnalysisTaskDto {
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        let task = manager.analysis_task_snapshot().unwrap();
        if task.stage == stage && task.state == state {
            return task;
        }
        assert!(
            Instant::now() < deadline,
            "expected {stage:?}/{state:?}, got {task:?}"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[cfg(unix)]
#[test]
fn analysis_task_queued_pause_never_submits_behind_a_busy_writer() {
    let temp = TestTempDir::new("task-unsent-pause");
    let script = format!(
        r#"exec python3 -u -c '
import sys,json,time,pathlib
root=pathlib.Path("{}")
def emit(q):
    print(json.dumps(dict(id=q["id"],isDuringSearch=False,turnNumber=0,
        rootInfo=dict(visits=2,winrate=0.6),moveInfos=[dict(move="D4",visits=2,winrate=0.6)])),flush=True)
emit(json.loads(sys.stdin.readline()))
while not (root/"release").exists(): time.sleep(0.01)
for line in sys.stdin:
    with (root/"delivered.jsonl").open("a") as log: log.write(line)
    q=json.loads(line)
    if "action" not in q: emit(q)
'"#,
        temp.path().display()
    );
    let (manager, _, _, run_id) = ready_manager(&temp, &script);
    let mut selected = selected_request(&run_id, 42, vec![]);
    selected.query.moves = (0..20000)
        .map(|i| (if i % 2 == 0 { "B" } else { "W" }.into(), "pass".into()))
        .collect();
    let writer = manager.clone();
    let selected_thread = std::thread::spawn(move || {
        writer.start_selected_node_job(selected).unwrap();
    });
    std::thread::sleep(Duration::from_millis(100));
    let task = manager
        .start_analysis_task(
            whole_game_request(&run_id, 42, 1),
            analysis_scope(),
            task_conditions(32),
        )
        .unwrap();
    let paused = manager.pause_analysis_task(&run_id, &task.task_id).unwrap();
    assert_eq!(paused.state, AnalysisTaskStateDto::Paused);
    assert!(paused.completed.is_empty());
    std::fs::write(temp.path().join("release"), "").unwrap();
    selected_thread.join().unwrap();
    std::thread::sleep(Duration::from_millis(100));
    let delivered = std::fs::read_to_string(temp.path().join("delivered.jsonl")).unwrap();
    assert!(
        !delivered.contains(&task.job_id),
        "queued Pause delivered a query or terminate"
    );
    manager.teardown().unwrap();
}

#[cfg(unix)]
#[test]
fn analysis_task_stalled_pipe_keeps_pause_and_run_failure_responsive() {
    let temp = TestTempDir::new("task-stalled-pipe");
    let script = r#"exec python3 -u -c '
import sys,json,time
q=json.loads(sys.stdin.readline())
print(json.dumps(dict(id=q["id"],isDuringSearch=False,turnNumber=0)),flush=True)
time.sleep(8)
'"#;
    let (manager, _, events, run_id) = ready_manager(&temp, script);
    let mut request = whole_game_request(&run_id, 42, 1);
    request.work_items[0].query.moves = (0..20000)
        .map(|i| (if i % 2 == 0 { "B" } else { "W" }.into(), "pass".into()))
        .collect();
    let started = Instant::now();
    let task = manager
        .start_analysis_task(request, analysis_scope(), task_conditions(32))
        .unwrap();
    assert!(
        started.elapsed() < Duration::from_millis(500),
        "Start waited for a blocked pipe"
    );
    std::thread::sleep(Duration::from_millis(100));
    let pause_at = Instant::now();
    assert_eq!(manager.analysis_task_snapshot().unwrap().task_id, task.task_id);
    let pausing = manager.pause_analysis_task(&run_id, &task.task_id).unwrap();
    assert_eq!(pausing.state, AnalysisTaskStateDto::Pausing);
    assert!(pause_at.elapsed() < Duration::from_millis(500));
    let failed = wait_snapshot(&events, Duration::from_secs(7), |state| {
        matches!(state, ForegroundEngineLifecycleDto::Error { .. })
    });
    assert!(pause_at.elapsed() < Duration::from_secs(7));
    assert!(failed.whole_game_job.is_none());
    assert_eq!(
        manager.analysis_task_snapshot().unwrap().state,
        AnalysisTaskStateDto::Failed
    );
    assert!(manager.analysis_task_snapshot().unwrap().completed.is_empty());
    manager.teardown().unwrap();
}

#[cfg(unix)]
#[test]
fn analysis_task_rejected_cancel_preserves_run_job_and_pausing_identity() {
    let temp = TestTempDir::new("task-cancel-identity");
    std::fs::write(temp.path().join("hold-first"), "").unwrap();
    let (manager, _, _, run_id) = ready_manager(&temp, &task_engine_script(temp.path()));
    let task = manager
        .start_analysis_task(
            whole_game_request(&run_id, 42, 1),
            analysis_scope(),
            task_conditions(32),
        )
        .unwrap();
    let before = wait_task(&manager, AnalysisTaskStateDto::Searching);
    for (run, job) in [
        ("wrong-run", task.job_id.as_str()),
        (run_id.as_str(), "wrong-job"),
    ] {
        assert_eq!(
            manager.cancel_job(run, job).unwrap_err().kind,
            EngineFailureKind::InvalidState
        );
        assert_eq!(manager.analysis_task_snapshot().unwrap(), before);
        assert_eq!(manager.snapshot().whole_game_job.unwrap().job_id, task.job_id);
    }
    let pausing = manager.pause_analysis_task(&run_id, &task.task_id).unwrap();
    assert!(manager.cancel_job("wrong-run", &task.job_id).is_err());
    assert_eq!(manager.analysis_task_snapshot().unwrap(), pausing);
    manager.cancel_job(&run_id, &task.job_id).unwrap();
    assert_eq!(
        manager.analysis_task_snapshot().unwrap().state,
        AnalysisTaskStateDto::Cancelled
    );
    std::fs::write(temp.path().join("cancel-final"), "").unwrap();
    manager
        .wait_for_job_cancellation(&run_id, &task.job_id, Duration::from_secs(3))
        .unwrap();
    assert!(manager
        .continue_analysis_task(&run_id, &task.task_id, 42)
        .is_err());
    manager.teardown().unwrap();
}

#[cfg(unix)]
#[test]
fn analysis_task_pause_continue_retains_completed_work_and_restarts_full_budget() {
    let temp = TestTempDir::new("task-pause-continue");
    let (manager, _, events, run_id) = ready_manager(&temp, &task_engine_script(temp.path()));
    let task = manager
        .start_analysis_task(
            whole_game_request(&run_id, 42, 3),
            analysis_scope(),
            task_conditions(32),
        )
        .unwrap();
    wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == task.job_id && job.outcome == AnalysisJobOutcomeDto::Progress
    });
    wait_task(&manager, AnalysisTaskStateDto::Searching);
    let pausing = manager.pause_analysis_task(&run_id, &task.task_id).unwrap();
    assert_eq!(pausing.state, AnalysisTaskStateDto::Pausing);
    assert_eq!(pausing.completed, vec![NodePath::default()]);
    assert!(manager
        .continue_analysis_task(&run_id, &task.task_id, 42)
        .is_err());
    manager.pause_analysis_task(&run_id, &task.task_id).unwrap();
    std::thread::sleep(Duration::from_millis(100));
    assert_eq!(
        manager.analysis_task_snapshot().unwrap().state,
        AnalysisTaskStateDto::Pausing
    );
    std::fs::write(temp.path().join("cancel-final"), "go").unwrap();
    let paused = wait_task(&manager, AnalysisTaskStateDto::Paused);
    assert_eq!(paused.completed, pausing.completed);
    assert!(manager.snapshot().whole_game_job.is_none());
    assert_eq!(
        manager
            .start_analysis_task(
                whole_game_request(&run_id, 42, 1),
                analysis_scope(),
                task_conditions(1)
            )
            .unwrap_err()
            .kind,
        EngineFailureKind::Occupied
    );
    let continued = manager
        .continue_analysis_task(&run_id, &task.task_id, 42)
        .unwrap();
    assert_eq!(continued.task_id, task.task_id);
    assert_eq!(continued.run_id, run_id);
    assert_ne!(continued.job_id, task.job_id);
    assert_eq!(continued.completed, paused.completed);
    assert!(manager
        .continue_analysis_task(&run_id, &task.task_id, 42)
        .is_err());
    std::fs::write(temp.path().join("finish"), "go").unwrap();
    let done = wait_task(&manager, AnalysisTaskStateDto::Completed);
    assert_eq!(done.completed, done.requested);
    let queries: Vec<serde_json::Value> = std::fs::read_to_string(temp.path().join("queries.jsonl"))
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(queries.len(), 4, "completed root must not be searched again");
    assert!(queries
        .iter()
        .all(|query| query["overrideSettings"]["maxVisits"] == 32));
    assert!(queries[0]["id"].as_str().unwrap().starts_with(&task.job_id));
    assert!(queries[1]["id"].as_str().unwrap().starts_with(&task.job_id));
    assert!(queries[2]["id"].as_str().unwrap().starts_with(&continued.job_id));
    assert!(queries[3]["id"].as_str().unwrap().starts_with(&continued.job_id));
    assert_ne!(queries[0]["id"], queries[1]["id"]);
    assert_ne!(queries[2]["id"], queries[3]["id"]);
    assert!(manager
        .continue_analysis_task(&run_id, &task.task_id, 42)
        .is_err());
    manager.teardown().unwrap();
}

#[cfg(unix)]
#[test]
fn analysis_task_pause_is_lane_local_for_one_and_two_thread_capacity() {
    for one_thread in [true, false] {
        let temp = TestTempDir::new("task-pause-lanes");
        std::fs::write(temp.path().join("hold-first"), "").unwrap();
        if one_thread {
            std::fs::write(temp.path().join("one-thread"), "").unwrap();
        }
        let (manager, _, events, run_id) = ready_manager(&temp, &task_engine_script(temp.path()));
        let selected = manager
            .start_selected_node_job(continuous_request(&run_id, 42, vec![]))
            .unwrap();
        wait_job(&events, Duration::from_secs(2), |job| {
            job.job_id == selected.job_id && job.outcome == AnalysisJobOutcomeDto::Progress
        });
        let task = manager
            .start_analysis_task(
                whole_game_request(&run_id, 42, 2),
                analysis_scope(),
                task_conditions(32),
            )
            .unwrap();
        if one_thread {
            std::thread::sleep(Duration::from_millis(100));
            assert_eq!(
                manager.analysis_task_snapshot().unwrap().state,
                AnalysisTaskStateDto::Queued
            );
        } else {
            wait_task(&manager, AnalysisTaskStateDto::Searching);
        }
        manager.pause_analysis_task(&run_id, &task.task_id).unwrap();
        std::fs::write(temp.path().join("cancel-final"), "").unwrap();
        let paused = wait_task(&manager, AnalysisTaskStateDto::Paused);
        assert!(paused.completed.is_empty());
        let query_count = std::fs::read_to_string(temp.path().join("queries.jsonl"))
            .unwrap()
            .lines()
            .count();
        assert_eq!(query_count, 2, "Pause must not submit another search");
        let continued = manager
            .continue_analysis_task(&run_id, &task.task_id, 42)
            .unwrap();
        manager.pause_analysis_task(&run_id, &task.task_id).unwrap();
        wait_task(&manager, AnalysisTaskStateDto::Paused);
        assert_ne!(continued.job_id, task.job_id);
        let frame = wait_job(&events, Duration::from_secs(2), |job| {
            job.job_id == selected.job_id && job.outcome == AnalysisJobOutcomeDto::Progress
        });
        assert!(frame.frame.unwrap().visits > 0);
        assert_eq!(
            manager.snapshot().selected_node_job.unwrap().job_id,
            selected.job_id
        );
        manager.cancel_job(&run_id, &continued.job_id).unwrap();
        assert_eq!(
            manager.analysis_task_snapshot().unwrap().state,
            AnalysisTaskStateDto::Cancelled
        );
        assert!(manager
            .continue_analysis_task(&run_id, &task.task_id, 42)
            .is_err());
        manager.teardown().unwrap();
    }
}

#[cfg(unix)]
#[test]
fn analysis_task_pause_ack_without_final_fails_run_and_both_lanes() {
    let temp = TestTempDir::new("task-pause-timeout");
    std::fs::write(temp.path().join("hold-first"), "").unwrap();
    let (manager, _, events, run_id) = ready_manager(&temp, &task_engine_script(temp.path()));
    manager
        .start_selected_node_job(continuous_request(&run_id, 42, vec![]))
        .unwrap();
    let task = manager
        .start_analysis_task(
            whole_game_request(&run_id, 42, 2),
            analysis_scope(),
            task_conditions(32),
        )
        .unwrap();
    wait_task(&manager, AnalysisTaskStateDto::Searching);
    let started = Instant::now();
    manager.pause_analysis_task(&run_id, &task.task_id).unwrap();
    wait_snapshot(&events, Duration::from_secs(7), |state| {
        matches!(state, ForegroundEngineLifecycleDto::Error { .. })
    });
    assert!(started.elapsed() >= Duration::from_secs(5));
    assert!(started.elapsed() < Duration::from_secs(7));
    assert_eq!(
        manager.analysis_task_snapshot().unwrap().state,
        AnalysisTaskStateDto::Failed
    );
    assert!(manager.snapshot().whole_game_job.is_none());
    assert!(manager.snapshot().selected_node_job.is_none());
    assert!(manager
        .continue_analysis_task(&run_id, &task.task_id, 42)
        .is_err());
    manager.teardown().unwrap();
}

#[cfg(unix)]
#[test]
fn analysis_task_paused_progress_obeys_semantic_run_and_departure_fences() {
    for ending in ["semantic", "restart", "switch", "stop", "departure"] {
        let temp = TestTempDir::new("task-paused-invalidation");
        std::fs::write(temp.path().join("cancel-final"), "").unwrap();
        let (manager, catalog, events, run_id) = ready_manager(&temp, &task_engine_script(temp.path()));
        let task = manager
            .start_analysis_task(
                whole_game_request(&run_id, 42, 2),
                analysis_scope(),
                task_conditions(32),
            )
            .unwrap();
        wait_job(&events, Duration::from_secs(2), |job| {
            job.outcome == AnalysisJobOutcomeDto::Progress
        });
        manager.pause_analysis_task(&run_id, &task.task_id).unwrap();
        let paused = wait_task(&manager, AnalysisTaskStateDto::Paused);
        manager.invalidate_analysis_task(42);
        catalog.upsert(SavedEngineProfile {
            profile_id: "broken".into(),
            profile: EngineProfileDto {
                program: "/missing/task-engine".into(),
                ..catalog.get("profile-1").unwrap().profile
            },
        });
        manager.switch_to("broken").unwrap();
        wait_failure(&events, Duration::from_secs(3), |error| {
            error.operation == EngineOperationDto::Switch
        });
        assert_eq!(manager.analysis_task_snapshot().unwrap(), paused);
        match ending {
            "semantic" => manager.invalidate_analysis_task(43),
            "restart" => {
                manager.restart().unwrap();
                wait_lifecycle(&manager, Duration::from_secs(3), |state| {
                    matches!(state, ForegroundEngineLifecycleDto::Ready { .. })
                });
            }
            "switch" => {
                catalog.upsert(SavedEngineProfile {
                    profile_id: "replacement".into(),
                    profile: catalog.get("profile-1").unwrap().profile,
                });
                manager.switch_to("replacement").unwrap();
                wait_lifecycle(&manager, Duration::from_secs(3), |state| {
                    matches!(state, ForegroundEngineLifecycleDto::Ready { .. })
                });
            }
            "stop" => {
                manager.stop().unwrap();
            }
            "departure" => {
                manager
                    .set_continuous_preferences(true, app_model::ContinuousAnalysisBudgetDto::default())
                    .unwrap();
                manager.begin_continuous_departure();
                manager.finish_continuous_departure(false);
                assert_eq!(
                    manager.snapshot().continuous.phase,
                    app_model::ContinuousAnalysisPhaseDto::SafetyHold
                );
            }
            _ => unreachable!(),
        }
        let ended = manager.analysis_task_snapshot().unwrap();
        assert_eq!(
            ended.state,
            if ending == "departure" {
                AnalysisTaskStateDto::Cancelled
            } else {
                AnalysisTaskStateDto::Invalidated
            }
        );
        assert_eq!(ended.completed, paused.completed);
        assert!(manager
            .continue_analysis_task(&run_id, &task.task_id, 42)
            .is_err());
        manager.teardown().unwrap();
    }
}

#[cfg(unix)]
fn budget_conditions(
    time: Option<u32>,
    total: Option<u32>,
    leading: Option<u32>,
) -> AnalysisStageConditionsDto {
    AnalysisStageConditionsDto {
        time_seconds: AnalysisTaskLimitDto {
            enabled: time.is_some(),
            value: time.unwrap_or(10),
        },
        total_visits: AnalysisTaskLimitDto {
            enabled: total.is_some(),
            value: total.unwrap_or(800),
        },
        leading_candidate_visits: AnalysisTaskLimitDto {
            enabled: leading.is_some(),
            value: leading.unwrap_or(500),
        },
    }
}

#[cfg(unix)]
fn wait_for_file(path: &Path) {
    let deadline = Instant::now() + Duration::from_secs(2);
    while !path.exists() {
        assert!(
            Instant::now() < deadline,
            "timed out waiting for {}",
            path.display()
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[cfg(unix)]
#[test]
fn analysis_task_engine_time_starts_after_query_admission() {
    let temp = TestTempDir::new("task-time-budget");
    std::fs::write(temp.path().join("budget-smoke"), "time_seconds").unwrap();
    let (manager, _, _, run_id) = ready_manager(&temp, &task_engine_script(temp.path()));
    let admitted = Instant::now();
    manager
        .start_analysis_task(
            whole_game_request(&run_id, 70, 1),
            analysis_scope(),
            budget_conditions(Some(1), None, None),
        )
        .unwrap();

    let completed = wait_task(&manager, AnalysisTaskStateDto::Completed);

    assert!(admitted.elapsed() >= Duration::from_millis(900));
    assert_eq!(completed.ending_conditions, vec!["time_seconds"]);
    manager.teardown().unwrap();
}

#[cfg(unix)]
#[test]
fn analysis_task_total_budget_waits_for_target_final_not_control_ack() {
    let temp = TestTempDir::new("task-total-budget-cleanup");
    std::fs::write(temp.path().join("budget-smoke"), "total_visits").unwrap();
    let (manager, _, _, run_id) = ready_manager(&temp, &task_engine_script(temp.path()));
    manager
        .start_analysis_task(
            whole_game_request(&run_id, 71, 2),
            analysis_scope(),
            budget_conditions(None, Some(8), None),
        )
        .unwrap();
    wait_for_file(&temp.path().join("cancel-seen"));

    let cleaning = manager.analysis_task_snapshot().unwrap();
    assert!(cleaning.completed.is_empty());
    assert_eq!(
        std::fs::read_to_string(temp.path().join("queries.jsonl"))
            .unwrap()
            .lines()
            .count(),
        1
    );
    assert!(cleaning
        .reason
        .as_deref()
        .is_some_and(|reason| reason.contains("total_visits")));
    std::fs::write(temp.path().join("cancel-final"), "go").unwrap();
    let completed = wait_task(&manager, AnalysisTaskStateDto::Completed);
    assert_eq!(completed.ending_conditions, vec!["total_visits"]);
    assert_eq!(completed.completed.len(), 2);
    manager.teardown().unwrap();
}

#[cfg(unix)]
#[test]
fn analysis_task_malformed_budget_final_fails_run_without_releasing_live_query() {
    let temp = TestTempDir::new("task-malformed-budget-final");
    std::fs::write(temp.path().join("budget-smoke"), "total_visits").unwrap();
    std::fs::write(temp.path().join("malformed-final"), "").unwrap();
    std::fs::write(temp.path().join("cancel-final"), "").unwrap();
    let (manager, _, _, run_id) = ready_manager(&temp, &task_engine_script(temp.path()));
    manager
        .start_analysis_task(
            whole_game_request(&run_id, 71, 1),
            analysis_scope(),
            budget_conditions(None, Some(8), None),
        )
        .unwrap();
    let failed = wait_task(&manager, AnalysisTaskStateDto::Failed);
    assert!(failed.completed.is_empty());
    assert!(matches!(
        manager.snapshot().lifecycle,
        ForegroundEngineLifecycleDto::Error { .. }
    ));
    manager.teardown().unwrap();
}

#[cfg(unix)]
#[test]
fn review_semantic_budget_final_keeps_run_failure_cleanup() {
    for variant in ["semantic", "no_results", "unmarked"] {
        let temp = TestTempDir::new("task-semantic-budget-final");
        std::fs::write(temp.path().join("budget-smoke"), "total_visits").unwrap();
        std::fs::write(temp.path().join("malformed-final"), variant).unwrap();
        std::fs::write(temp.path().join("cancel-final"), "").unwrap();
        let (manager, _, events, run_id) = ready_manager(&temp, &task_engine_script(temp.path()));
        let task = manager
            .start_analysis_task(
                whole_game_request(&run_id, 71, 2),
                analysis_scope(),
                budget_conditions(None, Some(8), None),
            )
            .unwrap();
        let terminal = wait_job(&events, Duration::from_secs(3), |job| {
            job.job_id == task.job_id
                && matches!(
                    job.outcome,
                    AnalysisJobOutcomeDto::Failed | AnalysisJobOutcomeDto::Completed
                )
        });
        assert_eq!(terminal.outcome, AnalysisJobOutcomeDto::Failed, "{variant}");
        assert!(terminal.frame.is_none());
        assert!(manager.analysis_task_snapshot().unwrap().completed.is_empty());
        assert!(
            matches!(
                manager.snapshot().lifecycle,
                ForegroundEngineLifecycleDto::Error { .. }
            ),
            "{variant}"
        );
        assert_eq!(
            std::fs::read_to_string(temp.path().join("queries.jsonl"))
                .unwrap()
                .lines()
                .count(),
            1
        );
        manager.teardown().unwrap();
    }
}

#[cfg(unix)]
#[test]
fn review_unmarked_natural_response_cannot_infer_time_completion() {
    let temp = TestTempDir::new("task-unmarked-natural-final");
    std::fs::write(temp.path().join("budget-smoke"), "unmarked").unwrap();
    let (manager, _, events, run_id) = ready_manager(&temp, &task_engine_script(temp.path()));
    let task = manager
        .start_analysis_task(
            whole_game_request(&run_id, 71, 1),
            analysis_scope(),
            budget_conditions(Some(1), None, None),
        )
        .unwrap();
    let terminal = wait_job(&events, Duration::from_secs(3), |job| {
        job.job_id == task.job_id
            && matches!(
                job.outcome,
                AnalysisJobOutcomeDto::Failed | AnalysisJobOutcomeDto::Completed
            )
    });
    assert_eq!(terminal.outcome, AnalysisJobOutcomeDto::Failed);
    assert!(terminal.frame.is_none());
    assert!(manager.analysis_task_snapshot().unwrap().completed.is_empty());
    manager.teardown().unwrap();
}

#[cfg(unix)]
#[test]
fn root_only_one_visit_final_completes_without_fabricating_candidates() {
    let temp = TestTempDir::new("task-root-only-one-visit");
    std::fs::write(temp.path().join("budget-smoke"), "root_only_one_visit").unwrap();
    let (manager, _, events, run_id) = ready_manager(&temp, &task_engine_script(temp.path()));
    let task = manager
        .start_analysis_task(
            whole_game_request(&run_id, 71, 1),
            analysis_scope(),
            budget_conditions(None, Some(1), None),
        )
        .unwrap();

    let progress = wait_job(&events, Duration::from_secs(3), |job| {
        job.job_id == task.job_id
            && job.outcome == AnalysisJobOutcomeDto::Progress
            && job.frame.as_ref().is_some_and(|frame| frame.visits == 1)
    });
    let frame = progress.frame.expect("root-only analysis frame");
    assert_eq!(frame.visits, 1);
    assert!(frame.candidates.is_empty());
    let completed = wait_task(&manager, AnalysisTaskStateDto::Completed);
    assert_eq!(completed.completed.len(), 1);
    assert_eq!(completed.ending_conditions, vec!["total_visits"]);
    manager.teardown().unwrap();
}

#[cfg(unix)]
#[test]
fn review_natural_final_cannot_retarget_deferred_termination() {
    let temp = TestTempDir::new("task-natural-final-termination-race");
    std::fs::write(temp.path().join("budget-smoke"), "natural_race").unwrap();
    std::fs::write(temp.path().join("cancel-final"), "").unwrap();
    let (manager, _, events, run_id) = ready_manager(&temp, &task_engine_script(temp.path()));
    for _ in 0..32 {
        let task = manager
            .start_analysis_task(
                whole_game_request(&run_id, 71, 2),
                analysis_scope(),
                budget_conditions(None, Some(8), None),
            )
            .unwrap();
        let terminal = wait_job(&events, Duration::from_secs(3), |job| {
            job.job_id == task.job_id
                && matches!(
                    job.outcome,
                    AnalysisJobOutcomeDto::Failed | AnalysisJobOutcomeDto::Completed
                )
        });
        assert!(
            !temp.path().join("stale-terminate").exists(),
            "a previous target's termination reached the next target"
        );
        assert_eq!(terminal.outcome, AnalysisJobOutcomeDto::Completed);
        assert_eq!(manager.analysis_task_snapshot().unwrap().completed.len(), 2);
    }
    manager.teardown().unwrap();
}

#[cfg(unix)]
#[test]
fn analysis_task_invalid_candidate_report_cannot_trigger_budget_termination() {
    let temp = TestTempDir::new("task-invalid-budget-report");
    std::fs::write(temp.path().join("budget-smoke"), "invalid_report").unwrap();
    let (manager, _, _, run_id) = ready_manager(&temp, &task_engine_script(temp.path()));
    manager
        .start_analysis_task(
            whole_game_request(&run_id, 71, 1),
            analysis_scope(),
            budget_conditions(None, None, Some(8)),
        )
        .unwrap();
    wait_for_file(&temp.path().join("invalid-sent"));
    std::thread::sleep(Duration::from_millis(100));
    assert!(!temp.path().join("cancel-seen").exists());
    assert!(manager.analysis_task_snapshot().unwrap().completed.is_empty());
    std::fs::write(temp.path().join("finish"), "").unwrap();
    let completed = wait_task(&manager, AnalysisTaskStateDto::Completed);
    assert_eq!(completed.ending_conditions, vec!["leading_candidate_visits"]);
    manager.teardown().unwrap();
}
#[cfg(unix)]
#[test]
fn analysis_task_time_budget_excludes_engine_queue_wait() {
    let temp = TestTempDir::new("task-time-budget-queue");
    std::fs::write(temp.path().join("budget-smoke"), "time_seconds").unwrap();
    std::fs::write(temp.path().join("one-thread"), "").unwrap();
    std::fs::write(temp.path().join("hold-first"), "").unwrap();
    let (manager, _, events, run_id) = ready_manager(&temp, &task_engine_script(temp.path()));
    let blocker = manager
        .start_selected_node_job(continuous_request(&run_id, 70, vec![]))
        .unwrap();
    wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == blocker.job_id && job.outcome == AnalysisJobOutcomeDto::Progress
    });
    manager
        .start_analysis_task(
            whole_game_request(&run_id, 70, 1),
            analysis_scope(),
            budget_conditions(Some(1), None, None),
        )
        .unwrap();
    std::thread::sleep(Duration::from_millis(1200));
    assert_eq!(
        manager.analysis_task_snapshot().unwrap().state,
        AnalysisTaskStateDto::Queued
    );
    manager.cancel_job(&run_id, &blocker.job_id).unwrap();
    std::fs::write(temp.path().join("cancel-final"), "go").unwrap();
    wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == blocker.job_id && job.outcome == AnalysisJobOutcomeDto::Cancelled
    });

    let released = Instant::now();
    let completed = wait_task(&manager, AnalysisTaskStateDto::Completed);
    assert!(released.elapsed() >= Duration::from_millis(900));
    assert_eq!(completed.ending_conditions, vec!["time_seconds"]);
    manager.teardown().unwrap();
}

#[cfg(unix)]
#[test]
fn analysis_task_leading_budget_tracks_current_order_zero_candidate() {
    let temp = TestTempDir::new("task-leading-budget");
    std::fs::write(temp.path().join("budget-smoke"), "leading_candidate_visits").unwrap();
    let (manager, _, _, run_id) = ready_manager(&temp, &task_engine_script(temp.path()));
    manager
        .start_analysis_task(
            whole_game_request(&run_id, 72, 1),
            analysis_scope(),
            budget_conditions(None, None, Some(5)),
        )
        .unwrap();
    wait_for_file(&temp.path().join("cancel-seen"));
    std::fs::write(temp.path().join("cancel-final"), "go").unwrap();

    let completed = wait_task(&manager, AnalysisTaskStateDto::Completed);
    assert_eq!(completed.ending_conditions, vec!["leading_candidate_visits"]);
    manager.teardown().unwrap();
}

#[cfg(unix)]
#[test]
fn analysis_task_reports_all_conditions_from_same_first_observation() {
    let temp = TestTempDir::new("task-budget-or-tie");
    std::fs::write(temp.path().join("budget-smoke"), "total_visits").unwrap();
    let (manager, _, _, run_id) = ready_manager(&temp, &task_engine_script(temp.path()));
    manager
        .start_analysis_task(
            whole_game_request(&run_id, 73, 1),
            analysis_scope(),
            budget_conditions(Some(10), Some(8), Some(8)),
        )
        .unwrap();
    wait_for_file(&temp.path().join("cancel-seen"));
    std::fs::write(temp.path().join("cancel-final"), "go").unwrap();

    let completed = wait_task(&manager, AnalysisTaskStateDto::Completed);
    assert_eq!(
        completed.ending_conditions,
        vec!["total_visits", "leading_candidate_visits"]
    );
    manager.teardown().unwrap();
}

#[cfg(unix)]
#[test]
fn pause_overrides_budget_stop_before_final_and_fences_completion() {
    let temp = TestTempDir::new("task-budget-pause-race");
    std::fs::write(temp.path().join("budget-smoke"), "total_visits").unwrap();
    let (manager, _, _, run_id) = ready_manager(&temp, &task_engine_script(temp.path()));
    let task = manager
        .start_analysis_task(
            whole_game_request(&run_id, 74, 1),
            analysis_scope(),
            budget_conditions(None, Some(8), None),
        )
        .unwrap();
    wait_for_file(&temp.path().join("cancel-seen"));
    let pausing = manager.pause_analysis_task(&run_id, &task.task_id).unwrap();
    assert_eq!(pausing.state, AnalysisTaskStateDto::Pausing);
    std::fs::write(temp.path().join("cancel-final"), "go").unwrap();

    let paused = wait_task(&manager, AnalysisTaskStateDto::Paused);
    assert!(paused.completed.is_empty());
    assert!(paused.ending_conditions.is_empty());
    manager.teardown().unwrap();
}

#[cfg(unix)]
#[test]
fn analysis_task_fails_when_engine_ignores_required_budget_settings() {
    let temp = TestTempDir::new("task-ignored-budget-setting");
    std::fs::write(temp.path().join("budget-smoke"), "ignored_setting").unwrap();
    let (manager, _, _, run_id) = ready_manager(&temp, &task_engine_script(temp.path()));
    manager
        .start_analysis_task(
            whole_game_request(&run_id, 75, 1),
            analysis_scope(),
            budget_conditions(Some(1), None, None),
        )
        .unwrap();

    let failed = wait_task(&manager, AnalysisTaskStateDto::Failed);
    assert!(failed.completed.is_empty());
    assert!(matches!(
        manager.snapshot().lifecycle,
        ForegroundEngineLifecycleDto::Error { .. }
    ));
    manager.teardown().unwrap();
}

#[cfg(unix)]
#[test]
fn budget_cleanup_deadline_fails_run_without_completing_position() {
    let temp = TestTempDir::new("task-budget-final-timeout");
    std::fs::write(temp.path().join("budget-smoke"), "total_visits").unwrap();
    let (manager, _, events, run_id) = ready_manager(&temp, &task_engine_script(temp.path()));
    manager
        .start_analysis_task(
            whole_game_request(&run_id, 75, 1),
            analysis_scope(),
            budget_conditions(None, Some(8), None),
        )
        .unwrap();
    wait_for_file(&temp.path().join("cancel-seen"));

    wait_snapshot(&events, Duration::from_secs(7), |state| {
        matches!(state, ForegroundEngineLifecycleDto::Error { .. })
    });
    let failed = manager.analysis_task_snapshot().unwrap();
    assert_eq!(failed.state, AnalysisTaskStateDto::Failed);
    assert!(failed.completed.is_empty());
    assert!(failed.ending_conditions.is_empty());
    manager.teardown().unwrap();
}

#[cfg(unix)]
#[test]
fn cancel_overrides_budget_stop_before_final_and_stays_terminal() {
    let temp = TestTempDir::new("task-budget-cancel-race");
    std::fs::write(temp.path().join("budget-smoke"), "leading_candidate_visits").unwrap();
    let (manager, _, _, run_id) = ready_manager(&temp, &task_engine_script(temp.path()));
    let task = manager
        .start_analysis_task(
            whole_game_request(&run_id, 75, 1),
            analysis_scope(),
            budget_conditions(None, None, Some(5)),
        )
        .unwrap();
    wait_for_file(&temp.path().join("cancel-seen"));
    manager.cancel_job(&run_id, &task.job_id).unwrap();
    assert_eq!(
        manager.analysis_task_snapshot().unwrap().state,
        AnalysisTaskStateDto::Cancelled
    );
    std::fs::write(temp.path().join("cancel-final"), "go").unwrap();
    manager
        .wait_for_job_cancellation(&run_id, &task.job_id, Duration::from_secs(2))
        .unwrap();

    let cancelled = manager.analysis_task_snapshot().unwrap();
    assert_eq!(cancelled.state, AnalysisTaskStateDto::Cancelled);
    assert!(cancelled.completed.is_empty());
    assert!(cancelled.ending_conditions.is_empty());
    manager.teardown().unwrap();
}

#[cfg(unix)]
#[test]
fn analysis_task_rejects_duplicate_and_late_reports_from_prior_position_identity() {
    let temp = TestTempDir::new("task-late-position-report");
    std::fs::write(temp.path().join("budget-smoke"), "duplicate_late").unwrap();
    let (manager, _, events, run_id) = ready_manager(&temp, &task_engine_script(temp.path()));
    let task = manager
        .start_analysis_task(
            whole_game_request(&run_id, 75, 2),
            analysis_scope(),
            budget_conditions(None, Some(8), None),
        )
        .unwrap();

    let completed = wait_task(&manager, AnalysisTaskStateDto::Completed);
    assert_eq!(completed.completed, completed.requested);
    assert_eq!(completed.ending_conditions, vec!["total_visits"]);
    let progress = collect_job_events(&events, Instant::now() + Duration::from_millis(100))
        .into_iter()
        .filter(|job| job.job_id == task.job_id && job.outcome == AnalysisJobOutcomeDto::Progress)
        .collect::<Vec<_>>();
    assert!(
        progress.len() <= 2,
        "duplicates must not publish or double-count: {progress:?}"
    );
    let queries = std::fs::read_to_string(temp.path().join("queries.jsonl")).unwrap();
    let identities = queries
        .lines()
        .map(|line| {
            serde_json::from_str::<serde_json::Value>(line).unwrap()["id"]
                .as_str()
                .unwrap()
                .to_owned()
        })
        .collect::<Vec<_>>();
    assert_eq!(identities.len(), 2);
    assert_ne!(identities[0], identities[1]);
    assert!(identities
        .iter()
        .all(|identity| identity.starts_with(&task.job_id)));
    manager.teardown().unwrap();
}

#[cfg(unix)]
#[test]
fn whole_game_user_cancel_keeps_run_ready_and_does_not_cancel_selected_node() {
    let temp = TestTempDir::new("whole-game-cancel");
    let log = temp.path().join("engine.log");
    let release = temp.path().join("release");
    let selected_cancelled = temp.path().join("selected-cancelled");
    let mut script = format!("ENGINE_LOG='{}'\n", log.display());
    script.push_str(&echo_first_then_hold_script(&release));
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
    let progress = wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == started.job_id
            && job.outcome == AnalysisJobOutcomeDto::Progress
            && job.completed == Some(1)
    });
    assert!(progress.frame.is_some());
    assert_eq!(progress.remaining, Some(1));
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
      if [ "${{WG_FIRST:-1}}" = 1 ]; then
        WG_FIRST=0
        printf '{{"id":"%s","isDuringSearch":false,"turnNumber":0,"rootInfo":{{"visits":4,"winrate":0.5}},"moveInfos":[{{"move":"E5","visits":4,"winrate":0.5}}]}}\n' "$id"
      else
        while [ ! -f '{release}' ]; do sleep 0.01; done
        printf '{{"id":"%s","isDuringSearch":false,"turnNumber":1,"rootInfo":{{"visits":4,"winrate":0.5}},"moveInfos":[{{"move":"E5","visits":4,"winrate":0.5}}]}}\n' "$id"
      fi
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
    assert_eq!(
        manager.analysis_task_snapshot().unwrap().state,
        AnalysisTaskStateDto::Invalidated
    );
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
      if [ "${{WG_FIRST:-1}}" = 1 ]; then
        WG_FIRST=0
        printf '{{"id":"%s","isDuringSearch":false,"turnNumber":0,"rootInfo":{{"visits":4,"winrate":0.5}},"moveInfos":[{{"move":"E5","visits":4,"winrate":0.5}}]}}\n' "$id"
      else
        while [ ! -f '{crash}' ]; do sleep 0.01; done
        exit 9
      fi
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
            .analysis
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

#[test]
fn empty_whole_game_worklist_is_rejected_before_run_admission() {
    let catalog = Arc::new(InMemoryEngineProfileCatalog::new());
    let manager = ForegroundEngineManager::new(catalog, ForegroundEngineConfig::for_tests());
    let err = manager
        .start_whole_game_analysis(WholeGameJobRequest {
            run_id: "run-empty".into(),
            generation: 1,
            work_items: Vec::new(),
        })
        .unwrap_err();
    assert_eq!(err.kind, EngineFailureKind::InvalidState);
    assert_eq!(err.run_id.as_deref(), Some("run-empty"));
}

#[cfg(unix)]
#[test]
fn whole_game_progress_publishes_exact_node_path_and_writes_one_query_per_node() {
    let temp = TestTempDir::new("whole-game-exact-nodes");
    let log = temp.path().join("engine.log");
    let mut script = format!("ENGINE_LOG='{}'\n", log.display());
    script.push_str(&resident_echo_script());
    let (manager, _, events, run_id) = ready_manager(&temp, &script);
    let mut root_query = sample_query();
    root_query.initial_stones = vec![("B".into(), "A1".into()), ("W".into(), "C3".into())];
    root_query.komi = 0.5;
    root_query.board_x_size = 5;
    root_query.board_y_size = 5;
    let mut child_query = sample_query();
    child_query.initial_stones = vec![
        ("B".into(), "A1".into()),
        ("W".into(), "C3".into()),
        ("W".into(), "D4".into()),
    ];
    child_query.komi = 0.5;
    child_query.board_x_size = 5;
    child_query.board_y_size = 5;
    let started = manager
        .start_whole_game_analysis(WholeGameJobRequest {
            run_id: run_id.clone(),
            generation: 21,
            work_items: vec![
                WholeGameWorkItem {
                    node_path: NodePath { indices: Vec::new() },
                    query: root_query,
                    board_width: 5,
                    board_height: 5,
                    move_number: 0,
                },
                WholeGameWorkItem {
                    node_path: NodePath { indices: vec![0] },
                    query: child_query,
                    board_width: 5,
                    board_height: 5,
                    move_number: 1,
                },
            ],
        })
        .unwrap();
    let first = wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == started.job_id
            && job.outcome == AnalysisJobOutcomeDto::Progress
            && job.completed == Some(1)
    });
    assert_eq!(first.node_path.indices, Vec::<u32>::new());
    assert_eq!(first.remaining, Some(1));
    assert_eq!(first.frame.as_ref().map(|frame| frame.turn), Some(0));
    let second = wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == started.job_id
            && job.outcome == AnalysisJobOutcomeDto::Progress
            && job.completed == Some(2)
    });
    assert_eq!(second.node_path.indices, vec![0]);
    assert_eq!(second.remaining, Some(0));
    assert_eq!(second.frame.as_ref().map(|frame| frame.turn), Some(1));
    wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == started.job_id && job.outcome == AnalysisJobOutcomeDto::Completed
    });
    let logged = std::fs::read_to_string(&log).unwrap();
    assert_eq!(count_job_queries(&logged, &started.job_id), 2);
    assert!(logged.contains("\"initialStones\""));
    assert!(logged.contains("A1"));
    assert!(logged.contains("C3"));
    assert!(logged.contains("D4"));
    assert!(
        !logged.contains("B5"),
        "sibling variation must not enter first-child work items"
    );
}

#[cfg(unix)]
#[test]
fn whole_game_timeout_stops_remaining_work_and_keeps_completed_progress() {
    let temp = TestTempDir::new("whole-game-timeout");
    let log = temp.path().join("engine.log");
    let release = temp.path().join("never-release");
    let mut script = format!("ENGINE_LOG='{}'\n", log.display());
    script.push_str(&echo_first_then_hold_script(&release));
    let (manager, _, events, run_id) = ready_manager(&temp, &script);
    let started = manager
        .start_whole_game_analysis(whole_game_request(&run_id, 22, 2))
        .unwrap();
    let progress = wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == started.job_id
            && job.outcome == AnalysisJobOutcomeDto::Progress
            && job.completed == Some(1)
    });
    assert!(progress.frame.is_some());
    let timed_out = wait_job(&events, Duration::from_secs(4), |job| {
        job.job_id == started.job_id && job.outcome == AnalysisJobOutcomeDto::Timeout
    });
    assert_eq!(timed_out.completed, Some(1));
    assert_eq!(timed_out.expected, Some(2));
    assert_eq!(timed_out.remaining, Some(1));
    assert!(manager.snapshot().whole_game_job.is_none());
    let logged = std::fs::read_to_string(&log).unwrap();
    assert_eq!(count_job_queries(&logged, &started.job_id), 2);
}

#[cfg(unix)]
#[test]
fn whole_game_protocol_error_stops_remaining_work() {
    let temp = TestTempDir::new("whole-game-protocol");
    let log = temp.path().join("engine.log");
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
      printf '{{"id":"%s","error":"engine failed"}}\n' "$id"
      ;;
  esac
done
"#,
        log = log.display(),
    );
    let (manager, _, events, run_id) = ready_manager(&temp, &script);
    let started = manager
        .start_whole_game_analysis(whole_game_request(&run_id, 23, 2))
        .unwrap();
    let failed = wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == started.job_id && job.outcome == AnalysisJobOutcomeDto::Failed
    });
    assert_eq!(failed.completed, Some(0));
    assert_eq!(failed.remaining, Some(2));
    assert_eq!(
        failed.failure.as_ref().map(|failure| failure.kind),
        Some(EngineFailureKind::Protocol)
    );
    let task = manager.analysis_task_snapshot().unwrap();
    assert_eq!(task.state, AnalysisTaskStateDto::Failed);
    assert!(task.completed.is_empty());
    let logged = std::fs::read_to_string(&log).unwrap();
    assert_eq!(count_job_queries(&logged, &started.job_id), 1);
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
        program: format!("/definitely/missing/{name}-katago"),
        argv: vec![],
        working_dir: Some(temp.path().to_string_lossy().into_owned()),
        adapter: EngineAdapterSettings::KataGoAnalysis(KataGoSettings {
            model_path: Some(format!("/definitely/missing/{name}-model.bin")),
            config_path: Some(format!("/definitely/missing/{name}.cfg")),
            max_visits: 800,
        }),
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
        program: not_binary.to_string_lossy().into_owned(),
        argv: vec![],
        working_dir: Some(temp.path().to_string_lossy().into_owned()),
        adapter: EngineAdapterSettings::KataGoAnalysis(KataGoSettings {
            model_path: Some(model_path.to_string_lossy().into_owned()),
            config_path: Some(config_path.to_string_lossy().into_owned()),
            max_visits: 800,
        }),
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
        program: engine_path.to_string_lossy().into_owned(),
        argv: vec![],
        working_dir: Some(temp.path().to_string_lossy().into_owned()),
        adapter: EngineAdapterSettings::KataGoAnalysis(KataGoSettings {
            model_path: Some(model_path.to_string_lossy().into_owned()),
            config_path: Some(config_path.to_string_lossy().into_owned()),
            max_visits: 800,
        }),
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
  printf '{{"id":"%s","isDuringSearch":false,"turnNumber":0,"rootInfo":{{"visits":4,"winrate":0.5}},"moveInfos":[{{"move":"E5","visits":4,"winrate":0.5}}]}}\n' "$id"
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

#[cfg(unix)]
#[test]
fn durable_reorder_and_failure_preserve_ready_run_launch_and_job_bindings() {
    struct DiskCatalog(PathBuf);
    impl EngineProfileCatalog for DiskCatalog {
        fn get(&self, id: &str) -> Option<SavedEngineProfile> {
            engine_manager::load_engine_profiles(&self.0)
                .ok()?
                .profiles
                .into_iter()
                .find(|record| record.id == id)
                .map(|record| SavedEngineProfile {
                    profile_id: record.id,
                    profile: record.profile,
                })
        }
        fn autoload_profile_id(&self) -> Option<String> {
            engine_manager::load_engine_profiles(&self.0)
                .ok()?
                .startup_profile_id().map(str::to_owned)
        }
    }
    let temp = TestTempDir::new("order-ready-job");
    let path = temp.path().join("catalog.json");
    let current = engine_manager::EngineProfilesSettings {
        version: 2,
        selected_profile_id: "profile-b".into(),
        startup: app_model::EngineStartupPolicyDto::Fixed { profile_id: "profile-a".into() },
        last_primary_profile_id: None,
        startup_evaluation: Default::default(),
        profiles: vec![
            engine_manager::EngineProfileRecord {
                id: "profile-a".into(),
                preload: false,
                profile: setup_named_profile(&temp, "a", &hold_after_probe_script()),
            },
            engine_manager::EngineProfileRecord {
                id: "profile-b".into(),
                preload: false,
                profile: setup_named_profile(&temp, "b", &resident_echo_script()),
            },
        ],
    };
    engine_manager::save_engine_profiles(&path, current.clone()).unwrap();
    let manager = ForegroundEngineManager::new(
        Arc::new(DiskCatalog(path.clone())),
        ForegroundEngineConfig::for_tests(),
    );
    let events = manager.subscribe();
    manager.start("profile-a").unwrap();
    let ready = wait_snapshot(&events, Duration::from_secs(3), |phase| {
        matches!(phase, ForegroundEngineLifecycleDto::Ready { .. })
    });
    let run = run_from_ready(&ready.lifecycle);
    let job = manager
        .start_selected_node_job(selected_request(&run.run_id, 17, vec![0]))
        .unwrap();
    let before = manager.snapshot();
    assert_eq!(before.selected_node_job.as_ref().unwrap().job_id, job.job_id);
    let request = app_model::EngineProfileOrderRequestDto {
        expected_profile_ids: vec!["profile-a".into(), "profile-b".into()],
        profile_ids: vec!["profile-b".into(), "profile-a".into()],
    };
    let reordered = engine_manager::reorder_engine_profiles(&path, current.clone(), &request).unwrap();
    assert_eq!(engine_manager::load_engine_profiles(&path).unwrap(), reordered);
    assert_eq!(reordered.selected_profile_id, "profile-b");
    assert_eq!(reordered.startup_profile_id(), Some("profile-a"));
    assert_eq!(manager.snapshot(), before);
    let bytes = std::fs::read(&path).unwrap();
    std::fs::create_dir(path.with_extension("json.tmp")).unwrap();
    let reverse = app_model::EngineProfileOrderRequestDto {
        expected_profile_ids: request.profile_ids,
        profile_ids: request.expected_profile_ids,
    };
    assert!(engine_manager::reorder_engine_profiles(&path, reordered.clone(), &reverse).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    assert_eq!(engine_manager::load_engine_profiles(&path).unwrap(), reordered);
    assert_eq!(manager.snapshot(), before);
    manager.stop().unwrap();
}

#[test]
fn apply_autoload_without_mark_stays_no_engine() {
    let catalog = Arc::new(InMemoryEngineProfileCatalog::new());
    catalog.upsert(SavedEngineProfile {
        profile_id: "profile-1".into(),
        profile: EngineProfileDto {
            name: "Unused".into(),
            program: "/bin/unused".into(),
            argv: vec![],
            working_dir: None,
            adapter: EngineAdapterSettings::KataGoAnalysis(KataGoSettings {
                model_path: None,
                config_path: None,
                max_visits: 800,
            }),
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
            program: "/definitely/missing/katago".into(),
            argv: vec![],
            working_dir: None,
            adapter: EngineAdapterSettings::KataGoAnalysis(KataGoSettings {
                model_path: Some("/definitely/missing/model.bin".into()),
                config_path: Some("/definitely/missing/analysis.cfg".into()),
                max_visits: 800,
            }),
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
            program: "/definitely/missing/katago".into(),
            argv: vec![],
            working_dir: None,
            adapter: EngineAdapterSettings::KataGoAnalysis(KataGoSettings {
                model_path: Some("/definitely/missing/model.bin".into()),
                config_path: Some("/definitely/missing/analysis.cfg".into()),
                max_visits: 800,
            }),
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
            program: "/definitely/missing/katago".into(),
            argv: vec![],
            working_dir: None,
            adapter: EngineAdapterSettings::KataGoAnalysis(KataGoSettings {
                model_path: Some("/definitely/missing/model.bin".into()),
                config_path: Some("/definitely/missing/analysis.cfg".into()),
                max_visits: 800,
            }),
        },
    });
    catalog.upsert(SavedEngineProfile {
        profile_id: "bad-start".into(),
        profile: EngineProfileDto {
            name: "Broken Start".into(),
            program: "/definitely/missing/katago-2".into(),
            argv: vec![],
            working_dir: None,
            adapter: EngineAdapterSettings::KataGoAnalysis(KataGoSettings {
                model_path: Some("/definitely/missing/model-2.bin".into()),
                config_path: Some("/definitely/missing/analysis-2.cfg".into()),
                max_visits: 800,
            }),
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
    let before_delete = manager.snapshot();
    let candidate_delete = manager.assert_profile_deletable("profile-b").unwrap_err();
    assert_eq!(candidate_delete.kind, EngineFailureKind::ProfileInUse);
    assert_eq!(
        candidate_delete.run_id.as_deref(),
        Some(candidate.run_id.as_str())
    );
    assert_eq!(manager.snapshot(), before_delete);
    manager.assert_profile_deletable("unrelated-profile").unwrap();

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
    assert_eq!(
        manager.analysis_task_snapshot().unwrap().state,
        AnalysisTaskStateDto::Invalidated
    );
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
        ready_two_profiles(&temp, &resident_whole_game_script(), &resident_echo_script());
    let task_job = manager
        .start_whole_game_analysis(whole_game_request(&run_a, 1, 2))
        .unwrap();
    wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == task_job.job_id && job.outcome == AnalysisJobOutcomeDto::Completed
    });
    let completed_task = manager.analysis_task_snapshot().unwrap();
    catalog.set_autoload_profile_id(Some("profile-a".into()));
    catalog.upsert(SavedEngineProfile {
        profile_id: "profile-b".into(),
        profile: EngineProfileDto {
            name: "Broken B".into(),
            program: "/definitely/missing/katago-b".into(),
            argv: vec![],
            working_dir: Some(temp.path().to_string_lossy().into_owned()),
            adapter: EngineAdapterSettings::KataGoAnalysis(KataGoSettings {
                model_path: Some("/definitely/missing/model-b.bin".into()),
                config_path: Some("/definitely/missing/b.cfg".into()),
                max_visits: 800,
            }),
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
    assert_eq!(manager.analysis_task_snapshot().unwrap(), completed_task);
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
fn switch_non_file_executable_keeps_ready_a_with_component_kind() {
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
        failure.kind == EngineFailureKind::Executable
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
fn non_file_executable_stays_no_engine_with_component_kind() {
    let temp = TestTempDir::new("spawn-start");
    let catalog = Arc::new(InMemoryEngineProfileCatalog::new());
    let mut profile = setup_profile(&temp, &resident_echo_script());
    let not_binary = temp.path().join("not-a-binary");
    std::fs::create_dir_all(&not_binary).unwrap();
    profile.program = not_binary.to_string_lossy().into_owned();
    catalog.upsert(SavedEngineProfile {
        profile_id: "profile-1".into(),
        profile,
    });
    let manager = ForegroundEngineManager::new(catalog, ForegroundEngineConfig::for_tests());
    let events = manager.subscribe();
    manager.start("profile-1").unwrap();
    let failure = wait_failure(&events, Duration::from_secs(2), |failure| {
        failure.kind == EngineFailureKind::Executable
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
    manager.set_continuous_preferences(true, app_model::ContinuousAnalysisBudgetDto::default()).unwrap();
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
    assert_eq!(ready.continuous.phase, app_model::ContinuousAnalysisPhaseDto::SafetyHold);
    assert!(ready.selected_node_job.is_none());
    manager.teardown().unwrap();
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
    broken.profile.program = "/definitely/missing/katago".into();
    if let EngineAdapterSettings::KataGoAnalysis(settings) = &mut broken.profile.adapter {
        settings.model_path = Some("/definitely/missing/model.bin".into());
        settings.config_path = Some("/definitely/missing/analysis.cfg".into());
    }
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

#[cfg(unix)]
fn streaming_engine_script() -> String {
    r#"
exec python3 -u -c '
import json, os, sys, threading, time
scenario = os.environ.get("SCENARIO", "stream")
active = set()
queued = []
def search(identity):
    visits = 16
    while identity in active:
        time.sleep(0.1)
        visits += 8
        if identity in active: emit(frame(identity, visits, True))
def emit(value):
    print(json.dumps(value), flush=True)
def frame(identity, visits, during):
    return dict(id=identity, turnNumber=0, isDuringSearch=during,
        ownership=[0.1] * 81, policy=[0.01] * 82,
        rootInfo=dict(visits=visits, winrate=0.6, scoreLead=2.5),
        moveInfos=[dict(move="D4", visits=visits, winrate=0.6, scoreMean=2.5, pv=["D4", "E5"])])
first = True
for line in sys.stdin:
    q = json.loads(line)
    if q.get("action") == "terminate":
        assert "terminateId" in q and q["id"] != q["terminateId"]
        emit(q)
        if scenario == "ack-only": continue
        active.discard(q["terminateId"])
        emit(frame(q["terminateId"], 999, True))
        time.sleep(0.05)
        emit(dict(id=q["terminateId"], isDuringSearch=False, noResults=True, turnNumber=0))
        for identity in queued: emit(frame(identity, 32, False))
        queued.clear()
    elif first:
        first = False
        emit(frame(q["id"], 2, False))
    elif ":" in q["id"]:
        if scenario in ["queued", "ack-only"]:
            queued.append(q["id"])
        else:
            cap = q["overrideSettings"]["maxVisits"]
            emit(frame(q["id"], min(cap, 32), False))
    elif "overrideSettings" in q:
        assert q["reportDuringSearchEvery"] == 0.1
        assert q["overrideSettings"]["maxTime"] == 600
        assert q["overrideSettings"]["maxVisits"] == 2**50
        assert q["overrideSettings"]["maxPlayouts"] == 2**50
        assert "maxVisits" not in q
        emit(dict(id=q["id"], warning="informational warning", field="rules"))
        if scenario == "ignored":
            emit(dict(id=q["id"], warning="setting ignored", field="overrideSettings.maxTime"))
            continue
        emit(dict(id=q["id"], action="terminate", terminateId="unrelated"))
        emit(frame(q["id"], 0, True))
        emit(frame(q["id"], 8, True))
        time.sleep(0.05)
        emit(frame(q["id"], 16, True))
        if scenario == "limited":
            emit(frame(q["id"], 24, False))
            emit(frame(q["id"], 999, False))
        if scenario == "queued":
            active.add(q["id"])
            threading.Thread(target=search, args=(q["id"],), daemon=True).start()
    else:
        if scenario in ["queued", "ack-only"]: queued.append(q["id"])
        else: emit(frame(q["id"], 32, False))
'
"#
    .into()
}

#[cfg(unix)]
#[test]
fn continuous_progress_remains_owned_until_target_final_and_releases_compute() {
    let temp = TestTempDir::new("continuous-progress");
    let (manager, _, events, run_id) = ready_manager(&temp, &streaming_engine_script());
    let mut request = selected_request(&run_id, 7, vec![0, 1]);
    request.mode = app_model::AnalysisJobModeDto::Continuous;
    let started = manager.start_selected_node_job(request).unwrap();
    for visits in [8, 16] {
        let event = wait_job(&events, Duration::from_secs(2), |job| {
            job.job_id == started.job_id && job.outcome != AnalysisJobOutcomeDto::Started
        });
        assert_eq!(event.outcome, AnalysisJobOutcomeDto::Progress);
        let frame = event.frame.unwrap();
        assert_eq!(frame.visits, visits);
        assert_eq!(frame.score_mean_black, Some(2.5));
        assert_eq!(frame.ownership, Some(vec![0.1; 81]));
        assert_eq!(frame.policy, Some(vec![0.01; 82]));
        assert_eq!(
            manager.snapshot().selected_node_job.unwrap().job_id,
            started.job_id
        );
    }
    manager.cancel_job(&run_id, &started.job_id).unwrap();
    let stopping = wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == started.job_id
    });
    assert_eq!(stopping.outcome, AnalysisJobOutcomeDto::Stopping);
    let stopped = wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == started.job_id
    });
    assert_eq!(stopped.outcome, AnalysisJobOutcomeDto::Cancelled);
    assert!(stopped.frame.is_none());
    assert!(manager.snapshot().selected_node_job.is_none());
    let finite = manager
        .start_selected_node_job(selected_request(&run_id, 7, vec![0, 1]))
        .unwrap();
    let final_event = wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == finite.job_id && job.outcome == AnalysisJobOutcomeDto::Completed
    });
    assert_eq!(final_event.frame.unwrap().visits, 32);
    manager.teardown().unwrap();
}

#[cfg(unix)]
#[test]
fn continuous_ack_without_target_final_fails_both_lanes_at_five_seconds() {
    let temp = TestTempDir::new("continuous-ack-only");
    let script = format!("export SCENARIO=ack-only\n{}", streaming_engine_script());
    let (manager, _, events, run_id) = ready_manager(&temp, &script);
    let mut request = selected_request(&run_id, 7, vec![]);
    request.mode = app_model::AnalysisJobModeDto::Continuous;
    let started = manager.start_selected_node_job(request).unwrap();
    wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == started.job_id && job.frame.as_ref().is_some_and(|frame| frame.visits == 16)
    });
    let whole = manager
        .start_whole_game_analysis(whole_game_request(&run_id, 7, 1))
        .unwrap();
    let before = Instant::now();
    manager.cancel_job(&run_id, &started.job_id).unwrap();
    std::thread::sleep(Duration::from_millis(200));
    assert_eq!(
        manager.snapshot().selected_node_job.unwrap().state,
        app_model::AnalysisJobStateDto::Stopping
    );
    let failed = wait_job(&events, Duration::from_secs(6), |job| {
        job.job_id == started.job_id && job.outcome == AnalysisJobOutcomeDto::Failed
    });
    assert!(before.elapsed() >= Duration::from_secs(4));
    assert!(before.elapsed() < Duration::from_secs(6));
    assert!(failed.frame.is_none());
    let whole_failed = wait_job(&events, Duration::from_secs(1), |job| {
        job.job_id == whole.job_id && job.outcome == AnalysisJobOutcomeDto::Failed
    });
    assert!(whole_failed.frame.is_none());
    let snapshot = manager.snapshot();
    assert!(matches!(
        snapshot.lifecycle,
        ForegroundEngineLifecycleDto::Error { .. }
    ));
    assert!(snapshot.selected_node_job.is_none() && snapshot.whole_game_job.is_none());
    assert!(manager
        .start_selected_node_job(selected_request(&run_id, 7, vec![]))
        .is_err());
    assert!(manager
        .wait_for_job_cancellation(&run_id, &started.job_id, Duration::from_secs(1))
        .is_err());
    manager.teardown().unwrap();
}

#[cfg(unix)]
#[test]
fn continuous_shared_activity_keeps_one_thread_queued_lane_alive() {
    let temp = TestTempDir::new("continuous-queued");
    let script = format!("export SCENARIO=queued\n{}", streaming_engine_script());
    let (manager, _, events, run_id) = ready_manager(&temp, &script);
    let mut request = selected_request(&run_id, 7, vec![]);
    request.mode = app_model::AnalysisJobModeDto::Continuous;
    let started = manager.start_selected_node_job(request).unwrap();
    wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == started.job_id && job.frame.as_ref().is_some_and(|frame| frame.visits == 16)
    });
    let whole = manager
        .start_whole_game_analysis(whole_game_request(&run_id, 7, 1))
        .unwrap();
    std::thread::sleep(Duration::from_millis(1300));
    let snapshot = manager.snapshot();
    assert_eq!(
        snapshot.selected_node_job.unwrap().state,
        app_model::AnalysisJobStateDto::Searching
    );
    assert_eq!(
        snapshot.whole_game_job.unwrap().state,
        app_model::AnalysisJobStateDto::Queued
    );
    manager.cancel_job(&run_id, &started.job_id).unwrap();
    let progress = wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == whole.job_id && job.outcome != AnalysisJobOutcomeDto::Started
    });
    assert_eq!(progress.outcome, AnalysisJobOutcomeDto::Progress);
    assert_eq!(progress.completed, Some(1));
    assert_eq!(progress.frame.unwrap().visits, 32);
    wait_job(&events, Duration::from_secs(1), |job| {
        job.job_id == whole.job_id && job.outcome == AnalysisJobOutcomeDto::Completed
    });
    manager.teardown().unwrap();
}

#[cfg(unix)]
#[test]
fn continuous_time_limit_retains_final_snapshot_without_resubmission() {
    let temp = TestTempDir::new("continuous-limited");
    let script = format!("export SCENARIO=limited\n{}", streaming_engine_script());
    let (manager, _, events, run_id) = ready_manager(&temp, &script);
    let mut request = selected_request(&run_id, 7, vec![]);
    request.mode = app_model::AnalysisJobModeDto::Continuous;
    let started = manager.start_selected_node_job(request).unwrap();
    let limited = wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == started.job_id && job.outcome == AnalysisJobOutcomeDto::TimeLimited
    });
    assert_eq!(limited.frame.unwrap().visits, 24);
    assert!(limited.failure.is_none());
    std::thread::sleep(Duration::from_millis(900));
    let snapshot = manager.snapshot();
    assert!(matches!(
        snapshot.lifecycle,
        ForegroundEngineLifecycleDto::Ready { .. }
    ));
    let retained = snapshot.selected_node_job.unwrap();
    assert_eq!(retained.job_id, started.job_id);
    assert_eq!(retained.state, app_model::AnalysisJobStateDto::TimeLimited);
    assert!(collect_job_events(&events, Instant::now() + Duration::from_millis(100)).is_empty());
    manager.teardown().unwrap();
}

#[cfg(unix)]
#[test]
fn continuous_ignored_required_budget_fails_visibly_without_analysis() {
    let temp = TestTempDir::new("continuous-ignored");
    let script = format!("export SCENARIO=ignored\n{}", streaming_engine_script());
    let (manager, _, events, run_id) = ready_manager(&temp, &script);
    let mut request = selected_request(&run_id, 7, vec![]);
    request.mode = app_model::AnalysisJobModeDto::Continuous;
    let started = manager.start_selected_node_job(request).unwrap();
    let failed = wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == started.job_id && job.outcome != AnalysisJobOutcomeDto::Started
    });
    assert_eq!(failed.outcome, AnalysisJobOutcomeDto::Failed);
    assert!(failed.frame.is_none());
    assert!(failed.failure.unwrap().message.contains("ignored"));
    manager.teardown().unwrap();
}

#[cfg(unix)]
#[test]
fn continuous_cleanup_failure_remains_visible_and_blocks_new_work() {
    let temp = TestTempDir::new("continuous-cleanup-failure");
    let script = format!("export SCENARIO=ack-only\n{}", streaming_engine_script());
    let catalog = Arc::new(InMemoryEngineProfileCatalog::new());
    catalog.upsert(SavedEngineProfile {
        profile_id: "profile-1".into(),
        profile: setup_profile(&temp, &script),
    });
    let mut config = ForegroundEngineConfig::for_tests();
    config.stop_drain_timeout = Duration::ZERO;
    let manager = ForegroundEngineManager::new(catalog, config);
    let events = manager.subscribe();
    manager.start("profile-1").unwrap();
    let ready = wait_snapshot(&events, Duration::from_secs(2), |state| {
        matches!(state, ForegroundEngineLifecycleDto::Ready { .. })
    });
    let run_id = run_from_ready(&ready.lifecycle).run_id.clone();
    let mut request = selected_request(&run_id, 7, vec![]);
    request.mode = app_model::AnalysisJobModeDto::Continuous;
    let started = manager.start_selected_node_job(request).unwrap();
    wait_job(&events, Duration::from_secs(2), |job| {
        job.frame.as_ref().is_some_and(|frame| frame.visits == 16)
    });
    manager.cancel_job(&run_id, &started.job_id).unwrap();
    let failed = wait_snapshot(&events, Duration::from_secs(6), |state| {
        matches!(state, ForegroundEngineLifecycleDto::Error { .. })
    });
    let ForegroundEngineLifecycleDto::Error { failure, .. } = failed.lifecycle else {
        unreachable!()
    };
    assert!(failure.message.contains("cleanup failed"));
    assert!(manager.start("profile-1").is_err());
    assert!(manager
        .start_selected_node_job(selected_request(&run_id, 7, vec![]))
        .is_err());
    assert!(manager.assert_profile_deletable("profile-1").is_err());
    let retry_deadline = Instant::now() + Duration::from_secs(2);
    loop {
        match manager.teardown() {
            Ok(()) => break,
            Err(failure) => {
                assert!(failure.message.contains("cleanup failed"));
                assert!(
                    Instant::now() < retry_deadline,
                    "cleanup ownership did not release after bounded retries: {failure:?}"
                );
                std::thread::sleep(Duration::from_millis(10));
            }
        }
    }
    assert!(matches!(
        manager.snapshot().lifecycle,
        ForegroundEngineLifecycleDto::NoEngine { .. }
    ));
}

fn continuous_request(run_id: &str, generation: u64, indices: Vec<u32>) -> SelectedNodeJobRequest {
    let mut request = selected_request(run_id, generation, indices);
    request.mode = app_model::AnalysisJobModeDto::Continuous;
    request
}

#[cfg(unix)]
fn wait_continuous_phase(
    manager: &ForegroundEngineManager,
    expected: app_model::ContinuousAnalysisPhaseDto,
) -> app_model::ForegroundEngineSnapshotDto {
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        let snapshot = manager.snapshot();
        if snapshot.continuous.phase == expected {
            return snapshot;
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for {expected:?}, last={snapshot:?}"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn continuous_preferences_failed_persistence_preserves_intent_and_budget() {
    let temp = TestTempDir::new("continuous-preferences-write-failure");
    let manager = ForegroundEngineManager::new(
        Arc::new(InMemoryEngineProfileCatalog::new()),
        ForegroundEngineConfig::for_tests(),
    );
    let budget = app_model::ContinuousAnalysisBudgetDto::default();
    manager.set_continuous_preferences(false, budget).unwrap();
    let before = manager.snapshot().continuous;
    let events = manager.subscribe();
    let mut changed_budget = budget;
    changed_budget.continuous_visits_limit += 1;

    let error = manager
        .commit_continuous_preferences(true, changed_budget, || {
            // Writing to a directory exercises a real failed persistence operation.
            std::fs::write(temp.path(), "preferences").map_err(|error| error.to_string())
        })
        .unwrap_err();

    assert!(!error.is_empty());
    assert_eq!(manager.snapshot().continuous, before);
    assert!(events.try_recv().is_err());
    manager.reserve_match("persistence-rollback").unwrap();
    // Only an unchanged intent and budget are admitted while reserved.
    manager.set_continuous_preferences(false, budget).unwrap();
    manager.abort_reserved_match("persistence-rollback").unwrap();
    let saved = manager
        .commit_continuous_preferences(true, changed_budget, || {
            std::fs::write(temp.path().join("preferences"), "saved").map_err(|error| error.to_string())?;
            Ok("saved")
        })
        .unwrap();
    assert_eq!(saved, "saved");
    assert_eq!(manager.snapshot().continuous.enabled, Some(true));
}

#[test]
fn continuous_intent_without_an_engine_waits_without_creating_a_job() {
    let manager = ForegroundEngineManager::new(
        Arc::new(InMemoryEngineProfileCatalog::new()),
        ForegroundEngineConfig::for_tests(),
    );
    manager
        .set_continuous_preferences(true, app_model::ContinuousAnalysisBudgetDto::default())
        .unwrap();
    manager.follow_continuous_position(continuous_request("ignored", 1, vec![]));

    let snapshot = manager.snapshot();
    assert_eq!(snapshot.continuous.enabled, Some(true));
    assert_eq!(
        snapshot.continuous.phase,
        app_model::ContinuousAnalysisPhaseDto::Waiting
    );
    assert!(snapshot.selected_node_job.is_none());
    assert!(matches!(
        snapshot.lifecycle,
        ForegroundEngineLifecycleDto::NoEngine { .. }
    ));
}

#[cfg(unix)]
#[test]
fn continuous_navigation_coalesces_to_the_latest_position_during_target_cleanup() {
    let temp = TestTempDir::new("continuous-follow-latest");
    let script = format!("export SCENARIO=queued\n{}", streaming_engine_script());
    let (manager, _, events, run_id) = ready_manager(&temp, &script);
    manager
        .set_continuous_preferences(true, app_model::ContinuousAnalysisBudgetDto::default())
        .unwrap();
    manager.follow_continuous_position(continuous_request("stale-wire-run", 20, vec![0]));
    let first = manager.snapshot().selected_node_job.unwrap();
    assert_eq!(first.run_id, run_id);

    manager.follow_continuous_position(continuous_request("ignored", 20, vec![1]));
    manager.follow_continuous_position(continuous_request("ignored", 20, vec![2]));

    let deadline = Instant::now() + Duration::from_secs(3);
    let mut started_paths = Vec::new();
    while Instant::now() < deadline {
        if let Ok(ForegroundEngineEventDto::Job { job }) = events.recv_timeout(Duration::from_millis(100)) {
            if job.outcome == AnalysisJobOutcomeDto::Started && job.job_id != first.job_id {
                started_paths.push(job.node_path.indices.clone());
                if job.node_path.indices == vec![2] {
                    break;
                }
            }
        }
    }
    assert_eq!(started_paths, vec![vec![2]]);
    assert_eq!(
        manager.snapshot().selected_node_job.unwrap().node_path.indices,
        vec![2]
    );
    manager.teardown().unwrap();
}

#[cfg(unix)]
#[test]
fn continuous_limit_survives_duplicate_ready_intent_and_reselection_but_not_navigation() {
    let temp = TestTempDir::new("continuous-managed-limit");
    let script = format!("export SCENARIO=limited\n{}", streaming_engine_script());
    let (manager, _, events, run_id) = ready_manager(&temp, &script);
    manager
        .set_continuous_preferences(true, app_model::ContinuousAnalysisBudgetDto::default())
        .unwrap();
    manager.follow_continuous_position(continuous_request(&run_id, 21, vec![]));
    let limited = wait_job(&events, Duration::from_secs(2), |job| {
        job.outcome == AnalysisJobOutcomeDto::TimeLimited
    });

    manager
        .set_continuous_preferences(true, app_model::ContinuousAnalysisBudgetDto::default())
        .unwrap();
    manager.follow_continuous_position(continuous_request("ignored", 21, vec![]));
    std::thread::sleep(Duration::from_millis(200));
    let held = manager.snapshot();
    assert_eq!(
        held.continuous.phase,
        app_model::ContinuousAnalysisPhaseDto::TimeLimited
    );
    assert_eq!(held.selected_node_job.unwrap().job_id, limited.job_id);
    assert!(
        collect_job_events(&events, Instant::now() + Duration::from_millis(150))
            .iter()
            .all(|job| job.outcome != AnalysisJobOutcomeDto::Started)
    );

    manager.follow_continuous_position(continuous_request("ignored", 21, vec![0]));
    let restarted = wait_job(&events, Duration::from_secs(2), |job| {
        job.outcome == AnalysisJobOutcomeDto::Started && job.node_path.indices == vec![0]
    });
    assert_ne!(restarted.job_id, limited.job_id);
    manager.teardown().unwrap();
}

#[cfg(unix)]
#[test]
fn continuous_error_is_not_cleared_by_navigation_and_explicit_resume_reauthorizes_it() {
    let temp = TestTempDir::new("continuous-managed-error");
    let script = r#"
first=1
failed=0
while IFS= read -r line; do
  id=$(printf '%s' "$line" | sed -n 's/.*"id":"\([^"]*\)".*/\1/p')
  [ -z "$id" ] && id="ok"
  if printf '%s' "$line" | grep -q '"action":"terminate"'; then
    target=$(printf '%s' "$line" | sed -n 's/.*"terminateId":"\([^"]*\)".*/\1/p')
    [ -n "$target" ] && [ "$target" != "$id" ] || exit 20
    printf '%s\n' "$line"
    printf '{"id":"%s","isDuringSearch":false,"noResults":true,"turnNumber":0}\n' "$target"
  elif [ "$first" = 1 ]; then
    printf '{"id":"%s","turnNumber":0}\n' "$id"
    first=0
  elif [ "$failed" = 0 ]; then
    printf '{"id":"%s","isDuringSearch":false,"noResults":true,"turnNumber":0}\n' "$id"
    failed=1
  fi
done
"#;
    let (manager, _, events, run_id) = ready_manager(&temp, script);
    manager
        .set_continuous_preferences(true, app_model::ContinuousAnalysisBudgetDto::default())
        .unwrap();
    manager.follow_continuous_position(continuous_request(&run_id, 22, vec![]));
    wait_job(&events, Duration::from_secs(2), |job| {
        job.outcome == AnalysisJobOutcomeDto::Failed
    });
    wait_continuous_phase(&manager, app_model::ContinuousAnalysisPhaseDto::Error);

    manager.follow_continuous_position(continuous_request("ignored", 22, vec![0]));
    manager
        .set_continuous_preferences(
            true,
            app_model::ContinuousAnalysisBudgetDto {
                continuous_time_limit_seconds: 2,
                ..app_model::ContinuousAnalysisBudgetDto::default()
            },
        )
        .unwrap();
    assert_eq!(
        manager.snapshot().continuous.phase,
        app_model::ContinuousAnalysisPhaseDto::Error
    );
    assert_eq!(
        manager.continuous_primary_action().unwrap(),
        engine_manager::ContinuousPrimaryAction::Resume
    );
    manager.resume_continuous().unwrap();
    let resumed = manager.snapshot().selected_node_job.unwrap();
    assert_eq!(resumed.node_path.indices, vec![0]);
    manager.teardown().unwrap();
}

#[cfg(unix)]
#[test]
fn finite_owner_blocks_automatic_work_and_cancel_pauses_only_the_same_admission() {
    let temp = TestTempDir::new("continuous-finite-cancel");
    let (manager, _, events, run_id) = ready_manager(&temp, &hold_after_probe_script());
    manager.follow_continuous_position(continuous_request(&run_id, 30, vec![]));
    let finite = manager
        .start_selected_node_job(selected_request(&run_id, 30, vec![]))
        .unwrap();
    manager
        .set_continuous_preferences(true, app_model::ContinuousAnalysisBudgetDto::default())
        .unwrap();
    let active = manager.snapshot();
    assert_eq!(
        active.continuous.phase,
        app_model::ContinuousAnalysisPhaseDto::Finite
    );
    assert_eq!(active.selected_node_job.unwrap().job_id, finite.job_id);

    manager.cancel_job(&run_id, &finite.job_id).unwrap();
    wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == finite.job_id && job.outcome == AnalysisJobOutcomeDto::Cancelled
    });
    let paused = wait_continuous_phase(&manager, app_model::ContinuousAnalysisPhaseDto::Paused);
    assert!(paused.selected_node_job.is_none());

    manager.follow_continuous_position(continuous_request("ignored", 30, vec![0]));
    let restarted = manager.snapshot().selected_node_job.unwrap();
    assert_eq!(restarted.mode, app_model::AnalysisJobModeDto::Continuous);
    assert_eq!(restarted.node_path.indices, vec![0]);
    manager.teardown().unwrap();
}

#[cfg(unix)]
#[test]
fn failed_finite_owner_latches_error_across_intent_and_navigation() {
    let temp = TestTempDir::new("continuous-finite-failure");
    let (manager, _, events, run_id) = ready_manager(&temp, &selected_node_protocol_error_script());
    manager.follow_continuous_position(continuous_request(&run_id, 31, vec![]));
    let finite = manager
        .start_selected_node_job(selected_request(&run_id, 31, vec![]))
        .unwrap();
    wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == finite.job_id && job.outcome == AnalysisJobOutcomeDto::Failed
    });
    manager
        .set_continuous_preferences(true, app_model::ContinuousAnalysisBudgetDto::default())
        .unwrap();
    wait_continuous_phase(&manager, app_model::ContinuousAnalysisPhaseDto::Error);

    manager.follow_continuous_position(continuous_request("ignored", 31, vec![0]));
    let held = manager.snapshot();
    assert_eq!(
        held.continuous.phase,
        app_model::ContinuousAnalysisPhaseDto::Error
    );
    assert!(held.selected_node_job.is_none());
    manager.teardown().unwrap();
}

#[cfg(unix)]
#[test]
fn finite_stale_position_is_rejected_without_stopping_current_continuous_work() {
    let temp = TestTempDir::new("finite-stale-admission");
    let (manager, _, events, run_id) = ready_manager(&temp, budget_engine_script());
    manager
        .set_continuous_preferences(true, app_model::ContinuousAnalysisBudgetDto::default())
        .unwrap();
    manager.follow_continuous_position(continuous_request(&run_id, 32, vec![0]));
    let progress = wait_job(&events, Duration::from_secs(2), |job| {
        job.outcome == AnalysisJobOutcomeDto::Progress
    });
    let rejected = manager
        .start_selected_node_job(selected_request(&run_id, 32, vec![]))
        .unwrap_err();
    assert_eq!(rejected.kind, EngineFailureKind::InvalidState);
    let active = manager.snapshot().selected_node_job.unwrap();
    assert_eq!(active.job_id, progress.job_id);
    assert_eq!(active.state, app_model::AnalysisJobStateDto::Searching);
    let next = wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == progress.job_id && job.outcome == AnalysisJobOutcomeDto::Progress
    });
    assert!(next.frame.unwrap().visits > progress.frame.unwrap().visits);
    manager.teardown().unwrap();
}

#[cfg(unix)]
#[test]
fn successful_finite_handoff_restores_once_with_latest_continuous_budget() {
    let temp = TestTempDir::new("continuous-finite-complete");
    let (manager, _, events, run_id) = ready_manager(&temp, budget_engine_script());
    let mut budget = app_model::ContinuousAnalysisBudgetDto::default();
    manager.set_continuous_preferences(true, budget).unwrap();
    manager.follow_continuous_position(continuous_request(&run_id, 32, vec![]));
    let continuous = wait_job(&events, Duration::from_secs(2), |job| {
        job.outcome == AnalysisJobOutcomeDto::Progress
    });
    let mut request = selected_request(&run_id, 32, vec![]);
    request.query.max_visits = Some(128);
    let finite = manager.start_selected_node_job(request).unwrap();
    assert_ne!(finite.job_id, continuous.job_id);
    assert_eq!(manager.snapshot().continuous.enabled, Some(true));
    budget.continuous_visits_limit_enabled = true;
    budget.continuous_visits_limit = 16;
    manager.set_continuous_preferences(true, budget).unwrap();
    let completed = wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == finite.job_id && job.outcome == AnalysisJobOutcomeDto::Completed
    });
    assert_eq!(completed.frame.unwrap().visits, 128);
    let restored = wait_job(&events, Duration::from_secs(2), |job| {
        job.mode == app_model::AnalysisJobModeDto::Continuous && job.outcome == AnalysisJobOutcomeDto::Started
    });
    let limited = wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == restored.job_id && job.outcome == AnalysisJobOutcomeDto::VisitsLimited
    });
    assert_ne!(restored.job_id, continuous.job_id);
    assert_ne!(restored.job_id, finite.job_id);
    assert_eq!(limited.frame.unwrap().visits, 16);
    assert!(
        collect_job_events(&events, Instant::now() + Duration::from_millis(100))
            .iter()
            .all(|job| job.outcome != AnalysisJobOutcomeDto::Started)
    );
    manager.teardown().unwrap();
}

#[cfg(unix)]
#[test]
fn finite_completion_observes_off_without_cancelling_or_leaving_a_pause() {
    let temp = TestTempDir::new("finite-off-complete");
    let (manager, _, events, run_id) = ready_manager(&temp, budget_engine_script());
    let mut budget = app_model::ContinuousAnalysisBudgetDto::default();
    manager.follow_continuous_position(continuous_request(&run_id, 33, vec![]));
    let mut request = selected_request(&run_id, 33, vec![]);
    request.query.max_visits = Some(128);
    let finite = manager.start_selected_node_job(request).unwrap();
    manager.set_continuous_preferences(true, budget).unwrap();
    manager.set_continuous_preferences(false, budget).unwrap();
    let completed = wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == finite.job_id && job.outcome == AnalysisJobOutcomeDto::Completed
    });
    assert_eq!(completed.frame.unwrap().visits, 128);
    assert_eq!(
        manager.snapshot().continuous.phase,
        app_model::ContinuousAnalysisPhaseDto::Off
    );
    assert!(manager.snapshot().selected_node_job.is_none());
    budget.continuous_visits_limit_enabled = true;
    budget.continuous_visits_limit = 16;
    manager.set_continuous_preferences(true, budget).unwrap();
    let restored = wait_job(&events, Duration::from_secs(2), |job| {
        job.outcome == AnalysisJobOutcomeDto::VisitsLimited
    });
    assert_eq!(restored.frame.unwrap().visits, 16);
    manager.teardown().unwrap();
}

#[cfg(unix)]
#[test]
fn finite_navigation_supersession_resumes_only_latest_position_and_keeps_whole_game() {
    let temp = TestTempDir::new("finite-navigation");
    let (manager, _, events, run_id) = ready_manager(&temp, budget_engine_script());
    manager.follow_continuous_position(continuous_request(&run_id, 34, vec![]));
    let mut request = selected_request(&run_id, 34, vec![]);
    request.query.max_visits = Some(10000);
    let finite = manager.start_selected_node_job(request).unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    while manager.snapshot().selected_node_job.as_ref().unwrap().state
        != app_model::AnalysisJobStateDto::Searching
    {
        assert!(
            Instant::now() < deadline,
            "finite request never obtained search capacity"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    let mut whole_request = whole_game_request(&run_id, 34, 2);
    for item in &mut whole_request.work_items {
        item.query.max_visits = Some(10000);
    }
    let whole = manager.start_whole_game_analysis(whole_request).unwrap();
    manager
        .set_continuous_preferences(true, app_model::ContinuousAnalysisBudgetDto::default())
        .unwrap();
    manager.follow_continuous_position(continuous_request(&run_id, 34, vec![0, 1]));
    let superseded = wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == finite.job_id && job.outcome == AnalysisJobOutcomeDto::Superseded
    });
    assert!(superseded.frame.is_none());
    let restored = wait_job(&events, Duration::from_secs(2), |job| {
        job.mode == app_model::AnalysisJobModeDto::Continuous && job.outcome == AnalysisJobOutcomeDto::Started
    });
    assert_eq!(restored.node_path.indices, vec![0, 1]);
    assert_eq!(manager.snapshot().whole_game_job.unwrap().job_id, whole.job_id);
    assert_eq!(
        manager
            .start_whole_game_analysis(whole_game_request(&run_id, 34, 2))
            .unwrap_err()
            .kind,
        EngineFailureKind::Occupied
    );
    manager.cancel_job(&run_id, &whole.job_id).unwrap();
    wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == restored.job_id && job.outcome == AnalysisJobOutcomeDto::Progress
    });
    assert_eq!(
        manager.snapshot().selected_node_job.unwrap().job_id,
        restored.job_id
    );
    assert!(
        collect_job_events(&events, Instant::now() + Duration::from_millis(100))
            .iter()
            .all(|job| {
                job.outcome != AnalysisJobOutcomeDto::Started
                    && !(job.job_id == finite.job_id && job.outcome == AnalysisJobOutcomeDto::Completed)
            })
    );
    manager.teardown().unwrap();
}

#[cfg(unix)]
#[test]
fn finite_admission_cannot_cross_a_confirmed_departure() {
    let temp = TestTempDir::new("finite-admission-departure");
    let release = temp.path().join("release-cancellation");
    let script = streaming_engine_script().replace(
        "        time.sleep(0.05)",
        &format!(
            "        while not os.path.exists({:?}): time.sleep(0.01)",
            release
        ),
    );
    let (manager, _, events, run_id) = ready_manager(&temp, &script);
    manager
        .set_continuous_preferences(true, app_model::ContinuousAnalysisBudgetDto::default())
        .unwrap();
    manager.follow_continuous_position(continuous_request(&run_id, 35, vec![]));
    let old = wait_job(&events, Duration::from_secs(2), |job| {
        job.outcome == AnalysisJobOutcomeDto::Progress
    });
    let request = selected_request(&run_id, 35, vec![]);
    let admitting = manager.clone();
    let finite = std::thread::spawn(move || {
        admitting
            .start_selected_node_job(request)
            .map_err(|error| error.kind)
    });
    wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == old.job_id && job.outcome == AnalysisJobOutcomeDto::Stopping
    });
    manager.begin_continuous_departure();
    std::fs::write(&release, "release").unwrap();
    assert_eq!(
        finite.join().unwrap().unwrap_err(),
        EngineFailureKind::InvalidState
    );
    assert!(manager.snapshot().selected_node_job.is_none());
    manager.finish_continuous_departure(false);
    manager.follow_continuous_position(continuous_request(&run_id, 35, vec![0]));
    assert_eq!(
        manager.snapshot().continuous.phase,
        app_model::ContinuousAnalysisPhaseDto::SafetyHold
    );
    assert!(manager.snapshot().selected_node_job.is_none());
    manager.resume_continuous().unwrap();
    let resumed = wait_job(&events, Duration::from_secs(2), |job| {
        job.outcome == AnalysisJobOutcomeDto::Progress && job.job_id != old.job_id
    });
    assert_eq!(resumed.node_path.indices, vec![0]);
    manager.teardown().unwrap();
}

#[cfg(unix)]
#[test]
fn safety_hold_survives_navigation_readiness_and_authorization_during_departure() {
    let temp = TestTempDir::new("continuous-safety-readiness");
    let catalog = Arc::new(InMemoryEngineProfileCatalog::new());
    catalog.upsert(SavedEngineProfile {
        profile_id: "profile-1".into(),
        profile: setup_profile(&temp, &streaming_engine_script()),
    });
    let manager = ForegroundEngineManager::new(catalog, ForegroundEngineConfig::for_tests());
    let events = manager.subscribe();
    manager
        .set_continuous_preferences(true, app_model::ContinuousAnalysisBudgetDto::default())
        .unwrap();
    manager.follow_continuous_position(continuous_request("ignored", 40, vec![]));
    manager.begin_continuous_departure();
    manager.finish_continuous_departure(false);
    assert_eq!(
        manager.snapshot().continuous.phase,
        app_model::ContinuousAnalysisPhaseDto::SafetyHold
    );

    manager.begin_continuous_departure();
    manager.authorize_continuous_start();
    manager.finish_continuous_departure(false);
    manager.follow_continuous_position(continuous_request("ignored", 40, vec![0]));
    manager.start("profile-1").unwrap();
    wait_snapshot(&events, Duration::from_secs(2), |state| {
        matches!(state, ForegroundEngineLifecycleDto::Ready { .. })
    });
    let held = manager.snapshot();
    assert_eq!(
        held.continuous.phase,
        app_model::ContinuousAnalysisPhaseDto::SafetyHold
    );
    assert!(held.selected_node_job.is_none());

    manager.resume_continuous().unwrap();
    assert_eq!(
        manager.snapshot().selected_node_job.unwrap().node_path.indices,
        vec![0]
    );
    manager.teardown().unwrap();
}

#[cfg(unix)]
#[test]
fn continuous_snapshot_prioritizes_pending_target_cleanup_over_intent_and_finite_owner() {
    let continuous_temp = TestTempDir::new("continuous-stop-priority");
    let script = format!("export SCENARIO=ack-only\n{}", streaming_engine_script());
    let (manager, _, events, run_id) = ready_manager(&continuous_temp, &script);
    manager
        .set_continuous_preferences(true, app_model::ContinuousAnalysisBudgetDto::default())
        .unwrap();
    manager.follow_continuous_position(continuous_request(&run_id, 50, vec![]));
    wait_job(&events, Duration::from_secs(2), |job| {
        job.mode == app_model::AnalysisJobModeDto::Continuous
            && job.frame.as_ref().is_some_and(|frame| frame.visits == 16)
    });
    manager
        .set_continuous_preferences(false, app_model::ContinuousAnalysisBudgetDto::default())
        .unwrap();
    assert_eq!(
        manager.snapshot().continuous.phase,
        app_model::ContinuousAnalysisPhaseDto::Stopping
    );
    assert!(manager.continuous_primary_action().is_err());
    manager.teardown().unwrap();

    let finite_temp = TestTempDir::new("finite-stop-priority");
    let script = format!("export SCENARIO=ack-only\n{}", streaming_engine_script());
    let (manager, _, _, run_id) = ready_manager(&finite_temp, &script);
    manager.follow_continuous_position(continuous_request(&run_id, 51, vec![]));
    let finite = manager
        .start_selected_node_job(selected_request(&run_id, 51, vec![]))
        .unwrap();
    manager
        .set_continuous_preferences(true, app_model::ContinuousAnalysisBudgetDto::default())
        .unwrap();
    manager.cancel_job(&run_id, &finite.job_id).unwrap();
    assert_eq!(
        manager.snapshot().continuous.phase,
        app_model::ContinuousAnalysisPhaseDto::Stopping
    );
    assert!(manager.continuous_primary_action().is_err());
    manager.teardown().unwrap();
}

#[cfg(unix)]
fn budget_engine_script() -> &'static str {
    r#"
exec python3 -u -c '
import json, sys, threading, time
capacity = threading.Lock()
output = threading.Lock()
cancels = {}
def emit(value):
    with output: print(json.dumps(value), flush=True)
def frame(identity, visits, during):
    return dict(id=identity, turnNumber=0, isDuringSearch=during,
        rootInfo=dict(visits=visits, winrate=0.6, scoreLead=2.5),
        moveInfos=[dict(move="D4", visits=visits, winrate=0.6, scoreMean=2.5, pv=["D4"])] if visits > 1 else [])
def search(q, cancel):
    while not capacity.acquire(timeout=0.01):
        if cancel.is_set():
            emit(dict(id=q["id"], isDuringSearch=False, noResults=True))
            return
    try:
        # Like KataGo, start search time only after obtaining analysis capacity.
        start = time.monotonic()
        settings = q.get("overrideSettings", {})
        cap = settings.get("maxVisits", q.get("maxVisits", 500))
        seconds = settings.get("maxTime", 1e20)
        visits = 0
        while not cancel.is_set():
            time.sleep(0.02)
            visits = min(visits + 8, cap)
            limited = visits >= cap or time.monotonic() - start >= seconds
            emit(frame(q["id"], visits, not limited))
            if limited: return
        emit(dict(id=q["id"], isDuringSearch=False, noResults=True))
    finally:
        capacity.release()
first = True
for line in sys.stdin:
    q = json.loads(line)
    if q.get("action") == "terminate":
        assert q["id"] != q["terminateId"]
        emit(q)
        cancels[q["terminateId"]].set()
    elif first:
        first = False
        emit(frame(q["id"], 2, False))
    else:
        cancel = threading.Event()
        cancels[q["id"]] = cancel
        threading.Thread(target=search, args=(q,cancel), daemon=True).start()
'
"#
}

#[cfg(unix)]
#[test]
fn continuous_budgets_stop_at_first_limit_and_only_new_admissions_reset_it() {
    use app_model::{ContinuousAnalysisBudgetDto, ContinuousAnalysisPhaseDto};
    let temp = TestTempDir::new("continuous-budget-or");
    let (manager, _, events, run_id) = ready_manager(&temp, budget_engine_script());
    let mut budget = ContinuousAnalysisBudgetDto {
        continuous_time_limit_seconds: 1,
        continuous_visits_limit_enabled: true,
        continuous_visits_limit: 8,
        ..ContinuousAnalysisBudgetDto::default()
    };
    manager.set_continuous_preferences(true, budget).unwrap();
    manager.follow_continuous_position(continuous_request(&run_id, 60, vec![]));
    let visits = wait_job(&events, Duration::from_secs(2), |job| {
        job.outcome == AnalysisJobOutcomeDto::VisitsLimited
    });
    assert_eq!(visits.frame.unwrap().visits, 8);
    manager.set_continuous_preferences(true, budget).unwrap();
    manager.follow_continuous_position(continuous_request(&run_id, 60, vec![]));
    assert_eq!(
        manager.snapshot().selected_node_job.unwrap().job_id,
        visits.job_id
    );
    assert!(
        collect_job_events(&events, Instant::now() + Duration::from_millis(100))
            .iter()
            .all(|job| job.outcome != AnalysisJobOutcomeDto::Started)
    );

    manager.resume_continuous().unwrap();
    let resumed = wait_job(&events, Duration::from_secs(2), |job| {
        job.outcome == AnalysisJobOutcomeDto::VisitsLimited
    });
    assert_ne!(resumed.job_id, visits.job_id);
    manager.follow_continuous_position(continuous_request(&run_id, 60, vec![0]));
    let navigated = wait_job(&events, Duration::from_secs(2), |job| {
        job.outcome == AnalysisJobOutcomeDto::VisitsLimited
    });
    assert_ne!(navigated.job_id, resumed.job_id);

    budget.continuous_visits_limit = 10000;
    manager.set_continuous_preferences(true, budget).unwrap();
    let timed = wait_job(&events, Duration::from_secs(3), |job| {
        job.outcome == AnalysisJobOutcomeDto::TimeLimited
    });
    assert!(timed.frame.unwrap().visits < 10000);
    assert_ne!(timed.job_id, navigated.job_id);
    assert_eq!(
        manager.snapshot().continuous.phase,
        ContinuousAnalysisPhaseDto::TimeLimited
    );
    assert!(matches!(
        manager.snapshot().lifecycle,
        ForegroundEngineLifecycleDto::Ready { .. }
    ));
    manager.restart().unwrap();
    let fresh = wait_job(&events, Duration::from_secs(3), |job| {
        job.outcome == AnalysisJobOutcomeDto::Started
    });
    assert_ne!(fresh.run_id, run_id);
    manager.teardown().unwrap();
}

#[cfg(unix)]
#[test]
fn continuous_one_visit_budget_limits_without_fabricating_candidates() {
    use app_model::{ContinuousAnalysisBudgetDto, ContinuousAnalysisPhaseDto};
    let temp = TestTempDir::new("continuous-budget-root-only");
    let (manager, _, events, run_id) = ready_manager(&temp, budget_engine_script());
    manager
        .set_continuous_preferences(
            true,
            ContinuousAnalysisBudgetDto {
                continuous_time_limit_enabled: false,
                continuous_visits_limit_enabled: true,
                continuous_visits_limit: 1,
                ..ContinuousAnalysisBudgetDto::default()
            },
        )
        .unwrap();
    manager.follow_continuous_position(continuous_request(&run_id, 62, vec![]));
    let terminal = wait_job(&events, Duration::from_secs(2), |job| {
        !matches!(
            job.outcome,
            AnalysisJobOutcomeDto::Started | AnalysisJobOutcomeDto::Progress
        )
    });
    assert_eq!(terminal.outcome, AnalysisJobOutcomeDto::VisitsLimited);
    let frame = terminal.frame.expect("root-only analysis frame");
    assert_eq!(frame.visits, 1);
    assert!(frame.candidates.is_empty());
    assert_eq!(
        manager.snapshot().continuous.phase,
        ContinuousAnalysisPhaseDto::VisitsLimited
    );
    assert_eq!(manager.snapshot().continuous.enabled, Some(true));
    manager.resume_continuous().unwrap();
    let resumed = wait_job(&events, Duration::from_secs(2), |job| {
        job.outcome == AnalysisJobOutcomeDto::VisitsLimited
    });
    assert_ne!(resumed.job_id, terminal.job_id);
    manager.teardown().unwrap();
}

#[cfg(unix)]
#[test]
fn continuous_budget_search_time_excludes_one_thread_queue_wait() {
    let temp = TestTempDir::new("continuous-budget-queue");
    let (manager, _, events, run_id) = ready_manager(&temp, budget_engine_script());
    let mut whole = whole_game_request(&run_id, 61, 1);
    whole.work_items[0].query.max_visits = Some(100000);
    let whole = manager.start_whole_game_analysis(whole).unwrap();
    wait_task(&manager, AnalysisTaskStateDto::Searching);
    manager
        .set_continuous_preferences(
            true,
            app_model::ContinuousAnalysisBudgetDto {
                continuous_time_limit_seconds: 1,
                ..app_model::ContinuousAnalysisBudgetDto::default()
            },
        )
        .unwrap();
    manager.follow_continuous_position(continuous_request(&run_id, 61, vec![]));
    let queued = manager.snapshot().selected_node_job.unwrap();
    std::thread::sleep(Duration::from_millis(1200));
    assert_eq!(
        manager.snapshot().selected_node_job.unwrap().state,
        app_model::AnalysisJobStateDto::Queued
    );
    manager.cancel_job(&run_id, &whole.job_id).unwrap();
    let released = Instant::now();
    let limited = wait_job(&events, Duration::from_secs(3), |job| {
        job.outcome == AnalysisJobOutcomeDto::TimeLimited
    });
    assert_eq!(limited.job_id, queued.job_id);
    assert!(
        released.elapsed() >= Duration::from_millis(900),
        "queue wait was charged as search time"
    );
    assert!(limited.frame.unwrap().visits > 0);
    assert!(matches!(
        manager.snapshot().lifecycle,
        ForegroundEngineLifecycleDto::Ready { .. }
    ));
    manager.teardown().unwrap();
}

#[cfg(unix)]
#[test]
fn continuous_unlimited_budget_ignores_config_cap_and_remains_cancellable() {
    let temp = TestTempDir::new("continuous-unlimited");
    let (manager, _, events, run_id) = ready_manager(&temp, budget_engine_script());
    let unlimited = app_model::ContinuousAnalysisBudgetDto {
        continuous_time_limit_enabled: false,
        continuous_visits_limit_enabled: false,
        ..app_model::ContinuousAnalysisBudgetDto::default()
    };
    manager.set_continuous_preferences(true, unlimited).unwrap();
    manager.follow_continuous_position(continuous_request(&run_id, 62, vec![]));
    let progress = wait_job(&events, Duration::from_secs(3), |job| {
        job.frame.as_ref().is_some_and(|frame| frame.visits > 500)
    });
    assert_eq!(progress.outcome, AnalysisJobOutcomeDto::Progress);
    manager.set_continuous_preferences(false, unlimited).unwrap();
    wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == progress.job_id && job.outcome == AnalysisJobOutcomeDto::Cancelled
    });
    assert_eq!(manager.snapshot().continuous.enabled, Some(false));
    assert!(matches!(
        manager.snapshot().lifecycle,
        ForegroundEngineLifecycleDto::Ready { .. }
    ));
    manager.teardown().unwrap();
}

#[cfg(unix)]
#[test]
fn continuous_budget_updates_leave_finite_and_whole_game_requests_unchanged() {
    let temp = TestTempDir::new("continuous-budget-independent");
    let (manager, _, events, run_id) = ready_manager(&temp, budget_engine_script());
    let mut budget = app_model::ContinuousAnalysisBudgetDto {
        continuous_visits_limit_enabled: true,
        continuous_visits_limit: 8,
        ..app_model::ContinuousAnalysisBudgetDto::default()
    };
    manager.set_continuous_preferences(false, budget).unwrap();
    manager.follow_continuous_position(continuous_request(&run_id, 63, vec![]));
    let mut request = selected_request(&run_id, 63, vec![]);
    request.query.max_visits = Some(128);
    let finite = manager.start_selected_node_job(request).unwrap();
    budget.continuous_visits_limit = 16;
    manager.set_continuous_preferences(false, budget).unwrap();
    let completed = wait_job(&events, Duration::from_secs(2), |job| {
        job.outcome == AnalysisJobOutcomeDto::Completed
    });
    assert_eq!(completed.job_id, finite.job_id);
    assert_eq!(completed.frame.unwrap().visits, 128);

    let mut request = whole_game_request(&run_id, 63, 1);
    request.work_items[0].query.max_visits = Some(64);
    let whole = manager.start_whole_game_analysis(request).unwrap();
    budget.continuous_visits_limit = 24;
    manager.set_continuous_preferences(false, budget).unwrap();
    let node = wait_job(&events, Duration::from_secs(2), |job| {
        job.job_id == whole.job_id && job.frame.is_some()
    });
    assert_eq!(node.frame.unwrap().visits, 64);
    assert!(manager.snapshot().selected_node_job.is_none());
    manager.set_continuous_preferences(true, budget).unwrap();
    let continuous = wait_job(&events, Duration::from_secs(2), |job| {
        job.outcome == AnalysisJobOutcomeDto::VisitsLimited
    });
    assert_eq!(continuous.frame.unwrap().visits, 24);
    assert_ne!(continuous.job_id, finite.job_id);
    manager.teardown().unwrap();
}

#[cfg(unix)]
#[test]
fn actual_child_receives_exact_argv_paths_spaces_chinese_and_empty_argument() {
    let temp = TestTempDir::new("argv-exact");
    let cwd = temp.path().join("工作 目录");
    std::fs::create_dir_all(&cwd).unwrap();
    let log = cwd.join("argv.log");
    let executable = cwd.join("引擎 executable.sh");
    let model = cwd.join("模型 data.bin");
    let config = cwd.join("配置 analysis.cfg");
    std::fs::write(&model, "").unwrap();
    std::fs::write(&config, "").unwrap();
    write_executable(
        &executable,
        &format!(
            "printf '<%s>\\n' \"$@\" > '{}'\npwd > cwd.log\n{}",
            log.display(),
            resident_echo_script()
        ),
    );
    let profile = EngineProfileDto {
        name: "exact argv".into(),
        program: executable.to_string_lossy().into_owned(),
        argv: vec![
            "--custom".into(),
            "space value 中文".into(),
            "".into(),
            "  padded  ".into(),
        ],
        working_dir: Some(cwd.to_string_lossy().into_owned()),
        adapter: EngineAdapterSettings::KataGoAnalysis(KataGoSettings {
            model_path: Some("模型 data.bin".into()),
            config_path: Some("配置 analysis.cfg".into()),
            max_visits: 123,
        }),
    };
    let catalog = Arc::new(InMemoryEngineProfileCatalog::new());
    catalog.upsert(SavedEngineProfile {
        profile_id: "argv".into(),
        profile: profile.clone(),
    });
    let manager = ForegroundEngineManager::new(catalog, ForegroundEngineConfig::for_tests());
    let events = manager.subscribe();
    manager.start("argv").unwrap();
    let ready = wait_snapshot(&events, Duration::from_secs(3), |lifecycle| {
        matches!(lifecycle, ForegroundEngineLifecycleDto::Ready { .. })
    });
    assert_eq!(run_from_ready(&ready.lifecycle).profile_snapshot, profile);
    assert_eq!(
        std::fs::read_to_string(&log).unwrap(),
        format!(
            "<analysis>\n<-config>\n<{}>\n<-model>\n<{}>\n<--custom>\n<space value 中文>\n<>\n<  padded  >\n",
            config.display(),
            model.display()
        )
    );
    assert_eq!(
        std::fs::read_to_string(cwd.join("cwd.log")).unwrap(),
        format!("{}\n", cwd.display())
    );
    manager.teardown().unwrap();
}

#[cfg(unix)]
#[test]
fn saving_every_profile_field_and_autoload_keeps_live_run_immutable_until_restart() {
    let temp = TestTempDir::new("immutable-profile");
    let (manager, catalog, events, old_id) = ready_manager(&temp, &resident_echo_script());
    let before = manager.snapshot();
    let before_run = run_from_ready(&before.lifecycle).clone();
    let mut edited = setup_named_profile(&temp, "new 引擎", &resident_echo_script());
    edited.name = "edited name".into();
    edited.argv = vec!["--custom=value".into(), "".into()];
    let cwd = temp.path().join("new cwd");
    std::fs::create_dir_all(&cwd).unwrap();
    edited.working_dir = Some(cwd.to_string_lossy().into_owned());
    if let EngineAdapterSettings::KataGoAnalysis(settings) = &mut edited.adapter {
        settings.max_visits = 321;
    }
    catalog.upsert(SavedEngineProfile {
        profile_id: "profile-1".into(),
        profile: edited.clone(),
    });
    catalog.set_autoload_profile_id(Some("profile-1".into()));
    assert_eq!(manager.snapshot(), before);
    assert_eq!(
        run_from_ready(&manager.snapshot().lifecycle).profile_snapshot,
        before_run.profile_snapshot
    );
    assert_eq!(
        run_from_ready(&manager.snapshot().lifecycle).capability_snapshot,
        before_run.capability_snapshot
    );
    let mut generic = edited.clone();
    generic.adapter = EngineAdapterSettings::GenericGtp(GenericGtpSettings {});
    catalog.upsert(SavedEngineProfile {
        profile_id: "profile-1".into(),
        profile: generic,
    });
    assert_eq!(manager.snapshot(), before);
    catalog.upsert(SavedEngineProfile {
        profile_id: "profile-1".into(),
        profile: edited.clone(),
    });
    manager.restart().unwrap();
    let after = wait_snapshot(
        &events,
        Duration::from_secs(4),
        |lifecycle| matches!(lifecycle, ForegroundEngineLifecycleDto::Ready { run } if run.run_id != old_id),
    );
    assert_eq!(run_from_ready(&after.lifecycle).profile_snapshot, edited);
    assert_eq!(
        run_from_ready(&after.lifecycle).capability_snapshot,
        before_run.capability_snapshot
    );
    manager.teardown().unwrap();
}

#[cfg(unix)]
fn gtp_profile(temp: &TestTempDir, mode: &str) -> EngineProfileDto {
    EngineProfileDto {
        name: format!("GTP {mode}"),
        program: "/usr/bin/python3".into(),
        adapter: EngineAdapterSettings::GenericGtp(GenericGtpSettings {}),
        argv: vec![
            format!("{}/tests/fixtures/gtp_process.py", env!("CARGO_MANIFEST_DIR")),
            mode.into(),
            temp.path().join(mode).to_string_lossy().into(),
        ],
        ..engine_manager::default_engine_profile_record().profile
    }
}

#[cfg(unix)]
fn gtp_manager(
    temp: &TestTempDir,
    mode: &str,
) -> (ForegroundEngineManager, Arc<InMemoryEngineProfileCatalog>) {
    let catalog = Arc::new(InMemoryEngineProfileCatalog::new());
    catalog.upsert(SavedEngineProfile {
        profile_id: "gtp".into(),
        profile: gtp_profile(temp, mode),
    });
    let manager = ForegroundEngineManager::new(
        catalog.clone(),
        ForegroundEngineConfig {
            readiness_timeout: Duration::from_millis(500),
            stop_drain_timeout: Duration::from_millis(100),
            ..ForegroundEngineConfig::for_tests()
        },
    );
    (manager, catalog)
}

#[cfg(unix)]
fn assert_gtp_reaped(temp: &TestTempDir, mode: &str) {
    let pid = std::fs::read_to_string(temp.path().join(format!("{mode}.pid"))).unwrap();
    assert!(
        !Path::new(&format!("/proc/{pid}")).exists(),
        "GTP child {pid} was not reaped"
    );
}

#[cfg(unix)]
#[test]
fn gtp_fragmented_crlf_multiline_and_stderr_publish_only_verified_facts() {
    for mode in ["fragment", "stderr"] {
        let temp = TestTempDir::new(mode);
        let (manager, _) = gtp_manager(&temp, mode);
        manager.start("gtp").unwrap();
        let snapshot = wait_lifecycle(&manager, Duration::from_secs(2), |s| {
            matches!(s, ForegroundEngineLifecycleDto::Ready { .. })
        });
        let run = run_from_ready(&snapshot.lifecycle);
        let caps = run.capability_snapshot.as_ref().unwrap();
        assert_eq!(caps.adapter_kind, EngineBackend::GenericGtp);
        assert!(caps.analysis.is_none());
        let facts = caps.gtp.as_ref().unwrap();
        assert_eq!(
            (&*facts.name, &*facts.version, facts.protocol_version),
            ("Fixture GTP", "0.1", 2)
        );
        assert_eq!(
            facts.commands,
            [
                "boardsize",
                "clear_board",
                "komi",
                "play",
                "genmove",
                "quit",
                "time_settings",
                "time_left"
            ]
        );
        let before = std::fs::read_to_string(temp.path().join(mode)).unwrap();
        assert_eq!(
            manager
                .start_selected_node_job(selected_request(&run.run_id, 1, vec![]))
                .unwrap_err()
                .kind,
            EngineFailureKind::UnsupportedCapability
        );
        assert_eq!(std::fs::read_to_string(temp.path().join(mode)).unwrap(), before);
        assert_eq!(before, "1 protocol_version\n2 name\n3 version\n4 list_commands\n");
        manager.teardown().unwrap();
        assert_gtp_reaped(&temp, mode);
    }
}

#[cfg(unix)]
#[test]
fn gtp_typed_failures_have_total_deadline_and_reap_children() {
    for (mode, expected) in [
        ("badid", EngineFailureKind::Protocol),
        ("badframe", EngineFailureKind::Protocol),
        ("missing", EngineFailureKind::UnsupportedCapability),
        ("v1", EngineFailureKind::UnsupportedCapability),
        ("error", EngineFailureKind::Command),
        ("exit", EngineFailureKind::ProcessExit),
        ("hang", EngineFailureKind::Timeout),
        ("dribble", EngineFailureKind::Timeout),
        ("flood", EngineFailureKind::Protocol),
        ("oversize", EngineFailureKind::Protocol),
    ] {
        let temp = TestTempDir::new(mode);
        let (manager, _) = gtp_manager(&temp, mode);
        let events = manager.subscribe();
        let start = Instant::now();
        manager.start("gtp").unwrap();
        let failure = wait_failure(&events, Duration::from_secs(2), |_| true);
        assert_eq!(failure.kind, expected, "{mode}: {failure:?}");
        assert!(start.elapsed() < Duration::from_secs(2));
        assert!(
            matches!(
                manager.snapshot().lifecycle,
                ForegroundEngineLifecycleDto::NoEngine { failure: Some(_) }
            ),
            "{mode} must require explicit recovery"
        );
        assert_gtp_reaped(&temp, mode);
        manager.teardown().unwrap();
    }
}

#[cfg(unix)]
#[test]
fn gtp_idle_eof_or_unsolicited_response_seals_run_until_manual_restart() {
    for (mode, expected) in [
        ("eof", EngineFailureKind::ProcessExit),
        ("unsolicited", EngineFailureKind::Protocol),
    ] {
        let temp = TestTempDir::new(mode);
        let (manager, catalog) = gtp_manager(&temp, mode);
        manager.set_continuous_preferences(true, app_model::ContinuousAnalysisBudgetDto::default()).unwrap();
        manager.start("gtp").unwrap();
        let ready = wait_lifecycle(&manager, Duration::from_secs(2), |s| {
            matches!(s, ForegroundEngineLifecycleDto::Ready { .. })
        });
        let old_id = run_from_ready(&ready.lifecycle).run_id.clone();
        std::fs::write(temp.path().join(format!("{mode}.idle-release")), "release").unwrap();
        let error = wait_lifecycle(&manager, Duration::from_secs(2), |s| {
            matches!(s, ForegroundEngineLifecycleDto::Error { .. })
        });
        let ForegroundEngineLifecycleDto::Error { failure, .. } = error.lifecycle else {
            unreachable!()
        };
        assert_eq!(failure.kind, expected);
        assert_gtp_reaped(&temp, mode);
        catalog.upsert(SavedEngineProfile {
            profile_id: "gtp".into(),
            profile: gtp_profile(&temp, "good"),
        });
        assert_eq!(
            manager.snapshot().lifecycle,
            ForegroundEngineLifecycleDto::Error {
                run: lifecycle_run(&manager.snapshot().lifecycle).unwrap().clone(),
                failure
            }
        );
        manager.restart().unwrap();
        let ready = wait_lifecycle(&manager, Duration::from_secs(2), |s| {
            matches!(s, ForegroundEngineLifecycleDto::Ready { .. })
        });
        assert_ne!(run_from_ready(&ready.lifecycle).run_id, old_id);
        assert_eq!(ready.continuous.phase, app_model::ContinuousAnalysisPhaseDto::SafetyHold);
        assert!(ready.selected_node_job.is_none());
        manager.teardown().unwrap();
        assert_gtp_reaped(&temp, "good");
    }
}

#[cfg(unix)]
#[test]
fn gtp_cross_protocol_switch_preserves_primary_work_and_latest_intent() {
    let temp = TestTempDir::new("gtp-switch");
    let (manager, catalog, events, primary) = ready_manager(&temp, &hold_then_echo_script());
    let job = manager
        .start_selected_node_job(continuous_request(&primary, 1, vec![]))
        .unwrap();
    for mode in ["badid", "slow", "good"] {
        catalog.upsert(SavedEngineProfile {
            profile_id: mode.into(),
            profile: gtp_profile(&temp, mode),
        });
    }
    manager.switch_to("badid").unwrap();
    let failure = wait_failure(&events, Duration::from_secs(2), |f| f.switch_id.is_some());
    assert_eq!(failure.kind, EngineFailureKind::Protocol);
    let snapshot = manager.snapshot();
    assert_eq!(run_from_ready(&snapshot.lifecycle).run_id, primary);
    assert_eq!(snapshot.selected_node_job.unwrap().job_id, job.job_id);
    manager.switch_to("slow").unwrap();
    manager.switch_to("good").unwrap();
    let ready = wait_lifecycle(
        &manager,
        Duration::from_secs(3),
        |s| matches!(s, ForegroundEngineLifecycleDto::Ready { run } if run.profile_id == "good"),
    );
    assert_eq!(
        run_from_ready(&ready.lifecycle).adapter_kind,
        EngineBackend::GenericGtp
    );
    manager.switch_to("profile-1").unwrap();
    let ready = wait_lifecycle(
        &manager,
        Duration::from_secs(3),
        |s| matches!(s, ForegroundEngineLifecycleDto::Ready { run } if run.profile_id == "profile-1"),
    );
    assert!(run_from_ready(&ready.lifecycle)
        .capability_snapshot
        .as_ref()
        .unwrap()
        .gtp
        .is_none());
    manager.teardown().unwrap();
    assert_gtp_reaped(&temp, "good");
}

#[cfg(unix)]
#[test]
fn gtp_stop_during_handshake_cannot_publish_late_ready() {
    let temp = TestTempDir::new("gtp-stop");
    let (manager, _) = gtp_manager(&temp, "slow");
    let events = manager.subscribe();
    manager.start("gtp").unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    while !temp.path().join("slow.pid").exists() {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(5));
    }
    manager.stop().unwrap();
    wait_lifecycle(&manager, Duration::from_secs(2), |s| {
        matches!(s, ForegroundEngineLifecycleDto::NoEngine { .. })
    });
    assert_gtp_reaped(&temp, "slow");
    std::thread::sleep(Duration::from_millis(850));
    assert!(events.try_iter().all(|e| !matches!(e, ForegroundEngineEventDto::Snapshot { snapshot } if matches!(snapshot.lifecycle, ForegroundEngineLifecycleDto::Ready { .. }))));
}

#[cfg(unix)]
#[test]
fn local_qualification_binds_actual_content_to_the_run_without_managed_trust() {
    let temp = TestTempDir::new("resource-identity");
    let (manager, _, _, _) = ready_manager(&temp, &resident_echo_script());
    let snapshot = manager.snapshot();
    let run = run_from_ready(&snapshot.lifecycle);
    let identity = run.qualified_resource.as_ref().expect("qualified resource");
    assert_eq!(identity.origin, "local_unknown");
    assert!(!identity.static_zlib_exemption);
    assert_eq!(identity.profile_revision.len(), 64);
    let model = identity
        .resources
        .iter()
        .find(|resource| resource.component == "model")
        .unwrap();
    assert_eq!(
        model.sha256,
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    assert_eq!(model.bytes, 0);
    assert!(!model
        .resolved_path
        .contains(&temp.path().to_string_lossy().to_string()));
    let diagnostics = manager.diagnostic_snapshots().pop().unwrap();
    assert!(diagnostics.command.contains(&model.resolved_path));
    manager.teardown().unwrap();
}

#[cfg(unix)]
#[test]
fn diagnostics_capture_live_burst_without_waiting_for_exit_and_keep_frozen_identity() {
    let temp = TestTempDir::new("diagnostics-burst");
    let release = temp.path().join("release");
    let script = format!(r#"exec python3 -u -c '
import json,sys,time,pathlib
q=json.loads(sys.stdin.readline())
print(json.dumps(dict(id=q["id"],turnNumber=0,isDuringSearch=False,rootInfo=dict(visits=4,winrate=0.6,scoreMean=1.25),moveInfos=[])),flush=True)
while not pathlib.Path("{}").exists(): time.sleep(0.005)
for i in range(3000): print("WARN burst " + str(i) + " " + "x"*1000,file=sys.stderr)
print("token=FAKE-secret with spaces",file=sys.stderr)
print("roomId=424242",file=sys.stderr)
print("http://fake.invalid/private?token=FAKE-url",file=sys.stderr)
print("/home/fake-user/private/model.bin",file=sys.stderr)
print("diagnostic-barrier",file=sys.stderr,flush=True)
while True: time.sleep(1)
'"#, release.display());
    let (manager, _, _, first_run) = ready_manager(&temp, &script);
    let before = manager.diagnostic_snapshots().pop().unwrap();
    assert_eq!(before.run_id, first_run);
    assert!(!before.full_trace);
    assert!(before.records.iter().any(|record| record.source == "startup-probe-stdout"));
    std::fs::write(release, "go").unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    let captured = loop {
        let snapshot = manager.diagnostic_snapshots().pop().unwrap();
        if snapshot.records.iter().any(|record| record.text == "diagnostic-barrier") { break snapshot; }
        assert!(Instant::now() < deadline, "live stderr was not observed");
        std::thread::sleep(Duration::from_millis(10));
    };
    assert!(captured.dropped_records > 0);
    assert!(captured.retained_bytes <= 64 * 1024);
    assert!(captured.records.len() <= 256);
    let encoded = serde_json::to_string(&captured).unwrap();
    for private in ["FAKE-secret", "424242", "fake.invalid", "fake-user"] { assert!(!encoded.contains(private), "leaked {private}"); }
    assert!(captured.records.windows(2).all(|records| records[0].sequence < records[1].sequence));
    assert!(captured.metrics.iter().any(|metric| metric.name == "resident-memory" && metric.value.is_some_and(|value| value > 0.0)));
    manager.stop().unwrap();
    wait_lifecycle(&manager, Duration::from_secs(3), |state| matches!(state, ForegroundEngineLifecycleDto::NoEngine { .. }));
    assert_eq!(captured.run_id, first_run);
    assert_eq!(serde_json::to_string(&captured).unwrap(), encoded);
}

#[cfg(unix)]
#[test]
fn resource_replacement_and_saved_revision_changes_cannot_promote_a_late_candidate() {
    for change in ["model", "config", "layered_include", "executable", "profile"] {
        let temp = TestTempDir::new(change);
        let release = temp.path().join("release-resource");
        let entered = temp.path().join("entered-resource");
        let script = format!(
            "touch '{}'\n{}",
            entered.display(),
            hold_probe_until_release_script(&release)
        );
        let (manager, catalog, events, run_a) = ready_two_profiles(&temp, &resident_echo_script(), &script);
        if change == "layered_include" {
            std::fs::write(temp.path().join("cache-defaults.cfg"), "@include='custom-cache.cfg'\n").unwrap();
            std::fs::write(temp.path().join("custom-cache.cfg"), "homeDataDir = original\n").unwrap();
            let mut saved = catalog.get("profile-b").unwrap();
            saved.profile.argv.extend(["-config".into(), temp.path().join("cache-defaults.cfg").to_string_lossy().into_owned()]);
            catalog.upsert(saved);
        }
        let before = manager.snapshot();
        manager.switch_to("profile-b").unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        while !entered.exists() {
            assert!(Instant::now() < deadline, "candidate did not enter probe");
            std::thread::sleep(Duration::from_millis(5));
        }
        match change {
            "model" => std::fs::write(temp.path().join("engine-b.bin"), b"replacement model").unwrap(),
            "config" => std::fs::write(temp.path().join("engine-b.cfg"), b"numSearchThreads=3").unwrap(),
            "layered_include" => std::fs::write(temp.path().join("custom-cache.cfg"), "homeDataDir = replacement\n").unwrap(),
            "executable" => {
                let replacement = temp.path().join("replacement.sh");
                write_executable(&replacement, &resident_echo_script());
                std::fs::rename(replacement, temp.path().join("engine-b.sh")).unwrap();
            }
            _ => {
                let mut saved = catalog.get("profile-b").unwrap();
                saved.profile.name = "new saved revision".into();
                catalog.upsert(saved);
            }
        }
        std::fs::write(&release, "release").unwrap();
        let failure = wait_failure(&events, Duration::from_secs(3), |failure| {
            failure.operation == EngineOperationDto::Switch
        });
        assert_eq!(failure.kind, EngineFailureKind::ResourceChanged, "{failure:?}");
        assert_eq!(failure.profile_id.as_deref(), Some("profile-b"));
        assert_ready_a(
            &manager.snapshot().lifecycle,
            &run_a,
            run_from_ready(&before.lifecycle),
        );
        manager.teardown().unwrap();
    }
}

#[cfg(unix)]
#[test]
fn startup_dependency_failures_are_bounded_redacted_and_never_replace_ready_a() {
    for (component, expected) in [
        ("nvonnxparser.dll missing", EngineFailureKind::TensorRtParser),
        ("cudnn64_9.dll missing", EngineFailureKind::Cudnn),
        ("nvrtc64.dll missing", EngineFailureKind::Nvrtc),
        ("CUDA driver error", EngineFailureKind::Cuda),
        ("zlib1.dll missing", EngineFailureKind::Zlib),
    ] {
        let temp = TestTempDir::new("dependency-redaction");
        let script = format!("printf '%s\\n' '{component} token=fake-secret session_id=fake-session https://example.test/private /home/alice/private-model' >&2\nexit 7");
        let (manager, _, events, run_a) = ready_two_profiles(&temp, &resident_echo_script(), &script);
        manager.switch_to("profile-b").unwrap();
        let failure = wait_failure(&events, Duration::from_secs(3), |failure| {
            failure.operation == EngineOperationDto::Switch
        });
        assert_eq!(failure.kind, expected, "{failure:?}");
        let summary = failure.diagnostic_summary.unwrap();
        assert!(summary.len() <= 8192);
        for secret in ["fake-secret", "fake-session", "example.test", "/home/alice", "\u{1}"] {
            assert!(!summary.contains(secret), "{summary}");
        }
        let diagnostics = manager.diagnostic_snapshots().pop().unwrap();
        assert_eq!(Some(diagnostics.run_id.as_str()), failure.run_id.as_deref());
        assert!(diagnostics.failure.as_ref().unwrap().contains(&format!("{expected:?}")));
        assert_eq!(run_from_ready(&manager.snapshot().lifecycle).run_id, run_a);
        manager.teardown().unwrap();
    }
}

#[cfg(unix)]
#[test]
fn diagnostics_preserve_healthy_primary_through_failed_attempts_and_cancel_burst() {
    let temp = TestTempDir::new("diagnostics-retention");
    let release = temp.path().join("release");
    let script = format!(r#"exec python3 -u -c '
import json,sys,time,pathlib
q=json.loads(sys.stdin.readline())
print(json.dumps(dict(id=q["id"],turnNumber=0,isDuringSearch=False,rootInfo=dict(visits=4,winrate=0.6,scoreMean=1.25),moveInfos=[])),flush=True)
while not pathlib.Path("{}").exists(): time.sleep(0.005)
while True: print("WARN continuous " + "x"*1000,file=sys.stderr,flush=True)
'"#, release.display());
    let (manager, catalog, events, primary) = ready_manager(&temp, &script);
    for index in 0..6 {
        let id = format!("missing-{index}");
        catalog.upsert(SavedEngineProfile { profile_id: id.clone(), profile: missing_assets_profile(&id, &temp) });
        manager.switch_to(&id).unwrap();
        wait_failure(&events, Duration::from_secs(2), |failure| failure.profile_id.as_deref() == Some(id.as_str()));
        assert_eq!(run_from_ready(&manager.snapshot().lifecycle).run_id, primary);
    }
    let snapshots = manager.diagnostic_snapshots();
    assert_eq!(snapshots.len(), 4);
    assert_eq!(snapshots[0].run_id, primary);
    assert!(snapshots.iter().skip(1).all(|snapshot| snapshot.failure.is_some() && snapshot.run_id != primary));
    std::fs::write(release, "go").unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    while manager.diagnostic_snapshots()[0].dropped_records == 0 {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(5));
    }
    manager.stop().unwrap();
    wait_lifecycle(&manager, Duration::from_secs(3), |state| matches!(state, ForegroundEngineLifecycleDto::NoEngine { .. }));
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let snapshot = manager.diagnostic_snapshots().remove(0);
        if snapshot.process_exited && snapshot.stderr_complete && snapshot.stdout_complete { break; }
        assert!(Instant::now() < deadline, "burst readers did not retire");
        std::thread::sleep(Duration::from_millis(5));
    }
}

#[test]
fn last_primary_tracks_only_successful_promotion_and_survives_stop() {
    let temp = TestTempDir::new("last-primary-promotions");
    let catalog = Arc::new(InMemoryEngineProfileCatalog::new());
    for id in ["a", "b"] {
        catalog.upsert(SavedEngineProfile {
            profile_id: id.into(),
            profile: setup_named_profile(&temp, id, &resident_echo_script()),
        });
    }
    let manager = ForegroundEngineManager::new(catalog, ForegroundEngineConfig::for_tests());
    assert_eq!(manager.last_primary_profile_id(), None);
    assert!(manager.start("missing").is_err());
    assert_eq!(manager.last_primary_profile_id(), None);
    manager.start("a").unwrap();
    wait_lifecycle(&manager, Duration::from_secs(3), |phase| matches!(phase, ForegroundEngineLifecycleDto::Ready { .. }));
    assert_eq!(manager.last_primary_profile_id().as_deref(), Some("a"));
    assert!(manager.switch_to("missing").is_err());
    assert_eq!(manager.last_primary_profile_id().as_deref(), Some("a"));
    manager.switch_to("b").unwrap();
    wait_lifecycle(&manager, Duration::from_secs(3), |phase| matches!(phase, ForegroundEngineLifecycleDto::Ready { run } if run.profile_id == "b"));
    assert_eq!(manager.last_primary_profile_id().as_deref(), Some("b"));
    manager.stop().unwrap();
    assert_eq!(manager.last_primary_profile_id().as_deref(), Some("b"));
}

#[cfg(unix)]
fn wait_preload(
    manager: &ForegroundEngineManager,
    phase: app_model::EnginePreloadPhaseDto,
) -> app_model::EnginePreloadDto {
    let deadline = Instant::now() + Duration::from_secs(4);
    loop {
        let slots = manager.preload_snapshot();
        if let Some(slot) = slots.iter().find(|slot| slot.phase == phase) {
            return slot.clone();
        }
        assert!(Instant::now() < deadline, "preload timeout: {slots:?}");
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[cfg(unix)]
#[test]
fn preload_requires_opt_in_and_only_explicit_switch_promotes_same_child() {
    use app_model::EnginePreloadPhaseDto as Phase;
    let temp = TestTempDir::new("preload-promotion");
    let (manager, catalog, _, a) =
        ready_two_profiles(&temp, &resident_echo_script(), &resident_echo_script());
    assert!(manager.prepare_preload("profile-b").is_err());
    catalog.set_preload("profile-b", true);
    manager.prepare_preload("profile-b").unwrap();
    let prepared = wait_preload(&manager, Phase::Ready);
    assert_eq!(run_from_ready(&manager.snapshot().lifecycle).run_id, a);
    assert!(manager
        .start_selected_node_job(selected_request(&prepared.run.run_id, 1, vec![]))
        .is_err());
    assert_eq!(manager.preload_snapshot()[0].phase, Phase::Ready);
    manager.switch_to("profile-b").unwrap();
    assert_eq!(
        run_from_ready(&manager.snapshot().lifecycle).run_id,
        prepared.run.run_id
    );
    assert!(manager.preload_snapshot().is_empty());
    manager.teardown().unwrap();
}

#[cfg(unix)]
#[test]
fn preload_changed_resource_or_profile_cannot_replace_healthy_primary() {
    use app_model::EnginePreloadPhaseDto as Phase;
    for change_profile in [false, true] {
        let temp = TestTempDir::new("preload-stale");
        let (manager, catalog, _, a) =
            ready_two_profiles(&temp, &resident_echo_script(), &resident_echo_script());
        catalog.set_preload("profile-b", true);
        manager.prepare_preload("profile-b").unwrap();
        wait_preload(&manager, Phase::Ready);
        if change_profile {
            let mut saved = catalog.get("profile-b").unwrap();
            saved.profile.name = "changed revision".into();
            catalog.upsert(saved);
        } else {
            std::fs::write(temp.path().join("engine-b.bin"), "replacement").unwrap();
        }
        assert!(manager.switch_to("profile-b").is_err());
        assert_eq!(wait_preload(&manager, Phase::Failed).run.profile_id, "profile-b");
        assert_eq!(run_from_ready(&manager.snapshot().lifecycle).run_id, a);
        manager.teardown().unwrap();
    }
}

#[cfg(unix)]
#[test]
fn preload_cancel_and_foreground_admission_retire_late_preparation() {
    use app_model::EnginePreloadPhaseDto as Phase;
    for foreground in [false, true] {
        let temp = TestTempDir::new("preload-cancel");
        let release = temp.path().join("release");
        let started = temp.path().join("started");
        let script = format!(
            "echo $$ > '{}'\n{}",
            started.display(),
            hold_probe_until_release_script(&release)
        );
        let (manager, catalog, _, a) = ready_two_profiles(&temp, &resident_echo_script(), &script);
        catalog.set_preload("profile-b", true);
        manager.prepare_preload("profile-b").unwrap();
        wait_preload(&manager, Phase::Preparing);
        let deadline = Instant::now() + Duration::from_secs(3);
        while !started.exists() {
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(5));
        }
        let pid = std::fs::read_to_string(&started).unwrap();
        if foreground {
            manager
                .start_selected_node_job(selected_request(&a, 1, vec![]))
                .unwrap();
        } else {
            manager.cancel_preload("profile-b").unwrap();
        }
        std::fs::write(&release, "release").unwrap();
        wait_preload(&manager, Phase::Cancelled);
        manager.cancel_preload("profile-b").unwrap();
        #[cfg(target_os = "linux")]
        assert!(
            !Path::new(&format!("/proc/{}", pid.trim())).exists(),
            "owned preparation child must be reaped"
        );
        assert_eq!(manager.preload_snapshot()[0].phase, Phase::Cancelled);
        assert_eq!(run_from_ready(&manager.snapshot().lifecycle).run_id, a);
        manager.teardown().unwrap();
        assert!(manager.prepare_preload("profile-b").is_err());
    }
}

#[cfg(unix)]
#[test]
fn preload_resource_failure_preserves_primary() {
    use app_model::EnginePreloadPhaseDto as Phase;
    let temp = TestTempDir::new("preload-failure");
    let (manager, catalog, _, a) = ready_two_profiles(
        &temp,
        &resident_echo_script(),
        "echo 'model allocation failed' >&2; exit 1",
    );
    catalog.set_preload("profile-b", true);
    manager.prepare_preload("profile-b").unwrap();
    assert!(wait_preload(&manager, Phase::Failed).failure.is_some());
    assert_eq!(run_from_ready(&manager.snapshot().lifecycle).run_id, a);
    manager.teardown().unwrap();
}

#[cfg(unix)]
#[test]
fn preload_ready_yields_to_match_reservation_without_taking_role() {
    use app_model::EnginePreloadPhaseDto as Phase;
    let temp = TestTempDir::new("preload-match");
    let (manager, catalog, _, a) =
        ready_two_profiles(&temp, &resident_echo_script(), &resident_echo_script());
    catalog.set_preload("profile-b", true);
    manager.prepare_preload("profile-b").unwrap();
    wait_preload(&manager, Phase::Ready);
    manager.reserve_match("match-owner").unwrap();
    assert_eq!(manager.preload_snapshot()[0].phase, Phase::Cancelled);
    assert_eq!(manager.match_reservation_owner().as_deref(), Some("match-owner"));
    assert!(manager.prepare_preload("profile-b").is_err());
    assert_eq!(run_from_ready(&manager.snapshot().lifecycle).run_id, a);
    manager.teardown().unwrap();
}

#[cfg(unix)]
#[test]
fn preload_gtp_readiness_keeps_foreground_authority_until_same_run_promotion() {
    let temp = TestTempDir::new("preload-gtp");
    let (manager, catalog) = gtp_manager(&temp, "fragment");
    catalog.upsert(SavedEngineProfile {
        profile_id: "background".into(),
        profile: gtp_profile(&temp, "stderr"),
    });
    catalog.set_preload("background", true);
    manager.start("gtp").unwrap();
    let ready = wait_lifecycle(&manager, Duration::from_secs(3), |state| {
        matches!(state, ForegroundEngineLifecycleDto::Ready { .. })
    });
    let a = run_from_ready(&ready.lifecycle).run_id.clone();
    manager.prepare_preload("background").unwrap();
    let background = wait_preload(&manager, app_model::EnginePreloadPhaseDto::Ready);
    assert!(background
        .run
        .capability_snapshot
        .as_ref()
        .unwrap()
        .analysis
        .is_none());
    assert_eq!(run_from_ready(&manager.snapshot().lifecycle).run_id, a);
    manager.switch_to("background").unwrap();
    assert_eq!(
        run_from_ready(&manager.snapshot().lifecycle).run_id,
        background.run.run_id
    );
    manager.teardown().unwrap();
    assert_gtp_reaped(&temp, "fragment");
    assert_gtp_reaped(&temp, "stderr");
}

#[cfg(unix)]
fn held_startup_preload(
    label: &str,
    opted_in: bool,
    readiness_timeout: Duration,
) -> (TestTempDir, ForegroundEngineManager, Arc<InMemoryEngineProfileCatalog>, PathBuf) {
    let temp = TestTempDir::new(label);
    let release = temp.path().join("startup-release");
    let catalog = Arc::new(InMemoryEngineProfileCatalog::new());
    catalog.upsert(SavedEngineProfile {
        profile_id: "a".into(),
        profile: setup_named_profile(&temp, "a", &hold_probe_until_release_script(&release)),
    });
    catalog.upsert(SavedEngineProfile {
        profile_id: "b".into(),
        profile: setup_named_profile(&temp, "b", &resident_echo_script()),
    });
    catalog.set_preload("b", opted_in);
    catalog.set_autoload_profile_id(Some("a".into()));
    let mut config = ForegroundEngineConfig::for_tests();
    config.readiness_timeout = readiness_timeout;
    let manager = ForegroundEngineManager::new(catalog.clone(), config);
    manager.apply_autoload().unwrap();
    manager.schedule_startup_preloads();
    wait_lifecycle(&manager, Duration::from_secs(3), |state| {
        matches!(state, ForegroundEngineLifecycleDto::Starting { .. })
    });
    (temp, manager, catalog, release)
}

#[cfg(unix)]
#[test]
fn startup_preload_save_during_starting_requires_explicit_prepare() {
    let (_temp, manager, catalog, release) =
        held_startup_preload("startup-preload-save", false, Duration::from_secs(10));
    catalog.set_preload("b", true);
    std::fs::write(release, "release").unwrap();
    wait_lifecycle(&manager, Duration::from_secs(3), |state| {
        matches!(state, ForegroundEngineLifecycleDto::Ready { .. })
    });
    std::thread::sleep(Duration::from_millis(300));
    assert!(manager.preload_snapshot().is_empty());
    manager.prepare_preload("b").unwrap();
    wait_preload(&manager, app_model::EnginePreloadPhaseDto::Ready);
    manager.teardown().unwrap();
}

#[cfg(unix)]
#[test]
#[ignore = "holds a real startup pipe beyond the former 120-second scheduler horizon"]
fn startup_preload_survives_long_autoload_and_consumes_once() {
    use app_model::EnginePreloadPhaseDto as Phase;
    let (_temp, manager, _catalog, release) =
        held_startup_preload("startup-preload-long", true, Duration::from_secs(140));
    let started = Instant::now();
    std::thread::sleep(Duration::from_secs(122));
    assert!(matches!(manager.snapshot().lifecycle, ForegroundEngineLifecycleDto::Starting { .. }));
    std::fs::write(release, "release").unwrap();
    let prepared = wait_preload(&manager, Phase::Ready);
    eprintln!("SPEC-002 original autoload held {:?}; prepared {}", started.elapsed(), prepared.run.run_id);
    manager.schedule_startup_preloads();
    manager.apply_preloads();
    assert_eq!(manager.preload_snapshot().len(), 1);
    assert_eq!(manager.preload_snapshot()[0].run.run_id, prepared.run.run_id);
    manager.cancel_preload("b").unwrap();
    manager.schedule_startup_preloads();
    std::thread::sleep(Duration::from_millis(100));
    assert_eq!(manager.preload_snapshot()[0].phase, Phase::Cancelled);
    manager.teardown().unwrap();
}

#[cfg(unix)]
#[test]
fn startup_preload_operation_shutdown_and_foreground_retire_authorization() {
    for action in ["stop", "shutdown", "foreground"] {
        let (_temp, manager, _catalog, release) =
            held_startup_preload("startup-preload-retired", true, Duration::from_secs(10));
        match action {
            "stop" => manager.stop().unwrap(),
            "shutdown" => manager.teardown().unwrap(),
            _ => {
                std::fs::write(&release, "release").unwrap();
                wait_lifecycle(&manager, Duration::from_secs(3), |state| {
                    matches!(state, ForegroundEngineLifecycleDto::Ready { .. })
                });
                manager.reserve_match("startup-priority").unwrap();
                manager.stop_reserved_match("startup-priority").unwrap();
            }
        }
        std::fs::write(release, "release").unwrap();
        manager.schedule_startup_preloads();
        std::thread::sleep(Duration::from_millis(300));
        assert!(manager.preload_snapshot().iter().all(|slot| {
            slot.phase == app_model::EnginePreloadPhaseDto::Cancelled
        }), "{action}");
        manager.teardown().unwrap();
    }
}

#[cfg(unix)]
#[test]
fn startup_preload_changed_or_deleted_saved_target_cannot_launch_replacement() {
    struct DiskCatalog(PathBuf);
    impl EngineProfileCatalog for DiskCatalog {
        fn get(&self, id: &str) -> Option<SavedEngineProfile> {
            self.preload_profiles().into_iter().find(|saved| saved.profile_id == id)
        }
        fn preload_profiles(&self) -> Vec<SavedEngineProfile> {
            engine_manager::load_engine_profiles(&self.0).unwrap().profiles.into_iter()
                .filter(|record| record.preload)
                .map(|record| SavedEngineProfile { profile_id: record.id, profile: record.profile })
                .collect()
        }
    }
    for deleted in [false, true] {
        let temp = TestTempDir::new("startup-preload-revision");
        let path = temp.path().join("catalog.json");
        let mut settings = engine_manager::default_engine_profiles_settings();
        settings.profiles.push(engine_manager::EngineProfileRecord {
            id: "b".into(), preload: true,
            profile: setup_named_profile(&temp, "b", &resident_echo_script()),
        });
        engine_manager::save_engine_profiles(&path, settings.clone()).unwrap();
        let manager = ForegroundEngineManager::new(
            Arc::new(DiskCatalog(path.clone())), ForegroundEngineConfig::for_tests());
        if deleted {
            settings.profiles.retain(|record| record.id != "b");
        } else {
            settings.profiles.last_mut().unwrap().profile.name = "replacement revision".into();
        }
        engine_manager::save_engine_profiles(&path, settings).unwrap();
        manager.schedule_startup_preloads();
        std::thread::sleep(Duration::from_millis(300));
        assert!(manager.preload_snapshot().is_empty());
        manager.teardown().unwrap();
        if !deleted {
            let fresh = ForegroundEngineManager::new(
                Arc::new(DiskCatalog(path)), ForegroundEngineConfig::for_tests());
            fresh.schedule_startup_preloads();
            let ready = wait_preload(&fresh, app_model::EnginePreloadPhaseDto::Ready);
            assert_eq!(ready.run.profile_snapshot.name, "replacement revision");
            fresh.teardown().unwrap();
        }
    }
}

#[cfg(unix)]
#[test]
fn preload_preparing_and_ready_retire_when_paused_task_continues() {
    use app_model::EnginePreloadPhaseDto as Phase;
    for ready in [false, true] {
        let temp = TestTempDir::new("preload-task-continue");
        let (manager, catalog, events, run) = ready_manager(&temp, &task_engine_script(temp.path()));
        let task = manager.start_analysis_task(
            whole_game_request(&run, 42, 3), analysis_scope(), task_conditions(32)).unwrap();
        wait_job(&events, Duration::from_secs(2), |job| {
            job.job_id == task.job_id && job.outcome == AnalysisJobOutcomeDto::Progress
        });
        manager.pause_analysis_task(&run, &task.task_id).unwrap();
        std::fs::write(temp.path().join("cancel-final"), "go").unwrap();
        wait_task(&manager, AnalysisTaskStateDto::Paused);
        let release = temp.path().join("preload-release");
        let pid_file = temp.path().join("preload-pid");
        let script = format!("echo $$ > '{}'\n{}", pid_file.display(),
            if ready { resident_echo_script() } else { hold_probe_until_release_script(&release) });
        catalog.upsert(SavedEngineProfile {
            profile_id: "b".into(), profile: setup_named_profile(&temp, "background", &script),
        });
        catalog.set_preload("b", true);
        manager.prepare_preload("b").unwrap();
        wait_preload(&manager, if ready { Phase::Ready } else { Phase::Preparing });
        let deadline = Instant::now() + Duration::from_secs(3);
        while !pid_file.exists() {
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(5));
        }
        let pid = std::fs::read_to_string(pid_file).unwrap();
        let continued = manager.continue_analysis_task(&run, &task.task_id, 42).unwrap();
        assert_eq!(continued.state, AnalysisTaskStateDto::Queued);
        assert_eq!(continued.task_id, task.task_id);
        assert_eq!(continued.run_id, run);
        assert_eq!(manager.preload_snapshot()[0].phase, Phase::Cancelled);
        std::fs::write(release, "late release").unwrap();
        #[cfg(target_os = "linux")]
        while Path::new(&format!("/proc/{}", pid.trim())).exists() {
            assert!(Instant::now() < deadline, "Continue must reap its background child");
            std::thread::sleep(Duration::from_millis(10));
        }
        assert_eq!(manager.preload_snapshot()[0].phase, Phase::Cancelled);
        assert_eq!(run_from_ready(&manager.snapshot().lifecycle).run_id, run);
        manager.teardown().unwrap();
    }
}
