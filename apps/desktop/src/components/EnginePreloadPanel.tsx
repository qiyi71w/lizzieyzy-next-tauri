import { useEffect, useState } from "react";
import { isTauriRuntime } from "../api/backend";
import { cancelEnginePreload, enginePreloadSnapshot, prepareEnginePreload } from "../api/enginePreload";
import type { EnginePreloadDto, EngineProfileRecordDto } from "../domain/types";
import { t } from "../i18n/resources";

export function EnginePreloadPanel({ profiles, disabled }: { profiles: EngineProfileRecordDto[]; disabled: boolean }) {
  const [preloads, setPreloads] = useState<EnginePreloadDto[]>([]);
  const [pending, setPending] = useState<string | null>(null);
  const [error, setError] = useState("");
  useEffect(() => {
    let active = true;
    let timer: number;
    async function refresh() {
      try {
        const snapshot = await enginePreloadSnapshot();
        if (active) setPreloads(snapshot);
      } catch {
        if (active) setError(t("enginePreload.unavailable"));
      } finally {
        if (active) timer = window.setTimeout(() => void refresh(), 300);
      }
    }
    void refresh();
    return () => { active = false; clearTimeout(timer); };
  }, []);

  async function act(profileId: string, cancel: boolean) {
    setPending(profileId);
    setError("");
    try {
      if (cancel) await cancelEnginePreload(profileId);
      else await prepareEnginePreload(profileId);
      setPreloads(await enginePreloadSnapshot());
    } catch (failure) {
      setError(failure && typeof failure === "object" && "message" in failure
        ? String(failure.message) : t("enginePreload.unavailable"));
    } finally {
      setPending(null);
    }
  }

  return <section aria-label={t("enginePreload.title")}>
    <h3>{t("enginePreload.title")}</h3>
    <p className="engine-resource-message">{t("enginePreload.hint")}</p>
    {!isTauriRuntime() ? <p>{t("enginePreload.desktopOnly")}</p> : null}
    {profiles.filter((profile) => profile.preload || preloads.some((slot) => slot.run.profile_id === profile.id)).map((profile) => {
      const slot = preloads.find((entry) => entry.run.profile_id === profile.id);
      const running = slot?.phase === "preparing" || slot?.phase === "ready";
      return <div key={profile.id}>
        <strong>{profile.profile.name}</strong>{" "}
        <span role="status">{slot ? t(`enginePreload.${slot.phase}`) : t("enginePreload.notPrepared")}</span>{" "}
        <button type="button" disabled={disabled || pending !== null || running || !profile.preload || !isTauriRuntime()}
          onClick={() => void act(profile.id, false)}>{t("enginePreload.prepare")}</button>{" "}
        <button type="button" disabled={pending !== null || !running || !isTauriRuntime()}
          onClick={() => void act(profile.id, true)}>{t("enginePreload.cancel")}</button>
        {slot?.failure ? <p role="alert" className="engine-resource-message">{slot.failure.message} {slot.failure.diagnostic_summary}</p> : null}
      </div>;
    })}
    {error ? <p role="alert" className="engine-resource-message">{error}</p> : null}
  </section>;
}
