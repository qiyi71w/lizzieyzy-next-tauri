use super::*;
use std::io::Write;

#[test]
#[ignore = "requires explicit read-only KataGo/model and a fresh private evidence directory"]
fn real_cpu_continuation_jsonl_and_gtp_both_modes_stop_undo_save() {
    let binary = std::env::var("R13_KATAGO").expect("R13_KATAGO");
    let model = std::env::var("R13_MODEL").expect("R13_MODEL");
    let evidence = std::path::PathBuf::from(std::env::var("R13_EVIDENCE").expect("R13_EVIDENCE"));
    std::fs::create_dir(&evidence).expect("fresh evidence directory");
    for gtp in [false, true] {
        let directory = evidence.join(if gtp { "gtp" } else { "jsonl" });
        std::fs::create_dir(&directory).unwrap();
        let config = directory.join("cpu.cfg");
        std::fs::write(&config, "numSearchThreads=1\nnumAnalysisThreads=1\nnumNNServerThreadsPerModel=1\nnnMaxBatchSize=1\nnnCacheSizePowerOfTwo=16\nnnMutexPoolSizePowerOfTwo=12\nmaxVisits=8\nlogToStderr=true\nlogSearchInfo=false\nlogAllGTPCommunication=false\nlogSearchInfoForChosenMove=false\nrules=tromp-taylor\nhomeDataDir=.\n").unwrap();
        let catalog = Arc::new(InMemoryEngineProfileCatalog::new());
        let settings = app_model::KataGoSettings { model_path: Some(model.clone()), config_path: Some(config.to_string_lossy().into()), max_visits: 8 };
        catalog.upsert(engine_manager::SavedEngineProfile { profile_id: "test".into(), thread_policy: Default::default(), profile: app_model::EngineProfileDto {
            name: "R13 continuation CPU".into(), program: binary.clone(), argv: vec![], working_dir: Some(directory.to_string_lossy().into()),
            adapter: if gtp { app_model::EngineAdapterSettings::KataGoGtp(settings) } else { app_model::EngineAdapterSettings::KataGoAnalysis(settings) },
        }});
        let manager = ForegroundEngineManager::new(catalog.clone(), ForegroundEngineConfig { readiness_timeout: Duration::from_secs(120), job_timeout: Duration::from_secs(120), ..Default::default() });
        let events = manager.subscribe();
        // The fixture removes its scratch directory on drop; immutable receipts stay outside it.
        let transient = directory.join("transient");
        std::fs::create_dir(&transient).unwrap();
        let engine = LiveFixture { directory: transient, manager, catalog, events };
        let state = CurrentGameState::default();
        state.connect_analysis_manager(engine.manager.clone());
        engine.manager.set_continuous_preferences(false, app_model::ContinuousAnalysisBudgetDto { continuous_time_limit_enabled: false, continuous_visits_limit_enabled: false, ..Default::default() }).unwrap();
        let game = state.replace("(;SZ[9]RU[Chinese]KM[7.5]C[personal root];B[ee](;W[aa]C[old first])(;W[cc]C[selected]))", None).unwrap();
        state.select_path(NodePath { indices: vec![0] }, game.generation).unwrap();
        engine.manager.start("test").unwrap();
        let deadline = Instant::now() + Duration::from_secs(120);
        let run = loop {
            match engine.manager.snapshot().lifecycle {
                app_model::ForegroundEngineLifecycleDto::Ready { run } => break run,
                app_model::ForegroundEngineLifecycleDto::Error { failure, .. } => panic!("{failure:?}"),
                _ => { assert!(Instant::now() < deadline); let _ = engine.events.recv_timeout(Duration::from_millis(20)); }
            }
        };
        engine.manager.set_diagnostic_trace(&run.run_id, true).unwrap();
        std::fs::write(directory.join("ready.json"), serde_json::to_vec_pretty(&engine.manager.snapshot()).unwrap()).unwrap();
        let start = begin(&engine, &state, Mode::LeafOnly, vec![app_model::AnalysisBranchChoiceDto { parent: NodePath { indices: vec![0] }, child: 1 }]);
        let first = real_advance(&engine, &state, &start, &directory);
        assert_eq!(first.current_game.as_ref().unwrap().selected_path.indices, vec![0,1]);
        assert_ne!(first.request_id, start.request_id);
        let second = real_advance(&engine, &state, &first, &directory);
        let committed = second.current_game.as_ref().unwrap();
        assert_eq!(committed.selected_path.indices, vec![0,1,0]);
        assert!(committed.can_undo);
        assert_ne!(second.request_id, first.request_id);
        real_frame(&engine, second.request_id.as_deref().unwrap(), &directory);
        let stopped = state.stop_continuation(&engine.manager, "Actual CPU Stop");
        assert_eq!(stopped.phase, Phase::Stopped);
        let before = state.serialize().unwrap();
        let late = state.continuation_beat_at(&engine.manager, start.session_id.as_deref().unwrap(), Instant::now() + Duration::from_secs(20));
        assert_eq!(late.phase, Phase::Stopped); assert_eq!(state.serialize().unwrap(), before);
        let file = directory.join("leaf-complete.sgf");
        state.save_to_path(file.to_string_lossy().into(), committed.selected_path.clone()).unwrap();
        let parsed = CurrentSgfDocument::open(&std::fs::read_to_string(file).unwrap()).unwrap();
        assert_eq!(parsed.snapshot(&NodePath { indices: vec![0,0] }).unwrap().personal_comment, "old first");
        assert_eq!(parsed.snapshot(&NodePath { indices: vec![0,1] }).unwrap().personal_comment, "selected");
        assert_eq!(parsed.snapshot(&committed.selected_path).unwrap().position.move_number, 3);
        let undone = state.undo(committed.generation).unwrap();
        assert_eq!(undone.selected_path.indices, vec![0,1]);
        std::fs::write(directory.join("leaf-stopped.json"), serde_json::to_vec_pretty(&stopped).unwrap()).unwrap();
        std::fs::write(directory.join("leaf-undo.json"), serde_json::to_vec_pretty(&undone).unwrap()).unwrap();
        let game = state.replace("(;SZ[9]RU[Chinese]KM[6.5]C[every personal];B[ee];W[]C[recorded pass])", None).unwrap();
        state.select_path(NodePath { indices: vec![0] }, game.generation).unwrap();
        let start = begin(&engine, &state, Mode::EveryBeat, vec![]);
        let advanced = real_advance(&engine, &state, &start, &directory);
        let committed = advanced.current_game.as_ref().unwrap();
        assert_eq!(committed.selected_path.indices, vec![0,1], "engine action must not prefer recorded pass");
        assert_eq!(committed.tree.children[0].children.len(), 2);
        real_frame(&engine, advanced.request_id.as_deref().unwrap(), &directory);
        state.stop_continuation(&engine.manager, "Actual every-beat Stop");
        let file = directory.join("every-complete.sgf");
        state.save_to_path(file.to_string_lossy().into(), committed.selected_path.clone()).unwrap();
        let parsed = CurrentSgfDocument::open(&std::fs::read_to_string(file).unwrap()).unwrap();
        assert_eq!(parsed.snapshot(&NodePath { indices: vec![0,0] }).unwrap().personal_comment, "recorded pass");
        assert_eq!(parsed.snapshot(&committed.selected_path).unwrap().position.move_number, 2);
        assert!(matches!(engine.manager.snapshot().lifecycle, app_model::ForegroundEngineLifecycleDto::Ready { run: current } if current.run_id == run.run_id));
        std::fs::write(directory.join("every-advanced.json"), serde_json::to_vec_pretty(&advanced).unwrap()).unwrap();
        std::fs::write(directory.join("diagnostics.json"), serde_json::to_vec_pretty(&engine.manager.diagnostic_snapshots()).unwrap()).unwrap();
        engine.manager.teardown().unwrap();
    }
    std::fs::write(evidence.join("result.txt"), format!("PASS actual JSONL/GTP CPU: both modes, nonfirst selected branch, real legal single-move creation, fresh next requests, Stop, Undo, SGF Save/parseback.\nbinary={binary}\nmodel={model}\n")).unwrap();
}

fn real_frame(engine: &LiveFixture, request: &str, directory: &std::path::Path) -> app_model::AnalysisJobEventDto {
    let deadline = Instant::now() + Duration::from_secs(90);
    let mut log = std::fs::OpenOptions::new().create(true).append(true).open(directory.join("events.jsonl")).unwrap();
    loop {
        assert!(Instant::now() < deadline, "real frame deadline: {:?}", engine.manager.snapshot());
        if let Ok(event) = engine.events.recv_timeout(Duration::from_millis(20)) {
            writeln!(log, "{}", serde_json::to_string(&event).unwrap()).unwrap();
            if let app_model::ForegroundEngineEventDto::Job { job } = event {
                if job.job_id == request && job.frame.is_some() { return job; }
                if let Some(failure) = job.failure { panic!("{failure:?}"); }
            }
        }
    }
}
fn real_advance(engine: &LiveFixture, state: &CurrentGameState, previous: &app_model::ContinuationSnapshotDto, directory: &std::path::Path) -> app_model::ContinuationSnapshotDto {
    real_frame(engine, previous.request_id.as_deref().unwrap(), directory);
    let deadline = Instant::now() + Duration::from_secs(90);
    loop {
        let next = state.continuation_beat(&engine.manager, previous.session_id.as_deref().unwrap());
        assert_ne!(next.phase, Phase::Failed, "{next:?}");
        if next.current_game.is_some() {
            std::fs::write(directory.join(format!("step-{}.json", next.request_id.as_deref().unwrap_or("terminal"))), serde_json::to_vec_pretty(&next).unwrap()).unwrap();
            return next;
        }
        assert!(Instant::now() < deadline);
        let _ = engine.events.recv_timeout(Duration::from_millis(20));
    }
}
