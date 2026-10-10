// @vitest-environment jsdom
import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { OrdinaryRulesPanel } from "./OrdinaryRulesPanel";
import * as backend from "../api/backend";
import type { EngineRunDto, OrdinaryRulesSnapshotDto } from "../domain/types";

let root: Root;
let host: HTMLDivElement;
const run: EngineRunDto = { run_id: "actual-run", profile_id: "gtp", adapter_kind: "kata_go_gtp",
  profile_snapshot: { name: "GTP", program: "/katago", argv: [], working_dir: null, adapter_kind: "kata_go_gtp", settings: { model_path: "/model", config_path: "/config", max_visits: 4 } } };
const receipt: OrdinaryRulesSnapshotDto = {
  identity: { run_id: run.run_id, job_id: "confirmed-request", generation: 7, node_path: { indices: [] } },
  reader_id: run.run_id, profile_revision: "revision", confirmed_rules: '{"ko":"SIMPLE"}', stones: [], true_final_move: null,
  position: { board_width: 9, board_height: 9, komi: 7.5, rules: "chinese", initial_player: "black", to_play: "black", initial_stones: [], moves: [] },
};
beforeEach(() => { globalThis.IS_REACT_ACT_ENVIRONMENT = true; host = document.createElement("div"); document.body.append(host); root = createRoot(host); });
afterEach(async () => { await act(async () => root.unmount()); host.remove(); vi.restoreAllMocks(); });
async function render(generation = 7, indices: number[] = [], native = true) {
  await act(async () => root.render(<OrdinaryRulesPanel run={run} generation={generation} nodePath={{ indices }} native={native} disabled={false} />));
}
const button = () => host.querySelector("button")!;
it("keeps sent requests unconfirmed and only presents an identity-matching actual receipt", async () => {
  const { promise, resolve } = Promise.withResolvers<OrdinaryRulesSnapshotDto>();
  vi.spyOn(backend, "confirmOrdinaryRules").mockReturnValue(promise);
  await render();
  await act(async () => button().click());
  expect(button().disabled).toBe(true);
  expect(host.textContent).toContain("尚未确认");
  expect(host.textContent).not.toContain("confirmed-request");
  await act(async () => resolve(receipt));
  expect(host.textContent).toContain("已确认：");
  expect(host.textContent).toContain("confirmed-request");
  await render(7, [0]);
  expect(host.textContent).not.toContain("confirmed-request");
  expect(host.textContent).toContain("尚未确认");
});
it("ignores late completion after navigation and shows a current rejection without fake confirmation", async () => {
  const { promise, resolve } = Promise.withResolvers<OrdinaryRulesSnapshotDto>();
  const call = vi.spyOn(backend, "confirmOrdinaryRules").mockReturnValueOnce(promise);
  await render();
  await act(async () => button().click());
  await render(8);
  await act(async () => resolve(receipt));
  expect(host.textContent).not.toContain("confirmed-request");
  call.mockRejectedValueOnce({ message: "unsupported white setup" });
  await act(async () => button().click());
  expect(host.querySelector('[role="alert"]')?.textContent).toContain("unsupported white setup");
  expect(host.textContent).not.toContain("已确认：");
});
it("does not emulate native confirmation in browser preview", async () => {
  const call = vi.spyOn(backend, "confirmOrdinaryRules");
  await render(7, [], false);
  expect(button().disabled).toBe(true);
  await act(async () => button().click());
  expect(call).not.toHaveBeenCalled();
  expect(host.textContent).toContain("需要原生桌面");
});
