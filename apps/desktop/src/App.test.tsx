// @vitest-environment jsdom

import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { CurrentGameResultDto, GameDto } from "./domain/types";

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
  startKataGoGameAnalysis: vi.fn()
}));

vi.mock("./api/backend", () => ({
  ...backend,
  isTauriRuntime: () => true,
  nativeCurrentGameUnavailable: "Native current-game commands require the Tauri desktop runtime."
}));

vi.mock("./api/analysisCache", () => ({
  computeGameCacheKey: vi.fn(() => Promise.resolve({ gameKey: "game", fileKey: "file" })),
  loadAnalysisCache: vi.fn(() => Promise.resolve({ status: "miss" })),
  saveAnalysisCache: vi.fn()
}));

vi.mock("./api/preferences", () => ({
  loadAppPreferences: vi.fn(() => Promise.reject(new Error("preferences unavailable in test"))),
  saveAppPreferences: vi.fn()
}));

vi.mock("./components/AnalysisPanel", () => ({ AnalysisPanel: () => null }));
vi.mock("./components/CacheStatusBadge", () => ({ CacheStatusBadge: () => null }));
vi.mock("./components/EngineSetupPanel", () => ({ EngineSetupPanel: () => null }));
vi.mock("./components/PreferencesPanel", () => ({ PreferencesPanel: () => null }));
vi.mock("./components/ProviderPanel", () => ({ ProviderPanel: () => null }));
vi.mock("./components/WinrateChart", () => ({ WinrateChart: () => null }));

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

const acceptedGame: CurrentGameResultDto = {
  tree: {
    properties: [],
    children: [{ properties: [{ key: "B", values: ["fe"] }], children: [] }]
  },
  selected_path: { indices: [0] },
  snapshot: {
    path: { indices: [0] },
    position: {
      ...emptyPosition,
      move_number: 1,
      to_play: "white",
      stones: [{ x: 5, y: 4, color: "black" as const }],
      last_move: { color: "black", vertex: { point: { x: 5, y: 4 } }, move_number: 1 }
    },
    personal_comment: ""
  },
  generation: 2,
  dirty: true,
  native_path: null
};

let root: Root | null = null;

beforeEach(() => {
  globalThis.IS_REACT_ACT_ENVIRONMENT = true;
  vi.spyOn(HTMLCanvasElement.prototype, "getContext").mockImplementation(function (this: HTMLCanvasElement) {
    return canvasContext(this);
  });
  backend.replaceCurrentGame.mockResolvedValue(initialGame);
  backend.projectCurrentGameMainline.mockResolvedValue(initialProjection);
});

afterEach(() => {
  act(() => root?.unmount());
  root = null;
  document.body.replaceChildren();
  vi.clearAllMocks();
  vi.restoreAllMocks();
});

describe("App board intent feedback", () => {
  it("keeps review interaction live, deduplicates a pending intent, and announces rejection until the next accepted intent", async () => {
    let rejectPendingMove = false;
    backend.playCurrentGame
      .mockImplementationOnce(async () => {
        await vi.waitUntil(() => rejectPendingMove);
        throw { kind: "occupied_point", message: "point is occupied" };
      })
      .mockResolvedValueOnce(acceptedGame);

    const host = document.createElement("div");
    document.body.append(host);
    root = createRoot(host);
    act(() => root?.render(<App />));
    await act(async () => {
      await backend.replaceCurrentGame.mock.results[0]?.value;
      await backend.projectCurrentGameMainline.mock.results[0]?.value;
    });

    const canvas = requiredElement(host, "canvas");
    act(() => canvas.focus());
    expect(document.activeElement).toBe(canvas);

    dispatchKey(canvas, "Enter");
    dispatchKey(canvas, "Enter");
    expect(backend.playCurrentGame).toHaveBeenCalledTimes(1);
    expect(backend.playCurrentGame).toHaveBeenCalledWith({ indices: [] }, { point: { x: 4, y: 4 } });

    dispatchKey(canvas, "ArrowRight");
    expect(document.activeElement).toBe(canvas);

    const coordinates = buttonNamed(host, "坐标");
    expect(coordinates.getAttribute("aria-pressed")).toBe("true");
    act(() => coordinates.click());
    expect(coordinates.getAttribute("aria-pressed")).toBe("false");

    const rejectedMove = backend.playCurrentGame.mock.results[0].value;
    await act(async () => {
      rejectPendingMove = true;
      await rejectedMove.catch(() => undefined);
    });
    const status = requiredElement(host, ".board-intent-status");
    expect(status.getAttribute("role")).toBe("status");
    expect(status.getAttribute("aria-live")).toBe("polite");
    expect(status.textContent).toContain("落子失败: point is occupied");
    expect(requiredElement<HTMLInputElement>(host, 'input[aria-label="跳转手数"]').value).toBe("0");

    act(() => canvas.focus());
    dispatchKey(canvas, "Enter");
    await act(async () => {
      await backend.playCurrentGame.mock.results[1].value;
      await backend.projectCurrentGameMainline.mock.results.at(-1)?.value;
    });
    expect(backend.playCurrentGame).toHaveBeenCalledTimes(2);
    expect(backend.playCurrentGame).toHaveBeenLastCalledWith({ indices: [] }, { point: { x: 5, y: 4 } });
    expect(requiredElement<HTMLInputElement>(host, 'input[aria-label="跳转手数"]').value).toBe("1");
    expect(host.querySelector(".board-intent-status")).toBeNull();
    expect(requiredElement(host, ".nav-message").textContent).toBe("落子已接受。");
  });
});

function dispatchKey(element: HTMLElement, key: string) {
  act(() => {
    element.dispatchEvent(new KeyboardEvent("keydown", { key, bubbles: true, cancelable: true }));
  });
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
