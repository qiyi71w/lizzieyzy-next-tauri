use crate::{NodePath, OrdinaryRulesSnapshotDto, QualifiedLocalResourceDto};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Explicit session request, never a display preference or an SGF analysis slot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RawNetRequestDto {
    pub run_id: String,
    pub generation: u64,
    pub node_path: NodePath,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RawNetSnapshotDto {
    pub confirmation: OrdinaryRulesSnapshotDto,
    pub resource: Arc<QualifiedLocalResourceDto>,
    pub document_identity: u64,
    pub board_revision: u64,
    /// The single main-board raw-net host, not primary/secondary ordinary analysis.
    pub slot: RawNetSlotDto,
    /// Black-positive, row-major, top-left. Absence is unavailable, never neutral zeros.
    pub ownership: Option<Vec<f32>>,
    pub unavailable: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RawNetSlotDto {
    MainBoard,
}
