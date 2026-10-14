use app_model::{AnalysisFrameDto, MoveRankPreferencesDto, PlayerColor, ProblemMarkerDto, ProblemSeverity};
use std::cmp::Reverse;

mod move_rank;
pub use move_rank::{
    classify_move_rank, classify_played_move, is_blunder_bar_rank, DisplayedAnalysis, MoveRank,
};

pub fn sort_candidates_by_visits(frame: &mut AnalysisFrameDto) {
    frame
        .candidates
        .sort_by_key(|candidate| Reverse(candidate.visits));
}

pub fn classify_problem_markers(frames: &[AnalysisFrameDto], settings: &MoveRankPreferencesDto) -> Vec<ProblemMarkerDto> {
    let mut markers = Vec::new();
    for pair in frames.windows(2) {
        let previous = &pair[0];
        let current = &pair[1];
        let color = if previous.turn % 2 == 0 { PlayerColor::Black } else { PlayerColor::White };
        let parent = DisplayedAnalysis { visits: previous.visits, winrate_black: previous.winrate_black, score_mean_black: previous.score_mean_black };
        let child = DisplayedAnalysis { visits: current.visits, winrate_black: current.winrate_black, score_mean_black: current.score_mean_black };
        let Some(rank) = classify_played_move(Some(&parent), Some(&child), color, settings) else { continue; };
        let sign = if color == PlayerColor::Black { 1.0 } else { -1.0 };
        let winrate_loss = (sign * (previous.winrate_black - current.winrate_black)).max(0.0);
        let score_loss = previous.score_mean_black.zip(current.score_mean_black).map(|(parent, child)| (sign * (parent - child)).max(0.0));
        let severity = match rank {
            MoveRank::Blunder => ProblemSeverity::Blunder,
            MoveRank::Mistake => ProblemSeverity::Mistake,
            MoveRank::Inaccuracy => ProblemSeverity::Inaccuracy,
            _ => ProblemSeverity::Info,
        };
        if severity != ProblemSeverity::Info {
            markers.push(ProblemMarkerDto {
                turn: current.turn,
                severity,
                winrate_loss,
                score_loss,
                label: label_for(severity).to_string(),
            });
        }
    }
    markers
}

fn label_for(severity: ProblemSeverity) -> &'static str {
    match severity {
        ProblemSeverity::Info => "正常波动",
        ProblemSeverity::Inaccuracy => "疑似缓手",
        ProblemSeverity::Mistake => "明显问题手",
        ProblemSeverity::Blunder => "重大失误",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use app_model::AnalysisFrameDto;
    use uuid::Uuid;
    fn f(t: u32, w: f32, s: f32) -> AnalysisFrameDto {
        AnalysisFrameDto {
            job_id: Uuid::new_v4(),
            game_id: None,
            node_id: None,
            turn: t,
            visits: 100,
            winrate_black: w,
            score_mean_black: Some(s),
            score_stdev: None,
            candidates: vec![],
            ownership: None,
            policy: None,
        }
    }
    #[test]
    fn classifies() {
        assert_eq!(
            classify_problem_markers(&[f(0, 0.62, 3.0), f(1, 0.48, -5.0)], &MoveRankPreferencesDto::default()).len(),
            1
        )
    }
}
