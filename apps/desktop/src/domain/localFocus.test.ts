import { describe, expect, it } from "vitest";
import { defaultLocalFocusPreferences, focusFilteredCandidates, focusQuality, validLocalFocusPreferences } from "./localFocus";
import { defaultMoveRankPreferences } from "./moveRank";
import type { AnalysisFrameDto, LocalFocusPointDto } from "./types";

const frame: AnalysisFrameDto = { job_id: "ordinary", turn: 0, visits: 1000, winrate_black: 0.99,
  candidates: [
    { order: 0, vertex: { point: { x: 0, y: 0 } }, visits: 800, winrate_black: 0.5, score_mean_black: 0, pv: [] },
    { order: 3, vertex: { point: { x: 3, y: 3 } }, visits: 1, winrate_black: 0.49, score_mean_black: -6, pv: [] }
  ] };
const watched: LocalFocusPointDto = { vertex: frame.candidates[1].vertex, target_visits: 500, visits: 500, status: "completed" };

describe("ordinary focus presentation", () => {
  it("keeps completed watched candidates outside the ordinary count limit and restores filtering on removal", () => {
    expect(focusFilteredCandidates(frame, 1, [watched])).toEqual(frame.candidates);
    expect(focusFilteredCandidates(frame, 1, [])).toEqual([frame.candidates[0]]);
    expect(frame.candidates[1].order).toBe(3);
  });
  it("grades against candidate order zero rather than the root and honors player color", () => {
    expect(focusQuality(frame, watched.vertex, "black", defaultMoveRankPreferences)).toBe("mistake");
    expect(focusQuality(frame, watched.vertex, "white", defaultMoveRankPreferences)).toBe("best");
    expect(focusQuality({ ...frame, candidates: [frame.candidates[1]] }, watched.vertex, "black", defaultMoveRankPreferences)).toBeNull();
    expect(focusQuality(undefined, watched.vertex, "black", defaultMoveRankPreferences)).toBeNull();
  });
  it("defaults to 500 visits and 92 percent outline opacity with strict positive visits", () => {
    expect(defaultLocalFocusPreferences).toEqual({ maxVisits: 500, showOutline: true, outlineOpacity: 92 });
    expect(validLocalFocusPreferences(defaultLocalFocusPreferences)).toBe(true);
    for (const maxVisits of [0, -1, 0.5, NaN, Infinity, 4294967296]) expect(validLocalFocusPreferences({ ...defaultLocalFocusPreferences, maxVisits })).toBe(false);
    expect(validLocalFocusPreferences({ ...defaultLocalFocusPreferences, showOutline: false })).toBe(true);
  });
});
