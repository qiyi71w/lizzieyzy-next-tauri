// @vitest-environment jsdom
import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import * as backend from "../api/backend";
import type { RuntimeThreadsSnapshotDto } from "../domain/types";
import { RuntimeThreadsPanel } from "./RuntimeThreadsPanel";
import { t } from "../i18n/resources";

const unknown: RuntimeThreadsSnapshotDto = { run_id: "run-a", profile_revision: "revision-a", supported: true, reason: null,
  minimum: 1, maximum: 4096, sources: { entries: [], saved: 1, launch_override: 2, effective: 2, analysis_threads: null, error: null },
  request_id: null, requested: null, actual: null, temporary: false, status: "unknown", failure: null };
let host: HTMLDivElement;
let root: Root;
beforeEach(() => {
  globalThis.IS_REACT_ACT_ENVIRONMENT = true;
  vi.useFakeTimers();
  host = document.createElement("div"); document.body.append(host); root = createRoot(host);
  vi.spyOn(backend, "isTauriRuntime").mockReturnValue(true);
  vi.spyOn(backend, "getRuntimeThreads").mockResolvedValue(unknown);
  vi.spyOn(backend, "requestRuntimeThreads").mockImplementation(async (request) => ({ ...unknown, request_id: request.identity.request_id,
    requested: request.value, actual: request.action === "reset" ? 2 : request.value, status: "confirmed", temporary: request.action === "apply" }));
});
afterEach(async () => { await act(async () => root.unmount()); host.remove(); vi.restoreAllMocks(); vi.useRealTimers(); });
async function render(runId: string | null = "run-a") { await act(async () => root.render(<RuntimeThreadsPanel runId={runId} disabled={false} />)); }
async function edit(value: string) {
  await act(async () => {
    const input = host.querySelector("input")!;
    Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")!.set!.call(input, value);
    input.dispatchEvent(new Event("input", { bubbles: true }));
  });
}
async function click(key: "apply" | "reset" | "read") { await act(async () => Array.from(host.querySelectorAll("button")).find(button => button.textContent === t(`threads.${key}`))!.click()); }

it("keeps editing unsent and uses the same explicit form action for Apply and reset", async () => {
  await render(); await edit("3");
  expect(backend.requestRuntimeThreads).not.toHaveBeenCalled();
  await click("apply");
  expect(backend.requestRuntimeThreads).toHaveBeenLastCalledWith(expect.objectContaining({ action: "apply", value: 3 }));
  expect(host.textContent).toContain(t("threads.confirmed"));
  await edit("4");
  await act(async () => host.querySelector("form")!.dispatchEvent(new Event("submit", { bubbles: true, cancelable: true })));
  expect(backend.requestRuntimeThreads).toHaveBeenLastCalledWith(expect.objectContaining({ action: "apply", value: 4 }));
  await click("reset");
  expect(backend.requestRuntimeThreads).toHaveBeenLastCalledWith(expect.objectContaining({ action: "reset", value: null }));
  expect(host.querySelector("input")!.value).toBe("4");
  expect(host.textContent).not.toContain(t("threads.temporary"));
});

it("readback failures and delayed success never replace a newer unsent draft", async () => {
  let resolve!: (value: RuntimeThreadsSnapshotDto) => void;
  vi.mocked(backend.requestRuntimeThreads).mockReturnValue(new Promise(done => { resolve = done; }));
  await render(); await edit("2"); await click("apply");
  expect(host.textContent).toContain(t("threads.pending"));
  await edit("7");
  const request = vi.mocked(backend.requestRuntimeThreads).mock.calls[0][0];
  await act(async () => resolve({ ...unknown, request_id: request.identity.request_id, requested: 2, actual: 1, status: "failed", failure: "timeout" }));
  expect(host.querySelector("input")!.value).toBe("7");
  expect(host.textContent).toContain(t("threads.failed"));
  expect(host.textContent).toContain("timeout");
});

it("expires old Run state and rejects a late response without discarding the unsent draft", async () => {
  let resolve!: (value: RuntimeThreadsSnapshotDto) => void;
  vi.mocked(backend.requestRuntimeThreads).mockReturnValue(new Promise(done => { resolve = done; }));
  await render(); await edit("2"); await click("apply"); await edit("8");
  const request = vi.mocked(backend.requestRuntimeThreads).mock.calls[0][0];
  vi.mocked(backend.getRuntimeThreads).mockResolvedValue({ ...unknown, run_id: "run-b", profile_revision: "revision-b" });
  await render("run-b");
  await act(async () => resolve({ ...unknown, request_id: request.identity.request_id, requested: 2, actual: 2, temporary: true, status: "confirmed" }));
  expect(host.textContent).toContain(t("threads.unknown"));
  expect(host.textContent).not.toContain(t("threads.temporary"));
  expect(host.querySelector("input")!.value).toBe("8");
  expect(backend.requestRuntimeThreads).toHaveBeenCalledTimes(1);
  await render(null);
  expect(host.querySelector("button")!.disabled).toBe(true);
});

it("refuses unsupported browser and invalid whole-number domains without sending", async () => {
  await render();
  for (const value of ["0", "4097", "2.5", "NaN", ""]) {
    await edit(value); await click("apply");
    expect(host.querySelector("button")!.disabled).toBe(true);
  }
  expect(backend.requestRuntimeThreads).not.toHaveBeenCalled();
  vi.mocked(backend.isTauriRuntime).mockReturnValue(false);
  await render(); await edit("2"); await click("apply");
  expect(host.textContent).toContain(t("threads.native"));
  expect(backend.requestRuntimeThreads).not.toHaveBeenCalled();
});
