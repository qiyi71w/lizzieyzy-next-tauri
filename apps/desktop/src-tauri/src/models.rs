use app_model::{EngineAdapterSettings, ModelInventoryDto, ModelPathDto, ModelSelectionRequestDto};
use engine_manager::models::ModelInventory;
use tauri::{AppHandle, Manager};

pub fn saved_paths(settings: &engine_manager::EngineProfilesSettings) -> Vec<ModelPathDto> {
    settings
        .profiles
        .iter()
        .filter_map(|record| match &record.profile.adapter {
            EngineAdapterSettings::KataGoAnalysis(settings) => settings
                .model_path
                .as_ref()
                .filter(|path| !path.is_empty())
                .map(|path| ModelPathDto {
                    path: path.clone(),
                    working_dir: record.profile.working_dir.clone(),
                }),
            EngineAdapterSettings::GenericGtp(_) => None,
        })
        .collect()
}

#[tauri::command]
pub async fn model_inventory_snapshot(app: AppHandle) -> Result<ModelInventoryDto, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let settings = super::load_engine_profiles_from_disk(&app)?;
        let inventory = app.state::<ModelInventory>();
        inventory.remember_saved(&saved_paths(&settings))?;
        inventory.snapshot()
    })
    .await
    .map_err(|_| "model_inventory_worker_failed".to_string())?
}

#[tauri::command]
pub async fn refresh_model_inventory(
    app: AppHandle,
    paths: Vec<ModelPathDto>,
) -> Result<ModelInventoryDto, String> {
    tauri::async_runtime::spawn_blocking(move || app.state::<ModelInventory>().refresh(&paths))
        .await
        .map_err(|_| "model_inventory_worker_failed".to_string())?
}

#[tauri::command]
pub async fn select_installed_model(
    app: AppHandle,
    request: ModelSelectionRequestDto,
) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || app.state::<ModelInventory>().select(&request))
        .await
        .map_err(|_| "model_inventory_worker_failed".to_string())?
}
