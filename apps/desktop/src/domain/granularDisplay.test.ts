import { expect, it } from "vitest";
import { candidateDisplayScore, defaultGranularDisplay, displayVariationSteps, pvVisitsLabel, readGranularDisplay, validGranularDisplay } from "./granularDisplay";
import type { CandidateMoveDto } from "./types";
const candidate: CandidateMoveDto = { vertex: { point: { x: 0, y: 0 } }, visits: 900, winrate_black: 0.6, score_mean_black: -2.5, pv: [{ point: { x: 0, y: 0 } }, "pass", { point: { x: 1, y: 1 } }], pv_visits: [12, 8, 3] };
it("keeps PV visits paired through a pass and display/replay truncation with a strict threshold", () => {
  const preferences = { ...defaultGranularDisplay, pvVisitsMode: "all" as const, pvVisitsThreshold: 3, pvLength: 3 };
  const steps = displayVariationSteps(candidate, preferences);
  expect(steps.map(step => [step.offset, step.pvIndex])).toEqual([[0, 0], [2, 2]]);
  expect(pvVisitsLabel(candidate, 0, 3, preferences)).toBe("12");
  expect(pvVisitsLabel(candidate, 2, 3, preferences)).toBeNull();
  expect(displayVariationSteps(candidate, { ...preferences, pvLength: 2 }).length).toBe(1);
  const passEnded = displayVariationSteps(candidate, { ...preferences, pvLength: 2 });
  expect(pvVisitsLabel(candidate, passEnded[0].pvIndex, passEnded[0].lastPvIndex + 1, { ...preferences, pvVisitsMode: "last" })).toBeNull();
  expect(displayVariationSteps(candidate, preferences, 1).length).toBe(1);
  expect(pvVisitsLabel({ ...candidate, pv_visits: [900] }, 0, 1, preferences)).toBeNull();
  expect(pvVisitsLabel({ ...candidate, pv_visits: undefined }, 0, 1, preferences)).toBeNull();
  expect(pvVisitsLabel(candidate, 0, 3, { ...preferences, pvVisitsMode: "last" })).toBeNull();
  expect(pvVisitsLabel(candidate, 2, 3, { ...preferences, pvVisitsMode: "last", pvVisitsThreshold: 0 })).toBe("3");
  expect(candidate.pv).toHaveLength(3);
});
it("adds komi once before perspective conversion and preserves unknown score/context", () => {
  const preferences = { ...defaultGranularDisplay, scoreMode: "board" as const };
  expect(candidateDisplayScore(candidate, preferences, "black", "white", 7.5)).toBe(5);
  expect(candidateDisplayScore(candidate, preferences, "sideToPlay", "white", 7.5)).toBe(-5);
  expect(candidateDisplayScore(candidate, defaultGranularDisplay, "sideToPlay", "white", 7.5)).toBe(2.5);
  expect(candidateDisplayScore(candidate, preferences, "black", "black")).toBeNull();
  expect(candidateDisplayScore({ ...candidate, score_mean_black: null }, preferences, "black", "black", 7.5)).toBeNull();
});
it("defaults absent fields but rejects malformed values and keeps source integer domains", () => {
  expect(readGranularDisplay(undefined)).toEqual(defaultGranularDisplay);
  expect(() => readGranularDisplay({ pvLength: 2147483648 })).toThrow();
  expect(validGranularDisplay({ ...defaultGranularDisplay, pvLength: -2147483648, pvVisitsThreshold: 2147483647 })).toBe(true);
  expect(validGranularDisplay({ ...defaultGranularDisplay, delaySeconds: 1.001 })).toBe(true);
  expect(validGranularDisplay({ ...defaultGranularDisplay, textOrder: ["visits", "visits", "score"] })).toBe(false);
  for (const delaySeconds of [NaN, Infinity, 0, -1, 0.0001, 2147483.648]) expect(validGranularDisplay({ ...defaultGranularDisplay, delaySeconds })).toBe(false);
});
