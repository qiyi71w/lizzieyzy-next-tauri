pub mod application;
use crate::{engine_profile_path, load_engine_profiles_from_disk, ENGINE_CATALOG_WRITES};
use engine_manager::{EngineProfilesSettings, ForegroundEngineManager};
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn measured_report_review(
    manager: State<'_, ForegroundEngineManager>,
    target_id: String,
    report: String,
) -> Result<app_model::MeasuredReviewDto, String> {
    let manager = manager.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        manager.review_measured_report(&target_id, report.as_bytes())
    })
    .await
    .map_err(|_| "Report worker failed".to_owned())?
}
#[tauri::command]
pub async fn measured_report_apply(
    app: AppHandle,
    manager: State<'_, ForegroundEngineManager>,
    operation_id: String,
    generation: u64,
) -> Result<EngineProfilesSettings, String> {
    let manager = manager.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let (lease, review) = manager.validate_measured_apply(&operation_id, generation)?;
        let _transaction = ENGINE_CATALOG_WRITES
            .lock()
            .map_err(|_| "Engine catalog lock poisoned")?;
        let mut latest = load_engine_profiles_from_disk(&app)?;
        engine_manager::apply_measured_to_catalog(&mut latest, &review, &lease)?;
        let path = engine_profile_path(&app)?;
        let saved = lease.commit(|| engine_manager::save_engine_profiles(&path, latest))?;
        manager.invalidate_preload(&review.target_id);
        Ok(saved)
    })
    .await
    .map_err(|_| "Report Apply worker failed".to_owned())?
}
#[tauri::command]
pub async fn measured_report_restore(
    app: AppHandle,
    manager: State<'_, ForegroundEngineManager>,
    target_id: String,
    revision: u64,
) -> Result<EngineProfilesSettings, String> {
    let manager = manager.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let lease = manager.begin_setup_operation()?;
        let _transaction = ENGINE_CATALOG_WRITES
            .lock()
            .map_err(|_| "Engine catalog lock poisoned")?;
        let mut latest = load_engine_profiles_from_disk(&app)?;
        engine_manager::restore_measured_catalog(&mut latest, &target_id, revision, &lease)?;
        let path = engine_profile_path(&app)?;
        let saved = lease.commit(|| engine_manager::save_engine_profiles(&path, latest))?;
        manager.invalidate_preload(&target_id);
        Ok(saved)
    })
    .await
    .map_err(|_| "Report Restore worker failed".to_owned())?
}
#[tauri::command]
pub fn measured_report_retire(manager: State<'_, ForegroundEngineManager>) -> Result<(), String> {
    manager.retire_setup_operation()
}
#[tauri::command]
pub fn measured_report_busy(manager: State<'_, ForegroundEngineManager>) -> Result<bool, String> {
    manager.setup_operation_status()
}

#[tauri::command]
pub fn measured_scene_preview(
    current: State<'_, crate::CurrentGameState>,
    generation: u64,
    scope: app_model::AnalysisScopeDto,
) -> Result<app_model::AnalysisScopePreviewDto, String> {
    current
        .preview_analysis_scope(generation, scope, None)
        .map_err(|e| e.message)
}
#[tauri::command]
pub async fn measured_scene_start(
    app: AppHandle,
    manager: State<'_, ForegroundEngineManager>,
    target_id: String,
    preview: app_model::AnalysisScopePreviewDto,
    budget: u32,
) -> Result<app_model::AnalysisTaskDto, String> {
    use tauri::Manager;
    let manager = manager.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        if budget == 0 || budget > i32::MAX as u32 {
            return Err("Scene budget must be a positive i32".into());
        }
        let lease = manager.begin_setup_operation()?;
        let current = app.state::<crate::CurrentGameState>();
        if current
            .preview_analysis_scope(preview.generation, preview.scope.clone(), None)
            .map_err(|e| e.message)?
            != preview
        {
            return Err("Scene scope changed; preview again".into());
        }
        let run = manager.start_independent_measured_scene(
            &target_id,
            app_model::MeasuredSceneDto::WholeGame,
            &lease,
        )?;
        let conditions = app_model::AnalysisStageConditionsDto {
            time_seconds: app_model::AnalysisTaskLimitDto {
                enabled: false,
                value: 1,
            },
            total_visits: app_model::AnalysisTaskLimitDto {
                enabled: true,
                value: budget,
            },
            leading_candidate_visits: app_model::AnalysisTaskLimitDto {
                enabled: false,
                value: 1,
            },
        };
        let task = match current.start_measured_analysis_task(
            &manager,
            run.run_id.clone(),
            preview,
            conditions,
            &lease,
        ) {
            Ok(task) => task,
            Err(error) => {
                manager.stop_measured_scene(&run.run_id)?;
                return Err(error.to_string());
            }
        };
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(900);
        let outcome = loop {
            if let Err(error) = lease.check() {
                break Err(error);
            }
            if std::time::Instant::now() >= deadline {
                break Err("Independent scene completion timed out".into());
            }
            let Some(state) = manager
                .analysis_task_snapshot()
                .filter(|state| state.task_id == task.task_id)
            else {
                break Err("Independent scene Task was replaced".into());
            };
            match state.state {
                // This user-started Task preserves existing document retention policy. Only
                // measurement rows reset fixtures and require adopted incoming visit evidence.
                app_model::AnalysisTaskStateDto::Completed => break Ok(state),
                app_model::AnalysisTaskStateDto::Failed
                | app_model::AnalysisTaskStateDto::Cancelled
                | app_model::AnalysisTaskStateDto::Invalidated => {
                    break Err(format!("Independent scene ended as {:?}", state.state))
                }
                _ => {}
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        };
        // A terminal response is published only after the sole process owner confirms actual reap.
        manager.stop_measured_scene(&run.run_id)?;
        outcome
    })
    .await
    .map_err(|_| "Independent scene worker failed".to_owned())?
}
