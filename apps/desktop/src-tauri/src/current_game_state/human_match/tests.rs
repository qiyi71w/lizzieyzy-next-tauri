use super::*;
use engine_manager::{ForegroundEngineConfig, InMemoryEngineProfileCatalog, SavedEngineProfile};
use std::{os::unix::fs::PermissionsExt, path::PathBuf, sync::Arc, time::{Duration, Instant}};

mod analysis;

struct Rig {
    state: CurrentGameState,
    manager: ForegroundEngineManager,
    preferences: crate::continuous_analysis::PreferencesState,
    profile: EngineProfileDto,
    catalog: Arc<InMemoryEngineProfileCatalog>,
    directory: PathBuf,
    path: PathBuf,
}
impl Rig {
    fn new(mode: &str) -> Self {
        Self::with_adapter(mode, true)
    }
    fn with_adapter(mode: &str, kata: bool) -> Self {
        Self::with_config(mode, kata, ForegroundEngineConfig::for_tests())
    }
    fn with_config(mode: &str, kata: bool, config: ForegroundEngineConfig) -> Self {
        let directory = std::env::temp_dir().join(format!("human-match-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(directory.join("mode"), mode).unwrap();
        std::fs::write(directory.join("model"), "").unwrap();
        std::fs::write(directory.join("config"), "").unwrap();
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../crates/engine-manager/tests/fixtures/game_move_process.py");
        let engine = directory.join("engine");
        std::fs::write(&engine, format!("#!/bin/sh\nexec /usr/bin/python3 '{}' \"$@\"\n", fixture.display())).unwrap();
        std::fs::set_permissions(&engine, std::fs::Permissions::from_mode(0o755)).unwrap();
        let profile = EngineProfileDto { name: "controlled engine".into(),
            program: engine.to_string_lossy().into(),
            argv: if kata { vec![] } else { ["--mode", "gtp", "--chinese-rules", "--positional-superko", "--forbid-suicide", "--level", "1", "--seed", "1"].map(String::from).into() },
            working_dir: Some(directory.to_string_lossy().into()),
            adapter: if kata { EngineAdapterSettings::KataGoAnalysis(KataGoSettings {
                model_path: Some("model".into()), config_path: Some("config".into()), max_visits: 8 }) }
                else { EngineAdapterSettings::GenericGtp(GenericGtpSettings {}) } };
        let catalog = Arc::new(InMemoryEngineProfileCatalog::new());
        catalog.upsert(SavedEngineProfile { profile_id: "engine".into(), profile: profile.clone() });
        let manager = ForegroundEngineManager::new(catalog.clone(), config);
        let state = CurrentGameState::default();
        state.connect_analysis_manager(manager.clone());
        state.replace("(;SZ[5]RU[Chinese-KGS]KM[6.5]C[old]XX[opaque](;B[aa]C[branch])(;B[bb]))",
            Some(directory.join("old.sgf").to_string_lossy().into())).unwrap();
        state.select_path(NodePath { indices: vec![1] }, state.inspect().0).unwrap();
        let preferences = crate::continuous_analysis::PreferencesState::default();
        let path = directory.join("preferences.json");
        preferences.load(&path, &manager).unwrap();
        Self { state, manager, preferences, profile, catalog, directory, path }
    }
    fn request(&self, color: PlayerColor) -> HumanMatchStartDto {
        let current = self.state.human_match_snapshot().current.unwrap();
        HumanMatchStartDto { settings: MatchDefaultsDto { board_size: 5, human_color: color,
            rules: Some(ExactRulesDto::ChineseKgs), profile_id: Some("engine".into()),
            deadline_ms: if self.profile.adapter_kind() == EngineBackend::GenericGtp { 3000 } else { 1000 },
            kata_max_visits: 8, ..MatchDefaultsDto::default() },
            generation: current.generation, snapshot_seq: current.snapshot_seq,
            start: MatchStartDto::New { discard_confirmed: true } }
    }
    fn open(&self, sgf: &str, selected: Vec<u32>) {
        self.state.replace(sgf, Some(self.directory.join("old.sgf").to_string_lossy().into())).unwrap();
        self.state.select_path(NodePath { indices: selected }, self.state.inspect().0).unwrap();
    }
    fn continue_request(&self, color: PlayerColor, confirmed: bool) -> HumanMatchStartDto {
        let mut request = self.request(color);
        let node_path = self.state.human_match_snapshot().current.unwrap().selected_path;
        request.start = MatchStartDto::Continue { node_path, root_metadata_confirmed: confirmed };
        request
    }
    fn try_start(&self, request: HumanMatchStartDto) -> Result<MatchUpdateDto, EngineFailureDto> {
        self.state.start_human_match(&self.manager, &self.preferences, &self.path, request, self.profile.clone(), |_| {})
    }
    fn start(&self, color: PlayerColor) -> MatchUpdateDto {
        self.state.start_human_match(&self.manager, &self.preferences, &self.path, self.request(color), self.profile.clone(), |_| {}).unwrap()
    }
    fn token(&self) -> MatchTurnDto {
        let update = self.state.human_match_snapshot();
        let current = update.current.unwrap();
        MatchTurnDto { session_id: update.match_state.session_id.unwrap(), turn: update.match_state.turn,
            generation: current.generation, node_path: current.selected_path }
    }
    fn human(&self, vertex: MoveVertex) -> MatchUpdateDto {
        self.state.human_match_action(&self.manager, self.token(), HumanMatchActionDto::Play { vertex }).unwrap()
    }
    fn engine(&self) -> MatchUpdateDto {
        let (token, handle) = self.state.take_match_engine_turn(&self.manager).unwrap().unwrap();
        self.state.accept_match_engine_result(&self.manager, &token, handle.wait())
    }
    fn stop(&self) -> MatchUpdateDto {
        self.state.stop_human_match(&self.manager, &self.token().session_id).unwrap()
    }
    fn foreground(&self) -> String {
        self.manager.start("engine").unwrap();
        let until = Instant::now() + Duration::from_secs(4);
        loop {
            match self.manager.snapshot().lifecycle {
                ForegroundEngineLifecycleDto::Ready { run } => return run.run_id,
                ForegroundEngineLifecycleDto::Error { failure, .. } => panic!("{failure:?}"),
                _ => assert!(Instant::now() < until),
            }
            std::thread::yield_now();
        }
    }
}
impl Drop for Rig {
    fn drop(&mut self) { self.manager.teardown().unwrap(); let _ = std::fs::remove_dir_all(&self.directory); }
}
fn point(x: u8, y: u8) -> MoveVertex { MoveVertex::Point(PointDto { x, y }) }

#[test]
fn analysis_is_session_only_off_and_verified_capability_gated() {
    for (kata, supported) in [(true, true), (false, false)] {
        let rig = Rig::with_adapter("normal", kata);
        let update = rig.start(PlayerColor::Black);
        assert_eq!(update.match_state.analysis.supported, supported);
        assert_eq!(update.match_state.analysis.policy, MatchAnalysisPolicyDto::Off);
        assert!(update.match_state.analysis.frame.is_none());
        rig.stop();
    }
}

#[test]
fn human_black_commits_once_and_rejects_all_ordinary_document_admissions() {
    let rig = Rig::new("normal");
    let update = rig.start(PlayerColor::Black);
    assert!(update.match_state.committed);
    let token = rig.token();
    assert!(rig.state.select_path(NodePath::default(), token.generation).is_err());
    assert!(rig.state.play(NodePath::default(), point(0, 0)).is_err());
    assert!(rig.state.set_personal_comment(NodePath::default(), "forbidden".into()).is_err());
    assert!(rig.state.admit_selected_node(token.generation, &token.node_path).is_err());
    assert!(rig.state.enter_trial().is_err());
    assert!(rig.state.human_match_action(&rig.manager, token.clone(), HumanMatchActionDto::Play { vertex: point(8, 8) }).is_err());
    assert_eq!(rig.token(), token);
    rig.human(point(0, 0));
    assert!(rig.state.human_match_action(&rig.manager, token, HumanMatchActionDto::Play { vertex: point(1, 1) }).is_err());
    let update = rig.engine();
    assert_eq!(update.match_state.turn, 3);
    assert_eq!(update.match_state.to_play, Some(PlayerColor::Black));
    let serialized = rig.state.serialize().unwrap();
    assert!(serialized.contains(";B[aa];W[db]"), "{serialized}");
    assert_eq!(rig.stop().match_state.end, Some(MatchEndDto::Stopped));
    assert!(!rig.state.serialize().unwrap().contains("RE["));
}

#[test]
fn human_white_engine_opening_and_occupied_human_point_preserve_turn() {
    let rig = Rig::new("normal");
    rig.start(PlayerColor::White);
    assert!(rig.state.human_match_action(&rig.manager, rig.token(), HumanMatchActionDto::Play { vertex: point(0, 0) }).is_err());
    let update = rig.engine();
    assert_eq!(update.match_state.to_play, Some(PlayerColor::White));
    let before = rig.token();
    assert!(rig.state.human_match_action(&rig.manager, before.clone(), HumanMatchActionDto::Play { vertex: point(3, 1) }).is_err());
    assert_eq!(before, rig.token());
    rig.human(MoveVertex::Pass);
    assert!(rig.state.serialize().unwrap().contains(";B[db];W[]"));
    rig.stop();
}

#[test]
fn failed_defaults_write_preserves_entire_document_foreground_and_durable_settings() {
    let rig = Rig::new("normal");
    let old_run = rig.foreground();
    rig.state.set_personal_comment(NodePath { indices: vec![1] }, "unsaved personal".into()).unwrap();
    let before = rig.state.human_match_snapshot().current.unwrap();
    let sgf = rig.state.serialize().unwrap();
    let durable = app_preferences::save_to_path(&rig.path, app_preferences::default_app_preferences()).unwrap();
    let bytes = std::fs::read(&rig.path).unwrap();
    let blocked = rig.directory.join("not-a-directory");
    std::fs::write(&blocked, "block").unwrap();
    let result = rig.state.start_human_match(&rig.manager, &rig.preferences, &blocked.join("prefs.json"),
        rig.request(PlayerColor::White), rig.profile.clone(), |_| {});
    assert!(result.is_err());
    assert_eq!(rig.state.human_match_snapshot().current.unwrap(), before);
    assert_eq!(rig.state.serialize().unwrap(), sgf);
    assert_eq!(std::fs::read(&rig.path).unwrap(), bytes);
    assert_eq!(app_preferences::load_from_path(&rig.path).unwrap().preferences, durable);
    assert!(matches!(rig.manager.snapshot().lifecycle, ForegroundEngineLifecycleDto::Ready { run } if run.run_id == old_run));
    assert!(!rig.state.human_match_snapshot().match_state.resources_held);
}

#[test]
fn starting_stop_and_exit_race_never_publish_candidate_document_or_defaults() {
    for exit in [false, true] {
        let rig = Rig::new("normal");
        let before = rig.state.human_match_snapshot().current.unwrap();
        let old = rig.state.serialize().unwrap();
        let result = rig.state.start_human_match(&rig.manager, &rig.preferences, &rig.path,
            rig.request(PlayerColor::Black), rig.profile.clone(), |update| {
                if update.match_state.phase != MatchPhaseDto::Starting { return; }
                if exit { rig.state.prepare_exit().unwrap(); }
                else { rig.state.stop_human_match(&rig.manager, update.match_state.session_id.as_deref().unwrap()).unwrap(); }
            });
        assert!(result.is_err());
        assert_eq!(rig.state.serialize().unwrap(), old);
        assert_eq!(rig.state.human_match_snapshot().current.unwrap(), before);
        assert!(!rig.path.exists());
        assert!(!rig.state.human_match_snapshot().match_state.committed);
    }
}

#[test]
fn two_passes_seal_before_scoring_and_human_resignation_writes_standard_result() {
    let rig = Rig::new("pass");
    rig.start(PlayerColor::Black);
    rig.human(MoveVertex::Pass);
    let update = rig.engine();
    assert_eq!(update.match_state.end, Some(MatchEndDto::TwoPasses));
    assert!(rig.state.serialize().unwrap().contains(";B[];W[]"));
    assert!(rig.state.human_match_action(&rig.manager, rig.token(), HumanMatchActionDto::Resign).is_err());
    rig.stop();
    assert!(!rig.state.serialize().unwrap().contains("RE["));
    rig.start(PlayerColor::Black);
    rig.state.human_match_action(&rig.manager, rig.token(), HumanMatchActionDto::Resign).unwrap();
    rig.stop();
    assert!(rig.state.serialize().unwrap().contains("RE[W+R]"));
}

#[test]
fn completed_engine_result_is_consumed_once_and_stop_seals_unpublished_result() {
    for stop_before in [false, true] {
        let rig = Rig::new("normal");
        rig.start(PlayerColor::White);
        let (token, handle) = rig.state.take_match_engine_turn(&rig.manager).unwrap().unwrap();
        let result = handle.wait().unwrap();
        if stop_before { rig.stop(); }
        let update = rig.state.accept_match_engine_result(&rig.manager, &token, Ok(result.clone()));
        let sgf = rig.state.serialize().unwrap();
        rig.state.accept_match_engine_result(&rig.manager, &token, Ok(result));
        assert_eq!(rig.state.serialize().unwrap(), sgf);
        assert_eq!(update.match_state.turn, if stop_before { 1 } else { 2 });
        rig.stop();
    }
}

#[test]
fn runtime_failures_keep_committed_document_and_profile_without_fake_result() {
    for mode in ["occupied", "hold", "missing", "exit"] {
        let rig = Rig::new(mode);
        rig.start(PlayerColor::Black);
        rig.human(point(0, 0));
        let before = rig.state.serialize().unwrap();
        let update = rig.engine();
        assert_eq!(update.match_state.end, Some(MatchEndDto::Failed));
        let failure = update.match_state.failure.as_ref().unwrap();
        assert_eq!(failure.profile_id.as_deref(), Some("engine"));
        assert_eq!(failure.run_id, update.match_state.run_id);
        assert_eq!(update.match_state.failed_side, Some(PlayerColor::White));
        assert!(update.match_state.committed);
        assert_eq!(rig.state.serialize().unwrap(), before);
        let stopped = rig.stop();
        assert!(!stopped.match_state.resources_held);
        assert_eq!(stopped.match_state.phase, MatchPhaseDto::Error);
        assert!(!rig.state.serialize().unwrap().contains("RE["));
        let pid = std::fs::read_to_string(rig.directory.join("pid")).unwrap();
        assert!(!PathBuf::from(format!("/proc/{}", pid.trim())).exists());
    }
}

#[test]
fn start_is_one_history_unit_and_saved_defaults_never_contain_live_session() {
    let rig = Rig::new("normal");
    let before = rig.state.serialize().unwrap();
    let old_selection = rig.state.human_match_snapshot().current.unwrap().selected_path;
    let update = rig.start(PlayerColor::Black);
    let settings = update.match_state.settings.unwrap();
    let started = rig.state.serialize().unwrap();
    rig.stop();
    let undo = rig.state.undo(rig.state.inspect().0).unwrap();
    assert_eq!(rig.state.serialize().unwrap(), before);
    assert_eq!(undo.selected_path, old_selection);
    rig.state.redo(rig.state.inspect().0).unwrap();
    assert_eq!(rig.state.serialize().unwrap(), started);
    assert_eq!(rig.state.human_match_snapshot().match_state.phase, MatchPhaseDto::Idle);
    let saved = rig.state.save_to_path(rig.directory.join("match.sgf").to_string_lossy().into(), NodePath::default()).unwrap();
    assert!(!saved.dirty);
    let defaults = app_preferences::load_from_path(&rig.path).unwrap().preferences;
    assert_eq!(defaults.match_defaults, settings);
    let persisted = std::fs::read_to_string(&rig.path).unwrap();
    for forbidden in ["session_id", "run_id", "job_id", "human-1", "Playing"] { assert!(!persisted.contains(forbidden)); }
    let restarted = CurrentGameState::default();
    restarted.replace(&std::fs::read_to_string(rig.directory.join("match.sgf")).unwrap(), None).unwrap();
    assert_eq!(restarted.human_match_snapshot().match_state.phase, MatchPhaseDto::Idle);
    assert_eq!(restarted.serialize().unwrap(), started);
}

#[test]
fn recovery_producer_excludes_starting_candidate_and_restores_only_committed_review() {
    let rig = Rig::new("normal");
    let old = rig.state.serialize().unwrap();
    rig.state.start_human_match(&rig.manager, &rig.preferences, &rig.path,
        rig.request(PlayerColor::Black), rig.profile.clone(), |update| {
            if update.match_state.phase != MatchPhaseDto::Starting { return; }
            let envelope = rig.state.take_due_recovery_write(u64::MAX).unwrap();
            assert_eq!(envelope.sgf_text, old);
            rig.state.finish_recovery_write(envelope, Ok(()));
        }).unwrap();
    rig.human(point(0, 0));
    let envelope = rig.state.take_due_recovery_write(u64::MAX).unwrap();
    assert_eq!(envelope.sgf_text, rig.state.serialize().unwrap());
    assert!(envelope.sgf_text.contains(";B[aa]"));
    assert!(rig.state.restore_envelope(envelope.clone()).is_err());
    let bytes = serde_json::to_vec(&envelope).unwrap();
    let path = rig.directory.join("recovery.json");
    std::fs::write(&path, &bytes).unwrap();
    let target = CurrentGameState::default();
    target.restore_envelope(serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap()).unwrap();
    assert_eq!(target.serialize().unwrap(), envelope.sgf_text);
    assert_eq!(target.human_match_snapshot().match_state, MatchSnapshotDto::default());
    assert!(!target.human_match_snapshot().current.unwrap().can_undo);
    rig.stop();
}

#[test]
fn reserved_display_save_is_consistent_and_analysis_change_is_atomic() {
    let rig = Rig::new("normal");
    rig.start(PlayerColor::Black);
    let mut display = rig.preferences.load(&rig.path, &rig.manager).unwrap().preferences;
    display.board_theme = "high-contrast".into();
    let saved = rig.preferences.save(&rig.path, &rig.manager, display).unwrap();
    assert_eq!(saved.board_theme, "high-contrast");
    assert_eq!(rig.preferences.load(&rig.path, &rig.manager).unwrap().preferences, saved);
    assert_eq!(app_preferences::load_from_path(&rig.path).unwrap().preferences, saved);
    let before = std::fs::read(&rig.path).unwrap();
    let mut rejected = saved.clone();
    rejected.continuous_analysis_enabled = !saved.continuous_analysis_enabled;
    assert!(rig.preferences.save(&rig.path, &rig.manager, rejected).is_err());
    assert_eq!(std::fs::read(&rig.path).unwrap(), before);
    assert_eq!(rig.preferences.load(&rig.path, &rig.manager).unwrap().preferences, saved);
    rig.stop();
}

#[test]
fn restored_match_with_ready_engine_and_enabled_intent_stays_review_only() {
    let rig = Rig::new("normal");
    rig.start(PlayerColor::Black);
    rig.human(point(0, 0));
    let envelope = rig.state.take_due_recovery_write(u64::MAX).unwrap();
    rig.stop();
    for ready_before_restore in [true, false] {
        let resumed = Rig::new("normal");
        let target = CurrentGameState::default();
        target.connect_analysis_manager(resumed.manager.clone());
        resumed.manager.clear_continuous_position();
        resumed.manager.set_continuous_preferences(true, Default::default()).unwrap();
        if ready_before_restore { resumed.foreground(); }
        target.restore_envelope(envelope.clone()).unwrap();
        if !ready_before_restore { resumed.foreground(); }
        assert!(resumed.manager.snapshot().selected_node_job.is_none(), "recovery admitted ordinary analysis");
        assert_eq!(target.human_match_snapshot().match_state, MatchSnapshotDto::default());
    }
}

#[test]
fn corrupt_match_defaults_recover_and_allow_next_preference_save() {
    let rig = Rig::new("normal");
    for invalid in [serde_json::json!({"deadline_ms": 0}), serde_json::json!({"handicap": 1})] {
        let mut json = serde_json::to_value(app_preferences::default_app_preferences()).unwrap();
        json["matchDefaults"] = invalid;
        std::fs::write(&rig.path, serde_json::to_vec(&json).unwrap()).unwrap();
        let preferences = crate::continuous_analysis::PreferencesState::default();
        let recovered = preferences.load(&rig.path, &rig.manager).unwrap();
        assert!(recovered.recovery.is_some());
        assert_eq!(recovered.preferences.match_defaults, MatchDefaultsDto::default());
        preferences.save(&rig.path, &rig.manager, recovered.preferences).unwrap();
    }
}

#[test]
fn fixed_handicap_commits_actual_stones_white_turn_and_durable_defaults() {
    let rig = Rig::new("normal");
    let mut request = rig.request(PlayerColor::White);
    request.settings.board_size = 9;
    request.settings.handicap = 2;
    let settings = request.settings.clone();
    let update = rig.state.start_human_match(&rig.manager, &rig.preferences, &rig.path,
        request, rig.profile.clone(), |_| {}).unwrap();
    assert_eq!(update.match_state.phase, MatchPhaseDto::Playing);
    assert_eq!(update.match_state.to_play, Some(PlayerColor::White));
    assert_eq!(app_preferences::load_from_path(&rig.path).unwrap().preferences.match_defaults, settings);
    let sgf = rig.state.serialize().unwrap();
    assert!(sgf.contains("AB[cg][gc]") || sgf.contains("AB[gc][cg]"), "{sgf}");
    assert!(sgf.contains("HA[2]") && sgf.contains("PL[W]"), "{sgf}");
    assert!(rig.state.take_match_engine_turn(&rig.manager).unwrap().is_none());
    rig.human(MoveVertex::Pass);
    assert!(rig.state.serialize().unwrap().contains(";W[]"));
    rig.stop();
}

#[test]
fn unsupported_exact_position_preserves_owner_and_foreground_after_ready_validation() {
    let rig = Rig::new("normal");
    let old_run = rig.foreground();
    let before = rig.state.human_match_snapshot().current.unwrap();
    let old = rig.state.serialize().unwrap();
    let mut request = rig.request(PlayerColor::Black);
    request.settings.komi = 500.0;
    let error = rig.state.start_human_match(&rig.manager, &rig.preferences, &rig.path,
        request, rig.profile.clone(), |_| {}).unwrap_err();
    assert_eq!(error.kind, EngineFailureKind::UnsupportedCapability);
    assert_eq!(rig.state.human_match_snapshot().current.unwrap(), before);
    assert_eq!(rig.state.serialize().unwrap(), old);
    assert!(!rig.path.exists());
    assert!(!rig.state.human_match_snapshot().match_state.resources_held);
    assert!(matches!(rig.manager.snapshot().lifecycle, ForegroundEngineLifecycleDto::Ready { run } if run.run_id == old_run));
}

#[test]
fn engine_resignation_is_current_side_result_and_late_results_are_sealed() {
    let rig = Rig::with_adapter("resign", false);
    rig.start(PlayerColor::White);
    let (token, handle) = rig.state.take_match_engine_turn(&rig.manager).unwrap().unwrap();
    let result = handle.wait().unwrap();
    let update = rig.state.accept_match_engine_result(&rig.manager, &token, Ok(result.clone()));
    assert_eq!(update.match_state.end, Some(MatchEndDto::Resigned));
    assert!(rig.state.serialize().unwrap().contains("RE[W+R]"));
    rig.stop();
    let before = rig.state.serialize().unwrap();
    rig.state.accept_match_engine_result(&rig.manager, &token, Ok(result));
    assert_eq!(rig.state.serialize().unwrap(), before);
}

const CONTINUE_SOURCE: &str = "(;SZ[5]RU[Chinese-KGS]KM[6.5]PB[Old black]PW[Old white]RE[W+R]C[old](;B[aa]C[branch])(;B[bb];W[db]C[old reply]))";

#[test]
fn continue_from_non_mainline_node_is_independent_mainline_and_one_history_unit() {
    let rig = Rig::new("normal");
    rig.open(CONTINUE_SOURCE, vec![1]);
    let before = rig.state.human_match_snapshot().current.unwrap();
    let old = rig.state.serialize().unwrap();
    let mut request = rig.continue_request(PlayerColor::White, true);
    // New-game fields never rewrite the inherited position.
    request.settings.board_size = 9;
    request.settings.komi = 0.5;
    request.settings.handicap = 2;
    let update = rig.try_start(request).unwrap();
    let current = update.current.unwrap();
    assert_eq!(current.selected_path, NodePath { indices: vec![0, 0] });
    assert_eq!(current.native_path, before.native_path);
    assert!(current.generation > before.generation && current.dirty);
    assert_eq!(update.match_state.to_play, Some(PlayerColor::White));
    let defaults = app_preferences::load_from_path(&rig.path).unwrap().preferences.match_defaults;
    assert_eq!(defaults, MatchDefaultsDto { human_color: PlayerColor::White, profile_id: Some("engine".into()),
        deadline_ms: 1000, kata_max_visits: 8, ..MatchDefaultsDto::default() });
    assert_eq!(update.match_state.settings, Some(defaults));
    let started = rig.state.serialize().unwrap();
    assert!(started.contains("SZ[5]") && started.contains("KM[6.5]") && !started.contains("HA["), "{started}");
    assert!(started.contains("PB[controlled engine]") && started.contains("PW[Human]") && !started.contains("RE["), "{started}");
    assert!(started.contains("C[old]") && started.contains("C[branch]") && started.contains("C[old reply]"), "{started}");
    // References from before the start are stale after the path reorder.
    let stale = MatchTurnDto { session_id: update.match_state.session_id.clone().unwrap(), turn: 1,
        generation: before.generation, node_path: before.selected_path.clone() };
    assert!(rig.state.human_match_action(&rig.manager, stale, HumanMatchActionDto::Play { vertex: point(3, 1) }).is_err());
    // The same move as the old continuation creates a new node instead of reusing it.
    let played = rig.human(point(3, 1)).current.unwrap();
    assert_eq!(played.selected_path, NodePath { indices: vec![0, 0, 0] });
    let envelope = rig.state.take_due_recovery_write(u64::MAX).unwrap();
    assert_eq!(envelope.sgf_text, rig.state.serialize().unwrap());
    assert_eq!(envelope.sgf_text.matches("W[db]").count(), 2, "{}", envelope.sgf_text);
    rig.stop();
    rig.state.undo(rig.state.inspect().0).unwrap();
    assert_eq!(rig.state.serialize().unwrap(), started);
    let undone = rig.state.undo(rig.state.inspect().0).unwrap();
    assert_eq!(rig.state.serialize().unwrap(), old);
    assert_eq!(undone.selected_path, before.selected_path);
    rig.state.redo(rig.state.inspect().0).unwrap();
    let redone = rig.state.redo(rig.state.inspect().0).unwrap();
    let continued = rig.state.serialize().unwrap();
    let file = rig.directory.join("continued.sgf");
    assert!(!rig.state.save_to_path(file.to_string_lossy().into(), redone.selected_path).unwrap().dirty);
    let reopened = CurrentGameState::default();
    reopened.replace(&std::fs::read_to_string(&file).unwrap(), None).unwrap();
    assert_eq!(reopened.serialize().unwrap(), continued);
    assert_eq!(reopened.human_match_snapshot().current.unwrap().selected_path, NodePath { indices: vec![0, 0, 0] });
    assert_eq!(reopened.human_match_snapshot().match_state, MatchSnapshotDto::default());
}

#[test]
fn zero_move_stop_or_resign_keeps_structure_endpoint_without_fake_move() {
    for resign in [false, true] {
        let rig = Rig::new("normal");
        rig.open(CONTINUE_SOURCE, vec![1]);
        rig.try_start(rig.continue_request(PlayerColor::White, true)).unwrap();
        if resign { rig.state.human_match_action(&rig.manager, rig.token(), HumanMatchActionDto::Resign).unwrap(); }
        let stopped = rig.stop();
        let current = stopped.current.unwrap();
        assert_eq!(current.selected_path, NodePath { indices: vec![0, 0] });
        assert!(current.tree.children[0].children[0].children.is_empty());
        let sgf = rig.state.serialize().unwrap();
        assert_eq!(sgf.matches(";W[").count(), 1, "{sgf}");
        assert_eq!(sgf.contains("RE[B+R]"), resign, "{sgf}");
        assert!(!sgf.contains("RE[W+R]"), "{sgf}");
    }
}

#[test]
fn inherited_trailing_pass_and_structure_node_complete_double_pass() {
    let rig = Rig::new("normal");
    rig.open("(;SZ[5]RU[Chinese-KGS]KM[6.5];B[aa];W[])", vec![0, 0]);
    rig.try_start(rig.continue_request(PlayerColor::Black, true)).unwrap();
    let update = rig.human(MoveVertex::Pass);
    assert_eq!(update.match_state.end, Some(MatchEndDto::TwoPasses));
    rig.stop();
}

#[test]
fn continue_from_implicit_handicap_setup_agrees_on_white_to_play() {
    let rig = Rig::new("normal");
    rig.open("(;SZ[5]RU[Chinese-KGS]KM[0.5]HA[2]AB[aa][ee])", vec![]);
    let update = rig.try_start(rig.continue_request(PlayerColor::White, true)).unwrap();
    assert_eq!(update.match_state.to_play, Some(PlayerColor::White));
    let current = update.current.unwrap();
    assert_eq!(current.snapshot.position.to_play, PlayerColor::White);
    assert_eq!(current.snapshot.position.stones.len(), 2);
    let sgf = rig.state.serialize().unwrap();
    assert!(sgf.contains("HA[2]") && sgf.contains("AB[aa][ee]") && !sgf.contains("PL[") && !sgf.contains(";W["), "{sgf}");
    rig.stop();
}

#[test]
fn continue_survives_analysis_accepted_while_the_dialog_was_open() {
    let rig = Rig::new("normal");
    rig.open(CONTINUE_SOURCE, vec![1]);
    // The dialog captured this request; continuous analysis then attaches to the same position.
    let request = rig.continue_request(PlayerColor::White, true);
    let payload = sgf::SgfAnalysisPayload { engine_name: "KataGo".into(), visits: 64, winrate_black: 0.5,
        score_mean_black: None, score_stdev: None, pda: None, candidates: vec![app_model::CandidateMoveDto {
            vertex: MoveVertex::Pass, visits: 64, winrate_black: 0.5, score_mean_black: 0.0, policy_prior: None,
            pv: vec![MoveVertex::Pass] }], ownership: None };
    let attached = rig.state.attach_primary_analysis(request.generation, NodePath { indices: vec![1] }, payload).unwrap();
    assert!(attached.snapshot_seq > request.snapshot_seq && attached.generation == request.generation);
    let update = rig.try_start(request).unwrap();
    assert_eq!(update.current.unwrap().selected_path, NodePath { indices: vec![0, 0] });
    // Analysis accepted before the start stays on the start node.
    assert!(rig.state.serialize().unwrap().contains("LZ"), "{}", rig.state.serialize().unwrap());
    rig.stop();
}

#[test]
fn rejected_or_failed_continue_preserves_document_foreground_and_defaults() {
    let rig = Rig::new("normal");
    let old_run = rig.foreground();
    rig.open(CONTINUE_SOURCE, vec![1]);
    rig.state.set_personal_comment(NodePath { indices: vec![1] }, "unsaved personal".into()).unwrap();
    let before = rig.state.human_match_snapshot().current.unwrap();
    let old = rig.state.serialize().unwrap();
    let unchanged = |rig: &Rig| {
        assert_eq!(rig.state.human_match_snapshot().current.unwrap(), before);
        assert_eq!(rig.state.serialize().unwrap(), old);
        assert!(!rig.path.exists());
        assert!(!rig.state.human_match_snapshot().match_state.resources_held);
        let lifecycle = rig.manager.snapshot().lifecycle;
        assert!(matches!(&lifecycle, ForegroundEngineLifecycleDto::Ready { run } if run.run_id == old_run), "{lifecycle:?}; trace={:?}", std::fs::read_to_string(rig.directory.join("trace")));
    };
    assert!(rig.try_start(rig.continue_request(PlayerColor::White, false)).is_err());
    unchanged(&rig);
    let mut moved = rig.continue_request(PlayerColor::White, true);
    moved.start = MatchStartDto::Continue { node_path: NodePath { indices: vec![0] }, root_metadata_confirmed: true };
    assert!(rig.try_start(moved).is_err());
    unchanged(&rig);
    let blocked = rig.directory.join("not-a-directory");
    std::fs::write(&blocked, "block").unwrap();
    assert!(rig.state.start_human_match(&rig.manager, &rig.preferences, &blocked.join("prefs.json"),
        rig.continue_request(PlayerColor::White, true), rig.profile.clone(), |_| {}).is_err());
    unchanged(&rig);

    let rig = Rig::new("normal");
    rig.open("(;SZ[5]RU[Japanese]KM[6.5];B[aa])", vec![0]);
    let old = rig.state.serialize().unwrap();
    let error = rig.try_start(rig.continue_request(PlayerColor::White, true)).unwrap_err();
    assert_eq!(error.kind, EngineFailureKind::UnsupportedCapability);
    assert_eq!(rig.state.serialize().unwrap(), old);
    assert!(!rig.path.exists());
    assert_eq!(rig.state.human_match_snapshot().match_state.phase, MatchPhaseDto::Idle);
}

mod pk;
