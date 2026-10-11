import { useEffect, useRef, useState } from "react";
import { getThreadPolicyStatus, saveThreadPolicy } from "../api/threadPolicy";
import { isTauriRuntime } from "../api/backend";
import type { EngineProfileRecordDto, ThreadPolicyStatusDto, ThreadStrategyDto } from "../domain/types";
import { baseResources, t, type ResourceKey } from "../i18n/resources";

export function threadPolicyMessage(value: unknown): string {
  const message = value && typeof value === "object" && "message" in value ? String(value.message) : String(value);
  const key = `threadPolicy.reason.${message}`;
  return key in baseResources ? t(key as ResourceKey) : message;
}

type Props = {
  record: EngineProfileRecordDto;
  evaluationId: string | null;
  disabled: boolean;
  onSaved?: (profiles: EngineProfileRecordDto[]) => void;
  onBusyChange?: (busy: boolean) => void;
};

export function EngineThreadPolicyPanel({ record, evaluationId, disabled, onSaved, onBusyChange }: Props) {
  const [status, setStatus] = useState<ThreadPolicyStatusDto | null>(null);
  const [strategy, setStrategy] = useState<ThreadStrategyDto>(record.thread_policy?.strategy ?? "cfg");
  const [refresh, setRefresh] = useState(0);
  const [pending, setPending] = useState(false);
  const [feedback, setFeedback] = useState("");
  const generation = useRef(0);
  const writing = useRef(false);
  const native = isTauriRuntime();
  useEffect(() => {
    const token = ++generation.current;
    setStatus(null);
    setStrategy(record.thread_policy?.strategy ?? "cfg");
    if (!native) return;
    void getThreadPolicyStatus(record.id, evaluationId).then((next) => {
      if (generation.current === token && next.target_id === record.id) setStatus(next);
    }).catch((error: unknown) => {
      if (generation.current === token) setFeedback(threadPolicyMessage(error));
    });
    return () => { generation.current++; };
  }, [record.id, record.profile, record.thread_policy?.revision, evaluationId, refresh, native]);

  async function save() {
    if (!status || disabled || writing.current) return;
    writing.current = true;
    setPending(true);
    onBusyChange?.(true);
    setFeedback("");
    const token = generation.current;
    try {
      const saved = await saveThreadPolicy({ target_id: status.target_id, expected_revision: status.policy.revision,
        input_revision: status.input_revision, strategy, evaluation_id: evaluationId });
      if (generation.current !== token) return;
      onSaved?.(saved.profiles);
      setFeedback(t("threadPolicy.saved"));
      setRefresh((value) => value + 1);
    } catch (error) {
      if (generation.current === token) setFeedback(`${t("threadPolicy.failed")}${threadPolicyMessage(error)}`);
    } finally {
      writing.current = false;
      onBusyChange?.(false);
      if (generation.current === token) setPending(false);
    }
  }

  return <section aria-label={t("threadPolicy.title")} aria-busy={pending}>
    <h4>{t("threadPolicy.title")}: {record.profile.name} ({record.id})</h4>
    <p>{t("threadPolicy.hint")}</p>
    <label>{t("threadPolicy.strategy")}
      <select aria-label={t("threadPolicy.strategy")} value={strategy} disabled={disabled || pending}
        onChange={(event) => setStrategy(event.target.value === "benchmark" ? "benchmark" : "cfg")}>
        <option value="cfg">{t("threadPolicy.cfg")}</option>
        <option value="benchmark">{t("threadPolicy.benchmark")}</option>
      </select>
    </label>
    {status ? <div role="status">
      <p>{t(`threadPolicy.${status.validity}`)} — {threadPolicyMessage(status.reason)}</p>
      {status.recommendation && <p>{t("threadPolicy.source")}: {status.recommendation.source_revision} / {status.recommendation.seconds_per_move} / {status.recommendation.threads}<br />{status.recommendation.source_row}</p>}
    </div> : <p role="status">{native ? t("threadPolicy.loading") : threadPolicyMessage("policy_native_only")}</p>}
    <button type="button" disabled={!native || !status || disabled || pending || (strategy === "benchmark" && status.validity !== "valid")}
      onClick={() => void save()}>{t("threadPolicy.save")}</button>
    <button type="button" disabled={pending} onClick={() => { setStrategy(record.thread_policy?.strategy ?? "cfg"); setFeedback(""); }}>{t("threadPolicy.cancel")}</button>
    <button type="button" disabled={!native || pending} onClick={() => setRefresh((value) => value + 1)}>{t("threadPolicy.refresh")}</button>
    {feedback && <p role="alert">{feedback}</p>}
  </section>;
}
