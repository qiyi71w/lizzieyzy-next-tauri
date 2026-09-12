use super::*;
use ::current_game_recovery::{classify_startup, FileRecoveryStore, RecoverySnapshot};
use app_model::{ApplicationExitDispositionDto, ApplicationExitOutcomeDto, RecoveryStartupDto};
use std::time::{SystemTime, UNIX_EPOCH};

impl CurrentGameState {
    pub(super) fn note_recovery(&self, holder: &CurrentGameHolder) {
        let Ok(snapshot) = holder.recovery_snapshot() else {
            return;
        };
        self.recovery
            .lock()
            .expect("current game recovery")
            .note(snapshot, now_ms());
    }

    pub fn recovery_protection(&self) -> RecoveryProtectionDto {
        self.recovery.lock().expect("current game recovery").protection()
    }

    pub fn take_due_recovery_write(&self, now_ms: u64) -> Option<RecoveryEnvelopeDto> {
        self.recovery
            .lock()
            .expect("current game recovery")
            .take_due_write(now_ms)
    }

    pub fn finish_recovery_write(&self, envelope: RecoveryEnvelopeDto, result: Result<(), String>) {
        self.recovery
            .lock()
            .expect("current game recovery")
            .finish_write(envelope, result);
    }

    pub fn recovery_flush(&self, disposition: ApplicationExitDispositionDto) -> Option<RecoveryEnvelopeDto> {
        let holder = self.holder.lock().expect("current game state");
        let snapshot = holder.recovery_snapshot().ok()?;
        self.recovery
            .lock()
            .expect("current game recovery")
            .take_flush_write(snapshot, disposition)
    }

    pub fn retry_recovery(&self, now_ms: u64) -> Option<RecoveryEnvelopeDto> {
        self.recovery.lock().expect("current game recovery").retry(now_ms)
    }

    pub fn persist_flush(
        &self,
        store: &FileRecoveryStore,
        disposition: ApplicationExitDispositionDto,
    ) -> Result<(), String> {
        let Some(envelope) = self.recovery_flush(disposition) else {
            return Ok(());
        };
        let result = persist_recovery_envelope(store, &envelope);
        self.finish_recovery_write(envelope, result.clone());
        result
    }

    pub fn persist_retry(&self, store: &FileRecoveryStore, now_ms: u64) -> RecoveryProtectionDto {
        if let Some(envelope) = self.retry_recovery(now_ms) {
            let result = persist_recovery_envelope(store, &envelope);
            self.finish_recovery_write(envelope, result);
        }
        self.recovery_protection()
    }

    pub fn seed_stored_envelope(&self, envelope: &RecoveryEnvelopeDto) {
        {
            let mut holder = self.holder.lock().expect("current game state");
            holder.document_seq = holder.document_seq.max(envelope.document_seq);
        }
        self.recovery
            .lock()
            .expect("current game recovery")
            .load_checkpoint(envelope.clone());
    }

    pub fn mark_discard_committed(&self, envelope: RecoveryEnvelopeDto) {
        {
            let mut holder = self.holder.lock().expect("current game state");
            holder.document_seq = holder.document_seq.max(envelope.document_seq);
        }
        self.recovery
            .lock()
            .expect("current game recovery")
            .mark_discarded_committed(envelope);
    }

    pub fn restore_from_store(
        &self,
        store: &FileRecoveryStore,
    ) -> Result<CurrentGameResultDto, CurrentGameError> {
        match inspect_recovery_store(store) {
            RecoveryStartupDto::Abnormal { envelope } | RecoveryStartupDto::Normal { envelope } => {
                self.restore_envelope(envelope)
            }
            RecoveryStartupDto::Unreadable { message } => Err(CurrentGameError {
                kind: CurrentGameErrorKind::MalformedSgf,
                message,
            }),
            RecoveryStartupDto::None => Err(CurrentGameError {
                kind: CurrentGameErrorKind::NoCurrentGame,
                message: "No current-game recovery snapshot is available.".to_string(),
            }),
        }
    }

    pub fn restore_envelope(
        &self,
        envelope: RecoveryEnvelopeDto,
    ) -> Result<CurrentGameResultDto, CurrentGameError> {
        let document = CurrentSgfDocument::open(&envelope.sgf_text)?;
        let snapshot = document.snapshot(&envelope.selected_path)?;
        let tree = document.tree()?;
        let mut holder = self.holder.lock().expect("current game state");
        if holder.departure.is_some() {
            return Err(departure::departure_in_progress());
        }
        holder.document = Some(document);
        holder.generation = holder.generation.saturating_add(1);
        holder.dirty = envelope.dirty;
        if envelope.dirty {
            holder.dirty_epoch = holder.dirty_epoch.saturating_add(1);
        }
        holder.native_path = envelope.source_path.clone();
        holder.selected_path = envelope.selected_path.clone();
        holder.document_seq = envelope.document_seq;
        holder.snapshot_seq = envelope.snapshot_seq;
        holder.edits_blocked = false;
        holder.closed_jobs.clear();
        self.follow_continuous_position(&mut holder);
        Ok(CurrentGameResultDto {
            tree,
            selected_path: envelope.selected_path,
            snapshot,
            generation: holder.generation,
            dirty: holder.dirty,
            native_path: holder.native_path.clone(),
        })
    }
}

impl CurrentGameHolder {
    fn recovery_snapshot(&self) -> Result<RecoverySnapshot, CurrentGameError> {
        let document = self.document.as_ref().ok_or_else(no_current_game)?;
        Ok(RecoverySnapshot {
            document_seq: self.document_seq.max(1),
            snapshot_seq: self.snapshot_seq.max(1),
            sgf_text: document.serialize()?,
            selected_path: self.selected_path.clone(),
            source_path: self.native_path.clone(),
            dirty: self.dirty,
        })
    }
}

pub fn inspect_recovery_store(store: &FileRecoveryStore) -> RecoveryStartupDto {
    classify_startup(store.load())
}

pub fn inspect_and_seed(state: &CurrentGameState, store: &FileRecoveryStore) -> RecoveryStartupDto {
    let loaded = store.load();
    if let Ok(Some(envelope)) = &loaded {
        state.seed_stored_envelope(envelope);
    }
    classify_startup(loaded)
}

pub fn persist_recovery_envelope(
    store: &FileRecoveryStore,
    envelope: &RecoveryEnvelopeDto,
) -> Result<(), String> {
    store.replace(envelope)
}

pub fn persist_committed_exit(
    state: &CurrentGameState,
    store: &FileRecoveryStore,
    mut outcome: ApplicationExitOutcomeDto,
) -> ApplicationExitOutcomeDto {
    let Some(disposition) = outcome.disposition else {
        return outcome;
    };
    if !outcome.committed {
        return outcome;
    }
    match state.persist_flush(store, disposition) {
        Ok(()) => outcome,
        Err(error) => {
            outcome.recovery_persist_error = Some(error.clone());
            outcome.message = format!("Failed to persist current-game recovery: {error}");
            outcome
        }
    }
}

pub fn persist_replacement_snapshot(
    state: &CurrentGameState,
    store: &FileRecoveryStore,
) -> Result<(), String> {
    state.persist_flush(store, ApplicationExitDispositionDto::ExitIncomplete)
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
