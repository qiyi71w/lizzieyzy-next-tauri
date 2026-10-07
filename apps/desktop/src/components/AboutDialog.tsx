import { useEffect, useRef, useState } from "react";
import { getBuildIdentity, openBuildAddress, type BuildIdentity } from "../api/buildIdentity";
import { scheduleOwnedFocus } from "../domain/focusNavigation";
import { t } from "../i18n/resources";

export function AboutDialog({ onClose }: { onClose: () => void }) {
  const [identity, setIdentity] = useState<BuildIdentity | null>(null);
  const [failure, setFailure] = useState<string | null>(null);
  const ownerRef = useRef<HTMLDivElement>(null);
  const closeRef = useRef<HTMLButtonElement>(null);
  useEffect(() => {
    let active = true;
    getBuildIdentity().then((value) => { if (active) setIdentity(value); }).catch((error) => { if (active) setFailure(String(error)); });
    const cancel = ownerRef.current ? scheduleOwnedFocus(ownerRef.current, () => closeRef.current) : undefined;
    return () => { active = false; cancel?.(); };
  }, []);
  return <div className="shortcut-reference-backdrop" onClick={onClose}>
    <div ref={ownerRef} className="new-document-dialog" role="dialog" aria-label={t("about.title")} aria-modal="true"
      onClick={(event) => event.stopPropagation()} onKeyDown={(event) => { if (event.key === "Escape") { event.stopPropagation(); onClose(); } }}>
      <h2>{t("about.title")}</h2>
      {!identity && !failure ? <p role="status">{t("about.loading")}</p> : null}
      {identity ? <>
        <p>{t(identity.native ? "about.version" : "about.preview")}: <strong>{identity.version}</strong></p>
        {([ ["source_repository", "about.source"], ["issues", "about.issues"], ["help", "about.help"] ] as const).map(([kind, label]) =>
          <p key={kind}><a href={identity.addresses[kind]} target="_blank" rel="noopener noreferrer" onClick={(event) => {
            if (!identity.native) return;
            event.preventDefault();
            void openBuildAddress(kind).catch((error) => setFailure(String(error)));
          }}>{t(label)}</a><br /><small>{identity.addresses[kind]}</small></p>)}
        <p>{t("about.release")}</p>
      </> : null}
      {failure ? <p role="alert">{t("about.failure")}{failure}</p> : null}
      <button ref={closeRef} type="button" onClick={onClose}>{t("about.close")}</button>
    </div>
  </div>;
}
