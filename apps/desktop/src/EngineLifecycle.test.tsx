// @vitest-environment jsdom

import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { CurrentGameResultDto, ForegroundEngineSnapshotDto, GameDto } from "./domain/types";

const listeners: {
  onSnapshot?: (snapshot: ForegroundEngineSnapshotDto) => void;
  onJob?: (job: unknown) => void;
} = {};
const analysisListeners: {
  onProgress?: (payload: { run_id: string; job_id: string; completed: number; expected: number; turn: number; response_jsonl: string }) => void;
  onComplete?: (payload: { run_id: string; job_id: string; frames: unknown[] }) => void;
  onError?: (payload: { run_id: string; job_id: string; message: string }) => void;
  onCancelled?: (payload: { run_id: string; job_id: string; message: string }) => void;
} = {};
const analysisCache = vi.hoisted(() => ({
  computeGameCacheKey: vi.fn(() => Promise.resolve({ gameKey: "game", sgfHash: "hash" })),
  loadAnalysisCache: vi.fn(() => Promise.resolve({ status: "miss" })),
  saveAnalysisCache: vi.fn(() => Promise.resolve({ id: "c1", gameKey: "game", updatedAt: "now" }))
}));

const backend = vi.hoisted(() => ({
  getHealth: vi.fn(() => Promise.resolve({ status: "ok" })),
  replaceCurrentGame: vi.fn(),
  serializeCurrentGame: vi.fn(() => Promise.resolve("(;SZ[9])")),
  projectCurrentGameMainline: vi.fn(),
  playCurrentGame: vi.fn(),
  selectCurrentGameNode: vi.fn(),
  analyzeKataGoOnce: vi.fn(),
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

vi.mock("./api/backend", () => ({
  ...backend,
  isTauriRuntime: () => true,
  nativeCurrentGameUnavailable: "Native current-game commands require the Tauri desktop runtime."
}));

vi.mock("./api/analysisCache", () => analysisCache);

vi.mock("./api/preferences", () => ({
  loadAppPreferences: vi.fn(() => Promise.reject(new Error("preferences unavailable in test"))),
  saveAppPreferences: vi.fn()
}));

vi.mock("./components/CacheStatusBadge", () => ({ CacheStatusBadge: () => null }));
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

const capability = {
  adapter_kind: "kata_go_analysis" as const,
  selected_node_analysis: true,
  whole_game_analysis: true,
  protocol_cancel: true
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
  backend.subscribeForegroundEngine.mockImplementation(async (onSnapshot, _onFailure, onJob) => {
    listeners.onSnapshot = onSnapshot;
    listeners.onJob = onJob;
    onSnapshot({ revision: 0, lifecycle: { state: "no_engine" } });
    return () => undefined;
  });
  backend.listenToKataGoAnalysisEvents.mockImplementation(async (handlers) => {
    analysisListeners.onProgress = handlers.onProgress;
    analysisListeners.onComplete = handlers.onComplete;
    analysisListeners.onError = handlers.onError;
    analysisListeners.onCancelled = handlers.onCancelled;
    return () => undefined;
  });
  backend.startKataGoGameAnalysis.mockResolvedValue("job-1");
  backend.cancelKataGoAnalysis.mockResolvedValue(undefined);
});

afterEach(() => {
  act(() => root?.unmount());
  root = null;
  document.body.replaceChildren();
  vi.clearAllMocks();
  vi.restoreAllMocks();
  analysisListeners.onProgress = undefined;
  analysisListeners.onComplete = undefined;
  analysisListeners.onError = undefined;
  analysisListeners.onCancelled = undefined;
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
    expect(backend.analyzeKataGoOnce).not.toHaveBeenCalled();
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
  it("starts whole-game analysis with the Ready Run identity and keeps cancel from stopping the engine", async () => {
    const host = await renderApp();
    await readyEngine(host);
    await act(async () => {
      buttonNamed(host, "自动分析").click();
      await backend.startKataGoGameAnalysis.mock.results.at(-1)?.value;
    });
    expect(backend.startKataGoGameAnalysis).toHaveBeenCalledWith("run-1", "(;SZ[9])", 800);
    await act(async () => {
      analysisListeners.onProgress?.({
        run_id: "run-1",
        job_id: "job-1",
        completed: 1,
        expected: 2,
        turn: 0,
        response_jsonl: "{}"
      });
    });
    expect(host.querySelector(".nav-progress")?.textContent).toContain("1/2");
    expect(host.textContent).toContain("Analyzing move 0");
    await act(async () => {
      buttonNamed(host, "取消").click();
      await backend.cancelKataGoAnalysis.mock.results.at(-1)?.value;
    });
    expect(backend.cancelKataGoAnalysis).toHaveBeenCalledWith("run-1", "job-1");
    expect(backend.stopForegroundEngine).not.toHaveBeenCalled();
    await act(async () => {
      analysisListeners.onCancelled?.({
        run_id: "run-1",
        job_id: "job-1",
        message: "analysis job was cancelled"
      });
    });
    expect(host.textContent).toContain("analysis job was cancelled");
    expect(host.querySelector(".engine-chip-label")?.textContent).toBe("Local KataGo");
  });

  it("does not restore presentation or cache from a stale whole-game completion", async () => {
    const host = await renderApp();
    await readyEngine(host);
    await act(async () => {
      buttonNamed(host, "自动分析").click();
      await backend.startKataGoGameAnalysis.mock.results.at(-1)?.value;
    });
    await act(async () => {
      buttonNamed(host, "取消").click();
      await backend.cancelKataGoAnalysis.mock.results.at(-1)?.value;
      analysisListeners.onCancelled?.({
        run_id: "run-1",
        job_id: "job-1",
        message: "analysis job was cancelled"
      });
    });
    analysisCache.saveAnalysisCache.mockClear();
    await act(async () => {
      analysisListeners.onComplete?.({
        run_id: "run-1",
        job_id: "job-1",
        frames: [{
          job_id: "job-1",
          turn: 0,
          visits: 8,
          winrate_black: 0.5,
          score_mean_black: 0,
          candidates: []
        }]
      });
      analysisListeners.onComplete?.({
        run_id: "run-old",
        job_id: "job-1",
        frames: [{
          job_id: "job-1",
          turn: 1,
          visits: 8,
          winrate_black: 0.9,
          score_mean_black: 4,
          candidates: []
        }]
      });
    });
    expect(analysisCache.saveAnalysisCache).not.toHaveBeenCalled();
    expect(host.textContent).not.toContain("Full-game KataGo analysis completed");
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
        (id) => !settings.profiles.some((profile) => profile.id === id)
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
});
