// @vitest-environment jsdom
import { act, useState } from "react";
import { createRoot } from "react-dom/client";
import { expect, it, vi } from "vitest";
import { PreferencesPanel } from "./PreferencesPanel";
import { defaultAppPreferences } from "../domain/preferences";
import { loadAppPreferences, saveAppPreferences } from "../api/preferences";

it("applies exact positive centiseconds atomically, exposes saved versus actual and preserves failed/cancelled drafts", async () => {
  globalThis.IS_REACT_ACT_ENVIRONMENT = true;
  localStorage.clear();
  const host = document.createElement("div"); document.body.append(host);
  const root = createRoot(host);
  function Surface({ actual = 10 }: { actual?: number }) {
    const [preferences, setPreferences] = useState(defaultAppPreferences);
    const [status, setStatus] = useState("");
    return <PreferencesPanel preferences={preferences} status={status} actualRefreshInterval={actual}
      onChange={next => { void saveAppPreferences(next).then(setPreferences).catch(error => setStatus(String(error))); }}
      windowGeometryStatus="" windowGeometryDisabled={false} onResetWindowGeometry={() => {}} onRailVisibility={() => {}} />;
  }
  await act(async () => root.render(<Surface />));
  const fieldset = [...host.querySelectorAll("fieldset")].find(field => field.querySelector("legend")?.textContent === "本地分析结果刷新间隔")!;
  const input = fieldset.querySelector<HTMLInputElement>("input")!;
  const buttons = [...fieldset.querySelectorAll("button")];
  const apply = buttons.find(button => button.textContent === "应用刷新间隔")!;
  const cancel = buttons.find(button => button.textContent === "取消刷新间隔")!;
  const set = (value: string) => act(() => {
    Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")!.set!.call(input, value);
    input.dispatchEvent(new Event("input", { bubbles: true }));
  });
  expect(input.value).toBe("10");
  expect(fieldset.textContent).toContain("100 毫秒");
  for (const value of ["", "0", "-1", "1.5", "9007199254740991.1", "9007199254740992"]) {
    set(value);
    await act(async () => apply.click());
    expect(fieldset.querySelector('[role="alert"]')).not.toBeNull();
    expect((await loadAppPreferences()).preferences.analysisRefreshIntervalCentiseconds).toBe(10);
  }
  set("17");
  expect(fieldset.textContent).toContain("170 毫秒");
  await act(async () => cancel.click());
  expect(input.value).toBe("10");
  expect(localStorage.length).toBe(0);
  set("37");
  const failure = vi.spyOn(Storage.prototype, "setItem").mockImplementation(() => { throw new Error("disk full"); });
  await act(async () => apply.click());
  expect(host.textContent).toContain("disk full");
  expect(input.value).toBe("37");
  expect(fieldset.textContent).toContain("已保存：10 厘秒");
  failure.mockRestore();
  await act(async () => apply.click());
  expect((await loadAppPreferences()).preferences.analysisRefreshIntervalCentiseconds).toBe(37);
  expect(fieldset.textContent).toContain("已保存：37 厘秒");
  expect(fieldset.textContent).toContain("当前普通分析实际：10 厘秒");
  expect(fieldset.textContent).toContain("等待安全更新");
  await act(async () => root.render(<Surface actual={37} />));
  expect(fieldset.textContent).not.toContain("等待安全更新");
  set("9007199254740991");
  expect(fieldset.textContent).toContain("90071992547409910 毫秒");
  await act(async () => apply.click());
  expect((await loadAppPreferences()).preferences.analysisRefreshIntervalCentiseconds).toBe(Number.MAX_SAFE_INTEGER);
  await act(async () => root.unmount()); host.remove(); localStorage.clear();
});

it("rejects invalid API values without changing durable settings and recovers invalid stored intervals", async () => {
  localStorage.clear();
  const saved = await saveAppPreferences({ ...defaultAppPreferences, analysisRefreshIntervalCentiseconds: 23 });
  for (const value of [0, -1, 1.2, Infinity, NaN, Number.MAX_SAFE_INTEGER + 1]) {
    await expect(saveAppPreferences({ ...saved, analysisRefreshIntervalCentiseconds: value })).rejects.toThrow();
    expect((await loadAppPreferences()).preferences.analysisRefreshIntervalCentiseconds).toBe(23);
  }
  localStorage.setItem("lizzieyzy-next-app-preferences", JSON.stringify({ ...saved, analysisRefreshIntervalCentiseconds: 0 }));
  const recovered = await loadAppPreferences();
  expect(recovered.recovery).toBeTruthy();
  expect(recovered.preferences.analysisRefreshIntervalCentiseconds).toBe(10);
  localStorage.clear();
});
