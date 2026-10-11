import type { ContinuationPreferencesDto } from "./types";

export const defaultContinuationPreferences = (): ContinuationPreferencesDto => ({ mode: "leaf_only", intervalSeconds: 1 });
export function validateContinuationPreferences(value: ContinuationPreferencesDto): void {
  const milliseconds = value.intervalSeconds * 1000;
  if (!["leaf_only", "every_beat"].includes(value.mode) || !Number.isFinite(milliseconds)
    || !Number.isInteger(milliseconds) || milliseconds < 1 || milliseconds > 2147483647
    || milliseconds / 1000 !== value.intervalSeconds) {
    throw new Error("Continuation interval must be positive, finite, and exactly representable in whole milliseconds.");
  }
}
