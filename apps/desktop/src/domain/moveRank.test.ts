import { describe, expect, it } from "vitest";
import { classifyMoveRank as classifyAutoMoveRank, classifyPlayedMove, classifyProblemFrames, isBlunderBarRank } from "./moveRank";
import { defaultMoveRankPreferences as DEFAULT_MOVE_RANK } from "./moveRank";
import type { AnalysisFrameDto } from "./types";

describe("Java Auto Move Rank", () => {
  it("uses inclusive winrate-only boundaries", () => {
    expect(classifyAutoMoveRank(0.999)).toBe("best");
    expect(classifyAutoMoveRank(1)).toBe("good");
    expect(classifyAutoMoveRank(3)).toBe("normal");
    expect(classifyAutoMoveRank(6)).toBe("inaccuracy");
    expect(classifyAutoMoveRank(12)).toBe("mistake");
    expect(classifyAutoMoveRank(24)).toBe("blunder");
  });

  it("uses score for ordinary Auto ranks and winrate as Mistake/Blunder guard", () => {
    expect(classifyAutoMoveRank(6, 0.4)).toBe("best");
    expect(classifyAutoMoveRank(0, 0.5)).toBe("good");
    expect(classifyAutoMoveRank(20, 0.1)).toBe("mistake");
    expect(classifyAutoMoveRank(24)).toBe("blunder");
  });

  it("computes same-player loss from parent side-to-play", () => {
    expect(classifyPlayedMove(
      { visits: 10, winrateBlack: 0.6 },
      { visits: 10, winrateBlack: 0.48 },
      "black"
    )).toBe("mistake");
    expect(classifyPlayedMove(
      { visits: 10, winrateBlack: 0.4 },
      { visits: 10, winrateBlack: 0.52 },
      "white"
    )).toBe("mistake");
    expect(classifyPlayedMove(
      { visits: 10, winrateBlack: 0.4 },
      { visits: 10, winrateBlack: 0.52 },
      "black"
    )).toBe("best");
  });

  it("leaves missing, malformed, and zero-visit pairs ungraded", () => {
    const ok = { visits: 8, winrateBlack: 0.55, scoreMeanBlack: 1 };
    expect(classifyPlayedMove(null, ok, "black")).toBeNull();
    expect(classifyPlayedMove(ok, { visits: 0, winrateBlack: 0.1, scoreMeanBlack: -12 }, "black")).toBeNull();
  });

  it("filters Blunder Bar to the last three ranks", () => {
    expect(isBlunderBarRank("best")).toBe(false);
    expect(isBlunderBarRank("normal")).toBe(false);
    expect(isBlunderBarRank("inaccuracy")).toBe(true);
    expect(isBlunderBarRank("blunder")).toBe(true);
  });
});

it("grades a White-first game from its played move rather than turn parity", () => {
  const frame = (turn: number, winrate_black: number): AnalysisFrameDto => ({
    job_id: "white-first", turn, visits: 100, winrate_black, score_mean_black: null,
    score_stdev: null, candidates: [], ownership: null, policy: null,
  });
  const moves = [{ color: "white" as const, vertex: "pass" as const, move_number: 1 }];
  const loss = classifyProblemFrames([frame(0, 0.3), frame(1, 0.6)], DEFAULT_MOVE_RANK, moves);
  expect(loss).toHaveLength(1);
  expect(loss[0].severity).toBe("blunder");
  expect(loss[0].winrate_loss).toBeCloseTo(0.3);
  expect(classifyProblemFrames([frame(0, 0.6), frame(1, 0.3)], DEFAULT_MOVE_RANK, moves)).toEqual([]);
});
