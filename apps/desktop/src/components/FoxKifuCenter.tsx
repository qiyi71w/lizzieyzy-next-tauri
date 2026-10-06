import { useEffect, useRef, useState } from "react";
import {
  beginProviderRequest,
  cancelProviderRequest,
  clearFoxRecents,
  fetchFoxList,
  fetchFoxListMore,
  loadFoxKifuState,
  previewFoxGame,
  rememberFoxLookup
} from "../api/providers";
import type {
  FoxAccount,
  FoxGameEntry,
  FoxLookup,
  FoxLookupKind,
  FoxPreviewResult,
  NetworkRoute,
  NetworkSnapshot,
  ProviderImportResult,
  ProviderRequestIdentity
} from "../domain/providers";

type Props = {
  disabled?: boolean;
  network: NetworkSnapshot | null;
  onImport: (result: ProviderImportResult, identity?: ProviderRequestIdentity) => void | Promise<void>;
};

/** Accumulated list for one account lookup. Continuations reuse the frozen account and cursor. */
type FoxList = { account: FoxAccount; games: FoxGameEntry[]; cursor: string | null; hasMore: boolean };
type Operation =
  | { kind: "list"; lookup: FoxLookup }
  | { kind: "more"; fromPage: number }
  | { kind: "preview"; chessid: string };

export const foxPageSize = 25;

const lookupLabels: Record<FoxLookupKind, string> = { nickname: "昵称", uid: "UID", chessid: "棋谱 chessid" };

export function FoxKifuCenter({ disabled = false, network, onImport }: Props) {
  const [kind, setKind] = useState<FoxLookupKind>("nickname");
  const [value, setValue] = useState("");
  const [recents, setRecents] = useState<FoxAccount[]>([]);
  const [list, setList] = useState<FoxList | null>(null);
  const [page, setPage] = useState(1);
  const [preview, setPreview] = useState<FoxPreviewResult | null>(null);
  const [routes, setRoutes] = useState<NetworkRoute[]>([]);
  const [busy, setBusy] = useState(false);
  const [importing, setImporting] = useState(false);
  const [status, setStatus] = useState("按昵称、UID 查询棋谱列表，或输入 chessid 直接预览。预览不会改变当前棋谱。");
  const [error, setError] = useState("");
  const [persistenceError, setPersistenceError] = useState("");
  const [failed, setFailed] = useState<Operation | null>(null);
  const sequence = useRef(0);
  const identity = useRef<ProviderRequestIdentity | null>(null);
  const mounted = useRef(false);
  const edited = useRef(false);
  const importingRef = useRef(false);
  const listRef = useRef<FoxList | null>(null);
  const pageRef = useRef(1);
  const revision = network?.policy_revision;
  const seenRevision = useRef(revision);

  useEffect(() => {
    mounted.current = true;
    void loadFoxKifuState().then((saved) => {
      if (!mounted.current) return;
      setRecents(saved.recents);
      if (!edited.current && saved.last_query) {
        setKind(saved.last_query.kind);
        setValue(saved.last_query.value);
      }
    }).catch((cause: unknown) => {
      if (mounted.current) setPersistenceError(`无法读取野狐最近查询：${errorMessage(cause)}`);
    });
    return () => {
      mounted.current = false;
      sequence.current += 1;
      const previous = identity.current;
      identity.current = null;
      if (previous) void cancelProviderRequest(previous).catch(() => {});
    };
  }, []);

  // A saved network policy or a blocked document seals in-flight reads and any preview built
  // under them; the already-listed rows stay visible and later reads use the new policy.
  useEffect(() => {
    if (seenRevision.current === revision && !disabled) return;
    seenRevision.current = revision;
    if (importingRef.current) return;
    invalidate();
    setPreview(null);
  }, [revision, disabled]);

  function invalidate() {
    const current = ++sequence.current;
    const previous = identity.current;
    identity.current = null;
    if (previous) void cancelProviderRequest(previous).catch((cause: unknown) => {
      if (mounted.current && current === sequence.current) setError(`取消查询失败：${errorMessage(cause)}`);
    });
    setBusy(false);
    setRoutes([]);
    setError("");
    setFailed(null);
    return current;
  }

  function showList(next: FoxList | null) {
    listRef.current = next;
    setList(next);
  }

  function remember(lookup: FoxLookup, account: FoxAccount | null) {
    setPersistenceError("");
    void rememberFoxLookup(lookup, account).then((saved) => {
      if (mounted.current) setRecents(saved.recents);
    }).catch((cause: unknown) => {
      if (mounted.current) setPersistenceError(`最近查询保存失败：${errorMessage(cause)}。查询结果不受影响。`);
    });
  }

  async function run(operation: Operation) {
    if (disabled || importingRef.current || revision === undefined) return;
    const frozen = listRef.current;
    if (operation.kind === "more" && !frozen?.cursor) return;
    const current = invalidate();
    setPreview(null);
    if (operation.kind === "list") {
      showList(null);
      movePage(1);
    }
    setBusy(true);
    setStatus(operation.kind === "preview" ? "正在获取棋谱预览；当前棋谱未改动。" : "正在读取野狐棋谱列表；可取消或更换查询。");
    try {
      const lease = await beginProviderRequest(revision);
      if (!mounted.current || current !== sequence.current) {
        await cancelProviderRequest(lease);
        return;
      }
      identity.current = lease;
      if (operation.kind === "preview") {
        const response = await previewFoxGame(operation.chessid, lease);
        if (!mounted.current || current !== sequence.current || !sameIdentity(response.identity, lease)) return;
        setPreview(response);
        setRoutes(response.routes);
        setStatus("预览就绪；当前棋谱未改动。确认后可导入。");
        return;
      }
      const response = operation.kind === "list"
        ? await fetchFoxList(operation.lookup, lease)
        : await fetchFoxListMore(frozen!.account, frozen!.cursor!, lease);
      if (!mounted.current || current !== sequence.current || !sameIdentity(response.identity, lease)) return;
      const batch = response.result;
      setRoutes(response.routes);
      if (operation.kind === "list") {
        showList({ account: batch.account, games: batch.games, cursor: batch.next_cursor ?? null, hasMore: batch.has_more });
        setStatus(batch.games.length === 0 ? "没有找到棋谱。" : "列表已更新。选局仅预览，导入需另行确认。");
        // Frozen Java records an account only when its list has games; the lookup itself is always kept.
        remember(operation.lookup, batch.games.length > 0 ? batch.account : null);
        return;
      }
      const seen = new Set(frozen!.games.map((game) => game.chessid));
      const fresh = batch.games.filter((game) => !seen.has(game.chessid));
      const games = [...frozen!.games, ...fresh];
      showList({ ...frozen!, games, cursor: batch.next_cursor ?? frozen!.cursor, hasMore: batch.has_more && fresh.length > 0 });
      // Show the page holding the first new game (normalized batches need not end on a page
      // boundary), unless the user navigated elsewhere while the batch was loading.
      if (fresh.length > 0 && pageRef.current === operation.fromPage) {
        movePage(Math.floor(frozen!.games.length / foxPageSize) + 1);
      }
      setStatus(fresh.length === 0 ? "没有更多棋谱。" : "已续取下一批棋谱。");
    } catch (cause) {
      if (!mounted.current || current !== sequence.current) return;
      setError(`查询失败：${errorMessage(cause)}`);
      setFailed(operation);
      setStatus("当前棋谱未改动。可重试本次查询。");
    } finally {
      if (mounted.current && current === sequence.current) setBusy(false);
    }
  }

  function search(lookup: FoxLookup) {
    const trimmed = { ...lookup, value: lookup.value.trim() };
    if (!trimmed.value) return;
    if (trimmed.kind === "chessid") {
      showList(null);
      remember(trimmed, null);
      void run({ kind: "preview", chessid: trimmed.value });
      return;
    }
    void run({ kind: "list", lookup: trimmed });
  }

  function changeQuery(nextKind: FoxLookupKind, nextValue: string) {
    edited.current = true;
    invalidate();
    setPreview(null);
    setKind(nextKind);
    setValue(nextValue);
  }

  function nextPage() {
    if (!list) return;
    if (page * foxPageSize < list.games.length) movePage(page + 1);
    else if (list.hasMore) void run({ kind: "more", fromPage: page });
  }

  function movePage(next: number) {
    pageRef.current = next;
    setPage(next);
  }

  async function clearRecents() {
    setPersistenceError("");
    try {
      const saved = await clearFoxRecents();
      if (mounted.current) setRecents(saved.recents);
    } catch (cause) {
      if (mounted.current) setPersistenceError(`清除最近查询失败：${errorMessage(cause)}`);
    }
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

  const locked = disabled || importing || revision === undefined;
  const pageCount = list ? Math.max(1, Math.ceil(list.games.length / foxPageSize)) : 1;
  const rows = list ? list.games.slice((page - 1) * foxPageSize, page * foxPageSize) : [];
  const canNext = !!list && (page < pageCount || list.hasMore);
  return (
    <section className="fox-kifu-center" aria-label="野狐棋谱">
      <div className="provider-subheader"><h3>野狐棋谱</h3></div>
      <div className="fox-query-controls" role="radiogroup" aria-label="查询方式">
        {(Object.keys(lookupLabels) as FoxLookupKind[]).map((option) =>
          <button key={option} role="radio" aria-checked={kind === option} disabled={locked}
            onClick={() => changeQuery(option, value)}>{lookupLabels[option]}</button>
        )}
      </div>
      <div className="provider-grid">
        <label><span>{lookupLabels[kind]}</span><input value={value} disabled={locked}
          placeholder={kind === "nickname" ? "野狐昵称" : kind === "uid" ? "数字 UID" : "数字 chessid"}
          onChange={(event) => changeQuery(kind, event.target.value)}
          onKeyDown={(event) => { if (event.key === "Enter") search({ kind, value }); }} /></label>
        <button disabled={locked || !value.trim()} onClick={() => search({ kind, value })}>{kind === "chessid" ? "预览棋局" : "查询棋谱"}</button>
      </div>
      <div className="fox-recents" role="group" aria-label="野狐最近查询">
        <span>最近：</span>
        {recents.length === 0 ? <span>暂无</span> : recents.map((account) =>
          <button key={account.uid} disabled={locked} onClick={() => {
            changeQuery("uid", account.uid);
            search({ kind: "uid", value: account.uid });
          }}>{accountLabel(account)}</button>
        )}
        <button disabled={locked || recents.length === 0} onClick={() => void clearRecents()}>清除最近</button>
      </div>
      {list ? <>
        <p role="status">当前用户：{accountLabel(list.account)}</p>
        {list.games.length === 0 ? <p role="status">没有找到棋谱。</p> : <ul className="fox-game-list" aria-label="野狐棋谱列表">
          {rows.map((game, index) => <li key={game.chessid}>
            <button disabled={locked} onClick={() => void run({ kind: "preview", chessid: game.chessid })}>
              {(page - 1) * foxPageSize + index + 1}. {game.start_time} · {game.black_name} {game.black_rank} vs {game.white_name} {game.white_rank} · {game.result} · {game.move_count} 手
            </button>
          </li>)}
        </ul>}
        <div className="fox-query-controls" role="group" aria-label="列表分页">
          <button disabled={locked || page <= 1} onClick={() => movePage(page - 1)}>上一页</button>
          <span>第 {page} / {pageCount}{list.hasMore ? "…" : ""} 页</span>
          <button disabled={locked || busy || !canNext} onClick={nextPage}>下一页</button>
        </div>
        {!list.hasMore && list.games.length > 0 ? <p role="status">已到列表末尾。</p> : null}
      </> : null}
      <div className="fox-query-controls">
        <button disabled={importing || !busy} onClick={() => {
          invalidate();
          setStatus("已取消查询；当前棋谱未改动。");
        }}>取消查询</button>
        {failed ? <button disabled={locked} onClick={() => void run(failed)}>重试查询</button> : null}
      </div>
      <p className="provider-status" role="status">{status}</p>
      {error ? <p role="alert">{error}</p> : null}
      {persistenceError ? <p role="alert">{persistenceError}</p> : null}
      {routes.length > 0 ? <ul aria-label="实际网络路由">{routes.map((route, index) =>
        <li key={index}>{route.mode} / {route.source} → {route.target} via {route.proxy ?? "DIRECT"}</li>
      )}</ul> : null}
      {preview ? <section aria-label="Provider 棋谱预览">
        <p>{preview.result.summary.black_name ?? "黑方"} vs {preview.result.summary.white_name ?? "白方"} · {preview.result.summary.board_width}×{preview.result.summary.board_height} · {preview.result.summary.move_count} 手</p>
        <button disabled={locked || busy} onClick={() => void importPreview()}>导入预览棋谱</button>
      </section> : null}
    </section>
  );
}

function accountLabel(account: FoxAccount): string {
  if (!account.nickname || account.nickname === account.uid) return account.uid;
  return `${account.nickname} (${account.uid})`;
}

function sameIdentity(left: ProviderRequestIdentity, right: ProviderRequestIdentity): boolean {
  return left.request_id === right.request_id && left.policy_revision === right.policy_revision && left.document_identity === right.document_identity;
}

function errorMessage(cause: unknown): string {
  if (cause instanceof Error) return cause.message;
  if (typeof cause === "object" && cause !== null && "message" in cause && typeof cause.message === "string") return cause.message;
  return String(cause);
}
