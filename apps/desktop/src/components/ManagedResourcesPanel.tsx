import { useEffect, useRef, useState } from "react";
import { acquireManagedResources, cancelManagedResources, loadManagedResources } from "../api/managedResources";
import { networkSnapshot } from "../api/providers";
import type { EngineProfileDto, ManagedInstallationDto, ManagedResourcesDto } from "../domain/types";
import { t } from "../i18n/resources";

type Props = {
  profileId: string;
  profile: EngineProfileDto | undefined;
  disabled: boolean;
  beginOperation: () => boolean;
  endOperation: () => void;
  onUse: (installation: ManagedInstallationDto) => void;
};
const terminal = (phase: string) => ["succeeded", "failed", "cancelled"].includes(phase);

export function ManagedResourcesPanel(props: Props) {
  const [snapshot, setSnapshot] = useState<ManagedResourcesDto | null>(null);
  const [target, setTarget] = useState("");
  const [model, setModel] = useState("");
  const [busy, setBusy] = useState(false);
  const [failure, setFailure] = useState(false);
  const [policy, setPolicy] = useState("");
  const latest = useRef(props);
  latest.current = props;
  const pending = useRef(false);
  const requesting = useRef(false);
  const ownedId = useRef<string | null>(null);
  const epoch = useRef(0);

  useEffect(() => {
    const generation = ++epoch.current;
    let timer: number | undefined;
    async function refresh() {
      try {
        const next = await loadManagedResources();
        if (epoch.current !== generation) return;
        setSnapshot(next);
        if (!next) return;
        if (next) {
          setTarget((value) => value || next.catalog.targets.find((entry) => entry.acquisition_allowed)?.id || next.catalog.targets[0]?.id || "");
          setModel((value) => value || next.catalog.default_model_id);
          const active = next.operation && !terminal(next.operation.phase);
          if (active && !pending.current && latest.current.beginOperation()) {
            pending.current = true;
            setBusy(true);
          }
          if (!active && pending.current && !requesting.current && (!ownedId.current || ownedId.current === next.operation?.operation_id)) {
            pending.current = false;
            setBusy(false);
            latest.current.endOperation();
          }
        }
      } catch {
        if (epoch.current === generation) setFailure(true);
      }
      if (epoch.current === generation) timer = window.setTimeout(() => void refresh(), 400);
    }
    void refresh();
    return () => {
      ++epoch.current;
      clearTimeout(timer);
      if (pending.current) {
        if (ownedId.current) void cancelManagedResources(ownedId.current).catch(() => {});
        latest.current.endOperation();
      }
    };
  }, []);

  async function acquire() {
    if (pending.current || props.disabled || !props.profile || !props.beginOperation()) return;
    pending.current = true;
    requesting.current = true;
    ownedId.current = null;
    setBusy(true);
    setFailure(false);
    const generation = epoch.current;
    const captured = props;
    try {
      const network = await networkSnapshot();
      if (epoch.current !== generation || latest.current.profileId !== captured.profileId || latest.current.profile !== captured.profile) throw new Error("managed_stale");
      setPolicy(`${network.settings.mode} · ${network.policy_revision}`);
      const id = await acquireManagedResources({ profile_id: captured.profileId, profile: captured.profile!, target_id: target, model_id: model, policy_revision: network.policy_revision });
      if (epoch.current !== generation) { await cancelManagedResources(id); return; }
      ownedId.current = id;
      const next = await loadManagedResources();
      if (epoch.current === generation) setSnapshot(next);
    } catch {
      if (epoch.current !== generation) return;
      setFailure(true);
      pending.current = false;
      setBusy(false);
      latest.current.endOperation();
    } finally {
      requesting.current = false;
    }
  }

  const selected = snapshot?.catalog.targets.find((entry) => entry.id === target);
  const selectedModel = snapshot?.catalog.models.find((entry) => entry.id === model);
  const operation = snapshot?.operation;
  const installation = operation?.installation;
  return <section aria-label={t("managed.title")} style={{ overflowWrap: "anywhere" }}>
    <h3>{t("managed.title")}</h3>
    <p>{t("managed.hint")}</p>
    {!snapshot ? <p role="status">{t("managed.native")}</p> : <>
      <p>schema{snapshot.catalog.schema_version} · {snapshot.catalog.source_commit} · KataGo {snapshot.catalog.katago_version} · {snapshot.catalog.katago_source_commit}</p>
      <p>{snapshot.catalog.engine_repository}@{snapshot.catalog.engine_tag} · models@{snapshot.catalog.model_tag}</p>
      <label>{t("managed.target")}<select aria-label={t("managed.target")} value={target} disabled={props.disabled || busy} onChange={(event) => setTarget(event.target.value)}>
        {snapshot.catalog.targets.map((entry) => <option key={entry.id} value={entry.id}>{entry.id} · {entry.backend}</option>)}
      </select></label>
      <label>{t("managed.model")}<select aria-label={t("managed.model")} value={model} disabled={props.disabled || busy} onChange={(event) => setModel(event.target.value)}>
        {snapshot.catalog.models.map((entry) => <option key={entry.id} value={entry.id}>{entry.file_name}</option>)}
      </select></label>
      {selected && <>
        <p>{t("managed.source")}: {selected.source_availability} · {t("managed.artifact")}: {selected.artifact_availability}</p>
        <p>{t("managed.hardware")}: {selected.hardware_qualification} · {t("managed.runtime")}: {selected.runtime_acceptance}</p>
        <p>{selected.archive} · {selected.size_bytes} bytes · SHA256 {selected.sha256}</p>
        <p>Executable SHA256 {selected.executable_sha256}</p>
        {!selected.acquisition_allowed && <p>{t("managed.unqualified")}</p>}
      </>}
      {selectedModel && <p>{selectedModel.file_name} · {selectedModel.size_bytes} bytes · minimum KataGo {selectedModel.minimum_katago_version} · SHA256 {selectedModel.sha256}</p>}
      <p>{t("managed.integrity")}</p><p>{t("managed.guidance")}</p>
      <button type="button" disabled={props.disabled || busy || !props.profile || !selected?.acquisition_allowed || !selectedModel} onClick={() => void acquire()}>{t("managed.acquire")}</button>
      {operation && <div aria-live="polite">
        <p>{operation.operation_id} · {operation.target_id} · {t(`managed.phase.${operation.phase}`)}</p>
        <progress aria-label={t("managed.title")} value={operation.transferred_bytes} max={operation.total_bytes} />
        <p>{operation.transferred_bytes} / {operation.total_bytes} bytes</p>
        <p>{t("managed.network")}: {policy || operation.routes.map((route) => `${route.mode}/${route.source}`).join(" → ")}</p>
        {operation.message && <p>{operation.message}</p>}
        {!terminal(operation.phase) && <button type="button" onClick={() => void cancelManagedResources(operation.operation_id).catch(() => setFailure(true))}>{t("managed.cancel")}</button>}
        {installation && operation.profile_id === props.profileId && operation.target_id === target && operation.model_id === model && <button type="button" disabled={props.disabled || busy} onClick={() => props.onUse(installation)}>{t("managed.use")}</button>}
      </div>}
    </>}
    {failure && <p role="alert">{t("managed.failure")}</p>}
  </section>;
}
