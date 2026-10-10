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
    let layer = std::path::PathBuf::from(std::env::var("LIZZIEYZY_KATAGO_WORKDIR").unwrap())
        .join(format!("promotion-layer-{}.cfg", uuid::Uuid::new_v4()));
    std::fs::write(&layer, "numSearchThreads=2\n").unwrap();
    let catalog = Arc::new(InMemoryEngineProfileCatalog::new());
    catalog.upsert(SavedEngineProfile {
        profile_id: "gtp".into(),
        profile: EngineProfileDto {
            name: "explicit KataGo GTP".into(),
            program: std::env::var("LIZZIEYZY_KATAGO_ENGINE").unwrap(),
            argv: vec![
                "-config".into(),
                layer.file_name().unwrap().to_string_lossy().into_owned(),
                "-override-config".into(),
                "numSearchThreads=1".into(),
            ],
            working_dir: Some(std::env::var("LIZZIEYZY_KATAGO_WORKDIR").unwrap()),
            adapter: EngineAdapterSettings::KataGoGtp(KataGoSettings {
                model_path: Some(std::env::var("LIZZIEYZY_KATAGO_MODEL").unwrap()),
                config_path: Some(std::env::var("LIZZIEYZY_KATAGO_CONFIG").unwrap()),
                max_visits: 4,
            }),
        },
    });
    catalog.upsert(SavedEngineProfile {
        profile_id: "prepared-gtp".into(),
        profile: catalog.get("gtp").unwrap().profile,
    });
    catalog.set_preload("prepared-gtp", true);
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
    let primary_id = run.run_id.clone();
    manager.prepare_preload("prepared-gtp").unwrap();
    let deadline = Instant::now() + Duration::from_secs(40);
    let prepared = loop {
        let preparation = manager.preload_snapshot().into_iter().next().unwrap();
        if preparation.phase == EnginePreloadPhaseDto::Ready {
            break preparation.run;
        }
        assert!(Instant::now() < deadline, "{preparation:?}");
        std::thread::sleep(Duration::from_millis(10));
    };
    assert_eq!(manager.last_primary_profile_id().as_deref(), Some("gtp"));
    manager.switch_to("prepared-gtp").unwrap();
    let run = match manager.snapshot().lifecycle {
        ForegroundEngineLifecycleDto::Ready { run } => run,
        other => panic!("{other:?}"),
    };
    assert_ne!(run.run_id, primary_id);
    assert_eq!(run.run_id, prepared.run_id);
    assert_eq!(manager.last_primary_profile_id().as_deref(), Some("prepared-gtp"));
    println!(
        "GTP PRELOAD primary={primary_id} promoted={} same_run=true",
        run.run_id
    );
    println!("RUN {}", serde_json::to_string(&run).unwrap());
    let events = manager.subscribe();
    assert!(!run.capability_snapshot.as_ref().unwrap().game_move);
    assert!(
        run.capability_snapshot
            .as_ref()
            .unwrap()
            .analysis
            .as_ref()
            .unwrap()
            .selected_node_analysis
    );
    assert!(manager
        .diagnostic_snapshots()
        .last()
        .unwrap()
        .records
        .iter()
        .any(|record| record.source == "startup-probe-stderr"));
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
        let query = katago_protocol::analysis_query_from_position(
            9,
            9,
            expected.komi,
            &selected_snapshot.position.stones,
            expected.to_play,
            katago_protocol::AnalysisQueryOptions {
                id: "pending".into(),
                rules: "chinese".into(),
                turn: expected.moves.len() as u32,
                max_visits: Some(16),
                include_ownership: Some(true),
                include_policy: Some(true),
            },
        )
        .unwrap();
        let started = manager
            .start_selected_node_job(SelectedNodeJobRequest {
                run_id: run.run_id.clone(),
                generation: generation as u64 + 1,
                node_path,
                mode: AnalysisJobModeDto::Finite,
                query,
                board_width: 9,
                board_height: 9,
                position_empty: selected_snapshot.position.stones.is_empty(),
                exact_position: Ok(document.exact_position(&selected_snapshot.path).unwrap()),
            })
            .unwrap();
        let until = Instant::now() + Duration::from_secs(40);
        loop {
            match events
                .recv_timeout(until.saturating_duration_since(Instant::now()))
                .unwrap()
            {
                ForegroundEngineEventDto::Job { job } if job.job_id == started.job_id => {
                    println!("MAIN_ANALYSIS {}", serde_json::to_string(&job).unwrap());
                    assert!(!matches!(
                        job.outcome,
                        AnalysisJobOutcomeDto::Failed | AnalysisJobOutcomeDto::Timeout
                    ));
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
                _ => {}
            }
        }
        assert_eq!(
            document.serialize().unwrap(),
            sgf::CurrentSgfDocument::open(text).unwrap().serialize().unwrap()
        );
    }
    let document =
        sgf::CurrentSgfDocument::open("(;SZ[9]RU[Chinese]KM[6.5]C[personal];B[dd](;W[ff])(;W[ee]))").unwrap();
    let before = document.serialize().unwrap();
    let request = |indices: Vec<u32>| {
        let node_path = NodePath { indices };
        let snapshot = document.snapshot(&node_path).unwrap();
        let position = document.exact_position(&node_path).unwrap();
        SelectedNodeJobRequest {
            run_id: run.run_id.clone(),
            generation: 77,
            node_path,
            mode: AnalysisJobModeDto::Continuous,
            board_width: 9,
            board_height: 9,
            position_empty: snapshot.position.stones.is_empty(),
            query: katago_protocol::analysis_query_from_position(
                9,
                9,
                6.5,
                &snapshot.position.stones,
                snapshot.position.to_play,
                katago_protocol::AnalysisQueryOptions {
                    id: "pending".into(),
                    rules: "chinese".into(),
                    turn: snapshot.position.move_number,
                    max_visits: None,
                    include_ownership: Some(true),
                    include_policy: Some(true),
                },
            )
            .unwrap(),
            exact_position: Ok(position),
        }
    };
    let wait_event = |outcome, path: &[u32]| {
        let deadline = Instant::now() + Duration::from_secs(30);
        loop {
            match events
                .recv_timeout(deadline.saturating_duration_since(Instant::now()))
                .unwrap()
            {
                ForegroundEngineEventDto::Job { job }
                    if job.outcome == outcome && job.node_path.indices == path =>
                {
                    println!("CONTINUOUS {}", serde_json::to_string(&job).unwrap());
                    break job;
                }
                ForegroundEngineEventDto::Failure { failure } => panic!("{failure:?}"),
                _ => {}
            }
        }
    };
    let budget = ContinuousAnalysisBudgetDto {
        continuous_time_limit_enabled: false,
        ..Default::default()
    };
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
    for key in [
        "LIZZIEYZY_KATAGO_ENGINE",
        "LIZZIEYZY_KATAGO_MODEL",
        "LIZZIEYZY_KATAGO_CONFIG",
    ] {
        assert!(!encoded.contains(&std::env::var(key).unwrap()));
    }
    assert!(
        matches!(manager.snapshot().lifecycle, ForegroundEngineLifecycleDto::Ready { run: current } if current.run_id == run.run_id)
    );
    println!("GTP DIAGNOSTICS {encoded}");
    manager.teardown().unwrap();
    assert_eq!(serde_json::to_string(&frozen).unwrap(), encoded);
    assert_eq!(std::fs::read_to_string(&layer).unwrap(), "numSearchThreads=2\n");
    std::fs::remove_file(layer).unwrap();
}

#[cfg(unix)]
mod controlled {
    use super::*;
    use std::{os::unix::fs::PermissionsExt, path::PathBuf};
    struct Rig {
        manager: ForegroundEngineManager,
        dir: PathBuf,
        run: String,
        catalog: Arc<InMemoryEngineProfileCatalog>,
    }
    impl Rig {
        fn new(mode: &str) -> Self {
            Self::with_pending_target(mode, false)
        }
        fn with_pending_target(mode: &str, pending: bool) -> Self {
            let dir = std::env::temp_dir().join(format!("rules 空格 {}", uuid::Uuid::new_v4()));
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join("mode"), mode).unwrap();
            std::fs::write(dir.join("model"), "").unwrap();
            std::fs::write(dir.join("config"), "numSearchThreads = 1\n").unwrap();
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
            let manager = ForegroundEngineManager::new(catalog.clone(), ForegroundEngineConfig::for_tests());
            let mut rig = Self {
                manager,
                dir,
                catalog,
                run: String::new(),
            };
            if pending {
                rig.manager
                    .set_continuous_preferences(
                        true,
                        ContinuousAnalysisBudgetDto {
                            continuous_time_limit_enabled: false,
                            ..Default::default()
                        },
                    )
                    .unwrap();
                rig.manager
                    .follow_continuous_position(selected(&rig, 7, AnalysisJobModeDto::Continuous));
            }
            rig.manager.start("gtp").unwrap();
            rig.run = ready(&rig.manager);
            rig
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

    #[test]
    fn thread_sources_follow_include_insertion_and_cli_without_rewriting_bytes() {
        let rig = Rig::new("analysis");
        rig.manager.teardown().unwrap();
        let cfg = "@include = 'base config.cfg'\nnumSearchThreads = \"3\" # parent after include wins\nnumSearchThreadsPerAnalysisThread = 7\nnumAnalysisThreads = 2\n";
        let included = "numSearchThreads = 2\n";
        std::fs::write(rig.dir.join("config"), cfg).unwrap();
        std::fs::write(rig.dir.join("base config.cfg"), included).unwrap();
        let mut saved = rig.catalog.get("gtp").unwrap();
        saved.profile.argv = vec![
            "-override-config".into(),
            "numSearchThreads=4,numAnalysisThreads=5".into(),
        ];
        rig.catalog.upsert(saved);
        rig.manager.start("gtp").unwrap();
        ready(&rig.manager);
        let sources = rig.manager.runtime_threads_snapshot().sources.unwrap();
        assert_eq!(sources.saved, Some(3));
        assert_eq!(sources.launch_override, Some(4));
        assert_eq!(sources.effective, Some(4));
        assert_eq!(sources.analysis_threads, Some(5));
        assert_eq!(
            sources
                .entries
                .iter()
                .map(|entry| entry.value)
                .collect::<Vec<_>>(),
            [Some(2), Some(3), Some(7), Some(2), Some(4), Some(5)]
        );
        assert_eq!(sources.entries[0].layer, "include");
        assert!(sources
            .entries
            .iter()
            .all(|entry| !entry.source.contains(&rig.dir.to_string_lossy().to_string())));
        let reset = rig
            .manager
            .runtime_threads(threads_request(
                &rig.manager,
                RuntimeThreadsActionDto::Reset,
                None,
            ))
            .unwrap();
        assert_eq!(reset.actual, Some(4));
        assert_eq!(std::fs::read_to_string(rig.dir.join("config")).unwrap(), cfg);
        assert_eq!(
            std::fs::read_to_string(rig.dir.join("base config.cfg")).unwrap(),
            included
        );
        std::fs::write(rig.dir.join("config"), "numSearchThreads=6\n").unwrap();
        let reset = rig
            .manager
            .runtime_threads(threads_request(
                &rig.manager,
                RuntimeThreadsActionDto::Reset,
                None,
            ))
            .unwrap();
        assert_eq!(reset.actual, Some(4));
    }

    fn threads_request(
        manager: &ForegroundEngineManager,
        action: RuntimeThreadsActionDto,
        value: Option<u32>,
    ) -> RuntimeThreadsRequestDto {
        let current = manager.runtime_threads_snapshot();
        RuntimeThreadsRequestDto {
            identity: RuntimeControlIdentityDto {
                run_id: current.run_id.unwrap(),
                profile_revision: current.profile_revision.unwrap(),
                request_id: uuid::Uuid::new_v4().to_string(),
            },
            action,
            value,
        }
    }

    fn wait_file(path: &std::path::Path) {
        let until = Instant::now() + Duration::from_secs(5);
        while !path.exists() {
            assert!(Instant::now() < until, "missing barrier {path:?}");
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    #[test]
    fn runtime_threads_apply_readback_reset_expiry_and_unchanged_config() {
        let rig = Rig::new("analysis");
        let bytes = std::fs::read(rig.dir.join("config")).unwrap();
        let request = threads_request(&rig.manager, RuntimeThreadsActionDto::Apply, Some(2));
        assert_eq!(rig.manager.runtime_threads_snapshot().actual, None);
        let result = rig.manager.runtime_threads(request.clone()).unwrap();
        assert_eq!(result.status, RuntimeThreadsStatusDto::Confirmed);
        assert_eq!(result.actual, Some(2));
        assert!(result.temporary);
        assert_eq!(result.sources.as_ref().unwrap().effective, Some(1));
        let reset = rig
            .manager
            .runtime_threads(threads_request(
                &rig.manager,
                RuntimeThreadsActionDto::Reset,
                None,
            ))
            .unwrap();
        assert_eq!(reset.actual, Some(1));
        assert!(!reset.temporary);
        for value in [0, 4097] {
            assert!(rig
                .manager
                .runtime_threads(threads_request(
                    &rig.manager,
                    RuntimeThreadsActionDto::Apply,
                    Some(value)
                ))
                .is_err());
        }
        assert_eq!(std::fs::read(rig.dir.join("config")).unwrap(), bytes);
        rig.manager.restart().unwrap();
        let replacement = ready(&rig.manager);
        assert_ne!(replacement, rig.run);
        assert_eq!(rig.manager.runtime_threads_snapshot().actual, None);
        assert!(rig.manager.runtime_threads(request).is_err());
        assert_eq!(
            std::fs::read_to_string(rig.dir.join("trace"))
                .unwrap()
                .matches("kata-set-param")
                .count(),
            2
        );
    }

    #[test]
    fn runtime_threads_errors_retain_last_valid_without_claiming_confirmation() {
        let rig = Rig::new("analysis");
        rig.manager
            .runtime_threads(threads_request(
                &rig.manager,
                RuntimeThreadsActionDto::Apply,
                Some(2),
            ))
            .unwrap();
        for mode in ["negative", "mismatch", "invalid"] {
            std::fs::write(rig.dir.join("thread-mode"), mode).unwrap();
            let result = rig
                .manager
                .runtime_threads(threads_request(
                    &rig.manager,
                    RuntimeThreadsActionDto::Apply,
                    Some(3),
                ))
                .unwrap();
            assert_eq!(result.status, RuntimeThreadsStatusDto::Failed, "{mode}");
            assert_eq!(result.actual, Some(2));
            assert_eq!(result.requested, Some(3));
            assert!(result.failure.is_some());
            assert_eq!(ready(&rig.manager), rig.run);
        }
    }

    #[test]
    fn runtime_control_waits_for_stop_ack_blocks_finite_and_preserves_new_pause_target() {
        let rig = Rig::new("analysis_hold_stop");
        let events = rig.manager.subscribe();
        let budget = ContinuousAnalysisBudgetDto {
            continuous_time_limit_enabled: false,
            ..Default::default()
        };
        rig.manager.set_continuous_preferences(true, budget).unwrap();
        rig.manager
            .follow_continuous_position(selected(&rig, 7, AnalysisJobModeDto::Continuous));
        event(&events, AnalysisJobOutcomeDto::Progress);
        let request = threads_request(&rig.manager, RuntimeThreadsActionDto::Apply, Some(2));
        let manager = rig.manager.clone();
        let worker = std::thread::spawn(move || manager.runtime_threads(request));
        wait_file(&rig.dir.join("held"));
        assert!(!std::fs::read_to_string(rig.dir.join("trace"))
            .unwrap()
            .contains("kata-set-param"));
        assert_eq!(
            rig.manager
                .start_selected_node_job(selected(&rig, 7, AnalysisJobModeDto::Finite))
                .unwrap_err()
                .kind,
            EngineFailureKind::Occupied
        );
        rig.manager.set_continuous_preferences(false, budget).unwrap();
        rig.manager
            .follow_continuous_position(selected(&rig, 8, AnalysisJobModeDto::Continuous));
        std::fs::write(rig.dir.join("release"), "").unwrap();
        assert_eq!(worker.join().unwrap().unwrap().actual, Some(2));
        assert_eq!(rig.manager.snapshot().continuous.enabled, Some(false));
        assert!(rig.manager.snapshot().selected_node_job.is_none());
        assert_eq!(
            std::fs::read_to_string(rig.dir.join("trace"))
                .unwrap()
                .matches("kata-analyze")
                .count(),
            1
        );
        rig.manager.set_continuous_preferences(true, budget).unwrap();
        assert_eq!(event(&events, AnalysisJobOutcomeDto::Progress).generation, 8);
    }

    #[test]
    fn runtime_threads_timeout_late_reply_and_new_run_cannot_confirm_old_request() {
        let rig = Rig::new("analysis");
        rig.manager
            .runtime_threads(threads_request(
                &rig.manager,
                RuntimeThreadsActionDto::Apply,
                Some(2),
            ))
            .unwrap();
        std::fs::write(rig.dir.join("thread-mode"), "timeout").unwrap();
        let result = rig
            .manager
            .runtime_threads(threads_request(
                &rig.manager,
                RuntimeThreadsActionDto::Apply,
                Some(3),
            ))
            .unwrap();
        assert_eq!(result.status, RuntimeThreadsStatusDto::Failed);
        assert_eq!(result.actual, Some(2));
        assert!(result.failure.as_ref().unwrap().contains("timed out"));
        std::fs::write(rig.dir.join("thread-release"), "").unwrap();
        std::fs::write(rig.dir.join("thread-mode"), "valid").unwrap();
        let read = rig
            .manager
            .runtime_threads(threads_request(&rig.manager, RuntimeThreadsActionDto::Read, None))
            .unwrap();
        assert_eq!(read.actual, Some(3));
        assert_eq!(read.status, RuntimeThreadsStatusDto::Confirmed);
        std::fs::remove_file(rig.dir.join("thread-release")).unwrap();
        std::fs::write(rig.dir.join("thread-mode"), "hold").unwrap();
        let request = threads_request(&rig.manager, RuntimeThreadsActionDto::Apply, Some(4));
        let manager = rig.manager.clone();
        let worker = std::thread::spawn(move || manager.runtime_threads(request));
        let until = Instant::now() + Duration::from_secs(5);
        while rig.manager.runtime_threads_snapshot().requested != Some(4) {
            assert!(Instant::now() < until);
            std::thread::sleep(Duration::from_millis(5));
        }
        rig.manager.restart().unwrap();
        ready(&rig.manager);
        assert!(worker.join().unwrap().is_err());
        assert_eq!(rig.manager.runtime_threads_snapshot().actual, None);
        assert!(!rig.manager.runtime_threads_snapshot().temporary);
    }

    fn pair_request(manager: &ForegroundEngineManager) -> RuntimeControlIdentityDto {
        let snapshot = manager.runtime_parameters_snapshot();
        RuntimeControlIdentityDto {
            run_id: snapshot.run_id.unwrap(),
            profile_revision: snapshot.profile_revision.unwrap(),
            request_id: uuid::Uuid::new_v4().to_string(),
        }
    }

    #[test]
    fn runtime_pair_publishes_both_numbered_values_in_either_order() {
        for mode in ["forward", "reverse"] {
            let rig = Rig::new("analysis");
            assert_eq!(
                rig.manager.runtime_parameters_snapshot().status,
                RuntimeParametersStatusDto::Unknown
            );
            assert_eq!(rig.manager.runtime_parameters_snapshot().last_valid, None);
            std::fs::write(rig.dir.join("pair-mode"), mode).unwrap();
            let result = rig
                .manager
                .read_runtime_parameters(pair_request(&rig.manager))
                .unwrap();
            assert_eq!(
                result.status,
                RuntimeParametersStatusDto::Confirmed,
                "{mode}: {result:?}"
            );
            assert_eq!(
                result.last_valid,
                Some(RuntimeParameterPairDto {
                    playout_doubling_advantage: -0.5,
                    analysis_wide_root_noise: 0.04
                })
            );
            assert_eq!(ready(&rig.manager), rig.run);
        }
    }

    fn pair_held(rig: &Rig) {
        let deadline = Instant::now() + Duration::from_secs(3);
        while !rig.dir.join("pair-held").exists() {
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(2));
        }
    }

    #[test]
    fn runtime_pair_invalid_halves_keep_last_valid_and_never_overwrite_thread_draft_state() {
        let rig = Rig::new("analysis");
        let valid = rig
            .manager
            .read_runtime_parameters(pair_request(&rig.manager))
            .unwrap();
        let thread = rig
            .manager
            .runtime_threads(threads_request(
                &rig.manager,
                RuntimeThreadsActionDto::Apply,
                Some(2),
            ))
            .unwrap();
        for mode in [
            "error_first",
            "error_second",
            "nan",
            "infinity",
            "pda_domain",
            "wrn_domain",
            "extra_value",
        ] {
            std::fs::write(rig.dir.join("pair-mode"), mode).unwrap();
            let failed = rig
                .manager
                .read_runtime_parameters(pair_request(&rig.manager))
                .unwrap();
            assert_eq!(
                failed.status,
                RuntimeParametersStatusDto::Failed,
                "{mode}: {failed:?}"
            );
            assert_eq!(failed.last_valid, valid.last_valid);
            assert!(failed.failure.is_some());
            assert_ne!(failed.request_id, valid.request_id);
            assert_eq!(rig.manager.runtime_threads_snapshot(), thread);
            assert_eq!(ready(&rig.manager), rig.run);
        }
    }

    #[test]
    fn runtime_pair_half_is_not_current_and_same_transaction_excludes_threads_preserves_pause() {
        let rig = Rig::new("analysis");
        let valid = rig
            .manager
            .read_runtime_parameters(pair_request(&rig.manager))
            .unwrap();
        std::fs::write(rig.dir.join("pair-mode"), "hold").unwrap();
        let manager = rig.manager.clone();
        let request = pair_request(&manager);
        let worker = std::thread::spawn(move || manager.read_runtime_parameters(request));
        pair_held(&rig);
        let pending = rig.manager.runtime_parameters_snapshot();
        assert_eq!(pending.status, RuntimeParametersStatusDto::Pending);
        assert_eq!(pending.last_valid, valid.last_valid);
        assert!(rig
            .manager
            .runtime_threads(threads_request(&rig.manager, RuntimeThreadsActionDto::Read, None))
            .is_err());
        assert!(rig
            .manager
            .read_runtime_parameters(pair_request(&rig.manager))
            .is_err());
        let budget = ContinuousAnalysisBudgetDto::default();
        rig.manager.set_continuous_preferences(false, budget).unwrap();
        std::fs::write(rig.dir.join("pair-release"), "").unwrap();
        let result = worker.join().unwrap().unwrap();
        assert_eq!(result.status, RuntimeParametersStatusDto::Confirmed);
        assert_eq!(result.last_valid.unwrap().playout_doubling_advantage, 1.5);
        assert_eq!(rig.manager.snapshot().continuous.enabled, Some(false));
        assert!(rig.manager.snapshot().selected_node_job.is_none());
        let thread = rig
            .manager
            .runtime_threads(threads_request(
                &rig.manager,
                RuntimeThreadsActionDto::Apply,
                Some(2),
            ))
            .unwrap();
        assert_eq!(thread.actual, Some(2));
    }

    #[test]
    fn runtime_pair_missing_old_and_stream_halves_timeout_without_cross_round_assembly() {
        for mode in ["missing", "old_ids", "stream_only", "timeout"] {
            let rig = Rig::new("analysis");
            let valid = rig
                .manager
                .read_runtime_parameters(pair_request(&rig.manager))
                .unwrap();
            std::fs::write(rig.dir.join("pair-mode"), mode).unwrap();
            let failed = rig
                .manager
                .read_runtime_parameters(pair_request(&rig.manager))
                .unwrap();
            assert_eq!(
                failed.status,
                RuntimeParametersStatusDto::Failed,
                "{mode}: {failed:?}"
            );
            assert_eq!(failed.last_valid, valid.last_valid);
            assert!(failed.failure.unwrap().contains("timed out"));
            // A late complete old response cannot satisfy this new round's IDs.
            std::fs::write(rig.dir.join("pair-release"), "").unwrap();
            std::fs::write(rig.dir.join("pair-mode"), "reverse").unwrap();
            let next = rig
                .manager
                .read_runtime_parameters(pair_request(&rig.manager))
                .unwrap();
            assert_eq!(next.status, RuntimeParametersStatusDto::Confirmed);
            assert_eq!(next.last_valid, valid.last_valid);
            assert_ne!(next.request_id, failed.request_id);
        }
    }

    #[test]
    fn runtime_pair_restart_switch_disconnect_retire_pending_reader_and_new_run_is_unknown() {
        for action in ["restart", "switch", "stop"] {
            let rig = Rig::new("analysis");
            std::fs::write(rig.dir.join("pair-mode"), "hold").unwrap();
            let manager = rig.manager.clone();
            let request = pair_request(&manager);
            let stale = request.clone();
            let worker = std::thread::spawn(move || manager.read_runtime_parameters(request));
            pair_held(&rig);
            match action {
                "restart" => {
                    rig.manager.restart().unwrap();
                    ready(&rig.manager);
                }
                "switch" => {
                    let mut profile = rig.catalog.get("gtp").unwrap();
                    profile.profile_id = "other".into();
                    rig.catalog.upsert(profile);
                    rig.manager.switch_to("other").unwrap();
                    ready(&rig.manager);
                }
                _ => rig.manager.teardown().unwrap(),
            }
            // Switch keeps A visible while B starts: retirement can return A's
            // explicit Failed snapshot before B promotion, or an expired error.
            if let Ok(retired) = worker.join().unwrap() {
                assert_eq!(
                    retired.status,
                    RuntimeParametersStatusDto::Failed,
                    "{action}: {retired:?}"
                );
                assert_eq!(retired.last_valid, None);
            }
            assert!(rig.manager.read_runtime_parameters(stale).is_err());
            let state = rig.manager.runtime_parameters_snapshot();
            assert_ne!(state.run_id.as_deref(), Some(rig.run.as_str()));
            assert_eq!(state.status, RuntimeParametersStatusDto::Unknown);
            assert_eq!(state.last_valid, None);
        }
    }

    #[test]
    fn runtime_pair_wrong_framing_cannot_publish_and_finite_work_is_not_cancelled() {
        let rig = Rig::new("analysis");
        std::fs::write(rig.dir.join("pair-mode"), "wrong_order").unwrap();
        let result = rig.manager.read_runtime_parameters(pair_request(&rig.manager));
        assert!(result.is_err() || result.unwrap().status == RuntimeParametersStatusDto::Failed);
        assert_ne!(
            rig.manager.runtime_parameters_snapshot().status,
            RuntimeParametersStatusDto::Confirmed
        );
        let rig = Rig::new("analysis_hold_stop");
        let events = rig.manager.subscribe();
        let job = rig
            .manager
            .start_selected_node_job(selected(&rig, 7, AnalysisJobModeDto::Finite))
            .unwrap();
        event(&events, AnalysisJobOutcomeDto::Progress);
        assert!(rig
            .manager
            .read_runtime_parameters(pair_request(&rig.manager))
            .is_err());
        assert_eq!(
            rig.manager.snapshot().selected_node_job.unwrap().job_id,
            job.job_id
        );
        std::fs::write(rig.dir.join("release"), "").unwrap();
    }

    #[test]
    fn runtime_pair_refuses_match_and_exclusive_position_without_retiring_them() {
        let rig = Rig::new("analysis");
        rig.manager.reserve_match("pair-test-match").unwrap();
        assert!(rig
            .manager
            .read_runtime_parameters(pair_request(&rig.manager))
            .is_err());
        assert_eq!(
            rig.manager.match_reservation_owner().as_deref(),
            Some("pair-test-match")
        );
        rig.manager.abort_reserved_match("pair-test-match").unwrap();
        assert_eq!(ready(&rig.manager), rig.run);
        let rig = Rig::new("hold");
        let position = rig.manager.start_ordinary_rules(rig.request(3000)).unwrap();
        rig.held();
        assert!(rig
            .manager
            .read_runtime_parameters(pair_request(&rig.manager))
            .is_err());
        std::fs::write(rig.dir.join("release"), "").unwrap();
        assert_eq!(position.wait().unwrap().identity.run_id, rig.run);
    }

    #[test]
    fn runtime_pair_holds_analysis_across_halves_then_reconciles_latest_target() {
        let rig = Rig::new("analysis");
        let events = rig.manager.subscribe();
        let budget = ContinuousAnalysisBudgetDto {
            continuous_time_limit_enabled: false,
            ..Default::default()
        };
        rig.manager.set_continuous_preferences(true, budget).unwrap();
        rig.manager
            .follow_continuous_position(selected(&rig, 7, AnalysisJobModeDto::Continuous));
        let first = event(&events, AnalysisJobOutcomeDto::Progress);
        std::fs::write(rig.dir.join("pair-mode"), "hold").unwrap();
        let manager = rig.manager.clone();
        let request = pair_request(&manager);
        let worker = std::thread::spawn(move || manager.read_runtime_parameters(request));
        pair_held(&rig);
        rig.manager
            .follow_continuous_position(selected(&rig, 8, AnalysisJobModeDto::Continuous));
        assert!(rig.manager.snapshot().selected_node_job.is_none());
        assert_eq!(
            std::fs::read_to_string(rig.dir.join("trace"))
                .unwrap()
                .matches("kata-analyze")
                .count(),
            1
        );
        assert!(rig
            .manager
            .runtime_threads(threads_request(
                &rig.manager,
                RuntimeThreadsActionDto::Apply,
                Some(2)
            ))
            .is_err());
        std::fs::write(rig.dir.join("pair-release"), "").unwrap();
        assert_eq!(
            worker.join().unwrap().unwrap().status,
            RuntimeParametersStatusDto::Confirmed
        );
        let resumed = event(&events, AnalysisJobOutcomeDto::Progress);
        assert_eq!(resumed.generation, 8);
        assert_eq!(resumed.run_id, first.run_id);
        assert_ne!(resumed.job_id, first.job_id);
        rig.manager.set_continuous_preferences(false, budget).unwrap();
        event(&events, AnalysisJobOutcomeDto::Cancelled);
    }

    #[test]
    fn runtime_pair_reader_failure_retires_both_replies_and_restart_read_preserves_safety_hold() {
        let rig = Rig::new("analysis");
        let valid = rig
            .manager
            .read_runtime_parameters(pair_request(&rig.manager))
            .unwrap();
        let events = rig.manager.subscribe();
        rig.manager
            .set_continuous_preferences(
                true,
                ContinuousAnalysisBudgetDto {
                    continuous_time_limit_enabled: false,
                    ..Default::default()
                },
            )
            .unwrap();
        rig.manager
            .follow_continuous_position(selected(&rig, 7, AnalysisJobModeDto::Continuous));
        event(&events, AnalysisJobOutcomeDto::Progress);
        std::fs::write(rig.dir.join("pair-mode"), "closed_stdout").unwrap();
        let request = pair_request(&rig.manager);
        let stale = request.clone();
        let manager = rig.manager.clone();
        let worker = std::thread::spawn(move || manager.read_runtime_parameters(request));
        pair_held(&rig);
        assert_eq!(
            rig.manager.runtime_parameters_snapshot().status,
            RuntimeParametersStatusDto::Pending
        );
        assert_eq!(
            rig.manager.runtime_parameters_snapshot().last_valid,
            valid.last_valid
        );
        assert!(rig.manager.snapshot().selected_node_job.is_none());
        std::fs::write(rig.dir.join("pair-release"), "").unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        while !matches!(
            rig.manager.snapshot().lifecycle,
            ForegroundEngineLifecycleDto::Error { .. }
        ) {
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(5));
        }
        assert!(worker.join().unwrap().is_err());
        std::fs::write(rig.dir.join("pair-mode"), "reverse").unwrap();
        rig.manager.restart().unwrap();
        assert_ne!(ready(&rig.manager), rig.run);
        assert_eq!(rig.manager.runtime_parameters_snapshot().last_valid, None);
        assert!(rig.manager.read_runtime_parameters(stale).is_err());
        let fresh = rig
            .manager
            .read_runtime_parameters(pair_request(&rig.manager))
            .unwrap();
        assert_eq!(fresh.status, RuntimeParametersStatusDto::Confirmed);
        assert_eq!(fresh.last_valid, valid.last_valid);
        assert_eq!(
            rig.manager.snapshot().continuous.phase,
            ContinuousAnalysisPhaseDto::SafetyHold
        );
        assert!(rig.manager.snapshot().selected_node_job.is_none());
    }

    fn selected(rig: &Rig, generation: u64, mode: AnalysisJobModeDto) -> SelectedNodeJobRequest {
        SelectedNodeJobRequest {
            run_id: rig.run.clone(),
            generation,
            node_path: NodePath { indices: vec![] },
            mode,
            board_width: 9,
            board_height: 9,
            position_empty: true,
            exact_position: Ok(rig.request(2000).position),
            query: katago_protocol::analysis_query_from_position(
                9,
                9,
                7.5,
                &[],
                PlayerColor::Black,
                katago_protocol::AnalysisQueryOptions {
                    id: "pending".into(),
                    rules: "chinese".into(),
                    turn: 0,
                    max_visits: Some(16),
                    include_ownership: Some(true),
                    include_policy: Some(true),
                },
            )
            .unwrap(),
        }
    }

    fn event(
        receiver: &std::sync::mpsc::Receiver<ForegroundEngineEventDto>,
        outcome: AnalysisJobOutcomeDto,
    ) -> AnalysisJobEventDto {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            match receiver
                .recv_timeout(deadline.saturating_duration_since(Instant::now()))
                .expect("analysis event deadline")
            {
                ForegroundEngineEventDto::Job { job } if job.outcome == outcome => return job,
                ForegroundEngineEventDto::Failure { failure } => panic!("{failure:?}"),
                _ => {}
            }
        }
    }

    #[test]
    fn rich_analysis_uses_exact_restore_same_run_and_preserves_absent_fields() {
        let rig = Rig::new("analysis");
        let events = rig.manager.subscribe();
        let started = rig
            .manager
            .start_selected_node_job(selected(&rig, 7, AnalysisJobModeDto::Finite))
            .unwrap();
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
        let budget = ContinuousAnalysisBudgetDto {
            continuous_time_limit_enabled: false,
            ..Default::default()
        };
        rig.manager.set_continuous_preferences(true, budget).unwrap();
        rig.manager
            .follow_continuous_position(selected(&rig, 7, AnalysisJobModeDto::Continuous));
        let first = event(&events, AnalysisJobOutcomeDto::Progress);
        assert_eq!(first.frame.as_ref().unwrap().winrate_black, 0.61);
        rig.manager.set_continuous_preferences(false, budget).unwrap();
        rig.held();
        assert_eq!(
            rig.manager.snapshot().selected_node_job.unwrap().state,
            AnalysisJobStateDto::Stopping
        );
        assert!(rig.manager.resume_continuous().is_err());
        let occupied = rig.manager.confirm_ordinary_rules(rig.request(2000)).unwrap_err();
        assert_eq!(occupied.kind, EngineFailureKind::Occupied);
        rig.manager
            .follow_continuous_position(selected(&rig, 8, AnalysisJobModeDto::Continuous));
        std::fs::write(rig.dir.join("release"), "").unwrap();
        let cancelled = event(&events, AnalysisJobOutcomeDto::Cancelled);
        assert_eq!(cancelled.job_id, first.job_id);
        assert!(cancelled.frame.is_none());
        std::thread::sleep(Duration::from_millis(100));
        assert!(rig.manager.snapshot().selected_node_job.is_none());
        assert_eq!(
            std::fs::read_to_string(rig.dir.join("trace"))
                .unwrap()
                .matches("kata-analyze")
                .count(),
            1
        );
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
    fn ready_gtp_reader_starts_the_existing_enabled_target() {
        let rig = Rig::with_pending_target("analysis", true);
        let events = rig.manager.subscribe();
        let frame = event(&events, AnalysisJobOutcomeDto::Progress);
        assert_eq!(frame.run_id, rig.run);
        assert_eq!(frame.generation, 7);
        assert_eq!(frame.mode, AnalysisJobModeDto::Continuous);
        rig.manager
            .set_continuous_preferences(false, ContinuousAnalysisBudgetDto::default())
            .unwrap();
        event(&events, AnalysisJobOutcomeDto::Cancelled);
    }

    #[test]
    fn live_position_change_drains_then_starts_the_latest_target_on_same_run() {
        let rig = Rig::new("analysis");
        let events = rig.manager.subscribe();
        let budget = ContinuousAnalysisBudgetDto {
            continuous_time_limit_enabled: false,
            ..Default::default()
        };
        rig.manager.set_continuous_preferences(true, budget).unwrap();
        rig.manager
            .follow_continuous_position(selected(&rig, 7, AnalysisJobModeDto::Continuous));
        let first = event(&events, AnalysisJobOutcomeDto::Progress);
        rig.manager
            .follow_continuous_position(selected(&rig, 8, AnalysisJobModeDto::Continuous));
        let next = event(&events, AnalysisJobOutcomeDto::Progress);
        assert_ne!(next.job_id, first.job_id);
        assert_eq!(next.generation, 8);
        assert_eq!(next.run_id, first.run_id);
        assert_eq!(next.frame.unwrap().winrate_black, 0.62);
        rig.manager.set_continuous_preferences(false, budget).unwrap();
        event(&events, AnalysisJobOutcomeDto::Cancelled);
    }

    #[test]
    fn active_gtp_reader_failure_retires_job_before_document_departure() {
        let rig = Rig::new("analysis_closed_stdout");
        let events = rig.manager.subscribe();
        rig.manager
            .set_continuous_preferences(
                true,
                ContinuousAnalysisBudgetDto {
                    continuous_time_limit_enabled: false,
                    ..Default::default()
                },
            )
            .unwrap();
        rig.manager
            .follow_continuous_position(selected(&rig, 7, AnalysisJobModeDto::Continuous));
        let first = event(&events, AnalysisJobOutcomeDto::Progress);
        rig.held();
        let request = threads_request(&rig.manager, RuntimeThreadsActionDto::Apply, Some(2));
        let stale_request = request.clone();
        let control_manager = rig.manager.clone();
        let control = std::thread::spawn(move || control_manager.runtime_threads(request));
        let pending_deadline = Instant::now() + Duration::from_secs(3);
        while rig.manager.runtime_threads_snapshot().status != RuntimeThreadsStatusDto::Pending {
            assert!(Instant::now() < pending_deadline);
            std::thread::sleep(Duration::from_millis(2));
        }
        std::fs::write(rig.dir.join("release"), "").unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            let snapshot = rig.manager.snapshot();
            if let ForegroundEngineLifecycleDto::Error { run, .. } = snapshot.lifecycle {
                assert_eq!(run.run_id, first.run_id);
                assert!(
                    snapshot.selected_node_job.is_none(),
                    "failed reader left a live departure job"
                );
                break;
            }
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(5));
        }
        let failed = event(&events, AnalysisJobOutcomeDto::Failed);
        assert_eq!(failed.job_id, first.job_id);
        assert_eq!(failed.run_id, first.run_id);
        assert!(control.join().unwrap().is_err());
        assert_eq!(rig.manager.runtime_threads_snapshot().actual, None);
        assert_eq!(
            rig.manager.snapshot().continuous.phase,
            ContinuousAnalysisPhaseDto::Error
        );
        std::fs::write(rig.dir.join("mode"), "analysis").unwrap();
        rig.manager.restart().unwrap();
        let replacement = ready(&rig.manager);
        assert_ne!(replacement, first.run_id);
        assert!(rig.manager.runtime_threads(stale_request).is_err());
        let read = rig
            .manager
            .runtime_threads(threads_request(&rig.manager, RuntimeThreadsActionDto::Read, None))
            .unwrap();
        assert_eq!(read.status, RuntimeThreadsStatusDto::Confirmed);
        assert_eq!(read.actual, Some(1));
        assert_eq!(
            rig.manager.snapshot().continuous.phase,
            ContinuousAnalysisPhaseDto::SafetyHold
        );
        assert!(rig.manager.snapshot().selected_node_job.is_none());
    }

    #[test]
    fn invalid_stream_requires_explicit_continue_after_restart_and_new_document() {
        let mut rig = Rig::new("analysis_invalid");
        let budget = ContinuousAnalysisBudgetDto {
            continuous_time_limit_enabled: false,
            ..Default::default()
        };
        rig.manager.set_continuous_preferences(true, budget).unwrap();
        rig.manager
            .follow_continuous_position(selected(&rig, 7, AnalysisJobModeDto::Continuous));
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if let ForegroundEngineLifecycleDto::Error { failure, .. } = rig.manager.snapshot().lifecycle {
                assert_eq!(failure.kind, EngineFailureKind::Protocol);
                break;
            }
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(5));
        }
        rig.manager.begin_continuous_departure();
        rig.manager.clear_continuous_position();
        rig.manager
            .follow_continuous_position(selected(&rig, 8, AnalysisJobModeDto::Continuous));
        rig.manager.finish_continuous_departure(true);
        std::fs::write(rig.dir.join("mode"), "analysis").unwrap();
        rig.manager.restart().unwrap();
        let replacement = ready(&rig.manager);
        assert_ne!(replacement, rig.run);
        rig.run = replacement;
        rig.manager
            .follow_continuous_position(selected(&rig, 8, AnalysisJobModeDto::Continuous));
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
