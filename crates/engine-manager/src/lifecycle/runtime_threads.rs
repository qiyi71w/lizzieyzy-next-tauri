use super::*;
use app_model::{
    RuntimeThreadsActionDto as Action, RuntimeThreadsRequestDto, RuntimeThreadsSnapshotDto,
    RuntimeThreadsStatusDto as Status,
};

fn snapshot(state: &ManagerState) -> RuntimeThreadsSnapshotDto {
    let run = match &state.phase {
        Phase::Ready(run) => Some(run),
        Phase::Switching { primary, .. } => Some(primary),
        _ => None,
    };
    let resource = run.and_then(|run| run.qualified_resource.as_ref());
    let supported = run.is_some_and(|run| {
        run.adapter_kind == EngineBackend::KataGoGtp
            && resource.is_some()
            && run
                .capability_snapshot
                .as_ref()
                .and_then(|caps| caps.gtp.as_ref())
                .is_some_and(|facts| {
                    facts.name == "KataGo"
                        && facts.version.split('+').next() == Some("1.18.2")
                        && ["kata-set-param", "kata-get-param"]
                            .iter()
                            .all(|command| facts.commands.iter().any(|value| value == command))
                })
    });
    let mut result = state
        .runtime_threads
        .as_ref()
        .filter(|previous| previous.run_id.as_deref() == run.map(|run| run.run_id.as_str()))
        .cloned()
        .unwrap_or(RuntimeThreadsSnapshotDto {
            run_id: run.map(|run| run.run_id.clone()),
            profile_revision: resource.map(|r| r.profile_revision.clone()),
            supported,
            reason: None,
            minimum: 1,
            maximum: 4096,
            sources: resource.and_then(|r| r.thread_sources.clone()),
            request_id: None,
            requested: None,
            actual: None,
            temporary: false,
            status: Status::Unknown,
            failure: None,
        });
    result.supported = supported && matches!(state.phase, Phase::Ready(_));
    result.reason = if !supported {
        Some("Requires a qualified KataGo 1.18.2 GTP Run; JSONL query overrides are separate.".into())
    } else if !matches!(state.phase, Phase::Ready(_)) {
        Some("Wait for the explicit lifecycle operation to finish.".into())
    } else {
        None
    };
    result
}

impl ForegroundEngineManager {
    pub fn runtime_threads_snapshot(&self) -> RuntimeThreadsSnapshotDto {
        snapshot(&self.lock())
    }

    /// Explicit read, Apply or reset; never persists configuration or starts a process.
    pub fn runtime_threads(
        &self,
        request: RuntimeThreadsRequestDto,
    ) -> Result<RuntimeThreadsSnapshotDto, String> {
        let requested = {
            let state = self.lock();
            let current = snapshot(&state);
            if !current.supported
                || current.run_id.as_deref() != Some(&request.identity.run_id)
                || current.profile_revision.as_deref() != Some(&request.identity.profile_revision)
            {
                return Err("runtime threads require the captured qualified current GTP Run".into());
            }
            let value = match request.action {
                Action::Read => None,
                Action::Apply => Some(
                    request
                        .value
                        .ok_or("Apply requires a whole-number thread value")?,
                ),
                Action::Reset => Some(
                    current
                        .sources
                        .as_ref()
                        .and_then(|sources| sources.effective)
                        .ok_or("recorded effective launch thread source is unknown; reset is unavailable")?,
                ),
            };
            if value.is_some_and(|value| !(1..=4096).contains(&value)) {
                return Err("KataGo 1.18.2 requires numSearchThreads in 1..4096".into());
            }
            value
        };
        let transaction = self.begin_runtime_control(&request.identity)?;
        {
            let mut state = self.lock();
            transaction.current(&state)?;
            let mut current = snapshot(&state);
            current.request_id = Some(request.identity.request_id.clone());
            current.requested = requested;
            current.status = Status::Pending;
            current.failure = None;
            state.runtime_threads = Some(current);
        }
        let result = (|| {
            transaction.drain()?;
            if let Some(value) = requested {
                let ack = transaction.command(&format!("kata-set-param numSearchThreads {value}"))?;
                if !ack.trim().is_empty() {
                    return Err("unexpected thread set acknowledgement body".to_owned());
                }
            }
            let body = transaction.command("kata-get-param numSearchThreads")?;
            let value = body
                .trim()
                .parse::<u32>()
                .map_err(|_| "thread readback is not a whole number".to_owned())?;
            if !(1..=4096).contains(&value) {
                return Err("thread readback is outside the qualified domain".into());
            }
            if requested.is_some_and(|requested| requested != value) {
                return Err("thread readback did not confirm the requested value".into());
            }
            Ok(value)
        })();
        let current = {
            let mut state = self.lock();
            let mut current = snapshot(&state);
            if current.run_id.as_deref() != Some(&request.identity.run_id)
                || current.request_id.as_deref() != Some(&request.identity.request_id)
            {
                return Err("thread response belongs to an expired Run or request".into());
            }
            match result.and_then(|value| transaction.current(&state).map(|()| value)) {
                Ok(value) => {
                    current.actual = Some(value);
                    current.status = Status::Confirmed;
                    if request.action != Action::Read {
                        current.temporary = request.action == Action::Apply;
                    }
                }
                Err(error) => {
                    current.status = Status::Failed;
                    current.failure = Some(error);
                }
            }
            state.runtime_threads = Some(current.clone());
            current
        };
        drop(transaction);
        Ok(current)
    }
}
