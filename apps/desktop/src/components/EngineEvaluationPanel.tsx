import { useEffect, useRef, useState } from "react";
import { cancelEngineEvaluation, getEngineEvaluation, isTauriRuntime, startEngineEvaluation } from "../api/backend";
import type { EngineProfileRecordDto, EngineRunDto, EvaluationSnapshotDto } from "../domain/types";
import { t } from "../i18n/resources";
import { EngineThreadPolicyPanel, threadPolicyMessage } from "./EngineThreadPolicyPanel";

type Props = { profiles: EngineProfileRecordDto[]; disabled: boolean; currentRun?: EngineRunDto | null; onProfilesSaved?: (profiles: EngineProfileRecordDto[]) => void; onPolicyBusyChange?: (busy: boolean) => void };

export function EngineEvaluationPanel({ profiles, disabled, currentRun = null, onProfilesSaved, onPolicyBusyChange }: Props) {
  const [target, setTarget] = useState("");
  const [publication, setPublication] = useState<{ snapshot: EvaluationSnapshotDto; scope: string } | null>(null);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState("");
  const owner = useRef<{ mounted: boolean; generation: number; id: string | null; revision: string | null; pending: boolean }>({ mounted: false, generation: 0, id: null, revision: null, pending: false });
  const retired = useRef(new Set<string>());
  const adoptingObservedTarget = useRef(false);
  const policyRecord = profiles.find((record) => record.id === target);
  const inputKey = JSON.stringify(target ? policyRecord?.profile : profiles.map((record) => [record.id, record.profile]));
  const scope = JSON.stringify([target, inputKey]);
  const snapshot = publication?.scope === scope ? publication.snapshot : null;
  function setSnapshot(next: EvaluationSnapshotDto | null) {
    setPublication(next ? { snapshot: next, scope } : null);
  }
  const native = isTauriRuntime();
  useEffect(() => {
    const state = owner.current;
    state.mounted = true;
    state.id = null;
    state.revision = null;
    state.pending = false;
    setPending(false); setSnapshot(null); setError("");
    let stopped = false;
    let timer: number;
    async function poll() {
      if (state.pending) {
        if (!stopped) timer = window.setTimeout(() => void poll(), 250);
        return;
      }
      const generation = state.generation;
      try {
        const next = await getEngineEvaluation();
        if (!stopped && !state.pending && generation === state.generation
          && (!target || next.target_id === target)
          && (!next.evaluation_id || !retired.current.has(next.evaluation_id))
          && (!state.id || (next.evaluation_id === state.id && next.input_revision === state.revision))) {
          state.id = next.evaluation_id;
          state.revision = next.input_revision;
          setSnapshot(next);
        }
      } catch (failure) {
        if (!stopped && !state.pending && generation === state.generation) setError(message(failure));
      }
      if (!stopped) timer = window.setTimeout(() => void poll(), 250);
    }
    if (native) void poll();
    return () => {
      stopped = true;
      state.mounted = false;
      state.generation++;
      clearTimeout(timer);
      if (state.id && !adoptingObservedTarget.current) {
        retired.current.add(state.id);
        void cancelEngineEvaluation(state.id).catch(() => undefined);
      }
      adoptingObservedTarget.current = false;
    };
  }, [native, target, inputKey]);

  async function start() {
    if (!native || owner.current.pending || !policyRecord || disabled) return;
    const state = owner.current;
    const generation = ++state.generation;
    if (state.id) retired.current.add(state.id);
    state.id = null; state.revision = null;
    state.pending = true;
    setPending(true); setError(""); setSnapshot(null);
    try {
      const next = await startEngineEvaluation(target);
      if (!state.mounted || generation !== state.generation || next.target_id !== target) {
        if (next.evaluation_id) await cancelEngineEvaluation(next.evaluation_id);
        return;
      }
      state.id = next.evaluation_id;
      state.revision = next.input_revision;
      setSnapshot(next);
    } catch (failure) {
      if (state.mounted && generation === state.generation) setError(message(failure));
    } finally {
      if (state.mounted && generation === state.generation) { state.pending = false; setPending(false); }
    }
  }

  async function cancel() {
    const state = owner.current;
    if (!state.id || state.pending) return;
    const generation = ++state.generation;
    state.pending = true;
    setPending(true); setError("");
    try {
      const next = await cancelEngineEvaluation(state.id);
      if (state.mounted && generation === state.generation && next.evaluation_id === state.id
        && (!target || next.target_id === target) && next.input_revision === state.revision) setSnapshot(next);
    } catch (failure) {
      if (state.mounted && generation === state.generation) setError(message(failure));
    } finally {
      if (state.mounted && generation === state.generation) { state.pending = false; setPending(false); }
    }
  }

  const result = snapshot?.phase === "completed" && snapshot.result?.evaluation_id === snapshot.evaluation_id
    && snapshot.result.target_id === snapshot.target_id && snapshot.result.input_revision === snapshot.input_revision ? snapshot.result : null;
  const phase = pending && !snapshot ? "qualifying" : error && !snapshot ? "failed" : snapshot?.phase ?? "idle";
  return <section aria-label={t("evaluation.title")} style={{ overflowWrap: "anywhere" }}>
    <h3>{t("evaluation.title")}</h3>
    <p className="message">{t("evaluation.hint")}</p>
    {!native && <p role="status">{t("evaluation.native")}</p>}
    <div className="engine-run-row">
      <label>{t("evaluation.target")}
        <select value={target} disabled={pending || disabled} onChange={(event) => {
          adoptingObservedTarget.current = !target && event.target.value === snapshot?.target_id;
          setTarget(event.target.value);
        }}>
          <option value="">{t("evaluation.selectTarget")}</option>
          {profiles.map((record) => <option key={record.id} value={record.id}>{record.profile.name} ({record.id})</option>)}
        </select>
      </label>
      <button type="button" disabled={!native || disabled || !policyRecord} aria-disabled={pending} onClick={() => void start()}>{t("evaluation.start")}</button>
      <button type="button" disabled={!snapshot?.evaluation_id} aria-disabled={pending} onClick={() => void cancel()}>{t("evaluation.cancel")}</button>
    </div>
    <p role="status" aria-live="polite" aria-atomic="true">
      {t(`evaluation.${phase}`)}
      {!snapshot?.target_id && target && ` · ${t("evaluation.target")}: ${policyRecord?.profile.name ?? target} (${target})`}
      {snapshot?.target_id && ` · ${t("evaluation.target")}: ${snapshot.target_id} · ${t("evaluation.identity")}: ${snapshot.evaluation_id} / ${snapshot.input_revision}`}
      {result && ` · ${t("engineResource.backend")}: ${result.qualified_resource.backend ?? t("evaluation.unavailable")}`}
      {result && ` · ${t("evaluation.measurements")}: ${result.measurements.length} · ${t("evaluation.nnSpeed")} / ${t("evaluation.speed")}`}
    </p>
    {error && <p role="alert">{error}</p>}
    {snapshot?.target_id && <p>{snapshot.target_id} · {snapshot.input_revision}</p>}
    {snapshot?.message && <p>{threadPolicyMessage(snapshot.message)}</p>}
    {snapshot && <p>{t("evaluation.exit")}: {snapshot.exit_code ?? "—"}</p>}
    {result && <>
      <dl>
        <dt>{t("evaluation.identity")}</dt><dd>{result.evaluation_id} · {result.target_id} · {result.input_revision}</dd>
        <dt>{t("evaluation.elapsed")}</dt><dd>{result.elapsed_ms}</dd>
        <dt>{t("engineResource.version")}</dt><dd>{result.qualified_resource.version ?? t("engineResource.unknown")}</dd>
        <dt>{t("engineResource.backend")}</dt><dd>{result.qualified_resource.backend ?? t("engineResource.unknown")}</dd>
        {result.qualified_resource.resources.map((resource, index) => <div key={index}>
          <dt>{resource.component}</dt><dd>{resource.sha256} · {resource.bytes} bytes</dd>
        </div>)}
      </dl>
      <div tabIndex={0} role="region" aria-label={t("evaluation.measurements")} style={{ overflowX: "auto" }}>
        <table>
          <caption>{t("evaluation.measurements")}</caption>
          <thead><tr>
            <th scope="col">{t("evaluation.threads")}</th>
            <th scope="col">{t("evaluation.nnSpeed")}</th>
            <th scope="col">{t("evaluation.speed")}</th>
          </tr></thead>
          <tbody>{result.measurements.map((row, index) => <tr key={index}>
            <th scope="row">{row.measured_threads}</th>
            <td>{row.nn_evals_per_second ?? t("evaluation.unavailable")}</td>
            <td>{row.search_visits_per_second ?? t("evaluation.unavailable")}</td>
          </tr>)}</tbody>
        </table>
      </div>
    </>}
    {policyRecord && <EngineThreadPolicyPanel key={policyRecord.id} record={policyRecord} currentRun={currentRun}
      evaluationId={result?.target_id === policyRecord.id ? result.evaluation_id : null}
      disabled={disabled || pending || snapshot?.phase === "running" || snapshot?.phase === "qualifying"}
      onSaved={onProfilesSaved} onBusyChange={onPolicyBusyChange} />}
    {Boolean(snapshot?.output.length) && <details open>
      <summary>{t("evaluation.output")}</summary>
      <pre tabIndex={0} style={{ maxHeight: "12rem", overflow: "auto", whiteSpace: "pre-wrap", overflowWrap: "anywhere" }}>{snapshot?.output.join("\n")}</pre>
    </details>}
    {snapshot?.output_truncated && <p>{t("evaluation.truncated")}</p>}
  </section>;
}

function message(error: unknown): string {
  return threadPolicyMessage(error);
}
