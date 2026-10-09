// @vitest-environment jsdom

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { loadEngineProfilesSettings, reorderEngineProfilesSettings, saveEngineProfilesSettings } from "./backend";
import type { EngineProfilesSettingsDto } from "../domain/types";

const key = "lizzieyzy-next-engine-profile";
const legacyProfile = {
  name: " 本地 KataGo 中文 ", engine_path: "/path with spaces/katago", model_path: "/模型/model.bin",
  config_path: "/config with spaces/analysis.cfg", working_dir: "/工作目录", backend: "kata_go_analysis"
};

function catalog(): EngineProfilesSettingsDto {
  return {
    version: 2, selected_profile_id: "gtp", startup: { mode: "fixed", profile_id: "kata" }, startup_evaluation: { enabled: false, target_profile_id: null }, last_primary_profile_id: null,
    profiles: [
      { id: "kata", profile: { name: "KataGo", program: "/with spaces/katago", argv: ["-override-config", "reportAnalysisWinratesAs=BLACK", "中文 参数", ""], working_dir: "/中文 工作目录", adapter_kind: "kata_go_analysis", settings: { model_path: "/模型", config_path: "/配置", max_visits: 1234 } } },
      { id: "gtp", profile: { name: "GNU Go", program: "/program with spaces/gnugo", argv: ["--mode", "gtp", "two words", "中文", ""], working_dir: null, adapter_kind: "generic_gtp", settings: {} } }
    ]
  };
}

beforeEach(() => {
  delete window.__TAURI_INTERNALS__;
  localStorage.clear();
});
afterEach(() => { vi.restoreAllMocks(); localStorage.clear(); });

describe("browser engine profile persistence", () => {
  it("loads an unconfigured default without writing storage or claiming a live run", async () => {
    const result = await loadEngineProfilesSettings();
    expect(result).toEqual({ version: 2, selected_profile_id: "default", startup: { mode: "off" }, startup_evaluation: { enabled: false, target_profile_id: null }, last_primary_profile_id: null, profiles: [{ id: "default", profile: { name: "Local KataGo", program: "", argv: [], working_dir: null, adapter_kind: "kata_go_analysis", settings: { model_path: null, config_path: null, max_visits: 800 } } }] });
    expect(localStorage.getItem(key)).toBeNull();
  });

  it("reads the legacy collection losslessly and only rewrites it on explicit save", async () => {
    const legacy = { selected_profile_id: "second", autoload_profile_id: "first", profiles: [{ id: "first", profile: legacyProfile, max_visits: 17 }, { id: "second", profile: { ...legacyProfile, name: "Second", working_dir: null }, max_visits: 4321 }] };
    const raw = JSON.stringify(legacy);
    localStorage.setItem(key, raw);
    const loaded = await loadEngineProfilesSettings();
    expect(loaded).toEqual({ version: 2, selected_profile_id: "second", startup: { mode: "fixed", profile_id: "first" }, startup_evaluation: { enabled: false, target_profile_id: null }, last_primary_profile_id: null, profiles: [{ id: "first", profile: { name: legacyProfile.name, program: legacyProfile.engine_path, argv: [], working_dir: legacyProfile.working_dir, adapter_kind: "kata_go_analysis", settings: { model_path: legacyProfile.model_path, config_path: legacyProfile.config_path, max_visits: 17 } } }, { id: "second", profile: { name: "Second", program: legacyProfile.engine_path, argv: [], working_dir: null, adapter_kind: "kata_go_analysis", settings: { model_path: legacyProfile.model_path, config_path: legacyProfile.config_path, max_visits: 4321 } } }] });
    expect(localStorage.getItem(key)).toBe(raw);
    await saveEngineProfilesSettings(loaded);
    expect(JSON.parse(localStorage.getItem(key)!)).toEqual(loaded);
  });

  it("reads the legacy single profile without rewriting its exact paths or budget", async () => {
    const raw = JSON.stringify({ profile: legacyProfile, max_visits: 987 });
    localStorage.setItem(key, raw);
    const loaded = await loadEngineProfilesSettings();
    expect(loaded.selected_profile_id).toBe("default");
    expect(loaded.startup).toEqual({ mode: "off" });
    expect(loaded.profiles[0]).toEqual({ id: "default", profile: { name: legacyProfile.name, program: legacyProfile.engine_path, argv: [], working_dir: legacyProfile.working_dir, adapter_kind: "kata_go_analysis", settings: { model_path: legacyProfile.model_path, config_path: legacyProfile.config_path, max_visits: 987 } } });
    expect(localStorage.getItem(key)).toBe(raw);
  });

  it("roundtrips both adapters with ordered, empty, spaced and Chinese argv", async () => {
    const settings = catalog();
    expect(await saveEngineProfilesSettings(settings)).toEqual(settings);
    expect(await loadEngineProfilesSettings()).toEqual(settings);
    expect(JSON.parse(localStorage.getItem(key)!)).toEqual(settings);
  });

  it.each([null, 0, 3, "1"])("rejects explicit unsupported version %s without rewriting it", async (version) => {
    const raw = JSON.stringify({ ...catalog(), version });
    localStorage.setItem(key, raw);
    await expect(loadEngineProfilesSettings()).rejects.toThrow("version");
    expect(localStorage.getItem(key)).toBe(raw);
  });

  it.each(["", "{", "null", JSON.stringify({ profile: { ...legacyProfile, backend: "generic_gtp" }, max_visits: 800 }), JSON.stringify({ version: 1, selected_profile_id: "default", profiles: [{ id: "default", profile: legacyProfile, max_visits: 800 }] })])("keeps malformed/unsupported stored data observable (%s)", async (raw) => {
    localStorage.setItem(key, raw);
    await expect(loadEngineProfilesSettings()).rejects.toThrow();
    expect(localStorage.getItem(key)).toBe(raw);
  });

  it.each([
    ["unknown adapter", (value: EngineProfilesSettingsDto) => ({ ...value, profiles: [{ id: "gtp", profile: { ...value.profiles[1].profile, adapter_kind: "unknown" } }] })],
    ["wrong settings", (value: EngineProfilesSettingsDto) => ({ ...value, profiles: [{ id: "gtp", profile: { ...value.profiles[1].profile, settings: { max_visits: 5 } } }] })],
    ["duplicate IDs", (value: EngineProfilesSettingsDto) => ({ ...value, profiles: [value.profiles[0], value.profiles[0]] })],
    ["dangling selection", (value: EngineProfilesSettingsDto) => ({ ...value, selected_profile_id: "missing" })],
    ["corrupt fixed identity", (value: EngineProfilesSettingsDto) => ({ ...value, startup: { mode: "fixed", profile_id: "bad\0id" } })],
    ["NUL argument", (value: EngineProfilesSettingsDto) => ({ ...value, profiles: value.profiles.map((record) => ({ ...record, profile: { ...record.profile, argv: ["bad\0argument"] } })) })],
    ["invalid visits", (value: EngineProfilesSettingsDto) => ({ ...value, profiles: [{ ...value.profiles[0], profile: { ...value.profiles[0].profile, settings: { model_path: null, config_path: null, max_visits: 0 } } }, value.profiles[1]] })]
  ] as const)("rejects %s without replacing the saved catalog", async (_label, invalid) => {
    const settings = catalog();
    await saveEngineProfilesSettings(settings);
    const raw = localStorage.getItem(key);
    await expect(saveEngineProfilesSettings(invalid(settings) as EngineProfilesSettingsDto)).rejects.toThrow();
    expect(localStorage.getItem(key)).toBe(raw);
    expect(await loadEngineProfilesSettings()).toEqual(settings);
  });

  it.each(["analysis", "gtp", "analysis=override", "selfplay", "-model", "--model=model.bin", "-config=analysis.cfg", "--config"])("rejects KataGo adapter-owned argv conflict %s", async (argument) => {
    const settings = catalog();
    settings.profiles[0].profile.argv = [argument];
    await expect(saveEngineProfilesSettings(settings)).rejects.toThrow("conflicts");
    expect(localStorage.getItem(key)).toBeNull();
  });

  it("surfaces storage read and write failures while preserving the previously stored catalog", async () => {
    const settings = catalog();
    await saveEngineProfilesSettings(settings);
    const raw = localStorage.getItem(key);
    const writing = vi.spyOn(Storage.prototype, "setItem").mockImplementation(() => { throw new Error("quota exhausted"); });
    await expect(saveEngineProfilesSettings({ ...settings, startup: { mode: "off" } })).rejects.toThrow("quota exhausted");
    expect(localStorage.getItem(key)).toBe(raw);
    writing.mockRestore();
    const reading = vi.spyOn(Storage.prototype, "getItem").mockImplementation(() => { throw new Error("storage denied"); });
    await expect(loadEngineProfilesSettings()).rejects.toThrow("storage denied");
    reading.mockRestore();
    expect(await loadEngineProfilesSettings()).toEqual(settings);
  });

  it("reorders only IDs, preserves late profile edits and identities, and rejects stale consumers", async () => {
    const current = catalog();
    await saveEngineProfilesSettings(current);
    const request = { expected_profile_ids: ["kata", "gtp"], profile_ids: ["gtp", "kata"] };
    current.profiles[0].profile.name = "Late edit";
    await saveEngineProfilesSettings(current);
    const result = await reorderEngineProfilesSettings(request);
    expect(result).toEqual({ ...current, profiles: [current.profiles[1], current.profiles[0]] });
    expect(await loadEngineProfilesSettings()).toEqual(result);
    const raw = localStorage.getItem(key);
    await expect(reorderEngineProfilesSettings(request)).rejects.toThrow("stale");
    expect(localStorage.getItem(key)).toBe(raw);
  });

  it("rejects invalid complete sets and retains durable order on storage failure", async () => {
    const current = catalog();
    await saveEngineProfilesSettings(current);
    const raw = localStorage.getItem(key);
    for (const ids of [["kata"], ["gtp", "gtp"], ["gtp", "missing"], ["kata", "gtp", "extra"]]) {
      await expect(reorderEngineProfilesSettings({ expected_profile_ids: ["kata", "gtp"], profile_ids: ids })).rejects.toThrow();
      expect(localStorage.getItem(key)).toBe(raw);
    }
    vi.spyOn(Storage.prototype, "setItem").mockImplementation(() => { throw new Error("quota exhausted"); });
    expect(await reorderEngineProfilesSettings({ expected_profile_ids: ["kata", "gtp"], profile_ids: ["kata", "gtp"] })).toEqual(current);
    await expect(reorderEngineProfilesSettings({ expected_profile_ids: ["kata", "gtp"], profile_ids: ["gtp", "kata"] })).rejects.toThrow("quota exhausted");
    expect(await loadEngineProfilesSettings()).toEqual(current);
    expect(localStorage.getItem(key)).toBe(raw);
  });

  it("keeps committed order when a late full editor save carries an older order", async () => {
    const stale = catalog();
    await saveEngineProfilesSettings(stale);
    await reorderEngineProfilesSettings({ expected_profile_ids: ["kata", "gtp"], profile_ids: ["gtp", "kata"] });
    stale.profiles[0].profile.name = "Late editor save";
    const extra = { ...stale.profiles[0], id: "new" };
    stale.profiles.unshift(extra);
    const saved = await saveEngineProfilesSettings(stale);
    expect(saved.profiles.map((record) => record.id)).toEqual(["gtp", "kata", "new"]);
    expect(saved.profiles[1].profile.name).toBe("Late editor save");
    expect(await loadEngineProfilesSettings()).toEqual(saved);
  });
  it.each([null, "kata"])("migrates version one startup %s without writing until save", async (id) => {
    const current = catalog();
    const old = { version: 1, selected_profile_id: current.selected_profile_id, autoload_profile_id: id, profiles: current.profiles };
    const bytes = JSON.stringify(old);
    localStorage.setItem(key, bytes);
    const loaded = await loadEngineProfilesSettings();
    expect(loaded.startup).toEqual(id === null ? { mode: "off" } : { mode: "fixed", profile_id: "kata" });
    expect(loaded.last_primary_profile_id).toBeNull();
    expect(localStorage.getItem(key)).toBe(bytes);
    await saveEngineProfilesSettings(loaded);
    expect(JSON.parse(localStorage.getItem(key)!)).not.toHaveProperty("autoload_profile_id");
  });

  it("reopens all modes and preserves exit-owned identity against stale editor writes", async () => {
    const current = { ...catalog(), last_primary_profile_id: "kata" };
    localStorage.setItem(key, JSON.stringify(current));
    for (const startup of [{ mode: "off" }, { mode: "fixed", profile_id: "gtp" }, { mode: "last_primary" }] as const) {
      const saved = await saveEngineProfilesSettings({ ...current, startup, last_primary_profile_id: "gtp" });
      expect(saved.startup).toEqual(startup);
      expect(saved.last_primary_profile_id).toBe("kata");
      expect(await loadEngineProfilesSettings()).toEqual(saved);
    }
    const missing = await saveEngineProfilesSettings({ ...current, startup: { mode: "fixed", profile_id: "deleted" } });
    expect(missing.startup).toEqual({ mode: "fixed", profile_id: "deleted" });
    expect((await loadEngineProfilesSettings()).selected_profile_id).toBe("gtp");
  });
  it("loads missing policy as disabled and rejects corrupt policy without rewriting durable bytes", async () => {
    const value = JSON.parse(JSON.stringify(catalog()));
    delete value.startup_evaluation;
    localStorage.setItem(key, JSON.stringify(value));
    expect((await loadEngineProfilesSettings()).startup_evaluation).toEqual({ enabled: false, target_profile_id: null });
    for (const invalid of [null, { enabled: "true" }, { enabled: true, target_profile_id: "bad\0id" }, { enabled: true, execution: "running" }]) {
      value.startup_evaluation = invalid;
      const bytes = JSON.stringify(value);
      localStorage.setItem(key, bytes);
      await expect(loadEngineProfilesSettings()).rejects.toThrow();
      expect(localStorage.getItem(key)).toBe(bytes);
    }
    const saved = { ...catalog(), startup_evaluation: { enabled: true, target_profile_id: "deleted" } };
    localStorage.setItem(key, JSON.stringify(catalog()));
    await saveEngineProfilesSettings(saved);
    expect((await loadEngineProfilesSettings()).startup_evaluation).toEqual(saved.startup_evaluation);
    expect((await loadEngineProfilesSettings()).selected_profile_id).toBe("gtp");
  });
});
