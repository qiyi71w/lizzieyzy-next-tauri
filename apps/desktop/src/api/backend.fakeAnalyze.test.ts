// @vitest-environment jsdom

import { afterEach, describe, expect, it } from "vitest";
import { fakeAnalyze, parseSgfSummary } from "./backend";

describe("fakeAnalyze ownership", () => {
  afterEach(() => {
    delete window.__TAURI_INTERNALS__;
  });

  it("keeps rectangular browser demonstration frames non-authoritative and in bounds", async () => {
    const sgf = "(;GM[1]FF[4]SZ[9:5];B[de])";
    const game = await parseSgfSummary(sgf);
    const frames = await fakeAnalyze(sgf);
    expect(game.summary).toEqual(expect.objectContaining({ board_width: 9, board_height: 5 }));
    expect(frames.length).toBeGreaterThan(0);
    expect(frames.every((frame) => frame.job_id === "browser-preview")).toBe(true);
    expect(frames[0]?.candidates.length).toBeGreaterThan(0);
    expect(frames.flatMap((frame) => frame.candidates).every((candidate) => (
      typeof candidate.vertex === "string"
      || (candidate.vertex.point.x < 9 && candidate.vertex.point.y < 5)
    ))).toBe(true);
  });

  it("does not manufacture substitute candidates when the native runtime is active", async () => {
    window.__TAURI_INTERNALS__ = {};
    await expect(fakeAnalyze("(;GM[1]FF[4]SZ[9];B[pd])")).rejects.toThrow(/non-authoritative/i);
  });
});
