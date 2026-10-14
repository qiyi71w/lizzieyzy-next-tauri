import { invoke } from "@tauri-apps/api/core";
import { isTauriRuntime } from "./backend";
import type { RawNetRequestDto, RawNetSnapshotDto } from "../domain/types";

export async function computeRawNet(request: RawNetRequestDto): Promise<RawNetSnapshotDto> {
  if (!isTauriRuntime()) throw new Error("Raw-net requires the native runtime and a qualified local engine.");
  return invoke<RawNetSnapshotDto>("foreground_engine_compute_raw_net", { request });
}
