/// <reference lib="esnext.promise" />
// @vitest-environment jsdom
import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { beforeEach, afterEach, expect, it, vi } from "vitest";
import * as backend from "../api/backend";
import * as api from "../api/measuredReport";
import type { EngineProfileRecordDto, MeasuredReviewDto } from "../domain/types";
import { MeasuredReportPanel } from "./MeasuredReportPanel";
import { t } from "../i18n/resources";

const target: EngineProfileRecordDto = { id: "saved", preload: false, profile: { name: "Saved target", program: "katago", argv: ["  manual bytes  "], working_dir: null, adapter_kind: "generic_gtp", settings: {} }, measured_scenes: { revision: 4, live: null, whole_game: null } };
const review: MeasuredReviewDto = { operation_id: "receipt", generation: 7, target_id: "saved", target_name: "Saved target", scene: "live", profile_revision: "revision", policy_revision: 0, overlay_revision: 4, baseline_parameters: { numSearchThreads: 4, nnMaxBatchSize: 8 }, candidate_parameters: { numSearchThreads: 8, nnMaxBatchSize: 8 }, assessment: { eligible: true, needs_five_pairs: false, reasons: [], pair_ratios: [1.04, 1.05, 1.06], median_ratio: 1.05, sample_cv: 0.01, experimental: true }, qualification_reasons: [], cold_verification_seconds: 0.1, report: { schemaVersion: 1 } };
let host: HTMLDivElement;
let root: Root | null;
const begin = vi.fn(() => true);
const end = vi.fn();
const saved = vi.fn();
beforeEach(() => {
  globalThis.IS_REACT_ACT_ENVIRONMENT = true;
  host = document.createElement("div"); document.body.append(host); root = createRoot(host);
  begin.mockClear(); end.mockClear(); saved.mockClear();
  vi.spyOn(backend, "isTauriRuntime").mockReturnValue(true);
  vi.spyOn(api, "retireMeasuredReport").mockResolvedValue();
  vi.spyOn(api, "measuredReportBusy").mockResolvedValue(false);
  vi.spyOn(api, "reviewMeasuredReport").mockResolvedValue(review);
  vi.spyOn(api, "applyMeasuredReport").mockResolvedValue({ version: 2, selected_profile_id: "saved", startup: { mode: "off" }, last_primary_profile_id: null, startup_evaluation: { enabled: false, target_profile_id: null }, profiles: [target] });
  vi.spyOn(api, "restoreMeasuredReport").mockResolvedValue({ version: 2, selected_profile_id: "saved", startup: { mode: "off" }, last_primary_profile_id: null, startup_evaluation: { enabled: false, target_profile_id: null }, profiles: [target] });
});
afterEach(async () => { await act(async () => root?.unmount()); host.remove(); vi.restoreAllMocks(); });
async function render(inputRevision = "draft") { await act(async () => root!.render(<MeasuredReportPanel target={target} documentGeneration={1} documentNode={{ indices: [] }} inputRevision={inputRevision} disabled={false} beginOperation={begin} endOperation={end} onProfilesSaved={saved} />)); }
async function importFile() {
  const file = new File(['{"schemaVersion":1}'], "report.json", { type: "application/json" });
  Object.defineProperty(file, "text", { value: () => Promise.resolve('{"schemaVersion":1}') });
  await act(async () => { const input = host.querySelector<HTMLInputElement>('input[accept="application/json,.json"]')!; Object.defineProperty(input, "files", { configurable: true, value: [file] }); input.dispatchEvent(new Event("change", { bubbles: true })); });
}
async function click(text: string) { await act(async () => Array.from(host.querySelectorAll("button")).find((button) => button.textContent === text)!.click()); }

it("imports without writing and requires explicit confirmation before Apply", async () => {
  await render(); await importFile();
  expect(api.reviewMeasuredReport).toHaveBeenCalledWith("saved", '{"schemaVersion":1}');
  expect(api.applyMeasuredReport).not.toHaveBeenCalled(); expect(saved).not.toHaveBeenCalled();
  await click(t("measured.apply")); expect(api.applyMeasuredReport).not.toHaveBeenCalled();
  await act(async () => { const checkbox = Array.from(host.querySelectorAll<HTMLInputElement>('input[type="checkbox"]')).find((input) => input.parentElement?.textContent?.includes(t("measured.confirm")))!; checkbox.click(); });
  await click(t("measured.apply")); expect(api.applyMeasuredReport).toHaveBeenCalledWith(review); expect(saved).toHaveBeenCalledWith([target]);
});
it("retires a late Review when the draft changes without publishing a receipt", async () => {
  const { promise, resolve } = Promise.withResolvers<MeasuredReviewDto>();
  vi.mocked(api.reviewMeasuredReport).mockReturnValue(promise);
  await render(); await importFile(); await render("edited");
  await act(async () => resolve(review));
  expect(host.textContent).not.toContain("Paired ratios:"); expect(api.retireMeasuredReport).toHaveBeenCalled(); expect(end).toHaveBeenCalledTimes(1);
});
it("restores the captured saved ID and exact overlay revision without rewriting its manual argv", async () => {
  await render(); await click(t("measured.restore"));
  expect(api.restoreMeasuredReport).toHaveBeenCalledWith("saved", 4); expect(target.profile.argv).toEqual(["  manual bytes  "]);
});
it("rejects reentrant imports while the first worker owns the shared operation", async () => {
  const { promise, resolve } = Promise.withResolvers<MeasuredReviewDto>();
  vi.mocked(api.reviewMeasuredReport).mockReturnValue(promise);
  await render(); await importFile(); await importFile();
  expect(api.reviewMeasuredReport).toHaveBeenCalledTimes(1); expect(begin).toHaveBeenCalledTimes(1);
  await act(async () => resolve(review));
});
it("browser fallback never offers runnable measurement or imports as native evidence", async () => {
  vi.mocked(backend.isTauriRuntime).mockReturnValue(false); await render();
  expect(host.querySelector<HTMLInputElement>('input[accept="application/json,.json"]')!.disabled).toBe(true);
  expect(host.textContent).toContain(t("measured.native")); expect(api.reviewMeasuredReport).not.toHaveBeenCalled();
});
