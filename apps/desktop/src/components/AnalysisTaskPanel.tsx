import type {
  AnalysisScopeModeDto,
  AnalysisScopePreviewDto,
  AnalysisTaskDto,
  PlayerColor
} from "../domain/types";

export type AnalysisScopeDraft = {
  mode: AnalysisScopeModeDto;
  intervalEnabled: boolean;
  intervalStart: string;
  intervalEnd: string;
  toPlay: PlayerColor | "both";
  totalVisits: string;
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
  const first = props.preview?.targets[0];
  const last = props.preview?.targets.at(-1);

  function update(patch: Partial<AnalysisScopeDraft>) {
    props.onDraftChange({ ...props.draft, ...patch });
  }

  return (
    <section className="sheet-row analysis-task-panel" aria-label="Analysis task">
      <div className="sgf-tools">
        <div className="document-row">
          <strong>Analysis task</strong>
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
          <label>
            Total visits
            <input
              aria-label="Total visits"
              type="number"
              min={1}
              max={1000000}
              step={1}
              value={props.draft.totalVisits}
              onChange={(event) => update({ totalVisits: event.target.value })}
            />
          </label>
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
            {props.preview.targets.length} targets
            {first ? ` · first ${formatTarget(first)}` : ""}
            {last && last !== first ? ` · last ${formatTarget(last)}` : ""}
          </p>
        ) : null}
        {props.error ? <p role="alert">{props.error}</p> : null}
        {props.task ? (
          <p role="status" data-analysis-task-state={props.task.state}>
            {formatTaskStage(props.task.stage)} · {props.task.state} · {completed}/{requested} completed
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

function formatTaskStage(stage: string): string {
  return stage === "single_stage" ? "single-stage" : stage;
}
