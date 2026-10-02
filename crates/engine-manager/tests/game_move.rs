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
    dir: PathBuf,
    run: String,
    kata: bool,
}
impl Rig {
    fn new(kata: bool, mode: &str) -> Self {
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
            catalog,
            ForegroundEngineConfig {
                stop_drain_timeout: Duration::from_millis(150),
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
        }
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
