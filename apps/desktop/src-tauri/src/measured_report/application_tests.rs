use super::*;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

#[test]
fn measurement_fixture_publication_is_committed_and_coherent_even_when_delivery_fails() {
    let current = crate::CurrentGameState::default();
    let seen = std::cell::RefCell::new(Vec::new());
    let fixture = "(;GM[1]FF[4]SZ[9]KM[7.5];B[dd];W[ff])";
    let first = reset_fixture(&current, fixture, &|result| {
        let captured = current.capture_measurement_document(result)?;
        assert_eq!(&captured.0, result);
        assert_eq!(captured.2.summary.board_width, 9);
        assert_eq!(captured.2.moves.len(), 2);
        assert!(captured.1.contains("SZ[9]"));
        seen.borrow_mut().push(captured);
        Ok(())
    }).unwrap();
    assert_eq!(seen.borrow().len(), 1);
    let failed = reset_fixture(&current, "(;SZ[13])", &|result| {
        assert!(result.generation > first.generation);
        assert_eq!(current.capture_measurement_document(result)?.2.summary.board_width, 13);
        Err("WebView closed after commit".into())
    });
    assert!(failed.unwrap_err().contains("WebView closed"));
    assert_eq!(current.mainline_projection().unwrap().summary.board_width, 13);
    assert!(current.capture_measurement_document(&first).is_err());
}

#[test]
#[ignore = "requires named real CPU resources; does not qualify GPU reports or WebView timing"]
fn real_application_measurement_scene_smoke() {
    assert_eq!(std::env::var("LIZZIEYZY_REAL_KATAGO").as_deref(), Ok("1"));
    let engine = std::env::var("LIZZIEYZY_KATAGO_ENGINE").expect("named real engine");
    let model = std::env::var("LIZZIEYZY_KATAGO_MODEL").expect("named real model");
    let directory = std::env::temp_dir().join(format!("r13-measured-application-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&directory).unwrap();
    let config_path = directory.join("application.cfg");
    std::fs::write(&config_path,"rules=chinese\nlogAllGTPCommunication=false\nlogSearchInfo=false\nnumSearchThreads=2\nnumAnalysisThreads=1\nnumNNServerThreadsPerModel=1\nnnMaxBatchSize=8\nnnCacheSizePowerOfTwo=16\nnnMutexPoolSizePowerOfTwo=10\nlogToStderr=true\nreportAnalysisWinratesAs=BLACK\n").unwrap();
    let catalog = Arc::new(engine_manager::InMemoryEngineProfileCatalog::new());
    catalog.upsert(engine_manager::SavedEngineProfile {
        profile_id: "measurement-cpu".into(),
        profile: EngineProfileDto {
            name: "Actual CPU application measurement smoke".into(),
            program: engine,
            argv: vec![],
            working_dir: Some(directory.to_string_lossy().into()),
            adapter: EngineAdapterSettings::KataGoGtp(KataGoSettings {
                model_path: Some(model),
                config_path: Some(config_path.to_string_lossy().into()),
                max_visits: 8,
            }),
        },
        thread_policy: Default::default(),
        measured_scenes: Default::default(),
    });
    let mut config = engine_manager::ForegroundEngineConfig::for_tests();
    config.readiness_timeout = Duration::from_secs(120);
    config.job_timeout = Duration::from_secs(120);
    config.stop_drain_timeout = Duration::from_secs(10);
    config.admit_whole_game_analysis = true;
    let manager = ForegroundEngineManager::new(catalog, config);
    let current = crate::CurrentGameState::default();
    current.connect_analysis_manager(manager.clone());
    let events = manager.subscribe();
    let done = AtomicBool::new(false);
    let result = std::thread::scope(|scope| {
        let watchdog_manager = manager.clone();
        let watchdog_done = &done;
        scope.spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(180);
            while !watchdog_done.load(Ordering::Acquire) && Instant::now() < deadline {
                std::thread::sleep(Duration::from_millis(20));
            }
            if !watchdog_done.load(Ordering::Acquire) {
                eprintln!(
                    "ACTUAL_CPU_APPLICATION_TIMEOUT snapshot={:?} task={:?}",
                    watchdog_manager.snapshot(),
                    watchdog_manager.analysis_task_snapshot()
                );
                let _ = watchdog_manager.retire_setup_operation();
            }
        });
        let current_ref = &current;
        let done_ref = &done;
        scope.spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(240);
            while !done_ref.load(Ordering::Acquire) && Instant::now() < deadline {
                if let Ok(ForegroundEngineEventDto::Job { job }) =
                    events.recv_timeout(Duration::from_millis(10))
                {
                    current_ref.attach_from_job_event(&job);
                }
            }
        });
        let result = (|| -> Result<(), String> {
            for scene in [MeasuredSceneDto::Live, MeasuredSceneDto::WholeGame] {
                let lease = manager.begin_setup_operation()?;
                let parameters = if scene == MeasuredSceneDto::Live {
                    serde_json::json!({"numSearchThreads":2,"nnMaxBatchSize":8})
                } else {
                    serde_json::json!({"numAnalysisThreads":1,"numSearchThreadsPerAnalysisThread":2,"nnMaxBatchSize":8})
                };
                lease.begin_row(
                    "measurement-cpu",
                    scene,
                    1,
                    false,
                    serde_json::from_value(parameters).unwrap(),
                )?;
                let run = manager.start_independent_measured_scene("measurement-cpu", scene, &lease)?;
                assert!(manager.reserve_match("unrelated-match").is_err());
                let fixture = "(;GM[1]FF[4]SZ[9]KM[7.5]RU[Chinese];B[dd];W[ff])";
                let game = reset_fixture(&current, fixture, &|_| Ok(()))?;
                let row = exercise(&current, &manager, &lease, &run, &game, 8, true, fixture, &|_| Ok(()));
                let stopped = manager.stop_measured_scene(&run.run_id);
                let row = row?;
                stopped?;
                assert_eq!(manager.measured_process_count(), 0);
                assert!(matches!(
                    manager.snapshot().lifecycle,
                    ForegroundEngineLifecycleDto::NoEngine { .. }
                ));
                assert!(row["seconds"].as_f64().unwrap() > 0.0);
                assert!(row["responseSeconds"].as_f64().unwrap() > 0.0);
                if scene == MeasuredSceneDto::Live {
                    assert!(row["observedRootVisits"].as_u64().unwrap() >= 8);
                } else {
                    assert!(row["rootVisitsByTurn"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .all(|value| value.as_u64().unwrap() >= 8));
                }
                eprintln!(
                    "ACTUAL_CPU_APPLICATION_SCENE scene={scene:?} run={} evidence={row}",
                    run.run_id
                );
            }
            Ok(())
        })();
        done.store(true, Ordering::Release);
        result
    });
    if result.is_err() {
        eprintln!(
            "ACTUAL_CPU_APPLICATION_FAILURE diagnostics={:?}",
            manager.diagnostic_snapshots().last()
        );
    }
    let cleanup = manager.teardown();
    std::fs::remove_dir_all(directory).unwrap();
    result.unwrap();
    cleanup.unwrap();
}
