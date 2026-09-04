import { isPoint } from "./board";
import type { CandidateMoveDto, MoveVertex, NodePath, PointDto } from "./types";

export type VariationReplayIdentity = {
  path: number[];
  candidate: string;
  pv: string[];
};

export function variationReplayPointSteps(candidate: CandidateMoveDto | null | undefined): PointDto[] {
  if (!candidate) return [];
  const steps: PointDto[] = [];
  if (isPoint(candidate.vertex)) steps.push(candidate.vertex.point);
  for (const vertex of candidate.pv) {
    if (isPoint(vertex)) steps.push(vertex.point);
  }
  return steps;
}

export function variationReplayIdentity(
  path: NodePath | undefined,
  candidate: CandidateMoveDto | null | undefined
): VariationReplayIdentity | null {
  if (!path || !candidate) return null;
  return {
    path: [...path.indices],
    candidate: vertexKey(candidate.vertex),
    pv: candidate.pv.map(vertexKey)
  };
}

export function variationReplayIdentityKey(identity: VariationReplayIdentity | null): string {
  return identity == null ? "" : JSON.stringify(identity);
}

function vertexKey(vertex: MoveVertex): string {
  return isPoint(vertex) ? `${vertex.point.x}:${vertex.point.y}` : "pass";
}
