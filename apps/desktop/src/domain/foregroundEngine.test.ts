import { describe, expect, it } from "vitest";
import {
  admitsForegroundEngineJobs,
  canRestartForegroundEngine,
  canStopForegroundEngine,
  displayedEngineFailure,
  emptyForegroundEngineSnapshot,
  engineStatusLabel,
  isForegroundEngineReady,
  mergeForegroundEngineSnapshot,
  profileHasPendingChanges,
  shouldAcceptFailureEvent
} from "./foregroundEngine";
import type { EngineFailureDto, EngineRunDto, ForegroundEngineSnapshotDto } from "./types";

const run: EngineRunDto = {
  run_id: "run-1",
  profile_id: "profile-1",
  adapter_kind: "kata_go_analysis",
  profile_snapshot: {
    name: "Local KataGo",
    program: "/bin/katago", argv: [],
    settings: { model_path: "/models/model.bin", config_path: "/configs/analysis.cfg", max_visits: 800 },
    working_dir: "/tmp",
    adapter_kind: "kata_go_analysis"
  },
  capability_snapshot: {
    adapter_kind: "kata_go_analysis",
    analysis: {
      selected_node_analysis: true,
      continuous_analysis: true,
      candidates: true,
      pv: true,
      winrate: true,
      ownership: true,
      policy: true,
      visits_limit: true,
      whole_game_analysis: true,
      root_score: true,
      protocol_cancel: true
    }
  }
};

function snapshot(revision: number, lifecycle: ForegroundEngineSnapshotDto["lifecycle"]): ForegroundEngineSnapshotDto {
  return { revision, lifecycle, continuous: { enabled: null, phase: "loading" } };
}

describe("foreground engine snapshot merge", () => {
  it("keeps the higher revision and does not apply an older snapshot", () => {
    const ready = snapshot(4, { state: "ready", run });
    const stale = snapshot(2, { state: "starting", run: { ...run, capability_snapshot: null } });
    expect(mergeForegroundEngineSnapshot(ready, stale)).toEqual(ready);
    expect(mergeForegroundEngineSnapshot(stale, ready)).toEqual(ready);
  });

  it("labels no-engine without using profile completeness", () => {
    const empty = emptyForegroundEngineSnapshot();
    expect(engineStatusLabel(empty)).toBe("未加载引擎");
    expect(isForegroundEngineReady(empty)).toBe(false);
  });

  it("keeps A job admission and stop available during Switching", () => {
    const candidate = { ...run, run_id: "run-b", profile_id: "profile-b", capability_snapshot: null };
    const switching = snapshot(3, { state: "switching", primary: run, candidate, switch_id: "1" });
    expect(isForegroundEngineReady(switching)).toBe(false);
    expect(admitsForegroundEngineJobs(switching)).toBe(true);
    expect(canStopForegroundEngine(switching)).toBe(true);
    expect(engineStatusLabel(switching)).toBe("正在切换 Local KataGo");
  });

  it("reports pending changes against the immutable run snapshot", () => {
    const ready = snapshot(1, { state: "ready", run });
    expect(profileHasPendingChanges(run.profile_snapshot, ready)).toBe(false);
    expect(profileHasPendingChanges({ ...run.profile_snapshot, program: "/other/katago" }, ready)).toBe(true);
  });

  it("compares every saved field without mutating the live run or verified capability", () => {
    const ready = snapshot(1, { state: "ready", run });
    const original = structuredClone(ready);
    if (run.profile_snapshot.adapter_kind !== "kata_go_analysis") throw new Error("Expected KataGo fixture");
    const profile = run.profile_snapshot;
    const edits = [
      { ...profile, name: "新名称" },
      { ...profile, program: "/path with spaces/katago" },
      { ...profile, working_dir: "/other directory" },
      { ...profile, argv: ["two words", "中文", ""] },
      { ...profile, settings: { ...profile.settings, model_path: "/other/model" } },
      { ...profile, settings: { ...profile.settings, config_path: "/other/config" } },
      { ...profile, settings: { ...profile.settings, max_visits: 1200 } },
      { ...profile, adapter_kind: "generic_gtp" as const, settings: {} }
    ];
    for (const edited of edits) expect(profileHasPendingChanges(edited, ready)).toBe(true);
    expect(ready).toEqual(original);
    const updated = edits[3];
    const promoted = snapshot(2, { state: "ready", run: { ...run, run_id: "run-2", profile_snapshot: updated } });
    expect(profileHasPendingChanges(updated, promoted)).toBe(false);
    const reordered = { ...updated, argv: ["中文", "two words", ""] };
    expect(profileHasPendingChanges(reordered, promoted)).toBe(true);
    expect(profileHasPendingChanges({ ...updated, argv: ["two words", "中文"] }, promoted)).toBe(true);
  });

  it("does not infer analysis capability from an adapter or a Ready state", () => {
    const unverified = { ...run, capability_snapshot: null };
    expect(admitsForegroundEngineJobs(snapshot(1, { state: "ready", run: unverified }))).toBe(false);
    const generic = { ...run, adapter_kind: "generic_gtp" as const, capability_snapshot: { adapter_kind: "generic_gtp" as const, analysis: null } };
    expect(admitsForegroundEngineJobs(snapshot(2, { state: "ready", run: generic }))).toBe(false);
    const refused = { ...run, capability_snapshot: { adapter_kind: "kata_go_analysis" as const, analysis: { selected_node_analysis: false, continuous_analysis: true, whole_game_analysis: true, candidates: true, pv: true, winrate: true, root_score: true, ownership: true, policy: true, visits_limit: true, protocol_cancel: true } } };
    expect(admitsForegroundEngineJobs(snapshot(3, { state: "ready", run: refused }))).toBe(false);
  });
});

const crash: EngineFailureDto = {
  operation: "unexpected_exit",
  run_id: "run-1",
  profile_id: "profile-1",
  kind: "nonzero_exit",
  message: "engine process exited unexpectedly"
};

describe("foreground engine failure presentation", () => {
  it("keeps Error recovery actions available and classifies the snapshot failure", () => {
    const error = snapshot(3, { state: "error", run, failure: crash });
    expect(engineStatusLabel(error)).toBe("引擎错误");
    expect(canStopForegroundEngine(error)).toBe(true);
    expect(canRestartForegroundEngine(error)).toBe(true);
    expect(displayedEngineFailure(error, null)).toEqual(crash);
  });

  it("rejects a stale failure after a newer Ready run identity", () => {
    const ready = snapshot(5, { state: "ready", run: { ...run, run_id: "run-2" } });
    expect(shouldAcceptFailureEvent(ready, null, crash)).toBe(false);
    expect(displayedEngineFailure(ready, crash)).toBeNull();
  });

  it("keeps an attempt-scoped failure on No-engine", () => {
    const asset: EngineFailureDto = {
      operation: "autoload",
      run_id: "attempt-1",
      profile_id: "profile-1",
      kind: "asset",
      message: "required engine assets are missing"
    };
    const missed = snapshot(2, { state: "no_engine", failure: asset });
    expect(shouldAcceptFailureEvent(missed, null, asset)).toBe(true);
    expect(displayedEngineFailure(missed, null)).toEqual(asset);
    expect(displayedEngineFailure(missed, crash)).toEqual(asset);
    expect(shouldAcceptFailureEvent(missed, asset, crash)).toBe(false);
  });

  it("does not revive a stale event failure on a clean No-engine snapshot", () => {
    const empty = emptyForegroundEngineSnapshot();
    const asset: EngineFailureDto = {
      operation: "start",
      profile_id: "profile-1",
      kind: "asset",
      message: "required engine assets are missing"
    };
    expect(displayedEngineFailure(empty, asset)).toBeNull();
  });

  it("shows a switch failure on Ready A without replacing the primary", () => {
    const readyA = snapshot(4, { state: "ready", run });
    const switchFail: EngineFailureDto = {
      operation: "switch",
      run_id: "run-b",
      switch_id: "7",
      profile_id: "profile-b",
      kind: "asset",
      message: "required engine assets are missing"
    };
    expect(shouldAcceptFailureEvent(readyA, null, switchFail, "7")).toBe(true);
    expect(displayedEngineFailure(readyA, switchFail, "7")).toEqual(switchFail);
  });

  it("rejects a stale switch failure after a later switch identity", () => {
    const readyC = snapshot(8, { state: "ready", run: { ...run, run_id: "run-c", profile_id: "profile-c" } });
    const staleB: EngineFailureDto = {
      operation: "switch",
      run_id: "run-b",
      switch_id: "7",
      profile_id: "profile-b",
      kind: "asset",
      message: "required engine assets are missing"
    };
    expect(shouldAcceptFailureEvent(readyC, null, staleB, "8")).toBe(false);
    expect(displayedEngineFailure(readyC, staleB, "8")).toBeNull();
  });

  it("rejects a switch failure once the primary has entered Error", () => {
    const error = snapshot(5, { state: "error", run, failure: crash });
    const switchFail: EngineFailureDto = {
      operation: "switch",
      run_id: "run-b",
      switch_id: "7",
      profile_id: "profile-b",
      kind: "timeout",
      message: "engine readiness probe timed out"
    };
    expect(shouldAcceptFailureEvent(error, crash, switchFail, "7")).toBe(false);
    expect(displayedEngineFailure(error, switchFail, "7")).toEqual(crash);
  });

  it("rejects a switch failure whose run identity is already the Ready primary", () => {
    const readyC = snapshot(8, { state: "ready", run: { ...run, run_id: "run-c", profile_id: "profile-c" } });
    const latePromoted: EngineFailureDto = {
      operation: "switch",
      run_id: "run-c",
      switch_id: "8",
      profile_id: "profile-c",
      kind: "timeout",
      message: "engine readiness probe timed out"
    };
    expect(shouldAcceptFailureEvent(readyC, null, latePromoted, "8")).toBe(false);
    expect(displayedEngineFailure(readyC, latePromoted, "8")).toBeNull();
  });
});
