use super::*;
use app_model::{ApplicationExitDispositionDto, NodePath, RecoveryProtectionDto, RecoveryStartupDto};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const EMPTY: &str = "(;GM[1]FF[4]SZ[19]KM[7.5]PB[黑]PW[白])";
const BRANCH: &str = "(;GM[1]FF[4]SZ[19]KM[7.5]PB[黑]PW[白];B[pd])";
const BRANCH_COMMENT: &str = "(;GM[1]FF[4]SZ[19]KM[7.5]PB[黑]PW[白]C[root personal];B[pd])";

fn path(indices: &[u32]) -> NodePath {
    NodePath {
        indices: indices.to_vec(),
    }
}

fn snapshot(
    document_seq: u64,
    snapshot_seq: u64,
    sgf: &str,
    selected: &[u32],
    dirty: bool,
) -> RecoverySnapshot {
    RecoverySnapshot {
        document_seq,
        snapshot_seq,
        sgf_text: sgf.to_string(),
        selected_path: path(selected),
        source_path: Some("/tmp/game.sgf".to_string()),
        dirty,
    }
}

fn commit(coord: &mut RecoveryCoordinator, envelope: RecoveryEnvelopeDto) {
    coord.finish_write(envelope.clone(), Ok(()));
}

#[test]
fn first_outstanding_change_starts_write_within_one_second() {
    let mut coord = RecoveryCoordinator::default();
    coord.note(snapshot(1, 1, EMPTY, &[], true), 0);
    assert!(coord.take_due_write(999).is_none());
    let due = coord.take_due_write(1000).expect("write due at one second");
    assert_eq!(due.sgf_text, EMPTY);
    assert_eq!(due.snapshot_seq, 1);
    assert!(due.dirty);
    assert_eq!(due.disposition, ApplicationExitDispositionDto::ExitIncomplete);
}

#[test]
fn continuous_changes_keep_the_latest_pending_without_postponing() {
    let mut coord = RecoveryCoordinator::default();
    coord.note(snapshot(1, 1, EMPTY, &[], true), 0);
    coord.note(snapshot(1, 2, BRANCH, &[0], true), 400);
    coord.note(snapshot(1, 3, BRANCH_COMMENT, &[0], true), 900);
    assert!(coord.take_due_write(999).is_none());
    let due = coord.take_due_write(1000).expect("original deadline");
    assert_eq!(due.sgf_text, BRANCH_COMMENT);
    assert_eq!(due.selected_path, path(&[0]));
    assert_eq!(due.snapshot_seq, 3);
}

#[test]
fn busy_writer_keeps_latest_pending_and_writes_it_immediately_afterward() {
    let mut coord = RecoveryCoordinator::default();
    coord.note(snapshot(1, 1, EMPTY, &[], true), 0);
    let first = coord.take_due_write(1000).unwrap();
    coord.note(snapshot(1, 2, BRANCH, &[0], true), 1100);
    coord.note(snapshot(1, 3, BRANCH_COMMENT, &[0], true), 1200);
    assert!(coord.take_due_write(2000).is_none());
    commit(&mut coord, first);
    let next = coord
        .take_due_write(1200)
        .expect("pending write starts immediately");
    assert_eq!(next.sgf_text, BRANCH_COMMENT);
    assert_eq!(next.snapshot_seq, 3);
}

#[test]
fn write_failure_keeps_last_success_and_retry_rewrites_latest() {
    let mut coord = RecoveryCoordinator::default();
    coord.note(snapshot(1, 1, EMPTY, &[], true), 0);
    let first = coord.take_due_write(1000).unwrap();
    commit(&mut coord, first);
    coord.note(snapshot(1, 2, BRANCH, &[0], true), 1500);
    let failed = coord.take_due_write(2500).unwrap();
    coord.finish_write(failed, Err("disk full".to_string()));
    assert_eq!(
        coord.protection(),
        RecoveryProtectionDto::Unprotected {
            message: UNPROTECTED_WRITE_MESSAGE.to_string()
        }
    );
    assert_eq!(coord.last_committed().unwrap().sgf_text, EMPTY);
    let retry = coord.retry(2600).expect("explicit retry");
    assert_eq!(retry.sgf_text, BRANCH);
    commit(&mut coord, retry);
    assert_eq!(coord.protection(), RecoveryProtectionDto::Protected);
    assert_eq!(coord.last_committed().unwrap().sgf_text, BRANCH);
}

#[test]
fn later_change_after_write_failure_schedules_another_write() {
    let mut coord = RecoveryCoordinator::default();
    coord.note(snapshot(1, 1, EMPTY, &[], true), 0);
    let first = coord.take_due_write(1000).unwrap();
    commit(&mut coord, first);
    coord.note(snapshot(1, 2, BRANCH, &[0], true), 1500);
    let failed = coord.take_due_write(2500).unwrap();
    coord.finish_write(failed, Err("disk full".to_string()));
    coord.note(snapshot(1, 3, BRANCH_COMMENT, &[0], true), 2600);
    assert!(coord.take_due_write(3599).is_none());
    let retried = coord
        .take_due_write(3600)
        .expect("later change retries within one second");
    assert_eq!(retried.sgf_text, BRANCH_COMMENT);
    assert_eq!(retried.snapshot_seq, 3);
    assert_eq!(coord.last_committed().unwrap().sgf_text, EMPTY);
}

#[test]
fn analysis_only_and_already_dirty_snapshots_still_schedule() {
    let mut coord = RecoveryCoordinator::default();
    coord.note(snapshot(1, 1, EMPTY, &[], true), 0);
    let first = coord.take_due_write(1000).unwrap();
    commit(&mut coord, first);
    coord.note(snapshot(1, 2, BRANCH_COMMENT, &[], true), 1500);
    let due = coord.take_due_write(2500).unwrap();
    assert_eq!(due.sgf_text, BRANCH_COMMENT);
    assert!(due.dirty);
    assert_eq!(due.document_seq, 1);
}

#[test]
fn older_document_snapshot_cannot_overwrite_replaced_document() {
    let mut coord = RecoveryCoordinator::default();
    coord.note(snapshot(1, 1, EMPTY, &[], true), 0);
    let old = coord.take_due_write(1000).unwrap();
    coord.note(snapshot(2, 1, BRANCH, &[], false), 1100);
    commit(&mut coord, old);
    let replacement = coord.take_due_write(1100).unwrap();
    assert_eq!(replacement.document_seq, 2);
    assert_eq!(replacement.sgf_text, BRANCH);
    commit(&mut coord, replacement);
    coord.note(snapshot(1, 9, EMPTY, &[], true), 3000);
    assert!(coord.take_due_write(4000).is_none());
    assert_eq!(coord.last_committed().unwrap().sgf_text, BRANCH);
}

#[test]
fn committed_discard_is_not_revived_by_a_late_writer() {
    let mut coord = RecoveryCoordinator::default();
    coord.note(snapshot(1, 1, EMPTY, &[], true), 0);
    let late = coord.take_due_write(1000).unwrap();
    let discard = coord
        .take_flush_write(
            snapshot(1, 2, EMPTY, &[], true),
            ApplicationExitDispositionDto::ExplicitDiscard,
        )
        .unwrap();
    commit(&mut coord, discard);
    coord.finish_write(late, Ok(()));
    assert_eq!(
        coord.last_committed().unwrap().disposition,
        ApplicationExitDispositionDto::ExplicitDiscard
    );
    coord.note(snapshot(1, 3, BRANCH, &[], true), 2000);
    assert!(coord.take_due_write(3000).is_none());
}

#[test]
fn flush_writes_immediately_including_clean_exit_without_pending_edits() {
    let mut coord = RecoveryCoordinator::default();
    coord.note(snapshot(1, 1, EMPTY, &[], false), 0);
    let first = coord.take_due_write(1000).unwrap();
    commit(&mut coord, first);
    let flushed = coord
        .take_flush_write(
            snapshot(1, 1, EMPTY, &[], false),
            ApplicationExitDispositionDto::CleanCompleted,
        )
        .unwrap();
    assert_eq!(flushed.disposition, ApplicationExitDispositionDto::CleanCompleted);
    commit(&mut coord, flushed);
    assert_eq!(
        coord.last_committed().unwrap().disposition,
        ApplicationExitDispositionDto::CleanCompleted
    );
}

#[test]
fn failed_flush_does_not_report_success_and_keeps_retryable_error() {
    let mut coord = RecoveryCoordinator::default();
    let flushed = coord
        .take_flush_write(
            snapshot(1, 1, EMPTY, &[], false),
            ApplicationExitDispositionDto::CleanCompleted,
        )
        .unwrap();
    coord.finish_write(flushed, Err("disk full".to_string()));
    assert_eq!(
        coord.protection(),
        RecoveryProtectionDto::Unprotected {
            message: UNPROTECTED_WRITE_MESSAGE.to_string()
        }
    );
    assert!(coord.last_committed().is_none());
    let retry = coord.retry(0).expect("retry flush");
    assert_eq!(retry.disposition, ApplicationExitDispositionDto::CleanCompleted);
}

#[test]
fn startup_classifies_abnormal_normal_discarded_and_unreadable() {
    let abnormal = envelope_from_snapshot(
        snapshot(1, 1, EMPTY, &[], true),
        ApplicationExitDispositionDto::ExitIncomplete,
    );
    assert!(matches!(
        classify_startup(Ok(Some(abnormal))),
        RecoveryStartupDto::Abnormal { .. }
    ));
    let clean = envelope_from_snapshot(
        snapshot(1, 1, EMPTY, &[], false),
        ApplicationExitDispositionDto::CleanCompleted,
    );
    assert!(matches!(
        classify_startup(Ok(Some(clean))),
        RecoveryStartupDto::Normal { .. }
    ));
    let discarded = envelope_from_snapshot(
        snapshot(1, 1, EMPTY, &[], true),
        ApplicationExitDispositionDto::ExplicitDiscard,
    );
    assert!(matches!(
        classify_startup(Ok(Some(discarded))),
        RecoveryStartupDto::None
    ));
    assert!(matches!(classify_startup(Ok(None)), RecoveryStartupDto::None));
    assert!(matches!(
        classify_startup(Err("nope".to_string())),
        RecoveryStartupDto::Unreadable { .. }
    ));
}

#[test]
fn restore_validates_sgf_and_cursor_and_rejects_corrupt_payload() {
    let valid = envelope_from_snapshot(
        snapshot(1, 1, EMPTY, &[], true),
        ApplicationExitDispositionDto::ExitIncomplete,
    );
    validate_envelope(&valid).unwrap();

    let bad_path = RecoveryEnvelopeDto {
        selected_path: path(&[9]),
        ..valid.clone()
    };
    assert!(validate_envelope(&bad_path).is_err());

    let malformed = RecoveryEnvelopeDto {
        sgf_text: "not-sgf".to_string(),
        ..valid
    };
    assert!(validate_envelope(&malformed).is_err());
}

#[test]
fn file_store_atomic_replace_failure_keeps_previous_envelope() {
    let (dir, path) = temp_recovery();
    let store = FileRecoveryStore::new(path.clone());
    let first = envelope_from_snapshot(
        snapshot(1, 1, EMPTY, &[], true),
        ApplicationExitDispositionDto::ExitIncomplete,
    );
    store.replace(&first).unwrap();
    let before = fs::read_to_string(&path).unwrap();

    let newer = envelope_from_snapshot(
        snapshot(1, 2, BRANCH, &[0], true),
        ApplicationExitDispositionDto::ExitIncomplete,
    );
    let err = atomic_replace_json_with(&path, &newer, |_from, _to| {
        Err(std::io::Error::other("rename boom"))
    })
    .unwrap_err();
    assert!(err.contains("replace"));
    assert_eq!(fs::read_to_string(&path).unwrap(), before);
    assert_eq!(store.load().unwrap().unwrap().sgf_text, EMPTY);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn file_store_rejects_stale_document_after_newer_replace() {
    let (dir, path) = temp_recovery();
    let store = FileRecoveryStore::new(path);
    let newer = envelope_from_snapshot(
        snapshot(2, 1, BRANCH, &[], false),
        ApplicationExitDispositionDto::ExitIncomplete,
    );
    store.replace(&newer).unwrap();
    let older = envelope_from_snapshot(
        snapshot(1, 9, EMPTY, &[], true),
        ApplicationExitDispositionDto::ExitIncomplete,
    );
    store.replace(&older).unwrap();
    assert_eq!(store.load().unwrap().unwrap().sgf_text, BRANCH);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn file_store_rejects_late_incomplete_after_clean_or_discard() {
    let (dir, path) = temp_recovery();
    let store = FileRecoveryStore::new(path);
    let clean = envelope_from_snapshot(
        snapshot(1, 2, EMPTY, &[], false),
        ApplicationExitDispositionDto::CleanCompleted,
    );
    store.replace(&clean).unwrap();
    let late = envelope_from_snapshot(
        snapshot(1, 2, BRANCH, &[], true),
        ApplicationExitDispositionDto::ExitIncomplete,
    );
    store.replace(&late).unwrap();
    assert_eq!(
        store.load().unwrap().unwrap().disposition,
        ApplicationExitDispositionDto::CleanCompleted
    );

    let discarded = envelope_from_snapshot(
        snapshot(2, 1, BRANCH, &[], true),
        ApplicationExitDispositionDto::ExplicitDiscard,
    );
    store.replace(&discarded).unwrap();
    let revived = envelope_from_snapshot(
        snapshot(2, 3, BRANCH_COMMENT, &[0], true),
        ApplicationExitDispositionDto::ExitIncomplete,
    );
    store.replace(&revived).unwrap();
    assert_eq!(
        store.load().unwrap().unwrap().disposition,
        ApplicationExitDispositionDto::ExplicitDiscard
    );
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn load_checkpoint_seeds_discard_without_making_it_a_startup_candidate() {
    let mut coord = RecoveryCoordinator::default();
    let discarded = envelope_from_snapshot(
        snapshot(4, 1, EMPTY, &[], true),
        ApplicationExitDispositionDto::ExplicitDiscard,
    );
    coord.load_checkpoint(discarded);
    coord.note(snapshot(4, 2, BRANCH, &[], true), 0);
    assert!(coord.take_due_write(1000).is_none());
    coord.note(snapshot(5, 1, BRANCH, &[], false), 0);
    let next = coord.take_due_write(1000).expect("newer document after discard");
    assert_eq!(next.document_seq, 5);
}

fn temp_recovery() -> (PathBuf, PathBuf) {
    let unique = format!(
        "{}-{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
    );
    let dir = std::env::temp_dir().join("lizzieyzy-recovery").join(unique);
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join(RECOVERY_FILE);
    (dir, path)
}

fn atomic_replace_json_with(
    path: &Path,
    envelope: &RecoveryEnvelopeDto,
    rename: impl FnOnce(&Path, &Path) -> std::io::Result<()>,
) -> Result<(), String> {
    let json = serde_json::to_string_pretty(envelope).unwrap();
    let tmp = {
        let mut tmp = path.as_os_str().to_os_string();
        tmp.push(".tmp");
        PathBuf::from(tmp)
    };
    fs::write(&tmp, json).unwrap();
    match rename(&tmp, path) {
        Ok(()) => Ok(()),
        Err(err) => {
            let _ = fs::remove_file(&tmp);
            Err(format!("failed to replace {}: {err}", path.display()))
        }
    }
}
