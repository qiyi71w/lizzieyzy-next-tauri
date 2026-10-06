#![cfg(unix)]
use app_model::*;
use engine_manager::*;
use sgf::CurrentSgfDocument;
use std::os::unix::fs::PermissionsExt;
use std::{
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

const GNU_ARGS: [&str; 9] = [
    "--mode",
    "gtp",
    "--chinese-rules",
    "--positional-superko",
    "--forbid-suicide",
    "--level",
    "1",
    "--seed",
    "1",
];
struct Rig {
    manager: ForegroundEngineManager,
    catalog: Arc<InMemoryEngineProfileCatalog>,
    dir: PathBuf,
    run: String,
    kata: bool,
}
impl Rig {
    fn new(kata: bool, mode: &str) -> Self {
        Self::with_cleanup_timeout(kata, mode, Duration::from_millis(150))
    }
    fn with_cleanup_timeout(kata: bool, mode: &str, stop_drain_timeout: Duration) -> Self {
        let dir = std::env::temp_dir().join(format!("r8-move-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("mode"), mode).unwrap();
        std::fs::write(dir.join("model"), "").unwrap();
        std::fs::write(dir.join("config"), "").unwrap();
        let executable = dir.join("engine");
        std::fs::write(
            &executable,
            format!(
                "#!/bin/sh\nexec /usr/bin/python3 '{}/tests/fixtures/game_move_process.py' \"$@\"\n",
                env!("CARGO_MANIFEST_DIR")
            ),
        )
        .unwrap();
        std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o755)).unwrap();
        let adapter = if kata {
            EngineAdapterSettings::KataGoAnalysis(KataGoSettings {
                model_path: Some(dir.join("model").to_string_lossy().into()),
                config_path: Some(dir.join("config").to_string_lossy().into()),
                max_visits: 8,
            })
        } else {
            EngineAdapterSettings::GenericGtp(GenericGtpSettings {})
        };
        let catalog = Arc::new(InMemoryEngineProfileCatalog::new());
        catalog.upsert(SavedEngineProfile {
            profile_id: "engine".into(),
            profile: EngineProfileDto {
                name: "pipe fixture".into(),
                program: executable.to_string_lossy().into(),
                argv: if kata {
                    vec![]
                } else {
                    GNU_ARGS.map(String::from).into()
                },
                working_dir: Some(dir.to_string_lossy().into()),
                adapter,
            },
        });
        let mut second = catalog.get("engine").unwrap();
        second.profile_id = "second".into();
        catalog.upsert(second);
        let manager = ForegroundEngineManager::new(
            catalog.clone(),
            ForegroundEngineConfig {
                stop_drain_timeout,
                ..ForegroundEngineConfig::for_tests()
            },
        );
        manager.start("engine").unwrap();
        let deadline = Instant::now() + Duration::from_secs(3);
        let run = loop {
            match manager.snapshot().lifecycle {
                ForegroundEngineLifecycleDto::Ready { run } => break run.run_id,
                ForegroundEngineLifecycleDto::Error { failure, .. } => panic!("{failure:?}"),
                _ => assert!(Instant::now() < deadline, "readiness timed out"),
            }
            std::thread::sleep(Duration::from_millis(2));
        };
        Self {
            manager,
            catalog,
            dir,
            run,
            kata,
        }
    }
    fn request(&self, deadline_ms: u32) -> GameMoveRequest {
        let document = CurrentSgfDocument::open("(;SZ[5]RU[Chinese-KGS]KM[6.5];B[aa];W[])").unwrap();
        GameMoveRequest {
            identity: GameMoveRequestDto {
                run_id: self.run.clone(),
                generation: 17,
                node_path: NodePath { indices: vec![0, 0] },
                budget: ComputeBudgetDto {
                    deadline_ms,
                    max_visits: self.kata.then_some(8),
                },
            },
            position: document
                .exact_position(&NodePath { indices: vec![0, 0] })
                .unwrap(),
        }
    }
    fn trace(&self) -> String {
        std::fs::read_to_string(self.dir.join("trace")).unwrap()
    }
    fn wait_for(&self, needle: &str) {
        let deadline = Instant::now() + Duration::from_secs(2);
        while !self.trace().contains(needle) {
            assert!(Instant::now() < deadline, "missing {needle} in {}", self.trace());
            std::thread::sleep(Duration::from_millis(2));
        }
    }
    fn reaped(&self) {
        let pid = std::fs::read_to_string(self.dir.join("pid")).unwrap();
        assert!(
            !std::path::Path::new(&format!("/proc/{pid}")).exists(),
            "child {pid} survives"
        );
    }
}
impl Drop for Rig {
    fn drop(&mut self) {
        let _ = self.manager.teardown();
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

#[test]
fn exact_pipe_move_returns_without_mutating_position_and_replays_full_history_each_time() {
    for kata in [false, true] {
        let rig = Rig::new(kata, "normal");
        for _ in 0..2 {
            let request = rig.request(4000);
            let before = request.position.dto().clone();
            let result = rig
                .manager
                .start_game_move(request.clone())
                .unwrap()
                .wait()
                .unwrap();
            assert_eq!(
                result.result,
                GameMoveDto::Move {
                    vertex: MoveVertex::Point(PointDto { x: 3, y: 1 })
                }
            );
            assert_eq!(
                (result.run_id, result.generation, result.node_path),
                (rig.run.clone(), 17, NodePath { indices: vec![0, 0] })
            );
            assert_eq!(result.engine_time_mapped, !kata);
            assert_eq!(request.position.dto(), &before);
        }
        let trace = rig.trace();
        if kata {
            let request: serde_json::Value = serde_json::from_str(trace.lines().last().unwrap()).unwrap();
            assert_eq!(request["moves"], serde_json::json!([["B", "A5"], ["W", "pass"]]));
            assert_eq!(request["initialStones"], serde_json::json!([]));
            assert_eq!(request["rules"]["ko"], "POSITIONAL");
            assert_eq!(request["initialPlayer"], "B");
        } else {
            assert_eq!(trace.matches("boardsize 5").count(), 2);
            assert_eq!(trace.matches("play B A5").count(), 2);
            assert_eq!(trace.matches("play W pass").count(), 2);
            assert!(!trace.contains("play B D4"));
        }
        assert!(matches!(
            rig.manager.snapshot().lifecycle,
            ForegroundEngineLifecycleDto::Ready { .. }
        ));
    }
}

#[test]
fn unsupported_budget_position_and_identity_are_pure_admission_failures() {
    for (kata, mode) in [(false, "normal"), (false, "identity"), (true, "normal")] {
        let rig = Rig::new(kata, mode);
        let before = rig.trace();
        let mut request = rig.request(3000);
        request.identity.budget.max_visits = if kata { None } else { Some(1) };
        assert_eq!(
            rig.manager.start_game_move(request).err().unwrap().kind,
            EngineFailureKind::UnsupportedCapability
        );
        let mut request = rig.request(3000);
        request.identity.budget.deadline_ms = 0;
        assert_eq!(
            rig.manager.start_game_move(request).err().unwrap().kind,
            EngineFailureKind::UnsupportedCapability
        );
        if !kata {
            let mut request = rig.request(3000);
            request.position = CurrentSgfDocument::open("(;SZ[5]RU[Chinese])")
                .unwrap()
                .exact_position(&NodePath::default())
                .unwrap();
            assert_eq!(
                rig.manager.start_game_move(request).err().unwrap().kind,
                EngineFailureKind::UnsupportedCapability
            );
        }
        assert_eq!(rig.trace(), before);
        assert!(rig.manager.snapshot().game_move_job.is_none());
    }
}

#[test]
fn gtp_time_mapping_is_pairwise_and_uses_remaining_total_budget() {
    for mode in ["no_time_left", "no_time_settings", "shrinking"] {
        let rig = Rig::new(false, mode);
        let result = rig
            .manager
            .start_game_move(rig.request(2200))
            .unwrap()
            .wait()
            .unwrap();
        assert_eq!(result.engine_time_mapped, mode == "shrinking");
        let trace = rig.trace();
        if mode == "shrinking" {
            assert!(trace.contains("time_settings 0 2 1"), "{trace}");
            assert!(trace.contains("time_left B 1 1"), "{trace}");
        } else {
            assert!(!trace.contains(" time_settings ") && !trace.contains(" time_left "));
        }
    }
    let rig = Rig::new(false, "slow_sync");
    let started = Instant::now();
    assert_eq!(
        rig.manager
            .start_game_move(rig.request(1250))
            .unwrap()
            .wait()
            .unwrap_err()
            .kind,
        EngineFailureKind::Timeout
    );
    assert!(started.elapsed() < Duration::from_secs(2));
    assert!(!rig.trace().contains("genmove B"));
    assert!(matches!(
        rig.manager.snapshot().lifecycle,
        ForegroundEngineLifecycleDto::Error { .. }
    ));
    rig.reaped();
}

#[test]
fn typed_results_and_invalid_completed_candidates_are_not_guessed() {
    for (mode, expected) in [
        (
            "pass",
            GameMoveDto::Move {
                vertex: MoveVertex::Pass,
            },
        ),
        ("resign", GameMoveDto::Resign),
    ] {
        let rig = Rig::new(false, mode);
        assert_eq!(
            rig.manager
                .start_game_move(rig.request(3000))
                .unwrap()
                .wait()
                .unwrap()
                .result,
            expected
        );
    }
    for (kata, modes) in [
        (false, &["occupied", "malformed"][..]),
        (
            true,
            &[
                "duplicate",
                "missing",
                "incomplete",
                "wrongturn",
                "occupied",
                "resign",
                "warning",
            ][..],
        ),
    ] {
        for mode in modes {
            let rig = Rig::new(kata, mode);
            assert_eq!(
                rig.manager
                    .start_game_move(rig.request(3000))
                    .unwrap()
                    .wait()
                    .unwrap_err()
                    .kind,
                EngineFailureKind::Protocol,
                "{kata} {mode}"
            );
            if kata && *mode == "warning" {
                rig.wait_for("terminateId");
                assert!(matches!(rig.manager.snapshot().lifecycle,
                    ForegroundEngineLifecycleDto::Ready { run } if run.run_id == rig.run));
            }
        }
    }
}

#[test]
fn rejected_katago_query_keeps_the_same_run_available_for_the_next_move() {
    for mode in ["error", "errors"] {
        let rig = Rig::new(true, mode);
        let failure = rig
            .manager
            .start_game_move(rig.request(3000))
            .unwrap()
            .wait()
            .unwrap_err();
        assert_eq!(failure.kind, EngineFailureKind::Protocol);
        assert!(
            matches!(rig.manager.snapshot().lifecycle,
            ForegroundEngineLifecycleDto::Ready { run } if run.run_id == rig.run),
            "{mode}"
        );
        assert!(!rig.trace().contains("terminateId"));
        let result = rig
            .manager
            .start_game_move(rig.request(3000))
            .unwrap()
            .wait()
            .unwrap();
        assert_eq!(result.run_id, rig.run);
        assert_eq!(
            result.result,
            GameMoveDto::Move {
                vertex: MoveVertex::Point(PointDto { x: 3, y: 1 }),
            }
        );
    }
}

#[test]
fn move_slot_excludes_lanes_until_cancel_cleanup_and_preserves_continuous_intent() {
    for kata in [false, true] {
        let rig = Rig::new(kata, "hold");
        let active = rig.manager.start_game_move(rig.request(5000)).unwrap();
        rig.manager
            .set_continuous_preferences(true, ContinuousAnalysisBudgetDto::default())
            .unwrap();
        rig.wait_for(if kata { "\"maxVisits\":8" } else { "genmove B" });
        assert_eq!(
            rig.manager.start_game_move(rig.request(5000)).err().unwrap().kind,
            EngineFailureKind::Occupied
        );
        assert_eq!(
            rig.manager
                .register_job(
                    &rig.run,
                    AnalysisJobLane::WholeGame,
                    Arc::new(AnalysisCancelToken::default())
                )
                .unwrap_err()
                .kind,
            EngineFailureKind::Occupied
        );
        rig.manager
            .cancel_game_move(&active.identity.run_id, &active.identity.job_id)
            .unwrap();
        assert!(rig.manager.snapshot().game_move_job.is_some());
        assert_eq!(active.wait().unwrap_err().kind, EngineFailureKind::Cancellation);
        let snapshot = rig.manager.snapshot();
        assert_eq!(snapshot.continuous.enabled, Some(true));
        assert!(snapshot.game_move_job.is_none());
        if kata {
            assert!(matches!(
                snapshot.lifecycle,
                ForegroundEngineLifecycleDto::Ready { .. }
            ));
        } else {
            assert!(matches!(
                snapshot.lifecycle,
                ForegroundEngineLifecycleDto::NoEngine { .. }
            ));
            rig.reaped();
        }
    }
}

#[test]
fn timeout_and_position_retirement_fence_late_responses() {
    for (kata, invalidate) in [(false, false), (true, false), (false, true), (true, true)] {
        let rig = Rig::new(kata, "hold");
        let active = rig
            .manager
            .start_game_move(rig.request(if invalidate || !kata { 1500 } else { 150 }))
            .unwrap();
        if invalidate {
            rig.manager
                .invalidate_game_move_position(18, &NodePath { indices: vec![0, 0] });
        }
        assert_eq!(
            active.wait().unwrap_err().kind,
            if invalidate {
                EngineFailureKind::Cancellation
            } else {
                EngineFailureKind::Timeout
            }
        );
        assert!(rig.manager.snapshot().game_move_job.is_none());
        if kata {
            assert!(matches!(
                rig.manager.snapshot().lifecycle,
                ForegroundEngineLifecycleDto::Ready { .. }
            ));
        } else {
            rig.reaped();
        }
    }
    let rig = Rig::new(true, "unclean");
    let active = rig.manager.start_game_move(rig.request(100)).unwrap();
    assert_eq!(active.wait().unwrap_err().kind, EngineFailureKind::Timeout);
    assert!(matches!(
        rig.manager.snapshot().lifecycle,
        ForegroundEngineLifecycleDto::Error { .. }
    ));
    rig.reaped();
}

#[test]
fn active_lanes_survive_busy_move_admission_without_cancel_or_pipe_writes() {
    let rig = Rig::new(true, "hold");
    let cancel = Arc::new(AnalysisCancelToken::new());
    rig.manager
        .register_job(&rig.run, AnalysisJobLane::WholeGame, cancel.clone())
        .unwrap();
    let before = rig.trace();
    assert_eq!(
        rig.manager.start_game_move(rig.request(3000)).err().unwrap().kind,
        EngineFailureKind::Occupied
    );
    assert!(!cancel.is_cancelled());
    assert_eq!(rig.trace(), before);
}

#[test]
fn engine_exit_is_not_misreported_as_user_cancellation() {
    let rig = Rig::new(false, "exit");
    let error = rig
        .manager
        .start_game_move(rig.request(3000))
        .unwrap()
        .wait()
        .unwrap_err();
    assert_eq!(error.kind, EngineFailureKind::ProcessExit);
    assert!(matches!(
        rig.manager.snapshot().lifecycle,
        ForegroundEngineLifecycleDto::Error { .. }
    ));
    rig.reaped();
}

#[test]
fn lifecycle_retirement_never_returns_the_old_move_or_releases_a_live_child() {
    for kata in [false, true] {
        for action in ["stop", "restart", "switch", "exit", "path", "departure"] {
            let rig = Rig::new(kata, "hold");
            let active = rig.manager.start_game_move(rig.request(5000)).unwrap();
            rig.wait_for(if kata { "\"maxVisits\":8" } else { "genmove B" });
            let old_pid = std::fs::read_to_string(rig.dir.join("pid")).unwrap();
            match action {
                "stop" => rig.manager.stop().unwrap(),
                "restart" => rig.manager.restart().unwrap(),
                "switch" => rig.manager.switch_to("second").unwrap(),
                "exit" => rig.manager.teardown().unwrap(),
                "path" => rig
                    .manager
                    .invalidate_game_move_position(17, &NodePath::default()),
                "departure" => rig.manager.begin_continuous_departure(),
                _ => unreachable!(),
            }
            assert_eq!(
                active.wait().unwrap_err().kind,
                EngineFailureKind::Cancellation,
                "{kata} {action}"
            );
            let deadline = Instant::now() + Duration::from_secs(3);
            loop {
                let snapshot = rig.manager.snapshot();
                let finished = match action {
                    "restart" | "switch" => {
                        matches!(&snapshot.lifecycle, ForegroundEngineLifecycleDto::Ready { run } if run.run_id != rig.run)
                    }
                    "path" | "departure" if kata => {
                        matches!(snapshot.lifecycle, ForegroundEngineLifecycleDto::Ready { .. })
                    }
                    _ => matches!(snapshot.lifecycle, ForegroundEngineLifecycleDto::NoEngine { .. }),
                };
                let needs_reap = !kata || !matches!(action, "path" | "departure");
                let reaped = !needs_reap || !std::path::Path::new(&format!("/proc/{old_pid}")).exists();
                if finished && snapshot.game_move_job.is_none() && reaped {
                    break;
                }
                assert!(
                    Instant::now() < deadline,
                    "retirement stuck: {kata} {action} {snapshot:?}"
                );
                std::thread::sleep(Duration::from_millis(2));
            }
            if !kata || !matches!(action, "path" | "departure") {
                assert!(!std::path::Path::new(&format!("/proc/{old_pid}")).exists());
            }
        }
    }
}

#[test]
fn exact_root_player_and_handicap_remain_setup_not_synthetic_plays() {
    let rig = Rig::new(true, "normal");
    for text in [
        "(;SZ[5]RU[Chinese-KGS]PL[W])",
        "(;SZ[5]RU[Chinese-KGS]HA[2]AB[aa][ee]PL[W])",
    ] {
        let mut request = rig.request(3000);
        request.position = CurrentSgfDocument::open(text)
            .unwrap()
            .exact_position(&NodePath::default())
            .unwrap();
        request.identity.node_path = NodePath::default();
        let stones = request.position.dto().initial_stones.clone();
        rig.manager.start_game_move(request).unwrap().wait().unwrap();
        let trace = rig.trace();
        let query: serde_json::Value = serde_json::from_str(trace.lines().last().unwrap()).unwrap();
        assert_eq!(query["initialPlayer"], "W");
        assert_eq!(query["moves"], serde_json::json!([]));
        let expected = if stones.is_empty() {
            serde_json::json!([])
        } else {
            serde_json::json!([["B", "A5"], ["B", "E1"]])
        };
        assert_eq!(query["initialStones"], expected);
    }
    let rig = Rig::new(false, "normal");
    let mut request = rig.request(3000);
    request.position = CurrentSgfDocument::open("(;SZ[5]RU[Chinese-KGS]HA[2]AB[aa][ee])")
        .unwrap()
        .exact_position(&NodePath::default())
        .unwrap();
    let before = rig.trace();
    assert_eq!(
        rig.manager.start_game_move(request).err().unwrap().kind,
        EngineFailureKind::UnsupportedCapability
    );
    assert_eq!(rig.trace(), before);
}

#[test]
fn old_query_id_cannot_supply_a_current_move() {
    let rig = Rig::new(true, "wrongid");
    let error = rig
        .manager
        .start_game_move(rig.request(200))
        .unwrap()
        .wait()
        .unwrap_err();
    assert_eq!(error.kind, EngineFailureKind::Timeout);
    assert!(matches!(
        rig.manager.snapshot().lifecycle,
        ForegroundEngineLifecycleDto::Ready { .. }
    ));
}

fn match_analysis_request(run_id: &str) -> SelectedNodeJobRequest {
    SelectedNodeJobRequest {
        run_id: run_id.into(),
        mode: AnalysisJobModeDto::Finite,
        generation: 17,
        node_path: NodePath::default(),
        board_width: 5,
        board_height: 5,
        position_empty: true,
        query: katago_protocol::AnalysisQuery {
            id: "analysis".into(),
            moves: Vec::new(),
            initial_stones: Vec::new(),
            rules: "chinese".into(),
            komi: 6.5,
            board_x_size: 5,
            board_y_size: 5,
            analyze_turns: Some(vec![0]),
            max_visits: Some(8),
            include_ownership: None,
            include_policy: None,
            report_during_search_every: None,
            override_settings: None,
        },
    }
}

#[test]
fn match_preparation_drains_analysis_or_preserves_unresponsive_foreground() {
    for clean in [true, false] {
        let rig = Rig::new(true, if clean { "hold" } else { "unclean" });
        let old_pid = std::fs::read_to_string(rig.dir.join("pid")).unwrap();
        let job = rig
            .manager
            .start_selected_node_job(match_analysis_request(&rig.run))
            .unwrap();
        rig.wait_for(&job.job_id);
        std::fs::write(rig.dir.join("mode"), "normal").unwrap();
        let request = rig.request(3000);
        rig.manager.reserve_match("drain").unwrap();
        let prepared =
            rig.manager
                .prepare_reserved_match("drain", "second", &request.position, request.identity.budget);
        assert_eq!(prepared.is_ok(), clean);
        if !clean {
            assert_eq!(prepared.unwrap_err().kind, EngineFailureKind::Timeout);
        }
        rig.manager.abort_reserved_match("drain").unwrap();
        assert!(std::path::Path::new(&format!("/proc/{old_pid}")).exists());
        assert!(matches!(rig.manager.snapshot().lifecycle,
            ForegroundEngineLifecycleDto::Ready { run } if run.run_id == rig.run));
        assert!(rig.trace().contains("terminateId"));
        if !clean {
            let deadline = Instant::now() + Duration::from_secs(7);
            loop {
                if let ForegroundEngineLifecycleDto::Error { failure, .. } = rig.manager.snapshot().lifecycle
                {
                    assert_eq!(failure.kind, EngineFailureKind::Timeout);
                    break;
                }
                assert!(
                    Instant::now() < deadline,
                    "aborted match left analysis Stopping without cancellation supervision"
                );
                std::thread::sleep(Duration::from_millis(10));
            }
            rig.manager.stop().unwrap();
            let stop_deadline = Instant::now() + Duration::from_secs(3);
            while !matches!(
                rig.manager.snapshot().lifecycle,
                ForegroundEngineLifecycleDto::NoEngine { .. }
            ) {
                assert!(Instant::now() < stop_deadline, "foreground stop did not complete");
                std::thread::sleep(Duration::from_millis(5));
            }
            assert!(!std::path::Path::new(&format!("/proc/{old_pid}")).exists());
            rig.manager.start("second").unwrap();
            let ready_deadline = Instant::now() + Duration::from_secs(3);
            let recovered_run = loop {
                match rig.manager.snapshot().lifecycle {
                    ForegroundEngineLifecycleDto::Ready { run } => break run,
                    ForegroundEngineLifecycleDto::Error { failure, .. } => panic!("{failure:?}"),
                    _ => assert!(
                        Instant::now() < ready_deadline,
                        "recovered engine readiness timed out"
                    ),
                }
                std::thread::sleep(Duration::from_millis(5));
            };
            assert_ne!(recovered_run.run_id, rig.run);
            assert_eq!(recovered_run.profile_id, "second");
            let recovered_job = rig
                .manager
                .start_selected_node_job(match_analysis_request(&recovered_run.run_id))
                .unwrap();
            rig.wait_for(&recovered_job.job_id);
            let finish_deadline = Instant::now() + Duration::from_secs(2);
            loop {
                if rig.manager.snapshot().selected_node_job.is_none() {
                    break;
                }
                assert!(
                    Instant::now() < finish_deadline,
                    "recovered engine analysis did not complete"
                );
                std::thread::sleep(Duration::from_millis(5));
            }
            assert!(matches!(
                rig.manager.snapshot().lifecycle,
                ForegroundEngineLifecycleDto::Ready { run } if run.run_id == recovered_run.run_id
            ));
        }
    }
}

#[test]
fn match_profile_change_prevents_install_closure_and_keeps_candidate_deletion_protected() {
    let rig = Rig::new(true, "normal");
    let request = rig.request(3000);
    rig.manager.reserve_match("profile").unwrap();
    let run = rig
        .manager
        .prepare_reserved_match("profile", "second", &request.position, request.identity.budget)
        .unwrap();
    assert!(rig.manager.assert_profile_deletable(&run.profile_id).is_err());
    assert!(rig
        .manager
        .commit_reserved_match::<()>("other", || panic!("wrong owner installed"))
        .is_err());
    let mut changed = rig.catalog.get("second").unwrap();
    changed.profile.name = "changed after prepare".into();
    rig.catalog.upsert(changed);
    assert!(rig
        .manager
        .commit_reserved_match::<()>("profile", || panic!("changed profile installed"))
        .is_err());
    rig.manager.abort_reserved_match("profile").unwrap();
    rig.manager.abort_reserved_match("profile").unwrap();
    rig.manager.stop_reserved_match("profile").unwrap();
    rig.manager.assert_profile_deletable("second").unwrap();
}

#[test]
fn match_reservation_refuses_ordinary_mutations_and_analysis_without_process_io() {
    let rig = Rig::new(true, "normal");
    let before = rig.trace();
    rig.manager.reserve_match("session").unwrap();
    assert_eq!(rig.manager.match_reservation_owner().as_deref(), Some("session"));
    assert!(rig.manager.reserve_match("other").is_err());
    assert!(rig.manager.start("second").is_err());
    assert!(rig.manager.stop().is_err());
    assert!(rig.manager.restart().is_err());
    assert!(rig.manager.switch_to("second").is_err());
    assert!(rig.manager.start_game_move(rig.request(3000)).is_err());
    assert!(rig
        .manager
        .start_selected_node_job(match_analysis_request(&rig.run))
        .is_err());
    let selected = match_analysis_request(&rig.run);
    assert!(rig
        .manager
        .start_whole_game_analysis(WholeGameJobRequest {
            run_id: rig.run.clone(),
            generation: 17,
            work_items: vec![WholeGameWorkItem {
                node_path: selected.node_path,
                query: selected.query,
                board_width: 5,
                board_height: 5,
                move_number: 0
            }],
        })
        .is_err());
    assert!(rig.manager.check_analysis_task_admission(None).is_err());
    assert!(rig.manager.resume_continuous().is_err());
    assert!(rig.manager.abort_reserved_match("other").is_err());
    assert!(matches!(rig.manager.snapshot().lifecycle,
        ForegroundEngineLifecycleDto::Ready { run } if run.run_id == rig.run));
    assert_eq!(rig.trace(), before);
    rig.manager.abort_reserved_match("session").unwrap();
    assert_eq!(rig.manager.match_reservation_owner(), None);
    assert!(matches!(rig.manager.snapshot().lifecycle,
        ForegroundEngineLifecycleDto::Ready { run } if run.run_id == rig.run));
}

#[test]
fn match_candidate_failure_and_failed_install_preserve_old_foreground() {
    for kata in [false, true] {
        let rig = Rig::new(kata, "normal");
        let old_pid = std::fs::read_to_string(rig.dir.join("pid")).unwrap();
        std::fs::write(rig.dir.join("mode"), "readiness_fail").unwrap();
        rig.manager.reserve_match("failed-ready").unwrap();
        let request = rig.request(3000);
        assert!(rig
            .manager
            .prepare_reserved_match(
                "failed-ready",
                "second",
                &request.position,
                request.identity.budget
            )
            .is_err());
        rig.reaped();
        assert!(std::path::Path::new(&format!("/proc/{old_pid}")).exists());
        assert!(matches!(rig.manager.snapshot().lifecycle,
            ForegroundEngineLifecycleDto::Ready { run } if run.run_id == rig.run));
        assert_eq!(rig.manager.match_reservation_owner(), None);

        std::fs::write(rig.dir.join("mode"), "normal").unwrap();
        rig.manager.reserve_match("persist-failure").unwrap();
        let candidate = rig
            .manager
            .prepare_reserved_match(
                "persist-failure",
                "second",
                &request.position,
                request.identity.budget,
            )
            .unwrap();
        assert_ne!(candidate.run_id, rig.run);
        assert!(candidate.capability_snapshot.as_ref().unwrap().game_move);
        assert!(rig.manager.assert_profile_deletable("second").is_err());
        let error = rig
            .manager
            .commit_reserved_match::<()>("persist-failure", || Err("disk full".into()))
            .unwrap_err();
        assert!(error.message.contains("disk full"));
        assert!(matches!(rig.manager.snapshot().lifecycle,
            ForegroundEngineLifecycleDto::Ready { run } if run.run_id == rig.run));
        rig.manager.abort_reserved_match("persist-failure").unwrap();
        rig.reaped();
        assert!(std::path::Path::new(&format!("/proc/{old_pid}")).exists());
    }
}

#[test]
fn reserved_move_uses_selected_candidate_and_consumes_publication_once() {
    for kata in [false, true] {
        let rig = Rig::new(kata, "normal");
        let old_pid = std::fs::read_to_string(rig.dir.join("pid")).unwrap();
        let mut request = rig.request(3000);
        rig.manager.reserve_match("session").unwrap();
        let candidate = rig
            .manager
            .prepare_reserved_match("session", "second", &request.position, request.identity.budget)
            .unwrap();
        let installed = rig.manager.commit_reserved_match("session", || Ok(42)).unwrap();
        assert_eq!(installed, 42);
        assert!(rig.manager.abort_reserved_match("session").is_err());
        assert!(rig
            .manager
            .start_reserved_game_move("session", request.clone())
            .is_err());
        request.identity.run_id = candidate.run_id.clone();
        assert!(rig
            .manager
            .start_reserved_game_move("other", request.clone())
            .is_err());
        let result = rig
            .manager
            .start_reserved_game_move("session", request)
            .unwrap()
            .wait()
            .unwrap();
        assert_eq!(result.run_id, candidate.run_id);
        assert!(rig.manager.claim_game_move_result(&result).is_err());
        assert!(rig
            .manager
            .claim_reserved_game_move_result("other", &result)
            .is_err());
        let mut stale = result.clone();
        stale.generation += 1;
        assert!(rig
            .manager
            .claim_reserved_game_move_result("session", &stale)
            .is_err());
        rig.manager
            .claim_reserved_game_move_result("session", &result)
            .unwrap();
        assert!(rig
            .manager
            .claim_reserved_game_move_result("session", &result)
            .is_err());
        rig.manager.stop_reserved_match("session").unwrap();
        rig.reaped();
        assert!(!std::path::Path::new(&format!("/proc/{old_pid}")).exists());
        assert_eq!(rig.manager.match_reservation_owner(), None);
        assert!(matches!(
            rig.manager.snapshot().lifecycle,
            ForegroundEngineLifecycleDto::NoEngine { .. }
        ));
        assert!(rig
            .manager
            .claim_reserved_game_move_result("session", &result)
            .is_err());
    }
}

#[test]
fn match_prepare_enforces_exact_setup_and_qualified_gtp_identity() {
    for (kata, mode, allowed) in [
        (true, "normal", true),
        (false, "normal", false),
        (false, "identity", false),
    ] {
        let rig = Rig::new(kata, "normal");
        std::fs::write(rig.dir.join("mode"), mode).unwrap();
        let position = CurrentSgfDocument::open("(;SZ[5]RU[Chinese-KGS]KM[6.5]AB[aa][ee]PL[W])")
            .unwrap()
            .exact_position(&NodePath::default())
            .unwrap();
        rig.manager.reserve_match("setup").unwrap();
        let result = rig.manager.prepare_reserved_match(
            "setup",
            "second",
            &position,
            rig.request(3000).identity.budget,
        );
        assert_eq!(result.is_ok(), allowed);
        if allowed {
            rig.manager.abort_reserved_match("setup").unwrap();
        }
        assert!(matches!(rig.manager.snapshot().lifecycle,
            ForegroundEngineLifecycleDto::Ready { run } if run.run_id == rig.run));
    }
    let rig = Rig::new(false, "normal");
    std::fs::write(rig.dir.join("mode"), "identity").unwrap();
    let request = rig.request(3000);
    rig.manager.reserve_match("identity").unwrap();
    let error = rig
        .manager
        .prepare_reserved_match("identity", "second", &request.position, request.identity.budget)
        .unwrap_err();
    assert_eq!(error.kind, EngineFailureKind::UnsupportedCapability);
}

#[test]
fn match_stop_seals_pending_move_reaps_children_and_does_not_resume_analysis() {
    for kata in [false, true] {
        let rig = Rig::new(kata, "normal");
        std::fs::write(rig.dir.join("mode"), "hold").unwrap();
        let mut request = rig.request(3000);
        rig.manager.reserve_match("stop").unwrap();
        let candidate = rig
            .manager
            .prepare_reserved_match("stop", "second", &request.position, request.identity.budget)
            .unwrap();
        rig.manager.commit_reserved_match("stop", || Ok(())).unwrap();
        request.identity.run_id = candidate.run_id;
        let handle = rig.manager.start_reserved_game_move("stop", request).unwrap();
        if kata {
            rig.wait_for("initialStones");
        } else {
            rig.wait_for("genmove");
        }
        rig.manager.stop_reserved_match("stop").unwrap();
        assert!(handle.wait().is_err());
        rig.reaped();
        let after = rig.trace();
        assert!(rig.manager.snapshot().selected_node_job.is_none());
        assert_eq!(rig.manager.match_reservation_owner(), None);
        assert_eq!(rig.trace(), after);
    }
}

#[test]
fn match_can_commit_from_unloaded_and_rejects_zero_budget_before_commit() {
    let rig = Rig::new(true, "normal");
    rig.manager.teardown().unwrap();
    let request = rig.request(3000);
    rig.manager.reserve_match("invalid-budget").unwrap();
    assert!(rig
        .manager
        .prepare_reserved_match(
            "invalid-budget",
            "second",
            &request.position,
            ComputeBudgetDto {
                deadline_ms: 0,
                max_visits: Some(8)
            }
        )
        .is_err());
    assert!(matches!(
        rig.manager.snapshot().lifecycle,
        ForegroundEngineLifecycleDto::NoEngine { .. }
    ));
    rig.manager.reserve_match("new").unwrap();
    let run = rig
        .manager
        .prepare_reserved_match("new", "second", &request.position, request.identity.budget)
        .unwrap();
    rig.manager.commit_reserved_match("new", || Ok(())).unwrap();
    assert!(matches!(rig.manager.snapshot().lifecycle,
        ForegroundEngineLifecycleDto::Ready { run: current } if current.run_id == run.run_id));
    rig.manager.stop_reserved_match("new").unwrap();
    rig.reaped();
}

#[test]
fn match_abort_during_handshake_keeps_old_process_and_releases_candidate() {
    for kata in [false, true] {
        let rig = Rig::new(kata, "normal");
        let old_pid = std::fs::read_to_string(rig.dir.join("pid")).unwrap();
        std::fs::write(rig.dir.join("mode"), "readiness_hold").unwrap();
        rig.manager.reserve_match("handshake").unwrap();
        let manager = rig.manager.clone();
        let request = rig.request(3000);
        let preparing = std::thread::spawn(move || {
            manager
                .prepare_reserved_match("handshake", "second", &request.position, request.identity.budget)
                .unwrap_err()
        });
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            let pid = std::fs::read_to_string(rig.dir.join("pid")).unwrap();
            if !pid.is_empty() && pid != old_pid {
                break;
            }
            assert!(Instant::now() < deadline, "candidate never spawned");
            std::thread::sleep(Duration::from_millis(2));
        }
        // Cleanup has a fixed budget, not a guarantee that SIGKILL has been scheduled.
        abort_candidate_with_retry(&rig, "handshake");
        let failure = preparing.join().unwrap();
        assert_eq!(failure.kind, EngineFailureKind::Cancellation);
        rig.reaped();
        assert!(std::path::Path::new(&format!("/proc/{old_pid}")).exists());
        assert_eq!(rig.manager.match_reservation_owner(), None);
        assert!(matches!(rig.manager.snapshot().lifecycle,
            ForegroundEngineLifecycleDto::Ready { run } if run.run_id == rig.run));
    }
}

fn abort_candidate_with_retry(rig: &Rig, owner: &str) {
    let deadline = Instant::now() + Duration::from_secs(3);
    while let Err(failure) = rig.manager.abort_reserved_match(owner) {
        assert_eq!(failure.kind, EngineFailureKind::Timeout, "{failure:?}");
        assert_eq!(failure.operation, EngineOperationDto::Job);
        assert_eq!(failure.profile_id.as_deref(), Some("second"));
        assert!(failure.run_id.as_ref().is_some_and(|run| run != &rig.run));
        assert!(matches!(rig.manager.snapshot().lifecycle,
            ForegroundEngineLifecycleDto::Ready { run } if run.run_id == rig.run));
        assert!(
            Instant::now() < deadline,
            "candidate cleanup did not finish: {failure:?}"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn match_abort_unconfirmed_cleanup_retains_reservation_until_retry_reaps_candidate() {
    for kata in [false, true] {
        let rig = Rig::with_cleanup_timeout(kata, "normal", Duration::ZERO);
        let old_pid = std::fs::read_to_string(rig.dir.join("pid")).unwrap();
        rig.manager.reserve_match("cleanup").unwrap();
        let request = rig.request(3000);
        let candidate = rig
            .manager
            .prepare_reserved_match("cleanup", "second", &request.position, request.identity.budget)
            .unwrap();

        // A live candidate and no observation budget force the unconfirmed-kill path.
        let failure = rig.manager.abort_reserved_match("cleanup").unwrap_err();
        assert_eq!(failure.kind, EngineFailureKind::Timeout);
        assert_eq!(failure.operation, EngineOperationDto::Job);
        assert_eq!(failure.run_id.as_deref(), Some(candidate.run_id.as_str()));
        assert_eq!(failure.profile_id.as_deref(), Some("second"));
        assert_eq!(rig.manager.match_reservation_owner().as_deref(), Some("cleanup"));
        assert!(rig.manager.reserve_match("replacement").is_err());
        assert!(rig.manager.start("engine").is_err());
        assert!(rig.manager.assert_profile_deletable("second").is_err());
        assert!(matches!(rig.manager.snapshot().lifecycle,
            ForegroundEngineLifecycleDto::Ready { run } if run.run_id == rig.run));

        abort_candidate_with_retry(&rig, "cleanup");
        rig.reaped();
        assert!(std::path::Path::new(&format!("/proc/{old_pid}")).exists());
        assert_eq!(rig.manager.match_reservation_owner(), None);
        rig.manager.assert_profile_deletable("second").unwrap();
        rig.manager.reserve_match("replacement").unwrap();
        rig.manager.abort_reserved_match("replacement").unwrap();

        // This rig also gives teardown zero observation time; explicitly finish reaping it.
        let deadline = Instant::now() + Duration::from_secs(3);
        while let Err(failure) = rig.manager.teardown() {
            assert_eq!(failure.kind, EngineFailureKind::Timeout, "{failure:?}");
            assert!(
                Instant::now() < deadline,
                "old engine cleanup did not finish: {failure:?}"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
        assert!(!std::path::Path::new(&format!("/proc/{old_pid}")).exists());
    }
}

fn pk_pid(rig: &Rig, run: &EngineRunDto) -> String {
    std::fs::read_to_string(rig.dir.join(format!("run-{}", run.run_id))).unwrap()
}

fn assert_pid_alive(pid: &str, alive: bool) {
    assert_eq!(
        std::path::Path::new(&format!("/proc/{pid}")).exists(),
        alive,
        "unexpected process ownership for PID {pid}"
    );
}

fn prepare_pk(rig: &Rig, owner: &str) -> [EngineRunDto; 2] {
    let request = rig.request(3000);
    rig.manager.reserve_match(owner).unwrap();
    rig.manager
        .prepare_reserved_pk(
            owner,
            [
                ("engine", request.identity.budget),
                ("engine", request.identity.budget),
            ],
            &request.position,
        )
        .unwrap()
}

#[test]
fn pk_same_profile_owns_distinct_processes_and_dispatches_both_sides() {
    let rig = Rig::new(true, "normal");
    let old_pid = std::fs::read_to_string(rig.dir.join("pid")).unwrap();
    let runs = prepare_pk(&rig, "pk");
    assert_ne!(runs[0].run_id, runs[1].run_id);
    assert_eq!(runs[0].profile_id, runs[1].profile_id);
    let pids = runs.each_ref().map(|run| pk_pid(&rig, run));
    assert_ne!(pids[0], pids[1]);
    assert_ne!(pids[0], old_pid);
    assert_ne!(pids[1], old_pid);
    assert!(matches!(rig.manager.snapshot().lifecycle,
        ForegroundEngineLifecycleDto::Ready { run } if run.run_id == rig.run));
    rig.manager.commit_reserved_match("pk", || Ok(())).unwrap();
    for run in &runs {
        let mut request = rig.request(3000);
        request.identity.run_id = run.run_id.clone();
        let result = rig
            .manager
            .start_reserved_game_move("pk", request)
            .unwrap()
            .wait()
            .unwrap();
        assert_eq!(result.run_id, run.run_id);
        rig.manager
            .claim_reserved_game_move_result("pk", &result)
            .unwrap();
        assert!(rig.manager.assert_profile_deletable("engine").is_err());
        for pid in &pids {
            assert_pid_alive(pid, true);
        }
    }
    rig.manager.stop_reserved_match("pk").unwrap();
    for pid in &pids {
        assert_pid_alive(pid, false);
    }
    assert_pid_alive(&old_pid, false);
    assert_eq!(rig.manager.match_reservation_owner(), None);
}

#[test]
fn pk_second_prepare_failure_reports_white_and_rolls_back_only_candidates() {
    let rig = Rig::new(true, "normal");
    let old_pid = std::fs::read_to_string(rig.dir.join("pid")).unwrap();
    std::fs::write(rig.dir.join("mode"), "second_readiness_fail").unwrap();
    std::fs::write(rig.dir.join("remaining-ready"), "1").unwrap();
    rig.manager.reserve_match("pk").unwrap();
    let request = rig.request(3000);
    let (side, error) = rig
        .manager
        .prepare_reserved_pk(
            "pk",
            [
                ("engine", request.identity.budget),
                ("engine", request.identity.budget),
            ],
            &request.position,
        )
        .unwrap_err();
    assert_eq!(side, PlayerColor::White);
    assert_eq!(error.profile_id.as_deref(), Some("engine"));
    assert!(error.run_id.is_some());
    for entry in std::fs::read_dir(&rig.dir).unwrap() {
        let entry = entry.unwrap();
        if entry.file_name().to_string_lossy().starts_with("run-") {
            let pid = std::fs::read_to_string(entry.path()).unwrap();
            assert_pid_alive(&pid, pid == old_pid);
        }
    }
    assert_pid_alive(&old_pid, true);
    assert_eq!(rig.manager.match_reservation_owner(), None);
    assert!(matches!(rig.manager.snapshot().lifecycle,
        ForegroundEngineLifecycleDto::Ready { run } if run.run_id == rig.run));
}

#[test]
fn pk_commit_checks_second_profile_before_install_and_abort_preserves_foreground() {
    let rig = Rig::new(true, "normal");
    let old_pid = std::fs::read_to_string(rig.dir.join("pid")).unwrap();
    let request = rig.request(3000);
    rig.manager.reserve_match("pk").unwrap();
    let runs = rig
        .manager
        .prepare_reserved_pk(
            "pk",
            [
                ("engine", request.identity.budget),
                ("second", request.identity.budget),
            ],
            &request.position,
        )
        .unwrap();
    assert!(rig.manager.assert_profile_deletable("second").is_err());
    let pids = runs.each_ref().map(|run| pk_pid(&rig, run));
    let mut changed = rig.catalog.get("second").unwrap();
    changed.profile.name.push_str(" changed");
    rig.catalog.upsert(changed);
    let error = rig
        .manager
        .commit_reserved_match::<()>("pk", || panic!("stale profile installed"))
        .unwrap_err();
    assert_eq!(error.run_id.as_deref(), Some(runs[1].run_id.as_str()));
    rig.manager.abort_reserved_match("pk").unwrap();
    for pid in &pids {
        assert_pid_alive(pid, false);
    }
    assert_pid_alive(&old_pid, true);
    assert!(matches!(rig.manager.snapshot().lifecycle,
        ForegroundEngineLifecycleDto::Ready { run } if run.run_id == rig.run));
    let runs = prepare_pk(&rig, "failed-install");
    let pids = runs.each_ref().map(|run| pk_pid(&rig, run));
    assert!(rig
        .manager
        .commit_reserved_match::<()>("failed-install", || Err("disk full".into()))
        .is_err());
    rig.manager.abort_reserved_match("failed-install").unwrap();
    for pid in &pids {
        assert_pid_alive(pid, false);
    }
    assert_pid_alive(&old_pid, true);
}

#[test]
fn pk_candidate_exit_before_commit_never_installs_or_retires_the_old_foreground() {
    for side in 0..2 {
        let rig = Rig::new(true, "normal");
        let old_pid = std::fs::read_to_string(rig.dir.join("pid")).unwrap();
        let runs = prepare_pk(&rig, "pk");
        let pids = runs.each_ref().map(|run| pk_pid(&rig, run));
        assert!(std::process::Command::new("kill")
            .args(["-TERM", &pids[side]])
            .status()
            .unwrap()
            .success());
        let deadline = Instant::now() + Duration::from_secs(3);
        while std::fs::read_to_string(format!("/proc/{}/stat", pids[side]))
            .is_ok_and(|status| status.split_whitespace().nth(2) != Some("Z"))
        {
            assert!(Instant::now() < deadline, "candidate did not exit before commit");
            std::thread::sleep(Duration::from_millis(5));
        }
        let error = rig
            .manager
            .commit_reserved_match::<()>("pk", || panic!("dead candidate installed"))
            .unwrap_err();
        assert_eq!(error.kind, EngineFailureKind::ProcessExit);
        assert_eq!(error.run_id.as_deref(), Some(runs[side].run_id.as_str()));
        assert_eq!(error.profile_id.as_deref(), Some("engine"));
        rig.manager.abort_reserved_match("pk").unwrap();
        for pid in &pids {
            assert_pid_alive(pid, false);
        }
        assert_pid_alive(&old_pid, true);
        assert_eq!(rig.manager.match_reservation_owner(), None);
        assert!(matches!(rig.manager.snapshot().lifecycle,
            ForegroundEngineLifecycleDto::Ready { run } if run.run_id == rig.run));
    }
}

#[test]
fn pk_gtp_visits_budget_is_refused_before_spawn_and_deadline_only_gtp_pair_dispatches_each_run() {
    let rig = Rig::new(false, "normal");
    let old_pid = std::fs::read_to_string(rig.dir.join("pid")).unwrap();
    let request = rig.request(3000);
    let visits = ComputeBudgetDto {
        deadline_ms: 3000,
        max_visits: Some(8),
    };
    rig.manager.reserve_match("pk").unwrap();
    let (side, error) = rig
        .manager
        .prepare_reserved_pk(
            "pk",
            [("engine", request.identity.budget), ("second", visits)],
            &request.position,
        )
        .unwrap_err();
    assert_eq!(side, PlayerColor::White);
    assert_eq!(error.kind, EngineFailureKind::UnsupportedCapability);
    assert_eq!(error.profile_id.as_deref(), Some("second"));
    assert_eq!(std::fs::read_to_string(rig.dir.join("pid")).unwrap(), old_pid);
    rig.manager.abort_reserved_match("pk").unwrap();
    assert_pid_alive(&old_pid, true);
    let runs = prepare_pk(&rig, "gtp-pair");
    assert_ne!(runs[0].run_id, runs[1].run_id);
    assert!(runs
        .iter()
        .all(|run| run.adapter_kind == EngineBackend::GenericGtp));
    rig.manager.commit_reserved_match("gtp-pair", || Ok(())).unwrap();
    // The White run is a match resident, not the primary; its stdout must still reach the move.
    for run in &runs {
        let mut request = rig.request(3000);
        request.identity.run_id = run.run_id.clone();
        let result = rig
            .manager
            .start_reserved_game_move("gtp-pair", request)
            .unwrap()
            .wait()
            .unwrap();
        assert_eq!(result.run_id, run.run_id);
        rig.manager
            .claim_reserved_game_move_result("gtp-pair", &result)
            .unwrap();
    }
    rig.manager.stop_reserved_match("gtp-pair").unwrap();
    assert_pid_alive(&old_pid, false);
}

#[test]
fn pk_pause_acknowledges_cancel_and_admits_fresh_job_on_same_process() {
    let rig = Rig::new(true, "pause_once");
    let runs = prepare_pk(&rig, "pk");
    let pids = runs.each_ref().map(|run| pk_pid(&rig, run));
    rig.manager.commit_reserved_match("pk", || Ok(())).unwrap();
    let mut request = rig.request(3000);
    request.identity.run_id = runs[1].run_id.clone();
    let first = rig
        .manager
        .start_reserved_game_move("pk", request.clone())
        .unwrap();
    let old_job = first.identity.job_id.clone();
    rig.wait_for(&old_job);
    rig.manager.pause_reserved_match("pk").unwrap();
    assert_eq!(first.wait().unwrap_err().kind, EngineFailureKind::Cancellation);
    assert_eq!(rig.manager.match_reservation_owner().as_deref(), Some("pk"));
    for pid in &pids {
        assert_pid_alive(pid, true);
    }
    let second = rig.manager.start_reserved_game_move("pk", request).unwrap();
    assert_ne!(second.identity.job_id, old_job);
    let result = second.wait().unwrap();
    rig.manager
        .claim_reserved_game_move_result("pk", &result)
        .unwrap();
    rig.manager.pause_reserved_match("pk").unwrap();
    assert!(rig
        .manager
        .claim_reserved_game_move_result("pk", &result)
        .is_err());
    rig.manager.stop_reserved_match("pk").unwrap();
    for pid in &pids {
        assert_pid_alive(pid, false);
    }
}

#[test]
fn pk_idle_second_exit_is_observable_during_work_and_after_pause() {
    for paused in [false, true] {
        let rig = Rig::new(true, "hold");
        let runs = prepare_pk(&rig, "pk");
        let pids = runs.each_ref().map(|run| pk_pid(&rig, run));
        rig.manager.commit_reserved_match("pk", || Ok(())).unwrap();
        let events = rig.manager.subscribe();
        let mut request = rig.request(3000);
        request.identity.run_id = runs[0].run_id.clone();
        let handle = rig
            .manager
            .start_reserved_game_move("pk", request.clone())
            .unwrap();
        rig.wait_for(&handle.identity.job_id);
        let active = if paused {
            rig.manager.pause_reserved_match("pk").unwrap();
            assert!(handle.wait().is_err());
            None
        } else {
            Some(handle)
        };
        assert!(std::process::Command::new("kill")
            .args(["-TERM", &pids[1]])
            .status()
            .unwrap()
            .success());
        let deadline = Instant::now() + Duration::from_secs(3);
        let error = loop {
            if let Ok(ForegroundEngineEventDto::Failure { failure }) =
                events.recv_timeout(Duration::from_millis(20))
            {
                if failure.operation == EngineOperationDto::UnexpectedExit {
                    break failure;
                }
            }
            assert!(Instant::now() < deadline, "idle run exit was not published");
        };
        assert_eq!(error.run_id.as_deref(), Some(runs[1].run_id.as_str()));
        assert_eq!(error.profile_id.as_deref(), Some("engine"));
        if let Some(handle) = active {
            assert!(handle.wait().is_err());
        }
        assert!(rig.manager.start_reserved_game_move("pk", request).is_err());
        assert_eq!(rig.manager.match_reservation_owner().as_deref(), Some("pk"));
        rig.manager.stop_reserved_match("pk").unwrap();
        for pid in &pids {
            assert_pid_alive(pid, false);
        }
    }
}

fn commit_analysis_match(rig: &Rig, deadline_ms: u32) -> GameMoveRequest {
    let mut request = rig.request(deadline_ms);
    rig.manager.reserve_match("analysis").unwrap();
    let candidate = rig
        .manager
        .prepare_reserved_match("analysis", "second", &request.position, request.identity.budget)
        .unwrap();
    rig.manager.commit_reserved_match("analysis", || Ok(())).unwrap();
    request.identity.run_id = candidate.run_id;
    request
}

#[test]
fn reserved_analysis_frames_use_exact_move_job_and_never_grant_a_human_move_permit() {
    let rig = Rig::new(true, "normal");
    let request = commit_analysis_match(&rig, 3000);
    for analysis_only in [true, false] {
        let handle = if analysis_only {
            rig.manager.start_reserved_analysis("analysis", request.clone())
        } else {
            rig.manager
                .start_reserved_game_move_with_analysis("analysis", request.clone())
        }
        .unwrap();
        let identity = handle.identity.clone();
        let mut frames = Vec::new();
        let result = handle
            .wait_with_analysis(|job, frame| {
                assert_eq!(job, &identity);
                assert_eq!(frame.job_id.to_string(), job.job_id);
                assert_eq!(frame.turn, request.position.dto().moves.len() as u32);
                frames.push(frame);
            })
            .unwrap();
        assert_eq!(frames.len(), 2);
        assert_eq!(frames[0].visits, 1);
        assert_eq!(frames[1].visits, 8);
        assert_eq!(frames[1].candidates.len(), 2);
        assert_eq!(result.job_id, identity.job_id);
        assert_eq!(
            result.result,
            GameMoveDto::Move {
                vertex: MoveVertex::Point(PointDto { x: 3, y: 1 }),
            }
        );
        if analysis_only {
            assert!(rig
                .manager
                .claim_reserved_game_move_result("analysis", &result)
                .is_err());
            rig.manager
                .cancel_reserved_analysis("analysis", &identity)
                .unwrap();
        } else {
            rig.manager
                .claim_reserved_game_move_result("analysis", &result)
                .unwrap();
            assert!(rig
                .manager
                .claim_reserved_game_move_result("analysis", &result)
                .is_err());
        }
        let trace = rig.trace();
        let query: serde_json::Value = serde_json::from_str(trace.lines().last().unwrap()).unwrap();
        assert_eq!(query["id"], identity.job_id);
        assert_eq!(query["reportDuringSearchEvery"], 0.05);
        assert_eq!(query["maxVisits"], 8);
        assert_eq!(query["includeOwnership"], true);
        assert_eq!(frames[1].ownership.as_ref().unwrap().len(), 25);
        assert_eq!(query["moves"], serde_json::json!([["B", "A5"], ["W", "pass"]]));
        assert!(rig.manager.snapshot().selected_node_job.is_none());
        assert!(rig.manager.snapshot().whole_game_job.is_none());
    }
    // Compatibility wait ignores optional analysis and cannot manufacture a permit.
    let result = rig
        .manager
        .start_reserved_analysis("analysis", request)
        .unwrap()
        .wait()
        .unwrap();
    assert!(rig
        .manager
        .claim_reserved_game_move_result("analysis", &result)
        .is_err());
    rig.manager.stop_reserved_match("analysis").unwrap();
}

#[test]
fn reserved_analysis_cancel_drains_before_move_and_cannot_cancel_an_engine_job() {
    let rig = Rig::new(true, "analysis_hold");
    let request = commit_analysis_match(&rig, 3000);
    let handle = rig
        .manager
        .start_reserved_analysis("analysis", request.clone())
        .unwrap();
    let analysis_job = handle.identity.clone();
    rig.wait_for("reportDuringSearchEvery");
    let before = rig.trace();
    assert!(rig
        .manager
        .cancel_reserved_analysis("other", &analysis_job)
        .is_err());
    let mut stale = analysis_job.clone();
    stale.generation += 1;
    assert!(rig.manager.cancel_reserved_analysis("analysis", &stale).is_err());
    stale = analysis_job.clone();
    stale.run_id = rig.run.clone();
    assert!(rig.manager.cancel_reserved_analysis("analysis", &stale).is_err());
    assert!(rig
        .manager
        .start_reserved_game_move("analysis", request.clone())
        .is_err());
    assert!(rig
        .manager
        .start_selected_node_job(match_analysis_request(&analysis_job.run_id))
        .is_err());
    assert_eq!(rig.trace(), before);
    rig.manager
        .cancel_reserved_analysis("analysis", &analysis_job)
        .unwrap();
    assert!(rig.manager.snapshot().game_move_job.is_none());
    assert_eq!(handle.wait().unwrap_err().kind, EngineFailureKind::Cancellation);
    let result = rig
        .manager
        .start_reserved_game_move("analysis", request.clone())
        .unwrap()
        .wait()
        .unwrap();
    rig.manager
        .claim_reserved_game_move_result("analysis", &result)
        .unwrap();
    let handle = rig
        .manager
        .start_reserved_game_move_with_analysis("analysis", request)
        .unwrap();
    let engine_job = handle.identity.clone();
    assert!(rig
        .manager
        .cancel_reserved_analysis("analysis", &engine_job)
        .is_err());
    rig.manager
        .cancel_reserved_analysis("analysis", &analysis_job)
        .unwrap();
    assert_eq!(rig.manager.snapshot().game_move_job, Some(engine_job));
    rig.manager.stop_reserved_match("analysis").unwrap();
    assert!(handle.wait().is_err());
    rig.reaped();
}

#[test]
fn gtp_reserved_analysis_is_rejected_before_any_process_mutation() {
    let rig = Rig::new(false, "normal");
    let request = commit_analysis_match(&rig, 3000);
    let before = rig.trace();
    for move_enabled in [false, true] {
        let error = match if move_enabled {
            rig.manager
                .start_reserved_game_move_with_analysis("analysis", request.clone())
        } else {
            rig.manager.start_reserved_analysis("analysis", request.clone())
        } {
            Ok(_) => panic!("GTP admitted reserved candidates"),
            Err(error) => error,
        };
        assert_eq!(error.kind, EngineFailureKind::UnsupportedCapability);
    }
    assert_eq!(rig.trace(), before);
    assert!(rig.manager.snapshot().game_move_job.is_none());
    let result = rig
        .manager
        .start_reserved_game_move("analysis", request)
        .unwrap()
        .wait()
        .unwrap();
    rig.manager
        .claim_reserved_game_move_result("analysis", &result)
        .unwrap();
    rig.manager.stop_reserved_match("analysis").unwrap();
}

#[test]
fn slow_reserved_analysis_consumer_cannot_hold_deadline_or_final_completion() {
    let rig = Rig::new(true, "analysis_flood");
    let mut request = commit_analysis_match(&rig, 3000);
    request.identity.budget.max_visits = Some(32);
    let handle = rig
        .manager
        .start_reserved_game_move_with_analysis("analysis", request)
        .unwrap();
    let mut frames = Vec::new();
    let result = handle
        .wait_with_analysis(|_, frame| {
            if frames.is_empty() {
                // This callback cannot finish until the independent worker has released ownership.
                let deadline = Instant::now() + Duration::from_secs(2);
                while rig.manager.snapshot().game_move_job.is_some() {
                    assert!(Instant::now() < deadline, "slow callback stalled move worker");
                    std::thread::sleep(Duration::from_millis(2));
                }
            }
            frames.push(frame);
        })
        .unwrap();
    assert!(frames.len() <= 10, "intermediate publication must stay bounded");
    assert_eq!(
        frames.last().unwrap().visits,
        32,
        "completed frame must not be dropped"
    );
    rig.manager
        .claim_reserved_game_move_result("analysis", &result)
        .unwrap();
    rig.manager.stop_reserved_match("analysis").unwrap();
}

#[test]
fn reserved_analysis_preserves_strict_warnings_turn_fences_and_hard_deadline() {
    for mode in [
        "warning",
        "wrongturn",
        "analysis_invalid_accounting",
        "analysis_hold",
    ] {
        let rig = Rig::new(true, mode);
        let request = commit_analysis_match(&rig, if mode == "analysis_hold" { 100 } else { 3000 });
        let handle = rig.manager.start_reserved_analysis("analysis", request).unwrap();
        let error = handle.wait_with_analysis(|_, _| {}).unwrap_err();
        assert_eq!(
            error.kind,
            if mode == "analysis_hold" {
                EngineFailureKind::Timeout
            } else {
                EngineFailureKind::Protocol
            }
        );
        assert!(rig.manager.snapshot().game_move_job.is_none());
        assert!(matches!(
            rig.manager.snapshot().lifecycle,
            ForegroundEngineLifecycleDto::Ready { .. }
        ));
        rig.manager.stop_reserved_match("analysis").unwrap();
    }
}

#[test]
fn rejected_reserved_analysis_keeps_run_ready_but_seals_match_without_frames() {
    for (mode, analysis_only) in [
        ("error", true),
        ("errors", true),
        ("error", false),
        ("errors", false),
    ] {
        let rig = Rig::new(true, mode);
        let request = commit_analysis_match(&rig, 3000);
        let handle = if analysis_only {
            rig.manager.start_reserved_analysis("analysis", request.clone())
        } else {
            rig.manager
                .start_reserved_game_move_with_analysis("analysis", request.clone())
        }
        .unwrap();
        let identity = handle.identity.clone();
        let failure = handle
            .wait_with_analysis(|_, _| panic!("rejected query published analysis"))
            .unwrap_err();
        assert_eq!(failure.kind, EngineFailureKind::Protocol);
        assert_eq!(failure.run_id.as_deref(), Some(identity.run_id.as_str()));
        assert_eq!(failure.job_id.as_deref(), Some(identity.job_id.as_str()));
        assert_eq!(failure.profile_id.as_deref(), Some("second"));
        assert!(matches!(rig.manager.snapshot().lifecycle,
            ForegroundEngineLifecycleDto::Ready { run } if run.run_id == identity.run_id));
        assert!(rig.manager.snapshot().game_move_job.is_none());
        assert!(!rig.trace().contains("terminateId"));
        let pid = std::fs::read_to_string(rig.dir.join("pid")).unwrap();
        assert_pid_alive(&pid, true);
        assert_eq!(rig.manager.match_reservation_owner().as_deref(), Some("analysis"));
        assert!(rig.manager.start_reserved_game_move("analysis", request).is_err());
        rig.manager.stop_reserved_match("analysis").unwrap();
        rig.reaped();
        assert!(rig.manager.match_reservation_owner().is_none());
    }
}

#[test]
fn pk_unconfirmed_pause_preserves_occupancy_until_stop() {
    let rig = Rig::new(true, "unclean");
    let runs = prepare_pk(&rig, "pk");
    rig.manager.commit_reserved_match("pk", || Ok(())).unwrap();
    let mut request = rig.request(3000);
    request.identity.run_id = runs[1].run_id.clone();
    let handle = rig
        .manager
        .start_reserved_game_move("pk", request.clone())
        .unwrap();
    rig.wait_for(&handle.identity.job_id);
    let error = rig.manager.pause_reserved_match("pk").unwrap_err();
    assert_eq!(error.kind, EngineFailureKind::Timeout);
    assert_eq!(error.run_id.as_deref(), Some(runs[1].run_id.as_str()));
    assert_eq!(error.profile_id.as_deref(), Some("engine"));
    assert!(error.job_id.is_some());
    assert!(handle.wait().is_err());
    assert_eq!(rig.manager.match_reservation_owner().as_deref(), Some("pk"));
    assert!(rig.manager.start_reserved_game_move("pk", request).is_err());
    assert!(rig.manager.assert_profile_deletable("engine").is_err());
    rig.manager.stop_reserved_match("pk").unwrap();
    for run in &runs {
        assert_pid_alive(&pk_pid(&rig, run), false);
    }
}

#[test]
fn pk_stop_during_pause_drain_never_revives_reservation() {
    let rig = Rig::new(true, "hold");
    let runs = prepare_pk(&rig, "pk");
    rig.manager.commit_reserved_match("pk", || Ok(())).unwrap();
    let mut request = rig.request(3000);
    request.identity.run_id = runs[1].run_id.clone();
    let handle = rig
        .manager
        .start_reserved_game_move("pk", request.clone())
        .unwrap();
    rig.wait_for(&handle.identity.job_id);
    let manager = rig.manager.clone();
    let pausing = std::thread::spawn(move || assert!(manager.pause_reserved_match("pk").is_err()));
    rig.wait_for("terminateId");
    rig.manager.stop_reserved_match("pk").unwrap();
    pausing.join().unwrap();
    assert!(handle.wait().is_err());
    assert_eq!(rig.manager.match_reservation_owner(), None);
    assert!(rig.manager.start_reserved_game_move("pk", request).is_err());
    for run in &runs {
        assert_pid_alive(&pk_pid(&rig, run), false);
    }
}

#[test]
fn match_stop_seals_reserved_human_analysis_and_keeps_it_unclaimable() {
    let rig = Rig::new(true, "analysis_hold");
    let request = commit_analysis_match(&rig, 3000);
    let handle = rig.manager.start_reserved_analysis("analysis", request).unwrap();
    rig.wait_for("reportDuringSearchEvery");
    rig.manager.stop_reserved_match("analysis").unwrap();
    assert_eq!(handle.wait().unwrap_err().kind, EngineFailureKind::Cancellation);
    assert!(rig.manager.snapshot().game_move_job.is_none());
    assert!(rig.manager.match_reservation_owner().is_none());
    rig.reaped();
}

#[test]
fn missing_match_profile_releases_reservation_before_any_process_io() {
    for missing_side in [None, Some(PlayerColor::Black), Some(PlayerColor::White)] {
        let rig = Rig::new(true, "normal");
        let request = rig.request(3000);
        let trace = rig.trace();
        rig.manager.reserve_match("missing-profile").unwrap();
        rig.manager.assert_profile_deletable("second").unwrap();
        let error = match missing_side {
            None => rig
                .manager
                .prepare_reserved_match(
                    "missing-profile",
                    "deleted",
                    &request.position,
                    request.identity.budget,
                )
                .unwrap_err(),
            Some(side) => {
                let ids = if side == PlayerColor::Black {
                    ["deleted", "second"]
                } else {
                    ["second", "deleted"]
                };
                let (failed_side, error) = rig
                    .manager
                    .prepare_reserved_pk(
                        "missing-profile",
                        [
                            (ids[0], request.identity.budget),
                            (ids[1], request.identity.budget),
                        ],
                        &request.position,
                    )
                    .unwrap_err();
                assert_eq!(failed_side, side);
                error
            }
        };
        assert_eq!(error.kind, EngineFailureKind::InvalidState);
        assert_eq!(error.profile_id.as_deref(), Some("deleted"));
        assert_eq!(rig.manager.match_reservation_owner(), None);
        assert_eq!(rig.trace(), trace);
        assert!(matches!(rig.manager.snapshot().lifecycle,
            ForegroundEngineLifecycleDto::Ready { run } if run.run_id == rig.run));
        rig.manager.reserve_match("retry").unwrap();
        rig.manager.abort_reserved_match("retry").unwrap();
        let result = rig.manager.start_game_move(request).unwrap().wait().unwrap();
        assert_eq!(
            result.result,
            GameMoveDto::Move {
                vertex: MoveVertex::Point(PointDto { x: 3, y: 1 })
            }
        );
    }
}
