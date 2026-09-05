// @vitest-environment jsdom

import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { AnalysisFrameDto, ApplicationExitOutcomeDto, CurrentGameResultDto, DocumentDepartureAdmissionDto, GameDto, NodePath } from "./domain/types";

const backend = vi.hoisted(() => ({
  getHealth: vi.fn(() => Promise.resolve({ status: "ok" })),
  replaceCurrentGame: vi.fn(),
  prepareDocumentReplacement: vi.fn(async () => ({ status: "ready", departure_id: 1 })),
  prepareApplicationExit: vi.fn(async (): Promise<DocumentDepartureAdmissionDto> => ({ status: "ready", departure_id: 1 })),
  resolveApplicationExit: vi.fn(async (input: { action: string }): Promise<ApplicationExitOutcomeDto> => {
    if (input.action === "cancel") {
      return { committed: false, analysis_stopped: false, current: null, message: "Exit cancelled.", disposition: null, teardown: null };
    }
    const current = await backend.replaceCurrentGame("", null) as CurrentGameResultDto;
    return { committed: true, analysis_stopped: true, current, message: "Application exit completed.", disposition: "clean_completed", teardown: { status: "completed" } };
  }),
  retryApplicationTeardown: vi.fn(async (): Promise<ApplicationExitOutcomeDto> => ({ committed: true, analysis_stopped: true, current: null, message: "Application exit completed.", disposition: "clean_completed", teardown: { status: "completed" } })),
  confirmApplicationExitAnyway: vi.fn(async (): Promise<ApplicationExitOutcomeDto> => ({ committed: true, analysis_stopped: true, current: null, message: "Application exit completed.", disposition: "exit_incomplete", teardown: { status: "timed_out", outstanding: ["foreground engine"] } })),
  confirmNativeExit: vi.fn(async () => undefined),
  subscribeApplicationExitRequested: vi.fn(async (_onRequest: () => void) => () => undefined),
  resolveDocumentReplacement: vi.fn(async (input: { action: string }) => {
    if (input.action === "cancel") {
      return { committed: false, analysis_stopped: false, current: null, message: "Replacement cancelled." };
    }
    const current = await backend.replaceCurrentGame("", null);
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
  loadAppPreferences: vi.fn(() => Promise.reject(new Error("preferences unavailable in test"))),
  saveAppPreferences: vi.fn(async (preferences: unknown) => preferences)
}));

vi.mock("./api/preferences", () => preferencesApi);

vi.mock("./components/EngineSetupPanel", () => ({ EngineSetupPanel: () => null }));
vi.mock("./components/PreferencesPanel", () => ({ PreferencesPanel: () => null }));
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

const branchingGame: CurrentGameResultDto = {
  tree: {
    properties: [],
    children: [{
      properties: [{ key: "B", values: ["pd"] }],
      children: [
        { properties: [{ key: "W", values: ["dd"] }], children: [] },
        { properties: [{ key: "W", values: ["pp"] }], children: [] }
      ]
    }]
  },
  selected_path: { indices: [0, 1] },
  snapshot: {
    path: { indices: [0, 1] },
    position: { ...emptyPosition, move_number: 2, to_play: "black" },
    personal_comment: ""
  },
  generation: 3,
  dirty: true,
  native_path: "/tmp/review.sgf"
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

    const canvas = requiredElement(host, 'canvas[aria-label="棋盘"]');
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
    backend.classifyProblems.mockResolvedValue([{
      turn: 1,
      severity: "mistake",
      winrate_loss: 0.12,
      score_loss: 3,
      label: "恶手"
    }]);

    const host = await renderApp();
    await runFakeAnalyze(host);
    expect(candidateCoords(host)).toEqual(["C6", "G4"]);
    expect(buttonNamed(host, "问题手 (1)")).toBeTruthy();

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
    expect(buttonNamed(host, "问题手 (0)")).toBeTruthy();
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

    const canvas = requiredElement(host, 'canvas[aria-label="棋盘"]');
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

describe("App focus-safe review controls", () => {
  beforeEach(() => {
    backend.replaceCurrentGame.mockResolvedValue(branchingGame);
    backend.projectCurrentGameMainline.mockResolvedValue({
      summary: { id: "branch", board_size: 9, komi: 7.5, move_count: 2 },
      moves: []
    });
    backend.selectCurrentGameNode.mockImplementation(async (path: NodePath) => ({
      ...branchingGame,
      selected_path: path,
      snapshot: {
        ...branchingGame.snapshot,
        path,
        position: { ...branchingGame.snapshot.position, move_number: path.indices.length }
      }
    }));
    backend.saveCurrentGame.mockResolvedValue({ ...branchingGame, dirty: false });
    backend.openSgfDocument.mockResolvedValue({ sgfText: "(;SZ[9])", path: "/tmp/opened.sgf" });
    backend.playCurrentGame.mockImplementation(async (path: NodePath) => ({
      ...branchingGame,
      selected_path: path,
      snapshot: { ...branchingGame.snapshot, path },
      generation: 5,
      dirty: true
    }));
    backend.setCurrentGamePersonalComment.mockResolvedValue({
      ...branchingGame,
      dirty: true
    });
    backend.removeCurrentGameVariation.mockResolvedValue({
      ...branchingGame,
      selected_path: { indices: [0] },
      snapshot: {
        ...branchingGame.snapshot,
        path: { indices: [0] },
        position: { ...emptyPosition, move_number: 1 }
      },
      generation: 4
    });
    Object.defineProperty(navigator, "clipboard", {
      configurable: true,
      value: {
        writeText: vi.fn(async () => undefined),
        readText: vi.fn(async () => "(;SZ[9])")
      }
    });
    vi.spyOn(window, "confirm").mockReturnValue(true);
  });

  it("ignores application shortcuts on the board and text-editing targets", async () => {
    const host = await renderApp();
    const canvas = requiredElement(host, 'canvas[aria-label="棋盘"]');
    const coordinates = buttonNamed(host, "坐标");
    const jump = requiredElement<HTMLInputElement>(host, 'input[aria-label="跳转手数"]');

    backend.selectCurrentGameNode.mockClear();
    act(() => canvas.focus());
    pressKey(canvas, "ArrowLeft");
    pressKey(canvas, "c");
    expect(backend.selectCurrentGameNode).not.toHaveBeenCalled();
    expect(coordinates.getAttribute("aria-pressed")).toBe("true");

    act(() => jump.focus());
    pressKey(jump, "ArrowLeft");
    pressKey(jump, "c");
    expect(backend.selectCurrentGameNode).not.toHaveBeenCalled();
    expect(coordinates.getAttribute("aria-pressed")).toBe("true");

    const sheet = document.createElement("textarea");
    host.append(sheet);
    act(() => sheet.focus());
    pressKey(sheet, "c");
    expect(coordinates.getAttribute("aria-pressed")).toBe("true");

    const combo = document.createElement("select");
    host.append(combo);
    act(() => combo.focus());
    pressKey(combo, "c");
    expect(coordinates.getAttribute("aria-pressed")).toBe("true");

    const editable = document.createElement("div");
    editable.setAttribute("contenteditable", "true");
    host.append(editable);
    act(() => editable.focus());
    pressKey(editable, "c");
    expect(coordinates.getAttribute("aria-pressed")).toBe("true");
  });

  it("dispatches claimed navigation shortcuts through the same actions as the visible controls", async () => {
    const hostParent = await renderApp();
    backend.selectCurrentGameNode.mockClear();
    act(() => buttonNamed(hostParent, "父节点").click());
    await flushLast(backend.selectCurrentGameNode);
    expect(backend.selectCurrentGameNode).toHaveBeenLastCalledWith({ indices: [0] });
    expect(requiredElement<HTMLInputElement>(hostParent, 'input[aria-label="跳转手数"]').value).toBe("1");

    const hostLeft = await renderApp();
    backend.selectCurrentGameNode.mockClear();
    pressKey(buttonNamed(hostLeft, "坐标"), "ArrowLeft");
    await flushLast(backend.selectCurrentGameNode);
    expect(backend.selectCurrentGameNode).toHaveBeenLastCalledWith({ indices: [0] });

    const hostPrevMove = await renderApp();
    backend.selectCurrentGameNode.mockClear();
    act(() => buttonLabeled(hostPrevMove, "上一手").click());
    await flushLast(backend.selectCurrentGameNode);
    expect(backend.selectCurrentGameNode).toHaveBeenLastCalledWith({ indices: [0] });

    const hostUp = await renderApp();
    backend.selectCurrentGameNode.mockClear();
    act(() => buttonNamed(hostUp, "上一分支").click());
    await flushLast(backend.selectCurrentGameNode);
    expect(backend.selectCurrentGameNode).toHaveBeenLastCalledWith({ indices: [0, 0] });
    expect(requiredElement(hostUp, '.nav-cluster[aria-label="变化导航"] .move-indicator').textContent).toBe("分支 1/2");

    const hostUpKey = await renderApp();
    backend.selectCurrentGameNode.mockClear();
    pressKey(buttonNamed(hostUpKey, "坐标"), "ArrowUp");
    await flushLast(backend.selectCurrentGameNode);
    expect(backend.selectCurrentGameNode).toHaveBeenLastCalledWith({ indices: [0, 0] });

    const hostRight = await renderApp();
    backend.selectCurrentGameNode.mockClear();
    act(() => buttonNamed(hostRight, "父节点").click());
    await flushLast(backend.selectCurrentGameNode);
    backend.selectCurrentGameNode.mockClear();
    act(() => buttonNamed(hostRight, "下一变化").click());
    await flushLast(backend.selectCurrentGameNode);
    expect(backend.selectCurrentGameNode).toHaveBeenLastCalledWith({ indices: [0, 1] });

    const hostRightKey = await renderApp();
    backend.selectCurrentGameNode.mockClear();
    act(() => buttonNamed(hostRightKey, "父节点").click());
    await flushLast(backend.selectCurrentGameNode);
    backend.selectCurrentGameNode.mockClear();
    pressKey(buttonNamed(hostRightKey, "坐标"), "ArrowRight");
    await flushLast(backend.selectCurrentGameNode);
    expect(backend.selectCurrentGameNode).toHaveBeenLastCalledWith({ indices: [0, 1] });

    const hostDown = await renderApp();
    backend.selectCurrentGameNode.mockClear();
    act(() => buttonNamed(hostDown, "上一分支").click());
    await flushLast(backend.selectCurrentGameNode);
    backend.selectCurrentGameNode.mockClear();
    act(() => buttonNamed(hostDown, "下一分支").click());
    await flushLast(backend.selectCurrentGameNode);
    expect(backend.selectCurrentGameNode).toHaveBeenLastCalledWith({ indices: [0, 1] });

    const hostDownKey = await renderApp();
    backend.selectCurrentGameNode.mockClear();
    act(() => buttonNamed(hostDownKey, "上一分支").click());
    await flushLast(backend.selectCurrentGameNode);
    backend.selectCurrentGameNode.mockClear();
    pressKey(buttonNamed(hostDownKey, "坐标"), "ArrowDown");
    await flushLast(backend.selectCurrentGameNode);
    expect(backend.selectCurrentGameNode).toHaveBeenLastCalledWith({ indices: [0, 1] });

    const hostFirst = await renderApp();
    backend.selectCurrentGameNode.mockClear();
    act(() => buttonLabeled(hostFirst, "首手").click());
    await flushLast(backend.selectCurrentGameNode);
    expect(backend.selectCurrentGameNode).toHaveBeenLastCalledWith({ indices: [] });
    pressKey(buttonNamed(hostFirst, "坐标"), "Home");
    expect(backend.selectCurrentGameNode).toHaveBeenLastCalledWith({ indices: [] });

    const hostLast = await renderApp();
    backend.selectCurrentGameNode.mockClear();
    act(() => buttonLabeled(hostLast, "首手").click());
    await flushLast(backend.selectCurrentGameNode);
    backend.selectCurrentGameNode.mockClear();
    act(() => buttonLabeled(hostLast, "末手").click());
    await flushLast(backend.selectCurrentGameNode);
    expect(backend.selectCurrentGameNode).toHaveBeenLastCalledWith({ indices: [0, 1] });

    const hostEnd = await renderApp();
    backend.selectCurrentGameNode.mockClear();
    act(() => buttonLabeled(hostEnd, "首手").click());
    await flushLast(backend.selectCurrentGameNode);
    backend.selectCurrentGameNode.mockClear();
    pressKey(buttonNamed(hostEnd, "坐标"), "End");
    await flushLast(backend.selectCurrentGameNode);
    expect(backend.selectCurrentGameNode).toHaveBeenLastCalledWith({ indices: [0, 1] });

    const hostBack = await renderApp();
    backend.selectCurrentGameNode.mockClear();
    act(() => buttonLabeled(hostBack, "回退 10 手").click());
    await flushLast(backend.selectCurrentGameNode);
    expect(backend.selectCurrentGameNode).toHaveBeenLastCalledWith({ indices: [] });
    pressKey(buttonNamed(hostBack, "坐标"), "PageUp");
    expect(backend.selectCurrentGameNode).toHaveBeenLastCalledWith({ indices: [] });

    const hostForward = await renderApp();
    backend.selectCurrentGameNode.mockClear();
    act(() => buttonLabeled(hostForward, "首手").click());
    await flushLast(backend.selectCurrentGameNode);
    backend.selectCurrentGameNode.mockClear();
    act(() => buttonLabeled(hostForward, "前进 10 手").click());
    await flushLast(backend.selectCurrentGameNode);
    expect(backend.selectCurrentGameNode).toHaveBeenLastCalledWith({ indices: [0, 1] });

    const hostPage = await renderApp();
    backend.selectCurrentGameNode.mockClear();
    act(() => buttonLabeled(hostPage, "首手").click());
    await flushLast(backend.selectCurrentGameNode);
    backend.selectCurrentGameNode.mockClear();
    pressKey(buttonNamed(hostPage, "坐标"), "PageDown");
    await flushLast(backend.selectCurrentGameNode);
    expect(backend.selectCurrentGameNode).toHaveBeenLastCalledWith({ indices: [0, 1] });

    const hostNextMove = await renderApp();
    backend.selectCurrentGameNode.mockClear();
    act(() => buttonLabeled(hostNextMove, "首手").click());
    await flushLast(backend.selectCurrentGameNode);
    backend.selectCurrentGameNode.mockClear();
    act(() => buttonLabeled(hostNextMove, "下一手").click());
    await flushLast(backend.selectCurrentGameNode);
    expect(backend.selectCurrentGameNode).toHaveBeenLastCalledWith({ indices: [0] });
  });

  it("dispatches claimed file and edit shortcuts through the same actions as the visible controls", async () => {
    backend.playCurrentGame.mockClear();
    const hostPass = await renderApp();
    act(() => buttonLabeled(hostPass, "虚手").click());
    await flushLast(backend.playCurrentGame);
    expect(backend.playCurrentGame).toHaveBeenLastCalledWith({ indices: [0, 1] }, "pass");

    const hostPassKey = await renderApp();
    backend.playCurrentGame.mockClear();
    pressKey(buttonNamed(hostPassKey, "坐标"), "p");
    await flushLast(backend.playCurrentGame);
    expect(backend.playCurrentGame).toHaveBeenLastCalledWith({ indices: [0, 1] }, "pass");

    backend.saveCurrentGame.mockClear();
    const hostSave = await renderApp();
    act(() => buttonNamed(hostSave, "存档").click());
    await flushLast(backend.saveCurrentGame);
    expect(backend.saveCurrentGame).toHaveBeenCalledTimes(1);
    expect(backend.saveCurrentGame).toHaveBeenLastCalledWith("/tmp/review.sgf", { indices: [0, 1] }, "review.sgf");

    backend.saveCurrentGame.mockClear();
    const hostSaveKey = await renderApp();
    pressKey(buttonNamed(hostSaveKey, "坐标"), "s", { ctrlKey: true });
    await flushLast(backend.saveCurrentGame);
    expect(backend.saveCurrentGame).toHaveBeenCalledTimes(1);
    expect(backend.saveCurrentGame).toHaveBeenLastCalledWith("/tmp/review.sgf", { indices: [0, 1] }, "review.sgf");

    backend.openSgfDocument.mockClear();
    const hostOpen = await renderApp();
    act(() => buttonLabeled(hostOpen, "打开").click());
    await flushLast(backend.openSgfDocument);
    expect(backend.openSgfDocument).toHaveBeenCalledTimes(1);
    pressKey(buttonNamed(hostOpen, "坐标"), "o");
    await flushLast(backend.openSgfDocument);
    expect(backend.openSgfDocument).toHaveBeenCalledTimes(2);

    backend.removeCurrentGameVariation.mockClear();
    const hostRemove = await renderApp();
    act(() => buttonNamed(hostRemove, "删除变化").click());
    await flushLast(backend.removeCurrentGameVariation);
    expect(backend.removeCurrentGameVariation).toHaveBeenCalledTimes(1);

    const hostRemoveKey = await renderApp();
    pressKey(buttonNamed(hostRemoveKey, "坐标"), "Delete", { shiftKey: true });
    await flushLast(backend.removeCurrentGameVariation);
    expect(backend.removeCurrentGameVariation).toHaveBeenCalledTimes(2);

    const hostKeys = await renderApp();
    const replaceCalls = backend.replaceCurrentGame.mock.calls.length;
    pressKey(buttonNamed(hostKeys, "坐标"), "c", { ctrlKey: true });
    await flushLast(backend.serializeCurrentGame);
    expect(backend.serializeCurrentGame).toHaveBeenCalled();

    pressKey(buttonNamed(hostKeys, "坐标"), "v", { ctrlKey: true });
    await act(async () => {
      await Promise.resolve();
    });
    expect(backend.replaceCurrentGame.mock.calls.length).toBeGreaterThan(replaceCalls);

    pressKey(buttonNamed(hostKeys, "坐标"), "n");
    await act(async () => {
      await Promise.resolve();
    });
    expect(backend.replaceCurrentGame).toHaveBeenCalled();

    const newCalls = backend.replaceCurrentGame.mock.calls.length;
    pressKey(buttonNamed(hostKeys, "坐标"), "Home", { ctrlKey: true });
    await act(async () => {
      await Promise.resolve();
    });
    expect(backend.replaceCurrentGame.mock.calls.length).toBeGreaterThan(newCalls);

    backend.saveCurrentGame.mockClear();
    pressKey(buttonNamed(hostKeys, "坐标"), "s");
    await flushLast(backend.saveCurrentGame);
    expect(backend.saveCurrentGame).toHaveBeenLastCalledWith(null, { indices: [0, 1] }, "review.sgf");
  });

  it("dispatches claimed view shortcuts through the same actions as the visible controls", async () => {
    const host = await renderApp();
    const moveNumbers = buttonNamed(host, "手数");
    expect(moveNumbers.getAttribute("aria-pressed")).toBe("false");
    pressKey(buttonNamed(host, "坐标"), "c");
    expect(buttonNamed(host, "坐标").getAttribute("aria-pressed")).toBe("false");
    pressKey(buttonNamed(host, "坐标"), "m");
    expect(moveNumbers.getAttribute("aria-pressed")).toBe("true");

    const autoplay = buttonNamed(host, "自动播放");
    expect(autoplay.getAttribute("aria-pressed")).toBe("false");
    pressKey(autoplay, "a", { ctrlKey: true });
    expect(autoplay.getAttribute("aria-pressed")).toBe("true");

    act(() => buttonNamed(host, "显示").click());
    const policyItem = [...host.querySelectorAll("button")].find((candidate) => candidate.textContent?.includes("策略网络(T)"));
    expect(policyItem?.getAttribute("aria-checked")).toBe("true");
    pressKey(buttonNamed(host, "坐标"), "t");
    await flushLast(preferencesApi.saveAppPreferences);
    act(() => buttonNamed(host, "显示").click());
    act(() => buttonNamed(host, "显示").click());
    const policyItemAfter = [...host.querySelectorAll("button")].find((candidate) => candidate.textContent?.includes("策略网络(T)"));
    expect(policyItemAfter?.getAttribute("aria-checked")).toBe("false");
  });

  it("routes enabled chrome menu items through the same claimed owners", async () => {
    const hostFirst = await renderApp();
    backend.selectCurrentGameNode.mockClear();
    act(() => buttonNamed(hostFirst, "编辑").click());
    act(() => buttonNamed(hostFirst, "跳转到最前").click());
    await flushLast(backend.selectCurrentGameNode);
    expect(backend.selectCurrentGameNode).toHaveBeenLastCalledWith({ indices: [] });

    const hostRemove = await renderApp();
    backend.removeCurrentGameVariation.mockClear();
    act(() => buttonNamed(hostRemove, "编辑").click());
    act(() => buttonNamed(hostRemove, "删除分支").click());
    await flushLast(backend.removeCurrentGameVariation);
    expect(backend.removeCurrentGameVariation).toHaveBeenCalledTimes(1);

    const hostPass = await renderApp();
    backend.playCurrentGame.mockClear();
    act(() => buttonNamed(hostPass, "编辑").click());
    act(() => buttonNamed(hostPass, "停一手(P)").click());
    await flushLast(backend.playCurrentGame);
    expect(backend.playCurrentGame).toHaveBeenLastCalledWith({ indices: [0, 1] }, "pass");

    backend.openSgfDocument.mockClear();
    const hostOpen = await renderApp();
    act(() => buttonNamed(hostOpen, "文件").click());
    act(() => buttonNamed(hostOpen, "打开棋谱(O)").click());
    await flushLast(backend.openSgfDocument);
    expect(backend.openSgfDocument).toHaveBeenCalledTimes(1);

    backend.saveCurrentGame.mockClear();
    const hostSaveAs = await renderApp();
    act(() => buttonNamed(hostSaveAs, "文件").click());
    act(() => buttonNamed(hostSaveAs, "另存为(S)").click());
    await flushLast(backend.saveCurrentGame);
    expect(backend.saveCurrentGame).toHaveBeenLastCalledWith(null, { indices: [0, 1] }, "review.sgf");

    const sampleCalls = backend.replaceCurrentGame.mock.calls.length;
    const hostSample = await renderApp();
    act(() => buttonNamed(hostSample, "文件").click());
    act(() => buttonNamed(hostSample, "载入示例").click());
    await act(async () => {
      await Promise.resolve();
    });
    expect(backend.replaceCurrentGame.mock.calls.length).toBeGreaterThan(sampleCalls);

    const hostImport = await renderApp();
    act(() => buttonNamed(hostImport, "文件").click());
    act(() => buttonNamed(hostImport, "导入棋谱…").click());
    expect(hostImport.querySelector('textarea[aria-label="棋谱载入文本"]')).not.toBeNull();

    const parseCalls = backend.replaceCurrentGame.mock.calls.length;
    const hostParse = await renderApp();
    act(() => buttonNamed(hostParse, "棋局").click());
    act(() => buttonNamed(hostParse, "解析棋谱").click());
    await act(async () => {
      await Promise.resolve();
    });
    expect(backend.replaceCurrentGame.mock.calls.length).toBeGreaterThan(parseCalls);

    const refreshCalls = backend.replaceCurrentGame.mock.calls.length;
    const hostRefresh = await renderApp();
    act(() => buttonNamed(hostRefresh, "刷新").click());
    await act(async () => {
      await Promise.resolve();
    });
    expect(backend.replaceCurrentGame.mock.calls.length).toBeGreaterThan(refreshCalls);
  });

  it("selects candidates from number keys and the candidate list while ignoring board and text targets", async () => {
    installCandidateAnalysis();
    const host = await renderApp();
    await waitForCandidateRows(host);

    const [first, second] = candidateRows(host);
    expect(first?.classList.contains("is-selected")).toBe(true);

    pressKey(buttonNamed(host, "坐标"), "1");
    expect(first?.classList.contains("is-selected")).toBe(true);

    const canvas = requiredElement(host, 'canvas[aria-label="棋盘"]');
    act(() => canvas.focus());
    pressKey(canvas, "2");
    expect(first?.classList.contains("is-selected")).toBe(true);
    expect(second?.classList.contains("is-selected")).toBe(false);

    const jump = requiredElement<HTMLInputElement>(host, 'input[aria-label="跳转手数"]');
    act(() => jump.focus());
    pressKey(jump, "2");
    expect(first?.classList.contains("is-selected")).toBe(true);

    pressKey(buttonNamed(host, "坐标"), "2");
    expect(second?.classList.contains("is-selected")).toBe(true);
    act(() => first?.dispatchEvent(new MouseEvent("click", { bubbles: true })));
    expect(first?.classList.contains("is-selected")).toBe(true);
  });

  it("applies overlay and filter controls from their visible owners", async () => {
    installCandidateAnalysis();
    const host = await renderApp();
    await waitForCandidateRows(host);

    const policyOverlay = buttonNamed(host, "策略");
    expect(policyOverlay.disabled).toBe(false);
    act(() => buttonNamed(host, "纯网络").click());
    expect(policyOverlay.getAttribute("aria-pressed")).toBe("true");

    const hostKey = await renderApp();
    await waitForCandidateRows(hostKey);
    pressKey(buttonNamed(hostKey, "坐标"), "h");
    expect(buttonNamed(hostKey, "策略").getAttribute("aria-pressed")).toBe("true");

    act(() => buttonNamed(hostKey, "Kata评估").click());
    expect(buttonNamed(hostKey, "领地").getAttribute("aria-pressed")).toBe("true");

    const blackFilter = requiredElement<HTMLInputElement>(hostKey, ".check-label input");
    expect(blackFilter.checked).toBe(true);
    act(() => blackFilter.click());
    expect(blackFilter.checked).toBe(false);
  });

  it("routes claimed preference and comment controls through their existing owners", async () => {
    const host = await renderApp();
    act(() => buttonNamed(host, "显示").click());
    const candidatePref = menuCheck(host, "候选");
    const ownershipPref = menuCheck(host, "领地");
    expect(candidatePref.getAttribute("aria-checked")).toBe("true");
    expect(ownershipPref.getAttribute("aria-checked")).toBe("true");
    act(() => candidatePref.click());
    await flushLast(preferencesApi.saveAppPreferences);
    act(() => buttonNamed(host, "显示").click());
    expect(menuCheck(host, "候选").getAttribute("aria-checked")).toBe("false");
    act(() => menuCheck(host, "领地").click());
    await flushLast(preferencesApi.saveAppPreferences);
    act(() => buttonNamed(host, "显示").click());
    expect(menuCheck(host, "领地").getAttribute("aria-checked")).toBe("false");

    const editor = requiredElement<HTMLTextAreaElement>(host, 'textarea[aria-label="个人评论"]');
    const coordinates = buttonNamed(host, "坐标");
    expect(coordinates.getAttribute("aria-pressed")).toBe("true");
    act(() => editor.focus());
    pressKey(editor, "c");
    expect(coordinates.getAttribute("aria-pressed")).toBe("true");
    backend.setCurrentGamePersonalComment.mockClear();
    act(() => {
      const valueSetter = Object.getOwnPropertyDescriptor(HTMLTextAreaElement.prototype, "value")?.set;
      valueSetter?.call(editor, "reviewer note");
      editor.dispatchEvent(new Event("input", { bubbles: true }));
    });
    act(() => editor.blur());
    await flushLast(backend.setCurrentGamePersonalComment);
    expect(backend.setCurrentGamePersonalComment).toHaveBeenLastCalledWith({ indices: [0, 1] }, "reviewer note");
  });

  it("keeps unsupported baseline actions visible-disabled with 尚未接入 and does not claim analysis keys", async () => {
    const host = await renderApp();
    const hawkeye = buttonLabeled(host, "超级鹰眼");
    expect(hawkeye.disabled).toBe(true);
    expect(hawkeye.title).toBe("尚未接入");
    act(() => buttonNamed(host, "编辑").click());
    const deleteMove = buttonNamed(host, "删除一手");
    expect(deleteMove.disabled).toBe(true);
    expect(deleteMove.title).toBe("尚未接入");
    const setMain = buttonNamed(host, "设为主分支");
    expect(setMain.disabled).toBe(true);
    expect(setMain.title).toBe("尚未接入");
    const pass = buttonNamed(host, "停一手(P)");
    expect(pass.disabled).toBe(false);

    backend.startSelectedNodeAnalysis.mockClear();
    backend.startKataGoGameAnalysis.mockClear();
    pressKey(buttonNamed(host, "坐标"), "a");
    pressKey(buttonNamed(host, "坐标"), " ");
    expect(backend.startSelectedNodeAnalysis).not.toHaveBeenCalled();
    expect(backend.startKataGoGameAnalysis).not.toHaveBeenCalled();
  });

  it("opens the same searchable Shortcut Reference from Help and focus-safe ?", async () => {
    const host = await renderApp();
    pressKey(buttonNamed(host, "坐标"), "?", { shiftKey: true });
    const dialog = requiredElement(host, '[role="dialog"][aria-label="快捷键参考"]');
    expect(dialog.textContent).toContain("自动播放");
    expect(dialog.textContent).toContain("Ctrl+A");
    expect(dialog.textContent).toContain("快捷键参考");
    expect(dialog.textContent).toContain("?");

    const search = requiredElement<HTMLInputElement>(dialog, 'input[aria-label="搜索快捷键"]');
    act(() => {
      const valueSetter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")?.set;
      valueSetter?.call(search, "自动");
      search.dispatchEvent(new Event("input", { bubbles: true }));
    });
    expect(dialog.textContent).toContain("自动播放");
    expect(dialog.textContent).not.toContain("打开棋谱");

    pressKey(dialog, "Escape");
    expect(host.querySelector('[role="dialog"][aria-label="快捷键参考"]')).toBeNull();

    act(() => buttonNamed(host, "帮助").click());
    act(() => buttonNamed(host, "快捷键参考(?)").click());
    const fromHelp = requiredElement(host, '[role="dialog"][aria-label="快捷键参考"]');
    expect(fromHelp.textContent).toContain("自动播放");
    expect(fromHelp.textContent).toContain("Ctrl+A");
  });

  it("keeps typed ? in editable controls instead of opening Shortcut Reference", async () => {
    const host = await renderApp();
    const editor = requiredElement<HTMLTextAreaElement>(host, 'textarea[aria-label="个人评论"]');
    act(() => editor.focus());
    pressKey(editor, "?", { shiftKey: true });
    expect(host.querySelector('[role="dialog"][aria-label="快捷键参考"]')).toBeNull();
    expect(buttonNamed(host, "坐标").getAttribute("aria-pressed")).toBe("true");
  });
});

describe("App document replacement", () => {
  beforeEach(() => {
    backend.projectCurrentGameMainline.mockResolvedValue(initialProjection);
    backend.openSgfDocument.mockResolvedValue({ sgfText: "(;SZ[9])", path: "/tmp/opened.sgf" });
    Object.defineProperty(navigator, "clipboard", {
      configurable: true,
      value: {
        writeText: vi.fn(async () => undefined),
        readText: vi.fn(async () => "(;SZ[9])")
      }
    });
  });

  it("rejects an invalid candidate before prompting and keeps the current game", async () => {
    backend.prepareDocumentReplacement
      .mockResolvedValueOnce({ status: "ready", departure_id: 1 })
      .mockRejectedValueOnce({ kind: "malformed_sgf", message: "malformed SGF" });
    const host = await renderApp();
    await act(async () => {
      (navigator.clipboard.readText as ReturnType<typeof vi.fn>).mockResolvedValue("not an sgf");
      pressKey(buttonNamed(host, "坐标"), "v", { ctrlKey: true });
      await Promise.resolve();
      await Promise.resolve();
    });
    expect(host.querySelector('[role="dialog"][aria-label="保存当前棋谱"]')).toBeNull();
    expect(backend.resolveDocumentReplacement).toHaveBeenCalledTimes(1);
    expect(host.textContent).toContain("粘贴失败");
  });

  it("shows Save/Discard/Cancel for a dirty document and Cancel does not stop analysis", async () => {
    backend.prepareDocumentReplacement.mockImplementation(async () => {
      if (backend.prepareDocumentReplacement.mock.calls.length <= 1) {
        return { status: "ready", departure_id: 1 };
      }
      return { status: "needs_decision", departure_id: 2 };
    });
    const host = await renderApp();
    backend.cancelSelectedNodeAnalysis.mockClear();
    backend.resolveDocumentReplacement.mockClear();
    await act(async () => {
      buttonLabeled(host, "新建").click();
      await backend.prepareDocumentReplacement.mock.results.at(-1)?.value;
    });
    expect(host.querySelector('[role="dialog"][aria-label="保存当前棋谱"]')).not.toBeNull();
    expect(backend.resolveDocumentReplacement).not.toHaveBeenCalled();
    await act(async () => {
      buttonLabeled(host, "Cancel").click();
      await backend.resolveDocumentReplacement.mock.results.at(-1)?.value;
    });
    expect(backend.resolveDocumentReplacement).toHaveBeenCalledWith(expect.objectContaining({ action: "cancel" }));
    expect(backend.cancelSelectedNodeAnalysis).not.toHaveBeenCalled();
    expect(host.querySelector('[role="dialog"][aria-label="保存当前棋谱"]')).toBeNull();
    expect(host.textContent).toContain("已取消替换");
  });

  it("discards without writing the old source file", async () => {
    backend.prepareDocumentReplacement.mockImplementation(async () => {
      if (backend.prepareDocumentReplacement.mock.calls.length <= 1) {
        return { status: "ready", departure_id: 1 };
      }
      return { status: "needs_decision", departure_id: 2 };
    });
    const host = await renderApp();
    backend.saveCurrentGame.mockClear();
    backend.resolveDocumentReplacement.mockClear();
    await act(async () => {
      buttonLabeled(host, "新建").click();
      await backend.prepareDocumentReplacement.mock.results.at(-1)?.value;
    });
    await act(async () => {
      buttonLabeled(host, "Discard").click();
      await backend.resolveDocumentReplacement.mock.results.at(-1)?.value;
    });
    expect(backend.resolveDocumentReplacement).toHaveBeenCalledWith(expect.objectContaining({ action: "discard" }));
    expect(backend.saveCurrentGame).not.toHaveBeenCalled();
  });

  it("save-and-leave asks the owner to save then replace", async () => {
    backend.prepareDocumentReplacement.mockImplementation(async () => {
      if (backend.prepareDocumentReplacement.mock.calls.length <= 1) {
        return { status: "ready", departure_id: 1 };
      }
      return { status: "needs_decision", departure_id: 2 };
    });
    const host = await renderApp();
    backend.saveCurrentGame.mockClear();
    backend.resolveDocumentReplacement.mockClear();
    await act(async () => {
      buttonLabeled(host, "新建").click();
      await backend.prepareDocumentReplacement.mock.results.at(-1)?.value;
    });
    await act(async () => {
      buttonLabeled(host, "Save").click();
      await backend.resolveDocumentReplacement.mock.results.at(-1)?.value;
    });
    expect(backend.resolveDocumentReplacement).toHaveBeenCalledWith(expect.objectContaining({ action: "save" }));
    expect(backend.saveCurrentGame).not.toHaveBeenCalled();
  });

  it("opens a file picker and validates the candidate before any dirty prompt", async () => {
    const host = await renderApp();
    backend.prepareDocumentReplacement.mockClear();
    backend.prepareDocumentReplacement.mockResolvedValue({ status: "ready", departure_id: 2 });
    backend.openSgfDocument.mockClear();
    backend.openSgfDocument.mockResolvedValue({ sgfText: "(;SZ[9])", path: "/tmp/opened.sgf" });
    await act(async () => {
      buttonLabeled(host, "打开").click();
      await backend.openSgfDocument.mock.results.at(-1)?.value;
      await backend.prepareDocumentReplacement.mock.results.at(-1)?.value;
    });
    expect(backend.openSgfDocument).toHaveBeenCalledTimes(1);
    expect(backend.prepareDocumentReplacement).toHaveBeenCalledWith("(;SZ[9])", "/tmp/opened.sgf");
    expect(host.querySelector('[role="dialog"][aria-label="保存当前棋谱"]')).toBeNull();
  });

  it("routes paste, sample, New, and parse through the shared replacement owner", async () => {
    const host = await renderApp();
    backend.prepareDocumentReplacement.mockClear();
    backend.prepareDocumentReplacement.mockResolvedValue({ status: "ready", departure_id: 3 });
    await act(async () => {
      pressKey(buttonNamed(host, "坐标"), "v", { ctrlKey: true });
      await backend.prepareDocumentReplacement.mock.results.at(-1)?.value;
    });
    await act(async () => {
      buttonNamed(host, "文件").click();
    });
    await act(async () => {
      buttonNamed(host, "载入示例").click();
      await backend.prepareDocumentReplacement.mock.results.at(-1)?.value;
    });
    await act(async () => {
      buttonLabeled(host, "新建").click();
      await backend.prepareDocumentReplacement.mock.results.at(-1)?.value;
    });
    await act(async () => {
      buttonNamed(host, "棋局").click();
    });
    await act(async () => {
      buttonNamed(host, "解析棋谱").click();
      await backend.prepareDocumentReplacement.mock.results.at(-1)?.value;
    });
    expect(backend.prepareDocumentReplacement.mock.calls.length).toBe(4);
  });
});


describe("App application exit", () => {
  it("routes File Exit and window close through one clean-exit transaction", async () => {
    const host = await renderApp();
    backend.prepareApplicationExit.mockClear();
    backend.resolveApplicationExit.mockClear();
    backend.confirmNativeExit.mockClear();
    await act(async () => {
      buttonNamed(host, "文件").click();
    });
    await act(async () => {
      buttonNamed(host, "退出").click();
      await backend.prepareApplicationExit.mock.results.at(-1)?.value;
      await backend.resolveApplicationExit.mock.results.at(-1)?.value;
      await backend.confirmNativeExit.mock.results.at(-1)?.value;
    });
    expect(backend.prepareApplicationExit).toHaveBeenCalledTimes(1);
    expect(backend.resolveApplicationExit).toHaveBeenCalledWith(expect.objectContaining({ action: "continue" }));
    expect(backend.confirmNativeExit).toHaveBeenCalledTimes(1);
    expect(host.querySelector('[role="dialog"][aria-label="保存当前棋谱"]')).toBeNull();
  });

  it("keeps analysis running when the initial exit prompt is cancelled", async () => {
    backend.prepareApplicationExit.mockResolvedValue({ status: "needs_decision", departure_id: 4 });
    const host = await renderApp();
    backend.cancelSelectedNodeAnalysis.mockClear();
    backend.resolveApplicationExit.mockClear();
    backend.confirmNativeExit.mockClear();
    await act(async () => {
      buttonNamed(host, "文件").click();
    });
    await act(async () => {
      buttonNamed(host, "退出").click();
      await backend.prepareApplicationExit.mock.results.at(-1)?.value;
    });
    expect(host.querySelector('[role="dialog"][aria-label="保存当前棋谱"]')).not.toBeNull();
    await act(async () => {
      buttonLabeled(host, "Cancel").click();
      await backend.resolveApplicationExit.mock.results.at(-1)?.value;
    });
    expect(backend.resolveApplicationExit).toHaveBeenCalledWith(expect.objectContaining({ action: "cancel" }));
    expect(backend.cancelSelectedNodeAnalysis).not.toHaveBeenCalled();
    expect(backend.confirmNativeExit).not.toHaveBeenCalled();
    expect(host.textContent).toContain("已取消退出");
  });

  it("does not open a second prompt for a repeated close while exit is in progress", async () => {
    backend.prepareApplicationExit.mockResolvedValue({ status: "needs_decision", departure_id: 5 });
    const host = await renderApp();
    await act(async () => {
      buttonNamed(host, "文件").click();
    });
    await act(async () => {
      buttonNamed(host, "退出").click();
      await backend.prepareApplicationExit.mock.results.at(-1)?.value;
    });
    backend.prepareApplicationExit.mockClear();
    await act(async () => {
      buttonNamed(host, "文件").click();
    });
    await act(async () => {
      buttonNamed(host, "退出").click();
    });
    expect(backend.prepareApplicationExit).not.toHaveBeenCalled();
    expect(host.querySelectorAll('[role="dialog"][aria-label="保存当前棋谱"]')).toHaveLength(1);
  });

  it("uses the same File Exit owner for a native close request", async () => {
    const listeners: Array<() => void> = [];
    backend.subscribeApplicationExitRequested.mockImplementation(async (onRequest: () => void) => {
      listeners.push(onRequest);
      return () => undefined;
    });
    const host = await renderApp();
    backend.prepareApplicationExit.mockClear();
    backend.resolveApplicationExit.mockClear();
    backend.confirmNativeExit.mockClear();
    expect(listeners).toHaveLength(1);
    await act(async () => {
      listeners[0]();
      await backend.prepareApplicationExit.mock.results.at(-1)?.value;
      await backend.resolveApplicationExit.mock.results.at(-1)?.value;
      await backend.confirmNativeExit.mock.results.at(-1)?.value;
    });
    expect(backend.prepareApplicationExit).toHaveBeenCalledTimes(1);
    expect(backend.resolveApplicationExit).toHaveBeenCalledWith(expect.objectContaining({ action: "continue" }));
    expect(backend.confirmNativeExit).toHaveBeenCalledTimes(1);
    expect(host.querySelector('[role="dialog"][aria-label="保存当前棋谱"]')).toBeNull();
  });

  it("names outstanding resources and offers Retry then Exit anyway", async () => {
    backend.prepareApplicationExit.mockResolvedValue({ status: "ready", departure_id: 6 });
    backend.resolveApplicationExit.mockResolvedValue({
      committed: true,
      analysis_stopped: true,
      current: initialGame,
      message: "Exit teardown timed out: foreground engine. Retry or exit anyway.",
      disposition: "exit_incomplete",
      teardown: { status: "timed_out", outstanding: ["foreground engine"] }
    });
    backend.retryApplicationTeardown.mockResolvedValue({
      committed: true,
      analysis_stopped: true,
      current: initialGame,
      message: "Exit teardown timed out: foreground engine. Retry or exit anyway.",
      disposition: "exit_incomplete",
      teardown: { status: "timed_out", outstanding: ["foreground engine"] }
    });
    const host = await renderApp();
    backend.confirmNativeExit.mockClear();
    await act(async () => {
      buttonNamed(host, "文件").click();
    });
    await act(async () => {
      buttonNamed(host, "退出").click();
      await backend.prepareApplicationExit.mock.results.at(-1)?.value;
      await backend.resolveApplicationExit.mock.results.at(-1)?.value;
    });
    const timeoutDialog = host.querySelector('[role="dialog"][aria-label="退出清理未完成"]');
    expect(timeoutDialog).not.toBeNull();
    expect(timeoutDialog?.textContent).toContain("foreground engine");
    expect(backend.confirmNativeExit).not.toHaveBeenCalled();
    await act(async () => {
      buttonLabeled(host, "Retry").click();
      await backend.retryApplicationTeardown.mock.results.at(-1)?.value;
    });
    expect(backend.retryApplicationTeardown).toHaveBeenCalledTimes(1);
    await act(async () => {
      buttonLabeled(host, "Exit anyway").click();
      await backend.confirmApplicationExitAnyway.mock.results.at(-1)?.value;
      await backend.confirmNativeExit.mock.results.at(-1)?.value;
    });
    expect(backend.confirmApplicationExitAnyway).toHaveBeenCalledWith(expect.objectContaining({
      outstanding: ["foreground engine"]
    }));
    expect(backend.confirmNativeExit).toHaveBeenCalledTimes(1);
  });

  it("saves a dirty document then exits without a second prompt", async () => {
    backend.prepareApplicationExit.mockResolvedValue({ status: "needs_decision", departure_id: 7 });
    backend.resolveApplicationExit.mockResolvedValue({
      committed: true,
      analysis_stopped: true,
      current: { ...initialGame, dirty: false, native_path: "/tmp/review.sgf" },
      message: "Application exit completed.",
      disposition: "clean_completed",
      teardown: { status: "completed" }
    });
    const host = await renderApp();
    backend.confirmNativeExit.mockClear();
    await act(async () => {
      buttonNamed(host, "文件").click();
    });
    await act(async () => {
      buttonNamed(host, "退出").click();
      await backend.prepareApplicationExit.mock.results.at(-1)?.value;
    });
    await act(async () => {
      buttonLabeled(host, "Save").click();
      await backend.resolveApplicationExit.mock.results.at(-1)?.value;
      await backend.confirmNativeExit.mock.results.at(-1)?.value;
    });
    expect(backend.resolveApplicationExit).toHaveBeenCalledWith(expect.objectContaining({ action: "save" }));
    expect(backend.confirmNativeExit).toHaveBeenCalledTimes(1);
  });

  it("keeps the window after a failed final save and does not exit", async () => {
    backend.prepareApplicationExit.mockResolvedValue({ status: "needs_decision", departure_id: 8 });
    backend.resolveApplicationExit.mockResolvedValue({
      committed: false,
      analysis_stopped: true,
      current: initialGame,
      message: "disk full. Analysis is stopped; restart it explicitly.",
      disposition: null,
      teardown: null
    });
    const host = await renderApp();
    backend.confirmNativeExit.mockClear();
    await act(async () => {
      buttonNamed(host, "文件").click();
    });
    await act(async () => {
      buttonNamed(host, "退出").click();
      await backend.prepareApplicationExit.mock.results.at(-1)?.value;
    });
    await act(async () => {
      buttonLabeled(host, "Save").click();
      await backend.resolveApplicationExit.mock.results.at(-1)?.value;
    });
    expect(backend.confirmNativeExit).not.toHaveBeenCalled();
    expect(host.textContent).toContain("disk full");
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

async function waitForCandidateRows(host: HTMLElement) {
  await act(async () => {
    await vi.waitFor(() => expect(candidateRows(host)).toHaveLength(2));
  });
}

function candidateRows(host: HTMLElement): HTMLTableRowElement[] {
  return [...host.querySelectorAll<HTMLTableRowElement>(".cand-row")];
}

function installCandidateAnalysis() {
  const policy = Array.from({ length: 81 }, (_, index) => (index === 0 ? 0.2 : 0));
  const ownership = Array.from({ length: 81 }, () => 0);
  const frame: AnalysisFrameDto = {
    job_id: "job",
    turn: 0,
    visits: 20,
    winrate_black: 0.5,
    score_mean_black: 0,
    candidates: [
      { vertex: { point: { x: 2, y: 2 } }, visits: 8, winrate_black: 0.55, score_mean_black: 1, pv: [] },
      { vertex: { point: { x: 3, y: 3 } }, visits: 6, winrate_black: 0.48, score_mean_black: 0, pv: [] }
    ],
    ownership,
    policy
  };
  backend.replaceCurrentGame.mockResolvedValue({
    ...initialGame,
    snapshot: {
      ...initialGame.snapshot,
      primary_analysis: frame
    }
  });
}

function pressKey(target: EventTarget, key: string, init: KeyboardEventInit = {}) {
  act(() => {
    target.dispatchEvent(new KeyboardEvent("keydown", { key, bubbles: true, cancelable: true, ...init }));
  });
}

function menuCheck(host: HTMLElement, name: string): HTMLButtonElement {
  const button = [...host.querySelectorAll('[role="menuitemcheckbox"]')].find((candidate) => (
    candidate.textContent?.includes(name) && (name !== "候选" || candidate.textContent === "✓候选" || candidate.textContent === "候选")
  ));
  if (!(button instanceof HTMLButtonElement)) throw new Error(`Missing menu check: ${name}`);
  return button;
}

function buttonLabeled(host: HTMLElement, name: string): HTMLButtonElement {
  const button = [...host.querySelectorAll("button")].find((candidate) => (
    candidate.getAttribute("aria-label") === name || candidate.getAttribute("title") === name || candidate.textContent === name
  ));
  if (!(button instanceof HTMLButtonElement)) throw new Error(`Missing labeled button: ${name}`);
  return button;
}

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
