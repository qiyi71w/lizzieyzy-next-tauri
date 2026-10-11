import { useEffect, useRef, useState } from "react";
import { continuationBeat, continuationNativeAvailable, startContinuation, stopContinuation } from "../api/continuation";
import { loadContinuationPreferences, saveContinuationPreferences } from "../api/preferences";
import { defaultContinuationPreferences } from "../domain/continuation";
import type { AnalysisBranchChoiceDto, ContinuationModeDto, ContinuationSnapshotDto, CurrentGameResultDto } from "../domain/types";
import { t } from "../i18n/resources";
import "./BatchPanel.css";

type Props = { current: CurrentGameResultDto | null; choices: AnalysisBranchChoiceDto[]; onCurrent: (value: CurrentGameResultDto) => void; onStart: () => void };
export function ContinuationPanel({ current, choices, onCurrent, onStart }: Props) {
  const [open, setOpen] = useState(false);
  const [saved, setSaved] = useState(defaultContinuationPreferences);
  const [mode, setMode] = useState<ContinuationModeDto>("leaf_only");
  const [seconds, setSeconds] = useState("1");
  const [snapshot, setSnapshot] = useState<ContinuationSnapshotDto | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const serial = useRef(0);
  const source = useRef<HTMLButtonElement>(null);
  const panel = useRef<HTMLElement>(null);
  const onCurrentRef = useRef(onCurrent); onCurrentRef.current = onCurrent;
  const native = continuationNativeAvailable();
  const active = snapshot?.phase === "waiting" || snapshot?.phase === "running";
  useEffect(() => {
    if (!open) return;
    let disposed = false;
    panel.current?.querySelector<HTMLButtonElement>("button")?.focus();
    void loadContinuationPreferences().then(value => {
      if (!disposed) { setSaved(value); setMode(value.mode); setSeconds(String(value.intervalSeconds)); }
    }).catch(value => { if (!disposed) setError(String(value)); });
    return () => { disposed = true; };
  }, [open]);
  useEffect(() => {
    if (!open || !active || !snapshot?.session_id) return;
    const session = snapshot.session_id;
    const sequence = serial.current;
    let disposed = false;
    let timer: number;
    const tick = async () => {
      try {
        const next = await continuationBeat(session);
        if (!disposed && serial.current === sequence) {
          setSnapshot(next);
          if (next.current_game) onCurrentRef.current(next.current_game);
        }
      } catch (value) { if (!disposed && serial.current === sequence) setError(String(value)); }
      if (!disposed && serial.current === sequence) timer = window.setTimeout(tick, 100);
    };
    timer = window.setTimeout(tick, 100);
    return () => { disposed = true; clearTimeout(timer); };
  }, [open, active, snapshot?.session_id]);
  async function stop(close: boolean) {
    const sequence = ++serial.current;
    setBusy(true);
    try {
      if (native) {
        const next = await stopContinuation();
        if (sequence !== serial.current) return;
        setSnapshot(next);
        if (next.current_game) onCurrentRef.current(next.current_game);
      }
      if (close) { setMode(saved.mode); setSeconds(String(saved.intervalSeconds)); setOpen(false); source.current?.focus(); }
    } catch (value) { setError(String(value)); }
    finally { if (sequence === serial.current) setBusy(false); }
  }
  async function start() {
    if (!current || busy) return;
    const sequence = ++serial.current;
    setBusy(true); setError(null); onStart();
    try {
      const next = await startContinuation(current.generation, current.selected_path, choices);
      if (sequence === serial.current) setSnapshot(next);
    } catch (value) { if (sequence === serial.current) setError(String(value)); }
    finally { if (sequence === serial.current) setBusy(false); }
  }
  async function save() {
    setBusy(true); setError(null);
    try {
      const value = await saveContinuationPreferences({ mode, intervalSeconds: Number(seconds) });
      setSaved(value); setMode(value.mode); setSeconds(String(value.intervalSeconds));
    } catch (value) { setError(String(value)); }
    finally { setBusy(false); }
  }
  return <>
    <div className="batch-global"><button ref={source} type="button" onClick={() => setOpen(true)}>{t("continuation.title")}</button></div>
    {open ? <section ref={panel} className="batch-panel" role="dialog" aria-modal="true" aria-label={t("continuation.title")} onKeyDown={event => {
      if (event.nativeEvent.isComposing) return;
      if (event.key === "Escape") { event.stopPropagation(); event.preventDefault(); void stop(true); }
      if (event.key === "Tab") {
        const controls = Array.from(panel.current?.querySelectorAll<HTMLElement>("button:not(:disabled), input:not(:disabled), select:not(:disabled)") ?? []);
        const first = controls[0], last = controls.at(-1);
        if (event.shiftKey && document.activeElement === first) { event.preventDefault(); last?.focus(); }
        else if (!event.shiftKey && document.activeElement === last) { event.preventDefault(); first?.focus(); }
      }
    }}>
      <header><h2>{t("continuation.title")}</h2><button type="button" onClick={() => void stop(true)}>{t("continuation.close")}</button></header>
      {!native ? <p>{t("continuation.native")}</p> : null}
      <fieldset disabled={busy}>
        <label>{t("continuation.mode")}<select value={mode} onChange={event => setMode(event.target.value as ContinuationModeDto)}>
          <option value="leaf_only">{t("continuation.leaf")}</option><option value="every_beat">{t("continuation.every")}</option>
        </select></label>
        <label>{t("continuation.interval")}<input type="number" min="0.001" step="0.001" value={seconds} onChange={event => setSeconds(event.target.value)} /></label>
        <button type="button" onClick={() => void save()}>{t("continuation.save")}</button>
        <button type="button" onClick={() => { setMode(saved.mode); setSeconds(String(saved.intervalSeconds)); setError(null); }}>{t("continuation.cancel")}</button>
      </fieldset>
      <p>{t("continuation.explicit")}</p>
      <button type="button" disabled={!native || !current || busy || active} onClick={() => void start()}>{t("continuation.start")}</button>
      <button type="button" disabled={!native} onClick={() => void stop(false)}>{t("continuation.stop")}</button>
      <p role="status">{t(`continuation.${snapshot?.phase ?? "stopped"}`)}{snapshot?.reason ? ` · ${snapshot.reason}` : ""}</p>
      {error ? <p role="alert">{error}</p> : null}
    </section> : null}
  </>;
}
