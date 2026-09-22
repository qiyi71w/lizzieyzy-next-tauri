import captureLargeUrl from "../assets/sounds/capture-large.wav";
import captureSmallUrl from "../assets/sounds/capture-small.wav";
import moveUrl from "../assets/sounds/move.wav";
import passUrl from "../assets/sounds/pass.wav";

export type MoveSoundKind = "move" | "capture-small" | "capture-large" | "pass";

// These original assets were generated for this project with deterministic synthesis:
// mono 44.1 kHz PCM using short decaying resonances/noise, with no external samples.
const moveSoundUrls: Record<MoveSoundKind, string> = {
  move: moveUrl,
  "capture-small": captureSmallUrl,
  "capture-large": captureLargeUrl,
  pass: passUrl
};

export async function playMoveSound(kind: MoveSoundKind): Promise<void> {
  const source = moveSoundUrls[kind];

  try {
    const audio = new Audio(source);
    audio.preload = "auto";
    await audio.play();
  } catch (cause) {
    const detail = cause instanceof Error ? cause.message : String(cause);
    throw new Error(`Failed to play ${kind} sound from bundled resource ${source}: ${detail}`, { cause });
  }
}
