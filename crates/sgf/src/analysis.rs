use app_model::{AnalysisFrameDto, CandidateMoveDto, MoveVertex, PointDto};
use uuid::Uuid;

use crate::{property_values, SgfNode, SgfProperty};

#[derive(Debug, Clone, PartialEq)]
pub struct SgfAnalysisPayload {
    pub engine_name: String,
    pub visits: u32,
    pub winrate_black: f32,
    pub score_mean_black: Option<f32>,
    pub score_stdev: Option<f32>,
    pub pda: Option<f32>,
    pub candidates: Vec<CandidateMoveDto>,
    pub ownership: Option<Vec<f32>>,
}

impl SgfAnalysisPayload {
    pub fn is_projectable(&self) -> bool {
        self.visits > 0 && !self.candidates.is_empty()
    }

    pub fn from_frame(frame: &AnalysisFrameDto, engine_name: impl Into<String>) -> Self {
        Self {
            engine_name: engine_name.into(),
            visits: frame.visits,
            winrate_black: frame.winrate_black,
            score_mean_black: frame.score_mean_black,
            score_stdev: frame.score_stdev,
            pda: None,
            candidates: frame.candidates.clone(),
            ownership: frame.ownership.clone(),
        }
    }

    pub fn to_frame(&self, turn: u32) -> AnalysisFrameDto {
        AnalysisFrameDto {
            job_id: Uuid::nil(),
            game_id: None,
            node_id: None,
            turn,
            visits: self.visits,
            winrate_black: self.winrate_black,
            score_mean_black: self.score_mean_black,
            score_stdev: self.score_stdev,
            candidates: self.candidates.clone(),
            ownership: self.ownership.clone(),
            policy: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NodeAnalysisProjection {
    pub primary: Option<SgfAnalysisPayload>,
    pub secondary: Option<SgfAnalysisPayload>,
}

pub fn project_node_analysis(node: &SgfNode, board_size: u8, is_root: bool) -> NodeAnalysisProjection {
    let primary_keys = if is_root { ["LZOP", "LZ"] } else { ["LZ", "LZOP"] };
    let secondary_keys = if is_root {
        ["LZOP2", "LZ2"]
    } else {
        ["LZ2", "LZOP2"]
    };
    NodeAnalysisProjection {
        primary: first_projectable(node, &primary_keys, board_size),
        secondary: first_projectable(node, &secondary_keys, board_size),
    }
}

fn first_projectable(node: &SgfNode, keys: &[&str], board_size: u8) -> Option<SgfAnalysisPayload> {
    for key in keys {
        let Some(values) = property_values(node, key) else {
            continue;
        };
        let Some(raw) = values.first() else {
            continue;
        };
        let Some(payload) = parse_analysis_payload(raw, board_size) else {
            continue;
        };
        if payload.is_projectable() {
            return Some(payload);
        }
    }
    None
}

pub fn parse_analysis_payload(raw: &str, board_size: u8) -> Option<SgfAnalysisPayload> {
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }
    let (header_line, detail_line) = match raw.split_once('\n') {
        Some((header, detail)) => (header.trim(), detail.trim()),
        None => (raw, ""),
    };
    let header: Vec<&str> = header_line.split_whitespace().collect();
    if header.len() < 3 || header[0].is_empty() {
        return None;
    }
    let engine_name = header[0].to_string();
    let white_winrate = parse_f32(header[1]).unwrap_or(50.0);
    let visits = parse_playouts(header[2]);
    let score_mean_black = header.get(3).and_then(|value| parse_f32(value));
    let score_stdev = header.get(4).and_then(|value| parse_f32(value));
    let pda = header.get(5).and_then(|value| parse_f32(value));
    let (analysis_line, ownership) = split_ownership(detail_line);
    let candidates = parse_candidates(analysis_line, board_size, score_mean_black);
    Some(SgfAnalysisPayload {
        engine_name,
        visits,
        winrate_black: (100.0 - white_winrate) / 100.0,
        score_mean_black,
        score_stdev,
        pda,
        candidates,
        ownership,
    })
}

fn split_ownership(detail_line: &str) -> (&str, Option<Vec<f32>>) {
    let Some((analysis, ownership_raw)) = detail_line.split_once("ownership") else {
        return (detail_line, None);
    };
    let mut values = Vec::new();
    for token in ownership_raw.split_whitespace() {
        let Ok(value) = token.parse::<f32>() else {
            break;
        };
        values.push(value);
    }
    (analysis, if values.is_empty() { None } else { Some(values) })
}

fn parse_candidates(
    analysis_line: &str,
    board_size: u8,
    header_score_mean: Option<f32>,
) -> Vec<CandidateMoveDto> {
    analysis_line
        .split(" info ")
        .filter_map(|variation| parse_candidate(variation, board_size, header_score_mean))
        .collect()
}

fn parse_candidate(
    variation: &str,
    board_size: u8,
    header_score_mean: Option<f32>,
) -> Option<CandidateMoveDto> {
    let tokens: Vec<&str> = variation.split_whitespace().collect();
    if tokens.is_empty() {
        return None;
    }
    let mut coordinate = None;
    let mut visits = 0;
    let mut winrate_black = 0.0;
    let mut policy_prior = None;
    let mut score_mean_black = header_score_mean.unwrap_or(0.0);
    let mut pv = Vec::new();
    let mut index = 0;
    while index + 1 < tokens.len() {
        let key = tokens[index];
        if key == "pv" {
            let pv_end = tokens[index + 1..]
                .iter()
                .position(|token| *token == "pvVisits")
                .map(|offset| index + 1 + offset)
                .unwrap_or(tokens.len());
            pv = tokens[index + 1..pv_end]
                .iter()
                .map(|token| gtp_vertex(token, board_size))
                .collect();
            break;
        }
        let value = tokens[index + 1];
        match key {
            "move" => coordinate = Some(gtp_vertex(value, board_size)),
            "visits" => visits = parse_playouts(value),
            "winrate" => winrate_black = parse_playouts(value) as f32 / 10_000.0,
            "prior" => policy_prior = Some(parse_prior(value)),
            "scoreMean" => score_mean_black = parse_f32(value).unwrap_or(score_mean_black),
            _ => {}
        }
        index += 2;
    }
    let vertex = coordinate?;
    if pv.is_empty() && !matches!(vertex, MoveVertex::Pass) {
        pv.push(vertex.clone());
    }
    Some(CandidateMoveDto {
        vertex,
        visits,
        winrate_black,
        score_mean_black,
        policy_prior,
        pv,
    })
}

fn parse_prior(raw: &str) -> f32 {
    if let Ok(value) = raw.parse::<i32>() {
        return value as f32 / 10_000.0;
    }
    parse_f32(raw).unwrap_or(0.0)
}

fn parse_playouts(raw: &str) -> u32 {
    let normalized = raw.trim().to_ascii_lowercase().replace(',', "");
    if normalized.is_empty() {
        return 0;
    }
    let (number, multiplier) = if let Some(prefix) = normalized.strip_suffix('m') {
        (prefix, 1_000_000.0)
    } else if let Some(prefix) = normalized.strip_suffix('k') {
        (prefix, 1_000.0)
    } else {
        (normalized.as_str(), 1.0)
    };
    let cleaned: String = number
        .chars()
        .filter(|ch| ch.is_ascii_digit() || *ch == '.' || *ch == '-')
        .collect();
    if let Ok(value) = cleaned.parse::<f64>() {
        if value <= 0.0 {
            return 0;
        }
        return (value * multiplier).round() as u32;
    }
    cleaned
        .chars()
        .filter(|ch| ch.is_ascii_digit())
        .collect::<String>()
        .parse()
        .unwrap_or(0)
}

fn parse_f32(raw: &str) -> Option<f32> {
    raw.parse().ok()
}

fn gtp_vertex(vertex: &str, board_size: u8) -> MoveVertex {
    if vertex.eq_ignore_ascii_case("pass") || vertex.is_empty() {
        return MoveVertex::Pass;
    }
    let mut chars = vertex.chars();
    let Some(col) = chars.next() else {
        return MoveVertex::Pass;
    };
    let Ok(row_num) = chars.as_str().parse::<u8>() else {
        return MoveVertex::Pass;
    };
    let col_upper = col.to_ascii_uppercase();
    let skipped_i = u8::from(col_upper > 'I');
    let x = (col_upper as u8).saturating_sub(b'A').saturating_sub(skipped_i);
    let y = board_size.saturating_sub(row_num);
    if x >= board_size || y >= board_size {
        MoveVertex::Pass
    } else {
        MoveVertex::Point(PointDto { x, y })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnalysisSlot {
    Primary { root: bool },
    Secondary { root: bool },
}

impl AnalysisSlot {
    fn property_key(self) -> &'static str {
        match self {
            Self::Primary { root: true } => "LZOP",
            Self::Primary { root: false } => "LZ",
            Self::Secondary { root: true } => "LZOP2",
            Self::Secondary { root: false } => "LZ2",
        }
    }
}

pub fn encode_analysis_payload(
    payload: &SgfAnalysisPayload,
    slot: AnalysisSlot,
    board_size: u8,
) -> SgfProperty {
    SgfProperty {
        key: slot.property_key().to_string(),
        values: vec![format_analysis_payload(payload, board_size)],
    }
}

const PRIMARY_PROPERTY_KEYS: [&str; 2] = ["LZ", "LZOP"];

pub fn replace_primary_analysis(
    node: &mut SgfNode,
    payload: &SgfAnalysisPayload,
    board_size: u8,
    is_root: bool,
) -> bool {
    if !payload.is_projectable() {
        return false;
    }
    let encoded = encode_analysis_payload(payload, AnalysisSlot::Primary { root: is_root }, board_size);
    let primary_count = node
        .properties
        .iter()
        .filter(|property| PRIMARY_PROPERTY_KEYS.contains(&property.key.as_str()))
        .count();
    let unchanged = primary_count == 1
        && node.properties.iter().any(|property| {
            PRIMARY_PROPERTY_KEYS.contains(&property.key.as_str())
                && property.key == encoded.key
                && property.values == encoded.values
        });
    if unchanged {
        return false;
    }
    let mut wrote = false;
    node.properties.retain_mut(|property| {
        if !PRIMARY_PROPERTY_KEYS.contains(&property.key.as_str()) {
            return true;
        }
        if wrote {
            return false;
        }
        *property = encoded.clone();
        wrote = true;
        true
    });
    if !wrote {
        node.properties.push(encoded);
    }
    true
}

fn format_analysis_payload(payload: &SgfAnalysisPayload, board_size: u8) -> String {
    let white_winrate = (1.0 - payload.winrate_black) * 100.0;
    let mut header = format!(
        "{} {:.1} {}",
        payload.engine_name,
        white_winrate,
        format_playouts(payload.visits)
    );
    if let Some(score_mean) = payload.score_mean_black {
        header.push(' ');
        header.push_str(&format_analysis_scalar(score_mean));
        if let Some(score_stdev) = payload.score_stdev {
            header.push(' ');
            header.push_str(&format_analysis_scalar(score_stdev));
            if let Some(pda) = payload.pda {
                header.push(' ');
                header.push_str(&format_analysis_scalar(pda));
            }
        }
    }
    let detail = format_detail_line(payload, board_size);
    if detail.is_empty() {
        header
    } else {
        format!("{header}\n{detail}")
    }
}

fn format_detail_line(payload: &SgfAnalysisPayload, board_size: u8) -> String {
    let mut detail = payload
        .candidates
        .iter()
        .map(|candidate| format_candidate(candidate, board_size))
        .collect::<Vec<_>>()
        .join(" info ");
    if let Some(ownership) = &payload.ownership {
        if !ownership.is_empty() {
            if !detail.is_empty() {
                detail.push(' ');
            }
            detail.push_str("ownership");
            for value in ownership {
                detail.push(' ');
                detail.push_str(&value.to_string());
            }
        }
    }
    detail
}

fn format_candidate(candidate: &CandidateMoveDto, board_size: u8) -> String {
    let mut body = format!(
        "move {} visits {} winrate {} prior {}",
        vertex_to_gtp(&candidate.vertex, board_size),
        candidate.visits,
        (candidate.winrate_black * 10_000.0).round() as i32,
        (candidate.policy_prior.unwrap_or(0.0) * 10_000.0).round() as i32
    );
    body.push_str(&format!(" scoreMean {:.2}", candidate.score_mean_black));
    body.push_str(" pv");
    let pv = if candidate.pv.is_empty() {
        vec![candidate.vertex.clone()]
    } else {
        candidate.pv.clone()
    };
    for vertex in pv {
        body.push(' ');
        body.push_str(&vertex_to_gtp(&vertex, board_size));
    }
    body
}

fn format_playouts(playouts: u32) -> String {
    if playouts >= 10_000_000 {
        format!("{}m", ((f64::from(playouts) / 100_000.0).round() / 10.0))
    } else if playouts >= 9950 {
        format!("{}k", (f64::from(playouts) / 1_000.0).round())
    } else if playouts >= 1_000 {
        format!("{}k", ((f64::from(playouts) / 100.0).round() / 10.0))
    } else {
        playouts.to_string()
    }
}

fn format_analysis_scalar(value: f32) -> String {
    let rendered = value.to_string();
    if rendered.contains('.') {
        rendered
    } else {
        format!("{value}.0")
    }
}

fn vertex_to_gtp(vertex: &MoveVertex, board_size: u8) -> String {
    match vertex {
        MoveVertex::Pass => "pass".to_string(),
        MoveVertex::Point(point) => {
            let col_index = if point.x >= 8 { point.x + 1 } else { point.x };
            let col = char::from(b'A' + col_index);
            let row = board_size.saturating_sub(point.y);
            format!("{col}{row}")
        }
    }
}
