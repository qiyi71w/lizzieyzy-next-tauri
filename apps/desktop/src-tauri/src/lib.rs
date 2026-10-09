use app_model::CurrentGameSaveResultDto;
use app_model::{
    AnalysisFrameDto, AnalysisJobModeDto, AnalysisJobStartedDto, AppHealthDto, CurrentGameError,
    CurrentGameResultDto, EngineFailureDto, EngineFailureKind, EngineOperationDto, EngineProfileDto,
    ForegroundEngineEventDto, ForegroundEngineSnapshotDto, GameFileFormatDto, GameFileImportDto, MoveVertex,
    NodePath, PlayerColor, PositionDto, ProviderError, ProviderErrorKind, ProviderImportRequest,
    ProviderImportResult, ProviderKind, ReadboardSidecarSyncSnapshotRequest,
    ReadboardSidecarSyncSnapshotResult, StoneDto,
};
use engine_manager::{
    build_command_spec, check_assets, default_engine_profiles_settings, parse_engine_profiles,
    save_engine_profiles as persist_engine_profiles, AssetCheck, CommandSpec, EngineProfileCatalog,
    EngineProfilesSettings as EngineProfilesSettingsDto, ForegroundEngineConfig, ForegroundEngineManager,
    SavedEngineProfile, SelectedNodeJobRequest, WholeGameWorkItem,
};
use katago_protocol::{analysis_query_from_position, AnalysisQueryOptions};
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter, Manager, State};

mod continuous_analysis;
mod gesture_timing;
use gesture_timing::board_gesture_timing;
mod external_sync;
mod provider_network;
use provider_network::{
    begin_provider_request, cancel_provider_request, clear_fox_recents, load_fox_kifu_state,
    load_tencent_history, load_yike_locator, network_snapshot, provider_fox_list, provider_fox_list_more,
    provider_fox_preview, provider_tencent_list, provider_tencent_preview, provider_yike_list,
    provider_yike_preview, remember_fox_lookup, save_network_settings, save_tencent_query, save_yike_locator,
};
mod window_geometry;
use continuous_analysis::{foreground_engine_continuous_action, PreferencesState};
mod main_window_pin;
use main_window_pin::{main_window_pin_status, set_main_window_pin, MainWindowPin};
mod current_game_state;
mod document_departure;
mod export;
mod file_activation;
mod human_match;
mod readboard;
mod save_as;
mod session_recovery;
mod models;
mod managed_resources;
#[cfg(windows)]
extern crate windows_core;
#[cfg(test)]
use app_model::ProviderGameMetadata;
use app_preferences::{AppPreferencesDto, AppPreferencesLoadResultDto, APP_PREFERENCES_FILE};
use current_game_recovery::FileRecoveryStore;
use current_game_state::{CurrentGameState, WholeGameAdmission};
use document_departure::{
    confirm_application_exit_anyway, confirm_native_exit, prepare_application_exit,
    prepare_document_replacement, resolve_application_exit, resolve_document_replacement,
    retry_application_teardown, APPLICATION_EXIT_REQUESTED_EVENT,
};
use file_activation::{
    handle_file_drop, handle_second_instance, mark_file_activation_ready, set_file_activation_busy,
    take_initial_file_activation, take_pending_file_activation, FileActivationOwner,
};
use session_recovery::{
    current_game_recovery_protection, discard_current_game_recovery, inspect_current_game_recovery,
    restore_current_game_recovery, retry_current_game_recovery, spawn_recovery_writer,
};
use std::sync::Mutex;
#[cfg(test)]
use uuid::Uuid;

const ENGINE_PROFILE_FILE: &str = "lizzieyzy-next-engine-profile.json";
static ENGINE_CATALOG_WRITES: Mutex<()> = Mutex::new(());

type EngineCommandResult<T> = Result<T, Box<EngineFailureDto>>;

struct DiskEngineCatalog {
    handle: AppHandle,
}

impl EngineProfileCatalog for DiskEngineCatalog {
    fn get(&self, profile_id: &str) -> Option<SavedEngineProfile> {
        let settings = load_engine_profiles_from_disk(&self.handle).ok()?;
        settings
            .profiles
            .into_iter()
            .find(|record| record.id == profile_id)
            .map(|record| SavedEngineProfile {
                profile_id: record.id,
                profile: record.profile,
            })
    }

    fn autoload_profile_id(&self) -> Option<String> {
        load_engine_profiles_from_disk(&self.handle)
            .ok()
            .and_then(|settings| settings.autoload_profile_id)
    }
}

fn map_engine_failure(failure: EngineFailureDto) -> String {
    failure.message
}

#[tauri::command]
fn health() -> AppHealthDto {
    AppHealthDto {
        app: "LizzieYzy Next".to_string(),
        architecture: "Tauri 2 + Rust workspace + TypeScript UI".to_string(),
        rust_backend_ready: true,
        notes: vec![
            "SGF parser command is wired".to_string(),
            "Fake analysis command is wired for UI development before KataGo process streaming".to_string(),
            "KataGo launch plan command is wired".to_string(),
        ],
    }
}

#[tauri::command]
fn parse_sgf_summary(sgf_text: String) -> Result<app_model::GameDto, String> {
    let document = sgf::parse_sgf(&sgf_text).map_err(|err| err.to_string())?;
    Ok(sgf::to_game_dto(document))
}

#[tauri::command]
fn provider_import_from_payload(
    request: ProviderImportRequest,
) -> Result<ProviderImportResult, ProviderError> {
    let result = match request.provider {
        ProviderKind::Yike => provider_yike::import_payload(request),
        ProviderKind::Fox => provider_fox::import_payload(request),
        ProviderKind::Tencent => provider_tencent::import_payload(request),
    }?;
    enrich_provider_import_result(result)
}

#[tauri::command]
fn readboard_sidecar_sync_snapshot(
    request: ReadboardSidecarSyncSnapshotRequest,
) -> Result<ReadboardSidecarSyncSnapshotResult, ProviderError> {
    validate_timeout_ms(request.timeout_ms, "readboard_sidecar_sync_snapshot")?;
    if request
        .image_path
        .as_deref()
        .is_none_or(|value| value.trim().is_empty())
        && request
            .image_base64
            .as_deref()
            .is_none_or(|value| value.trim().is_empty())
        && request
            .sgf_text
            .as_deref()
            .is_none_or(|value| value.trim().is_empty())
    {
        return Err(ProviderError {
            kind: ProviderErrorKind::InvalidRequest,
            message: "readboard_sidecar_sync_snapshot requires image_path, image_base64, or sgf_text"
                .to_string(),
        });
    }
    if let Some(protocol_line) = request
        .sgf_text
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return readboard_sidecar::preview_snapshot_line(&request, protocol_line).map_err(readboard_error);
    }
    Err(ProviderError {
        kind: ProviderErrorKind::RuntimeUnavailable,
        message: "readboard image OCR runtime is unavailable; provide sgf_text as an offline snapshot protocol line"
            .to_string(),
    })
}

#[tauri::command]
fn replay_sgf_positions(sgf_text: String) -> Result<Vec<PositionDto>, String> {
    sgf::replay_sgf_positions(&sgf_text).map_err(|err| err.to_string())
}

#[tauri::command]
fn read_game_file(path: String) -> Result<GameFileImportDto, String> {
    let path = non_empty_path(path)?;
    let path = fs::canonicalize(&path)
        .map_err(|error| format!("failed to read game file {}: {error}", path.display()))?;
    let display_path = path.display().to_string();
    let input =
        fs::read(&path).map_err(|error| format!("failed to read game file {display_path}: {error}"))?;
    import_game_input(display_path.clone(), input, Some(display_path))
}

#[tauri::command]
fn import_game_bytes(file_name: String, input: Vec<u8>) -> Result<GameFileImportDto, String> {
    let display_path = non_empty_path(file_name)?.display().to_string();
    import_game_input(display_path, input, None)
}

fn import_game_input(
    display_path: String,
    input: Vec<u8>,
    source_native_path: Option<String>,
) -> Result<GameFileImportDto, String> {
    let path = Path::new(&display_path);
    let display_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("game file has no valid file name: {display_path}"))?
        .to_string();
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .ok_or_else(|| format!("unsupported game file: {display_path}"))?;
    if !matches!(extension.as_str(), "sgf" | "txt" | "gib") {
        return Err(format!("unsupported game file: {display_path}"));
    }
    let opened_path = source_native_path.clone();
    let (format, sgf_text, native_path) = if extension == "gib" {
        (
            GameFileFormatDto::Gib,
            sgf::import_gib(&input)
                .map_err(|error| format!("failed to import GIB file {display_path}: {error}"))?,
            None,
        )
    } else {
        (
            GameFileFormatDto::Sgf,
            String::from_utf8(input)
                .map_err(|error| format!("failed to decode game file {display_path} as UTF-8: {error}"))?,
            source_native_path,
        )
    };
    Ok(GameFileImportDto {
        format,
        sgf_text,
        display_path,
        display_name,
        native_path,
        opened_path,
    })
}

#[tauri::command]
fn serialize_current_game(state: State<CurrentGameState>) -> Result<String, CurrentGameError> {
    state.serialize()
}

#[tauri::command]
async fn save_current_game(
    app: AppHandle,
    state: State<'_, CurrentGameState>,
    path: String,
    selected_path: NodePath,
) -> Result<CurrentGameSaveResultDto, String> {
    let snapshot = state.capture_save_snapshot(selected_path)?;
    tauri::async_runtime::spawn_blocking(move || {
        app.state::<CurrentGameState>()
            .persist_save_snapshot(path, snapshot)
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn save_current_game_as(
    app: AppHandle,
    state: State<'_, CurrentGameState>,
    selected_path: NodePath,
    default_file_name: Option<String>,
) -> Result<Option<CurrentGameSaveResultDto>, String> {
    let snapshot = state.capture_save_snapshot(selected_path)?;
    let default_file_name = default_file_name.unwrap_or_else(|| "review.sgf".to_string());
    tauri::async_runtime::spawn_blocking(move || {
        let outcome = save_as::pick_save_as_outcome(&app, &default_file_name)?;
        save_as::persist_current_game_save_as(&app.state::<CurrentGameState>(), outcome, snapshot)
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
fn project_current_game_mainline(
    state: State<CurrentGameState>,
) -> Result<app_model::GameDto, CurrentGameError> {
    state.mainline_projection()
}

#[tauri::command]
fn select_current_game_node(
    state: State<CurrentGameState>,
    path: NodePath,
    generation: u64,
) -> Result<CurrentGameResultDto, String> {
    state
        .select_path(path, generation)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn find_current_game_recorded_point(
    state: State<CurrentGameState>,
    path: NodePath,
    generation: u64,
    point: app_model::PointDto,
    choices: Vec<app_model::AnalysisBranchChoiceDto>,
    scope: app_model::PointSearchScopeDto,
) -> Result<Option<NodePath>, String> {
    state
        .find_recorded_point(path, generation, point, choices, scope)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn play_current_game(
    state: State<CurrentGameState>,
    path: NodePath,
    vertex: MoveVertex,
) -> Result<CurrentGameResultDto, String> {
    state.play(path, vertex).map_err(|error| error.to_string())
}

#[tauri::command]
fn author_current_game(
    state: State<CurrentGameState>,
    generation: u64,
    path: NodePath,
    action: app_model::SgfAuthoringActionDto,
) -> Result<CurrentGameResultDto, String> {
    state
        .author(generation, path, action)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn set_current_game_personal_comment(
    state: State<CurrentGameState>,
    path: NodePath,
    comment: String,
) -> Result<CurrentGameResultDto, String> {
    state
        .set_personal_comment(path, comment)
        .map_err(|error| error.to_string())
}
#[tauri::command]
fn set_current_game_metadata(
    state: State<CurrentGameState>,
    generation: u64,
    black_name: String,
    white_name: String,
    komi: f32,
) -> Result<CurrentGameResultDto, String> {
    state
        .set_metadata(generation, black_name, white_name, komi)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn edit_current_game_markup(
    state: State<CurrentGameState>,
    path: NodePath,
    generation: u64,
    action: app_model::SgfMarkupActionDto,
) -> Result<CurrentGameResultDto, String> {
    state
        .edit_markup(path, generation, action)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn remove_current_game_variation(
    state: State<CurrentGameState>,
    path: NodePath,
    generation: u64,
) -> Result<CurrentGameResultDto, String> {
    state
        .remove_variation(path, generation)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn promote_current_game_to_main(
    state: State<CurrentGameState>,
    path: NodePath,
    generation: u64,
) -> Result<CurrentGameResultDto, String> {
    state
        .promote_to_main(path, generation)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn apply_root_setup(
    state: State<CurrentGameState>,
    generation: u64,
    stones: Vec<StoneDto>,
    to_play: PlayerColor,
) -> Result<CurrentGameResultDto, String> {
    state
        .apply_root_setup(generation, stones, to_play)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn convert_to_root_setup(
    state: State<CurrentGameState>,
    generation: u64,
    path: NodePath,
) -> Result<CurrentGameResultDto, String> {
    state
        .convert_to_root_setup(generation, path)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn undo_current_game(
    state: State<CurrentGameState>,
    generation: u64,
) -> Result<CurrentGameResultDto, String> {
    state.undo(generation).map_err(|error| error.to_string())
}

#[tauri::command]
fn redo_current_game(
    state: State<CurrentGameState>,
    generation: u64,
) -> Result<CurrentGameResultDto, String> {
    state.redo(generation).map_err(|error| error.to_string())
}

#[tauri::command]
fn enter_trial(state: State<CurrentGameState>) -> Result<app_model::TrialSessionDto, String> {
    state.enter_trial()
}

#[tauri::command]
fn exit_trial(state: State<CurrentGameState>, session_id: u64) -> Result<CurrentGameResultDto, String> {
    state.exit_trial(session_id)
}
#[tauri::command]
fn enter_scoring(
    state: State<CurrentGameState>,
    rule: app_model::ScoringRuleDto,
) -> Result<app_model::ScoringSessionDto, String> {
    state.enter_scoring(rule)
}

#[tauri::command]
fn update_scoring(
    state: State<CurrentGameState>,
    session_id: u64,
    revision: u64,
    action: app_model::ScoringActionDto,
) -> Result<app_model::ScoringSessionDto, CurrentGameError> {
    state.update_scoring(session_id, revision, action)
}

#[tauri::command]
fn exit_scoring(
    state: State<CurrentGameState>,
    session_id: u64,
    revision: u64,
    confirm: bool,
) -> Result<CurrentGameResultDto, String> {
    state.exit_scoring(session_id, revision, confirm)
}

#[tauri::command]
fn trial_snapshot(
    state: State<CurrentGameState>,
    session_id: u64,
) -> Result<app_model::TrialSessionDto, CurrentGameError> {
    state.trial_snapshot(session_id)
}

#[tauri::command]
fn trial_select(
    state: State<CurrentGameState>,
    session_id: u64,
    revision: u64,
    path: NodePath,
) -> Result<app_model::TrialSessionDto, CurrentGameError> {
    state.trial_select(session_id, revision, path)
}

#[tauri::command]
fn trial_play(
    state: State<CurrentGameState>,
    session_id: u64,
    revision: u64,
    vertex: MoveVertex,
) -> Result<app_model::TrialSessionDto, CurrentGameError> {
    state.trial_play(session_id, revision, vertex)
}

#[tauri::command]
fn trial_undo(
    state: State<CurrentGameState>,
    session_id: u64,
    revision: u64,
) -> Result<app_model::TrialSessionDto, CurrentGameError> {
    state.trial_undo(session_id, revision)
}

#[tauri::command]
fn trial_start_finite(
    manager: State<ForegroundEngineManager>,
    state: State<CurrentGameState>,
    session_id: u64,
    revision: u64,
    run_id: String,
    max_visits: u32,
) -> EngineCommandResult<AnalysisJobStartedDto> {
    state.trial_start_finite(&manager, session_id, revision, run_id, max_visits)
}

#[tauri::command]
fn classify_problems(frames: Vec<AnalysisFrameDto>) -> Vec<app_model::ProblemMarkerDto> {
    analysis_core::classify_problem_markers(&frames)
}

#[tauri::command]
fn katago_launch_plan(profile: EngineProfileDto) -> Result<CommandSpec, String> {
    build_command_spec(&profile).map_err(|err| err.to_string())
}

#[tauri::command]
fn engine_asset_checks(profile: EngineProfileDto) -> Vec<AssetCheck> {
    check_assets(&profile)
}

#[tauri::command]
fn load_app_preferences(
    app_handle: AppHandle,
    preferences: State<PreferencesState>,
    manager: State<ForegroundEngineManager>,
) -> Result<AppPreferencesLoadResultDto, String> {
    preferences.load(&app_preferences_path(&app_handle)?, &manager)
}

#[tauri::command]
fn save_app_preferences(
    app_handle: AppHandle,
    state: State<PreferencesState>,
    manager: State<ForegroundEngineManager>,
    preferences: AppPreferencesDto,
) -> Result<AppPreferencesDto, String> {
    let path = app_preferences_path(&app_handle)?;
    state.save(&path, &manager, preferences)
}

#[tauri::command]
fn update_workspace_visibility(
    app_handle: AppHandle,
    state: State<PreferencesState>,
    left: Option<bool>,
    right: Option<bool>,
) -> Result<app_model::WorkspaceVisibilityDto, String> {
    state.update_workspace_visibility(&app_preferences_path(&app_handle)?, left, right)
}

#[tauri::command]
fn update_recent_game_history(
    app_handle: AppHandle,
    state: State<PreferencesState>,
    opened_path: Option<String>,
) -> Result<Vec<String>, String> {
    state.update_recent_history(&app_preferences_path(&app_handle)?, opened_path.as_deref())
}

#[tauri::command]
fn update_workspace_shares(
    app_handle: AppHandle,
    state: State<PreferencesState>,
    shares: Option<app_model::WorkspaceSharesDto>,
) -> Result<Option<app_model::WorkspaceSharesDto>, String> {
    state.update_workspace_shares(&app_preferences_path(&app_handle)?, shares)
}

#[tauri::command]
fn load_engine_profiles_settings(app_handle: AppHandle) -> Result<EngineProfilesSettingsDto, String> {
    load_engine_profiles_from_disk(&app_handle)
}

fn load_engine_profiles_from_disk(app_handle: &AppHandle) -> Result<EngineProfilesSettingsDto, String> {
    let path = engine_profile_path(app_handle)?;
    match fs::read_to_string(&path) {
        Ok(contents) => parse_engine_profiles_settings(&contents, &path),
        Err(err) if err.kind() == ErrorKind::NotFound => {
            load_legacy_engine_profile_settings(&legacy_engine_profile_path()?)
        }
        Err(err) => Err(format!("failed to read {}: {err}", path.display())),
    }
}

#[tauri::command]
fn save_engine_profiles_settings(
    app_handle: AppHandle,
    manager: State<'_, ForegroundEngineManager>,
    settings: EngineProfilesSettingsDto,
) -> Result<EngineProfilesSettingsDto, String> {
    let _transaction = ENGINE_CATALOG_WRITES
        .lock()
        .map_err(|_| "engine catalog lock poisoned")?;
    let current = load_engine_profiles_from_disk(&app_handle)?;
    let path = engine_profile_path(&app_handle)?;
    let mut retained = models::saved_paths(&current);
    retained.extend(models::saved_paths(&settings));
    app_handle.state::<engine_manager::models::ModelInventory>().remember_saved(&retained)?;
    save_engine_profiles_at_path(&path, &manager, &current, settings)
}

fn save_engine_profiles_at_path(
    path: &Path,
    manager: &ForegroundEngineManager,
    current: &EngineProfilesSettingsDto,
    settings: EngineProfilesSettingsDto,
) -> Result<EngineProfilesSettingsDto, String> {
    let settings = engine_manager::prepare_engine_profiles_save(current, settings)?;
    for record in &current.profiles {
        if !settings.profiles.iter().any(|next| next.id == record.id) {
            manager
                .assert_profile_deletable(&record.id)
                .map_err(map_engine_failure)?;
        }
    }
    persist_engine_profiles(path, settings)
}

#[tauri::command]
fn reorder_engine_profiles_settings(
    app_handle: AppHandle,
    request: app_model::EngineProfileOrderRequestDto,
) -> Result<EngineProfilesSettingsDto, String> {
    let _transaction = ENGINE_CATALOG_WRITES
        .lock()
        .map_err(|_| "engine catalog lock poisoned")?;
    let current = load_engine_profiles_from_disk(&app_handle)?;
    let path = engine_profile_path(&app_handle)?;
    reorder_engine_profiles_at_path(&path, current, &request)
}

fn reorder_engine_profiles_at_path(
    path: &Path,
    current: EngineProfilesSettingsDto,
    request: &app_model::EngineProfileOrderRequestDto,
) -> Result<EngineProfilesSettingsDto, String> {
    engine_manager::reorder_engine_profiles(path, current, request)
}

fn bind_selected_node_job(
    current_game: &CurrentGameState,
    run_id: String,
    generation: u64,
    node_path: NodePath,
    mode: AnalysisJobModeDto,
    max_visits: Option<u32>,
) -> EngineCommandResult<SelectedNodeJobRequest> {
    let (snapshot, board_width, board_height, komi, rules) = current_game
        .admit_selected_node(generation, &node_path)
        .map_err(|error| {
            Box::new(EngineFailureDto {
                operation: EngineOperationDto::Job,
                run_id: Some(run_id.clone()),
                switch_id: None,
                job_id: None,
                profile_id: None,
                kind: EngineFailureKind::InvalidState,
                message: error.message,
                diagnostic_summary: None,
            })
        })?;
    let mut request = continuous_analysis::position_request(
        generation,
        node_path,
        snapshot,
        board_width,
        board_height,
        komi,
        rules,
    )
    .map_err(|error| {
        Box::new(EngineFailureDto {
            operation: EngineOperationDto::Job,
            run_id: Some(run_id.clone()),
            switch_id: None,
            job_id: None,
            profile_id: None,
            kind: EngineFailureKind::Protocol,
            message: error.to_string(),
            diagnostic_summary: None,
        })
    })?;
    request.run_id = run_id;
    request.mode = mode;
    request.query.max_visits = max_visits;
    Ok(request)
}

#[tauri::command]
fn foreground_engine_start_selected_node(
    manager: State<'_, ForegroundEngineManager>,
    current_game: State<'_, CurrentGameState>,
    run_id: String,
    generation: u64,
    node_path: NodePath,
    max_visits: u32,
) -> EngineCommandResult<AnalysisJobStartedDto> {
    manager
        .start_selected_node_job(bind_selected_node_job(
            &current_game,
            run_id,
            generation,
            node_path,
            AnalysisJobModeDto::Finite,
            Some(max_visits),
        )?)
        .map_err(Box::new)
}

#[tauri::command]
async fn foreground_engine_confirm_rules(
    manager: State<'_, ForegroundEngineManager>,
    current_game: State<'_, CurrentGameState>,
    request: app_model::OrdinaryRulesRequestDto,
) -> EngineCommandResult<app_model::OrdinaryRulesSnapshotDto> {
    let run_id = request.run_id.clone();
    let handle = current_game.start_ordinary_rules(&manager, request)?;
    let result = tauri::async_runtime::spawn_blocking(move || handle.wait().map_err(Box::new))
        .await
        .map_err(|_| {
            Box::new(EngineFailureDto {
                operation: EngineOperationDto::Job,
                run_id: Some(run_id),
                job_id: None,
                switch_id: None,
                profile_id: None,
                kind: EngineFailureKind::Protocol,
                message: "Rules confirmation worker failed".into(),
                diagnostic_summary: None,
            })
        })??;
    current_game.publish_ordinary_rules(&manager, result)
}

#[tauri::command]
async fn foreground_engine_game_move(
    manager: State<'_, ForegroundEngineManager>,
    current_game: State<'_, CurrentGameState>,
    request: app_model::GameMoveRequestDto,
) -> EngineCommandResult<app_model::GameMoveResultDto> {
    let handle = current_game.start_game_move(&manager, request)?;
    let identity = handle.identity.clone();
    let result = tauri::async_runtime::spawn_blocking(move || handle.wait().map_err(Box::new))
        .await
        .map_err(|error| {
            Box::new(EngineFailureDto {
                operation: EngineOperationDto::Job,
                run_id: Some(identity.run_id),
                job_id: Some(identity.job_id),
                switch_id: None,
                profile_id: None,
                kind: EngineFailureKind::Protocol,
                message: format!("move worker failed: {error}"),
                diagnostic_summary: None,
            })
        })??;
    current_game.publish_game_move(&manager, result)
}

#[tauri::command]
fn foreground_engine_cancel_game_move(
    manager: State<'_, ForegroundEngineManager>,
    run_id: String,
    job_id: String,
) -> EngineCommandResult<()> {
    manager.cancel_game_move(&run_id, &job_id).map_err(Box::new)
}

fn cancel_document_analysis_job(
    current_game: &CurrentGameState,
    manager: &ForegroundEngineManager,
    run_id: &str,
    job_id: &str,
) -> EngineCommandResult<()> {
    for job in document_departure::jobs_from_snapshot(&manager.snapshot()) {
        if job.run_id == run_id && job.job_id == job_id {
            current_game.seal_job(&job);
        }
    }
    manager.cancel_job(run_id, job_id).map_err(Box::new)
}

#[tauri::command]
fn foreground_engine_cancel_job(
    manager: State<'_, ForegroundEngineManager>,
    current_game: State<'_, CurrentGameState>,
    run_id: String,
    job_id: String,
) -> EngineCommandResult<()> {
    cancel_document_analysis_job(&current_game, &manager, &run_id, &job_id)
}

fn job_failure(run_id: &str, kind: EngineFailureKind, message: String) -> EngineFailureDto {
    EngineFailureDto {
        operation: EngineOperationDto::Job,
        run_id: Some(run_id.to_string()),
        switch_id: None,
        job_id: None,
        profile_id: None,
        kind,
        message,
        diagnostic_summary: None,
    }
}

fn whole_game_work_items(
    admitted: &WholeGameAdmission,
    max_visits: u32,
    run_id: &str,
) -> EngineCommandResult<Vec<WholeGameWorkItem>> {
    admitted
        .nodes
        .iter()
        .map(|snapshot| {
            let query = analysis_query_from_position(
                admitted.board_width,
                admitted.board_height,
                admitted.komi,
                &snapshot.position.stones,
                snapshot.position.to_play,
                AnalysisQueryOptions {
                    id: "pending".to_string(),
                    rules: admitted.rules.clone(),
                    turn: snapshot.position.move_number,
                    max_visits: Some(max_visits),
                    include_ownership: Some(true),
                    include_policy: Some(true),
                },
            )
            .map_err(|error| {
                Box::new(job_failure(
                    run_id,
                    EngineFailureKind::Protocol,
                    error.to_string(),
                ))
            })?;
            Ok(WholeGameWorkItem {
                node_path: snapshot.path.clone(),
                query,
                board_width: admitted.board_width,
                board_height: admitted.board_height,
                move_number: snapshot.position.move_number,
            })
        })
        .collect()
}

#[tauri::command]
fn katago_start_analyze_game(
    manager: State<'_, ForegroundEngineManager>,
    current_game: State<'_, CurrentGameState>,
    run_id: String,
    generation: u64,
    max_visits: u32,
) -> EngineCommandResult<AnalysisJobStartedDto> {
    current_game.start_first_child_analysis(&manager, run_id, generation, max_visits)
}

#[tauri::command]
fn preview_analysis_scope(
    manager: State<'_, ForegroundEngineManager>,
    current_game: State<'_, CurrentGameState>,
    generation: u64,
    scope: app_model::AnalysisScopeDto,
    swing_criteria: Option<app_model::AnalysisSwingCriteriaDto>,
) -> EngineCommandResult<app_model::AnalysisScopePreviewDto> {
    manager
        .check_analysis_task_admission(swing_criteria.as_ref())
        .map_err(Box::new)?;
    current_game
        .preview_analysis_scope(generation, scope, swing_criteria)
        .map_err(|error| {
            Box::new(EngineFailureDto {
                operation: EngineOperationDto::Job,
                run_id: None,
                switch_id: None,
                job_id: None,
                profile_id: None,
                kind: EngineFailureKind::InvalidState,
                message: error.message,
                diagnostic_summary: None,
            })
        })
}

#[tauri::command]
fn start_analysis_task(
    manager: State<'_, ForegroundEngineManager>,
    current_game: State<'_, CurrentGameState>,
    run_id: String,
    preview: app_model::AnalysisScopePreviewDto,
    strategy: app_model::AnalysisTaskStrategyDto,
    conditions: app_model::AnalysisStageConditionsDto,
    overview_conditions: Option<app_model::AnalysisStageConditionsDto>,
) -> EngineCommandResult<app_model::AnalysisTaskDto> {
    match strategy {
        app_model::AnalysisTaskStrategyDto::SingleStage => {
            current_game.start_analysis_task(&manager, run_id, preview, conditions)
        }
        app_model::AnalysisTaskStrategyDto::AllPositionsTwoStage => {
            let overview_conditions = overview_conditions.ok_or_else(|| {
                job_failure(
                    &run_id,
                    EngineFailureKind::InvalidState,
                    "All-position analysis requires overview conditions.".into(),
                )
            })?;
            current_game.start_all_positions_analysis_task(
                &manager,
                run_id,
                preview,
                overview_conditions,
                conditions,
            )
        }
        app_model::AnalysisTaskStrategyDto::SwingSelectedTwoStage => {
            let overview_conditions = overview_conditions.ok_or_else(|| {
                job_failure(
                    &run_id,
                    EngineFailureKind::InvalidState,
                    "Swing-selected analysis requires overview conditions.".into(),
                )
            })?;
            current_game.start_swing_analysis_task(&manager, run_id, preview, overview_conditions, conditions)
        }
    }
}

#[tauri::command]
fn analysis_task_snapshot(manager: State<'_, ForegroundEngineManager>) -> Option<app_model::AnalysisTaskDto> {
    manager.analysis_task_snapshot()
}

#[tauri::command]
fn pause_analysis_task(
    manager: State<'_, ForegroundEngineManager>,
    current_game: State<'_, CurrentGameState>,
    run_id: String,
    task_id: String,
) -> EngineCommandResult<app_model::AnalysisTaskDto> {
    current_game.pause_analysis_task(&manager, &run_id, &task_id)
}

#[tauri::command]
fn continue_analysis_task(
    manager: State<'_, ForegroundEngineManager>,
    current_game: State<'_, CurrentGameState>,
    run_id: String,
    task_id: String,
) -> EngineCommandResult<app_model::AnalysisTaskDto> {
    current_game.continue_analysis_task(&manager, &run_id, &task_id)
}

#[tauri::command]
fn katago_cancel_analysis(
    manager: State<'_, ForegroundEngineManager>,
    current_game: State<'_, CurrentGameState>,
    run_id: String,
    job_id: String,
) -> EngineCommandResult<()> {
    cancel_document_analysis_job(&current_game, &manager, &run_id, &job_id)
}

fn parse_engine_profiles_settings(contents: &str, path: &Path) -> Result<EngineProfilesSettingsDto, String> {
    parse_engine_profiles(contents).map_err(|err| format!("failed to parse {}: {err}", path.display()))
}

fn load_legacy_engine_profile_settings(legacy_path: &Path) -> Result<EngineProfilesSettingsDto, String> {
    match fs::read_to_string(legacy_path) {
        Ok(contents) => parse_engine_profiles_settings(&contents, legacy_path),
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(default_engine_profiles_settings()),
        Err(err) => Err(format!("failed to read {}: {err}", legacy_path.display())),
    }
}

fn engine_profile_path(app_handle: &AppHandle) -> Result<PathBuf, String> {
    let dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|err| format!("failed to resolve app data directory for engine profiles: {err}"))?;
    fs::create_dir_all(&dir).map_err(|err| {
        format!(
            "failed to create engine profile directory {}: {err}",
            dir.display()
        )
    })?;
    Ok(dir.join(ENGINE_PROFILE_FILE))
}

fn legacy_engine_profile_path() -> Result<PathBuf, String> {
    std::env::current_dir()
        .map(|dir| dir.join(ENGINE_PROFILE_FILE))
        .map_err(|err| format!("failed to resolve current directory for legacy engine profile: {err}"))
}

fn app_preferences_path(app_handle: &AppHandle) -> Result<PathBuf, String> {
    let dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|err| format!("failed to resolve app data directory for app preferences: {err}"))?;
    fs::create_dir_all(&dir).map_err(|err| {
        format!(
            "failed to create app preferences directory {}: {err}",
            dir.display()
        )
    })?;
    Ok(dir.join(APP_PREFERENCES_FILE))
}

fn non_empty_path(path: String) -> Result<PathBuf, String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err("path must not be empty".to_string());
    }
    Ok(PathBuf::from(trimmed))
}

fn enrich_provider_import_result(
    mut result: ProviderImportResult,
) -> Result<ProviderImportResult, ProviderError> {
    let document = sgf::parse_sgf(&result.sgf_text).map_err(|err| ProviderError {
        kind: ProviderErrorKind::ParseFailed,
        message: format!("failed to parse imported provider SGF: {err}"),
    })?;
    result.summary.provider = result.provider;
    result.summary.board_width = Some(document.board_width);
    result.summary.board_height = Some(document.board_height);
    result.summary.komi = Some(document.komi);
    result.summary.handicap = document.handicap;
    result.summary.black_name = document.black_name;
    result.summary.white_name = document.white_name;
    result.summary.result = document.result;
    result.summary.move_count = Some(document.moves.len());
    Ok(result)
}

fn validate_timeout_ms(timeout_ms: Option<u64>, command_name: &str) -> Result<(), ProviderError> {
    if timeout_ms == Some(0) {
        return Err(ProviderError {
            kind: ProviderErrorKind::InvalidRequest,
            message: format!("{command_name} timeout_ms must be greater than zero"),
        });
    }
    Ok(())
}

fn readboard_error(error: readboard_sidecar::ReadboardSidecarError) -> ProviderError {
    ProviderError {
        kind: ProviderErrorKind::InvalidPayload,
        message: error.to_string(),
    }
}
#[tauri::command]
fn foreground_engine_snapshot(manager: State<'_, ForegroundEngineManager>) -> ForegroundEngineSnapshotDto {
    manager.snapshot()
}

#[tauri::command]
fn engine_diagnostic_snapshots(manager: State<'_, ForegroundEngineManager>) -> Vec<app_model::EngineDiagnosticSnapshotDto> {
    manager.diagnostic_snapshots()
}

#[tauri::command]
fn set_engine_diagnostic_trace(manager: State<'_, ForegroundEngineManager>, attempt_id: String, enabled: bool) -> Result<(), String> {
    manager.set_diagnostic_trace(&attempt_id, enabled)
}


#[tauri::command]
fn foreground_engine_start(
    manager: State<'_, ForegroundEngineManager>,
    profile_id: String,
) -> EngineCommandResult<()> {
    manager.start(&profile_id).map_err(Box::new)
}

#[tauri::command]
fn foreground_engine_stop(manager: State<'_, ForegroundEngineManager>) -> EngineCommandResult<()> {
    manager.stop().map_err(Box::new)
}

#[tauri::command]
fn foreground_engine_restart(manager: State<'_, ForegroundEngineManager>) -> EngineCommandResult<()> {
    manager.restart().map_err(Box::new)
}

#[tauri::command]
fn foreground_engine_switch(
    manager: State<'_, ForegroundEngineManager>,
    profile_id: String,
) -> EngineCommandResult<()> {
    manager.switch_to(&profile_id).map_err(Box::new)
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(handle_second_instance))
        .manage(FileActivationOwner::from_process())
        .manage(CurrentGameState::default())
        .manage(PreferencesState::default())
        .manage(MainWindowPin::default())
        .manage(provider_yike::sync::YikeSyncRuntime::default())
        .setup(|app| {
            readboard::install(app.handle());
            let recovery_path = session_recovery::recovery_file_path(app.handle())?;
            app.manage(Mutex::new(FileRecoveryStore::new(recovery_path)));
            spawn_recovery_writer(app.handle().clone());
            let _ = load_engine_profiles_from_disk(app.handle());
            app.manage(engine_manager::models::ModelInventory::new(
                app.path().app_data_dir()?.join("lizzieyzy-next-model-inventory.json"),
            ));
            let managed_root = app.path().app_data_dir()?.join("managed-resources");
            app.manage(engine_manager::managed::ManagedResources::new(managed_root.clone()));
            let catalog = std::sync::Arc::new(DiskEngineCatalog {
                handle: app.handle().clone(),
            });
            let manager = ForegroundEngineManager::new(catalog, ForegroundEngineConfig {
                managed_resources_root: Some(managed_root),
                ..ForegroundEngineConfig::default()
            });
            app.state::<CurrentGameState>()
                .connect_analysis_manager(manager.clone());
            let events = manager.subscribe();
            let emit_handle = app.handle().clone();
            std::thread::spawn(move || {
                while let Ok(event) = events.recv() {
                    match event {
                        ForegroundEngineEventDto::Snapshot { snapshot } => {
                            let _ = emit_handle.emit("foreground-engine://snapshot", snapshot);
                        }
                        ForegroundEngineEventDto::Failure { failure } => {
                            if let Some(update) = emit_handle
                                .state::<CurrentGameState>()
                                .observe_match_failure(&failure)
                            {
                                human_match::publish(&emit_handle, update);
                                human_match::drive(emit_handle.clone());
                            }
                            let _ = emit_handle.emit("foreground-engine://failure", failure);
                        }
                        ForegroundEngineEventDto::Job { mut job } => {
                            if let Some(state) = emit_handle.try_state::<CurrentGameState>() {
                                job.current_game = state.attach_from_job_event(&job);
                                if let Some(trial) = state.attach_trial_from_job_event(&job) {
                                    let _ = emit_handle.emit("trial://analysis", trial);
                                }
                            }
                            let _ = emit_handle.emit("foreground-engine://job", job);
                        }
                    }
                }
            });
            let _ = manager.apply_autoload();
            app.manage(manager);
            let preferences = app.state::<PreferencesState>();
            // The frontend load command owns recovery/error reporting. A failed
            // preference load must leave the window operable and pin uninitialized.
            if let Ok(path) = app_preferences_path(app.handle()) {
                if preferences
                    .load(&path, &app.state::<ForegroundEngineManager>())
                    .is_ok()
                {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = app
                            .state::<MainWindowPin>()
                            .apply(&window, &preferences, &path, None);
                    }
                }
            }
            window_geometry::start(app.handle());
            external_sync::start(app.handle())?;
            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            export::export_selected_line,
            export::export_rendered_image,
            external_sync::external_sync_snapshot,
            external_sync::load_yike_sync_preferences,
            external_sync::save_yike_sync_preferences,
            external_sync::begin_yike_sync,
            external_sync::prepare_yike_sync,
            external_sync::cancel_external_sync_start,
            external_sync::resolve_external_sync_start,
            external_sync::retry_external_sync,
            external_sync::stop_external_sync,
            external_sync::open_yike_sync_browser,
            window_geometry::window_geometry_status,
            window_geometry::reset_window_geometry,
            window_geometry::retry_window_geometry,
            window_geometry::flush_window_geometry,
            window_geometry::freeze_window_geometry,
            main_window_pin_status,
            set_main_window_pin,
            health,
            parse_sgf_summary,
            provider_import_from_payload,
            provider_yike_list,
            provider_yike_preview,
            load_yike_locator,
            save_yike_locator,
            provider_fox_list,
            provider_fox_list_more,
            provider_fox_preview,
            load_fox_kifu_state,
            remember_fox_lookup,
            clear_fox_recents,
            provider_tencent_list,
            provider_tencent_preview,
            load_tencent_history,
            save_tencent_query,
            network_snapshot,
            save_network_settings,
            begin_provider_request,
            cancel_provider_request,
            readboard::readboard_runtime_snapshot,
            readboard::readboard_runtime_path,
            readboard::readboard_save_runtime_path,
            readboard::readboard_runtime_start,
            readboard::readboard_runtime_stop,
            readboard::readboard_runtime_restart,
            readboard::load_readboard_sync_preferences,
            readboard::save_readboard_sync_preferences,
            readboard::begin_readboard_sync,
            readboard::prepare_readboard_sync,
            readboard::readboard_focus,
            readboard_sidecar_sync_snapshot,
            replay_sgf_positions,
            read_game_file,
            import_game_bytes,
            prepare_document_replacement,
            resolve_document_replacement,
            prepare_application_exit,
            resolve_application_exit,
            retry_application_teardown,
            confirm_application_exit_anyway,
            confirm_native_exit,
            inspect_current_game_recovery,
            restore_current_game_recovery,
            discard_current_game_recovery,
            retry_current_game_recovery,
            current_game_recovery_protection,
            take_initial_file_activation,
            take_pending_file_activation,
            mark_file_activation_ready,
            set_file_activation_busy,
            serialize_current_game,
            save_current_game,
            save_current_game_as,
            project_current_game_mainline,
            select_current_game_node,
            find_current_game_recorded_point,
            board_gesture_timing,
            play_current_game,
            author_current_game,
            set_current_game_personal_comment,
            set_current_game_metadata,
            edit_current_game_markup,
            remove_current_game_variation,
            promote_current_game_to_main,
            apply_root_setup,
            convert_to_root_setup,
            undo_current_game,
            redo_current_game,
            enter_trial,
            exit_trial,
            enter_scoring,
            update_scoring,
            exit_scoring,
            trial_snapshot,
            trial_select,
            trial_play,
            trial_undo,
            trial_start_finite,
            classify_problems,
            katago_launch_plan,
            engine_asset_checks,
            load_app_preferences,
            save_app_preferences,
            update_workspace_visibility,
            update_recent_game_history,
            update_workspace_shares,
            load_engine_profiles_settings,
            save_engine_profiles_settings,
            reorder_engine_profiles_settings,
            models::model_inventory_snapshot,
            models::refresh_model_inventory,
            models::select_installed_model,
            managed_resources::managed_resources_snapshot,
            managed_resources::acquire_managed_resources,
            managed_resources::cancel_managed_resources,
            katago_start_analyze_game,
            preview_analysis_scope,
            start_analysis_task,
            analysis_task_snapshot,
            pause_analysis_task,
            continue_analysis_task,
            katago_cancel_analysis,
            foreground_engine_snapshot,
            engine_diagnostic_snapshots,
            set_engine_diagnostic_trace,
            foreground_engine_start,
            foreground_engine_stop,
            foreground_engine_restart,
            foreground_engine_switch,
            foreground_engine_start_selected_node,
            foreground_engine_game_move,
            foreground_engine_confirm_rules,
            human_match::human_match_snapshot,
            human_match::human_match_start,
            human_match::human_match_action,
            human_match::human_match_stop,
            human_match::pk_match_start,
            human_match::pk_match_pause,
            human_match::pk_match_resume,
            human_match::human_match_analysis_policy,
            foreground_engine_cancel_game_move,
            foreground_engine_continuous_action,
            foreground_engine_cancel_job
        ])
        .build(tauri::generate_context!())
        .expect("failed to build LizzieYzy Next")
        .run(|app, event| match event {
            tauri::RunEvent::WindowEvent {
                label,
                event: tauri::WindowEvent::DragDrop(tauri::DragDropEvent::Drop { paths, .. }),
                ..
            } if label == "main" => handle_file_drop(app, paths),

            tauri::RunEvent::WindowEvent { label, event, .. }
                if label == "main"
                    && matches!(
                        event,
                        tauri::WindowEvent::Moved(_)
                            | tauri::WindowEvent::Resized(_)
                            | tauri::WindowEvent::ScaleFactorChanged { .. }
                    ) =>
            {
                window_geometry::observe(app);
            }

            tauri::RunEvent::WindowEvent {
                label,
                event: tauri::WindowEvent::CloseRequested { api, .. },
                ..
            } => {
                api.prevent_close();
                if let Some(window) = app.get_webview_window(&label) {
                    let _ = window.emit(APPLICATION_EXIT_REQUESTED_EVENT, ());
                }
            }
            tauri::RunEvent::Exit => {
                if let Some(manager) = app.try_state::<ForegroundEngineManager>() {
                    let _ = manager.teardown();
                }
            }
            _ => {}
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_gateway_read_preserves_file_and_surfaces_corruption() {
        let directory = std::env::temp_dir().join(format!("legacy-profile-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("legacy.json");
        let original = r#"{"profile":{"name":"围棋 引擎","engine_path":" D:\\引擎 空格\\katago.exe ","model_path":"模型.bin","config_path":"配置.cfg","working_dir":null,"backend":"kata_go_analysis"},"max_visits":123}"#;
        std::fs::write(&path, original).unwrap();
        let loaded = load_legacy_engine_profile_settings(&path).unwrap();
        assert_eq!(loaded.profiles[0].profile.program, " D:\\引擎 空格\\katago.exe ");
        assert_eq!(loaded.profiles[0].profile.name, "围棋 引擎");
        assert_eq!(std::fs::read_to_string(&path).unwrap(), original);
        std::fs::write(&path, "{broken").unwrap();
        let failure = load_legacy_engine_profile_settings(&path).unwrap_err();
        assert!(failure.contains(&path.display().to_string()));
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "{broken");
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn gateway_save_validation_and_write_errors_preserve_catalog_and_runtime() {
        let directory = std::env::temp_dir().join(format!("profile-save-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("catalog.json");
        let current = default_engine_profiles_settings();
        persist_engine_profiles(&path, current.clone()).unwrap();
        let before = std::fs::read(&path).unwrap();
        let catalog = std::sync::Arc::new(engine_manager::InMemoryEngineProfileCatalog::new());
        let manager = ForegroundEngineManager::new(catalog, ForegroundEngineConfig::for_tests());
        let run_before = manager.snapshot();
        let mut invalid = current.clone();
        invalid.profiles[0].profile.argv = vec!["-model=other".into()];
        assert!(save_engine_profiles_at_path(&path, &manager, &current, invalid).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), before);
        assert_eq!(engine_manager::load_engine_profiles(&path).unwrap(), current);
        assert_eq!(manager.snapshot(), run_before);
        let mut next = current.clone();
        next.profiles[0].profile.name = "edited name".into();
        std::fs::create_dir(path.with_extension("json.tmp")).unwrap();
        assert!(save_engine_profiles_at_path(&path, &manager, &current, next).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), before);
        assert_eq!(engine_manager::load_engine_profiles(&path).unwrap(), current);
        assert_eq!(manager.snapshot(), run_before);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn gateway_order_preserves_late_edits_and_runtime_and_rejects_late_consumers() {
        let directory = std::env::temp_dir().join(format!("profile-order-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("catalog.json");
        let mut current = default_engine_profiles_settings();
        let mut second = current.profiles[0].clone();
        second.id = "second".into();
        second.profile.name = "Second".into();
        current.profiles.push(second);
        current.autoload_profile_id = Some("second".into());
        let request = app_model::EngineProfileOrderRequestDto {
            expected_profile_ids: vec!["default".into(), "second".into()],
            profile_ids: vec!["second".into(), "default".into()],
        };
        current.profiles[0].profile.name = "Late saved edit".into();
        persist_engine_profiles(&path, current.clone()).unwrap();
        let catalog = std::sync::Arc::new(engine_manager::InMemoryEngineProfileCatalog::new());
        let manager = ForegroundEngineManager::new(catalog, ForegroundEngineConfig::for_tests());
        let runtime = manager.snapshot();
        let reordered = reorder_engine_profiles_at_path(
            &path,
            engine_manager::load_engine_profiles(&path).unwrap(),
            &request,
        )
        .unwrap();
        assert_eq!(reordered.selected_profile_id, current.selected_profile_id);
        assert_eq!(reordered.autoload_profile_id, current.autoload_profile_id);
        assert_eq!(reordered.profiles[1], current.profiles[0]);
        assert_eq!(manager.snapshot(), runtime);
        let before = std::fs::read(&path).unwrap();
        assert!(
            reorder_engine_profiles_at_path(&path, reordered.clone(), &request)
                .unwrap_err()
                .contains("stale")
        );
        std::fs::create_dir(path.with_extension("json.tmp")).unwrap();
        let reverse = app_model::EngineProfileOrderRequestDto {
            expected_profile_ids: request.profile_ids,
            profile_ids: request.expected_profile_ids,
        };
        assert!(reorder_engine_profiles_at_path(&path, reordered.clone(), &reverse).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), before);
        assert_eq!(engine_manager::load_engine_profiles(&path).unwrap(), reordered);
        assert_eq!(manager.snapshot(), runtime);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn gateway_late_full_save_keeps_committed_order_and_appends_new_profiles() {
        let directory = std::env::temp_dir().join(format!("profile-late-save-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("catalog.json");
        let mut stale = default_engine_profiles_settings();
        let mut second = stale.profiles[0].clone();
        second.id = "second".into();
        stale.profiles.push(second);
        let mut current = stale.clone();
        current.profiles.reverse();
        persist_engine_profiles(&path, current.clone()).unwrap();
        stale.profiles[0].profile.name = "Late editor save".into();
        let mut new_record = stale.profiles[0].clone();
        new_record.id = "new".into();
        stale.profiles.insert(0, new_record);
        let manager = ForegroundEngineManager::new(
            std::sync::Arc::new(engine_manager::InMemoryEngineProfileCatalog::new()),
            ForegroundEngineConfig::for_tests(),
        );
        let saved = save_engine_profiles_at_path(&path, &manager, &current, stale).unwrap();
        assert_eq!(
            saved
                .profiles
                .iter()
                .map(|record| record.id.as_str())
                .collect::<Vec<_>>(),
            ["second", "default", "new"]
        );
        assert_eq!(saved.profiles[1].profile.name, "Late editor save");
        assert_eq!(engine_manager::load_engine_profiles(&path).unwrap(), saved);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn rejected_edit_keeps_continuous_job_admitted() {
        use std::os::unix::fs::PermissionsExt;
        use std::sync::Arc;
        use std::time::Duration;
        let directory = std::env::temp_dir().join(format!("continuous-edit-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&directory).unwrap();
        let executable = directory.join("engine");
        std::fs::write(
            &executable,
            r##"#!/usr/bin/env python3
import json, sys
for line in sys.stdin:
    q = json.loads(line)
    if q.get("action") == "terminate":
        assert "terminateId" in q and q["id"] != q["terminateId"]
        print(json.dumps(dict(id=q["terminateId"], isDuringSearch=False, noResults=True)), flush=True)
    else:
        print(json.dumps(dict(id=q["id"], isDuringSearch=True, turnNumber=0,
            rootInfo=dict(visits=8, winrate=0.6, scoreMean=2.5),
            moveInfos=[dict(move="E5", visits=8, winrate=0.6, scoreMean=2.5)])), flush=True)
"##,
        )
        .unwrap();
        std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o755)).unwrap();
        std::fs::write(directory.join("model"), "").unwrap();
        std::fs::write(directory.join("config"), "").unwrap();
        let catalog = Arc::new(engine_manager::InMemoryEngineProfileCatalog::new());
        catalog.upsert(SavedEngineProfile {
            profile_id: "test".into(),
            profile: EngineProfileDto {
                name: "test".into(),
                program: executable.to_string_lossy().into(),
                argv: vec![],
                working_dir: Some(directory.to_string_lossy().into()),
                adapter: app_model::EngineAdapterSettings::KataGoAnalysis(app_model::KataGoSettings {
                    model_path: Some("model".into()),
                    config_path: Some("config".into()),
                    max_visits: 800,
                }),
            },
        });
        let manager =
            ForegroundEngineManager::new(catalog, engine_manager::ForegroundEngineConfig::for_tests());
        let events = manager.subscribe();
        manager.start("test").unwrap();
        let run_id = loop {
            if let ForegroundEngineEventDto::Snapshot { snapshot } =
                events.recv_timeout(Duration::from_secs(2)).unwrap()
            {
                if let app_model::ForegroundEngineLifecycleDto::Ready { run } = snapshot.lifecycle {
                    break run.run_id;
                }
            }
        };
        let state = CurrentGameState::default();
        state.connect_analysis_manager(manager.clone());
        let opened = state.replace("(;SZ[9]AB[dd])", None).unwrap();
        let started = manager
            .start_selected_node_job(
                bind_selected_node_job(
                    &state,
                    run_id,
                    opened.generation,
                    opened.selected_path.clone(),
                    AnalysisJobModeDto::Continuous,
                    None,
                )
                .unwrap(),
            )
            .unwrap();
        let rejected = state.play(
            opened.selected_path,
            MoveVertex::Point(app_model::PointDto { x: 3, y: 3 }),
        );
        let job = manager.snapshot().selected_node_job.unwrap();
        manager.teardown().unwrap();
        std::fs::remove_dir_all(directory).unwrap();
        assert!(rejected.is_err());
        assert_eq!(job.job_id, started.job_id);
        assert_ne!(job.state, app_model::AnalysisJobStateDto::Stopping);
    }

    const BRANCHING: &str = include_str!("../../../../tests/golden/editable-workspace-branching.sgf");

    #[test]
    fn whole_game_work_items_capture_first_child_positions_rules_and_policy() {
        let state = CurrentGameState::default();
        let opened = state.replace(BRANCHING, None).unwrap();
        let admitted = state.admit_whole_game(opened.generation).unwrap();
        let items = whole_game_work_items(&admitted, 64, "run-1").unwrap();
        assert_eq!(items.len(), 4);
        assert!(items[0].node_path.indices.is_empty());
        assert_eq!(items[1].node_path.indices, vec![0]);
        assert_eq!(items[2].node_path.indices, vec![0, 0]);
        assert_eq!(items[3].node_path.indices, vec![0, 0, 0]);
        assert_eq!(items[0].move_number, 0);
        assert_eq!(items[3].move_number, 3);
        let root: serde_json::Value =
            serde_json::from_str(items[0].query.to_jsonl().unwrap().trim()).unwrap();
        assert_eq!(root["includeOwnership"], true);
        assert_eq!(root["includePolicy"], true);
        assert_eq!(root["rules"], "chinese");
        assert!((root["komi"].as_f64().unwrap() - 0.5).abs() < f64::EPSILON);
        assert_eq!(root["boardXSize"], 5);
        assert_eq!(root["boardYSize"], 5);
        let stones = root["initialStones"].as_array().expect("setup stones");
        assert!(stones
            .iter()
            .any(|stone| stone == &serde_json::json!(["B", "A5"])));
        assert!(stones
            .iter()
            .any(|stone| stone == &serde_json::json!(["W", "C3"])));
        assert!(!stones
            .iter()
            .any(|stone| stone == &serde_json::json!(["B", "A2"])));
        assert_eq!(items[0].query.analyze_turns, Some(vec![1]));
        assert_eq!(items[1].query.analyze_turns, Some(vec![0]));
    }

    #[test]
    fn whole_game_work_items_use_current_game_owned_rules_and_dimensions() {
        let state = CurrentGameState::default();
        let opened = state
            .replace("(;GM[1]FF[4]SZ[9:13]KM[6.5]RU[Japanese];B[di])", None)
            .unwrap();
        let admitted = state.admit_whole_game(opened.generation).unwrap();
        let items = whole_game_work_items(&admitted, 32, "run-rules").unwrap();
        assert_eq!(items.len(), 2);
        for item in items {
            assert_eq!(item.board_width, 9);
            assert_eq!(item.board_height, 13);
            assert_eq!(item.query.board_x_size, 9);
            assert_eq!(item.query.board_y_size, 13);
            assert_eq!(item.query.rules, "japanese");
        }
    }

    fn registered_tauri_commands(source: &str) -> Vec<&str> {
        let list = source
            .split("tauri::generate_handler![")
            .nth(1)
            .and_then(|rest| rest.split(']').next())
            .expect("tauri invoke handler list");
        list.lines()
            .map(str::trim)
            .map(|line| line.trim_end_matches(','))
            .filter(|line| !line.is_empty())
            .collect()
    }

    #[test]
    fn native_close_and_file_exit_share_application_exit_commands() {
        let source = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/lib.rs"));
        let commands = registered_tauri_commands(source);
        for command in [
            "prepare_application_exit",
            "resolve_application_exit",
            "retry_application_teardown",
            "confirm_application_exit_anyway",
            "confirm_native_exit",
        ] {
            assert!(
                commands.contains(&command),
                "{command} must be registered: {commands:?}"
            );
        }
        assert!(source.contains("CloseRequested"));
        assert!(source.contains("prevent_close"));
        assert!(source.contains("APPLICATION_EXIT_REQUESTED_EVENT"));
        assert!(source.contains("prepare_application_exit"));
    }

    #[test]
    fn whole_game_gateway_drops_legacy_event_bus_and_stays_registered() {
        let source = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/lib.rs"));
        for forbidden in [
            concat!("katago://", "analysis-progress"),
            concat!("katago://", "analysis-complete"),
            concat!("katago://", "analysis-error"),
            concat!("katago://", "analysis-cancelled"),
            concat!("forward_whole_game_job", "_events"),
        ] {
            assert!(
                !source.contains(forbidden),
                "legacy whole-game event bus must not remain: {forbidden}"
            );
        }
        let commands = registered_tauri_commands(source);
        assert!(
            commands.contains(&"katago_start_analyze_game"),
            "whole-game analysis must stay on the manager-owned run command"
        );
    }

    #[test]
    fn selected_node_request_captures_exact_position_turn_rules_komi_and_dimensions_before_protocol() {
        let state = CurrentGameState::default();
        let opened = state
            .replace("(;GM[1]FF[4]SZ[5:7]KM[6.5]RU[Japanese];B[cc];W[eg])", None)
            .unwrap();
        let path = NodePath { indices: vec![0, 0] };
        let selected = state.select_path(path.clone(), opened.generation).unwrap();
        assert_eq!(selected.snapshot.position.move_number, 2);
        assert_eq!(selected.snapshot.position.to_play, app_model::PlayerColor::Black);

        let request = bind_selected_node_job(
            &state,
            "run-ready".to_string(),
            opened.generation,
            path.clone(),
            AnalysisJobModeDto::Finite,
            Some(64),
        )
        .unwrap();

        assert_eq!(request.run_id, "run-ready");
        assert_eq!(request.generation, opened.generation);
        assert_eq!(request.node_path, path);
        assert_eq!(request.mode, AnalysisJobModeDto::Finite);
        assert_eq!(request.board_width, 5);
        assert_eq!(request.board_height, 7);
        assert_eq!(request.query.board_x_size, 5);
        assert_eq!(request.query.board_y_size, 7);
        assert_eq!(request.query.komi, 6.5);
        assert_eq!(request.query.rules, "japanese");
        assert_eq!(request.query.include_ownership, Some(true));
        assert_eq!(request.query.include_policy, Some(true));
        assert_eq!(request.query.max_visits, Some(64));
        assert_eq!(
            request.query.initial_stones,
            vec![
                ("B".to_string(), "C5".to_string()),
                ("W".to_string(), "E1".to_string()),
            ]
        );
        assert!(request.query.moves.is_empty());
        assert_eq!(request.query.analyze_turns, Some(vec![0]));
    }

    #[test]
    fn selected_node_illegal_path_is_typed_invalid_state_before_protocol() {
        let state = CurrentGameState::default();
        let opened = state
            .replace("(;GM[1]FF[4]SZ[5]KM[6.5]RU[Japanese];B[cc])", None)
            .unwrap();
        let error = match bind_selected_node_job(
            &state,
            "run-ready".to_string(),
            opened.generation,
            NodePath { indices: vec![9] },
            AnalysisJobModeDto::Finite,
            Some(8),
        ) {
            Err(error) => error,
            Ok(_) => panic!("illegal path must fail before protocol"),
        };
        assert_eq!(error.kind, EngineFailureKind::InvalidState);
        assert_eq!(error.operation, EngineOperationDto::Job);
        assert_eq!(error.run_id.as_deref(), Some("run-ready"));
        assert!(error.job_id.is_none());
        assert_eq!(error.message, "invalid node path");
    }

    #[test]
    fn selected_node_stale_generation_is_typed_invalid_state_before_protocol() {
        let state = CurrentGameState::default();
        let opened = state
            .replace("(;GM[1]FF[4]SZ[5]KM[6.5]RU[Japanese];B[cc])", None)
            .unwrap();
        let error = match bind_selected_node_job(
            &state,
            "run-ready".to_string(),
            opened.generation + 1,
            opened.selected_path,
            AnalysisJobModeDto::Finite,
            Some(8),
        ) {
            Err(error) => error,
            Ok(_) => panic!("stale generation must fail before protocol"),
        };
        assert_eq!(error.kind, EngineFailureKind::InvalidState);
        assert_eq!(error.message, "current game generation does not match");
        assert!(error.job_id.is_none());
    }

    #[test]
    fn continuous_selected_node_uses_exact_admission_without_finite_visit_budget() {
        let state = CurrentGameState::default();
        let opened = state
            .replace("(;GM[1]FF[4]SZ[5]KM[6.5]RU[Japanese];B[cc])", None)
            .unwrap();
        let request = bind_selected_node_job(
            &state,
            "run-ready".to_string(),
            opened.generation,
            opened.selected_path,
            AnalysisJobModeDto::Continuous,
            None,
        )
        .unwrap();

        assert_eq!(request.mode, AnalysisJobModeDto::Continuous);
        assert_eq!(request.query.max_visits, None);
        assert_eq!(request.query.include_ownership, Some(true));
        assert_eq!(request.query.include_policy, Some(true));
    }

    #[test]
    fn provider_enrichment_projects_rectangular_dimensions() {
        let enriched = enrich_provider_import_result(ProviderImportResult {
            provider: ProviderKind::Yike,
            sgf_text: "(;GM[1]FF[4]SZ[9:13]KM[7.5])".to_string(),
            summary: app_model::ProviderGameSummary {
                provider: ProviderKind::Yike,
                source_id: None,
                board_width: None,
                board_height: None,
                komi: None,
                handicap: None,
                black_name: None,
                white_name: None,
                result: None,
                date: None,
                move_count: None,
            },
            metadata: ProviderGameMetadata::default(),
            warnings: Vec::new(),
        })
        .unwrap();

        assert_eq!(enriched.summary.board_width, Some(9));
        assert_eq!(enriched.summary.board_height, Some(13));
    }

    #[test]
    fn readboard_sidecar_sync_snapshot_supports_offline_protocol_line() {
        let result = readboard_sidecar_sync_snapshot(ReadboardSidecarSyncSnapshotRequest {
            endpoint: None,
            snapshot_id: Some("snapshot-1".to_string()),
            image_path: None,
            image_base64: None,
            sgf_text: Some("snapshot board_size=2 move_number=1 codes=3000".to_string()),
            metadata: std::collections::BTreeMap::new(),
            timeout_ms: Some(100),
        })
        .unwrap();

        assert_eq!(result.snapshot_id, "snapshot-1");
        let position = result.position.unwrap();
        assert_eq!(position.board_width, 2);
        assert_eq!(position.board_height, 2);
        assert_eq!(position.move_number, 1);
        assert_eq!(position.stones.len(), 1);
    }

    #[test]
    fn readboard_sidecar_sync_snapshot_reports_image_runtime_unavailable() {
        let sync_error = readboard_sidecar_sync_snapshot(ReadboardSidecarSyncSnapshotRequest {
            endpoint: Some("http://127.0.0.1:39081".to_string()),
            snapshot_id: Some("snapshot-1".to_string()),
            image_path: Some("/tmp/board.png".to_string()),
            image_base64: None,
            sgf_text: None,
            metadata: std::collections::BTreeMap::new(),
            timeout_ms: Some(100),
        })
        .unwrap_err();

        assert_eq!(sync_error.kind, ProviderErrorKind::RuntimeUnavailable);
        assert!(sync_error
            .message
            .contains("readboard image OCR runtime is unavailable"));
    }

    #[test]
    fn game_file_import_routes_formats_and_keeps_gib_read_only() {
        let directory = std::env::temp_dir().join(format!("game-file-import-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&directory).unwrap();
        let sgf_path = directory.join("named.sgf");
        std::fs::write(&sgf_path, "(;SZ[9]PB[Black]PW[White];B[dd])").unwrap();
        let sgf_import = read_game_file(sgf_path.to_string_lossy().into_owned()).unwrap();
        assert_eq!(sgf_import.format, app_model::GameFileFormatDto::Sgf);
        assert_eq!(sgf_import.display_name, "named.sgf");
        assert_eq!(
            sgf_import.native_path.as_deref(),
            std::fs::canonicalize(&sgf_path).unwrap().to_str()
        );
        assert_eq!(sgf_import.sgf_text, "(;SZ[9]PB[Black]PW[White];B[dd])");

        let gib_path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../tests/fixtures/tygem-named-pass-lf.gib");
        let original = std::fs::read(&gib_path).unwrap();
        let gib_import = read_game_file(gib_path.to_string_lossy().into_owned()).unwrap();
        assert_eq!(gib_import.format, app_model::GameFileFormatDto::Gib);
        assert_eq!(gib_import.display_name, "tygem-named-pass-lf.gib");
        assert_eq!(gib_import.native_path, None);
        assert_eq!(
            gib_import.opened_path.as_deref(),
            std::fs::canonicalize(&gib_path).unwrap().to_str()
        );
        assert_eq!(std::fs::read(&gib_path).unwrap(), original);
        let imported = sgf::CurrentSgfDocument::open(&gib_import.sgf_text).unwrap();
        assert_eq!(imported.default_selected_path().indices, vec![0, 0, 0]);

        let uploaded = import_game_bytes("uploaded.gib".to_string(), original.clone()).unwrap();
        assert_eq!(uploaded.format, app_model::GameFileFormatDto::Gib);
        assert_eq!(uploaded.display_name, "uploaded.gib");
        assert_eq!(uploaded.native_path, None);
        assert_eq!(uploaded.opened_path, None);
        assert_eq!(uploaded.sgf_text, gib_import.sgf_text);

        let unsupported = directory.join("unsupported.ngf");
        std::fs::write(&unsupported, "not supported").unwrap();
        assert!(read_game_file(unsupported.to_string_lossy().into_owned())
            .unwrap_err()
            .contains("unsupported game file"));
        assert!(
            read_game_file(directory.join("missing.sgf").to_string_lossy().into_owned())
                .unwrap_err()
                .contains("failed to read game file")
        );
        std::fs::remove_dir_all(directory).unwrap();
    }
}
