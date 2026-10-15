import { useEffect, useRef, useState } from "react";
import { isTauriRuntime } from "../api/backend";
import { applyMeasuredReport, waitForMeasuredWorker, previewMeasuredScene, restoreMeasuredReport, retireMeasuredReport, reviewMeasuredReport, startMeasuredScene } from "../api/measuredReport";
import { measureWithWebViewProbe } from "../api/measurementProbe";
import type { ApplicationMeasurementDocumentDto, AnalysisScopePreviewDto, EngineProfileRecordDto, MeasuredReviewDto, MeasuredSceneDto, NodePath } from "../domain/types";
import { t } from "../i18n/resources";

type Props = { target?: EngineProfileRecordDto; documentGeneration: number | null; documentNode: NodePath; inputRevision: string; disabled: boolean; beginOperation: () => boolean; endOperation: () => void; onProfilesSaved: (profiles: EngineProfileRecordDto[]) => void; onMeasurementStart?: () => (document: ApplicationMeasurementDocumentDto) => void };
export function MeasuredReportPanel({ target, documentGeneration, documentNode, inputRevision, disabled, beginOperation, endOperation, onProfilesSaved, onMeasurementStart }: Props) {
  const [review, setReview] = useState<MeasuredReviewDto | null>(null);
  const [confirmed, setConfirmed] = useState(false);
  const [pending, setPending] = useState(false);
  const [status, setStatus] = useState<string>(t("measured.idle"));
  const [scenePreview, setScenePreview] = useState<AnalysisScopePreviewDto | null>(null);
  const [sceneBudget, setSceneBudget] = useState(100);
  const [measurementScene, setMeasurementScene] = useState<MeasuredSceneDto>("live");
  const [pairs, setPairs] = useState(3);
  const [candidateText, setCandidateText] = useState('{"numSearchThreads":8,"nnMaxBatchSize":8}');
  const [fixture, setFixture] = useState<File | null>(null);
  const [replaceConfirmed, setReplaceConfirmed] = useState(false);
  const [measurementReport, setMeasurementReport] = useState<string | null>(null);
  const [measurementEvidence, setMeasurementEvidence] = useState<string | null>(null);
  const owner = useRef({ generation: 0, active: true, operation: 0, owned: false });
  const callbacks = useRef({ beginOperation, endOperation, onProfilesSaved });
  callbacks.current = { beginOperation, endOperation, onProfilesSaved };
  const targetKey = JSON.stringify([target?.id, target?.profile, target?.thread_policy?.revision, target?.measured_scenes?.revision, inputRevision]);
  useEffect(() => {
    const state = owner.current;
    state.active = true; state.generation++;
    setReview(null); setConfirmed(false); setStatus(t("measured.idle"));
    setMeasurementReport(null); setMeasurementEvidence(null);
    setScenePreview(null);
    return () => {
      state.active = false; state.generation++;
      void retireMeasuredReport().catch(() => { /* A failed retirement never fabricates a completed receipt. */ });
    };
  }, [targetKey]);
  useEffect(() => { setScenePreview(null); }, [documentGeneration, JSON.stringify(documentNode)]);

  async function operate(action: "import" | "apply" | "restore", file?: File) {
    const state = owner.current;
    if (!target || disabled || state.owned || !state.active || !callbacks.current.beginOperation()) return;
    state.owned = true;
    const generation = state.generation;
    const operation = ++state.operation;
    const captured = target;
    const receipt = review;
    setPending(true); setConfirmed(false); setStatus(t("measured.running"));
    const current = () => state.active && state.generation === generation && state.operation === operation;
    try {
      if (action === "import") {
        setReview(null);
        if (!file || file.size > 1024 * 1024) throw new Error(t("measured.tooLarge"));
        const report = await file.text();
        if (!current()) return;
        const result = await reviewMeasuredReport(captured.id, report);
        if (current()) { setReview(result); setStatus(t("measured.review")); }
      } else {
        if (action === "apply" && (!receipt || !confirmed || receipt.target_id !== captured.id)) return;
        const saved = action === "apply" ? await applyMeasuredReport(receipt!) : await restoreMeasuredReport(captured.id, captured.measured_scenes?.revision ?? 0);
        if (current()) {
          callbacks.current.onProfilesSaved(saved.profiles);
          setReview(null); setStatus(t(action === "apply" ? "measured.applied" : "measured.restored"));
        }
      }
    } catch (failure) {
      if (current()) setStatus(failure instanceof Error ? failure.message : String(failure));
    } finally {
      if (state.operation === operation) {
        state.owned = false;
        callbacks.current.endOperation();
        if (state.active) setPending(false);
      }
    }
  }
  async function scene(action: "preview" | "start") {
    const state = owner.current;
    if (!target || documentGeneration === null || disabled || state.owned || !state.active || !callbacks.current.beginOperation()) return;
    state.owned = true;
    const generation = state.generation;
    const operation = ++state.operation;
    const current = () => state.active && state.generation === generation && state.operation === operation;
    setPending(true);
    try {
      if (action === "preview") {
        const preview = await previewMeasuredScene(documentGeneration, { mode: "first_child_mainline", current_node: documentNode, branch_choices: [] });
        if (current()) setScenePreview(preview);
      } else {
        if (!scenePreview || !Number.isInteger(sceneBudget) || sceneBudget < 1 || sceneBudget > 2147483647) return;
        const task = await startMeasuredScene(target.id, scenePreview, sceneBudget);
        if (current()) setStatus(`${task.task_id} · ${task.state}`);
        await waitForMeasuredWorker();
      }
    } catch (failure) {
      if (current()) setStatus(String(failure));
    } finally {
      if (state.operation === operation) {
        state.owned = false; callbacks.current.endOperation();
        if (state.active) setPending(false);
      }
    }
  }
  async function measure() {
    const state = owner.current;
    if (!target || !fixture || !replaceConfirmed || disabled || state.owned || !state.active || !callbacks.current.beginOperation()) return;
    state.owned = true;
    const generation = state.generation;
    const operation = ++state.operation;
    const captured = target;
    const current = () => state.active && state.generation === generation && state.operation === operation;
    setPending(true); setReview(null); setMeasurementReport(null); setMeasurementEvidence(null); setStatus(t("measured.running"));
    try {
      if (fixture.size > 1024 * 1024) throw new Error("Fixture exceeds 1 MiB");
      const candidate: unknown = JSON.parse(candidateText);
      if (!candidate || typeof candidate !== "object" || Array.isArray(candidate) || Object.values(candidate).some((value) => typeof value !== "number" || !Number.isInteger(value) || value < 1)) throw new Error("Candidate parameters must be positive integer values");
      const parameters: Record<string, number> = {};
      for (const [key, value] of Object.entries(candidate)) { if (typeof value === "number") parameters[key] = value; }
      const text = await fixture.text();
      if (!current()) return;
      if (!onMeasurementStart) throw new Error("Authoritative measurement document consumer is unavailable");
      const onDocument = onMeasurementStart();
      const report = await measureWithWebViewProbe({ target_id: captured.id, scene: measurementScene, candidate_parameters: parameters, pairs, budget: sceneBudget, fixture_sgf: text, replace_current_document: true }, onDocument);
      if (!current()) return;
      const serialized = JSON.stringify(report.report, null, 2);
      setMeasurementReport(serialized);
      setMeasurementEvidence(JSON.stringify(report.evidence, null, 2));
      const result = await reviewMeasuredReport(captured.id, serialized);
      if (current()) { setReview(result); setStatus(t("measured.review")); }
    } catch (failure) {
      if (current()) setStatus(String(failure));
    } finally {
      if (state.operation === operation) { state.owned = false; callbacks.current.endOperation(); if (state.active) setPending(false); }
    }
  }
  async function cancel() {
    const state = owner.current;
    state.generation++;
    setReview(null); setConfirmed(false); setStatus(t("measured.retiring"));
    const generation = state.generation;
    try {
      await retireMeasuredReport();
      await waitForMeasuredWorker();
      if (state.active && generation === state.generation) setStatus(t("measured.idle"));
    } catch (failure) {
      if (state.active && generation === state.generation) setStatus(String(failure));
    }
  }
  const eligible = !!review?.assessment.eligible && review.qualification_reasons.length === 0;
  const native = isTauriRuntime();
  return <section aria-label={t("measured.title")} className="measured-report-panel">
    <h3>{t("measured.title")}</h3>
    <p>{target ? `${target.profile.name} · ${target.id}` : t("measured.idle")}</p>
    <p>{t("measured.experimental")}</p>
    {!native && <p>{t("measured.native")}</p>}
    <label>{t("measured.import")}<input type="file" accept="application/json,.json" disabled={!native || !target || disabled || pending} onChange={(event) => { const file = event.currentTarget.files?.[0]; event.currentTarget.value = ""; if (file) void operate("import", file); }} /></label>
    <div className="engine-actions">
      <button type="button" aria-disabled={!native || disabled || pending || !target} onClick={() => { if (native && !disabled && !pending && target) void operate("restore"); }}>{t("measured.restore")}</button>
      <button type="button" aria-disabled={!native} onClick={() => { if (native) void cancel(); }}>{t("measured.cancel")}</button>
    </div>
    <fieldset disabled={!native || !target || disabled || pending || documentGeneration === null}>
      <legend>{t("measured.independent")}</legend>
      <p>{t("measured.independentHint")}</p>
      <p>{t("measured.wholeClear")}</p>
      <label>{t("measured.budget")} <input type="number" min={1} max={2147483647} value={sceneBudget} onChange={(event) => setSceneBudget(Number(event.currentTarget.value))} /></label>
      <button type="button" onClick={() => void scene("preview")}>{t("measured.preview")}</button>
      {scenePreview && <><p>{scenePreview.targets.length} positions · document {scenePreview.generation}</p><button type="button" onClick={() => void scene("start")}>{t("measured.startScene")}</button></>}
    </fieldset>
    <fieldset disabled={!native || !target || disabled || pending}>
      <legend>{t("measured.measurement")}</legend>
      <p>{t("measured.measurementHint")}</p>
      <label>{t("measured.scene")} <select value={measurementScene} onChange={(event) => { const scene = event.currentTarget.value === "live" ? "live" : "whole-game"; setMeasurementScene(scene); setCandidateText(scene === "live" ? '{"numSearchThreads":8,"nnMaxBatchSize":8}' : '{"numAnalysisThreads":2,"numSearchThreadsPerAnalysisThread":4,"nnMaxBatchSize":8}'); }}><option value="live">{t("measured.live")}</option><option value="whole-game">{t("measured.whole")}</option></select></label>
      <label>{t("measured.pairs")} <select value={pairs} onChange={(event) => setPairs(Number(event.currentTarget.value))}><option value={3}>{t("measured.threePairs")}</option><option value={5}>{t("measured.fivePairs")}</option></select></label>
      <label>{t("measured.budget")} <input type="number" min={1} max={2147483647} value={sceneBudget} onChange={(event) => setSceneBudget(Number(event.currentTarget.value))} /></label>
      <label>{t("measured.candidateMap")} <textarea value={candidateText} onChange={(event) => setCandidateText(event.currentTarget.value)} /></label>
      <label>{t("measured.fixture")} <input type="file" accept=".sgf" onChange={(event) => setFixture(event.currentTarget.files?.[0] ?? null)} /></label>
      <label><input type="checkbox" checked={replaceConfirmed} onChange={(event) => setReplaceConfirmed(event.currentTarget.checked)} />{t("measured.replaceConsent")}</label>
      <button type="button" disabled={!fixture || !replaceConfirmed} onClick={() => void measure()}>{t("measured.measure")}</button>
    </fieldset>
    {measurementReport && <details><summary>{t("measured.generated")}</summary><textarea readOnly aria-label={t("measured.generated")} value={measurementReport} rows={12} style={{ width: "100%" }} /></details>}
    {measurementEvidence && <details><summary>{t("measured.evidence")}</summary><textarea readOnly aria-label={t("measured.evidence")} value={measurementEvidence} rows={12} style={{ width: "100%" }} /></details>}
    <p role="status" aria-live="polite" aria-atomic="true">{status}</p>
    {review && <div>
      <h4>{t("measured.review")}</h4>
      <p>{review.target_name} · {review.target_id} · {review.scene}</p>
      <div style={{ overflowX: "auto", maxWidth: "100%" }} tabIndex={0} role="region" aria-label={t("measured.parameters")}>
        <table><caption>{t("measured.parameters")}</caption><thead><tr><th>{t("measured.parameter")}</th><th>{t("measured.baseline")}</th><th>{t("measured.candidate")}</th></tr></thead><tbody>{Object.entries(review.baseline_parameters).map(([key, value]) => <tr key={key}><th scope="row">{key}</th><td>{value}</td><td>{review.candidate_parameters[key]}</td></tr>)}</tbody></table>
      </div>
      {review.scene === "whole-game" && <p>{t("measured.wholeClear")}</p>}
      <h4>{t("measured.quality")}</h4>
      <p>Paired ratios: {review.assessment.pair_ratios.join(", ")} · median: {review.assessment.median_ratio ?? "unknown"} · sample CV: {review.assessment.sample_cv ?? "unknown"}</p>
      <p>coldVerificationSeconds: {review.cold_verification_seconds}</p>
      <ul>{[...review.assessment.reasons, ...review.qualification_reasons].map((reason, index) => <li key={index}>{reason}</li>)}</ul>
      <details><summary>{t("measured.raw")}</summary><pre style={{ whiteSpace: "pre-wrap", overflowWrap: "anywhere", maxHeight: "24rem", overflowY: "auto" }} tabIndex={0}>{JSON.stringify(review.report, null, 2)}</pre></details>
      <label><input type="checkbox" checked={confirmed} disabled={!eligible || pending || disabled} onChange={(event) => setConfirmed(event.target.checked)} />{t("measured.confirm")}</label>
      <button type="button" aria-disabled={!eligible || !confirmed || pending || disabled} onClick={() => { if (eligible && confirmed && !pending && !disabled) void operate("apply"); }}>{t("measured.apply")}</button>
    </div>}
  </section>;
}
