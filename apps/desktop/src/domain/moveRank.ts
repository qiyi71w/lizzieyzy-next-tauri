export type MoveRank = "best" | "good" | "normal" | "inaccuracy" | "mistake" | "blunder";
export type PlayerColor = "black" | "white";

export type DisplayedAnalysis = {
  visits: number;
  winrateBlack: number;
  scoreMeanBlack?: number;
};

const WINRATE_THRESHOLDS_PP = [1, 3, 6, 12, 24];
const SCORE_THRESHOLDS = [0.5, 1.5, 3, 6, 12];
const MISTAKE_LEVEL = 3;
const RANKS: MoveRank[] = ["best", "good", "normal", "inaccuracy", "mistake", "blunder"];

export function classifyAutoMoveRank(winrateLossPp: number, scoreLoss?: number | null): MoveRank {
  const winrateLoss = Math.max(0, winrateLossPp);
  const boundedScore = scoreLoss == null || !Number.isFinite(scoreLoss) ? null : Math.max(0, scoreLoss);
  for (let level = 5; level >= 1; level -= 1) {
    if (reachesAutoThreshold(winrateLoss, boundedScore, level)) return RANKS[level];
  }
  return "best";
}

export function classifyPlayedMove(
  parent: DisplayedAnalysis | null | undefined,
  child: DisplayedAnalysis | null | undefined,
  parentToPlay: PlayerColor
): MoveRank | null {
  const from = displayed(parent);
  const to = displayed(child);
  if (!from || !to) return null;
  const winrateLossPp = playerWinratePp(from, parentToPlay) - playerWinratePp(to, parentToPlay);
  const scoreLoss =
    from.scoreMeanBlack == null || to.scoreMeanBlack == null
      ? null
      : playerScore(from.scoreMeanBlack, parentToPlay) - playerScore(to.scoreMeanBlack, parentToPlay);
  return classifyAutoMoveRank(winrateLossPp, scoreLoss);
}

export function isBlunderBarRank(rank: MoveRank): rank is "inaccuracy" | "mistake" | "blunder" {
  return rank === "inaccuracy" || rank === "mistake" || rank === "blunder";
}

function displayed(analysis: DisplayedAnalysis | null | undefined): DisplayedAnalysis | null {
  if (!analysis || analysis.visits <= 0) return null;
  return analysis;
}

function playerWinratePp(analysis: DisplayedAnalysis, color: PlayerColor): number {
  return color === "black" ? analysis.winrateBlack * 100 : (1 - analysis.winrateBlack) * 100;
}

function playerScore(scoreMeanBlack: number, color: PlayerColor): number {
  return color === "black" ? scoreMeanBlack : -scoreMeanBlack;
}

function reachesAutoThreshold(winrateLossPp: number, scoreLoss: number | null, level: number): boolean {
  const reachesWinrate = winrateLossPp >= WINRATE_THRESHOLDS_PP[level - 1];
  const reachesScore = scoreLoss != null && scoreLoss >= SCORE_THRESHOLDS[level - 1];
  if (scoreLoss == null) return reachesWinrate;
  return reachesScore || (level > MISTAKE_LEVEL && reachesWinrate);
}
