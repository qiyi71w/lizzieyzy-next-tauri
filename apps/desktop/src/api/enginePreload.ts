import { invoke } from "@tauri-apps/api/core";
import type { EnginePreloadDto } from "../domain/types";
import { isTauriRuntime } from "./backend";

export async function enginePreloadSnapshot(): Promise<EnginePreloadDto[]> {
  return isTauriRuntime() ? invoke<EnginePreloadDto[]>("engine_preload_snapshot") : [];
}

export async function prepareEnginePreload(profileId: string): Promise<void> {
  if (!isTauriRuntime()) throw new Error("Background engines require the Tauri desktop runtime.");
  await invoke("engine_preload_prepare", { profileId });
}

export async function cancelEnginePreload(profileId: string): Promise<void> {
  if (!isTauriRuntime()) throw new Error("Background engines require the Tauri desktop runtime.");
  await invoke("engine_preload_cancel", { profileId });
}
