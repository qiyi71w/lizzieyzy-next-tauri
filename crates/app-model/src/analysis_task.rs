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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisTaskStrategyDto {
    SingleStage,
    AllPositionsTwoStage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisTaskStageDto {
    SingleStage,
    Overview,
    Deep,
}

impl AnalysisStageConditionsDto {
    pub fn validate_all_positions_two_stage(overview: &Self, deep: &Self) -> Result<(), String> {
        overview.validate_single_stage()?;
        deep.validate_single_stage()?;
        if deep.total_visits.value < 500 {
            return Err("All-position deep total visits must be at least 500.".into());
        }
        Ok(())
    }
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnalysisTaskOverviewDto {
    pub node_path: NodePath,
    pub frame: crate::AnalysisFrameDto,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnalysisTaskDto {
    pub task_id: String,
    pub run_id: String,
    pub job_id: String,
    pub generation: u64,
    pub scope: AnalysisScopeDto,
    pub strategy: AnalysisTaskStrategyDto,
    pub stage: AnalysisTaskStageDto,
    pub conditions: AnalysisStageConditionsDto,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub overview_conditions: Option<AnalysisStageConditionsDto>,
    pub requested: Vec<NodePath>,
    #[serde(default)]
    pub overview_completed: Vec<NodePath>,
    pub completed: Vec<NodePath>,
    #[serde(default)]
    pub overview_summaries: Vec<AnalysisTaskOverviewDto>,
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

    #[test]
    fn all_position_stage_budgets_validate_independently_and_keep_deep_minimum() {
        let overview = AnalysisStageConditionsDto {
            total_visits: AnalysisTaskLimitDto {
                enabled: true,
                value: 32,
            },
            ..AnalysisStageConditionsDto::default()
        };
        let deep = AnalysisStageConditionsDto::default();
        assert!(AnalysisStageConditionsDto::validate_all_positions_two_stage(&overview, &deep).is_ok());

        let mut shallow = deep.clone();
        shallow.total_visits.value = 499;
        assert!(AnalysisStageConditionsDto::validate_all_positions_two_stage(&overview, &shallow).is_err());

        let mut disabled = overview.clone();
        disabled.total_visits.enabled = false;
        assert!(AnalysisStageConditionsDto::validate_all_positions_two_stage(&disabled, &deep).is_err());
    }
}
