use super::*;
use std::os::unix::fs::PermissionsExt;

#[test]
#[ignore = "requires explicitly supplied read-only CPU KataGo and model"]
fn real_cpu_auto_load_attachment_pause_and_confirmed_recovery() {
    let binary = std::env::var("R13_KATAGO").expect("R13_KATAGO");
    let model = std::env::var("R13_MODEL").expect("R13_MODEL");
    let evidence = std::path::PathBuf::from(std::env::var("R13_EVIDENCE").expect("R13_EVIDENCE"));
    std::fs::create_dir_all(&evidence).unwrap();
    let directory = evidence.join("runtime");
    std::fs::create_dir(&directory).unwrap();
    std::fs::write(directory.join("cpu.cfg"), "numAnalysisThreads = 1\nnumSearchThreads = 1\nnnCacheSizePowerOfTwo = 16\nnnMutexPoolSizePowerOfTwo = 12\nmaxVisits = 8\nlogToStderr = true\nlogSearchInfo = false\n").unwrap();
    // Transparent test transport: real KataGo supplies every response. The one
    // fault is a process exit before the post-auto ordinary handback can answer.
    let proxy = directory.join("transport.py");
    std::fs::write(&proxy, r#"#!/usr/bin/env python3
import subprocess, sys, threading, json, pathlib, os
p = subprocess.Popen([os.environ['R13_KATAGO']] + sys.argv[1:], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=open('engine.stderr','a'), text=True, bufsize=1)
with open('pids.log','a') as f: f.write(str(p.pid)+'\n')
def output():
    for line in p.stdout:
        with open('responses.jsonl','a') as f: f.write(line)
        sys.stdout.write(line); sys.stdout.flush()
threading.Thread(target=output, daemon=True).start()
try:
    for line in sys.stdin:
        q=json.loads(line)
        with open('queries.jsonl','a') as f: f.write(line)
        if pathlib.Path('fail-handback').exists() and ':' not in q.get('id','') and q.get('reportDuringSearchEvery') and q.get('action') != 'terminate':
            p.terminate(); p.wait(); sys.exit(9)
        p.stdin.write(line); p.stdin.flush()
finally:
    if p.poll() is None: p.terminate()
    p.wait()
"#).unwrap();
    std::fs::set_permissions(&proxy, std::fs::Permissions::from_mode(0o755)).unwrap();
    let catalog = Arc::new(InMemoryEngineProfileCatalog::new());
    catalog.upsert(engine_manager::SavedEngineProfile { profile_id: "test".into(), thread_policy: Default::default(), profile: app_model::EngineProfileDto { name: "R13 isolated CPU".into(), program: proxy.to_string_lossy().into(), argv: vec![], working_dir: Some(directory.to_string_lossy().into()), adapter: app_model::EngineAdapterSettings::KataGoAnalysis(app_model::KataGoSettings { model_path: Some(model), config_path: Some(directory.join("cpu.cfg").to_string_lossy().into()), max_visits: 8 }) } });
    let manager = ForegroundEngineManager::new(catalog.clone(), ForegroundEngineConfig { readiness_timeout: Duration::from_secs(120), job_timeout: Duration::from_secs(120), ..Default::default() });
    let events = manager.subscribe();
    let engine = LiveFixture { directory: directory.clone(), manager, catalog, events };
    let state = CurrentGameState::default();
    state.connect_analysis_manager(engine.manager.clone());
    engine.manager.set_auto_load_preferences(app_model::AutoLoadPreferencesDto { enabled: true, total_visits: 8 }).unwrap();
    engine.manager.start("test").unwrap();
    drive(&engine, &state, || matches!(engine.manager.snapshot().lifecycle, app_model::ForegroundEngineLifecycleDto::Ready { .. }));
    let initial = engine.manager.snapshot();
    std::fs::write(evidence.join("ready.json"), serde_json::to_vec_pretty(&initial).unwrap()).unwrap();
    let opened = load(&engine, &state, "(;SZ[9]RU[Chinese]KM[7.5]AB[aa]PL[W]C[personal];W[];B[cc](;W[dd])(;W[ee]))", Source::Open);
    engine.manager.set_continuous_preferences(true, app_model::ContinuousAnalysisBudgetDto::default()).unwrap();
    let selected = NodePath { indices: vec![0,0,1] };
    state.select_path(selected.clone(), opened.generation).unwrap();
    drive(&engine, &state, || CurrentSgfDocument::open(&state.serialize().unwrap()).unwrap().missing_primary_mainline_snapshots().unwrap().is_empty());
    assert_eq!(state.holder.lock().expect("holder").current_result().unwrap().selected_path, selected);
    drive(&engine, &state, || CurrentSgfDocument::open(&state.serialize().unwrap()).unwrap().snapshot(&selected).unwrap().primary_analysis.is_some());
    let handback = engine.manager.snapshot();
    assert_eq!(handback.selected_node_job.as_ref().unwrap().node_path, selected);
    std::fs::write(evidence.join("handback.json"), serde_json::to_vec_pretty(&handback).unwrap()).unwrap();
    engine.manager.set_continuous_preferences(false, app_model::ContinuousAnalysisBudgetDto::default()).unwrap();
    let save = evidence.join("complete.sgf");
    state.save_to_path(save.to_string_lossy().into(), selected).unwrap();
    let saved = std::fs::read_to_string(&save).unwrap();
    assert_eq!(CurrentSgfDocument::open(&saved).unwrap().snapshot(&NodePath::default()).unwrap().personal_comment, "personal");
    load(&engine, &state, &saved, Source::Recent);
    assert_eq!(engine.manager.snapshot().auto_load.phase, AutoPhase::NotNeeded);
    engine.manager.set_auto_load_preferences(app_model::AutoLoadPreferencesDto { enabled: true, total_visits: 100000 }).unwrap();
    load(&engine, &state, "(;SZ[9];B[dd];W[ee])", Source::Text);
    let paused = engine.manager.analysis_task_snapshot().unwrap();
    engine.manager.pause_analysis_task(&paused.run_id, &paused.task_id).unwrap();
    drive(&engine, &state, || engine.manager.snapshot().whole_game_job.is_none());
    assert_eq!(engine.manager.snapshot().auto_load.phase, AutoPhase::Cancelled);
    engine.manager.set_auto_load_preferences(app_model::AutoLoadPreferencesDto { enabled: true, total_visits: 8 }).unwrap();
    std::fs::write(directory.join("fail-handback"), "").unwrap();
    load(&engine, &state, "(;SZ[9]RU[Chinese]KM[7.5];B[dd])", Source::Open);
    engine.manager.set_continuous_preferences(true, app_model::ContinuousAnalysisBudgetDto::default()).unwrap();
    drive(&engine, &state, || engine.manager.snapshot().auto_load_recovery == Some(RecoveryPhase::RestartRequired));
    std::fs::remove_file(directory.join("fail-handback")).unwrap();
    let imported = load(&engine, &state, "(;SZ[13]RU[Chinese-KGS]KM[6.5]AB[aa]PL[W]C[new personal];W[];B[cc])", Source::Text);
    let unchanged = state.serialize().unwrap();
    assert!(engine.manager.resume_continuous().is_err());
    engine.manager.restart().unwrap();
    drive(&engine, &state, || engine.manager.snapshot().auto_load_recovery == Some(RecoveryPhase::Confirmed));
    assert_eq!(state.serialize().unwrap(), unchanged);
    std::fs::write(evidence.join("confirmed.json"), serde_json::to_vec_pretty(&engine.manager.snapshot()).unwrap()).unwrap();
    engine.manager.resume_continuous().unwrap();
    drive(&engine, &state, || engine.manager.snapshot().selected_node_job.as_ref().is_some_and(|j| j.mode == app_model::AnalysisJobModeDto::Continuous && j.generation == imported.generation));
    drive(&engine, &state, || CurrentSgfDocument::open(&state.serialize().unwrap()).unwrap().snapshot(&imported.selected_path).unwrap().primary_analysis.is_some());
    std::fs::write(evidence.join("continued.json"), serde_json::to_vec_pretty(&engine.manager.snapshot()).unwrap()).unwrap();
    engine.manager.teardown().unwrap();
    for file in ["queries.jsonl", "responses.jsonl", "engine.stderr", "pids.log", "cpu.cfg"] { std::fs::copy(directory.join(file), evidence.join(file)).unwrap(); }
    std::fs::write(evidence.join("result.txt"), format!("PASS real CPU auto load, exact attachment, browse, Save/reopen, Pause, failed handback, new import, explicit Restart, new-reader terminal receipt, explicit Continue\nbinary={binary}\n")).unwrap();
}

fn drive(engine: &LiveFixture, state: &CurrentGameState, done: impl Fn() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(150);
    while !done() {
        assert!(Instant::now() < deadline, "CPU smoke timed out: {:?}", engine.manager.snapshot());
        match engine.events.recv_timeout(Duration::from_millis(20)) {
            Ok(app_model::ForegroundEngineEventDto::Job { job }) => { state.attach_from_job_event(&job); }
            Ok(app_model::ForegroundEngineEventDto::Snapshot { .. }) => state.reconcile_auto_load(),
            Ok(_) | Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {},
            Err(error) => panic!("events: {error}"),
        }
    }
}
