use app_model::{EngineAdapterSettings, ModelInventoryDto, ModelPathDto, ModelSelectionRequestDto};
use engine_manager::models::ModelInventory;
use tauri::{AppHandle, Manager};

pub fn saved_paths(settings: &engine_manager::EngineProfilesSettings) -> Vec<ModelPathDto> {
    settings
        .profiles
        .iter()
        .filter_map(|record| match &record.profile.adapter {
            EngineAdapterSettings::KataGoAnalysis(settings) | EngineAdapterSettings::KataGoGtp(settings) => {
                settings
                    .model_path
                    .as_ref()
                    .filter(|path| !path.is_empty())
                    .map(|path| ModelPathDto {
                        path: path.clone(),
                        working_dir: record.profile.working_dir.clone(),
                    })
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_katago_protocols_retain_saved_models_without_changing_profiles() {
        let directory = std::env::temp_dir().join(format!("gtp-model-retention-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&directory).unwrap();
        let settings: engine_manager::EngineProfilesSettings = serde_json::from_value(serde_json::json!({
            "version": engine_manager::ENGINE_PROFILES_VERSION,
            "startup": app_model::EngineStartupPolicyDto::Off,
            "selected_profile_id": "gtp",
            "profiles": (["kata_go_analysis", "kata_go_gtp"].map(|adapter| serde_json::json!({
                "id": if adapter == "kata_go_gtp" { "gtp" } else { "jsonl" },
                "profile": { "name": adapter, "program": "katago", "argv": [],
                    "working_dir": directory.to_string_lossy(), "adapter_kind": adapter,
                    "settings": { "model_path": format!("{adapter}.bin.gz"), "config_path": null, "max_visits": 4 }
                }
            })))
        })).unwrap();
        let before = serde_json::to_vec(&settings).unwrap();
        let inventory = ModelInventory::new(directory.join("inventory.json"));
        inventory.remember_saved(&saved_paths(&settings)).unwrap();
        let retained = inventory.snapshot().unwrap();
        assert_eq!(retained.models.len(), 2);
        assert!(retained
            .models
            .iter()
            .any(|model| model.path.ends_with("kata_go_gtp.bin.gz")));
        assert!(retained
            .models
            .iter()
            .any(|model| model.path.ends_with("kata_go_analysis.bin.gz")));
        assert_eq!(serde_json::to_vec(&settings).unwrap(), before);
        let reopened = ModelInventory::new(directory.join("inventory.json"))
            .snapshot()
            .unwrap();
        assert_eq!(reopened.models, retained.models);
        std::fs::remove_dir_all(directory).unwrap();
    }
}
