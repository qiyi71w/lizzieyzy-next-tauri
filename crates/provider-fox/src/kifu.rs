//! Fox kifu workflow: account lookup → cursor-paged game lists → chessid preview, plus the
//! Fox-owned recents state. Lists never become SGF; only a chessid fetch yields a game.

use crate::{
    cgi_sgf_requests, chess_list_request, h5_sgf_request, import_payload, is_decimal_id, is_fox_uid,
    json_i64, json_scalar_string, parse_user_info, query_user_request,
};
use app_model::{
    FoxAccountDto, FoxGameEntryDto, FoxKifuStateDto, FoxListPageDto, FoxLookupDto, FoxLookupKindDto,
    ProviderErrorKind, ProviderFetchRequest, ProviderFetchResult, ProviderImportRequest,
    ProviderImportResult, ProviderKind,
};
use provider_core::{
    checked_response, invalid_payload, invalid_request, provider_error, ProviderResult, ProviderTransport,
};
use serde_json::Value;
use std::collections::HashSet;

/// Upstream batch cap. The first upstream batch may contain one extra row; it is cut here and
/// reached again through the cursor, so every batch the UI sees holds at most this many games.
pub const FOX_LIST_BATCH_LIMIT: usize = 100;
pub const FOX_RECENTS_LIMIT: usize = 8;
const FOX_NICKNAME_MAX_CHARS: usize = 64;

/// Validates and trims one user lookup before any I/O.
pub fn parse_lookup(lookup: &FoxLookupDto) -> ProviderResult<FoxLookupDto> {
    let value = lookup.value.trim();
    let valid = match lookup.kind {
        FoxLookupKindDto::Nickname => is_nickname(value),
        FoxLookupKindDto::Uid => is_fox_uid(value),
        FoxLookupKindDto::Chessid => is_decimal_id(value),
    };
    if !valid {
        return Err(invalid_request(match lookup.kind {
            FoxLookupKindDto::Nickname => "Enter a Fox nickname.",
            FoxLookupKindDto::Uid => "A Fox UID is a positive number.",
            FoxLookupKindDto::Chessid => "A Fox chessid is a number.",
        }));
    }
    Ok(FoxLookupDto {
        kind: lookup.kind,
        value: value.to_string(),
    })
}

/// First batch for a nickname or UID lookup. A chessid lookup is a single game, not a list.
pub fn fetch_account_list<T: ProviderTransport + ?Sized>(
    transport: &T,
    lookup: &FoxLookupDto,
) -> ProviderResult<FoxListPageDto> {
    let lookup = parse_lookup(lookup)?;
    let account = match lookup.kind {
        FoxLookupKindDto::Uid => FoxAccountDto {
            uid: lookup.value,
            nickname: String::new(),
        },
        FoxLookupKindDto::Nickname => {
            let response = read(transport, query_user_request(&lookup.value)?)?;
            parse_user_info(&response.payload, &lookup.value)?
        }
        FoxLookupKindDto::Chessid => {
            return Err(invalid_request(
                "A chessid names one game; preview it instead of listing.",
            ))
        }
    };
    fetch_batch(transport, account, "0")
}

/// Continues a list from the frozen account and the chessid cursor of the previous batch.
pub fn fetch_list_continuation<T: ProviderTransport + ?Sized>(
    transport: &T,
    account: &FoxAccountDto,
    cursor: &str,
) -> ProviderResult<FoxListPageDto> {
    let uid = account.uid.trim();
    let cursor = cursor.trim();
    if !is_fox_uid(uid) || !is_decimal_id(cursor) {
        return Err(invalid_request(
            "Fox list continuation needs the listed account and cursor.",
        ));
    }
    let account = FoxAccountDto {
        uid: uid.to_string(),
        nickname: account.nickname.trim().to_string(),
    };
    fetch_batch(transport, account, cursor)
}

/// Fetches and normalizes one game. CGI endpoints and H5 are alternative read paths of the same
/// operation: transient failures move on under the operation's single retry budget, and a CGI
/// endpoint that answers with a miss (result code without a record, or an HTML page) yields to the
/// next path without consuming retries. Malformed responses are terminal on every path; a miss
/// at H5 means the record does not exist.
pub fn fetch_game_preview<T: ProviderTransport + ?Sized>(
    transport: &T,
    chessid: &str,
) -> ProviderResult<ProviderImportResult> {
    let chessid = chessid.trim();
    if !is_decimal_id(chessid) {
        return Err(invalid_request("A Fox chessid is a number."));
    }
    let mut requests = cgi_sgf_requests(chessid)?;
    requests.push(h5_sgf_request(chessid)?);
    let h5 = requests.len() - 1;
    let mut start = 0;
    loop {
        let response = transport.fetch_alternatives(&requests[start..])?;
        let answered = start + answering_endpoint(&requests[start..], &response);
        match classify_record(&response.payload, answered < h5)? {
            RecordResponse::Miss if answered < h5 => {
                start = answered + 1;
                continue;
            }
            RecordResponse::Miss => {
                return Err(provider_error(
                    ProviderErrorKind::NotFound,
                    "Fox has no game record for this chessid.",
                ))
            }
            RecordResponse::Record => {}
        }
        let mut metadata = response.metadata;
        metadata.source_id = Some(chessid.to_string());
        return import_payload(ProviderImportRequest {
            provider: ProviderKind::Fox,
            payload: response.payload,
            source_url: None,
            source_id: Some(chessid.to_string()),
            metadata,
        });
    }
}

/// Records a lookup: the last query always, and the account at the front of the recents when a
/// list resolved one. Recents dedupe by UID or identical nickname and keep at most eight.
pub fn remember_lookup(
    state: &FoxKifuStateDto,
    lookup: &FoxLookupDto,
    account: Option<&FoxAccountDto>,
) -> ProviderResult<FoxKifuStateDto> {
    let lookup = parse_lookup(lookup)?;
    let mut next = sanitize_state(state);
    next.last_query = Some(lookup);
    if let Some(account) = account {
        let account =
            sanitize_account(account).ok_or_else(|| invalid_request("Fox recent account is invalid."))?;
        next.recents.retain(|existing| {
            existing.uid != account.uid
                && (account.nickname.is_empty() || existing.nickname != account.nickname)
        });
        next.recents.insert(0, account);
        next.recents.truncate(FOX_RECENTS_LIMIT);
    }
    Ok(next)
}

/// Drops entries that could not have been written by this owner (bad UIDs, oversize names) and
/// enforces the cap, so a hand-edited preference file cannot widen the stored surface.
pub fn sanitize_state(state: &FoxKifuStateDto) -> FoxKifuStateDto {
    let mut seen = HashSet::new();
    FoxKifuStateDto {
        recents: state
            .recents
            .iter()
            .filter_map(sanitize_account)
            .filter(|account| seen.insert(account.uid.clone()))
            .take(FOX_RECENTS_LIMIT)
            .collect(),
        last_query: state
            .last_query
            .as_ref()
            .and_then(|lookup| parse_lookup(lookup).ok()),
    }
}

fn sanitize_account(account: &FoxAccountDto) -> Option<FoxAccountDto> {
    let uid = account.uid.trim();
    let nickname = account.nickname.trim();
    (is_fox_uid(uid) && (nickname.is_empty() || is_nickname(nickname))).then(|| FoxAccountDto {
        uid: uid.to_string(),
        nickname: nickname.to_string(),
    })
}

fn is_nickname(value: &str) -> bool {
    !value.is_empty()
        && value.chars().count() <= FOX_NICKNAME_MAX_CHARS
        && !value.chars().any(char::is_control)
}

fn read<T: ProviderTransport + ?Sized>(
    transport: &T,
    request: ProviderFetchRequest,
) -> ProviderResult<ProviderFetchResult> {
    transport.fetch(&request).and_then(checked_response)
}

fn fetch_batch<T: ProviderTransport + ?Sized>(
    transport: &T,
    mut account: FoxAccountDto,
    cursor: &str,
) -> ProviderResult<FoxListPageDto> {
    let response = read(transport, chess_list_request(&account.uid, cursor)?)?;
    let json: Value = serde_json::from_str(response.payload.trim())
        .map_err(|_| invalid_payload("Fox game list is not JSON."))?;
    if json.get("result").and_then(json_i64).unwrap_or(-1) != 0 {
        return Err(invalid_payload("Fox rejected the game list request."));
    }
    let rows = json
        .get("chesslist")
        .and_then(Value::as_array)
        .ok_or_else(|| invalid_payload("Fox game list has no chesslist."))?;
    let has_more = rows.len() >= FOX_LIST_BATCH_LIMIT;
    let mut seen = HashSet::new();
    let mut games = Vec::new();
    for row in rows.iter().take(FOX_LIST_BATCH_LIMIT) {
        let game = parse_game(row, &account.uid)?;
        // The cursor row itself is never part of the continuation.
        if game.chessid != cursor && seen.insert(game.chessid.clone()) {
            games.push(game);
        }
    }
    if account.nickname.is_empty() {
        account.nickname = games
            .iter()
            .find_map(|game| {
                if game.black_uid == account.uid {
                    Some(game.black_name.clone())
                } else if game.white_uid == account.uid {
                    Some(game.white_name.clone())
                } else {
                    None
                }
            })
            .unwrap_or_default();
    }
    let next_cursor = games.last().map(|game| game.chessid.clone());
    Ok(FoxListPageDto {
        account,
        games,
        has_more: has_more && next_cursor.is_some(),
        next_cursor,
    })
}

fn parse_game(row: &Value, account_uid: &str) -> ProviderResult<FoxGameEntryDto> {
    let text = |key: &str| row.get(key).and_then(json_scalar_string).unwrap_or_default();
    let number = |key: &str| row.get(key).and_then(json_i64);
    let chessid = text("chessid");
    if !is_decimal_id(&chessid) {
        return Err(invalid_payload("Fox game list row has no chessid."));
    }
    let black_uid = text("blackuid");
    let white_uid = text("whiteuid");
    let winner = number("winner").unwrap_or(0);
    let account_won = match winner {
        1 if black_uid == account_uid => Some(true),
        2 if white_uid == account_uid => Some(true),
        1 | 2 if black_uid == account_uid || white_uid == account_uid => Some(false),
        _ => None,
    };
    Ok(FoxGameEntryDto {
        chessid,
        start_time: text("starttime"),
        title: text("title"),
        black_name: player_name(row, ["blacknick", "blacknickname", "blackname", "blackenname"]),
        black_uid,
        black_rank: rank_label(number("blackdan")),
        white_name: player_name(row, ["whitenick", "whitenickname", "whitename", "whiteenname"]),
        white_uid,
        white_rank: rank_label(number("whitedan")),
        result: result_label(winner, number("point").unwrap_or(0), number("rule").unwrap_or(-1)),
        move_count: number("movenum")
            .and_then(|value| u32::try_from(value).ok())
            .unwrap_or(0),
        board_size: number("boardsize").and_then(|value| u8::try_from(value).ok()),
        account_won,
    })
}

fn player_name(row: &Value, keys: [&str; 4]) -> String {
    keys.iter()
        .filter_map(|key| row.get(*key).and_then(json_scalar_string))
        .find(|value| !value.is_empty())
        .unwrap_or_else(|| "未知昵称".to_string())
}

/// Amateur levels follow the frozen Java mapping (`dan - 17`). Fox encodes professionals from
/// 100 upward (its own SGF renders 100 as `P1段`, 108 as `P9段`); the Java mapping would print
/// e.g. 83段 for those.
fn rank_label(dan: Option<i64>) -> String {
    match dan {
        None => String::new(),
        Some(dan) if dan >= 100 => format!("职业{}段", dan - 99),
        Some(dan) if dan - 17 > 0 => format!("{}段", dan - 17),
        Some(dan) => format!("{}级", (dan - 17).abs() + 1),
    }
}

fn result_label(winner: i64, point: i64, rule: i64) -> String {
    let side = match winner {
        1 => "黑",
        2 => "白",
        _ => return "其他".to_string(),
    };
    match point {
        -1 => format!("{side}中盘胜"),
        -2 => format!("{side}超时胜"),
        point if point < 0 => format!("{side}胜"),
        point => {
            let unit = match rule {
                1 => "子",
                0 => "目",
                _ => "",
            };
            let whole = point / 100;
            let fraction = point % 100;
            let score = if fraction == 0 {
                whole.to_string()
            } else {
                format!("{whole}.{fraction:02}").trim_end_matches('0').to_string()
            };
            format!("{side}胜{score}{unit}")
        }
    }
}

/// Which slice entry produced `response`. The transport reports the sanitized request URL (query
/// removed); an unidentifiable response is attributed to the first entry.
fn answering_endpoint(requests: &[ProviderFetchRequest], response: &ProviderFetchResult) -> usize {
    let answered = response.metadata.request_url.as_deref().unwrap_or(&response.url);
    requests
        .iter()
        .position(|request| request.url.split('?').next() == Some(answered))
        .unwrap_or(0)
}

enum RecordResponse {
    Record,
    Miss,
}

/// A record is a JSON object with `result` 0 and non-empty `chess`. A miss is a JSON object with a
/// `result` code but no record; the second CGI endpoint also answers misses with an HTML page.
/// Anything else is malformed and terminal on every path.
fn classify_record(payload: &str, html_is_miss: bool) -> ProviderResult<RecordResponse> {
    let payload = payload.trim();
    if html_is_miss && payload.starts_with('<') {
        return Ok(RecordResponse::Miss);
    }
    let json: Value =
        serde_json::from_str(payload).map_err(|_| invalid_payload("Fox game record is not JSON."))?;
    let has_chess = json
        .get("chess")
        .and_then(json_scalar_string)
        .is_some_and(|chess| !chess.is_empty());
    match json.get("result").and_then(json_i64) {
        Some(0) if has_chess => Ok(RecordResponse::Record),
        Some(_) if !has_chess => Ok(RecordResponse::Miss),
        _ => Err(invalid_payload("Fox game record is malformed.")),
    }
}
