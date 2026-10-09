use app_model::{ManagedAcquireRequestDto, ManagedResourcesDto};
use engine_manager::managed::ManagedResources;
use tauri::{AppHandle, Manager};

fn revalidate(app: &AppHandle, request: &ManagedAcquireRequestDto) -> Result<(), String> {
    let settings = super::load_engine_profiles_from_disk(app).map_err(|_| "managed_profile_unavailable")?;
    if settings.selected_profile_id != request.profile_id
        || !settings.profiles.iter().any(|record| record.id == request.profile_id && record.profile == request.profile) {
        return Err("managed_target_revision_changed".into());
    }
    Ok(())
}
#[tauri::command]
pub fn managed_resources_snapshot(app: AppHandle) -> ManagedResourcesDto {
    app.state::<ManagedResources>().snapshot()
}
#[tauri::command]
pub async fn acquire_managed_resources(app: AppHandle, request: ManagedAcquireRequestDto) -> Result<String, String> {
    let id = {
        let _transaction = super::ENGINE_CATALOG_WRITES.lock().map_err(|_| "managed_profile_unavailable")?;
        revalidate(&app, &request)?;
        app.state::<ManagedResources>().begin(request, app.state::<super::continuous_analysis::PreferencesState>().network())?
    };
    let worker_id = id.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let resources = app.state::<ManagedResources>();
        let result = (|| {
            let prepared = resources.prepare(&worker_id)?;
            let _transaction = super::ENGINE_CATALOG_WRITES.lock().map_err(|_| "managed_profile_unavailable")?;
            revalidate(&app, &resources.request(&worker_id)?)?;
            resources.publish(prepared, &app.state::<engine_manager::models::ModelInventory>())?;
            Ok::<(), String>(())
        })();
        if let Err(error) = result { resources.fail(&worker_id, &error); }
    });
    Ok(id)
}
#[tauri::command]
pub async fn cancel_managed_resources(app: AppHandle, operation_id: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || app.state::<ManagedResources>().cancel(&operation_id))
        .await.map_err(|_| "managed_worker_failed".to_string())?
}
