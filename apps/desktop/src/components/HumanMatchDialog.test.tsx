// @vitest-environment jsdom
import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { defaultAppPreferences } from "../domain/preferences";
import type { EngineProfileRecordDto, ForegroundEngineSnapshotDto, MatchDefaultsDto, MatchModeDto, PositionDto, SgfTreeNodeDto } from "../domain/types";
import { HumanMatchDialog } from "./HumanMatchDialog";

let root: Root | null = null;
beforeEach(() => { globalThis.IS_REACT_ACT_ENVIRONMENT = true; });
afterEach(() => { act(() => root?.unmount()); root = null; document.body.replaceChildren(); });
const profile: EngineProfileRecordDto = { id: "generic", profile: { name: "Saved GTP", program: "gtp", argv: [], working_dir: null, adapter_kind: "generic_gtp", settings: {} } };
const kataProfile: EngineProfileRecordDto = { id: "kata", profile: { name: "Saved KataGo", program: "katago", argv: [], working_dir: null, adapter_kind: "kata_go_analysis", settings: { model_path: "model.bin", config_path: "analysis.cfg", max_visits: 800 } } };
function renderDialog(pending = false, engineSnapshot?: ForegroundEngineSnapshotDto, continuation?: PositionDto, mode: MatchModeDto = "human", defaults: MatchDefaultsDto = defaultAppPreferences.matchDefaults, continuationRoot?: SgfTreeNodeDto) {
  const host = document.createElement("div");
  document.body.append(host);
  root = createRoot(host);
  const start = vi.fn();
  const cancel = vi.fn();
  act(() => root?.render(<HumanMatchDialog mode={mode} defaults={defaults} profiles={[profile, kataProfile]} engineSnapshot={engineSnapshot} continuation={continuation} continuationRoot={continuationRoot} pending={pending} onStart={start} onCancel={cancel} />));
  return { host, start, cancel };
}
function select(host: HTMLElement, label: string, value: string) {
  const field = [...host.querySelectorAll("label")].find((item) => item.querySelector("span")?.textContent === label)?.querySelector("select");
  if (!field) throw new Error(`Missing field ${label}`);
  act(() => { field.value = value; field.dispatchEvent(new Event("change", { bubbles: true })); });
}
describe("HumanMatchDialog", () => {
  const ready: ForegroundEngineSnapshotDto = {
    revision: 1, continuous: { enabled: false, phase: "off" },
    lifecycle: { state: "ready", run: { run_id: "gtp-run", profile_id: profile.id,
      adapter_kind: "generic_gtp", profile_snapshot: profile.profile,
      capability_snapshot: { adapter_kind: "generic_gtp", game_move: true } } }
  };
  it("shows qualification only after selecting the exact saved profile for the Ready run", () => {
    const { host } = renderDialog(false, ready);
    const label = () => host.querySelector('[aria-label="所选配置落子能力"]')?.textContent;
    expect(label()).toContain("未验证");
    select(host, "已保存的引擎配置", profile.id);
    expect(label()).toContain("落子支持");
    expect(label()).toContain("Chinese KGS");
    expect(label()).toContain("不支持 setup/让子");
    select(host, "已保存的引擎配置", "");
    expect(label()).toContain("未验证");
  });
  it.each(["another-profile", "stale-argv", "stale-program", "starting"])("does not borrow capabilities from %s", (mismatch) => {
    const snapshot = structuredClone(ready);
    if (snapshot.lifecycle.state !== "ready") throw new Error("Expected Ready fixture");
    const run = snapshot.lifecycle.run;
    if (mismatch === "another-profile") run.profile_id = "another";
    if (mismatch === "stale-argv") run.profile_snapshot.argv = ["old"];
    if (mismatch === "stale-program") run.profile_snapshot.program = "old";
    if (mismatch === "starting") snapshot.lifecycle = { state: "starting", run };
    const { host } = renderDialog(false, snapshot);
    select(host, "已保存的引擎配置", profile.id);
    const label = host.querySelector('[aria-label="所选配置落子能力"]')?.textContent;
    expect(label).toContain("未验证");
    expect(label).not.toContain("落子支持");
    expect(label).not.toContain("Chinese KGS");
  });
  it("requires explicit saved profile and exact rules, keeping generic visits hidden", () => {
    const { host, start } = renderDialog();
    const submit = host.querySelector<HTMLButtonElement>('button[type="submit"]')!;
    expect(submit.disabled).toBe(true);
    select(host, "已保存的引擎配置", "generic");
    expect(submit.disabled).toBe(true);
    expect(host.textContent).not.toContain("KataGo 每步最大 visits");
    select(host, "精确规则", "chinese_kgs");
    expect(submit.disabled).toBe(false);
    act(() => submit.click());
    expect(start).toHaveBeenCalledWith({ ...defaultAppPreferences.matchDefaults, profile_id: "generic", rules: "chinese_kgs" });
    expect(defaultAppPreferences.matchDefaults.profile_id).toBeNull();
  });
  it("freezes the draft during startup while keeping Stop available", () => {
    const { host, start, cancel } = renderDialog(true);
    expect(host.querySelector("fieldset")?.disabled).toBe(true);
    expect(host.querySelector<HTMLButtonElement>('button[type="submit"]')?.disabled).toBe(true);
    const stop = [...host.querySelectorAll("button")].find((button) => button.textContent === "停止启动")!;
    expect(stop.disabled).toBe(false);
    act(() => stop.click());
    expect(cancel).toHaveBeenCalledOnce();
    expect(start).not.toHaveBeenCalled();
  });
  it("continues only after the whole-document metadata scope is confirmed, without position overrides", () => {
    const position: PositionDto = { board_width: 9, board_height: 9, move_number: 12, to_play: "white", stones: [], captures_black: 0, captures_white: 0, errors: [] };
    const { host, start } = renderDialog(false, undefined, position);
    expect(host.querySelector('[role="dialog"]')?.getAttribute("aria-label")).toBe("人机续弈");
    for (const label of ["棋盘大小", "贴目", "让子（0 或 2–9）", "精确规则"]) {
      expect([...host.querySelectorAll("label span")].some((span) => span.textContent === label)).toBe(false);
    }
    const submit = host.querySelector<HTMLButtonElement>('button[type="submit"]')!;
    select(host, "已保存的引擎配置", "generic");
    expect(submit.disabled).toBe(true);
    const confirm = host.querySelector<HTMLInputElement>(".human-match-root-confirmation input")!;
    act(() => confirm.click());
    expect(submit.disabled).toBe(false);
    act(() => submit.click());
    expect(start).toHaveBeenCalledWith({ ...defaultAppPreferences.matchDefaults, profile_id: "generic" });
  });
});

describe("PK form", () => {
  const defaults: MatchDefaultsDto = { ...defaultAppPreferences.matchDefaults, rules: "chinese", pk_black: { profile_id: kataProfile.id, deadline_ms: 30000, kata_max_visits: 800 }, pk_white: { profile_id: kataProfile.id, deadline_ms: 12000, kata_max_visits: 300 } };
  it("accepts the same saved KataGo twice and a GTP side budgeted only by its deadline", () => {
    const { host, start } = renderDialog(false, undefined, undefined, "pk", defaults);
    expect(host.querySelector('[role="dialog"]')?.getAttribute("aria-label")).toBe("PK新局");
    expect(host.textContent).toContain("GNU Go 3.8");
    expect(host.textContent).toContain("原始配置快照重建");
    const submit = host.querySelector<HTMLButtonElement>('button[type="submit"]')!;
    expect(submit.disabled).toBe(false);
    select(host, "白方已保存的引擎配置", profile.id);
    expect(host.textContent).not.toContain("白方 KataGo 每步最大 visits");
    expect(host.textContent).toContain("白方 GTP：仅使用每步硬截止");
    expect(host.textContent).toContain("黑方 KataGo 每步最大 visits");
    expect(submit.disabled).toBe(false);
    act(() => submit.click());
    expect(start).toHaveBeenCalledWith({ ...defaults, pk_white: { ...defaults.pk_white, profile_id: profile.id } });
    expect(defaultAppPreferences.matchDefaults.pk_black.profile_id).toBeNull();
  });
  it.each([
    ["black deadline", { ...defaults, pk_black: { ...defaults.pk_black, deadline_ms: 0 } }],
    ["black visits", { ...defaults, pk_black: { ...defaults.pk_black, kata_max_visits: 0 } }],
    ["white deadline", { ...defaults, pk_white: { ...defaults.pk_white, deadline_ms: 0 } }],
    ["white visits", { ...defaults, pk_white: { ...defaults.pk_white, kata_max_visits: 0 } }],
    ["move limit", { ...defaults, pk_max_moves: 0 }],
    ["fractional visits", { ...defaults, pk_white: { ...defaults.pk_white, kata_max_visits: 1.5 } }]
  ])("rejects an invalid %s without invoking start", (_name, invalid) => {
    const { host, start } = renderDialog(false, undefined, undefined, "pk", invalid);
    expect(host.querySelector<HTMLButtonElement>('button[type="submit"]')?.disabled).toBe(true);
    act(() => host.querySelector("form")!.dispatchEvent(new Event("submit", { bubbles: true, cancelable: true })));
    expect(start).not.toHaveBeenCalled();
  });
  it("shows inherited board/rules instead of defaults and requires root confirmation for PK Continue", () => {
    const position: PositionDto = { board_width: 9, board_height: 13, move_number: 21, to_play: "white", stones: [], captures_black: 0, captures_white: 0, errors: [] };
    const tree: SgfTreeNodeDto = { properties: [{ key: "RU", values: ["Chinese KGS"] }, { key: "KM", values: ["6.5"] }, { key: "HA", values: ["2"] }], children: [] };
    const { host, start } = renderDialog(false, undefined, position, "pk", defaults, tree);
    expect(host.textContent).toContain("棋盘 9×13 · 规则 Chinese KGS · 贴目 6.5 · 让子 2");
    expect(host.textContent).toContain("白方行棋");
    for (const label of ["棋盘大小", "贴目", "让子（0 或 2–9）", "精确规则"]) {
      expect([...host.querySelectorAll("label span")].some((span) => span.textContent === label)).toBe(false);
    }
    const submit = host.querySelector<HTMLButtonElement>('button[type="submit"]')!;
    expect(submit.disabled).toBe(true);
    act(() => host.querySelector<HTMLInputElement>(".human-match-root-confirmation input")!.click());
    act(() => submit.click());
    expect(start).toHaveBeenCalledWith(defaults);
  });
});
