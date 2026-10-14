use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum MoveRankMode {
    #[default]
    Auto,
    Score,
    Winrate,
    Combined,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct MoveRankPreferencesDto {
    pub mode: MoveRankMode,
    pub winrate_thresholds_pp: [f64; 5],
    pub score_thresholds: [f64; 5],
}

impl Default for MoveRankPreferencesDto {
    fn default() -> Self {
        Self {
            mode: MoveRankMode::Auto,
            winrate_thresholds_pp: [1.0, 3.0, 6.0, 12.0, 24.0],
            score_thresholds: [0.5, 1.5, 3.0, 6.0, 12.0],
        }
    }
}

impl MoveRankPreferencesDto {
    pub fn validate(&self) -> Result<(), String> {
        for thresholds in [&self.winrate_thresholds_pp, &self.score_thresholds] {
            if !thresholds.iter().all(|value| value.is_finite())
                || thresholds.windows(2).any(|pair| pair[0] > pair[1])
            {
                return Err("Move Rank thresholds must be finite and nondecreasing.".into());
            }
        }
        Ok(())
    }
}
