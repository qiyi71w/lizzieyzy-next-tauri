import { useEffect, useRef, useState } from "react";
import {
  beginProviderRequest,
  cancelProviderRequest,
  fetchTencentList,
  loadTencentHistory,
  previewTencentChess,
  saveTencentQuery
} from "../api/providers";
import type {
  NetworkRoute,
  NetworkSnapshot,
  ProviderImportResult,
  ProviderRequestIdentity,
  TencentHistory,
  TencentListEntry,
  TencentPreviewResult,
  TencentQuery
} from "../domain/providers";

type Props = {
  disabled?: boolean;
  network: NetworkSnapshot | null;
  onImport: (result: ProviderImportResult, identity?: ProviderRequestIdentity) => void | Promise<void>;
};
type ListState = { username: string; rows: TencentListEntry[]; page: number; cursor: string; end: boolean; cursors: string[] };
type Operation = { kind: "list"; username: string; cursor: string; page: number } | { kind: "preview"; chessId: string };
type HistoryWrite = { serial: number; query: TencentQuery | null; inputRevision: number };
const pageSize = 25;
// Accepted preference writes outlive a mounted center and retain action order across remounts.
let historyQueue: Promise<void> = Promise.resolve();
let historyFailure: { query: TencentQuery | null; message: string } | null = null;

export function TencentKifuCenter({ disabled = false, network, onImport }: Props) {
  const [kind, setKind] = useState<TencentQuery["kind"]>("username");
  const [value, setValue] = useState("");
  const [list, setList] = useState<ListState | null>(null);
  const [preview, setPreview] = useState<TencentPreviewResult | null>(null);
  const [history, setHistory] = useState<TencentHistory>({ recent: [], last_query: null });
  const [routes, setRoutes] = useState<NetworkRoute[]>([]);
  const [busy, setBusy] = useState(false);
  const [importing, setImporting] = useState(false);
  const [status, setStatus] = useState("请输入 username 或 chessId。查询和预览不会改变当前棋谱。");
  const [error, setError] = useState("");
  const [failed, setFailed] = useState<Operation | null>(null);
  const [persistenceError, setPersistenceError] = useState("");
  const [historyFailed, setHistoryFailed] = useState(false);
  const [historyLoadFailed, setHistoryLoadFailed] = useState(false);
  const mounted = useRef(false);
  const lifecycle = useRef(0);
  const sequence = useRef(0);
  const identity = useRef<ProviderRequestIdentity | null>(null);
  const listRef = useRef<ListState | null>(null);
  const importingRef = useRef(false);
  const inputRevision = useRef(0);
  const historySerial = useRef(0);
  const historyLoadSequence = useRef(0);
  const desiredWrite = useRef<HistoryWrite | null>(null);
  const clearAfterImport = useRef(false);
  const revision = network?.policy_revision;
  const availability = useRef({ disabled, revision });
  availability.current = { disabled, revision };
  const previousRevision = useRef(revision);

  useEffect(() => {
    mounted.current = true;
    const generation = ++lifecycle.current;
    void loadHistory(generation);
    return () => {
      mounted.current = false;
      lifecycle.current += 1;
      sequence.current += 1;
      const previous = identity.current;
      identity.current = null;
      if (previous) void cancelProviderRequest(previous).catch(() => {});
    };
  }, []);

  useEffect(() => {
    const changed = previousRevision.current !== revision;
    previousRevision.current = revision;
    if (changed || (disabled && !importingRef.current)) {
      invalidate();
      setStatus("网络设置或可用状态已更改；请显式重新查询。当前棋谱未改动。");
    }
  }, [revision, disabled]);

  function updateList(next: ListState | null) {
    listRef.current = next;
    setList(next);
  }

  function invalidate() {
    const current = ++sequence.current;
    const previous = identity.current;
    identity.current = null;
    if (previous) void cancelProviderRequest(previous).catch((cause: unknown) => {
      if (mounted.current && current === sequence.current) setError(`取消查询失败：${errorMessage(cause)}`);
    });
    setBusy(false);
    setPreview(null);
    setRoutes([]);
    setError("");
    setFailed(null);
    return current;
  }

  function isCurrent(current: number, lease: ProviderRequestIdentity) {
    return mounted.current && current === sequence.current && !availability.current.disabled
      && availability.current.revision === lease.policy_revision
      && identity.current !== null && sameIdentity(identity.current, lease);
  }

  async function loadHistory(generation = lifecycle.current) {
    const serial = historySerial.current;
    const current = ++historyLoadSequence.current;
    try {
      await historyQueue;
      const saved = await loadTencentHistory();
      if (!mounted.current || generation !== lifecycle.current || serial !== historySerial.current || current !== historyLoadSequence.current) return;
      setHistory(saved);
      setHistoryLoadFailed(false);
      setPersistenceError("");
      if (inputRevision.current === 0 && saved.last_query) {
        setKind(saved.last_query.kind);
        setValue(saved.last_query.value);
      }
      if (historyFailure) {
        desiredWrite.current = { serial: historySerial.current, query: historyFailure.query, inputRevision: inputRevision.current };
        setHistoryFailed(true);
        setPersistenceError(historyFailure.message);
      }
    } catch (cause) {
      if (!mounted.current || generation !== lifecycle.current || serial !== historySerial.current || current !== historyLoadSequence.current) return;
      setHistoryLoadFailed(true);
      setPersistenceError(`Tencent 查询历史读取失败：${errorMessage(cause)}`);
    }
  }

  function queueHistory(query: TencentQuery | null) {
    if (importingRef.current || availability.current.disabled) return;
    const write: HistoryWrite = { serial: ++historySerial.current, query, inputRevision: inputRevision.current };
    desiredWrite.current = write;
    setHistoryFailed(false);
    setHistoryLoadFailed(false);
    setPersistenceError("");
    const generation = lifecycle.current;
    historyQueue = historyQueue.then(async () => {
      try {
        const saved = await saveTencentQuery(write.query);
        historyFailure = null;
        if (!mounted.current || generation !== lifecycle.current || write.serial !== historySerial.current) return;
        setHistory(saved);
        setHistoryFailed(false);
        setPersistenceError("");
        if (write.query === null && write.inputRevision === inputRevision.current) {
          if (importingRef.current) clearAfterImport.current = true;
          else clearInput();
        }
      } catch (cause) {
        const message = `Tencent 查询历史保存失败：${errorMessage(cause)}。查询结果和当前棋谱不受影响。`;
        historyFailure = { query: write.query, message };
        if (!mounted.current || generation !== lifecycle.current || write.serial !== historySerial.current) return;
        setHistoryFailed(true);
        setPersistenceError(message);
      }
    });
  }

  function clearInput() {
    inputRevision.current += 1;
    invalidate();
    setValue("");
    updateList(null);
    setStatus("Tencent 查询历史已清除；当前棋谱未改动。");
  }

  async function run(operation: Operation) {
    if (availability.current.disabled || importingRef.current || revision === undefined) return;
    const current = invalidate();
    setBusy(true);
    setStatus(operation.kind === "list" ? "正在读取 Tencent 公开列表；可取消或切换查询。" : "正在获取 Tencent 棋谱预览；当前棋谱未改动。");
    try {
      const lease = await beginProviderRequest(revision);
      if (!mounted.current || current !== sequence.current || availability.current.disabled
        || availability.current.revision !== revision || lease.policy_revision !== revision) {
        await cancelProviderRequest(lease);
        return;
      }
      identity.current = lease;
      if (operation.kind === "list") {
        const response = await fetchTencentList(operation.username, operation.cursor, lease);
        if (!isCurrent(current, lease)) return;
        if (!sameIdentity(response.identity, lease)) throw new Error("列表响应标识已失效；请重新查询。");
        const cached = listRef.current;
        if (!cached || cached.username !== operation.username || cached.cursor !== operation.cursor) return;
        const seen = new Set(cached.rows.map((row) => row.chess_id));
        const rows = [...cached.rows];
        for (const row of response.result.games) {
          if (!seen.has(row.chess_id)) {
            seen.add(row.chess_id);
            rows.push(row);
          }
        }
        const nextCursor = response.result.last_code;
        const end = !response.result.has_more || response.result.games.length === 0
          || !nextCursor || cached.cursors.includes(nextCursor);
        updateList({ ...cached, rows, cursor: nextCursor, end,
          page: Math.min(operation.page, Math.max(1, Math.ceil(rows.length / pageSize))),
          cursors: [...cached.cursors, nextCursor] });
        setRoutes(response.routes);
        setStatus(rows.length === cached.rows.length && !end
          ? "本批没有新增棋局；可显式继续读取。不会自动连续请求。"
          : "列表已更新。选局仅预览，导入需另行确认。");
      } else {
        const response = await previewTencentChess(operation.chessId, lease);
        if (!isCurrent(current, lease)) return;
        if (!sameIdentity(response.identity, lease)) throw new Error("预览响应标识已失效；请重新查询。");
        setPreview(response);
        setRoutes(response.routes);
        setStatus("预览就绪；当前棋谱未改动。确认后可导入。");
      }
    } catch (cause) {
      if (!mounted.current || current !== sequence.current || availability.current.disabled || availability.current.revision !== revision) return;
      setError(`查询失败：${errorMessage(cause)}`);
      setFailed(operation);
      setStatus("当前棋谱未改动。可重试本次查询。");
    } finally {
      if (mounted.current && current === sequence.current) setBusy(false);
    }
  }

  function startQuery(query: TencentQuery) {
    if (availability.current.disabled || importingRef.current || revision === undefined || !query.value.trim()) return;
    const trimmed = { ...query, value: query.value.trim() };
    inputRevision.current += 1;
    setKind(trimmed.kind);
    setValue(trimmed.value);
    updateList(trimmed.kind === "username"
      ? { username: trimmed.value, rows: [], page: 1, cursor: "0", end: false, cursors: ["0"] }
      : null);
    void run(trimmed.kind === "username"
      ? { kind: "list", username: trimmed.value, cursor: "0", page: 1 }
      : { kind: "preview", chessId: trimmed.value });
    queueHistory(trimmed);
  }

  function editInput(nextKind: TencentQuery["kind"], nextValue: string) {
    if (importingRef.current || availability.current.disabled) return;
    inputRevision.current += 1;
    invalidate();
    updateList(null);
    setKind(nextKind);
    setValue(nextValue);
    setStatus("查询已更改；请显式查询。当前棋谱未改动。");
  }

  function changePage(page: number) {
    if (importingRef.current || availability.current.disabled || revision === undefined) return;
    const cached = listRef.current;
    if (!cached || page < 1) return;
    if ((page - 1) * pageSize < cached.rows.length) {
      invalidate();
      updateList({ ...cached, page });
      setStatus("已切换缓存页；选局仅预览。当前棋谱未改动。");
    } else if (!cached.end && !busy) {
      const targetPage = cached.rows.length % pageSize === 0 ? page : cached.page;
      void run({ kind: "list", username: cached.username, cursor: cached.cursor, page: targetPage });
    }
  }

  async function importPreview() {
    const lease = identity.current;
    if (!preview || !lease || disabled || busy || importingRef.current || revision !== lease.policy_revision
      || !sameIdentity(preview.identity, lease)) return;
    importingRef.current = true;
    setImporting(true);
    const current = sequence.current;
    try {
      await onImport(preview.result, preview.identity);
      if (mounted.current && current === sequence.current) setStatus("导入请求已交给棋谱确认流程；取消确认会保留当前棋谱。");
    } catch (cause) {
      if (mounted.current && current === sequence.current) setError(`导入失败：${errorMessage(cause)}`);
    } finally {
      importingRef.current = false;
      if (mounted.current) {
        setImporting(false);
        if (clearAfterImport.current) {
          clearAfterImport.current = false;
          clearInput();
        } else if (availability.current.disabled) invalidate();
      }
    }
  }

  const locked = disabled || importing;
  const visibleRows = list?.rows.slice((list.page - 1) * pageSize, list.page * pageSize) ?? [];
  const hasNext = list !== null && (list.page * pageSize < list.rows.length || !list.end);
  const needsMore = list !== null && !list.end && list.page * pageSize >= list.rows.length && list.rows.length % pageSize !== 0;
  return (
    <section className="yike-public-center" aria-label="Tencent 棋谱中心">
      <div className="provider-subheader"><h3>Tencent 棋谱中心</h3></div>
      <div className="provider-grid">
        <label><span>Tencent 查询方式</span><select disabled={locked} value={kind} onChange={(event) => editInput(event.target.value as TencentQuery["kind"], "")}>
          <option value="username">username 列表</option><option value="chess_id">chessId 单局</option>
        </select></label>
        <label><span>{kind === "username" ? "Tencent username" : "Tencent chessId"}</span><input disabled={locked} value={value} placeholder={kind === "username" ? "输入公开 username" : "输入公开 chessId"} onChange={(event) => editInput(kind, event.target.value)} /></label>
        <button disabled={locked || revision === undefined || !value.trim()} onClick={() => startQuery({ kind, value })}>{kind === "username" ? "查询棋局列表" : "预览棋局"}</button>
      </div>
      <div className="yike-query-controls" role="group" aria-label="Tencent 查询历史">
        {history.recent.map((query) => <button key={`${query.kind}:${query.value}`} disabled={locked || revision === undefined} onClick={() => startQuery(query)}>{query.kind === "username" ? "username" : "chessId"}: {query.value}</button>)}
        <button disabled={locked} onClick={() => queueHistory(null)}>清除查询历史</button>
      </div>
      {list ? <>
        <div className="yike-query-controls" role="group" aria-label="列表分页">
          <button disabled={locked || revision === undefined || list.page <= 1} onClick={() => changePage(list.page - 1)}>上一页</button>
          <span>第 {list.page} 页 · 每页最多 25 局</span>
          <button disabled={locked || revision === undefined || !hasNext || needsMore || (busy && list.page * pageSize >= list.rows.length)} onClick={() => changePage(list.page + 1)}>下一页</button>
          {needsMore ? <button disabled={locked || revision === undefined || busy} onClick={() => changePage(list.page + 1)}>加载更多棋局</button> : null}
        </div>
        {visibleRows.length ? <ul className="yike-game-list" aria-label="Tencent 公开棋局">{visibleRows.map((game) => <li key={game.chess_id}>
          <button disabled={locked || revision === undefined} onClick={() => { if (!importingRef.current) void run({ kind: "preview", chessId: game.chess_id }); }}>
            {game.chess_id} · {game.black_name || "黑方"}{game.black_rank ? ` (${game.black_rank})` : ""} vs {game.white_name || "白方"}{game.white_rank ? ` (${game.white_rank})` : ""} · {game.move_count} 手{game.played_at ? ` · ${game.played_at}` : ""}{game.result ? ` · ${game.result}` : ""}
          </button>
        </li>)}</ul> : !busy ? <p role="status">暂无公开棋局。</p> : null}
        {list.end && !hasNext ? <p role="status">已到列表末页。</p> : null}
      </> : null}
      <div className="yike-query-controls">
        <button disabled={importing || (!busy && !preview)} onClick={() => {
          if (importingRef.current) return;
          invalidate();
          setStatus("已取消查询；当前棋谱未改动。");
        }}>取消查询</button>
        {failed ? <button disabled={locked || revision === undefined} onClick={() => void run(failed)}>重试查询</button> : null}
        {historyFailed ? <button disabled={locked} onClick={() => { if (desiredWrite.current) queueHistory(desiredWrite.current.query); }}>重试保存历史</button> : null}
        {historyLoadFailed ? <button disabled={locked} onClick={() => void loadHistory()}>重试读取历史</button> : null}
      </div>
      <p className="provider-status" role="status">{status}</p>
      {error ? <p role="alert">{error}</p> : null}
      {persistenceError ? <p role="alert">{persistenceError}</p> : null}
      {routes.length ? <ul className="yike-read-routes" aria-label="实际网络路由">{routes.map((route, index) => <li key={index}>{route.mode} / {route.source} → {route.target} via {route.proxy ?? "DIRECT"}</li>)}</ul> : null}
      {preview ? <section aria-label="Provider 棋谱预览">
        <p>{preview.result.summary.black_name ?? "黑方"} vs {preview.result.summary.white_name ?? "白方"} · {preview.result.summary.board_width}×{preview.result.summary.board_height} · {preview.result.summary.move_count} 手{preview.result.summary.result ? ` · ${preview.result.summary.result}` : ""}</p>
        {preview.result.warnings.length ? <ul aria-label="Tencent 预览提示">{preview.result.warnings.map((warning, index) => <li key={index}>{warning}</li>)}</ul> : null}
        <button disabled={locked || busy} onClick={() => void importPreview()}>导入预览棋谱</button>
      </section> : null}
    </section>
  );
}

function sameIdentity(left: ProviderRequestIdentity, right: ProviderRequestIdentity) {
  return left.request_id === right.request_id && left.policy_revision === right.policy_revision && left.document_identity === right.document_identity;
}

function errorMessage(cause: unknown): string {
  if (cause instanceof Error) return cause.message;
  if (typeof cause === "object" && cause !== null && "message" in cause && typeof cause.message === "string") return cause.message;
  return String(cause);
}
