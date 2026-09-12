use super::*;
use engine_manager::{ForegroundEngineConfig, ForegroundEngineManager, InMemoryEngineProfileCatalog};
use std::sync::Arc;

#[test]
fn confirmed_departure_hold_survives_navigation_and_preference_reload() {
    let manager = ForegroundEngineManager::new(
        Arc::new(InMemoryEngineProfileCatalog::new()),
        ForegroundEngineConfig::for_tests(),
    );
    let state = CurrentGameState::default();
    state.connect_analysis_manager(manager.clone());
    manager
        .set_continuous_preferences(true, app_model::ContinuousAnalysisBudgetDto::default())
        .unwrap();
    let opened = state.replace("(;SZ[9];B[dd])", None).unwrap();
    let admission = state.prepare_replacement("(;SZ[9])", None).unwrap();
    let id = match admission {
        app_model::DocumentDepartureAdmissionDto::Ready { departure_id }
        | app_model::DocumentDepartureAdmissionDto::NeedsDecision { departure_id } => departure_id,
    };
    state.begin_protected_commit(id, &[]).unwrap();
    assert_eq!(
        manager.snapshot().continuous.phase,
        app_model::ContinuousAnalysisPhaseDto::Departing
    );
    state.abort_protected_commit(id, opened.selected_path).unwrap();
    state.select_path(NodePath { indices: vec![] }).unwrap();
    manager
        .set_continuous_preferences(false, app_model::ContinuousAnalysisBudgetDto::default())
        .unwrap();
    manager
        .set_continuous_preferences(true, app_model::ContinuousAnalysisBudgetDto::default())
        .unwrap();
    assert_eq!(
        manager.snapshot().continuous.phase,
        app_model::ContinuousAnalysisPhaseDto::SafetyHold
    );
    assert!(manager.resume_continuous().is_err());
    assert_eq!(
        manager.snapshot().continuous.phase,
        app_model::ContinuousAnalysisPhaseDto::SafetyHold
    );
    assert!(matches!(
        manager.snapshot().lifecycle,
        app_model::ForegroundEngineLifecycleDto::NoEngine { .. }
    ));
}

#[cfg(unix)]
struct LiveFixture {
    directory: std::path::PathBuf,
    manager: ForegroundEngineManager,
    events: std::sync::mpsc::Receiver<app_model::ForegroundEngineEventDto>,
}

#[cfg(unix)]
impl LiveFixture {
    fn new() -> Self {
        use std::os::unix::fs::PermissionsExt;
        let directory = std::env::temp_dir().join(format!("continuous-holder-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&directory).unwrap();
        let executable = directory.join("engine");
        std::fs::write(
            &executable,
            r##"#!/usr/bin/env python3
import json, sys, time, pathlib
for line in sys.stdin:
    q = json.loads(line)
    with open("queries.jsonl", "a") as log:
        log.write(json.dumps(q) + "\n")
    if q.get("action") == "terminate":
        assert "terminateId" in q and q["id"] != q["terminateId"]
        print(json.dumps(dict(id=q["id"], action="terminate")), flush=True)
        while not pathlib.Path("release").exists():
            time.sleep(0.005)
        print(json.dumps(dict(id=q["terminateId"], isDuringSearch=False, noResults=True)), flush=True)
    else:
        limit = q.get("overrideSettings", {}).get("maxVisits", q.get("maxVisits", 0))
        if limit in (1, 32):
            pathlib.Path("whole-started").touch()
            while not pathlib.Path("whole-release").exists():
                time.sleep(0.005)
            print(json.dumps(dict(id=q["id"], isDuringSearch=False, turnNumber=0,
                rootInfo=dict(visits=limit, winrate=0.6, scoreMean=2.5),
                moveInfos=[dict(move="E5", order=0, visits=limit, winrate=0.6, scoreMean=2.5)])), flush=True)
            continue
        for visits in [8,16]:
            print(json.dumps(dict(id=q["id"], isDuringSearch=True, turnNumber=0,
                rootInfo=dict(visits=visits, winrate=0.6, scoreMean=2.5),
                moveInfos=[dict(move="E5", order=0, visits=visits, winrate=0.6, scoreMean=2.5)])), flush=True)
"##,
        )
        .unwrap();
        std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o755)).unwrap();
        std::fs::write(directory.join("model"), "").unwrap();
        std::fs::write(directory.join("config"), "").unwrap();
        let catalog = Arc::new(InMemoryEngineProfileCatalog::new());
        catalog.upsert(engine_manager::SavedEngineProfile {
            profile_id: "test".into(),
            profile: app_model::EngineProfileDto {
                name: "test".into(),
                engine_path: executable.to_string_lossy().into(),
                model_path: Some("model".into()),
                config_path: Some("config".into()),
                working_dir: Some(directory.to_string_lossy().into()),
                backend: app_model::EngineBackend::KataGoAnalysis,
            },
        });
        let manager = ForegroundEngineManager::new(catalog, ForegroundEngineConfig::for_tests());
        let events = manager.subscribe();
        Self {
            directory,
            manager,
            events,
        }
    }

    fn progress(&self, previous: Option<&str>) -> app_model::AnalysisJobEventDto {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(4);
        loop {
            let event = self
                .events
                .recv_timeout(deadline.saturating_duration_since(std::time::Instant::now()))
                .unwrap();
            if let app_model::ForegroundEngineEventDto::Job { job } = event {
                if job.outcome == app_model::AnalysisJobOutcomeDto::Progress
                    && previous != Some(job.job_id.as_str())
                {
                    return job;
                }
            }
        }
    }
}

#[cfg(unix)]
impl Drop for LiveFixture {
    fn drop(&mut self) {
        let _ = self.manager.teardown();
        let _ = std::fs::remove_dir_all(&self.directory);
    }
}

#[cfg(unix)]
fn task_fixture() -> LiveFixture {
    let engine = LiveFixture::new();
    let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../tests/fixtures/analysis_task_engine.py");
    std::fs::write(
        engine.directory.join("engine"),
        format!(
            "#!/bin/sh\nexport TASK_ENGINE_DIR='{}'\nexec python3 -u '{}'\n",
            engine.directory.display(),
            script.display()
        ),
    )
    .unwrap();
    engine
}

#[cfg(unix)]
fn start_live_task(
    engine: &LiveFixture,
    state: &CurrentGameState,
    generation: u64,
) -> app_model::AnalysisTaskDto {
    engine.manager.start("test").unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(4);
    let run_id = loop {
        if let app_model::ForegroundEngineLifecycleDto::Ready { run } = engine.manager.snapshot().lifecycle {
            break run.run_id;
        }
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(5));
    };
    let job = state
        .start_first_child_analysis(&engine.manager, run_id, generation, 32)
        .unwrap();
    let task = engine.manager.analysis_task_snapshot().unwrap();
    assert_eq!(job.job_id, task.job_id);
    task
}

#[cfg(unix)]
fn wait_live_task(
    engine: &LiveFixture,
    expected: app_model::AnalysisTaskStateDto,
) -> app_model::AnalysisTaskDto {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(4);
    loop {
        let task = engine.manager.analysis_task_snapshot().unwrap();
        if task.state == expected {
            return task;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "expected {expected:?}, got {task:?}"
        );
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

#[cfg(unix)]
fn wait_live_task_stage(
    engine: &LiveFixture,
    expected_stage: app_model::AnalysisTaskStageDto,
    expected_state: app_model::AnalysisTaskStateDto,
) -> app_model::AnalysisTaskDto {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(4);
    loop {
        let task = engine.manager.analysis_task_snapshot().unwrap();
        if task.stage == expected_stage && task.state == expected_state {
            return task;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "expected {expected_stage:?}/{expected_state:?}, got {task:?}"
        );
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

#[cfg(unix)]
#[test]
fn analysis_task_pause_continue_controllable_engine_smoke() {
    use app_model::AnalysisTaskStateDto;
    let engine = task_fixture();
    let state = CurrentGameState::default();
    state.connect_analysis_manager(engine.manager.clone());
    let opened = state.replace("(;SZ[9];B[dd];W[ee])", None).unwrap();
    let task = start_live_task(&engine, &state, opened.generation);
    let first = engine.progress(None);
    assert!(state.attach_from_job_event(&first).is_some());
    wait_live_task(&engine, AnalysisTaskStateDto::Searching);
    let pausing = state
        .pause_analysis_task(&engine.manager, &task.run_id, &task.task_id)
        .unwrap();
    assert_eq!(pausing.state, AnalysisTaskStateDto::Pausing);
    assert!(state.attach_from_job_event(&first).is_none());
    assert!(state
        .continue_analysis_task(&engine.manager, &task.run_id, &task.task_id)
        .is_err());
    std::fs::write(engine.directory.join("cancel-final"), "").unwrap();
    let paused = wait_live_task(&engine, AnalysisTaskStateDto::Paused);
    assert_eq!(paused.completed, vec![NodePath::default()]);
    state.select_path(NodePath::default()).unwrap();
    state
        .set_personal_comment(NodePath::default(), "paused review note".into())
        .unwrap();
    let saved_path = engine.directory.join("paused.sgf").to_string_lossy().into_owned();
    let saved = state
        .save_to_path(saved_path.clone(), NodePath::default())
        .unwrap();
    assert!(!saved.dirty);
    assert_eq!(saved.generation, task.generation);
    assert_eq!(engine.manager.analysis_task_snapshot().unwrap(), paused);
    assert!(
        state.attach_from_job_event(&first).is_none(),
        "late pre-Pause frame cannot dirty the saved document"
    );
    let envelope = state
        .recovery_flush(app_model::ApplicationExitDispositionDto::ExitIncomplete)
        .unwrap();
    let recovered = CurrentGameState::default();
    let recovered_manager = ForegroundEngineManager::new(
        Arc::new(InMemoryEngineProfileCatalog::new()),
        ForegroundEngineConfig::for_tests(),
    );
    recovered.connect_analysis_manager(recovered_manager.clone());
    recovered
        .replace(&envelope.sgf_text, envelope.source_path)
        .unwrap();
    assert!(recovered_manager.analysis_task_snapshot().is_none());
    assert!(recovered.analysis_jobs().unwrap().is_empty());
    assert!(matches!(
        recovered_manager.snapshot().lifecycle,
        app_model::ForegroundEngineLifecycleDto::NoEngine { .. }
    ));
    let continued = state
        .continue_analysis_task(&engine.manager, &task.run_id, &task.task_id)
        .unwrap();
    assert_eq!(continued.task_id, task.task_id);
    assert_ne!(continued.job_id, task.job_id);
    std::fs::write(engine.directory.join("finish"), "").unwrap();
    let mut attached = Vec::new();
    while attached.len() < 2 {
        let event = engine.progress(Some(&task.job_id));
        if let Some(game) = state.attach_from_job_event(&event) {
            assert!(game.dirty);
            attached.push(event.node_path.indices);
        }
    }
    assert_eq!(attached, vec![vec![0], vec![0, 0]]);
    wait_live_task(&engine, AnalysisTaskStateDto::Completed);
    state
        .save_to_path(saved_path.clone(), NodePath::default())
        .unwrap();
    let reopened = CurrentSgfDocument::open(&std::fs::read_to_string(saved_path).unwrap()).unwrap();
    assert_eq!(
        reopened.snapshot(&NodePath::default()).unwrap().personal_comment,
        "paused review note"
    );
    assert!(reopened
        .first_child_mainline_snapshots()
        .unwrap()
        .iter()
        .all(|node| node
            .primary_analysis
            .as_ref()
            .is_some_and(|frame| frame.visits == 32)));
    println!("Pause/Continue smoke: target ACK held Pausing; final released Paused; same task/Run, fresh Job; root skipped, two remaining nodes searched at 32 visits; late old frame fenced; navigation/comment/Save/recovery retained SGF without runtime work.");
}

#[cfg(unix)]
#[test]
fn all_positions_two_stage_pause_continue_controllable_engine_smoke() {
    use app_model::{
        AnalysisJobModeDto, AnalysisScopeDto, AnalysisScopeModeDto, AnalysisStageConditionsDto,
        AnalysisTaskLimitDto, AnalysisTaskStageDto, AnalysisTaskStateDto,
    };
    let engine = task_fixture();
    let state = CurrentGameState::default();
    state.connect_analysis_manager(engine.manager.clone());
    let opened = state.replace("(;SZ[9];B[dd])", None).unwrap();
    state
        .set_personal_comment(NodePath::default(), "two-stage review note".into())
        .unwrap();
    engine.manager.start("test").unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(4);
    let run_id = loop {
        if let app_model::ForegroundEngineLifecycleDto::Ready { run } = engine.manager.snapshot().lifecycle {
            break run.run_id;
        }
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(5));
    };
    let selected = engine
        .manager
        .start_selected_node_job(
            crate::bind_selected_node_job(
                &state,
                run_id.clone(),
                opened.generation,
                NodePath::default(),
                AnalysisJobModeDto::Continuous,
                Some(999),
            )
            .unwrap(),
        )
        .unwrap();
    loop {
        let event = engine.progress(None);
        if event.job_id == selected.job_id {
            state.attach_from_job_event(&event).unwrap();
            break;
        }
    }
    let scope = AnalysisScopeDto {
        mode: AnalysisScopeModeDto::FirstChildMainline,
        current_node: NodePath::default(),
        branch_choices: Vec::new(),
        interval: None,
        to_play: None,
    };
    let preview = state
        .preview_analysis_scope(opened.generation, scope, None)
        .unwrap();
    let conditions = |visits| AnalysisStageConditionsDto {
        time_seconds: AnalysisTaskLimitDto {
            enabled: false,
            value: 10,
        },
        total_visits: AnalysisTaskLimitDto {
            enabled: true,
            value: visits,
        },
        leading_candidate_visits: AnalysisTaskLimitDto {
            enabled: false,
            value: visits,
        },
    };
    std::fs::write(engine.directory.join("finish"), "go").unwrap();
    std::fs::write(engine.directory.join("hold-deep"), "go").unwrap();
    let task = state
        .start_all_positions_analysis_task(
            &engine.manager,
            run_id.clone(),
            preview,
            conditions(32),
            conditions(500),
        )
        .unwrap();

    let mut deep_progress_attached = false;
    while !deep_progress_attached {
        let event = engine.progress(None);
        if event.job_id == task.job_id
            && state.attach_from_job_event(&event).is_some()
            && engine
                .manager
                .analysis_task_snapshot()
                .is_some_and(|snapshot| snapshot.stage == AnalysisTaskStageDto::Deep)
        {
            deep_progress_attached = true;
        }
    }
    let deep = wait_live_task_stage(
        &engine,
        AnalysisTaskStageDto::Deep,
        AnalysisTaskStateDto::Searching,
    );
    assert_eq!(deep.overview_completed, deep.requested);
    assert!(deep.completed.is_empty());
    assert_eq!(deep.overview_summaries.len(), 2);
    assert!(deep
        .overview_summaries
        .iter()
        .all(|summary| summary.frame.visits == 32));

    let overview_identity = (
        deep.overview_completed.clone(),
        deep.completed.clone(),
        deep.overview_summaries.clone(),
    );
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(4);
    loop {
        let event = engine
            .events
            .recv_timeout(deadline.saturating_duration_since(std::time::Instant::now()))
            .unwrap();
        let app_model::ForegroundEngineEventDto::Job { job } = event else {
            continue;
        };
        if job.job_id == selected.job_id && job.outcome == app_model::AnalysisJobOutcomeDto::Progress {
            state.attach_from_job_event(&job).unwrap();
            break;
        }
        state.attach_from_job_event(&job);
    }
    assert_eq!(
        state
            .select_path(NodePath::default())
            .unwrap()
            .snapshot
            .primary_analysis
            .unwrap()
            .visits,
        999
    );
    let after_selected = engine.manager.analysis_task_snapshot().unwrap();
    assert_eq!(
        (
            after_selected.overview_completed,
            after_selected.completed,
            after_selected.overview_summaries,
        ),
        overview_identity
    );

    let pausing = state
        .pause_analysis_task(&engine.manager, &task.run_id, &task.task_id)
        .unwrap();
    assert_eq!(pausing.stage, AnalysisTaskStageDto::Deep);
    assert_eq!(pausing.state, AnalysisTaskStateDto::Pausing);
    assert_eq!(
        engine
            .manager
            .snapshot()
            .selected_node_job
            .as_ref()
            .map(|job| job.job_id.as_str()),
        Some(selected.job_id.as_str())
    );
    state.seal_job(&selected);
    engine
        .manager
        .cancel_job(&selected.run_id, &selected.job_id)
        .unwrap();
    std::fs::write(engine.directory.join("cancel-final"), "go").unwrap();
    let paused = wait_live_task_stage(&engine, AnalysisTaskStageDto::Deep, AnalysisTaskStateDto::Paused);
    assert!(paused.completed.is_empty());
    assert_eq!(paused.overview_completed, paused.requested);

    let continued = state
        .continue_analysis_task(&engine.manager, &task.run_id, &task.task_id)
        .unwrap();
    assert_eq!(continued.task_id, task.task_id);
    assert_ne!(continued.job_id, task.job_id);
    assert_eq!(continued.stage, AnalysisTaskStageDto::Deep);
    std::fs::remove_file(engine.directory.join("hold-deep")).unwrap();
    let mut deep_paths = Vec::new();
    while deep_paths.len() < 2 {
        let event = engine.progress(Some(&task.job_id));
        if state.attach_from_job_event(&event).is_some() {
            deep_paths.push(event.node_path.indices);
        }
    }
    assert_eq!(deep_paths, vec![Vec::<u32>::new(), vec![0]]);
    let completed = wait_live_task(&engine, AnalysisTaskStateDto::Completed);
    assert_eq!(completed.stage, AnalysisTaskStageDto::Deep);
    assert_eq!(completed.overview_completed, completed.requested);
    assert_eq!(completed.completed, completed.requested);
    assert!(completed
        .overview_summaries
        .iter()
        .all(|summary| summary.frame.visits == 32));

    let saved_path = engine
        .directory
        .join("two-stage.sgf")
        .to_string_lossy()
        .into_owned();
    state
        .save_to_path(saved_path.clone(), NodePath::default())
        .unwrap();
    let reopened = CurrentSgfDocument::open(&std::fs::read_to_string(&saved_path).unwrap()).unwrap();
    assert_eq!(
        reopened.snapshot(&NodePath::default()).unwrap().personal_comment,
        "two-stage review note"
    );
    assert!(reopened
        .first_child_mainline_snapshots()
        .unwrap()
        .iter()
        .all(|node| node
            .primary_analysis
            .as_ref()
            .is_some_and(|frame| frame.visits == 500)));
    let envelope = state
        .recovery_flush(app_model::ApplicationExitDispositionDto::ExitIncomplete)
        .unwrap();
    let recovered = CurrentGameState::default();
    let recovered_manager = ForegroundEngineManager::new(
        Arc::new(InMemoryEngineProfileCatalog::new()),
        ForegroundEngineConfig::for_tests(),
    );
    recovered.connect_analysis_manager(recovered_manager.clone());
    recovered
        .replace(&envelope.sgf_text, envelope.source_path)
        .unwrap();
    assert!(recovered_manager.analysis_task_snapshot().is_none());
    assert!(recovered.analysis_jobs().unwrap().is_empty());
    println!(
        "two-stage smoke: all overview targets completed at 32 visits before deep; deep Pause fenced the interrupted target; Continue used a fresh Job and completed both targets at 500 visits; Save/reopen retained comment and accepted analysis; recovery restored no runtime task."
    );
}

#[cfg(unix)]
#[test]
fn swing_selected_two_stage_freezes_exact_paths_after_complete_overview() {
    use app_model::{
        AnalysisMoveActorFilterDto, AnalysisPositionIntervalDto, AnalysisScopeDto, AnalysisScopeModeDto,
        AnalysisStageConditionsDto, AnalysisSwingCriteriaDto, AnalysisSwingThresholdDto,
        AnalysisTaskLimitDto, AnalysisTaskStateDto,
    };
    let engine = task_fixture();
    std::fs::write(engine.directory.join("budget-smoke"), "swing").unwrap();
    std::fs::write(engine.directory.join("hold-deep"), "go").unwrap();
    let state = CurrentGameState::default();
    state.connect_analysis_manager(engine.manager.clone());
    let opened = state.replace("(;SZ[9];B[dd];W[ee];B[ff])", None).unwrap();
    engine.manager.start("test").unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(4);
    let run_id = loop {
        if let app_model::ForegroundEngineLifecycleDto::Ready { run } = engine.manager.snapshot().lifecycle {
            break run.run_id;
        }
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(5));
    };
    let scope = AnalysisScopeDto {
        mode: AnalysisScopeModeDto::FirstChildMainline,
        current_node: NodePath::default(),
        branch_choices: Vec::new(),
        interval: Some(AnalysisPositionIntervalDto { start: 1, end: 3 }),
        to_play: None,
    };
    let criteria = AnalysisSwingCriteriaDto {
        move_actors: AnalysisMoveActorFilterDto::Both,
        winrate_change_percentage_points: AnalysisSwingThresholdDto {
            enabled: true,
            value: 10.0,
        },
        score_change_points: AnalysisSwingThresholdDto {
            enabled: false,
            value: 100.0,
        },
    };
    let preview = state
        .preview_analysis_scope(opened.generation, scope, Some(criteria.clone()))
        .unwrap();
    assert_eq!(
        preview
            .targets
            .iter()
            .map(|target| target.node_path.indices.clone())
            .collect::<Vec<_>>(),
        vec![vec![0], vec![0, 0], vec![0, 0, 0]]
    );
    assert_eq!(
        preview
            .supporting_targets
            .iter()
            .map(|target| target.node_path.indices.clone())
            .collect::<Vec<_>>(),
        vec![Vec::<u32>::new()]
    );
    assert_eq!(preview.swing_comparisons.len(), 3);
    let overview = AnalysisStageConditionsDto {
        time_seconds: AnalysisTaskLimitDto {
            enabled: false,
            value: 10,
        },
        total_visits: AnalysisTaskLimitDto {
            enabled: true,
            value: 32,
        },
        leading_candidate_visits: AnalysisTaskLimitDto {
            enabled: false,
            value: 32,
        },
    };
    let deep = AnalysisStageConditionsDto {
        time_seconds: AnalysisTaskLimitDto {
            enabled: true,
            value: 10,
        },
        total_visits: AnalysisTaskLimitDto {
            enabled: false,
            value: 800,
        },
        leading_candidate_visits: AnalysisTaskLimitDto {
            enabled: false,
            value: 500,
        },
    };
    let started = state
        .start_swing_analysis_task(&engine.manager, run_id, preview, overview, deep)
        .unwrap();
    let deep_searching = wait_live_task_stage(
        &engine,
        app_model::AnalysisTaskStageDto::Deep,
        AnalysisTaskStateDto::Searching,
    );
    let frozen = vec![
        NodePath::default(),
        NodePath { indices: vec![0] },
        NodePath { indices: vec![0, 0] },
        NodePath {
            indices: vec![0, 0, 0],
        },
    ];
    assert_eq!(deep_searching.selected_for_deep.as_ref(), Some(&frozen));
    let pausing = state
        .pause_analysis_task(&engine.manager, &started.run_id, &started.task_id)
        .unwrap();
    assert_eq!(pausing.selected_for_deep.as_ref(), Some(&frozen));
    std::fs::write(engine.directory.join("cancel-final"), "go").unwrap();
    let paused = wait_live_task_stage(
        &engine,
        app_model::AnalysisTaskStageDto::Deep,
        AnalysisTaskStateDto::Paused,
    );
    assert_eq!(paused.selected_for_deep.as_ref(), Some(&frozen));
    let continued = state
        .continue_analysis_task(&engine.manager, &started.run_id, &started.task_id)
        .unwrap();
    assert_eq!(continued.task_id, started.task_id);
    assert_ne!(continued.job_id, started.job_id);
    assert_eq!(continued.selected_for_deep.as_ref(), Some(&frozen));
    std::fs::remove_file(engine.directory.join("hold-deep")).unwrap();

    let completed = wait_live_task(&engine, AnalysisTaskStateDto::Completed);
    assert_eq!(completed.task_id, started.task_id);
    assert_eq!(
        completed.overview_completed,
        vec![
            NodePath { indices: vec![0] },
            NodePath { indices: vec![0, 0] },
            NodePath {
                indices: vec![0, 0, 0]
            },
            NodePath::default(),
        ]
    );
    assert_eq!(completed.selected_for_deep.as_ref(), Some(&frozen));
    assert_eq!(completed.completed, frozen);
    assert_eq!(completed.overview_summaries[1].frame.score_mean_black, None);
    assert_eq!(completed.reason, None);
    let queries = std::fs::read_to_string(engine.directory.join("queries.jsonl")).unwrap();
    assert_eq!(queries.lines().count(), 9);
}

#[cfg(unix)]
#[test]
fn swing_selected_missing_required_score_fails_without_fake_deep_search() {
    use app_model::{
        AnalysisMoveActorFilterDto, AnalysisPositionIntervalDto, AnalysisScopeDto, AnalysisScopeModeDto,
        AnalysisStageConditionsDto, AnalysisSwingCriteriaDto, AnalysisSwingThresholdDto,
        AnalysisTaskStateDto, EngineBackend, EngineCapabilitySnapshotDto, EngineFailureKind,
    };
    let engine = task_fixture();
    std::fs::write(engine.directory.join("budget-smoke"), "swing_zero").unwrap();
    let state = CurrentGameState::default();
    state.connect_analysis_manager(engine.manager.clone());
    let opened = state.replace("(;SZ[9];B[dd])", None).unwrap();
    engine.manager.start("test").unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(4);
    let run_id = loop {
        if let app_model::ForegroundEngineLifecycleDto::Ready { run } = engine.manager.snapshot().lifecycle {
            break run.run_id;
        }
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(5));
    };
    let scope = AnalysisScopeDto {
        mode: AnalysisScopeModeDto::FirstChildMainline,
        current_node: NodePath::default(),
        branch_choices: Vec::new(),
        interval: Some(AnalysisPositionIntervalDto { start: 1, end: 1 }),
        to_play: None,
    };
    let criteria = AnalysisSwingCriteriaDto {
        move_actors: AnalysisMoveActorFilterDto::Both,
        winrate_change_percentage_points: AnalysisSwingThresholdDto {
            enabled: false,
            value: 10.0,
        },
        score_change_points: AnalysisSwingThresholdDto {
            enabled: true,
            value: 3.0,
        },
    };
    let preview = state
        .preview_analysis_scope(opened.generation, scope, Some(criteria))
        .unwrap();
    engine
        .manager
        .set_capability_snapshot_for_tests(EngineCapabilitySnapshotDto {
            adapter_kind: EngineBackend::KataGoAnalysis,
            selected_node_analysis: true,
            whole_game_analysis: true,
            root_score: false,
            protocol_cancel: true,
        });
    let unsupported = state
        .start_swing_analysis_task(
            &engine.manager,
            run_id.clone(),
            preview.clone(),
            AnalysisStageConditionsDto::default(),
            AnalysisStageConditionsDto::default(),
        )
        .unwrap_err();
    assert_eq!(unsupported.kind, EngineFailureKind::UnsupportedCapability);
    engine
        .manager
        .set_capability_snapshot_for_tests(EngineCapabilitySnapshotDto {
            adapter_kind: EngineBackend::KataGoAnalysis,
            selected_node_analysis: true,
            whole_game_analysis: true,
            root_score: true,
            protocol_cancel: true,
        });
    let started = state
        .start_swing_analysis_task(
            &engine.manager,
            run_id,
            preview,
            AnalysisStageConditionsDto::default(),
            AnalysisStageConditionsDto::default(),
        )
        .unwrap();
    let failed = wait_live_task(&engine, AnalysisTaskStateDto::Failed);
    assert_eq!(failed.task_id, started.task_id);
    assert_eq!(failed.selected_for_deep, None);
    assert!(failed.completed.is_empty());
    assert!(failed
        .reason
        .as_deref()
        .is_some_and(|reason| reason.contains("required root score")));
    let queries = std::fs::read_to_string(engine.directory.join("queries.jsonl")).unwrap();
    assert_eq!(queries.lines().count(), 1);
}

#[cfg(unix)]
#[test]
fn paused_task_does_not_resume_after_departure_save_cancellation_or_failure() {
    for fail_save in [false, true] {
        let engine = task_fixture();
        let state = CurrentGameState::default();
        state.connect_analysis_manager(engine.manager.clone());
        let opened = state.replace("(;SZ[9];B[dd])", None).unwrap();
        let task = start_live_task(&engine, &state, opened.generation);
        let first = engine.progress(None);
        state.attach_from_job_event(&first).unwrap();
        state
            .pause_analysis_task(&engine.manager, &task.run_id, &task.task_id)
            .unwrap();
        std::fs::write(engine.directory.join("cancel-final"), "").unwrap();
        let paused = wait_live_task(&engine, app_model::AnalysisTaskStateDto::Paused);
        let admission = state.prepare_replacement("(;SZ[13])", None).unwrap();
        let departure_id = match admission {
            app_model::DocumentDepartureAdmissionDto::Ready { departure_id }
            | app_model::DocumentDepartureAdmissionDto::NeedsDecision { departure_id } => departure_id,
        };
        state.cancel_replacement(departure_id).unwrap();
        assert_eq!(engine.manager.analysis_task_snapshot().unwrap(), paused);
        let admission = state.prepare_replacement("(;SZ[13])", None).unwrap();
        let departure_id = match admission {
            app_model::DocumentDepartureAdmissionDto::Ready { departure_id }
            | app_model::DocumentDepartureAdmissionDto::NeedsDecision { departure_id } => departure_id,
        };
        let outcome = crate::document_departure::resolve_replacement(
            &state,
            departure_id,
            app_model::DocumentDepartureActionDto::Save,
            opened.selected_path.clone(),
            &[],
            |job| {
                engine
                    .manager
                    .cancel_job(&job.run_id, &job.job_id)
                    .map_err(|error| error.to_string())
            },
            |job, budget| {
                engine
                    .manager
                    .wait_for_job_cancellation(&job.run_id, &job.job_id, budget)
                    .map_err(|error| error.to_string())
            },
            || {
                Ok(if fail_save {
                    Some(engine.directory.to_string_lossy().into_owned())
                } else {
                    None
                })
            },
        )
        .unwrap();
        assert!(!outcome.committed);
        assert_eq!(
            engine.manager.analysis_task_snapshot().unwrap().state,
            app_model::AnalysisTaskStateDto::Cancelled
        );
        assert!(state
            .continue_analysis_task(&engine.manager, &task.run_id, &task.task_id)
            .is_err());
        assert_eq!(
            state
                .select_path(NodePath::default())
                .unwrap()
                .snapshot
                .position
                .board_size,
            9
        );
        assert!(state.attach_from_job_event(&first).is_none());
        assert!(matches!(
            engine.manager.snapshot().lifecycle,
            app_model::ForegroundEngineLifecycleDto::Ready { .. }
        ));
        assert!(engine.manager.snapshot().whole_game_job.is_none());
    }
}

#[cfg(unix)]
#[test]
fn task_pause_cleanup_failure_aborts_departure_and_retains_game() {
    for delivery_failure in [false, true] {
        let engine = task_fixture();
        if delivery_failure {
            std::fs::write(engine.directory.join("close-input"), "").unwrap();
        }
        let state = CurrentGameState::default();
        state.connect_analysis_manager(engine.manager.clone());
        let opened = state.replace("(;SZ[9];B[dd])", None).unwrap();
        let task = start_live_task(&engine, &state, opened.generation);
        state.attach_from_job_event(&engine.progress(None)).unwrap();
        wait_live_task(&engine, app_model::AnalysisTaskStateDto::Searching);
        if delivery_failure {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
            while !engine.directory.join("input-closed").exists() {
                assert!(std::time::Instant::now() < deadline);
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
        }
        state
            .pause_analysis_task(&engine.manager, &task.run_id, &task.task_id)
            .unwrap();
        let admission = state.prepare_replacement("(;SZ[13])", None).unwrap();
        let departure_id = match admission {
            app_model::DocumentDepartureAdmissionDto::Ready { departure_id }
            | app_model::DocumentDepartureAdmissionDto::NeedsDecision { departure_id } => departure_id,
        };
        let save_called = std::cell::Cell::new(false);
        let outcome = crate::document_departure::resolve_replacement(
            &state,
            departure_id,
            app_model::DocumentDepartureActionDto::Save,
            opened.selected_path,
            &[],
            |job| {
                engine
                    .manager
                    .cancel_job(&job.run_id, &job.job_id)
                    .map_err(|error| error.to_string())
            },
            |job, budget| {
                engine
                    .manager
                    .wait_for_job_cancellation(&job.run_id, &job.job_id, budget)
                    .map_err(|error| error.to_string())
            },
            || {
                save_called.set(true);
                Ok(Some(
                    engine
                        .directory
                        .join("must-not-save.sgf")
                        .to_string_lossy()
                        .into_owned(),
                ))
            },
        )
        .unwrap();
        assert!(!outcome.committed);
        assert!(!save_called.get());
        assert!(matches!(
            engine.manager.snapshot().lifecycle,
            app_model::ForegroundEngineLifecycleDto::Error { .. }
        ));
        assert_ne!(
            engine.manager.analysis_task_snapshot().unwrap().state,
            app_model::AnalysisTaskStateDto::Paused
        );
        assert!(state
            .continue_analysis_task(&engine.manager, &task.run_id, &task.task_id)
            .is_err());
        assert_eq!(
            state
                .select_path(NodePath::default())
                .unwrap()
                .snapshot
                .position
                .board_size,
            9
        );
        assert!(engine.manager.snapshot().whole_game_job.is_none());
        assert!(engine.manager.snapshot().selected_node_job.is_none());
    }
}

#[cfg(unix)]
#[test]
fn paused_task_invalidates_on_edit_outside_its_frozen_mainline() {
    let engine = task_fixture();
    let state = CurrentGameState::default();
    state.connect_analysis_manager(engine.manager.clone());
    let opened = state.replace("(;SZ[9];B[dd](;W[ee])(;W[ff]))", None).unwrap();
    let task = start_live_task(&engine, &state, opened.generation);
    let first = engine.progress(None);
    state.attach_from_job_event(&first).unwrap();
    state
        .pause_analysis_task(&engine.manager, &task.run_id, &task.task_id)
        .unwrap();
    std::fs::write(engine.directory.join("cancel-final"), "").unwrap();
    wait_live_task(&engine, app_model::AnalysisTaskStateDto::Paused);
    state.remove_variation(NodePath { indices: vec![0, 1] }).unwrap();
    assert_eq!(
        engine.manager.analysis_task_snapshot().unwrap().state,
        app_model::AnalysisTaskStateDto::Invalidated
    );
    assert!(state
        .continue_analysis_task(&engine.manager, &task.run_id, &task.task_id)
        .is_err());
    assert!(state.attach_from_job_event(&first).is_none());
    assert!(state
        .select_path(NodePath::default())
        .unwrap()
        .snapshot
        .primary_analysis
        .is_some());
}

#[cfg(unix)]
#[test]
fn whole_game_comment_save_controllable_engine_smoke() {
    let engine = LiveFixture::new();
    let state = CurrentGameState::default();
    state.connect_analysis_manager(engine.manager.clone());
    let opened = state.replace("(;SZ[9];B[dd])", None).unwrap();
    engine.manager.start("test").unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(4);
    let run_id = loop {
        if let app_model::ForegroundEngineLifecycleDto::Ready { run } = engine.manager.snapshot().lifecycle {
            break run.run_id;
        }
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(5));
    };
    let admission = state.admit_whole_game(opened.generation).unwrap();
    let job = engine
        .manager
        .start_whole_game_analysis(engine_manager::WholeGameJobRequest {
            run_id: run_id.clone(),
            generation: admission.generation,
            work_items: crate::whole_game_work_items(&admission, 32, &run_id).unwrap(),
        })
        .unwrap();
    while !engine.directory.join("whole-started").exists() {
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    state
        .set_personal_comment(opened.selected_path.clone(), "live whole-game note".into())
        .unwrap();
    let target = engine.directory.join("review.sgf");
    assert!(
        !state
            .save_to_path(
                target.to_string_lossy().into_owned(),
                opened.selected_path.clone()
            )
            .unwrap()
            .dirty
    );
    std::fs::write(engine.directory.join("whole-release"), "").unwrap();
    let mut attached = 0;
    loop {
        let event = engine
            .events
            .recv_timeout(std::time::Duration::from_secs(4))
            .unwrap();
        if let app_model::ForegroundEngineEventDto::Job { job: event } = event {
            if event.job_id != job.job_id {
                continue;
            }
            if let Some(result) = state.attach_from_job_event(&event) {
                assert!(result.dirty);
                attached += 1;
            }
            if event.outcome == app_model::AnalysisJobOutcomeDto::Completed {
                break;
            }
            assert!(!matches!(
                event.outcome,
                app_model::AnalysisJobOutcomeDto::Failed | app_model::AnalysisJobOutcomeDto::Timeout
            ));
        }
    }
    assert_eq!(attached, 2);
    state
        .save_to_path(
            target.to_string_lossy().into_owned(),
            opened.selected_path.clone(),
        )
        .unwrap();
    let reopened = CurrentSgfDocument::open(&std::fs::read_to_string(target).unwrap()).unwrap();
    assert_eq!(
        reopened.snapshot(&opened.selected_path).unwrap().personal_comment,
        "live whole-game note"
    );
    for node in reopened.first_child_mainline_snapshots().unwrap() {
        assert_eq!(node.primary_analysis.unwrap().visits, 32);
    }
    println!("whole-game smoke: same Job attached both nodes across comment/Save; SGF reopened with note and results");
}

#[cfg(unix)]
#[test]
fn explicit_analysis_scopes_controllable_engine_smoke() {
    use app_model::{
        AnalysisScopeDto, AnalysisScopeModeDto, AnalysisStageConditionsDto, AnalysisTaskLimitDto,
    };
    let engine = LiveFixture::new();
    let state = CurrentGameState::default();
    state.connect_analysis_manager(engine.manager.clone());
    let opened = state
        .replace("(;SZ[9];B[dd];C[note](;W[];AB[aa]PL[W])(;W[ee]))", None)
        .unwrap();
    engine.manager.start("test").unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(4);
    let run_id = loop {
        if let app_model::ForegroundEngineLifecycleDto::Ready { run } = engine.manager.snapshot().lifecycle {
            break run.run_id;
        }
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(5));
    };
    std::fs::write(engine.directory.join("whole-release"), "").unwrap();
    let conditions = AnalysisStageConditionsDto {
        time_seconds: AnalysisTaskLimitDto {
            enabled: false,
            value: 10,
        },
        total_visits: AnalysisTaskLimitDto {
            enabled: true,
            value: 32,
        },
        leading_candidate_visits: AnalysisTaskLimitDto {
            enabled: false,
            value: 1,
        },
    };
    let cases = [
        (AnalysisScopeModeDto::CurrentNode, None, None, vec![vec![0, 0]]),
        (
            AnalysisScopeModeDto::SelectedReviewLine,
            None,
            None,
            vec![vec![], vec![0], vec![0, 0, 1]],
        ),
        (
            AnalysisScopeModeDto::FirstChildMainline,
            None,
            None,
            vec![vec![], vec![0], vec![0, 0, 0], vec![0, 0, 0, 0]],
        ),
        (
            AnalysisScopeModeDto::AllBranches,
            None,
            None,
            vec![vec![], vec![0], vec![0, 0, 0], vec![0, 0, 0, 0], vec![0, 0, 1]],
        ),
        (
            AnalysisScopeModeDto::FirstChildMainline,
            Some(app_model::AnalysisPositionIntervalDto { start: 2, end: 2 }),
            Some(app_model::PlayerColor::White),
            vec![vec![0, 0, 0, 0]],
        ),
        (
            AnalysisScopeModeDto::AllBranches,
            Some(app_model::AnalysisPositionIntervalDto { start: 0, end: 0 }),
            None,
            vec![vec![]],
        ),
    ];
    let mut task_jobs = Vec::new();
    for (mode, interval, to_play, expected) in cases {
        let scope = AnalysisScopeDto {
            mode,
            current_node: NodePath { indices: vec![0, 0] },
            branch_choices: vec![app_model::AnalysisBranchChoiceDto {
                parent: NodePath { indices: vec![0, 0] },
                child: 1,
            }],
            interval,
            to_play,
        };
        let preview = state
            .preview_analysis_scope(opened.generation, scope, None)
            .unwrap();
        let mut stage_conditions = conditions.clone();
        if expected == vec![Vec::<u32>::new()] {
            stage_conditions.total_visits.value = 1;
        }
        assert_eq!(
            preview
                .targets
                .iter()
                .map(|target| target.node_path.indices.clone())
                .collect::<Vec<_>>(),
            expected
        );
        let task = state
            .start_analysis_task(&engine.manager, run_id.clone(), preview, stage_conditions)
            .unwrap();
        task_jobs.push(task.job_id.clone());
        assert_ne!(task.task_id, task.job_id);
        let mut attached = Vec::new();
        loop {
            let event = engine
                .events
                .recv_timeout(std::time::Duration::from_secs(4))
                .unwrap();
            if let app_model::ForegroundEngineEventDto::Job { job } = event {
                if job.job_id != task.job_id {
                    continue;
                }
                if state.attach_from_job_event(&job).is_some() {
                    attached.push(job.node_path.indices.clone());
                }
                if job.outcome == app_model::AnalysisJobOutcomeDto::Completed {
                    break;
                }
                assert!(!matches!(
                    job.outcome,
                    app_model::AnalysisJobOutcomeDto::Failed | app_model::AnalysisJobOutcomeDto::Timeout
                ));
            }
        }
        assert_eq!(attached, expected);
        let completed = engine.manager.analysis_task_snapshot().unwrap();
        assert_eq!(completed.state, app_model::AnalysisTaskStateDto::Completed);
        assert_eq!(
            completed
                .completed
                .iter()
                .map(|path| path.indices.clone())
                .collect::<Vec<_>>(),
            expected
        );
        println!("scope {mode:?}: searched and attached {attached:?}");
    }
    let queries: Vec<serde_json::Value> = std::fs::read_to_string(engine.directory.join("queries.jsonl"))
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .filter(|query: &serde_json::Value| {
            query["id"]
                .as_str()
                .is_some_and(|wire_id| task_jobs.iter().any(|id| wire_id.starts_with(id)))
        })
        .collect();
    assert_eq!(
        queries.len(),
        15,
        "each new task recomputes positions even with existing SGF results"
    );
    assert!(queries[..14]
        .iter()
        .all(|query| query["overrideSettings"]["maxVisits"] == 32));
    assert_eq!(queries[14]["overrideSettings"]["maxVisits"], 1);
    assert_eq!(queries[13]["moves"], serde_json::json!([["B", "pass"]]));
    assert_eq!(queries[14]["moves"], serde_json::json!([]));
    let saved = engine.directory.join("scopes.sgf");
    state
        .save_to_path(saved.to_string_lossy().into(), opened.selected_path)
        .unwrap();
    let reopened = CurrentSgfDocument::open(&std::fs::read_to_string(saved).unwrap()).unwrap();
    assert_eq!(
        reopened
            .snapshot(&NodePath { indices: vec![0, 0] })
            .unwrap()
            .personal_comment,
        "note"
    );
    for path in [
        vec![],
        vec![0],
        vec![0, 0],
        vec![0, 0, 0],
        vec![0, 0, 0, 0],
        vec![0, 0, 1],
    ] {
        let expected_visits = if path.is_empty() { 1 } else { 32 };
        assert_eq!(
            reopened
                .snapshot(&NodePath { indices: path })
                .unwrap()
                .primary_analysis
                .unwrap()
                .visits,
            expected_visits
        );
    }
}

#[cfg(unix)]
#[test]
fn authoritative_following_seals_old_progress_and_failed_write_preserves_search() {
    let engine = LiveFixture::new();
    let state = CurrentGameState::default();
    state.connect_analysis_manager(engine.manager.clone());
    let opened = state.replace("(;SZ[9];B[dd])", None).unwrap();
    let preferences = crate::continuous_analysis::PreferencesState::default();
    let path = engine.directory.join("preferences.json");
    preferences.load(&path, &engine.manager).unwrap();
    engine.manager.start("test").unwrap();
    let first = engine.progress(None);
    assert_eq!(first.node_path, opened.selected_path);
    assert!(state.attach_from_job_event(&first).is_some());
    let departure = state.prepare_replacement("(;SZ[9])", None).unwrap();
    let departure_id = match departure {
        app_model::DocumentDepartureAdmissionDto::Ready { departure_id }
        | app_model::DocumentDepartureAdmissionDto::NeedsDecision { departure_id } => departure_id,
    };
    state.cancel_replacement(departure_id).unwrap();
    assert_eq!(
        engine.manager.snapshot().selected_node_job.unwrap().job_id,
        first.job_id
    );
    assert!(state.attach_from_job_event(&first).is_some());
    state.select_path(NodePath { indices: vec![] }).unwrap();
    state.select_path(opened.selected_path.clone()).unwrap();
    std::fs::write(engine.directory.join("release"), "").unwrap();
    assert!(
        state.attach_from_job_event(&first).is_none(),
        "A-B-A must not reopen old A publication"
    );
    let next = engine.progress(Some(&first.job_id));
    assert_eq!(next.node_path, opened.selected_path);
    assert!(state.attach_from_job_event(&next).is_some());
    assert!(preferences.primary(&engine.directory, &engine.manager).is_err());
    assert_eq!(
        engine.manager.snapshot().selected_node_job.unwrap().job_id,
        next.job_id
    );
    assert!(state.attach_from_job_event(&next).is_some());
    preferences.primary(&path, &engine.manager).unwrap();
    assert!(
        state.attach_from_job_event(&next).is_none(),
        "durable Stop seals already queued progress"
    );
}

#[cfg(unix)]
#[test]
fn continuous_budget_write_failure_preserves_search_and_success_seals_old_identity() {
    let engine = LiveFixture::new();
    let state = CurrentGameState::default();
    state.connect_analysis_manager(engine.manager.clone());
    state.replace("(;SZ[9];B[dd]C[retained])", None).unwrap();
    let preferences = crate::continuous_analysis::PreferencesState::default();
    let path = engine.directory.join("preferences.json");
    let mut settings = preferences.load(&path, &engine.manager).unwrap().preferences;
    engine.manager.start("test").unwrap();
    let first = engine.progress(None);
    assert!(state.attach_from_job_event(&first).is_some());
    settings.continuous_budget.continuous_time_limit_seconds = 2;
    assert!(preferences
        .save(&engine.directory, &engine.manager, settings.clone())
        .is_err());
    assert_eq!(
        engine.manager.snapshot().selected_node_job.unwrap().job_id,
        first.job_id
    );
    assert!(state.attach_from_job_event(&first).is_some());

    let mut invalid = settings.clone();
    invalid.continuous_budget.continuous_visits_limit = 0;
    assert!(preferences.save(&path, &engine.manager, invalid).is_err());
    assert_eq!(
        engine.manager.snapshot().selected_node_job.unwrap().job_id,
        first.job_id
    );
    preferences
        .save(&path, &engine.manager, settings.clone())
        .unwrap();
    assert_eq!(
        engine.manager.snapshot().continuous.phase,
        app_model::ContinuousAnalysisPhaseDto::Stopping
    );
    assert!(state.attach_from_job_event(&first).is_none());
    std::fs::write(engine.directory.join("release"), "").unwrap();
    let replacement = engine.progress(Some(&first.job_id));
    assert_ne!(replacement.job_id, first.job_id);
    assert!(state.attach_from_job_event(&replacement).is_some());
    assert!(state.serialize().unwrap().contains("C[retained]"));

    engine.manager.begin_continuous_departure();
    engine
        .manager
        .cancel_job(&replacement.run_id, &replacement.job_id)
        .unwrap();
    engine
        .manager
        .wait_for_job_cancellation(
            &replacement.run_id,
            &replacement.job_id,
            std::time::Duration::from_secs(2),
        )
        .unwrap();
    engine.manager.finish_continuous_departure(false);
    settings.continuous_budget.continuous_time_limit_seconds = 3;
    preferences.save(&path, &engine.manager, settings).unwrap();
    assert_eq!(
        engine.manager.snapshot().continuous.phase,
        app_model::ContinuousAnalysisPhaseDto::SafetyHold
    );
    assert!(engine.manager.snapshot().selected_node_job.is_none());
    preferences.primary(&path, &engine.manager).unwrap();
    let resumed = engine.progress(Some(&replacement.job_id));
    assert_ne!(resumed.job_id, replacement.job_id);
}

#[cfg(unix)]
#[test]
fn continuous_empty_board_uses_stones_not_root_or_move_number() {
    let engine = LiveFixture::new();
    std::fs::write(engine.directory.join("release"), "").unwrap();
    let state = CurrentGameState::default();
    state.connect_analysis_manager(engine.manager.clone());
    let opened = state.replace("(;SZ[9]AB[dd]C[setup])", None).unwrap();
    assert_eq!(opened.snapshot.position.move_number, 0);
    let preferences = crate::continuous_analysis::PreferencesState::default();
    let path = engine.directory.join("preferences.json");
    let mut settings = preferences.load(&path, &engine.manager).unwrap().preferences;
    settings.continuous_budget.continuous_stop_on_empty_board = true;
    preferences
        .save(&path, &engine.manager, settings.clone())
        .unwrap();
    engine.manager.start("test").unwrap();
    let root_search = engine.progress(None);
    assert_eq!(root_search.node_path.indices, Vec::<u32>::new());

    let empty = state.replace("(;SZ[9]C[root];B[]C[empty])", None).unwrap();
    assert_eq!(empty.snapshot.position.move_number, 1);
    assert!(empty.snapshot.position.stones.is_empty());
    engine
        .manager
        .wait_for_job_cancellation(
            &root_search.run_id,
            &root_search.job_id,
            std::time::Duration::from_secs(2),
        )
        .unwrap();
    let before = state.serialize().unwrap();
    assert_eq!(
        engine.manager.snapshot().continuous.phase,
        app_model::ContinuousAnalysisPhaseDto::EmptyBoard
    );
    assert_eq!(engine.manager.snapshot().continuous.enabled, Some(true));
    assert!(engine.manager.snapshot().selected_node_job.is_none());
    assert_eq!(state.serialize().unwrap(), before);

    settings.continuous_budget.continuous_stop_on_empty_board = false;
    preferences
        .save(&path, &engine.manager, settings.clone())
        .unwrap();
    let resumed = engine.progress(Some(&root_search.job_id));
    assert_eq!(resumed.node_path, empty.selected_path);
    settings.continuous_budget.continuous_stop_on_empty_board = true;
    preferences.save(&path, &engine.manager, settings).unwrap();
    engine
        .manager
        .wait_for_job_cancellation(
            &resumed.run_id,
            &resumed.job_id,
            std::time::Duration::from_secs(2),
        )
        .unwrap();
    assert_eq!(
        engine.manager.snapshot().continuous.phase,
        app_model::ContinuousAnalysisPhaseDto::EmptyBoard
    );
    state.replace("(;SZ[9]AB[dd]C[setup])", None).unwrap();
    assert_ne!(engine.progress(Some(&resumed.job_id)).job_id, resumed.job_id);
}

#[cfg(unix)]
#[test]
fn task_search_budgets_controllable_engine_smoke() {
    use app_model::{
        AnalysisScopeDto, AnalysisScopeModeDto, AnalysisStageConditionsDto, AnalysisTaskStateDto,
    };
    let engine = task_fixture();
    let state = CurrentGameState::default();
    state.connect_analysis_manager(engine.manager.clone());
    let opened = state.replace("(;SZ[9]C[budget smoke];B[dd])", None).unwrap();
    engine.manager.start("test").unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(4);
    let run_id = loop {
        if let app_model::ForegroundEngineLifecycleDto::Ready { run } = engine.manager.snapshot().lifecycle {
            break run.run_id;
        }
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(5));
    };
    for condition in ["time_seconds", "total_visits", "leading_candidate_visits"] {
        std::fs::write(engine.directory.join("budget-smoke"), condition).unwrap();
        let _ = std::fs::remove_file(engine.directory.join("cancel-final"));
        let _ = std::fs::remove_file(engine.directory.join("cancel-seen"));
        let mut conditions = AnalysisStageConditionsDto::default();
        conditions.time_seconds.enabled = condition == "time_seconds";
        conditions.time_seconds.value = 1;
        conditions.total_visits.enabled = condition == "total_visits";
        conditions.total_visits.value = 8;
        conditions.leading_candidate_visits.enabled = condition == "leading_candidate_visits";
        conditions.leading_candidate_visits.value = 5;
        let preview = state
            .preview_analysis_scope(
                opened.generation,
                AnalysisScopeDto {
                    mode: AnalysisScopeModeDto::CurrentNode,
                    current_node: NodePath { indices: vec![] },
                    branch_choices: vec![],
                    interval: None,
                    to_play: None,
                },
                None,
            )
            .unwrap();
        let task = state
            .start_analysis_task(&engine.manager, run_id.clone(), preview, conditions.clone())
            .unwrap();
        let preferences = crate::continuous_analysis::PreferencesState::default();
        let path = engine.directory.join("preferences.json");
        let mut preset = app_preferences::default_app_preferences();
        preset.continuous_analysis_enabled = false;
        preset.task_deep_conditions = Some(AnalysisStageConditionsDto::default());
        preferences.save(&path, &engine.manager, preset.clone()).unwrap();
        assert_eq!(
            engine.manager.analysis_task_snapshot().unwrap().conditions,
            conditions
        );
        let mut rejected = preset.clone();
        rejected.task_deep_conditions.as_mut().unwrap().time_seconds.value = 0;
        assert!(preferences.save(&path, &engine.manager, rejected).is_err());
        assert!(preferences
            .save(&engine.directory, &engine.manager, preset.clone())
            .is_err());
        assert_eq!(
            preferences.load(&path, &engine.manager).unwrap().preferences,
            preset
        );
        assert_eq!(
            app_preferences::load_from_path(&path).unwrap().preferences,
            preset
        );
        assert_eq!(
            engine.manager.analysis_task_snapshot().unwrap().conditions,
            conditions
        );
        if condition != "time_seconds" {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(4);
            while !engine.directory.join("cancel-seen").exists() {
                assert!(std::time::Instant::now() < deadline);
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
            assert!(engine
                .manager
                .analysis_task_snapshot()
                .unwrap()
                .completed
                .is_empty());
            std::fs::write(engine.directory.join("cancel-final"), "").unwrap();
        }
        let mut attached = Vec::new();
        loop {
            let event = engine
                .events
                .recv_timeout(std::time::Duration::from_secs(4))
                .unwrap();
            if let app_model::ForegroundEngineEventDto::Job { job } = event {
                if job.job_id != task.job_id {
                    continue;
                }
                if state.attach_from_job_event(&job).is_some() {
                    attached.push(job.node_path.indices.clone());
                }
                assert!(
                    !matches!(
                        job.outcome,
                        app_model::AnalysisJobOutcomeDto::Failed | app_model::AnalysisJobOutcomeDto::Timeout
                    ),
                    "{job:?}"
                );
                if job.outcome == app_model::AnalysisJobOutcomeDto::Completed {
                    break;
                }
            }
        }
        let completed = wait_live_task(&engine, AnalysisTaskStateDto::Completed);
        assert_eq!(completed.ending_conditions, vec![condition]);
        assert_eq!(completed.completed, vec![NodePath { indices: vec![] }]);
        assert_eq!(attached, vec![Vec::<u32>::new()]);
        println!(
            "budget {condition}: actual query finished, ending={:?}, attached={attached:?}",
            completed.ending_conditions
        );
    }
    let saved = engine.directory.join("budgets.sgf");
    state
        .save_to_path(saved.to_string_lossy().into(), opened.selected_path)
        .unwrap();
    let reopened = CurrentSgfDocument::open(&std::fs::read_to_string(saved).unwrap()).unwrap();
    let root = reopened.snapshot(&NodePath { indices: vec![] }).unwrap();
    assert_eq!(root.personal_comment, "budget smoke");
    assert!(root.primary_analysis.is_some());
}
