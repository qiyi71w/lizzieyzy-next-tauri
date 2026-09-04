use app_model::{
    EngineBackend, EngineFailureDto, EngineFailureKind, EngineOperationDto, EngineProfileDto,
    ForegroundEngineEventDto, ForegroundEngineLifecycleDto,
};
use engine_manager::{
    EngineProfileCatalog, ForegroundEngineConfig, ForegroundEngineManager, InMemoryEngineProfileCatalog,
    SavedEngineProfile, SelectedNodeJobRequest,
};
use katago_protocol::AnalysisQuery;
use std::env;
use std::path::Path;
use std::process::Command;
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::time::{Duration, Instant};

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
            .unwrap_or_else(|_| panic!("timed out waiting for real KataGo snapshot after {timeout:?}"));
        if let ForegroundEngineEventDto::Snapshot { snapshot } = event {
            eprintln!(
                "snapshot revision={} state={:?}",
                snapshot.revision, snapshot.lifecycle
            );
            if predicate(&snapshot.lifecycle) {
                return snapshot;
            }
        }
    }
}

fn wait_job(
    events: &Receiver<ForegroundEngineEventDto>,
    timeout: Duration,
    mut predicate: impl FnMut(&app_model::AnalysisJobEventDto) -> bool,
) -> app_model::AnalysisJobEventDto {
    let deadline = Instant::now() + timeout;
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        let event = events
            .recv_timeout(remaining)
            .unwrap_or_else(|_| panic!("timed out waiting for real KataGo job after {timeout:?}"));
        if let ForegroundEngineEventDto::Job { job } = event {
            eprintln!(
                "job {} {} {:?} gen={} path={:?}",
                job.job_id, job.run_id, job.outcome, job.generation, job.node_path.indices
            );
            if predicate(&job) {
                return job;
            }
        }
    }
}

fn wait_failure(
    events: &Receiver<ForegroundEngineEventDto>,
    timeout: Duration,
    mut predicate: impl FnMut(&EngineFailureDto) -> bool,
) -> EngineFailureDto {
    let deadline = Instant::now() + timeout;
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        let event = events
            .recv_timeout(remaining)
            .unwrap_or_else(|_| panic!("timed out waiting for real KataGo failure after {timeout:?}"));
        if let ForegroundEngineEventDto::Failure { failure } = event {
            eprintln!(
                "failure op={:?} kind={:?} run={:?} profile={:?} switch={:?} msg={}",
                failure.operation,
                failure.kind,
                failure.run_id,
                failure.profile_id,
                failure.switch_id,
                failure.message
            );
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

fn wait_current(
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
                "timed out waiting for current snapshot after {timeout:?}: {:?}",
                snapshot.lifecycle
            );
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

fn require_real_katago() {
    assert_eq!(
        env::var("LIZZIEYZY_REAL_KATAGO").ok().as_deref(),
        Some("1"),
        "refusing to treat fixture absence as a real-engine pass"
    );
}

fn manager_with(
    catalog: Arc<InMemoryEngineProfileCatalog>,
) -> (ForegroundEngineManager, Receiver<ForegroundEngineEventDto>) {
    let manager = ForegroundEngineManager::new(
        catalog,
        ForegroundEngineConfig {
            readiness_timeout: Duration::from_secs(300),
            stop_drain_timeout: Duration::from_secs(15),
            job_timeout: Duration::from_secs(120),
            admit_whole_game_analysis: true,
        },
    );
    let events = manager.subscribe();
    (manager, events)
}

fn env_path(name: &str, default: &str) -> String {
    env::var(name).unwrap_or_else(|_| default.to_string())
}

fn real_profile(name: &str) -> EngineProfileDto {
    let engine = env_path(
        "LIZZIEYZY_KATAGO_ENGINE",
        r"D:\dev\weiqi\lizzieyzy-next\engines\katago\windows-x64\katago.exe",
    );
    let model = env_path(
        "LIZZIEYZY_KATAGO_MODEL",
        r"D:\dev\weiqi\lizzieyzy-next\weights\default.bin.gz",
    );
    let config = env_path(
        "LIZZIEYZY_KATAGO_CONFIG",
        r"D:\dev\weiqi\lizzieyzy-next\engines\katago\configs\analysis.cfg",
    );
    let working_dir = env_path(
        "LIZZIEYZY_KATAGO_WORKDIR",
        r"D:\dev\weiqi\lizzieyzy-next\engines\katago\windows-x64",
    );
    for (label, path) in [
        ("engine", engine.as_str()),
        ("model", model.as_str()),
        ("config", config.as_str()),
        ("working_dir", working_dir.as_str()),
    ] {
        assert!(Path::new(path).exists(), "real KataGo {label} is missing: {path}");
    }
    EngineProfileDto {
        name: name.into(),
        engine_path: engine,
        model_path: Some(model),
        config_path: Some(config),
        working_dir: Some(working_dir),
        backend: EngineBackend::KataGoAnalysis,
    }
}

fn missing_model_profile(name: &str) -> EngineProfileDto {
    let mut profile = real_profile(name);
    profile.model_path = Some(r"D:\dev\weiqi\tmp\r3-ticket-09-missing-model.bin.gz".into());
    profile
}

fn query(max_visits: u32) -> AnalysisQuery {
    AnalysisQuery {
        id: "placeholder".into(),
        moves: Vec::new(),
        initial_stones: Vec::new(),
        rules: "chinese".into(),
        komi: 7.5,
        board_x_size: 9,
        board_y_size: 9,
        analyze_turns: Some(vec![0]),
        max_visits: Some(max_visits),
        include_ownership: Some(true),
        include_policy: Some(true),
    }
}

fn selected_request(run_id: &str, generation: u64, max_visits: u32) -> SelectedNodeJobRequest {
    SelectedNodeJobRequest {
        run_id: run_id.into(),
        generation,
        node_path: app_model::NodePath { indices: vec![] },
        query: query(max_visits),
        board_size: 9,
    }
}

fn katago_pids() -> Vec<u32> {
    let output = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-Command",
            "Get-Process -Name katago -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Id",
        ])
        .output()
        .expect("list katago processes");
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| line.trim().parse().ok())
        .collect()
}

fn kill_new_katago(before: &[u32]) {
    let after = katago_pids();
    let targets: Vec<u32> = after.into_iter().filter(|pid| !before.contains(pid)).collect();
    assert!(
        !targets.is_empty(),
        "expected a live KataGo process to kill after Ready"
    );
    for pid in targets {
        eprintln!("killing KataGo pid={pid}");
        let status = Command::new("powershell.exe")
            .args([
                "-NoProfile",
                "-Command",
                &format!("Stop-Process -Id {pid} -Force"),
            ])
            .status()
            .expect("kill KataGo");
        assert!(status.success(), "Stop-Process {pid} failed: {status}");
    }
}

#[test]
#[ignore = "requires real KataGoAnalysis assets; run with LIZZIEYZY_REAL_KATAGO=1 --ignored"]
fn real_katago_ready_job_stop_restart_and_switch() {
    require_real_katago();

    let catalog = Arc::new(InMemoryEngineProfileCatalog::new());
    catalog.upsert(SavedEngineProfile {
        profile_id: "profile-a".into(),
        profile: real_profile("Real KataGo A"),
    });
    catalog.upsert(SavedEngineProfile {
        profile_id: "profile-b".into(),
        profile: real_profile("Real KataGo B"),
    });
    let (manager, events) = manager_with(catalog.clone());
    assert!(matches!(
        manager.snapshot().lifecycle,
        ForegroundEngineLifecycleDto::NoEngine { .. }
    ));

    manager.start("profile-a").unwrap();
    wait_snapshot(&events, Duration::from_secs(5), |lifecycle| {
        matches!(lifecycle, ForegroundEngineLifecycleDto::Starting { .. })
    });
    let ready = wait_snapshot(&events, Duration::from_secs(300), |lifecycle| {
        matches!(lifecycle, ForegroundEngineLifecycleDto::Ready { .. })
    });
    let run_a = run_from_ready(&ready.lifecycle).clone();
    assert_eq!(run_a.profile_id, "profile-a");
    assert_eq!(run_a.adapter_kind, EngineBackend::KataGoAnalysis);
    assert_eq!(run_a.profile_snapshot.name, "Real KataGo A");
    assert!(run_a.capability_snapshot.is_some());
    eprintln!("Ready A run_id={}", run_a.run_id);

    let completed = manager
        .start_selected_node_job(selected_request(&run_a.run_id, 1, 2))
        .unwrap();
    let done = wait_job(&events, Duration::from_secs(120), |job| {
        job.job_id == completed.job_id
            && job.outcome == app_model::AnalysisJobOutcomeDto::Completed
            && job.frame.is_some()
    });
    assert_eq!(done.run_id, run_a.run_id);
    assert_eq!(done.generation, 1);
    assert!(matches!(
        manager.snapshot().lifecycle,
        ForegroundEngineLifecycleDto::Ready { .. }
    ));

    let cancellable = manager
        .start_selected_node_job(selected_request(&run_a.run_id, 2, 400))
        .unwrap();
    wait_job(&events, Duration::from_secs(30), |job| {
        job.job_id == cancellable.job_id && job.outcome == app_model::AnalysisJobOutcomeDto::Started
    });
    manager.cancel_job(&run_a.run_id, &cancellable.job_id).unwrap();
    wait_job(&events, Duration::from_secs(30), |job| {
        job.job_id == cancellable.job_id
            && matches!(
                job.outcome,
                app_model::AnalysisJobOutcomeDto::Cancelled | app_model::AnalysisJobOutcomeDto::Timeout
            )
    });
    assert!(matches!(
        manager.snapshot().lifecycle,
        ForegroundEngineLifecycleDto::Ready { .. }
    ));

    manager.stop().unwrap();
    wait_snapshot(&events, Duration::from_secs(5), |lifecycle| {
        matches!(lifecycle, ForegroundEngineLifecycleDto::Stopping { .. })
    });
    wait_snapshot(&events, Duration::from_secs(30), |lifecycle| {
        matches!(lifecycle, ForegroundEngineLifecycleDto::NoEngine { .. })
    });

    manager.start("profile-a").unwrap();
    let ready_again = wait_snapshot(&events, Duration::from_secs(300), |lifecycle| {
        matches!(lifecycle, ForegroundEngineLifecycleDto::Ready { .. })
    });
    let run_restart_base = run_from_ready(&ready_again.lifecycle).clone();
    assert_ne!(run_restart_base.run_id, run_a.run_id);

    let mut repaired = catalog.get("profile-a").unwrap();
    repaired.profile.name = "Repaired KataGo A".into();
    catalog.upsert(repaired);
    assert_eq!(
        run_from_ready(&manager.snapshot().lifecycle)
            .profile_snapshot
            .name,
        "Real KataGo A"
    );
    manager.restart().unwrap();
    wait_snapshot(&events, Duration::from_secs(10), |lifecycle| {
        matches!(lifecycle, ForegroundEngineLifecycleDto::Stopping { .. })
    });
    let restarted = wait_snapshot(&events, Duration::from_secs(300), |lifecycle| {
        matches!(lifecycle, ForegroundEngineLifecycleDto::Ready { .. })
    });
    let run_restarted = run_from_ready(&restarted.lifecycle).clone();
    assert_ne!(run_restarted.run_id, run_restart_base.run_id);
    assert_eq!(run_restarted.profile_id, "profile-a");
    assert_eq!(run_restarted.profile_snapshot.name, "Repaired KataGo A");

    manager.switch_to("profile-b").unwrap();
    wait_snapshot(&events, Duration::from_secs(10), |lifecycle| {
        matches!(lifecycle, ForegroundEngineLifecycleDto::Switching { .. })
    });
    let promoted = wait_snapshot(
        &events,
        Duration::from_secs(300),
        |lifecycle| matches!(lifecycle, ForegroundEngineLifecycleDto::Ready { run } if run.profile_id == "profile-b"),
    );
    let run_b = run_from_ready(&promoted.lifecycle).clone();
    assert_ne!(run_b.run_id, run_restarted.run_id);
    assert_eq!(run_b.profile_snapshot.name, "Real KataGo B");
    manager
        .start_selected_node_job(selected_request(&run_restarted.run_id, 3, 2))
        .unwrap_err();
    let b_job = manager
        .start_selected_node_job(selected_request(&run_b.run_id, 3, 2))
        .unwrap();
    wait_job(&events, Duration::from_secs(120), |job| {
        job.job_id == b_job.job_id && job.outcome == app_model::AnalysisJobOutcomeDto::Completed
    });

    manager.stop().unwrap();
    wait_current(&manager, Duration::from_secs(30), |lifecycle| {
        matches!(lifecycle, ForegroundEngineLifecycleDto::NoEngine { .. })
    });
}

#[test]
#[ignore = "requires real KataGoAnalysis assets; run with LIZZIEYZY_REAL_KATAGO=1 --ignored"]
fn real_katago_failed_start_failed_switch_crash_and_autoload() {
    require_real_katago();
    let before_pids = katago_pids();

    let catalog = Arc::new(InMemoryEngineProfileCatalog::new());
    catalog.upsert(SavedEngineProfile {
        profile_id: "profile-good".into(),
        profile: real_profile("Real KataGo Good"),
    });
    catalog.upsert(SavedEngineProfile {
        profile_id: "profile-bad".into(),
        profile: missing_model_profile("Missing Model"),
    });

    catalog.set_autoload_profile_id(Some("profile-bad".into()));
    let (manager, events) = manager_with(catalog.clone());
    manager.apply_autoload().unwrap();
    let autoload_failure = wait_failure(&events, Duration::from_secs(10), |failure| {
        failure.operation == EngineOperationDto::Autoload
            && failure.kind == EngineFailureKind::Asset
            && failure.profile_id.as_deref() == Some("profile-bad")
    });
    eprintln!("autoload failure {}", autoload_failure.message);
    assert!(matches!(
        manager.snapshot().lifecycle,
        ForegroundEngineLifecycleDto::NoEngine { .. }
    ));

    manager.start("profile-bad").unwrap();
    wait_snapshot(&events, Duration::from_secs(5), |lifecycle| {
        matches!(lifecycle, ForegroundEngineLifecycleDto::Starting { .. })
    });
    let start_failure = wait_failure(&events, Duration::from_secs(10), |failure| {
        failure.operation == EngineOperationDto::Start && failure.kind == EngineFailureKind::Asset
    });
    eprintln!("start failure {}", start_failure.message);
    assert!(matches!(
        manager.snapshot().lifecycle,
        ForegroundEngineLifecycleDto::NoEngine { .. }
    ));

    catalog.set_autoload_profile_id(Some("profile-good".into()));
    manager.apply_autoload().unwrap();
    wait_snapshot(&events, Duration::from_secs(5), |lifecycle| {
        matches!(lifecycle, ForegroundEngineLifecycleDto::Starting { .. })
    });
    let autoloaded = wait_snapshot(&events, Duration::from_secs(300), |lifecycle| {
        matches!(lifecycle, ForegroundEngineLifecycleDto::Ready { .. })
    });
    let run_good = run_from_ready(&autoloaded.lifecycle).clone();
    assert_eq!(run_good.profile_id, "profile-good");
    eprintln!("autoload Ready run_id={}", run_good.run_id);

    manager.switch_to("profile-bad").unwrap();
    wait_snapshot(&events, Duration::from_secs(10), |lifecycle| {
        matches!(lifecycle, ForegroundEngineLifecycleDto::Switching { .. })
    });
    let switch_failure = wait_failure(&events, Duration::from_secs(15), |failure| {
        failure.operation == EngineOperationDto::Switch
            && failure.kind == EngineFailureKind::Asset
            && failure.profile_id.as_deref() == Some("profile-bad")
    });
    eprintln!("switch failure {}", switch_failure.message);
    let after_failed_switch = manager.snapshot();
    assert!(
        matches!(after_failed_switch.lifecycle, ForegroundEngineLifecycleDto::Ready { ref run } if run.run_id == run_good.run_id)
    );
    assert_eq!(
        run_from_ready(&after_failed_switch.lifecycle).profile_id,
        "profile-good"
    );

    let job = manager
        .start_selected_node_job(selected_request(&run_good.run_id, 4, 400))
        .unwrap();
    wait_job(&events, Duration::from_secs(30), |event| {
        event.job_id == job.job_id && event.outcome == app_model::AnalysisJobOutcomeDto::Started
    });
    kill_new_katago(&before_pids);
    let crashed = wait_current(&manager, Duration::from_secs(30), |lifecycle| {
        matches!(lifecycle, ForegroundEngineLifecycleDto::Error { .. })
    });
    match &crashed.lifecycle {
        ForegroundEngineLifecycleDto::Error { run, failure } => {
            assert_eq!(run.run_id, run_good.run_id);
            assert_eq!(failure.operation, EngineOperationDto::UnexpectedExit);
            eprintln!("crash failure kind={:?} msg={}", failure.kind, failure.message);
        }
        other => panic!("expected Error, got {other:?}"),
    }

    manager.stop().unwrap();
    wait_snapshot(&events, Duration::from_secs(30), |lifecycle| {
        matches!(lifecycle, ForegroundEngineLifecycleDto::NoEngine { .. })
    });
}
