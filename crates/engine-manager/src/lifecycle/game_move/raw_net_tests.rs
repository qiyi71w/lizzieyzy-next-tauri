//! Controlled transport tests. Injected resource identity is not producer qualification.
use super::*;
use app_model::*;
use crate::{InMemoryEngineProfileCatalog, SavedEngineProfile};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

struct Rig { manager: ForegroundEngineManager, dir: PathBuf, run: String }
impl Rig {
    fn new() -> Self { Self::with_mode("valid") }
    fn with_mode(mode: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("raw-net-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("mode"), mode).unwrap();
        std::fs::write(dir.join("model"), "controlled model").unwrap();
        std::fs::write(dir.join("config"), "numSearchThreads=1\n").unwrap();
        let executable = dir.join("engine");
        std::fs::write(&executable, format!("#!/bin/sh\nexec /usr/bin/python3 '{}/tests/fixtures/rules_process.py' \"$@\"\n", env!("CARGO_MANIFEST_DIR"))).unwrap();
        std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o755)).unwrap();
        let catalog = Arc::new(InMemoryEngineProfileCatalog::new());
        catalog.upsert(SavedEngineProfile { profile_id: "raw".into(), measured_scenes: Default::default(), thread_policy: Default::default(), profile: EngineProfileDto {
            name: "controlled raw transport".into(), program: executable.to_string_lossy().into(), argv: vec![], working_dir: Some(dir.to_string_lossy().into()),
            adapter: EngineAdapterSettings::KataGoGtp(KataGoSettings { model_path: Some("model".into()), config_path: Some("config".into()), max_visits: 4 }),
        }});
        let manager = ForegroundEngineManager::new(catalog, ForegroundEngineConfig::for_tests());
        manager.set_continuous_preferences(false, Default::default()).unwrap();
        manager.start("raw").unwrap();
        let until = Instant::now() + Duration::from_secs(5);
        let run = loop {
            match manager.snapshot().lifecycle {
                ForegroundEngineLifecycleDto::Ready { run } => break run.run_id,
                ForegroundEngineLifecycleDto::Error { failure, .. } => panic!("{failure:?}"),
                _ => assert!(Instant::now() < until),
            }
            thread::sleep(Duration::from_millis(2));
        };
        // Unit seam: only this controlled manager receives the admitted resource identity.
        // Production admission remains pinned to actual bytes; no test bypass is shipped.
        {
            let mut state = manager.lock();
            let Phase::Ready(run) = &mut state.phase else { unreachable!() };
            let resource = Arc::make_mut(run.qualified_resource.as_mut().unwrap());
            for item in &mut resource.resources {
                if item.component == "executable" { item.sha256 = "20c4ede9f39455ab3c019ada7a22a6e4d7696361161feed5c56f2c42d1507bd0".into(); }
                if item.component == "model" { item.sha256 = "7c8a84ed9ee737e9c7e741a08bf242d63db37b648e7f64942f3a8b1b5101e7c2".into(); }
            }
        }
        Self { manager, dir, run }
    }
    fn request(&self) -> GameMoveRequest {
        GameMoveRequest {
            identity: GameMoveRequestDto { run_id: self.run.clone(), generation: 7, node_path: NodePath::default(), budget: ComputeBudgetDto { deadline_ms: 3000, max_visits: None } },
            position: sgf::CurrentSgfDocument::open("(;SZ[9]RU[Chinese]KM[7.5])").unwrap().exact_position(&NodePath::default()).unwrap(),
        }
    }
    fn start(&self) -> RawNetHandle { self.manager.start_raw_net(self.request(), 12, 7).unwrap() }
    fn mode(&self, mode: &str) { std::fs::write(self.dir.join("raw-mode"), mode).unwrap(); }
    fn held(&self) {
        let until = Instant::now() + Duration::from_secs(5);
        while !self.dir.join("raw-held").exists() { assert!(Instant::now() < until); thread::sleep(Duration::from_millis(2)); }
    }
    fn release(&self) { std::fs::write(self.dir.join("raw-release"), "").unwrap(); }
    fn trace(&self) -> String { std::fs::read_to_string(self.dir.join("trace")).unwrap() }
}
impl Drop for Rig {
    fn drop(&mut self) { let _ = self.manager.teardown(); let _ = std::fs::remove_dir_all(&self.dir); }
}

#[test]
fn raw_complete_payload_owns_identity_and_never_becomes_ordinary_frame() {
    let rig = Rig::new();
    let result = rig.start().wait().unwrap();
    assert_eq!(result.ownership.as_ref().unwrap(), &vec![-0.25; 81]);
    assert_eq!(result.document_identity, 12);
    assert_eq!(result.board_revision, 7);
    assert_eq!(result.confirmation.identity.run_id, rig.run);
    assert_eq!(result.confirmation.identity.generation, 7);
    assert_eq!(result.confirmation.position.to_play, PlayerColor::Black);
    assert_eq!(result.slot, RawNetSlotDto::MainBoard);
    let value = serde_json::to_value(&result).unwrap();
    assert!(value.get("frame").is_none() && value.get("visits").is_none() && value.get("candidates").is_none());
    rig.manager.claim_ordinary_rules(&result.confirmation).unwrap();
    assert!(rig.manager.claim_ordinary_rules(&result.confirmation).is_err());
    let trace = rig.trace();
    assert!(trace.find("kata-get-rules").unwrap() < trace.find("kata-raw-nn 0").unwrap());
    assert!(!trace.contains("kata-analyze") && !trace.contains("genmove"));
    assert_eq!(rig.manager.snapshot().continuous.phase, ContinuousAnalysisPhaseDto::Off);
}

#[test]
fn raw_missing_invalid_and_rejected_remain_unavailable_without_killing_run() {
    let rig = Rig::new();
    for mode in ["missing", "invalid"] {
        rig.mode(mode);
        let result = rig.start().wait().unwrap();
        assert!(result.ownership.is_none() && result.unavailable.is_some());
    }
    rig.mode("rejected");
    assert!(rig.start().wait().is_err());
    assert!(matches!(rig.manager.snapshot().lifecycle, ForegroundEngineLifecycleDto::Ready { .. }));
    rig.mode("valid");
    assert!(rig.start().wait().unwrap().ownership.is_some());
}

#[test]
fn raw_written_retirement_holds_occupancy_until_complete_final_and_never_revives() {
    let rig = Rig::new();
    rig.mode("hold-final");
    let handle = rig.start();
    rig.held();
    assert!(rig.manager.snapshot().game_move_job.is_some());
    rig.manager.invalidate_game_move_position(8, &NodePath::default());
    rig.manager.invalidate_game_move_position(7, &NodePath::default());
    assert!(rig.manager.start_raw_net(rig.request(), 12, 7).is_err());
    assert!(rig.manager.start_ordinary_rules(rig.request()).is_err());
    rig.release();
    assert_eq!(handle.wait().unwrap_err().kind, EngineFailureKind::Cancellation);
    assert!(rig.manager.snapshot().game_move_job.is_none());
    assert!(matches!(rig.manager.snapshot().lifecycle, ForegroundEngineLifecycleDto::Ready { .. }));
    rig.mode("valid");
    assert!(rig.start().wait().unwrap().ownership.is_some());
}

#[test]
fn raw_queued_retirement_writes_nothing_and_completed_old_request_cannot_publish() {
    let rig = Rig::new();
    let stdin = engine_stdin(&rig.manager.lock(), &rig.run).unwrap();
    let guard = stdin.lock().unwrap();
    let before = rig.trace();
    let handle = rig.start();
    rig.manager.cancel_current_game_move();
    drop(guard);
    assert!(handle.wait().is_err());
    assert_eq!(rig.trace(), before);
    assert!(matches!(rig.manager.snapshot().lifecycle, ForegroundEngineLifecycleDto::Ready { .. }));
    let old = rig.start().wait().unwrap();
    let new = rig.start().wait().unwrap();
    assert!(rig.manager.claim_ordinary_rules(&old.confirmation).is_err());
    rig.manager.claim_ordinary_rules(&new.confirmation).unwrap();
    let old = rig.start().wait().unwrap();
    rig.manager.teardown().unwrap();
    assert!(rig.manager.claim_ordinary_rules(&old.confirmation).is_err());
}

#[test]
fn raw_unconfirmed_or_unqualified_never_writes_raw_and_timeout_retires_run() {
    let rejected = Rig::with_mode("negative");
    assert!(rejected.start().wait().is_err());
    assert!(!rejected.trace().contains("kata-raw-nn"));
    let rig = Rig::new();
    {
        let mut state = rig.manager.lock();
        let Phase::Ready(run) = &mut state.phase else { unreachable!() };
        Arc::make_mut(run.qualified_resource.as_mut().unwrap()).resources.clear();
    }
    assert!(rig.manager.start_raw_net(rig.request(), 12, 7).is_err());
    assert!(!rig.trace().contains("kata-raw-nn"));
    let timeout = Rig::new();
    timeout.mode("timeout");
    let mut request = timeout.request(); request.identity.budget.deadline_ms = 200;
    assert!(timeout.manager.start_raw_net(request, 12, 7).unwrap().wait().is_err());
    assert!(matches!(timeout.manager.snapshot().lifecycle, ForegroundEngineLifecycleDto::Error { .. }));
}

#[test]
fn raw_refuses_finite_admission_and_leased_measurement_without_commands() {
    let rig = Rig::new();
    let before = rig.trace();
    rig.manager.lock().finite_admission_pending = true;
    assert!(rig.manager.start_raw_net(rig.request(), 12, 7).is_err());
    rig.manager.lock().finite_admission_pending = false;
    let lease = rig.manager.begin_setup_operation().unwrap();
    lease.begin_row("raw", MeasuredSceneDto::Live, 1, false,
        [("numSearchThreads".into(), 1), ("nnMaxBatchSize".into(), 1)].into()).unwrap();
    assert!(rig.manager.start_raw_net(rig.request(), 12, 7).is_err());
    assert_eq!(rig.trace(), before);
    assert!(rig.manager.snapshot().game_move_job.is_none());
    drop(lease);
    assert!(rig.start().wait().unwrap().ownership.is_some());
}

#[test]
fn raw_eof_before_final_never_publishes_or_frees_a_live_run() {
    let rig = Rig::new();
    rig.mode("eof");
    assert!(rig.start().wait().is_err());
    assert!(matches!(rig.manager.snapshot().lifecycle, ForegroundEngineLifecycleDto::Error { .. }));
    assert!(rig.manager.start_raw_net(rig.request(), 12, 7).is_err());
}

#[test]
fn raw_refuses_ordinary_and_match_ownership_and_old_run_publication() {
    let rig = Rig::with_mode("analysis");
    let events = rig.manager.subscribe();
    let request = crate::SelectedNodeJobRequest {
        run_id: rig.run.clone(), generation: 7, node_path: NodePath::default(),
        mode: AnalysisJobModeDto::Continuous, board_width: 9, board_height: 9,
        position_empty: true, exact_position: Ok(rig.request().position),
        query: katago_protocol::analysis_query_from_position(9, 9, 7.5, &[], PlayerColor::Black,
            katago_protocol::AnalysisQueryOptions { id: "pending".into(), rules: "chinese".into(), turn: 0,
                max_visits: None, include_ownership: Some(true), include_policy: Some(true) }).unwrap(),
    };
    let ordinary = rig.manager.start_selected_node_job(request).unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let ForegroundEngineEventDto::Job { job } = events.recv_timeout(deadline.saturating_duration_since(Instant::now())).unwrap() {
            if job.job_id == ordinary.job_id && job.outcome == AnalysisJobOutcomeDto::Progress { break; }
        }
    }
    let before = rig.trace();
    assert!(rig.manager.start_raw_net(rig.request(), 12, 7).is_err());
    assert_eq!(rig.trace(), before);
    assert_eq!(rig.manager.snapshot().selected_node_job.unwrap().job_id, ordinary.job_id);
    rig.manager.teardown().unwrap();

    let reserved = Rig::new();
    reserved.manager.reserve_match("raw-refusal").unwrap();
    let before = reserved.trace();
    assert!(reserved.manager.start_raw_net(reserved.request(), 12, 7).is_err());
    assert_eq!(reserved.trace(), before);

    let replaced = Rig::new();
    let old = replaced.start().wait().unwrap();
    replaced.manager.restart().unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let ForegroundEngineLifecycleDto::Ready { run } = replaced.manager.snapshot().lifecycle {
            if run.run_id != replaced.run { break; }
        }
        assert!(Instant::now() < deadline);
        thread::sleep(Duration::from_millis(2));
    }
    assert!(replaced.manager.claim_ordinary_rules(&old.confirmation).is_err());
    assert!(replaced.manager.start_raw_net(replaced.request(), 12, 7).is_err());
}
