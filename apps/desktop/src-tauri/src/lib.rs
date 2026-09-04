use app_model::{
    AnalysisFrameDto, AnalysisJobStartedDto, AppHealthDto, CandidateMoveDto, CurrentGameError,
    CurrentGameResultDto, EngineBackend, EngineFailureDto, EngineFailureKind, EngineOperationDto,
    EngineProfileDto, ForegroundEngineEventDto, ForegroundEngineSnapshotDto, MoveVertex, NodePath, PointDto,
    PositionDto, ProviderError, ProviderErrorKind, ProviderFetchMethod, ProviderFetchRequest,
    ProviderFetchResult, ProviderGameMetadata, ProviderImportRequest, ProviderImportResult, ProviderKind,
    ReadboardSidecarProbeRequest, ReadboardSidecarProbeResult, ReadboardSidecarSyncSnapshotRequest,
    ReadboardSidecarSyncSnapshotResult,
};
use engine_manager::{
    build_command_spec, check_assets, default_engine_profiles_settings, normalize_engine_profiles,
    parse_engine_profiles, save_engine_profiles as persist_engine_profiles, AssetCheck, CommandSpec,
    EngineProfileCatalog, EngineProfileRecord as EngineProfileRecordDto,
    EngineProfilesSettings as EngineProfilesSettingsDto, ForegroundEngineConfig, ForegroundEngineManager,
    SavedEngineProfile, SelectedNodeJobRequest, WholeGameJobRequest, WholeGameWorkItem,
    DEFAULT_ENGINE_PROFILE_ID,
};
use go_core::ReadBoardLocalContext;
use katago_protocol::{analysis_query_from_position, AnalysisQueryOptions};
use provider_core::{
    invalid_payload, invalid_request, invalid_url, timeout, transport_failed, ProviderResult,
    ProviderTransport,
};
use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager, State};

mod current_game_state;
mod save_as;
#[cfg(windows)]
extern crate windows_core;
use app_preferences::{
    load_from_path as load_app_preferences_from_path, save_to_path, AppPreferencesDto,
    AppPreferencesLoadResultDto, APP_PREFERENCES_FILE,
};
use current_game_state::{CurrentGameState, WholeGameAdmission};
use uuid::Uuid;

const ENGINE_PROFILE_FILE: &str = "lizzieyzy-next-engine-profile.json";
const ANALYSIS_CACHE_DB_FILE: &str = "analysis-cache.sqlite3";
const DEFAULT_PROVIDER_HTTP_TIMEOUT_MS: u64 = 30_000;

#[derive(Debug, Default)]
struct ReqwestProviderTransport;

impl ProviderTransport for ReqwestProviderTransport {
    fn fetch(&self, request: &ProviderFetchRequest) -> ProviderResult<ProviderFetchResult> {
        if !is_http_url(&request.url) {
            return Err(invalid_url(format!(
                "provider transport only supports http(s) URLs: {}",
                request.url
            )));
        }
        let client = reqwest::blocking::Client::builder()
            .redirect(reqwest::redirect::Policy::limited(10))
            .build()
            .map_err(map_reqwest_error)?;
        let method = match request.method {
            ProviderFetchMethod::Get => reqwest::Method::GET,
            ProviderFetchMethod::Post => reqwest::Method::POST,
        };
        let mut builder = client
            .request(method, &request.url)
            .timeout(Duration::from_millis(
                request.timeout_ms.unwrap_or(DEFAULT_PROVIDER_HTTP_TIMEOUT_MS),
            ));
        for (name, value) in &request.headers {
            builder = builder.header(
                reqwest::header::HeaderName::from_bytes(name.as_bytes()).map_err(|err| {
                    invalid_request(format!("invalid provider request header `{name}`: {err}"))
                })?,
                reqwest::header::HeaderValue::from_str(value).map_err(|err| {
                    invalid_request(format!(
                        "invalid provider request header value for `{name}`: {err}"
                    ))
                })?,
            );
        }
        if let Some(body) = &request.body {
            builder = builder.body(body.clone());
        }

        let response = builder.send().map_err(map_reqwest_error)?;
        let url = response.url().to_string();
        let status_code = response.status().as_u16();
        let headers = response_headers(response.headers());
        let content_type = headers
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case("content-type"))
            .map(|(_, value)| value.clone());
        let payload = response.text().map_err(map_reqwest_error)?;

        Ok(ProviderFetchResult {
            provider: request.provider,
            url,
            status_code,
            payload,
            headers,
            content_type,
            metadata: ProviderGameMetadata {
                source_url: request.source_url.clone(),
                request_url: Some(request.url.clone()),
                source_id: request.source_id.clone(),
                ..ProviderGameMetadata::default()
            },
            warnings: Vec::new(),
        })
    }
}

fn map_reqwest_error(error: reqwest::Error) -> ProviderError {
    if error.is_timeout() {
        return timeout(format!("provider request timed out: {error}"));
    }
    if error.is_builder() || error.is_request() {
        return invalid_request(format!("provider request could not be built: {error}"));
    }
    transport_failed(format!("provider request failed: {error}"))
}

fn response_headers(headers: &reqwest::header::HeaderMap) -> BTreeMap<String, String> {
    headers
        .iter()
        .filter_map(|(name, value)| {
            value
                .to_str()
                .ok()
                .map(|value| (name.as_str().to_string(), value.to_string()))
        })
        .collect()
}

fn prepare_yike_fetch_request(mut request: ProviderFetchRequest) -> ProviderFetchRequest {
    let signature = provider_yike::YikeRequestSignature::now();
    let signed_headers = provider_yike::signed_headers(signature.current_time_millis, signature.nonce);
    for (name, value) in signed_headers {
        insert_header_if_missing(&mut request.headers, &name, value);
    }
    request
}

fn fetch_yike_with_transport<T: ProviderTransport + ?Sized>(
    request: ProviderFetchRequest,
    transport: &T,
) -> ProviderResult<ProviderFetchResult> {
    let result = transport.fetch(&prepare_yike_fetch_request(request))?;
    ensure_provider_http_success(&result, "Yike provider fetch failed")?;
    validate_yike_fetch_payload(&result)?;
    Ok(result)
}

fn validate_yike_fetch_payload(result: &ProviderFetchResult) -> ProviderResult<()> {
    let url = result.url.to_ascii_lowercase();
    let request_url = result
        .metadata
        .request_url
        .as_deref()
        .unwrap_or_default()
        .to_ascii_lowercase();
    if url.contains("/golive/list") || request_url.contains("/golive/list") {
        provider_yike::parse_live_list_json(&result.payload).map(|_| ())
    } else if url.contains("/golives/")
        || url.contains("/golive/dtl")
        || request_url.contains("/golives/")
        || request_url.contains("/golive/dtl")
    {
        provider_yike::parse_live_detail_json(&result.payload).map(|_| ())
    } else {
        Err(invalid_payload(format!(
            "unsupported Yike fetch response URL for runtime validation: {}",
            result.url
        )))
    }
}

fn prepare_fox_http_fetch_request(mut request: ProviderFetchRequest) -> ProviderFetchRequest {
    insert_header_if_missing(
        &mut request.headers,
        "User-Agent",
        provider_fox::FOX_MOBILE_USER_AGENT.to_string(),
    );
    request
}

fn fetch_fox_with_transport<T: ProviderTransport + ?Sized>(
    request: ProviderFetchRequest,
    transport: &T,
) -> ProviderResult<ProviderFetchResult> {
    if is_http_url(&request.url) {
        let mut result = transport.fetch(&prepare_fox_http_fetch_request(request))?;
        ensure_provider_http_success(&result, "Fox provider fetch failed")?;
        result.warnings.push(
            "Fox HTTP URL fetched directly; provider command normalization was not applied.".to_string(),
        );
        return Ok(result);
    }
    provider_fox::fetch_command(&request.url, transport)
}

fn ensure_provider_http_success(result: &ProviderFetchResult, context: &str) -> ProviderResult<()> {
    if !(200..400).contains(&result.status_code) {
        return Err(transport_failed(format!(
            "{context}: HTTP {} for {}",
            result.status_code, result.url
        )));
    }
    Ok(())
}

fn insert_header_if_missing(headers: &mut BTreeMap<String, String>, name: &str, value: String) {
    if !headers.keys().any(|key| key.eq_ignore_ascii_case(name)) {
        headers.insert(name.to_string(), value);
    }
}

fn is_http_url(url: &str) -> bool {
    reqwest::Url::parse(url)
        .map(|url| matches!(url.scheme(), "http" | "https"))
        .unwrap_or(false)
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EngineProfileSettingsDto {
    profile: EngineProfileDto,
    max_visits: u32,
}

#[derive(Debug, Clone, Serialize)]
struct ComputeGameCacheKeyDto {
    game_key: String,
    sgf_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AnalysisCacheRecordDto {
    id: String,
    game_key: String,
    sgf_hash: String,
    profile_id: Option<String>,
    engine_kind: Option<String>,
    source: String,
    move_count: u32,
    analyzed_move_count: u32,
    payload: Value,
    created_at: Option<String>,
    updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
struct GetAnalysisCacheDto {
    status: String,
    record: Option<AnalysisCacheRecordDto>,
    error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct SaveAnalysisCacheDto {
    id: String,
    game_key: String,
    updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
struct DeleteAnalysisCacheDto {
    deleted: usize,
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
fn provider_parse_yike_url(raw_url: String) -> Result<provider_yike::YikeUrlDescriptor, ProviderError> {
    provider_yike::parse_yike_url(&raw_url)
}

#[tauri::command]
fn provider_import_from_payload(
    request: ProviderImportRequest,
) -> Result<ProviderImportResult, ProviderError> {
    let result = match request.provider {
        ProviderKind::Yike => provider_yike::import_payload(request),
        ProviderKind::Fox => provider_fox::import_payload(request),
    }?;
    enrich_provider_import_result(result)
}

#[tauri::command]
fn provider_fetch_yike(request: ProviderFetchRequest) -> Result<ProviderFetchResult, ProviderError> {
    validate_provider_fetch_request(&request, ProviderKind::Yike, "provider_fetch_yike")?;
    let transport = ReqwestProviderTransport;
    fetch_yike_with_transport(request, &transport)
}

#[tauri::command]
fn provider_fetch_fox(request: ProviderFetchRequest) -> Result<ProviderFetchResult, ProviderError> {
    validate_provider_fetch_request(&request, ProviderKind::Fox, "provider_fetch_fox")?;
    let transport = ReqwestProviderTransport;
    fetch_fox_with_transport(request, &transport)
}

#[tauri::command]
fn readboard_sidecar_probe(
    request: ReadboardSidecarProbeRequest,
) -> Result<ReadboardSidecarProbeResult, ProviderError> {
    validate_timeout_ms(request.timeout_ms, "readboard_sidecar_probe")?;
    Ok(readboard_sidecar::probe_readboard_sidecar(
        &request,
        &readboard_sidecar::ReadboardSidecarOptions::default(),
    )
    .into_dto())
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
        let parsed = readboard_sidecar::parse_snapshot_line(protocol_line).map_err(readboard_error)?;
        let local = ReadBoardLocalContext {
            board_size: parsed.snapshot.board_size,
            positions: Vec::new(),
            current_index: 0,
            main_end_index: 0,
        };
        let first_sync = request
            .metadata
            .get("first_sync")
            .map(|value| value != "false" && value != "0")
            .unwrap_or(true);
        return readboard_sidecar::sync_snapshot_line(&request, protocol_line, first_sync, local)
            .map(|outcome| outcome.into_dto())
            .map_err(readboard_error);
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
fn read_sgf_file(path: String) -> Result<String, String> {
    let path = non_empty_path(path)?;
    fs::read_to_string(&path).map_err(|err| format!("failed to read SGF file {}: {err}", path.display()))
}

#[tauri::command]
fn replace_current_game(
    state: State<CurrentGameState>,
    sgf_text: String,
    native_path: Option<String>,
) -> Result<CurrentGameResultDto, CurrentGameError> {
    state.replace(&sgf_text, native_path)
}

#[tauri::command]
fn serialize_current_game(state: State<CurrentGameState>) -> Result<String, CurrentGameError> {
    state.serialize()
}

#[tauri::command]
fn save_current_game(
    state: State<CurrentGameState>,
    path: String,
    selected_path: NodePath,
) -> Result<CurrentGameResultDto, String> {
    state.save_to_path(path, selected_path)
}

#[tauri::command]
async fn save_current_game_as(
    app: AppHandle,
    state: State<'_, CurrentGameState>,
    selected_path: NodePath,
    default_file_name: Option<String>,
) -> Result<Option<CurrentGameResultDto>, String> {
    let default_file_name = default_file_name.unwrap_or_else(|| "review.sgf".to_string());
    let app = app.clone();
    let outcome =
        tauri::async_runtime::spawn_blocking(move || save_as::pick_save_as_outcome(&app, &default_file_name))
            .await
            .map_err(|error| error.to_string())??;
    save_as::persist_current_game_save_as(&state, outcome, selected_path)
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
) -> Result<CurrentGameResultDto, CurrentGameError> {
    state.select_path(path)
}

#[tauri::command]
fn play_current_game(
    state: State<CurrentGameState>,
    path: NodePath,
    vertex: MoveVertex,
) -> Result<CurrentGameResultDto, CurrentGameError> {
    state.play(path, vertex)
}

#[tauri::command]
fn set_current_game_personal_comment(
    state: State<CurrentGameState>,
    path: NodePath,
    comment: String,
) -> Result<CurrentGameResultDto, CurrentGameError> {
    state.set_personal_comment(path, comment)
}

#[tauri::command]
fn remove_current_game_variation(
    state: State<CurrentGameState>,
    path: NodePath,
) -> Result<CurrentGameResultDto, CurrentGameError> {
    state.remove_variation(path)
}

#[tauri::command]
fn fake_analyze(sgf_text: String) -> Result<Vec<AnalysisFrameDto>, String> {
    let document = sgf::parse_sgf(&sgf_text).map_err(|err| err.to_string())?;
    let job_id = Uuid::new_v4();
    let mut frames = Vec::new();
    for turn in 0..=document.moves.len() as u32 {
        let drift = ((turn as f32 * 0.73).sin()) * 0.13;
        let winrate = (0.52 + drift).clamp(0.05, 0.95);
        let score = (turn as f32 * 0.31).cos() * 6.0;
        frames.push(AnalysisFrameDto {
            job_id,
            game_id: None,
            node_id: None,
            turn,
            visits: 256,
            winrate_black: winrate,
            score_mean_black: score,
            score_stdev: Some(4.2),
            candidates: demo_candidates(turn, document.board_size),
            ownership: None,
            policy: None,
        });
    }
    Ok(frames)
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
    let mut checks = check_assets(&profile);
    if matches!(profile.backend, EngineBackend::KataGoAnalysis) {
        ensure_asset_check(&mut checks, &profile.model_path, "model");
        ensure_asset_check(&mut checks, &profile.config_path, "config");
    }
    checks
}

#[tauri::command]
fn load_app_preferences(app_handle: AppHandle) -> Result<AppPreferencesLoadResultDto, String> {
    let path = app_preferences_path(&app_handle)?;
    load_app_preferences_from_path(&path)
}

#[tauri::command]
fn save_app_preferences(
    app_handle: AppHandle,
    preferences: AppPreferencesDto,
) -> Result<AppPreferencesDto, String> {
    let path = app_preferences_path(&app_handle)?;
    save_to_path(&path, preferences)
}

#[tauri::command]
fn load_engine_profile_settings(app_handle: AppHandle) -> Result<Option<EngineProfileSettingsDto>, String> {
    let settings = load_engine_profiles_settings(app_handle)?;
    let selected = selected_engine_profile_record(&settings)
        .or_else(|| settings.profiles.first())
        .cloned();
    Ok(selected.map(|record| EngineProfileSettingsDto {
        profile: record.profile,
        max_visits: record.max_visits,
    }))
}

#[tauri::command]
fn save_engine_profile_settings(
    app_handle: AppHandle,
    manager: State<'_, ForegroundEngineManager>,
    settings: EngineProfileSettingsDto,
) -> Result<EngineProfileSettingsDto, String> {
    validate_engine_profile_settings(&settings)?;
    let current = load_engine_profiles_from_disk(&app_handle).ok();
    let collection = EngineProfilesSettingsDto {
        selected_profile_id: DEFAULT_ENGINE_PROFILE_ID.to_string(),
        autoload_profile_id: current.and_then(|settings| settings.autoload_profile_id),
        profiles: vec![EngineProfileRecordDto {
            id: DEFAULT_ENGINE_PROFILE_ID.to_string(),
            profile: settings.profile.clone(),
            max_visits: settings.max_visits,
        }],
    };
    let saved = save_engine_profiles_settings(app_handle, manager, collection)?;
    let selected = selected_engine_profile_record(&saved)
        .ok_or_else(|| "saved engine profile collection did not include the selected profile".to_string())?;
    Ok(EngineProfileSettingsDto {
        profile: selected.profile.clone(),
        max_visits: selected.max_visits,
    })
}

#[tauri::command]
fn load_engine_profiles_settings(app_handle: AppHandle) -> Result<EngineProfilesSettingsDto, String> {
    load_engine_profiles_from_disk(&app_handle)
}

fn load_engine_profiles_from_disk(app_handle: &AppHandle) -> Result<EngineProfilesSettingsDto, String> {
    let path = engine_profile_path(app_handle)?;
    match fs::read_to_string(&path) {
        Ok(contents) => parse_engine_profiles_settings(&contents, &path),
        Err(err) if err.kind() == ErrorKind::NotFound => load_legacy_engine_profile_settings(app_handle),
        Err(err) => Err(format!("failed to read {}: {err}", path.display())),
    }
}

#[tauri::command]
fn save_engine_profiles_settings(
    app_handle: AppHandle,
    manager: State<'_, ForegroundEngineManager>,
    settings: EngineProfilesSettingsDto,
) -> Result<EngineProfilesSettingsDto, String> {
    let current = load_engine_profiles_from_disk(&app_handle)?;
    let settings = normalize_engine_profiles(settings)?;
    for record in &current.profiles {
        if !settings.profiles.iter().any(|next| next.id == record.id) {
            manager
                .assert_profile_deletable(&record.id)
                .map_err(map_engine_failure)?;
        }
    }
    let path = engine_profile_path(&app_handle)?;
    persist_engine_profiles(&path, settings)
}

#[tauri::command]
fn compute_game_cache_key(
    sgf_text: String,
    file_path: Option<String>,
) -> Result<ComputeGameCacheKeyDto, String> {
    let _ = file_path;
    let document = sgf::parse_sgf(&sgf_text).map_err(|err| err.to_string())?;
    let sgf_hash = stable_hash_hex(&sgf_text);
    let canonical = serde_json::json!({
        "document": document,
        "raw_sgf_hash": sgf_hash,
    });
    let canonical_text = serde_json::to_string(&canonical)
        .map_err(|err| format!("failed to serialize canonical SGF cache key: {err}"))?;
    let game_key = format!("sgf:{}", stable_hash_hex(&canonical_text));
    Ok(ComputeGameCacheKeyDto { game_key, sgf_hash })
}

#[tauri::command]
fn get_analysis_cache(
    app_handle: AppHandle,
    game_key: String,
    profile_id: Option<String>,
    engine_kind: Option<String>,
) -> Result<GetAnalysisCacheDto, String> {
    let path = analysis_cache_db_path(&app_handle)?;
    get_analysis_cache_at_path(&path, game_key, profile_id, engine_kind)
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
fn save_analysis_cache(
    app_handle: AppHandle,
    game_key: String,
    sgf_hash: String,
    profile_id: Option<String>,
    engine_kind: String,
    source: String,
    move_count: u32,
    analyzed_move_count: u32,
    payload: Value,
) -> Result<SaveAnalysisCacheDto, String> {
    let path = analysis_cache_db_path(&app_handle)?;
    save_analysis_cache_at_path(
        &path,
        game_key,
        sgf_hash,
        profile_id,
        engine_kind,
        source,
        move_count,
        analyzed_move_count,
        payload,
    )
}

#[tauri::command]
fn delete_analysis_cache(
    app_handle: AppHandle,
    game_key: String,
    profile_id: Option<String>,
    engine_kind: Option<String>,
) -> Result<DeleteAnalysisCacheDto, String> {
    let path = analysis_cache_db_path(&app_handle)?;
    delete_analysis_cache_at_path(&path, game_key, profile_id, engine_kind)
}

fn bind_selected_node_job(
    current_game: &CurrentGameState,
    run_id: String,
    generation: u64,
    node_path: NodePath,
    max_visits: u32,
) -> Result<SelectedNodeJobRequest, EngineFailureDto> {
    let (snapshot, board_size, komi, rules) = current_game
        .admit_selected_node(generation, &node_path)
        .map_err(|error| EngineFailureDto {
            operation: EngineOperationDto::Job,
            run_id: Some(run_id.clone()),
            switch_id: None,
            job_id: None,
            profile_id: None,
            kind: EngineFailureKind::InvalidState,
            message: error.message,
            diagnostic_summary: None,
        })?;
    let query = analysis_query_from_position(
        board_size,
        komi,
        &snapshot.position.stones,
        snapshot.position.to_play,
        AnalysisQueryOptions {
            id: "pending".to_string(),
            rules,
            turn: snapshot.position.move_number,
            max_visits: Some(max_visits),
            include_ownership: Some(true),
            include_policy: Some(true),
        },
    )
    .map_err(|error| EngineFailureDto {
        operation: EngineOperationDto::Job,
        run_id: Some(run_id.clone()),
        switch_id: None,
        job_id: None,
        profile_id: None,
        kind: EngineFailureKind::Protocol,
        message: error.to_string(),
        diagnostic_summary: None,
    })?;
    Ok(SelectedNodeJobRequest {
        run_id,
        generation,
        node_path,
        query,
        board_size,
    })
}

#[tauri::command]
fn foreground_engine_start_selected_node(
    manager: State<'_, ForegroundEngineManager>,
    current_game: State<'_, CurrentGameState>,
    run_id: String,
    generation: u64,
    node_path: NodePath,
    max_visits: u32,
) -> Result<AnalysisJobStartedDto, EngineFailureDto> {
    manager.start_selected_node_job(bind_selected_node_job(
        &current_game,
        run_id,
        generation,
        node_path,
        max_visits,
    )?)
}

#[tauri::command]
fn foreground_engine_cancel_job(
    manager: State<'_, ForegroundEngineManager>,
    run_id: String,
    job_id: String,
) -> Result<(), EngineFailureDto> {
    manager.cancel_job(&run_id, &job_id)
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
) -> Result<Vec<WholeGameWorkItem>, EngineFailureDto> {
    admitted
        .nodes
        .iter()
        .map(|snapshot| {
            let query = analysis_query_from_position(
                admitted.board_size,
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
            .map_err(|error| job_failure(run_id, EngineFailureKind::Protocol, error.to_string()))?;
            Ok(WholeGameWorkItem {
                node_path: snapshot.path.clone(),
                query,
                board_size: admitted.board_size,
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
) -> Result<AnalysisJobStartedDto, EngineFailureDto> {
    let admitted = current_game
        .admit_whole_game(generation)
        .map_err(|error| job_failure(&run_id, EngineFailureKind::InvalidState, error.message))?;
    let work_items = whole_game_work_items(&admitted, max_visits, &run_id)?;
    manager.start_whole_game_analysis(WholeGameJobRequest {
        run_id,
        generation: admitted.generation,
        work_items,
    })
}

#[tauri::command]
fn katago_cancel_analysis(
    manager: State<'_, ForegroundEngineManager>,
    run_id: String,
    job_id: String,
) -> Result<(), EngineFailureDto> {
    manager.cancel_job(&run_id, &job_id)
}

fn ensure_asset_check(checks: &mut Vec<AssetCheck>, path: &Option<String>, label: &str) {
    if checks.iter().any(|check| check.label == label) {
        return;
    }
    checks.push(AssetCheck {
        path: path.clone().unwrap_or_default(),
        exists: false,
        required: true,
        label: label.to_string(),
    });
}

fn selected_engine_profile_record(settings: &EngineProfilesSettingsDto) -> Option<&EngineProfileRecordDto> {
    settings
        .profiles
        .iter()
        .find(|profile| profile.id == settings.selected_profile_id)
}

fn parse_engine_profiles_settings(contents: &str, path: &Path) -> Result<EngineProfilesSettingsDto, String> {
    parse_engine_profiles(contents).map_err(|err| format!("failed to parse {}: {err}", path.display()))
}

fn load_legacy_engine_profile_settings(app_handle: &AppHandle) -> Result<EngineProfilesSettingsDto, String> {
    let legacy_path = legacy_engine_profile_path()?;
    match fs::read_to_string(&legacy_path) {
        Ok(contents) => {
            let settings = parse_engine_profiles_settings(&contents, &legacy_path)?;
            let path = engine_profile_path(app_handle)?;
            persist_engine_profiles(&path, settings.clone()).map_err(|err| {
                format!(
                    "failed to migrate engine profiles from {} to {}: {err}",
                    legacy_path.display(),
                    path.display()
                )
            })?;
            Ok(settings)
        }
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(default_engine_profiles_settings()),
        Err(err) => Err(format!("failed to read {}: {err}", legacy_path.display())),
    }
}

fn validate_engine_profile_settings(settings: &EngineProfileSettingsDto) -> Result<(), String> {
    if settings.max_visits == 0 {
        return Err("max_visits must be greater than 0".to_string());
    }
    if settings.profile.name.trim().is_empty() {
        return Err("engine profile name is required".to_string());
    }
    Ok(())
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

fn analysis_cache_db_path(app_handle: &AppHandle) -> Result<PathBuf, String> {
    let dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|err| format!("failed to resolve app data directory for analysis cache: {err}"))?;
    fs::create_dir_all(&dir).map_err(|err| {
        format!(
            "failed to create analysis cache directory {}: {err}",
            dir.display()
        )
    })?;
    Ok(dir.join(ANALYSIS_CACHE_DB_FILE))
}

fn open_analysis_cache_connection(path: &Path) -> Result<Connection, String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| {
            format!(
                "failed to create analysis cache directory {}: {err}",
                parent.display()
            )
        })?;
    }
    let mut conn = Connection::open(path)
        .map_err(|err| format!("failed to open analysis cache database {}: {err}", path.display()))?;
    storage::apply_migrations(&mut conn).map_err(|err| {
        format!(
            "failed to migrate analysis cache database {}: {err}",
            path.display()
        )
    })?;
    Ok(conn)
}

#[allow(clippy::too_many_arguments)]
fn save_analysis_cache_at_path(
    path: &Path,
    game_key: String,
    sgf_hash: String,
    profile_id: Option<String>,
    engine_kind: String,
    source: String,
    move_count: u32,
    analyzed_move_count: u32,
    payload: Value,
) -> Result<SaveAnalysisCacheDto, String> {
    let conn = open_analysis_cache_connection(path)?;
    let input = AnalysisCacheSaveInput {
        game_key,
        sgf_hash,
        profile_id,
        engine_kind,
        source,
        move_count,
        analyzed_move_count,
        payload,
    };
    save_analysis_cache_in_connection(&conn, input)
}

fn get_analysis_cache_at_path(
    path: &Path,
    game_key: String,
    profile_id: Option<String>,
    engine_kind: Option<String>,
) -> Result<GetAnalysisCacheDto, String> {
    let conn = open_analysis_cache_connection(path)?;
    let record =
        latest_analysis_cache_record(&conn, &game_key, profile_id.as_deref(), engine_kind.as_deref())?;
    Ok(GetAnalysisCacheDto {
        status: if record.is_some() { "hit" } else { "miss" }.to_string(),
        record,
        error: None,
    })
}

fn delete_analysis_cache_at_path(
    path: &Path,
    game_key: String,
    profile_id: Option<String>,
    engine_kind: Option<String>,
) -> Result<DeleteAnalysisCacheDto, String> {
    let conn = open_analysis_cache_connection(path)?;
    let deleted = if profile_id.is_none() && engine_kind.is_none() {
        storage::delete_analysis_for_game(&conn, &game_key, None)
            .map_err(|err| format!("failed to delete analysis cache for {game_key}: {err}"))?
    } else {
        matching_cache_scope_ids(&conn, &game_key, profile_id.as_deref(), engine_kind.as_deref())?
            .into_iter()
            .map(|scope_id| {
                storage::delete_analysis_for_game(&conn, &game_key, Some(&scope_id))
                    .map_err(|err| format!("failed to delete analysis cache for {game_key}: {err}"))
            })
            .try_fold(0usize, |total, deleted| deleted.map(|deleted| total + deleted))?
    };
    Ok(DeleteAnalysisCacheDto { deleted })
}

struct AnalysisCacheSaveInput {
    game_key: String,
    sgf_hash: String,
    profile_id: Option<String>,
    engine_kind: String,
    source: String,
    move_count: u32,
    analyzed_move_count: u32,
    payload: Value,
}

#[derive(Debug, Clone)]
struct StoredAnalysisJob {
    id: String,
    engine_profile_id: Option<String>,
    model_hash: Option<String>,
    created_at: Option<String>,
    finished_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AnalysisPositionRawJson {
    record: Option<AnalysisCacheRecordDto>,
    frame: Option<AnalysisFrameDto>,
}

fn save_analysis_cache_in_connection(
    conn: &Connection,
    input: AnalysisCacheSaveInput,
) -> Result<SaveAnalysisCacheDto, String> {
    let frames = parse_cached_frames(&input.payload)?;
    let id = cache_record_id(
        &input.game_key,
        input.profile_id.as_deref(),
        Some(input.engine_kind.as_str()),
    );
    let now = cache_timestamp();
    let created_at = existing_analysis_job_created_at(conn, &id)?.unwrap_or_else(|| now.clone());
    let scope_id = cache_scope_id(input.profile_id.as_deref(), Some(input.engine_kind.as_str()));
    let record = AnalysisCacheRecordDto {
        id: id.clone(),
        game_key: input.game_key.clone(),
        sgf_hash: input.sgf_hash.clone(),
        profile_id: input.profile_id.clone(),
        engine_kind: Some(input.engine_kind.clone()),
        source: input.source.clone(),
        move_count: input.move_count,
        analyzed_move_count: input.analyzed_move_count,
        payload: input.payload,
        created_at: Some(created_at.clone()),
        updated_at: now.clone(),
    };
    let game = storage::GameMetadata {
        id: input.game_key.clone(),
        source: input.source,
        source_id: Some(input.sgf_hash.clone()),
        board_size: i64::from(infer_board_size(&frames)),
        komi: 7.5,
        black_name: None,
        white_name: None,
        result: None,
        sgf_hash: Some(input.sgf_hash),
    };
    let job = storage::AnalysisJob {
        id: id.clone(),
        game_id: Some(input.game_key.clone()),
        engine_profile_id: scope_id,
        model_hash: Some(input.engine_kind),
        visits: frames
            .iter()
            .map(|frame| i64::from(frame.visits))
            .max()
            .unwrap_or(0),
        status: "finished".to_string(),
        created_at: Some(created_at),
        finished_at: Some(now.clone()),
    };

    conn.execute_batch("SAVEPOINT save_analysis_cache")
        .map_err(|err| format!("failed to start analysis cache save: {err}"))?;
    let result = (|| -> Result<(), String> {
        storage::upsert_game_metadata(conn, &game)
            .map_err(|err| format!("failed to upsert cached game metadata: {err}"))?;
        if analysis_job_exists(conn, &id)? {
            storage::update_analysis_job(conn, &job)
                .map_err(|err| format!("failed to update analysis cache job: {err}"))?;
        } else {
            storage::create_analysis_job(conn, &job)
                .map_err(|err| format!("failed to create analysis cache job: {err}"))?;
        }
        conn.execute("DELETE FROM analysis_positions WHERE job_id = ?1", [&id])
            .map_err(|err| format!("failed to replace cached analysis positions: {err}"))?;
        save_cached_positions(conn, &record, &frames)?;
        Ok(())
    })();

    match result {
        Ok(()) => {
            if let Err(err) = conn.execute_batch("RELEASE SAVEPOINT save_analysis_cache") {
                let _ = conn.execute_batch(
                    "ROLLBACK TO SAVEPOINT save_analysis_cache;
                    RELEASE SAVEPOINT save_analysis_cache;",
                );
                return Err(format!("failed to commit analysis cache save: {err}"));
            }
        }
        Err(err) => {
            let _ = conn.execute_batch(
                "ROLLBACK TO SAVEPOINT save_analysis_cache;
                RELEASE SAVEPOINT save_analysis_cache;",
            );
            return Err(err);
        }
    }

    Ok(SaveAnalysisCacheDto {
        id,
        game_key: input.game_key,
        updated_at: now,
    })
}

fn parse_cached_frames(payload: &Value) -> Result<Vec<AnalysisFrameDto>, String> {
    let frames = payload
        .get("frames")
        .ok_or_else(|| "analysis cache payload must include frames".to_string())?;
    if !payload.get("problems").is_some_and(Value::is_array) {
        return Err("analysis cache payload must include problems".to_string());
    }
    serde_json::from_value(frames.clone())
        .map_err(|err| format!("failed to parse analysis cache payload frames: {err}"))
}

fn save_cached_positions(
    conn: &Connection,
    record: &AnalysisCacheRecordDto,
    frames: &[AnalysisFrameDto],
) -> Result<(), String> {
    let mut saved_turn_zero_record = false;
    for frame in frames {
        let raw = AnalysisPositionRawJson {
            record: if frame.turn == 0 {
                saved_turn_zero_record = true;
                Some(record.clone())
            } else {
                None
            },
            frame: Some(frame.clone()),
        };
        let position = analysis_position_from_frame(&record.id, frame, raw)?;
        storage::upsert_analysis_position(conn, &position)
            .map_err(|err| format!("failed to save cached analysis position {}: {err}", frame.turn))?;
    }

    if !saved_turn_zero_record {
        let raw = AnalysisPositionRawJson {
            record: Some(record.clone()),
            frame: None,
        };
        let position = storage::AnalysisPosition {
            id: format!("{}:turn:0", record.id),
            job_id: record.id.clone(),
            node_id: None,
            turn: 0,
            visits: 0,
            winrate_black: 0.0,
            score_mean_black: 0.0,
            score_stdev: None,
            policy_json: None,
            ownership_json: None,
            candidates_json: "[]".to_string(),
            raw_json: Some(serialize_raw_position(&raw)?),
        };
        storage::upsert_analysis_position(conn, &position)
            .map_err(|err| format!("failed to save cached analysis payload: {err}"))?;
    }
    Ok(())
}

fn analysis_position_from_frame(
    job_id: &str,
    frame: &AnalysisFrameDto,
    raw: AnalysisPositionRawJson,
) -> Result<storage::AnalysisPosition, String> {
    Ok(storage::AnalysisPosition {
        id: format!("{job_id}:turn:{}", frame.turn),
        job_id: job_id.to_string(),
        node_id: frame.node_id.map(|id| id.to_string()),
        turn: i64::from(frame.turn),
        visits: i64::from(frame.visits),
        winrate_black: f64::from(frame.winrate_black),
        score_mean_black: f64::from(frame.score_mean_black),
        score_stdev: frame.score_stdev.map(f64::from),
        policy_json: optional_json_string(&frame.policy)?,
        ownership_json: optional_json_string(&frame.ownership)?,
        candidates_json: serde_json::to_string(&frame.candidates)
            .map_err(|err| format!("failed to serialize cached candidate moves: {err}"))?,
        raw_json: Some(serialize_raw_position(&raw)?),
    })
}

fn optional_json_string<T: Serialize>(value: &Option<T>) -> Result<Option<String>, String> {
    value
        .as_ref()
        .map(serde_json::to_string)
        .transpose()
        .map_err(|err| format!("failed to serialize analysis cache JSON field: {err}"))
}

fn serialize_raw_position(raw: &AnalysisPositionRawJson) -> Result<String, String> {
    serde_json::to_string(raw)
        .map_err(|err| format!("failed to serialize cached raw analysis payload: {err}"))
}

fn latest_analysis_cache_record(
    conn: &Connection,
    game_key: &str,
    profile_id: Option<&str>,
    engine_kind: Option<&str>,
) -> Result<Option<AnalysisCacheRecordDto>, String> {
    for job in load_finished_analysis_jobs(conn, game_key)? {
        let record = analysis_cache_record_from_job(conn, game_key, &job)?;
        if cache_record_matches(&record, game_key, profile_id, engine_kind) {
            return Ok(Some(record));
        }
    }
    Ok(None)
}

fn matching_cache_scope_ids(
    conn: &Connection,
    game_key: &str,
    profile_id: Option<&str>,
    engine_kind: Option<&str>,
) -> Result<Vec<String>, String> {
    let mut scopes = HashSet::new();
    for job in load_finished_analysis_jobs(conn, game_key)? {
        let Some(scope_id) = job.engine_profile_id.clone() else {
            continue;
        };
        let record = analysis_cache_record_from_job(conn, game_key, &job)?;
        if cache_record_matches(&record, game_key, profile_id, engine_kind) {
            scopes.insert(scope_id);
        }
    }
    Ok(scopes.into_iter().collect())
}

fn load_finished_analysis_jobs(conn: &Connection, game_key: &str) -> Result<Vec<StoredAnalysisJob>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, engine_profile_id, model_hash, created_at, finished_at
            FROM analysis_jobs
            WHERE game_id = ?1 AND status = 'finished'
            ORDER BY COALESCE(finished_at, created_at) DESC, created_at DESC, id DESC",
        )
        .map_err(|err| format!("failed to prepare analysis cache lookup: {err}"))?;
    let rows = stmt
        .query_map([game_key], |row| {
            Ok(StoredAnalysisJob {
                id: row.get(0)?,
                engine_profile_id: row.get(1)?,
                model_hash: row.get(2)?,
                created_at: row.get(3)?,
                finished_at: row.get(4)?,
            })
        })
        .map_err(|err| format!("failed to query analysis cache jobs: {err}"))?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|err| format!("failed to read analysis cache jobs: {err}"))
}

fn analysis_cache_record_from_job(
    conn: &Connection,
    game_key: &str,
    job: &StoredAnalysisJob,
) -> Result<AnalysisCacheRecordDto, String> {
    let positions = storage::load_analysis_positions(conn, &job.id)
        .map_err(|err| format!("failed to load cached analysis positions: {err}"))?;
    if let Some(record) = positions.iter().find_map(raw_record_from_position) {
        return Ok(record);
    }

    let sgf_hash = conn
        .query_row("SELECT sgf_hash FROM games WHERE id = ?1", [game_key], |row| {
            row.get::<_, Option<String>>(0)
        })
        .optional()
        .map_err(|err| format!("failed to load cached game metadata: {err}"))?
        .flatten()
        .unwrap_or_default();
    let frames = positions
        .iter()
        .filter_map(raw_frame_from_position)
        .collect::<Vec<_>>();
    let payload = serde_json::json!({
        "frames": frames,
        "problems": [],
    });
    Ok(AnalysisCacheRecordDto {
        id: job.id.clone(),
        game_key: game_key.to_string(),
        sgf_hash,
        profile_id: None,
        engine_kind: job.model_hash.clone(),
        source: job.model_hash.clone().unwrap_or_else(|| "katago".to_string()),
        move_count: positions
            .iter()
            .map(|position| position.turn as u32)
            .max()
            .unwrap_or(0),
        analyzed_move_count: positions.len() as u32,
        payload,
        created_at: job.created_at.clone(),
        updated_at: job
            .finished_at
            .clone()
            .or_else(|| job.created_at.clone())
            .unwrap_or_else(cache_timestamp),
    })
}

fn raw_record_from_position(position: &storage::AnalysisPosition) -> Option<AnalysisCacheRecordDto> {
    position
        .raw_json
        .as_deref()
        .and_then(|raw| serde_json::from_str::<AnalysisPositionRawJson>(raw).ok())
        .and_then(|raw| raw.record)
}

fn raw_frame_from_position(position: &storage::AnalysisPosition) -> Option<AnalysisFrameDto> {
    if let Some(frame) = position
        .raw_json
        .as_deref()
        .and_then(|raw| serde_json::from_str::<AnalysisPositionRawJson>(raw).ok())
        .and_then(|raw| raw.frame)
    {
        return Some(frame);
    }
    let candidates = serde_json::from_str::<Vec<CandidateMoveDto>>(&position.candidates_json).ok()?;
    let ownership = position
        .ownership_json
        .as_deref()
        .map(serde_json::from_str::<Vec<f32>>)
        .transpose()
        .ok()?;
    let policy = position
        .policy_json
        .as_deref()
        .map(serde_json::from_str::<Vec<f32>>)
        .transpose()
        .ok()?;
    Some(AnalysisFrameDto {
        job_id: Uuid::nil(),
        game_id: None,
        node_id: position
            .node_id
            .as_deref()
            .and_then(|id| Uuid::parse_str(id).ok()),
        turn: position.turn as u32,
        visits: position.visits as u32,
        winrate_black: position.winrate_black as f32,
        score_mean_black: position.score_mean_black as f32,
        score_stdev: position.score_stdev.map(|value| value as f32),
        candidates,
        ownership,
        policy,
    })
}

fn existing_analysis_job_created_at(conn: &Connection, job_id: &str) -> Result<Option<String>, String> {
    conn.query_row(
        "SELECT created_at FROM analysis_jobs WHERE id = ?1",
        [job_id],
        |row| row.get(0),
    )
    .optional()
    .map_err(|err| format!("failed to read existing analysis cache job: {err}"))
}

fn analysis_job_exists(conn: &Connection, job_id: &str) -> Result<bool, String> {
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(1) FROM analysis_jobs WHERE id = ?1",
            [job_id],
            |row| row.get(0),
        )
        .map_err(|err| format!("failed to inspect analysis cache job: {err}"))?;
    Ok(count > 0)
}

fn cache_scope_id(profile_id: Option<&str>, engine_kind: Option<&str>) -> Option<String> {
    if profile_id.is_none() && engine_kind.is_none() {
        return None;
    }
    let scope = serde_json::json!({
        "profile_id": profile_id,
        "engine_kind": engine_kind,
    });
    Some(format!(
        "cache-scope:{}",
        stable_hash_hex(&scope.to_string())
            .chars()
            .take(24)
            .collect::<String>()
    ))
}

fn infer_board_size(frames: &[AnalysisFrameDto]) -> u8 {
    frames
        .iter()
        .flat_map(|frame| [frame.ownership.as_ref(), frame.policy.as_ref()])
        .flatten()
        .find_map(|values| perfect_square_board_size(values.len()))
        .unwrap_or(19)
}

fn perfect_square_board_size(value_count: usize) -> Option<u8> {
    let size = (value_count as f64).sqrt() as usize;
    if (2..=25).contains(&size) && size * size == value_count {
        Some(size as u8)
    } else {
        None
    }
}

fn cache_record_matches(
    record: &AnalysisCacheRecordDto,
    game_key: &str,
    profile_id: Option<&str>,
    engine_kind: Option<&str>,
) -> bool {
    if record.game_key != game_key {
        return false;
    }
    if profile_id.is_some() && record.profile_id.as_deref() != profile_id {
        return false;
    }
    if engine_kind.is_some() && record.engine_kind.as_deref() != engine_kind {
        return false;
    }
    true
}

fn cache_record_id(game_key: &str, profile_id: Option<&str>, engine_kind: Option<&str>) -> String {
    format!(
        "cache:{}",
        stable_hash_hex(&format!(
            "{}\n{}\n{}",
            game_key,
            profile_id.unwrap_or(""),
            engine_kind.unwrap_or("")
        ))
        .chars()
        .take(24)
        .collect::<String>()
    )
}

fn cache_timestamp() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default();
    format_unix_seconds_utc(seconds)
}

fn stable_hash_hex(value: &str) -> String {
    const SEEDS: [u64; 4] = [
        0xcbf2_9ce4_8422_2325,
        0x8422_2325_cbf2_9ce4,
        0x9e37_79b9_7f4a_7c15,
        0x94d0_49bb_1331_11eb,
    ];
    SEEDS
        .iter()
        .map(|seed| format!("{:016x}", fnv1a64(value.as_bytes(), *seed)))
        .collect::<String>()
}

fn fnv1a64(bytes: &[u8], seed: u64) -> u64 {
    const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
    let mut hash = seed;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash ^= bytes.len() as u64;
    hash.wrapping_mul(FNV_PRIME)
}

fn format_unix_seconds_utc(seconds: u64) -> String {
    let days = (seconds / 86_400) as i64;
    let seconds_of_day = seconds % 86_400;
    let (year, month, day) = civil_from_days(days);
    let hour = seconds_of_day / 3_600;
    let minute = (seconds_of_day % 3_600) / 60;
    let second = seconds_of_day % 60;
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

fn civil_from_days(days_since_unix_epoch: i64) -> (i64, u32, u32) {
    let days = days_since_unix_epoch + 719_468;
    let era = if days >= 0 { days } else { days - 146_096 } / 146_097;
    let day_of_era = days - era * 146_097;
    let year_of_era = (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    let year = year + if month <= 2 { 1 } else { 0 };
    (year, month as u32, day as u32)
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
    result.summary.board_size = Some(document.board_size);
    result.summary.komi = Some(document.komi);
    result.summary.handicap = document.handicap;
    result.summary.black_name = document.black_name;
    result.summary.white_name = document.white_name;
    result.summary.result = document.result;
    result.summary.move_count = Some(document.moves.len());
    Ok(result)
}

fn validate_provider_fetch_request(
    request: &ProviderFetchRequest,
    expected_provider: ProviderKind,
    command_name: &str,
) -> Result<(), ProviderError> {
    if request.provider != expected_provider {
        return Err(ProviderError {
            kind: ProviderErrorKind::InvalidRequest,
            message: format!("{command_name} received provider {:?}", request.provider),
        });
    }
    if request.url.trim().is_empty() {
        return Err(ProviderError {
            kind: ProviderErrorKind::InvalidRequest,
            message: format!("{command_name} requires a non-empty url"),
        });
    }
    validate_timeout_ms(request.timeout_ms, command_name)
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
    let kind = match &error {
        readboard_sidecar::ReadboardSidecarError::MissingLaunchTarget => {
            ProviderErrorKind::RuntimeUnavailable
        }
        readboard_sidecar::ReadboardSidecarError::EmptyProtocolLine
        | readboard_sidecar::ReadboardSidecarError::MissingField(_)
        | readboard_sidecar::ReadboardSidecarError::InvalidField { .. }
        | readboard_sidecar::ReadboardSidecarError::DuplicateField { .. } => {
            ProviderErrorKind::InvalidPayload
        }
        readboard_sidecar::ReadboardSidecarError::Sync(_) => ProviderErrorKind::ParseFailed,
    };
    ProviderError {
        kind,
        message: error.to_string(),
    }
}

fn demo_candidates(turn: u32, board_size: u8) -> Vec<CandidateMoveDto> {
    let anchors = [(15usize, 3usize), (3, 15), (15, 15), (3, 3), (9, 9), (10, 15)];
    anchors
        .iter()
        .enumerate()
        .map(|(index, (x, y))| CandidateMoveDto {
            vertex: MoveVertex::Point(PointDto {
                x: ((*x + turn as usize + index) % board_size as usize) as u8,
                y: ((*y + index * 2) % board_size as usize) as u8,
            }),
            visits: 128u32.saturating_sub(index as u32 * 13),
            winrate_black: (0.58 - index as f32 * 0.025).clamp(0.0, 1.0),
            score_mean_black: 4.5 - index as f32,
            policy_prior: Some(0.18 - index as f32 * 0.015),
            pv: Vec::new(),
        })
        .collect()
}

#[tauri::command]
fn foreground_engine_snapshot(manager: State<'_, ForegroundEngineManager>) -> ForegroundEngineSnapshotDto {
    manager.snapshot()
}

#[tauri::command]
fn foreground_engine_start(
    manager: State<'_, ForegroundEngineManager>,
    profile_id: String,
) -> Result<(), EngineFailureDto> {
    manager.start(&profile_id)
}

#[tauri::command]
fn foreground_engine_stop(manager: State<'_, ForegroundEngineManager>) -> Result<(), EngineFailureDto> {
    manager.stop()
}

#[tauri::command]
fn foreground_engine_restart(manager: State<'_, ForegroundEngineManager>) -> Result<(), EngineFailureDto> {
    manager.restart()
}

#[tauri::command]
fn foreground_engine_switch(
    manager: State<'_, ForegroundEngineManager>,
    profile_id: String,
) -> Result<(), EngineFailureDto> {
    manager.switch_to(&profile_id)
}

pub fn run() {
    tauri::Builder::default()
        .manage(CurrentGameState::default())
        .setup(|app| {
            let _ = load_engine_profiles_from_disk(app.handle());
            let catalog = std::sync::Arc::new(DiskEngineCatalog {
                handle: app.handle().clone(),
            });
            let manager = ForegroundEngineManager::new(catalog, ForegroundEngineConfig::default());
            let events = manager.subscribe();
            let emit_handle = app.handle().clone();
            std::thread::spawn(move || {
                while let Ok(event) = events.recv() {
                    match event {
                        ForegroundEngineEventDto::Snapshot { snapshot } => {
                            let _ = emit_handle.emit("foreground-engine://snapshot", snapshot);
                        }
                        ForegroundEngineEventDto::Failure { failure } => {
                            let _ = emit_handle.emit("foreground-engine://failure", failure);
                        }
                        ForegroundEngineEventDto::Job { job } => {
                            if let Some(state) = emit_handle.try_state::<CurrentGameState>() {
                                let _ = state.attach_from_job_event(&job);
                            }
                            let _ = emit_handle.emit("foreground-engine://job", job);
                        }
                    }
                }
            });
            let _ = manager.apply_autoload();
            app.manage(manager);
            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            health,
            parse_sgf_summary,
            provider_parse_yike_url,
            provider_import_from_payload,
            provider_fetch_yike,
            provider_fetch_fox,
            readboard_sidecar_probe,
            readboard_sidecar_sync_snapshot,
            replay_sgf_positions,
            read_sgf_file,
            replace_current_game,
            serialize_current_game,
            save_current_game,
            save_current_game_as,
            project_current_game_mainline,
            select_current_game_node,
            play_current_game,
            set_current_game_personal_comment,
            remove_current_game_variation,
            fake_analyze,
            classify_problems,
            katago_launch_plan,
            engine_asset_checks,
            load_app_preferences,
            save_app_preferences,
            load_engine_profile_settings,
            save_engine_profile_settings,
            load_engine_profiles_settings,
            save_engine_profiles_settings,
            compute_game_cache_key,
            get_analysis_cache,
            save_analysis_cache,
            delete_analysis_cache,
            katago_start_analyze_game,
            katago_cancel_analysis,
            foreground_engine_snapshot,
            foreground_engine_start,
            foreground_engine_stop,
            foreground_engine_restart,
            foreground_engine_switch,
            foreground_engine_start_selected_node,
            foreground_engine_cancel_job
        ])
        .build(tauri::generate_context!())
        .expect("failed to build LizzieYzy Next")
        .run(|app, event| {
            if let tauri::RunEvent::Exit = event {
                if let Some(manager) = app.try_state::<ForegroundEngineManager>() {
                    let _ = manager.teardown();
                }
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn whole_game_work_items_use_current_game_owned_rules() {
        let state = CurrentGameState::default();
        let opened = state
            .replace("(;GM[1]FF[4]SZ[9]KM[6.5]RU[Japanese];B[dd])", None)
            .unwrap();
        let admitted = state.admit_whole_game(opened.generation).unwrap();
        let items = whole_game_work_items(&admitted, 32, "run-rules").unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].query.rules, "japanese");
        assert_eq!(items[1].query.rules, "japanese");
    }

    #[test]
    fn whole_game_admission_rejects_stale_generation_before_worklist() {
        let state = CurrentGameState::default();
        let opened = state.replace("(;GM[1]FF[4]SZ[9])", None).unwrap();
        let stale = state.admit_whole_game(opened.generation + 1).unwrap_err();
        assert_eq!(stale.message, "current game generation does not match");
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
    fn obsolete_profile_to_process_commands_are_not_registered() {
        let commands =
            registered_tauri_commands(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/lib.rs")));
        assert!(
            commands.iter().all(|command| *command != "katago_analyze_once"),
            "legacy one-shot command must not remain registered: {commands:?}"
        );
        assert!(
            commands.iter().all(|command| *command != "katago_analyze_game"),
            "legacy batch command must not remain registered: {commands:?}"
        );
        assert!(
            commands.contains(&"katago_start_analyze_game"),
            "whole-game analysis must stay on the manager-owned run command"
        );
        assert!(
            commands.contains(&"foreground_engine_start_selected_node"),
            "selected-node analysis must stay on the manager-owned run command"
        );
        assert!(
            commands.contains(&"fake_analyze"),
            "browser/native fake analysis remains available and non-authoritative"
        );
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
    fn whole_game_command_admits_current_game_instead_of_caller_sgf_batch() {
        let source = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/lib.rs"));
        let command = source
            .split("fn katago_start_analyze_game(")
            .nth(1)
            .and_then(|rest| rest.split("fn katago_cancel_analysis(").next())
            .expect("katago_start_analyze_game");
        assert!(command.contains("admit_whole_game"));
        assert!(command.contains("generation: u64"));
        assert!(!command.contains("sgf_text"));
        assert!(!command.contains("analysis_batch_query_from_game"));
        assert!(!command.contains("prepare_katago_batch_analysis"));
    }

    #[test]
    fn selected_node_request_captures_exact_position_turn_rules_and_komi_before_protocol() {
        let state = CurrentGameState::default();
        let opened = state
            .replace("(;GM[1]FF[4]SZ[5]KM[6.5]RU[Japanese];B[cc];W[ee])", None)
            .unwrap();
        let path = NodePath { indices: vec![0, 0] };
        let selected = state.select_path(path.clone()).unwrap();
        assert_eq!(selected.snapshot.position.move_number, 2);
        assert_eq!(selected.snapshot.position.to_play, app_model::PlayerColor::Black);

        let request = bind_selected_node_job(
            &state,
            "run-ready".to_string(),
            opened.generation,
            path.clone(),
            64,
        )
        .unwrap();

        assert_eq!(request.run_id, "run-ready");
        assert_eq!(request.generation, opened.generation);
        assert_eq!(request.node_path, path);
        assert_eq!(request.board_size, 5);
        assert_eq!(request.query.komi, 6.5);
        assert_eq!(request.query.rules, "japanese");
        assert_eq!(request.query.include_ownership, Some(true));
        assert_eq!(request.query.include_policy, Some(true));
        assert_eq!(request.query.max_visits, Some(64));
        assert_eq!(
            request.query.initial_stones,
            vec![
                ("B".to_string(), "C3".to_string()),
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
            8,
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
            8,
        ) {
            Err(error) => error,
            Ok(_) => panic!("stale generation must fail before protocol"),
        };
        assert_eq!(error.kind, EngineFailureKind::InvalidState);
        assert_eq!(error.message, "current game generation does not match");
        assert!(error.job_id.is_none());
    }

    #[test]
    fn provider_fetch_yike_adds_missing_signature_headers_without_network() {
        let mut request = provider_fetch_request(ProviderKind::Yike);
        request
            .headers
            .insert("AppKey".to_string(), "caller-app-key".to_string());

        let prepared = prepare_yike_fetch_request(request);

        assert_eq!(
            prepared.headers.get("AppKey").map(String::as_str),
            Some("caller-app-key")
        );
        assert!(prepared.headers.contains_key("CurTime"));
        assert!(prepared.headers.contains_key("CheckSum"));
        assert!(prepared.headers.contains_key("Nonce"));
        assert!(prepared.headers.contains_key("accesstoken"));
    }

    #[test]
    fn provider_fetch_fox_http_adds_default_user_agent_without_network() {
        let request = provider_fetch_request(ProviderKind::Fox);

        let prepared = prepare_fox_http_fetch_request(request);

        assert_eq!(
            prepared.headers.get("User-Agent").map(String::as_str),
            Some(provider_fox::FOX_MOBILE_USER_AGENT)
        );
    }

    #[test]
    fn provider_fetch_yike_validates_detail_payload_and_preserves_signature_headers_without_network() {
        let mut request = provider_fetch_request(ProviderKind::Yike);
        request.url = "https://api-new.yikeweiqi.com/v1/golives/186031".to_string();
        request
            .headers
            .insert("AppKey".to_string(), "caller-app-key".to_string());
        let transport = provider_core::RecordingProviderTransport::with_result(Ok(provider_fetch_result(
            ProviderKind::Yike,
            "https://api-new.yikeweiqi.com/v1/golives/186031",
            200,
            r#"{"status":0,"result":{"sgf":"(;GM[1]SZ[19];B[aa])","status":2}}"#,
        )));

        let result = fetch_yike_with_transport(request, &transport).unwrap();

        assert_eq!(result.status_code, 200);
        let requests = transport.requests().unwrap();
        assert_eq!(requests.len(), 1);
        assert_eq!(
            requests[0].headers.get("AppKey").map(String::as_str),
            Some("caller-app-key")
        );
        assert!(requests[0].headers.contains_key("CurTime"));
        assert!(requests[0].headers.contains_key("CheckSum"));
        assert!(requests[0].headers.contains_key("Nonce"));
        assert!(requests[0].headers.contains_key("accesstoken"));
    }

    #[test]
    fn provider_fetch_yike_maps_http_and_bad_json_without_network() {
        let mut request = provider_fetch_request(ProviderKind::Yike);
        request.url = "https://api-new.yikeweiqi.com/v1/golives/186031".to_string();
        let transport = provider_core::StaticProviderTransport::ok(provider_fetch_result(
            ProviderKind::Yike,
            "https://api-new.yikeweiqi.com/v1/golives/186031",
            503,
            "service unavailable",
        ));

        let error = fetch_yike_with_transport(request.clone(), &transport).unwrap_err();
        assert_eq!(error.kind, ProviderErrorKind::TransportFailed);
        assert!(error.message.contains("HTTP 503"));

        let transport = provider_core::StaticProviderTransport::ok(provider_fetch_result(
            ProviderKind::Yike,
            "https://api-new.yikeweiqi.com/v1/golives/186031",
            200,
            "{",
        ));

        let error = fetch_yike_with_transport(request, &transport).unwrap_err();
        assert_eq!(error.kind, ProviderErrorKind::InvalidPayload);
    }

    #[test]
    fn provider_fetch_yike_validates_list_payload_without_network() {
        let mut request = provider_fetch_request(ProviderKind::Yike);
        request.url = "https://api.yikeweiqi.com/v2/golive/list?p=1&since=0&official=&version=2".to_string();
        let transport = provider_core::StaticProviderTransport::ok(provider_fetch_result(
            ProviderKind::Yike,
            &request.url,
            200,
            r#"{"Status":1200,"Result":{"since":12,"list":[]}}"#,
        ));

        let result = fetch_yike_with_transport(request, &transport).unwrap();

        assert_eq!(result.status_code, 200);
    }

    #[test]
    fn provider_fetch_fox_http_checks_status_and_warns_without_network() {
        let mut request = provider_fetch_request(ProviderKind::Fox);
        request.url = "https://example.test/fox".to_string();
        let transport = provider_core::StaticProviderTransport::ok(provider_fetch_result(
            ProviderKind::Fox,
            "https://example.test/fox",
            500,
            "server error",
        ));

        let error = fetch_fox_with_transport(request.clone(), &transport).unwrap_err();
        assert_eq!(error.kind, ProviderErrorKind::TransportFailed);
        assert!(error.message.contains("HTTP 500"));

        let transport = provider_core::StaticProviderTransport::ok(provider_fetch_result(
            ProviderKind::Fox,
            "https://example.test/fox",
            200,
            "{}",
        ));
        let result = fetch_fox_with_transport(request, &transport).unwrap();

        assert!(result.warnings.iter().any(|warning| warning.contains("directly")));
    }

    #[test]
    fn provider_fetch_commands_validate_provider_before_runtime() {
        let error = provider_fetch_yike(provider_fetch_request(ProviderKind::Fox)).unwrap_err();

        assert_eq!(error.kind, ProviderErrorKind::InvalidRequest);
        assert!(error.message.contains("provider_fetch_yike"));
    }

    #[test]
    fn provider_fetch_fox_non_http_uses_command_parser_without_network() {
        let mut request = provider_fetch_request(ProviderKind::Fox);
        request.url = "not-a-fox-command".to_string();

        let error = provider_fetch_fox(request).unwrap_err();

        assert_eq!(error.kind, ProviderErrorKind::InvalidRequest);
        assert!(error.message.contains("Fox command"));
    }

    #[test]
    fn readboard_sidecar_probe_returns_structured_runtime_status() {
        let result = readboard_sidecar_probe(ReadboardSidecarProbeRequest {
            endpoint: Some("local-test-endpoint".to_string()),
            timeout_ms: Some(100),
        })
        .unwrap();

        assert!(!result.available);
        assert_eq!(result.endpoint.as_deref(), Some("local-test-endpoint"));
        assert!(result
            .warnings
            .iter()
            .any(|warning| warning.contains("UnsupportedEndpoint")));
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
        assert_eq!(position.board_size, 2);
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
    fn analysis_cache_saves_and_restores_payload_from_sqlite() {
        let path = std::env::temp_dir().join(format!("lizzieyzy-analysis-cache-{}.sqlite3", Uuid::new_v4()));
        let sgf_text = "(;GM[1]FF[4]SZ[19]KM[7.5]PB[Black]PW[White];B[dd];W[qq])";
        let cache_key = compute_game_cache_key(sgf_text.to_string(), None).unwrap();
        let frames = fake_analyze(sgf_text.to_string()).unwrap();
        let problems = classify_problems(frames.clone());
        let payload = serde_json::json!({
            "frames": frames,
            "problems": problems,
        });

        let saved = save_analysis_cache_at_path(
            &path,
            cache_key.game_key.clone(),
            cache_key.sgf_hash.clone(),
            None,
            "fake".to_string(),
            "fake".to_string(),
            2,
            payload["frames"].as_array().unwrap().len() as u32,
            payload.clone(),
        )
        .unwrap();
        let lookup =
            get_analysis_cache_at_path(&path, cache_key.game_key.clone(), None, Some("fake".to_string()))
                .unwrap();

        assert_eq!(lookup.status, "hit");
        let record = lookup.record.unwrap();
        assert_eq!(record.id, saved.id);
        assert_eq!(record.game_key, cache_key.game_key);
        assert_eq!(record.sgf_hash, cache_key.sgf_hash);
        assert_eq!(record.engine_kind.as_deref(), Some("fake"));
        assert_eq!(
            record.payload["frames"].as_array().unwrap().len(),
            payload["frames"].as_array().unwrap().len()
        );
        assert_eq!(
            record.payload["problems"].as_array().unwrap().len(),
            payload["problems"].as_array().unwrap().len()
        );
        assert_eq!(record.payload["frames"][0]["turn"], serde_json::json!(0));

        let conn = open_analysis_cache_connection(&path).unwrap();
        let game_count: i64 = conn
            .query_row("SELECT COUNT(1) FROM games", [], |row| row.get(0))
            .unwrap();
        let job_count: i64 = conn
            .query_row("SELECT COUNT(1) FROM analysis_jobs", [], |row| row.get(0))
            .unwrap();
        let position_count: i64 = conn
            .query_row("SELECT COUNT(1) FROM analysis_positions", [], |row| row.get(0))
            .unwrap();
        let full_payload_rows: i64 = conn
            .query_row(
                "SELECT COUNT(1) FROM analysis_positions WHERE turn = 0 AND raw_json LIKE '%\"record\"%'",
                [],
                |row| row.get(0),
            )
            .unwrap();

        let _ = fs::remove_file(&path);
        assert_eq!(game_count, 1);
        assert_eq!(job_count, 1);
        assert_eq!(position_count, payload["frames"].as_array().unwrap().len() as i64);
        assert_eq!(full_payload_rows, 1);
    }

    fn provider_fetch_request(provider: ProviderKind) -> ProviderFetchRequest {
        ProviderFetchRequest {
            provider,
            url: "https://example.test/provider".to_string(),
            method: app_model::ProviderFetchMethod::Get,
            headers: std::collections::BTreeMap::new(),
            body: None,
            source_url: None,
            source_id: None,
            timeout_ms: Some(100),
        }
    }

    fn provider_fetch_result(
        provider: ProviderKind,
        url: &str,
        status_code: u16,
        payload: &str,
    ) -> ProviderFetchResult {
        ProviderFetchResult {
            provider,
            url: url.to_string(),
            status_code,
            payload: payload.to_string(),
            headers: std::collections::BTreeMap::new(),
            content_type: Some("application/json".to_string()),
            metadata: ProviderGameMetadata {
                request_url: Some(url.to_string()),
                ..ProviderGameMetadata::default()
            },
            warnings: Vec::new(),
        }
    }
}
