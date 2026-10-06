use app_model::{
    ProviderErrorKind, ProviderFetchMethod, ProviderFetchRequest, ProviderFetchResult, ProviderGameMetadata,
    ProviderImportRequest, ProviderKind, YikeCategoryDto,
};
use provider_core::{ProviderTransport, RecordingProviderTransport};
use provider_yike::{
    canonical_yike_locator, fetch_public_list, fetch_public_preview, import_payload, parse_yike_url,
    YikeRoomKind,
};
use serde_json::{json, Value};
use std::collections::BTreeMap;

const SOURCE_TREE: &str = "(;GM[1]FF[4]SZ[19]PB[Black]PW[White]GN[Source]C[root metadata];B[aa](;W[bb]C[first];B[cc])(;W[dd]C[second];B[ee]))";

fn fetch_result(status_code: u16, payload: String) -> ProviderFetchResult {
    ProviderFetchResult {
        provider: ProviderKind::Yike,
        url: "https://api.yikeweiqi.com/fixture".to_string(),
        status_code,
        payload,
        headers: BTreeMap::from([("CheckSum".to_string(), "do-not-expose".to_string())]),
        content_type: Some("application/json".to_string()),
        metadata: ProviderGameMetadata::default(),
        warnings: Vec::new(),
    }
}

fn transport(status_code: u16, payload: Value) -> RecordingProviderTransport {
    RecordingProviderTransport::with_result(Ok(fetch_result(status_code, payload.to_string())))
}

fn new_detail() -> Value {
    json!({"status":0,"result":{"sgf":SOURCE_TREE,"status":2,"game_result":"B+R"}})
}

fn old_detail() -> Value {
    json!({"Status":1200,"Result":{"live":{"Content":SOURCE_TREE,"Status":3,"GameResult":"W+2.5"},"cmt":[],"branch":[]}})
}

fn assert_signed(request: &ProviderFetchRequest) {
    assert_eq!(request.provider, ProviderKind::Yike);
    assert_eq!(request.method, ProviderFetchMethod::Get);
    assert_eq!(request.timeout_ms, Some(10_000));
    assert!(request.body.is_none());
    for key in ["AppKey", "CurTime", "Nonce", "CheckSum", "accesstoken"] {
        assert!(!request.headers.get(key).unwrap().is_empty());
    }
    assert_eq!(request.headers.get("usertoken").map(String::as_str), Some("-1"));
}

#[test]
fn signed_live_and_legacy_families_preserve_source_tree() {
    let cases = [
        (
            "http://home.yikeweiqi.com/#/live/new-room/0196106/0/0",
            "https://home.yikeweiqi.com/#/live/new-room/196106",
            YikeRoomKind::NewLiveRoom,
            "https://api-new.yikeweiqi.com/v1/golives/196106",
            "196106",
            "196106",
            new_detail(),
            "2",
            "B+R",
        ),
        (
            "https://home.yikeweiqi.com/#/live/room/18328/1/15630642",
            "https://home.yikeweiqi.com/#/live/room/18328/1/15630642",
            YikeRoomKind::OldLiveRoom,
            "https://api.yikeweiqi.com/golive/dtl?id=18328&flag=1",
            "18328",
            "15630642",
            old_detail(),
            "3",
            "W+2.5",
        ),
        (
            "https://home.yikeweiqi.com/#/live/room/4903/0/0",
            "https://home.yikeweiqi.com/#/live/room/4903",
            YikeRoomKind::OldLiveBoard,
            "https://api.yikeweiqi.com/golive/dtl?id=4903",
            "4903",
            "4903",
            old_detail(),
            "3",
            "W+2.5",
        ),
        (
            "https://home.yikeweiqi.com/#/game/play/1/15630642",
            "https://home.yikeweiqi.com/#/game/play/1/15630642",
            YikeRoomKind::GameRoom,
            "https://api.yikeweiqi.com/golive/dtl?id=15630642",
            "15630642",
            "15630642",
            old_detail(),
            "3",
            "W+2.5",
        ),
    ];
    for (locator, canonical, kind, endpoint, id, room, response, status, result) in cases {
        let descriptor = parse_yike_url(locator).unwrap();
        assert_eq!(descriptor.room_kind, kind, "{locator}");
        assert_eq!(descriptor.id, id);
        assert_eq!(descriptor.request_url, endpoint);
        assert_eq!(canonical_yike_locator(locator).unwrap(), canonical);
        assert_eq!(canonical_yike_locator(canonical).unwrap(), canonical);
        let transport = transport(200, response.clone());
        let preview = fetch_public_preview(&transport as &dyn ProviderTransport, locator).unwrap();
        assert_eq!(preview.sgf_text, SOURCE_TREE, "{locator}");
        assert_eq!(preview.metadata.source_url.as_deref(), Some(canonical));
        assert_eq!(preview.metadata.request_url.as_deref(), Some(endpoint));
        assert_eq!(preview.metadata.source_id.as_deref(), Some(id));
        assert_eq!(preview.metadata.room_id.as_deref(), Some(room));
        assert_eq!(preview.metadata.provider_status.as_deref(), Some(status));
        assert_eq!(preview.summary.result.as_deref(), Some(result));
        let imported = import_payload(ProviderImportRequest {
            provider: ProviderKind::Yike,
            payload: response.to_string(),
            source_url: Some(canonical.to_string()),
            source_id: Some(id.to_string()),
            metadata: ProviderGameMetadata::default(),
        })
        .unwrap();
        assert_eq!(imported.sgf_text, SOURCE_TREE);
        assert_eq!(imported.summary.result.as_deref(), Some(result));
    }
}

#[test]
fn hall_query_retains_required_room_and_hall_locators() {
    for locator in [
        "https://home.yikeweiqi.com/#/?hall=true&room=015630642",
        "https://home.yikeweiqi.com/?room=15630642&hall=true",
    ] {
        let canonical = "https://home.yikeweiqi.com/#/?room=15630642&hall=true";
        assert_eq!(canonical_yike_locator(locator).unwrap(), canonical);
        let transport = transport(200, old_detail());
        let result = fetch_public_preview(&transport, locator).unwrap();
        assert_eq!(result.metadata.source_url.as_deref(), Some(canonical));
        assert_eq!(result.sgf_text, SOURCE_TREE);
        assert_eq!(
            transport.requests().unwrap()[0].url,
            "https://api.yikeweiqi.com/golive/dtl?id=15630642"
        );
    }
}

#[test]
fn invalid_locator_and_secrets_are_rejected_before_transport() {
    let transport = transport(200, new_detail());
    for locator in [
        "",
        "https://example.com/#/live/new-room/1",
        "https://home.yikeweiqi.com.evil.test/#/live/new-room/1",
        "https://attacker.yikeweiqi.com/#/live/new-room/1",
        "https://user:secret@home.yikeweiqi.com/#/live/new-room/1",
        "https://@home.yikeweiqi.com/#/live/new-room/1",
        "https://home.yikeweiqi.com:8080/#/live/new-room/1",
        "ftp://home.yikeweiqi.com/#/live/new-room/1",
        "https://home.yikeweiqi.com/#/live/new-room/0",
        "https://home.yikeweiqi.com/#/live/new-room/-1",
        "https://home.yikeweiqi.com/#/live/new-room/abc",
        "https://home.yikeweiqi.com/#/live/new-room/9223372036854775808",
        "https://home.yikeweiqi.com/#/live/new-room/1/0/2/extra",
        "https://home.yikeweiqi.com/#/live/new-room/1/1/0",
        "https://home.yikeweiqi.com/#/live/new-room/1?token=secret",
        "https://home.yikeweiqi.com/?token=secret#/live/new-room/1",
        "https://home.yikeweiqi.com/#/?room=1&hall=true&access_token=secret",
        "https://home.yikeweiqi.com/#/?room=1&room=2&hall=true",
        "https://home.yikeweiqi.com/#/?room=1",
        "https://home.yikeweiqi.com/#/game/play/1/0",
        "https://unite.yikeweiqi.com/#/unite/0",
        "https://home.yikeweiqi.com/#/live/new-room/%31",
        "https://home.yikeweiqi.com/ignored/../live/new-room/1",
    ] {
        let error = fetch_public_preview(&transport, locator).unwrap_err();
        assert_eq!(error.kind, ProviderErrorKind::InvalidUrl, "{locator}");
        assert!(!error.message.contains("secret"));
    }
    assert!(transport.requests().unwrap().is_empty());
}

#[test]
fn envelope_errors_never_import_incidental_sgf_or_expose_response_secrets() {
    let cases = [
        (
            "https://home.yikeweiqi.com/#/live/new-room/1",
            json!({"status":403,"message":"raw-secret","result":{"sgf":SOURCE_TREE}}),
            ProviderErrorKind::AuthenticationFailed,
        ),
        (
            "https://home.yikeweiqi.com/#/live/new-room/1",
            json!({"status":404,"result":{"sgf":SOURCE_TREE}}),
            ProviderErrorKind::NotFound,
        ),
        (
            "https://home.yikeweiqi.com/#/live/new-room/1",
            json!({"status":0,"result":null,"incidental":{"sgf":SOURCE_TREE}}),
            ProviderErrorKind::InvalidPayload,
        ),
        (
            "https://home.yikeweiqi.com/#/live/room/1",
            json!({"Status":1403,"Message":"raw-secret","Result":{"live":{"Content":SOURCE_TREE}}}),
            ProviderErrorKind::AuthenticationFailed,
        ),
        (
            "https://home.yikeweiqi.com/#/live/room/1",
            json!({"Status":1404,"Result":{"live":{"Content":SOURCE_TREE}}}),
            ProviderErrorKind::NotFound,
        ),
        (
            "https://home.yikeweiqi.com/#/game/play/0/1",
            json!({"Status":1200,"Result":{"live":null,"sgf":SOURCE_TREE}}),
            ProviderErrorKind::InvalidPayload,
        ),
        (
            "https://unite.yikeweiqi.com/#/unite/1",
            json!({"code":1,"msg":"raw-secret","data":{"sgf":SOURCE_TREE}}),
            ProviderErrorKind::AuthenticationFailed,
        ),
        (
            "https://unite.yikeweiqi.com/#/unite/1",
            json!({"code":404,"data":{"sgf":SOURCE_TREE}}),
            ProviderErrorKind::NotFound,
        ),
        (
            "https://unite.yikeweiqi.com/#/unite/1",
            json!({"code":0,"status":403,"data":{"sgf":SOURCE_TREE}}),
            ProviderErrorKind::AuthenticationFailed,
        ),
        (
            "https://unite.yikeweiqi.com/#/unite/1",
            json!({"code":0,"data":null,"sgf":SOURCE_TREE}),
            ProviderErrorKind::InvalidPayload,
        ),
    ];
    for (locator, response, expected) in cases {
        let error = if locator.contains("/unite/") {
            fetch_public_preview(
                &AnonymousGameService {
                    detail: response.clone(),
                },
                locator,
            )
            .unwrap_err()
        } else {
            fetch_public_preview(&transport(200, response.clone()), locator).unwrap_err()
        };
        assert_eq!(error.kind, expected, "{locator}");
        assert!(!error.message.contains("raw-secret"));
        assert!(!error.message.contains(SOURCE_TREE));
        assert!(!error.message.contains("do-not-expose"));
        let imported_error = import_payload(ProviderImportRequest {
            provider: ProviderKind::Yike,
            payload: response.to_string(),
            source_url: None,
            source_id: None,
            metadata: ProviderGameMetadata::default(),
        })
        .unwrap_err();
        assert_eq!(imported_error.kind, expected);
    }
}

#[test]
fn missing_or_malformed_sgf_is_not_a_successful_preview() {
    for response in [
        json!({"status":0,"result":{}}),
        json!({"status":0,"result":{"sgf":12}}),
        json!({"result":{"sgf":SOURCE_TREE}}),
    ] {
        let error = fetch_public_preview(
            &transport(200, response),
            "https://home.yikeweiqi.com/#/live/new-room/1",
        )
        .unwrap_err();
        assert_eq!(error.kind, ProviderErrorKind::InvalidPayload);
    }
    let malformed = RecordingProviderTransport::with_result(Ok(fetch_result(
        200,
        "{\"private\":\"raw-secret\" broken".to_string(),
    )));
    let error = fetch_public_preview(&malformed, "https://home.yikeweiqi.com/#/live/new-room/1").unwrap_err();
    assert_eq!(error.kind, ProviderErrorKind::InvalidPayload);
    assert!(!error.message.contains("raw-secret"));
}

#[test]
fn http_auth_not_found_and_temporary_failures_are_typed_and_sanitized() {
    for (status, kind) in [
        (401, ProviderErrorKind::AuthenticationFailed),
        (403, ProviderErrorKind::AuthenticationFailed),
        (404, ProviderErrorKind::NotFound),
        (502, ProviderErrorKind::TransportFailed),
    ] {
        let error = fetch_public_preview(
            &transport(status, json!({"private":"raw-secret"})),
            "https://home.yikeweiqi.com/#/live/new-room/1",
        )
        .unwrap_err();
        assert_eq!(error.kind, kind);
        assert!(error.message.contains(&format!("HTTP {status}")));
        assert!(!error.message.contains("raw-secret"));
    }
}

fn list_entry(id: u64, version: u64, hall: u64, room: u64) -> Value {
    json!({"Id":id,"Version":version,"hall":hall,"room":room,"GameName":"Title","BlackName":"Black","WhiteName":"White","Status":2,"HandsCount":123})
}

#[test]
fn recommend_and_local_keep_their_own_page_cursor_and_deduplicated_locators() {
    for (category, official) in [(YikeCategoryDto::Recommend, "1"), (YikeCategoryDto::Local, "0")] {
        let transport = transport(
            200,
            json!({"Status":1200,"Result":{"since":196115,"list":[list_entry(196106,2,0,0),list_entry(196106,2,0,0),list_entry(18328,1,1,15630642)]}}),
        );
        let page = fetch_public_list(&transport as &dyn ProviderTransport, category, 2, 196115).unwrap();
        assert_eq!(page.category, category);
        assert_eq!(page.page, 2);
        assert_eq!(page.since, 196115);
        assert!(page.has_more);
        assert_eq!(page.games.len(), 2);
        assert_eq!(
            page.games[0].locator,
            "https://home.yikeweiqi.com/#/live/new-room/196106"
        );
        assert_eq!(
            page.games[1].locator,
            "https://home.yikeweiqi.com/#/live/room/18328/1/15630642"
        );
        assert_eq!(page.games[0].title, "Title");
        assert_eq!(page.games[0].black_name, "Black");
        assert_eq!(page.games[0].white_name, "White");
        assert_eq!(page.games[0].status, "正在直播");
        assert_eq!(page.games[0].move_count, 123);
        let requests = transport.requests().unwrap();
        assert_eq!(requests.len(), 1);
        assert_eq!(
            requests[0].url,
            format!(
                "https://api.yikeweiqi.com/v2/golive/list?p=2&since=196115&official={official}&version=2"
            )
        );
        assert_signed(&requests[0]);
    }
}

#[test]
fn genuine_empty_list_is_explicit_end_but_nonempty_partial_page_is_not_guessed_end() {
    for items in [Vec::new(), vec![list_entry(1, 2, 0, 0)]] {
        let nonempty = !items.is_empty();
        let transport = transport(200, json!({"Status":1200,"Result":{"since":42,"list":items}}));
        let page = fetch_public_list(&transport, YikeCategoryDto::Recommend, 1, 0).unwrap();
        assert_eq!(page.has_more, nonempty);
        assert_eq!(page.since, 42);
        assert_eq!(page.games.is_empty(), !nonempty);
    }
}

#[test]
fn malformed_list_is_never_an_empty_result() {
    for response in [
        json!({"Status":1200}),
        json!({"Status":1200,"Result":null}),
        json!({"Status":1200,"Result":{"since":0}}),
        json!({"Status":1200,"Result":{"since":0,"list":{}}}),
        json!({"Status":1200,"Result":{"since":-1,"list":[]}}),
        json!({"Status":1200,"Result":{"list":[]}}),
        json!({"Status":1200,"Result":{"since":0,"list":[null]}}),
        json!({"Status":1200,"Result":{"since":0,"list":[{}]}}),
        json!({"Status":1200,"Result":{"since":0,"list":[{"Id":1,"Version":"invalid"}]}}),
        json!({"Status":1200,"Result":{"since":0,"list":[list_entry(0,2,0,0)]}}),
    ] {
        let error = fetch_public_list(&transport(200, response), YikeCategoryDto::Local, 1, 0).unwrap_err();
        assert_eq!(error.kind, ProviderErrorKind::InvalidPayload);
    }
    let error = fetch_public_list(
        &transport(
            200,
            json!({"Status":1403,"Message":"raw-secret","Result":{"since":0,"list":[]}}),
        ),
        YikeCategoryDto::Recommend,
        1,
        0,
    )
    .unwrap_err();
    assert_eq!(error.kind, ProviderErrorKind::AuthenticationFailed);
    assert!(!error.message.contains("raw-secret"));
}

#[test]
fn invalid_page_and_cursor_are_rejected_before_transport() {
    let transport = transport(200, json!({"Status":1200,"Result":{"since":0,"list":[]}}));
    for (page, since) in [(0, 0), (u32::MAX, 0), (1, u64::MAX)] {
        let error = fetch_public_list(&transport, YikeCategoryDto::Local, page, since).unwrap_err();
        assert_eq!(error.kind, ProviderErrorKind::InvalidRequest);
    }
    assert!(transport.requests().unwrap().is_empty());
}

#[test]
fn import_payload_rejects_error_or_malformed_status_and_nested_incidental_sgf() {
    for payload in [
        json!({"status":403,"sgf":SOURCE_TREE}),
        json!({"status":"invalid","sgf":SOURCE_TREE}),
        json!({"private":{"sgf":SOURCE_TREE}}),
        json!({"code":1,"data":null,"backup":{"sgf":SOURCE_TREE}}),
    ] {
        assert!(import_payload(ProviderImportRequest {
            provider: ProviderKind::Yike,
            payload: payload.to_string(),
            source_url: None,
            source_id: None,
            metadata: ProviderGameMetadata::default(),
        })
        .is_err());
    }
}

struct AnonymousGameService {
    detail: Value,
}

impl ProviderTransport for AnonymousGameService {
    fn fetch(&self, request: &ProviderFetchRequest) -> provider_core::ProviderResult<ProviderFetchResult> {
        if request.url == "https://api-new.yikeweiqi.com/v1/user/tpop/authorizations/anonymous" {
            let body: Value = serde_json::from_str(request.body.as_deref().unwrap_or("null")).unwrap();
            if request.method != ProviderFetchMethod::Post
                || body != json!({"tpop_name":"og","refresh_flag":0})
            {
                return Ok(fetch_result(400, json!({"status":400}).to_string()));
            }
            return Ok(fetch_result(
                200,
                json!({"status":0,"result":{"token":"anonymous-test-token"}}).to_string(),
            ));
        }
        if request.headers.get("Authorization") != Some(&format!("Bearer {}", "anonymous-test-token")) {
            return Ok(fetch_result(
                200,
                json!({"code":1,"msg":"invalid token"}).to_string(),
            ));
        }
        Ok(fetch_result(200, self.detail.to_string()))
    }
}

#[test]
fn unite_public_preview_uses_anonymous_access_and_a_reachable_browser_locator() {
    let service = AnonymousGameService {
        detail: json!({"code":0,"data":{"id":42,"game_type":"territory","sgf":SOURCE_TREE,"status":"ended","result":"B+R"}}),
    };
    let preview = fetch_public_preview(&service, "https://unite.yikeweiqi.com/#/unite/42").unwrap();
    assert_eq!(
        sgf::parse_sgf(&preview.sgf_text).unwrap().result.as_deref(),
        Some("B+R")
    );
    assert_eq!(preview.summary.result.as_deref(), Some("B+R"));
    assert_eq!(
        preview.metadata.source_url.as_deref(),
        Some("https://home.yikeweiqi.com/#/unite/42")
    );
    assert_eq!(preview.metadata.room_id.as_deref(), Some("42"));
    assert!(!serde_json::to_string(&preview)
        .unwrap()
        .contains("anonymous-test-token"));
}

#[test]
fn unite_metadata_survives_sgf_only_import_save_and_reopen() {
    let source = "(;RU[cn]KM[7.5]HA[0]SZ[19]AB[aa]C[root];W[bb](;B[cc]C[main])(;B[dd]C[variation]))";
    let service = AnonymousGameService {
        detail: json!({"code":0,"data":{
            "id":42,"game_type":"territory","sgf":source,"status":"ended","result":"B+R",
            "players":{"blacks":[{"name":"黑方]\\"},{"name":"队友"}],"whites":[{"name":"白方"}]}
        }}),
    };
    let preview = fetch_public_preview(&service, "https://home.yikeweiqi.com/#/unite/42").unwrap();
    // The gateway reads these fields from SGF; SGF-07 imports only that SGF.
    let projected = sgf::parse_sgf(&preview.sgf_text).unwrap();
    assert_eq!(projected.black_name.as_deref(), Some("黑方]\\,队友"));
    assert_eq!(projected.white_name.as_deref(), Some("白方"));
    assert_eq!(projected.result.as_deref(), Some("B+R"));
    let imported = sgf::CurrentSgfDocument::open(&preview.sgf_text).unwrap();
    let saved = imported.serialize().unwrap();
    let reopened = sgf::CurrentSgfDocument::open(&saved).unwrap();
    assert_eq!(reopened.tree().unwrap(), imported.tree().unwrap());
    let mut restored = sgf::parse_sgf(&saved).unwrap().root.unwrap();
    restored
        .properties
        .retain(|property| !matches!(property.key.as_str(), "PB" | "PW" | "RE"));
    assert_eq!(restored, sgf::parse_sgf(source).unwrap().root.unwrap());
}

#[test]
fn unite_keeps_existing_source_metadata_over_separate_response_metadata() {
    let source = "(;SZ[19]PB[Source black]PW[]RE[W+2.5]C[original];B[aa](;W[bb])(;W[cc]))";
    let service = AnonymousGameService {
        detail: json!({"code":0,"data":{
            "id":42,"game_type":"territory","sgf":source,"status":"ended","result":"B+R",
            "players":{"blacks":[{"name":"Other black"}],"whites":[{"name":"Other white"}]}
        }}),
    };
    let preview = fetch_public_preview(&service, "https://home.yikeweiqi.com/#/unite/42").unwrap();
    assert_eq!(
        sgf::parse_sgf(&preview.sgf_text).unwrap().root,
        sgf::parse_sgf(source).unwrap().root
    );
    assert_eq!(preview.summary.result.as_deref(), Some("W+2.5"));
}

#[test]
fn unite_rejects_non_go_and_mismatched_room_instead_of_importing_their_sgf() {
    for (id, game_type) in [(42, "fir"), (43, "territory")] {
        let service = AnonymousGameService {
            detail: json!({"code":0,"data":{"id":id,"game_type":game_type,"sgf":SOURCE_TREE,"status":"processing","result":""}}),
        };
        let error = fetch_public_preview(&service, "https://home.yikeweiqi.com/#/unite/42").unwrap_err();
        assert_eq!(error.kind, ProviderErrorKind::InvalidPayload);
        assert!(!error.message.contains(SOURCE_TREE));
    }
}

#[test]
fn anonymous_bootstrap_failure_is_terminal_and_never_exposes_credentials() {
    for (payload, kind) in [
        (
            json!({"status":403,"message":"private-credential","result":{"token":"private-credential"}}),
            ProviderErrorKind::AuthenticationFailed,
        ),
        (
            json!({"status":0,"result":{"token":"private-credential\r\nInjected: value"}}),
            ProviderErrorKind::InvalidPayload,
        ),
        (json!({"status":0,"result":{}}), ProviderErrorKind::InvalidPayload),
    ] {
        let service = transport(200, payload);
        let error = fetch_public_preview(&service, "https://home.yikeweiqi.com/#/unite/42").unwrap_err();
        assert_eq!(error.kind, kind);
        assert!(!error.message.contains("private-credential"));
        assert!(service
            .requests()
            .unwrap()
            .iter()
            .all(|request| !request.url.contains("game-server")));
    }
}
