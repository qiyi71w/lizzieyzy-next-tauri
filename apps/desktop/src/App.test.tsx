// @vitest-environment jsdom

import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { AnalysisFrameDto, CurrentGameResultDto, GameDto } from "./domain/types";

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

const analysisFrame: AnalysisFrameDto = {
  job_id: "fake-job",
  turn: 0,
  visits: 100,
  winrate_black: 0.52,
  score_mean_black: 1.5,
  candidates: [
    {
      vertex: { point: { x: 2, y: 3 } },
      visits: 100,
      winrate_black: 0.52,
      score_mean_black: 1.5,
      pv: [{ point: { x: 3, y: 3 } }]
    },
    {
      vertex: { point: { x: 6, y: 5 } },
      visits: 80,
      winrate_black: 0.49,
      score_mean_black: -0.5,
      pv: [{ point: { x: 5, y: 5 } }]
    }
  ]
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

const navigableRoot: CurrentGameResultDto = {
  ...initialGame,
  tree: {
    properties: [],
    children: [{ properties: [{ key: "B", values: ["fe"] }], children: [] }]
  }
};

const navigableChild: CurrentGameResultDto = {
  ...navigableRoot,
  selected_path: { indices: [0] },
  snapshot: {
    path: { indices: [0] },
    position: acceptedGame.snapshot.position,
    personal_comment: ""
  }
};

const staleAnalysisFrame: AnalysisFrameDto = {
  ...analysisFrame,
  job_id: "stale-job",
  candidates: [
    {
      vertex: { point: { x: 0, y: 0 } },
      visits: 10,
      winrate_black: 0.11,
      score_mean_black: -8,
      pv: [{ point: { x: 1, y: 1 } }]
    }
  ]
};

const currentAnalysisFrame: AnalysisFrameDto = {
  ...analysisFrame,
  job_id: "current-job",
  candidates: [
    {
      vertex: { point: { x: 8, y: 8 } },
      visits: 90,
      winrate_black: 0.71,
      score_mean_black: 4,
      pv: [{ point: { x: 7, y: 7 } }]
    }
  ]
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

describe("App candidate continuation preview", () => {
  it("propagates board dwell to the mini-board without mutating the selected game", async () => {
    backend.fakeAnalyze.mockResolvedValue([analysisFrame]);
    backend.classifyProblems.mockResolvedValue([]);
    const host = document.createElement("div");
    document.body.append(host);
    root = createRoot(host);
    act(() => root?.render(<App />));
    await act(async () => {
      await backend.replaceCurrentGame.mock.results[0]?.value;
      await backend.projectCurrentGameMainline.mock.results[0]?.value;
    });

    await act(async () => {
      buttonNamed(host, "AI 解说").click();
      await vi.waitFor(() => expect(backend.fakeAnalyze).toHaveBeenCalledOnce());
      await backend.fakeAnalyze.mock.results[0]?.value;
      await backend.classifyProblems.mock.results[0]?.value;
    });
    expect(host.querySelectorAll(".cand-row")).toHaveLength(2);

    const board = requiredElement<HTMLCanvasElement>(host, 'canvas[aria-label="棋盘"]');
    const miniBoard = requiredElement<HTMLCanvasElement>(host, 'canvas[aria-label="参考图变化副棋盘"]');
    board.getBoundingClientRect = () => new DOMRect(0, 0, 100, 100);
    const beforePreview = miniBoard.dataset.drawn;
    const serializedCalls = backend.serializeCurrentGame.mock.calls.length;
    const currentMove = requiredElement<HTMLInputElement>(host, 'input[aria-label="跳转手数"]').value;

    await act(async () => {
      board.dispatchEvent(new MouseEvent("pointermove", {
        bubbles: true,
        clientX: 70.5,
        clientY: 60.25
      }));
      await new Promise((resolve) => setTimeout(resolve, 130));
    });

    expect(miniBoard.dataset.drawn).not.toBe(beforePreview);
    const rows = host.querySelectorAll(".cand-row");
    expect(rows[0]?.classList.contains("is-selected")).toBe(true);
    expect(rows[1]?.classList.contains("is-selected")).toBe(false);
    expect(requiredElement<HTMLInputElement>(host, 'input[aria-label="跳转手数"]').value).toBe(currentMove);
    expect(backend.serializeCurrentGame).toHaveBeenCalledTimes(serializedCalls);
    expect(backend.selectCurrentGameNode).not.toHaveBeenCalled();
    expect(backend.playCurrentGame).not.toHaveBeenCalled();
    expect(backend.saveCurrentGame).not.toHaveBeenCalled();
    expect(backend.setCurrentGamePersonalComment).not.toHaveBeenCalled();
    expect(backend.removeCurrentGameVariation).not.toHaveBeenCalled();

    act(() => {
      board.dispatchEvent(new MouseEvent("pointerout", { bubbles: true }));
    });
    expect(miniBoard.dataset.drawn).toBe(beforePreview);
  });
});

describe("App stale review presentation", () => {
  it("drops candidate and hover presentation when the selected NodePath changes", async () => {
    backend.replaceCurrentGame.mockResolvedValue(navigableRoot);
    backend.selectCurrentGameNode.mockResolvedValue(navigableChild);
    backend.fakeAnalyze.mockResolvedValue([analysisFrame]);
    backend.classifyProblems.mockResolvedValue([]);

    const host = await renderApp();
    await runFakeAnalyze(host);
    expect(candidateCoords(host)).toEqual(["C6", "G4"]);

    const board = requiredElement<HTMLCanvasElement>(host, 'canvas[aria-label="棋盘"]');
    const miniBoard = requiredElement<HTMLCanvasElement>(host, 'canvas[aria-label="参考图变化副棋盘"]');
    board.getBoundingClientRect = () => new DOMRect(0, 0, 100, 100);
    const beforePreview = miniBoard.dataset.drawn;
    await act(async () => {
      board.dispatchEvent(new MouseEvent("pointermove", {
        bubbles: true,
        clientX: 70.5,
        clientY: 60.25
      }));
      await new Promise((resolve) => setTimeout(resolve, 130));
    });
    expect(miniBoard.dataset.drawn).not.toBe(beforePreview);

    await act(async () => {
      buttonNamed(host, "下一变化").click();
      await backend.selectCurrentGameNode.mock.results[0]?.value;
    });

    expect(requiredElement<HTMLInputElement>(host, 'input[aria-label="跳转手数"]').value).toBe("1");
    expect(candidateCoords(host)).toEqual([]);
    expect(host.querySelector(".cand-row.is-selected")).toBeNull();
  });

  it("ignores a late completion after the selected NodePath has already changed", async () => {
    let finishStale: ((frames: AnalysisFrameDto[]) => void) | undefined;
    backend.replaceCurrentGame.mockResolvedValue(navigableRoot);
    backend.selectCurrentGameNode.mockResolvedValue(navigableChild);
    backend.fakeAnalyze.mockImplementation(() => new Promise((resolve) => {
      finishStale = resolve;
    }));
    backend.classifyProblems.mockResolvedValue([]);

    const host = await renderApp();
    act(() => buttonNamed(host, "AI 解说").click());
    await act(async () => {
      await vi.waitFor(() => expect(backend.fakeAnalyze).toHaveBeenCalledOnce());
    });

    await act(async () => {
      buttonNamed(host, "下一变化").click();
      await backend.selectCurrentGameNode.mock.results[0]?.value;
    });
    expect(requiredElement<HTMLInputElement>(host, 'input[aria-label="跳转手数"]').value).toBe("1");

    await act(async () => {
      finishStale?.([analysisFrame]);
      await backend.fakeAnalyze.mock.results[0]?.value;
      await backend.classifyProblems.mock.results[0]?.value;
    });

    expect(candidateCoords(host)).toEqual([]);
    expect(requiredElement<HTMLInputElement>(host, 'input[aria-label="跳转手数"]').value).toBe("1");
  });

  it("keeps the superseding request and ignores the earlier completion", async () => {
    let finishStale: ((frames: AnalysisFrameDto[]) => void) | undefined;
    backend.fakeAnalyze
      .mockImplementationOnce(() => new Promise((resolve) => {
        finishStale = resolve;
      }))
      .mockResolvedValueOnce([currentAnalysisFrame]);
    backend.classifyProblems.mockResolvedValue([]);

    const host = await renderApp();
    act(() => buttonNamed(host, "AI 解说").click());
    await act(async () => {
      await vi.waitFor(() => expect(backend.fakeAnalyze).toHaveBeenCalledOnce());
    });

    await runFakeAnalyze(host);
    expect(candidateCoords(host)).toEqual(["J1"]);

    await act(async () => {
      finishStale?.([staleAnalysisFrame]);
      await backend.fakeAnalyze.mock.results[0]?.value;
      await backend.classifyProblems.mock.results.at(-1)?.value;
    });

    expect(candidateCoords(host)).toEqual(["J1"]);
  });

  it("clears presentation on game replacement before a late completion can return", async () => {
    let finishStale: ((frames: AnalysisFrameDto[]) => void) | undefined;
    backend.fakeAnalyze.mockImplementation(() => new Promise((resolve) => {
      finishStale = resolve;
    }));
    backend.classifyProblems.mockResolvedValue([]);
    backend.replaceCurrentGame
      .mockResolvedValueOnce(initialGame)
      .mockResolvedValueOnce({ ...initialGame, generation: 4 });

    const host = await renderApp();
    act(() => buttonNamed(host, "AI 解说").click());
    await act(async () => {
      await vi.waitFor(() => expect(backend.fakeAnalyze).toHaveBeenCalledOnce());
    });

    await act(async () => {
      buttonNamed(host, "新对局").click();
      await backend.replaceCurrentGame.mock.results.at(-1)?.value;
      await backend.projectCurrentGameMainline.mock.results.at(-1)?.value;
    });
    expect(requiredElement<HTMLInputElement>(host, 'input[aria-label="跳转手数"]').value).toBe("0");
    expect(candidateCoords(host)).toEqual([]);

    await act(async () => {
      finishStale?.([analysisFrame]);
      await backend.fakeAnalyze.mock.results[0]?.value;
      await backend.classifyProblems.mock.results[0]?.value;
    });
    expect(candidateCoords(host)).toEqual([]);
  });

  it("clears presentation when a move mutates the current game", async () => {
    backend.fakeAnalyze.mockResolvedValue([analysisFrame]);
    backend.classifyProblems.mockResolvedValue([]);
    backend.playCurrentGame.mockResolvedValue(acceptedGame);

    const host = await renderApp();
    await runFakeAnalyze(host);
    expect(candidateCoords(host)).toEqual(["C6", "G4"]);

    const canvas = requiredElement(host, "canvas");
    act(() => canvas.focus());
    dispatchKey(canvas, "Enter");
    await act(async () => {
      await backend.playCurrentGame.mock.results[0]?.value;
      await backend.projectCurrentGameMainline.mock.results.at(-1)?.value;
    });

    expect(requiredElement<HTMLInputElement>(host, 'input[aria-label="跳转手数"]').value).toBe("1");
    expect(candidateCoords(host)).toEqual([]);
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

async function renderApp(): Promise<HTMLElement> {
  const host = document.createElement("div");
  document.body.append(host);
  root = createRoot(host);
  act(() => root?.render(<App />));
  await act(async () => {
    await backend.replaceCurrentGame.mock.results[0]?.value;
    await backend.projectCurrentGameMainline.mock.results[0]?.value;
  });
  return host;
}

async function runFakeAnalyze(host: HTMLElement): Promise<void> {
  const prior = backend.fakeAnalyze.mock.calls.length;
  await act(async () => {
    buttonNamed(host, "AI 解说").click();
    await vi.waitFor(() => expect(backend.fakeAnalyze.mock.calls.length).toBeGreaterThan(prior));
    await backend.fakeAnalyze.mock.results.at(-1)?.value;
    await backend.classifyProblems.mock.results.at(-1)?.value;
  });
}

function candidateCoords(host: HTMLElement): string[] {
  return [...host.querySelectorAll(".cand-coord")].map((cell) => cell.textContent ?? "");
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
