use crate::{continuous_analysis::PreferencesState, CurrentGameState};
use app_model::{AnalysisBranchChoiceDto, ContinuationPreferencesDto, ContinuationSnapshotDto, NodePath};
use engine_manager::ForegroundEngineManager;
use tauri::{AppHandle, Manager, State};

#[tauri::command]
pub fn continuation_snapshot(manager: State<ForegroundEngineManager>) -> ContinuationSnapshotDto {
    manager.continuation_snapshot()
}

#[tauri::command]
pub async fn start_engine_continuation(
    app: AppHandle,
    generation: u64,
    path: NodePath,
    choices: Vec<AnalysisBranchChoiceDto>,
) -> Result<ContinuationSnapshotDto, String> {
    let epoch = app.state::<ForegroundEngineManager>().continuation_epoch();
    tauri::async_runtime::spawn_blocking(move || {
        let manager = app.state::<ForegroundEngineManager>();
        let preferences = app
            .state::<PreferencesState>()
            .load(&crate::app_preferences_path(&app)?, &manager)?
            .preferences
            .continuation;
        app.state::<CurrentGameState>().start_continuation(
            &manager,
            generation,
            path,
            choices,
            preferences,
            epoch,
        )
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn engine_continuation_beat(
    app: AppHandle,
    session: String,
) -> Result<ContinuationSnapshotDto, String> {
    tauri::async_runtime::spawn_blocking(move || {
        app.state::<CurrentGameState>()
            .continuation_beat(&app.state::<ForegroundEngineManager>(), &session)
    })
    .await
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn stop_engine_continuation(
    current: State<CurrentGameState>,
    manager: State<ForegroundEngineManager>,
) -> ContinuationSnapshotDto {
    current.stop_continuation(&manager, "Stopped by user.")
}

#[tauri::command]
pub fn load_continuation_preferences(
    app: AppHandle,
    preferences: State<PreferencesState>,
    manager: State<ForegroundEngineManager>,
) -> Result<ContinuationPreferencesDto, String> {
    Ok(preferences
        .load(&crate::app_preferences_path(&app)?, &manager)?
        .preferences
        .continuation)
}

#[tauri::command]
pub fn save_continuation_preferences(
    app: AppHandle,
    preferences: State<PreferencesState>,
    value: ContinuationPreferencesDto,
) -> Result<ContinuationPreferencesDto, String> {
    preferences.save_continuation(&crate::app_preferences_path(&app)?, value)
}
