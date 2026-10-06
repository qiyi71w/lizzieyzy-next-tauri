// @vitest-environment jsdom
import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type {
  NetworkSnapshot,
  ProviderImportResult,
  ProviderRequestIdentity,
  TencentHistory,
  TencentListEntry,
  TencentListResult,
  TencentPreviewResult,
  TencentQuery
} from "../domain/providers";
import { TencentKifuCenter } from "./TencentKifuCenter";

const api = vi.hoisted(() => ({
  beginProviderRequest: vi.fn(), cancelProviderRequest: vi.fn(),
  fetchTencentList: vi.fn(), previewTencentChess: vi.fn(),
  loadTencentHistory: vi.fn(), saveTencentQuery: vi.fn()
}));
vi.mock("../api/providers", () => api);

const network: NetworkSnapshot = {
  policy_revision: 1,
  settings: { mode: "direct", manual_host: "127.0.0.1", manual_port: 7897 }
};
const importResult: ProviderImportResult = {
  provider: "tencent", sgf_text: "(;GM[1]FF[4]SZ[19]PB[Black]PW[White];B[qd](;W[dc])(;W[dp]C[variation]))",
  summary: { provider: "tencent", source_id: "42", board_width: 19, board_height: 19, black_name: "Black", white_name: "White", move_count: 2, result: "B+R" },
  metadata: { source_id: "42", extra: {} }, warnings: ["公开棋谱预览"]
};
let root: Root | null = null;
let host: HTMLDivElement;
let nextRequest: number;
let stored: TencentHistory;

function deferred<T>() {
  let resolve!: (value: T | PromiseLike<T>) => void;
  let reject!: (reason?: unknown) => void;
  const promise = new Promise<T>((res, rej) => { resolve = res; reject = rej; });
  return { promise, resolve, reject };
}
function games(first: number, count: number): TencentListEntry[] {
  return Array.from({ length: count }, (_, index) => ({
    chess_id: String(first + index), black_name: `Black${first + index}`, white_name: "White",
    black_rank: "9d", white_rank: "8d", played_at: "2026-10-05", result: "B+R", move_count: 2
  }));
}
function listResponse(identity: ProviderRequestIdentity, rows = games(1, 2), lastCode = "2", hasMore = false, username = "public-player"): TencentListResult {
  return { identity, result: { username, games: rows, last_code: lastCode, has_more: hasMore },
    routes: [{ mode: "direct", source: "app", target: "https://public.tencent/list", proxy: null }] };
}
function previewResponse(identity: ProviderRequestIdentity, source = "42"): TencentPreviewResult {
  return { identity, result: { ...importResult, metadata: { ...importResult.metadata, source_id: source } },
    routes: [{ mode: "direct", source: "app", target: `https://public.tencent/chess/${source}`, proxy: null }] };
}
function button(label: string): HTMLButtonElement {
  const element = Array.from(host.querySelectorAll("button")).find((item) => item.textContent?.trim() === label);
  if (!element) throw new Error(`Missing button: ${label}`);
  return element;
}
function input(): HTMLInputElement {
  const element = host.querySelector<HTMLInputElement>("input");
  if (!element) throw new Error("Missing query input");
  return element;
}
async function type(value: string) {
  await act(async () => {
    Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")?.set?.call(input(), value);
    input().dispatchEvent(new Event("input", { bubbles: true }));
    input().dispatchEvent(new Event("change", { bubbles: true }));
  });
}
async function click(element: HTMLButtonElement) {
  await act(async () => { element.dispatchEvent(new MouseEvent("click", { bubbles: true })); });
}
async function select(kind: TencentQuery["kind"]) {
  await act(async () => {
    const element = host.querySelector("select")!;
    element.value = kind;
    element.dispatchEvent(new Event("change", { bubbles: true }));
  });
}
async function renderCenter(onImport = vi.fn<(result: ProviderImportResult, identity?: ProviderRequestIdentity) => void | Promise<void>>(), snapshot: NetworkSnapshot | null = network, disabled = false) {
  if (!root) root = createRoot(host);
  await act(async () => { root?.render(<TencentKifuCenter network={snapshot} disabled={disabled} onImport={onImport} />); });
  return onImport;
}
async function queryUsername(value = "public-player") {
  await type(value);
  await click(button("查询棋局列表"));
}
async function queryChess(value = "42") {
  await select("chess_id");
  await type(value);
  await click(button("预览棋局"));
}
function visibleIds(): string[] {
  return Array.from(host.querySelectorAll('[aria-label="Tencent 公开棋局"] li')).map((row) => row.textContent!.split(" · ")[0]);
}

beforeEach(() => {
  globalThis.IS_REACT_ACT_ENVIRONMENT = true;
  vi.resetAllMocks();
  nextRequest = 100;
  stored = { recent: [], last_query: null };
  api.beginProviderRequest.mockImplementation(async (policy_revision: number) => ({ request_id: ++nextRequest, policy_revision, document_identity: 7001 }));
  api.cancelProviderRequest.mockResolvedValue(undefined);
  api.loadTencentHistory.mockResolvedValue(stored);
  api.saveTencentQuery.mockImplementation(async (query: TencentQuery | null) => {
    stored = query ? { recent: [query, ...stored.recent.filter((item) => item.kind !== query.kind || item.value !== query.value)].slice(0, 8), last_query: query } : { recent: [], last_query: null };
    return stored;
  });
  api.fetchTencentList.mockImplementation(async (username: string, _cursor: string, identity: ProviderRequestIdentity) => listResponse(identity, games(1, 2), "2", false, username));
  api.previewTencentChess.mockImplementation(async (chessId: string, identity: ProviderRequestIdentity) => previewResponse(identity, chessId));
  host = document.createElement("div");
  document.body.append(host);
});
afterEach(async () => {
  await act(async () => { root?.unmount(); });
  root = null;
  host.remove();
});

describe("Tencent public kifu center", () => {
  it("loads the last public query into accessible input without initiating network or import", async () => {
    api.loadTencentHistory.mockResolvedValueOnce({ recent: [{ kind: "chess_id", value: "42" }], last_query: { kind: "chess_id", value: "42" } });
    const onImport = await renderCenter();
    expect(host.querySelector("label:nth-child(2)")?.textContent).toContain("Tencent chessId");
    expect(input().value).toBe("42");
    expect(button("取消查询")).toBeDefined();
    expect(api.beginProviderRequest).not.toHaveBeenCalled();
    expect(api.fetchTencentList).not.toHaveBeenCalled();
    expect(api.previewTencentChess).not.toHaveBeenCalled();
    expect(api.saveTencentQuery).not.toHaveBeenCalled();
    expect(onImport).not.toHaveBeenCalled();
  });

  it("shows 25 records per page, fetches at cache end, deduplicates batches, and stops on empty continuation", async () => {
    api.fetchTencentList.mockImplementation(async (_username: string, cursor: string, identity: ProviderRequestIdentity) => {
      if (cursor === "0") return listResponse(identity, games(1, 100), "100", true);
      if (cursor === "100") return listResponse(identity, games(91, 40), "130", true);
      return listResponse(identity, [], "130", false);
    });
    await renderCenter();
    await queryUsername();
    expect(visibleIds()).toHaveLength(25);
    expect(visibleIds()[0]).toBe("1");
    expect(visibleIds()[24]).toBe("25");
    await click(button("下一页"));
    expect(visibleIds()[0]).toBe("26");
    await click(button("下一页"));
    await click(button("下一页"));
    expect(visibleIds()[0]).toBe("76");
    expect(api.fetchTencentList).toHaveBeenCalledTimes(1);
    await click(button("下一页"));
    expect(visibleIds()[0]).toBe("101");
    expect(visibleIds()[24]).toBe("125");
    expect(api.fetchTencentList.mock.calls[1][1]).toBe("100");
    await click(button("下一页"));
    expect(visibleIds()).toEqual(["126", "127", "128", "129", "130"]);
    expect(api.fetchTencentList).toHaveBeenCalledTimes(2);
    await click(button("加载更多棋局"));
    expect(api.fetchTencentList.mock.calls[2][1]).toBe("130");
    expect(host.textContent).toContain("已到列表末页");
    expect(button("下一页").disabled).toBe(true);
    await click(button("上一页"));
    expect(visibleIds()[0]).toBe("101");
    expect(api.fetchTencentList).toHaveBeenCalledTimes(3);
  });

  it("continues short 60-record batches and fills the partial page before advancing without skipping records", async () => {
    api.fetchTencentList.mockImplementation(async (_username: string, cursor: string, identity: ProviderRequestIdentity) =>
      cursor === "0" ? listResponse(identity, games(1, 60), "60", true) : listResponse(identity, games(61, 60), "120", true));
    await renderCenter();
    await queryUsername();
    await click(button("下一页"));
    await click(button("下一页"));
    expect(visibleIds()).toEqual(["51", "52", "53", "54", "55", "56", "57", "58", "59", "60"]);
    expect(button("下一页").disabled).toBe(true);
    await click(button("加载更多棋局"));
    expect(host.querySelector('[aria-label="列表分页"]')?.textContent).toContain("第 3 页");
    expect(visibleIds()[0]).toBe("51");
    expect(visibleIds()[24]).toBe("75");
    expect(api.fetchTencentList.mock.calls[1][1]).toBe("60");
    await click(button("下一页"));
    expect(visibleIds()[0]).toBe("76");
    expect(visibleIds()[24]).toBe("100");
    await click(button("上一页"));
    expect(visibleIds()[24]).toBe("75");
    expect(api.fetchTencentList).toHaveBeenCalledTimes(2);
  });

  it("permits another explicit continuation after a duplicates-only batch with an advancing cursor without automatic fetching", async () => {
    api.fetchTencentList.mockImplementation(async (_username: string, cursor: string, identity: ProviderRequestIdentity) => {
      if (cursor === "0") return listResponse(identity, games(1, 25), "25", true);
      if (cursor === "25") return listResponse(identity, games(1, 25), "50", true);
      return listResponse(identity, games(26, 25), "75", true);
    });
    await renderCenter();
    await queryUsername();
    await click(button("下一页"));
    expect(visibleIds()[0]).toBe("1");
    expect(host.textContent).toContain("本批没有新增棋局");
    expect(button("下一页").disabled).toBe(false);
    expect(api.fetchTencentList).toHaveBeenCalledTimes(2);
    await click(button("下一页"));
    expect(visibleIds()[0]).toBe("26");
    expect(api.fetchTencentList.mock.calls[2][1]).toBe("50");
  });

  it("ends on a repeated cursor even when a nonempty batch claims more records", async () => {
    api.fetchTencentList.mockImplementationOnce(async (_username: string, _cursor: string, identity: ProviderRequestIdentity) => listResponse(identity, games(1, 25), "0", true));
    await renderCenter();
    await queryUsername();
    expect(visibleIds()).toHaveLength(25);
    expect(host.textContent).toContain("已到列表末页");
    expect(button("下一页").disabled).toBe(true);
  });

  it("resets rows, page and cursor on a new query and ignores the old late result", async () => {
    const old = deferred<TencentListResult>();
    await renderCenter();
    await queryUsername("first");
    api.fetchTencentList.mockReturnValueOnce(old.promise);
    await queryUsername("second");
    const lease = api.fetchTencentList.mock.calls[1][2];
    expect(visibleIds()).toHaveLength(0);
    await queryUsername("third");
    await act(async () => { old.resolve(listResponse(lease, games(80, 30), "110", true, "second")); });
    expect(visibleIds()).toEqual(["1", "2"]);
    expect(api.cancelProviderRequest).toHaveBeenCalledWith(lease);
    expect(api.fetchTencentList.mock.calls[2].slice(0, 2)).toEqual(["third", "0"]);
    expect(host.querySelector('[aria-label="列表分页"]')?.textContent).toContain("第 1 页");
  });

  it("cancels a lease acquired by a late begin after the input changed", async () => {
    const begin = deferred<ProviderRequestIdentity>();
    api.beginProviderRequest.mockReturnValueOnce(begin.promise);
    await renderCenter();
    await queryUsername();
    await type("another-player");
    const lease = { request_id: 900, policy_revision: 1, document_identity: 7001 };
    await act(async () => { begin.resolve(lease); });
    expect(api.cancelProviderRequest).toHaveBeenCalledWith(lease);
    expect(api.fetchTencentList).not.toHaveBeenCalled();
    expect(visibleIds()).toHaveLength(0);
  });

  it("selection previews summary, warnings and actual routes without importing; explicit Import forwards the parsed full SGF and matching lease once", async () => {
    const onImport = await renderCenter();
    await queryUsername();
    await click(host.querySelector<HTMLButtonElement>('[aria-label="Tencent 公开棋局"] button')!);
    expect(host.querySelector('[aria-label="Provider 棋谱预览"]')?.textContent).toContain("Black vs White · 19×19 · 2 手 · B+R");
    expect(host.querySelector('[aria-label="Tencent 预览提示"]')?.textContent).toContain("公开棋谱预览");
    expect(host.querySelector('[aria-label="实际网络路由"]')?.textContent).toContain("chess/1 via DIRECT");
    expect(onImport).not.toHaveBeenCalled();
    expect(api.saveTencentQuery).toHaveBeenCalledTimes(1);
    await click(button("导入预览棋谱"));
    expect(onImport).toHaveBeenCalledTimes(1);
    expect(onImport.mock.calls[0][0].sgf_text).toContain("W[dp]C[variation]");
    expect(onImport.mock.calls[0][1]).toEqual(api.previewTencentChess.mock.calls[0][1]);
    expect(host.textContent).not.toMatch(/导入成功|Imported/);
  });

  it("the direct chessId path trims and saves only its public query, previews without list lookup, and waits for Import", async () => {
    const onImport = await renderCenter();
    await queryChess(" 42 ");
    expect(input().value).toBe("42");
    expect(api.fetchTencentList).not.toHaveBeenCalled();
    expect(api.previewTencentChess.mock.calls[0][0]).toBe("42");
    expect(api.saveTencentQuery).toHaveBeenCalledWith({ kind: "chess_id", value: "42" });
    expect(host.querySelector('[aria-label="Provider 棋谱预览"]')).not.toBeNull();
    expect(onImport).not.toHaveBeenCalled();
  });

  it.each(["list", "preview"])("cancellation suppresses a late %s error and never changes the current game", async (stage) => {
    const pending = deferred<never>();
    if (stage === "list") api.fetchTencentList.mockReturnValueOnce(pending.promise);
    else api.previewTencentChess.mockReturnValueOnce(pending.promise);
    const onImport = await renderCenter();
    if (stage === "list") await queryUsername();
    else await queryChess();
    const lease = stage === "list" ? api.fetchTencentList.mock.calls[0][2] : api.previewTencentChess.mock.calls[0][1];
    await click(button("取消查询"));
    await act(async () => { pending.reject(new Error("obsolete failure")); });
    expect(api.cancelProviderRequest).toHaveBeenCalledWith(lease);
    expect(host.textContent).toContain("已取消查询");
    expect(host.textContent).not.toContain("obsolete failure");
    expect(host.querySelector('[aria-label="Provider 棋谱预览"]')).toBeNull();
    expect(onImport).not.toHaveBeenCalled();
  });

  it("retains list rows when selection fails and explicitly retries the failed chessId without importing", async () => {
    api.previewTencentChess.mockRejectedValueOnce(new Error("read deadline expired"));
    const onImport = await renderCenter();
    await queryUsername();
    await click(host.querySelector<HTMLButtonElement>('[aria-label="Tencent 公开棋局"] button')!);
    expect(host.textContent).toContain("read deadline expired");
    expect(visibleIds()).toEqual(["1", "2"]);
    expect(onImport).not.toHaveBeenCalled();
    await click(button("重试查询"));
    expect(api.previewTencentChess.mock.calls[1][0]).toBe("1");
    expect(host.querySelector('[aria-label="Provider 棋谱预览"]')).not.toBeNull();
    expect(host.textContent).not.toContain("read deadline expired");
    expect(onImport).not.toHaveBeenCalled();
  });

  it.each(["input", "network", "disabled", "unmount"])("fences a pending preview and cancels its lease after %s changes", async (transition) => {
    const pending = deferred<TencentPreviewResult>();
    api.previewTencentChess.mockReturnValueOnce(pending.promise);
    const onImport = await renderCenter();
    await queryChess();
    const lease = api.previewTencentChess.mock.calls[0][1];
    if (transition === "input") await type("77");
    if (transition === "network") await renderCenter(onImport, { ...network, policy_revision: 2 });
    if (transition === "disabled") await renderCenter(onImport, network, true);
    if (transition === "unmount") await act(async () => { root?.unmount(); root = null; });
    await act(async () => { pending.resolve(previewResponse(lease)); });
    expect(api.cancelProviderRequest).toHaveBeenCalledWith(lease);
    expect(host.querySelector('[aria-label="Provider 棋谱预览"]')).toBeNull();
    expect(onImport).not.toHaveBeenCalled();
    expect(api.previewTencentChess).toHaveBeenCalledTimes(1);
    if (transition === "network") {
      await click(button("预览棋局"));
      expect(api.beginProviderRequest).toHaveBeenLastCalledWith(2);
    }
  });

  it("cached page navigation invalidates a completed preview without fetching or importing", async () => {
    api.fetchTencentList.mockImplementationOnce(async (_username: string, _cursor: string, identity: ProviderRequestIdentity) => listResponse(identity, games(1, 30), "30", false));
    const onImport = await renderCenter();
    await queryUsername();
    await click(host.querySelector<HTMLButtonElement>('[aria-label="Tencent 公开棋局"] button')!);
    const lease = api.previewTencentChess.mock.calls[0][1];
    await click(button("下一页"));
    expect(visibleIds()[0]).toBe("26");
    expect(host.querySelector('[aria-label="Provider 棋谱预览"]')).toBeNull();
    expect(api.cancelProviderRequest).toHaveBeenCalledWith(lease);
    expect(api.fetchTencentList).toHaveBeenCalledTimes(1);
    expect(onImport).not.toHaveBeenCalled();
  });

  it.each(["request_id", "policy_revision", "document_identity"] as const)("rejects preview responses whose %s differs from the lease", async (field) => {
    api.previewTencentChess.mockImplementationOnce(async (_chessId: string, lease: ProviderRequestIdentity) => previewResponse({ ...lease, [field]: lease[field] + 1 }));
    const onImport = await renderCenter();
    await queryChess();
    expect(host.textContent).toContain("预览响应标识已失效");
    expect(host.querySelector('[aria-label="Provider 棋谱预览"]')).toBeNull();
    expect(onImport).not.toHaveBeenCalled();
  });

  it("a failed history save remains visible after cancellation while the valid preview can still be imported", async () => {
    api.saveTencentQuery.mockRejectedValueOnce(new Error("disk full"));
    const onImport = await renderCenter();
    await queryChess();
    expect(host.textContent).toContain("disk full");
    expect(host.querySelector('[aria-label="Provider 棋谱预览"]')).not.toBeNull();
    await click(button("导入预览棋谱"));
    expect(onImport).toHaveBeenCalledTimes(1);
    await click(button("取消查询"));
    expect(host.textContent).toContain("disk full");
    expect(button("重试保存历史")).toBeDefined();
    await click(button("重试保存历史"));
    expect(host.textContent).not.toContain("disk full");
    expect(host.querySelector('[aria-label="Tencent 查询历史"]')?.textContent).toContain("chessId: 42");
  });

  it("late history load never overwrites typed input and never starts a request", async () => {
    const pending = deferred<TencentHistory>();
    api.loadTencentHistory.mockReturnValueOnce(pending.promise);
    await renderCenter();
    await type("typed-player");
    await act(async () => { pending.resolve({ recent: [{ kind: "chess_id", value: "42" }], last_query: { kind: "chess_id", value: "42" } }); });
    expect(input().value).toBe("typed-player");
    expect(host.querySelector("select")?.value).toBe("username");
    expect(host.querySelector('[aria-label="Tencent 查询历史"]')?.textContent).toContain("chessId: 42");
    expect(api.beginProviderRequest).not.toHaveBeenCalled();
  });

  it("a history read failure is visible and its Retry does not overwrite previously typed input", async () => {
    api.loadTencentHistory.mockRejectedValueOnce(new Error("history unreadable"));
    await renderCenter();
    expect(host.textContent).toContain("history unreadable");
    await type("typed-player");
    api.loadTencentHistory.mockResolvedValueOnce({ recent: [], last_query: { kind: "username", value: "saved-player" } });
    await click(button("重试读取历史"));
    expect(host.textContent).not.toContain("history unreadable");
    expect(input().value).toBe("typed-player");
    expect(api.beginProviderRequest).not.toHaveBeenCalled();
  });

  it("serializes pending saves before clear, ignores the older completion, and retries the newest failed clear", async () => {
    const save = deferred<TencentHistory>();
    api.saveTencentQuery.mockReturnValueOnce(save.promise).mockRejectedValueOnce(new Error("clear failed"));
    const onImport = await renderCenter();
    await queryChess();
    await click(button("清除查询历史"));
    expect(api.saveTencentQuery).toHaveBeenCalledTimes(1);
    expect(input().value).toBe("42");
    await act(async () => { save.resolve({ recent: [{ kind: "chess_id", value: "42" }], last_query: { kind: "chess_id", value: "42" } }); });
    expect(api.saveTencentQuery.mock.calls[1][0]).toBeNull();
    expect(host.textContent).toContain("clear failed");
    expect(input().value).toBe("42");
    expect(host.querySelector('[aria-label="Provider 棋谱预览"]')).not.toBeNull();
    expect(host.querySelector('[aria-label="Tencent 查询历史"]')?.textContent).not.toContain("chessId: 42");
    expect(onImport).not.toHaveBeenCalled();
    await click(button("重试保存历史"));
    expect(api.saveTencentQuery.mock.calls[2][0]).toBeNull();
    expect(input().value).toBe("");
    expect(host.querySelector('[aria-label="Provider 棋谱预览"]')).toBeNull();
    expect(host.textContent).not.toContain("clear failed");
  });

  it("persists an accepted clear across unmount before loading the next center", async () => {
    const pending = deferred<TencentHistory>();
    api.saveTencentQuery.mockReturnValueOnce(pending.promise);
    api.loadTencentHistory.mockImplementation(async () => stored);
    await renderCenter();
    await queryChess();
    await click(button("清除查询历史"));
    await act(async () => { root?.unmount(); root = null; });
    stored = { recent: [{ kind: "chess_id", value: "42" }], last_query: { kind: "chess_id", value: "42" } };
    await act(async () => { pending.resolve(stored); });
    await renderCenter();
    expect(stored).toEqual({ recent: [], last_query: null });
    expect(input().value).toBe("");
    expect(host.querySelector('[aria-label="Tencent 查询历史"]')?.textContent).not.toContain("chessId: 42");
  });

  it("orders accepted writes from the old and new centers by user action", async () => {
    const pending = deferred<TencentHistory>();
    api.saveTencentQuery.mockReturnValueOnce(pending.promise);
    api.loadTencentHistory.mockImplementation(async () => stored);
    await renderCenter();
    await queryUsername("first");
    await queryUsername("queued");
    await act(async () => { root?.unmount(); root = null; });
    await renderCenter();
    await queryUsername("newest");
    stored = { recent: [{ kind: "username", value: "first" }], last_query: { kind: "username", value: "first" } };
    await act(async () => { pending.resolve(stored); });
    expect(stored.recent.map((query) => query.value)).toEqual(["newest", "queued", "first"]);
    expect(stored.last_query?.value).toBe("newest");
    expect(input().value).toBe("newest");
  });

  it("shows and retries a clear that failed after the old center unmounted", async () => {
    const pending = deferred<TencentHistory>();
    api.saveTencentQuery.mockReturnValueOnce(pending.promise).mockRejectedValueOnce(new Error("disk unavailable"));
    api.loadTencentHistory.mockImplementation(async () => stored);
    await renderCenter();
    await queryChess();
    await click(button("清除查询历史"));
    await act(async () => { root?.unmount(); root = null; });
    stored = { recent: [{ kind: "chess_id", value: "42" }], last_query: { kind: "chess_id", value: "42" } };
    await act(async () => { pending.resolve(stored); });
    await renderCenter();
    expect(host.textContent).toContain("disk unavailable");
    await click(button("重试保存历史"));
    expect(stored).toEqual({ recent: [], last_query: null });
    expect(input().value).toBe("");
    expect(host.textContent).not.toContain("disk unavailable");
  });

  it("does not let a late initial history snapshot replace the explicitly saved new query", async () => {
    const load = deferred<TencentHistory>();
    api.loadTencentHistory.mockReturnValueOnce(load.promise);
    await renderCenter();
    await queryUsername("new-player");
    await act(async () => { load.resolve({ recent: [{ kind: "username", value: "old-player" }], last_query: { kind: "username", value: "old-player" } }); });
    expect(input().value).toBe("new-player");
    expect(host.querySelector('[aria-label="Tencent 查询历史"]')?.textContent).toContain("username: new-player");
    expect(host.querySelector('[aria-label="Tencent 查询历史"]')?.textContent).not.toContain("old-player");
  });

  it("renders backend recents and runs each recent query through its own list or direct-preview path", async () => {
    const recent: TencentQuery[] = [{ kind: "username", value: "remembered-player" }, { kind: "chess_id", value: "77" }];
    stored = { recent, last_query: null };
    api.loadTencentHistory.mockResolvedValueOnce({ recent, last_query: null });
    await renderCenter();
    await click(button("username: remembered-player"));
    expect(input().value).toBe("remembered-player");
    expect(visibleIds()).toEqual(["1", "2"]);
    await click(button("chessId: 77"));
    expect(input().value).toBe("77");
    expect(host.querySelector("select")?.value).toBe("chess_id");
    expect(visibleIds()).toHaveLength(0);
    expect(host.querySelector('[aria-label="Provider 棋谱预览"]')).not.toBeNull();
    expect(api.previewTencentChess.mock.calls[0][0]).toBe("77");
  });

  it("synchronously blocks duplicate Import admission, preserves its lease through its own disabled transition, and releases controls after rejection", async () => {
    const pending = deferred<void>();
    const onImport = vi.fn<(result: ProviderImportResult, identity?: ProviderRequestIdentity) => void | Promise<void>>(() => {
      button("清除查询历史").dispatchEvent(new MouseEvent("click", { bubbles: true }));
      button("预览棋局").dispatchEvent(new MouseEvent("click", { bubbles: true }));
      return pending.promise;
    });
    await renderCenter(onImport);
    await queryChess();
    const lease = api.previewTencentChess.mock.calls[0][1];
    await act(async () => {
      const importButton = button("导入预览棋谱");
      importButton.dispatchEvent(new MouseEvent("click", { bubbles: true }));
      importButton.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });
    await renderCenter(onImport, network, true);
    expect(onImport).toHaveBeenCalledTimes(1);
    expect(api.saveTencentQuery).toHaveBeenCalledTimes(1);
    expect(api.previewTencentChess).toHaveBeenCalledTimes(1);
    expect(api.cancelProviderRequest).not.toHaveBeenCalledWith(lease);
    expect(input().disabled).toBe(true);
    expect(button("取消查询").disabled).toBe(true);
    expect(button("清除查询历史").disabled).toBe(true);
    await renderCenter(onImport, network, false);
    await act(async () => { pending.reject(new Error("import admission rejected")); });
    expect(host.textContent).toContain("import admission rejected");
    expect(input().disabled).toBe(false);
    expect(button("预览棋局").disabled).toBe(false);
    expect(onImport).toHaveBeenCalledTimes(1);
  });
  it("retries the exact failed cursor without discarding cached rows or importing", async () => {
    api.fetchTencentList.mockImplementationOnce(async (_username: string, _cursor: string, identity: ProviderRequestIdentity) => listResponse(identity, games(1, 25), "25", true));
    const onImport = await renderCenter();
    await queryUsername();
    api.fetchTencentList.mockRejectedValueOnce(new Error("continuation failed"));
    await click(button("下一页"));
    expect(visibleIds()[0]).toBe("1");
    expect(host.textContent).toContain("continuation failed");
    api.fetchTencentList.mockImplementationOnce(async (_username: string, _cursor: string, identity: ProviderRequestIdentity) => listResponse(identity, games(26, 25), "50", true));
    await click(button("重试查询"));
    expect(api.fetchTencentList.mock.calls[2][1]).toBe("25");
    expect(visibleIds()[0]).toBe("26");
    expect(host.textContent).not.toContain("continuation failed");
    expect(onImport).not.toHaveBeenCalled();
  });

  it("clear success removes saved recents but does not erase input typed while persistence was pending", async () => {
    const pending = deferred<TencentHistory>();
    await renderCenter();
    await queryUsername();
    api.saveTencentQuery.mockReturnValueOnce(pending.promise);
    await click(button("清除查询历史"));
    await type("new-unsaved-input");
    await act(async () => { pending.resolve({ recent: [], last_query: null }); });
    expect(input().value).toBe("new-unsaved-input");
    expect(host.querySelector('[aria-label="Tencent 查询历史"]')?.textContent).not.toContain("username: public-player");
    expect(api.fetchTencentList).toHaveBeenCalledTimes(1);
  });

  it("renders the backend's eight most recent unique queries rather than retaining a ninth local entry", async () => {
    await renderCenter();
    for (const username of ["one", "two", "three", "four", "five", "six", "seven", "eight", "nine"]) await queryUsername(username);
    const recents = host.querySelectorAll('[aria-label="Tencent 查询历史"] button');
    expect(recents).toHaveLength(9);
    expect(recents[0].textContent).toBe("username: nine");
    expect(host.querySelector('[aria-label="Tencent 查询历史"]')?.textContent).not.toContain("username: one");
    await queryUsername("five");
    expect(host.querySelectorAll('[aria-label="Tencent 查询历史"] button')).toHaveLength(9);
    expect(host.querySelector('[aria-label="Tencent 查询历史"] button')?.textContent).toBe("username: five");
  });

  it("cancels a begin lease arriving after unmount without dispatching a remote read", async () => {
    const pending = deferred<ProviderRequestIdentity>();
    api.beginProviderRequest.mockReturnValueOnce(pending.promise);
    const onImport = await renderCenter();
    await queryChess();
    await act(async () => { root?.unmount(); root = null; });
    const lease = { request_id: 999, policy_revision: 1, document_identity: 7001 };
    await act(async () => { pending.resolve(lease); });
    expect(api.cancelProviderRequest).toHaveBeenCalledWith(lease);
    expect(api.previewTencentChess).not.toHaveBeenCalled();
    expect(onImport).not.toHaveBeenCalled();
  });
});
