import { useEffect, useState } from "react";
import { loadReadboardSyncPreferences, saveReadboardSyncPreferences } from "../api/externalSync";
import type { ExternalSyncSnapshot, ReadboardSyncPreferences } from "../domain/providers";

export type ReadboardSyncPanelProps = {
  /** Owner snapshot narrowed to readboard; idle when another source owns the game. */
  snapshot: ExternalSyncSnapshot | null;
  disabled?: boolean;
  starting: boolean;
  onStart: () => Promise<void>;
  onCancelStart: () => Promise<void>;
  onRetry: () => Promise<void>;
  onStop: () => Promise<void>;
};

const PREFERENCE_LABELS: [keyof ReadboardSyncPreferences, string][] = [
  ["alwaysSync", "始终跟随：查看来源末端时随新盘面前进"],
  ["focus", "预览候选变化时让 readboard 让出焦点"],
  ["mute", "同步时不播放落子声"],
  ["jumpToLast", "来源更新时总是跳到最新局面"]
];

const PHASE_LABELS: Record<ExternalSyncSnapshot["phase"], string> = {
  idle: "未同步",
  starting: "等待 readboard 盘面",
  syncing: "同步中",
  retrying: "重新连接中",
  error_paused: "已暂停（错误）"
};

function errorMessage(error: unknown): string {
  if (error instanceof Error) return error.message;
  if (typeof error === "object" && error !== null && "message" in error && typeof error.message === "string") return error.message;
  return String(error);
}

export function ReadboardSyncPanel({ snapshot, disabled = false, starting, onStart, onCancelStart, onRetry, onStop }: ReadboardSyncPanelProps) {
  const [preferences, setPreferences] = useState<ReadboardSyncPreferences | null>(null);
  const [error, setError] = useState("");
  const [pending, setPending] = useState(false);

  useEffect(() => {
    let active = true;
    loadReadboardSyncPreferences()
      .then((loaded) => { if (active) setPreferences(loaded); })
      .catch((cause: unknown) => { if (active) setError(`加载 readboard 同步设置失败：${errorMessage(cause)}`); });
    return () => { active = false; };
  }, []);

  const phase = starting ? "starting" : snapshot?.phase ?? "idle";
  const active = phase === "syncing" || phase === "retrying" || phase === "error_paused";
  const busy = disabled || pending || snapshot === null;

  async function run(action: () => Promise<void>, label: string) {
    setError("");
    setPending(true);
    try {
      await action();
    } catch (cause: unknown) {
      setError(`${label}失败：${errorMessage(cause)}`);
    } finally {
      setPending(false);
    }
  }

  async function toggle(key: keyof ReadboardSyncPreferences, value: boolean) {
    if (!preferences) return;
    const previous = preferences;
    const next = { ...preferences, [key]: value };
    setPreferences(next);
    await run(async () => {
      try {
        setPreferences(await saveReadboardSyncPreferences(next));
      } catch (cause: unknown) {
        setPreferences(previous);
        throw cause;
      }
    }, "保存设置");
  }

  const status = snapshot?.readboard;
  return (
    <section className="provider-panel readboard-sync-panel" aria-label="readboard 持续同步">
      <div className="provider-subheader">
        <h3>readboard 持续同步</h3>
        <span>状态：<strong data-testid="readboard-sync-phase">{PHASE_LABELS[phase]}</strong></span>
      </div>
      <p className="provider-status">
        在 readboard 中选择目标窗口并点击同步。Next 只读接收结构化盘面，不回写落子；Stop 后棋谱恢复可编辑。
      </p>
      {status?.source_move_number != null && active ? (
        <p className="provider-status" data-testid="readboard-source-move">来源手数：{status.source_move_number}</p>
      ) : null}
      {snapshot?.source_status ? <p className="provider-status" data-testid="readboard-sync-status">{snapshot.source_status}</p> : null}
      {snapshot?.failure ? <p role="alert" className="provider-status">{snapshot.failure.message}</p> : null}
      {error ? <p role="alert" className="provider-status">{error}</p> : null}
      <fieldset className="readboard-sync-preferences" disabled={!preferences || pending}>
        {PREFERENCE_LABELS.map(([key, label]) => (
          <label key={key}>
            <input type="checkbox" checked={preferences?.[key] ?? false} onChange={(event) => void toggle(key, event.target.checked)} />
            <span>{label}</span>
          </label>
        ))}
      </fieldset>
      <div className="provider-actions">
        {phase === "starting" ? (
          <button type="button" disabled={disabled} onClick={() => void run(onCancelStart, "取消启动")}>取消启动</button>
        ) : active ? (
          <>
            {phase === "error_paused" ? <button type="button" disabled={busy} onClick={() => void run(onRetry, "重试")}>重试</button> : null}
            <button type="button" disabled={busy} onClick={() => void run(onStop, "停止")}>停止同步</button>
          </>
        ) : (
          <button type="button" disabled={busy || !preferences} onClick={() => void run(onStart, "开始同步")}>开始同步</button>
        )}
      </div>
    </section>
  );
}
