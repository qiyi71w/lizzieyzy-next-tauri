import { describe, expect, it } from "vitest";
import * as backend from "./backend";

describe("frontend analysis API ownership", () => {
  it("does not export retired process or force-replacement wrappers", () => {
    expect("analyzeKataGoOnce" in backend).toBe(false);
    expect("analyzeKataGoGame" in backend).toBe(false);
    expect("listenToKataGoAnalysisEvents" in backend).toBe(false);
    expect("replaceCurrentGame" in backend).toBe(false);
  });

  it("keeps manager-owned analysis entry points and browser-only demonstration analysis", () => {
    expect(typeof backend.startSelectedNodeAnalysis).toBe("function");
    expect(typeof backend.startKataGoGameAnalysis).toBe("function");
    expect(typeof backend.fakeAnalyze).toBe("function");
  });
});
