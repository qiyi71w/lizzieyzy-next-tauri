import { useEffect, useRef, useState } from "react";
import { loadModelInventory, refreshModelInventory, selectInstalledModel } from "../api/models";
import type { InstalledModelDto, ModelInventoryDto } from "../domain/types";
import { t } from "../i18n/resources";

type Props = {
  modelPath: string;
  savedModelPath: string;
  workingDir: string;
  disabled: boolean;
  beginOperation: () => boolean;
  endOperation: () => void;
  onSelect: (path: string) => void;
};

export function ModelInventoryPanel(props: Props) {
  const [inventory, setInventory] = useState<ModelInventoryDto | null>(null);
  const [busy, setBusy] = useState(false);
  const [stale, setStale] = useState(false);
  const [message, setMessage] = useState<"selected" | "failed" | null>(null);
  const epoch = useRef(0);
  const pending = useRef(false);
  const latest = useRef(props);
  latest.current = props;

  useEffect(() => {
    const request = ++epoch.current;
    void loadModelInventory().then((result) => {
      if (epoch.current === request) setInventory(result);
    }).catch(() => {
      if (epoch.current === request) { setStale(true); setMessage("failed"); }
    });
    return () => {
      ++epoch.current;
      if (pending.current) latest.current.endOperation();
    };
  }, []);

  async function operate(model?: InstalledModelDto) {
    if (pending.current || latest.current.disabled || !latest.current.beginOperation()) return;
    pending.current = true;
    setBusy(true);
    setMessage(null);
    const request = ++epoch.current;
    const captured = latest.current;
    try {
      if (model) {
        if (stale || !inventory || model.inspection.status !== "header_recognized" || !model.inspection.sha256) return;
        const path = await selectInstalledModel({ revision: inventory.revision, model_id: model.id, sha256: model.inspection.sha256 });
        if (epoch.current !== request) return;
        if (latest.current.disabled || latest.current.modelPath !== captured.modelPath || latest.current.workingDir !== captured.workingDir) {
          setStale(true);
          return;
        }
        latest.current.onSelect(path);
        setMessage("selected");
      } else {
        setStale(true);
        const result = await refreshModelInventory(captured.modelPath.length > 0
          ? [{ path: captured.modelPath, working_dir: captured.workingDir || null }] : []);
        if (epoch.current !== request) return;
        setInventory(result);
        setStale(false);
      }
    } catch {
      if (epoch.current === request) { setStale(true); setMessage("failed"); }
    } finally {
      if (epoch.current === request) {
        pending.current = false;
        setBusy(false);
        latest.current.endOperation();
      }
    }
  }

  const browser = inventory?.revision === "browser-unavailable";
  return <section aria-label={t("models.title")} aria-busy={busy}>
    <h3>{t("models.title")}</h3>
    <p className="message">{t("models.hint")}</p>
    <p className="message">{t("models.scope")}</p>
    <p style={{ overflowWrap: "anywhere" }}>{t("models.saved")}: {props.savedModelPath || t("models.none")}</p>
    <p style={{ overflowWrap: "anywhere" }}>{t("models.draft")}: {props.modelPath || t("models.none")}</p>
    <button type="button" disabled={props.disabled || busy || browser} onClick={() => void operate()}>{t("models.refresh")}</button>
    {browser && <p role="status">{t("models.native")}</p>}
    {busy && <p role="status">{t("models.busy")}</p>}
    {stale && <p role="status">{t("models.stale")}</p>}
    {message && (message !== "selected" || props.modelPath !== props.savedModelPath) && <p role="status">{t(`models.${message}`)}</p>}
    <ul>
      {inventory?.models.map((model) => <li key={model.id} style={{ overflowWrap: "anywhere" }}>
        <p>{t("models.name")}: {model.inspection.model_name ?? t("models.status.unknown")}</p>
        <p>{model.path}</p>
        <p>{t(`models.origin.${model.origin.kind}`)}{model.origin.kind === "managed" ? ` · ${model.origin.catalog_id} · SHA256 ${model.origin.installed_sha256}` : ""}</p>
        <p>{t(`models.status.${model.inspection.status}`)}</p>
        {model.inspection.format && <p>{t("models.format")}: {model.inspection.format} · {t("models.version")}: {model.inspection.format_version}</p>}
        {model.inspection.sha256 && <p>SHA256: {model.inspection.sha256}</p>}
        <button type="button" disabled={props.disabled || busy || stale || model.inspection.status !== "header_recognized"}
          aria-label={`${t("models.use")} ${model.path}`} onClick={() => void operate(model)}>{t("models.use")}</button>
      </li>)}
    </ul>
  </section>;
}
