import { invoke } from "@tauri-apps/api/core";
import { isTauriRuntime } from "./backend";
import type { ManagedAcquireRequestDto, ManagedResourcesDto } from "../domain/types";

export async function loadManagedResources(): Promise<ManagedResourcesDto | null> {
  if (!isTauriRuntime()) return null;
  return invoke<ManagedResourcesDto>("managed_resources_snapshot");
}
export async function acquireManagedResources(request: ManagedAcquireRequestDto): Promise<string> {
  if (!isTauriRuntime()) throw new Error("managed_native_required");
  return invoke<string>("acquire_managed_resources", { request });
}
export async function cancelManagedResources(operationId: string): Promise<void> {
  if (!isTauriRuntime()) throw new Error("managed_native_required");
  return invoke<void>("cancel_managed_resources", { operationId });
}
