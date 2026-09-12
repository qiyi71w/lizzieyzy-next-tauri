import type {
  AnalysisMoveActorFilterDto,
  AnalysisScopeModeDto,
  AnalysisScopePreviewDto,
  AnalysisTaskDto,
  AnalysisTaskStrategyDto,
  PlayerColor
} from "../domain/types";

export type AnalysisScopeDraft = {
  strategy: AnalysisTaskStrategyDto;
  mode: AnalysisScopeModeDto;
  intervalEnabled: boolean;
  intervalStart: string;
  intervalEnd: string;
  toPlay: PlayerColor | "both";
  moveActors: AnalysisMoveActorFilterDto;
  winrateChangeEnabled: boolean;
  winrateChangeThreshold: string;
  scoreChangeEnabled: boolean;
  scoreChangeThreshold: string;
  overviewTimeEnabled: boolean;
  overviewTimeSeconds: string;
  overviewTotalVisitsEnabled: boolean;
  overviewTotalVisits: string;
  overviewLeadingCandidateVisitsEnabled: boolean;
  overviewLeadingCandidateVisits: string;
  singleTimeEnabled: boolean;
  singleTimeSeconds: string;
  singleTotalVisitsEnabled: boolean;
  singleTotalVisits: string;
  singleLeadingCandidateVisitsEnabled: boolean;
  singleLeadingCandidateVisits: string;
  timeEnabled: boolean;
  timeSeconds: string;
  totalVisitsEnabled: boolean;
  totalVisits: string;
  leadingCandidateVisitsEnabled: boolean;
  leadingCandidateVisits: string;
};

type Props = {
  draft: AnalysisScopeDraft;
  preview: AnalysisScopePreviewDto | null;
  task: AnalysisTaskDto | null;
  canRun: boolean;
  busy: boolean;
  error: string | null;
  onDraftChange: (draft: AnalysisScopeDraft) => void;
  onPreview: () => void;
  onStart: () => void;
  onPause: () => void;
  onContinue: () => void;
  onCancel: () => void;
};

const modes: Array<{ value: AnalysisScopeModeDto; label: string }> = [
  { value: "current_node", label: "Current node" },
  { value: "selected_review_line", label: "Selected Review Line" },
  { value: "first_child_mainline", label: "First-child mainline" },
  { value: "all_branches", label: "All branches" }
];

export function AnalysisTaskPanel(props: Props) {
  const reserved = props.task != null && ["queued", "searching", "pausing", "paused"].includes(props.task.state);
  const pauseable = props.task?.state === "queued" || props.task?.state === "searching";
  const continuable = props.task?.state === "paused";
  const completed = props.task?.completed.length ?? 0;
  const requested = props.task?.requested.length ?? 0;
  const overviewExpected = requested + (props.task?.supporting.length ?? 0);
  const deepTargets = props.task?.selected_for_deep ?? props.task?.requested ?? [];
  const deepExpected = deepTargets.length;
  const requestedDeepTargets = deepTargets.filter((path) =>
    props.task?.requested.some((requestedPath) => sameNodePath(path, requestedPath))
  );
  const supportingDeepTargets = deepTargets.filter((path) =>
    props.task?.supporting.some((supportingPath) => sameNodePath(path, supportingPath))
  );
  const requestedDeepCompleted = requestedDeepTargets.filter((path) =>
    props.task?.completed.some((completedPath) => sameNodePath(path, completedPath))
  ).length;
  const supportingDeepCompleted = supportingDeepTargets.filter((path) =>
    props.task?.completed.some((completedPath) => sameNodePath(path, completedPath))
  ).length;
  const requestedOverviewCompleted = props.task?.requested.filter((path) =>
    props.task?.overview_completed.some((completedPath) => sameNodePath(path, completedPath))
  ).length ?? 0;
  const supportingOverviewCompleted = props.task?.supporting.filter((path) =>
    props.task?.overview_completed.some((completedPath) => sameNodePath(path, completedPath))
  ).length ?? 0;
  const first = props.preview?.targets[0];
  const last = props.preview?.targets.at(-1);
  const supportingFirst = props.preview?.supporting_targets[0];
  const supportingLast = props.preview?.supporting_targets.at(-1);
  const singleStage = props.draft.strategy === "single_stage";
  const swingSelected = props.draft.strategy === "swing_selected_two_stage";

  function update(patch: Partial<AnalysisScopeDraft>) {
    props.onDraftChange({ ...props.draft, ...patch });
  }

  return (
    <section className="sheet-row analysis-task-panel" aria-label="Analysis task">
      <div className="sgf-tools">
        <div className="document-row">
          <strong>Analysis task</strong>
          <label>
            Strategy
            <select
              aria-label="Analysis strategy"
              value={props.draft.strategy}
              onChange={(event) => update({ strategy: event.target.value as AnalysisTaskStrategyDto })}
            >
              <option value="all_positions_two_stage">All positions · overview then deep</option>
              <option value="swing_selected_two_stage">Swing-selected · overview then deep</option>
              <option value="single_stage">Single stage</option>
            </select>
          </label>
          <label>
            Scope
            <select
              aria-label="Analysis scope"
              value={props.draft.mode}
              onChange={(event) => update({ mode: event.target.value as AnalysisScopeModeDto })}
            >
              {modes.map((mode) => <option key={mode.value} value={mode.value}>{mode.label}</option>)}
            </select>
          </label>
          <label>
            <input
              aria-label="Use position interval"
              type="checkbox"
              checked={props.draft.intervalEnabled}
              onChange={(event) => update({ intervalEnabled: event.target.checked })}
            />
            Position interval
          </label>
          <label>
            Start
            <input
              aria-label="Position interval start"
              type="number"
              min={0}
              step={1}
              disabled={!props.draft.intervalEnabled}
              value={props.draft.intervalStart}
              onChange={(event) => update({ intervalStart: event.target.value })}
            />
          </label>
          <label>
            End
            <input
              aria-label="Position interval end"
              type="number"
              min={0}
              step={1}
              disabled={!props.draft.intervalEnabled}
              value={props.draft.intervalEnd}
              onChange={(event) => update({ intervalEnd: event.target.value })}
            />
          </label>
          <label>
            To play
            <select
              aria-label="Side to play"
              value={props.draft.toPlay}
              onChange={(event) => update({ toPlay: event.target.value as PlayerColor | "both" })}
            >
              <option value="both">Both</option>
              <option value="black">Black</option>
              <option value="white">White</option>
            </select>
          </label>
          {swingSelected ? (
            <fieldset>
              <legend>Swing selection (enabled thresholds use OR)</legend>
              <label>
                Move actor
                <select
                  aria-label="Swing move actor"
                  value={props.draft.moveActors}
                  onChange={(event) => update({ moveActors: event.target.value as AnalysisMoveActorFilterDto })}
                >
                  <option value="both">Both</option>
                  <option value="black">Black</option>
                  <option value="white">White</option>
                </select>
              </label>
              <label>
                <input aria-label="Enable winrate swing threshold" type="checkbox" checked={props.draft.winrateChangeEnabled} onChange={(event) => update({ winrateChangeEnabled: event.target.checked })} />
                Winrate change (percentage points)
                <input aria-label="Winrate swing threshold" type="number" min="0.000001" step="any" value={props.draft.winrateChangeThreshold} onChange={(event) => update({ winrateChangeThreshold: event.target.value })} />
              </label>
              <label>
                <input aria-label="Enable score swing threshold" type="checkbox" checked={props.draft.scoreChangeEnabled} onChange={(event) => update({ scoreChangeEnabled: event.target.checked })} />
                Score change (points)
                <input aria-label="Score swing threshold" type="number" min="0.000001" step="any" value={props.draft.scoreChangeThreshold} onChange={(event) => update({ scoreChangeThreshold: event.target.value })} />
              </label>
            </fieldset>
          ) : null}
          {!singleStage ? (
            <fieldset>
              <legend>Overview conditions (any enabled condition ends the search)</legend>
              <label>
                <input aria-label="Enable overview search time" type="checkbox" checked={props.draft.overviewTimeEnabled} onChange={(event) => update({ overviewTimeEnabled: event.target.checked })} />
                Time (seconds)
                <input aria-label="Overview search time seconds" type="number" min={1} max={4294967295} step={1} value={props.draft.overviewTimeSeconds} onChange={(event) => update({ overviewTimeSeconds: event.target.value })} />
              </label>
              <label>
                <input aria-label="Enable overview total visits" type="checkbox" checked={props.draft.overviewTotalVisitsEnabled} onChange={(event) => update({ overviewTotalVisitsEnabled: event.target.checked })} />
                Total visits
                <input aria-label="Overview total visits" type="number" min={1} max={4294967295} step={1} value={props.draft.overviewTotalVisits} onChange={(event) => update({ overviewTotalVisits: event.target.value })} />
              </label>
              <label>
                <input aria-label="Enable overview leading candidate visits" type="checkbox" checked={props.draft.overviewLeadingCandidateVisitsEnabled} onChange={(event) => update({ overviewLeadingCandidateVisitsEnabled: event.target.checked })} />
                Leading candidate visits
                <input aria-label="Overview leading candidate visits" type="number" min={1} max={4294967295} step={1} value={props.draft.overviewLeadingCandidateVisits} onChange={(event) => update({ overviewLeadingCandidateVisits: event.target.value })} />
              </label>
            </fieldset>
          ) : null}
          <fieldset>
            <legend>{singleStage ? "Single-stage" : "Deep"} conditions (any enabled condition ends the search)</legend>
            <label>
              <input
                aria-label="Enable search time"
                type="checkbox"
                checked={singleStage ? props.draft.singleTimeEnabled : props.draft.timeEnabled}
                onChange={(event) => singleStage
                  ? update({ singleTimeEnabled: event.target.checked })
                  : update({ timeEnabled: event.target.checked })}
              />
              Time (seconds)
              <input
                aria-label="Search time seconds"
                type="number"
                min={1}
                max={4294967295}
                step={1}
                value={singleStage ? props.draft.singleTimeSeconds : props.draft.timeSeconds}
                onChange={(event) => singleStage
                  ? update({ singleTimeSeconds: event.target.value })
                  : update({ timeSeconds: event.target.value })}
              />
            </label>
            <label>
              <input
                aria-label="Enable total visits"
                type="checkbox"
                checked={singleStage ? props.draft.singleTotalVisitsEnabled : props.draft.totalVisitsEnabled}
                onChange={(event) => singleStage
                  ? update({ singleTotalVisitsEnabled: event.target.checked })
                  : update({ totalVisitsEnabled: event.target.checked })}
              />
              Total visits
              <input
                aria-label="Total visits"
                type="number"
                min={1}
                max={4294967295}
                step={1}
                value={singleStage ? props.draft.singleTotalVisits : props.draft.totalVisits}
                onChange={(event) => singleStage
                  ? update({ singleTotalVisits: event.target.value })
                  : update({ totalVisits: event.target.value })}
              />
            </label>
            <label>
              <input
                aria-label="Enable leading candidate visits"
                type="checkbox"
                checked={singleStage
                  ? props.draft.singleLeadingCandidateVisitsEnabled
                  : props.draft.leadingCandidateVisitsEnabled}
                onChange={(event) => singleStage
                  ? update({ singleLeadingCandidateVisitsEnabled: event.target.checked })
                  : update({ leadingCandidateVisitsEnabled: event.target.checked })}
              />
              Leading candidate visits
              <input
                aria-label="Leading candidate visits"
                type="number"
                min={1}
                max={4294967295}
                step={1}
                value={singleStage
                  ? props.draft.singleLeadingCandidateVisits
                  : props.draft.leadingCandidateVisits}
                onChange={(event) => singleStage
                  ? update({ singleLeadingCandidateVisits: event.target.value })
                  : update({ leadingCandidateVisits: event.target.value })}
              />
            </label>
          </fieldset>
        </div>
        <div className="button-row">
          <button type="button" onClick={props.onPreview} disabled={!props.canRun || props.busy}>Preview scope</button>
          <button type="button" onClick={props.onStart} disabled={!props.canRun || props.busy || reserved || !props.preview}>Start task</button>
          <button type="button" onClick={props.onPause} disabled={!props.canRun || props.busy || !pauseable}>Pause task</button>
          <button type="button" onClick={props.onContinue} disabled={!props.canRun || props.busy || !continuable}>Continue task</button>
          <button type="button" onClick={props.onCancel} disabled={props.busy || !reserved}>Cancel task</button>
        </div>
        {props.preview ? (
          <p role="status">
            {props.preview.targets.length} requested targets
            {props.preview.supporting_targets.length > 0
              ? ` · ${props.preview.supporting_targets.length} supporting predecessor targets`
              : ""}
            {supportingFirst ? ` · supporting first ${formatTarget(supportingFirst)}` : ""}
            {supportingLast && supportingLast !== supportingFirst
              ? ` · supporting last ${formatTarget(supportingLast)}`
              : ""}
            {first ? ` · first ${formatTarget(first)}` : ""}
            {last && last !== first ? ` · last ${formatTarget(last)}` : ""}
          </p>
        ) : null}
        {props.error ? <p role="alert">{props.error}</p> : null}
        {props.task ? (
          <p role="status" data-analysis-task-state={props.task.state}>
            {formatTaskStage(props.task.stage)} · {props.task.state}
            {props.task.strategy === "swing_selected_two_stage"
              ? ` · overview ${props.task.overview_completed.length}/${overviewExpected} (requested ${requestedOverviewCompleted}/${requested}, supporting ${supportingOverviewCompleted}/${props.task.supporting.length}) · deep ${completed}/${deepExpected} (requested ${requestedDeepCompleted}/${requestedDeepTargets.length}, supporting ${supportingDeepCompleted}/${supportingDeepTargets.length})`
              : props.task.strategy === "all_positions_two_stage"
                ? ` · overview ${props.task.overview_completed.length}/${overviewExpected} · deep ${completed}/${deepExpected}`
                : ` · ${completed}/${requested} completed`}
            {props.task.swing_criteria
              ? ` · ${formatSwingCriteria(props.task.swing_criteria)}`
              : ""}
            {props.task.overview_conditions
              ? ` · overview conditions ${formatTaskConditions(props.task.overview_conditions)}`
              : ""}
            {` · ${props.task.stage === "deep" ? "deep " : ""}conditions ${formatTaskConditions(props.task.conditions)}`}
            {props.task.ending_conditions.length > 0
              ? ` · ended by ${props.task.ending_conditions.map(formatEndingCondition).join(" + ")}`
              : ""}
            {props.task.reason ? ` · ${props.task.reason}` : ""}
          </p>
        ) : null}
      </div>
    </section>
  );
}

function formatTarget(target: AnalysisScopePreviewDto["targets"][number]): string {
  const path = target.node_path.indices.length === 0 ? "root" : target.node_path.indices.join(".");
  return `move ${target.move_number} (${path}, ${target.to_play} to play)`;
}

function formatTaskStage(stage: AnalysisTaskDto["stage"]): string {
  if (stage === "single_stage") return "single-stage";
  return stage;
}

function formatEndingCondition(condition: string): string {
  if (condition === "time_seconds") return "time";
  if (condition === "total_visits") return "total visits";
  if (condition === "leading_candidate_visits") return "leading candidate visits";
  return condition;
}

function formatTaskConditions(conditions: AnalysisTaskDto["conditions"]): string {
  const enabled: string[] = [];
  if (conditions.time_seconds.enabled) enabled.push(`${conditions.time_seconds.value}s`);
  if (conditions.total_visits.enabled) enabled.push(`${conditions.total_visits.value} total visits`);
  if (conditions.leading_candidate_visits.enabled) {
    enabled.push(`${conditions.leading_candidate_visits.value} leading candidate visits`);
  }
  return enabled.join(" OR ");
}

function formatSwingCriteria(criteria: NonNullable<AnalysisTaskDto["swing_criteria"]>): string {
  const thresholds: string[] = [];
  if (criteria.winrate_change_percentage_points.enabled) {
    thresholds.push(`${criteria.winrate_change_percentage_points.value}pp winrate`);
  }
  if (criteria.score_change_points.enabled) {
    thresholds.push(`${criteria.score_change_points.value} score`);
  }
  return `move actor ${criteria.move_actors} · swing ${thresholds.join(" OR ")}`;
}

function sameNodePath(left: { indices: number[] }, right: { indices: number[] }): boolean {
  return left.indices.length === right.indices.length
    && left.indices.every((index, offset) => index === right.indices[offset]);
}
