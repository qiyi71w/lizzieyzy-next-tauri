use super::*;
use app_model::{
    AnalysisFrameDto, AnalysisJobEventDto, AnalysisJobLaneDto, AnalysisJobModeDto, AnalysisJobOutcomeDto,
    CandidateMoveDto, CurrentGameErrorKind, MoveVertex, NodePath, PointDto,
};
use sgf::SgfAnalysisPayload;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;
const BRANCHING: &str = include_str!("../../../../../tests/golden/java-analysis-branching.sgf");

fn path(indices: &[u32]) -> NodePath {
    NodePath {
        indices: indices.to_vec(),
    }
}

fn payload(engine: &str, visits: u32, x: u8, y: u8) -> SgfAnalysisPayload {
    SgfAnalysisPayload {
        engine_name: engine.to_string(),
        visits,
        winrate_black: 0.61,
        score_mean_black: Some(2.25),
        score_stdev: Some(0.5),
        pda: None,
        candidates: vec![CandidateMoveDto {
            vertex: MoveVertex::Point(PointDto { x, y }),
            visits,
            winrate_black: 0.61,
            score_mean_black: 2.25,
            policy_prior: Some(0.4),
            pv: vec![MoveVertex::Point(PointDto { x, y })],
        }],
        ownership: None,
    }
}

#[test]
fn semantic_replacement_rejects_old_analysis_at_surviving_paths() {
    for replacement in [
        "(;SZ[19]AB[dd])",
        "(;SZ[19]PL[W])",
        "(;SZ[19]RU[Japanese])",
        "(;SZ[19]KM[6.5])",
        "(;SZ[19];B[dd])",
    ] {
        let state = CurrentGameState::default();
        let opened = state.replace("(;SZ[19])", None).unwrap();
        state.replace(replacement, None).unwrap();
        assert!(state
            .attach_primary_analysis(opened.generation, path(&[]), payload("KataGo", 400, 3, 3))
            .is_err());
        assert!(state
            .select_path(path(&[]))
            .unwrap()
            .snapshot
            .primary_analysis
            .is_none());
    }
}

#[test]
fn rejected_edits_and_existing_continuation_keep_admitted_analysis() {
    let state = CurrentGameState::default();
    let opened = state.replace("(;SZ[19];B[dd])", None).unwrap();
    assert!(state.remove_variation(path(&[])).is_err());
    assert!(state.set_personal_comment(path(&[9]), "invalid".into()).is_err());
    state.set_personal_comment(path(&[]), "".into()).unwrap();
    state
        .play(path(&[]), MoveVertex::Point(PointDto { x: 3, y: 3 }))
        .unwrap();
    assert!(state
        .play(path(&[0]), MoveVertex::Point(PointDto { x: 3, y: 3 }))
        .is_err());
    let attached = state
        .attach_primary_analysis(opened.generation, path(&[0]), payload("KataGo", 400, 4, 4))
        .unwrap();
    assert_eq!(attached.snapshot.primary_analysis.unwrap().visits, 400);
    state.play(path(&[0]), MoveVertex::Pass).unwrap();
    assert!(state
        .attach_primary_analysis(opened.generation, path(&[0]), payload("KataGo", 900, 4, 4))
        .is_err());
    assert_eq!(
        state
            .select_path(path(&[0]))
            .unwrap()
            .snapshot
            .primary_analysis
            .unwrap()
            .visits,
        400
    );
}

#[test]
fn admitted_analysis_survives_personal_comment_and_save() {
    let state = CurrentGameState::default();
    let opened = state.replace(BRANCHING, None).unwrap();
    let admission = state.admit_whole_game(opened.generation).unwrap();
    let edited = state
        .set_personal_comment(path(&[]), "review in progress".into())
        .unwrap();
    assert!(edited.dirty);
    let attached = state
        .attach_primary_analysis(admission.generation, path(&[]), payload("KataGo", 400, 3, 3))
        .unwrap();
    assert_eq!(attached.snapshot.personal_comment, "review in progress");
    assert_eq!(attached.snapshot.primary_analysis.unwrap().visits, 400);
    let target = unique_sgf("comment-analysis");
    let saved = state
        .save_to_path(target.to_string_lossy().into_owned(), path(&[]))
        .unwrap();
    assert!(!saved.dirty);
    let reopened = CurrentSgfDocument::open(&fs::read_to_string(&target).unwrap()).unwrap();
    let snapshot = reopened.snapshot(&path(&[])).unwrap();
    assert_eq!(snapshot.personal_comment, "review in progress");
    assert_eq!(snapshot.primary_analysis.unwrap().visits, 400);
    let later = state
        .attach_primary_analysis(admission.generation, path(&[0]), payload("KataGo", 900, 15, 3))
        .unwrap();
    assert!(later.dirty);
    fs::remove_file(target).unwrap();
}

#[test]
fn attach_primary_dirties_without_changing_generation_and_keeps_secondary() {
    let state = CurrentGameState::default();
    let opened = state
        .replace(BRANCHING, Some("/tmp/branching.sgf".to_string()))
        .unwrap();
    assert!(!opened.dirty);

    let attached = state
        .attach_primary_analysis(opened.generation, path(&[]), payload("KataGo", 400, 3, 3))
        .unwrap();

    assert_eq!(attached.generation, opened.generation);
    assert!(attached.dirty);
    assert_eq!(attached.snapshot.primary_analysis.unwrap().visits, 400);
    assert_eq!(attached.snapshot.secondary_analysis.unwrap().visits, 150);
    assert_eq!(attached.snapshot.personal_comment, "root personal");
    assert_eq!(attached.native_path.as_deref(), Some("/tmp/branching.sgf"));
    let serialized = state.serialize().unwrap();
    assert!(serialized.contains("LZOP[KataGo"));
    assert!(serialized.contains("LZOP2[SubEngine"));
    assert!(serialized.contains("C[root personal]"));
    assert!(serialized.contains("XY[keep-me]"));
}

#[test]
fn stale_generation_invalid_path_and_malformed_payload_do_not_change_tree_or_dirty() {
    let state = CurrentGameState::default();
    let opened = state.replace(BRANCHING, None).unwrap();
    let before = state.inspect();

    let stale = state
        .attach_primary_analysis(opened.generation + 1, path(&[]), payload("KataGo", 400, 3, 3))
        .unwrap_err();
    assert_eq!(stale.kind, CurrentGameErrorKind::NoCurrentGame);
    assert_eq!(state.inspect(), before);

    let invalid = state
        .attach_primary_analysis(opened.generation, path(&[9]), payload("KataGo", 400, 3, 3))
        .unwrap_err();
    assert_eq!(invalid.kind, CurrentGameErrorKind::InvalidNodePath);
    assert_eq!(state.inspect(), before);

    let mut empty = payload("KataGo", 400, 3, 3);
    empty.visits = 0;
    empty.candidates.clear();
    let unchanged = state
        .attach_primary_analysis(opened.generation, path(&[]), empty)
        .unwrap();
    assert!(!unchanged.dirty);
    assert_eq!(unchanged.generation, opened.generation);
    assert_eq!(state.inspect(), before);
}

#[test]
fn same_canonical_payload_is_idempotent_and_replace_keeps_existing_dirty() {
    let state = CurrentGameState::default();
    let opened = state.replace(BRANCHING, None).unwrap();
    let first = payload("KataGo", 400, 3, 3);
    let attached = state
        .attach_primary_analysis(opened.generation, path(&[]), first.clone())
        .unwrap();
    assert!(attached.dirty);
    let generation = attached.generation;
    let serialized = state.serialize().unwrap();

    let again = state
        .attach_primary_analysis(generation, path(&[]), first)
        .unwrap();
    assert!(again.dirty);
    assert_eq!(again.generation, generation);
    assert_eq!(state.serialize().unwrap(), serialized);

    let replaced = state
        .attach_primary_analysis(generation, path(&[]), payload("KataGo", 401, 3, 3))
        .unwrap();
    assert!(replaced.dirty);
    assert_eq!(replaced.generation, generation);
    assert_eq!(replaced.snapshot.primary_analysis.unwrap().visits, 401);
    assert_eq!(replaced.snapshot.secondary_analysis.unwrap().visits, 150);
}

fn unique_sgf(label: &str) -> std::path::PathBuf {
    let unique = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    std::env::temp_dir().join(format!("lizzieyzy-attach-{label}-{unique}.sgf"))
}

#[test]
fn save_after_attach_writes_call_time_payloads_and_clears_dirty() {
    let state = CurrentGameState::default();
    let opened = state
        .replace(BRANCHING, Some("/tmp/branching.sgf".to_string()))
        .unwrap();
    let attached = state
        .attach_primary_analysis(opened.generation, path(&[]), payload("KataGo", 400, 3, 3))
        .unwrap();
    let path_on_disk = unique_sgf("save");
    let saved = state
        .save_to_path(
            path_on_disk.to_string_lossy().into_owned(),
            attached.selected_path.clone(),
        )
        .unwrap();
    let written = fs::read_to_string(&path_on_disk).unwrap();
    let _ = fs::remove_file(&path_on_disk);

    assert_eq!(saved.generation, attached.generation);
    assert!(!saved.dirty);
    assert!(written.contains("LZOP[KataGo"));
    assert!(written.contains("LZOP2[SubEngine"));
    assert!(written.contains("C[root personal]"));
    assert_eq!(written, state.serialize().unwrap());
}

#[test]
fn failed_save_keeps_dirty_and_in_memory_payloads() {
    let state = CurrentGameState::default();
    let opened = state
        .replace(BRANCHING, Some("/tmp/branching.sgf".to_string()))
        .unwrap();
    let attached = state
        .attach_primary_analysis(opened.generation, path(&[]), payload("KataGo", 400, 3, 3))
        .unwrap();
    let before = state.inspect();
    let unique = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let dir = std::env::temp_dir().join(format!("lizzieyzy-attach-save-failure-{unique}"));
    fs::create_dir_all(&dir).unwrap();
    let error = state
        .save_to_path(dir.to_string_lossy().into_owned(), attached.selected_path.clone())
        .unwrap_err();
    let _ = fs::remove_dir_all(&dir);

    assert!(error.contains("failed to write"), "{error}");
    assert_eq!(state.inspect(), before);
    assert!(state.serialize().unwrap().contains("LZOP[KataGo"));
}

#[test]
fn later_attach_after_save_redirties_and_next_save_includes_new_node() {
    let state = CurrentGameState::default();
    let opened = state
        .replace(BRANCHING, Some("/tmp/branching.sgf".to_string()))
        .unwrap();
    state
        .attach_primary_analysis(opened.generation, path(&[]), payload("KataGo", 400, 3, 3))
        .unwrap();
    let first_path = unique_sgf("snapshot-a");
    let saved = state
        .save_to_path(first_path.to_string_lossy().into_owned(), path(&[]))
        .unwrap();
    assert!(!saved.dirty);
    let snapshot_a = fs::read_to_string(&first_path).unwrap();

    let later = state
        .attach_primary_analysis(saved.generation, path(&[0]), payload("KataGo", 900, 15, 3))
        .unwrap();
    assert!(later.dirty);
    assert_eq!(later.generation, saved.generation);
    let second_path = unique_sgf("snapshot-b");
    let saved_b = state
        .save_to_path(second_path.to_string_lossy().into_owned(), path(&[0]))
        .unwrap();
    let snapshot_b = fs::read_to_string(&second_path).unwrap();
    let _ = fs::remove_file(&first_path);
    let _ = fs::remove_file(&second_path);

    assert!(!saved_b.dirty);
    assert!(snapshot_a.contains("LZOP[KataGo"));
    assert!(!snapshot_a.contains("LZ[KataGo"));
    assert!(snapshot_b.contains("LZOP[KataGo"));
    assert!(snapshot_b.contains("LZ[KataGo"));
    assert!(snapshot_b.contains("LZ2[SubEngine"));
}

#[test]
fn save_does_not_clear_dirty_when_a_later_attach_wins_the_epoch() {
    let state = CurrentGameState::default();
    let opened = state
        .replace(BRANCHING, Some("/tmp/branching.sgf".to_string()))
        .unwrap();
    state
        .attach_primary_analysis(opened.generation, path(&[]), payload("KataGo", 400, 3, 3))
        .unwrap();
    let save_path = unique_sgf("epoch");
    let saved = state
        .save_to_path_after_hook(save_path.to_string_lossy().into_owned(), path(&[]), || {
            state
                .attach_primary_analysis(opened.generation, path(&[0]), payload("KataGo", 900, 15, 3))
                .unwrap();
        })
        .unwrap();
    let written = fs::read_to_string(&save_path).unwrap();
    let _ = fs::remove_file(&save_path);

    assert!(saved.dirty);
    assert_eq!(saved.generation, opened.generation);
    assert!(written.contains("LZOP[KataGo"));
    assert!(!written.contains("LZ[KataGo 61.0 900"));
    assert!(state.serialize().unwrap().contains("LZ[KataGo"));
}

fn projectable_frame(visits: u32, x: u8, y: u8) -> AnalysisFrameDto {
    AnalysisFrameDto {
        job_id: Uuid::nil(),
        game_id: None,
        node_id: None,
        turn: 0,
        visits,
        winrate_black: 0.61,
        score_mean_black: Some(2.25),
        score_stdev: Some(0.5),
        candidates: vec![CandidateMoveDto {
            vertex: MoveVertex::Point(PointDto { x, y }),
            visits,
            winrate_black: 0.61,
            score_mean_black: 2.25,
            policy_prior: Some(0.4),
            pv: vec![MoveVertex::Point(PointDto { x, y })],
        }],
        ownership: None,
        policy: None,
    }
}

fn job_event(
    lane: AnalysisJobLaneDto,
    outcome: AnalysisJobOutcomeDto,
    generation: u64,
    indices: &[u32],
    frame: Option<AnalysisFrameDto>,
) -> AnalysisJobEventDto {
    AnalysisJobEventDto {
        run_id: "run-1".into(),
        job_id: "job-1".into(),
        lane,
        mode: AnalysisJobModeDto::Finite,
        generation,
        node_path: path(indices),
        outcome,
        completed: None,
        expected: None,
        remaining: None,
        frame,
        failure: None,
        current_game: None,
    }
}

fn continuous_job_event(
    outcome: AnalysisJobOutcomeDto,
    generation: u64,
    indices: &[u32],
    frame: Option<AnalysisFrameDto>,
) -> AnalysisJobEventDto {
    let mut event = job_event(
        AnalysisJobLaneDto::SelectedNode,
        outcome,
        generation,
        indices,
        frame,
    );
    event.mode = AnalysisJobModeDto::Continuous;
    event
}

#[test]
fn identity_valid_job_events_attach_and_terminal_failures_do_not() {
    let state = CurrentGameState::default();
    let opened = state.replace(BRANCHING, None).unwrap();
    let selected = state
        .attach_from_job_event(&job_event(
            AnalysisJobLaneDto::SelectedNode,
            AnalysisJobOutcomeDto::Completed,
            opened.generation,
            &[],
            Some(projectable_frame(400, 3, 3)),
        ))
        .unwrap();
    assert!(selected.dirty);
    assert_eq!(selected.generation, opened.generation);
    assert_eq!(selected.snapshot.primary_analysis.unwrap().visits, 400);

    let whole = state
        .attach_from_job_event(&job_event(
            AnalysisJobLaneDto::WholeGame,
            AnalysisJobOutcomeDto::Progress,
            opened.generation,
            &[0],
            Some(projectable_frame(900, 15, 3)),
        ))
        .unwrap();
    assert!(whole.dirty);
    assert_eq!(whole.snapshot.primary_analysis.unwrap().visits, 900);

    let before = state.inspect();
    for (lane, outcome) in [
        (AnalysisJobLaneDto::SelectedNode, AnalysisJobOutcomeDto::Cancelled),
        (AnalysisJobLaneDto::SelectedNode, AnalysisJobOutcomeDto::Failed),
        (AnalysisJobLaneDto::WholeGame, AnalysisJobOutcomeDto::Completed),
        (AnalysisJobLaneDto::WholeGame, AnalysisJobOutcomeDto::Failed),
    ] {
        assert!(state
            .attach_from_job_event(&job_event(
                lane,
                outcome,
                opened.generation,
                &[],
                Some(projectable_frame(50, 1, 1)),
            ))
            .is_none());
        assert_eq!(state.inspect(), before);
    }

    let mut empty = projectable_frame(50, 1, 1);
    empty.visits = 0;
    empty.candidates.clear();
    assert!(state
        .attach_from_job_event(&job_event(
            AnalysisJobLaneDto::SelectedNode,
            AnalysisJobOutcomeDto::Completed,
            opened.generation,
            &[],
            Some(empty),
        ))
        .is_none());
    assert_eq!(state.inspect(), before);

    assert!(state
        .attach_from_job_event(&job_event(
            AnalysisJobLaneDto::SelectedNode,
            AnalysisJobOutcomeDto::Completed,
            opened.generation + 1,
            &[],
            Some(projectable_frame(50, 1, 1)),
        ))
        .is_none());
    assert_eq!(state.inspect(), before);
}

#[test]
fn continuous_progress_updates_same_node_and_save_reopens_latest_snapshot() {
    let state = CurrentGameState::default();
    let opened = state.replace(BRANCHING, None).unwrap();
    let selected = opened.selected_path.clone();
    let personal_comment = opened.snapshot.personal_comment.clone();

    let first = state
        .attach_from_job_event(&continuous_job_event(
            AnalysisJobOutcomeDto::Progress,
            opened.generation,
            &selected.indices,
            Some(projectable_frame(100, 3, 3)),
        ))
        .unwrap();
    let mut policy_frame = projectable_frame(250, 4, 4);
    let mut policy = vec![0.5 / 361.0; 362];
    policy[0] = 0.5;
    policy_frame.policy = Some(policy);
    let second = state
        .attach_from_job_event(&continuous_job_event(
            AnalysisJobOutcomeDto::Progress,
            opened.generation,
            &selected.indices,
            Some(policy_frame),
        ))
        .unwrap();

    assert_eq!(second.generation, opened.generation);
    assert!(second.dirty);
    assert_eq!(first.snapshot.primary_analysis.unwrap().visits, 100);
    assert_eq!(second.snapshot.primary_analysis.as_ref().unwrap().visits, 250);
    assert_eq!(
        second
            .snapshot
            .primary_analysis
            .as_ref()
            .unwrap()
            .policy
            .as_ref()
            .map(|values| values[0]),
        Some(0.5)
    );
    assert_eq!(second.snapshot.personal_comment, personal_comment);
    let recovery = state
        .take_due_recovery_write(u64::MAX)
        .expect("analysis-only update reaches the bounded recovery writer");
    let recovered = CurrentGameState::default()
        .replace(&recovery.sgf_text, None)
        .unwrap();
    assert_eq!(recovered.snapshot.primary_analysis.unwrap().visits, 250);

    let save_path = unique_sgf("continuous-reopen");
    let saved = state
        .save_to_path(save_path.to_string_lossy().into_owned(), selected.clone())
        .unwrap();
    assert!(!saved.dirty);
    let later = state
        .attach_from_job_event(&continuous_job_event(
            AnalysisJobOutcomeDto::Progress,
            opened.generation,
            &selected.indices,
            Some(projectable_frame(375, 5, 5)),
        ))
        .unwrap();
    assert!(later.dirty);
    assert_eq!(later.generation, saved.generation);
    assert_eq!(later.snapshot.primary_analysis.unwrap().visits, 375);
    let reopened = CurrentGameState::default()
        .replace(&fs::read_to_string(&save_path).unwrap(), None)
        .unwrap();
    let _ = fs::remove_file(save_path);
    assert_eq!(reopened.snapshot.primary_analysis.unwrap().visits, 250);
    assert_eq!(reopened.snapshot.personal_comment, personal_comment);
}

#[test]
fn continuous_attachment_observes_selected_path_and_sealed_job_cutoff() {
    let state = CurrentGameState::default();
    let opened = state.replace(BRANCHING, None).unwrap();
    let selected = opened.selected_path.clone();
    let mut late = continuous_job_event(
        AnalysisJobOutcomeDto::Progress,
        opened.generation,
        &selected.indices,
        Some(projectable_frame(100, 3, 3)),
    );
    let started = app_model::AnalysisJobStartedDto {
        run_id: late.run_id.clone(),
        job_id: late.job_id.clone(),
        lane: late.lane,
        mode: late.mode,
        state: app_model::AnalysisJobStateDto::Searching,
        generation: late.generation,
        node_path: late.node_path.clone(),
    };

    state.seal_job(&started);
    assert!(state.attach_from_job_event(&late).is_none());
    let before = state.inspect();
    late.frame = Some(projectable_frame(200, 4, 4));
    assert!(state.attach_from_job_event(&late).is_none());
    assert_eq!(state.inspect(), before);

    let other = if selected.indices.is_empty() {
        path(&[0])
    } else {
        path(&[])
    };
    state.select_path(other).unwrap();
    let fresh = continuous_job_event(
        AnalysisJobOutcomeDto::Progress,
        opened.generation,
        &selected.indices,
        Some(projectable_frame(300, 5, 5)),
    );
    assert!(state.attach_from_job_event(&fresh).is_none());
}

#[test]
fn latest_admitted_lane_result_wins_primary_and_survives_save_and_recovery() {
    let state = CurrentGameState::default();
    state.replace(BRANCHING, None).unwrap();
    let opened = state.select_path(path(&[])).unwrap();
    let personal_comment = opened.snapshot.personal_comment.clone();
    let secondary_visits = opened.snapshot.secondary_analysis.as_ref().unwrap().visits;

    let continuous = state
        .attach_from_job_event(&continuous_job_event(
            AnalysisJobOutcomeDto::Progress,
            opened.generation,
            &[],
            Some(projectable_frame(600, 3, 3)),
        ))
        .unwrap();
    assert_eq!(continuous.snapshot.primary_analysis.unwrap().visits, 600);

    let mut whole_game_completed_node = job_event(
        AnalysisJobLaneDto::WholeGame,
        AnalysisJobOutcomeDto::Progress,
        opened.generation,
        &[],
        Some(projectable_frame(900, 4, 4)),
    );
    whole_game_completed_node.job_id = "whole-game-job".into();
    whole_game_completed_node.completed = Some(1);
    whole_game_completed_node.expected = Some(2);
    whole_game_completed_node.remaining = Some(1);
    let whole_game = state.attach_from_job_event(&whole_game_completed_node).unwrap();
    assert_eq!(whole_game.snapshot.primary_analysis.unwrap().visits, 900);

    let mut later_continuous = continuous_job_event(
        AnalysisJobOutcomeDto::Progress,
        opened.generation,
        &[],
        Some(projectable_frame(75, 5, 5)),
    );
    later_continuous.job_id = "later-continuous-job".into();
    let latest = state.attach_from_job_event(&later_continuous).unwrap();
    assert_eq!(latest.snapshot.primary_analysis.as_ref().unwrap().visits, 75);
    assert_eq!(latest.snapshot.personal_comment, personal_comment);
    assert_eq!(
        latest.snapshot.secondary_analysis.as_ref().unwrap().visits,
        secondary_visits
    );

    let started = app_model::AnalysisJobStartedDto {
        run_id: later_continuous.run_id.clone(),
        job_id: later_continuous.job_id.clone(),
        lane: later_continuous.lane,
        mode: later_continuous.mode,
        state: app_model::AnalysisJobStateDto::Searching,
        generation: later_continuous.generation,
        node_path: later_continuous.node_path.clone(),
    };
    state.seal_job(&started);
    let mut after_cutoff = later_continuous.clone();
    after_cutoff.frame = Some(projectable_frame(1_200, 6, 6));
    assert!(state.attach_from_job_event(&after_cutoff).is_none());

    for outcome in [AnalysisJobOutcomeDto::Cancelled, AnalysisJobOutcomeDto::Failed] {
        let rejected = job_event(
            AnalysisJobLaneDto::SelectedNode,
            outcome,
            opened.generation,
            &[],
            Some(projectable_frame(1_300, 7, 7)),
        );
        assert!(state.attach_from_job_event(&rejected).is_none());
    }
    let mut stale = continuous_job_event(
        AnalysisJobOutcomeDto::Progress,
        opened.generation + 1,
        &[],
        Some(projectable_frame(1_400, 8, 8)),
    );
    stale.job_id = "stale-generation-job".into();
    assert!(state.attach_from_job_event(&stale).is_none());

    let authoritative = state.select_path(path(&[])).unwrap();
    assert_eq!(
        authoritative.snapshot.primary_analysis.as_ref().unwrap().visits,
        75
    );
    assert_eq!(authoritative.snapshot.personal_comment, personal_comment);
    assert_eq!(
        authoritative.snapshot.secondary_analysis.as_ref().unwrap().visits,
        secondary_visits
    );

    let recovery = state
        .take_due_recovery_write(u64::MAX)
        .expect("admitted analysis reaches recovery");
    let recovered_state = CurrentGameState::default();
    recovered_state.replace(&recovery.sgf_text, None).unwrap();
    let recovered = recovered_state.select_path(path(&[])).unwrap();

    let save_path = unique_sgf("two-lane-admission-order");
    state
        .save_to_path(save_path.to_string_lossy().into_owned(), path(&[]))
        .unwrap();
    let reopened_state = CurrentGameState::default();
    reopened_state
        .replace(&fs::read_to_string(&save_path).unwrap(), None)
        .unwrap();
    let reopened = reopened_state.select_path(path(&[])).unwrap();
    let _ = fs::remove_file(save_path);

    for snapshot in [recovered.snapshot, reopened.snapshot] {
        assert_eq!(snapshot.primary_analysis.as_ref().unwrap().visits, 75);
        assert_eq!(snapshot.personal_comment, personal_comment);
        assert_eq!(
            snapshot.secondary_analysis.as_ref().unwrap().visits,
            secondary_visits
        );
    }
}

#[test]
fn existing_child_play_selects_without_dirtying_and_accepts_continuous_progress() {
    let state = CurrentGameState::default();
    let opened = state.replace(BRANCHING, None).unwrap();
    state.select_path(path(&[])).unwrap();
    let selected = state
        .play(path(&[]), MoveVertex::Point(PointDto { x: 3, y: 3 }))
        .unwrap();
    assert_eq!(selected.selected_path, path(&[0]));
    assert_eq!(selected.generation, opened.generation);
    assert!(!selected.dirty);
    assert_eq!(
        state.take_due_recovery_write(u64::MAX).unwrap().selected_path,
        path(&[0])
    );
    assert!(state
        .attach_from_job_event(&continuous_job_event(
            AnalysisJobOutcomeDto::Progress,
            selected.generation,
            &[0],
            Some(projectable_frame(100, 4, 4)),
        ))
        .is_some());
}
