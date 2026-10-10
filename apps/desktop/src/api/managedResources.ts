import { invoke } from "@tauri-apps/api/core";
import { isTauriRuntime } from "./backend";
import type { EngineProfileDto, ManagedAcquireRequestDto, ManagedRepairPreviewDto, ManagedResourcesDto } from "../domain/types";

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
export async function inspectManagedTrtRepair(request: ManagedAcquireRequestDto): Promise<ManagedRepairPreviewDto> {
  if (!isTauriRuntime()) throw new Error("managed_native_required");
  return invoke<ManagedRepairPreviewDto>("inspect_managed_trt_repair", { request });
}
export async function repairManagedTrt(admissionId: string): Promise<string> {
  if (!isTauriRuntime()) throw new Error("managed_native_required");
  return invoke<string>("repair_managed_trt", { admissionId });
}
export async function managedRepairDraft(operationId: string): Promise<EngineProfileDto> {
  if (!isTauriRuntime()) throw new Error("managed_native_required");
  return invoke<EngineProfileDto>("managed_repair_draft", { operationId });
}
