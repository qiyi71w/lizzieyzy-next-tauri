import { Channel, invoke } from "@tauri-apps/api/core";
import { isTauriRuntime } from "./backend";
import type { AnalysisScopeDto, AnalysisScopePreviewDto, AnalysisTaskDto, ApplicationMeasurementRequestDto, ApplicationMeasurementProbeDto, ApplicationMeasurementProgressDto, EngineProfilesSettingsDto, MeasuredReviewDto } from "../domain/types";
import type { ApplicationMeasurementDocumentDto, ApplicationMeasurementResultDto } from "../domain/types";
function native() { if (!isTauriRuntime()) throw new Error("Measured reports require the desktop runtime and actual local resources."); }
export function reviewMeasuredReport(targetId: string, report: string): Promise<MeasuredReviewDto> { native(); return invoke("measured_report_review", { targetId, report }); }
export function applyMeasuredReport(review: MeasuredReviewDto): Promise<EngineProfilesSettingsDto> { native(); return invoke("measured_report_apply", { operationId: review.operation_id, generation: review.generation }); }
export function restoreMeasuredReport(targetId: string, revision: number): Promise<EngineProfilesSettingsDto> { native(); return invoke("measured_report_restore", { targetId, revision }); }
export function retireMeasuredReport(): Promise<void> { return isTauriRuntime() ? invoke("measured_report_retire") : Promise.resolve(); }
export function measuredReportBusy(): Promise<boolean> { return isTauriRuntime() ? invoke("measured_report_busy") : Promise.resolve(false); }
export function previewMeasuredScene(generation: number, scope: AnalysisScopeDto): Promise<AnalysisScopePreviewDto> { native(); return invoke("measured_scene_preview", { generation, scope }); }
export function startMeasuredScene(targetId: string, preview: AnalysisScopePreviewDto, budget: number): Promise<AnalysisTaskDto> { native(); return invoke("measured_scene_start", { targetId, preview, budget }); }
export function measureApplication(request: ApplicationMeasurementRequestDto, onDocument: (document: ApplicationMeasurementDocumentDto) => void): Promise<ApplicationMeasurementResultDto> {
  native();
  const documents = new Channel<ApplicationMeasurementDocumentDto>();
  let operationId: string | null = null;
  documents.onmessage = (document) => {
    operationId ??= document.operation_id;
    if (document.operation_id === operationId && document.row_id && document.run_id) onDocument(document);
  };
  return invoke("measure_application", { request, documents });
}
export function measurementProgress(): Promise<ApplicationMeasurementProgressDto | null> { native(); return invoke("measurement_progress"); }
export function submitMeasurementProbe(probe: ApplicationMeasurementProbeDto): Promise<void> { native(); return invoke("measurement_probe", { probe }); }
export async function waitForMeasuredWorker(): Promise<void> {
  while (await measuredReportBusy()) {
    await new Promise<void>((resolve) => window.setTimeout(resolve, 50));
  }
}
