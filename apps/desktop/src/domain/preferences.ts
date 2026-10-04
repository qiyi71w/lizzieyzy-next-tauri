import type { NextMoveReviewMarkerMode } from "./nextMoveReviewMarker";
import type { AnalysisStageConditionsDto, AnalysisSwingCriteriaDto, ContinuousAnalysisBudgetDto, MatchDefaultsDto, PkSideDefaultsDto, WindowGeometryDto, WorkspaceSharesDto } from "./types";

export type ReviewMode = "quick" | "deep";
export type BoardTheme = "classic" | "high-contrast";
export type GraphPerspective = "black" | "sideToPlay";
export type SubBoardContentMode = "variation" | "raw";
export type ScoringRule = "area" | "territory";
export type { NextMoveReviewMarkerMode };

import type { WorkspaceVisibilityDto } from "./types";

export type AppPreferences = ContinuousAnalysisBudgetDto & {
  matchDefaults: MatchDefaultsDto;
  workspaceShares: WorkspaceSharesDto | null;
  windowGeometry: WindowGeometryDto | null;
  workspaceVisibility: WorkspaceVisibilityDto;
  mainWindowAlwaysOnTop: boolean;
  showCoordinates: boolean;
  showMoveNumbers: boolean;
  showOwnership: boolean;
  showPolicy: boolean;
  showCandidates: boolean;
  candidateLimit: number;
  defaultMaxVisits: number;
  taskSingleStageConditions: AnalysisStageConditionsDto;
  taskOverviewConditions: AnalysisStageConditionsDto;
  taskDeepConditions: AnalysisStageConditionsDto;
  taskSwingOverviewConditions: AnalysisStageConditionsDto;
  taskSwingDeepConditions: AnalysisStageConditionsDto;
  taskSwingCriteria: AnalysisSwingCriteriaDto;
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
  soundEnabled: boolean;
  scoringRule: ScoringRule;
  continuousAnalysisEnabled: boolean;
  defaultBoardWidth: number;
  defaultBoardHeight: number;
  defaultKomi: number;
  recentGamePaths: string[];
};

export const defaultAppPreferences: AppPreferences = {
  matchDefaults: { board_size: 19, komi: 7.5, handicap: 0, human_color: "black", rules: null, profile_id: null, deadline_ms: 30_000, kata_max_visits: 800,
    pk_black: { profile_id: null, deadline_ms: 30_000, kata_max_visits: 800 },
    pk_white: { profile_id: null, deadline_ms: 30_000, kata_max_visits: 800 }, pk_max_moves: 450 },
  workspaceShares: null,
  windowGeometry: null,
  workspaceVisibility: { left: true, right: true },
  mainWindowAlwaysOnTop: false,
  showCoordinates: true,
  showMoveNumbers: false,
  showOwnership: true,
  showPolicy: true,
  showCandidates: true,
  candidateLimit: 8,
  defaultMaxVisits: 800,
  defaultBoardWidth: 19,
  defaultBoardHeight: 19,
  defaultKomi: 7.5,
  taskSingleStageConditions: {
    time_seconds: { enabled: false, value: 10 },
    total_visits: { enabled: true, value: 800 },
    leading_candidate_visits: { enabled: false, value: 500 }
  },
  taskOverviewConditions: {
    time_seconds: { enabled: false, value: 10 },
    total_visits: { enabled: true, value: 32 },
    leading_candidate_visits: { enabled: false, value: 32 }
  },
  taskDeepConditions: {
    time_seconds: { enabled: false, value: 10 },
    total_visits: { enabled: true, value: 800 },
    leading_candidate_visits: { enabled: false, value: 500 }
  },
  taskSwingOverviewConditions: {
    time_seconds: { enabled: false, value: 10 },
    total_visits: { enabled: true, value: 32 },
    leading_candidate_visits: { enabled: false, value: 32 }
  },
  taskSwingDeepConditions: {
    time_seconds: { enabled: true, value: 10 },
    total_visits: { enabled: false, value: 800 },
    leading_candidate_visits: { enabled: false, value: 500 }
  },
  taskSwingCriteria: {
    move_actors: "both",
    winrate_change_percentage_points: { enabled: true, value: 10 },
    score_change_points: { enabled: false, value: 3 }
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
  soundEnabled: true,
  scoringRule: "area",
  continuousAnalysisEnabled: true,
  recentGamePaths: [],
  continuousTimeLimitEnabled: true,
  continuousTimeLimitSeconds: 600,
  continuousVisitsLimitEnabled: false,
  continuousVisitsLimit: 100000,
  continuousStopOnEmptyBoard: false
};

type StoredMatchDefaults = Omit<Partial<MatchDefaultsDto>, "pk_black" | "pk_white"> & {
  pk_black?: Partial<PkSideDefaultsDto>;
  pk_white?: Partial<PkSideDefaultsDto>;
};
type StoredAppPreferences = Omit<Partial<AppPreferences>, "matchDefaults"> & {
  matchDefaults?: StoredMatchDefaults;
  taskConditions?: AnalysisStageConditionsDto;
};

export function normalizeWorkspaceShares(value: WorkspaceSharesDto | null | undefined): WorkspaceSharesDto | null {
  if (!value || !Number.isFinite(value.left) || !Number.isFinite(value.right)
    || value.left < 0 || value.right < 0 || value.left + value.right >= 1) return null;
  return { left: value.left, right: value.right };
}

function normalizeWindowGeometry(value: WindowGeometryDto | null | undefined): WindowGeometryDto | null {
  if (!value || !((value.x === null && value.y === null)
    || (Number.isFinite(value.x) && Number.isFinite(value.y)))
    || !Number.isFinite(value.width) || !Number.isFinite(value.height) || !Number.isFinite(value.scaleFactor)
    || value.width <= 0 || value.height <= 0 || value.scaleFactor <= 0 || typeof value.maximized !== "boolean") return null;
  return { x: value.x, y: value.y, width: value.width, height: value.height, scaleFactor: value.scaleFactor, maximized: value.maximized };
}

function normalizePkSide(value: Partial<PkSideDefaultsDto> | undefined): PkSideDefaultsDto {
  return {
    profile_id: typeof value?.profile_id === "string" && value.profile_id.trim() ? value.profile_id : null,
    deadline_ms: integerValue(value?.deadline_ms, 30_000, 1, 4_294_967_295),
    kata_max_visits: integerValue(value?.kata_max_visits, 800, 1, 4_294_967_295)
  };
}

function normalizeMatchDefaults(value: StoredMatchDefaults | undefined): MatchDefaultsDto {
  const fallback = defaultAppPreferences.matchDefaults;
  return {
    board_size: integerValue(value?.board_size, fallback.board_size, 2, 19),
    komi: typeof value?.komi === "number" && Number.isFinite(value.komi) && Number.isInteger(value.komi * 2) ? value.komi : fallback.komi,
    handicap: typeof value?.handicap === "number" && (value.handicap === 0 || (Number.isInteger(value.handicap) && value.handicap >= 2 && value.handicap <= 9)) ? value.handicap : 0,
    human_color: value?.human_color === "white" ? "white" : "black",
    rules: value?.rules === "chinese" || value?.rules === "chinese_kgs" ? value.rules : null,
    profile_id: typeof value?.profile_id === "string" && value.profile_id.trim() ? value.profile_id : null,
    deadline_ms: integerValue(value?.deadline_ms, fallback.deadline_ms, 1, 4_294_967_295),
    kata_max_visits: integerValue(value?.kata_max_visits, fallback.kata_max_visits, 1, 4_294_967_295),
    pk_black: normalizePkSide(value?.pk_black),
    pk_white: normalizePkSide(value?.pk_white),
    pk_max_moves: integerValue(value?.pk_max_moves, fallback.pk_max_moves, 1, 4_294_967_295)
  };
}

export function normalizeAppPreferences(value: StoredAppPreferences | null | undefined): AppPreferences {
  const winrateLine = booleanValue(value?.winrateLine, defaultAppPreferences.winrateLine);
  const scoreLeadLine = booleanValue(value?.scoreLeadLine, defaultAppPreferences.scoreLeadLine);
  const defaultMaxVisits = integerValue(value?.defaultMaxVisits, defaultAppPreferences.defaultMaxVisits, 1, 1_000_000);
  return {
    matchDefaults: normalizeMatchDefaults(value?.matchDefaults),
    workspaceShares: normalizeWorkspaceShares(value?.workspaceShares),
    windowGeometry: normalizeWindowGeometry(value?.windowGeometry),
    workspaceVisibility: {
      left: booleanValue(value?.workspaceVisibility?.left, true),
      right: booleanValue(value?.workspaceVisibility?.right, true)
    },
    mainWindowAlwaysOnTop: booleanValue(value?.mainWindowAlwaysOnTop, false),
    showCoordinates: booleanValue(value?.showCoordinates, defaultAppPreferences.showCoordinates),
    showMoveNumbers: booleanValue(value?.showMoveNumbers, defaultAppPreferences.showMoveNumbers),
    showOwnership: booleanValue(value?.showOwnership, defaultAppPreferences.showOwnership),
    showPolicy: booleanValue(value?.showPolicy, defaultAppPreferences.showPolicy),
    showCandidates: booleanValue(value?.showCandidates, defaultAppPreferences.showCandidates),
    candidateLimit: integerValue(value?.candidateLimit, defaultAppPreferences.candidateLimit, 1, 20),
    defaultMaxVisits,
    defaultBoardWidth: integerValue(value?.defaultBoardWidth, defaultAppPreferences.defaultBoardWidth, 2, 25),
    defaultBoardHeight: integerValue(value?.defaultBoardHeight, defaultAppPreferences.defaultBoardHeight, 2, 25),
    defaultKomi: finiteValue(value?.defaultKomi, defaultAppPreferences.defaultKomi),
    taskSingleStageConditions: normalizeSingleStageTaskConditions(value, defaultMaxVisits),
    taskOverviewConditions: normalizeTaskConditions(
      value?.taskOverviewConditions,
      defaultAppPreferences.taskOverviewConditions
    ),
    taskDeepConditions: normalizeDeepTaskConditions(value, defaultMaxVisits),
    taskSwingOverviewConditions: normalizeTaskConditions(
      value?.taskSwingOverviewConditions,
      defaultAppPreferences.taskSwingOverviewConditions
    ),
    taskSwingDeepConditions: normalizeTaskConditions(
      value?.taskSwingDeepConditions,
      defaultAppPreferences.taskSwingDeepConditions
    ),
    taskSwingCriteria: normalizeSwingCriteria(value?.taskSwingCriteria),
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
    soundEnabled: booleanValue(value?.soundEnabled, defaultAppPreferences.soundEnabled),
    scoringRule: value?.scoringRule === "territory" ? "territory" : "area",
    continuousAnalysisEnabled: booleanValue(value?.continuousAnalysisEnabled, defaultAppPreferences.continuousAnalysisEnabled),
    recentGamePaths: Array.isArray(value?.recentGamePaths) ? value.recentGamePaths : [],
    continuousTimeLimitEnabled: booleanValue(value?.continuousTimeLimitEnabled, defaultAppPreferences.continuousTimeLimitEnabled),
    continuousTimeLimitSeconds: value?.continuousTimeLimitSeconds ?? defaultAppPreferences.continuousTimeLimitSeconds,
    continuousVisitsLimitEnabled: booleanValue(value?.continuousVisitsLimitEnabled, defaultAppPreferences.continuousVisitsLimitEnabled),
    continuousVisitsLimit: value?.continuousVisitsLimit ?? defaultAppPreferences.continuousVisitsLimit,
    continuousStopOnEmptyBoard: booleanValue(value?.continuousStopOnEmptyBoard, defaultAppPreferences.continuousStopOnEmptyBoard)
  };
}

export function newGameDefaultsError(value: Pick<AppPreferences, "defaultBoardWidth" | "defaultBoardHeight" | "defaultKomi">): string | null {
  if (!Number.isInteger(value.defaultBoardWidth) || value.defaultBoardWidth < 2 || value.defaultBoardWidth > 25
    || !Number.isInteger(value.defaultBoardHeight) || value.defaultBoardHeight < 2 || value.defaultBoardHeight > 25) {
    return "棋盘宽高须为 2–25 的整数。";
  }
  if (!Number.isFinite(value.defaultKomi)) return "贴目须为有限数值。";
  return null;
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

export function taskStageConditionsError(
  overview: AnalysisStageConditionsDto,
  deep: AnalysisStageConditionsDto
): string | null {
  const error = taskConditionsError(overview) ?? taskConditionsError(deep);
  if (error) return error;
  if (deep.total_visits.value < 500) {
    return "All-position deep total visits must be at least 500.";
  }
  return null;
}

export function swingCriteriaError(value: AnalysisSwingCriteriaDto): string | null {
  const thresholds = [value.winrate_change_percentage_points, value.score_change_points];
  if (thresholds.some((threshold) => !Number.isFinite(threshold.value) || threshold.value <= 0)) {
    return "Swing threshold values must be positive finite numbers; disabled thresholds retain their values.";
  }
  if (!thresholds.some((threshold) => threshold.enabled)) {
    return "Enable at least one swing selection threshold.";
  }
  return null;
}

function normalizeTaskConditions(
  value: AnalysisStageConditionsDto | undefined,
  fallback: AnalysisStageConditionsDto
): AnalysisStageConditionsDto {
  if (!value) return cloneTaskConditions(fallback);
  return {
    time_seconds: normalizeTaskLimit(value.time_seconds),
    total_visits: normalizeTaskLimit(value.total_visits),
    leading_candidate_visits: normalizeTaskLimit(value.leading_candidate_visits)
  };
}

function normalizeSingleStageTaskConditions(
  value: StoredAppPreferences | null | undefined,
  defaultMaxVisits: number
): AnalysisStageConditionsDto {
  const fallback = cloneTaskConditions(defaultAppPreferences.taskSingleStageConditions);
  fallback.total_visits.value = defaultMaxVisits;
  return normalizeTaskConditions(value?.taskSingleStageConditions ?? value?.taskConditions, fallback);
}

function normalizeDeepTaskConditions(value: StoredAppPreferences | null | undefined, defaultMaxVisits: number) {
  if (value?.taskDeepConditions) {
    return normalizeTaskConditions(value.taskDeepConditions, defaultAppPreferences.taskDeepConditions);
  }
  if (value?.taskConditions) {
    const migrated = normalizeTaskConditions(value.taskConditions, defaultAppPreferences.taskDeepConditions);
    migrated.total_visits.value = Math.max(500, migrated.total_visits.value);
    return migrated;
  }
  const fallback = cloneTaskConditions(defaultAppPreferences.taskDeepConditions);
  fallback.total_visits.value = Math.max(500, defaultMaxVisits);
  return fallback;
}

function cloneTaskConditions(value: AnalysisStageConditionsDto): AnalysisStageConditionsDto {
  return {
    time_seconds: { ...value.time_seconds },
    total_visits: { ...value.total_visits },
    leading_candidate_visits: { ...value.leading_candidate_visits }
  };
}

function normalizeSwingCriteria(value: AnalysisSwingCriteriaDto | undefined): AnalysisSwingCriteriaDto {
  const fallback = defaultAppPreferences.taskSwingCriteria;
  return {
    move_actors: value?.move_actors === "black" || value?.move_actors === "white"
      ? value.move_actors
      : "both",
    winrate_change_percentage_points: normalizeSwingThreshold(
      value?.winrate_change_percentage_points,
      fallback.winrate_change_percentage_points
    ),
    score_change_points: normalizeSwingThreshold(
      value?.score_change_points,
      fallback.score_change_points
    )
  };
}

function normalizeSwingThreshold(
  value: AnalysisSwingCriteriaDto["winrate_change_percentage_points"] | undefined,
  fallback: AnalysisSwingCriteriaDto["winrate_change_percentage_points"]
) {
  return {
    enabled: typeof value?.enabled === "boolean" ? value.enabled : fallback.enabled,
    value: typeof value?.value === "number" ? value.value : fallback.value
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

function finiteValue(value: unknown, fallback: number): number {
  const parsed = typeof value === "number" ? value : Number(value);
  return Number.isFinite(parsed) ? parsed : fallback;
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
