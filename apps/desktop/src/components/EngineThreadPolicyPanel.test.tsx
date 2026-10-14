/// <reference lib="esnext.promise" />
// @vitest-environment jsdom
import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, expect, it, vi, type Mock } from "vitest";
import * as backend from "../api/backend";
import * as api from "../api/threadPolicy";
import type { EngineProfileRecordDto, EngineProfilesSettingsDto, EngineRunDto, RuntimeThreadsSnapshotDto, ThreadPolicyStatusDto } from "../domain/types";
import { EngineThreadPolicyPanel } from "./EngineThreadPolicyPanel";
import { t } from "../i18n/resources";

const a: EngineProfileRecordDto = { id: "a", preload: false, profile: { name: "Engine A", program: "katago", argv: [], working_dir: null, adapter_kind: "generic_gtp", settings: {} } };
const b: EngineProfileRecordDto = { ...a, id: "b", profile: { ...a.profile, name: "Engine B" } };
const qualified: ThreadPolicyStatusDto = { target_id: "a", input_revision: "input-a", policy: { strategy: "cfg", revision: 3, recommendation: null }, validity: "valid", reason: "policy_valid", recommendation: null };
let host: HTMLDivElement;
let root: Root;
let published: Mock;
beforeEach(() => {
  globalThis.IS_REACT_ACT_ENVIRONMENT = true;
  host = document.createElement("div"); document.body.append(host); root = createRoot(host);
  published = vi.fn();
  vi.spyOn(backend, "isTauriRuntime").mockReturnValue(true);
  vi.spyOn(api, "getThreadPolicyStatus").mockResolvedValue(qualified);
  vi.spyOn(api, "saveThreadPolicy").mockResolvedValue({ version: 2, selected_profile_id: "b", startup: { mode: "off" }, last_primary_profile_id: null, startup_evaluation: { enabled: false, target_profile_id: null }, profiles: [a, b] });
});
afterEach(async () => { await act(async () => root.unmount()); host.remove(); vi.restoreAllMocks(); });
async function render(record = a, evaluationId: string | null = "evaluation-a") {
  await act(async () => root.render(<EngineThreadPolicyPanel key={record.id} record={record} evaluationId={evaluationId} disabled={false} onSaved={published} />));
}
async function choose(value: string) {
  await act(async () => { const select = host.querySelector("select")!; select.value = value; select.dispatchEvent(new Event("change", { bubbles: true })); });
}
function button(key: "save" | "cancel" | "refresh") { return [...host.querySelectorAll("button")].find(b => b.textContent === t(`threadPolicy.${key}`))!; }
async function click(key: "save" | "cancel" | "refresh") { await act(async () => button(key).click()); }

it("keeps measurement and policy selection as a draft until explicit Save; Cancel writes nothing", async () => {
  await render();
  await choose("benchmark");
  expect(api.saveThreadPolicy).not.toHaveBeenCalled();
  await click("cancel");
  expect(host.querySelector("select")!.value).toBe("cfg");
  expect(api.saveThreadPolicy).not.toHaveBeenCalled();
  await choose("benchmark"); await click("save");
  expect(api.saveThreadPolicy).toHaveBeenCalledWith({ target_id: "a", expected_revision: 3, input_revision: "input-a", strategy: "benchmark", evaluation_id: "evaluation-a" });
  expect(published).toHaveBeenCalledWith([a, b]);
  expect(host.textContent).toContain(t("threadPolicy.saved"));
});

it("retains the draft and committed catalog after a failed write and allows an explicit retry", async () => {
  vi.mocked(api.saveThreadPolicy).mockRejectedValueOnce(new Error("disk full"));
  await render(); await choose("benchmark"); await click("save");
  expect(host.querySelector("select")!.value).toBe("benchmark");
  expect(host.textContent).toContain(`${t("threadPolicy.failed")}disk full`);
  expect(published).not.toHaveBeenCalled();
  await click("save");
  expect(api.saveThreadPolicy).toHaveBeenCalledTimes(2);
  expect(published).toHaveBeenCalledTimes(1);
});

it.each(["none", "invalid", "changed", "unknown"] as const)("shows %s qualification and refuses Benchmark Save without treating it as CFG", async (validity) => {
  vi.mocked(api.getThreadPolicyStatus).mockResolvedValue({ ...qualified, validity, reason: "policy_resources_changed" });
  await render(); await choose("benchmark");
  expect(host.textContent).toContain(t(`threadPolicy.${validity}`));
  expect(button("save").disabled).toBe(true);
  expect(host.querySelector("select")!.value).toBe("benchmark");
  await choose("cfg");
  expect(button("save").disabled).toBe(false);
});

it("fences a late A qualification after switching to B and reports B's identity", async () => {
  const pending = Promise.withResolvers<ThreadPolicyStatusDto>();
  vi.mocked(api.getThreadPolicyStatus).mockReturnValueOnce(pending.promise)
    .mockResolvedValue({ ...qualified, target_id: "b", input_revision: "input-b", validity: "unknown", reason: "policy_environment_unknown" });
  await render(a); await render(b, null);
  await act(async () => pending.resolve(qualified));
  expect(host.textContent).toContain("Engine B (b)");
  expect(host.textContent).toContain(t("threadPolicy.unknown"));
  await choose("benchmark"); expect(button("save").disabled).toBe(true);
  expect(api.saveThreadPolicy).not.toHaveBeenCalled();
});

it("browser preview exposes native qualification limits while draft Cancel stays keyboard accessible", async () => {
  vi.mocked(backend.isTauriRuntime).mockReturnValue(false);
  await render(); await choose("benchmark");
  expect(host.textContent).toContain(t("threadPolicy.reason.policy_native_only"));
  expect(button("save").disabled).toBe(true);
  button("cancel").focus(); expect(document.activeElement).toBe(button("cancel"));
  await click("cancel"); expect(host.querySelector("select")!.value).toBe("cfg");
  expect(api.getThreadPolicyStatus).not.toHaveBeenCalled();
});

it("keeps recommendation, saved strategy, launch and actual readback independent", async () => {
  const resource = { profile_revision: "run-revision", resources: [], origin: "local_unknown", version: "1.18.2", backend: "Eigen", source_commit: null, static_zlib_exemption: false,
    thread_sources: { entries: [], saved: 2, launch_override: 6, effective: 6, analysis_threads: 1, error: null } };
  const recommendation = { evaluation_id: "evaluation-a", target_id: "a", input_revision: "input-a", policy_revision: 3, source_revision: "upstream", threads: 4, seconds_per_move: "5", source_row: "recommended", qualified_resource: resource, command_revision: "command", environment: "environment" };
  vi.mocked(api.getThreadPolicyStatus).mockResolvedValue({ ...qualified, recommendation });
  const run: EngineRunDto = { run_id: "run-b", profile_id: "b", adapter_kind: "generic_gtp", profile_snapshot: b.profile, qualified_resource: resource };
  const runtime: RuntimeThreadsSnapshotDto = { run_id: "run-b", profile_revision: "run-revision", supported: true, reason: null, minimum: 1, maximum: 4096, sources: resource.thread_sources, request_id: "read", requested: 8, actual: 8, temporary: true, status: "confirmed", failure: null };
  const read = vi.spyOn(backend, "getRuntimeThreads").mockResolvedValue(runtime);
  await act(async () => root.render(<EngineThreadPolicyPanel record={a} evaluationId="evaluation-a" disabled={false} currentRun={run} />));
  expect([...host.querySelectorAll("dd")].map(node => node.textContent)).toEqual([
    `4 · ${t("threadPolicy.valid")}`, t("threadPolicy.cfg"), "6 · Engine B (b) · run-b", `8 · ${t("threads.confirmed")}`,
  ]);
  await choose("benchmark");
  expect(host.querySelectorAll("dd")[1].textContent).toBe(t("threadPolicy.cfg"));
  const next = Promise.withResolvers<RuntimeThreadsSnapshotDto>();
  read.mockReturnValue(next.promise);
  await act(async () => root.render(<EngineThreadPolicyPanel record={a} evaluationId="evaluation-a" disabled={false} currentRun={{ ...run, run_id: "new-run" }} />));
  expect(host.querySelectorAll("dd")[3].textContent).toContain(t("evaluation.unavailable"));
  await act(async () => next.resolve(runtime));
  expect(host.querySelectorAll("dd")[3].textContent).not.toContain("8");
});

it("rejects qualification and save feedback from an earlier saved input revision", async () => {
  const stale = Promise.withResolvers<ThreadPolicyStatusDto>();
  vi.mocked(api.getThreadPolicyStatus).mockReturnValueOnce(stale.promise).mockResolvedValue({ ...qualified, validity: "changed" });
  await render();
  const changed = { ...a, profile: { ...a.profile, argv: ["-t", "3"] } };
  await render(changed);
  await act(async () => stale.resolve(qualified));
  expect(host.textContent).toContain(t("threadPolicy.changed"));
  expect(host.textContent).not.toContain(t("threadPolicy.valid"));
  const saving = Promise.withResolvers<EngineProfilesSettingsDto>();
  vi.mocked(api.saveThreadPolicy).mockReturnValueOnce(saving.promise);
  await click("save");
  await render({ ...changed, profile: { ...changed.profile, argv: ["-t", "5"] } });
  await act(async () => saving.reject(new Error("OLD SAVE ERROR")));
  expect(host.textContent).not.toContain("OLD SAVE ERROR");
  expect(published).not.toHaveBeenCalled();
});
