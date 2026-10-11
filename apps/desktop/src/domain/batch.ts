import type { AnalysisStageConditionsDto, BatchPreferencesDto } from "./types";

const visits = (value: number): AnalysisStageConditionsDto => ({
  time_seconds: { enabled: false, value: 10 }, total_visits: { enabled: true, value },
  leading_candidate_visits: { enabled: false, value: 500 }
});
export function defaultBatchPreferences(): BatchPreferencesDto {
  const swingDeep = visits(800);
  swingDeep.time_seconds.enabled = true;
  swingDeep.total_visits.enabled = false;
  return { strategy: "single_stage", scope: { mode: "first_child_mainline", current_node: { indices: [] }, branch_choices: [], interval: null, to_play: null },
    normal: visits(100), overview: visits(32), deep: visits(800), swingOverview: visits(32), swingDeep,
    swingCriteria: { move_actors: "both", winrate_change_percentage_points: { enabled: true, value: 10 }, score_change_points: { enabled: false, value: 3 } } };
}
export function validateBatchPreferences(value: BatchPreferencesDto): void {
  for (const stage of [value.normal, value.overview, value.deep, value.swingOverview, value.swingDeep]) {
    const limits = [stage.time_seconds, stage.total_visits, stage.leading_candidate_visits];
    if (limits.some(v => !Number.isInteger(v.value) || v.value < 1 || v.value > 4294967295) || !limits.some(v => v.enabled)) throw new Error("Task limits must be positive integers, including disabled values; enable at least one OR condition.");
  }
  if (value.deep.total_visits.value < 500) throw new Error("All-position deep total visits must be at least 500.");
  const thresholds = [value.swingCriteria.winrate_change_percentage_points, value.swingCriteria.score_change_points];
  if (thresholds.some(v => !Number.isFinite(v.value) || v.value <= 0) || !thresholds.some(v => v.enabled)) throw new Error("Swing needs positive retained thresholds and at least one enabled threshold.");
  const interval = value.scope.interval;
  if (interval && (!Number.isInteger(interval.start) || !Number.isInteger(interval.end) || interval.start < 0 || interval.start > interval.end || interval.end > 4294967295)) throw new Error("Invalid closed move interval.");
}
