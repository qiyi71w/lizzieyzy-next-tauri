import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { BatchCommandDto, BatchSnapshotDto } from "../domain/types";
import { confirm } from "@tauri-apps/plugin-dialog";
import { t } from "../i18n/resources";
export const batchNativeAvailable = () => typeof window !== "undefined" && window.__TAURI_INTERNALS__ !== undefined;
function native(): void { if (!batchNativeAvailable()) throw new Error("Batch file analysis and output require the native Tauri backend. Browser preview edits preferences only."); }
export async function batchSnapshot(): Promise<BatchSnapshotDto> { native(); return invoke("batch_snapshot"); }
export async function batchCommand(command: BatchCommandDto): Promise<BatchSnapshotDto> { native(); return invoke("batch_command", { command }); }
export async function pickBatchFiles(): Promise<string[]> { native(); return invoke("pick_batch_files"); }
export async function pickBatchDirectory(): Promise<string | null> { native(); return invoke("pick_batch_directory"); }
export async function pickBatchSave(name: string): Promise<string | null> { native(); return invoke("pick_batch_save", { name }); }
export async function listenBatchIntake(handler: (paths: string[]) => void): Promise<() => void> {
  if (!batchNativeAvailable()) return () => {};
  return listen<string[]>("batch://intake", event => handler(event.payload));
}

export async function confirmBatchOverwrite(path: string): Promise<boolean> {
  native();
  return confirm(`${t("batch.overwrite")}\n${path}`, { title: t("batch.title"), kind: "warning" });
}

export async function confirmBatchExit(): Promise<boolean> {
  if (!batchNativeAvailable()) return true;
  const snapshot = await batchSnapshot();
  const loss = snapshot.rows.some(row => ["pending", "waiting", "running"].includes(row.state) || row.has_result && row.output_state !== "saved");
  return !loss || await confirm(t("batch.exitWarning"), { title: t("batch.title"), kind: "warning" });
}
