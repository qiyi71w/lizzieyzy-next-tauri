import { measureApplication, measurementProgress, retireMeasuredReport, submitMeasurementProbe } from "./measuredReport";
import type { ApplicationMeasurementProgressDto, ApplicationMeasurementRequestDto, ApplicationMeasurementResultDto } from "../domain/types";

/** Samples the actual WebView timer; never substitutes engine timing or synthetic delays. */
export async function measureWithWebViewProbe(request: ApplicationMeasurementRequestDto): Promise<ApplicationMeasurementResultDto> {
  if (document.hidden) throw new Error("Application measurement requires a visible WebView");
  let progress: ApplicationMeasurementProgressDto | null = null;
  let samples: number[] = [];
  let sampleTimes: [number, number][] = [];
  let clockExchange: [number, number, number] | null = null;
  let querying = false;
  let sending = false;
  let closedRow: string | null = null;
  let failure: unknown;
  let active = true;
  let previous = performance.now();
  const tick = window.setInterval(() => {
    const now = performance.now();
    const expected = previous + 10;
    const delay = Math.max(0, now - expected) / 1000;
    previous = now;
    if (document.hidden) failure = new Error("WebView hidden during application measurement");
    if (progress?.sampling || progress?.closing) { samples.push(delay); sampleTimes.push([expected, now]); }
    if (!querying) {
      querying = true;
      const sent = performance.now();
      void measurementProgress().then((next) => {
        if (!active) return;
        const received = performance.now();
        clockExchange = next ? [sent, next.backend_clock_ms, received] : null;
        if (next?.row_id !== progress?.row_id || next?.sampling && !progress?.sampling) { samples = []; sampleTimes = []; previous = performance.now(); }
        progress = next;
      }).catch((error: unknown) => { failure = error; }).finally(() => { querying = false; });
    }
    if (!sending && progress && clockExchange && closedRow !== progress.row_id && (samples.length > 0 || progress.closing)) {
      sending = true;
      const captured = progress;
      const capturedSamples = samples;
      const capturedTimes = sampleTimes;
      samples = []; sampleTimes = [];
      void submitMeasurementProbe({ operation_id: captured.operation_id, row_id: captured.row_id, samples_seconds: capturedSamples, sample_times_ms: capturedTimes, clock_exchange_ms: clockExchange, finished: captured.closing })
        .then(() => { if (captured.closing) closedRow = captured.row_id; })
        .catch(async (error: unknown) => {
          const latest = await measurementProgress();
          if (latest?.row_id === captured.row_id) failure = error;
        })
        .catch((error: unknown) => { failure = error; })
        .finally(() => { sending = false; });
    }
    if (failure && active) {
      active = false;
      window.clearInterval(tick);
      void retireMeasuredReport().catch((error: unknown) => { failure = error; });
    }
  }, 10);
  try {
    const report = await measureApplication(request);
    if (failure) throw failure;
    return report;
  } finally {
    active = false;
    window.clearInterval(tick);
  }
}
