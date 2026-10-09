use app_model::*;
use engine_manager::*;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

#[test]
#[ignore = "requires explicitly provided real KataGo 1.18.2 resources"]
fn real_katago_gtp_rules_and_exact_selected_positions() {
    assert_eq!(std::env::var("LIZZIEYZY_REAL_KATAGO").unwrap(), "1");
    let catalog = Arc::new(InMemoryEngineProfileCatalog::new());
    catalog.upsert(SavedEngineProfile {
        profile_id: "gtp".into(),
        profile: EngineProfileDto {
            name: "explicit KataGo GTP".into(),
            program: std::env::var("LIZZIEYZY_KATAGO_ENGINE").unwrap(),
            argv: vec![],
            working_dir: Some(std::env::var("LIZZIEYZY_KATAGO_WORKDIR").unwrap()),
            adapter: EngineAdapterSettings::KataGoGtp(KataGoSettings {
                model_path: Some(std::env::var("LIZZIEYZY_KATAGO_MODEL").unwrap()),
                config_path: Some(std::env::var("LIZZIEYZY_KATAGO_CONFIG").unwrap()),
                max_visits: 4,
            }),
        },
    });
    let manager = ForegroundEngineManager::new(catalog, ForegroundEngineConfig::default());
    manager.start("gtp").unwrap();
    let deadline = Instant::now() + Duration::from_secs(40);
    let run = loop {
        match manager.snapshot().lifecycle {
            ForegroundEngineLifecycleDto::Ready { run } => break run,
            ForegroundEngineLifecycleDto::Error { failure, .. } => panic!("{failure:?}"),
            _ => assert!(Instant::now() < deadline),
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    println!("RUN {}", serde_json::to_string(&run).unwrap());
    let events = manager.subscribe();
    assert!(!run.capability_snapshot.as_ref().unwrap().game_move);
    assert!(run.capability_snapshot.as_ref().unwrap().analysis.as_ref().unwrap().selected_node_analysis);
    assert!(manager.diagnostic_snapshots().last().unwrap().records.iter().any(|record| record.source == "startup-probe-stderr"));
    let cases = [
        ("(;SZ[9]RU[Chinese]KM[7.5])", vec![]),
        ("(;SZ[9]RU[Chinese]KM[6.5]PL[W])", vec![]),
        ("(;SZ[9]RU[Chinese-KGS]KM[6.5];B[dd];W[];B[fg])", vec![0, 0, 0]),
        (
            "(;SZ[9]RU[Chinese-KGS]KM[0.5]HA[2]AB[cc][gg]PL[W];W[dd];B[])",
            vec![0, 0],
        ),
        (
            "(;SZ[9]RU[Chinese]KM[6.5];B[dd](;W[ff])(;W[ee];B[]))",
            vec![0, 1, 0],
        ),
        ("(;SZ[9]RU[Chinese]KM[6.5];B[dd](;W[ff])(;W[ee];B[]))", vec![0]),
    ];
    for (generation, (text, indices)) in cases.into_iter().enumerate() {
        let document = sgf::CurrentSgfDocument::open(text).unwrap();
        let node_path = NodePath { indices };
        let position = document.exact_position(&node_path).unwrap();
        let expected = position.dto().clone();
        let result = manager.confirm_ordinary_rules(GameMoveRequest {
            identity: GameMoveRequestDto {
                run_id: run.run_id.clone(),
                generation: 1,
                node_path: node_path.clone(),
                budget: ComputeBudgetDto {
                    deadline_ms: 10000,
                    max_visits: None,
                },
            },
            position,
        });
        println!("CASE {text}: {result:?}");
        let snapshot = result.unwrap();
        assert_eq!(snapshot.position, expected);
        assert_eq!(snapshot.reader_id, run.run_id);
        assert_eq!(snapshot.true_final_move, expected.moves.last().cloned());
        let selected_snapshot = document.snapshot(&node_path).unwrap();
        let query = katago_protocol::analysis_query_from_position(9, 9, expected.komi,
            &selected_snapshot.position.stones, expected.to_play,
            katago_protocol::AnalysisQueryOptions { id: "pending".into(), rules: "chinese".into(),
                turn: expected.moves.len() as u32, max_visits: Some(16), include_ownership: Some(true), include_policy: Some(true) }).unwrap();
        let started = manager.start_selected_node_job(SelectedNodeJobRequest {
            run_id: run.run_id.clone(), generation: generation as u64 + 1, node_path,
            mode: AnalysisJobModeDto::Finite, query, board_width: 9, board_height: 9,
            position_empty: selected_snapshot.position.stones.is_empty(),
            exact_position: Ok(document.exact_position(&selected_snapshot.path).unwrap()),
        }).unwrap();
        let until = Instant::now() + Duration::from_secs(40);
        loop {
            match events.recv_timeout(until.saturating_duration_since(Instant::now())).unwrap() {
                ForegroundEngineEventDto::Job { job } if job.job_id == started.job_id => {
                    println!("MAIN_ANALYSIS {}", serde_json::to_string(&job).unwrap());
                    assert!(!matches!(job.outcome, AnalysisJobOutcomeDto::Failed | AnalysisJobOutcomeDto::Timeout));
                    if job.outcome == AnalysisJobOutcomeDto::Completed {
                        let frame = job.frame.unwrap();
                        assert!(frame.visits >= 16);
                        assert!(!frame.candidates.is_empty());
                        assert!(!frame.candidates[0].pv.is_empty());
                        assert_eq!(frame.ownership.as_ref().unwrap().len(), 81);
                        assert_eq!(frame.policy, None);
                        assert_eq!(frame.turn, expected.moves.len() as u32);
                        break;
                    }
                }
                ForegroundEngineEventDto::Failure { failure } => panic!("{failure:?}"),
                _ => {},
            }
        }
        assert_eq!(document.serialize().unwrap(), sgf::CurrentSgfDocument::open(text).unwrap().serialize().unwrap());
    }
    let document = sgf::CurrentSgfDocument::open("(;SZ[9]RU[Chinese]KM[6.5]C[personal];B[dd](;W[ff])(;W[ee]))").unwrap();
    let before = document.serialize().unwrap();
    let request = |indices: Vec<u32>| {
        let node_path = NodePath { indices };
        let snapshot = document.snapshot(&node_path).unwrap();
        let position = document.exact_position(&node_path).unwrap();
        SelectedNodeJobRequest {
            run_id: run.run_id.clone(), generation: 77, node_path, mode: AnalysisJobModeDto::Continuous,
            board_width: 9, board_height: 9, position_empty: snapshot.position.stones.is_empty(),
            query: katago_protocol::analysis_query_from_position(9, 9, 6.5, &snapshot.position.stones,
                snapshot.position.to_play, katago_protocol::AnalysisQueryOptions { id: "pending".into(),
                    rules: "chinese".into(), turn: snapshot.position.move_number, max_visits: None,
                    include_ownership: Some(true), include_policy: Some(true) }).unwrap(),
            exact_position: Ok(position),
        }
    };
    let wait_event = |outcome, path: &[u32]| {
        let deadline = Instant::now() + Duration::from_secs(30);
        loop {
            match events.recv_timeout(deadline.saturating_duration_since(Instant::now())).unwrap() {
                ForegroundEngineEventDto::Job { job } if job.outcome == outcome && job.node_path.indices == path => {
                    println!("CONTINUOUS {}", serde_json::to_string(&job).unwrap());
                    break job;
                }
                ForegroundEngineEventDto::Failure { failure } => panic!("{failure:?}"),
                _ => {},
            }
        }
    };
    let budget = ContinuousAnalysisBudgetDto { continuous_time_limit_enabled: false, ..Default::default() };
    manager.set_continuous_preferences(true, budget).unwrap();
    manager.follow_continuous_position(request(vec![0, 0]));
    let first = wait_event(AnalysisJobOutcomeDto::Progress, &[0, 0]);
    manager.follow_continuous_position(request(vec![0, 1]));
    let branch = wait_event(AnalysisJobOutcomeDto::Progress, &[0, 1]);
    assert_ne!(branch.job_id, first.job_id);
    assert_eq!(branch.run_id, run.run_id);
    assert_eq!(branch.frame.as_ref().unwrap().turn, 2);
    manager.set_continuous_preferences(false, budget).unwrap();
    let paused = wait_event(AnalysisJobOutcomeDto::Cancelled, &[0, 1]);
    assert_eq!(paused.job_id, branch.job_id);
    assert!(paused.frame.is_none());
    manager.follow_continuous_position(request(vec![]));
    std::thread::sleep(Duration::from_millis(100));
    assert!(manager.snapshot().selected_node_job.is_none());
    manager.set_continuous_preferences(true, budget).unwrap();
    let resumed = wait_event(AnalysisJobOutcomeDto::Progress, &[]);
    assert_eq!(resumed.run_id, run.run_id);
    assert_ne!(resumed.job_id, branch.job_id);
    assert_eq!(resumed.frame.as_ref().unwrap().turn, 0);
    manager.set_continuous_preferences(false, budget).unwrap();
    wait_event(AnalysisJobOutcomeDto::Cancelled, &[]);
    assert_eq!(document.serialize().unwrap(), before);
    let frozen = manager.diagnostic_snapshots().pop().unwrap();
    assert_eq!(frozen.run_id, run.run_id);
    assert!(!frozen.process_exited);
    assert!(!frozen.full_trace);
    assert!(frozen.records.iter().any(|record| record.source == "stdout"));
    assert!(frozen.retained_bytes <= 64 * 1024 && frozen.records.len() <= 256);
    let encoded = serde_json::to_string(&frozen).unwrap();
    for key in ["LIZZIEYZY_KATAGO_ENGINE", "LIZZIEYZY_KATAGO_MODEL", "LIZZIEYZY_KATAGO_CONFIG"] {
        assert!(!encoded.contains(&std::env::var(key).unwrap()));
    }
    assert!(matches!(manager.snapshot().lifecycle, ForegroundEngineLifecycleDto::Ready { run: current } if current.run_id == run.run_id));
    println!("GTP DIAGNOSTICS {encoded}");
    manager.teardown().unwrap();
    assert_eq!(serde_json::to_string(&frozen).unwrap(), encoded);
}

#[cfg(unix)]
mod controlled {
    use super::*;
    use std::{os::unix::fs::PermissionsExt, path::PathBuf};
    struct Rig {
        manager: ForegroundEngineManager,
        dir: PathBuf,
        run: String,
    }
    impl Rig {
        fn new(mode: &str) -> Self {
            let dir = std::env::temp_dir().join(format!("rules 空格 {}", uuid::Uuid::new_v4()));
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join("mode"), mode).unwrap();
            std::fs::write(dir.join("model"), "").unwrap();
            std::fs::write(dir.join("config"), "").unwrap();
            let executable = dir.join("engine");
            std::fs::write(
                &executable,
                format!(
                    "#!/bin/sh\nexec /usr/bin/python3 '{}/tests/fixtures/rules_process.py' \"$@\"\n",
                    env!("CARGO_MANIFEST_DIR")
                ),
            )
            .unwrap();
            std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o755)).unwrap();
            let catalog = Arc::new(InMemoryEngineProfileCatalog::new());
            catalog.upsert(SavedEngineProfile {
                profile_id: "gtp".into(),
                profile: EngineProfileDto {
                    name: "controlled GTP".into(),
                    program: executable.to_string_lossy().into(),
                    argv: vec![],
                    working_dir: Some(dir.to_string_lossy().into()),
                    adapter: EngineAdapterSettings::KataGoGtp(KataGoSettings {
                        model_path: Some("model".into()),
                        config_path: Some("config".into()),
                        max_visits: 4,
                    }),
                },
            });
            let manager = ForegroundEngineManager::new(catalog, ForegroundEngineConfig::for_tests());
            manager.start("gtp").unwrap();
            let run = ready(&manager);
            Self { manager, dir, run }
        }
        fn request(&self, deadline_ms: u32) -> GameMoveRequest {
            GameMoveRequest {
                identity: GameMoveRequestDto {
                    run_id: self.run.clone(),
                    generation: 7,
                    node_path: NodePath { indices: vec![] },
                    budget: ComputeBudgetDto {
                        deadline_ms,
                        max_visits: None,
                    },
                },
                position: sgf::CurrentSgfDocument::open("(;SZ[9]RU[Chinese]KM[7.5])")
                    .unwrap()
                    .exact_position(&NodePath { indices: vec![] })
                    .unwrap(),
            }
        }
        fn held(&self) {
            let deadline = Instant::now() + Duration::from_secs(3);
            while !self.dir.join("held").exists() {
                assert!(Instant::now() < deadline);
                std::thread::sleep(Duration::from_millis(2));
            }
        }
        fn no_restore_files(&self) {
            assert!(std::fs::read_dir(&self.dir).unwrap().all(|entry| !entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with(".lizzie-rules-")));
        }
    }
    impl Drop for Rig {
        fn drop(&mut self) {
            let _ = self.manager.teardown();
            let _ = std::fs::remove_dir_all(&self.dir);
        }
    }
    fn ready(manager: &ForegroundEngineManager) -> String {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            match manager.snapshot().lifecycle {
                ForegroundEngineLifecycleDto::Ready { run } => return run.run_id,
                ForegroundEngineLifecycleDto::Error { failure, .. } => panic!("{failure:?}"),
                _ => assert!(Instant::now() < deadline),
            }
            std::thread::sleep(Duration::from_millis(2));
        }
    }
    #[test]
    fn stream_frames_do_not_acknowledge_control_and_receipt_retires_on_navigation() {
        let rig = Rig::new("stream");
        let receipt = rig.manager.confirm_ordinary_rules(rig.request(2000)).unwrap();
        assert_eq!(receipt.position.komi, 7.5);
        assert_eq!(receipt.position.to_play, PlayerColor::Black);
        assert!(receipt.stones.is_empty());
        assert_eq!(receipt.true_final_move, None);
        rig.no_restore_files();
        rig.manager
            .invalidate_game_move_position(7, &NodePath { indices: vec![0] });
        assert_eq!(
            rig.manager.claim_ordinary_rules(&receipt).unwrap_err().kind,
            EngineFailureKind::Cancellation
        );
        assert_eq!(ready(&rig.manager), rig.run);
    }
    #[test]
    fn negative_mismatch_old_ack_exit_and_missing_ack_never_confirm() {
        for mode in [
            "negative",
            "wrong_rules",
            "old_ack",
            "exit",
            "stream_only",
            "send_failure",
        ] {
            let rig = Rig::new(mode);
            if mode == "send_failure" {
                rig.held();
            }
            let result = rig.manager.confirm_ordinary_rules(rig.request(250));
            assert!(result.is_err(), "{mode} confirmed");
            assert!(!matches!(
                rig.manager.snapshot().lifecycle,
                ForegroundEngineLifecycleDto::Ready { .. }
            ));
            rig.no_restore_files();
        }
    }
    #[test]
    fn late_reply_after_position_change_cannot_publish_or_authorize_new_run() {
        let rig = Rig::new("hold");
        let handle = rig.manager.start_ordinary_rules(rig.request(3000)).unwrap();
        rig.held();
        rig.manager
            .invalidate_game_move_position(8, &NodePath { indices: vec![] });
        std::fs::write(rig.dir.join("release"), "").unwrap();
        assert_eq!(handle.wait().unwrap_err().kind, EngineFailureKind::Cancellation);
        rig.no_restore_files();
        rig.manager.teardown().unwrap();
        std::fs::write(rig.dir.join("mode"), "stream").unwrap();
        rig.manager.start("gtp").unwrap();
        let replacement = ready(&rig.manager);
        assert_ne!(replacement, rig.run);
        let mut request = rig.request(2000);
        request.identity.run_id = replacement.clone();
        request.identity.generation = 8;
        let receipt = rig.manager.confirm_ordinary_rules(request).unwrap();
        assert_eq!(receipt.reader_id, replacement);
        assert_eq!(receipt.identity.generation, 8);
    }
    #[test]
    fn unsupported_projection_and_unreadable_restore_preserve_ready_run() {
        let rig = Rig::new("stream");
        let before = std::fs::read(rig.dir.join("trace")).unwrap();
        for sgf in [
            "(;SZ[9]RU[Chinese]AW[aa])",
            "(;SZ[9]RU[Chinese]AB[aa][bb]AW[cc])",
            "(;SZ[9]RU[Chinese];B[aa];AB[bb])",
        ] {
            let document = sgf::CurrentSgfDocument::open(sgf).unwrap();
            assert!(document
                .exact_position(&document.default_selected_path())
                .is_err());
        }
        assert_eq!(std::fs::read(rig.dir.join("trace")).unwrap(), before);
        assert_eq!(ready(&rig.manager), rig.run);
        // Remove the process cwd name while preserving its already-open process/resources.
        let renamed = rig.dir.with_extension("moved");
        std::fs::rename(&rig.dir, &renamed).unwrap();
        let result = rig.manager.confirm_ordinary_rules(rig.request(2000));
        std::fs::rename(&renamed, &rig.dir).unwrap();
        assert_eq!(result.unwrap_err().kind, EngineFailureKind::UnsupportedCapability);
        assert_eq!(ready(&rig.manager), rig.run);
        assert_eq!(std::fs::read(rig.dir.join("trace")).unwrap(), before);
    }

    fn selected(rig: &Rig, generation: u64, mode: AnalysisJobModeDto) -> SelectedNodeJobRequest {
        SelectedNodeJobRequest {
            run_id: rig.run.clone(), generation, node_path: NodePath { indices: vec![] }, mode,
            board_width: 9, board_height: 9, position_empty: true,
            exact_position: Ok(rig.request(2000).position),
            query: katago_protocol::analysis_query_from_position(9, 9, 7.5, &[], PlayerColor::Black,
                katago_protocol::AnalysisQueryOptions { id: "pending".into(), rules: "chinese".into(), turn: 0,
                    max_visits: Some(16), include_ownership: Some(true), include_policy: Some(true) }).unwrap(),
        }
    }

    fn event(receiver: &std::sync::mpsc::Receiver<ForegroundEngineEventDto>, outcome: AnalysisJobOutcomeDto) -> AnalysisJobEventDto {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            match receiver.recv_timeout(deadline.saturating_duration_since(Instant::now())).expect("analysis event deadline") {
                ForegroundEngineEventDto::Job { job } if job.outcome == outcome => return job,
                ForegroundEngineEventDto::Failure { failure } => panic!("{failure:?}"),
                _ => {},
            }
        }
    }

    #[test]
    fn rich_analysis_uses_exact_restore_same_run_and_preserves_absent_fields() {
        let rig = Rig::new("analysis");
        let events = rig.manager.subscribe();
        let started = rig.manager.start_selected_node_job(selected(&rig, 7, AnalysisJobModeDto::Finite)).unwrap();
        let completed = event(&events, AnalysisJobOutcomeDto::Completed);
        assert_eq!(completed.job_id, started.job_id);
        let frame = completed.frame.unwrap();
        assert_eq!(frame.winrate_black, 0.61);
        assert_eq!(frame.visits, 32);
        assert_eq!(frame.candidates[0].pv.len(), 2);
        assert_eq!(frame.candidates[0].score_mean_black, None);
        assert_eq!(frame.ownership, None);
        assert_eq!(frame.policy, None);
        assert_eq!(ready(&rig.manager), rig.run);
        let trace = std::fs::read_to_string(rig.dir.join("trace")).unwrap();
        assert!(trace.find("loadsgf").unwrap() < trace.find("kata-analyze").unwrap());
        assert!(trace.find("printsgf").unwrap() < trace.find("kata-analyze").unwrap());
        assert_eq!(trace.matches("kata-analyze").count(), 1);
        assert!(trace.contains(" stop\n"));
    }

    #[test]
    fn pause_waits_for_own_stop_ack_and_navigation_cannot_resume_or_publish_old_stream() {
        let rig = Rig::new("analysis_hold_stop");
        let events = rig.manager.subscribe();
        let budget = ContinuousAnalysisBudgetDto { continuous_time_limit_enabled: false, ..Default::default() };
        rig.manager.set_continuous_preferences(true, budget).unwrap();
        rig.manager.follow_continuous_position(selected(&rig, 7, AnalysisJobModeDto::Continuous));
        let first = event(&events, AnalysisJobOutcomeDto::Progress);
        assert_eq!(first.frame.as_ref().unwrap().winrate_black, 0.61);
        rig.manager.set_continuous_preferences(false, budget).unwrap();
        rig.held();
        assert_eq!(rig.manager.snapshot().selected_node_job.unwrap().state, AnalysisJobStateDto::Stopping);
        assert!(rig.manager.resume_continuous().is_err());
        let occupied = rig.manager.confirm_ordinary_rules(rig.request(2000)).unwrap_err();
        assert_eq!(occupied.kind, EngineFailureKind::Occupied);
        rig.manager.follow_continuous_position(selected(&rig, 8, AnalysisJobModeDto::Continuous));
        std::fs::write(rig.dir.join("release"), "").unwrap();
        let cancelled = event(&events, AnalysisJobOutcomeDto::Cancelled);
        assert_eq!(cancelled.job_id, first.job_id);
        assert!(cancelled.frame.is_none());
        std::thread::sleep(Duration::from_millis(100));
        assert!(rig.manager.snapshot().selected_node_job.is_none());
        assert_eq!(std::fs::read_to_string(rig.dir.join("trace")).unwrap().matches("kata-analyze").count(), 1);
        rig.manager.set_continuous_preferences(true, budget).unwrap();
        let next = event(&events, AnalysisJobOutcomeDto::Progress);
        assert_eq!(next.generation, 8);
        assert_ne!(next.job_id, first.job_id);
        assert_eq!(next.frame.unwrap().winrate_black, 0.62);
        assert_eq!(ready(&rig.manager), rig.run);
        rig.manager.set_continuous_preferences(false, budget).unwrap();
        event(&events, AnalysisJobOutcomeDto::Cancelled);
    }
    #[test]
    fn live_position_change_drains_then_starts_the_latest_target_on_same_run() {
        let rig = Rig::new("analysis");
        let events = rig.manager.subscribe();
        let budget = ContinuousAnalysisBudgetDto { continuous_time_limit_enabled: false, ..Default::default() };
        rig.manager.set_continuous_preferences(true, budget).unwrap();
        rig.manager.follow_continuous_position(selected(&rig, 7, AnalysisJobModeDto::Continuous));
        let first = event(&events, AnalysisJobOutcomeDto::Progress);
        rig.manager.follow_continuous_position(selected(&rig, 8, AnalysisJobModeDto::Continuous));
        let next = event(&events, AnalysisJobOutcomeDto::Progress);
        assert_ne!(next.job_id, first.job_id);
        assert_eq!(next.generation, 8);
        assert_eq!(next.run_id, first.run_id);
        assert_eq!(next.frame.unwrap().winrate_black, 0.62);
        rig.manager.set_continuous_preferences(false, budget).unwrap();
        event(&events, AnalysisJobOutcomeDto::Cancelled);
    }

    #[test]
    fn invalid_stream_requires_explicit_continue_after_restart_and_new_document() {
        let mut rig = Rig::new("analysis_invalid");
        let budget = ContinuousAnalysisBudgetDto { continuous_time_limit_enabled: false, ..Default::default() };
        rig.manager.set_continuous_preferences(true, budget).unwrap();
        rig.manager.follow_continuous_position(selected(&rig, 7, AnalysisJobModeDto::Continuous));
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if let ForegroundEngineLifecycleDto::Error { failure, .. } = rig.manager.snapshot().lifecycle {
                assert_eq!(failure.kind, EngineFailureKind::Protocol);
                break;
            }
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(5));
        }
        std::fs::write(rig.dir.join("mode"), "analysis").unwrap();
        rig.manager.restart().unwrap();
        let replacement = ready(&rig.manager);
        assert_ne!(replacement, rig.run);
        rig.run = replacement;
        rig.manager.follow_continuous_position(selected(&rig, 8, AnalysisJobModeDto::Continuous));
        std::thread::sleep(Duration::from_millis(100));
        assert!(rig.manager.snapshot().selected_node_job.is_none());
        let events = rig.manager.subscribe();
        rig.manager.authorize_continuous_start();
        let current = event(&events, AnalysisJobOutcomeDto::Progress);
        assert_eq!(current.run_id, rig.run);
        assert_eq!(current.generation, 8);
        rig.manager.set_continuous_preferences(false, budget).unwrap();
        event(&events, AnalysisJobOutcomeDto::Cancelled);
    }
}
