use app_model::{
    EngineAdapterSettings, EngineBackend, EngineFailureDto, EngineFailureKind, EngineOperationDto,
    EngineProfileDto, ForegroundEngineEventDto, ForegroundEngineLifecycleDto, KataGoSettings,
};
use engine_manager::{
    EngineProfileCatalog, ForegroundEngineConfig, ForegroundEngineManager, InMemoryEngineProfileCatalog,
    SavedEngineProfile, SelectedNodeJobRequest,
};
use katago_protocol::AnalysisQuery;
use std::env;
use std::path::Path;
#[cfg(target_os = "linux")]
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
            managed_resources_root: None,
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
        program: engine,
        argv: vec![],
        working_dir: Some(working_dir),
        adapter: EngineAdapterSettings::KataGoAnalysis(KataGoSettings {
            model_path: Some(model),
            config_path: Some(config),
            max_visits: 800,
        }),
    }
}

fn missing_model_profile(name: &str) -> EngineProfileDto {
    let mut profile = real_profile(name);
    if let EngineAdapterSettings::KataGoAnalysis(settings) = &mut profile.adapter {
        settings.model_path = Some(r"D:\dev\weiqi\tmp\r3-ticket-09-missing-model.bin.gz".into());
    }
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
        report_during_search_every: None,
        override_settings: None,
    }
}

fn selected_request(run_id: &str, generation: u64, max_visits: u32) -> SelectedNodeJobRequest {
    SelectedNodeJobRequest {
        run_id: run_id.into(),
        mode: app_model::AnalysisJobModeDto::Finite,
        generation,
        node_path: app_model::NodePath { indices: vec![] },
        query: query(max_visits),
        board_width: 9,
        board_height: 9,
        position_empty: true,
    }
}

#[cfg(target_os = "linux")]
#[derive(Clone, Debug, PartialEq, Eq)]
struct LinuxProcess {
    pid: u32,
    parent: u32,
    start_ticks: u64,
    executable: std::path::PathBuf,
    cwd: std::path::PathBuf,
    argv: Vec<Vec<u8>>,
}

#[cfg(target_os = "linux")]
fn linux_process(pid: u32) -> Option<LinuxProcess> {
    let root = std::path::PathBuf::from(format!("/proc/{pid}"));
    let stat = std::fs::read_to_string(root.join("stat")).ok()?;
    // comm can contain spaces and parentheses; fields after its final ')' start at field3.
    let fields: Vec<_> = stat.rsplit_once(')')?.1.split_whitespace().collect();
    Some(LinuxProcess {
        pid,
        parent: fields.get(1)?.parse().ok()?,
        start_ticks: fields.get(19)?.parse().ok()?,
        executable: std::fs::read_link(root.join("exe")).ok()?,
        cwd: std::fs::read_link(root.join("cwd")).ok()?,
        argv: std::fs::read(root.join("cmdline"))
            .ok()?
            .split(|byte| *byte == 0)
            .filter(|arg| !arg.is_empty())
            .map(<[u8]>::to_vec)
            .collect(),
    })
}

#[cfg(target_os = "linux")]
fn is_owned_katago(
    process: &LinuxProcess,
    parent: u32,
    executable: &Path,
    cwd: &Path,
    config: &Path,
) -> bool {
    use std::os::unix::ffi::OsStrExt;
    process.parent == parent
        && process.executable == executable
        && process.cwd == cwd
        && process.argv.windows(2).any(|args| {
            args[0] == b"-config" && args[1] == config.as_os_str().as_bytes()
        })
}

#[cfg(target_os = "linux")]
fn kill_owned_katago(profile: &EngineProfileDto) {
    use std::os::fd::{AsRawFd, FromRawFd};
    let executable = std::fs::canonicalize(&profile.program).unwrap();
    let cwd = std::fs::canonicalize(profile.working_dir.as_ref().unwrap()).unwrap();
    let EngineAdapterSettings::KataGoAnalysis(settings) = &profile.adapter else {
        panic!("crash smoke requires KataGoAnalysis");
    };
    let config = Path::new(settings.config_path.as_ref().unwrap());
    assert!(config.is_absolute(), "crash smoke requires an absolute private config");
    let parent = std::process::id();
    let targets: Vec<_> = std::fs::read_dir("/proc")
        .unwrap()
        .filter_map(|entry| entry.ok()?.file_name().to_str()?.parse().ok())
        .filter_map(linux_process)
        .filter(|process| is_owned_katago(process, parent, &executable, &cwd, config))
        .collect();
    assert_eq!(targets.len(), 1, "expected exactly one owned KataGo child: {targets:?}");
    let target = &targets[0];
    eprintln!("dry-owned-selection={target:?}");
    // A pidfd pins the selected process so PID reuse cannot redirect SIGKILL.
    let fd = unsafe { libc::syscall(libc::SYS_pidfd_open, target.pid, 0) };
    assert!(fd >= 0, "pidfd_open failed: {}", std::io::Error::last_os_error());
    let handle = unsafe { std::fs::File::from_raw_fd(fd as i32) };
    let current = linux_process(target.pid).expect("owned KataGo exited before recheck");
    assert_eq!(&current, target, "process identity changed before crash injection");
    assert!(is_owned_katago(&current, parent, &executable, &cwd, config));
    let result = unsafe {
        libc::syscall(
            libc::SYS_pidfd_send_signal,
            handle.as_raw_fd(),
            libc::SIGKILL,
            std::ptr::null::<libc::siginfo_t>(),
            0,
        )
    };
    assert_eq!(result, 0, "owned SIGKILL failed: {}", std::io::Error::last_os_error());
    eprintln!("killed-owned-child pid={} parent={} start_ticks={}", target.pid, parent, target.start_ticks);
}

#[cfg(not(target_os = "linux"))]
fn kill_owned_katago(_: &EngineProfileDto) {
    panic!("crash smoke ownership is unsupported on this platform; never use global PID differences");
}

#[cfg(target_os = "linux")]
#[test]
fn owned_crash_selector_rejects_unrelated_lookalikes_without_signalling() {
    use std::os::unix::process::CommandExt;
    let cwd = std::env::current_dir().unwrap();
    let executable = std::fs::canonicalize("/bin/sleep").unwrap();
    let config = cwd.join("private-analysis.cfg");
    let parent = std::process::id();
    let selected = LinuxProcess {
        pid: 1,
        parent,
        start_ticks: 123,
        executable: executable.clone(),
        cwd: cwd.clone(),
        argv: vec![b"katago".to_vec(), b"-config".to_vec(), config.as_os_str().as_encoded_bytes().to_vec()],
    };
    assert!(is_owned_katago(&selected, parent, &executable, &cwd, &config));
    for field in ["parent", "executable", "cwd", "config"] {
        let mut lookalike = selected.clone();
        match field {
            "parent" => lookalike.parent = 0,
            "executable" => lookalike.executable = cwd.join("unrelated-katago"),
            "cwd" => lookalike.cwd = cwd.join("unrelated-run"),
            "config" => lookalike.argv[2] = b"unrelated-analysis.cfg".to_vec(),
            _ => unreachable!(),
        }
        assert!(!is_owned_katago(&lookalike, parent, &executable, &cwd, &config), "accepted mismatched {field}");
    }
    // Same real parent, executable and cwd, misleading argv[0], but no private config.
    // Inspect only: this short-lived owned lookalike exits naturally, never receives a signal.
    let mut lookalike = Command::new(&executable).arg0("katago").arg("1").current_dir(&cwd).spawn().unwrap();
    let identity = linux_process(lookalike.id()).unwrap();
    assert_eq!(identity.parent, parent);
    assert_eq!(identity.executable, executable);
    assert_eq!(identity.cwd, cwd);
    assert!(!is_owned_katago(&identity, parent, &executable, &cwd, &config));
    eprintln!("dry-rejected-live-lookalike={identity:?}; no signal sent");
    assert!(lookalike.wait().unwrap().success());
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
    assert!(cfg!(target_os = "linux"), "crash smoke requires Linux owned-child pidfd support; other platforms remain an explicit unrun gate");

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
    kill_owned_katago(&run_good.profile_snapshot);
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

#[test]
#[ignore = "requires named real KataGo assets; LIZZIEYZY_REAL_KATAGO=1"]
fn real_local_resource_qualification_start_switch_and_corrupt_model_preserve_primary() {
    require_real_katago();
    let catalog = Arc::new(InMemoryEngineProfileCatalog::new());
    for id in ["qualified-a", "qualified-b"] {
        catalog.upsert(SavedEngineProfile {
            profile_id: id.into(),
            profile: real_profile(id),
        });
    }
    let corrupt = std::path::PathBuf::from(env::var("LIZZIEYZY_KATAGO_WORKDIR").unwrap())
        .join(format!("corrupt-model-{}.bin.gz", uuid::Uuid::new_v4()));
    std::fs::write(&corrupt, "not a KataGo model").unwrap();
    let mut invalid = real_profile("invalid-model");
    if let EngineAdapterSettings::KataGoAnalysis(settings) = &mut invalid.adapter {
        settings.model_path = Some(corrupt.to_string_lossy().into_owned());
    }
    catalog.upsert(SavedEngineProfile {
        profile_id: "invalid".into(),
        profile: invalid,
    });
    let manager = ForegroundEngineManager::new(
        catalog,
        ForegroundEngineConfig {
            readiness_timeout: Duration::from_secs(90),
            stop_drain_timeout: Duration::from_secs(2),
            job_timeout: Duration::from_secs(30),
            admit_whole_game_analysis: true,
            managed_resources_root: None,
        },
    );
    let events = manager.subscribe();
    manager.start("invalid").unwrap();
    let invalid_start = wait_failure(&events, Duration::from_secs(90), |_| true);
    eprintln!("resource-invalid-start={invalid_start:?}");
    assert!(matches!(
        manager.snapshot().lifecycle,
        ForegroundEngineLifecycleDto::NoEngine { .. }
    ));
    manager.start("qualified-a").unwrap();
    let ready = wait_snapshot(&events, Duration::from_secs(90), |s| {
        matches!(s, ForegroundEngineLifecycleDto::Ready { .. })
    });
    let run_a = run_from_ready(&ready.lifecycle);
    let qualification = run_a.qualified_resource.as_ref().expect("real resource identity");
    assert_eq!(qualification.origin, "local_unknown");
    assert!(
        qualification
            .version
            .as_deref()
            .is_some_and(|version| !version.is_empty()),
        "real KataGo must report its version"
    );
    assert!(!qualification.static_zlib_exemption);
    assert_eq!(qualification.resources.len(), 3);
    assert!(qualification
        .resources
        .iter()
        .all(|resource| resource.sha256.len() == 64 && resource.bytes > 0));
    eprintln!(
        "resource-qualified-a={}",
        serde_json::to_string(qualification).unwrap()
    );
    manager.switch_to("qualified-b").unwrap();
    let ready = wait_snapshot(
        &events,
        Duration::from_secs(90),
        |s| matches!(s, ForegroundEngineLifecycleDto::Ready { run } if run.profile_id == "qualified-b"),
    );
    let before = run_from_ready(&ready.lifecycle).clone();
    assert_ne!(before.run_id, run_a.run_id);
    manager.switch_to("invalid").unwrap();
    let invalid_switch = wait_failure(&events, Duration::from_secs(90), |f| {
        f.operation == EngineOperationDto::Switch
    });
    eprintln!("resource-invalid-switch={invalid_switch:?}");
    assert_eq!(run_from_ready(&manager.snapshot().lifecycle), &before);
    let job = manager
        .start_selected_node_job(selected_request(&before.run_id, 1, 2))
        .unwrap();
    wait_job(&events, Duration::from_secs(30), |event| {
        event.job_id == job.job_id && event.outcome == app_model::AnalysisJobOutcomeDto::Completed
    });
    manager.teardown().unwrap();
    std::fs::remove_file(corrupt).unwrap();
}

#[test]
#[ignore = "requires real KataGoAnalysis assets; run with LIZZIEYZY_REAL_KATAGO=1 --ignored"]
fn real_katago_bounded_live_diagnostics() {
    require_real_katago();
    let profile = real_profile("Real diagnostics");
    let catalog = Arc::new(InMemoryEngineProfileCatalog::new());
    catalog.upsert(SavedEngineProfile { profile_id: "diagnostics-real".into(), profile: profile.clone() });
    let (manager, events) = manager_with(catalog);
    manager.start("diagnostics-real").unwrap();
    let ready = wait_current(&manager, Duration::from_secs(300), |state| matches!(state, ForegroundEngineLifecycleDto::Ready { .. }));
    let run = run_from_ready(&ready.lifecycle);
    let qualification = run.qualified_resource.as_ref().expect("qualified real Run");
    assert!(qualification.version.as_deref().is_some_and(|value| !value.is_empty()));
    assert!(qualification.backend.as_deref().is_some_and(|value| !value.is_empty()));
    assert_eq!(qualification.origin, "local_unknown");
    assert!(!qualification.static_zlib_exemption);
    assert!(qualification.resources.iter().all(|resource| resource.bytes > 0 && resource.sha256.len() == 64));
    eprintln!("real qualification: {}", serde_json::to_string(qualification).unwrap());
    let job = manager.start_selected_node_job(selected_request(&run.run_id, 1, 2)).unwrap();
    wait_job(&events, Duration::from_secs(120), |event| event.job_id == job.job_id && event.outcome == app_model::AnalysisJobOutcomeDto::Completed);
    let frozen = manager.diagnostic_snapshots().pop().unwrap();
    assert_eq!(frozen.run_id, run.run_id);
    assert!(!frozen.process_exited);
    assert!(!frozen.full_trace);
    assert!(frozen.records.iter().any(|record| record.source == "startup-probe-stderr"));
    assert!(frozen.records.iter().any(|record| record.source == "stdout"));
    assert!(frozen.retained_bytes <= 64 * 1024 && frozen.records.len() <= 256);
    let encoded = serde_json::to_string(&frozen).unwrap();
    assert!(!encoded.contains(&profile.program));
    if let EngineAdapterSettings::KataGoAnalysis(settings) = &profile.adapter {
        assert!(!encoded.contains(settings.model_path.as_ref().unwrap()));
        assert!(!encoded.contains(settings.config_path.as_ref().unwrap()));
    }
    eprintln!("real diagnostic snapshot: {encoded}");
    eprintln!("real analysis: run={} job={} completed=true", run.run_id, job.job_id);
    manager.stop().unwrap();
    wait_current(&manager, Duration::from_secs(20), |state| matches!(state, ForegroundEngineLifecycleDto::NoEngine { .. }));
    assert_eq!(serde_json::to_string(&frozen).unwrap(), encoded);
    eprintln!("real diagnostics: attempt={} records={} bytes={} frozen=true live-stdout=true startup-stderr=true", frozen.run_id, frozen.records.len(), frozen.retained_bytes);
}

#[test]
#[ignore = "requires explicit real KataGo resources and isolated working directory"]
fn real_katago_startup_modes_reopen_durable_catalog() {
    require_real_katago();
    struct DiskCatalog(std::path::PathBuf);
    impl EngineProfileCatalog for DiskCatalog {
        fn get(&self, id: &str) -> Option<SavedEngineProfile> {
            engine_manager::load_engine_profiles(&self.0).ok()?.profiles.into_iter()
                .find(|record| record.id == id)
                .map(|record| SavedEngineProfile { profile_id: record.id, profile: record.profile })
        }
        fn autoload_profile_id(&self) -> Option<String> {
            engine_manager::load_engine_profiles(&self.0).ok()?.startup_profile_id().map(str::to_owned)
        }
    }
    let profile = real_profile("Startup A");
    let directory = std::path::PathBuf::from(profile.working_dir.as_ref().unwrap())
        .join(format!("startup-catalog-{}", std::process::id()));
    std::fs::create_dir(&directory).unwrap();
    let path = directory.join("catalog.json");
    let mut catalog = engine_manager::EngineProfilesSettings {
        version: 2, selected_profile_id: "a".into(),
        startup: app_model::EngineStartupPolicyDto::Off,
        last_primary_profile_id: None,
        profiles: vec![
            engine_manager::EngineProfileRecord { id: "a".into(), profile: profile.clone(), preload: false },
            engine_manager::EngineProfileRecord { id: "b".into(), profile: EngineProfileDto { name: "Startup B".into(), ..profile }, preload: false },
        ],
    };
    let new_manager = || ForegroundEngineManager::new(Arc::new(DiskCatalog(path.clone())), ForegroundEngineConfig {
        readiness_timeout: Duration::from_secs(300), stop_drain_timeout: Duration::from_secs(15),
        job_timeout: Duration::from_secs(120), admit_whole_game_analysis: true,
        managed_resources_root: None,
    });
    engine_manager::save_engine_profiles(&path, catalog.clone()).unwrap();
    let off = new_manager();
    off.apply_autoload().unwrap();
    assert!(matches!(off.snapshot().lifecycle, ForegroundEngineLifecycleDto::NoEngine { failure: None }));
    assert_eq!(off.last_primary_profile_id(), None);
    drop(off);
    catalog.startup = app_model::EngineStartupPolicyDto::Fixed { profile_id: "b".into() };
    engine_manager::save_engine_profiles(&path, catalog.clone()).unwrap();
    let fixed = new_manager();
    let events = fixed.subscribe();
    fixed.apply_autoload().unwrap();
    let first = wait_snapshot(&events, Duration::from_secs(300), |phase| matches!(phase, ForegroundEngineLifecycleDto::Ready { .. }));
    let first_run = run_from_ready(&first.lifecycle);
    assert_eq!(first_run.profile_id, "b");
    fixed.teardown().unwrap();
    catalog = engine_manager::persist_last_primary(&path, catalog, fixed.last_primary_profile_id()).unwrap();
    drop(fixed);
    catalog.startup = app_model::EngineStartupPolicyDto::LastPrimary;
    engine_manager::save_engine_profiles(&path, catalog.clone()).unwrap();
    let last = new_manager();
    let events = last.subscribe();
    last.apply_autoload().unwrap();
    let reopened = wait_snapshot(&events, Duration::from_secs(300), |phase| matches!(phase, ForegroundEngineLifecycleDto::Ready { .. }));
    let reopened_run = run_from_ready(&reopened.lifecycle);
    assert_eq!(reopened_run.profile_id, "b");
    assert_ne!(reopened_run.run_id, first_run.run_id);
    last.switch_to("a").unwrap();
    wait_snapshot(&events, Duration::from_secs(300), |phase| matches!(phase, ForegroundEngineLifecycleDto::Ready { run } if run.profile_id == "a"));
    assert_eq!(last.last_primary_profile_id().as_deref(), Some("a"));
    assert_eq!(engine_manager::load_engine_profiles(&path).unwrap().last_primary_profile_id.as_deref(), Some("b"));
    last.teardown().unwrap();
    drop(last); // No normal-exit persistence: previous durable B must still reopen.
    let previous = new_manager();
    let events = previous.subscribe();
    previous.apply_autoload().unwrap();
    let persisted = wait_snapshot(&events, Duration::from_secs(300), |phase| matches!(phase, ForegroundEngineLifecycleDto::Ready { .. }));
    assert_eq!(run_from_ready(&persisted.lifecycle).profile_id, "b");
    previous.teardown().unwrap();
    drop(previous);
    catalog.last_primary_profile_id = Some("deleted".into());
    engine_manager::save_engine_profiles(&path, catalog).unwrap();
    let missing = new_manager();
    assert!(missing.apply_autoload().is_err());
    assert!(matches!(missing.snapshot().lifecycle, ForegroundEngineLifecycleDto::NoEngine { .. }));
    assert_eq!(missing.last_primary_profile_id(), None);
    eprintln!("startup smoke: off, fixed B, last-primary B/new Run, unsaved A retains durable B, deleted identity has no fallback");
    std::fs::remove_dir_all(directory).unwrap();
}

#[cfg(target_os = "linux")]
#[test]
#[ignore = "requires explicit real KataGo CPU assets and isolated cwd"]
fn real_preload_promotion_cancellation_and_memory_pressure_preserve_primary() {
    use app_model::EnginePreloadPhaseDto as Phase;
    require_real_katago();
    let catalog = Arc::new(InMemoryEngineProfileCatalog::new());
    for id in ["preload-a", "preload-b", "preload-pressure"] {
        catalog.upsert(SavedEngineProfile {
            profile_id: id.into(),
            profile: real_profile(id),
        });
        catalog.set_preload(id, id == "preload-b");
    }
    let directory = std::path::PathBuf::from(env::var("LIZZIEYZY_KATAGO_WORKDIR").unwrap());
    let wrapper = directory.join("limited-engine.sh");
    std::fs::write(
        &wrapper,
        "#!/bin/sh\nexec /usr/bin/prlimit --as=268435456 -- \"$LIZZIEYZY_KATAGO_ENGINE\" \"$@\"\n",
    )
    .unwrap();
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(&wrapper, std::fs::Permissions::from_mode(0o700)).unwrap();
    let mut pressure = catalog.get("preload-pressure").unwrap();
    pressure.profile.program = wrapper.to_string_lossy().into_owned();
    catalog.upsert(pressure);
    catalog.set_autoload_profile_id(Some("preload-a".into()));
    let manager = ForegroundEngineManager::new(
        catalog.clone(),
        ForegroundEngineConfig {
            readiness_timeout: Duration::from_secs(90),
            stop_drain_timeout: Duration::from_millis(400),
            job_timeout: Duration::from_secs(30),
            admit_whole_game_analysis: true,
            managed_resources_root: None,
        },
    );
    let events = manager.subscribe();
    let wait = |phase| {
        let deadline = Instant::now() + Duration::from_secs(100);
        loop {
            let slots = manager.preload_snapshot();
            if let Some(slot) = slots.iter().find(|slot| slot.phase == phase) {
                return slot.clone();
            }
            assert!(Instant::now() < deadline, "real preload timeout: {slots:?}");
            std::thread::sleep(Duration::from_millis(25));
        }
    };
    assert_eq!(manager.last_primary_profile_id(), None);
    manager.apply_autoload().unwrap();
    manager.schedule_startup_preloads();
    let ready = wait_current(&manager, Duration::from_secs(100), |state| {
        matches!(state, ForegroundEngineLifecycleDto::Ready { .. })
    });
    let a = run_from_ready(&ready.lifecycle).clone();
    assert_eq!(manager.last_primary_profile_id().as_deref(), Some("preload-a"));
    let b = wait(Phase::Ready);
    eprintln!(
        "real-preload-a={} background-b={}",
        serde_json::to_string(&a).unwrap(),
        serde_json::to_string(&b).unwrap()
    );
    assert_eq!(run_from_ready(&manager.snapshot().lifecycle).run_id, a.run_id);
    assert_eq!(manager.last_primary_profile_id().as_deref(), Some("preload-a"));
    manager.switch_to("preload-b").unwrap();
    assert_eq!(run_from_ready(&manager.snapshot().lifecycle).run_id, b.run.run_id);
    assert_eq!(manager.last_primary_profile_id().as_deref(), Some("preload-b"));
    catalog.set_preload("preload-a", true);
    manager.prepare_preload("preload-a").unwrap();
    wait(Phase::Ready);
    manager.cancel_preload("preload-a").unwrap();
    assert_eq!(wait(Phase::Cancelled).run.profile_id, "preload-a");
    catalog.set_preload("preload-pressure", true);
    manager.prepare_preload("preload-pressure").unwrap();
    let failed = wait(Phase::Failed);
    eprintln!(
        "real-preload-pressure={}",
        serde_json::to_string(&failed).unwrap()
    );
    assert!(
        failed
            .failure
            .as_ref()
            .unwrap()
            .diagnostic_summary
            .as_ref()
            .is_some_and(|text| text.contains("alloc")
                || text.contains("memory")
                || text.contains("resource")),
        "{failed:?}"
    );
    assert_eq!(run_from_ready(&manager.snapshot().lifecycle).run_id, b.run.run_id);
    assert_eq!(manager.last_primary_profile_id().as_deref(), Some("preload-b"));
    manager.prepare_preload("preload-a").unwrap();
    wait(Phase::Ready);
    let evaluation = manager.start_evaluation("preload-a").unwrap();
    assert!(evaluation.evaluation_id.is_some());
    assert_eq!(wait(Phase::Ready).run.profile_id, "preload-a");
    let job = manager
        .start_selected_node_job(selected_request(&b.run.run_id, 1, 2))
        .unwrap();
    assert_eq!(manager.evaluation_snapshot().phase, app_model::EvaluationPhaseDto::Yielded);
    assert_eq!(manager.evaluation_snapshot().process_id, None);
    assert_eq!(wait(Phase::Cancelled).run.profile_id, "preload-a");
    wait_job(&events, Duration::from_secs(30), |event| {
        event.job_id == job.job_id && event.outcome == app_model::AnalysisJobOutcomeDto::Completed
    });
    manager.teardown().unwrap();
    assert!(matches!(
        manager.snapshot().lifecycle,
        ForegroundEngineLifecycleDto::NoEngine { .. }
    ));
    assert_eq!(manager.last_primary_profile_id().as_deref(), Some("preload-b"));
    std::fs::remove_file(wrapper).unwrap();
}
