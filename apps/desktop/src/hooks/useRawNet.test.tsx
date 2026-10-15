// @vitest-environment jsdom
import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { useRawNet, type RawNetPresentation } from "./useRawNet";
import { computeRawNet } from "../api/rawNet";
import type { RawNetSnapshotDto } from "../domain/types";
vi.mock("../api/rawNet", () => ({ computeRawNet: vi.fn() }));
let root: Root;
let host: HTMLDivElement;
let control: RawNetPresentation;
function Harness({ scope = "A", run = "run", generation = 7 }: { scope?: string; run?: string; generation?: number }) {
  control = useRawNet(scope, { run_id: run, generation, node_path: { indices: [] } });
  return <output>{control.pending ? "pending" : control.payload?.ownership?.join(",") ?? control.error ?? ""}</output>;
}
const payload: RawNetSnapshotDto = {
  confirmation: { identity: { run_id: "run", job_id: "raw-1", generation: 7, node_path: { indices: [] } }, reader_id: "reader", profile_revision: "revision",
    position: { board_width: 2, board_height: 1, komi: 7.5, rules: "chinese", initial_player: "black", to_play: "black", initial_stones: [], moves: [] }, confirmed_rules: "{}", stones: [], true_final_move: null },
  resource: { profile_revision: "revision", resources: [], origin: "controlled-test", version: null, source_commit: null, backend: null, static_zlib_exemption: false },
  document_identity: 1, board_revision: 7, slot: "main_board", ownership: [-0.25, 0.5], unavailable: null,
};
beforeEach(() => { globalThis.IS_REACT_ACT_ENVIRONMENT = true; host = document.createElement("div"); document.body.append(host); root = createRoot(host); vi.mocked(computeRawNet).mockReset(); });
afterEach(() => { act(() => root.unmount()); host.remove(); });
it("computes only explicitly and retires data after leaving and returning", async () => {
  vi.mocked(computeRawNet).mockResolvedValue(payload);
  act(() => root.render(<Harness />));
  expect(computeRawNet).not.toHaveBeenCalled();
  await act(async () => { await control.compute(); });
  expect(host.textContent).toBe("-0.25,0.5");
  act(() => root.render(<Harness />));
  expect(computeRawNet).toHaveBeenCalledTimes(1);
  act(() => root.render(<Harness scope="B" />));
  act(() => root.render(<Harness />));
  expect(host.textContent).toBe("");
  expect(computeRawNet).toHaveBeenCalledTimes(1);
});
it("rejects late old-node and replaced-Run completion rather than reviving payload", async () => {
  const { promise, resolve } = Promise.withResolvers<RawNetSnapshotDto>();
  vi.mocked(computeRawNet).mockReturnValue(promise);
  act(() => root.render(<Harness />));
  let pending!: Promise<void>;
  act(() => { pending = control.compute(); });
  expect(host.textContent).toBe("pending");
  act(() => root.render(<Harness scope="replacement" run="new-run" generation={8} />));
  await act(async () => { resolve(payload); await pending; });
  expect(host.textContent).toBe("");
  act(() => root.render(<Harness />));
  expect(host.textContent).toBe("");
});
it("a new request seals older completion and busy refusal offers Pause without retry", async () => {
  const { promise, resolve } = Promise.withResolvers<RawNetSnapshotDto>();
  vi.mocked(computeRawNet).mockReturnValueOnce(promise).mockRejectedValueOnce(new Error("finite owner busy"));
  act(() => root.render(<Harness />));
  let pending!: Promise<void>;
  act(() => { pending = control.compute(); });
  await act(async () => { await control.compute(); });
  expect(host.textContent).toContain("finite owner busy");
  expect(host.textContent).toContain("暂停");
  await act(async () => { resolve(payload); await pending; });
  expect(host.textContent).toContain("finite owner busy");
  expect(computeRawNet).toHaveBeenCalledTimes(2);
});
