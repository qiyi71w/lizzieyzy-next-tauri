// @vitest-environment jsdom
import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import * as backend from "../api/backend";
import type { EngineProfileRecordDto, EvaluationSnapshotDto } from "../domain/types";
import { EngineEvaluationPanel } from "./EngineEvaluationPanel";
import { t } from "../i18n/resources";

const idle: EvaluationSnapshotDto = { evaluation_id: null, target_id: null, input_revision: null, phase: "idle", process_id: null, exit_code: null, output: [], output_truncated: false, message: null, result: null };
const running: EvaluationSnapshotDto = { ...idle, evaluation_id: "one", target_id: "saved", input_revision: "revision", phase: "running", process_id: 123 };
const profiles: EngineProfileRecordDto[] = [{ id: "saved", profile: { name: "Saved local", program: "katago", argv: [], working_dir: null, adapter_kind: "generic_gtp", settings: {} } }];
let host: HTMLDivElement;
let root: Root | null;
beforeEach(() => {
  globalThis.IS_REACT_ACT_ENVIRONMENT = true;
  vi.useFakeTimers();
  host = document.createElement("div"); document.body.append(host);
  root = createRoot(host);
  vi.spyOn(backend, "isTauriRuntime").mockReturnValue(true);
  vi.spyOn(backend, "getEngineEvaluation").mockResolvedValue(idle);
  vi.spyOn(backend, "startEngineEvaluation").mockResolvedValue(running);
  vi.spyOn(backend, "cancelEngineEvaluation").mockResolvedValue({ ...running, phase: "cancelled", process_id: null });
});
afterEach(async () => {
  await act(async () => root?.unmount()); root = null;
  host.remove(); vi.restoreAllMocks(); vi.useRealTimers();
});
async function render() { await act(async () => root!.render(<EngineEvaluationPanel profiles={profiles} disabled={false} />)); }
async function select() {
  await act(async () => { const select = host.querySelector("select")!; select.value = "saved"; select.dispatchEvent(new Event("change", { bubbles: true })); });
}
async function click(text: string) {
  await act(async () => { Array.from(host.querySelectorAll("button")).find(button => button.textContent === text)!.click(); });
}

it("requires explicit saved target, renders real result and cancels only its evaluation", async () => {
  await render();
  expect(host.textContent).toContain(t("evaluation.idle"));
  expect(host.querySelector("button")!.disabled).toBe(true);
  await select(); await click(t("evaluation.start"));
  expect(backend.startEngineEvaluation).toHaveBeenCalledWith("saved");
  expect(host.textContent).toContain(t("evaluation.running"));
  vi.mocked(backend.getEngineEvaluation).mockResolvedValue({ ...running, phase: "completed", process_id: null, exit_code: 0,
    result: { target_id: "saved", input_revision: "revision", elapsed_ms: 1000, search_visits_per_second: null,
      qualified_resource: { profile_revision: "revision", resources: [], origin: "local_unknown", version: "1.18.2", backend: "Eigen", source_commit: null, static_zlib_exemption: false } } });
  await act(async () => { await vi.advanceTimersByTimeAsync(250); });
  expect(host.textContent).toContain(t("evaluation.completed"));
  expect(host.textContent).toContain(t("evaluation.unavailable"));
  await click(t("evaluation.cancel"));
  expect(backend.cancelEngineEvaluation).toHaveBeenCalledWith("one");
  expect(host.textContent).toContain(t("evaluation.cancelled"));
  expect(host.textContent).not.toContain(t("evaluation.elapsed"));
});

it("retires a start response arriving after close without publishing output", async () => {
  let resolve!: (value: EvaluationSnapshotDto) => void;
  vi.mocked(backend.startEngineEvaluation).mockReturnValue(new Promise(done => { resolve = done; }));
  await render(); await select(); await click(t("evaluation.start"));
  await act(async () => { root!.unmount(); root = null; });
  await act(async () => resolve(running));
  expect(backend.cancelEngineEvaluation).toHaveBeenCalledWith("one");
  expect(host.textContent).toBe("");
});

it("browser preview cannot fabricate measurements or invoke the runner", async () => {
  vi.mocked(backend.isTauriRuntime).mockReturnValue(false);
  await render(); await select();
  expect(host.textContent).toContain(t("evaluation.native"));
  expect(host.querySelector("button")!.disabled).toBe(true);
  expect(backend.startEngineEvaluation).not.toHaveBeenCalled();
  expect(backend.getEngineEvaluation).not.toHaveBeenCalled();
});
