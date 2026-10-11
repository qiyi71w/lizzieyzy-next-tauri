use crate::*;
use serde::{Deserialize, Serialize};

/// Independent durable presets; neither a queue nor permission to write files.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct BatchPreferencesDto {
    pub strategy: AnalysisTaskStrategyDto,
    pub scope: AnalysisScopeDto,
    pub normal: AnalysisStageConditionsDto,
    pub overview: AnalysisStageConditionsDto,
    pub deep: AnalysisStageConditionsDto,
    pub swing_overview: AnalysisStageConditionsDto,
    pub swing_deep: AnalysisStageConditionsDto,
    pub swing_criteria: AnalysisSwingCriteriaDto,
}
impl Default for BatchPreferencesDto {
    fn default() -> Self {
        let visits = |value| AnalysisStageConditionsDto {
            total_visits: AnalysisTaskLimitDto { enabled: true, value },
            ..Default::default()
        };
        let mut swing_deep = AnalysisStageConditionsDto::default();
        swing_deep.time_seconds.enabled = true;
        swing_deep.total_visits.enabled = false;
        Self {
            strategy: AnalysisTaskStrategyDto::SingleStage,
            scope: AnalysisScopeDto {
                mode: AnalysisScopeModeDto::FirstChildMainline,
                current_node: NodePath::default(),
                branch_choices: vec![],
                interval: None,
                to_play: None,
            },
            normal: visits(100),
            overview: visits(32),
            deep: visits(800),
            swing_overview: visits(32),
            swing_deep,
            swing_criteria: Default::default(),
        }
    }
}
impl BatchPreferencesDto {
    pub fn validate(&self) -> Result<(), String> {
        self.normal.validate_single_stage()?;
        AnalysisStageConditionsDto::validate_all_positions_two_stage(&self.overview, &self.deep)?;
        self.swing_overview.validate_single_stage()?;
        self.swing_deep.validate_single_stage()?;
        self.swing_criteria.validate()?;
        if !matches!(
            self.scope.mode,
            AnalysisScopeModeDto::FirstChildMainline | AnalysisScopeModeDto::AllBranches
        ) || !self.scope.current_node.indices.is_empty()
            || !self.scope.branch_choices.is_empty()
        {
            return Err("Batch scope must resolve from each file, not the current game.".into());
        }
        if self.scope.interval.as_ref().is_some_and(|v| v.start > v.end) {
            return Err("The closed interval start exceeds its end.".into());
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BatchRowStateDto {
    Pending,
    Waiting,
    Running,
    Completed,
    Failed,
    Cancelled,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BatchOutputStateDto {
    Unsaved,
    Saving,
    Saved,
    Failed,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchIdentityDto {
    pub batch_id: String,
    pub row_id: String,
    pub attempt: u64,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BatchRowDto {
    pub identity: BatchIdentityDto,
    pub source_path: String,
    pub output_name: String,
    pub state: BatchRowStateDto,
    pub conditions: BatchPreferencesDto,
    pub preview: Option<AnalysisScopePreviewDto>,
    pub task: Option<AnalysisTaskDto>,
    pub document_id: Option<String>,
    pub input_receipt: Option<String>,
    pub calculation_complete: bool,
    pub has_result: bool,
    pub output_state: BatchOutputStateDto,
    pub saved_path: Option<String>,
    pub incomplete: bool,
    pub reason: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BatchSnapshotDto {
    pub batch_id: String,
    pub rows: Vec<BatchRowDto>,
    pub notices: Vec<String>,
    pub started: bool,
    pub paused: bool,
    pub reason: Option<String>,
    pub output_directory: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum BatchCommandDto {
    Intake {
        paths: Vec<String>,
        conditions: BatchPreferencesDto,
    },
    Remove {
        identity: BatchIdentityDto,
    },
    Reorder {
        row_ids: Vec<String>,
    },
    Start {
        output: BatchOutputAuthorizationDto,
    },
    Continue,
    Pause,
    StopCurrent,
    StopAll,
    Rerun {
        identity: BatchIdentityDto,
    },
    Save {
        identity: BatchIdentityDto,
        path: String,
        overwrite: bool,
        incomplete: bool,
    },
    NewBatch,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum BatchOutputAuthorizationDto {
    SessionOnly,
    Directory {
        path: String,
        confirmed_names: Vec<String>,
    },
}
