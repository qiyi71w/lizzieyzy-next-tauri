use app_model::{
    MoveDto, MoveVertex, PlayerColor, PointDto, PositionDto, ReadboardSidecarSyncSnapshotRequest,
    ReadboardSidecarSyncSnapshotResult, StoneDto,
};
use go_core::{Color, Point};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::PathBuf};
use thiserror::Error;

mod frame;
mod protocol;
mod runtime;
pub use frame::{FrameDecoder, ReadboardInbound};
pub use runtime::{ReadboardInboundEvent, ReadboardRuntime, ReadboardRuntimeEvent};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadboardWarningCode {
    UnsupportedProvider,
    IgnoredToken,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadboardSidecarWarning {
    pub code: ReadboardWarningCode,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<PathBuf>,
}

impl ReadboardSidecarWarning {
    fn new(code: ReadboardWarningCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            path: None,
        }
    }

    pub fn as_dto_string(&self) -> String {
        match &self.path {
            Some(path) => format!("{:?}: {} ({})", self.code, self.message, path.display()),
            None => format!("{:?}: {}", self.code, self.message),
        }
    }
}

/// Offline preview of one pasted snapshot line. It never decides history, turn recovery or
/// imports; live READ-02 frames go through `FrameDecoder` and the current-game owner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreviewSnapshot {
    pub board_size: u8,
    pub stones: Vec<Option<Color>>,
    pub last_move: Option<(Point, Color)>,
    pub remote_move_number: Option<u32>,
    pub fox: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedReadboardLine {
    pub snapshot_id: Option<String>,
    pub snapshot: PreviewSnapshot,
    pub warnings: Vec<ReadboardSidecarWarning>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ReadboardSidecarError {
    #[error("readboard protocol line is empty")]
    EmptyProtocolLine,
    #[error("readboard protocol field `{0}` is missing")]
    MissingField(&'static str),
    #[error("readboard protocol field `{field}` has invalid value `{value}`")]
    InvalidField { field: &'static str, value: String },
    #[error("readboard protocol field `{field}` is duplicated")]
    DuplicateField { field: String },
}

pub fn parse_snapshot_line(line: &str) -> Result<ParsedReadboardLine, ReadboardSidecarError> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return Err(ReadboardSidecarError::EmptyProtocolLine);
    }
    if is_legacy_code_payload(trimmed) {
        let board_size =
            infer_square_board_size(trimmed.len()).ok_or_else(|| ReadboardSidecarError::InvalidField {
                field: "codes",
                value: trimmed.to_string(),
            })?;
        return Ok(ParsedReadboardLine {
            snapshot_id: None,
            snapshot: preview_snapshot(board_size, &parse_code_chars(trimmed)?, None, false)?,
            warnings: Vec::new(),
        });
    }

    let fields = parse_key_values(trimmed)?;
    let codes = required_field(&fields, "codes")?;
    let board_size = optional_u8(&fields, "board_size")?
        .or_else(|| infer_square_board_size(codes.len()))
        .ok_or(ReadboardSidecarError::MissingField("board_size"))?;
    let fox = parse_provider(fields.get("provider").map(String::as_str))?;
    Ok(ParsedReadboardLine {
        snapshot_id: fields.get("snapshot_id").cloned(),
        snapshot: preview_snapshot(
            board_size,
            &parse_code_chars(codes)?,
            optional_u32(&fields, "move_number")?,
            fox,
        )?,
        warnings: ignored_field_warnings(&fields),
    })
}

pub fn preview_snapshot_line(
    request: &ReadboardSidecarSyncSnapshotRequest,
    protocol_line: &str,
) -> Result<ReadboardSidecarSyncSnapshotResult, ReadboardSidecarError> {
    let parsed = parse_snapshot_line(protocol_line)?;
    Ok(ReadboardSidecarSyncSnapshotResult {
        snapshot_id: request
            .snapshot_id
            .clone()
            .or(parsed.snapshot_id)
            .unwrap_or_else(|| "readboard-snapshot".to_string()),
        position: Some(snapshot_to_position(&parsed.snapshot)),
        warnings: parsed
            .warnings
            .iter()
            .map(ReadboardSidecarWarning::as_dto_string)
            .collect(),
    })
}

fn preview_snapshot(
    board_size: u8,
    codes: &[u8],
    remote_move_number: Option<u32>,
    fox: bool,
) -> Result<PreviewSnapshot, ReadboardSidecarError> {
    if !(2..=25).contains(&board_size) || codes.len() != board_size as usize * board_size as usize {
        return Err(ReadboardSidecarError::InvalidField {
            field: "codes",
            value: format!("{} codes for board_size {board_size}", codes.len()),
        });
    }
    let mut last_move = None;
    let stones = codes
        .iter()
        .enumerate()
        .map(|(index, code)| {
            let point = Point {
                x: (index % board_size as usize) as u8,
                y: (index / board_size as usize) as u8,
            };
            match code {
                1 => Some(Color::Black),
                2 => Some(Color::White),
                3 | 4 => {
                    let color = if *code == 3 { Color::Black } else { Color::White };
                    last_move = Some((point, color));
                    Some(color)
                }
                _ => None,
            }
        })
        .collect();
    Ok(PreviewSnapshot {
        board_size,
        stones,
        last_move,
        remote_move_number,
        fox,
    })
}

fn snapshot_to_position(snapshot: &PreviewSnapshot) -> PositionDto {
    let occupied = snapshot.stones.iter().filter(|stone| stone.is_some()).count() as u32;
    let move_number = snapshot.remote_move_number.unwrap_or(occupied);
    let to_play = if snapshot_black_to_play(snapshot, occupied) {
        PlayerColor::Black
    } else {
        PlayerColor::White
    };
    PositionDto {
        board_width: snapshot.board_size,
        board_height: snapshot.board_size,
        move_number,
        to_play,
        stones: snapshot
            .stones
            .iter()
            .enumerate()
            .filter_map(|(index, stone)| {
                stone.map(|color| StoneDto {
                    x: (index % snapshot.board_size as usize) as u8,
                    y: (index / snapshot.board_size as usize) as u8,
                    color: color_to_dto(color),
                })
            })
            .collect(),
        captures_black: 0,
        captures_white: 0,
        last_move: snapshot.last_move.map(|(point, color)| MoveDto {
            color: color_to_dto(color),
            vertex: MoveVertex::Point(PointDto {
                x: point.x,
                y: point.y,
            }),
            move_number,
        }),
        errors: Vec::new(),
    }
}

fn is_legacy_code_payload(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(|byte| matches!(byte, b'0'..=b'4'))
}

fn infer_square_board_size(len: usize) -> Option<u8> {
    let size = (len as f64).sqrt() as usize;
    (size * size == len && (2..=25).contains(&size)).then_some(size as u8)
}

fn parse_code_chars(value: &str) -> Result<Vec<u8>, ReadboardSidecarError> {
    value
        .chars()
        .map(|ch| {
            ch.to_digit(10)
                .filter(|digit| *digit <= 4)
                .map(|digit| digit as u8)
                .ok_or_else(|| ReadboardSidecarError::InvalidField {
                    field: "codes",
                    value: ch.to_string(),
                })
        })
        .collect()
}

fn parse_key_values(line: &str) -> Result<BTreeMap<String, String>, ReadboardSidecarError> {
    let mut fields = BTreeMap::new();
    let mut tokens = line.split_whitespace();
    if matches!(tokens.clone().next(), Some("snapshot" | "readboard_snapshot")) {
        tokens.next();
    }
    for token in tokens {
        let Some((key, value)) = token.split_once('=') else {
            return Err(ReadboardSidecarError::InvalidField {
                field: "token",
                value: token.to_string(),
            });
        };
        let normalized = key.trim().to_ascii_lowercase();
        if fields
            .insert(normalized.clone(), value.trim().to_string())
            .is_some()
        {
            return Err(ReadboardSidecarError::DuplicateField { field: normalized });
        }
    }
    Ok(fields)
}

fn required_field<'a>(
    fields: &'a BTreeMap<String, String>,
    field: &'static str,
) -> Result<&'a str, ReadboardSidecarError> {
    fields
        .get(field)
        .map(String::as_str)
        .ok_or(ReadboardSidecarError::MissingField(field))
}

fn optional_u8(
    fields: &BTreeMap<String, String>,
    field: &'static str,
) -> Result<Option<u8>, ReadboardSidecarError> {
    fields
        .get(field)
        .map(|value| {
            value.parse().map_err(|_| ReadboardSidecarError::InvalidField {
                field,
                value: value.clone(),
            })
        })
        .transpose()
}

fn optional_u32(
    fields: &BTreeMap<String, String>,
    field: &'static str,
) -> Result<Option<u32>, ReadboardSidecarError> {
    fields
        .get(field)
        .map(|value| {
            value.parse().map_err(|_| ReadboardSidecarError::InvalidField {
                field,
                value: value.clone(),
            })
        })
        .transpose()
}

fn parse_provider(raw: Option<&str>) -> Result<bool, ReadboardSidecarError> {
    match raw.unwrap_or("generic") {
        "generic" => Ok(false),
        "fox_live" | "fox-live" | "foxlive" | "fox_record" | "fox-record" | "foxrecord" => Ok(true),
        value => Err(ReadboardSidecarError::InvalidField {
            field: "provider",
            value: value.to_string(),
        }),
    }
}

fn ignored_field_warnings(fields: &BTreeMap<String, String>) -> Vec<ReadboardSidecarWarning> {
    fields
        .keys()
        .filter(|key| {
            !matches!(
                key.as_str(),
                "board_size" | "codes" | "move_number" | "provider" | "source" | "snapshot_id"
            )
        })
        .map(|key| {
            ReadboardSidecarWarning::new(
                ReadboardWarningCode::IgnoredToken,
                format!("ignored readboard protocol field `{key}`"),
            )
        })
        .collect()
}

fn snapshot_black_to_play(snapshot: &PreviewSnapshot, occupied: u32) -> bool {
    if let Some((_, color)) = snapshot.last_move {
        return color == Color::White;
    }
    if snapshot.fox {
        if let Some(move_number) = snapshot.remote_move_number {
            return move_number & 1 == 0;
        }
    }
    occupied & 1 == 0
}

fn color_to_dto(color: Color) -> PlayerColor {
    match color {
        Color::Black => PlayerColor::Black,
        Color::White => PlayerColor::White,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_legacy_codes_are_rejected() {
        let error =
            parse_snapshot_line("snapshot board_size=2 codes=0009").expect_err("code 9 should be rejected");

        assert!(matches!(
            error,
            ReadboardSidecarError::InvalidField { field: "codes", .. }
        ));
    }

    #[test]
    fn codes_must_fill_the_declared_board() {
        let error = parse_snapshot_line("snapshot board_size=3 codes=0000").unwrap_err();
        assert!(matches!(
            error,
            ReadboardSidecarError::InvalidField { field: "codes", .. }
        ));
    }

    #[test]
    fn parses_simple_snapshot_line_with_legacy_marker() {
        let parsed = parse_snapshot_line(
            "snapshot snapshot_id=s1 board_size=2 provider=fox_live source=room-1 move_number=1 codes=3000",
        )
        .unwrap();

        assert_eq!(parsed.snapshot_id, Some("s1".to_string()));
        assert_eq!(parsed.snapshot.board_size, 2);
        assert_eq!(parsed.snapshot.remote_move_number, Some(1));
        assert!(parsed.snapshot.fox);
        assert_eq!(
            parsed.snapshot.last_move,
            Some((Point { x: 0, y: 0 }, Color::Black))
        );
    }

    #[test]
    fn preview_reports_position_without_history_decision() {
        let request = ReadboardSidecarSyncSnapshotRequest {
            snapshot_id: Some("from-request".to_string()),
            ..Default::default()
        };

        let preview =
            preview_snapshot_line(&request, "snapshot board_size=2 move_number=1 codes=1000").unwrap();

        assert_eq!(preview.snapshot_id, "from-request");
        let position = preview.position.unwrap();
        assert_eq!(position.stones.len(), 1);
        assert_eq!(position.to_play, PlayerColor::White);
    }
}
