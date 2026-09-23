use super::trial::{mode_error, seal_review_jobs, wait_temporary_jobs, TrialMode};
use super::*;
use app_model::{
    AreaCompensationDto, PointDto, PositionDto, ScoringActionDto, ScoringRuleDto, ScoringSessionDto,
};
use go_core::{Board, Color, Point};

pub(super) struct ScoringSession {
    id: u64,
    revision: u64,
    entry_path: NodePath,
    generation: u64,
    position: PositionDto,
    komi: f32,
    rule: ScoringRuleDto,
    compensation: AreaCompensationDto,
    handicap: u32,
    dead: Vec<Point>,
    neutral: Vec<Point>,
}

fn board_from_position(position: &PositionDto) -> Result<Board, CurrentGameError> {
    let mut board = Board::new(position.board_width, position.board_height).map_err(scoring_error)?;
    for stone in &position.stones {
        board
            .set_stone(
                Point {
                    x: stone.x,
                    y: stone.y,
                },
                Some(match stone.color {
                    PlayerColor::Black => Color::Black,
                    PlayerColor::White => Color::White,
                }),
            )
            .map_err(scoring_error)?;
    }
    Ok(board)
}

fn scoring_error(error: go_core::RuleError) -> CurrentGameError {
    CurrentGameError {
        kind: CurrentGameErrorKind::InvalidNodePath,
        message: error.to_string(),
    }
}

// SGF komi is stored as f32; its display string is the canonical decimal users see.
// Adding integer points in decimal avoids binary cancellation in the winning margin.
fn add_points_to_komi(komi: &str, points: i64) -> String {
    let (mantissa, exponent) = komi.split_once(['e', 'E']).map_or((komi, 0), |(text, exponent)| {
        (text, exponent.parse::<i32>().expect("finite f32 exponent"))
    });
    let (negative, unsigned) = mantissa
        .strip_prefix('-')
        .map_or((false, mantissa), |s| (true, s));
    let (whole, fraction) = unsigned.split_once('.').unwrap_or((unsigned, ""));
    let mut digits: Vec<u8> = whole
        .bytes()
        .chain(fraction.bytes())
        .rev()
        .map(|digit| digit - b'0')
        .collect();
    let mut scale = fraction.len() as i32 - exponent;
    if scale < 0 {
        digits.splice(0..0, std::iter::repeat_n(0, -scale as usize));
        scale = 0;
    }
    let mut other = vec![0; scale as usize];
    other.extend(
        points
            .unsigned_abs()
            .to_string()
            .bytes()
            .rev()
            .map(|digit| digit - b'0'),
    );
    while digits.len() > 1 && digits.last() == Some(&0) {
        digits.pop();
    }
    while other.len() > 1 && other.last() == Some(&0) {
        other.pop();
    }
    let (mut sum, negative) = if negative == (points < 0) {
        let mut carry = 0;
        let sum = (0..digits.len().max(other.len()))
            .map(|i| {
                let value = digits.get(i).copied().unwrap_or(0) + other.get(i).copied().unwrap_or(0) + carry;
                carry = value / 10;
                value % 10
            })
            .collect::<Vec<_>>();
        let mut sum = sum;
        if carry != 0 {
            sum.push(carry);
        }
        (sum, negative)
    } else {
        let left_larger = digits.len() > other.len()
            || (digits.len() == other.len() && digits.iter().rev().cmp(other.iter().rev()).is_ge());
        let (large, small, result_negative) = if left_larger {
            (&digits, &other, negative)
        } else {
            (&other, &digits, points < 0)
        };
        let mut borrow = 0i16;
        let difference = (0..large.len())
            .map(|i| {
                let mut value = i16::from(large[i]) - i16::from(small.get(i).copied().unwrap_or(0)) - borrow;
                borrow = if value < 0 {
                    value += 10;
                    1
                } else {
                    0
                };
                value as u8
            })
            .collect();
        (difference, result_negative)
    };
    while sum.len() > 1 && sum.last() == Some(&0) {
        sum.pop();
    }
    if sum.len() == 1 && sum[0] == 0 {
        return "0".into();
    }
    let mut scale = scale as usize;
    let trailing = sum.iter().take(scale).take_while(|&&digit| digit == 0).count();
    sum.drain(..trailing);
    scale -= trailing;
    let mut value: String = sum.iter().rev().map(|digit| char::from(b'0' + digit)).collect();
    if scale != 0 {
        if value.len() <= scale {
            value = format!("{}{}", "0".repeat(scale + 1 - value.len()), value);
        }
        value.insert(value.len() - scale, '.');
    }
    if negative && value != "0" {
        format!("-{value}")
    } else {
        value
    }
}

impl ScoringSession {
    fn result(&self) -> Result<ScoringSessionDto, CurrentGameError> {
        let board = board_from_position(&self.position)?;
        let tally = board.score(&self.dead, &self.neutral).map_err(scoring_error)?;
        let black = match self.rule {
            ScoringRuleDto::Area => u64::from(tally.black_stones) + u64::from(tally.black_territory),
            ScoringRuleDto::Territory => {
                u64::from(tally.black_territory)
                    + u64::from(self.position.captures_black)
                    + u64::from(tally.white_dead)
            }
        };
        let white = match self.rule {
            ScoringRuleDto::Area => u64::from(tally.white_stones) + u64::from(tally.white_territory),
            ScoringRuleDto::Territory => {
                u64::from(tally.white_territory)
                    + u64::from(self.position.captures_white)
                    + u64::from(tally.black_dead)
            }
        };
        let bonus = if self.rule == ScoringRuleDto::Territory {
            0
        } else {
            match self.compensation {
                AreaCompensationDto::None => 0,
                AreaCompensationDto::Handicap => self.handicap,
                AreaCompensationDto::HandicapMinusOne => self.handicap.saturating_sub(1),
            }
        };
        let komi = self.komi.to_string();
        let black_total = black.to_string();
        let white_total = add_points_to_komi(&komi, (white + u64::from(bonus)) as i64);
        let margin = add_points_to_komi(&komi, (white + u64::from(bonus)) as i64 - black as i64);
        let result = if margin == "0" {
            "0".into()
        } else if let Some(black_margin) = margin.strip_prefix('-') {
            format!("B+{black_margin}")
        } else {
            format!("W+{margin}")
        };
        Ok(ScoringSessionDto {
            session_id: self.id,
            revision: self.revision,
            entry_path: self.entry_path.clone(),
            generation: self.generation,
            position: self.position.clone(),
            dead: self.dead.iter().map(|p| PointDto { x: p.x, y: p.y }).collect(),
            neutral: self.neutral.iter().map(|p| PointDto { x: p.x, y: p.y }).collect(),
            ownership: tally
                .ownership
                .into_iter()
                .map(|c| {
                    c.map(|c| match c {
                        Color::Black => PlayerColor::Black,
                        Color::White => PlayerColor::White,
                    })
                })
                .collect(),
            rule: self.rule,
            compensation: self.compensation,
            handicap: self.handicap,
            komi: self.komi,
            black_stones: tally.black_stones,
            white_stones: tally.white_stones,
            black_territory: tally.black_territory,
            white_territory: tally.white_territory,
            black_dead: tally.black_dead,
            white_dead: tally.white_dead,
            black_total,
            white_total,
            result,
        })
    }
}

fn initial_handicap(
    root: &app_model::SgfTreeNodeDto,
    rules: &str,
    inferred: Option<u32>,
) -> (u32, AreaCompensationDto) {
    let handicap = inferred.unwrap_or(0);
    let compensation =
        if inferred.is_none() || handicap == 0 || !root.properties.iter().any(|p| p.key == "RU") {
            AreaCompensationDto::None
        } else if rules.contains("aga") {
            AreaCompensationDto::Handicap
        } else if rules.contains("chinese") {
            AreaCompensationDto::HandicapMinusOne
        } else {
            AreaCompensationDto::None
        };
    (handicap, compensation)
}

impl CurrentGameState {
    pub fn enter_scoring(&self, rule: ScoringRuleDto) -> Result<ScoringSessionDto, String> {
        let manager = self.analysis_manager.get();
        let (id, jobs) = {
            let mut holder = self.holder.lock().expect("current game state");
            holder.ensure_editable().map_err(|e| e.message)?;
            let document = holder
                .document
                .as_ref()
                .ok_or_else(|| no_current_game().message)?;
            let position = document
                .snapshot(&holder.selected_path)
                .map_err(|e| e.message)?
                .position;
            if !document.komi().is_finite() {
                return Err("Scoring requires finite komi.".into());
            }
            let (handicap, compensation) = initial_handicap(
                &document.tree().map_err(|e| e.message)?,
                &document.rules(),
                document.scoring_handicap().map_err(|e| e.message)?,
            );
            let komi = document.komi();
            holder.next_trial_id = holder.next_trial_id.saturating_add(1);
            holder.next_trial_revision = holder.next_trial_revision.saturating_add(1).max(1 << 48);
            let session = ScoringSession {
                id: holder.next_trial_id,
                revision: holder.next_trial_revision,
                entry_path: holder.selected_path.clone(),
                generation: holder.generation,
                position,
                komi,
                rule,
                handicap,
                compensation,
                dead: Vec::new(),
                neutral: Vec::new(),
            };
            let id = session.id;
            holder.trial_mode = TrialMode::EnteringScoring(session);
            (id, seal_review_jobs(&mut holder, manager))
        };
        let result = jobs.and_then(|jobs| {
            if let Some(manager) = manager {
                wait_temporary_jobs(manager, &jobs)?;
            }
            Ok(())
        });
        let mut holder = self.holder.lock().expect("current game state");
        if let Err(error) = result {
            holder.trial_mode = TrialMode::Review;
            holder.analysis_target = None;
            self.follow_continuous_position(&mut holder);
            return Err(error);
        }
        let TrialMode::EnteringScoring(session) = std::mem::take(&mut holder.trial_mode) else {
            return Err(mode_error().message);
        };
        if session.id != id {
            return Err(mode_error().message);
        }
        let output = session.result().map_err(|e| e.message)?;
        holder.trial_mode = TrialMode::Scoring(session);
        Ok(output)
    }

    pub fn update_scoring(
        &self,
        id: u64,
        revision: u64,
        action: ScoringActionDto,
    ) -> Result<ScoringSessionDto, CurrentGameError> {
        let mut holder = self.holder.lock().expect("current game state");
        if holder.departure.is_some() {
            return Err(mode_error());
        }
        let TrialMode::Scoring(session) = &mut holder.trial_mode else {
            return Err(mode_error());
        };
        if session.id != id || session.revision != revision {
            return Err(mode_error());
        }
        match action {
            ScoringActionDto::Point { point } => {
                let point = Point {
                    x: point.x,
                    y: point.y,
                };
                let board = board_from_position(&session.position)?;
                match board.get(point).map_err(scoring_error)? {
                    Some(_) => {
                        let group = board.connected_group(point).map_err(scoring_error)?;
                        if session.dead.contains(&point) {
                            session.dead.retain(|p| !group.contains(p));
                        } else {
                            session.dead.extend(group);
                        }
                    }
                    None => {
                        if session.neutral.contains(&point) {
                            session.neutral.retain(|p| *p != point);
                        } else {
                            session.neutral.push(point);
                        }
                    }
                }
            }
            ScoringActionDto::Settings {
                rule,
                compensation,
                handicap,
            } => {
                if handicap > 625 {
                    return Err(CurrentGameError {
                        kind: CurrentGameErrorKind::InvalidNodePath,
                        message: "Handicap exceeds board capacity.".into(),
                    });
                }
                session.rule = rule;
                session.compensation = compensation;
                session.handicap = handicap;
            }
        }
        holder.next_trial_revision = holder.next_trial_revision.saturating_add(1);
        let revision = holder.next_trial_revision;
        let TrialMode::Scoring(session) = &mut holder.trial_mode else {
            unreachable!()
        };
        session.revision = revision;
        session.result()
    }

    pub fn exit_scoring(
        &self,
        id: u64,
        revision: u64,
        confirm: bool,
    ) -> Result<CurrentGameResultDto, String> {
        let manager = self.analysis_manager.get();
        let jobs = {
            let mut holder = self.holder.lock().expect("current game state");
            if holder.departure.is_some() {
                return Err(mode_error().message);
            }
            let mode = std::mem::take(&mut holder.trial_mode);
            let TrialMode::Scoring(session) = mode else {
                holder.trial_mode = mode;
                return Err(mode_error().message);
            };
            if session.id != id || session.revision != revision {
                holder.trial_mode = TrialMode::Scoring(session);
                return Err(mode_error().message);
            }
            holder.trial_mode = TrialMode::LeavingScoring(session);
            manager
                .map(|manager| {
                    let jobs = crate::document_departure::jobs_from_snapshot(&manager.snapshot());
                    for job in &jobs {
                        holder
                            .closed_jobs
                            .insert((job.run_id.clone(), job.job_id.clone()));
                    }
                    manager.clear_continuous_position();
                    jobs
                })
                .unwrap_or_default()
        };
        let result = manager
            .map(|manager| wait_temporary_jobs(manager, &jobs))
            .unwrap_or(Ok(()));
        let mut holder = self.holder.lock().expect("current game state");
        let mode = std::mem::take(&mut holder.trial_mode);
        let TrialMode::LeavingScoring(session) = mode else {
            holder.trial_mode = mode;
            return Err(mode_error().message);
        };
        if let Err(error) = result {
            holder.trial_mode = TrialMode::Scoring(session);
            return Err(error);
        }
        if confirm {
            let result = match session.result() {
                Ok(result) => result.result,
                Err(error) => {
                    holder.trial_mode = TrialMode::Scoring(session);
                    return Err(error.message);
                }
            };
            let selected_path = holder.selected_path.clone();
            let outcome = match holder
                .document
                .as_mut()
                .ok_or_else(no_current_game)
                .and_then(|document| document.set_result_with_history(&selected_path, &result))
            {
                Ok(outcome) => outcome,
                Err(error) => {
                    holder.trial_mode = TrialMode::Scoring(session);
                    return Err(error.message);
                }
            };
            if let Some(edit) = outcome.edit {
                holder.commit_edit(edit);
                holder.bump_snapshot();
                self.note_recovery(&holder);
            }
        }
        holder.analysis_target = None;
        self.follow_continuous_position(&mut holder);
        holder.current_result().map_err(|e| e.message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cancel_and_confirm_are_distinct_history_actions() {
        let state = CurrentGameState::default();
        let opened = state
            .replace("(;SZ[3:2]KM[-0.5]RE[B+R]XY[keep]AB[aa][ba][ca])", None)
            .unwrap();
        let preview = state.enter_scoring(ScoringRuleDto::Area).unwrap();
        assert!(
            !state
                .exit_scoring(preview.session_id, preview.revision, false)
                .unwrap()
                .dirty
        );
        assert!(!state.serialize().unwrap().contains("RE[0]"));
        let preview = state.enter_scoring(ScoringRuleDto::Territory).unwrap();
        let confirmed = state
            .exit_scoring(preview.session_id, preview.revision, true)
            .unwrap();
        assert_eq!(confirmed.generation, opened.generation);
        assert!(confirmed.dirty);
        assert!(state.serialize().unwrap().contains("XY[keep]"));
        assert!(state
            .undo(confirmed.generation)
            .unwrap()
            .tree
            .properties
            .iter()
            .any(|p| p.key == "RE" && p.values == ["B+R"]));
    }
    #[test]
    fn rules_compensation_komi_and_captures_share_exact_result() {
        let state = CurrentGameState::default();
        let opened = state
            .replace(
                "(;SZ[3:3]KM[0.5]HA[2]RU[Chinese]AB[ab][ba][cb]AW[bb];B[bc])",
                None,
            )
            .unwrap();
        let selected = state
            .select_path(NodePath { indices: vec![0] }, opened.generation)
            .unwrap();
        assert_eq!(selected.snapshot.position.captures_black, 1);
        let mut preview = state.enter_scoring(ScoringRuleDto::Area).unwrap();
        assert_eq!(preview.compensation, AreaCompensationDto::HandicapMinusOne);
        assert_eq!(preview.black_total, "9");
        assert_eq!(preview.white_total, "1.5");
        assert_eq!(preview.result, "B+7.5");
        preview = state
            .update_scoring(
                preview.session_id,
                preview.revision,
                ScoringActionDto::Settings {
                    rule: ScoringRuleDto::Area,
                    compensation: AreaCompensationDto::Handicap,
                    handicap: 2,
                },
            )
            .unwrap();
        assert_eq!(preview.result, "B+6.5");
        preview = state
            .update_scoring(
                preview.session_id,
                preview.revision,
                ScoringActionDto::Settings {
                    rule: ScoringRuleDto::Territory,
                    compensation: AreaCompensationDto::Handicap,
                    handicap: 2,
                },
            )
            .unwrap();
        assert_eq!(preview.black_territory, 5);
        assert_eq!(preview.black_total, "6");
        assert_eq!(preview.white_total, "0.5");
        assert_eq!(preview.result, "B+5.5");
        state
            .exit_scoring(preview.session_id, preview.revision, false)
            .unwrap();
        assert!(state.serialize().unwrap().contains("HA[2]"));
        assert!(state.serialize().unwrap().contains("RU[Chinese]"));
        assert!(!state.serialize().unwrap().contains("RE["));
    }

    #[test]
    fn dead_group_neutral_correction_and_negative_komi_are_preview_only() {
        let state = CurrentGameState::default();
        state
            .replace("(;SZ[3:2]KM[-0.5]AB[aa][ba][ca]AW[bb][cb])", None)
            .unwrap();
        let original = state.serialize().unwrap();
        let mut preview = state.enter_scoring(ScoringRuleDto::Area).unwrap();
        preview = state
            .update_scoring(
                preview.session_id,
                preview.revision,
                ScoringActionDto::Point {
                    point: PointDto { x: 1, y: 1 },
                },
            )
            .unwrap();
        assert_eq!(preview.dead.len(), 2);
        assert_eq!(preview.black_stones, 3);
        assert_eq!(preview.white_dead, 2);
        assert_eq!(preview.black_territory, 3);
        assert_eq!(preview.result, "B+6.5");
        preview = state
            .update_scoring(
                preview.session_id,
                preview.revision,
                ScoringActionDto::Point {
                    point: PointDto { x: 0, y: 1 },
                },
            )
            .unwrap();
        assert_eq!(preview.black_territory, 2);
        assert_eq!(preview.result, "B+5.5");
        preview = state
            .update_scoring(
                preview.session_id,
                preview.revision,
                ScoringActionDto::Point {
                    point: PointDto { x: 1, y: 1 },
                },
            )
            .unwrap();
        assert!(preview.dead.is_empty());
        state
            .exit_scoring(preview.session_id, preview.revision, false)
            .unwrap();
        assert_eq!(state.serialize().unwrap(), original);
    }

    #[test]
    fn tie_and_unknown_handicap_inference_do_not_mutate_root() {
        let state = CurrentGameState::default();
        state
            .replace("(;SZ[2:3]KM[0]AB[aa][ba]AW[ac][bc])", None)
            .unwrap();
        let preview = state.enter_scoring(ScoringRuleDto::Area).unwrap();
        assert_eq!(preview.handicap, 0);
        assert_eq!(preview.compensation, AreaCompensationDto::None);
        assert_eq!(preview.result, "0");
        assert!(state.enter_trial().is_err());
        assert!(state.enter_scoring(ScoringRuleDto::Area).is_err());
        let result = state
            .exit_scoring(preview.session_id, preview.revision, true)
            .unwrap();
        assert_eq!(result.generation, 1);
        assert!(state.serialize().unwrap().contains("RE[0]"));
    }
    #[test]
    fn compressed_setup_matches_expanded_handicap_and_result() {
        for source in [
            "(;SZ[5]KM[0]RU[Chinese]PL[W]AB[aa:ba])",
            "(;SZ[5]KM[0]RU[Chinese]PL[W]AB[aa][ba])",
        ] {
            let state = CurrentGameState::default();
            state.replace(source, None).unwrap();
            let preview = state.enter_scoring(ScoringRuleDto::Area).unwrap();
            assert_eq!(preview.handicap, 2);
            assert_eq!(preview.compensation, AreaCompensationDto::HandicapMinusOne);
            assert_eq!(preview.white_total, "1");
            assert_eq!(preview.result, "B+24");
            state
                .exit_scoring(preview.session_id, preview.revision, true)
                .unwrap();
            assert!(state.serialize().unwrap().contains("RE[B+24]"));
        }
    }

    #[test]
    fn fractional_komi_margin_is_exact_in_preview_and_sgf_history() {
        let state = CurrentGameState::default();
        let opened = state.replace("(;SZ[2]KM[0.1]AB[aa]AW[bb])", None).unwrap();
        let preview = state.enter_scoring(ScoringRuleDto::Area).unwrap();
        assert_eq!(preview.black_total, "1");
        assert_eq!(preview.white_total, "1.1");
        assert_eq!(preview.result, "W+0.1");
        state
            .exit_scoring(preview.session_id, preview.revision, true)
            .unwrap();
        assert!(state.serialize().unwrap().contains("RE[W+0.1]"));
        state.undo(opened.generation).unwrap();
        state.redo(opened.generation).unwrap();
        assert!(state.serialize().unwrap().contains("RE[W+0.1]"));
    }
    #[test]
    fn decimal_komi_arithmetic_keeps_sign_and_scale() {
        assert_eq!(add_points_to_komi("0.0", 0), "0");
        assert_eq!(add_points_to_komi("0.1", -1), "-0.9");
        assert_eq!(add_points_to_komi("-0.1", 1), "0.9");
        assert_eq!(add_points_to_komi("1.1", -1), "0.1");
        assert_eq!(add_points_to_komi("-1.1", 1), "-0.1");
        assert_eq!(add_points_to_komi("1e-10", 1), "1.0000000001");
        assert_eq!(add_points_to_komi("1e10", -1), "9999999999");
    }
}
