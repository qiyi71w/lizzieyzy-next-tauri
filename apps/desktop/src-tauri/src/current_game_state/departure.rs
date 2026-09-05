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
        } = session.target
        else {
            holder.departure = Some(session);
            return Err(departure_blocked());
        };
        let current = holder.install_document(candidate, candidate_path)?;
        self.note_recovery(&holder);
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
        holder.edits_blocked = false;
        holder.exit_disposition = None;
        let current = holder.result_dto(selected_path)?;
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
        self.save_to_path_allowing_departure(path, selected_path, || {}, true)
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

    pub fn application_exit_disposition(&self) -> Option<ApplicationExitDispositionDto> {
        self.holder.lock().expect("current game state").exit_disposition
    }

    fn admit_departure(
        &self,
        target: DepartureTarget,
    ) -> Result<DocumentDepartureAdmissionDto, CurrentGameError> {
        let mut holder = self.holder.lock().expect("current game state");
        if holder.departure.is_some() {
            return Err(departure_in_progress());
        }
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
        let document = self.document.as_ref().ok_or_else(no_current_game)?;
        Ok(CurrentGameResultDto {
            tree: document.tree()?,
            snapshot: document.snapshot(&selected_path)?,
            selected_path,
            generation: self.generation,
            dirty: self.dirty,
            native_path: self.native_path.clone(),
        })
    }

    pub(super) fn install_document(
        &mut self,
        document: CurrentSgfDocument,
        native_path: Option<String>,
    ) -> Result<CurrentGameResultDto, CurrentGameError> {
        let selected_path = document.default_selected_path();
        let snapshot = document.snapshot(&selected_path)?;
        let tree = document.tree()?;
        self.document = Some(document);
        self.generation += 1;
        self.dirty = false;
        self.native_path = native_path;
        self.selected_path = selected_path.clone();
        self.document_seq = self.document_seq.saturating_add(1);
        self.snapshot_seq = 1;
        self.edits_blocked = false;
        self.departure = None;
        self.closed_jobs.clear();
        Ok(CurrentGameResultDto {
            tree,
            selected_path,
            snapshot,
            generation: self.generation,
            dirty: self.dirty,
            native_path: self.native_path.clone(),
        })
    }

    pub(super) fn rejects_job(&self, run_id: &str, job_id: &str) -> bool {
        self.edits_blocked
            || self
                .closed_jobs
                .contains(&(run_id.to_string(), job_id.to_string()))
    }

    pub(super) fn ensure_editable(&self) -> Result<(), CurrentGameError> {
        if self.edits_blocked {
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
