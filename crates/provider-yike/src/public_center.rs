use super::*;
use app_model::{ProviderError, ProviderErrorKind, YikeCategoryDto, YikeListEntryDto, YikeListPageDto};
use provider_core::{invalid_request, provider_error};
use std::collections::BTreeSet;
use url::Url;

struct PublicLocator {
    descriptor: YikeUrlDescriptor,
    canonical: String,
}

pub fn canonical_yike_locator(locator: &str) -> ProviderResult<String> {
    parse_public_locator(locator).map(|locator| locator.canonical)
}

pub fn fetch_public_list<T: ProviderTransport + ?Sized>(
    transport: &T,
    category: YikeCategoryDto,
    page: u32,
    since: u64,
) -> ProviderResult<YikeListPageDto> {
    if page == 0 || page > i32::MAX as u32 || since > i64::MAX as u64 {
        return Err(invalid_request(
            "Yike page or cursor is outside the supported range",
        ));
    }
    let official = match category {
        YikeCategoryDto::Recommend => "1",
        YikeCategoryDto::Local => "0",
    };
    let response = transport.fetch(&signed_get_request(
        live_list_url(Some(official), i64::from(page), since as i64),
        None,
        None,
        YikeRequestSignature::now(),
    ))?;
    ensure_http_success(&response, "Yike public list request failed")?;
    let parsed = parse_live_list_json(&response.payload)?;
    // The supported envelope has only list/since, not a total or next-page flag.
    // Only an actually empty upstream page proves the end, not a deduplicated page.
    let has_more = !parsed.games.is_empty();
    let mut seen = BTreeSet::new();
    let mut games = Vec::with_capacity(parsed.games.len());
    for game in parsed.games {
        let mut locator = game.to_room_url();
        if game.hall == 0 && game.room == 0 {
            locator.truncate(locator.len() - 4);
        }
        if seen.insert(locator.clone()) {
            let status = game.status_text();
            games.push(YikeListEntryDto {
                locator,
                title: game.game_name,
                black_name: game.black_name,
                white_name: game.white_name,
                status,
                move_count: game.hands_count,
            });
        }
    }
    Ok(YikeListPageDto {
        category,
        page,
        since: parsed.since,
        games,
        has_more,
    })
}

pub fn fetch_public_preview<T: ProviderTransport + ?Sized>(
    transport: &T,
    locator: &str,
) -> ProviderResult<ProviderImportResult> {
    let locator = parse_public_locator(locator)?;
    let descriptor = locator.descriptor;
    let mut request = signed_get_request(
        descriptor.request_url.clone(),
        Some(locator.canonical.clone()),
        Some(descriptor.id.clone()),
        YikeRequestSignature::now(),
    );
    if descriptor.room_kind == YikeRoomKind::UniteRoom {
        request.headers.insert(
            "Authorization".to_string(),
            anonymous_game_authorization(transport)?,
        );
    }
    let response = transport.fetch(&request)?;
    ensure_http_success(&response, "Yike public preview request failed")?;
    let root = parse_json(&response.payload, "Yike public preview")?;
    let detail = match descriptor.room_kind {
        YikeRoomKind::NewLiveRoom => parse_live_detail_value(&root)?,
        YikeRoomKind::OldLiveRoom | YikeRoomKind::OldLiveBoard | YikeRoomKind::GameRoom => {
            parse_old_detail_value(&root)?
        }
        YikeRoomKind::UniteRoom => {
            let detail = parse_unite_detail_value(&root)?;
            if root
                .get("data")
                .and_then(|data| data.get("id"))
                .and_then(value_u64)
                != Some(descriptor.room_id)
            {
                return Err(invalid_payload(
                    "Yike unite response does not match the requested room",
                ));
            }
            detail
        }
    };
    let mut result = detail_to_import_result(
        detail,
        Some(descriptor.id),
        Some(locator.canonical),
        Some(descriptor.request_url),
    )?;
    result.metadata.room_id = Some(descriptor.room_id.to_string());
    if descriptor.room_kind == YikeRoomKind::UniteRoom {
        result.metadata.provider_status = root.get("data").and_then(|data| object_string(data, "status"));
    }
    Ok(result)
}

fn anonymous_game_authorization<T: ProviderTransport + ?Sized>(transport: &T) -> ProviderResult<String> {
    let mut request = signed_get_request(
        "https://api-new.yikeweiqi.com/v1/user/tpop/authorizations/anonymous".to_string(),
        None,
        None,
        YikeRequestSignature::now(),
    );
    request.method = ProviderFetchMethod::Post;
    request.body = Some(r#"{"tpop_name":"og","refresh_flag":0}"#.to_string());
    let response = transport.fetch(&request)?;
    ensure_http_success(&response, "Yike anonymous public access request failed")?;
    let root = parse_json(&response.payload, "Yike anonymous public access")?;
    validate_status(&root, "status", 0)?;
    let token = root
        .get("result")
        .and_then(|result| result.get("token"))
        .and_then(Value::as_str)
        .filter(|token| !token.is_empty() && token.bytes().all(|byte| byte.is_ascii_graphic()))
        .ok_or_else(|| invalid_payload("Yike anonymous public access response is missing a valid token"))?;
    // This guest credential belongs only to this operation. It never reaches
    // preferences, browser storage, diagnostics or the returned import DTO.
    Ok(format!("Bearer {token}"))
}

pub(super) fn parse_descriptor(raw: &str) -> ProviderResult<YikeUrlDescriptor> {
    parse_public_locator(raw).map(|locator| locator.descriptor)
}

fn locator_id(raw: &str, allow_zero: bool) -> ProviderResult<u64> {
    if raw.is_empty() || !raw.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(invalid_url("Yike locator IDs must be decimal integers"));
    }
    let id = raw
        .parse::<u64>()
        .map_err(|_| invalid_url("Yike locator ID is outside the supported range"))?;
    if (!allow_zero && id == 0) || id > i64::MAX as u64 {
        return Err(invalid_url("Yike locator ID is outside the supported range"));
    }
    Ok(id)
}

fn parse_public_locator(raw: &str) -> ProviderResult<PublicLocator> {
    let raw = raw.trim();
    if raw.is_empty()
        || raw.chars().any(|ch| ch.is_whitespace() || ch.is_control())
        || raw.contains('\\')
        || raw.contains('%')
        || raw.contains("/./")
        || raw.contains("/../")
        || raw.split_once("://").is_some_and(|(_, rest)| {
            rest.split(['/', '?', '#'])
                .next()
                .is_some_and(|authority| authority.contains('@'))
        })
    {
        return Err(invalid_url("invalid Yike public locator"));
    }
    let url = Url::parse(raw).map_err(|_| invalid_url("invalid Yike public locator"))?;
    if !matches!(url.scheme(), "http" | "https")
        || !url.username().is_empty()
        || url.password().is_some()
        || url.port().is_some()
        || !matches!(
            url.host_str(),
            Some("home.yikeweiqi.com" | "unite.yikeweiqi.com" | "www.yikeweiqi.com" | "yikeweiqi.com")
        )
    {
        return Err(invalid_url("unsupported Yike public origin"));
    }
    // Public routes carry no credentials. Reject all undeclared query keys rather
    // than forwarding or persisting a token under an unexpected spelling.
    let route = match url.fragment() {
        Some(fragment) if url.path() == "/" && url.query().is_none() => fragment,
        None => url.path(),
        _ => return Err(invalid_url("unsupported Yike public locator")),
    };
    let (path, query) = match route.split_once('?') {
        Some((path, query)) => (path, Some(query)),
        None => (route, url.query()),
    };
    if matches!(path, "/" | "/hall" | "/hall/") && query.is_some() {
        let mut room = None;
        let mut hall = None;
        for (key, value) in url::form_urlencoded::parse(query.unwrap_or_default().as_bytes()) {
            match key.as_ref() {
                "room" if room.is_none() => room = Some(locator_id(&value, false)?),
                "hall" if hall.is_none() => {
                    hall = Some(match value.as_ref() {
                        "true" | "false" => value.into_owned(),
                        _ => locator_id(&value, true)?.to_string(),
                    });
                }
                _ => return Err(invalid_url("unsupported Yike locator query")),
            }
        }
        let room = room.ok_or_else(|| invalid_url("Yike hall locator requires room"))?;
        let hall = hall.ok_or_else(|| invalid_url("Yike hall locator requires hall"))?;
        return Ok(PublicLocator {
            descriptor: YikeUrlDescriptor {
                provider: ProviderKind::Yike,
                room_kind: YikeRoomKind::GameRoom,
                id: room.to_string(),
                room_id: room,
                request_url: format!("https://api.yikeweiqi.com/golive/dtl?id={room}"),
            },
            canonical: format!("https://home.yikeweiqi.com/#/?room={room}&hall={hall}"),
        });
    }
    if query.is_some() {
        return Err(invalid_url("unsupported Yike locator query"));
    }
    let segments: Vec<&str> = path.strip_prefix('/').unwrap_or(path).split('/').collect();
    let (room_kind, id, room_id, canonical, request_url) = match segments.as_slice() {
        ["live", family @ ("new-room" | "room"), raw_id, tail @ ..] if tail.is_empty() || tail.len() == 2 => {
            let id = locator_id(raw_id, false)?;
            let (hall, room) = if tail.is_empty() {
                (0, 0)
            } else {
                (locator_id(tail[0], true)?, locator_id(tail[1], true)?)
            };
            if room == 0 && hall != 0 {
                return Err(invalid_url("Yike room locator requires a positive room ID"));
            }
            let suffix = if hall == 0 && room == 0 {
                String::new()
            } else {
                format!("/{hall}/{room}")
            };
            let kind = if *family == "new-room" {
                YikeRoomKind::NewLiveRoom
            } else if room == 0 {
                YikeRoomKind::OldLiveBoard
            } else {
                YikeRoomKind::OldLiveRoom
            };
            let request = if *family == "new-room" {
                live_detail_url(&id.to_string())
            } else {
                format!(
                    "https://api.yikeweiqi.com/golive/dtl?id={id}{}",
                    if kind == YikeRoomKind::OldLiveRoom {
                        "&flag=1"
                    } else {
                        ""
                    }
                )
            };
            (
                kind,
                id,
                if room == 0 { id } else { room },
                format!("https://home.yikeweiqi.com/#/live/{family}/{id}{suffix}"),
                request,
            )
        }
        ["game", mode, raw_hall, raw_room]
            if !mode.is_empty() && mode.bytes().all(|byte| byte.is_ascii_alphabetic()) =>
        {
            let hall = locator_id(raw_hall, true)?;
            let room = locator_id(raw_room, false)?;
            (
                YikeRoomKind::GameRoom,
                room,
                room,
                format!("https://home.yikeweiqi.com/#/game/{mode}/{hall}/{room}"),
                format!("https://api.yikeweiqi.com/golive/dtl?id={room}"),
            )
        }
        ["unite", raw_id] => {
            let id = locator_id(raw_id, false)?;
            (YikeRoomKind::UniteRoom, id, id,
                format!("https://home.yikeweiqi.com/#/unite/{id}"),
                format!("https://game-server.yikeweiqi.com/game/info?id={id}&sgf_option=true&players_option=true&setting_option=true&clock_option=true&is_void=true"))
        }
        _ => return Err(invalid_url("unsupported Yike public locator")),
    };
    Ok(PublicLocator {
        descriptor: YikeUrlDescriptor {
            provider: ProviderKind::Yike,
            room_kind,
            id: id.to_string(),
            room_id,
            request_url,
        },
        canonical,
    })
}

pub(super) fn validate_status(root: &Value, key: &str, success: i64) -> ProviderResult<()> {
    let status = object_i64(root, key)
        .ok_or_else(|| invalid_payload("Yike response is missing a valid envelope status"))?;
    if status == success {
        return Ok(());
    }
    let kind = match status {
        401 | 403 | 1401 | 1403 => ProviderErrorKind::AuthenticationFailed,
        404 | 1404 => ProviderErrorKind::NotFound,
        _ => ProviderErrorKind::InvalidPayload,
    };
    Err(provider_error(
        kind,
        format!("Yike public response failed with status {status}"),
    ))
}

pub(super) fn parse_old_detail_value(root: &Value) -> ProviderResult<YikeLiveDetail> {
    validate_status(root, "Status", 1200)?;
    let live = root
        .get("Result")
        .and_then(|result| result.get("live"))
        .filter(|live| live.is_object())
        .ok_or_else(|| invalid_payload("Yike old detail response is missing live"))?;
    let sgf = live
        .get("Content")
        .and_then(Value::as_str)
        .filter(|sgf| !sgf.trim().is_empty())
        .ok_or_else(|| invalid_payload("Yike old detail response is missing SGF"))?;
    Ok(YikeLiveDetail {
        sgf: sgf.to_string(),
        status: object_i64(live, "Status").unwrap_or(0),
        game_result: object_string(live, "GameResult").unwrap_or_default(),
    })
}

pub(super) fn parse_unite_detail_value(root: &Value) -> ProviderResult<YikeLiveDetail> {
    // The public game-server reports code=1 for an invalid guest token.
    if object_i64(root, "code") == Some(1) {
        return Err(provider_error(
            ProviderErrorKind::AuthenticationFailed,
            "Yike unite public access was rejected",
        ));
    }
    for key in ["code", "status"] {
        if root.get(key).is_some() {
            validate_status(root, key, 0)?;
        }
    }
    if root.get("Status").is_some() {
        validate_status(root, "Status", 1200)?;
    }
    let data = root
        .get("data")
        .filter(|data| data.is_object())
        .ok_or_else(|| invalid_payload("Yike unite response is missing data"))?;
    if data.get("game_type").and_then(Value::as_str) != Some("territory") {
        return Err(invalid_payload("Yike unite source is not a Go game"));
    }
    let mut detail = YikeLiveDetail::from_value(data);
    detail.game_result = object_string(data, "result").unwrap_or_default();
    if detail.sgf.trim().is_empty() {
        return Err(invalid_payload("Yike unite response is missing SGF"));
    }
    let mut document = sgf::parse_sgf(&detail.sgf)
        .map_err(|_| invalid_payload("Yike unite response contains invalid SGF"))?;
    let sgf_root = document
        .root
        .as_mut()
        .ok_or_else(|| invalid_payload("Yike unite response contains invalid SGF"))?;
    let mut changed = false;
    for (key, side) in [("PB", "blacks"), ("PW", "whites")] {
        if sgf_root.properties.iter().any(|property| property.key == key) {
            continue;
        }
        let mut names = String::new();
        if let Some(players) = data
            .get("players")
            .and_then(|players| players.get(side))
            .and_then(Value::as_array)
        {
            for name in players
                .iter()
                .filter_map(|player| player.get("name").and_then(Value::as_str))
            {
                if name.trim().is_empty() {
                    continue;
                }
                if !names.is_empty() {
                    names.push(',');
                }
                names.push_str(name);
            }
        }
        if !names.is_empty() {
            sgf_root.properties.push(sgf::SgfProperty {
                key: key.to_string(),
                values: vec![names],
            });
            changed = true;
        }
    }
    if !detail.game_result.trim().is_empty()
        && !sgf_root.properties.iter().any(|property| property.key == "RE")
    {
        sgf_root.properties.push(sgf::SgfProperty {
            key: "RE".to_string(),
            values: vec![detail.game_result.clone()],
        });
        changed = true;
    }
    if changed {
        detail.sgf = sgf::serialize_sgf_document(&document)
            .map_err(|_| invalid_payload("Yike unite response contains invalid SGF"))?;
    }
    if let Some(result) = document.result {
        detail.game_result = result;
    }
    Ok(detail)
}

pub(super) fn http_error(status: u16, context: &str) -> ProviderError {
    let kind = match status {
        401 | 403 => ProviderErrorKind::AuthenticationFailed,
        404 => ProviderErrorKind::NotFound,
        _ => ProviderErrorKind::TransportFailed,
    };
    provider_error(kind, format!("{context}: HTTP {status}"))
}
