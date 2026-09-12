import { describe, expect, it } from "vitest";
import * as backend from "./backend";

describe("frontend analysis API ownership", () => {
  it("does not export retired process or force-replacement wrappers", () => {
    expect("analyzeKataGoOnce" in backend).toBe(false);
    expect("analyzeKataGoGame" in backend).toBe(false);
    expect("listenToKataGoAnalysisEvents" in backend).toBe(false);
    expect("replaceCurrentGame" in backend).toBe(false);
    expect("startForegroundContinuousNodeAnalysis" in backend).toBe(false);
  });

});
