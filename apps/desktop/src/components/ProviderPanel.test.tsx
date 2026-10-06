// @vitest-environment jsdom
import type { Root } from "react-dom/client";
import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type {
  FoxAccount,
  FoxGameEntry,
  FoxListResult,
  FoxPreviewResult,
  NetworkSnapshot,
  ProviderImportResult,
  ProviderRequestIdentity,
  YikeCategory,
  YikeListResult,
  YikePreviewResult
} from "../domain/providers";
import { ProviderPanel } from "./ProviderPanel";

const api = vi.hoisted(() => ({
  beginProviderRequest: vi.fn(),
  cancelProviderRequest: vi.fn(),
  networkSnapshot: vi.fn(),
  saveNetworkSettings: vi.fn(),
  fetchYikeList: vi.fn(),
  previewYikeLocator: vi.fn(),
  loadYikeLocator: vi.fn(),
  saveYikeLocator: vi.fn(),
  fetchFoxList: vi.fn(),
  fetchFoxListMore: vi.fn(),
  previewFoxGame: vi.fn(),
  loadFoxKifuState: vi.fn(),
  rememberFoxLookup: vi.fn(),
  clearFoxRecents: vi.fn(),
  fetchTencentList: vi.fn(),
  previewTencentChess: vi.fn(),
  loadTencentHistory: vi.fn(),
  saveTencentQuery: vi.fn(),
  importProviderPayload: vi.fn(),
  syncReadboardSidecarSnapshot: vi.fn()
}));
vi.mock("../api/providers", () => api);

const directSnapshot: NetworkSnapshot = {
  settings: { mode: "direct", manual_host: "127.0.0.1", manual_port: 7897 },
  policy_revision: 1
};
const canonicalLocator = "https://www.yikeweiqi.com/golive/dtl?id=98765&flag=1";
const yikeImportResult: ProviderImportResult = {
  provider: "yike",
  sgf_text: "(;GM[1]FF[4]SZ[19]PB[Shusaku]PW[Gennan];B[qd](;W[dc])(;W[dp]C[source variation]))",
  summary: { provider: "yike", source_id: "98765", board_width: 19, board_height: 19, black_name: "Shusaku", white_name: "Gennan", move_count: 2 },
  metadata: { source_url: canonicalLocator, source_id: "98765", request_url: "https://api.yikeweiqi.com/live/98765", extra: {} },
  warnings: []
};
const foxIdentity: ProviderRequestIdentity = {
  request_id: 102,
  policy_revision: 1,
  document_identity: 7002
};

const foxImportResult: ProviderImportResult = {
  ...yikeImportResult,
  provider: "fox",
  sgf_text: "(;GM[1]FF[4]SZ[19]PB[PlayerA]PW[PlayerB]RE[B+R];B[dp];W[pd])",
  summary: {
    provider: "fox",
    source_id: "123456",
    board_width: 19,
    board_height: 19,
    black_name: "PlayerA",
    white_name: "PlayerB",
    result: "B+R",
    move_count: 2
  },
  metadata: { source_id: "123456", extra: {} },
  warnings: []
};

function listResponse(category: YikeCategory, page: number, since: number, identity: ProviderRequestIdentity): YikeListResult {
  return {
    identity,
    result: {
      category, page, since: since || 1700000000, has_more: true,
      games: [{ locator: canonicalLocator, title: `${category} game ${page}`, black_name: "Shusaku", white_name: "Gennan", status: "live", move_count: 2 }]
    },
    routes: [{ mode: "direct", source: "app", target: "https://api.yikeweiqi.com/public/list", proxy: null }]
  };
}
function previewResponse(identity: ProviderRequestIdentity): YikePreviewResult {
  return {
    identity,
    result: yikeImportResult,
    routes: [{ mode: "direct", source: "app", target: "https://api.yikeweiqi.com/live/98765", proxy: null }]
  };
}

const foxPreviewResult: FoxPreviewResult = {
  identity: foxIdentity,
  result: foxImportResult,
  routes: [{ mode: "direct", source: "direct", target: "h5.foxwq.com:443", proxy: null }]
};

const foxAccount: FoxAccount = { uid: "8772065", nickname: "绝艺" };

/** `count` games with descending chessids starting at `first`; names carry the id for assertions. */
function foxGames(count: number, first: number): FoxGameEntry[] {
  return Array.from({ length: count }, (_, index) => {
    const id = String(first - index);
    return {
      chessid: id, start_time: "2026-10-05 12:00", title: "", black_name: `p${id}`, black_uid: "1",
      black_rank: "9段", white_name: "绝艺", white_uid: "8772065", white_rank: "职业9段",
      result: "白中盘胜", move_count: 200
    };
  });
}

function foxList(identity: ProviderRequestIdentity, games: FoxGameEntry[], cursor: string | null, hasMore: boolean): FoxListResult {
  return { identity, result: { account: foxAccount, games, next_cursor: cursor, has_more: hasMore }, routes: [] };
}

function foxRows(): HTMLLIElement[] {
  return Array.from(host.querySelectorAll<HTMLLIElement>('ul[aria-label="野狐棋谱列表"] li'));
}

let root: Root | null = null;
let host: HTMLDivElement;
let nextRequest: number;

function deferred<T>() {
  let resolve!: (value: T | PromiseLike<T>) => void;
  let reject!: (reason?: unknown) => void;
  const promise = new Promise<T>((res, rej) => { resolve = res; reject = rej; });
  return { promise, resolve, reject };
}
function selectByLabel(label: string): HTMLSelectElement {
  const wrapper = Array.from(host.querySelectorAll("label")).find((element) => element.querySelector("span")?.textContent?.trim() === label);
  const element = wrapper?.querySelector("select");
  if (!element) throw new Error(`Missing select: ${label}`);
  return element;
}
function inputByLabel(label: string): HTMLInputElement {
  const wrapper = Array.from(host.querySelectorAll("label")).find((element) => element.querySelector("span")?.textContent?.trim() === label);
  const element = wrapper?.querySelector("input");
  if (!element) throw new Error(`Missing input: ${label}`);
  return element;
}
function button(label: string): HTMLButtonElement {
  const element = Array.from(host.querySelectorAll("button")).find((item) => item.textContent?.trim() === label);
  if (!element) throw new Error(`Missing button: ${label}`);
  return element;
}
async function changeSelect(element: HTMLSelectElement, value: string) {
  await act(async () => { element.value = value; element.dispatchEvent(new Event("change", { bubbles: true })); });
}
async function changeInput(element: HTMLInputElement | HTMLTextAreaElement, value: string) {
  await act(async () => {
    const prototype = element instanceof HTMLTextAreaElement ? HTMLTextAreaElement.prototype : HTMLInputElement.prototype;
    Object.getOwnPropertyDescriptor(prototype, "value")?.set?.call(element, value);
    element.dispatchEvent(new Event("input", { bubbles: true }));
    element.dispatchEvent(new Event("change", { bubbles: true }));
  });
}
async function click(element: HTMLButtonElement) {
  await act(async () => { element.dispatchEvent(new MouseEvent("click", { bubbles: true })); });
}
async function renderPanel(initialProvider: "yike" | "fox" | "tencent" = "yike", onImport = vi.fn<(result: ProviderImportResult, identity?: ProviderRequestIdentity) => void | Promise<void>>()) {
  root = createRoot(host);
  await act(async () => { root?.render(<ProviderPanel initialProvider={initialProvider} onImport={onImport} />); });
  return onImport;
}
async function previewLocator() {
  await changeInput(inputByLabel("Yike URL"), canonicalLocator);
  await click(button("预览棋局"));
}

beforeEach(() => {
  globalThis.IS_REACT_ACT_ENVIRONMENT = true;
  vi.resetAllMocks();
  nextRequest = 100;
  api.networkSnapshot.mockResolvedValue(directSnapshot);
  api.beginProviderRequest.mockImplementation(async (policy_revision: number) => ({ request_id: ++nextRequest, policy_revision, document_identity: 7001 }));
  api.cancelProviderRequest.mockResolvedValue(undefined);
  api.loadYikeLocator.mockResolvedValue(null);
  api.saveYikeLocator.mockImplementation(async (locator: string) => locator);
  api.fetchYikeList.mockImplementation(async (category: YikeCategory, page: number, since: number, identity: ProviderRequestIdentity) => listResponse(category, page, since, identity));
  api.previewYikeLocator.mockImplementation(async (_locator: string, identity: ProviderRequestIdentity) => previewResponse(identity));
  api.loadFoxKifuState.mockResolvedValue({ recents: [], last_query: null });
  api.loadTencentHistory.mockResolvedValue({ recent: [], last_query: null });
  api.saveTencentQuery.mockImplementation(async (query) => ({ recent: query ? [query] : [], last_query: query }));
  host = document.createElement("div");
  document.body.append(host);
});
afterEach(async () => {
  await act(async () => { root?.unmount(); });
  root = null;
  host.remove();
  document.body.replaceChildren();
});

describe("ProviderPanel Yike public center", () => {
  it("opens Recommend by default, switches to Local page one, and keeps category/page session-only", async () => {
    await renderPanel();
    expect(button("Recommend").getAttribute("aria-pressed")).toBe("true");
    expect(api.fetchYikeList).toHaveBeenLastCalledWith("recommend", 1, 0, expect.objectContaining({ policy_revision: 1 }));
    expect(host.querySelector('[aria-label="Yike 公开棋局"]')?.textContent).toContain("recommend game 1");
    await click(button("下一页"));
    await click(button("Local"));
    expect(button("Local").getAttribute("aria-pressed")).toBe("true");
    expect(api.fetchYikeList).toHaveBeenLastCalledWith("local", 1, 0, expect.any(Object));
    expect(api.saveYikeLocator).not.toHaveBeenCalled();
    await act(async () => { root?.unmount(); root = null; });
    await renderPanel();
    expect(button("Recommend").getAttribute("aria-pressed")).toBe("true");
    expect(api.fetchYikeList).toHaveBeenLastCalledWith("recommend", 1, 0, expect.any(Object));
  });

  it("retains the accepted cursor through next/previous paging and deduplicates records", async () => {
    api.fetchYikeList.mockImplementationOnce(async (category: YikeCategory, page: number, since: number, identity: ProviderRequestIdentity) => {
      const response = listResponse(category, page, since, identity);
      response.result.games.push({ ...response.result.games[0] });
      return response;
    });
    await renderPanel();
    expect(host.querySelectorAll('[aria-label="Yike 公开棋局"] li')).toHaveLength(1);
    expect(button("上一页").disabled).toBe(true);
    await click(button("下一页"));
    expect(api.fetchYikeList).toHaveBeenLastCalledWith("recommend", 2, 1700000000, expect.any(Object));
    expect(host.querySelector('[aria-label="Yike 公开棋局"]')?.textContent).toContain("recommend game 2");
    await click(button("上一页"));
    expect(api.fetchYikeList).toHaveBeenLastCalledWith("recommend", 1, 1700000000, expect.any(Object));
  });

  it("shows empty and end states, then retries the exact failed page and cursor", async () => {
    await renderPanel();
    api.fetchYikeList.mockRejectedValueOnce(new Error("read deadline expired"));
    await click(button("下一页"));
    expect(host.querySelector('[role="alert"]')?.textContent).toContain("read deadline expired");
    expect(button("上一页").disabled).toBe(false);
    api.fetchYikeList.mockImplementationOnce(async (category: YikeCategory, page: number, since: number, identity: ProviderRequestIdentity) => ({
      ...listResponse(category, page, since, identity), result: { category, page, since, games: [], has_more: false }
    }));
    await click(button("重试查询"));
    expect(api.fetchYikeList).toHaveBeenLastCalledWith("recommend", 2, 1700000000, expect.any(Object));
    expect(host.textContent).toContain("本页暂无公开棋局");
    expect(host.textContent).toContain("已到列表末页");
    expect(button("下一页").disabled).toBe(true);
    expect(host.querySelector('[role="alert"]')).toBeNull();
  });

  it("selection previews players, size, moves and actual routes without importing; explicit Import passes the complete SGF once", async () => {
    const onImport = await renderPanel();
    const row = host.querySelector<HTMLButtonElement>('[aria-label="Yike 公开棋局"] button')!;
    await click(row);
    expect(api.previewYikeLocator).toHaveBeenCalledWith(canonicalLocator, expect.any(Object));
    const preview = host.querySelector('section[aria-label="Provider 棋谱预览"]');
    expect(preview?.textContent).toContain("Shusaku vs Gennan");
    expect(preview?.textContent).toContain("19×19");
    expect(preview?.textContent).toContain("2 手");
    expect(host.querySelector('[aria-label="实际网络路由"]')?.textContent).toContain("https://api.yikeweiqi.com/live/98765 via DIRECT");
    expect(onImport).not.toHaveBeenCalled();
    expect(api.importProviderPayload).not.toHaveBeenCalled();
    expect(api.saveYikeLocator).toHaveBeenCalledWith(canonicalLocator);
    await click(button("导入预览棋谱"));
    const lease = api.previewYikeLocator.mock.calls[0][1];
    expect(onImport).toHaveBeenCalledTimes(1);
    expect(onImport).toHaveBeenCalledWith(yikeImportResult, lease);
    expect(onImport.mock.calls[0][0].sgf_text).toContain("(;W[dp]C[source variation])");
    expect(host.textContent).not.toMatch(/Imported|导入成功/);
  });

  it("pasted locator uses the same preview API and persists only the successful canonical URL", async () => {
    const onImport = await renderPanel();
    const pasted = `${canonicalLocator}&tracking=unpersisted`;
    await changeInput(inputByLabel("Yike URL"), pasted);
    expect(api.saveYikeLocator).not.toHaveBeenCalled();
    await click(button("预览棋局"));
    expect(api.previewYikeLocator).toHaveBeenCalledWith(pasted, expect.any(Object));
    expect(api.saveYikeLocator).toHaveBeenCalledTimes(1);
    expect(api.saveYikeLocator).toHaveBeenCalledWith(canonicalLocator);
    expect(onImport).not.toHaveBeenCalled();
  });

  it("does not overwrite typed locator with late persisted load", async () => {
    const saved = deferred<string | null>();
    api.loadYikeLocator.mockReturnValueOnce(saved.promise);
    await renderPanel();
    await changeInput(inputByLabel("Yike URL"), "https://www.yikeweiqi.com/golive/dtl?id=444");
    await act(async () => { saved.resolve(canonicalLocator); });
    expect(inputByLabel("Yike URL").value).toContain("id=444");
  });

  it("loads a saved locator without starting a preview or persisting category/page", async () => {
    api.loadYikeLocator.mockResolvedValueOnce(canonicalLocator);
    const onImport = await renderPanel();
    expect(inputByLabel("Yike URL").value).toBe(canonicalLocator);
    expect(api.previewYikeLocator).not.toHaveBeenCalled();
    expect(api.saveYikeLocator).not.toHaveBeenCalled();
    expect(onImport).not.toHaveBeenCalled();
  });

  it("makes locator save failure visible while preserving the valid preview for import", async () => {
    api.saveYikeLocator.mockRejectedValueOnce(new Error("atomic preferences write failed"));
    const onImport = await renderPanel();
    await previewLocator();
    expect(host.querySelector('[role="alert"]')?.textContent).toContain("atomic preferences write failed");
    expect(button("导入预览棋谱").disabled).toBe(false);
    await click(button("导入预览棋谱"));
    expect(onImport).toHaveBeenCalledTimes(1);
  });

  it("fences stale category response and late begin identity while leaving query controls usable", async () => {
    const firstBegin = deferred<ProviderRequestIdentity>();
    api.beginProviderRequest.mockReturnValueOnce(firstBegin.promise);
    await renderPanel();
    expect(button("取消查询").disabled).toBe(false);
    expect(button("Local").disabled).toBe(false);
    await click(button("Local"));
    const oldIdentity = { request_id: 900, policy_revision: 1, document_identity: 7001 };
    await act(async () => { firstBegin.resolve(oldIdentity); });
    expect(api.cancelProviderRequest).toHaveBeenCalledWith(oldIdentity);
    expect(api.fetchYikeList).toHaveBeenCalledTimes(1);
    expect(host.querySelector('[aria-label="Yike 公开棋局"]')?.textContent).toContain("local game 1");
  });

  it("cannot replace Local with a stale Recommend list response", async () => {
    const old = deferred<YikeListResult>();
    api.fetchYikeList.mockReturnValueOnce(old.promise);
    await renderPanel();
    const oldIdentity = api.fetchYikeList.mock.calls[0][3];
    await click(button("Local"));
    await act(async () => { old.resolve(listResponse("recommend", 1, 0, oldIdentity)); });
    expect(api.cancelProviderRequest).toHaveBeenCalledWith(oldIdentity);
    expect(host.querySelector('[aria-label="Yike 公开棋局"]')?.textContent).toContain("local game 1");
    expect(host.querySelector('[aria-label="Yike 公开棋局"]')?.textContent).not.toContain("recommend game");
  });

  it("fences a stale page response after previous-page navigation", async () => {
    await renderPanel();
    const old = deferred<YikeListResult>();
    api.fetchYikeList.mockReturnValueOnce(old.promise);
    await click(button("下一页"));
    const oldIdentity = api.fetchYikeList.mock.calls[1][3];
    expect(button("上一页").disabled).toBe(false);
    await click(button("上一页"));
    await act(async () => { old.resolve(listResponse("recommend", 2, 1700000000, oldIdentity)); });
    expect(host.querySelector('[aria-label="Yike 公开棋局"]')?.textContent).toContain("recommend game 1");
    expect(host.querySelector('[aria-label="列表分页"]')?.textContent).toContain("第 1 页");
  });

  it("cancels a pending list and suppresses its late failure", async () => {
    const old = deferred<YikeListResult>();
    api.fetchYikeList.mockReturnValueOnce(old.promise);
    const onImport = await renderPanel();
    const oldIdentity = api.fetchYikeList.mock.calls[0][3];
    await click(button("取消查询"));
    expect(api.cancelProviderRequest).toHaveBeenCalledWith(oldIdentity);
    expect(button("取消查询").disabled).toBe(true);
    await act(async () => { old.reject(new Error("obsolete read failure")); });
    expect(host.querySelector('[role="alert"]')).toBeNull();
    expect(host.querySelector('[aria-label="Yike 公开棋局"]')).toBeNull();
    expect(onImport).not.toHaveBeenCalled();
  });

  it("retries a failed locator preview without saving or importing the failed result", async () => {
    api.previewYikeLocator.mockRejectedValueOnce(new Error("unsupported public locator"));
    const onImport = await renderPanel();
    await previewLocator();
    expect(host.querySelector('[role="alert"]')?.textContent).toContain("unsupported public locator");
    expect(host.querySelector('section[aria-label="Provider 棋谱预览"]')).toBeNull();
    expect(api.saveYikeLocator).not.toHaveBeenCalled();
    expect(onImport).not.toHaveBeenCalled();
    await click(button("重试查询"));
    expect(api.previewYikeLocator).toHaveBeenLastCalledWith(canonicalLocator, expect.any(Object));
    expect(host.querySelector('section[aria-label="Provider 棋谱预览"]')).not.toBeNull();
    expect(onImport).not.toHaveBeenCalled();
  });

  it.each(["locator", "category", "cancel", "provider"])("discards old preview after %s change or cancellation", async (transition) => {
    const old = deferred<YikePreviewResult>();
    api.previewYikeLocator.mockReturnValueOnce(old.promise);
    const onImport = await renderPanel();
    await previewLocator();
    const oldIdentity = api.previewYikeLocator.mock.calls[0][1];
    expect(button("取消查询").disabled).toBe(false);
    expect(inputByLabel("Yike URL").disabled).toBe(false);
    if (transition === "locator") await changeInput(inputByLabel("Yike URL"), "https://www.yikeweiqi.com/golive/dtl?id=444");
    if (transition === "category") await click(button("Local"));
    if (transition === "cancel") await click(button("取消查询"));
    if (transition === "provider") await changeSelect(selectByLabel("Source"), "fox");
    await act(async () => { old.resolve(previewResponse(oldIdentity)); });
    expect(api.cancelProviderRequest).toHaveBeenCalledWith(oldIdentity);
    expect(host.querySelector('section[aria-label="Provider 棋谱预览"]')).toBeNull();
    expect(api.saveYikeLocator).not.toHaveBeenCalled();
    expect(onImport).not.toHaveBeenCalled();
  });

  it("invalidates an already completed preview immediately on locator editing", async () => {
    const onImport = await renderPanel();
    await previewLocator();
    expect(host.querySelector('section[aria-label="Provider 棋谱预览"]')).not.toBeNull();
    await changeInput(inputByLabel("Yike URL"), canonicalLocator + "0");
    expect(host.querySelector('section[aria-label="Provider 棋谱预览"]')).toBeNull();
    expect(onImport).not.toHaveBeenCalled();
  });

  it("new preview fences a late list response and keeps preview routes", async () => {
    const old = deferred<YikeListResult>();
    api.fetchYikeList.mockReturnValueOnce(old.promise);
    const onImport = await renderPanel();
    const oldIdentity = api.fetchYikeList.mock.calls[0][3];
    await previewLocator();
    await act(async () => { old.resolve(listResponse("recommend", 1, 0, oldIdentity)); });
    expect(host.querySelector('[aria-label="Yike 公开棋局"]')).toBeNull();
    expect(host.querySelector('[aria-label="实际网络路由"]')?.textContent).toContain("live/98765");
    expect(host.querySelector('section[aria-label="Provider 棋谱预览"]')).not.toBeNull();
    expect(onImport).not.toHaveBeenCalled();
  });

  it("successful network save fences old preview and queries with the new policy revision", async () => {
    const old = deferred<YikePreviewResult>();
    api.previewYikeLocator.mockReturnValueOnce(old.promise);
    const onImport = await renderPanel();
    await previewLocator();
    const oldIdentity = api.previewYikeLocator.mock.calls[0][1];
    api.saveNetworkSettings.mockResolvedValueOnce({ ...directSnapshot, settings: { ...directSnapshot.settings, mode: "system" }, policy_revision: 2 });
    await changeSelect(selectByLabel("Mode"), "system");
    await click(button("Save"));
    await act(async () => { old.resolve(previewResponse(oldIdentity)); });
    expect(api.cancelProviderRequest).toHaveBeenCalledWith(oldIdentity);
    expect(api.beginProviderRequest).toHaveBeenLastCalledWith(2);
    expect(host.querySelector('section[aria-label="Provider 棋谱预览"]')).toBeNull();
    expect(onImport).not.toHaveBeenCalled();
  });

  it("failed network policy save preserves the completed preview", async () => {
    const onImport = await renderPanel();
    await previewLocator();
    api.saveNetworkSettings.mockRejectedValueOnce(new Error("Storage disk failure"));
    await changeSelect(selectByLabel("Mode"), "manual");
    await click(button("Save"));
    expect(host.querySelector('[data-testid="network-settings-error"]')?.textContent).toContain("Storage disk failure");
    expect(host.querySelector('section[aria-label="Provider 棋谱预览"]')).not.toBeNull();
    await click(button("导入预览棋谱"));
    expect(onImport).toHaveBeenCalledTimes(1);
  });

  it.each(["request", "begin"])("closing the panel fences pending %s and cancels its exact lease", async (stage) => {
    const pendingBegin = deferred<ProviderRequestIdentity>();
    const pendingPreview = deferred<YikePreviewResult>();
    const onImport = await renderPanel();
    if (stage === "begin") api.beginProviderRequest.mockReturnValueOnce(pendingBegin.promise);
    else api.previewYikeLocator.mockReturnValueOnce(pendingPreview.promise);
    await previewLocator();
    const lease = stage === "begin" ? { request_id: 999, policy_revision: 1, document_identity: 7001 } : api.previewYikeLocator.mock.calls[0][1];
    await act(async () => { root?.unmount(); root = null; });
    await act(async () => {
      if (stage === "begin") pendingBegin.resolve(lease);
      else pendingPreview.resolve(previewResponse(lease));
    });
    expect(api.cancelProviderRequest).toHaveBeenCalledWith(lease);
    expect(api.saveYikeLocator).not.toHaveBeenCalled();
    expect(onImport).not.toHaveBeenCalled();
  });

  it("freezes query and provider controls while awaiting one explicit import callback without claiming commitment", async () => {
    const pending = deferred<void>();
    const onImport = vi.fn<(result: ProviderImportResult, identity?: ProviderRequestIdentity) => void | Promise<void>>(() => pending.promise);
    await renderPanel("yike", onImport);
    await previewLocator();
    await click(button("导入预览棋谱"));
    expect(button("Local").disabled).toBe(true);
    expect(inputByLabel("Yike URL").disabled).toBe(true);
    expect(selectByLabel("Source").disabled).toBe(true);
    expect(button("导入预览棋谱").disabled).toBe(true);
    await click(button("导入预览棋谱"));
    expect(onImport).toHaveBeenCalledTimes(1);
    await act(async () => { pending.resolve(); });
    expect(host.textContent).not.toMatch(/Imported|导入成功/);
    expect(button("Local").disabled).toBe(false);
  });
});

describe("ProviderPanel preserved local and Fox paths", () => {
  it.each(["provider", "policy", "disabled", "document"])("Fox late preview is fenced after %s changes", async (transition) => {
    const pending = deferred<FoxPreviewResult>();
    api.beginProviderRequest.mockResolvedValueOnce(foxIdentity);
    api.previewFoxGame.mockReturnValueOnce(pending.promise);
    api.rememberFoxLookup.mockResolvedValue({ recents: [], last_query: null });
    const onImport = await renderPanel("fox");
    await click(button("棋谱 chessid"));
    await changeInput(inputByLabel("棋谱 chessid"), "123456");
    await click(button("预览棋局"));
    if (transition === "provider") await changeSelect(selectByLabel("Source"), "tencent");
    if (transition === "policy") {
      api.saveNetworkSettings.mockResolvedValueOnce({ ...directSnapshot, policy_revision: 2 });
      await click(button("Save"));
    }
    if (transition === "disabled") {
      await act(async () => { root?.render(<ProviderPanel initialProvider="fox" disabled onImport={onImport} />); });
    }
    if (transition === "document") {
      await act(async () => { root?.render(<ProviderPanel key="replacement-document" initialProvider="fox" onImport={onImport} />); });
    }
    await act(async () => { pending.resolve(foxPreviewResult); });
    expect(api.cancelProviderRequest).toHaveBeenCalledWith(foxIdentity);
    expect(host.querySelector('section[aria-label="Provider 棋谱预览"]')).toBeNull();
    expect(onImport).not.toHaveBeenCalled();
  });

  it("Fox chessid lookup previews one game; only explicit import admits it with its identity", async () => {
    api.beginProviderRequest.mockResolvedValueOnce(foxIdentity);
    api.previewFoxGame.mockResolvedValueOnce(foxPreviewResult);
    api.rememberFoxLookup.mockResolvedValue({ recents: [], last_query: { kind: "chessid", value: "123456" } });

    const onImport = await renderPanel("fox");
    await click(button("棋谱 chessid"));
    await changeInput(inputByLabel("棋谱 chessid"), " 123456 ");
    await click(button("预览棋局"));

    expect(api.previewFoxGame).toHaveBeenCalledWith("123456", foxIdentity);
    expect(api.fetchFoxList).not.toHaveBeenCalled();
    expect(api.rememberFoxLookup).toHaveBeenCalledWith({ kind: "chessid", value: "123456" }, null);
    expect(host.querySelector('section[aria-label="Provider 棋谱预览"]')?.textContent).toContain("PlayerA vs PlayerB");
    expect(onImport).not.toHaveBeenCalled();

    await click(button("导入预览棋谱"));
    expect(onImport).toHaveBeenCalledTimes(1);
    expect(onImport).toHaveBeenCalledWith(foxImportResult, foxIdentity);
    expect(api.fetchYikeList).not.toHaveBeenCalled();
  });

  it("Fox nickname list pages by 25, continues from the frozen account cursor, and selection only previews", async () => {
    const first = foxGames(100, 5000);
    const second = foxGames(31, 4900);
    api.beginProviderRequest
      .mockResolvedValueOnce(foxIdentity)
      .mockResolvedValueOnce({ ...foxIdentity, request_id: 103 })
      .mockResolvedValueOnce({ ...foxIdentity, request_id: 104 });
    api.fetchFoxList.mockResolvedValueOnce(foxList(foxIdentity, first, "4901", true));
    api.fetchFoxListMore.mockResolvedValueOnce(foxList({ ...foxIdentity, request_id: 103 }, [first[99], ...second], "4870", false));
    api.previewFoxGame.mockResolvedValueOnce({ ...foxPreviewResult, identity: { ...foxIdentity, request_id: 104 } });
    api.rememberFoxLookup.mockResolvedValue({ recents: [foxAccount], last_query: { kind: "nickname", value: "绝艺" } });

    const onImport = await renderPanel("fox");
    await changeInput(inputByLabel("昵称"), "绝艺");
    await click(button("查询棋谱"));

    expect(api.fetchFoxList).toHaveBeenCalledWith({ kind: "nickname", value: "绝艺" }, foxIdentity);
    expect(api.rememberFoxLookup).toHaveBeenCalledWith({ kind: "nickname", value: "绝艺" }, foxAccount);
    expect(foxRows()).toHaveLength(25);
    expect(host.textContent).toContain("第 1 / 4… 页");
    for (let page = 0; page < 3; page += 1) await click(button("下一页"));
    expect(api.fetchFoxListMore).not.toHaveBeenCalled();
    expect(foxRows()[0].textContent).toContain("76.");

    await click(button("下一页"));
    expect(api.fetchFoxListMore).toHaveBeenCalledWith(foxAccount, "4901", { ...foxIdentity, request_id: 103 });
    expect(host.textContent).toContain("第 5 / 6 页");
    expect(foxRows()[0].textContent).toContain("101.");
    await click(button("下一页"));
    expect(foxRows()).toHaveLength(6);
    expect(button("下一页").disabled).toBe(true);
    expect(host.textContent).toContain("已到列表末尾。");

    await click(foxRows()[0].querySelector("button")!);
    expect(api.previewFoxGame).toHaveBeenCalledWith("4875", { ...foxIdentity, request_id: 104 });
    expect(onImport).not.toHaveBeenCalled();
  });

  it("a Fox lookup switch seals the old request so its late list cannot overwrite the new one", async () => {
    const stale = deferred<FoxListResult>();
    const nextIdentity = { ...foxIdentity, request_id: 103 };
    api.beginProviderRequest.mockResolvedValueOnce(foxIdentity).mockResolvedValueOnce(nextIdentity);
    api.fetchFoxList.mockReturnValueOnce(stale.promise).mockResolvedValueOnce(foxList(nextIdentity, foxGames(2, 10), "9", false));
    api.rememberFoxLookup.mockResolvedValue({ recents: [], last_query: null });

    await renderPanel("fox");
    await changeInput(inputByLabel("昵称"), "old");
    await click(button("查询棋谱"));
    await click(button("UID"));
    await changeInput(inputByLabel("UID"), "42");
    expect(api.cancelProviderRequest).toHaveBeenCalledWith(foxIdentity);
    await click(button("查询棋谱"));
    await act(async () => stale.resolve(foxList(foxIdentity, foxGames(25, 900), "876", true)));

    expect(foxRows()).toHaveLength(2);
    expect(foxRows()[0].textContent).toContain("p10");
    expect(api.rememberFoxLookup).toHaveBeenCalledTimes(1);
  });

  it("an empty Fox result, failure and retry are visible and keep the current game untouched", async () => {
    api.beginProviderRequest.mockResolvedValue(foxIdentity);
    api.fetchFoxList
      .mockRejectedValueOnce({ kind: "not_found", message: "No Fox account matches this nickname." })
      .mockResolvedValueOnce(foxList(foxIdentity, [], null, false));
    api.rememberFoxLookup.mockResolvedValue({ recents: [], last_query: null });

    const onImport = await renderPanel("fox");
    await changeInput(inputByLabel("昵称"), "ghost");
    await click(button("查询棋谱"));
    expect(host.querySelector('[role="alert"]')?.textContent).toContain("No Fox account matches this nickname.");
    expect(api.rememberFoxLookup).not.toHaveBeenCalled();

    await click(button("重试查询"));
    expect(api.fetchFoxList).toHaveBeenCalledTimes(2);
    expect(host.textContent).toContain("没有找到棋谱。");
    expect(button("下一页").disabled).toBe(true);
    expect(onImport).not.toHaveBeenCalled();
  });

  it("a Fox continuation that does not end on a page boundary shows every new game in order", async () => {
    api.beginProviderRequest.mockResolvedValue(foxIdentity);
    api.fetchFoxList.mockResolvedValueOnce(foxList(foxIdentity, foxGames(99, 5000), "4902", true));
    api.fetchFoxListMore.mockResolvedValueOnce(foxList(foxIdentity, foxGames(30, 4901), "4872", false));
    api.rememberFoxLookup.mockResolvedValue({ recents: [], last_query: null });

    await renderPanel("fox");
    await changeInput(inputByLabel("昵称"), "绝艺");
    await click(button("查询棋谱"));
    for (let page = 0; page < 3; page += 1) await click(button("下一页"));
    expect(foxRows()).toHaveLength(24);

    await click(button("下一页"));
    expect(host.textContent).toContain("第 4 / 6 页");
    expect(foxRows()).toHaveLength(25);
    expect(foxRows()[24].textContent).toContain("100. ");
    expect(foxRows()[24].textContent).toContain("p4901");
  });

  it("navigating back while a Fox continuation loads keeps the chosen page", async () => {
    const more = deferred<FoxListResult>();
    api.beginProviderRequest.mockResolvedValue(foxIdentity);
    api.fetchFoxList.mockResolvedValueOnce(foxList(foxIdentity, foxGames(100, 5000), "4901", true));
    api.fetchFoxListMore.mockReturnValueOnce(more.promise);
    api.rememberFoxLookup.mockResolvedValue({ recents: [], last_query: null });

    await renderPanel("fox");
    await changeInput(inputByLabel("昵称"), "绝艺");
    await click(button("查询棋谱"));
    for (let page = 0; page < 4; page += 1) await click(button("下一页"));
    await click(button("上一页"));
    await act(async () => more.resolve(foxList(foxIdentity, foxGames(10, 4900), "4891", false)));

    expect(host.textContent).toContain("第 3 / 5 页");
    expect(foxRows()[0].textContent).toContain("51. ");
  });

  it("an empty Fox list keeps the lookup but does not add the account to recents", async () => {
    api.beginProviderRequest.mockResolvedValue(foxIdentity);
    api.fetchFoxList.mockResolvedValueOnce(foxList(foxIdentity, [], null, false));
    api.rememberFoxLookup.mockResolvedValue({ recents: [], last_query: { kind: "uid", value: "100000" } });

    await renderPanel("fox");
    await click(button("UID"));
    await changeInput(inputByLabel("UID"), "100000");
    await click(button("查询棋谱"));

    expect(api.rememberFoxLookup).toHaveBeenCalledWith({ kind: "uid", value: "100000" }, null);
  });

  it("Fox recents restore the last query, re-run by UID and clear without touching the last query", async () => {
    api.loadFoxKifuState.mockResolvedValueOnce({ recents: [foxAccount], last_query: { kind: "uid", value: "8772065" } });
    api.beginProviderRequest.mockResolvedValue(foxIdentity);
    api.fetchFoxList.mockResolvedValue(foxList(foxIdentity, foxGames(1, 3), "3", false));
    api.rememberFoxLookup.mockResolvedValue({ recents: [foxAccount], last_query: { kind: "uid", value: "8772065" } });
    api.clearFoxRecents.mockResolvedValueOnce({ recents: [], last_query: { kind: "uid", value: "8772065" } });

    await renderPanel("fox");
    expect(inputByLabel("UID").value).toBe("8772065");
    await click(button("绝艺 (8772065)"));
    expect(api.fetchFoxList).toHaveBeenCalledWith({ kind: "uid", value: "8772065" }, foxIdentity);

    await click(button("清除最近"));
    expect(api.clearFoxRecents).toHaveBeenCalledTimes(1);
    expect(Array.from(host.querySelectorAll("button")).find((item) => item.textContent?.trim() === "绝艺 (8772065)")).toBeUndefined();
    expect(inputByLabel("UID").value).toBe("8772065");
  });

  it("Tencent entry previews through its own center and retains its import identity", async () => {
    const result: ProviderImportResult = {
      ...foxImportResult,
      provider: "tencent",
      summary: { ...foxImportResult.summary, provider: "tencent" }
    };
    api.previewTencentChess.mockImplementationOnce(async (_chessId: string, identity: ProviderRequestIdentity) => ({ identity, result, routes: [] }));
    const onImport = await renderPanel();
    await changeSelect(selectByLabel("Source"), "tencent");
    expect(host.querySelector('[aria-label="Tencent 棋谱中心"]')).not.toBeNull();
    await changeSelect(selectByLabel("Tencent 查询方式"), "chess_id");
    await changeInput(inputByLabel("Tencent chessId"), "123456");
    await click(button("预览棋局"));
    expect(api.previewTencentChess).toHaveBeenCalledWith("123456", expect.any(Object));
    expect(api.previewFoxGame).not.toHaveBeenCalled();
    expect(onImport).not.toHaveBeenCalled();
    await click(button("导入预览棋谱"));
    expect(onImport).toHaveBeenCalledWith(result, api.previewTencentChess.mock.calls[0][1]);
  });

  it("pasted SGF remains a separate offline import path", async () => {
    api.importProviderPayload.mockResolvedValueOnce(yikeImportResult);
    const onImport = await renderPanel();
    const payload = host.querySelector<HTMLTextAreaElement>('[aria-label="Provider payload or SGF"]')!;
    await changeInput(payload, yikeImportResult.sgf_text);
    await click(button("Import pasted payload"));
    expect(api.importProviderPayload).toHaveBeenCalledWith(expect.objectContaining({ provider: "yike", payload: yikeImportResult.sgf_text }));
    expect(onImport).toHaveBeenCalledTimes(1);
    expect(onImport).toHaveBeenCalledWith(yikeImportResult);
    expect(api.previewYikeLocator).not.toHaveBeenCalled();
  });

  it("readboard snapshot preview never imports the current game", async () => {
    api.syncReadboardSidecarSnapshot.mockResolvedValueOnce({ snapshot_id: "snapshot-1", position: null, warnings: [] });
    const onImport = await renderPanel();
    const line = host.querySelector<HTMLTextAreaElement>('[aria-label="Readboard protocol preview line"]')!;
    await changeInput(line, "snapshot protocol input");
    await click(button("Preview snapshot"));
    expect(api.syncReadboardSidecarSnapshot).toHaveBeenCalledWith(expect.objectContaining({ sgf_text: "snapshot protocol input" }));
    expect(onImport).not.toHaveBeenCalled();
  });
});
