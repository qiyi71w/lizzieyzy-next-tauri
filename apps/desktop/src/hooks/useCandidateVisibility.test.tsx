// @vitest-environment jsdom
import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { useCandidateVisibility } from "./useCandidateVisibility";
import { defaultGranularDisplay, type GranularDisplay } from "../domain/granularDisplay";
import { createShortcutRegistry } from "../domain/shortcuts";
let root: Root;
let host: HTMLDivElement;
function Probe({ identity, eligible, preferences, visits = 1 }: { identity: string; eligible: boolean; preferences: GranularDisplay; visits?: number }) {
  const state = useCandidateVisibility(identity, eligible, preferences);
  const registry = createShortcutRegistry();
  registry.bind("view.candidates", state.toggle);
  return <div onKeyDown={event => registry.dispatch(event.nativeEvent)}><button disabled={!state.eligible} onClick={state.toggle}>{state.visible ? "shown" : "hidden"}</button><input type="number" aria-label="number" /><span>{visits}</span></div>;
}
beforeEach(() => { globalThis.IS_REACT_ACT_ENVIRONMENT = true; vi.useFakeTimers(); host = document.createElement("div"); document.body.append(host); root = createRoot(host); });
afterEach(() => { act(() => root.unmount()); host.remove(); vi.useRealTimers(); });
it("keeps the original timed deadline across statistics and cancels it after explicit reveal then hide", () => {
  const preferences = { ...defaultGranularDisplay, delayMode: "timed" as const };
  act(() => root.render(<Probe identity="doc/node/job" eligible preferences={preferences} />));
  expect(host.textContent).toContain("hidden");
  act(() => vi.advanceTimersByTime(9000));
  act(() => root.render(<Probe identity="doc/node/job" eligible preferences={preferences} visits={200} />));
  act(() => vi.advanceTimersByTime(1000));
  expect(host.textContent).toContain("shown");
  act(() => root.render(<Probe identity="doc/next/job" eligible preferences={preferences} />));
  expect(host.textContent).toContain("hidden");
  act(() => host.querySelector("button")!.click());
  expect(host.textContent).toContain("shown");
  act(() => host.querySelector("button")!.click());
  act(() => vi.advanceTimersByTime(20000));
  expect(host.textContent).toContain("hidden");
});
it("retires invalid scopes and applies Off, custom Timed and Manual with focus-safe F", () => {
  let preferences = { ...defaultGranularDisplay };
  act(() => root.render(<Probe identity="a" eligible preferences={preferences} />));
  expect(host.textContent).toContain("shown");
  preferences = { ...preferences, delayMode: "timed", delaySeconds: 0.125 };
  act(() => root.render(<Probe identity="a" eligible preferences={preferences} />));
  act(() => vi.advanceTimersByTime(124));
  expect(host.textContent).toContain("hidden");
  act(() => root.render(<Probe identity="b" eligible={false} preferences={preferences} />));
  act(() => vi.advanceTimersByTime(1000));
  expect(host.textContent).toContain("hidden");
  preferences = { ...preferences, delayMode: "manual" };
  act(() => root.render(<Probe identity="b" eligible preferences={preferences} />));
  const input = host.querySelector("input")!;
  act(() => input.dispatchEvent(new KeyboardEvent("keydown", { key: "f", bubbles: true })));
  expect(host.textContent).toContain("hidden");
  const button = host.querySelector("button")!;
  for (const extra of [{ isComposing: true }, { ctrlKey: true }, { shiftKey: true }]) {
    act(() => button.dispatchEvent(new KeyboardEvent("keydown", { key: "f", bubbles: true, ...extra })));
    expect(host.textContent).toContain("hidden");
  }
  act(() => button.dispatchEvent(new KeyboardEvent("keydown", { key: "f", bubbles: true })));
  expect(host.textContent).toContain("shown");
  act(() => button.dispatchEvent(new KeyboardEvent("keydown", { key: "f", bubbles: true })));
  expect(host.textContent).toContain("hidden");
  expect(preferences.showBlackCandidates).toBe(true);
  expect(preferences.showWhiteCandidates).toBe(true);
});
