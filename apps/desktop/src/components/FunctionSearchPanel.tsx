import { useEffect, useRef, useState } from "react";
import { searchFunctions, type FunctionSearchAction } from "../domain/functionSearch";
import { t } from "../i18n/resources";

type Props = { catalog: readonly FunctionSearchAction[]; onCancel: () => void; onExecute: (action: FunctionSearchAction) => void };

export function FunctionSearchPanel({ catalog, onCancel, onExecute }: Props) {
  const [query, setQuery] = useState("");
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const results = searchFunctions(catalog, query);
  const selected = results.findIndex((entry) => entry.id === selectedId);
  const index = selected < 0 ? 0 : selected;
  useEffect(() => { inputRef.current?.focus(); }, []);

  return <div className="shortcut-reference-backdrop" onClick={onCancel}>
    <section className="new-document-dialog function-search" role="dialog" aria-modal="true" aria-label={t("search.title")}
      onClick={(event) => event.stopPropagation()}
      onKeyDown={(event) => {
        if (event.key === "Escape") { event.preventDefault(); event.stopPropagation(); onCancel(); }
        if (event.key === "ArrowDown" || event.key === "ArrowUp") {
          event.preventDefault();
          const next = Math.max(0, Math.min(results.length - 1, index + (event.key === "ArrowDown" ? 1 : -1)));
          setSelectedId(results[next]?.id ?? null);
        }
        if (event.key === "Enter") {
          event.preventDefault();
          const action = results[index];
          if (action && !action.disabledReason) onExecute(action);
        }
      }}>
      <header className="new-document-header"><h2>{t("search.title")}</h2><p>{t("search.hint")}</p></header>
      <input ref={inputRef} aria-label={t("search.title")} placeholder={t("search.placeholder")} value={query}
        aria-controls="function-search-results" aria-activedescendant={results[index] ? `search-${results[index].id}` : undefined}
        onChange={(event) => { setQuery(event.target.value); setSelectedId(null); }} />
      <div id="function-search-results" role="listbox" aria-label={t("search.results")} style={{ maxHeight: "50vh", overflow: "auto" }}>
        {results.map((action, offset) => <button key={action.id} id={`search-${action.id}`} type="button" role="option"
          aria-selected={offset === index} aria-disabled={Boolean(action.disabledReason)}
          className={offset === index ? "primary" : undefined} style={{ display: "block", width: "100%", textAlign: "left", marginTop: 6 }}
          onFocus={() => setSelectedId(action.id)}
          onClick={() => { setSelectedId(action.id); if (!action.disabledReason) onExecute(action); }}>
          {t(action.label)} {action.shortcut ? <kbd>{action.shortcut}</kbd> : null}
          {action.disabledReason ? <small style={{ display: "block" }}>{t(action.disabledReason)}</small> : null}
        </button>)}
        {results.length === 0 ? <p role="status">{t("search.empty")}</p> : null}
      </div>
      <button type="button" onClick={onCancel}>{t("search.close")}</button>
    </section>
  </div>;
}
