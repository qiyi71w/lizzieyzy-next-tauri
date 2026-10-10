import { useEffect, useMemo, useState } from "react";
import { getEngineDiagnostics, setEngineDiagnosticTrace, stopForegroundEngine } from "../api/backend";
import type { EngineDiagnosticSnapshotDto, ForegroundEngineSnapshotDto } from "../domain/types";
import { runFromSnapshot } from "../domain/foregroundEngine";
import { t } from "../i18n/resources";
import { DiagnosticExportPanel } from "./DiagnosticExportPanel";

export function EngineDiagnosticsPanel({ engineSnapshot, disabled = false }: {
  engineSnapshot: ForegroundEngineSnapshotDto; disabled?: boolean;
}) {
  const [attempts, setAttempts] = useState<EngineDiagnosticSnapshotDto[]>([]);
  const [selected, setSelected] = useState("");
  const [pinned, setPinned] = useState<EngineDiagnosticSnapshotDto | null>(null);
  const [filter, setFilter] = useState("");
  const [notice, setNotice] = useState("");
  const [busy, setBusy] = useState(false);
  useEffect(() => {
    let timer: number | undefined;
    let disposed = false;
    async function refresh() {
      try {
        const next = await getEngineDiagnostics();
        if (!disposed) setAttempts(next);
      } catch {
        if (!disposed) setNotice(t("diagnostics.failed"));
      } finally {
        // One request in flight, no output-event queue in React.
        if (!disposed) timer = window.setTimeout(() => void refresh(), 500);
      }
    }
    if (!pinned) void refresh();
    return () => { disposed = true; clearTimeout(timer); };
  }, [pinned]);
  const displayed = pinned ?? attempts.find((attempt) => attempt.attempt_id === selected) ?? attempts.at(-1);
  const text = useMemo(() => displayed ? JSON.stringify(displayed, null, 2) : "", [displayed]);
  const output = useMemo(() => displayed?.records.filter((record) => !filter || record.text.toLocaleLowerCase().includes(filter.toLocaleLowerCase()))
    .map((record) => `${record.sequence} ${record.at_ms} [${record.source}] ${record.text}`).join("\n") ?? "", [displayed, filter]);
  const currentRun = engineSnapshot.lifecycle.state === "starting" ? engineSnapshot.lifecycle.run
    : engineSnapshot.lifecycle.state === "switching" ? engineSnapshot.lifecycle.candidate : runFromSnapshot(engineSnapshot);
  async function copy() {
    if (!displayed) return;
    const frozen = displayed;
    setPinned(frozen);
    try { await navigator.clipboard.writeText(JSON.stringify(frozen, null, 2)); setNotice(t("diagnostics.copied")); }
    catch { setNotice(t("diagnostics.copyFailed")); }
  }
  async function trace(enabled: boolean) {
    if (!displayed) return;
    setBusy(true);
    try {
      await setEngineDiagnosticTrace(displayed.attempt_id, enabled);
      setAttempts((current) => current.map((attempt) => attempt.attempt_id === displayed.attempt_id ? { ...attempt, full_trace: enabled } : attempt));
    } catch { setNotice(t("diagnostics.traceFailed")); }
    finally { setBusy(false); }
  }
  async function stop() {
    setBusy(true);
    try { await stopForegroundEngine(); }
    catch { setNotice(t("diagnostics.stopFailed")); }
    finally { setBusy(false); }
  }
  return <details className="engine-diagnostics">
    <summary>{t("diagnostics.title")}</summary>
    <p>{t("diagnostics.status")}: {engineSnapshot.lifecycle.state}</p>
    {!displayed ? <p>{t("diagnostics.empty")}</p> : <>
      <div className="engine-run-row">
        <label>{t("diagnostics.attempt")}<select value={displayed.attempt_id} disabled={Boolean(pinned)} onChange={(event) => setSelected(event.target.value)}>
          {attempts.map((attempt) => <option key={attempt.attempt_id} value={attempt.attempt_id}>{attempt.profile_id} / {attempt.attempt_id}</option>)}
        </select></label>
        <button type="button" onClick={() => void copy()}>{t("diagnostics.copy")}</button>
        <button type="button" onClick={() => setPinned(pinned ? null : displayed)}>{t(pinned ? "diagnostics.resume" : "diagnostics.freeze")}</button>
        <button type="button" disabled={disabled || busy || !currentRun || currentRun.run_id !== displayed.run_id} onClick={() => void stop()}>{t("diagnostics.stop")}</button>
      </div>
      <DiagnosticExportPanel displayed={displayed} freeze={setPinned} disabled={disabled} />
      <label><input type="checkbox" checked={displayed.full_trace} disabled={disabled || busy || Boolean(pinned)} onChange={(event) => void trace(event.target.checked)} />{t("diagnostics.trace")}</label>
      <label>{t("diagnostics.filter")}<input value={filter} maxLength={128} onChange={(event) => setFilter(event.target.value)} /></label>
      <pre style={{ whiteSpace: "pre-wrap", overflowWrap: "anywhere" }}>{t("diagnostics.command")}: {displayed.command}</pre>
      {displayed.failure && <p>{t("diagnostics.failure")}: {displayed.failure}</p>}
      <p>{t("diagnostics.exit")}: {displayed.process_exited ? displayed.exit_code ?? t("diagnostics.unknownCode") : t("diagnostics.notExited")}; {t("diagnostics.streams")}: stdout={String(displayed.stdout_complete)}, stderr={String(displayed.stderr_complete)}</p>
      <p>{displayed.retained_bytes} bytes; {t("diagnostics.dropped")}: {displayed.dropped_records}</p>
      <pre tabIndex={0} aria-label={t("diagnostics.output")} style={{ maxHeight: "18rem", overflow: "auto", whiteSpace: "pre-wrap", overflowWrap: "anywhere" }}>{output}</pre>
      <ul>{displayed.metrics.map((metric) => <li key={`${metric.role}:${metric.name}`}>{metric.role} / {metric.name}: {metric.value ?? `${t("diagnostics.missing")} (${metric.missing})`} {metric.unit} @ {metric.at_ms}</li>)}</ul>
      {pinned && <textarea readOnly aria-label={t("diagnostics.copy")} value={text} rows={8} style={{ width: "100%", boxSizing: "border-box" }} />}
    </>}
    {notice && <p role="status">{notice}</p>}
  </details>;
}
