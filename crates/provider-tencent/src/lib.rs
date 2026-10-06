use app_model::{
    ProviderFetchMethod, ProviderFetchRequest, ProviderGameMetadata, ProviderGameSummary,
    ProviderImportRequest, ProviderImportResult, ProviderKind, TencentListEntryDto, TencentListPageDto,
};
use provider_core::{
    checked_response, invalid_payload, invalid_request, parse_failed, require_non_blank, ProviderResult,
    ProviderTransport,
};
use serde_json::Value;
use std::borrow::Cow;
use std::collections::{BTreeMap, HashSet};

const LIST_URL: &str = "https://cgi.huanle.qq.com/cgi-bin/CommonMobileCGI/TXWQFetchChessList";
const DETAIL_URL: &str = "https://happyapp.huanle.qq.com/cgi-bin/CommonMobileCGI/TXWQFetchChess";
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
    (KHTML, like Gecko) Chrome/124.0 Safari/537.36";
const BATCH_SIZE: usize = 100;

pub fn fetch_list<T: ProviderTransport + ?Sized>(
    transport: &T,
    username: &str,
    last_code: &str,
) -> ProviderResult<TencentListPageDto> {
    let username = require_non_blank(username, "Tencent username")?;
    let last_code = match last_code.trim() {
        "" => "0",
        value => value,
    };
    let srcuid = if username.bytes().all(|byte| byte.is_ascii_digit()) {
        username
    } else {
        ""
    };
    let request = get_request(
        format!(
            "{LIST_URL}?type=7&lastCode={}&username={}&srcuid={}&txwqsession=lizzieyzy-next&fetchnum=100",
            url_encode(last_code),
            url_encode(username),
            url_encode(srcuid),
        ),
        None,
    );
    let response = checked_response(transport.fetch(&request)?)?;
    let json = parse_envelope(&response.payload)?;
    let raw_games = json
        .get("chesslist")
        .and_then(Value::as_array)
        .ok_or_else(|| invalid_payload("Tencent response does not contain a chess list."))?;
    if raw_games.len() > BATCH_SIZE {
        return Err(invalid_payload("Tencent chess list exceeds the batch limit."));
    }

    let mut games = Vec::with_capacity(raw_games.len());
    let mut seen: HashSet<Cow<'_, str>> = HashSet::with_capacity(raw_games.len());
    // The upstream cursor belongs to the raw tail, not the last unique display row.
    let next_code = raw_games
        .last()
        .map(chess_id)
        .transpose()?
        .map(Cow::into_owned)
        .unwrap_or_else(|| last_code.to_string());
    for raw in raw_games {
        let chess_id = chess_id(raw)?;
        if seen.contains(chess_id.as_ref()) {
            continue;
        }
        games.push(TencentListEntryDto {
            chess_id: chess_id.to_string(),
            black_name: player_name(raw, ["blacknick", "blackenname", "blackname", "blackuid"]),
            white_name: player_name(raw, ["whitenick", "whiteenname", "whitename", "whiteuid"]),
            black_rank: rank(integer(raw.get("blackdan")).unwrap_or(0)),
            white_rank: rank(integer(raw.get("whitedan")).unwrap_or(0)),
            played_at: non_blank_scalar(raw.get("starttime"))
                .or_else(|| non_blank_scalar(raw.get("gamestarttime")))
                .unwrap_or_default(),
            result: result_text(raw),
            move_count: integer(raw.get("movenum"))
                .and_then(|value| u32::try_from(value).ok())
                .unwrap_or(0),
        });
        seen.insert(chess_id);
    }
    // Public upstream batches can be short even when older games remain available.
    let has_more = !raw_games.is_empty() && next_code != last_code;
    Ok(TencentListPageDto {
        username: username.to_string(),
        last_code: next_code,
        games,
        has_more,
    })
}

pub fn fetch_preview<T: ProviderTransport + ?Sized>(
    transport: &T,
    chess_id: &str,
) -> ProviderResult<ProviderImportResult> {
    let chess_id = require_non_blank(chess_id, "Tencent chess ID")?;
    let request = get_request(
        format!("{DETAIL_URL}?chessid={}", url_encode(chess_id)),
        Some(chess_id.to_string()),
    );
    let response = checked_response(transport.fetch(&request)?)?;
    let json = parse_envelope(&response.payload)?;
    parsed_import(chess_text(&json)?, Some(chess_id.to_string()))
}

pub fn import_payload(request: ProviderImportRequest) -> ProviderResult<ProviderImportResult> {
    if request.provider != ProviderKind::Tencent {
        return Err(invalid_request("Tencent import requires the Tencent provider."));
    }
    let payload = require_non_blank(&request.payload, "Tencent payload")?;
    let source_id = request
        .source_id
        .filter(|value| !value.trim().is_empty())
        .map(|value| value.trim().to_string());
    let raw_sgf = payload.trim_start_matches(|c: char| c == '\u{feff}' || c.is_whitespace());
    if raw_sgf.starts_with('(') || raw_sgf.starts_with('\\') {
        return parsed_import(payload, source_id);
    }
    let json = parse_envelope(payload)?;
    let source_id = source_id.or_else(|| non_blank_scalar(json.get("chessid")));
    // Caller/response metadata is not trusted: URLs, sessions and raw diagnostics stay out.
    parsed_import(chess_text(&json)?, source_id)
}

fn get_request(url: String, source_id: Option<String>) -> ProviderFetchRequest {
    ProviderFetchRequest {
        provider: ProviderKind::Tencent,
        url,
        method: ProviderFetchMethod::Get,
        headers: BTreeMap::from([
            (
                "Accept".to_string(),
                "application/json,text/plain,*/*".to_string(),
            ),
            ("Connection".to_string(), "close".to_string()),
            ("User-Agent".to_string(), USER_AGENT.to_string()),
        ]),
        body: None,
        source_url: None,
        source_id,
        timeout_ms: Some(25_000),
    }
}

fn parse_envelope(payload: &str) -> ProviderResult<Value> {
    let json: Value =
        serde_json::from_str(payload).map_err(|_| invalid_payload("Tencent response is not valid JSON."))?;
    if !json.is_object() {
        return Err(invalid_payload("Tencent response must be a JSON object."));
    }
    for key in ["result", "ret"] {
        if let Some(value) = json.get(key) {
            if integer(Some(value)) != Some(0) {
                return Err(invalid_payload("Tencent response did not report success."));
            }
        }
    }
    Ok(json)
}

fn chess_id(raw: &Value) -> ProviderResult<Cow<'_, str>> {
    match raw.get("chessid") {
        Some(Value::String(value)) if !value.trim().is_empty() => Ok(Cow::Borrowed(value.trim())),
        Some(Value::Number(value)) => Ok(Cow::Owned(value.to_string())),
        _ => Err(invalid_payload(
            "Tencent chess list contains a missing or invalid chess ID.",
        )),
    }
}

fn chess_text(json: &Value) -> ProviderResult<&str> {
    json.get("chess")
        .and_then(Value::as_str)
        .filter(|text| !text.trim().is_empty())
        .ok_or_else(|| invalid_payload("Tencent response does not contain chess SGF text."))
}

fn parsed_import(sgf_text: &str, source_id: Option<String>) -> ProviderResult<ProviderImportResult> {
    let mut sgf_text = sanitize_sgf(sgf_text);
    let mut document =
        sgf::parse_sgf(&sgf_text).map_err(|_| parse_failed("Tencent chess SGF could not be parsed."))?;
    if normalize_root_komi(&mut document) {
        sgf_text = sgf::serialize_sgf_document(&document)
            .map_err(|_| parse_failed("Tencent chess SGF could not be parsed."))?;
        document =
            sgf::parse_sgf(&sgf_text).map_err(|_| parse_failed("Tencent chess SGF could not be parsed."))?;
    }
    let date = document.root.as_ref().and_then(|root| {
        root.properties
            .iter()
            .find(|property| property.key == "DT")
            .and_then(|property| property.values.first())
            .cloned()
    });
    let summary = ProviderGameSummary {
        provider: ProviderKind::Tencent,
        source_id: source_id.clone(),
        board_width: Some(document.board_width),
        board_height: Some(document.board_height),
        komi: Some(document.komi),
        handicap: document.handicap,
        black_name: document.black_name,
        white_name: document.white_name,
        result: document.result,
        date,
        move_count: Some(document.moves.len()),
    };
    Ok(ProviderImportResult {
        provider: ProviderKind::Tencent,
        sgf_text,
        summary,
        metadata: ProviderGameMetadata {
            source_id,
            ..ProviderGameMetadata::default()
        },
        warnings: Vec::new(),
    })
}

// Tencent downloads enter the frozen Java shared parser, whose root komi unit
// conversion is independent of provider identity. Never use Fox tree normalization.
fn normalize_root_komi(document: &mut sgf::SgfDocument) -> bool {
    let Some(root) = document.root.as_mut() else {
        return false;
    };
    let fox_branded = root_contains(root, "AP", "foxwq");
    let chinese = root_contains(root, "RU", "chinese");
    let Some(property) = root.properties.iter_mut().find(|property| property.key == "KM") else {
        return false;
    };
    let Some(raw) = property
        .values
        .first()
        .and_then(|value| value.trim().parse::<f64>().ok())
    else {
        return false;
    };
    if !raw.is_finite() {
        return false;
    }
    let mut normalized = raw;
    if fox_branded {
        if normalized.abs() >= 50.0 {
            normalized /= 100.0;
            if chinese && (-4.0..=4.0).contains(&normalized) {
                normalized *= 2.0;
            }
        }
    } else {
        if normalized >= 200.0 {
            normalized /= 100.0;
            if (-4.0..=4.0).contains(&normalized) {
                normalized *= 2.0;
            }
        }
        let fraction = normalized.fract().abs();
        if fraction == 0.75 || fraction == 0.25 {
            normalized *= 2.0;
        }
    }
    if normalized == raw {
        return false;
    }
    property.values[0] = normalized.to_string();
    true
}

fn root_contains(root: &sgf::SgfNode, key: &str, needle: &str) -> bool {
    root.properties
        .iter()
        .filter(|property| property.key == key)
        .flat_map(|property| &property.values)
        .any(|value| {
            value
                .as_bytes()
                .windows(needle.len())
                .any(|part| part.eq_ignore_ascii_case(needle.as_bytes()))
        })
}

fn sanitize_sgf(input: &str) -> String {
    let text = input.trim_matches(|c: char| c == '\u{feff}' || c.is_whitespace());
    let mut output = String::with_capacity(text.len());
    let mut chars = text.chars().filter(|c| *c != '\u{feff}');
    let mut inside_value = false;
    while let Some(current) = chars.next() {
        if inside_value {
            output.push(current);
            if current == '\\' {
                if let Some(escaped) = chars.next() {
                    output.push(escaped);
                }
            } else if current == ']' {
                inside_value = false;
            }
        } else if current != '\\' {
            output.push(current);
            if current == '[' {
                inside_value = true;
            }
        }
    }
    output
}

fn non_blank_scalar(value: Option<&Value>) -> Option<String> {
    match value? {
        Value::String(value) if !value.trim().is_empty() => Some(value.trim().to_string()),
        Value::Number(value) => Some(value.to_string()),
        _ => None,
    }
}

fn integer(value: Option<&Value>) -> Option<i64> {
    match value? {
        Value::Number(value) => value.as_i64(),
        Value::String(value) => value.trim().parse().ok(),
        _ => None,
    }
}

fn player_name(raw: &Value, keys: [&str; 4]) -> String {
    keys.into_iter()
        .find_map(|key| non_blank_scalar(raw.get(key)))
        .unwrap_or_default()
}

fn rank(raw: i64) -> String {
    match raw {
        100.. => format!("P{}段", raw - 99),
        18..=99 => format!("{}段", raw - 17),
        1..=17 => format!("{}级", 18 - raw),
        _ => String::new(),
    }
}

fn result_text(raw: &Value) -> String {
    let winner = match integer(raw.get("winner")) {
        Some(1) => "Black",
        Some(2) => "White",
        _ => return "Other".to_string(),
    };
    let point = integer(raw.get("point")).unwrap_or(0);
    match point {
        -1 => format!("{winner} wins by resignation"),
        -2 => format!("{winner} wins on time"),
        ..=-3 => format!("{winner} wins"),
        _ => {
            let unit = if integer(raw.get("rule")) == Some(1) {
                "stones"
            } else {
                "points"
            };
            // Frozen point units are hundredths, not SGF komi or a synthetic RE property.
            let whole = point / 100;
            let fraction = point % 100;
            let score = match fraction {
                0 => whole.to_string(),
                value if value % 10 == 0 => format!("{whole}.{}", value / 10),
                value => format!("{whole}.{value:02}"),
            };
            format!("{winner} +{score} {unit}")
        }
    }
}

fn url_encode(value: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut output = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'.' | b'-' | b'*' | b'_' => output.push(byte as char),
            b' ' => output.push('+'),
            _ => {
                output.push('%');
                output.push(HEX[(byte >> 4) as usize] as char);
                output.push(HEX[(byte & 15) as usize] as char);
            }
        }
    }
    output
}
