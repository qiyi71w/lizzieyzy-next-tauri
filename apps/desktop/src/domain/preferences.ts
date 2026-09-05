import type { NextMoveReviewMarkerMode } from "./nextMoveReviewMarker";

export type ReviewMode = "quick" | "deep";
export type BoardTheme = "classic" | "high-contrast";
export type GraphPerspective = "black" | "sideToPlay";
export type SubBoardContentMode = "variation" | "raw";
export type { NextMoveReviewMarkerMode };

export type AppPreferences = {
  showOwnership: boolean;
  showPolicy: boolean;
  showCandidates: boolean;
  candidateLimit: number;
  defaultMaxVisits: number;
  reviewMode: ReviewMode;
  boardTheme: BoardTheme;
  graphPerspective: GraphPerspective;
  winrateLine: boolean;
  scoreLeadLine: boolean;
  blunderBar: boolean;
  graphHover: boolean;
  scoreLeadScale: number;
  nextMoveReviewMarker: NextMoveReviewMarkerMode;
  subBoardContentMode: SubBoardContentMode;
  variationReplayEnabled: boolean;
  variationReplayIntervalMs: number;
  restoreLastSession: boolean;
};

export const defaultAppPreferences: AppPreferences = {
  showOwnership: true,
  showPolicy: true,
  showCandidates: true,
  candidateLimit: 8,
  defaultMaxVisits: 800,
  reviewMode: "quick",
  boardTheme: "classic",
  graphPerspective: "black",
  winrateLine: true,
  scoreLeadLine: true,
  blunderBar: false,
  graphHover: true,
  scoreLeadScale: 15,
  nextMoveReviewMarker: "variations",
  subBoardContentMode: "variation",
  variationReplayEnabled: false,
  variationReplayIntervalMs: 500,
  restoreLastSession: false
};

export function normalizeAppPreferences(value: Partial<AppPreferences> | null | undefined): AppPreferences {
  const winrateLine = booleanValue(value?.winrateLine, defaultAppPreferences.winrateLine);
  const scoreLeadLine = booleanValue(value?.scoreLeadLine, defaultAppPreferences.scoreLeadLine);
  return {
    showOwnership: booleanValue(value?.showOwnership, defaultAppPreferences.showOwnership),
    showPolicy: booleanValue(value?.showPolicy, defaultAppPreferences.showPolicy),
    showCandidates: booleanValue(value?.showCandidates, defaultAppPreferences.showCandidates),
    candidateLimit: integerValue(value?.candidateLimit, defaultAppPreferences.candidateLimit, 1, 20),
    defaultMaxVisits: integerValue(value?.defaultMaxVisits, defaultAppPreferences.defaultMaxVisits, 1, 1_000_000),
    reviewMode: value?.reviewMode === "deep" ? "deep" : "quick",
    boardTheme: value?.boardTheme === "high-contrast" ? "high-contrast" : "classic",
    graphPerspective: value?.graphPerspective === "sideToPlay" ? "sideToPlay" : "black",
    winrateLine: winrateLine || !scoreLeadLine,
    scoreLeadLine,
    blunderBar: booleanValue(value?.blunderBar, defaultAppPreferences.blunderBar),
    graphHover: booleanValue(value?.graphHover, defaultAppPreferences.graphHover),
    scoreLeadScale: positiveFloor(value?.scoreLeadScale, defaultAppPreferences.scoreLeadScale, 1, 1000),
    nextMoveReviewMarker: nextMoveReviewMarkerValue(value?.nextMoveReviewMarker),
    subBoardContentMode: value?.subBoardContentMode === "raw" ? "raw" : "variation",
    variationReplayEnabled: booleanValue(value?.variationReplayEnabled, defaultAppPreferences.variationReplayEnabled),
    variationReplayIntervalMs: integerValue(
      value?.variationReplayIntervalMs,
      defaultAppPreferences.variationReplayIntervalMs,
      100,
      5000
    ),
    restoreLastSession: booleanValue(value?.restoreLastSession, defaultAppPreferences.restoreLastSession)
  };
}

function nextMoveReviewMarkerValue(value: unknown): NextMoveReviewMarkerMode {
  return value === "off" || value === "graded" ? value : "variations";
}

function booleanValue(value: unknown, fallback: boolean): boolean {
  return typeof value === "boolean" ? value : fallback;
}

function integerValue(value: unknown, fallback: number, min: number, max: number): number {
  const parsed = typeof value === "number" ? value : Number(value);
  if (!Number.isFinite(parsed)) return fallback;
  return Math.min(max, Math.max(min, Math.floor(parsed)));
}

function positiveFloor(value: unknown, fallback: number, min: number, max: number): number {
  const parsed = typeof value === "number" ? value : Number(value);
  if (!Number.isFinite(parsed) || parsed <= 0) return fallback;
  return Math.min(max, Math.max(min, Math.floor(parsed)));
}
