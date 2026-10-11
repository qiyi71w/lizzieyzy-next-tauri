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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThreadStrategyDto {
    #[default]
    Cfg,
    Benchmark,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SavedThreadPolicyDto {
    pub strategy: ThreadStrategyDto,
    pub revision: u64,
    pub recommendation: Option<ThreadRecommendationDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThreadRecommendationDto {
    pub evaluation_id: String,
    pub target_id: String,
    pub input_revision: String,
    pub policy_revision: u64,
    pub source_revision: String,
    pub threads: u32,
    pub seconds_per_move: String,
    pub source_row: String,
    pub qualified_resource: QualifiedLocalResourceDto,
    pub command_revision: String,
    pub environment: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecommendationValidityDto {
    None,
    Invalid,
    Changed,
    Unknown,
    Valid,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThreadPolicyStatusDto {
    pub target_id: String,
    pub input_revision: String,
    pub policy: SavedThreadPolicyDto,
    pub validity: RecommendationValidityDto,
    pub reason: String,
    pub recommendation: Option<ThreadRecommendationDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThreadPolicySaveRequestDto {
    pub target_id: String,
    pub expected_revision: u64,
    pub input_revision: String,
    pub strategy: ThreadStrategyDto,
    pub evaluation_id: Option<String>,
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

/// One complete protocol row; unavailable metrics are independent, never inferred.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvaluationMeasurementDto {
    pub measured_threads: u32,
    pub nn_evals_per_second: Option<f64>,
    pub search_visits_per_second: Option<f64>,
}

/// Actual benchmark observations and an optional source-qualified draft recommendation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvaluationResultDto {
    pub evaluation_id: String,
    pub target_id: String,
    pub input_revision: String,
    pub qualified_resource: QualifiedLocalResourceDto,
    pub elapsed_ms: u64,
    pub measurements: Vec<EvaluationMeasurementDto>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recommendation: Option<ThreadRecommendationDto>,
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
