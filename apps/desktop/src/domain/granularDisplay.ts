import type { CandidateMoveDto, PlayerColor } from "./types";
import { isPoint } from "./board";
import { t } from "../i18n/resources";

export type OwnershipStyle = "small" | "normal" | "notOnLive" | "bigBelow" | "bySize";
export type CandidateTextField = "winrate" | "visits" | "score";
export type GranularDisplay = {
  delayMode: "off" | "timed" | "manual";
  delaySeconds: number;
  showBlackCandidates: boolean;
  showWhiteCandidates: boolean;
  pvVisitsMode: "off" | "last" | "all";
  pvVisitsThreshold: number;
  pvLength: number;
  showWinrate: boolean;
  showVisits: boolean;
  showScore: boolean;
  scoreMode: "lead" | "board";
  showOrder: boolean;
  emphasizeMax: boolean;
  textOrder: [CandidateTextField, CandidateTextField, CandidateTextField];
  ownershipStyle: OwnershipStyle;
  useCandidateOwnership: boolean;
  rawNetOwnershipStyle: OwnershipStyle;
};
export const defaultGranularDisplay: GranularDisplay = {
  delayMode: "off", delaySeconds: 10, showBlackCandidates: true, showWhiteCandidates: true,
  pvVisitsMode: "off", pvVisitsThreshold: 0, pvLength: 15,
  showWinrate: true, showVisits: true, showScore: true, scoreMode: "lead",
  showOrder: true, emphasizeMax: false, textOrder: ["winrate", "visits", "score"],
  ownershipStyle: "small", useCandidateOwnership: true, rawNetOwnershipStyle: "bigBelow"
};
export function validGranularDisplay(value: GranularDisplay): boolean {
  const ms = Math.round(value.delaySeconds * 1000);
  const styles = ["small", "normal", "notOnLive", "bigBelow", "bySize"];
  return ["off", "timed", "manual"].includes(value.delayMode)
    && typeof value.delaySeconds === "number" && Number.isInteger(ms) && ms / 1000 === value.delaySeconds && ms >= 1 && ms <= 2147483647
    && Number.isInteger(value.pvVisitsThreshold) && value.pvVisitsThreshold >= 0 && value.pvVisitsThreshold <= 2147483647
    && Number.isInteger(value.pvLength) && value.pvLength >= -2147483648 && value.pvLength <= 2147483647
    && ["off", "last", "all"].includes(value.pvVisitsMode) && ["lead", "board"].includes(value.scoreMode)
    && styles.includes(value.ownershipStyle) && styles.includes(value.rawNetOwnershipStyle)
    && Array.isArray(value.textOrder) && value.textOrder.length === 3 && new Set(value.textOrder).size === 3
    && value.textOrder.every(field => ["winrate", "visits", "score"].includes(field))
    && [value.showBlackCandidates, value.showWhiteCandidates, value.showWinrate, value.showVisits, value.showScore,
      value.showOrder, value.emphasizeMax, value.useCandidateOwnership].every(flag => typeof flag === "boolean");
}
export function readGranularDisplay(value: Partial<GranularDisplay> | undefined): GranularDisplay {
  if (value !== undefined && (value === null || typeof value !== "object" || Array.isArray(value))) throw new Error(t("display.invalid"));
  const result = { ...defaultGranularDisplay, ...value };
  if (!validGranularDisplay(result)) throw new Error(t("display.invalid"));
  return result;
}
export function displayVariationSteps(candidate: CandidateMoveDto | null | undefined, preferences: GranularDisplay, replay?: number) {
  if (!candidate) return [];
  const first = candidate.pv[0];
  const startsWithCandidate = JSON.stringify(first) === JSON.stringify(candidate.vertex);
  const vertices = startsWithCandidate ? candidate.pv : [candidate.vertex, ...candidate.pv];
  const prefix = vertices.slice(0, preferences.pvLength > 0 ? preferences.pvLength : Infinity);
  const steps = prefix.flatMap((vertex, offset) => isPoint(vertex) ? [{ point: vertex.point, offset, pvIndex: startsWithCandidate ? offset : offset - 1 }] : []);
  const visible = steps.slice(0, replay ?? Infinity);
  const lastOffset = visible.length < steps.length ? visible.at(-1)?.offset ?? -1 : prefix.length - 1;
  return visible.map(step => ({ ...step, lastPvIndex: startsWithCandidate ? lastOffset : lastOffset - 1 }));
}
export function pvVisitsLabel(candidate: CandidateMoveDto, index: number, visibleLength: number, preferences: GranularDisplay): string | null {
  const values = candidate.pv_visits;
  if (preferences.pvVisitsMode === "off" || !values || values.length !== candidate.pv.length
    || (preferences.pvVisitsMode === "last" && index !== visibleLength - 1)) return null;
  const visits = values[index];
  return Number.isInteger(visits) && visits > preferences.pvVisitsThreshold ? String(visits) : null;
}
export function candidateDisplayScore(candidate: CandidateMoveDto, preferences: GranularDisplay, perspective: "black" | "sideToPlay", toPlay: PlayerColor, komi?: number): number | null {
  const score = candidate.score_mean_black;
  if (score == null || !Number.isFinite(score) || (preferences.scoreMode === "board" && (komi == null || !Number.isFinite(komi)))) return null;
  const black = score + (preferences.scoreMode === "board" ? komi! : 0);
  return perspective === "sideToPlay" && toPlay === "white" ? -black : black;
}
