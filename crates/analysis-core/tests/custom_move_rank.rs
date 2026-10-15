use analysis_core::{classify_move_rank, classify_played_move, DisplayedAnalysis, MoveRank};
use app_model::{MoveRankMode, MoveRankPreferencesDto, PlayerColor};

#[test]
fn every_boundary_below_equal_above_and_duplicate_uses_worst_match() {
    let ranks = [
        MoveRank::Best,
        MoveRank::Good,
        MoveRank::Normal,
        MoveRank::Inaccuracy,
        MoveRank::Mistake,
        MoveRank::Blunder,
    ];
    for mode in [
        MoveRankMode::Winrate,
        MoveRankMode::Score,
        MoveRankMode::Combined,
        MoveRankMode::Auto,
    ] {
        let mut settings = MoveRankPreferencesDto {
            mode,
            ..Default::default()
        };
        for (index, boundary) in settings.score_thresholds.iter().copied().enumerate() {
            if mode == MoveRankMode::Winrate {
                continue;
            }
            for (delta, expected) in [
                (-0.0001, ranks[index]),
                (0.0, ranks[index + 1]),
                (0.0001, ranks[index + 1]),
            ] {
                assert_eq!(
                    classify_move_rank(0.0, Some(boundary + delta), &settings),
                    expected
                );
            }
        }
        for (index, boundary) in settings.winrate_thresholds_pp.iter().copied().enumerate() {
            if mode == MoveRankMode::Score {
                continue;
            }
            for (delta, expected) in [
                (-0.0001, ranks[index]),
                (0.0, ranks[index + 1]),
                (0.0001, ranks[index + 1]),
            ] {
                assert_eq!(classify_move_rank(boundary + delta, None, &settings), expected);
            }
        }
        settings.winrate_thresholds_pp = [2.0; 5];
        settings.score_thresholds = [2.0; 5];
        assert_eq!(classify_move_rank(1.999, Some(1.999), &settings), MoveRank::Best);
        assert_eq!(classify_move_rank(2.0, Some(2.0), &settings), MoveRank::Blunder);
        assert_eq!(
            classify_move_rank(2.001, Some(2.001), &settings),
            MoveRank::Blunder
        );
        settings.winrate_thresholds_pp = [-2.0, -1.0, 0.0, 150.0, 200.0];
        settings.score_thresholds = settings.winrate_thresholds_pp;
        assert!(settings.validate().is_ok());
        assert_eq!(
            classify_move_rank(-20.0, Some(-10.0), &settings),
            MoveRank::Inaccuracy
        );
    }
}

#[test]
fn modes_missing_score_positive_visits_and_both_perspectives() {
    for (mode, ordinary, severe, missing) in [
        (
            MoveRankMode::Auto,
            MoveRank::Best,
            MoveRank::Mistake,
            MoveRank::Blunder,
        ),
        (
            MoveRankMode::Score,
            MoveRank::Best,
            MoveRank::Best,
            MoveRank::Best,
        ),
        (
            MoveRankMode::Winrate,
            MoveRank::Inaccuracy,
            MoveRank::Mistake,
            MoveRank::Blunder,
        ),
        (
            MoveRankMode::Combined,
            MoveRank::Inaccuracy,
            MoveRank::Mistake,
            MoveRank::Blunder,
        ),
    ] {
        let settings = MoveRankPreferencesDto {
            mode,
            ..Default::default()
        };
        assert_eq!(classify_move_rank(6.0, Some(0.1), &settings), ordinary);
        assert_eq!(classify_move_rank(12.0, Some(0.1), &settings), severe);
        assert_eq!(classify_move_rank(24.0, None, &settings), missing);
        for color in [PlayerColor::Black, PlayerColor::White] {
            let sign = if color == PlayerColor::Black { 1.0 } else { -1.0 };
            let parent = DisplayedAnalysis {
                visits: 10,
                winrate_black: 0.5,
                score_mean_black: Some(0.0),
            };
            let child = DisplayedAnalysis {
                visits: 10,
                winrate_black: 0.5 - sign * 0.25,
                score_mean_black: None,
            };
            assert_eq!(
                classify_played_move(Some(&parent), Some(&child), color, &settings),
                Some(missing)
            );
            assert_eq!(classify_played_move(None, Some(&child), color, &settings), None);
            assert_eq!(
                classify_played_move(
                    Some(&parent),
                    Some(&DisplayedAnalysis { visits: 0, ..child }),
                    color,
                    &settings
                ),
                None
            );
        }
    }
}
