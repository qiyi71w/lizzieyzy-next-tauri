use super::*;
use app_model::{
    AnalysisJobStartedDto, ApplicationExitDispositionDto, ApplicationExitOutcomeDto,
    ApplicationTeardownAttemptDto, DocumentDepartureAdmissionDto, DocumentDepartureOutcomeDto,
};

pub(super) struct DepartureSession {
    pub id: u64,
    pub phase: DeparturePhase,
    pub target: DepartureTarget,
}

pub(super) enum DepartureTarget {
    Replacement {
        candidate: CurrentSgfDocument,
        candidate_path: Option<String>,
        network_lease: Option<(provider_core::network::NetworkLease, u64)>,
        external_start: Option<std::num::NonZeroU64>,
    },
    ApplicationExit,
}

pub(super) enum DeparturePhase {
    AwaitingDecision,
    Protected,
    TearingDown {
        disposition: ApplicationExitDispositionDto,
    },
}

impl CurrentGameState {
    pub fn prepare_replacement(
        &self,
        sgf_text: &str,
        native_path: Option<String>,
    ) -> Result<DocumentDepartureAdmissionDto, CurrentGameError> {
        let candidate = CurrentSgfDocument::open(sgf_text)?;
        self.admit_departure(DepartureTarget::Replacement {
            candidate,
            candidate_path: native_path,
            network_lease: None,
            external_start: None,
        })
    }

    pub fn document_identity(&self) -> u64 {
        self.holder.lock().expect("current game state").document_identity
    }

    pub fn prepare_provider_replacement(
        &self,
        sgf_text: &str,
        lease: provider_core::network::NetworkLease,
        document_identity: u64,
    ) -> Result<DocumentDepartureAdmissionDto, CurrentGameError> {
        lease.check().map_err(|error| CurrentGameError {
            kind: CurrentGameErrorKind::DepartureBlocked,
            message: error.message,
        })?;
        let candidate = CurrentSgfDocument::open(sgf_text)?;
        self.admit_departure(DepartureTarget::Replacement {
            candidate,
            candidate_path: None,
            network_lease: Some((lease, document_identity)),
            external_start: None,
        })
    }

    pub fn prepare_exit(&self) -> Result<DocumentDepartureAdmissionDto, CurrentGameError> {
        self.admit_departure(DepartureTarget::ApplicationExit)
    }

    pub fn cancel_replacement(
        &self,
        departure_id: u64,
    ) -> Result<DocumentDepartureOutcomeDto, CurrentGameError> {
        let mut holder = self.holder.lock().expect("current game state");
        match holder.departure.as_ref() {
            Some(session) if session.id == departure_id => {
                if !matches!(session.phase, DeparturePhase::AwaitingDecision) {
                    return Err(departure_blocked());
                }
            }
            _ => return Err(departure_in_progress()),
        }
        if let Some(DepartureSession {
            target:
                DepartureTarget::Replacement {
                    external_start: Some(id),
                    ..
                },
            ..
        }) = holder.departure.as_ref()
        {
            let id = id.get();
            holder.external_sync.cancel_start(id);
        }
        holder.departure = None;
        Ok(DocumentDepartureOutcomeDto {
            committed: false,
            analysis_stopped: false,
            current: None,
            message: "Replacement cancelled.".to_string(),
        })
    }

    pub fn begin_protected_commit(
        &self,
        departure_id: u64,
        closed_jobs: &[AnalysisJobStartedDto],
    ) -> Result<(), CurrentGameError> {
        let mut holder = self.holder.lock().expect("current game state");
        match holder.departure.as_mut() {
            Some(session) if session.id == departure_id => {
                if matches!(session.phase, DeparturePhase::TearingDown { .. }) {
                    return Err(departure_blocked());
                }
                session.phase = DeparturePhase::Protected;
            }
            _ => return Err(departure_in_progress()),
        }
        holder.edits_blocked = true;
        if let Some(manager) = self.analysis_manager.get() {
            manager.begin_continuous_departure();
        }
        for job in closed_jobs {
            holder
                .closed_jobs
                .insert((job.run_id.clone(), job.job_id.clone()));
        }
        Ok(())
    }

    pub fn commit_replacement(
        &self,
        departure_id: u64,
    ) -> Result<DocumentDepartureOutcomeDto, CurrentGameError> {
        let mut holder = self.holder.lock().expect("current game state");
        match holder.departure.as_ref() {
            Some(session)
                if session.id == departure_id && matches!(session.phase, DeparturePhase::Protected) => {}
            Some(session) if session.id == departure_id => return Err(departure_blocked()),
            _ => return Err(departure_in_progress()),
        }
        let session = holder.departure.take().expect("departure required");
        let DepartureTarget::Replacement {
            candidate,
            candidate_path,
            network_lease,
            external_start,
        } = session.target
        else {
            holder.departure = Some(session);
            return Err(departure_blocked());
        };
        let external_start = external_start.map(std::num::NonZeroU64::get);
        let yike_source = match external_start.filter(|id| holder.external_sync.start_needs_source(*id)) {
            Some(_) => Some((candidate.clone(), candidate.serialize()?)),
            None => None,
        };
        // Document and session install together, inside the provider lease when one exists.
        let install = |holder: &mut CurrentGameHolder| -> Result<CurrentGameResultDto, CurrentGameError> {
            let current = holder.install_document(candidate, candidate_path)?;
            let Some(id) = external_start else {
                return Ok(current);
            };
            let document_identity = holder.document_identity;
            match holder.external_sync.install(id, document_identity, yike_source) {
                // A readboard candidate carries the engine's cursor inside the installed document.
                Some(selected) => holder.select_path(selected),
                None => Ok(current),
            }
        };
        let current = if let Some((lease, _)) = network_lease {
            match lease.with_valid(|| install(&mut holder)) {
                Ok(current) => current?,
                Err(error) => {
                    holder.edits_blocked = false;
                    if let Some(id) = external_start {
                        holder.external_sync.cancel_start(id);
                    }
                    self.match_departure.notify_all();
                    if let Some(manager) = self.analysis_manager.get() {
                        manager.finish_continuous_departure(false);
                    }
                    return Ok(DocumentDepartureOutcomeDto {
                        committed: false,
                        analysis_stopped: true,
                        current: Some(holder.current_result()?),
                        message: error.message,
                    });
                }
            }
        } else {
            install(&mut holder)?
        };
        self.note_recovery(&holder);
        if let Some(manager) = self.analysis_manager.get() {
            manager.clear_continuous_position();
            self.follow_continuous_position(&mut holder);
            manager.finish_continuous_departure(true);
        }
        Ok(DocumentDepartureOutcomeDto {
            committed: true,
            analysis_stopped: true,
            current: Some(current),
            message: "Replacement committed.".to_string(),
        })
    }

    pub fn abort_protected_commit(
        &self,
        departure_id: u64,
        selected_path: NodePath,
    ) -> Result<DocumentDepartureOutcomeDto, CurrentGameError> {
        let mut holder = self.holder.lock().expect("current game state");
        match holder.departure.as_ref() {
            Some(session)
                if session.id == departure_id && matches!(session.phase, DeparturePhase::Protected) => {}
            Some(session) if session.id == departure_id => return Err(departure_in_progress()),
            _ => return Err(departure_in_progress()),
        }
        holder.departure = None;
        if let Some(id) = holder.external_sync.starting_id() {
            holder.external_sync.cancel_start(id);
        }
        holder.edits_blocked = false;
        self.match_departure.notify_all();
        holder.exit_disposition = None;
        let current = holder.result_dto(selected_path)?;
        if let Some(manager) = self.analysis_manager.get() {
            manager.finish_continuous_departure(false);
        }
        Ok(DocumentDepartureOutcomeDto {
            committed: false,
            analysis_stopped: true,
            current: Some(current),
            message: "Save cancelled. Analysis is stopped; restart it explicitly.".to_string(),
        })
    }

    pub fn save_sealed_departure(
        &self,
        departure_id: u64,
        path: String,
        selected_path: NodePath,
    ) -> Result<CurrentGameResultDto, String> {
        {
            let holder = self.holder.lock().expect("current game state");
            match holder.departure.as_ref() {
                Some(session)
                    if session.id == departure_id && matches!(session.phase, DeparturePhase::Protected) => {}
                _ => return Err(departure_blocked().to_string()),
            }
        }
        self.save_to_path_allowing_departure(path, selected_path, || {}, true)?
            .current_game
            .ok_or_else(|| departure_blocked().to_string())
    }

    pub fn begin_application_teardown(
        &self,
        departure_id: u64,
        selected_path: NodePath,
        disposition: ApplicationExitDispositionDto,
    ) -> Result<ApplicationExitOutcomeDto, CurrentGameError> {
        let mut holder = self.holder.lock().expect("current game state");
        match holder.departure.as_mut() {
            Some(session)
                if session.id == departure_id
                    && matches!(session.target, DepartureTarget::ApplicationExit)
                    && matches!(session.phase, DeparturePhase::Protected) =>
            {
                session.phase = DeparturePhase::TearingDown { disposition };
            }
            Some(session) if session.id == departure_id => return Err(departure_blocked()),
            _ => return Err(departure_in_progress()),
        }
        holder.exit_disposition = Some(disposition);
        if holder.human_match.blocks() {
            holder.human_match.seal(app_model::MatchEndDto::Stopped, None);
            self.match_departure.notify_all();
        }
        let current = holder.result_dto(selected_path)?;
        Ok(ApplicationExitOutcomeDto {
            committed: true,
            analysis_stopped: true,
            current: Some(current),
            message: match disposition {
                ApplicationExitDispositionDto::ExplicitDiscard => {
                    "Document discarded. Stopping owned resources.".to_string()
                }
                _ => "Departure committed. Stopping owned resources.".to_string(),
            },
            disposition: Some(disposition),
            teardown: None,
            recovery_persist_error: None,
        })
    }

    pub fn finish_application_teardown(
        &self,
        departure_id: u64,
        selected_path: NodePath,
        attempt: ApplicationTeardownAttemptDto,
        exit_anyway: bool,
    ) -> Result<ApplicationExitOutcomeDto, CurrentGameError> {
        let mut holder = self.holder.lock().expect("current game state");
        let Some(session) = holder.departure.as_ref() else {
            return Err(departure_in_progress());
        };
        if session.id != departure_id {
            return Err(departure_in_progress());
        }
        let DeparturePhase::TearingDown { disposition } = session.phase else {
            return Err(departure_blocked());
        };
        let completed = matches!(attempt, ApplicationTeardownAttemptDto::Completed);
        let disposition =
            if completed && !matches!(disposition, ApplicationExitDispositionDto::ExplicitDiscard) {
                ApplicationExitDispositionDto::CleanCompleted
            } else {
                disposition
            };
        holder.exit_disposition = Some(disposition);
        holder.graceful_exit_completed = completed;
        let current = holder.result_dto(selected_path.clone())?;
        if completed || exit_anyway {
            holder.departure = None;
            if completed {
                holder.edits_blocked = false;
                holder.closed_jobs.clear();
            }
        }
        Ok(ApplicationExitOutcomeDto {
            committed: true,
            analysis_stopped: true,
            current: Some(current),
            message: match (&attempt, disposition) {
                (ApplicationTeardownAttemptDto::TimedOut { outstanding }, _) => format!(
                    "Exit teardown timed out: {}. Retry or exit anyway.",
                    outstanding.join(", ")
                ),
                (_, ApplicationExitDispositionDto::ExplicitDiscard) => {
                    "Application exit discarded the document.".to_string()
                }
                _ => "Application exit completed.".to_string(),
            },
            disposition: Some(disposition),
            teardown: Some(attempt),
            recovery_persist_error: None,
        })
    }

    pub fn graceful_exit_completed(&self) -> bool {
        self.holder
            .lock()
            .expect("current game state")
            .graceful_exit_completed
    }

    #[cfg(test)]
    pub fn application_exit_disposition(&self) -> Option<ApplicationExitDispositionDto> {
        self.holder.lock().expect("current game state").exit_disposition
    }

    pub(super) fn admit_departure(
        &self,
        target: DepartureTarget,
    ) -> Result<DocumentDepartureAdmissionDto, CurrentGameError> {
        let mut holder = self.holder.lock().expect("current game state");
        if let DepartureTarget::Replacement {
            network_lease: Some((_, identity)),
            ..
        } = &target
        {
            if holder.document_identity != *identity {
                return Err(CurrentGameError {
                    kind: CurrentGameErrorKind::DepartureBlocked,
                    message: "The current document changed; preview the provider again.".into(),
                });
            }
        }
        if let DepartureTarget::Replacement {
            external_start,
            network_lease,
            ..
        } = &target
        {
            if let Some(id) = external_start {
                holder.external_sync.validate_start(id.get())?;
                if !matches!(holder.trial_mode, super::trial::TrialMode::Review) {
                    return Err(departure_blocked());
                }
            } else if holder.external_sync.starting_id().is_some()
                || (holder.external_sync.active.is_some() && network_lease.is_none())
            {
                return Err(external_sync::sync_stale());
            }
        }
        if holder.departure.is_some() {
            return Err(departure_in_progress());
        }
        if holder.human_match.blocks() && !matches!(target, DepartureTarget::ApplicationExit) {
            return Err(departure_blocked());
        }
        if matches!(
            holder.trial_mode,
            super::trial::TrialMode::Entering(_)
                | super::trial::TrialMode::Leaving(_)
                | super::trial::TrialMode::EnteringScoring(_)
                | super::trial::TrialMode::LeavingScoring(_)
        ) {
            return Err(departure_blocked());
        }
        holder.graceful_exit_completed = false;
        holder.next_departure_id = holder.next_departure_id.saturating_add(1);
        let departure_id = holder.next_departure_id;
        let dirty = holder.dirty;
        holder.departure = Some(DepartureSession {
            id: departure_id,
            phase: DeparturePhase::AwaitingDecision,
            target,
        });
        if dirty {
            Ok(DocumentDepartureAdmissionDto::NeedsDecision { departure_id })
        } else {
            Ok(DocumentDepartureAdmissionDto::Ready { departure_id })
        }
    }
}

impl CurrentGameHolder {
    fn result_dto(&self, selected_path: NodePath) -> Result<CurrentGameResultDto, CurrentGameError> {
        self.result_at(&selected_path)
    }

    pub(super) fn install_document(
        &mut self,
        document: CurrentSgfDocument,
        native_path: Option<String>,
    ) -> Result<CurrentGameResultDto, CurrentGameError> {
        let selected_path = document.default_selected_path();
        document.snapshot(&selected_path)?;
        self.external_sync.seal_active();
        self.document = Some(document);
        self.generation = self.generation.saturating_add(1);
        self.history.clear();
        self.undo_revisions.clear();
        self.redo_revisions.clear();
        self.next_edit_revision = self.next_edit_revision.saturating_add(1);
        self.edit_revision = self.next_edit_revision;
        self.nonhistory_revision = self.nonhistory_revision.saturating_add(1);
        self.saved_version = self.content_version();
        self.refresh_dirty();
        self.native_path = native_path;
        self.selected_path = selected_path;
        self.document_seq = self.document_seq.saturating_add(1);
        self.document_identity = self.document_identity.saturating_add(1);
        self.snapshot_seq = 1;
        self.edits_blocked = false;
        self.departure = None;
        self.closed_jobs.clear();
        self.analysis_target = None;
        self.trial_mode = super::trial::TrialMode::Review;
        self.current_result()
    }

    pub(super) fn rejects_job(&self, run_id: &str, job_id: &str) -> bool {
        self.edits_blocked
            || self
                .closed_jobs
                .contains(&(run_id.to_string(), job_id.to_string()))
    }

    pub(super) fn ensure_editable(&self) -> Result<(), CurrentGameError> {
        if self.external_sync.active.is_some() || self.external_sync.starting_id().is_some() {
            return Err(CurrentGameError { kind: CurrentGameErrorKind::DepartureBlocked, message: "External sync owns this read-only game. Stop synchronization before editing, Match or Trial.".into() });
        }
        self.ensure_review_access()
    }

    pub(super) fn ensure_review_access(&self) -> Result<(), CurrentGameError> {
        if self.edits_blocked || self.human_match.blocks() {
            Err(departure_blocked())
        } else if self.departure.is_some() {
            Err(departure_in_progress())
        } else if !matches!(self.trial_mode, super::trial::TrialMode::Review) {
            Err(departure_blocked())
        } else {
            Ok(())
        }
    }
}

pub(super) fn departure_in_progress() -> CurrentGameError {
    CurrentGameError {
        kind: CurrentGameErrorKind::DepartureInProgress,
        message: "a document departure is already in progress".to_string(),
    }
}

pub(super) fn departure_blocked() -> CurrentGameError {
    CurrentGameError {
        kind: CurrentGameErrorKind::DepartureBlocked,
        message: "the departing document is closed to new edits and jobs".to_string(),
    }
}
