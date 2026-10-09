import { useEffect, useRef, useState } from "react";
import { isTauriRuntime } from "../api/backend";
import { cancelDiagnosticExport, diagnosticExportStatus, estimateDiagnosticExport, openDiagnosticExportFolder, startDiagnosticExport } from "../api/diagnosticExport";
import type { DiagnosticExportStatusDto, EngineDiagnosticSnapshotDto } from "../domain/types";
import { t } from "../i18n/resources";

export function DiagnosticExportPanel({ displayed, freeze, disabled }: {
  displayed: EngineDiagnosticSnapshotDto; freeze: (snapshot: EngineDiagnosticSnapshotDto) => void; disabled: boolean;
}) {
  const [status, setStatus] = useState<DiagnosticExportStatusDto | null>(null);
  const [generation, setGeneration] = useState<number | null>(null);
  const [notice, setNotice] = useState("");
  const [busy, setBusy] = useState(false);
  const request = useRef(0);
  const native = isTauriRuntime();
  useEffect(() => {
    if (!native) return;
    let disposed = false;
    let timer: number | undefined;
    const pollingRequest = request.current;
    async function refresh() {
      try {
        const result = await diagnosticExportStatus();
        if (!disposed && pollingRequest === request.current && (generation === null || result.generation === generation)) setStatus(result);
      } catch { if (!disposed && pollingRequest === request.current) setNotice(t("diagnostics.export.failed")); }
      finally { if (!disposed && pollingRequest === request.current) timer = window.setTimeout(() => void refresh(), 150); }
    }
    void refresh();
    // Closing the panel does not retarget or cancel its frozen native operation.
    return () => { disposed = true; clearTimeout(timer); };
  }, [generation, native]);
  const active = status && ["collecting", "archiving", "syncing", "publishing", "cancelling"].includes(status.phase);
  async function estimate() {
    const sequence = ++request.current;
    const frozen = displayed;
    freeze(frozen);
    setBusy(true); setNotice(""); setStatus(null);
    try {
      const next = await estimateDiagnosticExport(frozen);
      if (sequence === request.current) setGeneration(next);
    } catch { if (sequence === request.current) setNotice(t("diagnostics.export.failed")); }
    finally { if (sequence === request.current) setBusy(false); }
  }
  async function act(action: "start" | "cancel" | "folder") {
    if (!status) return;
    const target = status.generation;
    setBusy(true); setNotice("");
    try {
      if (action === "start") await startDiagnosticExport(target);
      else if (action === "cancel") await cancelDiagnosticExport(target);
      else {
        const result = await openDiagnosticExportFolder(target);
        setNotice(t(result === "opened" ? "diagnostics.export.folderOpened" : result === "timed_out" ? "diagnostics.export.folderTimeout" : "diagnostics.export.folderFailed"));
      }
      const next = await diagnosticExportStatus();
      if (next.generation === target) setStatus(next);
    } catch { setNotice(t("diagnostics.export.failed")); }
    finally { setBusy(false); }
  }
  return <section aria-label={t("diagnostics.export.title")} style={{ overflowWrap: "anywhere" }}>
    <h4>{t("diagnostics.export.title")}</h4>
    {!native && <p>{t("diagnostics.export.native")}</p>}
    <div className="engine-run-row">
      <button type="button" disabled={!native || disabled || busy || Boolean(active)} onClick={() => void estimate()}>{t("diagnostics.export.estimate")}</button>
      <button type="button" disabled={disabled || busy || status?.phase !== "ready"} onClick={() => void act("start")}>{t("diagnostics.export.start")}</button>
      <button type="button" disabled={busy || !status || ["idle", "cancelled", "closed", "completed", "failed"].includes(status.phase)} onClick={() => void act("cancel")}>{t("diagnostics.export.cancel")}</button>
      <button type="button" disabled={disabled || busy || status?.phase !== "completed"} onClick={() => void act("folder")}>{t("diagnostics.export.folder")}</button>
    </div>
    {status && <div role="status">
      <p>{t(`diagnostics.export.phase.${status.phase}`)}</p>
      <p>{t("diagnostics.attempt")}: {status.attempt_id ?? "—"} @ {status.captured_at_ms ?? "—"} / #{status.generation}</p>
      <p>{t("diagnostics.export.size")}: {status.source_bytes === null ? t("diagnostics.missing") : `${(status.source_bytes / 1048576).toFixed(3)} MiB`}; {status.completed_entries}/{status.entries ?? "?"}</p>
      {status.failed_stage && <p>{t(`diagnostics.export.phase.${status.failed_stage}`)}: {status.message}</p>}
      {status.file_name && <p>{status.file_name}</p>}
      {status.cleanup_pending && <p>{t("diagnostics.export.cleanup")}</p>}
    </div>}
    {notice && <p role="status">{notice}</p>}
  </section>;
}
