// @vitest-environment jsdom
import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { ManagedResourcesPanel } from "./ManagedResourcesPanel";
import * as managed from "../api/managedResources";
import * as providers from "../api/providers";
import type { EngineProfileDto, ManagedResourcesDto } from "../domain/types";
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
    state.operation = { operation_id: "operation", profile_id: "default", target_id: "windows-cpu", model_id: "b11-flagship", phase: "downloading_model", transferred_bytes: 30, total_bytes: 110, message: null, installation: null, routes: [] };
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
