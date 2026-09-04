// @vitest-environment jsdom

import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { CurrentGameResultDto, EngineFailureDto, ForegroundEngineSnapshotDto, GameDto, NodePath } from "./domain/types";

const listeners: {
  onSnapshot?: (snapshot: ForegroundEngineSnapshotDto) => void;
  onFailure?: (failure: EngineFailureDto) => void;
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

vi.mock("./api/backend", () => ({
  ...backend,
  isTauriRuntime: () => true,
  nativeCurrentGameUnavailable: "Native current-game commands require the Tauri desktop runtime."
}));

vi.mock("./api/preferences", () => ({
  loadAppPreferences: vi.fn(() => Promise.reject(new Error("preferences unavailable in test"))),
  saveAppPreferences: vi.fn()
}));

vi.mock("./components/PreferencesPanel", () => ({ PreferencesPanel: () => null }));
vi.mock("./components/ProviderPanel", () => ({ ProviderPanel: () => null }));
vi.mock("./components/WinrateChart", () => ({ WinrateChart: () => <canvas aria-label="胜率走势" /> }));
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
  tree: { properties: [], children: [] },
  selected_path: { indices: [] },
  snapshot: { path: { indices: [] }, position: emptyPosition, personal_comment: "" },
  generation: 1,
  dirty: false,
  native_path: null
};

const initialProjection: GameDto = {
  summary: { id: "game", board_size: 9, komi: 7.5, move_count: 0 },
  moves: []
};

const mainlineRoot: CurrentGameResultDto = {
  ...initialGame,
  tree: {
    properties: [],
    children: [{ properties: [{ key: "B", values: ["fe"] }], children: [] }]
  }
};

function snapshotAt(path: NodePath): CurrentGameResultDto {
  return {
    ...mainlineRoot,
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

const savedProfileB = {
  id: "profile-2",
  max_visits: 400,
  profile: {
    name: "Other KataGo",
    engine_path: "/bin/katago-b",
    model_path: "/models/b.bin",
    config_path: "/configs/b.cfg",
    working_dir: "/tmp",
    backend: "kata_go_analysis" as const
  }
};

const savedProfileC = {
  id: "profile-3",
  max_visits: 200,
  profile: {
    name: "Third KataGo",
    engine_path: "/bin/katago-c",
    model_path: "/models/c.bin",
    config_path: "/configs/c.cfg",
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

const crashFailure: EngineFailureDto = {
  operation: "unexpected_exit",
  run_id: "run-1",
  profile_id: "profile-1",
  kind: "nonzero_exit",
  message: "engine process exited unexpectedly"
};

function readyRun(runId: string, profile: typeof savedProfile) {
  return {
    run_id: runId,
    profile_id: profile.id,
    adapter_kind: "kata_go_analysis" as const,
    profile_snapshot: profile.profile,
    capability_snapshot: capability
  };
}

let root: Root | null = null;

beforeEach(() => {
  globalThis.IS_REACT_ACT_ENVIRONMENT = true;
  vi.spyOn(HTMLCanvasElement.prototype, "getContext").mockImplementation(function (this: HTMLCanvasElement) {
    return {
      canvas: this,
      fillRect: vi.fn(),
      clearRect: vi.fn(),
      getImageData: vi.fn(),
      putImageData: vi.fn(),
      createImageData: vi.fn(),
      setTransform: vi.fn(),
      drawImage: vi.fn(),
      save: vi.fn(),
      restore: vi.fn(),
      beginPath: vi.fn(),
      moveTo: vi.fn(),
      lineTo: vi.fn(),
      closePath: vi.fn(),
      stroke: vi.fn(),
      fill: vi.fn(),
      translate: vi.fn(),
      scale: vi.fn(),
      rotate: vi.fn(),
      arc: vi.fn(),
      fillText: vi.fn(),
      measureText: () => ({ width: 0 }),
      setLineDash: vi.fn(),
      getLineDash: () => [],
      clip: vi.fn()
    } as unknown as CanvasRenderingContext2D;
  });
  backend.replaceCurrentGame.mockResolvedValue(initialGame);
  backend.projectCurrentGameMainline.mockResolvedValue(initialProjection);
  backend.loadEngineProfilesSettings.mockResolvedValue({
    selected_profile_id: "profile-1",
    autoload_profile_id: null,
    profiles: [savedProfile, savedProfileB]
  });
  backend.saveEngineProfilesSettings.mockImplementation(async (settings) => settings);
  backend.getForegroundEngineSnapshot.mockResolvedValue({ revision: 0, lifecycle: { state: "no_engine" } });
  backend.startSelectedNodeAnalysis.mockResolvedValue({
    run_id: "run-1",
    job_id: "job-1",
    lane: "selected_node",
    generation: 1,
    node_path: { indices: [] }
  });
  backend.classifyProblems.mockResolvedValue([]);
  backend.subscribeForegroundEngine.mockImplementation(async (onSnapshot, onFailure, onJob) => {
    listeners.onSnapshot = onSnapshot;
    listeners.onFailure = onFailure;
    listeners.onJob = onJob;
    onSnapshot({ revision: 0, lifecycle: { state: "no_engine" } });
    return () => undefined;
  });
  backend.startKataGoGameAnalysis.mockResolvedValue({
    run_id: "run-1",
    job_id: "job-wg",
    lane: "whole_game",
    generation: 1,
    node_path: { indices: [] }
  });
  backend.cancelKataGoAnalysis.mockResolvedValue(undefined);
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
    await backend.replaceCurrentGame.mock.results[0]?.value;
    await backend.projectCurrentGameMainline.mock.results[0]?.value;
    await backend.subscribeForegroundEngine.mock.results[0]?.value;
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
          capability_snapshot: {
            adapter_kind: "kata_go_analysis",
            selected_node_analysis: true,
            whole_game_analysis: true,
            protocol_cancel: true
          }
        }
      }
    });
  });
}

function buttonNamed(host: HTMLElement, label: string) {
  return Array.from(host.querySelectorAll("button")).find((button) => button.textContent === label) as HTMLButtonElement;
}

async function openEngineSettings(host: HTMLElement) {
  act(() => {
    Array.from(host.querySelectorAll("button")).find((button) => button.textContent === "设置")?.click();
  });
  await act(async () => {
    await Promise.resolve();
  });
  return host.querySelector(".engine-setup-panel") as HTMLElement;
}

describe("foreground engine lifecycle UI", () => {
  it("shows authoritative no-engine status and does not start from Engine Settings selection", async () => {
    const host = await renderApp();
    expect(host.querySelector(".engine-chip-label")?.textContent).toBe("未加载引擎");
    const analyzeOnce = Array.from(host.querySelectorAll("button")).find((button) => button.textContent === "分析此手");
    expect(analyzeOnce).toBeUndefined();
    act(() => {
      Array.from(host.querySelectorAll("button")).find((button) => button.textContent === "设置")?.click();
    });
    const settingsSelect = host.querySelector('.engine-setup-panel select') as HTMLSelectElement | null;
    expect(settingsSelect).not.toBeNull();
    await act(async () => {
      settingsSelect!.value = "profile-1";
      settingsSelect!.dispatchEvent(new Event("change", { bubbles: true }));
    });
    expect(backend.startForegroundEngine).not.toHaveBeenCalled();
    expect(backend.saveEngineProfilesSettings).toHaveBeenCalled();
  });

  it("starts from the Engine Switcher and enables stop after Ready", async () => {
    const host = await renderApp();
    const switcher = host.querySelector('select[aria-label="Foreground Engine Profile"]') as HTMLSelectElement;
    await act(async () => {
      switcher.value = "profile-1";
      switcher.dispatchEvent(new Event("change", { bubbles: true }));
      await backend.startForegroundEngine.mock.results[0]?.value;
    });
    expect(backend.startForegroundEngine).toHaveBeenCalledWith("profile-1");
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
            capability_snapshot: {
              adapter_kind: "kata_go_analysis",
              selected_node_analysis: true,
              whole_game_analysis: true,
              protocol_cancel: true
            }
          }
        }
      });
    });
    expect(host.querySelector(".engine-chip-label")?.textContent).toBe("Local KataGo");
    const stop = Array.from(host.querySelectorAll("button")).find((button) => button.textContent === "停止") as HTMLButtonElement;
    const restart = Array.from(host.querySelectorAll("button")).find((button) => button.textContent === "重启") as HTMLButtonElement;
    expect(stop.disabled).toBe(false);
    expect(restart.disabled).toBe(false);
    await act(async () => {
      stop.click();
    });
    expect(backend.stopForegroundEngine).toHaveBeenCalledOnce();
  });

  it("does not restart when the current primary profile is selected again", async () => {
    const host = await renderApp();
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
            capability_snapshot: {
              adapter_kind: "kata_go_analysis",
              selected_node_analysis: true,
              whole_game_analysis: true,
              protocol_cancel: true
            }
          }
        }
      });
    });
    backend.startForegroundEngine.mockClear();
    backend.restartForegroundEngine.mockClear();
    await act(async () => {
      switcher.value = "profile-1";
      switcher.dispatchEvent(new Event("change", { bubbles: true }));
    });
    expect(backend.startForegroundEngine).not.toHaveBeenCalled();
    expect(backend.restartForegroundEngine).not.toHaveBeenCalled();
  });

  it("starts selected-node analysis with run, generation, and NodePath identities", async () => {
    const host = await renderApp();
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
            capability_snapshot: {
              adapter_kind: "kata_go_analysis",
              selected_node_analysis: true,
              whole_game_analysis: true,
              protocol_cancel: true
            }
          }
        }
      });
    });
    const analyzeOnce = Array.from(host.querySelectorAll("button")).find((button) => button.textContent === "继续分析") as HTMLButtonElement;
    expect(analyzeOnce.disabled).toBe(false);
    await act(async () => {
      analyzeOnce.click();
      await backend.startSelectedNodeAnalysis.mock.results[0]?.value;
    });
    expect(backend.startSelectedNodeAnalysis).toHaveBeenCalledWith({
      runId: "run-1",
      generation: 1,
      nodePath: { indices: [] },
      maxVisits: 800
    });
  });

  it("presents selected-node analysis after a personal comment advances the document generation", async () => {
    backend.setCurrentGamePersonalComment.mockResolvedValue({
      ...initialGame,
      snapshot: { ...initialGame.snapshot, personal_comment: "reviewer note" },
      generation: 2,
      dirty: true
    });
    backend.startSelectedNodeAnalysis.mockResolvedValue({
      run_id: "run-1",
      job_id: "job-comment",
      lane: "selected_node",
      generation: 2,
      node_path: { indices: [] }
    });
    const host = await renderApp();
    await readyEngine(host);
    const editor = host.querySelector('textarea[aria-label="个人评论"]') as HTMLTextAreaElement;

    act(() => {
      editor.focus();
      const valueSetter = Object.getOwnPropertyDescriptor(HTMLTextAreaElement.prototype, "value")?.set;
      valueSetter?.call(editor, "reviewer note");
      editor.dispatchEvent(new Event("input", { bubbles: true }));
    });
    await act(async () => {
      editor.blur();
      await vi.waitFor(() => expect(backend.projectCurrentGameMainline).toHaveBeenCalledTimes(2));
    });

    await act(async () => {
      buttonNamed(host, "继续分析").click();
      await backend.startSelectedNodeAnalysis.mock.results.at(-1)?.value;
    });
    expect(backend.startSelectedNodeAnalysis).toHaveBeenLastCalledWith({
      runId: "run-1",
      generation: 2,
      nodePath: { indices: [] },
      maxVisits: 800
    });

    await act(async () => {
      listeners.onJob?.({
        run_id: "run-1",
        job_id: "job-comment",
        lane: "selected_node",
        generation: 1,
        node_path: { indices: [] },
        outcome: "completed",
        frame: {
          job_id: "job-comment",
          turn: 0,
          visits: 16,
          winrate_black: 0.11,
          score_mean_black: -9,
          candidates: [{ vertex: { point: { x: 0, y: 0 } }, visits: 16, winrate_black: 0.11, score_mean_black: -9, pv: [] }]
        }
      });
    });
    expect(host.querySelector(".cand-row")).toBeNull();

    await act(async () => {
      listeners.onJob?.({
        run_id: "run-1",
        job_id: "job-comment",
        lane: "selected_node",
        generation: 2,
        node_path: { indices: [] },
        outcome: "completed",
        frame: {
          job_id: "job-comment",
          turn: 0,
          visits: 32,
          winrate_black: 0.71,
          score_mean_black: 4,
          candidates: [{ vertex: { point: { x: 2, y: 3 } }, visits: 32, winrate_black: 0.71, score_mean_black: 4, pv: [] }]
        }
      });
      await backend.classifyProblems.mock.results.at(-1)?.value;
    });
    expect(Array.from(host.querySelectorAll(".cand-coord")).map((node) => node.textContent)).toEqual(["C6"]);
  });

  it("does not revive analysis presentation from a stale job event", async () => {
    const host = await renderApp();
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
            capability_snapshot: {
              adapter_kind: "kata_go_analysis",
              selected_node_analysis: true,
              whole_game_analysis: true,
              protocol_cancel: true
            }
          }
        }
      });
    });
    backend.startSelectedNodeAnalysis
      .mockResolvedValueOnce({
        run_id: "run-1",
        job_id: "job-old",
        lane: "selected_node",
        generation: 1,
        node_path: { indices: [] }
      })
      .mockResolvedValueOnce({
        run_id: "run-1",
        job_id: "job-new",
        lane: "selected_node",
        generation: 1,
        node_path: { indices: [] }
      });
    const analyzeOnce = Array.from(host.querySelectorAll("button")).find((button) => button.textContent === "继续分析") as HTMLButtonElement;
    expect(analyzeOnce.disabled).toBe(false);
    await act(async () => {
      analyzeOnce.click();
      await backend.startSelectedNodeAnalysis.mock.results[0]?.value;
    });
    await act(async () => {
      analyzeOnce.click();
      await backend.startSelectedNodeAnalysis.mock.results[1]?.value;
    });
    await act(async () => {
      listeners.onJob?.({
        run_id: "run-1",
        job_id: "job-old",
        lane: "selected_node",
        generation: 1,
        node_path: { indices: [] },
        outcome: "completed",
        frame: {
          job_id: "job-old",
          turn: 0,
          visits: 64,
          winrate_black: 0.11,
          score_mean_black: -9,
          candidates: [{ vertex: { point: { x: 0, y: 0 } }, visits: 64, winrate_black: 0.11, score_mean_black: -9, pv: [] }]
        }
      });
    });
    expect(host.querySelector(".cand-row")).toBeNull();
    await act(async () => {
      listeners.onJob?.({
        run_id: "run-1",
        job_id: "job-new",
        lane: "selected_node",
        generation: 1,
        node_path: { indices: [] },
        outcome: "completed",
        frame: {
          job_id: "job-new",
          turn: 0,
          visits: 32,
          winrate_black: 0.71,
          score_mean_black: 4,
          candidates: [{ vertex: { point: { x: 2, y: 3 } }, visits: 32, winrate_black: 0.71, score_mean_black: 4, pv: [] }]
        }
      });
      await backend.classifyProblems.mock.results.at(-1)?.value;
    });
    const coords = Array.from(host.querySelectorAll(".cand-coord")).map((node) => node.textContent);
    expect(coords).toEqual(["C6"]);
  });

  it("shows selected-node timeout while the Ready Run still owns the job", async () => {
    const host = await renderApp();
    await readyEngine(host);
    await act(async () => {
      buttonNamed(host, "继续分析").click();
      await backend.startSelectedNodeAnalysis.mock.results[0]?.value;
    });
    await act(async () => {
      listeners.onJob?.({
        run_id: "run-1",
        job_id: "job-1",
        lane: "selected_node",
        generation: 1,
        node_path: { indices: [] },
        outcome: "timeout"
      });
    });
    expect(host.textContent).toContain("Selected-node analysis timed out.");
  });

  it("does not show selected-node timeout after Stop or Switch leaves the job's Run", async () => {
    const host = await renderApp();
    await readyEngine(host);
    await act(async () => {
      buttonNamed(host, "继续分析").click();
      await backend.startSelectedNodeAnalysis.mock.results[0]?.value;
    });
    await act(async () => {
      buttonNamed(host, "停止").click();
      await backend.stopForegroundEngine.mock.results[0]?.value;
    });
    await act(async () => {
      listeners.onSnapshot?.({
        revision: 3,
        lifecycle: { state: "no_engine" }
      });
    });
    await act(async () => {
      listeners.onJob?.({
        run_id: "run-1",
        job_id: "job-1",
        lane: "selected_node",
        generation: 1,
        node_path: { indices: [] },
        outcome: "timeout"
      });
    });
    expect(host.textContent).not.toContain("Selected-node analysis timed out.");

    await readyEngine(host);
    backend.startSelectedNodeAnalysis.mockResolvedValueOnce({
      run_id: "run-1",
      job_id: "job-2",
      lane: "selected_node",
      generation: 1,
      node_path: { indices: [] }
    });
    await act(async () => {
      buttonNamed(host, "继续分析").click();
      await backend.startSelectedNodeAnalysis.mock.results.at(-1)?.value;
    });
    await act(async () => {
      listeners.onSnapshot?.({
        revision: 4,
        lifecycle: { state: "ready", run: readyRun("run-b", savedProfileB) }
      });
    });
    await act(async () => {
      listeners.onJob?.({
        run_id: "run-1",
        job_id: "job-2",
        lane: "selected_node",
        generation: 1,
        node_path: { indices: [] },
        outcome: "timeout"
      });
    });
    expect(host.textContent).not.toContain("Selected-node analysis timed out.");
  });

  it("starts whole-game analysis with the Ready Run identity and keeps cancel from stopping the engine", async () => {
    const host = await renderApp();
    await readyEngine(host);
    await act(async () => {
      buttonNamed(host, "自动分析").click();
      await backend.startKataGoGameAnalysis.mock.results.at(-1)?.value;
    });
    expect(backend.startKataGoGameAnalysis).toHaveBeenCalledWith({
      runId: "run-1",
      generation: 1,
      maxVisits: 800
    });
    await act(async () => {
      listeners.onJob?.({
        run_id: "run-1",
        job_id: "job-wg",
        lane: "whole_game",
        generation: 1,
        node_path: { indices: [] },
        outcome: "progress",
        completed: 1,
        expected: 2,
        remaining: 1,
        frame: {
          job_id: "job-wg",
          turn: 0,
          visits: 8,
          winrate_black: 0.61,
          score_mean_black: 1.5,
          candidates: [{ vertex: { point: { x: 0, y: 0 } }, visits: 8, winrate_black: 0.61, score_mean_black: 1.5, pv: [] }]
        }
      });
      await backend.selectCurrentGameNode.mock.results.at(-1)?.value;
    });
    expect(host.querySelector(".nav-progress")?.textContent).toContain("整局 1/2");
    expect(host.querySelector(".nav-progress")?.textContent).toContain("剩余 1");
    expect(host.textContent).toContain("61.0%");
    await act(async () => {
      buttonNamed(host, "取消整局").click();
      await backend.cancelKataGoAnalysis.mock.results.at(-1)?.value;
    });
    expect(backend.cancelKataGoAnalysis).toHaveBeenCalledWith("run-1", "job-wg");
    expect(backend.stopForegroundEngine).not.toHaveBeenCalled();
    await act(async () => {
      listeners.onJob?.({
        run_id: "run-1",
        job_id: "job-wg",
        lane: "whole_game",
        generation: 1,
        node_path: { indices: [] },
        outcome: "cancelled",
        failure: { operation: "job", kind: "cancellation", message: "analysis job was cancelled" }
      });
    });
    expect(host.querySelector(".engine-chip-label")?.textContent).toBe("Local KataGo");
    expect(host.textContent).toContain("61.0%");
  });

  it("keeps review navigation live during whole-game analysis and restores completed node results", async () => {
    backend.replaceCurrentGame.mockResolvedValue(mainlineRoot);
    backend.selectCurrentGameNode.mockImplementation(async (path: NodePath) => snapshotAt(path));
    const host = await renderApp();
    await readyEngine(host);
    const nextMove = host.querySelector('button[title="下一手"]') as HTMLButtonElement;
    const prevMove = host.querySelector('button[title="上一手"]') as HTMLButtonElement;
    expect(nextMove.disabled).toBe(false);

    await act(async () => {
      buttonNamed(host, "自动分析").click();
      await backend.startKataGoGameAnalysis.mock.results.at(-1)?.value;
    });
    await act(async () => {
      listeners.onJob?.({
        run_id: "run-1",
        job_id: "job-wg",
        lane: "whole_game",
        generation: 1,
        node_path: { indices: [] },
        outcome: "progress",
        completed: 1,
        expected: 2,
        remaining: 1,
        frame: {
          job_id: "job-wg",
          turn: 0,
          visits: 8,
          winrate_black: 0.61,
          score_mean_black: 1.5,
          candidates: [{ vertex: { point: { x: 0, y: 0 } }, visits: 8, winrate_black: 0.61, score_mean_black: 1.5, pv: [] }]
        }
      });
      await backend.selectCurrentGameNode.mock.results.at(-1)?.value;
    });
    expect(host.textContent).toContain("61.0%");
    expect(nextMove.disabled).toBe(false);

    backend.cancelKataGoAnalysis.mockClear();
    await act(async () => {
      nextMove.click();
      await backend.selectCurrentGameNode.mock.results.at(-1)?.value;
    });
    expect(backend.selectCurrentGameNode).toHaveBeenLastCalledWith({ indices: [0] });
    expect(backend.cancelKataGoAnalysis).not.toHaveBeenCalled();
    expect(buttonNamed(host, "取消整局")).toBeTruthy();
    expect(host.querySelector(".nav-progress")?.textContent).toContain("整局 1/2");
    expect(host.querySelector(".nav-progress")?.textContent).toContain("剩余 1");
    expect((host.querySelector('input[aria-label="跳转手数"]') as HTMLInputElement).value).toBe("1");
    expect(host.textContent).not.toContain("61.0%");

    await act(async () => {
      prevMove.click();
      await backend.selectCurrentGameNode.mock.results.at(-1)?.value;
    });
    expect(backend.selectCurrentGameNode).toHaveBeenLastCalledWith({ indices: [] });
    expect(backend.cancelKataGoAnalysis).not.toHaveBeenCalled();
    expect((host.querySelector('input[aria-label="跳转手数"]') as HTMLInputElement).value).toBe("0");
    expect(host.textContent).toContain("61.0%");
    expect(host.querySelector(".nav-progress")?.textContent).toContain("整局 1/2");
  });

  it("does not restore a previous document's whole-game node result after Open", async () => {
    backend.replaceCurrentGame.mockResolvedValue(mainlineRoot);
    backend.selectCurrentGameNode.mockImplementation(async (path: NodePath) => snapshotAt(path));
    backend.openSgfDocument.mockResolvedValue({ sgfText: "(;SZ[9]B[fe])", path: "/tmp/other.sgf" });
    const host = await renderApp();
    await readyEngine(host);
    const nextMove = host.querySelector('button[title="下一手"]') as HTMLButtonElement;
    const prevMove = host.querySelector('button[title="上一手"]') as HTMLButtonElement;

    await act(async () => {
      buttonNamed(host, "自动分析").click();
      await backend.startKataGoGameAnalysis.mock.results.at(-1)?.value;
    });
    await act(async () => {
      listeners.onJob?.({
        run_id: "run-1",
        job_id: "job-wg",
        lane: "whole_game",
        generation: 1,
        node_path: { indices: [] },
        outcome: "progress",
        completed: 1,
        expected: 2,
        remaining: 1,
        frame: {
          job_id: "job-wg",
          turn: 0,
          visits: 8,
          winrate_black: 0.61,
          score_mean_black: 1.5,
          candidates: [{ vertex: { point: { x: 0, y: 0 } }, visits: 8, winrate_black: 0.61, score_mean_black: 1.5, pv: [] }]
        }
      });
      await backend.selectCurrentGameNode.mock.results.at(-1)?.value;
    });
    expect(host.textContent).toContain("61.0%");

    backend.replaceCurrentGame.mockResolvedValue({
      ...mainlineRoot,
      generation: 2,
      native_path: "/tmp/other.sgf"
    });
    await act(async () => {
      buttonNamed(host, "文件").click();
    });
    await act(async () => {
      buttonNamed(host, "打开棋谱(O)").click();
      await backend.openSgfDocument.mock.results.at(-1)?.value;
      await backend.replaceCurrentGame.mock.results.at(-1)?.value;
    });
    expect(host.textContent).not.toContain("61.0%");
    expect(backend.cancelKataGoAnalysis).toHaveBeenCalledWith("run-1", "job-wg");

    await act(async () => {
      nextMove.click();
      await backend.selectCurrentGameNode.mock.results.at(-1)?.value;
    });
    await act(async () => {
      prevMove.click();
      await backend.selectCurrentGameNode.mock.results.at(-1)?.value;
    });
    expect(host.textContent).not.toContain("61.0%");
    expect(host.textContent).not.toContain("取消整局");
  });

  it("drops a selected-node session when Open replaces the document", async () => {
    backend.replaceCurrentGame.mockResolvedValue(mainlineRoot);
    backend.selectCurrentGameNode.mockImplementation(async (path: NodePath) => snapshotAt(path));
    backend.openSgfDocument.mockResolvedValue({ sgfText: "(;SZ[9]B[fe])", path: "/tmp/other.sgf" });
    const host = await renderApp();
    await readyEngine(host);
    const nextMove = host.querySelector('button[title="下一手"]') as HTMLButtonElement;
    const prevMove = host.querySelector('button[title="上一手"]') as HTMLButtonElement;

    await act(async () => {
      buttonNamed(host, "继续分析").click();
      await backend.startSelectedNodeAnalysis.mock.results.at(-1)?.value;
    });

    backend.replaceCurrentGame.mockResolvedValue({
      ...mainlineRoot,
      generation: 2,
      native_path: "/tmp/other.sgf"
    });
    backend.selectCurrentGameNode.mockImplementation(async (path: NodePath) => ({
      ...snapshotAt(path),
      generation: 2,
      native_path: "/tmp/other.sgf"
    }));
    await act(async () => {
      buttonNamed(host, "文件").click();
    });
    await act(async () => {
      buttonNamed(host, "打开棋谱(O)").click();
      await backend.openSgfDocument.mock.results.at(-1)?.value;
      await backend.replaceCurrentGame.mock.results.at(-1)?.value;
    });
    expect.soft(host.textContent).not.toContain("取消此手");
    expect.soft(backend.cancelSelectedNodeAnalysis).toHaveBeenCalledWith({ runId: "run-1", jobId: "job-1" });

    await act(async () => {
      listeners.onJob?.({
        run_id: "run-1",
        job_id: "job-1",
        lane: "selected_node",
        generation: 1,
        node_path: { indices: [] },
        outcome: "completed",
        frame: {
          job_id: "job-1",
          turn: 0,
          visits: 8,
          winrate_black: 0.61,
          score_mean_black: 1.5,
          candidates: [{ vertex: { point: { x: 0, y: 0 } }, visits: 8, winrate_black: 0.61, score_mean_black: 1.5, pv: [] }]
        }
      });
      await backend.classifyProblems.mock.results.at(-1)?.value;
    });
    expect.soft(host.textContent).not.toContain("61.0%");

    await act(async () => {
      nextMove.click();
      await backend.selectCurrentGameNode.mock.results.at(-1)?.value;
    });
    await act(async () => {
      prevMove.click();
      await backend.selectCurrentGameNode.mock.results.at(-1)?.value;
    });
    expect(host.textContent).not.toContain("61.0%");
  });

  it("does not restore presentation or cache from a stale whole-game completion", async () => {
    const host = await renderApp();
    await readyEngine(host);
    await act(async () => {
      buttonNamed(host, "自动分析").click();
      await backend.startKataGoGameAnalysis.mock.results.at(-1)?.value;
    });
    await act(async () => {
      buttonNamed(host, "取消整局").click();
      await backend.cancelKataGoAnalysis.mock.results.at(-1)?.value;
      listeners.onJob?.({
        run_id: "run-1",
        job_id: "job-wg",
        lane: "whole_game",
        generation: 1,
        node_path: { indices: [] },
        outcome: "cancelled",
        failure: { operation: "job", kind: "cancellation", message: "analysis job was cancelled" }
      });
    });
    await act(async () => {
      listeners.onJob?.({
        run_id: "run-1",
        job_id: "job-wg",
        lane: "whole_game",
        generation: 1,
        node_path: { indices: [] },
        outcome: "completed",
        completed: 2,
        expected: 2
      });
      listeners.onJob?.({
        run_id: "run-old",
        job_id: "job-wg",
        lane: "whole_game",
        generation: 1,
        node_path: { indices: [] },
        outcome: "completed",
        completed: 2,
        expected: 2
      });
      listeners.onJob?.({
        run_id: "run-1",
        job_id: "job-other",
        lane: "whole_game",
        generation: 1,
        node_path: { indices: [] },
        outcome: "completed",
        completed: 2,
        expected: 2
      });
      listeners.onJob?.({
        run_id: "run-1",
        job_id: "job-wg",
        lane: "whole_game",
        generation: 99,
        node_path: { indices: [] },
        outcome: "completed",
        completed: 2,
        expected: 2
      });
      listeners.onJob?.({
        run_id: "run-1",
        job_id: "job-wg",
        lane: "whole_game",
        generation: 1,
        node_path: { indices: [0] },
        outcome: "completed",
        completed: 2,
        expected: 2
      });
    });
    expect(host.textContent).not.toContain("整局分析完成");
  });

  it("runs selected-node and whole-game lanes concurrently with independent cancel", async () => {
    const host = await renderApp();
    await readyEngine(host);
    await act(async () => {
      buttonNamed(host, "继续分析").click();
      buttonNamed(host, "自动分析").click();
      await backend.startSelectedNodeAnalysis.mock.results.at(-1)?.value;
      await backend.startKataGoGameAnalysis.mock.results.at(-1)?.value;
    });
    expect(backend.startSelectedNodeAnalysis).toHaveBeenCalled();
    expect(backend.startKataGoGameAnalysis).toHaveBeenCalled();
    backend.cancelKataGoAnalysis.mockClear();
    backend.cancelSelectedNodeAnalysis.mockClear();
    await act(async () => {
      buttonNamed(host, "取消此手").click();
      await backend.cancelSelectedNodeAnalysis.mock.results.at(-1)?.value;
    });
    expect(backend.cancelSelectedNodeAnalysis).toHaveBeenCalledWith({ runId: "run-1", jobId: "job-1" });
    expect(backend.cancelKataGoAnalysis).not.toHaveBeenCalled();
    await act(async () => {
      listeners.onJob?.({
        run_id: "run-1",
        job_id: "job-1",
        lane: "selected_node",
        generation: 1,
        node_path: { indices: [] },
        outcome: "cancelled"
      });
      listeners.onJob?.({
        run_id: "run-1",
        job_id: "job-wg",
        lane: "whole_game",
        generation: 1,
        node_path: { indices: [] },
        outcome: "progress",
        completed: 1,
        expected: 2
      });
    });
    expect(host.querySelector(".nav-progress")?.textContent).toContain("整局 1/2");
    expect(buttonNamed(host, "取消整局")).toBeTruthy();
    backend.cancelSelectedNodeAnalysis.mockClear();
    await act(async () => {
      buttonNamed(host, "取消整局").click();
      await backend.cancelKataGoAnalysis.mock.results.at(-1)?.value;
    });
    expect(backend.cancelKataGoAnalysis).toHaveBeenCalledWith("run-1", "job-wg");
    expect(backend.cancelSelectedNodeAnalysis).not.toHaveBeenCalled();
  });

  it("rejects a second whole-game start as occupied without clearing the first job or blocking selected-node", async () => {
    const host = await renderApp();
    await readyEngine(host);
    await act(async () => {
      buttonNamed(host, "自动分析").click();
      await backend.startKataGoGameAnalysis.mock.results.at(-1)?.value;
    });
    backend.startKataGoGameAnalysis.mockRejectedValueOnce({
      operation: "job",
      kind: "occupied",
      message: "whole-game analysis is already running on this Foreground Engine Run"
    });
    await act(async () => {
      buttonNamed(host, "自动分析").click();
      await backend.startKataGoGameAnalysis.mock.results.at(-1)?.value.catch(() => undefined);
    });
    expect(host.textContent).toContain("whole-game analysis is already running on this Foreground Engine Run");
    await act(async () => {
      buttonNamed(host, "继续分析").click();
      await backend.startSelectedNodeAnalysis.mock.results.at(-1)?.value;
    });
    expect(backend.startSelectedNodeAnalysis).toHaveBeenCalled();
    expect(buttonNamed(host, "取消此手")).toBeTruthy();
    backend.cancelKataGoAnalysis.mockClear();
    await act(async () => {
      buttonNamed(host, "取消整局").click();
      await backend.cancelKataGoAnalysis.mock.results.at(-1)?.value;
    });
    expect(backend.cancelKataGoAnalysis).toHaveBeenCalledWith("run-1", "job-wg");
  });

  it("does not cancel whole-game when selected-node is superseded", async () => {
    const host = await renderApp();
    await readyEngine(host);
    await act(async () => {
      buttonNamed(host, "自动分析").click();
      await backend.startKataGoGameAnalysis.mock.results.at(-1)?.value;
    });
    backend.cancelKataGoAnalysis.mockClear();
    backend.startSelectedNodeAnalysis
      .mockResolvedValueOnce({
        run_id: "run-1",
        job_id: "job-old",
        lane: "selected_node",
        generation: 1,
        node_path: { indices: [] }
      })
      .mockResolvedValueOnce({
        run_id: "run-1",
        job_id: "job-new",
        lane: "selected_node",
        generation: 1,
        node_path: { indices: [] }
      });
    const analyzeOnce = buttonNamed(host, "继续分析");
    await act(async () => {
      analyzeOnce.click();
      await backend.startSelectedNodeAnalysis.mock.results.at(-2)?.value;
    });
    await act(async () => {
      analyzeOnce.click();
      await backend.startSelectedNodeAnalysis.mock.results.at(-1)?.value;
    });
    expect(backend.cancelKataGoAnalysis).not.toHaveBeenCalled();
    expect(buttonNamed(host, "取消整局")).toBeTruthy();
  });

  it("keeps Save and Save As available while both analysis lanes are running", async () => {
    backend.replaceCurrentGame.mockResolvedValue({ ...initialGame, dirty: true });
    const host = await renderApp();
    await readyEngine(host);
    await act(async () => {
      buttonNamed(host, "继续分析").click();
      buttonNamed(host, "自动分析").click();
      await backend.startSelectedNodeAnalysis.mock.results.at(-1)?.value;
      await backend.startKataGoGameAnalysis.mock.results.at(-1)?.value;
    });
    expect(buttonNamed(host, "存档").disabled).toBe(false);
    act(() => {
      buttonNamed(host, "文件").click();
    });
    expect(buttonNamed(host, "另存为(S)").disabled).toBe(false);
  });

  it("ignores progress and complete events that do not match the live lane identity", async () => {
    const host = await renderApp();
    await readyEngine(host);
    await act(async () => {
      buttonNamed(host, "继续分析").click();
      buttonNamed(host, "自动分析").click();
      await backend.startSelectedNodeAnalysis.mock.results.at(-1)?.value;
      await backend.startKataGoGameAnalysis.mock.results.at(-1)?.value;
    });
    await act(async () => {
      listeners.onJob?.({
        run_id: "run-1",
        job_id: "job-wg",
        lane: "whole_game",
        generation: 1,
        node_path: { indices: [] },
        outcome: "progress",
        completed: 1,
        expected: 2
      });
    });
    expect(host.querySelector(".nav-progress")?.textContent).toContain("整局 1/2");
    await act(async () => {
      listeners.onJob?.({
        run_id: "run-1",
        job_id: "job-1",
        lane: "whole_game",
        generation: 1,
        node_path: { indices: [] },
        outcome: "progress",
        completed: 9,
        expected: 9
      });
      listeners.onJob?.({
        run_id: "run-1",
        job_id: "job-wg",
        lane: "whole_game",
        generation: 99,
        node_path: { indices: [] },
        outcome: "progress",
        completed: 9,
        expected: 9
      });
      listeners.onJob?.({
        run_id: "run-1",
        job_id: "job-wg",
        lane: "whole_game",
        generation: 99,
        node_path: { indices: [] },
        outcome: "completed",
        completed: 9,
        expected: 9
      });
      listeners.onJob?.({
        run_id: "run-1",
        job_id: "job-1",
        lane: "selected_node",
        generation: 99,
        node_path: { indices: [] },
        outcome: "completed",
        frame: {
          job_id: "job-1",
          turn: 0,
          visits: 8,
          winrate_black: 0.9,
          score_mean_black: 4,
          candidates: []
        }
      });
    });
    expect(host.querySelector(".nav-progress")?.textContent).toContain("整局 1/2");
    expect(host.textContent).not.toContain("整局分析完成");
    expect(buttonNamed(host, "取消此手")).toBeTruthy();
    expect(buttonNamed(host, "取消整局")).toBeTruthy();
  });

  it("keeps A analysis available while Switching and does not bind jobs to B", async () => {
    const host = await renderApp();
    await readyEngine(host);
    const switcher = host.querySelector('select[aria-label="Foreground Engine Profile"]') as HTMLSelectElement;
    backend.startForegroundEngine.mockClear();
    backend.switchForegroundEngine.mockClear();
    await act(async () => {
      switcher.value = "profile-2";
      switcher.dispatchEvent(new Event("change", { bubbles: true }));
      await backend.switchForegroundEngine.mock.results[0]?.value;
    });
    expect(backend.switchForegroundEngine).toHaveBeenCalledWith("profile-2");
    expect(backend.startForegroundEngine).not.toHaveBeenCalled();
    await act(async () => {
      listeners.onSnapshot?.({
        revision: 3,
        lifecycle: {
          state: "switching",
          primary: readyRun("run-1", savedProfile),
          candidate: {
            ...readyRun("run-b", savedProfileB),
            capability_snapshot: null
          },
          switch_id: "7"
        }
      });
    });
    expect(host.querySelector(".engine-chip-label")?.textContent).toBe("正在切换 Other KataGo");
    expect(switcher.value).toBe("profile-1");
    const analyzeOnce = buttonNamed(host, "继续分析");
    expect(analyzeOnce.disabled).toBe(false);
    await act(async () => {
      analyzeOnce.click();
      await backend.startSelectedNodeAnalysis.mock.results.at(-1)?.value;
    });
    expect(backend.startSelectedNodeAnalysis).toHaveBeenCalledWith({
      runId: "run-1",
      generation: 1,
      nodePath: { indices: [] },
      maxVisits: 800
    });
  });

  it("rebinds operations to B after promotion without changing Settings selection or unsaved form", async () => {
    const host = await renderApp();
    await readyEngine(host);
    await act(async () => {
      Array.from(host.querySelectorAll("button")).find((button) => button.textContent === "设置")?.click();
      await backend.loadEngineProfilesSettings.mock.results.at(-1)?.value;
    });
    const settingsSelect = host.querySelector(".engine-setup-panel select") as HTMLSelectElement;
    const nameInput = host.querySelector('.engine-setup-panel input[placeholder="本地 KataGo"]') as HTMLInputElement;
    expect(settingsSelect.value).toBe("profile-1");
    await act(async () => {
      const setter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")?.set;
      setter?.call(nameInput, "Unsaved draft");
      nameInput.dispatchEvent(new Event("input", { bubbles: true }));
    });
    backend.saveEngineProfilesSettings.mockClear();
    backend.startForegroundEngine.mockClear();
    const switcher = host.querySelector('select[aria-label="Foreground Engine Profile"]') as HTMLSelectElement;
    await act(async () => {
      switcher.value = "profile-2";
      switcher.dispatchEvent(new Event("change", { bubbles: true }));
      await backend.switchForegroundEngine.mock.results.at(-1)?.value;
    });
    await act(async () => {
      listeners.onSnapshot?.({
        revision: 4,
        lifecycle: {
          state: "ready",
          run: readyRun("run-b", savedProfileB)
        }
      });
    });
    expect(backend.switchForegroundEngine).toHaveBeenCalledWith("profile-2");
    expect(backend.startForegroundEngine).not.toHaveBeenCalled();
    expect(backend.saveEngineProfilesSettings).not.toHaveBeenCalled();
    expect(settingsSelect.value).toBe("profile-1");
    expect(nameInput.value).toBe("Unsaved draft");
    expect(host.querySelector(".engine-chip-label")?.textContent).toBe("Other KataGo");
    expect(switcher.value).toBe("profile-2");
    backend.startSelectedNodeAnalysis.mockClear();
    await act(async () => {
      buttonNamed(host, "继续分析").click();
      await backend.startSelectedNodeAnalysis.mock.results.at(-1)?.value;
    });
    expect(backend.startSelectedNodeAnalysis).toHaveBeenCalledWith({
      runId: "run-b",
      generation: 1,
      nodePath: { indices: [] },
      maxVisits: 400
    });
    expect(buttonNamed(host, "停止").disabled).toBe(false);

    const activeProfileId = "profile-2";
    backend.saveEngineProfilesSettings.mockImplementation(async (settings) => {
      const removed = [savedProfile.id, savedProfileB.id].filter(
        (id) => !settings.profiles.some((profile: { id: string }) => profile.id === id)
      );
      if (removed.includes(activeProfileId)) {
        throw new Error("an active Foreground Engine Run still holds this profile identity");
      }
      return settings;
    });
    const deleteProfile = () => Array.from(host.querySelectorAll(".engine-setup-panel button")).find((button) => button.textContent === "删除") as HTMLButtonElement;
    await act(async () => {
      settingsSelect.value = "profile-2";
      settingsSelect.dispatchEvent(new Event("change", { bubbles: true }));
      await backend.saveEngineProfilesSettings.mock.results.at(-1)?.value;
    });
    await act(async () => {
      deleteProfile().click();
      await backend.saveEngineProfilesSettings.mock.results.at(-1)?.value.catch(() => undefined);
    });
    expect(host.textContent).toContain("Delete failed");
    expect(host.textContent).toContain("an active Foreground Engine Run still holds this profile identity");
    expect(Array.from(settingsSelect.options).map((option) => option.value)).toEqual(["profile-1", "profile-2"]);
    await act(async () => {
      settingsSelect.value = "profile-1";
      settingsSelect.dispatchEvent(new Event("change", { bubbles: true }));
      await backend.saveEngineProfilesSettings.mock.results.at(-1)?.value;
    });
    await act(async () => {
      deleteProfile().click();
      await backend.saveEngineProfilesSettings.mock.results.at(-1)?.value;
    });
    expect(Array.from(settingsSelect.options).map((option) => option.value)).toEqual(["profile-2"]);
  });

  it("keeps A available after a failed switch and shows B's typed failure", async () => {
    const host = await renderApp();
    await readyEngine(host);
    const switcher = host.querySelector('select[aria-label="Foreground Engine Profile"]') as HTMLSelectElement;
    await act(async () => {
      switcher.value = "profile-2";
      switcher.dispatchEvent(new Event("change", { bubbles: true }));
      await backend.switchForegroundEngine.mock.results.at(-1)?.value;
    });
    await act(async () => {
      listeners.onSnapshot?.({
        revision: 3,
        lifecycle: {
          state: "switching",
          primary: readyRun("run-1", savedProfile),
          candidate: {
            ...readyRun("run-b", savedProfileB),
            capability_snapshot: null
          },
          switch_id: "7"
        }
      });
    });
    await act(async () => {
      listeners.onSnapshot?.({
        revision: 4,
        lifecycle: { state: "ready", run: readyRun("run-1", savedProfile) }
      });
      listeners.onFailure?.({
        operation: "switch",
        run_id: "run-b",
        switch_id: "7",
        profile_id: "profile-2",
        kind: "asset",
        message: "required engine assets are missing"
      });
    });
    expect(host.querySelector(".engine-chip-label")?.textContent).toBe("Local KataGo");
    expect(switcher.value).toBe("profile-1");
    const failure = host.querySelector(".engine-failure") as HTMLElement;
    expect(failure.dataset.failureKind).toBe("asset");
    expect(failure.textContent).toContain("required engine assets are missing");
    backend.startSelectedNodeAnalysis.mockClear();
    await act(async () => {
      buttonNamed(host, "继续分析").click();
      await backend.startSelectedNodeAnalysis.mock.results.at(-1)?.value;
    });
    expect(backend.startSelectedNodeAnalysis).toHaveBeenCalledWith({
      runId: "run-1",
      generation: 1,
      nodePath: { indices: [] },
      maxVisits: 800
    });
    await act(async () => {
      listeners.onJob?.({
        run_id: "run-b",
        job_id: "job-b",
        lane: "selected_node",
        generation: 1,
        node_path: { indices: [] },
        outcome: "completed",
        frame: {
          job_id: "job-b",
          turn: 0,
          visits: 8,
          winrate_black: 0.9,
          score_mean_black: 4,
          candidates: [{ vertex: { point: { x: 1, y: 1 } }, visits: 8, winrate_black: 0.9, score_mean_black: 4, pv: [] }]
        }
      });
    });
    expect(host.querySelector(".cand-row")).toBeNull();
    expect(host.querySelector(".engine-chip-label")?.textContent).toBe("Local KataGo");
    expect((host.querySelector(".engine-failure") as HTMLElement).dataset.failureKind).toBe("asset");
  });

  it("lets a later switch win and ignores a superseded B failure", async () => {
    backend.loadEngineProfilesSettings.mockResolvedValue({
      selected_profile_id: "profile-1",
      autoload_profile_id: null,
      profiles: [savedProfile, savedProfileB, savedProfileC]
    });
    const host = await renderApp();
    await readyEngine(host);
    const switcher = host.querySelector('select[aria-label="Foreground Engine Profile"]') as HTMLSelectElement;
    await act(async () => {
      switcher.value = "profile-2";
      switcher.dispatchEvent(new Event("change", { bubbles: true }));
      await backend.switchForegroundEngine.mock.results.at(-1)?.value;
    });
    await act(async () => {
      listeners.onSnapshot?.({
        revision: 3,
        lifecycle: {
          state: "switching",
          primary: readyRun("run-1", savedProfile),
          candidate: {
            ...readyRun("run-b", savedProfileB),
            capability_snapshot: null
          },
          switch_id: "7"
        }
      });
    });
    backend.switchForegroundEngine.mockClear();
    await act(async () => {
      switcher.value = "profile-3";
      switcher.dispatchEvent(new Event("change", { bubbles: true }));
      await backend.switchForegroundEngine.mock.results.at(-1)?.value;
    });
    expect(backend.switchForegroundEngine).toHaveBeenCalledWith("profile-3");
    await act(async () => {
      listeners.onSnapshot?.({
        revision: 5,
        lifecycle: {
          state: "switching",
          primary: readyRun("run-1", savedProfile),
          candidate: {
            ...readyRun("run-c", savedProfileC),
            capability_snapshot: null
          },
          switch_id: "8"
        }
      });
    });
    await act(async () => {
      listeners.onSnapshot?.({
        revision: 6,
        lifecycle: { state: "ready", run: readyRun("run-c", savedProfileC) }
      });
      listeners.onFailure?.({
        operation: "switch",
        run_id: "run-b",
        switch_id: "7",
        profile_id: "profile-2",
        kind: "timeout",
        message: "engine readiness probe timed out"
      });
    });
    expect(host.querySelector(".engine-failure")).toBeNull();
    expect(host.querySelector(".engine-chip-label")?.textContent).toBe("Third KataGo");
    expect(switcher.value).toBe("profile-3");
    backend.startSelectedNodeAnalysis.mockClear();
    await act(async () => {
      buttonNamed(host, "继续分析").click();
      await backend.startSelectedNodeAnalysis.mock.results.at(-1)?.value;
    });
    expect(backend.startSelectedNodeAnalysis).toHaveBeenCalledWith({
      runId: "run-c",
      generation: 1,
      nodePath: { indices: [] },
      maxVisits: 200
    });
  });

  it("keeps B's typed failure when Ready A arrives after the switch failure event", async () => {
    const host = await renderApp();
    await readyEngine(host);
    await act(async () => {
      listeners.onSnapshot?.({
        revision: 3,
        lifecycle: {
          state: "switching",
          primary: readyRun("run-1", savedProfile),
          candidate: {
            ...readyRun("run-b", savedProfileB),
            capability_snapshot: null
          },
          switch_id: "7"
        }
      });
    });
    await act(async () => {
      listeners.onFailure?.({
        operation: "switch",
        run_id: "run-b",
        switch_id: "7",
        profile_id: "profile-2",
        kind: "asset",
        message: "required engine assets are missing"
      });
    });
    await act(async () => {
      listeners.onSnapshot?.({
        revision: 4,
        lifecycle: { state: "ready", run: readyRun("run-1", savedProfile) }
      });
    });
    expect(host.querySelector(".engine-chip-label")?.textContent).toBe("Local KataGo");
    const failure = host.querySelector(".engine-failure") as HTMLElement;
    expect(failure.dataset.failureKind).toBe("asset");
    expect(failure.textContent).toContain("required engine assets are missing");
  });

  it("enters Error when A crashes during switch and ignores B's failure", async () => {
    const host = await renderApp();
    await readyEngine(host);
    await act(async () => {
      listeners.onSnapshot?.({
        revision: 3,
        lifecycle: {
          state: "switching",
          primary: readyRun("run-1", savedProfile),
          candidate: {
            ...readyRun("run-b", savedProfileB),
            capability_snapshot: null
          },
          switch_id: "7"
        }
      });
    });
    await act(async () => {
      listeners.onSnapshot?.({
        revision: 4,
        lifecycle: {
          state: "error",
          run: readyRun("run-1", savedProfile),
          failure: crashFailure
        }
      });
      listeners.onFailure?.({
        operation: "switch",
        run_id: "run-b",
        switch_id: "7",
        profile_id: "profile-2",
        kind: "timeout",
        message: "engine readiness probe timed out"
      });
    });
    expect(host.querySelector(".engine-chip-label")?.textContent).toBe("引擎错误");
    const failure = host.querySelector(".engine-failure") as HTMLElement;
    expect(failure.dataset.failureKind).toBe("nonzero_exit");
    expect(failure.textContent).toContain("engine process exited unexpectedly");
    expect(buttonNamed(host, "停止").disabled).toBe(false);
    expect(buttonNamed(host, "重启").disabled).toBe(false);
    backend.startSelectedNodeAnalysis.mockClear();
    await act(async () => {
      buttonNamed(host, "继续分析").click();
    });
    expect(backend.startSelectedNodeAnalysis).not.toHaveBeenCalled();
  });

  it("shows a typed Autoload failure from the first No-engine snapshot without a Failure event", async () => {
    const host = await renderApp();
    await act(async () => {
      listeners.onSnapshot?.({
        revision: 2,
        lifecycle: {
          state: "no_engine",
          failure: {
            operation: "autoload",
            run_id: "attempt-1",
            profile_id: "profile-1",
            kind: "asset",
            message: "required engine assets are missing"
          }
        }
      });
    });
    expect(host.querySelector(".engine-chip-label")?.textContent).toBe("未加载引擎");
    const failure = host.querySelector(".engine-failure") as HTMLElement;
    expect(failure.dataset.failureKind).toBe("asset");
    expect(failure.dataset.failureOperation).toBe("autoload");
    expect(failure.textContent).toContain("autoload");
    expect(failure.textContent).toContain("required engine assets are missing");
    expect((buttonNamed(host, "重启") as HTMLButtonElement).disabled).toBe(true);
  });

  it("does not revive a stale event failure after Stop returns to clean No-engine", async () => {
    const host = await renderApp();
    await readyEngine(host);
    await act(async () => {
      listeners.onFailure?.({
        operation: "autoload",
        run_id: "attempt-1",
        profile_id: "profile-1",
        kind: "asset",
        message: "required engine assets are missing"
      });
    });
    await act(async () => {
      listeners.onSnapshot?.({ revision: 9, lifecycle: { state: "no_engine" } });
    });
    expect(host.querySelector(".engine-chip-label")?.textContent).toBe("未加载引擎");
    expect(host.querySelector(".engine-failure")).toBeNull();
    await act(async () => {
      listeners.onFailure?.({
        operation: "autoload",
        run_id: "attempt-1",
        profile_id: "profile-1",
        kind: "asset",
        message: "required engine assets are missing"
      });
    });
    expect(host.querySelector(".engine-failure")).toBeNull();
  });

  it("saves Autoload Default from Engine Settings without changing the current Run", async () => {
    const host = await renderApp();
    await readyEngine(host);
    const panel = await openEngineSettings(host);
    const checkbox = panel.querySelector('input[aria-label="Autoload Default"]') as HTMLInputElement;
    expect(checkbox.checked).toBe(false);
    backend.startForegroundEngine.mockClear();
    backend.stopForegroundEngine.mockClear();
    backend.restartForegroundEngine.mockClear();
    await act(async () => {
      checkbox.click();
      await backend.saveEngineProfilesSettings.mock.results.at(-1)?.value;
    });
    expect(backend.saveEngineProfilesSettings).toHaveBeenCalledWith(
      expect.objectContaining({ autoload_profile_id: "profile-1", selected_profile_id: "profile-1" })
    );
    expect(backend.startForegroundEngine).not.toHaveBeenCalled();
    expect(backend.stopForegroundEngine).not.toHaveBeenCalled();
    expect(backend.restartForegroundEngine).not.toHaveBeenCalled();
    expect(host.querySelector(".engine-chip-label")?.textContent).toBe("Local KataGo");
    expect(checkbox.checked).toBe(true);
  });

  it("rolls back Autoload Default when persistence fails and leaves the current Run unchanged", async () => {
    const host = await renderApp();
    await readyEngine(host);
    const panel = await openEngineSettings(host);
    const checkbox = panel.querySelector('input[aria-label="Autoload Default"]') as HTMLInputElement;
    backend.saveEngineProfilesSettings.mockRejectedValueOnce(new Error("disk full"));
    backend.startForegroundEngine.mockClear();
    backend.stopForegroundEngine.mockClear();
    backend.restartForegroundEngine.mockClear();
    await act(async () => {
      checkbox.click();
      await Promise.resolve();
    });
    expect(checkbox.checked).toBe(false);
    expect(panel.textContent).toContain("disk full");
    expect(backend.startForegroundEngine).not.toHaveBeenCalled();
    expect(backend.stopForegroundEngine).not.toHaveBeenCalled();
    expect(backend.restartForegroundEngine).not.toHaveBeenCalled();
    expect(host.querySelector(".engine-chip-label")?.textContent).toBe("Local KataGo");
  });

  it("does not rewrite Autoload Default when the Engine Switcher starts a profile", async () => {
    backend.saveEngineProfilesSettings.mockClear();
    const host = await renderApp();
    const switcher = host.querySelector('select[aria-label="Foreground Engine Profile"]') as HTMLSelectElement;
    await act(async () => {
      switcher.value = "profile-1";
      switcher.dispatchEvent(new Event("change", { bubbles: true }));
      await backend.startForegroundEngine.mock.results[0]?.value;
    });
    expect(backend.startForegroundEngine).toHaveBeenCalledWith("profile-1");
    expect(backend.saveEngineProfilesSettings).not.toHaveBeenCalled();
  });

  it("shows typed Error recovery with Restart, Stop, and Open Settings", async () => {
    const host = await renderApp();
    await act(async () => {
      listeners.onSnapshot?.({
        revision: 3,
        lifecycle: {
          state: "error",
          run: readyRun("run-1", savedProfile),
          failure: crashFailure
        }
      });
    });
    expect(host.querySelector(".engine-chip-label")?.textContent).toBe("引擎错误");
    const failure = host.querySelector(".engine-failure") as HTMLElement;
    expect(failure.dataset.failureKind).toBe("nonzero_exit");
    expect(failure.textContent).toContain("nonzero_exit");
    const stop = buttonNamed(host, "停止");
    const restart = buttonNamed(host, "重启");
    const settings = buttonNamed(host, "设置");
    expect(stop.disabled).toBe(false);
    expect(restart.disabled).toBe(false);
    expect(settings).toBeTruthy();
    await act(async () => {
      restart.click();
      stop.click();
      settings.click();
    });
    expect(backend.restartForegroundEngine).toHaveBeenCalledOnce();
    expect(backend.stopForegroundEngine).toHaveBeenCalledOnce();
    expect(host.querySelector(".engine-setup-panel")).not.toBeNull();
  });

  it("applies a repaired saved record as a new Run identity after Restart", async () => {
    const host = await renderApp();
    await act(async () => {
      listeners.onSnapshot?.({
        revision: 3,
        lifecycle: { state: "error", run: readyRun("run-1", savedProfile), failure: crashFailure }
      });
    });
    await act(async () => {
      buttonNamed(host, "重启").click();
    });
    expect(backend.restartForegroundEngine).toHaveBeenCalledOnce();
    await act(async () => {
      listeners.onSnapshot?.({
        revision: 6,
        lifecycle: {
          state: "ready",
          run: {
            ...readyRun("run-2", savedProfile),
            profile_snapshot: { ...savedProfile.profile, name: "Repaired KataGo" }
          }
        }
      });
    });
    expect(host.querySelector(".engine-chip-label")?.textContent).toBe("Repaired KataGo");
    expect(host.querySelector(".engine-failure")).toBeNull();
  });

  it("keeps a typed failure after a failed recovery Restart and ignores a stale crash", async () => {
    const host = await renderApp();
    await act(async () => {
      listeners.onSnapshot?.({
        revision: 3,
        lifecycle: { state: "error", run: readyRun("run-1", savedProfile), failure: crashFailure }
      });
    });
    await act(async () => {
      buttonNamed(host, "重启").click();
    });
    expect(backend.restartForegroundEngine).toHaveBeenCalledOnce();
    await act(async () => {
      listeners.onSnapshot?.({
        revision: 5,
        lifecycle: {
          state: "no_engine",
          failure: {
            operation: "start",
            run_id: "run-2",
            profile_id: "profile-1",
            kind: "asset",
            message: "required engine assets are missing"
          }
        }
      });
    });
    const failure = host.querySelector(".engine-failure") as HTMLElement;
    expect(failure.dataset.failureKind).toBe("asset");
    expect(failure.dataset.failureOperation).toBe("start");
    expect((buttonNamed(host, "重启") as HTMLButtonElement).disabled).toBe(true);
    await act(async () => {
      listeners.onFailure?.(crashFailure);
    });
    expect((host.querySelector(".engine-failure") as HTMLElement).dataset.failureKind).toBe("asset");
  });

  it("rejects deleting the Error profile until Stop", async () => {
    backend.loadEngineProfilesSettings.mockResolvedValue({
      selected_profile_id: "profile-1",
      autoload_profile_id: null,
      profiles: [savedProfile, savedProfileB]
    });
    backend.saveEngineProfilesSettings.mockImplementation(async (settings) => {
      if (!settings.profiles.some((profile: { id: string }) => profile.id === "profile-1")) {
        throw new Error("an active Foreground Engine Run still holds this profile identity");
      }
      return settings;
    });
    const host = await renderApp();
    await act(async () => {
      listeners.onSnapshot?.({
        revision: 3,
        lifecycle: { state: "error", run: readyRun("run-1", savedProfile), failure: crashFailure }
      });
    });
    const panel = await openEngineSettings(host);
    const deleteButton = Array.from(panel.querySelectorAll("button")).find((button) => button.textContent === "删除") as HTMLButtonElement;
    expect(deleteButton.disabled).toBe(false);
    await act(async () => {
      deleteButton.click();
      await backend.saveEngineProfilesSettings.mock.results.at(-1)?.value.catch(() => undefined);
    });
    expect(panel.querySelector(".message")?.textContent).toContain("Delete failed");
  });
});
