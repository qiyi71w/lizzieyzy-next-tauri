//! Export owns only detached diagnostic bytes. It never holds a manager or Run handle.
use app_model::{DiagnosticExportPhaseDto as Phase, DiagnosticExportStatusDto, EngineDiagnosticSnapshotDto};
use parking_lot::{Condvar, Mutex};
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};

const SOURCE_BYTES: usize = 1024 * 1024;
const BUNDLE_BYTES: usize = 2 * 1024 * 1024;
const COLLECTION_BUDGET: Duration = Duration::from_secs(2);
const ARCHIVE_BUDGET: Duration = Duration::from_secs(5);
const LOCK_BUDGET: Duration = Duration::from_millis(25);
type Bundle = Vec<(&'static str, Vec<u8>)>;

#[derive(Clone)]
pub struct DiagnosticExport {
    inner: Arc<Inner>,
}
struct Inner {
    directory: PathBuf,
    state: Mutex<State>,
    finished: Condvar,
    publication: Mutex<()>,
    closing: AtomicBool,
    #[cfg(test)]
    checkpoint: Option<Arc<dyn Fn(Phase) -> Result<(), String> + Send + Sync>>,
}
struct State {
    status: DiagnosticExportStatusDto,
    pending: Option<(u64, EngineDiagnosticSnapshotDto)>,
    bundle: Option<Bundle>,
    export_requested: bool,
    running: bool,
    closed: bool,
    cancelled: bool,
}
impl DiagnosticExport {
    pub fn new(directory: PathBuf) -> Self {
        Self {
            inner: Arc::new(Inner {
                directory,
                finished: Condvar::new(),
                publication: Mutex::new(()),
                closing: AtomicBool::new(false),
                state: Mutex::new(State {
                    status: DiagnosticExportStatusDto {
                        generation: 0,
                        attempt_id: None,
                        captured_at_ms: None,
                        phase: Phase::Idle,
                        source_bytes: None,
                        entries: None,
                        completed_entries: 0,
                        failed_stage: None,
                        message: None,
                        file_name: None,
                        cleanup_pending: false,
                    },
                    pending: None,
                    bundle: None,
                    export_requested: false,
                    running: false,
                    closed: false,
                    cancelled: false,
                }),
                #[cfg(test)]
                checkpoint: None,
            }),
        }
    }
    /// One active estimate and one latest pending snapshot; obsolete work cannot publish.
    pub fn estimate(&self, snapshot: EngineDiagnosticSnapshotDto) -> Result<u64, String> {
        validate_source(&snapshot)?;
        let mut state = self
            .inner
            .state
            .try_lock_for(LOCK_BUDGET)
            .ok_or("Publication is completing")?;
        if state.closed || self.inner.closing.load(Ordering::Acquire) {
            return Err("Diagnostic export is closed".into());
        }
        if exporting(state.status.phase) {
            return Err("An export is already active".into());
        }
        if state.status.cleanup_pending {
            return Err("Owned staging cleanup is outstanding; no new export is admitted".into());
        }
        state.status.generation = state
            .status
            .generation
            .checked_add(1)
            .ok_or("Estimate generation exhausted")?;
        let generation = state.status.generation;
        state.status.attempt_id = Some(snapshot.attempt_id.clone());
        state.status.captured_at_ms = Some(snapshot.captured_at_ms);
        state.status.phase = if state.running {
            Phase::EstimateObsolete
        } else {
            Phase::Estimating
        };
        state.status.source_bytes = None;
        state.status.entries = None;
        state.status.completed_entries = 0;
        state.status.failed_stage = None;
        state.status.message = None;
        state.status.file_name = None;
        state.bundle = None;
        state.cancelled = false;
        state.pending = Some((generation, snapshot));
        self.start_worker(&mut state);
        Ok(generation)
    }
    pub fn status(&self) -> DiagnosticExportStatusDto {
        self.inner.state.lock().status.clone()
    }
    pub fn export(&self, generation: u64) -> Result<(), String> {
        let mut state = self
            .inner
            .state
            .try_lock_for(LOCK_BUDGET)
            .ok_or("Publication is completing")?;
        if state.closed
            || self.inner.closing.load(Ordering::Acquire)
            || state.status.generation != generation
            || state.status.phase != Phase::Ready
        {
            return Err("Estimate is obsolete or not ready".into());
        }
        state.status.phase = Phase::Collecting;
        state.export_requested = true;
        self.start_worker(&mut state);
        Ok(())
    }
    pub fn cancel(&self, generation: u64) -> Result<(), String> {
        let _publication = self
            .inner
            .publication
            .try_lock_for(LOCK_BUDGET)
            .ok_or("Publication is completing; refresh its result")?;
        let mut state = self
            .inner
            .state
            .try_lock_for(LOCK_BUDGET)
            .ok_or("Publication is completing; refresh its result")?;
        if generation != state.status.generation {
            return Err("Estimate is obsolete".into());
        }
        if state.status.file_name.is_some()
            || matches!(
                state.status.phase,
                Phase::Completed | Phase::Failed | Phase::Cancelled | Phase::Closed | Phase::Idle
            )
        {
            return Ok(());
        }
        state.cancelled = true;
        state.pending = None;
        state.bundle = None;
        state.export_requested = false;
        state.status.phase = if state.running {
            Phase::Cancelling
        } else {
            Phase::Cancelled
        };
        Ok(())
    }
    /// Seals admission even when a filesystem call cannot finish within the exit budget.
    /// False means cleanup is outstanding, never a claim of durable completion.
    pub fn shutdown(&self, budget: Duration) -> bool {
        let deadline = Instant::now() + budget;
        self.inner.closing.store(true, Ordering::Release);
        let Some(mut state) = self.inner.state.try_lock_for(budget) else {
            return false;
        };
        state.closed = true;
        state.cancelled = true;
        state.pending = None;
        state.bundle = None;
        state.export_requested = false;
        while state.running {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return false;
            }
            self.inner.finished.wait_for(&mut state, remaining);
        }
        if state.status.phase != Phase::Completed {
            state.status.phase = Phase::Closed;
        }
        !state.status.cleanup_pending
    }
    pub fn completed_directory(&self, generation: u64) -> Result<PathBuf, String> {
        let state = self.inner.state.lock();
        if state.closed || state.status.generation != generation || state.status.phase != Phase::Completed {
            return Err("No completed export for this request".into());
        }
        Ok(self.inner.directory.clone())
    }
    fn start_worker(&self, state: &mut State) {
        if state.running {
            return;
        }
        state.running = true;
        let owner = self.clone();
        if std::thread::Builder::new()
            .name("diagnostic-export".into())
            .spawn(move || owner.work())
            .is_err()
        {
            state.running = false;
            state.pending = None;
            state.bundle = None;
            state.export_requested = false;
            fail(
                state,
                state.status.phase,
                "Cannot start diagnostic export worker".into(),
            );
        }
    }
    fn work(&self) {
        loop {
            let task = {
                let mut state = self.inner.state.lock();
                if state.cancelled || state.closed {
                    if state.status.file_name.is_none() {
                        state.status.phase = if state.closed {
                            Phase::Closed
                        } else {
                            Phase::Cancelled
                        };
                    }
                    None
                } else if let Some((generation, snapshot)) = state.pending.take() {
                    state.status.phase = Phase::Estimating;
                    Some(Work::Estimate(generation, snapshot))
                } else if state.export_requested {
                    state.export_requested = false;
                    state
                        .bundle
                        .take()
                        .map(|bundle| Work::Export(state.status.generation, bundle))
                } else {
                    None
                }
            };
            match task {
                Some(Work::Estimate(generation, snapshot)) => {
                    let result = self
                        .stage(generation, Phase::Estimating, Instant::now() + COLLECTION_BUDGET)
                        .and_then(|()| collect(&snapshot));
                    let mut state = self.inner.state.lock();
                    if state.status.generation == generation && !state.cancelled && !state.closed {
                        match result {
                            Ok(bundle) => {
                                state.status.source_bytes =
                                    Some(bundle.iter().map(|(_, bytes)| bytes.len() as u64).sum());
                                state.status.entries = Some(bundle.len() as u32);
                                state.status.phase = Phase::Ready;
                                state.bundle = Some(bundle);
                            }
                            Err(error) => fail(&mut state, Phase::Estimating, error),
                        }
                    }
                }
                Some(Work::Export(generation, bundle)) => self.write_archive(generation, bundle),
                None => {
                    let mut state = self.inner.state.lock();
                    // Admission may have supplied work between the two short critical sections.
                    if !state.closed
                        && !state.cancelled
                        && (state.pending.is_some() || state.export_requested)
                    {
                        continue;
                    }
                    if state.status.file_name.is_none() && (state.closed || state.cancelled) {
                        state.status.phase = if state.closed {
                            Phase::Closed
                        } else {
                            Phase::Cancelled
                        };
                    }
                    state.running = false;
                    self.inner.finished.notify_all();
                    return;
                }
            }
        }
    }
    fn stage(&self, generation: u64, phase: Phase, deadline: Instant) -> Result<(), String> {
        #[cfg(test)]
        if let Some(checkpoint) = &self.inner.checkpoint {
            checkpoint(phase)?;
        }
        let mut state = self.inner.state.lock();
        if state.closed
            || self.inner.closing.load(Ordering::Acquire)
            || state.cancelled
            || state.status.generation != generation
        {
            return Err("Export cancelled".into());
        }
        if Instant::now() >= deadline {
            return Err("Export stage deadline exceeded".into());
        }
        state.status.phase = phase;
        Ok(())
    }
    fn write_archive(&self, generation: u64, bundle: Bundle) {
        let deadline = Instant::now() + ARCHIVE_BUDGET;
        let id = uuid::Uuid::new_v4();
        let file_name = format!("diagnostics-{id}.zip");
        let temporary = self.inner.directory.join(format!(".diagnostics-{id}.partial"));
        let destination = self.inner.directory.join(&file_name);
        let mut owned = None;
        let mut stage = Phase::Collecting;
        let result = (|| -> Result<(), String> {
            let collection_deadline = Instant::now() + COLLECTION_BUDGET;
            self.stage(generation, stage, collection_deadline)?;
            fs::create_dir_all(&self.inner.directory)
                .map_err(|_| "Cannot create diagnostic export directory")?;
            self.stage(generation, stage, collection_deadline)?;
            let mut options = OpenOptions::new();
            options.write(true).read(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            let file = options
                .open(&temporary)
                .map_err(|_| "Cannot create private export staging file")?;
            owned = Some(OwnedTemporary::new(temporary.clone(), &file));
            self.stage(generation, stage, collection_deadline)?;
            stage = Phase::Archiving;
            self.stage(generation, stage, deadline)?;
            let mut archive = ZipWriter::new(file);
            for (name, bytes) in bundle {
                self.stage(generation, stage, deadline)?;
                archive
                    .start_file(
                        name,
                        SimpleFileOptions::default().compression_method(CompressionMethod::Stored),
                    )
                    .map_err(|_| "Cannot create ZIP entry")?;
                for chunk in bytes.chunks(16 * 1024) {
                    self.stage(generation, stage, deadline)?;
                    archive.write_all(chunk).map_err(|_| "Cannot write ZIP entry")?;
                }
                self.inner.state.lock().status.completed_entries += 1;
            }
            let file = archive.finish().map_err(|_| "Cannot close ZIP archive")?;
            stage = Phase::Syncing;
            self.stage(generation, stage, deadline)?;
            file.sync_all().map_err(|_| "Cannot force ZIP bytes to storage")?;
            stage = Phase::Publishing;
            self.stage(generation, stage, deadline)?;
            // Cancel and publication share this linearization point. A cancellation
            // admitted first prevents publication; afterwards the completed ZIP wins.
            let _publication = self.inner.publication.lock();
            {
                let state = self.inner.state.lock();
                if state.cancelled
                    || state.closed
                    || self.inner.closing.load(Ordering::Acquire)
                    || state.status.generation != generation
                {
                    return Err("Export cancelled".into());
                }
            }
            if Instant::now() >= deadline {
                return Err("Export stage deadline exceeded".into());
            }
            // Same-directory hard-link publication is atomic and never replaces an
            // existing name. The file is already closed as a ZIP and synced.
            fs::hard_link(&temporary, &destination).map_err(|_| "Cannot atomically publish ZIP archive")?;
            self.inner.state.lock().status.file_name = Some(file_name);
            Ok(())
        })();
        let clean = owned.as_ref().is_none_or(OwnedTemporary::prune);
        let mut state = self.inner.state.lock();
        state.status.cleanup_pending = !clean;
        if let Err(error) = result {
            if state.cancelled || state.closed {
                state.status.phase = if state.closed {
                    Phase::Closed
                } else {
                    Phase::Cancelled
                };
            } else {
                fail(&mut state, stage, error);
            }
        } else {
            state.status.phase = Phase::Completed;
        }
    }
}
enum Work {
    Estimate(u64, EngineDiagnosticSnapshotDto),
    Export(u64, Bundle),
}
fn exporting(phase: Phase) -> bool {
    matches!(
        phase,
        Phase::Collecting | Phase::Archiving | Phase::Syncing | Phase::Publishing | Phase::Cancelling
    )
}
fn fail(state: &mut State, stage: Phase, message: String) {
    state.status.phase = Phase::Failed;
    state.status.failed_stage = Some(stage);
    state.status.message = Some(message);
}
fn validate_source(snapshot: &EngineDiagnosticSnapshotDto) -> Result<(), String> {
    let bounded = |text: &str| text.len() <= crate::diagnostics::RECORD_BYTES;
    if snapshot.records.len() > crate::diagnostics::CAPTURE_RECORDS
        || snapshot.metrics.len() > 32
        || [&snapshot.attempt_id, &snapshot.run_id, &snapshot.profile_id]
            .iter()
            .any(|id| id.len() > 128)
        || !bounded(&snapshot.command)
        || snapshot.failure.as_deref().is_some_and(|text| !bounded(text))
        || snapshot
            .records
            .iter()
            .any(|record| !bounded(&record.text) || record.source.len() > 64)
        || snapshot
            .records
            .iter()
            .map(|record| record.text.len())
            .sum::<usize>()
            > crate::diagnostics::CAPTURE_BYTES
        || snapshot.metrics.iter().any(|metric| {
            [&metric.role, &metric.name, &metric.unit]
                .iter()
                .any(|text| text.len() > 128)
                || metric.missing.as_deref().is_some_and(|text| !bounded(text))
        })
    {
        return Err("Diagnostic source exceeds its capture budget".into());
    }
    Ok(())
}
fn collect(snapshot: &EngineDiagnosticSnapshotDto) -> Result<Bundle, String> {
    let deadline = Instant::now() + COLLECTION_BUDGET;
    let snapshot_bytes =
        serde_json::to_vec_pretty(snapshot).map_err(|_| "Cannot encode diagnostic snapshot")?;
    let mut records = Vec::new();
    for record in &snapshot.records {
        if Instant::now() >= deadline {
            return Err("Diagnostic collection deadline exceeded".into());
        }
        serde_json::to_writer(&mut records, record).map_err(|_| "Cannot encode diagnostic record")?;
        records.push(b'\n');
    }
    let metrics =
        serde_json::to_vec_pretty(&snapshot.metrics).map_err(|_| "Cannot encode diagnostic metrics")?;
    let manifest = serde_json::to_vec_pretty(&serde_json::json!({"format": 1, "attempt_id": snapshot.attempt_id, "captured_at_ms": snapshot.captured_at_ms, "sources": ["snapshot.json", "records.jsonl", "metrics.json"], "privacy": "canonical captured aliases; no raw files", "missing": "only metrics present in the displayed capture are included"})).map_err(|_| "Cannot encode manifest")?;
    let bundle = vec![
        ("snapshot.json", snapshot_bytes),
        ("records.jsonl", records),
        ("metrics.json", metrics),
        ("manifest.json", manifest),
    ];
    if bundle.iter().any(|(_, bytes)| bytes.len() > SOURCE_BYTES)
        || bundle.iter().map(|(_, bytes)| bytes.len()).sum::<usize>() > BUNDLE_BYTES
    {
        return Err("Diagnostic bundle exceeds byte budget".into());
    }
    if Instant::now() >= deadline {
        return Err("Diagnostic collection deadline exceeded".into());
    }
    Ok(bundle)
}

struct OwnedTemporary {
    path: PathBuf,
    identity: Option<FileIdentity>,
}
#[derive(PartialEq)]
struct FileIdentity {
    id: [u8; 24],
    created: std::time::SystemTime,
}
impl OwnedTemporary {
    fn new(path: PathBuf, file: &File) -> Self {
        Self {
            path,
            identity: file_identity(file).ok(),
        }
    }
    fn prune(&self) -> bool {
        let Ok(file) = File::open(&self.path) else {
            return !self.path.exists();
        };
        let Some(identity) = &self.identity else {
            return false;
        };
        if file_identity(&file).ok().as_ref() != Some(identity) {
            return false;
        }
        fs::remove_file(&self.path).is_ok()
    }
}
fn file_identity(file: &File) -> io::Result<FileIdentity> {
    let metadata = file.metadata()?;
    let created = metadata.created()?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let mut id = [0; 24];
        id[..8].copy_from_slice(&metadata.dev().to_le_bytes());
        id[8..16].copy_from_slice(&metadata.ino().to_le_bytes());
        Ok(FileIdentity { id, created })
    }
    #[cfg(windows)]
    {
        use std::os::windows::io::AsRawHandle;
        use windows_sys::Win32::Storage::FileSystem::{
            FileIdInfo, GetFileInformationByHandleEx, FILE_ID_INFO,
        };
        let mut info: FILE_ID_INFO = unsafe { std::mem::zeroed() };
        if unsafe {
            GetFileInformationByHandleEx(
                file.as_raw_handle(),
                FileIdInfo,
                (&mut info as *mut FILE_ID_INFO).cast(),
                std::mem::size_of::<FILE_ID_INFO>() as u32,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        let mut id = [0; 24];
        id[..8].copy_from_slice(&info.VolumeSerialNumber.to_le_bytes());
        id[8..].copy_from_slice(&info.FileId.Identifier);
        Ok(FileIdentity { id, created })
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = created;
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "File identity unavailable",
        ))
    }
}

#[cfg(test)]
mod tests;
