use crate::{EngineFailureDto, ExactRulesDto, GameMoveJobDto, MoveVertex, NodePath, PlayerColor};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PkSideSettingsDto {
    pub profile_id: Option<String>,
    pub deadline_ms: u32,
    pub kata_max_visits: u32,
}

impl Default for PkSideSettingsDto {
    fn default() -> Self {
        Self { profile_id: None, deadline_ms: 30_000, kata_max_visits: 800 }
    }
}

/// Stable intentions only. No session, run, job, cursor or transient engine state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MatchDefaultsDto {
    pub board_size: u8,
    pub komi: f32,
    pub handicap: u8,
    pub human_color: PlayerColor,
    pub rules: Option<ExactRulesDto>,
    pub profile_id: Option<String>,
    pub deadline_ms: u32,
    pub kata_max_visits: u32,
    pub pk_max_moves: u32,
    pub pk_black: PkSideSettingsDto,
    pub pk_white: PkSideSettingsDto,
}

impl Default for MatchDefaultsDto {
    fn default() -> Self {
        Self {
            board_size: 19, komi: 7.5, handicap: 0, human_color: PlayerColor::Black,
            rules: None, profile_id: None, deadline_ms: 30_000, kata_max_visits: 800,
            pk_max_moves: 450, pk_black: PkSideSettingsDto::default(), pk_white: PkSideSettingsDto::default(),
        }
    }
}

impl MatchDefaultsDto {
    /// Validates every field, as required before a New game rewrites board, komi, handicap and rules.
    pub fn validate(&self) -> Result<(), String> {
        if !(2..=19).contains(&self.board_size) {
            return Err("Match board size must be between 2 and 19.".into());
        }
        if !self.komi.is_finite() || (self.komi * 2.0).fract() != 0.0 {
            return Err("Match komi must be a finite half-point value.".into());
        }
        if self.handicap != 0 && (!(2..=9).contains(&self.handicap) || ![9, 13, 19].contains(&self.board_size)) {
            return Err("Fixed handicap requires 2–9 stones on a 9, 13 or 19 board.".into());
        }
        self.validate_participants()
    }

    /// Validates only the participant fields; a Continue inherits the position fields from the document.
    pub fn validate_participants(&self) -> Result<(), String> {
        if self.deadline_ms == 0 || self.kata_max_visits == 0 || self.pk_max_moves == 0 {
            return Err("Move deadline, KataGo visits and PK move limit must be positive.".into());
        }
        if self.profile_id.as_ref().is_some_and(|id| id.trim().is_empty()) {
            return Err("Choose a saved engine profile.".into());
        }
        for (side, settings) in [("Black", &self.pk_black), ("White", &self.pk_white)] {
            if settings.deadline_ms == 0 || settings.kata_max_visits == 0 {
                return Err(format!("PK {side} move deadline and KataGo visits must be positive."));
            }
            if settings.profile_id.as_ref().is_some_and(|id| id.trim().is_empty()) {
                return Err(format!("Choose a saved engine profile for PK {side}."));
            }
        }
        Ok(())
    }

    /// Saved defaults after a Continue: only participant choices change, never position fields.
    pub fn with_participants_of(&self, chosen: &Self) -> Self {
        Self {
            human_color: chosen.human_color,
            profile_id: chosen.profile_id.clone(),
            deadline_ms: chosen.deadline_ms,
            kata_max_visits: chosen.kata_max_visits,
            pk_black: chosen.pk_black.clone(),
            pk_white: chosen.pk_white.clone(),
            pk_max_moves: chosen.pk_max_moves,
            ..self.clone()
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchModeDto { #[default] Human, Pk }

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchPhaseDto { #[default] Idle, Starting, Playing, Paused, Ending, Error }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchEndDto { Stopped, Resigned, TwoPasses, MoveLimit, Failed }

/// Session-only policy; never part of match defaults or recovery.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchAnalysisPolicyDto { #[default] Off, HumanTurn, EngineTurn, Both }

impl MatchAnalysisPolicyDto {
    pub fn includes(self, human: bool) -> bool {
        self == Self::Both || if human { self == Self::HumanTurn } else { self == Self::EngineTurn }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchAnalysisFrameDto {
    pub turn: MatchTurnDto,
    pub epoch: u64,
    pub job: GameMoveJobDto,
    pub frame: crate::AnalysisFrameDto,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MatchAnalysisDto {
    pub supported: bool,
    pub policy: MatchAnalysisPolicyDto,
    pub epoch: u64,
    pub frame: Option<MatchAnalysisFrameDto>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchSnapshotDto {
    pub revision: u64,
    pub phase: MatchPhaseDto,
    #[serde(default)]
    pub mode: MatchModeDto,
    pub session_id: Option<String>,
    pub turn: u64,
    pub to_play: Option<PlayerColor>,
    pub settings: Option<MatchDefaultsDto>,
    pub run_id: Option<String>,
    /// Immutable participant snapshots in Black, White order.
    #[serde(default)]
    pub pk_runs: Option<[crate::EngineRunDto; 2]>,
    pub job: Option<GameMoveJobDto>,
    pub end: Option<MatchEndDto>,
    pub failure: Option<EngineFailureDto>,
    pub failed_side: Option<PlayerColor>,
    pub resources_held: bool,
    pub committed: bool,
    #[serde(default)]
    pub committed_moves: u32,
    #[serde(default)]
    pub pause_pending: bool,
    /// Sides whose cancelled GTP run Pause reaped. Only an explicit Resume rebuilds them,
    /// from the immutable profile snapshot in `pk_runs`.
    #[serde(default)]
    pub rebuild_sides: Vec<PlayerColor>,
    /// Paused-internal reconstruction is in progress; repeated Resume does not start another.
    #[serde(default)]
    pub resume_pending: bool,
    pub analysis: MatchAnalysisDto,
}

impl Default for MatchSnapshotDto {
    fn default() -> Self {
        Self { revision: 0, phase: MatchPhaseDto::Idle, mode: MatchModeDto::Human, session_id: None, turn: 0,
            to_play: None, settings: None, run_id: None, pk_runs: None, job: None, end: None,
            failure: None, failed_side: None, resources_held: false, committed: false,
            committed_moves: 0, pause_pending: false, rebuild_sides: Vec::new(), resume_pending: false,
            analysis: MatchAnalysisDto::default() }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchTurnDto {
    pub session_id: String,
    pub turn: u64,
    pub generation: u64,
    pub node_path: NodePath,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum HumanMatchActionDto { Play { vertex: MoveVertex }, Resign }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchUpdateDto {
    pub match_state: MatchSnapshotDto,
    pub current: Option<crate::CurrentGameResultDto>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HumanMatchStartDto {
    pub settings: MatchDefaultsDto,
    pub generation: u64,
    pub snapshot_seq: u64,
    pub start: MatchStartDto,
}

/// Where a human match begins.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MatchStartDto {
    /// Replaces the current document with a fresh game built from the settings.
    New { discard_confirmed: bool },
    /// Continues from the exact selected node, inheriting board, rules, komi, setup and player to move.
    /// `root_metadata_confirmed` acknowledges that root PB/PW are replaced and RE is cleared.
    Continue { node_path: NodePath, root_metadata_confirmed: bool },
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn legacy_defaults_supply_independent_pk_settings() {
        let defaults: MatchDefaultsDto = serde_json::from_value(json!({
            "board_size": 9,
            "komi": 6.5,
            "human_color": "white",
            "profile_id": "human-engine",
            "deadline_ms": 1200,
            "kata_max_visits": 50,
            "pk_max_moves": 42
        })).unwrap();
        assert_eq!(defaults.board_size, 9);
        assert_eq!(defaults.profile_id.as_deref(), Some("human-engine"));
        assert_eq!(defaults.pk_max_moves, 42);
        assert_eq!(defaults.pk_black, PkSideSettingsDto::default());
        assert_eq!(defaults.pk_white, PkSideSettingsDto::default());
        assert_eq!(defaults.pk_black.deadline_ms, 30_000);
        assert_eq!(defaults.pk_white.kata_max_visits, 800);
        defaults.validate_participants().unwrap();

        let partial: PkSideSettingsDto = serde_json::from_value(json!({ "profile_id": "black" })).unwrap();
        assert_eq!(partial, PkSideSettingsDto {
            profile_id: Some("black".into()), ..PkSideSettingsDto::default()
        });
        let empty: MatchDefaultsDto = serde_json::from_value(json!({})).unwrap();
        assert_eq!(empty, MatchDefaultsDto::default());
    }

    #[test]
    fn zero_pk_budgets_are_rejected_on_both_sides() {
        for side in ["pk_black", "pk_white"] {
            for budget in ["deadline_ms", "kata_max_visits"] {
                let mut json = serde_json::to_value(MatchDefaultsDto::default()).unwrap();
                json[side][budget] = json!(0);
                let defaults: MatchDefaultsDto = serde_json::from_value(json).unwrap();
                assert!(defaults.validate_participants().is_err(), "{side} {budget}");
                assert!(defaults.validate().is_err(), "{side} {budget}");
            }
        }
        let defaults = MatchDefaultsDto { pk_max_moves: 0, ..MatchDefaultsDto::default() };
        assert!(defaults.validate_participants().is_err());
    }

    #[test]
    fn pk_profile_selection_is_optional_until_start_but_never_blank() {
        MatchDefaultsDto::default().validate_participants().unwrap();
        for black in [true, false] {
            let mut defaults = MatchDefaultsDto::default();
            let side = if black { &mut defaults.pk_black } else { &mut defaults.pk_white };
            side.profile_id = Some(" \t\n".into());
            assert!(defaults.validate_participants().is_err());
        }
    }

    #[test]
    fn continue_copies_all_participants_without_position_settings() {
        let saved = MatchDefaultsDto {
            board_size: 9, komi: 6.5, handicap: 2, rules: Some(ExactRulesDto::Chinese),
            ..MatchDefaultsDto::default()
        };
        let chosen = MatchDefaultsDto {
            human_color: PlayerColor::White, profile_id: Some("human".into()),
            deadline_ms: 1000, kata_max_visits: 10, pk_max_moves: 3,
            pk_black: PkSideSettingsDto { profile_id: Some("black".into()), deadline_ms: 2000, kata_max_visits: 20 },
            pk_white: PkSideSettingsDto { profile_id: Some("white".into()), deadline_ms: 3000, kata_max_visits: 30 },
            ..MatchDefaultsDto::default()
        };
        let merged = saved.with_participants_of(&chosen);
        assert_eq!(merged.board_size, saved.board_size);
        assert_eq!(merged.komi, saved.komi);
        assert_eq!(merged.handicap, saved.handicap);
        assert_eq!(merged.rules, saved.rules);
        assert_eq!(merged.human_color, chosen.human_color);
        assert_eq!(merged.profile_id, chosen.profile_id);
        assert_eq!(merged.deadline_ms, chosen.deadline_ms);
        assert_eq!(merged.kata_max_visits, chosen.kata_max_visits);
        assert_eq!(merged.pk_black, chosen.pk_black);
        assert_eq!(merged.pk_white, chosen.pk_white);
        assert_eq!(merged.pk_max_moves, chosen.pk_max_moves);
        let round_trip: MatchDefaultsDto = serde_json::from_value(serde_json::to_value(&merged).unwrap()).unwrap();
        assert_eq!(round_trip, merged);
    }
}
