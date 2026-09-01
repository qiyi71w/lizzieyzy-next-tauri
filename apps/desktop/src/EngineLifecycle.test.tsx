// @vitest-environment jsdom

import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { CurrentGameResultDto, EngineFailureDto, ForegroundEngineSnapshotDto, GameDto } from "./domain/types";

const listeners: {
  onSnapshot?: (snapshot: ForegroundEngineSnapshotDto) => void;
  onFailure?: (failure: EngineFailureDto) => void;
} = {};

const backend = vi.hoisted(() => ({
  getHealth: vi.fn(() => Promise.resolve({ status: "ok" })),
  replaceCurrentGame: vi.fn(),
  serializeCurrentGame: vi.fn(() => Promise.resolve("(;SZ[9])")),
  projectCurrentGameMainline: vi.fn(),
  playCurrentGame: vi.fn(),
  selectCurrentGameNode: vi.fn(),
  analyzeKataGoOnce: vi.fn(),
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
  getForegroundEngineSnapshot: vi.fn(),
  subscribeForegroundEngine: vi.fn()
}));

vi.mock("./api/backend", () => ({
  ...backend,
  isTauriRuntime: () => true,
  nativeCurrentGameUnavailable: "Native current-game commands require the Tauri desktop runtime."
}));

vi.mock("./api/analysisCache", () => ({
  computeGameCacheKey: vi.fn(() => Promise.resolve({ gameKey: "game", sgfHash: "hash" })),
  loadAnalysisCache: vi.fn(() => Promise.resolve({ status: "miss" })),
  saveAnalysisCache: vi.fn()
}));

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
    profiles: [savedProfile]
  });
  backend.saveEngineProfilesSettings.mockImplementation(async (settings) => settings);
  backend.getForegroundEngineSnapshot.mockResolvedValue({ revision: 0, lifecycle: { state: "no_engine" } });
  backend.subscribeForegroundEngine.mockImplementation(async (onSnapshot, onFailure) => {
    listeners.onSnapshot = onSnapshot;
    listeners.onFailure = onFailure;
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

  it("shows typed Error recovery with Restart, Stop, and Open Settings", async () => {
    const host = await renderApp();
    await act(async () => {
      listeners.onSnapshot?.({
        revision: 3,
        lifecycle: {
          state: "error",
          run: {
            run_id: "run-1",
            profile_id: "profile-1",
            adapter_kind: "kata_go_analysis",
            profile_snapshot: savedProfile.profile,
            capability_snapshot: capability
          },
          failure: crashFailure
        }
      });
    });
    expect(host.querySelector(".engine-chip-label")?.textContent).toBe("引擎错误");
    const failure = host.querySelector(".engine-failure") as HTMLElement;
    expect(failure.dataset.failureKind).toBe("nonzero_exit");
    expect(failure.textContent).toContain("nonzero_exit");
    const stop = Array.from(host.querySelectorAll("button")).find((button) => button.textContent === "停止") as HTMLButtonElement;
    const restart = Array.from(host.querySelectorAll("button")).find((button) => button.textContent === "重启") as HTMLButtonElement;
    const settings = Array.from(host.querySelectorAll("button")).find((button) => button.textContent === "设置") as HTMLButtonElement;
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
        lifecycle: {
          state: "error",
          run: {
            run_id: "run-1",
            profile_id: "profile-1",
            adapter_kind: "kata_go_analysis",
            profile_snapshot: savedProfile.profile,
            capability_snapshot: capability
          },
          failure: crashFailure
        }
      });
    });
    await act(async () => {
      Array.from(host.querySelectorAll("button")).find((button) => button.textContent === "重启")?.click();
    });
    expect(backend.restartForegroundEngine).toHaveBeenCalledOnce();
    await act(async () => {
      listeners.onSnapshot?.({
        revision: 6,
        lifecycle: {
          state: "ready",
          run: {
            run_id: "run-2",
            profile_id: "profile-1",
            adapter_kind: "kata_go_analysis",
            profile_snapshot: { ...savedProfile.profile, name: "Repaired KataGo" },
            capability_snapshot: capability
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
        lifecycle: {
          state: "error",
          run: {
            run_id: "run-1",
            profile_id: "profile-1",
            adapter_kind: "kata_go_analysis",
            profile_snapshot: savedProfile.profile,
            capability_snapshot: capability
          },
          failure: crashFailure
        }
      });
    });
    await act(async () => {
      Array.from(host.querySelectorAll("button")).find((button) => button.textContent === "重启")?.click();
    });
    expect(backend.restartForegroundEngine).toHaveBeenCalledOnce();
    await act(async () => {
      listeners.onSnapshot?.({ revision: 5, lifecycle: { state: "no_engine" } });
      listeners.onFailure?.({
        operation: "start",
        run_id: "run-2",
        profile_id: "profile-1",
        kind: "asset",
        message: "required engine assets are missing"
      });
    });
    const failure = host.querySelector(".engine-failure") as HTMLElement;
    expect(failure.dataset.failureKind).toBe("asset");
    await act(async () => {
      listeners.onFailure?.(crashFailure);
    });
    expect((host.querySelector(".engine-failure") as HTMLElement).dataset.failureKind).toBe("asset");
  });

  it("rejects deleting the Error profile until Stop", async () => {
    const extraProfile = {
      id: "profile-2",
      max_visits: 400,
      profile: { ...savedProfile.profile, name: "Other KataGo" }
    };
    backend.loadEngineProfilesSettings.mockResolvedValue({
      selected_profile_id: "profile-1",
      profiles: [savedProfile, extraProfile]
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
        lifecycle: {
          state: "error",
          run: {
            run_id: "run-1",
            profile_id: "profile-1",
            adapter_kind: "kata_go_analysis",
            profile_snapshot: savedProfile.profile,
            capability_snapshot: capability
          },
          failure: crashFailure
        }
      });
    });
    act(() => {
      Array.from(host.querySelectorAll("button")).find((button) => button.textContent === "设置")?.click();
    });
    const deleteButton = Array.from(host.querySelectorAll("button")).find((button) => button.textContent === "删除") as HTMLButtonElement;
    expect(deleteButton.disabled).toBe(false);
    await act(async () => {
      deleteButton.click();
    });
    expect(host.querySelector(".engine-setup-panel .message")?.textContent).toContain("Delete failed");
  });
});
