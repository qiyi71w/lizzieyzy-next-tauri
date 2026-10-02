import type { EngineAnalysisCapabilitiesDto, EngineFailureDto, EngineProfileDto, EngineRunDto, ForegroundEngineSnapshotDto } from "./types";

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

export function admitsForegroundEngineJobs(
  snapshot: ForegroundEngineSnapshotDto,
  capability: keyof EngineAnalysisCapabilitiesDto = "selected_node_analysis"
): boolean {
  return (snapshot.lifecycle.state === "ready" || snapshot.lifecycle.state === "switching")
    && runFromSnapshot(snapshot)?.capability_snapshot?.analysis?.[capability] === true;
}

export function admitsForegroundEngineQuery(
  snapshot: ForegroundEngineSnapshotDto,
  capability: "selected_node_analysis" | "continuous_analysis" | "whole_game_analysis" = "selected_node_analysis"
): boolean {
  const analysis = runFromSnapshot(snapshot)?.capability_snapshot?.analysis;
  // The current gateway queries always request ownership and policy together.
  return admitsForegroundEngineJobs(snapshot, capability) && analysis?.ownership === true && analysis.policy === true;
}

export function verifiedEngineCapabilitiesLabel(snapshot: ForegroundEngineSnapshotDto): string {
  const run = runFromSnapshot(snapshot);
  if (!run) return "当前没有运行引擎";
  const capabilities = run.capability_snapshot;
  if (!capabilities) return "当前 run：能力待验证";
  if (capabilities.gtp) {
    const facts = capabilities.gtp;
    const clocks = ["time_settings", "time_left"]
      .map((command) => `${command} ${facts.commands.includes(command) ? "已发现" : "未发现"}`).join(" · ");
    return `当前 run 已验证：${facts.name} ${facts.version} · GTP v${facts.protocol_version}；${clocks}；已发现命令：${facts.commands.join(", ")}。时钟、setup、rules 命令的存在不代表已支持时间映射、准确规则或任意局面；不提供 rich-analysis，公共取步操作尚未实现。`;
  }
  if (!capabilities.analysis) return "当前 run 已验证：不支持分析";
  const labels: [keyof EngineAnalysisCapabilitiesDto, string][] = [
    ["selected_node_analysis", "单点"], ["continuous_analysis", "连续"],
    ["whole_game_analysis", "整谱/task"], ["candidates", "候选"], ["pv", "PV"],
    ["winrate", "胜率"], ["root_score", "分数"], ["ownership", "ownership"],
    ["policy", "policy"], ["visits_limit", "visits 限制"], ["protocol_cancel", "协议取消"]
  ];
  return `当前 run 已验证：${labels.map(([key, label]) => `${label} ${capabilities.analysis![key] ? "支持" : "不支持"}`).join(" · ")}`;
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
  saved: EngineProfileDto,
  snapshot: ForegroundEngineSnapshotDto
): boolean {
  const run = runFromSnapshot(snapshot);
  if (!run) return false;
  const current = run.profile_snapshot;
  if (current.name !== saved.name || current.program !== saved.program
    || current.working_dir !== saved.working_dir || current.adapter_kind !== saved.adapter_kind
    || current.argv.length !== saved.argv.length
    || current.argv.some((argument, index) => argument !== saved.argv[index])) return true;
  if (current.adapter_kind === "kata_go_analysis" && saved.adapter_kind === "kata_go_analysis") {
    return current.settings.model_path !== saved.settings.model_path
      || current.settings.config_path !== saved.settings.config_path
      || current.settings.max_visits !== saved.settings.max_visits;
  }
  return false;
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
