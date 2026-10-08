// The WebView scheduler truncates fractional milliseconds and overflows signed 32-bit delays.
export const MAX_REVIEW_AUTOPLAY_INTERVAL_MS = 2_147_483_647;

export function validReviewAutoplayInterval(value: unknown): value is number {
  return typeof value === "number" && Number.isSafeInteger(value)
    && value > 0 && value <= MAX_REVIEW_AUTOPLAY_INTERVAL_MS;
}

/** Seconds are editable; whole milliseconds are the lossless durable/scheduler representation. */
export function parseReviewAutoplaySeconds(text: string): number | null {
  if (text.trim() === "") return null;
  const seconds = Number(text);
  const milliseconds = Math.round(seconds * 1000);
  return Number.isFinite(seconds) && validReviewAutoplayInterval(milliseconds)
    && milliseconds / 1000 === seconds ? milliseconds : null;
}
