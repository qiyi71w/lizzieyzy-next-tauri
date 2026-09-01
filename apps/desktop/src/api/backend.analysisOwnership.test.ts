import { describe, expect, it } from "vitest";
import * as backend from "./backend";

describe("frontend analysis API ownership", () => {
  it("does not export profile-to-process analysis wrappers", () => {
    expect("analyzeKataGoOnce" in backend).toBe(false);
    expect("analyzeKataGoGame" in backend).toBe(false);
  });

  it("keeps manager-owned analysis entry points and non-authoritative fake analysis", () => {
    expect(typeof backend.startSelectedNodeAnalysis).toBe("function");
    expect(typeof backend.startKataGoGameAnalysis).toBe("function");
    expect(typeof backend.fakeAnalyze).toBe("function");
  });
});
