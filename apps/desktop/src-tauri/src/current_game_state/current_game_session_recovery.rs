use super::*;
use app_model::{CandidateMoveDto, CurrentGameErrorKind, MoveVertex, NodePath, PointDto};
use sgf::SgfAnalysisPayload;

const BRANCHING: &str = include_str!("../../../../../tests/golden/java-analysis-branching.sgf");
const EMPTY: &str = "(;GM[1]FF[4]SZ[19]KM[7.5]PB[黑]PW[白])";

fn path(indices: &[u32]) -> NodePath {
    NodePath {
        indices: indices.to_vec(),
    }
}

fn payload() -> SgfAnalysisPayload {
    SgfAnalysisPayload {
        engine_name: "KataGo".to_string(),
        visits: 400,
        winrate_black: 0.61,
        score_mean_black: Some(2.25),
        score_stdev: Some(0.5),
        pda: None,
        candidates: vec![CandidateMoveDto {
            vertex: MoveVertex::Point(PointDto { x: 3, y: 3 }),
            visits: 400,
            winrate_black: 0.61,
            score_mean_black: 2.25,
            policy_prior: Some(0.4),
            pv: vec![MoveVertex::Point(PointDto { x: 3, y: 3 })],
        }],
        ownership: None,
    }
}

#[test]
fn restore_installs_tree_analysis_comment_cursor_source_and_dirty() {
    let source = CurrentGameState::default();
    source
        .replace(BRANCHING, Some("/tmp/branching.sgf".to_string()))
        .unwrap();
    let commented = source
        .set_personal_comment(path(&[]), "restored personal".to_string())
        .unwrap();
    let attached = source
        .attach_primary_analysis(commented.generation, path(&[]), payload())
        .unwrap();
    let selected = source.select_path(path(&[0])).unwrap();
    assert_eq!(attached.generation, commented.generation);
    assert!(attached.dirty);

    let envelope = source.take_due_recovery_write(u64::MAX).expect("snapshot");
    assert!(envelope.sgf_text.contains("C[restored personal]"));
    assert!(envelope.sgf_text.contains("LZOP[KataGo"));
    assert_eq!(envelope.selected_path, selected.selected_path);
    assert_eq!(envelope.source_path.as_deref(), Some("/tmp/branching.sgf"));
    assert!(envelope.dirty);

    let target = CurrentGameState::default();
    target.replace(EMPTY, None).unwrap();
    let restored = target.restore_envelope(envelope).unwrap();
    assert_eq!(restored.selected_path, path(&[0]));
    assert_eq!(restored.snapshot.personal_comment, "move comment");
    let root = target.select_path(path(&[])).unwrap();
    assert_eq!(root.snapshot.personal_comment, "restored personal");
    assert_eq!(root.snapshot.primary_analysis.unwrap().visits, 400);
    assert_eq!(restored.native_path.as_deref(), Some("/tmp/branching.sgf"));
    assert!(restored.dirty);
    assert!(target.serialize().unwrap().contains("C[restored personal]"));
    assert!(target.serialize().unwrap().contains("LZOP[KataGo"));
}

#[test]
fn restore_validation_failure_keeps_the_current_game() {
    let state = CurrentGameState::default();
    let opened = state
        .replace(EMPTY, Some("/tmp/current.sgf".to_string()))
        .unwrap();
    let before = state.serialize().unwrap();
    let bad = RecoveryEnvelopeDto {
        document_seq: 1,
        snapshot_seq: 1,
        sgf_text: "not-sgf".to_string(),
        selected_path: path(&[]),
        source_path: Some("/tmp/other.sgf".to_string()),
        dirty: true,
        disposition: ApplicationExitDispositionDto::ExitIncomplete,
    };
    let err = state.restore_envelope(bad).unwrap_err();
    assert_eq!(err.kind, CurrentGameErrorKind::MalformedSgf);
    assert_eq!(state.serialize().unwrap(), before);
    assert_eq!(
        state.inspect(),
        (opened.generation, false, opened.native_path.clone(), Some(before))
    );
}

#[test]
fn analysis_and_cursor_changes_schedule_recovery_without_generation_bump() {
    let state = CurrentGameState::default();
    let opened = state
        .replace(BRANCHING, Some("/tmp/branching.sgf".to_string()))
        .unwrap();
    let first = state.take_due_recovery_write(u64::MAX).expect("open snapshot");
    state.finish_recovery_write(first, Ok(()));

    let attached = state
        .attach_primary_analysis(opened.generation, path(&[]), payload())
        .unwrap();
    assert_eq!(attached.generation, opened.generation);
    assert!(attached.dirty);
    let analysis = state
        .take_due_recovery_write(u64::MAX)
        .expect("analysis snapshot");
    assert!(analysis.sgf_text.contains("LZOP[KataGo"));
    assert!(analysis.dirty);
    state.finish_recovery_write(analysis, Ok(()));

    let selected = state.select_path(path(&[0])).unwrap();
    let cursor = state.take_due_recovery_write(u64::MAX).expect("cursor snapshot");
    assert_eq!(cursor.selected_path, selected.selected_path);
}
