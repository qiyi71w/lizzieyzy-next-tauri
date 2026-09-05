// @vitest-environment jsdom

import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { CurrentGameResultDto, ForegroundEngineSnapshotDto, GameDto } from "./domain/types";

const listeners: {
  onSnapshot?: (snapshot: ForegroundEngineSnapshotDto) => void;
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
    autoload_profile_id: null,
    profiles: [savedProfile]
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

describe("selected-node analysis presentation", () => {
  it("publishes matching selected-node candidates, PV, ownership, policy, and score", async () => {
    const host = await renderApp();
    await readyEngine(host);
    await act(async () => {
      buttonNamed(host, "分析当前节点").click();
      await backend.startSelectedNodeAnalysis.mock.results[0]?.value;
    });
    const ownership = Array.from({ length: 81 }, () => 0.25);
    const policy = Array.from({ length: 81 }, () => 0);
    policy[3 + 3 * 9] = 0.4;
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
          visits: 48,
          winrate_black: 0.61,
          score_mean_black: 2.7,
          score_stdev: 8.2,
          candidates: [
            {
              vertex: { point: { x: 3, y: 3 } },
              visits: 40,
              winrate_black: 0.62,
              score_mean_black: 2.8,
              pv: [{ point: { x: 3, y: 3 } }, { point: { x: 2, y: 2 } }]
            }
          ],
          ownership,
          policy
        }
      });
      await Promise.all([
        backend.classifyProblems.mock.results.at(-1)?.value,
        backend.selectCurrentGameNode.mock.results.at(-1)?.value
      ]);
    });
    expect(Array.from(host.querySelectorAll(".cand-coord")).map((node) => node.textContent)).toEqual(["D6"]);
    expect(host.querySelector(".cand-winrate")?.textContent).toBe("62.0%");
    expect(host.querySelector(".cand-visits")?.textContent).toBe("40");
    expect(host.querySelector(".cand-score")?.textContent).toBe("+2.8");
    expect(host.textContent).toContain("KataGo 推荐首选 D6");
    expect(host.textContent).toContain("胜率 61.0%");
    expect(host.textContent).toContain("+2.7");
    expect(host.textContent).toContain("领地已评估");
    expect(buttonNamed(host, "领地").disabled).toBe(false);
    expect(buttonNamed(host, "策略").disabled).toBe(false);
  });

  it("does not publish selected-node candidates after a typed protocol failure", async () => {
    const host = await renderApp();
    await readyEngine(host);
    await act(async () => {
      buttonNamed(host, "分析当前节点").click();
      await backend.startSelectedNodeAnalysis.mock.results[0]?.value;
    });
    await act(async () => {
      listeners.onJob?.({
        run_id: "run-1",
        job_id: "job-1",
        lane: "selected_node",
        generation: 1,
        node_path: { indices: [] },
        outcome: "failed",
        failure: {
          operation: "job",
          kind: "protocol",
          message: "stderr boom"
        }
      });
    });
    expect(host.querySelector(".cand-row")).toBeNull();
    expect(buttonNamed(host, "领地").disabled).toBe(true);
    expect(buttonNamed(host, "策略").disabled).toBe(true);
    expect(host.textContent).toContain("stderr boom");
  });
});
