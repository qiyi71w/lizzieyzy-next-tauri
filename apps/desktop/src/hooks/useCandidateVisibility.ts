import { useLayoutEffect, useRef, useState } from "react";
import type { GranularDisplay } from "../domain/granularDisplay";

/** A presentation range owns exactly one deadline; statistics never renew it. */
export function useCandidateVisibility(identity: string | null, eligible: boolean, preferences: GranularDisplay) {
  const key = eligible && identity !== null ? JSON.stringify([identity, preferences.delayMode, preferences.delaySeconds]) : null;
  const currentKey = useRef(key);
  currentKey.current = key;
  const timer = useRef<number | undefined>(undefined);
  const revision = useRef(0);
  const [override, setOverride] = useState<{ key: string; visible: boolean } | null>(null);
  const visible = key !== null && (override?.key === key ? override.visible : preferences.delayMode === "off");
  useLayoutEffect(() => {
    const epoch = ++revision.current;
    setOverride(null);
    if (key !== null && preferences.delayMode === "timed") {
      timer.current = window.setTimeout(() => {
        if (currentKey.current === key && revision.current === epoch) setOverride({ key, visible: true });
      }, Math.round(preferences.delaySeconds * 1000));
    }
    return () => {
      ++revision.current;
      clearTimeout(timer.current);
      timer.current = undefined;
    };
  }, [key]);
  function toggle() {
    if (key === null) return;
    ++revision.current;
    clearTimeout(timer.current);
    timer.current = undefined;
    setOverride({ key, visible: !visible });
  }
  return { visible, toggle, eligible: key !== null };
}
