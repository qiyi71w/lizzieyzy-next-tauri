import { useEffect, useRef, useState } from "react";
import { confirmOrdinaryRules } from "../api/backend";
import type { EngineRunDto, NodePath, OrdinaryRulesSnapshotDto } from "../domain/types";
import { t } from "../i18n/resources";

type Props = {
  run: EngineRunDto | null;
  generation: number | null;
  nodePath: NodePath;
  native: boolean;
  disabled: boolean;
};

export function OrdinaryRulesPanel({ run, generation, nodePath, native, disabled }: Props) {
  const [pending, setPending] = useState(false);
  const [receipt, setReceipt] = useState<OrdinaryRulesSnapshotDto | null>(null);
  const [error, setError] = useState("");
  const identity = JSON.stringify([run?.run_id, generation, nodePath.indices]);
  const current = useRef(identity);
  current.current = identity;
  const round = useRef(0);
  useEffect(() => {
    round.current += 1;
    setReceipt(null); setError(""); setPending(false);
    return () => { round.current += 1; };
  }, [identity]);
  const available = native && run?.adapter_kind === "kata_go_gtp" && generation !== null;
  const visible = receipt && receipt.identity.run_id === run?.run_id
    && receipt.identity.generation === generation
    && JSON.stringify(receipt.identity.node_path.indices) === JSON.stringify(nodePath.indices) ? receipt : null;
  async function confirm() {
    if (!available || disabled || pending || !run || generation === null) return;
    const requestRound = ++round.current;
    const captured = identity;
    setPending(true); setReceipt(null); setError("");
    try {
      const result = await confirmOrdinaryRules({ run_id: run.run_id, generation, node_path: nodePath });
      if (round.current === requestRound && current.current === captured) setReceipt(result);
    } catch (reason) {
      if (round.current === requestRound && current.current === captured) {
        const message = reason && typeof reason === "object" && "message" in reason && typeof reason.message === "string"
          ? reason.message.slice(0, 1024) : t("engineRules.rejected");
        setError(message);
      }
    } finally {
      if (round.current === requestRound && current.current === captured) setPending(false);
    }
  }
  return <section aria-label={t("engineRules.title")} className="message ordinary-rules-panel">
    <h3>{t("engineRules.title")}</h3>
    <p>{t("engineRules.boundary")}</p>
    <button type="button" disabled={!available || disabled || pending} onClick={() => void confirm()}>{t("engineRules.confirm")}</button>
    <p role="status">{pending ? t("engineRules.pending") : visible ? t("engineRules.confirmed") : t("engineRules.unconfirmed")}</p>
    {!available && <p>{t("engineRules.unavailable")}</p>}
    {error && <p role="alert">{t("engineRules.rejected")}: {error}</p>}
    {visible && <dl>
      <dt>{t("engineRules.identity")}</dt><dd>{visible.identity.run_id} / {visible.identity.job_id} / {visible.identity.generation}:{visible.identity.node_path.indices.join(".") || "root"}</dd>
      <dt>{t("engineRules.position")}</dt><dd>{visible.position.komi} / {visible.position.to_play} / {visible.position.moves.length}</dd>
      <dt>{t("engineRules.rules")}</dt><dd>{visible.confirmed_rules}</dd>
    </dl>}
  </section>;
}
