// @vitest-environment jsdom

import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { AnalysisScopeDto, AnalysisScopePreviewDto, AnalysisStageConditionsDto, AnalysisTaskDto, CurrentGameResultDto, ForegroundEngineSnapshotDto, GameDto } from "./domain/types";
import { defaultAppPreferences, type AppPreferences } from "./domain/preferences";

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
const preferencesApi = vi.hoisted(() => ({
  loadAppPreferences: vi.fn((): Promise<{ preferences: AppPreferences }> => Promise.reject(new Error("preferences unavailable in test"))),
  saveAppPreferences: vi.fn(async (preferences: AppPreferences) => preferences)
}));
const taskRuntime = vi.hoisted(() => ({ snapshot: null as AnalysisTaskDto | null }));
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
  previewAnalysisScope: vi.fn(),
  startAnalysisTask: vi.fn(),
  pauseAnalysisTask: vi.fn(),
  continueAnalysisTask: vi.fn(),
  analysisTaskSnapshot: vi.fn(),
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

vi.mock("./api/preferences", () => preferencesApi);

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
  taskRuntime.snapshot = null;
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
  backend.previewAnalysisScope.mockImplementation(async ({ generation, scope }: { generation: number; scope: AnalysisScopeDto }) => ({
    generation,
    scope,
    targets: [
      { node_path: { indices: [] }, move_number: 0, to_play: "black" },
      { node_path: { indices: [0] }, move_number: 1, to_play: "white" }
    ]
  }));
  backend.startAnalysisTask.mockImplementation(async ({ runId, preview, conditions }: { runId: string; preview: AnalysisScopePreviewDto; conditions: AnalysisStageConditionsDto }) => {
    const task: AnalysisTaskDto = {
      task_id: "task-1",
      run_id: runId,
      job_id: "job-task-1",
      generation: preview.generation,
      scope: preview.scope,
      stage: "single_stage",
      conditions,
      requested: preview.targets.map((target) => target.node_path),
      completed: [preview.targets[0].node_path],
      state: "searching",
      reason: null,
      ending_conditions: []
    };
    taskRuntime.snapshot = task;
    return task;
  });
  backend.pauseAnalysisTask.mockImplementation(async ({ runId, taskId }: { runId: string; taskId: string }) => {
    const task = taskRuntime.snapshot;
    if (!task || task.run_id !== runId || task.task_id !== taskId) throw new Error("Task not found");
    taskRuntime.snapshot = { ...task, state: "pausing" };
    return taskRuntime.snapshot;
  });
  backend.continueAnalysisTask.mockImplementation(async ({ runId, taskId }: { runId: string; taskId: string }) => {
    const task = taskRuntime.snapshot;
    if (!task || task.run_id !== runId || task.task_id !== taskId) throw new Error("Task not found");
    taskRuntime.snapshot = { ...task, job_id: "job-task-2", state: "searching" };
    return taskRuntime.snapshot;
  });
  backend.analysisTaskSnapshot.mockImplementation(async () => taskRuntime.snapshot);
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

function inputNamed(host: HTMLElement, label: string) {
  const input = host.querySelector(`input[aria-label="${label}"]`);
  if (!(input instanceof HTMLInputElement)) throw new Error(`Missing input: ${label}`);
  return input;
}

async function changeNumber(input: HTMLInputElement, value: string) {
  await act(async () => {
    Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")!.set!.call(input, value);
    input.dispatchEvent(new Event("input", { bubbles: true }));
  });
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
    expect(host.querySelector('button[aria-label="闪电分析"]')).toBeInstanceOf(HTMLButtonElement);
    expect((host.querySelector('button[aria-label="闪电分析"]') as HTMLButtonElement).disabled).toBe(true);

    openAnalyzeMenu(host);
    const lightning = buttonNamed(host, "试复盘 / 闪电分析(Ctrl+B)");
    expect(lightning.disabled).toBe(true);
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
  it("serializes task preset persistence with a concurrent display edit", async () => {
    const host = await renderApp();
    await readyEngine(host);
    const visits = host.querySelector('input[aria-label="Total visits"]') as HTMLInputElement;
    await act(async () => {
      Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")!.set!.call(visits, "32");
      visits.dispatchEvent(new Event("input", { bubbles: true }));
    });
    await act(async () => { buttonNamed(host, "Preview scope").click(); });
    let releaseSave!: (preferences: AppPreferences) => void;
    preferencesApi.saveAppPreferences.mockImplementationOnce(() => new Promise<AppPreferences>((resolve) => { releaseSave = resolve; }));
    await act(async () => { buttonNamed(host, "Start task").click(); });
    expect(backend.startAnalysisTask).not.toHaveBeenCalled();
    await act(async () => { buttonNamed(host, "显示").click(); });
    const candidates = Array.from(host.querySelectorAll('[role="menuitemcheckbox"]')).find((item) => item.textContent?.includes("候选")) as HTMLElement;
    await act(async () => { candidates.click(); });
    expect(preferencesApi.saveAppPreferences).toHaveBeenCalledTimes(1);
    await act(async () => {
      releaseSave(preferencesApi.saveAppPreferences.mock.calls[0][0]);
      await Promise.resolve();
      await Promise.resolve();
    });
    expect(preferencesApi.saveAppPreferences).toHaveBeenLastCalledWith(expect.objectContaining({
      taskConditions: expect.objectContaining({ total_visits: { enabled: true, value: 32 } }),
      showCandidates: false
    }));
    expect(backend.startAnalysisTask).toHaveBeenCalledTimes(1);
    await act(async () => { buttonNamed(host, "显示").click(); });
    const savedCandidates = Array.from(host.querySelectorAll('[role="menuitemcheckbox"]')).find((item) => item.textContent?.includes("候选"));
    expect(savedCandidates?.getAttribute("aria-checked")).toBe("false");
  });
  it("does not leak a failed task preset through a concurrent preference save", async () => {
    const host = await renderApp();
    await readyEngine(host);
    await changeNumber(inputNamed(host, "Total visits"), "32");
    await act(async () => { buttonNamed(host, "Preview scope").click(); });
    let rejectTaskSave!: (error: Error) => void;
    preferencesApi.saveAppPreferences.mockImplementationOnce(() => new Promise<AppPreferences>((_, reject) => {
      rejectTaskSave = reject;
    }));
    await act(async () => { buttonNamed(host, "Start task").click(); });
    await act(async () => { buttonNamed(host, "显示").click(); });
    const candidates = Array.from(host.querySelectorAll('[role="menuitemcheckbox"]'))
      .find((item) => item.textContent?.includes("候选")) as HTMLElement;
    await act(async () => { candidates.click(); });

    await act(async () => {
      rejectTaskSave(new Error("task preset write failed"));
      await preferencesApi.saveAppPreferences.mock.results[0]?.value.catch(() => undefined);
      await Promise.resolve();
      await Promise.resolve();
    });
    expect(backend.startAnalysisTask).not.toHaveBeenCalled();
    expect(preferencesApi.saveAppPreferences).toHaveBeenCalledTimes(2);
    expect(preferencesApi.saveAppPreferences.mock.calls[1][0]).toEqual(expect.objectContaining({
      taskConditions: defaultAppPreferences.taskConditions,
      showCandidates: false
    }));
  });

  it("validates and persists independent OR task conditions before starting", async () => {
    const host = await renderApp();
    await readyEngine(host);
    await act(async () => { buttonNamed(host, "Preview scope").click(); });

    const timeEnabled = inputNamed(host, "Enable search time");
    const totalEnabled = inputNamed(host, "Enable total visits");
    const leadingEnabled = inputNamed(host, "Enable leading candidate visits");
    const time = inputNamed(host, "Search time seconds");
    const total = inputNamed(host, "Total visits");
    const leading = inputNamed(host, "Leading candidate visits");
    expect([time.value, total.value, leading.value]).toEqual(["10", "800", "500"]);

    await act(async () => { totalEnabled.click(); });
    await act(async () => { buttonNamed(host, "Start task").click(); });
    expect(preferencesApi.saveAppPreferences).not.toHaveBeenCalled();
    expect(backend.startAnalysisTask).not.toHaveBeenCalled();

    await act(async () => { timeEnabled.click(); leadingEnabled.click(); });
    await changeNumber(time, "1.5");
    await act(async () => { buttonNamed(host, "Preview scope").click(); });
    await act(async () => { buttonNamed(host, "Start task").click(); });
    expect(preferencesApi.saveAppPreferences).not.toHaveBeenCalled();

    await changeNumber(time, "12");
    await changeNumber(total, "41");
    await changeNumber(leading, "7");
    await act(async () => { buttonNamed(host, "Preview scope").click(); });
    await act(async () => {
      buttonNamed(host, "Start task").click();
      await Promise.resolve();
      await Promise.resolve();
    });
    const conditions = {
      time_seconds: { enabled: true, value: 12 },
      total_visits: { enabled: false, value: 41 },
      leading_candidate_visits: { enabled: true, value: 7 }
    };
    expect(preferencesApi.saveAppPreferences).toHaveBeenCalledWith(expect.objectContaining({ taskConditions: conditions }));
    expect(backend.startAnalysisTask).toHaveBeenCalledWith(expect.objectContaining({ conditions }));
    expect(host.querySelector('[data-analysis-task-state="searching"]')?.textContent)
      .toContain("conditions 12s OR 7 leading candidate visits");

    await changeNumber(time, "99");
    expect(host.querySelector('[data-analysis-task-state="searching"]')?.textContent)
      .toContain("conditions 12s OR 7 leading candidate visits");
  });

  it("restores the durable task preset into the rendered controls", async () => {
    preferencesApi.loadAppPreferences.mockResolvedValueOnce({
      preferences: {
        ...defaultAppPreferences,
        defaultMaxVisits: 999,
        taskConditions: {
          time_seconds: { enabled: true, value: 15 },
          total_visits: { enabled: false, value: 64 },
          leading_candidate_visits: { enabled: true, value: 8 }
        }
      }
    });
    const host = await renderApp();
    await act(async () => { await preferencesApi.loadAppPreferences.mock.results[0]?.value; });
    expect(inputNamed(host, "Enable search time").checked).toBe(true);
    expect(inputNamed(host, "Search time seconds").value).toBe("15");
    expect(inputNamed(host, "Enable total visits").checked).toBe(false);
    expect(inputNamed(host, "Total visits").value).toBe("64");
    expect(inputNamed(host, "Enable leading candidate visits").checked).toBe(true);
    expect(inputNamed(host, "Leading candidate visits").value).toBe("8");
  });

  it("loads a durable task preset without overwriting an in-progress draft edit", async () => {
    let resolveLoad!: (value: { preferences: AppPreferences }) => void;
    const load = new Promise<{ preferences: AppPreferences }>((resolve) => { resolveLoad = resolve; });
    preferencesApi.loadAppPreferences.mockReturnValueOnce(load);
    const host = await renderApp();
    const time = inputNamed(host, "Search time seconds");
    await changeNumber(time, "27");

    await act(async () => {
      resolveLoad({
        preferences: {
          ...defaultAppPreferences,
          defaultMaxVisits: 999,
          taskConditions: {
            time_seconds: { enabled: true, value: 15 },
            total_visits: { enabled: false, value: 64 },
            leading_candidate_visits: { enabled: true, value: 8 }
          }
        }
      });
      await load;
    });
    expect(time.value).toBe("27");
    expect(inputNamed(host, "Total visits").value).toBe("800");
    expect(inputNamed(host, "Leading candidate visits").value).toBe("500");
  });


  it("cancels the newly started legacy task before its snapshot arrives", async () => {
    const host = await renderApp();
    await readyEngine(host);
    await act(async () => {
      window.dispatchEvent(new KeyboardEvent("keydown", { key: "b", ctrlKey: true, bubbles: true }));
      await Promise.resolve();
      await Promise.resolve();
    });
    taskRuntime.snapshot = { ...taskRuntime.snapshot!, state: "completed" };
    await act(async () => { listeners.onJob?.({ ...taskRuntime.snapshot, lane: "whole_game", mode: "finite", node_path: { indices: [] }, outcome: "completed" }); });
    backend.analysisTaskSnapshot.mockImplementationOnce(() => new Promise(() => {}));
    await act(async () => { buttonNamed(host, "分析第一子主线").click(); });
    backend.cancelKataGoAnalysis.mockImplementationOnce(async (_runId: string, jobId: string) => {
      if (jobId !== "job-wg") throw new Error("Previous task already ended");
      taskRuntime.snapshot = { ...taskRuntime.snapshot!, task_id: "legacy-task", job_id: jobId, state: "cancelled" };
    });
    await act(async () => { buttonNamed(host, "取消整局").click(); });
    expect(host.querySelector('[data-analysis-task-state="cancelled"]')).not.toBeNull();
    expect(host.textContent).not.toContain("Previous task already ended");
  });

  it("keeps Cancel terminal when an older task snapshot arrives late", async () => {
    const host = await renderApp();
    await readyEngine(host);
    await act(async () => {
      window.dispatchEvent(new KeyboardEvent("keydown", { key: "b", ctrlKey: true, bubbles: true }));
      await Promise.resolve();
      await Promise.resolve();
    });
    const searching = taskRuntime.snapshot!;
    let releaseSnapshot!: (task: AnalysisTaskDto) => void;
    backend.analysisTaskSnapshot.mockImplementationOnce(() => new Promise<AnalysisTaskDto>((resolve) => {
      releaseSnapshot = resolve;
    }));
    await act(async () => {
      listeners.onJob?.({ run_id: searching.run_id, job_id: searching.job_id, lane: "whole_game", mode: "finite", generation: 1, node_path: { indices: [] }, outcome: "started" });
    });
    backend.cancelKataGoAnalysis.mockImplementationOnce(async () => {
      taskRuntime.snapshot = { ...searching, state: "cancelled" };
    });
    await act(async () => {
      buttonNamed(host, "Cancel task").click();
      await Promise.resolve();
      await Promise.resolve();
    });
    expect(buttonNamed(host, "Cancel task").disabled).toBe(true);
    await act(async () => { releaseSnapshot(searching); });
    expect(buttonNamed(host, "Cancel task").disabled).toBe(true);
    expect(host.querySelector('[data-analysis-task-state="cancelled"]')).not.toBeNull();
  });

  it("fences stale snapshots and frames across Pause and Continue without duplicate commands", async () => {
    const host = await renderApp();
    await readyEngine(host);
    await act(async () => { buttonNamed(host, "Preview scope").click(); });
    await act(async () => { buttonNamed(host, "Start task").click(); });
    const searching = taskRuntime.snapshot!;

    let releaseOldSnapshot!: (task: AnalysisTaskDto) => void;
    backend.analysisTaskSnapshot.mockImplementationOnce(() => new Promise<AnalysisTaskDto>((resolve) => {
      releaseOldSnapshot = resolve;
    }));
    await act(async () => {
      listeners.onJob?.({
        run_id: searching.run_id,
        job_id: searching.job_id,
        lane: "whole_game",
        mode: "finite",
        generation: searching.generation,
        node_path: { indices: [] },
        outcome: "started"
      });
      await Promise.resolve();
    });

    let releasePause!: (task: AnalysisTaskDto) => void;
    backend.pauseAnalysisTask.mockImplementationOnce(() => new Promise<AnalysisTaskDto>((resolve) => {
      releasePause = resolve;
    }));
    const pauseButton = buttonNamed(host, "Pause task");
    await act(async () => {
      pauseButton.click();
      pauseButton.click();
      await Promise.resolve();
    });
    expect(backend.pauseAnalysisTask).toHaveBeenCalledTimes(1);
    expect(backend.pauseAnalysisTask).toHaveBeenCalledWith({ runId: "run-1", taskId: "task-1" });
    expect(buttonNamed(host, "Continue task").disabled).toBe(true);

    await act(async () => {
      listeners.onJob?.({
        run_id: searching.run_id,
        job_id: searching.job_id,
        lane: "whole_game",
        mode: "finite",
        generation: searching.generation,
        node_path: { indices: [0] },
        outcome: "progress",
        completed: 2,
        expected: 2,
        remaining: 0,
        frame: {
          job_id: searching.job_id,
          turn: 1,
          visits: 32,
          winrate_black: 0.9,
          score_mean_black: 10,
          candidates: [{ vertex: "pass", visits: 32, winrate_black: 0.9, score_mean_black: 10, pv: [] }]
        }
      });
      releasePause({ ...searching, state: "pausing" });
      await Promise.resolve();
    });
    expect(host.querySelector('[data-analysis-task-state="pausing"]')?.textContent).toContain("1/2 completed");
    expect(buttonNamed(host, "Continue task").disabled).toBe(true);
    expect(buttonNamed(host, "Cancel task").disabled).toBe(false);
    expect(host.querySelectorAll(".cand-row")).toHaveLength(0);

    await act(async () => { releaseOldSnapshot(searching); });
    expect(host.querySelector('[data-analysis-task-state="pausing"]')).not.toBeNull();

    taskRuntime.snapshot = { ...searching, state: "paused" };
    await act(async () => {
      listeners.onJob?.({ ...searching, lane: "whole_game", mode: "finite", node_path: { indices: [] }, outcome: "progress" });
      await Promise.resolve();
      await Promise.resolve();
    });
    expect(host.querySelector('[data-analysis-task-state="paused"]')?.textContent).toContain("1/2 completed");

    let releasePausedSnapshot!: (task: AnalysisTaskDto) => void;
    backend.analysisTaskSnapshot.mockImplementationOnce(() => new Promise<AnalysisTaskDto>((resolve) => {
      releasePausedSnapshot = resolve;
    }));
    await act(async () => {
      listeners.onSnapshot?.({
        revision: 3,
        continuous: { enabled: true, phase: "waiting" },
        lifecycle: {
          state: "ready",
          run: {
            run_id: "run-1",
            profile_id: "profile-1",
            adapter_kind: "kata_go_analysis",
            profile_snapshot: savedProfile.profile
          }
        }
      });
      await Promise.resolve();
    });
    let releaseContinue!: (task: AnalysisTaskDto) => void;
    backend.continueAnalysisTask.mockImplementationOnce(() => new Promise<AnalysisTaskDto>((resolve) => {
      releaseContinue = resolve;
    }));
    const continueButton = buttonNamed(host, "Continue task");
    await act(async () => {
      continueButton.click();
      continueButton.click();
      await Promise.resolve();
    });
    expect(backend.continueAnalysisTask).toHaveBeenCalledTimes(1);
    expect(backend.continueAnalysisTask).toHaveBeenCalledWith({ runId: "run-1", taskId: "task-1" });

    const continued = { ...searching, job_id: "job-task-2", state: "searching" as const };
    taskRuntime.snapshot = continued;
    await act(async () => {
      releaseContinue(continued);
      await Promise.resolve();
      releasePausedSnapshot({ ...searching, state: "paused" });
    });
    expect(host.querySelector('[data-analysis-task-state="searching"]')?.textContent).toContain("1/2 completed");
    expect(buttonNamed(host, "Pause task").disabled).toBe(false);

    await act(async () => {
      listeners.onJob?.({
        run_id: searching.run_id,
        job_id: searching.job_id,
        lane: "whole_game",
        mode: "finite",
        generation: searching.generation,
        node_path: { indices: [0] },
        outcome: "progress",
        completed: 2,
        expected: 2,
        remaining: 0
      });
      await Promise.resolve();
    });
    expect(host.querySelector('[data-analysis-task-state="searching"]')?.textContent).toContain("1/2 completed");
    expect(host.textContent).not.toContain("整局 2/2");
  });

  it("reserves only the whole-game lane while paused and lets Cancel make it terminal", async () => {
    const host = await renderApp();
    await readyEngine(host);
    await act(async () => { buttonNamed(host, "Preview scope").click(); });
    await act(async () => { buttonNamed(host, "Start task").click(); });
    const searching = taskRuntime.snapshot!;
    await act(async () => { buttonNamed(host, "Pause task").click(); });
    taskRuntime.snapshot = { ...searching, state: "paused" };
    await act(async () => {
      listeners.onJob?.({ ...searching, lane: "whole_game", mode: "finite", node_path: { indices: [] }, outcome: "progress" });
      await Promise.resolve();
    });

    expect(buttonNamed(host, "Start task").disabled).toBe(true);
    expect(buttonNamed(host, "闪电分析").disabled).toBe(true);
    expect(Array.from(host.querySelectorAll("button")).filter((button) => button.textContent === "分析第一子主线").every((button) => button.disabled)).toBe(true);
    expect(buttonNamed(host, "分析当前节点").disabled).toBe(false);
    await act(async () => { buttonNamed(host, "分析当前节点").click(); });
    expect(backend.startSelectedNodeAnalysis).toHaveBeenCalledTimes(1);

    await act(async () => {
      listeners.onSnapshot?.({
        revision: 3,
        continuous: { enabled: true, phase: "error" },
        lifecycle: {
          state: "error",
          run: {
            run_id: "run-1",
            profile_id: "profile-1",
            adapter_kind: "kata_go_analysis",
            profile_snapshot: savedProfile.profile
          },
          failure: { operation: "job", kind: "protocol", message: "engine held" }
        }
      });
    });
    expect(buttonNamed(host, "Continue task").disabled).toBe(true);

    backend.cancelKataGoAnalysis.mockImplementationOnce(async () => {
      taskRuntime.snapshot = { ...searching, state: "cancelled", reason: "Cancelled by user." };
    });
    await act(async () => { buttonNamed(host, "Cancel task").click(); });
    expect(backend.cancelKataGoAnalysis).toHaveBeenCalledWith("run-1", "job-task-1");
    expect(host.querySelector('[data-analysis-task-state="cancelled"]')?.textContent).toContain("1/2 completed");
    expect(buttonNamed(host, "Cancel task").disabled).toBe(true);
  });

  it("previews, starts, reports, cancels, and focus-guards a quick task", async () => {
    const host = await renderApp();
    await readyEngine(host);

    const scope = host.querySelector('select[aria-label="Analysis scope"]') as HTMLSelectElement;
    const interval = host.querySelector('input[aria-label="Use position interval"]') as HTMLInputElement;
    const start = host.querySelector('input[aria-label="Position interval start"]') as HTMLInputElement;
    const end = host.querySelector('input[aria-label="Position interval end"]') as HTMLInputElement;
    const side = host.querySelector('select[aria-label="Side to play"]') as HTMLSelectElement;
    const visits = host.querySelector('input[aria-label="Total visits"]') as HTMLInputElement;
    expect(Array.from(scope.options).map((option) => option.value)).toEqual([
      "current_node",
      "selected_review_line",
      "first_child_mainline",
      "all_branches"
    ]);
    await act(async () => {
      scope.value = "selected_review_line";
      scope.dispatchEvent(new Event("change", { bubbles: true }));
    });
    await act(async () => { interval.click(); });
    for (const [input, value] of [[start, "0"], [end, "1"], [visits, "32"]] as const) {
      await act(async () => {
        Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")!.set!.call(input, value);
        input.dispatchEvent(new Event("input", { bubbles: true }));
      });
    }
    await act(async () => {
      side.value = "black";
      side.dispatchEvent(new Event("change", { bubbles: true }));
    });
    await act(async () => {
      buttonNamed(host, "Preview scope").click();
      await backend.previewAnalysisScope.mock.results.at(-1)?.value;
    });
    expect(host.textContent).toContain("2 targets");
    expect(host.textContent).toContain("move 0 (root, black to play)");
    expect(host.textContent).toContain("move 1 (0, white to play)");

    await act(async () => {
      Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")!.set!.call(visits, "4294967296");
      visits.dispatchEvent(new Event("input", { bubbles: true }));
    });
    await act(async () => {
      buttonNamed(host, "Preview scope").click();
      await backend.previewAnalysisScope.mock.results.at(-1)?.value;
    });
    await act(async () => {
      buttonNamed(host, "Start task").click();
      await Promise.resolve();
    });
    expect(backend.startAnalysisTask).not.toHaveBeenCalled();
    expect(preferencesApi.saveAppPreferences).not.toHaveBeenCalled();
    expect(host.textContent).toContain("Total visits must be a whole number");
    await act(async () => {
      Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")!.set!.call(visits, "32");
      visits.dispatchEvent(new Event("input", { bubbles: true }));
    });
    await act(async () => {
      buttonNamed(host, "Preview scope").click();
      await backend.previewAnalysisScope.mock.results.at(-1)?.value;
    });
    preferencesApi.saveAppPreferences.mockRejectedValueOnce(new Error("preferences disk full"));
    await act(async () => {
      buttonNamed(host, "Start task").click();
      await Promise.resolve();
      await Promise.resolve();
    });
    expect(backend.startAnalysisTask).not.toHaveBeenCalled();
    expect(host.textContent).toContain("preferences disk full");


    await act(async () => {
      buttonNamed(host, "Start task").click();
      await backend.startAnalysisTask.mock.results.at(-1)?.value;
    });
    expect(host.textContent).toContain("single-stage · searching · 1/2 completed");
    expect(backend.startAnalysisTask).toHaveBeenCalledWith(expect.objectContaining({
      runId: "run-1",
      conditions: {
        time_seconds: { enabled: false, value: 10 },
        total_visits: { enabled: true, value: 32 },
        leading_candidate_visits: { enabled: false, value: 500 }
      }
    }));
    expect(preferencesApi.saveAppPreferences).toHaveBeenCalledWith(expect.objectContaining({
      taskConditions: {
        time_seconds: { enabled: false, value: 10 },
        total_visits: { enabled: true, value: 32 },
        leading_candidate_visits: { enabled: false, value: 500 }
      }
    }));

    taskRuntime.snapshot = { ...taskRuntime.snapshot!, state: "cancelled", reason: "Cancelled by user." };
    await act(async () => {
      buttonNamed(host, "Cancel task").click();
      await backend.cancelKataGoAnalysis.mock.results.at(-1)?.value;
      await Promise.resolve();
    });
    expect(backend.cancelKataGoAnalysis).toHaveBeenCalledWith("run-1", "job-task-1");
    expect(host.textContent).toContain("cancelled · 1/2 completed");

    backend.startAnalysisTask.mockClear();
    visits.focus();
    act(() => visits.dispatchEvent(new KeyboardEvent("keydown", { key: "b", ctrlKey: true, bubbles: true })));
    expect(backend.startAnalysisTask).not.toHaveBeenCalled();
    visits.blur();
    taskRuntime.snapshot = null;
    await act(async () => {
      window.dispatchEvent(new KeyboardEvent("keydown", { key: "b", ctrlKey: true, bubbles: true }));
      await backend.previewAnalysisScope.mock.results.at(-1)?.value;
      await backend.startAnalysisTask.mock.results.at(-1)?.value;
    });
    expect(backend.startAnalysisTask).toHaveBeenCalledWith(expect.objectContaining({
      conditions: expect.objectContaining({ total_visits: { enabled: true, value: 1 } })
    }));
  });

  it("shows the observed normal ending conditions", async () => {
    const host = await renderApp();
    await readyEngine(host);
    await act(async () => { buttonNamed(host, "Preview scope").click(); });
    await act(async () => { buttonNamed(host, "Start task").click(); });
    taskRuntime.snapshot = {
      ...taskRuntime.snapshot!,
      state: "completed",
      ending_conditions: ["time_seconds", "leading_candidate_visits"],
      reason: "Budget reached."
    };
    await act(async () => {
      listeners.onJob?.({
        run_id: "run-1",
        job_id: "job-task-1",
        lane: "whole_game",
        mode: "finite",
        generation: 1,
        node_path: { indices: [0] },
        outcome: "completed"
      });
      await Promise.resolve();
      await Promise.resolve();
    });
    expect(host.querySelector('[data-analysis-task-state="completed"]')?.textContent)
      .toContain("ended by time + leading candidate visits");
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
