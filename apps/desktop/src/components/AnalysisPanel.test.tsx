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
  board_width: 9,
  board_height: 9,
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
let fillText = vi.fn();

beforeEach(() => {
  globalThis.IS_REACT_ACT_ENVIRONMENT = true;
  vi.spyOn(HTMLCanvasElement.prototype, "clientWidth", "get").mockReturnValue(240);
  vi.spyOn(HTMLCanvasElement.prototype, "clientHeight", "get").mockReturnValue(240);
  drawArc = vi.fn();
  fillText = vi.fn();
  vi.spyOn(HTMLCanvasElement.prototype, "getContext").mockImplementation(() => canvasContext(drawArc, fillText));
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
        boardWidth={9}
        boardHeight={9}
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

describe("AnalysisPanel sub-board content mode", () => {
  it("suppresses PV, branch, and move numbers in Raw even with a live hover candidate", () => {
    const host = document.createElement("div");
    document.body.append(host);
    root = createRoot(host);
    const stonesOnBoard = { ...position, stones: [{ x: 4, y: 4, color: "black" as const }] };

    act(() => root?.render(
      <AnalysisPanel
        pane="reference"
        frame={frame}
        problems={[]}
        boardWidth={9}
        boardHeight={9}
        currentMove={0}
        currentPosition={stonesOnBoard}
        selectedCandidateIndex={0}
        previewCandidateIndex={1}
        contentMode="raw"
        onSelectCandidate={() => undefined}
        onSelectProblem={() => undefined}
      />
    ));

    expect(host.querySelector(".subboard-canvas")?.getAttribute("aria-label")).toBe("纯棋子副棋盘");
    expect(fillText.mock.calls.map((call) => call[0])).toEqual([]);
    expect(drawArc.mock.calls.some((call) => Number(call[0].toFixed(1)) === 120 && Number(call[1].toFixed(1)) === 120)).toBe(true);
    expect(drawArc.mock.calls.some((call) => Number(call[0].toFixed(1)) === 170.4 && Number(call[1].toFixed(1)) === 145.2)).toBe(false);
  });
});
describe("AnalysisPanel personal comment draft", () => {
  it("retains comment draft on blur without invoking onCommitPersonalComment and commits on explicit Apply", () => {
    const host = document.createElement("div");
    document.body.append(host);
    root = createRoot(host);
    const onCommit = vi.fn();

    act(() => root?.render(
      <AnalysisPanel
        pane="commentary"
        problems={[]}
        boardWidth={9}
        boardHeight={9}
        currentMove={0}
        currentPosition={position}
        selectedCandidateIndex={null}
        onSelectCandidate={() => undefined}
        onSelectProblem={() => undefined}
        commentEditorEnabled={true}
        personalComment="initial comment"
        onCommitPersonalComment={onCommit}
      />
    ));

    const editor = host.querySelector('textarea[aria-label="个人评论"]') as HTMLTextAreaElement;
    expect(editor.value).toBe("initial comment");

    act(() => {
      const valueSetter = Object.getOwnPropertyDescriptor(HTMLTextAreaElement.prototype, "value")?.set;
      valueSetter?.call(editor, "updated draft");
      editor.dispatchEvent(new Event("input", { bubbles: true }));
      editor.dispatchEvent(new Event("change", { bubbles: true }));
    });
    expect(editor.value).toBe("updated draft");

    act(() => {
      editor.dispatchEvent(new Event("blur", { bubbles: true }));
    });
    expect(onCommit).not.toHaveBeenCalled();
    expect(editor.value).toBe("updated draft");

    const applyButton = Array.from(host.querySelectorAll("button")).find((btn) => btn.textContent === "应用评论") as HTMLButtonElement;
    expect(applyButton).toBeInstanceOf(HTMLButtonElement);

    act(() => {
      applyButton.click();
    });
    expect(onCommit).toHaveBeenCalledExactlyOnceWith("updated draft");
  });

  it("retains draft while container is hidden and preserves draft until explicit Apply", () => {
    const host = document.createElement("div");
    document.body.append(host);
    root = createRoot(host);
    const onCommit = vi.fn();

    act(() => root?.render(
      <AnalysisPanel
        pane="commentary"
        problems={[]}
        boardWidth={9}
        boardHeight={9}
        currentMove={0}
        currentPosition={position}
        selectedCandidateIndex={null}
        onSelectCandidate={() => undefined}
        onSelectProblem={() => undefined}
        commentEditorEnabled={true}
        personalComment="persisted comment"
        onCommitPersonalComment={onCommit}
      />
    ));

    const editor = host.querySelector('textarea[aria-label="个人评论"]') as HTMLTextAreaElement;
    act(() => {
      const valueSetter = Object.getOwnPropertyDescriptor(HTMLTextAreaElement.prototype, "value")?.set;
      valueSetter?.call(editor, "rail03 un-applied draft");
      editor.dispatchEvent(new Event("input", { bubbles: true }));
      editor.dispatchEvent(new Event("change", { bubbles: true }));
    });

    host.hidden = true;
    act(() => {
      editor.dispatchEvent(new Event("blur", { bubbles: true }));
    });
    expect(onCommit).not.toHaveBeenCalled();
    expect(editor.value).toBe("rail03 un-applied draft");

    host.hidden = false;
    expect(editor.value).toBe("rail03 un-applied draft");

    const applyButton = Array.from(host.querySelectorAll("button")).find((btn) => btn.textContent === "应用评论") as HTMLButtonElement;
    act(() => {
      applyButton.click();
    });
    expect(onCommit).toHaveBeenCalledExactlyOnceWith("rail03 un-applied draft");
  });

  it("resets comment draft when selected node personalComment changes", () => {
    const host = document.createElement("div");
    document.body.append(host);
    root = createRoot(host);

    act(() => root?.render(
      <AnalysisPanel
        pane="commentary"
        problems={[]}
        boardWidth={9}
        boardHeight={9}
        currentMove={0}
        currentPosition={position}
        selectedCandidateIndex={null}
        onSelectCandidate={() => undefined}
        onSelectProblem={() => undefined}
        commentEditorEnabled={true}
        personalComment="node 1 comment"
      />
    ));

    const editor = host.querySelector('textarea[aria-label="个人评论"]') as HTMLTextAreaElement;
    expect(editor.value).toBe("node 1 comment");

    act(() => {
      const valueSetter = Object.getOwnPropertyDescriptor(HTMLTextAreaElement.prototype, "value")?.set;
      valueSetter?.call(editor, "uncommitted change");
      editor.dispatchEvent(new Event("input", { bubbles: true }));
      editor.dispatchEvent(new Event("change", { bubbles: true }));
    });
    expect(editor.value).toBe("uncommitted change");

    act(() => root?.render(
      <AnalysisPanel
        pane="commentary"
        problems={[]}
        boardWidth={9}
        boardHeight={9}
        currentMove={0}
        currentPosition={position}
        selectedCandidateIndex={null}
        onSelectCandidate={() => undefined}
        onSelectProblem={() => undefined}
        commentEditorEnabled={true}
        personalComment="node 2 comment"
      />
    ));
    expect(editor.value).toBe("node 2 comment");
  });

  it("resets an unapplied draft when switching nodes with equal persisted comments", () => {
    const host = document.createElement("div");
    document.body.append(host);
    root = createRoot(host);
    const onCommit = vi.fn();
    const renderNode = (index: number) => act(() => root?.render(
      <AnalysisPanel problems={[]} boardWidth={9} boardHeight={9} currentMove={index}
        selectedPath={{ indices: [index] }} selectedCandidateIndex={null}
        onSelectCandidate={() => undefined} onSelectProblem={() => undefined}
        commentEditorEnabled personalComment="" onCommitPersonalComment={onCommit} />
    ));
    renderNode(0);
    const editor = host.querySelector('textarea[aria-label="个人评论"]') as HTMLTextAreaElement;
    act(() => {
      Object.getOwnPropertyDescriptor(HTMLTextAreaElement.prototype, "value")?.set?.call(editor, "node A draft");
      editor.dispatchEvent(new Event("input", { bubbles: true }));
    });
    renderNode(0);
    expect(editor.value).toBe("node A draft");
    renderNode(1);
    expect(editor.value).toBe("");
    expect(onCommit).not.toHaveBeenCalled();
  });

  it("preserves generated information isolation alongside personal comment editor", () => {
    const host = document.createElement("div");
    document.body.append(host);
    root = createRoot(host);

    act(() => root?.render(
      <AnalysisPanel
        pane="commentary"
        problems={[]}
        boardWidth={9}
        boardHeight={9}
        currentMove={0}
        currentPosition={position}
        selectedCandidateIndex={null}
        onSelectCandidate={() => undefined}
        onSelectProblem={() => undefined}
        commentEditorEnabled={true}
        personalComment="human note"
        generatedInformation="KataGo generated analysis"
      />
    ));

    const editor = host.querySelector('textarea[aria-label="个人评论"]') as HTMLTextAreaElement;
    expect(editor.value).toBe("human note");
    expect(host.querySelector(".generated-information")?.textContent).toBe("KataGo generated analysis");
  });
});


function canvasContext(arc: Mock, text: Mock = vi.fn()): CanvasRenderingContext2D {
  return {
    beginPath: vi.fn(),
    arc,
    clearRect: vi.fn(),
    fill: vi.fn(),
    fillRect: vi.fn(),
    fillText: text,
    lineTo: vi.fn(),
    moveTo: vi.fn(),
    scale: vi.fn(),
    stroke: vi.fn()
  } as unknown as CanvasRenderingContext2D;
}
