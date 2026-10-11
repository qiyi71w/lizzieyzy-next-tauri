//! Session-only private SGF documents. Engine execution stays in the existing finite task lane.
use super::*;
use app_model::*;
use sha2::{Digest, Sha256};
use sgf::CurrentSgfDocument;
use std::path::{Path, PathBuf};
use parking_lot::Mutex;

pub(super) struct Row {
    dto: BatchRowDto,
    document: Option<CurrentSgfDocument>,
    gate: Arc<Mutex<bool>>,
    preview_digest: Option<String>,
}
pub(super) struct Owner {
    pub snapshot: BatchSnapshotDto,
    rows: Vec<Row>,
    pub current: Option<BatchIdentityDto>,
    run_id: Option<String>,
    worker: bool,
    closed: bool,
    next_row: u64,
    review_allowed: bool,
}
impl Default for Owner {
    fn default() -> Self {
        Self { snapshot: BatchSnapshotDto { batch_id: Uuid::new_v4().simple().to_string(), rows: vec![], notices: vec![], started: false, paused: false, reason: None, output_directory: None }, rows: vec![], current: None, run_id: None, worker: false, closed: false, next_row: 1, review_allowed: true }
    }
}
fn row_mut<'a>(owner: &'a mut Owner, id: &BatchIdentityDto) -> Result<&'a mut Row, String> {
    owner.rows.iter_mut().find(|r| &r.dto.identity == id).ok_or_else(|| "Stale batch / row / attempt identity.".into())
}
fn pending(row: &Row) -> bool { matches!(row.dto.state, BatchRowStateDto::Pending | BatchRowStateDto::Waiting) }
pub(super) fn occupied(owner: &Owner) -> bool {
    owner.current.is_some()
        || owner.snapshot.started && !owner.snapshot.paused && owner.rows.iter().any(pending)
}
fn receipt(bytes: &[u8]) -> String { format!("sha256:{:x}", Sha256::digest(bytes)) }
fn read_document(path: &str) -> Result<(CurrentSgfDocument, String), String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{path}: {e}"))?;
    let digest = receipt(&bytes);
    let text = std::str::from_utf8(&bytes).map_err(|e| e.to_string())?;
    Ok((CurrentSgfDocument::open(text).map_err(|e| e.to_string())?, digest))
}
fn resolve(document: &CurrentSgfDocument, conditions: &BatchPreferencesDto) -> Result<(AnalysisScopePreviewDto, Vec<SelectedNodeSnapshotDto>), String> {
    let (nodes, supporting, comparisons) = if conditions.strategy == AnalysisTaskStrategyDto::SwingSelectedTwoStage {
        let resolved = document.swing_analysis_scope(&conditions.scope, conditions.swing_criteria.move_actors).map_err(|e| e.to_string())?;
        (resolved.requested, resolved.supporting, resolved.comparisons)
    } else { (document.analysis_scope_snapshots(&conditions.scope).map_err(|e| e.to_string())?, vec![], vec![]) };
    let target = |s: &SelectedNodeSnapshotDto| AnalysisScopeTargetDto { node_path: s.path.clone(), move_number: s.position.move_number, to_play: s.position.to_play };
    let preview = AnalysisScopePreviewDto { generation: 0, scope: conditions.scope.clone(), targets: nodes.iter().map(target).collect(), supporting_targets: supporting.iter().map(target).collect(), swing_comparisons: comparisons, swing_criteria: (conditions.strategy == AnalysisTaskStrategyDto::SwingSelectedTwoStage).then(|| conditions.swing_criteria.clone()) };
    let mut all = nodes;
    all.extend(supporting);
    Ok((preview, all))
}

impl ForegroundEngineManager {
    /// Stable owner discrimination also rejects late private frames after a new batch begins.
    pub fn is_batch_job(&self, job_id: &str) -> bool { job_id.starts_with("batch:") }
    /// Future explicit consumers must refuse this occupancy; Ready is not permission to steal it.
    pub fn batch_occupied(&self) -> bool { occupied(&self.lock().batch) }
    pub fn set_batch_review_access(&self, allowed: bool) {
        let changed = {
            let mut state = self.lock();
            let changed = state.batch.review_allowed != allowed;
            state.batch.review_allowed = allowed;
            if changed && !allowed && state.batch.snapshot.started { pause(&mut state.batch, "Authority changed; explicit Continue is required."); }
            changed
        };
        if changed && !allowed {
            if let Err(error) = self.batch_stop(false) { pause(&mut self.lock().batch, &error); }
        }
    }
    pub fn retire_batch_for_authority(&self) -> Result<(), String> {
        self.set_batch_review_access(false);
        pause(&mut self.lock().batch, "Authority changed; explicitly Continue after returning to review.");
        self.batch_stop(false)?;
        let deadline = Instant::now() + self.inner.config.stop_drain_timeout;
        while self.batch_occupied() {
            if Instant::now() >= deadline { return Err("Batch retirement did not drain.".into()); }
            thread::sleep(Duration::from_millis(10));
        }
        Ok(())
    }
    pub fn shutdown_batch(&self, budget: Duration) -> bool {
        self.lock().batch.closed = true;
        if self.batch_stop(true).is_err() { return false; }
        let deadline = Instant::now() + budget;
        loop {
            let state = self.lock();
            if !state.batch.worker && state.batch.current.is_none() && !state.batch.rows.iter().any(|r| r.dto.output_state == BatchOutputStateDto::Saving) { return true; }
            drop(state);
            if Instant::now() >= deadline { return false; }
            thread::sleep(Duration::from_millis(10));
        }
    }
    pub fn batch_snapshot(&self) -> BatchSnapshotDto {
        let state = self.lock();
        let mut snapshot = state.batch.snapshot.clone();
        snapshot.rows = state.batch.rows.iter().map(|r| r.dto.clone()).collect();
        snapshot
    }
    pub fn batch_command(&self, command: BatchCommandDto) -> Result<BatchSnapshotDto, String> {
        if self.lock().batch.closed { return Err("Application exit has sealed the batch queue.".into()); }
        match command {
            BatchCommandDto::Intake { paths, conditions } => self.batch_intake(paths, conditions)?,
            BatchCommandDto::Start { output } => {
                self.retire_auto_load().map_err(|e| e.message)?;
                let mut state = self.lock();
                if state.batch.snapshot.started { return Err("This batch already started; use Continue or create a new batch.".into()); }
                let directory = match output {
                    BatchOutputAuthorizationDto::SessionOnly => None,
                    BatchOutputAuthorizationDto::Directory { path, confirmed_names } => {
                        let expected: Vec<_> = state.batch.rows.iter().map(|r| r.dto.output_name.clone()).collect();
                        if expected != confirmed_names { return Err("Output naming preview changed; confirm it again.".into()); }
                        let directory = std::fs::canonicalize(path).map_err(|e| e.to_string())?;
                        if !directory.is_dir() { return Err("Choose an output directory.".into()); }
                        Some(directory.to_string_lossy().into_owned())
                    }
                };
                state.batch.snapshot.output_directory = directory;
                state.batch.snapshot.started = true;
                state.batch.snapshot.paused = false;
                state.batch.run_id = current_admitting_run(&state.phase).map(|r| r.run_id);
                if state.batch.run_id.is_none() { pause(&mut state.batch, "No Ready whole-game Run; explicitly Continue when available."); }
            }
            BatchCommandDto::Continue => {
                self.retire_auto_load().map_err(|e| e.message)?;
                let mut state = self.lock();
                if !state.batch.review_allowed { return Err("Return to local review before continuing the batch.".into()); }
                if !state.batch.snapshot.started { return Err("Choose output authorization and Start first.".into()); }
                if state.batch.current.is_some() { return Err("Wait for the current row to drain.".into()); }
                let run = current_admitting_run(&state.phase).ok_or("A Ready Run is required.")?;
                validate_whole_game_capabilities(&run, None).map_err(|e| e.message)?;
                if state.continuous_safety_hold || state.continuous_error { return Err("Resolve the engine safety hold before continuing the batch.".into()); }
                state.batch.run_id = Some(run.run_id);
                state.batch.snapshot.paused = false;
                state.batch.snapshot.reason = None;
            }
            BatchCommandDto::Pause => { pause(&mut self.lock().batch, "Dispatch paused; pending files retained."); }
            BatchCommandDto::StopCurrent => self.batch_stop(false)?,
            BatchCommandDto::StopAll => self.batch_stop(true)?,
            BatchCommandDto::Remove { identity } => {
                let mut state = self.lock();
                if !pending(row_mut(&mut state.batch, &identity)?) { return Err("Only pending rows may be removed.".into()); }
                state.batch.rows.retain(|r| r.dto.identity != identity);
            }
            BatchCommandDto::Reorder { row_ids } => {
                let mut state = self.lock();
                let slots: Vec<_> = state.batch.rows.iter().enumerate().filter(|(_, r)| pending(r)).map(|(i, _)| i).collect();
                let mut existing: Vec<_> = slots.iter().map(|&i| state.batch.rows[i].dto.identity.row_id.clone()).collect();
                let mut wanted = row_ids.clone(); existing.sort(); wanted.sort();
                if existing != wanted { return Err("Reorder must name every pending row exactly once.".into()); }
                for (slot, id) in slots.into_iter().zip(row_ids) {
                    let from = state.batch.rows.iter().position(|r| r.dto.identity.row_id == id).ok_or("Stale row order")?;
                    state.batch.rows.swap(slot, from);
                }
            }
            BatchCommandDto::Rerun { identity } => {
                let mut state = self.lock();
                let row = row_mut(&mut state.batch, &identity)?;
                if row.dto.state != BatchRowStateDto::Failed || row.dto.output_state == BatchOutputStateDto::Saving { return Err("Only a failed, non-saving row may be explicitly rerun.".into()); }
                row.dto.identity.attempt += 1;
                row.dto.state = BatchRowStateDto::Pending;
                row.dto.task = None; row.dto.reason = None; row.dto.document_id = None;
                row.dto.has_result = false; row.dto.calculation_complete = false; row.dto.incomplete = false;
                row.dto.output_state = BatchOutputStateDto::Unsaved; row.dto.saved_path = None;
                row.document = None; row.gate = Arc::new(Mutex::new(false));
                pause(&mut state.batch, "New attempt queued. Explicitly Continue to dispatch.");
            }
            BatchCommandDto::Save { identity, path, overwrite, incomplete } => self.batch_save(&identity, Path::new(&path), overwrite, incomplete, true)?,
            BatchCommandDto::NewBatch => {
                let mut state = self.lock();
                if state.batch.current.is_some() || state.batch.worker || state.batch.rows.iter().any(|r| r.dto.output_state == BatchOutputStateDto::Saving || pending(r)) { return Err("Stop all and finish pending writes before creating a new batch.".into()); }
                if state.batch.rows.iter().any(|r| r.dto.has_result && r.dto.output_state != BatchOutputStateDto::Saved) { return Err("Unsaved session results remain; save them or exit with explicit loss confirmation.".into()); }
                let review_allowed = state.batch.review_allowed;
                state.batch = Owner { review_allowed, ..Owner::default() };
            }
        }
        self.kick_batch();
        Ok(self.batch_snapshot())
    }
    fn batch_intake(&self, paths: Vec<String>, conditions: BatchPreferencesDto) -> Result<(), String> {
        conditions.validate()?;
        let batch_id = self.lock().batch.snapshot.batch_id.clone();
        // Preview I/O is not an analysis snapshot. Every row reads again only when admitted.
        for path in paths {
            let canonical = std::fs::canonicalize(&path);
            let source = canonical.as_ref().map(|p| p.to_string_lossy().into_owned()).unwrap_or_else(|_| path.clone());
            let parsed = if !Path::new(&source).extension().and_then(|v| v.to_str()).is_some_and(|v| v.eq_ignore_ascii_case("sgf")) { Err("Batch accepts SGF files only.".into()) }
                else if let Err(error) = canonical { Err(error.to_string()) }
                else { read_document(&source).and_then(|(document, digest)| resolve(&document, &conditions).map(|(preview, _)| (preview, digest))) };
            let mut state = self.lock();
            if state.batch.closed || state.batch.snapshot.batch_id != batch_id { return Err("The intake batch retired before its file preview completed.".into()); }
            if state.batch.rows.iter().any(|r| r.dto.source_path == source) { state.batch.snapshot.notices.push(format!("Duplicate path retained only once: {source}")); continue; }
            let number = state.batch.next_row;
            state.batch.next_row += 1;
            let stem = Path::new(&source).file_stem().unwrap_or_default().to_string_lossy();
            let id = BatchIdentityDto { batch_id: state.batch.snapshot.batch_id.clone(), row_id: Uuid::new_v4().to_string(), attempt: 1 };
            let name = format!("{stem}_已分析_{}_{number}.sgf", id.batch_id);
            let (preview, digest, reason) = match parsed { Ok((p, d)) => (Some(p), Some(d), None), Err(e) => (None, None, Some(e)) };
            state.batch.rows.push(Row { dto: BatchRowDto { identity: id, source_path: source, output_name: name, state: if reason.is_some() { BatchRowStateDto::Failed } else { BatchRowStateDto::Pending }, conditions: conditions.clone(), preview, task: None, document_id: None, input_receipt: None, calculation_complete: false, has_result: false, output_state: BatchOutputStateDto::Unsaved, saved_path: None, incomplete: false, reason }, document: None, gate: Arc::new(Mutex::new(false)), preview_digest: digest });
        }
        Ok(())
    }
    fn batch_stop(&self, all: bool) -> Result<(), String> {
        let mut state = self.lock();
        if all { pause(&mut state.batch, "Stop all: unstarted rows cancelled; published files retained."); }
        let current = state.batch.current.clone();
        let mut task = None;
        for row in &mut state.batch.rows {
            if current.as_ref() == Some(&row.dto.identity) || all && (pending(row) || row.dto.output_state == BatchOutputStateDto::Saving) {
                *row.gate.lock() = true;
                row.dto.state = BatchRowStateDto::Cancelled;
                row.dto.incomplete = !row.dto.calculation_complete;
                row.dto.reason = Some("Cancelled; any already published output is retained.".into());
                if current.as_ref() == Some(&row.dto.identity) { task = row.dto.task.clone(); }
            }
        }
        if let Some(task) = task {
            if state.jobs.iter().any(|j| j.job_id == task.job_id && !j.terminal) {
                self.request_job_stop_locked(&mut state, &task.run_id, &task.job_id, JobDisposition::Cancelled).map_err(|e| e.message)?;
            }
        }
        Ok(())
    }
    fn kick_batch(&self) {
        let mut state = self.lock();
        if state.batch.worker || !state.batch.snapshot.started || state.batch.snapshot.paused && state.batch.current.is_none() { return; }
        state.batch.worker = true;
        let manager = self.clone();
        thread::spawn(move || manager.batch_worker());
    }
    fn batch_worker(&self) {
        loop {
            let next = {
                let mut state = self.lock();
                if state.batch.snapshot.paused || !state.batch.snapshot.started {
                    state.batch.worker = false; return;
                }
                let ready = current_admitting_run(&state.phase);
                if ready.as_ref().map(|r| &r.run_id) != state.batch.run_id.as_ref() {
                    pause(&mut state.batch, "Run changed; explicit Continue is required.");
                    state.batch.worker = false; return;
                }
                let Some(index) = state.batch.rows.iter().position(pending) else { state.batch.worker = false; return; };
                let busy = !state.batch.review_allowed || state.match_reservation.is_some() || state.game_move.is_some() || state.runtime_control.is_some() || state.continuous_departing || state.finite_admission_pending
                    || state.analysis_task.as_ref().is_some_and(|t| matches!(t.state, AnalysisTaskStateDto::Queued | AnalysisTaskStateDto::Searching | AnalysisTaskStateDto::Pausing | AnalysisTaskStateDto::Paused))
                    || state.jobs.iter().any(|j| !j.terminal && j.mode == AnalysisJobModeDto::Finite);
                if busy {
                    state.batch.rows[index].dto.state = BatchRowStateDto::Waiting;
                    state.batch.rows[index].dto.reason = Some("Waiting for the explicit foreground owner; no takeover.".into());
                    None
                } else if state.continuous_safety_hold || state.continuous_error {
                    pause(&mut state.batch, "Engine safety hold; resolve explicitly before Continue."); None
                } else {
                    let id = state.batch.rows[index].dto.identity.clone();
                    state.batch.current = Some(id.clone());
                    state.batch.rows[index].dto.state = BatchRowStateDto::Running;
                    state.batch.rows[index].dto.reason = None;
                    Some((id, state.batch.rows[index].dto.source_path.clone(), state.batch.rows[index].dto.conditions.clone(), ready.expect("run checked").run_id))
                }
            };
            let Some((id, path, conditions, run_id)) = next else { thread::sleep(Duration::from_millis(30)); continue; };
            let result = self.run_batch_row(&id, &path, &conditions, &run_id);
            let mut state = self.lock();
            if let Err(error) = result {
                if let Ok(row) = row_mut(&mut state.batch, &id) {
                    if row.dto.state == BatchRowStateDto::Running { row.dto.state = BatchRowStateDto::Failed; row.dto.incomplete = true; }
                    row.dto.reason = Some(error.clone());
                }
                pause(&mut state.batch, &error);
            }
            if state.batch.current.as_ref() == Some(&id) { state.batch.current = None; }
            let hand_back = state.batch.snapshot.paused || !state.batch.rows.iter().any(pending);
            drop(state);
            if hand_back { self.reconcile_continuous(); }
        }
    }
    fn run_batch_row(&self, id: &BatchIdentityDto, path: &str, conditions: &BatchPreferencesDto, run_id: &str) -> Result<(), String> {
        let selected = { let state = self.lock(); current_selected_job(&state).filter(|j| !j.state.is_limited()) };
        if let Some(job) = selected {
            self.request_job_stop(&job.run_id, &job.job_id, JobDisposition::Superseded).map_err(|e| e.message)?;
            self.wait_for_job_cancellation(&job.run_id, &job.job_id, self.inner.config.stop_drain_timeout).map_err(|e| e.message)?;
        }
        let input = read_document(path).and_then(|(document, digest)| resolve(&document, conditions).map(|(preview, nodes)| (document, digest, preview, nodes)));
        let (document, digest, preview, nodes) = match input {
            Ok(value) => value,
            Err(error) => { let mut state = self.lock(); let row = row_mut(&mut state.batch, id)?; if row.dto.state != BatchRowStateDto::Cancelled { row.dto.state = BatchRowStateDto::Failed; row.dto.reason = Some(error); } return Ok(()); }
        };
        let mut work_items = Vec::with_capacity(nodes.len());
        for node in nodes {
            let query = katago_protocol::analysis_query_from_position(document.board_width(), document.board_height(), document.komi(), &node.position.stones, node.position.to_play,
                katago_protocol::AnalysisQueryOptions { id: "pending".into(), rules: document.rules(), turn: node.position.move_number, max_visits: Some(1), include_ownership: Some(true), include_policy: Some(true) });
            let query = match query {
                Ok(query) => query,
                Err(error) => {
                    let mut state = self.lock();
                    let row = row_mut(&mut state.batch, id)?;
                    if row.dto.state != BatchRowStateDto::Cancelled { row.dto.state = BatchRowStateDto::Failed; row.dto.reason = Some(error.to_string()); }
                    return Ok(());
                }
            };
            work_items.push(WholeGameWorkItem { node_path: node.path, query, board_width: document.board_width(), board_height: document.board_height(), move_number: node.position.move_number });
        }
        {
            let mut state = self.lock();
            let row = row_mut(&mut state.batch, id)?;
            if row.dto.state == BatchRowStateDto::Cancelled { return Ok(()); }
            row.dto.document_id = Some(Uuid::new_v4().to_string());
            row.dto.input_receipt = Some(format!("{digest}; SZ[{}:{}] RU[{}] KM[{}]{}", document.board_width(), document.board_height(), document.rules(), document.komi(), if row.preview_digest.as_ref() != Some(&digest) { "; changed since intake; read at row admission" } else { "; read at row admission" }));
            row.dto.preview = Some(preview.clone()); row.document = Some(document);
        }
        if work_items.is_empty() {
            let mut state = self.lock(); let row = row_mut(&mut state.batch, id)?;
            row.dto.calculation_complete = true; row.dto.has_result = true;
        } else {
            let (overview, deep) = match conditions.strategy {
                AnalysisTaskStrategyDto::SingleStage => (None, conditions.normal.clone()),
                AnalysisTaskStrategyDto::AllPositionsTwoStage => (Some(conditions.overview.clone()), conditions.deep.clone()),
                AnalysisTaskStrategyDto::SwingSelectedTwoStage => (Some(conditions.swing_overview.clone()), conditions.swing_deep.clone()),
            };
            let targets = AnalysisTaskTargets { requested: Some(preview.targets.iter().map(|t| t.node_path.clone()).collect()), supporting: preview.supporting_targets.iter().map(|t| t.node_path.clone()).collect(), swing_comparisons: preview.swing_comparisons, swing_criteria: preview.swing_criteria };
            let task = self.start_analysis_task_with_strategy(WholeGameJobRequest { run_id: run_id.into(), generation: id.attempt, work_items }, conditions.scope.clone(), conditions.strategy, overview, deep, targets, None, Some(id.clone()));
            let task = match task {
                Ok(task) => task,
                Err(error) => { let mut state = self.lock(); let row = row_mut(&mut state.batch, id)?; if row.dto.state != BatchRowStateDto::Cancelled { row.dto.state = BatchRowStateDto::Failed; row.dto.reason = Some(error.message.clone()); } return Err(error.message); }
            };
            loop {
                let (terminal, cancelled) = {
                    let mut state = self.lock();
                    let current_task = state.analysis_task.as_ref().filter(|t| t.job_id == task.job_id).cloned();
                    let alive = state.jobs.iter().any(|j| j.job_id == task.job_id && !j.terminal);
                    let changed = current_admitting_run(&state.phase).is_none_or(|r| r.run_id != run_id);
                    let row = row_mut(&mut state.batch, id)?;
                    if let Some(t) = current_task { row.dto.task = Some(t); }
                    // A protocol failure can retire Ready itself. Preserve the task's failure
                    // rather than relabeling that failure as a user/Run-change cancellation.
                    let task_failed = row.dto.task.as_ref().is_some_and(|t| t.state == AnalysisTaskStateDto::Failed);
                    if changed && !task_failed { row.dto.state = BatchRowStateDto::Cancelled; row.dto.reason = Some("Run changed; explicit Continue required.".into()); *row.gate.lock() = true; }
                    let cancelled = row.dto.state == BatchRowStateDto::Cancelled;
                    (if !alive { row.dto.task.clone() } else { None }, cancelled)
                };
                if cancelled {
                    self.wait_for_job_cancellation(run_id, &task.job_id, self.inner.config.stop_drain_timeout).map_err(|e| e.message)?;
                    let mut state = self.lock(); let row = row_mut(&mut state.batch, id)?; row.dto.incomplete = true;
                    return Ok(());
                }
                if let Some(terminal) = terminal {
                    let mut state = self.lock(); let row = row_mut(&mut state.batch, id)?;
                    if terminal.state != AnalysisTaskStateDto::Completed {
                        let reason = terminal.reason.unwrap_or_else(|| "Engine task failed or retired; explicit resolution required.".into());
                        row.dto.state = BatchRowStateDto::Failed; row.dto.reason = Some(reason.clone()); row.dto.incomplete = true; return Err(reason);
                    }
                    row.dto.calculation_complete = true; row.dto.has_result = true; break;
                }
                thread::sleep(Duration::from_millis(15));
            }
        }
        let output = {
            let mut state = self.lock();
            let directory = state.batch.snapshot.output_directory.clone();
            let row = row_mut(&mut state.batch, id)?;
            if row.dto.state == BatchRowStateDto::Cancelled { return Ok(()); }
            row.dto.state = BatchRowStateDto::Completed;
            directory.map(|d| PathBuf::from(d).join(&row.dto.output_name))
        };
        if let Some(path) = output { self.batch_save(id, &path, false, false, false)?; }
        Ok(())
    }
    fn batch_save(&self, id: &BatchIdentityDto, path: &Path, overwrite: bool, incomplete: bool, explicit: bool) -> Result<(), String> {
        let (mut document, gate, receipt) = {
            let mut state = self.lock();
            if state.batch.closed { return Err("The batch session is closing.".into()); }
            let row = row_mut(&mut state.batch, id)?;
            if !explicit && *row.gate.lock() { return Ok(()); }
            if row.dto.output_state == BatchOutputStateDto::Saving { return Err("An output is already being published.".into()); }
            if !row.dto.calculation_complete && !incomplete { return Err("Explicitly authorize an incomplete output.".into()); }
            if !row.dto.has_result { return Err("This row has no valid analysis result to save.".into()); }
            if row.dto.state == BatchRowStateDto::Running { return Err("Stop or complete the row before saving.".into()); }
            let document = row.document.clone().ok_or("No input snapshot")?;
            row.dto.output_state = BatchOutputStateDto::Saving;
            // An explicit later Save grants a new publication attempt, not more computation.
            if explicit { row.gate = Arc::new(Mutex::new(false)); }
            (document, row.gate.clone(), row.dto.input_receipt.clone().unwrap_or_default())
        };
        let result = (|| {
            let source = {
                let mut state = self.lock();
                row_mut(&mut state.batch, id)?.dto.source_path.clone()
            };
            let parent = path.parent().ok_or("Output needs a parent directory")?;
            let resolved = std::fs::canonicalize(parent).map_err(|e| e.to_string())?.join(path.file_name().ok_or("Output needs a file name")?);
            let actual = std::fs::canonicalize(&resolved).unwrap_or(resolved);
            if actual == Path::new(&source) { return Err("The source file is never a batch output target.".into()); }
            document.mark_batch_output(&format!("{}:{}:{};{receipt}", id.batch_id, id.row_id, id.attempt), incomplete).map_err(|e| e.to_string())?;
            sgf::save_document_authorized(&document, path, overwrite, &gate)
        })();
        let mut state = self.lock();
        let row = row_mut(&mut state.batch, id)?;
        match &result {
            Ok(()) => { row.dto.output_state = BatchOutputStateDto::Saved; row.dto.saved_path = Some(path.to_string_lossy().into_owned()); row.dto.incomplete = incomplete; }
            Err(error) if *gate.lock() => { row.dto.output_state = BatchOutputStateDto::Unsaved; row.dto.reason = Some(error.clone()); return Ok(()); }
            Err(error) => { row.dto.output_state = BatchOutputStateDto::Failed; row.dto.reason = Some(error.clone()); pause(&mut state.batch, error); }
        }
        result
    }
}
fn pause(owner: &mut Owner, reason: &str) { owner.snapshot.paused = true; owner.snapshot.reason = Some(reason.into()); }

/// Called under the same manager lock as finite publication, before observers can see a frame.
pub(super) fn observe(state: &mut ManagerState, event: &ForegroundEngineEventDto) {
    let ForegroundEngineEventDto::Job { job } = event else { return; };
    if !job.job_id.starts_with("batch:") || !admits_analysis_attachment(job) { return; }
    let Some(row) = state.batch.rows.iter_mut().find(|r| r.dto.task.as_ref().is_some_and(|t| t.job_id == job.job_id && t.run_id == job.run_id && t.generation == job.generation)) else { return; };
    if row.dto.state != BatchRowStateDto::Running { return; }
    if let (Some(document), Some(frame)) = (&mut row.document, &job.frame) {
        match document.replace_primary_analysis(&job.node_path, &sgf::SgfAnalysisPayload::from_frame(frame, "KataGo")) {
            Ok(_) => row.dto.has_result = true,
            Err(error) => { row.dto.reason = Some(error.to_string()); }
        }
    }
}
pub(super) fn bind(state: &mut ManagerState, id: &BatchIdentityDto, task: &AnalysisTaskDto) {
    if let Ok(row) = row_mut(&mut state.batch, id) { row.dto.task = Some(task.clone()); }
}
pub(super) fn admit(state: &ManagerState, id: Option<&BatchIdentityDto>) -> Result<(), EngineFailureDto> {
    if id.is_some() && (state.batch.closed || !state.batch.review_allowed) { return Err(continuous_invalid_state(state, "Batch authority is no longer available.")); }
    if id.is_none() && occupied(&state.batch) || state.batch.current.as_ref() != id || id.is_some_and(|id| state.batch.rows.iter().find(|r| &r.dto.identity == id).is_none_or(|r| r.dto.state != BatchRowStateDto::Running)) {
        return Err(continuous_invalid_state(state, "The private batch owner holds this Run, or this batch attempt was retired."));
    }
    Ok(())
}
