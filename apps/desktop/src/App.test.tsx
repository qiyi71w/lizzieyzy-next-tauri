// @vitest-environment jsdom

import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { AnalysisFrameDto, ApplicationExitOutcomeDto, CurrentGameResultDto, DocumentDepartureAdmissionDto, GameDto, NodePath, RecoveryProtectionDto, RecoveryStartupDto } from "./domain/types";

const currentGameFixture = vi.hoisted(() => vi.fn());
const backend = vi.hoisted(() => ({
  getHealth: vi.fn(() => Promise.resolve({ status: "ok" })),
  prepareDocumentReplacement: vi.fn(async () => ({ status: "ready", departure_id: 1 })),
  prepareApplicationExit: vi.fn(async (): Promise<DocumentDepartureAdmissionDto> => ({ status: "ready", departure_id: 1 })),
  resolveApplicationExit: vi.fn(async (input: { action: string }): Promise<ApplicationExitOutcomeDto> => {
    if (input.action === "cancel") {
      return { committed: false, analysis_stopped: false, current: null, message: "Exit cancelled.", disposition: null, teardown: null };
    }
    const current = await currentGameFixture("", null) as CurrentGameResultDto;
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
  openSgfDocument: vi.fn(),
  parseSgfSummary: vi.fn(),
  replaySgfPositions: vi.fn(),
  saveCurrentGame: vi.fn(),
  setCurrentGamePersonalComment: vi.fn(),
  removeCurrentGameVariation: vi.fn(),
  startKataGoGameAnalysis: vi.fn(),
  loadEngineProfilesSettings: vi.fn(),
  subscribeForegroundEngine: vi.fn(),
  startForegroundEngine: vi.fn(),
  stopForegroundEngine: vi.fn(),
  restartForegroundEngine: vi.fn(),
  switchForegroundEngine: vi.fn(),
  getForegroundEngineSnapshot: vi.fn(() => Promise.resolve({ revision: 0, lifecycle: { state: "no_engine" }, continuous: { enabled: null, phase: "loading" } })),
  foregroundEngineContinuousAction: vi.fn(),
  inspectCurrentGameRecovery: vi.fn(async (): Promise<RecoveryStartupDto> => ({ status: "none" })),
  restoreCurrentGameRecovery: vi.fn(),
  discardCurrentGameRecovery: vi.fn(async () => undefined),
  retryCurrentGameRecovery: vi.fn(async (): Promise<RecoveryProtectionDto> => ({ status: "protected" })),
  currentGameRecoveryProtection: vi.fn(async (): Promise<RecoveryProtectionDto> => ({ status: "protected" })),
  subscribeCurrentGameRecoveryProtection: vi.fn(async (_onProtection: (protection: RecoveryProtectionDto) => void) => () => undefined)
}));

const listeners: {
  onSnapshot?: (snapshot: unknown) => void;
  onJob?: (job: unknown) => void;
} = {};

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
import { defaultAppPreferences } from "./domain/preferences";

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
  summary: { id: "test", board_size: 9, komi: 7.5, move_count: 0 },
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
  snapshot_seq: 1,
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
  snapshot_seq: 1,
  dirty: true,
  native_path: "/tmp/review.sgf"
};

let root: Root | null = null;

beforeEach(() => {
  globalThis.IS_REACT_ACT_ENVIRONMENT = true;
  vi.spyOn(HTMLCanvasElement.prototype, "getContext").mockImplementation(function (this: HTMLCanvasElement) {
    return canvasContext(this);
  });
  currentGameFixture.mockResolvedValue(initialGame);
  backend.projectCurrentGameMainline.mockResolvedValue(initialProjection);
  backend.foregroundEngineContinuousAction.mockResolvedValue(defaultAppPreferences);
  backend.startForegroundEngine.mockResolvedValue(undefined);
  backend.startSelectedNodeAnalysis.mockResolvedValue({
    run_id: "run-1",
    job_id: "job-1",
    lane: "selected_node",
    mode: "finite",
    state: "queued",
    generation: 1,
    node_path: { indices: [] }
  });
  backend.loadEngineProfilesSettings.mockResolvedValue({
    selected_profile_id: "profile-1",
    autoload_profile_id: null,
    profiles: [savedProfile]
  });
  backend.subscribeForegroundEngine.mockImplementation(async (
    onSnapshot: (snapshot: unknown) => void,
    _onFailure: unknown,
    onJob?: (job: unknown) => void
  ) => {
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
      await backend.inspectCurrentGameRecovery.mock.results.at(-1)?.value;
      await currentGameFixture.mock.results[0]?.value;
      await backend.projectCurrentGameMainline.mock.results[0]?.value;
    });

    act(() => buttonNamed(host, "键盘落子").click());
    const canvas = requiredElement(host, 'canvas[aria-label="棋盘"]');
    expect(document.activeElement).toBe(canvas);
    expect(buttonNamed(host, "键盘落子").getAttribute("aria-pressed")).toBe("true");

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
    backend.classifyProblems.mockResolvedValue([]);
    const host = await renderApp();
    await readyEngine(host);
    await startSelectedNode(host);
    await completeSelectedNode("job-1", { indices: [] }, analysisFrame);
    expect(host.querySelectorAll(".cand-row")).toHaveLength(2);
    const selectCalls = backend.selectCurrentGameNode.mock.calls.length;

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
    expect(backend.selectCurrentGameNode).toHaveBeenCalledTimes(selectCalls);
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
    currentGameFixture.mockResolvedValue(navigableRoot);
    backend.selectCurrentGameNode.mockImplementation(async (path: NodePath) => (
      path.indices.length === 0 ? navigableRoot : navigableChild
    ));
    backend.classifyProblems.mockResolvedValue([{
      turn: 1,
      severity: "mistake",
      winrate_loss: 0.12,
      score_loss: 3,
      label: "恶手"
    }]);

    const host = await renderApp();
    await readyEngine(host);
    await startSelectedNode(host);
    await completeSelectedNode("job-1", { indices: [] }, analysisFrame);
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
      await backend.selectCurrentGameNode.mock.results.at(-1)?.value;
    });

    expect(requiredElement<HTMLInputElement>(host, 'input[aria-label="跳转手数"]').value).toBe("1");
    expect(candidateCoords(host)).toEqual([]);
    expect(host.querySelector(".cand-row.is-selected")).toBeNull();
    expect(buttonNamed(host, "问题手 (0)")).toBeTruthy();
  });

  it("ignores a late completion after the selected NodePath has already changed", async () => {
    currentGameFixture.mockResolvedValue(navigableRoot);
    backend.selectCurrentGameNode.mockResolvedValue(navigableChild);
    backend.classifyProblems.mockResolvedValue([]);

    const host = await renderApp();
    await readyEngine(host);
    await startSelectedNode(host, "job-1");

    await act(async () => {
      buttonNamed(host, "下一变化").click();
      await backend.selectCurrentGameNode.mock.results.at(-1)?.value;
    });
    expect(requiredElement<HTMLInputElement>(host, 'input[aria-label="跳转手数"]').value).toBe("1");

    await completeSelectedNode("job-1", { indices: [] }, analysisFrame);

    expect(candidateCoords(host)).toEqual([]);
    expect(requiredElement<HTMLInputElement>(host, 'input[aria-label="跳转手数"]').value).toBe("1");
  });

  it("keeps the superseding request and ignores the earlier completion", async () => {
    backend.classifyProblems.mockResolvedValue([]);
    const host = await renderApp();
    await readyEngine(host);
    await startSelectedNode(host, "stale-job");
    await startSelectedNode(host, "current-job");
    await completeSelectedNode("current-job", { indices: [] }, currentAnalysisFrame);
    expect(candidateCoords(host)).toEqual(["J1"]);

    await completeSelectedNode("stale-job", { indices: [] }, staleAnalysisFrame);
    expect(candidateCoords(host)).toEqual(["J1"]);
  });

  it("clears presentation on game replacement before a late completion can return", async () => {
    backend.classifyProblems.mockResolvedValue([]);
    currentGameFixture
      .mockResolvedValueOnce(initialGame)
      .mockResolvedValueOnce({ ...initialGame, generation: 4 });

    const host = await renderApp();
    await readyEngine(host);
    await startSelectedNode(host, "job-1");

    await act(async () => {
      buttonNamed(host, "新对局").click();
      await currentGameFixture.mock.results.at(-1)?.value;
      await backend.projectCurrentGameMainline.mock.results.at(-1)?.value;
    });
    expect(requiredElement<HTMLInputElement>(host, 'input[aria-label="跳转手数"]').value).toBe("0");
    expect(candidateCoords(host)).toEqual([]);

    await completeSelectedNode("job-1", { indices: [] }, analysisFrame);
    expect(candidateCoords(host)).toEqual([]);
  });

  it("clears presentation when a move mutates the current game", async () => {
    backend.classifyProblems.mockResolvedValue([]);
    backend.playCurrentGame.mockResolvedValue(acceptedGame);

    const host = await renderApp();
    await readyEngine(host);
    await startSelectedNode(host);
    await completeSelectedNode("job-1", { indices: [] }, analysisFrame);
    expect(candidateCoords(host)).toEqual(["C6", "G4"]);

    act(() => buttonNamed(host, "键盘落子").click());
    const canvas = requiredElement(host, 'canvas[aria-label="棋盘"]');
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
    currentGameFixture.mockResolvedValue(branchingGame);
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

  it("keeps review shortcuts on the board while isolating text, select, contenteditable, and IME", async () => {
    const host = await renderApp();
    const canvas = requiredElement(host, 'canvas[aria-label="棋盘"]');
    const coordinates = buttonNamed(host, "坐标");
    const jump = requiredElement<HTMLInputElement>(host, 'input[aria-label="跳转手数"]');

    backend.selectCurrentGameNode.mockClear();
    backend.playCurrentGame.mockClear();
    backend.saveCurrentGame.mockClear();
    act(() => canvas.focus());
    pressKey(canvas, "p");
    await flushLast(backend.playCurrentGame);
    expect(backend.playCurrentGame).toHaveBeenLastCalledWith({ indices: [0, 1] }, "pass");
    pressKey(canvas, "s", { ctrlKey: true });
    await flushLast(backend.saveCurrentGame);
    expect(backend.saveCurrentGame).toHaveBeenCalledTimes(1);
    pressKey(canvas, "ArrowUp");
    await flushLast(backend.selectCurrentGameNode);
    expect(backend.selectCurrentGameNode).toHaveBeenLastCalledWith({ indices: [0] });
    pressKey(canvas, "c");
    expect(coordinates.getAttribute("aria-pressed")).toBe("false");
    pressKey(canvas, "c");
    expect(coordinates.getAttribute("aria-pressed")).toBe("true");

    const composing = new KeyboardEvent("keydown", { key: "c", bubbles: true, cancelable: true });
    Object.defineProperty(composing, "isComposing", { value: true });
    act(() => canvas.dispatchEvent(composing));
    expect(coordinates.getAttribute("aria-pressed")).toBe("true");

    backend.selectCurrentGameNode.mockClear();
    act(() => jump.focus());
    pressKey(jump, "ArrowUp");
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
    pressKey(buttonNamed(hostLeft, "坐标"), "ArrowUp");
    await flushLast(backend.selectCurrentGameNode);
    expect(backend.selectCurrentGameNode).toHaveBeenLastCalledWith({ indices: [0] });

    const hostBoardUp = await renderApp();
    backend.selectCurrentGameNode.mockClear();
    const board = requiredElement(hostBoardUp, 'canvas[aria-label="棋盘"]');
    act(() => board.focus());
    pressKey(board, "ArrowUp");
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
    pressKey(buttonNamed(hostUpKey, "坐标"), "ArrowLeft");
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
    pressKey(buttonNamed(hostRightKey, "坐标"), "ArrowDown");
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
    pressKey(buttonNamed(hostDownKey, "坐标"), "ArrowRight");
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
    pressKey(buttonNamed(hostKeys, "坐标"), "c", { ctrlKey: true });
    await flushLast(backend.serializeCurrentGame);
    expect(backend.serializeCurrentGame).toHaveBeenCalled();

    currentGameFixture.mockResolvedValue({
      ...branchingGame,
      selected_path: { indices: [] },
      snapshot: {
        ...branchingGame.snapshot,
        path: { indices: [] },
        position: { ...emptyPosition, move_number: 0 }
      }
    });
    pressKey(buttonNamed(hostKeys, "坐标"), "v", { ctrlKey: true });
    await act(async () => {
      await Promise.resolve();
      await currentGameFixture.mock.results.at(-1)?.value;
      await backend.projectCurrentGameMainline.mock.results.at(-1)?.value;
    });
    expect(requiredElement<HTMLInputElement>(hostKeys, 'input[aria-label="跳转手数"]').value).toBe("0");

    pressKey(buttonNamed(hostKeys, "坐标"), "n");
    await act(async () => {
      await Promise.resolve();
    });
    expect(requiredElement(hostKeys, ".nav-message").textContent).toContain("人机对局尚未接入");
    expect(requiredElement<HTMLInputElement>(hostKeys, 'input[aria-label="跳转手数"]').value).toBe("0");

    currentGameFixture.mockResolvedValue(branchingGame);
    pressKey(buttonNamed(hostKeys, "坐标"), "Home", { ctrlKey: true });
    await act(async () => {
      await Promise.resolve();
      await currentGameFixture.mock.results.at(-1)?.value;
      await backend.projectCurrentGameMainline.mock.results.at(-1)?.value;
    });
    expect(requiredElement<HTMLInputElement>(hostKeys, 'input[aria-label="跳转手数"]').value).toBe("2");

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

    const hostSample = await renderApp();
    currentGameFixture.mockResolvedValue({
      ...branchingGame,
      selected_path: { indices: [] },
      snapshot: {
        ...branchingGame.snapshot,
        path: { indices: [] },
        position: { ...emptyPosition, move_number: 0 }
      }
    });
    act(() => buttonNamed(hostSample, "文件").click());
    act(() => buttonNamed(hostSample, "载入示例").click());
    await act(async () => {
      await Promise.resolve();
      await currentGameFixture.mock.results.at(-1)?.value;
      await backend.projectCurrentGameMainline.mock.results.at(-1)?.value;
    });
    expect(requiredElement<HTMLInputElement>(hostSample, 'input[aria-label="跳转手数"]').value).toBe("0");

    const hostImport = await renderApp();
    act(() => buttonNamed(hostImport, "文件").click());
    act(() => buttonNamed(hostImport, "导入棋谱…").click());
    expect(hostImport.querySelector('textarea[aria-label="棋谱载入文本"]')).not.toBeNull();

    const hostParse = await renderApp();
    currentGameFixture.mockResolvedValue({
      ...branchingGame,
      selected_path: { indices: [] },
      snapshot: {
        ...branchingGame.snapshot,
        path: { indices: [] },
        position: { ...emptyPosition, move_number: 0 }
      }
    });
    act(() => buttonNamed(hostParse, "棋局").click());
    act(() => buttonNamed(hostParse, "解析棋谱").click());
    await act(async () => {
      await Promise.resolve();
      await currentGameFixture.mock.results.at(-1)?.value;
      await backend.projectCurrentGameMainline.mock.results.at(-1)?.value;
    });
    expect(requiredElement<HTMLInputElement>(hostParse, 'input[aria-label="跳转手数"]').value).toBe("0");

    const hostRefresh = await renderApp();
    currentGameFixture.mockResolvedValue({
      ...branchingGame,
      selected_path: { indices: [] },
      snapshot: {
        ...branchingGame.snapshot,
        path: { indices: [] },
        position: { ...emptyPosition, move_number: 0 }
      }
    });
    act(() => buttonNamed(hostRefresh, "刷新").click());
    await act(async () => {
      await Promise.resolve();
      await currentGameFixture.mock.results.at(-1)?.value;
      await backend.projectCurrentGameMainline.mock.results.at(-1)?.value;
    });
    expect(requiredElement<HTMLInputElement>(hostRefresh, 'input[aria-label="跳转手数"]').value).toBe("0");

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
    expect(second?.classList.contains("is-selected")).toBe(true);

    const jump = requiredElement<HTMLInputElement>(host, 'input[aria-label="跳转手数"]');
    act(() => jump.focus());
    pressKey(jump, "2");
    expect(second?.classList.contains("is-selected")).toBe(true);
    expect(first?.classList.contains("is-selected")).toBe(false);

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

  it("keeps unavailable editing tools disabled while Space remains stone-placement safe", async () => {
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
    backend.playCurrentGame.mockClear();
    pressKey(buttonNamed(host, "坐标"), "a");
    pressKey(buttonNamed(host, "坐标"), " ");
    expect(backend.startSelectedNodeAnalysis).not.toHaveBeenCalled();
    expect(backend.startKataGoGameAnalysis).not.toHaveBeenCalled();
    expect(backend.playCurrentGame).not.toHaveBeenCalled();
  });

  it("opens the same searchable Shortcut Reference from Help and focus-safe ?", async () => {
    const host = await renderApp();
    pressKey(buttonNamed(host, "坐标"), "?", { shiftKey: true });
    const dialog = requiredElement(host, '[role="dialog"][aria-label="快捷键参考"]');
    expect(dialog.textContent).toContain("自动播放");
    expect(dialog.textContent).toContain("Ctrl+A");
    expect(dialog.textContent).toContain("快捷键参考");
    expect(dialog.textContent).toContain("?");
    expect(dialog.textContent).toContain("上一手");
    expect(dialog.textContent).toContain("Up");
    expect(dialog.textContent).toContain("下一分支");
    expect(dialog.textContent).toContain("Right");
    expect(dialog.textContent).toContain("新建");
    expect(dialog.textContent).toContain("Ctrl+Home");
    expect(dialog.textContent).not.toContain("N, Ctrl+Home");
    expect(dialog.textContent).toContain("人机对局（未接入）");
    expect(dialog.textContent).toContain("Space");

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

  it("uses explicit keyboard placement for arrows and Enter and never submits Space", async () => {
    const host = await renderApp();
    const toggle = buttonNamed(host, "键盘落子");
    expect(toggle.getAttribute("aria-pressed")).toBe("false");
    backend.playCurrentGame.mockClear();
    backend.selectCurrentGameNode.mockClear();

    act(() => toggle.click());
    expect(toggle.getAttribute("aria-pressed")).toBe("true");
    const canvas = requiredElement(host, 'canvas[aria-label="棋盘"]');
    expect(document.activeElement).toBe(canvas);

    pressKey(canvas, "ArrowRight");
    expect(backend.selectCurrentGameNode).not.toHaveBeenCalled();
    pressKey(canvas, " ");
    expect(backend.playCurrentGame).not.toHaveBeenCalled();
    pressKey(canvas, "Enter", { shiftKey: true });
    expect(backend.playCurrentGame).not.toHaveBeenCalled();

    pressKey(canvas, "s", { ctrlKey: true });
    await flushLast(backend.saveCurrentGame);
    expect(backend.saveCurrentGame).toHaveBeenCalled();

    pressKey(canvas, "Enter");
    await flushLast(backend.playCurrentGame);
    expect(backend.playCurrentGame).toHaveBeenCalled();

    pressKey(canvas, "Escape");
    expect(buttonNamed(host, "键盘落子").getAttribute("aria-pressed")).toBe("false");
    backend.playCurrentGame.mockClear();
    pressKey(canvas, "Enter");
    expect(backend.playCurrentGame).not.toHaveBeenCalled();
  });

  it("rejects an illegal keyboard placement and leaves the tree unchanged", async () => {
    backend.playCurrentGame.mockRejectedValueOnce({ kind: "occupied_point", message: "point is occupied" });
    const host = await renderApp();
    act(() => buttonNamed(host, "键盘落子").click());
    const canvas = requiredElement(host, 'canvas[aria-label="棋盘"]');
    const moveField = requiredElement<HTMLInputElement>(host, 'input[aria-label="跳转手数"]');
    const before = moveField.value;
    pressKey(canvas, "Enter");
    await act(async () => {
      await Promise.resolve();
      const last = backend.playCurrentGame.mock.results.at(-1)?.value;
      if (last && typeof (last as Promise<unknown>).then === "function") {
        await (last as Promise<unknown>).catch(() => undefined);
      }
    });
    expect(requiredElement(host, ".board-intent-status").textContent).toContain("落子失败: point is occupied");
    expect(moveField.value).toBe(before);
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

  it("exits keyboard placement after a committed document replacement", async () => {
    const host = await renderApp();
    act(() => buttonNamed(host, "键盘落子").click());
    expect(buttonNamed(host, "键盘落子").getAttribute("aria-pressed")).toBe("true");
    backend.prepareDocumentReplacement.mockClear();
    backend.prepareDocumentReplacement.mockResolvedValue({ status: "ready", departure_id: 9 });
    await act(async () => {
      buttonLabeled(host, "新建").click();
      await backend.prepareDocumentReplacement.mock.results.at(-1)?.value;
      await backend.resolveDocumentReplacement.mock.results.at(-1)?.value;
    });
    expect(buttonNamed(host, "键盘落子").getAttribute("aria-pressed")).toBe("false");
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


describe("App current-game recovery", () => {
  const recoveredGame: CurrentGameResultDto = {
    ...initialGame,
    dirty: true,
    native_path: "/tmp/recovered.sgf",
    snapshot: { ...initialGame.snapshot, personal_comment: "restored personal" }
  };

  it("prompts Restore/Discard for an abnormal snapshot before sample load", async () => {
    backend.inspectCurrentGameRecovery.mockResolvedValue({
      status: "abnormal",
      envelope: {
        document_seq: 1,
        snapshot_seq: 2,
        sgf_text: "(;SZ[9])",
        selected_path: { indices: [] },
        source_path: "/tmp/recovered.sgf",
        dirty: true,
        disposition: "exit_incomplete"
      }
    });
    act(() => root?.unmount());
    const host = document.createElement("div");
    document.body.append(host);
    root = createRoot(host);
    act(() => root?.render(<App />));
    await act(async () => {
      await preferencesApi.loadAppPreferences.mock.results.at(-1)?.value.catch(() => undefined);
      await backend.inspectCurrentGameRecovery.mock.results.at(-1)?.value;
    });
    expect(host.querySelector('[role="dialog"][aria-label="恢复当前棋谱"]')).not.toBeNull();
    expect(backend.prepareDocumentReplacement).not.toHaveBeenCalled();
  });

  it("discards the abnormal candidate then loads the sample", async () => {
    backend.inspectCurrentGameRecovery.mockResolvedValue({
      status: "abnormal",
      envelope: {
        document_seq: 1,
        snapshot_seq: 2,
        sgf_text: "(;SZ[9])",
        selected_path: { indices: [] },
        dirty: true,
        disposition: "exit_incomplete"
      }
    });
    act(() => root?.unmount());
    const host = document.createElement("div");
    document.body.append(host);
    root = createRoot(host);
    act(() => root?.render(<App />));
    await act(async () => {
      await backend.inspectCurrentGameRecovery.mock.results.at(-1)?.value;
    });
    await act(async () => {
      buttonLabeled(host, "Discard").click();
      await backend.discardCurrentGameRecovery.mock.results.at(-1)?.value;
      await currentGameFixture.mock.results.at(-1)?.value;
      await backend.projectCurrentGameMainline.mock.results.at(-1)?.value;
    });
    expect(backend.discardCurrentGameRecovery).toHaveBeenCalled();
    expect(host.querySelector('[role="dialog"][aria-label="恢复当前棋谱"]')).toBeNull();
    expect(backend.prepareDocumentReplacement).toHaveBeenCalled();
  });

  it("restores the abnormal document without loading the sample", async () => {
    backend.inspectCurrentGameRecovery.mockResolvedValue({
      status: "abnormal",
      envelope: {
        document_seq: 1,
        snapshot_seq: 2,
        sgf_text: "(;SZ[9];B[pd])",
        selected_path: { indices: [0] },
        source_path: "/tmp/recovered.sgf",
        dirty: true,
        disposition: "exit_incomplete"
      }
    });
    backend.restoreCurrentGameRecovery.mockResolvedValue(recoveredGame);
    backend.serializeCurrentGame.mockResolvedValue("(;SZ[9];B[pd])");
    act(() => root?.unmount());
    const host = document.createElement("div");
    document.body.append(host);
    root = createRoot(host);
    act(() => root?.render(<App />));
    await act(async () => {
      await backend.inspectCurrentGameRecovery.mock.results.at(-1)?.value;
    });
    backend.prepareDocumentReplacement.mockClear();
    await act(async () => {
      buttonLabeled(host, "Restore").click();
      await backend.restoreCurrentGameRecovery.mock.results.at(-1)?.value;
      await backend.serializeCurrentGame.mock.results.at(-1)?.value;
      await backend.projectCurrentGameMainline.mock.results.at(-1)?.value;
    });
    expect(backend.restoreCurrentGameRecovery).toHaveBeenCalled();
    expect(backend.prepareDocumentReplacement).not.toHaveBeenCalled();
    expect(host.textContent).toContain("已恢复上次未正常退出的棋谱");
  });

  it("keeps the current game when restore validation fails", async () => {
    backend.inspectCurrentGameRecovery.mockResolvedValue({
      status: "abnormal",
      envelope: {
        document_seq: 1,
        snapshot_seq: 1,
        sgf_text: "not-sgf",
        selected_path: { indices: [] },
        dirty: true,
        disposition: "exit_incomplete"
      }
    });
    backend.restoreCurrentGameRecovery.mockRejectedValue({ kind: "malformed_sgf", message: "malformed SGF" });
    act(() => root?.unmount());
    const host = document.createElement("div");
    document.body.append(host);
    root = createRoot(host);
    act(() => root?.render(<App />));
    await act(async () => {
      await backend.inspectCurrentGameRecovery.mock.results.at(-1)?.value;
    });
    await act(async () => {
      buttonLabeled(host, "Restore").click();
      await backend.restoreCurrentGameRecovery.mock.results.at(-1)?.value.catch(() => undefined);
    });
    expect(host.querySelector('[role="dialog"][aria-label="恢复当前棋谱"]')).not.toBeNull();
    expect(host.textContent).toContain("恢复失败");
    expect(backend.prepareDocumentReplacement).not.toHaveBeenCalled();
  });

  it("shows unprotected recovery status and retries the write", async () => {
    backend.subscribeCurrentGameRecoveryProtection.mockImplementation(async (onProtection: (protection: RecoveryProtectionDto) => void) => {
      onProtection({ status: "unprotected", message: "Recent current-game changes are not yet protected." });
      return () => undefined;
    });
    const host = await renderApp();
    expect(host.textContent).toContain("Recent current-game changes are not yet protected.");
    await act(async () => {
      buttonLabeled(host, "Retry recovery write").click();
      await backend.retryCurrentGameRecovery.mock.results.at(-1)?.value;
    });
    expect(backend.retryCurrentGameRecovery).toHaveBeenCalled();
  });

  it("keeps an unreadable recovery explanation after sample load", async () => {
    backend.inspectCurrentGameRecovery.mockResolvedValue({
      status: "unreadable",
      message: "Recovery snapshot is unreadable and was not applied."
    });
    const host = await renderApp();
    expect(host.querySelector('[role="dialog"][aria-label="恢复当前棋谱"]')).toBeNull();
    expect(host.textContent).toContain("Recovery snapshot is unreadable and was not applied.");
    expect(backend.prepareDocumentReplacement).toHaveBeenCalled();
  });

  it("does not confirm native exit when recovery persist fails", async () => {
    backend.resolveApplicationExit.mockResolvedValue({
      committed: true,
      analysis_stopped: true,
      current: initialGame,
      message: "Failed to persist current-game recovery: disk full",
      disposition: "clean_completed",
      teardown: { status: "completed" },
      recovery_persist_error: "disk full"
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
    await preferencesApi.loadAppPreferences.mock.results.at(-1)?.value.catch(() => undefined);
    await backend.inspectCurrentGameRecovery.mock.results.at(-1)?.value;
    await currentGameFixture.mock.results.at(-1)?.value;
    await backend.projectCurrentGameMainline.mock.results.at(-1)?.value;
    await backend.loadEngineProfilesSettings.mock.results.at(-1)?.value;
    await backend.subscribeForegroundEngine.mock.results.at(-1)?.value;
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
  currentGameFixture.mockResolvedValue({
    ...initialGame,
    snapshot: {
      ...initialGame.snapshot,
      primary_analysis: frame
    }
  });
}

describe("accepted navigation coalescing", () => {
  it.each([false, true])("preserves newer comment and Save state behind delayed navigation (save=%s)", async (save) => {
    const initial = { ...navigableRoot, dirty: save, native_path: "/tmp/original.sgf" };
    currentGameFixture.mockResolvedValue(initial);
    const host = await renderApp();
    let release!: (result: CurrentGameResultDto) => void;
    const pending = new Promise<CurrentGameResultDto>((resolve) => { release = resolve; });
    let authoritative: CurrentGameResultDto = initial;
    backend.selectCurrentGameNode.mockImplementation(async (path: NodePath) => ({
      ...authoritative, snapshot_seq: 5, selected_path: path,
      snapshot: path.indices.length === 0 ? authoritative.snapshot : navigableChild.snapshot
    })).mockReturnValueOnce(pending);
    act(() => requiredElement<HTMLButtonElement>(host, 'button[title="下一手"]').click());
    const editor = requiredElement<HTMLTextAreaElement>(host, 'textarea[aria-label="个人评论"]');
    authoritative = {
      ...initial, snapshot_seq: 3, dirty: true,
      tree: { ...initial.tree, properties: [{ key: "C", values: ["retained annotation"] }] },
      snapshot: { ...initial.snapshot, personal_comment: "retained annotation" }
    };
    backend.setCurrentGamePersonalComment.mockResolvedValue(authoritative);
    act(() => {
      editor.focus();
      Object.getOwnPropertyDescriptor(HTMLTextAreaElement.prototype, "value")?.set?.call(editor, "retained annotation");
      editor.dispatchEvent(new Event("input", { bubbles: true }));
    });
    await act(async () => {
      editor.blur();
      await backend.setCurrentGamePersonalComment.mock.results.at(-1)?.value;
    });
    if (save) {
      authoritative = { ...authoritative, snapshot_seq: 4, dirty: false, native_path: "/tmp/saved.sgf" };
      backend.saveCurrentGame.mockResolvedValue(authoritative);
      await act(async () => {
        buttonLabeled(host, "保存").click();
        await backend.saveCurrentGame.mock.results.at(-1)?.value;
      });
    }
    await act(async () => {
      release({ ...initial, snapshot_seq: 2, selected_path: navigableChild.selected_path, snapshot: navigableChild.snapshot });
      await pending;
      await Promise.resolve();
    });
    expect(requiredElement(host, ".doc-name").textContent).toBe(save ? "saved.sgf" : "original.sgf *");
    expect(requiredElement<HTMLInputElement>(host, 'input[aria-label="跳转手数"]').value).toBe("1");
    await act(async () => {
      requiredElement<HTMLButtonElement>(host, 'button[title="上一手"]').click();
      await backend.selectCurrentGameNode.mock.results.at(-1)?.value;
    });
    expect(editor.value).toBe("retained annotation");
  });

  it("shows only the latest path requested while an earlier native selection is pending", async () => {
    currentGameFixture.mockResolvedValue(branchingGame);
    let resolveFirstSelection!: (value: CurrentGameResultDto) => void;
    const firstSelection = new Promise<CurrentGameResultDto>((resolve) => { resolveFirstSelection = resolve; });
    const host = await renderApp();
    backend.selectCurrentGameNode.mockClear();
    backend.selectCurrentGameNode
      .mockReturnValueOnce(firstSelection)
      .mockResolvedValueOnce({ ...branchingGame, selected_path: initialGame.selected_path, snapshot: initialGame.snapshot });

    act(() => buttonNamed(host, "父节点").click());
    const jump = requiredElement<HTMLInputElement>(host, 'input[aria-label="跳转手数"]');
    act(() => {
      const valueSetter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")?.set;
      valueSetter?.call(jump, "0");
      jump.dispatchEvent(new Event("input", { bubbles: true }));
    });
    expect(backend.selectCurrentGameNode).toHaveBeenCalledTimes(1);

    await act(async () => {
      resolveFirstSelection({ ...branchingGame, selected_path: { indices: [0] }, snapshot: navigableChild.snapshot });
      await firstSelection;
      await Promise.resolve();
    });
    expect(backend.selectCurrentGameNode).toHaveBeenCalledTimes(2);
    expect(backend.selectCurrentGameNode).toHaveBeenLastCalledWith({ indices: [] });
    expect(jump.value).toBe("0");
    expect(backend.cancelSelectedNodeAnalysis).not.toHaveBeenCalled();
  });
});

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
            root_score: true,
            protocol_cancel: true
          }
        }
      }
    });
  });
}

async function startSelectedNode(host: HTMLElement, jobId = "job-1") {
  backend.startSelectedNodeAnalysis.mockResolvedValueOnce({
    run_id: "run-1",
    job_id: jobId,
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
}

async function completeSelectedNode(jobId: string, path: { indices: number[] }, frame: AnalysisFrameDto) {
  await act(async () => {
    listeners.onJob?.({
      run_id: "run-1",
      job_id: jobId,
      lane: "selected_node",
      mode: "finite",
      generation: 1,
      node_path: path,
      outcome: "completed",
      frame
    });
    await Promise.all([
      backend.classifyProblems.mock.results.at(-1)?.value,
      backend.selectCurrentGameNode.mock.results.at(-1)?.value
    ]);
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
