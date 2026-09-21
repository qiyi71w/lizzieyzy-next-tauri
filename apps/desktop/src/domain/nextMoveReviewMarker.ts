import { classifyPlayedMove, type MoveRank, type PlayerColor } from "./moveRank";
import type { PointDto, SgfTreeNodeDto } from "./types";
import { displayedPrimaryAnalysis } from "./winrateChart";

export type NextMoveReviewMarkerMode = "off" | "variations" | "graded";

export type NextMoveReviewMarker = {
  point: PointDto;
  primary: boolean;
  rank: MoveRank | null;
};

const MODES: NextMoveReviewMarkerMode[] = ["off", "variations", "graded"];

export function cycleNextMoveReviewMarker(mode: NextMoveReviewMarkerMode): NextMoveReviewMarkerMode {
  const index = MODES.indexOf(mode);
  return MODES[(index < 0 ? 1 : index + 1) % MODES.length];
}

export function buildNextMoveReviewMarkers(input: {
  mode: NextMoveReviewMarkerMode;
  selectedNode: SgfTreeNodeDto | null;
  selectedIsRoot: boolean;
  toPlay: PlayerColor;
  boardWidth: number;
  boardHeight: number;
}): NextMoveReviewMarker[] {
  if (input.mode === "off" || !input.selectedNode) return [];
  const parentAnalysis = input.mode === "graded"
    ? displayedPrimaryAnalysis(input.selectedNode, input.selectedIsRoot)
    : null;
  const markers: NextMoveReviewMarker[] = [];
  for (const [index, child] of input.selectedNode.children.entries()) {
    const move = child.properties.find((property) => property.key === "B" || property.key === "W");
    if (!move) continue;
    const raw = move.values[0] ?? "";
    if (raw.length === 0 || (raw.toLowerCase() === "tt" && input.boardWidth <= 19 && input.boardHeight <= 19)) continue;
    if (raw.length !== 2) continue;
    const point = { x: raw.charCodeAt(0) - 97, y: raw.charCodeAt(1) - 97 };
    if (point.x < 0 || point.y < 0 || point.x >= input.boardWidth || point.y >= input.boardHeight) continue;
    const primary = index === 0;
    const rank = input.mode === "graded" && primary
      ? classifyPlayedMove(parentAnalysis, displayedPrimaryAnalysis(child, false), input.toPlay)
      : null;
    markers.push({ point, primary, rank });
  }
  return markers;
}
