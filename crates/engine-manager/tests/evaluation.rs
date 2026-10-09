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

struct DiskStartupCatalog(PathBuf);
impl EngineProfileCatalog for DiskStartupCatalog {
    fn get(&self, id: &str) -> Option<SavedEngineProfile> {
        engine_manager::load_engine_profiles(&self.0).ok()?.profiles.into_iter()
            .find(|record| record.id == id)
            .map(|record| SavedEngineProfile { profile_id: record.id, profile: record.profile })
    }
    fn autoload_profile_id(&self) -> Option<String> {
        engine_manager::load_engine_profiles(&self.0).ok()?.startup_profile_id().map(str::to_owned)
    }
    fn startup_evaluation_target(&self) -> Result<Option<SavedEngineProfile>, String> {
        engine_manager::load_engine_profiles(&self.0)
            .map_err(|_| "Startup evaluation unavailable: saved settings could not be loaded".to_string())?
            .startup_evaluation_target()
    }
    fn preload_profiles(&self) -> Vec<SavedEngineProfile> {
        engine_manager::load_engine_profiles(&self.0).unwrap().profiles.into_iter()
            .filter(|record| record.preload)
            .map(|record| SavedEngineProfile { profile_id: record.id, profile: record.profile })
            .collect()
    }
}

impl Fixture {
    fn install_startup(&mut self, enabled: bool, target: Option<&str>) -> PathBuf {
        let mut settings = engine_manager::default_engine_profiles_settings();
        let saved = self.catalog.get("target").unwrap();
        settings.profiles.push(engine_manager::EngineProfileRecord { id: saved.profile_id, profile: saved.profile, preload: false });
        settings.startup_evaluation = app_model::StartupEvaluationSettingsDto {
            enabled, target_profile_id: target.map(str::to_owned),
        };
        let path = self.dir.join("catalog.json");
        engine_manager::save_engine_profiles(&path, settings).unwrap();
        self.fresh_startup(&path);
        path
    }
    fn fresh_startup(&mut self, path: &std::path::Path) {
        self.manager.teardown().unwrap();
        self.manager = ForegroundEngineManager::new(Arc::new(DiskStartupCatalog(path.into())),
            ForegroundEngineConfig { readiness_timeout: self.wait_timeout, ..ForegroundEngineConfig::default() });
    }
}

#[test]
fn startup_authorization_is_frozen_one_shot_and_not_editor_selection() {
    let mut f = Fixture::new("success");
    let path = f.install_startup(false, Some("target"));
    let mut settings = engine_manager::load_engine_profiles(&path).unwrap();
    settings.startup_evaluation.enabled = true;
    engine_manager::save_engine_profiles(&path, settings).unwrap();
    f.manager.apply_startup_evaluation();
    assert_eq!(f.manager.evaluation_snapshot().phase, Phase::Idle);
    assert!(!f.dir.join("observed.json").exists());
    f.fresh_startup(&path);
    let bytes = std::fs::read(&path).unwrap();
    let foreground = f.manager.snapshot();
    f.manager.apply_startup_evaluation();
    let result = f.wait(|s| s.phase == Phase::Completed);
    assert_eq!(result.target_id.as_deref(), Some("target"));
    assert_eq!(result.result.as_ref().unwrap().search_visits_per_second, Some(123.5));
    f.manager.apply_startup_evaluation();
    assert_eq!(f.manager.evaluation_snapshot(), result);
    assert_eq!(f.manager.snapshot(), foreground);
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    f.manager.cancel_evaluation(result.evaluation_id.as_ref().unwrap()).unwrap();
    f.manager.apply_startup_evaluation();
    assert_eq!(f.manager.evaluation_snapshot().phase, Phase::Cancelled);
}

#[test]
fn startup_missing_corrupt_deleted_and_changed_targets_never_fall_back() {
    let mut f = Fixture::new("success");
    for target in [None, Some("missing")] {
        f.install_startup(true, target);
        f.manager.apply_startup_evaluation();
        assert_eq!(f.manager.evaluation_snapshot().phase, Phase::Unavailable);
        assert!(!f.dir.join("observed.json").exists());
    }
    for delete in [false, true] {
        let path = f.install_startup(true, Some("target"));
        let mut settings = engine_manager::load_engine_profiles(&path).unwrap();
        if delete { settings.profiles.pop(); } else { settings.profiles[1].profile.name = "later revision".into(); }
        engine_manager::save_engine_profiles(&path, settings).unwrap();
        f.manager.apply_startup_evaluation();
        assert_eq!(f.manager.evaluation_snapshot().phase, Phase::Unavailable);
        assert!(!f.dir.join("observed.json").exists());
    }
    let path = f.dir.join("catalog.json");
    std::fs::write(&path, "{corrupt").unwrap();
    f.fresh_startup(&path);
    f.manager.apply_startup_evaluation();
    assert_eq!(f.manager.evaluation_snapshot().phase, Phase::Unavailable);
    assert_eq!(std::fs::read_to_string(path).unwrap(), "{corrupt");
}

#[test]
fn startup_yields_to_match_and_never_reactivates_after_handback() {
    let mut f = Fixture::new("hold");
    let path = f.install_startup(true, Some("target"));
    let bytes = std::fs::read(&path).unwrap();
    f.manager.apply_startup_evaluation();
    let running = f.wait(|s| s.phase == Phase::Running && !s.output.is_empty());
    f.manager.reserve_match("startup-priority").unwrap();
    let yielded = f.manager.evaluation_snapshot();
    assert_eq!(yielded.phase, Phase::Yielded);
    assert_eq!(yielded.evaluation_id, running.evaluation_id);
    assert!(yielded.process_id.is_none() && yielded.result.is_none());
    f.manager.stop_reserved_match("startup-priority").unwrap();
    f.manager.apply_startup_evaluation();
    assert_eq!(f.manager.evaluation_snapshot(), yielded);
    assert_eq!(std::fs::read(path).unwrap(), bytes);
}

#[test]
fn startup_q1_modes_do_not_retarget_evaluation_and_wait_only_for_autoload() {
    for startup in [app_model::EngineStartupPolicyDto::Off,
        app_model::EngineStartupPolicyDto::Fixed { profile_id: "primary".into() },
        app_model::EngineStartupPolicyDto::LastPrimary] {
        let mut f = Fixture::new("success");
        let path = f.install_startup(true, Some("target"));
        let mut settings = engine_manager::load_engine_profiles(&path).unwrap();
        let mut primary = settings.profiles[1].clone();
        primary.id = "primary".into();
        primary.profile.argv = vec!["-override-config".into(), "testMode=hold".into()];
        primary.profile.adapter = EngineAdapterSettings::KataGoAnalysis(KataGoSettings {
            model_path: Some("model.bin".into()), config_path: Some("config.cfg".into()), max_visits: 10,
        });
        settings.profiles.push(primary);
        settings.last_primary_profile_id = Some("primary".into());
        settings.startup = startup.clone();
        engine_manager::save_engine_profiles(&path, settings).unwrap();
        f.fresh_startup(&path);
        f.manager.apply_autoload().unwrap();
        f.manager.apply_startup_evaluation();
        let result = f.wait(|s| s.phase == Phase::Completed);
        assert_eq!(result.target_id.as_deref(), Some("target"));
        if startup == app_model::EngineStartupPolicyDto::Off {
            assert!(matches!(f.manager.snapshot().lifecycle, ForegroundEngineLifecycleDto::NoEngine { .. }));
        } else {
            assert!(matches!(f.manager.snapshot().lifecycle, ForegroundEngineLifecycleDto::Ready { run } if run.profile_id == "primary"));
        }
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
            exact_position: Err("JSONL fixture does not require exact GTP history".into()),
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
fn startup_task_start_and_explicit_continue_preempt_benchmark_without_releasing_pause() {
    let mut f = Fixture::new("hold");
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
    f.install_startup(true, Some("target"));
    f.manager.start("target").unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    let run = loop {
        if let ForegroundEngineLifecycleDto::Ready { run } = f.manager.snapshot().lifecycle {
            break run;
        }
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(10));
    };
    f.manager.apply_startup_evaluation();
    f.wait(|s| s.phase == Phase::Running && !s.output.is_empty());
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
        let qualifying = f.manager.start_evaluation("target").unwrap();
        assert_eq!(qualifying.phase, Phase::Qualifying);
        let cancelled = f.manager.cancel_evaluation(qualifying.evaluation_id.as_deref().unwrap()).unwrap();
        assert_eq!(cancelled.phase, Phase::Cancelled);
        assert!(cancelled.result.is_none());
        assert!(cancelled.process_id.is_none());
        // Re-admission waits for the actual retired qualification worker, not only its DTO.
        let restart = Instant::now();
        f.manager.start_evaluation("target").unwrap();
        assert!(restart.elapsed() < Duration::from_secs(3));
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

#[test]
fn startup_failure_cancel_and_preexisting_match_preserve_durable_policy() {
    for mode in ["failure", "hold"] {
        let mut f = Fixture::new(mode);
        let path = f.install_startup(true, Some("target"));
        let bytes = std::fs::read(&path).unwrap();
        f.manager.apply_startup_evaluation();
        if mode == "failure" {
            f.wait(|s| s.phase == Phase::Failed);
        } else {
            let running = f.wait(|s| s.phase == Phase::Running && !s.output.is_empty());
            f.manager.cancel_evaluation(running.evaluation_id.as_ref().unwrap()).unwrap();
            std::fs::write(f.dir.join("release"), "late completion").unwrap();
            f.manager.apply_startup_evaluation();
            assert_eq!(f.manager.evaluation_snapshot().phase, Phase::Cancelled);
            assert!(f.manager.evaluation_snapshot().process_id.is_none());
        }
        assert!(f.manager.evaluation_snapshot().result.is_none());
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        f.fresh_startup(&path);
        f.manager.reserve_match("already-active").unwrap();
        f.manager.apply_startup_evaluation();
        assert_eq!(f.manager.evaluation_snapshot().phase, Phase::Yielded);
        f.manager.stop_reserved_match("already-active").unwrap();
        f.manager.apply_startup_evaluation();
        assert_eq!(f.manager.evaluation_snapshot().phase, Phase::Yielded);
    }
}

#[test]
#[ignore = "requires actual frozen KataGo/model/private config; fresh manager startup and real analysis handoff"]
fn real_startup_evaluation_result_and_autoload_analysis_handoff() {
    real_startup_handoff(false);
}

#[test]
#[ignore = "requires actual KataGo; startup slots coexist with explicit GTP controls before foreground handoff"]
fn real_startup_background_slots_runtime_controls_and_foreground_handoff() {
    real_startup_handoff(true);
}

fn real_startup_handoff(gtp: bool) {
    let engine = std::env::var("LIZZIEYZY_KATAGO_ENGINE").unwrap();
    let model = std::env::var("LIZZIEYZY_KATAGO_MODEL").unwrap();
    let config = std::env::var("LIZZIEYZY_KATAGO_CONFIG").unwrap();
    let workdir = std::env::var("LIZZIEYZY_KATAGO_WORKDIR").unwrap();
    let config_bytes = std::fs::read(&config).unwrap();
    let mut f = Fixture::new("success");
    f.wait_timeout = Duration::from_secs(120);
    f.catalog.0.lock().as_mut().unwrap().profile = EngineProfileDto {
        name: "Actual startup benchmark".into(), program: engine,
        argv: vec!["gtp".into(), "-config".into(), config.clone(), "-model".into(), model.clone(),
            "-t".into(), "1".into(), "-v".into(), "2".into(), "-n".into(), "1".into()],
        working_dir: Some(workdir), adapter: EngineAdapterSettings::GenericGtp(app_model::GenericGtpSettings::default()),
    };
    let path = f.install_startup(true, Some("target"));
    let before = std::fs::read(&path).unwrap();
    f.manager.apply_autoload().unwrap();
    f.manager.apply_startup_evaluation();
    let done = f.wait(|s| matches!(s.phase, Phase::Completed | Phase::Failed));
    assert_eq!(done.phase, Phase::Completed, "{done:?}");
    assert!(done.result.as_ref().unwrap().search_visits_per_second.unwrap() > 0.0);
    assert_eq!(done.target_id.as_deref(), Some("target"));
    assert_eq!(std::fs::read(&path).unwrap(), before);
    println!("REAL_STARTUP_RESULT {}", serde_json::to_string(&done).unwrap());
    let mut settings = engine_manager::load_engine_profiles(&path).unwrap();
    let mut primary = settings.profiles[1].clone();
    primary.id = "primary".into();
    primary.profile.argv.clear();
    let adapter_settings = KataGoSettings {
        model_path: Some(model), config_path: Some(config.clone()), max_visits: 2,
    };
    primary.profile.adapter = if gtp { EngineAdapterSettings::KataGoGtp(adapter_settings) }
        else { EngineAdapterSettings::KataGoAnalysis(adapter_settings) };
    settings.profiles[1].profile.argv[8] = "100000".into();
    let mut prepared = primary.clone();
    prepared.id = "prepared".into();
    prepared.preload = true;
    settings.profiles.push(prepared);
    settings.profiles.push(primary);
    settings.startup = app_model::EngineStartupPolicyDto::Fixed { profile_id: "primary".into() };
    engine_manager::save_engine_profiles(&path, settings).unwrap();
    f.fresh_startup(&path);
    let before = std::fs::read(&path).unwrap();
    f.manager.set_continuous_preferences(false, app_model::ContinuousAnalysisBudgetDto::default()).unwrap();
    f.manager.apply_autoload().unwrap();
    f.manager.schedule_startup_preloads();
    f.manager.apply_startup_evaluation();
    f.wait(|s| s.phase == Phase::Running && !s.output.is_empty());
    let ForegroundEngineLifecycleDto::Ready { run } = f.manager.snapshot().lifecycle else { panic!("autoload not ready") };
    assert_eq!(run.profile_id, "primary");
    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        let prepared = f.manager.preload_snapshot();
        if prepared.iter().any(|slot| slot.phase == app_model::EnginePreloadPhaseDto::Ready) { break; }
        assert!(Instant::now() < deadline, "{prepared:?}");
        std::thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(f.manager.evaluation_snapshot().phase, Phase::Running);
    assert_eq!(f.manager.last_primary_profile_id().as_deref(), Some("primary"));
    if gtp {
        let evaluation_id = f.manager.evaluation_snapshot().evaluation_id;
        let prepared = f.manager.preload_snapshot();
        for (action, value, expected) in [
            (app_model::RuntimeThreadsActionDto::Apply, Some(2), 2),
            (app_model::RuntimeThreadsActionDto::Reset, None, 1),
        ] {
            let result = f.manager.runtime_threads(app_model::RuntimeThreadsRequestDto {
                identity: app_model::RuntimeControlIdentityDto {
                    run_id: run.run_id.clone(),
                    profile_revision: run.qualified_resource.as_ref().unwrap().profile_revision.clone(),
                    request_id: uuid::Uuid::new_v4().to_string(),
                }, action, value,
            }).unwrap();
            assert_eq!(result.actual, Some(expected));
            assert_eq!(result.status, app_model::RuntimeThreadsStatusDto::Confirmed);
            assert_eq!(f.manager.evaluation_snapshot().evaluation_id, evaluation_id);
            assert_eq!(f.manager.evaluation_snapshot().phase, Phase::Running);
            assert_eq!(f.manager.preload_snapshot(), prepared);
            assert_eq!(f.manager.snapshot().continuous.enabled, Some(false));
        }
    }
    let events = f.manager.subscribe();
    let job = f.manager.start_selected_node_job(engine_manager::SelectedNodeJobRequest {
        run_id: run.run_id.clone(), mode: app_model::AnalysisJobModeDto::Finite, generation: 1,
        node_path: app_model::NodePath { indices: vec![] }, board_width: 9, board_height: 9, position_empty: true,
        exact_position: sgf::CurrentSgfDocument::open("(;SZ[9]RU[Chinese]KM[7.5])").unwrap()
            .exact_position(&app_model::NodePath { indices: vec![] }).map_err(|error| error.to_string()),
        query: katago_protocol::AnalysisQuery { id: "startup-handoff".into(), moves: vec![], initial_stones: vec![],
            rules: "chinese".into(), komi: 7.5, board_x_size: 9, board_y_size: 9, analyze_turns: Some(vec![0]),
            max_visits: Some(2), include_ownership: None, include_policy: None, report_during_search_every: None, override_settings: None },
    }).unwrap();
    let yielded = f.manager.evaluation_snapshot();
    assert_eq!(yielded.phase, Phase::Yielded);
    assert!(yielded.process_id.is_none() && yielded.result.is_none());
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let event = events.recv_timeout(deadline.saturating_duration_since(Instant::now())).unwrap();
        if let app_model::ForegroundEngineEventDto::Job { job: observed } = event {
            if observed.job_id == job.job_id && observed.outcome == app_model::AnalysisJobOutcomeDto::Completed { break; }
        }
    }
    f.manager.apply_startup_evaluation();
    assert_eq!(f.manager.evaluation_snapshot(), yielded);
    assert!(f.manager.preload_snapshot().iter().all(|slot| slot.phase == app_model::EnginePreloadPhaseDto::Cancelled));
    assert!(matches!(f.manager.snapshot().lifecycle, ForegroundEngineLifecycleDto::Ready { run: current } if current.run_id == run.run_id));
    assert_eq!(f.manager.snapshot().continuous.enabled, Some(false));
    assert_eq!(std::fs::read(path).unwrap(), before);
    assert_eq!(std::fs::read(config).unwrap(), config_bytes);
    println!("REAL_STARTUP_HANDOFF {}", serde_json::to_string(&yielded).unwrap());
}
