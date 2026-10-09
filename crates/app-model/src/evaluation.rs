use crate::QualifiedLocalResourceDto;
use serde::{Deserialize, Serialize};

/// Startup authorization only; neither execution nor a measured recommendation.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StartupEvaluationSettingsDto {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub target_profile_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvaluationPhaseDto {
    Idle,
    Unavailable,
    Qualifying,
    Running,
    Completed,
    Failed,
    Cancelled,
    Yielded,
    Retired,
}

/// Actual benchmark observations only; never a recommendation or durable policy.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvaluationResultDto {
    pub target_id: String,
    pub input_revision: String,
    pub qualified_resource: QualifiedLocalResourceDto,
    pub elapsed_ms: u64,
    pub search_visits_per_second: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvaluationSnapshotDto {
    pub evaluation_id: Option<String>,
    pub target_id: Option<String>,
    pub input_revision: Option<String>,
    pub phase: EvaluationPhaseDto,
    pub process_id: Option<u32>,
    pub exit_code: Option<i32>,
    pub output: Vec<String>,
    pub output_truncated: bool,
    pub message: Option<String>,
    pub result: Option<EvaluationResultDto>,
}

impl Default for EvaluationSnapshotDto {
    fn default() -> Self {
        Self {
            evaluation_id: None,
            target_id: None,
            input_revision: None,
            phase: EvaluationPhaseDto::Idle,
            process_id: None,
            exit_code: None,
            output: Vec::new(),
            output_truncated: false,
            message: None,
            result: None,
        }
    }
}
