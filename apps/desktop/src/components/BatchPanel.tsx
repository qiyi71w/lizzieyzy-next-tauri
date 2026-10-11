import { useEffect, useRef, useState } from "react";
import { batchCommand, batchNativeAvailable, batchSnapshot, confirmBatchOverwrite, listenBatchIntake, pickBatchDirectory, pickBatchFiles, pickBatchSave } from "../api/batch";
import { loadBatchPreferences, saveBatchPreferences } from "../api/preferences";
import { defaultBatchPreferences, validateBatchPreferences } from "../domain/batch";
import type { AnalysisStageConditionsDto, BatchCommandDto, BatchRowDto, BatchSnapshotDto } from "../domain/types";
import { t } from "../i18n/resources";
import "./BatchPanel.css";

export function BatchPanel({ openRequest }: { openRequest: number }) {
  const [open, setOpen] = useState(false);
  const [snapshot, setSnapshot] = useState<BatchSnapshotDto | null>(null);
  const [saved, setSaved] = useState(defaultBatchPreferences);
  const [draft, setDraft] = useState(defaultBatchPreferences);
  const [paths, setPaths] = useState<string[]>([]);
  const [mode, setMode] = useState<"" | "session_only" | "directory">("");
  const [directory, setDirectory] = useState<string | null>(null);
  const [confirmedNames, setConfirmedNames] = useState(false);
  const [overwrite, setOverwrite] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const generation = useRef(0);
  const mounted = useRef(true);
  const panel = useRef<HTMLElement>(null);
  const opener = useRef<HTMLButtonElement>(null);
  const native = batchNativeAvailable();

  useEffect(() => { if (openRequest > 0) setOpen(true); }, [openRequest]);
  useEffect(() => { if (open) panel.current?.querySelector<HTMLButtonElement>("button")?.focus(); }, [open]);
  useEffect(() => {
    mounted.current = true;
    let stopped = false;
    let release: (() => void) | undefined;
    void loadBatchPreferences().then(value => { if (!stopped) { setSaved(value); setDraft(value); } }).catch(e => { if (!stopped) setError(String(e)); });
    void listenBatchIntake(value => { setPaths(value); setOpen(true); }).then(fn => { if (stopped) fn(); else release = fn; }).catch(e => { if (!stopped) setError(String(e)); });
    return () => { stopped = true; mounted.current = false; release?.(); };
  }, []);
  useEffect(() => {
    if (!native || busy) return;
    let stopped = false;
    let timer: number;
    const refresh = async () => {
      const sequence = ++generation.current;
      try { const next = await batchSnapshot(); if (!stopped && sequence === generation.current) setSnapshot(next); }
      catch (e) { if (!stopped) setError(String(e)); }
      if (!stopped) timer = window.setTimeout(refresh, 250);
    };
    void refresh();
    return () => { stopped = true; clearTimeout(timer); };
  }, [native, busy]);

  async function command(command: BatchCommandDto) {
    const sequence = ++generation.current;
    setBusy(true); setError(null);
    try { const value = await batchCommand(command); if (mounted.current && sequence === generation.current) setSnapshot(value); return true; }
    catch (e) { if (mounted.current && sequence === generation.current) setError(String(e)); return false; }
    finally { if (mounted.current && sequence === generation.current) setBusy(false); }
  }
  async function addFiles() {
    try { const selected = await pickBatchFiles(); if (selected.length) { setPaths(selected); setOpen(true); } }
    catch (e) { setError(String(e)); }
  }
  async function saveRow(row: BatchRowDto) {
    try {
      const path = await pickBatchSave(row.output_name);
      if (!path) return;
      const allowOverwrite = overwrite && await confirmBatchOverwrite(path);
      setOverwrite(false);
      if (overwrite && !allowOverwrite) return;
      await command({ action: "save", identity: row.identity, path, overwrite: allowOverwrite, incomplete: !row.calculation_complete });
    } catch (e) { setError(String(e)); }
  }
  function reorder(row: BatchRowDto, direction: number) {
    const ids = (snapshot?.rows ?? []).filter(r => r.state === "pending" || r.state === "waiting").map(r => r.identity.row_id);
    const from = ids.indexOf(row.identity.row_id), to = from + direction;
    if (to < 0 || to >= ids.length) return;
    [ids[from], ids[to]] = [ids[to], ids[from]];
    setConfirmedNames(false);
    void command({ action: "reorder", row_ids: ids });
  }
  function close() { setDraft(saved); setOpen(false); opener.current?.focus(); }
  const active = snapshot?.rows.some(r => ["pending", "waiting", "running"].includes(r.state));
  const stageKey = draft.strategy === "single_stage" ? "normal" : draft.strategy === "all_positions_two_stage" ? "deep" : "swingDeep";
  const overviewKey = draft.strategy === "all_positions_two_stage" ? "overview" : "swingOverview";
  const rows = snapshot?.rows ?? [];
  return <>
    <div className="batch-global" aria-label={t("batch.title")}>
      <button ref={opener} type="button" onClick={() => setOpen(true)}>{t("batch.title")}</button>
      {snapshot?.started ? <span role="status">{rows.filter(r => r.calculation_complete).length}/{rows.length} · {snapshot.paused ? t("batch.pause") : rows.find(r => r.state === "running")?.source_path ?? snapshot.reason}</span> : null}
      {active || rows.some(r => r.output_state === "saving") ? <button type="button" onClick={() => void command({ action: "stop_all" })}>{t("batch.stopAll")}</button> : null}
      {!open && error ? <span role="alert">{error}</span> : null}
    </div>
    {open ? <section ref={panel} className="batch-panel" role="dialog" aria-modal="true" aria-label={t("batch.title")} onKeyDown={event => {
      if (event.key === "Escape") { event.stopPropagation(); close(); }
      if (event.key === "Tab") {
        const controls = Array.from(panel.current?.querySelectorAll<HTMLElement>("button:not(:disabled), input:not(:disabled), select:not(:disabled), textarea:not(:disabled), summary") ?? []);
        const first = controls[0], last = controls.at(-1);
        if (event.shiftKey && document.activeElement === first) { event.preventDefault(); last?.focus(); }
        else if (!event.shiftKey && document.activeElement === last) { event.preventDefault(); first?.focus(); }
      }
    }}>
      <header><h2>{t("batch.title")}</h2><button type="button" onClick={close}>{t("batch.hide")}</button></header>
      {!native ? <p role="status">{t("batch.native")}</p> : null}
      <fieldset disabled={busy}>
        <legend>{t("batch.conditions")}</legend>
        <div className="batch-fields">
          <label>{t("batch.strategy")}<select value={draft.strategy} onChange={e => {
            const strategy = e.target.value; if (strategy === "single_stage" || strategy === "all_positions_two_stage" || strategy === "swing_selected_two_stage") setDraft({ ...draft, strategy });
          }}>{["single_stage", "all_positions_two_stage", "swing_selected_two_stage"].map(value => <option key={value} value={value}>{value === "single_stage" ? t("batch.single_stage") : value === "all_positions_two_stage" ? t("batch.all_positions_two_stage") : t("batch.swing_selected_two_stage")}</option>)}</select></label>
          <label>{t("batch.scope")}<select value={draft.scope.mode} onChange={e => setDraft({ ...draft, scope: { ...draft.scope, mode: e.target.value === "all_branches" ? "all_branches" : "first_child_mainline" } })}><option value="first_child_mainline">{t("batch.mainline")}</option><option value="all_branches">{t("batch.branches")}</option></select></label>
          <label>{t("batch.toPlay")}<select value={draft.scope.to_play ?? "both"} onChange={e => setDraft({ ...draft, scope: { ...draft.scope, to_play: e.target.value === "black" ? "black" : e.target.value === "white" ? "white" : null } })}><option value="both">{t("batch.both")}</option><option value="black">{t("batch.black")}</option><option value="white">{t("batch.white")}</option></select></label>
          <label><input type="checkbox" checked={draft.scope.interval !== null} onChange={e => setDraft({ ...draft, scope: { ...draft.scope, interval: e.target.checked ? { start: 0, end: 999 } : null } })} />{t("batch.interval")}</label>
          {draft.scope.interval ? <>{(["start", "end"] as const).map(key => <label key={key}>{t(key === "start" ? "batch.from" : "batch.to")}<input type="number" min={0} value={draft.scope.interval?.[key]} onChange={e => setDraft({ ...draft, scope: { ...draft.scope, interval: { ...draft.scope.interval!, [key]: Number(e.target.value) } } })} /></label>)}</> : null}
        </div>
        {draft.strategy !== "single_stage" ? <Stage label={t("batch.overview")} value={draft[overviewKey]} change={value => setDraft({ ...draft, [overviewKey]: value })} /> : null}
        <Stage label={draft.strategy === "single_stage" ? t("batch.single_stage") : t("batch.deep")} value={draft[stageKey]} change={value => setDraft({ ...draft, [stageKey]: value })} />
        {draft.strategy === "swing_selected_two_stage" ? <div className="batch-fields">
          <label>{t("batch.actor")}<select value={draft.swingCriteria.move_actors} onChange={e => setDraft({ ...draft, swingCriteria: { ...draft.swingCriteria, move_actors: e.target.value === "black" ? "black" : e.target.value === "white" ? "white" : "both" } })}><option value="both">{t("batch.both")}</option><option value="black">{t("batch.black")}</option><option value="white">{t("batch.white")}</option></select></label>
          {(["winrate_change_percentage_points", "score_change_points"] as const).map(key => <label key={key}><input type="checkbox" checked={draft.swingCriteria[key].enabled} onChange={e => setDraft({ ...draft, swingCriteria: { ...draft.swingCriteria, [key]: { ...draft.swingCriteria[key], enabled: e.target.checked } } })} />{t(key === "score_change_points" ? "batch.score" : "batch.winrate")}<input aria-label={`${t("batch.swing_selected_two_stage")} ${t(key === "score_change_points" ? "batch.score" : "batch.winrate")}`} type="number" min={0.01} step="any" value={draft.swingCriteria[key].value} onChange={e => setDraft({ ...draft, swingCriteria: { ...draft.swingCriteria, [key]: { ...draft.swingCriteria[key], value: Number(e.target.value) } } })} /></label>)}
        </div> : null}
        <p>{t("batch.or")}</p>
        <button type="button" onClick={async () => { setBusy(true); setError(null); try { const value = await saveBatchPreferences(draft); setSaved(value); setDraft(value); } catch(e) { setError(String(e)); } finally { setBusy(false); } }}>{t("batch.saveSettings")}</button>
        <button type="button" onClick={() => { setDraft(saved); setError(null); }}>{t("batch.cancelSettings")}</button>
      </fieldset>
      <div className="batch-fields"><button type="button" disabled={!native || busy} onClick={() => void addFiles()}>{t("batch.add")}</button>
        <label>{t("batch.paths")}<textarea value={paths.join("\n")} onChange={e => setPaths(e.target.value.split("\n").filter(Boolean))} /></label>
        <button type="button" disabled={!native || busy || !paths.length} onClick={() => { try { validateBatchPreferences(draft); setConfirmedNames(false); void command({ action: "intake", paths, conditions: draft }).then(ok => { if (ok) setPaths([]); }); } catch(e) { setError(String(e)); } }}>{t("batch.confirmIntake")}</button>
      </div>
      {!snapshot?.started ? <fieldset disabled={busy}>
        <legend>{t("batch.output")}</legend>
        <label><input name="batch-output" type="radio" checked={mode === "session_only"} onChange={() => setMode("session_only")} />{t("batch.sessionOnly")}</label>
        <label><input name="batch-output" type="radio" checked={mode === "directory"} onChange={() => setMode("directory")} />{t("batch.directory")}</label>
        {mode === "directory" ? <><button type="button" disabled={!native} onClick={async () => { try { setDirectory(await pickBatchDirectory()); setConfirmedNames(false); } catch(e) { setError(String(e)); } }}>{t("batch.chooseDirectory")}</button><p>{directory}</p>
          <label><input type="checkbox" checked={confirmedNames} onChange={e => setConfirmedNames(e.target.checked)} />{t("batch.naming")}</label><ul>{rows.map(r => <li key={r.identity.row_id}>{r.output_name}</li>)}</ul></> : null}
        <button type="button" disabled={!native || !rows.length || !mode || mode === "directory" && (!directory || !confirmedNames)} onClick={() => void command({ action: "start", output: mode === "directory" && directory ? { mode: "directory", path: directory, confirmed_names: rows.map(r => r.output_name) } : { mode: "session_only" } })}>{t("batch.start")}</button>
      </fieldset> : null}
      <div className="batch-fields">
        <button type="button" disabled={!native || !snapshot?.started} onClick={() => void command({ action: "stop_current" })}>{t("batch.stopCurrent")}</button>
        <button type="button" disabled={!native} onClick={() => void command({ action: "stop_all" })}>{t("batch.stopAll")}</button>
        <button type="button" disabled={!native || !snapshot?.started || busy} onClick={() => void command({ action: "pause" })}>{t("batch.pause")}</button>
        <button type="button" disabled={!native || !snapshot?.paused || busy} onClick={() => void command({ action: "continue" })}>{t("batch.continue")}</button>
        <button type="button" disabled={!native || busy || active} onClick={() => { setMode(""); setDirectory(null); setConfirmedNames(false); void command({ action: "new_batch" }); }}>{t("batch.new")}</button>
      </div>
      {snapshot?.reason ? <p role="status">{snapshot.reason}</p> : null}
      {error ? <p role="alert">{error}</p> : null}
      {snapshot?.notices.map((notice, index) => <p key={index} role="status">{notice}</p>)}
      <label><input type="checkbox" checked={overwrite} onChange={e => setOverwrite(e.target.checked)} />{t("batch.overwrite")}</label>
      <div className="batch-table"><table><thead><tr><th>{t("batch.source")}</th><th>{t("batch.calculation")}</th><th>{t("batch.result")}</th><th>{t("batch.actions")}</th></tr></thead><tbody>
        {rows.map(row => <tr key={`${row.identity.row_id}:${row.identity.attempt}`}>
          <td>{row.source_path}<small>{row.input_receipt ?? row.document_id}</small><small>{row.identity.row_id} / {row.identity.attempt}</small></td>
          <td>{t(`batch.state.${row.state}`)} · {t(`batch.${row.conditions.strategy}`)}<small>{row.task?.stage} {row.task?.completed.length}/{row.task?.requested.length} · overview {row.task?.overview_completed.length}</small>
            <details><summary>{row.preview?.targets.length ?? 0} {t("batch.requested")} / {row.preview?.supporting_targets.length ?? 0} {t("batch.supporting")}</summary>
              <pre>{JSON.stringify({ requested: row.preview?.targets, supporting: row.preview?.supporting_targets, conditions: row.conditions }, null, 2)}</pre></details></td>
          <td>{row.calculation_complete ? t("batch.complete") : t("batch.incomplete")}<small>{t(`batch.${row.output_state}`)}</small><small>{row.saved_path}</small>{row.reason ? <p role="status">{row.reason}</p> : null}</td>
          <td>{["pending", "waiting"].includes(row.state) ? <><button type="button" disabled={busy} onClick={() => reorder(row, -1)}>{t("batch.up")}</button><button type="button" disabled={busy} onClick={() => reorder(row, 1)}>{t("batch.down")}</button><button type="button" disabled={busy} onClick={() => void command({ action: "remove", identity: row.identity })}>{t("batch.remove")}</button></> : null}
            {row.state === "failed" ? <button type="button" disabled={busy} onClick={() => void command({ action: "rerun", identity: row.identity })}>{t("batch.rerun")}</button> : null}
            {row.has_result && row.state !== "running" ? <button type="button" disabled={row.output_state === "saving" || busy} onClick={() => void saveRow(row)}>{row.calculation_complete ? t("batch.save") : t("batch.partial")}</button> : null}</td>
        </tr>)}
      </tbody></table></div>
    </section> : null}
  </>;
}

function Stage({ label, value, change }: { label: string; value: AnalysisStageConditionsDto; change: (value: AnalysisStageConditionsDto) => void }) {
  return <fieldset className="batch-fields"><legend>{label}</legend>{(["time_seconds", "total_visits", "leading_candidate_visits"] as const).map(key => <label key={key}>
    <input type="checkbox" checked={value[key].enabled} onChange={e => change({ ...value, [key]: { ...value[key], enabled: e.target.checked } })} />
    {t(key === "time_seconds" ? "batch.time" : key === "total_visits" ? "batch.visits" : "batch.leading")}
    <input aria-label={`${label} ${t(key === "time_seconds" ? "batch.time" : key === "total_visits" ? "batch.visits" : "batch.leading")}`} type="number" min={1} max={4294967295} value={value[key].value} onChange={e => change({ ...value, [key]: { ...value[key], value: Number(e.target.value) } })} />
  </label>)}</fieldset>;
}
