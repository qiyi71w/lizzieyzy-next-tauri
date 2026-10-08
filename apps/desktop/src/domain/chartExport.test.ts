// @vitest-environment jsdom

import { describe, expect, it, vi } from "vitest";
import { captureChartExport, renderChartExport } from "./chartExport";
import { chartBaseline, displayedScore, scoreLeadText, type WinrateChartModel } from "./winrateChart";
import { renderWinrateChart } from "./renderWinrateChart";
import { createTranslator } from "../i18n/resources";

const model: WinrateChartModel = {
  points: [
    { path: { indices: [] }, moveNumber: 0, isMove: false, toPlay: "black", analysis: { visits: 100, winrateBlack: 0.6, scoreMeanBlack: 7.3 } },
    { path: { indices: [1] }, moveNumber: 1, isMove: true, toPlay: "white", analysis: null },
    { path: { indices: [1, 0] }, moveNumber: 2, isMove: true, toPlay: "black", analysis: { visits: 80, winrateBlack: 0.4, scoreMeanBlack: -4.6 } }
  ], currentMove: 1, selectedPath: { indices: [1] }, selectedToPlay: "white", perspective: "sideToPlay", scoreAvailable: true,
  showWinrate: true, showScore: true, scoreScale: 15, bars: [{ fromIndex: 0, toIndex: 2, rank: "mistake" }], hoverEnabled: true
};

it.each([[7.3, "B+7.3"], [-7.3, "W+7.3"], [4.6, "B+4.6"], [-4.6, "W+4.6"], [0, "0.0"], [-0, "0.0"], [0.04, "0.0"], [-0.04, "0.0"], [0.05, "B+0.1"], [-0.05, "W+0.1"]])("formats black-frame %s consistently under every perspective", (blackScore, text) => {
  for (const perspective of ["black", "sideToPlay"] as const) for (const side of ["black", "white"] as const) {
    const point = { ...model.points[0], analysis: { visits: 1, winrateBlack: 0.5, scoreMeanBlack: Number(blackScore) } };
    expect(scoreLeadText(displayedScore(point, perspective, side), perspective, side, "B+", "W+")).toBe(text);
  }
  expect(scoreLeadText(null, "black", "black", "B+", "W+")).toBeNull();
});

it.each([
  [true, false, true, "50%"], [false, true, true, "0"], [true, true, true, null], [false, false, false, null]
])("renders baseline for winrate=%s score=%s", (winrate, score, visible, mark) => {
  expect(chartBaseline({ ...model, showWinrate: Boolean(winrate), showScore: Boolean(score) })).toEqual({ visible, mark });
});

it("uses actual renderable metrics rather than visibility toggles for the baseline", () => {
  const noScore = { ...model, points: model.points.map((point) => ({ ...point, analysis: point.analysis ? { ...point.analysis, scoreMeanBlack: undefined } : null })) };
  expect(chartBaseline(noScore)).toEqual({ visible: true, mark: "50%" });
  expect(chartBaseline({ ...model, points: model.points.map((point) => ({ ...point, analysis: null })) })).toEqual({ visible: false, mark: null });
});

describe("invocation-time chart export", () => {
  it("freezes line, gaps, perspective, marker, scale and bars without mutating source", () => {
    const source = structuredClone(model);
    const before = structuredClone(source);
    const captured = captureChartExport(source, "C:\\games\\branch.game.SGF");
    expect(source).toEqual(before);
    expect(captured.defaultFileName).toBe("branch.game-winrate-m1.png");
    source.points[0].analysis!.scoreMeanBlack = 99;
    source.points[1].analysis = { visits: 1000, winrateBlack: 0.9 };
    source.selectedPath.indices[0] = 0;
    source.perspective = "black";
    source.showScore = false;
    source.scoreScale = 100;
    source.bars[0].rank = "blunder";
    expect(captured.model).toEqual({ ...before, hoverEnabled: false });
    expect(captureChartExport(model, null).defaultFileName).toBe("untitled-winrate-m1.png");
    expect(captureChartExport(model, "not-sgf.gib").defaultFileName).toBe("untitled-winrate-m1.png");
  });
  it("admits one valid isolated point and visibly refuses missing/invalid-only data", () => {
    expect(captureChartExport({ ...model, points: [model.points[0]] }, null).model.points).toHaveLength(1);
    for (const analysis of [null, { visits: 0, winrateBlack: 0.5 }, { visits: 1, winrateBlack: NaN }]) {
      expect(() => captureChartExport({ ...model, points: [{ ...model.points[0], analysis }] }, null)).toThrow("所选线路没有有效分析");
    }
  });
  it.each([
    { showWinrate: true, showScore: false, winrateBlack: -0.5, scoreMeanBlack: 7.3 },
    { showWinrate: false, showScore: true, winrateBlack: 0.6, scoreMeanBlack: undefined },
    { showWinrate: false, showScore: false, winrateBlack: 0.6, scoreMeanBlack: 7.3 }
  ])("refuses data available only in hidden metrics: %j", ({ showWinrate, showScore, ...metrics }) => {
    const source = {
      ...model, showWinrate, showScore,
      points: [{ ...model.points[0], analysis: { visits: 100, ...metrics } }]
    };
    expect(() => captureChartExport(source, null)).toThrow(Error);
  });
  it.each([
    { showWinrate: true, showScore: false, winrateBlack: 0.6, scoreMeanBlack: undefined },
    { showWinrate: false, showScore: true, winrateBlack: -0.5, scoreMeanBlack: 7.3 }
  ])("admits an isolated valid enabled metric: %j", ({ showWinrate, showScore, ...metrics }) => {
    const source = {
      ...model, showWinrate, showScore,
      points: [{ ...model.points[0], analysis: { visits: 100, ...metrics } }]
    };
    expect(captureChartExport(source, null).model).toEqual({ ...source, hoverEnabled: false });
  });
});

it("renders actual frozen layers, gap islands and separated fixed labels with no synthesized gap marker", () => {
  const ctx = { clearRect: vi.fn(), fillRect: vi.fn(), fillText: vi.fn(), setLineDash: vi.fn(), beginPath: vi.fn(), moveTo: vi.fn(), lineTo: vi.fn(), stroke: vi.fn(), arc: vi.fn(), fill: vi.fn() };
  renderWinrateChart(ctx as unknown as CanvasRenderingContext2D, captureChartExport(model, null).model, 1600, 600, true);
  const labels = ctx.fillText.mock.calls.map(([text]) => text);
  expect(labels).toContain("当前行棋方视角（白） · 胜率 · 目差 · 失误条");
  expect(labels).toContain("当前 1手 · 无分析");
  expect(labels).toContain("0手 · 40.0% · B+7.3");
  expect(labels).toContain("2手 · 60.0% · W+4.6");
  expect(ctx.arc).not.toHaveBeenCalled();
  // No segment may bridge the unanalyzed middle node: the only horizontal
  // lineTo calls are grid/baseline/axis; the two data islands are point fills.
  expect(ctx.lineTo.mock.calls.filter(([x, y]) => x === 1540 && y !== 191 && y !== 298 && y !== 405 && y !== 512 && y !== 84)).toHaveLength(0);
  const bar = ctx.fillRect.mock.calls.find(([x, , width]) => x === 60 && width === 1480);
  expect(bar).toBeDefined();
  expect(bar![1]).toBeCloseTo(362.2);
  expect(bar![3]).toBeCloseTo(149.8);
});

it("falls back chart semantic resources through the single foundation translator", () => {
  const translate = createTranslator("en", { en: { "chart.score": "Score lead", "chart.blackPrefix": " " } });
  expect(translate("chart.score")).toBe("Score lead");
  expect(translate("chart.blackPrefix")).toBe("B+");
  expect(translate("chart.whitePrefix")).toBe("W+");
});

it("reports unavailable canvas and pixel-capture errors before the image writer", () => {
  const getContext = vi.spyOn(HTMLCanvasElement.prototype, "getContext");
  try {
    getContext.mockReturnValue(null);
    expect(() => renderChartExport(captureChartExport(model, null))).toThrow("无法创建胜率图画布");
    const ctx = { clearRect: vi.fn(), fillRect: vi.fn(), fillText: vi.fn(), setLineDash: vi.fn(), beginPath: vi.fn(), moveTo: vi.fn(), lineTo: vi.fn(), stroke: vi.fn(), arc: vi.fn(), fill: vi.fn(), getImageData: vi.fn(() => { throw new Error("pixel capture failed"); }) };
    getContext.mockReturnValue(ctx as unknown as CanvasRenderingContext2D);
    expect(() => renderChartExport(captureChartExport(model, null))).toThrow("pixel capture failed");
    expect(ctx.getImageData).toHaveBeenCalledWith(0, 0, 1600, 600);
  } finally {
    getContext.mockRestore();
  }
});

it("plots positive scores above and negative below the unique highlighted baseline", () => {
  for (const perspective of ["black", "sideToPlay"] as const) {
    const ctx = { clearRect: vi.fn(), fillRect: vi.fn(), fillText: vi.fn(), setLineDash: vi.fn(), beginPath: vi.fn(), moveTo: vi.fn(), lineTo: vi.fn(), stroke: vi.fn(), arc: vi.fn(), fill: vi.fn() };
    renderWinrateChart(ctx as unknown as CanvasRenderingContext2D, { ...model, perspective, showWinrate: false, bars: [] }, 1600, 600, true);
    const islands = ctx.fillRect.mock.calls.filter(([, , width, height]) => width === 4 && height === 4);
    expect(islands).toHaveLength(2);
    if (perspective === "black") {
      expect(islands[0][1]).toBeLessThan(298);
      expect(islands[1][1]).toBeGreaterThan(298);
    } else {
      expect(islands[0][1]).toBeGreaterThan(298);
      expect(islands[1][1]).toBeLessThan(298);
    }
    expect(ctx.lineTo.mock.calls.filter(([x, y]) => x === 1540 && y === 298)).toHaveLength(1);
    expect(ctx.fillText.mock.calls.map(([text]) => text)).toContain("0");
  }
  const ctx = { clearRect: vi.fn(), fillRect: vi.fn(), fillText: vi.fn(), setLineDash: vi.fn(), beginPath: vi.fn(), moveTo: vi.fn(), lineTo: vi.fn(), stroke: vi.fn(), arc: vi.fn(), fill: vi.fn() };
  renderWinrateChart(ctx as unknown as CanvasRenderingContext2D, { ...model, points: model.points.map((point) => ({ ...point, analysis: null })) }, 1600, 600, true);
  expect(ctx.lineTo.mock.calls.filter(([x, y]) => x === 1540 && y === 298)).toHaveLength(0);
  expect(ctx.fillText.mock.calls.map(([text]) => text)).not.toContain("50%");
  expect(ctx.fillText.mock.calls.map(([text]) => text)).not.toContain("0");
});
