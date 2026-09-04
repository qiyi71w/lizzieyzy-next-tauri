import { describe, expect, it } from "vitest";
import type { SgfTreeNodeDto } from "./types";
import {
  admitsChartSeriesChange,
  blunderBars,
  buildWinrateChartModel,
  displayedScore,
  displayedWinrate,
  effectiveChartSeries,
  parseDisplayedAnalysis,
  parsePositiveScoreLeadScale,
  sessionScoreLeadScale,
  walkSelectedLine
} from "./winrateChart";

function lz(header: string, move = "move D16 visits 10 winrate 5000 pv D16"): { key: string; values: string[] } {
  return { key: "LZ", values: [`${header}\n${move}`] };
}

function lzop(header: string, move = "move D16 visits 10 winrate 5000 pv D16"): { key: string; values: string[] } {
  return { key: "LZOP", values: [`${header}\n${move}`] };
}

const branching: SgfTreeNodeDto = {
  properties: [lzop("MainEngine 40.0 100 2.0")],
  children: [
    {
      properties: [{ key: "B", values: ["dd"] }, lz("MainEngine 50.0 100 1.0")],
      children: [
        {
          properties: [{ key: "W", values: ["pp"] }, lz("MainEngine 62.0 100 -2.0")],
          children: []
        },
        {
          properties: [{ key: "W", values: ["pq"] }, lz("MainEngine 55.0 100")],
          children: []
        }
      ]
    }
  ]
};

describe("winrate chart selected line", () => {
  it("walks the chosen root-to-leaf variation and keeps missing analysis as a gap", () => {
    const gapped: SgfTreeNodeDto = {
      properties: [lzop("MainEngine 40.0 100")],
      children: [{
        properties: [{ key: "B", values: ["dd"] }],
        children: [{
          properties: [{ key: "W", values: ["pp"] }, lz("MainEngine 55.0 80")],
          children: []
        }]
      }]
    };
    const points = walkSelectedLine(gapped, new Map());
    expect(points.map((point) => [point.moveNumber, point.toPlay, point.analysis != null])).toEqual([
      [0, "black", true],
      [1, "white", false],
      [2, "black", true]
    ]);
  });

  it("follows chosen children onto a branch instead of first-child mainline", () => {
    const points = walkSelectedLine(branching, new Map([["0", 1]]));
    expect(points.map((point) => point.moveNumber)).toEqual([0, 1, 2]);
    expect(points[2]?.analysis?.winrateBlack).toBeCloseTo(0.45);
    expect(points[2]?.analysis?.scoreMeanBlack).toBeUndefined();
  });

  it("rejects malformed and zero-visit payloads", () => {
    expect(parseDisplayedAnalysis("not-analysis")).toBeNull();
    expect(parseDisplayedAnalysis("MainEngine 50.0 0\nmove D16 visits 0 winrate 5000 pv D16")).toBeNull();
    expect(parseDisplayedAnalysis("MainEngine 50.0 100")).toBeNull();
  });
});

describe("winrate chart encoding", () => {
  const settings = {
    graphPerspective: "black" as const,
    winrateLine: true,
    scoreLeadLine: true,
    blunderBar: true,
    graphHover: true,
    scoreLeadScale: 15
  };

  it("converts the whole visible series to selected-node side-to-play", () => {
    const model = buildWinrateChartModel({
      root: branching,
      chosen: new Map(),
      selectedPathLength: 2,
      selectedToPlay: "black",
      settings: { ...settings, graphPerspective: "sideToPlay" }
    });
    expect(displayedWinrate(model.points[0], "sideToPlay", "black")).toBeCloseTo(0.6);
    expect(displayedScore(model.points[0], "sideToPlay", "black")).toBeCloseTo(2);

    expect(displayedWinrate(model.points[0], "sideToPlay", "white")).toBeCloseTo(0.4);
    expect(displayedScore(model.points[0], "sideToPlay", "white")).toBeCloseTo(-2);
    expect(displayedWinrate(model.points[2], "sideToPlay", "white")).toBeCloseTo(0.62);
  });

  it("falls back to winrate when scoreMean is missing without treating that as neither", () => {
    const noScore: SgfTreeNodeDto = {
      properties: [lzop("MainEngine 40.0 100")],
      children: []
    };
    expect(effectiveChartSeries({ winrateLine: false, scoreLeadLine: true }, false)).toEqual({
      showWinrate: true,
      showScore: false
    });
    expect(admitsChartSeriesChange({ winrateLine: true, scoreLeadLine: true }, { winrateLine: false }, false)).toBe(false);
    expect(admitsChartSeriesChange({ winrateLine: true, scoreLeadLine: true }, { scoreLeadLine: false }, true)).toBe(true);
    expect(admitsChartSeriesChange({ winrateLine: true, scoreLeadLine: false }, { winrateLine: false }, true)).toBe(false);
    const model = buildWinrateChartModel({
      root: noScore,
      chosen: new Map(),
      selectedPathLength: 0,
      selectedToPlay: "black",
      settings: { ...settings, winrateLine: false, scoreLeadLine: true }
    });
    expect(model.scoreAvailable).toBe(false);
    expect(model.showWinrate).toBe(true);
    expect(model.showScore).toBe(false);
  });

  it("uses persisted score scale as a floor and grows only for the current session", () => {
    const points = walkSelectedLine(branching, new Map());
    expect(sessionScoreLeadScale(15, points)).toBe(15);
    expect(sessionScoreLeadScale(15, [
      { moveNumber: 0, toPlay: "black", analysis: { visits: 10, winrateBlack: 0.5, scoreMeanBlack: 21 } }
    ])).toBe(21);
    expect(parsePositiveScoreLeadScale(-1)).toBeNull();
    expect(parsePositiveScoreLeadScale("abc")).toBeNull();
    expect(parsePositiveScoreLeadScale(0)).toBeNull();
    expect(parsePositiveScoreLeadScale(15.9)).toBe(15);
  });

  it("draws Blunder Bar only for Inaccuracy, Mistake, and Blunder between adjacent displayed analyses", () => {
    const tree: SgfTreeNodeDto = {
      properties: [lzop("MainEngine 40.0 100 8.0")],
      children: [{
        properties: [{ key: "B", values: ["dd"] }, lz("MainEngine 52.0 100 2.0")],
        children: [{
          properties: [{ key: "W", values: ["pp"] }],
          children: [{
            properties: [{ key: "B", values: ["dq"] }, lz("MainEngine 40.0 100 1.9")],
            children: []
          }]
        }]
      }]
    };
    const bars = blunderBars(walkSelectedLine(tree, new Map()));
    expect(bars).toEqual([
      { fromMove: 0, toMove: 1, rank: "mistake" },
      { fromMove: 1, toMove: 3, rank: "mistake" }
    ]);
  });
});
