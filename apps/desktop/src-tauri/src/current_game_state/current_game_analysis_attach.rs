use super::*;
use app_model::{
    AnalysisFrameDto, AnalysisJobEventDto, AnalysisJobLaneDto, AnalysisJobOutcomeDto, CandidateMoveDto,
    CurrentGameErrorKind, MoveVertex, NodePath, PointDto,
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
        generation,
        node_path: path(indices),
        outcome,
        completed: None,
        expected: None,
        remaining: None,
        frame,
        failure: None,
    }
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
