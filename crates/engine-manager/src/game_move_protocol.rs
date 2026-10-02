use app_model::{
    EngineBackend, EngineRunDto, ExactPositionDto, ExactRulesDto, GameMoveDto, MoveVertex, PlayerColor,
    PointDto,
};
use serde_json::{json, Value};

pub(crate) fn katago_query(position: &ExactPositionDto, query_id: &str, max_visits: u32) -> Value {
    let (ko, white_handicap_bonus) = match position.rules {
        ExactRulesDto::Chinese => ("SIMPLE", "N"),
        ExactRulesDto::ChineseKgs => ("POSITIONAL", "N"),
    };
    let initial_stones: Vec<_> = position
        .initial_stones
        .iter()
        .map(|stone| {
            json!([
                color_text(stone.color),
                point_text(stone.x, stone.y, position.board_height)
            ])
        })
        .collect();
    let moves: Vec<_> = position
        .moves
        .iter()
        .map(|played| {
            json!([
                color_text(played.color),
                vertex_text(&played.vertex, position.board_height)
            ])
        })
        .collect();
    json!({
        "id": query_id,
        "initialStones": initial_stones,
        "moves": moves,
        "initialPlayer": color_text(position.initial_player),
        "rules": {
            "ko": ko,
            "scoring": "AREA",
            "tax": "NONE",
            "suicide": false,
            "hasButton": false,
            "whiteHandicapBonus": white_handicap_bonus,
            "friendlyPassOk": true
        },
        "komi": position.komi,
        "boardXSize": position.board_width,
        "boardYSize": position.board_height,
        "analyzeTurns": [position.moves.len()],
        "maxVisits": max_visits
    })
}

pub(crate) enum KataGoMoveResponse {
    Searching,
    Complete(GameMoveDto),
    Rejected(String),
}

pub(crate) fn katago_result(
    line: &str,
    query_id: &str,
    turn: usize,
    width: u8,
    height: u8,
) -> Result<KataGoMoveResponse, String> {
    let response: Value =
        serde_json::from_str(line).map_err(|error| format!("Invalid KataGo game-move JSON: {error}"))?;
    let response = response
        .as_object()
        .ok_or_else(|| "KataGo game-move response must be an object".to_owned())?;
    if response.get("id").and_then(Value::as_str) != Some(query_id) {
        return Err("KataGo game-move response has a missing or mismatched query id".to_owned());
    }
    for field in ["error", "errors"] {
        if let Some(detail) = response.get(field) {
            return Ok(KataGoMoveResponse::Rejected(format!(
                "KataGo game-move {field}: {detail}"
            )));
        }
    }
    // Warning responses have no turnNumber. Even when followed by a successful
    // result, they may describe a rules downgrade and cannot be ignored.
    for field in ["warning", "warnings"] {
        if let Some(detail) = response.get(field) {
            return Err(format!("KataGo game-move {field}: {detail}"));
        }
    }
    if response.get("turnNumber").and_then(Value::as_u64) != Some(turn as u64) {
        return Err("KataGo game-move response has a missing or mismatched turn".to_owned());
    }
    match response.get("isDuringSearch").and_then(Value::as_bool) {
        Some(true) => return Ok(KataGoMoveResponse::Searching),
        Some(false) => {}
        None => return Err("KataGo game-move response lacks an explicit completion state".to_owned()),
    }
    let moves = response
        .get("moveInfos")
        .and_then(Value::as_array)
        .ok_or_else(|| "KataGo completed response lacks moveInfos".to_owned())?;
    let mut best = moves
        .iter()
        .filter(|candidate| candidate.get("order").and_then(Value::as_u64) == Some(0));
    let candidate = best
        .next()
        .ok_or_else(|| "KataGo completed response lacks an integer order=0 move".to_owned())?;
    if best.next().is_some() {
        return Err("KataGo completed response has multiple order=0 moves".to_owned());
    }
    let raw = candidate
        .get("move")
        .and_then(Value::as_str)
        .ok_or_else(|| "KataGo order=0 move is not a string".to_owned())?;
    parse_gtp_move(raw, width, height, false).map(KataGoMoveResponse::Complete)
}

pub(crate) fn parse_gtp_move(
    raw: &str,
    width: u8,
    height: u8,
    allow_resign: bool,
) -> Result<GameMoveDto, String> {
    if width == 0 || width > 25 || height == 0 {
        return Err("Move response board dimensions are outside GTP coordinate support".to_owned());
    }
    let token = raw.trim();
    if token.eq_ignore_ascii_case("pass") {
        return Ok(GameMoveDto::Move {
            vertex: MoveVertex::Pass,
        });
    }
    if token.eq_ignore_ascii_case("resign") {
        return if allow_resign {
            Ok(GameMoveDto::Resign)
        } else {
            Err("KataGo analysis cannot return resignation".to_owned())
        };
    }
    let bytes = token.as_bytes();
    if bytes.len() < 2 {
        return Err("Move response must contain one GTP vertex, pass, or resign".to_owned());
    }
    let column = bytes[0].to_ascii_uppercase();
    if !column.is_ascii_uppercase()
        || column == b'I'
        || !(b'1'..=b'9').contains(&bytes[1])
        || !bytes[1..].iter().all(u8::is_ascii_digit)
    {
        return Err("Move response contains a malformed GTP vertex or multiple tokens".to_owned());
    }
    let x = column - b'A' - u8::from(column > b'I');
    let row: u8 = token[1..]
        .parse()
        .map_err(|_| "Move response row is outside the board".to_owned())?;
    if x >= width || row > height {
        return Err("Move response vertex is outside the board".to_owned());
    }
    Ok(GameMoveDto::Move {
        vertex: MoveVertex::Point(PointDto { x, y: height - row }),
    })
}

pub(crate) fn qualified_gtp_launch(
    profile: &app_model::EngineProfileDto,
    facts: &app_model::EngineGtpFactsDto,
) -> bool {
    const ARGV: [&str; 9] = [
        "--mode",
        "gtp",
        "--chinese-rules",
        "--positional-superko",
        "--forbid-suicide",
        "--level",
        "1",
        "--seed",
        "1",
    ];
    profile.adapter_kind() == EngineBackend::GenericGtp
        && facts.protocol_version == 2
        && facts.name == "GNU Go"
        && facts.version == "3.8"
        && profile.argv.iter().map(String::as_str).eq(ARGV)
        && ["boardsize", "clear_board", "komi", "play", "genmove"]
            .iter()
            .all(|required| facts.commands.iter().any(|command| command == required))
}

pub(crate) fn gtp_sync_plan(run: &EngineRunDto, position: &ExactPositionDto) -> Result<Vec<String>, String> {
    let capabilities = run
        .capability_snapshot
        .as_ref()
        .ok_or_else(|| "Exact GTP synchronization requires observed capabilities".to_owned())?;
    let facts = capabilities
        .gtp
        .as_ref()
        .ok_or_else(|| "Exact GTP synchronization requires observed GTP identity".to_owned())?;
    if run.adapter_kind != EngineBackend::GenericGtp
        || capabilities.adapter_kind != EngineBackend::GenericGtp
        || !qualified_gtp_launch(&run.profile_snapshot, facts)
    {
        return Err("Exact GTP synchronization requires the qualified GNU Go 3.8 launch".to_owned());
    }
    if position.rules != ExactRulesDto::ChineseKgs {
        return Err("Qualified GNU Go rules require ChineseKgs positional superko".to_owned());
    }
    if position.board_width != position.board_height || ![2, 5, 9, 13, 19].contains(&position.board_width) {
        return Err("Board dimensions are outside the qualified GNU Go sizes".to_owned());
    }
    if !position.komi.is_finite() || position.komi.abs() > 1000.0 || (position.komi * 2.0).fract() != 0.0 {
        return Err("Komi is not exactly representable by the qualified GTP mapping".to_owned());
    }
    if !position.initial_stones.is_empty() {
        return Err("Exact GNU Go handicap-rule compensation has not been qualified".to_owned());
    }
    for played in &position.moves {
        if let MoveVertex::Point(point) = &played.vertex {
            if point.x >= position.board_width || point.y >= position.board_height {
                return Err("Move history has an out-of-bounds vertex".to_owned());
            }
        }
    }
    let mut commands = Vec::with_capacity(3 + position.moves.len());
    commands.push(format!("boardsize {}", position.board_width));
    commands.push("clear_board".to_owned());
    commands.push(format!("komi {}", position.komi));
    for played in &position.moves {
        commands.push(format!(
            "play {} {}",
            color_text(played.color),
            vertex_text(&played.vertex, position.board_height)
        ));
    }
    Ok(commands)
}

pub(crate) fn vertex_text(vertex: &MoveVertex, height: u8) -> String {
    match vertex {
        MoveVertex::Point(point) => point_text(point.x, point.y, height),
        MoveVertex::Pass => "pass".to_owned(),
    }
}

fn point_text(x: u8, y: u8, height: u8) -> String {
    let column = char::from(b'A' + x + u8::from(x >= 8));
    format!("{column}{}", height - y)
}

pub(crate) fn color_text(color: PlayerColor) -> &'static str {
    match color {
        PlayerColor::Black => "B",
        PlayerColor::White => "W",
    }
}
