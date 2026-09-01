import { describe, expect, it } from "vitest";
import { admitsAnalysisPublication } from "./analysisJob";
import type { AnalysisFrameDto, AnalysisJobEventDto, AnalysisPublicationScopeDto } from "./types";

const frame: AnalysisFrameDto = {
  job_id: "job-1",
  turn: 0,
  visits: 2,
  winrate_black: 0.5,
  score_mean_black: 0,
  candidates: []
};

const current: AnalysisPublicationScopeDto = {
  run_id: "run-1",
  job_id: "job-1",
  generation: 3,
  node_path: { indices: [0, 1] }
};

function event(overrides: Partial<AnalysisJobEventDto> = {}): AnalysisJobEventDto {
  return {
    run_id: "run-1",
    job_id: "job-1",
    lane: "selected_node",
    generation: 3,
    node_path: { indices: [0, 1] },
    outcome: "completed",
    frame,
    ...overrides
  };
}

describe("admitsAnalysisPublication", () => {
  it("requires a completed frame with matching run, job, generation, and NodePath", () => {
    expect(admitsAnalysisPublication(event(), current)).toBe(true);
    expect(admitsAnalysisPublication(event({ run_id: "run-2" }), current)).toBe(false);
    expect(admitsAnalysisPublication(event({ job_id: "job-2" }), current)).toBe(false);
    expect(admitsAnalysisPublication(event({ generation: 4 }), current)).toBe(false);
    expect(admitsAnalysisPublication(event({ node_path: { indices: [1] } }), current)).toBe(false);
    expect(admitsAnalysisPublication(event({ outcome: "cancelled" }), current)).toBe(false);
    expect(admitsAnalysisPublication(event({ frame: undefined }), current)).toBe(false);
  });
});
