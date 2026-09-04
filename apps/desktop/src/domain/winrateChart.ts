import type { PlayerColor, SgfPropertyDto, SgfTreeNodeDto } from "./types";
import {
  classifyPlayedMove,
  isBlunderBarRank,
  type DisplayedAnalysis,
  type MoveRank
} from "./moveRank";

export type GraphPerspective = "black" | "sideToPlay";

export type ChartPoint = {
  moveNumber: number;
  toPlay: PlayerColor;
  analysis: DisplayedAnalysis | null;
};

export type BlunderBar = {
  fromMove: number;
  toMove: number;
  rank: Extract<MoveRank, "inaccuracy" | "mistake" | "blunder">;
};

export type WinrateChartSettings = {
  graphPerspective: GraphPerspective;
  winrateLine: boolean;
  scoreLeadLine: boolean;
  blunderBar: boolean;
  graphHover: boolean;
  scoreLeadScale: number;
};

export type WinrateChartModel = {
  points: ChartPoint[];
  currentMove: number;
  selectedToPlay: PlayerColor;
  perspective: GraphPerspective;
  scoreAvailable: boolean;
  showWinrate: boolean;
  showScore: boolean;
  scoreScale: number;
  bars: BlunderBar[];
  hoverEnabled: boolean;
};

const PRIMARY_KEYS_ROOT = ["LZOP", "LZ"];
const PRIMARY_KEYS_NODE = ["LZ", "LZOP"];

export function displayedPrimaryAnalysis(node: SgfTreeNodeDto, isRoot: boolean): DisplayedAnalysis | null {
  const keys = isRoot ? PRIMARY_KEYS_ROOT : PRIMARY_KEYS_NODE;
  for (const key of keys) {
    const raw = firstPropertyValue(node, key);
    if (raw == null) continue;
    const parsed = parseDisplayedAnalysis(raw);
    if (parsed) return parsed;
  }
  return null;
}

export function parseDisplayedAnalysis(raw: string): DisplayedAnalysis | null {
  const trimmed = raw.trim();
  if (!trimmed) return null;
  const newline = trimmed.indexOf("\n");
  const headerLine = (newline === -1 ? trimmed : trimmed.slice(0, newline)).trim();
  const detail = newline === -1 ? "" : trimmed.slice(newline + 1);
  const header = headerLine.split(/\s+/).filter(Boolean);
  if (header.length < 3 || !header[0]) return null;
  const whiteWinrate = Number(header[1]);
  const visits = parsePlayouts(header[2]);
  if (!Number.isFinite(whiteWinrate) || visits <= 0) return null;
  if (!/\bmove\b/i.test(detail) && !/\bmove\b/i.test(headerLine)) return null;
  const scoreToken = header[3];
  const scoreMeanBlack = scoreToken == null || scoreToken === "" ? undefined : Number(scoreToken);
  return {
    visits,
    winrateBlack: (100 - whiteWinrate) / 100,
    scoreMeanBlack: scoreMeanBlack != null && Number.isFinite(scoreMeanBlack) ? scoreMeanBlack : undefined
  };
}

export function walkSelectedLine(root: SgfTreeNodeDto, chosen: Map<string, number>): ChartPoint[] {
  const points: ChartPoint[] = [];
  let node: SgfTreeNodeDto | undefined = root;
  const indices: number[] = [];
  let toPlay: PlayerColor = "black";
  let moveNumber = 0;
  while (node) {
    const played = playedColor(node);
    if (played) {
      moveNumber += 1;
      toPlay = played === "black" ? "white" : "black";
    }
    points.push({
      moveNumber,
      toPlay,
      analysis: displayedPrimaryAnalysis(node, indices.length === 0)
    });
    if (node.children.length === 0) break;
    const childIndex = chosenChildIndex(chosen, indices, node.children.length);
    indices.push(childIndex);
    node = node.children[childIndex];
  }
  return points;
}

export function effectiveChartSeries(
  settings: Pick<WinrateChartSettings, "winrateLine" | "scoreLeadLine">,
  scoreAvailable: boolean
): { showWinrate: boolean; showScore: boolean } {
  const showScore = settings.scoreLeadLine && scoreAvailable;
  const showWinrate = settings.winrateLine || !showScore;
  return { showWinrate, showScore };
}

export function admitsChartSeriesChange(
  settings: Pick<WinrateChartSettings, "winrateLine" | "scoreLeadLine">,
  patch: Partial<Pick<WinrateChartSettings, "winrateLine" | "scoreLeadLine">>,
  scoreAvailable: boolean
): boolean {
  const next = { ...settings, ...patch };
  return next.winrateLine || (next.scoreLeadLine && scoreAvailable);
}

export function parsePositiveScoreLeadScale(value: unknown): number | null {
  const parsed = typeof value === "number" ? value : Number(value);
  if (!Number.isFinite(parsed) || parsed <= 0) return null;
  return Math.min(1000, Math.max(1, Math.floor(parsed)));
}

export function sessionScoreLeadScale(persistedFloor: number, points: ChartPoint[]): number {
  let peak = persistedFloor;
  for (const point of points) {
    const score = point.analysis?.scoreMeanBlack;
    if (score == null) continue;
    peak = Math.max(peak, Math.abs(score));
  }
  return peak;
}

export function displayedWinrate(point: ChartPoint, perspective: GraphPerspective, selectedToPlay: PlayerColor): number | null {
  const winrate = point.analysis?.winrateBlack;
  if (winrate == null) return null;
  return flipForPerspective(perspective, selectedToPlay) ? 1 - winrate : winrate;
}

export function displayedScore(point: ChartPoint, perspective: GraphPerspective, selectedToPlay: PlayerColor): number | null {
  const score = point.analysis?.scoreMeanBlack;
  if (score == null) return null;
  return flipForPerspective(perspective, selectedToPlay) ? -score : score;
}

export function buildWinrateChartModel(input: {
  root: SgfTreeNodeDto;
  chosen: Map<string, number>;
  selectedPathLength: number;
  selectedToPlay: PlayerColor;
  settings: WinrateChartSettings;
}): WinrateChartModel {
  const points = walkSelectedLine(input.root, input.chosen);
  const scoreAvailable = points.some((point) => point.analysis?.scoreMeanBlack != null);
  const series = effectiveChartSeries(input.settings, scoreAvailable);
  const currentMove = Math.min(
    Math.max(input.selectedPathLength, 0),
    points.at(-1)?.moveNumber ?? 0
  );
  return {
    points,
    currentMove,
    selectedToPlay: input.selectedToPlay,
    perspective: input.settings.graphPerspective,
    scoreAvailable,
    showWinrate: series.showWinrate,
    showScore: series.showScore,
    scoreScale: sessionScoreLeadScale(input.settings.scoreLeadScale, points),
    bars: input.settings.blunderBar ? blunderBars(points) : [],
    hoverEnabled: input.settings.graphHover
  };
}

export function blunderBars(points: ChartPoint[]): BlunderBar[] {
  const displayed = points.filter((point) => point.analysis);
  const bars: BlunderBar[] = [];
  for (let index = 1; index < displayed.length; index += 1) {
    const parent = displayed[index - 1];
    const child = displayed[index];
    const rank = classifyPlayedMove(parent.analysis, child.analysis, parent.toPlay);
    if (!rank || !isBlunderBarRank(rank)) continue;
    bars.push({ fromMove: parent.moveNumber, toMove: child.moveNumber, rank });
  }
  return bars;
}

function flipForPerspective(perspective: GraphPerspective, selectedToPlay: PlayerColor): boolean {
  return perspective === "sideToPlay" && selectedToPlay === "white";
}

function firstPropertyValue(node: SgfTreeNodeDto, key: string): string | undefined {
  const property = node.properties.find((entry: SgfPropertyDto) => entry.key === key);
  return property?.values[0];
}

function playedColor(node: SgfTreeNodeDto): PlayerColor | null {
  for (const property of node.properties) {
    if (property.key === "B") return "black";
    if (property.key === "W") return "white";
  }
  return null;
}

function pathKey(indices: number[]): string {
  return indices.join(",");
}

function chosenChildIndex(chosen: Map<string, number>, indices: number[], childCount: number): number {
  const remembered = chosen.get(pathKey(indices));
  if (remembered !== undefined && remembered >= 0 && remembered < childCount) return remembered;
  return 0;
}

function parsePlayouts(raw: string): number {
  const normalized = raw.trim().toLowerCase().replace(/,/g, "");
  if (!normalized) return 0;
  let number = normalized;
  let multiplier = 1;
  if (normalized.endsWith("m")) {
    number = normalized.slice(0, -1);
    multiplier = 1_000_000;
  } else if (normalized.endsWith("k")) {
    number = normalized.slice(0, -1);
    multiplier = 1_000;
  }
  const cleaned = number.replace(/[^0-9.+-]/g, "");
  const parsed = Number(cleaned);
  if (!Number.isFinite(parsed) || parsed <= 0) return 0;
  return Math.round(parsed * multiplier);
}
