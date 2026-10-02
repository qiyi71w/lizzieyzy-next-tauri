use super::*;
use app_model::*;
use engine_manager::{
    EngineProfileCatalog, ForegroundEngineConfig, InMemoryEngineProfileCatalog, SavedEngineProfile,
};
use std::{
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

struct Rig {
    state: CurrentGameState,
    manager: ForegroundEngineManager,
    directory: PathBuf,
    run: String,
    catalog: Arc<InMemoryEngineProfileCatalog>,
}
impl Rig {
    fn new(mode: &str) -> Self {
        let directory = std::env::temp_dir().join(format!("move-holder-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(directory.join("mode"), mode).unwrap();
        std::fs::write(directory.join("model"), "").unwrap();
        std::fs::write(directory.join("config"), "").unwrap();
        let engine = directory.join("engine");
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../crates/engine-manager/tests/fixtures/game_move_process.py");
        std::fs::write(
            &engine,
            format!(
                "#!/bin/sh\nexec /usr/bin/python3 '{}' \"$@\"\n",
                fixture.display()
            ),
        )
        .unwrap();
        std::fs::set_permissions(&engine, std::fs::Permissions::from_mode(0o755)).unwrap();
        let catalog = Arc::new(InMemoryEngineProfileCatalog::new());
        catalog.upsert(SavedEngineProfile {
            profile_id: "engine".into(),
            profile: EngineProfileDto {
                name: "fixture".into(),
                program: engine.to_string_lossy().into(),
                argv: vec![],
                working_dir: Some(directory.to_string_lossy().into()),
                adapter: EngineAdapterSettings::KataGoAnalysis(KataGoSettings {
                    model_path: Some("model".into()),
                    config_path: Some("config".into()),
                    max_visits: 8,
                }),
            },
        });
        let manager = ForegroundEngineManager::new(catalog.clone(), ForegroundEngineConfig::for_tests());
        manager.start("engine").unwrap();
        let deadline = Instant::now() + Duration::from_secs(3);
        let run = loop {
            match manager.snapshot().lifecycle {
                ForegroundEngineLifecycleDto::Ready { run } => break run.run_id,
                ForegroundEngineLifecycleDto::Error { failure, .. } => panic!("{failure:?}"),
                _ => assert!(Instant::now() < deadline),
            }
            std::thread::sleep(Duration::from_millis(2));
        };
        let state = CurrentGameState::default();
        state.connect_analysis_manager(manager.clone());
        state
            .replace(
                "(;SZ[5]RU[Chinese-KGS]KM[6.5]C[my root];B[aa]C[my move];W[]C[my pass])",
                Some("/isolated/game.sgf".into()),
            )
            .unwrap();
        let generation = state.inspect().0;
        state
            .select_path(NodePath { indices: vec![0, 0] }, generation)
            .unwrap();
        Self {
            state,
            manager,
            directory,
            run,
            catalog,
        }
    }
    fn request(&self) -> GameMoveRequestDto {
        GameMoveRequestDto {
            run_id: self.run.clone(),
            generation: self.state.inspect().0,
            node_path: NodePath { indices: vec![0, 0] },
            budget: ComputeBudgetDto {
                deadline_ms: 3000,
                max_visits: Some(8),
            },
        }
    }
}
impl Drop for Rig {
    fn drop(&mut self) {
        self.manager.teardown().unwrap();
        let _ = std::fs::remove_dir_all(&self.directory);
    }
}

#[test]
fn move_success_and_refusal_leave_current_game_and_profile_untouched() {
    let rig = Rig::new("normal");
    let before = rig.state.inspect();
    let serialized = rig.state.serialize().unwrap();
    let defaults = rig.catalog.get("engine").unwrap();
    let selected = rig
        .state
        .holder
        .lock()
        .ok()
        .map(|holder| holder.selected_path.clone());
    let result = rig
        .state
        .start_game_move(&rig.manager, rig.request())
        .unwrap()
        .wait()
        .unwrap();
    let result = rig.state.publish_game_move(&rig.manager, result).unwrap();
    assert_eq!(
        result.result,
        GameMoveDto::Move {
            vertex: MoveVertex::Point(PointDto { x: 3, y: 1 })
        }
    );
    let mut unsupported = rig.request();
    unsupported.budget.max_visits = None;
    assert_eq!(
        rig.state
            .start_game_move(&rig.manager, unsupported)
            .err()
            .unwrap()
            .kind,
        EngineFailureKind::UnsupportedCapability
    );
    assert_eq!(rig.state.inspect(), before);
    assert_eq!(rig.state.serialize().unwrap(), serialized);
    assert_eq!(
        rig.state
            .holder
            .lock()
            .ok()
            .map(|holder| holder.selected_path.clone()),
        selected
    );
    assert_eq!(rig.catalog.get("engine").unwrap(), defaults);
}

#[test]
fn navigation_and_generation_changes_seal_pending_and_completed_results() {
    for pending in [false, true] {
        for generation_change in [false, true] {
            let rig = Rig::new(if pending { "hold" } else { "normal" });
            let mut active = Some(rig.state.start_game_move(&rig.manager, rig.request()).unwrap());
            let completed = if pending {
                None
            } else {
                Some(active.take().unwrap().wait().unwrap())
            };
            // Completed results still cross the holder publication guard after async waiting.
            if generation_change {
                rig.state
                    .replace(
                        "(;SZ[5]RU[Chinese-KGS]KM[7.5];B[aa];W[])",
                        Some("/isolated/changed.sgf".into()),
                    )
                    .unwrap();
            } else {
                rig.state
                    .select_path(NodePath::default(), rig.state.inspect().0)
                    .unwrap();
            }
            let state_after_user_action = rig.state.serialize().unwrap();
            if let Some(result) = completed {
                assert_eq!(
                    rig.state
                        .publish_game_move(&rig.manager, result)
                        .unwrap_err()
                        .kind,
                    EngineFailureKind::Cancellation
                );
            }
            if let Some(active) = active {
                assert_eq!(active.wait().unwrap_err().kind, EngineFailureKind::Cancellation);
            }
            assert_eq!(rig.state.serialize().unwrap(), state_after_user_action);
        }
    }
}
