use crate::{continuous_analysis::PreferencesState, current_game_state::CurrentGameState, external_sync};
use app_model::{ExternalSyncStartDto, ReadboardRuntimeDto, ReadboardSyncPreferencesDto};
use go_core::ReadboardControl;
use readboard_sidecar::{ReadboardInbound, ReadboardInboundEvent, ReadboardRuntime, ReadboardRuntimeEvent};
use std::{path::Path, time::Duration};
use tauri::{AppHandle, Emitter, Manager, State};

const STOP_BUDGET: Duration = Duration::from_secs(2);
/// A started session waits this long for the user to press sync in readboard.
const FIRST_FRAME_BUDGET: Duration = Duration::from_secs(300);
const SYNC_REQUESTED: &str = "readboard://sync-requested";

pub fn install(app: &AppHandle) {
    let runtime = ReadboardRuntime::default();
    let events = runtime.subscribe();
    app.manage(runtime);
    let handle = app.clone();
    // One consumer keeps the runtime's publication order: a connection's Ready reaches the
    // owner before its frames, and its sealing snapshot before anything of the next run.
    std::thread::spawn(move || {
        while let Ok(event) = events.recv() {
            let current = handle.state::<CurrentGameState>();
            match event {
                ReadboardRuntimeEvent::Lifecycle(snapshot) => {
                    if current.readboard_runtime_changed(&snapshot).is_some() {
                        external_sync::publish(&handle, None);
                    }
                    let _ = handle.emit("readboard://runtime", snapshot);
                }
                ReadboardRuntimeEvent::Inbound(ReadboardInboundEvent { generation, inbound }) => {
                    match inbound {
                        ReadboardInbound::Frame(frame) => {
                            if let Some(update) = current.observe_readboard_frame(generation, &frame) {
                                let _ = handle.emit(external_sync::EVENT, update);
                            }
                        }
                        ReadboardInbound::Control(control) => {
                            if current.readboard_control(generation, control).is_some() {
                                external_sync::publish(&handle, None);
                            } else if control == ReadboardControl::Sync {
                                let sync = current.external_sync_snapshot();
                                if sync.source != Some(app_model::ExternalSyncSourceDto::Readboard)
                                    && sync.starting_id.is_none()
                                {
                                    // An explicit tool Start may switch from Yike through the same SGF-07 decision.
                                    let _ = handle.emit(SYNC_REQUESTED, generation);
                                }
                            }
                        }
                        ReadboardInbound::Rejected(reason) => {
                            if current.readboard_frame_rejected(generation, &reason).is_some() {
                                external_sync::publish(&handle, None);
                            }
                        }
                    }
                }
            }
        }
    });
}

#[tauri::command]
pub fn readboard_runtime_snapshot(runtime: State<ReadboardRuntime>) -> ReadboardRuntimeDto {
    runtime.snapshot()
}

#[tauri::command]
pub fn readboard_runtime_path(preferences: State<PreferencesState>) -> Result<Option<String>, String> {
    preferences.readboard_path()
}

#[tauri::command]
pub fn readboard_save_runtime_path(
    app: AppHandle,
    preferences: State<PreferencesState>,
    path: String,
) -> Result<String, String> {
    let executable = Path::new(&path);
    if !executable.is_absolute()
        || !executable
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("exe"))
    {
        return Err("Select an absolute Windows readboard .exe path.".into());
    }
    preferences.save_readboard_path(&crate::app_preferences_path(&app)?, path)
}

#[tauri::command]
pub fn readboard_runtime_start(
    preferences: State<PreferencesState>,
    runtime: State<ReadboardRuntime>,
) -> Result<ReadboardRuntimeDto, String> {
    runtime.start(preferences.readboard_path()?.as_deref())
}

#[tauri::command]
pub async fn readboard_runtime_stop(
    runtime: State<'_, ReadboardRuntime>,
) -> Result<ReadboardRuntimeDto, String> {
    let runtime = runtime.inner().clone();
    tauri::async_runtime::spawn_blocking(move || runtime.stop(STOP_BUDGET))
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn readboard_runtime_restart(
    preferences: State<'_, PreferencesState>,
    runtime: State<'_, ReadboardRuntime>,
) -> Result<ReadboardRuntimeDto, String> {
    let path = preferences.readboard_path()?;
    let runtime = runtime.inner().clone();
    tauri::async_runtime::spawn_blocking(move || runtime.restart(path.as_deref(), STOP_BUDGET))
        .await
        .map_err(|error| error.to_string())?
}

/// Reconnect for a paused readboard session: a new runtime generation, verified by its next frame.
pub(crate) async fn restart_for_retry(app: &AppHandle) -> Result<ReadboardRuntimeDto, String> {
    let path = app.state::<PreferencesState>().readboard_path()?;
    let runtime = app.state::<ReadboardRuntime>().inner().clone();
    tauri::async_runtime::spawn_blocking(move || runtime.restart(path.as_deref(), STOP_BUDGET))
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
pub fn load_readboard_sync_preferences(
    preferences: State<PreferencesState>,
) -> Result<ReadboardSyncPreferencesDto, String> {
    preferences.readboard_sync_preferences()
}

#[tauri::command]
pub fn save_readboard_sync_preferences(
    app: AppHandle,
    preferences: State<PreferencesState>,
    current: State<CurrentGameState>,
    always_sync: bool,
    focus: bool,
    mute: bool,
    jump_to_last: bool,
) -> Result<ReadboardSyncPreferencesDto, String> {
    let value = ReadboardSyncPreferencesDto {
        always_sync,
        focus,
        mute,
        jump_to_last,
    };
    let saved = preferences.save_readboard_sync_preferences(&crate::app_preferences_path(&app)?, value)?;
    current.update_readboard_preferences(saved);
    external_sync::publish(&app, None);
    Ok(saved)
}

#[tauri::command]
pub fn begin_readboard_sync(
    app: AppHandle,
    preferences: State<PreferencesState>,
    current: State<CurrentGameState>,
    runtime: State<ReadboardRuntime>,
) -> Result<u64, String> {
    let settings = preferences.readboard_sync_preferences()?;
    let id = current
        .begin_readboard_start(&runtime.snapshot(), settings)
        .map_err(|error| error.message)?;
    external_sync::publish(&app, None);
    Ok(id)
}

#[tauri::command]
pub async fn prepare_readboard_sync(app: AppHandle, start_id: u64) -> Result<ExternalSyncStartDto, String> {
    let result = tauri::async_runtime::spawn_blocking({
        let app = app.clone();
        move || {
            app.state::<CurrentGameState>()
                .prepare_readboard_candidate(start_id, FIRST_FRAME_BUDGET)
                .map_err(|error| error.message)
        }
    })
    .await
    .map_err(|_| "readboard start worker failed.".to_string())
    .and_then(|result| result);
    if result.is_err() {
        app.state::<CurrentGameState>().cancel_external_start(start_id);
    }
    external_sync::publish(&app, None);
    result
}

/// Wheel/preview interaction on the Next board asks readboard to release focus (Java `loss`).
#[tauri::command]
pub fn readboard_focus(
    current: State<CurrentGameState>,
    runtime: State<ReadboardRuntime>,
) -> Result<bool, String> {
    match current.readboard_focus_generation() {
        Some(generation) => runtime.request_focus(generation).map(|()| true),
        None => Ok(false),
    }
}
