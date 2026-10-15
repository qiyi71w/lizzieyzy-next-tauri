import type { AnalysisFrameDto, GameDto, MoveRankPreferencesDto, ProblemMarkerDto } from "./types";

export const defaultMoveRankPreferences: MoveRankPreferencesDto = {
  mode: "auto",
  winrateThresholdsPp: [1, 3, 6, 12, 24],
  scoreThresholds: [0.5, 1.5, 3, 6, 12]
};

export function validMoveRankPreferences(value: unknown): value is MoveRankPreferencesDto {
  if (!value || typeof value !== "object") return false;
  const settings = value as MoveRankPreferencesDto;
  return ["auto", "score", "winrate", "combined"].includes(settings.mode)
    && [settings.winrateThresholdsPp, settings.scoreThresholds].every((values) =>
      Array.isArray(values) && values.length === 5 && values.every((value, index) =>
        Number.isFinite(value) && (index === 0 || values[index - 1] <= value)));
}

export function readMoveRankPreferences(value: unknown): MoveRankPreferencesDto {
  const settings = validMoveRankPreferences(value) ? value : defaultMoveRankPreferences;
  return { mode: settings.mode, winrateThresholdsPp: [...settings.winrateThresholdsPp], scoreThresholds: [...settings.scoreThresholds] };
}

export type MoveRank = "best" | "good" | "normal" | "inaccuracy" | "mistake" | "blunder";
export type PlayerColor = "black" | "white";

export type DisplayedAnalysis = {
  visits: number;
  winrateBlack: number;
  scoreMeanBlack?: number;
};

const RANKS: MoveRank[] = ["best", "good", "normal", "inaccuracy", "mistake", "blunder"];

export function classifyMoveRank(winrateLossPp: number, scoreLoss?: number | null, settings: MoveRankPreferencesDto = defaultMoveRankPreferences): MoveRank {
  const winrateLoss = Math.max(0, winrateLossPp);
  const boundedScore = scoreLoss == null || !Number.isFinite(scoreLoss) ? null : Math.max(0, scoreLoss);
  for (let level = 5; level >= 1; level -= 1) {
    if (reachesThreshold(winrateLoss, boundedScore, level, settings)) return RANKS[level];
  }
  return "best";
}

export function classifyPlayedMove(
  parent: DisplayedAnalysis | null | undefined,
  child: DisplayedAnalysis | null | undefined,
  parentToPlay: PlayerColor,
  settings: MoveRankPreferencesDto = defaultMoveRankPreferences
): MoveRank | null {
  const from = displayed(parent);
  const to = displayed(child);
  if (!from || !to) return null;
  const winrateLossPp = playerWinratePp(from, parentToPlay) - playerWinratePp(to, parentToPlay);
  const scoreLoss =
    from.scoreMeanBlack == null || to.scoreMeanBlack == null
      ? null
      : playerScore(from.scoreMeanBlack, parentToPlay) - playerScore(to.scoreMeanBlack, parentToPlay);
  return classifyMoveRank(winrateLossPp, scoreLoss, settings);
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

function reachesThreshold(winrateLossPp: number, scoreLoss: number | null, level: number, settings: MoveRankPreferencesDto): boolean {
  const reachesWinrate = winrateLossPp >= settings.winrateThresholdsPp[level - 1];
  const reachesScore = scoreLoss != null && scoreLoss >= settings.scoreThresholds[level - 1];
  switch (settings.mode) {
    case "auto": return scoreLoss == null ? reachesWinrate : reachesScore || (level > 3 && reachesWinrate);
    case "score": return reachesScore;
    case "winrate": return reachesWinrate;
    case "combined": return reachesScore || reachesWinrate;
  }
}

export function classifyProblemFrames(frames: AnalysisFrameDto[], settings: MoveRankPreferencesDto, moves: GameDto["moves"]): ProblemMarkerDto[] {
  const problems: ProblemMarkerDto[] = [];
  for (let index = 1; index < frames.length; index += 1) {
    const parent = frames[index - 1];
    const child = frames[index];
    const color = moves[parent.turn]?.color;
    if (!color) continue;
    const from = { visits: parent.visits, winrateBlack: parent.winrate_black, scoreMeanBlack: parent.score_mean_black ?? undefined };
    const to = { visits: child.visits, winrateBlack: child.winrate_black, scoreMeanBlack: child.score_mean_black ?? undefined };
    const rank = classifyPlayedMove(from, to, color, settings);
    if (!rank || !isBlunderBarRank(rank)) continue;
    const sign = color === "black" ? 1 : -1;
    problems.push({ turn: child.turn, severity: rank, label: rank,
      winrate_loss: Math.max(0, sign * (parent.winrate_black - child.winrate_black)),
      score_loss: parent.score_mean_black == null || child.score_mean_black == null ? null : Math.max(0, sign * (parent.score_mean_black - child.score_mean_black)) });
  }
  return problems;
}
