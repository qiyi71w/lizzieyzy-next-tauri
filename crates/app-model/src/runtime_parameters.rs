use serde::{Deserialize, Serialize};

/// Atomic, finite values from one read-only current-Run request, never editor state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RuntimeParameterPairDto {
    pub playout_doubling_advantage: f64,
    pub analysis_wide_root_noise: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeParametersStatusDto { Unknown, Pending, Confirmed, Failed }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RuntimeParametersSnapshotDto {
    pub run_id: Option<String>,
    pub profile_revision: Option<String>,
    pub supported: bool,
    pub reason: Option<String>,
    pub request_id: Option<String>,
    /// Last complete pair for this Run; only Confirmed means current.
    pub last_valid: Option<RuntimeParameterPairDto>,
    pub status: RuntimeParametersStatusDto,
    pub failure: Option<String>,
}
