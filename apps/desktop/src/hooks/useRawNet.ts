import { useRef, useState } from "react";
import { computeRawNet } from "../api/rawNet";
import type { RawNetRequestDto, RawNetSnapshotDto } from "../domain/types";
import { t } from "../i18n/resources";

export type RawNetPresentation = {
  payload: RawNetSnapshotDto | null;
  pending: boolean;
  error: string;
  compute: () => Promise<void>;
};

/** Display changes are deliberately not inputs: only explicit Compute issues IPC. */
export function useRawNet(scope: string, request: RawNetRequestDto | null) {
  const current = useRef({ scope, revision: 0, request });
  if (current.current.scope !== scope) {
    current.current = { scope, revision: current.current.revision + 1, request };
  } else current.current.request = request;
  const [state, setState] = useState<{ revision: number; pending: boolean; payload: RawNetSnapshotDto | null; error: string }>({
    revision: -1, pending: false, payload: null, error: "",
  });
  const visible = state.revision === current.current.revision ? state : null;
  async function compute() {
    const captured = current.current.request;
    const revision = ++current.current.revision;
    setState({ revision, pending: Boolean(captured), payload: null, error: captured ? "" : t("display.rawNeedRun") });
    if (!captured) return;
    try {
      const payload = await computeRawNet(captured);
      if (revision !== current.current.revision) return;
      const identity = payload.confirmation.identity;
      if (identity.run_id !== captured.run_id || identity.generation !== captured.generation
        || JSON.stringify(identity.node_path) !== JSON.stringify(captured.node_path)) {
        throw new Error("Raw-net result identity changed.");
      }
      setState({ revision, pending: false, payload, error: payload.unavailable ?? "" });
    } catch (error) {
      if (revision !== current.current.revision) return;
      const message = error && typeof error === "object" && "message" in error ? String(error.message) : String(error);
      setState({ revision, pending: false, payload: null, error: `${message} · ${t("display.rawPause")}` });
    }
  }
  return { payload: visible?.payload ?? null, pending: visible?.pending ?? false, error: visible?.error ?? "", compute };
}
