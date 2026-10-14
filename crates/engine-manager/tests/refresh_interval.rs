use app_model::*;
use engine_manager::*;
use std::{sync::Arc, time::{Duration, Instant}};

#[test]
#[ignore = "requires explicitly provided real local KataGo resources and private workdir"]
fn real_cpu_refresh_interval_preserves_tree_job_budget_and_pause() {
    assert_eq!(std::env::var("LIZZIEYZY_REAL_KATAGO").unwrap(), "1");
    let directory = std::path::PathBuf::from(std::env::var("LIZZIEYZY_KATAGO_WORKDIR").unwrap());
    let catalog = Arc::new(InMemoryEngineProfileCatalog::new());
    catalog.upsert(SavedEngineProfile {
        profile_id: "refresh".into(), measured_scenes: Default::default(), thread_policy: Default::default(),
        profile: EngineProfileDto {
            name: "Real CPU refresh interval".into(), program: std::env::var("LIZZIEYZY_KATAGO_ENGINE").unwrap(), argv: vec![],
            working_dir: Some(directory.to_string_lossy().into()),
            adapter: EngineAdapterSettings::KataGoGtp(KataGoSettings {
                model_path: Some(std::env::var("LIZZIEYZY_KATAGO_MODEL").unwrap()),
                config_path: Some(std::env::var("LIZZIEYZY_KATAGO_CONFIG").unwrap()), max_visits: 1_000_000,
            }),
        },
    });
    let manager = ForegroundEngineManager::new(catalog, ForegroundEngineConfig::default());
    struct Stop(ForegroundEngineManager);
    impl Drop for Stop { fn drop(&mut self) { let _ = self.0.teardown(); } }
    let _stop = Stop(manager.clone());
    let events = manager.subscribe();
    manager.start("refresh").unwrap();
    let deadline = Instant::now() + Duration::from_secs(60);
    let run = loop {
        match manager.snapshot().lifecycle {
            ForegroundEngineLifecycleDto::Ready { run } => break run,
            ForegroundEngineLifecycleDto::Error { failure, .. } => panic!("{failure:?}"),
            _ => assert!(Instant::now() < deadline),
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    let budget = ContinuousAnalysisBudgetDto { continuous_time_limit_seconds: 15, continuous_visits_limit_enabled: false, ..Default::default() };
    manager.commit_continuous_preferences(true, budget, Some(AnalysisRefreshIntervalDto(17)), || Ok(())).unwrap();
    let document = sgf::CurrentSgfDocument::open("(;SZ[9]RU[Chinese]KM[7.5])").unwrap();
    let path = NodePath::default();
    let query = katago_protocol::analysis_query_from_position(9, 9, 7.5, &[], PlayerColor::Black,
        katago_protocol::AnalysisQueryOptions { id: "pending".into(), rules: "chinese".into(), turn: 0, max_visits: None, include_ownership: Some(true), include_policy: Some(true) }).unwrap();
    manager.follow_continuous_position(SelectedNodeJobRequest { run_id: run.run_id.clone(), generation: 7, node_path: path.clone(), mode: AnalysisJobModeDto::Continuous, query, board_width: 9, board_height: 9, position_empty: true, exact_position: Ok(document.exact_position(&path).unwrap()) });
    let next = |previous: Option<u32>| {
        let until = Instant::now() + Duration::from_secs(30);
        loop {
            match events.recv_timeout(until.saturating_duration_since(Instant::now())).unwrap() {
                ForegroundEngineEventDto::Job { job } if job.outcome == AnalysisJobOutcomeDto::Progress
                    && job.local_focus_output.is_some_and(|output| Some(output.command) != previous)
                    && manager.local_focus_publication(&job).is_ok() => break job,
                ForegroundEngineEventDto::Failure { failure } => panic!("{failure:?}"),
                _ => {}
            }
        }
    };
    let first = next(None);
    let began = Instant::now();
    assert_eq!(manager.snapshot().local_focus.actual_interval_centiseconds, Some(17));
    manager.local_focus_action(LocalFocusRequestDto { run_id: first.run_id.clone(), job_id: first.job_id.clone(), generation: 7, node_path: path.clone(),
        action: LocalFocusActionDto::Add { vertex: MoveVertex::Point(PointDto { x: 3, y: 5 }), max_visits: 1_000_000 } }).unwrap();
    let focused = next(Some(first.local_focus_output.unwrap().command));
    std::fs::write(directory.join("focus-transition-diagnostics.json"), serde_json::to_vec_pretty(&manager.diagnostic_snapshots()).unwrap()).unwrap();
    assert_eq!(manager.snapshot().local_focus.actual_interval_centiseconds, Some(17));
    std::thread::sleep(Duration::from_secs(3));
    manager.commit_continuous_preferences(true, budget, Some(AnalysisRefreshIntervalDto(37)), || Ok(())).unwrap();
    assert!(manager.snapshot().local_focus.points.is_empty());
    assert!(manager.local_focus_publication(&first).is_err());
    assert!(manager.local_focus_publication(&focused).is_err());
    let updated = next(Some(focused.local_focus_output.unwrap().command));
    std::fs::write(directory.join("refresh-transition-diagnostics.json"), serde_json::to_vec_pretty(&manager.diagnostic_snapshots()).unwrap()).unwrap();
    assert_eq!(updated.job_id, first.job_id);
    assert_eq!(updated.run_id, first.run_id);
    assert_eq!(updated.generation, first.generation);
    assert_eq!(updated.node_path, first.node_path);
    assert_eq!(manager.snapshot().local_focus.actual_interval_centiseconds, Some(37));
    assert!(updated.frame.as_ref().unwrap().visits >= focused.frame.as_ref().unwrap().visits);
    let final_event = loop {
        match events.recv_timeout(Duration::from_secs(20)).unwrap() {
            ForegroundEngineEventDto::Job { job } if job.outcome == AnalysisJobOutcomeDto::TimeLimited => break job,
            ForegroundEngineEventDto::Failure { failure } => panic!("{failure:?}"),
            _ => {}
        }
    };
    assert_eq!(final_event.job_id, first.job_id);
    assert!(began.elapsed() < Duration::from_secs(17), "original 15-second budget was reset");
    assert_eq!(manager.local_focus_publication(&final_event), Ok(true));
    assert!(final_event.frame.as_ref().unwrap().visits > 0);
    let elapsed = began.elapsed().as_secs_f64();
    manager.resume_continuous().unwrap();
    let resumed = next(Some(updated.local_focus_output.unwrap().command));
    manager.set_continuous_preferences(false, budget).unwrap();
    loop {
        if let ForegroundEngineEventDto::Job { job } = events.recv_timeout(Duration::from_secs(5)).unwrap() {
            if job.job_id == resumed.job_id && job.outcome == AnalysisJobOutcomeDto::Cancelled { break; }
        }
    }
    manager.commit_continuous_preferences(false, budget, Some(AnalysisRefreshIntervalDto(53)), || Ok(())).unwrap();
    assert_eq!(manager.snapshot().continuous.phase, ContinuousAnalysisPhaseDto::Off);
    assert!(manager.snapshot().local_focus.points.is_empty());
    assert_eq!(manager.snapshot().local_focus.actual_interval_centiseconds, None);
    let receipt = serde_json::json!({ "run": run, "first": first, "focused": focused, "updated": updated, "final": final_event,
        "elapsed_from_first_seconds": elapsed, "paused": manager.snapshot() });
    println!("REAL_REFRESH {}", serde_json::to_string(&receipt).unwrap());
    std::fs::write(directory.join("refresh-receipt.json"), serde_json::to_vec_pretty(&receipt).unwrap()).unwrap();
    manager.teardown().unwrap();
    std::fs::write(directory.join("refresh-diagnostics.json"), serde_json::to_vec_pretty(&manager.diagnostic_snapshots()).unwrap()).unwrap();
}
