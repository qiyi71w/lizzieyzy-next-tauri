// @vitest-environment jsdom

import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { AnalysisFrameDto, CurrentGameResultDto, GameDto, NodePath } from "./domain/types";
import { defaultAppPreferences, type AppPreferences } from "./domain/preferences";

const listeners: {
  onSnapshot?: (snapshot: unknown) => void;
  onJob?: (job: unknown) => void;
} = {};

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
  serializeCurrentGame: vi.fn(() => Promise.resolve("(;SZ[9])")),
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
  loadEngineProfilesSettings: vi.fn(),
  saveEngineProfilesSettings: vi.fn(),
  checkEngineAssets: vi.fn(),
  startForegroundEngine: vi.fn(() => Promise.resolve()),
  stopForegroundEngine: vi.fn(() => Promise.resolve()),
  restartForegroundEngine: vi.fn(() => Promise.resolve()),
  switchForegroundEngine: vi.fn(() => Promise.resolve()),
  getForegroundEngineSnapshot: vi.fn(),
  subscribeForegroundEngine: vi.fn(),
  inspectCurrentGameRecovery: vi.fn(async (): Promise<{ status: "none" | "abnormal" | "normal" | "unreadable"; envelope?: unknown; message?: string }> => ({ status: "none" })),
  restoreCurrentGameRecovery: vi.fn(),
  discardCurrentGameRecovery: vi.fn(async () => undefined),
  retryCurrentGameRecovery: vi.fn(async () => ({ status: "protected" })),
  currentGameRecoveryProtection: vi.fn(async () => ({ status: "protected" })),
  subscribeCurrentGameRecoveryProtection: vi.fn(async () => () => undefined)
}));

const preferencesApi = vi.hoisted(() => ({
  loadAppPreferences: vi.fn(),
  saveAppPreferences: vi.fn()
}));

vi.mock("./api/backend", () => ({
  ...backend,
  isTauriRuntime: () => true,
  nativeCurrentGameUnavailable: "Native current-game commands require the Tauri desktop runtime."
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
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn() }));

import { App } from "./App";

const emptyPosition = {
  board_size: 9,
  move_number: 0,
  to_play: "black" as const,
  stones: [],
  captures_black: 0,
  captures_white: 0,
  last_move: null,
  errors: []
};

const initialGame: CurrentGameResultDto = {
  tree: {
    properties: [],
    children: [{ properties: [{ key: "B", values: ["fe"] }], children: [] }]
  },
  selected_path: { indices: [] },
  snapshot: { path: { indices: [] }, position: emptyPosition, personal_comment: "" },
  generation: 1,
  dirty: false,
  native_path: null
};

const initialProjection: GameDto = {
  summary: { id: "test", board_size: 9, komi: 7.5, move_count: 1 },
  moves: [{ move_number: 1, color: "black", vertex: { point: { x: 5, y: 4 } } }]
};

const savedProfile = {
  id: "profile-1",
  max_visits: 800,
  profile: {
    name: "Local KataGo",
    engine_path: "/bin/katago",
    model_path: "/models/model.bin",
    config_path: "/configs/analysis.cfg",
    working_dir: "/tmp",
    backend: "kata_go_analysis" as const
  }
};

const capability = {
  adapter_kind: "kata_go_analysis" as const,
  selected_node_analysis: true,
  whole_game_analysis: true,
  protocol_cancel: true
};

const fourMovePv: AnalysisFrameDto["candidates"] = [
  {
    vertex: { point: { x: 3, y: 3 } },
    visits: 40,
    winrate_black: 0.62,
    score_mean_black: 2.8,
    pv: [{ point: { x: 3, y: 3 } }, { point: { x: 2, y: 2 } }, { point: { x: 4, y: 4 } }, { point: { x: 5, y: 5 } }]
  }
];

let root: Root | null = null;

beforeEach(() => {
  globalThis.IS_REACT_ACT_ENVIRONMENT = true;
  vi.spyOn(HTMLCanvasElement.prototype, "getContext").mockImplementation(function (this: HTMLCanvasElement) {
    return canvasContext(this);
  });
  currentGameFixture.mockResolvedValue(initialGame);
  backend.projectCurrentGameMainline.mockResolvedValue(initialProjection);
  backend.selectCurrentGameNode.mockImplementation(async (path: NodePath) => ({
    ...initialGame,
    selected_path: path,
    snapshot: {
      path,
      position: { ...emptyPosition, move_number: path.indices.length },
      personal_comment: ""
    }
  }));
  backend.playCurrentGame.mockResolvedValue(initialGame);
  preferencesApi.loadAppPreferences.mockResolvedValue({ preferences: defaultAppPreferences });
  preferencesApi.saveAppPreferences.mockImplementation(async (preferences: AppPreferences) => preferences);
  backend.loadEngineProfilesSettings.mockResolvedValue({
    selected_profile_id: "profile-1",
    autoload_profile_id: null,
    profiles: [savedProfile]
  });
  backend.getForegroundEngineSnapshot.mockResolvedValue({ revision: 0, lifecycle: { state: "no_engine" } });
  backend.classifyProblems.mockResolvedValue([]);
  backend.subscribeForegroundEngine.mockImplementation(async (onSnapshot, _onFailure, onJob) => {
    listeners.onSnapshot = onSnapshot;
    listeners.onJob = onJob;
    onSnapshot({ revision: 0, lifecycle: { state: "no_engine" } });
    return () => undefined;
  });
});

afterEach(() => {
  act(() => root?.unmount());
  root = null;
  document.body.replaceChildren();
  vi.clearAllMocks();
  vi.restoreAllMocks();
  vi.useRealTimers();
});

describe("one active Variation Replay across eligible boards", () => {
  it("starts disabled and shows the full PV on Variation Sub-Board", async () => {
    const host = await renderApp();
    await readyEngine(host);
    await startSelectedNode(host);
    await completeSelectedNode("job-1", { indices: [] }, analysisFrame());

    act(() => buttonNamed(host, "显示").click());
    expect(menuCheck(host, "变化回放").getAttribute("aria-checked")).toBe("false");
    openPreferences(host);
    expect(labeledCheckbox(host, "变化回放").checked).toBe(false);
    expect(labeledNumber(host, "回放间隔").value).toBe("500");
    expect(subBoardNumbers(host)).toEqual(["1", "2", "3", "4"]);
  });

  it("shows the first PV move immediately on both eligible surfaces, then advances once per interval and idles armed", async () => {
    const host = await renderAnalyzedApp();
    hideCoordinates(host);
    backend.selectCurrentGameNode.mockClear();
    backend.playCurrentGame.mockClear();
    await enableReplay(host);

    expect(subBoardNumbers(host)).toEqual(["1"]);
    expect(mainBoardNumbers(host)).toEqual(["1"]);
    expect(backend.playCurrentGame).not.toHaveBeenCalled();
    expect(host.querySelector(".doc-name")?.textContent?.endsWith(" *")).toBe(false);

    await advanceReplay(499);
    expect(subBoardNumbers(host)).toEqual(["1"]);
    expect(mainBoardNumbers(host)).toEqual(["1"]);

    await advanceReplay(1);
    expect(subBoardNumbers(host)).toEqual(["1", "2"]);
    expect(mainBoardNumbers(host)).toEqual(["1", "2"]);

    await advanceReplay(500);
    expect(subBoardNumbers(host)).toEqual(["1", "2", "3"]);
    await advanceReplay(500);
    expect(subBoardNumbers(host)).toEqual(["1", "2", "3", "4"]);
    expect(mainBoardNumbers(host)).toEqual(["1", "2", "3", "4"]);

    await advanceReplay(1500);
    expect(subBoardNumbers(host)).toEqual(["1", "2", "3", "4"]);
    expect(mainBoardNumbers(host)).toEqual(["1", "2", "3", "4"]);
    expect(backend.selectCurrentGameNode).not.toHaveBeenCalled();
    expect(backend.playCurrentGame).not.toHaveBeenCalled();
  });

  it("resets on a new candidate or PV sequence and keeps progress for statistics-only updates", async () => {
    const host = await renderApp();
    await readyEngine(host);
    await startSelectedNode(host);
    await completeSelectedNode("job-1", { indices: [] }, analysisFrame({
      candidates: [
        fourMovePv[0],
        {
          vertex: { point: { x: 6, y: 5 } },
          visits: 30,
          winrate_black: 0.49,
          score_mean_black: -0.5,
          pv: [{ point: { x: 5, y: 5 } }]
        }
      ]
    }));
    hideCoordinates(host);
    await enableReplay(host);
    await advanceReplay(500);
    expect(subBoardNumbers(host)).toEqual(["1", "2"]);

    act(() => (host.querySelectorAll(".cand-row")[1] as HTMLTableRowElement).click());
    expect(subBoardNumbers(host)).toEqual(["1"]);
    expect(mainBoardNumbers(host)).toEqual(["1"]);

    act(() => (host.querySelectorAll(".cand-row")[0] as HTMLTableRowElement).click());
    await advanceReplay(500);
    expect(subBoardNumbers(host)).toEqual(["1", "2"]);

    await startWholeGame(host);
    await emitWholeGameProgress({ indices: [] }, analysisFrame({
      job_id: "job-wg",
      visits: 90,
      winrate_black: 0.58,
      candidates: [{ ...fourMovePv[0], visits: 88, winrate_black: 0.57 }]
    }));
    expect(subBoardNumbers(host)).toEqual(["1", "2"]);
    expect(mainBoardNumbers(host)).toEqual(["1", "2"]);

    await emitWholeGameProgress({ indices: [] }, analysisFrame({
      job_id: "job-wg",
      candidates: [{
        ...fourMovePv[0],
        pv: [{ point: { x: 1, y: 1 } }, { point: { x: 4, y: 4 } }, { point: { x: 5, y: 5 } }]
      }]
    }));
    expect(subBoardNumbers(host)).toEqual(["1"]);
    expect(mainBoardNumbers(host)).toEqual(["1"]);
  });

  it("pauses when no eligible surface is visible and resumes from the same prefix", async () => {
    const host = await renderAnalyzedApp();
    hideCoordinates(host);
    await enableReplay(host);
    await advanceReplay(500);
    expect(subBoardNumbers(host)).toEqual(["1", "2"]);

    act(() => buttonNamed(host, "显示").click());
    act(() => menuCheck(host, "纯棋子").click());
    await flushLast(preferencesApi.saveAppPreferences);
    await advanceReplay(500);
    expect(mainBoardNumbers(host)).toEqual(["1", "2", "3"]);

    act(() => buttonNamed(host, "显示").click());
    act(() => menuCheck(host, "候选").click());
    await flushLast(preferencesApi.saveAppPreferences);
    await advanceReplay(1000);
    expect(subBoardNumbers(host)).toEqual([]);

    act(() => buttonNamed(host, "显示").click());
    act(() => menuCheck(host, "变化图").click());
    await flushLast(preferencesApi.saveAppPreferences);
    act(() => buttonNamed(host, "显示").click());
    act(() => menuCheck(host, "候选").click());
    await flushLast(preferencesApi.saveAppPreferences);
    expect(subBoardNumbers(host)).toEqual(["1", "2", "3"]);
    expect(mainBoardNumbers(host)).toEqual(["1", "2", "3"]);

    act(() => buttonNamed(host, "显示").click());
    act(() => menuCheck(host, "候选").click());
    await flushLast(preferencesApi.saveAppPreferences);
    act(() => buttonNamed(host, "显示").click());
    act(() => menuCheck(host, "选点列表面板").click());
    await advanceReplay(1000);
    expect(host.querySelector(".sheet-col")?.hasAttribute("hidden")).toBe(true);

    act(() => buttonNamed(host, "显示").click());
    act(() => menuCheck(host, "选点列表面板").click());
    act(() => buttonNamed(host, "显示").click());
    act(() => menuCheck(host, "候选").click());
    await flushLast(preferencesApi.saveAppPreferences);
    expect(subBoardNumbers(host)).toEqual(["1", "2", "3"]);
    await advanceReplay(500);
    expect(subBoardNumbers(host)).toEqual(["1", "2", "3", "4"]);
  });

  it("restores full PV when disabled and applies a live interval to the next gap", async () => {
    const host = await renderAnalyzedApp();
    hideCoordinates(host);
    await enableReplay(host);
    await advanceReplay(499);
    expect(subBoardNumbers(host)).toEqual(["1"]);

    openPreferences(host);
    act(() => {
      const input = labeledNumber(host, "回放间隔");
      const setter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")?.set;
      setter?.call(input, "1000");
      input.dispatchEvent(new Event("input", { bubbles: true }));
      input.dispatchEvent(new Event("change", { bubbles: true }));
    });
    await flushLast(preferencesApi.saveAppPreferences);
    expect(labeledNumber(host, "回放间隔").value).toBe("1000");
    await advanceReplay(1);
    expect(subBoardNumbers(host)).toEqual(["1", "2"]);
    await advanceReplay(500);
    expect(subBoardNumbers(host)).toEqual(["1", "2"]);
    await advanceReplay(500);
    expect(subBoardNumbers(host)).toEqual(["1", "2", "3"]);

    act(() => buttonNamed(host, "显示").click());
    act(() => menuCheck(host, "变化回放").click());
    await flushLast(preferencesApi.saveAppPreferences);
    expect(subBoardNumbers(host)).toEqual(["1", "2", "3", "4"]);
    expect(mainBoardNumbers(host)).not.toEqual(["1", "2", "3"]);
  });

  it("keeps replay off when persist fails and restores a saved enablement on the next load", async () => {
    let rejectSave!: (error: Error) => void;
    preferencesApi.saveAppPreferences.mockImplementation(
      () => new Promise((_, reject) => {
        rejectSave = reject;
      })
    );
    const host = await renderAnalyzedApp();
    act(() => buttonNamed(host, "显示").click());
    act(() => menuCheck(host, "变化回放").click());
    expect(subBoardNumbers(host)).toEqual(["1", "2", "3", "4"]);

    await act(async () => {
      rejectSave(new Error("disk full"));
      await preferencesApi.saveAppPreferences.mock.results.at(-1)?.value.catch(() => undefined);
    });
    act(() => buttonNamed(host, "显示").click());
    expect(menuCheck(host, "变化回放").getAttribute("aria-checked")).toBe("false");
    expect(subBoardNumbers(host)).toEqual(["1", "2", "3", "4"]);
    openPreferences(host);
    expect(host.querySelector(".preferences-header span")?.textContent).toBe("Save failed: disk full");

    act(() => root?.unmount());
    root = null;
    document.body.replaceChildren();
    preferencesApi.saveAppPreferences.mockImplementation(async (preferences: AppPreferences) => preferences);
    preferencesApi.loadAppPreferences.mockResolvedValue({
      preferences: { ...defaultAppPreferences, variationReplayEnabled: true }
    });
    const restored = await renderApp();
    await readyEngine(restored);
    await startSelectedNode(restored);
    await completeSelectedNode("job-1", { indices: [] }, analysisFrame());
    expect(subBoardNumbers(restored)).toEqual(["1"]);
  });
});

async function startWholeGame(host: HTMLElement) {
  backend.startKataGoGameAnalysis.mockResolvedValueOnce({
    run_id: "run-1",
    job_id: "job-wg",
    lane: "whole_game",
    mode: "finite",
    state: "queued",
    generation: 1,
    node_path: { indices: [] }
  });
  await act(async () => {
    buttonNamed(host, "分析第一子主线").click();
    await backend.startKataGoGameAnalysis.mock.results.at(-1)?.value;
  });
}

async function emitWholeGameProgress(path: NodePath, frame: AnalysisFrameDto) {
  await act(async () => {
    listeners.onJob?.({
      run_id: "run-1",
      job_id: "job-wg",
      lane: "whole_game",
      mode: "finite",
      generation: 1,
      node_path: path,
      outcome: "progress",
      completed: 1,
      expected: 2,
      remaining: 1,
      frame
    });
  });
}

async function renderAnalyzedApp(): Promise<HTMLElement> {
  const host = await renderApp();
  await readyEngine(host);
  await startSelectedNode(host);
  await completeSelectedNode("job-1", { indices: [] }, analysisFrame());
  return host;
}

async function renderApp(): Promise<HTMLElement> {
  const host = document.createElement("div");
  document.body.append(host);
  root = createRoot(host);
  act(() => root?.render(<App />));
  await act(async () => {
    await backend.inspectCurrentGameRecovery.mock.results.at(-1)?.value;
    await currentGameFixture.mock.results.at(-1)?.value;
    await backend.projectCurrentGameMainline.mock.results.at(-1)?.value;
    await backend.subscribeForegroundEngine.mock.results.at(-1)?.value;
    await preferencesApi.loadAppPreferences.mock.results.at(-1)?.value.catch(() => undefined);
  });
  return host;
}

async function readyEngine(host: HTMLElement) {
  const switcher = host.querySelector('select[aria-label="Foreground Engine Profile"]') as HTMLSelectElement;
  await act(async () => {
    switcher.value = "profile-1";
    switcher.dispatchEvent(new Event("change", { bubbles: true }));
    await backend.startForegroundEngine.mock.results[0]?.value;
  });
  await act(async () => {
    listeners.onSnapshot?.({
      revision: 2,
      lifecycle: {
        state: "ready",
        run: {
          run_id: "run-1",
          profile_id: "profile-1",
          adapter_kind: "kata_go_analysis",
          profile_snapshot: savedProfile.profile,
          capability_snapshot: capability
        }
      }
    });
  });
}

async function startSelectedNode(host: HTMLElement) {
  backend.startSelectedNodeAnalysis.mockResolvedValueOnce({
    run_id: "run-1",
    job_id: "job-1",
    lane: "selected_node",
    mode: "finite",
    state: "queued",
    generation: 1,
    node_path: { indices: [] }
  });
  await act(async () => {
    buttonNamed(host, "分析当前节点").click();
    await backend.startSelectedNodeAnalysis.mock.results.at(-1)?.value;
  });
}

async function completeSelectedNode(jobId: string, path: NodePath, frame: AnalysisFrameDto) {
  await act(async () => {
    listeners.onJob?.({
      run_id: "run-1",
      job_id: jobId,
      lane: "selected_node",
      mode: "finite",
      generation: 1,
      node_path: path,
      outcome: "completed",
      frame
    });
    await backend.classifyProblems.mock.results.at(-1)?.value;
  });
}

async function enableReplay(host: HTMLElement) {
  vi.useFakeTimers();
  act(() => buttonNamed(host, "显示").click());
  act(() => menuCheck(host, "变化回放").click());
  await flushLast(preferencesApi.saveAppPreferences);
}

async function advanceReplay(ms: number) {
  await act(async () => {
    vi.advanceTimersByTime(ms);
  });
}

function hideCoordinates(host: HTMLElement) {
  act(() => buttonNamed(host, "坐标").click());
}

function analysisFrame(overrides: Partial<AnalysisFrameDto> = {}): AnalysisFrameDto {
  return {
    job_id: "job-1",
    turn: 0,
    visits: 48,
    winrate_black: 0.61,
    score_mean_black: 2.7,
    candidates: fourMovePv,
    ...overrides
  };
}

async function flushLast(mock: { mock: { results: Array<{ value?: unknown }> } }) {
  await act(async () => {
    await mock.mock.results.at(-1)?.value;
  });
}

function openPreferences(host: HTMLElement) {
  act(() => buttonNamed(host, "参数").click());
}

function labeledCheckbox(host: HTMLElement, name: string): HTMLInputElement {
  const label = [...host.querySelectorAll(".preferences-panel label")].find((candidate) => (
    candidate.querySelector("span")?.textContent === name
  ));
  const control = label?.querySelector("input[type='checkbox']");
  if (!(control instanceof HTMLInputElement)) throw new Error(`Missing preference checkbox: ${name}`);
  return control;
}

function labeledNumber(host: HTMLElement, name: string): HTMLInputElement {
  const label = [...host.querySelectorAll(".preferences-panel label")].find((candidate) => (
    candidate.querySelector("span")?.textContent === name
  ));
  const control = label?.querySelector("input[type='number']");
  if (!(control instanceof HTMLInputElement)) throw new Error(`Missing preference number: ${name}`);
  return control;
}

function menuCheck(host: HTMLElement, name: string): HTMLButtonElement {
  const button = [...host.querySelectorAll('[role="menuitemcheckbox"]')].find((candidate) => (
    candidate.textContent?.includes(name)
  ));
  if (!(button instanceof HTMLButtonElement)) throw new Error(`Missing menu check: ${name}`);
  return button;
}

function buttonNamed(host: HTMLElement, name: string): HTMLButtonElement {
  const button = [...host.querySelectorAll("button")].find((candidate) => candidate.textContent === name);
  if (!(button instanceof HTMLButtonElement)) throw new Error(`Missing button: ${name}`);
  return button;
}

function subBoardCanvas(host: HTMLElement): HTMLCanvasElement {
  const canvas = host.querySelector("canvas.subboard-canvas");
  if (!(canvas instanceof HTMLCanvasElement)) throw new Error("Missing sub-board canvas");
  return canvas;
}

function mainBoardCanvas(host: HTMLElement): HTMLCanvasElement {
  const canvas = host.querySelector('canvas[aria-label="棋盘"]');
  if (!(canvas instanceof HTMLCanvasElement)) throw new Error("Missing main board canvas");
  return canvas;
}

function subBoardNumbers(host: HTMLElement): string[] {
  return numberedDraws(subBoardCanvas(host));
}

function mainBoardNumbers(host: HTMLElement): string[] {
  return numberedDraws(mainBoardCanvas(host));
}

function numberedDraws(canvas: HTMLCanvasElement): string[] {
  return [...(canvas.dataset.drawn ?? "").matchAll(/\|(\d+)@/g)].map((match) => match[1]);
}

function canvasContext(canvas: HTMLCanvasElement): CanvasRenderingContext2D {
  const target = { canvas } as CanvasRenderingContext2D;
  return new Proxy(target, {
    get(object, property) {
      if (property in object) return Reflect.get(object, property);
      if (property === "clearRect") {
        return () => {
          canvas.dataset.drawn = "";
        };
      }
      if (property === "fillText") {
        return (text: string, x: number, y: number) => {
          canvas.dataset.drawn += `|${text}@${x.toFixed(1)},${y.toFixed(1)}`;
        };
      }
      if (property === "measureText") return () => ({ width: 0 });
      return vi.fn();
    },
    set(object, property, value) {
      Reflect.set(object, property, value);
      return true;
    }
  });
}
