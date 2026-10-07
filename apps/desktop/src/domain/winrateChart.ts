import type { NodePath, PlayerColor, SgfPropertyDto, SgfTreeNodeDto } from "./types";
import {
  classifyPlayedMove,
  isBlunderBarRank,
  type DisplayedAnalysis,
  type MoveRank
} from "./moveRank";

export type GraphPerspective = "black" | "sideToPlay";

export type ChartPoint = {
  path: NodePath;
  moveNumber: number;
  isMove: boolean;
  toPlay: PlayerColor;
  analysis: DisplayedAnalysis | null;
};

export type BlunderBar = {
  fromIndex: number;
  toIndex: number;
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
  selectedPath: NodePath;
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
  const handicap = Number(firstPropertyValue(root, "HA"));
  let toPlay: PlayerColor = Number.isFinite(handicap) && handicap >= 2 ? "white" : "black";
  let moveNumber = 0;
  while (node) {
    const played = playedColor(node);
    if (played) {
      moveNumber += 1;
      toPlay = played === "black" ? "white" : "black";
    }
    toPlay = explicitPlayer(node) ?? toPlay;
    points.push({
      path: { indices: [...indices] },
      moveNumber,
      isMove: played != null,
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
  if (!validChartAnalysis(point) || winrate == null || !Number.isFinite(winrate) || winrate < 0 || winrate > 1) return null;
  return flipForPerspective(perspective, selectedToPlay) ? 1 - winrate : winrate;
}

export function displayedScore(point: ChartPoint, perspective: GraphPerspective, selectedToPlay: PlayerColor): number | null {
  const score = point.analysis?.scoreMeanBlack;
  if (!validChartAnalysis(point) || score == null || !Number.isFinite(score)) return null;
  return flipForPerspective(perspective, selectedToPlay) ? -score : score;
}

/** Attached SGF analysis is admitted by the authoritative document owner. Invalid
 * or zero-visit imports must remain gaps, never exportable synthetic values. */
export function validChartAnalysis(point: ChartPoint): boolean {
  return point.analysis != null && Number.isFinite(point.analysis.visits) && point.analysis.visits > 0;
}

export function chartBaseline(model: WinrateChartModel): { visible: boolean; mark: "50%" | "0" | null } {
  const winrate = model.showWinrate && model.points.some((point) => displayedWinrate(point, model.perspective, model.selectedToPlay) != null);
  const score = model.showScore && model.points.some((point) => displayedScore(point, model.perspective, model.selectedToPlay) != null);
  return { visible: winrate || score, mark: winrate === score ? null : winrate ? "50%" : "0" };
}

/** Recover the actual leader from the encoded, whole-series perspective. */
export function scoreLeadText(score: number | null, perspective: GraphPerspective, selectedToPlay: PlayerColor, blackPrefix: string, whitePrefix: string): string | null {
  if (score == null || !Number.isFinite(score)) return null;
  const blackScore = flipForPerspective(perspective, selectedToPlay) ? -score : score;
  const rounded = Math.round((Math.abs(blackScore) + Number.EPSILON) * 10) / 10;
  if (rounded === 0) return "0.0";
  return `${blackScore > 0 ? blackPrefix : whitePrefix}${rounded.toFixed(1)}`;
}

export function buildWinrateChartModel(input: {
  root: SgfTreeNodeDto;
  chosen: Map<string, number>;
  selectedPath: NodePath;
  selectedToPlay: PlayerColor;
  settings: WinrateChartSettings;
}): WinrateChartModel {
  const points = walkSelectedLine(input.root, input.chosen);
  const scoreAvailable = points.some((point) => point.analysis?.scoreMeanBlack != null);
  const series = effectiveChartSeries(input.settings, scoreAvailable);
  const selectedPath = { indices: [...input.selectedPath.indices] };
  const currentMove = points.find((point) => (
    point.path.indices.length === selectedPath.indices.length
    && point.path.indices.every((index, position) => index === selectedPath.indices[position])
  ))?.moveNumber ?? 0;
  return {
    points,
    currentMove,
    selectedPath,
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
  const displayed = points
    .map((point, index) => ({ point, index }))
    .filter(({ point }) => point.analysis);
  const bars: BlunderBar[] = [];
  for (let index = 1; index < displayed.length; index += 1) {
    const parent = displayed[index - 1];
    const child = displayed[index];
    if (!child.point.isMove) continue;
    const rank = classifyPlayedMove(parent.point.analysis, child.point.analysis, parent.point.toPlay);
    if (!rank || !isBlunderBarRank(rank)) continue;
    bars.push({ fromIndex: parent.index, toIndex: child.index, rank });
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

function explicitPlayer(node: SgfTreeNodeDto): PlayerColor | null {
  const value = firstPropertyValue(node, "PL")?.trim().toUpperCase();
  if (value === "B") return "black";
  if (value === "W") return "white";
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
