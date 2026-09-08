import type { AnalysisJobEventDto, AnalysisJobStartedDto, AnalysisPublicationScopeDto } from "./types";

export function admitsAnalysisPublication(
  event: AnalysisJobEventDto,
  current: AnalysisPublicationScopeDto
): boolean {
  return (event.outcome === "completed" || (event.mode === "continuous" && (event.outcome === "progress" || event.outcome === "time_limited")))
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
    && pending.mode === job.mode
    && pending.generation === job.generation;
}

export function admitsWholeGameNodeResult(job: AnalysisJobEventDto): boolean {
  return job.lane === "whole_game"
    && job.outcome === "progress"
    && job.frame != null;
}

export function admitsAnalysisAttachment(event: AnalysisJobEventDto): boolean {
  const frame = event.frame;
  if (frame == null || frame.visits === 0 || frame.candidates.length === 0) return false;
  if (event.lane === "selected_node") {
    return event.mode === "continuous"
      ? event.outcome === "progress" || event.outcome === "completed" || event.outcome === "time_limited"
      : event.outcome === "completed";
  }
  return event.lane === "whole_game" && event.outcome === "progress";
}
