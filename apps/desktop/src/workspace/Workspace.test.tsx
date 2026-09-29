// @vitest-environment jsdom
import { act, useState } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, expect, it, vi } from "vitest";
import { Workspace } from "./Workspace";
import { useWorkspace } from "./useWorkspace";
import type { WorkspaceShares } from "./projection";

globalThis.IS_REACT_ACT_ENVIRONMENT = true;
const cleanup: (() => void)[] = [];
afterEach(() => { cleanup.splice(0).forEach(fn => fn()); vi.restoreAllMocks(); });

it("resizes focused separators without navigating the game and leaves undo available", () => {
  vi.spyOn(HTMLElement.prototype, "clientWidth", "get").mockReturnValue(1200);
  const host = document.createElement("div");
  document.body.append(host);
  const root = createRoot(host);
  cleanup.push(() => { act(() => root.unmount()); host.remove(); });
  const navigate = vi.fn();
  document.addEventListener("keydown", navigate);
  cleanup.push(() => document.removeEventListener("keydown", navigate));
  function Harness() {
    const [shares, setShares] = useState<WorkspaceShares | null>(null);
    return <Workspace shares={shares} onSharesChange={setShares} visibility={{ left: true, right: true }}>
      <aside className="rail" /><div className="diagram" /><aside className="sheet-col" />
    </Workspace>;
  }
  act(() => root.render(<Harness />));
  const separator = host.querySelector('[role="separator"]')!;
  const key = (value: string, options: KeyboardEventInit = {}) => act(() => {
    separator.dispatchEvent(new KeyboardEvent("keydown", { key: value, bubbles: true, cancelable: true, ...options }));
  });
  key("ArrowRight");
  expect(separator.getAttribute("aria-valuenow")).toBe("236");
  key("ArrowRight", { shiftKey: true });
  expect(separator.getAttribute("aria-valuenow")).toBe("268");
  expect(navigate).not.toHaveBeenCalled();
  key("z", { ctrlKey: true });
  expect(navigate).toHaveBeenCalledTimes(1);
});

it("settles active dragging and capture when the dragged separator disappears and leaves remaining and restored separators usable", () => {
  vi.spyOn(HTMLElement.prototype, "clientWidth", "get").mockReturnValue(1200);
  const captured = new Map<HTMLElement, Set<number>>();
  const setPointerCapture = vi.fn(function (this: HTMLElement, id: number) {
    let set = captured.get(this);
    if (!set) {
      set = new Set();
      captured.set(this, set);
    }
    set.add(id);
  });
  const releasePointerCapture = vi.fn(function (this: HTMLElement, id: number) {
    captured.get(this)?.delete(id);
  });
  const hasPointerCapture = vi.fn(function (this: HTMLElement, id: number) {
    return Boolean(captured.get(this)?.has(id));
  });
  type PointerCaptureHost = {
    setPointerCapture?: (id: number) => void;
    releasePointerCapture?: (id: number) => void;
    hasPointerCapture?: (id: number) => boolean;
  };
  const captureHost = HTMLElement.prototype as unknown as PointerCaptureHost;
  captureHost.setPointerCapture = setPointerCapture;
  captureHost.releasePointerCapture = releasePointerCapture;
  captureHost.hasPointerCapture = hasPointerCapture;
  cleanup.push(() => {
    delete captureHost.setPointerCapture;
    delete captureHost.releasePointerCapture;
    delete captureHost.hasPointerCapture;
  });

  const host = document.createElement("div");
  document.body.append(host);
  const root = createRoot(host);
  cleanup.push(() => { act(() => root.unmount()); host.remove(); });

  function pointerEvent(type: string, init: MouseEventInit & { pointerId?: number } = {}) {
    if (typeof PointerEvent !== "undefined") {
      return new PointerEvent(type, init);
    }
    const event = new MouseEvent(type, init);
    Object.defineProperty(event, "pointerId", { value: init.pointerId ?? 0 });
    return event;
  }

  function Harness({ visibility }: { visibility: { left: boolean; right: boolean } }) {
    const workspace = useWorkspace();
    return (
      <Workspace
        shares={workspace.shares}
        onSharesChange={workspace.setShares}
        visibility={visibility}
      >
        <aside className="rail" />
        <div className="diagram" />
        <aside className="sheet-col" />
      </Workspace>
    );
  }

  act(() => root.render(<Harness visibility={{ left: true, right: true }} />));

  const scrollContainer = host.querySelector<HTMLElement>(".workspace-scroll")!;
  const leftSeparator = host.querySelector<HTMLElement>(".workspace-separator-left")!;
  expect(leftSeparator).not.toBeNull();
  expect(scrollContainer.classList.contains("workspace-dragging")).toBe(false);

  // Start dragging left separator
  act(() => {
    leftSeparator.dispatchEvent(
      pointerEvent("pointerdown", { bubbles: true, cancelable: true, button: 0, pointerId: 1, clientX: 200 })
    );
  });
  expect(scrollContainer.classList.contains("workspace-dragging")).toBe(true);
  expect(setPointerCapture).toHaveBeenCalledWith(1);

  // Move left separator to resize and commit valid shares
  act(() => {
    leftSeparator.dispatchEvent(
      pointerEvent("pointermove", { bubbles: true, cancelable: true, pointerId: 1, clientX: 250 })
    );
  });

  // Now left separator disappears while dragging
  act(() => root.render(<Harness visibility={{ left: false, right: true }} />));

  // Left separator is removed from DOM
  expect(host.querySelector(".workspace-separator-left")).toBeNull();
  // Dragging state and class are cleared
  expect(scrollContainer.classList.contains("workspace-dragging")).toBe(false);
  // Pointer capture on the removed separator was released
  expect(releasePointerCapture).toHaveBeenCalledWith(1);

  // Clicks on workspace are not suppressed
  const clickEvent = new MouseEvent("click", { bubbles: true, cancelable: true });
  const dispatched = scrollContainer.dispatchEvent(clickEvent);
  expect(dispatched).toBe(true);
  expect(clickEvent.defaultPrevented).toBe(false);

  // Remaining right separator is usable for dragging
  const rightSeparator = host.querySelector<HTMLElement>(".workspace-separator-right")!;
  expect(rightSeparator).not.toBeNull();
  act(() => {
    rightSeparator.dispatchEvent(
      pointerEvent("pointerdown", { bubbles: true, cancelable: true, button: 0, pointerId: 2, clientX: 900 })
    );
  });
  expect(scrollContainer.classList.contains("workspace-dragging")).toBe(true);
  act(() => {
    rightSeparator.dispatchEvent(
      pointerEvent("pointerup", { bubbles: true, cancelable: true, pointerId: 2 })
    );
  });
  expect(scrollContainer.classList.contains("workspace-dragging")).toBe(false);

  // Restoring left separator renders it, preserves last valid shares, and is usable
  act(() => root.render(<Harness visibility={{ left: true, right: true }} />));
  const restoredLeft = host.querySelector<HTMLElement>(".workspace-separator-left")!;
  expect(restoredLeft).not.toBeNull();
  expect(restoredLeft.getAttribute("aria-valuenow")).not.toBeNull();
  act(() => {
    restoredLeft.dispatchEvent(
      pointerEvent("pointerdown", { bubbles: true, cancelable: true, button: 0, pointerId: 3, clientX: 250 })
    );
  });
  expect(scrollContainer.classList.contains("workspace-dragging")).toBe(true);
  act(() => {
    restoredLeft.dispatchEvent(
      pointerEvent("pointerup", { bubbles: true, cancelable: true, pointerId: 3 })
    );
  });
  expect(scrollContainer.classList.contains("workspace-dragging")).toBe(false);
});
