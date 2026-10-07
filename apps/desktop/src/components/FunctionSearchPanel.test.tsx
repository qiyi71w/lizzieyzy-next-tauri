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
