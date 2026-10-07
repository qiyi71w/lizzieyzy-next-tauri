// @vitest-environment jsdom
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { captureFocusReturn, restoreOwnedFocus, scheduleOwnedFocus } from "./focusNavigation";

let callbacks: Map<number, FrameRequestCallback>;
let focused: boolean;
beforeEach(() => {
  callbacks = new Map();
  let serial = 0;
  focused = true;
  vi.stubGlobal("requestAnimationFrame", (callback: FrameRequestCallback) => { callbacks.set(++serial, callback); return serial; });
  vi.stubGlobal("cancelAnimationFrame", (id: number) => { callbacks.delete(id); });
  vi.spyOn(document, "hasFocus").mockImplementation(() => focused);
  vi.spyOn(HTMLElement.prototype, "getClientRects").mockReturnValue([{ width: 10, height: 10 }] as unknown as DOMRectList);
  document.body.innerHTML = '<main tabindex="-1"><button data-focus-anchor>workspace</button><section data-focus-owner="prefs" tabindex="-1"><input id="first"><input id="target"></section></main>';
});
afterEach(() => { document.body.replaceChildren(); vi.restoreAllMocks(); vi.unstubAllGlobals(); });
function frame() {
  const pending = [...callbacks.values()];
  callbacks.clear();
  pending.forEach((callback) => callback(0));
}

describe("activation-fenced focus navigation", () => {
  it("waits for activation and focuses the exact non-first target", () => {
    focused = false;
    const owner = document.querySelector<HTMLElement>("section")!;
    scheduleOwnedFocus(owner, () => document.querySelector("#target"));
    frame();
    expect(document.activeElement?.id).not.toBe("target");
    focused = true;
    window.dispatchEvent(new Event("focus"));
    frame();
    expect(document.activeElement?.id).toBe("target");
  });
  it("cancels late activation after disposal or a newer owner and never steals from a new dialog", () => {
    focused = false;
    const oldOwner = document.querySelector<HTMLElement>("section")!;
    const cancel = scheduleOwnedFocus(oldOwner, () => document.querySelector("#first"));
    frame();
    cancel();
    const dialog = document.createElement("section");
    dialog.setAttribute("role", "dialog");
    dialog.innerHTML = '<input id="new-target">';
    document.body.append(dialog);
    scheduleOwnedFocus(dialog, () => dialog.querySelector("input"));
    focused = true;
    window.dispatchEvent(new Event("focus"));
    frame();
    expect(document.activeElement?.id).toBe("new-target");
    scheduleOwnedFocus(oldOwner, () => document.querySelector("#first"));
    frame();
    expect(document.activeElement?.id).toBe("new-target");
  });
  it("restores the original input, then valid owner anchor/workspace, but never a destroyed owner", () => {
    const main = document.querySelector<HTMLElement>("main")!;
    const input = document.querySelector<HTMLInputElement>("#target")!;
    input.focus();
    const source = captureFocusReturn(main);
    document.querySelector<HTMLInputElement>("#first")!.focus();
    restoreOwnedFocus(source); frame();
    expect(document.activeElement).toBe(input);
    input.disabled = true;
    restoreOwnedFocus(source); frame();
    expect(document.activeElement).toBe(source.owner);
    source.owner.hidden = true;
    restoreOwnedFocus(source); frame();
    expect(document.activeElement).toBe(main);
    source.owner.remove();
    const anchor = main.querySelector<HTMLButtonElement>("button")!;
    anchor.focus();
    restoreOwnedFocus(source); frame();
    expect(document.activeElement).toBe(anchor);
  });
  it("reports unavailable targets instead of focusing the first input", () => {
    const owner = document.querySelector<HTMLElement>("section")!;
    const unavailable = vi.fn();
    scheduleOwnedFocus(owner, () => owner.querySelector("#unsupported"), unavailable);
    frame();
    expect(unavailable).toHaveBeenCalledOnce();
    expect(document.activeElement?.id).not.toBe("first");
  });
});
