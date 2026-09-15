use app_model::{MoveVertex, PlayerColor, PointDto};
use encoding_rs::GB18030;
use thiserror::Error;

use crate::{
    chain_sequence, serialize_sgf_document, serialize_vertex, CurrentSgfDocument, SgfDocument, SgfNode,
    SgfProperty,
};

const BOARD_SIZE: u8 = 19;
const DEFAULT_KOMI: f32 = 7.5;
const HANDICAP_PLACEMENTS: &[(u8, u8)] = &[
    (3, 15),
    (15, 3),
    (15, 15),
    (3, 3),
    (3, 9),
    (15, 9),
    (9, 3),
    (9, 15),
    (9, 9),
];

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum GibError {
    #[error("GIB is empty")]
    Empty,
    #[error("GIB text encoding is invalid")]
    InvalidEncoding,
    #[error("unsupported or malformed GIB")]
    Malformed,
}

#[derive(Debug)]
enum GibAction {
    Move { color: PlayerColor, point: PointDto },
    Pass,
}

pub fn import_gib(input: &[u8]) -> Result<String, GibError> {
    let text = decode_gib(input)?;
    let document = parse_gib_text(&text)?;
    let sgf = serialize_sgf_document(&document).map_err(|_| GibError::Malformed)?;
    let validated = CurrentSgfDocument::open(&sgf).map_err(|_| GibError::Malformed)?;
    let snapshots = validated
        .first_child_mainline_snapshots()
        .map_err(|_| GibError::Malformed)?;
    if snapshots
        .iter()
        .any(|snapshot| !snapshot.position.errors.is_empty())
    {
        return Err(GibError::Malformed);
    }
    Ok(sgf)
}

fn decode_gib(input: &[u8]) -> Result<String, GibError> {
    if input.is_empty() {
        return Err(GibError::Empty);
    }
    if let Ok(text) = std::str::from_utf8(input) {
        let text = text.trim_start_matches('\u{feff}');
        if text.trim().is_empty() {
            return Err(GibError::Empty);
        }
        return Ok(text.to_string());
    }
    let (decoded, had_errors) = GB18030.decode_without_bom_handling(input);
    if had_errors {
        return Err(GibError::InvalidEncoding);
    }
    if decoded.trim().is_empty() {
        return Err(GibError::Empty);
    }
    Ok(decoded.into_owned())
}

fn parse_gib_text(text: &str) -> Result<SgfDocument, GibError> {
    let lines: Vec<&str> = text.lines().map(|line| line.trim_end_matches('\r')).collect();
    let mut black_name = "Player 2".to_string();
    let mut white_name = "Player 1".to_string();
    let mut komi = DEFAULT_KOMI;
    let mut handicap = 0_u8;
    let mut recognized = false;

    for line in &lines {
        if let Some(value) = metadata_value(line, "GAMEBLACKNAME")? {
            black_name = value.to_string();
            recognized = true;
        } else if let Some(value) = metadata_value(line, "GAMEWHITENAME")? {
            white_name = value.to_string();
            recognized = true;
        } else if line.starts_with(r"\[GAMEINFOMAIN=") {
            recognized = true;
            if line.contains("GONGJE:") {
                let value = field_value(line, "GONGJE:").ok_or(GibError::Malformed)?;
                komi = value.parse::<i32>().map_err(|_| GibError::Malformed)? as f32 / 10.0;
            }
        } else if first_field(line) == Some("INI") {
            let fields: Vec<&str> = line.split_whitespace().collect();
            handicap = fields
                .get(3)
                .ok_or(GibError::Malformed)?
                .parse::<u8>()
                .map_err(|_| GibError::Malformed)?
                .min(9);
            recognized = true;
        }
    }

    let mut actions = Vec::new();
    for line in &lines {
        match first_field(line) {
            Some("STO") => {
                let fields: Vec<&str> = line.split_whitespace().collect();
                let color = match fields.get(3).copied() {
                    Some("1") => PlayerColor::Black,
                    Some("2") => PlayerColor::White,
                    _ => return Err(GibError::Malformed),
                };
                let x = parse_coordinate(fields.get(4).copied())?;
                let y = parse_coordinate(fields.get(5).copied())?;
                actions.push(GibAction::Move {
                    color,
                    point: PointDto { x, y },
                });
                recognized = true;
            }
            Some("SKI") => {
                if line.split_whitespace().count() < 2 {
                    return Err(GibError::Malformed);
                }
                actions.push(GibAction::Pass);
                recognized = true;
            }
            _ => {}
        }
    }

    if !recognized {
        return Err(GibError::Malformed);
    }

    let mut root_properties = vec![
        property("GM", "1"),
        property("FF", "4"),
        property("CA", "UTF-8"),
        property("SZ", "19"),
        property("KM", &komi.to_string()),
        property("PB", &black_name),
        property("PW", &white_name),
    ];
    if handicap >= 2 {
        root_properties.push(property("HA", &handicap.to_string()));
        root_properties.push(property("PL", "W"));
        root_properties.push(SgfProperty {
            key: "AB".to_string(),
            values: handicap_points(handicap)
                .into_iter()
                .map(|point| serialize_point(point).expect("fixed handicap point"))
                .collect(),
        });
    }

    let mut sequence = vec![SgfNode {
        properties: root_properties,
        children: Vec::new(),
    }];
    let mut next_color = if handicap >= 2 {
        PlayerColor::White
    } else {
        PlayerColor::Black
    };
    for action in actions {
        let (color, vertex) = match action {
            GibAction::Move { color, point } => (color, MoveVertex::Point(point)),
            GibAction::Pass => (next_color, MoveVertex::Pass),
        };
        next_color = opposite(color);
        sequence.push(SgfNode {
            properties: vec![property(
                match color {
                    PlayerColor::Black => "B",
                    PlayerColor::White => "W",
                },
                &serialize_vertex(&vertex, BOARD_SIZE).map_err(|_| GibError::Malformed)?,
            )],
            children: Vec::new(),
        });
    }

    Ok(SgfDocument {
        board_size: BOARD_SIZE,
        komi,
        handicap: (handicap >= 2).then_some(handicap),
        black_name: Some(black_name),
        white_name: Some(white_name),
        result: None,
        moves: Vec::new(),
        root: Some(chain_sequence(sequence)),
    })
}

fn metadata_value<'a>(line: &'a str, key: &str) -> Result<Option<&'a str>, GibError> {
    let prefix = format!(r"\[{key}=");
    let Some(value) = line.strip_prefix(&prefix) else {
        return Ok(None);
    };
    let value = value.strip_suffix(r"\]").ok_or(GibError::Malformed)?;
    if value.trim().is_empty() {
        return Err(GibError::Malformed);
    }
    Ok(Some(value))
}

fn field_value<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    line.split_once(key)
        .and_then(|(_, rest)| rest.split_once(',').map(|(value, _)| value))
}

fn first_field(line: &str) -> Option<&str> {
    line.split_whitespace().next()
}

fn parse_coordinate(value: Option<&str>) -> Result<u8, GibError> {
    let coordinate = value
        .ok_or(GibError::Malformed)?
        .parse::<u8>()
        .map_err(|_| GibError::Malformed)?;
    (coordinate < BOARD_SIZE)
        .then_some(coordinate)
        .ok_or(GibError::Malformed)
}

fn handicap_points(handicap: u8) -> Vec<PointDto> {
    let mut points = Vec::with_capacity(handicap as usize);
    let count = if handicap == 5 || handicap == 7 {
        points.push(PointDto { x: 9, y: 9 });
        handicap - 1
    } else {
        handicap
    };
    points.extend(
        HANDICAP_PLACEMENTS[..count as usize]
            .iter()
            .map(|&(x, y)| PointDto { x, y }),
    );
    points
}

fn serialize_point(point: PointDto) -> Result<String, GibError> {
    serialize_vertex(&MoveVertex::Point(point), BOARD_SIZE).map_err(|_| GibError::Malformed)
}

fn property(key: &str, value: &str) -> SgfProperty {
    SgfProperty {
        key: key.to_string(),
        values: vec![value.to_string()],
    }
}

fn opposite(color: PlayerColor) -> PlayerColor {
    match color {
        PlayerColor::Black => PlayerColor::White,
        PlayerColor::White => PlayerColor::Black,
    }
}
