use crate::{NodePath, PlayerColor};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisScopeModeDto {
    CurrentNode,
    SelectedReviewLine,
    FirstChildMainline,
    AllBranches,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnalysisBranchChoiceDto {
    pub parent: NodePath,
    pub child: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnalysisPositionIntervalDto {
    pub start: u32,
    pub end: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnalysisScopeDto {
    pub mode: AnalysisScopeModeDto,
    pub current_node: NodePath,
    pub branch_choices: Vec<AnalysisBranchChoiceDto>,
    pub interval: Option<AnalysisPositionIntervalDto>,
    pub to_play: Option<PlayerColor>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnalysisTaskLimitDto {
    pub enabled: bool,
    pub value: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnalysisStageConditionsDto {
    pub time_seconds: AnalysisTaskLimitDto,
    pub total_visits: AnalysisTaskLimitDto,
    pub leading_candidate_visits: AnalysisTaskLimitDto,
}

impl AnalysisStageConditionsDto {
    pub fn validate_single_stage(&self) -> Result<u32, String> {
        if self.time_seconds.enabled || self.leading_candidate_visits.enabled {
            return Err("Only total-visit conditions are supported for this task.".into());
        }
        if !self.total_visits.enabled || self.total_visits.value == 0 {
            return Err("A positive total-visit budget is required.".into());
        }
        Ok(self.total_visits.value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnalysisScopeTargetDto {
    pub node_path: NodePath,
    pub move_number: u32,
    pub to_play: PlayerColor,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnalysisScopePreviewDto {
    pub generation: u64,
    pub scope: AnalysisScopeDto,
    pub targets: Vec<AnalysisScopeTargetDto>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisTaskStateDto {
    Queued,
    Searching,
    Completed,
    Cancelled,
    Failed,
    Invalidated,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnalysisTaskDto {
    pub task_id: String,
    pub run_id: String,
    pub job_id: String,
    pub generation: u64,
    pub scope: AnalysisScopeDto,
    pub stage: String,
    pub conditions: AnalysisStageConditionsDto,
    pub requested: Vec<NodePath>,
    pub completed: Vec<NodePath>,
    pub state: AnalysisTaskStateDto,
    pub reason: Option<String>,
}
