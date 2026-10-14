use app_model::*;
use engine_manager::measured_report::{
    fingerprint::semantic_equal,
    measurement::{self, MemorySampler},
};
use engine_manager::{ForegroundEngineManager, SetupLease};
use serde_json::{json, Value};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager, State};

#[tauri::command]
pub fn measurement_progress(
    manager: State<'_, ForegroundEngineManager>,
) -> Option<ApplicationMeasurementProgressDto> {
    manager.measurement_progress()
}
#[tauri::command]
pub fn measurement_probe(
    manager: State<'_, ForegroundEngineManager>,
    probe: ApplicationMeasurementProbeDto,
) -> Result<(), String> {
    manager.measurement_probe(probe)
}
#[tauri::command]
pub async fn measure_application(
    app: AppHandle,
    manager: State<'_, ForegroundEngineManager>,
    request: ApplicationMeasurementRequestDto,
) -> Result<ApplicationMeasurementResultDto, String> {
    let manager = manager.inner().clone();
    tauri::async_runtime::spawn_blocking(move || execute(&app, &manager, request))
        .await
        .map_err(|_| "Application measurement worker failed".to_string())?
}

fn reset_fixture(current: &crate::CurrentGameState, fixture: &str) -> Result<CurrentGameResultDto, String> {
    let admission = current
        .prepare_replacement(fixture, None)
        .map_err(|e| e.to_string())?;
    let id = match admission {
        DocumentDepartureAdmissionDto::Ready { departure_id }
        | DocumentDepartureAdmissionDto::NeedsDecision { departure_id } => departure_id,
    };
    current
        .begin_protected_commit(id, &[])
        .map_err(|e| e.to_string())?;
    current
        .commit_replacement(id)
        .map_err(|e| e.to_string())?
        .current
        .ok_or("Fixture document was not committed".into())
}
fn scope(
    current: &crate::CurrentGameState,
    game: &CurrentGameResultDto,
) -> Result<AnalysisScopePreviewDto, String> {
    current
        .preview_analysis_scope(
            game.generation,
            AnalysisScopeDto {
                mode: AnalysisScopeModeDto::FirstChildMainline,
                current_node: NodePath::default(),
                branch_choices: Vec::new(),
                interval: None,
                to_play: None,
            },
            None,
        )
        .map_err(|e| e.to_string())
}
fn execute(
    app: &AppHandle,
    manager: &ForegroundEngineManager,
    request: ApplicationMeasurementRequestDto,
) -> Result<ApplicationMeasurementResultDto, String> {
    measurement::validate_request(&request)?;
    let lease = manager.begin_setup_operation()?;
    // Qualification is cold and independent of any row completion stopwatch.
    let capture = Instant::now();
    let (fingerprint, baseline) = manager.measurement_inputs(&request.target_id, request.scene, &lease)?;
    if baseline == request.candidate_parameters {
        return Err("Candidate must differ from the effective baseline".into());
    }
    let mut cold = vec![json!({"qualificationSeconds":capture.elapsed().as_secs_f64()})];
    let current = app.state::<crate::CurrentGameState>();
    let mut rows = Vec::new();
    let mut positions = None;
    let mut gpu_uuid = None;
    for round in 1..=request.pairs {
        for candidate in if round % 2 == 1 {
            [false, true]
        } else {
            [true, false]
        } {
            lease.check()?;
            let verify = Instant::now();
            let (fresh, policy) = manager.measurement_inputs(&request.target_id, request.scene, &lease)?;
            if !semantic_equal(&fingerprint, &fresh) || policy != baseline {
                return Err("Measurement resources or effective baseline changed between rows".into());
            }
            lease.begin_row(
                &request.target_id,
                request.scene,
                round,
                candidate,
                if candidate {
                    request.candidate_parameters.clone()
                } else {
                    baseline.clone()
                },
            )?;
            let sampler = MemorySampler::start(manager)?;
            let start = Instant::now();
            let run = manager.start_independent_measured_scene(&request.target_id, request.scene, &lease)?;
            let observation: Result<(Value, f64), String> = (|| {
                // Every independent process warms on the actual application path, then a new
                // document identity prevents retained SGF results from satisfying the timed row.
                let warm = reset_fixture(&current, &request.fixture_sgf)?;
                exercise(
                    &current,
                    manager,
                    &lease,
                    &run,
                    &warm,
                    request.budget,
                    false,
                    &request.fixture_sgf,
                )?;
                let named_backend=run.qualified_resource.as_ref().and_then(|resource|resource.backend.as_deref()).filter(|backend|matches!(*backend,"CUDA"|"TensorRT"|"OpenCL")).ok_or("Owned application launch did not prove a named GPU backend; CPU receipts cannot qualify")?;
                cold.push(json!({"round":round,"candidate":candidate,"backend":named_backend}));
                let game = reset_fixture(&current, &request.fixture_sgf)?;
                let count = if request.scene == MeasuredSceneDto::Live {
                    1
                } else {
                    scope(&current, &game)?.targets.len()
                };
                if positions.is_some_and(|old| old != count) {
                    return Err("Fixture scope changed between rows".into());
                }
                positions = Some(count);
                let warmup_seconds = start.elapsed().as_secs_f64();
                lease.begin_sampling()?;
                let row = exercise(
                    &current,
                    manager,
                    &lease,
                    &run,
                    &game,
                    request.budget,
                    true,
                    &request.fixture_sgf,
                )?;
                Ok((row, warmup_seconds))
            })();
            // On success, error, or cancellation the same manager owns Stop and actual reap.
            let reaping = manager.stop_measured_scene(&run.run_id);
            let webview = if observation.is_ok() && reaping.is_ok() {
                lease.finish_sampling()
            } else {
                Err("Application row did not complete its sampling window".into())
            };
            let memory = sampler.finish();
            reaping?;
            let (mut row, warmup_seconds) = observation?;
            let memory = memory?;
            let gpu = &memory.identity;
            if gpu.name != fingerprint["gpuName"]
                || gpu.driver != fingerprint["driverVersion"]
                || gpu.physical_mib != fingerprint["memoryMiB"].as_f64().ok_or("Missing GPU memory")?
                || gpu_uuid.as_ref().is_some_and(|uuid| uuid != &gpu.uuid)
            {
                return Err("GPU capture differs from qualified fingerprint or previous row UUID".into());
            }
            gpu_uuid = Some(gpu.uuid.clone());
            if memory.max_engine_processes != 1 {
                return Err("Lifecycle sampler did not prove exactly one application engine".into());
            }
            row["profile"] = json!(if candidate { "candidate" } else { "baseline" });
            row["round"] = json!(round);
            row["phase"] = json!("warm");
            row["maxMemoryMiB"] = json!(memory.max_memory_mib);
            row["maxEngineProcesses"] = json!(memory.max_engine_processes);
            cold.push(json!({"round":round,"candidate":candidate,"gpu":memory,"samplingPeriodSeconds":0.2}));
            let webview = webview?;
            row["edtP95Seconds"] = webview["p95Seconds"].clone();
            cold.push(json!({"round":round,"candidate":candidate,"webview":webview}));
            cold.push(json!({"round":round,"candidate":candidate,"verificationSeconds":verify.elapsed().as_secs_f64()-start.elapsed().as_secs_f64(),"startupAndWarmupSeconds":warmup_seconds}));
            rows.push(row);
        }
    }
    measurement::assemble(
        &request,
        fingerprint,
        baseline,
        positions.ok_or("No measured positions")?,
        rows,
        cold,
    )
}

fn exercise(
    current: &crate::CurrentGameState,
    manager: &ForegroundEngineManager,
    lease: &SetupLease,
    run: &EngineRunDto,
    game: &CurrentGameResultDto,
    budget: u32,
    timed: bool,
    fixture: &str,
) -> Result<Value, String> {
    manager.measured_begin_observation(lease)?;
    let started = Instant::now();
    let deadline = started + Duration::from_secs(900);
    let scene = run.measured_scene.ok_or("Measurement scene missing")?;
    let (job_id, paths, task_id) = dispatch(current, manager, lease, run, game, budget)?;
    let mut processes = 0;
    let adopted = loop {
        lease.check()?;
        if matches!(
            manager.snapshot().lifecycle,
            ForegroundEngineLifecycleDto::Error { .. } | ForegroundEngineLifecycleDto::NoEngine { .. }
        ) {
            return Err("Measurement engine exited or failed before completion".into());
        }
        if Instant::now() >= deadline {
            return Err("Application measurement completion timed out".into());
        }
        processes = processes.max(manager.measured_process_count());
        if processes != 1 {
            return Err("Application measurement requires exactly one owned engine process".into());
        }
        let adopted = manager.measured_adopted(&run.run_id, &job_id);
        let budget_met = paths.iter().all(|path| {
            adopted
                .iter()
                .any(|(actual, visits)| path == actual && *visits >= budget as u64)
        });
        let completed = if let Some(id) = &task_id {
            let task = manager
                .analysis_task_snapshot()
                .filter(|task| &task.task_id == id)
                .ok_or("Measurement Task was replaced")?;
            if matches!(
                task.state,
                AnalysisTaskStateDto::Failed
                    | AnalysisTaskStateDto::Cancelled
                    | AnalysisTaskStateDto::Invalidated
            ) {
                return Err("Measurement Task failed or retired".into());
            }
            task.state == AnalysisTaskStateDto::Completed
        } else {
            true
        };
        if budget_met && completed {
            break adopted;
        }
        std::thread::sleep(Duration::from_millis(5));
    };
    let seconds = started.elapsed().as_secs_f64();
    let first = manager.measured_first_result(started);
    if scene == MeasuredSceneDto::Live {
        pause_live(current, manager, lease, run, &job_id, deadline)?;
    }
    let mut response_seconds = 0.0;
    if timed {
        let probe_game = reset_fixture(current, fixture)?;
        manager.measured_begin_observation(lease)?;
        let (probe_id, _, probe_task) = dispatch(current, manager, lease, run, &probe_game, i32::MAX as u32)?;
        loop {
            lease.check()?;
            if Instant::now() >= deadline {
                return Err("Independent response probe did not produce active search".into());
            }
            let snapshot = manager.snapshot();
            if matches!(
                snapshot.lifecycle,
                ForegroundEngineLifecycleDto::Error { .. } | ForegroundEngineLifecycleDto::NoEngine { .. }
            ) {
                return Err("Measurement engine exited or failed before response probe".into());
            }
            let searching = if let Some(task_id) = &probe_task {
                manager.analysis_task_snapshot().is_some_and(|task| {
                    &task.task_id == task_id && task.state == AnalysisTaskStateDto::Searching
                })
            } else {
                snapshot
                    .selected_node_job
                    .as_ref()
                    .is_some_and(|job| job.job_id == probe_id && job.state == AnalysisJobStateDto::Searching)
            };
            let observed = if scene == MeasuredSceneDto::WholeGame {
                manager.measured_search_progress(&run.run_id, &probe_id, probe_game.generation)
            } else {
                manager
                    .measured_adopted(&run.run_id, &probe_id)
                    .iter()
                    .any(|(_, visits)| *visits > 0)
            };
            if searching && observed {
                break;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        let response = Instant::now();
        if scene == MeasuredSceneDto::Live {
            pause_live(current, manager, lease, run, &probe_id, deadline)?;
        } else {
            manager.stop_measured_scene(&run.run_id)?;
        }
        let finished = Instant::now();
        response_seconds = finished.duration_since(response).as_secs_f64();
        lease.mark_sample_window(started, finished)?;
    }
    let mut row =
        json!({"seconds":seconds,"responseSeconds":response_seconds,"maxEngineProcesses":processes});
    if scene == MeasuredSceneDto::Live {
        row["firstResultSeconds"] = json!(first.ok_or("No actual current-game first result")?);
        row["observedRootVisits"] = json!(adopted[0].1);
    } else {
        row["rootVisitsByTurn"] = json!(paths
            .iter()
            .map(|path| adopted
                .iter()
                .find(|(actual, _)| actual == path)
                .expect("budget proven")
                .1)
            .collect::<Vec<_>>());
    }
    Ok(row)
}

fn dispatch(
    current: &crate::CurrentGameState,
    manager: &ForegroundEngineManager,
    lease: &SetupLease,
    run: &EngineRunDto,
    game: &CurrentGameResultDto,
    budget: u32,
) -> Result<(String, Vec<NodePath>, Option<String>), String> {
    if run.measured_scene == Some(MeasuredSceneDto::Live) {
        let request = crate::bind_selected_node_job(
            current,
            run.run_id.clone(),
            game.generation,
            game.selected_path.clone(),
            AnalysisJobModeDto::Continuous,
            None,
        )
        .map_err(|e| e.to_string())?;
        let job = manager
            .start_measured_selected_node_job(request, lease)
            .map_err(|e| e.to_string())?;
        Ok((job.job_id, vec![game.selected_path.clone()], None))
    } else {
        let preview = scope(current, game)?;
        let conditions = AnalysisStageConditionsDto {
            time_seconds: AnalysisTaskLimitDto {
                enabled: false,
                value: 1,
            },
            total_visits: AnalysisTaskLimitDto {
                enabled: true,
                value: budget,
            },
            leading_candidate_visits: AnalysisTaskLimitDto {
                enabled: false,
                value: 1,
            },
        };
        let task = current
            .start_measured_analysis_task(manager, run.run_id.clone(), preview, conditions, lease)
            .map_err(|e| e.to_string())?;
        Ok((task.job_id, task.requested, Some(task.task_id)))
    }
}
fn pause_live(
    current: &crate::CurrentGameState,
    manager: &ForegroundEngineManager,
    lease: &SetupLease,
    run: &EngineRunDto,
    job_id: &str,
    deadline: Instant,
) -> Result<(), String> {
    crate::cancel_document_analysis_job(current, manager, &run.run_id, job_id).map_err(|e| e.to_string())?;
    loop {
        lease.check()?;
        let snapshot = manager.snapshot();
        if matches!(snapshot.lifecycle, ForegroundEngineLifecycleDto::Error { .. }) {
            return Err("Live Pause failed".into());
        }
        if snapshot
            .selected_node_job
            .as_ref()
            .is_none_or(|job| job.job_id != job_id)
        {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err("Live Pause dual fence timed out".into());
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}

#[cfg(test)]
#[path = "application_tests.rs"]
mod tests;
