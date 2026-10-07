import { useEffect, useRef } from "react";

type Options = {
  playing: boolean;
  scope: string;
  blocked: boolean;
  intervalMs: number;
  step: () => void;
  stop: () => void;
};

/** One ordinary review timer. Preferences are captured only on a stopped → playing transition. */
export function useReviewAutoplay(options: Options) {
  const latest = useRef(options);
  latest.current = options;
  const activeScope = useRef<string | null>(null);
  useEffect(() => {
    if (!options.playing) return;
    if (latest.current.blocked) { latest.current.stop(); return; }
    const scope = latest.current.scope;
    activeScope.current = scope;
    let active = true;
    const timer = window.setInterval(() => {
      const current = latest.current;
      if (!active || !current.playing) return;
      if (current.blocked || current.scope !== scope) { current.stop(); return; }
      current.step();
    }, latest.current.intervalMs);
    return () => { active = false; activeScope.current = null; window.clearInterval(timer); };
    // Capturing settings here, not on each render/gap, is the ordinary main-board contract.
  }, [options.playing]);
  useEffect(() => {
    if (options.playing && (options.blocked || (activeScope.current !== null && activeScope.current !== options.scope))) {
      options.stop();
    }
  }, [options.playing, options.blocked, options.scope]);
}
