use crate::{continuous_analysis::PreferencesState, current_game_state::CurrentGameState};
use app_model::{
    CurrentGameError, DocumentDepartureActionDto, DocumentDepartureOutcomeDto, ExternalSyncSnapshotDto,
    ExternalSyncSourceDto, ExternalSyncStartDto, ExternalSyncUpdateDto, NodePath, YikeSyncPreferencesDto,
};
use provider_yike::sync::{YikeSyncHost, YikeSyncRuntime, YikeSyncWork};
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_opener::OpenerExt;

pub(crate) const EVENT: &str = "external-sync-updated";
struct Host(AppHandle);
impl YikeSyncHost for Host {
    fn next(&self) -> YikeSyncWork {
        let current = self.0.state::<CurrentGameState>();
        let preferences = self.0.state::<PreferencesState>();
        let work = current.next_external_poll(preferences.network());
        if !matches!(work, YikeSyncWork::Wait(Some(_))) {
            publish(&self.0, None);
        }
        work
    }
    fn completed(
        &self,
        identity: app_model::ProviderRequestIdentityDto,
        result: Result<app_model::ProviderImportResult, app_model::ProviderError>,
    ) {
        let current = self.0.state::<CurrentGameState>();
        let policy = self
            .0
            .state::<PreferencesState>()
            .network()
            .snapshot()
            .policy_revision;
        let update = current.complete_external_poll(identity, result, policy);
        let _ = self.0.emit(EVENT, update);
    }
}
pub fn start(app: &AppHandle) -> Result<(), String> {
    app.state::<YikeSyncRuntime>().start(Host(app.clone()))
}
pub fn publish(app: &AppHandle, current: Option<app_model::CurrentGameResultDto>) {
    let sync = app.state::<CurrentGameState>().external_sync_snapshot();
    let _ = app.emit(EVENT, ExternalSyncUpdateDto { sync, current });
}
pub fn wake(app: &AppHandle) {
    app.state::<YikeSyncRuntime>().wake();
}

#[tauri::command]
pub fn external_sync_snapshot(current: State<CurrentGameState>) -> ExternalSyncSnapshotDto {
    current.external_sync_snapshot()
}

#[tauri::command]
pub fn load_yike_sync_preferences(
    preferences: State<PreferencesState>,
) -> Result<YikeSyncPreferencesDto, String> {
    preferences.yike_sync_preferences()
}

#[tauri::command]
pub fn save_yike_sync_preferences(
    app: AppHandle,
    preferences: State<PreferencesState>,
    current: State<CurrentGameState>,
    interval_seconds: u32,
    locator: Option<String>,
    jump_to_last: bool,
    mute: bool,
) -> Result<YikeSyncPreferencesDto, String> {
    let saved = preferences.save_yike_sync_preferences(
        &crate::app_preferences_path(&app)?,
        YikeSyncPreferencesDto {
            interval_seconds,
            locator,
            jump_to_last,
            mute,
        },
    )?;
    current.update_external_preferences(saved.clone());
    wake(&app);
    publish(&app, None);
    Ok(saved)
}

#[tauri::command]
pub fn begin_yike_sync(
    app: AppHandle,
    preferences: State<PreferencesState>,
    current: State<CurrentGameState>,
    locator: String,
) -> Result<u64, String> {
    let locator = provider_yike::canonical_yike_locator(&locator).map_err(|error| error.message)?;
    let mut settings = preferences.yike_sync_preferences()?;
    settings.locator = Some(locator.clone());
    let id = current
        .begin_external_start(preferences.network(), locator, settings)
        .map_err(|error| error.message)?;
    publish(&app, None);
    Ok(id)
}

#[tauri::command]
pub async fn prepare_yike_sync(app: AppHandle, start_id: u64) -> Result<ExternalSyncStartDto, String> {
    let result = tauri::async_runtime::spawn_blocking({
        let app = app.clone();
        move || {
            let current = app.state::<CurrentGameState>();
            let (operation, locator) = current
                .external_start_request(start_id)
                .map_err(|error| error.message)?;
            let result =
                provider_yike::fetch_public_preview(&operation, &locator).map_err(|error| error.message)?;
            current
                .prepare_external_candidate(start_id, result)
                .map_err(|error| error.message)
        }
    })
    .await
    .map_err(|_| "Yike source preparation worker failed.".to_string())
    .and_then(|result| result);
    if result.is_err() {
        app.state::<CurrentGameState>().cancel_external_start(start_id);
    }
    publish(&app, None);
    result
}

#[tauri::command]
pub fn cancel_external_sync_start(
    app: AppHandle,
    current: State<CurrentGameState>,
    start_id: u64,
) -> ExternalSyncSnapshotDto {
    let snapshot = current.cancel_external_start(start_id);
    wake(&app);
    publish(&app, None);
    snapshot
}

/// Commits a Yike or readboard start through the single SGF-07 replacement transaction.
#[tauri::command]
pub async fn resolve_external_sync_start(
    app: AppHandle,
    current: State<'_, CurrentGameState>,
    manager: State<'_, engine_manager::ForegroundEngineManager>,
    departure_id: u64,
    action: DocumentDepartureActionDto,
    selected_path: NodePath,
    default_file_name: Option<String>,
) -> Result<DocumentDepartureOutcomeDto, CurrentGameError> {
    let mut outcome = crate::document_departure::resolve_document_replacement(
        app.clone(),
        current,
        manager,
        departure_id,
        action,
        selected_path,
        default_file_name,
    )
    .await?;
    if outcome.committed {
        let snapshot = app.state::<CurrentGameState>().external_sync_snapshot();
        if snapshot.source == Some(ExternalSyncSourceDto::Yike) {
            let preferences = app.state::<PreferencesState>();
            let saved = crate::app_preferences_path(&app)
                .and_then(|path| preferences.save_yike_sync_preferences(&path, snapshot.preferences));
            if let Err(error) = saved {
                outcome.message = format!("Sync started; preferences were not saved: {error}");
            }
        }
        wake(&app);
    }
    publish(&app, outcome.current.clone());
    Ok(outcome)
}

/// Yike polls again; readboard restarts its runtime and resumes only on a newer Ready connection.
#[tauri::command]
pub async fn retry_external_sync(
    app: AppHandle,
    session_id: u64,
) -> Result<ExternalSyncSnapshotDto, CurrentGameError> {
    let snapshot = app.state::<CurrentGameState>().retry_external_sync(session_id)?;
    publish(&app, None);
    if snapshot.source == Some(ExternalSyncSourceDto::Readboard) {
        if let Err(message) = crate::readboard::restart_for_retry(&app).await {
            app.state::<CurrentGameState>()
                .readboard_retry_failed(session_id, message);
        }
    } else {
        wake(&app);
    }
    let snapshot = app.state::<CurrentGameState>().external_sync_snapshot();
    publish(&app, None);
    Ok(snapshot)
}

#[tauri::command]
pub fn stop_external_sync(
    app: AppHandle,
    current: State<CurrentGameState>,
    session_id: u64,
) -> Result<ExternalSyncUpdateDto, CurrentGameError> {
    let update = current.stop_external_sync(session_id)?;
    wake(&app);
    let _ = app.emit(EVENT, &update);
    Ok(update)
}

#[tauri::command]
pub fn open_yike_sync_browser(
    app: AppHandle,
    current: State<CurrentGameState>,
    session_id: u64,
) -> Result<ExternalSyncSnapshotDto, String> {
    let locator = current
        .external_browser_locator(session_id)
        .map_err(|error| error.message)?;
    let opened = app.opener().open_url(locator, None::<String>);
    let error = opened.err().map(|_| {
        "Could not open the system browser. Read-only sync remains active; retry opening the browser."
            .to_string()
    });
    let snapshot = current.external_browser_result(session_id, error);
    publish(&app, None);
    Ok(snapshot)
}
