import { describe, expect, it } from "vitest";
import type { SgfTreeNodeDto } from "./types";
import {
  buildNextMoveReviewMarkers,
  cycleNextMoveReviewMarker
} from "./nextMoveReviewMarker";

function lz(key: string, header: string): { key: string; values: string[] } {
  return { key, values: [`${header}\nmove D16 visits 10 winrate 5000 pv D16`] };
}

function node(properties: SgfTreeNodeDto["properties"], children: SgfTreeNodeDto[] = []): SgfTreeNodeDto {
  return { properties, children };
}

const dd = { x: 3, y: 3 };
const pp = { x: 15, y: 15 };
const pq = { x: 15, y: 16 };

describe("next-move review marker", () => {
  it("cycles Off, Variations, then Graded", () => {
    expect(cycleNextMoveReviewMarker("off")).toBe("variations");
    expect(cycleNextMoveReviewMarker("variations")).toBe("graded");
    expect(cycleNextMoveReviewMarker("graded")).toBe("off");
  });

  it("draws nothing in Off even when coordinate-bearing children exist", () => {
    const selected = node([], [
      node([{ key: "B", values: ["dd"] }]),
      node([{ key: "B", values: ["pp"] }])
    ]);
    expect(buildNextMoveReviewMarkers({
      mode: "off",
      selectedNode: selected,
      selectedIsRoot: true,
      toPlay: "black",
      boardSize: 19
    })).toEqual([]);
  });

  it("marks every coordinate-bearing child and emphasizes the Primary Child", () => {
    const selected = node([], [
      node([{ key: "B", values: ["dd"] }]),
      node([{ key: "B", values: ["pp"] }]),
      node([{ key: "B", values: ["pq"] }])
    ]);
    expect(buildNextMoveReviewMarkers({
      mode: "variations",
      selectedNode: selected,
      selectedIsRoot: true,
      toPlay: "black",
      boardSize: 19
    })).toEqual([
      { point: dd, primary: true, rank: null },
      { point: pp, primary: false, rank: null },
      { point: pq, primary: false, rank: null }
    ]);
  });

  it("keeps pass and non-coordinate children as tree nodes without board marks", () => {
    const selected = node([], [
      node([{ key: "B", values: [""] }]),
      node([{ key: "C", values: ["note"] }]),
      node([{ key: "W", values: ["tt"] }]),
      node([{ key: "B", values: ["dd"] }])
    ]);
    expect(buildNextMoveReviewMarkers({
      mode: "variations",
      selectedNode: selected,
      selectedIsRoot: true,
      toPlay: "black",
      boardSize: 19
    })).toEqual([
      { point: dd, primary: false, rank: null }
    ]);
  });

  it("grades only the Primary Child from attached positive-visit Java payloads", () => {
    const selected = node([lz("LZOP", "Engine 40.0 100")], [
      node([{ key: "B", values: ["dd"] }, lz("LZ", "Engine 52.0 100")]),
      node([{ key: "B", values: ["pp"] }, lz("LZ", "Engine 64.0 100")])
    ]);
    expect(buildNextMoveReviewMarkers({
      mode: "graded",
      selectedNode: selected,
      selectedIsRoot: true,
      toPlay: "black",
      boardSize: 19
    })).toEqual([
      { point: dd, primary: true, rank: "mistake" },
      { point: pp, primary: false, rank: null }
    ]);
  });

  it("uses the selected node's side to play for both colors", () => {
    const whiteToPlay = node([lz("LZ", "Engine 60.0 100")], [
      node([{ key: "W", values: ["dd"] }, lz("LZ", "Engine 48.0 100")])
    ]);
    expect(buildNextMoveReviewMarkers({
      mode: "graded",
      selectedNode: whiteToPlay,
      selectedIsRoot: false,
      toPlay: "white",
      boardSize: 19
    })).toEqual([
      { point: dd, primary: true, rank: "mistake" }
    ]);
  });

  it("applies every inclusive winrate-only Auto boundary to the Primary Child", () => {
    const ranks = [
      ["Engine 40.5 100", "best"],
      ["Engine 41.0 100", "good"],
      ["Engine 43.0 100", "normal"],
      ["Engine 46.0 100", "inaccuracy"],
      ["Engine 52.0 100", "mistake"],
      ["Engine 64.0 100", "blunder"]
    ] as const;
    for (const [childHeader, rank] of ranks) {
      const selected = node([lz("LZOP", "Engine 40.0 100")], [
        node([{ key: "B", values: ["dd"] }, lz("LZ", childHeader)])
      ]);
      expect(buildNextMoveReviewMarkers({
        mode: "graded",
        selectedNode: selected,
        selectedIsRoot: true,
        toPlay: "black",
        boardSize: 19
      }), childHeader).toEqual([
        { point: dd, primary: true, rank }
      ]);
    }
  });

  it("applies Auto score ordinary ranks and the winrate Mistake/Blunder guard", () => {
    const selectedGood = node([lz("LZOP", "Engine 40.0 100 2.0")], [
      node([{ key: "B", values: ["dd"] }, lz("LZ", "Engine 46.0 100 1.5")])
    ]);
    expect(buildNextMoveReviewMarkers({
      mode: "graded",
      selectedNode: selectedGood,
      selectedIsRoot: true,
      toPlay: "black",
      boardSize: 19
    })).toEqual([
      { point: dd, primary: true, rank: "good" }
    ]);

    const selectedGuard = node([lz("LZOP", "Engine 40.0 100 2.0")], [
      node([{ key: "B", values: ["dd"] }, lz("LZ", "Engine 52.0 100 1.9")])
    ]);
    expect(buildNextMoveReviewMarkers({
      mode: "graded",
      selectedNode: selectedGuard,
      selectedIsRoot: true,
      toPlay: "black",
      boardSize: 19
    })).toEqual([
      { point: dd, primary: true, rank: "mistake" }
    ]);
  });

  it("keeps variation marks without a grade for missing, malformed, zero-visit, pass-only, or absent primary analysis", () => {
    const missing = node([lz("LZOP", "Engine 40.0 100")], [
      node([{ key: "B", values: ["dd"] }])
    ]);
    const malformed = node([lz("LZOP", "Engine 40.0 100")], [
      node([{ key: "B", values: ["dd"] }, { key: "LZ", values: ["not-analysis"] }])
    ]);
    const zeroVisit = node([lz("LZOP", "Engine 40.0 100")], [
      node([{ key: "B", values: ["dd"] }, lz("LZ", "Engine 52.0 0")])
    ]);
    const passPrimary = node([lz("LZOP", "Engine 40.0 100")], [
      node([{ key: "B", values: [""] }, lz("LZ", "Engine 52.0 100")]),
      node([{ key: "B", values: ["pp"] }])
    ]);
    const noChildren = node([lz("LZOP", "Engine 40.0 100")]);

    expect(buildNextMoveReviewMarkers({
      mode: "graded",
      selectedNode: missing,
      selectedIsRoot: true,
      toPlay: "black",
      boardSize: 19
    })).toEqual([{ point: dd, primary: true, rank: null }]);
    expect(buildNextMoveReviewMarkers({
      mode: "graded",
      selectedNode: malformed,
      selectedIsRoot: true,
      toPlay: "black",
      boardSize: 19
    })).toEqual([{ point: dd, primary: true, rank: null }]);
    expect(buildNextMoveReviewMarkers({
      mode: "graded",
      selectedNode: zeroVisit,
      selectedIsRoot: true,
      toPlay: "black",
      boardSize: 19
    })).toEqual([{ point: dd, primary: true, rank: null }]);
    expect(buildNextMoveReviewMarkers({
      mode: "graded",
      selectedNode: passPrimary,
      selectedIsRoot: true,
      toPlay: "black",
      boardSize: 19
    })).toEqual([{ point: pp, primary: false, rank: null }]);
    expect(buildNextMoveReviewMarkers({
      mode: "graded",
      selectedNode: noChildren,
      selectedIsRoot: true,
      toPlay: "black",
      boardSize: 19
    })).toEqual([]);
  });
});
