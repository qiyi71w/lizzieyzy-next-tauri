import type { NextMoveReviewMarkerMode } from "./nextMoveReviewMarker";
import type { AnalysisStageConditionsDto, ContinuousAnalysisBudgetDto } from "./types";

export type ReviewMode = "quick" | "deep";
export type BoardTheme = "classic" | "high-contrast";
export type GraphPerspective = "black" | "sideToPlay";
export type SubBoardContentMode = "variation" | "raw";
export type { NextMoveReviewMarkerMode };

export type AppPreferences = ContinuousAnalysisBudgetDto & {
  showOwnership: boolean;
  showPolicy: boolean;
  showCandidates: boolean;
  candidateLimit: number;
  defaultMaxVisits: number;
  taskConditions: AnalysisStageConditionsDto;
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
  continuousAnalysisEnabled: boolean;
};

export const defaultAppPreferences: AppPreferences = {
  showOwnership: true,
  showPolicy: true,
  showCandidates: true,
  candidateLimit: 8,
  defaultMaxVisits: 800,
  taskConditions: {
    time_seconds: { enabled: false, value: 10 },
    total_visits: { enabled: true, value: 800 },
    leading_candidate_visits: { enabled: false, value: 500 }
  },
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
  restoreLastSession: false,
  continuousAnalysisEnabled: true,
  continuousTimeLimitEnabled: true,
  continuousTimeLimitSeconds: 600,
  continuousVisitsLimitEnabled: false,
  continuousVisitsLimit: 100000,
  continuousStopOnEmptyBoard: false
};

export function normalizeAppPreferences(value: Partial<AppPreferences> | null | undefined): AppPreferences {
  const winrateLine = booleanValue(value?.winrateLine, defaultAppPreferences.winrateLine);
  const scoreLeadLine = booleanValue(value?.scoreLeadLine, defaultAppPreferences.scoreLeadLine);
  const defaultMaxVisits = integerValue(value?.defaultMaxVisits, defaultAppPreferences.defaultMaxVisits, 1, 1_000_000);
  return {
    showOwnership: booleanValue(value?.showOwnership, defaultAppPreferences.showOwnership),
    showPolicy: booleanValue(value?.showPolicy, defaultAppPreferences.showPolicy),
    showCandidates: booleanValue(value?.showCandidates, defaultAppPreferences.showCandidates),
    candidateLimit: integerValue(value?.candidateLimit, defaultAppPreferences.candidateLimit, 1, 20),
    defaultMaxVisits,
    taskConditions: normalizeTaskConditions(value?.taskConditions, defaultMaxVisits),
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
    restoreLastSession: booleanValue(value?.restoreLastSession, defaultAppPreferences.restoreLastSession),
    continuousAnalysisEnabled: booleanValue(value?.continuousAnalysisEnabled, defaultAppPreferences.continuousAnalysisEnabled),
    continuousTimeLimitEnabled: booleanValue(value?.continuousTimeLimitEnabled, defaultAppPreferences.continuousTimeLimitEnabled),
    continuousTimeLimitSeconds: value?.continuousTimeLimitSeconds ?? defaultAppPreferences.continuousTimeLimitSeconds,
    continuousVisitsLimitEnabled: booleanValue(value?.continuousVisitsLimitEnabled, defaultAppPreferences.continuousVisitsLimitEnabled),
    continuousVisitsLimit: value?.continuousVisitsLimit ?? defaultAppPreferences.continuousVisitsLimit,
    continuousStopOnEmptyBoard: booleanValue(value?.continuousStopOnEmptyBoard, defaultAppPreferences.continuousStopOnEmptyBoard)
  };
}

export function continuousBudgetError(value: ContinuousAnalysisBudgetDto): string | null {
  for (const limit of [value.continuousTimeLimitSeconds, value.continuousVisitsLimit]) {
    if (!Number.isInteger(limit) || limit < 1 || limit > 4294967295) {
      return "连续分析时间和 visits 上限须为 1–4294967295 的整数；关闭上限会保留原数值。";
    }
  }
  return null;
}

export function taskConditionsError(value: AnalysisStageConditionsDto): string | null {
  const limits = [value.time_seconds, value.total_visits, value.leading_candidate_visits];
  if (limits.some((limit) => !Number.isInteger(limit.value) || limit.value < 1 || limit.value > 4294967295)) {
    return "Task condition values must be whole numbers from 1 to 4294967295; disabled conditions retain their values.";
  }
  if (!limits.some((limit) => limit.enabled)) {
    return "Enable at least one task ending condition.";
  }
  return null;
}

function normalizeTaskConditions(
  value: AnalysisStageConditionsDto | undefined,
  legacyDefaultMaxVisits: number
): AnalysisStageConditionsDto {
  if (!value) {
    return {
      time_seconds: { enabled: false, value: 10 },
      total_visits: { enabled: true, value: legacyDefaultMaxVisits },
      leading_candidate_visits: { enabled: false, value: 500 }
    };
  }
  return {
    time_seconds: normalizeTaskLimit(value.time_seconds),
    total_visits: normalizeTaskLimit(value.total_visits),
    leading_candidate_visits: normalizeTaskLimit(value.leading_candidate_visits)
  };
}

function normalizeTaskLimit(value: AnalysisStageConditionsDto["time_seconds"] | undefined) {
  return {
    enabled: typeof value?.enabled === "boolean" ? value.enabled : false,
    value: typeof value?.value === "number" ? value.value : Number.NaN
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
