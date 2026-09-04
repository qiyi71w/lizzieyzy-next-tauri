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
  let pvOffset = 0;
  if (isPoint(candidate.vertex)) {
    const candidatePoint = candidate.vertex.point;
    steps.push(candidatePoint);
    const firstPvVertex = candidate.pv[0];
    if (
      firstPvVertex
      && isPoint(firstPvVertex)
      && firstPvVertex.point.x === candidatePoint.x
      && firstPvVertex.point.y === candidatePoint.y
    ) {
      pvOffset = 1;
    }
  }
  for (let index = pvOffset; index < candidate.pv.length; index += 1) {
    const vertex = candidate.pv[index];
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
