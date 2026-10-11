use super::*;
use app_model::{ContinuationModeDto as Mode, ContinuationPhaseDto as Phase, ContinuationPreferencesDto};
use std::time::{Duration, Instant};

#[path = "continuation_cpu_smoke.rs"]
mod cpu_smoke;

fn setup(text: &str, path: Vec<u32>) -> (LiveFixture, CurrentGameState) {
    let engine = task_fixture();
    std::fs::write(engine.directory.join("cancel-final"), "").unwrap();
    let state = CurrentGameState::default();
    state.connect_analysis_manager(engine.manager.clone());
    engine.manager.set_continuous_preferences(false, app_model::ContinuousAnalysisBudgetDto {
        continuous_time_limit_enabled: false, continuous_visits_limit_enabled: false,
        ..Default::default()
    }).unwrap();
    let game = state.replace(text, None).unwrap();
    state.select_path(NodePath { indices: path }, game.generation).unwrap();
    engine.manager.start("test").unwrap();
    let deadline = Instant::now() + Duration::from_secs(4);
    while !matches!(engine.manager.snapshot().lifecycle, app_model::ForegroundEngineLifecycleDto::Ready { .. }) {
        assert!(Instant::now() < deadline);
        let _ = engine.events.recv_timeout(Duration::from_millis(20));
    }
    (engine, state)
}
fn begin(engine: &LiveFixture, state: &CurrentGameState, mode: Mode, choices: Vec<app_model::AnalysisBranchChoiceDto>) -> app_model::ContinuationSnapshotDto {
    let current = state.holder.lock().unwrap().current_result().unwrap();
    state.start_continuation(&engine.manager, current.generation, current.selected_path, choices,
        ContinuationPreferencesDto { mode, interval_seconds: 1.0 }, engine.manager.continuation_epoch()).unwrap()
}
fn frame(engine: &LiveFixture, request: &str) -> app_model::AnalysisJobEventDto {
    let deadline = Instant::now() + Duration::from_secs(4);
    loop {
        assert!(Instant::now() < deadline, "missing frame for {request}: {:?}", engine.manager.snapshot());
        if let Ok(app_model::ForegroundEngineEventDto::Job { job }) = engine.events.recv_timeout(Duration::from_millis(20)) {
            if job.job_id == request && job.frame.is_some() { return job; }
        }
    }
}
fn current(state: &CurrentGameState) -> CurrentGameResultDto { state.holder.lock().unwrap().current_result().unwrap() }

#[test]
fn continuation_start_rejects_old_cached_and_late_same_run_position_then_requires_fresh_interval() {
    let (engine, state) = setup("(;SZ[9]RU[Chinese]KM[7.5]C[personal];B[aa])", vec![0]);
    let first = begin(&engine, &state, Mode::EveryBeat, vec![]);
    let old = first.request_id.unwrap();
    let old_frame = frame(&engine, &old);
    state.attach_from_job_event(&old_frame);
    state.stop_continuation(&engine.manager, "explicit Stop");
    std::fs::write(engine.directory.join("hold-ordinary"), "").unwrap();
    std::fs::write(engine.directory.join("continuation-late"), &old).unwrap();
    let before = state.serialize().unwrap();
    let origin = Instant::now();
    let next = begin(&engine, &state, Mode::EveryBeat, vec![]);
    assert_eq!(first.run_id, next.run_id);
    assert_ne!(Some(old), next.request_id);
    let session = next.session_id.unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    while !engine.directory.join("continuation-late-sent").exists() {
        assert!(Instant::now() < deadline);
        let _ = engine.events.recv_timeout(Duration::from_millis(10));
    }
    assert_eq!(state.continuation_beat_at(&engine.manager, &session, origin + Duration::from_secs(3)).phase, Phase::Waiting);
    assert_eq!(state.serialize().unwrap(), before);
    std::fs::remove_file(engine.directory.join("hold-ordinary")).unwrap();
    frame(&engine, next.request_id.as_deref().unwrap());
    assert!(state.continuation_beat_at(&engine.manager, &session, origin + Duration::from_millis(999)).current_game.is_none());
    let advanced = state.continuation_beat_at(&engine.manager, &session, origin + Duration::from_secs(4));
    assert_eq!(advanced.current_game.as_ref().unwrap().snapshot.position.move_number, 2);
    assert_ne!(advanced.request_id, next.request_id);
    let after = state.serialize().unwrap();
    assert!(state.continuation_beat_at(&engine.manager, &session, origin + Duration::from_secs(4)).current_game.is_none());
    assert_eq!(state.serialize().unwrap(), after);
    let stopped = state.stop_continuation(&engine.manager, "Stop while waiting");
    assert_eq!(stopped.phase, Phase::Stopped);
    state.continuation_beat_at(&engine.manager, &session, origin + Duration::from_secs(30));
    assert_eq!(state.serialize().unwrap(), after);
}

#[test]
fn continuation_leaf_freezes_nonfirst_branch_then_creates_one_undoable_move_and_saves() {
    let (engine, state) = setup("(;SZ[9]RU[Chinese]C[root];B[aa](;W[bb]C[old])(;W[cc]C[selected]))", vec![0]);
    let origin = Instant::now();
    let started = begin(&engine, &state, Mode::LeafOnly, vec![app_model::AnalysisBranchChoiceDto { parent: NodePath { indices: vec![0] }, child: 1 }]);
    frame(&engine, started.request_id.as_deref().unwrap());
    let session = started.session_id.unwrap();
    let navigated = state.continuation_beat_at(&engine.manager, &session, origin + Duration::from_secs(2));
    let game = navigated.current_game.unwrap();
    assert_eq!(game.selected_path.indices, vec![0,1]);
    assert!(!game.dirty); assert!(!game.can_undo);
    frame(&engine, navigated.request_id.as_deref().unwrap());
    assert!(state.continuation_beat_at(&engine.manager, &session, origin + Duration::from_millis(2999)).current_game.is_none());
    let added = state.continuation_beat_at(&engine.manager, &session, origin + Duration::from_secs(3));
    let game = added.current_game.unwrap();
    assert_eq!(game.selected_path.indices, vec![0,1,0]); assert!(game.dirty && game.can_undo);
    state.stop_continuation(&engine.manager, "Stop");
    let saved = engine.directory.join("continued.sgf");
    state.save_to_path(saved.to_string_lossy().into(), game.selected_path.clone()).unwrap();
    let parsed = CurrentSgfDocument::open(&std::fs::read_to_string(saved).unwrap()).unwrap();
    assert_eq!(parsed.snapshot(&NodePath { indices: vec![0,0] }).unwrap().personal_comment, "old");
    assert_eq!(parsed.snapshot(&NodePath { indices: vec![0,1] }).unwrap().personal_comment, "selected");
    assert_eq!(parsed.snapshot(&game.selected_path).unwrap().position.move_number, 3);
    let undone = state.undo(game.generation).unwrap();
    assert_eq!(undone.selected_path.indices, vec![0,1]); assert!(!undone.can_undo);
}

#[test]
fn continuation_every_beat_reuses_identical_child_or_appends_without_following_recorded_choice() {
    for (sgf, expected, edited) in [
        ("(;SZ[9]RU[Chinese];B[aa](;W[bb])(;W[df]C[reuse]))", vec![0,1], false),
        ("(;SZ[9]RU[Chinese];B[aa](;W[bb])(;W[cc]))", vec![0,2], true),
    ] {
        let (engine, state) = setup(sgf, vec![0]);
        let started = begin(&engine, &state, Mode::EveryBeat, vec![]);
        frame(&engine, started.request_id.as_deref().unwrap());
        let moved = state.continuation_beat_at(&engine.manager, started.session_id.as_deref().unwrap(), Instant::now() + Duration::from_secs(2));
        let game = moved.current_game.unwrap();
        assert_eq!(game.selected_path.indices, expected);
        assert_eq!(game.can_undo, edited); assert_eq!(game.dirty, edited);
        assert_eq!(game.tree.children[0].children.len(), if edited { 3 } else { 2 });
        state.stop_continuation(&engine.manager, "Stop");
    }
}

#[test]
fn continuation_double_actual_pass_initial_terminal_and_illegal_candidate_stop_without_retry() {
    for mode in [Mode::LeafOnly, Mode::EveryBeat] {
        let (engine, state) = setup("(;SZ[9]RU[Chinese]C[keep])", vec![]);
        std::fs::write(engine.directory.join("continuation-move"), "pass").unwrap();
        let started = begin(&engine, &state, mode, vec![]);
        frame(&engine, started.request_id.as_deref().unwrap());
        let time = Instant::now() + Duration::from_secs(2);
        let first = state.continuation_beat_at(&engine.manager, started.session_id.as_deref().unwrap(), time);
        frame(&engine, first.request_id.as_deref().unwrap());
        let second = state.continuation_beat_at(&engine.manager, started.session_id.as_deref().unwrap(), time + Duration::from_secs(1));
        assert_eq!(second.phase, Phase::Terminal);
        assert_eq!(second.current_game.unwrap().snapshot.position.move_number, 2);
        let terminal = begin(&engine, &state, mode, vec![]);
        assert_eq!(terminal.phase, Phase::Terminal); assert!(terminal.request_id.is_none());
    }
    let (engine, state) = setup("(;SZ[9]RU[Chinese];B[df])", vec![0]);
    let started = begin(&engine, &state, Mode::EveryBeat, vec![]);
    frame(&engine, started.request_id.as_deref().unwrap());
    let before = state.serialize().unwrap();
    let failed = state.continuation_beat_at(&engine.manager, started.session_id.as_deref().unwrap(), Instant::now() + Duration::from_secs(2));
    assert_eq!(failed.phase, Phase::Failed); assert!(failed.reason.is_some());
    assert_eq!(state.serialize().unwrap(), before);
    assert_eq!(current(&state).snapshot.position.move_number, 1);
}

#[test]
fn continuation_external_navigation_comment_and_undo_seal_before_late_beat() {
    for action in ["navigation", "comment", "undo", "komi", "play", "branch", "setup"] {
        let (engine, state) = setup("(;SZ[9]RU[Chinese];B[aa](;W[bb])(;W[cc]))", vec![0]);
        if action == "undo" { state.play(NodePath { indices: vec![0] }, MoveVertex::Pass).unwrap(); }
        let started = begin(&engine, &state, Mode::EveryBeat, vec![]);
        frame(&engine, started.request_id.as_deref().unwrap());
        let current = current(&state);
        match action {
            "navigation" => { state.select_path(NodePath::default(), current.generation).unwrap(); }
            "comment" => { state.set_personal_comment(current.selected_path, "manual".into()).unwrap(); }
            "komi" => { state.set_metadata(current.generation, "B".into(), "W".into(), 6.5).unwrap(); }
            "play" => { state.play(current.selected_path, MoveVertex::Pass).unwrap(); }
            "branch" => { state.promote_to_main(NodePath { indices: vec![0,1] }, current.generation).unwrap(); }
            "setup" => { state.convert_to_root_setup(current.generation, current.selected_path).unwrap(); }
            _ => { state.undo(current.generation).unwrap(); }
        }
        let before = state.serialize().unwrap();
        let late = state.continuation_beat_at(&engine.manager, started.session_id.as_deref().unwrap(), Instant::now() + Duration::from_secs(5));
        assert_eq!(late.phase, Phase::Stopped);
        assert_eq!(state.serialize().unwrap(), before);
    }
}

#[test]
fn continuation_pending_start_is_sealed_and_empty_result_never_fakes_pass() {
    let (engine, state) = setup("(;SZ[9]RU[Chinese])", vec![]);
    let epoch = engine.manager.continuation_epoch();
    state.stop_continuation(&engine.manager, "Close before pending Start acquired document");
    let game = current(&state);
    assert!(state.start_continuation(&engine.manager, game.generation, game.selected_path, vec![],
        ContinuationPreferencesDto::default(), epoch).is_err());
    assert!(engine.manager.continuation_snapshot().session_id.is_none());
    std::fs::write(engine.directory.join("continuation-move"), "empty").unwrap();
    let started = begin(&engine, &state, Mode::EveryBeat, vec![]);
    frame(&engine, started.request_id.as_deref().unwrap());
    let before = state.serialize().unwrap();
    let failed = state.continuation_beat_at(&engine.manager, started.session_id.as_deref().unwrap(), Instant::now() + Duration::from_secs(2));
    assert_eq!(failed.phase, Phase::Failed);
    assert_eq!(state.serialize().unwrap(), before);
}

#[test]
fn continuation_trial_and_run_retirement_prevent_late_actions() {
    for trial in [false, true] {
        let (engine, state) = setup("(;SZ[9]RU[Chinese])", vec![]);
        let started = begin(&engine, &state, Mode::EveryBeat, vec![]);
        frame(&engine, started.request_id.as_deref().unwrap());
        if trial { state.enter_trial().unwrap(); } else { engine.manager.teardown().unwrap(); }
        let before = state.serialize().unwrap();
        let late = state.continuation_beat_at(&engine.manager, started.session_id.as_deref().unwrap(), Instant::now() + Duration::from_secs(2));
        assert!(matches!(late.phase, Phase::Stopped | Phase::Failed));
        assert_eq!(state.serialize().unwrap(), before);
        if trial {
            let game = current(&state);
            assert!(state.start_continuation(&engine.manager, game.generation, game.selected_path, vec![], ContinuationPreferencesDto::default(), engine.manager.continuation_epoch()).is_err());
        }
    }
}

#[test]
fn continuation_retires_auto_load_and_protected_load_seals_session() {
    let (engine, state) = setup("(;SZ[9]RU[Chinese])", vec![]);
    engine.manager.set_auto_load_preferences(app_model::AutoLoadPreferencesDto { enabled: true, total_visits: 8 }).unwrap();
    std::fs::write(engine.directory.join("hold-first"), "").unwrap();
    super::auto_load_tests::load(&engine, &state, "(;SZ[9]RU[Chinese];B[aa])", app_model::GameLoadSourceDto::Open);
    state.reconcile_auto_load();
    let automatic = engine.manager.analysis_task_snapshot().expect("automatic owner");
    let started = begin(&engine, &state, Mode::EveryBeat, vec![]);
    assert_ne!(started.request_id.as_ref(), Some(&automatic.job_id));
    assert!(!matches!(engine.manager.snapshot().auto_load.phase, app_model::AutoLoadPhaseDto::Running));
    frame(&engine, started.request_id.as_deref().unwrap());
    super::auto_load_tests::load(&engine, &state, "(;SZ[13]RU[Chinese]KM[6.5])", app_model::GameLoadSourceDto::Excluded);
    let before = state.serialize().unwrap();
    let late = state.continuation_beat_at(&engine.manager, started.session_id.as_deref().unwrap(), Instant::now() + Duration::from_secs(2));
    assert_eq!(late.phase, Phase::Stopped); assert_eq!(state.serialize().unwrap(), before);
}

#[test]
fn continuation_cannot_steal_armed_waiting_batch_and_match_seals_direct_manager_admission() {
    use app_model::{BatchCommandDto as Command, BatchOutputAuthorizationDto as Output};
    let (engine, state) = setup("(;SZ[9]RU[Chinese];B[aa])", vec![]);
    std::fs::write(engine.directory.join("hold-first"), "").unwrap();
    let game = current(&state);
    let run = match engine.manager.snapshot().lifecycle { app_model::ForegroundEngineLifecycleDto::Ready { run } => run.run_id, _ => unreachable!() };
    state.start_first_child_analysis(&engine.manager, run, game.generation, 32).unwrap();
    let path = engine.directory.join("waiting-continuation.sgf"); std::fs::write(&path, "(;SZ[9];B[bb])").unwrap();
    engine.manager.batch_command(Command::Intake { paths: vec![path.to_string_lossy().into()], conditions: Default::default() }).unwrap();
    engine.manager.batch_command(Command::Start { output: Output::SessionOnly }).unwrap();
    assert!(engine.manager.batch_occupied());
    let error = state.start_continuation(&engine.manager, game.generation, game.selected_path, vec![], ContinuationPreferencesDto::default(), engine.manager.continuation_epoch()).unwrap_err();
    assert!(error.to_lowercase().contains("batch"), "{error}");
    engine.manager.batch_command(Command::StopAll).unwrap();
    engine.manager.teardown().unwrap();
    let (engine, state) = setup("(;SZ[9]RU[Chinese])", vec![]);
    let started = begin(&engine, &state, Mode::EveryBeat, vec![]);
    frame(&engine, started.request_id.as_deref().unwrap());
    engine.manager.reserve_match("continuation-match-fence").unwrap();
    let before = state.serialize().unwrap();
    let late = state.continuation_beat_at(&engine.manager, started.session_id.as_deref().unwrap(), Instant::now() + Duration::from_secs(2));
    assert_eq!(late.phase, Phase::Stopped); assert_eq!(state.serialize().unwrap(), before);
}

#[test]
fn continuation_engine_failure_stops_with_reason_without_an_extra_move_or_retry() {
    let (engine, state) = setup("(;SZ[9]RU[Chinese];B[aa])", vec![0]);
    let started = begin(&engine, &state, Mode::EveryBeat, vec![]);
    frame(&engine, started.request_id.as_deref().unwrap());
    let before = state.serialize().unwrap();
    std::fs::write(engine.directory.join("fail-handback"), "").unwrap();
    let deadline = Instant::now() + Duration::from_secs(4);
    while !matches!(engine.manager.snapshot().lifecycle, app_model::ForegroundEngineLifecycleDto::Error { .. }) {
        assert!(Instant::now() < deadline, "engine failure was not published");
        let _ = engine.events.recv_timeout(Duration::from_millis(10));
    }
    let requests = std::fs::read(engine.directory.join("queries.jsonl")).unwrap();
    for seconds in [2, 20, 200] {
        let stopped = state.continuation_beat_at(&engine.manager, started.session_id.as_deref().unwrap(), Instant::now() + Duration::from_secs(seconds));
        assert_eq!(stopped.phase, Phase::Failed);
        assert!(!stopped.reason.as_deref().unwrap_or_default().is_empty());
        assert_eq!(state.serialize().unwrap(), before);
    }
    assert_eq!(std::fs::read(engine.directory.join("queries.jsonl")).unwrap(), requests);
    assert_eq!(current(&state).snapshot.position.move_number, 1);
}

#[test]
fn continuation_gtp_position_confirmation_failure_stops_before_candidate_commit() {
    let engine = LiveFixture::new();
    let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../crates/engine-manager/tests/fixtures/rules_process.py");
    std::fs::write(engine.directory.join("engine"), format!("#!/bin/sh\nexec /usr/bin/python3 '{}' \"$@\"\n", script.display())).unwrap();
    std::fs::write(engine.directory.join("mode"), "analysis").unwrap();
    std::fs::write(engine.directory.join("config"), "numSearchThreads = 1\n").unwrap();
    use engine_manager::EngineProfileCatalog;
    let mut profile = engine.catalog.get("test").unwrap();
    profile.profile.adapter = app_model::EngineAdapterSettings::KataGoGtp(app_model::KataGoSettings {
        model_path: Some("model".into()), config_path: Some("config".into()), max_visits: 4,
    });
    engine.catalog.upsert(profile);
    let state = CurrentGameState::default();
    state.connect_analysis_manager(engine.manager.clone());
    engine.manager.set_continuous_preferences(false, Default::default()).unwrap();
    // The controlled GTP child reports 7.5, not the requested 6.5: a real
    // confirmation mismatch, after qualified Ready but before kata-analyze.
    state.replace("(;SZ[9]RU[Chinese]KM[6.5]C[keep])", None).unwrap();
    engine.manager.start("test").unwrap();
    let deadline = Instant::now() + Duration::from_secs(4);
    while !matches!(engine.manager.snapshot().lifecycle, app_model::ForegroundEngineLifecycleDto::Ready { .. }) {
        assert!(Instant::now() < deadline);
        let _ = engine.events.recv_timeout(Duration::from_millis(10));
    }
    let started = begin(&engine, &state, Mode::EveryBeat, vec![]);
    let before = state.serialize().unwrap();
    let deadline = Instant::now() + Duration::from_secs(4);
    loop {
        let stopped = state.continuation_beat_at(&engine.manager, started.session_id.as_deref().unwrap(), Instant::now() + Duration::from_secs(2));
        if stopped.phase == Phase::Failed {
            assert!(stopped.reason.as_deref().unwrap_or_default().to_lowercase().contains("komi"), "{stopped:?}");
            break;
        }
        assert!(Instant::now() < deadline, "{stopped:?}");
        let _ = engine.events.recv_timeout(Duration::from_millis(10));
    }
    let trace = std::fs::read_to_string(engine.directory.join("trace")).unwrap();
    assert!(trace.contains("loadsgf") && trace.contains("get_komi"), "{trace}");
    assert!(!trace.lines().any(|line| line.split_whitespace().nth(1) == Some("kata-analyze")), "{trace}");
    state.continuation_beat_at(&engine.manager, started.session_id.as_deref().unwrap(), Instant::now() + Duration::from_secs(200));
    assert_eq!(state.serialize().unwrap(), before);
    assert_eq!(current(&state).snapshot.position.move_number, 0);
    assert_eq!(std::fs::read_to_string(engine.directory.join("trace")).unwrap(), trace);
}

#[test]
fn continuation_provider_authority_rejects_start_and_seals_cached_and_late_actions() {
    let (engine, state) = setup("(;SZ[9]RU[Chinese];B[aa])", vec![0]);
    let started = begin(&engine, &state, Mode::EveryBeat, vec![]);
    let cached = frame(&engine, started.request_id.as_deref().unwrap());
    let network = provider_core::network::NetworkState::default();
    let locator = "https://home.yikeweiqi.com/#/unite/continuation-authority";
    let id = state.begin_external_start(&network, locator.into(), app_model::YikeSyncPreferencesDto::default()).unwrap();
    let game = current(&state);
    let error = state.start_continuation(&engine.manager, game.generation, game.selected_path, vec![],
        ContinuationPreferencesDto::default(), engine.manager.continuation_epoch()).unwrap_err();
    assert!(error.contains("External sync"), "{error}");
    let before = state.serialize().unwrap();
    let late = state.continuation_beat_at(&engine.manager, started.session_id.as_deref().unwrap(), Instant::now() + Duration::from_secs(2));
    assert_eq!(late.phase, Phase::Stopped);
    assert_eq!(state.serialize().unwrap(), before);
    let imported = provider_yike::import_payload(app_model::ProviderImportRequest {
        provider: app_model::ProviderKind::Yike, payload: "(;SZ[9]RU[Chinese]C[provider];B[bb])".into(),
        source_url: Some(locator.into()), source_id: None,
        metadata: app_model::ProviderGameMetadata { provider_status: Some("playing".into()), ..Default::default() },
    }).unwrap();
    let candidate = state.prepare_external_candidate(id, imported).unwrap();
    let departure = match candidate.admission {
        app_model::DocumentDepartureAdmissionDto::Ready { departure_id }
        | app_model::DocumentDepartureAdmissionDto::NeedsDecision { departure_id } => departure_id,
    };
    state.begin_protected_commit(departure, &[]).unwrap();
    assert!(state.commit_replacement(departure).unwrap().committed);
    assert_eq!(state.external_sync_snapshot().session_id, Some(id));
    let game = current(&state);
    let before = state.serialize().unwrap();
    let error = state.start_continuation(&engine.manager, game.generation, game.selected_path, vec![],
        ContinuationPreferencesDto::default(), engine.manager.continuation_epoch()).unwrap_err();
    assert!(error.contains("External sync"), "{error}");
    assert!(state.attach_from_job_event(&cached).is_none());
    state.continuation_beat_at(&engine.manager, started.session_id.as_deref().unwrap(), Instant::now() + Duration::from_secs(200));
    assert_eq!(state.serialize().unwrap(), before);
}

#[test]
fn continuation_stop_handback_preserves_user_pause_and_healthy_run() {
    let (engine, state) = setup("(;SZ[9]RU[Chinese];B[aa])", vec![0]);
    let preferences = crate::continuous_analysis::PreferencesState::default();
    let path = engine.directory.join("preferences.json");
    let mut saved = preferences.load(&path, &engine.manager).unwrap().preferences;
    saved.continuous_analysis_enabled = true;
    saved.continuous_budget.continuous_time_limit_enabled = false;
    saved.continuous_budget.continuous_visits_limit_enabled = false;
    preferences.save(&path, &engine.manager, saved).unwrap();
    let started = begin(&engine, &state, Mode::EveryBeat, vec![]);
    frame(&engine, started.request_id.as_deref().unwrap());
    let before = state.serialize().unwrap();
    let paused = preferences.primary(&path, &engine.manager).unwrap();
    assert!(!paused.continuous_analysis_enabled);
    state.stop_continuation(&engine.manager, "Stop after user Pause");
    let deadline = Instant::now() + Duration::from_secs(4);
    while engine.manager.snapshot().selected_node_job.is_some() {
        assert!(Instant::now() < deadline, "{:?}", engine.manager.snapshot());
        let _ = engine.events.recv_timeout(Duration::from_millis(10));
    }
    let requests = std::fs::read(engine.directory.join("queries.jsonl")).unwrap();
    for seconds in [2, 20, 200] {
        state.continuation_beat_at(&engine.manager, started.session_id.as_deref().unwrap(), Instant::now() + Duration::from_secs(seconds));
    }
    assert_eq!(state.serialize().unwrap(), before);
    assert_eq!(std::fs::read(engine.directory.join("queries.jsonl")).unwrap(), requests);
    assert!(engine.manager.snapshot().selected_node_job.is_none());
    let run = match engine.manager.snapshot().lifecycle { app_model::ForegroundEngineLifecycleDto::Ready { run } => run.run_id, other => panic!("{other:?}") };
    assert_eq!(Some(run), started.run_id);
    assert!(!app_preferences::load_from_path(&path).unwrap().preferences.continuous_analysis_enabled);
}

#[test]
fn continuation_explicit_restart_captures_current_node_with_new_request_and_full_first_interval() {
    let (engine, state) = setup("(;SZ[9]RU[Chinese];B[aa])", vec![0]);
    let first = begin(&engine, &state, Mode::EveryBeat, vec![]);
    frame(&engine, first.request_id.as_deref().unwrap());
    let advanced = state.continuation_beat_at(&engine.manager, first.session_id.as_deref().unwrap(), Instant::now() + Duration::from_secs(2));
    assert_eq!(advanced.current_game.as_ref().unwrap().selected_path.indices, vec![0, 0]);
    let old = advanced.request_id.as_deref().unwrap();
    let cached = frame(&engine, old);
    state.attach_from_job_event(&cached);
    state.stop_continuation(&engine.manager, "explicit Stop at newly created node");
    std::fs::write(engine.directory.join("hold-ordinary"), "").unwrap();
    std::fs::write(engine.directory.join("continuation-late"), old).unwrap();
    let before = state.serialize().unwrap();
    let origin = Instant::now();
    let restarted = begin(&engine, &state, Mode::EveryBeat, vec![]);
    assert_eq!(restarted.run_id, first.run_id);
    assert_ne!(restarted.session_id, first.session_id);
    assert_ne!(restarted.request_id, advanced.request_id);
    assert_eq!(restarted.node_path.indices, vec![0, 0]);
    assert_eq!(restarted.generation, current(&state).generation);
    let deadline = Instant::now() + Duration::from_secs(4);
    while !engine.directory.join("continuation-late-sent").exists() {
        assert!(Instant::now() < deadline);
        let _ = engine.events.recv_timeout(Duration::from_millis(10));
    }
    let session = restarted.session_id.as_deref().unwrap();
    assert_eq!(state.continuation_beat_at(&engine.manager, session, origin + Duration::from_secs(10)).phase, Phase::Waiting);
    state.continuation_beat_at(&engine.manager, first.session_id.as_deref().unwrap(), origin + Duration::from_secs(20));
    assert_eq!(state.serialize().unwrap(), before);
    std::fs::write(engine.directory.join("continuation-move"), "E5").unwrap();
    std::fs::remove_file(engine.directory.join("hold-ordinary")).unwrap();
    frame(&engine, restarted.request_id.as_deref().unwrap());
    assert!(state.continuation_beat_at(&engine.manager, session, origin + Duration::from_millis(999)).current_game.is_none());
    assert_eq!(state.serialize().unwrap(), before);
    let moved = state.continuation_beat_at(&engine.manager, session, origin + Duration::from_secs(2));
    assert_eq!(moved.current_game.as_ref().unwrap().snapshot.position.move_number, 3);
    assert_eq!(moved.node_path.indices, vec![0, 0, 0]);
    assert_ne!(moved.request_id, restarted.request_id);
    let after = state.serialize().unwrap();
    state.continuation_beat_at(&engine.manager, session, origin + Duration::from_secs(2));
    assert_eq!(state.serialize().unwrap(), after);
    state.stop_continuation(&engine.manager, "Stop completed restart proof");
}
