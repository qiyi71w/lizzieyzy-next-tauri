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
  boardSize: number;
}): NextMoveReviewMarker[] {
  if (input.mode === "off" || !input.selectedNode) return [];
  const parentAnalysis = input.mode === "graded"
    ? displayedPrimaryAnalysis(input.selectedNode, input.selectedIsRoot)
    : null;
  const markers: NextMoveReviewMarker[] = [];
  for (const [index, child] of input.selectedNode.children.entries()) {
    const point = childCoordinate(child, input.boardSize);
    if (!point) continue;
    const primary = index === 0;
    const rank = input.mode === "graded" && primary
      ? classifyPlayedMove(parentAnalysis, displayedPrimaryAnalysis(child, false), input.toPlay)
      : null;
    markers.push({ point, primary, rank });
  }
  return markers;
}

function childCoordinate(node: SgfTreeNodeDto, boardSize: number): PointDto | null {
  const move = node.properties.find((property) => property.key === "B" || property.key === "W");
  if (!move) return null;
  return parseSgfPoint(move.values[0] ?? "", boardSize);
}

function parseSgfPoint(raw: string, boardSize: number): PointDto | null {
  if (raw.length === 0 || (raw.toLowerCase() === "tt" && boardSize <= 19)) return null;
  if (raw.length !== 2) return null;
  const x = raw.charCodeAt(0) - 97;
  const y = raw.charCodeAt(1) - 97;
  if (x < 0 || y < 0 || x >= boardSize || y >= boardSize) return null;
  return { x, y };
}
