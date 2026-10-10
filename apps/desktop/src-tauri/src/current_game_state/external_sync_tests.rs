use super::*;
use app_model::{
    AnalysisFrameDto, AnalysisJobLaneDto, AnalysisJobOutcomeDto, CandidateMoveDto,
    DocumentDepartureAdmissionDto, EngineAdapterSettings, EngineFailureKind, EngineProfileDto, ExactRulesDto,
    ExternalSyncPhaseDto as Phase, GenericGtpSettings, HumanMatchStartDto, MatchDefaultsDto, MatchPhaseDto,
    MatchStartDto, PointDto, ProviderError, ProviderErrorKind, ProviderGameMetadata, ProviderImportRequest,
    ProviderImportResult, ProviderKind, ProviderRequestIdentityDto, YikeSyncPreferencesDto,
};
use engine_manager::{ForegroundEngineConfig, ForegroundEngineManager, InMemoryEngineProfileCatalog};
use provider_core::network::{NetworkOperation, NetworkState};
use provider_yike::sync::YikeSyncWork;
use std::{cell::Cell, fs, path::PathBuf, sync::Arc, time::Duration};

const ROOM: &str = "https://home.yikeweiqi.com/#/unite/owner-room";
const OTHER_ROOM: &str = "https://home.yikeweiqi.com/#/unite/other-owner-room";
const ORIGINAL: &str = "(;GM[1]FF[4]SZ[19]KM[7.5]PB[Local];B[aa];W[bb])";
const SOURCE: &str = "(;GM[1]FF[4]SZ[19]KM[7.5]PB[Source]C[root note];B[dd]C[move note];W[pp])";
const APPENDED: &str = "(;GM[1]FF[4]SZ[19]KM[7.5]PB[Source]C[root note];B[dd]C[move note];W[pp];B[qq])";
const REVISED: &str = "(;GM[1]FF[4]SZ[19]KM[7.5]PB[Source]C[root note];B[dd]C[move note];W[pq])";
const FINISHED: &str = "(;GM[1]FF[4]SZ[19]KM[7.5]PB[Source]RE[B+R]C[root note];B[dd]C[move note];W[pq])";

fn path(indices: &[u32]) -> NodePath {
    NodePath {
        indices: indices.to_vec(),
    }
}

fn current(state: &CurrentGameState) -> CurrentGameResultDto {
    state
        .human_match_snapshot()
        .current
        .expect("committed current document")
}

fn normalized(text: &str) -> String {
    CurrentSgfDocument::open(text).unwrap().serialize().unwrap()
}

// These fixtures exercise the owner seam, not HTTP or a live public room.
fn imported(locator: &str, text: &str) -> ProviderImportResult {
    provider_yike::import_payload(ProviderImportRequest {
        provider: ProviderKind::Yike,
        payload: text.into(),
        source_url: Some(locator.into()),
        source_id: None,
        metadata: ProviderGameMetadata {
            provider_status: Some("playing".into()),
            ..ProviderGameMetadata::default()
        },
    })
    .unwrap()
}

fn departure_id(admission: DocumentDepartureAdmissionDto) -> u64 {
    match admission {
        DocumentDepartureAdmissionDto::NeedsDecision { departure_id }
        | DocumentDepartureAdmissionDto::Ready { departure_id } => departure_id,
    }
}

fn install(state: &CurrentGameState, network: &NetworkState, locator: &str, text: &str) -> u64 {
    let id = state
        .begin_external_start(network, locator.into(), YikeSyncPreferencesDto::default())
        .unwrap();
    let candidate = state
        .prepare_external_candidate(id, imported(locator, text))
        .unwrap();
    let departure = departure_id(candidate.admission);
    state.begin_protected_commit(departure, &[]).unwrap();
    let outcome = state.commit_replacement(departure).unwrap();
    assert!(outcome.committed);
    assert!(!outcome.current.unwrap().dirty);
    assert_eq!(state.external_sync_snapshot().session_id, Some(id));
    id
}

fn poll(state: &CurrentGameState, network: &NetworkState) -> NetworkOperation {
    match state.next_external_poll(network) {
        YikeSyncWork::Fetch { operation, locator } => {
            assert_eq!(Some(locator), state.external_sync_snapshot().locator);
            operation
        }
        YikeSyncWork::Wait(_) => panic!("expected an immediately scheduled owner request"),
    }
}

fn schedule_poll(state: &CurrentGameState, network: &NetworkState) -> NetworkOperation {
    let preferences = state.external_sync_snapshot().preferences;
    state.update_external_preferences(preferences);
    poll(state, network)
}

fn complete(
    state: &CurrentGameState,
    network: &NetworkState,
    operation: &NetworkOperation,
    result: Result<ProviderImportResult, ProviderError>,
) -> app_model::ExternalSyncUpdateDto {
    state.complete_external_poll(operation.identity(), result, network.snapshot().policy_revision)
}

fn job_event(generation: u64, indices: &[u32], visits: u32) -> AnalysisJobEventDto {
    AnalysisJobEventDto {
        run_id: "external-sync-analysis-run".into(),
        job_id: format!("external-sync-analysis-{visits}"),
        lane: AnalysisJobLaneDto::SelectedNode,
        mode: AnalysisJobModeDto::Finite,
        generation,
        node_path: path(indices),
        outcome: AnalysisJobOutcomeDto::Completed,
        completed: None,
        expected: None,
        remaining: None,
        frame: Some(AnalysisFrameDto {
            job_id: uuid::Uuid::nil(),
            game_id: None,
            node_id: None,
            turn: 0,
            visits,
            winrate_black: 0.61,
            score_mean_black: Some(2.25),
            score_stdev: Some(0.5),
            candidates: vec![CandidateMoveDto {
                vertex: MoveVertex::Point(PointDto { x: 3, y: 3 }),
                visits,
                winrate_black: 0.61,
                score_mean_black: Some(2.25),
                policy_prior: Some(0.4),
                pv: vec![MoveVertex::Point(PointDto { x: 3, y: 3 })],
            }],
            ownership: None,
            policy: None,
        }),
        failure: None,
        current_game: None,
    }
}

fn unique_path(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "lizzieyzy-external-sync-{label}-{}.sgf",
        uuid::Uuid::new_v4()
    ))
}

fn assert_stale_completion(
    state: &CurrentGameState,
    identity: ProviderRequestIdentityDto,
    network: &NetworkState,
    locator: &str,
) {
    let before = current(state);
    let sync = state.external_sync_snapshot();
    let update = state.complete_external_poll(
        identity,
        Ok(imported(locator, APPENDED)),
        network.snapshot().policy_revision,
    );
    assert!(update.current.is_none());
    assert_eq!(update.sync, sync);
    assert_eq!(current(state), before);
}

#[test]
fn dirty_initial_start_cancel_preserves_document_selection_savepoint_and_no_session() {
    let state = CurrentGameState::default();
    let network = NetworkState::default();
    let opened = state.replace(ORIGINAL, Some("local-source.sgf".into())).unwrap();
    state
        .set_personal_comment(path(&[]), "unsaved local review".into())
        .unwrap();
    state.select_path(path(&[0]), opened.generation).unwrap();
    let before = current(&state);
    let text = state.serialize().unwrap();
    let identity = state.document_identity();
    let start = state
        .begin_external_start(&network, ROOM.into(), YikeSyncPreferencesDto::default())
        .unwrap();
    let request = state.external_start_request(start).unwrap().0;
    assert_eq!(state.external_sync_snapshot().phase, Phase::Starting);
    let candidate = state
        .prepare_external_candidate(start, imported(ROOM, SOURCE))
        .unwrap();
    let DocumentDepartureAdmissionDto::NeedsDecision { departure_id } = candidate.admission else {
        panic!("dirty first-frame install must use the single SGF-07 decision");
    };
    assert_eq!(current(&state), before);
    assert_eq!(
        state.commit_replacement(departure_id).unwrap_err().kind,
        CurrentGameErrorKind::DepartureBlocked
    );
    assert!(!state.cancel_replacement(departure_id).unwrap().committed);
    assert_eq!(current(&state), before);
    assert_eq!(state.serialize().unwrap(), text);
    assert_eq!(state.document_identity(), identity);
    assert_eq!(state.external_sync_snapshot().phase, Phase::Idle);
    assert!(state.external_sync_snapshot().starting_id.is_none());
    assert!(state.external_sync_snapshot().session_id.is_none());
    assert!(request.lease().check().is_err());
    assert!(state
        .prepare_external_candidate(start, imported(ROOM, SOURCE))
        .is_err());
    state
        .set_personal_comment(path(&[]), "editing resumed".into())
        .unwrap();
}

#[test]
fn malformed_initial_candidate_never_changes_document_or_enters_departure() {
    for (text, kind) in [
        ("not SGF", CurrentGameErrorKind::MalformedSgf),
        ("(;GM[1]FF[4]SZ[99])", CurrentGameErrorKind::UnsupportedBoardSize),
    ] {
        let state = CurrentGameState::default();
        let network = NetworkState::default();
        state.replace(ORIGINAL, None).unwrap();
        state
            .set_personal_comment(path(&[]), "dirty before malformed candidate".into())
            .unwrap();
        state.select_path(path(&[0]), current(&state).generation).unwrap();
        let before = current(&state);
        let identity = state.document_identity();
        let start = state
            .begin_external_start(&network, ROOM.into(), YikeSyncPreferencesDto::default())
            .unwrap();
        let mut candidate = imported(ROOM, SOURCE);
        candidate.sgf_text = text.into();
        assert_eq!(
            state
                .prepare_external_candidate(start, candidate)
                .unwrap_err()
                .kind,
            kind
        );
        assert_eq!(current(&state), before);
        assert_eq!(state.document_identity(), identity);
        assert!(state.external_sync_snapshot().session_id.is_none());
        // A malformed candidate never admitted a departure, so ordinary review still works.
        state
            .admit_selected_node(before.generation, &before.selected_path)
            .unwrap();
        assert_eq!(state.cancel_external_start(start).phase, Phase::Idle);
        state
            .set_personal_comment(path(&[]), "still editable".into())
            .unwrap();
    }
}

#[test]
fn first_frame_installs_one_clean_document_identity_and_identical_poll_is_a_noop() {
    let state = CurrentGameState::default();
    let network = NetworkState::default();
    state.replace(ORIGINAL, None).unwrap();
    let old_identity = state.document_identity();
    let old_generation = current(&state).generation;
    let start = state
        .begin_external_start(&network, ROOM.into(), YikeSyncPreferencesDto::default())
        .unwrap();
    let candidate = state
        .prepare_external_candidate(start, imported(ROOM, SOURCE))
        .unwrap();
    assert!(matches!(
        candidate.admission,
        DocumentDepartureAdmissionDto::Ready { .. }
    ));
    let departure = departure_id(candidate.admission);
    state.begin_protected_commit(departure, &[]).unwrap();
    let installed = state.commit_replacement(departure).unwrap().current.unwrap();
    assert!(!installed.dirty);
    assert!(!installed.can_undo);
    assert_eq!(installed.generation, old_generation + 1);
    assert_eq!(state.document_identity(), old_identity + 1);
    assert_eq!(state.serialize().unwrap(), normalized(SOURCE));
    let sync = state.external_sync_snapshot();
    assert_eq!(sync.phase, Phase::Syncing);
    assert_eq!(sync.document_identity, Some(old_identity + 1));
    assert_eq!(sync.source_tip, Some(path(&[0, 0])));
    let request = poll(&state, &network);
    assert_eq!(request.identity().document_identity, old_identity + 1);
    let update = complete(&state, &network, &request, Ok(imported(ROOM, SOURCE)));
    assert!(update.current.is_none());
    assert_eq!(current(&state), installed);
    assert_eq!(state.document_identity(), old_identity + 1);
    assert_eq!(update.sync.session_id, Some(start));
    assert!(matches!(
        state.next_external_poll(&network),
        YikeSyncWork::Wait(Some(_))
    ));
}

#[test]
fn same_move_count_content_and_terminal_result_revisions_dirty_without_replacing_document() {
    let state = CurrentGameState::default();
    let network = NetworkState::default();
    let session = install(&state, &network, ROOM, SOURCE);
    let identity = state.document_identity();
    let generation = current(&state).generation;
    assert_eq!(state.mainline_projection().unwrap().summary.move_count, 2);
    let request = poll(&state, &network);
    let revised = complete(&state, &network, &request, Ok(imported(ROOM, REVISED)))
        .current
        .unwrap();
    assert!(revised.dirty);
    assert_eq!(revised.generation, generation + 1);
    assert_eq!(state.document_identity(), identity);
    assert_eq!(state.serialize().unwrap(), normalized(REVISED));
    assert_eq!(state.mainline_projection().unwrap().summary.move_count, 2);
    let request = schedule_poll(&state, &network);
    let mut result = imported(ROOM, FINISHED);
    result.metadata.provider_status = Some("finished".into());
    let update = complete(&state, &network, &request, Ok(result));
    assert!(update.current.unwrap().dirty);
    assert_eq!(state.document_identity(), identity);
    assert_eq!(update.sync.document_identity, Some(identity));
    assert_eq!(update.sync.session_id, Some(session));
    assert_eq!(update.sync.source_status.as_deref(), Some("finished"));
    let summary = state.mainline_projection().unwrap().summary;
    assert_eq!(summary.move_count, 2);
    assert_eq!(summary.result.as_deref(), Some("B+R"));
    assert_eq!(state.serialize().unwrap(), normalized(FINISHED));
    let unchanged = current(&state);
    let request = schedule_poll(&state, &network);
    let mut result = imported(ROOM, FINISHED);
    result.metadata.provider_status = Some("finished".into());
    assert!(complete(&state, &network, &request, Ok(result)).current.is_none());
    assert_eq!(current(&state), unchanged);
}

#[test]
fn external_occupancy_allows_browsing_and_exact_analysis_but_rejects_edits_trial_and_match() {
    let state = CurrentGameState::default();
    let network = NetworkState::default();
    install(&state, &network, ROOM, SOURCE);
    let selected = state.select_path(path(&[0]), current(&state).generation).unwrap();
    let admitted = state
        .admit_selected_node(selected.generation, &selected.selected_path)
        .unwrap();
    assert_eq!(admitted.0.path, path(&[0]));
    let attached = state
        .attach_from_job_event(&job_event(selected.generation, &[0], 400))
        .unwrap();
    assert!(attached.dirty);
    assert_eq!(attached.snapshot.primary_analysis.unwrap().visits, 400);
    let before = current(&state);
    let text = state.serialize().unwrap();
    for error in [
        state
            .play(path(&[0]), MoveVertex::Point(PointDto { x: 1, y: 1 }))
            .unwrap_err(),
        state
            .set_personal_comment(path(&[0]), "forbidden".into())
            .unwrap_err(),
        state
            .set_metadata(before.generation, "forbidden".into(), "forbidden".into(), 6.5)
            .unwrap_err(),
        state
            .edit_markup(
                path(&[0]),
                before.generation,
                app_model::SgfMarkupActionDto::Clear,
            )
            .unwrap_err(),
        state.remove_variation(path(&[0]), before.generation).unwrap_err(),
        state.promote_to_main(path(&[0]), before.generation).unwrap_err(),
        state
            .apply_root_setup(before.generation, vec![], PlayerColor::Black)
            .unwrap_err(),
        state
            .convert_to_root_setup(before.generation, path(&[0]))
            .unwrap_err(),
        state.undo(before.generation).unwrap_err(),
        state.redo(before.generation).unwrap_err(),
        state.prepare_replacement(ORIGINAL, None).unwrap_err(),
    ] {
        assert_eq!(error.kind, CurrentGameErrorKind::DepartureBlocked);
    }
    assert!(state.enter_trial().unwrap_err().contains("External sync"));
    let manager = ForegroundEngineManager::new(
        Arc::new(InMemoryEngineProfileCatalog::new()),
        ForegroundEngineConfig::for_tests(),
    );
    let preferences = crate::continuous_analysis::PreferencesState::default();
    let preferences_path = unique_path("unwritten-match-preferences");
    let profile = EngineProfileDto {
        name: "must never start".into(),
        program: "external-sync-guard-unused-engine".into(),
        argv: vec![],
        working_dir: None,
        adapter: EngineAdapterSettings::GenericGtp(GenericGtpSettings::default()),
    };
    let request = HumanMatchStartDto {
        settings: MatchDefaultsDto {
            rules: Some(ExactRulesDto::ChineseKgs),
            ..MatchDefaultsDto::default()
        },
        generation: before.generation,
        snapshot_seq: before.snapshot_seq,
        start: MatchStartDto::New {
            discard_confirmed: true,
        },
    };
    let published = Cell::new(false);
    let error = state
        .start_human_match(
            &manager,
            &preferences,
            &preferences_path,
            request.clone(),
            profile.clone(),
            |_| published.set(true),
        )
        .unwrap_err();
    assert_eq!(error.kind, EngineFailureKind::InvalidState);
    assert!(error.message.contains("External sync"));
    let error = state
        .start_pk_match(
            &manager,
            &preferences,
            &preferences_path,
            request,
            [profile.clone(), profile],
            |_| published.set(true),
        )
        .unwrap_err();
    assert_eq!(error.kind, EngineFailureKind::InvalidState);
    assert!(error.message.contains("External sync"));
    assert!(!published.get());
    assert!(!preferences_path.exists());
    assert_eq!(
        state.human_match_snapshot().match_state.phase,
        MatchPhaseDto::Idle
    );
    assert!(matches!(
        manager.snapshot().lifecycle,
        app_model::ForegroundEngineLifecycleDto::NoEngine { .. }
    ));
    assert_eq!(current(&state), before);
    assert_eq!(state.serialize().unwrap(), text);
}

#[test]
fn append_preserves_valid_old_attachments_and_cursor_but_rejects_late_old_generation() {
    let state = CurrentGameState::default();
    let network = NetworkState::default();
    install(&state, &network, ROOM, SOURCE);
    let generation = current(&state).generation;
    state.select_path(path(&[0]), generation).unwrap();
    state
        .attach_from_job_event(&job_event(generation, &[], 400))
        .unwrap();
    state
        .attach_from_job_event(&job_event(generation, &[0], 900))
        .unwrap();
    let request = poll(&state, &network);
    let update = complete(&state, &network, &request, Ok(imported(ROOM, APPENDED)));
    let appended = update.current.unwrap();
    assert_eq!(appended.selected_path, path(&[0]));
    assert_eq!(update.sync.source_tip, Some(path(&[0, 0, 0])));
    assert_eq!(appended.snapshot.personal_comment, "move note");
    assert_eq!(appended.snapshot.primary_analysis.unwrap().visits, 900);
    let root = state.select_path(path(&[]), appended.generation).unwrap();
    assert_eq!(root.snapshot.personal_comment, "root note");
    assert_eq!(root.snapshot.primary_analysis.unwrap().visits, 400);
    let before = current(&state);
    let text = state.serialize().unwrap();
    assert!(state
        .attach_from_job_event(&job_event(generation, &[], 1200))
        .is_none());
    assert_eq!(current(&state), before);
    assert_eq!(state.serialize().unwrap(), text);
    state.admit_selected_node(before.generation, &path(&[0])).unwrap();
    assert_eq!(
        state
            .attach_from_job_event(&job_event(before.generation, &[0], 1500))
            .unwrap()
            .snapshot
            .primary_analysis
            .unwrap()
            .visits,
        1500
    );
}

#[test]
fn rewritten_source_position_drops_invalid_attachment_and_relocates_to_proven_ancestor() {
    let state = CurrentGameState::default();
    let network = NetworkState::default();
    install(&state, &network, ROOM, SOURCE);
    let generation = current(&state).generation;
    state
        .attach_from_job_event(&job_event(generation, &[0], 400))
        .unwrap();
    state
        .attach_from_job_event(&job_event(generation, &[0, 0], 900))
        .unwrap();
    state.select_path(path(&[0, 0]), generation).unwrap();
    let request = poll(&state, &network);
    let updated = complete(&state, &network, &request, Ok(imported(ROOM, REVISED)))
        .current
        .unwrap();
    assert_eq!(updated.selected_path, path(&[0]));
    assert_eq!(updated.snapshot.primary_analysis.unwrap().visits, 400);
    let rewritten = state.select_path(path(&[0, 0]), updated.generation).unwrap();
    assert!(rewritten.snapshot.primary_analysis.is_none());
    let before = current(&state);
    assert!(state
        .attach_from_job_event(&job_event(generation, &[0, 0], 1200))
        .is_none());
    assert_eq!(current(&state), before);
}

#[test]
fn jump_to_last_is_independent_from_source_following_and_mute_preference() {
    for jump_to_last in [false, true] {
        let state = CurrentGameState::default();
        let network = NetworkState::default();
        install(&state, &network, ROOM, SOURCE);
        state.select_path(path(&[0]), current(&state).generation).unwrap();
        let preferences = YikeSyncPreferencesDto {
            interval_seconds: 5,
            locator: Some(OTHER_ROOM.into()),
            jump_to_last,
            mute: true,
        };
        let snapshot = state.update_external_preferences(preferences);
        assert_eq!(snapshot.locator.as_deref(), Some(ROOM));
        assert!(snapshot.preferences.mute);
        let request = poll(&state, &network);
        let update = complete(&state, &network, &request, Ok(imported(ROOM, APPENDED)));
        assert_eq!(update.sync.source_tip, Some(path(&[0, 0, 0])));
        assert_eq!(
            update.current.unwrap().selected_path,
            if jump_to_last {
                path(&[0, 0, 0])
            } else {
                path(&[0])
            }
        );
        assert_eq!(state.mainline_projection().unwrap().summary.move_count, 3);
    }
}

#[test]
fn terminal_poll_errors_pause_last_good_readonly_without_automatic_reconnect() {
    for kind in [
        ProviderErrorKind::Timeout,
        ProviderErrorKind::NotFound,
        ProviderErrorKind::AuthenticationFailed,
    ] {
        let state = CurrentGameState::default();
        let network = NetworkState::default();
        let session = install(&state, &network, ROOM, SOURCE);
        state.select_path(path(&[0]), current(&state).generation).unwrap();
        let before = current(&state);
        let request = poll(&state, &network);
        let failure = ProviderError {
            kind,
            message: "owner seam terminal failure".into(),
        };
        let update = complete(&state, &network, &request, Err(failure.clone()));
        assert!(update.current.is_none());
        assert_eq!(update.sync.phase, Phase::ErrorPaused);
        assert_eq!(update.sync.failure, Some(failure));
        assert_eq!(update.sync.session_id, Some(session));
        assert_eq!(current(&state), before);
        assert_eq!(
            state
                .set_personal_comment(path(&[]), "forbidden while paused".into())
                .unwrap_err()
                .kind,
            CurrentGameErrorKind::DepartureBlocked
        );
        assert!(state.enter_trial().is_err());
        state.update_external_preferences(YikeSyncPreferencesDto::default());
        for _ in 0..3 {
            assert!(matches!(
                state.next_external_poll(&network),
                YikeSyncWork::Wait(None)
            ));
        }
        assert_eq!(state.external_sync_snapshot().phase, Phase::ErrorPaused);
        state
            .admit_selected_node(before.generation, &before.selected_path)
            .unwrap();
        let stopped = state.stop_external_sync(session).unwrap();
        assert_eq!(stopped.sync.phase, Phase::Idle);
        assert_eq!(stopped.current.unwrap(), before);
        assert_stale_completion(&state, request.identity(), &network, ROOM);
        state
            .set_personal_comment(path(&[]), "editing resumed after paused Stop".into())
            .unwrap();
    }
}

#[test]
fn malformed_or_wrong_source_poll_pauses_without_publishing_partial_candidate() {
    for wrong_source in [false, true] {
        let state = CurrentGameState::default();
        let network = NetworkState::default();
        install(&state, &network, ROOM, SOURCE);
        let before = current(&state);
        let request = poll(&state, &network);
        let mut candidate = imported(if wrong_source { OTHER_ROOM } else { ROOM }, APPENDED);
        if !wrong_source {
            candidate.sgf_text = "broken source response".into();
        }
        let update = complete(&state, &network, &request, Ok(candidate));
        assert!(update.current.is_none());
        assert_eq!(update.sync.phase, Phase::ErrorPaused);
        assert_eq!(
            update.sync.failure.unwrap().kind,
            if wrong_source {
                ProviderErrorKind::InvalidPayload
            } else {
                ProviderErrorKind::ParseFailed
            }
        );
        assert_eq!(current(&state), before);
        assert_eq!(state.serialize().unwrap(), normalized(SOURCE));
        assert!(matches!(
            state.next_external_poll(&network),
            YikeSyncWork::Wait(None)
        ));
    }
}

#[test]
fn explicit_retry_creates_new_request_and_old_request_cannot_publish() {
    let state = CurrentGameState::default();
    let network = NetworkState::default();
    let session = install(&state, &network, ROOM, SOURCE);
    let old = poll(&state, &network);
    complete(
        &state,
        &network,
        &old,
        Err(ProviderError {
            kind: ProviderErrorKind::Timeout,
            message: "retry budget exhausted upstream".into(),
        }),
    );
    let retry = state.retry_external_sync(session).unwrap();
    assert_eq!(retry.phase, Phase::Syncing);
    assert!(retry.failure.is_none());
    assert_stale_completion(&state, old.identity(), &network, ROOM);
    let new = poll(&state, &network);
    assert_ne!(new.identity().request_id, old.identity().request_id);
    assert_eq!(new.identity().document_identity, old.identity().document_identity);
    assert_eq!(new.retry_count(), 0);
    assert!(old.lease().check().is_err());
    assert_stale_completion(&state, old.identity(), &network, ROOM);
    let updated = complete(&state, &network, &new, Ok(imported(ROOM, APPENDED)));
    assert_eq!(updated.sync.phase, Phase::Syncing);
    assert!(updated.current.unwrap().dirty);
    assert_eq!(state.mainline_projection().unwrap().summary.move_count, 3);
}

#[test]
fn canceled_and_failed_switch_candidates_preserve_previous_session_and_dirty_document() {
    let state = CurrentGameState::default();
    let network = NetworkState::default();
    let session = install(&state, &network, ROOM, SOURCE);
    let generation = current(&state).generation;
    state.select_path(path(&[0]), generation).unwrap();
    state
        .attach_from_job_event(&job_event(generation, &[0], 400))
        .unwrap();
    let before = current(&state);
    let identity = state.document_identity();
    let old_poll = poll(&state, &network);
    let start = state
        .begin_external_start(&network, OTHER_ROOM.into(), YikeSyncPreferencesDto::default())
        .unwrap();
    let candidate_request = state.external_start_request(start).unwrap().0;
    let candidate = state
        .prepare_external_candidate(start, imported(OTHER_ROOM, ORIGINAL))
        .unwrap();
    assert!(matches!(
        candidate.admission,
        DocumentDepartureAdmissionDto::NeedsDecision { .. }
    ));
    assert!(
        !state
            .cancel_replacement(departure_id(candidate.admission))
            .unwrap()
            .committed
    );
    assert_eq!(current(&state), before);
    assert_eq!(state.document_identity(), identity);
    assert_eq!(state.external_sync_snapshot().session_id, Some(session));
    assert_eq!(state.external_sync_snapshot().locator.as_deref(), Some(ROOM));
    assert!(old_poll.lease().check().is_ok());
    assert!(candidate_request.lease().check().is_err());
    let start = state
        .begin_external_start(&network, OTHER_ROOM.into(), YikeSyncPreferencesDto::default())
        .unwrap();
    let mut malformed = imported(OTHER_ROOM, ORIGINAL);
    malformed.sgf_text = "invalid candidate".into();
    assert!(state.prepare_external_candidate(start, malformed).is_err());
    assert_eq!(current(&state), before);
    assert_eq!(state.external_sync_snapshot().session_id, Some(session));
    state.cancel_external_start(start);
    let start = state
        .begin_external_start(&network, OTHER_ROOM.into(), YikeSyncPreferencesDto::default())
        .unwrap();
    let candidate = state
        .prepare_external_candidate(start, imported(OTHER_ROOM, ORIGINAL))
        .unwrap();
    let departure = departure_id(candidate.admission);
    state.begin_protected_commit(departure, &[]).unwrap();
    network
        .commit(app_model::NetworkSettingsDto::default(), || Ok(()))
        .unwrap();
    let failed = state.commit_replacement(departure).unwrap();
    assert!(!failed.committed);
    assert_eq!(current(&state), before);
    assert_eq!(state.document_identity(), identity);
    assert_eq!(state.external_sync_snapshot().session_id, Some(session));
    assert!(state.external_sync_snapshot().starting_id.is_none());
    state
        .admit_selected_node(before.generation, &before.selected_path)
        .unwrap();
    assert_eq!(
        state
            .set_personal_comment(path(&[]), "still externally owned".into())
            .unwrap_err()
            .kind,
        CurrentGameErrorKind::DepartureBlocked
    );
}

#[test]
fn successful_source_switch_seals_old_request_and_changes_document_identity_only_at_commit() {
    let state = CurrentGameState::default();
    let network = NetworkState::default();
    let old_session = install(&state, &network, ROOM, SOURCE);
    let identity = state.document_identity();
    let old_request = poll(&state, &network);
    let before = current(&state);
    let start = state
        .begin_external_start(&network, OTHER_ROOM.into(), YikeSyncPreferencesDto::default())
        .unwrap();
    assert_ne!(start, old_session);
    let candidate = state
        .prepare_external_candidate(start, imported(OTHER_ROOM, ORIGINAL))
        .unwrap();
    assert_eq!(current(&state), before);
    assert_eq!(state.document_identity(), identity);
    assert!(old_request.lease().check().is_ok());
    let departure = departure_id(candidate.admission);
    state.begin_protected_commit(departure, &[]).unwrap();
    assert!(state.commit_replacement(departure).unwrap().committed);
    assert_eq!(state.document_identity(), identity + 1);
    assert_eq!(state.external_sync_snapshot().session_id, Some(start));
    assert_eq!(
        state.external_sync_snapshot().locator.as_deref(),
        Some(OTHER_ROOM)
    );
    assert!(old_request.lease().check().is_err());
    assert_stale_completion(&state, old_request.identity(), &network, ROOM);
    assert!(state.stop_external_sync(old_session).is_err());
    assert_eq!(state.serialize().unwrap(), normalized(ORIGINAL));
    let new_request = poll(&state, &network);
    assert_eq!(new_request.identity().document_identity, identity + 1);
}

#[test]
fn provider_import_from_active_sync_preserves_cancel_and_seals_old_publication_at_commit() {
    for readboard in [false, true] {
        let state = CurrentGameState::default();
        let network = NetworkState::default();
        let session = if readboard {
            state.replace(LOCAL_3, None).unwrap();
            readboard_session(&state, 1)
        } else {
            install(&state, &network, ROOM, SOURCE)
        };
        let old_poll = (!readboard).then(|| poll(&state, &network));
        let generation = current(&state).generation;
        state
            .attach_from_job_event(&job_event(generation, &[], 400))
            .unwrap();
        let before = current(&state);
        assert!(before.dirty);
        let identity = state.document_identity();
        let request = network.begin(0, identity).unwrap();
        let lease = network.operation(&request).unwrap().lease();
        assert!(state
            .prepare_provider_replacement("invalid", lease.clone(), identity)
            .is_err());
        assert_eq!(current(&state), before);
        let admission = state
            .prepare_provider_replacement(ORIGINAL, lease.clone(), identity)
            .unwrap();
        assert!(matches!(
            admission,
            DocumentDepartureAdmissionDto::NeedsDecision { .. }
        ));
        state.cancel_replacement(departure_id(admission)).unwrap();
        assert_eq!(current(&state), before);
        assert_eq!(state.external_sync_snapshot().session_id, Some(session));
        assert!(state.prepare_replacement(ORIGINAL, None).is_err());
        let admission = state
            .prepare_provider_replacement(ORIGINAL, lease, identity)
            .unwrap();
        let departure = departure_id(admission);
        state.begin_protected_commit(departure, &[]).unwrap();
        assert!(state.commit_replacement(departure).unwrap().committed);
        assert_eq!(state.external_sync_snapshot().phase, Phase::Idle);
        assert_eq!(state.document_identity(), identity + 1);
        assert_eq!(state.serialize().unwrap(), normalized(ORIGINAL));
        if let Some(old) = old_poll {
            assert!(old.lease().check().is_err());
            assert_stale_completion(&state, old.identity(), &network, ROOM);
        } else {
            assert!(state
                .observe_readboard_frame(1, &board(["100", "040", "001"]))
                .is_none());
            assert_eq!(state.serialize().unwrap(), normalized(ORIGINAL));
        }
        state
            .set_personal_comment(path(&[]), "local after provider import".into())
            .unwrap();
    }
}

#[test]
fn network_policy_change_discards_old_frame_and_next_request_uses_new_revision() {
    let state = CurrentGameState::default();
    let network = NetworkState::default();
    let session = install(&state, &network, ROOM, SOURCE);
    let identity = state.document_identity();
    let before = current(&state);
    let old = poll(&state, &network);
    let policy = network
        .commit(app_model::NetworkSettingsDto::default(), || Ok(()))
        .unwrap();
    assert!(old.lease().check().is_err());
    let discarded = complete(&state, &network, &old, Ok(imported(ROOM, APPENDED)));
    assert!(discarded.current.is_none());
    assert_eq!(current(&state), before);
    assert_eq!(discarded.sync.session_id, Some(session));
    assert_eq!(discarded.sync.phase, Phase::Syncing);
    assert!(discarded.sync.failure.is_none());
    let new = poll(&state, &network);
    assert_eq!(new.identity().policy_revision, policy.policy_revision);
    assert_eq!(new.identity().document_identity, identity);
    assert_stale_completion(&state, old.identity(), &network, ROOM);
    assert!(
        complete(&state, &network, &new, Ok(imported(ROOM, APPENDED)))
            .current
            .unwrap()
            .dirty
    );
    assert_eq!(state.document_identity(), identity);
}

#[test]
fn stop_seals_inflight_publication_preserves_last_good_and_restores_ordinary_edits() {
    let state = CurrentGameState::default();
    let network = NetworkState::default();
    let session = install(&state, &network, ROOM, SOURCE);
    let identity = state.document_identity();
    let before = current(&state);
    let request = poll(&state, &network);
    let stopped = state.stop_external_sync(session).unwrap();
    assert_eq!(stopped.sync.phase, Phase::Idle);
    assert!(stopped.sync.session_id.is_none());
    assert_eq!(stopped.current.unwrap(), before);
    assert_eq!(state.document_identity(), identity);
    assert!(request.lease().check().is_err());
    assert_stale_completion(&state, request.identity(), &network, ROOM);
    assert!(matches!(
        state.next_external_poll(&network),
        YikeSyncWork::Wait(None)
    ));
    let edited = state
        .set_personal_comment(path(&[]), "ordinary review after Stop".into())
        .unwrap();
    assert!(edited.dirty);
    let played = state
        .play(path(&[0, 0]), MoveVertex::Point(PointDto { x: 1, y: 1 }))
        .unwrap();
    assert!(played.dirty);
    let trial = state.enter_trial().unwrap();
    state.exit_trial(trial.session_id).unwrap();
    let save_path = unique_path("stopped-edit");
    let saved = state
        .save_to_path(
            save_path.to_string_lossy().into_owned(),
            current(&state).selected_path,
        )
        .unwrap()
        .current_game
        .unwrap();
    assert!(!saved.dirty);
    let reopened = CurrentSgfDocument::open(&fs::read_to_string(&save_path).unwrap()).unwrap();
    assert_eq!(
        reopened.snapshot(&path(&[])).unwrap().personal_comment,
        "ordinary review after Stop"
    );
    fs::remove_file(save_path).unwrap();
}

#[test]
fn save_writes_invocation_frame_and_later_sync_frame_remains_dirty_in_same_document() {
    let state = CurrentGameState::default();
    let network = NetworkState::default();
    let session = install(&state, &network, ROOM, SOURCE);
    let identity = state.document_identity();
    state.select_path(path(&[0]), current(&state).generation).unwrap();
    let request = poll(&state, &network);
    complete(&state, &network, &request, Ok(imported(ROOM, REVISED)));
    let invocation = state.serialize().unwrap();
    let invocation_generation = current(&state).generation;
    let next = schedule_poll(&state, &network);
    let save_path = unique_path("frame-during-save");
    let saved = state
        .save_to_path_after_hook(save_path.to_string_lossy().into_owned(), path(&[0]), || {
            let update = complete(&state, &network, &next, Ok(imported(ROOM, FINISHED)));
            assert!(update.current.unwrap().dirty);
        })
        .unwrap()
        .current_game
        .unwrap();
    assert_eq!(fs::read_to_string(&save_path).unwrap(), invocation);
    assert!(saved.dirty);
    assert_eq!(saved.generation, invocation_generation + 1);
    assert_eq!(state.document_identity(), identity);
    assert_eq!(state.external_sync_snapshot().session_id, Some(session));
    assert_eq!(
        state.mainline_projection().unwrap().summary.result.as_deref(),
        Some("B+R")
    );
    let saved_again = state
        .save_to_path(save_path.to_string_lossy().into_owned(), saved.selected_path)
        .unwrap()
        .current_game
        .unwrap();
    assert!(!saved_again.dirty);
    assert_eq!(
        fs::read_to_string(&save_path).unwrap(),
        state.serialize().unwrap()
    );
    fs::remove_file(save_path).unwrap();
}

#[test]
fn save_as_picker_delay_keeps_invocation_frame_and_later_source_dirty() {
    let state = CurrentGameState::default();
    let network = NetworkState::default();
    let session = install(&state, &network, ROOM, SOURCE);
    let identity = state.document_identity();
    let selected = current(&state).selected_path;
    let snapshot = state.capture_save_snapshot(selected).unwrap();
    let invocation_text = state.serialize().unwrap();
    // A native picker is pending while the worker commits a newer frame.
    let operation = poll(&state, &network);
    complete(&state, &network, &operation, Ok(imported(ROOM, APPENDED)));
    let path = std::env::temp_dir().join(format!("sync-picker-{}.sgf", uuid::Uuid::new_v4()));
    let saved = crate::save_as::persist_current_game_save_as(
        &state,
        save_as_dialog::SaveAsDialogOutcome::Chosen(path.to_string_lossy().into_owned()),
        snapshot,
    )
    .unwrap()
    .unwrap()
    .current_game
    .unwrap();
    let written = fs::read_to_string(&path).unwrap();
    fs::remove_file(path).unwrap();
    assert_eq!(written, invocation_text);
    assert!(saved.dirty);
    assert_eq!(state.document_identity(), identity);
    assert_eq!(state.external_sync_snapshot().session_id, Some(session));
    assert_eq!(state.mainline_projection().unwrap().summary.move_count, 3);
}

#[test]
fn exit_seals_active_and_pending_requests_and_recovery_restores_only_document() {
    let state = CurrentGameState::default();
    let network = NetworkState::default();
    install(&state, &network, ROOM, SOURCE);
    let request = poll(&state, &network);
    complete(&state, &network, &request, Ok(imported(ROOM, APPENDED)));
    state.select_path(path(&[0]), current(&state).generation).unwrap();
    state
        .attach_from_job_event(&job_event(current(&state).generation, &[0], 900))
        .unwrap();
    let before = current(&state);
    let text = state.serialize().unwrap();
    let envelope = state
        .recovery_flush(ApplicationExitDispositionDto::ExitIncomplete)
        .unwrap();
    assert_eq!(envelope.sgf_text, text);
    assert!(envelope.dirty);
    assert_eq!(envelope.selected_path, before.selected_path);
    let inflight = schedule_poll(&state, &network);
    let pending = state
        .begin_external_start(&network, OTHER_ROOM.into(), YikeSyncPreferencesDto::default())
        .unwrap();
    let pending_request = state.external_start_request(pending).unwrap().0;
    let departure = departure_id(state.prepare_exit().unwrap());
    state.begin_protected_commit(departure, &[]).unwrap();
    state
        .begin_application_teardown(
            departure,
            before.selected_path.clone(),
            ApplicationExitDispositionDto::ExitIncomplete,
        )
        .unwrap();
    state.seal_external_sync_for_exit();
    assert!(inflight.lease().check().is_err());
    assert!(pending_request.lease().check().is_err());
    assert!(network.shutdown(Duration::from_secs(1)));
    assert_eq!(state.external_sync_snapshot().phase, Phase::Idle);
    assert!(state.external_sync_snapshot().session_id.is_none());
    assert!(state.external_sync_snapshot().starting_id.is_none());
    assert_stale_completion(&state, inflight.identity(), &network, ROOM);
    assert!(state
        .prepare_external_candidate(pending, imported(OTHER_ROOM, ORIGINAL))
        .is_err());
    let exited = state
        .finish_application_teardown(
            departure,
            before.selected_path.clone(),
            app_model::ApplicationTeardownAttemptDto::Completed,
            false,
        )
        .unwrap();
    assert!(exited.committed);
    assert_eq!(state.serialize().unwrap(), text);
    let restored_state = CurrentGameState::default();
    let restored = restored_state.restore_envelope(envelope).unwrap();
    assert!(restored.dirty);
    assert_eq!(restored.selected_path, before.selected_path);
    assert_eq!(restored.snapshot.primary_analysis.unwrap().visits, 900);
    assert_eq!(restored_state.serialize().unwrap(), text);
    assert_eq!(restored_state.external_sync_snapshot().phase, Phase::Idle);
    assert!(restored_state.external_sync_snapshot().session_id.is_none());
    assert!(restored_state.external_sync_snapshot().starting_id.is_none());
    assert!(matches!(
        restored_state.next_external_poll(&NetworkState::default()),
        YikeSyncWork::Wait(None)
    ));
    restored_state
        .set_personal_comment(path(&[]), "restored document is ordinary review".into())
        .unwrap();
    let trial = restored_state.enter_trial().unwrap();
    restored_state.exit_trial(trial.session_id).unwrap();
}

const LOCAL_3: &str = "(;GM[1]FF[4]SZ[3]PB[Local];B[aa])";

/// Runtime snapshots carry a monotonic revision like the crate-owned runtime publishes.
fn runtime(generation: u64, phase: app_model::ReadboardPhaseDto) -> app_model::ReadboardRuntimeDto {
    static REVISION: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1_000);
    runtime_at(
        generation,
        REVISION.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
        phase,
    )
}

fn runtime_at(
    generation: u64,
    revision: u64,
    phase: app_model::ReadboardPhaseDto,
) -> app_model::ReadboardRuntimeDto {
    app_model::ReadboardRuntimeDto {
        generation,
        revision,
        phase,
        executable_path: None,
        process_id: None,
        endpoint: None,
        wire_version: Some("220430".into()),
        resources_held: true,
        message: "owner seam runtime".into(),
    }
}

fn board(rows: [&str; 3]) -> go_core::ReadboardFrame {
    go_core::ReadboardFrame {
        width: 3,
        height: 3,
        codes: rows
            .iter()
            .flat_map(|row| row.bytes().map(|byte| byte - b'0'))
            .collect(),
        context: go_core::ReadboardRemoteContext::generic(false),
    }
}

fn readboard_session(state: &CurrentGameState, generation: u64) -> u64 {
    let id = state
        .begin_readboard_start(
            &runtime(generation, app_model::ReadboardPhaseDto::Ready),
            app_model::ReadboardSyncPreferencesDto::default(),
        )
        .unwrap();
    assert!(state
        .observe_readboard_frame(generation, &board(["100", "040", "000"]))
        .is_some());
    let candidate = state
        .prepare_readboard_candidate(id, Duration::from_millis(50))
        .unwrap();
    let departure = departure_id(candidate.admission);
    state.begin_protected_commit(departure, &[]).unwrap();
    assert!(state.commit_replacement(departure).unwrap().committed);
    id
}

#[test]
fn pending_readboard_sync_control_preserves_the_received_candidate() {
    let state = CurrentGameState::default();
    state.replace(LOCAL_3, None).unwrap();
    let id = state
        .begin_readboard_start(
            &runtime(7, app_model::ReadboardPhaseDto::Ready),
            app_model::ReadboardSyncPreferencesDto::default(),
        )
        .unwrap();
    state
        .observe_readboard_frame(7, &board(["100", "040", "000"]))
        .unwrap();
    state
        .readboard_control(7, go_core::ReadboardControl::Sync)
        .unwrap();
    let candidate = state.prepare_readboard_candidate(id, Duration::ZERO).unwrap();
    let departure = departure_id(candidate.admission);
    state.begin_protected_commit(departure, &[]).unwrap();
    assert!(state.commit_replacement(departure).unwrap().committed);
    assert_eq!(
        state.serialize().unwrap(),
        normalized("(;GM[1]FF[4]SZ[3]PB[Local];B[aa];W[bb])")
    );
    assert_eq!(
        state
            .external_sync_snapshot()
            .readboard
            .unwrap()
            .source_move_number,
        Some(2)
    );
}

#[test]
fn pending_readboard_clear_discards_the_candidates_move_number_context() {
    let state = CurrentGameState::default();
    let original = "(;SZ[3];B[aa];W[ba])";
    state.replace(original, None).unwrap();
    let id = state
        .begin_readboard_start(
            &runtime(7, app_model::ReadboardPhaseDto::Ready),
            app_model::ReadboardSyncPreferencesDto::default(),
        )
        .unwrap();
    let mut discarded = board(["000", "110", "000"]);
    discarded.context = go_core::ReadboardRemoteContext::generic(true);
    state.observe_readboard_frame(7, &discarded).unwrap();
    state
        .readboard_control(7, go_core::ReadboardControl::Clear)
        .unwrap();
    state
        .observe_readboard_frame(7, &board(["140", "000", "000"]))
        .unwrap();
    assert_eq!(state.serialize().unwrap(), normalized(original));
    let candidate = state.prepare_readboard_candidate(id, Duration::ZERO).unwrap();
    let departure = departure_id(candidate.admission);
    state.begin_protected_commit(departure, &[]).unwrap();
    let installed = state.commit_replacement(departure).unwrap();
    assert!(installed.committed);
    assert_eq!(installed.current.unwrap().selected_path, path(&[0, 0]));
    assert_eq!(state.serialize().unwrap(), normalized(original));
    assert_eq!(
        state
            .external_sync_snapshot()
            .readboard
            .unwrap()
            .source_move_number,
        Some(2)
    );
}

#[test]
fn pending_readboard_clear_board_survives_start_before_the_first_frame() {
    let state = CurrentGameState::default();
    let original = "(;SZ[3]PB[Alice];B[aa];W[ba])";
    state.replace(original, None).unwrap();
    let id = state
        .begin_readboard_start(
            &runtime(7, app_model::ReadboardPhaseDto::Ready),
            app_model::ReadboardSyncPreferencesDto::default(),
        )
        .unwrap();
    state
        .readboard_control(7, go_core::ReadboardControl::ClearBoard)
        .unwrap();
    state
        .readboard_control(7, go_core::ReadboardControl::Start { size: Some((3, 3)) })
        .unwrap();
    state
        .observe_readboard_frame(7, &board(["140", "000", "000"]))
        .unwrap();
    assert_eq!(state.serialize().unwrap(), normalized(original));
    let candidate = state.prepare_readboard_candidate(id, Duration::ZERO).unwrap();
    let departure = departure_id(candidate.admission);
    state.begin_protected_commit(departure, &[]).unwrap();
    let installed = state.commit_replacement(departure).unwrap();
    assert!(installed.committed);
    let current = installed.current.unwrap();
    assert_eq!(current.selected_path, path(&[]));
    assert!(current.tree.children.is_empty());
    let properties = current
        .tree
        .properties
        .into_iter()
        .map(|property| (property.key, property.values))
        .collect::<std::collections::BTreeMap<_, _>>();
    assert_eq!(properties.get("AB").unwrap(), &["aa"]);
    assert_eq!(properties.get("AW").unwrap(), &["ba"]);
    assert_eq!(properties.get("PL").unwrap(), &["B"]);
    assert!(!properties.contains_key("PB"));
}

#[test]
fn protected_readboard_start_cannot_be_cancelled_before_atomic_install() {
    let state = CurrentGameState::default();
    state.replace(LOCAL_3, None).unwrap();
    let original_identity = state.document_identity();
    let id = state
        .begin_readboard_start(
            &runtime(7, app_model::ReadboardPhaseDto::Ready),
            app_model::ReadboardSyncPreferencesDto::default(),
        )
        .unwrap();
    state
        .observe_readboard_frame(7, &board(["100", "040", "000"]))
        .unwrap();
    let candidate = state
        .prepare_readboard_candidate(id, Duration::from_millis(50))
        .unwrap();
    let departure = departure_id(candidate.admission);
    state.begin_protected_commit(departure, &[]).unwrap();
    state.cancel_external_start(id);

    let installed = state.commit_replacement(departure).unwrap();
    assert!(installed.committed);
    assert_ne!(state.document_identity(), original_identity);
    assert_eq!(installed.current.unwrap().selected_path, path(&[0, 0]));
    assert_eq!(
        state.serialize().unwrap(),
        normalized("(;GM[1]FF[4]SZ[3]PB[Local];B[aa];W[bb])")
    );
    let sync = state.cancel_external_start(id);
    assert_eq!((sync.phase, sync.session_id), (Phase::Syncing, Some(id)));
}

#[test]
fn readboard_start_passes_one_sgf07_decision_and_later_frames_update_the_same_document() {
    let state = CurrentGameState::default();
    state.replace(LOCAL_3, None).unwrap();
    state
        .set_personal_comment(path(&[]), "unsaved review".into())
        .unwrap();
    let before = current(&state);
    let id = state
        .begin_readboard_start(
            &runtime(7, app_model::ReadboardPhaseDto::Ready),
            app_model::ReadboardSyncPreferencesDto::default(),
        )
        .unwrap();
    assert_eq!(
        state.external_sync_snapshot().source,
        Some(app_model::ExternalSyncSourceDto::Readboard)
    );
    // Frames of another connection never become the candidate.
    assert!(state
        .observe_readboard_frame(6, &board(["100", "040", "000"]))
        .is_none());
    assert!(state
        .prepare_readboard_candidate(id, Duration::from_millis(20))
        .is_err());
    // The gateway cancels a start whose preparation failed.
    state.cancel_external_start(id);

    let id = state
        .begin_readboard_start(
            &runtime(7, app_model::ReadboardPhaseDto::Ready),
            app_model::ReadboardSyncPreferencesDto::default(),
        )
        .unwrap();
    state
        .observe_readboard_frame(7, &board(["100", "040", "000"]))
        .unwrap();
    let candidate = state
        .prepare_readboard_candidate(id, Duration::from_millis(50))
        .unwrap();
    let DocumentDepartureAdmissionDto::NeedsDecision { departure_id } = candidate.admission else {
        panic!("a dirty document needs the single SGF-07 decision");
    };
    assert_eq!(
        current(&state),
        before,
        "the candidate is not installed before the decision"
    );
    state.begin_protected_commit(departure_id, &[]).unwrap();
    let installed = state.commit_replacement(departure_id).unwrap().current.unwrap();
    assert!(!installed.dirty);
    assert_eq!(installed.selected_path, path(&[0, 0]));
    assert_eq!(
        state.serialize().unwrap(),
        normalized("(;GM[1]FF[4]SZ[3]PB[Local]C[unsaved review];B[aa];W[bb])")
    );
    let identity = state.document_identity();
    let sync = state.external_sync_snapshot();
    assert_eq!((sync.phase, sync.session_id), (Phase::Syncing, Some(id)));

    let update = state
        .observe_readboard_frame(7, &board(["100", "020", "003"]))
        .unwrap();
    let moved = update.current.unwrap();
    assert_eq!(moved.selected_path, path(&[0, 0, 0]));
    assert!(moved.dirty, "later source content participates in dirty/recovery");
    assert_eq!(
        state.document_identity(),
        identity,
        "frames never re-create the document identity"
    );
    assert!(state
        .observe_readboard_frame(6, &board(["000", "000", "000"]))
        .is_none());
    assert_eq!(
        state
            .set_personal_comment(path(&[]), "forbidden".into())
            .unwrap_err()
            .kind,
        CurrentGameErrorKind::DepartureBlocked
    );
}

#[test]
fn readboard_disconnect_pauses_retry_needs_a_newer_ready_and_stop_restores_editing() {
    let state = CurrentGameState::default();
    state.replace(LOCAL_3, None).unwrap();
    let session = readboard_session(&state, 7);
    let last_good = current(&state);
    let paused = state
        .readboard_runtime_changed(&runtime(7, app_model::ReadboardPhaseDto::Disconnected))
        .unwrap();
    assert_eq!(paused.phase, Phase::ErrorPaused);
    assert_eq!(
        paused.failure.unwrap().kind,
        ProviderErrorKind::RuntimeUnavailable
    );
    assert!(state
        .observe_readboard_frame(7, &board(["100", "020", "003"]))
        .is_none());
    assert_eq!(current(&state), last_good);

    assert_eq!(state.retry_external_sync(session).unwrap().phase, Phase::Retrying);
    assert!(state
        .readboard_runtime_changed(&runtime(7, app_model::ReadboardPhaseDto::Ready))
        .is_none());
    assert!(state
        .readboard_runtime_changed(&runtime(8, app_model::ReadboardPhaseDto::Starting))
        .is_none());
    let resumed = state
        .readboard_runtime_changed(&runtime(9, app_model::ReadboardPhaseDto::Ready))
        .unwrap();
    assert_eq!(resumed.phase, Phase::Syncing);
    assert_eq!(resumed.readboard.unwrap().runtime_generation, Some(9));
    assert!(
        state
            .observe_readboard_frame(7, &board(["100", "020", "003"]))
            .is_none(),
        "the old run stays stale"
    );
    assert!(state
        .observe_readboard_frame(9, &board(["100", "020", "003"]))
        .unwrap()
        .current
        .is_some());

    let stopped = state.stop_external_sync(session).unwrap();
    assert_eq!(stopped.sync.phase, Phase::Idle);
    assert!(state
        .observe_readboard_frame(9, &board(["100", "020", "000"]))
        .is_none());
    state
        .set_personal_comment(path(&[]), "editable after Stop".into())
        .unwrap();
}

#[test]
fn readboard_pending_start_holds_conflicts_and_is_cancelled_when_its_runtime_leaves_ready() {
    let state = CurrentGameState::default();
    state.replace(LOCAL_3, None).unwrap();
    let id = state
        .begin_readboard_start(
            &runtime(3, app_model::ReadboardPhaseDto::Ready),
            app_model::ReadboardSyncPreferencesDto::default(),
        )
        .unwrap();
    let conflict = board(["000", "000", "120"]);
    state.observe_readboard_frame(3, &conflict).unwrap();
    assert!(
        state
            .prepare_readboard_candidate(id, Duration::from_millis(20))
            .is_err(),
        "a held conflict is not a candidate"
    );
    state.cancel_external_start(id);

    let id = state
        .begin_readboard_start(
            &runtime(3, app_model::ReadboardPhaseDto::Ready),
            app_model::ReadboardSyncPreferencesDto::default(),
        )
        .unwrap();
    state.observe_readboard_frame(3, &conflict).unwrap();
    state.observe_readboard_frame(3, &conflict).unwrap();
    assert!(
        state
            .prepare_readboard_candidate(id, Duration::from_millis(50))
            .is_ok(),
        "the repeated conflict rebuilds"
    );
    state.cancel_external_start(id);

    let id = state
        .begin_readboard_start(
            &runtime(3, app_model::ReadboardPhaseDto::Ready),
            app_model::ReadboardSyncPreferencesDto::default(),
        )
        .unwrap();
    let cancelled = state
        .readboard_runtime_changed(&runtime(4, app_model::ReadboardPhaseDto::Stopping))
        .unwrap();
    assert!(cancelled.starting_id.is_none());
    assert!(state
        .prepare_readboard_candidate(id, Duration::from_millis(20))
        .is_err());
    assert!(state
        .begin_readboard_start(
            &runtime(5, app_model::ReadboardPhaseDto::Exited),
            app_model::ReadboardSyncPreferencesDto::default()
        )
        .is_err());
}

#[test]
fn readboard_preferences_drive_focus_and_attachments_follow_the_06_permission() {
    let state = CurrentGameState::default();
    state.replace(LOCAL_3, None).unwrap();
    readboard_session(&state, 7);
    assert_eq!(state.readboard_focus_generation(), Some(7));
    let snapshot = state.update_readboard_preferences(app_model::ReadboardSyncPreferencesDto {
        focus: false,
        ..Default::default()
    });
    let preferences = snapshot.readboard.unwrap().preferences;
    assert!(!preferences.focus && preferences.mute && preferences.always_sync && !preferences.jump_to_last);
    assert_eq!(state.readboard_focus_generation(), None);

    let generation = current(&state).generation;
    state.admit_selected_node(generation, &path(&[0, 0])).unwrap();
    assert!(
        state
            .attach_from_job_event(&job_event(generation, &[0, 0], 700))
            .is_some(),
        "exact analysis still attaches"
    );
    let held = state
        .observe_readboard_frame(7, &board(["000", "000", "000"]))
        .unwrap();
    assert!(held.current.is_none(), "an emptied board is held once");
    let rebuilt = state
        .observe_readboard_frame(7, &board(["000", "000", "000"]))
        .unwrap()
        .current
        .unwrap();
    assert!(rebuilt.generation > generation);
    let before = current(&state);
    assert!(
        state
            .attach_from_job_event(&job_event(generation, &[0, 0], 900))
            .is_none(),
        "late result of the old source is dropped"
    );
    assert_eq!(current(&state), before);
}

#[test]
fn readboard_connection_lost_during_the_sgf07_decision_installs_a_recoverable_pause() {
    let state = CurrentGameState::default();
    state.replace(LOCAL_3, None).unwrap();
    state
        .set_personal_comment(path(&[]), "unsaved review".into())
        .unwrap();
    let id = state
        .begin_readboard_start(
            &runtime(7, app_model::ReadboardPhaseDto::Ready),
            app_model::ReadboardSyncPreferencesDto::default(),
        )
        .unwrap();
    state
        .observe_readboard_frame(7, &board(["100", "040", "000"]))
        .unwrap();
    let candidate = state
        .prepare_readboard_candidate(id, Duration::from_millis(50))
        .unwrap();
    let DocumentDepartureAdmissionDto::NeedsDecision { departure_id } = candidate.admission else {
        panic!("a dirty document needs the SGF-07 decision");
    };
    state.readboard_runtime_changed(&runtime(7, app_model::ReadboardPhaseDto::Disconnected));
    state.begin_protected_commit(departure_id, &[]).unwrap();
    assert!(state.commit_replacement(departure_id).unwrap().committed);
    let sync = state.external_sync_snapshot();
    assert_eq!(
        (sync.phase, sync.session_id),
        (Phase::ErrorPaused, Some(id)),
        "never Syncing on a dead connection"
    );
    assert_eq!(sync.failure.unwrap().kind, ProviderErrorKind::RuntimeUnavailable);
    assert!(state
        .observe_readboard_frame(7, &board(["100", "020", "003"]))
        .is_none());
    assert_eq!(state.retry_external_sync(id).unwrap().phase, Phase::Retrying);
    assert_eq!(
        state
            .readboard_runtime_changed(&runtime(8, app_model::ReadboardPhaseDto::Ready))
            .unwrap()
            .phase,
        Phase::Syncing
    );
}

#[test]
fn readboard_stop_and_end_controls_release_the_session_like_stop() {
    for control in [
        go_core::ReadboardControl::StopSync,
        go_core::ReadboardControl::EndSync,
    ] {
        let state = CurrentGameState::default();
        state.replace(LOCAL_3, None).unwrap();
        readboard_session(&state, 7);
        let last_good = state.serialize().unwrap();
        assert!(
            state.readboard_control(6, control).is_none(),
            "another connection cannot stop the session"
        );
        let stopped = state.readboard_control(7, control).unwrap();
        assert_eq!(
            (stopped.phase, stopped.session_id),
            (Phase::Idle, None),
            "{control:?}"
        );
        assert!(state
            .observe_readboard_frame(7, &board(["100", "020", "003"]))
            .is_none());
        assert_eq!(state.serialize().unwrap(), last_good);
        state
            .set_personal_comment(path(&[]), "editable after readboard stop".into())
            .unwrap();

        let id = state
            .begin_readboard_start(
                &runtime(7, app_model::ReadboardPhaseDto::Ready),
                app_model::ReadboardSyncPreferencesDto::default(),
            )
            .unwrap();
        assert!(state.readboard_control(7, control).unwrap().starting_id.is_none());
        assert!(state
            .prepare_readboard_candidate(id, Duration::from_millis(20))
            .is_err());
    }
}

#[test]
fn readboard_lifecycle_snapshots_older_than_the_admitted_one_are_ignored() {
    let state = CurrentGameState::default();
    state.replace(LOCAL_3, None).unwrap();
    let id = state
        .begin_readboard_start(
            &runtime_at(7, 50, app_model::ReadboardPhaseDto::Ready),
            app_model::ReadboardSyncPreferencesDto::default(),
        )
        .unwrap();
    // Queued snapshots published before the one Start observed.
    assert!(state
        .readboard_runtime_changed(&runtime_at(6, 48, app_model::ReadboardPhaseDto::Stopped))
        .is_none());
    assert!(state
        .readboard_runtime_changed(&runtime_at(7, 49, app_model::ReadboardPhaseDto::Starting))
        .is_none());
    assert_eq!(state.external_sync_snapshot().starting_id, Some(id));
    state
        .observe_readboard_frame(7, &board(["100", "040", "000"]))
        .unwrap();
    let candidate = state
        .prepare_readboard_candidate(id, Duration::from_millis(50))
        .unwrap();
    let departure = departure_id(candidate.admission);
    state.begin_protected_commit(departure, &[]).unwrap();
    assert!(state.commit_replacement(departure).unwrap().committed);
    assert!(state
        .readboard_runtime_changed(&runtime_at(7, 50, app_model::ReadboardPhaseDto::Ready))
        .is_none());
    assert_eq!(state.external_sync_snapshot().phase, Phase::Syncing);
    // A genuine later loss still pauses.
    assert_eq!(
        state
            .readboard_runtime_changed(&runtime_at(7, 51, app_model::ReadboardPhaseDto::Disconnected))
            .unwrap()
            .phase,
        Phase::ErrorPaused
    );
}

#[test]
fn a_direct_start_on_a_newer_runtime_still_pauses_the_session_of_the_lost_connection() {
    let state = CurrentGameState::default();
    state.replace(LOCAL_3, None).unwrap();
    let session = readboard_session(&state, 7);
    // The Start command reads Ready(9) before the bridge drained generation 7's closing snapshots.
    let id = state
        .begin_readboard_start(
            &runtime(9, app_model::ReadboardPhaseDto::Ready),
            app_model::ReadboardSyncPreferencesDto::default(),
        )
        .unwrap();
    state.cancel_external_start(id);
    let sync = state.external_sync_snapshot();
    assert_eq!((sync.phase, sync.session_id), (Phase::ErrorPaused, Some(session)));
    assert!(state
        .observe_readboard_frame(7, &board(["100", "020", "003"]))
        .is_none());
    assert_eq!(state.retry_external_sync(session).unwrap().phase, Phase::Retrying);
    assert_eq!(
        state
            .readboard_runtime_changed(&runtime(10, app_model::ReadboardPhaseDto::Ready))
            .unwrap()
            .phase,
        Phase::Syncing
    );
}
