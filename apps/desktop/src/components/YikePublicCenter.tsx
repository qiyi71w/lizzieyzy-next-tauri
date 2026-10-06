import { useEffect, useRef, useState } from "react";
import {
  beginProviderRequest,
  cancelProviderRequest,
  fetchYikeList,
  loadYikeLocator,
  previewYikeLocator,
  saveYikeLocator
} from "../api/providers";
import type {
  NetworkRoute,
  NetworkSnapshot,
  ProviderImportResult,
  ProviderRequestIdentity,
  YikeCategory,
  YikeListPage,
  YikePreviewResult
} from "../domain/providers";

type Props = {
  disabled?: boolean;
  network: NetworkSnapshot | null;
  onImport: (result: ProviderImportResult, identity?: ProviderRequestIdentity) => void | Promise<void>;
  onStartSync?: (locator: string, play: boolean) => Promise<void>;
};
type ListQuery = { kind: "list"; category: YikeCategory; page: number; since: number };
type Operation = ListQuery | { kind: "preview"; locator: string };
const firstQuery: ListQuery = { kind: "list", category: "recommend", page: 1, since: 0 };

export function YikePublicCenter({ disabled = false, network, onImport, onStartSync }: Props) {
  const [query, setQuery] = useState(firstQuery);
  const [list, setList] = useState<YikeListPage | null>(null);
  const [locator, setLocator] = useState("");
  const [preview, setPreview] = useState<YikePreviewResult | null>(null);
  const [routes, setRoutes] = useState<NetworkRoute[]>([]);
  const [busy, setBusy] = useState(false);
  const [importing, setImporting] = useState(false);
  const [status, setStatus] = useState("等待网络设置，默认打开 Recommend。预览不会改变当前棋谱。");
  const [error, setError] = useState("");
  const [persistenceError, setPersistenceError] = useState("");
  const [failed, setFailed] = useState<Operation | null>(null);
  const sequence = useRef(0);
  const identity = useRef<ProviderRequestIdentity | null>(null);
  const mounted = useRef(false);
  const editedLocator = useRef(false);
  const importingRef = useRef(false);
  const openedRevision = useRef<number | null>(null);
  const queryRef = useRef(firstQuery);
  const cursor = useRef(0);
  const revision = network?.policy_revision;

  useEffect(() => {
    mounted.current = true;
    void loadYikeLocator().then((saved) => {
      if (mounted.current && !editedLocator.current && saved) setLocator(saved);
    }).catch((cause: unknown) => {
      if (mounted.current && !editedLocator.current) setPersistenceError(`无法读取上次的公开 URL：${errorMessage(cause)}`);
    });
    return () => {
      mounted.current = false;
      sequence.current += 1;
      const previous = identity.current;
      identity.current = null;
      if (previous) void cancelProviderRequest(previous).catch(() => {});
    };
  }, []);

  useEffect(() => {
    if (disabled && !importingRef.current) {
      invalidate();
      return;
    }
    if (revision === undefined || openedRevision.current === revision) return;
    if (disabled || importingRef.current) {
      invalidate();
      return;
    }
    openedRevision.current = revision;
    void run(queryRef.current);
  }, [revision, disabled]);

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

  async function run(operation: Operation) {
    if (disabled || importingRef.current || revision === undefined) return;
    const current = invalidate();
    setBusy(true);
    setStatus(operation.kind === "list" ? "正在读取公开列表；可取消或切换查询。" : "正在获取棋谱预览；当前棋谱未改动。");
    if (operation.kind === "list") {
      queryRef.current = operation;
      setQuery(operation);
    }
    try {
      const lease = await beginProviderRequest(revision);
      if (!mounted.current || current !== sequence.current) {
        await cancelProviderRequest(lease);
        return;
      }
      identity.current = lease;
      if (operation.kind === "list") {
        const response = await fetchYikeList(operation.category, operation.page, operation.since, lease);
        if (!mounted.current || current !== sequence.current || !sameIdentity(response.identity, lease)) return;
        const seen = new Set<string>();
        setList({ ...response.result, games: response.result.games.filter((game) => {
          if (seen.has(game.locator)) return false;
          seen.add(game.locator);
          return true;
        }) });
        cursor.current = response.result.since;
        setRoutes(response.routes);
        setStatus("列表已更新。选局仅预览，导入需另行确认。");
      } else {
        const response = await previewYikeLocator(operation.locator, lease);
        if (!mounted.current || current !== sequence.current || !sameIdentity(response.identity, lease)) return;
        setPreview(response);
        setRoutes(response.routes);
        setStatus("预览就绪；当前棋谱未改动。确认后可导入。");
        const canonical = response.result.metadata.source_url;
        if (canonical) {
          setPersistenceError("");
          void saveYikeLocator(canonical).catch((cause: unknown) => {
            if (mounted.current && current === sequence.current) setPersistenceError(`公开 URL 保存失败：${errorMessage(cause)}。预览仍可导入。`);
          });
        }
      }
    } catch (cause) {
      if (!mounted.current || current !== sequence.current) return;
      setError(`查询失败：${errorMessage(cause)}`);
      setFailed(operation);
      setStatus("当前棋谱未改动。可重试本次查询。");
    } finally {
      if (mounted.current && current === sequence.current) setBusy(false);
    }
  }

  function changeCategory(category: YikeCategory) {
    cursor.current = 0;
    setList(null);
    void run({ kind: "list", category, page: 1, since: 0 });
  }

  function changeLocator(value: string) {
    editedLocator.current = true;
    invalidate();
    setLocator(value);
    setStatus("URL 已更改；请重新预览。当前棋谱未改动。");
  }

  async function importPreview() {
    if (!preview || disabled || busy || importingRef.current) return;
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
      if (mounted.current) setImporting(false);
    }
  }

  async function startSync(play: boolean) {
    if (!onStartSync || locked || busy || !locator.trim()) return;
    try { await onStartSync(locator.trim(), play); }
    catch (cause) { if (mounted.current) setError(`同步未启动：${errorMessage(cause)}`); }
  }

  const locked = disabled || importing;
  const currentList = list?.category === query.category && list.page === query.page ? list : null;
  return (
    <section className="yike-public-center" aria-label="Yike 公共中心">
      <div className="provider-subheader"><h3>Yike 公共中心</h3></div>
      <div className="yike-query-controls" role="group" aria-label="公开列表分类">
        <button disabled={locked || revision === undefined} aria-pressed={query.category === "recommend"} onClick={() => changeCategory("recommend")}>Recommend</button>
        <button disabled={locked || revision === undefined} aria-pressed={query.category === "local"} onClick={() => changeCategory("local")}>Local</button>
        <button disabled={locked || revision === undefined} onClick={() => void run({ ...query, since: cursor.current })}>刷新列表</button>
      </div>
      <div className="yike-query-controls" role="group" aria-label="列表分页">
        <button disabled={locked || query.page <= 1 || revision === undefined} onClick={() => void run({ ...query, page: query.page - 1, since: cursor.current })}>上一页</button>
        <span>第 {query.page} 页</span>
        <button disabled={locked || !currentList?.has_more || revision === undefined} onClick={() => void run({ ...query, page: query.page + 1, since: cursor.current })}>下一页</button>
      </div>
      {currentList ? <>
        {currentList.games.length === 0 ? <p role="status">本页暂无公开棋局。</p> : <ul className="yike-game-list" aria-label="Yike 公开棋局">
          {currentList.games.map((game) => <li key={game.locator}>
            <button disabled={locked} onClick={() => {
              editedLocator.current = true;
              setLocator(game.locator);
              void run({ kind: "preview", locator: game.locator });
            }}>
              {game.title || "公开棋局"} · {game.black_name ?? "黑方"} vs {game.white_name ?? "白方"}
              {game.move_count !== null && game.move_count !== undefined ? ` · ${game.move_count} 手` : ""}
              {game.status ? ` · ${game.status}` : ""}
            </button>
          </li>)}
        </ul>}
        {!currentList.has_more ? <p role="status">已到列表末页。</p> : null}
      </> : null}
      <div className="provider-grid">
        <label><span>Yike URL</span><input value={locator} disabled={locked} placeholder="粘贴 Yike 公开棋局 URL" onChange={(event) => changeLocator(event.target.value)} /></label>
        <button disabled={locked || revision === undefined || !locator.trim()} onClick={() => void run({ kind: "preview", locator: locator.trim() })}>预览棋局</button>
      </div>
      <div className="yike-query-controls">
        <button disabled={importing || !busy} onClick={() => {
          invalidate();
          setStatus("已取消查询；当前棋谱未改动。");
        }}>取消查询</button>
        {failed ? <button disabled={locked || revision === undefined} onClick={() => void run(failed)}>重试查询</button> : null}
      </div>
      <p className="provider-status" role="status">{status}</p>
      {error ? <p role="alert">{error}</p> : null}
      {persistenceError ? <p role="alert">{persistenceError}</p> : null}
      {routes.length > 0 ? <ul className="yike-read-routes" aria-label="实际网络路由">{routes.map((route, index) =>
        <li key={index}>{route.mode} / {route.source} → {route.target} via {route.proxy ?? "DIRECT"}</li>
      )}</ul> : null}
      {preview ? <section aria-label="Provider 棋谱预览">
        <p>{preview.result.summary.black_name ?? "黑方"} vs {preview.result.summary.white_name ?? "白方"} · {preview.result.summary.board_width}×{preview.result.summary.board_height} · {preview.result.summary.move_count} 手</p>
        {preview.result.warnings.length > 0 ? <ul aria-label="Yike 预览提示">{preview.result.warnings.map((warning, index) => <li key={index}>{warning}</li>)}</ul> : null}
        <button disabled={locked || busy} onClick={() => void importPreview()}>导入预览棋谱</button>
        {onStartSync ? <>
          <button disabled={locked || busy} onClick={() => void startSync(false)}>Start sync 此局</button>
          <button disabled={locked || busy} onClick={() => void startSync(true)}>Play &amp; Sync 此局</button>
        </> : null}
      </section> : null}
    </section>
  );
}

function sameIdentity(left: ProviderRequestIdentity, right: ProviderRequestIdentity): boolean {
  return left.request_id === right.request_id && left.policy_revision === right.policy_revision && left.document_identity === right.document_identity;
}

function errorMessage(cause: unknown): string {
  if (cause instanceof Error) return cause.message;
  if (typeof cause === "object" && cause !== null && "message" in cause && typeof cause.message === "string") return cause.message;
  return String(cause);
}
