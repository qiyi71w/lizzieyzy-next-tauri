import { expect, it } from "vitest";
import { classifyMoveRank, classifyPlayedMove, defaultMoveRankPreferences, readMoveRankPreferences, validMoveRankPreferences } from "./moveRank";
import { reviewLineProblems } from "./reviewNavigation";
import { blunderBars, type ChartPoint } from "./winrateChart";
import type { MoveRankMode } from "./types";

it("honors every inclusive boundary below/equal/above in all four modes and worst duplicate thresholds", () => {
  const ranks = ["best", "good", "normal", "inaccuracy", "mistake", "blunder"];
  for (const mode of ["auto", "score", "winrate", "combined"] as const) {
    const settings = { ...readMoveRankPreferences(defaultMoveRankPreferences), mode };
    for (const [index, threshold] of settings.winrateThresholdsPp.entries()) {
      if (mode === "score") continue;
      for (const delta of [-0.0001, 0, 0.0001]) expect(classifyMoveRank(threshold + delta, null, settings)).toBe(ranks[index + (delta < 0 ? 0 : 1)]);
    }
    for (const [index, threshold] of settings.scoreThresholds.entries()) {
      if (mode === "winrate") continue;
      for (const delta of [-0.0001, 0, 0.0001]) expect(classifyMoveRank(0, threshold + delta, settings)).toBe(ranks[index + (delta < 0 ? 0 : 1)]);
    }
    settings.winrateThresholdsPp = [2, 2, 2, 2, 2]; settings.scoreThresholds = [2, 2, 2, 2, 2];
    expect(classifyMoveRank(1.999, 1.999, settings)).toBe("best");
    expect(classifyMoveRank(2, 2, settings)).toBe("blunder");
    expect(classifyMoveRank(2.001, 2.001, settings)).toBe("blunder");
    settings.winrateThresholdsPp = [-2, -1, 0, 150.123456789, 200]; settings.scoreThresholds = [...settings.winrateThresholdsPp];
    expect(validMoveRankPreferences(settings)).toBe(true);
    expect(classifyMoveRank(-10, -10, settings)).toBe("inaccuracy");
  }
});

it("keeps missing score distinct from absent/zero-visit analysis for both perspectives and all modes", () => {
  for (const mode of ["auto", "score", "winrate", "combined"] as MoveRankMode[]) {
    const settings = { ...defaultMoveRankPreferences, mode };
    expect(classifyMoveRank(6, 0.1, settings)).toBe(mode === "auto" || mode === "score" ? "best" : "inaccuracy");
    expect(classifyMoveRank(12, 0.1, settings)).toBe(mode === "score" ? "best" : "mistake");
    for (const color of ["black", "white"] as const) {
      const parent = { visits: 10, winrateBlack: 0.5, scoreMeanBlack: 0 };
      const child = { visits: 10, winrateBlack: color === "black" ? 0.25 : 0.75 };
      expect(classifyPlayedMove(parent, child, color, settings)).toBe(mode === "score" ? "best" : "blunder");
      expect(classifyPlayedMove(null, child, color, settings)).toBeNull();
      expect(classifyPlayedMove(parent, { ...child, visits: 0 }, color, settings)).toBeNull();
      const points: ChartPoint[] = [
        { path: { indices: [] }, moveNumber: 0, isMove: false, toPlay: color, analysis: parent },
        { path: { indices: [0] }, moveNumber: 1, isMove: true, toPlay: color === "black" ? "white" : "black", analysis: child }
      ];
      expect(blunderBars(points, settings).map(bar => bar.rank)).toEqual(mode === "score" ? [] : ["blunder"]);
      expect(reviewLineProblems(points, settings).map(problem => problem.severity)).toEqual(mode === "score" ? [] : ["blunder"]);
    }
  }
});
