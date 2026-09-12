// @vitest-environment jsdom

import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { CurrentGameResultDto, ForegroundEngineSnapshotDto, GameDto } from "./domain/types";
import { defaultAppPreferences } from "./domain/preferences";

const listeners: {
  onSnapshot?: (snapshot: ForegroundEngineSnapshotDto) => void;
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
  foregroundEngineContinuousAction: vi.fn(),
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
  subscribeForegroundEngine: vi.fn(),
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

let root: Root | null = null;
let snapshotRevision = 2;

beforeEach(() => {
  globalThis.IS_REACT_ACT_ENVIRONMENT = true;
  snapshotRevision = 2;
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
  currentGameFixture.mockResolvedValue(initialGame);
  backend.projectCurrentGameMainline.mockResolvedValue(initialProjection);
  backend.loadEngineProfilesSettings.mockResolvedValue({
    selected_profile_id: "profile-1",
    autoload_profile_id: null,
    profiles: [savedProfile]
  });
  backend.saveEngineProfilesSettings.mockImplementation(async (settings) => settings);
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
  backend.foregroundEngineContinuousAction.mockResolvedValue({
    ...defaultAppPreferences,
    continuousAnalysisEnabled: false
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
      },
      continuous: { enabled: true, phase: "waiting" }
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
    expect(host.querySelector(".nav-progress")?.textContent).toContain("此手分析中");
    const ownership = Array.from({ length: 81 }, () => 0.25);
    const policy = Array.from({ length: 81 }, () => 0);
    policy[3 + 3 * 9] = 0.4;
    await act(async () => {
      listeners.onJob?.({
        run_id: "run-1",
        job_id: "job-1",
        lane: "selected_node",
        mode: "finite",
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
        mode: "finite",
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

function continuousFrame(visits: number) {
  return {
    job_id: "job-continuous",
    turn: 0,
    visits,
    winrate_black: 0.55 + visits / 1000,
    score_mean_black: visits / 10,
    candidates: [{
      vertex: { point: { x: 3, y: 3 } } as const,
      visits,
      winrate_black: 0.56,
      score_mean_black: visits / 10,
      pv: [{ point: { x: 3, y: 3 } } as const]
    }],
    ownership: Array.from({ length: 81 }, () => 0.1),
    policy: Array.from({ length: 81 }, (_, index) => index === 30 ? 0.4 : 0)
  };
}

function emitContinuousSnapshot(phase: ForegroundEngineSnapshotDto["continuous"]["phase"], options: {
  enabled?: boolean;
  job?: boolean;
  jobMode?: "continuous" | "finite";
  jobState?: "queued" | "searching" | "stopping" | "time_limited" | "visits_limited";
  wholeGameJob?: boolean;
  lifecycle?: ForegroundEngineSnapshotDto["lifecycle"];
} = {}) {
  snapshotRevision += 1;
  listeners.onSnapshot?.({
    revision: snapshotRevision,
    lifecycle: options.lifecycle ?? {
      state: "ready",
      run: {
        run_id: "run-1", profile_id: "profile-1", adapter_kind: "kata_go_analysis",
        profile_snapshot: savedProfile.profile,
        capability_snapshot: { adapter_kind: "kata_go_analysis", selected_node_analysis: true, whole_game_analysis: true, protocol_cancel: true }
      }
    },
    continuous: { enabled: options.enabled ?? true, phase },
    selected_node_job: options.job === false ? null : {
      run_id: "run-1", job_id: "job-continuous", lane: "selected_node", mode: options.jobMode ?? "continuous",
      state: options.jobState ?? (phase === "searching" ? "searching" : phase === "stopping" ? "stopping" : phase === "time_limited" || phase === "visits_limited" ? phase : "queued"),
      generation: 1, node_path: { indices: [] }
    },
    whole_game_job: options.wholeGameJob ? {
      run_id: "run-1", job_id: "job-wg", lane: "whole_game", mode: "finite", state: "searching",
      generation: 1, node_path: { indices: [] }
    } : undefined
  });
}

async function publishContinuousProgress(visits: number, phase: "searching" | "time_limited" | "visits_limited" = "searching") {
  const frame = continuousFrame(visits);
  await act(async () => {
    if (phase === "searching") emitContinuousSnapshot(phase);
    listeners.onJob?.({
      run_id: "run-1", job_id: "job-continuous", lane: "selected_node", mode: "continuous",
      generation: 1, node_path: { indices: [] }, outcome: phase === "searching" ? "progress" : phase,
      frame,
      current_game: {
        ...initialGame,
        snapshot_seq: visits,
        dirty: true,
        snapshot: { ...initialGame.snapshot, primary_analysis: { ...frame, job_id: "00000000-0000-0000-0000-000000000000" } }
      }
    });
    if (phase !== "searching") emitContinuousSnapshot(phase);
    await Promise.resolve();
  });
}

describe("authoritative finite selected-node completion", () => {
  it("keeps an early terminal result when the finite invoke resolves after Started, Completed, and the empty snapshot", async () => {
    let resolveStart!: (job: {
      run_id: string; job_id: string; lane: "selected_node"; mode: "finite"; state: "queued";
      generation: number; node_path: { indices: number[] };
    }) => void;
    const pendingStart = new Promise<Parameters<typeof resolveStart>[0]>((resolve) => { resolveStart = resolve; });
    backend.startSelectedNodeAnalysis.mockReturnValueOnce(pendingStart);
    const host = await renderApp();
    await readyEngine(host);
    act(() => buttonNamed(host, "分析当前节点").click());

    const frame = { ...continuousFrame(64), job_id: "job-finite" };
    await act(async () => {
      listeners.onJob?.({
        run_id: "run-1", job_id: "job-finite", lane: "selected_node", mode: "finite",
        generation: 1, node_path: { indices: [] }, outcome: "started"
      });
      listeners.onJob?.({
        run_id: "run-1", job_id: "job-finite", lane: "selected_node", mode: "finite",
        generation: 1, node_path: { indices: [] }, outcome: "completed", frame
      });
      emitContinuousSnapshot("waiting", { job: false });
      resolveStart({
        run_id: "run-1", job_id: "job-finite", lane: "selected_node", mode: "finite", state: "queued",
        generation: 1, node_path: { indices: [] }
      });
      await pendingStart;
      await Promise.resolve();
    });

    expect(host.querySelector(".cand-visits")?.textContent).toBe("64");
    expect(host.textContent).not.toContain("此手分析中");
    expect(Array.from(host.querySelectorAll("button")).find((button) => button.textContent === "取消此手")).toBeUndefined();
  });

  it("keeps a fresh continuous owner and frame when the finite invoke and classification resolve late", async () => {
    let resolveStart!: (job: {
      run_id: string; job_id: string; lane: "selected_node"; mode: "finite"; state: "queued";
      generation: number; node_path: { indices: number[] };
    }) => void;
    const pendingStart = new Promise<Parameters<typeof resolveStart>[0]>((resolve) => { resolveStart = resolve; });
    let resolveClassification!: (markers: []) => void;
    const pendingClassification = new Promise<[]>((resolve) => { resolveClassification = resolve; });
    backend.startSelectedNodeAnalysis.mockReturnValueOnce(pendingStart);
    backend.classifyProblems.mockReturnValueOnce(pendingClassification);
    const host = await renderApp();
    await readyEngine(host);
    act(() => buttonNamed(host, "分析当前节点").click());

    const finiteFrame = { ...continuousFrame(64), job_id: "job-finite" };
    await act(async () => {
      listeners.onJob?.({
        run_id: "run-1", job_id: "job-finite", lane: "selected_node", mode: "finite",
        generation: 1, node_path: { indices: [] }, outcome: "started"
      });
      listeners.onJob?.({
        run_id: "run-1", job_id: "job-finite", lane: "selected_node", mode: "finite",
        generation: 1, node_path: { indices: [] }, outcome: "completed", frame: finiteFrame
      });
      await Promise.resolve();
    });

    const continuous = continuousFrame(91);
    await act(async () => {
      listeners.onJob?.({
        run_id: "run-1", job_id: "job-continuous", lane: "selected_node", mode: "continuous",
        generation: 1, node_path: { indices: [] }, outcome: "started"
      });
      emitContinuousSnapshot("searching");
      listeners.onJob?.({
        run_id: "run-1", job_id: "job-continuous", lane: "selected_node", mode: "continuous",
        generation: 1, node_path: { indices: [] }, outcome: "progress", frame: continuous,
        current_game: {
          ...initialGame,
          dirty: true,
          snapshot: { ...initialGame.snapshot, primary_analysis: { ...continuous, job_id: "00000000-0000-0000-0000-000000000000" } }
        }
      });
      await Promise.resolve();
    });
    expect(host.querySelector(".cand-visits")?.textContent).toBe("91");
    expect(host.textContent).toContain("连续分析：搜索中");

    await act(async () => {
      resolveStart({
        run_id: "run-1", job_id: "job-finite", lane: "selected_node", mode: "finite", state: "queued",
        generation: 1, node_path: { indices: [] }
      });
      resolveClassification([]);
      await Promise.all([pendingStart, pendingClassification]);
      await Promise.resolve();
    });

    expect(host.querySelector(".cand-visits")?.textContent).toBe("91");
    expect(host.textContent).toContain("连续分析：搜索中");
    expect(buttonNamed(host, "取消此手")).toBeInstanceOf(HTMLButtonElement);
    expect(backend.startSelectedNodeAnalysis).toHaveBeenCalledTimes(1);
    expect(backend.foregroundEngineContinuousAction).not.toHaveBeenCalled();
  });
});

describe("authoritative continuous selected-node analysis", () => {
  it("keeps newer comments and saved state when the same continuous job delivers an older snapshot", async () => {
    const host = await renderApp();
    await readyEngine(host);
    await publishContinuousProgress(12);
    const commented: CurrentGameResultDto = {
      ...initialGame, snapshot_seq: 13, dirty: true,
      snapshot: { ...initialGame.snapshot, personal_comment: "keep this note", primary_analysis: continuousFrame(12) }
    };
    backend.setCurrentGamePersonalComment.mockResolvedValue(commented);
    const editor = host.querySelector('textarea[aria-label="个人评论"]') as HTMLTextAreaElement;
    act(() => {
      editor.focus();
      Object.getOwnPropertyDescriptor(HTMLTextAreaElement.prototype, "value")?.set?.call(editor, "keep this note");
      editor.dispatchEvent(new Event("input", { bubbles: true }));
    });
    await act(async () => {
      editor.blur();
      await backend.setCurrentGamePersonalComment.mock.results.at(-1)?.value;
    });
    backend.saveCurrentGame.mockResolvedValue({ ...commented, snapshot_seq: 14, dirty: false, native_path: "/tmp/comment.sgf" });
    await act(async () => {
      (host.querySelector('button[aria-label="保存"]') as HTMLButtonElement).click();
      await backend.saveCurrentGame.mock.results.at(-1)?.value;
    });
    await publishContinuousProgress(12);
    expect(editor.value).toBe("keep this note");
    expect(host.querySelector(".doc-name")?.textContent).not.toContain("*");
    expect(backend.cancelSelectedNodeAnalysis).not.toHaveBeenCalled();
    expect(host.querySelector(".cand-visits")?.textContent).toBe("12");
  });

  it("adopts an automatic job without issuing browser-side work and renders waiting, searching, and limited phases", async () => {
    const host = await renderApp();
    await readyEngine(host);
    expect(buttonNamed(host, "停止连续分析")).toBeInstanceOf(HTMLButtonElement);
    expect(host.textContent).toContain("连续分析：等待可用引擎");
    expect(backend.foregroundEngineContinuousAction).not.toHaveBeenCalled();

    act(() => {
      emitContinuousSnapshot("queued");
      listeners.onJob?.({
        run_id: "run-1", job_id: "job-continuous", lane: "selected_node", mode: "continuous",
        generation: 1, node_path: { indices: [] }, outcome: "started"
      });
    });
    await publishContinuousProgress(12);
    expect(host.querySelector(".cand-visits")?.textContent).toBe("12");
    expect(host.textContent).toContain("连续分析：搜索中");
    await publishContinuousProgress(45, "time_limited");
    expect(host.querySelector(".cand-visits")?.textContent).toBe("45");
    expect(buttonNamed(host, "继续连续分析").disabled).toBe(false);
    act(() => emitContinuousSnapshot("empty_board", { job: false }));
    expect(host.textContent).toContain("空棋盘停止已启用");
    expect(buttonNamed(host, "停止连续分析").disabled).toBe(false);
  });

  it.each(["time_limited", "visits_limited"] as const)("adopts a first terminal frame before its %s snapshot", async (phase) => {
    const host = await renderApp();
    await readyEngine(host);
    act(() => emitContinuousSnapshot("queued"));
    await publishContinuousProgress(64, phase);
    expect(host.querySelector(".cand-visits")?.textContent).toBe("64");
    expect(buttonNamed(host, "继续连续分析").disabled).toBe(false);
    expect(backend.foregroundEngineContinuousAction).not.toHaveBeenCalled();
  });

  it("routes focus-safe Space, visible, and menu actions through the same contextual command", async () => {
    const host = await renderApp();
    await readyEngine(host);
    const textarea = document.createElement("textarea");
    host.append(textarea);
    act(() => textarea.dispatchEvent(new KeyboardEvent("keydown", { key: " ", bubbles: true })));
    expect(backend.foregroundEngineContinuousAction).not.toHaveBeenCalled();

    await act(async () => {
      window.dispatchEvent(new KeyboardEvent("keydown", { key: " ", bubbles: true }));
      await backend.foregroundEngineContinuousAction.mock.results.at(-1)?.value;
    });
    expect(backend.foregroundEngineContinuousAction).toHaveBeenCalledTimes(1);
    act(() => emitContinuousSnapshot("off", { enabled: false, job: false }));
    await act(async () => {
      buttonNamed(host, "开始连续分析").click();
      await backend.foregroundEngineContinuousAction.mock.results.at(-1)?.value;
    });
    act(() => buttonNamed(host, "分析").click());
    await act(async () => {
      buttonNamed(host, "开始连续分析").click();
      await backend.foregroundEngineContinuousAction.mock.results.at(-1)?.value;
    });
    expect(backend.foregroundEngineContinuousAction).toHaveBeenCalledTimes(3);
    expect(backend.cancelSelectedNodeAnalysis).not.toHaveBeenCalled();
  });

  it("keeps authoritative holds through stale terminal events and blocks departure and Run failures", async () => {
    const host = await renderApp();
    await readyEngine(host);
    act(() => emitContinuousSnapshot("safety_hold", { job: false }));
    act(() => listeners.onJob?.({
      run_id: "run-1", job_id: "job-continuous", lane: "selected_node", mode: "continuous",
      generation: 1, node_path: { indices: [] }, outcome: "cancelled"
    }));
    expect(buttonNamed(host, "继续连续分析")).toBeInstanceOf(HTMLButtonElement);

    await act(async () => {
      buttonNamed(host, "继续连续分析").click();
      await backend.foregroundEngineContinuousAction.mock.results.at(-1)?.value;
    });
    expect(backend.foregroundEngineContinuousAction).toHaveBeenCalledTimes(1);
    act(() => emitContinuousSnapshot("departing", { job: false }));
    expect(buttonNamed(host, "停止连续分析").disabled).toBe(true);

    act(() => emitContinuousSnapshot("error", {
      job: false,
      lifecycle: {
        state: "error",
        run: { run_id: "run-1", profile_id: "profile-1", adapter_kind: "kata_go_analysis", profile_snapshot: savedProfile.profile },
        failure: { operation: "job", run_id: "run-1", kind: "timeout", message: "target final never arrived" }
      }
    }));
    const blocked = buttonNamed(host, "继续连续分析");
    expect(blocked.disabled).toBe(true);
    expect(blocked.title).toMatch(/重启引擎/);
  });

  it("preserves the rendered intent when Start, Stop, or finite-owner preference writes fail", async () => {
    const host = await renderApp();
    await readyEngine(host);
    for (const state of [
      { phase: "off" as const, enabled: false, label: "开始连续分析" },
      { phase: "searching" as const, enabled: true, label: "停止连续分析" },
      { phase: "finite" as const, enabled: true, label: "停止连续分析" }
    ]) {
      act(() => emitContinuousSnapshot(state.phase, { enabled: state.enabled, job: state.phase === "searching" }));
      backend.foregroundEngineContinuousAction.mockRejectedValueOnce(new Error("disk full"));
      await act(async () => {
        buttonNamed(host, state.label).click();
        await Promise.resolve(backend.foregroundEngineContinuousAction.mock.results.at(-1)?.value).catch(() => undefined);
      });
      expect(buttonNamed(host, state.label)).toBeInstanceOf(HTMLButtonElement);
      expect(host.textContent).toContain("disk full");
    }
  });

  it("does not issue commands for repeated Ready snapshots", async () => {
    const host = await renderApp();
    await readyEngine(host);
    act(() => {
      emitContinuousSnapshot("waiting", { job: false });
      emitContinuousSnapshot("waiting", { job: false });
    });
    expect(host.textContent).toContain("连续分析：等待可用引擎");
    expect(backend.foregroundEngineContinuousAction).not.toHaveBeenCalled();
    expect(backend.startSelectedNodeAnalysis).not.toHaveBeenCalled();
  });

  it("changes future intent around an active finite job without cancelling it", async () => {
    const host = await renderApp();
    await readyEngine(host);
    backend.foregroundEngineContinuousAction
      .mockResolvedValueOnce({ ...defaultAppPreferences, continuousAnalysisEnabled: false })
      .mockResolvedValueOnce({ ...defaultAppPreferences, continuousAnalysisEnabled: true });
    act(() => emitContinuousSnapshot("finite", { enabled: true, jobMode: "finite", jobState: "searching" }));

    await act(async () => {
      buttonNamed(host, "停止连续分析").click();
      await backend.foregroundEngineContinuousAction.mock.results.at(-1)?.value;
    });
    act(() => emitContinuousSnapshot("finite", { enabled: false, jobMode: "finite", jobState: "searching" }));
    expect(buttonNamed(host, "开始连续分析")).toBeInstanceOf(HTMLButtonElement);
    expect(host.querySelector(".nav-progress")?.textContent).toContain("此手分析中");

    await act(async () => {
      buttonNamed(host, "开始连续分析").click();
      await backend.foregroundEngineContinuousAction.mock.results.at(-1)?.value;
    });
    act(() => emitContinuousSnapshot("finite", { enabled: true, jobMode: "finite", jobState: "searching" }));
    expect(buttonNamed(host, "停止连续分析")).toBeInstanceOf(HTMLButtonElement);
    expect(backend.cancelSelectedNodeAnalysis).not.toHaveBeenCalled();
    expect(backend.foregroundEngineContinuousAction).toHaveBeenCalledTimes(2);
  });

  it("blocks button, menu, and Space while target cancellation is unfinished", async () => {
    const host = await renderApp();
    await readyEngine(host);
    act(() => emitContinuousSnapshot("stopping", { jobState: "stopping" }));
    const primary = buttonNamed(host, "停止连续分析");
    expect(primary.disabled).toBe(true);
    act(() => primary.click());
    act(() => window.dispatchEvent(new KeyboardEvent("keydown", { key: " ", bubbles: true })));
    act(() => buttonNamed(host, "分析").click());
    const menuPrimary = Array.from(host.querySelectorAll('[role="menuitem"]')).find((item) => item.textContent === "停止连续分析") as HTMLButtonElement;
    expect(menuPrimary.disabled).toBe(true);
    act(() => menuPrimary.click());
    expect(backend.foregroundEngineContinuousAction).not.toHaveBeenCalled();
  });

  it("keeps A's admitted presentation while B is switching and after Ready A rollback", async () => {
    const host = await renderApp();
    await readyEngine(host);
    await publishContinuousProgress(37);
    const runA = {
      run_id: "run-1", profile_id: "profile-1", adapter_kind: "kata_go_analysis" as const,
      profile_snapshot: savedProfile.profile,
      capability_snapshot: { adapter_kind: "kata_go_analysis" as const, selected_node_analysis: true, whole_game_analysis: true, protocol_cancel: true }
    };
    act(() => emitContinuousSnapshot("searching", {
      lifecycle: {
        state: "switching",
        primary: runA,
        candidate: { ...runA, run_id: "run-b", profile_id: "profile-b" },
        switch_id: "switch-b"
      }
    }));
    expect(host.querySelector(".cand-visits")?.textContent).toBe("37");
    act(() => emitContinuousSnapshot("searching", { lifecycle: { state: "ready", run: runA } }));
    expect(host.querySelector(".cand-visits")?.textContent).toBe("37");
    expect(backend.foregroundEngineContinuousAction).not.toHaveBeenCalled();
  });

  it("keeps the retained game stopped after an aborted departure until explicit Resume", async () => {
    const retained = {
      ...initialGame,
      dirty: true,
      snapshot: { ...initialGame.snapshot, primary_analysis: { ...continuousFrame(28), job_id: "00000000-0000-0000-0000-000000000000" } }
    };
    currentGameFixture.mockResolvedValue(retained);
    const host = await renderApp();
    await act(async () => {
      await backend.prepareDocumentReplacement.mock.results[0]?.value;
      await Promise.resolve();
      await backend.resolveDocumentReplacement.mock.results[0]?.value;
    });
    backend.prepareDocumentReplacement.mockResolvedValueOnce({ status: "needs_decision", departure_id: 9 });
    backend.resolveDocumentReplacement.mockResolvedValueOnce({
      committed: false,
      analysis_stopped: true,
      current: retained,
      message: "save failed; document retained"
    });
    await readyEngine(host);
    await publishContinuousProgress(28);

    act(() => (host.querySelector('button[aria-label="新建"]') as HTMLButtonElement).click());
    await act(async () => {
      await backend.prepareDocumentReplacement.mock.results.at(-1)?.value;
      await Promise.resolve();
    });
    expect(host.querySelector('[role="dialog"][aria-label="保存当前棋谱"]')).toBeInstanceOf(HTMLElement);
    await act(async () => {
      (host.querySelector('button[aria-label="Save"]') as HTMLButtonElement).click();
      await backend.resolveDocumentReplacement.mock.results.at(-1)?.value;
    });
    act(() => emitContinuousSnapshot("safety_hold", { job: false }));
    expect(host.querySelector(".cand-visits")?.textContent).toBe("28");
    expect(host.textContent).toContain("save failed; document retained");
    await act(async () => {
      buttonNamed(host, "继续连续分析").click();
      await backend.foregroundEngineContinuousAction.mock.results.at(-1)?.value;
    });
    expect(backend.foregroundEngineContinuousAction).toHaveBeenCalledTimes(1);
  });

  it("does not replace a newer progress snapshot when an older event arrives later", async () => {
    const host = await renderApp();
    await readyEngine(host);
    await publishContinuousProgress(37);
    const stale = continuousFrame(12);
    act(() => listeners.onJob?.({
      run_id: "run-old", job_id: "job-continuous", lane: "selected_node", mode: "continuous",
      generation: 1, node_path: { indices: [] }, outcome: "progress", frame: stale,
      current_game: { ...initialGame, dirty: true, snapshot: { ...initialGame.snapshot, primary_analysis: stale } }
    }));
    expect(host.querySelector(".cand-visits")?.textContent).toBe("37");
  });

  it("presents the latest accepted frame across continuous and whole-game lanes regardless of visits", async () => {
    const host = await renderApp();
    await readyEngine(host);
    await publishContinuousProgress(80);
    expect(host.querySelector(".cand-visits")?.textContent).toBe("80");

    await act(async () => {
      buttonNamed(host, "分析第一子主线").click();
      await backend.startKataGoGameAnalysis.mock.results.at(-1)?.value;
    });
    const wholeGameFrame = { ...continuousFrame(44), job_id: "job-wg" };
    await act(async () => {
      listeners.onJob?.({
        run_id: "run-1", job_id: "job-wg", lane: "whole_game", mode: "finite",
        generation: 1, node_path: { indices: [] }, outcome: "progress",
        completed: 1, expected: 1, remaining: 0, frame: wholeGameFrame
      });
      await Promise.resolve();
    });
    expect(host.querySelector(".cand-visits")?.textContent).toBe("44");

    const laterContinuousFrame = continuousFrame(12);
    await act(async () => {
      emitContinuousSnapshot("searching", { wholeGameJob: true });
      listeners.onJob?.({
        run_id: "run-1", job_id: "job-continuous", lane: "selected_node", mode: "continuous",
        generation: 1, node_path: { indices: [] }, outcome: "progress", frame: laterContinuousFrame,
        current_game: {
          ...initialGame,
          snapshot_seq: 81,
          dirty: true,
          snapshot: { ...initialGame.snapshot, primary_analysis: { ...laterContinuousFrame, job_id: "00000000-0000-0000-0000-000000000000" } }
        }
      });
      await Promise.resolve();
    });
    expect(host.querySelector(".cand-visits")?.textContent).toBe("12");

    const cancelledFrame = { ...continuousFrame(99), job_id: "job-wg" };
    act(() => listeners.onJob?.({
      run_id: "run-1", job_id: "job-wg", lane: "whole_game", mode: "finite",
      generation: 1, node_path: { indices: [] }, outcome: "cancelled", frame: cancelledFrame,
      completed: 1, expected: 1, remaining: 0,
      current_game: {
        ...initialGame,
        dirty: true,
        snapshot: { ...initialGame.snapshot, primary_analysis: cancelledFrame }
      }
    }));

    expect(host.querySelector(".cand-visits")?.textContent).toBe("12");
    expect(buttonNamed(host, "取消整局")).toBeUndefined();
  });
});
