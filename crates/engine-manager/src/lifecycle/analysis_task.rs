//! Finite task execution on the foreground manager's existing state and reader.
use super::*;

#[derive(Default)]
pub(super) struct AnalysisTaskTargets {
    pub(super) requested: Option<Vec<NodePath>>,
    pub(super) supporting: Vec<NodePath>,
    pub(super) swing_comparisons: Vec<AnalysisSwingComparisonDto>,
    pub(super) swing_criteria: Option<AnalysisSwingCriteriaDto>,
}

pub(super) struct AnalysisTaskPlan {
    pub(super) strategy: AnalysisTaskStrategyDto,
    pub(super) overview_conditions: Option<AnalysisStageConditionsDto>,
    pub(super) conditions: AnalysisStageConditionsDto,
}

pub(super) enum WholeGameAdvance {
    Submit {
        started: AnalysisJobStartedDto,
        jsonl: String,
    },
    Terminate {
        run_id: String,
        job_id: String,
        query_id: String,
    },
}

impl ForegroundEngineManager {
    pub fn analysis_task_snapshot(&self) -> Option<AnalysisTaskDto> {
        self.lock()
            .analysis_task
            .as_ref()
            .filter(|task| !task.job_id.starts_with("batch:"))
            .cloned()
    }

    pub fn check_analysis_task_admission(
        &self,
        swing_criteria: Option<&AnalysisSwingCriteriaDto>,
    ) -> Result<(), EngineFailureDto> {
        let state = self.lock();
        match_reservation::require_unreserved(&state)?;
        let run = current_admitting_run(&state.phase).ok_or_else(|| {
            continuous_invalid_state(&state, "Analysis task requires a Ready Foreground Engine Run")
        })?;
        validate_whole_game_capabilities(&run, swing_criteria)
    }

    pub fn start_whole_game_analysis(
        &self,
        request: WholeGameJobRequest,
    ) -> Result<AnalysisJobStartedDto, EngineFailureDto> {
        let Some(first) = request.work_items.first() else {
            return Err(empty_whole_game_failure(&request.run_id));
        };
        let total_visits = first.query.max_visits.unwrap_or(0);
        let scope = AnalysisScopeDto {
            mode: AnalysisScopeModeDto::FirstChildMainline,
            current_node: first.node_path.clone(),
            branch_choices: Vec::new(),
            interval: None,
            to_play: None,
        };
        let conditions = single_stage_conditions(total_visits);
        let task = self.start_analysis_task(request, scope, conditions)?;
        let node_path = task
            .requested
            .first()
            .cloned()
            .unwrap_or(NodePath { indices: Vec::new() });
        Ok(AnalysisJobStartedDto {
            run_id: task.run_id,
            job_id: task.job_id,
            lane: AnalysisJobLaneDto::WholeGame,
            mode: AnalysisJobModeDto::Finite,
            state: AnalysisJobStateDto::Queued,
            generation: task.generation,
            node_path,
        })
    }

    pub fn start_analysis_task(&self,request:WholeGameJobRequest,scope:AnalysisScopeDto,conditions:AnalysisStageConditionsDto)->Result<AnalysisTaskDto,EngineFailureDto>{
        if self.measured_reservation(){return Err(invalid_task_conditions(&request.run_id,"A leased measurement scene owns this Run; explicitly Stop before ordinary Task admission".into()));}
        self.start_analysis_task_owned(request,scope,conditions)
    }
    pub fn start_measured_analysis_task(&self,request:WholeGameJobRequest,scope:AnalysisScopeDto,conditions:AnalysisStageConditionsDto,lease:&measured_report::SetupLease)->Result<AnalysisTaskDto,EngineFailureDto>{
        lease.check().map_err(|message|invalid_task_conditions(&request.run_id,message))?;
        self.start_analysis_task_owned(request,scope,conditions)
    }
    fn start_analysis_task_owned(
        &self,
        request: WholeGameJobRequest,
        scope: AnalysisScopeDto,
        conditions: AnalysisStageConditionsDto,
    ) -> Result<AnalysisTaskDto, EngineFailureDto> {
        conditions
            .validate_single_stage()
            .map_err(|message| invalid_task_conditions(&request.run_id, message))?;
        self.retire_auto_load()?;
        self.start_analysis_task_with_strategy(
            request,
            scope,
            AnalysisTaskPlan {
                strategy: AnalysisTaskStrategyDto::SingleStage,
                overview_conditions: None,
                conditions,
            },
            AnalysisTaskTargets::default(),
            None,
            None,
        )
    }

    pub fn start_all_positions_analysis_task(
        &self,
        request: WholeGameJobRequest,
        scope: AnalysisScopeDto,
        overview_conditions: AnalysisStageConditionsDto,
        deep_conditions: AnalysisStageConditionsDto,
    ) -> Result<AnalysisTaskDto, EngineFailureDto> {
        if self.measured_reservation(){return Err(invalid_task_conditions(&request.run_id,"A leased measurement scene owns this Run; explicitly Stop before ordinary Task admission".into()));}
        AnalysisStageConditionsDto::validate_all_positions_two_stage(&overview_conditions, &deep_conditions)
            .map_err(|message| invalid_task_conditions(&request.run_id, message))?;
        self.retire_auto_load()?;
        self.start_analysis_task_with_strategy(
            request,
            scope,
            AnalysisTaskPlan {
                strategy: AnalysisTaskStrategyDto::AllPositionsTwoStage,
                overview_conditions: Some(overview_conditions),
                conditions: deep_conditions,
            },
            AnalysisTaskTargets::default(),
            None,
            None,
        )
    }

    pub fn start_swing_analysis_task(
        &self,
        request: SwingAnalysisTaskRequest,
    ) -> Result<AnalysisTaskDto, EngineFailureDto> {
        let SwingAnalysisTaskRequest {
            job: request,
            scope,
            requested,
            supporting,
            swing_comparisons,
            swing_criteria,
            overview_conditions,
            deep_conditions,
        } = request;
        if self.measured_reservation(){return Err(invalid_task_conditions(&request.run_id,"A leased measurement scene owns this Run; explicitly Stop before ordinary Task admission".into()));}
        overview_conditions
            .validate_single_stage()
            .and_then(|_| deep_conditions.validate_single_stage())
            .and_then(|_| swing_criteria.validate())
            .map_err(|message| invalid_task_conditions(&request.run_id, message))?;
        self.retire_auto_load()?;
        self.start_analysis_task_with_strategy(
            request,
            scope,
            AnalysisTaskPlan {
                strategy: AnalysisTaskStrategyDto::SwingSelectedTwoStage,
                overview_conditions: Some(overview_conditions),
                conditions: deep_conditions,
            },
            AnalysisTaskTargets {
                requested: Some(requested),
                supporting,
                swing_comparisons,
                swing_criteria: Some(swing_criteria),
            },
            None,
            None,
        )
    }

    pub(super) fn start_analysis_task_with_strategy(
        &self,
        mut request: WholeGameJobRequest,
        scope: AnalysisScopeDto,
        plan: AnalysisTaskPlan,
        targets: AnalysisTaskTargets,
        auto_intent: Option<app_model::AutoLoadSnapshotDto>,
        batch_identity: Option<app_model::BatchIdentityDto>,
    ) -> Result<AnalysisTaskDto, EngineFailureDto> {
        let AnalysisTaskPlan {
            strategy,
            overview_conditions,
            conditions,
        } = plan;
        if request.work_items.is_empty() {
            return Err(empty_whole_game_failure(&request.run_id));
        }
        let initial_conditions = overview_conditions.as_ref().unwrap_or(&conditions);
        for item in &mut request.work_items {
            item.query.task(initial_conditions);
        }

        let cancel = AnalysisCancelToken::new();
        let (task, bound_query) = {
            let mut state = self.lock();
            continuation::require_idle(&state)?;
            batch::admit(&state, batch_identity.as_ref())?;
            if let Some(intent) = &auto_intent {
                auto_load::admit(&state, intent)?;
            }
            match_reservation::require_unreserved(&state)?;
            game_move::require_idle_move(&state)?;
            runtime_control::require_idle(&state)?;
            let run = match admitting_run(&state.phase, &request.run_id) {
                Some(run) => run,
                None => {
                    return Err(failure(
                        EngineOperationDto::Job,
                        EngineFailureKind::InvalidState,
                        "Analysis Job admission requires a Ready Foreground Engine Run".into(),
                        Some(request.run_id.as_str()),
                        None,
                        None,
                    ));
                }
            };
            validate_whole_game_capabilities(&run, targets.swing_criteria.as_ref())?;
            validate_task_conditions_capabilities(&run, &conditions)?;
            if let Some(overview) = overview_conditions.as_ref() {
                validate_task_conditions_capabilities(&run, overview)?;
            }
            for item in &request.work_items {
                validate_query_capabilities(&run, &item.query, false)?;
            }
            if state.analysis_task.as_ref().is_some_and(|task| {
                matches!(
                    task.state,
                    AnalysisTaskStateDto::Pausing | AnalysisTaskStateDto::Paused
                )
            }) {
                return Err(failure(
                    EngineOperationDto::Job,
                    EngineFailureKind::Occupied,
                    "The paused analysis task still owns this lane; Continue or Cancel it.".into(),
                    Some(&request.run_id),
                    None,
                    None,
                ));
            }
            if let Some(occupied) = state.jobs.iter().find(|job| {
                job.run_id == request.run_id && job.lane == AnalysisJobLane::WholeGame && !job.terminal
            }) {
                return Err(failure(
                    EngineOperationDto::Job,
                    EngineFailureKind::Occupied,
                    "whole-game analysis is already running on this Foreground Engine Run".into(),
                    Some(request.run_id.as_str()),
                    Some(run.profile_id.as_str()),
                    None,
                )
                .with_job_id(&occupied.job_id));
            }

            let task_id = Uuid::new_v4().to_string();
            let job_id = match &batch_identity {
                Some(id) => format!(
                    "batch:{}:{}:{}:{}",
                    id.batch_id,
                    id.row_id,
                    id.attempt,
                    Uuid::new_v4()
                ),
                None => Uuid::new_v4().to_string(),
            };
            let query_id = target_query_id(&job_id);
            let first_node_path = request.work_items[0].node_path.clone();
            let first_board_width = request.work_items[0].board_width;
            let first_board_height = request.work_items[0].board_height;
            let bound_query = bound_work_item_query(&request.work_items[0], &query_id, &job_id)?;
            evaluation::yield_to_foreground(&mut state)?;
            let started = AnalysisJobStartedDto {
                run_id: request.run_id.clone(),
                job_id: job_id.clone(),
                lane: AnalysisJobLaneDto::WholeGame,
                mode: AnalysisJobModeDto::Finite,
                state: AnalysisJobStateDto::Queued,
                generation: request.generation,
                node_path: first_node_path.clone(),
            };
            let requested = targets.requested.unwrap_or_else(|| {
                request
                    .work_items
                    .iter()
                    .map(|item| item.node_path.clone())
                    .collect()
            });
            let task = AnalysisTaskDto {
                task_id,
                run_id: request.run_id.clone(),
                job_id: job_id.clone(),
                generation: request.generation,
                scope,
                strategy,
                stage: if strategy == AnalysisTaskStrategyDto::SingleStage {
                    AnalysisTaskStageDto::SingleStage
                } else {
                    AnalysisTaskStageDto::Overview
                },
                conditions,
                overview_conditions,
                requested,
                supporting: targets.supporting,
                swing_comparisons: targets.swing_comparisons,
                swing_criteria: targets.swing_criteria,
                selected_for_deep: None,
                overview_completed: Vec::new(),
                completed: Vec::new(),
                overview_summaries: Vec::new(),
                ending_conditions: Vec::new(),
                state: AnalysisTaskStateDto::Queued,
                reason: None,
            };
            let expected = request.work_items.len();
            self.yield_preloads_locked(&mut state);
            state.jobs.push(RegisteredJob {
                ordinary_context: None,
                job_id: job_id.clone(),
                query_id,
                run_id: request.run_id.clone(),
                lane: AnalysisJobLane::WholeGame,
                mode: AnalysisJobModeDto::Finite,
                continuous_budget: None,
                state: AnalysisJobStateDto::Queued,
                cancel_deadline: None,
                cleanup_deadline: None,
                submitted: false,
                submitted_at: Instant::now(),
                generation: request.generation,
                node_path: first_node_path,
                board_width: first_board_width,
                board_height: first_board_height,
                cancel: Arc::new(cancel),
                disposition: JobDisposition::Running,
                expected: Some(expected),
                work_items: request.work_items,
                current_index: 0,
                pending_ending_conditions: Vec::new(),
                terminal: false,
            });
            state.analysis_task = Some(task.clone());
            if let Some(id) = &batch_identity {
                batch::bind(&mut state, id, &task);
            }
            if auto_intent.is_some() {
                state.auto_load.snapshot.task_id = Some(task.task_id.clone());
                state.auto_load.snapshot.phase = app_model::AutoLoadPhaseDto::Running;
                state.auto_load.handback_pending = true;
            }
            publish_event(
                &mut state,
                ForegroundEngineEventDto::Job {
                    job: whole_game_job_event(
                        &started,
                        AnalysisJobOutcomeDto::Started,
                        started.node_path.clone(),
                        Some(0),
                        Some(expected),
                        Some(expected),
                        None,
                        None,
                    ),
                },
            );
            publish_snapshot(&mut state);
            (task, bound_query)
        };
        self.dispatch_whole_game_query(task.run_id.clone(), task.job_id.clone(), bound_query);
        Ok(task)
    }

    pub fn pause_analysis_task(
        &self,
        run_id: &str,
        task_id: &str,
    ) -> Result<AnalysisTaskDto, EngineFailureDto> {
        let mut state = self.lock();
        match_reservation::require_unreserved(&state)?;
        batch::admit(&state, None)?;
        if state.auto_load.snapshot.task_id.as_deref() == Some(task_id)
            && state.auto_load.snapshot.run_id.as_deref() == Some(run_id)
        {
            auto_load::retire_locked(self, &mut state)?;
            publish_snapshot(&mut state);
            return state
                .analysis_task
                .clone()
                .ok_or_else(|| continuous_invalid_state(&state, "Automatic task retired."));
        }
        let task = state
            .analysis_task
            .as_ref()
            .filter(|task| task.run_id == run_id && task.task_id == task_id)
            .ok_or_else(|| {
                continuous_invalid_state(&state, "Analysis task identity is no longer current.")
            })?;
        if matches!(
            task.state,
            AnalysisTaskStateDto::Pausing | AnalysisTaskStateDto::Paused
        ) {
            return Ok(task.clone());
        }
        if !matches!(
            task.state,
            AnalysisTaskStateDto::Queued | AnalysisTaskStateDto::Searching
        ) {
            return Err(continuous_invalid_state(
                &state,
                "Pause requires an active analysis task.",
            ));
        }
        let job_id = task.job_id.clone();
        self.request_job_stop_locked(&mut state, run_id, &job_id, JobDisposition::Paused)?;
        Ok(state.analysis_task.as_ref().unwrap().clone())
    }

    pub fn continue_analysis_task(
        &self,
        run_id: &str,
        task_id: &str,
        generation: u64,
    ) -> Result<AnalysisTaskDto, EngineFailureDto> {
        if self.measured_reservation(){return Err(invalid_task_conditions(run_id,"A leased measurement scene owns this Run; explicitly Stop before Continue".into()));}
        let (task, jsonl) = {
            let mut state = self.lock();
            match_reservation::require_unreserved(&state)?;
            batch::admit(&state, None)?;
            let task = state
                .analysis_task
                .as_ref()
                .filter(|task| {
                    task.run_id == run_id
                        && task.task_id == task_id
                        && task.generation == generation
                        && task.state == AnalysisTaskStateDto::Paused
                })
                .ok_or_else(|| {
                    continuous_invalid_state(&state, "Continue requires the same paused task and game.")
                })?;
            if admitting_run(&state.phase, run_id).is_none()
                || state.continuous_departing
                || state.continuous_safety_hold
            {
                return Err(continuous_invalid_state(
                    &state,
                    "Continue is blocked by Run or departure state.",
                ));
            }
            let run = admitting_run(&state.phase, run_id).expect("Ready Run checked above");
            validate_whole_game_capabilities(&run, task.swing_criteria.as_ref())?;
            validate_task_conditions_capabilities(&run, &task.conditions)?;
            if let Some(overview) = task.overview_conditions.as_ref() {
                validate_task_conditions_capabilities(&run, overview)?;
            }
            let old_job_id = task.job_id.clone();
            let index = state
                .jobs
                .iter()
                .position(|job| job.job_id == old_job_id && job.run_id == run_id && job.terminal)
                .ok_or_else(|| continuous_invalid_state(&state, "Paused task cleanup has not finished."))?;
            for item in &state.jobs[index].work_items[state.jobs[index].current_index..] {
                validate_query_capabilities(&run, &item.query, false)?;
            }
            evaluation::yield_to_foreground(&mut state)?;
            self.yield_preloads_locked(&mut state);
            let job_id = Uuid::new_v4().to_string();
            let query_id = target_query_id(&job_id);
            let job = &mut state.jobs[index];
            let jsonl = bound_work_item_query(&job.work_items[job.current_index], &query_id, &job_id)?;
            job.job_id = job_id.clone();
            job.query_id = query_id;
            job.state = AnalysisJobStateDto::Queued;
            job.submitted = false;
            job.submitted_at = Instant::now();
            job.cancel_deadline = None;
            job.cleanup_deadline = None;
            job.cancel = Arc::new(AnalysisCancelToken::new());
            job.disposition = JobDisposition::Running;
            job.pending_ending_conditions.clear();
            job.terminal = false;
            let started = started_from(job);
            let counts = whole_game_progress_counts(job);
            let task = state.analysis_task.as_mut().unwrap();
            task.job_id = job_id;
            task.state = AnalysisTaskStateDto::Queued;
            task.reason = None;
            let task = task.clone();
            publish_event(
                &mut state,
                ForegroundEngineEventDto::Job {
                    job: whole_game_job_event(
                        &started,
                        AnalysisJobOutcomeDto::Started,
                        started.node_path.clone(),
                        counts.0,
                        counts.1,
                        counts.2,
                        None,
                        None,
                    ),
                },
            );
            publish_snapshot(&mut state);
            (task, jsonl)
        };
        self.dispatch_whole_game_query(task.run_id.clone(), task.job_id.clone(), jsonl);
        Ok(task)
    }

    fn dispatch_whole_game_query(&self, run_id: String, job_id: String, jsonl: String) {
        let manager = self.clone();
        // Neither the current-game holder nor the stdout deadline pump waits for pipe IO.
        thread::spawn(move || {
            if let Err(error) = manager.submit_whole_game_query(&run_id, &job_id, &jsonl) {
                manager.abandon_job(&run_id, &job_id, error.with_job_id(&job_id));
            }
        });
    }

    fn submit_whole_game_query(
        &self,
        run_id: &str,
        job_id: &str,
        jsonl: &str,
    ) -> Result<(), EngineFailureDto> {
        loop {
            let mut state = self.lock();
            let Some(index) = state.jobs.iter().position(|job| {
                job.run_id == run_id
                    && job.job_id == job_id
                    && !job.terminal
                    && job.disposition == JobDisposition::Running
                    && !job.submitted
            }) else {
                return Ok(());
            };
            let stdin = engine_stdin(&state, run_id)
                .ok_or_else(|| continuous_invalid_state(&state, "Run process is not available."))?;
            match stdin.try_lock() {
                Ok(mut guard) => {
                    let writer = guard
                        .as_mut()
                        .ok_or_else(|| continuous_invalid_state(&state, "Run stdin is closed."))?;
                    // Commit delivery atomically with cancellation admission. The stdin guard
                    // orders this query before its terminate; blocking IO holds no owner lock.
                    state.jobs[index].submitted = true;
                    drop(state);
                    return write_jsonl(writer, jsonl).map_err(|error| {
                        failure(
                            EngineOperationDto::Job,
                            EngineFailureKind::Protocol,
                            self.inner
                                .diagnostic_text(run_id, &error.to_string())
                                .prefixed("failed to write analysis query: "),
                            Some(run_id),
                            None,
                            None,
                        )
                    });
                }
                Err(std::sync::TryLockError::Poisoned(_)) => {
                    return Err(continuous_invalid_state(&state, "Run stdin lock is poisoned."));
                }
                Err(std::sync::TryLockError::WouldBlock) => {}
            }
            drop(state);
            thread::sleep(Duration::from_millis(1));
        }
    }

    pub fn invalidate_analysis_task(&self, generation: u64) {
        let mut state = self.lock();
        if state.batch.current.is_some() {
            return;
        }
        if state.auto_load.snapshot.generation != generation {
            let _ = auto_load::retire_locked(self, &mut state);
        }
        let Some(task) = state.analysis_task.as_ref() else {
            return;
        };
        if task.generation == generation
            || matches!(
                task.state,
                AnalysisTaskStateDto::Cancelled | AnalysisTaskStateDto::Invalidated
            )
        {
            return;
        }
        let run_id = task.run_id.clone();
        let job_id = task.job_id.clone();
        invalidate_analysis_task_locked(&mut state, "The current game changed; start a new analysis task.");
        if state
            .jobs
            .iter()
            .any(|job| job.run_id == run_id && job.job_id == job_id && !job.terminal)
        {
            let _ = self.request_job_stop_locked(&mut state, &run_id, &job_id, JobDisposition::Cancelled);
        } else {
            publish_snapshot(&mut state);
        }
    }
}

impl Inner {
    pub(super) fn route_whole_game_line(
        &self,
        state: &mut ManagerState,
        run_id: &str,
        response_id: String,
        trimmed: &str,
    ) -> Option<WholeGameAdvance> {
        let job_index = state.jobs.iter().position(|job| {
            job.query_id == response_id
                && job.run_id == run_id
                && !job.terminal
                && matches!(
                    job.disposition,
                    JobDisposition::Running | JobDisposition::BudgetReached
                )
                && job.lane == AnalysisJobLane::WholeGame
        })?;
        let started = started_from(&state.jobs[job_index]);
        let current_index = state.jobs[job_index].current_index;
        let item = state.jobs[job_index].work_items.get(current_index)?.clone();
        let expected = state.jobs[job_index].expected?;
        let conditions = state
            .analysis_task
            .as_ref()
            .filter(|task| task.job_id == started.job_id)
            .and_then(active_task_conditions)
            .cloned()?;
        match parse_response_line(trimmed) {
            Ok(response) if response.id == response_id => {
                if response.action.is_some() {
                    return None;
                }
                let during = response.is_during_search == Some(true);
                let has_search_data = response.has_valid_search_result(item.board_width, item.board_height);
                if has_search_data {
                    state.last_activity = Instant::now();
                    if state.jobs[job_index].disposition == JobDisposition::Running {
                        state.jobs[job_index].state = AnalysisJobStateDto::Searching;
                        mark_analysis_task_searching(state, &started.job_id);
                        if during && response.root_info.as_ref().is_some_and(|root|root.visits>0) && current_admitting_run(&state.phase).is_some_and(|run|run.measured_scene==Some(app_model::MeasuredSceneDto::WholeGame)) {
                            // Legal in-flight data proves response-probe activity only, never a completed/adopted visit budget.
                            self.measured_note_search_progress(run_id,&started.job_id,started.generation);
                        }
                    }
                }

                let observed = if has_search_data {
                    response.observed_task_ending_conditions(&conditions, !during)
                } else {
                    Vec::new()
                };
                if during {
                    if !observed.is_empty() && state.jobs[job_index].disposition == JobDisposition::Running {
                        let job = &mut state.jobs[job_index];
                        job.cancel.cancel();
                        job.disposition = JobDisposition::BudgetReached;
                        job.state = AnalysisJobStateDto::Stopping;
                        job.cancel_deadline = Some(Instant::now() + Duration::from_secs(5));
                        job.pending_ending_conditions = observed.clone();
                        if let Some(task) = state.analysis_task.as_mut() {
                            task.reason =
                                Some(format!("Stopping after reaching {}.", observed.join(" and ")));
                        }
                        let counts = whole_game_progress_counts(job);
                        publish_event(
                            state,
                            ForegroundEngineEventDto::Job {
                                job: whole_game_job_event(
                                    &started,
                                    AnalysisJobOutcomeDto::Stopping,
                                    item.node_path,
                                    counts.0,
                                    counts.1,
                                    counts.2,
                                    None,
                                    None,
                                ),
                            },
                        );
                        publish_snapshot(state);
                        return Some(WholeGameAdvance::Terminate {
                            run_id: started.run_id,
                            job_id: started.job_id,
                            query_id: state.jobs[job_index].query_id.clone(),
                        });
                    }
                    publish_snapshot(state);
                    return None;
                }
                let ending_conditions = if state.jobs[job_index].pending_ending_conditions.is_empty() {
                    observed
                } else {
                    state.jobs[job_index].pending_ending_conditions.clone()
                };
                if !has_search_data || ending_conditions.is_empty() {
                    let published = failure(
                        EngineOperationDto::Job,
                        EngineFailureKind::Protocol,
                        "whole-game search ended without valid analysis reaching an enabled condition".into(),
                        Some(run_id),
                        None,
                        None,
                    )
                    .with_job_id(&started.job_id);
                    finish_failed_job(state, &started, published);
                    publish_snapshot(state);
                    return None;
                }

                let job_uuid = Uuid::parse_str(&started.job_id).unwrap_or_else(|_| Uuid::nil());
                let mut frame = normalize_response(job_uuid, response, item.board_width, item.board_height);
                frame.turn = item.move_number;
                let requires_root_score = state.analysis_task.as_ref().is_some_and(|task| {
                    task.job_id == started.job_id
                        && task.stage == AnalysisTaskStageDto::Overview
                        && task
                            .swing_criteria
                            .as_ref()
                            .is_some_and(|criteria| criteria.score_change_points.enabled)
                });
                if requires_root_score && frame.score_mean_black.is_none() {
                    let published = failure(
                        EngineOperationDto::Job,
                        EngineFailureKind::Protocol,
                        "swing overview result omitted the required root score".into(),
                        Some(run_id),
                        None,
                        None,
                    )
                    .with_job_id(&started.job_id);
                    finish_failed_job(state, &started, published);
                    publish_snapshot(state);
                    return None;
                }
                let completed = current_index + 1;
                let remaining = expected.saturating_sub(completed);
                state.jobs[job_index].current_index = completed;
                state.jobs[job_index].pending_ending_conditions.clear();
                state.jobs[job_index].cancel_deadline = None;
                state.jobs[job_index].disposition = JobDisposition::Running;
                record_analysis_task_completion(
                    state,
                    &started.job_id,
                    &item.node_path,
                    &frame,
                    ending_conditions,
                );
                publish_event(
                    state,
                    ForegroundEngineEventDto::Job {
                        job: whole_game_job_event(
                            &started,
                            AnalysisJobOutcomeDto::Progress,
                            item.node_path.clone(),
                            Some(completed),
                            Some(expected),
                            Some(remaining),
                            Some(frame),
                            None,
                        ),
                    },
                );
                if completed >= expected {
                    let begins_deep = state.analysis_task.as_ref().is_some_and(|task| {
                        task.job_id == started.job_id
                            && task.strategy != AnalysisTaskStrategyDto::SingleStage
                            && task.stage == AnalysisTaskStageDto::Overview
                    });
                    if !begins_deep {
                        finish_whole_game_job(
                            state,
                            &started,
                            AnalysisJobOutcomeDto::Completed,
                            Some(completed),
                            Some(expected),
                            Some(0),
                            None,
                        );
                        publish_snapshot(state);
                        return None;
                    }

                    let (deep_conditions, selected, selection_reason) = {
                        let task = state.analysis_task.as_ref().expect("two-stage task");
                        let (selected, reason) =
                            if task.strategy == AnalysisTaskStrategyDto::SwingSelectedTwoStage {
                                let selected = select_swing_deep_paths(task);
                                let reason = selected.is_empty().then(|| swing_zero_selection_reason(task));
                                (selected, reason)
                            } else {
                                (task.requested.clone(), None)
                            };
                        (task.conditions.clone(), selected, reason)
                    };
                    let selected_work_items = selected
                        .iter()
                        .filter_map(|path| {
                            state.jobs[job_index]
                                .work_items
                                .iter()
                                .find(|item| item.node_path == *path)
                                .cloned()
                        })
                        .collect::<Vec<_>>();
                    if selected_work_items.len() != selected.len() {
                        let published = failure(
                            EngineOperationDto::Job,
                            EngineFailureKind::InvalidState,
                            "Frozen swing selection did not match the admitted overview positions.".into(),
                            Some(run_id),
                            None,
                            None,
                        )
                        .with_job_id(&started.job_id);
                        finish_failed_job(state, &started, published);
                        publish_snapshot(state);
                        return None;
                    }
                    {
                        let task = state.analysis_task.as_mut().expect("two-stage task");
                        task.selected_for_deep = Some(selected.clone());
                        task.stage = AnalysisTaskStageDto::Deep;
                        task.state = AnalysisTaskStateDto::Queued;
                        task.reason = selection_reason;
                        task.ending_conditions.clear();
                    }
                    {
                        let job = &mut state.jobs[job_index];
                        job.work_items = selected_work_items;
                        job.expected = Some(selected.len());
                        job.current_index = 0;
                        for work_item in &mut job.work_items {
                            work_item.query.task(&deep_conditions);
                        }
                    }
                    if selected.is_empty() {
                        finish_whole_game_job(
                            state,
                            &started,
                            AnalysisJobOutcomeDto::Completed,
                            Some(0),
                            Some(0),
                            Some(0),
                            None,
                        );
                        publish_snapshot(state);
                        return None;
                    }
                }

                let next_index = state.jobs[job_index].current_index;
                let expected = state.jobs[job_index].expected.unwrap_or_default();
                let next = state.jobs[job_index].work_items[next_index].clone();
                let query_id = target_query_id(&started.job_id);
                let jsonl = match bound_work_item_query(&next, &query_id, &started.job_id) {
                    Ok(jsonl) => jsonl,
                    Err(error) => {
                        finish_whole_game_job(
                            state,
                            &started,
                            AnalysisJobOutcomeDto::Failed,
                            Some(next_index),
                            Some(expected),
                            Some(expected.saturating_sub(next_index)),
                            Some(error),
                        );
                        publish_snapshot(state);
                        return None;
                    }
                };
                state.jobs[job_index].query_id = query_id;
                state.jobs[job_index].node_path = next.node_path;
                state.jobs[job_index].state = AnalysisJobStateDto::Queued;
                state.jobs[job_index].submitted = false;
                state.jobs[job_index].cancel = Arc::new(AnalysisCancelToken::new());
                state.jobs[job_index].submitted_at = Instant::now();
                if let Some(task) = state.analysis_task.as_mut() {
                    task.state = AnalysisTaskStateDto::Queued;
                    task.reason = None;
                }
                publish_snapshot(state);
                Some(WholeGameAdvance::Submit { started, jsonl })
            }
            Ok(_) => None,
            Err(error) if trimmed.contains(&response_id) => {
                let completed = current_index;
                let remaining = expected.saturating_sub(completed);
                finish_whole_game_job(
                    state,
                    &started,
                    AnalysisJobOutcomeDto::Failed,
                    Some(completed),
                    Some(expected),
                    Some(remaining),
                    Some(
                        failure(
                            EngineOperationDto::Job,
                            EngineFailureKind::Protocol,
                            self.diagnostic_text(&started.run_id, &error.to_string())
                                .prefixed("whole-game response was not parseable: "),
                            Some(started.run_id.as_str()),
                            None,
                            None,
                        )
                        .with_job_id(&started.job_id),
                    ),
                );
                publish_snapshot(state);
                None
            }
            Err(_) => None,
        }
    }

    pub(super) fn advance_whole_game(self: &Arc<Self>, pending: WholeGameAdvance) {
        let manager = ForegroundEngineManager {
            inner: Arc::clone(self),
        };
        match pending {
            WholeGameAdvance::Submit { started, jsonl } => {
                manager.dispatch_whole_game_query(started.run_id, started.job_id, jsonl);
            }
            WholeGameAdvance::Terminate {
                run_id,
                job_id,
                query_id,
            } => {
                thread::spawn(move || manager.write_terminate_after_submission(&run_id, &job_id, &query_id));
            }
        }
    }
}

pub(super) fn whole_game_progress_counts(
    job: &RegisteredJob,
) -> (Option<usize>, Option<usize>, Option<usize>) {
    let expected = job.expected;
    let completed = Some(job.current_index);
    let remaining = expected.map(|expected| expected.saturating_sub(job.current_index));
    (completed, expected, remaining)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn whole_game_job_event(
    started: &AnalysisJobStartedDto,
    outcome: AnalysisJobOutcomeDto,
    node_path: NodePath,
    completed: Option<usize>,
    expected: Option<usize>,
    remaining: Option<usize>,
    frame: Option<app_model::AnalysisFrameDto>,
    failure: Option<EngineFailureDto>,
) -> AnalysisJobEventDto {
    AnalysisJobEventDto { local_focus_output: None, run_id: started.run_id.clone(),
    job_id: started.job_id.clone(),
    lane: started.lane,
    mode: started.mode,
    generation: started.generation,
    node_path,
    outcome,
    completed,
    expected,
    remaining,
    frame,
    failure,
    current_game: None, }
}

fn target_query_id(job_id: &str) -> String {
    format!("{job_id}:{}", Uuid::new_v4())
}

fn bound_work_item_query(
    item: &WholeGameWorkItem,
    query_id: &str,
    job_id: &str,
) -> Result<String, EngineFailureDto> {
    let mut query = item.query.clone();
    query.id = query_id.to_string();
    query.report_during_search_every = Some(0.1);
    query.to_jsonl().map_err(|error| {
        failure(
            EngineOperationDto::Job,
            EngineFailureKind::Protocol,
            FailureText::unscoped(&error.to_string()).prefixed("failed to serialize whole-game query: "),
            None,
            None,
            None,
        )
        .with_job_id(job_id)
    })
}

pub(super) fn finish_whole_game_job(
    state: &mut ManagerState,
    started: &AnalysisJobStartedDto,
    outcome: AnalysisJobOutcomeDto,
    completed: Option<usize>,
    expected: Option<usize>,
    remaining: Option<usize>,
    failure: Option<EngineFailureDto>,
) {
    let owned = state
        .jobs
        .iter()
        .any(|job| job.job_id == started.job_id && job.run_id == started.run_id);
    if !owned {
        return;
    }
    let node_path = state
        .jobs
        .iter()
        .find(|job| job.job_id == started.job_id && job.run_id == started.run_id)
        .map(|job| {
            if job.work_items.is_empty() {
                return job.node_path.clone();
            }
            let index = job.current_index.min(job.work_items.len() - 1);
            job.work_items
                .get(index)
                .map(|item| item.node_path.clone())
                .unwrap_or_else(|| job.node_path.clone())
        })
        .unwrap_or_else(|| started.node_path.clone());
    finish_analysis_task_locked(state, &started.job_id, outcome, failure.as_ref());
    if state
        .analysis_task
        .as_ref()
        .is_some_and(|task| task.job_id == started.job_id && task.state == AnalysisTaskStateDto::Paused)
    {
        let job = state
            .jobs
            .iter_mut()
            .find(|job| job.job_id == started.job_id)
            .unwrap();
        job.terminal = true;
        job.cancel_deadline = None;
    } else {
        state.jobs.retain(|job| job.job_id != started.job_id);
    }
    publish_event(
        state,
        ForegroundEngineEventDto::Job {
            job: whole_game_job_event(
                started, outcome, node_path, completed, expected, remaining, None, failure,
            ),
        },
    );
}

fn mark_analysis_task_searching(state: &mut ManagerState, job_id: &str) {
    if let Some(task) = state
        .analysis_task
        .as_mut()
        .filter(|task| task.job_id == job_id && task.state == AnalysisTaskStateDto::Queued)
    {
        task.state = AnalysisTaskStateDto::Searching;
    }
}

fn active_task_conditions(task: &AnalysisTaskDto) -> Option<&AnalysisStageConditionsDto> {
    match task.stage {
        AnalysisTaskStageDto::Overview => task.overview_conditions.as_ref(),
        AnalysisTaskStageDto::SingleStage | AnalysisTaskStageDto::Deep => Some(&task.conditions),
    }
}

fn select_swing_deep_paths(task: &AnalysisTaskDto) -> Vec<NodePath> {
    let criteria = task.swing_criteria.as_ref().expect("swing task criteria");
    let mut selected = Vec::new();
    for comparison in &task.swing_comparisons {
        if !criteria.move_actors.admits(comparison.move_actor) {
            continue;
        }
        let Some(before) = task
            .overview_summaries
            .iter()
            .find(|summary| summary.node_path == comparison.before)
        else {
            continue;
        };
        let Some(after) = task
            .overview_summaries
            .iter()
            .find(|summary| summary.node_path == comparison.after)
        else {
            continue;
        };
        let player_winrate = |winrate_black: f32| match comparison.move_actor {
            PlayerColor::Black => winrate_black,
            PlayerColor::White => 1.0 - winrate_black,
        };
        let winrate_change =
            (player_winrate(after.frame.winrate_black) - player_winrate(before.frame.winrate_black)).abs()
                * 100.0;
        let mut qualifies = criteria.winrate_change_percentage_points.enabled
            && meets_swing_threshold(winrate_change, criteria.winrate_change_percentage_points.value);
        if criteria.score_change_points.enabled {
            let (before_score, after_score) = (
                before
                    .frame
                    .score_mean_black
                    .expect("validated overview root score"),
                after
                    .frame
                    .score_mean_black
                    .expect("validated overview root score"),
            );
            let player_score = |score_black: f32| match comparison.move_actor {
                PlayerColor::Black => score_black,
                PlayerColor::White => -score_black,
            };
            qualifies |= meets_swing_threshold(
                (player_score(after_score) - player_score(before_score)).abs(),
                criteria.score_change_points.value,
            );
        }
        if qualifies {
            for path in [&comparison.before, &comparison.after] {
                if !selected.contains(path) {
                    selected.push(path.clone());
                }
            }
        }
    }
    selected
}

fn meets_swing_threshold(observed: f32, threshold: f32) -> bool {
    observed >= threshold || (observed - threshold).abs() <= f32::EPSILON * threshold.abs().max(1.0) * 4.0
}

fn swing_zero_selection_reason(task: &AnalysisTaskDto) -> String {
    if task.swing_comparisons.is_empty() {
        return "Overview completed; no played-move predecessor comparisons were available in the requested scope."
            .into();
    }
    "Overview completed; no moves met the swing thresholds.".into()
}

fn record_analysis_task_completion(
    state: &mut ManagerState,
    job_id: &str,
    node_path: &NodePath,
    frame: &app_model::AnalysisFrameDto,
    ending_conditions: Vec<String>,
) {
    if let Some(task) = state.analysis_task.as_mut().filter(|task| {
        task.job_id == job_id
            && matches!(
                task.state,
                AnalysisTaskStateDto::Queued | AnalysisTaskStateDto::Searching
            )
    }) {
        let admitted = if task.stage == AnalysisTaskStageDto::Overview {
            task.requested.contains(node_path) || task.supporting.contains(node_path)
        } else {
            task.selected_for_deep
                .as_ref()
                .unwrap_or(&task.requested)
                .contains(node_path)
        };
        if !admitted {
            return;
        }
        if task.stage == AnalysisTaskStageDto::Overview {
            if !task.overview_completed.contains(node_path) {
                task.overview_completed.push(node_path.clone());
                task.overview_summaries.push(AnalysisTaskOverviewDto {
                    node_path: node_path.clone(),
                    frame: frame.clone(),
                });
            }
        } else if !task.completed.contains(node_path) {
            task.completed.push(node_path.clone());
        }
        task.ending_conditions = ending_conditions;
        task.reason = None;
    }
}

fn finish_analysis_task_locked(
    state: &mut ManagerState,
    job_id: &str,
    outcome: AnalysisJobOutcomeDto,
    failure: Option<&EngineFailureDto>,
) {
    let Some(task) = state.analysis_task.as_mut().filter(|task| task.job_id == job_id) else {
        return;
    };
    if matches!(
        task.state,
        AnalysisTaskStateDto::Cancelled | AnalysisTaskStateDto::Failed | AnalysisTaskStateDto::Invalidated
    ) {
        return;
    }
    match outcome {
        AnalysisJobOutcomeDto::Completed
            if task.completed.len() == task.selected_for_deep.as_ref().unwrap_or(&task.requested).len() =>
        {
            task.state = AnalysisTaskStateDto::Completed;
        }
        AnalysisJobOutcomeDto::Cancelled if task.state == AnalysisTaskStateDto::Pausing => {
            task.state = AnalysisTaskStateDto::Paused;
            task.reason = None;
        }
        AnalysisJobOutcomeDto::Cancelled => {
            task.state = AnalysisTaskStateDto::Cancelled;
            task.reason = Some("Analysis task was cancelled.".into());
        }
        AnalysisJobOutcomeDto::Completed => {
            task.state = AnalysisTaskStateDto::Failed;
            task.reason =
                Some("Analysis task ended before every requested position produced a result.".into());
        }
        AnalysisJobOutcomeDto::Failed | AnalysisJobOutcomeDto::Timeout => {
            task.state = AnalysisTaskStateDto::Failed;
            task.reason = Some(
                failure
                    .map(|failure| failure.message.clone())
                    .unwrap_or_else(|| "Analysis task failed.".into()),
            );
        }
        AnalysisJobOutcomeDto::Superseded => {
            task.state = AnalysisTaskStateDto::Invalidated;
            task.reason = Some("Analysis task was superseded.".into());
        }
        _ => {}
    }
}

pub(super) fn cancel_analysis_task_locked(state: &mut ManagerState, job_id: &str) {
    if let Some(task) = state.analysis_task.as_mut().filter(|task| {
        task.job_id == job_id
            && matches!(
                task.state,
                AnalysisTaskStateDto::Queued
                    | AnalysisTaskStateDto::Searching
                    | AnalysisTaskStateDto::Pausing
                    | AnalysisTaskStateDto::Paused
            )
    }) {
        task.state = AnalysisTaskStateDto::Cancelled;
        task.reason = Some("Analysis task was cancelled.".into());
    }
}

pub(super) fn invalidate_analysis_task_locked(state: &mut ManagerState, reason: &str) {
    if let Some(task) = state.analysis_task.as_mut().filter(|task| {
        matches!(
            task.state,
            AnalysisTaskStateDto::Queued
                | AnalysisTaskStateDto::Searching
                | AnalysisTaskStateDto::Pausing
                | AnalysisTaskStateDto::Paused
                | AnalysisTaskStateDto::Completed
                | AnalysisTaskStateDto::Failed
        )
    }) {
        task.state = AnalysisTaskStateDto::Invalidated;
        task.reason = Some(reason.into());
        state
            .jobs
            .retain(|job| !(job.job_id == task.job_id && job.terminal));
    }
}

pub(super) fn fail_analysis_task_locked(state: &mut ManagerState, job_id: Option<&str>, reason: &str) {
    if let Some(task) = state.analysis_task.as_mut().filter(|task| {
        job_id.is_none_or(|job_id| task.job_id == job_id)
            && matches!(
                task.state,
                AnalysisTaskStateDto::Queued
                    | AnalysisTaskStateDto::Searching
                    | AnalysisTaskStateDto::Pausing
                    | AnalysisTaskStateDto::Paused
            )
    }) {
        task.state = AnalysisTaskStateDto::Failed;
        task.reason = Some(reason.into());
        state
            .jobs
            .retain(|job| !(job.job_id == task.job_id && job.terminal));
    }
}

pub(super) fn validate_whole_game_capabilities(
    run: &EngineRunDto,
    swing_criteria: Option<&AnalysisSwingCriteriaDto>,
) -> Result<(), EngineFailureDto> {
    let capabilities = analysis_capabilities(run)?;
    require_capability(run, capabilities.whole_game_analysis, "whole-game analysis")?;
    if let Some(criteria) = swing_criteria {
        if criteria.score_change_points.enabled {
            require_capability(run, capabilities.root_score, "root-score comparisons")?;
        }
        if criteria.winrate_change_percentage_points.enabled {
            require_capability(run, capabilities.winrate, "winrate comparisons")?;
        }
    }
    Ok(())
}

fn validate_task_conditions_capabilities(
    run: &EngineRunDto,
    conditions: &AnalysisStageConditionsDto,
) -> Result<(), EngineFailureDto> {
    let capabilities = analysis_capabilities(run)?;
    if conditions.total_visits.enabled || conditions.leading_candidate_visits.enabled {
        require_capability(run, capabilities.visits_limit, "visits limits")?;
    }
    if conditions.leading_candidate_visits.enabled {
        require_capability(run, capabilities.candidates, "candidate visits")?;
    }
    Ok(())
}

fn empty_whole_game_failure(run_id: &str) -> EngineFailureDto {
    failure(
        EngineOperationDto::Job,
        EngineFailureKind::InvalidState,
        "whole-game analysis requires a captured first-child mainline worklist".into(),
        Some(run_id),
        None,
        None,
    )
}

fn invalid_task_conditions(run_id: &str, message: String) -> EngineFailureDto {
    failure(
        EngineOperationDto::Job,
        EngineFailureKind::InvalidState,
        FailureText::unscoped(&message),
        Some(run_id),
        None,
        None,
    )
}

fn single_stage_conditions(total_visits: u32) -> AnalysisStageConditionsDto {
    AnalysisStageConditionsDto {
        time_seconds: AnalysisTaskLimitDto {
            enabled: false,
            value: 10,
        },
        total_visits: AnalysisTaskLimitDto {
            enabled: true,
            value: total_visits,
        },
        leading_candidate_visits: AnalysisTaskLimitDto {
            enabled: false,
            value: 500,
        },
    }
}

#[cfg(test)]
mod swing_selection_tests {
    use super::*;
    use app_model::{AnalysisMoveActorFilterDto, AnalysisSwingThresholdDto};

    fn path(indices: &[u32]) -> NodePath {
        NodePath {
            indices: indices.to_vec(),
        }
    }

    fn summary(
        indices: &[u32],
        winrate_black: f32,
        score_mean_black: Option<f32>,
    ) -> AnalysisTaskOverviewDto {
        AnalysisTaskOverviewDto {
            node_path: path(indices),
            frame: app_model::AnalysisFrameDto {
                job_id: Uuid::nil(),
                game_id: None,
                node_id: None,
                turn: indices.len() as u32,
                visits: 32,
                winrate_black,
                score_mean_black,
                score_stdev: None,
                candidates: Vec::new(),
                ownership: None,
                policy: None,
            },
        }
    }

    fn task(criteria: AnalysisSwingCriteriaDto) -> AnalysisTaskDto {
        let requested = vec![path(&[0]), path(&[0, 0])];
        AnalysisTaskDto {
            task_id: "task".into(),
            run_id: "run".into(),
            job_id: "job".into(),
            generation: 1,
            scope: AnalysisScopeDto {
                mode: AnalysisScopeModeDto::FirstChildMainline,
                current_node: NodePath::default(),
                branch_choices: Vec::new(),
                interval: None,
                to_play: None,
            },
            strategy: AnalysisTaskStrategyDto::SwingSelectedTwoStage,
            stage: AnalysisTaskStageDto::Overview,
            conditions: AnalysisStageConditionsDto::default(),
            overview_conditions: Some(AnalysisStageConditionsDto::default()),
            requested: requested.clone(),
            supporting: vec![NodePath::default()],
            swing_comparisons: vec![
                AnalysisSwingComparisonDto {
                    before: NodePath::default(),
                    after: requested[0].clone(),
                    move_actor: PlayerColor::Black,
                },
                AnalysisSwingComparisonDto {
                    before: requested[0].clone(),
                    after: requested[1].clone(),
                    move_actor: PlayerColor::White,
                },
            ],
            swing_criteria: Some(criteria),
            selected_for_deep: None,
            overview_completed: vec![NodePath::default(), requested[0].clone(), requested[1].clone()],
            completed: Vec::new(),
            overview_summaries: vec![
                summary(&[], 0.5, Some(0.0)),
                summary(&[0], 0.4, Some(5.0)),
                summary(&[0, 0], 0.5, Some(4.0)),
            ],
            state: AnalysisTaskStateDto::Searching,
            reason: None,
            ending_conditions: Vec::new(),
        }
    }

    #[test]
    fn swing_predicates_cover_equality_both_directions_score_only_and_or() {
        let criteria = AnalysisSwingCriteriaDto {
            move_actors: AnalysisMoveActorFilterDto::Both,
            winrate_change_percentage_points: AnalysisSwingThresholdDto {
                enabled: true,
                value: 10.0,
            },
            score_change_points: AnalysisSwingThresholdDto {
                enabled: false,
                value: 5.0,
            },
        };
        let winrate_only = select_swing_deep_paths(&task(criteria.clone()));
        assert_eq!(winrate_only, vec![path(&[]), path(&[0]), path(&[0, 0])]);

        let mut score_only = criteria.clone();
        score_only.winrate_change_percentage_points.enabled = false;
        score_only.score_change_points.enabled = true;
        let selected = select_swing_deep_paths(&task(score_only));
        assert_eq!(selected, vec![path(&[]), path(&[0])]);

        let mut either = criteria;
        either.winrate_change_percentage_points.value = 100.0;
        either.score_change_points.enabled = true;
        let selected = select_swing_deep_paths(&task(either));
        assert_eq!(selected, vec![path(&[]), path(&[0])]);

        let mut unformable = task(AnalysisSwingCriteriaDto::default());
        unformable.swing_comparisons.clear();
        assert!(swing_zero_selection_reason(&unformable)
            .contains("no played-move predecessor comparisons were available"));
    }
}
