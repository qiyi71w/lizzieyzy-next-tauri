// @vitest-environment jsdom

import { afterEach, describe, expect, it } from "vitest";
import { fakeAnalyze } from "./backend";

describe("fakeAnalyze ownership", () => {
  afterEach(() => {
    delete window.__TAURI_INTERNALS__;
  });

  it("keeps browser demonstration frames marked non-authoritative", async () => {
    const frames = await fakeAnalyze("(;GM[1]FF[4]SZ[9];B[pd])");
    expect(frames.length).toBeGreaterThan(0);
    expect(frames.every((frame) => frame.job_id === "browser-preview")).toBe(true);
    expect(frames[0]?.candidates.length).toBeGreaterThan(0);
  });

  it("does not manufacture substitute candidates when the native runtime is active", async () => {
    window.__TAURI_INTERNALS__ = {};
    await expect(fakeAnalyze("(;GM[1]FF[4]SZ[9];B[pd])")).rejects.toThrow(/non-authoritative/i);
  });
});
