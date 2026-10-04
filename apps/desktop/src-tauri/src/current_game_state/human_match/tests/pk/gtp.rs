use super::*;
use engine_manager::EngineProfileCatalog;

const GNU: [&str; 9] = [
    "--mode",
    "gtp",
    "--chinese-rules",
    "--positional-superko",
    "--forbid-suicide",
    "--level",
    "1",
    "--seed",
    "1",
];

impl PkRig {
    /// Saves a qualified GNU Go 3.8 profile that launches the same controlled fixture in GTP dialect.
    fn gnugo(&self) -> EngineProfileDto {
        let mut profile = self.0.profile.clone();
        profile.name = "controlled gnugo".into();
        profile.argv = GNU.map(String::from).into();
        profile.adapter = EngineAdapterSettings::GenericGtp(GenericGtpSettings {});
        self.0.catalog.upsert(SavedEngineProfile {
            profile_id: "gnugo".into(),
            profile: profile.clone(),
        });
        profile
    }
    fn sides_request(&self, black_gtp: bool, white_gtp: bool) -> (HumanMatchStartDto, [EngineProfileDto; 2]) {
        let gnugo = self.gnugo();
        let mut request = self.request(20, false);
        let pick = |gtp: bool| {
            if gtp {
                ("gnugo", gnugo.clone())
            } else {
                ("engine", self.0.profile.clone())
            }
        };
        let (black_id, black) = pick(black_gtp);
        let (white_id, white) = pick(white_gtp);
        request.settings.pk_black.profile_id = Some(black_id.into());
        request.settings.pk_white.profile_id = Some(white_id.into());
        (request, [black, white])
    }
    fn start_sides(&self, black_gtp: bool, white_gtp: bool) -> MatchUpdateDto {
        let (request, profiles) = self.sides_request(black_gtp, white_gtp);
        self.0
            .state
            .start_pk_match(
                &self.0.manager,
                &self.0.preferences,
                &self.0.path,
                request,
                profiles,
                |_| {},
            )
            .unwrap()
    }
    fn take_gtp(&self, pid: u32) -> (MatchTurnDto, GameMoveHandle, serde_json::Value) {
        let (token, handle) = self
            .0
            .state
            .take_match_engine_turn(&self.0.manager)
            .unwrap()
            .unwrap();
        let path = self.0.directory.join(format!("genmove-{pid}"));
        wait_for(|| {
            std::fs::read_to_string(&path)
                .ok()
                .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok())
                .is_some()
        });
        (
            token,
            handle,
            serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap(),
        )
    }
    fn reply_gtp(&self, pid: u32, value: &str) {
        std::fs::write(self.0.directory.join(format!("reply-gtp-{pid}")), value).unwrap();
    }
    fn argv(&self, pid: u32) -> Vec<String> {
        serde_json::from_str(&std::fs::read_to_string(self.0.directory.join(format!("argv-{pid}"))).unwrap())
            .unwrap()
    }
    fn session(&self) -> String {
        self.0
            .state
            .human_match_snapshot()
            .match_state
            .session_id
            .unwrap()
    }
}

fn alive(pid: u32) -> bool {
    PathBuf::from(format!("/proc/{pid}")).exists()
}

fn synced(pending: &serde_json::Value) -> Vec<String> {
    pending["synced"]
        .as_array()
        .unwrap()
        .iter()
        .map(|command| command.as_str().unwrap().to_owned())
        .collect()
}

#[test]
fn mixed_pk_reaps_only_the_cancelled_gtp_side_and_resume_rebuilds_the_original_snapshot() {
    let rig = PkRig::new();
    let started = rig.start_sides(false, true);
    let runs = started.match_state.pk_runs.clone().unwrap();
    assert_eq!(runs[0].adapter_kind, EngineBackend::KataGoAnalysis);
    assert_eq!(runs[1].adapter_kind, EngineBackend::GenericGtp);
    let pids = rig.pids();
    assert_eq!(pids.len(), 2);
    assert_eq!(rig.argv(pids[1]), GNU);
    let (_, black) = rig.play("A5");
    assert_eq!(black["pid"], pids[0]);
    assert_eq!(black["request"]["maxVisits"], 11);
    let (old_token, old_handle, pending) = rig.take_gtp(pids[1]);
    let old_job = old_handle.identity.job_id.clone();
    assert_eq!(pending["command"], "genmove W");
    assert_eq!(
        synced(&pending)[..4],
        ["boardsize 5", "clear_board", "komi 7.5", "play B A5"]
    );
    let defaults = std::fs::read(&rig.0.path).unwrap();

    let paused = rig.pause();
    assert_eq!(paused.match_state.phase, MatchPhaseDto::Paused);
    assert!(!paused.match_state.pause_pending);
    assert_eq!(paused.match_state.rebuild_sides, vec![PlayerColor::White]);
    assert_eq!(paused.match_state.committed_moves, 1);
    assert_eq!(paused.match_state.to_play, Some(PlayerColor::White));
    assert_eq!(paused.match_state.pk_runs.as_ref(), Some(&runs));
    assert_eq!(
        old_handle.wait().unwrap_err().kind,
        EngineFailureKind::Cancellation
    );
    rig.assert_reaped(&pids[1..]);
    assert!(alive(pids[0]), "the idle KataGo side must not be reclaimed");
    assert_exclusive(&rig.0);

    // A saved-profile edit during Pause must not reach the rebuilt process.
    let mut edited = rig.0.catalog.get("gnugo").unwrap();
    edited.profile.program = "/nonexistent/gnugo".into();
    edited.profile.argv = vec!["--mode".into(), "gtp".into(), "--level".into(), "10".into()];
    rig.0.catalog.upsert(edited);
    let id = rig.session();
    let resumed = rig.resume(&id);
    assert_eq!(resumed.match_state.phase, MatchPhaseDto::Playing);
    assert!(resumed.match_state.rebuild_sides.is_empty() && !resumed.match_state.resume_pending);
    assert_eq!(resumed.match_state.committed_moves, 1);
    assert_eq!(resumed.match_state.to_play, Some(PlayerColor::White));
    assert_eq!(resumed.current, paused.current);
    let rebuilt = resumed.match_state.pk_runs.clone().unwrap();
    assert_eq!(rebuilt[0], runs[0]);
    assert_ne!(rebuilt[1].run_id, runs[1].run_id);
    assert_eq!(rebuilt[1].profile_snapshot, runs[1].profile_snapshot);
    let after = rig.pids();
    assert_eq!(after.len(), 3);
    assert_eq!(rig.argv(after[2]), GNU);
    assert_eq!(
        rig.resume(&id),
        resumed,
        "Resume of a playing session is not a second attempt"
    );
    assert_eq!(rig.pids().len(), 3);

    let (token, handle, pending) = rig.take_gtp(after[2]);
    let commands = synced(&pending);
    assert_eq!(
        commands[..4],
        ["boardsize 5", "clear_board", "komi 7.5", "play B A5"]
    );
    // A new job gets the full side budget (4000 ms → three whole GTP seconds), not a remainder.
    assert_eq!(commands[4], "time_settings 0 3 1");
    assert_ne!(handle.identity.job_id, old_job);
    assert_eq!(handle.identity.run_id, rebuilt[1].run_id);
    let before_late = rig.0.state.human_match_snapshot();
    assert_eq!(
        rig.0.state.accept_match_engine_result(
            &rig.0.manager,
            &old_token,
            Err(match_failure(EngineFailureKind::Cancellation, "late"))
        ),
        before_late
    );
    rig.reply_gtp(after[2], "B5");
    let update = rig
        .0
        .state
        .accept_match_engine_result(&rig.0.manager, &token, handle.wait());
    assert_eq!(update.match_state.committed_moves, 2);
    let (_, black) = rig.play("C5");
    assert_eq!(black["pid"], pids[0]);
    assert_eq!(
        black["request"]["moves"],
        serde_json::json!([["B", "A5"], ["W", "B5"]])
    );
    assert!(rig.0.state.serialize().unwrap().contains(";B[aa];W[ba];B[ca]"));
    assert_eq!(
        std::fs::read(&rig.0.path).unwrap(),
        defaults,
        "Resume must not write defaults"
    );
    assert!(!rig.0.stop().match_state.resources_held);
    rig.assert_reaped(&rig.pids());
}

#[test]
fn gtp_pair_rebuilds_whichever_side_pause_interrupted() {
    let rig = PkRig::new();
    let started = rig.start_sides(true, true);
    let runs = started.match_state.pk_runs.unwrap();
    let pids = rig.pids();
    let id = rig.session();

    let (_, handle, _) = rig.take_gtp(pids[0]);
    assert_eq!(rig.pause().match_state.rebuild_sides, vec![PlayerColor::Black]);
    assert!(handle.wait().is_err());
    rig.assert_reaped(&pids[..1]);
    assert!(alive(pids[1]));
    let first = rig.resume(&id).match_state.pk_runs.unwrap();
    assert_ne!(first[0].run_id, runs[0].run_id);
    assert_eq!(first[1], runs[1]);
    let black_pid = rig.pids()[2];
    let (token, handle, _) = rig.take_gtp(black_pid);
    rig.reply_gtp(black_pid, "A5");
    assert_eq!(
        rig.0
            .state
            .accept_match_engine_result(&rig.0.manager, &token, handle.wait())
            .match_state
            .committed_moves,
        1
    );

    let (_, handle, pending) = rig.take_gtp(pids[1]);
    assert!(synced(&pending).contains(&"play B A5".to_owned()));
    assert_eq!(rig.pause().match_state.rebuild_sides, vec![PlayerColor::White]);
    assert!(handle.wait().is_err());
    rig.assert_reaped(&pids[1..]);
    assert!(alive(black_pid), "the rebuilt Black run keeps its identity");
    let second = rig.resume(&id).match_state.pk_runs.unwrap();
    assert_eq!(second[0], first[0]);
    assert_ne!(second[1].run_id, runs[1].run_id);
    let white_pid = rig.pids()[3];
    let (token, handle, _) = rig.take_gtp(white_pid);
    rig.reply_gtp(white_pid, "B5");
    let update = rig
        .0
        .state
        .accept_match_engine_result(&rig.0.manager, &token, handle.wait());
    assert_eq!(update.match_state.committed_moves, 2);
    assert!(rig.0.state.serialize().unwrap().contains(";B[aa];W[ba]"));
    rig.0.stop();
    rig.assert_reaped(&rig.pids());
}

#[test]
fn failed_rebuild_ends_at_committed_review_without_retry_or_default_write() {
    let rig = PkRig::new();
    rig.start_sides(false, true);
    rig.play("A5");
    let (_, handle, _) = rig.take_gtp(rig.pids()[1]);
    rig.pause();
    assert!(handle.wait().is_err());
    let before = rig.0.state.serialize().unwrap();
    let defaults = std::fs::read(&rig.0.path).unwrap();
    std::fs::write(rig.0.directory.join("startup-3"), "fail").unwrap();
    let id = rig.session();
    let failed = rig.resume(&id).match_state;
    assert_eq!(failed.phase, MatchPhaseDto::Ending);
    assert_eq!(failed.end, Some(MatchEndDto::Failed));
    assert_eq!(failed.failed_side, Some(PlayerColor::White));
    let failure = failed.failure.unwrap();
    assert_eq!(failure.profile_id.as_deref(), Some("gnugo"));
    assert_eq!(failure.kind, EngineFailureKind::ProcessExit);
    assert!(failed.resources_held && !failed.resume_pending && failed.rebuild_sides.is_empty());
    assert!(rig
        .0
        .state
        .take_match_engine_turn(&rig.0.manager)
        .unwrap()
        .is_none());
    assert!(rig.0.state.resume_pk_match(&rig.0.manager, &id, |_| {}).is_err());
    assert_eq!(rig.0.state.serialize().unwrap(), before);
    assert_eq!(std::fs::read(&rig.0.path).unwrap(), defaults);
    let stopped = rig.0.stop();
    assert!(!stopped.match_state.resources_held);
    assert_eq!(stopped.match_state.failed_side, Some(PlayerColor::White));
    assert_eq!(rig.pids().len(), 3, "a failed rebuild is never retried");
    rig.assert_reaped(&rig.pids());
    assert_eq!(rig.0.manager.match_reservation_owner(), None);
}

#[test]
fn repeated_resume_joins_one_attempt_and_stop_or_exit_seal_the_rebuild() {
    for exit in [false, true] {
        let rig = PkRig::new();
        rig.start_sides(false, true);
        rig.play("A5");
        let (_, handle, _) = rig.take_gtp(rig.pids()[1]);
        rig.pause();
        assert!(handle.wait().is_err());
        let before = rig.0.state.serialize().unwrap();
        std::fs::write(rig.0.directory.join("startup-3"), "hold").unwrap();
        let id = rig.session();
        std::thread::scope(|scope| {
            let first = scope.spawn(|| rig.0.state.resume_pk_match(&rig.0.manager, &id, |_| {}));
            wait_for(|| rig.pids().len() == 3);
            wait_for(|| rig.0.state.human_match_snapshot().match_state.resume_pending);
            let repeated = rig.resume(&id).match_state;
            assert!(
                repeated.resume_pending && repeated.phase == MatchPhaseDto::Paused,
                "{exit}"
            );
            assert_exclusive(&rig.0);
            if exit {
                let departure = match rig.0.state.prepare_exit().unwrap() {
                    DocumentDepartureAdmissionDto::NeedsDecision { departure_id }
                    | DocumentDepartureAdmissionDto::Ready { departure_id } => departure_id,
                };
                rig.0.state.begin_protected_commit(departure, &[]).unwrap();
                let path = rig.0.state.human_match_snapshot().current.unwrap().selected_path;
                rig.0
                    .state
                    .begin_application_teardown(
                        departure,
                        path,
                        ApplicationExitDispositionDto::ExplicitDiscard,
                    )
                    .unwrap();
                // Exit sealed the owner first; Ready now arrives late and must not revive the session.
                std::fs::write(rig.0.directory.join("release-startup-3"), "go").unwrap();
                let late = first.join().unwrap().unwrap().match_state;
                assert_eq!(late.phase, MatchPhaseDto::Ending);
                assert_eq!(late.end, Some(MatchEndDto::Stopped));
                assert!(rig
                    .0
                    .state
                    .take_match_engine_turn(&rig.0.manager)
                    .unwrap()
                    .is_none());
                rig.0.manager.teardown().unwrap();
            } else {
                let stopped = rig
                    .0
                    .state
                    .stop_human_match(&rig.0.manager, &id)
                    .unwrap()
                    .match_state;
                assert_eq!(stopped.phase, MatchPhaseDto::Idle);
                assert!(!stopped.resources_held);
                // The rebuild worker returns once Stop seals it, possibly before Stop publishes Idle.
                let late = first.join().unwrap().unwrap().match_state;
                assert!(
                    matches!(late.phase, MatchPhaseDto::Ending | MatchPhaseDto::Idle),
                    "{late:?}"
                );
                assert_eq!(
                    rig.0.state.human_match_snapshot().match_state.phase,
                    MatchPhaseDto::Idle
                );
                std::fs::write(rig.0.directory.join("release-startup-3"), "go").unwrap();
            }
        });
        assert_eq!(
            rig.pids().len(),
            3,
            "repeated Resume spawned another rebuild ({exit})"
        );
        rig.assert_reaped(&rig.pids());
        assert_eq!(rig.0.manager.match_reservation_owner(), None);
        assert_eq!(rig.0.state.serialize().unwrap(), before);
        assert!(!rig.0.state.serialize().unwrap().contains("RE["));
    }
}

#[test]
fn unacknowledged_kata_drain_in_mixed_pk_keeps_occupancy_and_never_reclaims_idle_gtp() {
    let rig = PkRig::new();
    rig.start_sides(false, true);
    let pids = rig.pids();
    std::fs::write(rig.0.directory.join("hold-cancel"), "hold").unwrap();
    let (_, handle, _) = rig.take();
    let failed = rig.pause().match_state;
    assert_eq!(failed.end, Some(MatchEndDto::Failed));
    assert_eq!(failed.failed_side, Some(PlayerColor::Black));
    assert_eq!(failed.failure.unwrap().kind, EngineFailureKind::Timeout);
    assert!(failed.resources_held && failed.rebuild_sides.is_empty());
    assert!(handle.wait().is_err());
    assert!(
        alive(pids[1]),
        "the idle GTP side is not reclaimed for another side's failure"
    );
    assert!(rig.0.manager.reserve_match("another-session").is_err());
    assert!(!rig.0.stop().match_state.resources_held);
    rig.assert_reaped(&pids);
}

#[test]
fn gtp_side_start_failures_roll_back_only_their_candidates() {
    for case in ["startup", "position"] {
        let rig = PkRig::new();
        let old_run = rig.0.foreground();
        let old_pid = rig.pids()[0];
        let before = rig.0.state.human_match_snapshot().current.unwrap();
        let sgf = rig.0.state.serialize().unwrap();
        app_preferences::save_to_path(&rig.0.path, app_preferences::default_app_preferences()).unwrap();
        let bytes = std::fs::read(&rig.0.path).unwrap();
        let (mut request, profiles) = rig.sides_request(false, true);
        if case == "startup" {
            std::fs::write(rig.0.directory.join("startup-3"), "fail").unwrap();
        }
        // Qualified GNU Go only admits Chinese KGS positional superko; KataGo accepts Chinese.
        else {
            request.settings.rules = Some(ExactRulesDto::Chinese);
        }
        let error = rig
            .0
            .state
            .start_pk_match(
                &rig.0.manager,
                &rig.0.preferences,
                &rig.0.path,
                request,
                profiles,
                |_| {},
            )
            .unwrap_err();
        if case == "position" {
            assert_eq!(error.kind, EngineFailureKind::UnsupportedCapability);
        }
        let state = rig.0.state.human_match_snapshot();
        assert_eq!(state.match_state.failed_side, Some(PlayerColor::White), "{case}");
        assert_eq!(state.current.unwrap(), before);
        assert!(!state.match_state.resources_held);
        assert_eq!(rig.0.state.serialize().unwrap(), sgf);
        assert_eq!(std::fs::read(&rig.0.path).unwrap(), bytes);
        assert!(
            matches!(rig.0.manager.snapshot().lifecycle, ForegroundEngineLifecycleDto::Ready { run } if run.run_id == old_run)
        );
        assert!(alive(old_pid));
        rig.assert_reaped(&rig.pids()[1..]);
    }
}

#[test]
fn gtp_resident_runtime_failures_end_the_whole_match_without_rebuild_or_result() {
    for cause in ["eof", "protocol", "timeout", "idle_exit"] {
        let rig = PkRig::new();
        let (mut request, profiles) = rig.sides_request(false, true);
        request.settings.pk_white.deadline_ms = 1500;
        rig.0
            .state
            .start_pk_match(
                &rig.0.manager,
                &rig.0.preferences,
                &rig.0.path,
                request,
                profiles,
                |_| {},
            )
            .unwrap();
        let pids = rig.pids();
        rig.play("A5");
        let (update, before) = if cause == "idle_exit" {
            let (token, handle, _) = rig.take_gtp(pids[1]);
            rig.reply_gtp(pids[1], "B5");
            rig.0
                .state
                .accept_match_engine_result(&rig.0.manager, &token, handle.wait());
            let before = rig.0.state.serialize().unwrap();
            // White GTP is a match resident, idle while Black's KataGo move is pending.
            let events = rig.0.manager.subscribe();
            let (token, handle, _) = rig.take();
            std::fs::write(rig.0.directory.join(format!("exit-{}", pids[1])), "exit").unwrap();
            let until = Instant::now() + Duration::from_secs(5);
            let observed = loop {
                assert!(Instant::now() < until, "idle GTP exit was not published");
                if let Ok(ForegroundEngineEventDto::Failure { failure }) =
                    events.recv_timeout(Duration::from_millis(50))
                {
                    if let Some(update) = rig.0.state.observe_match_failure(&failure) {
                        break update;
                    }
                }
            };
            assert_eq!(
                rig.0
                    .state
                    .accept_match_engine_result(&rig.0.manager, &token, handle.wait()),
                observed
            );
            (observed, before)
        } else {
            let before = rig.0.state.serialize().unwrap();
            let (token, handle, _) = rig.take_gtp(pids[1]);
            if cause != "timeout" {
                rig.reply_gtp(pids[1], cause);
            }
            (
                rig.0
                    .state
                    .accept_match_engine_result(&rig.0.manager, &token, handle.wait()),
                before,
            )
        };
        let state = update.match_state;
        assert_eq!(state.end, Some(MatchEndDto::Failed), "{cause}");
        assert_eq!(state.failed_side, Some(PlayerColor::White), "{cause}");
        let failure = state.failure.unwrap();
        assert_eq!(failure.profile_id.as_deref(), Some("gnugo"), "{cause}");
        assert_eq!(
            failure.kind,
            match cause {
                "protocol" => EngineFailureKind::Command,
                "timeout" => EngineFailureKind::Timeout,
                _ => EngineFailureKind::ProcessExit,
            },
            "{cause}: {failure:?}"
        );
        assert!(state.rebuild_sides.is_empty() && state.resources_held, "{cause}");
        assert!(rig
            .0
            .state
            .resume_pk_match(&rig.0.manager, state.session_id.as_deref().unwrap(), |_| {})
            .is_err());
        assert_eq!(rig.0.state.serialize().unwrap(), before, "{cause}");
        assert!(!before.contains("RE["));
        assert!(!rig.0.stop().match_state.resources_held, "{cause}");
        assert_eq!(
            rig.pids().len(),
            2,
            "{cause}: an unexpected failure must not rebuild"
        );
        rig.assert_reaped(&rig.pids());
        assert_eq!(rig.0.manager.match_reservation_owner(), None);
    }
}

#[test]
fn unconfirmed_gtp_reap_keeps_occupancy_and_never_authorizes_rebuild() {
    // A zero cleanup budget leaves each kill unobserved before its deadline.
    let rig = PkRig::with_config(ForegroundEngineConfig {
        stop_drain_timeout: Duration::ZERO,
        ..ForegroundEngineConfig::for_tests()
    });
    rig.start_sides(true, true);
    let pids = rig.pids();
    let id = rig.session();
    let (_, handle, _) = rig.take_gtp(pids[0]);
    let failed = rig.pause().match_state;
    assert_eq!(failed.end, Some(MatchEndDto::Failed));
    assert_eq!(failed.failed_side, Some(PlayerColor::Black));
    assert_eq!(failed.failure.as_ref().unwrap().kind, EngineFailureKind::Timeout);
    assert!(failed.rebuild_sides.is_empty() && failed.resources_held);
    assert!(handle.wait().is_err());
    assert!(rig.0.state.resume_pk_match(&rig.0.manager, &id, |_| {}).is_err());
    assert!(rig.0.manager.reserve_match("another-session").is_err());
    assert!(rig.0.manager.assert_profile_deletable("gnugo").is_err());
    let held = rig.0.stop().match_state;
    assert!(held.resources_held, "unconfirmed cleanup must keep occupancy");
    assert_eq!(held.phase, MatchPhaseDto::Error);
    // The idle White child was killed but its exit was not observed: it stays owned, unreaped.
    assert!(alive(pids[1]));
    assert!(rig.0.manager.reserve_match("another-session").is_err());
    // Both children were killed; once they are observable as exited, an explicit retry releases them.
    wait_for(|| {
        pids.iter().all(|pid| {
            std::fs::read_to_string(format!("/proc/{pid}/stat"))
                .map_or(true, |stat| stat.split_whitespace().nth(2) == Some("Z"))
        })
    });
    assert!(!rig.0.stop().match_state.resources_held);
    assert_eq!(rig.pids().len(), 2, "no rebuild was attempted");
    rig.assert_reaped(&pids);
    assert_eq!(rig.0.manager.match_reservation_owner(), None);
}
