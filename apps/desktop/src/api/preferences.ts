import { invoke } from "@tauri-apps/api/core";
import { continuousBudgetError, defaultAppPreferences, normalizeAppPreferences, taskConditionsError, type AppPreferences } from "../domain/preferences";

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
  const error = continuousBudgetError(preferences) ?? taskConditionsError(preferences.taskConditions);
  if (error) throw new Error(error);
  const normalized = normalizeAppPreferences(preferences);
  if (!isTauriRuntime()) {
    saveBrowserPreferences(normalized);
    return normalized;
  }
  return normalizeAppPreferences(await invoke<AppPreferences>("save_app_preferences", { preferences: normalized }));
}

function loadBrowserPreferences(): AppPreferencesLoadResult {
  if (typeof window === "undefined") return { preferences: defaultAppPreferences };
  const raw = window.localStorage.getItem(browserPreferencesKey);
  if (!raw) return { preferences: defaultAppPreferences };
  try {
    const preferences = normalizeAppPreferences(JSON.parse(raw) as Partial<AppPreferences>);
    const error = continuousBudgetError(preferences) ?? taskConditionsError(preferences.taskConditions);
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
