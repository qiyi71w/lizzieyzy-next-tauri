import { beforeEach, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { isTauriRuntime } from "./backend";
import { computeRawNet } from "./rawNet";
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("./backend", () => ({ isTauriRuntime: vi.fn() }));
const request = { run_id: "captured-run", generation: 7, node_path: { indices: [0, 1] } };
beforeEach(() => { vi.resetAllMocks(); });
it("dispatches exactly the captured typed request and preserves unavailable failures without retry", async () => {
  vi.mocked(isTauriRuntime).mockReturnValue(true);
  const failure = { message: "Busy; Pause ordinary analysis first" };
  vi.mocked(invoke).mockRejectedValue(failure);
  await expect(computeRawNet(request)).rejects.toBe(failure);
  expect(invoke).toHaveBeenCalledExactlyOnceWith("foreground_engine_compute_raw_net", { request });
});
it("refuses browser computation without invoking an engine or fabricating data", async () => {
  vi.mocked(isTauriRuntime).mockReturnValue(false);
  await expect(computeRawNet(request)).rejects.toThrow("native runtime");
  expect(invoke).not.toHaveBeenCalled();
});
