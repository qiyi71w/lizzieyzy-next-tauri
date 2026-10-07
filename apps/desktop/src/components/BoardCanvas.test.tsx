// @vitest-environment jsdom

import { act, useLayoutEffect } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi, type Mock } from "vitest";
import type { ReviewPresentationScope } from "../domain/reviewPresentation";
import type { AnalysisFrameDto, PointDto, PositionDto, SgfAuthoringActionDto } from "../domain/types";
import { BoardCanvas } from "./BoardCanvas";

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

const initialPreviewScope: ReviewPresentationScope = {
  generation: 1,
  selectedPath: [0],
  requestToken: "local:1"
};

const analysis: AnalysisFrameDto = {
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
let drawArc: Mock = vi.fn();
let drawStroke: Mock = vi.fn();
let drawText: Mock = vi.fn();

beforeEach(() => {
  globalThis.IS_REACT_ACT_ENVIRONMENT = true;
  drawArc = vi.fn();
  drawStroke = vi.fn();
  drawText = vi.fn();
  vi.spyOn(HTMLCanvasElement.prototype, "clientWidth", "get").mockReturnValue(100);
  vi.spyOn(HTMLCanvasElement.prototype, "clientHeight", "get").mockReturnValue(100);
  vi.spyOn(HTMLCanvasElement.prototype, "getContext").mockImplementation(() => canvasContext(drawArc, drawStroke));
});

afterEach(() => {
  act(() => root?.unmount());
  root = null;
  document.body.replaceChildren();
  vi.restoreAllMocks();
  vi.useRealTimers();
});

it("keeps rectangular corner stones inside the canvas and maps their clicks to the same vertices", () => {
  const corners: PointDto[] = [{ x: 0, y: 0 }, { x: 4, y: 0 }, { x: 0, y: 2 }, { x: 4, y: 2 }];
  const rectangular: PositionDto = {
    ...position, board_width: 5, board_height: 3,
    stones: corners.map((point) => ({ ...point, color: "black" }))
  };
  const onPointClick = vi.fn<(point: PointDto) => void>();
  const { canvas, rerender } = renderBoard({ initialPosition: rectangular, onPointClick });
  Object.defineProperties(canvas, {
    clientWidth: { configurable: true, value: 926 },
    clientHeight: { configurable: true, value: 555 }
  });
  canvas.getBoundingClientRect = () => new DOMRect(0, 0, 926, 555);
  drawArc.mockClear();
  rerender({ ...rectangular });
  const stones = drawArc.mock.calls.slice(-4);
  for (const [index, [x, y, radius]] of stones.entries()) {
    expect(x - radius).toBeGreaterThanOrEqual(0);
    expect(y - radius).toBeGreaterThanOrEqual(0);
    expect(x + radius).toBeLessThanOrEqual(926);
    expect(y + radius).toBeLessThanOrEqual(555);
    act(() => canvas.dispatchEvent(new MouseEvent("click", { bubbles: true, clientX: x, clientY: y })));
    expect(onPointClick).toHaveBeenLastCalledWith(corners[index]);
  }
});

describe("BoardCanvas keyboard intent", () => {
  it("does not consume review keys until explicit placement mode is on", () => {
    const onPointClick = vi.fn<(point: PointDto) => void>();
    const globalKeydown = vi.fn();
    window.addEventListener("keydown", globalKeydown);

    const { canvas } = renderBoard({ onPointClick });
    act(() => canvas.focus());
    const right = new KeyboardEvent("keydown", { key: "ArrowRight", bubbles: true, cancelable: true });
    const enter = new KeyboardEvent("keydown", { key: "Enter", bubbles: true, cancelable: true });
    const space = new KeyboardEvent("keydown", { key: " ", bubbles: true, cancelable: true });
    act(() => expect(canvas.dispatchEvent(right)).toBe(true));
    act(() => expect(canvas.dispatchEvent(enter)).toBe(true));
    act(() => expect(canvas.dispatchEvent(space)).toBe(true));
    expect(globalKeydown).toHaveBeenCalledTimes(3);
    expect(onPointClick).not.toHaveBeenCalled();
    window.removeEventListener("keydown", globalKeydown);
  });

  it("submits the focused keyboard cursor without leaking board keys", () => {
    const onPointClick = vi.fn<(point: PointDto) => void>();
    const globalKeydown = vi.fn();
    window.addEventListener("keydown", globalKeydown);

    const { canvas } = renderBoard({ onPointClick, keyboardPlacement: true });
    expect(canvas.getAttribute("aria-label")).toBe("棋盘");
    expect(canvas.tabIndex).toBe(0);
    expect(canvas.getAttribute("role")).toBe("application");
    Object.defineProperties(canvas, {
      clientWidth: { configurable: true, value: 100 },
      clientHeight: { configurable: true, value: 100 }
    });

    act(() => canvas.focus());
    expect(document.activeElement).toBe(canvas);

    const right = new KeyboardEvent("keydown", { key: "ArrowRight", bubbles: true, cancelable: true });
    const down = new KeyboardEvent("keydown", { key: "ArrowDown", bubbles: true, cancelable: true });
    act(() => expect(canvas.dispatchEvent(right)).toBe(false));
    act(() => expect(canvas.dispatchEvent(down)).toBe(false));
    expect(globalKeydown).not.toHaveBeenCalled();

    const enter = new KeyboardEvent("keydown", { key: "Enter", bubbles: true, cancelable: true });
    const space = new KeyboardEvent("keydown", { key: " ", bubbles: true, cancelable: true });
    const shiftEnter = new KeyboardEvent("keydown", { key: "Enter", shiftKey: true, bubbles: true, cancelable: true });
    act(() => expect(canvas.dispatchEvent(enter)).toBe(false));
    act(() => expect(canvas.dispatchEvent(space)).toBe(true));
    act(() => expect(canvas.dispatchEvent(shiftEnter)).toBe(true));

    expect(globalKeydown).toHaveBeenCalledTimes(2);
    expect(onPointClick).toHaveBeenCalledTimes(1);
    expect(onPointClick).toHaveBeenCalledWith({ x: 5, y: 5 });

    act(() => canvas.blur());
    window.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowLeft", bubbles: true }));
    expect(globalKeydown).toHaveBeenCalledTimes(3);
    window.removeEventListener("keydown", globalKeydown);
  });

  it("retains the cursor across blur and clamps it after a board-size change", () => {
    const onPointClick = vi.fn<(point: PointDto) => void>();
    const { canvas, rerender } = renderBoard({ onPointClick, keyboardPlacement: true });

    act(() => canvas.focus());
    for (let index = 0; index < 10; index += 1) dispatchKey(canvas, "ArrowRight");
    act(() => canvas.blur());
    act(() => canvas.focus());
    dispatchKey(canvas, "Enter");
    expect(onPointClick).toHaveBeenLastCalledWith({ x: 8, y: 4 });

    rerender({ ...position, board_width: 5, board_height: 7 });
    dispatchKey(canvas, "Enter");
    expect(onPointClick).toHaveBeenLastCalledWith({ x: 4, y: 4 });
  });

  it("preserves pointer coordinate submission", () => {
    const onPointClick = vi.fn<(point: PointDto) => void>();
    const rectangular = { ...position, board_width: 9, board_height: 5 };
    const { canvas, rerender } = renderBoard({ initialPosition: rectangular, onPointClick });
    Object.defineProperty(canvas, "clientWidth", { configurable: true, value: 160 });
    canvas.getBoundingClientRect = () => new DOMRect(0, 0, 160, 100);
    rerender({ ...rectangular });

    act(() => {
      canvas.dispatchEvent(new MouseEvent("click", {
        bubbles: true,
        ...renderedPoint(2, 3, 5)
      }));
    });

    expect(onPointClick).toHaveBeenCalledOnce();
    expect(onPointClick).toHaveBeenCalledWith({ x: 2, y: 3 });
  });
});

describe("BoardCanvas candidate preview", () => {
  it("publishes a candidate only after the 120 ms dwell and clears it on exit", () => {
    vi.useFakeTimers();
    const onCandidatePreview = vi.fn<(index: number | null) => void>();
    const { canvas } = renderBoard({ analysis, onCandidatePreview });
    setCanvasBounds(canvas);

    dispatchPointer(canvas, "pointermove", 2, 3);
    act(() => vi.advanceTimersByTime(119));
    expect(onCandidatePreview).not.toHaveBeenCalled();

    act(() => vi.advanceTimersByTime(1));
    expect(onCandidatePreview).toHaveBeenLastCalledWith(0);

    dispatchPointer(canvas, "pointerout", 2, 3);
    expect(onCandidatePreview).toHaveBeenLastCalledWith(null);
  });

  it("submits a quick click without waiting for or publishing its pending preview", () => {
    vi.useFakeTimers();
    const onCandidatePreview = vi.fn<(index: number | null) => void>();
    const onPointClick = vi.fn<(point: PointDto) => void>();
    const { canvas } = renderBoard({ analysis, onCandidatePreview, onPointClick });
    setCanvasBounds(canvas);

    dispatchPointer(canvas, "pointermove", 2, 3);
    act(() => vi.advanceTimersByTime(60));
    act(() => {
      canvas.dispatchEvent(new MouseEvent("click", {
        bubbles: true,
        ...renderedPoint(2, 3)
      }));
    });
    act(() => vi.runAllTimers());

    expect(onPointClick).toHaveBeenCalledWith({ x: 2, y: 3 });
    expect(onCandidatePreview).not.toHaveBeenCalled();
  });

  it("replaces a pending candidate and clears a visible preview before click submission", () => {
    vi.useFakeTimers();
    const onCandidatePreview = vi.fn<(index: number | null) => void>();
    const onPointClick = vi.fn<(point: PointDto) => void>();
    const { canvas } = renderBoard({ analysis, onCandidatePreview, onPointClick });
    setCanvasBounds(canvas);

    dispatchPointer(canvas, "pointermove", 2, 3);
    act(() => vi.advanceTimersByTime(60));
    dispatchPointer(canvas, "pointermove", 6, 5);
    act(() => vi.advanceTimersByTime(119));
    expect(onCandidatePreview).not.toHaveBeenCalled();
    act(() => vi.advanceTimersByTime(1));
    expect(onCandidatePreview).toHaveBeenLastCalledWith(1);

    act(() => {
      canvas.dispatchEvent(new MouseEvent("click", {
        bubbles: true,
        ...renderedPoint(6, 5)
      }));
    });
    expect(onCandidatePreview).toHaveBeenLastCalledWith(null);
    expect(onPointClick).toHaveBeenCalledWith({ x: 6, y: 5 });

    act(() => vi.runAllTimers());
    expect(onCandidatePreview).toHaveBeenCalledTimes(2);
  });

  it("cancels pending and visible preview when its presentation scope changes", () => {
    vi.useFakeTimers();
    const onCandidatePreview = vi.fn<(index: number | null) => void>();
    const { canvas, rerender } = renderBoard({
      analysis,
      onCandidatePreview,
      previewScope: initialPreviewScope
    });
    setCanvasBounds(canvas);

    dispatchPointer(canvas, "pointermove", 2, 3);
    act(() => vi.advanceTimersByTime(120));
    expect(onCandidatePreview).toHaveBeenLastCalledWith(0);

    rerender(position, { ...initialPreviewScope, selectedPath: [1] });
    expect(onCandidatePreview).toHaveBeenLastCalledWith(null);

    dispatchPointer(canvas, "pointermove", 2, 3);
    act(() => vi.advanceTimersByTime(60));
    rerender(position, { ...initialPreviewScope, selectedPath: [2] });
    act(() => vi.runAllTimers());
    expect(onCandidatePreview).toHaveBeenCalledTimes(2);
  });

  it("keeps a pending preview when its scope is value-equivalent", () => {
    vi.useFakeTimers();
    const onCandidatePreview = vi.fn<(index: number | null) => void>();
    const { canvas, rerender } = renderBoard({
      analysis,
      onCandidatePreview,
      previewScope: initialPreviewScope
    });
    setCanvasBounds(canvas);

    dispatchPointer(canvas, "pointermove", 2, 3);
    act(() => vi.advanceTimersByTime(60));
    rerender(position, { ...initialPreviewScope, selectedPath: [0] });
    act(() => vi.advanceTimersByTime(60));

    expect(onCandidatePreview).toHaveBeenLastCalledWith(0);
  });

  it("does not publish an old deadline during a scope-changing commit", () => {
    vi.useFakeTimers();
    const onCandidatePreview = vi.fn<(index: number | null) => void>();
    const host = document.createElement("div");
    document.body.append(host);
    root = createRoot(host);
    const initialScope = initialPreviewScope;

    act(() => root?.render(
      <ScopeTransitionBoard
        previewScope={initialScope}
        onCandidatePreview={onCandidatePreview}
        advanceDeadline={false}
      />
    ));
    const canvas = host.querySelector("canvas");
    if (!canvas) throw new Error("BoardCanvas did not render a canvas");
    setCanvasBounds(canvas);
    dispatchPointer(canvas, "pointermove", 2, 3);
    act(() => vi.advanceTimersByTime(119));

    act(() => root?.render(
      <ScopeTransitionBoard
        previewScope={{ ...initialScope, selectedPath: [1] }}
        onCandidatePreview={onCandidatePreview}
        advanceDeadline
      />
    ));

    expect(onCandidatePreview).not.toHaveBeenCalled();
  });
});


describe("BoardCanvas next-move review markers", () => {
  const variationMarkers = [
    { point: { x: 3, y: 3 }, primary: true, rank: null },
    { point: { x: 6, y: 5 }, primary: false, rank: null }
  ];

  it("draws nothing in Off", () => {
    const { canvas, host, rerender } = renderBoard({
      nextMoveMode: "off",
      nextMoveMarkers: variationMarkers
    });
    paintAtCssSize(canvas, rerender);
    expect(host.querySelector(".board-canvas")?.getAttribute("data-next-move-mode")).toBe("off");
    expect(drawStroke.mock.calls.some((call) => call[0] === "#163f96" || call[0] === "rgba(33,86,199,.55)")).toBe(false);
  });

  it("marks all children and uses a thicker Primary Child ring", () => {
    const { canvas, host, rerender } = renderBoard({
      nextMoveMode: "variations",
      nextMoveMarkers: variationMarkers
    });
    paintAtCssSize(canvas, rerender);
    expect(host.querySelector(".board-canvas")?.getAttribute("data-next-move-mode")).toBe("variations");
    const primary = renderedPoint(3, 3);
    const secondary = renderedPoint(6, 5);
    expect(drawArc.mock.calls.some((call) => call[0] === primary.clientX && call[1] === primary.clientY)).toBe(true);
    expect(drawArc.mock.calls.some((call) => call[0] === secondary.clientX && call[1] === secondary.clientY)).toBe(true);
    const primaryStroke = drawStroke.mock.calls.find((call) => call[0] === "#163f96");
    const secondaryStroke = drawStroke.mock.calls.find((call) => call[0] === "rgba(33,86,199,.55)");
    expect(primaryStroke?.[1]).toBeGreaterThan(secondaryStroke?.[1]);
  });

  it("keeps the Primary Child grade visible in Graded", () => {
    const graded = [
      { point: { x: 3, y: 3 }, primary: true, rank: "blunder" as const },
      { point: { x: 6, y: 5 }, primary: false, rank: null }
    ];
    const { host } = renderBoard({
      nextMoveMode: "graded",
      nextMoveMarkers: graded
    });
    expect(host.querySelector(".board-canvas")?.getAttribute("data-next-move-markers")).toBe(JSON.stringify(graded));
  });

  it("keeps next-move marks independent of candidate overlay mode", () => {
    const { host } = renderBoard({
      overlayMode: "ownership",
      nextMoveMode: "variations",
      nextMoveMarkers: variationMarkers
    });
    expect(host.querySelector(".board-canvas")?.getAttribute("data-next-move-mode")).toBe("variations");
    expect(JSON.parse(host.querySelector(".board-canvas")?.getAttribute("data-next-move-markers") ?? "[]")).toEqual(variationMarkers);
  });
});

describe("BoardCanvas authoring gestures", () => {
  function authorBoard(allowDrag = true) {
    const onAuthoring = vi.fn(); const onPointClick = vi.fn(); const onGestureRefused = vi.fn();
    const rendered = renderBoard({ initialPosition: { ...position, stones: [{ x: 0, y: 0, color: "black" }] }, previewScope: initialPreviewScope,
      allowDrag, authoringEnabled: true, onAuthoring, onPointClick, onGestureRefused });
    rendered.canvas.getBoundingClientRect = () => new DOMRect(0, 0, 500, 500);
    rendered.canvas.setPointerCapture = vi.fn();
    return { ...rendered, onAuthoring, onPointClick, onGestureRefused };
  }
  function pointer(canvas: HTMLCanvasElement, type: string, x: number, y: number) {
    const event = new MouseEvent(type, { bubbles: true, clientX: x, clientY: y, button: 0 });
    Object.defineProperty(event, "pointerId", { value: 1 });
    act(() => canvas.dispatchEvent(event));
  }
  it("emits one recorded-stone drag with the start identity and suppresses ordinary play", () => {
    const { canvas, onAuthoring, onPointClick } = authorBoard();
    pointer(canvas,"pointerdown",68,68); pointer(canvas,"pointerup",159,159); pointer(canvas,"click",159,159);
    expect(onAuthoring).toHaveBeenCalledExactlyOnceWith({ kind: "drag", from: { x:0,y:0 }, to: {x:2,y:2} },initialPreviewScope);
    expect(onPointClick).not.toHaveBeenCalled();
  });
  it("does not admit drag without the independent permission", () => {
    const { canvas, onAuthoring } = authorBoard(false);
    pointer(canvas,"pointerdown",68,68); pointer(canvas,"pointerup",159,159);
    expect(onAuthoring).not.toHaveBeenCalled();
  });
  it("cancels Escape, pointercancel and lost capture without authoring or ordinary-play fallthrough", () => {
    const { canvas, onAuthoring, onPointClick } = authorBoard();
    for (const type of ["Escape", "pointercancel", "lostpointercapture"]) {
      pointer(canvas,"pointerdown",68,68);
      if (type === "Escape") dispatchKey(canvas,"Escape"); else pointer(canvas,type,159,159);
      pointer(canvas,"pointerup",159,159); pointer(canvas,"click",159,159);
    }
    expect(onAuthoring).not.toHaveBeenCalled(); expect(onPointClick).not.toHaveBeenCalled();
  });
  it("rejects changed cursor identity and off-board release visibly", () => {
    const { canvas, rerender, onAuthoring, onGestureRefused } = authorBoard();
    pointer(canvas,"pointerdown",68,68);
    rerender({ ...position, stones: [{ x:0,y:0,color:"black" }] },{ ...initialPreviewScope, selectedPath:[1] });
    pointer(canvas,"pointerup",159,159);
    expect(onAuthoring).not.toHaveBeenCalled(); expect(onGestureRefused).toHaveBeenLastCalledWith("棋谱或所选节点已变化；本次手势未提交。");
    pointer(canvas,"pointerdown",68,68); pointer(canvas,"pointerup",700,700);
    expect(onAuthoring).not.toHaveBeenCalled(); expect(onGestureRefused).toHaveBeenLastCalledWith("拖动目标超出棋盘；原谱未更改。");
  });
  it("runs context insertion on its captured point and Cancel never edits", () => {
    const { canvas, host, onAuthoring } = authorBoard();
    pointer(canvas,"contextmenu",159,159);
    const white = [...host.querySelectorAll<HTMLButtonElement>('[role="menuitem"]')].find(b=>b.textContent==="列表插入白子");
    act(()=>white?.click());
    expect(onAuthoring).toHaveBeenCalledExactlyOnceWith({ kind:"add",point:{x:2,y:2},color:"white",insert:true },initialPreviewScope);
    pointer(canvas,"contextmenu",159,159);
    act(()=>[...host.querySelectorAll<HTMLButtonElement>('[role="menuitem"]')].find(b=>b.textContent==="取消棋子编辑")?.click());
    expect(onAuthoring).toHaveBeenCalledTimes(1);
  });
  it("keeps double-click permission independent of ordinary click and drag", () => {
    const onPointClick=vi.fn(); const {canvas}=renderBoard({onPointClick,allowDoubleClick:false});
    canvas.getBoundingClientRect=()=>new DOMRect(0,0,500,500);
    for (const detail of [1,2]) act(()=>canvas.dispatchEvent(new MouseEvent("click",{bubbles:true,clientX:159,clientY:159,detail})));
    expect(onPointClick).toHaveBeenCalledTimes(1);
  });
});

function dispatchKey(canvas: HTMLCanvasElement, key: string) {
  act(() => {
    canvas.dispatchEvent(new KeyboardEvent("keydown", { key, bubbles: true, cancelable: true }));
  });
}
function renderBoard({
  onPointClick = () => undefined,
  onCandidatePreview,
  analysis: boardAnalysis,
  previewScope,
  initialPosition = position,
  overlayMode,
  nextMoveMode,
  nextMoveMarkers,
  keyboardPlacement,
  allowDrag, allowDoubleClick, authoringEnabled, onAuthoring, onGestureRefused
}: {
  onPointClick?: (point: PointDto) => void;
  onCandidatePreview?: (index: number | null) => void;
  analysis?: AnalysisFrameDto;
  previewScope?: ReviewPresentationScope;
  initialPosition?: PositionDto;
  overlayMode?: "candidates" | "ownership" | "policy";
  nextMoveMode?: "off" | "variations" | "graded";
  nextMoveMarkers?: Array<{ point: { x: number; y: number }; primary: boolean; rank: "best" | "good" | "normal" | "inaccuracy" | "mistake" | "blunder" | null }>;
  keyboardPlacement?: boolean;
  allowDrag?: boolean;
  allowDoubleClick?: boolean;
  authoringEnabled?: boolean;
  onAuthoring?: (action: SgfAuthoringActionDto, captured: ReviewPresentationScope) => void;
  onGestureRefused?: (message: string) => void;
}) {
  const host = document.createElement("div");
  document.body.append(host);
  root = createRoot(host);
  const rerender = (nextPosition: PositionDto, nextPreviewScope = previewScope) => {
    act(() => root?.render(
      <BoardCanvas
        position={nextPosition}
        analysis={boardAnalysis}
        onPointClick={onPointClick}
        onCandidatePreview={onCandidatePreview}
        previewScope={nextPreviewScope}
        overlayMode={overlayMode}
        nextMoveMode={nextMoveMode}
        nextMoveMarkers={nextMoveMarkers}
        keyboardPlacement={keyboardPlacement}
        allowDrag={allowDrag}
        allowDoubleClick={allowDoubleClick}
        authoringEnabled={authoringEnabled}
        onAuthoring={onAuthoring}
        onGestureRefused={onGestureRefused}
      />
    ));
  };
  rerender(initialPosition);
  const canvas = host.querySelector("canvas");
  if (!canvas) throw new Error("BoardCanvas did not render a canvas");
  return { canvas, rerender, host };
}

function paintAtCssSize(canvas: HTMLCanvasElement, rerender: (position: PositionDto) => void) {
  Object.defineProperties(canvas, {
    clientWidth: { configurable: true, value: 100 },
    clientHeight: { configurable: true, value: 100 }
  });
  rerender({ ...position });
}

function ScopeTransitionBoard({
  previewScope,
  onCandidatePreview,
  advanceDeadline
}: {
  previewScope: ReviewPresentationScope;
  onCandidatePreview: (index: number | null) => void;
  advanceDeadline: boolean;
}) {
  useLayoutEffect(() => {
    if (advanceDeadline) vi.advanceTimersByTime(1);
  }, [advanceDeadline, previewScope]);
  return (
    <BoardCanvas
      position={position}
      analysis={analysis}
      previewScope={previewScope}
      onCandidatePreview={onCandidatePreview}
    />
  );
}
it("keeps every context action inside the viewport at the lower-right board edge", () => {
  const scope: ReviewPresentationScope = { generation: 1, requestToken: "edge-menu", selectedPath: [] };
  const { canvas, host } = renderBoard({ authoringEnabled: true, previewScope: scope, onAuthoring: vi.fn() });
  canvas.getBoundingClientRect = () => new DOMRect(window.innerWidth - 100, window.innerHeight - 100, 100, 100);
  const bounds = vi.spyOn(HTMLElement.prototype, "getBoundingClientRect").mockReturnValue(new DOMRect(0, 0, 240, 220));
  act(() => canvas.dispatchEvent(new MouseEvent("contextmenu", { bubbles: true, clientX: window.innerWidth - 50, clientY: window.innerHeight - 50 })));
  const menu = host.querySelector<HTMLElement>('[role="menu"]');
  expect(menu?.style.left).toBe(`${window.innerWidth - 240}px`);
  expect(menu?.style.top).toBe(`${window.innerHeight - 220}px`);
  expect(menu?.querySelectorAll('[role="menuitem"]').length).toBe(7);
  bounds.mockRestore();
});

function setCanvasBounds(canvas: HTMLCanvasElement) {
  canvas.getBoundingClientRect = () => new DOMRect(0, 0, 100, 100);
}

function renderedPoint(x: number, y: number, boardHeight = 9) {
  const column = drawText.mock.calls.find((call) => call[0] === "ABCDEFGHJKLMNOPQRSTUVWXYZ"[x]);
  const row = drawText.mock.calls.find((call) => call[0] === String(boardHeight - y));
  if (!column || !row) throw new Error("Board coordinates were not rendered");
  return { clientX: column[1] as number, clientY: row[2] as number };
}

function dispatchPointer(canvas: HTMLCanvasElement, type: "pointermove" | "pointerout", x: number, y: number) {
  act(() => {
    canvas.dispatchEvent(new MouseEvent(type, { bubbles: true, ...renderedPoint(x, y) }));
  });
}

function canvasContext(arc: Mock, stroke: Mock): CanvasRenderingContext2D {
  let strokeStyle: string | CanvasGradient | CanvasPattern = "";
  let lineWidth = 1;
  return {
    beginPath: vi.fn(),
    arc,
    clearRect: vi.fn(() => drawText.mockClear()),
    fill: vi.fn(),
    fillRect: vi.fn(),
    fillText: drawText,
    get lineWidth() {
      return lineWidth;
    },
    set lineWidth(value) {
      lineWidth = value;
    },
    lineTo: vi.fn(),
    moveTo: vi.fn(),
    restore: vi.fn(),
    save: vi.fn(),
    scale: vi.fn(),
    stroke: vi.fn(() => stroke(strokeStyle, lineWidth)),
    get strokeStyle() {
      return strokeStyle;
    },
    set strokeStyle(value) {
      strokeStyle = value;
    },
    strokeRect: vi.fn()
  } as unknown as CanvasRenderingContext2D;
}
