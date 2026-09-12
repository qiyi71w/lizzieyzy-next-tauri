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
    if q.get("action") == "terminate":
        assert "terminateId" in q and q["id"] != q["terminateId"]
        print(json.dumps(dict(id=q["id"], action="terminate")), flush=True)
        while not pathlib.Path("release").exists():
            time.sleep(0.005)
        print(json.dumps(dict(id=q["terminateId"], isDuringSearch=False, noResults=True)), flush=True)
    else:
        if q.get("maxVisits", 0) == 32:
            pathlib.Path("whole-started").touch()
            while not pathlib.Path("whole-release").exists():
                time.sleep(0.005)
            print(json.dumps(dict(id=q["id"], isDuringSearch=False, turnNumber=0,
                rootInfo=dict(visits=32, winrate=0.6, scoreMean=2.5),
                moveInfos=[dict(move="E5", visits=32, winrate=0.6, scoreMean=2.5)])), flush=True)
            continue
        for visits in [8,16]:
            print(json.dumps(dict(id=q["id"], isDuringSearch=True, turnNumber=0,
                rootInfo=dict(visits=visits, winrate=0.6, scoreMean=2.5),
                moveInfos=[dict(move="E5", visits=visits, winrate=0.6, scoreMean=2.5)])), flush=True)
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
