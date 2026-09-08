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
    manager.set_continuous_intent(true);
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
    manager.set_continuous_intent(false);
    manager.set_continuous_intent(true);
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
