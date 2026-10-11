// @vitest-environment jsdom
import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import * as backend from "../api/backend";
import type { EngineProfileRecordDto, EvaluationSnapshotDto } from "../domain/types";
import { EngineEvaluationPanel } from "./EngineEvaluationPanel";
import { t } from "../i18n/resources";
import productionResult from "../../../../../crates/engine-manager/tests/fixtures/katago-1.18.2-benchmark/variations-result.json";

const idle: EvaluationSnapshotDto = { evaluation_id: null, target_id: null, input_revision: null, phase: "idle", process_id: null, exit_code: null, output: [], output_truncated: false, message: null, result: null };
const running: EvaluationSnapshotDto = { ...idle, evaluation_id: "one", target_id: "saved", input_revision: "revision", phase: "running", process_id: 123 };
const profiles: EngineProfileRecordDto[] = [{ id: "saved", preload: false, profile: { name: "Saved local", program: "katago", argv: [], working_dir: null, adapter_kind: "generic_gtp", settings: {} } }];
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
    result: { evaluation_id: "one", target_id: "saved", input_revision: "revision", elapsed_ms: 1000,
      measurements: [{ measured_threads: 1, nn_evals_per_second: 87.25, search_visits_per_second: null }],
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

it("renders the production parser DTO as independently paired rows with frozen identity", async () => {
  const completed: EvaluationSnapshotDto = {
    ...idle, phase: "completed", evaluation_id: productionResult.evaluation_id,
    target_id: productionResult.target_id, input_revision: productionResult.input_revision,
    exit_code: 0, result: productionResult,
  };
  vi.mocked(backend.getEngineEvaluation).mockResolvedValue(completed);
  await render();
  expect(host.textContent).toContain(t("evaluation.completed"));
  expect(host.textContent).toContain(productionResult.evaluation_id);
  expect(host.textContent).toContain(productionResult.input_revision);
  expect(host.textContent).toContain("Using Eigen(CPU) backend");
  for (const resource of productionResult.qualified_resource.resources) {
    expect(host.textContent).toContain(resource.sha256);
  }
  expect(Array.from(host.querySelectorAll("thead th"), cell => cell.textContent)).toEqual([
    t("evaluation.threads"), t("evaluation.nnSpeed"), t("evaluation.speed"),
  ]);
  const unavailable = t("evaluation.unavailable");
  expect(Array.from(host.querySelectorAll("tbody tr"), row =>
    Array.from(row.children, cell => cell.textContent))).toEqual([
    ["1", "87.25", "123.5"], ["2", unavailable, "250"],
    ["4", "310.5", unavailable], ["8", "411.75", unavailable],
    ["16", unavailable, "500"], ["32", unavailable, unavailable],
  ]);
  const region = host.querySelector<HTMLElement>("[role=region]")!;
  expect(region.getAttribute("aria-label")).toBe(t("evaluation.measurements"));
  expect(region.tabIndex).toBe(0);
  region.focus();
  expect(document.activeElement).toBe(region);
  // Editor/profile changes cannot relabel the result's own ID, threads or resource tuple.
  await act(async () => root!.render(<EngineEvaluationPanel profiles={[{
    ...profiles[0], profile: { ...profiles[0].profile, name: "Later target", argv: ["-t", "64"] },
  }]} disabled={false} />));
  expect(host.querySelector("tbody tr")!.textContent).toBe("187.25123.5");
  expect(host.textContent).toContain(productionResult.input_revision);
  for (const phase of ["failed", "cancelled", "yielded", "retired"] as const) {
    vi.mocked(backend.getEngineEvaluation).mockResolvedValue({ ...completed, phase, result: null });
    await act(async () => { await vi.advanceTimersByTimeAsync(250); });
    expect(host.textContent).toContain(t(`evaluation.${phase}`));
    expect(host.querySelector("table")).toBeNull();
  }
});
