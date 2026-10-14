use crate::{MoveVertex, NodePath};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct LocalFocusPreferencesDto {
    pub max_visits: u32,
    pub show_outline: bool,
    pub outline_opacity: u8,
}
impl Default for LocalFocusPreferencesDto {
    fn default() -> Self { Self { max_visits: 500, show_outline: true, outline_opacity: 92 } }
}
impl LocalFocusPreferencesDto {
    pub fn validate(&self) -> Result<(), String> {
        if self.max_visits == 0 || self.outline_opacity > 100 {
            return Err("Focus visits must be positive and outline opacity must be 0–100.".into());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalFocusOutputDto {
    pub command: u32,
    pub lineage: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LocalFocusPointStatusDto { Active, Completed, Blocked, Expired, Stopped }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalFocusPointDto {
    pub vertex: MoveVertex,
    pub target_visits: u32,
    pub visits: u32,
    pub status: LocalFocusPointStatusDto,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalFocusSnapshotDto {
    pub run_id: Option<String>,
    pub job_id: Option<String>,
    pub generation: Option<u64>,
    pub node_path: Option<NodePath>,
    pub points: Vec<LocalFocusPointDto>,
    pub withdrawal_pending: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum LocalFocusActionDto {
    Add { vertex: MoveVertex, max_visits: u32 },
    Remove { vertex: MoveVertex },
    Clear,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalFocusRequestDto {
    pub run_id: String,
    pub job_id: String,
    pub generation: u64,
    pub node_path: NodePath,
    #[serde(flatten)]
    pub action: LocalFocusActionDto,
}
