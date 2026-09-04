import { describe, expect, it } from "vitest";
import { admitsAnalysisAttachment, admitsAnalysisPublication, admitsWholeGameNodeResult, matchesWholeGameJobIdentity } from "./analysisJob";
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

describe("whole-game incremental publication", () => {
  it("matches the job identity without requiring the started NodePath", () => {
    const pending = { run_id: "run-1", job_id: "job-1", lane: "whole_game" as const, generation: 3, node_path: { indices: [] } };
    expect(matchesWholeGameJobIdentity(pending, event({ lane: "whole_game", outcome: "progress", node_path: { indices: [0] } }))).toBe(true);
    expect(matchesWholeGameJobIdentity(pending, event({ lane: "whole_game", run_id: "run-2" }))).toBe(false);
    expect(matchesWholeGameJobIdentity(null, event({ lane: "whole_game" }))).toBe(false);
  });

  it("admits progress frames as current-session node results", () => {
    expect(admitsWholeGameNodeResult(event({ lane: "whole_game", outcome: "progress", frame }))).toBe(true);
    expect(admitsWholeGameNodeResult(event({ lane: "whole_game", outcome: "completed", frame }))).toBe(false);
    expect(admitsWholeGameNodeResult(event({ lane: "whole_game", outcome: "progress", frame: undefined }))).toBe(false);
    expect(admitsWholeGameNodeResult(event({ outcome: "progress", frame }))).toBe(false);
  });
});

describe("admitsAnalysisAttachment", () => {
  const projectable = {
    ...frame,
    visits: 48,
    candidates: [{ vertex: { point: { x: 3, y: 3 } }, visits: 40, winrate_black: 0.62, score_mean_black: 2.8, pv: [] }]
  };

  it("requires projectable selected-node completed and whole-game progress frames", () => {
    expect(admitsAnalysisAttachment(event({ frame: projectable }))).toBe(true);
    expect(admitsAnalysisAttachment(event({ frame: projectable, outcome: "cancelled" }))).toBe(false);
    expect(admitsAnalysisAttachment(event({ frame: projectable, outcome: "failed" }))).toBe(false);
    expect(admitsAnalysisAttachment(event({ frame: undefined }))).toBe(false);
    expect(admitsAnalysisAttachment(event({ frame }))).toBe(false);

    expect(admitsAnalysisAttachment(event({ lane: "whole_game", outcome: "progress", frame: projectable }))).toBe(true);
    expect(admitsAnalysisAttachment(event({ lane: "whole_game", outcome: "completed", frame: projectable }))).toBe(false);
    expect(admitsAnalysisAttachment(event({
      lane: "whole_game",
      outcome: "progress",
      frame: { ...projectable, visits: 0, candidates: [] }
    }))).toBe(false);
  });
});
