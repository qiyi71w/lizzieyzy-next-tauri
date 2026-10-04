// @vitest-environment jsdom
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { defaultAppPreferences, normalizeAppPreferences } from "../domain/preferences";
import { humanMatchSnapshot, humanMatchStop, pkMatchStart, pkMatchPause, pkMatchResume } from "./match";

const transport = vi.hoisted(() => ({ invoke: vi.fn(), listen: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: transport.invoke }));
vi.mock("@tauri-apps/api/event", () => ({ listen: transport.listen }));
beforeEach(() => { vi.clearAllMocks(); });
afterEach(() => { delete window.__TAURI_INTERNALS__; });

describe("native human match transport", () => {
  it("does not manufacture a browser match", async () => {
    await expect(humanMatchSnapshot()).rejects.toThrow("桌面运行时");
    await expect(humanMatchStop("session")).rejects.toThrow("桌面运行时");
    await expect(pkMatchStart({ settings: defaultAppPreferences.matchDefaults, generation: 1, snapshot_seq: 1, start: { kind: "new", discard_confirmed: false } })).rejects.toThrow("桌面运行时");
    await expect(pkMatchPause("session")).rejects.toThrow("桌面运行时");
    await expect(pkMatchResume("session")).rejects.toThrow("桌面运行时");
    expect(transport.invoke).not.toHaveBeenCalled();
  });
  it("normalizes stable defaults without inventing a profile or rules", () => {
    expect(normalizeAppPreferences(null).matchDefaults).toEqual({ board_size: 19, komi: 7.5, handicap: 0, human_color: "black", rules: null, profile_id: null, deadline_ms: 30000, kata_max_visits: 800, pk_black: { profile_id: null, deadline_ms: 30000, kata_max_visits: 800 }, pk_white: { profile_id: null, deadline_ms: 30000, kata_max_visits: 800 }, pk_max_moves: 450 });
    const defaults = { ...defaultAppPreferences.matchDefaults, profile_id: "saved", rules: "chinese_kgs" as const, human_color: "white" as const };
    expect(normalizeAppPreferences({ matchDefaults: defaults }).matchDefaults).toEqual(defaults);
  });
  it("loads legacy human preferences with independent PK sides and no live session", () => {
    const legacy = { board_size: 9, komi: 6.5, handicap: 0, human_color: "white" as const, rules: "chinese" as const, profile_id: "human-saved", deadline_ms: 1000, kata_max_visits: 30, pk_max_moves: 500 };
    const stored = { matchDefaults: { ...legacy, session_id: "stale", run_id: "stale-run", phase: "paused" } };
    const normalized = normalizeAppPreferences(stored).matchDefaults;
    expect(normalized).toEqual({ ...legacy, pk_black: { profile_id: null, deadline_ms: 30000, kata_max_visits: 800 }, pk_white: { profile_id: null, deadline_ms: 30000, kata_max_visits: 800 } });
    expect(normalized.pk_black).not.toBe(normalized.pk_white);
    expect(normalized).not.toHaveProperty("session_id");
    expect(normalized).not.toHaveProperty("run_id");
    expect(normalized).not.toHaveProperty("phase");
  });
  it("persists only stable side IDs and valid positive budgets", () => {
    const normalized = normalizeAppPreferences({ matchDefaults: { pk_black: { profile_id: "same", deadline_ms: 0, kata_max_visits: -1 }, pk_white: { profile_id: "same", deadline_ms: 7000, kata_max_visits: 400 }, pk_max_moves: 0 } }).matchDefaults;
    expect(normalized.pk_black).toEqual({ profile_id: "same", deadline_ms: 1, kata_max_visits: 1 });
    expect(normalized.pk_white).toEqual({ profile_id: "same", deadline_ms: 7000, kata_max_visits: 400 });
    expect(normalized.pk_max_moves).toBe(1);
  });
});
