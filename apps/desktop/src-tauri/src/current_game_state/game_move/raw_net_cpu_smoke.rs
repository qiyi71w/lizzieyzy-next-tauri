use super::*;
use app_model::*;
use engine_manager::{ForegroundEngineConfig, InMemoryEngineProfileCatalog, SavedEngineProfile};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[test]
#[ignore = "requires explicit real CPU KataGo/model and fresh private evidence directory"]
fn real_cpu_raw_net_gateway_black_white_rectangle_and_later_ordinary() {
    let binary = std::env::var("R13_KATAGO").expect("read-only real CPU binary");
    let model = std::env::var("R13_MODEL").expect("read-only main model");
    let directory = std::path::PathBuf::from(std::env::var("R13_EVIDENCE").expect("fresh evidence directory"));
    std::fs::create_dir(&directory).unwrap();
    let config = directory.join("raw-net.cfg");
    std::fs::write(&config, format!("rules=chinese\nnumSearchThreads=1\nnumEigenThreadsPerModel=1\nnnMaxBatchSize=1\nnnCacheSizePowerOfTwo=16\nnnMutexPoolSizePowerOfTwo=12\nmaxVisits=32\nponderingEnabled=false\nlogDir=logs\nlogAllGTPCommunication=true\nlogSearchInfo=true\nlogToStderr=true\nhomeDataDir={}\n", directory.join("home").display())).unwrap();
    let catalog = Arc::new(InMemoryEngineProfileCatalog::new());
    catalog.upsert(SavedEngineProfile { profile_id: "raw-cpu".into(), measured_scenes: Default::default(), thread_policy: Default::default(), profile: EngineProfileDto {
        name: "Real CPU raw-net gateway smoke".into(), program: binary, argv: vec![], working_dir: Some(directory.to_string_lossy().into()),
        adapter: EngineAdapterSettings::KataGoGtp(KataGoSettings { model_path: Some(model), config_path: Some(config.to_string_lossy().into()), max_visits: 8 }),
    }});
    let manager = ForegroundEngineManager::new(catalog, ForegroundEngineConfig::default());
    struct Stop(ForegroundEngineManager);
    impl Drop for Stop { fn drop(&mut self) { let _ = self.0.teardown(); } }
    let _stop = Stop(manager.clone());
    let state = CurrentGameState::default();
    state.connect_analysis_manager(manager.clone());
    manager.set_continuous_preferences(false, Default::default()).unwrap();
    manager.start("raw-cpu").unwrap();
    let until = Instant::now() + Duration::from_secs(120);
    let run = loop {
        match manager.snapshot().lifecycle {
            ForegroundEngineLifecycleDto::Ready { run } => break run,
            ForegroundEngineLifecycleDto::Error { failure, .. } => panic!("{failure:?}"),
            _ => { assert!(Instant::now() < until); std::thread::sleep(Duration::from_millis(10)); }
        }
    };
    std::fs::write(directory.join("run.json"), serde_json::to_vec_pretty(&run).unwrap()).unwrap();
    let mut receipts = Vec::new();
    for (name, sgf, path, color, width, height) in [
        ("black9", "(;SZ[9]RU[Chinese]KM[7.5];B[cg];W[gc])", vec![0,0], PlayerColor::Black, 9, 9),
        ("white9", "(;SZ[9]RU[Chinese]KM[7.5];B[cg];W[gc];B[df])", vec![0,0,0], PlayerColor::White, 9, 9),
        ("white9x7", "(;SZ[9:7]RU[Chinese]KM[7.5];B[cb])", vec![0], PlayerColor::White, 9, 7),
        ("black19", "(;SZ[19]RU[Chinese]KM[7.5])", vec![], PlayerColor::Black, 19, 19),
    ] {
        let opened = state.replace(sgf, None).unwrap();
        let selected = state.select_path(NodePath { indices: path }, opened.generation).unwrap();
        let before = state.serialize().unwrap();
        let request = RawNetRequestDto { run_id: run.run_id.clone(), generation: selected.generation, node_path: selected.selected_path.clone() };
        let result = state.start_raw_net(&manager, request).unwrap().wait().unwrap();
        let result = state.publish_raw_net(&manager, result).unwrap();
        assert_eq!(result.confirmation.identity.run_id, run.run_id);
        assert_eq!(result.confirmation.identity.generation, selected.generation);
        assert_eq!(result.confirmation.identity.node_path, selected.selected_path);
        assert_eq!(result.confirmation.position.to_play, color);
        assert_eq!((result.confirmation.position.board_width, result.confirmation.position.board_height), (width, height));
        let values = result.ownership.as_ref().unwrap();
        assert_eq!(values.len(), usize::from(width) * usize::from(height));
        assert!(values.iter().all(|value| value.is_finite() && (-1.0..=1.0).contains(value)));
        assert!(result.unavailable.is_none());
        assert_eq!(state.serialize().unwrap(), before);
        state.save_to_path(directory.join(format!("{name}.sgf")).to_string_lossy().into(), selected.selected_path.clone()).unwrap();
        assert_eq!(state.serialize().unwrap(), before);
        assert_eq!(manager.snapshot().continuous.phase, ContinuousAnalysisPhaseDto::Off);
        std::fs::write(directory.join(format!("{name}.json")), serde_json::to_vec_pretty(&result).unwrap()).unwrap();
        std::fs::write(directory.join(format!("{name}-diagnostics.json")), serde_json::to_vec_pretty(&manager.diagnostic_snapshots()).unwrap()).unwrap();
        receipts.push(result);
        if name == "black9" {
            let captured = RawNetRequestDto { run_id: run.run_id.clone(), generation: selected.generation,
                node_path: selected.selected_path.clone() };
            let unpublished = state.start_raw_net(&manager, captured.clone()).unwrap().wait().unwrap();
            state.select_path(NodePath::default(), selected.generation).unwrap();
            assert!(state.start_raw_net(&manager, captured).is_err());
            state.select_path(selected.selected_path.clone(), selected.generation).unwrap();
            assert!(state.publish_raw_net(&manager, unpublished).is_err());
            assert_eq!(state.serialize().unwrap(), before);
        }
    }
    // Real later ordinary analysis uses the same manager/reader and authoritative attach path.
    let opened = state.replace("(;SZ[9]RU[Chinese]KM[7.5];B[dd])", None).unwrap();
    let events = manager.subscribe();
    let request = crate::bind_selected_node_job(&state, run.run_id.clone(), opened.generation, opened.selected_path, AnalysisJobModeDto::Finite, Some(8)).unwrap();
    let started = manager.start_selected_node_job(request).unwrap();
    let until = Instant::now() + Duration::from_secs(120);
    let ordinary = loop {
        if let ForegroundEngineEventDto::Job { job } = events.recv_timeout(until.saturating_duration_since(Instant::now())).unwrap() {
            if job.job_id != started.job_id { continue; }
            assert!(!matches!(job.outcome, AnalysisJobOutcomeDto::Failed | AnalysisJobOutcomeDto::Timeout), "{job:?}");
            if job.outcome == AnalysisJobOutcomeDto::Completed { break job; }
        }
    };
    assert!(ordinary.frame.as_ref().unwrap().visits >= 8);
    assert!(state.attach_from_job_event(&ordinary).is_some());
    std::fs::write(directory.join("later-ordinary.json"), serde_json::to_vec_pretty(&ordinary).unwrap()).unwrap();
    state.save_to_path(directory.join("later-ordinary.sgf").to_string_lossy().into(), NodePath::default()).unwrap();
    assert!(state.publish_raw_net(&manager, receipts.remove(0)).is_err());
    manager.teardown().unwrap();
    assert!(matches!(manager.snapshot().lifecycle, ForegroundEngineLifecycleDto::NoEngine { .. }));
    std::fs::write(directory.join("diagnostics.json"), serde_json::to_vec_pretty(&manager.diagnostic_snapshots()).unwrap()).unwrap();
    std::fs::write(directory.join("final-snapshot.json"), serde_json::to_vec_pretty(&manager.snapshot()).unwrap()).unwrap();
    println!("REAL RAW-NET GATEWAY PASS: {} Run={} four geometries/turns, no raw SGF mutation, later ordinary attach, NoEngine/reaped", directory.display(), run.run_id);
}
