export type ReviewMode = "quick" | "deep";
export type BoardTheme = "classic" | "high-contrast";

export type AppPreferences = {
  showOwnership: boolean;
  showPolicy: boolean;
  showCandidates: boolean;
  candidateLimit: number;
  defaultMaxVisits: number;
  reviewMode: ReviewMode;
  boardTheme: BoardTheme;
};

export const defaultAppPreferences: AppPreferences = {
  showOwnership: true,
  showPolicy: true,
  showCandidates: true,
  candidateLimit: 8,
  defaultMaxVisits: 800,
  reviewMode: "quick",
  boardTheme: "classic"
};

export function normalizeAppPreferences(value: Partial<AppPreferences> | null | undefined): AppPreferences {
  return {
    showOwnership: booleanValue(value?.showOwnership, defaultAppPreferences.showOwnership),
    showPolicy: booleanValue(value?.showPolicy, defaultAppPreferences.showPolicy),
    showCandidates: booleanValue(value?.showCandidates, defaultAppPreferences.showCandidates),
    candidateLimit: integerValue(value?.candidateLimit, defaultAppPreferences.candidateLimit, 1, 20),
    defaultMaxVisits: integerValue(value?.defaultMaxVisits, defaultAppPreferences.defaultMaxVisits, 1, 1_000_000),
    reviewMode: value?.reviewMode === "deep" ? "deep" : "quick",
    boardTheme: value?.boardTheme === "high-contrast" ? "high-contrast" : "classic"
  };
}

function booleanValue(value: unknown, fallback: boolean): boolean {
  return typeof value === "boolean" ? value : fallback;
}

function integerValue(value: unknown, fallback: number, min: number, max: number): number {
  const parsed = typeof value === "number" ? value : Number(value);
  if (!Number.isFinite(parsed)) return fallback;
  return Math.min(max, Math.max(min, Math.floor(parsed)));
}
