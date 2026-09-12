// @vitest-environment jsdom

import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { AnalysisFrameDto, CurrentGameResultDto, GameDto, NodePath, SgfTreeNodeDto } from "./domain/types";
import { defaultAppPreferences, type AppPreferences } from "./domain/preferences";

const currentGameFixture = vi.hoisted(() => vi.fn());
const backend = vi.hoisted(() => ({
  getHealth: vi.fn(() => Promise.resolve({ status: "ok" })),
  prepareDocumentReplacement: vi.fn(async () => ({ status: "ready", departure_id: 1 })),
  prepareApplicationExit: vi.fn(async () => ({ status: "ready", departure_id: 1 })),
  resolveApplicationExit: vi.fn(async (input: { action: string }) => {
    if (input.action === "cancel") {
      return { committed: false, analysis_stopped: false, current: null, message: "Exit cancelled.", disposition: null, teardown: null };
    }
    const current = await currentGameFixture("", null);
    return { committed: true, analysis_stopped: true, current, message: "Application exit completed.", disposition: "clean_completed", teardown: { status: "completed" } };
  }),
  retryApplicationTeardown: vi.fn(async () => ({ committed: true, analysis_stopped: true, current: null, message: "Application exit completed.", disposition: "clean_completed", teardown: { status: "completed" } })),
  confirmApplicationExitAnyway: vi.fn(async () => ({ committed: true, analysis_stopped: true, current: null, message: "Application exit completed.", disposition: "exit_incomplete", teardown: { status: "timed_out", outstanding: ["foreground engine"] } })),
  confirmNativeExit: vi.fn(async () => undefined),
  subscribeApplicationExitRequested: vi.fn(async () => () => undefined),
  resolveDocumentReplacement: vi.fn(async (input: { action: string }) => {
    if (input.action === "cancel") {
      return { committed: false, analysis_stopped: false, current: null, message: "Replacement cancelled." };
    }
    const current = await currentGameFixture("", null);
    return { committed: true, analysis_stopped: true, current, message: "Replacement committed." };
  }),
  serializeCurrentGame: vi.fn(() => Promise.resolve("(;SZ[19])")),
  projectCurrentGameMainline: vi.fn(),
  playCurrentGame: vi.fn(),
  selectCurrentGameNode: vi.fn(),
  startSelectedNodeAnalysis: vi.fn(),
  cancelSelectedNodeAnalysis: vi.fn(),
  cancelKataGoAnalysis: vi.fn(),
  classifyProblems: vi.fn(),
  fakeAnalyze: vi.fn(),
  listenToKataGoAnalysisEvents: vi.fn(),
  openSgfDocument: vi.fn(),
  parseSgfSummary: vi.fn(),
  replaySgfPositions: vi.fn(),
  saveCurrentGame: vi.fn(),
  setCurrentGamePersonalComment: vi.fn(),
  removeCurrentGameVariation: vi.fn(),
  startKataGoGameAnalysis: vi.fn(),
  loadEngineProfilesSettings: vi.fn(() => Promise.resolve({ selected_profile_id: "default", profiles: [] })),
  subscribeForegroundEngine: vi.fn(() => Promise.resolve(() => undefined)),
  startForegroundEngine: vi.fn(),
  stopForegroundEngine: vi.fn(),
  restartForegroundEngine: vi.fn(),
  switchForegroundEngine: vi.fn(),
  getForegroundEngineSnapshot: vi.fn(() => Promise.resolve({ revision: 0, lifecycle: { state: "no_engine" }, continuous: { enabled: null, phase: "loading" } })),
  foregroundEngineContinuousAction: vi.fn(),
  inspectCurrentGameRecovery: vi.fn(async (): Promise<{ status: "none" | "abnormal" | "normal" | "unreadable"; envelope?: unknown; message?: string }> => ({ status: "none" })),
  restoreCurrentGameRecovery: vi.fn(),
  discardCurrentGameRecovery: vi.fn(async () => undefined),
  retryCurrentGameRecovery: vi.fn(async () => ({ status: "protected" })),
  currentGameRecoveryProtection: vi.fn(async () => ({ status: "protected" })),
  subscribeCurrentGameRecoveryProtection: vi.fn(async () => () => undefined)
}));

vi.mock("./api/backend", () => ({
  ...backend,
  isTauriRuntime: () => true,
  nativeCurrentGameUnavailable: "Native current-game commands require the Tauri desktop runtime."
}));

const preferencesApi = vi.hoisted(() => ({
  loadAppPreferences: vi.fn(),
  saveAppPreferences: vi.fn()
}));

vi.mock("./api/preferences", async (importOriginal) => {
  const original = await importOriginal<typeof import("./api/preferences")>();
  return {
    ...original,
    loadAppPreferences: preferencesApi.loadAppPreferences,
    saveAppPreferences: preferencesApi.saveAppPreferences
  };
});

vi.mock("./components/EngineSetupPanel", () => ({ EngineSetupPanel: () => null }));
vi.mock("./components/ProviderPanel", () => ({ ProviderPanel: () => null }));
vi.mock("./components/WinrateChart", () => ({
  WinrateChart: () => <canvas aria-label="胜率走势" />
}));

import { App } from "./App";

const emptyPosition = {
  board_size: 19,
  move_number: 0,
  to_play: "black" as const,
  stones: [],
  captures_black: 0,
  captures_white: 0,
  last_move: null,
  errors: []
};

function lz(header: string): { key: string; values: string[] } {
  return { key: "LZ", values: [`${header}\nmove D16 visits 10 winrate 5000 pv D16`] };
}

function lzop(header: string): { key: string; values: string[] } {
  return { key: "LZOP", values: [`${header}\nmove D16 visits 10 winrate 5000 pv D16`] };
}

function node(properties: SgfTreeNodeDto["properties"], children: SgfTreeNodeDto[] = []): SgfTreeNodeDto {
  return { properties, children };
}

const markerTree: SgfTreeNodeDto = node([lzop("Engine 40.0 100")], [
  node([{ key: "B", values: ["dd"] }, lz("Engine 52.0 100")], [
    node([{ key: "W", values: ["pp"] }, lz("Engine 40.0 100")])
  ]),
  node([{ key: "B", values: ["pq"] }]),
  node([{ key: "B", values: [""] }]),
  node([{ key: "C", values: ["note"] }])
]);

const treeWithoutPayloads: SgfTreeNodeDto = node([], [
  node([{ key: "B", values: ["dd"] }], [
    node([{ key: "W", values: ["pp"] }])
  ]),
  node([{ key: "B", values: ["pq"] }]),
  node([{ key: "B", values: [""] }]),
  node([{ key: "C", values: ["note"] }])
]);

const staleFrame: AnalysisFrameDto = {
  job_id: "stale",
  turn: 0,
  visits: 400,
  winrate_black: 0.52,
  score_mean_black: 1.2,
  candidates: []
};

const initialProjection: GameDto = {
  summary: { id: "test", board_size: 19, komi: 7.5, move_count: 2 },
  moves: [
    { move_number: 1, color: "black", vertex: { point: { x: 3, y: 3 } } },
    { move_number: 2, color: "white", vertex: { point: { x: 15, y: 15 } } }
  ]
};

let root: Root | null = null;
let activeTree: SgfTreeNodeDto = markerTree;
let snapshotAnalysis: AnalysisFrameDto | null = null;

function gameAt(path: NodePath): CurrentGameResultDto {
  return {
    tree: activeTree,
    selected_path: path,
    snapshot: {
      path,
      position: {
        ...emptyPosition,
        move_number: path.indices.length,
        to_play: path.indices.length % 2 === 0 ? "black" : "white"
      },
      personal_comment: "",
      primary_analysis: snapshotAnalysis
    },
    generation: 1,
    dirty: false,
    native_path: null
  };
}

beforeEach(() => {
  globalThis.IS_REACT_ACT_ENVIRONMENT = true;
  activeTree = markerTree;
  snapshotAnalysis = null;
  vi.spyOn(HTMLCanvasElement.prototype, "getContext").mockImplementation(function (this: HTMLCanvasElement) {
    return canvasContext(this);
  });
  currentGameFixture.mockResolvedValue(gameAt({ indices: [] }));
  backend.projectCurrentGameMainline.mockResolvedValue(initialProjection);
  backend.selectCurrentGameNode.mockImplementation(async (path: NodePath) => gameAt(path));
  preferencesApi.loadAppPreferences.mockResolvedValue({ preferences: defaultAppPreferences });
  preferencesApi.saveAppPreferences.mockImplementation(async (preferences: AppPreferences) => preferences);
});

afterEach(() => {
  act(() => root?.unmount());
  root = null;
  document.body.replaceChildren();
  vi.clearAllMocks();
  vi.restoreAllMocks();
});

describe("下一手标记", () => {
  it("首次使用默认变化，只圈出坐标变化且强调主变化", async () => {
    const host = await renderApp();
    backend.startSelectedNodeAnalysis.mockClear();
    expect(board(host).dataset.nextMoveMode).toBe("variations");
    expect(markersOf(host)).toEqual([
      { point: { x: 3, y: 3 }, primary: true, rank: null },
      { point: { x: 15, y: 16 }, primary: false, rank: null }
    ]);
    expect(backend.startSelectedNodeAnalysis).not.toHaveBeenCalled();
  });

  it("关闭清空标记，分级只给主变化着色且不启动分析、不显示不确定", async () => {
    const host = await renderApp();
    backend.startSelectedNodeAnalysis.mockClear();
    await pressJ();
    expect(board(host).dataset.nextMoveMode).toBe("graded");
    expect(markersOf(host)).toEqual([
      { point: { x: 3, y: 3 }, primary: true, rank: "mistake" },
      { point: { x: 15, y: 16 }, primary: false, rank: null }
    ]);
    expect(host.textContent).not.toContain("不确定");
    expect(backend.startSelectedNodeAnalysis).not.toHaveBeenCalled();
    expect(preferencesApi.saveAppPreferences).toHaveBeenLastCalledWith({
      ...defaultAppPreferences,
      nextMoveReviewMarker: "graded"
    });
    await pressJ();
    expect(board(host).dataset.nextMoveMode).toBe("off");
    expect(markersOf(host)).toEqual([]);
    await pressJ();
    expect(board(host).dataset.nextMoveMode).toBe("variations");
  });

  it("J、显示菜单和参数面板共用同一偏好，可编辑控件吞掉 J", async () => {
    const host = await renderApp();
    act(() => buttonNamed(host, "显示").click());
    act(() => menuCheck(host, "关闭").click());
    await flushLast(preferencesApi.saveAppPreferences);
    expect(board(host).dataset.nextMoveMode).toBe("off");
    openPreferences(host);
    act(() => {
      const select = labeledSelect(host, "下一手标记");
      select.value = "graded";
      select.dispatchEvent(new Event("change", { bubbles: true }));
    });
    await flushLast(preferencesApi.saveAppPreferences);
    expect(board(host).dataset.nextMoveMode).toBe("graded");
    expect(labeledSelect(host, "下一手标记").value).toBe("graded");
    act(() => {
      const comment = requiredElement<HTMLTextAreaElement>(host, 'textarea[aria-label="个人评论"]');
      comment.focus();
      comment.dispatchEvent(new KeyboardEvent("keydown", { key: "j", bubbles: true }));
    });
    expect(board(host).dataset.nextMoveMode).toBe("graded");
  });

  it("偏好写入失败时回滚可见状态", async () => {
    preferencesApi.saveAppPreferences.mockRejectedValue(new Error("disk full"));
    const host = await renderApp();
    expect(board(host).dataset.nextMoveMode).toBe("variations");
    act(() => window.dispatchEvent(new KeyboardEvent("keydown", { key: "j", bubbles: true })));
    await act(async () => {
      await preferencesApi.saveAppPreferences.mock.results.at(-1)?.value.catch(() => undefined);
    });
    expect(board(host).dataset.nextMoveMode).toBe("variations");
    openPreferences(host);
    expect(host.querySelector(".preferences-header span")?.textContent).toBe("Save failed: disk full");
  });

  it("下一变化后分级跟随新的主变化", async () => {
    const host = await renderApp();
    openPreferences(host);
    act(() => {
      const select = labeledSelect(host, "下一手标记");
      select.value = "graded";
      select.dispatchEvent(new Event("change", { bubbles: true }));
    });
    await flushLast(preferencesApi.saveAppPreferences);
    act(() => buttonNamed(host, "下一变化").click());
    await flushLast(backend.selectCurrentGameNode);
    expect(board(host).dataset.nextMoveMode).toBe("graded");
    expect(markersOf(host)).toEqual([
      { point: { x: 15, y: 15 }, primary: true, rank: "mistake" }
    ]);
  });

  it("过期会话快照不会给分级着色，也不启动分析", async () => {
    activeTree = treeWithoutPayloads;
    snapshotAnalysis = staleFrame;
    currentGameFixture.mockResolvedValue(gameAt({ indices: [] }));
    const host = await renderApp();
    backend.startSelectedNodeAnalysis.mockClear();
    await pressJ();
    expect(board(host).dataset.nextMoveMode).toBe("graded");
    expect(markersOf(host)).toEqual([
      { point: { x: 3, y: 3 }, primary: true, rank: null },
      { point: { x: 15, y: 16 }, primary: false, rank: null }
    ]);
    expect(backend.startSelectedNodeAnalysis).not.toHaveBeenCalled();
  });

  it("快捷键参考列出下一手标记", async () => {
    const host = await renderApp();
    act(() => buttonNamed(host, "帮助").click());
    act(() => buttonNamed(host, "快捷键参考(?)").click());
    expect(host.textContent).toContain("下一手标记");
    expect(host.textContent).toMatch(/\bJ\b/);
  });
});

async function renderApp(): Promise<HTMLElement> {
  act(() => root?.unmount());
  root = null;
  const host = document.createElement("div");
  document.body.append(host);
  root = createRoot(host);
  act(() => root?.render(<App />));
  await act(async () => {
    await backend.inspectCurrentGameRecovery.mock.results.at(-1)?.value;
    await currentGameFixture.mock.results.at(-1)?.value;
    await backend.projectCurrentGameMainline.mock.results.at(-1)?.value;
    await preferencesApi.loadAppPreferences.mock.results.at(-1)?.value.catch(() => undefined);
  });
  return host;
}

async function pressJ() {
  act(() => window.dispatchEvent(new KeyboardEvent("keydown", { key: "j", bubbles: true })));
  await flushLast(preferencesApi.saveAppPreferences);
}

async function flushLast(mock: { mock: { results: Array<{ value?: unknown }> } }) {
  await act(async () => {
    await mock.mock.results.at(-1)?.value;
  });
}

function openPreferences(host: HTMLElement) {
  act(() => buttonNamed(host, "参数").click());
}

function board(host: HTMLElement): HTMLElement {
  return requiredElement(host, "[data-next-move-mode]");
}

function markersOf(host: HTMLElement) {
  return JSON.parse(board(host).dataset.nextMoveMarkers ?? "[]");
}

function labeledSelect(host: HTMLElement, name: string): HTMLSelectElement {
  const label = [...host.querySelectorAll(".preferences-panel label")].find((candidate) => (
    candidate.querySelector("span")?.textContent === name
  ));
  const control = label?.querySelector("select");
  if (!(control instanceof HTMLSelectElement)) throw new Error(`Missing preference control: ${name}`);
  return control;
}

function menuCheck(host: HTMLElement, name: string): HTMLButtonElement {
  const button = [...host.querySelectorAll('[role="menuitemcheckbox"]')].find((candidate) => (
    candidate.textContent === name
  ));
  if (!(button instanceof HTMLButtonElement)) throw new Error(`Missing menu check: ${name}`);
  return button;
}

function buttonNamed(host: HTMLElement, name: string): HTMLButtonElement {
  const button = [...host.querySelectorAll("button")].find((candidate) => candidate.textContent === name);
  if (!(button instanceof HTMLButtonElement)) throw new Error(`Missing button: ${name}`);
  return button;
}

function requiredElement<T extends Element = HTMLElement>(host: HTMLElement, selector: string): T {
  const element = host.querySelector(selector);
  if (!element) throw new Error(`Missing element: ${selector}`);
  return element as T;
}

function canvasContext(canvas: HTMLCanvasElement): CanvasRenderingContext2D {
  const target = { canvas } as CanvasRenderingContext2D;
  return new Proxy(target, {
    get(object, property) {
      if (property in object) return Reflect.get(object, property);
      return vi.fn();
    },
    set(object, property, value) {
      Reflect.set(object, property, value);
      return true;
    }
  });
}
