import { invoke } from "@tauri-apps/api/core";
import { isTauriRuntime } from "./backend";
import type { DiagnosticExportStatusDto, DiagnosticFolderOutcomeDto, EngineDiagnosticSnapshotDto } from "../domain/types";

function nativeOnly() {
  if (!isTauriRuntime()) throw new Error("Diagnostic filesystem export requires the native desktop backend");
}
export function estimateDiagnosticExport(snapshot: EngineDiagnosticSnapshotDto): Promise<number> {
  nativeOnly();
  return invoke("estimate_diagnostic_export", { snapshot });
}
export function diagnosticExportStatus(): Promise<DiagnosticExportStatusDto> {
  nativeOnly();
  return invoke("diagnostic_export_status");
}
export function startDiagnosticExport(generation: number): Promise<void> {
  nativeOnly();
  return invoke("start_diagnostic_export", { generation });
}
export function cancelDiagnosticExport(generation: number): Promise<void> {
  nativeOnly();
  return invoke("cancel_diagnostic_export", { generation });
}
export function openDiagnosticExportFolder(generation: number): Promise<DiagnosticFolderOutcomeDto> {
  nativeOnly();
  return invoke("open_diagnostic_export_folder", { generation });
}
