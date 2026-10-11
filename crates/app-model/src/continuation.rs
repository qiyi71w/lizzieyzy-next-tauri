use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContinuationModeDto {
    #[default]
    LeafOnly,
    EveryBeat,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContinuationPreferencesDto {
    pub mode: ContinuationModeDto,
    pub interval_seconds: f64,
}

impl Default for ContinuationPreferencesDto {
    fn default() -> Self {
        Self { mode: ContinuationModeDto::LeafOnly, interval_seconds: 1.0 }
    }
}

impl ContinuationPreferencesDto {
    /// Shares the lossless whole-millisecond representation of review settings.
    pub fn interval_millis(&self) -> Result<u64, String> {
        let millis = self.interval_seconds * 1000.0;
        if !millis.is_finite() || millis < 1.0 || millis > i32::MAX as f64
            || millis.fract() != 0.0 || millis / 1000.0 != self.interval_seconds {
            return Err("Continuation interval must be positive, finite, and exactly representable in whole milliseconds.".into());
        }
        Ok(millis as u64)
    }
    pub fn validate(&self) -> Result<(), String> { self.interval_millis().map(|_| ()) }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContinuationPhaseDto {
    #[default]
    Stopped,
    Waiting,
    Running,
    Failed,
    Terminal,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ContinuationSnapshotDto {
    pub session_id: Option<String>,
    pub phase: ContinuationPhaseDto,
    pub mode: ContinuationModeDto,
    pub run_id: Option<String>,
    pub request_id: Option<String>,
    pub generation: u64,
    pub node_path: crate::NodePath,
    pub reason: Option<String>,
    pub current_game: Option<crate::CurrentGameResultDto>,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn continuation_settings_reject_lossy_or_nonpositive_intervals() {
        let mut preferences = ContinuationPreferencesDto::default();
        assert_eq!(preferences.interval_millis().unwrap(), 1000);
        for seconds in [0.0, -1.0, f64::NAN, f64::INFINITY, 0.0001, 2147483.648] {
            preferences.interval_seconds = seconds;
            assert!(preferences.validate().is_err());
        }
        for (seconds, millis) in [(0.001, 1), (0.8, 800), (1.25, 1250)] {
            preferences.interval_seconds = seconds;
            assert_eq!(preferences.interval_millis().unwrap(), millis);
        }
        assert_eq!(ContinuationSnapshotDto::default().phase, ContinuationPhaseDto::Stopped);
    }
}
