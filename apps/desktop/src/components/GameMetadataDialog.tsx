import { useEffect, useRef, useState } from "react";
import { scheduleOwnedFocus } from "../domain/focusNavigation";
import { t } from "../i18n/resources";

type Props = {
  blackName: string;
  whiteName: string;
  komi: number;
  handicap: string | null;
  focusTarget?: "game.komi" | "game.black-name" | "game.white-name";
  onTargetUnavailable?: () => void;
  onApply: (blackName: string, whiteName: string, komi: number) => Promise<void>;
  onCancel: () => void;
};

export function GameMetadataDialog({ blackName, whiteName, komi, handicap, onApply, onCancel, focusTarget = "game.komi", onTargetUnavailable }: Props) {
  const [blackDraft, setBlackDraft] = useState(blackName);
  const [whiteDraft, setWhiteDraft] = useState(whiteName);
  const [komiDraft, setKomiDraft] = useState(String(komi));
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState(false);
  const ownerRef = useRef<HTMLFormElement>(null);
  useEffect(() => {
    const owner = ownerRef.current;
    if (!owner) return;
    return scheduleOwnedFocus(owner, () => owner.querySelector(`[data-search-target="${focusTarget}"]`), onTargetUnavailable);
  }, [focusTarget, onTargetUnavailable]);

  useEffect(() => {
    function onKey(event: KeyboardEvent) {
      if (event.key === "Escape" && !pending) {
        event.preventDefault();
        onCancel();
      }
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onCancel, pending]);

  async function submit() {
    if (pending) return;
    const parsed = Number(komiDraft);
    if (komiDraft.trim() === "" || !Number.isFinite(parsed) || !Number.isFinite(Math.fround(parsed))) {
      setError(t("gameInfo.invalidKomi"));
      return;
    }
    setError(null);
    setPending(true);
    try {
      await onApply(blackDraft, whiteDraft, parsed);
    } catch (failure) {
      setError(failure instanceof Error ? failure.message : String(failure));
    } finally {
      setPending(false);
    }
  }

  return (
    <div className="shortcut-reference-backdrop" onClick={() => { if (!pending) onCancel(); }}>
      <form ref={ownerRef} className="new-document-dialog" role="dialog" aria-label={t("gameInfo.title")} aria-modal="true"
        onClick={(event) => event.stopPropagation()}
        onSubmit={(event) => { event.preventDefault(); void submit(); }}>
        <div className="new-document-header">
          <h2>{t("gameInfo.title")}</h2>
          <p>{t("gameInfo.description")}</p>
        </div>
        <div className="new-document-fields">
          <label><span>{t("gameInfo.black")}</span><input data-search-target="game.black-name" type="text" value={blackDraft} disabled={pending} onChange={(event) => setBlackDraft(event.target.value)} /></label>
          <label><span>{t("gameInfo.white")}</span><input data-search-target="game.white-name" type="text" value={whiteDraft} disabled={pending} onChange={(event) => setWhiteDraft(event.target.value)} /></label>
          <label><span>{t("gameInfo.komi")}</span><input data-search-target="game.komi" type="number" step="any" value={komiDraft} disabled={pending} onChange={(event) => setKomiDraft(event.target.value)} /></label>
          {handicap !== null ? <label><span>{t("gameInfo.handicap")}</span><input type="text" value={handicap} readOnly /></label> : null}
        </div>
        {error ? <p className="new-document-error" role="alert">{error}</p> : null}
        <div className="document-departure-actions">
          <button className="primary" type="submit" disabled={pending}>{t("gameInfo.apply")}</button>
          <button type="button" disabled={pending} onClick={onCancel}>{t("gameInfo.cancel")}</button>
        </div>
      </form>
    </div>
  );
}
