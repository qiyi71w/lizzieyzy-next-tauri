use crate::{
    current_game_state::human_match::match_failure, CurrentGameState, EngineCommandResult, PreferencesState,
};
use app_model::*;
use engine_manager::{EngineProfileCatalog, ForegroundEngineManager};
use tauri::{AppHandle, Emitter, Manager, State};

pub(crate) fn publish(app: &AppHandle, update: MatchUpdateDto) {
    let _ = app.emit("human-match-updated", update);
}

/// Each move is reserved under the document lock. Concurrent wakeups cannot take the same turn.
pub(crate) fn drive(app: AppHandle) {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<CurrentGameState>();
        let manager = app.state::<ForegroundEngineManager>();
        loop {
            let snapshot = state.human_match_snapshot();
            if snapshot.match_state.phase == MatchPhaseDto::Ending {
                if let Some(id) = snapshot.match_state.session_id {
                    if let Ok(update) = state.stop_human_match(&manager, &id) {
                        publish(&app, update);
                    }
                }
                return;
            }
            match state.take_match_work(&manager) {
                Ok(Some(work)) => {
                    publish(&app, state.human_match_snapshot());
                    let result = work.handle.wait_with_analysis(|job, frame| {
                        if let Some(update) = state.publish_match_analysis(&work.turn, work.epoch, job, frame)
                        {
                            publish(&app, update);
                        }
                    });
                    let update = if work.human {
                        state.finish_match_analysis(&work.turn, work.epoch, result)
                    } else {
                        state.accept_match_engine_result(&manager, &work.turn, result)
                    };
                    let ending = update.match_state.phase == MatchPhaseDto::Ending;
                    publish(&app, update);
                    if ending {
                        if let Ok(update) = state.stop_human_match(&manager, &work.turn.session_id) {
                            publish(&app, update);
                        }
                        return;
                    }
                    if work.human {
                        return;
                    }
                }
                Err(_) => {
                    let update = state.human_match_snapshot();
                    let id = update.match_state.session_id.clone();
                    publish(&app, update);
                    if let Some(id) = id {
                        if let Ok(update) = state.stop_human_match(&manager, &id) {
                            publish(&app, update);
                        }
                    }
                    return;
                }
                Ok(None) => return,
            }
        }
    });
}

#[tauri::command]
pub fn human_match_snapshot(state: State<'_, CurrentGameState>) -> MatchUpdateDto {
    state.human_match_snapshot()
}

#[tauri::command]
pub async fn human_match_start(
    app: AppHandle,
    request: HumanMatchStartDto,
) -> EngineCommandResult<MatchUpdateDto> {
    start(app, request, MatchModeDto::Human).await
}

#[tauri::command]
pub async fn pk_match_start(
    app: AppHandle,
    request: HumanMatchStartDto,
) -> EngineCommandResult<MatchUpdateDto> {
    start(app, request, MatchModeDto::Pk).await
}

async fn start(
    app: AppHandle,
    request: HumanMatchStartDto,
    mode: MatchModeDto,
) -> EngineCommandResult<MatchUpdateDto> {
    let worker_app = app.clone();
    let update = tauri::async_runtime::spawn_blocking(move || {
        let state = worker_app.state::<CurrentGameState>();
        let preferences = worker_app.state::<PreferencesState>();
        let manager = worker_app.state::<ForegroundEngineManager>();
        let path = crate::app_preferences_path(&worker_app)
            .map_err(|error| match_failure(EngineFailureKind::Command, error))?;
        preferences
            .load(&path, &manager)
            .map_err(|error| match_failure(EngineFailureKind::Command, error))?;
        let catalog = crate::DiskEngineCatalog {
            handle: worker_app.clone(),
        };
        let load = |id: Option<&str>| -> EngineCommandResult<EngineProfileDto> {
            id.and_then(|id| catalog.get(id))
                .map(|record| record.profile)
                .ok_or_else(|| {
                    Box::new(match_failure(
                        EngineFailureKind::ProfileNotFound,
                        "Choose an existing saved engine profile.",
                    ))
                })
        };
        if mode == MatchModeDto::Pk {
            let profiles = [
                load(request.settings.pk_black.profile_id.as_deref())?,
                load(request.settings.pk_white.profile_id.as_deref())?,
            ];
            state.start_pk_match(&manager, &preferences, &path, request, profiles, |update| {
                publish(&worker_app, update)
            })
        } else {
            let profile = load(request.settings.profile_id.as_deref())?;
            state.start_human_match(&manager, &preferences, &path, request, profile, |update| {
                publish(&worker_app, update)
            })
        }
    })
    .await
    .map_err(|error| {
        Box::new(match_failure(
            EngineFailureKind::Protocol,
            format!("Match start worker failed: {error}"),
        ))
    })??;
    publish(&app, update.clone());
    drive(app);
    Ok(update)
}

#[tauri::command]
pub async fn human_match_action(
    app: AppHandle,
    turn: MatchTurnDto,
    action: HumanMatchActionDto,
) -> EngineCommandResult<MatchUpdateDto> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<CurrentGameState>();
        let manager = app.state::<ForegroundEngineManager>();
        let result = state.human_match_action(&manager, turn, action);
        publish(&app, state.human_match_snapshot());
        drive(app.clone());
        result
    })
    .await
    .map_err(|error| {
        Box::new(match_failure(
            EngineFailureKind::Protocol,
            format!("Match action worker failed: {error}"),
        ))
    })?
}

#[tauri::command]
pub async fn human_match_analysis_policy(
    app: AppHandle,
    turn: MatchTurnDto,
    policy: MatchAnalysisPolicyDto,
) -> EngineCommandResult<MatchUpdateDto> {
    tauri::async_runtime::spawn_blocking(move || {
        let result = app.state::<CurrentGameState>().human_match_analysis_policy(
            &app.state::<ForegroundEngineManager>(),
            turn,
            policy,
        );
        publish(&app, app.state::<CurrentGameState>().human_match_snapshot());
        drive(app.clone());
        result
    })
    .await
    .map_err(|error| {
        Box::new(match_failure(
            EngineFailureKind::Protocol,
            format!("Match analysis policy worker failed: {error}"),
        ))
    })?
}

#[tauri::command]
pub async fn human_match_stop(app: AppHandle, session_id: String) -> EngineCommandResult<MatchUpdateDto> {
    tauri::async_runtime::spawn_blocking(move || {
        let update = app
            .state::<CurrentGameState>()
            .stop_human_match(&app.state::<ForegroundEngineManager>(), &session_id)?;
        publish(&app, update.clone());
        Ok(update)
    })
    .await
    .map_err(|error| {
        Box::new(match_failure(
            EngineFailureKind::Protocol,
            format!("Match stop worker failed: {error}"),
        ))
    })?
}

#[tauri::command]
pub async fn pk_match_pause(app: AppHandle, session_id: String) -> EngineCommandResult<MatchUpdateDto> {
    let worker_app = app.clone();
    let update = tauri::async_runtime::spawn_blocking(move || {
        worker_app.state::<CurrentGameState>().pause_pk_match(
            &worker_app.state::<ForegroundEngineManager>(),
            &session_id,
            |update| publish(&worker_app, update),
        )
    })
    .await
    .map_err(|error| {
        Box::new(match_failure(
            EngineFailureKind::Protocol,
            format!("PK pause worker failed: {error}"),
        ))
    })??;
    publish(&app, update.clone());
    if update.match_state.phase == MatchPhaseDto::Ending {
        drive(app);
    }
    Ok(update)
}

/// Rebuilding a reaped GTP side runs readiness, so Resume blocks on a worker like Pause.
#[tauri::command]
pub async fn pk_match_resume(app: AppHandle, session_id: String) -> EngineCommandResult<MatchUpdateDto> {
    let worker_app = app.clone();
    let update = tauri::async_runtime::spawn_blocking(move || {
        worker_app.state::<CurrentGameState>().resume_pk_match(
            &worker_app.state::<ForegroundEngineManager>(),
            &session_id,
            |update| publish(&worker_app, update),
        )
    })
    .await
    .map_err(|error| {
        Box::new(match_failure(
            EngineFailureKind::Protocol,
            format!("PK resume worker failed: {error}"),
        ))
    })??;
    publish(&app, update.clone());
    drive(app);
    Ok(update)
}
