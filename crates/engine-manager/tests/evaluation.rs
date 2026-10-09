#![cfg(unix)]
use app_model::{
    EngineAdapterSettings, EngineProfileDto, EvaluationPhaseDto as Phase, EvaluationSnapshotDto,
    ForegroundEngineLifecycleDto, KataGoSettings,
};
use engine_manager::{
    AnalysisCancelToken, AnalysisJobLane, EngineProfileCatalog, ForegroundEngineConfig,
    ForegroundEngineManager, SavedEngineProfile,
};
use parking_lot::Mutex;
use std::{
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

#[derive(Default)]
struct Catalog(Mutex<Option<SavedEngineProfile>>);
impl EngineProfileCatalog for Catalog {
    fn get(&self, id: &str) -> Option<SavedEngineProfile> {
        self.0
            .lock()
            .as_ref()
            .filter(|saved| saved.profile_id == id)
            .cloned()
    }
}
struct Fixture {
    dir: PathBuf,
    catalog: Arc<Catalog>,
    manager: ForegroundEngineManager,
    wait_timeout: Duration,
}
impl Fixture {
    fn new(mode: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("evaluation-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("model.bin"), "model").unwrap();
        std::fs::write(
            dir.join("config.cfg"),
            "@include included.cfg\nnumSearchThreads=1",
        )
        .unwrap();
        std::fs::write(dir.join("included.cfg"), "nnMaxBatchSize=1").unwrap();
        std::fs::write(dir.join("extra.cfg"), "logToStderr=true").unwrap();
        let profile = EngineProfileDto {
            name: "saved target".into(),
            program: format!(
                "{}/tests/fixtures/evaluation_process.py",
                env!("CARGO_MANIFEST_DIR")
            ),
            argv: vec![
                "gtp".into(),
                "-config".into(),
                "config.cfg".into(),
                "-model".into(),
                "model.bin".into(),
                "-config".into(),
                "extra.cfg".into(),
                "-override-config".into(),
                format!("testMode={mode}"),
            ],
            working_dir: Some(dir.to_string_lossy().into()),
            adapter: EngineAdapterSettings::GenericGtp(app_model::GenericGtpSettings::default()),
        };
        let catalog = Arc::new(Catalog(Mutex::new(Some(SavedEngineProfile {
            profile_id: "target".into(),
            profile,
        }))));
        let manager = ForegroundEngineManager::new(catalog.clone(), ForegroundEngineConfig::for_tests());
        Self {
            dir,
            catalog,
            manager,
            wait_timeout: Duration::from_secs(10),
        }
    }
    fn wait(&self, predicate: impl Fn(&EvaluationSnapshotDto) -> bool) -> EvaluationSnapshotDto {
        let deadline = Instant::now() + self.wait_timeout;
        loop {
            let snapshot = self.manager.evaluation_snapshot();
            if predicate(&snapshot) {
                return snapshot;
            }
            assert!(Instant::now() < deadline, "evaluation timeout: {snapshot:?}");
            std::thread::sleep(Duration::from_millis(10));
        }
    }
    fn running(&self) -> EvaluationSnapshotDto {
        self.manager.start_evaluation("target").unwrap();
        let running = self.wait(|s| {
            s.phase == Phase::Running && s.output.iter().any(|line| line.contains("benchmark started"))
        });
        assert_eq!(running.exit_code, None, "probe exit is not measurement exit");
        running
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        self.manager.teardown().unwrap();
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

#[test]
fn saved_target_preserves_argv_all_configs_workdir_and_returns_real_result() {
    let f = Fixture::new("success");
    let before = f.catalog.get("target").unwrap();
    let admitted = f.manager.start_evaluation("target").unwrap();
    let done = f.wait(|s| s.phase == Phase::Completed);
    assert_eq!(done.exit_code, Some(0));
    assert_eq!(done.process_id, None);
    let result = done.result.unwrap();
    assert_eq!(result.target_id, "target");
    assert_eq!(Some(result.input_revision.clone()), admitted.input_revision);
    assert_eq!(result.search_visits_per_second, Some(123.5));
    assert!(
        result
            .qualified_resource
            .resources
            .iter()
            .filter(|r| r.component == "config")
            .count()
            >= 3
    );
    let observed: serde_json::Value =
        serde_json::from_slice(&std::fs::read(f.dir.join("observed.json")).unwrap()).unwrap();
    assert_eq!(observed["argv"][0], "benchmark");
    assert_eq!(
        observed["argv"].as_array().unwrap()[1..],
        before.profile.argv[1..]
    );
    assert_eq!(observed["cwd"], f.dir.to_string_lossy().as_ref());
    assert_eq!(f.catalog.get("target"), Some(before));
    assert_eq!(f.manager.last_primary_profile_id(), None);
    assert!(f.manager.diagnostic_snapshots().is_empty());
    assert!(matches!(
        f.manager.snapshot().lifecycle,
        ForegroundEngineLifecycleDto::NoEngine { failure: None }
    ));
}

#[test]
fn canonical_display_sanitization_preserves_raw_measurement_and_isolated_identity() {
    let f = Fixture::new("privacy");
    f.manager.start_evaluation("target").unwrap();
    let done = f.wait(|s| s.phase == Phase::Completed);
    let display = done.output.join("\n");
    for secret in ["private-first-line", "private-second-line", "private-owner", "\u{1b}"] {
        assert!(!display.contains(secret), "{display}");
    }
    assert_eq!(done.result.unwrap().search_visits_per_second, Some(123.5));
    assert_eq!(f.manager.last_primary_profile_id(), None);
    assert!(f.manager.diagnostic_snapshots().is_empty());
}

#[test]
fn failure_missing_metrics_and_bounded_output_are_truthful() {
    let f = Fixture::new("failure");
    f.manager.start_evaluation("target").unwrap();
    let failed = f.wait(|s| s.phase == Phase::Failed);
    assert_eq!(failed.exit_code, Some(7));
    assert!(failed.result.is_none());
    assert!(failed.output.is_empty());
    assert!(!failed.message.as_deref().unwrap().contains("do-not-show"));
    assert!(!failed.message.as_deref().unwrap().contains("/private/user/path"));
    let f = Fixture::new("missing");
    f.manager.start_evaluation("target").unwrap();
    assert_eq!(
        f.wait(|s| s.phase == Phase::Completed)
            .result
            .unwrap()
            .search_visits_per_second,
        None
    );
    let f = Fixture::new("burst");
    f.manager.start_evaluation("target").unwrap();
    let done = f.wait(|s| s.phase == Phase::Completed);
    assert!(done.output_truncated);
    assert!(done.output.iter().map(String::len).sum::<usize>() <= 8192);
}

#[test]
fn cancel_new_target_edit_delete_and_close_retire_process_and_late_output() {
    let f = Fixture::new("hold");
    let first = f.running();
    let cancelled = f
        .manager
        .cancel_evaluation(first.evaluation_id.as_ref().unwrap())
        .unwrap();
    assert_eq!(cancelled.phase, Phase::Cancelled);
    assert_eq!(cancelled.process_id, None);
    assert!(cancelled.output.is_empty());
    let second = f.running();
    f.manager.start_evaluation("target").unwrap();
    let third = f.wait(|s| s.phase == Phase::Running && s.evaluation_id != second.evaluation_id);
    assert!(f
        .manager
        .cancel_evaluation(second.evaluation_id.as_ref().unwrap())
        .is_err());
    f.catalog.0.lock().as_mut().unwrap().profile.name = "edited".into();
    let retired = f.manager.evaluation_snapshot();
    assert_eq!(retired.phase, Phase::Retired);
    assert!(retired.result.is_none() && retired.output.is_empty() && retired.process_id.is_none());
    assert_eq!(retired.evaluation_id, third.evaluation_id);
    f.running();
    *f.catalog.0.lock() = None;
    assert_eq!(f.manager.evaluation_snapshot().phase, Phase::Retired);
    assert!(f.manager.start_evaluation("target").is_err());
    let f = Fixture::new("hold");
    f.running();
    f.manager.teardown().unwrap();
    std::fs::write(f.dir.join("release"), "late").unwrap();
    assert!(f.manager.evaluation_snapshot().result.is_none());
    assert_eq!(f.manager.evaluation_snapshot().process_id, None);
}

#[test]
fn match_and_analysis_admission_reap_evaluation_before_foreground_work() {
    let f = Fixture::new("hold");
    f.running();
    f.manager.reserve_match("match").unwrap();
    let yielded = f.manager.evaluation_snapshot();
    assert_eq!(yielded.phase, Phase::Yielded);
    assert_eq!(yielded.process_id, None);
    assert!(f.manager.start_evaluation("target").is_err());
    f.manager.stop_reserved_match("match").unwrap();
    {
        let mut catalog = f.catalog.0.lock();
        let profile = &mut catalog.as_mut().unwrap().profile;
        profile.argv = vec!["-override-config".into(), "testMode=hold".into()];
        profile.adapter = EngineAdapterSettings::KataGoAnalysis(KataGoSettings {
            model_path: Some("model.bin".into()),
            config_path: Some("config.cfg".into()),
            max_visits: 10,
        });
    }
    f.manager.start("target").unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    let run = loop {
        if let ForegroundEngineLifecycleDto::Ready { run } = f.manager.snapshot().lifecycle {
            break run;
        }
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(10));
    };
    f.manager
        .set_continuous_preferences(false, app_model::ContinuousAnalysisBudgetDto::default())
        .unwrap();
    let foreground = f.manager.snapshot();
    let primary = f.manager.last_primary_profile_id();
    let attempts: Vec<_> = f.manager.diagnostic_snapshots().into_iter().map(|s| s.run_id).collect();
    f.running();
    assert_eq!(f.manager.last_primary_profile_id(), primary);
    assert_eq!(f.manager.diagnostic_snapshots().into_iter().map(|s| s.run_id).collect::<Vec<_>>(), attempts);
    f.manager
        .register_job(
            &run.run_id,
            AnalysisJobLane::SelectedNode,
            Arc::new(AnalysisCancelToken::new()),
        )
        .unwrap();
    assert_eq!(f.manager.evaluation_snapshot().phase, Phase::Yielded);
    assert_eq!(f.manager.evaluation_snapshot().process_id, None);
    assert_eq!(f.manager.snapshot().lifecycle, foreground.lifecycle);
    assert_eq!(f.manager.snapshot().continuous.enabled, Some(false));
    assert!(f.manager.start_evaluation("target").is_err());
}

#[test]
#[ignore = "requires frozen compatible KataGo and model; production measurement, not a probe"]
fn real_katago_measurement_failure_cancel_and_foreground_handoff() {
    let engine = std::env::var("LIZZIEYZY_KATAGO_ENGINE").unwrap();
    let model = std::env::var("LIZZIEYZY_KATAGO_MODEL").unwrap();
    let config = std::env::var("LIZZIEYZY_KATAGO_CONFIG").unwrap();
    let workdir = std::env::var("LIZZIEYZY_KATAGO_WORKDIR").unwrap();
    let config_before = std::fs::read(&config).unwrap();
    let mut f = Fixture::new("success");
    f.wait_timeout = Duration::from_secs(70);
    f.manager = ForegroundEngineManager::new(
        f.catalog.clone(),
        ForegroundEngineConfig {
            readiness_timeout: Duration::from_secs(60),
            ..ForegroundEngineConfig::default()
        },
    );
    let original = SavedEngineProfile {
        profile_id: "target".into(),
        profile: EngineProfileDto {
            name: "Real measured target".into(),
            program: engine,
            argv: vec![
                "gtp".into(),
                "-config".into(),
                config.clone(),
                "-model".into(),
                model.clone(),
                "-t".into(),
                "1".into(),
                "-v".into(),
                "2".into(),
                "-n".into(),
                "1".into(),
            ],
            working_dir: Some(workdir),
            adapter: EngineAdapterSettings::GenericGtp(app_model::GenericGtpSettings::default()),
        },
    };
    *f.catalog.0.lock() = Some(original.clone());
    f.manager.start_evaluation("target").unwrap();
    let deadline = Instant::now() + Duration::from_secs(120);
    let success = loop {
        let s = f.manager.evaluation_snapshot();
        if s.phase == Phase::Completed {
            break s;
        }
        assert_ne!(s.phase, Phase::Failed, "{s:?}");
        assert!(Instant::now() < deadline, "{s:?}");
        std::thread::sleep(Duration::from_millis(50));
    };
    assert!(success.result.as_ref().unwrap().search_visits_per_second.unwrap() > 0.0);
    assert_eq!(success.exit_code, Some(0));
    println!("REAL_SUCCESS {}", serde_json::to_string(&success).unwrap());
    f.catalog
        .0
        .lock()
        .as_mut()
        .unwrap()
        .profile
        .argv
        .push("-invalid-benchmark-option".into());
    f.manager.start_evaluation("target").unwrap();
    let failure = f.wait(|s| s.phase == Phase::Failed);
    assert_ne!(failure.exit_code, Some(0));
    assert!(failure.result.is_none());
    println!("REAL_FAILURE {}", serde_json::to_string(&failure).unwrap());
    *f.catalog.0.lock() = Some(original.clone());
    f.manager.start_evaluation("target").unwrap();
    let running = f.wait(|s| s.phase == Phase::Running && !s.output.is_empty());
    let cancelled = f
        .manager
        .cancel_evaluation(running.evaluation_id.as_ref().unwrap())
        .unwrap();
    assert_eq!(cancelled.process_id, None);
    println!("REAL_CANCEL {}", serde_json::to_string(&cancelled).unwrap());
    f.manager.start_evaluation("target").unwrap();
    f.wait(|s| s.phase == Phase::Running && !s.output.is_empty());
    f.manager.reserve_match("real-priority").unwrap();
    assert_eq!(f.manager.evaluation_snapshot().phase, Phase::Yielded);
    assert_eq!(f.manager.evaluation_snapshot().process_id, None);
    f.manager.stop_reserved_match("real-priority").unwrap();
    {
        let mut catalog = f.catalog.0.lock();
        let profile = &mut catalog.as_mut().unwrap().profile;
        profile.argv.clear();
        profile.adapter = EngineAdapterSettings::KataGoAnalysis(KataGoSettings {
            model_path: Some(model),
            config_path: Some(config.clone()),
            max_visits: 2,
        });
    }
    f.manager.start("target").unwrap();
    let deadline = Instant::now() + Duration::from_secs(65);
    let run = loop {
        if let ForegroundEngineLifecycleDto::Ready { run } = f.manager.snapshot().lifecycle {
            break run;
        }
        assert!(Instant::now() < deadline, "{:?}", f.manager.snapshot());
        std::thread::sleep(Duration::from_millis(50));
    };
    *f.catalog.0.lock() = Some(original.clone());
    f.manager
        .set_continuous_preferences(false, app_model::ContinuousAnalysisBudgetDto::default())
        .unwrap();
    f.manager.start_evaluation("target").unwrap();
    f.wait(|s| s.phase == Phase::Running && !s.output.is_empty());
    let events = f.manager.subscribe();
    let job = f
        .manager
        .start_selected_node_job(engine_manager::SelectedNodeJobRequest {
            run_id: run.run_id.clone(),
            mode: app_model::AnalysisJobModeDto::Finite,
            generation: 1,
            node_path: app_model::NodePath { indices: vec![] },
            board_width: 9,
            board_height: 9,
            position_empty: true,
            query: katago_protocol::AnalysisQuery {
                id: "measurement-handoff".into(),
                moves: vec![],
                initial_stones: vec![],
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
            },
        })
        .unwrap();
    assert_eq!(f.manager.evaluation_snapshot().phase, Phase::Yielded);
    assert_eq!(f.manager.evaluation_snapshot().process_id, None);
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let event = events
            .recv_timeout(deadline.saturating_duration_since(Instant::now()))
            .unwrap();
        if let app_model::ForegroundEngineEventDto::Job { job: observed } = event {
            if observed.job_id == job.job_id
                && observed.outcome == app_model::AnalysisJobOutcomeDto::Completed
            {
                break;
            }
        }
    }
    assert!(
        matches!(f.manager.snapshot().lifecycle, ForegroundEngineLifecycleDto::Ready { run: ref current } if current.run_id == run.run_id)
    );
    assert_eq!(f.manager.snapshot().continuous.enabled, Some(false));
    assert_eq!(f.catalog.get("target"), Some(original));
    assert_eq!(std::fs::read(config).unwrap(), config_before);
    println!(
        "REAL_ANALYSIS_HANDOFF {}",
        serde_json::to_string(&f.manager.evaluation_snapshot()).unwrap()
    );
}

#[test]
fn exit_zero_without_measurement_and_changed_resource_cannot_publish_results() {
    let f = Fixture::new("help");
    f.manager.start_evaluation("target").unwrap();
    let failed = f.wait(|s| s.phase == Phase::Failed);
    assert_eq!(failed.exit_code, Some(0));
    assert!(failed.result.is_none());
    let f = Fixture::new("hold");
    f.running();
    std::fs::write(f.dir.join("included.cfg"), "nnMaxBatchSize=2").unwrap();
    std::fs::write(f.dir.join("release"), "release").unwrap();
    let failed = f.wait(|s| s.phase == Phase::Failed);
    assert!(failed.result.is_none());
    assert!(failed.message.as_deref().unwrap().contains("changed"));
    let f = Fixture::new("success");
    f.manager.start_evaluation("target").unwrap();
    f.wait(|s| s.phase == Phase::Completed);
    assert!(f.manager.start_evaluation("deleted").is_err());
    assert!(f.manager.evaluation_snapshot().result.is_none());
    assert!(f.manager.evaluation_snapshot().output.is_empty());
}

#[test]
fn task_start_and_explicit_continue_preempt_benchmark_without_releasing_pause() {
    let f = Fixture::new("hold");
    {
        let mut catalog = f.catalog.0.lock();
        let profile = &mut catalog.as_mut().unwrap().profile;
        profile.argv = vec!["-override-config".into(), "testMode=hold".into()];
        profile.adapter = EngineAdapterSettings::KataGoAnalysis(KataGoSettings {
            model_path: Some("model.bin".into()),
            config_path: Some("config.cfg".into()),
            max_visits: 10,
        });
    }
    f.manager.start("target").unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    let run = loop {
        if let ForegroundEngineLifecycleDto::Ready { run } = f.manager.snapshot().lifecycle {
            break run;
        }
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(10));
    };
    f.running();
    f.manager
        .start_whole_game_analysis(engine_manager::WholeGameJobRequest {
            run_id: run.run_id.clone(),
            generation: 1,
            work_items: vec![engine_manager::WholeGameWorkItem {
                node_path: app_model::NodePath { indices: vec![] },
                board_width: 9,
                board_height: 9,
                move_number: 0,
                query: katago_protocol::AnalysisQuery {
                    id: "task".into(),
                    moves: vec![],
                    initial_stones: vec![],
                    rules: "chinese".into(),
                    komi: 7.5,
                    board_x_size: 9,
                    board_y_size: 9,
                    analyze_turns: Some(vec![0]),
                    max_visits: Some(100),
                    include_ownership: None,
                    include_policy: None,
                    report_during_search_every: None,
                    override_settings: None,
                },
            }],
        })
        .unwrap();
    assert_eq!(f.manager.evaluation_snapshot().phase, Phase::Yielded);
    let task = f.manager.analysis_task_snapshot().unwrap();
    f.manager.pause_analysis_task(&run.run_id, &task.task_id).unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if f.manager.analysis_task_snapshot().unwrap().state == app_model::AnalysisTaskStateDto::Paused {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "{:?}",
            f.manager.analysis_task_snapshot()
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    let paused = f.manager.analysis_task_snapshot();
    f.running();
    assert_eq!(f.manager.analysis_task_snapshot(), paused);
    f.manager
        .continue_analysis_task(&run.run_id, &task.task_id, 1)
        .unwrap();
    assert_eq!(f.manager.evaluation_snapshot().phase, Phase::Yielded);
    assert_eq!(f.manager.evaluation_snapshot().process_id, None);
}

#[test]
#[ignore = "requires an owned public acquisition root with installation.json; real JSONL/GTP benchmark and managed tamper rejection"]
fn real_managed_evaluation_qualifies_both_adapters_without_primary_authority() {
    let root = PathBuf::from(std::env::var("LIZZIEYZY_ACQUISITION_ROOT").unwrap());
    let installed: app_model::ManagedInstallationDto = serde_json::from_slice(
        &std::fs::read(root.join("installation.json")).unwrap(),
    ).unwrap();
    let mut f = Fixture::new("success");
    f.wait_timeout = Duration::from_secs(130);
    f.manager = ForegroundEngineManager::new(f.catalog.clone(), ForegroundEngineConfig {
        managed_resources_root: Some(root.join("resources")), ..Default::default()
    });
    let settings = KataGoSettings { model_path: Some(installed.model_path.clone()), config_path: Some(installed.config_path.clone()), max_visits: 2 };
    for adapter in [EngineAdapterSettings::KataGoAnalysis(settings.clone()), EngineAdapterSettings::KataGoGtp(settings)] {
        let saved = SavedEngineProfile { profile_id: "target".into(), profile: EngineProfileDto {
            name: "Managed explicit evaluation".into(), program: installed.program.clone(),
            argv: vec!["-t".into(), "1".into(), "-v".into(), "2".into(), "-n".into(), "1".into(),
                "-override-config".into(), "numAnalysisThreads=1,numSearchThreads=1,nnMaxBatchSize=1,nnCacheSizePowerOfTwo=16,nnMutexPoolSizePowerOfTwo=12".into()],
            working_dir: Some(f.dir.to_string_lossy().into_owned()), adapter,
        }};
        *f.catalog.0.lock() = Some(saved.clone());
        f.manager.start_evaluation("target").unwrap();
        let terminal = f.wait(|s| matches!(s.phase, Phase::Completed | Phase::Failed));
        assert_eq!(terminal.phase, Phase::Completed, "{terminal:?}");
        let result = terminal.result.as_ref().unwrap();
        assert_eq!(result.qualified_resource.origin, "project-source-build");
        assert!(result.qualified_resource.resources.iter().any(|r| r.component == "managed_receipt"));
        assert!(result.search_visits_per_second.unwrap() > 0.0);
        assert_eq!(f.catalog.get("target"), Some(saved));
        assert_eq!(f.manager.last_primary_profile_id(), None);
        assert!(f.manager.diagnostic_snapshots().is_empty());
        assert!(matches!(f.manager.snapshot().lifecycle, ForegroundEngineLifecycleDto::NoEngine { failure: None }));
        println!("REAL_MANAGED_EVALUATION {}", serde_json::to_string(&terminal).unwrap());
    }
    let original = std::fs::read(&installed.config_path).unwrap();
    std::fs::write(&installed.config_path, b"numSearchThreads=999\n").unwrap();
    f.manager.start_evaluation("target").unwrap();
    let failed = f.wait(|s| matches!(s.phase, Phase::Completed | Phase::Failed));
    std::fs::write(&installed.config_path, original).unwrap();
    assert_eq!(failed.phase, Phase::Failed);
    assert!(failed.result.is_none());
    assert!(failed.message.as_deref().unwrap().contains("managed_content_changed"));
    assert_eq!(f.manager.last_primary_profile_id(), None);
    println!("REAL_MANAGED_REJECTION {}", serde_json::to_string(&failed).unwrap());
}
