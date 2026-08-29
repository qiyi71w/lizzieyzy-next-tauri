// @vitest-environment jsdom

import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi, type Mock } from "vitest";
import type { PointDto, PositionDto } from "../domain/types";
import { BoardCanvas } from "./BoardCanvas";

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

let root: Root | null = null;
let drawArc: Mock = vi.fn();
let drawStroke: Mock = vi.fn();

beforeEach(() => {
  globalThis.IS_REACT_ACT_ENVIRONMENT = true;
  drawArc = vi.fn();
  drawStroke = vi.fn();
  vi.spyOn(HTMLCanvasElement.prototype, "getContext").mockImplementation(() => canvasContext(drawArc, drawStroke));
});

afterEach(() => {
  act(() => root?.unmount());
  root = null;
  document.body.replaceChildren();
  vi.restoreAllMocks();
});

describe("BoardCanvas keyboard intent", () => {
  it("submits the focused keyboard cursor without leaking board keys", () => {
    const onPointClick = vi.fn<(point: PointDto) => void>();
    const globalKeydown = vi.fn();
    window.addEventListener("keydown", globalKeydown);

    const { canvas } = renderBoard({ onPointClick });
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
    expect(drawArc.mock.calls.slice(-2).map((call) => call.slice(0, 2))).toEqual([
      [60.25, 60.25],
      [60.25, 60.25]
    ]);
    expect(drawStroke.mock.calls.slice(-2)).toEqual([
      ["#ffffff", 3],
      ["#2156c7", 1.5]
    ]);

    const enter = new KeyboardEvent("keydown", { key: "Enter", bubbles: true, cancelable: true });
    const space = new KeyboardEvent("keydown", { key: " ", bubbles: true, cancelable: true });
    act(() => expect(canvas.dispatchEvent(enter)).toBe(false));
    act(() => expect(canvas.dispatchEvent(space)).toBe(false));

    expect(globalKeydown).not.toHaveBeenCalled();
    expect(onPointClick).toHaveBeenNthCalledWith(1, { x: 5, y: 5 });
    expect(onPointClick).toHaveBeenNthCalledWith(2, { x: 5, y: 5 });
    expect(onPointClick).toHaveBeenCalledTimes(2);

    act(() => canvas.blur());
    window.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowLeft", bubbles: true }));
    expect(globalKeydown).toHaveBeenCalledTimes(1);
    window.removeEventListener("keydown", globalKeydown);
  });

  it("retains the cursor across blur and clamps it after a board-size change", () => {
    const onPointClick = vi.fn<(point: PointDto) => void>();
    const { canvas, rerender } = renderBoard({ onPointClick });

    act(() => canvas.focus());
    for (let index = 0; index < 10; index += 1) dispatchKey(canvas, "ArrowRight");
    act(() => canvas.blur());
    act(() => canvas.focus());
    dispatchKey(canvas, "Enter");
    expect(onPointClick).toHaveBeenLastCalledWith({ x: 8, y: 4 });

    rerender({ ...position, board_size: 5 });
    dispatchKey(canvas, "Enter");
    expect(onPointClick).toHaveBeenLastCalledWith({ x: 4, y: 4 });
  });

  it("preserves pointer coordinate submission", () => {
    const onPointClick = vi.fn<(point: PointDto) => void>();
    const { canvas } = renderBoard({ onPointClick });
    canvas.getBoundingClientRect = () => new DOMRect(0, 0, 100, 100);

    act(() => {
      canvas.dispatchEvent(new MouseEvent("click", {
        bubbles: true,
        clientX: 29.5,
        clientY: 39.75
      }));
    });

    expect(onPointClick).toHaveBeenCalledOnce();
    expect(onPointClick).toHaveBeenCalledWith({ x: 2, y: 3 });
  });
});

function dispatchKey(canvas: HTMLCanvasElement, key: string) {
  act(() => {
    canvas.dispatchEvent(new KeyboardEvent("keydown", { key, bubbles: true, cancelable: true }));
  });
}
function renderBoard({
  onPointClick,
  initialPosition = position
}: {
  onPointClick: (point: PointDto) => void;
  initialPosition?: PositionDto;
}) {
  const host = document.createElement("div");
  document.body.append(host);
  root = createRoot(host);
  const rerender = (nextPosition: PositionDto) => {
    act(() => root?.render(<BoardCanvas position={nextPosition} onPointClick={onPointClick} />));
  };
  rerender(initialPosition);
  const canvas = host.querySelector("canvas");
  if (!canvas) throw new Error("BoardCanvas did not render a canvas");
  return { canvas, rerender };
}

function canvasContext(arc: Mock, stroke: Mock): CanvasRenderingContext2D {
  let strokeStyle: string | CanvasGradient | CanvasPattern = "";
  let lineWidth = 1;
  return {
    beginPath: vi.fn(),
    arc,
    clearRect: vi.fn(),
    fill: vi.fn(),
    fillRect: vi.fn(),
    fillText: vi.fn(),
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
