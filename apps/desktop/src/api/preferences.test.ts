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
});
