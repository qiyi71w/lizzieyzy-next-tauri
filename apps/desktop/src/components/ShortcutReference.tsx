import { useEffect, useMemo, useState } from "react";
import { filterShortcutReference, type ShortcutReferenceEntry } from "../domain/shortcuts";

type Props = {
  entries: ShortcutReferenceEntry[];
  onClose: () => void;
};

export function ShortcutReference({ entries, onClose }: Props) {
  const [query, setQuery] = useState("");
  const visible = useMemo(() => filterShortcutReference(entries, query), [entries, query]);

  useEffect(() => {
    function onKey(event: KeyboardEvent) {
      if (event.key !== "Escape") return;
      event.preventDefault();
      onClose();
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);

  return (
    <div className="shortcut-reference-backdrop" onClick={onClose}>
      <div
        className="shortcut-reference"
        role="dialog"
        aria-label="快捷键参考"
        aria-modal="true"
        onClick={(event) => event.stopPropagation()}
      >
        <header className="shortcut-reference-header">
          <h2>快捷键参考</h2>
          <button type="button" onClick={onClose} aria-label="关闭">关闭</button>
        </header>
        <input
          aria-label="搜索快捷键"
          value={query}
          placeholder="搜索动作或按键"
          autoFocus
          onChange={(event) => setQuery(event.target.value)}
        />
        <div className="shortcut-reference-body">
          <table className="shortcut-reference-table">
            <thead>
              <tr>
                <th>动作</th>
                <th>快捷键</th>
              </tr>
            </thead>
            <tbody>
              {visible.map((entry) => (
                <tr key={entry.id}>
                  <td>{entry.label}</td>
                  <td>{entry.keys}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}
