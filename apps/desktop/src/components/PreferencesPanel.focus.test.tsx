// @vitest-environment jsdom
import { act } from "react";
import { createRoot } from "react-dom/client";
import { expect, it, vi } from "vitest";
import { PreferencesPanel } from "./PreferencesPanel";
import { defaultAppPreferences } from "../domain/preferences";

it("keeps focus edits as drafts until explicit valid apply, retains failed writes and cancels without saving", () => {
  globalThis.IS_REACT_ACT_ENVIRONMENT = true;
  const host = document.createElement("div"); document.body.append(host);
  const root = createRoot(host);
  const onChange = vi.fn();
  const render = () => act(() => root.render(<PreferencesPanel preferences={defaultAppPreferences} status="" onChange={onChange}
    windowGeometryStatus="" windowGeometryDisabled={false} onResetWindowGeometry={vi.fn()} onRailVisibility={vi.fn()} />));
  render();
  const fieldset = [...host.querySelectorAll("fieldset")].find((field) => field.querySelector("legend")?.textContent === "同树重点分析")!;
  const visits = fieldset.querySelector<HTMLInputElement>('input[type="number"]')!;
  const set = (value: string) => act(() => {
    Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")!.set!.call(visits, value);
    visits.dispatchEvent(new Event("input", { bubbles: true }));
  });
  const button = (text: string) => [...fieldset.querySelectorAll("button")].find((item) => item.textContent === text)!;
  expect(visits.value).toBe("500");
  set("0");
  act(() => button("应用关注设置").click());
  expect(onChange).not.toHaveBeenCalled();
  expect(fieldset.querySelector('[role="alert"]')).not.toBeNull();
  set("937");
  expect(onChange).not.toHaveBeenCalled();
  act(() => button("应用关注设置").click());
  expect(onChange.mock.calls[0][0].localFocus.maxVisits).toBe(937);
  render(); // The committed value is unchanged after a failed outer save.
  expect(visits.value).toBe("937");
  act(() => button("取消关注设置").click());
  expect(visits.value).toBe("500");
  expect(onChange).toHaveBeenCalledTimes(1);
  act(() => root.unmount()); host.remove();
});
