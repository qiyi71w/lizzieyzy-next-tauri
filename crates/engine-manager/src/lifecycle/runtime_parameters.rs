use super::*;
use app_model::{
    RuntimeControlIdentityDto, RuntimeParameterPairDto, RuntimeParametersSnapshotDto,
    RuntimeParametersStatusDto as Status,
};

fn snapshot(state: &ManagerState) -> RuntimeParametersSnapshotDto {
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
                        && facts.commands.iter().any(|command| command == "kata-get-param")
                })
    });
    let mut result = state
        .runtime_parameters
        .as_ref()
        .filter(|previous| previous.run_id.as_deref() == run.map(|run| run.run_id.as_str()))
        .cloned()
        .unwrap_or(RuntimeParametersSnapshotDto {
            run_id: run.map(|run| run.run_id.clone()),
            profile_revision: resource.map(|resource| resource.profile_revision.clone()),
            supported,
            reason: None,
            request_id: None,
            last_valid: None,
            status: Status::Unknown,
            failure: None,
        });
    result.supported = supported && matches!(state.phase, Phase::Ready(_));
    result.reason = if !supported {
        Some("Requires a qualified KataGo 1.18.2 GTP Run; saved parameters and JSONL query overrides are separate.".into())
    } else if !matches!(state.phase, Phase::Ready(_)) {
        Some("Wait for the explicit lifecycle operation to finish.".into())
    } else {
        None
    };
    result
}

impl ForegroundEngineManager {
    /// Read retained state only. Never issues protocol commands or starts a process.
    pub fn runtime_parameters_snapshot(&self) -> RuntimeParametersSnapshotDto {
        snapshot(&self.lock())
    }

    /// One explicit read-only round, with no partial publication or automatic retry.
    pub fn read_runtime_parameters(
        &self,
        identity: RuntimeControlIdentityDto,
    ) -> Result<RuntimeParametersSnapshotDto, String> {
        {
            let current = snapshot(&self.lock());
            if !current.supported
                || current.run_id.as_deref() != Some(&identity.run_id)
                || current.profile_revision.as_deref() != Some(&identity.profile_revision)
            {
                return Err("parameter readback requires the captured qualified current GTP Run".into());
            }
        }
        let transaction = self.begin_runtime_control(&identity)?;
        {
            let mut state = self.lock();
            transaction.current(&state)?;
            let mut current = snapshot(&state);
            current.request_id = Some(identity.request_id.clone());
            current.status = Status::Pending;
            current.failure = None;
            state.runtime_parameters = Some(current);
        }
        let result = (|| {
            transaction.drain()?;
            let (pda, wrn) = transaction.parameter_pair()?;
            // Frozen 1.18.2 setup.cpp and gtp.cpp parameter domains. Both values
            // are dimensionless, not percentages; PDA counts playout doublings.
            let pda = finite_parameter(&pda, -3.0, 3.0)?;
            let wrn = finite_parameter(&wrn, 0.0, 5.0)?;
            Ok(RuntimeParameterPairDto {
                playout_doubling_advantage: pda,
                analysis_wide_root_noise: wrn,
            })
        })();
        let current = {
            let mut state = self.lock();
            let mut current = snapshot(&state);
            if current.run_id.as_deref() != Some(&identity.run_id)
                || current.request_id.as_deref() != Some(&identity.request_id)
            {
                return Err("parameter response belongs to an expired Run or request".into());
            }
            match result.and_then(|pair| transaction.current(&state).map(|()| pair)) {
                Ok(pair) => {
                    current.last_valid = Some(pair);
                    current.status = Status::Confirmed;
                }
                Err(error) => {
                    current.status = Status::Failed;
                    current.failure = Some(error);
                }
            }
            state.runtime_parameters = Some(current.clone());
            current
        };
        drop(transaction);
        Ok(current)
    }
}

fn finite_parameter(body: &str, minimum: f64, maximum: f64) -> Result<f64, String> {
    let value = body
        .trim()
        .parse::<f64>()
        .map_err(|_| "parameter reply is not a single finite number".to_owned())?;
    if !value.is_finite() || !(minimum..=maximum).contains(&value) {
        return Err("parameter reply is outside the qualified finite domain".into());
    }
    Ok(value)
}
