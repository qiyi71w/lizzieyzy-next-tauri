use super::*;
use std::sync::{mpsc, Barrier};

fn source(id: &str) -> EngineDiagnosticSnapshotDto {
    EngineDiagnosticSnapshotDto {
        attempt_id: id.into(),
        run_id: id.into(),
        profile_id: "profile".into(),
        captured_at_ms: 8,
        full_trace: false,
        command: "program=[private-1]".into(),
        failure: None,
        stdout_complete: true,
        stderr_complete: true,
        process_exited: true,
        exit_code: Some(1),
        records: vec![],
        dropped_records: 0,
        retained_bytes: 0,
        metrics: vec![],
    }
}
fn directory() -> PathBuf {
    let path = std::env::temp_dir().join(format!("diagnostic-export-barrier-{}", uuid::Uuid::new_v4()));
    fs::create_dir(&path).unwrap();
    path
}
fn wait(owner: &DiagnosticExport, phase: Phase) -> DiagnosticExportStatusDto {
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        let status = owner.status();
        if status.phase == phase {
            return status;
        }
        assert!(Instant::now() < deadline, "{phase:?}: {status:?}");
        std::thread::sleep(Duration::from_millis(2));
    }
}
fn held(phase: Phase) -> (DiagnosticExport, mpsc::Receiver<()>, Arc<Barrier>) {
    let mut owner = DiagnosticExport::new(directory());
    let barrier = Arc::new(Barrier::new(2));
    let release = barrier.clone();
    let once = AtomicBool::new(false);
    let (send, receive) = mpsc::sync_channel(1);
    Arc::get_mut(&mut owner.inner).unwrap().checkpoint = Some(Arc::new(move |stage| {
        if stage == phase && !once.swap(true, Ordering::AcqRel) {
            send.send(()).unwrap();
            release.wait();
        }
        Ok(())
    }));
    (owner, receive, barrier)
}
#[test]
fn latest_pending_estimate_replaces_old_generation_without_retargeting_export() {
    let (owner, reached, release) = held(Phase::Estimating);
    let old = owner.estimate(source("old-a")).unwrap();
    reached.recv_timeout(Duration::from_secs(1)).unwrap();
    owner.estimate(source("discarded-b")).unwrap();
    let latest = owner.estimate(source("latest-c")).unwrap();
    assert_eq!(owner.status().phase, Phase::EstimateObsolete);
    assert!(owner.export(old).is_err());
    release.wait();
    let ready = wait(&owner, Phase::Ready);
    assert_eq!(ready.generation, latest);
    assert_eq!(ready.attempt_id.as_deref(), Some("latest-c"));
    owner.export(latest).unwrap();
    let done = wait(&owner, Phase::Completed);
    owner.cancel(latest).unwrap();
    assert_eq!(owner.status().phase, Phase::Completed);
    assert!(owner.completed_directory(latest).is_ok());
    assert!(owner.shutdown(Duration::from_secs(1)));
    assert!(owner.inner.directory.join(done.file_name.unwrap()).is_file());
    fs::remove_dir_all(&owner.inner.directory).unwrap();
}
#[test]
fn cancel_before_publication_has_no_final_and_only_prunes_owned_staging() {
    for phase in [
        Phase::Collecting,
        Phase::Archiving,
        Phase::Syncing,
        Phase::Publishing,
    ] {
        let (owner, reached, release) = held(phase);
        let unrelated = owner.inner.directory.join("human-note.partial");
        fs::write(&unrelated, b"keep").unwrap();
        let generation = owner.estimate(source("frozen-a")).unwrap();
        wait(&owner, Phase::Ready);
        owner.export(generation).unwrap();
        reached.recv_timeout(Duration::from_secs(1)).unwrap();
        assert!(owner.estimate(source("retry-b")).is_err());
        owner.cancel(generation).unwrap();
        assert_eq!(owner.status().phase, Phase::Cancelling);
        release.wait();
        wait(&owner, Phase::Cancelled);
        assert!(owner.shutdown(Duration::from_secs(1)));
        assert_eq!(fs::read_dir(&owner.inner.directory).unwrap().count(), 1);
        assert_eq!(fs::read(&unrelated).unwrap(), b"keep");
        fs::remove_dir_all(&owner.inner.directory).unwrap();
    }
}
#[test]
fn each_failed_stage_keeps_final_absent_and_reports_its_stage() {
    for failed in [
        Phase::Estimating,
        Phase::Collecting,
        Phase::Archiving,
        Phase::Syncing,
        Phase::Publishing,
    ] {
        let mut owner = DiagnosticExport::new(directory());
        Arc::get_mut(&mut owner.inner).unwrap().checkpoint = Some(Arc::new(move |stage| {
            if stage == failed {
                Err("Injected filesystem stage failure".into())
            } else {
                Ok(())
            }
        }));
        let generation = owner.estimate(source("failed-a")).unwrap();
        if failed != Phase::Estimating {
            wait(&owner, Phase::Ready);
            owner.export(generation).unwrap();
        }
        let result = wait(&owner, Phase::Failed);
        assert_eq!(result.failed_stage, Some(failed));
        assert!(result.file_name.is_none());
        assert!(owner.shutdown(Duration::from_secs(1)));
        assert_eq!(fs::read_dir(&owner.inner.directory).unwrap().count(), 0);
        fs::remove_dir_all(&owner.inner.directory).unwrap();
    }
}
#[test]
fn shutdown_has_a_bounded_outstanding_result_and_never_reopens_admission() {
    let (owner, reached, release) = held(Phase::Publishing);
    let generation = owner.estimate(source("exiting-a")).unwrap();
    wait(&owner, Phase::Ready);
    owner.export(generation).unwrap();
    reached.recv_timeout(Duration::from_secs(1)).unwrap();
    let start = Instant::now();
    assert!(!owner.shutdown(Duration::from_millis(20)));
    assert!(start.elapsed() < Duration::from_millis(500));
    assert!(owner.estimate(source("late-b")).is_err());
    assert!(owner.export(generation).is_err());
    release.wait();
    wait(&owner, Phase::Closed);
    assert!(owner.shutdown(Duration::from_secs(1)));
    assert_eq!(fs::read_dir(&owner.inner.directory).unwrap().count(), 0);
    fs::remove_dir_all(&owner.inner.directory).unwrap();
}
#[test]
fn pruning_retains_replacements_and_unknown_identity_and_removes_the_owned_file() {
    let dir = directory();
    let path = dir.join("owned.partial");
    let original = File::create(&path).unwrap();
    let mut owned = OwnedTemporary::new(path.clone(), &original);
    assert!(
        owned.identity.is_some(),
        "this platform must provide identity for this test"
    );
    fs::rename(&path, dir.join("moved-original")).unwrap();
    let replacement = File::create(&path).unwrap();
    let replacement_identity = file_identity(&replacement).unwrap();
    owned.identity.as_mut().unwrap().created = replacement_identity.created;
    assert_ne!(
        replacement_identity.id,
        owned.identity.as_ref().unwrap().id,
        "same creation time is not identity"
    );
    assert!(!owned.prune());
    assert!(path.exists());
    assert!(!OwnedTemporary {
        path: path.clone(),
        identity: None
    }
    .prune());
    let actual = OwnedTemporary::new(path.clone(), &replacement);
    drop(replacement);
    assert!(actual.prune());
    drop(original);
    fs::remove_dir_all(dir).unwrap();
}
#[test]
fn oversized_source_is_rejected_without_worker_or_filesystem_changes() {
    let owner = DiagnosticExport::new(directory());
    let mut input = source("too-large");
    input.command = "x".repeat(crate::diagnostics::RECORD_BYTES + 1);
    assert!(owner.estimate(input).is_err());
    assert_eq!(owner.status().phase, Phase::Idle);
    assert_eq!(fs::read_dir(&owner.inner.directory).unwrap().count(), 0);
    fs::remove_dir_all(&owner.inner.directory).unwrap();
}
