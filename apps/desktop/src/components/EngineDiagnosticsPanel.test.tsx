// @vitest-environment jsdom
import { act } from "react";
import { createRoot } from "react-dom/client";
import type { Root } from "react-dom/client";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import * as backend from "../api/backend";
import * as exportApi from "../api/diagnosticExport";
import type { EngineDiagnosticSnapshotDto, ForegroundEngineSnapshotDto } from "../domain/types";
import { EngineDiagnosticsPanel } from "./EngineDiagnosticsPanel";
import { t } from "../i18n/resources";

let host: HTMLDivElement;
let root: Root;
const snapshot: ForegroundEngineSnapshotDto = { revision: 1, lifecycle: { state: "no_engine" }, continuous: { enabled: false, phase: "off" } };
const attempt = (id: string): EngineDiagnosticSnapshotDto => ({
  attempt_id: id, run_id: id, profile_id: "default", captured_at_ms: 1234, full_trace: false,
  command: "program=[private-1] argv=[]", failure: null, stdout_complete: false, stderr_complete: false, process_exited: false, exit_code: null,
  retained_bytes: 64000, dropped_records: 1000,
  records: Array.from({ length: 256 }, (_, sequence) => ({ sequence, at_ms: 1234, source: "stderr", text: `WARN ${sequence} ${"x".repeat(200)}` })),
  metrics: [{ role: "application-process", name: "resident-memory", unit: "bytes", at_ms: 1234, value: 4096, missing: null }],
});
beforeEach(() => {
  globalThis.IS_REACT_ACT_ENVIRONMENT = true;
  vi.useFakeTimers();
  host = document.createElement("div"); document.body.append(host); root = createRoot(host);
});
afterEach(async () => { await act(async () => root.unmount()); host.remove(); vi.restoreAllMocks(); vi.useRealTimers(); });
function button(label: string) {
  const element = Array.from(host.querySelectorAll("button")).find((item) => item.textContent === label);
  if (!element) throw new Error(`Missing ${label}`);
  return element;
}
it("keeps editable focus during admitted bursts and copies the pinned attempt after a newer response", async () => {
  const old = attempt("old-run");
  const next = attempt("new-run");
  const collect = vi.spyOn(backend, "getEngineDiagnostics").mockResolvedValue([old]);
  const clipboard = vi.fn().mockResolvedValue(undefined);
  Object.defineProperty(navigator, "clipboard", { configurable: true, value: { writeText: clipboard } });
  await act(async () => root.render(<EngineDiagnosticsPanel engineSnapshot={snapshot} />));
  const input = host.querySelector<HTMLInputElement>('input:not([type="checkbox"])');
  if (!input) throw new Error("Missing diagnostic filter");
  input.focus();
  await act(async () => {
    Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")!.set!.call(input, "WARN 25");
    input.dispatchEvent(new Event("input", { bubbles: true }));
    await vi.advanceTimersByTimeAsync(500);
  });
  expect(document.activeElement).toBe(input);
  expect(input.value).toBe("WARN 25");
  expect(host.querySelector("pre[aria-label]")?.textContent).toContain("WARN 255");
  expect(host.querySelector("pre[aria-label]")?.textContent).not.toContain("WARN 24 ");
  await act(async () => button(t("diagnostics.freeze")).click());
  collect.mockResolvedValue([next]);
  await act(async () => vi.advanceTimersByTimeAsync(1500));
  await act(async () => button(t("diagnostics.copy")).click());
  expect(JSON.parse(clipboard.mock.calls[0][0]).attempt_id).toBe("old-run");
  expect(button(t("diagnostics.stop")).disabled).toBe(true);
  await act(async () => button(t("diagnostics.resume")).click());
  expect(host.querySelector("select")?.value).toBe("new-run");
});

it("exports the displayed capture after Retry and keeps export success when opening its folder fails", async () => {
  vi.spyOn(backend, "isTauriRuntime").mockReturnValue(true);
  const old = attempt("failed-a");
  const collect = vi.spyOn(backend, "getEngineDiagnostics").mockResolvedValue([old]);
  const estimate = vi.spyOn(exportApi, "estimateDiagnosticExport").mockResolvedValue(7);
  const status = { generation: 7, attempt_id: "failed-a", captured_at_ms: 1234, phase: "ready" as const, source_bytes: 123456, entries: 4, completed_entries: 0, failed_stage: null, message: null, file_name: null, cleanup_pending: false };
  const poll = vi.spyOn(exportApi, "diagnosticExportStatus").mockResolvedValue({ ...status, phase: "idle" });
  const start = vi.spyOn(exportApi, "startDiagnosticExport").mockResolvedValue(undefined);
  vi.spyOn(exportApi, "openDiagnosticExportFolder").mockResolvedValue("failed");
  await act(async () => root.render(<EngineDiagnosticsPanel engineSnapshot={snapshot} />));
  poll.mockResolvedValue(status);
  await act(async () => button(t("diagnostics.export.estimate")).click());
  expect(estimate).toHaveBeenCalledWith(old);
  collect.mockResolvedValue([attempt("retry-b")]);
  await act(async () => button(t("diagnostics.resume")).click());
  await act(async () => vi.advanceTimersByTimeAsync(500));
  expect(host.querySelector("select")?.value).toBe("retry-b");
  await act(async () => button(t("diagnostics.export.start")).click());
  expect(start).toHaveBeenCalledWith(7);
  poll.mockResolvedValue({ ...status, phase: "completed", completed_entries: 4, file_name: "diagnostics-test.zip" });
  await act(async () => vi.advanceTimersByTimeAsync(150));
  await act(async () => button(t("diagnostics.export.folder")).click());
  expect(host.textContent).toContain(t("diagnostics.export.phase.completed"));
  expect(host.textContent).toContain(t("diagnostics.export.folderFailed"));
  expect(host.textContent).toContain("failed-a");
  expect(host.textContent).toContain("diagnostics-test.zip");
  poll.mockResolvedValue({ ...status, generation: 8, attempt_id: "replacement-c" });
  await act(async () => button(t("diagnostics.export.folder")).click());
  expect(host.textContent).toContain(t("diagnostics.export.phase.completed"));
  expect(host.textContent).toContain("diagnostics-test.zip");
  expect(host.textContent).not.toContain("replacement-c");
});

it("does not re-enable an older estimate after the displayed replacement is rejected", async () => {
  vi.spyOn(backend, "isTauriRuntime").mockReturnValue(true);
  vi.spyOn(backend, "getEngineDiagnostics").mockResolvedValue([attempt("new-displayed")]);
  vi.spyOn(exportApi, "estimateDiagnosticExport").mockRejectedValue(new Error("source budget"));
  vi.spyOn(exportApi, "diagnosticExportStatus").mockResolvedValue({ generation: 2, attempt_id: "old-target", captured_at_ms: 1, phase: "ready", source_bytes: 12, entries: 4, completed_entries: 0, failed_stage: null, message: null, file_name: null, cleanup_pending: false });
  await act(async () => root.render(<EngineDiagnosticsPanel engineSnapshot={snapshot} />));
  await act(async () => button(t("diagnostics.export.estimate")).click());
  await act(async () => vi.advanceTimersByTimeAsync(300));
  expect(button(t("diagnostics.export.start")).disabled).toBe(true);
  expect(host.textContent).toContain(t("diagnostics.export.failed"));
});
