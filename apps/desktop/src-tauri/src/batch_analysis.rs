use crate::continuous_analysis::PreferencesState;
use app_model::{BatchCommandDto, BatchPreferencesDto, BatchSnapshotDto};
use engine_manager::ForegroundEngineManager;
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

#[tauri::command]
pub fn batch_snapshot(manager: State<ForegroundEngineManager>) -> BatchSnapshotDto {
    manager.batch_snapshot()
}

#[tauri::command]
pub async fn batch_command(
    manager: State<'_, ForegroundEngineManager>,
    command: BatchCommandDto,
) -> Result<BatchSnapshotDto, String> {
    let manager = manager.inner().clone();
    tauri::async_runtime::spawn_blocking(move || manager.batch_command(command))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub fn load_batch_preferences(
    app: AppHandle,
    preferences: State<PreferencesState>,
    manager: State<ForegroundEngineManager>,
) -> Result<BatchPreferencesDto, String> {
    Ok(preferences
        .load(&crate::app_preferences_path(&app)?, &manager)?
        .preferences
        .batch)
}

#[tauri::command]
pub fn save_batch_preferences(
    app: AppHandle,
    preferences: State<PreferencesState>,
    batch: BatchPreferencesDto,
) -> Result<BatchPreferencesDto, String> {
    preferences.save_batch(&crate::app_preferences_path(&app)?, batch)
}

#[tauri::command]
pub async fn pick_batch_files(app: AppHandle) -> Result<Vec<String>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        app.dialog()
            .file()
            .add_filter("SGF", &["sgf"])
            .blocking_pick_files()
            .unwrap_or_default()
            .into_iter()
            .map(|p| {
                p.into_path()
                    .map(|p| p.to_string_lossy().into_owned())
                    .map_err(|e| e.to_string())
            })
            .collect()
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn pick_batch_directory(app: AppHandle) -> Result<Option<String>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        app.dialog()
            .file()
            .blocking_pick_folder()
            .map(|p| {
                p.into_path()
                    .map(|p| p.to_string_lossy().into_owned())
                    .map_err(|e| e.to_string())
            })
            .transpose()
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn pick_batch_save(app: AppHandle, name: String) -> Result<Option<String>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        app.dialog()
            .file()
            .add_filter("SGF", &["sgf"])
            .set_file_name(name)
            .blocking_save_file()
            .map(|p| {
                p.into_path()
                    .map(|p| p.to_string_lossy().into_owned())
                    .map_err(|e| e.to_string())
            })
            .transpose()
    })
    .await
    .map_err(|e| e.to_string())?
}
