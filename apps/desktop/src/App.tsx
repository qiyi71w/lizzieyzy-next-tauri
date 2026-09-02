import { useEffect, useMemo, useRef, useState } from "react";
import { BoardCanvas } from "./components/BoardCanvas";
import { WinrateChart } from "./components/WinrateChart";
import { AnalysisPanel } from "./components/AnalysisPanel";
import { EngineSetupPanel } from "./components/EngineSetupPanel";
import { CacheStatusBadge } from "./components/CacheStatusBadge";
import { AppChrome, BottomBar, type OverlayMode, type SheetId } from "./components/AppChrome";
import { PreferencesPanel } from "./components/PreferencesPanel";
import { ShortcutReference } from "./components/ShortcutReference";
import { ProviderPanel } from "./components/ProviderPanel";
import {
  cancelKataGoAnalysis,
  cancelSelectedNodeAnalysis,
  classifyProblems,
  fakeAnalyze,
  getHealth,
  isTauriRuntime,
  nativeCurrentGameUnavailable,
  openSgfDocument,
  parseSgfSummary,
  playCurrentGame,
  projectCurrentGameMainline,
  replaySgfPositions,
  replaceCurrentGame,
  saveCurrentGame,
  serializeCurrentGame,
  selectCurrentGameNode,
  setCurrentGamePersonalComment,
  removeCurrentGameVariation,
  startKataGoGameAnalysis,
  startSelectedNodeAnalysis,
  subscribeForegroundEngine,
  startForegroundEngine,
  stopForegroundEngine,
  restartForegroundEngine,
  switchForegroundEngine,
  loadEngineProfilesSettings
} from "./api/backend";
import {
  admitsForegroundEngineJobs,
  canRestartForegroundEngine,
  canStopForegroundEngine,
  displayedEngineFailure,
  emptyForegroundEngineSnapshot,
  engineStatusLabel,
  runFromSnapshot,
  shouldAcceptFailureEvent
} from "./domain/foregroundEngine";
import { computeGameCacheKey, loadAnalysisCache, saveAnalysisCache } from "./api/analysisCache";
import { loadAppPreferences, saveAppPreferences } from "./api/preferences";
import { clampMoveNumberToPositions, createDemoGame, replayGamePositions, selectExactPosition } from "./domain/board";
import type { AnalysisCacheRecord, CacheStatus, GameCacheKey, JsonValue } from "./domain/cache";
import { defaultAppPreferences, normalizeAppPreferences, type AppPreferences } from "./domain/preferences";
import { providerDocumentName, providerLabel, providerSourceLabel, type ProviderImportResult } from "./domain/providers";
import { admitsAnalysisPublication } from "./domain/analysisJob";
import { createShortcutRegistry } from "./domain/shortcuts";
import {
  createLocalRequestToken,
  shouldPublishReviewPresentation,
  type ReviewPresentationScope
} from "./domain/reviewPresentation";
import type { AnalysisFrameDto, AnalysisJobEventDto, AnalysisJobStartedDto, AppHealthDto, CurrentGameResultDto, EngineProfileDto, EngineProfileRecordDto, EngineFailureDto, ForegroundEngineSnapshotDto, GameDto, MoveVertex, NodePath, PositionDto, ProblemMarkerDto, SgfTreeNodeDto } from "./domain/types";

const demoSgf = "(;GM[1]FF[4]SZ[19]KM[7.5]PB[李昌镐]PW[芮乃伟]RE[B+R];B[pd];W[dd];B[pp];W[dp];B[jq];W[qj];B[nc];W[fc];B[qf];W[cn];B[cp];W[do];B[co];W[dn];B[fq];W[eq];B[fp];W[gp];B[gq];W[hp])";
const emptySgf = "(;GM[1]FF[4]SZ[19]KM[7.5]PB[黑]PW[白])";
const demoGame = createDemoGame();
type WholeGameProgress = { completed: number; expected: number };
type CacheEngineKind = "fake" | "katago";
type CachedAnalysisPayload = { frames: AnalysisFrameDto[]; problems: ProblemMarkerDto[] };
type PendingPreferencesSave = { version: number; preferences: AppPreferences };
type CandidatePreview = { index: number; scope: ReviewPresentationScope };
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
  const [boardIntentFeedback, setBoardIntentFeedback] = useState<string | null>(null);
  const nativeRuntime = isTauriRuntime();
  const [selectedNodeRunning, setSelectedNodeRunning] = useState(false);
  const [wholeGameRunning, setWholeGameRunning] = useState(false);
  const [wholeGameProgress, setWholeGameProgress] = useState<WholeGameProgress | null>(null);
  const [selectedCandidateIndex, setSelectedCandidateIndex] = useState<number | null>(null);
  const [candidatePreview, setCandidatePreview] = useState<CandidatePreview | null>(null);
  const [currentFilePath, setCurrentFilePath] = useState<string | null>(null);
  const [fallbackFileName, setFallbackFileName] = useState<string | null>(null);
  const [dirty, setDirty] = useState(false);
  const [currentGame, setCurrentGame] = useState<CurrentGameResultDto | null>(null);
  const [chosenChildren, setChosenChildren] = useState<Map<string, number>>(() => new Map());
  const navigatingRef = useRef(false);
  const documentGenerationRef = useRef(0);
  const pendingSelectedPathRef = useRef<NodePath | null>(null);
  const pendingBoardIntentRef = useRef<string | null>(null);
  const [cacheStatus, setCacheStatus] = useState<CacheStatus>("idle");
  const [cacheRecord, setCacheRecord] = useState<AnalysisCacheRecord | null>(null);
  const [cacheError, setCacheError] = useState<string | null>(null);
  const [currentCacheKey, setCurrentCacheKey] = useState<GameCacheKey | null>(null);
  const [preferences, setPreferences] = useState<AppPreferences>(() => defaultAppPreferences);
  const [preferencesStatus, setPreferencesStatus] = useState("正在载入设置…");
  const [sheet, setSheet] = useState<"none" | SheetId>("none");
  const [engineSnapshot, setEngineSnapshot] = useState<ForegroundEngineSnapshotDto>(() => emptyForegroundEngineSnapshot());
  const [engineProfiles, setEngineProfiles] = useState<EngineProfileRecordDto[]>([]);
  const [engineFailure, setEngineFailure] = useState<EngineFailureDto | null>(null);
  const engineSnapshotRef = useRef(engineSnapshot);
  engineSnapshotRef.current = engineSnapshot;
  const engineFailureRef = useRef(engineFailure);
  engineFailureRef.current = engineFailure;
  const lastSwitchIdRef = useRef<string | null>(null);
  const visibleEngineFailure = displayedEngineFailure(
    engineSnapshot,
    engineFailure,
    lastSwitchIdRef.current
  );
  const engineLabel = engineStatusLabel(engineSnapshot);
  const engineReady = admitsForegroundEngineJobs(engineSnapshot);
  const [showCoordinates, setShowCoordinates] = useState(true);
  const [showMoveNumbers, setShowMoveNumbers] = useState(false);
  const [showBlackCandidates, setShowBlackCandidates] = useState(true);
  const [showWhiteCandidates, setShowWhiteCandidates] = useState(true);
  const [overlayMode, setOverlayMode] = useState<OverlayMode>("candidates");
  const [autoPlaying, setAutoPlaying] = useState(false);
  const [shortcutReferenceOpen, setShortcutReferenceOpen] = useState(false);
  const shortcutRegistry = useMemo(() => createShortcutRegistry(), []);
  const jumpRef = useRef<HTMLInputElement | null>(null);
  const requestSerialRef = useRef(0);
  const [activeRequestToken, setActiveRequestToken] = useState("idle:0");
  const activeRequestTokenRef = useRef("idle:0");
  const [publishedScope, setPublishedScope] = useState<ReviewPresentationScope | null>(null);
  const startingAnalysisRef = useRef(false);
  const preferencesLoadSettledRef = useRef(false);
  const committedPreferencesRef = useRef<AppPreferences>(defaultAppPreferences);
  const preferencesSaveInFlightRef = useRef(false);
  const preferencesSaveVersionRef = useRef(0);
  const pendingPreferencesSaveRef = useRef<PendingPreferencesSave | null>(null);
  const currentGameRef = useRef<CurrentGameResultDto | null>(null);
  const selectedNodeJobRef = useRef<AnalysisJobStartedDto | null>(null);
  const wholeGameJobRef = useRef<AnalysisJobStartedDto | null>(null);
  const handleSelectedNodeJobRef = useRef<(job: AnalysisJobEventDto) => void>(() => undefined);
  const handleWholeGameJobRef = useRef<(job: AnalysisJobEventDto) => void>(() => undefined);

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
        if (!isMounted) return;
        settleLoadedPreferences(loaded.preferences, loaded.recovery?.message ?? "Preferences loaded.");
      })
      .catch((error: unknown) => {
        if (!isMounted) return;
        settleLoadedPreferences(defaultAppPreferences, `Load failed: ${errorMessage(error)}`);
      });
    return () => {
      isMounted = false;
    };
  }, []);

  const currentPosition = useMemo(() => {
    if (currentGame) return currentGame.snapshot.position;
    return selectExactPosition(positions, currentMove, game.summary.board_size);
  }, [currentGame, currentMove, positions, game.summary.board_size]);
  currentGameRef.current = currentGame;
  const selectedPersonalComment = currentGame ? currentGame.snapshot.personal_comment : "";
  const selectedGeneratedInformation = currentGame?.snapshot.generated_information ?? null;
  const selectedPath = currentGame?.selected_path ?? { indices: [] };
  const selectedNode = currentGame ? nodeAt(currentGame.tree, selectedPath) : null;
  const selectedPathKey = selectedPath.indices.join(".");
  const activeScope = useMemo<ReviewPresentationScope>(() => ({
    generation: currentGame?.generation ?? 0,
    selectedPath: selectedPath.indices,
    requestToken: activeRequestToken
  }), [currentGame?.generation, selectedPathKey, activeRequestToken]);
  const presentationLive = publishedScope !== null && shouldPublishReviewPresentation(activeScope, publishedScope);
  const visibleFrames = useMemo(() => presentationLive ? frames : [], [presentationLive, frames]);
  const visibleProblems = useMemo(() => presentationLive ? problems : [], [presentationLive, problems]);
  const currentFrame = useMemo(
    () => visibleFrames.find((frame) => frame.turn === currentMove) ?? visibleFrames.at(-1),
    [visibleFrames, currentMove]
  );
  const visibleCurrentFrame = useMemo(() => applyPreferencesToFrame(currentFrame, preferences), [currentFrame, preferences]);
  const previewCandidateIndex = candidatePreview
    && presentationLive
    && shouldPublishReviewPresentation(activeScope, candidatePreview.scope)
    ? candidatePreview.index
    : null;
  const parentOfSelected = parentPath(selectedPath);
  const parentNode = currentGame && parentOfSelected ? nodeAt(currentGame.tree, parentOfSelected) : null;
  const siblingIndex = selectedPath.indices.at(-1);
  const canParent = Boolean(currentGame && parentOfSelected);
  const canRemoveVariation = Boolean(nativeRuntime && currentGame && selectedPath.indices.length > 0);
  const canNext = Boolean(selectedNode && selectedNode.children.length > 0);
  const canPrevSibling = Boolean(parentNode && siblingIndex !== undefined && siblingIndex > 0);
  const canNextSibling = Boolean(parentNode && siblingIndex !== undefined && siblingIndex + 1 < parentNode.children.length);
  const siblingLabel = parentNode && siblingIndex !== undefined ? `${siblingIndex + 1}/${parentNode.children.length}` : "—";
  const maxMove = Math.max(positions.at(-1)?.move_number ?? 0, 1);
  const reviewIndex = currentGame ? selectedPath.indices.length : currentMove;
  const reviewMax = currentGame
    ? chosenLeafPath(currentGame.tree, { indices: [] }, chosenChildren).indices.length
    : maxMove;
  const documentDirty = currentGame?.dirty ?? dirty;
  const documentPath = currentGame?.native_path ?? currentFilePath;
  const documentName = useMemo(() => documentPath ? fileNameFromPath(documentPath) : fallbackFileName ?? "未命名棋谱", [documentPath, fallbackFileName]);
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
    const openEnabled = nativeRuntime;
    const saveEnabled = openEnabled && documentDirty;
    const passEnabled = openEnabled;

    shortcutRegistry.bind("file.new", () => {
      void handleNewGame();
    });
    shortcutRegistry.bind("file.open", () => {
      if (openEnabled) void handleOpenSgfDocument();
    });
    shortcutRegistry.bind("file.save", () => {
      if (saveEnabled) void handleSaveSgfDocument(false);
    });
    shortcutRegistry.bind("file.save-as", () => {
      if (openEnabled) void handleSaveSgfDocument(true);
    });
    shortcutRegistry.bind("file.copy-sgf", () => {
      void handleCopySgf();
    });
    shortcutRegistry.bind("file.paste-sgf", () => {
      void handlePasteSgf();
    });
    shortcutRegistry.bind("review.pass", () => {
      if (passEnabled) void playAt("pass");
    });
    shortcutRegistry.bind("review.remove-variation", () => {
      void handleRemoveVariation();
    });
    shortcutRegistry.bind("review.parent", () => {
      if (currentGame) handleParent();
      else setCurrentMove((move) => clampMoveNumberToPositions(positions, move - 1));
    });
    shortcutRegistry.bind("review.next-child", () => {
      if (currentGame) handleNextChild();
      else setCurrentMove((move) => clampMoveNumberToPositions(positions, move + 1));
    });
    shortcutRegistry.bind("review.prev-sibling", handlePrevSibling);
    shortcutRegistry.bind("review.next-sibling", handleNextSibling);
    shortcutRegistry.bind("review.first", () => {
      handleMoveSelect(0);
    });
    shortcutRegistry.bind("review.last", () => {
      handleMoveSelect(reviewMax);
    });
    shortcutRegistry.bind("review.back-10", () => {
      handleMoveSelect(reviewIndex - 10);
    });
    shortcutRegistry.bind("review.forward-10", () => {
      handleMoveSelect(reviewIndex + 10);
    });
    shortcutRegistry.bind("review.select-candidate", (event) => {
      const index = Number(event.key) - 1;
      if (visibleCurrentFrame?.candidates[index]) setSelectedCandidateIndex(index);
    });
    shortcutRegistry.bind("view.coordinates", () => {
      setShowCoordinates((value) => !value);
    });
    shortcutRegistry.bind("view.move-numbers", () => {
      setShowMoveNumbers((value) => !value);
    });
    shortcutRegistry.bind("view.policy", () => {
      void handlePreferencesChange({ ...preferences, showPolicy: !preferences.showPolicy });
    });
    shortcutRegistry.bind("view.policy-overlay", () => {
      setOverlayMode("policy");
    });
    shortcutRegistry.bind("review.autoplay", () => {
      setAutoPlaying((value) => !value);
    });
    shortcutRegistry.bind("help.shortcut-reference", () => {
      setShortcutReferenceOpen((value) => !value);
    });

    function onKey(event: KeyboardEvent) {
      shortcutRegistry.dispatch(event);
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [
    chosenChildren,
    currentGame,
    documentDirty,
    nativeRuntime,
    positions,
    preferences,
    reviewIndex,
    reviewMax,
    shortcutRegistry,
    visibleCurrentFrame
  ]);

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

  useEffect(() => {
    let cancelled = false;
    void loadEngineProfilesSettings()
      .then((settings) => {
        if (!cancelled) setEngineProfiles(settings.profiles);
      })
      .catch(() => undefined);
    const unlistenPromise = subscribeForegroundEngine(
      (snapshot) => {
        if (cancelled) return;
        engineSnapshotRef.current = snapshot;
        setEngineSnapshot(snapshot);
        if (snapshot.lifecycle.state === "switching") {
          lastSwitchIdRef.current = snapshot.lifecycle.switch_id;
        } else if (snapshot.lifecycle.state === "no_engine") {
          lastSwitchIdRef.current = null;
        }
        if (snapshot.lifecycle.state === "error") {
          engineFailureRef.current = snapshot.lifecycle.failure;
          setEngineFailure(snapshot.lifecycle.failure);
        } else if (!(
          snapshot.lifecycle.state === "ready"
          && engineFailureRef.current?.operation === "switch"
          && engineFailureRef.current.switch_id
          && engineFailureRef.current.switch_id === lastSwitchIdRef.current
          && engineFailureRef.current.run_id !== snapshot.lifecycle.run.run_id
        )) {
          engineFailureRef.current = null;
          setEngineFailure(null);
        }
        if (!selectedNodeJobRef.current && snapshot.selected_node_job) {
          selectedNodeJobRef.current = snapshot.selected_node_job;
          setSelectedNodeRunning(true);
        }
        if (!wholeGameJobRef.current && snapshot.whole_game_job) {
          wholeGameJobRef.current = snapshot.whole_game_job;
          setWholeGameRunning(true);
        }
      },
      (failure) => {
        if (cancelled) return;
        if (!shouldAcceptFailureEvent(
          engineSnapshotRef.current,
          engineFailureRef.current,
          failure,
          lastSwitchIdRef.current
        )) return;
        engineFailureRef.current = failure;
        setEngineFailure(failure);
        setMessage(failure.message);
      },
      (job) => {
        if (cancelled) return;
        if (job.lane === "whole_game") handleWholeGameJobRef.current(job);
        else handleSelectedNodeJobRef.current(job);
      }
    );
    return () => {
      cancelled = true;
      void unlistenPromise.then((unlisten) => unlisten());
    };
  }, []);

  function handleEngineCommand(kind: "once" | "game") {
    const run = runFromSnapshot(engineSnapshot);
    if (!run || !engineReady) return;
    const record = engineProfiles.find((profile) => profile.id === run.profile_id);
    const maxVisits = record?.max_visits ?? 800;
    if (kind === "once") void handleRunKataGo(run.profile_snapshot, maxVisits);
    else void handleAnalyzeKataGoGame(run.run_id, maxVisits);
  }

  async function handleSelectSwitcherProfile(profileId: string) {
    if (!profileId) return;
    const run = runFromSnapshot(engineSnapshot);
    if (run?.profile_id === profileId) return;
    const state = engineSnapshot.lifecycle.state;
    if (state !== "no_engine" && state !== "ready" && state !== "switching") return;
    setEngineFailure(null);
    try {
      if (state === "no_engine") await startForegroundEngine(profileId);
      else await switchForegroundEngine(profileId);
    } catch (error) {
      setMessage(error instanceof Error ? error.message : String(error));
    }
  }


  function handlePreferencesChange(nextPreferences: AppPreferences) {
    if (!preferencesLoadSettledRef.current) return;
    const patch = preferencePatch(preferences, normalizeAppPreferences(nextPreferences));
    if (Object.keys(patch).length === 0) return;
    queuePreferencesSave(pendingPreferencesSaveRef.current?.preferences ?? committedPreferencesRef.current, patch);
  }

  function settleLoadedPreferences(loaded: AppPreferences, status: string) {
    preferencesLoadSettledRef.current = true;
    committedPreferencesRef.current = loaded;
    setPreferences(loaded);
    setPreferencesStatus(status);
  }

  function queuePreferencesSave(base: AppPreferences, ...patches: Array<Partial<AppPreferences>>) {
    pendingPreferencesSaveRef.current = {
      version: preferencesSaveVersionRef.current + 1,
      preferences: applyPreferencePatches(base, patches)
    };
    preferencesSaveVersionRef.current = pendingPreferencesSaveRef.current.version;
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
          const saved = await saveAppPreferences(pending.preferences);
          committedPreferencesRef.current = saved;
          setPreferences(saved);
          if (pendingPreferencesSaveRef.current?.version === pending.version) {
            pendingPreferencesSaveRef.current = null;
            setPreferencesStatus("Preferences saved.");
            return;
          }
          setPreferencesStatus("Saving preferences...");
        } catch (error) {
          if (pendingPreferencesSaveRef.current?.version === pending.version) {
            pendingPreferencesSaveRef.current = null;
            setPreferencesStatus(`Save failed: ${errorMessage(error)}`);
            return;
          }
          setPreferencesStatus("Saving preferences...");
        }
      }
    } finally {
      preferencesSaveInFlightRef.current = false;
    }
  }


  function confirmDirtyReplacement(confirmMessage: string): boolean {
    return !documentDirty || window.confirm(confirmMessage);
  }

  function adoptCurrentGame(result: CurrentGameResultDto) {
    documentGenerationRef.current = result.generation;
    currentGameRef.current = result;
    setCurrentGame(result);
  }

  function isCurrentDocumentGeneration(capturedGeneration: number): boolean {
    return documentGenerationRef.current === capturedGeneration;
  }

  function activeScopeFromRefs(): ReviewPresentationScope {
    return {
      generation: documentGenerationRef.current,
      selectedPath: currentGameRef.current?.selected_path.indices ?? [],
      requestToken: activeRequestTokenRef.current
    };
  }

  function beginReviewRequest(requestToken?: string): ReviewPresentationScope {
    const token = requestToken ?? createLocalRequestToken(() => {
      requestSerialRef.current += 1;
      return requestSerialRef.current;
    });
    activeRequestTokenRef.current = token;
    setActiveRequestToken(token);
    clearReviewData();
    return {
      generation: documentGenerationRef.current,
      selectedPath: [...(currentGameRef.current?.selected_path.indices ?? [])],
      requestToken: token
    };
  }

  function adoptRequestToken(captured: ReviewPresentationScope, requestToken: string): ReviewPresentationScope {
    if (!shouldPublishReviewPresentation(activeScopeFromRefs(), captured)) return captured;
    const next = { ...captured, requestToken };
    activeRequestTokenRef.current = requestToken;
    setActiveRequestToken(requestToken);
    return next;
  }

  function publishReviewPresentation(
    captured: ReviewPresentationScope,
    nextFrames: AnalysisFrameDto[],
    nextProblems: ProblemMarkerDto[]
  ): boolean {
    if (!shouldPublishReviewPresentation(activeScopeFromRefs(), captured)) return false;
    setPublishedScope(captured);
    setFrames(nextFrames);
    setProblems(nextProblems);
    setSelectedCandidateIndex(null);
    setCandidatePreview(null);
    return true;
  }

  async function artifactsFromCurrentGame(): Promise<{ serialized: string; projection: GameDto }> {
    const [serialized, projection] = await Promise.all([serializeCurrentGame(), projectCurrentGameMainline()]);
    return { serialized, projection };
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
        documentGenerationRef.current = 0;
        setCurrentGame(null);
        pendingSelectedPathRef.current = null;
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
        else await checkAnalysisCacheForGame(sgfInput, null, parsed, `${nativeCurrentGameUnavailable} ${previewMessage}`);
        return true;
      } catch (error) {
        setMessage(`${options.failurePrefix}: ${errorMessage(error)}`);
        return false;
      }
    }

    try {
      const result = await replaceCurrentGame(sgfInput, nativePath);
      adoptCurrentGame(result);
      const artifacts = await artifactsFromCurrentGame();
      pendingSelectedPathRef.current = result.selected_path;
      setChosenChildren(chosenFromPath(result.selected_path));
      setSgfText(sgfInput);
      setCurrentFilePath(result.native_path ?? nativePath);
      setFallbackFileName(result.native_path ? null : options.fallbackName ?? null);
      setDirty(result.dirty);
      setGame(artifacts.projection);
      setCurrentMove(result.snapshot.position.move_number);
      setFrames([]);
      setProblems([]);
      setSelectedCandidateIndex(null);
      const fileName = fileNameFromPath(result.native_path ?? options.fallbackName ?? "SGF");
      const success = options.successMessage(artifacts.projection, fileName);
      setMessage(success);
      if (options.checkCache === false) resetAnalysisCacheState();
      else await checkAnalysisCacheForGame(artifacts.serialized, result.native_path ?? null, artifacts.projection, success);
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
      adoptCurrentGame(result);
      const artifacts = await artifactsFromCurrentGame();
      pendingSelectedPathRef.current = result.selected_path;
      setSgfText(document.sgfText);
      setCurrentFilePath(result.native_path ?? document.path);
      setFallbackFileName(null);
      setDirty(result.dirty);
      setGame(artifacts.projection);
      setCurrentMove(result.snapshot.position.move_number);
      setFrames([]);
      setProblems([]);
      setSelectedCandidateIndex(null);
      const openedMessage = `Opened ${fileNameFromPath(result.native_path ?? document.path ?? "SGF")}: ${artifacts.projection.summary.move_count} moves.`;
      setMessage(openedMessage);
      await checkAnalysisCacheForGame(artifacts.serialized, result.native_path ?? document.path, artifacts.projection, openedMessage);
    } catch (error) {
      setMessage(`Open failed: ${errorMessage(error)}`);
    }
  }

  async function handleSaveSgfDocument(saveAs = false) {
    if (!nativeRuntime) {
      setMessage(nativeCurrentGameUnavailable);
      return;
    }
    if (!currentGame) {
      setMessage(nativeCurrentGameUnavailable);
      return;
    }
    try {
      const saved = await saveCurrentGame(saveAs ? null : documentPath, currentGame.selected_path, saveFileName);
      if (!saved) {
        setMessage("Save cancelled.");
        return;
      }
      adoptCurrentGame(saved);
      setCurrentFilePath(saved.native_path ?? null);
      setDirty(saved.dirty);
      setFallbackFileName(saved.native_path ? null : fallbackFileName);
      setMessage(`Saved ${saved.native_path ? fileNameFromPath(saved.native_path) : saveFileName}.`);
    } catch (error) {
      setMessage(`Save failed: ${errorMessage(error)}`);
    }
  }

  async function handleFakeAnalyze() {
    const captured = beginReviewRequest();
    try {
      if (!nativeRuntime) {
        const [parsed, result, replayed] = await Promise.all([parseSgfSummary(sgfText), fakeAnalyze(sgfText), replaySgfPositions(sgfText)]);
        const classified = await classifyProblems(result);
        if (!publishReviewPresentation(captured, result, classified)) return;
        setGame(parsed);
        setPositions(replayed);
        setCurrentMove(replayed.at(-1)?.move_number ?? parsed.moves.length);
        const cacheMessage = await saveAnalysisCacheForGame(sgfText, currentFilePath, parsed, result, classified, "fake");
        if (!shouldPublishReviewPresentation(activeScopeFromRefs(), captured)) return;
        setMessage(`${nativeCurrentGameUnavailable} 已生成 ${result.length} 个预览复盘局面。${cacheMessage}`);
        return;
      }
      const artifacts = await artifactsFromCurrentGame();
      const result = await fakeAnalyze(artifacts.serialized);
      const classified = await classifyProblems(result);
      if (!publishReviewPresentation(captured, result, classified)) return;
      setGame(artifacts.projection);
      const cacheMessage = await saveAnalysisCacheForGame(artifacts.serialized, documentPath, artifacts.projection, result, classified, "fake");
      if (!shouldPublishReviewPresentation(activeScopeFromRefs(), captured)) return;
      setMessage(`已生成 ${result.length} 个复盘局面，含候选与胜率。${cacheMessage}`);
    } catch (error) {
      setMessage(errorMessage(error));
    }
  }

  function clearSelectedNodeRunning(jobId: string) {
    if (selectedNodeJobRef.current?.job_id !== jobId) return;
    selectedNodeJobRef.current = null;
    setSelectedNodeRunning(false);
  }

  function clearWholeGameLane() {
    wholeGameJobRef.current = null;
    setWholeGameRunning(false);
    setWholeGameProgress(null);
  }

  handleSelectedNodeJobRef.current = (job: AnalysisJobEventDto) => {
    const pending = selectedNodeJobRef.current;
    if (!matchesPendingAnalysisJob(pending, job)) return;
    const publication = {
      run_id: pending.run_id,
      job_id: pending.job_id,
      generation: pending.generation,
      node_path: pending.node_path
    };
    if (job.outcome === "completed") {
      const captured: ReviewPresentationScope = {
        generation: job.generation,
        selectedPath: [...job.node_path.indices],
        requestToken: job.job_id
      };
      if (!admitsAnalysisPublication(job, publication) || !job.frame || !shouldPublishReviewPresentation(activeScopeFromRefs(), captured)) {
        clearSelectedNodeRunning(job.job_id);
        return;
      }
      const frame = job.frame;
      void (async () => {
        try {
          const classified = await classifyProblems([frame]);
          if (!publishReviewPresentation(captured, [frame], classified)) return;
          setMessage(`KataGo analysis completed for move ${frame.turn} with ${frame.visits} visits.`);
        } catch (error) {
          setMessage(`KataGo analysis failed: ${errorMessage(error)}`);
        } finally {
          clearSelectedNodeRunning(job.job_id);
        }
      })();
      return;
    }
    if (job.outcome === "cancelled" || job.outcome === "superseded" || job.outcome === "timeout" || job.outcome === "failed") {
      if (job.outcome === "timeout") {
        const snapshot = engineSnapshotRef.current;
        const run = runFromSnapshot(snapshot);
        if (!admitsForegroundEngineJobs(snapshot) || run?.run_id !== pending.run_id) {
          clearSelectedNodeRunning(job.job_id);
          return;
        }
        setMessage("Selected-node analysis timed out.");
      } else if (job.outcome === "failed") setMessage(job.failure?.message ?? "Selected-node analysis failed.");
      else if (job.outcome === "cancelled") setMessage("Selected-node analysis cancelled.");
      clearSelectedNodeRunning(job.job_id);
    }
  };

  async function handleRunKataGo(_profile: EngineProfileDto, maxVisits: number) {
    const run = runFromSnapshot(engineSnapshot);
    const game = currentGameRef.current;
    if (!run || !game) {
      setMessage("Selected-node analysis requires a Ready Foreground Engine Run and current game.");
      return;
    }
    const visits = resolveAnalysisMaxVisits(maxVisits, preferences);
    try {
      const started = await startSelectedNodeAnalysis({
        runId: run.run_id,
        generation: game.generation,
        nodePath: game.selected_path,
        maxVisits: visits
      });
      beginReviewRequest(started.job_id);
      selectedNodeJobRef.current = started;
      setSelectedNodeRunning(true);
      setMessage(`Running KataGo analysis (${started.job_id})...`);
    } catch (error) {
      setMessage(`KataGo analysis failed: ${errorMessage(error)}`);
    }
  }

  handleWholeGameJobRef.current = (job: AnalysisJobEventDto) => {
    const pending = wholeGameJobRef.current;
    if (!matchesPendingAnalysisJob(pending, job)) return;
    if (job.outcome === "progress") {
      setWholeGameProgress({
        completed: job.completed ?? 0,
        expected: job.expected ?? 0
      });
      return;
    }
    if (job.outcome === "completed" || job.outcome === "cancelled" || job.outcome === "failed" || job.outcome === "timeout") {
      if (job.outcome === "completed") {
        setMessage(`整局分析完成 ${job.completed ?? 0}/${job.expected ?? 0}`);
      } else if (job.outcome === "cancelled") {
        setMessage(job.failure?.message ?? "整局分析已取消");
      } else if (job.outcome === "failed") {
        setMessage(job.failure?.message ?? "整局分析失败");
      } else {
        setMessage("整局分析超时");
      }
      clearWholeGameLane();
    }
  };

  async function handleAnalyzeKataGoGame(runId: string, maxVisits: number) {
    const visits = resolveAnalysisMaxVisits(maxVisits, preferences);
    try {
      const artifacts = nativeRuntime
        ? await artifactsFromCurrentGame()
        : { serialized: sgfText, projection: await parseSgfSummary(sgfText) };
      if (!nativeRuntime) setMessage(nativeCurrentGameUnavailable);
      const started = await startKataGoGameAnalysis(runId, artifacts.serialized, visits);
      wholeGameJobRef.current = started;
      setWholeGameRunning(true);
      setWholeGameProgress(null);
      setMessage(`Full-game KataGo analysis started (${started.job_id}).`);
    } catch (error) {
      setMessage(errorMessage(error));
    }
  }

  async function handleCancelSelectedNodeAnalysis() {
    const selected = selectedNodeJobRef.current;
    if (!selected) return;
    try {
      setMessage("Cancelling selected-node KataGo analysis...");
      await cancelSelectedNodeAnalysis({ runId: selected.run_id, jobId: selected.job_id });
    } catch (error) {
      setMessage(`Cancel failed: ${errorMessage(error)}`);
    }
  }

  async function handleCancelWholeGameAnalysis() {
    const pending = wholeGameJobRef.current;
    if (!pending) return;
    try {
      setMessage("Cancelling full-game KataGo analysis...");
      await cancelKataGoAnalysis(pending.run_id, pending.job_id);
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

  async function handleCommitPersonalComment(comment: string) {
    if (!currentGame) return;
    if (!nativeRuntime) {
      setMessage(nativeCurrentGameUnavailable);
      return;
    }
    if (comment === currentGame.snapshot.personal_comment) return;
    const previousGeneration = currentGame.generation;
    const editedPath = currentGame.selected_path;
    try {
      const result = await setCurrentGamePersonalComment(editedPath, comment);
      const latestPath = pendingSelectedPathRef.current ?? currentGameRef.current?.selected_path ?? editedPath;
      const stillOnEditedNode = samePath(latestPath, editedPath);
      setCurrentGame((prev) => {
        if (stillOnEditedNode || prev === null) return result;
        return {
          ...result,
          selected_path: prev.selected_path,
          snapshot: prev.snapshot
        };
      });
      if (stillOnEditedNode) {
        setChosenChildren((prev) => rememberChosenChildren(prev, result.selected_path));
        setCurrentMove(result.snapshot.position.move_number);
        setSelectedCandidateIndex(null);
      }
      setDirty(result.dirty);
      if (result.generation === previousGeneration) return;
      const artifacts = await artifactsFromCurrentGame();
      setGame(artifacts.projection);
      clearReviewData();
      resetAnalysisCacheState();
      setMessage("已更新选中节点的个人评论。");
    } catch (error) {
      setMessage(`评论更新失败: ${errorMessage(error)}`);
    }
  }

  async function selectNode(path: NodePath) {
    if (!currentGame || navigatingRef.current) return;
    if (samePath(path, currentGame.selected_path)) return;
    navigatingRef.current = true;
    pendingSelectedPathRef.current = path;
    try {
      const result = await selectCurrentGameNode(path);
      documentGenerationRef.current = Math.max(documentGenerationRef.current, result.generation);
      setCurrentGame((prev) => {
        if (prev !== null && prev.generation > result.generation) {
          return {
            ...prev,
            selected_path: result.selected_path,
            snapshot: result.snapshot
          };
        }
        return result;
      });
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

  async function playAt(vertex: MoveVertex) {
    if (!nativeRuntime) {
      setMessage(nativeCurrentGameUnavailable);
      return;
    }
    if (!currentGame || pendingBoardIntentRef.current !== null) return;
    const intentKey = vertex === "pass" ? "pass" : `${vertex.point.x},${vertex.point.y}`;
    pendingBoardIntentRef.current = intentKey;
    const previousGeneration = currentGame.generation;
    try {
      const result = await playCurrentGame(currentGame.selected_path, vertex);
      documentGenerationRef.current = result.generation;
      setCurrentGame(result);
      pendingSelectedPathRef.current = result.selected_path;
      setChosenChildren((prev) => rememberChosenChildren(prev, result.selected_path));
      setDirty(result.dirty);
      setCurrentMove(result.snapshot.position.move_number);
      setSelectedCandidateIndex(null);
      setBoardIntentFeedback(null);
      setMessage("落子已接受。");
      if (result.generation !== previousGeneration) {
        clearReviewData();
        resetAnalysisCacheState();
        const artifacts = await artifactsFromCurrentGame();
        if (documentGenerationRef.current !== result.generation) return;
        setGame(artifacts.projection);
      }
    } catch (error) {
      const feedback = `落子失败: ${errorMessage(error)}`;
      setBoardIntentFeedback(feedback);
      setMessage(feedback);
    } finally {
      if (pendingBoardIntentRef.current === intentKey) pendingBoardIntentRef.current = null;
    }
  }

  async function handleRemoveVariation() {
    if (!currentGame || selectedPath.indices.length === 0) return;
    try {
      const result = await removeCurrentGameVariation(selectedPath);
      adoptCurrentGame(result);
      pendingSelectedPathRef.current = result.selected_path;
      setChosenChildren(chosenFromPath(result.selected_path));
      setDirty(result.dirty);
      setCurrentMove(result.snapshot.position.move_number);
      clearReviewData();
      resetAnalysisCacheState();
      const artifacts = await artifactsFromCurrentGame();
      if (!isCurrentDocumentGeneration(result.generation)) return;
      setGame(artifacts.projection);
      setMessage("已删除选中变化，并回到其父节点。");
    } catch (error) {
      setMessage(`删除变化失败: ${errorMessage(error)}`);
    }
  }

  async function checkAnalysisCacheForGame(text: string, filePath: string | null, parsed: GameDto, baseMessage: string) {
    const captured = beginReviewRequest();
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
        if (!publishReviewPresentation(captured, payload.frames, payload.problems)) return;
        setCurrentMove(nativeRuntime
          ? (currentGameRef.current?.snapshot.position.move_number ?? payload.frames.at(-1)?.turn ?? parsed.moves.length)
          : clampMoveNumberToPositions(positions, payload.frames.at(-1)?.turn ?? parsed.moves.length));
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

  function previewCandidate(index: number | null) {
    setCandidatePreview(index === null ? null : { index, scope: activeScope });
  }

  function selectCandidate(index: number) {
    setCandidatePreview(null);
    setSelectedCandidateIndex(index);
  }

  function clearReviewData() {
    setFrames([]);
    setProblems([]);
    setSelectedCandidateIndex(null);
    setCandidatePreview(null);
    setPublishedScope(null);
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
      busy={false}
      dirty={documentDirty}
      documentName={documentName}
      engineLabel={engineLabel}
      engineReady={engineReady}
      engineSwitcher={{
        profiles: engineProfiles.map((profile) => ({ id: profile.id, name: profile.profile.name })),
        selectedProfileId: runFromSnapshot(engineSnapshot)?.profile_id
          ?? (engineSnapshot.lifecycle.state === "no_engine" ? "" : ""),
        canStop: canStopForegroundEngine(engineSnapshot),
        canRestart: canRestartForegroundEngine(engineSnapshot),
        failureMessage: visibleEngineFailure?.message ?? null,
        failureKind: visibleEngineFailure?.kind ?? null,
        failureOperation: visibleEngineFailure?.operation ?? null,
        onSelectProfile: (profileId) => void handleSelectSwitcherProfile(profileId),
        onStop: () => {
          void stopForegroundEngine().catch((error) => {
            setMessage(error instanceof Error ? error.message : String(error));
          });
        },
        onRestart: () => {
          void restartForegroundEngine().catch((error) => {
            setMessage(error instanceof Error ? error.message : String(error));
          });
        }
      }}
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
      selectedNodeRunning={selectedNodeRunning}
      wholeGameRunning={wholeGameRunning}
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
      onCancelSelectedNode={() => void handleCancelSelectedNodeAnalysis()}
      onCancelWholeGame={() => void handleCancelWholeGameAnalysis()}
      onAbout={() => setMessage("LizzieYzy Next 0.1.0 · 桌面复盘工作区")}
      onOpenShortcutReference={() => setShortcutReferenceOpen(true)}
      onCopySgf={() => void handleCopySgf()}
      onPasteSgf={() => void handlePasteSgf()}
      onClearBoard={() => void handleNewGame()}
      onPass={() => void playAt("pass")}
      onRemoveVariation={() => void handleRemoveVariation()}
      canRemoveVariation={canRemoveVariation}
      onFirstMove={() => handleMoveSelect(0)}
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
          <WinrateChart frames={visibleFrames} currentMove={currentMove} />
          <div id="board-layers" />
        </div>
        <AnalysisPanel
          pane="commentary"
          frame={visibleCurrentFrame}
          problems={visibleProblems}
          moves={game.moves}
          boardSize={game.summary.board_size}
          currentMove={currentMove}
          currentPosition={currentPosition}
          personalComment={selectedPersonalComment}
          generatedInformation={selectedGeneratedInformation}
          commentEditorEnabled={nativeRuntime && Boolean(currentGame)}
          onCommitPersonalComment={(comment) => void handleCommitPersonalComment(comment)}
          selectedCandidateIndex={selectedCandidateIndex}
          onSelectCandidate={selectCandidate}
          onSelectProblem={handleMoveSelect}
        />
      </aside>
      <div className="diagram">
        <BoardCanvas
          position={currentPosition}
          analysis={visibleCurrentFrame}
          selectedCandidateIndex={selectedCandidateIndex}
          previewScope={activeScope}
          onCandidatePreview={previewCandidate}
          moves={game.moves}
          showCoordinates={showCoordinates}
          showMoveNumbers={showMoveNumbers}
          overlayMode={overlayMode}
          onOverlayModeChange={setOverlayMode}
          hideCandidates={(currentPosition.to_play === "black" && !showBlackCandidates) || (currentPosition.to_play === "white" && !showWhiteCandidates)}
          onPointClick={(point) => void playAt({ point })}
        />
        {boardIntentFeedback ? (
          <p className="board-intent-status" role="status" aria-live="polite">{boardIntentFeedback}</p>
        ) : null}
      </div>
      <aside className="sheet-col">
        <AnalysisPanel
          pane="reference"
          frame={visibleCurrentFrame}
          problems={visibleProblems}
          moves={game.moves}
          boardSize={game.summary.board_size}
          currentMove={currentMove}
          currentPosition={currentPosition}
          selectedCandidateIndex={selectedCandidateIndex}
          previewCandidateIndex={previewCandidateIndex}
          onSelectCandidate={selectCandidate}
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
      canRemoveVariation={canRemoveVariation}
      siblingLabel={siblingLabel}
      nativeUnavailable={nativeRuntime ? undefined : nativeCurrentGameUnavailable}
      onParent={handleParent}
      onNext={handleNextChild}
      onPrevSibling={handlePrevSibling}
      onNextSibling={handleNextSibling}
      onRemoveVariation={() => void handleRemoveVariation()}
      engineReady={engineReady}
      selectedNodeRunning={selectedNodeRunning}
      wholeGameRunning={wholeGameRunning}
      wholeGameProgress={wholeGameProgress}
      onAnalyzeOnce={() => handleEngineCommand("once")}
      onAnalyzeGame={() => handleEngineCommand("game")}
      onCancelSelectedNode={() => void handleCancelSelectedNodeAnalysis()}
      onCancelWholeGame={() => void handleCancelWholeGameAnalysis()}
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
          <strong title={documentPath ?? documentName}>{documentName}{documentDirty ? " *" : ""}</strong>
          <span>{documentDirty ? "未保存" : "已保存"}</span>
        </div>
        <textarea value={sgfText} onChange={(event) => {
          setSgfText(event.target.value);
          setMessage("这段文本只是载入输入。解析棋谱才会替换当前对局。");
        }} spellCheck={false} aria-label="棋谱载入文本" />
        <div className="button-row">
          <label className="file-button">
            导入棋谱
            <input type="file" accept=".sgf,.txt,application/x-go-sgf,text/plain" disabled={false} onChange={(event) => void handleImportFile(event.target.files?.[0] ?? null)} />
          </label>
          <button type="button" onClick={() => void loadSample()} disabled={false}>载入示例</button>
        </div>
      </div> : null}
      {sheet === "sync" ? <ProviderPanel disabled={false} onImport={handleProviderImport} /> : null}
      <div hidden={sheet !== "engine"}>
        <EngineSetupPanel
          disabled={false}
          engineSnapshot={engineSnapshot}
        />
      </div>
      {sheet === "prefs" ? <PreferencesPanel
        preferences={preferences}
        status={preferencesStatus}
        disabled={false}
        onChange={(nextPreferences) => void handlePreferencesChange(nextPreferences)}
      /> : null}
    </section>
    {shortcutReferenceOpen ? (
      <ShortcutReference
        entries={shortcutRegistry.referenceEntries()}
        onClose={() => setShortcutReferenceOpen(false)}
      />
    ) : null}
  </main>;
}

function preferencePatch(from: AppPreferences, to: AppPreferences): Partial<AppPreferences> {
  const patch: Partial<AppPreferences> = {};
  (Object.keys(to) as Array<keyof AppPreferences>).forEach((key) => {
    if (from[key] !== to[key]) {
      Object.assign(patch, { [key]: to[key] });
    }
  });
  return patch;
}

function applyPreferencePatches(base: AppPreferences, patches: Array<Partial<AppPreferences>>): AppPreferences {
  return patches.reduce<AppPreferences>(
    (current, patch) => normalizeAppPreferences({ ...current, ...patch }),
    base
  );
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

function matchesPendingAnalysisJob(pending: AnalysisJobStartedDto | null, job: AnalysisJobEventDto): pending is AnalysisJobStartedDto {
  return pending != null
    && pending.run_id === job.run_id
    && pending.job_id === job.job_id
    && pending.lane === job.lane
    && pending.generation === job.generation
    && samePath(pending.node_path, job.node_path);
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
