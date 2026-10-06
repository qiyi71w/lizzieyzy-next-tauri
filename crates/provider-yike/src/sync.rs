//! One sleeping worker owns Yike polling; no queued or overlapping polls.
use app_model::{ProviderImportResult, ProviderRequestIdentityDto};
use provider_core::{network::NetworkOperation, ProviderResult};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

pub enum YikeSyncWork {
    Wait(Option<Duration>),
    Fetch {
        operation: NetworkOperation,
        locator: String,
    },
}

pub trait YikeSyncHost: Send + Sync + 'static {
    fn next(&self) -> YikeSyncWork;
    fn completed(&self, identity: ProviderRequestIdentityDto, result: ProviderResult<ProviderImportResult>);
}

#[derive(Default)]
struct WorkerState {
    wake: u64,
    stopped: bool,
    running: bool,
}

#[derive(Default)]
pub struct YikeSyncRuntime {
    state: Arc<(Mutex<WorkerState>, Condvar)>,
}
impl YikeSyncRuntime {
    pub fn start(&self, host: impl YikeSyncHost) -> Result<(), String> {
        let state = self.state.clone();
        {
            let mut worker = state.0.lock().expect("Yike sync runtime");
            if worker.running || worker.stopped {
                return Err("Yike sync worker cannot be started twice.".into());
            }
            worker.running = true;
        }
        if let Err(error) = std::thread::Builder::new()
            .name("yike-sync".into())
            .spawn(move || {
                loop {
                    let revision = {
                        let worker = state.0.lock().expect("Yike sync runtime");
                        if worker.stopped {
                            break;
                        }
                        worker.wake
                    };
                    match host.next() {
                        YikeSyncWork::Fetch { operation, locator } => {
                            let result = crate::fetch_public_preview(&operation, &locator);
                            host.completed(operation.identity(), result);
                        }
                        YikeSyncWork::Wait(delay) => {
                            let worker = state.0.lock().expect("Yike sync runtime");
                            if worker.stopped {
                                break;
                            }
                            if worker.wake != revision {
                                continue;
                            }
                            if let Some(delay) = delay {
                                drop(
                                    state
                                        .1
                                        .wait_timeout_while(worker, delay, |worker| {
                                            !worker.stopped && worker.wake == revision
                                        })
                                        .expect("Yike sync timer"),
                                );
                            } else {
                                drop(
                                    state
                                        .1
                                        .wait_while(worker, |worker| {
                                            !worker.stopped && worker.wake == revision
                                        })
                                        .expect("Yike sync wake"),
                                );
                            }
                        }
                    }
                }
                state.0.lock().expect("Yike sync runtime").running = false;
                state.1.notify_all();
            })
        {
            self.state.0.lock().expect("Yike sync runtime").running = false;
            return Err(format!("Could not start Yike sync worker: {error}"));
        }
        Ok(())
    }

    pub fn wake(&self) {
        let mut worker = self.state.0.lock().expect("Yike sync runtime");
        worker.wake = worker.wake.wrapping_add(1);
        self.state.1.notify_all();
    }

    /// The current-game owner seals and cancels its request before this bounded drain.
    pub fn shutdown(&self, budget: Duration) -> bool {
        let deadline = Instant::now() + budget;
        let mut worker = self.state.0.lock().expect("Yike sync runtime");
        worker.stopped = true;
        self.state.1.notify_all();
        while worker.running {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return false;
            }
            worker = self
                .state
                .1
                .wait_timeout(worker, remaining)
                .expect("Yike sync drain")
                .0;
        }
        true
    }
}
