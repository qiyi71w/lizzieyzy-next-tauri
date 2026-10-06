use app_model::{
    FoxAccountDto, ProviderFetchMethod, ProviderFetchRequest, ProviderGameMetadata, ProviderGameSummary,
    ProviderImportRequest, ProviderImportResult, ProviderKind,
};
use provider_core::{first_non_blank, invalid_payload, provider_error, require_non_blank, ProviderResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

mod kifu;
pub use kifu::*;

pub const FOX_BASE_URL: &str = "https://h5.foxwq.com/yehuDiamond/chessbook_local";
pub const FOX_QUERY_USER_URL: &str = "https://newframe.foxwq.com/cgi/QueryUserInfoPanel";
pub const FOX_SGF_CGI_URLS: [&str; 2] = [
    "http://happyapp.huanle.qq.com/cgi-bin/CommonMobileCGI/TXWQFetchChess",
    "http://cgi.foxwq.com/cgi-bin/CommonMobileCGI/TXWQFetchChess",
];
pub const FOX_HTTP_READ_TIMEOUT_MS: u64 = 25_000;
pub const FOX_MOBILE_USER_AGENT: &str =
    "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 \
     (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1";
pub const FOX_CGI_USER_AGENT: &str = "okhttp/3.12.12";
const FORM_URLENCODED: &str = "application/x-www-form-urlencoded";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct FoxNormalizedPayload {
    pub sgf_text: String,
    pub metadata: ProviderGameMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SgfTree {
    nodes: Vec<SgfNode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SgfNode {
    properties: Vec<SgfProperty>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SgfProperty {
    name: String,
    values: Vec<String>,
}

struct SgfParser<'a> {
    input: &'a str,
    index: usize,
}

pub fn query_user_request(user_name: &str) -> ProviderResult<ProviderFetchRequest> {
    let user_name = require_non_blank(user_name, "user_name")?;
    Ok(get_request(
        format!("{FOX_QUERY_USER_URL}?srcuid=0&username={}", url_encode(user_name)),
        None,
    ))
}

/// The list endpoint honours only the camelCase `lastCode` cursor; `lastcode` is silently ignored
/// upstream and returns the first batch again.
pub fn chess_list_request(uid: &str, last_code: &str) -> ProviderResult<ProviderFetchRequest> {
    let uid = require_non_blank(uid, "uid")?;
    let last_code = require_non_blank(last_code, "last_code")?;
    Ok(get_request(
        format!(
            "{FOX_BASE_URL}/YHWQFetchChessList?srcuid=0&dstuid={}&type=1&lastCode={}&searchkey=&uin={}",
            url_encode(uid),
            url_encode(last_code),
            url_encode(uid)
        ),
        Some(uid.to_string()),
    ))
}

pub fn cgi_sgf_requests(chessid: &str) -> ProviderResult<Vec<ProviderFetchRequest>> {
    let chessid = require_non_blank(chessid, "chessid")?;
    Ok(FOX_SGF_CGI_URLS
        .iter()
        .map(|endpoint| {
            let mut request = post_form_request(
                (*endpoint).to_string(),
                format!("chessid={}", url_encode(chessid)),
                Some(chessid.to_string()),
            );
            request
                .headers
                .insert("User-Agent".to_string(), FOX_CGI_USER_AGENT.to_string());
            request
        })
        .collect())
}

pub fn h5_sgf_request(chessid: &str) -> ProviderResult<ProviderFetchRequest> {
    let chessid = require_non_blank(chessid, "chessid")?;
    Ok(get_request(
        format!("{FOX_BASE_URL}/YHWQFetchChess?chessid={}", url_encode(chessid)),
        Some(chessid.to_string()),
    ))
}

pub fn import_payload(request: ProviderImportRequest) -> ProviderResult<ProviderImportResult> {
    let mut normalized = normalize_payload(&request.payload)?;
    normalized.metadata.source_url = normalized.metadata.source_url.or(request.source_url);
    normalized.metadata.source_id = normalized.metadata.source_id.or(request.source_id);
    merge_metadata(&mut normalized.metadata, request.metadata);

    let mut summary = metadata_summary(&normalized.metadata);
    summary.source_id = normalized.metadata.source_id.clone();
    Ok(ProviderImportResult {
        provider: ProviderKind::Fox,
        sgf_text: normalized.sgf_text,
        summary,
        metadata: normalized.metadata,
        warnings: Vec::new(),
    })
}

pub fn normalize_payload(payload: &str) -> ProviderResult<FoxNormalizedPayload> {
    let payload = require_non_blank(payload, "payload")?;
    if payload.trim_start().starts_with('(') {
        let sgf_text = normalize_sgf(payload);
        let metadata = metadata_from_sgf(&sgf_text);
        return Ok(FoxNormalizedPayload { sgf_text, metadata });
    }

    let json: Value = serde_json::from_str(payload)
        .map_err(|err| invalid_payload(format!("failed to parse Fox payload JSON: {err}")))?;
    let sgf = json
        .get("chess")
        .and_then(json_scalar_string)
        .ok_or_else(|| invalid_payload("Fox payload does not contain chess SGF text"))?;
    if sgf.trim().is_empty() {
        return Err(invalid_payload("Fox payload chess SGF text is empty"));
    }

    let sgf_text = normalize_sgf(&sgf);
    let mut metadata = metadata_from_sgf(&sgf_text);
    enrich_metadata_from_json(&json, &mut metadata);
    Ok(FoxNormalizedPayload { sgf_text, metadata })
}

pub fn normalize_sgf(sgf: &str) -> String {
    sgf::normalize_fox_sgf(sgf)
}

pub fn sanitize_sgf(sgf: &str) -> String {
    sgf::sanitize_fox_sgf(sgf)
}

/// Resolves a nickname lookup response into a non-secret account. Any rejection is `not_found`.
fn parse_user_info(payload: &str, query_text: &str) -> ProviderResult<FoxAccountDto> {
    let json: Value = serde_json::from_str(require_non_blank(payload, "payload")?)
        .map_err(|err| invalid_payload(format!("failed to parse Fox user JSON: {err}")))?;
    if !json.is_object() {
        return Err(invalid_payload("Fox user payload JSON must be an object"));
    }
    let result = if json.get("result").is_some() {
        json.get("result").and_then(json_i64).unwrap_or(-1)
    } else {
        json.get("errcode").and_then(json_i64).unwrap_or(-1)
    };
    let uid = json
        .get("uid")
        .and_then(json_scalar_string)
        .filter(|value| is_fox_uid(value))
        .unwrap_or_default();
    if result != 0 || uid.is_empty() {
        return Err(provider_error(
            app_model::ProviderErrorKind::NotFound,
            "No Fox account matches this nickname.",
        ));
    }
    let username = json
        .get("username")
        .and_then(json_scalar_string)
        .unwrap_or_default();
    let name = json.get("name").and_then(json_scalar_string).unwrap_or_default();
    let english_name = json
        .get("englishname")
        .and_then(json_scalar_string)
        .unwrap_or_default();
    let nickname = first_non_blank([
        username.as_str(),
        name.as_str(),
        english_name.as_str(),
        query_text,
    ])
    .unwrap_or(query_text)
    .to_string();

    Ok(FoxAccountDto { uid, nickname })
}

fn metadata_from_sgf(sgf: &str) -> ProviderGameMetadata {
    let mut metadata = ProviderGameMetadata::default();
    if let Some(root) = parse_root_properties(sgf) {
        set_extra_from_property(&root, &mut metadata, "PB", "black_name");
        set_extra_from_property(&root, &mut metadata, "PW", "white_name");
        set_extra_from_property(&root, &mut metadata, "RE", "result");
        set_extra_from_property(&root, &mut metadata, "SZ", "board_size");
        set_extra_from_property(&root, &mut metadata, "KM", "komi");
        set_extra_from_property(&root, &mut metadata, "HA", "handicap");
        set_extra_from_property(&root, &mut metadata, "DT", "date");
        metadata.title = first_non_blank([
            root.get("GN").map(String::as_str).unwrap_or_default(),
            root.get("EV").map(String::as_str).unwrap_or_default(),
        ])
        .map(ToString::to_string);
    }
    metadata
}

fn metadata_summary(metadata: &ProviderGameMetadata) -> ProviderGameSummary {
    let board_size = metadata
        .extra
        .get("board_size")
        .and_then(|value| value.parse().ok());
    ProviderGameSummary {
        provider: ProviderKind::Fox,
        source_id: metadata.source_id.clone(),
        board_width: board_size,
        board_height: board_size,
        komi: metadata.extra.get("komi").and_then(|value| value.parse().ok()),
        handicap: metadata
            .extra
            .get("handicap")
            .and_then(|value| value.parse().ok()),
        black_name: metadata.extra.get("black_name").cloned(),
        white_name: metadata.extra.get("white_name").cloned(),
        result: metadata.extra.get("result").cloned(),
        date: metadata.extra.get("date").cloned(),
        move_count: None,
    }
}

fn enrich_metadata_from_json(json: &Value, metadata: &mut ProviderGameMetadata) {
    for key in ["chessid", "chess_id", "id"] {
        if let Some(value) = json.get(key).and_then(json_scalar_string) {
            if !value.trim().is_empty() {
                metadata.source_id = Some(value);
                break;
            }
        }
    }
    if let Some(value) = json.get("result").and_then(json_scalar_string) {
        metadata.provider_status = Some(value);
    }
    if let Some(value) = json.get("resultstr").and_then(json_scalar_string) {
        metadata.extra.insert("provider_message".to_string(), value);
    }
}

fn set_extra_from_property(
    root: &BTreeMap<String, String>,
    metadata: &mut ProviderGameMetadata,
    property_name: &str,
    extra_name: &str,
) {
    if let Some(value) = root.get(property_name).filter(|value| !value.trim().is_empty()) {
        metadata.extra.insert(extra_name.to_string(), value.clone());
    }
}

fn parse_root_properties(sgf: &str) -> Option<BTreeMap<String, String>> {
    let mut parser = SgfParser::new(sgf);
    parser.skip_whitespace();
    let tree = parser.parse_tree().ok()?;
    let mut out = BTreeMap::new();
    for property in tree.nodes.first()?.properties.iter() {
        if let Some(value) = property.values.first() {
            out.insert(property.name.clone(), value.clone());
        }
    }
    Some(out)
}

fn json_scalar_string(value: &Value) -> Option<String> {
    match value {
        Value::String(value) => Some(value.trim().to_string()),
        Value::Number(value) => Some(value.to_string()),
        Value::Bool(value) => Some(value.to_string()),
        _ => None,
    }
}

fn json_i64(value: &Value) -> Option<i64> {
    match value {
        Value::Number(value) => value.as_i64(),
        Value::String(value) => value.trim().parse().ok(),
        _ => None,
    }
}

fn merge_metadata(target: &mut ProviderGameMetadata, source: ProviderGameMetadata) {
    target.source_url = target.source_url.take().or(source.source_url);
    target.request_url = target.request_url.take().or(source.request_url);
    target.source_id = target.source_id.take().or(source.source_id);
    target.room_id = target.room_id.take().or(source.room_id);
    target.title = target.title.take().or(source.title);
    target.provider_status = target.provider_status.take().or(source.provider_status);
    target.extra.extend(source.extra);
}

fn get_request(url: String, source_id: Option<String>) -> ProviderFetchRequest {
    ProviderFetchRequest {
        provider: ProviderKind::Fox,
        url,
        method: ProviderFetchMethod::Get,
        headers: default_headers(FOX_MOBILE_USER_AGENT),
        body: None,
        source_url: None,
        source_id,
        timeout_ms: Some(FOX_HTTP_READ_TIMEOUT_MS),
    }
}

fn post_form_request(url: String, body: String, source_id: Option<String>) -> ProviderFetchRequest {
    let mut headers = default_headers(FOX_MOBILE_USER_AGENT);
    headers.insert("Content-Type".to_string(), FORM_URLENCODED.to_string());
    ProviderFetchRequest {
        provider: ProviderKind::Fox,
        url,
        method: ProviderFetchMethod::Post,
        headers,
        body: Some(body),
        source_url: None,
        source_id,
        timeout_ms: Some(FOX_HTTP_READ_TIMEOUT_MS),
    }
}

fn default_headers(user_agent: &str) -> BTreeMap<String, String> {
    BTreeMap::from([
        (
            "Accept".to_string(),
            "application/json,text/plain,*/*".to_string(),
        ),
        ("Connection".to_string(), "close".to_string()),
        ("User-Agent".to_string(), user_agent.to_string()),
    ])
}

/// Fox UIDs are positive decimal integers. `0` and non-digits make the list endpoint return
/// unrelated public games instead of an error.
pub(crate) fn is_fox_uid(value: &str) -> bool {
    is_decimal_id(value) && value.bytes().any(|byte| byte != b'0')
}

pub(crate) fn is_decimal_id(value: &str) -> bool {
    !value.is_empty() && value.len() <= 32 && value.bytes().all(|byte| byte.is_ascii_digit())
}

fn url_encode(value: &str) -> String {
    let mut out = String::new();
    for byte in value.as_bytes() {
        match *byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'.' | b'-' | b'*' | b'_' => {
                out.push(*byte as char);
            }
            b' ' => out.push('+'),
            byte => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

impl<'a> SgfParser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, index: 0 }
    }

    fn parse_tree(&mut self) -> Result<SgfTree, ()> {
        self.expect('(')?;
        self.skip_whitespace();
        let mut nodes = Vec::new();
        while self.peek() == Some(';') {
            nodes.push(self.parse_node()?);
            self.skip_whitespace();
        }
        while self.peek() == Some('(') {
            self.parse_tree()?;
            self.skip_whitespace();
        }
        self.expect(')')?;
        Ok(SgfTree { nodes })
    }

    fn parse_node(&mut self) -> Result<SgfNode, ()> {
        self.expect(';')?;
        let mut properties = Vec::new();
        loop {
            self.skip_whitespace();
            match self.peek() {
                None | Some(';') | Some('(') | Some(')') => break,
                _ => properties.push(self.parse_property()?),
            }
        }
        Ok(SgfNode { properties })
    }

    fn parse_property(&mut self) -> Result<SgfProperty, ()> {
        let name_start = self.index;
        while let Some(current) = self.peek() {
            if current.is_ascii_alphabetic() {
                self.index += current.len_utf8();
            } else {
                break;
            }
        }
        if name_start == self.index {
            return Err(());
        }
        let name = self.input[name_start..self.index].to_string();
        self.skip_whitespace();
        let mut values = Vec::new();
        while self.peek() == Some('[') {
            values.push(self.parse_value()?);
            self.skip_whitespace();
        }
        if values.is_empty() {
            return Err(());
        }
        Ok(SgfProperty { name, values })
    }

    fn parse_value(&mut self) -> Result<String, ()> {
        self.expect('[')?;
        let mut value = String::new();
        while let Some(current) = self.peek() {
            self.index += current.len_utf8();
            if current == '\\' {
                if let Some(next) = self.peek() {
                    self.index += next.len_utf8();
                    value.push(current);
                    value.push(next);
                }
            } else if current == ']' {
                return Ok(value);
            } else {
                value.push(current);
            }
        }
        Err(())
    }

    fn skip_whitespace(&mut self) {
        while let Some(current) = self.peek() {
            if current.is_whitespace() {
                self.index += current.len_utf8();
            } else {
                break;
            }
        }
    }

    fn expect(&mut self, expected: char) -> Result<(), ()> {
        if self.peek() == Some(expected) {
            self.index += expected.len_utf8();
            Ok(())
        } else {
            Err(())
        }
    }

    fn peek(&self) -> Option<char> {
        self.input[self.index..].chars().next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use app_model::{
        FoxKifuStateDto, FoxLookupDto, FoxLookupKindDto, ProviderErrorKind, ProviderFetchResult,
    };
    use provider_core::{transport_failed, ProviderTransport};
    use std::collections::VecDeque;
    use std::sync::Mutex;

    #[test]
    fn normalize_payload_promotes_leading_setup_nodes_into_root() {
        let payload = serde_json::json!({
            "chess": "(;GM[1]FF[4]SZ[19]KM[0]HA[2]PB[Black]PW[White];AB[pd][dp];W[pp];B[dd])"
        })
        .to_string();

        let normalized = normalize_payload(&payload).unwrap();

        assert!(
            normalized.sgf_text.starts_with("(;GM[1]FF[4]"),
            "{}",
            normalized.sgf_text
        );
        assert!(normalized.sgf_text.contains("HA[2]"), "{}", normalized.sgf_text);
        assert!(
            normalized.sgf_text.contains("AB[pd][dp]"),
            "{}",
            normalized.sgf_text
        );
        assert!(
            normalized.sgf_text.contains(";W[pp];B[dd])"),
            "{}",
            normalized.sgf_text
        );
        assert!(
            !normalized.sgf_text.contains(";AB[pd][dp];"),
            "{}",
            normalized.sgf_text
        );
    }

    #[test]
    fn sanitize_removes_backslashes_outside_values_only() {
        let sgf = "\\(;GM[1]C[a\\]b];B[aa]\\)";

        assert_eq!(sanitize_sgf(sgf), "(;GM[1]C[a\\]b];B[aa])");
    }

    #[test]
    fn import_payload_extracts_metadata() {
        let request = ProviderImportRequest {
            provider: ProviderKind::Fox,
            payload: serde_json::json!({
                "result": 0,
                "chessid": "abc123",
                "chess": "(;GM[1]FF[4]SZ[19]KM[6.5]PB[Black]PW[White]RE[B+R];B[dd])"
            })
            .to_string(),
            source_url: Some("https://fox.example/game/abc123".to_string()),
            source_id: None,
            metadata: ProviderGameMetadata::default(),
        };

        let result = import_payload(request).unwrap();

        assert_eq!(result.provider, ProviderKind::Fox);
        assert_eq!(result.summary.source_id.as_deref(), Some("abc123"));
        assert_eq!(result.summary.black_name.as_deref(), Some("Black"));
        assert_eq!(result.summary.white_name.as_deref(), Some("White"));
        assert_eq!(result.summary.result.as_deref(), Some("B+R"));
        assert_eq!(
            result.metadata.source_url.as_deref(),
            Some("https://fox.example/game/abc123")
        );
    }

    #[test]
    fn normalize_payload_recovers_windowed_fox_chunks_from_json() {
        let payload = serde_json::json!({
            "chessid": "windowed-json",
            "result": 0,
            "resultstr": "ok",
            "chess": windowed_fox_sgf()
        })
        .to_string();

        let normalized = normalize_payload(&payload).unwrap();
        let document = sgf::parse_sgf(&normalized.sgf_text).unwrap();

        assert_eq!(normalized.metadata.source_id.as_deref(), Some("windowed-json"));
        assert_eq!(
            normalized.metadata.extra.get("black_name").map(String::as_str),
            Some("Black")
        );
        assert_eq!(
            normalized.metadata.extra.get("white_name").map(String::as_str),
            Some("White")
        );
        assert_eq!(normalized.metadata.provider_status.as_deref(), Some("0"));
        assert_eq!(
            normalized
                .metadata
                .extra
                .get("provider_message")
                .map(String::as_str),
            Some("ok")
        );
        assert_eq!(normalized.sgf_text.matches('(').count(), 1);
        assert_eq!(document.moves.len(), 84, "{}", normalized.sgf_text);
    }

    #[test]
    fn import_payload_recovers_windowed_fox_chunks_from_raw_sgf() {
        let request = ProviderImportRequest {
            provider: ProviderKind::Fox,
            payload: windowed_fox_sgf(),
            source_url: Some("https://fox.example/game/windowed-raw".to_string()),
            source_id: Some("windowed-raw".to_string()),
            metadata: ProviderGameMetadata::default(),
        };

        let result = import_payload(request).unwrap();
        let document = sgf::parse_sgf(&result.sgf_text).unwrap();

        assert_eq!(result.provider, ProviderKind::Fox);
        assert_eq!(result.summary.source_id.as_deref(), Some("windowed-raw"));
        assert_eq!(result.summary.black_name.as_deref(), Some("Black"));
        assert_eq!(result.summary.white_name.as_deref(), Some("White"));
        assert_eq!(
            result.metadata.source_url.as_deref(),
            Some("https://fox.example/game/windowed-raw")
        );
        assert_eq!(result.sgf_text.matches('(').count(), 1);
        assert_eq!(document.moves.len(), 84, "{}", result.sgf_text);
    }

    #[test]
    fn raw_sgf_payload_imports_without_json_wrapper() {
        let normalized = normalize_payload("(;GM[1]SZ[19];B[aa](;W[bb])(;W[cc]))").unwrap();

        assert_eq!(normalized.sgf_text, "(;GM[1]FF[4]CA[UTF-8]SZ[19];B[aa];W[bb])");
    }

    #[test]
    fn list_request_uses_the_cursor_parameter_the_endpoint_honours() {
        let request = chess_list_request("12345", "678").unwrap();
        assert_eq!(
            request.url,
            format!("{FOX_BASE_URL}/YHWQFetchChessList?srcuid=0&dstuid=12345&type=1&lastCode=678&searchkey=&uin=12345")
        );
        let cgi = cgi_sgf_requests("9").unwrap();
        assert_eq!(cgi[0].method, ProviderFetchMethod::Post);
        assert_eq!(cgi[0].body.as_deref(), Some("chessid=9"));
        assert_eq!(
            cgi[0].headers.get("User-Agent").map(String::as_str),
            Some(FOX_CGI_USER_AGENT)
        );
    }

    #[test]
    fn lookups_are_validated_before_io() {
        let transport = SequenceTransport::new(vec![]);
        for (kind, value) in [
            (FoxLookupKindDto::Uid, "0"),
            (FoxLookupKindDto::Uid, "12a"),
            (FoxLookupKindDto::Nickname, "  "),
            (FoxLookupKindDto::Nickname, "a\nb"),
            (FoxLookupKindDto::Chessid, "123"),
        ] {
            let error = fetch_account_list(&transport, &lookup(kind, value)).unwrap_err();
            assert_eq!(error.kind, ProviderErrorKind::InvalidRequest, "{kind:?} {value}");
        }
        assert_eq!(
            fetch_game_preview(&transport, "abc").unwrap_err().kind,
            ProviderErrorKind::InvalidRequest
        );
        assert_eq!(
            fetch_list_continuation(&transport, &account("0", ""), "5")
                .unwrap_err()
                .kind,
            ProviderErrorKind::InvalidRequest
        );
        assert!(transport.requests().is_empty());
    }

    #[test]
    fn nickname_lookup_resolves_account_then_lists_games_without_sgf() {
        let transport = SequenceTransport::new(vec![
            Ok(fetch_response(
                FOX_QUERY_USER_URL,
                200,
                r#"{"result":0,"uid":"2468","username":"Good Player"}"#,
            )),
            Ok(fetch_response(
                FOX_BASE_URL,
                200,
                &list_json(&[("901", "2468", "1"), ("900", "1", "2468")]),
            )),
        ]);
        let page =
            fetch_account_list(&transport, &lookup(FoxLookupKindDto::Nickname, " Good Player ")).unwrap();
        let requests = transport.requests();
        assert_eq!(
            requests[0].url,
            format!("{FOX_QUERY_USER_URL}?srcuid=0&username=Good+Player")
        );
        assert!(requests[1].url.contains("dstuid=2468&type=1&lastCode=0"));
        assert_eq!(page.account, account("2468", "Good Player"));
        assert_eq!(
            page.games.iter().map(|g| g.chessid.as_str()).collect::<Vec<_>>(),
            ["901", "900"]
        );
        assert_eq!(page.games[0].account_won, Some(true));
        assert_eq!(page.games[0].result, "黑中盘胜");
        assert_eq!(page.games[0].black_rank, "职业9段");
        assert_eq!(page.next_cursor.as_deref(), Some("900"));
        assert!(!page.has_more, "a short batch ends the list");
    }

    #[test]
    fn unknown_nickname_is_not_found_and_does_not_list() {
        let transport = SequenceTransport::new(vec![Ok(fetch_response(
            FOX_QUERY_USER_URL,
            200,
            r#"{"result":104013,"uid":"0","username":""}"#,
        ))]);
        let error = fetch_account_list(&transport, &lookup(FoxLookupKindDto::Nickname, "ghost")).unwrap_err();
        assert_eq!(error.kind, ProviderErrorKind::NotFound);
        assert_eq!(transport.requests().len(), 1);
    }

    #[test]
    fn first_batch_is_capped_and_continuation_skips_cursor_and_duplicates() {
        let first: Vec<_> = (0..101)
            .map(|i| (format!("{}", 5000 - i), "7".to_string()))
            .collect();
        let first_rows: Vec<_> = first
            .iter()
            .map(|(id, uid)| (id.as_str(), uid.as_str(), "1"))
            .collect();
        let transport = SequenceTransport::new(vec![Ok(fetch_response(
            FOX_BASE_URL,
            200,
            &list_json(&first_rows),
        ))]);
        let page = fetch_account_list(&transport, &lookup(FoxLookupKindDto::Uid, "7")).unwrap();
        assert_eq!(page.games.len(), FOX_LIST_BATCH_LIMIT);
        assert_eq!(page.next_cursor.as_deref(), Some("4901"));
        assert!(page.has_more);
        assert_eq!(
            page.account.nickname, "p7",
            "UID lookups learn the nickname from the account's games"
        );

        let transport = SequenceTransport::new(vec![Ok(fetch_response(
            FOX_BASE_URL,
            200,
            &list_json(&[
                ("4901", "7", "1"),
                ("4900", "7", "1"),
                ("4900", "7", "1"),
                ("4899", "7", "1"),
            ]),
        ))]);
        let next = fetch_list_continuation(&transport, &page.account, "4901").unwrap();
        assert!(transport.requests()[0].url.contains("lastCode=4901"));
        assert_eq!(
            next.games.iter().map(|g| g.chessid.as_str()).collect::<Vec<_>>(),
            ["4900", "4899"]
        );
        assert!(!next.has_more);
    }

    #[test]
    fn empty_batch_ends_and_rejected_or_malformed_lists_are_typed() {
        let transport = SequenceTransport::new(vec![Ok(fetch_response(
            FOX_BASE_URL,
            200,
            r#"{"result":0,"chesslist":[]}"#,
        ))]);
        let page = fetch_list_continuation(&transport, &account("7", "p7"), "11").unwrap();
        assert!(page.games.is_empty() && !page.has_more && page.next_cursor.is_none());
        for body in [
            r#"{"result":5,"chesslist":[]}"#,
            r#"{"result":0}"#,
            "<html>",
            r#"{"result":0,"chesslist":[{"chessid":""}]}"#,
        ] {
            let transport = SequenceTransport::new(vec![Ok(fetch_response(FOX_BASE_URL, 200, body))]);
            let error = fetch_account_list(&transport, &lookup(FoxLookupKindDto::Uid, "7")).unwrap_err();
            assert_eq!(error.kind, ProviderErrorKind::InvalidPayload, "{body}");
        }
        let transport = SequenceTransport::new(vec![Ok(fetch_response(FOX_BASE_URL, 500, "down"))]);
        assert_eq!(
            fetch_account_list(&transport, &lookup(FoxLookupKindDto::Uid, "7"))
                .unwrap_err()
                .kind,
            ProviderErrorKind::TransportFailed
        );
    }

    #[test]
    fn chessid_preview_uses_first_cgi_record_and_normalizes_sgf() {
        let transport = SequenceTransport::new(vec![Ok(fetch_response(
            FOX_SGF_CGI_URLS[0],
            200,
            r#"{"result":0,"chess":"(;SZ[19]PB[Black]PW[White];B[aa](;W[bb])(;W[cc]))"}"#,
        ))]);
        let preview = fetch_game_preview(&transport, "42").unwrap();
        assert_eq!(transport.requests().len(), 1);
        assert_eq!(
            preview.sgf_text,
            "(;GM[1]FF[4]CA[UTF-8]SZ[19]PB[Black]PW[White];B[aa];W[bb])"
        );
        assert_eq!(preview.summary.source_id.as_deref(), Some("42"));
    }

    #[test]
    fn cgi_misses_and_transient_failures_fall_through_to_h5() {
        let h5 = format!("{FOX_BASE_URL}/YHWQFetchChess");
        let transport = SequenceTransport::new(vec![
            Ok(fetch_response(
                FOX_SGF_CGI_URLS[0],
                200,
                r#"{"result":-3,"resultstr":"FetchChessFromDB Failed!!"}"#,
            )),
            Ok(fetch_response(
                FOX_SGF_CGI_URLS[1],
                200,
                "<!DOCTYPE html><title>提示</title>",
            )),
            Ok(fetch_response(
                &h5,
                200,
                r#"{"result":0,"chessid":"42","chess":"(;SZ[13];B[aa])"}"#,
            )),
        ]);
        assert_eq!(
            fetch_game_preview(&transport, "42").unwrap().sgf_text,
            "(;GM[1]FF[4]CA[UTF-8]SZ[13];B[aa])"
        );
        assert_eq!(transport.requests().len(), 3);

        let transport = SequenceTransport::new(vec![
            Err(transport_failed("reset")),
            Ok(fetch_response(FOX_SGF_CGI_URLS[1], 503, "busy")),
            Ok(fetch_response(
                &h5,
                200,
                r#"{"result":0,"chess":"(;SZ[19];B[aa])"}"#,
            )),
        ]);
        assert!(fetch_game_preview(&transport, "42").is_ok());
        assert_eq!(transport.requests().len(), 3);
    }

    #[test]
    fn h5_miss_is_not_found_and_malformed_h5_is_terminal() {
        let h5 = format!("{FOX_BASE_URL}/YHWQFetchChess");
        for (body, kind) in [
            (
                r#"{"result":101200,"chessid":"1","chess":""}"#,
                ProviderErrorKind::NotFound,
            ),
            ("{", ProviderErrorKind::InvalidPayload),
        ] {
            let transport = SequenceTransport::new(vec![
                Ok(fetch_response(FOX_SGF_CGI_URLS[0], 200, r#"{"result":-3}"#)),
                Ok(fetch_response(FOX_SGF_CGI_URLS[1], 200, "<html>")),
                Ok(fetch_response(&h5, 200, body)),
            ]);
            assert_eq!(
                fetch_game_preview(&transport, "1").unwrap_err().kind,
                kind,
                "{body}"
            );
            assert_eq!(transport.requests().len(), 3);
        }
        let transport = SequenceTransport::new(vec![Ok(fetch_response(FOX_SGF_CGI_URLS[0], 404, "gone"))]);
        assert_eq!(
            fetch_game_preview(&transport, "1").unwrap_err().kind,
            ProviderErrorKind::NotFound
        );
        assert_eq!(
            transport.requests().len(),
            1,
            "classified failures do not switch endpoint"
        );
    }

    #[test]
    fn escaped_line_breaks_between_properties_do_not_corrupt_the_record() {
        // Live H5 record 1785337045010001403 carries literal `\r\n` text between root properties.
        let chess = "(;GM[1]FF[4]\\r\\nSZ[19]\\r\\nPB[A]\\r\\n;B[pd];W[dd])";
        let transport = SequenceTransport::new(vec![Ok(fetch_response(
            FOX_SGF_CGI_URLS[0],
            200,
            &serde_json::json!({ "result": 0, "chess": chess }).to_string(),
        ))]);
        let preview = fetch_game_preview(&transport, "1").unwrap();
        let document = sgf::parse_sgf(&preview.sgf_text).unwrap();
        assert_eq!(
            (document.board_width, document.moves.len()),
            (19, 2),
            "{}",
            preview.sgf_text
        );
    }

    #[test]
    fn malformed_cgi_record_is_terminal_without_reading_another_endpoint() {
        for body in ["{", "[1]", "null"] {
            let transport = SequenceTransport::new(vec![
                Ok(fetch_response(FOX_SGF_CGI_URLS[0], 200, body)),
                Ok(fetch_response(
                    FOX_SGF_CGI_URLS[1],
                    200,
                    r#"{"result":0,"chess":"(;SZ[19];B[aa])"}"#,
                )),
            ]);
            assert_eq!(
                fetch_game_preview(&transport, "1").unwrap_err().kind,
                ProviderErrorKind::InvalidPayload,
                "{body}"
            );
            assert_eq!(transport.requests().len(), 1, "{body}");
        }
    }

    #[test]
    fn professional_rank_codes_start_at_one_hundred() {
        let row = |dan: i64| {
            serde_json::json!({ "result": 0, "chesslist": [{
            "chessid": "1", "blackuid": 7, "whiteuid": 8, "blackdan": dan, "whitedan": 17, "winner": 0,
        }] })
            .to_string()
        };
        for (dan, black, white) in [
            (100, "职业1段", "1级"),
            (108, "职业9段", "1级"),
            (27, "10段", "1级"),
            (16, "2级", "1级"),
        ] {
            let transport = SequenceTransport::new(vec![Ok(fetch_response(FOX_BASE_URL, 200, &row(dan)))]);
            let game = fetch_account_list(&transport, &lookup(FoxLookupKindDto::Uid, "7"))
                .unwrap()
                .games
                .remove(0);
            assert_eq!(
                (game.black_rank.as_str(), game.white_rank.as_str()),
                (black, white),
                "{dan}"
            );
        }
    }

    #[test]
    fn recents_keep_eight_unique_accounts_and_the_last_query() {
        let mut state = FoxKifuStateDto::default();
        for uid in 1..=9 {
            let uid = uid.to_string();
            state = remember_lookup(
                &state,
                &lookup(FoxLookupKindDto::Uid, &uid),
                Some(&account(&uid, &format!("n{uid}"))),
            )
            .unwrap();
        }
        assert_eq!(state.recents.len(), FOX_RECENTS_LIMIT);
        assert_eq!(state.recents[0].uid, "9");
        assert_eq!(state.recents[7].uid, "2");
        state = remember_lookup(
            &state,
            &lookup(FoxLookupKindDto::Nickname, "n5"),
            Some(&account("5", "n5")),
        )
        .unwrap();
        assert_eq!(state.recents.iter().filter(|a| a.uid == "5").count(), 1);
        assert_eq!(state.recents[0].uid, "5");
        state = remember_lookup(&state, &lookup(FoxLookupKindDto::Chessid, " 77 "), None).unwrap();
        assert_eq!(
            state.recents[0].uid, "5",
            "a chessid lookup changes only the last query"
        );
        assert_eq!(state.last_query, Some(lookup(FoxLookupKindDto::Chessid, "77")));
        assert!(remember_lookup(&state, &lookup(FoxLookupKindDto::Uid, "0"), None).is_err());

        let polluted = FoxKifuStateDto {
            recents: vec![
                account("0", "bad"),
                account("3", "x"),
                account("3", "dup"),
                account("4", ""),
            ],
            last_query: Some(lookup(FoxLookupKindDto::Uid, "x")),
        };
        let clean = sanitize_state(&polluted);
        assert_eq!(clean.recents, vec![account("3", "x"), account("4", "")]);
        assert_eq!(clean.last_query, None);
    }

    fn lookup(kind: FoxLookupKindDto, value: &str) -> FoxLookupDto {
        FoxLookupDto {
            kind,
            value: value.to_string(),
        }
    }

    fn account(uid: &str, nickname: &str) -> FoxAccountDto {
        FoxAccountDto {
            uid: uid.to_string(),
            nickname: nickname.to_string(),
        }
    }

    /// Rows of (chessid, black uid, white uid); winner is black, players are named `p<uid>`.
    fn list_json(rows: &[(&str, &str, &str)]) -> String {
        let rows: Vec<Value> = rows.iter().map(|(id, black, white)| serde_json::json!({
            "chessid": id, "blackuid": black.parse::<i64>().unwrap(), "whiteuid": white.parse::<i64>().unwrap(),
            "blacknick": format!("p{black}"), "whitenick": format!("p{white}"), "blackdan": 108, "whitedan": 25,
            "winner": 1, "point": -1, "rule": 1, "movenum": 120, "boardsize": 19, "starttime": "2026-10-05 12:00:00",
        })).collect();
        serde_json::json!({ "result": 0, "chesslist": rows }).to_string()
    }

    fn windowed_fox_sgf() -> String {
        let mut input = String::from("(;SZ[19]PB[Black]PW[White]");
        for start in (0..80).step_by(4).take(20) {
            input.push('(');
            for index in start..start + 8 {
                let color = if index % 2 == 0 { "B" } else { "W" };
                input.push(';');
                input.push_str(color);
                input.push('[');
                input.push_str(&test_sgf_coord(index));
                input.push(']');
            }
            input.push(')');
        }
        input.push(')');
        input
    }

    fn test_sgf_coord(index: usize) -> String {
        let x = (index % 19) as u8;
        let y = (index / 19) as u8;
        format!("{}{}", (b'a' + x) as char, (b'a' + y) as char)
    }

    fn fetch_response(url: &str, status_code: u16, payload: &str) -> ProviderFetchResult {
        ProviderFetchResult {
            provider: ProviderKind::Fox,
            url: url.to_string(),
            status_code,
            payload: payload.to_string(),
            headers: BTreeMap::new(),
            content_type: Some("application/json".to_string()),
            metadata: ProviderGameMetadata::default(),
            warnings: Vec::new(),
        }
    }

    struct SequenceTransport {
        requests: Mutex<Vec<ProviderFetchRequest>>,
        responses: Mutex<VecDeque<ProviderResult<ProviderFetchResult>>>,
    }

    impl SequenceTransport {
        fn new(responses: Vec<ProviderResult<ProviderFetchResult>>) -> Self {
            Self {
                requests: Mutex::default(),
                responses: Mutex::new(VecDeque::from(responses)),
            }
        }

        fn requests(&self) -> Vec<ProviderFetchRequest> {
            self.requests.lock().unwrap().clone()
        }
    }

    impl ProviderTransport for SequenceTransport {
        fn fetch(&self, request: &ProviderFetchRequest) -> ProviderResult<ProviderFetchResult> {
            self.requests.lock().unwrap().push(request.clone());
            self.responses
                .lock()
                .unwrap()
                .pop_front()
                .unwrap_or_else(|| Err(transport_failed("missing test response")))
        }
    }
}
