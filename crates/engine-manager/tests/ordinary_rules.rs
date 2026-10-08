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
    assert!(!run.capability_snapshot.as_ref().unwrap().game_move);
    assert!(run.capability_snapshot.as_ref().unwrap().analysis.is_none());
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
    for (text, indices) in cases {
        let document = sgf::CurrentSgfDocument::open(text).unwrap();
        let node_path = NodePath { indices };
        let position = document.exact_position(&node_path).unwrap();
        let expected = position.dto().clone();
        let result = manager.confirm_ordinary_rules(GameMoveRequest {
            identity: GameMoveRequestDto {
                run_id: run.run_id.clone(),
                generation: 1,
                node_path,
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
    }
    manager.teardown().unwrap();
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
}
