import { invoke } from "@tauri-apps/api/core";
import { isTauriRuntime } from "./backend";
import type { LocalFocusRequestDto, LocalFocusSnapshotDto } from "../domain/types";

export async function localFocusAction(request: LocalFocusRequestDto): Promise<LocalFocusSnapshotDto> {
  if (!isTauriRuntime()) throw new Error("Local focus requires the native qualified ordinary analysis owner.");
  return await invoke<LocalFocusSnapshotDto>("local_focus_action", { request });
}
