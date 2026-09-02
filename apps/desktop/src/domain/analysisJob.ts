import type { AnalysisJobEventDto, AnalysisPublicationScopeDto } from "./types";

export function admitsAnalysisPublication(
  event: AnalysisJobEventDto,
  current: AnalysisPublicationScopeDto
): boolean {
  return event.outcome === "completed"
    && event.frame != null
    && event.run_id === current.run_id
    && event.job_id === current.job_id
    && event.generation === current.generation
    && event.node_path.indices.length === current.node_path.indices.length
    && event.node_path.indices.every((index, offset) => index === current.node_path.indices[offset]);
}
