use super::*;

fn policy(rig: &Rig, value: MatchAnalysisPolicyDto) -> MatchUpdateDto {
    rig.state.human_match_analysis_policy(&rig.manager, rig.token(), value).unwrap()
}

fn finish(rig: &Rig, work: crate::current_game_state::human_match::MatchWork) -> usize {
    let mut frames = 0;
    let result = work.handle.wait_with_analysis(|job, frame| {
        let update = rig.state.publish_match_analysis(&work.turn, work.epoch, job, frame).unwrap();
        assert!(update.match_state.analysis.frame.is_some());
        frames += 1;
    });
    if work.human {
        rig.state.finish_match_analysis(&work.turn, work.epoch, result);
    } else {
        rig.state.accept_match_engine_result(&rig.manager, &work.turn, result);
    }
    frames
}

#[test]
fn each_role_policy_uses_only_its_turn_and_never_attaches_or_persists_analysis() {
    for value in [MatchAnalysisPolicyDto::Off, MatchAnalysisPolicyDto::HumanTurn,
        MatchAnalysisPolicyDto::EngineTurn, MatchAnalysisPolicyDto::Both] {
        let rig = Rig::new("normal");
        rig.start(PlayerColor::Black);
        let preferences = std::fs::read(&rig.path).unwrap();
        let document = rig.state.serialize().unwrap();
        policy(&rig, value);
        let work = rig.state.take_match_work(&rig.manager).unwrap();
        assert_eq!(work.is_some(), value.includes(true));
        if let Some(work) = work {
            assert!(work.human);
            assert!(finish(&rig, work) >= 1);
            assert!(rig.state.take_match_work(&rig.manager).unwrap().is_none());
            assert!(rig.state.human_match_snapshot().match_state.analysis.frame.is_some());
        }
        assert_eq!(rig.state.serialize().unwrap(), document);
        rig.human(MoveVertex::Pass);
        assert!(rig.state.human_match_snapshot().match_state.analysis.frame.is_none());
        let work = rig.state.take_match_work(&rig.manager).unwrap().unwrap();
        assert!(!work.human);
        assert_eq!(finish(&rig, work) > 0, value.includes(false));
        let snapshot = rig.state.human_match_snapshot();
        assert_eq!(snapshot.match_state.turn, 3);
        assert!(snapshot.match_state.analysis.frame.is_none());
        assert!(rig.state.serialize().unwrap().contains(";B[];W[db]"));
        assert!(!rig.state.serialize().unwrap().contains("RE["));
        assert_eq!(std::fs::read(&rig.path).unwrap(), preferences);
        rig.stop();
        let next = rig.start(PlayerColor::Black);
        assert_eq!(next.match_state.analysis.policy, MatchAnalysisPolicyDto::Off);
        assert!(next.match_state.analysis.frame.is_none());
        rig.stop();
    }
}

#[test]
fn policy_round_trip_seals_old_frames_and_drains_before_next_reserved_move() {
    let rig = Rig::new("analysis_hold");
    rig.start(PlayerColor::Black);
    policy(&rig, MatchAnalysisPolicyDto::Both);
    let work = rig.state.take_match_work(&rig.manager).unwrap().unwrap();
    let turn = work.turn.clone();
    let epoch = work.epoch;
    let job = work.handle.identity.clone();
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::scope(|scope| {
        let driver = scope.spawn(|| {
            let result = work.handle.wait_with_analysis(|job, frame| {
                rig.state.publish_match_analysis(&turn, epoch, job, frame.clone());
                tx.send(frame).unwrap();
            });
            rig.state.finish_match_analysis(&turn, epoch, result)
        });
        let frame = rx.recv_timeout(Duration::from_secs(3)).unwrap();
        assert!(rig.state.human_match_snapshot().match_state.analysis.frame.is_some());
        let mut wrong = job.clone();
        wrong.run_id = "retired-run".into();
        assert!(rig.state.publish_match_analysis(&turn, epoch, &wrong, frame.clone()).is_none());
        wrong = job.clone();
        wrong.job_id = uuid::Uuid::new_v4().to_string();
        assert!(rig.state.publish_match_analysis(&turn, epoch, &wrong, frame.clone()).is_none());
        let mut old_document = turn.clone();
        old_document.generation += 1;
        assert!(rig.state.publish_match_analysis(&old_document, epoch, &job, frame.clone()).is_none());
        let document = rig.state.serialize().unwrap();
        let off = policy(&rig, MatchAnalysisPolicyDto::Off);
        assert!(off.match_state.analysis.frame.is_none());
        assert!(rig.manager.snapshot().game_move_job.is_none());
        policy(&rig, MatchAnalysisPolicyDto::Both);
        assert!(rig.state.publish_match_analysis(&turn, epoch, &job, frame.clone()).is_none());
        assert_eq!(rig.state.serialize().unwrap(), document);
        assert_eq!(driver.join().unwrap().match_state.phase, MatchPhaseDto::Playing);
        policy(&rig, MatchAnalysisPolicyDto::EngineTurn);
        rig.human(MoveVertex::Pass);
        assert!(rig.state.publish_match_analysis(&turn, epoch, &job, frame.clone()).is_none());
        let engine = rig.state.take_match_work(&rig.manager).unwrap().unwrap();
        assert!(!engine.human);
        rig.stop();
        assert!(engine.handle.wait().is_err());
        assert!(rig.state.publish_match_analysis(&turn, epoch, &job, frame).is_none());
        assert!(!rig.state.serialize().unwrap().contains("RE["));
    });
}

#[test]
fn gtp_analysis_rejection_preserves_role_document_and_reservation() {
    let rig = Rig::with_adapter("normal", false);
    rig.start(PlayerColor::Black);
    let before = rig.state.serialize().unwrap();
    let failure = rig.state.human_match_analysis_policy(&rig.manager, rig.token(), MatchAnalysisPolicyDto::Both).unwrap_err();
    assert_eq!(failure.kind, EngineFailureKind::UnsupportedCapability);
    assert_eq!(rig.state.serialize().unwrap(), before);
    assert_eq!(rig.state.human_match_snapshot().match_state.turn, 1);
    assert!(rig.state.take_match_work(&rig.manager).unwrap().is_none());
    assert_eq!(rig.manager.match_reservation_owner().as_deref(), Some(rig.token().session_id.as_str()));
    rig.human(MoveVertex::Pass);
    assert_eq!(rig.engine().match_state.turn, 3);
    rig.stop();
}

fn finish_during_departure(mode: &str, protected: bool) -> (Rig, MatchUpdateDto) {
    let rig = Rig::new(mode);
    rig.start(PlayerColor::Black);
    policy(&rig, MatchAnalysisPolicyDto::Both);
    let work = rig.state.take_match_work(&rig.manager).unwrap().unwrap();
    let app_model::DocumentDepartureAdmissionDto::NeedsDecision { departure_id } = rig.state.prepare_exit().unwrap()
        else { panic!("New match must have a dirty-document exit decision"); };
    if protected { rig.state.begin_protected_commit(departure_id, &[]).unwrap(); }
    let result = work.handle.wait_with_analysis(|job, frame| {
        rig.state.publish_match_analysis(&work.turn, work.epoch, job, frame);
    });
    let update = rig.state.finish_match_analysis(&work.turn, work.epoch, result);
    if protected { rig.state.abort_protected_commit(departure_id, work.turn.node_path).unwrap(); }
    else { rig.state.cancel_replacement(departure_id).unwrap(); }
    assert!(rig.manager.snapshot().game_move_job.is_none());
    (rig, update)
}

#[test]
fn finite_analysis_completes_during_exit_decision_and_canceled_save() {
    for protected in [false, true] {
        let (rig, update) = finish_during_departure("normal", protected);
        assert!(update.match_state.job.is_none(), "completed work must retire its active identity");
        assert!(update.match_state.analysis.frame.is_some(), "final output survives a canceled departure");
        assert_eq!(update.match_state.phase, MatchPhaseDto::Playing);
        assert!(rig.state.take_match_work(&rig.manager).unwrap().is_none());
        rig.human(MoveVertex::Pass);
        assert_eq!(rig.engine().match_state.turn, 3);
        rig.stop();
    }
}

#[test]
fn analysis_timeout_during_canceled_departure_still_ends_the_match() {
    for protected in [false, true] {
        let (rig, update) = finish_during_departure("analysis_hold", protected);
        assert_eq!(update.match_state.phase, MatchPhaseDto::Ending);
        assert_eq!(update.match_state.failure.as_ref().unwrap().kind, EngineFailureKind::Timeout);
        assert!(update.match_state.job.is_none());
        assert!(update.match_state.analysis.frame.is_none());
        assert!(rig.state.human_match_action(&rig.manager, rig.token(), HumanMatchActionDto::Play { vertex: MoveVertex::Pass }).is_err());
        assert_eq!(rig.stop().match_state.phase, MatchPhaseDto::Error);
        assert!(rig.manager.match_reservation_owner().is_none());
    }
}

#[test]
fn continue_analysis_follows_new_mainline_and_cannot_leak_into_pk() {
    let rig = Rig::new("normal");
    rig.open(CONTINUE_SOURCE, vec![1]);
    let before = rig.state.human_match_snapshot().current.unwrap();
    rig.try_start(rig.continue_request(PlayerColor::White, true)).unwrap();
    policy(&rig, MatchAnalysisPolicyDto::Both);
    let work = rig.state.take_match_work(&rig.manager).unwrap().unwrap();
    let turn = work.turn.clone();
    let epoch = work.epoch;
    let job = work.handle.identity.clone();
    assert_ne!(turn.generation, before.generation);
    assert_eq!(turn.node_path.indices, vec![0, 0]);
    let mut observed = None;
    let result = work.handle.wait_with_analysis(|job, frame| {
        let update = rig.state.publish_match_analysis(&turn, epoch, job, frame.clone()).unwrap();
        assert_eq!(update.match_state.analysis.frame.as_ref().unwrap().turn, turn);
        observed = Some(frame);
    });
    rig.state.finish_match_analysis(&turn, epoch, result);
    let frame = observed.unwrap();
    let mut stale = turn.clone();
    stale.generation = before.generation;
    stale.node_path = before.selected_path;
    assert!(rig.state.publish_match_analysis(&stale, epoch, &job, frame.clone()).is_none());
    rig.human(MoveVertex::Pass);
    assert!(rig.state.publish_match_analysis(&turn, epoch, &job, frame.clone()).is_none());
    let engine = rig.state.take_match_work(&rig.manager).unwrap().unwrap();
    assert!(!engine.human);
    assert!(finish(&rig, engine) > 0);
    let current = rig.state.human_match_snapshot();
    assert_eq!(current.current.unwrap().selected_path.indices, vec![0, 0, 0, 0]);
    let sgf = rig.state.serialize().unwrap();
    assert!(sgf.contains(";W[];B[db]"), "{sgf}");
    assert!(sgf.contains("W[db]C[old reply]") && sgf.contains("C[branch]") && sgf.contains("C[old]"));
    assert!(!sgf.contains("RE[") && !sgf.contains("LZ["));
    rig.stop();
    assert!(rig.manager.snapshot().game_move_job.is_none());
    assert!(rig.manager.snapshot().selected_node_job.is_none());
    let mut request = rig.continue_request(PlayerColor::White, true);
    request.settings.pk_black = PkSideSettingsDto { profile_id: Some("engine".into()), deadline_ms: 3000, kata_max_visits: 8 };
    request.settings.pk_white = request.settings.pk_black.clone();
    let pk = rig.state.start_pk_match(&rig.manager, &rig.preferences, &rig.path, request,
        [rig.profile.clone(), rig.profile.clone()], |_| {}).unwrap();
    assert_eq!(pk.match_state.analysis, MatchAnalysisDto::default());
    assert!(rig.state.publish_match_analysis(&turn, epoch, &job, frame).is_none());
    assert!(rig.state.human_match_analysis_policy(&rig.manager, rig.token(), MatchAnalysisPolicyDto::Both).is_err());
    assert!(rig.manager.assert_profile_deletable("engine").is_err());
    let next = rig.state.take_match_work(&rig.manager).unwrap().unwrap();
    assert!(!next.human);
    assert_eq!(next.handle.identity.run_id, pk.match_state.pk_runs.unwrap()[1].run_id);
    rig.stop();
    assert!(rig.manager.match_reservation_owner().is_none());
}
