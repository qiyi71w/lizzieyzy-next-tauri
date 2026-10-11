use serde::{Deserialize, Serialize};

/// A preference, never an authorization to restart or analyze the current document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AutoLoadPreferencesDto {
    pub enabled: bool,
    pub total_visits: u32,
}

impl Default for AutoLoadPreferencesDto {
    fn default() -> Self {
        Self {
            enabled: false,
            total_visits: 1,
        }
    }
}

impl AutoLoadPreferencesDto {
    pub fn validate(&self) -> Result<(), String> {
        if self.total_visits == 0 {
            return Err("Automatic load visits must be a whole number from 1 to 4294967295.".into());
        }
        Ok(())
    }
}

/// Captured by the replacement transaction, not inferred from path or component mount.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GameLoadSourceDto {
    Open,
    Recent,
    FileDelivery,
    Clipboard,
    Text,
    #[default]
    Excluded,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AutoLoadPhaseDto {
    #[default]
    Idle,
    Waiting,
    Running,
    Complete,
    NotNeeded,
    Unavailable,
    Cancelled,
    Failed,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoLoadSnapshotDto {
    pub phase: AutoLoadPhaseDto,
    pub load_id: u64,
    pub generation: u64,
    pub run_id: Option<String>,
    pub task_id: Option<String>,
    pub total_visits: u32,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AutoLoadRecoveryPhaseDto {
    RestartRequired,
    Confirming,
    Confirmed,
}
