/// <reference lib="es2024.promise" />
// @vitest-environment jsdom
import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { getBoardGestureTiming } from "../api/backend";
import type { BoardGestureTimingDto } from "../domain/types";
import { useBoardClickClassification, type BoardClickClassificationControl, type BoardClickClassificationOptions } from "./useBoardClickClassification";

vi.mock("../api/backend", () => ({ getBoardGestureTiming: vi.fn() }));
let root: Root;
let control: BoardClickClassificationControl;
let options: BoardClickClassificationOptions;
const point = { x: 0, y: 0 };
function Consumer() {
  control = useBoardClickClassification(options);
  return null;
}
beforeEach(() => {
  globalThis.IS_REACT_ACT_ENVIRONMENT = true;
  vi.useFakeTimers({ toFake: ["setTimeout", "clearTimeout", "performance"] });
  vi.mocked(getBoardGestureTiming).mockReset().mockResolvedValue({ double_click_interval_ms: 1250 });
  options = { enabled: true, allowDoubleClick: true, scope: { generation: 4, selectedPath: [2], requestToken: "current" }, single: vi.fn(), double: vi.fn(), refuse: vi.fn() };
  root = createRoot(document.createElement("div"));
  act(() => root.render(<Consumer />));
});
afterEach(() => {
  act(() => root.unmount());
  vi.restoreAllMocks();
  vi.useRealTimers();
});
async function firstClick() {
  act(() => control.click(point, 1));
  await act(async () => { await Promise.resolve(); });
}

describe("native board click classification", () => {
  it("accepts a slow system-permitted double click without an earlier speculative edit", async () => {
    await firstClick();
    act(() => vi.advanceTimersByTime(1000));
    expect(options.single).not.toHaveBeenCalled();
    act(() => { control.pointerDown(); control.click(point, 2); });
    expect(options.double).toHaveBeenCalledExactlyOnceWith(point, options.scope);
    act(() => vi.advanceTimersByTime(2000));
    expect(options.single).not.toHaveBeenCalled();
    expect(getBoardGestureTiming).toHaveBeenCalledTimes(1);
  });
  it("holds first-click retirement through a long second press until click detail classifies it", async () => {
    await firstClick();
    act(() => { vi.advanceTimersByTime(1000); control.pointerDown(); vi.advanceTimersByTime(3000); });
    expect(options.single).not.toHaveBeenCalled();
    act(() => control.click(point, 2));
    expect(options.double).toHaveBeenCalledExactlyOnceWith(point, options.scope);
    expect(options.single).not.toHaveBeenCalled();
  });
  it("commits a real single click exactly once after the current native interval", async () => {
    await firstClick();
    act(() => vi.advanceTimersByTime(1249));
    expect(options.single).not.toHaveBeenCalled();
    act(() => vi.advanceTimersByTime(1));
    expect(options.single).toHaveBeenCalledExactlyOnceWith(point);
    act(() => vi.advanceTimersByTime(3000));
    expect(options.single).toHaveBeenCalledTimes(1);
    expect(options.double).not.toHaveBeenCalled();
  });
  it("ignores a native read that finishes after a newer first-click gesture supersedes it", async () => {
    const { promise, resolve: finish } = Promise.withResolvers<BoardGestureTimingDto>();
    vi.mocked(getBoardGestureTiming).mockReturnValueOnce(promise);
    await firstClick();
    act(() => control.cancel());
    vi.mocked(getBoardGestureTiming).mockResolvedValueOnce({ double_click_interval_ms: 750 });
    await firstClick();
    await act(async () => { finish({ double_click_interval_ms: 1250 }); await Promise.resolve(); });
    act(() => vi.advanceTimersByTime(750));
    expect(options.single).toHaveBeenCalledExactlyOnceWith(point);
    expect(options.refuse).not.toHaveBeenCalled();
  });
  it.each(["generation", "path", "request", "mode", "permission"])("retires a pending click on %s owner change", async (change) => {
    await firstClick();
    if (change === "generation") options = { ...options, scope: { ...options.scope!, generation: 5 } };
    if (change === "path") options = { ...options, scope: { ...options.scope!, selectedPath: [0] } };
    if (change === "request") options = { ...options, scope: { ...options.scope!, requestToken: "new" } };
    if (change === "mode") options = { ...options, enabled: false };
    if (change === "permission") options = { ...options, allowDoubleClick: false };
    act(() => root.render(<Consumer />));
    act(() => vi.advanceTimersByTime(3000));
    expect(options.single).not.toHaveBeenCalled();
    expect(options.double).not.toHaveBeenCalled();
  });
  it("cancels a held pointer gesture without leaving the next single click stuck", async () => {
    await firstClick();
    act(() => { control.pointerDown(); control.cancel(); vi.advanceTimersByTime(2000); });
    expect(options.single).not.toHaveBeenCalled();
    await firstClick();
    act(() => vi.advanceTimersByTime(1250));
    expect(options.single).toHaveBeenCalledExactlyOnceWith(point);
  });
  it("does not cancel a newer timer when a retired timer callback is invoked", async () => {
    const timers = vi.spyOn(window, "setTimeout");
    await firstClick();
    const retired = timers.mock.calls.at(-1)![0];
    act(() => control.cancel());
    await firstClick();
    if (typeof retired !== "function") throw new Error("Expected a timer callback");
    act(() => retired());
    act(() => vi.advanceTimersByTime(1250));
    expect(options.single).toHaveBeenCalledExactlyOnceWith(point);
  });
  it("refuses the ambiguous gesture visibly on read failure but leaves immediate modes intact", async () => {
    vi.mocked(getBoardGestureTiming).mockRejectedValueOnce(new Error("Native read failed"));
    await firstClick();
    expect(options.refuse).toHaveBeenCalledExactlyOnceWith("Native read failed");
    expect(options.single).not.toHaveBeenCalled();
    options = { ...options, enabled: false };
    act(() => root.render(<Consumer />));
    act(() => { control.click(point, 1); control.click(point, 2); });
    expect(options.single).toHaveBeenCalledTimes(2);
    expect(getBoardGestureTiming).toHaveBeenCalledTimes(1);
  });
  it("preserves disabled-double-click first-click and keyboard immediacy without querying timing", () => {
    options = { ...options, allowDoubleClick: false };
    act(() => root.render(<Consumer />));
    act(() => { control.click(point, 1); control.click(point, 2); control.click(point, 0); });
    expect(options.single).toHaveBeenCalledTimes(2);
    expect(getBoardGestureTiming).not.toHaveBeenCalled();
  });
  it("retires an outstanding native read on unmount", async () => {
    const { promise, resolve: finish } = Promise.withResolvers<BoardGestureTimingDto>();
    vi.mocked(getBoardGestureTiming).mockReturnValueOnce(promise);
    await firstClick();
    act(() => root.unmount());
    await act(async () => { finish({ double_click_interval_ms: 1250 }); await Promise.resolve(); });
    act(() => vi.advanceTimersByTime(3000));
    expect(options.single).not.toHaveBeenCalled();
    expect(options.refuse).not.toHaveBeenCalled();
  });
});
