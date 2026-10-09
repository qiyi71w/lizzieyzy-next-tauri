use super::*;
use app_model::{EnginePreloadDto, EnginePreloadPhaseDto};

pub(super) struct PreloadSlot {
    pub dto: EnginePreloadDto,
    pub live: Option<LiveEngine>,
    resources: Option<Arc<ResourceSnapshot>>,
    worker_running: bool,
}

pub(super) fn preparing(state: &ManagerState, run_id: &str) -> bool {
    state
        .preloads
        .iter()
        .any(|slot| slot.dto.run.run_id == run_id && slot.dto.phase == EnginePreloadPhaseDto::Preparing)
}

fn busy(state: &ManagerState) -> bool {
    state.preload_shutdown
        || state.continuous_departing
        || state.match_reservation.is_some()
        || state.game_move.is_some()
        || state.finite_admission_pending
        || state.jobs.iter().any(|job| !job.terminal)
        || !matches!(state.phase, Phase::NoEngine { .. } | Phase::Ready(_))
}

fn invalid(profile_id: &str, message: &str) -> EngineFailureDto {
    failure(
        EngineOperationDto::Start,
        EngineFailureKind::InvalidState,
        message.into(),
        None,
        Some(profile_id),
        None,
    )
}

fn retire(state: &mut ManagerState, profile_id: Option<&str>) {
    for slot in &mut state.preloads {
        if profile_id.is_some_and(|id| slot.dto.run.profile_id != id) {
            continue;
        }
        if matches!(
            slot.dto.phase,
            EnginePreloadPhaseDto::Preparing | EnginePreloadPhaseDto::Ready
        ) {
            slot.dto.phase = EnginePreloadPhaseDto::Cancelled;
            slot.resources = None;
        }
        if let Some(live) = slot.live.take() {
            state
                .retiring
                .push((slot.dto.run.clone(), Arc::new(Mutex::new(live))));
        }
    }
}

impl ForegroundEngineManager {
    pub fn preload_snapshot(&self) -> Vec<EnginePreloadDto> {
        self.lock().preloads.iter().map(|slot| slot.dto.clone()).collect()
    }

    pub fn prepare_preload(&self, profile_id: &str) -> Result<(), EngineFailureDto> {
        let saved = self
            .inner
            .catalog
            .preload_profiles()
            .into_iter()
            .find(|saved| saved.profile_id == profile_id)
            .ok_or_else(|| invalid(profile_id, "Save this profile's background preload opt-in first"))?;
        self.prepare_saved_preload(saved, None)
    }

    fn prepare_saved_preload(
        &self,
        saved: SavedEngineProfile,
        startup_operation: Option<u64>,
    ) -> Result<(), EngineFailureDto> {
        let profile_id = saved.profile_id.as_str();
        let (operation, run) = {
            let mut state = self.lock();
            if let Some(operation) = startup_operation {
                if state.operation != operation
                    || state.startup_preload_operation != Some(operation)
                    || !self.inner.catalog.preload_profiles().iter().any(|current| {
                        current.profile_id == saved.profile_id && current.profile == saved.profile
                    })
                {
                    return Err(invalid(profile_id, "Startup preload authorization was retired"));
                }
            }
            if busy(&state)
                || current_run_id(&state.phase).is_some_and(|id| {
                    state.live.as_ref().is_some_and(|live| live.run_id == id)
                        && matches!(&state.phase, Phase::Ready(run) if run.profile_id == profile_id)
                })
            {
                return Err(invalid(
                    profile_id,
                    "Foreground analysis, lifecycle or Match work has priority",
                ));
            }
            if state.preloads.iter().any(|slot| {
                slot.dto.run.profile_id == profile_id
                    && (slot.worker_running
                        || matches!(
                            slot.dto.phase,
                            EnginePreloadPhaseDto::Preparing | EnginePreloadPhaseDto::Ready
                        ))
            }) {
                return Err(invalid(
                    profile_id,
                    "This profile already has a background preparation",
                ));
            }
            state
                .preloads
                .retain(|slot| slot.dto.run.profile_id != profile_id);
            let run = starting_run(&saved);
            state.preloads.push(PreloadSlot {
                dto: EnginePreloadDto {
                    run: run.clone(),
                    phase: EnginePreloadPhaseDto::Preparing,
                    failure: None,
                },
                live: None,
                resources: None,
                worker_running: true,
            });
            (state.operation, run)
        };
        let inner = self.inner.clone();
        thread::spawn(move || {
            if let Err(mut error) = inner.start_resident(operation, &run, false, true) {
                identify_failed_executable(&run, &mut error);
                inner.fail_preload(&run.run_id, error);
            } else if preparing(&inner.lock(), &run.run_id) {
                inner.fail_preload(
                    &run.run_id,
                    invalid(&run.profile_id, "Foreground authority superseded background preparation"),
                );
            }
            if let Some(slot) = inner
                .lock()
                .preloads
                .iter_mut()
                .find(|slot| slot.dto.run.run_id == run.run_id)
            {
                slot.worker_running = false;
            }
        });
        Ok(())
    }

    /// Consumes the saved startup authorization once, after the original autoload finishes.
    pub fn apply_preloads(&self) {
        self.schedule_startup_preloads();
    }

    pub fn schedule_startup_preloads(&self) {
        let (operation, profiles) = {
            let mut state = self.lock();
            let Some(profiles) = state.startup_preloads.take() else {
                return;
            };
            if state.preload_shutdown || profiles.is_empty() {
                return;
            }
            let operation = state.operation;
            state.startup_preload_operation = Some(operation);
            (operation, profiles)
        };
        let weak = Arc::downgrade(&self.inner);
        thread::spawn(move || loop {
            let Some(inner) = weak.upgrade() else {
                return;
            };
            let manager = ForegroundEngineManager { inner };
            let mut state = manager.lock();
            if state.operation != operation
                || state.startup_preload_operation != Some(operation)
                || state.preload_shutdown
            {
                return;
            }
            if matches!(state.phase, Phase::Starting(_)) {
                drop(state);
                drop(manager);
                thread::sleep(Duration::from_millis(20));
                continue;
            }
            if busy(&state) {
                state.startup_preload_operation = None;
                return;
            }
            drop(state);
            for saved in profiles {
                let _ = manager.prepare_saved_preload(saved, Some(operation));
            }
            manager.lock().startup_preload_operation = None;
            return;
        });
    }

    fn wait_preload_workers(&self, profile_id: Option<&str>) -> Result<(), EngineFailureDto> {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            let pending =
                self.lock().preloads.iter().any(|slot| {
                    slot.worker_running && profile_id.is_none_or(|id| slot.dto.run.profile_id == id)
                });
            if !pending {
                return self.reap_retiring_runs();
            }
            if Instant::now() >= deadline {
                return Err(invalid(
                    profile_id.unwrap_or(""),
                    "Background cleanup is still pending; retry cancellation before exit",
                ));
            }
            thread::sleep(Duration::from_millis(10));
        }
    }

    pub fn cancel_preload(&self, profile_id: &str) -> Result<(), EngineFailureDto> {
        retire(&mut self.lock(), Some(profile_id));
        let result = self.wait_preload_workers(Some(profile_id));
        if let Some(slot) = self
            .lock()
            .preloads
            .iter_mut()
            .find(|slot| slot.dto.run.profile_id == profile_id)
        {
            if let Err(error) = &result {
                slot.dto.failure = Some(error.clone());
            }
        }
        result
    }

    pub fn invalidate_preload(&self, profile_id: &str) {
        retire(&mut self.lock(), Some(profile_id));
        let manager = self.clone();
        thread::spawn(move || {
            let _ = manager.reap_retiring_runs();
        });
    }

    pub(super) fn cancel_all_preloads(&self, shutdown: bool) -> Result<(), EngineFailureDto> {
        {
            let mut state = self.lock();
            state.preload_shutdown |= shutdown;
            state.startup_preloads = None;
            state.startup_preload_operation = None;
            retire(&mut state, None);
        }
        self.wait_preload_workers(None)
    }

    pub(super) fn yield_preloads_locked(&self, state: &mut ManagerState) {
        state.startup_preload_operation = None;
        if !state
            .preloads
            .iter()
            .any(|slot| slot.live.is_some() || slot.worker_running)
        {
            return;
        }
        retire(state, None);
        let manager = self.clone();
        thread::spawn(move || {
            let _ = manager.reap_retiring_runs();
        });
    }

    pub(super) fn switch_preload(&self, profile_id: &str) -> Result<bool, EngineFailureDto> {
        let (run, resources, operation, primary) = {
            let state = self.lock();
            let Some(slot) = state
                .preloads
                .iter()
                .find(|slot| slot.dto.run.profile_id == profile_id)
            else {
                return Ok(false);
            };
            if matches!(
                slot.dto.phase,
                EnginePreloadPhaseDto::Cancelled | EnginePreloadPhaseDto::Failed
            ) {
                return Ok(false);
            }
            if slot.dto.phase != EnginePreloadPhaseDto::Ready || busy(&state) {
                return Err(invalid(
                    profile_id,
                    "Prepared engine is not ready or foreground work owns the manager",
                ));
            }
            let Phase::Ready(primary) = &state.phase else {
                return Err(invalid(
                    profile_id,
                    "Promotion requires the captured Ready foreground engine",
                ));
            };
            (
                slot.dto.run.clone(),
                slot.resources.as_ref().expect("ready resource").clone(),
                state.operation,
                primary.clone(),
            )
        };
        let validate = || {
            if !self
                .inner
                .catalog
                .preload_profiles()
                .iter()
                .any(|saved| saved.profile_id == profile_id && saved.profile == run.profile_snapshot)
            {
                return Err(invalid(
                    profile_id,
                    "Saved profile or preload opt-in changed; prepare again explicitly",
                ));
            }
            resources
                .revalidate(Instant::now() + Duration::from_secs(30), &|| {
                    let state = self.lock();
                    state.operation != operation
                        || busy(&state)
                        || !matches!(&state.phase, Phase::Ready(current) if current.run_id == primary.run_id)
                        || !state.preloads.iter().any(|slot| {
                            slot.dto.run.run_id == run.run_id
                                && slot.dto.phase == EnginePreloadPhaseDto::Ready
                        })
                })
                .map_err(|(kind, message)| {
                    failure(
                        EngineOperationDto::Switch,
                        kind,
                        message,
                        Some(&run.run_id),
                        Some(profile_id),
                        None,
                    )
                })
        };
        if let Err(error) = validate() {
            self.inner.fail_preload(&run.run_id, error.clone());
            return Err(error);
        }
        let next_operation = {
            let mut state = self.lock();
            if state.operation != operation
                || busy(&state)
                || !matches!(&state.phase, Phase::Ready(current) if current.run_id == primary.run_id)
            {
                return Err(invalid(
                    profile_id,
                    "Foreground authority changed during preload validation",
                ));
            }
            let Some(index) = state.preloads.iter().position(|slot| {
                slot.dto.run.run_id == run.run_id && slot.dto.phase == EnginePreloadPhaseDto::Ready
            }) else {
                return Err(invalid(profile_id, "Preparation was cancelled during validation"));
            };
            if !self
                .inner
                .catalog
                .preload_profiles()
                .iter()
                .any(|saved| saved.profile_id == profile_id && saved.profile == run.profile_snapshot)
            {
                return Err(invalid(
                    profile_id,
                    "Saved preparation identity changed before promotion",
                ));
            }
            if !state.preloads[index]
                .live
                .as_mut()
                .is_some_and(|live| matches!(live.child.try_wait(), Ok(None)))
            {
                drop(state);
                let error = invalid(profile_id, "Prepared engine exited before promotion");
                self.inner.fail_preload(&run.run_id, error.clone());
                return Err(error);
            }
            state.candidate = state.preloads.remove(index).live;
            self.yield_preloads_locked(&mut state);
            state.operation += 1;
            state.operation_kind = EngineOperationDto::Switch;
            state.switch_seq += 1;
            state.phase = Phase::Switching {
                primary,
                candidate: run.clone(),
                switch_id: state.switch_seq.to_string(),
            };
            publish_snapshot(&mut state);
            state.operation
        };
        if let Err(error) = self.inner.promote_candidate(
            next_operation,
            &run,
            run.capability_snapshot.clone().expect("ready capabilities"),
        ) {
            let switch_id = self.lock().switch_seq.to_string();
            self.inner
                .fail_switch_candidate(next_operation, &run, &switch_id, error.clone());
            return Err(error);
        }
        Ok(true)
    }
}

impl Inner {
    pub(super) fn preparation_current(&self, operation: u64, run: &EngineRunDto, background: bool) -> bool {
        let state = self.lock();
        state.operation == operation && (!background || preparing(&state, &run.run_id))
    }

    pub(super) fn finish_preload(
        self: &Arc<Self>,
        operation: u64,
        mut run: EngineRunDto,
        capabilities: EngineCapabilitySnapshotDto,
        resources: ResourceSnapshot,
    ) -> Result<(), EngineFailureDto> {
        let mut state = self.lock();
        if state.operation != operation || !preparing(&state, &run.run_id) {
            return Ok(());
        }
        let slot = state
            .preloads
            .iter_mut()
            .find(|slot| slot.dto.run.run_id == run.run_id)
            .expect("preparing slot");
        if !slot
            .live
            .as_mut()
            .is_some_and(|live| matches!(live.child.try_wait(), Ok(None)))
        {
            return Err(invalid(
                &run.profile_id,
                "Prepared engine exited before readiness",
            ));
        }
        run.capability_snapshot = Some(capabilities);
        slot.dto.run = run.clone();
        slot.dto.phase = EnginePreloadPhaseDto::Ready;
        slot.resources = Some(Arc::new(resources));
        drop(state);
        let inner = self.clone();
        thread::spawn(move || loop {
            thread::sleep(Duration::from_millis(50));
            let mut state = inner.lock();
            let Some(slot) = state.preloads.iter_mut().find(|slot| {
                slot.dto.run.run_id == run.run_id && slot.dto.phase == EnginePreloadPhaseDto::Ready
            }) else {
                return;
            };
            let alive = slot
                .live
                .as_mut()
                .is_some_and(|live| matches!(live.child.try_wait(), Ok(None)));
            // Drain unsolicited background stdout without ever publishing analysis.
            if let Some(rx) = slot.live.as_ref().and_then(|live| live.stdout_rx.as_ref()) {
                for _ in 0..64 {
                    if rx.try_recv().is_err() {
                        break;
                    }
                }
            }
            drop(state);
            if !alive {
                inner.fail_preload(&run.run_id, invalid(&run.profile_id, "Background engine exited"));
                return;
            }
        });
        Ok(())
    }

    fn fail_preload(self: &Arc<Self>, run_id: &str, mut error: EngineFailureDto) {
        let mut state = self.lock();
        let Some(index) = state.preloads.iter().position(|slot| {
            slot.dto.run.run_id == run_id
                && matches!(
                    slot.dto.phase,
                    EnginePreloadPhaseDto::Preparing | EnginePreloadPhaseDto::Ready
                )
        }) else {
            return;
        };
        let slot = &mut state.preloads[index];
        if let Some(live) = &slot.live {
            attach_startup_failure(&mut error, &live.startup_output, &live.capture);
        }
        slot.dto.phase = EnginePreloadPhaseDto::Failed;
        slot.dto.failure = Some(error);
        slot.resources = None;
        let retiring = slot
            .live
            .take()
            .map(|live| (slot.dto.run.clone(), Arc::new(Mutex::new(live))));
        if let Some(retiring) = retiring {
            state.retiring.push(retiring);
        }
        drop(state);
        let _ = ForegroundEngineManager { inner: self.clone() }.reap_retiring_runs();
    }
}
