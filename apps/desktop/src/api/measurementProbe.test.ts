/// <reference lib="esnext.promise" />
// @vitest-environment jsdom
import { afterEach, expect, it, vi } from "vitest";
import * as api from "./measuredReport";
import { measureWithWebViewProbe } from "./measurementProbe";
import type { ApplicationMeasurementProgressDto, ApplicationMeasurementRequestDto, ApplicationMeasurementResultDto } from "../domain/types";

const request: ApplicationMeasurementRequestDto = { target_id: "saved", scene: "live", candidate_parameters: { numSearchThreads: 8, nnMaxBatchSize: 8 }, pairs: 3, budget: 100, fixture_sgf: "(;SZ[9])", replace_current_document: true };
afterEach(() => { vi.useRealTimers(); vi.restoreAllMocks(); });

it("records schedule/execute times and confirms the final flush for the exact row", async () => {
  vi.useFakeTimers();
  vi.spyOn(document, "hidden", "get").mockReturnValue(false);
  const completion = Promise.withResolvers<ApplicationMeasurementResultDto>();
  vi.spyOn(api, "measureApplication").mockReturnValue(completion.promise);
  const progress: ApplicationMeasurementProgressDto = { operation_id: "operation", row_id: "row", scene: "live", round: 1, candidate: false, sampling: true, closing: false, backend_clock_ms: 20 };
  vi.spyOn(api, "measurementProgress").mockImplementation(async () => ({ ...progress }));
  const submit = vi.spyOn(api, "submitMeasurementProbe").mockResolvedValue();
  const retire = vi.spyOn(api, "retireMeasuredReport").mockResolvedValue();
  const result = measureWithWebViewProbe(request);
  await vi.advanceTimersByTimeAsync(50);
  progress.sampling = false; progress.closing = true;
  await vi.advanceTimersByTimeAsync(30);
  expect(submit.mock.calls.some(([probe]) => probe.finished)).toBe(true);
  const finalProbe = submit.mock.calls.find(([probe]) => probe.finished)?.[0];
  expect(finalProbe?.sample_times_ms.length).toBeGreaterThan(0);
  for (const [probe] of submit.mock.calls) {
    expect(probe.operation_id).toBe("operation"); expect(probe.row_id).toBe("row");
    expect(probe.sample_times_ms).toHaveLength(probe.samples_seconds.length);
    expect(probe.clock_exchange_ms[0]).toBeLessThanOrEqual(probe.clock_exchange_ms[2]);
    expect(probe.clock_exchange_ms[1]).toBe(20);
    probe.sample_times_ms.forEach(([expected, actual], index) => expect(probe.samples_seconds[index]).toBe(Math.max(0, actual - expected) / 1000));
  }
  const value = { report: { schemaVersion: 1 }, evidence: { measured: true } };
  completion.resolve(value); expect(await result).toEqual(value);
  const calls = submit.mock.calls.length;
  await vi.advanceTimersByTimeAsync(100);
  expect(submit).toHaveBeenCalledTimes(calls); expect(retire).not.toHaveBeenCalled();
});

it("retires hidden measurements once and never substitutes a successful report", async () => {
  vi.useFakeTimers();
  const hidden = vi.spyOn(document, "hidden", "get").mockReturnValue(false);
  const completion = Promise.withResolvers<ApplicationMeasurementResultDto>();
  vi.spyOn(api, "measureApplication").mockReturnValue(completion.promise);
  vi.spyOn(api, "measurementProgress").mockResolvedValue(null);
  vi.spyOn(api, "submitMeasurementProbe").mockResolvedValue();
  const retire = vi.spyOn(api, "retireMeasuredReport").mockResolvedValue();
  const result = measureWithWebViewProbe(request);
  hidden.mockReturnValue(true);
  await vi.advanceTimersByTimeAsync(100);
  expect(retire).toHaveBeenCalledTimes(1);
  completion.resolve({ report: {}, evidence: {} });
  await expect(result).rejects.toThrow("hidden");
});

it("retains the actual send/receive bounds when progress delivery is delayed", async () => {
  vi.useFakeTimers();
  vi.spyOn(document, "hidden", "get").mockReturnValue(false);
  let now = 100;
  vi.spyOn(performance, "now").mockImplementation(() => now);
  const completion = Promise.withResolvers<ApplicationMeasurementResultDto>();
  const delayed = Promise.withResolvers<ApplicationMeasurementProgressDto | null>();
  vi.spyOn(api, "measureApplication").mockReturnValue(completion.promise);
  vi.spyOn(api, "measurementProgress").mockReturnValue(delayed.promise);
  const submit = vi.spyOn(api, "submitMeasurementProbe").mockResolvedValue();
  vi.spyOn(api, "retireMeasuredReport").mockResolvedValue();
  const result = measureWithWebViewProbe(request);
  await vi.advanceTimersByTimeAsync(10);
  now = 140;
  delayed.resolve({ operation_id: "operation", row_id: "row", scene: "live", round: 1, candidate: false, sampling: true, closing: false, backend_clock_ms: 8 });
  await vi.advanceTimersByTimeAsync(10);
  expect(submit.mock.calls[0][0].clock_exchange_ms).toEqual([100, 8, 140]);
  completion.resolve({ report: {}, evidence: {} });
  await result;
});
