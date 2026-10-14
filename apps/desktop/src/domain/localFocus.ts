import type { AnalysisFrameDto, LocalFocusPointDto, LocalFocusPreferencesDto, MoveRankPreferencesDto, MoveVertex, PlayerColor } from "./types";
import { classifyPlayedMove } from "./moveRank";

export const defaultLocalFocusPreferences: LocalFocusPreferencesDto = { maxVisits: 500, showOutline: true, outlineOpacity: 92 };
export function validLocalFocusPreferences(value: LocalFocusPreferencesDto): boolean {
  return Number.isInteger(value.maxVisits) && value.maxVisits > 0 && value.maxVisits <= 4294967295
    && typeof value.showOutline === "boolean" && Number.isInteger(value.outlineOpacity) && value.outlineOpacity >= 0 && value.outlineOpacity <= 100;
}
export function sameFocusVertex(a: MoveVertex, b: MoveVertex): boolean {
  return a === "pass" || b === "pass" ? a === b : a.point.x === b.point.x && a.point.y === b.point.y;
}
export function focusQuality(frame: AnalysisFrameDto | undefined, vertex: MoveVertex, toPlay: PlayerColor, grading: MoveRankPreferencesDto) {
  const baseline = frame?.candidates.find((candidate) => candidate.order === 0);
  const candidate = frame?.candidates.find((candidate) => sameFocusVertex(candidate.vertex, vertex));
  return classifyPlayedMove(
    baseline ? { visits: baseline.visits, winrateBlack: baseline.winrate_black, scoreMeanBlack: baseline.score_mean_black ?? undefined } : null,
    candidate ? { visits: candidate.visits, winrateBlack: candidate.winrate_black, scoreMeanBlack: candidate.score_mean_black ?? undefined } : null,
    toPlay, grading);
}
export function focusFilteredCandidates(frame: AnalysisFrameDto, limit: number, watched: readonly LocalFocusPointDto[]) {
  return frame.candidates.filter((candidate, index) => index < limit || watched.some((point) => sameFocusVertex(point.vertex, candidate.vertex)));
}
