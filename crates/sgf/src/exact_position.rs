use std::collections::HashSet;

use app_model::{ExactPositionDto, ExactRulesDto, MoveDto, MoveVertex, PlayerColor};
use go_core::{Board, Color, Point};

use crate::{
    parse_player_to_play, parse_setup_points, parse_vertex, root_board_dimensions, stones_from_board,
    to_core_color, to_core_vertex, SgfNode, SgfProperty,
};

/// A selected SGF position whose setup, move history and complete rules can be reproduced exactly.
#[derive(Clone, Debug)]
pub struct ExactPosition {
    dto: ExactPositionDto,
    board: Board,
    history: RepetitionHistory,
}

#[derive(Clone, Debug)]
enum RepetitionHistory {
    Simple {
        current: Vec<Option<Color>>,
        previous: Option<Vec<Option<Color>>>,
    },
    Positional(HashSet<Vec<Option<Color>>>),
}

impl RepetitionHistory {
    fn new(rules: ExactRulesDto, stones: Vec<Option<Color>>) -> Self {
        match rules {
            ExactRulesDto::Chinese => Self::Simple {
                current: stones,
                previous: None,
            },
            ExactRulesDto::ChineseKgs => Self::Positional(HashSet::from([stones])),
        }
    }

    fn check(&self, stones: &[Option<Color>]) -> Result<(), String> {
        let repeats = match self {
            Self::Simple { previous, .. } => previous.as_deref() == Some(stones),
            Self::Positional(positions) => positions.contains(stones),
        };
        if repeats {
            Err("move repeats a position forbidden by the selected ko rule".into())
        } else {
            Ok(())
        }
    }

    fn record(&mut self, stones: Option<Vec<Option<Color>>>) {
        match self {
            Self::Simple { current, previous } => {
                *previous = Some(match stones {
                    Some(stones) => std::mem::replace(current, stones),
                    None => current.clone(),
                });
            }
            Self::Positional(positions) => {
                if let Some(stones) = stones {
                    positions.insert(stones);
                }
            }
        }
    }
}

impl ExactPosition {
    pub(crate) fn from_nodes(nodes: &[&SgfNode]) -> Result<Self, String> {
        let root = nodes.first().ok_or("exact position requires an SGF root")?;
        let (board_width, board_height) = root_board_dimensions(root).map_err(|error| error.to_string())?;
        if board_width != board_height || !(2..=19).contains(&board_width) {
            return Err("exact position supports square boards from 2 through 19 only".into());
        }
        let rules = match single_value(root, "RU")? {
            Some(value) if value.eq_ignore_ascii_case("Chinese") => ExactRulesDto::Chinese,
            Some(value) if value.eq_ignore_ascii_case("Chinese-KGS") => ExactRulesDto::ChineseKgs,
            _ => return Err("exact position requires explicit Chinese or Chinese-KGS rules".into()),
        };
        let komi = exact_komi(single_value(root, "KM")?.unwrap_or("7.5"))?;
        if unique_property(root, "AW")?.is_some() || unique_property(root, "AE")?.is_some() {
            return Err("exact position does not support white setup or removed setup stones".into());
        }
        let mut board = Board::new(board_width, board_height).map_err(|error| error.to_string())?;
        let setup = unique_property(root, "AB")?;
        if let Some(setup) = setup {
            if setup.values.is_empty() {
                return Err("black setup must contain stones".into());
            }
            for raw in &setup.values {
                for point in
                    parse_setup_points(raw, board_width, board_height).map_err(|error| error.to_string())?
                {
                    if board.get(point).map_err(|error| error.to_string())?.is_some() {
                        return Err("black setup contains duplicate stones".into());
                    }
                    board
                        .set_stone(point, Some(Color::Black))
                        .map_err(|error| error.to_string())?;
                }
            }
        }
        let initial_stones = stones_from_board(&board);
        let initial_player = single_value(root, "PL")?
            .map(parse_player_to_play)
            .transpose()
            .map_err(|error| error.to_string())?
            .unwrap_or(if setup.is_some() {
                PlayerColor::White
            } else {
                PlayerColor::Black
            });
        if setup.is_some() && (initial_stones.len() < 2 || initial_player != PlayerColor::White) {
            return Err("black handicap setup requires at least two stones and White to play".into());
        }
        if let Some(raw) = single_value(root, "HA")? {
            let handicap = raw.parse::<usize>().map_err(|_| "invalid handicap count")?;
            if handicap != initial_stones.len() {
                return Err("handicap count does not match actual black setup stones".into());
            }
        }
        let history = RepetitionHistory::new(rules, board.stones_snapshot());
        let mut position = Self {
            dto: ExactPositionDto {
                board_width,
                board_height,
                komi,
                rules,
                initial_player,
                to_play: initial_player,
                initial_stones,
                moves: Vec::new(),
            },
            board,
            history,
        };
        for (index, node) in nodes.iter().enumerate() {
            if index != 0 {
                for key in ["AB", "AW", "AE", "RU", "KM", "SZ", "HA"] {
                    if unique_property(node, key)?.is_some() {
                        return Err(format!("exact position does not support intermediate {key}"));
                    }
                }
                if let Some(raw) = single_value(node, "PL")? {
                    let player = parse_player_to_play(raw).map_err(|error| error.to_string())?;
                    if player != position.dto.to_play {
                        return Err("intermediate PL changes the side to play".into());
                    }
                }
            }
            let black = single_value(node, "B")?;
            let white = single_value(node, "W")?;
            let (color, raw) = match (black, white) {
                (None, None) => continue,
                (Some(raw), None) => (PlayerColor::Black, raw),
                (None, Some(raw)) => (PlayerColor::White, raw),
                (Some(_), Some(_)) => return Err("a node cannot contain both Black and White moves".into()),
            };
            if index == 0 && setup.is_some() {
                return Err("root setup cannot also contain an ordinary move".into());
            }
            if color != position.dto.to_play {
                return Err(format!(
                    "move at path depth {index} does not alternate from the initial player"
                ));
            }
            let vertex = parse_vertex(raw, board_width, board_height).map_err(|error| error.to_string())?;
            let stones = play_checked(&mut position.board, &position.history, color, &vertex)
                .map_err(|error| format!("illegal move at path depth {index}: {error}"))?;
            position.history.record(stones);
            let move_number = u32::try_from(position.dto.moves.len() + 1).map_err(|_| "too many moves")?;
            position.dto.moves.push(MoveDto {
                color,
                vertex,
                move_number,
            });
            position.dto.to_play = match color {
                PlayerColor::Black => PlayerColor::White,
                PlayerColor::White => PlayerColor::Black,
            };
        }
        Ok(position)
    }

    pub fn dto(&self) -> &ExactPositionDto {
        &self.dto
    }

    pub fn stones(&self) -> Vec<app_model::StoneDto> {
        stones_from_board(&self.board)
    }

    /// Checks an engine's point or pass without mutating the selected position.
    pub fn validate_move(&self, vertex: &MoveVertex) -> Result<(), String> {
        if matches!(vertex, MoveVertex::Pass) {
            return Ok(());
        }
        let mut board = self.board.clone();
        play_checked(&mut board, &self.history, self.dto.to_play, vertex).map(|_| ())
    }
}

fn play_checked(
    board: &mut Board,
    history: &RepetitionHistory,
    color: PlayerColor,
    vertex: &MoveVertex,
) -> Result<Option<Vec<Option<Color>>>, String> {
    if matches!(vertex, MoveVertex::Point(_)) {
        // Board's local ko marker can also flag non-repeating single captures. Preserve the
        // board while clearing that marker, then enforce the complete position history below.
        let anchor = Point { x: 0, y: 0 };
        let existing = board.get(anchor).map_err(|error| error.to_string())?;
        board
            .set_stone(anchor, existing)
            .map_err(|error| error.to_string())?;
    }
    board
        .play(to_core_color(color), to_core_vertex(vertex))
        .map_err(|error| error.to_string())?;
    if matches!(vertex, MoveVertex::Pass) {
        return Ok(None);
    }
    let stones = board.stones_snapshot();
    history.check(&stones)?;
    Ok(Some(stones))
}

fn unique_property<'a>(node: &'a SgfNode, key: &str) -> Result<Option<&'a SgfProperty>, String> {
    let mut properties = node.properties.iter().filter(|property| property.key == key);
    let first = properties.next();
    if properties.next().is_some() {
        return Err(format!("duplicate {key} properties cannot be projected exactly"));
    }
    Ok(first)
}

fn single_value<'a>(node: &'a SgfNode, key: &str) -> Result<Option<&'a str>, String> {
    unique_property(node, key)?
        .map(|property| match property.values.as_slice() {
            [value] => Ok(value.as_str()),
            _ => Err(format!("{key} must have exactly one value")),
        })
        .transpose()
}

fn exact_komi(raw: &str) -> Result<f32, String> {
    let error = || "komi must be a finite, exactly representable half-integer SGF number".to_string();
    let unsigned = raw.strip_prefix(['+', '-']).unwrap_or(raw);
    let (integer, fraction) = unsigned.split_once('.').unwrap_or((unsigned, ""));
    if integer.is_empty() || !integer.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(error());
    }
    let fraction = match fraction.trim_end_matches('0') {
        "" => "0",
        "5" => "5",
        _ => return Err(error()),
    };
    let komi = raw.parse::<f32>().map_err(|_| error())?;
    if !komi.is_finite() {
        return Err(error());
    }
    let integer = integer.trim_start_matches('0');
    let integer = if integer.is_empty() { "0" } else { integer };
    let sign = if raw.starts_with('-') && komi != 0.0 {
        "-"
    } else {
        ""
    };
    let canonical = format!("{sign}{integer}.{fraction}");
    if format!("{:.1}", if komi == 0.0 { 0.0 } else { komi }) != canonical {
        return Err(error());
    }
    Ok(komi)
}

#[cfg(test)]
mod tests {
    use app_model::{ExactRulesDto, MoveDto, MoveVertex, NodePath, PlayerColor, PointDto, StoneDto};

    use crate::{CurrentSgfDocument, ExactPosition};

    fn point(x: u8, y: u8) -> MoveVertex {
        MoveVertex::Point(PointDto { x, y })
    }

    fn selected(input: &str) -> ExactPosition {
        let document = CurrentSgfDocument::open(input).unwrap();
        document
            .exact_position(&document.default_selected_path())
            .unwrap()
    }

    fn rejected(input: &str) {
        let document = CurrentSgfDocument::open(input).unwrap();
        assert!(
            document
                .exact_position(&document.default_selected_path())
                .is_err(),
            "{input}"
        );
    }

    #[test]
    fn empty_root_keeps_defaults_and_explicit_white_is_not_a_pass() {
        let position = selected("(;RU[cHiNeSe])");
        assert_eq!(
            (position.dto().board_width, position.dto().board_height),
            (19, 19)
        );
        assert_eq!(position.dto().komi, 7.5);
        assert_eq!(position.dto().rules, ExactRulesDto::Chinese);
        assert_eq!(position.dto().initial_player, PlayerColor::Black);
        assert_eq!(position.dto().to_play, PlayerColor::Black);
        assert_eq!(position.dto().moves, vec![]);
        assert_eq!(position.dto().initial_stones, vec![]);

        let white = selected("(;SZ[9]RU[Chinese-KGS]PL[W]KM[-0.5])");
        assert_eq!(white.dto().initial_player, PlayerColor::White);
        assert_eq!(white.dto().to_play, PlayerColor::White);
        assert_eq!(white.dto().rules, ExactRulesDto::ChineseKgs);
        assert_eq!(white.dto().komi, -0.5);
        assert_eq!(white.dto().moves, vec![]);
        assert!(white.validate_move(&point(0, 0)).is_ok());
    }

    #[test]
    fn exact_path_retains_root_moves_passes_and_annotations() {
        let document = CurrentSgfDocument::open(
            "(;SZ[5]RU[Chinese]PL[W]W[aa];C[annotation]TR[bb]PL[B](;B[bb])(;B[];C[still White]PL[W];W[cc]))",
        )
        .unwrap();
        let path = NodePath {
            indices: vec![0, 1, 0, 0],
        };
        let position = document.exact_position(&path).unwrap();
        assert_eq!(position.dto().initial_player, PlayerColor::White);
        assert_eq!(position.dto().to_play, PlayerColor::Black);
        assert_eq!(
            position.dto().moves,
            vec![
                MoveDto {
                    color: PlayerColor::White,
                    vertex: point(0, 0),
                    move_number: 1
                },
                MoveDto {
                    color: PlayerColor::Black,
                    vertex: MoveVertex::Pass,
                    move_number: 2
                },
                MoveDto {
                    color: PlayerColor::White,
                    vertex: point(2, 2),
                    move_number: 3
                },
            ]
        );
        assert!(position.validate_move(&point(0, 0)).is_err());
        assert!(position.validate_move(&point(1, 1)).is_ok());
        assert!(document.exact_position(&NodePath { indices: vec![1] }).is_err());
        assert_eq!(
            document
                .exact_position(&NodePath::default())
                .unwrap()
                .dto()
                .moves
                .len(),
            1
        );
    }

    #[test]
    fn black_handicap_is_real_setup_with_white_initial_player() {
        let document = CurrentSgfDocument::open("(;SZ[5]RU[Chinese]AB[aa:ba]HA[2];W[cc])").unwrap();
        let root = document.exact_position(&NodePath::default()).unwrap();
        assert_eq!(
            root.dto().initial_stones,
            vec![
                StoneDto {
                    x: 0,
                    y: 0,
                    color: PlayerColor::Black
                },
                StoneDto {
                    x: 1,
                    y: 0,
                    color: PlayerColor::Black
                },
            ]
        );
        assert_eq!(root.dto().moves, vec![]);
        assert_eq!(root.dto().to_play, PlayerColor::White);
        let position = document
            .exact_position(&document.default_selected_path())
            .unwrap();
        assert_eq!(position.dto().initial_player, PlayerColor::White);
        assert_eq!(position.dto().to_play, PlayerColor::Black);
        assert_eq!(
            position.dto().moves,
            vec![MoveDto {
                color: PlayerColor::White,
                vertex: point(2, 2),
                move_number: 1
            },]
        );
        assert!(position.validate_move(&point(0, 0)).is_err());
        assert_eq!(
            selected("(;SZ[5]RU[Chinese]AB[aa][ee]PL[W])")
                .dto()
                .initial_player,
            PlayerColor::White
        );
    }

    #[test]
    fn unsupported_root_semantics_are_rejected() {
        for input in [
            "(;SZ[5])",
            "(;SZ[5]RU[Japanese])",
            "(;SZ[5]RU[Chinese]RU[Chinese-KGS])",
            "(;SZ[5]RU[Chinese][Chinese-KGS])",
            "(;SZ[5:6]RU[Chinese])",
            "(;SZ[20]RU[Chinese])",
            "(;SZ[5]RU[Chinese]AW[aa])",
            "(;SZ[5]RU[Chinese]AB[aa][bb]AW[cc])",
            "(;SZ[5]RU[Chinese]AE[aa])",
            "(;SZ[5]RU[Chinese]AB[aa])",
            "(;SZ[5]RU[Chinese]AB[aa][bb]PL[B])",
            "(;SZ[5]RU[Chinese]AB[aa][bb]HA[3])",
            "(;SZ[5]RU[Chinese]HA[2])",
            "(;SZ[5]RU[Chinese]HA[invalid])",
            "(;SZ[5]RU[Chinese]AB[aa][bb]W[cc])",
            "(;SZ[5]RU[Chinese]AB[aa][aa])",
            "(;SZ[5]RU[Chinese]AB[aa:bb][bb])",
            "(;SZ[5]RU[Chinese]AB[aa]AB[bb])",
            "(;SZ[5]RU[Chinese]PL[B]PL[W])",
            "(;SZ[5]RU[Chinese]KM[6.5]KM[7.5])",
            "(;SZ[5]RU[Chinese]B[aa]W[bb])",
            "(;SZ[5]RU[Chinese]B[aa][bb])",
            "(;SZ[5]RU[Chinese]B[aa]B[bb])",
        ] {
            rejected(input);
        }
    }

    #[test]
    fn komi_is_checked_before_lossy_float_rounding() {
        for raw in [
            "NaN",
            "inf",
            "6.25",
            "7.5000000000000001",
            "16777217",
            "16777216.5",
            "1e100",
        ] {
            rejected(&format!("(;SZ[5]RU[Chinese]KM[{raw}])"));
        }
        for (raw, expected) in [
            ("+0007.500", 7.5),
            ("-0.000", 0.0),
            ("0", 0.0),
            ("-3.5", -3.5),
            ("16777216", 16777216.0),
        ] {
            assert_eq!(
                selected(&format!("(;SZ[5]RU[Chinese]KM[{raw}])")).dto().komi,
                expected
            );
        }
    }

    #[test]
    fn intermediate_changes_and_illegal_replay_are_rejected() {
        for suffix in [
            ";AB[aa]",
            ";AW[aa]",
            ";AE[aa]",
            ";RU[Chinese-KGS]",
            ";KM[6.5]",
            ";SZ[9]",
            ";HA[2]",
            ";PL[W]",
            ";W[aa]",
            ";B[aa];B[bb]",
            ";B[aa];W[aa]",
            ";B[ab];W[cc];B[ba];W[aa]",
            ";B[aa];PL[W]PL[B]",
        ] {
            rejected(&format!("(;SZ[5]RU[Chinese]{suffix})"));
        }
        for input in [
            "(;SZ[5]RU[Chinese]AB[zz][aa])",
            "(;SZ[5]RU[Chinese]PL[X])",
            "(;SZ[5]RU[Chinese];B[zz])",
        ] {
            assert!(CurrentSgfDocument::open(input).is_err());
        }
    }

    #[test]
    fn failed_variation_does_not_contaminate_another_projection() {
        let document = CurrentSgfDocument::open("(;SZ[5]RU[Chinese-KGS];B[aa](;W[aa])(;W[bb]))").unwrap();
        let good_path = NodePath { indices: vec![0, 1] };
        let before = document.exact_position(&good_path).unwrap();
        assert!(document
            .exact_position(&NodePath { indices: vec![0, 0] })
            .is_err());
        let after = document.exact_position(&good_path).unwrap();
        assert_eq!(after.dto(), before.dto());
        assert_eq!(
            after.dto().moves,
            vec![
                MoveDto {
                    color: PlayerColor::Black,
                    vertex: point(0, 0),
                    move_number: 1
                },
                MoveDto {
                    color: PlayerColor::White,
                    vertex: point(1, 1),
                    move_number: 2
                },
            ]
        );
        assert!(after.validate_move(&point(2, 2)).is_ok());
    }

    const KO: &str = ";B[ba];W[bb];B[ab];W[ca];B[bc];W[db];B[ee];W[cc];B[cb]";

    #[test]
    fn immediate_ko_and_pass_history_follow_the_complete_rule_set() {
        for rules in ["Chinese", "Chinese-KGS"] {
            let immediate = selected(&format!("(;SZ[5]RU[{rules}]{KO})"));
            assert!(immediate.validate_move(&point(1, 1)).is_err(), "{rules}");
            assert!(immediate.validate_move(&MoveVertex::Pass).is_ok());
            rejected(&format!("(;SZ[5]RU[{rules}]{KO};W[bb])"));
            let after_passes = selected(&format!("(;SZ[5]RU[{rules}]{KO};W[];B[])"));
            assert_eq!(after_passes.dto().to_play, immediate.dto().to_play);
            assert_eq!(
                after_passes.validate_move(&point(1, 1)).is_ok(),
                rules == "Chinese"
            );
            assert!(after_passes.validate_move(&MoveVertex::Pass).is_ok());
        }
        let simple = selected(&format!("(;SZ[5]RU[Chinese]{KO};W[];B[];W[bb])"));
        assert_eq!(simple.dto().to_play, PlayerColor::Black);
        rejected(&format!("(;SZ[5]RU[Chinese-KGS]{KO};W[];B[];W[bb])"));
    }

    #[test]
    fn result_validation_rejects_suicide_occupied_and_bounds_without_mutation() {
        let position = selected("(;SZ[3]RU[Chinese-KGS];B[ab];W[cc];B[ba])");
        let before = position.dto().clone();
        for vertex in [point(0, 0), point(1, 0), point(3, 0), point(0, 3)] {
            assert!(position.validate_move(&vertex).is_err());
            assert!(position.validate_move(&point(1, 1)).is_ok());
        }
        assert!(position.validate_move(&MoveVertex::Pass).is_ok());
        assert_eq!(position.dto(), &before);
    }
}
