use app_model::*;
use engine_manager::*;
use std::{sync::Arc, time::{Duration, Instant}};

struct OwnedManager(ForegroundEngineManager);
impl Drop for OwnedManager { fn drop(&mut self) { let _ = self.0.teardown(); } }

fn ready(manager: &ForegroundEngineManager) -> EngineRunDto {
    let deadline = Instant::now() + Duration::from_secs(40);
    loop {
        match manager.snapshot().lifecycle {
            ForegroundEngineLifecycleDto::Ready { run } => return run,
            ForegroundEngineLifecycleDto::Error { failure, .. } => panic!("{failure:?}"),
            other => assert!(Instant::now() < deadline, "{other:?}"),
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}
fn request(manager: &ForegroundEngineManager, action: RuntimeThreadsActionDto, value: Option<u32>) -> RuntimeThreadsRequestDto {
    let state = manager.runtime_threads_snapshot();
    RuntimeThreadsRequestDto { identity: RuntimeControlIdentityDto { run_id: state.run_id.unwrap(), profile_revision: state.profile_revision.unwrap(), request_id: uuid::Uuid::new_v4().to_string() }, action, value }
}
fn progress(events: &std::sync::mpsc::Receiver<ForegroundEngineEventDto>, after: Option<&str>) -> AnalysisJobEventDto {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        match events.recv_timeout(deadline.saturating_duration_since(Instant::now())).unwrap() {
            ForegroundEngineEventDto::Job { job } if job.outcome == AnalysisJobOutcomeDto::Progress && Some(job.job_id.as_str()) != after => return job,
            ForegroundEngineEventDto::Failure { failure } => panic!("{failure:?}"),
            _ => {},
        }
    }
}

#[cfg(target_os = "linux")]
fn owned_engine_identity(engine: &str) -> (u32, String) {
    let executable = std::fs::canonicalize(engine).unwrap();
    let mut result = Vec::new();
    // Observe only children of this test process; never terminate a PID found by enumeration.
    for task in std::fs::read_dir("/proc/self/task").unwrap() {
        if let Ok(children) = std::fs::read_to_string(task.unwrap().path().join("children")) {
            for child in children.split_whitespace() {
                if std::fs::read_link(format!("/proc/{child}/exe")).ok().as_ref() == Some(&executable) {
                    let stat = std::fs::read_to_string(format!("/proc/{child}/stat")).unwrap();
                    let start = stat.rsplit_once(')').unwrap().1.split_whitespace().nth(19).unwrap().to_owned();
                    result.push((child.parse().unwrap(), start));
                }
            }
        }
    }
    result.sort(); result.dedup();
    assert_eq!(result.len(), 1, "one actual manager-owned foreground engine");
    result.pop().unwrap()
}

#[test]
#[cfg(target_os = "linux")]
#[ignore = "requires frozen real KataGo 1.18.2 resources and private config/cwd"]
fn real_runtime_threads_same_pid_analysis_pause_reset_and_expiry() {
    assert_eq!(std::env::var("LIZZIEYZY_REAL_KATAGO").unwrap(), "1");
    let engine = std::env::var("LIZZIEYZY_KATAGO_ENGINE").unwrap();
    let config = std::env::var("LIZZIEYZY_KATAGO_CONFIG").unwrap();
    let original_bytes = std::fs::read(&config).unwrap();
    let catalog = Arc::new(InMemoryEngineProfileCatalog::new());
    catalog.upsert(SavedEngineProfile { profile_id: "threads".into(), profile: EngineProfileDto {
        name: "manual GTP threads".into(), program: engine.clone(),
        argv: vec!["-config".into(), config.clone(), "-override-config".into(), "numSearchThreads=1".into()],
        working_dir: Some(std::env::var("LIZZIEYZY_KATAGO_WORKDIR").unwrap()),
        adapter: EngineAdapterSettings::KataGoGtp(KataGoSettings { model_path: Some(std::env::var("LIZZIEYZY_KATAGO_MODEL").unwrap()), config_path: Some(config.clone()), max_visits: 4 }),
    }});
    let owner = OwnedManager(ForegroundEngineManager::new(catalog.clone(), ForegroundEngineConfig::default()));
    let manager = &owner.0;
    manager.start("threads").unwrap();
    let run = ready(manager);
    let pid = owned_engine_identity(&engine);
    println!("REAL_THREAD_RUN {} pid={pid:?}", serde_json::to_string(&run).unwrap());
    let initial = manager.runtime_threads(request(manager, RuntimeThreadsActionDto::Read, None)).unwrap();
    assert_eq!(initial.status, RuntimeThreadsStatusDto::Confirmed);
    assert_eq!(initial.actual, initial.sources.as_ref().unwrap().effective);
    // Exercise the protocol domain, not a 4096-worker benchmark: no search exists here,
    // and the recorded launch value is restored before analysis admission.
    let upper = manager.runtime_threads(request(manager, RuntimeThreadsActionDto::Apply, Some(4096))).unwrap();
    println!("REAL_THREAD_DOMAIN {}", serde_json::to_string(&upper).unwrap());
    assert_eq!(upper.status, RuntimeThreadsStatusDto::Confirmed);
    assert_eq!(upper.actual, Some(4096));
    let lower = manager.runtime_threads(request(manager, RuntimeThreadsActionDto::Apply, Some(1))).unwrap();
    assert_eq!(lower.actual, Some(1));
    manager.runtime_threads(request(manager, RuntimeThreadsActionDto::Reset, None)).unwrap();
    let document = sgf::CurrentSgfDocument::open("(;SZ[9]RU[Chinese]KM[7.5]C[personal];B[dd])").unwrap();
    let before = document.serialize().unwrap();
    let node_path = NodePath { indices: vec![0] };
    let position = document.snapshot(&node_path).unwrap();
    let selected = SelectedNodeJobRequest { run_id: run.run_id.clone(), generation: 11, node_path: node_path.clone(),
        mode: AnalysisJobModeDto::Continuous, board_width: 9, board_height: 9, position_empty: false,
        exact_position: Ok(document.exact_position(&node_path).unwrap()),
        query: katago_protocol::analysis_query_from_position(9, 9, 7.5, &position.position.stones, PlayerColor::White,
            katago_protocol::AnalysisQueryOptions { id: "pending".into(), rules: "chinese".into(), turn: 1, max_visits: None, include_ownership: Some(true), include_policy: Some(false) }).unwrap(),
    };
    let events = manager.subscribe();
    let budget = ContinuousAnalysisBudgetDto { continuous_time_limit_enabled: false, ..Default::default() };
    manager.set_continuous_preferences(true, budget).unwrap();
    manager.follow_continuous_position(selected);
    let first = progress(&events, None);
    let applied = manager.runtime_threads(request(manager, RuntimeThreadsActionDto::Apply, Some(2))).unwrap();
    println!("REAL_THREAD_APPLY {}", serde_json::to_string(&applied).unwrap());
    assert_eq!(applied.status, RuntimeThreadsStatusDto::Confirmed);
    assert_eq!(applied.actual, Some(2));
    assert!(applied.temporary);
    let resumed = progress(&events, Some(&first.job_id));
    assert_eq!(resumed.run_id, run.run_id);
    assert_eq!(resumed.generation, 11);
    assert_eq!(resumed.node_path, node_path);
    assert!(resumed.frame.as_ref().unwrap().visits > 0);
    assert_eq!(owned_engine_identity(&engine), pid);
    manager.set_continuous_preferences(false, budget).unwrap();
    let deadline = Instant::now() + Duration::from_secs(8);
    while manager.snapshot().selected_node_job.is_some() { assert!(Instant::now() < deadline); std::thread::sleep(Duration::from_millis(10)); }
    let applied = manager.runtime_threads(request(manager, RuntimeThreadsActionDto::Apply, Some(3))).unwrap();
    assert_eq!(applied.actual, Some(3));
    let reset = manager.runtime_threads(request(manager, RuntimeThreadsActionDto::Reset, None)).unwrap();
    println!("REAL_THREAD_RESET {}", serde_json::to_string(&reset).unwrap());
    assert_eq!(reset.actual, initial.actual);
    assert!(!reset.temporary);
    assert_eq!(manager.snapshot().continuous.enabled, Some(false));
    assert!(manager.snapshot().selected_node_job.is_none());
    assert_eq!(owned_engine_identity(&engine), pid);
    assert_eq!(document.serialize().unwrap(), before);
    assert_eq!(std::fs::read(&config).unwrap(), original_bytes);
    let stale = request(manager, RuntimeThreadsActionDto::Apply, Some(2));
    manager.restart().unwrap();
    let restarted = ready(manager);
    assert_ne!(restarted.run_id, run.run_id);
    assert_eq!(restarted.qualified_resource.as_ref().unwrap().resources,
        run.qualified_resource.as_ref().unwrap().resources);
    assert_ne!(owned_engine_identity(&engine), pid);
    assert!(manager.runtime_threads(stale).is_err());
    assert_eq!(manager.runtime_threads_snapshot().actual, None);
    assert!(!manager.runtime_threads_snapshot().temporary);
    assert_eq!(manager.snapshot().continuous.enabled, Some(false));
    assert!(manager.snapshot().selected_node_job.is_none());
    let restarted_value = manager.runtime_threads(request(manager, RuntimeThreadsActionDto::Read, None)).unwrap();
    assert_eq!(restarted_value.actual, initial.actual);
    println!("REAL_THREAD_RESTART {}", serde_json::to_string(&restarted_value).unwrap());
    let mut other = catalog.get("threads").unwrap(); other.profile_id = "other".into(); catalog.upsert(other);
    manager.switch_to("other").unwrap();
    let switched = ready(manager);
    assert_ne!(switched.run_id, restarted.run_id);
    assert_eq!(manager.runtime_threads_snapshot().actual, None);
    manager.teardown().unwrap();
    assert_eq!(manager.runtime_threads_snapshot().run_id, None);
    assert_eq!(manager.runtime_threads_snapshot().actual, None);
    assert_eq!(std::fs::read(&config).unwrap(), original_bytes);
}
