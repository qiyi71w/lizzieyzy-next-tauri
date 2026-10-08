use crate::{MoveDto, MoveVertex, NodePath, PlayerColor, StoneDto};
use serde::{Deserialize, Serialize};

/// Complete rule sets, not a scoring-only preference. Unknown SGF rules are rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExactRulesDto {
    Chinese,
    ChineseKgs,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExactPositionDto {
    pub board_width: u8,
    pub board_height: u8,
    pub komi: f32,
    pub rules: ExactRulesDto,
    pub initial_player: PlayerColor,
    pub to_play: PlayerColor,
    pub initial_stones: Vec<StoneDto>,
    pub moves: Vec<MoveDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrdinaryRulesRequestDto {
    pub run_id: String,
    pub generation: u64,
    pub node_path: NodePath,
}

/// An immutable observation, not authority to reuse it after a Run or position change.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrdinaryRulesSnapshotDto {
    pub identity: GameMoveJobDto,
    /// Each Run owns exactly one spawned process and stdout reader.
    pub reader_id: String,
    pub profile_revision: String,
    pub position: ExactPositionDto,
    /// Canonical actual rules JSON, including every engine-provided field.
    pub confirmed_rules: String,
    pub stones: Vec<StoneDto>,
    pub true_final_move: Option<MoveDto>,
}

/// A wall-clock ceiling for the entire operation; never a match clock.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComputeBudgetDto {
    pub deadline_ms: u32,
    pub max_visits: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameMoveRequestDto {
    pub run_id: String,
    pub generation: u64,
    pub node_path: NodePath,
    pub budget: ComputeBudgetDto,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameMoveJobDto {
    pub run_id: String,
    pub job_id: String,
    pub generation: u64,
    pub node_path: NodePath,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum GameMoveDto {
    Move { vertex: MoveVertex },
    Resign,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameMoveResultDto {
    pub run_id: String,
    pub job_id: String,
    pub generation: u64,
    pub node_path: NodePath,
    pub result: GameMoveDto,
    pub engine_time_mapped: bool,
}
