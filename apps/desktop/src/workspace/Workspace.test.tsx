// @vitest-environment jsdom
import { act, useState } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, expect, it, vi } from "vitest";
import { Workspace } from "./Workspace";
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
