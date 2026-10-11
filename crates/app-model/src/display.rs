use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum CandidateDelayMode {
    #[default]
    Off,
    Timed,
    Manual,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum PvVisitsMode {
    #[default]
    Off,
    Last,
    All,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum CandidateScoreMode {
    #[default]
    Lead,
    Board,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum OwnershipStyle {
    #[default]
    Small,
    Normal,
    NotOnLive,
    BigBelow,
    BySize,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CandidateTextField {
    Winrate,
    Visits,
    Score,
}

/// Display-only preferences. No field grants computation or changes analysis data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct GranularDisplayDto {
    pub delay_mode: CandidateDelayMode,
    pub delay_seconds: f64,
    pub show_black_candidates: bool,
    pub show_white_candidates: bool,
    pub pv_visits_mode: PvVisitsMode,
    pub pv_visits_threshold: i32,
    pub pv_length: i32,
    pub show_winrate: bool,
    pub show_visits: bool,
    pub show_score: bool,
    pub score_mode: CandidateScoreMode,
    pub show_order: bool,
    pub emphasize_max: bool,
    pub text_order: [CandidateTextField; 3],
    pub ownership_style: OwnershipStyle,
    pub use_candidate_ownership: bool,
    pub raw_net_ownership_style: OwnershipStyle,
}
impl Default for GranularDisplayDto {
    fn default() -> Self {
        Self {
            delay_mode: CandidateDelayMode::Off,
            delay_seconds: 10.0,
            show_black_candidates: true,
            show_white_candidates: true,
            pv_visits_mode: PvVisitsMode::Off,
            pv_visits_threshold: 0,
            pv_length: 15,
            show_winrate: true,
            show_visits: true,
            show_score: true,
            score_mode: CandidateScoreMode::Lead,
            show_order: true,
            emphasize_max: false,
            text_order: [
                CandidateTextField::Winrate,
                CandidateTextField::Visits,
                CandidateTextField::Score,
            ],
            ownership_style: OwnershipStyle::Small,
            use_candidate_ownership: true,
            raw_net_ownership_style: OwnershipStyle::BigBelow,
        }
    }
}
impl GranularDisplayDto {
    pub fn validate(&self) -> Result<(), String> {
        let ms = (self.delay_seconds * 1000.0).round();
        if !ms.is_finite() || ms < 1.0 || ms > i32::MAX as f64 || ms / 1000.0 != self.delay_seconds {
            return Err(
                "Candidate delay must represent positive whole milliseconds within the timer domain.".into(),
            );
        }
        if self.pv_visits_threshold < 0 {
            return Err("PV visits threshold must be a nonnegative signed 32-bit integer.".into());
        }
        if self.text_order[0] == self.text_order[1]
            || self.text_order[0] == self.text_order[2]
            || self.text_order[1] == self.text_order[2]
        {
            return Err("Candidate text order must be a permutation of winrate, visits and score.".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn display_defaults_and_source_domains_are_preserved() {
        let mut prefs: GranularDisplayDto = serde_json::from_str("{}").unwrap();
        assert_eq!(prefs, GranularDisplayDto::default());
        assert!(serde_json::from_str::<GranularDisplayDto>(r#"{"showWinrate":1}"#).is_err());
        assert!(serde_json::from_str::<GranularDisplayDto>(r#"{"pvLength":2147483648}"#).is_err());
        prefs.pv_length = i32::MIN;
        prefs.pv_visits_threshold = i32::MAX;
        assert!(prefs.validate().is_ok());
        prefs.delay_seconds = 1.001;
        assert!(prefs.validate().is_ok());
        for seconds in [0.0, -1.0, f64::NAN, f64::INFINITY, 0.0001, 2147483.648] {
            prefs.delay_seconds = seconds;
            assert!(prefs.validate().is_err());
        }
        prefs.delay_seconds = 2147483.647;
        assert!(prefs.validate().is_ok());
        prefs.text_order[1] = prefs.text_order[0];
        assert!(prefs.validate().is_err());
    }
}
