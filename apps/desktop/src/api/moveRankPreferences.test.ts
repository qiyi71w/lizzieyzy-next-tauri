// @vitest-environment jsdom
import { afterEach, expect, it, vi } from "vitest";
import { loadAppPreferences, saveAppPreferences } from "./preferences";
import { defaultAppPreferences } from "../domain/preferences";
import { classifyPlayedMove } from "../domain/moveRank";

afterEach(() => { vi.restoreAllMocks(); localStorage.clear(); });
it("reopens committed custom grading and keeps it after invalid input and an actual storage failure", async () => {
  const saved = await saveAppPreferences({ ...defaultAppPreferences, moveRank: { mode: "combined", winrateThresholdsPp: [-1, 0, 2, 2, 200], scoreThresholds: [0.1, 0.2, 0.3, 0.4, 0.5] } });
  const parent = { visits: 10, winrateBlack: 0.5, scoreMeanBlack: 1 };
  const child = { visits: 10, winrateBlack: 0.5, scoreMeanBlack: 0.5 };
  expect(classifyPlayedMove(parent, child, "black", (await loadAppPreferences()).preferences.moveRank)).toBe("blunder");
  await expect(saveAppPreferences({ ...saved, moveRank: { ...saved.moveRank, winrateThresholdsPp: [2, 1, 3, 4, 5] } })).rejects.toThrow("nondecreasing");
  vi.spyOn(Storage.prototype, "setItem").mockImplementationOnce(() => { throw new Error("disk full"); });
  await expect(saveAppPreferences(defaultAppPreferences)).rejects.toThrow("disk full");
  const reopened = (await loadAppPreferences()).preferences;
  expect(reopened.moveRank).toEqual(saved.moveRank);
  expect(classifyPlayedMove(parent, child, "black", reopened.moveRank)).toBe("blunder");
  await saveAppPreferences({ ...saved, moveRank: defaultAppPreferences.moveRank });
  expect((await loadAppPreferences()).preferences.moveRank).toEqual(defaultAppPreferences.moveRank);
  expect(classifyPlayedMove(parent, child, "black", (await loadAppPreferences()).preferences.moveRank)).toBe("good");
});
