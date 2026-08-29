// @vitest-environment jsdom

import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi, type Mock } from "vitest";
import type { AnalysisFrameDto, PositionDto } from "../domain/types";
import { AnalysisPanel } from "./AnalysisPanel";

declare global {
  var IS_REACT_ACT_ENVIRONMENT: boolean | undefined;
}

const position: PositionDto = {
  board_size: 9,
  move_number: 0,
  to_play: "black",
  stones: [],
  captures_black: 0,
  captures_white: 0,
  last_move: null,
  errors: []
};

const frame: AnalysisFrameDto = {
  job_id: "job-1",
  turn: 0,
  visits: 100,
  winrate_black: 0.52,
  score_mean_black: 1.5,
  candidates: [
    {
      vertex: { point: { x: 2, y: 3 } },
      visits: 100,
      winrate_black: 0.52,
      score_mean_black: 1.5,
      pv: [{ point: { x: 3, y: 3 } }]
    },
    {
      vertex: { point: { x: 6, y: 5 } },
      visits: 80,
      winrate_black: 0.49,
      score_mean_black: -0.5,
      pv: [{ point: { x: 5, y: 5 } }]
    }
  ]
};

let root: Root | null = null;
let drawArc = vi.fn();

beforeEach(() => {
  globalThis.IS_REACT_ACT_ENVIRONMENT = true;
  drawArc = vi.fn();
  vi.spyOn(HTMLCanvasElement.prototype, "getContext").mockImplementation(() => canvasContext(drawArc));
});

afterEach(() => {
  act(() => root?.unmount());
  root = null;
  document.body.replaceChildren();
  vi.restoreAllMocks();
});

describe("AnalysisPanel candidate preview", () => {
  it("renders the transient candidate on the mini-board without changing selection", () => {
    const host = document.createElement("div");
    document.body.append(host);
    root = createRoot(host);

    act(() => root?.render(
      <AnalysisPanel
        pane="reference"
        frame={frame}
        problems={[]}
        boardSize={9}
        currentMove={0}
        currentPosition={position}
        selectedCandidateIndex={0}
        previewCandidateIndex={1}
        onSelectCandidate={() => undefined}
        onSelectProblem={() => undefined}
      />
    ));

    const rows = host.querySelectorAll(".cand-row");
    expect(rows[0]?.classList.contains("is-selected")).toBe(true);
    expect(rows[1]?.classList.contains("is-selected")).toBe(false);
    expect(drawArc.mock.calls.slice(-2).map((call) => call.slice(0, 2).map((value) => Number(value.toFixed(1))))).toEqual([
      [170.4, 145.2],
      [145.2, 145.2]
    ]);
  });
});

function canvasContext(arc: Mock): CanvasRenderingContext2D {
  return {
    beginPath: vi.fn(),
    arc,
    clearRect: vi.fn(),
    fill: vi.fn(),
    fillRect: vi.fn(),
    fillText: vi.fn(),
    lineTo: vi.fn(),
    moveTo: vi.fn(),
    scale: vi.fn(),
    stroke: vi.fn()
  } as unknown as CanvasRenderingContext2D;
}
