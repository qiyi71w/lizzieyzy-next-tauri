use super::*;
use app_model::{
    AutoLoadPhaseDto as AutoPhase, AutoLoadRecoveryPhaseDto as RecoveryPhase, GameLoadSourceDto as Source,
};
use engine_manager::EngineProfileCatalog;
use std::time::{Duration, Instant};

fn pump(engine: &LiveFixture, state: &CurrentGameState, done: impl Fn() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(8);
    loop {
        if done() {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "snapshot={:?}; task={:?}; queries={:?}",
            engine.manager.snapshot(),
            engine.manager.analysis_task_snapshot(),
            std::fs::read_to_string(engine.directory.join("queries.jsonl"))
        );
        match engine.events.recv_timeout(Duration::from_millis(20)) {
            Ok(app_model::ForegroundEngineEventDto::Job { job }) => {
                state.attach_from_job_event(&job);
            }
            Ok(app_model::ForegroundEngineEventDto::Snapshot { .. }) => state.reconcile_auto_load(),
            Ok(_) | Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
            Err(error) => panic!("event stream: {error}"),
        }
    }
}

pub(super) fn load(
    engine: &LiveFixture,
    state: &CurrentGameState,
    text: &str,
    source: Source,
) -> app_model::CurrentGameResultDto {
    let admission = state.prepare_loaded_replacement(text, None, source).unwrap();
    let departure_id = match admission {
        app_model::DocumentDepartureAdmissionDto::Ready { departure_id }
        | app_model::DocumentDepartureAdmissionDto::NeedsDecision { departure_id } => departure_id,
    };
    let selected_path = state
        .holder
        .lock()
        .expect("current game state")
        .selected_path
        .clone();
    let jobs = crate::document_departure::jobs_from_snapshot(&engine.manager.snapshot());
    let result = crate::document_departure::resolve_replacement(
        state,
        crate::document_departure::ReplacementResolution {
            departure_id,
            action: app_model::DocumentDepartureActionDto::Discard,
            selected_path,
        },
        &jobs,
        |job| {
            engine
                .manager
                .cancel_job(&job.run_id, &job.job_id)
                .map_err(|e| e.message)
        },
        |job, budget| {
            engine
                .manager
                .wait_for_job_cancellation(&job.run_id, &job.job_id, budget)
                .map_err(|e| e.message)
        },
        || panic!("Discard must not Save"),
    )
    .unwrap();
    assert!(result.committed, "{}", result.message);
    result.current.unwrap()
}

fn setup() -> (LiveFixture, CurrentGameState) {
    let engine = task_fixture();
    std::fs::write(engine.directory.join("cancel-final"), "").unwrap();
    let state = CurrentGameState::default();
    state.connect_analysis_manager(engine.manager.clone());
    engine
        .manager
        .set_auto_load_preferences(app_model::AutoLoadPreferencesDto {
            enabled: true,
            total_visits: 1,
        })
        .unwrap();
    (engine, state)
}

#[test]
fn auto_load_refused_finite_capability_releases_ordinary_without_retry() {
    let (engine, state) = setup();
    engine
        .manager
        .set_continuous_preferences(true, app_model::ContinuousAnalysisBudgetDto::default())
        .unwrap();
    engine.manager.start("test").unwrap();
    pump(&engine, &state, || {
        matches!(
            engine.manager.snapshot().lifecycle,
            app_model::ForegroundEngineLifecycleDto::Ready { .. }
        )
    });
    let mut capability = match engine.manager.snapshot().lifecycle {
        app_model::ForegroundEngineLifecycleDto::Ready { run } => run.capability_snapshot.unwrap(),
        other => panic!("expected Ready, got {other:?}"),
    };
    capability.analysis.as_mut().unwrap().whole_game_analysis = false;
    engine
        .manager
        .set_capability_snapshot_for_tests(Some(capability.clone()));
    let opened = load(&engine, &state, "(;SZ[9];B[dd])", Source::Open);
    assert_eq!(engine.manager.snapshot().auto_load.phase, AutoPhase::Failed);
    pump(&engine, &state, || {
        engine.manager.snapshot().selected_node_job.is_some_and(|job| {
            job.generation == opened.generation && job.state == app_model::AnalysisJobStateDto::Searching
        })
    });
    capability.analysis.as_mut().unwrap().whole_game_analysis = true;
    engine.manager.set_capability_snapshot_for_tests(Some(capability));
    state.reconcile_auto_load();
    assert_eq!(engine.manager.snapshot().auto_load.phase, AutoPhase::Failed);
    assert!(engine.manager.analysis_task_snapshot().is_none());
    engine.manager.teardown().unwrap();
}

#[test]
fn auto_load_single_lane_replacement_completes_before_fresh_ordinary_handback() {
    for starting in [false, true] {
        let (engine, state) = setup();
        for marker in ["one-thread", "hold-first", "finish"] {
            std::fs::write(engine.directory.join(marker), "").unwrap();
        }
        engine
            .manager
            .set_continuous_preferences(true, app_model::ContinuousAnalysisBudgetDto::default())
            .unwrap();
        state.replace("(;SZ[9];B[aa])", None).unwrap();
        if starting {
            std::fs::write(engine.directory.join("hold-ready"), "").unwrap();
        }
        engine.manager.start("test").unwrap();
        let previous = if starting {
            None
        } else {
            pump(&engine, &state, || {
                engine
                    .manager
                    .snapshot()
                    .selected_node_job
                    .is_some_and(|job| job.state == app_model::AnalysisJobStateDto::Searching)
            });
            Some(engine.manager.snapshot().selected_node_job.unwrap().job_id)
        };
        let opened = load(&engine, &state, "(;SZ[9]C[personal];B[dd];W[ee])", Source::Recent);
        let selected = opened.selected_path.clone();
        if starting {
            assert_eq!(engine.manager.snapshot().auto_load.phase, AutoPhase::Waiting);
            std::fs::remove_file(engine.directory.join("hold-ready")).unwrap();
        }
        pump(&engine, &state, || {
            engine.manager.snapshot().auto_load.phase == AutoPhase::Complete
                && engine
                    .manager
                    .snapshot()
                    .selected_node_job
                    .is_some_and(|job| job.state == app_model::AnalysisJobStateDto::Searching)
        });
        let task = engine.manager.analysis_task_snapshot().unwrap();
        assert_eq!(task.state, app_model::AnalysisTaskStateDto::Completed);
        assert_eq!(
            task.completed,
            vec![
                NodePath::default(),
                NodePath { indices: vec![0] },
                NodePath { indices: vec![0, 0] }
            ]
        );
        let ordinary = engine.manager.snapshot().selected_node_job.unwrap();
        assert_ne!(previous.as_deref(), Some(ordinary.job_id.as_str()));
        assert_ne!(ordinary.job_id, task.job_id);
        assert_eq!(ordinary.run_id, task.run_id);
        assert_eq!(ordinary.generation, opened.generation);
        assert_eq!(ordinary.node_path, selected);
        let document = CurrentSgfDocument::open(&state.serialize().unwrap()).unwrap();
        assert!(document.missing_primary_mainline_snapshots().unwrap().is_empty());
        assert_eq!(
            document.snapshot(&NodePath::default()).unwrap().personal_comment,
            "personal"
        );
        assert_eq!(
            state
                .holder
                .lock()
                .unwrap()
                .current_result()
                .unwrap()
                .selected_path,
            selected
        );
        let complete = state.serialize().unwrap();
        let reopened = load(&engine, &state, &complete, Source::Recent);
        assert_eq!(engine.manager.snapshot().auto_load.phase, AutoPhase::NotNeeded);
        pump(&engine, &state, || {
            engine.manager.snapshot().selected_node_job.is_some_and(|job| {
                job.generation == reopened.generation
                    && job.state == app_model::AnalysisJobStateDto::Searching
            })
        });
        let excluded = load(&engine, &state, "(;SZ[9];B[cc])", Source::Excluded);
        assert_eq!(engine.manager.snapshot().auto_load.phase, AutoPhase::NotNeeded);
        pump(&engine, &state, || {
            engine.manager.snapshot().selected_node_job.is_some_and(|job| {
                job.generation == excluded.generation
                    && job.state == app_model::AnalysisJobStateDto::Searching
            })
        });
        engine.manager.teardown().unwrap();
    }
}

#[test]
fn finite_zero_deep_selection_publishes_released_owner_after_terminal_job() {
    let (engine, state) = setup();
    engine
        .manager
        .set_continuous_preferences(false, app_model::ContinuousAnalysisBudgetDto::default())
        .unwrap();
    std::fs::write(engine.directory.join("budget-smoke"), "swing_zero").unwrap();
    let opened = state.replace("(;SZ[9];B[dd])", None).unwrap();
    engine.manager.start("test").unwrap();
    pump(&engine, &state, || {
        matches!(
            engine.manager.snapshot().lifecycle,
            app_model::ForegroundEngineLifecycleDto::Ready { .. }
        )
    });
    let run_id = match engine.manager.snapshot().lifecycle {
        app_model::ForegroundEngineLifecycleDto::Ready { run } => run.run_id,
        other => panic!("expected Ready, got {other:?}"),
    };
    let preview = state
        .preview_analysis_scope(
            opened.generation,
            app_model::AnalysisScopeDto {
                mode: app_model::AnalysisScopeModeDto::FirstChildMainline,
                current_node: NodePath::default(),
                branch_choices: Vec::new(),
                interval: Some(app_model::AnalysisPositionIntervalDto { start: 1, end: 1 }),
                to_play: None,
            },
            Some(app_model::AnalysisSwingCriteriaDto {
                move_actors: app_model::AnalysisMoveActorFilterDto::Both,
                winrate_change_percentage_points: app_model::AnalysisSwingThresholdDto {
                    enabled: true,
                    value: 10.0,
                },
                score_change_points: app_model::AnalysisSwingThresholdDto {
                    enabled: false,
                    value: 3.0,
                },
            }),
        )
        .unwrap();
    let task = state
        .start_swing_analysis_task(
            &engine.manager,
            run_id,
            preview,
            app_model::AnalysisStageConditionsDto::default(),
            app_model::AnalysisStageConditionsDto::default(),
        )
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(8);
    let mut terminal = false;
    loop {
        assert!(
            Instant::now() < deadline,
            "zero-deep terminal snapshot was not published"
        );
        match engine.events.recv_timeout(Duration::from_millis(20)) {
            Ok(app_model::ForegroundEngineEventDto::Job { job }) => {
                state.attach_from_job_event(&job);
                if job.job_id == task.job_id && job.outcome == app_model::AnalysisJobOutcomeDto::Completed {
                    assert_eq!(job.expected, Some(0));
                    terminal = true;
                }
            }
            Ok(app_model::ForegroundEngineEventDto::Snapshot { snapshot }) if terminal => {
                assert!(snapshot.whole_game_job.is_none());
                assert_eq!(
                    snapshot.continuous.phase,
                    app_model::ContinuousAnalysisPhaseDto::Off
                );
                break;
            }
            Ok(_) | Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
            Err(error) => panic!("event stream: {error}"),
        }
    }
    let completed = engine.manager.analysis_task_snapshot().unwrap();
    assert_eq!(completed.state, app_model::AnalysisTaskStateDto::Completed);
    assert_eq!(completed.stage, app_model::AnalysisTaskStageDto::Deep);
    assert_eq!(completed.selected_for_deep, Some(Vec::new()));
    assert!(completed.completed.is_empty());
    assert_eq!(completed.overview_completed.len(), 2);
    assert_eq!(
        std::fs::read_to_string(engine.directory.join("queries.jsonl"))
            .unwrap()
            .lines()
            .count(),
        2
    );
    engine.manager.teardown().unwrap();
}

#[test]
fn auto_load_final_publishes_complete_before_idle_or_current_target_handback() {
    for (continuous, total_visits) in [(false, 1), (false, 8), (true, 8)] {
        let (engine, state) = setup();
        for marker in ["hold-first", "hold-ordinary"] {
            std::fs::write(engine.directory.join(marker), "").unwrap();
        }
        engine
            .manager
            .set_auto_load_preferences(app_model::AutoLoadPreferencesDto {
                enabled: true,
                total_visits,
            })
            .unwrap();
        engine
            .manager
            .set_continuous_preferences(continuous, app_model::ContinuousAnalysisBudgetDto::default())
            .unwrap();
        engine.manager.start("test").unwrap();
        pump(&engine, &state, || {
            matches!(
                engine.manager.snapshot().lifecycle,
                app_model::ForegroundEngineLifecycleDto::Ready { .. }
            )
        });
        if total_visits == 1 {
            std::fs::write(engine.directory.join("finish"), "").unwrap();
        }
        let opened = load(&engine, &state, "(;SZ[9]C[personal];B[dd];W[ee])", Source::Open);
        let task = engine.manager.analysis_task_snapshot().unwrap();
        let selected = if total_visits == 1 {
            opened.selected_path.clone()
        } else {
            let selected = NodePath { indices: vec![0, 0] };
            state.select_path(selected.clone(), opened.generation).unwrap();
            pump(&engine, &state, || {
                engine.directory.join("search-started").exists()
            });
            std::fs::write(engine.directory.join("finish"), "").unwrap();
            selected
        };
        // Once the child can finish, only consume the real reader event stream.
        // The one-visit case has no post-load navigation or other command at all.
        let deadline = Instant::now() + Duration::from_secs(8);
        let mut attached = Vec::new();
        let mut completed = false;
        let settled = loop {
            assert!(
                Instant::now() < deadline,
                "no settled automatic completion event; continuous={continuous}, terminal={completed}"
            );
            match engine.events.recv_timeout(Duration::from_millis(20)) {
                Ok(app_model::ForegroundEngineEventDto::Job { job }) => {
                    if job.job_id == task.job_id {
                        if job.frame.is_some() {
                            assert!(!completed, "final frame must precede terminal job");
                            attached.push(job.node_path.clone());
                        }
                        if job.outcome == app_model::AnalysisJobOutcomeDto::Completed {
                            assert_eq!(attached.len(), 3);
                            assert!(job.frame.is_none());
                            completed = true;
                        }
                    }
                    state.attach_from_job_event(&job);
                }
                Ok(app_model::ForegroundEngineEventDto::Snapshot { snapshot }) => {
                    state.reconcile_auto_load();
                    if snapshot.auto_load.task_id.as_deref() == Some(&task.task_id)
                        && snapshot.auto_load.phase == AutoPhase::Complete
                    {
                        assert!(completed, "terminal job must precede settled snapshot");
                        assert!(snapshot.whole_game_job.is_none());
                        if !continuous || snapshot.selected_node_job.is_some() {
                            break snapshot;
                        }
                    }
                }
                Ok(_) | Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
                Err(error) => panic!("event stream: {error}"),
            }
        };
        assert_eq!(
            attached,
            vec![
                NodePath::default(),
                NodePath { indices: vec![0] },
                NodePath { indices: vec![0, 0] }
            ]
        );
        let document = CurrentSgfDocument::open(&state.serialize().unwrap()).unwrap();
        assert!(document.missing_primary_mainline_snapshots().unwrap().is_empty());
        assert_eq!(
            document.snapshot(&NodePath::default()).unwrap().personal_comment,
            "personal"
        );
        assert_eq!(
            state
                .holder
                .lock()
                .unwrap()
                .current_result()
                .unwrap()
                .selected_path,
            selected
        );
        if continuous {
            let ordinary = settled.selected_node_job.unwrap();
            assert_eq!(ordinary.run_id, task.run_id);
            assert_eq!(ordinary.generation, opened.generation);
            assert_eq!(ordinary.node_path, selected);
            assert_eq!(ordinary.mode, app_model::AnalysisJobModeDto::Continuous);
            assert_ne!(ordinary.job_id, task.job_id);
        } else {
            assert_eq!(
                settled.continuous.phase,
                app_model::ContinuousAnalysisPhaseDto::Off
            );
            assert!(settled.selected_node_job.is_none());
        }
        engine.manager.teardown().unwrap();
    }
}

#[test]
fn auto_load_committed_sources_exclusions_reopen_and_no_latent_start() {
    let (engine, state) = setup();
    for source in [
        Source::Open,
        Source::Recent,
        Source::FileDelivery,
        Source::Clipboard,
        Source::Text,
    ] {
        let previous = engine.manager.snapshot().auto_load.load_id;
        let opened = load(&engine, &state, "(;SZ[9]C[personal];B[dd])", source);
        let status = engine.manager.snapshot().auto_load;
        assert!(status.load_id > previous);
        assert_eq!(status.generation, opened.generation);
        assert_eq!(status.phase, AutoPhase::Unavailable);
        assert!(engine.manager.analysis_task_snapshot().is_none());
    }
    let previous = engine.manager.snapshot().auto_load.load_id;
    assert!(state
        .prepare_loaded_replacement("not sgf", None, Source::Text)
        .is_err());
    let admission = state
        .prepare_loaded_replacement("(;SZ[13])", None, Source::Open)
        .unwrap();
    let id = match admission {
        app_model::DocumentDepartureAdmissionDto::Ready { departure_id }
        | app_model::DocumentDepartureAdmissionDto::NeedsDecision { departure_id } => departure_id,
    };
    state.cancel_replacement(id).unwrap();
    assert_eq!(engine.manager.snapshot().auto_load.load_id, previous);
    load(&engine, &state, "(;SZ[9])", Source::Excluded);
    assert_eq!(engine.manager.snapshot().auto_load.load_id, previous);
    engine.manager.start("test").unwrap();
    pump(&engine, &state, || {
        matches!(
            engine.manager.snapshot().lifecycle,
            app_model::ForegroundEngineLifecycleDto::Ready { .. }
        )
    });
    state.reconcile_auto_load();
    assert!(engine.manager.analysis_task_snapshot().is_none());
    let first = load(&engine, &state, "(;SZ[9];B[dd])", Source::Open);
    assert_eq!(engine.manager.snapshot().auto_load.phase, AutoPhase::Running);
    let first_task = engine.manager.analysis_task_snapshot().unwrap();
    let second = load(&engine, &state, "(;SZ[9];B[dd])", Source::Open);
    assert!(second.generation > first.generation);
    assert_ne!(
        engine.manager.analysis_task_snapshot().unwrap().task_id,
        first_task.task_id
    );
}

#[test]
fn auto_load_waits_only_for_captured_start_and_pause_retires_wait() {
    let (engine, state) = setup();
    std::fs::write(engine.directory.join("hold-ready"), "").unwrap();
    engine.manager.start("test").unwrap();
    load(&engine, &state, "(;SZ[9];B[dd])", Source::Text);
    assert_eq!(engine.manager.snapshot().auto_load.phase, AutoPhase::Waiting);
    let run = engine.manager.snapshot().auto_load.run_id;
    engine.manager.retire_auto_load().unwrap();
    std::fs::remove_file(engine.directory.join("hold-ready")).unwrap();
    pump(&engine, &state, || {
        matches!(
            engine.manager.snapshot().lifecycle,
            app_model::ForegroundEngineLifecycleDto::Ready { .. }
        )
    });
    state.reconcile_auto_load();
    assert_eq!(engine.manager.snapshot().auto_load.phase, AutoPhase::Cancelled);
    assert!(engine.manager.analysis_task_snapshot().is_none());
    assert_eq!(engine.manager.snapshot().auto_load.run_id, run);
    load(&engine, &state, "(;SZ[9];B[dd])", Source::Text);
    assert_eq!(engine.manager.snapshot().auto_load.phase, AutoPhase::Running);
}

#[test]
fn auto_load_starting_ready_freezes_missing_exact_paths_and_partial_save_without_cursor_changes() {
    let (engine, state) = setup();
    engine
        .manager
        .set_auto_load_preferences(app_model::AutoLoadPreferencesDto {
            enabled: true,
            total_visits: 8,
        })
        .unwrap();
    std::fs::write(engine.directory.join("hold-ready"), "").unwrap();
    engine.manager.start("test").unwrap();
    let opened = load(
        &engine,
        &state,
        "(;SZ[9]AB[aa]PL[W]C[personal root];W[];AB[cc]PL[B];B[dd](;W[ee])(;W[ff]))",
        Source::FileDelivery,
    );
    assert_eq!(engine.manager.snapshot().auto_load.phase, AutoPhase::Waiting);
    engine
        .manager
        .set_continuous_preferences(true, app_model::ContinuousAnalysisBudgetDto::default())
        .unwrap();
    let selected = NodePath {
        indices: vec![0, 0, 0, 1],
    };
    state.select_path(selected.clone(), opened.generation).unwrap();
    std::fs::remove_file(engine.directory.join("hold-ready")).unwrap();
    pump(&engine, &state, || {
        CurrentSgfDocument::open(&state.serialize().unwrap())
            .unwrap()
            .snapshot(&NodePath::default())
            .unwrap()
            .primary_analysis
            .is_some()
    });
    let task = engine.manager.analysis_task_snapshot().unwrap();
    assert_eq!(
        task.requested
            .iter()
            .map(|p| p.indices.clone())
            .collect::<Vec<_>>(),
        vec![vec![], vec![0], vec![0, 0], vec![0, 0, 0], vec![0, 0, 0, 0]]
    );
    assert_eq!(task.completed.len(), 1);
    assert_eq!(
        state
            .holder
            .lock()
            .unwrap()
            .current_result()
            .unwrap()
            .selected_path,
        selected
    );
    let path = engine.directory.join("partial.sgf");
    state
        .save_to_path(path.to_string_lossy().into(), selected.clone())
        .unwrap();
    let partial = CurrentSgfDocument::open(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(
        partial.snapshot(&NodePath::default()).unwrap().personal_comment,
        "personal root"
    );
    assert_eq!(partial.missing_primary_mainline_snapshots().unwrap().len(), 4);
    std::fs::write(engine.directory.join("finish"), "").unwrap();
    pump(&engine, &state, || {
        CurrentSgfDocument::open(&state.serialize().unwrap())
            .unwrap()
            .missing_primary_mainline_snapshots()
            .unwrap()
            .is_empty()
    });
    assert_eq!(
        state
            .holder
            .lock()
            .unwrap()
            .current_result()
            .unwrap()
            .selected_path,
        selected
    );
    let complete = state.serialize().unwrap();
    load(&engine, &state, &complete, Source::Recent);
    assert_eq!(engine.manager.snapshot().auto_load.phase, AutoPhase::NotNeeded);
    assert_eq!(
        engine.manager.analysis_task_snapshot().unwrap().task_id,
        task.task_id
    );
}

#[test]
fn auto_load_pause_cancel_late_final_and_explicit_owner_priority() {
    let (engine, state) = setup();
    std::fs::write(engine.directory.join("hold-first"), "").unwrap();
    engine.manager.start("test").unwrap();
    pump(&engine, &state, || {
        matches!(
            engine.manager.snapshot().lifecycle,
            app_model::ForegroundEngineLifecycleDto::Ready { .. }
        )
    });
    let opened = load(&engine, &state, "(;SZ[9];B[dd];W[ee])", Source::Text);
    let old = engine.manager.analysis_task_snapshot().unwrap();
    engine
        .manager
        .pause_analysis_task(&old.run_id, &old.task_id)
        .unwrap();
    pump(&engine, &state, || {
        engine.manager.snapshot().whole_game_job.is_none()
    });
    assert_eq!(
        engine.manager.analysis_task_snapshot().unwrap().state,
        app_model::AnalysisTaskStateDto::Cancelled
    );
    assert!(state.serialize().unwrap().find("LZ[").is_none());
    assert!(engine
        .manager
        .continue_analysis_task(&old.run_id, &old.task_id, opened.generation)
        .is_err());
    let next = load(&engine, &state, "(;SZ[9];B[dd];W[ee])", Source::Open);
    let auto = engine.manager.analysis_task_snapshot().unwrap();
    state
        .start_first_child_analysis(&engine.manager, auto.run_id.clone(), next.generation, 32)
        .unwrap();
    let explicit = engine.manager.analysis_task_snapshot().unwrap();
    assert_ne!(auto.task_id, explicit.task_id);
    assert_eq!(engine.manager.snapshot().auto_load.phase, AutoPhase::Cancelled);
    assert_eq!(explicit.conditions.total_visits.value, 32);
}

#[test]
fn auto_load_failed_handback_import_restart_confirms_new_reader_before_continue() {
    let (engine, state) = setup();
    std::fs::write(engine.directory.join("auto-recovery"), "").unwrap();
    std::fs::write(engine.directory.join("hold-ordinary"), "").unwrap();
    std::fs::write(engine.directory.join("finish"), "").unwrap();
    engine
        .manager
        .set_continuous_preferences(true, app_model::ContinuousAnalysisBudgetDto::default())
        .unwrap();
    engine.manager.start("test").unwrap();
    pump(&engine, &state, || {
        matches!(
            engine.manager.snapshot().lifecycle,
            app_model::ForegroundEngineLifecycleDto::Ready { .. }
        )
    });
    load(
        &engine,
        &state,
        "(;SZ[9]RU[Chinese]KM[7.5]C[old];B[dd])",
        Source::Open,
    );
    pump(&engine, &state, || {
        engine.manager.snapshot().auto_load.phase == AutoPhase::Complete
    });
    let old_auto = engine.manager.analysis_task_snapshot().unwrap();
    std::fs::write(engine.directory.join("fail-handback"), "").unwrap();
    pump(&engine, &state, || {
        matches!(
            engine.manager.snapshot().lifecycle,
            app_model::ForegroundEngineLifecycleDto::Error { .. }
        )
    });
    assert_eq!(
        engine.manager.snapshot().auto_load_recovery,
        Some(RecoveryPhase::RestartRequired)
    );
    std::fs::remove_file(engine.directory.join("fail-handback")).unwrap();
    let imported = load(
        &engine,
        &state,
        "(;SZ[13]RU[Chinese-KGS]KM[6.5]AB[aa]PL[W]C[new personal];W[];B[cc])",
        Source::Text,
    );
    let bytes = state.serialize().unwrap();
    assert!(engine.manager.resume_continuous().is_err());
    engine.manager.restart().unwrap();
    pump(&engine, &state, || {
        engine.directory.join("recovery-started").exists()
    });
    let confirming = engine.manager.snapshot();
    assert_eq!(confirming.auto_load_recovery, Some(RecoveryPhase::Confirming));
    assert_eq!(
        confirming.continuous.phase,
        app_model::ContinuousAnalysisPhaseDto::SafetyHold
    );
    let request = confirming.selected_node_job.unwrap();
    assert_ne!(request.run_id, old_auto.run_id);
    assert_eq!(request.generation, imported.generation);
    assert_eq!(request.node_path, imported.selected_path);
    assert!(engine.manager.resume_continuous().is_err());
    engine.manager.authorize_continuous_start();
    assert_eq!(
        engine.manager.snapshot().continuous.phase,
        app_model::ContinuousAnalysisPhaseDto::SafetyHold
    );
    std::fs::write(engine.directory.join("recovery-release"), "").unwrap();
    pump(&engine, &state, || {
        engine.manager.snapshot().auto_load_recovery == Some(RecoveryPhase::Confirmed)
    });
    assert_eq!(state.serialize().unwrap(), bytes);
    let queries = std::fs::read_to_string(engine.directory.join("queries.jsonl")).unwrap();
    let confirmation: serde_json::Value = queries
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
        .find(|q| q["id"] == request.job_id)
        .unwrap();
    assert_eq!(confirmation["komi"], 6.5);
    assert_eq!(confirmation["boardXSize"], 13);
    assert_eq!(confirmation["rules"], "chinese-kgs");
    assert!(confirmation["initialStones"].as_array().unwrap().len() >= 2);
    engine.manager.resume_continuous().unwrap();
    assert!(engine.manager.snapshot().auto_load_recovery.is_none());
    assert_eq!(
        engine.manager.analysis_task_snapshot().unwrap().task_id,
        old_auto.task_id
    );
    assert_ne!(
        engine.manager.analysis_task_snapshot().unwrap().state,
        app_model::AnalysisTaskStateDto::Searching
    );
    let current = engine.manager.snapshot().selected_node_job.unwrap();
    assert_eq!(current.generation, imported.generation);
    assert_eq!(current.node_path, imported.selected_path);
    assert_eq!(current.mode, app_model::AnalysisJobModeDto::Continuous);
}

#[test]
fn auto_load_preferences_roundtrip_invalid_and_disk_failure_keep_old_settings_without_start() {
    let engine = task_fixture();
    let state = CurrentGameState::default();
    state.connect_analysis_manager(engine.manager.clone());
    let preferences = crate::continuous_analysis::PreferencesState::default();
    let path = engine.directory.join("preferences.json");
    let original = preferences.load(&path, &engine.manager).unwrap().preferences;
    assert_eq!(
        original.auto_load,
        app_model::AutoLoadPreferencesDto {
            enabled: false,
            total_visits: 1
        }
    );
    load(&engine, &state, "(;SZ[9];B[dd])", Source::Text);
    assert_eq!(engine.manager.snapshot().auto_load.phase, AutoPhase::Idle);
    let before = engine.manager.snapshot().auto_load;
    let mut edited = original.clone();
    edited.auto_load = app_model::AutoLoadPreferencesDto {
        enabled: true,
        total_visits: 7,
    };
    let saved = preferences.save(&path, &engine.manager, edited.clone()).unwrap();
    assert_eq!(engine.manager.snapshot().auto_load, before);
    assert_eq!(
        saved.task_single_stage_conditions,
        original.task_single_stage_conditions
    );
    assert_eq!(
        app_preferences::load_from_path(&path)
            .unwrap()
            .preferences
            .auto_load,
        edited.auto_load
    );
    let bytes = std::fs::read(&path).unwrap();
    edited.auto_load.total_visits = 0;
    assert!(preferences.save(&path, &engine.manager, edited.clone()).is_err());
    edited.auto_load.total_visits = 99;
    assert!(preferences
        .save(&engine.directory, &engine.manager, edited)
        .is_err());
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    assert_eq!(
        preferences
            .load(&path, &engine.manager)
            .unwrap()
            .preferences
            .auto_load,
        saved.auto_load
    );
    load(&engine, &state, "(;SZ[9];B[cc])", Source::Open);
    assert_eq!(engine.manager.snapshot().auto_load.total_visits, 7);
    assert_eq!(engine.manager.snapshot().auto_load.phase, AutoPhase::Unavailable);
    assert!(engine.manager.analysis_task_snapshot().is_none());
}

#[test]
fn auto_load_departure_cancel_preserves_owner_but_semantic_change_and_new_run_retire_it() {
    let (engine, state) = setup();
    engine
        .manager
        .set_auto_load_preferences(app_model::AutoLoadPreferencesDto {
            enabled: true,
            total_visits: 8,
        })
        .unwrap();
    std::fs::write(engine.directory.join("hold-first"), "").unwrap();
    engine.manager.start("test").unwrap();
    pump(&engine, &state, || {
        matches!(
            engine.manager.snapshot().lifecycle,
            app_model::ForegroundEngineLifecycleDto::Ready { .. }
        )
    });
    let opened = load(&engine, &state, "(;SZ[9]KM[7.5];B[dd])", Source::Open);
    let task = engine.manager.analysis_task_snapshot().unwrap();
    let admission = state
        .prepare_loaded_replacement("(;SZ[13])", None, Source::Open)
        .unwrap();
    let departure_id = match admission {
        app_model::DocumentDepartureAdmissionDto::Ready { departure_id }
        | app_model::DocumentDepartureAdmissionDto::NeedsDecision { departure_id } => departure_id,
    };
    state.cancel_replacement(departure_id).unwrap();
    assert_eq!(
        engine.manager.analysis_task_snapshot().unwrap().task_id,
        task.task_id
    );
    assert_eq!(engine.manager.snapshot().auto_load.phase, AutoPhase::Running);
    state
        .set_metadata(opened.generation, "B".into(), "W".into(), 6.5)
        .unwrap();
    pump(&engine, &state, || {
        engine.manager.snapshot().whole_game_job.is_none()
    });
    assert_eq!(engine.manager.snapshot().auto_load.phase, AutoPhase::Cancelled);
    engine.manager.stop().unwrap();
    pump(&engine, &state, || {
        matches!(
            engine.manager.snapshot().lifecycle,
            app_model::ForegroundEngineLifecycleDto::NoEngine { .. }
        )
    });
    std::fs::write(engine.directory.join("hold-ready"), "").unwrap();
    engine.manager.start("test").unwrap();
    load(&engine, &state, "(;SZ[9];B[cc])", Source::Open);
    assert_eq!(engine.manager.snapshot().auto_load.phase, AutoPhase::Waiting);
    engine.manager.stop().unwrap();
    std::fs::remove_file(engine.directory.join("hold-ready")).unwrap();
    pump(&engine, &state, || {
        matches!(
            engine.manager.snapshot().lifecycle,
            app_model::ForegroundEngineLifecycleDto::NoEngine { .. }
        )
    });
    engine.manager.start("test").unwrap();
    pump(&engine, &state, || {
        matches!(
            engine.manager.snapshot().lifecycle,
            app_model::ForegroundEngineLifecycleDto::Ready { .. }
        )
    });
    assert_eq!(engine.manager.snapshot().auto_load.phase, AutoPhase::Cancelled);
    assert!(engine.manager.snapshot().whole_game_job.is_none());
}

fn failed_handback_fixture() -> (LiveFixture, CurrentGameState) {
    let (engine, state) = setup();
    for marker in ["auto-recovery", "hold-ordinary", "finish"] {
        std::fs::write(engine.directory.join(marker), "").unwrap();
    }
    engine
        .manager
        .set_continuous_preferences(true, app_model::ContinuousAnalysisBudgetDto::default())
        .unwrap();
    engine.manager.start("test").unwrap();
    pump(&engine, &state, || {
        matches!(
            engine.manager.snapshot().lifecycle,
            app_model::ForegroundEngineLifecycleDto::Ready { .. }
        )
    });
    load(&engine, &state, "(;SZ[9];B[dd])", Source::Open);
    pump(&engine, &state, || {
        engine.manager.snapshot().auto_load.phase == AutoPhase::Complete
    });
    std::fs::write(engine.directory.join("fail-handback"), "").unwrap();
    pump(&engine, &state, || {
        matches!(
            engine.manager.snapshot().lifecycle,
            app_model::ForegroundEngineLifecycleDto::Error { .. }
        )
    });
    std::fs::remove_file(engine.directory.join("fail-handback")).unwrap();
    assert_eq!(
        engine.manager.snapshot().auto_load_recovery,
        Some(RecoveryPhase::RestartRequired)
    );
    (engine, state)
}

#[test]
fn auto_load_recovery_cancellation_new_document_rules_position_run_and_failed_confirmation_never_unlock() {
    for action in [
        "cancel",
        "document",
        "rules",
        "position",
        "run",
        "confirmation_failure",
        "restart_failure",
    ] {
        let (engine, state) = failed_handback_fixture();
        let imported = load(
            &engine,
            &state,
            "(;SZ[13]RU[Chinese-KGS]KM[6.5];B[cc];W[])",
            Source::Text,
        );
        if action == "restart_failure" {
            let mut profile = engine.catalog.get("test").unwrap();
            profile.profile.program = engine.directory.join("missing-engine").to_string_lossy().into();
            engine.catalog.upsert(profile);
        }
        engine.manager.restart().unwrap();
        if action == "restart_failure" {
            pump(&engine, &state, || {
                matches!(
                    engine.manager.snapshot().lifecycle,
                    app_model::ForegroundEngineLifecycleDto::NoEngine { failure: Some(_) }
                        | app_model::ForegroundEngineLifecycleDto::Error { .. }
                )
            });
            assert!(engine.manager.resume_continuous().is_err());
            continue;
        }
        pump(&engine, &state, || {
            engine.directory.join("recovery-started").exists()
        });
        assert!(engine.manager.resume_continuous().is_err());
        match action {
            "cancel" => engine.manager.retire_auto_load().unwrap(),
            "document" => {
                load(&engine, &state, "(;SZ[9]KM[5.5];B[ee])", Source::Open);
            }
            "rules" => {
                state
                    .set_metadata(imported.generation, "B".into(), "W".into(), 5.5)
                    .unwrap();
            }
            "position" => {
                state
                    .select_path(NodePath::default(), imported.generation)
                    .unwrap();
            }
            "run" => {
                engine.manager.stop().unwrap();
                pump(&engine, &state, || {
                    matches!(
                        engine.manager.snapshot().lifecycle,
                        app_model::ForegroundEngineLifecycleDto::NoEngine { .. }
                    )
                });
                engine.manager.start("test").unwrap();
                pump(&engine, &state, || {
                    matches!(
                        engine.manager.snapshot().lifecycle,
                        app_model::ForegroundEngineLifecycleDto::Ready { .. }
                    )
                });
            }
            "confirmation_failure" => {
                std::fs::write(engine.directory.join("recovery-fail"), "").unwrap();
            }
            _ => unreachable!(),
        }
        std::fs::write(engine.directory.join("recovery-release"), "").unwrap();
        pump(&engine, &state, || {
            engine.manager.snapshot().selected_node_job.is_none()
        });
        assert!(engine.manager.resume_continuous().is_err(), "{action}");
        engine.manager.authorize_continuous_start();
        assert!(engine.manager.snapshot().selected_node_job.is_none(), "{action}");
        assert_ne!(
            engine.manager.snapshot().auto_load_recovery,
            Some(RecoveryPhase::Confirmed),
            "{action}"
        );
    }
}

#[path = "auto_load_cpu_smoke.rs"]
mod cpu_smoke;
