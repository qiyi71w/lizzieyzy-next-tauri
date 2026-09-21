// @vitest-environment jsdom

import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { WinrateChart } from "./WinrateChart";
import type { WinrateChartModel } from "../domain/winrateChart";

const model: WinrateChartModel = {
  points: [
    { path: { indices: [] }, moveNumber: 0, isMove: false, toPlay: "black", analysis: null },
    { path: { indices: [0] }, moveNumber: 1, isMove: true, toPlay: "white", analysis: null },
    { path: { indices: [0, 0] }, moveNumber: 1, isMove: false, toPlay: "white", analysis: null },
    { path: { indices: [0, 0, 0] }, moveNumber: 1, isMove: false, toPlay: "black", analysis: null }
  ],
  currentMove: 1,
  selectedPath: { indices: [0, 0] },
  selectedToPlay: "white",
  perspective: "black",
  scoreAvailable: false,
  showWinrate: true,
  showScore: false,
  scoreScale: 15,
  bars: [],
  hoverEnabled: false
};

let root: Root | null = null;
const markerArc = vi.fn();

beforeEach(() => {
  globalThis.IS_REACT_ACT_ENVIRONMENT = true;
  markerArc.mockReset();
  vi.spyOn(HTMLCanvasElement.prototype, "getContext").mockReturnValue({
    setTransform: vi.fn(),
    clearRect: vi.fn(),
    fillRect: vi.fn(),
    setLineDash: vi.fn(),
    beginPath: vi.fn(),
    moveTo: vi.fn(),
    lineTo: vi.fn(),
    stroke: vi.fn(),
    arc: markerArc,
    fill: vi.fn()
  } as unknown as CanvasRenderingContext2D);
});

afterEach(() => {
  act(() => root?.unmount());
  root = null;
  document.body.replaceChildren();
  vi.restoreAllMocks();
});

describe("WinrateChart exact node selection", () => {
  it("selects root and each same-hand node independently while hover is disabled", () => {
    const onSelectNode = vi.fn();
    const host = document.createElement("div");
    document.body.append(host);
    root = createRoot(host);
    act(() => root?.render(<WinrateChart model={model} onSelectNode={onSelectNode} />));
    expect(markerArc).toHaveBeenCalledWith(160, 45, 3.5, 0, Math.PI * 2);

    const canvas = host.querySelector("canvas");
    if (!canvas) throw new Error("chart canvas missing");

    for (const clientX of [0, 80, 160, 239]) {
      act(() => canvas.dispatchEvent(new MouseEvent("click", { bubbles: true, clientX })));
    }
    expect(onSelectNode.mock.calls.map(([path]) => path.indices)).toEqual([
      [],
      [0],
      [0, 0],
      [0, 0, 0]
    ]);

    const select = host.querySelector<HTMLSelectElement>('select[aria-label="选择图表节点"]');
    if (!select) throw new Error("accessible chart node selector missing");
    expect([...select.options].map((option) => option.textContent)).toEqual([
      "根节点 · 路径 根",
      "第 1 手 · 路径 0",
      "第 1 手后节点 · 路径 0.0",
      "第 1 手后节点 · 路径 0.0.0"
    ]);
    act(() => {
      select.value = "[]";
      select.dispatchEvent(new Event("change", { bubbles: true }));
    });
    expect(onSelectNode).toHaveBeenLastCalledWith({ indices: [] });

    canvas.hidden = true;
    act(() => canvas.dispatchEvent(new MouseEvent("click", { bubbles: true, clientX: 160 })));
    expect(onSelectNode).toHaveBeenCalledTimes(5);
  });
});
