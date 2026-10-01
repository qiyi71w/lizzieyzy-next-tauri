// @vitest-environment jsdom
import { act } from "react";
import { createRoot } from "react-dom/client";
import { expect, it, vi } from "vitest";
import { useMainWindowPin } from "./useMainWindowPin";
import { WindowPinControl } from "../components/WindowPinControl";
import type { MainWindowPinStatusDto } from "../domain/types";
const api = vi.hoisted(() => ({ loadMainWindowPin: vi.fn(), setMainWindowPin: vi.fn() }));
vi.mock("../api/mainWindowPin", () => api);
function Surface() {
  const control = useMainWindowPin(true, true, false);
  return <><WindowPinControl control={control} /><WindowPinControl control={control} /></>;
}
it("keeps successful retry state when an older startup status arrives", async () => {
  let finishLoad!: (value: MainWindowPinStatusDto) => void;
  api.loadMainWindowPin.mockReturnValue(new Promise<MainWindowPinStatusDto>(resolve => { finishLoad = resolve; }));
  api.setMainWindowPin.mockResolvedValue({ actual: true, durable: true, error: null });
  const container = document.createElement("div");
  document.body.append(container);
  const root = createRoot(container);
  try {
    await act(async () => root.render(<Surface />));
    const retry = Array.from(container.querySelectorAll("button")).find(button => button.textContent === "重试已保存的置顶意图")!;
    await act(async () => retry.click());
    await act(async () => finishLoad({ actual: false, durable: true, error: "startup failure" }));
    expect(Array.from(container.querySelectorAll('[role="checkbox"]')).map(control => control.getAttribute("aria-checked"))).toEqual(["true", "true"]);
    expect(container.textContent).not.toContain("startup failure");
    expect(container.textContent).not.toContain("重试已保存的置顶意图");
  } finally {
    await act(async () => root.unmount());
    container.remove();
  }
});
