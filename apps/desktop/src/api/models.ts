import { invoke } from "@tauri-apps/api/core";
import type { ModelInventoryDto, ModelPathDto, ModelSelectionRequestDto } from "../domain/types";
import { isTauriRuntime } from "./backend";

export async function loadModelInventory(): Promise<ModelInventoryDto> {
  if (!isTauriRuntime()) return { revision: "browser-unavailable", models: [] };
  return invoke<ModelInventoryDto>("model_inventory_snapshot");
}

export async function refreshModelInventory(paths: ModelPathDto[]): Promise<ModelInventoryDto> {
  if (!isTauriRuntime()) throw new Error("model_native_required");
  return invoke<ModelInventoryDto>("refresh_model_inventory", { paths });
}

export async function selectInstalledModel(request: ModelSelectionRequestDto): Promise<string> {
  if (!isTauriRuntime()) throw new Error("model_native_required");
  return invoke<string>("select_installed_model", { request });
}
