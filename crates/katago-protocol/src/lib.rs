use app_model::{
    AnalysisFrameDto, AnalysisJobId, CandidateMoveDto, ContinuousAnalysisBudgetDto, GameDto, MoveDto,
    MoveVertex, PlayerColor, PointDto,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type KataMove = (String, String);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisQuery {
    pub id: String,
    pub moves: Vec<KataMove>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub initial_stones: Vec<KataMove>,
    pub rules: String,
    pub komi: f32,
    pub board_x_size: u8,
    pub board_y_size: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub analyze_turns: Option<Vec<u32>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_visits: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub include_ownership: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub include_policy: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub report_during_search_every: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub override_settings: Option<ContinuousSearchSettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContinuousSearchSettings {
    pub max_time: f64,
    pub max_visits: u64,
    pub max_playouts: u64,
}

#[derive(Debug, Clone)]
pub struct AnalysisQueryOptions {
    pub id: String,
    pub rules: String,
    pub turn: u32,
    pub max_visits: Option<u32>,
    pub include_ownership: Option<bool>,
    pub include_policy: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct AnalysisBatchQueryOptions {
    pub id: String,
    pub rules: String,
    pub analyze_turns: Option<Vec<u32>>,
    pub max_visits: Option<u32>,
    pub include_ownership: Option<bool>,
    pub include_policy: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisResponse {
    pub id: String,
    #[serde(default)]
    pub turn_number: u32,
    #[serde(default)]
    pub root_info: Option<RootInfo>,
    #[serde(default)]
    pub move_infos: Vec<MoveInfo>,
    #[serde(default)]
    pub ownership: Option<Vec<f32>>,
    #[serde(default)]
    pub policy: Option<Vec<f32>>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub warning: Option<String>,
    #[serde(default)]
    pub is_during_search: Option<bool>,
    #[serde(default)]
    pub no_results: bool,
    #[serde(default)]
    pub action: Option<String>,
    #[serde(default)]
    pub field: Option<String>,
}

impl AnalysisResponse {
    /// Validates search accounting; publishing a frame additionally requires candidates.
    pub fn has_valid_search_result(&self, board_size: u8) -> bool {
        let Some(root) = &self.root_info else {
            return false;
        };
        let valid_vertex = |vertex: &str| {
            if vertex.eq_ignore_ascii_case("pass") {
                return true;
            }
            let bytes = vertex.as_bytes();
            if bytes.len() < 2 || !bytes[0].is_ascii_alphabetic() || bytes[0].eq_ignore_ascii_case(&b'I') {
                return false;
            }
            let col = bytes[0].to_ascii_uppercase();
            let x = col - b'A' - u8::from(col > b'I');
            x < board_size
                && vertex[1..]
                    .parse::<u8>()
                    .is_ok_and(|row| row > 0 && row <= board_size)
        };
        let area = usize::from(board_size).pow(2);
        !self.no_results
            && root.visits > 0
            && (0.0..=1.0).contains(&root.winrate)
            && root.score_lead.unwrap_or(root.score_mean).is_finite()
            && root
                .score_stdev
                .is_none_or(|value| value.is_finite() && value >= 0.0)
            && self.move_infos.iter().all(|info| {
                info.move_.as_deref().is_some_and(valid_vertex)
                    && (0.0..=1.0).contains(&info.winrate)
                    && info.score_mean.is_finite()
                    && info.prior.is_none_or(|value| (0.0..=1.0).contains(&value))
                    && info.pv.iter().all(|vertex| valid_vertex(vertex))
            })
            && self.ownership.as_ref().is_none_or(|values| {
                values.len() == area && values.iter().all(|value| (-1.0..=1.0).contains(value))
            })
            && self.policy.as_ref().is_none_or(|values| {
                values.len() == area + 1
                    && values
                        .iter()
                        .all(|value| *value == -1.0 || (0.0..=1.0).contains(value))
            })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RootInfo {
    #[serde(default)]
    pub visits: u32,
    #[serde(default)]
    pub winrate: f32,
    #[serde(default)]
    pub score_mean: f32,
    #[serde(default)]
    pub score_lead: Option<f32>,
    #[serde(default)]
    pub score_stdev: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveInfo {
    #[serde(rename = "move")]
    pub move_: Option<String>,
    #[serde(default)]
    pub visits: u32,
    #[serde(default)]
    pub winrate: f32,
    #[serde(default)]
    pub score_mean: f32,
    #[serde(default)]
    pub prior: Option<f32>,
    #[serde(default)]
    pub pv: Vec<String>,
}

#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("json parse error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("KataGo returned error: {0}")]
    Engine(String),
    #[error("move vertex ({x}, {y}) is outside board size {board_size}")]
    InvalidVertex { x: u8, y: u8, board_size: u8 },
}

impl AnalysisQuery {
    pub fn to_jsonl(&self) -> Result<String, ProtocolError> {
        Ok(format!("{}\n", serde_json::to_string(self)?))
    }

    pub fn continuous(&mut self, budget: ContinuousAnalysisBudgetDto) {
        self.max_visits = None;
        self.report_during_search_every = Some(0.1);
        self.override_settings = Some(ContinuousSearchSettings {
            max_time: if budget.continuous_time_limit_enabled {
                f64::from(budget.continuous_time_limit_seconds)
            } else {
                1e20
            },
            // KataGo's unbounded search sentinel, overriding finite config limits.
            max_visits: if budget.continuous_visits_limit_enabled {
                u64::from(budget.continuous_visits_limit)
            } else {
                1 << 50
            },
            max_playouts: 1 << 50,
        });
    }
}

pub fn parse_response_line(line: &str) -> Result<AnalysisResponse, ProtocolError> {
    let response: AnalysisResponse = serde_json::from_str(line)?;
    if let Some(error) = &response.error {
        return Err(ProtocolError::Engine(error.clone()));
    }
    Ok(response)
}

pub fn analysis_query_from_game(
    game: &GameDto,
    options: AnalysisQueryOptions,
) -> Result<AnalysisQuery, ProtocolError> {
    let board_size = game.summary.board_size;
    let turn = options.turn.min(game.moves.len() as u32);
    let moves = game
        .moves
        .iter()
        .take(turn as usize)
        .map(|move_| move_dto_to_kata_move(move_, board_size))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(AnalysisQuery {
        id: options.id,
        moves,
        initial_stones: Vec::new(),
        rules: options.rules,
        komi: game.summary.komi,
        board_x_size: board_size,
        board_y_size: board_size,
        analyze_turns: Some(vec![turn]),
        max_visits: options.max_visits,
        include_ownership: options.include_ownership,
        include_policy: options.include_policy,
        report_during_search_every: None,
        override_settings: None,
    })
}

pub fn analysis_query_from_position(
    board_size: u8,
    komi: f32,
    stones: &[app_model::StoneDto],
    to_play: PlayerColor,
    options: AnalysisQueryOptions,
) -> Result<AnalysisQuery, ProtocolError> {
    let initial_stones = stones
        .iter()
        .map(|stone| {
            Ok::<KataMove, ProtocolError>((
                player_color_to_kata(stone.color).to_string(),
                point_to_kata_coordinate(
                    &PointDto {
                        x: stone.x,
                        y: stone.y,
                    },
                    board_size,
                )?,
            ))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let moves = if to_play == PlayerColor::White {
        vec![("B".to_string(), "pass".to_string())]
    } else {
        Vec::new()
    };
    let turn = u32::try_from(moves.len()).expect("move count fits u32");
    Ok(AnalysisQuery {
        id: options.id,
        moves,
        initial_stones,
        rules: options.rules,
        komi,
        board_x_size: board_size,
        board_y_size: board_size,
        analyze_turns: Some(vec![turn]),
        max_visits: options.max_visits,
        include_ownership: options.include_ownership,
        include_policy: options.include_policy,
        report_during_search_every: None,
        override_settings: None,
    })
}

pub fn terminate_action_jsonl(control_id: &str, target_id: &str) -> String {
    let mut line =
        serde_json::json!({ "id": control_id, "action": "terminate", "terminateId": target_id }).to_string();
    line.push('\n');
    line
}

pub fn analysis_batch_query_from_game(
    game: &GameDto,
    options: AnalysisBatchQueryOptions,
) -> Result<AnalysisQuery, ProtocolError> {
    let board_size = game.summary.board_size;
    let move_count = game.moves.len() as u32;
    let moves = game
        .moves
        .iter()
        .map(|move_| move_dto_to_kata_move(move_, board_size))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(AnalysisQuery {
        id: options.id,
        moves,
        initial_stones: Vec::new(),
        rules: options.rules,
        komi: game.summary.komi,
        board_x_size: board_size,
        board_y_size: board_size,
        analyze_turns: Some(normalize_analysis_turns(options.analyze_turns, move_count)),
        max_visits: options.max_visits,
        include_ownership: options.include_ownership,
        include_policy: options.include_policy,
        report_during_search_every: None,
        override_settings: None,
    })
}

fn normalize_analysis_turns(turns: Option<Vec<u32>>, move_count: u32) -> Vec<u32> {
    let mut turns = turns.unwrap_or_else(|| (0..=move_count).collect());
    for turn in &mut turns {
        *turn = (*turn).min(move_count);
    }
    turns.sort_unstable();
    turns.dedup();
    turns
}

pub fn move_dto_to_kata_move(move_: &MoveDto, board_size: u8) -> Result<KataMove, ProtocolError> {
    Ok((
        player_color_to_kata(move_.color).to_string(),
        move_vertex_to_kata_coordinate(&move_.vertex, board_size)?,
    ))
}

pub fn player_color_to_kata(color: PlayerColor) -> &'static str {
    match color {
        PlayerColor::Black => "B",
        PlayerColor::White => "W",
    }
}

pub fn move_vertex_to_kata_coordinate(vertex: &MoveVertex, board_size: u8) -> Result<String, ProtocolError> {
    match vertex {
        MoveVertex::Pass => Ok("pass".to_string()),
        MoveVertex::Point(point) => point_to_kata_coordinate(point, board_size),
    }
}

pub fn point_to_kata_coordinate(point: &PointDto, board_size: u8) -> Result<String, ProtocolError> {
    if point.x >= board_size || point.y >= board_size {
        return Err(ProtocolError::InvalidVertex {
            x: point.x,
            y: point.y,
            board_size,
        });
    }
    let col = if point.x >= 8 {
        (b'A' + point.x + 1) as char
    } else {
        (b'A' + point.x) as char
    };
    let row = board_size - point.y;
    Ok(format!("{col}{row}"))
}

pub fn normalize_responses_for_turns(
    job_id: AnalysisJobId,
    responses: Vec<AnalysisResponse>,
    board_size: u8,
    turns: &[u32],
) -> Vec<AnalysisFrameDto> {
    let mut turns = turns.to_vec();
    turns.sort_unstable();
    turns.dedup();

    let mut frames = responses
        .into_iter()
        .filter(|response| turns.binary_search(&response.turn_number).is_ok())
        .map(|response| normalize_response(job_id, response, board_size))
        .collect::<Vec<_>>();
    frames.sort_by_key(|frame| frame.turn);
    frames
}

pub fn normalize_response(
    job_id: AnalysisJobId,
    response: AnalysisResponse,
    board_size: u8,
) -> AnalysisFrameDto {
    let root = response.root_info.unwrap_or(RootInfo {
        visits: 0,
        winrate: 0.5,
        score_mean: 0.0,
        score_lead: None,
        score_stdev: None,
    });
    AnalysisFrameDto {
        job_id,
        game_id: None,
        node_id: None,
        turn: response.turn_number,
        visits: root.visits,
        winrate_black: root.winrate,
        score_mean_black: root.score_lead.unwrap_or(root.score_mean),
        score_stdev: root.score_stdev,
        candidates: response
            .move_infos
            .into_iter()
            .map(|info| CandidateMoveDto {
                vertex: info
                    .move_
                    .as_deref()
                    .map(|m| gtp_vertex_to_dto(m, board_size))
                    .unwrap_or(MoveVertex::Pass),
                visits: info.visits,
                winrate_black: info.winrate,
                score_mean_black: info.score_mean,
                policy_prior: info.prior,
                pv: info.pv.iter().map(|m| gtp_vertex_to_dto(m, board_size)).collect(),
            })
            .collect(),
        ownership: response.ownership,
        policy: response.policy,
    }
}

pub fn gtp_vertex_to_dto(vertex: &str, board_size: u8) -> MoveVertex {
    if vertex.eq_ignore_ascii_case("pass") || vertex.is_empty() {
        return MoveVertex::Pass;
    }
    let mut chars = vertex.chars();
    let Some(col) = chars.next() else {
        return MoveVertex::Pass;
    };
    let row: String = chars.collect();
    let Ok(row_num) = row.parse::<u8>() else {
        return MoveVertex::Pass;
    };
    let col_upper = col.to_ascii_uppercase();
    let skipped_i = if col_upper > 'I' { 1 } else { 0 };
    let x = (col_upper as u8).saturating_sub(b'A').saturating_sub(skipped_i);
    let y = board_size.saturating_sub(row_num);
    if x >= board_size || y >= board_size {
        MoveVertex::Pass
    } else {
        MoveVertex::Point(PointDto { x, y })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use app_model::{GameId, GameSummaryDto};

    fn move_at(color: PlayerColor, x: u8, y: u8, move_number: u32) -> MoveDto {
        MoveDto {
            color,
            vertex: MoveVertex::Point(PointDto { x, y }),
            move_number,
        }
    }

    fn pass(color: PlayerColor, move_number: u32) -> MoveDto {
        MoveDto {
            color,
            vertex: MoveVertex::Pass,
            move_number,
        }
    }

    fn game(moves: Vec<MoveDto>) -> GameDto {
        GameDto {
            summary: GameSummaryDto {
                id: GameId::nil(),
                board_size: 19,
                komi: 7.5,
                black_name: None,
                white_name: None,
                result: None,
                move_count: moves.len(),
            },
            moves,
        }
    }

    fn options(turn: u32) -> AnalysisQueryOptions {
        AnalysisQueryOptions {
            id: "query-1".to_string(),
            rules: "chinese".to_string(),
            turn,
            max_visits: Some(128),
            include_ownership: Some(true),
            include_policy: Some(false),
        }
    }

    fn batch_options(analyze_turns: Option<Vec<u32>>) -> AnalysisBatchQueryOptions {
        AnalysisBatchQueryOptions {
            id: "batch-1".to_string(),
            rules: "chinese".to_string(),
            analyze_turns,
            max_visits: Some(256),
            include_ownership: Some(false),
            include_policy: Some(true),
        }
    }

    fn response(turn_number: u32, visits: u32) -> AnalysisResponse {
        AnalysisResponse {
            id: "batch-1".to_string(),
            turn_number,
            root_info: Some(RootInfo {
                visits,
                winrate: 0.5,
                score_mean: 0.0,
                score_lead: None,
                score_stdev: None,
            }),
            move_infos: Vec::new(),
            ownership: None,
            policy: None,
            error: None,
            warning: None,
            is_during_search: Some(false),
            no_results: false,
            action: None,
            field: None,
        }
    }

    #[test]
    fn parse_response_line_returns_engine_error_field() {
        let error = parse_response_line(r#"{"id":"query-1","error":"bad query"}"#).unwrap_err();

        match error {
            ProtocolError::Engine(message) => assert_eq!(message, "bad query"),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn parse_response_line_accepts_success_response_without_error_field() {
        let response = parse_response_line(
            r#"{"id":"query-1","turnNumber":2,"rootInfo":{"visits":64,"winrate":0.52,"scoreMean":1.5}}"#,
        )
        .unwrap();

        assert_eq!(response.id, "query-1");
        assert_eq!(response.turn_number, 2);
        assert_eq!(response.root_info.unwrap().visits, 64);
    }

    #[test]
    fn converts_pass_to_katago_pass() {
        let move_ = pass(PlayerColor::Black, 1);
        let kata_move = move_dto_to_kata_move(&move_, 19).unwrap();

        assert_eq!(kata_move, ("B".to_string(), "pass".to_string()));
    }

    #[test]
    fn skips_i_column_in_katago_coordinates() {
        let coordinate = point_to_kata_coordinate(&PointDto { x: 8, y: 9 }, 19).unwrap();

        assert_eq!(coordinate, "J10");
    }

    #[test]
    fn converts_19_line_board_edges() {
        let top_left = point_to_kata_coordinate(&PointDto { x: 0, y: 0 }, 19).unwrap();
        let bottom_right = point_to_kata_coordinate(&PointDto { x: 18, y: 18 }, 19).unwrap();

        assert_eq!(top_left, "A19");
        assert_eq!(bottom_right, "T1");
    }

    #[test]
    fn query_for_turn_truncates_moves_and_sets_analyze_turn() {
        let game = game(vec![
            move_at(PlayerColor::Black, 3, 15, 1),
            move_at(PlayerColor::White, 15, 3, 2),
            pass(PlayerColor::Black, 3),
        ]);

        let query = analysis_query_from_game(&game, options(2)).unwrap();

        assert_eq!(
            query.moves,
            vec![
                ("B".to_string(), "D4".to_string()),
                ("W".to_string(), "Q16".to_string())
            ]
        );
        assert_eq!(query.analyze_turns, Some(vec![2]));
        assert_eq!(query.max_visits, Some(128));
        assert_eq!(query.include_ownership, Some(true));
        assert_eq!(query.include_policy, Some(false));
    }

    #[test]
    fn batch_query_defaults_to_every_turn_and_full_main_line() {
        let game = game(vec![
            move_at(PlayerColor::Black, 3, 15, 1),
            move_at(PlayerColor::White, 15, 3, 2),
            pass(PlayerColor::Black, 3),
        ]);

        let query = analysis_batch_query_from_game(&game, batch_options(None)).unwrap();

        assert_eq!(
            query.moves,
            vec![
                ("B".to_string(), "D4".to_string()),
                ("W".to_string(), "Q16".to_string()),
                ("B".to_string(), "pass".to_string())
            ]
        );
        assert_eq!(query.analyze_turns, Some(vec![0, 1, 2, 3]));
        assert_eq!(query.max_visits, Some(256));
        assert_eq!(query.include_ownership, Some(false));
        assert_eq!(query.include_policy, Some(true));
    }

    #[test]
    fn batch_query_normalizes_custom_turns() {
        let game = game(vec![
            move_at(PlayerColor::Black, 3, 15, 1),
            move_at(PlayerColor::White, 15, 3, 2),
            pass(PlayerColor::Black, 3),
        ]);

        let query = analysis_batch_query_from_game(&game, batch_options(Some(vec![3, 1, 99, 1, 0]))).unwrap();

        assert_eq!(query.analyze_turns, Some(vec![0, 1, 3]));
    }

    #[test]
    fn normalizes_responses_for_requested_turns_sorted_by_turn() {
        let job_id = AnalysisJobId::nil();
        let frames = normalize_responses_for_turns(
            job_id,
            vec![response(3, 30), response(1, 10), response(2, 20)],
            19,
            &[3, 1, 3],
        );

        assert_eq!(
            frames.iter().map(|frame| frame.turn).collect::<Vec<_>>(),
            vec![1, 3]
        );
        assert_eq!(
            frames.iter().map(|frame| frame.visits).collect::<Vec<_>>(),
            vec![10, 30]
        );
    }

    #[test]
    fn normalizes_responses_filters_unrequested_turns_without_collapsing_duplicate_responses() {
        let job_id = AnalysisJobId::nil();
        let frames = normalize_responses_for_turns(
            job_id,
            vec![response(1, 10), response(2, 20), response(1, 11)],
            19,
            &[1, 1],
        );

        assert_eq!(
            frames.iter().map(|frame| frame.turn).collect::<Vec<_>>(),
            vec![1, 1]
        );
        assert_eq!(
            frames.iter().map(|frame| frame.visits).collect::<Vec<_>>(),
            vec![10, 11]
        );
    }

    #[test]
    fn normalizes_response_ownership_and_policy_arrays() {
        let job_id = AnalysisJobId::nil();
        let response = parse_response_line(
            r#"{"id":"query-1","turnNumber":2,"rootInfo":{"visits":64,"winrate":0.52,"scoreMean":1.5},"ownership":[0.1,-0.2,0.3],"policy":[0.01,0.02,0.03]}"#,
        )
        .unwrap();

        let frame = normalize_response(job_id, response, 19);

        assert_eq!(frame.ownership, Some(vec![0.1, -0.2, 0.3]));
        assert_eq!(frame.policy, Some(vec![0.01, 0.02, 0.03]));
    }

    #[test]
    fn terminate_action_jsonl_uses_protocol_id_and_action() {
        let line = terminate_action_jsonl("control-42", "job-42");
        assert!(line.ends_with('\n'));
        let value: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
        assert_eq!(value["id"], "control-42");
        assert_eq!(value["terminateId"], "job-42");
        assert_eq!(value["action"], "terminate");
    }

    #[test]
    fn analysis_query_from_position_uses_initial_stones_and_white_to_play_pass() {
        let stones = vec![app_model::StoneDto {
            x: 3,
            y: 3,
            color: PlayerColor::Black,
        }];
        let query = analysis_query_from_position(
            9,
            6.5,
            &stones,
            PlayerColor::White,
            AnalysisQueryOptions {
                id: "job-1".into(),
                rules: "chinese".into(),
                turn: 0,
                max_visits: Some(8),
                include_ownership: Some(true),
                include_policy: Some(true),
            },
        )
        .unwrap();
        assert_eq!(query.id, "job-1");
        assert_eq!(query.komi, 6.5);
        assert_eq!(query.board_x_size, 9);
        assert_eq!(query.initial_stones, vec![("B".to_string(), "D6".to_string())]);
        assert_eq!(query.moves, vec![("B".to_string(), "pass".to_string())]);
        assert_eq!(query.analyze_turns, Some(vec![1]));
    }
}
