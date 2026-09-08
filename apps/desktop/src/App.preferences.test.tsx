// @vitest-environment jsdom

import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { CurrentGameResultDto, GameDto } from "./domain/types";
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
  parseSgfSummary: vi.fn(),
  replaySgfPositions: vi.fn(),
  saveCurrentGame: vi.fn(),
  setCurrentGamePersonalComment: vi.fn(),
  removeCurrentGameVariation: vi.fn(),
  startKataGoGameAnalysis: vi.fn(),
  loadEngineProfilesSettings: vi.fn(() => Promise.resolve({ selected_profile_id: "default", profiles: [] })),
  subscribeForegroundEngine: vi.fn(async (_onSnapshot?: (snapshot: unknown) => void) => () => undefined),
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

const preferencesApi = vi.hoisted(() => ({
  loadAppPreferences: vi.fn(),
  saveAppPreferences: vi.fn()
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
  summary: { id: "test", board_size: 9, komi: 7.5, move_count: 0 },
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
});

afterEach(() => {
  act(() => root?.unmount());
  root = null;
  document.body.replaceChildren();
  vi.clearAllMocks();
  vi.restoreAllMocks();
});

describe("durable preferences surface", () => {
  it("loads owner defaults into the categorized Preferences surface", async () => {
    const host = await renderApp();
    openPreferences(host);

    expect(categoryLegends(host)).toEqual(["分析呈现", "启动", "复盘", "棋盘", "胜率图"]);
    expect(labeledCheckbox(host, "候选").checked).toBe(true);
    expect(labeledCheckbox(host, "领地").checked).toBe(true);
    expect(labeledCheckbox(host, "策略").checked).toBe(true);
    expect(labeledNumber(host, "显示候选数").value).toBe("8");
    expect(labeledNumber(host, "默认计算量").value).toBe("800");
    expect(labeledSelect(host, "复盘深度").value).toBe("quick");
    expect(labeledSelect(host, "棋盘对比").value).toBe("classic");
    expect(labeledSelect(host, "下一手标记").value).toBe("variations");
    expect(labeledSelect(host, "小棋盘内容").value).toBe("variation");
    expect(labeledCheckbox(host, "变化回放").checked).toBe(false);
    expect(labeledNumber(host, "回放间隔").value).toBe("500");
    expect(labeledCheckbox(host, "启动时恢复上次棋谱").checked).toBe(false);
    expect(labeledCheckbox(host, "连续分析").checked).toBe(true);
    expect(host.textContent).not.toContain("自动载入缓存");
    expect(host.textContent).not.toContain("自动保存分析");
    expect(host.textContent).not.toContain("缓存未用");
    expect(host.textContent).not.toContain("命中缓存");
    act(() => buttonNamed(host, "分析").click());
    expect(host.textContent).not.toContain("清除 Lizzie 缓存");
    expect(preferencesStatus(host)).toBe("Preferences loaded.");
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
      onSnapshot?.({ revision: 1, lifecycle: { state: "no_engine" }, continuous: { enabled: false, phase: "off" } });
      return () => undefined;
    });
    let resolveOlderSave!: (value: AppPreferences) => void;
    const olderSave = new Promise<AppPreferences>((resolve) => { resolveOlderSave = resolve; });
    preferencesApi.saveAppPreferences.mockReturnValueOnce(olderSave);
    const host = await renderApp();
    openPreferences(host);
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

function openPreferences(host: HTMLElement) {
  act(() => buttonNamed(host, "参数").click());
}

function categoryLegends(host: HTMLElement): string[] {
  return [...host.querySelectorAll(".preferences-panel fieldset legend")].map((legend) => legend.textContent ?? "");
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
