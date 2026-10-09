use app_model::{DiagnosticExportPhaseDto as Phase, EngineDiagnosticRecordDto, EngineDiagnosticSnapshotDto};
use engine_manager::diagnostic_export::DiagnosticExport;
use engine_manager::diagnostics::DiagnosticSanitizer;
use std::{
    fs,
    io::Read,
    time::{Duration, Instant},
};

fn source(id: &str) -> EngineDiagnosticSnapshotDto {
    let mut sanitizer = DiagnosticSanitizer::default();
    let text = sanitizer.sanitize("WARN roomId=424242\nthreadId=42 running\npassword=\"FAKE two words\nFAKE second line\"\nINFO \u{1b}ready\u{0}\nWARN https://fake.invalid/path?token=FAKE-url");
    let command = sanitizer.sanitize("C:\\Users\\FAKE Private User\\katago.exe");
    let failure = sanitizer.sanitize("WARN roomId=424242");
    EngineDiagnosticSnapshotDto {
        attempt_id: id.into(),
        run_id: id.into(),
        profile_id: "profile-a".into(),
        captured_at_ms: 42,
        full_trace: false,
        command,
        failure: Some(failure),
        stdout_complete: false,
        stderr_complete: false,
        process_exited: false,
        exit_code: None,
        retained_bytes: text.len(),
        records: vec![EngineDiagnosticRecordDto {
            sequence: 7,
            at_ms: 41,
            source: "stderr".into(),
            text,
        }],
        dropped_records: 0,
        metrics: vec![],
    }
}
fn wait(owner: &DiagnosticExport, phase: Phase) -> app_model::DiagnosticExportStatusDto {
    let end = Instant::now() + Duration::from_secs(5);
    loop {
        let status = owner.status();
        if status.phase == phase {
            return status;
        }
        assert!(Instant::now() < end, "waiting for {phase:?}: {status:?}");
        std::thread::sleep(Duration::from_millis(5));
    }
}
#[test]
fn frozen_source_publishes_complete_private_zip_without_retargeting() {
    let dir = std::env::temp_dir().join(format!("diagnostic-export-test-{}", uuid::Uuid::new_v4()));
    fs::create_dir(&dir).unwrap();
    let owner = DiagnosticExport::new(dir.clone());
    let mut displayed = source("attempt-a");
    let generation = owner.estimate(displayed.clone()).unwrap();
    displayed.attempt_id = "attempt-b".into();
    let ready = wait(&owner, Phase::Ready);
    assert_eq!(ready.attempt_id.as_deref(), Some("attempt-a"));
    assert_eq!(ready.generation, generation);
    owner.export(generation).unwrap();
    let done = wait(&owner, Phase::Completed);
    let path = dir.join(done.file_name.unwrap());
    let mut zip = zip::ZipArchive::new(fs::File::open(&path).unwrap()).unwrap();
    assert_eq!(zip.len(), 4);
    let mut snapshot = String::new();
    zip.by_name("snapshot.json")
        .unwrap()
        .read_to_string(&mut snapshot)
        .unwrap();
    let mut records = String::new();
    zip.by_name("records.jsonl")
        .unwrap()
        .read_to_string(&mut records)
        .unwrap();
    assert_eq!(
        records.lines().count(),
        1,
        "multiline data remains one JSONL record"
    );
    for data in [&snapshot, &records] {
        for secret in [
            "FAKE",
            "424242",
            "Private User",
            "fake.invalid",
            "\u{1b}",
            "\u{0}",
        ] {
            assert!(!data.contains(secret), "leaked {secret}");
        }
        assert!(data.contains("threadId=42 running"));
        assert!(data.contains("[private-1]"));
    }
    assert!(snapshot.contains("attempt-a"));
    assert!(!snapshot.contains("attempt-b"));
    assert_eq!(fs::read_dir(&dir).unwrap().count(), 1);
    assert!(owner.shutdown(Duration::from_secs(1)));
    drop(zip);
    fs::remove_dir_all(dir).unwrap();
}
