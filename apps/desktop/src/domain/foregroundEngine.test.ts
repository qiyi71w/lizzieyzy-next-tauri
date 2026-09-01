import { describe, expect, it } from "vitest";
import {
  emptyForegroundEngineSnapshot,
  engineStatusLabel,
  isForegroundEngineReady,
  mergeForegroundEngineSnapshot,
  profileHasPendingChanges
} from "./foregroundEngine";
import type { EngineRunDto, ForegroundEngineSnapshotDto } from "./types";

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

  it("reports pending changes against the immutable run snapshot", () => {
    const ready = snapshot(1, { state: "ready", run });
    expect(profileHasPendingChanges(run.profile_snapshot, ready)).toBe(false);
    expect(profileHasPendingChanges({ ...run.profile_snapshot, engine_path: "/other/katago" }, ready)).toBe(true);
  });
});
