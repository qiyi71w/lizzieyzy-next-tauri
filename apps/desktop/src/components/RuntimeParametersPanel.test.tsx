// @vitest-environment jsdom
import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import * as backend from "../api/backend";
import type { RuntimeParametersSnapshotDto } from "../domain/types";
import { RuntimeParametersPanel } from "./RuntimeParametersPanel";
import { RuntimeThreadsPanel } from "./RuntimeThreadsPanel";
import { t } from "../i18n/resources";

const unknown: RuntimeParametersSnapshotDto = { run_id: "run-a", profile_revision: "revision-a", supported: true, reason: null,
  request_id: null, last_valid: null, status: "unknown", failure: null };
const pair = { playout_doubling_advantage: -0.5, analysis_wide_root_noise: 0.04 };
let host: HTMLDivElement;
let root: Root;
beforeEach(() => {
  globalThis.IS_REACT_ACT_ENVIRONMENT = true;
  vi.useFakeTimers();
  host = document.createElement("div"); document.body.append(host); root = createRoot(host);
  vi.spyOn(backend, "isTauriRuntime").mockReturnValue(true);
  vi.spyOn(backend, "getRuntimeParameters").mockResolvedValue(unknown);
  vi.spyOn(backend, "readRuntimeParameters").mockImplementation(async identity => ({ ...unknown, request_id: identity.request_id, last_valid: pair, status: "confirmed" }));
  vi.spyOn(backend, "getRuntimeThreads").mockResolvedValue({ ...unknown, minimum: 1, maximum: 4096, sources: null, requested: null, actual: null, temporary: false });
  vi.spyOn(backend, "requestRuntimeThreads");
});
afterEach(async () => { await act(async () => root.unmount()); host.remove(); vi.restoreAllMocks(); vi.useRealTimers(); });
async function render(runId: string | null = "run-a") { await act(async () => root.render(<><RuntimeThreadsPanel runId={runId} disabled={false} /><RuntimeParametersPanel runId={runId} disabled={false} /></>)); }
function panel() { return host.querySelector(`[aria-label="${t("parameters.title")}"]`)!; }
async function readPair() { await act(async () => panel().querySelector("button")!.click()); }
async function editDraft(value: string) {
  await act(async () => {
    const input = host.querySelector("input")!;
    Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")!.set!.call(input, value);
    input.dispatchEvent(new Event("input", { bubbles: true }));
  });
}

it("starts unknown without protocol reads and explicitly submits one readonly pair by keyboard action", async () => {
  await render(); await editDraft("7");
  expect(panel().textContent).toContain(t("parameters.unknown"));
  expect(panel().querySelectorAll("dd")).toHaveLength(0);
  expect(backend.readRuntimeParameters).not.toHaveBeenCalled();
  await act(async () => panel().querySelector("form")!.dispatchEvent(new Event("submit", { bubbles: true, cancelable: true })));
  expect(backend.readRuntimeParameters).toHaveBeenCalledTimes(1);
  expect(backend.readRuntimeParameters).toHaveBeenCalledWith({ run_id: "run-a", profile_revision: "revision-a", request_id: expect.any(String) });
  expect(panel().textContent).toContain(t("parameters.confirmed"));
  expect(Array.from(panel().querySelectorAll("dd"), element => element.textContent)).toEqual(["-0.5", "0.04"]);
  expect(host.querySelector("input")!.value).toBe("7");
  expect(backend.requestRuntimeThreads).not.toHaveBeenCalled();
});

it("retains the complete last pair as stale while pending and failed without replacing newer edits", async () => {
  await render(); await readPair();
  let resolve!: (value: RuntimeParametersSnapshotDto) => void;
  vi.mocked(backend.readRuntimeParameters).mockReturnValue(new Promise(done => { resolve = done; }));
  await readPair(); await editDraft("9");
  expect(panel().textContent).toContain(t("parameters.pending"));
  expect(panel().textContent).toContain(t("parameters.stale"));
  expect(panel().textContent).not.toContain(t("parameters.confirmed"));
  expect(panel().querySelector("button")!.disabled).toBe(true);
  const identity = vi.mocked(backend.readRuntimeParameters).mock.calls[1][0];
  await act(async () => resolve({ ...unknown, request_id: identity.request_id, status: "failed", last_valid: pair, failure: "missing half timed out" }));
  expect(panel().textContent).toContain(t("parameters.failed"));
  expect(panel().textContent).toContain(t("parameters.stale"));
  expect(panel().textContent).toContain("missing half timed out");
  expect(Array.from(panel().querySelectorAll("dd"), element => element.textContent)).toEqual(["-0.5", "0.04"]);
  expect(host.querySelector("input")!.value).toBe("9");
  expect(backend.readRuntimeParameters).toHaveBeenCalledTimes(2);
});

it("does not resurrect a prior confirmation after an admission failure or stale poll", async () => {
  await render(); await readPair();
  vi.mocked(backend.getRuntimeParameters).mockResolvedValue({ ...unknown, status: "confirmed", last_valid: pair });
  vi.mocked(backend.readRuntimeParameters).mockRejectedValue(new Error("finite work owns admission"));
  await readPair();
  await act(async () => vi.advanceTimersByTimeAsync(900));
  expect(panel().textContent).toContain(t("parameters.failed"));
  expect(panel().textContent).toContain(t("parameters.stale"));
  expect(panel().textContent).not.toContain(t("parameters.confirmed"));
  expect(backend.readRuntimeParameters).toHaveBeenCalledTimes(2);
});

it("retires old Run and old round responses while retaining unsent editor input", async () => {
  let resolve!: (value: RuntimeParametersSnapshotDto) => void;
  vi.mocked(backend.readRuntimeParameters).mockReturnValue(new Promise(done => { resolve = done; }));
  await render(); await readPair(); await editDraft("8");
  const identity = vi.mocked(backend.readRuntimeParameters).mock.calls[0][0];
  vi.mocked(backend.getRuntimeParameters).mockResolvedValue({ ...unknown, run_id: "run-b", profile_revision: "revision-b" });
  await render("run-b");
  await act(async () => resolve({ ...unknown, request_id: identity.request_id, status: "confirmed", last_valid: pair }));
  expect(panel().textContent).toContain(t("parameters.unknown"));
  expect(panel().querySelectorAll("dd")).toHaveLength(0);
  expect(host.querySelector("input")!.value).toBe("8");
  await render(null);
  expect(panel().querySelector("button")!.disabled).toBe(true);
  expect(backend.readRuntimeParameters).toHaveBeenCalledTimes(1);
});

it("visibly refuses browser and unsupported instances without hidden reads", async () => {
  vi.mocked(backend.getRuntimeParameters).mockResolvedValue({ ...unknown, supported: false, reason: "JSONL is separate" });
  await render(); await readPair();
  expect(panel().textContent).toContain("JSONL is separate");
  expect(panel().querySelector("button")!.disabled).toBe(true);
  vi.mocked(backend.isTauriRuntime).mockReturnValue(false);
  await render(); await readPair();
  expect(panel().textContent).toContain(t("parameters.native"));
  expect(backend.readRuntimeParameters).not.toHaveBeenCalled();
});
