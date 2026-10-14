use super::*;
use app_model::{AnalysisJobOutcomeDto, ForegroundEngineEventDto, ForegroundEngineLifecycleDto};
use std::time::{Duration, Instant};
use engine_manager::EngineProfileCatalog;

fn fixture() -> (LiveFixture, CurrentGameState, String, u64) {
    let engine = LiveFixture::new();
    std::fs::write(engine.directory.join("engine"), r#"#!/usr/bin/env python3
import json, sys
for line in sys.stdin:
    q = json.loads(line)
    if q.get('action') == 'terminate':
        print(json.dumps(dict(id=q['id'],action='terminate')),flush=True)
        print(json.dumps(dict(id=q['terminateId'],isDuringSearch=False,noResults=True)),flush=True)
    elif q['id'].startswith('lifecycle-readiness-'):
        print(json.dumps(dict(id=q['id'],isDuringSearch=False,noResults=True)),flush=True)
    else:
        r = json.load(open('result.json'))
        r['id'] = q['id']
        print(json.dumps(r),flush=True)
"#).unwrap();
    let state = CurrentGameState::default();
    state.connect_analysis_manager(engine.manager.clone());
    engine.manager.set_continuous_preferences(false, Default::default()).unwrap();
    let opened = state.replace("(;SZ[9]RU[Chinese]KM[7.5]C[personal]LZOP2[Secondary 50.0 999\nmove E5 visits 999 winrate 5000 pv E5];B[dd])", None).unwrap();
    state.select_path(NodePath::default(), opened.generation).unwrap();
    engine.manager.start("test").unwrap();
    let deadline = Instant::now() + Duration::from_secs(4);
    let run = loop {
        if let ForegroundEngineLifecycleDto::Ready { run } = engine.manager.snapshot().lifecycle { break run; }
        assert!(Instant::now() < deadline, "{:?}", engine.manager.snapshot());
        std::thread::sleep(Duration::from_millis(5));
    };
    (engine, state, run.run_id, opened.generation)
}

fn calculate(engine: &LiveFixture, state: &CurrentGameState, run: &str, generation: u64,
    visits: u32, score: f64, ownership: bool) -> AnalysisJobEventDto {
    let map = ownership.then(|| vec![score / 100.0; 81]);
    std::fs::write(engine.directory.join("result.json"), serde_json::json!({
        "isDuringSearch": false, "turnNumber": 0,
        "rootInfo": {"visits": visits, "winrate": 0.6, "scoreLead": score},
        "moveInfos": [{"move":"E5", "order":0,"visits":visits,"winrate":0.6,"scoreLead":score,"pv":["E5"]}],
        "ownership": map
    }).to_string()).unwrap();
    let request = crate::bind_selected_node_job(state, run.into(), generation, NodePath::default(), AnalysisJobModeDto::Finite, Some(visits)).unwrap();
    let started = engine.manager.start_selected_node_job(request).unwrap();
    loop {
        if let ForegroundEngineEventDto::Job { job } = engine.events.recv_timeout(Duration::from_secs(4)).unwrap() {
            if job.job_id == started.job_id && job.outcome == AnalysisJobOutcomeDto::Completed { return job; }
            assert!(!matches!(job.outcome, AnalysisJobOutcomeDto::Failed | AnalysisJobOutcomeDto::Timeout), "{job:?}");
        }
    }
}

#[test]
fn ordinary_reuse_production_attach_lower_equal_higher_disable_unknown_and_save() {
    let (engine, state, run, generation) = fixture();
    let high = calculate(&engine, &state, &run, generation, 1000, 4.0, true);
    assert!(engine.manager.ordinary_analysis_context(&high).is_some());
    let attached = state.attach_from_job_event(&high).unwrap();
    assert_eq!(attached.snapshot.primary_analysis.as_ref().unwrap().visits, 1000);
    let old_bytes = state.serialize().unwrap();
    let lower = calculate(&engine, &state, &run, generation, 100, 2.0, true);
    assert!(state.attach_from_job_event(&lower).is_none());
    let retained = state.select_path(NodePath::default(), generation).unwrap();
    assert_eq!(retained.snapshot.primary_analysis.as_ref().unwrap().visits, 1000);
    assert_eq!(state.serialize().unwrap(), old_bytes);
    assert_eq!(retained.snapshot_seq, attached.snapshot_seq);
    let equal = calculate(&engine, &state, &run, generation, 1000, 3.0, true);
    let updated = state.attach_from_job_event(&equal).unwrap();
    assert_eq!(updated.snapshot.primary_analysis.unwrap().score_mean_black, Some(3.0));
    let higher = calculate(&engine, &state, &run, generation, 2000, 5.0, true);
    assert_eq!(state.attach_from_job_event(&higher).unwrap().snapshot.primary_analysis.unwrap().visits, 2000);
    let before_off = state.serialize().unwrap();
    let before_run = engine.manager.snapshot().lifecycle;
    engine.manager.set_retain_node_analysis(false);
    assert_eq!(state.serialize().unwrap(), before_off);
    assert_eq!(engine.manager.snapshot().lifecycle, before_run);
    let lower = calculate(&engine, &state, &run, generation, 10, 1.0, true);
    assert_eq!(state.attach_from_job_event(&lower).unwrap().snapshot.primary_analysis.unwrap().visits, 10);
    engine.manager.set_retain_node_analysis(true);
    let weaker = calculate(&engine, &state, &run, generation, 5, 0.5, true);
    assert!(state.attach_from_job_event(&weaker).is_none());
    assert_eq!(state.select_path(NodePath::default(), generation).unwrap().snapshot.primary_analysis.unwrap().visits, 10);
    let mut unknown = weaker.clone();
    unknown.job_id = "unknown-result-context".into();
    assert_eq!(state.attach_from_job_event(&unknown).unwrap().snapshot.primary_analysis.unwrap().visits, 5);
    let path = engine.directory.join("retained.sgf");
    state.save_to_path(path.to_string_lossy().into(), NodePath::default()).unwrap();
    let reopened = CurrentSgfDocument::open(&std::fs::read_to_string(path).unwrap()).unwrap();
    let root = reopened.snapshot(&NodePath::default()).unwrap();
    assert_eq!(root.primary_analysis.unwrap().visits, 5);
    assert_eq!(root.personal_comment, "personal");
    assert_eq!(root.secondary_analysis.unwrap().visits, 999);
    assert!(reopened.snapshot(&NodePath { indices: vec![0] }).unwrap().primary_analysis.is_none());
}

#[test]
fn ordinary_reuse_rejects_incomplete_and_never_splices_other_result_maps() {
    let (engine, state, run, generation) = fixture();
    let high = calculate(&engine, &state, &run, generation, 1000, 4.0, true);
    state.attach_from_job_event(&high).unwrap();
    let before = state.serialize().unwrap();
    let mut incomplete = calculate(&engine, &state, &run, generation, 2000, 2.0, true);
    incomplete.frame.as_mut().unwrap().candidates.clear();
    assert!(state.attach_from_job_event(&incomplete).is_none());
    assert_eq!(state.serialize().unwrap(), before);
    let mut invalid = incomplete.clone();
    invalid.frame = high.frame.clone();
    invalid.frame.as_mut().unwrap().candidates[0].vertex = MoveVertex::Point(app_model::PointDto { x: 40, y: 40 });
    assert!(state.attach_from_job_event(&invalid).is_none());
    // Imported/header-only content has no registered provenance and cannot block
    // a complete current result even when its counter is larger.
    let mut header = SgfAnalysisPayload::from_frame(high.frame.as_ref().unwrap(), "old");
    header.visits = 10000;
    header.candidates.clear();
    state.attach_primary_analysis(generation, NodePath::default(), header).unwrap();
    let low = calculate(&engine, &state, &run, generation, 100, 1.0, true);
    assert_eq!(state.attach_from_job_event(&low).unwrap().snapshot.primary_analysis.unwrap().visits, 100);
    let next = calculate(&engine, &state, &run, generation, 200, 2.0, true);
    let result = state.attach_from_job_event(&next).unwrap();
    assert!(result.snapshot.primary_analysis.unwrap().ownership.unwrap().iter().all(|v| (*v - 0.02).abs() < 0.001));
}

#[test]
fn ordinary_reuse_atomic_write_failure_preserves_effective_value_and_run() {
    let (engine, state, _, _) = fixture();
    let preferences = crate::continuous_analysis::PreferencesState::default();
    let path = engine.directory.join("prefs.json");
    let mut draft = preferences.load(&path, &engine.manager).unwrap().preferences;
    assert!(draft.retain_node_analysis);
    let before = state.serialize().unwrap();
    let run = engine.manager.snapshot().lifecycle;
    draft.retain_node_analysis = false;
    assert!(preferences.save(&engine.directory, &engine.manager, draft.clone()).is_err());
    assert!(engine.manager.retain_node_analysis());
    assert_eq!(engine.manager.snapshot().lifecycle, run);
    assert_eq!(state.serialize().unwrap(), before);
    preferences.save(&path, &engine.manager, draft).unwrap();
    assert!(!engine.manager.retain_node_analysis());
    assert!(!app_preferences::load_from_path(&path).unwrap().preferences.retain_node_analysis);
}

#[test]
fn ordinary_reuse_incompatible_context_and_explicit_replacement_take_priority() {
    let (engine, state, run, generation) = fixture();
    let high = calculate(&engine, &state, &run, generation, 10000, 4.0, true);
    state.attach_from_job_event(&high).unwrap();
    let original_context = engine.manager.ordinary_analysis_context(&high).unwrap();
    let child = NodePath { indices: vec![0] };
    state.attach_primary_analysis(generation, child.clone(), SgfAnalysisPayload::from_frame(high.frame.as_ref().unwrap(), "child")).unwrap();
    let low = calculate(&engine, &state, &run, generation, 1000, 2.0, true);
    assert!(state.attach_from_job_event(&low).is_none());
    let mut request = crate::bind_selected_node_job(&state, run.clone(), generation, NodePath::default(), AnalysisJobModeDto::Finite, Some(1000)).unwrap();
    request.query.include_policy = Some(false);
    let started = engine.manager.start_selected_node_job(request).unwrap();
    let changed = loop {
        if let ForegroundEngineEventDto::Job { job } = engine.events.recv_timeout(Duration::from_secs(4)).unwrap() {
            if job.job_id == started.job_id && job.outcome == AnalysisJobOutcomeDto::Completed { break job; }
        }
    };
    assert_ne!(engine.manager.ordinary_analysis_context(&changed).unwrap(), original_context);
    assert!(engine.manager.ordinary_analysis_context(&high).is_none(), "old events cannot acquire new registration context");
    assert_eq!(state.attach_from_job_event(&changed).unwrap().snapshot.primary_analysis.unwrap().visits, 1000);
    // The same unconditional seam used by Task/focus replaces a whole slot even
    // when a previous ordinary result was stronger. It retires that proof.
    let high = calculate(&engine, &state, &run, generation, 10000, 4.0, true);
    state.attach_from_job_event(&high).unwrap();
    let replacement = SgfAnalysisPayload::from_frame(low.frame.as_ref().unwrap(), "new-tree");
    assert_eq!(state.attach_primary_analysis(generation, NodePath::default(), replacement).unwrap().snapshot.primary_analysis.unwrap().visits, 1000);
    let task = AnalysisJobEventDto { lane: app_model::AnalysisJobLaneDto::WholeGame, outcome: AnalysisJobOutcomeDto::Progress, ..low };
    assert_eq!(state.attach_from_job_event(&task).unwrap().snapshot.primary_analysis.unwrap().visits, 1000);
    let child_result = state.select_path(child, generation).unwrap();
    assert_eq!(child_result.snapshot.primary_analysis.unwrap().visits, 10000);
    let root = state.select_path(NodePath::default(), generation).unwrap();
    assert_eq!(root.snapshot.secondary_analysis.unwrap().visits, 999);
}

#[test]
fn local_focus_production_parser_replaces_stronger_whole_slot_and_preserves_secondary() {
    let engine = LiveFixture::new();
    std::fs::write(engine.directory.join("engine"), concat!("#!/usr/bin/env python3\n", include_str!("../../../../../crates/engine-manager/tests/fixtures/rules_process.py"))).unwrap();
    std::fs::write(engine.directory.join("mode"), "analysis_focus_payload").unwrap();
    let mut profile = engine.catalog.get("test").unwrap();
    profile.profile.adapter = app_model::EngineAdapterSettings::KataGoGtp(app_model::KataGoSettings { model_path: Some("model".into()), config_path: Some("config".into()), max_visits: 100000 });
    engine.catalog.upsert(profile);
    let payload = |visits, point, value| format!("info move {point} visits {visits} winrate 0.6 order 0 pv {point} rootInfo visits {visits} winrate 0.6 ownership {}", vec![value; 81].join(" "));
    std::fs::write(engine.directory.join("analysis-frame"), payload(10000, "D4", "0.8")).unwrap();
    let state = CurrentGameState::default();
    state.connect_analysis_manager(engine.manager.clone());
    let budget = app_model::ContinuousAnalysisBudgetDto { continuous_time_limit_enabled: true, continuous_time_limit_seconds: 2, ..Default::default() };
    engine.manager.set_continuous_preferences(false, budget).unwrap();
    let opened = state.replace("(;SZ[9]RU[Chinese]KM[7.5]C[personal]LZOP2[Secondary 50.0 999\nmove E5 visits 999 winrate 5000 pv E5];B[dd])", None).unwrap();
    state.select_path(NodePath::default(), opened.generation).unwrap();
    engine.manager.start("test").unwrap();
    let until = Instant::now() + Duration::from_secs(5);
    let run = loop {
        if let ForegroundEngineLifecycleDto::Ready { run } = engine.manager.snapshot().lifecycle { break run; }
        assert!(Instant::now() < until);
        std::thread::sleep(Duration::from_millis(5));
    };
    let mut resource = (*run.qualified_resource.unwrap()).clone();
    resource.resources.iter_mut().find(|resource| resource.component == "executable").unwrap().sha256 = "20c4ede9f39455ab3c019ada7a22a6e4d7696361161feed5c56f2c42d1507bd0".into();
    engine.manager.set_qualified_resource_for_tests(resource);
    let preferences = crate::continuous_analysis::PreferencesState::default();
    let prefs_path = engine.directory.join("refresh-preferences.json");
    let mut settings = preferences.load(&prefs_path, &engine.manager).unwrap().preferences;
    settings.continuous_analysis_enabled = true;
    settings.continuous_budget = budget;
    settings.analysis_refresh_interval_centiseconds = app_model::AnalysisRefreshIntervalDto(17);
    let mut settings = preferences.save(&prefs_path, &engine.manager, settings).unwrap();
    let request = crate::bind_selected_node_job(&state, run.run_id.clone(), opened.generation, NodePath::default(), AnalysisJobModeDto::Continuous, None).unwrap();
    engine.manager.follow_continuous_position(request);
    let next = || loop {
        if let ForegroundEngineEventDto::Job { job } = engine.events.recv_timeout(Duration::from_secs(5)).unwrap() {
            assert!(!matches!(job.outcome, AnalysisJobOutcomeDto::Failed | AnalysisJobOutcomeDto::Timeout), "{job:?}");
            if job.outcome == AnalysisJobOutcomeDto::Progress { break job; }
        }
    };
    let high = next();
    assert_eq!(engine.manager.snapshot().local_focus.actual_interval_centiseconds, Some(17));
    assert_eq!(state.attach_from_job_event(&high).unwrap().snapshot.primary_analysis.unwrap().visits, 10000);
    let before = state.serialize().unwrap();
    std::fs::write(engine.directory.join("analysis-frame"), payload(1000, "E5", "-0.3")).unwrap();
    engine.manager.local_focus_action(app_model::LocalFocusRequestDto { run_id: high.run_id.clone(), job_id: high.job_id.clone(), generation: high.generation, node_path: high.node_path.clone(), action: app_model::LocalFocusActionDto::Add { vertex: MoveVertex::Point(app_model::PointDto { x: 4, y: 4 }), max_visits: 5000 } }).unwrap();
    assert_eq!(state.serialize().unwrap(), before, "intent does not overwrite the displayed cache");
    assert!(state.attach_from_job_event(&high).is_none(), "retired output cannot publish");
    let low = next();
    let mut retired_run = low.clone();
    retired_run.run_id = uuid::Uuid::new_v4().to_string();
    let mut retired_board = low.clone();
    retired_board.generation += 1;
    let mut retired_node = low.clone();
    retired_node.node_path = NodePath { indices: vec![0] };
    let mut retired_lineage = low.clone();
    retired_lineage.local_focus_output.as_mut().unwrap().lineage += 1;
    for stale in [retired_run, retired_board, retired_node, retired_lineage] {
        assert!(state.attach_from_job_event(&stale).is_none());
        assert_eq!(state.serialize().unwrap(), before);
    }
    let adopted = state.attach_from_job_event(&low).unwrap();
    let primary = adopted.snapshot.primary_analysis.unwrap();
    assert_eq!(primary.visits, 1000);
    assert_eq!(primary.candidates.len(), 1);
    assert_eq!(primary.candidates[0].order, Some(0));
    assert_eq!(primary.candidates[0].vertex, MoveVertex::Point(app_model::PointDto { x: 4, y: 4 }));
    assert!(primary.ownership.unwrap().iter().all(|value| (*value + 0.3).abs() < 0.001));
    assert_eq!(adopted.snapshot.secondary_analysis.unwrap().visits, 999);
    std::fs::write(engine.directory.join("analysis-frame"), "info move F6 visits 1000 winrate 0.6 order 0 pv F6 rootInfo visits 1000 winrate 0.6").unwrap();
    let retained = state.serialize().unwrap();
    settings.analysis_refresh_interval_centiseconds = app_model::AnalysisRefreshIntervalDto(37);
    let before_failure = engine.manager.snapshot().local_focus;
    assert!(preferences.save(&engine.directory, &engine.manager, settings.clone()).is_err());
    assert_eq!(engine.manager.snapshot().local_focus, before_failure);
    assert_eq!(app_preferences::load_from_path(&prefs_path).unwrap().preferences.analysis_refresh_interval_centiseconds.0, 17);
    preferences.save(&prefs_path, &engine.manager, settings).unwrap();
    assert!(engine.manager.snapshot().local_focus.points.is_empty());
    assert!(state.attach_from_job_event(&low).is_none());
    assert_eq!(state.serialize().unwrap(), retained);
    let same_tree = next();
    assert_eq!(same_tree.job_id, high.job_id);
    assert_eq!(engine.manager.snapshot().local_focus.actual_interval_centiseconds, Some(37));
    assert_eq!(app_preferences::load_from_path(&prefs_path).unwrap().preferences.analysis_refresh_interval_centiseconds.0, 37);
    let continued = state.attach_from_job_event(&same_tree).unwrap();
    assert!(continued.snapshot.primary_analysis.unwrap().ownership.unwrap().iter().all(|value| (*value + 0.3).abs() < 0.001));
    loop {
        if let ForegroundEngineEventDto::Job { job } = engine.events.recv_timeout(Duration::from_secs(5)).unwrap() {
            if job.outcome == AnalysisJobOutcomeDto::TimeLimited {
                let final_result = state.attach_from_job_event(&job).expect("fully drained current limited result remains attachable");
                let final_frame = final_result.snapshot.primary_analysis.unwrap();
                assert_eq!(final_frame.visits, 32);
                assert_eq!(final_frame.winrate_black, 0.01);
                assert!(final_frame.ownership.unwrap().iter().all(|value| (*value + 0.3).abs() < 0.001));
                break;
            }
        }
    }
    let finite = crate::bind_selected_node_job(&state, high.run_id.clone(), high.generation, NodePath::default(), AnalysisJobModeDto::Finite, Some(16)).unwrap();
    let started = engine.manager.start_selected_node_job(finite).unwrap();
    loop {
        if let ForegroundEngineEventDto::Job { job } = engine.events.recv_timeout(Duration::from_secs(5)).unwrap() {
            if job.job_id == started.job_id && job.outcome == AnalysisJobOutcomeDto::Completed {
                assert!(job.frame.as_ref().unwrap().ownership.is_none(), "new tree cannot inherit the previous tree map");
                assert!(state.attach_from_job_event(&job).unwrap().snapshot.primary_analysis.unwrap().ownership.is_none(), "new tree clears stored ownership instead of splicing it");
                break;
            }
        }
    }
    assert!(engine.manager.snapshot().local_focus.points.is_empty());
    let path = engine.directory.join("focus.sgf");
    state.save_to_path(path.to_string_lossy().into(), NodePath::default()).unwrap();
    let bytes = std::fs::read_to_string(path).unwrap();
    let reopened = CurrentSgfDocument::open(&bytes).unwrap().snapshot(&NodePath::default()).unwrap();
    assert_eq!(reopened.primary_analysis.unwrap().visits, 1000);
    assert_eq!(reopened.secondary_analysis.unwrap().visits, 999);
    assert!(!bytes.contains("target_visits"));
    assert!(!bytes.contains("withdrawal_pending"));
}

#[test]
#[ignore = "requires explicit real KataGo binary/model and fresh private smoke directory"]
fn real_cpu_node_analysis_reuse_smoke() {
    let binary = std::env::var("LIZZIEYZY_KATAGO_BINARY").expect("real binary required");
    let model = std::env::var("LIZZIEYZY_KATAGO_MODEL").expect("real model required");
    let directory = std::path::PathBuf::from(std::env::var("LIZZIEYZY_REUSE_SMOKE_DIR").expect("private directory required"));
    std::fs::create_dir(&directory).expect("fresh private smoke directory");
    let config = directory.join("reuse.cfg");
    std::fs::write(&config, "rules = chinese\nnumSearchThreads = 1\nnnMaxBatchSize = 1\nnnCacheSizePowerOfTwo = 16\nnnMutexPoolSizePowerOfTwo = 10\nlogDir = logs\nlogAllGTPCommunication = true\nlogSearchInfo = true\nlogToStderr = true\n").unwrap();
    let catalog = Arc::new(InMemoryEngineProfileCatalog::new());
    catalog.upsert(engine_manager::SavedEngineProfile {
        measured_scenes: Default::default(),
        profile_id: "real-reuse".into(), thread_policy: Default::default(),
        profile: app_model::EngineProfileDto {
            name: "Real CPU reuse smoke".into(), program: binary, argv: vec![],
            working_dir: Some(directory.to_string_lossy().into()),
            adapter: app_model::EngineAdapterSettings::KataGoGtp(app_model::KataGoSettings {
                model_path: Some(model), config_path: Some(config.to_string_lossy().into()), max_visits: 64,
            }),
        },
    });
    let manager = ForegroundEngineManager::new(catalog, ForegroundEngineConfig::default());
    struct Stop(ForegroundEngineManager);
    impl Drop for Stop { fn drop(&mut self) { let _ = self.0.teardown(); } }
    let _stop = Stop(manager.clone());
    let events = manager.subscribe();
    let state = CurrentGameState::default();
    state.connect_analysis_manager(manager.clone());
    let preferences = crate::continuous_analysis::PreferencesState::default();
    let preference_path = directory.join("preferences.json");
    let mut settings = preferences.load(&preference_path, &manager).unwrap().preferences;
    settings.continuous_analysis_enabled = false;
    preferences.save(&preference_path, &manager, settings.clone()).unwrap();
    let opened = state.replace("(;SZ[9]RU[Chinese]KM[7.5]C[real CPU reuse];B[dd])", None).unwrap();
    manager.start("real-reuse").unwrap();
    let deadline = Instant::now() + Duration::from_secs(120);
    let run = loop {
        match manager.snapshot().lifecycle {
            ForegroundEngineLifecycleDto::Ready { run } => break run,
            ForegroundEngineLifecycleDto::Error { failure, .. } => panic!("{failure:?}"),
            _ => { assert!(Instant::now() < deadline, "{:?}", manager.snapshot()); std::thread::sleep(Duration::from_millis(20)); }
        }
    };
    let revision = run.qualified_resource.as_ref().unwrap().profile_revision.clone();
    let identity = || app_model::RuntimeControlIdentityDto { run_id: run.run_id.clone(), profile_revision: revision.clone(), request_id: uuid::Uuid::new_v4().to_string() };
    let parameters = manager.read_runtime_parameters(identity()).unwrap();
    assert_eq!(parameters.status, app_model::RuntimeParametersStatusDto::Confirmed, "{parameters:?}");
    let threads = manager.runtime_threads(app_model::RuntimeThreadsRequestDto { identity: identity(), action: app_model::RuntimeThreadsActionDto::Read, value: None }).unwrap();
    assert_eq!(threads.status, app_model::RuntimeThreadsStatusDto::Confirmed, "{threads:?}");
    println!("qualified Run={} resource={:?} runtime={parameters:?} threads={threads:?}", run.run_id, run.qualified_resource);
    let calculate = |visits, path: NodePath| {
        state.select_path(path.clone(), opened.generation).unwrap();
        let request = crate::bind_selected_node_job(&state, run.run_id.clone(), opened.generation, path, AnalysisJobModeDto::Finite, Some(visits)).unwrap();
        let started = manager.start_selected_node_job(request).unwrap();
        let deadline = Instant::now() + Duration::from_secs(120);
        loop {
            if let ForegroundEngineEventDto::Job { job } = events.recv_timeout(deadline.saturating_duration_since(Instant::now())).unwrap() {
                if job.job_id != started.job_id { continue; }
                assert!(!matches!(job.outcome, AnalysisJobOutcomeDto::Failed | AnalysisJobOutcomeDto::Timeout), "{job:?}");
                if job.outcome == AnalysisJobOutcomeDto::Completed { break job; }
            }
        }
    };
    let high = calculate(512, NodePath::default());
    assert!(manager.ordinary_analysis_context(&high).is_some());
    let high_visits = high.frame.as_ref().unwrap().visits;
    state.attach_from_job_event(&high).unwrap();
    let preserved = state.serialize().unwrap();
    // A genuine different-position calculation leaves the prior search tree;
    // returning to the same SGF node tests retained cache, not engine tree reuse.
    let _other = calculate(8, NodePath { indices: vec![0] });
    let low = calculate(8, NodePath::default());
    let low_visits = low.frame.as_ref().unwrap().visits;
    assert!(low_visits < high_visits, "high={high_visits} low={low_visits}");
    assert!(state.attach_from_job_event(&low).is_none());
    assert_eq!(state.serialize().unwrap(), preserved);
    settings.retain_node_analysis = false;
    preferences.save(&preference_path, &manager, settings.clone()).unwrap();
    assert_eq!(state.serialize().unwrap(), preserved);
    let _other = calculate(8, NodePath { indices: vec![0] });
    let off = calculate(8, NodePath::default());
    let off_visits = off.frame.as_ref().unwrap().visits;
    let adopted = state.attach_from_job_event(&off).unwrap();
    assert_eq!(adopted.snapshot.primary_analysis.unwrap().visits, off_visits);
    settings.retain_node_analysis = true;
    preferences.save(&preference_path, &manager, settings).unwrap();
    let path = directory.join("result.sgf");
    state.save_to_path(path.to_string_lossy().into(), NodePath::default()).unwrap();
    let reopened = CurrentSgfDocument::open(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(reopened.snapshot(&NodePath::default()).unwrap().primary_analysis.unwrap().visits, off_visits);
    let ForegroundEngineLifecycleDto::Ready { run: final_run } = manager.snapshot().lifecycle else { panic!("real Run must remain ready"); };
    assert_eq!(final_run.run_id, run.run_id);
    assert_eq!(final_run.qualified_resource, run.qualified_resource);
    println!("REAL CPU REUSE PASS: incoming high={high_visits}, low={low_visits} retained, disabled incoming={off_visits} adopted, saved/reopened={}; same Run, no retry/restart", path.display());
    manager.teardown().unwrap();
}
