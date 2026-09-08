import type { EngineFailureDto, EngineRunDto, ForegroundEngineSnapshotDto } from "./types";

export const emptyForegroundEngineSnapshot = (): ForegroundEngineSnapshotDto => ({
  revision: 0,
  lifecycle: { state: "no_engine" },
  continuous: { enabled: null, phase: "loading" }
});

export function mergeForegroundEngineSnapshot(
  current: ForegroundEngineSnapshotDto | null,
  incoming: ForegroundEngineSnapshotDto
): ForegroundEngineSnapshotDto {
  if (!current || incoming.revision >= current.revision) return incoming;
  return current;
}

export function runFromSnapshot(snapshot: ForegroundEngineSnapshotDto | null): EngineRunDto | null {
  const lifecycle = snapshot?.lifecycle;
  if (!lifecycle) return null;
  if (lifecycle.state === "starting" || lifecycle.state === "ready" || lifecycle.state === "stopping" || lifecycle.state === "error") {
    return lifecycle.run;
  }
  if (lifecycle.state === "switching") return lifecycle.primary;
  return null;
}

export function engineStatusLabel(snapshot: ForegroundEngineSnapshotDto): string {
  switch (snapshot.lifecycle.state) {
    case "no_engine":
      return "未加载引擎";
    case "starting":
      return `正在启动 ${snapshot.lifecycle.run.profile_snapshot.name}`.trim();
    case "ready":
      return snapshot.lifecycle.run.profile_snapshot.name || "已加载引擎";
    case "switching":
      return `正在切换 ${snapshot.lifecycle.candidate.profile_snapshot.name}`.trim();
    case "stopping":
      return "正在停止";
    case "error":
      return "引擎错误";
    default:
      return "未加载引擎";
  }
}

export function isForegroundEngineReady(snapshot: ForegroundEngineSnapshotDto): boolean {
  return snapshot.lifecycle.state === "ready";
}

export function admitsForegroundEngineJobs(snapshot: ForegroundEngineSnapshotDto): boolean {
  return snapshot.lifecycle.state === "ready" || snapshot.lifecycle.state === "switching";
}

export function canStopForegroundEngine(snapshot: ForegroundEngineSnapshotDto): boolean {
  return snapshot.lifecycle.state === "ready"
    || snapshot.lifecycle.state === "switching"
    || snapshot.lifecycle.state === "error"
    || snapshot.lifecycle.state === "starting";
}

export function canRestartForegroundEngine(snapshot: ForegroundEngineSnapshotDto): boolean {
  return snapshot.lifecycle.state === "ready" || snapshot.lifecycle.state === "error";
}

export function profileHasPendingChanges(
  saved: { name: string; engine_path: string; model_path?: string | null; config_path?: string | null; working_dir?: string | null },
  snapshot: ForegroundEngineSnapshotDto
): boolean {
  const run = runFromSnapshot(snapshot);
  if (!run) return false;
  const current = run.profile_snapshot;
  return current.name !== saved.name
    || current.engine_path !== saved.engine_path
    || (current.model_path ?? "") !== (saved.model_path ?? "")
    || (current.config_path ?? "") !== (saved.config_path ?? "")
    || (current.working_dir ?? "") !== (saved.working_dir ?? "");
}

export function shouldAcceptFailureEvent(
  snapshot: ForegroundEngineSnapshotDto,
  currentFailure: { run_id?: string | null; switch_id?: string | null } | null,
  incoming: { run_id?: string | null; switch_id?: string | null; operation?: string },
  lastSwitchId?: string | null
): boolean {
  if (snapshot.lifecycle.state === "error") {
    return incoming.run_id === snapshot.lifecycle.run.run_id;
  }
  const switchId = snapshot.lifecycle.state === "switching"
    ? snapshot.lifecycle.switch_id
    : lastSwitchId ?? null;
  if (incoming.operation === "switch" && incoming.switch_id) {
    if (incoming.switch_id !== switchId) return false;
    const primaryRunId = runFromSnapshot(snapshot)?.run_id ?? null;
    if (primaryRunId && incoming.run_id === primaryRunId) return false;
    return true;
  }
  const currentRunId = runFromSnapshot(snapshot)?.run_id ?? currentFailure?.run_id ?? null;
  if (!incoming.run_id) return snapshot.lifecycle.state === "no_engine";
  if (!currentRunId) return snapshot.lifecycle.state === "no_engine";
  return incoming.run_id === currentRunId;
}

export function displayedEngineFailure(
  snapshot: ForegroundEngineSnapshotDto,
  eventFailure: EngineFailureDto | null,
  lastSwitchId?: string | null
): EngineFailureDto | null {
  if (snapshot.lifecycle.state === "error") return snapshot.lifecycle.failure;
  if (snapshot.lifecycle.state === "no_engine") return snapshot.lifecycle.failure ?? null;
  if (
    snapshot.lifecycle.state === "ready"
    && eventFailure?.operation === "switch"
    && eventFailure.switch_id
    && eventFailure.switch_id === lastSwitchId
    && eventFailure.run_id !== snapshot.lifecycle.run.run_id
  ) {
    return eventFailure;
  }
  return null;
}
