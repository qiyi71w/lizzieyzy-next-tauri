import { useEffect, useRef, useState } from "react";
import { getRuntimeParameters, isTauriRuntime, readRuntimeParameters } from "../api/backend";
import type { RuntimeParametersSnapshotDto } from "../domain/types";
import { t } from "../i18n/resources";

type Props = { runId: string | null; disabled: boolean };

export function RuntimeParametersPanel({ runId, disabled }: Props) {
  const [snapshot, setSnapshot] = useState<RuntimeParametersSnapshotDto | null>(null);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState("");
  const owner = useRef({ generation: 0, request: 0, pending: false, failed: false });
  const native = isTauriRuntime();
  useEffect(() => {
    const state = owner.current;
    const generation = ++state.generation;
    state.pending = false; state.failed = false;
    setPending(false); setSnapshot(null); setError("");
    let stopped = false;
    let timer: number;
    async function poll() {
      if (!state.pending && !state.failed) {
        const request = state.request;
        try {
          const next = await getRuntimeParameters();
          if (!stopped && !state.pending && !state.failed && request === state.request && generation === state.generation && next.run_id === runId) setSnapshot(next);
        } catch (failure) {
          if (!stopped && request === state.request && generation === state.generation) {
            state.failed = true;
            setError(String(failure));
            setSnapshot(previous => previous ? { ...previous, status: "failed" } : previous);
          }
        }
      }
      if (!stopped) timer = window.setTimeout(() => void poll(), 300);
    }
    if (native) void poll();
    return () => { stopped = true; state.generation++; clearTimeout(timer); };
  }, [runId, native]);

  const current = snapshot?.run_id === runId ? snapshot : null;
  const canRead = native && !disabled && !pending && current?.supported && Boolean(runId && current.profile_revision);
  async function readPair() {
    if (!canRead || owner.current.pending || !current?.run_id || !current.profile_revision) return;
    const state = owner.current;
    const generation = state.generation;
    const requestId = crypto.randomUUID();
    state.request++; state.pending = true; state.failed = false;
    setPending(true); setError("");
    setSnapshot({ ...current, request_id: requestId, status: "pending", failure: null });
    try {
      const next = await readRuntimeParameters({ run_id: current.run_id, profile_revision: current.profile_revision, request_id: requestId });
      if (generation === state.generation) {
        if (next.run_id !== runId || next.request_id !== requestId || next.profile_revision !== current.profile_revision) throw new Error("Parameter response identity expired.");
        setSnapshot(next);
      }
    } catch (failure) {
      if (generation === state.generation) {
        state.failed = true;
        setError(String(failure));
        setSnapshot(previous => previous ? { ...previous, status: "failed" } : previous);
      }
    } finally {
      if (generation === state.generation) { state.pending = false; setPending(false); }
    }
  }
  return <section aria-label={t("parameters.title")} style={{ overflowWrap: "anywhere" }}>
    <h3>{t("parameters.title")}</h3>
    <p>{t("parameters.hint")}</p>
    {!native && <p role="status">{t("parameters.native")}</p>}
    {current?.reason && <p>{current.reason}</p>}
    <p role="status">{t(`parameters.${error ? "failed" : current?.status ?? "unknown"}`)}</p>
    {current?.last_valid && <>
      {current.status !== "confirmed" && <p>{t("parameters.stale")}</p>}
      <dl>
        <dt>{t("parameters.pda")}</dt><dd>{current.last_valid.playout_doubling_advantage}</dd>
        <dt>{t("parameters.wrn")}</dt><dd>{current.last_valid.analysis_wide_root_noise}</dd>
      </dl>
    </>}
    <form onSubmit={event => { event.preventDefault(); void readPair(); }}>
      <button type="submit" disabled={!canRead}>{t("parameters.read")}</button>
    </form>
    {(error || current?.failure) && <p role="alert">{error || current?.failure}</p>}
    {current && <p>{current.run_id} · {current.profile_revision}</p>}
  </section>;
}
