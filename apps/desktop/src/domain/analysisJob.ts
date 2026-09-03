import type { AnalysisJobEventDto, AnalysisJobStartedDto, AnalysisPublicationScopeDto } from "./types";

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

export function matchesWholeGameJobIdentity(
  pending: AnalysisJobStartedDto | null,
  job: AnalysisJobEventDto
): pending is AnalysisJobStartedDto {
  return pending != null
    && pending.run_id === job.run_id
    && pending.job_id === job.job_id
    && pending.lane === "whole_game"
    && job.lane === "whole_game"
    && pending.generation === job.generation;
}

export function admitsWholeGameNodeResult(job: AnalysisJobEventDto): boolean {
  return job.lane === "whole_game"
    && job.outcome === "progress"
    && job.frame != null;
}
