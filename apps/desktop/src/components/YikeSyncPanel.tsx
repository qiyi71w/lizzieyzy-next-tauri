import { useEffect, useRef, useState } from "react";
import { loadYikeSyncPreferences, saveYikeSyncPreferences } from "../api/externalSync";
import type { ExternalSyncSnapshot, YikeSyncPreferences } from "../domain/providers";

export type YikeSyncPanelProps = {
  snapshot: ExternalSyncSnapshot | null;
  disabled?: boolean;
  starting: boolean;
  onStart: (locator: string, play: boolean) => Promise<void>;
  onCancelStart: () => Promise<void>;
  onRetry: () => Promise<void>;
  onStop: () => Promise<void>;
  onOpenBrowser: () => Promise<void>;
};

export function validateInterval(raw: string): { valid: true; value: number } | { valid: false; error: string } {
  const trimmed = raw.trim();
  if (!trimmed || !/^\d+$/.test(trimmed)) {
    return { valid: false, error: "刷新间隔必须是正整数（1 到 4294967295）。" };
  }
  const num = Number(trimmed);
  if (!Number.isSafeInteger(num) || num <= 0 || num > 4294967295) {
    return { valid: false, error: "刷新间隔必须是正整数（1 到 4294967295）。" };
  }
  return { valid: true, value: num };
}

function errorMessage(error: unknown): string {
  if (error instanceof Error) return error.message;
  if (typeof error === "object" && error !== null && "message" in error) {
    const message = error.message;
    if (typeof message === "string") return message;
  }
  return String(error);
}

function formatPhase(snapshot: ExternalSyncSnapshot | null, isStarting: boolean): string {
  if (isStarting) return "Starting";
  if (!snapshot) return "Unavailable";
  switch (snapshot.phase) {
    case "idle":
      return "Idle";
    case "starting":
      return "Starting";
    case "syncing":
      return "Syncing";
    case "retrying":
      return `Retrying (${snapshot.retry_count})`;
    case "error_paused":
      return "Error-paused";
    default:
      return "Idle";
  }
}

export function YikeSyncPanel({
  snapshot,
  disabled = false,
  starting,
  onStart,
  onCancelStart,
  onRetry,
  onStop,
  onOpenBrowser
}: YikeSyncPanelProps) {
  const [preferencesLoaded, setPreferencesLoaded] = useState(false);
  const [loadError, setLoadError] = useState("");

  const [draftInterval, setDraftInterval] = useState("1");
  const [draftJumpToLast, setDraftJumpToLast] = useState(false);
  const [draftMute, setDraftMute] = useState(false);
  const [draftLocator, setDraftLocator] = useState("");

  const [durableLocator, setDurableLocator] = useState<string | null>(null);
  const locatorInitialized = useRef(false);

  const [actionPending, setActionPending] = useState(false);
  const [localStarting, setLocalStarting] = useState(false);
  const startCancelled = useRef(false);

  const [validationError, setValidationError] = useState("");
  const [saveError, setSaveError] = useState("");
  const [actionError, setActionError] = useState("");

  useEffect(() => {
    let active = true;
    loadYikeSyncPreferences()
      .then((prefs: YikeSyncPreferences) => {
        if (!active) return;
        setPreferencesLoaded(true);
        setLoadError("");
        setDraftInterval(String(prefs.intervalSeconds ?? 1));
        setDraftJumpToLast(Boolean(prefs.jumpToLast));
        setDraftMute(Boolean(prefs.mute));
        if (!locatorInitialized.current) {
          locatorInitialized.current = true;
          setDraftLocator(prefs.locator ?? "");
        }
        setDurableLocator(prefs.locator ?? null);
      })
      .catch((err: unknown) => {
        if (!active) return;
        setPreferencesLoaded(false);
        setLoadError(`加载偏好设置失败：${errorMessage(err)}`);
      });

    return () => {
      active = false;
    };
  }, []);

  useEffect(() => {
    const backendLocator = snapshot?.preferences?.locator ?? snapshot?.locator;
    if (backendLocator) {
      setDurableLocator(backendLocator);
    }
  }, [snapshot?.preferences?.locator, snapshot?.locator]);

  const isStarting = starting || snapshot?.phase === "starting" || localStarting;
  const isControlsDisabled = disabled || !preferencesLoaded || snapshot === null || actionPending || isStarting;

  const currentLocator = draftLocator.trim() || snapshot?.locator || "";
  const isUnite = currentLocator.includes("/#/unite/");

  const intervalValidation = validateInterval(draftInterval);
  const intervalNum = intervalValidation.valid ? intervalValidation.value : 1;
  const effectiveInterval = isUnite ? Math.max(intervalNum, 5) : Math.max(intervalNum, 1);

  const phase = isStarting ? "starting" : (snapshot?.phase ?? "idle");
  const isErrorPaused = phase === "error_paused";
  const isActive = phase === "syncing" || phase === "retrying" || phase === "error_paused";

  async function handleSaveSettings() {
    if (isControlsDisabled) return;
    const validation = validateInterval(draftInterval);
    if (!validation.valid) {
      setValidationError(validation.error);
      return;
    }
    setValidationError("");
    setSaveError("");
    setActionError("");
    setActionPending(true);

    try {
      const saved = await saveYikeSyncPreferences({
        intervalSeconds: validation.value,
        jumpToLast: draftJumpToLast,
        mute: draftMute,
        locator: durableLocator
      });
      setDurableLocator(saved.locator ?? durableLocator);
      setSaveError("");
    } catch (err: unknown) {
      setSaveError(`保存偏好设置失败：${errorMessage(err)}`);
    } finally {
      setActionPending(false);
    }
  }

  async function handleStart(play: boolean) {
    if (isControlsDisabled) return;
    const trimmed = draftLocator.trim();
    if (!trimmed) {
      setValidationError("请输入公共棋局 URL。");
      return;
    }
    const validation = validateInterval(draftInterval);
    if (!validation.valid) {
      setValidationError(validation.error);
      return;
    }
    setValidationError("");
    setSaveError("");
    setActionError("");
    setActionPending(true);
    startCancelled.current = false;
    setLocalStarting(true);

    try {
      await saveYikeSyncPreferences({
        intervalSeconds: validation.value,
        jumpToLast: draftJumpToLast,
        mute: draftMute,
        locator: durableLocator
      });
    } catch (err: unknown) {
      setSaveError(`保存偏好设置失败：${errorMessage(err)}`);
      setActionPending(false);
      setLocalStarting(false);
      return;
    }

    try {
      if (!startCancelled.current) await onStart(trimmed, play);
    } catch (err: unknown) {
      setActionError(`启动同步失败：${errorMessage(err)}`);
    } finally {
      setActionPending(false);
      setLocalStarting(false);
    }
  }

  async function handleCancelStart() {
    if (disabled) return;
    startCancelled.current = true;
    setActionError("");
    try {
      await onCancelStart();
    } catch (err: unknown) {
      setActionError(`取消启动失败：${errorMessage(err)}`);
    }
  }

  async function handleRetry() {
    if (isControlsDisabled) return;
    setActionError("");
    setActionPending(true);
    try {
      await onRetry();
    } catch (err: unknown) {
      setActionError(`重试失败：${errorMessage(err)}`);
    } finally {
      setActionPending(false);
    }
  }

  async function handleStop() {
    if (isControlsDisabled) return;
    setActionError("");
    setActionPending(true);
    try {
      await onStop();
    } catch (err: unknown) {
      setActionError(`停止失败：${errorMessage(err)}`);
    } finally {
      setActionPending(false);
    }
  }

  async function handleOpenBrowser() {
    if (isControlsDisabled) return;
    setActionError("");
    setActionPending(true);
    try {
      await onOpenBrowser();
    } catch (err: unknown) {
      setActionError(`打开浏览器失败：${errorMessage(err)}`);
    } finally {
      setActionPending(false);
    }
  }

  const phaseLabel = formatPhase(snapshot, isStarting);

  return (
    <section className="provider-panel yike-sync-panel" aria-label="Yike 持续同步">
      <div className="provider-subheader">
        <h3>Yike 持续同步</h3>
        <span className="sync-phase-wrapper">
          状态：<strong data-testid="sync-phase">{phaseLabel}</strong>
        </span>
      </div>

      {loadError ? (
        <p role="alert" className="provider-status" data-testid="load-error">
          {loadError}
        </p>
      ) : null}

      {snapshot === null ? (
        <p className="provider-status" data-testid="snapshot-unavailable-notice">
          持续同步原生环境不可用或未提供快照。
        </p>
      ) : null}

      <div className="yike-query-controls" style={{ display: "grid", gap: "8px" }}>
        <label>
          <span>棋局 URL</span>
          <input
            type="text"
            aria-label="棋局 URL"
            placeholder="https://home.yikeweiqi.com/#/..."
            value={draftLocator}
            disabled={isControlsDisabled}
            data-testid="locator-input"
            onChange={(event) => {
              setDraftLocator(event.target.value);
              setValidationError("");
            }}
          />
        </label>

        <div className="provider-status" data-testid="effective-poll-interval">
          有效轮询间隔: <strong>{effectiveInterval} 秒</strong>
          {isUnite && intervalNum < 5
            ? "（unite 最小 5 秒，生效 5 秒）"
            : isUnite
            ? "（unite 最小 5 秒）"
            : "（默认最小 1 秒）"}
        </div>

        <label>
          <span>刷新间隔（秒）</span>
          <input
            type="number"
            aria-label="刷新间隔（秒）"
            min={1}
            max={4294967295}
            step={1}
            value={draftInterval}
            disabled={isControlsDisabled}
            data-testid="interval-input"
            onChange={(event) => {
              setDraftInterval(event.target.value);
              setValidationError("");
            }}
          />
        </label>

        <label className="toggle-row">
          <span>跳转到最新手</span>
          <input
            type="checkbox"
            aria-label="跳转到最新手"
            checked={draftJumpToLast}
            disabled={isControlsDisabled}
            data-testid="jump-to-last-input"
            onChange={(event) => setDraftJumpToLast(event.target.checked)}
          />
        </label>

        <label className="toggle-row">
          <span>同步静音</span>
          <input
            type="checkbox"
            aria-label="同步静音"
            checked={draftMute}
            disabled={isControlsDisabled}
            data-testid="mute-input"
            onChange={(event) => setDraftMute(event.target.checked)}
          />
        </label>

        <div className="yike-query-controls" role="group" aria-label="偏好设置控制">
          <button
            type="button"
            disabled={isControlsDisabled}
            onClick={() => void handleSaveSettings()}
            data-testid="save-button"
          >
            保存设置
          </button>
          {saveError ? (
            <button
              type="button"
              disabled={isControlsDisabled}
              onClick={() => void handleSaveSettings()}
              data-testid="retry-save-button"
            >
              重试保存
            </button>
          ) : null}
        </div>
      </div>

      {isStarting ? (
        <div className="yike-query-controls" role="group" aria-label="启动控制">
          <button
            type="button"
            disabled={disabled}
            onClick={() => void handleCancelStart()}
            data-testid="cancel-button"
          >
            取消启动
          </button>
          <button
            type="button"
            disabled={true}
            data-testid="start-button"
          >
            开始同步
          </button>
          <button
            type="button"
            disabled={true}
            data-testid="play-sync-button"
          >
            Play & Sync
          </button>
        </div>
      ) : isErrorPaused ? (
        <div className="yike-query-controls" role="group" aria-label="同步控制">
          <button
            type="button"
            disabled={isControlsDisabled}
            onClick={() => void handleRetry()}
            data-testid="retry-button"
          >
            重试同步
          </button>
          <button
            type="button"
            disabled={isControlsDisabled}
            onClick={() => void handleStop()}
            data-testid="stop-button"
          >
            停止同步
          </button>
          <button
            type="button"
            disabled={isControlsDisabled}
            onClick={() => void handleOpenBrowser()}
            data-testid="open-browser-button"
          >
            在浏览器打开
          </button>
        </div>
      ) : isActive ? (
        <div className="yike-query-controls" role="group" aria-label="同步控制">
          <button
            type="button"
            disabled={isControlsDisabled}
            onClick={() => void handleStop()}
            data-testid="stop-button"
          >
            停止同步
          </button>
          <button
            type="button"
            disabled={isControlsDisabled}
            onClick={() => void handleOpenBrowser()}
            data-testid="open-browser-button"
          >
            在浏览器打开
          </button>
        </div>
      ) : (
        <div className="yike-query-controls" role="group" aria-label="启动控制">
          <button
            type="button"
            disabled={isControlsDisabled}
            onClick={() => void handleStart(false)}
            data-testid="start-button"
          >
            开始同步
          </button>
          <button
            type="button"
            disabled={isControlsDisabled}
            onClick={() => void handleStart(true)}
            data-testid="play-sync-button"
          >
            Play & Sync
          </button>
        </div>
      )}

      <div className="provider-status">
        当前活跃来源：<span data-testid="active-locator">{snapshot?.locator ?? "无"}</span>
      </div>
      <div className="provider-status">
        来源状态：<span data-testid="source-status">{snapshot?.source_status ?? "无"}</span>
      </div>

      <dl className="provider-preview" aria-label="只读同步信息">
        <div>
          <dt>棋谱代次</dt>
          <dd data-testid="document-identity">{snapshot?.document_identity ?? "-"}</dd>
        </div>
        <div>
          <dt>会话 ID</dt>
          <dd data-testid="session-id">{snapshot?.session_id ?? "-"}</dd>
        </div>
        <div>
          <dt>修订号</dt>
          <dd data-testid="revision">{snapshot?.revision ?? "-"}</dd>
        </div>
        <div>
          <dt>重试次数</dt>
          <dd data-testid="retry-count">{snapshot?.retry_count ?? 0}</dd>
        </div>
        <div>
          <dt>来源末端</dt>
          <dd data-testid="source-tip">
            {snapshot?.source_tip
              ? snapshot.source_tip.indices.length > 0
                ? snapshot.source_tip.indices.join(",")
                : "0"
              : "-"}
          </dd>
        </div>
      </dl>

      {snapshot?.failure ? (
        <p role="alert" className="provider-status" data-testid="backend-failure">
          失败 [{snapshot.failure.kind}]: {snapshot.failure.message}
        </p>
      ) : null}

      {snapshot?.browser_error ? (
        <p role="alert" className="provider-status" data-testid="browser-error">
          浏览器错误: {snapshot.browser_error}
        </p>
      ) : null}

      {validationError ? (
        <p role="alert" className="provider-status" data-testid="validation-error">
          {validationError}
        </p>
      ) : null}

      {saveError ? (
        <p role="alert" className="provider-status" data-testid="save-error">
          {saveError}
        </p>
      ) : null}

      {actionError ? (
        <p role="alert" className="provider-status" data-testid="action-error">
          {actionError}
        </p>
      ) : null}
    </section>
  );
}
