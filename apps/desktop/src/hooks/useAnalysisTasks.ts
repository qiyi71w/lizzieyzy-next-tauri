import { useRef, useState } from "react";
import type { AnalysisScopeDraft } from "../components/AnalysisTaskPanel";
import { analysisTaskSnapshot, cancelKataGoAnalysis, continueAnalysisTask, pauseAnalysisTask, previewAnalysisScope, startAnalysisTask, startKataGoGameAnalysis, nativeCurrentGameUnavailable } from "../api/backend";
import { admitsForegroundEngineJobs, admitsForegroundEngineQuery, runFromSnapshot } from "../domain/foregroundEngine";
import { admitsAnalysisAttachment, admitsWholeGameNodeResult, matchesWholeGameJobIdentity } from "../domain/analysisJob";
import { defaultAppPreferences, swingCriteriaError, taskConditionsError, taskStageConditionsError, type AppPreferences } from "../domain/preferences";
import type { AnalysisBranchChoiceDto, AnalysisFrameDto, AnalysisJobEventDto, AnalysisJobStartedDto, AnalysisScopeDto, AnalysisScopePreviewDto, AnalysisStageConditionsDto, AnalysisSwingCriteriaDto, AnalysisTaskDto, AnalysisTaskStrategyDto, CurrentGameResultDto, ForegroundEngineSnapshotDto, NodePath } from "../domain/types";

type Host = {
  nativeRuntime: boolean;
  currentGame: () => CurrentGameResultDto | null;
  engineSnapshot: () => ForegroundEngineSnapshotDto;
  branchChoices: () => AnalysisBranchChoiceDto[];
  committedPreferences: () => AppPreferences;
  matchOwnsWorkspace: () => boolean;
  trialActive: () => boolean;
  scoringActive: () => boolean;
  departurePending: () => boolean;
  departurePrompt: () => boolean;
  preferenceSaveBusy: () => boolean;
  savePreferences: (patch: Partial<AppPreferences>) => Promise<AppPreferences>;
  resolveMaxVisits: (requested: number) => number;
  message: (message: string) => void;
  acceptFrame: (path: NodePath, frame: AnalysisFrameDto) => void;
  refreshAttachment: () => Promise<void>;
};

type WholeGameProgress = { completed: number; expected: number; remaining: number };
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

export function useAnalysisTasks(input: Host) {
  const host = useRef(input);
  host.current = input;
  const { nativeRuntime } = input;
  const [wholeGameRunning, setWholeGameRunning] = useState(false);
  const [wholeGameProgress, setWholeGameProgress] = useState<WholeGameProgress | null>(null);
  const [analysisScopeDraft, setAnalysisScopeDraft] = useState<AnalysisScopeDraft>(defaultAnalysisScopeDraft);
  const [analysisScopePreview, setAnalysisScopePreview] = useState<AnalysisScopePreviewDto | null>(null);
  const [analysisTask, setAnalysisTask] = useState<AnalysisTaskDto | null>(null);
  const [analysisTaskError, setAnalysisTaskError] = useState<string | null>(null);
  const [analysisTaskRequestPending, setAnalysisTaskRequestPending] = useState(false);
  const wholeGameJobRef = useRef<AnalysisJobStartedDto | null>(null);
  const analysisTaskRef = useRef<AnalysisTaskDto | null>(null);
  const analysisTaskSnapshotRequestRef = useRef(0);
  const analysisScopePreviewRequestRef = useRef(0);
  const analysisTaskActionInFlightRef = useRef(false);
  const analysisTaskPauseFenceRef = useRef<{ taskId: string; jobId: string; continued: boolean } | null>(null);
  const analysisConditionsDraftEditedRef = useRef(false);

  function loadPreferences(loaded: AppPreferences) {
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
  }
  function acceptSnapshot(snapshot: ForegroundEngineSnapshotDto) {
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
  }
  function resetRunDisplay() {
    wholeGameJobRef.current = null;
    setWholeGameRunning(false);
    setWholeGameProgress(null);
  }

  function retireDocument() {
    resetRunDisplay();
    analysisTaskRef.current = null;
    analysisTaskPauseFenceRef.current = null;
    ++analysisTaskSnapshotRequestRef.current;
    setAnalysisTask(null);
    setAnalysisScopePreview(null);
    setAnalysisTaskError(null);
  }

  async function enterTemporaryMode(mode: "试下" | "计分") {
    setAnalysisScopePreview(null);
    try { adoptAnalysisTask(await analysisTaskSnapshot()); }
    catch (error) { host.current.message(`${mode}已进入；范围任务状态读取失败: ${errorMessage(error)}`); }
  }

  function clearWholeGameRunning() {
    wholeGameJobRef.current = null;
    setWholeGameRunning(false);
  }

  function analysisScopeFromDraft(draft: AnalysisScopeDraft): AnalysisScopeDto {
    const game = host.current.currentGame();
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
      branch_choices: host.current.branchChoices(),
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
            ? host.current.committedPreferences().taskSwingOverviewConditions
            : host.current.committedPreferences().taskOverviewConditions,
          swing
            ? host.current.committedPreferences().taskSwingDeepConditions
            : host.current.committedPreferences().taskDeepConditions
        ),
        ...(swing ? swingCriteriaDraft(host.current.committedPreferences().taskSwingCriteria) : {})
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
    const game = host.current.currentGame();
    if (!nativeRuntime || !game) {
      setAnalysisTaskError(nativeRuntime ? "Analysis preview requires a current game." : nativeCurrentGameUnavailable);
      return;
    }
    if (!admitsForegroundEngineJobs(host.current.engineSnapshot(), "whole_game_analysis")) {
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
        && admitsForegroundEngineJobs(host.current.engineSnapshot(), "whole_game_analysis")) setAnalysisScopePreview(preview);
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
    const snapshot = host.current.engineSnapshot();
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

  function explicitTaskReserved() {
    const task = analysisTaskRef.current;
    return isAnalysisTaskReserved(task) && task?.task_id !== host.current.engineSnapshot().auto_load?.task_id;
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
    const game = host.current.currentGame();
    if (!nativeRuntime || !game) throw new Error("Analysis task requires a current game.");
    const stages = input.overviewConditions ? [input.conditions, input.overviewConditions] : [input.conditions];
    assertTaskCapabilities(input.runId, input.swingCriteria, stages);
    if (explicitTaskReserved()) {
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
        ? JSON.stringify(input.overviewConditions) !== JSON.stringify(host.current.committedPreferences().taskOverviewConditions)
          || JSON.stringify(input.conditions) !== JSON.stringify(host.current.committedPreferences().taskDeepConditions)
        : input.strategy === "swing_selected_two_stage"
          ? JSON.stringify(input.overviewConditions) !== JSON.stringify(host.current.committedPreferences().taskSwingOverviewConditions)
            || JSON.stringify(input.conditions) !== JSON.stringify(host.current.committedPreferences().taskSwingDeepConditions)
            || JSON.stringify(input.swingCriteria) !== JSON.stringify(host.current.committedPreferences().taskSwingCriteria)
          : JSON.stringify(input.conditions) !== JSON.stringify(host.current.committedPreferences().taskSingleStageConditions);
      if (changed) {
        if (host.current.preferenceSaveBusy()) {
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
        await host.current.savePreferences(patch);
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
    host.current.message(`Analysis task ${started.state}: ${analysisTaskProgress(started)} completed.`);
    void refreshAnalysisTaskSnapshot();
  }

  async function handleStartAnalysisTask() {
    if (host.current.matchOwnsWorkspace()) return;
    if (host.current.trialActive() || host.current.scoringActive()) return;
    const run = runFromSnapshot(host.current.engineSnapshot());
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
      host.current.message(detail);
    } finally {
      setAnalysisTaskRequestPending(false);
    }
  }

  async function handleQuickAnalysisTask() {
    if (host.current.matchOwnsWorkspace()) return;
    if (host.current.trialActive() || host.current.scoringActive()) return;
    const run = runFromSnapshot(host.current.engineSnapshot());
    const game = host.current.currentGame();
    if (!run || !game || !admitsForegroundEngineJobs(host.current.engineSnapshot(), "whole_game_analysis") || host.current.departurePending() || explicitTaskReserved()) return;
    setAnalysisTaskRequestPending(true);
    setAnalysisTaskError(null);
    try {
      await runAnalysisTask({
        runId: run.run_id,
        scope: presetAnalysisScope(game.selected_path, host.current.branchChoices()),
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
      host.current.message(detail);
    } finally {
      setAnalysisTaskRequestPending(false);
    }
  }

  async function handleAllPositionsAnalysisTask() {
    if (host.current.matchOwnsWorkspace()) return;
    if (host.current.trialActive() || host.current.scoringActive()) return;
    const run = runFromSnapshot(host.current.engineSnapshot());
    const game = host.current.currentGame();
    if (!run || !game || !admitsForegroundEngineJobs(host.current.engineSnapshot(), "whole_game_analysis") || host.current.departurePending() || explicitTaskReserved()) return;
    setAnalysisTaskRequestPending(true);
    setAnalysisTaskError(null);
    try {
      await runAnalysisTask({
        runId: run.run_id,
        scope: presetAnalysisScope(game.selected_path, host.current.branchChoices()),
        strategy: "all_positions_two_stage",
        overviewConditions: host.current.committedPreferences().taskOverviewConditions,
        conditions: host.current.committedPreferences().taskDeepConditions
      });
    } catch (error) {
      const detail = errorMessage(error);
      setAnalysisTaskError(detail);
      host.current.message(detail);
    } finally {
      setAnalysisTaskRequestPending(false);
    }
  }

  async function handlePauseAnalysisTask() {
    const task = analysisTaskRef.current;
    const run = runFromSnapshot(host.current.engineSnapshot());
    if (analysisTaskActionInFlightRef.current
      || !task
      || (task.state !== "queued" && task.state !== "searching")
      || !run
      || run.run_id !== task.run_id
      || host.current.departurePending()
      || host.current.departurePrompt()) return;
    analysisTaskActionInFlightRef.current = true;
    setAnalysisTaskRequestPending(true);
    setAnalysisTaskError(null);
    ++analysisTaskSnapshotRequestRef.current;
    analysisTaskPauseFenceRef.current = { taskId: task.task_id, jobId: task.job_id, continued: false };
    wholeGameJobRef.current = null;
    try {
      const paused = await pauseAnalysisTask({ runId: task.run_id, taskId: task.task_id });
      adoptAnalysisTask(paused);
      host.current.message(`Analysis task ${paused.state}: ${analysisTaskProgress(paused)} completed.`);
    } catch (error) {
      analysisTaskPauseFenceRef.current = null;
      adoptAnalysisTask(task);
      const detail = errorMessage(error);
      setAnalysisTaskError(detail);
      host.current.message(`Pause failed: ${detail}`);
    } finally {
      analysisTaskActionInFlightRef.current = false;
      setAnalysisTaskRequestPending(false);
    }
    void refreshAnalysisTaskSnapshot();
  }

  async function handleContinueAnalysisTask() {
    if (host.current.matchOwnsWorkspace()) return;
    if (host.current.trialActive()) return;
    const task = analysisTaskRef.current;
    const run = runFromSnapshot(host.current.engineSnapshot());
    if (analysisTaskActionInFlightRef.current
      || task?.state !== "paused"
      || !run
      || run.run_id !== task.run_id
      || !admitsForegroundEngineJobs(host.current.engineSnapshot(), "whole_game_analysis")
      || host.current.departurePending()
      || host.current.departurePrompt()) return;
    analysisTaskActionInFlightRef.current = true;
    setAnalysisTaskRequestPending(true);
    setAnalysisTaskError(null);
    ++analysisTaskSnapshotRequestRef.current;
    try {
      assertTaskCapabilities(task.run_id, task.swing_criteria, task.overview_conditions ? [task.conditions, task.overview_conditions] : [task.conditions]);
      const continued = await continueAnalysisTask({ runId: task.run_id, taskId: task.task_id });
      adoptAnalysisTask(continued);
      host.current.message(`Analysis task ${continued.state}: ${analysisTaskProgress(continued)} completed.`);
    } catch (error) {
      const detail = errorMessage(error);
      setAnalysisTaskError(detail);
      host.current.message(`Continue failed: ${detail}`);
    } finally {
      analysisTaskActionInFlightRef.current = false;
      setAnalysisTaskRequestPending(false);
    }
    void refreshAnalysisTaskSnapshot();
  }

  function acceptJob(job: AnalysisJobEventDto) {
    const pending = wholeGameJobRef.current;
    if (!matchesWholeGameJobIdentity(pending, job)) return;
    if (job.outcome === "started" || job.outcome === "progress") {
      setWholeGameProgress({
        completed: job.completed ?? 0,
        expected: job.expected ?? 0,
        remaining: job.remaining ?? Math.max(0, (job.expected ?? 0) - (job.completed ?? 0))
      });
      if (job.outcome === "progress" && admitsWholeGameNodeResult(job) && job.frame) {
        host.current.acceptFrame(job.node_path, job.frame);
        if (admitsAnalysisAttachment(job)) void host.current.refreshAttachment();
      }
      return;
    }
    if (job.outcome === "completed" || job.outcome === "cancelled" || job.outcome === "failed" || job.outcome === "timeout") {
      if (job.outcome === "completed") {
        host.current.message(`整局分析完成 ${job.completed ?? 0}/${job.expected ?? 0}`);
      } else if (job.outcome === "cancelled") {
        host.current.message(job.failure?.message ?? "整局分析已取消");
      } else if (job.outcome === "failed") {
        host.current.message(job.failure?.message ?? "整局分析失败");
      } else {
        host.current.message("整局分析超时");
      }
      clearWholeGameRunning();
    }
  };

  async function handleAnalyzeKataGoGame(runId: string, maxVisits: number) {
    if (host.current.matchOwnsWorkspace()) return;
    const game = host.current.currentGame();
    if (!admitsForegroundEngineQuery(host.current.engineSnapshot(), "whole_game_analysis")
      || !admitsForegroundEngineJobs(host.current.engineSnapshot(), "visits_limit")
      || runFromSnapshot(host.current.engineSnapshot())?.run_id !== runId) {
      host.current.message("当前 run 不支持整谱分析所请求的 ownership、policy 或 visits 限制。");
      return;
    }
    if (explicitTaskReserved()) return;
    if (!nativeRuntime || !game) {
      host.current.message(nativeRuntime ? "整局分析需要当前游戏。" : nativeCurrentGameUnavailable);
      return;
    }
    const visits = host.current.resolveMaxVisits(maxVisits);
    try {
      const started = await startKataGoGameAnalysis({
        runId,
        generation: game.generation,
        maxVisits: visits
      });
      wholeGameJobRef.current = started;
      setWholeGameRunning(true);
      setWholeGameProgress(null);
      host.current.message(`Full-game KataGo analysis started (${started.job_id}).`);
      void refreshAnalysisTaskSnapshot();
    } catch (error) {
      host.current.message(errorMessage(error));
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
      host.current.message("Cancelling analysis task...");
      await cancelKataGoAnalysis(runId, jobId);
      await refreshAnalysisTaskSnapshot(true);
    } catch (error) {
      host.current.message(`Cancel failed: ${errorMessage(error)}`);
    } finally {
      analysisTaskActionInFlightRef.current = false;
      setAnalysisTaskRequestPending(false);
    }
  }
  return {
    view: { wholeGameRunning, wholeGameProgress, analysisScopeDraft, analysisScopePreview, analysisTask, analysisTaskError, analysisTaskRequestPending },
    isActionPending: () => analysisTaskActionInFlightRef.current,
    currentJobId: () => wholeGameJobRef.current?.job_id,
    explicitTaskReserved,
    changeDraft: handleAnalysisScopeDraftChange,
    preview: handlePreviewAnalysisScope,
    start: handleStartAnalysisTask,
    quick: handleQuickAnalysisTask,
    allPositions: handleAllPositionsAnalysisTask,
    pause: handlePauseAnalysisTask,
    continue: handleContinueAnalysisTask,
    cancel: handleCancelWholeGameAnalysis,
    startLegacy: handleAnalyzeKataGoGame,
    loadPreferences, acceptSnapshot, acceptJob, refresh: refreshAnalysisTaskSnapshot,
    resetRunDisplay, retireDocument, enterTemporaryMode
  };
}

function isAnalysisTaskReserved(task: AnalysisTaskDto | null): boolean {
  return task != null && (task.state === "queued"
    || task.state === "searching"
    || task.state === "pausing"
    || task.state === "paused");
}
function errorMessage(error: unknown): string {
  if (error instanceof Error) return error.message;
  if (error && typeof error === "object" && "message" in error) {
    const message = Reflect.get(error, "message");
    if (typeof message === "string" && message.trim()) return message;
  }
  return String(error);
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
function presetAnalysisScope(currentNode: NodePath, choices: AnalysisBranchChoiceDto[]): AnalysisScopeDto {
  return {
    mode: "first_child_mainline",
    current_node: currentNode,
    branch_choices: choices,
    interval: null,
    to_play: null
  };
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
