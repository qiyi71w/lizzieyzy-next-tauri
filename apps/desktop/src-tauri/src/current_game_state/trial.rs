use super::*;
use app_model::{
    AnalysisJobLaneDto, AnalysisJobStateDto, AnalysisTaskStateDto, EngineFailureDto, EngineFailureKind,
};
use engine_manager::ForegroundEngineManager;
use std::time::{Duration, Instant};

#[derive(Default)]
pub(super) enum TrialMode {
    #[default]
    Review,
    Entering(TrialSession),
    Active(TrialSession),
    Leaving(TrialSession),
}

pub(super) struct TrialSession {
    id: u64,
    revision: u64,
    entry_path: NodePath,
    line: TrialLine,
    finite_job: Option<(String, String)>,
}

impl TrialSession {
    fn result(&self) -> Result<TrialSessionDto, CurrentGameError> {
        Ok(TrialSessionDto {
            session_id: self.id,
            revision: self.revision,
            entry_path: self.entry_path.clone(),
            tree: self.line.tree()?,
            selected_path: self.line.selected_path(),
            snapshot: self.line.snapshot()?,
            can_undo: self.line.can_undo(),
        })
    }
}

fn mode_error() -> CurrentGameError {
    CurrentGameError {
        kind: CurrentGameErrorKind::DepartureBlocked,
        message: "Try-play transition or session does not match the current mode.".into(),
    }
}

impl CurrentGameState {
    pub fn enter_trial(&self) -> Result<TrialSessionDto, String> {
        let manager = self.analysis_manager.get();
        let (id, jobs) = {
            let mut holder = self.holder.lock().expect("current game state");
            holder.ensure_editable().map_err(|error| error.message)?;
            let document = holder
                .document
                .as_ref()
                .ok_or_else(|| no_current_game().message)?;
            let line = TrialLine::new(document, &holder.selected_path).map_err(|error| error.message)?;
            holder.next_trial_id = holder.next_trial_id.saturating_add(1);
            let id = holder.next_trial_id;
            holder.next_trial_revision = holder.next_trial_revision.saturating_add(1).max(1 << 48);
            let session = TrialSession {
                id,
                revision: holder.next_trial_revision,
                entry_path: holder.selected_path.clone(),
                line,
                finite_job: None,
            };
            holder.trial_mode = TrialMode::Entering(session);
            holder.analysis_target = None;
            let jobs = manager
                .map(|manager| {
                    let snapshot = manager.snapshot();
                    let jobs = crate::document_departure::jobs_from_snapshot(&snapshot);
                    for job in &jobs {
                        holder
                            .closed_jobs
                            .insert((job.run_id.clone(), job.job_id.clone()));
                    }
                    manager.clear_continuous_position();
                    if let Some(task) = manager.analysis_task_snapshot().filter(|task| {
                        matches!(
                            task.state,
                            AnalysisTaskStateDto::Queued | AnalysisTaskStateDto::Searching
                        )
                    }) {
                        manager
                            .pause_analysis_task(&task.run_id, &task.task_id)
                            .map_err(|error| error.message)?;
                    }
                    Ok::<_, String>(jobs)
                })
                .transpose();
            (id, jobs)
        };
        let result = jobs.and_then(|jobs| {
            if let Some(manager) = manager {
                wait_jobs(manager, &jobs.unwrap_or_default())?;
            }
            Ok(())
        });
        let mut holder = self.holder.lock().expect("current game state");
        if result.is_err() {
            holder.trial_mode = TrialMode::Review;
            holder.analysis_target = None;
            self.follow_continuous_position(&mut holder);
            return Err(result.unwrap_err());
        }
        let TrialMode::Entering(session) = std::mem::take(&mut holder.trial_mode) else {
            return Err(mode_error().message);
        };
        if session.id != id {
            return Err(mode_error().message);
        }
        let result = session.result().map_err(|error| error.message)?;
        holder.trial_mode = TrialMode::Active(session);
        self.follow_trial_position(&mut holder);
        Ok(result)
    }

    pub fn exit_trial(&self, id: u64) -> Result<CurrentGameResultDto, String> {
        let manager = self.analysis_manager.get();
        let jobs = {
            let mut holder = self.holder.lock().expect("current game state");
            if holder.departure.is_some() {
                return Err(departure::departure_in_progress().message);
            }
            let mode = std::mem::take(&mut holder.trial_mode);
            let TrialMode::Active(session) = mode else {
                holder.trial_mode = mode;
                return Err(mode_error().message);
            };
            if session.id != id {
                holder.trial_mode = TrialMode::Active(session);
                return Err(mode_error().message);
            }
            holder.trial_mode = TrialMode::Leaving(session);
            let jobs = manager
                .map(|manager| {
                    let jobs = crate::document_departure::jobs_from_snapshot(&manager.snapshot())
                        .into_iter()
                        .filter(|job| job.lane == AnalysisJobLaneDto::SelectedNode)
                        .collect::<Vec<_>>();
                    for job in &jobs {
                        holder
                            .closed_jobs
                            .insert((job.run_id.clone(), job.job_id.clone()));
                    }
                    manager.clear_continuous_position();
                    jobs
                })
                .unwrap_or_default();
            jobs
        };
        let result = manager.map(|manager| wait_jobs(manager, &jobs)).unwrap_or(Ok(()));
        let mut holder = self.holder.lock().expect("current game state");
        let mode = std::mem::take(&mut holder.trial_mode);
        if let Err(error) = result {
            holder.trial_mode = match mode {
                TrialMode::Leaving(session) => TrialMode::Active(session),
                other => other,
            };
            return Err(error);
        }
        if !matches!(mode, TrialMode::Leaving(_)) {
            return Err(mode_error().message);
        }
        holder.analysis_target = None;
        self.follow_continuous_position(&mut holder);
        holder.current_result().map_err(|error| error.message)
    }

    pub fn trial_snapshot(&self, id: u64) -> Result<TrialSessionDto, CurrentGameError> {
        let holder = self.holder.lock().expect("current game state");
        let TrialMode::Active(session) = &holder.trial_mode else {
            return Err(mode_error());
        };
        if session.id != id {
            return Err(mode_error());
        }
        session.result()
    }

    pub fn trial_select(
        &self,
        id: u64,
        revision: u64,
        path: NodePath,
    ) -> Result<TrialSessionDto, CurrentGameError> {
        self.mutate_trial(id, revision, |line| {
            let before = line.selected_path();
            line.select(&path)?;
            Ok(before != line.selected_path())
        })
    }

    pub fn trial_play(
        &self,
        id: u64,
        revision: u64,
        vertex: MoveVertex,
    ) -> Result<TrialSessionDto, CurrentGameError> {
        self.mutate_trial(id, revision, |line| {
            let before = line.selected_path();
            let edited = line.play(vertex)?;
            Ok(edited || before != line.selected_path())
        })
    }

    pub fn trial_undo(&self, id: u64, revision: u64) -> Result<TrialSessionDto, CurrentGameError> {
        self.mutate_trial(id, revision, TrialLine::undo)
    }

    fn mutate_trial(
        &self,
        id: u64,
        revision: u64,
        change: impl FnOnce(&mut TrialLine) -> Result<bool, CurrentGameError>,
    ) -> Result<TrialSessionDto, CurrentGameError> {
        let mut holder = self.holder.lock().expect("current game state");
        if holder.departure.is_some() || holder.edits_blocked {
            return Err(departure::departure_blocked());
        }
        let changed = {
            let TrialMode::Active(session) = &mut holder.trial_mode else {
                return Err(mode_error());
            };
            if session.id != id || session.revision != revision {
                return Err(mode_error());
            }
            change(&mut session.line)?
        };
        if changed {
            holder.next_trial_revision = holder.next_trial_revision.saturating_add(1);
            let next = holder.next_trial_revision;
            if let TrialMode::Active(session) = &mut holder.trial_mode {
                session.revision = next;
                session.finite_job = None;
            }
            self.follow_trial_position(&mut holder);
        }
        let TrialMode::Active(session) = &holder.trial_mode else {
            unreachable!()
        };
        session.result()
    }

    pub fn trial_start_finite(
        &self,
        manager: &ForegroundEngineManager,
        id: u64,
        revision: u64,
        run_id: String,
        visits: u32,
    ) -> Result<AnalysisJobStartedDto, EngineFailureDto> {
        let mut holder = self.holder.lock().expect("current game state");
        let invalid = |message| crate::job_failure(&run_id, EngineFailureKind::InvalidState, message);
        if holder.departure.is_some() || holder.edits_blocked {
            return Err(invalid("Document departure is pending.".into()));
        }
        let TrialMode::Active(session) = &holder.trial_mode else {
            return Err(invalid(mode_error().message));
        };
        if session.id != id || session.revision != revision {
            return Err(invalid(mode_error().message));
        }
        let snapshot = session
            .line
            .raw_snapshot()
            .map_err(|error| invalid(error.message))?;
        let mut request = crate::continuous_analysis::position_request(
            session.revision,
            snapshot.path.clone(),
            snapshot,
            session.line.board_width(),
            session.line.board_height(),
            session.line.komi(),
            session.line.rules(),
        )
        .map_err(|error| invalid(error.to_string()))?;
        request.run_id = run_id;
        request.mode = AnalysisJobModeDto::Finite;
        request.query.max_visits = Some(visits);
        let started = manager.start_selected_node_job(request)?;
        if let TrialMode::Active(session) = &mut holder.trial_mode {
            session.finite_job = Some((started.run_id.clone(), started.job_id.clone()));
        }
        Ok(started)
    }

    fn follow_trial_position(&self, holder: &mut CurrentGameHolder) {
        let Some(manager) = self.analysis_manager.get() else {
            return;
        };
        let TrialMode::Active(session) = &holder.trial_mode else {
            return;
        };
        let Ok(snapshot) = session.line.raw_snapshot() else {
            return;
        };
        if let Ok(request) = crate::continuous_analysis::position_request(
            session.revision,
            snapshot.path.clone(),
            snapshot,
            session.line.board_width(),
            session.line.board_height(),
            session.line.komi(),
            session.line.rules(),
        ) {
            manager.follow_continuous_position(request)
        }
    }

    pub fn attach_trial_from_job_event(&self, event: &AnalysisJobEventDto) -> Option<TrialSessionDto> {
        if event.lane != AnalysisJobLaneDto::SelectedNode || !admits_analysis_attachment(event) {
            return None;
        }
        let frame = event.frame.as_ref()?;
        let mut holder = self.holder.lock().expect("current game state");
        if holder.departure.is_some()
            || holder.edits_blocked
            || holder.rejects_job(&event.run_id, &event.job_id)
        {
            return None;
        }
        let TrialMode::Active(session) = &mut holder.trial_mode else {
            return None;
        };
        if event.generation != session.revision || event.node_path != session.line.selected_path() {
            return None;
        }
        if let Some(manager) = self.analysis_manager.get() {
            let live = manager.snapshot().selected_node_job.is_some_and(|job| {
                job.run_id == event.run_id
                    && job.job_id == event.job_id
                    && job.state != AnalysisJobStateDto::Stopping
            });
            let finite_final = event.mode == AnalysisJobModeDto::Finite
                && session
                    .finite_job
                    .as_ref()
                    .is_some_and(|(run, job)| run == &event.run_id && job == &event.job_id);
            if event.mode == AnalysisJobModeDto::Finite {
                if !finite_final {
                    return None;
                }
            } else if !live {
                return None;
            }
        }
        if event.mode == AnalysisJobModeDto::Finite {
            session.finite_job = None;
        }
        session
            .line
            .attach_analysis(&event.node_path, &SgfAnalysisPayload::from_frame(frame, "KataGo"))
            .ok()?;
        let mut result = session.result().ok()?;
        if let Some(analysis) = result.snapshot.primary_analysis.as_mut() {
            analysis.policy = frame.policy.clone()
        }
        Some(result)
    }
}

fn wait_jobs(manager: &ForegroundEngineManager, jobs: &[AnalysisJobStartedDto]) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(5);
    for job in jobs {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err("Try-play query cleanup timed out.".into());
        }
        manager
            .wait_for_job_cancellation(&job.run_id, &job.job_id, remaining)
            .map_err(|error| error.message)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use app_model::PointDto;

    #[test]
    fn trial_preserves_document_identity_save_source_and_reentry() {
        let state = CurrentGameState::default();
        let opened = state
            .replace(
                "(;SZ[5]C[original];B[aa];W[bb]C[entry];B[cc])",
                Some("/tmp/source.sgf".into()),
            )
            .unwrap();
        let entry = NodePath { indices: vec![0, 0] };
        let original = state.select_path(entry.clone(), opened.generation).unwrap();
        let before = state.inspect();
        let source = state.serialize().unwrap();
        let trial = state.enter_trial().unwrap();
        assert!(state.enter_trial().is_err());
        assert_eq!(trial.entry_path, entry);
        assert_eq!(trial.snapshot.position.move_number, 0);
        assert_eq!(
            state.play(entry.clone(), MoveVertex::Pass).unwrap_err().kind,
            CurrentGameErrorKind::DepartureBlocked
        );
        let illegal = state
            .trial_play(
                trial.session_id,
                trial.revision,
                MoveVertex::Point(PointDto { x: 0, y: 0 }),
            )
            .unwrap_err();
        assert_eq!(state.trial_undo(trial.session_id, trial.revision).unwrap(), trial);
        assert_eq!(illegal.kind, CurrentGameErrorKind::OccupiedPoint);
        assert_eq!(state.trial_snapshot(trial.session_id).unwrap(), trial);
        let passed = state
            .trial_play(trial.session_id, trial.revision, MoveVertex::Pass)
            .unwrap();
        assert_eq!(passed.snapshot.position.move_number, 1);
        let played = state
            .trial_play(
                passed.session_id,
                passed.revision,
                MoveVertex::Point(PointDto { x: 2, y: 2 }),
            )
            .unwrap();
        assert_eq!(played.snapshot.position.move_number, 2);
        assert_eq!(
            state
                .trial_undo(played.session_id, passed.revision)
                .unwrap_err()
                .kind,
            CurrentGameErrorKind::DepartureBlocked
        );
        let undone = state.trial_undo(played.session_id, played.revision).unwrap();
        assert_eq!(undone.snapshot.position.move_number, 1);
        let root = state
            .trial_select(undone.session_id, undone.revision, NodePath::default())
            .unwrap();
        assert_eq!(root.snapshot.position.move_number, 0);
        assert_eq!(state.serialize().unwrap(), source);
        assert_eq!(state.inspect(), before);
        let invalid_destination = std::env::temp_dir().join(format!("trial-save-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&invalid_destination).unwrap();
        assert!(state
            .save_to_path(invalid_destination.to_string_lossy().into_owned(), entry.clone())
            .is_err());
        std::fs::remove_dir_all(&invalid_destination).unwrap();
        assert_eq!(state.trial_snapshot(root.session_id).unwrap(), root);
        assert_eq!(state.serialize().unwrap(), source);
        assert_eq!(state.inspect(), before);

        let admission = state.prepare_replacement("(;SZ[9])", None).unwrap();
        let departure_id = match admission {
            app_model::DocumentDepartureAdmissionDto::Ready { departure_id }
            | app_model::DocumentDepartureAdmissionDto::NeedsDecision { departure_id } => departure_id,
        };
        state.cancel_replacement(departure_id).unwrap();
        assert_eq!(state.trial_snapshot(root.session_id).unwrap(), root);
        assert!(state.exit_trial(root.session_id + 1).is_err());
        let returned = state.exit_trial(root.session_id).unwrap();
        assert_eq!(returned, original);
        assert_eq!(state.serialize().unwrap(), source);
        assert_eq!(state.inspect(), before);
        assert!(state.trial_snapshot(root.session_id).is_err());
        let reentered = state.enter_trial().unwrap();
        assert_ne!(reentered.session_id, root.session_id);
        assert!(reentered.revision > root.revision);
    }

    #[test]
    fn undo_from_edited_child_parent_revisions_the_trial_tree() {
        let state = CurrentGameState::default();
        state.replace("(;SZ[5])", None).unwrap();
        let entry = state.enter_trial().unwrap();
        let first = state
            .trial_play(entry.session_id, entry.revision, MoveVertex::Pass)
            .unwrap();
        let second = state
            .trial_play(first.session_id, first.revision, MoveVertex::Pass)
            .unwrap();
        let parent = state
            .trial_select(second.session_id, second.revision, NodePath { indices: vec![0] })
            .unwrap();
        assert!(parent.can_undo);
        let undone = state.trial_undo(parent.session_id, parent.revision).unwrap();
        assert_eq!(undone.selected_path.indices, vec![0]);
        assert!(undone.can_undo);
        assert!(undone.tree.children[0].children.is_empty());
        assert!(undone.revision > parent.revision);
        assert_eq!(
            state
                .trial_undo(parent.session_id, parent.revision)
                .unwrap_err()
                .kind,
            CurrentGameErrorKind::DepartureBlocked
        );
        assert_eq!(state.trial_snapshot(parent.session_id).unwrap(), undone);
    }
}
