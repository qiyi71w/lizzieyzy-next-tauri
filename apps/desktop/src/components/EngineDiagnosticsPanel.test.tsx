// @vitest-environment jsdom
import { act } from "react";
import { createRoot } from "react-dom/client";
import type { Root } from "react-dom/client";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import * as backend from "../api/backend";
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
