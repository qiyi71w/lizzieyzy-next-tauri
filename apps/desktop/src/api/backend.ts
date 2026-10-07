import type { ProviderRequestIdentity } from "../domain/providers";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import type {
  AnalysisFrameDto,
  AnalysisJobEventDto,
  AnalysisJobStartedDto,
  AnalysisScopeDto,
  AnalysisScopePreviewDto,
  AnalysisSwingCriteriaDto,
  AnalysisStageConditionsDto,
  AnalysisTaskDto,
  AnalysisTaskStrategyDto,
  AppHealthDto,
  AssetCheckDto,
  CandidateMoveDto,
  CurrentGameResultDto,
  CurrentGameSaveResultDto,
  ApplicationExitActionDto,
  ApplicationExitOutcomeDto,
  DocumentDepartureActionDto,
  DocumentDepartureAdmissionDto,
  DocumentDepartureOutcomeDto,
  NodePath,
  EngineProfileRecordDto,
  EngineProfileDto,
  EngineProfilesSettingsDto,
  EngineProfileOrderRequestDto,
  EngineFailureDto,
  ForegroundEngineSnapshotDto,
  FileActivationDeliveryDto,
  FileActivationRejectionDto,
  GameFileImportDto,
  GameDto,
  GameMoveRequestDto,
  GameMoveResultDto,
  MoveDto,
  MoveVertex,
  PlayerColor,
  PositionDto,
  ProblemMarkerDto,
  SgfMarkupActionDto,
  RecoveryProtectionDto,
  RecoveryStartupDto,
  StoneDto,
  TrialSessionDto,
  ScoringActionDto,
  ScoringRuleDto,
  ScoringSessionDto
} from "../domain/types";
import { emptyForegroundEngineSnapshot, mergeForegroundEngineSnapshot } from "../domain/foregroundEngine";
import { ensureInitialPosition, replayGamePositions } from "../domain/board";
import { normalizeAppPreferences, type AppPreferences } from "../domain/preferences";

const letters = "abcdefghijklmnopqrstuvwxyz";
const sampleGameId = "browser-sgf";
const gameDialogFilters = [{ name: "Game files", extensions: ["sgf", "txt", "gib"] }];
declare global {
  interface Window {
    __TAURI_INTERNALS__?: unknown;
  }
}

export const isTauriRuntime = () => typeof window !== "undefined" && window.__TAURI_INTERNALS__ !== undefined;

export async function getHealth(): Promise<AppHealthDto> {
  if (!isTauriRuntime()) {
    return browserHealth("Tauri APIs are unavailable, so SGF parsing and analysis are running in browser preview mode.");
  }
  try {
    return await invoke<AppHealthDto>("health");
  } catch {
    return browserHealth("Tauri is running, but backend commands are not ready; using local review fallback.");
  }
}

export async function parseSgfSummary(sgfText: string): Promise<GameDto> {
  if (!isTauriRuntime()) return parseSgfLocally(sgfText);
  try {
    return await invoke<GameDto>("parse_sgf_summary", { sgfText });
  } catch {
    return parseSgfLocally(sgfText);
  }
}

export const nativeSyntheticAnalysisUnavailable =
  "Synthetic analysis is not available in the native desktop product. Browser preview demonstration is non-authoritative.";

export async function fakeAnalyze(sgfText: string): Promise<AnalysisFrameDto[]> {
  if (!isTauriRuntime()) return buildBrowserAnalysis(parseSgfLocally(sgfText));
  throw new Error(nativeSyntheticAnalysisUnavailable);
}

export async function openSgfDocument(): Promise<GameFileImportDto | null> {
  if (!isTauriRuntime()) return null;
  const selected = await open({
    multiple: false,
    directory: false,
    filters: gameDialogFilters
  });
  if (typeof selected !== "string") return null;
  return readGameFile(selected);
}

export async function readGameFile(path: string): Promise<GameFileImportDto> {
  if (!isTauriRuntime()) throw new Error(nativeCurrentGameUnavailable);
  return invoke<GameFileImportDto>("read_game_file", { path });
}

export async function takeInitialFileActivation(): Promise<FileActivationDeliveryDto | null> {
  if (!isTauriRuntime()) return null;
  return invoke<FileActivationDeliveryDto | null>("take_initial_file_activation");
}

export async function takePendingFileActivation(): Promise<FileActivationDeliveryDto | null> {
  if (!isTauriRuntime()) return null;
  return invoke<FileActivationDeliveryDto | null>("take_pending_file_activation");
}

export async function markFileActivationReady(): Promise<void> {
  if (!isTauriRuntime()) return;
  await invoke("mark_file_activation_ready");
}

export async function setFileActivationBusy(busy: boolean): Promise<void> {
  if (!isTauriRuntime()) return;
  await invoke("set_file_activation_busy", { busy });
}

export async function subscribeFileActivationAvailable(onAvailable: () => void): Promise<() => void> {
  if (!isTauriRuntime()) return () => undefined;
  return listen("file-activation://available", onAvailable);
}

export async function subscribeFileActivationRejected(
  onRejected: (rejection: FileActivationRejectionDto) => void
): Promise<() => void> {
  if (!isTauriRuntime()) return () => undefined;
  return listen<FileActivationRejectionDto>("file-activation://rejected", (event) => onRejected(event.payload));
}

export async function importGameFile(file: File): Promise<GameFileImportDto> {
  if (!isTauriRuntime()) {
    if (file.name.toLowerCase().endsWith(".gib")) {
      throw new Error("GIB import requires the native Tauri desktop backend.");
    }
    return {
      format: "sgf",
      sgf_text: await file.text(),
      display_path: file.name,
      display_name: file.name,
      native_path: null
    };
  }
  const input = Array.from(new Uint8Array(await file.arrayBuffer()));
  return invoke<GameFileImportDto>("import_game_bytes", { fileName: file.name, input });
}

export const nativeCurrentGameUnavailable =
  "Native current-game, edit, and authoritative Save require the Tauri desktop backend. Browser preview is non-authoritative.";


export async function prepareDocumentReplacement(
  sgfText: string,
  nativePath: string | null,
  networkIdentity?: ProviderRequestIdentity
): Promise<DocumentDepartureAdmissionDto> {
  if (!isTauriRuntime()) {
    throw new Error(nativeCurrentGameUnavailable);
  }
  return invoke<DocumentDepartureAdmissionDto>("prepare_document_replacement", { sgfText, nativePath, networkIdentity: networkIdentity ?? null });
}

export async function resolveDocumentReplacement(input: {
  departureId: number;
  action: DocumentDepartureActionDto;
  selectedPath: NodePath;
  defaultFileName?: string | null;
}): Promise<DocumentDepartureOutcomeDto> {
  if (!isTauriRuntime()) {
    throw new Error(nativeCurrentGameUnavailable);
  }
  return invoke<DocumentDepartureOutcomeDto>("resolve_document_replacement", input);
}

export async function prepareApplicationExit(): Promise<DocumentDepartureAdmissionDto> {
  if (!isTauriRuntime()) {
    throw new Error(nativeCurrentGameUnavailable);
  }
  return invoke<DocumentDepartureAdmissionDto>("prepare_application_exit");
}

export async function resolveApplicationExit(input: {
  departureId: number;
  action: ApplicationExitActionDto;
  selectedPath: NodePath;
  defaultFileName?: string | null;
}): Promise<ApplicationExitOutcomeDto> {
  if (!isTauriRuntime()) {
    throw new Error(nativeCurrentGameUnavailable);
  }
  return invoke<ApplicationExitOutcomeDto>("resolve_application_exit", input);
}

export async function retryApplicationTeardown(input: {
  departureId: number;
  selectedPath: NodePath;
}): Promise<ApplicationExitOutcomeDto> {
  if (!isTauriRuntime()) {
    throw new Error(nativeCurrentGameUnavailable);
  }
  return invoke<ApplicationExitOutcomeDto>("retry_application_teardown", input);
}

export async function confirmApplicationExitAnyway(input: {
  departureId: number;
  selectedPath: NodePath;
  outstanding: string[];
}): Promise<ApplicationExitOutcomeDto> {
  if (!isTauriRuntime()) {
    throw new Error(nativeCurrentGameUnavailable);
  }
  return invoke<ApplicationExitOutcomeDto>("confirm_application_exit_anyway", input);
}

export async function confirmNativeExit(): Promise<void> {
  if (!isTauriRuntime()) {
    throw new Error(nativeCurrentGameUnavailable);
  }
  await invoke("confirm_native_exit");
}

export async function subscribeApplicationExitRequested(onRequest: () => void): Promise<() => void> {
  if (!isTauriRuntime()) {
    return () => undefined;
  }
  return listen("application-exit-requested", () => {
    onRequest();
  });
}

export async function inspectCurrentGameRecovery(): Promise<RecoveryStartupDto> {
  if (!isTauriRuntime()) {
    throw new Error(nativeCurrentGameUnavailable);
  }
  return invoke<RecoveryStartupDto>("inspect_current_game_recovery");
}

export async function restoreCurrentGameRecovery(): Promise<CurrentGameResultDto> {
  if (!isTauriRuntime()) {
    throw new Error(nativeCurrentGameUnavailable);
  }
  return invoke<CurrentGameResultDto>("restore_current_game_recovery");
}

export async function discardCurrentGameRecovery(): Promise<void> {
  if (!isTauriRuntime()) {
    throw new Error(nativeCurrentGameUnavailable);
  }
  await invoke("discard_current_game_recovery");
}

export async function retryCurrentGameRecovery(): Promise<RecoveryProtectionDto> {
  if (!isTauriRuntime()) {
    throw new Error(nativeCurrentGameUnavailable);
  }
  return invoke<RecoveryProtectionDto>("retry_current_game_recovery");
}

export async function currentGameRecoveryProtection(): Promise<RecoveryProtectionDto> {
  if (!isTauriRuntime()) {
    throw new Error(nativeCurrentGameUnavailable);
  }
  return invoke<RecoveryProtectionDto>("current_game_recovery_protection");
}

export async function subscribeCurrentGameRecoveryProtection(
  onProtection: (protection: RecoveryProtectionDto) => void
): Promise<() => void> {
  if (!isTauriRuntime()) {
    return () => undefined;
  }
  return listen<RecoveryProtectionDto>("current-game-recovery://protection", (event) => {
    onProtection(event.payload);
  });
}

export async function serializeCurrentGame(): Promise<string> {
  if (!isTauriRuntime()) {
    throw new Error(nativeCurrentGameUnavailable);
  }
  return invoke<string>("serialize_current_game");
}

export async function projectCurrentGameMainline(): Promise<GameDto> {
  if (!isTauriRuntime()) {
    throw new Error(nativeCurrentGameUnavailable);
  }
  return invoke<GameDto>("project_current_game_mainline");
}

export async function selectCurrentGameNode(path: NodePath, generation: number): Promise<CurrentGameResultDto> {
  if (!isTauriRuntime()) {
    throw new Error("Native current-game navigation requires the Tauri desktop backend.");
  }
  return invoke<CurrentGameResultDto>("select_current_game_node", { path, generation });
}

export async function playCurrentGame(path: NodePath, vertex: MoveVertex): Promise<CurrentGameResultDto> {
  if (!isTauriRuntime()) {
    throw new Error(nativeCurrentGameUnavailable);
  }
  return invoke<CurrentGameResultDto>("play_current_game", { path, vertex });
}

export async function setCurrentGamePersonalComment(path: NodePath, comment: string): Promise<CurrentGameResultDto> {
  if (!isTauriRuntime()) {
    throw new Error(nativeCurrentGameUnavailable);
  }
  return invoke<CurrentGameResultDto>("set_current_game_personal_comment", { path, comment });
}
export async function setCurrentGameMetadata(
  generation: number, blackName: string, whiteName: string, komi: number
): Promise<CurrentGameResultDto> {
  if (!isTauriRuntime()) throw new Error(nativeCurrentGameUnavailable);
  return invoke<CurrentGameResultDto>("set_current_game_metadata", { generation, blackName, whiteName, komi });
}


export async function editCurrentGameMarkup(path: NodePath, generation: number, action: SgfMarkupActionDto): Promise<CurrentGameResultDto> {
  if (!isTauriRuntime()) throw new Error(nativeCurrentGameUnavailable);
  return invoke<CurrentGameResultDto>("edit_current_game_markup", { path, generation, action });
}

export async function removeCurrentGameVariation(path: NodePath, generation: number): Promise<CurrentGameResultDto> {
  if (!isTauriRuntime()) {
    throw new Error(nativeCurrentGameUnavailable);
  }
  return invoke<CurrentGameResultDto>("remove_current_game_variation", { path, generation });
}

export async function promoteCurrentGameToMain(path: NodePath, generation: number): Promise<CurrentGameResultDto> {
  if (!isTauriRuntime()) {
    throw new Error(nativeCurrentGameUnavailable);
  }
  return invoke<CurrentGameResultDto>("promote_current_game_to_main", { path, generation });
}

export async function applyRootSetup(generation: number, stones: StoneDto[], toPlay: PlayerColor): Promise<CurrentGameResultDto> {
  if (!isTauriRuntime()) throw new Error(nativeCurrentGameUnavailable);
  return invoke<CurrentGameResultDto>("apply_root_setup", { generation, stones, toPlay });
}

export async function convertToRootSetup(generation: number, path: NodePath): Promise<CurrentGameResultDto> {
  if (!isTauriRuntime()) throw new Error(nativeCurrentGameUnavailable);
  return invoke<CurrentGameResultDto>("convert_to_root_setup", { generation, path });
}

export async function undoCurrentGame(generation: number): Promise<CurrentGameResultDto> {
  if (!isTauriRuntime()) {
    throw new Error(nativeCurrentGameUnavailable);
  }
  return invoke<CurrentGameResultDto>("undo_current_game", { generation });
}

export async function redoCurrentGame(generation: number): Promise<CurrentGameResultDto> {
  if (!isTauriRuntime()) {
    throw new Error(nativeCurrentGameUnavailable);
  }
  return invoke<CurrentGameResultDto>("redo_current_game", { generation });
}

export async function enterTrial(): Promise<TrialSessionDto> {
  if (!isTauriRuntime()) throw new Error(nativeCurrentGameUnavailable);
  return invoke<TrialSessionDto>("enter_trial");
}

export async function exitTrial(sessionId: number): Promise<CurrentGameResultDto> {
  if (!isTauriRuntime()) throw new Error(nativeCurrentGameUnavailable);
  return invoke<CurrentGameResultDto>("exit_trial", { sessionId });
}
export async function enterScoring(rule: ScoringRuleDto): Promise<ScoringSessionDto> {
  if (!isTauriRuntime()) throw new Error(nativeCurrentGameUnavailable);
  return invoke<ScoringSessionDto>("enter_scoring", { rule });
}

export async function updateScoring(sessionId: number, revision: number, action: ScoringActionDto): Promise<ScoringSessionDto> {
  if (!isTauriRuntime()) throw new Error(nativeCurrentGameUnavailable);
  return invoke<ScoringSessionDto>("update_scoring", { sessionId, revision, action });
}

export async function exitScoring(sessionId: number, revision: number, confirm: boolean): Promise<CurrentGameResultDto> {
  if (!isTauriRuntime()) throw new Error(nativeCurrentGameUnavailable);
  return invoke<CurrentGameResultDto>("exit_scoring", { sessionId, revision, confirm });
}

export async function trialSelect(sessionId: number, revision: number, path: NodePath): Promise<TrialSessionDto> {
  return invoke<TrialSessionDto>("trial_select", { sessionId, revision, path });
}

export async function trialPlay(sessionId: number, revision: number, vertex: MoveVertex): Promise<TrialSessionDto> {
  return invoke<TrialSessionDto>("trial_play", { sessionId, revision, vertex });
}

export async function trialUndo(sessionId: number, revision: number): Promise<TrialSessionDto> {
  return invoke<TrialSessionDto>("trial_undo", { sessionId, revision });
}

export async function trialStartFinite(sessionId: number, revision: number, runId: string, maxVisits: number): Promise<AnalysisJobStartedDto> {
  return invoke<AnalysisJobStartedDto>("trial_start_finite", { sessionId, revision, runId, maxVisits });
}

export async function subscribeTrialAnalysis(onTrial: (trial: TrialSessionDto) => void): Promise<() => void> {
  if (!isTauriRuntime()) return () => undefined;
  return listen<TrialSessionDto>("trial://analysis", (event) => onTrial(event.payload));
}

export async function saveCurrentGame(
  path: string | null,
  selectedPath: NodePath,
  defaultFileName = "review.sgf"
): Promise<CurrentGameSaveResultDto | null> {
  if (!isTauriRuntime()) {
    throw new Error(nativeCurrentGameUnavailable);
  }

  if (path) {
    return invoke<CurrentGameSaveResultDto>("save_current_game", { path, selectedPath });
  }
  return invoke<CurrentGameSaveResultDto | null>("save_current_game_as", { selectedPath, defaultFileName });
}

export async function startKataGoGameAnalysis(input: {
  runId: string;
  generation: number;
  maxVisits: number;
}): Promise<AnalysisJobStartedDto> {
  if (!isTauriRuntime()) {
    throw new Error("Full-game KataGo analysis requires the Tauri desktop backend. Browser preview cannot run real KataGo.");
  }
  return await invoke<AnalysisJobStartedDto>("katago_start_analyze_game", {
    runId: input.runId,
    generation: input.generation,
    maxVisits: input.maxVisits
  });
}

export async function previewAnalysisScope(input: {
  generation: number;
  scope: AnalysisScopeDto;
  swingCriteria?: AnalysisSwingCriteriaDto | null;
}): Promise<AnalysisScopePreviewDto> {
  if (!isTauriRuntime()) {
    throw new Error("Analysis task preview requires the Tauri desktop backend.");
  }
  return invoke<AnalysisScopePreviewDto>("preview_analysis_scope", input);
}

export async function startAnalysisTask(input: {
  runId: string;
  preview: AnalysisScopePreviewDto;
  strategy: AnalysisTaskStrategyDto;
  conditions: AnalysisStageConditionsDto;
  overviewConditions?: AnalysisStageConditionsDto | null;
}): Promise<AnalysisTaskDto> {
  if (!isTauriRuntime()) {
    throw new Error("Analysis tasks require the Tauri desktop backend.");
  }
  return invoke<AnalysisTaskDto>("start_analysis_task", input);
}

export async function pauseAnalysisTask(input: {
  runId: string;
  taskId: string;
}): Promise<AnalysisTaskDto> {
  if (!isTauriRuntime()) {
    throw new Error("Analysis tasks require the Tauri desktop backend.");
  }
  return invoke<AnalysisTaskDto>("pause_analysis_task", input);
}

export async function continueAnalysisTask(input: {
  runId: string;
  taskId: string;
}): Promise<AnalysisTaskDto> {
  if (!isTauriRuntime()) {
    throw new Error("Analysis tasks require the Tauri desktop backend.");
  }
  return invoke<AnalysisTaskDto>("continue_analysis_task", input);
}

export async function analysisTaskSnapshot(): Promise<AnalysisTaskDto | null> {
  if (!isTauriRuntime()) return null;
  return invoke<AnalysisTaskDto | null>("analysis_task_snapshot");
}

export async function cancelKataGoAnalysis(runId: string, jobId: string): Promise<void> {
  if (!isTauriRuntime()) return;
  await invoke<void>("katago_cancel_analysis", { runId, jobId });
}



export async function loadEngineProfilesSettings(): Promise<EngineProfilesSettingsDto> {
  if (!isTauriRuntime()) return loadBrowserEngineProfilesSettings();
  return await invoke<EngineProfilesSettingsDto>("load_engine_profiles_settings");
}

export async function saveEngineProfilesSettings(settings: EngineProfilesSettingsDto): Promise<EngineProfilesSettingsDto> {
  if (!isTauriRuntime()) {
    const normalized = decodeEngineProfilesSettings(settings, false);
    const current = loadBrowserEngineProfilesSettings();
    const records = new Map(normalized.profiles.map((record) => [record.id, record]));
    const retained: EngineProfileRecordDto[] = [];
    for (const record of current.profiles) {
      const saved = records.get(record.id);
      if (saved) {
        retained.push(saved);
        records.delete(record.id);
      }
    }
    normalized.profiles = [...retained, ...records.values()];
    saveBrowserEngineProfilesSettings(normalized);
    return normalized;
  }
  return await invoke<EngineProfilesSettingsDto>("save_engine_profiles_settings", { settings });
}

export async function reorderEngineProfilesSettings(request: EngineProfileOrderRequestDto): Promise<EngineProfilesSettingsDto> {
  if (isTauriRuntime()) return await invoke<EngineProfilesSettingsDto>("reorder_engine_profiles_settings", { request });
  const current = loadBrowserEngineProfilesSettings();
  const ids = current.profiles.map((record) => record.id);
  if (ids.length !== request.expected_profile_ids.length || ids.some((id, index) => id !== request.expected_profile_ids[index])) {
    throw new Error("Engine profile catalog order is stale; reload profiles before reordering.");
  }
  const records = new Map(current.profiles.map((record) => [record.id, record]));
  if (request.profile_ids.length !== ids.length || new Set(request.profile_ids).size !== ids.length || request.profile_ids.some((id) => !records.has(id))) {
    throw new Error("Engine profile order must contain the complete catalog without unknown or duplicate IDs.");
  }
  if (ids.every((id, index) => id === request.profile_ids[index])) return current;
  const reordered = { ...current, profiles: request.profile_ids.map((id) => records.get(id)!) };
  saveBrowserEngineProfilesSettings(reordered);
  return reordered;
}

export type ForegroundEngineEventHandlers = {
  onSnapshot?: (snapshot: ForegroundEngineSnapshotDto) => void;
  onFailure?: (failure: EngineFailureDto) => void;
  onJob?: (job: AnalysisJobEventDto) => void;
};

export async function startSelectedNodeAnalysis(input: {
  runId: string;
  generation: number;
  nodePath: NodePath;
  maxVisits: number;
}): Promise<AnalysisJobStartedDto> {
  if (!isTauriRuntime()) {
    throw new Error("Selected-node analysis requires a Ready Foreground Engine Run on the Tauri desktop backend.");
  }
  return await invoke<AnalysisJobStartedDto>("foreground_engine_start_selected_node", {
    runId: input.runId,
    generation: input.generation,
    nodePath: input.nodePath,
    maxVisits: input.maxVisits
  });
}

export async function computeGameMove(request: GameMoveRequestDto): Promise<GameMoveResultDto> {
  if (!isTauriRuntime()) {
    throw new Error("Exact game moves require a Ready Foreground Engine Run on the Tauri desktop backend.");
  }
  return invoke<GameMoveResultDto>("foreground_engine_game_move", { request });
}

export async function cancelGameMove(input: { runId: string; jobId: string }): Promise<void> {
  if (!isTauriRuntime()) return;
  await invoke<void>("foreground_engine_cancel_game_move", input);
}


export async function cancelSelectedNodeAnalysis(input: { runId: string; jobId: string }): Promise<void> {
  if (!isTauriRuntime()) return;
  await invoke<void>("foreground_engine_cancel_job", { runId: input.runId, jobId: input.jobId });
}

export async function getForegroundEngineSnapshot(): Promise<ForegroundEngineSnapshotDto> {
  if (!isTauriRuntime()) return emptyForegroundEngineSnapshot();
  return await invoke<ForegroundEngineSnapshotDto>("foreground_engine_snapshot");
}

export async function startForegroundEngine(profileId: string): Promise<void> {
  if (!isTauriRuntime()) {
    throw new Error("Starting a Foreground Engine Run requires the Tauri desktop backend.");
  }
  await invoke<void>("foreground_engine_start", { profileId });
}

export async function stopForegroundEngine(): Promise<void> {
  if (!isTauriRuntime()) return;
  await invoke<void>("foreground_engine_stop");
}

export async function restartForegroundEngine(): Promise<void> {
  if (!isTauriRuntime()) {
    throw new Error("Restarting a Foreground Engine Run requires the Tauri desktop backend.");
  }
  await invoke<void>("foreground_engine_restart");
}

export async function switchForegroundEngine(profileId: string): Promise<void> {
  if (!isTauriRuntime()) {
    throw new Error("Switching a Foreground Engine Run requires the Tauri desktop backend.");
  }
  await invoke<void>("foreground_engine_switch", { profileId });
}

export async function foregroundEngineContinuousAction(): Promise<AppPreferences> {
  if (!isTauriRuntime()) {
    throw new Error("Continuous analysis runtime actions require the Tauri desktop backend.");
  }
  return normalizeAppPreferences(await invoke<AppPreferences>("foreground_engine_continuous_action"));
}

export async function listenToForegroundEngineEvents(
  handlers: ForegroundEngineEventHandlers
): Promise<() => void> {
  if (!isTauriRuntime()) return () => undefined;
  const unlisteners = await Promise.all([
    listen<ForegroundEngineSnapshotDto>("foreground-engine://snapshot", (event) => handlers.onSnapshot?.(event.payload)),
    listen<EngineFailureDto>("foreground-engine://failure", (event) => handlers.onFailure?.(event.payload)),
    listen<AnalysisJobEventDto>("foreground-engine://job", (event) => handlers.onJob?.(event.payload))
  ]);
  return () => {
    for (const unlisten of unlisteners) unlisten();
  };
}

export async function subscribeForegroundEngine(
  onSnapshot: (snapshot: ForegroundEngineSnapshotDto) => void,
  onFailure?: (failure: EngineFailureDto) => void,
  onJob?: (job: AnalysisJobEventDto) => void
): Promise<() => void> {
  let current = emptyForegroundEngineSnapshot();
  const apply = (incoming: ForegroundEngineSnapshotDto) => {
    current = mergeForegroundEngineSnapshot(current, incoming);
    onSnapshot(current);
  };
  const unlisten = await listenToForegroundEngineEvents({
    onSnapshot: apply,
    onFailure,
    onJob
  });
  try {
    apply(await getForegroundEngineSnapshot());
  } catch (error) {
    unlisten();
    throw error;
  }
  return unlisten;
}

export async function checkEngineAssets(profile: EngineProfileDto): Promise<AssetCheckDto[]> {
  if (!isTauriRuntime()) {
    throw new Error("Asset checks require the Tauri desktop backend so local files can be inspected.");
  }
  return await invoke<AssetCheckDto[]>("engine_asset_checks", { profile });
}

export async function replaySgfPositions(sgfText: string): Promise<PositionDto[]> {
  if (!isTauriRuntime()) return replayGamePositions(parseSgfLocally(sgfText));
  try {
    const parsed = await parseSgfSummary(sgfText);
    const positions = await invoke<PositionDto[]>("replay_sgf_positions", { sgfText });
    return ensureInitialPosition(parsed.summary.board_width, parsed.summary.board_height, positions);
  } catch {
    return replayGamePositions(parseSgfLocally(sgfText));
  }
}

export async function classifyProblems(frames: AnalysisFrameDto[]): Promise<ProblemMarkerDto[]> {
  if (!isTauriRuntime()) return classifyProblemFrames(frames);
  try {
    return await invoke<ProblemMarkerDto[]>("classify_problems", { frames });
  } catch {
    return classifyProblemFrames(frames);
  }
}

function browserHealth(note: string): AppHealthDto {
  return { app: "LizzieYzy Next", architecture: "React review workspace fallback", rust_backend_ready: false, notes: [note] };
}

const browserEngineProfileKey = "lizzieyzy-next-engine-profile";
const defaultEngineProfileId = "default";

function loadBrowserEngineProfilesSettings(): EngineProfilesSettingsDto {
  if (typeof window === "undefined") return defaultBrowserEngineProfilesSettings();
  const raw = window.localStorage.getItem(browserEngineProfileKey);
  if (raw === null) return defaultBrowserEngineProfilesSettings();
  return decodeEngineProfilesSettings(JSON.parse(raw), true);
}

function saveBrowserEngineProfilesSettings(settings: EngineProfilesSettingsDto) {
  if (typeof window === "undefined") throw new Error("Profile storage requires a browser window.");
  window.localStorage.setItem(browserEngineProfileKey, JSON.stringify(settings));
}

function profileObject(value: unknown, label: string): Record<string, unknown> {
  if (!value || typeof value !== "object" || Array.isArray(value)) throw new Error(`${label} must be an object.`);
  return value as Record<string, unknown>;
}

function profileString(value: unknown, label: string, nonempty = false): string {
  if (typeof value !== "string" || value.includes("\0") || (nonempty && !value.trim())) {
    throw new Error(`${label} must be ${nonempty ? "a non-empty" : "a"} NUL-free string.`);
  }
  return value;
}

function profileOptionalPath(value: unknown, label: string): string | null {
  return value == null ? null : profileString(value, label);
}

function profileVisits(value: unknown): number {
  if (typeof value !== "number" || !Number.isInteger(value) || value < 1 || value > 0xffffffff) {
    throw new Error("KataGo max_visits must be a positive 32-bit integer.");
  }
  return value;
}

function decodeEngineProfile(value: unknown): EngineProfileDto {
  const profile = profileObject(value, "Profile");
  if ("engine_path" in profile || "backend" in profile || "model_path" in profile || "config_path" in profile) {
    throw new Error("Version 1 profiles require program, argv and adapter settings.");
  }
  const common = {
    name: profileString(profile.name, "Profile name", true),
    program: profileString(profile.program, "Program"),
    argv: Array.isArray(profile.argv)
      ? profile.argv.map((argument) => profileString(argument, "Argument"))
      : (() => { throw new Error("Profile argv must be an array."); })(),
    working_dir: profileOptionalPath(profile.working_dir, "Working directory")
  };
  const settings = profileObject(profile.settings, "Adapter settings");
  if (profile.adapter_kind === "generic_gtp") {
    if (Object.keys(settings).length) throw new Error("GenericGtp settings must be empty.");
    return { ...common, adapter_kind: "generic_gtp", settings: {} };
  }
  if (profile.adapter_kind !== "kata_go_analysis") throw new Error("Unknown engine adapter.");
  if (Object.keys(settings).some((key) => !["model_path", "config_path", "max_visits"].includes(key))) {
    throw new Error("Unknown KataGo setting.");
  }
  const reservedModes: Record<string, true> = {
    analysis: true, gtp: true, benchmark: true, tuner: true, contribute: true, match: true,
    gatekeeper: true, runtests: true, evalsgf: true, genconfig: true, version: true, help: true,
    selfplay: true, startposes: true, demoplay: true, clockinfo: true,
    "-model": true, "--model": true, "-config": true, "--config": true
  };
  if (common.argv.some((argument) => Object.hasOwn(reservedModes, argument.split("=", 1)[0]))) {
    throw new Error("KataGo argv conflicts with adapter-owned analysis/model/config arguments.");
  }
  return {
    ...common,
    adapter_kind: "kata_go_analysis",
    settings: {
      model_path: profileOptionalPath(settings.model_path, "Model path"),
      config_path: profileOptionalPath(settings.config_path, "Config path"),
      max_visits: profileVisits(settings.max_visits)
    }
  };
}

function decodeLegacyEngineProfile(value: unknown, maxVisits: unknown): EngineProfileDto {
  const profile = profileObject(value, "Legacy profile");
  if (profile.backend !== "kata_go_analysis") throw new Error("Unsupported legacy engine backend.");
  return decodeEngineProfile({
    name: profile.name,
    program: profile.engine_path,
    argv: [],
    working_dir: profile.working_dir,
    adapter_kind: "kata_go_analysis",
    settings: { model_path: profile.model_path, config_path: profile.config_path, max_visits: maxVisits }
  });
}

function decodeEngineProfilesSettings(value: unknown, allowLegacy: boolean): EngineProfilesSettingsDto {
  const input = profileObject(value, "Profile catalog");
  const legacy = !("version" in input);
  if ((!legacy && input.version !== 1) || (legacy && !allowLegacy)) throw new Error("Unsupported profile catalog version.");
  const single = legacy && !("profiles" in input);
  const records = single ? [{ id: defaultEngineProfileId, profile: input.profile, max_visits: input.max_visits }] : input.profiles;
  if (!Array.isArray(records) || records.length === 0) throw new Error("Profile catalog must contain at least one profile.");
  const profiles: EngineProfileRecordDto[] = records.map((value) => {
    const record = profileObject(value, "Profile record");
    if (!legacy && "max_visits" in record) throw new Error("Version 1 max_visits belongs in KataGo settings.");
    return {
      id: profileString(record.id, "Profile ID", true),
      profile: legacy ? decodeLegacyEngineProfile(record.profile, record.max_visits) : decodeEngineProfile(record.profile)
    };
  });
  const ids = new Set(profiles.map((record) => record.id));
  if (ids.size !== profiles.length) throw new Error("Duplicate profile ID.");
  const selected = single ? defaultEngineProfileId : profileString(input.selected_profile_id, "Selected profile ID", true);
  const autoload = single ? null : profileOptionalPath(input.autoload_profile_id, "Autoload profile ID");
  if (!ids.has(selected) || (autoload !== null && !ids.has(autoload))) throw new Error("Selected/Autoload profile ID is not in the catalog.");
  return { version: 1, selected_profile_id: selected, autoload_profile_id: autoload, profiles };
}

function defaultBrowserEngineProfilesSettings(): EngineProfilesSettingsDto {
  return {
    version: 1,
    selected_profile_id: defaultEngineProfileId,
    autoload_profile_id: null,
    profiles: [{
      id: defaultEngineProfileId,
      profile: {
        name: "Local KataGo", program: "", argv: [], working_dir: null,
        adapter_kind: "kata_go_analysis", settings: { model_path: null, config_path: null, max_visits: 800 }
      }
    }]
  };
}

function downloadSgf(sgfText: string, fileName: string) {
  if (typeof document === "undefined") return;
  const blob = new Blob([sgfText], { type: "application/x-go-sgf;charset=utf-8" });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = fileName || "review.sgf";
  document.body.append(anchor);
  anchor.click();
  anchor.remove();
  URL.revokeObjectURL(url);
}

function fileNameFromPath(path: string): string {
  return path.split(/[\\/]/).filter(Boolean).at(-1) ?? "review.sgf";
}

function parseSgfLocally(sgfText: string): GameDto {
  const text = sgfText.trim();
  if (!text.startsWith("(") || !text.includes(";")) {
    throw new Error("The SGF text does not look like a game tree.");
  }

  const { width: boardWidth, height: boardHeight } = parseBoardDimensions(textProperty(text, "SZ"));
  const komi = numberProperty(text, "KM") ?? 7.5;
  const moves = extractMainVariationNodes(text)
    .flatMap((node) => {
      const match = /(?:^|[^A-Za-z])([BW])\[([a-z]{0,2})\]/i.exec(node);
      const color: PlayerColor = match?.[1].toUpperCase() === "B" ? "black" : "white";
      return match ? [{ color, rawVertex: match[2] }] : [];
    })
    .map<MoveDto>((move, index) => ({
      color: move.color,
      vertex: parseVertex(move.rawVertex, boardWidth, boardHeight),
      move_number: index + 1
    }));


  return {
    summary: {
      id: sampleGameId,
      board_width: boardWidth,
      board_height: boardHeight,
      komi,
      black_name: textProperty(text, "PB") ?? "Black",
      white_name: textProperty(text, "PW") ?? "White",
      result: textProperty(text, "RE"),
      move_count: moves.length
    },
    moves
  };
}

export function extractMainVariationNodes(sgfText: string): string[] {
  const nodes: string[] = [];
  const text = sgfText.trim();
  const start = text.indexOf("(");
  if (start < 0) return nodes;

  function parseSequence(index: number): number {
    let cursor = index;
    while (cursor < text.length) {
      cursor = skipWhitespace(cursor);
      const char = text[cursor];
      if (char === ";") {
        const node = readNode(cursor + 1);
        nodes.push(node.value);
        cursor = node.nextIndex;
      } else if (char === "(") {
        return skipSiblingVariations(parseSequence(cursor + 1));
      } else if (char === ")") {
        return cursor + 1;
      } else {
        cursor += 1;
      }
    }
    return cursor;
  }

  parseSequence(start + 1);
  return nodes;

  function readNode(index: number): { value: string; nextIndex: number } {
    let cursor = index;
    let inValue = false;
    let escaped = false;
    while (cursor < text.length) {
      const char = text[cursor];
      if (inValue) {
        if (escaped) escaped = false;
        else if (char === "\\") escaped = true;
        else if (char === "]") inValue = false;
      } else if (char === "[") {
        inValue = true;
      } else if (char === ";" || char === "(" || char === ")") {
        break;
      }
      cursor += 1;
    }
    return { value: text.slice(index, cursor), nextIndex: cursor };
  }

  function skipSiblingVariations(index: number): number {
    let cursor = skipWhitespace(index);
    while (text[cursor] === "(") {
      cursor = skipGameTree(cursor);
      cursor = skipWhitespace(cursor);
    }
    return cursor;
  }

  function skipGameTree(index: number): number {
    let cursor = index;
    let depth = 0;
    let inValue = false;
    let escaped = false;
    while (cursor < text.length) {
      const char = text[cursor];
      if (inValue) {
        if (escaped) escaped = false;
        else if (char === "\\") escaped = true;
        else if (char === "]") inValue = false;
      } else if (char === "[") {
        inValue = true;
      } else if (char === "(") {
        depth += 1;
      } else if (char === ")") {
        depth -= 1;
        if (depth === 0) return cursor + 1;
      }
      cursor += 1;
    }
    return cursor;
  }

  function skipWhitespace(index: number): number {
    let cursor = index;
    while (/\s/.test(text[cursor] ?? "")) cursor += 1;
    return cursor;
  }
}

function parseVertex(raw: string, boardWidth: number, boardHeight: number): MoveVertex {
  if (raw.length !== 2) return "pass";
  const x = letters.indexOf(raw[0].toLowerCase());
  const y = letters.indexOf(raw[1].toLowerCase());
  if (x < 0 || y < 0 || x >= boardWidth || y >= boardHeight) return "pass";
  return { point: { x, y } };
}

function parseBoardDimensions(raw: string | null): { width: number; height: number } {
  if (raw === null) return { width: 19, height: 19 };
  const match = /^(\d+)(?::(\d+))?$/.exec(raw.trim());
  const width = Number(match?.[1]);
  const height = Number(match?.[2] ?? match?.[1]);
  if (!match || !Number.isInteger(width) || !Number.isInteger(height)
    || width < 2 || width > 25 || height < 2 || height > 25) {
    throw new Error("SGF SZ must contain board width and height from 2 to 25.");
  }
  return { width, height };
}

function numberProperty(text: string, property: string): number | null {
  const value = textProperty(text, property);
  if (value === null) return null;
  const parsed = Number(value);
  return Number.isFinite(parsed) ? parsed : null;
}

function textProperty(text: string, property: string): string | null {
  const match = new RegExp(`${property}\\[([^\\]]*)\\]`, "i").exec(text);
  return match?.[1] ?? null;
}

function buildBrowserAnalysis(game: GameDto): AnalysisFrameDto[] {
  const frames: AnalysisFrameDto[] = [];
  for (let turn = 0; turn <= game.moves.length; turn += 1) {
    const trend = Math.sin(turn * 0.62) * 0.08 + Math.cos(turn * 0.21) * 0.045;
    const movePressure = turn > 0 && turn % 7 === 0 ? -0.08 : 0;
    const winrate = clamp(0.51 + trend + movePressure, 0.18, 0.82);
    frames.push({
      job_id: "browser-preview",
      game_id: game.summary.id,
      node_id: `turn-${turn}`,
      turn,
      visits: 800 + turn * 137,
      winrate_black: winrate,
      score_mean_black: (winrate - 0.5) * 28,
      score_stdev: 8.5,
      candidates: buildCandidates(game, turn, winrate)
    });
  }
  return frames;
}

function buildCandidates(game: GameDto, turn: number, winrate: number): CandidateMoveDto[] {
  const occupied = new Set(
    game.moves
      .slice(0, turn)
      .map((move) => (typeof move.vertex === "object" ? `${move.vertex.point.x}:${move.vertex.point.y}` : "pass"))
  );
  const candidates: CandidateMoveDto[] = [];
  let cursor = turn * 5 + 3;
  while (candidates.length < 8 && cursor < game.summary.board_width * game.summary.board_height * 3) {
    const x = (cursor * 7 + 3) % game.summary.board_width;
    const y = (cursor * 11 + 5) % game.summary.board_height;
    cursor += 1;
    if (occupied.has(`${x}:${y}`)) continue;
    const rank = candidates.length;
    const rankPenalty = rank * 0.018;
    const candidateWinrate = clamp(winrate - rankPenalty + Math.sin((turn + rank) * 0.9) * 0.012, 0.04, 0.96);
    candidates.push({
      vertex: { point: { x, y } },
      visits: Math.max(32, Math.round((1100 + turn * 92) / (rank + 1.35))),
      winrate_black: candidateWinrate,
      score_mean_black: (candidateWinrate - 0.5) * 28,
      policy_prior: clamp(0.34 - rank * 0.035, 0.03, 0.34),
      pv: [{ point: { x, y } }]
    });
  }
  return candidates;
}

function classifyProblemFrames(frames: AnalysisFrameDto[]): ProblemMarkerDto[] {
  return frames
    .slice(1)
    .map((frame, index) => {
      const previous = frames[index];
      const loss = Math.max(0, previous.winrate_black - frame.winrate_black);
      return { frame, loss };
    })
    .filter(({ loss }) => loss >= 0.045)
    .map(({ frame, loss }) => ({
      turn: frame.turn,
      severity: loss >= 0.12 ? "blunder" : loss >= 0.085 ? "mistake" : "inaccuracy",
      winrate_loss: loss,
      score_loss: loss * 28,
      label: loss >= 0.12 ? "Major drop" : loss >= 0.085 ? "Mistake" : "Inaccuracy"
    }));
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value));
}
