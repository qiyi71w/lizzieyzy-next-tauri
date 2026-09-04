use app_model::PlayerColor;

const WINRATE_THRESHOLDS_PP: [f32; 5] = [1.0, 3.0, 6.0, 12.0, 24.0];
const SCORE_THRESHOLDS: [f32; 5] = [0.5, 1.5, 3.0, 6.0, 12.0];
const MISTAKE_LEVEL: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveRank {
    Best,
    Good,
    Normal,
    Inaccuracy,
    Mistake,
    Blunder,
}

impl MoveRank {
    fn from_level(level: usize) -> Self {
        match level {
            5 => Self::Blunder,
            4 => Self::Mistake,
            3 => Self::Inaccuracy,
            2 => Self::Normal,
            1 => Self::Good,
            _ => Self::Best,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DisplayedAnalysis {
    pub visits: u32,
    pub winrate_black: f32,
    pub score_mean_black: Option<f32>,
}

pub fn classify_auto_move_rank(winrate_loss_pp: f32, score_loss: Option<f32>) -> MoveRank {
    let winrate_loss = winrate_loss_pp.max(0.0);
    let score_loss = score_loss.map(|loss| loss.max(0.0));
    for level in (1..=5).rev() {
        if reaches_auto_threshold(winrate_loss, score_loss, level) {
            return MoveRank::from_level(level);
        }
    }
    MoveRank::Best
}

pub fn classify_played_move(
    parent: Option<&DisplayedAnalysis>,
    child: Option<&DisplayedAnalysis>,
    parent_to_play: PlayerColor,
) -> Option<MoveRank> {
    let parent = displayed(parent)?;
    let child = displayed(child)?;
    let winrate_loss_pp =
        player_winrate_pp(parent, parent_to_play) - player_winrate_pp(child, parent_to_play);
    let score_loss = match (parent.score_mean_black, child.score_mean_black) {
        (Some(parent_score), Some(child_score)) => {
            Some(player_score(parent_score, parent_to_play) - player_score(child_score, parent_to_play))
        }
        _ => None,
    };
    Some(classify_auto_move_rank(winrate_loss_pp, score_loss))
}

pub fn is_blunder_bar_rank(rank: MoveRank) -> bool {
    matches!(rank, MoveRank::Inaccuracy | MoveRank::Mistake | MoveRank::Blunder)
}

fn displayed(analysis: Option<&DisplayedAnalysis>) -> Option<&DisplayedAnalysis> {
    analysis.filter(|value| value.visits > 0)
}

fn player_winrate_pp(analysis: &DisplayedAnalysis, color: PlayerColor) -> f32 {
    match color {
        PlayerColor::Black => analysis.winrate_black * 100.0,
        PlayerColor::White => (1.0 - analysis.winrate_black) * 100.0,
    }
}

fn player_score(score_mean_black: f32, color: PlayerColor) -> f32 {
    match color {
        PlayerColor::Black => score_mean_black,
        PlayerColor::White => -score_mean_black,
    }
}

fn reaches_auto_threshold(winrate_loss_pp: f32, score_loss: Option<f32>, level: usize) -> bool {
    let reaches_winrate = winrate_loss_pp >= WINRATE_THRESHOLDS_PP[level - 1];
    let reaches_score = score_loss.is_some_and(|loss| loss >= SCORE_THRESHOLDS[level - 1]);
    match score_loss {
        None => reaches_winrate,
        Some(_) => reaches_score || (level > MISTAKE_LEVEL && reaches_winrate),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn analysis(visits: u32, winrate_black: f32, score_mean_black: Option<f32>) -> DisplayedAnalysis {
        DisplayedAnalysis {
            visits,
            winrate_black,
            score_mean_black,
        }
    }

    #[test]
    fn winrate_only_boundaries_are_inclusive() {
        let cases = [
            (0.0, MoveRank::Best),
            (0.999, MoveRank::Best),
            (1.0, MoveRank::Good),
            (2.999, MoveRank::Good),
            (3.0, MoveRank::Normal),
            (5.999, MoveRank::Normal),
            (6.0, MoveRank::Inaccuracy),
            (11.999, MoveRank::Inaccuracy),
            (12.0, MoveRank::Mistake),
            (23.999, MoveRank::Mistake),
            (24.0, MoveRank::Blunder),
            (100.0, MoveRank::Blunder),
        ];
        for (loss, rank) in cases {
            assert_eq!(classify_auto_move_rank(loss, None), rank, "winrate loss {loss}");
        }
    }

    #[test]
    fn score_ordinary_rank_ignores_moderate_winrate_loss() {
        assert_eq!(
            classify_auto_move_rank(6.0, Some(0.4)),
            MoveRank::Best,
            "Java autoUsesScoreForOrdinaryDifferencesWithoutEquatingEveryThreshold"
        );
        let cases = [
            (0.0, MoveRank::Best),
            (0.499, MoveRank::Best),
            (0.5, MoveRank::Good),
            (1.499, MoveRank::Good),
            (1.5, MoveRank::Normal),
            (2.999, MoveRank::Normal),
            (3.0, MoveRank::Inaccuracy),
            (5.999, MoveRank::Inaccuracy),
            (6.0, MoveRank::Mistake),
            (11.999, MoveRank::Mistake),
            (12.0, MoveRank::Blunder),
        ];
        for (loss, rank) in cases {
            assert_eq!(
                classify_auto_move_rank(0.0, Some(loss)),
                rank,
                "score loss {loss}"
            );
        }
    }

    #[test]
    fn winrate_guards_mistake_and_blunder_when_score_is_small() {
        assert_eq!(
            classify_auto_move_rank(20.0, Some(0.1)),
            MoveRank::Mistake,
            "Java autoKeepsLargeWinrateDropAsSafetyGuard"
        );
        assert_eq!(classify_auto_move_rank(12.0, Some(0.0)), MoveRank::Mistake);
        assert_eq!(
            classify_auto_move_rank(24.0, Some(0.0)),
            MoveRank::Blunder,
            "Java autoFallsBack is for missing score; with score, 24pp still Blunder"
        );
        assert_eq!(classify_auto_move_rank(11.999, Some(0.0)), MoveRank::Best);
    }

    #[test]
    fn missing_score_falls_back_to_winrate() {
        assert_eq!(
            classify_auto_move_rank(24.0, None),
            MoveRank::Blunder,
            "Java autoFallsBackToWinrateWhenScoreIsUnavailable"
        );
        assert_eq!(classify_auto_move_rank(1.0, None), MoveRank::Good);
    }

    #[test]
    fn gains_and_negative_inputs_are_best() {
        assert_eq!(classify_auto_move_rank(-8.0, None), MoveRank::Best);
        assert_eq!(classify_auto_move_rank(-8.0, Some(-4.0)), MoveRank::Best);
    }

    #[test]
    fn played_move_uses_parent_side_to_play() {
        let parent_black = analysis(10, 0.60, None);
        let child_after_black = analysis(10, 0.48, None);
        assert_eq!(
            classify_played_move(Some(&parent_black), Some(&child_after_black), PlayerColor::Black),
            Some(MoveRank::Mistake)
        );

        let parent_white = analysis(10, 0.40, None);
        let child_after_white = analysis(10, 0.52, None);
        assert_eq!(
            classify_played_move(Some(&parent_white), Some(&child_after_white), PlayerColor::White),
            Some(MoveRank::Mistake),
            "white-to-play loss is the drop in white winrate, not the rise in black winrate"
        );
        assert_eq!(
            classify_played_move(Some(&parent_white), Some(&child_after_white), PlayerColor::Black),
            Some(MoveRank::Best),
            "the same black-winrate rise would look like a gain if side-to-play were ignored"
        );

        let parent_white_score = analysis(10, 0.50, Some(-4.0));
        let child_white_score = analysis(10, 0.50, Some(-1.0));
        assert_eq!(
            classify_played_move(
                Some(&parent_white_score),
                Some(&child_white_score),
                PlayerColor::White
            ),
            Some(MoveRank::Inaccuracy)
        );
    }

    #[test]
    fn missing_malformed_and_zero_visit_are_ungraded() {
        let ok = analysis(8, 0.55, Some(1.0));
        let zero = analysis(0, 0.10, Some(-12.0));
        assert_eq!(classify_played_move(None, Some(&ok), PlayerColor::Black), None);
        assert_eq!(classify_played_move(Some(&ok), None, PlayerColor::Black), None);
        assert_eq!(
            classify_played_move(Some(&zero), Some(&ok), PlayerColor::Black),
            None
        );
        assert_eq!(
            classify_played_move(Some(&ok), Some(&zero), PlayerColor::Black),
            None
        );
    }

    #[test]
    fn last_three_ranks_are_the_blunder_bar_filter() {
        assert!(!is_blunder_bar_rank(MoveRank::Best));
        assert!(!is_blunder_bar_rank(MoveRank::Good));
        assert!(!is_blunder_bar_rank(MoveRank::Normal));
        assert!(is_blunder_bar_rank(MoveRank::Inaccuracy));
        assert!(is_blunder_bar_rank(MoveRank::Mistake));
        assert!(is_blunder_bar_rank(MoveRank::Blunder));
    }

    #[test]
    fn mixed_score_and_winrate_keep_one_most_severe_rank() {
        assert_eq!(classify_auto_move_rank(24.0, Some(6.0)), MoveRank::Blunder);
        assert_eq!(classify_auto_move_rank(6.0, Some(12.0)), MoveRank::Blunder);
        assert_eq!(classify_auto_move_rank(3.0, Some(0.5)), MoveRank::Good);
    }
}
