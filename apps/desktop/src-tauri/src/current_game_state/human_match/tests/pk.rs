use super::*;

struct PkRig(Rig);
impl PkRig {
    fn new() -> Self {
        Self::with_config(ForegroundEngineConfig::for_tests())
    }
    fn with_config(config: ForegroundEngineConfig) -> Self {
        let rig = Rig::with_config("normal", true, config);
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../crates/engine-manager/tests/fixtures/pk_process.py");
        std::fs::write(rig.directory.join("engine"), format!("#!/bin/sh\nexec /usr/bin/python3 '{}' \"$@\"\n", fixture.display())).unwrap();
        Self(rig)
    }
    fn request(&self, limit: u32, continuing: bool) -> HumanMatchStartDto {
        let mut request = if continuing { self.0.continue_request(PlayerColor::Black, true) }
            else { self.0.request(PlayerColor::Black) };
        request.settings.pk_black = PkSideSettingsDto { profile_id: Some("engine".into()), deadline_ms: 3000, kata_max_visits: 11 };
        request.settings.pk_white = PkSideSettingsDto { profile_id: Some("engine".into()), deadline_ms: 4000, kata_max_visits: 17 };
        request.settings.pk_max_moves = limit;
        request
    }
    fn start(&self, limit: u32, continuing: bool) -> MatchUpdateDto {
        self.0.state.start_pk_match(&self.0.manager, &self.0.preferences, &self.0.path,
            self.request(limit, continuing), [self.0.profile.clone(), self.0.profile.clone()], |_| {}).unwrap()
    }
    fn take(&self) -> (MatchTurnDto, GameMoveHandle, serde_json::Value) {
        let (token, handle) = self.0.state.take_match_engine_turn(&self.0.manager).unwrap().unwrap();
        let path = self.0.directory.join(format!("pending-{}", handle.identity.job_id));
        wait_for(|| std::fs::read_to_string(&path).ok().and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok()).is_some());
        let request = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        (token, handle, request)
    }
    fn reply(&self, job: &str, value: &str) {
        std::fs::write(self.0.directory.join(format!("reply-{job}")), value).unwrap();
    }
    fn play(&self, vertex: &str) -> (MatchUpdateDto, serde_json::Value) {
        let (token, handle, request) = self.take();
        self.reply(&handle.identity.job_id, vertex);
        (self.0.state.accept_match_engine_result(&self.0.manager, &token, handle.wait()), request)
    }
    fn pids(&self) -> Vec<u32> {
        std::fs::read_to_string(self.0.directory.join("pids")).unwrap().lines().map(|line| line.parse().unwrap()).collect()
    }
    fn assert_reaped(&self, pids: &[u32]) {
        wait_for(|| pids.iter().all(|pid| !PathBuf::from(format!("/proc/{pid}")).exists()));
    }
    fn pause(&self) -> MatchUpdateDto {
        self.0.state.pause_pk_match(&self.0.manager, &self.0.token().session_id, |_| {}).unwrap()
    }
    fn resume(&self, id: &str) -> MatchUpdateDto {
        self.0.state.resume_pk_match(&self.0.manager, id, |_| {}).unwrap()
    }
}
fn wait_for(mut predicate: impl FnMut() -> bool) {
    let until = Instant::now() + Duration::from_secs(5);
    while !predicate() { assert!(Instant::now() < until, "controlled subprocess handshake timed out"); std::thread::sleep(Duration::from_millis(5)); }
}
fn assert_exclusive(rig: &Rig) {
    let token = rig.token();
    assert!(rig.state.select_path(NodePath::default(), token.generation).is_err());
    assert!(rig.state.play(token.node_path.clone(), point(0, 0)).is_err());
    assert!(rig.state.human_match_action(&rig.manager, token.clone(), HumanMatchActionDto::Play { vertex: MoveVertex::Pass }).is_err());
    assert!(rig.state.admit_selected_node(token.generation, &token.node_path).is_err());
    assert!(rig.manager.start("engine").is_err());
    assert!(rig.manager.reserve_match("another-session").is_err());
    for policy in [MatchAnalysisPolicyDto::Off, MatchAnalysisPolicyDto::Both] {
        assert!(rig.state.human_match_analysis_policy(&rig.manager, token.clone(), policy).is_err());
    }
    let analysis = rig.state.human_match_snapshot().match_state.analysis;
    assert!(!analysis.supported);
    assert_eq!(analysis.policy, MatchAnalysisPolicyDto::Off);
    assert!(analysis.frame.is_none());
}

#[test]
fn same_profile_uses_two_pids_alternates_once_and_preserves_pause_identity_and_limit() {
    let rig = PkRig::new();
    let started = rig.start(3, false);
    let runs = started.match_state.pk_runs.unwrap();
    assert_ne!(runs[0].run_id, runs[1].run_id);
    let pids = rig.pids();
    assert_eq!(pids.len(), 2);
    assert_ne!(pids[0], pids[1]);
    assert_exclusive(&rig.0);
    let (first, request) = rig.play("A5");
    assert_eq!(request["pid"], pids[0]);
    assert_eq!(request["request"]["maxVisits"], 11);
    assert_eq!(first.match_state.committed_moves, 1);
    assert_eq!(first.match_state.to_play, Some(PlayerColor::White));
    let (old_token, old_handle, old_request) = rig.take();
    let old_job = old_handle.identity.job_id.clone();
    assert_eq!(old_request["pid"], pids[1]);
    let paused = rig.pause();
    assert_eq!(paused.match_state.phase, MatchPhaseDto::Paused);
    assert!(!paused.match_state.pause_pending);
    assert_eq!(paused.match_state.committed_moves, 1);
    assert_eq!(paused.match_state.to_play, Some(PlayerColor::White));
    assert_eq!(paused.current, first.current);
    assert_eq!(rig.pause(), paused);
    assert_exclusive(&rig.0);
    let id = paused.match_state.session_id.unwrap();
    rig.resume(&id);
    rig.resume(&id);
    let (token, handle, request) = rig.take();
    assert_ne!(handle.identity.job_id, old_job);
    assert_eq!(request["pid"], pids[1]);
    assert_eq!(request["request"]["maxVisits"], 17);
    assert_eq!(request["request"]["moves"], serde_json::json!([["B", "A5"]]));
    // Both old failures and results remain sealed after Resume at the same position.
    rig.0.state.accept_match_engine_result(&rig.0.manager, &old_token, old_handle.wait());
    assert_eq!(rig.0.state.human_match_snapshot().match_state.job.as_ref(), Some(&handle.identity));
    rig.reply(&old_job, "E1");
    rig.reply(&handle.identity.job_id, "B5");
    let result = handle.wait().unwrap();
    let accepted = rig.0.state.accept_match_engine_result(&rig.0.manager, &token, Ok(result.clone()));
    assert_eq!(accepted.match_state.committed_moves, 2);
    assert_eq!(rig.0.state.accept_match_engine_result(&rig.0.manager, &token, Ok(result)), accepted);
    let (ended, request) = rig.play("C5");
    assert_eq!(request["pid"], pids[0]);
    assert_eq!(ended.match_state.end, Some(MatchEndDto::MoveLimit));
    assert_eq!(ended.match_state.committed_moves, 3);
    assert!(rig.0.state.serialize().unwrap().contains(";B[aa];W[ba];B[ca]"));
    assert!(!rig.0.state.serialize().unwrap().contains("RE["));
    let stopped = rig.0.stop();
    assert!(!stopped.match_state.resources_held);
    assert_eq!(rig.0.stop(), stopped);
    rig.assert_reaped(&pids);
    assert!(rig.0.manager.snapshot().selected_node_job.is_none());
}

#[test]
fn continue_delivers_exact_history_to_both_runs_and_counts_only_new_commits() {
    let rig = PkRig::new();
    rig.0.open("(;SZ[5]RU[Chinese-KGS]KM[6.5]PB[old black]PW[old white]RE[B+R]HA[2]AB[aa][ee]PL[W];W[bb]C[keep](;B[cc]C[old branch]))", vec![0]);
    let before = rig.0.state.serialize().unwrap();
    let started = rig.start(2, true);
    assert_eq!(started.match_state.committed_moves, 0);
    assert_eq!(started.match_state.to_play, Some(PlayerColor::Black));
    let branch = started.current.unwrap().selected_path;
    let (first, black) = rig.play("D5");
    assert_eq!(black["request"]["initialStones"], serde_json::json!([["B", "A5"], ["B", "E1"]]));
    assert_eq!(black["request"]["moves"], serde_json::json!([["W", "B4"]]));
    assert_eq!(black["request"]["initialPlayer"], "W");
    assert_eq!(black["request"]["komi"], 6.5);
    assert_eq!(first.match_state.committed_moves, 1);
    assert!(first.current.unwrap().selected_path.indices.starts_with(&branch.indices));
    let (ended, white) = rig.play("E5");
    assert_eq!(white["request"]["initialStones"], black["request"]["initialStones"]);
    assert_eq!(white["request"]["moves"], serde_json::json!([["W", "B4"], ["B", "D5"]]));
    assert_eq!(white["request"]["rules"], black["request"]["rules"]);
    assert_eq!(ended.match_state.end, Some(MatchEndDto::MoveLimit));
    let after = rig.0.state.serialize().unwrap();
    assert!(after.contains("C[keep]") && after.contains("C[old branch]") && !after.contains("RE["), "{after}");
    rig.0.stop();
    rig.0.state.undo(rig.0.state.inspect().0).unwrap();
    rig.0.state.undo(rig.0.state.inspect().0).unwrap();
    rig.0.state.undo(rig.0.state.inspect().0).unwrap();
    assert_eq!(rig.0.state.serialize().unwrap(), before);
}

#[test]
fn second_candidate_failure_and_default_write_failure_leave_old_authority_untouched() {
    for failed_defaults in [false, true] {
        let rig = PkRig::new();
        let old_run = rig.0.foreground();
        let old_pid = rig.pids()[0];
        rig.0.state.set_personal_comment(NodePath { indices: vec![1] }, "dirty personal".into()).unwrap();
        let before = rig.0.state.human_match_snapshot().current.unwrap();
        let sgf = rig.0.state.serialize().unwrap();
        app_preferences::save_to_path(&rig.0.path, app_preferences::default_app_preferences()).unwrap();
        let bytes = std::fs::read(&rig.0.path).unwrap();
        let blocked = rig.0.directory.join("not-a-directory");
        std::fs::write(&blocked, "block").unwrap();
        if !failed_defaults { std::fs::write(rig.0.directory.join("startup-3"), "fail").unwrap(); }
        let path = if failed_defaults { blocked.join("prefs.json") } else { rig.0.path.clone() };
        let result = rig.0.state.start_pk_match(&rig.0.manager, &rig.0.preferences, &path,
            rig.request(4, false), [rig.0.profile.clone(), rig.0.profile.clone()], |update| {
                if update.match_state.phase == MatchPhaseDto::Starting { assert_exclusive(&rig.0); }
            });
        assert!(result.is_err());
        if !failed_defaults { assert_eq!(rig.0.state.human_match_snapshot().match_state.failed_side, Some(PlayerColor::White)); }
        assert_eq!(rig.0.state.human_match_snapshot().current.unwrap(), before);
        assert_eq!(rig.0.state.serialize().unwrap(), sgf);
        assert_eq!(std::fs::read(&rig.0.path).unwrap(), bytes);
        assert!(!rig.0.state.human_match_snapshot().match_state.resources_held);
        assert!(matches!(rig.0.manager.snapshot().lifecycle, ForegroundEngineLifecycleDto::Ready { run } if run.run_id == old_run));
        assert!(PathBuf::from(format!("/proc/{old_pid}")).exists());
        let pids = rig.pids();
        assert_eq!(pids.len(), 3);
        rig.assert_reaped(&pids[1..]);
    }
}

#[test]
fn each_side_failure_ends_whole_match_without_result_or_analysis_revival() {
    for white in [false, true] {
        for cause in ["eof", "protocol", "timeout"] {
            let rig = PkRig::new();
            let mut request = rig.request(8, false);
            request.settings.pk_black.deadline_ms = 500;
            request.settings.pk_white.deadline_ms = 500;
            rig.0.state.start_pk_match(&rig.0.manager, &rig.0.preferences, &rig.0.path,
                request, [rig.0.profile.clone(), rig.0.profile.clone()], |_| {}).unwrap();
            if white { rig.play("A5"); }
            let before = rig.0.state.serialize().unwrap();
            let (token, handle, _) = rig.take();
            if cause != "timeout" { rig.reply(&handle.identity.job_id, cause); }
            let update = rig.0.state.accept_match_engine_result(&rig.0.manager, &token, handle.wait());
            assert_eq!(update.match_state.end, Some(MatchEndDto::Failed), "{cause}: {update:?}");
            assert_eq!(update.match_state.failed_side, Some(if white { PlayerColor::White } else { PlayerColor::Black }));
            assert_eq!(update.match_state.failure.unwrap().profile_id.as_deref(), Some("engine"));
            assert_eq!(rig.0.state.serialize().unwrap(), before);
            assert!(!rig.0.stop().match_state.resources_held);
            rig.assert_reaped(&rig.pids());
            assert!(rig.0.manager.snapshot().selected_node_job.is_none());
        }
    }
}

#[test]
fn paused_extra_run_exit_is_terminal_and_recovery_reopens_review_only() {
    let rig = PkRig::new();
    let started = rig.start(8, false);
    rig.play("A5");
    let paused = rig.pause();
    let id = paused.match_state.session_id.unwrap();
    let envelope = rig.0.state.take_due_recovery_write(u64::MAX).unwrap();
    let recovery = rig.0.directory.join("paused-recovery.json");
    std::fs::write(&recovery, serde_json::to_vec(&envelope).unwrap()).unwrap();
    let target = CurrentGameState::default();
    target.restore_envelope(serde_json::from_slice(&std::fs::read(recovery).unwrap()).unwrap()).unwrap();
    assert_eq!(target.serialize().unwrap(), rig.0.state.serialize().unwrap());
    assert_eq!(target.human_match_snapshot().match_state, MatchSnapshotDto::default());
    assert!(target.resume_pk_match(&rig.0.manager, &id, |_| {}).is_err());
    let receiver = rig.0.manager.subscribe();
    std::fs::write(rig.0.directory.join(format!("exit-{}", rig.pids()[1])), "exit").unwrap();
    let until = Instant::now() + Duration::from_secs(5);
    loop {
        assert!(Instant::now() < until, "idle participant exit was not published");
        if let Ok(ForegroundEngineEventDto::Failure { failure }) = receiver.recv_timeout(Duration::from_millis(50)) {
            if let Some(update) = rig.0.state.observe_match_failure(&failure) {
                assert_eq!(update.match_state.failed_side, Some(PlayerColor::White));
                assert_eq!(update.match_state.failure.unwrap().run_id, Some(started.match_state.pk_runs.unwrap()[1].run_id.clone()));
                break;
            }
        }
    }
    assert!(rig.0.state.resume_pk_match(&rig.0.manager, &id, |_| {}).is_err());
    assert!(!rig.0.stop().match_state.resources_held);
    rig.assert_reaped(&rig.pids());
}

#[test]
fn inherited_pass_and_resignation_take_priority_over_move_limit() {
    for resign in [false, true] {
        let rig = PkRig::new();
        rig.0.open("(;SZ[5]RU[Chinese-KGS]KM[6.5];B[];C[structure])", vec![0, 0]);
        rig.start(1, true);
        let (token, handle, _) = rig.take();
        rig.reply(&handle.identity.job_id, "pass");
        let mut result = handle.wait().unwrap();
        // Typed-result owner seam only: KataGo analysis itself cannot produce resign.
        if resign { result.result = GameMoveDto::Resign; }
        let ended = rig.0.state.accept_match_engine_result(&rig.0.manager, &token, Ok(result));
        assert_eq!(ended.match_state.end, Some(if resign { MatchEndDto::Resigned } else { MatchEndDto::TwoPasses }));
        assert_eq!(ended.match_state.committed_moves, if resign { 0 } else { 1 });
        let sgf = rig.0.state.serialize().unwrap();
        if resign { assert!(sgf.contains("RE[B+R]"), "{sgf}"); } else { assert!(!sgf.contains("RE[")); }
        rig.0.stop();
    }
}

#[test]
fn cancellation_after_both_candidates_spawn_preserves_old_document_defaults_and_foreground() {
    let rig = PkRig::new();
    let old_run = rig.0.foreground();
    let before = rig.0.state.human_match_snapshot().current.unwrap();
    let sgf = rig.0.state.serialize().unwrap();
    app_preferences::save_to_path(&rig.0.path, app_preferences::default_app_preferences()).unwrap();
    let bytes = std::fs::read(&rig.0.path).unwrap();
    std::fs::write(rig.0.directory.join("startup-3"), "hold").unwrap();
    let request = rig.request(4, false);
    std::thread::scope(|scope| {
        let start = scope.spawn(|| rig.0.state.start_pk_match(&rig.0.manager, &rig.0.preferences, &rig.0.path,
            request, [rig.0.profile.clone(), rig.0.profile.clone()], |_| {}));
        wait_for(|| std::fs::read_to_string(rig.0.directory.join("pids")).unwrap_or_default().lines().count() == 3);
        assert_exclusive(&rig.0);
        rig.0.stop();
        assert!(start.join().unwrap().is_err());
    });
    assert_eq!(rig.0.state.human_match_snapshot().current.unwrap(), before);
    assert_eq!(rig.0.state.serialize().unwrap(), sgf);
    assert_eq!(std::fs::read(&rig.0.path).unwrap(), bytes);
    assert!(matches!(rig.0.manager.snapshot().lifecycle, ForegroundEngineLifecycleDto::Ready { run } if run.run_id == old_run));
    rig.assert_reaped(&rig.pids()[1..]);
}

#[test]
fn sealed_result_cannot_revive_when_a_new_session_returns_to_the_same_position() {
    let rig = PkRig::new();
    rig.start(4, false);
    let (old_token, old_handle, _) = rig.take();
    rig.reply(&old_handle.identity.job_id, "A5");
    let result = old_handle.wait().unwrap();
    rig.0.stop();
    rig.start(4, false);
    let (new_token, new_handle, _) = rig.take();
    assert_ne!(new_token.session_id, old_token.session_id);
    let before = rig.0.state.human_match_snapshot();
    assert_eq!(rig.0.state.accept_match_engine_result(&rig.0.manager, &old_token, Ok(result)), before);
    rig.reply(&new_handle.identity.job_id, "B5");
    let update = rig.0.state.accept_match_engine_result(&rig.0.manager, &new_token, new_handle.wait());
    assert_eq!(update.match_state.committed_moves, 1);
    assert!(rig.0.state.serialize().unwrap().contains(";B[ba]"));
    rig.0.stop();
}

mod gtp;
