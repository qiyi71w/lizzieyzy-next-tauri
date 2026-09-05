import { useEffect, useMemo, useRef, useState } from "react";
import { BoardCanvas } from "./components/BoardCanvas";
import { WinrateChart } from "./components/WinrateChart";
import { AnalysisPanel } from "./components/AnalysisPanel";
import { EngineSetupPanel } from "./components/EngineSetupPanel";
import { AppChrome, BottomBar, type OverlayMode, type SheetId } from "./components/AppChrome";
import { PreferencesPanel } from "./components/PreferencesPanel";
import { ShortcutReference } from "./components/ShortcutReference";
import { DocumentDepartureDialog } from "./components/DocumentDepartureDialog";
import { ApplicationTeardownDialog } from "./components/ApplicationTeardownDialog";
import { CurrentGameRecoveryDialog } from "./components/CurrentGameRecoveryDialog";
import { ProviderPanel } from "./components/ProviderPanel";
import {
  cancelKataGoAnalysis,
  cancelSelectedNodeAnalysis,
  classifyProblems,
  fakeAnalyze,
  getHealth,
  isTauriRuntime,
  nativeCurrentGameUnavailable,
  nativeSyntheticAnalysisUnavailable,
  openSgfDocument,
  parseSgfSummary,
  playCurrentGame,
  prepareApplicationExit,
  prepareDocumentReplacement,
  projectCurrentGameMainline,
  replaySgfPositions,
  resolveApplicationExit,
  resolveDocumentReplacement,
  retryApplicationTeardown,
  confirmApplicationExitAnyway,
  confirmNativeExit,
  subscribeApplicationExitRequested,
  inspectCurrentGameRecovery,
  restoreCurrentGameRecovery,
  discardCurrentGameRecovery,
  retryCurrentGameRecovery,
  subscribeCurrentGameRecoveryProtection,
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
import { loadAppPreferences, saveAppPreferences } from "./api/preferences";
import { clampMoveNumberToPositions, createDemoGame, replayGamePositions, selectExactPosition } from "./domain/board";
import { defaultAppPreferences, normalizeAppPreferences, type AppPreferences } from "./domain/preferences";
import { buildNextMoveReviewMarkers, cycleNextMoveReviewMarker } from "./domain/nextMoveReviewMarker";
import { admitsChartSeriesChange, buildWinrateChartModel, displayedWinrate } from "./domain/winrateChart";
import { providerDocumentName, providerLabel, providerSourceLabel, type ProviderImportResult } from "./domain/providers";
import { admitsAnalysisAttachment, admitsAnalysisPublication, admitsWholeGameNodeResult, matchesWholeGameJobIdentity } from "./domain/analysisJob";
import { createShortcutRegistry } from "./domain/shortcuts";
import {
  createLocalRequestToken,
  shouldPublishReviewPresentation,
  type ReviewPresentationScope
} from "./domain/reviewPresentation";
import {
  variationReplayIdentity,
  variationReplayIdentityKey,
  variationReplayPointSteps
} from "./domain/variationReplay";
import type { AnalysisFrameDto, AnalysisJobEventDto, AnalysisJobStartedDto, AppHealthDto, ApplicationExitActionDto, ApplicationExitOutcomeDto, CurrentGameResultDto, DocumentDepartureActionDto, EngineProfileDto, EngineProfileRecordDto, EngineFailureDto, ForegroundEngineSnapshotDto, GameDto, MoveVertex, NodePath, PositionDto, ProblemMarkerDto, RecoveryProtectionDto, RecoveryStartupDto, SgfTreeNodeDto } from "./domain/types";

const demoSgf = "(;GM[1]FF[4]SZ[19]KM[7.5]PB[李昌镐]PW[芮乃伟]RE[B+R];B[pd];W[dd];B[pp];W[dp];B[jq];W[qj];B[nc];W[fc];B[qf];W[cn];B[cp];W[do];B[co];W[dn];B[fq];W[eq];B[fp];W[gp];B[gq];W[hp])";
const emptySgf = "(;GM[1]FF[4]SZ[19]KM[7.5]PB[黑]PW[白])";
const demoGame = createDemoGame();
const emptyChartRoot: SgfTreeNodeDto = { properties: [], children: [] };
type WholeGameProgress = { completed: number; expected: number; remaining: number };
type PendingPreferencesSave = { version: number; preferences: AppPreferences };
type CandidatePreview = { index: number; scope: ReviewPresentationScope };

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
  const analysisRunIdRef = useRef<string | null>(null);
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
  const [referenceRailCollapsed, setReferenceRailCollapsed] = useState(false);
  const [replayProgress, setReplayProgress] = useState({ identity: "", prefix: 0 });
  const [overlayMode, setOverlayMode] = useState<OverlayMode>("candidates");
  const [autoPlaying, setAutoPlaying] = useState(false);
  const [shortcutReferenceOpen, setShortcutReferenceOpen] = useState(false);
  const [keyboardPlacement, setKeyboardPlacement] = useState(false);
  const [departurePrompt, setDeparturePrompt] = useState<{
    message: string;
    choose: (action: DocumentDepartureActionDto) => void;
  } | null>(null);
  const [teardownPrompt, setTeardownPrompt] = useState<{
    message: string;
    choose: (action: "retry" | "exit_anyway") => void;
  } | null>(null);
  const [recoveryPrompt, setRecoveryPrompt] = useState<Extract<RecoveryStartupDto, { status: "abnormal" }> | null>(null);
  const [recoveryProtection, setRecoveryProtection] = useState<RecoveryProtectionDto>({ status: "protected" });
  const exitInFlightRef = useRef(false);
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
  const wholeGameResultsRef = useRef<Map<string, AnalysisFrameDto>>(new Map());
  const handleSelectedNodeJobRef = useRef<(job: AnalysisJobEventDto) => void>(() => undefined);
  const handleWholeGameJobRef = useRef<(job: AnalysisJobEventDto) => void>(() => undefined);

  useEffect(() => {
    getHealth()
      .then(setHealth)
      .catch((error: unknown) => setMessage(errorMessage(error)));
  }, []);

  useEffect(() => {
    let isMounted = true;
    void (async () => {
      let loadedPrefs = defaultAppPreferences;
      try {
        const loaded = await loadAppPreferences();
        if (!isMounted) return;
        settleLoadedPreferences(loaded.preferences, loaded.recovery?.message ?? "Preferences loaded.");
        loadedPrefs = loaded.preferences;
      } catch (error: unknown) {
        if (!isMounted) return;
        settleLoadedPreferences(defaultAppPreferences, `Load failed: ${errorMessage(error)}`);
      }
      if (!isTauriRuntime()) return;
      let startup: RecoveryStartupDto = { status: "none" };
      try {
        startup = await inspectCurrentGameRecovery();
      } catch (error: unknown) {
        if (!isMounted) return;
        setMessage(`恢复检查失败: ${errorMessage(error)}`);
      }
      if (!isMounted) return;
      if (startup.status === "abnormal") {
        setRecoveryPrompt(startup);
        return;
      }
      if (startup.status === "unreadable") {
        setMessage(startup.message);
      }
      if (startup.status === "normal" && loadedPrefs.restoreLastSession) {
        try {
          await restoreRecoveredDocument("已恢复上次棋谱。");
          return;
        } catch (error: unknown) {
          if (!isMounted) return;
          setMessage(`恢复失败: ${errorMessage(error)}`);
        }
      }
      await applyReplacement(demoSgf, null, {
        confirmMessage: "放弃未保存的棋谱并载入示例？",
        fallbackName: "sample.sgf",
        successMessage: (projection) => `Sample SGF restored: ${projection.summary.move_count} moves.`,
        failurePrefix: "Sample load failed"
      });
      if (!isMounted) return;
      if (startup.status === "unreadable") {
        setMessage(startup.message);
      }
    })();
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
  const treeFrame = currentGame?.snapshot.primary_analysis ?? undefined;
  const currentFrame = useMemo(
    () => {
      const sessionFrame = visibleFrames.length <= 1
        ? visibleFrames[0]
        : visibleFrames.find((frame) => frame.turn === currentMove) ?? visibleFrames.at(-1);
      return sessionFrame ?? treeFrame;
    },
    [visibleFrames, currentMove, treeFrame]
  );
  const visibleCurrentFrame = useMemo(() => applyPreferencesToFrame(currentFrame, preferences), [currentFrame, preferences]);
  const previewCandidateIndex = candidatePreview
    && presentationLive
    && shouldPublishReviewPresentation(activeScope, candidatePreview.scope)
    ? candidatePreview.index
    : null;
  const hideCandidates = (currentPosition.to_play === "black" && !showBlackCandidates)
    || (currentPosition.to_play === "white" && !showWhiteCandidates);
  const activeCandidateIndex = previewCandidateIndex ?? selectedCandidateIndex ?? 0;
  const activeCandidate = visibleCurrentFrame?.candidates[activeCandidateIndex]
    ?? visibleCurrentFrame?.candidates[0]
    ?? null;
  const replayCandidate = currentFrame?.candidates[activeCandidateIndex]
    ?? currentFrame?.candidates[0]
    ?? null;
  const replaySteps = variationReplayPointSteps(replayCandidate);
  const replayIdentityKeyValue = variationReplayIdentityKey(variationReplayIdentity(selectedPath, replayCandidate));
  const replayArmed = preferences.variationReplayEnabled && replaySteps.length > 0;
  const replayEligible = replayArmed && (
    (preferences.showCandidates && overlayMode === "candidates" && !hideCandidates)
    || (preferences.showCandidates && preferences.subBoardContentMode === "variation" && !referenceRailCollapsed)
  );
  const replayPrefix = replayArmed
    ? (replayProgress.identity !== replayIdentityKeyValue ? 1 : Math.max(replayProgress.prefix, 1))
    : undefined;
  const replayIntervalRef = useRef(preferences.variationReplayIntervalMs);
  replayIntervalRef.current = preferences.variationReplayIntervalMs;

  useEffect(() => {
    if (!replayArmed) {
      if (replayProgress.identity !== "" || replayProgress.prefix !== 0) {
        setReplayProgress({ identity: "", prefix: 0 });
      }
      return;
    }
    if (replayProgress.identity !== replayIdentityKeyValue) {
      setReplayProgress({ identity: replayIdentityKeyValue, prefix: 1 });
      return;
    }
    if (!replayEligible || replayProgress.prefix >= replaySteps.length) return;
    const timer = window.setTimeout(() => {
      setReplayProgress((current) => (
        current.identity !== replayIdentityKeyValue
          ? current
          : { identity: current.identity, prefix: Math.min(current.prefix + 1, replaySteps.length) }
      ));
    }, replayIntervalRef.current);
    return () => window.clearTimeout(timer);
  }, [
    replayArmed,
    replayEligible,
    replayIdentityKeyValue,
    replayProgress.identity,
    replayProgress.prefix,
    replaySteps.length
  ]);
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
  const chartModel = useMemo(() => buildWinrateChartModel({
    root: currentGame?.tree ?? emptyChartRoot,
    chosen: chosenChildren,
    selectedPathLength: currentGame ? selectedPath.indices.length : currentMove,
    selectedToPlay: currentPosition.to_play,
    settings: preferences
  }), [currentGame, chosenChildren, selectedPath.indices.length, currentMove, currentPosition.to_play, preferences]);
  const currentChartPoint = chartModel.points.find((point) => point.moveNumber === chartModel.currentMove);
  const chartWinrate = currentChartPoint
    ? displayedWinrate(currentChartPoint, chartModel.perspective, chartModel.selectedToPlay)
    : null;
  const chartTitleSide = chartModel.perspective === "sideToPlay" && chartModel.selectedToPlay === "white" ? "白" : "黑";
  const nextMoveMarkers = useMemo(() => buildNextMoveReviewMarkers({
    mode: preferences.nextMoveReviewMarker,
    selectedNode,
    selectedIsRoot: selectedPath.indices.length === 0,
    toPlay: currentPosition.to_play,
    boardSize: currentPosition.board_size
  }), [preferences.nextMoveReviewMarker, selectedNode, selectedPath.indices.length, currentPosition.to_play, currentPosition.board_size]);
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
    shortcutRegistry.bind("game.human-vs-engine", () => {
      setMessage("人机对局尚未接入，N 不会新建棋谱。");
    });
    shortcutRegistry.bind("analysis.continuous", () => {
      setMessage("连续分析尚未接入，Space 不会落子或改为一次性分析。");
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
    shortcutRegistry.bind("review.next-move-marker", () => {
      void handlePreferencesChange({
        ...preferences,
        nextMoveReviewMarker: cycleNextMoveReviewMarker(preferences.nextMoveReviewMarker)
      });
    });
    shortcutRegistry.bind("review.autoplay", () => {
      setAutoPlaying((value) => !value);
    });
    shortcutRegistry.bind("help.shortcut-reference", () => {
      setShortcutReferenceOpen((value) => !value);
    });

    function onKey(event: KeyboardEvent) {
      if (departurePrompt) {
        if (event.key === "Escape") {
          event.preventDefault();
          departurePrompt.choose("cancel");
        }
        return;
      }
      if (
        event.key === "Escape"
        && keyboardPlacement
        && !(event.target instanceof Element && event.target.closest('[role="dialog"]'))
      ) {
        event.preventDefault();
        setKeyboardPlacement(false);
        return;
      }
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
    departurePrompt,
    keyboardPlacement,
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
        const nextRunId = snapshot.lifecycle.state === "ready" ? snapshot.lifecycle.run.run_id : null;
        if (analysisRunIdRef.current !== nextRunId) {
          if (analysisRunIdRef.current !== null) {
            clearReviewData();
            resetWholeGameSession();
            selectedNodeJobRef.current = null;
            setSelectedNodeRunning(false);
          }
          analysisRunIdRef.current = nextRunId;
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
    const normalized = normalizeAppPreferences(nextPreferences);
    const seriesChanged = normalized.winrateLine !== preferences.winrateLine
      || normalized.scoreLeadLine !== preferences.scoreLeadLine;
    if (seriesChanged && !admitsChartSeriesChange(preferences, {
      winrateLine: normalized.winrateLine,
      scoreLeadLine: normalized.scoreLeadLine
    }, chartModel.scoreAvailable)) {
      return;
    }
    const patch = preferencePatch(preferences, normalized);
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


  function adoptCurrentGame(result: CurrentGameResultDto) {
    documentGenerationRef.current = result.generation;
    currentGameRef.current = result;
    setCurrentGame(result);
  }

  async function refreshCurrentGameAfterAttach() {
    const game = currentGameRef.current;
    if (!nativeRuntime || !game) return;
    try {
      const refreshed = await selectCurrentGameNode(game.selected_path);
      if (!refreshed || !isCurrentDocumentGeneration(refreshed.generation)) return;
      adoptCurrentGame(refreshed);
      setDirty(refreshed.dirty);
    } catch {
      return;
    }
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

  function requestDepartureDecision(message: string): Promise<DocumentDepartureActionDto> {
    return new Promise((resolve) => {
      setDeparturePrompt({
        message,
        choose: (action) => {
          setDeparturePrompt(null);
          resolve(action);
        }
      });
    });
  }

  function requestTeardownDecision(message: string): Promise<"retry" | "exit_anyway"> {
    return new Promise((resolve) => {
      setTeardownPrompt({
        message,
        choose: (action) => {
          setTeardownPrompt(null);
          resolve(action);
        }
      });
    });
  }

  function isDepartureInProgressError(error: unknown): boolean {
    if (!error || typeof error !== "object") return false;
    const kind = Reflect.get(error, "kind");
    const message = Reflect.get(error, "message");
    return kind === "departure_in_progress"
      || (typeof message === "string" && message.includes("already in progress"));
  }

  async function finishExitTeardown(departureId: number, outcome: ApplicationExitOutcomeDto): Promise<void> {
    let current = outcome;
    const selectedPath = currentGameRef.current?.selected_path ?? { indices: [] };
    while (current.teardown?.status === "timed_out") {
      const outstanding = current.teardown.outstanding;
      const choice = await requestTeardownDecision(
        `退出清理未完成：${outstanding.join("、") || "owned resources"}。重试还是强制退出？`
      );
      current = choice === "retry"
        ? await retryApplicationTeardown({ departureId, selectedPath })
        : await confirmApplicationExitAnyway({ departureId, selectedPath, outstanding });
      if (current.analysis_stopped) {
        clearLocalAnalysisSession();
      }
      if (current.current) {
        adoptCurrentGame(current.current);
        setDirty(current.current.dirty);
        setCurrentFilePath(current.current.native_path ?? null);
      }
      if (choice === "exit_anyway") break;
    }
    if (current.recovery_persist_error) {
      setMessage(current.recovery_persist_error);
      setRecoveryProtection({ status: "unprotected", message: current.recovery_persist_error });
      return;
    }
    await confirmNativeExit();
  }

  async function handleApplicationExit() {
    if (!nativeRuntime) {
      setMessage(nativeCurrentGameUnavailable);
      return;
    }
    if (exitInFlightRef.current || departurePrompt || teardownPrompt) return;
    exitInFlightRef.current = true;
    try {
      const admission = await prepareApplicationExit();
      const action: ApplicationExitActionDto = admission.status === "needs_decision"
        ? await requestDepartureDecision("当前棋谱尚未保存。保存后退出，放弃更改，还是取消退出？")
        : "continue";
      const outcome = await resolveApplicationExit({
        departureId: admission.departure_id,
        action,
        selectedPath: currentGameRef.current?.selected_path ?? { indices: [] },
        defaultFileName: saveFileName
      });
      if (outcome.analysis_stopped) {
        clearLocalAnalysisSession();
      }
      if (!outcome.committed) {
        setMessage(action === "cancel" ? "已取消退出，当前棋谱和分析保持不变。" : outcome.message);
        return;
      }
      if (outcome.current) {
        adoptCurrentGame(outcome.current);
        setDirty(outcome.current.dirty);
        setCurrentFilePath(outcome.current.native_path ?? null);
      }
      await finishExitTeardown(admission.departure_id, outcome);
    } catch (error) {
      if (isDepartureInProgressError(error)) return;
      setMessage(`退出失败: ${errorMessage(error)}`);
    } finally {
      exitInFlightRef.current = false;
    }
  }

  const handleApplicationExitRef = useRef(handleApplicationExit);
  handleApplicationExitRef.current = handleApplicationExit;
  useEffect(() => {
    if (!nativeRuntime) return;
    let cancelled = false;
    let unlisten: () => void = () => undefined;
    void subscribeApplicationExitRequested(() => {
      void handleApplicationExitRef.current();
    }).then((fn) => {
      if (cancelled) fn();
      else unlisten = fn;
    });
    return () => {
      cancelled = true;
      unlisten();
    };
  }, [nativeRuntime]);

  useEffect(() => {
    if (!nativeRuntime) return;
    let cancelled = false;
    let unlisten: () => void = () => undefined;
    void subscribeCurrentGameRecoveryProtection((protection) => {
      if (!cancelled) {
        setRecoveryProtection(protection);
        if (protection.status === "unprotected") setMessage(protection.message);
      }
    }).then((fn) => {
      if (cancelled) fn();
      else unlisten = fn;
    });
    return () => {
      cancelled = true;
      unlisten();
    };
  }, [nativeRuntime]);

  function clearLocalAnalysisSession() {
    selectedNodeJobRef.current = null;
    setSelectedNodeRunning(false);
    resetWholeGameSession();
  }

  async function adoptCommittedReplacement(
    result: CurrentGameResultDto,
    sgfInput: string,
    nativePath: string | null,
    options: {
      fallbackName?: string | null;
      successMessage: (projection: GameDto, fileName: string) => string;
    }
  ) {
    adoptCurrentGame(result);
    clearLocalAnalysisSession();
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
    setKeyboardPlacement(false);
    const fileName = fileNameFromPath(result.native_path ?? options.fallbackName ?? "SGF");
    setMessage(options.successMessage(artifacts.projection, fileName));
  }

  async function finishNativeReplacement(
    departureId: number,
    action: DocumentDepartureActionDto,
    sgfInput: string,
    nativePath: string | null,
    options: {
      fallbackName?: string | null;
      successMessage: (projection: GameDto, fileName: string) => string;
      failurePrefix: string;
    }
  ): Promise<boolean> {
    const outcome = await resolveDocumentReplacement({
      departureId,
      action,
      selectedPath: currentGameRef.current?.selected_path ?? { indices: [] },
      defaultFileName: saveFileName
    });
    if (outcome.analysis_stopped) {
      clearLocalAnalysisSession();
    }
    if (!outcome.committed) {
      if (outcome.current) {
        adoptCurrentGame(outcome.current);
        setDirty(outcome.current.dirty);
        setCurrentFilePath(outcome.current.native_path ?? null);
      }
      setMessage(action === "cancel" ? "已取消替换，当前棋谱和分析保持不变。" : outcome.message);
      return false;
    }
    if (!outcome.current) {
      setMessage(`${options.failurePrefix}: replacement committed without a current game`);
      return false;
    }
    await adoptCommittedReplacement(outcome.current, sgfInput, nativePath, options);
    return true;
  }


  async function restoreRecoveredDocument(successMessage: string): Promise<boolean> {
    const restored = await restoreCurrentGameRecovery();
    const serialized = await serializeCurrentGame();
    await adoptCommittedReplacement(restored, serialized, restored.native_path ?? null, {
      fallbackName: restored.native_path ? null : "recovered.sgf",
      successMessage: () => successMessage
    });
    return true;
  }

  async function handleRestoreRecoveredGame() {
    try {
      await restoreRecoveredDocument("已恢复上次未正常退出的棋谱。");
      setRecoveryPrompt(null);
    } catch (error: unknown) {
      setMessage(`恢复失败: ${errorMessage(error)}`);
    }
  }

  async function handleDiscardRecoveredGame() {
    try {
      await discardCurrentGameRecovery();
      setRecoveryPrompt(null);
      await applyReplacement(demoSgf, null, {
        confirmMessage: "放弃未保存的棋谱并载入示例？",
        fallbackName: "sample.sgf",
        successMessage: (projection) => `Sample SGF restored: ${projection.summary.move_count} moves.`,
        failurePrefix: "Sample load failed"
      });
    } catch (error: unknown) {
      setMessage(`放弃恢复失败: ${errorMessage(error)}`);
    }
  }

  async function handleRetryRecoveryWrite() {
    try {
      const protection = await retryCurrentGameRecovery();
      setRecoveryProtection(protection);
      if (protection.status === "protected") {
        setMessage("当前棋谱恢复快照已写入。");
      } else {
        setMessage(protection.message);
      }
    } catch (error: unknown) {
      setMessage(`重试恢复写入失败: ${errorMessage(error)}`);
    }
  }

  async function applyReplacement(
    sgfInput: string,
    nativePath: string | null,
    options: {
      confirmMessage: string;
      fallbackName?: string | null;
      successMessage: (projection: GameDto, fileName: string) => string;
      failurePrefix: string;
    }
  ): Promise<boolean> {
    if (!nativeRuntime) {
      try {
        const [parsed, replayed] = await Promise.all([parseSgfSummary(sgfInput), replaySgfPositions(sgfInput)]);
        if (documentDirty && !window.confirm(options.confirmMessage)) return false;
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
        setKeyboardPlacement(false);
        await abandonAnalysisSessions();
        const previewMessage = options.successMessage(parsed, options.fallbackName ?? "SGF");
        setMessage(`${nativeCurrentGameUnavailable} ${previewMessage}`);
        return true;
      } catch (error) {
        setMessage(`${options.failurePrefix}: ${errorMessage(error)}`);
        return false;
      }
    }

    try {
      const admission = await prepareDocumentReplacement(sgfInput, nativePath);
      const action: DocumentDepartureActionDto = admission.status === "needs_decision"
        ? await requestDepartureDecision(options.confirmMessage)
        : "discard";
      return await finishNativeReplacement(admission.departure_id, action, sgfInput, nativePath, options);
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
    try {
      const document = await openSgfDocument();
      if (!document) return;
      await applyReplacement(document.sgfText, document.path, {
        confirmMessage: "放弃未保存的棋谱并打开这个文件？",
        successMessage: (projection, fileName) => `Opened ${fileName}: ${projection.summary.move_count} moves.`,
        failurePrefix: "Open failed"
      });
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
    if (nativeRuntime) {
      setMessage(nativeSyntheticAnalysisUnavailable);
      return;
    }
    const captured = beginReviewRequest();
    try {
      const [parsed, result, replayed] = await Promise.all([parseSgfSummary(sgfText), fakeAnalyze(sgfText), replaySgfPositions(sgfText)]);
      const classified = await classifyProblems(result);
      if (!publishReviewPresentation(captured, result, classified)) return;
      setGame(parsed);
      setPositions(replayed);
      setCurrentMove(replayed.at(-1)?.move_number ?? parsed.moves.length);
      if (!shouldPublishReviewPresentation(activeScopeFromRefs(), captured)) return;
      setMessage(`${nativeCurrentGameUnavailable} 已生成 ${result.length} 个预览复盘局面。`);
    } catch (error) {
      setMessage(errorMessage(error));
    }
  }

  function clearSelectedNodeRunning(jobId: string) {
    if (selectedNodeJobRef.current?.job_id !== jobId) return;
    selectedNodeJobRef.current = null;
    setSelectedNodeRunning(false);
  }

  function resetWholeGameSession() {
    wholeGameResultsRef.current = new Map();
    wholeGameJobRef.current = null;
    setWholeGameRunning(false);
    setWholeGameProgress(null);
  }

  async function abandonAnalysisSessions() {
    const selected = selectedNodeJobRef.current;
    const wholeGame = wholeGameJobRef.current;
    selectedNodeJobRef.current = null;
    setSelectedNodeRunning(false);
    resetWholeGameSession();
    const selectedCancellation = selected
      ? cancelSelectedNodeAnalysis({ runId: selected.run_id, jobId: selected.job_id })
      : Promise.resolve();
    const wholeGameCancellation = wholeGame
      ? cancelKataGoAnalysis(wholeGame.run_id, wholeGame.job_id)
      : Promise.resolve();
    await Promise.allSettled([selectedCancellation, wholeGameCancellation]);
  }

  function clearWholeGameRunning() {
    wholeGameJobRef.current = null;
    setWholeGameRunning(false);
  }

  function presentWholeGameFrame(path: NodePath, frame: AnalysisFrameDto) {
    const game = currentGameRef.current;
    if (!game || !samePath(game.selected_path, path)) return;
    const token = wholeGameJobRef.current?.job_id ?? activeRequestTokenRef.current;
    activeRequestTokenRef.current = token;
    setActiveRequestToken(token);
    documentGenerationRef.current = Math.max(documentGenerationRef.current, game.generation);
    publishReviewPresentation(
      {
        generation: game.generation,
        selectedPath: [...path.indices],
        requestToken: token
      },
      [frame],
      []
    );
  }

  function rememberWholeGameFrame(path: NodePath, frame: AnalysisFrameDto) {
    const next = new Map(wholeGameResultsRef.current);
    next.set(pathKey(path), frame);
    wholeGameResultsRef.current = next;
    presentWholeGameFrame(path, frame);
  }

  function forgetSessionFrame(path: NodePath) {
    const next = new Map(wholeGameResultsRef.current);
    next.delete(pathKey(path));
    wholeGameResultsRef.current = next;
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
      if (!admitsAnalysisPublication(job, publication) || !job.frame || !isCurrentDocumentGeneration(job.generation)) {
        clearSelectedNodeRunning(job.job_id);
        return;
      }
      const frame = job.frame;
      rememberWholeGameFrame(job.node_path, frame);
      if (admitsAnalysisAttachment(job)) void refreshCurrentGameAfterAttach();
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
      forgetSessionFrame(game.selected_path);
      selectedNodeJobRef.current = started;
      setSelectedNodeRunning(true);
      setMessage(`Running KataGo analysis (${started.job_id})...`);
    } catch (error) {
      setMessage(`KataGo analysis failed: ${errorMessage(error)}`);
    }
  }

  handleWholeGameJobRef.current = (job: AnalysisJobEventDto) => {
    const pending = wholeGameJobRef.current;
    if (!matchesWholeGameJobIdentity(pending, job)) return;
    if (job.outcome === "started" || job.outcome === "progress") {
      setWholeGameProgress({
        completed: job.completed ?? 0,
        expected: job.expected ?? 0,
        remaining: job.remaining ?? Math.max(0, (job.expected ?? 0) - (job.completed ?? 0))
      });
      if (job.outcome === "progress" && admitsWholeGameNodeResult(job) && job.frame) {
        rememberWholeGameFrame(job.node_path, job.frame);
        if (admitsAnalysisAttachment(job)) void refreshCurrentGameAfterAttach();
      }
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
      clearWholeGameRunning();
    }
  };

  async function handleAnalyzeKataGoGame(runId: string, maxVisits: number) {
    const game = currentGameRef.current;
    if (!nativeRuntime || !game) {
      setMessage(nativeRuntime ? "整局分析需要当前游戏。" : nativeCurrentGameUnavailable);
      return;
    }
    const visits = resolveAnalysisMaxVisits(maxVisits, preferences);
    try {
      const started = await startKataGoGameAnalysis({
        runId,
        generation: game.generation,
        maxVisits: visits
      });
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
      failurePrefix: "New game failed"
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
      documentGenerationRef.current = result.generation;
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
      await abandonAnalysisSessions();
      const artifacts = await artifactsFromCurrentGame();
      setGame(artifacts.projection);
      clearReviewData();
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
      currentGameRef.current = {
        ...(currentGameRef.current ?? result),
        selected_path: result.selected_path,
        snapshot: result.snapshot,
        generation: Math.max(currentGameRef.current?.generation ?? 0, result.generation)
      };
      const stored = wholeGameResultsRef.current.get(pathKey(result.selected_path));
      if (stored) presentWholeGameFrame(result.selected_path, stored);
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
        await abandonAnalysisSessions();
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
      await abandonAnalysisSessions();
      const artifacts = await artifactsFromCurrentGame();
      if (!isCurrentDocumentGeneration(result.generation)) return;
      setGame(artifacts.projection);
      setMessage("已删除选中变化，并回到其父节点。");
    } catch (error) {
      setMessage(`删除变化失败: ${errorMessage(error)}`);
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
      scoreLeadAvailable={chartModel.scoreAvailable}
      showCoordinates={showCoordinates}
      showMoveNumbers={showMoveNumbers}
      onShowCoordinates={setShowCoordinates}
      onShowMoveNumbers={setShowMoveNumbers}
      showBlackCandidates={showBlackCandidates}
      showWhiteCandidates={showWhiteCandidates}
      onShowBlackCandidates={setShowBlackCandidates}
      onShowWhiteCandidates={setShowWhiteCandidates}
      referenceRailCollapsed={referenceRailCollapsed}
      onReferenceRailCollapsed={setReferenceRailCollapsed}
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
      onExit={() => void handleApplicationExit()}
      onClearBoard={() => void handleNewGame()}
      onPass={() => void playAt("pass")}
      onRemoveVariation={() => void handleRemoveVariation()}
      canRemoveVariation={canRemoveVariation}
      onFirstMove={() => handleMoveSelect(0)}
      onAutoPlay={() => setAutoPlaying((value) => !value)}
      onOverlayMode={setOverlayMode}
      message={message}
      toPlay={currentPosition.to_play}
    />
    <section className="spread">
      <aside className="rail">
        <div className="rail-block">
          <h2>
            <span>胜率走势 ({chartTitleSide})</span>
            <span style={{ color: "#60a5fa", fontFamily: "var(--mono)" }}>
              {`${((chartWinrate ?? 0.5) * 100).toFixed(1)}%`}
            </span>
          </h2>
          <WinrateChart model={chartModel} />
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
          hideCandidates={hideCandidates}
          pvPrefixLength={replayPrefix}
          replayCandidateIndex={activeCandidateIndex}
          onPointClick={(point) => void playAt({ point })}
          keyboardPlacement={keyboardPlacement}
          nextMoveMode={preferences.nextMoveReviewMarker}
          nextMoveMarkers={nextMoveMarkers}
        />
        {boardIntentFeedback ? (
          <p className="board-intent-status" role="status" aria-live="polite">{boardIntentFeedback}</p>
        ) : null}
      </div>
      <aside className="sheet-col" hidden={referenceRailCollapsed}>
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
          contentMode={preferences.subBoardContentMode}
          pvPrefixLength={replayPrefix}
        />
      </aside>
    </section>
    {recoveryProtection.status === "unprotected" ? (
      <div className="recovery-unprotected">
        <span role="status">{recoveryProtection.message}</span>
        <button type="button" aria-label="Retry recovery write" onClick={() => void handleRetryRecoveryWrite()}>
          重试
        </button>
      </div>
    ) : null}
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
      onBrowserDemoAnalyze={nativeRuntime ? undefined : () => void handleFakeAnalyze()}
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
      keyboardPlacement={keyboardPlacement}
      onKeyboardPlacement={setKeyboardPlacement}
      jumpRef={jumpRef}
      message={message}
      toPlay={currentPosition.to_play}
    />
    <section className="sheet-row" hidden={sheet === "none"}>
      {sheet === "sgf" ? <div className="sgf-tools">
        <div className="document-row">
          <strong title={documentPath ?? documentName}>{documentName}{documentDirty ? " *" : ""}</strong>
          <span>{documentDirty ? "未保存" : "已保存"}</span>
          {recoveryProtection.status === "unprotected" ? (
            <>
              <span role="status">{recoveryProtection.message}</span>
              <button type="button" aria-label="Retry recovery write" onClick={() => void handleRetryRecoveryWrite()}>
                重试
              </button>
            </>
          ) : null}
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
        scoreLeadAvailable={chartModel.scoreAvailable}
        onChange={(nextPreferences) => void handlePreferencesChange(nextPreferences)}
      /> : null}
    </section>
    {departurePrompt ? (
      <DocumentDepartureDialog
        message={departurePrompt.message}
        onChoose={departurePrompt.choose}
      />
    ) : null}
    {teardownPrompt ? (
      <ApplicationTeardownDialog
        message={teardownPrompt.message}
        onRetry={() => teardownPrompt.choose("retry")}
        onExitAnyway={() => teardownPrompt.choose("exit_anyway")}
      />
    ) : null}
    {recoveryPrompt ? (
      <CurrentGameRecoveryDialog
        message="检测到未正常退出的棋谱。恢复上次棋谱，还是放弃该恢复候选？"
        onRestore={() => void handleRestoreRecoveredGame()}
        onDiscard={() => void handleDiscardRecoveredGame()}
      />
    ) : null}
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

function mergeAnalysisFrame(frames: AnalysisFrameDto[], frame: AnalysisFrameDto): AnalysisFrameDto[] {
  return [...frames.filter((item) => item.turn !== frame.turn), frame].sort((a, b) => a.turn - b.turn);
}

function errorMessage(error: unknown): string {
  if (error instanceof Error) return error.message;
  if (error && typeof error === "object" && "message" in error) {
    const message = Reflect.get(error, "message");
    if (typeof message === "string" && message.trim()) return message;
  }
  return String(error);
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
