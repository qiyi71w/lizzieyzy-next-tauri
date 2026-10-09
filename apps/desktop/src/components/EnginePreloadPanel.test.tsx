// @vitest-environment jsdom
import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, expect, it, vi } from "vitest";
import { EnginePreloadPanel } from "./EnginePreloadPanel";
import * as backend from "../api/backend";
import * as api from "../api/enginePreload";
import type { EnginePreloadDto, EngineProfileRecordDto } from "../domain/types";

const profile: EngineProfileRecordDto = { id: "b", preload: true, profile: {
  name: "Background B", program: "katago", argv: [], working_dir: null,
  adapter_kind: "generic_gtp", settings: {}
} };
afterEach(() => { vi.restoreAllMocks(); document.body.replaceChildren(); });

it("prepares and cancels the named saved profile without a foreground action", async () => {
  globalThis.IS_REACT_ACT_ENVIRONMENT = true;
  vi.spyOn(backend, "isTauriRuntime").mockReturnValue(true);
  let slots: EnginePreloadDto[] = [];
  vi.spyOn(api, "enginePreloadSnapshot").mockImplementation(async () => slots);
  const prepare = vi.spyOn(api, "prepareEnginePreload").mockImplementation(async () => {
    slots = [{ phase: "preparing", failure: null, run: { run_id: "prepared-b", profile_id: "b",
      adapter_kind: "generic_gtp", profile_snapshot: profile.profile } }];
  });
  const cancel = vi.spyOn(api, "cancelEnginePreload").mockImplementation(async () => {
    slots = [{ ...slots[0], phase: "cancelled" }];
  });
  const host = document.createElement("div");
  document.body.append(host);
  const root = createRoot(host);
  try {
    await act(async () => root.render(<EnginePreloadPanel profiles={[profile]} disabled={false} />));
    const buttons = host.querySelectorAll("button");
    expect(buttons[0].disabled).toBe(false);
    await act(async () => buttons[0].click());
    expect(prepare).toHaveBeenCalledWith("b");
    expect(host.textContent).toContain("准备中");
    expect(buttons[0].disabled).toBe(true);
    expect(buttons[1].disabled).toBe(false);
    await act(async () => buttons[1].click());
    expect(cancel).toHaveBeenCalledWith("b");
    expect(host.textContent).toContain("已取消");
    expect(buttons[0].disabled).toBe(false);
  } finally {
    await act(async () => root.unmount());
  }
});
