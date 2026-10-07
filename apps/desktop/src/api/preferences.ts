import { invoke } from "@tauri-apps/api/core";
import { continuousBudgetError, defaultAppPreferences, newGameDefaultsError, normalizeAppPreferences, swingCriteriaError, taskConditionsError, taskStageConditionsError, type AppPreferences } from "../domain/preferences";
import type { WorkspaceSharesDto, WorkspaceVisibilityDto } from "../domain/types";
import { normalizeWorkspaceShares } from "../domain/preferences";

declare global {
  interface Window {
    __TAURI_INTERNALS__?: unknown;
  }
}

export const UNREADABLE_PREFERENCES_RECOVERY_MESSAGE = "Unreadable preferences isolated; restored defaults.";

export type AppPreferencesRecovery = {
  isolatedPath: string;
  message: string;
};

export type AppPreferencesLoadResult = {
  preferences: AppPreferences;
  recovery?: AppPreferencesRecovery | null;
};

const browserPreferencesKey = "lizzieyzy-next-app-preferences";
const browserUnreadableKey = `${browserPreferencesKey}.unreadable`;
const isTauriRuntime = () => typeof window !== "undefined" && window.__TAURI_INTERNALS__ !== undefined;

export async function loadAppPreferences(): Promise<AppPreferencesLoadResult> {
  if (!isTauriRuntime()) return loadBrowserPreferences();
  const result = await invoke<AppPreferencesLoadResult>("load_app_preferences");
  return {
    preferences: normalizeAppPreferences(result.preferences),
    recovery: result.recovery ?? undefined
  };
}

export async function saveAppPreferences(preferences: AppPreferences): Promise<AppPreferences> {
  const error = newGameDefaultsError(preferences)
    ?? continuousBudgetError(preferences)
    ?? taskConditionsError(preferences.taskSingleStageConditions)
    ?? taskStageConditionsError(preferences.taskOverviewConditions, preferences.taskDeepConditions)
    ?? taskConditionsError(preferences.taskSwingOverviewConditions)
    ?? taskConditionsError(preferences.taskSwingDeepConditions)
    ?? swingCriteriaError(preferences.taskSwingCriteria);
  if (error) throw new Error(error);
  const normalized = normalizeAppPreferences(preferences);
  if (!isTauriRuntime()) {
    const latest = loadBrowserPreferences().preferences;
    const saved = { ...normalized, workspaceShares: latest.workspaceShares, windowGeometry: latest.windowGeometry, workspaceVisibility: latest.workspaceVisibility, recentGamePaths: latest.recentGamePaths, recentImageExportDirectory: latest.recentImageExportDirectory };
    saveBrowserPreferences(saved);
    return saved;
  }
  return normalizeAppPreferences(await invoke<AppPreferences>("save_app_preferences", { preferences: normalized }));
}

export async function updateWorkspaceShares(shares: WorkspaceSharesDto | null): Promise<WorkspaceSharesDto | null> {
  if (shares !== null && normalizeWorkspaceShares(shares) === null) throw new Error("Invalid workspace shares.");
  if (isTauriRuntime()) return invoke<WorkspaceSharesDto | null>("update_workspace_shares", { shares });
  const preferences = loadBrowserPreferences().preferences;
  saveBrowserPreferences({ ...preferences, workspaceShares: shares });
  return shares;
}

export async function updateWorkspaceVisibility(patch: Partial<WorkspaceVisibilityDto>): Promise<WorkspaceVisibilityDto> {
  if (isTauriRuntime()) return invoke<WorkspaceVisibilityDto>("update_workspace_visibility", patch);
  const latest = loadBrowserPreferences().preferences;
  const workspaceVisibility = { ...latest.workspaceVisibility, ...patch };
  saveBrowserPreferences({ ...latest, workspaceVisibility });
  return workspaceVisibility;
}

export async function updateRecentGameHistory(openedPath: string | null): Promise<string[]> {
  if (!isTauriRuntime()) throw new Error("Recent game history requires the native Tauri desktop backend.");
  return invoke<string[]>("update_recent_game_history", { openedPath });
}

function loadBrowserPreferences(): AppPreferencesLoadResult {
  if (typeof window === "undefined") return { preferences: defaultAppPreferences };
  const raw = window.localStorage.getItem(browserPreferencesKey);
  if (!raw) return { preferences: defaultAppPreferences };
  try {
    const preferences = normalizeAppPreferences(JSON.parse(raw) as Partial<AppPreferences>);
    const error = newGameDefaultsError(preferences)
      ?? continuousBudgetError(preferences)
      ?? taskConditionsError(preferences.taskSingleStageConditions)
      ?? taskStageConditionsError(preferences.taskOverviewConditions, preferences.taskDeepConditions)
      ?? taskConditionsError(preferences.taskSwingOverviewConditions)
      ?? taskConditionsError(preferences.taskSwingDeepConditions)
      ?? swingCriteriaError(preferences.taskSwingCriteria);
    if (error) throw new Error(error);
    return { preferences };
  } catch {
    window.localStorage.setItem(browserUnreadableKey, raw);
    window.localStorage.removeItem(browserPreferencesKey);
    return {
      preferences: defaultAppPreferences,
      recovery: {
        isolatedPath: browserUnreadableKey,
        message: UNREADABLE_PREFERENCES_RECOVERY_MESSAGE
      }
    };
  }
}

function saveBrowserPreferences(preferences: AppPreferences) {
  if (typeof window === "undefined") return;
  window.localStorage.setItem(browserPreferencesKey, JSON.stringify(preferences));
}
