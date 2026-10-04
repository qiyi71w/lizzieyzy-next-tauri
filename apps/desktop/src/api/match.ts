import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { isTauriRuntime } from "./backend";
import type { HumanMatchActionDto, HumanMatchStartDto, MatchAnalysisPolicyDto, MatchTurnDto, MatchUpdateDto } from "../domain/types";

export const nativeMatchUnavailable = "人机与 PK 对局仅在桌面运行时可用。";
function requireNative() {
  if (!isTauriRuntime()) throw new Error(nativeMatchUnavailable);
}
export async function humanMatchSnapshot(): Promise<MatchUpdateDto> {
  requireNative();
  return invoke("human_match_snapshot");
}
export async function humanMatchStart(request: HumanMatchStartDto): Promise<MatchUpdateDto> {
  requireNative();
  return invoke("human_match_start", { request });
}
export async function pkMatchStart(request: HumanMatchStartDto): Promise<MatchUpdateDto> {
  requireNative();
  return invoke("pk_match_start", { request });
}
export async function pkMatchPause(sessionId: string): Promise<MatchUpdateDto> {
  requireNative();
  return invoke("pk_match_pause", { sessionId });
}
export async function pkMatchResume(sessionId: string): Promise<MatchUpdateDto> {
  requireNative();
  return invoke("pk_match_resume", { sessionId });
}
export async function humanMatchAction(turn: MatchTurnDto, action: HumanMatchActionDto): Promise<MatchUpdateDto> {
  requireNative();
  return invoke("human_match_action", { turn, action });
}
export async function humanMatchAnalysisPolicy(turn: MatchTurnDto, policy: MatchAnalysisPolicyDto): Promise<MatchUpdateDto> {
  requireNative();
  return invoke("human_match_analysis_policy", { turn, policy });
}
export async function humanMatchStop(sessionId: string): Promise<MatchUpdateDto> {
  requireNative();
  return invoke("human_match_stop", { sessionId });
}
export async function subscribeHumanMatch(onUpdate: (update: MatchUpdateDto) => void): Promise<() => void> {
  requireNative();
  return listen<MatchUpdateDto>("human-match-updated", (event) => onUpdate(event.payload));
}
