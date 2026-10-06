import { useEffect, useRef, useState } from "react";
import { NetworkSettingsPanel } from "./NetworkSettingsPanel";
import { YikePublicCenter } from "./YikePublicCenter";
import { TencentKifuCenter } from "./TencentKifuCenter";
import { ReadboardPanel } from "./ReadboardPanel";
import { FoxKifuCenter } from "./FoxKifuCenter";
import {
  networkSnapshot,
  importProviderPayload,
  syncReadboardSidecarSnapshot
} from "../api/providers";
import {
  emptyProviderMetadata,
  providerLabel,
  type NetworkSnapshot,
  type ProviderRequestIdentity,
  type ProviderImportRequest,
  type ProviderImportResult,
  type ProviderKind,
  type ReadboardSidecarSyncSnapshotResult
} from "../domain/providers";

type Props = {
  initialProvider?: ProviderKind;
  disabled?: boolean;
  onImport: (result: ProviderImportResult, identity?: ProviderRequestIdentity) => void | Promise<void>;
  onStartSync?: (locator: string, play: boolean) => Promise<void>;
};

type OperationStatus = {
  fetch: string;
  import: string;
  readboardSync: string;
};

const readboardTimeoutMs = 5_000;

const initialStatuses: OperationStatus = {
  fetch: "Provider fetch ready.",
  import: "Payload import ready.",
  readboardSync: "Protocol snapshot preview ready."
};

export function ProviderPanel({ disabled = false, initialProvider = "yike", onImport, onStartSync }: Props) {
  const [provider, setProvider] = useState<ProviderKind>(initialProvider);
  const [payload, setPayload] = useState("");
  const [statuses, setStatuses] = useState<OperationStatus>(initialStatuses);
  const [readboardProtocolLine, setReadboardProtocolLine] = useState("");
  const [readboardSyncResult, setReadboardSyncResult] = useState<ReadboardSidecarSyncSnapshotResult | null>(null);
  const [providerWarnings, setProviderWarnings] = useState<string[]>([]);
  const [network, setNetwork] = useState<NetworkSnapshot | null>(null);
  const [networkError, setNetworkError] = useState("");
  const [busy, setBusy] = useState(false);
  const sequence = useRef(0);

  useEffect(() => {
    let mounted = true;
    void networkSnapshot().then((value) => { if (mounted) setNetwork(value); })
      .catch((error: unknown) => { if (mounted) setNetworkError(errorMessage(error)); });
    return () => {
      mounted = false;
      sequence.current += 1;
    };
  }, []);

  function invalidatePreview() {
    sequence.current += 1;
    setBusy(false);
  }

  function policySaved(snapshot: NetworkSnapshot) {
    invalidatePreview();
    setNetwork(snapshot);
    setNetworkError("");
    setOperationStatus("fetch", "网络策略已保存，旧预览已失效。请重新预览。");
  }
  const canImport = !disabled && !busy && payload.trim().length > 0;
  const canSyncReadboard = !disabled && readboardProtocolLine.trim().length > 0;
  const headerStatus = statuses.fetch !== initialStatuses.fetch
    ? statuses.fetch
    : statuses.import;

  function handleProviderChange(nextProvider: ProviderKind) {
    invalidatePreview();
    setProvider(nextProvider);
    setProviderWarnings([]);
    setOperationStatus("fetch", `${providerLabel(nextProvider)} fetch ready.`);
    setOperationStatus("import", `${providerLabel(nextProvider)} payload import ready.`);
  }



  async function handleImport() {
    if (!canImport) return;
    setOperationStatus("import", "Importing provider payload...");
    const requestSequence = sequence.current;
    setBusy(true);
    try {
      const result = await importProviderPayload(buildRequest(provider, payload));
      if (sequence.current !== requestSequence) return;
      await onImport(result);
      setProviderWarnings(result.warnings);
      setOperationStatus("import", "导入请求已交给棋谱确认流程。");
    } catch (error) {
      setProviderWarnings([]);
      setOperationStatus("import", `Import failed: ${errorMessage(error)}`);
    } finally { if (sequence.current === requestSequence) setBusy(false); }
  }


  async function handleReadboardSync() {
    if (!canSyncReadboard) return;
    setOperationStatus("readboardSync", "Previewing protocol snapshot...");
    try {
      const result = await syncReadboardSidecarSnapshot({
        sgf_text: readboardProtocolLine.trim(),
        metadata: { source: "provider_panel", input: "protocol_line" },
        timeout_ms: readboardTimeoutMs
      });
      setReadboardSyncResult(result);
      setOperationStatus("readboardSync", readboardSyncStatus(result));
    } catch (error) {
      setReadboardSyncResult(null);
      setOperationStatus("readboardSync", `Readboard preview failed: ${errorMessage(error)}`);
    }
  }


  function setOperationStatus(operation: keyof OperationStatus, status: string) {
    setStatuses((current) => ({ ...current, [operation]: status }));
  }

  return (
    <section className="provider-panel" aria-label="同步">
      <div className="provider-header">
        <h2>同步</h2>
        <span title={headerStatus}>{headerStatus}</span>
      </div>
      <NetworkSettingsPanel snapshot={network} disabled={disabled || busy} onSaved={policySaved} />
      {networkError ? <p role="alert">{networkError}</p> : null}
      <div className="provider-grid">
        <label>
          <span>Source</span>
          <select value={provider} disabled={disabled || busy} onChange={(event) => handleProviderChange(event.target.value as ProviderKind)}>
            <option value="yike">Yike</option>
            <option value="fox">Fox</option>
            <option value="tencent">Tencent</option>
          </select>
        </label>
      </div>
      {provider === "yike" ? <YikePublicCenter disabled={disabled || busy} network={network} onStartSync={onStartSync} onImport={async (result, identity) => {
        setBusy(true);
        try { await onImport(result, identity); }
        finally { setBusy(false); }
      }} /> : provider === "tencent" ? <TencentKifuCenter disabled={disabled || busy} network={network} onImport={async (result, identity) => {
        setBusy(true);
        try { await onImport(result, identity); }
        finally { setBusy(false); }
      }} /> : <FoxKifuCenter disabled={disabled || busy} network={network} onImport={async (result, identity) => {
        setBusy(true);
        try { await onImport(result, identity); }
        finally { setBusy(false); }
      }} />}
      <label className="provider-payload-label">
        <span>Payload / SGF</span>
        <textarea
          className="provider-payload"
          value={payload}
          disabled={disabled || busy}
          spellCheck={false}
          aria-label="Provider payload or SGF"
          placeholder='Paste raw SGF or JSON with "sgf", "clean_sgf", or "chess".'
          onChange={(event) => {
            setPayload(event.target.value);
            setProviderWarnings([]);
          }}
        />
      </label>
      <button onClick={() => void handleImport()} disabled={!canImport}>Import pasted payload</button>
      <p className="provider-status" title={statuses.import}>{statuses.import}</p>
      <WarningList label="Provider warnings" warnings={providerWarnings} />
      <ReadboardPanel disabled={disabled} />

      <div className="provider-readboard">
        <div className="provider-subheader">
          <h3>Readboard protocol preview</h3>
        </div>
        <label className="provider-payload-label">
          <span>Protocol preview line</span>
          <textarea
            className="provider-payload provider-readboard-line"
            value={readboardProtocolLine}
            disabled={disabled}
            spellCheck={false}
            aria-label="Readboard protocol preview line"
            placeholder="Paste readboard snapshot protocol line for position preview"
            onChange={(event) => setReadboardProtocolLine(event.target.value)}
          />
        </label>
        <button onClick={() => void handleReadboardSync()} disabled={!canSyncReadboard}>Preview snapshot</button>
        <p className="provider-status" title={statuses.readboardSync}>{statuses.readboardSync}</p>
        {readboardSyncResult ? (
          <dl className="provider-preview">
            <div>
              <dt>Snapshot</dt>
              <dd title={readboardSyncResult.snapshot_id}>{readboardSyncResult.snapshot_id}</dd>
            </div>
            <div>
              <dt>Position</dt>
              <dd>{positionStatus(readboardSyncResult)}</dd>
            </div>
            <div>
              <dt>Warnings</dt>
              <dd title={readboardSyncResult.warnings.join("; ")}>{warningCount(readboardSyncResult.warnings)}</dd>
            </div>
          </dl>
        ) : null}
        <WarningList label="Readboard snapshot warnings" warnings={readboardSyncResult?.warnings ?? []} />
      </div>
    </section>
  );
}

function WarningList({ label, warnings }: { label: string; warnings: string[] }) {
  if (warnings.length === 0) return null;
  return (
    <div className="warning-list" role="status" aria-label={label}>
      <strong>{label}</strong>
      <ul>
        {warnings.slice(0, 5).map((warning, index) => (
          <li key={`${index}:${warning}`} title={warning}>{warning}</li>
        ))}
      </ul>
      {warnings.length > 5 ? <small>{warnings.length - 5} more warning(s)</small> : null}
    </div>
  );
}


function buildRequest(provider: ProviderKind, payload: string): ProviderImportRequest {
  return {
    provider,
    payload,
    source_url: null,
    source_id: null,
    metadata: emptyProviderMetadata()
  };
}




function readboardSyncStatus(result: ReadboardSidecarSyncSnapshotResult): string {
  const position = result.position ? `position ${result.position.board_width}x${result.position.board_height} move ${result.position.move_number}` : "no position";
  const warnings = result.warnings.length > 0 ? `, ${result.warnings.length} warning(s)` : "";
  return `Snapshot preview ${result.snapshot_id}: ${position}${warnings}.`;
}

function positionStatus(result: ReadboardSidecarSyncSnapshotResult): string {
  if (!result.position) return "none";
  return `${result.position.board_width}x${result.position.board_height}, move ${result.position.move_number}, ${result.position.stones.length} stones`;
}

function warningCount(warnings: string[]): string {
  return warnings.length === 0 ? "none" : `${warnings.length} warning(s)`;
}

function errorMessage(error: unknown): string {
  if (error instanceof Error) return error.message;
  if (isErrorRecord(error) && typeof error.message === "string") return error.message;
  if (isErrorRecord(error) && typeof error.kind === "string") return error.kind;
  return String(error);
}

function isErrorRecord(value: unknown): value is { kind?: unknown; message?: unknown } {
  return typeof value === "object" && value !== null;
}
