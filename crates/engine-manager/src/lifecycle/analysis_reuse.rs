use super::*;

/// Comparable analysis conditions, not a search-tree identity. Never persisted in SGF.
#[derive(Debug, Clone, PartialEq)]
pub struct OrdinaryAnalysisContext {
    run_id: String,
    resource: Arc<app_model::QualifiedLocalResourceDto>,
    generation: u64,
    node_path: NodePath,
    query: serde_json::Value,
    position: app_model::ExactPositionDto,
    parameters: Option<app_model::RuntimeParameterPairDto>,
    threads: Option<u32>,
}

impl OrdinaryAnalysisContext {
    pub fn ownership_requested(&self) -> bool {
        self.query
            .get("includeOwnership")
            .and_then(serde_json::Value::as_bool)
            == Some(true)
    }
}

pub(super) fn capture(
    state: &ManagerState,
    run: &EngineRunDto,
    request: &SelectedNodeJobRequest,
) -> Option<Arc<OrdinaryAnalysisContext>> {
    let resource = run.qualified_resource.as_ref()?.clone();
    let position = request.exact_position.as_ref().ok()?.dto().clone();
    let (parameters, threads) = if run.adapter_kind == EngineBackend::KataGoGtp {
        let parameters = state.runtime_parameters.as_ref()?;
        let threads = state.runtime_threads.as_ref()?;
        if parameters.run_id.as_deref() != Some(&run.run_id)
            || parameters.profile_revision.as_deref() != Some(&resource.profile_revision)
            || parameters.status != app_model::RuntimeParametersStatusDto::Confirmed
            || threads.run_id.as_deref() != Some(&run.run_id)
            || threads.profile_revision.as_deref() != Some(&resource.profile_revision)
            || threads.status != app_model::RuntimeThreadsStatusDto::Confirmed
            || state.runtime_control.is_some()
        {
            return None;
        }
        (Some(parameters.last_valid.clone()?), Some(threads.actual?))
    } else {
        (None, None)
    };
    // These fields are scheduling/budget only. Keep every semantic/output option.
    let mut query = serde_json::to_value(&request.query).ok()?;
    let object = query.as_object_mut()?;
    for key in ["id", "maxVisits", "reportDuringSearchEvery", "overrideSettings"] {
        object.remove(key);
    }
    Some(Arc::new(OrdinaryAnalysisContext {
        run_id: run.run_id.clone(),
        resource,
        generation: request.generation,
        node_path: request.node_path.clone(),
        query,
        position,
        parameters,
        threads,
    }))
}

impl ForegroundEngineManager {
    /// Reads only the immutable registration capture. A delayed event never acquires
    /// the current Run's or current runtime settings' identity retroactively.
    pub fn ordinary_analysis_context(
        &self,
        event: &AnalysisJobEventDto,
    ) -> Option<Arc<OrdinaryAnalysisContext>> {
        let state = self.lock();
        state
            .jobs
            .iter()
            .find(|job| {
                job.run_id == event.run_id
                    && job.job_id == event.job_id
                    && job.generation == event.generation
                    && job.node_path == event.node_path
                    && job.lane == AnalysisJobLane::SelectedNode
                    && job.submitted
            })
            .and_then(|job| job.ordinary_context.clone())
            .or_else(|| {
                state
                    .last_ordinary_context
                    .as_ref()
                    .filter(|(scope, _)| {
                        scope.run_id == event.run_id
                            && scope.job_id == event.job_id
                            && scope.generation == event.generation
                            && scope.node_path == event.node_path
                    })
                    .map(|(_, context)| context.clone())
            })
    }

    /// Called only by the serialized durable preferences owner after a successful write.
    /// This prospective presentation policy never reconciles or restarts analysis.
    pub fn set_retain_node_analysis(&self, enabled: bool) {
        self.inner
            .retain_node_analysis
            .store(enabled, std::sync::atomic::Ordering::Release);
    }

    pub fn retain_node_analysis(&self) -> bool {
        self.inner
            .retain_node_analysis
            .load(std::sync::atomic::Ordering::Acquire)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordinary_reuse_context_captures_runtime_and_excludes_only_budgets() {
        let manager = ForegroundEngineManager::new(
            Arc::new(crate::InMemoryEngineProfileCatalog::new()),
            ForegroundEngineConfig::for_tests(),
        );
        let run = EngineRunDto {
            measured_scene: None,
            measured_scenes_snapshot: Default::default(),
            run_id: "run".into(),
            profile_id: "profile".into(),
            adapter_kind: EngineBackend::KataGoGtp,
            profile_snapshot: app_model::EngineProfileDto {
                name: "test".into(),
                program: "test".into(),
                argv: vec![],
                working_dir: None,
                adapter: app_model::EngineAdapterSettings::KataGoGtp(app_model::KataGoSettings {
                    model_path: None,
                    config_path: None,
                    max_visits: 800,
                }),
            },
            thread_policy_snapshot: Default::default(),
            capability_snapshot: None,
            qualified_resource: Some(Arc::new(app_model::QualifiedLocalResourceDto {
                profile_revision: "revision".into(),
                resources: vec![],
                origin: "fixture".into(),
                version: None,
                source_commit: None,
                backend: None,
                static_zlib_exemption: false,
                thread_sources: None,
            })),
        };
        let document = sgf::CurrentSgfDocument::open("(;SZ[9]RU[Chinese]KM[7.5])").unwrap();
        let mut request = SelectedNodeJobRequest {
            run_id: run.run_id.clone(), mode: AnalysisJobModeDto::Continuous, generation: 1, node_path: NodePath::default(),
            query: serde_json::from_value(serde_json::json!({"id":"query","moves":[],"rules":"chinese","komi":7.5,"boardXSize":9,"boardYSize":9})).unwrap(),
            board_width: 9, board_height: 9, position_empty: true, exact_position: document.exact_position(&NodePath::default()),
        };
        let mut state = manager.lock();
        assert!(capture(&state, &run, &request).is_none());
        state.runtime_parameters = Some(app_model::RuntimeParametersSnapshotDto {
            run_id: Some("run".into()),
            profile_revision: Some("revision".into()),
            supported: true,
            reason: None,
            request_id: None,
            last_valid: Some(app_model::RuntimeParameterPairDto {
                playout_doubling_advantage: 0.0,
                analysis_wide_root_noise: 0.0,
            }),
            status: app_model::RuntimeParametersStatusDto::Confirmed,
            failure: None,
        });
        state.runtime_threads = Some(app_model::RuntimeThreadsSnapshotDto {
            run_id: Some("run".into()),
            profile_revision: Some("revision".into()),
            supported: true,
            reason: None,
            minimum: 1,
            maximum: 64,
            sources: None,
            request_id: None,
            requested: None,
            actual: Some(1),
            temporary: false,
            status: app_model::RuntimeThreadsStatusDto::Confirmed,
            failure: None,
        });
        let before = capture(&state, &run, &request).unwrap();
        request.query.max_visits = Some(100);
        request.query.id = "new-budget".into();
        request.query.continuous(ContinuousAnalysisBudgetDto::default());
        assert_eq!(capture(&state, &run, &request).as_ref(), Some(&before));
        state
            .runtime_parameters
            .as_mut()
            .unwrap()
            .last_valid
            .as_mut()
            .unwrap()
            .playout_doubling_advantage = 1.0;
        let after = capture(&state, &run, &request).unwrap();
        assert_ne!(before, after);
        assert_eq!(
            before.parameters.as_ref().unwrap().playout_doubling_advantage,
            0.0
        );
        state.runtime_parameters.as_mut().unwrap().status = app_model::RuntimeParametersStatusDto::Failed;
        assert!(capture(&state, &run, &request).is_none());
        state.runtime_parameters.as_mut().unwrap().status = app_model::RuntimeParametersStatusDto::Confirmed;
        request.query.include_policy = Some(true);
        assert_ne!(capture(&state, &run, &request).as_ref(), Some(&after));
        let mut other_run = run.clone();
        other_run.run_id = "another-run".into();
        assert!(capture(&state, &other_run, &request).is_none());
    }
}
