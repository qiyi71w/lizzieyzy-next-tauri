// @vitest-environment jsdom

import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { AnalysisFrameDto, CurrentGameResultDto, GameDto, NodePath } from "./domain/types";
import { defaultAppPreferences } from "./domain/preferences";

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
  foregroundEngineContinuousAction: vi.fn(),
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
  saveAppPreferences: vi.fn(async (preferences: unknown) => preferences)
}));

vi.mock("./api/backend", () => ({
  ...backend,
  isTauriRuntime: () => true,
  nativeCurrentGameUnavailable: "Native current-game commands require the Tauri desktop runtime."
}));

vi.mock("./api/preferences", () => preferencesApi);

vi.mock("./components/PreferencesPanel", () => ({ PreferencesPanel: () => null }));
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

const navigableRoot: CurrentGameResultDto = {
  tree: {
    properties: [],
    children: [{ properties: [{ key: "B", values: ["fe"] }], children: [] }]
  },
  selected_path: { indices: [] },
  snapshot: { path: { indices: [] }, position: emptyPosition, personal_comment: "" },
  generation: 1,
  snapshot_seq: 1,
  dirty: false,
  native_path: null
};

function snapshotAt(path: NodePath): CurrentGameResultDto {
  return {
    ...navigableRoot,
    selected_path: path,
    snapshot: {
      path,
      position: {
        ...emptyPosition,
        move_number: path.indices.length,
        to_play: path.indices.length % 2 === 0 ? "black" : "white"
      },
      personal_comment: ""
    }
  };
}

const initialProjection: GameDto = {
  summary: { id: "game", board_size: 9, komi: 7.5, move_count: 1 },
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

let root: Root | null = null;

beforeEach(() => {
  globalThis.IS_REACT_ACT_ENVIRONMENT = true;
  vi.spyOn(HTMLCanvasElement.prototype, "getContext").mockImplementation(function (this: HTMLCanvasElement) {
    return canvasContext(this);
  });
  preferencesApi.loadAppPreferences.mockResolvedValue({ preferences: defaultAppPreferences });
  currentGameFixture.mockResolvedValue(navigableRoot);
  backend.projectCurrentGameMainline.mockResolvedValue(initialProjection);
  backend.selectCurrentGameNode.mockImplementation(async (path: NodePath) => snapshotAt(path));
  backend.loadEngineProfilesSettings.mockResolvedValue({
    selected_profile_id: "profile-1",
    autoload_profile_id: null,
    profiles: [savedProfile]
  });
  backend.getForegroundEngineSnapshot.mockResolvedValue({ revision: 0, lifecycle: { state: "no_engine" }, continuous: { enabled: null, phase: "loading" } });
  backend.startSelectedNodeAnalysis.mockResolvedValue({
    run_id: "run-1",
    job_id: "job-1",
    lane: "selected_node",
    mode: "finite",
    state: "queued",
    generation: 1,
    node_path: { indices: [] }
  });
  backend.startKataGoGameAnalysis.mockResolvedValue({
    run_id: "run-1",
    job_id: "job-wg",
    lane: "whole_game",
    mode: "finite",
    state: "queued",
    generation: 1,
    node_path: { indices: [] }
  });
  backend.classifyProblems.mockResolvedValue([]);
  backend.subscribeForegroundEngine.mockImplementation(async (onSnapshot, _onFailure, onJob) => {
    listeners.onSnapshot = onSnapshot;
    listeners.onJob = onJob;
    onSnapshot({ revision: 0, lifecycle: { state: "no_engine" }, continuous: { enabled: null, phase: "loading" } });
    return () => undefined;
  });
});

afterEach(() => {
  act(() => root?.unmount());
  root = null;
  document.body.replaceChildren();
  vi.clearAllMocks();
  vi.restoreAllMocks();
});

async function renderApp() {
  const host = document.createElement("div");
  document.body.append(host);
  root = createRoot(host);
  act(() => root?.render(<App />));
  await act(async () => {
    await backend.inspectCurrentGameRecovery.mock.results.at(-1)?.value;
      await currentGameFixture.mock.results[0]?.value;
    await backend.projectCurrentGameMainline.mock.results[0]?.value;
    await backend.subscribeForegroundEngine.mock.results[0]?.value;
    await preferencesApi.loadAppPreferences.mock.results[0]?.value;
  });
  return host;
}

async function readyEngine(host: HTMLElement, runId = "run-1") {
  const switcher = host.querySelector('select[aria-label="Foreground Engine Profile"]') as HTMLSelectElement;
  await act(async () => {
    switcher.value = "profile-1";
    switcher.dispatchEvent(new Event("change", { bubbles: true }));
    await backend.startForegroundEngine.mock.results[0]?.value;
  });
  await act(async () => {
    listeners.onSnapshot?.({
      revision: 2,
      continuous: { enabled: true, phase: "waiting" },
      lifecycle: {
        state: "ready",
        run: {
          run_id: runId,
          profile_id: "profile-1",
          adapter_kind: "kata_go_analysis",
          profile_snapshot: savedProfile.profile,
          capability_snapshot: capability
        }
      }
    });
  });
}

function buttonNamed(host: HTMLElement, label: string) {
  return Array.from(host.querySelectorAll("button")).find((button) => button.textContent === label) as HTMLButtonElement;
}

function candidateCoords(host: HTMLElement): string[] {
  return [...host.querySelectorAll(".cand-coord")].map((cell) => cell.textContent ?? "");
}

function candidateAt(x: number, y: number, pv: AnalysisFrameDto["candidates"][number]["pv"] = [], visits = 40) {
  return {
    vertex: { point: { x, y } },
    visits,
    winrate_black: 0.62,
    score_mean_black: 2.8,
    pv
  };
}

function candidate(index: number, pv: AnalysisFrameDto["candidates"][number]["pv"] = []) {
  return candidateAt(index % 9, Math.floor(index / 9), pv, 40 - index);
}

function analysisFrame(overrides: Partial<AnalysisFrameDto> = {}): AnalysisFrameDto {
  return {
    job_id: "job-1",
    turn: 0,
    visits: 48,
    winrate_black: 0.61,
    score_mean_black: 2.7,
    candidates: [candidateAt(3, 3, [{ point: { x: 3, y: 3 } }, { point: { x: 2, y: 2 } }])],
    ...overrides
  };
}

async function startSelectedNode(host: HTMLElement, jobId = "job-1", path: NodePath = { indices: [] }) {
  backend.startSelectedNodeAnalysis.mockResolvedValueOnce({
    run_id: "run-1",
    job_id: jobId,
    lane: "selected_node",
    mode: "finite",
    state: "queued",
    generation: 1,
    node_path: path
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
    await Promise.all([
      backend.classifyProblems.mock.results.at(-1)?.value,
      backend.selectCurrentGameNode.mock.results.at(-1)?.value
    ]);
  });
}

async function emitWholeGameProgress(path: NodePath, frame: AnalysisFrameDto, progress = { completed: 1, expected: 2, remaining: 1 }) {
  await act(async () => {
    listeners.onJob?.({
      run_id: "run-1",
      job_id: "job-wg",
      lane: "whole_game",
      mode: "finite",
      generation: 1,
      node_path: path,
      outcome: "progress",
      ...progress,
      frame
    });
    await backend.selectCurrentGameNode.mock.results.at(-1)?.value;
  });
}

async function startWholeGame(host: HTMLElement) {
  await act(async () => {
    buttonNamed(host, "分析第一子主线").click();
    await backend.startKataGoGameAnalysis.mock.results.at(-1)?.value;
  });
}

async function selectPath(host: HTMLElement, label: string) {
  await act(async () => {
    buttonNamed(host, label).click();
    await backend.selectCurrentGameNode.mock.results.at(-1)?.value;
  });
}

async function hoverBoard(host: HTMLElement) {
  const board = host.querySelector('canvas[aria-label="棋盘"]') as HTMLCanvasElement;
  board.getBoundingClientRect = () => new DOMRect(0, 0, 100, 100);
  await act(async () => {
    board.dispatchEvent(new MouseEvent("pointermove", {
      bubbles: true,
      clientX: 70.5,
      clientY: 60.25
    }));
    await new Promise((resolve) => setTimeout(resolve, 130));
  });
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

describe("analysis presentation bound to exact nodes", () => {
  it("projects candidate limit from durable preferences without a second hardcoded cap", async () => {
    preferencesApi.loadAppPreferences.mockResolvedValue({
      preferences: { ...defaultAppPreferences, candidateLimit: 12 }
    });
    const host = await renderApp();
    await readyEngine(host);
    await startSelectedNode(host);
    await completeSelectedNode("job-1", { indices: [] }, analysisFrame({
      candidates: Array.from({ length: 12 }, (_, index) => candidate(index))
    }));
    expect(candidateCoords(host)).toHaveLength(12);
  });

  it("shows provided score and candidates while keeping missing ownership and policy absent", async () => {
    const host = await renderApp();
    await readyEngine(host);
    await startSelectedNode(host);
    await completeSelectedNode("job-1", { indices: [] }, analysisFrame({
      ownership: undefined,
      policy: undefined
    }));
    expect(candidateCoords(host)).toEqual(["D6"]);
    expect(host.textContent).toContain("胜率 61.0%");
    expect(host.textContent).toContain("+2.7");
    expect(host.textContent).not.toContain("领地已评估");
    expect(buttonNamed(host, "领地").disabled).toBe(true);
    expect(buttonNamed(host, "策略").disabled).toBe(true);
  });

  it("keeps no-engine review empty without manufacturing analysis overlays", async () => {
    const host = await renderApp();
    expect(buttonNamed(host, "分析当前节点").disabled).toBe(true);
    expect(buttonNamed(host, "分析第一子主线").disabled).toBe(true);
    expect(candidateCoords(host)).toEqual([]);
    expect(buttonNamed(host, "领地").disabled).toBe(true);
    expect(buttonNamed(host, "策略").disabled).toBe(true);
    expect(host.textContent).not.toContain("领地已评估");
  });

  it("stores a selected-node result by NodePath and restores it after navigation without restoring hover", async () => {
    const host = await renderApp();
    await readyEngine(host);
    await startSelectedNode(host);
    await completeSelectedNode("job-1", { indices: [] }, analysisFrame({
      candidates: [
        candidateAt(2, 3, [{ point: { x: 2, y: 3 } }, { point: { x: 3, y: 3 } }]),
        candidateAt(6, 5, [{ point: { x: 6, y: 5 } }])
      ]
    }));
    const miniBoard = host.querySelector('canvas[aria-label="参考图变化副棋盘"]') as HTMLCanvasElement;
    const restingDrawn = miniBoard.dataset.drawn;
    await hoverBoard(host);
    expect(miniBoard.dataset.drawn).not.toBe(restingDrawn);

    await selectPath(host, "下一变化");
    expect(candidateCoords(host)).toEqual([]);
    expect(host.querySelector(".cand-row.is-selected")).toBeNull();

    await selectPath(host, "父节点");
    expect(candidateCoords(host)).toEqual(["C6", "G4"]);
    expect(miniBoard.dataset.drawn).toBe(restingDrawn);
    expect(host.querySelectorAll(".cand-row.is-selected")).toHaveLength(1);
    expect(host.querySelectorAll(".cand-row")[0]?.classList.contains("is-selected")).toBe(true);
  });

  it("keeps a selected-node result when it completes after the user has left the node", async () => {
    const host = await renderApp();
    await readyEngine(host);
    await startSelectedNode(host);
    await selectPath(host, "下一变化");
    expect((host.querySelector('input[aria-label="跳转手数"]') as HTMLInputElement).value).toBe("1");
    await completeSelectedNode("job-1", { indices: [] }, analysisFrame());
    expect(candidateCoords(host)).toEqual([]);

    await selectPath(host, "父节点");
    expect(candidateCoords(host)).toEqual(["D6"]);
    expect(host.textContent).toContain("胜率 61.0%");
  });

  it("ignores a stale selected-node completion after job supersession", async () => {
    const host = await renderApp();
    await readyEngine(host);
    await startSelectedNode(host, "job-old");
    await startSelectedNode(host, "job-new");
    await completeSelectedNode("job-old", { indices: [] }, analysisFrame({
      candidates: [candidate(0)]
    }));
    expect(candidateCoords(host)).toEqual([]);
    await completeSelectedNode("job-new", { indices: [] }, analysisFrame());
    expect(candidateCoords(host)).toEqual(["D6"]);
  });

  it("shows a whole-game node result by NodePath while selected-node is running and does not cancel whole-game", async () => {
    const host = await renderApp();
    await readyEngine(host);
    await startWholeGame(host);
    await emitWholeGameProgress({ indices: [] }, analysisFrame({
      job_id: "job-wg",
      candidates: [candidate(0)]
    }));
    await emitWholeGameProgress({ indices: [0] }, analysisFrame({
      job_id: "job-wg",
      turn: 1,
      winrate_black: 0.44,
      score_mean_black: -1.2,
      candidates: [candidate(1)]
    }), { completed: 2, expected: 2, remaining: 0 });
    expect(candidateCoords(host)).toEqual(["A9"]);

    await startSelectedNode(host, "job-sel");
    expect(candidateCoords(host)).toEqual([]);
    backend.cancelKataGoAnalysis.mockClear();
    await selectPath(host, "下一变化");
    expect(backend.selectCurrentGameNode).toHaveBeenLastCalledWith({ indices: [0] });
    expect(backend.cancelKataGoAnalysis).not.toHaveBeenCalled();
    expect(buttonNamed(host, "取消整局")).toBeTruthy();
    expect(candidateCoords(host)).toEqual(["B9"]);
    expect(host.textContent).toContain("44.0%");
  });

  it("keeps a selected-node session result after whole-game starts", async () => {
    const host = await renderApp();
    await readyEngine(host);
    await startSelectedNode(host);
    await completeSelectedNode("job-1", { indices: [] }, analysisFrame());
    expect(candidateCoords(host)).toEqual(["D6"]);

    await selectPath(host, "下一变化");
    await startSelectedNode(host, "job-child", { indices: [0] });
    await completeSelectedNode("job-child", { indices: [0] }, analysisFrame({
      job_id: "job-child",
      turn: 1,
      winrate_black: 0.44,
      score_mean_black: -1.2,
      candidates: [candidate(1)]
    }));
    expect(candidateCoords(host)).toEqual(["B9"]);

    await act(async () => {
      buttonNamed(host, "分析第一子主线").click();
      await backend.startKataGoGameAnalysis.mock.results.at(-1)?.value;
    });
    await selectPath(host, "父节点");
    expect(candidateCoords(host)).toEqual(["D6"]);
    expect(host.textContent).toContain("胜率 61.0%");
  });

  it("shows selected-node analysis from an opened Java SGF via the exact-node presentation path", async () => {
    const javaRoot: CurrentGameResultDto = {
      ...navigableRoot,
      snapshot: {
        ...navigableRoot.snapshot,
        primary_analysis: analysisFrame({
          candidates: [candidateAt(3, 3, [{ point: { x: 3, y: 3 } }, { point: { x: 2, y: 2 } }])]
        })
      }
    };
    currentGameFixture.mockResolvedValue(javaRoot);
    backend.selectCurrentGameNode.mockImplementation(async (path: NodePath) => {
      if (path.indices.length === 0) {
        return javaRoot;
      }
      return snapshotAt(path);
    });
    const host = await renderApp();
    expect(candidateCoords(host)).toEqual(["D6"]);
    expect(host.textContent).toContain("61.0%");

    await selectPath(host, "下一变化");
    expect(candidateCoords(host)).toEqual([]);

    await selectPath(host, "父节点");
    expect(candidateCoords(host)).toEqual(["D6"]);
  });

  it("clears presentation when the Foreground Engine Run leaves Ready", async () => {
    const host = await renderApp();
    await readyEngine(host);
    await startSelectedNode(host);
    await completeSelectedNode("job-1", { indices: [] }, analysisFrame());
    expect(candidateCoords(host)).toEqual(["D6"]);

    await act(async () => {
      listeners.onSnapshot?.({
        revision: 3,
        continuous: { enabled: true, phase: "waiting" },
        lifecycle: { state: "no_engine" }
      });
    });
    expect(candidateCoords(host)).toEqual([]);
    expect(buttonNamed(host, "领地").disabled).toBe(true);
    expect(buttonNamed(host, "策略").disabled).toBe(true);
  });
});

describe("attached SGF analysis as the active persistence path", () => {
  it("dirties on identity-valid whole-game progress, keeps Save enabled after a later node, and never writes SQLite", async () => {
    let dirty = false;
    backend.selectCurrentGameNode.mockImplementation(async (path: NodePath) => ({
      ...snapshotAt(path),
      dirty
    }));
    backend.saveCurrentGame.mockImplementation(async () => {
      dirty = false;
      return { ...snapshotAt({ indices: [] }), dirty: false, native_path: "/tmp/snapshot-a.sgf" };
    });

    const host = await renderApp();
    await readyEngine(host);
    await startWholeGame(host);
    expect(buttonNamed(host, "存档").disabled).toBe(true);

    dirty = true;
    await emitWholeGameProgress({ indices: [] }, analysisFrame({ job_id: "job-wg" }));
    expect(buttonNamed(host, "存档").disabled).toBe(false);
    expect(host.querySelector(".doc-name")?.textContent).toContain("*");

    await act(async () => {
      buttonNamed(host, "存档").click();
      await backend.saveCurrentGame.mock.results.at(-1)?.value;
    });
    expect(backend.saveCurrentGame).toHaveBeenCalledTimes(1);
    expect(buttonNamed(host, "存档").disabled).toBe(true);

    dirty = true;
    await emitWholeGameProgress({ indices: [0] }, analysisFrame({
      job_id: "job-wg",
      turn: 1,
      candidates: [candidate(1)]
    }), { completed: 2, expected: 2, remaining: 0 });
    expect(buttonNamed(host, "存档").disabled).toBe(false);
  });

  it("keeps dirty and in-memory analysis when Save fails", async () => {
    backend.selectCurrentGameNode.mockImplementation(async (path: NodePath) => ({
      ...snapshotAt(path),
      dirty: true
    }));
    backend.saveCurrentGame.mockRejectedValue(new Error("disk full"));

    const host = await renderApp();
    await readyEngine(host);
    await startWholeGame(host);
    await emitWholeGameProgress({ indices: [] }, analysisFrame({ job_id: "job-wg" }));
    expect(candidateCoords(host)).toEqual(["D6"]);
    expect(buttonNamed(host, "存档").disabled).toBe(false);

    await act(async () => {
      buttonNamed(host, "存档").click();
      await Promise.resolve();
    });
    expect(host.textContent).toContain("Save failed: disk full");
    expect(buttonNamed(host, "存档").disabled).toBe(false);
    expect(candidateCoords(host)).toEqual(["D6"]);
  });

  it("does not dirty or persist cancelled, failed, or non-projectable events", async () => {
    const host = await renderApp();
    await readyEngine(host);
    await startSelectedNode(host);
    const selectCalls = backend.selectCurrentGameNode.mock.calls.length;
    await act(async () => {
      listeners.onJob?.({
        run_id: "run-1",
        job_id: "job-1",
        lane: "selected_node",
        mode: "finite",
        generation: 1,
        node_path: { indices: [] },
        outcome: "cancelled"
      });
      listeners.onJob?.({
        run_id: "run-1",
        job_id: "job-1",
        lane: "selected_node",
        mode: "finite",
        generation: 1,
        node_path: { indices: [] },
        outcome: "failed",
        failure: { operation: "job", kind: "protocol", message: "stderr boom" }
      });
      listeners.onJob?.({
        run_id: "run-1",
        job_id: "job-1",
        lane: "selected_node",
        mode: "finite",
        generation: 1,
        node_path: { indices: [] },
        outcome: "completed",
        frame: analysisFrame({ visits: 0, candidates: [] })
      });
    });
    expect(backend.selectCurrentGameNode.mock.calls.length).toBe(selectCalls);
    expect(buttonNamed(host, "存档").disabled).toBe(true);
  });
});

