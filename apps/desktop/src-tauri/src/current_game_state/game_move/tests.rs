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
        Self::with_adapter(mode, false)
    }
    fn with_adapter(mode: &str, gtp: bool) -> Self {
        let directory = std::env::temp_dir().join(format!("move-holder-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(directory.join("mode"), mode).unwrap();
        std::fs::write(directory.join("model"), "").unwrap();
        std::fs::write(directory.join("config"), "").unwrap();
        let engine = directory.join("engine");
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(if gtp {
            "../../../crates/engine-manager/tests/fixtures/rules_process.py"
        } else {
            "../../../crates/engine-manager/tests/fixtures/game_move_process.py"
        });
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
                adapter: {
                    let settings = KataGoSettings {
                        model_path: Some("model".into()),
                        config_path: Some("config".into()),
                        max_visits: 8,
                    };
                    if gtp {
                        EngineAdapterSettings::KataGoGtp(settings)
                    } else {
                        EngineAdapterSettings::KataGoAnalysis(settings)
                    }
                },
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

#[test]
fn completed_results_stay_sealed_after_path_or_mode_round_trips() {
    for transition in ["path", "trial", "cancel", "departure"] {
        let rig = Rig::new("normal");
        let request = rig.request();
        let path = request.node_path.clone();
        let generation = request.generation;
        let result = rig
            .state
            .start_game_move(&rig.manager, request)
            .unwrap()
            .wait()
            .unwrap();
        let job_id = result.job_id.clone();
        match transition {
            "trial" => {
                let session = rig.state.enter_trial().unwrap();
                rig.state.exit_trial(session.session_id).unwrap();
            }
            "cancel" => rig
                .manager
                .cancel_game_move(&result.run_id, &result.job_id)
                .unwrap(),
            "departure" => {
                rig.manager.begin_continuous_departure();
                rig.manager.finish_continuous_departure(false);
            }
            _ => {
                rig.state.select_path(NodePath::default(), generation).unwrap();
                rig.state.select_path(path, generation).unwrap();
            }
        }
        let error = rig.state.publish_game_move(&rig.manager, result).unwrap_err();
        assert_eq!(error.kind, EngineFailureKind::Cancellation);
        assert_eq!(error.job_id.as_deref(), Some(job_id.as_str()));
    }
}

#[test]
fn ordinary_rules_gateway_preserves_authoritative_game_and_rejects_unsupported_setup() {
    let rig = Rig::with_adapter("stream", true);
    rig.state
        .replace("(;SZ[9]RU[Chinese]KM[7.5]C[personal];B[dd])", None)
        .unwrap();
    rig.state
        .select_path(NodePath::default(), rig.state.inspect().0)
        .unwrap();
    let request = || OrdinaryRulesRequestDto {
        run_id: rig.run.clone(),
        generation: rig.state.inspect().0,
        node_path: NodePath::default(),
    };
    let before = rig.state.serialize().unwrap();
    let receipt = rig
        .state
        .start_ordinary_rules(&rig.manager, request())
        .unwrap()
        .wait()
        .unwrap();
    let receipt = rig.state.publish_ordinary_rules(&rig.manager, receipt).unwrap();
    assert_eq!(receipt.position.komi, 7.5);
    assert_eq!(rig.state.serialize().unwrap(), before);
    assert_eq!(
        rig.state
            .publish_ordinary_rules(&rig.manager, receipt)
            .unwrap_err()
            .kind,
        EngineFailureKind::Cancellation
    );
    for text in [
        "(;SZ[9]RU[Chinese]AW[aa]C[keep])",
        "(;SZ[9]RU[Chinese]AB[aa][bb]AW[cc])",
        "(;SZ[9]RU[Chinese];B[aa];AB[bb])",
    ] {
        let loaded = rig.state.replace(text, None).unwrap();
        let before = rig.state.serialize().unwrap();
        let trace = std::fs::read(rig.directory.join("trace")).unwrap();
        let request = OrdinaryRulesRequestDto {
            run_id: rig.run.clone(),
            generation: loaded.generation,
            node_path: loaded.selected_path,
        };
        assert_eq!(
            rig.state
                .start_ordinary_rules(&rig.manager, request)
                .err()
                .unwrap()
                .kind,
            EngineFailureKind::UnsupportedCapability
        );
        assert_eq!(rig.state.serialize().unwrap(), before);
        assert_eq!(std::fs::read(rig.directory.join("trace")).unwrap(), trace);
        assert!(
            matches!(rig.manager.snapshot().lifecycle, ForegroundEngineLifecycleDto::Ready { ref run } if run.run_id == rig.run)
        );
    }
}

#[test]
fn ordinary_rules_gateway_rejects_completed_receipt_after_navigation_round_trip() {
    let rig = Rig::with_adapter("stream", true);
    rig.state
        .replace("(;SZ[9]RU[Chinese]KM[7.5];B[dd])", None)
        .unwrap();
    let generation = rig.state.inspect().0;
    rig.state.select_path(NodePath::default(), generation).unwrap();
    let receipt = rig
        .state
        .start_ordinary_rules(
            &rig.manager,
            OrdinaryRulesRequestDto {
                run_id: rig.run.clone(),
                generation,
                node_path: NodePath::default(),
            },
        )
        .unwrap()
        .wait()
        .unwrap();
    rig.state
        .select_path(NodePath { indices: vec![0] }, generation)
        .unwrap();
    rig.state.select_path(NodePath::default(), generation).unwrap();
    assert_eq!(
        rig.state
            .publish_ordinary_rules(&rig.manager, receipt)
            .unwrap_err()
            .kind,
        EngineFailureKind::Cancellation
    );
}

#[test]
fn gtp_main_analysis_binds_exact_history_attaches_and_rejects_replaced_document() {
    let rig = Rig::with_adapter("analysis", true);
    let loaded = rig.state.replace("(;SZ[9]RU[Chinese]KM[7.5]C[personal only])", None).unwrap();
    let events = rig.manager.subscribe();
    let request = crate::bind_selected_node_job(&rig.state, rig.run.clone(), loaded.generation,
        loaded.selected_path.clone(), AnalysisJobModeDto::Finite, Some(16)).unwrap();
    rig.manager.start_selected_node_job(request).unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    let completed = loop {
        match events.recv_timeout(deadline.saturating_duration_since(Instant::now())).unwrap() {
            ForegroundEngineEventDto::Job { job } if job.outcome == AnalysisJobOutcomeDto::Completed => break job,
            ForegroundEngineEventDto::Failure { failure } => panic!("{failure:?}"),
            _ => {},
        }
    };
    let attached = rig.state.attach_from_job_event(&completed).unwrap();
    assert_eq!(attached.snapshot.personal_comment, "personal only");
    let frame = attached.snapshot.primary_analysis.unwrap();
    assert_eq!(frame.winrate_black, 0.61);
    assert_eq!(frame.candidates[0].score_mean_black, None);
    assert_eq!(frame.candidates[0].pv.len(), 2);
    let persisted = rig.state.serialize().unwrap();
    let reopened = sgf::CurrentSgfDocument::open(&persisted).unwrap();
    let reopened = reopened.snapshot(&NodePath::default()).unwrap();
    assert_eq!(reopened.personal_comment, "personal only");
    assert_eq!(reopened.primary_analysis.unwrap().candidates[0].score_mean_black, None);
    rig.state.replace("(;SZ[9]RU[Chinese]KM[7.5]C[new document])", None).unwrap();
    let before = rig.state.serialize().unwrap();
    assert!(rig.state.attach_from_job_event(&completed).is_none());
    assert_eq!(rig.state.serialize().unwrap(), before);
}
