use super::*;
use app_model::{
    ExternalSyncPhaseDto as Phase, ExternalSyncSnapshotDto, ExternalSyncSourceDto, ExternalSyncStartDto,
    ExternalSyncUpdateDto, ProviderError, ProviderErrorKind, ProviderImportResult,
    ProviderRequestIdentityDto, ReadboardPhaseDto, ReadboardRuntimeDto, ReadboardSyncPreferencesDto,
    ReadboardSyncStatusDto, YikeSyncPreferencesDto,
};
use go_core::{ReadboardControl, ReadboardFrame};
use provider_core::network::{NetworkOperation, NetworkState};
use provider_yike::sync::YikeSyncWork;
use sgf::{ReadboardSync, ReadboardSyncOutcome, ReadboardViewPreferences};
use std::time::{Duration, Instant};

#[derive(Default)]
pub(super) struct ExternalSyncOwner {
    pub active: Option<SyncSession>,
    pending: Option<PendingStart>,
    next_id: u64,
    revision: u64,
    /// Latest readboard runtime revision consumed; older queued snapshots are stale.
    readboard_revision: u64,
}

pub(super) struct PendingStart {
    id: u64,
    source: PendingSource,
}

enum PendingSource {
    Yike {
        operation: NetworkOperation,
        locator: String,
        preferences: YikeSyncPreferencesDto,
        source_status: Option<String>,
    },
    Readboard(Box<PendingReadboard>),
}

struct PendingReadboard {
    runtime_generation: u64,
    preferences: ReadboardSyncPreferencesDto,
    sync: ReadboardSync,
    status: Option<String>,
    /// Accepted first frame: candidate document plus the engine's cursor/source in it.
    candidate: Option<(CurrentSgfDocument, ReadboardPosition)>,
    /// Set once the candidate entered SGF-07; later frames wait for installation.
    admitted: Option<ReadboardPosition>,
    /// Connection lost while the admitted candidate awaited SGF-07: install it paused.
    lost: Option<String>,
}

#[derive(Clone)]
struct ReadboardPosition {
    selected: NodePath,
    source: NodePath,
    source_move_number: u32,
}

pub(super) struct SyncSession {
    id: u64,
    document_identity: u64,
    phase: Phase,
    failure: Option<ProviderError>,
    source: SessionSource,
}

enum SessionSource {
    Yike(YikeSession),
    Readboard(ReadboardSession),
}

struct YikeSession {
    locator: String,
    source: CurrentSgfDocument,
    source_text: String,
    source_status: Option<String>,
    preferences: YikeSyncPreferencesDto,
    operation: Option<NetworkOperation>,
    next_poll: Instant,
    browser_error: Option<String>,
}

struct ReadboardSession {
    /// Runtime connection whose frames this session accepts; `None` while paused or reconnecting.
    runtime_generation: Option<u64>,
    /// Retry only admits a Ready connection newer than this generation.
    retry_after: Option<u64>,
    preferences: ReadboardSyncPreferencesDto,
    sync: ReadboardSync,
    source: NodePath,
    source_move_number: u32,
    status: Option<String>,
}

impl ExternalSyncOwner {
    pub(super) fn starting_id(&self) -> Option<u64> {
        self.pending.as_ref().map(|pending| pending.id)
    }
    pub(super) fn seal_active(&mut self) {
        // Remove publication authority before signalling cancellation.
        if let Some(session) = self.active.take() {
            self.revision += 1;
            if let SessionSource::Yike(YikeSession {
                operation: Some(operation),
                ..
            }) = session.source
            {
                operation.cancel();
            }
        }
    }
    /// Whether committing this start needs a copy of the candidate as the Yike reconciliation source.
    pub(super) fn start_needs_source(&self, id: u64) -> bool {
        self.pending
            .as_ref()
            .is_some_and(|pending| pending.id == id && matches!(pending.source, PendingSource::Yike { .. }))
    }
    /// Installs the admitted start; returns the readboard cursor for the installed document.
    pub(super) fn install(
        &mut self,
        id: u64,
        document_identity: u64,
        yike_source: Option<(CurrentSgfDocument, String)>,
    ) -> Option<NodePath> {
        let pending = self
            .pending
            .take()
            .filter(|pending| pending.id == id)
            .expect("admitted sync start");
        self.revision += 1;
        let (source, selected, phase, failure) = match pending.source {
            PendingSource::Yike {
                locator,
                preferences,
                source_status,
                ..
            } => {
                let (source, source_text) = yike_source.expect("Yike sync source");
                (
                    SessionSource::Yike(YikeSession {
                        locator,
                        source,
                        source_text,
                        source_status,
                        preferences,
                        operation: None,
                        next_poll: Instant::now(),
                        browser_error: None,
                    }),
                    None,
                    Phase::Syncing,
                    None,
                )
            }
            PendingSource::Readboard(pending) => {
                let position = pending.admitted.expect("admitted readboard candidate");
                let generation = pending.runtime_generation;
                let (runtime_generation, retry_after, phase, failure) = match pending.lost {
                    Some(message) => (
                        None,
                        Some(generation),
                        Phase::ErrorPaused,
                        Some(readboard_failure(message)),
                    ),
                    None => (Some(generation), None, Phase::Syncing, None),
                };
                (
                    SessionSource::Readboard(ReadboardSession {
                        runtime_generation,
                        retry_after,
                        preferences: pending.preferences,
                        sync: pending.sync,
                        source: position.source,
                        source_move_number: position.source_move_number,
                        status: pending.status,
                    }),
                    Some(position.selected),
                    phase,
                    failure,
                )
            }
        };
        self.active = Some(SyncSession {
            id,
            document_identity,
            phase,
            failure,
            source,
        });
        selected
    }
    pub(super) fn cancel_start(&mut self, id: u64) {
        if self.pending.as_ref().is_some_and(|pending| pending.id == id) {
            let pending = self.pending.take().expect("matching start");
            self.revision += 1;
            if let PendingSource::Yike { operation, .. } = pending.source {
                operation.cancel();
            }
        }
    }
    pub(super) fn validate_start(&self, id: u64) -> Result<(), CurrentGameError> {
        self.pending
            .as_ref()
            .filter(|pending| pending.id == id)
            .ok_or_else(sync_stale)?;
        Ok(())
    }
    pub(super) fn snapshot(&self) -> ExternalSyncSnapshotDto {
        let pending = self.pending.as_ref();
        let mut snapshot = ExternalSyncSnapshotDto {
            revision: self.revision,
            starting_id: pending.map(|pending| pending.id),
            phase: if pending.is_some() {
                Phase::Starting
            } else {
                Phase::Idle
            },
            ..Default::default()
        };
        if let Some(session) = &self.active {
            snapshot.session_id = Some(session.id);
            snapshot.phase = session.phase;
            snapshot.document_identity = Some(session.document_identity);
            snapshot.failure = session.failure.clone();
            match &session.source {
                SessionSource::Yike(yike) => {
                    let retry_count = yike.operation.as_ref().map_or(0, NetworkOperation::retry_count);
                    if retry_count > 0 && session.phase == Phase::Syncing {
                        snapshot.phase = Phase::Retrying;
                    }
                    snapshot.source = Some(ExternalSyncSourceDto::Yike);
                    snapshot.locator = Some(yike.locator.clone());
                    snapshot.source_status = yike.source_status.clone();
                    snapshot.source_tip = Some(yike.source.default_selected_path());
                    snapshot.request_identity = yike.operation.as_ref().map(NetworkOperation::identity);
                    snapshot.retry_count = retry_count;
                    snapshot.browser_error = yike.browser_error.clone();
                    snapshot.preferences = yike.preferences.clone();
                }
                SessionSource::Readboard(readboard) => {
                    snapshot.source = Some(ExternalSyncSourceDto::Readboard);
                    snapshot.source_tip = Some(readboard.source.clone());
                    snapshot.source_status = readboard.status.clone();
                    snapshot.readboard = Some(ReadboardSyncStatusDto {
                        preferences: readboard.preferences,
                        runtime_generation: readboard.runtime_generation,
                        source_move_number: Some(readboard.source_move_number),
                    });
                }
            }
        } else if let Some(pending) = pending {
            match &pending.source {
                PendingSource::Yike {
                    locator, preferences, ..
                } => {
                    snapshot.source = Some(ExternalSyncSourceDto::Yike);
                    snapshot.locator = Some(locator.clone());
                    snapshot.preferences = preferences.clone();
                }
                PendingSource::Readboard(readboard) => {
                    snapshot.source = Some(ExternalSyncSourceDto::Readboard);
                    snapshot.source_status = readboard.status.clone();
                    snapshot.readboard = Some(ReadboardSyncStatusDto {
                        preferences: readboard.preferences,
                        runtime_generation: Some(readboard.runtime_generation),
                        source_move_number: None,
                    });
                }
            }
        }
        snapshot
    }
    fn yike_mut(&mut self) -> Option<(&mut Phase, &mut Option<ProviderError>, u64, &mut YikeSession)> {
        let session = self.active.as_mut()?;
        match &mut session.source {
            SessionSource::Yike(yike) => Some((
                &mut session.phase,
                &mut session.failure,
                session.document_identity,
                yike,
            )),
            SessionSource::Readboard(_) => None,
        }
    }
    fn pending_readboard(&mut self, generation: u64) -> Option<&mut PendingReadboard> {
        match self.pending.as_mut().map(|pending| &mut pending.source) {
            Some(PendingSource::Readboard(pending)) if pending.runtime_generation == generation => {
                Some(&mut **pending)
            }
            _ => None,
        }
    }
    /// Applies one runtime snapshot newer than any consumed: losing a bound connection pauses
    /// (or cancels an unadmitted Start), a newer Ready resumes a Retry. Returns whether it changed.
    fn observe_runtime(&mut self, runtime: &ReadboardRuntimeDto) -> bool {
        if runtime.revision <= self.readboard_revision {
            return false;
        }
        self.readboard_revision = runtime.revision;
        let ready = runtime.phase == ReadboardPhaseDto::Ready;
        let lost = |bound: u64| !ready || bound != runtime.generation;
        let mut changed = false;
        if let Some(PendingSource::Readboard(pending)) =
            self.pending.as_mut().map(|pending| &mut pending.source)
        {
            if lost(pending.runtime_generation) {
                if pending.admitted.is_some() {
                    // The SGF-07 decision is open; a decided candidate installs paused.
                    pending
                        .lost
                        .get_or_insert_with(|| format!("readboard connection lost: {}", runtime.message));
                } else {
                    let id = self.starting_id().expect("pending readboard start");
                    self.cancel_start(id);
                }
                changed = true;
            }
        }
        if let Some(session) = self.active.as_mut() {
            if let SessionSource::Readboard(readboard) = &mut session.source {
                match (readboard.runtime_generation, readboard.retry_after) {
                    (Some(bound), _) if lost(bound) => {
                        readboard.runtime_generation = None;
                        readboard.retry_after = Some(bound);
                        session.phase = Phase::ErrorPaused;
                        session.failure = Some(readboard_failure(format!(
                            "readboard connection lost: {}",
                            runtime.message
                        )));
                        readboard.status = None;
                        changed = true;
                    }
                    (None, Some(after)) if session.phase == Phase::Retrying && runtime.generation > after => {
                        if ready {
                            readboard.runtime_generation = Some(runtime.generation);
                            readboard.retry_after = None;
                            readboard.sync.control(ReadboardControl::Start { size: None });
                            session.phase = Phase::Syncing;
                            readboard.status = Some(
                                "readboard reconnected; the next board is verified against the current game."
                                    .into(),
                            );
                            changed = true;
                        } else if !matches!(
                            runtime.phase,
                            ReadboardPhaseDto::Starting
                                | ReadboardPhaseDto::Stopping
                                | ReadboardPhaseDto::Stopped
                                | ReadboardPhaseDto::Idle
                        ) {
                            readboard.retry_after = None;
                            session.phase = Phase::ErrorPaused;
                            session.failure = Some(readboard_failure(format!(
                                "readboard reconnect failed: {}",
                                runtime.message
                            )));
                            readboard.status = None;
                            changed = true;
                        }
                    }
                    _ => {}
                }
            }
        }
        if changed {
            self.revision += 1;
        }
        changed
    }
}

pub(super) fn sync_stale() -> CurrentGameError {
    CurrentGameError {
        kind: CurrentGameErrorKind::DepartureBlocked,
        message: "External sync ownership changed; use the current session.".into(),
    }
}

fn view(preferences: &ReadboardSyncPreferencesDto) -> ReadboardViewPreferences {
    ReadboardViewPreferences {
        always_sync: preferences.always_sync,
        jump_to_last: preferences.jump_to_last,
    }
}

fn readboard_failure(message: impl Into<String>) -> ProviderError {
    ProviderError {
        kind: ProviderErrorKind::RuntimeUnavailable,
        message: message.into(),
    }
}

impl CurrentGameState {
    pub fn external_sync_snapshot(&self) -> ExternalSyncSnapshotDto {
        self.holder
            .lock()
            .expect("current game state")
            .external_sync
            .snapshot()
    }

    pub fn begin_external_start(
        &self,
        network: &NetworkState,
        locator: String,
        preferences: YikeSyncPreferencesDto,
    ) -> Result<u64, CurrentGameError> {
        let mut holder = self.holder.lock().expect("current game state");
        holder.ensure_review_access()?;
        if holder.external_sync.pending.is_some() {
            return Err(sync_stale());
        }
        let operation_id = network
            .begin(network.snapshot().policy_revision, holder.document_identity)
            .map_err(sync_network_error)?;
        let operation = network.operation(&operation_id).map_err(sync_network_error)?;
        holder.external_sync.next_id += 1;
        let id = holder.external_sync.next_id;
        holder.external_sync.pending = Some(PendingStart {
            id,
            source: PendingSource::Yike {
                operation,
                locator,
                preferences,
                source_status: None,
            },
        });
        holder.external_sync.revision += 1;
        Ok(id)
    }

    pub fn external_start_request(&self, id: u64) -> Result<(NetworkOperation, String), CurrentGameError> {
        let holder = self.holder.lock().expect("current game state");
        match holder
            .external_sync
            .pending
            .as_ref()
            .filter(|pending| pending.id == id)
            .map(|pending| &pending.source)
        {
            Some(PendingSource::Yike {
                operation, locator, ..
            }) => Ok((operation.clone(), locator.clone())),
            _ => Err(sync_stale()),
        }
    }

    pub fn cancel_external_start(&self, id: u64) -> ExternalSyncSnapshotDto {
        let mut holder = self.holder.lock().expect("current game state");
        // Protected departure owns the candidate until atomic install or explicit abort.
        if holder.departure.as_ref().is_some_and(|session| {
            matches!(session.phase, departure::DeparturePhase::Protected)
                && matches!(session.target, departure::DepartureTarget::Replacement {
                    external_start: Some(start), ..
                } if start.get() == id)
        }) {
            return holder.external_sync.snapshot();
        }
        if holder
            .external_sync
            .pending
            .as_ref()
            .is_some_and(|pending| pending.id == id)
        {
            holder.external_sync.cancel_start(id);
            if holder.departure.as_ref().is_some_and(|departure| matches!(departure.target, departure::DepartureTarget::Replacement { external_start: Some(start), .. } if start.get() == id)
                && matches!(departure.phase, departure::DeparturePhase::AwaitingDecision)) {
                holder.departure = None;
            }
            self.external_sync_changed.notify_all();
        }
        holder.external_sync.snapshot()
    }

    pub fn prepare_external_candidate(
        &self,
        id: u64,
        result: ProviderImportResult,
    ) -> Result<ExternalSyncStartDto, CurrentGameError> {
        let candidate = CurrentSgfDocument::open(&result.sgf_text)?;
        let (lease, document_identity) = {
            let mut holder = self.holder.lock().expect("current game state");
            let Some(PendingSource::Yike {
                operation,
                locator,
                source_status,
                ..
            }) = holder
                .external_sync
                .pending
                .as_mut()
                .filter(|pending| pending.id == id)
                .map(|pending| &mut pending.source)
            else {
                return Err(sync_stale());
            };
            if result.metadata.source_url.as_deref() != Some(locator.as_str()) {
                return Err(sync_stale());
            }
            operation.lease().check().map_err(sync_network_error)?;
            *source_status = result.metadata.provider_status;
            (operation.lease(), operation.identity().document_identity)
        };
        let admission = self.admit_departure(departure::DepartureTarget::Replacement {
            candidate,
            candidate_path: None,
            network_lease: Some((lease, document_identity)),
            external_start: Some(std::num::NonZeroU64::new(id).expect("reserved sync start")),
        })?;
        Ok(ExternalSyncStartDto {
            start_id: id,
            admission,
        })
    }

    pub fn next_external_poll(&self, network: &NetworkState) -> YikeSyncWork {
        let mut holder = self.holder.lock().expect("current game state");
        if holder.departure.is_some() || holder.edits_blocked {
            return YikeSyncWork::Wait(Some(Duration::from_millis(100)));
        }
        let Some((phase, failure, document_identity, session)) = holder.external_sync.yike_mut() else {
            return YikeSyncWork::Wait(None);
        };
        if *phase == Phase::ErrorPaused {
            return YikeSyncWork::Wait(None);
        }
        if session.operation.is_some() {
            return YikeSyncWork::Wait(Some(Duration::from_millis(100)));
        }
        let remaining = session.next_poll.saturating_duration_since(Instant::now());
        if !remaining.is_zero() {
            return YikeSyncWork::Wait(Some(remaining));
        }
        let work = match network.begin_sync(network.snapshot().policy_revision, document_identity) {
            Ok(operation) => {
                let locator = session.locator.clone();
                session.operation = Some(operation.clone());
                *phase = Phase::Syncing;
                YikeSyncWork::Fetch { operation, locator }
            }
            Err(error) => {
                *failure = Some(error);
                *phase = Phase::ErrorPaused;
                YikeSyncWork::Wait(None)
            }
        };
        holder.external_sync.revision += 1;
        work
    }

    pub fn complete_external_poll(
        &self,
        identity: ProviderRequestIdentityDto,
        result: Result<ProviderImportResult, ProviderError>,
        policy_revision: u64,
    ) -> ExternalSyncUpdateDto {
        let parsed = result.and_then(|result| {
            let document = CurrentSgfDocument::open(&result.sgf_text).map_err(|error| ProviderError {
                kind: ProviderErrorKind::ParseFailed,
                message: error.message,
            })?;
            let text = document.serialize().map_err(|error| ProviderError {
                kind: ProviderErrorKind::ParseFailed,
                message: error.message,
            })?;
            Ok((result, document, text))
        });
        let mut holder = self.holder.lock().expect("current game state");
        let mut current = None;
        let applies = holder.external_sync.active.as_ref().is_some_and(|session| session.document_identity == holder.document_identity
            && matches!(&session.source, SessionSource::Yike(yike) if yike.operation.as_ref().is_some_and(|operation| operation.identity() == identity)));
        if applies {
            let mut session = holder.external_sync.active.take().expect("matching session");
            let SessionSource::Yike(yike) = &mut session.source else {
                unreachable!("matched Yike session")
            };
            let operation = yike.operation.take().expect("matching operation");
            let interval = yike
                .preferences
                .interval_seconds
                .max(if yike.locator.contains("/#/unite/") { 5 } else { 1 });
            yike.next_poll = Instant::now() + Duration::from_secs(u64::from(interval));
            if policy_revision != identity.policy_revision {
                yike.next_poll = Instant::now();
            } else if holder.departure.is_none() && !holder.edits_blocked {
                let applied = operation
                    .lease()
                    .with_valid(|| -> Result<(), ProviderError> {
                        let (result, document, text) = parsed?;
                        if result.metadata.source_url.as_deref() != Some(yike.locator.as_str()) {
                            return Err(ProviderError {
                                kind: ProviderErrorKind::InvalidPayload,
                                message: "Yike returned a different public source.".into(),
                            });
                        }
                        if text != yike.source_text {
                            let (merged, selected) = holder
                                .document
                                .as_ref()
                                .ok_or_else(no_current_game)
                                .and_then(|existing| {
                                    existing.reconcile_external_source(
                                        &yike.source,
                                        &document,
                                        &holder.selected_path,
                                        yike.preferences.jump_to_last,
                                    )
                                })
                                .map_err(|error| ProviderError {
                                    kind: ProviderErrorKind::InvalidPayload,
                                    message: error.message,
                                })?;
                            holder.document = Some(merged);
                            holder.selected_path = selected;
                            holder.generation = holder.generation.saturating_add(1);
                            holder.mark_nonhistory_change();
                            holder.bump_snapshot();
                            self.follow_continuous_position(&mut holder);
                            self.note_recovery(&holder);
                            current = holder.current_result().ok();
                            yike.source = document;
                            yike.source_text = text;
                        }
                        yike.source_status = result.metadata.provider_status;
                        Ok(())
                    })
                    .and_then(|result| result);
                match applied {
                    Ok(()) => {
                        session.failure = None;
                        session.phase = Phase::Syncing;
                    }
                    Err(error) => {
                        session.failure = Some(error);
                        session.phase = Phase::ErrorPaused;
                    }
                }
            }
            holder.external_sync.active = Some(session);
            holder.external_sync.revision += 1;
        }
        ExternalSyncUpdateDto {
            sync: holder.external_sync.snapshot(),
            current,
        }
    }

    /// Yike: next poll starts a new request generation. Readboard: the caller restarts the
    /// runtime; only a newer Ready connection resumes the session.
    pub fn retry_external_sync(&self, session_id: u64) -> Result<ExternalSyncSnapshotDto, CurrentGameError> {
        let mut holder = self.holder.lock().expect("current game state");
        holder.ensure_review_access()?;
        let session = holder
            .external_sync
            .active
            .as_mut()
            .filter(|session| session.id == session_id && session.phase == Phase::ErrorPaused)
            .ok_or_else(sync_stale)?;
        session.failure = None;
        match &mut session.source {
            SessionSource::Yike(yike) => {
                if let Some(operation) = yike.operation.take() {
                    operation.cancel();
                }
                session.phase = Phase::Syncing;
                yike.next_poll = Instant::now();
            }
            SessionSource::Readboard(readboard) => {
                session.phase = Phase::Retrying;
                readboard.retry_after = Some(
                    readboard
                        .runtime_generation
                        .take()
                        .unwrap_or(0)
                        .max(readboard.retry_after.unwrap_or(0)),
                );
                readboard.status = Some(
                    "Reconnecting readboard; the game stays read-only until a new Ready connection.".into(),
                );
            }
        }
        holder.external_sync.revision += 1;
        Ok(holder.external_sync.snapshot())
    }

    pub fn stop_external_sync(&self, session_id: u64) -> Result<ExternalSyncUpdateDto, CurrentGameError> {
        let mut holder = self.holder.lock().expect("current game state");
        holder.ensure_review_access()?;
        if !holder
            .external_sync
            .active
            .as_ref()
            .is_some_and(|session| session.id == session_id)
        {
            return Err(sync_stale());
        }
        holder.external_sync.seal_active();
        if let Some(id) = holder.external_sync.snapshot().starting_id {
            holder.external_sync.cancel_start(id);
        }
        self.external_sync_changed.notify_all();
        Ok(ExternalSyncUpdateDto {
            sync: holder.external_sync.snapshot(),
            current: holder.current_result().ok(),
        })
    }

    pub fn seal_external_sync_for_exit(&self) {
        let mut holder = self.holder.lock().expect("current game state");
        holder.external_sync.seal_active();
        if let Some(id) = holder.external_sync.starting_id() {
            holder.external_sync.cancel_start(id);
        }
        holder.external_sync.revision += 1;
        self.external_sync_changed.notify_all();
    }

    pub fn external_browser_locator(&self, session_id: u64) -> Result<String, CurrentGameError> {
        let holder = self.holder.lock().expect("current game state");
        match holder
            .external_sync
            .active
            .as_ref()
            .filter(|session| session.id == session_id)
            .map(|session| &session.source)
        {
            Some(SessionSource::Yike(yike)) => Ok(yike.locator.clone()),
            _ => Err(sync_stale()),
        }
    }

    pub fn external_browser_result(&self, session_id: u64, error: Option<String>) -> ExternalSyncSnapshotDto {
        let mut holder = self.holder.lock().expect("current game state");
        if let Some(SyncSession {
            source: SessionSource::Yike(yike),
            ..
        }) = holder
            .external_sync
            .active
            .as_mut()
            .filter(|session| session.id == session_id)
        {
            yike.browser_error = error;
            holder.external_sync.revision += 1;
        }
        holder.external_sync.snapshot()
    }

    pub fn update_external_preferences(
        &self,
        preferences: YikeSyncPreferencesDto,
    ) -> ExternalSyncSnapshotDto {
        let mut holder = self.holder.lock().expect("current game state");
        if let Some((_, _, _, session)) = holder.external_sync.yike_mut() {
            session.preferences.interval_seconds = preferences.interval_seconds;
            session.preferences.jump_to_last = preferences.jump_to_last;
            session.preferences.mute = preferences.mute;
            session.next_poll = Instant::now();
            holder.external_sync.revision += 1;
        }
        holder.external_sync.snapshot()
    }

    /// Reserves a readboard start bound to one Ready runtime connection.
    pub fn begin_readboard_start(
        &self,
        runtime: &ReadboardRuntimeDto,
        preferences: ReadboardSyncPreferencesDto,
    ) -> Result<u64, CurrentGameError> {
        let mut holder = self.holder.lock().expect("current game state");
        holder.ensure_review_access()?;
        if holder.external_sync.pending.is_some() {
            return Err(sync_stale());
        }
        if runtime.phase != ReadboardPhaseDto::Ready {
            return Err(CurrentGameError {
                kind: CurrentGameErrorKind::DepartureBlocked,
                message: "Start readboard and wait for Ready before synchronizing.".into(),
            });
        }
        holder.external_sync.next_id += 1;
        let id = holder.external_sync.next_id;
        // Start consumed this snapshot directly; apply it before fencing older queued ones.
        holder.external_sync.observe_runtime(runtime);
        holder.external_sync.pending = Some(PendingStart {
            id,
            source: PendingSource::Readboard(Box::new(PendingReadboard {
                runtime_generation: runtime.generation,
                preferences,
                sync: ReadboardSync::new(),
                status: Some(
                    "Waiting for a readboard board frame; press sync in readboard if it is not sending."
                        .into(),
                ),
                candidate: None,
                admitted: None,
                lost: None,
            })),
        });
        holder.external_sync.revision += 1;
        Ok(id)
    }

    /// Blocks until the first provable frame yields a candidate, then enters SGF-07 once.
    pub fn prepare_readboard_candidate(
        &self,
        id: u64,
        budget: Duration,
    ) -> Result<ExternalSyncStartDto, CurrentGameError> {
        let deadline = Instant::now() + budget;
        let mut holder = self.holder.lock().expect("current game state");
        let candidate = loop {
            let pending = match holder
                .external_sync
                .pending
                .as_mut()
                .filter(|pending| pending.id == id)
                .map(|pending| &mut pending.source)
            {
                Some(PendingSource::Readboard(pending)) if pending.admitted.is_none() => pending,
                _ => return Err(sync_stale()),
            };
            if let Some((document, position)) = pending.candidate.take() {
                pending.admitted = Some(position);
                break document;
            }
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Err(CurrentGameError { kind: CurrentGameErrorKind::DepartureBlocked, message: "readboard did not send a provable board before the start deadline; press sync in readboard and Start again.".into() });
            }
            holder = self
                .external_sync_changed
                .wait_timeout(holder, remaining)
                .expect("external sync wait")
                .0;
        };
        drop(holder);
        let admission = self.admit_departure(departure::DepartureTarget::Replacement {
            candidate,
            candidate_path: None,
            network_lease: None,
            external_start: Some(std::num::NonZeroU64::new(id).expect("reserved sync start")),
        })?;
        Ok(ExternalSyncStartDto {
            start_id: id,
            admission,
        })
    }

    /// Routes one decoded frame of a runtime connection to the pending start or the active session.
    pub fn observe_readboard_frame(
        &self,
        generation: u64,
        frame: &ReadboardFrame,
    ) -> Option<ExternalSyncUpdateDto> {
        let mut holder = self.holder.lock().expect("current game state");
        let state = &mut *holder;
        if let Some(pending) = state.external_sync.pending_readboard(generation) {
            if pending.admitted.is_some() || pending.candidate.is_some() {
                return None;
            }
            let view = view(&pending.preferences);
            match pending
                .sync
                .apply(state.document.as_ref(), &state.selected_path, frame, view)
            {
                Ok(ReadboardSyncOutcome::Accepted {
                    document: changed,
                    selected,
                    source,
                    source_move_number,
                    ..
                }) => {
                    let candidate = changed
                        .map(|document| *document)
                        .or_else(|| state.document.clone())?;
                    pending.candidate = Some((
                        candidate,
                        ReadboardPosition {
                            selected,
                            source,
                            source_move_number,
                        },
                    ));
                    pending.status = Some("Board received; confirm replacing the current game.".into());
                }
                Ok(ReadboardSyncOutcome::Hold) => {
                    pending.status =
                        Some("Conflicting board held once; waiting for the same board again.".into())
                }
                Ok(ReadboardSyncOutcome::Ignored) => {
                    pending.status =
                        Some("Yike-platform frames are synchronized by Yike sync, not readboard.".into())
                }
                Err(error) => pending.status = Some(format!("Board frame rejected: {}", error.message)),
            }
            state.external_sync.revision += 1;
            self.external_sync_changed.notify_all();
            return Some(ExternalSyncUpdateDto {
                sync: state.external_sync.snapshot(),
                current: None,
            });
        }
        if state.departure.is_some() || state.edits_blocked {
            return None;
        }
        let identity = state.document_identity;
        let session = state
            .external_sync
            .active
            .as_mut()
            .filter(|session| session.document_identity == identity && session.phase == Phase::Syncing)?;
        let SessionSource::Readboard(readboard) = &mut session.source else {
            return None;
        };
        if readboard.runtime_generation != Some(generation) {
            return None;
        }
        let before = (
            readboard.status.clone(),
            readboard.source.clone(),
            readboard.source_move_number,
        );
        let outcome = readboard.sync.apply(
            state.document.as_ref(),
            &state.selected_path,
            frame,
            view(&readboard.preferences),
        );
        let mut next = None;
        match outcome {
            Ok(ReadboardSyncOutcome::Accepted {
                document,
                selected: cursor,
                source,
                source_move_number,
                rebuilt,
            }) => {
                readboard.source = source;
                readboard.source_move_number = source_move_number;
                readboard.status = rebuilt.then(|| {
                    "Rebuilt from a static readboard snapshot; earlier move order was not provable.".into()
                });
                if document.is_some() || cursor != state.selected_path {
                    next = Some((document, cursor));
                }
            }
            Ok(ReadboardSyncOutcome::Hold) => {
                readboard.status =
                    Some("Conflicting board held once; it rebuilds if readboard repeats it.".into())
            }
            Ok(ReadboardSyncOutcome::Ignored) => {
                readboard.status =
                    Some("Yike-platform frames are synchronized by Yike sync, not readboard.".into())
            }
            Err(error) => {
                readboard.runtime_generation = None;
                readboard.retry_after = Some(generation);
                session.failure = Some(ProviderError {
                    kind: ProviderErrorKind::InvalidPayload,
                    message: error.message,
                });
                session.phase = Phase::ErrorPaused;
            }
        }
        let unchanged = next.is_none()
            && session.phase == Phase::Syncing
            && matches!(&session.source,
            SessionSource::Readboard(after) if after.status == before.0 && after.source == before.1 && after.source_move_number == before.2);
        if unchanged {
            return None;
        }
        let mut current = None;
        if let Some((document, cursor)) = next {
            if let Some(document) = document {
                holder.document = Some(*document);
                holder.generation = holder.generation.saturating_add(1);
                holder.mark_nonhistory_change();
            }
            holder.selected_path = cursor;
            holder.bump_snapshot();
            self.follow_continuous_position(&mut holder);
            self.note_recovery(&holder);
            current = holder.current_result().ok();
        }
        holder.external_sync.revision += 1;
        Some(ExternalSyncUpdateDto {
            sync: holder.external_sync.snapshot(),
            current,
        })
    }

    /// readboard's own controls; its Stop/End ends the bound session exactly like Next's Stop.
    pub fn readboard_control(
        &self,
        generation: u64,
        control: ReadboardControl,
    ) -> Option<ExternalSyncSnapshotDto> {
        let mut holder = self.holder.lock().expect("current game state");
        let stop = matches!(control, ReadboardControl::StopSync | ReadboardControl::EndSync);
        if let Some(pending) = holder.external_sync.pending_readboard(generation) {
            if pending.admitted.is_some() {
                return None;
            }
            if stop {
                let id = holder
                    .external_sync
                    .starting_id()
                    .expect("pending readboard start");
                holder.external_sync.cancel_start(id);
                self.external_sync_changed.notify_all();
                return Some(holder.external_sync.snapshot());
            }
            if control != ReadboardControl::Sync {
                if pending.candidate.take().is_some() {
                    // The discarded candidate was never installed; its snapshot metadata is stale.
                    pending.sync = ReadboardSync::new();
                }
                pending.sync.control(control);
            }
        } else {
            let SyncSession {
                source: SessionSource::Readboard(readboard),
                ..
            } = holder.external_sync.active.as_mut()?
            else {
                return None;
            };
            if readboard.runtime_generation != Some(generation) {
                return None;
            }
            if stop {
                holder.external_sync.seal_active();
                return Some(holder.external_sync.snapshot());
            }
            readboard.sync.control(control);
            readboard.status = None;
        }
        holder.external_sync.revision += 1;
        Some(holder.external_sync.snapshot())
    }

    pub fn readboard_frame_rejected(&self, generation: u64, reason: &str) -> Option<ExternalSyncSnapshotDto> {
        let mut holder = self.holder.lock().expect("current game state");
        let status = Some(format!("Incomplete or invalid readboard frame ignored: {reason}"));
        if let Some(pending) = holder.external_sync.pending_readboard(generation) {
            pending.status = status;
        } else {
            let SyncSession {
                source: SessionSource::Readboard(readboard),
                ..
            } = holder.external_sync.active.as_mut()?
            else {
                return None;
            };
            if readboard.runtime_generation != Some(generation) {
                return None;
            }
            readboard.status = status;
        }
        holder.external_sync.revision += 1;
        Some(holder.external_sync.snapshot())
    }

    /// Runtime lifecycle edge from the bridge. Snapshots older than one already consumed
    /// (for example queued behind a direct Start read) are stale.
    pub fn readboard_runtime_changed(
        &self,
        runtime: &ReadboardRuntimeDto,
    ) -> Option<ExternalSyncSnapshotDto> {
        let mut holder = self.holder.lock().expect("current game state");
        if !holder.external_sync.observe_runtime(runtime) {
            return None;
        }
        self.external_sync_changed.notify_all();
        Some(holder.external_sync.snapshot())
    }

    /// A Retry whose runtime restart could not even begin returns to Error-paused.
    pub fn readboard_retry_failed(&self, session_id: u64, message: String) {
        let mut holder = self.holder.lock().expect("current game state");
        let Some(session) = holder
            .external_sync
            .active
            .as_mut()
            .filter(|session| session.id == session_id && session.phase == Phase::Retrying)
        else {
            return;
        };
        if matches!(session.source, SessionSource::Readboard(_)) {
            session.phase = Phase::ErrorPaused;
            session.failure = Some(readboard_failure(message));
            holder.external_sync.revision += 1;
        }
    }

    pub fn update_readboard_preferences(
        &self,
        preferences: ReadboardSyncPreferencesDto,
    ) -> ExternalSyncSnapshotDto {
        let mut holder = self.holder.lock().expect("current game state");
        let owner = &mut holder.external_sync;
        match (
            owner.active.as_mut().map(|session| &mut session.source),
            owner.pending.as_mut().map(|pending| &mut pending.source),
        ) {
            (Some(SessionSource::Readboard(readboard)), _) => readboard.preferences = preferences,
            (_, Some(PendingSource::Readboard(pending))) => pending.preferences = preferences,
            _ => return owner.snapshot(),
        }
        owner.revision += 1;
        owner.snapshot()
    }

    /// Runtime generation that may receive `loss`, only for an active readboard session with focus enabled.
    pub fn readboard_focus_generation(&self) -> Option<u64> {
        let holder = self.holder.lock().expect("current game state");
        match holder.external_sync.active.as_ref() {
            Some(SyncSession {
                phase: Phase::Syncing,
                source: SessionSource::Readboard(readboard),
                ..
            }) if readboard.preferences.focus => readboard.runtime_generation,
            _ => None,
        }
    }
}

fn sync_network_error(error: ProviderError) -> CurrentGameError {
    CurrentGameError {
        kind: CurrentGameErrorKind::DepartureBlocked,
        message: error.message,
    }
}
