import { useEffect, useRef, useState } from "react";
import { getRuntimeThreads, isTauriRuntime, requestRuntimeThreads } from "../api/backend";
import type { RuntimeThreadsRequestDto, RuntimeThreadsSnapshotDto } from "../domain/types";
import { t } from "../i18n/resources";

type Props = { runId: string | null; disabled: boolean };

export function RuntimeThreadsPanel({ runId, disabled }: Props) {
  const [snapshot, setSnapshot] = useState<RuntimeThreadsSnapshotDto | null>(null);
  const [draft, setDraft] = useState("");
  const [pending, setPending] = useState(false);
  const [error, setError] = useState("");
  const owner = useRef({ generation: 0, request: 0, pending: false });
  const native = isTauriRuntime();
  useEffect(() => {
    const state = owner.current;
    const generation = ++state.generation;
    state.pending = false;
    setPending(false); setSnapshot(null); setError("");
    let stopped = false;
    let timer: number;
    async function poll() {
      if (!state.pending) {
        const request = state.request;
        try {
          const next = await getRuntimeThreads();
          if (!stopped && !state.pending && request === state.request && generation === state.generation && next.run_id === runId) setSnapshot(next);
        } catch (failure) {
          if (!stopped && request === state.request && generation === state.generation) setError(String(failure));
        }
      }
      if (!stopped) timer = window.setTimeout(() => void poll(), 300);
    }
    if (native) void poll();
    return () => { stopped = true; state.generation++; clearTimeout(timer); };
  }, [runId, native]);

  const current = snapshot?.run_id === runId ? snapshot : null;
  const canAct = native && !disabled && !pending && current?.supported && Boolean(runId && current.profile_revision);
  const value = /^\d+$/.test(draft) ? Number(draft) : NaN;
  const valid = Number.isInteger(value) && value >= (current?.minimum ?? 1) && value <= (current?.maximum ?? 4096);
  async function act(action: RuntimeThreadsRequestDto["action"]) {
    if (!canAct || owner.current.pending || !current?.run_id || !current.profile_revision || (action === "apply" && !valid)) return;
    const state = owner.current;
    const generation = state.generation;
    state.request++;
    const requestId = crypto.randomUUID();
    const requested = action === "apply" ? value : action === "reset" ? current.sources?.effective ?? null : null;
    state.pending = true; setPending(true); setError("");
    setSnapshot({ ...current, requested, request_id: requestId, status: "pending", failure: null });
    try {
      const next = await requestRuntimeThreads({ identity: { run_id: current.run_id, profile_revision: current.profile_revision, request_id: requestId }, action, value: action === "apply" ? value : null });
      if (generation === state.generation && next.run_id === runId && next.request_id === requestId) setSnapshot(next);
    } catch (failure) {
      if (generation === state.generation) {
        setError(String(failure));
        setSnapshot((previous) => previous ? { ...previous, status: "failed" } : previous);
      }
    } finally {
      if (generation === state.generation) { state.pending = false; setPending(false); }
    }
  }
  return <section aria-label={t("threads.title")} style={{ overflowWrap: "anywhere" }}>
    <h3>{t("threads.title")}</h3>
    <p>{t("threads.hint")}</p>
    {!native && <p role="status">{t("threads.native")}</p>}
    {current?.reason && <p>{current.reason}</p>}
    <p role="status">{t(`threads.${current?.status ?? "unknown"}`)}{current?.temporary ? ` · ${t("threads.temporary")}` : ""}</p>
    <dl>
      <dt>{t("threads.saved")}</dt><dd>{current?.sources?.saved ?? "—"}</dd>
      <dt>{t("threads.override")}</dt><dd>{current?.sources?.launch_override ?? "—"}</dd>
      <dt>{t("threads.effective")}</dt><dd>{current?.sources?.effective ?? "—"}</dd>
      <dt>{t("threads.actual")}</dt><dd>{current?.actual ?? "—"}</dd>
      <dt>{t("threads.requested")}</dt><dd>{current?.requested ?? "—"}</dd>
    </dl>
    <form onSubmit={(event) => { event.preventDefault(); void act("apply"); }}>
      <label>{t("threads.draft")}<input type="text" inputMode="numeric" value={draft} onChange={(event) => setDraft(event.target.value)} /></label>
      <button type="submit" disabled={!canAct || !valid}>{t("threads.apply")}</button>
      <button type="button" disabled={!canAct || current?.sources?.effective == null} onClick={() => void act("reset")}>{t("threads.reset")}</button>
      <button type="button" disabled={!canAct} onClick={() => void act("read")}>{t("threads.read")}</button>
    </form>
    {(error || current?.failure || current?.sources?.error) && <p role="alert">{error || current?.failure || current?.sources?.error}</p>}
    {current && <p>{current.run_id} · {current.profile_revision}</p>}
    {Boolean(current?.sources?.entries.length) && <details><summary>{t("threads.sources")}</summary>
      <ol>{current?.sources?.entries.map((entry, index) => <li key={index}>{entry.layer} · {entry.source} · {entry.key} = {entry.value ?? "—"}</li>)}</ol>
    </details>}
  </section>;
}
