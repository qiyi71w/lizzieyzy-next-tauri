use super::*;
use app_model::{AnalysisJobStartedDto, DocumentDepartureAdmissionDto, DocumentDepartureOutcomeDto};

pub(super) struct DepartureSession {
    pub id: u64,
    pub phase: DeparturePhase,
    pub candidate: CurrentSgfDocument,
    pub candidate_path: Option<String>,
}

pub(super) enum DeparturePhase {
    AwaitingDecision,
    Protected,
}

impl CurrentGameState {
    pub fn prepare_replacement(
        &self,
        sgf_text: &str,
        native_path: Option<String>,
    ) -> Result<DocumentDepartureAdmissionDto, CurrentGameError> {
        let candidate = CurrentSgfDocument::open(sgf_text)?;
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
            candidate,
            candidate_path: native_path,
        });
        if dirty {
            Ok(DocumentDepartureAdmissionDto::NeedsDecision { departure_id })
        } else {
            Ok(DocumentDepartureAdmissionDto::Ready { departure_id })
        }
    }

    pub fn cancel_replacement(
        &self,
        departure_id: u64,
    ) -> Result<DocumentDepartureOutcomeDto, CurrentGameError> {
        let mut holder = self.holder.lock().expect("current game state");
        match holder.departure.as_ref() {
            Some(session) if session.id == departure_id => {
                if matches!(session.phase, DeparturePhase::Protected) {
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
        let current = holder.install_document(session.candidate, session.candidate_path)?;
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
