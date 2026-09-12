import { useEffect, useMemo, useRef, useState } from "react";
import { BoardCanvas } from "./components/BoardCanvas";
import { WinrateChart } from "./components/WinrateChart";
import { AnalysisPanel } from "./components/AnalysisPanel";
import { EngineSetupPanel } from "./components/EngineSetupPanel";
import { AppChrome, BottomBar, type ContinuousAnalysisAction, type OverlayMode, type SheetId } from "./components/AppChrome";
import { PreferencesPanel } from "./components/PreferencesPanel";
import { ShortcutReference } from "./components/ShortcutReference";
import { DocumentDepartureDialog } from "./components/DocumentDepartureDialog";
import { ApplicationTeardownDialog } from "./components/ApplicationTeardownDialog";
import { CurrentGameRecoveryDialog } from "./components/CurrentGameRecoveryDialog";
import { AnalysisTaskPanel, type AnalysisScopeDraft } from "./components/AnalysisTaskPanel";
import { ProviderPanel } from "./components/ProviderPanel";
import {
  analysisTaskSnapshot,
  cancelKataGoAnalysis,
  continueAnalysisTask,
  cancelSelectedNodeAnalysis,
  classifyProblems,
  fakeAnalyze,
  foregroundEngineContinuousAction,
  getHealth,
  isTauriRuntime,
  nativeCurrentGameUnavailable,
  nativeSyntheticAnalysisUnavailable,
  openSgfDocument,
  parseSgfSummary,
  playCurrentGame,
  prepareApplicationExit,
  prepareDocumentReplacement,
  pauseAnalysisTask,
  previewAnalysisScope,
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
  startAnalysisTask,
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
import { continuousBudgetError, defaultAppPreferences, normalizeAppPreferences, taskConditionsError, taskStageConditionsError, type AppPreferences } from "./domain/preferences";
import { buildNextMoveReviewMarkers, cycleNextMoveReviewMarker } from "./domain/nextMoveReviewMarker";
import { admitsChartSeriesChange, buildWinrateChartModel, displayedWinrate } from "./domain/winrateChart";
import { providerDocumentName, providerLabel, providerSourceLabel, type ProviderImportResult } from "./domain/providers";
import { admitsAnalysisAttachment, admitsAnalysisPublication, admitsWholeGameNodeResult, matchesWholeGameJobIdentity } from "./domain/analysisJob";
import { createShortcutRegistry } from "./domain/shortcuts";
import {
  CONTINUOUS_ANALYSIS_RESUME_LABEL,
  CONTINUOUS_ANALYSIS_START_LABEL,
  CONTINUOUS_ANALYSIS_STOP_LABEL
} from "./domain/analysisActions";
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
import type { AnalysisFrameDto, AnalysisJobEventDto, AnalysisJobStartedDto, AnalysisScopeDto, AnalysisScopePreviewDto, AnalysisStageConditionsDto, AnalysisTaskDto, AnalysisTaskStrategyDto, AppHealthDto, ApplicationExitActionDto, ApplicationExitOutcomeDto, ContinuousAnalysisPhaseDto, CurrentGameResultDto, DocumentDepartureActionDto, EngineProfileDto, EngineProfileRecordDto, EngineFailureDto, ForegroundEngineSnapshotDto, GameDto, MoveVertex, NodePath, PositionDto, ProblemMarkerDto, RecoveryProtectionDto, RecoveryStartupDto, SgfTreeNodeDto } from "./domain/types";

const demoSgf = "(;GM[1]FF[4]SZ[19]KM[7.5]PB[李昌镐]PW[芮乃伟]RE[B+R];B[pd];W[dd];B[pp];W[dp];B[jq];W[qj];B[nc];W[fc];B[qf];W[cn];B[cp];W[do];B[co];W[dn];B[fq];W[eq];B[fp];W[gp];B[gq];W[hp])";
const emptySgf = "(;GM[1]FF[4]SZ[19]KM[7.5]PB[黑]PW[白])";
const demoGame = createDemoGame();
const emptyChartRoot: SgfTreeNodeDto = { properties: [], children: [] };
type WholeGameProgress = { completed: number; expected: number; remaining: number };
type PendingPreferencesSave = {
  version: number;
  preferences: AppPreferences;
  patch: Partial<AppPreferences>;
  onSaved?: (preferences: AppPreferences) => void;
  onFailed?: (error: unknown) => void;
};
type CandidatePreview = { index: number; scope: ReviewPresentationScope };
const defaultAnalysisScopeDraft: AnalysisScopeDraft = {
  strategy: "all_positions_two_stage",
  mode: "first_child_mainline",
  intervalEnabled: false,
  intervalStart: "0",
  intervalEnd: "0",
  toPlay: "both",
  overviewTimeEnabled: defaultAppPreferences.taskOverviewConditions.time_seconds.enabled,
  overviewTimeSeconds: String(defaultAppPreferences.taskOverviewConditions.time_seconds.value),
  overviewTotalVisitsEnabled: defaultAppPreferences.taskOverviewConditions.total_visits.enabled,
  overviewTotalVisits: String(defaultAppPreferences.taskOverviewConditions.total_visits.value),
  overviewLeadingCandidateVisitsEnabled: defaultAppPreferences.taskOverviewConditions.leading_candidate_visits.enabled,
  overviewLeadingCandidateVisits: String(defaultAppPreferences.taskOverviewConditions.leading_candidate_visits.value),
  singleTimeEnabled: defaultAppPreferences.taskSingleStageConditions.time_seconds.enabled,
  singleTimeSeconds: String(defaultAppPreferences.taskSingleStageConditions.time_seconds.value),
  singleTotalVisitsEnabled: defaultAppPreferences.taskSingleStageConditions.total_visits.enabled,
  singleTotalVisits: String(defaultAppPreferences.taskSingleStageConditions.total_visits.value),
  singleLeadingCandidateVisitsEnabled: defaultAppPreferences.taskSingleStageConditions.leading_candidate_visits.enabled,
  singleLeadingCandidateVisits: String(defaultAppPreferences.taskSingleStageConditions.leading_candidate_visits.value),
  timeEnabled: defaultAppPreferences.taskDeepConditions.time_seconds.enabled,
  timeSeconds: String(defaultAppPreferences.taskDeepConditions.time_seconds.value),
  totalVisitsEnabled: defaultAppPreferences.taskDeepConditions.total_visits.enabled,
  totalVisits: String(defaultAppPreferences.taskDeepConditions.total_visits.value),
  leadingCandidateVisitsEnabled: defaultAppPreferences.taskDeepConditions.leading_candidate_visits.enabled,
  leadingCandidateVisits: String(defaultAppPreferences.taskDeepConditions.leading_candidate_visits.value)
};


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
  const [analysisScopeDraft, setAnalysisScopeDraft] = useState<AnalysisScopeDraft>(defaultAnalysisScopeDraft);
  const [analysisScopePreview, setAnalysisScopePreview] = useState<AnalysisScopePreviewDto | null>(null);
  const [analysisTask, setAnalysisTask] = useState<AnalysisTaskDto | null>(null);
  const [analysisTaskError, setAnalysisTaskError] = useState<string | null>(null);
  const [analysisTaskRequestPending, setAnalysisTaskRequestPending] = useState(false);
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
  const [departurePending, setDeparturePending] = useState(false);
  const shortcutRegistry = useMemo(() => createShortcutRegistry(), []);
  const jumpRef = useRef<HTMLInputElement | null>(null);
  const requestSerialRef = useRef(0);
  const [activeRequestToken, setActiveRequestToken] = useState("idle:0");
  const activeRequestTokenRef = useRef("idle:0");
  const [publishedScope, setPublishedScope] = useState<ReviewPresentationScope | null>(null);
  const preferencesLoadSettledRef = useRef(false);
  const [preferencesLoaded, setPreferencesLoaded] = useState(false);
  const [continuousActionPending, setContinuousActionPending] = useState(false);
  const committedPreferencesRef = useRef<AppPreferences>(defaultAppPreferences);
  const preferencesSaveInFlightRef = useRef(false);
  const preferencesSaveVersionRef = useRef(0);
  const pendingPreferencesSaveRef = useRef<PendingPreferencesSave | null>(null);
  const continuousActionInFlightRef = useRef(false);
  const currentGameRef = useRef<CurrentGameResultDto | null>(null);
  const selectedNodeJobRef = useRef<AnalysisJobStartedDto | null>(null);
  const lastFiniteTerminalJobIdRef = useRef<string | null>(null);
  const latestSnapshotRevisionRef = useRef(0);
  const departurePendingRef = useRef(false);
  const wholeGameJobRef = useRef<AnalysisJobStartedDto | null>(null);
  const wholeGameResultsRef = useRef<Map<string, AnalysisFrameDto>>(new Map());
  const handleSelectedNodeJobRef = useRef<(job: AnalysisJobEventDto) => void>(() => undefined);
  const handleWholeGameJobRef = useRef<(job: AnalysisJobEventDto) => void>(() => undefined);
  const analysisTaskRef = useRef<AnalysisTaskDto | null>(null);
  const analysisTaskSnapshotRequestRef = useRef(0);
  const analysisTaskActionInFlightRef = useRef(false);
  const analysisTaskPauseFenceRef = useRef<{ taskId: string; jobId: string; continued: boolean } | null>(null);
  const analysisConditionsDraftEditedRef = useRef(false);

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
  const continuousPhase: ContinuousAnalysisPhaseDto = nativeRuntime
    ? engineSnapshot.continuous.phase
    : preferences.continuousAnalysisEnabled ? "waiting" : "off";
  const continuousEnabled = nativeRuntime
    ? engineSnapshot.continuous.enabled
    : preferences.continuousAnalysisEnabled;
  const continuousAnalysisAction: ContinuousAnalysisAction = (() => {
    if (!preferencesLoaded || continuousEnabled === null || continuousPhase === "loading") {
      return { label: CONTINUOUS_ANALYSIS_START_LABEL, disabled: true, title: "正在载入连续分析设置。", status: "连续分析：正在载入设置" };
    }
    if (preferencesSaveInFlightRef.current || pendingPreferencesSaveRef.current) {
      return { label: continuousEnabled ? CONTINUOUS_ANALYSIS_STOP_LABEL : CONTINUOUS_ANALYSIS_START_LABEL, disabled: true, title: "正在保存偏好设置。", status: continuousPhaseStatus(continuousPhase) };
    }
    if (departurePending || departurePrompt || continuousPhase === "departing" || continuousPhase === "stopping") {
      return { label: CONTINUOUS_ANALYSIS_STOP_LABEL, disabled: true, title: "正在等待离开或分析取消完成。", status: continuousPhaseStatus(continuousPhase) };
    }
    if (engineSnapshot.lifecycle.state === "error") {
      return { label: CONTINUOUS_ANALYSIS_RESUME_LABEL, disabled: true, title: "前台引擎运行失败；请先显式重启引擎。", status: "连续分析：引擎失败" };
    }
    if (!continuousEnabled) {
      return { label: CONTINUOUS_ANALYSIS_START_LABEL, disabled: continuousActionPending, status: continuousPhaseStatus(continuousPhase) };
    }
    if (continuousPhase === "time_limited" || continuousPhase === "visits_limited" || continuousPhase === "paused" || continuousPhase === "error" || continuousPhase === "safety_hold") {
      const resumable = nativeRuntime && engineReady && Boolean(currentGame);
      return {
        label: CONTINUOUS_ANALYSIS_RESUME_LABEL,
        disabled: continuousActionPending || !resumable,
        title: resumable ? undefined : "需要可用的前台引擎和当前棋谱。",
        status: continuousPhaseStatus(continuousPhase)
      };
    }
    return {
      label: CONTINUOUS_ANALYSIS_STOP_LABEL,
      disabled: continuousActionPending,
      status: continuousPhaseStatus(continuousPhase)
    };
  })();
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
      void handleContinuousAnalysisAction();
    });
    shortcutRegistry.bind("analysis.quick", () => {
      void handleQuickAnalysisTask();
    });
    shortcutRegistry.bind("analysis.all-positions", () => {
      void handleAllPositionsAnalysisTask();
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
    visibleCurrentFrame,
    continuousPhase,
    preferencesLoaded,
    continuousActionPending,
    departurePending,
    engineReady,
    engineSnapshot.lifecycle.state,
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
        if (cancelled || snapshot.revision < latestSnapshotRevisionRef.current) return;
        latestSnapshotRevisionRef.current = snapshot.revision;
        engineSnapshotRef.current = snapshot;
        setEngineSnapshot(snapshot);
        if (snapshot.lifecycle.state === "switching") {
          lastSwitchIdRef.current = snapshot.lifecycle.switch_id;
        } else if (snapshot.lifecycle.state === "no_engine") {
          lastSwitchIdRef.current = null;
        }
        const nextRunId = runFromSnapshot(snapshot)?.run_id ?? null;
        if (analysisRunIdRef.current !== nextRunId) {
          if (analysisRunIdRef.current !== null) {
            clearReviewData();
            resetWholeGameSession();
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
        selectedNodeJobRef.current = snapshot.selected_node_job ?? null;
        setSelectedNodeRunning(snapshot.selected_node_job?.state === "queued" || snapshot.selected_node_job?.state === "searching" || snapshot.selected_node_job?.state === "stopping");
        if (snapshot.selected_node_job?.mode === "continuous") {
          adoptAutomaticJobToken(snapshot.selected_node_job);
        }
        const snapshotWholeGameJob = snapshot.whole_game_job ?? null;
        const taskReserved = isAnalysisTaskReserved(analysisTaskRef.current);
        const pauseFence = analysisTaskPauseFenceRef.current;
        const fencedJob = snapshotWholeGameJob != null
          && pauseFence != null && pauseFence.taskId === analysisTaskRef.current?.task_id
          && pauseFence.jobId === snapshotWholeGameJob.job_id;
        if (!fencedJob && (snapshotWholeGameJob || !taskReserved)) {
          wholeGameJobRef.current = snapshotWholeGameJob;
        }
        const snapshotWholeGameActive = snapshotWholeGameJob?.state === "queued"
          || snapshotWholeGameJob?.state === "searching"
          || snapshotWholeGameJob?.state === "stopping";
        setWholeGameRunning(taskReserved || (!fencedJob && snapshotWholeGameActive));
        if (!snapshotWholeGameJob && !taskReserved) setWholeGameProgress(null);
        void refreshAnalysisTaskSnapshot();
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
        void refreshAnalysisTaskSnapshot();
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
    if (continuousActionInFlightRef.current) return;
    if (!preferencesLoadSettledRef.current) return;
    const budgetError = continuousBudgetError(nextPreferences);
    if (budgetError) {
      setPreferencesStatus(budgetError);
      return;
    }
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
    setPreferencesLoaded(true);
    committedPreferencesRef.current = loaded;
    setPreferences(loaded);
    if (!analysisConditionsDraftEditedRef.current) {
      setAnalysisScopeDraft((current) => ({
        ...current,
        overviewTimeEnabled: loaded.taskOverviewConditions.time_seconds.enabled,
        overviewTimeSeconds: String(loaded.taskOverviewConditions.time_seconds.value),
        overviewTotalVisitsEnabled: loaded.taskOverviewConditions.total_visits.enabled,
        overviewTotalVisits: String(loaded.taskOverviewConditions.total_visits.value),
        overviewLeadingCandidateVisitsEnabled: loaded.taskOverviewConditions.leading_candidate_visits.enabled,
        overviewLeadingCandidateVisits: String(loaded.taskOverviewConditions.leading_candidate_visits.value),
        singleTimeEnabled: loaded.taskSingleStageConditions.time_seconds.enabled,
        singleTimeSeconds: String(loaded.taskSingleStageConditions.time_seconds.value),
        singleTotalVisitsEnabled: loaded.taskSingleStageConditions.total_visits.enabled,
        singleTotalVisits: String(loaded.taskSingleStageConditions.total_visits.value),
        singleLeadingCandidateVisitsEnabled: loaded.taskSingleStageConditions.leading_candidate_visits.enabled,
        singleLeadingCandidateVisits: String(loaded.taskSingleStageConditions.leading_candidate_visits.value),
        timeEnabled: loaded.taskDeepConditions.time_seconds.enabled,
        timeSeconds: String(loaded.taskDeepConditions.time_seconds.value),
        totalVisitsEnabled: loaded.taskDeepConditions.total_visits.enabled,
        totalVisits: String(loaded.taskDeepConditions.total_visits.value),
        leadingCandidateVisitsEnabled: loaded.taskDeepConditions.leading_candidate_visits.enabled,
        leadingCandidateVisits: String(loaded.taskDeepConditions.leading_candidate_visits.value)
      }));
    }
    setPreferencesStatus(status);
  }

  function queuePreferencesSave(base: AppPreferences, ...patches: Array<Partial<AppPreferences>>) {
    const patch = Object.assign({}, pendingPreferencesSaveRef.current?.patch, ...patches);
    pendingPreferencesSaveRef.current = {
      version: preferencesSaveVersionRef.current + 1,
      preferences: applyPreferencePatches(base, patches),
      patch
    };
    preferencesSaveVersionRef.current = pendingPreferencesSaveRef.current.version;
    setPreferencesStatus("Saving preferences...");
    void runPreferencesSaveLoop();
    return pendingPreferencesSaveRef.current;
  }

  async function runPreferencesSaveLoop() {
    if (preferencesSaveInFlightRef.current || continuousActionInFlightRef.current) return;
    preferencesSaveInFlightRef.current = true;
    try {
      while (pendingPreferencesSaveRef.current && !continuousActionInFlightRef.current) {
        const pending = pendingPreferencesSaveRef.current;
        try {
          const saved = await saveAppPreferences(pending.preferences);
          committedPreferencesRef.current = saved;
          setPreferences(saved);
          pending.onSaved?.(saved);
          if (pendingPreferencesSaveRef.current?.version === pending.version) {
            pendingPreferencesSaveRef.current = null;
            setPreferencesStatus("Preferences saved.");
          } else {
            setPreferencesStatus("Saving preferences...");
          }
        } catch (error) {
          pending.onFailed?.(error);
          const queued = pendingPreferencesSaveRef.current;
          if (queued?.version === pending.version) {
            pendingPreferencesSaveRef.current = null;
            setPreferencesStatus(`Save failed: ${errorMessage(error)}`);
          } else if (queued) {
            for (const key of Object.keys(pending.patch) as Array<keyof AppPreferences>) {
              if (JSON.stringify(queued.patch[key]) === JSON.stringify(pending.patch[key])) {
                delete queued.patch[key];
              }
            }
            if (Object.keys(queued.patch).length === 0) {
              pendingPreferencesSaveRef.current = null;
              setPreferencesStatus(`Save failed: ${errorMessage(error)}`);
            } else {
              queued.preferences = applyPreferencePatches(committedPreferencesRef.current, [queued.patch]);
              setPreferencesStatus("Saving preferences...");
            }
          }
        }
      }
    } finally {
      preferencesSaveInFlightRef.current = false;
    }
  }



  function isCurrentGameSnapshot(result: CurrentGameResultDto): boolean {
    const current = currentGameRef.current;
    return current !== null && result.generation === current.generation && result.snapshot_seq >= current.snapshot_seq;
  }

  function adoptCurrentGame(result: CurrentGameResultDto) {
    documentGenerationRef.current = result.generation;
    currentGameRef.current = result;
    setCurrentGame(result);
    if (selectedNodeJobRef.current?.mode === "continuous") {
      adoptAutomaticJobToken(selectedNodeJobRef.current);
    }
  }

  async function refreshCurrentGameAfterAttach() {
    const game = currentGameRef.current;
    if (!nativeRuntime || !game || departurePendingRef.current || navigatingRef.current) return;
    const requestToken = activeRequestTokenRef.current;
    try {
      const refreshed = await selectCurrentGameNode(game.selected_path);
      if (!refreshed || departurePendingRef.current || navigatingRef.current
        || requestToken !== activeRequestTokenRef.current
        || !samePath(game.selected_path, currentGameRef.current?.selected_path ?? { indices: [] })
        || !isCurrentGameSnapshot(refreshed)) return;
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

  function adoptAutomaticJobToken(job: AnalysisJobStartedDto): boolean {
    const run = runFromSnapshot(engineSnapshotRef.current);
    const game = currentGameRef.current;
    if (!run || !game || run.run_id !== job.run_id || game.generation !== job.generation || !samePath(game.selected_path, job.node_path)) {
      return false;
    }
    if (activeRequestTokenRef.current !== job.job_id) {
      beginReviewRequest(job.job_id);
      forgetSessionFrame(job.node_path);
    }
    return true;
  }

  async function artifactsFromCurrentGame(): Promise<{ serialized: string; projection: GameDto }> {
    const [serialized, projection] = await Promise.all([serializeCurrentGame(), projectCurrentGameMainline()]);
    return { serialized, projection };
  }
  function presentCurrentGameAnalysis(result: CurrentGameResultDto) {
    const frame = result.snapshot.primary_analysis;
    if (!frame || frame.visits === 0 || frame.candidates.length === 0) return;
    const captured: ReviewPresentationScope = {
      generation: result.generation,
      selectedPath: [...result.selected_path.indices],
      requestToken: activeRequestTokenRef.current
    };
    void classifyProblems([frame])
      .then((classified) => publishReviewPresentation(captured, [frame], classified))
      .catch((error) => setMessage(`分析结果读取失败: ${errorMessage(error)}`));
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
      if (action !== "cancel") {
        departurePendingRef.current = true;
        setDeparturePending(true);
      }
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
      departurePendingRef.current = false;
      setDeparturePending(false);
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
    analysisTaskRef.current = null;
    analysisTaskPauseFenceRef.current = null;
    ++analysisTaskSnapshotRequestRef.current;
    setAnalysisTask(null);
    setAnalysisScopePreview(null);
    setAnalysisTaskError(null);
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
    beginReviewRequest();
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
    presentCurrentGameAnalysis(result);
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
      if (action !== "cancel") {
        departurePendingRef.current = true;
        setDeparturePending(true);
      }
      return await finishNativeReplacement(admission.departure_id, action, sgfInput, nativePath, options);
    } catch (error) {
      setMessage(`${options.failurePrefix}: ${errorMessage(error)}`);
      return false;
    } finally {
      departurePendingRef.current = false;
      setDeparturePending(false);
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
      if (!isCurrentGameSnapshot(saved)) return;
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
    selectedNodeJobRef.current = null;
    setSelectedNodeRunning(false);
    sealReviewPresentation();
    resetWholeGameSession();
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

  function publishAuthoritativeContinuousFrame(job: AnalysisJobEventDto) {
    const pending = selectedNodeJobRef.current;
    if (!pending || pending.mode !== "continuous" || !matchesPendingAnalysisJob(pending, job)) return;
    if (!(["queued", "searching", "time_limited", "visits_limited"] as ContinuousAnalysisPhaseDto[]).includes(engineSnapshotRef.current.continuous.phase)) return;
    if (departurePendingRef.current || navigatingRef.current) return;
    const refreshed = job.current_game;
    if (!refreshed || !isCurrentGameSnapshot(refreshed) || !samePath(refreshed.selected_path, job.node_path)) return;
    const frame = refreshed.snapshot.primary_analysis;
    if (!frame || frame.visits === 0 || frame.candidates.length === 0) return;
    adoptCurrentGame(refreshed);
    setDirty(refreshed.dirty);
    const captured: ReviewPresentationScope = {
      generation: job.generation,
      selectedPath: [...job.node_path.indices],
      requestToken: job.job_id
    };
    // Problem markers compare successive positions, not progress within one position.
    if (!publishReviewPresentation(captured, [frame], [])) return;
    setMessage(`连续分析：${frame.visits} visits。`);
  }


  handleSelectedNodeJobRef.current = (job: AnalysisJobEventDto) => {
    if (job.outcome === "started") {
      const started: AnalysisJobStartedDto = {
        run_id: job.run_id,
        job_id: job.job_id,
        lane: job.lane,
        mode: job.mode,
        state: "queued",
        generation: job.generation,
        node_path: job.node_path
      };
      if (adoptAutomaticJobToken(started)) {
        selectedNodeJobRef.current = started;
        setSelectedNodeRunning(true);
      }
      return;
    }
    const pending = selectedNodeJobRef.current;
    if (!matchesPendingAnalysisJob(pending, job)) return;

    if (pending.mode === "continuous") {
      const phase = engineSnapshotRef.current.continuous.phase;
      if (job.outcome === "progress" && (phase === "queued" || phase === "searching")) {
        if (admitsAnalysisAttachment(job)) void publishAuthoritativeContinuousFrame(job);
      } else if ((job.outcome === "time_limited" || job.outcome === "visits_limited") && (phase === "queued" || phase === "searching" || phase === job.outcome)) {
        if (admitsAnalysisAttachment(job)) void publishAuthoritativeContinuousFrame(job);
        setMessage(`${continuousPhaseStatus(job.outcome)}；可显式继续。`);
      } else if ((job.outcome === "failed" || job.outcome === "timeout") && phase === "error") {
        setMessage(job.failure?.message ?? "连续分析失败；请显式继续或重启引擎。");
      }
      return;
    }
    if (job.outcome === "completed" || job.outcome === "cancelled" || job.outcome === "superseded" || job.outcome === "timeout" || job.outcome === "failed") {
      lastFiniteTerminalJobIdRef.current = job.job_id;
    }

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
      if (lastFiniteTerminalJobIdRef.current === started.job_id) return;
      beginReviewRequest(started.job_id);
      forgetSessionFrame(game.selected_path);
      selectedNodeJobRef.current = started;
      setSelectedNodeRunning(true);
      setMessage(`Running KataGo analysis (${started.job_id})...`);
    } catch (error) {
      setMessage(`KataGo analysis failed: ${errorMessage(error)}`);
    }
  }
  function sealReviewPresentation() {
    const token = createLocalRequestToken(() => {
      requestSerialRef.current += 1;
      return requestSerialRef.current;
    });
    activeRequestTokenRef.current = token;
    setActiveRequestToken(token);
    setCandidatePreview(null);
  }

  async function handleContinuousAnalysisAction() {
    if (!preferencesLoadSettledRef.current || continuousActionInFlightRef.current) return;
    const snapshot = engineSnapshotRef.current;
    if (preferencesSaveInFlightRef.current || pendingPreferencesSaveRef.current) return;
    const phase = snapshot.continuous.phase;
    if (nativeRuntime && (snapshot.continuous.enabled === null || phase === "loading")) return;
    if (departurePendingRef.current || phase === "departing" || phase === "stopping") {
      setMessage("正在等待离开或分析取消完成。");
      return;
    }
    if (snapshot.lifecycle.state === "error") {
      setMessage("前台引擎运行失败；请先显式重启引擎。");
      return;
    }
    if (!nativeRuntime) {
      const base = committedPreferencesRef.current;
      queuePreferencesSave(base, { continuousAnalysisEnabled: !base.continuousAnalysisEnabled });
      setMessage("浏览器预览只保存连续分析偏好，不会启动分析任务。");
      return;
    }
    if ((phase === "time_limited" || phase === "visits_limited" || phase === "paused" || phase === "error" || phase === "safety_hold")
      && (!admitsForegroundEngineJobs(snapshot) || !currentGameRef.current)) {
      setMessage("继续连续分析需要可用的前台引擎和当前棋谱。");
      return;
    }

    continuousActionInFlightRef.current = true;
    setContinuousActionPending(true);
    try {
      const saved = await foregroundEngineContinuousAction();
      committedPreferencesRef.current = saved;
      setPreferences(saved);
      setPreferencesStatus("Preferences saved.");
    } catch (error) {
      setPreferencesStatus(`Save failed: ${errorMessage(error)}`);
      setMessage(`连续分析操作失败: ${errorMessage(error)}`);
    } finally {
      continuousActionInFlightRef.current = false;
      setContinuousActionPending(false);
      void runPreferencesSaveLoop();
    }
  }


  function analysisScopeFromDraft(draft: AnalysisScopeDraft): AnalysisScopeDto {
    const game = currentGameRef.current;
    if (!game) throw new Error("Analysis scope requires a current game.");
    const interval = draft.intervalEnabled
      ? {
          start: parseU32(draft.intervalStart, "Interval start", true),
          end: parseU32(draft.intervalEnd, "Interval end", true)
        }
      : null;
    if (interval && interval.start > interval.end) {
      throw new Error("Interval start must not exceed interval end.");
    }
    return {
      mode: draft.mode,
      current_node: game.selected_path,
      branch_choices: analysisBranchChoices(chosenChildren),
      interval,
      to_play: draft.toPlay === "both" ? null : draft.toPlay
    };
  }

  function handleAnalysisScopeDraftChange(next: AnalysisScopeDraft) {
    const current = analysisScopeDraft;
    const conditionsChanged = next.overviewTimeEnabled !== current.overviewTimeEnabled
      || next.overviewTimeSeconds !== current.overviewTimeSeconds
      || next.overviewTotalVisitsEnabled !== current.overviewTotalVisitsEnabled
      || next.overviewTotalVisits !== current.overviewTotalVisits
      || next.overviewLeadingCandidateVisitsEnabled !== current.overviewLeadingCandidateVisitsEnabled
      || next.overviewLeadingCandidateVisits !== current.overviewLeadingCandidateVisits
      || next.singleTimeEnabled !== current.singleTimeEnabled
      || next.singleTimeSeconds !== current.singleTimeSeconds
      || next.singleTotalVisitsEnabled !== current.singleTotalVisitsEnabled
      || next.singleTotalVisits !== current.singleTotalVisits
      || next.singleLeadingCandidateVisitsEnabled !== current.singleLeadingCandidateVisitsEnabled
      || next.singleLeadingCandidateVisits !== current.singleLeadingCandidateVisits
      || next.timeEnabled !== current.timeEnabled
      || next.timeSeconds !== current.timeSeconds
      || next.totalVisitsEnabled !== current.totalVisitsEnabled
      || next.totalVisits !== current.totalVisits
      || next.leadingCandidateVisitsEnabled !== current.leadingCandidateVisitsEnabled
      || next.leadingCandidateVisits !== current.leadingCandidateVisits;
    if (conditionsChanged) analysisConditionsDraftEditedRef.current = true;
    const scopeChanged = next.strategy !== current.strategy
      || next.mode !== current.mode
      || next.intervalEnabled !== current.intervalEnabled
      || next.intervalStart !== current.intervalStart
      || next.intervalEnd !== current.intervalEnd
      || next.toPlay !== current.toPlay;
    setAnalysisScopeDraft(next);
    if (scopeChanged) setAnalysisScopePreview(null);
    setAnalysisTaskError(null);
  }

  async function handlePreviewAnalysisScope() {
    const game = currentGameRef.current;
    if (!nativeRuntime || !game) {
      setAnalysisTaskError(nativeRuntime ? "Analysis preview requires a current game." : nativeCurrentGameUnavailable);
      return;
    }
    setAnalysisTaskRequestPending(true);
    setAnalysisTaskError(null);
    try {
      const preview = await previewAnalysisScope({ generation: game.generation, scope: analysisScopeFromDraft(analysisScopeDraft) });
      setAnalysisScopePreview(preview);
    } catch (error) {
      setAnalysisTaskError(errorMessage(error));
    } finally {
      setAnalysisTaskRequestPending(false);
    }
  }

  function adoptAnalysisTask(next: AnalysisTaskDto | null) {
    const fence = analysisTaskPauseFenceRef.current;
    if (fence && !next) return;
    if (fence && next?.task_id === fence.taskId && next.job_id === fence.jobId) {
      if (fence.continued || next.state === "queued" || next.state === "searching") return;
    }
    if (fence && next) {
      if (next.task_id === fence.taskId && next.job_id !== fence.jobId && isAnalysisTaskReserved(next)) {
        fence.continued = true;
      } else if (next.task_id !== fence.taskId || !isAnalysisTaskReserved(next)) {
        analysisTaskPauseFenceRef.current = null;
      }
    }
    analysisTaskRef.current = next;
    setAnalysisTask(next);
    if (!next) return;
    const acceptsJobEvents = next.state === "queued" || next.state === "searching";
    setWholeGameRunning(isAnalysisTaskReserved(next));
    const stageCompleted = next.stage === "overview" ? next.overview_completed.length : next.completed.length;
    setWholeGameProgress({
      completed: stageCompleted,
      expected: next.requested.length,
      remaining: Math.max(0, next.requested.length - stageCompleted)
    });
    wholeGameJobRef.current = acceptsJobEvents ? {
      run_id: next.run_id,
      job_id: next.job_id,
      lane: "whole_game",
      mode: "finite",
      state: next.state === "searching" ? "searching" : "queued",
      generation: next.generation,
      node_path: next.requested[0] ?? { indices: [] }
    } : null;
  }

  async function refreshAnalysisTaskSnapshot(force = false) {
    if (!nativeRuntime || (analysisTaskActionInFlightRef.current && !force)) return analysisTaskRef.current;
    const request = ++analysisTaskSnapshotRequestRef.current;
    try {
      const next = await analysisTaskSnapshot();
      if (request === analysisTaskSnapshotRequestRef.current) adoptAnalysisTask(next);
      return next;
    } catch {
      return null;
    }
  }

  async function runAnalysisTask(input: {
    runId: string;
    scope: AnalysisScopeDto;
    strategy: AnalysisTaskStrategyDto;
    conditions: AnalysisStageConditionsDto;
    overviewConditions?: AnalysisStageConditionsDto | null;
    preview?: AnalysisScopePreviewDto | null;
    persistConditions?: boolean;
  }) {
    const game = currentGameRef.current;
    if (!nativeRuntime || !game) throw new Error("Analysis task requires a current game.");
    if (isAnalysisTaskReserved(analysisTaskRef.current)) {
      throw new Error("An analysis task is already active.");
    }
    const conditionsError = input.strategy === "all_positions_two_stage" && input.overviewConditions
      ? taskStageConditionsError(input.overviewConditions, input.conditions)
      : taskConditionsError(input.conditions);
    if (conditionsError) throw new Error(conditionsError);
    const preview = input.preview ?? await previewAnalysisScope({ generation: game.generation, scope: input.scope });
    if (preview.generation !== game.generation || JSON.stringify(preview.scope) !== JSON.stringify(input.scope)) {
      throw new Error("The scope preview is no longer current. Preview again before starting.");
    }
    if (input.persistConditions) {
      const changed = input.strategy === "all_positions_two_stage"
        ? input.overviewConditions != null
          && (JSON.stringify(input.overviewConditions) !== JSON.stringify(committedPreferencesRef.current.taskOverviewConditions)
            || JSON.stringify(input.conditions) !== JSON.stringify(committedPreferencesRef.current.taskDeepConditions))
        : JSON.stringify(input.conditions) !== JSON.stringify(committedPreferencesRef.current.taskSingleStageConditions);
      if (input.strategy === "all_positions_two_stage" && !input.overviewConditions) {
        throw new Error("All-position analysis requires overview conditions.");
      }
      if (changed) {
        if (preferencesSaveInFlightRef.current || pendingPreferencesSaveRef.current || continuousActionInFlightRef.current) {
          throw new Error("Wait for the current preference save before starting.");
        }
        const patch: Partial<AppPreferences> = input.strategy === "all_positions_two_stage"
          ? {
              taskOverviewConditions: input.overviewConditions!,
              taskDeepConditions: input.conditions
            }
          : { taskSingleStageConditions: input.conditions };
        await new Promise<AppPreferences>((resolve, reject) => {
          const pending = queuePreferencesSave(committedPreferencesRef.current, patch);
          pending.onSaved = resolve;
          pending.onFailed = reject;
        });
      }
    }
    const started = await startAnalysisTask({
      runId: input.runId,
      preview,
      strategy: input.strategy,
      conditions: input.conditions,
      overviewConditions: input.overviewConditions
    });
    ++analysisTaskSnapshotRequestRef.current;
    setAnalysisScopePreview(preview);
    adoptAnalysisTask(started);
    setMessage(`Analysis task ${started.state}: 0/${started.requested.length} completed.`);
    void refreshAnalysisTaskSnapshot();
  }

  async function handleStartAnalysisTask() {
    const run = runFromSnapshot(engineSnapshotRef.current);
    if (!run || !analysisScopePreview) return;
    setAnalysisTaskRequestPending(true);
    setAnalysisTaskError(null);
    try {
      const scope = analysisScopeFromDraft(analysisScopeDraft);
      const conditions = taskConditionsFromDraft(
        analysisScopeDraft,
        analysisScopeDraft.strategy === "single_stage" ? "single" : "deep"
      );
      const overviewConditions = analysisScopeDraft.strategy === "all_positions_two_stage"
        ? taskConditionsFromDraft(analysisScopeDraft, "overview")
        : null;
      await runAnalysisTask({
        runId: run.run_id,
        scope,
        strategy: analysisScopeDraft.strategy,
        conditions,
        overviewConditions,
        preview: analysisScopePreview,
        persistConditions: true
      });
    } catch (error) {
      const detail = errorMessage(error);
      setAnalysisTaskError(detail);
      setMessage(detail);
    } finally {
      setAnalysisTaskRequestPending(false);
    }
  }

  async function handleQuickAnalysisTask() {
    const run = runFromSnapshot(engineSnapshotRef.current);
    const game = currentGameRef.current;
    if (!run || !game || !admitsForegroundEngineJobs(engineSnapshotRef.current) || departurePendingRef.current || isAnalysisTaskReserved(analysisTaskRef.current)) return;
    setAnalysisTaskRequestPending(true);
    setAnalysisTaskError(null);
    try {
      await runAnalysisTask({
        runId: run.run_id,
        scope: presetAnalysisScope(game.selected_path, chosenChildren),
        strategy: "single_stage",
        conditions: {
          time_seconds: { enabled: false, value: 10 },
          total_visits: { enabled: true, value: 1 },
          leading_candidate_visits: { enabled: false, value: 500 }
        }
      });
    } catch (error) {
      const detail = errorMessage(error);
      setAnalysisTaskError(detail);
      setMessage(detail);
    } finally {
      setAnalysisTaskRequestPending(false);
    }
  }

  async function handleAllPositionsAnalysisTask() {
    const run = runFromSnapshot(engineSnapshotRef.current);
    const game = currentGameRef.current;
    if (!run || !game || !admitsForegroundEngineJobs(engineSnapshotRef.current) || departurePendingRef.current || isAnalysisTaskReserved(analysisTaskRef.current)) return;
    setAnalysisTaskRequestPending(true);
    setAnalysisTaskError(null);
    try {
      await runAnalysisTask({
        runId: run.run_id,
        scope: presetAnalysisScope(game.selected_path, chosenChildren),
        strategy: "all_positions_two_stage",
        overviewConditions: committedPreferencesRef.current.taskOverviewConditions,
        conditions: committedPreferencesRef.current.taskDeepConditions
      });
    } catch (error) {
      const detail = errorMessage(error);
      setAnalysisTaskError(detail);
      setMessage(detail);
    } finally {
      setAnalysisTaskRequestPending(false);
    }
  }

  async function handlePauseAnalysisTask() {
    const task = analysisTaskRef.current;
    const run = runFromSnapshot(engineSnapshotRef.current);
    if (analysisTaskActionInFlightRef.current
      || !task
      || (task.state !== "queued" && task.state !== "searching")
      || !run
      || run.run_id !== task.run_id
      || !admitsForegroundEngineJobs(engineSnapshotRef.current)
      || departurePendingRef.current
      || departurePrompt) return;
    analysisTaskActionInFlightRef.current = true;
    setAnalysisTaskRequestPending(true);
    setAnalysisTaskError(null);
    ++analysisTaskSnapshotRequestRef.current;
    analysisTaskPauseFenceRef.current = { taskId: task.task_id, jobId: task.job_id, continued: false };
    wholeGameJobRef.current = null;
    try {
      const paused = await pauseAnalysisTask({ runId: task.run_id, taskId: task.task_id });
      adoptAnalysisTask(paused);
      setMessage(`Analysis task ${paused.state}: ${paused.completed.length}/${paused.requested.length} completed.`);
    } catch (error) {
      analysisTaskPauseFenceRef.current = null;
      adoptAnalysisTask(task);
      const detail = errorMessage(error);
      setAnalysisTaskError(detail);
      setMessage(`Pause failed: ${detail}`);
    } finally {
      analysisTaskActionInFlightRef.current = false;
      setAnalysisTaskRequestPending(false);
    }
    void refreshAnalysisTaskSnapshot();
  }

  async function handleContinueAnalysisTask() {
    const task = analysisTaskRef.current;
    const run = runFromSnapshot(engineSnapshotRef.current);
    if (analysisTaskActionInFlightRef.current
      || task?.state !== "paused"
      || !run
      || run.run_id !== task.run_id
      || !admitsForegroundEngineJobs(engineSnapshotRef.current)
      || departurePendingRef.current
      || departurePrompt) return;
    analysisTaskActionInFlightRef.current = true;
    setAnalysisTaskRequestPending(true);
    setAnalysisTaskError(null);
    ++analysisTaskSnapshotRequestRef.current;
    try {
      const continued = await continueAnalysisTask({ runId: task.run_id, taskId: task.task_id });
      adoptAnalysisTask(continued);
      setMessage(`Analysis task ${continued.state}: ${continued.completed.length}/${continued.requested.length} completed.`);
    } catch (error) {
      const detail = errorMessage(error);
      setAnalysisTaskError(detail);
      setMessage(`Continue failed: ${detail}`);
    } finally {
      analysisTaskActionInFlightRef.current = false;
      setAnalysisTaskRequestPending(false);
    }
    void refreshAnalysisTaskSnapshot();
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
    if (isAnalysisTaskReserved(analysisTaskRef.current)) return;
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
      void refreshAnalysisTaskSnapshot();
    } catch (error) {
      setMessage(errorMessage(error));
    }
  }

  async function handleCancelSelectedNodeAnalysis() {
    const selected = selectedNodeJobRef.current;
    if (!selected) return;
    if (selected.mode === "continuous") {
      await handleContinuousAnalysisAction();
      return;
    }
    try {
      setMessage("Cancelling selected-node KataGo analysis...");
      await cancelSelectedNodeAnalysis({ runId: selected.run_id, jobId: selected.job_id });
    } catch (error) {
      setMessage(`Cancel failed: ${errorMessage(error)}`);
    }
  }

  async function handleCancelWholeGameAnalysis() {
    if (analysisTaskActionInFlightRef.current) return;
    const task = analysisTaskRef.current;
    const pending = wholeGameJobRef.current;
    const activeTask = isAnalysisTaskReserved(task) ? task : null;
    const runId = pending?.run_id ?? activeTask?.run_id;
    const jobId = pending?.job_id ?? activeTask?.job_id;
    if (!runId || !jobId) return;
    analysisTaskActionInFlightRef.current = true;
    setAnalysisTaskRequestPending(true);
    ++analysisTaskSnapshotRequestRef.current;
    try {
      setMessage("Cancelling analysis task...");
      await cancelKataGoAnalysis(runId, jobId);
      await refreshAnalysisTaskSnapshot(true);
    } catch (error) {
      setMessage(`Cancel failed: ${errorMessage(error)}`);
    } finally {
      analysisTaskActionInFlightRef.current = false;
      setAnalysisTaskRequestPending(false);
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
    const editedPath = currentGame.selected_path;
    try {
      const result = await setCurrentGamePersonalComment(editedPath, comment);
      if (!isCurrentGameSnapshot(result)) return;
      const latestPath = pendingSelectedPathRef.current ?? currentGameRef.current?.selected_path ?? editedPath;
      const stillOnEditedNode = samePath(latestPath, editedPath);
      const previous = currentGameRef.current;
      adoptCurrentGame(stillOnEditedNode || previous === null ? result : {
        ...result,
        selected_path: previous.selected_path,
        snapshot: previous.snapshot
      });
      if (stillOnEditedNode) {
        setChosenChildren((prev) => rememberChosenChildren(prev, result.selected_path));
        setCurrentMove(result.snapshot.position.move_number);
        setSelectedCandidateIndex(null);
      }
      setDirty(result.dirty);
      setMessage("已更新选中节点的个人评论。");
    } catch (error) {
      setMessage(`评论更新失败: ${errorMessage(error)}`);
    }
  }

  async function selectNode(path: NodePath) {
    const game = currentGameRef.current;
    if (!game) return;
    if (!navigatingRef.current && samePath(path, game.selected_path)) return;
    pendingSelectedPathRef.current = path;
    beginReviewRequest();
    if (navigatingRef.current) return;

    navigatingRef.current = true;
    try {
      while (pendingSelectedPathRef.current) {
        const requestedPath: NodePath = pendingSelectedPathRef.current;
        pendingSelectedPathRef.current = null;
        try {
          const result = await selectCurrentGameNode(requestedPath);
          const current = currentGameRef.current;
          if (!current || result.generation < current.generation) continue;
          if (result.generation === current.generation && result.snapshot_seq < current.snapshot_seq) {
            // Keep the latest cursor intent, but obtain its snapshot after the accepted edit/Save.
            pendingSelectedPathRef.current ??= requestedPath;
            continue;
          }
          adoptCurrentGame(result);
          setChosenChildren((previous) => rememberChosenChildren(previous, result.selected_path));
          if (result.snapshot.primary_analysis) presentCurrentGameAnalysis(result);
          else {
            const stored = wholeGameResultsRef.current.get(pathKey(result.selected_path));
            if (stored) presentWholeGameFrame(result.selected_path, stored);
          }
          setCurrentMove(result.snapshot.position.move_number);
          setSelectedCandidateIndex(null);
        } catch (error) {
          if (!pendingSelectedPathRef.current) setMessage(`导航失败: ${errorMessage(error)}`);
        }
      }
    } finally {
      navigatingRef.current = false;
      pendingSelectedPathRef.current = currentGameRef.current?.selected_path ?? null;
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
      onQuickAnalysis={() => void handleQuickAnalysisTask()}
      onAllPositionsAnalysis={() => void handleAllPositionsAnalysisTask()}
      continuousAnalysisAction={continuousAnalysisAction}
      onContinuousAnalysis={() => void handleContinuousAnalysisAction()}
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
    <div className="analysis-task-area">
    {recoveryProtection.status === "unprotected" ? (
      <div className="recovery-unprotected">
        <span role="status">{recoveryProtection.message}</span>
        <button type="button" aria-label="Retry recovery write" onClick={() => void handleRetryRecoveryWrite()}>
          重试
        </button>
      </div>
    ) : null}
    <AnalysisTaskPanel
      draft={analysisScopeDraft}
      preview={analysisScopePreview}
      task={analysisTask}
      canRun={nativeRuntime && engineReady && Boolean(currentGame) && !departurePending && !departurePrompt}
      busy={analysisTaskRequestPending}
      error={analysisTaskError}
      onDraftChange={handleAnalysisScopeDraftChange}
      onPreview={() => void handlePreviewAnalysisScope()}
      onStart={() => void handleStartAnalysisTask()}
      onPause={() => void handlePauseAnalysisTask()}
      onContinue={() => void handleContinueAnalysisTask()}
      onCancel={() => void handleCancelWholeGameAnalysis()}
    />
    </div>
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
      selectedNodeMode={selectedNodeJobRef.current?.mode}
      wholeGameRunning={wholeGameRunning}
      wholeGameProgress={wholeGameProgress}
      continuousAnalysisAction={continuousAnalysisAction}
      onContinuousAnalysis={() => void handleContinuousAnalysisAction()}
      onAnalyzeOnce={() => handleEngineCommand("once")}
      onAnalyzeGame={() => handleEngineCommand("game")}
      onQuickAnalysis={() => void handleQuickAnalysisTask()}
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
        disabled={!preferencesLoaded || continuousActionPending}
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

function isAnalysisTaskReserved(task: AnalysisTaskDto | null): boolean {
  return task != null && (task.state === "queued"
    || task.state === "searching"
    || task.state === "pausing"
    || task.state === "paused");
}

function continuousPhaseStatus(phase: ContinuousAnalysisPhaseDto): string {
  switch (phase) {
    case "loading": return "连续分析：正在载入设置";
    case "off": return "连续分析：已关闭";
    case "waiting": return "连续分析：等待可用引擎";
    case "unavailable": return "连续分析：当前引擎不可用";
    case "queued": return "连续分析：排队中";
    case "searching": return "连续分析：搜索中";
    case "stopping": return "连续分析：正在停止";
    case "time_limited": return "连续分析：已达时间限制";
    case "visits_limited": return "连续分析：已达 visits 限制";
    case "empty_board": return "连续分析：空棋盘停止已启用；请选择非空局面或关闭此设置";
    case "finite": return "连续分析：有限分析运行中";
    case "paused": return "连续分析：有限分析停止后已暂停";
    case "error": return "连续分析：已因错误停止";
    case "safety_hold": return "连续分析：离开操作中止后保持停止";
    case "departing": return "连续分析：正在离开当前棋谱";
  }
}

function preferencePatch(from: AppPreferences, to: AppPreferences): Partial<AppPreferences> {
  const patch: Partial<AppPreferences> = {};
  (Object.keys(to) as Array<keyof AppPreferences>).forEach((key) => {
    const unchanged = key === "taskOverviewConditions" || key === "taskDeepConditions"
      ? JSON.stringify(from[key]) === JSON.stringify(to[key])
      : from[key] === to[key];
    if (!unchanged) Object.assign(patch, { [key]: to[key] });
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
    && pending.mode === job.mode
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

function parseU32(value: string, label: string, allowZero: boolean): number {
  const parsed = Number(value);
  const minimum = allowZero ? 0 : 1;
  const maximum = 4294967295;
  if (!Number.isInteger(parsed) || parsed < minimum || parsed > maximum) {
    throw new Error(`${label} must be a whole number from ${minimum} to ${maximum}.`);
  }
  return parsed;
}

function analysisBranchChoices(chosen: Map<string, number>) {
  return [...chosen.entries()]
    .map(([key, child]) => ({
      parent: { indices: key === "" ? [] : key.split(",").map(Number) },
      child
    }))
    .sort((left, right) => left.parent.indices.length - right.parent.indices.length);
}

function presetAnalysisScope(currentNode: NodePath, chosen: Map<string, number>): AnalysisScopeDto {
  return {
    mode: "first_child_mainline",
    current_node: currentNode,
    branch_choices: analysisBranchChoices(chosen),
    interval: null,
    to_play: null
  };
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

function taskConditionsFromDraft(
  draft: AnalysisScopeDraft,
  stage: "overview" | "single" | "deep"
): AnalysisStageConditionsDto {
  const prefix = stage === "overview" ? "Overview" : stage === "single" ? "Single-stage" : "Deep";
  const timeEnabled = stage === "overview"
    ? draft.overviewTimeEnabled
    : stage === "single"
      ? draft.singleTimeEnabled
      : draft.timeEnabled;
  const timeSeconds = stage === "overview"
    ? draft.overviewTimeSeconds
    : stage === "single"
      ? draft.singleTimeSeconds
      : draft.timeSeconds;
  const totalVisitsEnabled = stage === "overview"
    ? draft.overviewTotalVisitsEnabled
    : stage === "single"
      ? draft.singleTotalVisitsEnabled
      : draft.totalVisitsEnabled;
  const totalVisits = stage === "overview"
    ? draft.overviewTotalVisits
    : stage === "single"
      ? draft.singleTotalVisits
      : draft.totalVisits;
  const leadingCandidateVisitsEnabled = stage === "overview"
    ? draft.overviewLeadingCandidateVisitsEnabled
    : stage === "single"
      ? draft.singleLeadingCandidateVisitsEnabled
      : draft.leadingCandidateVisitsEnabled;
  const leadingCandidateVisits = stage === "overview"
    ? draft.overviewLeadingCandidateVisits
    : stage === "single"
      ? draft.singleLeadingCandidateVisits
      : draft.leadingCandidateVisits;
  return {
    time_seconds: {
      enabled: timeEnabled,
      value: parseU32(timeSeconds, `${prefix} search time`, false)
    },
    total_visits: {
      enabled: totalVisitsEnabled,
      value: parseU32(totalVisits, `${prefix} total visits`, false)
    },
    leading_candidate_visits: {
      enabled: leadingCandidateVisitsEnabled,
      value: parseU32(leadingCandidateVisits, `${prefix} leading candidate visits`, false)
    }
  };
}
