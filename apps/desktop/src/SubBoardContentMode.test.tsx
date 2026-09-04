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

const backend = vi.hoisted(() => ({
  getHealth: vi.fn(() => Promise.resolve({ status: "ok" })),
  replaceCurrentGame: vi.fn(),
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
  subscribeForegroundEngine: vi.fn()
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

let root: Root | null = null;

beforeEach(() => {
  globalThis.IS_REACT_ACT_ENVIRONMENT = true;
  vi.spyOn(HTMLCanvasElement.prototype, "getContext").mockImplementation(function (this: HTMLCanvasElement) {
    return canvasContext(this);
  });
  backend.replaceCurrentGame.mockResolvedValue(initialGame);
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
});

describe("persisted Sub-Board Variation and Raw modes", () => {
  it("starts in Variation and keeps that durable mode when no-engine review has no candidates", async () => {
    const host = await renderApp();

    expect(subBoardLabel(host)).toBe("参考图变化副棋盘");
    act(() => buttonNamed(host, "显示").click());
    expect(menuCheck(host, "变化图").getAttribute("aria-checked")).toBe("true");
    expect(menuCheck(host, "纯棋子").getAttribute("aria-checked")).toBe("false");
    expect(subBoardNumbers(host)).toEqual([]);
  });

  it("lets View and Preferences edit the same durable mode without collapsing the rail or dirtying SGF", async () => {
    const host = await renderApp();

    act(() => buttonNamed(host, "显示").click());
    act(() => menuCheck(host, "纯棋子").click());
    await flushLast(preferencesApi.saveAppPreferences);

    expect(preferencesApi.saveAppPreferences).toHaveBeenCalledWith({
      ...defaultAppPreferences,
      subBoardContentMode: "raw"
    });
    expect(subBoardLabel(host)).toBe("纯棋子副棋盘");
    expect(host.querySelector(".reference-panel")).not.toBeNull();
    expect(host.querySelector(".reference-panel")?.hasAttribute("hidden")).toBe(false);
    expect(host.querySelector(".doc-name")?.textContent?.endsWith(" *")).toBe(false);

    openPreferences(host);
    expect(labeledSelect(host, "小棋盘内容").value).toBe("raw");

    act(() => {
      const select = labeledSelect(host, "小棋盘内容");
      select.value = "variation";
      select.dispatchEvent(new Event("change", { bubbles: true }));
    });
    await flushLast(preferencesApi.saveAppPreferences);

    expect(labeledSelect(host, "小棋盘内容").value).toBe("variation");
    expect(subBoardLabel(host)).toBe("参考图变化副棋盘");
    expect(preferencesApi.saveAppPreferences).toHaveBeenLastCalledWith(defaultAppPreferences);
  });

  it("keeps the prior visible mode when persist fails", async () => {
    let rejectSave!: (error: Error) => void;
    preferencesApi.saveAppPreferences.mockImplementation(
      () => new Promise((_, reject) => {
        rejectSave = reject;
      })
    );
    const host = await renderApp();

    act(() => buttonNamed(host, "显示").click());
    act(() => menuCheck(host, "纯棋子").click());
    expect(subBoardLabel(host)).toBe("参考图变化副棋盘");

    await act(async () => {
      rejectSave(new Error("disk full"));
      await preferencesApi.saveAppPreferences.mock.results.at(-1)?.value.catch(() => undefined);
    });

    expect(subBoardLabel(host)).toBe("参考图变化副棋盘");
    act(() => buttonNamed(host, "显示").click());
    expect(menuCheck(host, "变化图").getAttribute("aria-checked")).toBe("true");
    expect(menuCheck(host, "纯棋子").getAttribute("aria-checked")).toBe("false");
    openPreferences(host);
    expect(labeledSelect(host, "小棋盘内容").value).toBe("variation");
    expect(host.querySelector(".preferences-header span")?.textContent).toBe("Save failed: disk full");
  });

  it("restores a successfully saved Raw mode on the next load", async () => {
    preferencesApi.loadAppPreferences.mockResolvedValue({
      preferences: { ...defaultAppPreferences, subBoardContentMode: "raw" }
    });
    const host = await renderApp();

    expect(subBoardLabel(host)).toBe("纯棋子副棋盘");
    act(() => buttonNamed(host, "显示").click());
    expect(menuCheck(host, "纯棋子").getAttribute("aria-checked")).toBe("true");
    openPreferences(host);
    expect(labeledSelect(host, "小棋盘内容").value).toBe("raw");
  });

  it("shows identity-valid PV numbers in Variation and suppresses them in Raw, including hover", async () => {
    const host = await renderApp();
    await readyEngine(host);
    await startSelectedNode(host);
    await completeSelectedNode("job-1", { indices: [] }, {
      job_id: "job-1",
      turn: 0,
      visits: 48,
      winrate_black: 0.61,
      score_mean_black: 2.7,
      candidates: [
        {
          vertex: { point: { x: 3, y: 3 } },
          visits: 40,
          winrate_black: 0.62,
          score_mean_black: 2.8,
          pv: [{ point: { x: 2, y: 2 } }]
        },
        {
          vertex: { point: { x: 6, y: 5 } },
          visits: 30,
          winrate_black: 0.49,
          score_mean_black: -0.5,
          pv: [{ point: { x: 5, y: 5 } }]
        }
      ]
    });

    expect(subBoardLabel(host)).toBe("参考图变化副棋盘");
    expect(subBoardNumbers(host)).toEqual(["1", "2"]);

    act(() => buttonNamed(host, "显示").click());
    act(() => menuCheck(host, "纯棋子").click());
    await flushLast(preferencesApi.saveAppPreferences);
    await hoverBoard(host);

    expect(subBoardLabel(host)).toBe("纯棋子副棋盘");
    expect(subBoardNumbers(host)).toEqual([]);
  });
});

async function renderApp(): Promise<HTMLElement> {
  const host = document.createElement("div");
  document.body.append(host);
  root = createRoot(host);
  act(() => root?.render(<App />));
  await act(async () => {
    await backend.replaceCurrentGame.mock.results.at(-1)?.value;
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
    generation: 1,
    node_path: { indices: [] }
  });
  await act(async () => {
    buttonNamed(host, "继续分析").click();
    await backend.startSelectedNodeAnalysis.mock.results.at(-1)?.value;
  });
}

async function completeSelectedNode(jobId: string, path: NodePath, frame: AnalysisFrameDto) {
  await act(async () => {
    listeners.onJob?.({
      run_id: "run-1",
      job_id: jobId,
      lane: "selected_node",
      generation: 1,
      node_path: path,
      outcome: "completed",
      frame
    });
    await backend.classifyProblems.mock.results.at(-1)?.value;
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

async function flushLast(mock: { mock: { results: Array<{ value?: unknown }> } }) {
  await act(async () => {
    await mock.mock.results.at(-1)?.value;
  });
}

function openPreferences(host: HTMLElement) {
  act(() => buttonNamed(host, "参数").click());
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

function subBoardLabel(host: HTMLElement): string {
  return subBoardCanvas(host).getAttribute("aria-label") ?? "";
}

function subBoardNumbers(host: HTMLElement): string[] {
  return [...(subBoardCanvas(host).dataset.drawn ?? "").matchAll(/\|(\d+)@/g)].map((match) => match[1]);
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
