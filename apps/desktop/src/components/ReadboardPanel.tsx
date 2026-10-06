import { useCallback, useEffect, useRef, useState } from "react";
import { readboard } from "../api/readboard";
import type { ReadboardPhaseDto, ReadboardRuntimeDto } from "../domain/providers";

export type ReadboardPanelProps = {
  disabled?: boolean;
};

export function phaseLabel(phase: ReadboardPhaseDto): string {
  switch (phase) {
    case "idle":
      return "空闲 (idle)";
    case "starting":
      return "启动中 (starting)";
    case "ready":
      return "就绪 (ready)";
    case "stopping":
      return "停止中 (stopping)";
    case "stopped":
      return "已停止 (stopped)";
    case "unavailable":
      return "不可用 (unavailable)";
    case "incompatible":
      return "协议不兼容 (incompatible)";
    case "timeout":
      return "启动超时 (timeout)";
    case "exited":
      return "已退出 (exited)";
    case "disconnected":
      return "连接断开 (disconnected)";
    case "cleanup_failed":
      return "清理失败 (cleanup_failed)";
  }
}

function actionableDiagnostic(runtime: ReadboardRuntimeDto | null): string | null {
  if (!runtime) return null;
  if (runtime.phase === "cleanup_failed" || (runtime.resources_held && runtime.phase !== "ready" && runtime.phase !== "starting")) {
    return "资源清理失败：旧进程或端口仍被占用，请点击“停止”重新清理。";
  }
  if (runtime.phase === "incompatible") {
    const wire = runtime.wire_version ? `（实际为 ${runtime.wire_version}）` : "";
    return `协议版本不兼容：要求协议版本 220430${wire}。请使用匹配版本的 Readboard。`;
  }
  if (runtime.phase === "timeout") {
    return "启动或握手超时：Readboard 进程未能在规定时间内完成握手。请检查端口是否冲突或安全软件拦截。";
  }
  if (runtime.phase === "unavailable") {
    return "Readboard 运行时不可用：请检查执行文件路径是否有效且为支持的平台运行程序。";
  }
  if (runtime.phase === "exited") {
    return "进程已退出：Readboard 进程已提前退出。";
  }
  if (runtime.phase === "disconnected") {
    return "通信连接已断开：与 Readboard 运行时的连接已中断。";
  }
  if (runtime.message.trim().length > 0) {
    return runtime.message.trim();
  }
  return null;
}

function toErrorMessage(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }
  if (error && typeof error === "object" && "message" in error && typeof error.message === "string") {
    return error.message;
  }
  return String(error);
}

function isRevisionStale(incoming: ReadboardRuntimeDto, current: ReadboardRuntimeDto | null): boolean {
  if (!current) {
    return false;
  }
  if (incoming.generation < current.generation) {
    return true;
  }
  return incoming.generation === current.generation && incoming.revision < current.revision;
}

export function ReadboardPanel({ disabled = false }: ReadboardPanelProps) {
  const [savedPath, setSavedPath] = useState<string | null>(null);
  const [draftPath, setDraftPath] = useState("");
  const [runtime, setRuntime] = useState<ReadboardRuntimeDto | null>(null);
  const [isCommandPending, setIsCommandPending] = useState(false);
  const [stopPending, setStopPending] = useState(false);
  const [actionError, setActionError] = useState<string | null>(null);
  const [actionNotice, setActionNotice] = useState<string | null>(null);

  const hasUserEditedDraftRef = useRef(false);
  const latestRuntimeRef = useRef<ReadboardRuntimeDto | null>(null);
  const pathRevisionRef = useRef(0);

  const applyRuntime = useCallback((incoming: ReadboardRuntimeDto): boolean => {
    if (isRevisionStale(incoming, latestRuntimeRef.current)) {
      return false;
    }
    latestRuntimeRef.current = incoming;
    setRuntime(incoming);
    return true;
  }, []);

  useEffect(() => {
    let mounted = true;
    let unlisten: (() => void) | null = null;

    const pathRevision = pathRevisionRef.current;
    async function init() {
      try {
        unlisten = await readboard.subscribe((incoming) => {
          if (!mounted) return;
          applyRuntime(incoming);
        });
        if (!mounted) { unlisten(); return; }
      } catch (error) {
        if (mounted) {
          setActionError(`订阅状态失败: ${toErrorMessage(error)}`);
        }
      }

      try {
        const initial = await readboard.snapshot();
        if (mounted) {
          applyRuntime(initial);
        }
      } catch (error) {
        if (mounted) {
          setActionError(`获取运行时快照失败: ${toErrorMessage(error)}`);
        }
      }

      try {
        const persisted = await readboard.path();
        if (mounted && pathRevisionRef.current === pathRevision) {
          setSavedPath(persisted);
          if (!hasUserEditedDraftRef.current && persisted !== null) {
            setDraftPath(persisted);
          }
        }
      } catch (error) {
        if (mounted) {
          setActionError(`读取保存路径失败: ${toErrorMessage(error)}`);
        }
      }
    }

    void init();

    return () => {
      mounted = false;
      if (unlisten) {
        unlisten();
      }
    };
  }, [applyRuntime]);

  const normalizedDraft = draftPath.trim();
  const normalizedSaved = (savedPath ?? "").trim();
  const hasUnsavedDraft = normalizedDraft !== normalizedSaved;
  const hasSavedPath = normalizedSaved.length > 0;

  const isStarting = runtime?.phase === "starting";
  const isReady = runtime?.phase === "ready";
  const isStopping = runtime?.phase === "stopping";
  const resourcesHeld = runtime?.resources_held ?? false;
  const cleanupFailed = runtime?.phase === "cleanup_failed";

  const isRunningOrBusy = isStarting || isReady || isStopping || resourcesHeld;

  const canSave = !disabled && !isCommandPending && hasUnsavedDraft;
  const canStart = !disabled && !isCommandPending && !stopPending && !hasUnsavedDraft && hasSavedPath && !isRunningOrBusy;
  const showStop = isStarting || isReady || isStopping || resourcesHeld || cleanupFailed;
  const canStop = !stopPending && !isStopping;
  const canRestart = !disabled && !isCommandPending && !stopPending && !hasUnsavedDraft && isReady;

  async function handleChoosePath() {
    setActionError(null);
    setActionNotice(null);
    try {
      setIsCommandPending(true);
      const chosen = await readboard.choosePath();
      if (chosen !== null) {
        hasUserEditedDraftRef.current = true;
        setDraftPath(chosen);
      }
    } catch (error) {
      setActionError(`选择路径失败: ${toErrorMessage(error)}`);
    } finally {
      setIsCommandPending(false);
    }
  }

  async function handleSavePath() {
    setActionError(null);
    setActionNotice(null);
    try {
      setIsCommandPending(true);
      pathRevisionRef.current += 1;
      const saved = await readboard.savePath(draftPath);
      setSavedPath(saved);
      setDraftPath(saved);
      hasUserEditedDraftRef.current = false;
      setActionNotice("路径已保存。");
    } catch (error) {
      // Failure preserves saved path
      setActionError(`保存路径失败: ${toErrorMessage(error)}`);
    } finally {
      setIsCommandPending(false);
    }
  }

  async function handleStart() {
    setActionError(null);
    setActionNotice(null);
    try {
      setIsCommandPending(true);
      const result = await readboard.start();
      applyRuntime(result);
    } catch (error) {
      setActionError(`启动失败: ${toErrorMessage(error)}`);
    } finally {
      setIsCommandPending(false);
    }
  }

  async function handleStop() {
    setActionError(null);
    setActionNotice(null);
    try {
      setStopPending(true);
      const result = await readboard.stop();
      applyRuntime(result);
    } catch (error) {
      setActionError(`停止失败: ${toErrorMessage(error)}`);
    } finally {
      setStopPending(false);
    }
  }

  async function handleRestart() {
    setActionError(null);
    setActionNotice(null);
    try {
      setIsCommandPending(true);
      const result = await readboard.restart();
      applyRuntime(result);
    } catch (error) {
      setActionError(`重启失败: ${toErrorMessage(error)}`);
    } finally {
      setIsCommandPending(false);
    }
  }

  const diagnostic = actionableDiagnostic(runtime);
  const statusSummary = runtime ? `状态: ${phaseLabel(runtime.phase)}` : "未初始化";

  return (
    <div className="provider-readboard">
      <div className="provider-subheader">
        <h3>Readboard 运行时</h3>
        <span title={statusSummary}>{statusSummary}</span>
      </div>

      <div className="provider-grid">
        <label>
          <span>可执行程序路径</span>
          <input
            value={draftPath}
            disabled={disabled || isCommandPending}
            placeholder="输入或选择 Readboard.exe 路径"
            aria-label="Readboard 可执行程序路径"
            onChange={(event) => {
              hasUserEditedDraftRef.current = true;
              setDraftPath(event.target.value);
              setActionError(null);
              setActionNotice(null);
            }}
          />
        </label>
        <div style={{ display: "flex", gap: "6px" }}>
          <button
            type="button"
            onClick={() => void handleChoosePath()}
            disabled={disabled || isCommandPending}
          >
            浏览...
          </button>
          <button
            type="button"
            onClick={() => void handleSavePath()}
            disabled={!canSave}
            title={hasUnsavedDraft ? "保存当前路径草稿" : "路径已是最新保存值"}
          >
            保存路径
          </button>
        </div>
      </div>

      {hasUnsavedDraft ? (
        <p className="provider-status">存在未保存的路径草稿；启动必须使用已保存路径，请先保存。</p>
      ) : null}

      <div style={{ display: "flex", gap: "8px", alignItems: "center", flexWrap: "wrap" }}>
        <button
          type="button"
          className="primary"
          onClick={() => void handleStart()}
          disabled={!canStart}
          title={
            hasUnsavedDraft
              ? "存在未保存的路径草稿，请先保存"
              : !hasSavedPath
                ? "请先设置并保存可执行程序路径"
                : undefined
          }
        >
          启动
        </button>

        {showStop ? (
          <button
            type="button"
            onClick={() => void handleStop()}
            disabled={!canStop}
          >
            {isStarting ? "取消启动" : "停止"}
          </button>
        ) : null}

        {isReady ? (
          <button
            type="button"
            onClick={() => void handleRestart()}
            disabled={!canRestart}
          >
            重启
          </button>
        ) : null}
      </div>
      <p className="provider-status">就绪仅表示进程与协议握手完成，不导入或同步当前棋谱；无需引擎。图像 OCR 不受支持。</p>

      <dl className="provider-preview" aria-label="Readboard 运行时状态">
        <div>
          <dt>状态</dt>
          <dd data-testid="readboard-phase">{runtime ? phaseLabel(runtime.phase) : "未初始化"}</dd>
        </div>
        <div>
          <dt>协议</dt>
          <dd data-testid="readboard-wire-version">{runtime?.wire_version ?? "无"}</dd>
        </div>
        <div>
          <dt>端点</dt>
          <dd data-testid="readboard-endpoint">{runtime?.endpoint ?? "无"}</dd>
        </div>
        <div>
          <dt>进程</dt>
          <dd data-testid="readboard-pid">
            {runtime?.process_id !== null && runtime?.process_id !== undefined ? String(runtime.process_id) : "无"}
          </dd>
        </div>
        <div>
          <dt>资源</dt>
          <dd data-testid="readboard-resources">{runtime?.resources_held ? "已占用" : "已释放"}</dd>
        </div>
        <div>
          <dt>代次</dt>
          <dd data-testid="readboard-generation">
            {runtime ? `${runtime.generation} / ${runtime.revision}` : "无"}
          </dd>
        </div>
      </dl>
      {runtime?.message ? <p className="provider-status">{runtime.message}</p> : null}

      {diagnostic ? (
        <div className="warning-list" role="status" aria-label="诊断信息">
          <strong>Readboard 诊断</strong>
          <p style={{ margin: 0 }}>{diagnostic}</p>
        </div>
      ) : null}

      {actionError ? (
        <div className="warning-list" role="alert" aria-label="错误信息">
          <strong>操作错误</strong>
          <p style={{ margin: 0 }}>{actionError}</p>
        </div>
      ) : null}

      {actionNotice ? (
        <p className="provider-status">{actionNotice}</p>
      ) : null}
    </div>
  );
}
