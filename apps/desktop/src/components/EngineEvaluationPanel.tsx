import { useEffect, useRef, useState } from "react";
import { cancelEngineEvaluation, getEngineEvaluation, isTauriRuntime, startEngineEvaluation } from "../api/backend";
import type { EngineProfileRecordDto, EvaluationSnapshotDto } from "../domain/types";
import { t } from "../i18n/resources";
import { EngineThreadPolicyPanel, threadPolicyMessage } from "./EngineThreadPolicyPanel";

type Props = { profiles: EngineProfileRecordDto[]; disabled: boolean; onProfilesSaved?: (profiles: EngineProfileRecordDto[]) => void; onPolicyBusyChange?: (busy: boolean) => void };

export function EngineEvaluationPanel({ profiles, disabled, onProfilesSaved, onPolicyBusyChange }: Props) {
  const [target, setTarget] = useState("");
  const [snapshot, setSnapshot] = useState<EvaluationSnapshotDto | null>(null);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState("");
  const owner = useRef<{ mounted: boolean; generation: number; id: string | null; pending: boolean }>({ mounted: false, generation: 0, id: null, pending: false });
  const native = isTauriRuntime();
  useEffect(() => {
    const state = owner.current;
    state.mounted = true;
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
        if (!stopped && !state.pending && generation === state.generation) {
          state.id = next.evaluation_id;
          setSnapshot(next);
        }
      } catch (failure) {
        if (!stopped) setError(message(failure));
      }
      if (!stopped) timer = window.setTimeout(() => void poll(), 250);
    }
    if (native) void poll();
    return () => {
      stopped = true;
      state.mounted = false;
      state.generation++;
      clearTimeout(timer);
      if (state.id) void cancelEngineEvaluation(state.id).catch(() => undefined);
    };
  }, [native]);

  async function start() {
    if (owner.current.pending || !target || disabled) return;
    const state = owner.current;
    const generation = ++state.generation;
    state.pending = true;
    setPending(true); setError(""); setSnapshot(null);
    try {
      const next = await startEngineEvaluation(target);
      if (!state.mounted || generation !== state.generation) {
        if (next.evaluation_id) await cancelEngineEvaluation(next.evaluation_id);
        return;
      }
      state.id = next.evaluation_id;
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
      if (state.mounted && generation === state.generation) setSnapshot(next);
    } catch (failure) {
      if (state.mounted && generation === state.generation) setError(message(failure));
    } finally {
      if (state.mounted && generation === state.generation) { state.pending = false; setPending(false); }
    }
  }

  const result = snapshot?.phase === "completed" ? snapshot.result : null;
  const policyRecord = profiles.find((record) => record.id === target);
  return <section aria-label={t("evaluation.title")} style={{ overflowWrap: "anywhere" }}>
    <h3>{t("evaluation.title")}</h3>
    <p className="message">{t("evaluation.hint")}</p>
    {!native && <p role="status">{t("evaluation.native")}</p>}
    <div className="engine-run-row">
      <label>{t("evaluation.target")}
        <select value={target} disabled={pending || disabled} onChange={(event) => setTarget(event.target.value)}>
          <option value="">—</option>
          {profiles.map((record) => <option key={record.id} value={record.id}>{record.profile.name} ({record.id})</option>)}
        </select>
      </label>
      <button type="button" disabled={!native || disabled || pending || !profiles.some((record) => record.id === target)} onClick={() => void start()}>{t("evaluation.start")}</button>
      <button type="button" disabled={!snapshot?.evaluation_id || pending} onClick={() => void cancel()}>{t("evaluation.cancel")}</button>
    </div>
    <p role="status">{t(`evaluation.${snapshot?.phase ?? "idle"}`)}</p>
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
    {policyRecord && <EngineThreadPolicyPanel key={policyRecord.id} record={policyRecord}
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
