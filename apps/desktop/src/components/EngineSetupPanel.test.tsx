// @vitest-environment jsdom

import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { EngineSetupPanel } from "./EngineSetupPanel";
import { loadEngineProfilesSettings, saveEngineProfilesSettings } from "../api/backend";
import * as backend from "../api/backend";
import type { ForegroundEngineSnapshotDto } from "../domain/types";

let root: Root | null = null;
let host: HTMLDivElement;

beforeEach(() => {
  globalThis.IS_REACT_ACT_ENVIRONMENT = true;
  delete window.__TAURI_INTERNALS__;
  localStorage.clear();
  host = document.createElement("div");
  document.body.append(host);
});
afterEach(async () => {
  await act(async () => { root?.unmount(); });
  root = null;
  host.remove();
  vi.restoreAllMocks();
  localStorage.clear();
});

function field(label: string): HTMLInputElement | HTMLSelectElement {
  const wrapper = Array.from(host.querySelectorAll("label")).find((element) => element.querySelector("span")?.textContent === label);
  const input = wrapper?.querySelector("input, select");
  if (!input) throw new Error(`Missing field ${label}`);
  return input as HTMLInputElement | HTMLSelectElement;
}

async function change(input: HTMLInputElement | HTMLSelectElement, value: string) {
  await act(async () => {
    if (input instanceof HTMLSelectElement) {
      input.value = value;
      input.dispatchEvent(new Event("change", { bubbles: true }));
    } else {
      Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")!.set!.call(input, value);
      input.dispatchEvent(new Event("input", { bubbles: true }));
    }
  });
}

async function click(label: string) {
  const button = Array.from(host.querySelectorAll("button")).find((element) => element.textContent === label);
  if (!button) throw new Error(`Missing button ${label}`);
  await act(async () => { button.click(); });
}

async function render(snapshot: ForegroundEngineSnapshotDto | null = null) {
  root = createRoot(host);
  await act(async () => { root!.render(<EngineSetupPanel engineSnapshot={snapshot} />); });
}

describe("engine profile configuration editor", () => {
  it("creates, edits, saves and reloads both adapters with verbatim per-item argv", async () => {
    await render();
    await change(field("名称"), "中文 KataGo");
    await change(field("引擎"), "/path with spaces/katago");
    await change(field("模型"), "/模型/model.bin");
    await change(field("配置文件"), "/配置/analysis.cfg");
    await change(field("工作目录"), "/中文 工作目录");
    await change(field("最大计算量"), "1234");
    for (const argument of ["two words", "中文 参数", ""]) {
      await click("新增参数");
      const inputs = host.querySelectorAll<HTMLInputElement>('input[aria-label^="参数 "]');
      await change(inputs[inputs.length - 1], argument);
    }
    await click("保存配置");
    const kata = (await loadEngineProfilesSettings()).profiles[0];
    expect(kata.profile).toEqual({ name: "中文 KataGo", program: "/path with spaces/katago", argv: ["two words", "中文 参数", ""], working_dir: "/中文 工作目录", adapter_kind: "kata_go_analysis", settings: { model_path: "/模型/model.bin", config_path: "/配置/analysis.cfg", max_visits: 1234 } });

    await click("新增");
    const gtpId = (await loadEngineProfilesSettings()).selected_profile_id;
    expect(gtpId).not.toBe(kata.id);
    await change(field("适配器"), "generic_gtp");
    expect(host.textContent).not.toContain("最大计算量");
    expect(host.querySelector('input[placeholder="/path/to/model.bin.gz"]')).toBeNull();
    await change(field("名称"), "GNU Go 中文");
    await change(field("引擎"), "/gnu go/gnugo");
    await click("保存配置");
    const saved = await loadEngineProfilesSettings();
    expect(saved.profiles).toEqual([kata, { id: gtpId, profile: { name: "GNU Go 中文", program: "/gnu go/gnugo", argv: ["two words", "中文 参数", ""], working_dir: "/中文 工作目录", adapter_kind: "generic_gtp", settings: {} } }]);
    await change(field("名称"), "unsaved edit");
    await change(field("参数 1"), "not saved");
    await click("重新加载配置");
    expect(field("名称").value).toBe("GNU Go 中文");
    expect(field("参数 1").value).toBe("two words");
    expect(field("参数 3").value).toBe("");
    await change(field("配置"), kata.id);
    expect(field("适配器").value).toBe("kata_go_analysis");
    expect(field("模型").value).toBe("/模型/model.bin");
    expect(field("最大计算量").value).toBe("1234");
  });

  it("keeps the live run and capability immutable while adapter, argv and settings edits are pending", async () => {
    const settings = await loadEngineProfilesSettings();
    settings.profiles[0].profile.program = "/katago";
    await saveEngineProfilesSettings(settings);
    const run = { run_id: "run-1", profile_id: "default", adapter_kind: "kata_go_analysis" as const, profile_snapshot: structuredClone(settings.profiles[0].profile), capability_snapshot: { adapter_kind: "kata_go_analysis" as const, analysis: { selected_node_analysis: true, continuous_analysis: true, whole_game_analysis: true, candidates: true, pv: true, winrate: true, root_score: true, ownership: true, policy: true, visits_limit: true, protocol_cancel: true } } };
    const snapshot: ForegroundEngineSnapshotDto = { revision: 1, lifecycle: { state: "ready", run }, continuous: { enabled: true, phase: "waiting" } };
    const original = structuredClone(snapshot);
    await render(snapshot);
    expect(host.textContent).not.toContain("存在待应用更改");
    await change(field("最大计算量"), "2000");
    expect((await loadEngineProfilesSettings()).profiles[0].profile).toEqual(run.profile_snapshot);
    await click("保存配置");
    expect(host.textContent).toContain("存在待应用更改");
    await click("新增参数");
    await change(field("参数 1"), "中文 with spaces");
    await change(field("适配器"), "generic_gtp");
    await click("保存配置");
    expect(host.textContent).toContain("存在待应用更改");
    expect(snapshot).toEqual(original);
    const saved = await loadEngineProfilesSettings();
    expect(saved.profiles[0].profile.adapter_kind).toBe("generic_gtp");
    expect(saved.profiles[0].profile.argv).toEqual(["中文 with spaces"]);
    await change(field("适配器"), "kata_go_analysis");
    await click("保存配置");
    const restartProfile = (await loadEngineProfilesSettings()).profiles[0].profile;
    const restarted: ForegroundEngineSnapshotDto = { ...snapshot, revision: 2, lifecycle: { state: "ready", run: { ...run, run_id: "run-2", profile_snapshot: restartProfile } } };
    await act(async () => { root!.render(<EngineSetupPanel engineSnapshot={restarted} />); });
    expect(host.textContent).not.toContain("存在待应用更改");
  });

  it("keeps verified GTP facts visible when a saved profile changes adapter and identity", async () => {
    const settings = await loadEngineProfilesSettings();
    settings.profiles[0].profile = { name: "Saved GTP", program: "/gtp", argv: [], working_dir: null, adapter_kind: "generic_gtp", settings: {} };
    await saveEngineProfilesSettings(settings);
    const facts = { protocol_version: 2, name: "Handshake engine", version: "3.8", commands: ["boardsize", "clear_board", "komi", "play", "genmove", "quit", "time_settings", "time_left", "set_free_handicap", "custom_rules"] };
    const snapshot: ForegroundEngineSnapshotDto = {
      revision: 1, continuous: { enabled: true, phase: "unavailable" },
      lifecycle: { state: "ready", run: {
        run_id: "gtp-1", profile_id: settings.profiles[0].id, adapter_kind: "generic_gtp",
        profile_snapshot: structuredClone(settings.profiles[0].profile),
        capability_snapshot: { adapter_kind: "generic_gtp", gtp: facts }
      } }
    };
    await render(snapshot);
    const verifiedLabel = host.querySelector('[aria-label="当前 run 能力"]')?.textContent;
    expect(verifiedLabel).toContain(facts.name);
    expect(verifiedLabel).toContain(facts.version);
    for (const command of facts.commands) expect(verifiedLabel).toContain(command);
    expect(verifiedLabel).toContain("落子不可用");
    if (snapshot.lifecycle.state !== "ready") throw new Error("Expected Ready fixture");
    const qualified: ForegroundEngineSnapshotDto = { ...snapshot, revision: 2,
      lifecycle: { state: "ready", run: { ...snapshot.lifecycle.run,
        capability_snapshot: { adapter_kind: "generic_gtp", game_move: true, gtp: { ...facts, name: "GNU Go" } } } } };
    await act(async () => { root!.render(<EngineSetupPanel engineSnapshot={qualified} />); });
    const qualifiedLabel = host.querySelector('[aria-label="当前 run 能力"]')?.textContent;
    expect(qualifiedLabel).toContain("落子支持");
    expect(qualifiedLabel).toContain("2/5/9/13/19");
    expect(qualifiedLabel).toContain("Chinese KGS");
    await change(field("名称"), "Pending engine");
    await change(field("适配器"), "kata_go_analysis");
    await click("保存配置");
    expect((await loadEngineProfilesSettings()).profiles[0].profile.adapter_kind).toBe("kata_go_analysis");
    expect(host.querySelector('[aria-label="当前 run 能力"]')?.textContent).toBe(qualifiedLabel);
    expect(host.querySelector('[aria-label="当前 run 能力"]')?.textContent).not.toContain("Pending engine");
  });

  it("shows malformed storage and save errors instead of silently replacing the catalog", async () => {
    const key = "lizzieyzy-next-engine-profile";
    localStorage.setItem(key, "{broken");
    await render();
    expect(host.textContent).toContain("Load failed:");
    const save = Array.from(host.querySelectorAll("button")).find((button) => button.textContent === "保存配置")!;
    expect(save.disabled).toBe(true);
    expect(localStorage.getItem(key)).toBe("{broken");
    localStorage.removeItem(key);
    await click("重新加载配置");
    await click("新增参数");
    await change(field("参数 1"), "--model=override");
    await click("保存配置");
    expect(host.textContent).toContain("Save failed:");
    expect(localStorage.getItem(key)).toBeNull();
  });

  it("persists all four row moves without saving drafts or changing selection, default, Run or Job", async () => {
    const settings = await loadEngineProfilesSettings();
    const alpha = { ...settings.profiles[0], id: "alpha", profile: { ...settings.profiles[0].profile, name: "Alpha" } };
    const beta = { ...alpha, id: "beta", profile: { ...alpha.profile, name: "Beta" } };
    settings.profiles.push(alpha, beta);
    settings.selected_profile_id = "alpha";
    settings.startup = { mode: "fixed", profile_id: "alpha" };
    await saveEngineProfilesSettings(settings);
    const snapshot: ForegroundEngineSnapshotDto = {
      revision: 17, lifecycle: { state: "ready", run: { run_id: "immutable-run", profile_id: "default", adapter_kind: "kata_go_analysis", profile_snapshot: structuredClone(settings.profiles[0].profile) } },
      continuous: { enabled: true, phase: "searching" },
      selected_node_job: { run_id: "immutable-run", job_id: "immutable-job", lane: "selected_node", mode: "continuous", state: "searching", generation: 3, node_path: { indices: [0] } }
    };
    const original = structuredClone(snapshot);
    await render(snapshot);
    await change(field("名称"), "Pending name");
    await change(field("引擎"), "/pending engine");
    await click("新增参数");
    await change(field("参数 1"), "unsaved argv");
    const order = () => Array.from(host.querySelectorAll<HTMLElement>("[data-profile-order-id]")).map((row) => row.dataset.profileOrderId);
    const move = async (id: string, label: string) => {
      const row = host.querySelector(`[data-profile-order-id="${id}"]`)!;
      const button = Array.from(row.querySelectorAll("button")).find((element) => element.textContent === label)!;
      await act(async () => { button.click(); });
      expect(field("配置").value).toBe("alpha");
      expect(field("名称").value).toBe("Pending name");
      expect(field("引擎").value).toBe("/pending engine");
      expect(field("参数 1").value).toBe("unsaved argv");
      const saved = await loadEngineProfilesSettings();
      expect(saved.selected_profile_id).toBe("alpha");
      expect(saved.startup).toEqual({ mode: "fixed", profile_id: "alpha" });
      expect(saved.profiles.find((record) => record.id === "alpha")).toEqual(alpha);
      expect(snapshot).toEqual(original);
      expect((host.querySelector('[aria-label="启动方式"]') as HTMLSelectElement).value).toBe("fixed");
    };
    await move("beta", "置首"); expect(order()).toEqual(["beta", "default", "alpha"]);
    await move("alpha", "上移"); expect(order()).toEqual(["beta", "alpha", "default"]);
    await move("alpha", "下移"); expect(order()).toEqual(["beta", "default", "alpha"]);
    await move("beta", "置尾"); expect(order()).toEqual(["default", "alpha", "beta"]);
    for (const [id, labels] of [["default", ["置首", "上移"]], ["beta", ["下移", "置尾"]]] as const) {
      const row = host.querySelector(`[data-profile-order-id="${id}"]`)!;
      for (const label of labels) expect(Array.from(row.querySelectorAll("button")).find((button) => button.textContent === label)!.disabled).toBe(true);
    }
    const before = localStorage.getItem("lizzieyzy-next-engine-profile");
    vi.spyOn(Storage.prototype, "setItem").mockImplementation(() => { throw new Error("quota exhausted"); });
    await move("beta", "置首");
    expect(order()).toEqual(["default", "alpha", "beta"]);
    expect(localStorage.getItem("lizzieyzy-next-engine-profile")).toBe(before);
    expect(host.textContent).toContain("排序未保存");
    expect(host.textContent).toContain("quota exhausted");
    vi.restoreAllMocks();
    await act(async () => { root!.unmount(); }); root = null;
    await render(snapshot);
    expect(order()).toEqual(["default", "alpha", "beta"]);
    expect(field("配置").value).toBe("alpha");
  });

  it("retains edits typed during a reorder and fences concurrent catalog actions", async () => {
    const settings = await loadEngineProfilesSettings();
    settings.profiles.push({ ...settings.profiles[0], id: "second", profile: { ...settings.profiles[0].profile, name: "Second" } });
    await saveEngineProfilesSettings(settings);
    await render();
    let release!: () => void;
    const gate = new Promise<void>((resolve) => { release = resolve; });
    const reorder = backend.reorderEngineProfilesSettings;
    const pending = vi.spyOn(backend, "reorderEngineProfilesSettings").mockImplementation(async (request) => {
      await gate;
      return await reorder(request);
    });
    await click("置尾");
    expect(host.textContent).toContain("正在保存档案顺序");
    expect((field("配置") as HTMLSelectElement).disabled).toBe(true);
    expect(Array.from(host.querySelectorAll("button")).find((button) => button.textContent === "保存配置")!.disabled).toBe(true);
    await click("置尾");
    expect(pending).toHaveBeenCalledOnce();
    await change(field("名称"), "Typed while saving order");
    await change(field("引擎"), "/late draft");
    await act(async () => { release(); await gate; });
    expect(field("名称").value).toBe("Typed while saving order");
    expect(field("引擎").value).toBe("/late draft");
    expect(field("配置").value).toBe("default");
    expect(Array.from(host.querySelectorAll<HTMLElement>("[data-profile-order-id]")).map((row) => row.dataset.profileOrderId)).toEqual(["second", "default"]);
    expect((await loadEngineProfilesSettings()).profiles[1]).toEqual(settings.profiles[0]);
    expect((field("配置") as HTMLSelectElement).disabled).toBe(false);
  });

  it("shows a stale catalog error without losing the pending editor or replacing newer storage", async () => {
    const settings = await loadEngineProfilesSettings();
    settings.profiles.push({ ...settings.profiles[0], id: "second", profile: { ...settings.profiles[0].profile, name: "Second" } });
    await saveEngineProfilesSettings(settings);
    await render();
    await change(field("名称"), "Retained pending edit");
    await backend.reorderEngineProfilesSettings({ expected_profile_ids: ["default", "second"], profile_ids: ["second", "default"] });
    const newerBytes = localStorage.getItem("lizzieyzy-next-engine-profile");
    await click("置尾");
    expect(host.textContent).toContain("排序未保存");
    expect(host.textContent).toContain("stale");
    expect(field("名称").value).toBe("Retained pending edit");
    expect(field("配置").value).toBe("default");
    expect(localStorage.getItem("lizzieyzy-next-engine-profile")).toBe(newerBytes);
    expect(Array.from(host.querySelectorAll<HTMLElement>("[data-profile-order-id]")).map((row) => row.dataset.profileOrderId)).toEqual(["default", "second"]);
  });
});
