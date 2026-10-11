import { invoke } from "@tauri-apps/api/core";
import { isTauriRuntime } from "./backend";
import type { EngineProfilesSettingsDto, ThreadPolicySaveRequestDto, ThreadPolicyStatusDto } from "../domain/types";

export async function getThreadPolicyStatus(targetId: string, evaluationId: string | null): Promise<ThreadPolicyStatusDto> {
  if (!isTauriRuntime()) throw new Error("policy_native_only");
  return invoke("engine_thread_policy_status", { targetId, evaluationId });
}

export async function saveThreadPolicy(request: ThreadPolicySaveRequestDto): Promise<EngineProfilesSettingsDto> {
  if (!isTauriRuntime()) throw new Error("policy_native_only");
  return invoke("save_engine_thread_policy", { request });
}
