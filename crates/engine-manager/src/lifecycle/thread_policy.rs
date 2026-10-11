use super::*;
use crate::{thread_policy as policy, EngineProfilesSettings};
use app_model::{
    RecommendationValidityDto, ThreadPolicySaveRequestDto, ThreadPolicyStatusDto, ThreadStrategyDto,
};

impl ForegroundEngineManager {
    pub fn thread_policy_status(
        &self,
        target_id: &str,
        evaluation_id: Option<&str>,
    ) -> Result<ThreadPolicyStatusDto, String> {
        let saved = self.inner.catalog.get(target_id).ok_or("policy_target_deleted")?;
        let recommendation = if let Some(id) = evaluation_id {
            let snapshot = self.evaluation_snapshot();
            let result = snapshot
                .result
                .filter(|result| result.evaluation_id == id && result.target_id == target_id)
                .ok_or("policy_result_retired")?;
            result.recommendation
        } else {
            saved.thread_policy.recommendation.clone()
        };
        let (validity, reason) = if recommendation.is_none() {
            (RecommendationValidityDto::None, "policy_none".into())
        } else {
            match build_command_spec(&saved.profile) {
                Ok(spec) => {
                    let qualified = policy::validity(
                        &saved,
                        recommendation.as_ref(),
                        &spec,
                        self.inner.config.managed_resources_root.as_deref(),
                    );
                    let run = starting_run(&saved);
                    let configuration = crate::katago_config::Configuration::from_command(
                        &spec,
                        Instant::now() + Duration::from_secs(30),
                    );
                    match configuration.and_then(|config| {
                        config
                            .policy_override(run.adapter_kind, &AttemptCapture::new(&run))
                            .map_err(Into::into)
                    }) {
                        Ok(crate::katago_config::PolicyOverride::Legal) => {
                            (qualified.0, "policy_override".into())
                        }
                        Ok(crate::katago_config::PolicyOverride::Deleted) => {
                            (qualified.0, "policy_deleted_override".into())
                        }
                        Err(_) => (RecommendationValidityDto::Invalid, "policy_invalid_config".into()),
                        _ => qualified,
                    }
                }
                Err(_) => (
                    RecommendationValidityDto::Unknown,
                    "policy_resources_unknown".into(),
                ),
            }
        };
        let (validity, reason) = if recommendation.as_ref().is_some_and(|r| {
            r.policy_revision != saved.thread_policy.revision
                && saved
                    .thread_policy
                    .recommendation
                    .as_ref()
                    .is_none_or(|current| current.evaluation_id != r.evaluation_id)
        }) {
            (
                RecommendationValidityDto::Changed,
                "policy_revision_changed".into(),
            )
        } else {
            (validity, reason)
        };
        Ok(ThreadPolicyStatusDto {
            target_id: target_id.into(),
            input_revision: policy::input_revision(&saved.profile),
            policy: saved.thread_policy,
            validity,
            reason,
            recommendation,
        })
    }

    /// Caller holds the existing catalog transaction lock. No process mutation precedes persistence.
    pub fn save_thread_policy_at_path(
        &self,
        path: &std::path::Path,
        mut current: EngineProfilesSettings,
        request: ThreadPolicySaveRequestDto,
    ) -> Result<EngineProfilesSettings, String> {
        let record = current
            .profiles
            .iter_mut()
            .find(|r| r.id == request.target_id)
            .ok_or("policy_target_deleted")?;
        if record.thread_policy.revision != request.expected_revision
            || policy::input_revision(&record.profile) != request.input_revision
        {
            return Err("policy_revision_changed".into());
        }
        let saved = self
            .inner
            .catalog
            .get(&request.target_id)
            .ok_or("policy_target_deleted")?;
        if saved.profile != record.profile || saved.thread_policy != record.thread_policy {
            return Err("policy_revision_changed".into());
        }
        let recommendation = if request.strategy == ThreadStrategyDto::Benchmark {
            let status = self.thread_policy_status(&request.target_id, request.evaluation_id.as_deref())?;
            if status.validity != RecommendationValidityDto::Valid {
                return Err(status.reason);
            }
            status.recommendation
        } else {
            record.thread_policy.recommendation.clone()
        };
        // Seal evaluation identity across persistence: Cancel/replacement cannot race a late Save.
        let evaluation_guard = self.lock();
        if request.strategy == ThreadStrategyDto::Benchmark
            && request.evaluation_id.is_some()
            && !recommendation.as_ref().is_some_and(|rec| {
                evaluation_guard
                    .evaluation
                    .as_ref()
                    .is_some_and(|slot| slot.retains_recommendation(rec))
            })
        {
            return Err("policy_result_retired".into());
        }
        record.thread_policy = app_model::SavedThreadPolicyDto {
            strategy: request.strategy,
            revision: record
                .thread_policy
                .revision
                .checked_add(1)
                .ok_or("policy_revision_exhausted")?,
            recommendation,
        };
        let saved = crate::save_engine_profiles(path, current)?;
        drop(evaluation_guard);
        // The existing preparation owner seals and retires the old policy only after a successful write.
        self.invalidate_preload(&request.target_id);
        Ok(saved)
    }
}
