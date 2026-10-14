// @vitest-environment jsdom
import { act, useState } from "react";
import { createRoot } from "react-dom/client";
import { expect, it, vi } from "vitest";
import { PreferencesPanel } from "./PreferencesPanel";
import { defaultAppPreferences } from "../domain/preferences";
import { loadAppPreferences, saveAppPreferences } from "../api/preferences";
import { t } from "../i18n/resources";
import { computeRawNet } from "../api/rawNet";
vi.mock("../api/rawNet", () => ({ computeRawNet: vi.fn() }));

it("applies five independent raw styles without computing and cancels unsaved display drafts", async () => {
  globalThis.IS_REACT_ACT_ENVIRONMENT = true;
  localStorage.clear();
  const host = document.createElement("div"); document.body.append(host);
  const root = createRoot(host);
  const save = vi.fn(saveAppPreferences);
  function Surface() {
    const [preferences, setPreferences] = useState(defaultAppPreferences);
    return <PreferencesPanel preferences={preferences} status="" onChange={next => { void save(next).then(setPreferences); }}
      windowGeometryStatus="" windowGeometryDisabled={false} onResetWindowGeometry={() => {}} onRailVisibility={() => {}} />;
  }
  await act(async () => root.render(<Surface />));
  const select = host.querySelector<HTMLSelectElement>(`select[aria-label="${t("display.rawStyle")}"]`)!;
  const ordinary = host.querySelector<HTMLSelectElement>(`select[aria-label="${t("display.ownershipStyle")}"]`)!;
  const apply = host.querySelector<HTMLButtonElement>('button[aria-label="应用显示设置"]')!;
  const cancel = host.querySelector<HTMLButtonElement>('button[aria-label="取消显示设置"]')!;
  const change = (value: string) => act(() => { select.value = value; select.dispatchEvent(new Event("change", { bubbles: true })); });
  expect(select.value).toBe("bigBelow");
  expect([...select.options].map(option => option.value)).toEqual(["small", "normal", "notOnLive", "bigBelow", "bySize"]);
  change("small");
  await act(async () => cancel.click());
  expect(select.value).toBe("bigBelow");
  expect(save).not.toHaveBeenCalled();
  for (const style of ["small", "normal", "notOnLive", "bigBelow", "bySize"]) {
    change(style);
    await act(async () => apply.click());
    const persisted = (await loadAppPreferences()).preferences.granularDisplay;
    expect(persisted.rawNetOwnershipStyle).toBe(style);
    expect(persisted.ownershipStyle).toBe(defaultAppPreferences.granularDisplay.ownershipStyle);
    expect(ordinary.value).toBe(defaultAppPreferences.granularDisplay.ownershipStyle);
  }
  expect(save).toHaveBeenCalledTimes(5);
  expect(computeRawNet).not.toHaveBeenCalled();
  await act(async () => root.unmount()); host.remove(); localStorage.clear();
});
