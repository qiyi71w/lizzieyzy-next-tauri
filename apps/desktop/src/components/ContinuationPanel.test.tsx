// @vitest-environment jsdom
import { act } from "react";
import { createRoot } from "react-dom/client";
import { beforeEach, expect, it, vi } from "vitest";
import { ContinuationPanel } from "./ContinuationPanel";
import { defaultContinuationPreferences, validateContinuationPreferences } from "../domain/continuation";
import * as api from "../api/continuation";
import * as preferences from "../api/preferences";
import { t } from "../i18n/resources";
vi.mock("../api/continuation", () => ({ continuationNativeAvailable: () => false, continuationBeat: vi.fn(), startContinuation: vi.fn(), stopContinuation: vi.fn() }));
vi.mock("../api/preferences", () => ({ loadContinuationPreferences: vi.fn(), saveContinuationPreferences: vi.fn() }));
beforeEach(() => { globalThis.IS_REACT_ACT_ENVIRONMENT = true; vi.clearAllMocks(); vi.mocked(preferences.loadContinuationPreferences).mockResolvedValue(defaultContinuationPreferences()); vi.mocked(preferences.saveContinuationPreferences).mockImplementation(async value => { validateContinuationPreferences(value); return value; }); });
it("requires finite exactly representable positive seconds", () => {
  expect(defaultContinuationPreferences()).toEqual({ mode: "leaf_only", intervalSeconds: 1 });
  for (const value of [0, -1, NaN, Infinity, 0.0001, 1.0011, 2147483.648]) expect(() => validateContinuationPreferences({ mode: "every_beat", intervalSeconds: value })).toThrow();
  for (const value of [0.001, 0.8, 1.001, 1.005, 1.25, 2147483.647]) expect(() => validateContinuationPreferences({ mode: "every_beat", intervalSeconds: value })).not.toThrow();
});
it("keeps Save, Cancel, failure and reopen stopped and restores focus", async () => {
  const host = document.createElement("div"); document.body.append(host); const root = createRoot(host);
  await act(async () => root.render(<ContinuationPanel current={null} choices={[]} onCurrent={vi.fn()} onStart={vi.fn()} />));
  const button = (key: Parameters<typeof t>[0]) => Array.from(host.querySelectorAll("button")).find(value => value.textContent === t(key))!;
  await act(async () => button("continuation.title").click());
  const select = host.querySelector("select")!;
  act(() => { select.value = "every_beat"; select.dispatchEvent(new Event("change", { bubbles: true })); });
  act(() => button("continuation.cancel").click()); expect(select.value).toBe("leaf_only");
  act(() => { select.value = "every_beat"; select.dispatchEvent(new Event("change", { bubbles: true })); });
  await act(async () => button("continuation.save").click());
  expect(preferences.saveContinuationPreferences).toHaveBeenCalledWith({ mode: "every_beat", intervalSeconds: 1 });
  vi.mocked(preferences.saveContinuationPreferences).mockRejectedValueOnce(new Error("atomic save failed"));
  await act(async () => button("continuation.save").click()); expect(host.querySelector('[role="alert"]')?.textContent).toContain("atomic save failed");
  await act(async () => button("continuation.close").click()); expect(document.activeElement).toBe(button("continuation.title"));
  await act(async () => button("continuation.title").click());
  expect(button("continuation.start").disabled).toBe(true); expect(api.startContinuation).not.toHaveBeenCalled(); expect(api.continuationBeat).not.toHaveBeenCalled();
  act(() => root.unmount()); host.remove();
});
