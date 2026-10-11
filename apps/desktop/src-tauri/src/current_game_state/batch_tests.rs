use super::*;
use app_model::{BatchCommandDto as Command, BatchOutputAuthorizationDto as Output, BatchPreferencesDto, BatchRowStateDto as RowState, BatchOutputStateDto as OutputState, BatchSnapshotDto};
use std::time::{Duration, Instant};

fn ready(engine: &LiveFixture) {
    engine.manager.set_continuous_preferences(false, Default::default()).unwrap();
    engine.manager.start("test").unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    while !matches!(engine.manager.snapshot().lifecycle, app_model::ForegroundEngineLifecycleDto::Ready { .. }) {
        assert!(Instant::now() < deadline, "{:?}", engine.manager.snapshot());
        std::thread::sleep(Duration::from_millis(5));
    }
}
fn wait_batch(engine: &LiveFixture, state: &CurrentGameState, predicate: impl Fn(&BatchSnapshotDto) -> bool) -> BatchSnapshotDto {
    let deadline = Instant::now() + Duration::from_secs(8);
    loop {
        for event in engine.events.try_iter() {
            if let app_model::ForegroundEngineEventDto::Job { job } = event {
                if engine.manager.is_batch_job(&job.job_id) { assert!(state.attach_from_job_event(&job).is_none(), "private frame must never attach to current game"); }
                else { state.attach_from_job_event(&job); }
            }
        }
        let snapshot = engine.manager.batch_snapshot();
        if predicate(&snapshot) { return snapshot; }
        assert!(Instant::now() < deadline, "batch timeout: {snapshot:?}");
        std::thread::sleep(Duration::from_millis(5));
    }
}
fn file(engine: &LiveFixture, name: &str, text: &str) -> String {
    let path = engine.directory.join(name);
    std::fs::write(&path, text).unwrap();
    path.to_string_lossy().into_owned()
}
fn start(engine: &LiveFixture, directory: bool) {
    let snapshot = engine.manager.batch_snapshot();
    let output = if directory { Output::Directory { path: engine.directory.to_string_lossy().into_owned(), confirmed_names: snapshot.rows.iter().map(|r| r.output_name.clone()).collect() } } else { Output::SessionOnly };
    engine.manager.batch_command(Command::Start { output }).unwrap();
}

#[test]
fn batch_intake_defaults_order_dedup_errors_and_pending_editing_are_session_only() {
    let engine = task_fixture();
    let one = file(&engine, "one.sgf", "(;SZ[9];B[aa])");
    let two = file(&engine, "two.sgf", "(;SZ[13];B[bb])");
    let bad = file(&engine, "bad.sgf", "not sgf");
    let gib = file(&engine, "unsupported.gib", "[GIB]");
    let duplicate = engine.directory.join("./one.sgf").to_string_lossy().into_owned();
    let defaults = BatchPreferencesDto::default();
    assert_eq!(defaults.normal.total_visits.value, 100);
    assert_eq!(defaults.overview.total_visits.value, 32);
    assert_eq!(defaults.deep.total_visits.value, 800);
    assert!(defaults.swing_deep.time_seconds.enabled);
    assert!(!defaults.swing_deep.total_visits.enabled);
    assert_eq!(defaults.swing_deep.leading_candidate_visits.value, 500);
    let snapshot = engine.manager.batch_command(Command::Intake { paths: vec![two.clone(), one.clone(), duplicate, bad, gib], conditions: defaults.clone() }).unwrap();
    assert_eq!(snapshot.rows.len(), 4);
    assert_eq!(snapshot.rows[0].source_path, two);
    assert_eq!(snapshot.rows[1].source_path, one);
    assert_eq!(snapshot.notices.len(), 1);
    assert_eq!(snapshot.rows[2].state, RowState::Failed);
    assert_eq!(snapshot.rows[3].state, RowState::Failed);
    assert!(!snapshot.started);
    engine.manager.batch_command(Command::StopCurrent).unwrap();
    assert_eq!(engine.manager.batch_snapshot().rows[0].state, RowState::Pending);
    engine.manager.batch_command(Command::Reorder { row_ids: vec![snapshot.rows[1].identity.row_id.clone(), snapshot.rows[0].identity.row_id.clone()] }).unwrap();
    assert_eq!(engine.manager.batch_snapshot().rows[0].source_path, one);
    assert!(engine.manager.batch_command(Command::Remove { identity: snapshot.rows[2].identity.clone() }).is_err());
    let mut invalid = defaults;
    invalid.swing_deep.leading_candidate_visits.value = 0;
    assert!(engine.manager.batch_command(Command::Intake { paths: vec![one], conditions: invalid }).is_err());
    engine.manager.batch_command(Command::StopAll).unwrap();
    assert_eq!(engine.manager.batch_snapshot().rows[0].state, RowState::Cancelled);
    let fresh = ForegroundEngineManager::new(Arc::new(InMemoryEngineProfileCatalog::new()), ForegroundEngineConfig::for_tests());
    assert!(fresh.batch_snapshot().rows.is_empty());
}

#[test]
fn batch_serial_file_snapshots_output_and_current_game_identity_are_independent() {
    use std::os::unix::fs::PermissionsExt;
    let engine = task_fixture();
    let state = CurrentGameState::default(); state.connect_analysis_manager(engine.manager.clone());
    state.replace("(;SZ[19]KM[7.5]C[current personal];B[dd];W[pp])", None).unwrap();
    ready(&engine);
    let one = file(&engine, "one.sgf", "(;SZ[9]RU[Japanese]KM[6.5]C[first personal];B[aa])");
    let two = file(&engine, "two.sgf", "(;SZ[13]RU[Chinese]KM[7.5];B[bb])");
    let readonly = file(&engine, "readonly.sgf", "(;SZ[9]RU[Chinese]KM[0.5]C[readonly];B[cc])");
    std::fs::set_permissions(&readonly, std::fs::Permissions::from_mode(0o400)).unwrap();
    let source_bytes = std::fs::read(&readonly).unwrap();
    engine.manager.batch_command(Command::Intake { paths: vec![one.clone(), two.clone(), readonly.clone()], conditions: Default::default() }).unwrap();
    start(&engine, true);
    let live = wait_batch(&engine, &state, |s| s.rows[0].has_result);
    assert!(engine.manager.batch_occupied());
    assert!(engine.manager.analysis_task_snapshot().is_none());
    assert!(engine.manager.batch_command(Command::Remove { identity: live.rows[0].identity.clone() }).is_err());
    assert!(engine.manager.reserve_match("must-not-steal").is_err());
    std::fs::write(&one, "external edit after snapshot").unwrap();
    std::fs::write(&two, "(;SZ[5]RU[Chinese]KM[8.5]C[changed before turn];B[aa])").unwrap();
    let replacement = state.prepare_replacement("(;SZ[19]KM[5.5]C[new current];B[qq];W[dd])", None).unwrap();
    let departure_id = match replacement {
        app_model::DocumentDepartureAdmissionDto::NeedsDecision { departure_id }
        | app_model::DocumentDepartureAdmissionDto::Ready { departure_id } => departure_id,
    };
    crate::document_departure::confirm_departure(&state, departure_id, &[],
        |_| panic!("private batch job must not enter current-document departure"),
        |_, _| panic!("private batch job must not be drained by current-document replacement"),
        Duration::from_secs(2)).unwrap();
    assert!(state.commit_replacement(departure_id).unwrap().committed);
    assert!(engine.manager.batch_occupied());
    state.select_path(NodePath::default(), state.inspect().0).unwrap();
    let before = state.inspect(); let bytes = state.serialize().unwrap();
    file(&engine, "finish", "");
    let done = wait_batch(&engine, &state, |s| s.rows.iter().all(|r| r.output_state == OutputState::Saved));
    assert_eq!(state.inspect(), before); assert_eq!(state.serialize().unwrap(), bytes);
    assert_eq!(std::fs::read(&readonly).unwrap(), source_bytes);
    let first = sgf::CurrentSgfDocument::open(&std::fs::read_to_string(done.rows[0].saved_path.as_ref().unwrap()).unwrap()).unwrap();
    let second = sgf::CurrentSgfDocument::open(&std::fs::read_to_string(done.rows[1].saved_path.as_ref().unwrap()).unwrap()).unwrap();
    assert_eq!(first.board_width(), 9); assert_eq!(first.rules(), "japanese"); assert_eq!(first.komi(), 6.5);
    assert_eq!(second.board_width(), 5); assert_eq!(second.komi(), 8.5);
    assert_eq!(first.snapshot(&NodePath::default()).unwrap().personal_comment, "first personal");
    assert!(first.snapshot(&NodePath::default()).unwrap().primary_analysis.is_some());
    assert!(done.rows[1].input_receipt.as_ref().unwrap().contains("changed since intake"));
    assert_ne!(done.rows[0].document_id, done.rows[1].document_id);
    assert_ne!(done.rows[0].task.as_ref().unwrap().job_id, done.rows[1].task.as_ref().unwrap().job_id);
    assert!(engine.manager.shutdown_batch(Duration::from_secs(2)));
}

#[test]
fn batch_output_conflict_retains_computation_and_retry_save_never_recomputes_or_overwrites_source() {
    let engine = task_fixture(); file(&engine, "finish", "");
    let state = CurrentGameState::default(); state.connect_analysis_manager(engine.manager.clone());
    state.replace("(;SZ[19]C[current])", None).unwrap(); ready(&engine);
    let one = file(&engine, "one.sgf", "(;SZ[9]C[source];B[aa])");
    let two = file(&engine, "two.sgf", "(;SZ[13]C[pending source];B[bb])");
    let before = std::fs::read(&one).unwrap();
    let before_two = std::fs::read(&two).unwrap();
    let preview = engine.manager.batch_command(Command::Intake { paths: vec![one.clone(), two.clone()], conditions: Default::default() }).unwrap();
    let collision = engine.directory.join(&preview.rows[0].output_name);
    std::fs::write(&collision, "existing output").unwrap();
    start(&engine, true);
    let failed = wait_batch(&engine, &state, |s| s.paused && s.rows[0].output_state == OutputState::Failed);
    assert!(failed.rows[0].calculation_complete); assert!(failed.rows[0].has_result);
    assert_eq!(failed.rows[1].state, RowState::Pending);
    assert_eq!(std::fs::read_to_string(&collision).unwrap(), "existing output");
    let queries = std::fs::read(engine.directory.join("queries.jsonl")).unwrap();
    let alias = engine.directory.join("source-alias.sgf");
    std::os::unix::fs::symlink(&two, &alias).unwrap();
    let directory_alias = engine.directory.join("directory-alias");
    std::os::unix::fs::symlink(&engine.directory, &directory_alias).unwrap();
    for target in [one.clone(), two.clone(), alias.to_string_lossy().into_owned(), directory_alias.join("one.sgf").to_string_lossy().into_owned(), directory_alias.join("two.sgf").to_string_lossy().into_owned()] {
        assert!(engine.manager.batch_command(Command::Save { identity: failed.rows[0].identity.clone(), path: target.clone(), overwrite: true, incomplete: false }).is_err(), "queued source accepted: {target}");
        assert_eq!(std::fs::read(&one).unwrap(), before);
        assert_eq!(std::fs::read(&two).unwrap(), before_two);
        let rejected = engine.manager.batch_snapshot();
        assert_eq!(rejected.rows[0].state, RowState::Completed);
        assert_eq!(rejected.rows[0].output_state, OutputState::Failed);
        assert_eq!(rejected.rows[0].task, failed.rows[0].task);
        assert_eq!(rejected.rows[1], failed.rows[1]);
    }
    let saved = engine.manager.batch_command(Command::Save { identity: failed.rows[0].identity.clone(), path: collision.to_string_lossy().into_owned(), overwrite: true, incomplete: false }).unwrap();
    assert_eq!(saved.rows[0].output_state, OutputState::Saved);
    assert_eq!(std::fs::read(engine.directory.join("queries.jsonl")).unwrap(), queries);
    assert_eq!(std::fs::read(one).unwrap(), before);
    let output = sgf::CurrentSgfDocument::open(&std::fs::read_to_string(&collision).unwrap()).unwrap();
    assert_eq!(output.board_width(), 9);
    assert!(output.snapshot(&NodePath::default()).unwrap().primary_analysis.is_some());
    let mut stale = saved.rows[0].identity.clone(); stale.attempt += 1;
    assert!(engine.manager.batch_command(Command::Save { identity: stale, path: engine.directory.join("late.sgf").to_string_lossy().into_owned(), overwrite: false, incomplete: false }).is_err());
    assert!(!engine.directory.join("late.sgf").exists());
    engine.manager.batch_command(Command::Continue).unwrap();
    let completed = wait_batch(&engine, &state, |s| s.rows[1].output_state == OutputState::Saved);
    let pending_output = sgf::CurrentSgfDocument::open(&std::fs::read_to_string(completed.rows[1].saved_path.as_ref().unwrap()).unwrap()).unwrap();
    assert_eq!(pending_output.board_width(), 13);
    assert_eq!(pending_output.snapshot(&NodePath::default()).unwrap().personal_comment, "pending source");
    assert!(pending_output.snapshot(&NodePath::default()).unwrap().primary_analysis.is_some());
    assert_eq!(std::fs::read(&two).unwrap(), before_two);
    engine.manager.batch_command(Command::StopAll).unwrap();
    assert!(collision.exists());
    assert!(engine.manager.shutdown_batch(Duration::from_secs(2)));
}

#[test]
fn batch_stop_current_drains_before_next_and_partial_save_has_separate_generated_marker() {
    let engine = task_fixture();
    let state = CurrentGameState::default(); state.connect_analysis_manager(engine.manager.clone()); state.replace("(;SZ[19])", None).unwrap(); ready(&engine);
    let one = file(&engine, "one.sgf", "(;SZ[9]C[personal survives];B[aa];W[bb])");
    let two = file(&engine, "two.sgf", "(;SZ[9];B[cc])");
    engine.manager.batch_command(Command::Intake { paths: vec![one, two], conditions: Default::default() }).unwrap(); start(&engine, false);
    let live = wait_batch(&engine, &state, |s| s.rows[0].has_result);
    engine.manager.batch_command(Command::StopCurrent).unwrap();
    assert_eq!(engine.manager.batch_snapshot().rows[1].state, RowState::Pending);
    assert!(engine.manager.batch_occupied(), "control ACK is not drained final");
    file(&engine, "cancel-final", ""); file(&engine, "finish", "");
    let done = wait_batch(&engine, &state, |s| s.rows[1].calculation_complete);
    assert_eq!(done.rows[0].state, RowState::Cancelled);
    let output = engine.directory.join("partial.sgf").to_string_lossy().into_owned();
    assert!(engine.manager.batch_command(Command::Save { identity: live.rows[0].identity.clone(), path: output.clone(), overwrite: false, incomplete: false }).is_err());
    engine.manager.batch_command(Command::Save { identity: live.rows[0].identity.clone(), path: output.clone(), overwrite: false, incomplete: true }).unwrap();
    let text = std::fs::read_to_string(output).unwrap();
    assert!(text.contains("LZBATCH[incomplete;")); assert!(text.contains("C[personal survives]"));
    assert!(matches!(engine.manager.snapshot().lifecycle, app_model::ForegroundEngineLifecycleDto::Ready { .. }));
}

#[test]
fn batch_independent_preferences_are_atomic_and_preserved_by_stale_current_game_saves() {
    let engine = task_fixture();
    let state = crate::continuous_analysis::PreferencesState::default();
    let path = engine.directory.join("prefs.json");
    let original = state.load(&path, &engine.manager).unwrap().preferences;
    let mut batch = BatchPreferencesDto::default(); batch.normal.total_visits.value = 17;
    state.save_batch(&path, batch.clone()).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    let mut invalid = batch.clone(); invalid.deep.total_visits.value = 499;
    assert!(state.save_batch(&path, invalid).is_err());
    assert!(state.save_batch(&engine.directory, Default::default()).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    state.save(&path, &engine.manager, original.clone()).unwrap();
    let restarted = app_preferences::load_from_path(&path).unwrap().preferences;
    assert_eq!(restarted.batch, batch);
    assert_eq!(restarted.task_single_stage_conditions, original.task_single_stage_conditions);
    assert!(engine.manager.batch_snapshot().rows.is_empty());
}

#[test]
fn batch_swing_scope_uses_private_supporting_positions_and_reuses_strategy_scheduler() {
    let engine = task_fixture(); file(&engine, "budget-smoke", "swing");
    let state = CurrentGameState::default(); state.connect_analysis_manager(engine.manager.clone());
    state.replace("(;SZ[19];B[dd];W[pp])", None).unwrap(); ready(&engine);
    let path = file(&engine, "swing.sgf", "(;SZ[9];B[aa];W[bb];B[cc])");
    let mut conditions = BatchPreferencesDto::default();
    conditions.strategy = app_model::AnalysisTaskStrategyDto::SwingSelectedTwoStage;
    conditions.scope.interval = Some(app_model::AnalysisPositionIntervalDto { start: 1, end: 3 });
    conditions.swing_deep.time_seconds.enabled = false; conditions.swing_deep.total_visits.enabled = true;
    let preview = engine.manager.batch_command(Command::Intake { paths: vec![path], conditions: conditions.clone() }).unwrap();
    assert!(!preview.rows[0].preview.as_ref().unwrap().supporting_targets.is_empty());
    let current = state.serialize().unwrap(); start(&engine, false);
    let result = wait_batch(&engine, &state, |s| s.rows[0].calculation_complete);
    let task = result.rows[0].task.as_ref().unwrap();
    assert_eq!(task.strategy, conditions.strategy); assert!(!task.overview_completed.is_empty());
    assert!(task.selected_for_deep.is_some()); assert_eq!(state.serialize().unwrap(), current);
    let queries = std::fs::read_to_string(engine.directory.join("queries.jsonl")).unwrap();
    assert!(queries.contains("32"));
}

#[test]
fn batch_failed_file_rerun_is_a_new_attempt_and_late_save_is_rejected() {
    let engine = task_fixture(); file(&engine, "finish", "");
    let state = CurrentGameState::default(); state.connect_analysis_manager(engine.manager.clone()); state.replace("(;SZ[19])", None).unwrap(); ready(&engine);
    let path = file(&engine, "broken.sgf", "invalid");
    let preview = engine.manager.batch_command(Command::Intake { paths: vec![path.clone()], conditions: Default::default() }).unwrap();
    let old = preview.rows[0].identity.clone(); assert_eq!(preview.rows[0].state, RowState::Failed);
    std::fs::write(&path, "(;SZ[9];B[aa])").unwrap();
    let retry = engine.manager.batch_command(Command::Rerun { identity: old.clone() }).unwrap();
    assert_eq!(retry.rows[0].identity.attempt, old.attempt + 1);
    start(&engine, false); wait_batch(&engine, &state, |s| s.rows[0].calculation_complete);
    assert!(engine.manager.batch_command(Command::Save { identity: old, path: engine.directory.join("late.sgf").to_string_lossy().into(), overwrite: false, incomplete: false }).is_err());
    assert!(!engine.directory.join("late.sgf").exists());
}

#[test]
fn batch_waits_for_explicit_owner_and_stop_all_cancels_waiting_without_stealing_run() {
    let engine = task_fixture();
    file(&engine, "hold-first", "");
    let state = CurrentGameState::default();
    state.connect_analysis_manager(engine.manager.clone());
    let opened = state.replace("(;SZ[9];B[aa])", None).unwrap();
    ready(&engine);
    let run_id = match engine.manager.snapshot().lifecycle {
        app_model::ForegroundEngineLifecycleDto::Ready { run } => run.run_id,
        other => panic!("{other:?}"),
    };
    let explicit = state.start_first_child_analysis(&engine.manager, run_id.clone(), opened.generation, 32).unwrap();
    let path = file(&engine, "waiting.sgf", "(;SZ[13];B[bb])");
    engine.manager.batch_command(Command::Intake { paths: vec![path], conditions: Default::default() }).unwrap();
    start(&engine, false);
    let waiting = wait_batch(&engine, &state, |s| s.rows[0].state == RowState::Waiting);
    assert!(waiting.rows[0].document_id.is_none());
    assert!(engine.manager.batch_occupied(), "an armed batch reserves dispatch even while the prior explicit owner drains");
    assert_eq!(engine.manager.analysis_task_snapshot().unwrap().job_id, explicit.job_id);
    engine.manager.batch_command(Command::StopCurrent).unwrap();
    assert_eq!(engine.manager.batch_snapshot().rows[0].state, RowState::Waiting);
    engine.manager.batch_command(Command::StopAll).unwrap();
    assert_eq!(engine.manager.batch_snapshot().rows[0].state, RowState::Cancelled);
    assert!(!engine.manager.batch_occupied(), "Stop all releases the queued batch reservation, not the prior owner's job");
    assert_eq!(engine.manager.analysis_task_snapshot().unwrap().job_id, explicit.job_id);
    file(&engine, "finish", "");
    wait_live_task(&engine, app_model::AnalysisTaskStateDto::Completed);
    assert!(matches!(engine.manager.snapshot().lifecycle, app_model::ForegroundEngineLifecycleDto::Ready { run } if run.run_id == run_id));
    assert!(engine.manager.shutdown_batch(Duration::from_secs(2)));
}

#[test]
fn batch_run_and_authority_changes_require_explicit_continue_even_after_ready() {
    for change_run in [false, true] {
        let engine = task_fixture();
        let state = CurrentGameState::default();
        state.connect_analysis_manager(engine.manager.clone());
        state.replace("(;SZ[19])", None).unwrap(); ready(&engine);
        let one = file(&engine, "one.sgf", "(;SZ[9];B[aa];W[bb])");
        let two = file(&engine, "two.sgf", "(;SZ[13];B[cc])");
        engine.manager.batch_command(Command::Intake { paths: vec![one, two], conditions: Default::default() }).unwrap();
        start(&engine, false);
        wait_batch(&engine, &state, |s| s.rows[0].has_result);
        file(&engine, "cancel-final", "");
        if change_run {
            engine.manager.restart().unwrap();
        } else {
            engine.manager.retire_batch_for_authority().unwrap();
            assert!(engine.manager.batch_command(Command::Continue).is_err());
            engine.manager.set_batch_review_access(true);
        }
        let retired = wait_batch(&engine, &state, |s| s.paused && s.rows[0].state == RowState::Cancelled && !engine.manager.batch_occupied());
        assert_eq!(retired.rows[1].state, RowState::Pending);
        let deadline = Instant::now() + Duration::from_secs(5);
        while !matches!(engine.manager.snapshot().lifecycle, app_model::ForegroundEngineLifecycleDto::Ready { .. }) {
            assert!(Instant::now() < deadline); std::thread::sleep(Duration::from_millis(5));
        }
        assert!(engine.manager.batch_snapshot().paused);
        assert!(engine.manager.batch_snapshot().rows[1].document_id.is_none());
        file(&engine, "finish", "");
        engine.manager.batch_command(Command::Continue).unwrap();
        let done = wait_batch(&engine, &state, |s| s.rows[1].calculation_complete);
        assert_eq!(done.rows[0].state, RowState::Cancelled);
        assert!(engine.manager.shutdown_batch(Duration::from_secs(2)));
    }
}

#[test]
fn batch_engine_and_protocol_failures_stop_dispatch_and_preserve_pending() {
    for failure in ["engine_error", "unmarked", "ignored_setting"] {
        let engine = task_fixture();
        file(&engine, "budget-smoke", failure);
        file(&engine, "cancel-final", "");
        let state = CurrentGameState::default();
        state.connect_analysis_manager(engine.manager.clone());
        state.replace("(;SZ[19]C[unchanged])", None).unwrap(); ready(&engine);
        let before = state.serialize().unwrap();
        let one = file(&engine, "one.sgf", "(;SZ[9];B[aa])");
        let two = file(&engine, "two.sgf", "(;SZ[13];B[bb])");
        engine.manager.batch_command(Command::Intake { paths: vec![one, two], conditions: Default::default() }).unwrap();
        start(&engine, true);
        let failed = wait_batch(&engine, &state, |s| s.paused && s.rows[0].state == RowState::Failed);
        assert!(failed.reason.is_some());
        assert_eq!(failed.rows[1].state, RowState::Pending);
        assert!(failed.rows[1].document_id.is_none());
        assert!(!engine.directory.join(&failed.rows[0].output_name).exists());
        assert_eq!(state.serialize().unwrap(), before);
        assert!(engine.manager.shutdown_batch(Duration::from_secs(2)));
    }
}

#[test]
fn batch_stop_all_hands_back_only_the_current_unpaused_target() {
    for paused in [false, true] {
        let engine = task_fixture(); file(&engine, "cancel-final", "");
        let state = CurrentGameState::default(); state.connect_analysis_manager(engine.manager.clone());
        let opened = state.replace("(;SZ[19];B[dd];W[pp])", None).unwrap(); ready(&engine);
        let one = file(&engine, "one.sgf", "(;SZ[9];B[aa];W[bb])");
        engine.manager.batch_command(Command::Intake { paths: vec![one], conditions: Default::default() }).unwrap(); start(&engine, false);
        wait_batch(&engine, &state, |s| s.rows[0].has_result);
        engine.manager.set_continuous_preferences(true, Default::default()).unwrap();
        let selected = NodePath { indices: vec![0] };
        state.select_path(selected.clone(), opened.generation).unwrap();
        if paused { engine.manager.set_continuous_preferences(false, Default::default()).unwrap(); }
        engine.manager.batch_command(Command::StopAll).unwrap();
        wait_batch(&engine, &state, |_| !engine.manager.batch_occupied());
        if paused {
            assert!(engine.manager.snapshot().selected_node_job.is_none());
        } else {
            wait_batch(&engine, &state, |_| engine.manager.snapshot().selected_node_job.is_some());
            assert_eq!(engine.manager.snapshot().selected_node_job.unwrap().node_path, selected);
        }
        assert!(engine.manager.shutdown_batch(Duration::from_secs(2)));
    }
}

#[test]
fn batch_failed_current_target_handback_keeps_pending_and_requires_resolution() {
    let engine = task_fixture(); file(&engine, "cancel-final", "");
    let state = CurrentGameState::default(); state.connect_analysis_manager(engine.manager.clone());
    state.replace("(;SZ[19];B[dd])", None).unwrap(); ready(&engine);
    let one = file(&engine, "one.sgf", "(;SZ[9];B[aa])");
    let two = file(&engine, "two.sgf", "(;SZ[13];B[bb])");
    engine.manager.batch_command(Command::Intake { paths: vec![one, two], conditions: Default::default() }).unwrap(); start(&engine, false);
    wait_batch(&engine, &state, |s| s.rows[0].has_result);
    engine.manager.batch_command(Command::Pause).unwrap();
    engine.manager.set_continuous_preferences(true, Default::default()).unwrap();
    file(&engine, "fail-handback", ""); file(&engine, "finish", "");
    let failed = wait_batch(&engine, &state, |_| matches!(engine.manager.snapshot().lifecycle, app_model::ForegroundEngineLifecycleDto::Error { .. }));
    assert!(failed.paused);
    assert_eq!(failed.rows[1].state, RowState::Pending);
    assert!(failed.rows[1].document_id.is_none());
    assert!(engine.manager.batch_command(Command::Continue).is_err());
    assert!(engine.manager.shutdown_batch(Duration::from_secs(2)));
}

#[test]
fn batch_three_budget_or_conditions_end_actual_private_requests() {
    for condition in ["time_seconds", "total_visits", "leading_candidate_visits"] {
        let engine = task_fixture(); file(&engine, "budget-smoke", condition); file(&engine, "cancel-final", "");
        let state = CurrentGameState::default(); state.connect_analysis_manager(engine.manager.clone());
        state.replace("(;SZ[19];B[dd])", None).unwrap(); ready(&engine);
        let source = file(&engine, "budget.sgf", "(;SZ[9]RU[Japanese]KM[6.5])");
        let mut conditions = BatchPreferencesDto::default();
        conditions.normal.time_seconds.enabled = true;
        conditions.normal.time_seconds.value = if condition == "time_seconds" { 1 } else { 30 };
        conditions.normal.total_visits.value = if condition == "total_visits" { 8 } else { 100 };
        conditions.normal.leading_candidate_visits.enabled = true;
        conditions.normal.leading_candidate_visits.value = if condition == "leading_candidate_visits" { 5 } else { 100 };
        engine.manager.batch_command(Command::Intake { paths: vec![source], conditions }).unwrap(); start(&engine, false);
        let done = wait_batch(&engine, &state, |s| s.rows[0].calculation_complete);
        assert_eq!(done.rows[0].task.as_ref().unwrap().ending_conditions, vec![condition]);
        let queries: Vec<serde_json::Value> = std::fs::read_to_string(engine.directory.join("queries.jsonl")).unwrap().lines().map(|line| serde_json::from_str(line).unwrap()).collect();
        assert_eq!(queries.len(), 1);
        assert_eq!(queries[0]["boardXSize"], 9);
        assert_eq!(queries[0]["rules"], "japanese");
        assert_eq!(queries[0]["komi"], 6.5);
        assert_eq!(queries[0]["overrideSettings"]["maxTime"].as_f64(), Some(if condition == "time_seconds" { 1.0 } else { 30.0 }));
        assert_eq!(queries[0]["overrideSettings"]["maxVisits"], if condition == "total_visits" { 8 } else { 100 });
        assert!(engine.manager.shutdown_batch(Duration::from_secs(2)));
    }
}

#[test]
#[ignore = "requires explicitly supplied read-only CPU KataGo and model"]
fn real_cpu_batch_multiple_documents_and_two_stage_outputs() {
    let binary = std::env::var("R13_KATAGO").expect("R13_KATAGO");
    let model = std::env::var("R13_MODEL").expect("R13_MODEL");
    let directory = std::path::PathBuf::from(std::env::var("R13_EVIDENCE").expect("R13_EVIDENCE"));
    std::fs::create_dir_all(&directory).unwrap();
    let config = directory.join("cpu.cfg");
    std::fs::write(&config, "numAnalysisThreads = 1\nnumSearchThreads = 1\nnnCacheSizePowerOfTwo = 16\nnnMutexPoolSizePowerOfTwo = 12\nmaxVisits = 8\nlogToStderr = true\nlogSearchInfo = false\n").unwrap();
    let catalog = Arc::new(InMemoryEngineProfileCatalog::new());
    catalog.upsert(engine_manager::SavedEngineProfile { profile_id: "test".into(), thread_policy: Default::default(), profile: app_model::EngineProfileDto { name: "R13 batch isolated CPU".into(), program: binary, argv: vec![], working_dir: Some(directory.to_string_lossy().into()), adapter: app_model::EngineAdapterSettings::KataGoAnalysis(app_model::KataGoSettings { model_path: Some(model), config_path: Some(config.to_string_lossy().into()), max_visits: 8 }) } });
    let manager = ForegroundEngineManager::new(catalog, ForegroundEngineConfig { readiness_timeout: Duration::from_secs(120), job_timeout: Duration::from_secs(120), ..Default::default() });
    let events = manager.subscribe();
    let state = CurrentGameState::default(); state.connect_analysis_manager(manager.clone());
    state.replace("(;SZ[19]C[current retained];B[dd])", None).unwrap();
    manager.set_continuous_preferences(false, Default::default()).unwrap();
    manager.start("test").unwrap();
    let deadline = Instant::now() + Duration::from_secs(150);
    while !matches!(manager.snapshot().lifecycle, app_model::ForegroundEngineLifecycleDto::Ready { .. }) {
        assert!(Instant::now() < deadline, "{:?}", manager.snapshot()); std::thread::sleep(Duration::from_millis(20));
    }
    let before = state.serialize().unwrap(); let identity = state.inspect();
    for (name, source, two_stage) in [("nine", "(;SZ[9]RU[Japanese]KM[6.5]C[nine])", false), ("thirteen", "(;SZ[13]RU[Chinese]KM[7.5]C[thirteen])", true)] {
        let path = directory.join(format!("{name}.sgf")); std::fs::write(&path, source).unwrap();
        let mut conditions = BatchPreferencesDto::default();
        conditions.normal.total_visits.value = 8;
        if two_stage { conditions.strategy = app_model::AnalysisTaskStrategyDto::AllPositionsTwoStage; conditions.deep.total_visits.value = 500; }
        manager.batch_command(Command::Intake { paths: vec![path.to_string_lossy().into()], conditions }).unwrap();
    }
    let preview = manager.batch_snapshot();
    manager.batch_command(Command::Start { output: Output::Directory { path: directory.to_string_lossy().into(), confirmed_names: preview.rows.iter().map(|r| r.output_name.clone()).collect() } }).unwrap();
    let deadline = Instant::now() + Duration::from_secs(240);
    loop {
        for event in events.try_iter() { if let app_model::ForegroundEngineEventDto::Job { job } = event { assert!(state.attach_from_job_event(&job).is_none()); } }
        let batch = manager.batch_snapshot();
        assert!(!batch.paused, "{batch:?}");
        if batch.rows.iter().all(|r| r.output_state == OutputState::Saved) {
            for row in &batch.rows {
                let text = std::fs::read_to_string(row.saved_path.as_ref().unwrap()).unwrap();
                let document = CurrentSgfDocument::open(&text).unwrap();
                assert!(document.snapshot(&NodePath::default()).unwrap().primary_analysis.is_some());
                assert!(text.contains("LZBATCH[complete;"));
            }
            assert_eq!(state.serialize().unwrap(), before); assert_eq!(state.inspect(), identity);
            std::fs::write(directory.join("batch.json"), serde_json::to_vec_pretty(&batch).unwrap()).unwrap(); break;
        }
        assert!(Instant::now() < deadline, "{batch:?}"); std::thread::sleep(Duration::from_millis(20));
    }
    let wait = |predicate: &dyn Fn() -> bool| {
        let deadline = Instant::now() + Duration::from_secs(120);
        while !predicate() {
            assert!(Instant::now() < deadline, "{:?}; {:?}", manager.batch_snapshot(), manager.snapshot());
            std::thread::sleep(Duration::from_millis(20));
        }
    };
    wait(&|| !manager.batch_occupied());
    // New session authorization; earlier directory permission must not carry over.
    loop {
        match manager.batch_command(Command::NewBatch) {
            Ok(_) => break,
            Err(_) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(20)),
            Err(error) => panic!("{error}"),
        }
    }
    let mut long = BatchPreferencesDto::default();
    long.normal.total_visits.value = 1_000_000;
    manager.batch_command(Command::Intake { paths: vec![directory.join("nine.sgf").to_string_lossy().into(), directory.join("thirteen.sgf").to_string_lossy().into()], conditions: long }).unwrap();
    manager.batch_command(Command::Start { output: Output::SessionOnly }).unwrap();
    wait(&|| manager.batch_snapshot().rows[0].task.is_some());
    manager.batch_command(Command::StopCurrent).unwrap();
    wait(&|| manager.batch_snapshot().rows[1].task.is_some());
    assert_eq!(manager.batch_snapshot().rows[0].state, RowState::Cancelled);
    manager.set_continuous_preferences(true, Default::default()).unwrap();
    let current_path = NodePath { indices: vec![0] };
    state.select_path(current_path.clone(), state.inspect().0).unwrap();
    manager.batch_command(Command::StopAll).unwrap();
    wait(&|| !manager.batch_occupied() && manager.snapshot().selected_node_job.is_some());
    let handback = manager.snapshot().selected_node_job.unwrap();
    assert_eq!(handback.node_path, current_path);
    assert!(!manager.is_batch_job(&handback.job_id));
    wait(&|| events.try_iter().any(|event| matches!(event, app_model::ForegroundEngineEventDto::Job { job } if job.job_id == handback.job_id && job.frame.is_some())));
    assert!(manager.batch_snapshot().rows.iter().all(|r| r.state == RowState::Cancelled && r.output_state == OutputState::Unsaved));
    assert_eq!(state.serialize().unwrap(), before);
    std::fs::write(directory.join("stop-handback.json"), serde_json::to_vec_pretty(&manager.snapshot()).unwrap()).unwrap();
    std::fs::write(directory.join("stopped-batch.json"), serde_json::to_vec_pretty(&manager.batch_snapshot()).unwrap()).unwrap();
    assert!(manager.shutdown_batch(Duration::from_secs(5))); manager.teardown().unwrap();
    std::fs::write(directory.join("result.txt"), "PASS real CPU serial 9/Japanese and 13/Chinese; normal8 and overview32/deep500; independent output parseback and unchanged current document; Stop current drains then next; Stop all retains outputs; real current-target ordinary response after handback\n").unwrap();
}
