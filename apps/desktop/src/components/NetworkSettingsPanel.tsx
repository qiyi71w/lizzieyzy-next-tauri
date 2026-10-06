import type { FormEvent } from "react";
import { useEffect, useRef, useState } from "react";
import type { NetworkSettings, NetworkSnapshot } from "../domain/providers";
import { saveNetworkSettings } from "../api/providers";

type Props = {
  snapshot: NetworkSnapshot | null;
  disabled?: boolean;
  onSaved: (snapshot: NetworkSnapshot) => void;
};

const MODE_EXPLANATIONS: Record<NetworkSettings["mode"], string> = {
  direct: "Direct: Direct ignores env",
  system: "System: System env then platform PAC",
  manual: "Manual: HTTP CONNECT, NO_PROXY respected"
};

function isLikelyIpv6(host: string): boolean {
  const trimmed = host.replace(/^\[|\]$/g, "");
  const colons = (trimmed.match(/:/g) || []).length;
  return colons >= 2;
}

function validateNetworkDraft(
  mode: NetworkSettings["mode"],
  rawHost: string,
  rawPort: string
): { error: string } | { host: string; port: number } {
  const host = rawHost.trim();
  const portStr = rawPort.trim();

  if (
    host.includes("@") ||
    host.includes("://") ||
    host.includes("/") ||
    host.includes("\\") ||
    host.includes("?") ||
    host.includes("#") ||
    /\s/.test(host)
  ) {
    return {
      error: "Proxy host must contain only a hostname or IP address, without credentials or URL syntax."
    };
  }

  if (host.includes(":") && !isLikelyIpv6(host)) {
    return {
      error: "Enter the proxy port separately from its host."
    };
  }

  const effectivePortStr = portStr || (mode !== "manual" ? "7897" : "");
  if (!/^\d+$/.test(effectivePortStr)) {
    return {
      error: mode === "manual"
        ? "Manual proxy requires a nonempty host and port 1–65535."
        : "Proxy port must be an integer between 1 and 65535."
    };
  }

  const portNum = Number.parseInt(effectivePortStr, 10);
  if (Number.isNaN(portNum) || portNum < 1 || portNum > 65535) {
    return {
      error: mode === "manual"
        ? "Manual proxy requires a nonempty host and port 1–65535."
        : "Proxy port must be an integer between 1 and 65535."
    };
  }

  const effectiveHost = host || (mode !== "manual" ? "127.0.0.1" : "");
  if (mode === "manual" && effectiveHost.length === 0) {
    return {
      error: "Manual proxy requires a nonempty host and port 1–65535."
    };
  }

  return { host: effectiveHost, port: portNum };
}

export function NetworkSettingsPanel({ snapshot, disabled = false, onSaved }: Props) {
  const [draftMode, setDraftMode] = useState<NetworkSettings["mode"]>(
    () => snapshot?.settings.mode ?? "direct"
  );
  const [draftHost, setDraftHost] = useState<string>(
    () => snapshot?.settings.manual_host || "127.0.0.1"
  );
  const [draftPort, setDraftPort] = useState<string>(
    () => String(snapshot?.settings.manual_port || 7897)
  );
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState("");

  const lastSyncedSnapshot = useRef<NetworkSnapshot | null>(snapshot);

  useEffect(() => {
    if (snapshot !== lastSyncedSnapshot.current) {
      lastSyncedSnapshot.current = snapshot;
      if (snapshot) {
        setDraftMode(snapshot.settings.mode);
        setDraftHost(snapshot.settings.manual_host || "127.0.0.1");
        setDraftPort(String(snapshot.settings.manual_port || 7897));
        setError("");
      }
    }
  }, [snapshot]);

  const isFormDisabled = disabled || snapshot === null || saving;

  async function handleSave(event?: FormEvent) {
    if (event) {
      event.preventDefault();
    }
    if (isFormDisabled) {
      return;
    }

    const validation = validateNetworkDraft(draftMode, draftHost, draftPort);
    if ("error" in validation) {
      setError(validation.error);
      return;
    }

    setSaving(true);
    setError("");

    try {
      const updated = await saveNetworkSettings({
        mode: draftMode,
        manual_host: validation.host,
        manual_port: validation.port
      });
      lastSyncedSnapshot.current = updated;
      setDraftMode(updated.settings.mode);
      setDraftHost(updated.settings.manual_host || "127.0.0.1");
      setDraftPort(String(updated.settings.manual_port || 7897));
      setError("");
      onSaved(updated);
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message || "Failed to save network settings.");
    } finally {
      setSaving(false);
    }
  }

  return (
    <div className="network-settings-panel" aria-label="Network Settings">
      <div className="network-settings-header">
        <span>
          Effective Mode:{" "}
          <strong data-testid="effective-mode">
            {snapshot ? snapshot.settings.mode : "none"}
          </strong>
        </span>
        <span>
          Revision:{" "}
          <strong data-testid="effective-revision">
            {snapshot ? snapshot.policy_revision : "-"}
          </strong>
        </span>
      </div>

      {snapshot === null ? (
        <p className="network-settings-notice">Native network settings unavailable.</p>
      ) : null}

      <div className="network-settings-controls">
        <label>
          <span>Mode</span>
          <select
            aria-label="Mode"
            value={draftMode}
            disabled={isFormDisabled}
            onChange={(event) => {
              setDraftMode(event.target.value as NetworkSettings["mode"]);
              setError("");
            }}
          >
            <option value="direct">Direct</option>
            <option value="system">System</option>
            <option value="manual">Manual</option>
          </select>
        </label>

        <label>
          <span>Host</span>
          <input
            type="text"
            aria-label="Host"
            value={draftHost}
            disabled={isFormDisabled || draftMode !== "manual"}
            placeholder="127.0.0.1"
            onChange={(event) => {
              setDraftHost(event.target.value);
              setError("");
            }}
          />
        </label>

        <label>
          <span>Port</span>
          <input
            type="text"
            aria-label="Port"
            value={draftPort}
            disabled={isFormDisabled || draftMode !== "manual"}
            placeholder="7897"
            onChange={(event) => {
              setDraftPort(event.target.value);
              setError("");
            }}
          />
        </label>

        <button
          type="button"
          aria-label="Save"
          disabled={isFormDisabled}
          onClick={() => void handleSave()}
        >
          Save
        </button>
      </div>

      {error ? (
        <p role="alert" className="network-settings-error" data-testid="network-settings-error">
          {error}
        </p>
      ) : null}

      <div className="network-settings-explanations" aria-label="Mode Explanations">
        <p className="network-settings-mode-explanation" data-testid="mode-explanation">
          {MODE_EXPLANATIONS[draftMode]}
        </p>
        <ul className="network-settings-all-explanations">
          <li>Direct: Direct ignores env</li>
          <li>System: System env then platform PAC</li>
          <li>Manual: HTTP CONNECT, NO_PROXY respected</li>
        </ul>
      </div>
    </div>
  );
}
