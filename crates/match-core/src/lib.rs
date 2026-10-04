use app_model::{ExactPositionDto, MatchDefaultsDto, MatchEndDto, MoveVertex, PlayerColor};

/// Pure turn authority. Document legality and process ownership stay with their existing owners.
#[derive(Debug)]
pub struct HumanMatch {
    human: Option<PlayerColor>,
    to_play: PlayerColor,
    turn: u64,
    consecutive_passes: u8,
    committed_moves: u32,
    max_moves: Option<u32>,
    end: Option<MatchEndDto>,
}

impl HumanMatch {
    /// Starts at the exact committed position. Position fields of `settings` are not consulted:
    /// callers validate them only when they build the position (New). An inherited trailing pass
    /// counts toward the double pass that ends the match.
    pub fn new(settings: &MatchDefaultsDto, position: &ExactPositionDto) -> Result<Self, String> {
        settings.validate_participants()?;
        if settings.profile_id.is_none() {
            return Err("Choose a saved engine profile before starting.".into());
        }
        let trailing_pass = position.moves.last().is_some_and(|last| last.vertex == MoveVertex::Pass);
        Ok(Self { human: Some(settings.human_color), to_play: position.to_play,
            turn: 1, consecutive_passes: u8::from(trailing_pass), committed_moves: 0,
            max_moves: None, end: None })
    }

    /// Both sides are engines. Only moves committed after this start count toward the limit.
    pub fn new_pk(settings: &MatchDefaultsDto, position: &ExactPositionDto) -> Result<Self, String> {
        settings.validate_participants()?;
        for (side, settings) in [("Black", &settings.pk_black), ("White", &settings.pk_white)] {
            if settings.profile_id.is_none() {
                return Err(format!("Choose a saved engine profile for PK {side} before starting."));
            }
        }
        let trailing_pass = position.moves.last().is_some_and(|last| last.vertex == MoveVertex::Pass);
        Ok(Self { human: None, to_play: position.to_play,
            turn: 1, consecutive_passes: u8::from(trailing_pass), committed_moves: 0,
            max_moves: Some(settings.pk_max_moves), end: None })
    }

    pub fn turn(&self) -> u64 { self.turn }
    pub fn to_play(&self) -> PlayerColor { self.to_play }
    pub fn end(&self) -> Option<MatchEndDto> { self.end }
    pub fn committed_moves(&self) -> u32 { self.committed_moves }
    pub fn human_turn(&self) -> bool { self.end.is_none() && Some(self.to_play) == self.human }

    /// Revokes callbacks for this turn without changing the committed position or termination rules.
    pub fn invalidate_turn(&mut self) {
        self.turn += 1;
    }

    pub fn admits(&self, turn: u64, human: bool) -> bool {
        self.end.is_none() && self.turn == turn && (Some(self.to_play) == self.human) == human
    }

    /// Called only after a legal document edit has been committed.
    pub fn played(&mut self, pass: bool) {
        assert!(self.end.is_none(), "a sealed match cannot commit a move");
        self.consecutive_passes = if pass { self.consecutive_passes + 1 } else { 0 };
        self.committed_moves += 1;
        self.turn += 1;
        self.to_play = self.to_play.opponent();
        if self.consecutive_passes == 2 {
            self.end = Some(MatchEndDto::TwoPasses);
        } else if self.max_moves.is_some_and(|limit| self.committed_moves >= limit) {
            self.end = Some(MatchEndDto::MoveLimit);
        }
    }

    pub fn seal(&mut self, reason: MatchEndDto) {
        self.end.get_or_insert(reason);
    }

    pub fn resignation_result(&self) -> &'static str {
        match self.to_play { PlayerColor::Black => "W+R", PlayerColor::White => "B+R" }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use app_model::{ExactRulesDto, MoveDto, PointDto};

    fn pk_settings(max_moves: u32) -> MatchDefaultsDto {
        let mut settings = MatchDefaultsDto { pk_max_moves: max_moves, ..MatchDefaultsDto::default() };
        settings.pk_black.profile_id = Some("same-profile".into());
        settings.pk_white.profile_id = Some("same-profile".into());
        settings
    }

    fn position(vertices: &[MoveVertex]) -> ExactPositionDto {
        let mut to_play = PlayerColor::Black;
        let mut moves = Vec::new();
        for (index, vertex) in vertices.iter().enumerate() {
            moves.push(MoveDto { color: to_play, vertex: vertex.clone(), move_number: index as u32 + 1 });
            to_play = to_play.opponent();
        }
        ExactPositionDto {
            board_width: 9, board_height: 9, komi: 7.5, rules: ExactRulesDto::Chinese,
            initial_player: PlayerColor::Black, to_play, initial_stones: Vec::new(), moves,
        }
    }

    #[test]
    fn pk_counts_only_new_commits_and_seals_at_move_limit() {
        let inherited = position(&[
            MoveVertex::Point(PointDto { x: 0, y: 0 }),
            MoveVertex::Point(PointDto { x: 1, y: 0 }),
        ]);
        let mut game = HumanMatch::new_pk(&pk_settings(2), &inherited).unwrap();
        assert_eq!(game.committed_moves(), 0);
        assert_eq!(game.to_play(), inherited.to_play);
        assert_eq!(game.end(), None);
        game.played(false);
        assert_eq!(game.committed_moves(), 1);
        assert_eq!(game.end(), None);
        game.played(false);
        assert_eq!(game.committed_moves(), 2);
        assert_eq!(game.end(), Some(MatchEndDto::MoveLimit));
        assert!(!game.admits(game.turn(), false));
    }

    #[test]
    fn inherited_last_pass_ends_on_first_new_pass_before_limit() {
        let mut game = HumanMatch::new_pk(&pk_settings(1), &position(&[MoveVertex::Pass])).unwrap();
        assert_eq!(game.committed_moves(), 0);
        assert_eq!(game.to_play(), PlayerColor::White);
        game.played(true);
        assert_eq!(game.committed_moves(), 1);
        assert_eq!(game.end(), Some(MatchEndDto::TwoPasses));
    }

    #[test]
    fn double_pass_takes_precedence_when_limit_is_reached() {
        let mut game = HumanMatch::new_pk(&pk_settings(2), &position(&[])).unwrap();
        game.played(true);
        assert_eq!(game.end(), None);
        game.played(true);
        assert_eq!(game.committed_moves(), 2);
        assert_eq!(game.end(), Some(MatchEndDto::TwoPasses));
    }

    #[test]
    fn ordinary_move_breaks_inherited_pass_and_first_pass_counts_toward_limit() {
        let mut game = HumanMatch::new_pk(&pk_settings(2), &position(&[MoveVertex::Pass])).unwrap();
        game.played(false);
        game.played(true);
        assert_eq!(game.committed_moves(), 2);
        assert_eq!(game.end(), Some(MatchEndDto::MoveLimit));
    }

    #[test]
    fn pk_never_admits_human_and_rejects_old_turns() {
        let mut game = HumanMatch::new_pk(&pk_settings(3), &position(&[])).unwrap();
        assert!(!game.human_turn());
        assert!(!game.admits(1, true));
        assert!(game.admits(1, false));
        game.played(false);
        assert_eq!(game.to_play(), PlayerColor::White);
        assert!(!game.human_turn());
        assert!(!game.admits(1, false));
        assert!(!game.admits(2, true));
        assert!(game.admits(2, false));
    }

    #[test]
    fn pk_start_requires_each_profile_and_positive_budgets_and_limit() {
        let inherited = position(&[]);
        assert!(HumanMatch::new_pk(&MatchDefaultsDto::default(), &inherited).is_err());
        assert!(HumanMatch::new_pk(&pk_settings(0), &inherited).is_err());
        for black in [true, false] {
            for invalid in ["missing", "blank", "deadline", "visits"] {
                let mut settings = pk_settings(3);
                let side = if black { &mut settings.pk_black } else { &mut settings.pk_white };
                match invalid {
                    "missing" => side.profile_id = None,
                    "blank" => side.profile_id = Some(" \t".into()),
                    "deadline" => side.deadline_ms = 0,
                    "visits" => side.kata_max_visits = 0,
                    _ => unreachable!(),
                }
                assert!(HumanMatch::new_pk(&settings, &inherited).is_err(), "black={black}: {invalid}");
            }
        }
    }

    #[test]
    fn resignation_preserves_result_and_does_not_commit_a_move() {
        for (vertices, result) in [(vec![], "W+R"), (vec![MoveVertex::Pass], "B+R")] {
            let mut game = HumanMatch::new_pk(&pk_settings(1), &position(&vertices)).unwrap();
            assert_eq!(game.resignation_result(), result);
            game.seal(MatchEndDto::Resigned);
            game.seal(MatchEndDto::MoveLimit);
            assert_eq!(game.end(), Some(MatchEndDto::Resigned));
            assert_eq!(game.committed_moves(), 0);
            assert!(!game.admits(game.turn(), false));
        }
    }

    #[test]
    fn human_roles_and_unlimited_moves_are_unchanged() {
        let settings = MatchDefaultsDto {
            profile_id: Some("human-engine".into()), pk_max_moves: 1,
            ..MatchDefaultsDto::default()
        };
        let mut game = HumanMatch::new(&settings, &position(&[])).unwrap();
        assert!(game.human_turn());
        assert!(game.admits(1, true));
        assert!(!game.admits(1, false));
        game.played(false);
        assert!(!game.human_turn());
        assert!(game.admits(2, false));
        assert_eq!(game.committed_moves(), 1);
        assert_eq!(game.end(), None);
        game.played(true);
        game.played(true);
        assert_eq!(game.committed_moves(), 3);
        assert_eq!(game.end(), Some(MatchEndDto::TwoPasses));
    }

    #[test]
    fn invalidating_turn_rejects_old_identity_without_resetting_position_or_passes() {
        let mut game = HumanMatch::new_pk(&pk_settings(3), &position(&[MoveVertex::Pass])).unwrap();
        game.played(false);
        game.played(true);
        let old_turn = game.turn();
        let side = game.to_play();
        let committed_moves = game.committed_moves();
        let end = game.end();
        assert!(game.admits(old_turn, false));
        game.invalidate_turn();
        assert_eq!(game.turn(), old_turn + 1);
        assert!(!game.admits(old_turn, false));
        assert!(game.admits(game.turn(), false));
        assert_eq!(game.to_play(), side);
        assert_eq!(game.committed_moves(), committed_moves);
        assert_eq!(game.end(), end);
        game.played(true);
        assert_eq!(game.committed_moves(), 3);
        assert_eq!(game.end(), Some(MatchEndDto::TwoPasses));
    }
}
