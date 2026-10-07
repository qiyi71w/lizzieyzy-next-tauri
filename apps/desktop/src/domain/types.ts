export type WorkspaceSharesDto = { left: number; right: number };

export type WindowGeometryDto = {
  x: number | null;
  y: number | null;
  width: number;
  height: number;
  scaleFactor: number;
  maximized: boolean;
};
export type WindowGeometryStatusDto = {
  phase: "loading" | "saved" | "pending" | "saving" | "unsaved";
  geometry: WindowGeometryDto | null;
  error: string | null;
};
export type WorkspaceVisibilityDto = { left: boolean; right: boolean };
export type MainWindowPinStatusDto = {
  durable: boolean;
  actual: boolean | null;
  error: string | null;
};

export type PlayerColor = "black" | "white";
export type PointDto = { x: number; y: number };
export type MoveVertex = { point: PointDto } | "pass";
export type MoveDto = { color: PlayerColor; vertex: MoveVertex; move_number: number };
export type StoneDto = PointDto & { color: PlayerColor };
export type PositionDto = {
  board_width: number;
  board_height: number;
  move_number: number;
  to_play: PlayerColor;
  stones: StoneDto[];
  captures_black: number;
  captures_white: number;
  last_move?: MoveDto | null;
  errors: string[];
};
export type GameSummaryDto = { id: string; board_width: number; board_height: number; komi: number; black_name?: string | null; white_name?: string | null; result?: string | null; move_count: number };
export type GameDto = { summary: GameSummaryDto; moves: MoveDto[] };
export type GameFileFormatDto = "sgf" | "gib";
export type GameFileImportDto = {
  format: GameFileFormatDto;
  sgf_text: string;
  display_path: string;
  display_name: string;
  native_path?: string | null;
  opened_path?: string | null;
};
export type FileActivationDeliveryDto =
  | { kind: "open"; request_id: number; path: string }
  | { kind: "rejected"; message: string };
export type FileActivationRejectionDto = { message: string };

export type NodePath = { indices: number[] };
export type SgfPropertyDto = { key: string; values: string[] };
export type SgfTreeNodeDto = { properties: SgfPropertyDto[]; children: SgfTreeNodeDto[] };
export type SgfMarkupDto =
  | { kind: "label"; point: PointDto; text: string }
  | { kind: "circle" | "square" | "cross" | "triangle"; point: PointDto };
export type SgfMarkupToolDto =
  | { kind: "label"; text: string }
  | { kind: "letters" | "numbers" | "circle" | "square" | "cross" | "triangle" | "erase" };
export type SgfMarkupActionDto =
  | { kind: "clear" }
  | { kind: "point"; point: PointDto; tool: SgfMarkupToolDto };

export type SelectedNodeSnapshotDto = {
  path: NodePath;
  position: PositionDto;
  stone_move_numbers: MoveDto[];
  personal_comment: string;
  markup: SgfMarkupDto[];
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
  can_undo: boolean;
  can_redo: boolean;
  dirty: boolean;
  native_path?: string | null;
};
export type TrialSessionDto = {
  session_id: number;
  revision: number;
  entry_path: NodePath;
  tree: SgfTreeNodeDto;
  selected_path: NodePath;
  snapshot: SelectedNodeSnapshotDto;
  can_undo: boolean;
};
export type ScoringRuleDto = "area" | "territory";
export type AreaCompensationDto = "none" | "handicap" | "handicap_minus_one";
export type ScoringActionDto =
  | { kind: "point"; point: PointDto }
  | { kind: "settings"; rule: ScoringRuleDto; compensation: AreaCompensationDto; handicap: number };
export type ScoringSessionDto = {
  session_id: number;
  revision: number;
  entry_path: NodePath;
  generation: number;
  position: PositionDto;
  dead: PointDto[];
  neutral: PointDto[];
  ownership: (PlayerColor | null)[];
  rule: ScoringRuleDto;
  compensation: AreaCompensationDto;
  handicap: number;
  komi: number;
  black_stones: number;
  white_stones: number;
  black_territory: number;
  white_territory: number;
  black_dead: number;
  white_dead: number;
  black_total: string;
  white_total: string;
  result: string;
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
export type AnalysisFrameDto = { job_id: string; game_id?: string | null; node_id?: string | null; turn: number; visits: number; winrate_black: number; score_mean_black?: number | null; score_stdev?: number | null; candidates: CandidateMoveDto[]; ownership?: number[] | null; policy?: number[] | null };
export type ProblemMarkerDto = { turn: number; severity: "info" | "inaccuracy" | "mistake" | "blunder"; winrate_loss: number; score_loss?: number | null; label: string };
export type EngineBackendDto = "kata_go_analysis" | "generic_gtp";
export type KataGoSettingsDto = { model_path: string | null; config_path: string | null; max_visits: number };
export type EngineProfileDto = {
  name: string;
  program: string;
  argv: string[];
  working_dir: string | null;
} & (
  | { adapter_kind: "kata_go_analysis"; settings: KataGoSettingsDto }
  | { adapter_kind: "generic_gtp"; settings: Record<string, never> }
);
export type EngineProfileRecordDto = { id: string; profile: EngineProfileDto };
export type EngineProfilesSettingsDto = { version: number; selected_profile_id: string; autoload_profile_id: string | null; profiles: EngineProfileRecordDto[] };
export type EngineProfileOrderRequestDto = { expected_profile_ids: string[]; profile_ids: string[] };
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
  | "command"
  | "process_exit"
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
export type EngineGtpFactsDto = {
  protocol_version: number;
  name: string;
  version: string;
  commands: string[];
};
export type EngineCapabilitySnapshotDto = {
  adapter_kind: EngineBackendDto;
  analysis?: EngineAnalysisCapabilitiesDto | null;
  game_move?: boolean;
  gtp?: EngineGtpFactsDto | null;
};
export type EngineAnalysisCapabilitiesDto = {
  selected_node_analysis: boolean;
  continuous_analysis: boolean;
  whole_game_analysis: boolean;
  candidates: boolean;
  pv: boolean;
  winrate: boolean;
  root_score: boolean;
  ownership: boolean;
  policy: boolean;
  visits_limit: boolean;
  protocol_cancel: boolean;
};
export type EngineRunDto = {
  run_id: string;
  profile_id: string;
  adapter_kind: EngineBackendDto;
  profile_snapshot: EngineProfileDto;
  capability_snapshot?: EngineCapabilitySnapshotDto | null;
};
export type ExactRulesDto = "chinese" | "chinese_kgs";
export type ExactPositionDto = {
  board_width: number;
  board_height: number;
  komi: number;
  rules: ExactRulesDto;
  initial_player: PlayerColor;
  to_play: PlayerColor;
  initial_stones: StoneDto[];
  moves: MoveDto[];
};
export type ComputeBudgetDto = { deadline_ms: number; max_visits: number | null };
export type GameMoveRequestDto = {
  run_id: string;
  generation: number;
  node_path: NodePath;
  budget: ComputeBudgetDto;
};
export type GameMoveJobDto = {
  run_id: string;
  job_id: string;
  generation: number;
  node_path: NodePath;
};
export type GameMoveDto = { kind: "move"; vertex: MoveVertex } | { kind: "resign" };
export type GameMoveResultDto = GameMoveJobDto & {
  result: GameMoveDto;
  engine_time_mapped: boolean;
};

export type PkSideDefaultsDto = { profile_id: string | null; deadline_ms: number; kata_max_visits: number };
export type MatchModeDto = "human" | "pk";
export type MatchDefaultsDto = {
  board_size: number;
  komi: number;
  handicap: number;
  human_color: PlayerColor;
  rules: ExactRulesDto | null;
  profile_id: string | null;
  deadline_ms: number;
  kata_max_visits: number;
  pk_black: PkSideDefaultsDto;
  pk_white: PkSideDefaultsDto;
  pk_max_moves: number;
};
export type MatchPhaseDto = "idle" | "starting" | "playing" | "paused" | "ending" | "error";
export type MatchEndDto = "stopped" | "resigned" | "two_passes" | "failed" | "move_limit";
export type MatchAnalysisPolicyDto = "off" | "human_turn" | "engine_turn" | "both";
export type MatchAnalysisFrameDto = { turn: MatchTurnDto; epoch: number; job: GameMoveJobDto; frame: AnalysisFrameDto };
export type MatchAnalysisDto = { supported: boolean; policy: MatchAnalysisPolicyDto; epoch: number; frame: MatchAnalysisFrameDto | null };
export type MatchSnapshotDto = {
  revision: number;
  phase: MatchPhaseDto;
  mode: MatchModeDto;
  session_id: string | null;
  turn: number;
  to_play: PlayerColor | null;
  settings: MatchDefaultsDto | null;
  run_id: string | null;
  pk_runs: [EngineRunDto, EngineRunDto] | null;
  job: GameMoveJobDto | null;
  end: MatchEndDto | null;
  failure: EngineFailureDto | null;
  failed_side: PlayerColor | null;
  resources_held: boolean;
  committed: boolean;
  committed_moves: number;
  pause_pending: boolean;
  /** Sides whose cancelled GTP run Pause reaped; explicit Resume rebuilds them from the original snapshot. */
  rebuild_sides: PlayerColor[];
  resume_pending: boolean;
  analysis: MatchAnalysisDto;
};
export type MatchTurnDto = { session_id: string; turn: number; generation: number; node_path: NodePath };
export type HumanMatchActionDto = { kind: "play"; vertex: MoveVertex } | { kind: "resign" };
export type MatchUpdateDto = { match_state: MatchSnapshotDto; current: CurrentGameResultDto | null };
export type MatchStartDto =
  | { kind: "new"; discard_confirmed: boolean }
  | { kind: "continue"; node_path: NodePath; root_metadata_confirmed: boolean };
export type HumanMatchStartDto = { settings: MatchDefaultsDto; generation: number; snapshot_seq: number; start: MatchStartDto };
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
  game_move_job?: GameMoveJobDto | null;
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
export type AnalysisTaskStrategyDto =
  | "single_stage"
  | "all_positions_two_stage"
  | "swing_selected_two_stage";
export type AnalysisTaskStageDto = "single_stage" | "overview" | "deep";
export type AnalysisMoveActorFilterDto = "both" | "black" | "white";
export type AnalysisSwingThresholdDto = { enabled: boolean; value: number };
export type AnalysisSwingCriteriaDto = {
  move_actors: AnalysisMoveActorFilterDto;
  winrate_change_percentage_points: AnalysisSwingThresholdDto;
  score_change_points: AnalysisSwingThresholdDto;
};
export type AnalysisSwingComparisonDto = {
  before: NodePath;
  after: NodePath;
  move_actor: PlayerColor;
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
  supporting_targets: AnalysisScopeTargetDto[];
  swing_comparisons: AnalysisSwingComparisonDto[];
  swing_criteria?: AnalysisSwingCriteriaDto | null;
};
export type AnalysisTaskStateDto =
  | "queued"
  | "searching"
  | "pausing"
  | "paused"
  | "completed"
  | "cancelled"
  | "failed"
  | "invalidated";
export type AnalysisTaskOverviewDto = {
  node_path: NodePath;
  frame: AnalysisFrameDto;
};
export type AnalysisTaskDto = {
  task_id: string;
  run_id: string;
  job_id: string;
  generation: number;
  scope: AnalysisScopeDto;
  strategy: AnalysisTaskStrategyDto;
  stage: AnalysisTaskStageDto;
  conditions: AnalysisStageConditionsDto;
  overview_conditions?: AnalysisStageConditionsDto | null;
  requested: NodePath[];
  supporting: NodePath[];
  swing_comparisons: AnalysisSwingComparisonDto[];
  swing_criteria?: AnalysisSwingCriteriaDto | null;
  selected_for_deep?: NodePath[] | null;
  overview_completed: NodePath[];
  completed: NodePath[];
  overview_summaries: AnalysisTaskOverviewDto[];
  state: AnalysisTaskStateDto;
  reason?: string | null;
  ending_conditions: string[];
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
