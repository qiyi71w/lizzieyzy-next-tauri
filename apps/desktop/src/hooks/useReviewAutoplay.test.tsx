// @vitest-environment jsdom
import { act } from "react";
import { createRoot } from "react-dom/client";
import { expect, it, vi } from "vitest";
import { useReviewAutoplay } from "./useReviewAutoplay";

it("stops on document scope replacement and makes queued old ticks inert", () => {
  globalThis.IS_REACT_ACT_ENVIRONMENT = true;
  const host = document.createElement("div");
  const root = createRoot(host);
  const step = vi.fn();
  const stop = vi.fn();
  const callbacks: Array<() => void> = [];
  const interval = vi.spyOn(window, "setInterval").mockImplementation((callback) => {
    callbacks.push(callback as () => void);
    return callbacks.length;
  });
  const clear = vi.spyOn(window, "clearInterval");
  function Consumer({ scope, playing }: { scope: string; playing: boolean }) {
    useReviewAutoplay({ scope, playing, blocked: false, intervalMs: 800, step, stop });
    return null;
  }
  try {
    act(() => root.render(<Consumer scope="review:1" playing />));
    act(() => callbacks[0]());
    expect(step).toHaveBeenCalledTimes(1);
    act(() => root.render(<Consumer scope="review:2" playing />));
    expect(stop).toHaveBeenCalledTimes(1);
    act(() => callbacks[0]());
    expect(step).toHaveBeenCalledTimes(1);
    act(() => root.render(<Consumer scope="review:2" playing={false} />));
    expect(clear).toHaveBeenCalledWith(1);
    act(() => root.render(<Consumer scope="review:2" playing />));
    expect(interval).toHaveBeenCalledTimes(2);
    act(() => callbacks[0]());
    expect(step).toHaveBeenCalledTimes(1);
    act(() => callbacks[1]());
    expect(step).toHaveBeenCalledTimes(2);
  } finally {
    act(() => root.unmount());
    vi.restoreAllMocks();
  }
});
