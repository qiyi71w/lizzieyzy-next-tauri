import { useEffect, useMemo, useRef, useState } from "react";
import { Workspace } from "./workspace/Workspace";
import { useWorkspace } from "./workspace/useWorkspace";
import { acceptedMoveSound } from "./domain/acceptedMoveSound";
import { playMoveSound } from "./domain/moveSound";
import { BoardCanvas } from "./components/BoardCanvas";
import { ScoringControls } from "./components/ScoringControls";
import { WinrateChart } from "./components/WinrateChart";
import { ReviewTree } from "./components/ReviewTree";
import { reviewLineProblems, type ReviewProblem } from "./domain/reviewNavigation";
import { AnalysisPanel } from "./components/AnalysisPanel";
import { EngineSetupPanel } from "./components/EngineSetupPanel";
import { AppChrome, BottomBar, type ContinuousAnalysisAction, type OverlayMode, type SheetId } from "./components/AppChrome";
import { PreferencesPanel } from "./components/PreferencesPanel";
import { ShortcutReference } from "./components/ShortcutReference";
import { DocumentDepartureDialog } from "./components/DocumentDepartureDialog";
import { FunctionSearchPanel } from "./components/FunctionSearchPanel";
import { registeredFunctionCatalog, type FunctionSearchAction } from "./domain/functionSearch";
import { captureFocusReturn, restoreOwnedFocus, scheduleOwnedFocus, type FocusReturn } from "./domain/focusNavigation";
import { t, type ResourceKey } from "./i18n/resources";
import { AboutDialog } from "./components/AboutDialog";
import { ApplicationTeardownDialog } from "./components/ApplicationTeardownDialog";
import { CurrentGameRecoveryDialog } from "./components/CurrentGameRecoveryDialog";
import { AnalysisTaskPanel, type AnalysisScopeDraft } from "./components/AnalysisTaskPanel";
import { ProviderPanel } from "./components/ProviderPanel";
import { YikeSyncPanel } from "./components/YikeSyncPanel";
import { ReadboardSyncPanel } from "./components/ReadboardSyncPanel";
import { beginYikeSync, prepareYikeSync, beginReadboardSync, prepareReadboardSync, requestReadboardFocus, cancelExternalSyncStart, resolveExternalSyncStart, externalSyncSnapshot, subscribeExternalSync, subscribeReadboardSyncRequests, retryExternalSync, stopExternalSync, openYikeSyncBrowser } from "./api/externalSync";
import type { ExternalSyncSnapshot, ExternalSyncSource, ExternalSyncStart, ExternalSyncUpdate } from "./domain/providers";
import { NewDocumentDialog } from "./components/NewDocumentDialog";
import { GameMetadataDialog } from "./components/GameMetadataDialog";
import { HumanMatchDialog } from "./components/HumanMatchDialog";
import { humanMatchAction, humanMatchAnalysisPolicy, humanMatchSnapshot, humanMatchStart, humanMatchStop, pkMatchStart, pkMatchPause, pkMatchResume, nativeMatchUnavailable, subscribeHumanMatch } from "./api/match";
import type { HumanMatchActionDto, HumanMatchStartDto, MatchAnalysisPolicyDto, MatchDefaultsDto, MatchModeDto, MatchSnapshotDto, MatchUpdateDto } from "./domain/types";
import {
  analysisTaskSnapshot,
  applyRootSetup,
  cancelKataGoAnalysis,
  continueAnalysisTask,
  cancelSelectedNodeAnalysis,
  convertToRootSetup,
  classifyProblems,
  fakeAnalyze,
  enterTrial,
  exitTrial,
  enterScoring,
  updateScoring,
  exitScoring,
  foregroundEngineContinuousAction,
  getHealth,
  isTauriRuntime,
  importGameFile,
  nativeCurrentGameUnavailable,
  nativeSyntheticAnalysisUnavailable,
  openSgfDocument,
  readGameFile,
  takeInitialFileActivation,
  takePendingFileActivation,
  markFileActivationReady,
  setFileActivationBusy,
  subscribeFileActivationAvailable,
  subscribeFileActivationRejected,
  parseSgfSummary,
  editCurrentGameMarkup,
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
  currentGameRecoveryProtection,
  subscribeCurrentGameRecoveryProtection,
  saveCurrentGame,
  serializeCurrentGame,
  selectCurrentGameNode,
  setCurrentGamePersonalComment,
  setCurrentGameMetadata,
  removeCurrentGameVariation,
  promoteCurrentGameToMain,
  subscribeTrialAnalysis,
  trialPlay,
  trialSelect,
  trialStartFinite,
  trialUndo,
  undoCurrentGame,
  redoCurrentGame,
  startAnalysisTask,
  startKataGoGameAnalysis,
  startSelectedNodeAnalysis,
  subscribeForegroundEngine,
  startForegroundEngine,
  stopForegroundEngine,
  restartForegroundEngine,
  switchForegroundEngine,
} from "./api/backend";
import {
  admitsForegroundEngineJobs,
  admitsForegroundEngineQuery,
  verifiedEngineCapabilitiesLabel,
  canRestartForegroundEngine,
  canStopForegroundEngine,
  displayedEngineFailure,
  emptyForegroundEngineSnapshot,
  engineStatusLabel,
  runFromSnapshot,
  shouldAcceptFailureEvent
} from "./domain/foregroundEngine";
import { loadAppPreferences, saveAppPreferences, updateRecentGameHistory, updateWorkspaceVisibility } from "./api/preferences";
import { flushWindowGeometry, freezeWindowGeometry, nativeWindowGeometryUnavailable, resetWindowGeometry, retryWindowGeometry, subscribeWindowGeometryStatus, windowGeometryStatus } from "./api/windowGeometry";
import { useMainWindowPin } from "./hooks/useMainWindowPin";
import { clampMoveNumberToPositions, createDemoGame, replayGamePositions, selectExactPosition } from "./domain/board";
import { continuousBudgetError, defaultAppPreferences, normalizeAppPreferences, swingCriteriaError, taskConditionsError, taskStageConditionsError, type AppPreferences } from "./domain/preferences";
import { buildNextMoveReviewMarkers, cycleNextMoveReviewMarker } from "./domain/nextMoveReviewMarker";
import { admitsChartSeriesChange, buildWinrateChartModel, displayedWinrate } from "./domain/winrateChart";
import { providerDocumentName, providerLabel, providerSourceLabel, type ProviderImportResult, type ProviderRequestIdentity } from "./domain/providers";
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
import { newDocumentSgf, type NewDocumentParameters } from "./domain/newDocument";
import type { AnalysisFrameDto, AnalysisJobEventDto, AnalysisJobStartedDto, AnalysisScopeDto, AnalysisScopePreviewDto, AnalysisStageConditionsDto, AnalysisSwingCriteriaDto, AnalysisTaskDto, AnalysisTaskStrategyDto, AppHealthDto, ApplicationExitActionDto, ApplicationExitOutcomeDto, ContinuousAnalysisPhaseDto, CurrentGameResultDto, DocumentDepartureActionDto, EngineProfileDto, EngineProfileRecordDto, EngineFailureDto, FileActivationDeliveryDto, ForegroundEngineSnapshotDto, GameDto, GameFileImportDto, MoveVertex, NodePath, PlayerColor, PointDto, PositionDto, ProblemMarkerDto, RecoveryProtectionDto, RecoveryStartupDto, ScoringActionDto, ScoringSessionDto, SelectedNodeSnapshotDto, SgfMarkupActionDto, SgfMarkupToolDto, SgfTreeNodeDto, StoneDto, TrialSessionDto, WindowGeometryStatusDto } from "./domain/types";

const demoSgf = "(;GM[1]FF[4]SZ[19]KM[7.5]PB[李昌镐]PW[芮乃伟]RE[B+R];B[pd];W[dd];B[pp];W[dp];B[jq];W[qj];B[nc];W[fc];B[qf];W[cn];B[cp];W[do];B[co];W[dn];B[fq];W[eq];B[fp];W[gp];B[gq];W[hp])";
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
type CandidatePreview = { index: number; scope: ReviewPresentationScope; matchFrameIdentity: string | null };
type ReplacementOptions = {
  networkIdentity?: ProviderRequestIdentity;
  confirmMessage: string;
  fallbackName?: string | null;
  successMessage: (projection: GameDto, fileName: string) => string;
  failurePrefix: string;
  openedPath?: string | null;
};
type RootSetupDraft = { generation: number; stones: StoneDto[]; toPlay: PlayerColor; tool: PlayerColor | "erase" };
const defaultAnalysisScopeDraft: AnalysisScopeDraft = {
  strategy: "all_positions_two_stage",
  mode: "first_child_mainline",
  intervalEnabled: false,
  intervalStart: "0",
  intervalEnd: "0",
  toPlay: "both",
  moveActors: defaultAppPreferences.taskSwingCriteria.move_actors,
  winrateChangeEnabled: defaultAppPreferences.taskSwingCriteria.winrate_change_percentage_points.enabled,
  winrateChangeThreshold: String(defaultAppPreferences.taskSwingCriteria.winrate_change_percentage_points.value),
  scoreChangeEnabled: defaultAppPreferences.taskSwingCriteria.score_change_points.enabled,
  scoreChangeThreshold: String(defaultAppPreferences.taskSwingCriteria.score_change_points.value),
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
  const [matchState, setMatchState] = useState<MatchSnapshotDto | null>(null);
  const matchStateRef = useRef<MatchSnapshotDto | null>(null);
  const [matchDialogOpen, setMatchDialogOpen] = useState(false);
  const [matchDialogMode, setMatchDialogMode] = useState<MatchModeDto>("human");
  // Continue pins the node/generation the dialog describes; the backend rejects the start if the document moved on.
  const [matchContinuation, setMatchContinuation] = useState<Pick<CurrentGameResultDto, "generation" | "snapshot_seq" | "selected_path" | "snapshot" | "tree"> | null>(null);
  const [matchStarting, setMatchStarting] = useState(false);
  const [matchStartError, setMatchStartError] = useState<string | null>(null);
  const matchDocumentGenerationRef = useRef<number | null>(null);
  const matchStartingRef = useRef(false);
  const matchCancelRef = useRef(false);
  const matchActionPendingRef = useRef(false);
  const matchControlPendingRef = useRef<"pause" | "resume" | null>(null);
  const matchStopPendingRef = useRef(false);
  const [matchControlPending, setMatchControlPending] = useState<"pause" | "resume" | null>(null);
  const [matchStopPending, setMatchStopPending] = useState(false);
  const [matchAnalysisPending, setMatchAnalysisPending] = useState(false);
  const matchAnalysisPendingRef = useRef(false);
  const scoredMatchRef = useRef<string | null>(null);
  const committedMatchRef = useRef<string | null>(null);
  const matchBlocked = matchStarting || Boolean(matchState?.resources_held) || ["starting", "playing", "paused", "ending"].includes(matchState?.phase ?? "idle");
  const humanTurn = matchState?.mode === "human" && matchState.phase === "playing" && matchState.to_play === matchState.settings?.human_color;
  function matchOwnsWorkspace() {
    const state = matchStateRef.current;
    return matchStartingRef.current || Boolean(state?.resources_held) || ["starting", "playing", "paused", "ending"].includes(state?.phase ?? "idle");
  }
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
  const [externalSync, setExternalSync] = useState<ExternalSyncSnapshot | null>(null);
  const externalSyncRef = useRef<ExternalSyncSnapshot | null>(null);
  const [syncStarting, setSyncStarting] = useState<ExternalSyncSource | null>(null);
  const syncStartingRef = useRef(false);
  const syncStartIdRef = useRef<number | null>(null);
  const syncCancelRef = useRef(false);
  const externalBlocked = syncStarting !== null || externalSync?.session_id != null || externalSync?.starting_id != null;
  const [rootSetupDraft, setRootSetupDraft] = useState<RootSetupDraft | null>(null);
  const [conversionPrompt, setConversionPrompt] = useState<{ generation: number; path: NodePath } | null>(null);
  const [trial, setTrial] = useState<TrialSessionDto | null>(null);
  const trialRef = useRef<TrialSessionDto | null>(null);
  const [trialPending, setTrialPending] = useState(false);
  const trialTransitionRef = useRef(false);
  const [scoring, setScoring] = useState<ScoringSessionDto | null>(null);
  const scoringRef = useRef<ScoringSessionDto | null>(null);
  const [scoringPending, setScoringPending] = useState(false);
  const scoringPendingRef = useRef(false);
  useEffect(() => {
    if (!nativeRuntime) return;
    let disposed = false;
    let unlisten: (() => void) | null = null;
    void subscribeTrialAnalysis((next) => {
      const current = trialRef.current;
      if (!trialTransitionRef.current && current?.session_id === next.session_id && next.revision >= current.revision) {
        adoptTrial(next);
      }
    }).then((stop) => { if (disposed) stop(); else unlisten = stop; });
    return () => { disposed = true; unlisten?.(); };
  }, [nativeRuntime]);
  const [chosenChildren, setChosenChildren] = useState<Map<string, number>>(() => new Map());
  const navigatingRef = useRef(false);
  const documentGenerationRef = useRef(0);
  const pendingSelectedPathRef = useRef<NodePath | null>(null);
  const queuedSelectionRef = useRef<{ path: NodePath; generation: number; forward: boolean } | null>(null);
  const pendingBoardIntentRef = useRef<string | null>(null);
  const [editActionPending, setEditActionPending] = useState(false);
  const editActionPendingRef = useRef(false);
  const [preferences, setPreferences] = useState<AppPreferences>(() => defaultAppPreferences);
  const [preferencesStatus, setPreferencesStatus] = useState("正在载入设置…");
  const [recentHistoryBusy, setRecentHistoryBusy] = useState(false);
  const recentHistoryBusyRef = useRef(false);
  const [recentHistoryError, setRecentHistoryError] = useState<string | null>(null);
  const failedRecentActionRef = useRef<{ openedPath: string | null } | null>(null);
  const [sheet, setSheet] = useState<"none" | SheetId>("none");
  const [providerEntry, setProviderEntry] = useState<"yike" | "fox" | "tencent">("yike");
  const [newDocumentOpen, setNewDocumentOpen] = useState(false);
  const [metadataDraft, setMetadataDraft] = useState<{
    generation: number; blackName: string; whiteName: string; komi: number; handicap: string | null;
    focusTarget: "game.komi" | "game.black-name" | "game.white-name";
    returnToSearch: boolean;
  } | null>(null);
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
  const engineReady = !matchBlocked && admitsForegroundEngineQuery(engineSnapshot) && admitsForegroundEngineJobs(engineSnapshot, "visits_limit");
  const taskEngineReady = !matchBlocked && admitsForegroundEngineJobs(engineSnapshot, "whole_game_analysis");
  const continuousEngineReady = !matchBlocked && admitsForegroundEngineQuery(engineSnapshot, "continuous_analysis")
    && (!preferences.continuousVisitsLimitEnabled || admitsForegroundEngineJobs(engineSnapshot, "visits_limit"));
  const { showCoordinates, showMoveNumbers } = preferences;
  const [showBlackCandidates, setShowBlackCandidates] = useState(true);
  const [showWhiteCandidates, setShowWhiteCandidates] = useState(true);
  const [railVisibilityBusy, setRailVisibilityBusy] = useState(false);
  const railVisibilityBusyRef = useRef(false);
  const r7WritesRef = useRef(new Set<Promise<unknown>>());
  const [railVisibilityError, setRailVisibilityError] = useState<string | null>(null);
  const leftRailRef = useRef<HTMLElement>(null);
  const rightRailRef = useRef<HTMLElement>(null);
  const railRestoreRef = useRef<HTMLButtonElement>(null);
  const workspace = useWorkspace();
  const [windowGeometry, setWindowGeometry] = useState<WindowGeometryStatusDto>({ phase: "loading", geometry: null, error: null });
  const [windowGeometryActionPending, setWindowGeometryActionPending] = useState(false);
  const [windowGeometryError, setWindowGeometryError] = useState<string | null>(null);
  const windowGeometrySavedRef = useRef<WindowGeometryStatusDto["geometry"] | undefined>(undefined);
  const windowGeometryEventSequenceRef = useRef(0);
  const [layoutExitPrompt, setLayoutExitPrompt] = useState<{
    message: string; choose: (retry: boolean) => void;
  } | null>(null);
  const [finalLayoutExitError, setFinalLayoutExitError] = useState<string | null>(null);
  const [replayProgress, setReplayProgress] = useState({ identity: "", prefix: 0 });
  const [overlayMode, setOverlayMode] = useState<OverlayMode>("candidates");
  const [autoPlaying, setAutoPlaying] = useState(false);
  const [shortcutReferenceOpen, setShortcutReferenceOpen] = useState(false);
  const [functionSearchOpen, setFunctionSearchOpen] = useState(false);
  const searchSourceRef = useRef<FocusReturn | null>(null);
  const chromeSearchSourceRef = useRef<FocusReturn | null>(null);
  const workspaceRef = useRef<HTMLElement>(null);
  const [settingsTarget, setSettingsTarget] = useState<string | null>(null);
  const [aboutOpen, setAboutOpen] = useState(false);
  const searchSessionRef = useRef({ query: "", selectedId: null as string | null });
  const metadataSourceRef = useRef<FocusReturn | null>(null);
  const [keyboardPlacement, setKeyboardPlacement] = useState(false);
  const [markupTool, setMarkupTool] = useState<"play" | SgfMarkupToolDto["kind"]>("play");
  const [markupDialog, setMarkupDialog] = useState<{ point: PointDto; path: NodePath; generation: number; text: string } | null>(null);
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
  const pendingRecoveryContinuationRef = useRef<"discard_startup" | "native_exit" | null>(null);
  const exitInFlightRef = useRef(false);
  const [departurePending, setDeparturePending] = useState(false);
  const [fileFlowBusy, setFileFlowBusy] = useState(false);
  const fileFlowDepthRef = useRef(0);
  const newDocumentFlowReservedRef = useRef(false);
  const newDocumentAdmissionRef = useRef<Promise<void> | null>(null);
  const newDocumentSubmittingRef = useRef(false);
  const pendingStartupActivationRef = useRef<Extract<FileActivationDeliveryDto, { kind: "open" }> | null>(null);
  const pendingStartupRejectionRef = useRef<string | null>(null);
  const importPickerActiveRef = useRef(false);
  const importFileInputRef = useRef<HTMLInputElement | null>(null);
  const activationHandlerRef = useRef<(delivery: FileActivationDeliveryDto) => Promise<void>>(async () => undefined);
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
  const windowPin = useMainWindowPin(nativeRuntime, preferencesLoaded,
    departurePending || Boolean(departurePrompt) || Boolean(teardownPrompt) || workspace.frozen);
  const wholeGameJobRef = useRef<AnalysisJobStartedDto | null>(null);
  const wholeGameResultsRef = useRef<Map<string, AnalysisFrameDto>>(new Map());
  const handleSelectedNodeJobRef = useRef<(job: AnalysisJobEventDto) => void>(() => undefined);
  const handleWholeGameJobRef = useRef<(job: AnalysisJobEventDto) => void>(() => undefined);
  const analysisTaskRef = useRef<AnalysisTaskDto | null>(null);
  const analysisTaskSnapshotRequestRef = useRef(0);
  const analysisScopePreviewRequestRef = useRef(0);
  const analysisTaskActionInFlightRef = useRef(false);
  const analysisTaskPauseFenceRef = useRef<{ taskId: string; jobId: string; continued: boolean } | null>(null);
  const analysisConditionsDraftEditedRef = useRef(false);

  useEffect(() => {
    if (!nativeRuntime) return;
    let disposed = false;
    let receivedEvent = false;
    let unlisten: (() => void) | null = null;
    const adoptStatus = (status: WindowGeometryStatusDto) => {
      if (disposed) return;
      setWindowGeometry(status);
      setWindowGeometryError(null);
      if (status.phase === "saved") {
        windowGeometrySavedRef.current = status.geometry;
        if (preferencesLoadSettledRef.current) {
          committedPreferencesRef.current = { ...committedPreferencesRef.current, windowGeometry: status.geometry };
          setPreferences((current) => ({ ...current, windowGeometry: status.geometry }));
        }
      }
    };
    void subscribeWindowGeometryStatus((status) => {
      receivedEvent = true;
      windowGeometryEventSequenceRef.current += 1;
      adoptStatus(status);
    }).then(async (stop) => {
      if (disposed) { stop(); return; }
      unlisten = stop;
      try {
        const status = await windowGeometryStatus();
        if (!disposed && !receivedEvent) adoptStatus(status);
      } catch (error) {
        if (!disposed) setWindowGeometryError(errorMessage(error));
      }
    }).catch((error: unknown) => {
      if (!disposed) setWindowGeometryError(errorMessage(error));
    });
    return () => { disposed = true; unlisten?.(); };
  }, [nativeRuntime]);

  useEffect(() => {
    getHealth()
      .then(setHealth)
      .catch((error: unknown) => setMessage(errorMessage(error)));
  }, []);

  useEffect(() => () => {
    void releaseNewDocumentFlow(false);
  }, []);

  useEffect(() => {
    let isMounted = true;
    let subscriptionsDisposed = false;
    const unlisteners: Array<() => void> = [];
    const disposeSubscriptions = () => {
      subscriptionsDisposed = true;
      for (const unlisten of unlisteners.splice(0)) unlisten();
    };
    const registerSubscription = async (subscription: Promise<() => void>) => {
      const unlisten = await subscription;
      if (subscriptionsDisposed || !isMounted) {
        unlisten();
        return false;
      }
      unlisteners.push(unlisten);
      return true;
    };
    void (async () => {
      try {
        if (isTauriRuntime()) {
          const availableRegistered = await registerSubscription(subscribeFileActivationAvailable(() => {
            if (!isMounted) return;
            void takePendingFileActivation()
              .then((delivery) => {
                if (isMounted && delivery) return activationHandlerRef.current(delivery);
              })
              .catch((error: unknown) => {
                if (isMounted) setMessage(`Open failed: ${errorMessage(error)}`);
              });
          }));
          if (!availableRegistered) return;
          const rejectedRegistered = await registerSubscription(subscribeFileActivationRejected((rejection) => {
            if (isMounted) setMessage(rejection.message);
          }));
          if (!rejectedRegistered) return;
        }

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
          return;
        }
        if (!isMounted) return;

        const startupActivation = await takeInitialFileActivation();
        if (!isMounted) return;
        pendingStartupActivationRef.current = startupActivation?.kind === "open" ? startupActivation : null;
        pendingStartupRejectionRef.current = startupActivation?.kind === "rejected" ? startupActivation.message : null;
        if (startup.status === "abnormal") {
          setRecoveryPrompt(startup);
          return;
        }

        const explicitFile = pendingStartupActivationRef.current;
        const startupRejection = pendingStartupRejectionRef.current;
        pendingStartupActivationRef.current = null;
        pendingStartupRejectionRef.current = null;
        if (explicitFile) {
          await activationHandlerRef.current(explicitFile);
          await markFileActivationReady();
          return;
        }

        if (startup.status === "unreadable") {
          setMessage(startup.message);
        }
        if (startup.status === "normal" && loadedPrefs.restoreLastSession) {
          try {
            await restoreRecoveredDocument("已恢复上次棋谱。");
            if (startupRejection) setMessage(startupRejection);
            await markFileActivationReady();
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
        if (startupRejection) setMessage(startupRejection);
        else if (startup.status === "unreadable") setMessage(startup.message);
        await markFileActivationReady();
      } catch (error: unknown) {
        disposeSubscriptions();
        if (isMounted) setMessage(`File activation initialization failed: ${errorMessage(error)}`);
      }
    })();
    return () => {
      isMounted = false;
      disposeSubscriptions();
    };
  }, []);

  useEffect(() => {
    const input = importFileInputRef.current;
    if (!input) return;
    const handleCancel = () => void finishImportPicker();
    input.addEventListener("cancel", handleCancel);
    return () => input.removeEventListener("cancel", handleCancel);
  }, [sheet]);

  const reviewGame = currentGame && trial ? {
    ...currentGame, tree: trial.tree, selected_path: trial.selected_path,
    snapshot: trial.snapshot, generation: trial.revision,
    can_undo: trial.can_undo, can_redo: false
  } : currentGame;
  const currentPosition = useMemo(() => {
    if (reviewGame) return reviewGame.snapshot.position;
    return selectExactPosition(positions, currentMove, game.summary.board_width, game.summary.board_height);
  }, [reviewGame, currentMove, positions, game.summary.board_width, game.summary.board_height]);
  currentGameRef.current = currentGame;
  const selectedPersonalComment = trial ? "" : currentGame ? currentGame.snapshot.personal_comment : "";
  const selectedGeneratedInformation = trial ? null : currentGame?.snapshot.generated_information ?? null;
  const selectedPath = reviewGame?.selected_path ?? { indices: [] };
  const selectedNode = reviewGame ? nodeAt(reviewGame.tree, selectedPath) : null;
  const blackName = currentGame
    ? currentGame.tree.properties.find((property) => property.key === "PB")?.values[0]
    : game.summary.black_name;
  const whiteName = currentGame
    ? currentGame.tree.properties.find((property) => property.key === "PW")?.values[0]
    : game.summary.white_name;
  const selectedPathKey = selectedPath.indices.join(".");
  const activeScope = useMemo<ReviewPresentationScope>(() => ({
    generation: reviewGame?.generation ?? 0,
    selectedPath: selectedPath.indices,
    requestToken: activeRequestToken
  }), [reviewGame?.generation, selectedPathKey, activeRequestToken]);
  const presentationLive = publishedScope !== null && shouldPublishReviewPresentation(activeScope, publishedScope);
  const visibleFrames = useMemo(() => presentationLive ? frames : [], [presentationLive, frames]);
  const visibleProblems = useMemo(() => presentationLive ? problems : [], [presentationLive, problems]);
  const treeFrame = reviewGame?.snapshot.primary_analysis ?? undefined;
  const currentFrame = useMemo(
    () => {
      const sessionFrame = visibleFrames.length <= 1
        ? visibleFrames[0]
        : visibleFrames.find((frame) => frame.turn === currentMove) ?? visibleFrames.at(-1);
      return sessionFrame ?? treeFrame;
    },
    [visibleFrames, currentMove, treeFrame]
  );
  const matchFrame = useMemo(() => matchAnalysisPending ? undefined : currentMatchAnalysisFrame(matchState, currentGame), [matchState, currentGame, matchAnalysisPending]);
  const matchFrameIdentity = matchFrame && matchState?.analysis.frame
    ? JSON.stringify([matchState.session_id, matchState.turn, matchState.analysis.epoch, matchState.run_id, matchFrame.job_id, currentGame?.generation, selectedPathKey])
    : null;
  const presentationFrame = matchBlocked ? matchFrame : currentFrame;
  const visibleCurrentFrame = useMemo(() => applyPreferencesToFrame(presentationFrame, preferences), [presentationFrame, preferences]);
  useEffect(() => {
    setSelectedCandidateIndex(null);
    setCandidatePreview(null);
    setReplayProgress({ identity: "", prefix: 0 });
  }, [matchFrameIdentity]);
  const previewCandidateIndex = candidatePreview
    && candidatePreview.matchFrameIdentity === matchFrameIdentity
    && (matchBlocked ? Boolean(matchFrame) : presentationLive)
    && shouldPublishReviewPresentation(activeScope, candidatePreview.scope)
    ? candidatePreview.index
    : null;
  const hideCandidates = !matchBlocked && ((currentPosition.to_play === "black" && !showBlackCandidates)
    || (currentPosition.to_play === "white" && !showWhiteCandidates));
  const activeCandidateIndex = previewCandidateIndex ?? selectedCandidateIndex ?? 0;
  const activeCandidate = visibleCurrentFrame?.candidates[activeCandidateIndex]
    ?? visibleCurrentFrame?.candidates[0]
    ?? null;
  const replayCandidate = presentationFrame?.candidates[activeCandidateIndex]
    ?? presentationFrame?.candidates[0]
    ?? null;
  const replaySteps = variationReplayPointSteps(replayCandidate);
  const replayIdentityKeyValue = `${matchFrameIdentity ?? "review"}:${variationReplayIdentityKey(variationReplayIdentity(selectedPath, replayCandidate))}`;
  const replayArmed = preferences.variationReplayEnabled && replaySteps.length > 0;
  const replayEligible = replayArmed && (
    (preferences.showCandidates && overlayMode === "candidates" && !hideCandidates)
    || (preferences.showCandidates && preferences.subBoardContentMode === "variation" && preferences.workspaceVisibility.right)
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
  const parentNode = reviewGame && parentOfSelected ? nodeAt(reviewGame.tree, parentOfSelected) : null;
  const siblingIndex = selectedPath.indices.at(-1);
  const canParent = Boolean(!scoring && !scoringPending && reviewGame && parentOfSelected);
  const canRemoveVariation = Boolean(!externalBlocked && nativeRuntime && !trial && !trialPending && !scoring && !scoringPending && currentGame && selectedPath.indices.length > 0);
  const canRootSetup = !externalBlocked && !matchBlocked && nativeRuntime && Boolean(currentGame) && !trial && !trialPending && !scoring && !scoringPending && selectedPath.indices.length === 0
    && currentGame?.tree.children.length === 0 && !rootSetupDraft && !conversionPrompt && !editActionPending;
  const canConvertPosition = !externalBlocked && !matchBlocked && nativeRuntime && Boolean(currentGame) && !trial && !trialPending && !scoring && !scoringPending
    && (Boolean(currentGame?.tree.children.length) || Boolean(currentGame?.tree.properties.some((property) => property.key === "B" || property.key === "W")))
    && !rootSetupDraft && !conversionPrompt && !editActionPending;
  const canNext = Boolean(!scoring && !scoringPending && selectedNode && selectedNode.children.length > 0);
  const canPrevSibling = Boolean(!scoring && !scoringPending && parentNode && siblingIndex !== undefined && siblingIndex > 0);
  const canNextSibling = Boolean(!scoring && !scoringPending && parentNode && siblingIndex !== undefined && siblingIndex + 1 < parentNode.children.length);
  const siblingLabel = parentNode && siblingIndex !== undefined ? `${siblingIndex + 1}/${parentNode.children.length}` : "—";
  const maxMove = Math.max(positions.at(-1)?.move_number ?? 0, 1);
  const reviewIndex = currentGame ? currentPosition.move_number : currentMove;
  const chartModel = useMemo(() => buildWinrateChartModel({
    root: reviewGame?.tree ?? emptyChartRoot,
    chosen: chosenChildren,
    selectedPath,
    selectedToPlay: currentPosition.to_play,
    settings: preferences
  }), [reviewGame, chosenChildren, selectedPath.indices.length, currentMove, currentPosition.to_play, preferences]);
  const reviewMax = currentGame ? chartModel.points.at(-1)?.moveNumber ?? 0 : maxMove;
  const reviewProblems = useMemo(() => reviewGame ? reviewLineProblems(chartModel.points) : visibleProblems,
    [reviewGame, chartModel.points, visibleProblems]);
  const currentChartPoint = chartModel.points.find((point) => samePath(point.path, selectedPath));
  const chartWinrate = currentChartPoint
    ? displayedWinrate(currentChartPoint, chartModel.perspective, chartModel.selectedToPlay)
    : null;
  const chartTitleSide = chartModel.perspective === "sideToPlay" && chartModel.selectedToPlay === "white" ? "白" : "黑";
  const nextMoveMarkers = useMemo(() => buildNextMoveReviewMarkers({
    mode: preferences.nextMoveReviewMarker,
    selectedNode,
    selectedIsRoot: selectedPath.indices.length === 0,
    toPlay: currentPosition.to_play,
    boardWidth: currentPosition.board_width,
    boardHeight: currentPosition.board_height
  }), [preferences.nextMoveReviewMarker, selectedNode, selectedPath.indices.length, currentPosition.to_play, currentPosition.board_width, currentPosition.board_height]);
  const documentDirty = currentGame?.dirty ?? dirty;
  const recoveryDiscardRetryPending = recoveryPrompt !== null
    && recoveryProtection.status === "unprotected"
    && pendingRecoveryContinuationRef.current === "discard_startup";
  const continuousPhase: ContinuousAnalysisPhaseDto = nativeRuntime
    ? engineSnapshot.continuous.phase
    : preferences.continuousAnalysisEnabled ? "waiting" : "off";
  const continuousEnabled = nativeRuntime
    ? engineSnapshot.continuous.enabled
    : preferences.continuousAnalysisEnabled;
  const continuousAnalysisAction: ContinuousAnalysisAction = (() => {
    if (matchBlocked) return { label: CONTINUOUS_ANALYSIS_START_LABEL, disabled: true, title: "对局占用工作区。", status: "对局中" };
    if (scoring || scoringPending) {
      return { label: CONTINUOUS_ANALYSIS_START_LABEL, disabled: true, title: "计分期间不能启动分析。", status: continuousPhaseStatus(continuousPhase) };
    }
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
    const ownsContinuousJob = engineSnapshot.selected_node_job?.mode === "continuous";
    if (nativeRuntime && !continuousEngineReady && !ownsContinuousJob) {
      return { label: CONTINUOUS_ANALYSIS_START_LABEL, disabled: true, title: "当前 run 未验证连续分析能力；已保存的连续分析意图保持不变。", status: "连续分析：当前引擎不可用" };
    }
    if (!continuousEnabled) {
      return { label: CONTINUOUS_ANALYSIS_START_LABEL, disabled: continuousActionPending, status: continuousPhaseStatus(continuousPhase) };
    }
    if (continuousPhase === "time_limited" || continuousPhase === "visits_limited" || continuousPhase === "paused" || continuousPhase === "error" || continuousPhase === "safety_hold") {
      const resumable = nativeRuntime && continuousEngineReady && Boolean(currentGame);
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
  const documentFlowBusy = matchBlocked || matchDialogOpen || fileFlowBusy || departurePending || Boolean(newDocumentOpen) || Boolean(rootSetupDraft) || Boolean(conversionPrompt) || Boolean(metadataDraft) || Boolean(markupDialog) || Boolean(departurePrompt) || Boolean(teardownPrompt) || Boolean(recoveryPrompt);
  const historyActionBlocked = documentFlowBusy || externalBlocked || editActionPending;
  const canUndo = nativeRuntime && !scoring && !scoringPending && Boolean(trial ? trial.can_undo : currentGame?.can_undo) && !historyActionBlocked && !trialPending;
  const canRedo = nativeRuntime && !scoring && !scoringPending && !trial && Boolean(currentGame?.can_redo) && !historyActionBlocked && !trialPending;
  const canDeleteNode = Boolean(nativeRuntime && !scoring && !scoringPending && !trial && !trialPending && currentGame && !historyActionBlocked);
  const canPromoteMain = Boolean(nativeRuntime && !scoring && !scoringPending && !trial && !trialPending && currentGame && selectedPath.indices.some((index) => index !== 0) && !historyActionBlocked);
  const canReturnMain = Boolean(!scoring && !scoringPending && reviewGame && selectedPath.indices.some((index) => index !== 0) && !documentFlowBusy && !trialPending);


  useEffect(() => {
    setSelectedCandidateIndex(null);
  }, [currentMove]);

  useEffect(() => {
    if (!preferences.showCandidates || (selectedCandidateIndex !== null && selectedCandidateIndex >= preferences.candidateLimit)) {
      setSelectedCandidateIndex(null);
    }
  }, [preferences.showCandidates, preferences.candidateLimit, selectedCandidateIndex]);

  useEffect(() => {
    const openEnabled = nativeRuntime && !historyActionBlocked;
    const saveEnabled = nativeRuntime && !fileFlowBusy && !departurePending && !matchStarting && documentDirty;
    const passEnabled = humanTurn || openEnabled;

    shortcutRegistry.bind("file.new", () => {
      void handleNewGame();
    });
    shortcutRegistry.bind("game.board-dimensions", () => {
      void handleNewGame();
    });
    shortcutRegistry.bind("game.root-setup", () => {
      if (canRootSetup && !historyActionBlocked) openRootSetup();
    });
    shortcutRegistry.bind("game.convert-position", () => {
      if (canConvertPosition && !historyActionBlocked) promptPositionConversion();
    });
    shortcutRegistry.bind("game.metadata", (event) => {
      event.preventDefault();
      openMetadataEditor();
    });
    shortcutRegistry.bind("game.human-vs-engine", () => {
      setMessage(t("reason.n"));
    });
    shortcutRegistry.bind("review.try-play", () => {
      if (nativeRuntime && !trialPending) void handleToggleTrial();
    });
    shortcutRegistry.bind("review.scoring", () => { if (!scoringPending) void handleEnterScoring(); });
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
    for (let index = 0; index < 5; index += 1) {
      shortcutRegistry.bind(`file.recent-${index + 1}`, () => void handleOpenRecent(index));
    }
    shortcutRegistry.bind("file.clear-recent", () => void handleClearRecentHistory());
    shortcutRegistry.bind("file.retry-recent", () => void handleRetryRecentHistory());
    shortcutRegistry.bind("file.save", () => {
      if (saveEnabled) void handleSaveSgfDocument(false);
    });
    shortcutRegistry.bind("file.save-as", () => {
      if (nativeRuntime && !fileFlowBusy && !departurePending && !matchStarting) void handleSaveSgfDocument(true);
    });
    shortcutRegistry.bind("file.copy-sgf", () => {
      void handleCopySgf();
    });
    shortcutRegistry.bind("file.paste-sgf", () => {
      void handlePasteSgf();
    });
    shortcutRegistry.bind("edit.undo", () => {
      if (canUndo) void handleHistoryAction("undo");
    });
    shortcutRegistry.bind("edit.redo", () => {
      if (canRedo) void handleHistoryAction("redo");
    });
    for (const tool of ["label", "letters", "numbers", "circle", "square", "cross", "triangle", "erase"] as const) {
      shortcutRegistry.bind(`markup.${tool}`, () => { if (openEnabled && !trial && !trialPending) setMarkupTool(tool); });
    }
    shortcutRegistry.bind("markup.clear", () => { if (openEnabled && !trial && !trialPending) void commitMarkup({ kind: "clear" }); });
    shortcutRegistry.bind("review.pass", () => {
      if (passEnabled) void playAt("pass");
    });
    shortcutRegistry.bind("review.remove-variation", () => {
      if (canDeleteNode) void handleRemoveVariation();
    });
    shortcutRegistry.bind("review.promote-main", () => {
      if (canPromoteMain) void handlePromoteMain();
    });
    shortcutRegistry.bind("review.return-main", () => {
      if (canReturnMain) handleReturnMain();
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
      handleMoveSelect(Math.max(0, reviewIndex - 10));
    });
    shortcutRegistry.bind("review.forward-10", () => {
      handleMoveSelect(Math.min(reviewMax, reviewIndex + 10));
    });
    shortcutRegistry.bind("review.select-candidate", (event) => {
      const index = Number(event.key) - 1;
      if (visibleCurrentFrame?.candidates[index]) setSelectedCandidateIndex(index);
    });
    shortcutRegistry.bind("view.coordinates", () => {
      handlePreferencesChange({ ...preferences, showCoordinates: !preferences.showCoordinates });
    });
    shortcutRegistry.bind("view.move-numbers", () => {
      handlePreferencesChange({ ...preferences, showMoveNumbers: !preferences.showMoveNumbers });
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
      if (!scoring && !scoringPending) setAutoPlaying((value) => !value);
    });
    shortcutRegistry.bind("help.shortcut-reference", () => {
      setShortcutReferenceOpen((value) => !value);
    });
    shortcutRegistry.bind("navigation.function-search", () => openFunctionSearch());

    function onKey(event: KeyboardEvent) {
      if (shortcutRegistry.dispatch(event, "navigation.function-search")) return;
      if (matchOwnsWorkspace() && !departurePrompt) {
        if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "s" && !matchStartingRef.current) {
          event.preventDefault();
          void handleSaveSgfDocument(event.shiftKey);
        }
        return;
      }
      if (matchDialogOpen && !departurePrompt) return;
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
    matchBlocked,
    matchDialogOpen,
    humanTurn,
    matchStarting,
    fileFlowBusy,
    departurePending,
    chosenChildren,
    currentGame,
    trial,
    trialPending,
    scoring,
    scoringPending,
    documentDirty,
    canUndo,
    canRedo,
    canDeleteNode,
    canPromoteMain,
    canReturnMain,
    historyActionBlocked,
    nativeRuntime,
    positions,
    preferences,
    reviewIndex,
    reviewMax,
    departurePrompt,
    keyboardPlacement,
    shortcutRegistry,
    functionSearchOpen,
    visibleCurrentFrame,
    continuousPhase,
    preferencesLoaded,
    continuousActionPending,
    departurePending,
    engineReady,
    continuousEngineReady,
    taskEngineReady,
    engineSnapshot.lifecycle.state,
  ]);

  useEffect(() => {
    if (!autoPlaying || rootSetupDraft || conversionPrompt || markupDialog) return;
    const timer = window.setInterval(() => {
      if (reviewGame) {
        const node = nodeAt(reviewGame.tree, reviewGame.selected_path);
        if (!node || node.children.length === 0) {
          setAutoPlaying(false);
          return;
        }
        void selectNode(childPath(
          reviewGame.selected_path,
          chosenChildIndex(chosenChildren, reviewGame.selected_path, node.children.length)
        ), reviewGame.generation, true);
        return;
      }
      setCurrentMove((move) => {
        const next = clampMoveNumberToPositions(positions, move + 1);
        if (next >= Math.max(positions.at(-1)?.move_number ?? 0, 1)) setAutoPlaying(false);
        return next;
      });
    }, 800);
    return () => window.clearInterval(timer);
  }, [autoPlaying, positions, reviewGame, chosenChildren, rootSetupDraft, conversionPrompt, markupDialog]);

  function toggleSheet(next: SheetId) {
    if (matchOwnsWorkspace() && (next === "sync" || next === "engine" || next === "sgf")) return;
    setSettingsTarget(null);
    setSheet((current) => current === next ? "none" : next);
  }

  function openFunctionSearch(fromChrome = false) {
    if (document.querySelector('[role="dialog"]') || !workspaceRef.current) return;
    searchSourceRef.current = (fromChrome ? chromeSearchSourceRef.current : null) ?? captureFocusReturn(workspaceRef.current);
    chromeSearchSourceRef.current = null;
    setSettingsTarget(null);
    setFunctionSearchOpen(true);
  }

  function cancelFunctionSearch() {
    setFunctionSearchOpen(false);
    if (searchSourceRef.current) restoreOwnedFocus(searchSourceRef.current);
  }

  function openAbout() {
    if (!functionSearchOpen && workspaceRef.current) searchSourceRef.current = captureFocusReturn(workspaceRef.current);
    setSettingsTarget(null);
    setAboutOpen(true);
  }

  function openSearchTarget(target: string) {
    if (["game.komi", "game.black-name", "game.white-name"].includes(target)) {
      openMetadataEditor(target as "game.komi" | "game.black-name" | "game.white-name", true);
      return;
    }
    const targetSheet = target.startsWith("prefs.") ? "prefs" : target.startsWith("engine.") ? "engine" : null;
    if (!targetSheet || (targetSheet === "engine" && matchOwnsWorkspace())) { setMessage(t("search.targetUnavailable")); return; }
    setSheet(targetSheet);
    setSettingsTarget(target);
  }

  useEffect(() => {
    if (!settingsTarget || functionSearchOpen || metadataDraft || aboutOpen || shortcutReferenceOpen || matchDialogOpen) return;
    const owner = document.querySelector<HTMLElement>(`[data-focus-owner="${sheet}"]`);
    if (!owner) { setMessage(t("search.targetUnavailable")); return; }
    return scheduleOwnedFocus(owner, () => owner.querySelector(`[data-search-target="${settingsTarget}"]`), () => {
      owner.focus();
      setMessage(t("search.targetUnavailable"));
    });
  }, [settingsTarget, sheet, functionSearchOpen, metadataDraft, aboutOpen, shortcutReferenceOpen, matchDialogOpen]);

  function searchActionDisabled(id: string): ResourceKey | undefined {
    const documentReason = !nativeRuntime ? "reason.desktop" : matchBlocked ? "reason.match" : externalBlocked ? "reason.sync" : documentFlowBusy || editActionPending ? "reason.busy" : !currentGame ? "reason.noDocument" : undefined;
    const editReason = documentReason ?? (trial || trialPending || scoring || scoringPending ? "reason.trial" : undefined);
    if (id === "game.human-vs-engine") return "reason.n";
    if (id === "help.shortcut-reference") return undefined;
    if (id === "file.save" || id === "file.save-as") return !nativeRuntime ? "reason.desktop" : fileFlowBusy || departurePending || matchStarting ? "reason.busy" : id === "file.save" && !documentDirty ? "reason.clean" : undefined;
    if (id === "file.copy-sgf") return undefined;
    if (id === "file.clear-recent") return documentReason ?? (recentHistoryBusy ? "reason.busy" : preferences.recentGamePaths.length === 0 ? "reason.noTarget" : undefined);
    if (id === "file.retry-recent") return documentReason ?? (recentHistoryBusy ? "reason.busy" : !recentHistoryError ? "reason.noTarget" : undefined);
    if (id.startsWith("file.recent-")) return documentReason ?? (!preferences.recentGamePaths[Number(id.at(-1)) - 1] ? "reason.noTarget" : undefined);
    if (id.startsWith("file.")) return documentReason;
    if (id === "game.metadata") return editReason;
    if (id === "game.root-setup") return editReason ?? (!canRootSetup ? "reason.noTarget" : undefined);
    if (id === "game.convert-position") return editReason ?? (!canConvertPosition ? "reason.noTarget" : undefined);
    if (id.startsWith("game.")) return documentReason;
    if (id === "edit.undo") return documentReason ?? (!canUndo ? "reason.noUndo" : undefined);
    if (id === "edit.redo") return editReason ?? (!canRedo ? "reason.noRedo" : undefined);
    if (id.startsWith("markup.")) return editReason;
    if (id === "analysis.continuous") return continuousAnalysisAction.disabled ? (!continuousEngineReady ? "reason.engine" : "reason.busy") : undefined;
    if (id.startsWith("analysis.")) return documentReason ?? (!taskEngineReady ? "reason.engine" : wholeGameRunning ? "reason.busy" : undefined);
    if (id === "review.try-play") return documentReason ?? (trialPending || scoring || scoringPending ? "reason.trial" : undefined);
    if (id === "review.scoring") return editReason;
    if (id === "review.pass") return humanTurn ? undefined : documentReason;
    if (id === "review.remove-variation") return editReason ?? (!canDeleteNode ? "reason.noTarget" : undefined);
    if (id === "review.promote-main") return editReason ?? (!canPromoteMain ? "reason.noTarget" : undefined);
    if (id === "review.return-main") return documentReason ?? (!canReturnMain ? "reason.noTarget" : undefined);
    if (id === "review.select-candidate") return matchBlocked ? "reason.match" : !visibleCurrentFrame?.candidates.length ? "reason.noTarget" : undefined;
    if (id.startsWith("view.") || id === "review.next-move-marker") return !preferencesLoaded || workspace.frozen ? "reason.busy" : matchBlocked && id === "view.policy-overlay" ? "reason.match" : undefined;
    return matchBlocked ? "reason.match" : documentFlowBusy ? "reason.busy" : trialPending || scoring || scoringPending ? "reason.trial" : undefined;
  }

  const functionCatalog: FunctionSearchAction[] = [
    ...registeredFunctionCatalog(shortcutRegistry, searchActionDisabled),
    { id: "help.about", label: "action.help.about", keywords: ["关于", "guanyu", "about", "version", "build"], execute: openAbout },
    ...([
      ["prefs.candidate-limit", "target.prefs.candidate-limit", ["候选", "houxuan", "candidate", "limit", "settings"]],
      ["prefs.replay-interval", "target.prefs.replay-interval", ["变化", "bianhua", "variation", "replay", "interval"]],
      ["prefs.board-theme", "target.prefs.board-theme", ["棋盘", "qipan", "theme", "contrast"]],
      ["engine.model-path", "target.engine.model-path", ["模型", "moxing", "model", "path"]],
      ["engine.config-path", "target.engine.config-path", ["配置", "peizhi", "config", "path"]],
      ["game.komi", "target.game.komi", ["贴目", "tiemu", "komi", "game info"]],
      ["game.black-name", "target.game.black-name", ["黑方", "heifang", "black", "name"]],
      ["game.white-name", "target.game.white-name", ["白方", "baifang", "white", "name"]]
    ] as const).map(([id, label, keywords]): FunctionSearchAction => ({ id, label, keywords,
      disabledReason: id.startsWith("game.") ? searchActionDisabled("game.metadata") : id.startsWith("engine.") && matchBlocked ? "reason.match" : !preferencesLoaded || workspace.frozen ? "reason.busy" : undefined,
      execute: () => openSearchTarget(id) })),
    ...([
      ["game.human-new", "action.game.human-new", false, "human"],
      ["game.human-continue", "action.game.human-continue", true, "human"],
      ["game.pk-new", "action.game.pk-new", false, "pk"],
      ["game.pk-continue", "action.game.pk-continue", true, "pk"]
    ] as const).map(([id, label, continuing, mode]): FunctionSearchAction => ({ id, label, keywords: ["人机", "renji", "human", "match", mode, continuing ? "continue" : "new"],
      disabledReason: searchActionDisabled("game.metadata"), execute: () => openMatchDialog(continuing, mode) }))
  ];

  useEffect(() => {
    let cancelled = false;
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
    if (matchOwnsWorkspace()) return;
    if (scoringRef.current || scoringPendingRef.current || (trialRef.current && kind === "game")) return;
    const snapshot = engineSnapshotRef.current;
    const run = runFromSnapshot(snapshot);
    if (!run || !admitsForegroundEngineJobs(snapshot, kind === "once" ? "selected_node_analysis" : "whole_game_analysis")) {
      setMessage("当前 run 未验证所需的分析能力。历史分析仍可查看。");
      return;
    }
    const maxVisits = run.profile_snapshot.adapter_kind === "kata_go_analysis" ? run.profile_snapshot.settings.max_visits : 800;
    if (kind === "once") void handleRunKataGo(run.profile_snapshot, maxVisits);
    else void handleAnalyzeKataGoGame(run.run_id, maxVisits);
  }

  async function handleSelectSwitcherProfile(profileId: string) {
    if (matchOwnsWorkspace()) return;
    if (!profileId) return;
    const run = runFromSnapshot(engineSnapshot);
    if (run?.profile_id === profileId) return;
    const state = engineSnapshot.lifecycle.state;
    if (state !== "no_engine" && state !== "ready" && state !== "switching") return;
    const profile = engineProfiles.find((record) => record.id === profileId)?.profile;
    if (!profile) return;
    setEngineFailure(null);
    try {
      if (state === "no_engine") await startForegroundEngine(profileId);
      else await switchForegroundEngine(profileId);
    } catch (error) {
      setMessage(errorMessage(error));
    }
  }


  function handlePreferencesChange(nextPreferences: AppPreferences) {
    if (workspace.owner.snapshot.frozen) return;
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

  async function handleRailVisibility(side: "left" | "right", visible: boolean) {
    if (matchOwnsWorkspace()) return;
    if (workspace.owner.snapshot.frozen || !preferencesLoadSettledRef.current || railVisibilityBusyRef.current || departurePendingRef.current || departurePrompt) return;
    railVisibilityBusyRef.current = true;
    setRailVisibilityBusy(true);
    setRailVisibilityError(null);
    try {
      const workspaceVisibility = await trackR7Write(updateWorkspaceVisibility({ [side]: visible }));
      const rail = side === "left" ? leftRailRef.current : rightRailRef.current;
      const active = document.activeElement;
      const matchingSeparatorFocused = Boolean(
        active && (active.classList?.contains(`workspace-separator-${side}`) || active.closest?.(`.workspace-separator-${side}`))
      );
      if (!visible && (rail?.contains(active) || matchingSeparatorFocused)) railRestoreRef.current?.focus();
      committedPreferencesRef.current = { ...committedPreferencesRef.current, workspaceVisibility };
      setPreferences((current) => ({ ...current, workspaceVisibility }));
    } catch (error) {
      setRailVisibilityError(`侧栏保存失败：${errorMessage(error)}；请再次操作重试。`);
    } finally {
      railVisibilityBusyRef.current = false;
      setRailVisibilityBusy(false);
    }
  }

  function settleLoadedPreferences(loaded: AppPreferences, status: string) {
    preferencesLoadSettledRef.current = true;
    setPreferencesLoaded(true);
    workspace.owner.load(loaded.workspaceShares);
    const current = windowGeometrySavedRef.current === undefined ? loaded : { ...loaded, windowGeometry: windowGeometrySavedRef.current };
    committedPreferencesRef.current = current;
    setPreferences(current);
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
        leadingCandidateVisits: String(loaded.taskDeepConditions.leading_candidate_visits.value),
        moveActors: loaded.taskSwingCriteria.move_actors,
        winrateChangeEnabled: loaded.taskSwingCriteria.winrate_change_percentage_points.enabled,
        winrateChangeThreshold: String(loaded.taskSwingCriteria.winrate_change_percentage_points.value),
        scoreChangeEnabled: loaded.taskSwingCriteria.score_change_points.enabled,
        scoreChangeThreshold: String(loaded.taskSwingCriteria.score_change_points.value),
      }));
    }
    setPreferencesStatus(status);
  }

  function trackR7Write<T>(write: Promise<T>): Promise<T> {
    r7WritesRef.current.add(write);
    void write.then(
      () => { r7WritesRef.current.delete(write); },
      () => { r7WritesRef.current.delete(write); }
    );
    return write;
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
          const saved = { ...await trackR7Write(saveAppPreferences(pending.preferences)), matchDefaults: committedPreferencesRef.current.matchDefaults, recentGamePaths: committedPreferencesRef.current.recentGamePaths, windowGeometry: committedPreferencesRef.current.windowGeometry, workspaceVisibility: committedPreferencesRef.current.workspaceVisibility };
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


  async function adoptExternalUpdate(update: ExternalSyncUpdate) {
    const previousSync = externalSyncRef.current;
    if (update.sync.revision >= (previousSync?.revision ?? -1)) {
      externalSyncRef.current = update.sync;
      setExternalSync(update.sync);
    }
    const current = update.current;
    const previous = currentGameRef.current;
    if (!current || (previous && (current.generation < previous.generation || (current.generation === previous.generation && current.snapshot_seq <= previous.snapshot_seq)))) return;
    if (previousSync?.session_id !== update.sync.session_id) clearLocalAnalysisSession();
    beginReviewRequest();
    clearReviewData();
    adoptCurrentGame(current);
    pendingSelectedPathRef.current = current.selected_path;
    setChosenChildren(chosenFromPath(current.selected_path));
    setCurrentMove(current.snapshot.position.move_number);
    setDirty(current.dirty);
    setCurrentFilePath(current.native_path ?? null);
    setFallbackFileName(update.sync.source === "readboard" ? "readboard-sync.sgf" : "yike-sync.sgf");
    presentCurrentGameAnalysis(current);
    if (previous && previousSync?.session_id === update.sync.session_id && !syncMuted(update.sync)) soundAcceptedMove(previous.snapshot, current.snapshot);
    try {
      const artifacts = await artifactsFromCurrentGame();
      if (currentGameRef.current?.generation !== current.generation || currentGameRef.current?.snapshot_seq !== current.snapshot_seq) return;
      setGame(artifacts.projection);
      setSgfText(artifacts.serialized);
      setPositions(replayGamePositions(artifacts.projection));
    } catch (error) { setMessage(`同步棋谱投影失败: ${errorMessage(error)}`); }
  }

  useEffect(() => {
    if (!nativeRuntime) return;
    let disposed = false;
    let unsubscribe: (() => void) | undefined;
    let querying = false;
    const refresh = async () => {
      if (querying) return;
      querying = true;
      try { const sync = await externalSyncSnapshot(); if (!disposed) await adoptExternalUpdate({ sync, current: null }); }
      catch (error) { if (!disposed) setMessage(`同步状态读取失败: ${errorMessage(error)}`); }
      finally { querying = false; }
    };
    void subscribeExternalSync((update) => { if (!disposed) void adoptExternalUpdate(update); })
      .then((stop) => { if (disposed) stop(); else { unsubscribe = stop; void refresh(); } })
      .catch((error) => { if (!disposed) setMessage(`同步事件订阅失败: ${errorMessage(error)}`); });
    const timer = window.setInterval(() => { if (externalSyncRef.current?.session_id != null || syncStartingRef.current) void refresh(); }, 250);
    return () => { disposed = true; unsubscribe?.(); window.clearInterval(timer); };
  }, [nativeRuntime]);

  const readboardStartRef = useRef<() => Promise<void>>(async () => {});
  readboardStartRef.current = handleStartReadboardSync;
  useEffect(() => {
    if (!nativeRuntime) return;
    let disposed = false;
    let unsubscribe: (() => void) | undefined;
    void subscribeReadboardSyncRequests(() => {
      if (disposed || syncStartingRef.current || externalSyncRef.current?.source === "readboard") return;
      setSheet("sync");
      void readboardStartRef.current().catch((error) => setMessage(`readboard 同步启动失败: ${errorMessage(error)}`));
    })
      .then((stop) => { if (disposed) stop(); else unsubscribe = stop; })
      .catch((error) => { if (!disposed) setMessage(`readboard 同步请求订阅失败: ${errorMessage(error)}`); });
    return () => { disposed = true; unsubscribe?.(); };
  }, [nativeRuntime]);

  function handleStartSync(locator: string, play: boolean) {
    return runExternalStart("yike", async () => {
      const id = await beginYikeSync(locator);
      return { id, prepare: () => prepareYikeSync(id) };
    }, "保存当前棋谱后开始 Yike 只读同步？取消会保留当前局和原同步会话。", async (id) => {
      if (!play) return;
      const opened = await openYikeSyncBrowser(id);
      await adoptExternalUpdate({ sync: opened, current: null });
      if (opened.browser_error) setMessage(opened.browser_error);
    });
  }

  function handleStartReadboardSync() {
    return runExternalStart("readboard", async () => {
      const id = await beginReadboardSync();
      return { id, prepare: () => prepareReadboardSync(id) };
    }, "保存当前棋谱后开始 readboard 只读同步？取消会保留当前局和原同步会话。");
  }

  // One SGF-07 Start for both sources: reserve, prepare the candidate, one dirty decision, commit.
  async function runExternalStart(
    source: ExternalSyncSource,
    begin: () => Promise<{ id: number; prepare: () => Promise<ExternalSyncStart> }>,
    decision: string,
    afterCommit?: (id: number) => Promise<void>
  ) {
    if (!nativeRuntime || syncStartingRef.current || matchOwnsWorkspace() || documentFlowBusy || trialRef.current || scoringRef.current || editActionPendingRef.current) throw new Error("请先完成或取消当前编辑、试下或棋谱事务。");
    syncStartingRef.current = true;
    syncCancelRef.current = false;
    setSyncStarting(source);
    let enteredFileFlow = false;
    try {
      await enterFileFlow();
      enteredFileFlow = true;
      const { id, prepare } = await begin();
      syncStartIdRef.current = id;
      if (syncCancelRef.current) { await cancelExternalSyncStart(id); return; }
      const prepared = await prepare();
      if (syncCancelRef.current) { await cancelExternalSyncStart(id); return; }
      const action: DocumentDepartureActionDto = prepared.admission.status === "needs_decision"
        ? await requestDepartureDecision(decision) : "discard";
      if (action !== "cancel") { departurePendingRef.current = true; setDeparturePending(true); }
      const outcome = await resolveExternalSyncStart({ departureId: prepared.admission.departure_id, action, selectedPath: currentGameRef.current?.selected_path ?? { indices: [] }, defaultFileName: saveFileName });
      const sync = await externalSyncSnapshot();
      await adoptExternalUpdate({ sync, current: outcome.current ?? null });
      setMessage(outcome.message);
      if (outcome.committed && sync.session_id === id) await afterCommit?.(id);
    } catch (error) {
      if (syncStartIdRef.current != null) {
        try { await cancelExternalSyncStart(syncStartIdRef.current); }
        catch { /* Cleanup failure must not replace the original start error. */ }
      }
      if (!syncCancelRef.current) throw error;
    } finally {
      syncStartIdRef.current = null;
      syncStartingRef.current = false;
      setSyncStarting(null);
      departurePendingRef.current = false;
      setDeparturePending(false);
      if (enteredFileFlow) await leaveFileFlow();
    }
  }

  async function handleCancelSyncStart() {
    syncCancelRef.current = true;
    departurePrompt?.choose("cancel");
    const id = syncStartIdRef.current ?? externalSyncRef.current?.starting_id;
    if (id != null) await adoptExternalUpdate({ sync: await cancelExternalSyncStart(id), current: null });
  }

  async function handleSyncAction(action: "retry" | "stop" | "browser") {
    const id = externalSyncRef.current?.session_id;
    if (id == null) return;
    if (action === "stop") await adoptExternalUpdate(await stopExternalSync(id));
    else await adoptExternalUpdate({ sync: await (action === "retry" ? retryExternalSync(id) : openYikeSyncBrowser(id)), current: null });
  }


  async function adoptMatchUpdate(update: MatchUpdateDto) {
    const next = update.match_state;
    if (next.revision <= (matchStateRef.current?.revision ?? -1)) return;
    matchStateRef.current = next;
    setMatchState(next);
    if (next.committed && next.settings && committedMatchRef.current !== next.session_id) {
      committedMatchRef.current = next.session_id;
      committedPreferencesRef.current = { ...committedPreferencesRef.current, matchDefaults: next.settings };
      setPreferences((current) => ({ ...current, matchDefaults: next.settings! }));
      setMatchDialogOpen(false);
    }
    if (matchCancelRef.current && next.phase === "starting" && next.session_id) void handleStopMatch();
    const current = update.current;
    // Document generation/snapshot_seq order independently of analysis-only session revisions.
    const previous = currentGameRef.current;
    if (!current || (previous && (current.generation < previous.generation || current.snapshot_seq < previous.snapshot_seq))) return;
    if (previous && current.generation === previous.generation && current.snapshot_seq === previous.snapshot_seq) return;
    matchDocumentGenerationRef.current = current.generation;
    clearLocalAnalysisSession();
    beginReviewRequest();
    clearReviewData();
    setCandidatePreview(null);
    setAutoPlaying(false);
    setSheet("none");
    adoptCurrentGame(current);
    setChosenChildren(chosenFromPath(current.selected_path));
    pendingSelectedPathRef.current = current.selected_path;
    setCurrentMove(current.snapshot.position.move_number);
    setDirty(current.dirty);
    setCurrentFilePath(current.native_path ?? null);
    if (previous && next.phase === "playing") soundAcceptedMove(previous.snapshot, current.snapshot);
    try {
      const artifacts = await artifactsFromCurrentGame();
      if (currentGameRef.current?.generation !== current.generation || currentGameRef.current?.snapshot_seq !== current.snapshot_seq) return;
      setGame(artifacts.projection);
      setSgfText(artifacts.serialized);
      setPositions(replayGamePositions(artifacts.projection));
    } catch (error) { setMessage(`对局棋谱投影失败: ${errorMessage(error)}`); }
  }

  useEffect(() => {
    if (!nativeRuntime) return;
    let disposed = false;
    let unsubscribe: (() => void) | undefined;
    void subscribeHumanMatch((update) => { if (!disposed) void adoptMatchUpdate(update); })
      .then(async (stop) => {
        if (disposed) { stop(); return; }
        unsubscribe = stop;
        const update = await humanMatchSnapshot();
        if (!disposed) await adoptMatchUpdate(update);
      }).catch((error) => { if (!disposed) setMessage(`对局状态读取失败: ${errorMessage(error)}`); });
    return () => { disposed = true; unsubscribe?.(); };
  }, [nativeRuntime]);

  useEffect(() => {
    if (matchState?.end !== "two_passes" || matchState.resources_held || matchBlocked || documentFlowBusy
      || currentGameRef.current?.generation !== matchDocumentGenerationRef.current
      || !matchState.session_id || scoredMatchRef.current === matchState.session_id) return;
    scoredMatchRef.current = matchState.session_id;
    void handleEnterScoring();
  }, [matchState, matchBlocked, documentFlowBusy]);

  function openMatchDialog(continuing: boolean, mode: MatchModeDto = "human") {
    if (externalBlocked) return;
    if (!nativeRuntime) { setMessage(nativeMatchUnavailable); return; }
    if (documentFlowBusy || trial || trialPending || scoring || scoringPending || !preferencesLoaded) return;
    const baseline = continuing ? currentGameRef.current : null;
    if (continuing && !baseline) return;
    setAutoPlaying(false);
    setSheet("none");
    setMatchDialogMode(mode);
    setMatchStartError(null);
    setMatchContinuation(baseline ? { generation: baseline.generation, snapshot_seq: baseline.snapshot_seq, selected_path: baseline.selected_path, snapshot: baseline.snapshot, tree: baseline.tree } : null);
    setMatchDialogOpen(true);
  }

  async function handleStartMatch(settings: MatchDefaultsDto, continuation: typeof matchContinuation, mode: MatchModeDto) {
    const continuing = continuation !== null;
    if (!nativeRuntime || matchOwnsWorkspace() || fileFlowDepthRef.current !== 0 || !currentGameRef.current) return;
    matchStartingRef.current = true;
    matchCancelRef.current = false;
    setMatchStarting(true);
    setMatchStartError(null);
    let entered = false;
    try {
      await enterFileFlow();
      entered = true;
      let discardConfirmed = false;
      // A Continue keeps the current document and file; only a New game departs from it.
      if (!continuing && currentGameRef.current.dirty) {
        const decision = await requestDepartureDecision(`保存当前棋谱后开始${mode === "pk" ? " PK " : "人机"}新局？放弃仅在新局启动成功后替换原谱。`);
        if (decision === "cancel" || matchCancelRef.current) return;
        if (decision === "save") {
          await handleSaveSgfDocument(false);
          if (currentGameRef.current?.dirty || matchCancelRef.current) return;
        } else discardConfirmed = true;
      }
      if (matchCancelRef.current) return;
      const baseline = continuation ?? currentGameRef.current;
      if (!baseline) return;
      const request: HumanMatchStartDto = { settings, generation: baseline.generation, snapshot_seq: baseline.snapshot_seq,
        start: continuing ? { kind: "continue", node_path: baseline.selected_path, root_metadata_confirmed: true }
          : { kind: "new", discard_confirmed: discardConfirmed } };
      await adoptMatchUpdate(await (mode === "pk" ? pkMatchStart(request) : humanMatchStart(request)));
    } catch (error) {
      const failure = `${mode === "pk" ? "PK" : "人机"}启动失败: ${errorMessage(error)}`;
      setMatchStartError(failure);
      setMessage(failure);
    }
    finally {
      matchStartingRef.current = false;
      setMatchStarting(false);
      if (entered) await leaveFileFlow();
    }
  }

  async function handleHumanAction(action: HumanMatchActionDto) {
    const state = matchStateRef.current;
    const current = currentGameRef.current;
    if (matchActionPendingRef.current || !state?.session_id || state.mode !== "human" || state.phase !== "playing"
      || state.to_play !== state.settings?.human_color || !current) return;
    matchActionPendingRef.current = true;
    try {
      await adoptMatchUpdate(await humanMatchAction({ session_id: state.session_id, turn: state.turn, generation: current.generation, node_path: current.selected_path }, action));
    } catch (error) { setMessage(`人类落子失败: ${errorMessage(error)}`); }
    finally { matchActionPendingRef.current = false; }
  }

  async function handlePkControl(action: "pause" | "resume") {
    const state = matchStateRef.current;
    if (matchControlPendingRef.current || matchStopPendingRef.current || !state?.session_id || state.mode !== "pk"
      || state.pause_pending || state.resume_pending || (action === "pause" ? state.phase !== "playing" : state.phase !== "paused")) return;
    matchControlPendingRef.current = action;
    setMatchControlPending(action);
    try {
      await adoptMatchUpdate(await (action === "pause" ? pkMatchPause(state.session_id) : pkMatchResume(state.session_id)));
    } catch (error) { setMessage(`${action === "pause" ? "暂停" : "恢复"} PK 失败，可重试: ${errorMessage(error)}`); }
    finally { matchControlPendingRef.current = null; setMatchControlPending(null); }
  }

  async function handleMatchAnalysisPolicy(policy: MatchAnalysisPolicyDto) {
    const state = matchStateRef.current;
    const current = currentGameRef.current;
    if (!nativeRuntime || matchAnalysisPendingRef.current || matchStopPendingRef.current || !state?.session_id
      || state.mode !== "human" || state.phase !== "playing" || !state.analysis.supported || !current) return;
    matchAnalysisPendingRef.current = true;
    setMatchAnalysisPending(true);
    try {
      await adoptMatchUpdate(await humanMatchAnalysisPolicy({ session_id: state.session_id, turn: state.turn, generation: current.generation, node_path: current.selected_path }, policy));
    } catch (error) { setMessage(`本场分析策略更新失败: ${errorMessage(error)}`); }
    finally {
      matchAnalysisPendingRef.current = false;
      setMatchAnalysisPending(false);
    }
  }

  async function handleStopMatch() {
    matchCancelRef.current = true;
    const state = matchStateRef.current;
    const sessionOwnsWorkspace = Boolean(state?.resources_held) || ["starting", "playing", "paused", "ending"].includes(state?.phase ?? "idle");
    if (!state?.session_id || !sessionOwnsWorkspace) { if (!matchStartingRef.current) setMatchDialogOpen(false); return; }
    if (matchStopPendingRef.current) return;
    matchStopPendingRef.current = true;
    setMatchStopPending(true);
    try { await adoptMatchUpdate(await humanMatchStop(state.session_id)); }
    catch (error) { setMessage(`停止对局失败，可重试: ${errorMessage(error)}`); }
    finally { matchStopPendingRef.current = false; setMatchStopPending(false); }
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

  function adoptTrial(next: TrialSessionDto) {
    if (trialRef.current?.session_id !== next.session_id || trialRef.current.revision <= next.revision) {
      trialRef.current = next;
      setTrial(next);
      setCurrentMove(next.snapshot.position.move_number);
      setSelectedCandidateIndex(null);
    }
  }

  async function handleToggleTrial() {
    if (externalBlocked) return;
    if (matchOwnsWorkspace()) return;
    if (!nativeRuntime || !currentGameRef.current || scoringRef.current || scoringPendingRef.current || trialTransitionRef.current || (!trialRef.current && documentFlowBusy) || departurePendingRef.current || fileFlowDepthRef.current !== 0 || navigatingRef.current || analysisTaskActionInFlightRef.current || editActionPendingRef.current) return;
    trialTransitionRef.current = true;
    setTrialPending(true);
    setAutoPlaying(false);
    sealReviewPresentation();
    try {
      const active = trialRef.current;
      if (active) {
        const returned = await exitTrial(active.session_id);
        selectedNodeJobRef.current = null;
        setSelectedNodeRunning(false);
        trialRef.current = null;
        setTrial(null);
        setChosenChildren(chosenFromPath(returned.selected_path));
        adoptCurrentGame(returned);
        setCurrentMove(returned.snapshot.position.move_number);
        setMessage("已退出试下，回到原谱节点。");
      } else {
        const entered = await enterTrial();
        selectedNodeJobRef.current = null;
        setSelectedNodeRunning(false);
        adoptTrial(entered);
        setChosenChildren(new Map());
        setAnalysisScopePreview(null);
        void analysisTaskSnapshot().then(adoptAnalysisTask).catch((error) => {
          setMessage(`试下已进入；范围任务状态读取失败: ${errorMessage(error)}`);
        });
        setMessage("已进入独立试下；保存和复制仍使用原谱。");
      }
    } catch (error) {
      setMessage(`试下切换失败: ${errorMessage(error)}`);
    } finally {
      trialTransitionRef.current = false;
      setTrialPending(false);
    }
  }
  async function handleEnterScoring() {
    if (externalBlocked) return;
    if (matchOwnsWorkspace()) return;
    if (!nativeRuntime || !currentGameRef.current || scoringRef.current || scoringPendingRef.current || trialRef.current || trialTransitionRef.current || documentFlowBusy || navigatingRef.current || analysisTaskActionInFlightRef.current || editActionPendingRef.current) return;
    scoringPendingRef.current = true;
    setScoringPending(true);
    setAutoPlaying(false);
    sealReviewPresentation();
    try {
      const entered = await enterScoring(preferences.scoringRule);
      scoringRef.current = entered;
      setScoring(entered);
      selectedNodeJobRef.current = null;
      setSelectedNodeRunning(false);
      setAnalysisScopePreview(null);
      void analysisTaskSnapshot().then(adoptAnalysisTask).catch((error) => setMessage(`计分已进入；范围任务状态读取失败: ${errorMessage(error)}`));
      setMessage("本地计分：点击棋子切换整组死活，点击空点切换中立。原谱尚未更改。");
    } catch (error) {
      setMessage(`进入计分失败: ${errorMessage(error)}`);
    } finally {
      scoringPendingRef.current = false;
      setScoringPending(false);
    }
  }

  async function handleUpdateScoring(action: ScoringActionDto) {
    const active = scoringRef.current;
    if (!active || scoringPendingRef.current) return;
    scoringPendingRef.current = true;
    setScoringPending(true);
    try {
      const next = await updateScoring(active.session_id, active.revision, action);
      if (scoringRef.current?.session_id === active.session_id) {
        scoringRef.current = next;
        setScoring(next);
      }
    } catch (error) {
      setMessage(`计分修正失败: ${errorMessage(error)}`);
    } finally {
      scoringPendingRef.current = false;
      setScoringPending(false);
    }
  }

  function handleScoringRule(rule: "area" | "territory") {
    const active = scoringRef.current;
    if (!active || scoringPendingRef.current || rule === active.rule) return;
    if (preferencesSaveInFlightRef.current || pendingPreferencesSaveRef.current || continuousActionInFlightRef.current) {
      setMessage("请等待当前设置保存完成后再更换计分规则。");
      return;
    }
    scoringPendingRef.current = true;
    setScoringPending(true);
    const pending = queuePreferencesSave(committedPreferencesRef.current, { scoringRule: rule });
    pending.onSaved = () => {
      scoringPendingRef.current = false;
      setScoringPending(false);
      void handleUpdateScoring({ kind: "settings", rule, compensation: active.compensation, handicap: active.handicap });
    };
    pending.onFailed = (error) => {
      scoringPendingRef.current = false;
      setScoringPending(false);
      setMessage(`计分规则保存失败: ${errorMessage(error)}`);
    };
  }

  async function handleExitScoring(confirm: boolean) {
    const active = scoringRef.current;
    if (!active || scoringPendingRef.current) return;
    scoringPendingRef.current = true;
    setScoringPending(true);
    try {
      const result = await exitScoring(active.session_id, active.revision, confirm);
      scoringRef.current = null;
      setScoring(null);
      adoptCurrentGame(result);
      setDirty(result.dirty);
      setCurrentMove(result.snapshot.position.move_number);
      if (confirm) {
        const projection = await projectCurrentGameMainline();
        if (isCurrentDocumentGeneration(result.generation)) setGame(projection);
      }
      setMessage(confirm ? `已确认并写入结果 ${active.result}。` : "已取消计分，原谱结果未变。");
    } catch (error) {
      setMessage(`退出计分失败: ${errorMessage(error)}`);
    } finally {
      scoringPendingRef.current = false;
      setScoringPending(false);
    }
  }

  async function refreshCurrentGameAfterAttach() {
    const game = currentGameRef.current;
    if (!nativeRuntime || !game || departurePendingRef.current || navigatingRef.current) return;
    const requestToken = activeRequestTokenRef.current;
    try {
      const refreshed = await selectCurrentGameNode(game.selected_path, game.generation);
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
    if (!frame || frame.visits === 0) return;
    const captured: ReviewPresentationScope = {
      generation: result.generation,
      selectedPath: [...result.selected_path.indices],
      requestToken: activeRequestTokenRef.current
    };
    void classifyProblems([frame])
      .then((classified) => publishReviewPresentation(captured, [frame], classified))
      .catch((error) => setMessage(`分析结果读取失败: ${errorMessage(error)}`));
  }


  async function enterFileFlow(): Promise<void> {
    const outermost = fileFlowDepthRef.current === 0;
    fileFlowDepthRef.current += 1;
    if (!outermost) return;
    setFileFlowBusy(true);
    try {
      await setFileActivationBusy(true);
    } catch (error) {
      fileFlowDepthRef.current -= 1;
      setFileFlowBusy(false);
      throw error;
    }
  }

  async function leaveFileFlow(updateState = true): Promise<void> {
    fileFlowDepthRef.current = Math.max(0, fileFlowDepthRef.current - 1);
    if (fileFlowDepthRef.current !== 0) return;
    try {
      await setFileActivationBusy(false);
    } finally {
      if (updateState) setFileFlowBusy(false);
    }
  }

  async function releaseNewDocumentFlow(updateState = true): Promise<void> {
    if (!newDocumentFlowReservedRef.current) return;
    newDocumentFlowReservedRef.current = false;
    const admission = newDocumentAdmissionRef.current;
    newDocumentAdmissionRef.current = null;
    if (admission) {
      await admission.then(() => leaveFileFlow(updateState), () => undefined);
      return;
    }
    await leaveFileFlow(updateState);
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

  const windowGeometryDisabled = !nativeRuntime || !preferencesLoaded || workspace.frozen || windowGeometryActionPending;
  const windowGeometryFailure = windowGeometryError ?? windowGeometry.error;
  const windowGeometrySystemManaged = windowGeometry.geometry?.x === null && windowGeometry.geometry.y === null;
  const windowGeometryStatusText = !nativeRuntime ? nativeWindowGeometryUnavailable
    : windowGeometryFailure ? `窗口位置未保存：${windowGeometryFailure}`
    : windowGeometry.phase === "saved" ? windowGeometrySystemManaged ? "窗口尺寸已保存（位置由系统管理）" : "窗口位置已保存"
    : windowGeometry.phase === "pending" ? windowGeometrySystemManaged ? "窗口尺寸待保存（位置由系统管理）" : "窗口位置待保存"
    : windowGeometry.phase === "saving" ? windowGeometrySystemManaged ? "正在保存窗口尺寸（位置由系统管理）…" : "正在保存窗口位置…"
    : windowGeometry.phase === "unsaved" ? "窗口位置未保存"
    : "正在读取窗口位置…";

  async function performWindowGeometryAction(action: () => Promise<WindowGeometryStatusDto>) {
    if (windowGeometryDisabled) return;
    const eventSequence = windowGeometryEventSequenceRef.current;
    setWindowGeometryActionPending(true);
    try {
      const status = await trackR7Write(action());
      if (windowGeometryEventSequenceRef.current === eventSequence) {
        setWindowGeometry(status);
        setWindowGeometryError(null);
      }
    } catch (error) {
      if (windowGeometryEventSequenceRef.current === eventSequence) setWindowGeometryError(errorMessage(error));
    } finally {
      setWindowGeometryActionPending(false);
    }
  }

  async function flushR7Writes(): Promise<void> {
    const drain = async () => {
      do {
        await Promise.all([
          workspace.owner.flush(), flushWindowGeometry(), windowPin.flush(),
          ...r7WritesRef.current
        ]);
      } while (r7WritesRef.current.size > 0);
    };
    let timer: number | undefined;
    try {
      await Promise.race([
        drain(),
        new Promise<never>((_, reject) => {
          timer = window.setTimeout(() => reject(new Error("R7 preference saves timed out after 5 seconds.")), 5000);
        })
      ]);
    } finally {
      window.clearTimeout(timer);
    }
  }

  async function flushWorkspaceBeforeDeparture(): Promise<boolean> {
    while (true) {
      try {
        await flushR7Writes();
        return true;
      } catch (error) {
        const retry = await new Promise<boolean>((choose) => {
          setLayoutExitPrompt({ message: errorMessage(error), choose });
        });
        setLayoutExitPrompt(null);
        if (!retry) return false;
      }
    }
  }

  async function finishNativeExit(): Promise<void> {
    try {
      await flushR7Writes();
      setFinalLayoutExitError(null);
      await confirmNativeExit();
    } catch (error) {
      setFinalLayoutExitError(errorMessage(error));
    }
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
      pendingRecoveryContinuationRef.current = "native_exit";
      setMessage(current.recovery_persist_error);
      setRecoveryProtection({ status: "unprotected", message: current.recovery_persist_error });
      return;
    }
    await finishNativeExit();
  }

  async function handleApplicationExit() {
    if (!nativeRuntime) {
      setMessage(nativeCurrentGameUnavailable);
      return;
    }
    if (exitInFlightRef.current || departurePrompt || teardownPrompt || workspace.frozen) return;
    exitInFlightRef.current = true;
    let enteredFileFlow = false;
    let committed = false;
    let windowFreezeAttempted = false;
    try {
      if (!preferencesLoadSettledRef.current) {
        setMessage("设置仍在载入，请稍后退出。");
        return;
      }
      await enterFileFlow();
      enteredFileFlow = true;
      if (!await flushWorkspaceBeforeDeparture()) return;
      workspace.owner.freeze(true);
      windowFreezeAttempted = true;
      await freezeWindowGeometry(true);
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
        if (outcome.analysis_stopped) void refreshAnalysisTaskSnapshot(true);
        return;
      }
      committed = true;
      trialRef.current = null;
      setTrial(null);
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
      if (!committed) {
        if (windowFreezeAttempted) {
          try { await freezeWindowGeometry(false); }
          catch (error) { setMessage(`恢复窗口保存失败: ${errorMessage(error)}`); }
        }
        workspace.owner.freeze(false);
      }
      if (enteredFileFlow) await leaveFileFlow();
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
    trialRef.current = null;
    setTrial(null);
    scoringRef.current = null;
    setScoring(null);
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
    options: Omit<ReplacementOptions, "confirmMessage">
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
      if (outcome.analysis_stopped) void refreshAnalysisTaskSnapshot(true);
      return false;
    }
    if (!outcome.current) {
      setMessage(`${options.failurePrefix}: replacement committed without a current game`);
      return false;
    }
    if (options.openedPath) await persistRecentHistory(options.openedPath);
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
      const pending = pendingStartupActivationRef.current;
      const startupRejection = pendingStartupRejectionRef.current;
      pendingStartupActivationRef.current = null;
      pendingStartupRejectionRef.current = null;
      if (pending) await activationHandlerRef.current(pending);
      if (startupRejection) setMessage(startupRejection);
      await markFileActivationReady();
    } catch (error: unknown) {
      setMessage(`恢复失败: ${errorMessage(error)}`);
    }
  }

  async function finishDiscardRecoveredGame() {
    pendingRecoveryContinuationRef.current = null;
    setRecoveryPrompt(null);
    const pending = pendingStartupActivationRef.current;
    const startupRejection = pendingStartupRejectionRef.current;
    pendingStartupActivationRef.current = null;
    pendingStartupRejectionRef.current = null;
    if (pending) {
      await activationHandlerRef.current(pending);
    } else {
      await applyReplacement(demoSgf, null, {
        confirmMessage: "放弃未保存的棋谱并载入示例？",
        fallbackName: "sample.sgf",
        successMessage: (projection) => `Sample SGF restored: ${projection.summary.move_count} moves.`,
        failurePrefix: "Sample load failed"
      });
    }
    if (startupRejection) setMessage(startupRejection);
    await markFileActivationReady();
  }

  async function handleDiscardRecoveredGame() {
    try {
      await discardCurrentGameRecovery();
      await finishDiscardRecoveredGame();
    } catch (error: unknown) {
      const protection = await currentGameRecoveryProtection();
      if (protection.status === "unprotected") {
        pendingRecoveryContinuationRef.current = "discard_startup";
        setRecoveryProtection(protection);
        setMessage(protection.message);
      } else {
        setRecoveryProtection(protection);
        setMessage(`放弃恢复失败: ${errorMessage(error)}`);
      }
    }
  }

  async function handleRetryRecoveryWrite() {
    try {
      const protection = await retryCurrentGameRecovery();
      setRecoveryProtection(protection);
      if (protection.status === "protected") {
        if (pendingRecoveryContinuationRef.current === "discard_startup") {
          await finishDiscardRecoveredGame();
          return;
        }
        if (pendingRecoveryContinuationRef.current === "native_exit") {
          await finishNativeExit();
          pendingRecoveryContinuationRef.current = null;
          return;
        }
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
    options: ReplacementOptions
  ): Promise<boolean> {
    if (externalBlocked && !options.networkIdentity) { setMessage("外部来源占用只读棋谱；请先 Stop 再导入或编辑。"); return false; }
    if (matchOwnsWorkspace()) return false;
    if (rootSetupDraft || conversionPrompt || metadataDraft || markupDialog) {
      setMessage("请先完成或取消当前棋谱编辑。");
      return false;
    }
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
    await enterFileFlow();

    try {
      const admission = await prepareDocumentReplacement(sgfInput, nativePath, options.networkIdentity);
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
      await leaveFileFlow();
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

  async function replaceImportedGame(document: GameFileImportDto): Promise<boolean> {
    return applyReplacement(document.sgf_text, document.native_path ?? null, {
      confirmMessage: "放弃未保存的棋谱并打开这个文件？",
      fallbackName: document.format === "gib"
        ? document.display_name.replace(/\.gib$/i, ".sgf")
        : null,
      successMessage: (projection, fileName) => `Opened ${fileName}: ${projection.summary.move_count} moves.`,
      failurePrefix: "Open failed",
      openedPath: document.opened_path,
    });
  }

  async function persistRecentHistory(openedPath: string | null) {
    recentHistoryBusyRef.current = true;
    setRecentHistoryBusy(true);
    try {
      const recentGamePaths = await updateRecentGameHistory(openedPath);
      committedPreferencesRef.current = { ...committedPreferencesRef.current, recentGamePaths };
      setPreferences((current) => ({ ...current, recentGamePaths }));
      failedRecentActionRef.current = null;
      setRecentHistoryError(null);
    } catch (error) {
      failedRecentActionRef.current = { openedPath };
      setRecentHistoryError(`最近记录写入失败，保留已保存列表：${errorMessage(error)}`);
    } finally {
      recentHistoryBusyRef.current = false;
      setRecentHistoryBusy(false);
    }
  }

  async function handleOpenRecent(index: number) {
    if (!nativeRuntime || !preferencesLoaded || documentFlowBusy || fileFlowDepthRef.current !== 0 || recentHistoryBusyRef.current) return;
    const path = committedPreferencesRef.current.recentGamePaths[index];
    if (!path) return;
    await enterFileFlow();
    try {
      await replaceImportedGame(await readGameFile(path));
    } catch (error) {
      setMessage(`Open failed: ${errorMessage(error)}`);
    } finally {
      await leaveFileFlow();
    }
  }

  async function handleClearRecentHistory() {
    if (!nativeRuntime || !preferencesLoaded || fileFlowDepthRef.current !== 0 || recentHistoryBusyRef.current) return;
    await persistRecentHistory(null);
  }

  async function handleRetryRecentHistory() {
    if (!nativeRuntime || !preferencesLoaded || fileFlowDepthRef.current !== 0 || recentHistoryBusyRef.current) return;
    const action = failedRecentActionRef.current;
    if (action) await persistRecentHistory(action.openedPath);
  }

  async function handleFileActivation(delivery: FileActivationDeliveryDto): Promise<void> {
    if (matchOwnsWorkspace()) { setMessage("对局期间不能打开其他棋谱。"); return; }
    if (delivery.kind === "rejected") {
      setMessage(delivery.message);
      return;
    }
    if (rootSetupDraft || conversionPrompt || metadataDraft || markupDialog) {
      setMessage("请先完成或取消当前棋谱编辑；外部打开请求已拒绝。");
      return;
    }
    if (fileFlowDepthRef.current !== 0) {
      setMessage("Another file action is active; the external open request was rejected.");
      return;
    }
    await enterFileFlow();
    try {
      const document = await readGameFile(delivery.path);
      await replaceImportedGame(document);
    } catch (error) {
      setMessage(`Open failed: ${errorMessage(error)}`);
    } finally {
      await leaveFileFlow();
    }
  }

  activationHandlerRef.current = handleFileActivation;

  async function handleOpenSgfDocument() {
    if (documentFlowBusy) return;
    if (!nativeRuntime) {
      setMessage(nativeCurrentGameUnavailable);
      return;
    }
    await enterFileFlow();
    try {
      const document = await openSgfDocument();
      if (!document) return;
      await replaceImportedGame(document);
    } catch (error) {
      setMessage(`Open failed: ${errorMessage(error)}`);
    } finally {
      await leaveFileFlow();
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
    await enterFileFlow();
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
    } finally {
      await leaveFileFlow();
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
    if (!frame || frame.visits === 0) return;
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
    if (trialRef.current) {
      if (job.outcome === "started" && job.lane === "selected_node" && job.generation === trialRef.current.revision && samePath(job.node_path, trialRef.current.selected_path)) {
        selectedNodeJobRef.current = {
          run_id: job.run_id, job_id: job.job_id, lane: job.lane, mode: job.mode,
          state: "queued", generation: job.generation, node_path: job.node_path
        };
        setSelectedNodeRunning(true);
      } else if (selectedNodeJobRef.current?.job_id === job.job_id && job.outcome !== "progress") {
        if (["completed", "cancelled", "superseded", "timeout", "failed", "time_limited", "visits_limited"].includes(job.outcome)) {
          clearSelectedNodeRunning(job.job_id);
        }
      }
      return;
    }
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
      } else if ((job.outcome === "failed" || job.outcome === "timeout") && (phase === "queued" || phase === "searching" || phase === "error")) {
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
    if (matchOwnsWorkspace()) return;
    const snapshot = engineSnapshotRef.current;
    const run = runFromSnapshot(snapshot);
    if (!admitsForegroundEngineQuery(snapshot) || !admitsForegroundEngineJobs(snapshot, "visits_limit")) {
      setMessage("当前 run 不支持单点分析所请求的 ownership、policy 或 visits 限制。");
      return;
    }
    const active = trialRef.current;
    if (active) {
      const run = runFromSnapshot(engineSnapshotRef.current);
      if (!run) { setMessage("试下分析需要可用的前台引擎。"); return; }
      try {
        const started = await trialStartFinite(active.session_id, active.revision, run.run_id, resolveAnalysisMaxVisits(maxVisits, preferences));
        if (trialRef.current?.session_id !== active.session_id || trialRef.current.revision !== active.revision) return;
        selectedNodeJobRef.current = started;
        setSelectedNodeRunning(true);
        setMessage(`正在分析试下节点 (${started.job_id})。`);
      } catch (error) { setMessage(`试下分析失败: ${errorMessage(error)}`); }
      return;
    }
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
    if (matchOwnsWorkspace()) return;
    if (trialTransitionRef.current || scoringRef.current || scoringPendingRef.current) return;
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
    const continuousAdmitted = admitsForegroundEngineQuery(snapshot, "continuous_analysis")
      && (!committedPreferencesRef.current.continuousVisitsLimitEnabled || admitsForegroundEngineJobs(snapshot, "visits_limit"));
    if (!continuousAdmitted && snapshot.selected_node_job?.mode !== "continuous") {
      setMessage("当前 run 未验证连续分析或所请求的 ownership、policy、visits 限制能力；已保存的连续分析意图保持不变。");
      return;
    }
    if ((phase === "time_limited" || phase === "visits_limited" || phase === "paused" || phase === "error" || phase === "safety_hold")
      && (!continuousAdmitted || !currentGameRef.current)) {
      setMessage("继续连续分析需要可用的前台引擎和当前棋谱。");
      return;
    }

    continuousActionInFlightRef.current = true;
    setContinuousActionPending(true);
    try {
      const saved = { ...await foregroundEngineContinuousAction(), recentGamePaths: committedPreferencesRef.current.recentGamePaths, windowGeometry: committedPreferencesRef.current.windowGeometry, workspaceVisibility: committedPreferencesRef.current.workspaceVisibility };
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

  function handleAnalysisScopeDraftChange(incoming: AnalysisScopeDraft) {
    const current = analysisScopeDraft;
    let next = incoming;
    if (incoming.strategy !== current.strategy && incoming.strategy !== "single_stage") {
      const swing = incoming.strategy === "swing_selected_two_stage";
      next = {
        ...incoming,
        ...analysisTaskStageDraft(
          swing
            ? committedPreferencesRef.current.taskSwingOverviewConditions
            : committedPreferencesRef.current.taskOverviewConditions,
          swing
            ? committedPreferencesRef.current.taskSwingDeepConditions
            : committedPreferencesRef.current.taskDeepConditions
        ),
        ...(swing ? swingCriteriaDraft(committedPreferencesRef.current.taskSwingCriteria) : {})
      };
    }
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
      || next.leadingCandidateVisits !== current.leadingCandidateVisits
      || next.moveActors !== current.moveActors
      || next.winrateChangeEnabled !== current.winrateChangeEnabled
      || next.winrateChangeThreshold !== current.winrateChangeThreshold
      || next.scoreChangeEnabled !== current.scoreChangeEnabled
      || next.scoreChangeThreshold !== current.scoreChangeThreshold;
    if (conditionsChanged) analysisConditionsDraftEditedRef.current = true;
    const scopeChanged = next.strategy !== current.strategy
      || next.mode !== current.mode
      || next.intervalEnabled !== current.intervalEnabled
      || next.intervalStart !== current.intervalStart
      || next.intervalEnd !== current.intervalEnd
      || next.toPlay !== current.toPlay
      || next.moveActors !== current.moveActors
      || next.winrateChangeEnabled !== current.winrateChangeEnabled
      || next.winrateChangeThreshold !== current.winrateChangeThreshold
      || next.scoreChangeEnabled !== current.scoreChangeEnabled
      || next.scoreChangeThreshold !== current.scoreChangeThreshold;
    setAnalysisScopeDraft(next);
    if (scopeChanged) {
      ++analysisScopePreviewRequestRef.current;
      setAnalysisScopePreview(null);
    }
    setAnalysisTaskError(null);
  }

  async function handlePreviewAnalysisScope() {
    const game = currentGameRef.current;
    if (!nativeRuntime || !game) {
      setAnalysisTaskError(nativeRuntime ? "Analysis preview requires a current game." : nativeCurrentGameUnavailable);
      return;
    }
    if (!admitsForegroundEngineJobs(engineSnapshotRef.current, "whole_game_analysis")) {
      setAnalysisTaskError("当前 run 未验证整谱/task 分析能力。");
      return;
    }
    const request = ++analysisScopePreviewRequestRef.current;
    const draft = analysisScopeDraft;
    setAnalysisTaskRequestPending(true);
    setAnalysisTaskError(null);
    try {
      const swingCriteria = draft.strategy === "swing_selected_two_stage"
        ? swingCriteriaFromDraft(draft)
        : null;
      const criteriaError = swingCriteria ? swingCriteriaError(swingCriteria) : null;
      if (criteriaError) throw new Error(criteriaError);
      assertTaskCapabilities(undefined, swingCriteria);
      const preview = await previewAnalysisScope({
        generation: game.generation,
        scope: analysisScopeFromDraft(draft),
        swingCriteria
      });
      if (request === analysisScopePreviewRequestRef.current
        && admitsForegroundEngineJobs(engineSnapshotRef.current, "whole_game_analysis")) setAnalysisScopePreview(preview);
    } catch (error) {
      if (request === analysisScopePreviewRequestRef.current) setAnalysisTaskError(errorMessage(error));
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
    const expected = next.stage === "overview"
      ? next.requested.length + next.supporting.length
      : next.selected_for_deep?.length ?? next.requested.length;
    setWholeGameProgress({
      completed: stageCompleted,
      expected,
      remaining: Math.max(0, expected - stageCompleted)
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

  function assertTaskCapabilities(runId?: string, swingCriteria?: AnalysisSwingCriteriaDto | null, stages: AnalysisStageConditionsDto[] = []) {
    const snapshot = engineSnapshotRef.current;
    if (!admitsForegroundEngineJobs(snapshot, "whole_game_analysis")
      || (runId !== undefined && runFromSnapshot(snapshot)?.run_id !== runId)) {
      throw new Error("当前 run 未验证整谱/task 分析能力，或当前 run 已变化。");
    }
    if (stages.length > 0 && !admitsForegroundEngineQuery(snapshot, "whole_game_analysis")) {
      throw new Error("当前 run 不支持 task 请求的 ownership 或 policy。");
    }
    if (stages.some((stage) => stage.leading_candidate_visits.enabled)
      && !admitsForegroundEngineJobs(snapshot, "candidates")) throw new Error("当前 run 不支持候选 visits 条件。");
    if (stages.some((stage) => stage.total_visits.enabled || stage.leading_candidate_visits.enabled)
      && !admitsForegroundEngineJobs(snapshot, "visits_limit")) throw new Error("当前 run 不支持 visits 限制。");
    if (swingCriteria?.winrate_change_percentage_points.enabled && !admitsForegroundEngineJobs(snapshot, "winrate")) {
      throw new Error("当前 run 不支持胜率筛选。");
    }
    if (swingCriteria?.score_change_points.enabled && !admitsForegroundEngineJobs(snapshot, "root_score")) {
      throw new Error("当前 run 不支持分数筛选。");
    }
  }

  async function runAnalysisTask(input: {
    runId: string;
    scope: AnalysisScopeDto;
    strategy: AnalysisTaskStrategyDto;
    conditions: AnalysisStageConditionsDto;
    overviewConditions?: AnalysisStageConditionsDto | null;
    swingCriteria?: AnalysisSwingCriteriaDto | null;
    preview?: AnalysisScopePreviewDto | null;
    persistConditions?: boolean;
  }) {
    const game = currentGameRef.current;
    if (!nativeRuntime || !game) throw new Error("Analysis task requires a current game.");
    const stages = input.overviewConditions ? [input.conditions, input.overviewConditions] : [input.conditions];
    assertTaskCapabilities(input.runId, input.swingCriteria, stages);
    if (isAnalysisTaskReserved(analysisTaskRef.current)) {
      throw new Error("An analysis task is already active.");
    }
    const conditionsError = input.strategy === "all_positions_two_stage" && input.overviewConditions
      ? taskStageConditionsError(input.overviewConditions, input.conditions)
      : input.strategy === "swing_selected_two_stage" && input.overviewConditions && input.swingCriteria
        ? taskConditionsError(input.overviewConditions)
          ?? taskConditionsError(input.conditions)
          ?? swingCriteriaError(input.swingCriteria)
        : taskConditionsError(input.conditions);
    if (conditionsError) throw new Error(conditionsError);
    if (input.strategy !== "single_stage" && !input.overviewConditions) {
      throw new Error("Two-stage analysis requires overview conditions.");
    }
    if (input.strategy === "swing_selected_two_stage" && !input.swingCriteria) {
      throw new Error("Swing-selected analysis requires swing criteria.");
    }
    const preview = input.preview ?? await previewAnalysisScope({
      generation: game.generation,
      scope: input.scope,
      swingCriteria: input.swingCriteria
    });
    if (preview.generation !== game.generation
      || JSON.stringify(preview.scope) !== JSON.stringify(input.scope)
      || JSON.stringify(preview.swing_criteria ?? null) !== JSON.stringify(input.swingCriteria ?? null)) {
      throw new Error("The scope preview is no longer current. Preview again before starting.");
    }
    assertTaskCapabilities(input.runId, input.swingCriteria, stages);
    if (input.persistConditions) {
      const changed = input.strategy === "all_positions_two_stage"
        ? JSON.stringify(input.overviewConditions) !== JSON.stringify(committedPreferencesRef.current.taskOverviewConditions)
          || JSON.stringify(input.conditions) !== JSON.stringify(committedPreferencesRef.current.taskDeepConditions)
        : input.strategy === "swing_selected_two_stage"
          ? JSON.stringify(input.overviewConditions) !== JSON.stringify(committedPreferencesRef.current.taskSwingOverviewConditions)
            || JSON.stringify(input.conditions) !== JSON.stringify(committedPreferencesRef.current.taskSwingDeepConditions)
            || JSON.stringify(input.swingCriteria) !== JSON.stringify(committedPreferencesRef.current.taskSwingCriteria)
          : JSON.stringify(input.conditions) !== JSON.stringify(committedPreferencesRef.current.taskSingleStageConditions);
      if (changed) {
        if (preferencesSaveInFlightRef.current || pendingPreferencesSaveRef.current || continuousActionInFlightRef.current) {
          throw new Error("Wait for the current preference save before starting.");
        }
        const patch: Partial<AppPreferences> = input.strategy === "all_positions_two_stage"
          ? {
              taskOverviewConditions: input.overviewConditions!,
              taskDeepConditions: input.conditions
            }
          : input.strategy === "swing_selected_two_stage"
            ? {
                taskSwingOverviewConditions: input.overviewConditions!,
                taskSwingDeepConditions: input.conditions,
                taskSwingCriteria: input.swingCriteria!
              }
            : { taskSingleStageConditions: input.conditions };
        await new Promise<AppPreferences>((resolve, reject) => {
          const pending = queuePreferencesSave(committedPreferencesRef.current, patch);
          pending.onSaved = resolve;
          pending.onFailed = reject;
        });
      }
    }
    assertTaskCapabilities(input.runId, input.swingCriteria, stages);
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
    setMessage(`Analysis task ${started.state}: ${analysisTaskProgress(started)} completed.`);
    void refreshAnalysisTaskSnapshot();
  }

  async function handleStartAnalysisTask() {
    if (matchOwnsWorkspace()) return;
    if (trialRef.current || trialTransitionRef.current || scoringRef.current || scoringPendingRef.current) return;
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
      const overviewConditions = analysisScopeDraft.strategy !== "single_stage"
        ? taskConditionsFromDraft(analysisScopeDraft, "overview")
        : null;
      const swingCriteria = analysisScopeDraft.strategy === "swing_selected_two_stage"
        ? swingCriteriaFromDraft(analysisScopeDraft)
        : null;
      await runAnalysisTask({
        runId: run.run_id,
        scope,
        strategy: analysisScopeDraft.strategy,
        conditions,
        overviewConditions,
        swingCriteria,
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
    if (matchOwnsWorkspace()) return;
    if (trialRef.current || trialTransitionRef.current || scoringRef.current || scoringPendingRef.current) return;
    const run = runFromSnapshot(engineSnapshotRef.current);
    const game = currentGameRef.current;
    if (!run || !game || !admitsForegroundEngineJobs(engineSnapshotRef.current, "whole_game_analysis") || departurePendingRef.current || isAnalysisTaskReserved(analysisTaskRef.current)) return;
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
    if (matchOwnsWorkspace()) return;
    if (trialRef.current || trialTransitionRef.current || scoringRef.current || scoringPendingRef.current) return;
    const run = runFromSnapshot(engineSnapshotRef.current);
    const game = currentGameRef.current;
    if (!run || !game || !admitsForegroundEngineJobs(engineSnapshotRef.current, "whole_game_analysis") || departurePendingRef.current || isAnalysisTaskReserved(analysisTaskRef.current)) return;
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
      setMessage(`Analysis task ${paused.state}: ${analysisTaskProgress(paused)} completed.`);
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
    if (matchOwnsWorkspace()) return;
    if (trialRef.current || trialTransitionRef.current) return;
    const task = analysisTaskRef.current;
    const run = runFromSnapshot(engineSnapshotRef.current);
    if (analysisTaskActionInFlightRef.current
      || task?.state !== "paused"
      || !run
      || run.run_id !== task.run_id
      || !admitsForegroundEngineJobs(engineSnapshotRef.current, "whole_game_analysis")
      || departurePendingRef.current
      || departurePrompt) return;
    analysisTaskActionInFlightRef.current = true;
    setAnalysisTaskRequestPending(true);
    setAnalysisTaskError(null);
    ++analysisTaskSnapshotRequestRef.current;
    try {
      assertTaskCapabilities(task.run_id, task.swing_criteria, task.overview_conditions ? [task.conditions, task.overview_conditions] : [task.conditions]);
      const continued = await continueAnalysisTask({ runId: task.run_id, taskId: task.task_id });
      adoptAnalysisTask(continued);
      setMessage(`Analysis task ${continued.state}: ${analysisTaskProgress(continued)} completed.`);
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
    if (matchOwnsWorkspace()) return;
    const game = currentGameRef.current;
    if (!admitsForegroundEngineQuery(engineSnapshotRef.current, "whole_game_analysis")
      || !admitsForegroundEngineJobs(engineSnapshotRef.current, "visits_limit")
      || runFromSnapshot(engineSnapshotRef.current)?.run_id !== runId) {
      setMessage("当前 run 不支持整谱分析所请求的 ownership、policy 或 visits 限制。");
      return;
    }
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

  function handleImportPickerOpen() {
    if (importPickerActiveRef.current) return;
    importPickerActiveRef.current = true;
    void enterFileFlow().catch(async (error: unknown) => {
      importPickerActiveRef.current = false;
      setMessage(`Import failed: ${errorMessage(error)}`);
      await leaveFileFlow();
    });
  }

  async function finishImportPicker() {
    if (!importPickerActiveRef.current) return;
    importPickerActiveRef.current = false;
    await leaveFileFlow();
  }

  async function handleImportFile(file: File | null) {
    if (!file) {
      await finishImportPicker();
      return;
    }
    try {
      const imported = await importGameFile(file);
      await applyReplacement(imported.sgf_text, imported.native_path ?? null, {
        confirmMessage: "放弃未保存的棋谱并导入这个文件？",
        fallbackName: imported.format === "gib"
          ? imported.display_name.replace(/\.gib$/i, ".sgf")
          : imported.display_name,
        successMessage: (projection, fileName) => `Imported ${fileName}: ${projection.summary.move_count} moves.`,
        failurePrefix: "Import failed"
      });
    } catch (error) {
      setMessage(`Import failed: ${errorMessage(error)}`);
    } finally {
      await finishImportPicker();
    }
  }

  async function handleProviderImport(result: ProviderImportResult, networkIdentity?: ProviderRequestIdentity) {
    const source = providerSourceLabel(result);
    const warningText = result.warnings.length > 0 ? ` ${result.warnings.length} provider warning(s).` : "";
    const applied = await applyReplacement(result.sgf_text, null, {
      networkIdentity,
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
    if (externalBlocked) return;
    if (!preferencesLoaded || documentFlowBusy || newDocumentFlowReservedRef.current || departurePendingRef.current || departurePrompt) return;
    newDocumentFlowReservedRef.current = true;
    const admission = enterFileFlow();
    newDocumentAdmissionRef.current = admission;
    setNewDocumentOpen(true);
    try {
      await admission;
      newDocumentAdmissionRef.current = null;
    } catch (error) {
      setNewDocumentOpen(false);
      await releaseNewDocumentFlow();
      setMessage(`New game failed: ${errorMessage(error)}`);
    }
  }

  async function handleCancelNewGame() {
    if (newDocumentSubmittingRef.current) return;
    setNewDocumentOpen(false);
    await releaseNewDocumentFlow();
  }

  async function handleCreateNewGame(parameters: NewDocumentParameters) {
    if (!newDocumentFlowReservedRef.current || newDocumentSubmittingRef.current) return;
    newDocumentSubmittingRef.current = true;
    setNewDocumentOpen(false);
    try {
      const sgf = newDocumentSgf(parameters);
      await applyReplacement(sgf, null, {
        confirmMessage: "放弃未保存的棋谱并新建对局？",
        successMessage: () => `已新建 ${parameters.boardWidth} × ${parameters.boardHeight} 空谱。`,
        failurePrefix: "New game failed"
      });
    } finally {
      newDocumentSubmittingRef.current = false;
      await releaseNewDocumentFlow();
    }
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
    if (externalBlocked) return;
    if (matchOwnsWorkspace()) return;
    if (trialRef.current || trialTransitionRef.current || !currentGame || rootSetupDraft || conversionPrompt || editActionPendingRef.current) return;
    if (!nativeRuntime) {
      setMessage(nativeCurrentGameUnavailable);
      return;
    }
    if (comment === currentGame.snapshot.personal_comment) return;
    const editedPath = currentGame.selected_path;
    editActionPendingRef.current = true;
    setEditActionPending(true);
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
    } finally {
      editActionPendingRef.current = false;
      setEditActionPending(false);
    }
  }

  function soundAcceptedMove(before: SelectedNodeSnapshotDto, after: SelectedNodeSnapshotDto) {
    if (syncMuted(externalSyncRef.current)) return;
    if (!preferencesLoadSettledRef.current || !committedPreferencesRef.current.soundEnabled) return;
    const kind = acceptedMoveSound(before, after);
    if (kind) void playMoveSound(kind).catch((error) => setMessage(`声音播放失败: ${errorMessage(error)}`));
  }

  async function selectNode(path: NodePath, generation = reviewGame?.generation, forward = false) {
    if (matchOwnsWorkspace()) return;
    if (scoringRef.current || scoringPendingRef.current) return;
    const active = trialRef.current;
    if (active || trialTransitionRef.current) {
      if (!active || trialTransitionRef.current || editActionPendingRef.current || navigatingRef.current || generation !== active.revision || !nodeAt(active.tree, path) || samePath(active.selected_path, path)) return;
      navigatingRef.current = true;
      try {
        beginReviewRequest();
        const selected = await trialSelect(active.session_id, active.revision, path);
        if (trialRef.current?.session_id === active.session_id) {
          adoptTrial(selected);
          setChosenChildren((previous) => rememberChosenChildren(previous, selected.selected_path));
        }
      } catch (error) {
        setMessage(`试下导航失败: ${errorMessage(error)}`);
      } finally {
        navigatingRef.current = false;
      }
      return;
    }
    const game = currentGameRef.current;
    if (!game || rootSetupDraft || conversionPrompt || markupDialog || editActionPendingRef.current || generation !== game.generation || departurePendingRef.current
      || !path.indices.every((index) => Number.isInteger(index) && index >= 0)
      || !nodeAt(game.tree, path)) return;
    if (!navigatingRef.current && samePath(path, game.selected_path)) return;
    const request = { path: { indices: [...path.indices] }, generation, forward };
    queuedSelectionRef.current = request;
    pendingSelectedPathRef.current = request.path;
    beginReviewRequest();
    if (navigatingRef.current) return;

    navigatingRef.current = true;
    try {
      while (queuedSelectionRef.current) {
        const requested: { path: NodePath; generation: number; forward: boolean } = queuedSelectionRef.current;
        queuedSelectionRef.current = null;
        if (currentGameRef.current?.generation !== requested.generation) continue;
        try {
          const result = await selectCurrentGameNode(requested.path, requested.generation);
          const current = currentGameRef.current;
          if (!current || current.generation !== requested.generation
            || result.generation !== requested.generation || queuedSelectionRef.current) continue;
          if (result.snapshot_seq < current.snapshot_seq) {
            // Refresh the latest cursor against a newer accepted comment/Save snapshot.
            queuedSelectionRef.current = requested;
            continue;
          }
          adoptCurrentGame(result);
          if (requested.forward) soundAcceptedMove(current.snapshot, result.snapshot);
          setChosenChildren((previous) => rememberChosenChildren(previous, result.selected_path));
          if (result.snapshot.primary_analysis) presentCurrentGameAnalysis(result);
          else {
            const stored = wholeGameResultsRef.current.get(pathKey(result.selected_path));
            if (stored) presentWholeGameFrame(result.selected_path, stored);
          }
          setCurrentMove(result.snapshot.position.move_number);
          setSelectedCandidateIndex(null);
        } catch (error) {
          if (!queuedSelectionRef.current && currentGameRef.current?.generation === requested.generation) {
            setMessage(`导航失败: ${errorMessage(error)}`);
          }
        }
      }
    } finally {
      navigatingRef.current = false;
      pendingSelectedPathRef.current = currentGameRef.current?.selected_path ?? null;
    }
  }

  function handleMoveSelect(moveNumber: number, forward = false) {
    if (reviewGame) {
      if (!Number.isInteger(moveNumber) || moveNumber < 0) return;
      const target = moveNumber === 0
        ? chartModel.points[0]
        : chartModel.points.find((point) => point.isMove && point.moveNumber === moveNumber);
      if (target) void selectNode(target.path, reviewGame.generation, forward);
      return;
    }
    setCurrentMove(clampMoveNumberToPositions(positions, moveNumber));
    setSelectedCandidateIndex(null);
  }

  function handleReviewNodeSelect(path: NodePath) {
    if (!reviewGame || !chartModel.points.some((point) => samePath(point.path, path))) return;
    void selectNode(path, reviewGame.generation);
  }

  function handleProblemSelect(problem: ReviewProblem) {
    if (!reviewProblems.includes(problem)) return;
    if (reviewGame) {
      if (problem.path) void selectNode(problem.path, reviewGame.generation);
    } else handleMoveSelect(problem.turn);
  }

  function handleParent() {
    if (parentOfSelected) void selectNode(parentOfSelected);
  }

  function handleNextChild() {
    if (!selectedNode || selectedNode.children.length === 0) return;
    void selectNode(childPath(selectedPath, chosenChildIndex(chosenChildren, selectedPath, selectedNode.children.length)), reviewGame?.generation, true);
  }

  function handlePrevSibling() {
    if (siblingIndex === undefined || siblingIndex <= 0) return;
    void selectNode({ indices: [...selectedPath.indices.slice(0, -1), siblingIndex - 1] });
  }

  function handleNextSibling() {
    if (!parentNode || siblingIndex === undefined || siblingIndex + 1 >= parentNode.children.length) return;
    void selectNode({ indices: [...selectedPath.indices.slice(0, -1), siblingIndex + 1] });
  }

  function openRootSetup() {
    const game = currentGameRef.current;
    if (trialRef.current || trialTransitionRef.current || !game || !canRootSetup || documentFlowBusy || navigatingRef.current) return;
    setAutoPlaying(false);
    setRootSetupDraft({
      generation: game.generation,
      stones: game.snapshot.position.stones.map((stone) => ({ ...stone })),
      toPlay: game.snapshot.position.to_play,
      tool: "black"
    });
  }

  function promptPositionConversion() {
    const game = currentGameRef.current;
    if (trialRef.current || trialTransitionRef.current || !game || !canConvertPosition || documentFlowBusy || navigatingRef.current) return;
    setAutoPlaying(false);
    setConversionPrompt({ generation: game.generation, path: game.selected_path });
  }

  function placeSetupStone(point: PointDto) {
    setRootSetupDraft((draft) => {
      if (!draft || point.x < 0 || point.y < 0 || point.x >= currentPosition.board_width || point.y >= currentPosition.board_height) return draft;
      const stones = draft.stones.filter((stone) => stone.x !== point.x || stone.y !== point.y);
      if (draft.tool !== "erase") stones.push({ ...point, color: draft.tool });
      return { ...draft, stones };
    });
  }

  async function finishRootEdit(kind: "setup" | "convert") {
    const before = currentGameRef.current;
    const draft = rootSetupDraft;
    const prompt = conversionPrompt;
    if (trialRef.current || trialTransitionRef.current || !before || editActionPendingRef.current || navigatingRef.current
      || (kind === "setup" && (!draft || draft.generation !== before.generation || before.selected_path.indices.length !== 0))
      || (kind === "convert" && (!prompt || prompt.generation !== before.generation || !samePath(prompt.path, before.selected_path)))) return;
    editActionPendingRef.current = true;
    setEditActionPending(true);
    try {
      const result = kind === "setup"
        ? await applyRootSetup(draft!.generation, draft!.stones, draft!.toPlay)
        : await convertToRootSetup(prompt!.generation, prompt!.path);
      const latest = currentGameRef.current;
      if (!latest || latest.generation !== before.generation || latest.snapshot_seq > result.snapshot_seq) return;
      const changed = result.generation !== before.generation;
      adoptCurrentGame(result);
      pendingSelectedPathRef.current = result.selected_path;
      setChosenChildren(chosenFromPath(result.selected_path));
      setDirty(result.dirty);
      setCurrentMove(result.snapshot.position.move_number);
      setRootSetupDraft(null);
      setConversionPrompt(null);
      if (changed) {
        clearReviewData();
        await abandonAnalysisSessions();
        const artifacts = await artifactsFromCurrentGame();
        if (!isCurrentDocumentGeneration(result.generation)) return;
        setGame(artifacts.projection);
      }
      setMessage(kind === "setup" ? "起始局面已应用。" : "当前局面已转换为可编辑起始局面。");
    } catch (error) {
      setMessage(`${kind === "setup" ? "起始局面应用" : "局面转换"}失败: ${errorMessage(error)}`);
    } finally {
      editActionPendingRef.current = false;
      setEditActionPending(false);
    }
  }

  async function commitMarkup(action: SgfMarkupActionDto, captured?: { path: NodePath; generation: number }) {
    const before = currentGameRef.current;
    if (matchOwnsWorkspace()) return;
    if (trialRef.current || trialTransitionRef.current || !nativeRuntime || !before || rootSetupDraft || conversionPrompt || metadataDraft || newDocumentOpen || editActionPendingRef.current || departurePendingRef.current || fileFlowBusy) return;
    const path = captured?.path ?? before.selected_path;
    const generation = captured?.generation ?? before.generation;
    if (generation !== before.generation || !samePath(path, before.selected_path)) {
      setMessage("标记未提交：选中节点或棋谱已改变。");
      return;
    }
    editActionPendingRef.current = true;
    setEditActionPending(true);
    try {
      const result = await editCurrentGameMarkup(path, generation, action);
      if (!isCurrentGameSnapshot(result) || !samePath(currentGameRef.current?.selected_path ?? { indices: [] }, path)) return;
      adoptCurrentGame(result);
      setDirty(result.dirty);
      setMessage(result.snapshot_seq === before.snapshot_seq ? "标记未变化。" : "已更新当前节点标记。");
    } catch (error) {
      setMessage(`标记更新失败: ${errorMessage(error)}`);
    } finally {
      editActionPendingRef.current = false;
      setEditActionPending(false);
    }
  }

  function handleBoardPoint(point: PointDto) {
    if (externalBlocked) return;
    if (matchOwnsWorkspace()) { void handleHumanAction({ kind: "play", vertex: { point } }); return; }
    if (scoringRef.current || scoringPendingRef.current) {
      if (scoringRef.current) void handleUpdateScoring({ kind: "point", point });
      return;
    }
    if (trialRef.current || trialTransitionRef.current) {
      void playAt({ point });
      return;
    }
    if (rootSetupDraft) {
      placeSetupStone(point);
      return;
    }
    const before = currentGameRef.current;
    if (markupTool === "play") {
      void playAt({ point });
      return;
    }
    if (!before || !nativeRuntime || documentFlowBusy || editActionPendingRef.current) return;
    if (markupTool === "label") {
      const existing = before.snapshot.markup.find((mark) => mark.kind === "label" && mark.point.x === point.x && mark.point.y === point.y);
      setMarkupDialog({ point, path: before.selected_path, generation: before.generation, text: existing?.kind === "label" ? existing.text : "" });
      return;
    }
    void commitMarkup({ kind: "point", point, tool: { kind: markupTool } });
  }

  async function playAt(vertex: MoveVertex) {
    if (matchOwnsWorkspace()) { await handleHumanAction({ kind: "play", vertex }); return; }
    if (!nativeRuntime) {
      setMessage(nativeCurrentGameUnavailable);
      return;
    }
    if (!currentGame || scoringRef.current || scoringPendingRef.current || trialTransitionRef.current || documentFlowBusy || pendingBoardIntentRef.current !== null || editActionPendingRef.current) return;
    const intentKey = vertex === "pass" ? "pass" : `${vertex.point.x},${vertex.point.y}`;
    pendingBoardIntentRef.current = intentKey;
    editActionPendingRef.current = true;
    setEditActionPending(true);
    const previousGeneration = currentGame.generation;
    try {
      const active = trialRef.current;
      if (active) {
        const result = await trialPlay(active.session_id, active.revision, vertex);
        if (trialRef.current?.session_id === active.session_id) {
          adoptTrial(result);
          soundAcceptedMove(active.snapshot, result.snapshot);
          setChosenChildren((prev) => rememberChosenChildren(prev, result.selected_path));
          clearReviewData();
          setBoardIntentFeedback(null);
          setMessage("试下落子已接受（原谱未更改）。");
        }
        return;
      }
      const result = await playCurrentGame(currentGame.selected_path, vertex);
      adoptCurrentGame(result);
      pendingSelectedPathRef.current = result.selected_path;
      setChosenChildren((prev) => rememberChosenChildren(prev, result.selected_path));
      setDirty(result.dirty);
      setCurrentMove(result.snapshot.position.move_number);
      setSelectedCandidateIndex(null);
      setBoardIntentFeedback(null);
      setMessage("落子已接受。");
      soundAcceptedMove(currentGame.snapshot, result.snapshot);
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
      editActionPendingRef.current = false;
      setEditActionPending(false);
    }
  }

  async function handleRemoveVariation() {
    const before = currentGameRef.current;
    if (trialRef.current || trialTransitionRef.current || !nativeRuntime || !before || documentFlowBusy || editActionPendingRef.current
      || navigatingRef.current || queuedSelectionRef.current) return;
    const path = before.selected_path;
    if (path.indices.length === 0) {
      await handleNewGame();
      return;
    }
    const node = nodeAt(before.tree, path);
    if (!node || (node.children.length > 0 && !window.confirm("删除当前节点及其全部后续？此操作可以撤销。"))) return;
    editActionPendingRef.current = true;
    setEditActionPending(true);
    try {
      const result = await removeCurrentGameVariation(path, before.generation);
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
      setMessage("已删除当前节点及其后续，并回到父节点。");
    } catch (error) {
      setMessage(`删除节点失败: ${errorMessage(error)}`);
    } finally {
      editActionPendingRef.current = false;
      setEditActionPending(false);
    }
  }

  function handleReturnMain() {
    if (!canReturnMain) return;
    const firstVariation = selectedPath.indices.findIndex((index) => index !== 0);
    if (firstVariation >= 0) void selectNode({ indices: selectedPath.indices.slice(0, firstVariation) });
  }

  async function handlePromoteMain() {
    const before = currentGameRef.current;
    if (trialRef.current || trialTransitionRef.current || !nativeRuntime || !before || documentFlowBusy || editActionPendingRef.current
      || navigatingRef.current || queuedSelectionRef.current
      || !before.selected_path.indices.some((index) => index !== 0)) return;
    editActionPendingRef.current = true;
    setEditActionPending(true);
    try {
      const result = await promoteCurrentGameToMain(before.selected_path, before.generation);
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
      setMessage("已设为主分支。");
    } catch (error) {
      setMessage(`设为主分支失败: ${errorMessage(error)}`);
    } finally {
      editActionPendingRef.current = false;
      setEditActionPending(false);
    }
  }
  function openMetadataEditor(focusTarget: "game.komi" | "game.black-name" | "game.white-name" = "game.komi", returnToSearch = functionSearchOpen) {
    if (externalBlocked) return;
    const game = currentGameRef.current;
    if (trialRef.current || trialTransitionRef.current || !nativeRuntime || !game || documentFlowBusy || editActionPendingRef.current || navigatingRef.current) return;
    const value = (key: string) => game.tree.properties.find((property) => property.key === key)?.values[0];
    if (workspaceRef.current) metadataSourceRef.current = returnToSearch ? searchSourceRef.current : captureFocusReturn(workspaceRef.current);
    setSettingsTarget(null);
    setMetadataDraft({
      generation: game.generation,
      focusTarget, returnToSearch,
      blackName: value("PB") ?? "",
      whiteName: value("PW") ?? "",
      komi: Number.isFinite(Number(value("KM"))) && value("KM")?.trim() ? Number(value("KM")) : 7.5,
      handicap: value("HA") ?? null
    });
  }

  function cancelMetadataEditor() {
    const returnToSearch = metadataDraft?.returnToSearch;
    setMetadataDraft(null);
    if (returnToSearch) setFunctionSearchOpen(true);
    else if (metadataSourceRef.current) restoreOwnedFocus(metadataSourceRef.current);
  }

  async function handleApplyMetadata(generation: number, black: string, white: string, komi: number) {
    const before = currentGameRef.current;
    if (trialRef.current || trialTransitionRef.current || !nativeRuntime || !before || before.generation !== generation || departurePendingRef.current
      || editActionPendingRef.current || navigatingRef.current) {
      throw new Error("棋谱已变化，请重新打开棋局信息。");
    }
    editActionPendingRef.current = true;
    setEditActionPending(true);
    try {
      let result = await setCurrentGameMetadata(generation, black, white, komi);
      const latest = currentGameRef.current;
      if (!latest || latest.generation !== generation) throw new Error("棋谱已变化，请重新打开棋局信息。");
      if (latest.snapshot_seq > result.snapshot_seq) {
        result = await selectCurrentGameNode(latest.selected_path, result.generation);
        if (currentGameRef.current?.generation !== generation) throw new Error("棋谱已变化，请重新打开棋局信息。");
      }
      adoptCurrentGame(result);
      setDirty(result.dirty);
      if (result.generation !== generation) {
        clearReviewData();
        await abandonAnalysisSessions();
      }
      setMetadataDraft(null);
      if (metadataSourceRef.current) restoreOwnedFocus(metadataSourceRef.current);
      const projection = await projectCurrentGameMainline();
      if (isCurrentDocumentGeneration(result.generation)) setGame(projection);
      setMessage("已更新棋局信息。");
    } catch (error) {
      setMessage(`棋局信息更新失败: ${errorMessage(error)}`);
      throw error;
    } finally {
      editActionPendingRef.current = false;
      setEditActionPending(false);
    }
  }


  async function handleHistoryAction(action: "undo" | "redo") {
    if (scoringRef.current || scoringPendingRef.current) return;
    const active = trialRef.current;
    if (active || trialTransitionRef.current) {
      if (!active || action !== "undo" || !active.can_undo || editActionPendingRef.current || trialTransitionRef.current) return;
      editActionPendingRef.current = true;
      setEditActionPending(true);
      try {
        const result = await trialUndo(active.session_id, active.revision);
        if (trialRef.current?.session_id === active.session_id) {
          adoptTrial(result);
          setChosenChildren(chosenFromPath(result.selected_path));
          clearReviewData();
          setMessage("已撤销试下的一手（原谱未更改）。");
        }
      } catch (error) {
        setMessage(`试下撤销失败: ${errorMessage(error)}`);
      } finally {
        editActionPendingRef.current = false;
        setEditActionPending(false);
      }
      return;
    }
    const before = currentGameRef.current;
    if (!nativeRuntime || !before || documentFlowBusy || editActionPendingRef.current
      || navigatingRef.current || queuedSelectionRef.current) return;
    if (action === "undo" ? !before.can_undo : !before.can_redo) return;
    editActionPendingRef.current = true;
    setEditActionPending(true);
    try {
      const result = await (action === "undo" ? undoCurrentGame(before.generation) : redoCurrentGame(before.generation));
      const latest = currentGameRef.current;
      if (!latest || latest.generation !== before.generation || latest.snapshot_seq > result.snapshot_seq) return;
      const changed = result.generation !== before.generation || result.snapshot_seq !== before.snapshot_seq;
      const structural = result.generation !== before.generation;
      adoptCurrentGame(result);
      pendingSelectedPathRef.current = result.selected_path;
      setChosenChildren(chosenFromPath(result.selected_path));
      setDirty(result.dirty);
      setCurrentMove(result.snapshot.position.move_number);
      setSelectedCandidateIndex(null);
      if (structural) {
        clearReviewData();
        await abandonAnalysisSessions();
        const artifacts = await artifactsFromCurrentGame();
        if (!isCurrentDocumentGeneration(result.generation)) return;
        setGame(artifacts.projection);
      }
      setMessage(changed
        ? (action === "undo" ? "已撤销上一次编辑。" : "已重做上一次编辑。")
        : (action === "undo" ? "没有可撤销的编辑。" : "没有可重做的编辑。"));
    } catch (error) {
      setMessage(`${action === "undo" ? "撤销" : "重做"}失败: ${errorMessage(error)}`);
    } finally {
      editActionPendingRef.current = false;
      setEditActionPending(false);
    }
  }

  function previewCandidate(index: number | null) {
    setCandidatePreview(index === null ? null : { index, scope: activeScope, matchFrameIdentity });
    // Java FloatBoard: browsing a candidate variation lets readboard give focus back (`loss`), never per frame.
    const sync = externalSyncRef.current;
    if (index !== null && sync?.source === "readboard" && sync.phase === "syncing" && sync.readboard?.preferences.focus) {
      void requestReadboardFocus().catch((error) => setMessage(`readboard 焦点请求失败: ${errorMessage(error)}`));
    }
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

  return <main ref={workspaceRef} tabIndex={-1} data-focus-owner="workspace" className={`app-shell${preferences.boardTheme === "high-contrast" ? " theme-high-contrast" : ""}${nativeRuntime ? "" : " has-native-runtime-note"}`}>
    {!nativeRuntime ? <p className="native-runtime-note" role="status">{nativeCurrentGameUnavailable}</p> : null}
    <AppChrome
      onBeforeFunctionSearch={() => { if (workspaceRef.current) chromeSearchSourceRef.current = captureFocusReturn(workspaceRef.current); }}
      onFunctionSearch={() => openFunctionSearch(true)}
      windowPin={windowPin}
      saveBusy={fileFlowBusy || departurePending || matchStarting || Boolean(departurePrompt) || Boolean(teardownPrompt)}
      matchBlocked={matchBlocked}
      humanTurn={humanTurn}
      onHumanNew={() => openMatchDialog(false)}
      onHumanContinue={() => openMatchDialog(true)}
      onPkNew={() => openMatchDialog(false, "pk")}
      onPkContinue={() => openMatchDialog(true, "pk")}
      sheet={sheet}
      onToggleSheet={toggleSheet}
      onOpenProvider={(provider) => { if (!matchOwnsWorkspace()) { setProviderEntry(provider); setSheet("sync"); } }}
      busy={historyActionBlocked}
      trialActive={Boolean(trial)}
      trialPending={trialPending || Boolean(scoring) || scoringPending}
      onToggleTrial={() => void handleToggleTrial()}
      scoringActive={Boolean(scoring)}
      scoringPending={scoringPending}
      onEnterScoring={() => void handleEnterScoring()}
      dirty={documentDirty}
      documentName={documentName}
      engineLabel={engineLabel}
      engineReady={engineReady}
      taskEngineReady={taskEngineReady}
      engineCapabilities={verifiedEngineCapabilitiesLabel(engineSnapshot)}
      engineSwitcher={{
        profiles: engineProfiles.map((profile) => ({ id: profile.id, name: profile.profile.name })),
        selectedProfileId: runFromSnapshot(engineSnapshot)?.profile_id
          ?? (engineSnapshot.lifecycle.state === "no_engine" ? "" : ""),
        canStop: !matchBlocked && canStopForegroundEngine(engineSnapshot),
        canRestart: !matchBlocked && canRestartForegroundEngine(engineSnapshot),
        failureMessage: visibleEngineFailure?.message ?? null,
        failureKind: visibleEngineFailure?.kind ?? null,
        failureOperation: visibleEngineFailure?.operation ?? null,
        onSelectProfile: (profileId) => void handleSelectSwitcherProfile(profileId),
        onStop: () => {
          if (matchOwnsWorkspace()) return;
          void stopForegroundEngine().catch((error) => {
            setMessage(error instanceof Error ? error.message : String(error));
          });
        },
        onRestart: () => {
          if (matchOwnsWorkspace()) return;
          void restartForegroundEngine().catch((error) => {
            setMessage(errorMessage(error));
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
      onShowCoordinates={(value) => handlePreferencesChange({ ...preferences, showCoordinates: value })}
      onShowMoveNumbers={(value) => handlePreferencesChange({ ...preferences, showMoveNumbers: value })}
      showBlackCandidates={showBlackCandidates}
      showWhiteCandidates={showWhiteCandidates}
      onShowBlackCandidates={setShowBlackCandidates}
      onShowWhiteCandidates={setShowWhiteCandidates}
      onRestoreWorkspace={() => { if (!matchOwnsWorkspace()) workspace.restoreDefaults(); }}
      workspaceDisabled={workspace.disabled || matchBlocked}
      windowGeometryStatus={windowGeometryStatusText}
      windowGeometryCanRetry={Boolean(windowGeometryFailure) || windowGeometry.phase === "unsaved"}
      windowGeometryDisabled={windowGeometryDisabled}
      onResetWindowGeometry={() => void performWindowGeometryAction(resetWindowGeometry)}
      onRetryWindowGeometry={() => void performWindowGeometryAction(retryWindowGeometry)}
      railVisibilityDisabled={matchBlocked || !preferencesLoaded || railVisibilityBusy || departurePending || Boolean(departurePrompt) || workspace.frozen}
      onRailVisibility={(side, visible) => void handleRailVisibility(side, visible)}
      selectedNodeRunning={selectedNodeRunning}
      wholeGameRunning={wholeGameRunning}
      autoPlaying={autoPlaying}
      komi={game.summary.komi}
      onNew={() => void handleNewGame()}
      onRootSetup={openRootSetup}
      onConvertPosition={promptPositionConversion}
      canRootSetup={canRootSetup && !documentFlowBusy}
      canConvertPosition={canConvertPosition && !documentFlowBusy}
      onEditMetadata={openMetadataEditor}
      canEditMetadata={nativeRuntime && Boolean(currentGame) && !externalBlocked && !trial && !trialPending && !scoring && !scoringPending && !documentFlowBusy && !editActionPending}
      onOpen={() => void handleOpenSgfDocument()}
      recentGamePaths={preferences.recentGamePaths}
      recentHistoryBusy={recentHistoryBusy || !preferencesLoaded}
      recentHistoryError={recentHistoryError}
      onOpenRecent={(index) => void handleOpenRecent(index)}
      onClearRecentHistory={() => void handleClearRecentHistory()}
      onRetryRecentHistory={() => void handleRetryRecentHistory()}
      nativeRuntime={nativeRuntime}
      nativeUnavailable={nativeCurrentGameUnavailable}
      onSave={() => void handleSaveSgfDocument(false)}
      onSaveAs={() => void handleSaveSgfDocument(true)}
      onLoadSample={() => void loadSample()}
      onParse={() => void handleParseSgf()}
      onFakeAnalyze={() => void handleFakeAnalyze()}
      onCancelSelectedNode={() => void handleCancelSelectedNodeAnalysis()}
      onCancelWholeGame={() => void handleCancelWholeGameAnalysis()}
      onAbout={openAbout}
      onOpenShortcutReference={() => setShortcutReferenceOpen(true)}
      onCopySgf={() => void handleCopySgf()}
      onPasteSgf={() => void handlePasteSgf()}
      onExit={() => void handleApplicationExit()}
      onUndo={() => void handleHistoryAction("undo")}
      onRedo={() => void handleHistoryAction("redo")}
      canUndo={canUndo}
      canRedo={canRedo}
      onClearBoard={() => void handleNewGame()}
      onPass={() => void playAt("pass")}
      onRemoveVariation={() => void handleRemoveVariation()}
      canRemoveVariation={canRemoveVariation && !documentFlowBusy}
      canDeleteNode={canDeleteNode}
      canPromoteMain={canPromoteMain}
      canReturnMain={canReturnMain}
      onPromoteMain={() => void handlePromoteMain()}
      onReturnMain={handleReturnMain}
      onFirstMove={() => handleMoveSelect(0)}
      onAutoPlay={() => { if (!matchOwnsWorkspace() && !scoringRef.current && !scoringPendingRef.current) setAutoPlaying((value) => !value); }}
      onOverlayMode={setOverlayMode}
      message={recentHistoryError ? `${message} ${recentHistoryError}` : message}
      toPlay={currentPosition.to_play}
    />
    {matchDialogOpen ? <HumanMatchDialog mode={matchDialogMode} defaults={preferences.matchDefaults} profiles={engineProfiles} engineSnapshot={engineSnapshot} pending={matchBlocked}
      continuation={matchContinuation?.snapshot.position ?? null} continuationRoot={matchContinuation?.tree}
      error={matchStartError ?? (matchState?.failure ? `${matchState.failure.kind} · 配置 ${matchState.failure.profile_id ?? "—"} · ${matchState.failed_side ?? "—"} · ${matchState.failure.message}` : null)}
      onStart={(settings) => void handleStartMatch(settings, matchContinuation, matchDialogMode)} onCancel={() => {
        if (matchOwnsWorkspace()) void handleStopMatch();
        else setMatchDialogOpen(false);
      }} /> : null}
    <div className="workspace-visibility-controls" role="toolbar" aria-label="侧栏显隐">
    {matchState?.session_id || matchStarting ? <div className="human-match-status" role="region" aria-label="对局状态">
      <span role="status">{(matchStarting ? matchDialogMode : matchState?.mode) === "pk" ? "PK" : "人机"} · {matchStarting ? "starting" : matchState?.phase} · 轮到 {matchState?.to_play === "white" ? "白" : matchState?.to_play === "black" ? "黑" : "—"} · 本场提交 {matchState?.committed_moves ?? 0}{matchState?.mode === "pk" ? ` / ${matchState.settings?.pk_max_moves ?? "—"}` : ""}</span>
      {matchState?.mode === "human" ? <span>人类 {matchState.settings?.human_color === "white" ? "白" : "黑"}</span> : null}
      <span>占用：{matchState?.resources_held ? "是" : "否"}{matchState?.pause_pending ? " · 暂停清理中，暂不可恢复" : ""}{matchState?.resume_pending ? " · 正在按原始配置快照重建 GTP 引擎" : ""}</span>
      {matchState?.phase === "paused" && matchState.rebuild_sides.length > 0 && !matchState.resume_pending
        ? <span role="status">{matchState.rebuild_sides.map((side) => side === "white" ? "白方" : "黑方").join("、")} GTP 取步已取消并回收进程；点击恢复将按本场原始配置快照重建新 run，暂停期间的配置修改不生效。</span> : null}
      <span>Session: {matchState?.session_id ?? "—"} · Run: {matchState?.run_id ?? "—"} · Job: {matchState?.job?.job_id ?? "—"}</span>
      {matchState?.pk_runs?.map((run, side) => <span key={run.run_id}>{side === 0 ? "黑方" : "白方"}：{run.profile_snapshot.name}{run.adapter_kind === "generic_gtp" ? "（GTP，仅硬截止）" : ""} · 配置 {run.profile_id} · Run {run.run_id}</span>)}
      {matchState?.end ? <span>结束：{matchState.end}</span> : null}
      {matchState?.failure ? <span role="alert">{matchState.failure.kind} · 配置 {matchState.failure.profile_id ?? "—"} · {matchState.failed_side ?? "—"} · {matchState.failure.message}</span> : null}
      {matchState?.mode === "pk" ? <>
        <button type="button" className="chrome-btn" disabled={matchState.phase !== "playing" || matchState.pause_pending || matchState.resume_pending || Boolean(matchControlPending) || matchStopPending} onClick={() => void handlePkControl("pause")}>暂停 PK</button>
        <button type="button" className="chrome-btn" disabled={matchState.phase !== "paused" || matchState.pause_pending || matchState.resume_pending || Boolean(matchControlPending) || matchStopPending} onClick={() => void handlePkControl("resume")}>恢复 PK</button>
      </> : matchState?.mode === "human" ? <>
        <label>
          <input type="checkbox" aria-label="本场候选分析" aria-describedby="human-match-analysis-hint"
            checked={Boolean(matchState.analysis.policy && matchState.analysis.policy !== "off")}
            disabled={matchState.phase !== "playing" || !matchState.analysis.supported || matchAnalysisPending || matchStopPending}
            onChange={(event) => void handleMatchAnalysisPolicy(event.target.checked ? "both" : "off")} />
          本场候选分析
        </label>
        <label>分析回合
          <select aria-label="本场分析回合" aria-describedby="human-match-analysis-hint"
            value={matchState.analysis.policy === "off" ? "both" : matchState.analysis.policy}
            disabled={matchState.phase !== "playing" || !matchState.analysis.supported || matchState.analysis.policy === "off" || matchAnalysisPending || matchStopPending}
            onChange={(event) => {
              const policy = event.target.value;
              if (policy === "human_turn" || policy === "engine_turn" || policy === "both") void handleMatchAnalysisPolicy(policy);
            }}>
            <option value="human_turn">人类回合</option>
            <option value="engine_turn">引擎回合</option>
            <option value="both">双方回合</option>
          </select>
        </label>
        <span id="human-match-analysis-hint">{matchState.analysis.supported
          ? "仅本场有效，不改变普通复盘偏好。引擎正在取步时更改策略，分析从下一符合策略的回合生效；行棋角色与预算不变。"
          : "本场未验证候选分析能力；已合格 GTP 仅支持取步，不提供候选、visits 或 ownership 分析。"}</span>
        <button type="button" className="chrome-btn" disabled={!humanTurn} onClick={() => void handleHumanAction({ kind: "play", vertex: "pass" })}>停一手</button>
        <button type="button" className="chrome-btn" disabled={!humanTurn} onClick={() => void handleHumanAction({ kind: "resign" })}>认输</button>
      </> : null}
      <button type="button" className="chrome-btn" disabled={!matchBlocked || matchStopPending} onClick={() => void handleStopMatch()}>停止</button>
    </div> : null}
      <button ref={railRestoreRef} type="button" aria-pressed={preferences.workspaceVisibility.left}
        disabled={matchBlocked || !preferencesLoaded || departurePending || Boolean(departurePrompt) || workspace.frozen}
        onClick={() => void handleRailVisibility("left", !preferences.workspaceVisibility.left)}>左侧栏</button>
      <button type="button" aria-pressed={preferences.workspaceVisibility.right}
        disabled={matchBlocked || !preferencesLoaded || departurePending || Boolean(departurePrompt) || workspace.frozen}
        onClick={() => void handleRailVisibility("right", !preferences.workspaceVisibility.right)}>右侧栏</button>
      <details aria-label="当前 run 能力"><summary>当前 run 能力</summary><p>{verifiedEngineCapabilitiesLabel(engineSnapshot)}</p></details>
      {railVisibilityBusy ? <span role="status">正在保存侧栏…</span> : null}
      {railVisibilityError ? <span role="alert">{railVisibilityError}</span> : null}
    </div>
    <Workspace shares={workspace.shares} onSharesChange={(shares) => { if (!matchOwnsWorkspace()) workspace.setShares(shares); }} disabled={workspace.disabled || matchBlocked}
      visibility={preferences.workspaceVisibility}>
      <aside ref={leftRailRef} className="rail" hidden={!preferences.workspaceVisibility.left}>
        <div className="rail-block">
          <h2>
            <span>胜率走势 ({chartTitleSide})</span>
            <span style={{ color: "#60a5fa", fontFamily: "var(--mono)" }}>
              {`${((chartWinrate ?? 0.5) * 100).toFixed(1)}%`}
            </span>
          </h2>
          <WinrateChart model={preferences.workspaceVisibility.left ? chartModel : { ...chartModel, hoverEnabled: false }} onSelectNode={(path) => { if (preferences.workspaceVisibility.left) handleReviewNodeSelect(path); }} />
          <div id="board-layers" />
        </div>
        <AnalysisPanel
          pane="commentary"
          frame={visibleCurrentFrame}
          problems={reviewProblems}
          boardWidth={game.summary.board_width}
          boardHeight={game.summary.board_height}
          currentMove={currentMove}
          blackName={blackName}
          whiteName={whiteName}
          currentPosition={currentPosition}
          personalComment={selectedPersonalComment}
          generatedInformation={selectedGeneratedInformation}
          commentEditorEnabled={nativeRuntime && Boolean(currentGame) && !externalBlocked && !documentFlowBusy && !trial && !trialPending && !scoring && !scoringPending}
          onCommitPersonalComment={(comment) => void handleCommitPersonalComment(comment)}
          selectedCandidateIndex={selectedCandidateIndex}
          onSelectCandidate={(index) => { if (preferences.workspaceVisibility.left) selectCandidate(index); }}
          onSelectProblem={(problem) => { if (preferences.workspaceVisibility.left) handleProblemSelect(problem); }}
          selectedPath={selectedPath}
        />
      </aside>
      <div className={`diagram${rootSetupDraft ? " setup-active" : ""}${scoring ? " scoring-active" : ""}`}>
      <div className="markup-toolbar" role="toolbar" aria-label="节点标记工具">
        {([
          ["play", "落子"], ["label", "文字"], ["letters", "字母"], ["numbers", "数字"],
          ["circle", "圆"], ["square", "方"], ["cross", "叉"], ["triangle", "三角"], ["erase", "擦除"]
        ] as const).map(([tool, label]) => <button key={tool} type="button" aria-pressed={markupTool === tool}
          disabled={!nativeRuntime || !currentGame || historyActionBlocked || trialPending || scoringPending || Boolean(scoring) || (tool !== "play" && Boolean(trial))} onClick={() => setMarkupTool(tool)}>{label}</button>)}
        <button type="button" disabled={!nativeRuntime || !currentGame || historyActionBlocked || trialPending || scoringPending || Boolean(scoring) || Boolean(trial)}
          onClick={() => void commitMarkup({ kind: "clear" })}>清空标记</button>
      </div>
        <BoardCanvas
          position={rootSetupDraft ? { ...currentPosition, stones: rootSetupDraft.stones, to_play: rootSetupDraft.toPlay, last_move: null, move_number: 0 } : currentPosition}
          markup={reviewGame?.snapshot.markup}
          analysis={rootSetupDraft || scoring ? undefined : visibleCurrentFrame}
          scoring={scoring}
          selectedCandidateIndex={selectedCandidateIndex}
          previewScope={activeScope}
          onCandidatePreview={previewCandidate}
          stoneMoveNumbers={rootSetupDraft ? [] : reviewGame?.snapshot.stone_move_numbers}
          showCoordinates={showCoordinates}
          showMoveNumbers={showMoveNumbers}
          overlayMode={overlayMode}
          onOverlayModeChange={setOverlayMode}
          hideCandidates={hideCandidates}
          pvPrefixLength={replayPrefix}
          replayCandidateIndex={activeCandidateIndex}
          onPointClick={handleBoardPoint}
          keyboardPlacement={keyboardPlacement}
          nextMoveMode={rootSetupDraft || scoring ? "off" : preferences.nextMoveReviewMarker}
          nextMoveMarkers={rootSetupDraft || scoring ? [] : nextMoveMarkers}
        />
        {rootSetupDraft ? <div className="root-setup-tools" role="group" aria-label="起始局面草稿">
          <span>起始局面 · 点击棋盘预览</span>
          {(["black", "white", "erase"] as const).map((tool) => <button key={tool} type="button"
            aria-pressed={rootSetupDraft.tool === tool} onClick={() => setRootSetupDraft({ ...rootSetupDraft, tool })}>
            {tool === "black" ? "黑子" : tool === "white" ? "白子" : "擦除"}
          </button>)}
          <button type="button" onClick={() => setRootSetupDraft({ ...rootSetupDraft, stones: [] })}>清空</button>
          <label>下一手 <select value={rootSetupDraft.toPlay} onChange={(event) => setRootSetupDraft({ ...rootSetupDraft, toPlay: event.target.value as PlayerColor })}>
            <option value="black">黑</option><option value="white">白</option>
          </select></label>
          <button type="button" disabled={editActionPending} onClick={() => void finishRootEdit("setup")}>应用</button>
          <button type="button" disabled={editActionPending} onClick={() => setRootSetupDraft(null)}>取消</button>
        </div> : null}
        {scoring ? <ScoringControls scoring={scoring} busy={scoringPending} onRule={handleScoringRule}
          onSettings={(compensation, handicap) => void handleUpdateScoring({ kind: "settings", rule: scoring.rule, compensation, handicap })}
          onExit={(confirm) => void handleExitScoring(confirm)} /> : null}
        {boardIntentFeedback ? (
          <p className="board-intent-status" role="status" aria-live="polite">{boardIntentFeedback}</p>
        ) : null}
      </div>
      <aside ref={rightRailRef} className="sheet-col" hidden={!preferences.workspaceVisibility.right}>
        {reviewGame ? <ReviewTree
          key={trial ? `trial-${trial.session_id}` : reviewGame.generation}
          root={reviewGame.tree}
          selectedPath={selectedPath}
          generation={reviewGame.generation}
          onSelectNode={(path, generation) => {
            if (preferences.workspaceVisibility.right) void selectNode(path, generation);
          }}
        /> : null}
        <AnalysisPanel
          pane="reference"
          frame={visibleCurrentFrame}
          problems={reviewProblems}
          boardWidth={game.summary.board_width}
          boardHeight={game.summary.board_height}
          currentMove={currentMove}
          currentPosition={currentPosition}
          selectedCandidateIndex={selectedCandidateIndex}
          previewCandidateIndex={previewCandidateIndex}
          onSelectCandidate={(index) => { if (preferences.workspaceVisibility.right) selectCandidate(index); }}
          onSelectProblem={(problem) => { if (preferences.workspaceVisibility.right) handleProblemSelect(problem); }}
          selectedPath={selectedPath}
          reviewLine={reviewGame ? chartModel.points : undefined}
          onSelectNode={(path) => { if (preferences.workspaceVisibility.right) handleReviewNodeSelect(path); }}
          contentMode={preferences.subBoardContentMode}
          pvPrefixLength={replayPrefix}
        />
      </aside>
    </Workspace>
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
      canRun={nativeRuntime && taskEngineReady && Boolean(currentGame) && !trial && !trialPending && !scoring && !scoringPending && !departurePending && !departurePrompt}
      busy={matchBlocked || analysisTaskRequestPending}
      error={analysisTaskError}
      onDraftChange={handleAnalysisScopeDraftChange}
      onPreview={() => void handlePreviewAnalysisScope()}
      onStart={() => void handleStartAnalysisTask()}
      onPause={() => void handlePauseAnalysisTask()}
      onContinue={() => void handleContinueAnalysisTask()}
      onCancel={() => void handleCancelWholeGameAnalysis()}
    />
    <div className="workspace-save-status" role="status" aria-label="面板尺寸保存状态">
      {workspace.status === "saved" ? "面板尺寸已保存" : workspace.status === "pending" ? "面板尺寸待保存" : workspace.status === "saving" ? "正在保存面板尺寸…" : `面板尺寸未保存：${workspace.error}`}
      {workspace.status === "unsaved" ? <button type="button" onClick={() => void workspace.owner.retry()}>重试保存面板尺寸</button> : null}
    </div>
    </div>
    <BottomBar
      matchBlocked={matchBlocked}
      currentMove={reviewIndex}
      maxMove={reviewMax}
      onMove={handleMoveSelect}
      canParent={!matchBlocked && canParent}
      canNext={!matchBlocked && canNext}
      canPrevSibling={!matchBlocked && canPrevSibling}
      canNextSibling={!matchBlocked && canNextSibling}
      canRemoveVariation={!matchBlocked && canRemoveVariation}
      siblingLabel={siblingLabel}
      nativeUnavailable={nativeRuntime ? undefined : nativeCurrentGameUnavailable}
      onParent={handleParent}
      onNext={handleNextChild}
      onPrevSibling={handlePrevSibling}
      onNextSibling={handleNextSibling}
      onRemoveVariation={() => void handleRemoveVariation()}
      engineReady={engineReady}
      taskEngineReady={taskEngineReady}
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
      onAutoPlay={() => { if (!matchOwnsWorkspace() && !scoringRef.current && !scoringPendingRef.current) setAutoPlaying((value) => !value); }}
      autoPlaying={autoPlaying}
      showCoordinates={showCoordinates}
      showMoveNumbers={showMoveNumbers}
      onShowCoordinates={(value) => handlePreferencesChange({ ...preferences, showCoordinates: value })}
      onShowMoveNumbers={(value) => handlePreferencesChange({ ...preferences, showMoveNumbers: value })}
      keyboardPlacement={keyboardPlacement}
      onKeyboardPlacement={setKeyboardPlacement}
      jumpRef={jumpRef}
      message={[message, recentHistoryError, preferencesStatus.startsWith("Save failed:") ? preferencesStatus : null].filter(Boolean).join(" ")}
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
            <input
              ref={importFileInputRef}
              type="file"
              accept=".sgf,.txt,.gib,application/x-go-sgf,text/plain,application/octet-stream"
              disabled={matchBlocked}
              onClick={handleImportPickerOpen}
              onChange={(event) => void handleImportFile(event.target.files?.[0] ?? null)}
            />
          </label>
          <button type="button" onClick={() => void loadSample()} disabled={matchBlocked}>载入示例</button>
        </div>
      </div> : null}
      {sheet === "sync" ? <>
        <YikeSyncPanel snapshot={syncView(externalSync, "yike")} disabled={matchBlocked || Boolean(trial) || Boolean(scoring) || departurePending || syncStarting === "readboard"} starting={syncStarting === "yike"}
          onStart={handleStartSync} onCancelStart={handleCancelSyncStart} onRetry={() => handleSyncAction("retry")}
          onStop={() => handleSyncAction("stop")} onOpenBrowser={() => handleSyncAction("browser")} />
        <ReadboardSyncPanel snapshot={syncView(externalSync, "readboard")} disabled={matchBlocked || Boolean(trial) || Boolean(scoring) || departurePending || syncStarting === "yike"} starting={syncStarting === "readboard"}
          onStart={handleStartReadboardSync} onCancelStart={handleCancelSyncStart} onRetry={() => handleSyncAction("retry")}
          onStop={() => handleSyncAction("stop")} />
        <ProviderPanel key={`${externalSync?.session_id != null ? `sync:${externalSync.document_identity}` : `local:${currentGame?.generation ?? 0}`}:${providerEntry}`} initialProvider={providerEntry} disabled={matchBlocked || syncStarting !== null} onImport={handleProviderImport} onStartSync={handleStartSync} />
      </> : null}
      <div hidden={sheet !== "engine"}>
        <EngineSetupPanel
          disabled={matchBlocked}
          engineSnapshot={engineSnapshot}
          onProfilesChange={setEngineProfiles}
        />
      </div>
      {sheet === "prefs" ? <PreferencesPanel
        windowPin={windowPin}
        preferences={preferences}
        status={preferencesStatus}
        disabled={!preferencesLoaded || continuousActionPending || workspace.frozen}
        scoreLeadAvailable={chartModel.scoreAvailable}
        onChange={(nextPreferences) => void handlePreferencesChange(nextPreferences)}
        onRestoreWorkspace={() => { if (!matchOwnsWorkspace()) workspace.restoreDefaults(); }}
        workspaceDisabled={workspace.disabled || matchBlocked}
        windowGeometryStatus={windowGeometryStatusText}
        windowGeometryDisabled={windowGeometryDisabled}
        onResetWindowGeometry={() => void performWindowGeometryAction(resetWindowGeometry)}
        railVisibilityDisabled={matchBlocked || !preferencesLoaded || railVisibilityBusy || departurePending || Boolean(departurePrompt) || workspace.frozen}
        onRailVisibility={(side, visible) => void handleRailVisibility(side, visible)}
      /> : null}
    </section>
    {layoutExitPrompt ? <div className="shortcut-reference-backdrop" role="presentation">
      <div className="root-conversion-dialog" role="dialog" aria-modal="true" aria-label="工作区或窗口尚未保存">
        <h2>工作区或窗口尚未保存</h2><p>{layoutExitPrompt.message}</p>
        <button type="button" onClick={() => layoutExitPrompt.choose(true)}>重试</button>
        <button type="button" onClick={() => layoutExitPrompt.choose(false)}>取消关闭</button>
      </div>
    </div> : null}
    {finalLayoutExitError ? <div className="shortcut-reference-backdrop" role="presentation">
      <div className="root-conversion-dialog" role="dialog" aria-modal="true" aria-label="退出保存未完成">
        <h2>退出保存未完成</h2><p>{finalLayoutExitError}</p>
        <button type="button" onClick={() => void finishNativeExit()}>重试退出保存</button>
      </div>
    </div> : null}
    {newDocumentOpen ? (
      <NewDocumentDialog
        defaultBoardWidth={preferences.defaultBoardWidth}
        defaultBoardHeight={preferences.defaultBoardHeight}
        defaultKomi={preferences.defaultKomi}
        onCreate={(parameters) => void handleCreateNewGame(parameters)}
        onCancel={() => void handleCancelNewGame()}
      />
    ) : null}
    {conversionPrompt ? <div className="shortcut-reference-backdrop" role="presentation">
      <div className="root-conversion-dialog" role="dialog" aria-modal="true" aria-label="转换为起始局面">
        <h2>转换为起始局面？</h2>
        <p>将当前实际棋子及执色设为无后续根局面，并丢弃原着手树。此操作可撤销；原根信息、评论和源路径保留。</p>
        <div className="button-row">
          <button type="button" disabled={editActionPending} onClick={() => void finishRootEdit("convert")}>确认转换</button>
          <button type="button" disabled={editActionPending} onClick={() => setConversionPrompt(null)}>取消</button>
        </div>
      </div>
    </div> : null}
    {metadataDraft ? (
      <GameMetadataDialog
        blackName={metadataDraft.blackName}
        whiteName={metadataDraft.whiteName}
        komi={metadataDraft.komi}
        handicap={metadataDraft.handicap}
        focusTarget={metadataDraft.focusTarget}
        onApply={(black, white, komi) => handleApplyMetadata(metadataDraft.generation, black, white, komi)}
        onCancel={cancelMetadataEditor}
      />
    ) : null}
    {markupDialog ? (
      <div className="shortcut-reference-backdrop" onClick={() => setMarkupDialog(null)}>
        <form className="new-document-dialog" role="dialog" aria-modal="true" aria-label="编辑文字标记"
          onClick={(event) => event.stopPropagation()}
          onKeyDown={(event) => { if (event.key === "Escape") { event.stopPropagation(); setMarkupDialog(null); } }}
          onSubmit={(event) => {
            event.preventDefault();
            const pending = markupDialog;
            setMarkupDialog(null);
            void commitMarkup({ kind: "point", point: pending.point, tool: { kind: "label", text: pending.text } }, pending);
          }}>
          <h2>当前节点文字标记</h2>
          <label>文字 <input type="text" autoFocus value={markupDialog.text}
            onChange={(event) => setMarkupDialog({ ...markupDialog, text: event.target.value })} /></label>
          <div className="new-document-actions">
            <button type="button" onClick={() => setMarkupDialog(null)}>取消</button>
            <button type="submit">应用</button>
          </div>
        </form>
      </div>
    ) : null}
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
        retryMessage={recoveryDiscardRetryPending && recoveryProtection.status === "unprotected"
          ? recoveryProtection.message
          : null}
        onRestore={() => void handleRestoreRecoveredGame()}
        onDiscard={() => void handleDiscardRecoveredGame()}
        onRetry={recoveryDiscardRetryPending ? () => void handleRetryRecoveryWrite() : null}
      />
    ) : null}
    {aboutOpen ? <AboutDialog onClose={() => { setAboutOpen(false); if (searchSourceRef.current) restoreOwnedFocus(searchSourceRef.current); }} /> : null}
    {functionSearchOpen ? <FunctionSearchPanel catalog={functionCatalog} onCancel={cancelFunctionSearch}
      initialSession={searchSessionRef.current} onSessionChange={(session) => { searchSessionRef.current = session; }}
      onExecute={(action) => { setFunctionSearchOpen(false); action.execute(); }} /> : null}
    {shortcutReferenceOpen ? (
      <ShortcutReference
        entries={shortcutRegistry.referenceEntries()}
        onClose={() => setShortcutReferenceOpen(false)}
      />
    ) : null}
  </main>;
}


/** One source's view of the single owner: another source's session reads as idle here. */
function syncView(sync: ExternalSyncSnapshot | null, source: ExternalSyncSource): ExternalSyncSnapshot | null {
  if (!sync || sync.source === null || sync.source === source) return sync;
  return { ...sync, phase: "idle", session_id: null, starting_id: null, failure: null, source_status: null, browser_error: null, retry_count: 0, readboard: null };
}

function syncMuted(sync: ExternalSyncSnapshot | null): boolean {
  if (sync?.session_id == null) return false;
  return sync.source === "readboard" ? sync.readboard?.preferences.mute ?? true : sync.preferences.mute;
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
    if (key === "windowGeometry") return;
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

function currentMatchAnalysisFrame(match: MatchSnapshotDto | null, current: CurrentGameResultDto | null): AnalysisFrameDto | undefined {
  const analysis = match?.analysis;
  const envelope = analysis?.frame;
  if (!match || match.mode !== "human" || match.phase !== "playing" || match.end || match.failure || !analysis?.supported || !envelope || !current
    || !match.settings || !match.to_play || analysis.policy === "off") return undefined;
  const human = match.to_play === match.settings.human_color;
  if (analysis.policy !== "both" && analysis.policy !== (human ? "human_turn" : "engine_turn")) return undefined;
  const token = envelope.turn;
  const job = envelope.job;
  const activeJob = match.job;
  if (envelope.epoch !== analysis.epoch || token.session_id !== match.session_id || token.turn !== match.turn
    || token.generation !== current.generation || !samePath(token.node_path, current.selected_path)
    || job.run_id !== match.run_id || envelope.frame.job_id !== job.job_id
    || job.generation !== token.generation || !samePath(job.node_path, token.node_path)
    || (activeJob && (activeJob.job_id !== job.job_id || activeJob.run_id !== job.run_id
      || activeJob.generation !== job.generation || !samePath(activeJob.node_path, job.node_path)))) return undefined;
  return envelope.frame;
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

function analysisTaskStageDraft(
  overview: AnalysisStageConditionsDto,
  deep: AnalysisStageConditionsDto
): Partial<AnalysisScopeDraft> {
  return {
    overviewTimeEnabled: overview.time_seconds.enabled,
    overviewTimeSeconds: String(overview.time_seconds.value),
    overviewTotalVisitsEnabled: overview.total_visits.enabled,
    overviewTotalVisits: String(overview.total_visits.value),
    overviewLeadingCandidateVisitsEnabled: overview.leading_candidate_visits.enabled,
    overviewLeadingCandidateVisits: String(overview.leading_candidate_visits.value),
    timeEnabled: deep.time_seconds.enabled,
    timeSeconds: String(deep.time_seconds.value),
    totalVisitsEnabled: deep.total_visits.enabled,
    totalVisits: String(deep.total_visits.value),
    leadingCandidateVisitsEnabled: deep.leading_candidate_visits.enabled,
    leadingCandidateVisits: String(deep.leading_candidate_visits.value)
  };
}

function swingCriteriaDraft(criteria: AnalysisSwingCriteriaDto): Partial<AnalysisScopeDraft> {
  return {
    moveActors: criteria.move_actors,
    winrateChangeEnabled: criteria.winrate_change_percentage_points.enabled,
    winrateChangeThreshold: String(criteria.winrate_change_percentage_points.value),
    scoreChangeEnabled: criteria.score_change_points.enabled,
    scoreChangeThreshold: String(criteria.score_change_points.value)
  };
}

function swingCriteriaFromDraft(draft: AnalysisScopeDraft): AnalysisSwingCriteriaDto {
  const positive = (value: string, label: string) => {
    const parsed = Number(value);
    if (!Number.isFinite(parsed) || parsed <= 0) {
      throw new Error(`${label} must be a positive finite number.`);
    }
    return parsed;
  };
  return {
    move_actors: draft.moveActors,
    winrate_change_percentage_points: {
      enabled: draft.winrateChangeEnabled,
      value: positive(draft.winrateChangeThreshold, "Winrate swing threshold")
    },
    score_change_points: {
      enabled: draft.scoreChangeEnabled,
      value: positive(draft.scoreChangeThreshold, "Score swing threshold")
    }
  };
}

function analysisTaskProgress(task: AnalysisTaskDto): string {
  const completed = task.stage === "overview" ? task.overview_completed.length : task.completed.length;
  const expected = task.stage === "overview"
    ? task.requested.length + task.supporting.length
    : task.selected_for_deep?.length ?? task.requested.length;
  return `${completed}/${expected}`;
}
