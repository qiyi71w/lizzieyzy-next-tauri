import { useEffect, useRef, useState } from "react";
import { acquireManagedResources, cancelManagedResources, inspectManagedTrtRepair, loadManagedResources, managedRepairDraft, repairManagedTrt } from "../api/managedResources";
import { networkSnapshot } from "../api/providers";
import type { EngineProfileDto, ManagedInstallationDto, ManagedRepairPreviewDto, ManagedResourcesDto } from "../domain/types";
import { t } from "../i18n/resources";

type Props = {
  profileId: string;
  profile: EngineProfileDto | undefined;
  disabled: boolean;
  beginOperation: () => boolean;
  endOperation: () => void;
  onUse: (installation: ManagedInstallationDto, repairProfile?: EngineProfileDto) => void;
};
const terminal = (phase: string) => ["succeeded", "failed", "cancelled"].includes(phase);

export function ManagedResourcesPanel(props: Props) {
  const [snapshot, setSnapshot] = useState<ManagedResourcesDto | null>(null);
  const [target, setTarget] = useState("");
  const [model, setModel] = useState("");
  const [busy, setBusy] = useState(false);
  const [failure, setFailure] = useState(false);
  const [policy, setPolicy] = useState("");
  const [preview, setPreview] = useState<ManagedRepairPreviewDto | null>(null);
  const [checking, setChecking] = useState(false);
  const [repairError, setRepairError] = useState("");
  const applying = useRef(false);
  const selectionKey = JSON.stringify([props.profileId, props.profile, target, model]);
  const selection = useRef(selectionKey);
  const selectionVersion = useRef(0);
  if (selection.current !== selectionKey) {
    selection.current = selectionKey;
    ++selectionVersion.current;
  }
  useEffect(() => { setPreview(null); setRepairError(""); }, [selectionKey]);
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

  async function inspectRepair() {
    if (checking || pending.current || props.disabled || !props.profile) return;
    const captured = props;
    const key = selectionKey;
    const version = selectionVersion.current;
    const generation = epoch.current;
    setChecking(true);
    setPreview(null);
    setRepairError("");
    try {
      const network = await networkSnapshot();
      if (selection.current !== key || selectionVersion.current !== version || epoch.current !== generation) return;
      const result = await inspectManagedTrtRepair({ profile_id: captured.profileId, profile: captured.profile!, target_id: target, model_id: model, policy_revision: network.policy_revision });
      if (selection.current === key && selectionVersion.current === version && epoch.current === generation) setPreview(result);
    } catch (error) {
      if (selection.current === key && selectionVersion.current === version && epoch.current === generation) setRepairError(String(error));
    } finally {
      if (epoch.current === generation) setChecking(false);
    }
  }

  async function acquire(admissionId?: string) {
    if (pending.current || props.disabled || !props.profile || !props.beginOperation()) return;
    pending.current = true;
    requesting.current = true;
    ownedId.current = null;
    setBusy(true);
    setFailure(false);
    const generation = epoch.current;
    const captured = props;
    try {
      let id: string;
      if (admissionId) {
        id = await repairManagedTrt(admissionId);
      } else {
        const network = await networkSnapshot();
        if (epoch.current !== generation || latest.current.profileId !== captured.profileId || latest.current.profile !== captured.profile) throw new Error("managed_stale");
        setPolicy(`${network.settings.mode} · ${network.policy_revision}`);
        id = await acquireManagedResources({ profile_id: captured.profileId, profile: captured.profile!, target_id: target, model_id: model, policy_revision: network.policy_revision });
      }
      if (epoch.current !== generation) { await cancelManagedResources(id); return; }
      ownedId.current = id;
      const next = await loadManagedResources();
      if (epoch.current === generation) setSnapshot(next);
    } catch (error) {
      if (epoch.current !== generation) return;
      setFailure(true);
      if (admissionId) setRepairError(String(error));
      pending.current = false;
      setBusy(false);
      latest.current.endOperation();
    } finally {
      requesting.current = false;
    }
  }

  async function useInstallation(installation: ManagedInstallationDto, operationId: string) {
    if (props.disabled || busy || applying.current) return;
    if (!installation.repair_config_path) { props.onUse(installation); return; }
    applying.current = true;
    setChecking(true);
    const version = selectionVersion.current;
    const generation = epoch.current;
    try {
      const draft = await managedRepairDraft(operationId);
      if (selectionVersion.current !== version || epoch.current !== generation || latest.current.disabled) return;
      latest.current.onUse(installation, draft);
    } catch (error) {
      if (selectionVersion.current === version && epoch.current === generation) setRepairError(String(error));
    } finally {
      applying.current = false;
      if (epoch.current === generation) setChecking(false);
    }
  }

  const selected = snapshot?.catalog.targets.find((entry) => entry.id === target);
  const selectedModel = snapshot?.catalog.models.find((entry) => entry.id === model);
  const operation = snapshot?.operation;
  const installation = operation?.installation;
  const currentPreview = preview && JSON.stringify([preview.request.profile_id, preview.request.profile, preview.request.target_id, preview.request.model_id]) === selectionKey ? preview : null;
  return <section className="managed-resources" aria-label={t("managed.title")}>
    <h3>{t("managed.title")}</h3>
    <p>{t("managed.hint")}</p>
    {!snapshot ? <p role="status">{t("managed.native")}</p> : <>
      <p>schema{snapshot.catalog.schema_version} · {snapshot.catalog.source_commit} · KataGo {snapshot.catalog.katago_version} · {snapshot.catalog.katago_source_commit}</p>
      <p>{snapshot.catalog.engine_repository}@{snapshot.catalog.engine_tag} · models@{snapshot.catalog.model_tag}</p>
      <fieldset className="managed-group"><legend>{t("managed.selection")}</legend>
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
      </fieldset>
      {target === "windows-tensorrt" && <fieldset className="managed-group">
        <legend>{t("managed.repair.title")}</legend>
        <p>{t("managed.repair.impact")}</p>
        <button type="button" disabled={props.disabled || busy || checking || !props.profile} onClick={() => void inspectRepair()}>{t(checking ? "managed.repair.checking" : "managed.repair.inspect")}</button>
        {currentPreview && <div role="status">
          <p>{currentPreview.request.profile_id} · {currentPreview.request.target_id} · {currentPreview.runtime_version}</p>
          <p>{t("managed.hardware")}: {currentPreview.hardware.status} · {currentPreview.hardware.gpu_name ?? t("managed.repair.unknown")}</p>
          <p>{t("managed.repair.driver")}: {currentPreview.hardware.driver_version ?? t("managed.repair.unknown")} · {t("managed.repair.compute")}: {currentPreview.hardware.compute_capability ?? t("managed.repair.unknown")}</p>
          <p>{t("managed.repair.download")}: {currentPreview.download_bytes.toLocaleString()} bytes</p>
          <p>{t("managed.repair.disk")}: {currentPreview.additional_disk_bytes.toLocaleString()} bytes · {t("managed.repair.available")}: {currentPreview.available_disk_bytes?.toLocaleString() ?? t("managed.repair.unknown")}</p>
          <p>{t("managed.repair.qualification")}</p>
          <p>{currentPreview.reason} · {currentPreview.hardware.reason}</p>
          <button type="button" disabled={props.disabled || busy || checking || !currentPreview.repair_allowed} onClick={() => void acquire(currentPreview.admission_id)}>{t("managed.repair.start")}</button>
        </div>}
        {repairError && <p role="alert">{t("managed.repair.failure")}: {repairError}</p>}
      </fieldset>}
      {operation && <fieldset className="managed-group" aria-live="polite"><legend>{t("managed.status")}</legend>
        <p>{operation.operation_id} · {operation.target_id} · {t(`managed.phase.${operation.phase}`)}</p>
        {operation.repair_hardware && <p>{operation.repair_hardware.gpu_name} · {t("managed.repair.driver")}: {operation.repair_hardware.driver_version} · {operation.profile_id}</p>}
        <progress aria-label={t("managed.title")} value={operation.transferred_bytes} max={operation.total_bytes} />
        <p>{operation.transferred_bytes} / {operation.total_bytes} bytes</p>
        <p>{t("managed.network")}: {policy || operation.routes.map((route) => `${route.mode}/${route.source}`).join(" → ")}</p>
        {operation.message && <p>{operation.message}</p>}
        {!terminal(operation.phase) && <button type="button" onClick={() => void cancelManagedResources(operation.operation_id).catch(() => setFailure(true))}>{t("managed.cancel")}</button>}
        {installation && operation.profile_id === props.profileId && operation.target_id === target && operation.model_id === model && <button type="button" disabled={props.disabled || busy || checking} onClick={() => void useInstallation(installation, operation.operation_id)}>{t("managed.use")}</button>}
      </fieldset>}
    </>}
    {failure && <p role="alert">{t("managed.failure")}</p>}
  </section>;
}
