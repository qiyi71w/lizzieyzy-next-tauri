import type { EngineRunDto, ForegroundEngineSnapshotDto } from "./types";

export const emptyForegroundEngineSnapshot = (): ForegroundEngineSnapshotDto => ({
  revision: 0,
  lifecycle: { state: "no_engine" }
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
