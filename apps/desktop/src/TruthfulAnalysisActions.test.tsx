// @vitest-environment jsdom

import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { CurrentGameResultDto, ForegroundEngineSnapshotDto, GameDto } from "./domain/types";

const runtime = vi.hoisted(() => ({
  native: true,
  nativeUnavailable:
    "Native current-game, edit, and authoritative Save require the Tauri desktop backend. Browser preview is non-authoritative."
}));
const listeners: {
  onSnapshot?: (snapshot: ForegroundEngineSnapshotDto) => void;
  onJob?: (job: unknown) => void;
} = {};

const currentGameFixture = vi.hoisted(() => vi.fn());
const backend = vi.hoisted(() => ({
  getHealth: vi.fn(() => Promise.resolve({ status: "ok" })),
  prepareDocumentReplacement: vi.fn(async () => ({ status: "ready", departure_id: 1 })),
  resolveDocumentReplacement: vi.fn(async (input: { action: string }) => {
    if (input.action === "cancel") {
      return { committed: false, analysis_stopped: false, current: null, message: "Replacement cancelled." };
    }
    const current = await currentGameFixture("", null);
    return { committed: true, analysis_stopped: true, current, message: "Replacement committed." };
  }),
  inspectCurrentGameRecovery: vi.fn(async () => ({ status: "none" as const })),
  restoreCurrentGameRecovery: vi.fn(),
  discardCurrentGameRecovery: vi.fn(async () => undefined),
  retryCurrentGameRecovery: vi.fn(async () => ({ status: "protected" })),
  currentGameRecoveryProtection: vi.fn(async () => ({ status: "protected" })),
  subscribeCurrentGameRecoveryProtection: vi.fn(async () => () => undefined),
  subscribeApplicationExitRequested: vi.fn(async () => () => undefined),
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
  subscribeForegroundEngine: vi.fn()
}));

vi.mock("./api/backend", () => ({
  ...backend,
  isTauriRuntime: () => runtime.native,
  nativeCurrentGameUnavailable: runtime.nativeUnavailable
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
  snapshot_seq: 1,
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

const previewGame: GameDto = {
  summary: { id: "browser-sgf", board_size: 9, komi: 7.5, move_count: 1 },
  moves: [{ move_number: 1, color: "black", vertex: { point: { x: 3, y: 3 } } }]
};

let root: Root | null = null;

beforeEach(() => {
  globalThis.IS_REACT_ACT_ENVIRONMENT = true;
  runtime.native = true;
  vi.spyOn(HTMLCanvasElement.prototype, "getContext").mockImplementation(function (this: HTMLCanvasElement) {
    return canvasContext(this);
  });
  currentGameFixture.mockResolvedValue(initialGame);
  backend.projectCurrentGameMainline.mockResolvedValue(initialProjection);
  backend.parseSgfSummary.mockResolvedValue(previewGame);
  backend.replaySgfPositions.mockResolvedValue([emptyPosition]);
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
  backend.fakeAnalyze.mockResolvedValue([
    {
      job_id: "browser-preview",
      turn: 0,
      visits: 800,
      winrate_black: 0.51,
      score_mean_black: 0.3,
      candidates: [
        {
          vertex: { point: { x: 3, y: 3 } },
          visits: 40,
          winrate_black: 0.52,
          score_mean_black: 1.1,
          pv: []
        }
      ]
    }
  ]);
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
    await backend.inspectCurrentGameRecovery.mock.results.at(-1)?.value.catch(() => undefined);
    await currentGameFixture.mock.results[0]?.value.catch(() => undefined);
    await backend.projectCurrentGameMainline.mock.results[0]?.value.catch(() => undefined);
    await backend.subscribeForegroundEngine.mock.results[0]?.value;
    await Promise.resolve();
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
      continuous: { enabled: true, phase: "waiting" },
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
  const button = Array.from(host.querySelectorAll("button")).find((candidate) => candidate.textContent === label);
  if (!(button instanceof HTMLButtonElement)) throw new Error(`Missing button: ${label}`);
  return button;
}

function openAnalyzeMenu(host: HTMLElement) {
  act(() => buttonNamed(host, "分析").click());
}

function canvasContext(canvas: HTMLCanvasElement): CanvasRenderingContext2D {
  const target = { canvas } as CanvasRenderingContext2D;
  return new Proxy(target, {
    get(object, property) {
      if (property in object) return Reflect.get(object, property);
      if (property === "measureText") return () => ({ width: 0 });
      return vi.fn();
    },
    set(object, property, value) {
      Reflect.set(object, property, value);
      return true;
    }
  });
}

describe("truthful native analysis actions", () => {
  it("keeps unimplemented analysis names visible-disabled and does not call fakeAnalyze", async () => {
    const host = await renderApp();
    expect(buttonNamed(host, "分析当前节点").disabled).toBe(true);
    expect(buttonNamed(host, "分析第一子主线").disabled).toBe(true);
    expect(buttonNamed(host, "AI 解说").disabled).toBe(true);
    expect(buttonNamed(host, "AI 解说").title).toMatch(/尚未接入/);
    expect(buttonNamed(host, "闪电分析").disabled).toBe(true);
    expect(buttonNamed(host, "闪电分析").title).toMatch(/尚未接入|闪电分析/);
    expect(host.querySelector('button[aria-label="闪电分析"]')).toBeInstanceOf(HTMLButtonElement);
    expect((host.querySelector('button[aria-label="闪电分析"]') as HTMLButtonElement).disabled).toBe(true);

    openAnalyzeMenu(host);
    const lightning = buttonNamed(host, "试复盘 / 闪电分析");
    expect(lightning.disabled).toBe(true);
    expect(lightning.title).toMatch(/尚未接入|闪电分析/);
    const autoAnalyze = buttonNamed(host, "自动分析");
    expect(autoAnalyze.disabled).toBe(true);
    expect(autoAnalyze.textContent).toBe("自动分析");
    expect(host.textContent).not.toContain("自动分析(A)");
    expect(buttonNamed(host, "分析当前节点").textContent).not.toMatch(/Space|空格|\(A\)/i);
    expect(buttonNamed(host, "分析第一子主线").textContent).not.toMatch(/Space|空格|\(A\)/i);

    backend.fakeAnalyze.mockClear();
    backend.startSelectedNodeAnalysis.mockClear();
    backend.startKataGoGameAnalysis.mockClear();
    act(() => {
      buttonNamed(host, "AI 解说").click();
      buttonNamed(host, "闪电分析").click();
      lightning.click();
      autoAnalyze.click();
    });
    expect(backend.fakeAnalyze).not.toHaveBeenCalled();
    expect(backend.startSelectedNodeAnalysis).not.toHaveBeenCalled();
    expect(backend.startKataGoGameAnalysis).not.toHaveBeenCalled();
  });

  it("binds current-node and first-child-mainline actions to independent manager-backed commands", async () => {
    const host = await renderApp();
    await readyEngine(host);
    await act(async () => {
      buttonNamed(host, "分析当前节点").click();
      buttonNamed(host, "分析第一子主线").click();
      await backend.startSelectedNodeAnalysis.mock.results.at(-1)?.value;
      await backend.startKataGoGameAnalysis.mock.results.at(-1)?.value;
    });
    expect(backend.startSelectedNodeAnalysis).toHaveBeenCalledWith({
      runId: "run-1",
      generation: 1,
      nodePath: { indices: [] },
      maxVisits: 800
    });
    expect(backend.startKataGoGameAnalysis).toHaveBeenCalledWith({
      runId: "run-1",
      generation: 1,
      maxVisits: 800
    });
    expect(backend.fakeAnalyze).not.toHaveBeenCalled();

    backend.cancelSelectedNodeAnalysis.mockClear();
    backend.cancelKataGoAnalysis.mockClear();
    await act(async () => {
      buttonNamed(host, "取消此手").click();
      await backend.cancelSelectedNodeAnalysis.mock.results.at(-1)?.value;
    });
    expect(backend.cancelSelectedNodeAnalysis).toHaveBeenCalledWith({ runId: "run-1", jobId: "job-1" });
    expect(backend.cancelKataGoAnalysis).not.toHaveBeenCalled();

    await act(async () => {
      buttonNamed(host, "取消整局").click();
      await backend.cancelKataGoAnalysis.mock.results.at(-1)?.value;
    });
    expect(backend.cancelKataGoAnalysis).toHaveBeenCalledWith("run-1", "job-wg");
    expect(backend.cancelSelectedNodeAnalysis).toHaveBeenCalledTimes(1);
  });

  it("keeps a native analysis failure as an error without manufacturing candidates", async () => {
    const host = await renderApp();
    await readyEngine(host);
    backend.startSelectedNodeAnalysis.mockRejectedValueOnce(new Error("engine protocol boom"));
    await act(async () => {
      buttonNamed(host, "分析当前节点").click();
      await Promise.resolve(backend.startSelectedNodeAnalysis.mock.results.at(-1)?.value).catch(() => undefined);
    });
    expect(host.querySelectorAll(".cand-row")).toHaveLength(0);
    expect(host.textContent).toMatch(/engine protocol boom/);
    expect(backend.fakeAnalyze).not.toHaveBeenCalled();

    backend.startSelectedNodeAnalysis.mockResolvedValueOnce({
      run_id: "run-1",
      job_id: "job-fail",
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
    await act(async () => {
      listeners.onJob?.({
        run_id: "run-1",
        job_id: "job-fail",
        lane: "selected_node",
        mode: "finite",
        generation: 1,
        node_path: { indices: [] },
        outcome: "failed",
        failure: { operation: "job", kind: "protocol", message: "stderr boom" }
      });
    });
    expect(host.querySelectorAll(".cand-row")).toHaveLength(0);
    expect(host.textContent).toContain("stderr boom");
    expect(backend.fakeAnalyze).not.toHaveBeenCalled();
  });
});

describe("truthful browser demonstration analysis", () => {
  it("keeps the labeled preview isolated from native save and current-game writes", async () => {
    runtime.native = false;
    const host = await renderApp();
    expect(host.querySelector(".native-runtime-note")?.textContent).toMatch(/non-authoritative/i);
    const demos = [...host.querySelectorAll("button")].filter((button) => button.textContent === "预览复盘（非权威）");
    expect(demos.length).toBeGreaterThan(0);
    expect(demos.every((button) => !button.disabled)).toBe(true);
    backend.saveCurrentGame.mockClear();
    backend.prepareDocumentReplacement.mockClear();
    backend.resolveDocumentReplacement.mockClear();
    await act(async () => {
      demos[0]?.click();
      await backend.fakeAnalyze.mock.results.at(-1)?.value;
      await backend.classifyProblems.mock.results.at(-1)?.value;
    });
    expect(backend.fakeAnalyze).toHaveBeenCalled();
    expect(backend.saveCurrentGame).not.toHaveBeenCalled();
    expect(backend.prepareDocumentReplacement).not.toHaveBeenCalled();
    expect(backend.resolveDocumentReplacement).not.toHaveBeenCalled();
    expect(host.querySelectorAll(".cand-row").length).toBeGreaterThan(0);
    expect(host.textContent).toMatch(/non-authoritative|预览复盘/);
  });

});
