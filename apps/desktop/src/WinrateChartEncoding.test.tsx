// @vitest-environment jsdom

import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { CurrentGameResultDto, GameDto, NodePath, SgfTreeNodeDto } from "./domain/types";
import { defaultAppPreferences, type AppPreferences } from "./domain/preferences";

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
  loadEngineProfilesSettings: vi.fn(() => Promise.resolve({ selected_profile_id: "default", profiles: [] })),
  subscribeForegroundEngine: vi.fn(() => Promise.resolve(() => undefined)),
  startForegroundEngine: vi.fn(),
  stopForegroundEngine: vi.fn(),
  restartForegroundEngine: vi.fn(),
  switchForegroundEngine: vi.fn(),
  getForegroundEngineSnapshot: vi.fn(() => Promise.resolve({ revision: 0, lifecycle: { state: "no_engine" } }))
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

function lz(header: string): { key: string; values: string[] } {
  return { key: "LZ", values: [`${header}\nmove D16 visits 10 winrate 5000 pv D16`] };
}

function lzop(header: string): { key: string; values: string[] } {
  return { key: "LZOP", values: [`${header}\nmove D16 visits 10 winrate 5000 pv D16`] };
}

function node(properties: SgfTreeNodeDto["properties"], children: SgfTreeNodeDto[] = []): SgfTreeNodeDto {
  return { properties, children };
}

const scoredTree: SgfTreeNodeDto = node([lzop("MainEngine 40.0 100 2.0")], [
  node([{ key: "B", values: ["dd"] }, lz("MainEngine 46.0 100 1.0")], [
    node([{ key: "W", values: ["pp"] }, lz("MainEngine 34.0 100 -2.0")]),
    node([{ key: "W", values: ["pq"] }, lz("MainEngine 55.0 100")])
  ])
]);

const gappedTree: SgfTreeNodeDto = node([lzop("MainEngine 40.0 100 2.0")], [
  node([{ key: "B", values: ["dd"] }], [
    node([{ key: "W", values: ["pp"] }, lz("MainEngine 55.0 80 1.0")])
  ])
]);

const barTree: SgfTreeNodeDto = node([lzop("MainEngine 40.0 100")], [
  node([{ key: "B", values: ["dd"] }, lz("MainEngine 46.0 100")], [
    node([{ key: "W", values: ["pp"] }, lz("MainEngine 34.0 100")], [
      node([{ key: "B", values: ["dq"] }, lz("MainEngine 58.0 100")])
    ])
  ])
]);

const emptyTree: SgfTreeNodeDto = node([]);

const initialProjection: GameDto = {
  summary: { id: "test", board_size: 9, komi: 7.5, move_count: 2 },
  moves: [
    { move_number: 1, color: "black", vertex: { point: { x: 3, y: 3 } } },
    { move_number: 2, color: "white", vertex: { point: { x: 15, y: 15 } } }
  ]
};

let root: Root | null = null;
let activeTree: SgfTreeNodeDto = scoredTree;

function gameAt(path: NodePath): CurrentGameResultDto {
  return {
    tree: activeTree,
    selected_path: path,
    snapshot: {
      path,
      position: {
        ...emptyPosition,
        move_number: path.indices.length,
        to_play: path.indices.length % 2 === 0 ? "black" : "white"
      },
      personal_comment: ""
    },
    generation: 1,
    dirty: false,
    native_path: null
  };
}

beforeEach(() => {
  globalThis.IS_REACT_ACT_ENVIRONMENT = true;
  activeTree = scoredTree;
  vi.spyOn(HTMLCanvasElement.prototype, "getContext").mockImplementation(function (this: HTMLCanvasElement) {
    return canvasContext(this);
  });
  backend.replaceCurrentGame.mockResolvedValue(gameAt({ indices: [] }));
  backend.projectCurrentGameMainline.mockResolvedValue(initialProjection);
  backend.selectCurrentGameNode.mockImplementation(async (path: NodePath) => gameAt(path));
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

describe("winrate chart encoding surface", () => {
  it("starts with Black perspective, both series, Blunder Bar off, hover on, and scale 15", async () => {
    const host = await renderApp();
    const chart = chartShell(host);
    expect(chart.getAttribute("data-perspective")).toBe("black");
    expect(chart.getAttribute("data-show-winrate")).toBe("true");
    expect(chart.getAttribute("data-show-score")).toBe("true");
    expect(chart.getAttribute("data-score-available")).toBe("true");
    expect(chart.getAttribute("data-score-scale")).toBe("15");
    expect(chart.getAttribute("data-bar-count")).toBe("0");
    expect(chart.getAttribute("data-hover")).toBe("true");
    expect(chart.getAttribute("data-current-move")).toBe("0");
    expect(host.textContent).toContain("胜率走势 (黑)");

    openPreferences(host);
    expect(labeledSelect(host, "图表视角").value).toBe("black");
    expect(labeledCheckbox(host, "胜率线").checked).toBe(true);
    expect(labeledCheckbox(host, "目差线").checked).toBe(true);
    expect(labeledCheckbox(host, "失误条").checked).toBe(false);
    expect(labeledCheckbox(host, "图表悬停").checked).toBe(true);
    expect(labeledNumber(host, "目差刻度").value).toBe("15");
  });

  it("lets View and Preferences edit the same durable chart keys", async () => {
    const host = await renderApp();
    act(() => buttonNamed(host, "显示").click());
    act(() => menuCheck(host, "当前行棋方视角").click());
    await flushLast(preferencesApi.saveAppPreferences);
    expect(preferencesApi.saveAppPreferences).toHaveBeenCalledWith({
      ...defaultAppPreferences,
      graphPerspective: "sideToPlay"
    });
    expect(chartShell(host).getAttribute("data-perspective")).toBe("sideToPlay");

    openPreferences(host);
    expect(labeledSelect(host, "图表视角").value).toBe("sideToPlay");
    act(() => {
      const select = labeledSelect(host, "图表视角");
      select.value = "black";
      select.dispatchEvent(new Event("change", { bubbles: true }));
    });
    await flushLast(preferencesApi.saveAppPreferences);
    expect(labeledSelect(host, "图表视角").value).toBe("black");
    expect(chartShell(host).getAttribute("data-perspective")).toBe("black");
    expect(preferencesApi.saveAppPreferences).toHaveBeenLastCalledWith(defaultAppPreferences);
  });

  it("keeps the previous visible chart setting when persist fails", async () => {
    let rejectSave!: (error: Error) => void;
    preferencesApi.saveAppPreferences.mockImplementation(
      () => new Promise((_, reject) => {
        rejectSave = reject;
      })
    );
    const host = await renderApp();
    act(() => buttonNamed(host, "显示").click());
    act(() => menuCheck(host, "失误条").click());
    expect(chartShell(host).getAttribute("data-bar-count")).toBe("0");

    await act(async () => {
      rejectSave(new Error("disk full"));
      await preferencesApi.saveAppPreferences.mock.results.at(-1)?.value.catch(() => undefined);
    });

    expect(chartShell(host).getAttribute("data-bar-count")).toBe("0");
    act(() => buttonNamed(host, "显示").click());
    expect(menuCheck(host, "失误条").getAttribute("aria-checked")).toBe("false");
    openPreferences(host);
    expect(labeledCheckbox(host, "失误条").checked).toBe(false);
    expect(host.querySelector(".preferences-header span")?.textContent).toBe("Save failed: disk full");
  });

  it("flips the whole visible series for selected-node side-to-play", async () => {
    const host = await renderApp();
    act(() => buttonNamed(host, "下一变化").click());
    await act(async () => {
      await backend.selectCurrentGameNode.mock.results.at(-1)?.value;
    });
    act(() => buttonNamed(host, "显示").click());
    act(() => menuCheck(host, "当前行棋方视角").click());
    await flushLast(preferencesApi.saveAppPreferences);
    expect(chartShell(host).getAttribute("data-perspective")).toBe("sideToPlay");
    expect(chartShell(host).getAttribute("data-current-move")).toBe("1");
    expect(host.textContent).toContain("胜率走势 (白)");
  });

  it("keeps persisted score-only when scoreMean is missing without rewriting it", async () => {
    activeTree = node([lzop("MainEngine 40.0 100")]);
    backend.replaceCurrentGame.mockResolvedValue(gameAt({ indices: [] }));
    preferencesApi.loadAppPreferences.mockResolvedValue({
      preferences: { ...defaultAppPreferences, winrateLine: false, scoreLeadLine: true }
    });
    const host = await renderApp();
    expect(chartShell(host).getAttribute("data-score-available")).toBe("false");
    expect(chartShell(host).getAttribute("data-show-winrate")).toBe("true");
    expect(chartShell(host).getAttribute("data-show-score")).toBe("false");
    openPreferences(host);
    expect(labeledCheckbox(host, "胜率线").checked).toBe(false);
    expect(labeledCheckbox(host, "目差线").checked).toBe(true);
    expect(labeledCheckbox(host, "目差线").disabled).toBe(true);
    expect(labeledNumber(host, "目差刻度").disabled).toBe(true);
    act(() => buttonNamed(host, "显示").click());
    expect(menuCheck(host, "目差线").disabled).toBe(true);
    expect(menuCheck(host, "胜率线").getAttribute("aria-checked")).toBe("false");
  });

  it("refuses to turn off the last effective series when scoreMean is missing", async () => {
    activeTree = node([lzop("MainEngine 40.0 100")]);
    backend.replaceCurrentGame.mockResolvedValue(gameAt({ indices: [] }));
    const host = await renderApp();
    act(() => buttonNamed(host, "显示").click());
    act(() => menuCheck(host, "胜率线").click());
    expect(preferencesApi.saveAppPreferences).not.toHaveBeenCalled();
    act(() => buttonNamed(host, "显示").click());
    expect(menuCheck(host, "胜率线").getAttribute("aria-checked")).toBe("true");
    expect(chartShell(host).getAttribute("data-show-winrate")).toBe("true");
  });

  it("keeps missing analysis as a gap on the selected line", async () => {
    activeTree = gappedTree;
    backend.replaceCurrentGame.mockResolvedValue(gameAt({ indices: [] }));
    const host = await renderApp();
    expect(chartShell(host).getAttribute("data-gap-count")).toBe("1");
  });

  it("draws Blunder Bar only for Inaccuracy, Mistake, and Blunder", async () => {
    activeTree = barTree;
    backend.replaceCurrentGame.mockResolvedValue(gameAt({ indices: [] }));
    const host = await renderApp();
    act(() => buttonNamed(host, "显示").click());
    act(() => menuCheck(host, "失误条").click());
    await flushLast(preferencesApi.saveAppPreferences);
    const chart = chartShell(host);
    expect(chart.getAttribute("data-bar-count")).toBe("3");
    expect(chart.getAttribute("data-bar-ranks")).toBe("inaccuracy,mistake,blunder");
  });

  it("shows read-only hover values without navigating, and stays inert when hover is off", async () => {
    const host = await renderApp();
    const canvas = requiredElement<HTMLCanvasElement>(host, 'canvas[aria-label="胜率走势"]');
    canvas.getBoundingClientRect = () => ({
      x: 0, y: 0, left: 0, top: 0, right: 240, bottom: 80, width: 240, height: 80, toJSON: () => ({})
    });
    act(() => {
      canvas.dispatchEvent(new MouseEvent("mousemove", { clientX: 0, clientY: 10, bubbles: true }));
    });
    const status = host.querySelector('[role="status"]');
    expect(status?.textContent).toContain("第 0 手");
    expect(status?.textContent).toContain("胜率");
    expect(chartShell(host).getAttribute("data-current-move")).toBe("0");
    expect(requiredElement<HTMLInputElement>(host, 'input[aria-label="跳转手数"]').value).toBe("0");

    act(() => canvas.dispatchEvent(new MouseEvent("click", { clientX: 120, clientY: 10, bubbles: true })));
    expect(chartShell(host).getAttribute("data-current-move")).toBe("0");

    act(() => buttonNamed(host, "显示").click());
    act(() => menuCheck(host, "图表悬停").click());
    await flushLast(preferencesApi.saveAppPreferences);
    expect(chartShell(host).getAttribute("data-hover")).toBe("false");
    act(() => {
      canvas.dispatchEvent(new MouseEvent("mousemove", { clientX: 40, clientY: 10, bubbles: true }));
    });
    expect(host.querySelector(".winrate-chart-hover")).toBeNull();
  });

  it("keeps the empty chart surface when there is no engine analysis", async () => {
    activeTree = emptyTree;
    backend.replaceCurrentGame.mockResolvedValue(gameAt({ indices: [] }));
    const host = await renderApp();
    expect(host.querySelector('canvas[aria-label="胜率走势"]')).not.toBeNull();
    expect(chartShell(host).getAttribute("data-gap-count")).toBe("1");
    expect(chartShell(host).getAttribute("data-show-winrate")).toBe("true");
    expect(chartShell(host).getAttribute("data-show-score")).toBe("false");
    expect(chartShell(host).getAttribute("data-bar-count")).toBe("0");
  });

  it("grows the session score axis without persisting the peak, and ignores invalid scale input", async () => {
    activeTree = node([lzop("MainEngine 40.0 100 21.0")]);
    backend.replaceCurrentGame.mockResolvedValue(gameAt({ indices: [] }));
    const host = await renderApp();
    expect(chartShell(host).getAttribute("data-score-scale")).toBe("21");
    openPreferences(host);
    expect(labeledNumber(host, "目差刻度").value).toBe("15");
    act(() => {
      const input = labeledNumber(host, "目差刻度");
      const setter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")?.set;
      setter?.call(input, "-3");
      input.dispatchEvent(new InputEvent("input", { bubbles: true, data: "-3" }));
      input.dispatchEvent(new Event("change", { bubbles: true }));
    });
    expect(preferencesApi.saveAppPreferences).not.toHaveBeenCalled();
    expect(labeledNumber(host, "目差刻度").value).toBe("15");
    expect(chartShell(host).getAttribute("data-score-scale")).toBe("21");
  });
});

async function renderApp(): Promise<HTMLElement> {
  act(() => root?.unmount());
  root = null;
  const host = document.createElement("div");
  document.body.append(host);
  root = createRoot(host);
  act(() => root?.render(<App />));
  await act(async () => {
    await backend.replaceCurrentGame.mock.results.at(-1)?.value;
    await backend.projectCurrentGameMainline.mock.results.at(-1)?.value;
    await preferencesApi.loadAppPreferences.mock.results.at(-1)?.value.catch(() => undefined);
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

function chartShell(host: HTMLElement): HTMLElement {
  return requiredElement(host, ".winrate-chart-shell");
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
      return vi.fn();
    },
    set(object, property, value) {
      Reflect.set(object, property, value);
      return true;
    }
  });
}
