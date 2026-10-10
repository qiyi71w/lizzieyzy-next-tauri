use app_model::{ManagedAcquireRequestDto, ManagedRepairPreviewDto, ManagedResourcesDto};
use engine_manager::managed::ManagedResources;
use tauri::{AppHandle, Manager};

fn revalidate(app: &AppHandle, request: &ManagedAcquireRequestDto) -> Result<(), String> {
    let settings = super::load_engine_profiles_from_disk(app).map_err(|_| "managed_profile_unavailable")?;
    if settings.selected_profile_id != request.profile_id
        || !settings
            .profiles
            .iter()
            .any(|record| record.id == request.profile_id && record.profile == request.profile)
    {
        return Err("managed_target_revision_changed".into());
    }
    Ok(())
}
#[tauri::command]
pub fn managed_resources_snapshot(app: AppHandle) -> ManagedResourcesDto {
    app.state::<ManagedResources>().snapshot()
}
#[tauri::command]
pub async fn acquire_managed_resources(
    app: AppHandle,
    request: ManagedAcquireRequestDto,
) -> Result<String, String> {
    let id = {
        let _transaction = super::ENGINE_CATALOG_WRITES
            .lock()
            .map_err(|_| "managed_profile_unavailable")?;
        revalidate(&app, &request)?;
        app.state::<ManagedResources>().begin(
            request,
            app.state::<super::continuous_analysis::PreferencesState>()
                .network(),
        )?
    };
    run_operation(app, id.clone());
    Ok(id)
}
#[tauri::command]
pub async fn inspect_managed_trt_repair(
    app: AppHandle,
    request: ManagedAcquireRequestDto,
) -> Result<ManagedRepairPreviewDto, String> {
    tauri::async_runtime::spawn_blocking(move || {
        revalidate(&app, &request)?;
        let preview = app.state::<ManagedResources>().inspect_repair(request)?;
        let _transaction = super::ENGINE_CATALOG_WRITES
            .lock()
            .map_err(|_| "managed_profile_unavailable")?;
        revalidate(&app, &preview.request)?;
        Ok(preview)
    })
    .await
    .map_err(|_| "managed_worker_failed".to_string())?
}
#[tauri::command]
pub async fn repair_managed_trt(app: AppHandle, admission_id: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let request = app.state::<ManagedResources>().repair_request(&admission_id)?;
        revalidate(&app, &request)?;
        let id = app.state::<ManagedResources>().begin_repair(
            &admission_id,
            app.state::<super::continuous_analysis::PreferencesState>()
                .network(),
        )?;
        run_operation(app, id.clone());
        Ok(id)
    })
    .await
    .map_err(|_| "managed_worker_failed".to_string())?
}
#[tauri::command]
pub async fn managed_repair_draft(
    app: AppHandle,
    operation_id: String,
) -> Result<app_model::EngineProfileDto, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let resources = app.state::<ManagedResources>();
        let draft = resources.repair_draft(&operation_id)?;
        let _transaction = super::ENGINE_CATALOG_WRITES
            .lock()
            .map_err(|_| "managed_profile_unavailable")?;
        revalidate(&app, &resources.request(&operation_id)?)?;
        Ok(draft)
    })
    .await
    .map_err(|_| "managed_worker_failed".to_string())?
}
fn run_operation(app: AppHandle, worker_id: String) {
    tauri::async_runtime::spawn_blocking(move || {
        let resources = app.state::<ManagedResources>();
        let result = (|| {
            let prepared = resources.prepare(&worker_id)?;
            let _transaction = super::ENGINE_CATALOG_WRITES
                .lock()
                .map_err(|_| "managed_profile_unavailable")?;
            revalidate(&app, &resources.request(&worker_id)?)?;
            resources.publish(prepared, &app.state::<engine_manager::models::ModelInventory>())?;
            Ok::<(), String>(())
        })();
        if let Err(error) = result {
            resources.fail(&worker_id, &error);
        }
    });
}
#[tauri::command]
pub async fn cancel_managed_resources(app: AppHandle, operation_id: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || app.state::<ManagedResources>().cancel(&operation_id))
        .await
        .map_err(|_| "managed_worker_failed".to_string())?
}
