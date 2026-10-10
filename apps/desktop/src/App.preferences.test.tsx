// @vitest-environment jsdom

import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { CurrentGameResultDto, ForegroundEngineSnapshotDto, GameDto } from "./domain/types";
import { defaultAppPreferences, type AppPreferences } from "./domain/preferences";
import { UNREADABLE_PREFERENCES_RECOVERY_MESSAGE } from "./api/preferences";

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
  readGameFile: vi.fn(),
  takeInitialFileActivation: vi.fn(async () => null),
  takePendingFileActivation: vi.fn(async () => null),
  markFileActivationReady: vi.fn(async () => undefined),
  setFileActivationBusy: vi.fn(async () => undefined),
  subscribeFileActivationAvailable: vi.fn(async () => () => undefined),
  subscribeFileActivationRejected: vi.fn(async () => () => undefined),

  parseSgfSummary: vi.fn(),
  replaySgfPositions: vi.fn(),
  saveCurrentGame: vi.fn(),
  setCurrentGamePersonalComment: vi.fn(),
  removeCurrentGameVariation: vi.fn(),
  undoCurrentGame: vi.fn(),
  redoCurrentGame: vi.fn(),
  startKataGoGameAnalysis: vi.fn(),
  loadEngineProfilesSettings: vi.fn(() => Promise.resolve({ version: 2, selected_profile_id: "default", startup: { mode: "off" as const }, startup_evaluation: { enabled: false, target_profile_id: null }, last_primary_profile_id: null, profiles: [] })),
  subscribeForegroundEngine: vi.fn(async (_onSnapshot?: (snapshot: unknown) => void) => () => undefined),
  subscribeTrialAnalysis: vi.fn(async () => () => undefined),
  startForegroundEngine: vi.fn(),
  stopForegroundEngine: vi.fn(),
  restartForegroundEngine: vi.fn(),
  switchForegroundEngine: vi.fn(),
  getForegroundEngineSnapshot: vi.fn(() => Promise.resolve({ revision: 0, lifecycle: { state: "no_engine" }, continuous: { enabled: null, phase: "loading" } })),
  foregroundEngineContinuousAction: vi.fn(),
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
vi.mock("./api/match", () => ({ nativeMatchUnavailable: "Desktop only", humanMatchStart: vi.fn(), humanMatchAction: vi.fn(), humanMatchStop: vi.fn(), pkMatchStart: vi.fn(), pkMatchPause: vi.fn(), pkMatchResume: vi.fn(),
  subscribeHumanMatch: async () => () => undefined,
  humanMatchSnapshot: async () => ({ current: null, match_state: { revision: 0, mode: "human", pk_runs: null, committed_moves: 0, pause_pending: false, rebuild_sides: [], resume_pending: false, phase: "idle", session_id: null, turn: 0, to_play: null, settings: null, run_id: null, job: null, end: null, failure: null, failed_side: null, resources_held: false, committed: false } })
}));

const preferencesApi = vi.hoisted(() => ({
  loadAppPreferences: vi.fn(),
  saveAppPreferences: vi.fn(),
  updateWorkspaceVisibility: vi.fn()
}));

vi.mock("./api/preferences", async (importOriginal) => {
  const original = await importOriginal<typeof import("./api/preferences")>();
  return {
    ...original,
    loadAppPreferences: preferencesApi.loadAppPreferences,
    saveAppPreferences: preferencesApi.saveAppPreferences,
    updateWorkspaceVisibility: preferencesApi.updateWorkspaceVisibility
  };
});

vi.mock("./api/windowGeometry", () => ({
  nativeWindowGeometryUnavailable: "窗口位置仅在桌面版可用",
  windowGeometryStatus: vi.fn(async () => ({ phase: "saved", geometry: null, error: null })),
  subscribeWindowGeometryStatus: vi.fn(async () => () => undefined),
  resetWindowGeometry: vi.fn(async () => ({ phase: "saved", geometry: null, error: null })),
  retryWindowGeometry: vi.fn(async () => ({ phase: "saved", geometry: null, error: null })),
  freezeWindowGeometry: vi.fn(async () => undefined),
  flushWindowGeometry: vi.fn(async () => undefined)
}));

const pinApi = vi.hoisted(() => ({ loadMainWindowPin: vi.fn(), setMainWindowPin: vi.fn() }));
vi.mock("./api/mainWindowPin", () => pinApi);

vi.mock("./components/EngineSetupPanel", () => ({ EngineSetupPanel: () => null }));
vi.mock("./components/ProviderPanel", () => ({ ProviderPanel: () => null }));
vi.mock("./components/WinrateChart", () => ({
  WinrateChart: () => <canvas aria-label="胜率走势" />
}));

import { App } from "./App";

const emptyPosition = {
  board_width: 9,
  board_height: 9,
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
  snapshot: { path: { indices: [] }, position: emptyPosition, personal_comment: "", markup: [], stone_move_numbers: [] },
  generation: 1,
  snapshot_seq: 1,
  can_undo: false,
  can_redo: false,
  dirty: false,
  native_path: null
};

const initialProjection: GameDto = {
  summary: { id: "test", board_width: 9, board_height: 9, komi: 7.5, move_count: 0 },
  moves: []
};

let root: Root | null = null;

beforeEach(() => {
  globalThis.IS_REACT_ACT_ENVIRONMENT = true;
  vi.spyOn(HTMLCanvasElement.prototype, "getContext").mockImplementation(function (this: HTMLCanvasElement) {
    return canvasContext(this);
  });
  currentGameFixture.mockResolvedValue(initialGame);
  backend.projectCurrentGameMainline.mockResolvedValue(initialProjection);
  preferencesApi.loadAppPreferences.mockResolvedValue({ preferences: defaultAppPreferences });
  preferencesApi.saveAppPreferences.mockImplementation(async (preferences: AppPreferences) => preferences);
  preferencesApi.updateWorkspaceVisibility.mockImplementation(async (patch) => ({ left: true, right: true, ...patch }));
  pinApi.loadMainWindowPin.mockResolvedValue({ actual: false, durable: false, error: null });
  pinApi.setMainWindowPin.mockImplementation(async (value) => ({ actual: value, durable: value, error: null }));
});

afterEach(() => {
  act(() => root?.unmount());
  root = null;
  document.body.replaceChildren();
  vi.clearAllMocks();
  vi.restoreAllMocks();
});

it("validates autoplay drafts and keeps committed intervals after Cancel and write failure", async () => {
  const host = await renderApp();
  openPreferences(host);
  const input = labeledNumber(host, "主棋盘自动播放间隔（秒）");
  expect(input.value).toBe("0.8");
  changeBudgetNumber(input, "2");
  act(() => buttonNamed(host, "取消播放间隔").click());
  expect(input.value).toBe("0.8");
  for (const invalid of ["", "0", "-1", "NaN", "Infinity", "1e309", "2147483.648", "0.0001"]) {
    changeBudgetNumber(input, invalid);
    act(() => buttonNamed(host, "保存播放间隔").click());
    expect(host.textContent).toContain("请输入正数秒");
    expect(preferencesApi.saveAppPreferences).not.toHaveBeenCalled();
  }
  changeBudgetNumber(input, "0.8");
  await act(async () => buttonNamed(host, "保存播放间隔").click());
  expect(input.value).toBe("0.8");
  changeBudgetNumber(input, "1.001");
  await act(async () => buttonNamed(host, "保存播放间隔").click());
  expect(preferencesApi.saveAppPreferences).toHaveBeenLastCalledWith(expect.objectContaining({ reviewAutoplayIntervalMs: 1001 }));
  expect(input.value).toBe("1.001");
  changeBudgetNumber(input, "0.8");
  await act(async () => buttonNamed(host, "保存播放间隔").click());
  changeBudgetNumber(input, "2");
  preferencesApi.saveAppPreferences.mockRejectedValueOnce(new Error("interval replace failed"));
  await act(async () => buttonNamed(host, "保存播放间隔").click());
  expect(input.value).toBe("0.8");
  expect(preferencesStatus(host)).toContain("interval replace failed");
  changeBudgetNumber(input, "2");
  await act(async () => buttonNamed(host, "保存播放间隔").click());
  expect(preferencesApi.saveAppPreferences).toHaveBeenLastCalledWith(expect.objectContaining({ reviewAutoplayIntervalMs: 2000 }));
  expect(input.value).toBe("2");
});

it("captures one ordinary timer at Start, advances chosen nodes, and adopts a saved interval only after restart", async () => {
  const tree = { properties: [], children: [{ properties: [], children: [] }, { properties: [], children: [{ properties: [], children: [] }] }] };
  const start = { ...initialGame, tree, selected_path: { indices: [1] }, snapshot: { ...initialGame.snapshot, path: { indices: [1] } } };
  currentGameFixture.mockResolvedValue(start);
  backend.selectCurrentGameNode.mockImplementation(async (path) => ({ ...start, selected_path: path,
    snapshot_seq: 2, snapshot: { ...start.snapshot, path, position: { ...emptyPosition, move_number: 2 } } }));
  const host = await renderApp();
  const callbacks: Array<() => void> = [];
  const interval = vi.spyOn(window, "setInterval").mockImplementation((callback) => { callbacks.push(callback as () => void); return callbacks.length; });
  const clear = vi.spyOn(window, "clearInterval");
  act(() => buttonNamed(host, "自动播放").click());
  expect(interval).toHaveBeenLastCalledWith(expect.any(Function), 800);
  openPreferences(host);
  changeBudgetNumber(labeledNumber(host, "主棋盘自动播放间隔（秒）"), "2");
  await act(async () => buttonNamed(host, "保存播放间隔").click());
  expect(interval).toHaveBeenCalledTimes(1);
  await act(async () => callbacks[0]());
  expect(backend.selectCurrentGameNode).toHaveBeenLastCalledWith({ indices: [1, 0] }, 1);
  expect(interval).toHaveBeenCalledTimes(1);
  act(() => callbacks[0]());
  expect(buttonNamed(host, "自动播放").getAttribute("aria-pressed")).toBe("false");
  expect(clear).toHaveBeenCalledWith(1);
  act(() => buttonNamed(host, "自动播放").click());
  expect(interval).toHaveBeenLastCalledWith(expect.any(Function), 2000);
  const selectedCount = backend.selectCurrentGameNode.mock.calls.length;
  act(() => callbacks[0]());
  expect(backend.selectCurrentGameNode).toHaveBeenCalledTimes(selectedCount);
});

describe("durable preferences surface", () => {
  it("retains an unapplied focused comment across rail hiding until explicit Apply", async () => {
    const host = await renderApp();
    const editor = requiredElement<HTMLTextAreaElement>(host, 'textarea[aria-label="个人评论"]');
    act(() => {
      editor.focus();
      Object.getOwnPropertyDescriptor(HTMLTextAreaElement.prototype, "value")!.set!.call(editor, "retained rail draft");
      editor.dispatchEvent(new Event("input", { bubbles: true }));
    });
    await act(async () => buttonNamed(host, "左侧栏").click());
    expect(document.activeElement).toBe(buttonNamed(host, "左侧栏"));
    expect(backend.setCurrentGamePersonalComment).not.toHaveBeenCalled();
    await act(async () => buttonNamed(host, "左侧栏").click());
    expect(editor.value).toBe("retained rail draft");
    backend.setCurrentGamePersonalComment.mockResolvedValueOnce({ ...initialGame, dirty: true, snapshot: { ...initialGame.snapshot, personal_comment: "retained rail draft" } });
    await act(async () => buttonNamed(host, "应用评论").click());
    expect(backend.setCurrentGamePersonalComment).toHaveBeenCalledWith(initialGame.selected_path, "retained rail draft");
  });

  it("keeps the exit decision behind an in-flight rail visibility save", async () => {
    const host = await renderApp();
    let finish!: (value: { left: boolean; right: boolean }) => void;
    const pending = new Promise<{ left: boolean; right: boolean }>((resolve) => { finish = resolve; });
    preferencesApi.updateWorkspaceVisibility.mockReturnValueOnce(pending);
    backend.prepareApplicationExit.mockResolvedValueOnce({ status: "needs_decision", departure_id: 61 });
    act(() => buttonNamed(host, "左侧栏").click());
    act(() => buttonNamed(host, "文件").click());
    await act(async () => buttonNamed(host, "退出").click());
    try {
      expect(host.querySelector('[role="dialog"][aria-label="保存当前棋谱"]')).toBeNull();
      expect(backend.resolveApplicationExit).not.toHaveBeenCalled();
      expect(backend.cancelSelectedNodeAnalysis).not.toHaveBeenCalled();
    } finally {
      await act(async () => { finish({ left: false, right: true }); await pending; });
    }
    expect(host.querySelector('[role="dialog"][aria-label="保存当前棋谱"]')).not.toBeNull();
  });

  it.each(["theme", "pin"] as const)("keeps document departure behind an in-flight %s write", async (owner) => {
    const host = await renderApp();
    let finish!: () => void;
    const held = new Promise<void>((resolve) => { finish = resolve; });
    if (owner === "theme") {
      preferencesApi.saveAppPreferences.mockImplementationOnce(async (preferences) => { await held; return preferences; });
      act(() => buttonNamed(host, "参数").click());
      const theme = requiredElement<HTMLSelectElement>(host, 'select:has(option[value="high-contrast"])');
      act(() => { theme.value = "high-contrast"; theme.dispatchEvent(new Event("change", { bubbles: true })); });
    } else {
      pinApi.setMainWindowPin.mockImplementationOnce(async () => { await held; return { actual: true, durable: true, error: null }; });
      act(() => buttonNamed(host, "参数").click());
      act(() => requiredElement<HTMLButtonElement>(host, '.window-pin-control button[role="checkbox"]').click());
    }
    backend.prepareApplicationExit.mockResolvedValueOnce({ status: "needs_decision", departure_id: 62 });
    act(() => buttonNamed(host, "文件").click());
    await act(async () => buttonNamed(host, "退出").click());
    try {
      expect(host.querySelector('[role="dialog"][aria-label="保存当前棋谱"]')).toBeNull();
      expect(backend.resolveApplicationExit).not.toHaveBeenCalled();
    } finally {
      await act(async () => { finish(); await held; });
    }
    expect(host.querySelector('[role="dialog"][aria-label="保存当前棋谱"]')).not.toBeNull();
    await act(async () => requiredElement<HTMLButtonElement>(host, '[role="dialog"] [aria-label="Cancel"]').click());
    expect(backend.confirmNativeExit).not.toHaveBeenCalled();
  });

  it("offers cancel before document departure when a native pin write exceeds the exit deadline", async () => {
    const host = await renderApp();
    let finish!: () => void;
    const held = new Promise<void>((resolve) => { finish = resolve; });
    pinApi.setMainWindowPin.mockImplementationOnce(async () => { await held; return { actual: true, durable: true, error: null }; });
    act(() => buttonNamed(host, "参数").click());
    act(() => requiredElement<HTMLButtonElement>(host, '.window-pin-control button[role="checkbox"]').click());
    vi.useFakeTimers({ toFake: ["setTimeout", "clearTimeout"] });
    try {
      act(() => buttonNamed(host, "文件").click());
      await act(async () => buttonNamed(host, "退出").click());
      await act(async () => { await vi.advanceTimersByTimeAsync(5000); });
      expect(host.querySelector('[role="dialog"][aria-label="工作区或窗口尚未保存"]')).not.toBeNull();
      expect(backend.prepareApplicationExit).not.toHaveBeenCalled();
      expect(backend.resolveApplicationExit).not.toHaveBeenCalled();
      expect(backend.confirmNativeExit).not.toHaveBeenCalled();
      await act(async () => buttonNamed(host, "取消关闭").click());
      expect(buttonNamed(host, "左侧栏").disabled).toBe(false);
    } finally {
      vi.useRealTimers();
      await act(async () => { finish(); await held; });
    }
  });

  it("returns focus from a separator removed by a pending visibility commit", async () => {
    const host = await renderApp();
    let finish!: (value: { left: boolean; right: boolean }) => void;
    preferencesApi.updateWorkspaceVisibility.mockReturnValueOnce(new Promise((resolve) => { finish = resolve; }));
    act(() => buttonNamed(host, "左侧栏").click());
    act(() => requiredElement<HTMLElement>(host, ".workspace-separator-left").focus());
    await act(async () => finish({ left: false, right: true }));
    expect(document.activeElement).toBe(buttonNamed(host, "左侧栏"));
    expect(host.querySelector(".workspace-separator-left")).toBeNull();
  });

  it("commits rail visibility only after success and survives an older ordinary save response", async () => {
    const host = await renderApp();
    let finish!: (value: AppPreferences) => void;
    preferencesApi.saveAppPreferences.mockReturnValueOnce(new Promise<AppPreferences>((resolve) => { finish = resolve; }));
    act(() => buttonNamed(host, "坐标").click());
    preferencesApi.updateWorkspaceVisibility.mockRejectedValueOnce(new Error("rail disk full"));
    await act(async () => buttonNamed(host, "左侧栏").click());
    expect(host.querySelector(".rail")?.hasAttribute("hidden")).toBe(false);
    expect(host.textContent).toContain("rail disk full");
    await act(async () => buttonNamed(host, "左侧栏").click());
    expect(host.querySelector(".rail")?.hasAttribute("hidden")).toBe(true);
    await act(async () => finish({ ...defaultAppPreferences, showCoordinates: false }));
    expect(host.querySelector(".rail")?.hasAttribute("hidden")).toBe(true);
    expect(buttonNamed(host, "坐标").getAttribute("aria-pressed")).toBe("false");
  });

  it("keeps display controls committed until a write succeeds and reports failed shortcut writes", async () => {
    let finishSave!: (value: AppPreferences) => void;
    const pendingSave = new Promise<AppPreferences>((resolve) => { finishSave = resolve; });
    preferencesApi.saveAppPreferences.mockReturnValueOnce(pendingSave);
    const host = await renderApp();
    const coordinates = () => buttonNamed(host, "坐标");
    act(() => coordinates().click());
    expect(coordinates().getAttribute("aria-pressed")).toBe("true");
    await act(async () => {
      finishSave({ ...defaultAppPreferences, showCoordinates: false });
      await pendingSave;
    });
    expect(coordinates().getAttribute("aria-pressed")).toBe("false");
    act(() => buttonNamed(host, "显示").click());
    expect(menuCheck(host, "坐标(C)").getAttribute("aria-checked")).toBe("false");
    act(() => buttonNamed(host, "显示").click());
    preferencesApi.saveAppPreferences.mockRejectedValueOnce(new Error("display disk full"));
    act(() => window.dispatchEvent(new KeyboardEvent("keydown", { key: "c", bubbles: true })));
    await act(async () => { await preferencesApi.saveAppPreferences.mock.results.at(-1)?.value.catch(() => undefined); });
    expect(coordinates().getAttribute("aria-pressed")).toBe("false");
    expect(host.textContent).toContain("display disk full");
    openPreferences(host);
    expect(labeledCheckbox(host, "坐标").checked).toBe(false);
    expect(buttonNamed(host, "手数").getAttribute("aria-pressed")).toBe("false");
  });

  it("adopts mute only after a successful durable write and retains it after failure", async () => {
    const host = await renderApp();
    openPreferences(host);
    let resolveSave!: (value: AppPreferences) => void;
    preferencesApi.saveAppPreferences.mockImplementationOnce(() => new Promise<AppPreferences>((resolve) => { resolveSave = resolve; }));
    act(() => labeledCheckbox(host, "落子声音").click());
    expect(labeledCheckbox(host, "落子声音").checked).toBe(true);
    await act(async () => {
      resolveSave({ ...defaultAppPreferences, soundEnabled: false });
      await preferencesApi.saveAppPreferences.mock.results.at(-1)?.value;
    });
    expect(labeledCheckbox(host, "落子声音").checked).toBe(false);
    preferencesApi.saveAppPreferences.mockRejectedValueOnce(new Error("disk full"));
    act(() => labeledCheckbox(host, "落子声音").click());
    await act(async () => { await preferencesApi.saveAppPreferences.mock.results.at(-1)?.value.catch(() => undefined); });
    expect(labeledCheckbox(host, "落子声音").checked).toBe(false);
    expect(preferencesStatus(host)).toContain("disk full");
  });

  it("keeps invalid budgets local and restores the committed values when persistence fails", async () => {
    const host = await renderApp();
    act(() => buttonNamed(host, "分析").click());
    act(() => buttonNamed(host, "连续分析预算…").click());
    const seconds = () => labeledNumber(host, "连续分析时间（秒）");
    for (const value of ["0", "1.5", "4294967296"]) {
      changeBudgetNumber(seconds(), value);
      act(() => buttonNamed(host, "应用连续预算").click());
      expect(host.querySelector('[role="alert"]')?.textContent).toContain("整数");
      expect(preferencesApi.saveAppPreferences).not.toHaveBeenCalled();
    }
    changeBudgetNumber(seconds(), "2");
    preferencesApi.saveAppPreferences.mockRejectedValueOnce(new Error("disk full"));
    act(() => buttonNamed(host, "应用连续预算").click());
    await act(async () => { await preferencesApi.saveAppPreferences.mock.results.at(-1)?.value.catch(() => undefined); });
    expect(seconds().value).toBe("600");
    expect(preferencesStatus(host)).toContain("disk full");
    expect(backend.startSelectedNodeAnalysis).not.toHaveBeenCalled();

    changeBudgetNumber(seconds(), "2");
    act(() => labeledCheckbox(host, "限制连续分析 visits").click());
    changeBudgetNumber(labeledNumber(host, "连续分析 visits 上限"), "32");
    act(() => buttonNamed(host, "应用连续预算").click());
    await flushLast(preferencesApi.saveAppPreferences);
    expect(seconds().value).toBe("2");
    expect(labeledNumber(host, "连续分析 visits 上限").value).toBe("32");
    act(() => labeledCheckbox(host, "限制连续分析 visits").click());
    act(() => buttonNamed(host, "应用连续预算").click());
    await flushLast(preferencesApi.saveAppPreferences);
    expect(labeledNumber(host, "连续分析 visits 上限").disabled).toBe(true);
    expect(labeledNumber(host, "连续分析 visits 上限").value).toBe("32");
  });

  it("keeps continuous intent disabled until the durable preference load settles", async () => {
    let resolveLoad!: (value: { preferences: AppPreferences }) => void;
    const loadPromise = new Promise<{ preferences: AppPreferences }>((resolve) => { resolveLoad = resolve; });
    preferencesApi.loadAppPreferences.mockReturnValueOnce(loadPromise);
    const host = await renderApp({ waitForLoad: false });
    openPreferences(host);
    expect(labeledCheckbox(host, "连续分析").disabled).toBe(true);

    await act(async () => {
      resolveLoad({ preferences: { ...defaultAppPreferences, continuousAnalysisEnabled: false } });
      await loadPromise;
    });
    expect(labeledCheckbox(host, "连续分析").disabled).toBe(false);
    expect(labeledCheckbox(host, "连续分析").checked).toBe(false);
  });

  it("reports unreadable-storage recovery without replacing owner defaults", async () => {
    preferencesApi.loadAppPreferences.mockResolvedValue({
      preferences: defaultAppPreferences,
      recovery: {
        isolatedPath: "/tmp/lizzieyzy-next-app-preferences.json.unreadable-1",
        message: UNREADABLE_PREFERENCES_RECOVERY_MESSAGE
      }
    });
    const host = await renderApp();
    openPreferences(host);

    expect(labeledCheckbox(host, "候选").checked).toBe(true);
    expect(preferencesStatus(host)).toBe(UNREADABLE_PREFERENCES_RECOVERY_MESSAGE);
  });

  it("keeps the previous visible value when persist fails", async () => {
    let rejectSave!: (error: Error) => void;
    preferencesApi.saveAppPreferences.mockImplementation(
      () => new Promise((_, reject) => {
        rejectSave = reject;
      })
    );
    const host = await renderApp();
    openPreferences(host);

    act(() => labeledCheckbox(host, "候选").click());
    expect(labeledCheckbox(host, "候选").checked).toBe(true);
    expect(preferencesStatus(host)).toBe("Saving preferences...");

    await act(async () => {
      rejectSave(new Error("disk full"));
      await preferencesApi.saveAppPreferences.mock.results.at(-1)?.value.catch(() => undefined);
    });

    expect(labeledCheckbox(host, "候选").checked).toBe(true);
    expect(preferencesStatus(host)).toBe("Save failed: disk full");
    expect(preferencesApi.saveAppPreferences).toHaveBeenCalledTimes(1);
  });

  it("rolls back the continuous preference checkbox when persistence fails", async () => {
    preferencesApi.saveAppPreferences.mockRejectedValueOnce(new Error("intent write failed"));
    const host = await renderApp();
    openPreferences(host);
    act(() => labeledCheckbox(host, "连续分析").click());
    expect(labeledCheckbox(host, "连续分析").checked).toBe(true);
    await act(async () => {
      await Promise.resolve(preferencesApi.saveAppPreferences.mock.results.at(-1)?.value).catch(() => undefined);
    });
    expect(labeledCheckbox(host, "连续分析").checked).toBe(true);
    expect(preferencesStatus(host)).toBe("Save failed: intent write failed");
    expect(backend.foregroundEngineContinuousAction).not.toHaveBeenCalled();
  });

  it("blocks a contextual Start while an older whole-preferences save is pending", async () => {
    backend.subscribeForegroundEngine.mockImplementationOnce(async (onSnapshot?: (snapshot: unknown) => void) => {
      onSnapshot?.({
        revision: 1,
        lifecycle: { state: "ready", run: {
          run_id: "preferences-run",
          profile_id: "preferences-profile",
          adapter_kind: "kata_go_analysis",
          profile_snapshot: { name: "Preferences engine", program: "/katago", argv: [], working_dir: null,
            adapter_kind: "kata_go_analysis", settings: { model_path: "/model", config_path: "/config", max_visits: 16 } },
          capability_snapshot: { adapter_kind: "kata_go_analysis", analysis: {
            selected_node_analysis: true, continuous_analysis: true, whole_game_analysis: true,
            candidates: true, pv: true, winrate: true, root_score: true, ownership: true, policy: true,
            visits_limit: true, protocol_cancel: true
          } }
        } },
        continuous: { enabled: false, phase: "off" }
      } satisfies ForegroundEngineSnapshotDto);
      return () => undefined;
    });
    let resolveOlderSave!: (value: AppPreferences) => void;
    const olderSave = new Promise<AppPreferences>((resolve) => { resolveOlderSave = resolve; });
    preferencesApi.saveAppPreferences.mockReturnValueOnce(olderSave);
    const host = await renderApp();
    openPreferences(host);
    expect(buttonNamed(host, "开始连续分析").disabled).toBe(false);
    act(() => labeledCheckbox(host, "候选").click());
    const start = buttonNamed(host, "开始连续分析");
    expect(start.disabled).toBe(true);
    act(() => start.click());

    await act(async () => {
      resolveOlderSave({ ...defaultAppPreferences, showCandidates: false });
      await olderSave;
      await Promise.resolve();
    });
    expect(backend.foregroundEngineContinuousAction).not.toHaveBeenCalled();
    expect(buttonNamed(host, "开始连续分析").disabled).toBe(false);
    expect(labeledCheckbox(host, "候选").checked).toBe(false);
  });

  it("does not dispatch a stale visible Stop while disabling continuous intent is still saving", async () => {
    let publishSnapshot: ((snapshot: unknown) => void) | undefined;
    backend.subscribeForegroundEngine.mockImplementationOnce(async (onSnapshot?: (snapshot: unknown) => void) => {
      publishSnapshot = onSnapshot;
      onSnapshot?.({ revision: 1, lifecycle: { state: "no_engine" }, continuous: { enabled: true, phase: "searching" } });
      return () => undefined;
    });
    let resolveDisable!: (value: AppPreferences) => void;
    const disableSave = new Promise<AppPreferences>((resolve) => { resolveDisable = resolve; });
    preferencesApi.saveAppPreferences.mockReturnValueOnce(disableSave);
    backend.foregroundEngineContinuousAction.mockResolvedValueOnce({
      ...defaultAppPreferences,
      continuousAnalysisEnabled: true
    });
    const host = await renderApp();
    openPreferences(host);

    act(() => labeledCheckbox(host, "连续分析").click());
    const staleStop = buttonNamed(host, "停止连续分析");
    expect(staleStop.disabled).toBe(true);
    act(() => staleStop.click());

    await act(async () => {
      resolveDisable({ ...defaultAppPreferences, continuousAnalysisEnabled: false });
      await disableSave;
      await Promise.resolve();
      publishSnapshot?.({ revision: 2, lifecycle: { state: "no_engine" }, continuous: { enabled: false, phase: "off" } });
    });
    expect(labeledCheckbox(host, "连续分析").checked).toBe(false);
    expect(buttonNamed(host, "开始连续分析")).toBeInstanceOf(HTMLButtonElement);
    expect(backend.foregroundEngineContinuousAction).not.toHaveBeenCalled();
  });

  it("updates the visible value only after a successful write", async () => {
    const host = await renderApp();
    openPreferences(host);

    act(() => labeledCheckbox(host, "候选").click());
    expect(labeledCheckbox(host, "候选").checked).toBe(true);

    await flushLast(preferencesApi.saveAppPreferences);
    expect(labeledCheckbox(host, "候选").checked).toBe(false);
    expect(preferencesStatus(host)).toBe("Preferences saved.");
    expect(preferencesApi.saveAppPreferences).toHaveBeenCalledWith({
      ...defaultAppPreferences,
      showCandidates: false
    });
  });

  it("routes the View menu and Preferences panel through the same durable store", async () => {
    const host = await renderApp();

    act(() => buttonNamed(host, "显示").click());
    act(() => menuCheck(host, "候选").click());
    await flushLast(preferencesApi.saveAppPreferences);

    act(() => buttonNamed(host, "显示").click());
    expect(menuCheck(host, "候选").getAttribute("aria-checked")).toBe("false");
    expect(preferencesApi.saveAppPreferences).toHaveBeenCalledTimes(1);
    expect(preferencesApi.saveAppPreferences).toHaveBeenLastCalledWith({
      ...defaultAppPreferences,
      showCandidates: false
    });

    openPreferences(host);
    expect(labeledCheckbox(host, "候选").checked).toBe(false);

    act(() => labeledCheckbox(host, "候选").click());
    await flushLast(preferencesApi.saveAppPreferences);

    expect(labeledCheckbox(host, "候选").checked).toBe(true);
    expect(preferencesApi.saveAppPreferences).toHaveBeenCalledTimes(2);
    expect(preferencesApi.saveAppPreferences).toHaveBeenLastCalledWith(defaultAppPreferences);
  });

  it("merges a second explicit edit onto the in-flight durable snapshot", async () => {
    let firstStarted = false;
    let resolveFirstSave: ((preferences: AppPreferences) => void) | undefined;
    preferencesApi.saveAppPreferences.mockImplementation((preferences: AppPreferences) => {
      if (!firstStarted) {
        firstStarted = true;
        return new Promise((resolve) => {
          resolveFirstSave = resolve;
        });
      }
      return Promise.resolve(preferences);
    });
    const host = await renderApp();

    act(() => buttonNamed(host, "显示").click());
    act(() => menuCheck(host, "候选").click());
    expect(preferencesApi.saveAppPreferences).toHaveBeenCalledTimes(1);

    act(() => buttonNamed(host, "显示").click());
    act(() => menuCheck(host, "领地").click());
    expect(preferencesApi.saveAppPreferences).toHaveBeenCalledTimes(1);

    await act(async () => {
      resolveFirstSave?.(preferencesApi.saveAppPreferences.mock.calls[0][0] as AppPreferences);
      await preferencesApi.saveAppPreferences.mock.results[0]?.value;
    });
    await flushLast(preferencesApi.saveAppPreferences);

    expect(preferencesApi.saveAppPreferences).toHaveBeenLastCalledWith({
      ...defaultAppPreferences,
      showCandidates: false,
      showOwnership: false
    });
    act(() => buttonNamed(host, "显示").click());
    expect(menuCheck(host, "候选").getAttribute("aria-checked")).toBe("false");
    expect(menuCheck(host, "领地").getAttribute("aria-checked")).toBe("false");
  });

  it("discards a first-paint edit until durable load becomes the visible baseline", async () => {
    let resolveLoad: ((result: { preferences: AppPreferences }) => void) | undefined;
    preferencesApi.loadAppPreferences.mockImplementation(
      () => new Promise((resolve) => {
        resolveLoad = resolve;
      })
    );
    const host = await renderApp({ waitForLoad: false });

    act(() => buttonNamed(host, "显示").click());
    act(() => menuCheck(host, "候选").click());
    expect(preferencesApi.saveAppPreferences).not.toHaveBeenCalled();

    const loaded = {
      ...defaultAppPreferences,
      showOwnership: false,
      reviewMode: "deep" as const
    };
    await act(async () => {
      resolveLoad?.({ preferences: loaded });
      await preferencesApi.loadAppPreferences.mock.results.at(-1)?.value;
    });

    expect(preferencesApi.saveAppPreferences).not.toHaveBeenCalled();
    openPreferences(host);
    expect(preferencesStatus(host)).toBe("Preferences loaded.");
    expect(labeledCheckbox(host, "候选").checked).toBe(true);
    expect(labeledCheckbox(host, "领地").checked).toBe(false);
    act(() => buttonNamed(host, "显示").click());
    expect(menuCheck(host, "候选").getAttribute("aria-checked")).toBe("true");
    expect(menuCheck(host, "领地").getAttribute("aria-checked")).toBe("false");

    act(() => menuCheck(host, "候选").click());
    await flushLast(preferencesApi.saveAppPreferences);
    expect(preferencesApi.saveAppPreferences).toHaveBeenCalledWith({
      ...loaded,
      showCandidates: false
    });
  });

  it("keeps a successful write when a later in-flight save fails", async () => {
    let resolveFirstSave: ((preferences: AppPreferences) => void) | undefined;
    let rejectSecondSave: ((error: Error) => void) | undefined;
    let saveCount = 0;
    preferencesApi.saveAppPreferences.mockImplementation((preferences: AppPreferences) => {
      saveCount += 1;
      if (saveCount === 1) {
        return new Promise((resolve) => {
          resolveFirstSave = resolve;
        });
      }
      if (saveCount === 2) {
        return new Promise((_, reject) => {
          rejectSecondSave = reject;
        });
      }
      return Promise.resolve(preferences);
    });
    const host = await renderApp();

    act(() => buttonNamed(host, "显示").click());
    act(() => menuCheck(host, "候选").click());
    expect(preferencesApi.saveAppPreferences).toHaveBeenCalledTimes(1);

    act(() => buttonNamed(host, "显示").click());
    act(() => menuCheck(host, "领地").click());
    expect(preferencesApi.saveAppPreferences).toHaveBeenCalledTimes(1);

    await act(async () => {
      resolveFirstSave?.(preferencesApi.saveAppPreferences.mock.calls[0][0] as AppPreferences);
      await preferencesApi.saveAppPreferences.mock.results[0]?.value;
    });
    expect(preferencesApi.saveAppPreferences).toHaveBeenCalledTimes(2);

    await act(async () => {
      rejectSecondSave?.(new Error("disk full"));
      await preferencesApi.saveAppPreferences.mock.results[1]?.value.catch(() => undefined);
    });

    openPreferences(host);
    expect(labeledCheckbox(host, "候选").checked).toBe(false);
    expect(labeledCheckbox(host, "领地").checked).toBe(true);
    expect(preferencesStatus(host)).toBe("Save failed: disk full");

    act(() => buttonNamed(host, "显示").click());
    expect(menuCheck(host, "候选").getAttribute("aria-checked")).toBe("false");
    expect(menuCheck(host, "领地").getAttribute("aria-checked")).toBe("true");

    act(() => menuCheck(host, "策略").click());
    await flushLast(preferencesApi.saveAppPreferences);
    expect(preferencesApi.saveAppPreferences).toHaveBeenLastCalledWith({
      ...defaultAppPreferences,
      showCandidates: false,
      showPolicy: false
    });
  });

  it("does not use a failed write as the next persist base", async () => {
    let rejectFirstSave!: (error: Error) => void;
    let firstStarted = false;
    preferencesApi.saveAppPreferences.mockImplementation((preferences: AppPreferences) => {
      if (!firstStarted) {
        firstStarted = true;
        return new Promise((_, reject) => {
          rejectFirstSave = reject;
        });
      }
      return Promise.resolve(preferences);
    });
    const host = await renderApp();

    act(() => buttonNamed(host, "显示").click());
    act(() => menuCheck(host, "候选").click());
    await act(async () => {
      rejectFirstSave(new Error("disk full"));
      await preferencesApi.saveAppPreferences.mock.results[0]?.value.catch(() => undefined);
    });

    openPreferences(host);
    expect(labeledCheckbox(host, "候选").checked).toBe(true);
    expect(preferencesStatus(host)).toBe("Save failed: disk full");

    act(() => buttonNamed(host, "显示").click());
    expect(menuCheck(host, "候选").getAttribute("aria-checked")).toBe("true");
    act(() => menuCheck(host, "领地").click());
    await flushLast(preferencesApi.saveAppPreferences);

    expect(preferencesApi.saveAppPreferences).toHaveBeenLastCalledWith({
      ...defaultAppPreferences,
      showOwnership: false
    });
    act(() => buttonNamed(host, "显示").click());
    expect(menuCheck(host, "候选").getAttribute("aria-checked")).toBe("true");
    expect(menuCheck(host, "领地").getAttribute("aria-checked")).toBe("false");
  });

  it("keeps previously committed durable fields when a later owner key is saved", async () => {
    const loaded = {
      ...defaultAppPreferences,
      reviewMode: "deep" as const
    };
    preferencesApi.loadAppPreferences.mockResolvedValue({ preferences: loaded });
    const host = await renderApp();

    act(() => buttonNamed(host, "显示").click());
    act(() => menuCheck(host, "候选").click());
    await flushLast(preferencesApi.saveAppPreferences);

    act(() => buttonNamed(host, "显示").click());
    act(() => menuCheck(host, "领地").click());
    await flushLast(preferencesApi.saveAppPreferences);

    expect(preferencesApi.saveAppPreferences).toHaveBeenLastCalledWith({
      ...loaded,
      showCandidates: false,
      showOwnership: false
    });
  });
});


  it("restores the last document on a normal launch when the preference is enabled", async () => {
    preferencesApi.loadAppPreferences.mockResolvedValue({
      preferences: { ...defaultAppPreferences, restoreLastSession: true }
    });
    backend.inspectCurrentGameRecovery.mockResolvedValue({
      status: "normal",
      envelope: {
        document_seq: 1,
        snapshot_seq: 1,
        sgf_text: "(;SZ[9])",
        selected_path: { indices: [] },
        dirty: false,
        disposition: "clean_completed"
      }
    });
    backend.restoreCurrentGameRecovery.mockResolvedValue(initialGame);
    backend.serializeCurrentGame.mockResolvedValue("(;SZ[9])");
    backend.prepareDocumentReplacement.mockClear();
    const host = document.createElement("div");
    document.body.append(host);
    root = createRoot(host);
    act(() => root?.render(<App />));
    await act(async () => {
      await preferencesApi.loadAppPreferences.mock.results.at(-1)?.value;
      await backend.inspectCurrentGameRecovery.mock.results.at(-1)?.value;
      await backend.restoreCurrentGameRecovery.mock.results.at(-1)?.value;
      await backend.serializeCurrentGame.mock.results.at(-1)?.value;
      await backend.projectCurrentGameMainline.mock.results.at(-1)?.value;
    });
    expect(backend.restoreCurrentGameRecovery).toHaveBeenCalled();
    expect(backend.prepareDocumentReplacement).not.toHaveBeenCalled();
    expect(host.textContent).toContain("已恢复上次棋谱");
  });
async function renderApp(options?: { waitForLoad?: boolean }): Promise<HTMLElement> {
  act(() => root?.unmount());
  root = null;
  const host = document.createElement("div");
  document.body.append(host);
  root = createRoot(host);
  act(() => root?.render(<App />));
  await act(async () => {
    if (options?.waitForLoad !== false) {
      await preferencesApi.loadAppPreferences.mock.results.at(-1)?.value.catch(() => undefined);
    }
    await backend.inspectCurrentGameRecovery.mock.results.at(-1)?.value;
    await currentGameFixture.mock.results.at(-1)?.value;
    await backend.projectCurrentGameMainline.mock.results.at(-1)?.value;
  });
  return host;
}

async function flushLast(mock: { mock: { results: Array<{ value?: unknown }> } }) {
  await act(async () => {
    await mock.mock.results.at(-1)?.value;
  });
}

function changeBudgetNumber(input: HTMLInputElement, value: string) {
  act(() => {
    Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")!.set!.call(input, value);
    input.dispatchEvent(new Event("input", { bubbles: true }));
  });
}

function openPreferences(host: HTMLElement) {
  act(() => buttonNamed(host, "参数").click());
}


function preferencesStatus(host: HTMLElement): string {
  return requiredElement(host, ".preferences-header span").textContent ?? "";
}

function labeledCheckbox(host: HTMLElement, name: string): HTMLInputElement {
  return labeledControl(host, name, "input");
}

function labeledNumber(host: HTMLElement, name: string): HTMLInputElement {
  return labeledControl(host, name, "input");
}

function labeledSelect(host: HTMLElement, name: string): HTMLSelectElement {
  return labeledControl(host, name, "select");
}

function labeledControl<T extends Element>(host: HTMLElement, name: string, selector: string): T {
  const label = [...host.querySelectorAll(".preferences-panel label")].find((candidate) => (
    candidate.querySelector("span")?.textContent === name
  ));
  const control = label?.querySelector(selector);
  if (!control) throw new Error(`Missing preference control: ${name}`);
  return control as T;
}

function menuCheck(host: HTMLElement, name: string): HTMLButtonElement {
  const button = [...host.querySelectorAll('[role="menuitemcheckbox"]')].find((candidate) => (
    candidate.textContent?.includes(name) && (name !== "候选" || candidate.textContent === "✓候选" || candidate.textContent === "候选")
  ));
  if (!(button instanceof HTMLButtonElement)) throw new Error(`Missing menu check: ${name}`);
  return button;
}

function buttonNamed(host: HTMLElement, name: string): HTMLButtonElement {
  const button = [...host.querySelectorAll("button")].find((candidate) => candidate.textContent === name);
  if (!(button instanceof HTMLButtonElement)) throw new Error(`Missing button: ${name}`);
  return button;
}

function requiredElement<T extends Element = HTMLElement>(host: HTMLElement, selector: string): T {
  const element = host.querySelector(selector);
  if (!element) throw new Error(`Missing element: ${selector}`);
  return element as T;
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
      return vi.fn();
    },
    set(object, property, value) {
      Reflect.set(object, property, value);
      return true;
    }
  });
}
