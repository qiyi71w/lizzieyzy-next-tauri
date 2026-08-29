import { useEffect, useMemo, useRef, useState } from "react";
import { BoardCanvas } from "./components/BoardCanvas";
import { WinrateChart } from "./components/WinrateChart";
import { AnalysisPanel } from "./components/AnalysisPanel";
import { EngineSetupPanel } from "./components/EngineSetupPanel";
import { CacheStatusBadge } from "./components/CacheStatusBadge";
import { AppChrome, BottomBar, type EngineCommands, type OverlayMode, type SheetId } from "./components/AppChrome";
import { PreferencesPanel } from "./components/PreferencesPanel";
import { ProviderPanel } from "./components/ProviderPanel";
import {
  analyzeKataGoOnce,
  cancelKataGoAnalysis,
  classifyProblems,
  fakeAnalyze,
  getHealth,
  isTauriRuntime,
  listenToKataGoAnalysisEvents,
  nativeCurrentGameUnavailable,
  openSgfDocument,
  parseSgfSummary,
  projectCurrentGameMainline,
  replaySgfPositions,
  replaceCurrentGame,
  saveSgfDocument,
  serializeCurrentGame,
  selectCurrentGameNode,
  startKataGoGameAnalysis
} from "./api/backend";
import { computeGameCacheKey, loadAnalysisCache, saveAnalysisCache } from "./api/analysisCache";
import { loadAppPreferences, saveAppPreferences } from "./api/preferences";
import { clampMoveNumberToPositions, createDemoGame, replayGamePositions, selectExactPosition } from "./domain/board";
import type { AnalysisCacheRecord, CacheStatus, GameCacheKey, JsonValue } from "./domain/cache";
import { defaultAppPreferences, normalizeAppPreferences, type AppPreferences } from "./domain/preferences";
import { providerDocumentName, providerLabel, providerSourceLabel, type ProviderImportResult } from "./domain/providers";
import type { AnalysisFrameDto, AppHealthDto, CurrentGameResultDto, EngineProfileDto, GameDto, NodePath, PositionDto, ProblemMarkerDto, SgfTreeNodeDto } from "./domain/types";

const demoSgf = "(;GM[1]FF[4]SZ[19]KM[7.5]PB[李昌镐]PW[芮乃伟]RE[B+R];B[pd];W[dd];B[pp];W[dp];B[jq];W[qj];B[nc];W[fc];B[qf];W[cn];B[cp];W[do];B[co];W[dn];B[fq];W[eq];B[fp];W[gp];B[gq];W[hp])";
const emptySgf = "(;GM[1]FF[4]SZ[19]KM[7.5]PB[黑]PW[白])";
const demoGame = createDemoGame();
type AnalysisProgress = { jobId: string; completed: number; expected: number; turn: number; responseJsonl: string };
type PendingAnalysisTerminalEvent =
  | { kind: "complete"; frames: AnalysisFrameDto[] }
  | { kind: "error" | "cancelled"; message: string };
type CacheEngineKind = "fake" | "katago";
type CachedAnalysisPayload = { frames: AnalysisFrameDto[]; problems: ProblemMarkerDto[] };
type PendingPreferencesSave = { version: number; preferences: AppPreferences };
type AnalysisCacheLoadResult =
  | { status: "hit"; record: AnalysisCacheRecord; engineKind: CacheEngineKind }
  | { status: "miss" }
  | { status: "error"; message: string };

export function App() {
  const [health, setHealth] = useState<AppHealthDto | null>(null);
  const [game, setGame] = useState<GameDto>(() => demoGame);
  const [positions, setPositions] = useState<PositionDto[]>(() => replayGamePositions(demoGame));
  const [currentMove, setCurrentMove] = useState(() => demoGame.moves.length);
  const [frames, setFrames] = useState<AnalysisFrameDto[]>([]);
  const [problems, setProblems] = useState<ProblemMarkerDto[]>([]);
  const [sgfText, setSgfText] = useState(demoSgf);
  const [message, setMessage] = useState(
    isTauriRuntime() ? "谱面已就绪。打开棋谱或载入示例开始复盘。" : nativeCurrentGameUnavailable
  );
  const nativeRuntime = isTauriRuntime();
  const [isKataGoRunning, setIsKataGoRunning] = useState(false);
  const [selectedCandidateIndex, setSelectedCandidateIndex] = useState<number | null>(null);
  const [currentFilePath, setCurrentFilePath] = useState<string | null>(null);
  const [fallbackFileName, setFallbackFileName] = useState<string | null>(null);
  const [dirty, setDirty] = useState(false);
  const [currentGame, setCurrentGame] = useState<CurrentGameResultDto | null>(null);
  const [chosenChildren, setChosenChildren] = useState<Map<string, number>>(() => new Map());
  const navigatingRef = useRef(false);
  const [analysisProgress, setAnalysisProgress] = useState<AnalysisProgress | null>(null);
  const [activeJobId, setActiveJobId] = useState<string | null>(null);
  const [cacheStatus, setCacheStatus] = useState<CacheStatus>("idle");
  const [cacheRecord, setCacheRecord] = useState<AnalysisCacheRecord | null>(null);
  const [cacheError, setCacheError] = useState<string | null>(null);
  const [currentCacheKey, setCurrentCacheKey] = useState<GameCacheKey | null>(null);
  const [preferences, setPreferences] = useState<AppPreferences>(() => defaultAppPreferences);
  const [preferencesStatus, setPreferencesStatus] = useState("正在载入设置…");
  const [sheet, setSheet] = useState<"none" | SheetId>("none");
  const engineCommandsRef = useRef<EngineCommands | null>(null);
  const [engineLabel, setEngineLabel] = useState("未加载引擎");
  const [engineReady, setEngineReady] = useState(false);
  const [showCoordinates, setShowCoordinates] = useState(true);
  const [showMoveNumbers, setShowMoveNumbers] = useState(false);
  const [showBlackCandidates, setShowBlackCandidates] = useState(true);
  const [showWhiteCandidates, setShowWhiteCandidates] = useState(true);
  const [overlayMode, setOverlayMode] = useState<OverlayMode>("candidates");
  const [autoPlaying, setAutoPlaying] = useState(false);
  const jumpRef = useRef<HTMLInputElement | null>(null);
  const activeJobIdRef = useRef<string | null>(null);
  const startingAnalysisRef = useRef(false);
  const userChangedPreferencesRef = useRef(false);
  const preferencesSaveInFlightRef = useRef(false);
  const preferencesSaveVersionRef = useRef(0);
  const pendingPreferencesSaveRef = useRef<PendingPreferencesSave | null>(null);
  const pendingAnalysisProgressRef = useRef<Map<string, AnalysisProgress>>(new Map());
  const pendingAnalysisTerminalEventsRef = useRef<Map<string, PendingAnalysisTerminalEvent>>(new Map());
  const analysisCleanupRef = useRef<(() => void) | null>(null);

  useEffect(() => {
    getHealth()
      .then(setHealth)
      .catch((error: unknown) => setMessage(errorMessage(error)));
  }, []);

  useEffect(() => {
    if (!isTauriRuntime()) return;
    void applyReplacement(demoSgf, null, {
      confirmMessage: "放弃未保存的棋谱并载入示例？",
      fallbackName: "sample.sgf",
      successMessage: (projection) => `Sample SGF restored: ${projection.summary.move_count} moves.`,
      failurePrefix: "Sample load failed"
    });
  }, []);

  useEffect(() => {
    let isMounted = true;
    loadAppPreferences()
      .then((loaded) => {
        if (!isMounted || userChangedPreferencesRef.current) return;
        setPreferences(loaded);
        setPreferencesStatus("Preferences loaded.");
      })
      .catch((error: unknown) => {
        if (isMounted && !userChangedPreferencesRef.current) setPreferencesStatus(`Load failed: ${errorMessage(error)}`);
      });
    return () => {
      isMounted = false;
    };
  }, []);

  useEffect(() => {
    return () => cleanupAnalysisListeners();
  }, []);

  const currentFrame = useMemo(() => frames.find((f) => f.turn === currentMove) ?? frames.at(-1), [frames, currentMove]);
  const visibleCurrentFrame = useMemo(() => applyPreferencesToFrame(currentFrame, preferences), [currentFrame, preferences]);
  const currentPosition = useMemo(() => {
    if (currentGame) return currentGame.snapshot.position;
    return selectExactPosition(positions, currentMove, game.summary.board_size);
  }, [currentGame, currentMove, positions, game.summary.board_size]);
  const selectedPersonalComment = currentGame ? currentGame.snapshot.personal_comment : "";
  const selectedPath = currentGame?.selected_path ?? { indices: [] };
  const selectedNode = currentGame ? nodeAt(currentGame.tree, selectedPath) : null;
  const parentOfSelected = parentPath(selectedPath);
  const parentNode = currentGame && parentOfSelected ? nodeAt(currentGame.tree, parentOfSelected) : null;
  const siblingIndex = selectedPath.indices.at(-1);
  const canParent = Boolean(currentGame && parentOfSelected);
  const canNext = Boolean(selectedNode && selectedNode.children.length > 0);
  const canPrevSibling = Boolean(parentNode && siblingIndex !== undefined && siblingIndex > 0);
  const canNextSibling = Boolean(parentNode && siblingIndex !== undefined && siblingIndex + 1 < parentNode.children.length);
  const siblingLabel = parentNode && siblingIndex !== undefined ? `${siblingIndex + 1}/${parentNode.children.length}` : "—";
  const maxMove = Math.max(positions.at(-1)?.move_number ?? 0, 1);
  const reviewIndex = currentGame ? selectedPath.indices.length : currentMove;
  const reviewMax = currentGame
    ? chosenLeafPath(currentGame.tree, { indices: [] }, chosenChildren).indices.length
    : maxMove;
  const documentName = useMemo(() => currentFilePath ? fileNameFromPath(currentFilePath) : fallbackFileName ?? "未命名棋谱", [currentFilePath, fallbackFileName]);
  const saveFileName = documentName.toLowerCase().endsWith(".sgf") ? documentName : `${documentName}.sgf`;

  useEffect(() => {
    setSelectedCandidateIndex(null);
  }, [currentMove]);

  useEffect(() => {
    if (!preferences.showCandidates || (selectedCandidateIndex !== null && selectedCandidateIndex >= preferences.candidateLimit)) {
      setSelectedCandidateIndex(null);
    }
  }, [preferences.showCandidates, preferences.candidateLimit, selectedCandidateIndex]);

  useEffect(() => {
    function onKey(event: KeyboardEvent) {
      const target = event.target;
      if (target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement || target instanceof HTMLSelectElement) return;
      if (event.key >= "1" && event.key <= "9") {
        const index = Number(event.key) - 1;
        if (visibleCurrentFrame?.candidates[index]) setSelectedCandidateIndex(index);
        return;
      }
      if (event.key === "ArrowLeft") {
        event.preventDefault();
        if (currentGame) {
          const parent = parentPath(currentGame.selected_path);
          if (parent) void selectNode(parent);
          return;
        }
        setCurrentMove((move) => clampMoveNumberToPositions(positions, move - 1));
      }
      if (event.key === "ArrowRight") {
        event.preventDefault();
        if (currentGame) {
          const node = nodeAt(currentGame.tree, currentGame.selected_path);
          if (node && node.children.length > 0) {
            void selectNode(childPath(
              currentGame.selected_path,
              chosenChildIndex(chosenChildren, currentGame.selected_path, node.children.length)
            ));
          }
          return;
        }
        setCurrentMove((move) => clampMoveNumberToPositions(positions, move + 1));
      }
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [positions, visibleCurrentFrame, currentGame, chosenChildren]);

  useEffect(() => {
    if (!autoPlaying) return;
    const timer = window.setInterval(() => {
      if (currentGame) {
        const node = nodeAt(currentGame.tree, currentGame.selected_path);
        if (!node || node.children.length === 0) {
          setAutoPlaying(false);
          return;
        }
        void selectNode(childPath(
          currentGame.selected_path,
          chosenChildIndex(chosenChildren, currentGame.selected_path, node.children.length)
        ));
        return;
      }
      setCurrentMove((move) => {
        const next = clampMoveNumberToPositions(positions, move + 1);
        if (next >= Math.max(positions.at(-1)?.move_number ?? 0, 1)) setAutoPlaying(false);
        return next;
      });
    }, 800);
    return () => window.clearInterval(timer);
  }, [autoPlaying, positions, currentGame, chosenChildren]);

  function toggleSheet(next: SheetId) {
    setSheet((current) => current === next ? "none" : next);
  }

  function bindEngineCommands(commands: EngineCommands) {
    engineCommandsRef.current = commands;
    setEngineLabel((current) => current === commands.engineLabel ? current : commands.engineLabel);
    setEngineReady((current) => current === commands.canRun ? current : commands.canRun);
  }

  function handleEngineCommand(kind: "once" | "game") {
    if (kind === "once") engineCommandsRef.current?.analyzeOnce();
    else engineCommandsRef.current?.analyzeGame();
  }

  function handlePreferencesChange(nextPreferences: AppPreferences) {
    const normalized = normalizeAppPreferences(nextPreferences);
    userChangedPreferencesRef.current = true;
    pendingPreferencesSaveRef.current = {
      version: preferencesSaveVersionRef.current + 1,
      preferences: normalized
    };
    preferencesSaveVersionRef.current = pendingPreferencesSaveRef.current.version;
    setPreferences(normalized);
    setPreferencesStatus("Saving preferences...");
    void runPreferencesSaveLoop();
  }

  async function runPreferencesSaveLoop() {
    if (preferencesSaveInFlightRef.current) return;
    preferencesSaveInFlightRef.current = true;
    try {
      while (pendingPreferencesSaveRef.current) {
        const pending = pendingPreferencesSaveRef.current;
        try {
          await saveAppPreferences(pending.preferences);
        } catch (error) {
          if (pendingPreferencesSaveRef.current?.version === pending.version) {
            setPreferencesStatus(`Save failed: ${errorMessage(error)}`);
            return;
          }
          setPreferencesStatus("Saving preferences...");
          continue;
        }

        if (pendingPreferencesSaveRef.current?.version === pending.version) {
          pendingPreferencesSaveRef.current = null;
          setPreferencesStatus("Preferences saved.");
          return;
        }
        setPreferencesStatus("Saving preferences...");
      }
    } finally {
      preferencesSaveInFlightRef.current = false;
    }
  }


  function confirmDirtyReplacement(confirmMessage: string): boolean {
    return !dirty || window.confirm(confirmMessage);
  }

  async function artifactsFromCurrentGame(): Promise<{ serialized: string; projection: GameDto; replayed: PositionDto[] }> {
    const [serialized, projection] = await Promise.all([serializeCurrentGame(), projectCurrentGameMainline()]);
    return { serialized, projection, replayed: await replaySgfPositions(serialized) };
  }

  async function applyReplacement(
    sgfInput: string,
    nativePath: string | null,
    options: {
      confirmMessage: string;
      fallbackName?: string | null;
      successMessage: (projection: GameDto, fileName: string) => string;
      failurePrefix: string;
      checkCache?: boolean;
    }
  ): Promise<boolean> {
    if (!confirmDirtyReplacement(options.confirmMessage)) return false;

    if (!nativeRuntime) {
      try {
        const [parsed, replayed] = await Promise.all([parseSgfSummary(sgfInput), replaySgfPositions(sgfInput)]);
        setCurrentGame(null);
        setSgfText(sgfInput);
        setCurrentFilePath(null);
        setFallbackFileName(options.fallbackName ?? null);
        setDirty(false);
        setGame(parsed);
        setPositions(replayed);
        setCurrentMove(replayed.at(-1)?.move_number ?? parsed.moves.length);
        setFrames([]);
        setProblems([]);
        setSelectedCandidateIndex(null);
        const previewMessage = options.successMessage(parsed, options.fallbackName ?? "SGF");
        setMessage(`${nativeCurrentGameUnavailable} ${previewMessage}`);
        if (options.checkCache === false) resetAnalysisCacheState();
        else await checkAnalysisCacheForGame(sgfInput, null, parsed, replayed, `${nativeCurrentGameUnavailable} ${previewMessage}`);
        return true;
      } catch (error) {
        setMessage(`${options.failurePrefix}: ${errorMessage(error)}`);
        return false;
      }
    }

    try {
      const result = await replaceCurrentGame(sgfInput, nativePath);
      const artifacts = await artifactsFromCurrentGame();
      setCurrentGame(result);
      setChosenChildren(chosenFromPath(result.selected_path));
      setSgfText(artifacts.serialized);
      setCurrentFilePath(result.native_path ?? nativePath);
      setFallbackFileName(result.native_path ? null : options.fallbackName ?? null);
      setDirty(result.dirty);
      setGame(artifacts.projection);
      setPositions(artifacts.replayed);
      setCurrentMove(result.snapshot.position.move_number);
      setFrames([]);
      setProblems([]);
      setSelectedCandidateIndex(null);
      const fileName = fileNameFromPath(result.native_path ?? options.fallbackName ?? "SGF");
      const success = options.successMessage(artifacts.projection, fileName);
      setMessage(success);
      if (options.checkCache === false) resetAnalysisCacheState();
      else await checkAnalysisCacheForGame(artifacts.serialized, result.native_path ?? null, artifacts.projection, artifacts.replayed, success);
      return true;
    } catch (error) {
      setMessage(`${options.failurePrefix}: ${errorMessage(error)}`);
      return false;
    }
  }

  async function handleParseSgf() {
    await applyReplacement(sgfText, null, {
      confirmMessage: "放弃未保存的棋谱并载入这段文本？",
      fallbackName: fallbackFileName ?? "imported.sgf",
      successMessage: (projection) =>
        `Loaded ${projection.summary.black_name ?? "Black"} vs ${projection.summary.white_name ?? "White"}: ${projection.summary.move_count} moves.`,
      failurePrefix: "Parse failed"
    });
  }

  async function handleOpenSgfDocument() {
    if (!nativeRuntime) {
      setMessage(nativeCurrentGameUnavailable);
      return;
    }
    if (!confirmDirtyReplacement("Discard unsaved SGF changes and open another file?")) return;
    try {
      const document = await openSgfDocument();
      if (!document) return;
      const result = await replaceCurrentGame(document.sgfText, document.path);
      const artifacts = await artifactsFromCurrentGame();
      setCurrentGame(result);
      setSgfText(artifacts.serialized);
      setCurrentFilePath(result.native_path ?? document.path);
      setFallbackFileName(null);
      setDirty(result.dirty);
      setGame(artifacts.projection);
      setPositions(artifacts.replayed);
      setCurrentMove(result.snapshot.position.move_number);
      setFrames([]);
      setProblems([]);
      setSelectedCandidateIndex(null);
      const openedMessage = `Opened ${fileNameFromPath(result.native_path ?? document.path ?? "SGF")}: ${artifacts.projection.summary.move_count} moves.`;
      setMessage(openedMessage);
      await checkAnalysisCacheForGame(artifacts.serialized, result.native_path ?? document.path, artifacts.projection, artifacts.replayed, openedMessage);
    } catch (error) {
      setMessage(`Open failed: ${errorMessage(error)}`);
    }
  }

  async function handleSaveSgfDocument(saveAs = false) {
    if (!nativeRuntime) {
      setMessage(nativeCurrentGameUnavailable);
      return;
    }
    try {
      const serialized = await serializeCurrentGame();
      const saved = await saveSgfDocument(saveAs ? null : currentFilePath, serialized, saveFileName);
      if (!saved) {
        setMessage("Save cancelled.");
        return;
      }
      setCurrentFilePath(saved.path);
      setDirty(false);
      setMessage(`Saved ${saved.path ? fileNameFromPath(saved.path) : saveFileName}.`);
    } catch (error) {
      setMessage(`Save failed: ${errorMessage(error)}`);
    }
  }

  async function handleFakeAnalyze() {
    try {
      if (!nativeRuntime) {
        const [parsed, result, replayed] = await Promise.all([parseSgfSummary(sgfText), fakeAnalyze(sgfText), replaySgfPositions(sgfText)]);
        const classified = await classifyProblems(result);
        setGame(parsed);
        setPositions(replayed);
        setFrames(result);
        setProblems(classified);
        setCurrentMove(replayed.at(-1)?.move_number ?? parsed.moves.length);
        setSelectedCandidateIndex(null);
        const cacheMessage = await saveAnalysisCacheForGame(sgfText, currentFilePath, parsed, result, classified, "fake");
        setMessage(`${nativeCurrentGameUnavailable} 已生成 ${result.length} 个预览复盘局面。${cacheMessage}`);
        return;
      }
      const artifacts = await artifactsFromCurrentGame();
      const result = await fakeAnalyze(artifacts.serialized);
      const classified = await classifyProblems(result);
      setGame(artifacts.projection);
      setPositions(artifacts.replayed);
      setFrames(result);
      setProblems(classified);
      setCurrentMove(artifacts.replayed.at(-1)?.move_number ?? artifacts.projection.moves.length);
      setSelectedCandidateIndex(null);
      const cacheMessage = await saveAnalysisCacheForGame(artifacts.serialized, currentFilePath, artifacts.projection, result, classified, "fake");
      setMessage(`已生成 ${result.length} 个复盘局面，含候选与胜率。${cacheMessage}`);
    } catch (error) {
      setMessage(errorMessage(error));
    }
  }

  async function handleRunKataGo(profile: EngineProfileDto, maxVisits: number) {
    const targetTurn = currentMove;
    const visits = resolveAnalysisMaxVisits(maxVisits, preferences);
    setIsKataGoRunning(true);
    setMessage(`Running KataGo analysis for move ${targetTurn}...`);
    try {
      const artifacts = nativeRuntime
        ? await artifactsFromCurrentGame()
        : { serialized: sgfText, projection: await parseSgfSummary(sgfText), replayed: await replaySgfPositions(sgfText) };
      if (!nativeRuntime) setMessage(nativeCurrentGameUnavailable);
      const turn = clampMoveNumberToPositions(artifacts.replayed, Math.min(targetTurn, artifacts.replayed.at(-1)?.move_number ?? artifacts.projection.moves.length));
      const frame = await analyzeKataGoOnce(profile, artifacts.serialized, turn, visits);
      const mergedFrames = mergeAnalysisFrame(frames, frame);
      setGame(artifacts.projection);
      setPositions(artifacts.replayed);
      setFrames(mergedFrames);
      setProblems(await classifyProblems(mergedFrames));
      setCurrentMove(frame.turn);
      setSelectedCandidateIndex(null);
      setMessage(`KataGo analysis completed for move ${frame.turn} with ${frame.visits} visits.`);
    } catch (error) {
      setMessage(`KataGo analysis failed: ${errorMessage(error)}`);
    } finally {
      setIsKataGoRunning(false);
    }
  }

  async function handleAnalyzeKataGoGame(profile: EngineProfileDto, maxVisits: number) {
    if (activeJobIdRef.current || startingAnalysisRef.current) return;
    const visits = resolveAnalysisMaxVisits(maxVisits, preferences);
    startingAnalysisRef.current = true;
    pendingAnalysisProgressRef.current.clear();
    pendingAnalysisTerminalEventsRef.current.clear();
    setIsKataGoRunning(true);
    setAnalysisProgress(null);
    setMessage("Starting full-game KataGo analysis...");
    let cleanup: (() => void) | null = null;
    try {
      const artifacts = nativeRuntime
        ? await artifactsFromCurrentGame()
        : { serialized: sgfText, projection: await parseSgfSummary(sgfText), replayed: await replaySgfPositions(sgfText) };
      if (!nativeRuntime) setMessage(nativeCurrentGameUnavailable);
      const parsed = artifacts.projection;
      const replayed = artifacts.replayed;
      cleanup = await listenToKataGoAnalysisEvents({
        onProgress: (payload) => {
          if (startingAnalysisRef.current && activeJobIdRef.current === null) {
            pendingAnalysisProgressRef.current.set(payload.job_id, {
              jobId: payload.job_id,
              completed: payload.completed,
              expected: payload.expected,
              turn: payload.turn,
              responseJsonl: payload.response_jsonl
            });
            return;
          }
          if (!isCurrentAnalysisJob(payload.job_id)) return;
          setAnalysisProgress({
            jobId: payload.job_id,
            completed: payload.completed,
            expected: payload.expected,
            turn: payload.turn,
            responseJsonl: payload.response_jsonl
          });
          setMessage(`Analyzing move ${payload.turn}: ${payload.completed}/${payload.expected} positions complete.`);
        },
        onComplete: (payload) => {
          if (startingAnalysisRef.current && activeJobIdRef.current === null) {
            pendingAnalysisTerminalEventsRef.current.set(payload.job_id, { kind: "complete", frames: payload.frames });
            return;
          }
          if (!isCurrentAnalysisJob(payload.job_id)) return;
          void finishCompletedAnalysis(payload.job_id, payload.frames, parsed, replayed);
        },
        onError: (payload) => {
          if (startingAnalysisRef.current && activeJobIdRef.current === null) {
            pendingAnalysisTerminalEventsRef.current.set(payload.job_id, { kind: "error", message: payload.message });
            return;
          }
          if (!isCurrentAnalysisJob(payload.job_id)) return;
          finishStoppedAnalysis(payload.job_id);
          setMessage(`Full-game KataGo analysis failed: ${payload.message}`);
        },
        onCancelled: (payload) => {
          if (startingAnalysisRef.current && activeJobIdRef.current === null) {
            pendingAnalysisTerminalEventsRef.current.set(payload.job_id, { kind: "cancelled", message: payload.message });
            return;
          }
          if (!isCurrentAnalysisJob(payload.job_id)) return;
          finishStoppedAnalysis(payload.job_id);
          setAnalysisProgress(null);
          setMessage(payload.message || "Full-game KataGo analysis cancelled.");
        }
      });
      cleanupAnalysisListeners();
      analysisCleanupRef.current = cleanup;
      const jobId = await startKataGoGameAnalysis(profile, artifacts.serialized, visits);
      const pendingTerminalEvent = pendingAnalysisTerminalEventsRef.current.get(jobId);
      const pendingProgress = pendingAnalysisProgressRef.current.get(jobId);
      startingAnalysisRef.current = false;
      pendingAnalysisProgressRef.current.clear();
      pendingAnalysisTerminalEventsRef.current.clear();
      if (pendingTerminalEvent) {
        await finishPendingAnalysisTerminalEvent(jobId, pendingTerminalEvent, parsed, replayed);
        return;
      }
      activeJobIdRef.current = jobId;
      setActiveJobId(jobId);
      if (pendingProgress) setAnalysisProgress(pendingProgress);
      setMessage(`Full-game KataGo analysis started (${jobId}).`);
    } catch (error) {
      cleanup?.();
      if (analysisCleanupRef.current === cleanup) analysisCleanupRef.current = null;
      startingAnalysisRef.current = false;
      pendingAnalysisProgressRef.current.clear();
      pendingAnalysisTerminalEventsRef.current.clear();
      activeJobIdRef.current = null;
      setActiveJobId(null);
      setAnalysisProgress(null);
      setIsKataGoRunning(false);
      setMessage(`Full-game KataGo analysis failed: ${errorMessage(error)}`);
    }
  }

  async function handleCancelKataGoAnalysis() {
    const jobId = activeJobIdRef.current;
    if (!jobId) return;
    try {
      setMessage("Cancelling full-game KataGo analysis...");
      await cancelKataGoAnalysis(jobId);
    } catch (error) {
      setMessage(`Cancel failed: ${errorMessage(error)}`);
    }
  }

  async function handleImportFile(file: File | null) {
    if (!file) return;
    const text = await file.text();
    await applyReplacement(text, null, {
      confirmMessage: "放弃未保存的棋谱并导入这个文件？",
      fallbackName: file.name,
      successMessage: (projection, fileName) => `Imported ${fileName}: ${projection.summary.move_count} moves.`,
      failurePrefix: "Import failed"
    });
  }

  async function handleProviderImport(result: ProviderImportResult) {
    const source = providerSourceLabel(result);
    const warningText = result.warnings.length > 0 ? ` ${result.warnings.length} provider warning(s).` : "";
    const applied = await applyReplacement(result.sgf_text, null, {
      confirmMessage: "放弃未保存的棋谱并载入 Provider 棋谱？",
      fallbackName: providerDocumentName(result),
      successMessage: (projection) =>
        `Imported ${providerLabel(result.provider)} provider payload from ${source}: ${projection.summary.move_count} moves.${warningText}`,
      failurePrefix: "Provider import failed"
    });
    if (!applied) throw new Error("已取消载入，当前棋谱未改动。");
  }

  async function loadSample() {
    await applyReplacement(demoSgf, null, {
      confirmMessage: "放弃未保存的棋谱并载入示例？",
      fallbackName: "sample.sgf",
      successMessage: (projection) => `Sample SGF restored: ${projection.summary.move_count} moves.`,
      failurePrefix: "Sample load failed"
    });
  }

  async function handleNewGame() {
    await applyReplacement(emptySgf, null, {
      confirmMessage: "放弃未保存的棋谱并新建对局？",
      successMessage: () => "已新建空谱。",
      failurePrefix: "New game failed",
      checkCache: false
    });
  }

  async function handleCopySgf() {
    try {
      const text = nativeRuntime ? await serializeCurrentGame() : sgfText;
      await navigator.clipboard.writeText(text);
      setMessage("棋谱已复制到剪贴板。");
    } catch (error) {
      setMessage(`复制失败: ${errorMessage(error)}`);
    }
  }

  async function handlePasteSgf() {
    try {
      const text = (await navigator.clipboard.readText()).trim();
      if (!text) {
        setMessage("剪贴板没有棋谱。");
        return;
      }
      await applyReplacement(text, null, {
        confirmMessage: "放弃未保存的棋谱并粘贴剪贴板棋谱？",
        fallbackName: "clipboard.sgf",
        successMessage: (projection) => `已粘贴棋谱: ${projection.summary.move_count} 手。`,
        failurePrefix: "粘贴失败"
      });
    } catch (error) {
      setMessage(`粘贴失败: ${errorMessage(error)}`);
    }
  }

  async function selectNode(path: NodePath) {
    if (!currentGame || navigatingRef.current) return;
    if (samePath(path, currentGame.selected_path)) return;
    navigatingRef.current = true;
    try {
      const result = await selectCurrentGameNode(path);
      setCurrentGame(result);
      setChosenChildren((prev) => rememberChosenChildren(prev, result.selected_path));
      setCurrentMove(result.snapshot.position.move_number);
      setSelectedCandidateIndex(null);
    } catch (error) {
      setMessage(`导航失败: ${errorMessage(error)}`);
    } finally {
      navigatingRef.current = false;
    }
  }

  function handleMoveSelect(moveNumber: number) {
    if (currentGame) {
      const leaf = chosenLeafPath(currentGame.tree, { indices: [] }, chosenChildren);
      const depth = Math.max(0, Math.min(moveNumber, leaf.indices.length));
      void selectNode({ indices: leaf.indices.slice(0, depth) });
      return;
    }
    setCurrentMove(clampMoveNumberToPositions(positions, moveNumber));
    setSelectedCandidateIndex(null);
  }

  function handleParent() {
    if (parentOfSelected) void selectNode(parentOfSelected);
  }

  function handleNextChild() {
    if (!selectedNode || selectedNode.children.length === 0) return;
    void selectNode(childPath(selectedPath, chosenChildIndex(chosenChildren, selectedPath, selectedNode.children.length)));
  }

  function handlePrevSibling() {
    if (siblingIndex === undefined || siblingIndex <= 0) return;
    void selectNode({ indices: [...selectedPath.indices.slice(0, -1), siblingIndex - 1] });
  }

  function handleNextSibling() {
    if (!parentNode || siblingIndex === undefined || siblingIndex + 1 >= parentNode.children.length) return;
    void selectNode({ indices: [...selectedPath.indices.slice(0, -1), siblingIndex + 1] });
  }

  function cleanupAnalysisListeners() {
    analysisCleanupRef.current?.();
    analysisCleanupRef.current = null;
  }

  function isCurrentAnalysisJob(jobId: string): boolean {
    return activeJobIdRef.current === jobId;
  }

  async function finishPendingAnalysisTerminalEvent(jobId: string, event: PendingAnalysisTerminalEvent, parsed: GameDto, replayed: PositionDto[]) {
    if (event.kind === "complete") {
      await finishCompletedAnalysis(jobId, event.frames, parsed, replayed);
      return;
    }
    finishStoppedAnalysis(jobId);
    setAnalysisProgress(null);
    setMessage(event.kind === "error"
      ? `Full-game KataGo analysis failed: ${event.message}`
      : event.message || "Full-game KataGo analysis cancelled.");
  }

  async function finishCompletedAnalysis(jobId: string, result: AnalysisFrameDto[], parsed: GameDto, replayed: PositionDto[]) {
    const lastAnalyzedMove = result.at(-1)?.turn ?? replayed.at(-1)?.move_number ?? parsed.moves.length;
    const shownMove = clampMoveNumberToPositions(replayed, lastAnalyzedMove);
    const classified = await classifyProblems(result);
    setGame(parsed);
    setPositions(replayed);
    setFrames(result);
    setProblems(classified);
    setCurrentMove(shownMove);
    setSelectedCandidateIndex(null);
    setAnalysisProgress((progress) => progress ? { ...progress, completed: progress.expected || result.length, expected: progress.expected || result.length } : progress);
    finishStoppedAnalysis(jobId);
    const serialized = nativeRuntime ? await serializeCurrentGame() : sgfText;
    const cacheMessage = await saveAnalysisCacheForGame(serialized, currentFilePath, parsed, result, classified, "katago");
    setMessage(`Full-game KataGo analysis completed with ${result.length} frames. Showing move ${shownMove}.${cacheMessage}`);
  }

  function finishStoppedAnalysis(jobId: string) {
    if (activeJobIdRef.current !== null && activeJobIdRef.current !== jobId) return;
    activeJobIdRef.current = null;
    setActiveJobId(null);
    setIsKataGoRunning(false);
    cleanupAnalysisListeners();
  }

  async function checkAnalysisCacheForGame(text: string, filePath: string | null, parsed: GameDto, replayed: PositionDto[], baseMessage: string) {
    if (!preferences.autoLoadCache) {
      resetAnalysisCacheState();
      setMessage(`${baseMessage} Cache auto-load is off.`);
      return;
    }
    setCacheStatus("checking");
    setCacheRecord(null);
    setCacheError(null);
    try {
      const key = await computeGameCacheKey(text, filePath);
      setCurrentCacheKey(key);
      const lookup = await loadPreferredAnalysisCache(key.gameKey);
      if (lookup.status === "hit") {
        const payload = cachedAnalysisPayload(lookup.record.payload);
        if (!payload) {
          setCacheStatus("error");
          setCacheRecord(lookup.record);
          setCacheError("Cached payload is not compatible with this app version.");
          setMessage(`${baseMessage} ${cacheEngineLabel(lookup.engineKind)} cache hit, but the payload could not be restored.`);
          return;
        }
        setFrames(payload.frames);
        setProblems(payload.problems);
        setCurrentMove(clampMoveNumberToPositions(replayed, payload.frames.at(-1)?.turn ?? parsed.moves.length));
        setSelectedCandidateIndex(null);
        setCacheStatus("hit");
        setCacheRecord(lookup.record);
        setMessage(`${baseMessage} Restored ${payload.frames.length} cached ${cacheEngineLabel(lookup.engineKind)} review frames.`);
        return;
      }
      if (lookup.status === "error") {
        setCacheStatus("error");
        setCacheRecord(null);
        setCacheError(lookup.message);
        setMessage(`${baseMessage} Cache unavailable: ${lookup.message}`);
        return;
      }
      setCacheStatus("miss");
      setCacheRecord(null);
      setMessage(`${baseMessage} No cached review yet.`);
    } catch (error) {
      const message = errorMessage(error);
      setCacheStatus("error");
      setCacheRecord(null);
      setCacheError(message);
      setCurrentCacheKey(null);
      setMessage(`${baseMessage} Cache unavailable: ${message}`);
    }
  }

  async function loadPreferredAnalysisCache(gameKey: string): Promise<AnalysisCacheLoadResult> {
    const katagoLookup = await loadAnalysisCache(gameKey, null, "katago");
    if (katagoLookup.status === "hit" && katagoLookup.record) return { status: "hit", record: katagoLookup.record, engineKind: "katago" };
    if (katagoLookup.status === "error") return { status: "error", message: katagoLookup.error ?? "KataGo cache lookup failed." };

    const fakeLookup = await loadAnalysisCache(gameKey, null, "fake");
    if (fakeLookup.status === "hit" && fakeLookup.record) return { status: "hit", record: fakeLookup.record, engineKind: "fake" };
    if (fakeLookup.status === "error") return { status: "error", message: fakeLookup.error ?? "Fake review cache lookup failed." };

    return { status: "miss" };
  }

  async function saveAnalysisCacheForGame(
    text: string,
    filePath: string | null,
    parsed: GameDto,
    analysisFrames: AnalysisFrameDto[],
    analysisProblems: ProblemMarkerDto[],
    engineKind: CacheEngineKind
  ): Promise<string> {
    if (!preferences.autoSaveAnalysis) {
      setCacheStatus("idle");
      return " Cache auto-save is off.";
    }
    setCacheStatus("saving");
    setCacheError(null);
    try {
      const key = currentCacheKey ?? await computeGameCacheKey(text, filePath);
      setCurrentCacheKey(key);
      const payload = { frames: analysisFrames, problems: analysisProblems } as unknown as JsonValue;
      const saved = await saveAnalysisCache({
        gameKey: key.gameKey,
        sgfHash: key.sgfHash,
        profileId: null,
        engineKind,
        source: engineKind === "katago" ? "katago" : "browser",
        moveCount: parsed.summary.move_count,
        analyzedMoveCount: countAnalyzedMoves(analysisFrames, parsed.summary.move_count),
        payload
      });
      setCacheRecord({
        id: saved.id,
        gameKey: saved.gameKey,
        sgfHash: key.sgfHash,
        profileId: null,
        engineKind,
        source: engineKind === "katago" ? "katago" : "browser",
        moveCount: parsed.summary.move_count,
        analyzedMoveCount: countAnalyzedMoves(analysisFrames, parsed.summary.move_count),
        payload,
        updatedAt: saved.updatedAt
      });
      setCacheStatus("saved");
      return ` Cached ${analysisFrames.length} ${cacheEngineLabel(engineKind)} frames.`;
    } catch (error) {
      const failed = errorMessage(error);
      setCacheStatus("error");
      setCacheError(failed);
      return ` Cache save failed: ${failed}`;
    }
  }

  function clearReviewData() {
    setFrames([]);
    setProblems([]);
    setSelectedCandidateIndex(null);
    setAnalysisProgress(null);
    setCacheRecord(null);
  }

  function resetAnalysisCacheState() {
    setCacheStatus("idle");
    setCacheRecord(null);
    setCacheError(null);
    setCurrentCacheKey(null);
  }

  return <main className={`app-shell${preferences.boardTheme === "high-contrast" ? " theme-high-contrast" : ""}${nativeRuntime ? "" : " has-native-runtime-note"}`}>
    {!nativeRuntime ? <p className="native-runtime-note" role="status">{nativeCurrentGameUnavailable}</p> : null}
    <AppChrome
      sheet={sheet}
      onToggleSheet={toggleSheet}
      busy={isKataGoRunning}
      dirty={dirty}
      documentName={documentName}
      engineLabel={engineLabel}
      engineReady={engineReady}
      onEngineCommand={handleEngineCommand}
      preferences={preferences}
      onPreferencesChange={(next) => void handlePreferencesChange(next)}
      showCoordinates={showCoordinates}
      showMoveNumbers={showMoveNumbers}
      onShowCoordinates={setShowCoordinates}
      onShowMoveNumbers={setShowMoveNumbers}
      showBlackCandidates={showBlackCandidates}
      showWhiteCandidates={showWhiteCandidates}
      onShowBlackCandidates={setShowBlackCandidates}
      onShowWhiteCandidates={setShowWhiteCandidates}
      isKataGoRunning={isKataGoRunning}
      autoPlaying={autoPlaying}
      komi={game.summary.komi}
      onNew={() => void handleNewGame()}
      onOpen={() => void handleOpenSgfDocument()}
      nativeRuntime={nativeRuntime}
      nativeUnavailable={nativeCurrentGameUnavailable}
      onSave={() => void handleSaveSgfDocument(false)}
      onSaveAs={() => void handleSaveSgfDocument(true)}
      onLoadSample={() => void loadSample()}
      onParse={() => void handleParseSgf()}
      onFakeAnalyze={() => void handleFakeAnalyze()}
      onCancel={() => void handleCancelKataGoAnalysis()}
      onAbout={() => setMessage("LizzieYzy Next 0.1.0 · 桌面复盘工作区")}
      onCopySgf={() => void handleCopySgf()}
      onPasteSgf={() => void handlePasteSgf()}
      onClearBoard={() => void handleNewGame()}
      onAutoPlay={() => setAutoPlaying((value) => !value)}
      onOverlayMode={setOverlayMode}
      cacheBadge={<CacheStatusBadge status={cacheStatus} record={cacheRecord} error={cacheError} />}
      message={message}
      toPlay={currentPosition.to_play}
    />
    <section className="spread">
      <aside className="rail">
        <div className="rail-block">
          <h2>
            <span>胜率走势 (黑)</span>
            <span style={{ color: "#60a5fa", fontFamily: "var(--mono)" }}>
              {visibleCurrentFrame ? `${(visibleCurrentFrame.winrate_black * 100).toFixed(1)}%` : "50.0%"}
            </span>
          </h2>
          <WinrateChart frames={frames} currentMove={currentMove} />
          <div id="board-layers" />
        </div>
        <AnalysisPanel
          pane="commentary"
          frame={visibleCurrentFrame}
          problems={problems}
          moves={game.moves}
          boardSize={game.summary.board_size}
          currentMove={currentMove}
          currentPosition={currentPosition}
          personalComment={selectedPersonalComment}
          selectedCandidateIndex={selectedCandidateIndex}
          onSelectCandidate={setSelectedCandidateIndex}
          onSelectProblem={handleMoveSelect}
        />
      </aside>
      <div className="diagram">
        <BoardCanvas
          position={currentPosition}
          analysis={visibleCurrentFrame}
          selectedCandidateIndex={selectedCandidateIndex}
          moves={game.moves}
          showCoordinates={showCoordinates}
          showMoveNumbers={showMoveNumbers}
          overlayMode={overlayMode}
          onOverlayModeChange={setOverlayMode}
          hideCandidates={(currentPosition.to_play === "black" && !showBlackCandidates) || (currentPosition.to_play === "white" && !showWhiteCandidates)}
        />
      </div>
      <aside className="sheet-col">
        <AnalysisPanel
          pane="reference"
          frame={visibleCurrentFrame}
          problems={problems}
          moves={game.moves}
          boardSize={game.summary.board_size}
          currentMove={currentMove}
          currentPosition={currentPosition}
          selectedCandidateIndex={selectedCandidateIndex}
          onSelectCandidate={setSelectedCandidateIndex}
          onSelectProblem={handleMoveSelect}
        />
      </aside>
    </section>
    <BottomBar
      currentMove={reviewIndex}
      maxMove={reviewMax}
      onMove={handleMoveSelect}
      canParent={canParent}
      canNext={canNext}
      canPrevSibling={canPrevSibling}
      canNextSibling={canNextSibling}
      siblingLabel={siblingLabel}
      onParent={handleParent}
      onNext={handleNextChild}
      onPrevSibling={handlePrevSibling}
      onNextSibling={handleNextSibling}
      engineReady={engineReady}
      isKataGoRunning={isKataGoRunning}
      analysisProgress={analysisProgress}
      onAnalyzeOnce={() => handleEngineCommand("once")}
      onAnalyzeGame={() => handleEngineCommand("game")}
      onCancel={() => void handleCancelKataGoAnalysis()}
      onSync={() => toggleSheet("sync")}
      onFlashAnalyze={() => void handleFakeAnalyze()}
      onHeatmap={() => setOverlayMode("policy")}
      onRefresh={() => void handleParseSgf()}
      onClearBoard={() => void handleNewGame()}
      onEstimate={() => setOverlayMode("ownership")}
      onAutoPlay={() => setAutoPlaying((value) => !value)}
      autoPlaying={autoPlaying}
      showCoordinates={showCoordinates}
      showMoveNumbers={showMoveNumbers}
      onShowCoordinates={setShowCoordinates}
      onShowMoveNumbers={setShowMoveNumbers}
      jumpRef={jumpRef}
      message={message}
      toPlay={currentPosition.to_play}
    />
    <section className="sheet-row" hidden={sheet === "none"}>
      {sheet === "sgf" ? <div className="sgf-tools">
        <div className="document-row">
          <strong title={currentFilePath ?? documentName}>{documentName}{dirty ? " *" : ""}</strong>
          <span>{dirty ? "未保存" : "已保存"}</span>
        </div>
        <textarea value={sgfText} onChange={(event) => {
          setSgfText(event.target.value);
          setMessage("这段文本只是载入输入。解析棋谱才会替换当前对局。");
        }} spellCheck={false} aria-label="棋谱载入文本" />
        <div className="button-row">
          <label className={`file-button${isKataGoRunning ? " file-button-disabled" : ""}`}>
            导入棋谱
            <input type="file" accept=".sgf,.txt,application/x-go-sgf,text/plain" disabled={isKataGoRunning} onChange={(event) => void handleImportFile(event.target.files?.[0] ?? null)} />
          </label>
          <button type="button" onClick={() => void loadSample()} disabled={isKataGoRunning}>载入示例</button>
        </div>
      </div> : null}
      {sheet === "sync" ? <ProviderPanel disabled={isKataGoRunning} onImport={handleProviderImport} /> : null}
      <div hidden={sheet !== "engine"}>
        <EngineSetupPanel
          disabled={isKataGoRunning}
          onRun={handleRunKataGo}
          onAnalyzeGame={handleAnalyzeKataGoGame}
          onCancelAnalysis={handleCancelKataGoAnalysis}
          analysisProgress={analysisProgress}
          activeJobId={activeJobId}
          onBindCommands={bindEngineCommands}
        />
      </div>
      {sheet === "prefs" ? <PreferencesPanel
        preferences={preferences}
        status={preferencesStatus}
        disabled={isKataGoRunning}
        onChange={(nextPreferences) => void handlePreferencesChange(nextPreferences)}
      /> : null}
    </section>
  </main>;
}

function applyPreferencesToFrame(frame: AnalysisFrameDto | undefined, preferences: AppPreferences): AnalysisFrameDto | undefined {
  if (!frame) return undefined;
  return {
    ...frame,
    candidates: preferences.showCandidates ? frame.candidates.slice(0, preferences.candidateLimit) : [],
    ownership: preferences.showOwnership ? frame.ownership : null,
    policy: preferences.showPolicy ? frame.policy : null
  };
}

function resolveAnalysisMaxVisits(requestedMaxVisits: number | null | undefined, preferences: AppPreferences): number {
  if (typeof requestedMaxVisits === "number" && Number.isFinite(requestedMaxVisits) && requestedMaxVisits > 0) {
    return Math.floor(requestedMaxVisits);
  }
  return preferences.reviewMode === "deep" ? preferences.defaultMaxVisits * 2 : preferences.defaultMaxVisits;
}

function cachedAnalysisPayload(payload: JsonValue): CachedAnalysisPayload | null {
  if (!isJsonObject(payload)) return null;
  if (!Array.isArray(payload.frames) || !Array.isArray(payload.problems)) return null;
  return {
    frames: payload.frames as unknown as AnalysisFrameDto[],
    problems: payload.problems as unknown as ProblemMarkerDto[]
  };
}

function isJsonObject(value: JsonValue): value is { [key: string]: JsonValue } {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function mergeAnalysisFrame(frames: AnalysisFrameDto[], frame: AnalysisFrameDto): AnalysisFrameDto[] {
  return [...frames.filter((item) => item.turn !== frame.turn), frame].sort((a, b) => a.turn - b.turn);
}

function countAnalyzedMoves(frames: AnalysisFrameDto[], moveCount: number): number {
  const turns = new Set(frames.map((frame) => frame.turn).filter((turn) => turn > 0 && turn <= moveCount));
  return turns.size;
}

function errorMessage(error: unknown): string {
  if (error instanceof Error) return error.message;
  if (error && typeof error === "object" && "message" in error) {
    const message = Reflect.get(error, "message");
    if (typeof message === "string" && message.trim()) return message;
  }
  return String(error);
}

function cacheEngineLabel(engineKind: CacheEngineKind): string {
  return engineKind === "katago" ? "KataGo" : "fake";
}

function fileNameFromPath(path: string): string {
  return path.split(/[\\/]/).filter(Boolean).at(-1) ?? path;
}

function pathKey(path: NodePath): string {
  return path.indices.join(",");
}

function samePath(left: NodePath, right: NodePath): boolean {
  return left.indices.length === right.indices.length && left.indices.every((index, offset) => index === right.indices[offset]);
}

function parentPath(path: NodePath): NodePath | null {
  if (path.indices.length === 0) return null;
  return { indices: path.indices.slice(0, -1) };
}

function childPath(path: NodePath, index: number): NodePath {
  return { indices: [...path.indices, index] };
}

function nodeAt(root: SgfTreeNodeDto, path: NodePath): SgfTreeNodeDto | null {
  let node: SgfTreeNodeDto | undefined = root;
  for (const index of path.indices) {
    node = node.children[index];
    if (!node) return null;
  }
  return node;
}

function chosenFromPath(path: NodePath): Map<string, number> {
  const chosen = new Map<string, number>();
  for (let depth = 0; depth < path.indices.length; depth += 1) {
    chosen.set(pathKey({ indices: path.indices.slice(0, depth) }), path.indices[depth]);
  }
  return chosen;
}

function rememberChosenChildren(previous: Map<string, number>, path: NodePath): Map<string, number> {
  const next = new Map(previous);
  for (const [key, index] of chosenFromPath(path)) {
    next.set(key, index);
  }
  return next;
}

function chosenChildIndex(chosen: Map<string, number>, path: NodePath, childCount: number): number {
  const remembered = chosen.get(pathKey(path));
  if (remembered !== undefined && remembered >= 0 && remembered < childCount) return remembered;
  return 0;
}

function chosenLeafPath(root: SgfTreeNodeDto, start: NodePath, chosen: Map<string, number>): NodePath {
  const indices = [...start.indices];
  let node = nodeAt(root, start);
  while (node && node.children.length > 0) {
    const index = chosenChildIndex(chosen, { indices }, node.children.length);
    indices.push(index);
    node = node.children[index];
  }
  return { indices };
}
