// @vitest-environment jsdom
import { describe, expect, it, vi } from "vitest";
import { captureRenderedSurface, chosenLeaf } from "./export";
import { createShortcutRegistry } from "../domain/shortcuts";
import type { SgfTreeNodeDto } from "../domain/types";

describe("invocation-time export", () => {
  it("freezes the actual rectangular rendered pixels before later drawing", () => {
    const canvas = document.createElement("canvas");
    canvas.width = 3; canvas.height = 2;
    const pixels = new Uint8ClampedArray(24).fill(25);
    vi.spyOn(canvas, "getContext").mockReturnValue({ getImageData: () => ({ data: pixels }) } as unknown as CanvasRenderingContext2D);
    const captured = captureRenderedSurface(canvas);
    pixels.fill(90); canvas.width = 9;
    expect(captured).toEqual({ width: 3, height: 2, rgba: Array(24).fill(25) });
    expect(() => captureRenderedSurface(null)).toThrow();
  });
  it("keeps selected ancestry and only the remembered continuation", () => {
    const leaf = (): SgfTreeNodeDto => ({ properties: [], children: [] });
    const root = { properties: [], children: [{ properties: [], children: [leaf(), { properties: [], children: [leaf(), leaf()] }] }, leaf()] };
    const chosen = new Map([["0", 1], ["0,1", 1]]);
    expect(chosenLeaf(root, { indices: [0] }, chosen)).toEqual({ indices: [0, 1, 1] });
    expect(chosenLeaf(root, { indices: [1] }, chosen)).toEqual({ indices: [1] });
    expect(chosenLeaf(root, { indices: [] }, chosen)).toEqual({ indices: [0, 1, 1] });
    expect(root.children).toHaveLength(2);
  });
  it("routes mainboard and branch through independent focus-safe registry actions", () => {
    const registry = createShortcutRegistry();
    const board = vi.fn(); const branch = vi.fn();
    registry.bind("file.export-board", board); registry.bind("file.export-branch", branch);
    registry.dispatch(new KeyboardEvent("keydown", { key: "s", altKey: true }));
    registry.dispatch(new KeyboardEvent("keydown", { key: "s", altKey: true, ctrlKey: true }));
    expect(board).toHaveBeenCalledTimes(1); expect(branch).toHaveBeenCalledTimes(1);
    const input = document.createElement("input");
    input.addEventListener("keydown", event => registry.dispatch(event));
    input.dispatchEvent(new KeyboardEvent("keydown", { key: "s", altKey: true }));
    expect(board).toHaveBeenCalledTimes(1);
    expect(registry.dispatch(new KeyboardEvent("keydown", { key: "s", shiftKey: true }))).toBe(false);
  });
});
