use crate::continuous_analysis::PreferencesState;
use app_model::{
    FoxAccountDto, FoxKifuStateDto, FoxListResultDto, FoxLookupDto, FoxPreviewResultDto, NetworkSettingsDto,
    NetworkSnapshotDto, ProviderError, ProviderRequestIdentityDto,
};
use tauri::{AppHandle, State};

#[tauri::command]
pub fn network_snapshot(state: State<PreferencesState>) -> NetworkSnapshotDto {
    state.network().snapshot()
}

#[tauri::command]
pub fn save_network_settings(
    app: AppHandle,
    state: State<PreferencesState>,
    settings: NetworkSettingsDto,
) -> Result<NetworkSnapshotDto, String> {
    let snapshot = state.save_network(&super::app_preferences_path(&app)?, settings)?;
    crate::external_sync::wake(&app);
    Ok(snapshot)
}

#[tauri::command]
pub fn begin_provider_request(
    state: State<PreferencesState>,
    current: State<crate::current_game_state::CurrentGameState>,
    policy_revision: u64,
) -> Result<ProviderRequestIdentityDto, ProviderError> {
    state
        .network()
        .begin(policy_revision, current.document_identity())
}

#[tauri::command]
pub fn cancel_provider_request(state: State<PreferencesState>, identity: ProviderRequestIdentityDto) {
    state.network().cancel(&identity);
}

/// Runs one provider read under the exact request lease. Results are dropped when the lease, the
/// policy revision or the current document changed while the read was in flight.
async fn read_provider<T: Send + 'static>(
    state: &PreferencesState,
    current: &crate::current_game_state::CurrentGameState,
    identity: &ProviderRequestIdentityDto,
    read: impl FnOnce(&provider_core::network::NetworkOperation) -> Result<T, ProviderError> + Send + 'static,
) -> Result<(T, Vec<app_model::NetworkRouteDto>), ProviderError> {
    let operation = state.network().operation(identity)?;
    let response = tauri::async_runtime::spawn_blocking(move || {
        let result = read(&operation).map_err(|mut error| {
            // Parser diagnostics may quote an upstream body or signed URL. Never expose either.
            if matches!(
                error.kind,
                app_model::ProviderErrorKind::InvalidPayload
                    | app_model::ProviderErrorKind::ParseFailed
                    | app_model::ProviderErrorKind::InvalidUrl
                    | app_model::ProviderErrorKind::InvalidRequest
            ) {
                error.message =
                    "Provider request or response is invalid; check the provider input and retry preview."
                        .into();
            }
            error
        })?;
        operation.lease().check()?;
        Ok((result, operation.routes()))
    })
    .await
    .map_err(|_| provider_core::runtime_unavailable("Provider worker did not complete."))??;
    state.network().operation(identity)?.lease().check()?;
    if current.document_identity() != identity.document_identity {
        state.network().cancel(identity);
        return Err(provider_core::provider_error(
            app_model::ProviderErrorKind::Cancelled,
            "Current document changed; preview again.",
        ));
    }
    Ok(response)
}

#[tauri::command]
pub async fn provider_yike_list(
    state: State<'_, PreferencesState>,
    current: State<'_, crate::current_game_state::CurrentGameState>,
    identity: ProviderRequestIdentityDto,
    category: app_model::YikeCategoryDto,
    page: u32,
    since: u64,
) -> Result<app_model::YikeListResultDto, ProviderError> {
    let (result, routes) = read_provider(&state, &current, &identity, move |operation| {
        provider_yike::fetch_public_list(operation, category, page, since)
    })
    .await?;
    Ok(app_model::YikeListResultDto {
        identity,
        result,
        routes,
    })
}

#[tauri::command]
pub async fn provider_yike_preview(
    state: State<'_, PreferencesState>,
    current: State<'_, crate::current_game_state::CurrentGameState>,
    identity: ProviderRequestIdentityDto,
    locator: String,
) -> Result<app_model::YikePreviewResultDto, ProviderError> {
    let (result, routes) = read_provider(&state, &current, &identity, move |operation| {
        super::enrich_provider_import_result(provider_yike::fetch_public_preview(operation, &locator)?)
    })
    .await?;
    Ok(app_model::YikePreviewResultDto {
        identity,
        result,
        routes,
    })
}

#[tauri::command]
pub fn load_yike_locator(state: State<PreferencesState>) -> Result<Option<String>, String> {
    state.yike_locator()
}

#[tauri::command]
pub fn save_yike_locator(
    app: AppHandle,
    state: State<PreferencesState>,
    locator: Option<String>,
) -> Result<Option<String>, String> {
    state.save_yike_locator(&super::app_preferences_path(&app)?, locator)
}

/// First batch of a nickname or UID lookup. Validation precedes any I/O.
#[tauri::command]
pub async fn provider_fox_list(
    state: State<'_, PreferencesState>,
    current: State<'_, crate::current_game_state::CurrentGameState>,
    identity: ProviderRequestIdentityDto,
    lookup: FoxLookupDto,
) -> Result<FoxListResultDto, ProviderError> {
    let lookup = provider_fox::parse_lookup(&lookup)?;
    let (result, routes) = read_provider(&state, &current, &identity, move |operation| {
        provider_fox::fetch_account_list(operation, &lookup)
    })
    .await?;
    Ok(FoxListResultDto {
        identity,
        result,
        routes,
    })
}

/// Next batch for the account the list resolved; the cursor is the previous batch's last chessid.
#[tauri::command]
pub async fn provider_fox_list_more(
    state: State<'_, PreferencesState>,
    current: State<'_, crate::current_game_state::CurrentGameState>,
    identity: ProviderRequestIdentityDto,
    account: FoxAccountDto,
    cursor: String,
) -> Result<FoxListResultDto, ProviderError> {
    let (result, routes) = read_provider(&state, &current, &identity, move |operation| {
        provider_fox::fetch_list_continuation(operation, &account, &cursor)
    })
    .await?;
    Ok(FoxListResultDto {
        identity,
        result,
        routes,
    })
}

/// Fetches and parses one game for preview. Nothing is installed; Import goes through SGF-07.
#[tauri::command]
pub async fn provider_fox_preview(
    state: State<'_, PreferencesState>,
    current: State<'_, crate::current_game_state::CurrentGameState>,
    identity: ProviderRequestIdentityDto,
    chessid: String,
) -> Result<FoxPreviewResultDto, ProviderError> {
    let (result, routes) = read_provider(&state, &current, &identity, move |operation| {
        super::enrich_provider_import_result(provider_fox::fetch_game_preview(operation, &chessid)?)
    })
    .await?;
    Ok(FoxPreviewResultDto {
        identity,
        result,
        routes,
    })
}

#[tauri::command]
pub fn load_fox_kifu_state(state: State<PreferencesState>) -> Result<FoxKifuStateDto, String> {
    state.fox_kifu()
}

#[tauri::command]
pub fn remember_fox_lookup(
    app: AppHandle,
    state: State<PreferencesState>,
    lookup: FoxLookupDto,
    account: Option<FoxAccountDto>,
) -> Result<FoxKifuStateDto, String> {
    state.update_fox_kifu(&super::app_preferences_path(&app)?, |current| {
        provider_fox::remember_lookup(current, &lookup, account.as_ref()).map_err(|error| error.message)
    })
}

#[tauri::command]
pub fn clear_fox_recents(app: AppHandle, state: State<PreferencesState>) -> Result<FoxKifuStateDto, String> {
    state.update_fox_kifu(&super::app_preferences_path(&app)?, |current| {
        Ok(FoxKifuStateDto {
            recents: Vec::new(),
            last_query: current.last_query.clone(),
        })
    })
}

#[tauri::command]
pub async fn provider_tencent_list(
    state: State<'_, PreferencesState>,
    current: State<'_, crate::current_game_state::CurrentGameState>,
    identity: ProviderRequestIdentityDto,
    username: String,
    last_code: String,
) -> Result<app_model::TencentListResultDto, ProviderError> {
    let (result, routes) = read_provider(&state, &current, &identity, move |operation| {
        provider_tencent::fetch_list(operation, &username, &last_code)
    })
    .await?;
    Ok(app_model::TencentListResultDto {
        identity,
        result,
        routes,
    })
}

#[tauri::command]
pub async fn provider_tencent_preview(
    state: State<'_, PreferencesState>,
    current: State<'_, crate::current_game_state::CurrentGameState>,
    identity: ProviderRequestIdentityDto,
    chess_id: String,
) -> Result<app_model::TencentPreviewResultDto, ProviderError> {
    let (result, routes) = read_provider(&state, &current, &identity, move |operation| {
        super::enrich_provider_import_result(provider_tencent::fetch_preview(operation, &chess_id)?)
    })
    .await?;
    Ok(app_model::TencentPreviewResultDto {
        identity,
        result,
        routes,
    })
}

#[tauri::command]
pub fn load_tencent_history(state: State<PreferencesState>) -> Result<app_model::TencentHistoryDto, String> {
    state.tencent_history()
}

#[tauri::command]
pub fn save_tencent_query(
    app: AppHandle,
    state: State<PreferencesState>,
    query: Option<app_model::TencentQueryDto>,
) -> Result<app_model::TencentHistoryDto, String> {
    state.save_tencent_query(&super::app_preferences_path(&app)?, query)
}
