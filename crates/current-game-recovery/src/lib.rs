use app_model::{
    ApplicationExitDispositionDto, NodePath, RecoveryEnvelopeDto, RecoveryProtectionDto, RecoveryStartupDto,
    RECOVERY_UNREADABLE_MESSAGE,
};
use sgf::CurrentSgfDocument;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

pub const RECOVERY_FILE: &str = "current-game-recovery.json";
pub const RECOVERY_WRITE_DELAY_MS: u64 = 1000;
pub const UNPROTECTED_WRITE_MESSAGE: &str = "Recent current-game changes are not yet protected.";

#[derive(Debug, Clone, PartialEq)]
pub struct RecoverySnapshot {
    pub document_seq: u64,
    pub snapshot_seq: u64,
    pub sgf_text: String,
    pub selected_path: NodePath,
    pub source_path: Option<String>,
    pub dirty: bool,
}

#[derive(Debug)]
pub struct RecoveryCoordinator {
    pending: Option<RecoveryEnvelopeDto>,
    writing: Option<RecoveryEnvelopeDto>,
    last_committed: Option<RecoveryEnvelopeDto>,
    due_at_ms: Option<u64>,
    protection: RecoveryProtectionDto,
    discarded_document_seq: Option<u64>,
}

impl Default for RecoveryCoordinator {
    fn default() -> Self {
        Self {
            pending: None,
            writing: None,
            last_committed: None,
            due_at_ms: None,
            protection: RecoveryProtectionDto::Protected,
            discarded_document_seq: None,
        }
    }
}

impl RecoveryCoordinator {
    pub fn protection(&self) -> RecoveryProtectionDto {
        self.protection.clone()
    }

    pub fn last_committed(&self) -> Option<&RecoveryEnvelopeDto> {
        self.last_committed.as_ref()
    }

    pub fn note(&mut self, snapshot: RecoverySnapshot, now_ms: u64) {
        let envelope = envelope_from_snapshot(snapshot, ApplicationExitDispositionDto::ExitIncomplete);
        if self.is_stale(&envelope) {
            return;
        }
        self.pending = Some(envelope);
        if self.writing.is_none() && self.due_at_ms.is_none() {
            self.due_at_ms = Some(now_ms.saturating_add(RECOVERY_WRITE_DELAY_MS));
        }
    }

    pub fn take_due_write(&mut self, now_ms: u64) -> Option<RecoveryEnvelopeDto> {
        if self.writing.is_some() {
            return None;
        }
        let due_at = self.due_at_ms?;
        if now_ms < due_at {
            return None;
        }
        let envelope = self.pending.take()?;
        if self.is_stale(&envelope) {
            self.due_at_ms = None;
            return None;
        }
        self.due_at_ms = None;
        self.writing = Some(envelope.clone());
        Some(envelope)
    }

    pub fn finish_write(&mut self, envelope: RecoveryEnvelopeDto, result: Result<(), String>) {
        if self.writing.as_ref() != Some(&envelope) {
            return;
        }
        self.writing = None;
        match result {
            Ok(()) => {
                if !self.is_stale(&envelope) {
                    if matches!(
                        envelope.disposition,
                        ApplicationExitDispositionDto::ExplicitDiscard
                    ) {
                        self.discarded_document_seq = Some(envelope.document_seq);
                    }
                    self.last_committed = Some(envelope);
                    self.protection = RecoveryProtectionDto::Protected;
                }
                if self.pending.is_some() {
                    self.due_at_ms = Some(0);
                }
            }
            Err(_) => {
                if self.pending.is_none() {
                    self.pending = Some(envelope);
                }
                self.protection = RecoveryProtectionDto::Unprotected {
                    message: UNPROTECTED_WRITE_MESSAGE.to_string(),
                };
            }
        }
    }

    pub fn take_flush_write(
        &mut self,
        snapshot: RecoverySnapshot,
        disposition: ApplicationExitDispositionDto,
    ) -> Option<RecoveryEnvelopeDto> {
        if matches!(disposition, ApplicationExitDispositionDto::ExplicitDiscard) {
            self.discarded_document_seq = Some(snapshot.document_seq);
        }
        let envelope = envelope_from_snapshot(snapshot, disposition);
        self.pending = None;
        self.due_at_ms = None;
        if self.is_stale(&envelope) {
            return None;
        }
        self.writing = Some(envelope.clone());
        Some(envelope)
    }

    pub fn retry(&mut self, now_ms: u64) -> Option<RecoveryEnvelopeDto> {
        if self.writing.is_some() || self.pending.is_none() {
            return None;
        }
        self.due_at_ms = Some(now_ms);
        self.take_due_write(now_ms)
    }

    pub fn load_checkpoint(&mut self, envelope: RecoveryEnvelopeDto) {
        if self.last_committed.is_some() || self.pending.is_some() || self.writing.is_some() {
            return;
        }
        if matches!(
            envelope.disposition,
            ApplicationExitDispositionDto::ExplicitDiscard
        ) {
            self.discarded_document_seq = Some(envelope.document_seq);
        }
        self.last_committed = Some(envelope);
    }

    pub fn mark_discarded_committed(&mut self, mut envelope: RecoveryEnvelopeDto) {
        envelope.disposition = ApplicationExitDispositionDto::ExplicitDiscard;
        self.discarded_document_seq = Some(envelope.document_seq);
        self.pending = None;
        self.due_at_ms = None;
        self.writing = None;
        self.last_committed = Some(envelope);
        self.protection = RecoveryProtectionDto::Protected;
    }

    fn is_stale(&self, envelope: &RecoveryEnvelopeDto) -> bool {
        if self.discarded_document_seq.is_some_and(|seq| {
            envelope.document_seq <= seq
                && !matches!(
                    envelope.disposition,
                    ApplicationExitDispositionDto::ExplicitDiscard
                )
        }) {
            return true;
        }
        match &self.last_committed {
            None => false,
            Some(committed) => {
                if envelope.is_newer_than(committed) {
                    return false;
                }
                if envelope.document_seq == committed.document_seq
                    && envelope.snapshot_seq == committed.snapshot_seq
                    && envelope.disposition != committed.disposition
                {
                    return false;
                }
                true
            }
        }
    }
}

pub fn classify_startup(loaded: Result<Option<RecoveryEnvelopeDto>, String>) -> RecoveryStartupDto {
    match loaded {
        Err(_) => RecoveryStartupDto::Unreadable {
            message: RECOVERY_UNREADABLE_MESSAGE.to_string(),
        },
        Ok(None) => RecoveryStartupDto::None,
        Ok(Some(envelope)) => match envelope.disposition {
            ApplicationExitDispositionDto::ExplicitDiscard => RecoveryStartupDto::None,
            ApplicationExitDispositionDto::ExitIncomplete => RecoveryStartupDto::Abnormal { envelope },
            ApplicationExitDispositionDto::CleanCompleted => RecoveryStartupDto::Normal { envelope },
        },
    }
}

pub fn validate_envelope(envelope: &RecoveryEnvelopeDto) -> Result<(), String> {
    let document = CurrentSgfDocument::open(&envelope.sgf_text).map_err(|error| error.to_string())?;
    document
        .snapshot(&envelope.selected_path)
        .map(|_| ())
        .map_err(|error| error.to_string())
}

pub(crate) fn envelope_from_snapshot(
    snapshot: RecoverySnapshot,
    disposition: ApplicationExitDispositionDto,
) -> RecoveryEnvelopeDto {
    RecoveryEnvelopeDto {
        document_seq: snapshot.document_seq,
        snapshot_seq: snapshot.snapshot_seq,
        sgf_text: snapshot.sgf_text,
        selected_path: snapshot.selected_path,
        source_path: snapshot.source_path,
        dirty: snapshot.dirty,
        disposition,
    }
}

pub struct FileRecoveryStore {
    path: PathBuf,
}

impl FileRecoveryStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn load(&self) -> Result<Option<RecoveryEnvelopeDto>, String> {
        match fs::read_to_string(&self.path) {
            Ok(contents) => match serde_json::from_str::<RecoveryEnvelopeDto>(&contents) {
                Ok(envelope) => Ok(Some(envelope)),
                Err(_) => Err(RECOVERY_UNREADABLE_MESSAGE.to_string()),
            },
            Err(err) if err.kind() == ErrorKind::NotFound => Ok(None),
            Err(_) => Err(RECOVERY_UNREADABLE_MESSAGE.to_string()),
        }
    }

    pub fn replace(&self, envelope: &RecoveryEnvelopeDto) -> Result<(), String> {
        if let Some(existing) = self.load().ok().flatten() {
            if existing.document_seq > envelope.document_seq {
                return Ok(());
            }
            if existing.document_seq == envelope.document_seq && existing.snapshot_seq > envelope.snapshot_seq
            {
                return Ok(());
            }
            if matches!(
                existing.disposition,
                ApplicationExitDispositionDto::ExplicitDiscard
            ) && envelope.document_seq <= existing.document_seq
                && !matches!(
                    envelope.disposition,
                    ApplicationExitDispositionDto::ExplicitDiscard
                )
            {
                return Ok(());
            }
            if matches!(
                existing.disposition,
                ApplicationExitDispositionDto::CleanCompleted
            ) && envelope.document_seq == existing.document_seq
                && envelope.snapshot_seq <= existing.snapshot_seq
                && matches!(
                    envelope.disposition,
                    ApplicationExitDispositionDto::ExitIncomplete
                )
            {
                return Ok(());
            }
        }
        atomic_replace_json(&self.path, envelope)
    }
}

fn atomic_replace_json(path: &Path, envelope: &RecoveryEnvelopeDto) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create recovery directory {}: {err}", parent.display()))?;
    }
    let json = serde_json::to_string_pretty(envelope)
        .map_err(|err| format!("failed to serialize {}: {err}", path.display()))?;
    let tmp = tmp_path(path);
    fs::write(&tmp, json).map_err(|err| format!("failed to write {}: {err}", tmp.display()))?;
    match fs::rename(&tmp, path) {
        Ok(()) => Ok(()),
        Err(err) => {
            let _ = fs::remove_file(&tmp);
            Err(format!("failed to replace {}: {err}", path.display()))
        }
    }
}

fn tmp_path(path: &Path) -> PathBuf {
    let mut tmp = path.as_os_str().to_os_string();
    tmp.push(".tmp");
    PathBuf::from(tmp)
}

#[cfg(test)]
mod tests;
