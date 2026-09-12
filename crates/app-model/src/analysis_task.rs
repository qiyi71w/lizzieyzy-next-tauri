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
    pub fn validate_single_stage(&self) -> Result<(), String> {
        let limits = [
            &self.time_seconds,
            &self.total_visits,
            &self.leading_candidate_visits,
        ];
        if limits.iter().any(|limit| limit.value == 0) {
            return Err(
                "Task limits must be whole numbers from 1 to 4294967295, including retained disabled values."
                    .into(),
            );
        }
        if !limits.iter().any(|limit| limit.enabled) {
            return Err("Enable at least one task search condition.".into());
        }
        Ok(())
    }
}

impl Default for AnalysisStageConditionsDto {
    fn default() -> Self {
        Self {
            time_seconds: AnalysisTaskLimitDto {
                enabled: false,
                value: 10,
            },
            total_visits: AnalysisTaskLimitDto {
                enabled: true,
                value: 800,
            },
            leading_candidate_visits: AnalysisTaskLimitDto {
                enabled: false,
                value: 500,
            },
        }
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
    Pausing,
    Paused,
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
    #[serde(default)]
    pub ending_conditions: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_stage_admits_each_independent_budget_and_rejects_invalid_retained_values() {
        for enabled in 1..8 {
            let conditions = AnalysisStageConditionsDto {
                time_seconds: AnalysisTaskLimitDto {
                    enabled: enabled & 1 != 0,
                    value: 10,
                },
                total_visits: AnalysisTaskLimitDto {
                    enabled: enabled & 2 != 0,
                    value: u32::MAX,
                },
                leading_candidate_visits: AnalysisTaskLimitDto {
                    enabled: enabled & 4 != 0,
                    value: 500,
                },
            };
            assert!(conditions.validate_single_stage().is_ok(), "{conditions:?}");
            let mut invalid = conditions.clone();
            invalid.time_seconds.value = 0;
            assert!(invalid.validate_single_stage().is_err());
        }
        let disabled = AnalysisStageConditionsDto {
            time_seconds: AnalysisTaskLimitDto {
                enabled: false,
                value: 10,
            },
            total_visits: AnalysisTaskLimitDto {
                enabled: false,
                value: 800,
            },
            leading_candidate_visits: AnalysisTaskLimitDto {
                enabled: false,
                value: 500,
            },
        };
        assert!(disabled.validate_single_stage().is_err());
    }
}
