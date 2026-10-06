use app_model::{
    ProviderErrorKind, ProviderFetchMethod, ProviderFetchResult, ProviderGameMetadata, ProviderImportRequest,
    ProviderKind,
};
use provider_core::RecordingProviderTransport;
use serde_json::{json, Value};
use std::collections::BTreeMap;

fn transport(payload: Value) -> RecordingProviderTransport {
    raw_transport(&payload.to_string(), 200)
}

fn raw_transport(payload: &str, status_code: u16) -> RecordingProviderTransport {
    RecordingProviderTransport::with_result(Ok(ProviderFetchResult {
        provider: ProviderKind::Tencent,
        url: "https://happyapp.huanle.qq.com/?session=must-not-retain".into(),
        status_code,
        payload: payload.into(),
        headers: BTreeMap::new(),
        content_type: Some("application/json".into()),
        metadata: ProviderGameMetadata {
            extra: BTreeMap::from([("session".into(), "must-not-retain".into())]),
            ..ProviderGameMetadata::default()
        },
        warnings: vec!["raw-response-must-not-retain".into()],
    }))
}

fn games(count: usize) -> Vec<Value> {
    (1..=count).map(|id| json!({"chessid": id})).collect()
}

#[test]
fn unicode_query_and_cursor_cannot_inject_parameters() {
    let transport = transport(json!({"chesslist": []}));
    let page =
        provider_tencent::fetch_list(&transport, "  棋 手&txwqsession=secret  ", "  old&next=1  ").unwrap();
    assert_eq!(page.username, "棋 手&txwqsession=secret");
    let requests = transport.requests().unwrap();
    assert_eq!(requests.len(), 1);
    let request = &requests[0];
    assert_eq!(request.provider, ProviderKind::Tencent);
    assert_eq!(request.method, ProviderFetchMethod::Get);
    assert_eq!(request.url, "https://cgi.huanle.qq.com/cgi-bin/CommonMobileCGI/TXWQFetchChessList?type=7&lastCode=old%26next%3D1&username=%E6%A3%8B+%E6%89%8B%26txwqsession%3Dsecret&srcuid=&txwqsession=lizzieyzy-next&fetchnum=100");
    assert_eq!(
        request.headers.get("Accept").unwrap(),
        "application/json,text/plain,*/*"
    );
    assert_eq!(request.headers.get("User-Agent").unwrap(), "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0 Safari/537.36");
    assert_eq!(request.timeout_ms, Some(25_000));
    assert!(request.body.is_none());
    assert!(request.source_url.is_none());
}

#[test]
fn only_ascii_digits_supply_srcuid_and_blank_cursor_defaults_to_zero() {
    for (username, encoded, srcuid) in [
        (" 00123 ", "00123", "00123"),
        ("１２３", "%EF%BC%91%EF%BC%92%EF%BC%93", ""),
    ] {
        let transport = transport(json!({"chesslist": []}));
        provider_tencent::fetch_list(&transport, username, " \n ").unwrap();
        assert_eq!(transport.requests().unwrap()[0].url, format!("https://cgi.huanle.qq.com/cgi-bin/CommonMobileCGI/TXWQFetchChessList?type=7&lastCode=0&username={encoded}&srcuid={srcuid}&txwqsession=lizzieyzy-next&fetchnum=100"));
    }
}

#[test]
fn blank_queries_fail_before_any_transport_call() {
    let transport = transport(json!({"chesslist": [], "chess": "(;SZ[9])"}));
    for query in ["", " \t\n ", "\u{2003}"] {
        assert_eq!(
            provider_tencent::fetch_list(&transport, query, "0")
                .unwrap_err()
                .kind,
            ProviderErrorKind::InvalidRequest
        );
        assert_eq!(
            provider_tencent::fetch_preview(&transport, query)
                .unwrap_err()
                .kind,
            ProviderErrorKind::InvalidRequest
        );
    }
    assert!(transport.requests().unwrap().is_empty());
}

#[test]
fn nonempty_batches_continue_across_twenty_five_row_boundaries() {
    for count in [0, 1, 25, 26, 99, 100] {
        let transport = transport(json!({"chesslist": games(count)}));
        let page = provider_tencent::fetch_list(&transport, "name", "old").unwrap();
        assert_eq!(page.games.len(), count);
        assert_eq!(
            page.last_code,
            if count == 0 {
                "old".into()
            } else {
                count.to_string()
            }
        );
        assert_eq!(page.has_more, count != 0);
    }
}

#[test]
fn cursor_uses_last_raw_item_before_deduplication() {
    let mut rows = games(99);
    rows.push(json!({"chessid": "001", "blacknick": "first"}));
    rows[0] = json!({"chessid": "001", "blacknick": "first"});
    let transport = transport(json!({"chesslist": rows}));
    let page = provider_tencent::fetch_list(&transport, "name", "0").unwrap();
    assert_eq!(page.games.len(), 99);
    assert_eq!(page.last_code, "001");
    assert!(page.has_more);
    assert_eq!(page.games.last().unwrap().chess_id, "99");
}

#[test]
fn full_repeated_cursor_page_ends_even_when_rows_are_new() {
    let transport = transport(json!({"chesslist": games(100)}));
    let page = provider_tencent::fetch_list(&transport, "name", "100").unwrap();
    assert_eq!(page.games.len(), 100);
    assert_eq!(page.last_code, "100");
    assert!(!page.has_more);
}

#[test]
fn oversized_or_missing_id_rows_are_errors_not_false_end_of_list() {
    for payload in [
        json!({"chesslist": games(101)}),
        json!({"chesslist": [{"chessid": "ok"}, {}]}),
        json!({"chesslist": [{"chessid": "  "}]}),
        json!({"chesslist": [{"chessid": null}]}),
        json!({"chesslist": [{"chessid": {"bad": "id"}}]}),
        json!({"chesslist": [1]}),
        json!({"chesslist": {}}),
        json!({}),
    ] {
        let error = provider_tencent::fetch_list(&transport(payload), "name", "0").unwrap_err();
        assert_eq!(error.kind, ProviderErrorKind::InvalidPayload);
    }
}

#[test]
fn list_names_ranks_time_results_and_opaque_ids_are_normalized() {
    let transport = transport(json!({"chesslist": [
        {"chessid": " 0007 ", "blacknick": "  黑棋  ", "blackenname": "ignored", "whitenick": " ", "whiteenname": "White", "blackdan": "100", "whitedan": 17, "starttime": " 2026-10-05 12:30:00 ", "gamestarttime": 1, "winner": 1, "point": -1, "movenum": "123"},
        {"chessid": 9223372036854775807i64, "blackname": "Black", "whiteuid": 42, "blackdan": 18, "whitedan": 1, "gamestarttime": 1791190000, "winner": 2, "point": 250, "rule": 1},
        {"chessid": "last", "blackuid": "0088", "blackdan": 0, "whitedan": 109, "winner": 1, "point": -2, "movenum": -1}
    ]}));
    let page = provider_tencent::fetch_list(&transport, "name", "0").unwrap();
    assert_eq!(page.last_code, "last");
    let first = &page.games[0];
    assert_eq!(first.chess_id, "0007");
    assert_eq!((&first.black_name[..], &first.white_name[..]), ("黑棋", "White"));
    assert_eq!((&first.black_rank[..], &first.white_rank[..]), ("P1段", "1级"));
    assert_eq!(first.played_at, "2026-10-05 12:30:00");
    assert_eq!(first.result, "Black wins by resignation");
    assert_eq!(first.move_count, 123);
    let second = &page.games[1];
    assert_eq!(second.chess_id, "9223372036854775807");
    assert_eq!((&second.black_name[..], &second.white_name[..]), ("Black", "42"));
    assert_eq!((&second.black_rank[..], &second.white_rank[..]), ("1段", "17级"));
    assert_eq!(second.played_at, "1791190000");
    assert_eq!(second.result, "White +2.5 stones");
    assert_eq!(page.games[2].black_name, "0088");
    assert_eq!(page.games[2].white_name, "");
    assert_eq!(page.games[2].black_rank, "");
    assert_eq!(page.games[2].white_rank, "P10段");
    assert_eq!(page.games[2].result, "Black wins on time");
    assert_eq!(page.games[2].move_count, 0);
}

fn import_request(payload: &str) -> ProviderImportRequest {
    ProviderImportRequest {
        provider: ProviderKind::Tencent,
        payload: payload.into(),
        source_url: Some("https://example.test/?token=must-not-retain".into()),
        source_id: Some("opaque-id".into()),
        metadata: ProviderGameMetadata {
            extra: BTreeMap::from([("session".into(), "must-not-retain".into())]),
            ..ProviderGameMetadata::default()
        },
    }
}

#[test]
fn zero_or_absent_status_fields_succeed_but_nonintegers_and_nonzero_are_terminal() {
    for status in [json!(0), json!("0"), json!(" 0 ")] {
        let payload = json!({"result": status, "ret": 0, "chesslist": [], "chess": "(;SZ[9])"});
        let transport = transport(payload);
        provider_tencent::fetch_list(&transport, "name", "0").unwrap();
        provider_tencent::fetch_preview(&transport, "id").unwrap();
    }
    for key in ["result", "ret"] {
        for status in [
            json!(1),
            json!(-1),
            json!("1"),
            json!(0.0),
            json!(0.5),
            json!("0.0"),
            json!(true),
            json!(null),
            json!([]),
            json!({}),
        ] {
            let mut payload = json!({"chesslist": [], "chess": "(;SZ[9])", "resultstr": "session=never-echo", "token": "never-echo"});
            payload[key] = status;
            let transport = transport(payload.clone());
            for error in [
                provider_tencent::fetch_list(&transport, "name", "0").unwrap_err(),
                provider_tencent::fetch_preview(&transport, "id").unwrap_err(),
                provider_tencent::import_payload(import_request(&payload.to_string())).unwrap_err(),
            ] {
                assert_eq!(error.kind, ProviderErrorKind::InvalidPayload);
                assert!(!error.message.contains("never-echo"));
            }
            assert_eq!(transport.requests().unwrap().len(), 2);
        }
    }
}

#[test]
fn malformed_detail_and_json_errors_never_echo_payload_or_diagnostics() {
    for payload in [
        "session=never-echo",
        "null",
        "[]",
        "{\"resultstr\":\"never-echo\"}",
        "{\"chess\":42}",
        "{\"chess\":\"  \"}",
    ] {
        let error = provider_tencent::fetch_preview(&raw_transport(payload, 200), "id").unwrap_err();
        assert_eq!(error.kind, ProviderErrorKind::InvalidPayload);
        assert!(!error.message.contains("never-echo"));
    }
}

#[test]
fn http_failures_are_classified_before_payload_parsing() {
    for (status, kind) in [
        (403, ProviderErrorKind::AuthenticationFailed),
        (404, ProviderErrorKind::NotFound),
        (503, ProviderErrorKind::TransportFailed),
    ] {
        let transport = raw_transport("signed-token-never-echo", status);
        assert_eq!(
            provider_tencent::fetch_list(&transport, "name", "0")
                .unwrap_err()
                .kind,
            kind
        );
        let error = provider_tencent::fetch_preview(&transport, "id").unwrap_err();
        assert_eq!(error.kind, kind);
    }
}

#[test]
fn escaped_properties_setup_and_all_variations_survive_tencent_preview() {
    let clean = r"(;GM[1]FF[4]SZ[9:13]KM[7.5]HA[2]RU[Chinese]PB[A\]lice]PW[Bob\\Lee]RE[W+R]DT[2026-10-05]AB[aa][cc]AW[dd]C[escaped \] bracket and \\ slash (;)] ;B[ee](;W[ff]C[first];B[])(;W[gg]C[second];AB[hh]AE[aa]))";
    let polluted = format!(" \u{feff}\\{clean}\\ \n");
    let transport =
        transport(json!({"chess": polluted, "chessid": "different-id", "txwqsession": "must-not-retain"}));
    let result = provider_tencent::fetch_preview(&transport, "  棋&id=1  ").unwrap();
    let request = &transport.requests().unwrap()[0];
    assert_eq!(
        request.url,
        "https://happyapp.huanle.qq.com/cgi-bin/CommonMobileCGI/TXWQFetchChess?chessid=%E6%A3%8B%26id%3D1"
    );
    assert_eq!(request.source_id.as_deref(), Some("棋&id=1"));
    assert_eq!(result.provider, ProviderKind::Tencent);
    assert_eq!(result.summary.provider, ProviderKind::Tencent);
    assert_eq!(result.summary.source_id.as_deref(), Some("棋&id=1"));
    assert_eq!(
        (result.summary.board_width, result.summary.board_height),
        (Some(9), Some(13))
    );
    assert_eq!(result.summary.komi, Some(7.5));
    assert_eq!(result.summary.handicap, Some(2));
    assert_eq!(result.summary.black_name.as_deref(), Some("A]lice"));
    assert_eq!(result.summary.white_name.as_deref(), Some("Bob\\Lee"));
    assert_eq!(result.summary.result.as_deref(), Some("W+R"));
    assert_eq!(result.summary.date.as_deref(), Some("2026-10-05"));
    assert_eq!(result.summary.move_count, Some(3));
    assert_eq!(result.sgf_text, clean);
    assert_eq!(
        sgf::parse_sgf(&result.sgf_text).unwrap().root,
        sgf::parse_sgf(clean).unwrap().root
    );
    assert_eq!(
        result.metadata,
        ProviderGameMetadata {
            source_id: Some("棋&id=1".into()),
            ..ProviderGameMetadata::default()
        }
    );
    assert!(result.warnings.is_empty());
}

#[test]
fn invalid_side_variation_or_setup_is_rejected_before_preview_success() {
    for chess in [
        "(;SZ[9];B[aa](;W[bb])(;W[zz]))",
        "(;SZ[9]AB[zz])",
        "(;SZ[9];B[aa])signed-session-never-echo",
        "(;SZ[9]C[unterminated)",
    ] {
        let error = provider_tencent::fetch_preview(&transport(json!({"chess": chess})), "id").unwrap_err();
        assert_eq!(error.kind, ProviderErrorKind::ParseFailed);
        assert_eq!(error.message, "Tencent chess SGF could not be parsed.");
    }
}

#[test]
fn pasted_sgf_or_frozen_json_are_validated_with_independent_identity() {
    let clean = r"(;SZ[9]KM[6.5]PB[Black]PW[White]C[keep\]this];B[aa](;W[bb])(;W[cc]))";
    for payload in [
        format!(" \u{feff} \u{feff}\\{clean}\\ "),
        json!({"chess": clean, "result": "0", "ret": 0}).to_string(),
    ] {
        let result = provider_tencent::import_payload(import_request(&payload)).unwrap();
        assert_eq!(result.provider, ProviderKind::Tencent);
        assert_eq!(result.summary.provider, ProviderKind::Tencent);
        assert_eq!(result.sgf_text, clean);
        assert_eq!(
            result.metadata,
            ProviderGameMetadata {
                source_id: Some("opaque-id".into()),
                ..ProviderGameMetadata::default()
            }
        );
    }
    let mut request = import_request(clean);
    request.provider = ProviderKind::Fox;
    assert_eq!(
        provider_tencent::import_payload(request).unwrap_err().kind,
        ProviderErrorKind::InvalidRequest
    );
    assert_eq!(
        provider_tencent::import_payload(import_request(" \n "))
            .unwrap_err()
            .kind,
        ProviderErrorKind::InvalidRequest
    );
    assert_eq!(
        provider_tencent::import_payload(import_request("(;SZ[9];B[zz])"))
            .unwrap_err()
            .kind,
        ProviderErrorKind::ParseFailed
    );
}

#[test]
fn komi_uses_frozen_shared_parser_units_without_flattening_the_tree() {
    for (properties, expected) in [
        ("AP[GNU Go:3.8]AP[foxwq]RU[Chinese]KM[375]", 7.5),
        ("AP[FoXwQ]RU[Japanese]KM[650]", 6.5),
        ("AP[foxwq]RU[Chinese]KM[-375]", -7.5),
        ("AP[foxwq]RU[Chinese]KM[3.75]", 3.75),
        ("AP[other]RU[Chinese]KM[375]", 7.5),
        ("AP[other]RU[Japanese]KM[650]", 6.5),
        ("AP[other]RU[Chinese]KM[3.75]", 7.5),
        ("AP[other]KM[-3.25]", -6.5),
        ("AP[other]KM[7.5]", 7.5),
    ] {
        let original = format!(
            r"(;SZ[9]{properties}AB[aa][cc]C[root\] escaped];B[dd](;W[ee]C[first])(;W[ff]AW[hh]C[second\\slash]))"
        );
        let result = provider_tencent::fetch_preview(&transport(json!({"chess": original})), "id").unwrap();
        assert_eq!(result.summary.komi, Some(expected));
        let mut before = sgf::parse_sgf(&original).unwrap();
        for property in &mut before.root.as_mut().unwrap().properties {
            if property.key == "KM" {
                property.values = vec![expected.to_string()];
            }
        }
        let reopened = sgf::parse_sgf(&result.sgf_text).unwrap();
        assert_eq!(reopened.komi, expected);
        assert_eq!(reopened.root, before.root);
        assert_eq!(result.provider, ProviderKind::Tencent);
    }
}

#[test]
fn short_duplicate_batches_and_short_repeated_cursors_have_distinct_end_states() {
    let transport =
        transport(json!({"chesslist": [{"chessid": "first"}, {"chessid": "last"}, {"chessid": "first"}]}));
    let continuing = provider_tencent::fetch_list(&transport, "name", "old").unwrap();
    assert_eq!(continuing.games.len(), 2);
    assert_eq!(continuing.last_code, "first");
    assert!(continuing.has_more);
    let terminal = provider_tencent::fetch_list(&transport, "name", "first").unwrap();
    assert_eq!(terminal.games.len(), 2);
    assert!(!terminal.has_more);
}
