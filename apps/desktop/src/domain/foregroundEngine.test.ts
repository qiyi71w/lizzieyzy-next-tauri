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
    engine_path: "/bin/katago",
    model_path: "/models/model.bin",
    config_path: "/configs/analysis.cfg",
    working_dir: "/tmp",
    backend: "kata_go_analysis"
  },
  capability_snapshot: {
    adapter_kind: "kata_go_analysis",
    selected_node_analysis: true,
    whole_game_analysis: true,
    protocol_cancel: true
  }
};

function snapshot(revision: number, lifecycle: ForegroundEngineSnapshotDto["lifecycle"]): ForegroundEngineSnapshotDto {
  return { revision, lifecycle };
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
    expect(profileHasPendingChanges({ ...run.profile_snapshot, engine_path: "/other/katago" }, ready)).toBe(true);
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
    const empty = emptyForegroundEngineSnapshot();
    const asset: EngineFailureDto = {
      operation: "start",
      run_id: "attempt-1",
      profile_id: "profile-1",
      kind: "asset",
      message: "required engine assets are missing"
    };
    expect(shouldAcceptFailureEvent(empty, null, asset)).toBe(true);
    expect(displayedEngineFailure(empty, asset)).toEqual(asset);
    expect(shouldAcceptFailureEvent(empty, asset, crash)).toBe(false);
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
