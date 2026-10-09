//! KataGo 1.18.2 `kata-analyze ... rootInfo true` records.
//! The adapter fixes reportAnalysisWinratesAs=BLACK at launch. Missing optional
//! metrics stay absent; required root/candidate values never come from defaults.
use app_model::{AnalysisFrameDto, AnalysisJobId, CandidateMoveDto, MoveVertex, PointDto};

fn vertex(value: &str, width: u8, height: u8) -> Result<MoveVertex, String> {
    if value.eq_ignore_ascii_case("pass") {
        return Ok(MoveVertex::Pass);
    }
    let bytes = value.as_bytes();
    if bytes.len() < 2 {
        return Err("invalid GTP analysis vertex".into());
    }
    let column = bytes[0].to_ascii_uppercase();
    if !(b'A'..=b'Z').contains(&column) || column == b'I' {
        return Err("invalid GTP analysis column".into());
    }
    let x = column - b'A' - u8::from(column > b'I');
    let row = value[1..].parse::<u8>().map_err(|_| "invalid GTP analysis row")?;
    if x >= width || row == 0 || row > height {
        return Err("GTP analysis vertex outside board".into());
    }
    Ok(MoveVertex::Point(PointDto { x, y: height - row }))
}

fn number(value: &str) -> Result<f32, String> {
    value
        .parse::<f32>()
        .ok()
        .filter(|n| n.is_finite())
        .ok_or_else(|| "non-finite GTP analysis metric".into())
}

#[derive(Default)]
struct Metrics {
    visits: Option<u32>,
    winrate: Option<f32>,
    score: Option<f32>,
    stdev: Option<f32>,
    prior: Option<f32>,
    order: Option<u32>,
    coordinate: Option<MoveVertex>,
    pv: Option<Vec<MoveVertex>>,
}

fn metric_block(tokens: &[&str], width: u8, height: u8) -> Result<Metrics, String> {
    let mut metrics = Metrics::default();
    let mut seen = 0u16;
    let mut i = 0;
    while i < tokens.len() {
        let key = tokens[i];
        i += 1;
        let bit = match key {
            "move" => 1,
            "visits" => 2,
            "winrate" => 4,
            "scoreMean" => 8,
            "scoreStdev" => 16,
            "prior" => 32,
            "order" => 64,
            "pv" => 128,
            _ => 0,
        };
        if seen & bit != 0 {
            return Err("duplicate GTP analysis metric".into());
        }
        seen |= bit;
        if key == "pv" {
            let mut pv = Vec::new();
            while i < tokens.len()
                && (tokens[i].eq_ignore_ascii_case("pass")
                    || tokens[i].as_bytes().first().is_some_and(u8::is_ascii_uppercase))
            {
                pv.push(vertex(tokens[i], width, height)?);
                i += 1;
            }
            if pv.is_empty() {
                return Err("empty GTP candidate PV".into());
            }
            metrics.pv = Some(pv);
            continue;
        }
        let value = *tokens.get(i).ok_or("missing GTP analysis field value")?;
        i += 1;
        match key {
            "move" => metrics.coordinate = Some(vertex(value, width, height)?),
            "visits" => metrics.visits = Some(value.parse().map_err(|_| "invalid GTP visits")?),
            "winrate" => {
                let n = number(value)?;
                if !(0.0..=1.0).contains(&n) {
                    return Err("GTP winrate outside probability range".into());
                }
                metrics.winrate = Some(n);
            }
            "scoreMean" => metrics.score = Some(number(value)?),
            "scoreStdev" => {
                let n = number(value)?;
                if n < 0.0 {
                    return Err("negative GTP score deviation".into());
                }
                metrics.stdev = Some(n);
            }
            "prior" => {
                let n = number(value)?;
                if !(0.0..=1.0).contains(&n) {
                    return Err("GTP prior outside probability range".into());
                }
                metrics.prior = Some(n);
            }
            "order" => metrics.order = Some(value.parse().map_err(|_| "invalid GTP candidate order")?),
            // Named version scalar extensions do not change the admitted fields.
            "edgeVisits" | "utility" | "scoreLead" | "scoreSelfplay" | "lcb" | "utilityLcb" | "weight"
            | "isSymmetryOf" | "rawStWrError" | "rawStScoreError" | "rawVarTimeLeft" => {}
            _ => return Err("unqualified GTP analysis field".into()),
        }
    }
    Ok(metrics)
}

pub fn parse_gtp_analysis(
    line: &str,
    job_id: AnalysisJobId,
    width: u8,
    height: u8,
    turn: u32,
) -> Result<AnalysisFrameDto, String> {
    if line.len() > 64 * 1024 {
        return Err("GTP analysis record exceeds 64 KiB".into());
    }
    if width == 0 || width > 25 || height == 0 {
        return Err("invalid GTP analysis geometry".into());
    }
    let tokens: Vec<_> = line.split_ascii_whitespace().collect();
    let mut candidates = Vec::new();
    let mut root = None;
    let mut ownership = None;
    let mut i = 0;
    while i < tokens.len() {
        let kind = tokens[i];
        i += 1;
        let start = i;
        while i < tokens.len() && !matches!(tokens[i], "info" | "rootInfo" | "ownership") {
            i += 1;
        }
        let block = &tokens[start..i];
        match kind {
            "info" => {
                if candidates.len() >= usize::from(width) * usize::from(height) + 1 {
                    return Err("too many GTP candidates".into());
                }
                let m = metric_block(block, width, height)?;
                candidates.push((
                    m.order.ok_or("missing GTP candidate order")?,
                    CandidateMoveDto {
                        vertex: m.coordinate.ok_or("missing GTP candidate vertex")?,
                        visits: m.visits.ok_or("missing GTP candidate visits")?,
                        winrate_black: m.winrate.ok_or("missing GTP candidate winrate")?,
                        score_mean_black: m.score,
                        policy_prior: m.prior,
                        pv: m.pv.ok_or("missing GTP candidate PV")?,
                    },
                ));
            }
            "rootInfo" if root.is_none() => root = Some(metric_block(block, width, height)?),
            "ownership" if ownership.is_none() => {
                if block.len() != usize::from(width) * usize::from(height) {
                    return Err("GTP ownership geometry mismatch".into());
                }
                let values = block
                    .iter()
                    .map(|value| {
                        let n = number(value)?;
                        if !(-1.0..=1.0).contains(&n) {
                            return Err("GTP ownership outside range".into());
                        }
                        Ok(n)
                    })
                    .collect::<Result<Vec<_>, String>>()?;
                ownership = Some(values);
            }
            _ => return Err("unqualified or duplicate GTP record section".into()),
        }
    }
    let root = root.ok_or("GTP analysis omitted required rootInfo")?;
    let visits = root
        .visits
        .filter(|v| *v > 0)
        .ok_or("GTP root visits absent or zero")?;
    if candidates.is_empty() {
        return Err("GTP candidates absent".into());
    }
    candidates.sort_by_key(|(order, _)| *order);
    if candidates.windows(2).any(|pair| pair[0].0 == pair[1].0) {
        return Err("duplicate GTP candidate order".into());
    }
    Ok(AnalysisFrameDto {
        job_id,
        game_id: None,
        node_id: None,
        turn,
        visits,
        winrate_black: root.winrate.ok_or("GTP root winrate absent")?,
        score_mean_black: root.score,
        score_stdev: root.stdev,
        candidates: candidates.into_iter().map(|(_, candidate)| candidate).collect(),
        ownership,
        policy: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    const RAW: &str = include_str!("../tests/fixtures/katago-1.18.2-gtp-rich.txt");
    const ABSENT: &str = include_str!("../tests/fixtures/katago-1.18.2-gtp-no-ownership.txt");

    #[test]
    fn named_white_to_play_capture_has_black_metrics_and_exact_geometry() {
        let frame = parse_gtp_analysis(RAW, AnalysisJobId::nil(), 9, 9, 7).unwrap();
        assert_eq!(frame.turn, 7);
        assert_eq!(frame.visits, 2);
        assert!((frame.winrate_black - 0.000496895).abs() < 1e-8);
        assert_eq!(frame.score_mean_black, Some(-13.3791));
        assert_eq!(
            frame.candidates[0].vertex,
            MoveVertex::Point(PointDto { x: 3, y: 4 })
        );
        assert!(!frame.candidates[0].pv.is_empty());
        assert_eq!(frame.ownership.as_ref().unwrap().len(), 81);
        assert_eq!(frame.policy, None);
    }

    #[test]
    fn optional_absence_is_not_zero_or_fabricated_score() {
        let frame = parse_gtp_analysis(ABSENT, AnalysisJobId::nil(), 9, 9, 0).unwrap();
        assert_eq!(frame.ownership, None);
        let minimal = "info move pass visits 2 winrate 0.6 order 0 pv pass rootInfo visits 2 winrate 0.6";
        let frame = parse_gtp_analysis(minimal, AnalysisJobId::nil(), 9, 9, 0).unwrap();
        assert_eq!(frame.score_mean_black, None);
        assert_eq!(frame.score_stdev, None);
        assert_eq!(frame.candidates[0].score_mean_black, None);
        assert_eq!(frame.candidates[0].policy_prior, None);
    }

    #[test]
    fn controls_errors_terminal_and_invalid_records_are_not_analysis() {
        for line in include_str!("../tests/fixtures/katago-1.18.2-gtp-control.txt").lines() {
            assert!(parse_gtp_analysis(line, AnalysisJobId::nil(), 9, 9, 0).is_err());
        }
        for line in [
            "=13",
            "?13 invalid argument",
            "play D5",
            "play cancelled",
            "",
            "info move I5 visits 2 winrate 0.5 order 0 pv D5 rootInfo visits 2 winrate 0.5",
            "info move D5 visits 2 winrate NaN order 0 pv D5 rootInfo visits 2 winrate 0.5",
        ] {
            assert!(
                parse_gtp_analysis(line, AnalysisJobId::nil(), 9, 9, 0).is_err(),
                "{line}"
            );
        }
        assert!(parse_gtp_analysis(RAW, AnalysisJobId::nil(), 13, 9, 0).is_err());
    }
}
