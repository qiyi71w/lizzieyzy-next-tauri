import { useEffect, useState } from "react";

type Props = {
  blackName: string;
  whiteName: string;
  komi: number;
  handicap: string | null;
  onApply: (blackName: string, whiteName: string, komi: number) => Promise<void>;
  onCancel: () => void;
};

export function GameMetadataDialog({ blackName, whiteName, komi, handicap, onApply, onCancel }: Props) {
  const [blackDraft, setBlackDraft] = useState(blackName);
  const [whiteDraft, setWhiteDraft] = useState(whiteName);
  const [komiDraft, setKomiDraft] = useState(String(komi));
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState(false);

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
      setError("贴目须为有限数值。");
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
      <form className="new-document-dialog" role="dialog" aria-label="编辑棋局信息" aria-modal="true"
        onClick={(event) => event.stopPropagation()}
        onSubmit={(event) => { event.preventDefault(); void submit(); }}>
        <div className="new-document-header">
          <h2>编辑棋局信息</h2>
          <p>修改双方姓名与贴目。棋盘大小请使用新建棋谱。</p>
        </div>
        <div className="new-document-fields">
          <label><span>黑方姓名</span><input autoFocus type="text" value={blackDraft} disabled={pending} onChange={(event) => setBlackDraft(event.target.value)} /></label>
          <label><span>白方姓名</span><input type="text" value={whiteDraft} disabled={pending} onChange={(event) => setWhiteDraft(event.target.value)} /></label>
          <label><span>贴目</span><input type="number" step="any" value={komiDraft} disabled={pending} onChange={(event) => setKomiDraft(event.target.value)} /></label>
          {handicap !== null ? <label><span>让子</span><input type="text" value={handicap} readOnly /></label> : null}
        </div>
        {error ? <p className="new-document-error" role="alert">{error}</p> : null}
        <div className="document-departure-actions">
          <button className="primary" type="submit" disabled={pending}>应用</button>
          <button type="button" disabled={pending} onClick={onCancel}>取消</button>
        </div>
      </form>
    </div>
  );
}
