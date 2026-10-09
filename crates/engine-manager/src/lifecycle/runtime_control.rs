//! One bounded control transaction on the existing Run/reader. No position authority.
use super::*;
use app_model::RuntimeControlIdentityDto;
use std::sync::mpsc::RecvTimeoutError;
use std::sync::atomic::Ordering;

pub(super) struct Slot {
    identity: RuntimeControlIdentityDto,
}

pub(super) fn require_idle(state: &ManagerState) -> Result<(), EngineFailureDto> {
    if state.runtime_control.is_some() {
        return Err(failure(EngineOperationDto::Job, EngineFailureKind::Occupied,
            "a runtime control transaction owns the foreground Run".into(),
            current_run_id(&state.phase).as_deref(), None, None));
    }
    Ok(())
}

pub(super) fn retire(state: &mut ManagerState, run: &str) {
    if state.runtime_control.as_ref().is_some_and(|slot| slot.identity.run_id == run) {
        let slot = state.runtime_control.take().unwrap();
        state.gtp_dispatch.retire(run, &slot.identity.request_id);
    }
}

/// Dropping the lease releases only control admission, never user Pause or durable intent.
/// Subsequent read-only paired controls use this same lease and command method.
pub(super) struct Transaction {
    manager: ForegroundEngineManager,
    identity: RuntimeControlIdentityDto,
    deadline: Instant,
}

impl ForegroundEngineManager {
    pub(super) fn begin_runtime_control(&self, identity: &RuntimeControlIdentityDto) -> Result<Transaction, String> {
        if Uuid::parse_str(&identity.request_id).is_err() {
            return Err("runtime control request requires a UUID".into());
        }
        if self.inner.runtime_control_writer.load(Ordering::Acquire) {
            return Err("previous runtime control write is still draining".into());
        }
        {
            let mut state = self.lock();
            require_idle(&state).map_err(|e| e.message)?;
            match_reservation::require_unreserved(&state).map_err(|e| e.message)?;
            game_move::require_idle_move(&state).map_err(|e| e.message)?;
            let Phase::Ready(run) = &state.phase else { return Err("runtime control requires a Ready Run".into()); };
            if run.run_id != identity.run_id || run.adapter_kind != EngineBackend::KataGoGtp
                || run.qualified_resource.as_ref().is_none_or(|resource| resource.profile_revision != identity.profile_revision)
                || owned_live(&state, &identity.run_id).is_none() {
                return Err("runtime control identity expired".into());
            }
            if state.continuous_departing || state.finite_admission_pending
                || state.jobs.iter().any(|job| !job.terminal && (job.mode != AnalysisJobModeDto::Continuous || job.lane != AnalysisJobLane::SelectedNode))
                || state.analysis_task.as_ref().is_some_and(|task| matches!(task.state,
                    AnalysisTaskStateDto::Queued | AnalysisTaskStateDto::Searching | AnalysisTaskStateDto::Pausing | AnalysisTaskStateDto::Paused)) {
                return Err("runtime control is busy: finite work, task or document departure owns admission".into());
            }
            state.runtime_control = Some(Slot { identity: identity.clone() });
            if let Some(job) = current_selected_job(&state).filter(|job| !job.state.is_limited()) {
                if let Err(error) = self.request_job_stop_locked(&mut state, &job.run_id, &job.job_id, JobDisposition::Superseded) {
                    state.runtime_control = None;
                    return Err(error.message);
                }
            }
        }
        Ok(Transaction { manager: self.clone(), identity: identity.clone(),
            // Five seconds for the existing stop barrier plus five for this bounded control round.
            deadline: Instant::now() + Duration::from_secs(10) })
    }
}

impl Transaction {
    pub(super) fn current(&self, state: &ManagerState) -> Result<(), String> {
        if !state.runtime_control.as_ref().is_some_and(|slot| slot.identity == self.identity)
            || !matches!(&state.phase, Phase::Ready(run) if run.run_id == self.identity.run_id)
            || state.continuous_departing || state.match_reservation.is_some() {
            return Err("runtime control expired before confirmation".into());
        }
        if Instant::now() >= self.deadline { return Err("runtime control timed out".into()); }
        Ok(())
    }

    pub(super) fn drain(&self) -> Result<(), String> {
        loop {
            let state = self.manager.lock();
            self.current(&state)?;
            if !state.jobs.iter().any(|job| job.run_id == self.identity.run_id && !job.terminal)
                && state.game_move.is_none() { return Ok(()); }
            drop(state);
            thread::sleep(Duration::from_millis(5));
        }
    }

    /// Register before writing and require both successful write and complete own-ID reply.
    pub(super) fn command(&self, body: &str) -> Result<String, String> {
        let (id, response) = self.send(body)?;
        self.receive(id, response)
    }

    /// Both commands are issued before awaiting either reply; the sole dispatcher
    /// retains own-ID responses in either arrival order for this one lease.
    pub(super) fn parameter_pair(&self) -> Result<(String, String), String> {
        let (pda_id, pda) = self.send("kata-get-param playoutDoublingAdvantage")?;
        let (wrn_id, wrn) = self.send("kata-get-param analysisWideRootNoise")?;
        Ok((self.receive(pda_id, pda)?, self.receive(wrn_id, wrn)?))
    }

    fn send(&self, body: &str) -> Result<(u32, mpsc::Receiver<String>), String> {
        let (id, response, stdin) = {
            let mut state = self.manager.lock();
            self.current(&state)?;
            state.gtp_command_seq = state.gtp_command_seq.checked_add(1).ok_or("GTP command IDs exhausted")?;
            let id = state.gtp_command_seq;
            let response = state.gtp_dispatch.register(&self.identity.run_id, id, &self.identity.request_id, false)?;
            let stdin = engine_stdin(&state, &self.identity.run_id).ok_or("runtime control stdin retired")?;
            (id, response, stdin)
        };
        let (sender, written) = mpsc::sync_channel(1);
        let manager = self.manager.clone();
        let identity = self.identity.clone();
        let payload = format!("{id} {body}");
        let writer_busy = Arc::clone(&self.manager.inner.runtime_control_writer);
        if writer_busy.swap(true, Ordering::AcqRel) {
            return Err("previous runtime control write is still draining".into());
        }
        let deadline = self.deadline;
        thread::spawn(move || {
            let mut guard = stdin.lock().expect("engine stdin lock");
            // Check again after waiting for stdin. A retired request cannot write into a newer Run.
            let state = manager.lock();
            let valid = state.runtime_control.as_ref().is_some_and(|slot| slot.identity == identity)
                && matches!(&state.phase, Phase::Ready(run) if run.run_id == identity.run_id)
                && Instant::now() < deadline && !state.continuous_departing;
            drop(state);
            let result = if !valid { Err("runtime control write retired".to_owned()) }
                else if let Some(writer) = guard.as_mut() { write_jsonl(writer, &payload).map_err(|_| "runtime control write failed".to_owned()) }
                else { Err("runtime control stdin closed".to_owned()) };
            writer_busy.store(false, Ordering::Release);
            let _ = sender.send(result);
        });
        written.recv_timeout(self.deadline.saturating_duration_since(Instant::now()))
            .map_err(|_| "runtime control write timed out")??;
        Ok((id, response))
    }

    fn receive(&self, id: u32, response: mpsc::Receiver<String>) -> Result<String, String> {
        let mut decoder = ResponseDecoder::new(id);
        let result = (|| loop {
            self.current(&self.manager.lock())?;
            match response.recv_timeout(Duration::from_millis(20)) {
                Ok(line) => if let Some(reply) = decoder.push(&line)? {
                    self.current(&self.manager.lock())?;
                    if !reply.success { return Err("KataGo rejected the runtime parameter request".into()); }
                    return Ok(reply.body);
                },
                Err(RecvTimeoutError::Timeout) => continue,
                Err(RecvTimeoutError::Disconnected) => return Err("runtime control reader retired".into()),
            }
        })();
        // A failed first half can leave the other reply in flight. Retire both
        // registrations before dropping either receiver, not later in Drop.
        if result.is_err() {
            self.manager.lock().gtp_dispatch.retire(&self.identity.run_id, &self.identity.request_id);
        }
        result
    }
}

impl Drop for Transaction {
    fn drop(&mut self) {
        let mut state = self.manager.lock();
        if state.runtime_control.as_ref().is_some_and(|slot| slot.identity == self.identity) {
            state.gtp_dispatch.retire(&self.identity.run_id, &self.identity.request_id);
            state.runtime_control = None;
        }
        drop(state);
        self.manager.reconcile_continuous();
    }
}
