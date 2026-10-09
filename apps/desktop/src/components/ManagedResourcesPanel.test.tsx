// @vitest-environment jsdom
import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { ManagedResourcesPanel } from "./ManagedResourcesPanel";
import * as managed from "../api/managedResources";
import * as providers from "../api/providers";
import type { EngineProfileDto, ManagedRepairPreviewDto, ManagedResourcesDto } from "../domain/types";
import { t } from "../i18n/resources";

let root: Root | null;
let host: HTMLDivElement;
const profile: EngineProfileDto = { name: "custom", program: "/existing/katago", argv: [], working_dir: null, adapter_kind: "kata_go_analysis", settings: { model_path: "/existing/11750.bin.gz", config_path: "/existing/a.cfg", max_visits: 1 } };
function snapshot(): ManagedResourcesDto {
  return { catalog: { schema_version: 2, source_commit: "frozen", katago_source_commit: "source", katago_version: "1.18.2", engine_repository: "resources", engine_tag: "next-2026-10-08.1", model_tag: "v1.17.1", default_model_id: "b11-flagship", targets: [{ id: "windows-cpu", platform: "windows", backend: "EIGEN", archive: "cpu.zip", size_bytes: 10, sha256: "archive", executable_sha256: "executable", source_availability: "frozen", artifact_availability: "not_checked", hardware_qualification: "host_cpu", runtime_acceptance: "requires_explicit_start", acquisition_allowed: true }], models: [{ id: "b11-flagship", file_name: "kata1-tf3-b11c768-s12002M-d6304M.bin.gz", sha256: "model", size_bytes: 100, minimum_katago_version: "1.17.0" }] }, operation: null };
}
beforeEach(() => {
  globalThis.IS_REACT_ACT_ENVIRONMENT = true;
  vi.useFakeTimers();
  host = document.createElement("div"); document.body.append(host); root = createRoot(host);
  vi.spyOn(providers, "networkSnapshot").mockResolvedValue({ settings: { mode: "manual", manual_host: "localhost", manual_port: 3128 }, policy_revision: 7 });
});
afterEach(async () => {
  await act(async () => root?.unmount()); root = null; host.remove(); vi.useRealTimers(); vi.restoreAllMocks();
});
async function click(key: Parameters<typeof t>[0]) {
  const button = [...host.querySelectorAll("button")].find((element) => element.textContent === t(key));
  if (!button) throw new Error(`Missing action ${key}`);
  await act(async () => button.click());
}
it("requires explicit acquisition, shows progress and cancellation, and never applies or replaces the existing selection automatically", async () => {
  const state = snapshot();
  vi.spyOn(managed, "loadManagedResources").mockImplementation(async () => structuredClone(state));
  vi.spyOn(managed, "acquireManagedResources").mockImplementation(async () => {
    state.operation = { operation_id: "operation", profile_id: "default", target_id: "windows-cpu", model_id: "b11-flagship", phase: "downloading_model", transferred_bytes: 30, total_bytes: 110, message: null, installation: null, routes: [], repair_hardware: null };
    return "operation";
  });
  vi.spyOn(managed, "cancelManagedResources").mockImplementation(async () => { if (state.operation) state.operation.phase = "cancelled"; });
  const begin = vi.fn(() => true), end = vi.fn(), use = vi.fn();
  await act(async () => root!.render(<ManagedResourcesPanel profileId="default" profile={profile} disabled={false} beginOperation={begin} endOperation={end} onUse={use} />));
  expect(managed.acquireManagedResources).not.toHaveBeenCalled();
  expect(profile.settings.model_path).toBe("/existing/11750.bin.gz");
  await click("managed.acquire");
  expect(managed.acquireManagedResources).toHaveBeenCalledWith({ profile_id: "default", profile, target_id: "windows-cpu", model_id: "b11-flagship", policy_revision: 7 });
  expect(host.querySelector("progress")?.value).toBe(30);
  expect(begin).toHaveBeenCalledTimes(1);
  await click("managed.acquire");
  expect(managed.acquireManagedResources).toHaveBeenCalledTimes(1);
  await click("managed.cancel");
  await act(async () => vi.advanceTimersByTimeAsync(400));
  expect(end).toHaveBeenCalledTimes(1);
  expect(use).not.toHaveBeenCalled();
  expect(profile.settings.model_path).toBe("/existing/11750.bin.gz");
});
it("cancels late admission after unmount without applying paths or releasing the lock twice", async () => {
  vi.spyOn(managed, "loadManagedResources").mockResolvedValue(snapshot());
  const admission = Promise.withResolvers<string>();
  vi.spyOn(managed, "acquireManagedResources").mockReturnValue(admission.promise);
  vi.spyOn(managed, "cancelManagedResources").mockResolvedValue(undefined);
  const end = vi.fn(), use = vi.fn();
  await act(async () => root!.render(<ManagedResourcesPanel profileId="default" profile={profile} disabled={false} beginOperation={() => true} endOperation={end} onUse={use} />));
  await click("managed.acquire");
  await act(async () => root!.unmount()); root = null;
  await act(async () => admission.resolve("late"));
  expect(managed.cancelManagedResources).toHaveBeenCalledWith("late");
  expect(end).toHaveBeenCalledTimes(1);
  expect(use).not.toHaveBeenCalled();
});

function trtSnapshot() {
  const state = snapshot();
  state.catalog.targets.push({ ...state.catalog.targets[0], id: "windows-tensorrt", backend: "TENSORRT", acquisition_allowed: false, hardware_qualification: "unknown" });
  return state;
}
function repairPreview(): ManagedRepairPreviewDto {
  return { admission_id: "original-admission", request: { profile_id: "default", profile, target_id: "windows-tensorrt", model_id: "b11-flagship", policy_revision: 7 }, hardware: { status: "supported", gpu_name: "NVIDIA Test GPU", gpu_uuid: "GPU-test", driver_version: "591.66", compute_capability: "12.0", reason: "requires actual inference" }, runtime_version: "TensorRT 10.9.0.34", download_bytes: 3671756714, additional_disk_bytes: 6430823594, available_disk_bytes: 10000000000, repair_allowed: true, reason: "explicit action required" };
}
async function selectTarget(target: string) {
  await act(async () => {
    const select = host.querySelector<HTMLSelectElement>(`select[aria-label="${t("managed.target")}"]`)!;
    select.value = target;
    select.dispatchEvent(new Event("change", { bubbles: true }));
  });
}
it("repairs only the inspected target with explicit action while preserving the non-TRT profile and exposing Cancel", async () => {
  const state = trtSnapshot();
  vi.spyOn(managed, "loadManagedResources").mockImplementation(async () => structuredClone(state));
  vi.spyOn(managed, "inspectManagedTrtRepair").mockResolvedValue(repairPreview());
  vi.spyOn(managed, "repairManagedTrt").mockImplementation(async () => {
    state.operation = { operation_id: "trt-repair", profile_id: "default", target_id: "windows-tensorrt", model_id: "b11-flagship", phase: "downloading_runtime", transferred_bytes: 120, total_bytes: 3671756714, message: "TensorRT 10.9.0.34", installation: null, routes: [], repair_hardware: repairPreview().hardware };
    return "trt-repair";
  });
  vi.spyOn(managed, "cancelManagedResources").mockImplementation(async () => { state.operation!.phase = "cancelled"; });
  const use = vi.fn();
  await act(async () => root!.render(<ManagedResourcesPanel profileId="default" profile={profile} disabled={false} beginOperation={() => true} endOperation={() => {}} onUse={use} />));
  await selectTarget("windows-tensorrt");
  await click("managed.repair.inspect");
  expect(managed.repairManagedTrt).not.toHaveBeenCalled();
  expect(host.textContent).toContain("591.66");
  expect(host.textContent).toContain("TensorRT 10.9.0.34");
  await click("managed.repair.start");
  expect(managed.repairManagedTrt).toHaveBeenCalledExactlyOnceWith("original-admission");
  expect(profile.program).toBe("/existing/katago");
  expect(host.querySelector("progress")?.value).toBe(120);
  await click("managed.cancel");
  expect(managed.cancelManagedResources).toHaveBeenCalledExactlyOnceWith("trt-repair");
  await act(async () => vi.advanceTimersByTimeAsync(400));
  expect(use).not.toHaveBeenCalled();
});
it("cannot adopt a late GPU probe into an edited target or qualify unknown hardware", async () => {
  vi.spyOn(managed, "loadManagedResources").mockResolvedValue(trtSnapshot());
  const probe = Promise.withResolvers<ManagedRepairPreviewDto>();
  vi.spyOn(managed, "inspectManagedTrtRepair").mockReturnValue(probe.promise);
  vi.spyOn(managed, "repairManagedTrt").mockResolvedValue("should-not-start");
  const props = { profileId: "default", profile, disabled: false, beginOperation: () => true, endOperation: () => {}, onUse: vi.fn() };
  await act(async () => root!.render(<ManagedResourcesPanel {...props} />));
  await selectTarget("windows-tensorrt");
  await click("managed.repair.inspect");
  await act(async () => root!.render(<ManagedResourcesPanel {...props} profileId="edited" profile={{ ...profile, name: "new revision" }} />));
  await act(async () => probe.resolve(repairPreview()));
  expect(host.textContent).not.toContain("NVIDIA Test GPU");
  expect(managed.repairManagedTrt).not.toHaveBeenCalled();
  await act(async () => root!.render(<ManagedResourcesPanel {...props} />));
  vi.mocked(managed.inspectManagedTrtRepair).mockResolvedValue({ ...repairPreview(), repair_allowed: false, hardware: { ...repairPreview().hardware, status: "unknown" } });
  await click("managed.repair.inspect");
  const start = [...host.querySelectorAll("button")].find(button => button.textContent === t("managed.repair.start"))!;
  expect(start.disabled).toBe(true);
  await click("managed.repair.start");
  expect(managed.repairManagedTrt).not.toHaveBeenCalled();
});

it("keeps the old draft on repair adoption failure and discards a late adoption after selection changes", async () => {
  const state = trtSnapshot();
  const installation = { target_id: "windows-tensorrt", model_id: "b11-flagship", program: "/managed/katago.exe", model_path: "/managed/model.bin.gz", config_path: "/managed/frozen.cfg", manifest_sha256: "fixture", repair_config_path: "/cache/defaults.cfg" };
  state.operation = { operation_id: "repaired", profile_id: "default", target_id: "windows-tensorrt", model_id: "b11-flagship", phase: "succeeded", transferred_bytes: 100, total_bytes: 100, message: null, installation, routes: [], repair_hardware: repairPreview().hardware };
  vi.spyOn(managed, "loadManagedResources").mockResolvedValue(state);
  vi.spyOn(managed, "managedRepairDraft").mockRejectedValue(new Error("config is missing or inaccessible"));
  const use = vi.fn();
  const props = { profileId: "default", profile, disabled: false, beginOperation: () => true, endOperation: () => {}, onUse: use };
  await act(async () => root!.render(<ManagedResourcesPanel {...props} />));
  await selectTarget("windows-tensorrt");
  await click("managed.use");
  expect(use).not.toHaveBeenCalled();
  expect(host.textContent).toContain("config is missing or inaccessible");
  const adoption = Promise.withResolvers<EngineProfileDto>();
  vi.mocked(managed.managedRepairDraft).mockReturnValue(adoption.promise);
  await click("managed.use");
  await selectTarget("windows-cpu");
  await selectTarget("windows-tensorrt");
  await act(async () => adoption.resolve({ ...profile, program: installation.program }));
  expect(use).not.toHaveBeenCalled();
  vi.mocked(managed.managedRepairDraft).mockResolvedValue({ ...profile, program: installation.program });
  await click("managed.use");
  expect(use).toHaveBeenCalledExactlyOnceWith(installation, { ...profile, program: installation.program });
  expect(profile.program).toBe("/existing/katago");
});
