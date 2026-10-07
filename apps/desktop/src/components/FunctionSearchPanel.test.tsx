// @vitest-environment jsdom
import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, expect, it, vi } from "vitest";
import { FunctionSearchPanel } from "./FunctionSearchPanel";
globalThis.IS_REACT_ACT_ENVIRONMENT = true;

const host = document.createElement("div");
document.body.append(host);
const root = createRoot(host);
afterEach(async () => { await act(async () => root.render(null)); });
it("renders Chinese resources and searches source English/pinyin without execution, then executes the available selection", async () => {
  const execute = vi.fn();
  const onExecute = vi.fn((action) => action.execute());
  const onCancel = vi.fn();
  await act(async () => root.render(<FunctionSearchPanel onCancel={onCancel} onExecute={onExecute} catalog={[
    { id: "prefs.candidate-limit", label: "target.prefs.candidate-limit", keywords: ["houxuan", "candidate"], execute },
    { id: "game.komi", label: "target.game.komi", keywords: ["komi"], disabledReason: "reason.desktop", execute }
  ]} />));
  expect(host.textContent).toContain("功能搜索");
  const input = host.querySelector("input")!;
  await act(async () => {
    Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")!.set!.call(input, "komi");
    input.dispatchEvent(new Event("input", { bubbles: true }));
  });
  expect(host.textContent).toContain("此功能需要 Tauri 桌面运行时。");
  await act(async () => input.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true })));
  expect(execute).not.toHaveBeenCalled();
  await act(async () => {
    Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")!.set!.call(input, "houxuan");
    input.dispatchEvent(new Event("input", { bubbles: true }));
    input.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }));
  });
  expect(onCancel).toHaveBeenCalledOnce();
  expect(execute).not.toHaveBeenCalled();
  await act(async () => input.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true })));
  expect(execute).toHaveBeenCalledOnce();
});

function enter(control: HTMLElement) {
  const allowed = control.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true, cancelable: true }));
  // Supply jsdom's missing button default only when keydown was not cancelled.
  if (allowed && control instanceof HTMLButtonElement) control.click();
}

it.each(["input", "result"])("Enter from %s executes exactly the selected result once", async (origin) => {
  const first = vi.fn();
  const second = vi.fn();
  const onExecute = vi.fn((action) => action.execute());
  const onCancel = vi.fn();
  await act(async () => root.render(<FunctionSearchPanel onCancel={onCancel} onExecute={onExecute} catalog={[
    { id: "game.black-name", label: "target.game.black-name", keywords: [], execute: first },
    { id: "game.white-name", label: "target.game.white-name", keywords: [], execute: second }
  ]} />));
  const input = host.querySelector("input")!;
  const result = host.querySelectorAll<HTMLButtonElement>('[role="option"]')[1];
  await act(async () => {
    input.focus();
    input.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowDown", bubbles: true }));
    if (origin === "result") result.focus();
  });
  expect(result.getAttribute("aria-selected")).toBe("true");
  await act(async () => enter(origin === "input" ? input : result));
  expect(first).not.toHaveBeenCalled();
  expect(second).toHaveBeenCalledOnce();
  expect(onExecute).toHaveBeenCalledOnce();
  expect(onExecute.mock.calls[0][0].id).toBe("game.white-name");
  expect(onCancel).not.toHaveBeenCalled();
});

it.each(["input", "result"])("Enter from %s preserves the disabled result guard", async (origin) => {
  const execute = vi.fn();
  const onExecute = vi.fn((action) => action.execute());
  await act(async () => root.render(<FunctionSearchPanel onCancel={vi.fn()} onExecute={onExecute} catalog={[
    { id: "game.komi", label: "target.game.komi", keywords: [], disabledReason: "reason.desktop", execute }
  ]} />));
  const result = host.querySelector<HTMLButtonElement>('[role="option"]')!;
  expect(result.getAttribute("aria-disabled")).toBe("true");
  expect(result.textContent).toContain("此功能需要 Tauri 桌面运行时。");
  await act(async () => enter(origin === "input" ? host.querySelector("input")! : result));
  expect(execute).not.toHaveBeenCalled();
  expect(onExecute).not.toHaveBeenCalled();
});

it("keeps the Tab loop and Close Enter, Escape and pointer cancellation independent of the selected result", async () => {
  const execute = vi.fn();
  const onCancel = vi.fn();
  await act(async () => root.render(<FunctionSearchPanel onCancel={onCancel} onExecute={(action) => action.execute()} catalog={[
    { id: "game.komi", label: "target.game.komi", keywords: [], execute }
  ]} />));
  const input = host.querySelector("input")!;
  const close = host.querySelectorAll<HTMLButtonElement>("button")[1];
  await act(async () => {
    input.focus();
    input.dispatchEvent(new KeyboardEvent("keydown", { key: "Tab", shiftKey: true, bubbles: true }));
  });
  expect(document.activeElement).toBe(close);
  await act(async () => close.dispatchEvent(new KeyboardEvent("keydown", { key: "Tab", bubbles: true })));
  expect(document.activeElement).toBe(input);
  await act(async () => enter(close));
  expect(onCancel).toHaveBeenCalledTimes(1);
  await act(async () => input.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true })));
  expect(onCancel).toHaveBeenCalledTimes(2);
  await act(async () => close.click());
  expect(onCancel).toHaveBeenCalledTimes(3);
  expect(execute).not.toHaveBeenCalled();
});
