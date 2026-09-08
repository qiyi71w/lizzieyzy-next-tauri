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
        score_mean_black: 2.25,
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
    let mut policy = vec![0.5 / 25.0; 26];
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
fn whole_game_attachment_does_not_displace_continuous_selected_node() {
    let state = CurrentGameState::default();
    let opened = state.replace(BRANCHING, None).unwrap();
    state.select_path(path(&[])).unwrap();
    assert!(state
        .attach_from_job_event(&continuous_job_event(
            AnalysisJobOutcomeDto::Progress,
            opened.generation,
            &[],
            Some(projectable_frame(100, 3, 3)),
        ))
        .is_some());
    assert!(state
        .attach_from_job_event(&job_event(
            AnalysisJobLaneDto::WholeGame,
            AnalysisJobOutcomeDto::Progress,
            opened.generation,
            &[0],
            Some(projectable_frame(200, 4, 4)),
        ))
        .is_some());
    assert!(state
        .attach_from_job_event(&continuous_job_event(
            AnalysisJobOutcomeDto::Progress,
            opened.generation,
            &[],
            Some(projectable_frame(300, 5, 5)),
        ))
        .is_some());
    assert_eq!(
        state.take_due_recovery_write(u64::MAX).unwrap().selected_path,
        path(&[])
    );
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
