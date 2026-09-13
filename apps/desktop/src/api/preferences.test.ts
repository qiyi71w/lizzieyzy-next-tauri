// @vitest-environment jsdom

import { afterEach, describe, expect, it } from "vitest";
import { defaultAppPreferences } from "../domain/preferences";
import {
  loadAppPreferences,
  saveAppPreferences,
  UNREADABLE_PREFERENCES_RECOVERY_MESSAGE
} from "./preferences";

const storageKey = "lizzieyzy-next-app-preferences";
const unreadableKey = `${storageKey}.unreadable`;

afterEach(() => {
  window.localStorage.clear();
});

describe("browser preference storage", () => {
  it("loads owner defaults when storage is missing", async () => {
    const loaded = await loadAppPreferences();
    expect(loaded.preferences).toEqual(defaultAppPreferences);
    expect(loaded.recovery).toBeUndefined();
  });

  it("fills missing keys from owner defaults", async () => {
    window.localStorage.setItem(storageKey, JSON.stringify({ showCandidates: false }));
    const loaded = await loadAppPreferences();
    expect(loaded.preferences).toEqual({ ...defaultAppPreferences, showCandidates: false });
    expect(loaded.recovery).toBeUndefined();
  });

  it("derives missing stage presets from the legacy selected-node visit preference", async () => {
    window.localStorage.setItem(storageKey, JSON.stringify({ defaultMaxVisits: 321 }));
    const loaded = await loadAppPreferences();
    expect(loaded.preferences.taskSingleStageConditions.total_visits).toEqual({ enabled: true, value: 321 });
    expect(loaded.preferences.taskOverviewConditions.total_visits).toEqual({ enabled: true, value: 32 });
    expect(loaded.preferences.taskDeepConditions).toEqual({
      time_seconds: { enabled: false, value: 10 },
      total_visits: { enabled: true, value: 500 },
      leading_candidate_visits: { enabled: false, value: 500 }
    });
  });

  it("migrates the legacy task preset without losing unrelated preferences", async () => {
    window.localStorage.setItem(storageKey, JSON.stringify({
      continuousAnalysisEnabled: false,
      defaultMaxVisits: 32,
      taskConditions: {
        time_seconds: { enabled: false, value: 10 },
        total_visits: { enabled: true, value: 32 },
        leading_candidate_visits: { enabled: false, value: 32 }
      }
    }));
    const loaded = await loadAppPreferences();
    expect(loaded.recovery).toBeUndefined();
    expect(loaded.preferences.continuousAnalysisEnabled).toBe(false);
    expect(loaded.preferences.taskSingleStageConditions.total_visits.value).toBe(32);
    expect(loaded.preferences.taskOverviewConditions.total_visits.value).toBe(32);
    expect(loaded.preferences.taskDeepConditions.total_visits.value).toBe(500);
  });

  it("fills missing Sub-Board content mode as Variation", async () => {
    window.localStorage.setItem(storageKey, JSON.stringify({ showCandidates: false }));
    const loaded = await loadAppPreferences();
    expect(loaded.preferences.subBoardContentMode).toBe("variation");
  });

  it("fills missing Variation Replay as off with a 500 ms interval", async () => {
    window.localStorage.setItem(storageKey, JSON.stringify({ showCandidates: false }));
    const loaded = await loadAppPreferences();
    expect(loaded.preferences.variationReplayEnabled).toBe(false);
    expect(loaded.preferences.variationReplayIntervalMs).toBe(500);
  });

  it("fills missing restore-last-session as off", async () => {
    window.localStorage.setItem(storageKey, JSON.stringify({ showCandidates: false }));
    const loaded = await loadAppPreferences();
    expect(loaded.preferences.restoreLastSession).toBe(false);
  });

  it("fills missing continuous-analysis intent as enabled", async () => {
    window.localStorage.setItem(storageKey, JSON.stringify({ showCandidates: false }));
    const loaded = await loadAppPreferences();
    expect(loaded.preferences.continuousAnalysisEnabled).toBe(true);
  });

  it("clamps Variation Replay interval to 100–5000 ms", async () => {
    const low = await saveAppPreferences({ ...defaultAppPreferences, variationReplayIntervalMs: 50 });
    const high = await saveAppPreferences({ ...defaultAppPreferences, variationReplayIntervalMs: 9000 });
    expect(low.variationReplayIntervalMs).toBe(100);
    expect(high.variationReplayIntervalMs).toBe(5000);
  });

  it("isolates unreadable storage, loads defaults, and reports recovery", async () => {
    window.localStorage.setItem(storageKey, "{not-json");
    const loaded = await loadAppPreferences();
    expect(loaded.preferences).toEqual(defaultAppPreferences);
    expect(loaded.recovery).toEqual({
      isolatedPath: unreadableKey,
      message: UNREADABLE_PREFERENCES_RECOVERY_MESSAGE
    });
    expect(window.localStorage.getItem(storageKey)).toBeNull();
    expect(window.localStorage.getItem(unreadableKey)).toBe("{not-json");
  });

  it("reloads a successful write as the next load", async () => {
    const saved = await saveAppPreferences({ ...defaultAppPreferences, showCandidates: false, candidateLimit: 3 });
    const loaded = await loadAppPreferences();
    expect(loaded.preferences).toEqual(saved);
    expect(loaded.recovery).toBeUndefined();
  });
  it("reloads independent overview and deep task presets", async () => {
    const overview = {
      time_seconds: { enabled: true, value: 3 },
      total_visits: { enabled: true, value: 32 },
      leading_candidate_visits: { enabled: false, value: 32 }
    };
    const deep = {
      time_seconds: { enabled: true, value: 12 },
      total_visits: { enabled: false, value: 640 },
      leading_candidate_visits: { enabled: true, value: 7 }
    };
    await saveAppPreferences({
      ...defaultAppPreferences,
      defaultMaxVisits: 999,
      taskOverviewConditions: overview,
      taskDeepConditions: deep
    });
    const loaded = await loadAppPreferences();
    expect(loaded.preferences.defaultMaxVisits).toBe(999);
    expect(loaded.preferences.taskOverviewConditions).toEqual(overview);
    expect(loaded.preferences.taskDeepConditions).toEqual(deep);
  });

  it("rejects invalid stage values, all-disabled stages, and shallow deep presets without writing", async () => {
    await expect(saveAppPreferences({
      ...defaultAppPreferences,
      taskOverviewConditions: {
        ...defaultAppPreferences.taskOverviewConditions,
        time_seconds: { enabled: false, value: 0 }
      }
    })).rejects.toThrow("whole numbers");
    await expect(saveAppPreferences({
      ...defaultAppPreferences,
      taskOverviewConditions: {
        time_seconds: { enabled: false, value: 10 },
        total_visits: { enabled: false, value: 32 },
        leading_candidate_visits: { enabled: false, value: 32 }
      }
    })).rejects.toThrow("at least one");
    await expect(saveAppPreferences({
      ...defaultAppPreferences,
      taskDeepConditions: {
        ...defaultAppPreferences.taskDeepConditions,
        total_visits: { enabled: true, value: 499 }
      }
    })).rejects.toThrow("at least 500");
    expect(window.localStorage.getItem(storageKey)).toBeNull();
  });
});
