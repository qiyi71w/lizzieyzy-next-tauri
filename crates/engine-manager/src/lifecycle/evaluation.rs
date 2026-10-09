use super::*;
use app_model::{EvaluationPhaseDto as PhaseDto, EvaluationResultDto, EvaluationSnapshotDto};
use sha2::{Digest, Sha256};
use std::io::Read;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc::SyncSender, Weak};

// Direct KataGo owns no subprocess tree. Bound its wall time, pipes, and retained output.
const OUTPUT_BYTES: usize = 8192;
const LINE_BYTES: usize = 1024;
const QUEUE_LINES: usize = 64;
const MEASUREMENT_TIMEOUT: Duration = Duration::from_secs(120);
const CLEANUP_TIMEOUT: Duration = Duration::from_secs(2);

struct EvaluationWorker(Arc<AtomicBool>);
impl Drop for EvaluationWorker {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}

pub(super) struct EvaluationSlot {
    saved: SavedEngineProfile,
    snapshot: EvaluationSnapshotDto,
    child: Option<Child>,
    sanitizer: crate::diagnostics::DiagnosticSanitizer,
}

impl Drop for EvaluationSlot {
    fn drop(&mut self) {
        let _ = self.reap();
    }
}

impl EvaluationSlot {
    fn active(&self) -> bool {
        matches!(self.snapshot.phase, PhaseDto::Qualifying | PhaseDto::Running)
    }

    fn reap(&mut self) -> Result<(), EngineFailureDto> {
        let Some(child) = self.child.as_mut() else {
            return Ok(());
        };
        let _ = child.kill();
        let deadline = Instant::now() + CLEANUP_TIMEOUT;
        loop {
            match child.try_wait() {
                Ok(Some(status)) => {
                    self.snapshot.exit_code = status.code();
                    self.snapshot.process_id = None;
                    self.child = None;
                    return Ok(());
                }
                _ if Instant::now() >= deadline => {
                    return Err(eval_failure(
                        "Benchmark process cleanup exceeded two seconds; retry Cancel",
                    ));
                }
                _ => thread::sleep(Duration::from_millis(10)),
            }
        }
    }

    fn retire(&mut self, phase: PhaseDto, message: FailureText) -> Result<(), EngineFailureDto> {
        self.snapshot.phase = phase;
        self.snapshot.result = None;
        self.snapshot.output.clear();
        self.snapshot.message = Some(message.into_string());
        if let Err(error) = self.reap() {
            self.snapshot.phase = PhaseDto::Failed;
            self.snapshot.message = Some(error.message.clone());
            return Err(error);
        }
        Ok(())
    }
}

fn eval_failure(message: &'static str) -> EngineFailureDto {
    failure(
        EngineOperationDto::Job,
        EngineFailureKind::InvalidState,
        message.into(),
        None,
        None,
        None,
    )
}

pub(super) fn yield_to_foreground(state: &mut ManagerState) -> Result<(), EngineFailureDto> {
    if let Some(slot) = state
        .evaluation
        .as_mut()
        .filter(|slot| slot.active() || slot.child.is_some())
    {
        slot.retire(
            PhaseDto::Yielded,
            "Benchmark yielded to foreground analysis or Match".into(),
        )?;
    }
    Ok(())
}

pub(super) fn close(state: &mut ManagerState) -> Result<(), EngineFailureDto> {
    state.startup_evaluation = None;
    if let Some(slot) = state.evaluation.as_mut() {
        slot.retire(
            PhaseDto::Cancelled,
            "Benchmark closed; session-only output and result retired".into(),
        )?;
    }
    Ok(())
}

fn busy(state: &ManagerState) -> bool {
    state.match_reservation.is_some()
        || state.game_move.is_some()
        || state.finite_admission_pending
        || state.continuous_departing
        || state.jobs.iter().any(|job| !job.terminal)
        || !matches!(state.phase, Phase::NoEngine { .. } | Phase::Ready(_))
}

impl ForegroundEngineManager {
    /// Consumes only authorization captured at manager construction, never a later save.
    pub fn apply_startup_evaluation(&self) {
        let request = self.lock().startup_evaluation.take();
        let result = match request {
            None | Some(Ok(None)) => return,
            Some(Err(message)) => Err(failure(EngineOperationDto::Job, EngineFailureKind::InvalidState,
                FailureText::unscoped(&message), None, None, None)),
            Some(Ok(Some(saved))) => self.start_saved_evaluation(Ok(saved), true),
        };
        if let Err(error) = result {
            let mut state = self.lock();
            if state.evaluation.is_none() {
                state.evaluation_notice = EvaluationSnapshotDto {
                    phase: if busy(&state) { PhaseDto::Yielded } else { PhaseDto::Unavailable },
                    message: Some(error.message),
                    ..Default::default()
                };
            }
        }
    }

    pub fn evaluation_snapshot(&self) -> EvaluationSnapshotDto {
        let mut state = self.lock();
        if let Some(slot) = state.evaluation.as_mut() {
            if self.inner.catalog.get(&slot.saved.profile_id).as_ref() != Some(&slot.saved) {
                let _ = slot.retire(PhaseDto::Retired, "Saved benchmark target changed or was deleted".into());
            }
            return slot.snapshot.clone();
        }
        state.evaluation_notice.clone()
    }

    pub fn cancel_evaluation(&self, evaluation_id: &str) -> Result<EvaluationSnapshotDto, EngineFailureDto> {
        let mut state = self.lock();
        if let Some(slot) = state
            .evaluation
            .as_mut()
            .filter(|slot| slot.snapshot.evaluation_id.as_deref() == Some(evaluation_id))
        {
            slot.retire(
                PhaseDto::Cancelled,
                "Benchmark cancelled; no settings were changed".into(),
            )?;
            return Ok(slot.snapshot.clone());
        }
        Err(eval_failure("Benchmark identity is no longer current"))
    }

    pub fn start_evaluation(&self, profile_id: &str) -> Result<EvaluationSnapshotDto, EngineFailureDto> {
        self.lock().startup_evaluation = None;
        let saved = self.inner.catalog.get(profile_id)
            .ok_or_else(|| eval_failure("Select an existing saved local profile to benchmark"));
        self.start_saved_evaluation(saved, false)
    }

    fn start_saved_evaluation(&self, saved: Result<SavedEngineProfile, EngineFailureDto>, startup: bool) -> Result<EvaluationSnapshotDto, EngineFailureDto> {
        {
            let mut state = self.lock();
            if let Some(old) = state.evaluation.as_mut() {
                old.retire(PhaseDto::Retired, "Replaced by a new explicit benchmark".into())?;
            }
        }
        let deadline = Instant::now() + CLEANUP_TIMEOUT;
        while self.inner.evaluation_worker.load(Ordering::Acquire) {
            if Instant::now() >= deadline {
                return Err(eval_failure(
                    "Previous benchmark qualification is retiring; retry explicitly after cleanup",
                ));
            }
            thread::sleep(Duration::from_millis(10));
        }
        self.inner
            .evaluation_worker
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map_err(|_| eval_failure("Another benchmark admission is in progress"))?;
        let worker = EvaluationWorker(Arc::clone(&self.inner.evaluation_worker));
        let saved = saved?;
        if self.inner.catalog.get(&saved.profile_id).as_ref() != Some(&saved) {
            return Err(eval_failure("Startup evaluation unavailable: captured saved target changed or was deleted"));
        }
        let mut spec = build_command_spec(&saved.profile)
            .map_err(|_| eval_failure("Benchmark target has invalid or missing local resources"))?;
        // Only the explicit KataGo subcommand changes. All user argv, ordering,
        // repeated configs, overrides and working directory remain untouched.
        if !matches!(
            spec.args.first().map(String::as_str),
            Some("analysis" | "gtp" | "benchmark")
        ) {
            return Err(eval_failure("Unsupported benchmark target: direct-local KataGo only; remote, Java SSH and external SSH are unsupported"));
        }
        let executable = std::path::Path::new(&spec.program)
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if matches!(
            executable.as_str(),
            "ssh" | "java" | "bash" | "sh" | "cmd" | "powershell" | "pwsh"
        ) {
            return Err(eval_failure(
                "Remote and wrapper benchmark targets are unsupported",
            ));
        }
        let revision = format!(
            "{:x}",
            Sha256::digest(serde_json::to_vec(&saved.profile).unwrap())
        );
        let id = Uuid::new_v4().to_string();
        let snapshot = EvaluationSnapshotDto {
            evaluation_id: Some(id.clone()),
            target_id: Some(saved.profile_id.clone()),
            input_revision: Some(revision),
            phase: PhaseDto::Qualifying,
            ..EvaluationSnapshotDto::default()
        };
        let autoload_run = {
            let mut state = self.lock();
            let autoload_run = match &state.phase {
                Phase::Starting(run) if startup && state.operation_kind == EngineOperationDto::Autoload => Some(run.run_id.clone()),
                _ => None,
            };
            if busy(&state) && autoload_run.is_none() {
                return Err(eval_failure(
                    "Foreground analysis or Match has priority; benchmark is unavailable while busy",
                ));
            }
            state.evaluation = Some(EvaluationSlot {
                saved: saved.clone(),
                snapshot: snapshot.clone(),
                child: None,
                sanitizer: crate::diagnostics::DiagnosticSanitizer::default(),
            });
            autoload_run
        };
        let weak = Arc::downgrade(&self.inner);
        thread::spawn(move || {
            let _worker = worker;
            let result = wait_for_startup(&weak, &id, autoload_run.as_deref())
                .and_then(|()| execute(&weak, &id, &saved, &mut spec));
            if let Err(message) = result {
                if let Some(inner) = weak.upgrade() {
                    let mut state = inner.lock();
                    if let Some(slot) = current(&mut state, &id) {
                        let message = FailureText::dynamic(&mut slot.sanitizer, &message);
                        let _ = slot.retire(PhaseDto::Failed, message);
                    }
                }
            }
        });
        Ok(snapshot)
    }
}

fn wait_for_startup(weak: &Weak<Inner>, id: &str, autoload_run: Option<&str>) -> Result<(), String> {
    let Some(run_id) = autoload_run else { return Ok(()) };
    let deadline = Instant::now() + weak.upgrade().ok_or("Benchmark manager closed")?.config.readiness_timeout;
    loop {
        let inner = weak.upgrade().ok_or("Benchmark manager closed")?;
        let mut state = inner.lock();
        if current(&mut state, id).is_none() { return Err("Benchmark retired".into()); }
        match &state.phase {
            Phase::Starting(run) if run.run_id == run_id && Instant::now() < deadline => {},
            Phase::Ready(run) if run.run_id == run_id => return Ok(()),
            Phase::NoEngine { .. } => return Ok(()),
            _ => {
                yield_to_foreground(&mut state).map_err(|error| error.message)?;
                return Err("Startup benchmark yielded during foreground transition".into());
            }
        }
        drop(state);
        drop(inner);
        thread::sleep(Duration::from_millis(10));
    }
}

fn current<'a>(state: &'a mut ManagerState, id: &str) -> Option<&'a mut EvaluationSlot> {
    state
        .evaluation
        .as_mut()
        .filter(|slot| slot.active() && slot.snapshot.evaluation_id.as_deref() == Some(id))
}

fn execute(
    weak: &Weak<Inner>,
    id: &str,
    saved: &SavedEngineProfile,
    spec: &mut crate::CommandSpec,
) -> Result<(), String> {
    let managed_root = weak.upgrade().ok_or("Benchmark manager closed")?.config.managed_resources_root.clone();
    let mut evaluation_run = starting_run(saved);
    evaluation_run.run_id = id.into();
    // This capture belongs only to the evaluation; it is not a Foreground Run attempt.
    let diagnostics = crate::diagnostics::AttemptCapture::new(&evaluation_run);
    let is_retired = || {
        let Some(inner) = weak.upgrade() else { return true; };
        current(&mut inner.lock(), id).is_none()
            || inner.catalog.get(&saved.profile_id).as_ref() != Some(saved)
    };
    let resources = ResourceSnapshot::capture_evaluation(
        &evaluation_run,
        spec,
        Instant::now() + Duration::from_secs(30),
        diagnostics,
        managed_root.as_deref(),
        &is_retired,
    )
    .map_err(|(_, message)| message)?;
    let probe = crate::CommandSpec {
        program: spec.program.clone(),
        args: vec!["version".into()],
        working_dir: spec.working_dir.clone(),
        env: spec.env.clone(),
    };
    let (probe_exit, version_lines, _) = run_process(weak, id, &probe, Duration::from_secs(10), false)?;
    if probe_exit != Some(0) {
        return Err("Benchmark binary version probe failed".into());
    }
    let version = version_lines
        .iter()
        .find(|line| line.trim() == "KataGo v1.18.2")
        .ok_or("Unsupported benchmark binary: KataGo 1.18.2 has not been confirmed")?
        .clone();
    let backend = version_lines
        .iter()
        .find(|line| line.starts_with("Using ") && line.ends_with(" backend"))
        .cloned();
    resources
        .revalidate(Instant::now() + Duration::from_secs(30), &is_retired)
        .map_err(|(_, message)| message)?;
    spec.args[0] = "benchmark".into();
    let (exit_code, output, elapsed) = run_process(weak, id, spec, MEASUREMENT_TIMEOUT, true)?;
    if exit_code != Some(0) {
        return Err(format!(
            "Benchmark failed (exit code {exit_code:?}); no measurement available. {}",
            output.join("\n")
        ));
    }
    if !output.iter().any(|line| completed_row(line).is_some()) {
        return Err("Benchmark exited without a completed measurement; result unavailable".into());
    }
    resources
        .revalidate(Instant::now() + Duration::from_secs(30), &is_retired)
        .map_err(|(_, message)| message)?;
    let qualified = resources.qualified(Some(version), backend);
    let inner = weak.upgrade().ok_or("Benchmark manager closed")?;
    let mut state = inner.lock();
    let slot = current(&mut state, id).ok_or("Benchmark retired")?;
    if inner.catalog.get(&saved.profile_id).as_ref() != Some(saved) {
        let _ = slot.retire(PhaseDto::Retired, "Saved benchmark target changed or was deleted".into());
        return Ok(());
    }
    slot.snapshot.result = Some(EvaluationResultDto {
        target_id: saved.profile_id.clone(),
        input_revision: qualified.profile_revision.clone(),
        qualified_resource: qualified,
        elapsed_ms: elapsed.as_millis() as u64,
        search_visits_per_second: search_speed(&output),
    });
    slot.snapshot.phase = PhaseDto::Completed;
    slot.snapshot.message = Some(
        "Actual benchmark result; missing metrics are unavailable. No recommendation was saved or applied"
            .into(),
    );
    Ok(())
}

// A single reported search speed is meaningful. A multi-thread sweep has multiple
// observations; do not invent a scalar recommendation by selecting its fastest row.
fn search_speed(lines: &[String]) -> Option<f64> {
    let values: Vec<f64> = lines
        .iter()
        .filter_map(|line| {
            let rest = completed_row(line)?;
            rest.split_whitespace()
                .next()?
                .parse::<f64>()
                .ok()
                .filter(|value| value.is_finite() && *value > 0.0)
        })
        .collect();
    (values.len() == 1).then(|| values[0])
}

fn completed_row(line: &str) -> Option<&str> {
    let line = line.trim().strip_prefix("numSearchThreads =")?;
    let (threads, row) = line.split_once(':')?;
    if threads.trim().parse::<u32>().ok()? == 0 {
        return None;
    }
    let (progress, metric) = row.split_once(" positions, visits/s = ")?;
    let (done, total) = progress.split_once('/')?;
    let total = total.trim().parse::<u32>().ok()?;
    (total > 0 && done.trim().parse::<u32>().ok()? == total).then_some(metric)
}

struct OutputLine {
    text: String,
    truncated: bool,
}
fn read_output(mut reader: impl Read + Send + 'static, tx: SyncSender<Result<OutputLine, ()>>) {
    thread::spawn(move || {
        let mut bytes = [0u8; 1024];
        let mut line = Vec::with_capacity(LINE_BYTES);
        let mut truncated = false;
        loop {
            let count = match reader.read(&mut bytes) {
                Ok(n) => n,
                Err(_) => {
                    let _ = tx.send(Err(()));
                    return;
                }
            };
            for &byte in &bytes[..count] {
                if byte == b'\n' || byte == b'\r' {
                    if tx
                        .send(Ok(OutputLine {
                            text: String::from_utf8_lossy(&line).into_owned(),
                            truncated,
                        }))
                        .is_err()
                    {
                        return;
                    }
                    line.clear();
                    truncated = false;
                } else if line.len() < LINE_BYTES {
                    line.push(byte);
                } else {
                    truncated = true;
                }
            }
            if count == 0 {
                if !line.is_empty() {
                    let _ = tx.send(Ok(OutputLine {
                        text: String::from_utf8_lossy(&line).into_owned(),
                        truncated,
                    }));
                }
                return;
            }
        }
    });
}

fn run_process(
    weak: &Weak<Inner>,
    id: &str,
    spec: &crate::CommandSpec,
    timeout: Duration,
    measurement: bool,
) -> Result<(Option<i32>, Vec<String>, Duration), String> {
    let (tx, rx) = mpsc::sync_channel(QUEUE_LINES);
    {
        let inner = weak.upgrade().ok_or("Benchmark manager closed")?;
        let mut state = inner.lock();
        if busy(&state) {
            yield_to_foreground(&mut state).map_err(|error| error.message)?;
            return Err("Foreground work has priority".into());
        }
        let slot = current(&mut state, id).ok_or("Benchmark retired")?;
        if inner.catalog.get(&slot.saved.profile_id).as_ref() != Some(&slot.saved) {
            return Err("Saved benchmark target changed or was deleted".into());
        }
        let mut child = build_process_command(spec)
            .spawn()
            .map_err(|_| "Unable to start direct-local benchmark executable")?;
        child.stdin.take();
        read_output(child.stdout.take().unwrap(), tx.clone());
        read_output(child.stderr.take().unwrap(), tx);
        slot.snapshot.process_id = Some(child.id());
        slot.child = Some(child);
        if measurement {
            slot.snapshot.phase = PhaseDto::Running;
            slot.snapshot.exit_code = None;
            slot.snapshot.output.clear();
        }
    }
    let start = Instant::now();
    let mut raw = Vec::<String>::new();
    let mut raw_bytes = 0;
    let mut exited = None;
    loop {
        let line = rx.recv_timeout(Duration::from_millis(20));
        let disconnected = matches!(line, Err(mpsc::RecvTimeoutError::Disconnected));
        let inner = weak.upgrade().ok_or("Benchmark manager closed")?;
        let mut state = inner.lock();
        let slot = current(&mut state, id).ok_or("Benchmark retired")?;
        if inner.catalog.get(&slot.saved.profile_id).as_ref() != Some(&slot.saved) {
            let _ = slot.retire(PhaseDto::Retired, "Saved benchmark target changed or was deleted".into());
            return Err("Benchmark retired".into());
        }
        if let Ok(line) = line {
            let line = line.map_err(|_| "Unable to read benchmark output; measurement unavailable")?;
            if measurement {
                slot.snapshot.output_truncated |= line.truncated;
                // Sanitize only the display copy; completed-row parsing consumes raw bytes below.
                slot.snapshot.output.push(slot.sanitizer.sanitize(&line.text));
                while slot.snapshot.output.iter().map(String::len).sum::<usize>() > OUTPUT_BYTES
                    || slot.snapshot.output.len() > 128
                {
                    slot.snapshot.output.remove(0);
                    slot.snapshot.output_truncated = true;
                }
            }
            raw_bytes += line.text.len();
            raw.push(line.text);
            while raw_bytes > OUTPUT_BYTES || raw.len() > 128 {
                raw_bytes -= raw.remove(0).len();
                slot.snapshot.output_truncated = true;
            }
        }
        if let Some(child) = slot.child.as_mut() {
            if let Some(status) = child.try_wait().map_err(|_| "Unable to observe benchmark exit")? {
                exited = Some(status.code());
                slot.child = None;
                slot.snapshot.process_id = None;
                slot.snapshot.exit_code = status.code();
            }
        }
        if let Some(code) = exited {
            if disconnected {
                return Ok((code, raw, start.elapsed()));
            }
        }
        if start.elapsed() >= timeout {
            return Err("Benchmark timed out; no measurement available".into());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FailedPipe;

    impl Read for FailedPipe {
        fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other("controlled pipe failure"))
        }
    }

    #[test]
    fn pipe_read_failure_after_output_is_not_successful_eof() {
        let row = b"numSearchThreads = 1: 1/1 positions, visits/s = 2.5\n";
        let (tx, rx) = mpsc::sync_channel(QUEUE_LINES);
        read_output(std::io::Cursor::new(row).chain(FailedPipe), tx);
        let first = rx.recv_timeout(Duration::from_secs(1)).unwrap().unwrap();
        assert!(completed_row(&first.text).is_some());
        assert!(rx.recv_timeout(Duration::from_secs(1)).unwrap().is_err());
        assert!(matches!(
            rx.recv_timeout(Duration::from_secs(1)),
            Err(mpsc::RecvTimeoutError::Disconnected)
        ));

        let (tx, rx) = mpsc::sync_channel(QUEUE_LINES);
        read_output(std::io::Cursor::new(row), tx);
        assert!(rx.recv_timeout(Duration::from_secs(1)).unwrap().is_ok());
        assert!(matches!(
            rx.recv_timeout(Duration::from_secs(1)),
            Err(mpsc::RecvTimeoutError::Disconnected)
        ));
    }
}
