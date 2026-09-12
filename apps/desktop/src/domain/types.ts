export type PlayerColor = "black" | "white";
export type PointDto = { x: number; y: number };
export type MoveVertex = { point: PointDto } | "pass";
export type MoveDto = { color: PlayerColor; vertex: MoveVertex; move_number: number };
export type StoneDto = PointDto & { color: PlayerColor };
export type PositionDto = {
  board_size: number;
  move_number: number;
  to_play: PlayerColor;
  stones: StoneDto[];
  captures_black: number;
  captures_white: number;
  last_move?: MoveDto | null;
  errors: string[];
};
export type GameSummaryDto = { id: string; board_size: number; komi: number; black_name?: string | null; white_name?: string | null; result?: string | null; move_count: number };
export type GameDto = { summary: GameSummaryDto; moves: MoveDto[] };
export type NodePath = { indices: number[] };
export type SgfPropertyDto = { key: string; values: string[] };
export type SgfTreeNodeDto = { properties: SgfPropertyDto[]; children: SgfTreeNodeDto[] };
export type SelectedNodeSnapshotDto = {
  path: NodePath;
  position: PositionDto;
  personal_comment: string;
  generated_information?: string | null;
  primary_analysis?: AnalysisFrameDto | null;
  secondary_analysis?: AnalysisFrameDto | null;
};
export type CurrentGameResultDto = {
  tree: SgfTreeNodeDto;
  selected_path: NodePath;
  snapshot: SelectedNodeSnapshotDto;
  /** Semantic position/tree identity; comments, navigation and Save preserve it. */
  generation: number;
  snapshot_seq: number;
  dirty: boolean;
  native_path?: string | null;
};
export type CurrentGameErrorKind =
  | "no_current_game"
  | "invalid_node_path"
  | "malformed_sgf"
  | "unsupported_board_size"
  | "occupied_point"
  | "suicide"
  | "simple_ko"
  | "root_removal"
  | "departure_in_progress"
  | "departure_blocked";
export type CurrentGameError = { kind: CurrentGameErrorKind; message: string };
export type DocumentDepartureAdmissionDto =
  | { status: "needs_decision"; departure_id: number }
  | { status: "ready"; departure_id: number };
export type DocumentDepartureActionDto = "save" | "discard" | "cancel";
export type DocumentDepartureOutcomeDto = {
  committed: boolean;
  analysis_stopped: boolean;
  current?: CurrentGameResultDto | null;
  message: string;
};
export type ApplicationExitActionDto = "save" | "discard" | "cancel" | "continue";
export type ApplicationExitDispositionDto = "clean_completed" | "explicit_discard" | "exit_incomplete";
export type ApplicationTeardownAttemptDto =
  | { status: "completed" }
  | { status: "timed_out"; outstanding: string[] };
export type ApplicationExitOutcomeDto = {
  committed: boolean;
  analysis_stopped: boolean;
  current?: CurrentGameResultDto | null;
  message: string;
  disposition?: ApplicationExitDispositionDto | null;
  teardown?: ApplicationTeardownAttemptDto | null;
  recovery_persist_error?: string | null;
};
export type CandidateMoveDto = { vertex: MoveVertex; visits: number; winrate_black: number; score_mean_black: number; policy_prior?: number | null; pv: MoveVertex[] };
export type AnalysisFrameDto = { job_id: string; game_id?: string | null; node_id?: string | null; turn: number; visits: number; winrate_black: number; score_mean_black: number; score_stdev?: number | null; candidates: CandidateMoveDto[]; ownership?: number[] | null; policy?: number[] | null };
export type ProblemMarkerDto = { turn: number; severity: "info" | "inaccuracy" | "mistake" | "blunder"; winrate_loss: number; score_loss: number; label: string };
export type EngineBackendDto = "kata_go_analysis";
export type EngineProfileDto = { name: string; engine_path: string; model_path?: string | null; config_path?: string | null; working_dir?: string | null; backend: EngineBackendDto };
export type EngineProfileSettingsDto = { profile: EngineProfileDto; max_visits: number };
export type EngineProfileRecordDto = { id: string; profile: EngineProfileDto; max_visits: number };
export type EngineProfilesSettingsDto = { selected_profile_id: string; autoload_profile_id?: string | null; profiles: EngineProfileRecordDto[] };
export type AssetCheckDto = { path: string; exists: boolean; required: boolean; label: string };
export type AppHealthDto = { app: string; architecture: string; rust_backend_ready: boolean; notes: string[] };

export type EngineOperationDto =
  | "start"
  | "stop"
  | "restart"
  | "switch"
  | "autoload"
  | "teardown"
  | "job"
  | "delete_profile"
  | "unexpected_exit";
export type EngineFailureKind =
  | "start"
  | "asset"
  | "readiness"
  | "protocol"
  | "nonzero_exit"
  | "timeout"
  | "cancellation"
  | "unsupported_capability"
  | "invalid_state"
  | "occupied"
  | "profile_not_found"
  | "profile_in_use";
export type EngineFailureDto = {
  operation: EngineOperationDto;
  run_id?: string | null;
  switch_id?: string | null;
  job_id?: string | null;
  profile_id?: string | null;
  kind: EngineFailureKind;
  message: string;
  diagnostic_summary?: string | null;
};
export type EngineCapabilitySnapshotDto = {
  adapter_kind: EngineBackendDto;
  selected_node_analysis: boolean;
  whole_game_analysis: boolean;
  protocol_cancel: boolean;
};
export type EngineRunDto = {
  run_id: string;
  profile_id: string;
  adapter_kind: EngineBackendDto;
  profile_snapshot: EngineProfileDto;
  capability_snapshot?: EngineCapabilitySnapshotDto | null;
};
export type ForegroundEngineLifecycleDto =
  | { state: "no_engine"; failure?: EngineFailureDto }
  | { state: "starting"; run: EngineRunDto }
  | { state: "ready"; run: EngineRunDto }
  | { state: "switching"; primary: EngineRunDto; candidate: EngineRunDto; switch_id: string }
  | { state: "stopping"; run: EngineRunDto }
  | { state: "error"; run: EngineRunDto; failure: EngineFailureDto };
export type ContinuousAnalysisBudgetDto = {
  continuousTimeLimitEnabled: boolean;
  continuousTimeLimitSeconds: number;
  continuousVisitsLimitEnabled: boolean;
  continuousVisitsLimit: number;
  continuousStopOnEmptyBoard: boolean;
};
export type ContinuousAnalysisPhaseDto =
  | "loading"
  | "off"
  | "waiting"
  | "unavailable"
  | "queued"
  | "searching"
  | "stopping"
  | "time_limited"
  | "visits_limited"
  | "empty_board"
  | "finite"
  | "paused"
  | "error"
  | "safety_hold"
  | "departing";
export type ContinuousAnalysisSnapshotDto = {
  enabled: boolean | null;
  phase: ContinuousAnalysisPhaseDto;
};
export type ForegroundEngineSnapshotDto = {
  revision: number;
  lifecycle: ForegroundEngineLifecycleDto;
  continuous: ContinuousAnalysisSnapshotDto;
  selected_node_job?: AnalysisJobStartedDto | null;
  whole_game_job?: AnalysisJobStartedDto | null;
};
export type AnalysisJobLaneDto = "selected_node" | "whole_game";
export type AnalysisJobModeDto = "finite" | "continuous";
export type AnalysisJobStateDto = "queued" | "searching" | "stopping" | "time_limited" | "visits_limited";
export type AnalysisJobOutcomeDto =
  | "started"
  | "progress"
  | "completed"
  | "cancelled"
  | "superseded"
  | "timeout"
  | "stopping"
  | "time_limited"
  | "visits_limited"
  | "failed";
export type AnalysisJobStartedDto = {
  run_id: string;
  job_id: string;
  lane: AnalysisJobLaneDto;
  mode: AnalysisJobModeDto;
  state: AnalysisJobStateDto;
  generation: number;
  node_path: NodePath;
};
export type AnalysisPublicationScopeDto = {
  run_id: string;
  job_id: string;
  generation: number;
  node_path: NodePath;
};
export type AnalysisJobEventDto = {
  run_id: string;
  job_id: string;
  lane: AnalysisJobLaneDto;
  current_game?: CurrentGameResultDto | null;
  mode: AnalysisJobModeDto;
  generation: number;
  node_path: NodePath;
  outcome: AnalysisJobOutcomeDto;
  completed?: number | null;
  expected?: number | null;
  remaining?: number | null;
  frame?: AnalysisFrameDto | null;
  failure?: EngineFailureDto | null;
};
export type ForegroundEngineEventDto =
  | { type: "snapshot"; snapshot: ForegroundEngineSnapshotDto }
  | { type: "failure"; failure: EngineFailureDto }
  | { type: "job"; job: AnalysisJobEventDto };

export type AnalysisScopeModeDto =
  | "current_node"
  | "selected_review_line"
  | "first_child_mainline"
  | "all_branches";
export type AnalysisBranchChoiceDto = { parent: NodePath; child: number };
export type AnalysisPositionIntervalDto = { start: number; end: number };
export type AnalysisScopeDto = {
  mode: AnalysisScopeModeDto;
  current_node: NodePath;
  branch_choices: AnalysisBranchChoiceDto[];
  interval?: AnalysisPositionIntervalDto | null;
  to_play?: PlayerColor | null;
};
export type AnalysisTaskLimitDto = { enabled: boolean; value: number };
export type AnalysisStageConditionsDto = {
  time_seconds: AnalysisTaskLimitDto;
  total_visits: AnalysisTaskLimitDto;
  leading_candidate_visits: AnalysisTaskLimitDto;
};
export type AnalysisScopeTargetDto = {
  node_path: NodePath;
  move_number: number;
  to_play: PlayerColor;
};
export type AnalysisScopePreviewDto = {
  generation: number;
  scope: AnalysisScopeDto;
  targets: AnalysisScopeTargetDto[];
};
export type AnalysisTaskStateDto =
  | "queued"
  | "searching"
  | "completed"
  | "cancelled"
  | "failed"
  | "invalidated";
export type AnalysisTaskDto = {
  task_id: string;
  run_id: string;
  job_id: string;
  generation: number;
  scope: AnalysisScopeDto;
  stage: string;
  conditions: AnalysisStageConditionsDto;
  requested: NodePath[];
  completed: NodePath[];
  state: AnalysisTaskStateDto;
  reason?: string | null;
};

export type RecoveryEnvelopeDto = {
  document_seq: number;
  snapshot_seq: number;
  sgf_text: string;
  selected_path: NodePath;
  source_path?: string | null;
  dirty: boolean;
  disposition: ApplicationExitDispositionDto;
};
export type RecoveryProtectionDto =
  | { status: "protected" }
  | { status: "unprotected"; message: string };
export type RecoveryStartupDto =
  | { status: "none" }
  | { status: "abnormal"; envelope: RecoveryEnvelopeDto }
  | { status: "normal"; envelope: RecoveryEnvelopeDto }
  | { status: "unreadable"; message: string };
