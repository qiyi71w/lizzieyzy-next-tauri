import { WindowPinControl } from "./WindowPinControl";
import type { MainWindowPinControl } from "../hooks/useMainWindowPin";
import { useEffect, useState } from "react";
import { continuousBudgetError, newGameDefaultsError, type AppPreferences, type BoardTheme, type GraphPerspective, type NextMoveReviewMarkerMode, type ReviewMode, type SubBoardContentMode } from "../domain/preferences";
import { parsePositiveScoreLeadScale } from "../domain/winrateChart";
import { parseReviewAutoplaySeconds } from "../domain/reviewAutoplay";
import { t } from "../i18n/resources";
import { validGranularDisplay, type GranularDisplay, type CandidateTextField, type OwnershipStyle } from "../domain/granularDisplay";
import { defaultMoveRankPreferences, readMoveRankPreferences, validMoveRankPreferences } from "../domain/moveRank";
import type { MoveRankPreferencesDto } from "../domain/types";
import { validLocalFocusPreferences } from "../domain/localFocus";

type Props = {
  windowPin?: MainWindowPinControl;
  preferences: AppPreferences;
  status: string;
  disabled?: boolean;
  scoreLeadAvailable?: boolean;
  onChange: (preferences: AppPreferences) => void;
  onRestoreWorkspace?: () => void;
  workspaceDisabled?: boolean;
  windowGeometryStatus: string;
  windowGeometryDisabled: boolean;
  onResetWindowGeometry: () => void;
  railVisibilityDisabled?: boolean;
  onRailVisibility: (side: "left" | "right", visible: boolean) => void;
};

export function PreferencesPanel({ preferences, status, disabled = false, scoreLeadAvailable = true, onChange, onRestoreWorkspace, workspaceDisabled, windowGeometryStatus, windowGeometryDisabled, onResetWindowGeometry, railVisibilityDisabled = false, onRailVisibility, windowPin }: Props) {
  function update(patch: Partial<AppPreferences>) {
    onChange({ ...preferences, ...patch });
  }

  return (
    <section className="preferences-panel" aria-label="设置" data-focus-owner="prefs" tabIndex={-1}>
      <div className="preferences-header">
        <h2>设置</h2>
        <span>{status}</span>
      </div>
      <AutoLoadEditor preferences={preferences} disabled={disabled} onChange={onChange} />
      <NodeAnalysisReuseEditor preferences={preferences} disabled={disabled} onChange={onChange} />
      <MoveRankEditor preferences={preferences} disabled={disabled} onChange={onChange} />
      <LocalFocusEditor preferences={preferences} disabled={disabled} onChange={onChange} />
      <FlashVisitsEditor preferences={preferences} disabled={disabled} onChange={onChange} />
      {windowPin ? <fieldset className="preferences-grid">
        <legend>窗口</legend>
        <WindowPinControl control={windowPin} />
      </fieldset> : null}
      <fieldset className="preferences-grid">
        <legend>工作区</legend>
        <button type="button" disabled={workspaceDisabled} onClick={onRestoreWorkspace}>恢复面板尺寸</button>
      </fieldset>
      <fieldset className="preferences-grid">
        <legend>窗口</legend>
        <span role="status">{windowGeometryStatus}</span>
        <button type="button" disabled={windowGeometryDisabled} onClick={onResetWindowGeometry}>恢复窗口位置和大小</button>
      </fieldset>
      <fieldset className="preferences-grid">
        <legend>侧栏</legend>
        <Toggle label="左侧栏" checked={preferences.workspaceVisibility.left} disabled={railVisibilityDisabled} onChange={(visible) => onRailVisibility("left", visible)} />
        <Toggle label="右侧栏" checked={preferences.workspaceVisibility.right} disabled={railVisibilityDisabled} onChange={(visible) => onRailVisibility("right", visible)} />
      </fieldset>
      <fieldset className="preferences-grid">
        <legend>分析呈现</legend>
        <Toggle label="候选" checked={preferences.showCandidates} disabled={disabled} onChange={(checked) => update({ showCandidates: checked })} />
        <Toggle label="领地" checked={preferences.showOwnership} disabled={disabled} onChange={(checked) => update({ showOwnership: checked })} />
        <Toggle label="策略" checked={preferences.showPolicy} disabled={disabled} onChange={(checked) => update({ showPolicy: checked })} />
        <label>
          <span>下一手标记</span>
          <select
            value={preferences.nextMoveReviewMarker}
            disabled={disabled}
            onChange={(event) => update({ nextMoveReviewMarker: event.target.value as NextMoveReviewMarkerMode })}
          >
            <option value="off">关闭</option>
            <option value="variations">变化</option>
            <option value="graded">分级</option>
          </select>
        </label>
        <label>
          <span>显示候选数</span>
          <input
            data-search-target="prefs.candidate-limit"
            type="number"
            min={1}
            max={20}
            step={1}
            value={preferences.candidateLimit}
            disabled={disabled}
            onChange={(event) => update({ candidateLimit: Number(event.target.value) })}
          />
        </label>
        <label>
          <span>小棋盘内容</span>
          <select
            value={preferences.subBoardContentMode}
            disabled={disabled}
            onChange={(event) => update({ subBoardContentMode: event.target.value as SubBoardContentMode })}
          >
            <option value="variation">变化图</option>
            <option value="raw">纯棋子</option>
          </select>
        </label>
        <Toggle
          label="变化回放"
          checked={preferences.variationReplayEnabled}
          disabled={disabled}
          onChange={(checked) => update({ variationReplayEnabled: checked })}
        />
        <label>
          <span>回放间隔</span>
          <input
            type="number"
            data-search-target="prefs.replay-interval"
            min={100}
            max={5000}
            step={1}
            value={preferences.variationReplayIntervalMs}
            disabled={disabled}
            onChange={(event) => update({ variationReplayIntervalMs: Number(event.target.value) })}
          />
        </label>
      </fieldset>
      <fieldset className="preferences-grid">
        <legend>启动</legend>
        <Toggle
          label="启动时恢复上次棋谱"
          checked={preferences.restoreLastSession}
          disabled={disabled}
          onChange={(checked) => update({ restoreLastSession: checked })}
        />
        <Toggle
          label="连续分析"
          checked={preferences.continuousAnalysisEnabled}
          disabled={disabled}
          onChange={(checked) => update({ continuousAnalysisEnabled: checked })}
        />
      </fieldset>
      <ContinuousBudgetEditor preferences={preferences} disabled={disabled} onChange={onChange} />
      <fieldset className="preferences-grid">
        <legend>复盘</legend>
        <Toggle
          label="落子声音"
          checked={preferences.soundEnabled}
          disabled={disabled}
          onChange={(checked) => update({ soundEnabled: checked })}
        />
        <ReviewAutoplayEditor preferences={preferences} disabled={disabled} onChange={onChange} />
        <label>
          <span>默认计算量</span>
          <input
            type="number"
            min={1}
            step={1}
            value={preferences.defaultMaxVisits}
            disabled={disabled}
            onChange={(event) => update({ defaultMaxVisits: Number(event.target.value) })}
          />
        </label>
        <label>
          <span>复盘深度</span>
          <select value={preferences.reviewMode} disabled={disabled} onChange={(event) => update({ reviewMode: event.target.value as ReviewMode })}>
            <option value="quick">快复</option>
            <option value="deep">深复</option>
          </select>
        </label>
      </fieldset>
      <fieldset className="preferences-grid">
        <legend>棋盘</legend>
        <label>
          <span>棋盘对比</span>
          <select data-search-target="prefs.board-theme" value={preferences.boardTheme} disabled={disabled} onChange={(event) => update({ boardTheme: event.target.value as BoardTheme })}>
            <option value="classic">浅色</option>
            <option value="high-contrast">高对比</option>
          </select>
        </label>
      </fieldset>
      <DisplayPreferencesEditor preferences={preferences} disabled={disabled} onChange={onChange} />
      <NewGameDefaultsEditor preferences={preferences} disabled={disabled} onChange={onChange} />
      <fieldset className="preferences-grid">
        <legend>胜率图</legend>
        <label>
          <span>图表视角</span>
          <select
            value={preferences.graphPerspective}
            disabled={disabled}
            onChange={(event) => update({ graphPerspective: event.target.value as GraphPerspective })}
          >
            <option value="black">黑棋</option>
            <option value="sideToPlay">当前行棋方</option>
          </select>
        </label>
        <Toggle
          label="胜率线"
          checked={preferences.winrateLine}
          disabled={disabled}
          onChange={(checked) => update({ winrateLine: checked })}
        />
        <Toggle
          label="目差线"
          checked={preferences.scoreLeadLine}
          disabled={disabled || !scoreLeadAvailable}
          onChange={(checked) => update({ scoreLeadLine: checked })}
        />
        <Toggle label="失误条" checked={preferences.blunderBar} disabled={disabled} onChange={(checked) => update({ blunderBar: checked })} />
        <Toggle label="图表悬停" checked={preferences.graphHover} disabled={disabled} onChange={(checked) => update({ graphHover: checked })} />
        <label>
          <span>目差刻度</span>
          <input
            type="number"
            min={1}
            max={1000}
            step={1}
            value={preferences.scoreLeadScale}
            disabled={disabled || !scoreLeadAvailable}
            onChange={(event) => {
              const parsed = parsePositiveScoreLeadScale(event.target.value);
              if (parsed == null) {
                event.currentTarget.value = String(preferences.scoreLeadScale);
                return;
              }
              update({ scoreLeadScale: parsed });
            }}
          />
        </label>
      </fieldset>
    </section>
  );
}

function FlashVisitsEditor({ preferences, disabled, onChange }: Pick<Props, "preferences" | "disabled" | "onChange">) {
  const [edited, setEdited] = useState<string | null>(null);
  const value = edited ?? String(preferences.flashAnalysisVisits);
  const visits = Number(value);
  const valid = value.trim() !== "" && Number.isInteger(visits) && visits > 0 && visits <= 4294967295;
  useEffect(() => { setEdited(null); }, [preferences.flashAnalysisVisits]);
  return <fieldset className="preferences-grid" disabled={disabled}>
    <legend>显式闪电分析</legend>
    <label>显式闪电 visits<input aria-label="Explicit flash visits" type="number" min={1} max={4294967295} step={1} value={value} onChange={(event) => setEdited(event.target.value)} /></label>
    <span>仅影响下次显式闪电请求；保存不会开始或重试分析。</span>
    {!valid ? <span role="alert">Visits 必须是 1–4294967295 的整数。</span> : null}
    <button type="button" disabled={!valid || edited === null} onClick={() => onChange({ ...preferences, flashAnalysisVisits: visits })}>保存闪电设置</button>
    <button type="button" onClick={() => setEdited(null)}>取消闪电设置</button>
  </fieldset>;
}

function AutoLoadEditor({ preferences, disabled, onChange }: Pick<Props, "preferences" | "disabled" | "onChange">) {
  const [edited, setEdited] = useState<{ enabled: boolean; visits: string } | null>(null);
  const [invalid, setInvalid] = useState(false);
  useEffect(() => { setEdited(null); setInvalid(false); }, [preferences.autoLoad.enabled, preferences.autoLoad.totalVisits]);
  const draft = edited ?? { enabled: preferences.autoLoad.enabled, visits: String(preferences.autoLoad.totalVisits) };
  return <fieldset className="preferences-grid" disabled={disabled}>
    <legend>{t("autoLoad.title")}</legend>
    <Toggle label={t("autoLoad.enabled")} checked={draft.enabled} disabled={disabled}
      onChange={(enabled) => setEdited({ ...draft, enabled })} />
    <label><span>{t("autoLoad.visits")}</span>
      <input type="text" inputMode="numeric" value={draft.visits}
        onChange={(event) => { setEdited({ ...draft, visits: event.target.value }); setInvalid(false); }} />
    </label>
    <button type="button" disabled={disabled || edited === null} onClick={() => {
      const totalVisits = Number(draft.visits);
      if (!/^\d+$/.test(draft.visits) || !Number.isInteger(totalVisits) || totalVisits < 1 || totalVisits > 4294967295) {
        setInvalid(true); return;
      }
      onChange({ ...preferences, autoLoad: { enabled: draft.enabled, totalVisits } });
    }}>{t("autoLoad.save")}</button>
    <button type="button" disabled={disabled || edited === null} onClick={() => { setEdited(null); setInvalid(false); }}>{t("autoLoad.cancelDraft")}</button>
    <p>{t("autoLoad.boundary")}</p>
    {invalid ? <p role="alert">{t("autoLoad.invalid")}</p> : null}
  </fieldset>;
}

function ReviewAutoplayEditor({ preferences, disabled, onChange }: Pick<Props, "preferences" | "disabled" | "onChange">) {
  const [edited, setEdited] = useState<string | null>(null);
  const [error, setError] = useState(false);
  const seconds = edited ?? String(preferences.reviewAutoplayIntervalMs / 1000);
  return <div>
    <label>
      <span>{t("review.autoplay.interval")}</span>
      <input data-search-target="prefs.autoplay-interval" aria-label={t("review.autoplay.interval")}
        type="text" inputMode="decimal" value={seconds} disabled={disabled}
        onChange={(event) => { setEdited(event.target.value); setError(false); }} />
    </label>
    <small>{t("review.autoplay.hint")}</small>
    {error ? <p role="alert">{t("review.autoplay.invalid")}</p> : null}
    <button type="button" disabled={disabled || edited === null} onClick={() => {
      const milliseconds = parseReviewAutoplaySeconds(seconds);
      if (milliseconds === null) { setError(true); return; }
      onChange({ ...preferences, reviewAutoplayIntervalMs: milliseconds });
      setEdited(null);
      setError(false);
    }}>{t("review.autoplay.apply")}</button>
    <button type="button" disabled={disabled || edited === null} onClick={() => {
      setEdited(null); setError(false);
    }}>{t("review.autoplay.cancel")}</button>
  </div>;
}

function NodeAnalysisReuseEditor({ preferences, disabled, onChange }: Pick<Props, "preferences" | "disabled" | "onChange">) {
  const [edited, setEdited] = useState<boolean | null>(null);
  useEffect(() => {
    if (edited === preferences.retainNodeAnalysis) setEdited(null);
  }, [preferences.retainNodeAnalysis]);
  return <fieldset className="preferences-grid" disabled={disabled}>
    <legend>{t("reuse.title")}</legend>
    <Toggle label={t("reuse.enabled")} checked={edited ?? preferences.retainNodeAnalysis} disabled={disabled} onChange={setEdited} />
    <p>{t("reuse.help")}</p>
    <button type="button" disabled={disabled || edited === null} onClick={() => onChange({ ...preferences, retainNodeAnalysis: edited ?? preferences.retainNodeAnalysis })}>{t("reuse.apply")}</button>
    <button type="button" disabled={disabled || edited === null} onClick={() => setEdited(null)}>{t("reuse.cancel")}</button>
  </fieldset>;
}

function MoveRankEditor({ preferences, disabled, onChange }: Pick<Props, "preferences" | "disabled" | "onChange">) {
  const [edited, setEdited] = useState<MoveRankPreferencesDto | null>(null);
  const [error, setError] = useState(false);
  const draft = edited ?? preferences.moveRank;
  useEffect(() => {
    if (edited && JSON.stringify(edited) === JSON.stringify(preferences.moveRank)) setEdited(null);
  }, [preferences.moveRank]);
  return <fieldset className="preferences-grid move-rank-preferences" disabled={disabled}>
    <legend>{t("rank.title")}</legend>
    <p>{t("rank.help")}</p>
    <label>{t("rank.mode")}<select aria-label={t("rank.mode")} value={draft.mode} onChange={event => {
      setEdited({ ...draft, mode: event.target.value as MoveRankPreferencesDto["mode"] }); setError(false);
    }}>{(["auto", "score", "winrate", "combined"] as const).map(mode => <option key={mode} value={mode}>{t(`rank.${mode}`)}</option>)}</select></label>
    {(["winrateThresholdsPp", "scoreThresholds"] as const).map(key => <fieldset key={key}>
      <legend>{t(`rank.${key}`)}</legend>
      {([1, 2, 3, 4, 5] as const).map((level, index) => <label key={level}>{t(`rank.level${level}`)}<input
        aria-label={`${t(`rank.${key}`)} ${index + 1}`} type="number" step="any"
        value={Number.isFinite(draft[key][index]) ? draft[key][index] : ""} onChange={event => {
          const values = [...draft[key]] as MoveRankPreferencesDto[typeof key];
          values[index] = event.target.value === "" ? NaN : Number(event.target.value);
          setEdited({ ...draft, [key]: values }); setError(false);
        }} /></label>)}
    </fieldset>)}
    {error ? <p role="alert">{t("rank.invalid")}</p> : null}
    <button type="button" disabled={disabled || edited === null} onClick={() => {
      if (!validMoveRankPreferences(draft)) { setError(true); return; }
      onChange({ ...preferences, moveRank: draft });
    }}>{t("rank.apply")}</button>
    <button type="button" disabled={disabled || edited === null} onClick={() => { setEdited(null); setError(false); }}>{t("rank.cancel")}</button>
    <button type="button" onClick={() => { setEdited(readMoveRankPreferences(defaultMoveRankPreferences)); setError(false); }}>{t("rank.restore")}</button>
  </fieldset>;
}

function LocalFocusEditor({ preferences, disabled, onChange }: Pick<Props, "preferences" | "disabled" | "onChange">) {
  const [edited, setEdited] = useState<AppPreferences["localFocus"] | null>(null);
  const [error, setError] = useState(false);
  const draft = edited ?? preferences.localFocus;
  useEffect(() => {
    if (edited && JSON.stringify(edited) === JSON.stringify(preferences.localFocus)) setEdited(null);
  }, [preferences.localFocus]);
  return <fieldset className="preferences-grid" disabled={disabled}>
    <legend>{t("focus.title")}</legend><p>{t("focus.help")}</p>
    <label>{t("focus.visits")}<input type="number" min={1} step={1} value={Number.isFinite(draft.maxVisits) ? draft.maxVisits : ""} onChange={event => { setEdited({ ...draft, maxVisits: event.target.value === "" ? NaN : Number(event.target.value) }); setError(false); }} /></label>
    <Toggle label={t("focus.outline")} checked={draft.showOutline} onChange={showOutline => setEdited({ ...draft, showOutline })} />
    <label>{t("focus.opacity")}<input type="number" min={0} max={100} step={1} value={Number.isFinite(draft.outlineOpacity) ? draft.outlineOpacity : ""} onChange={event => { setEdited({ ...draft, outlineOpacity: event.target.value === "" ? NaN : Number(event.target.value) }); setError(false); }} /></label>
    {error ? <p role="alert">{t("focus.invalid")}</p> : null}
    <button type="button" disabled={disabled || edited === null} onClick={() => { if (!validLocalFocusPreferences(draft)) { setError(true); return; } onChange({ ...preferences, localFocus: draft }); }}>{t("focus.apply")}</button>
    <button type="button" disabled={disabled || edited === null} onClick={() => { setEdited(null); setError(false); }}>{t("focus.cancel")}</button>
  </fieldset>;
}

function DisplayPreferencesEditor({ preferences, disabled, onChange }: Pick<Props, "preferences" | "disabled" | "onChange">) {
  type DisplayDraft = Pick<AppPreferences, "showCoordinates" | "showMoveNumbers" | "granularDisplay">;
  const [edited, setEdited] = useState<DisplayDraft | null>(null);
  const [error, setError] = useState(false);
  const draft = edited ?? { showCoordinates: preferences.showCoordinates, showMoveNumbers: preferences.showMoveNumbers, granularDisplay: preferences.granularDisplay };
  const display = draft.granularDisplay;
  useEffect(() => {
    if (edited && edited.showCoordinates === preferences.showCoordinates && edited.showMoveNumbers === preferences.showMoveNumbers
      && JSON.stringify(edited.granularDisplay) === JSON.stringify(preferences.granularDisplay)) setEdited(null);
  }, [preferences]);
  function update(patch: Partial<GranularDisplay>) {
    setEdited({ ...draft, granularDisplay: { ...display, ...patch } }); setError(false);
  }
  const booleans = ["showBlackCandidates", "showWhiteCandidates", "showWinrate", "showVisits", "showScore", "showOrder", "emphasizeMax", "useCandidateOwnership"] as const;
  return <fieldset className="preferences-grid granular-display-preferences" disabled={disabled}>
    <legend>{t("display.title")}</legend>
    <Toggle label="坐标" checked={draft.showCoordinates} disabled={disabled} onChange={(checked) => setEdited({ ...draft, showCoordinates: checked })} />
    <Toggle label="全部手数" checked={draft.showMoveNumbers} disabled={disabled} onChange={(checked) => setEdited({ ...draft, showMoveNumbers: checked })} />
    <label>{t("display.delayMode")}<select aria-label={t("display.delayMode")} value={display.delayMode} onChange={event => update({ delayMode: event.target.value as GranularDisplay["delayMode"] })}>
      {(["off", "timed", "manual"] as const).map(mode => <option key={mode} value={mode}>{t(`display.${mode}`)}</option>)}
    </select></label>
    <label>{t("display.delaySeconds")}<input aria-label={t("display.delaySeconds")} type="number" step="0.001" value={Number.isNaN(display.delaySeconds) ? "" : display.delaySeconds} onChange={event => update({ delaySeconds: event.target.value === "" ? NaN : Number(event.target.value) })} /></label>
    {booleans.map(key => <Toggle key={key} label={t(`display.${key}`)} checked={display[key]} disabled={disabled} onChange={checked => update({ [key]: checked })} />)}
    <label>{t("display.pvVisitsMode")}<select aria-label={t("display.pvVisitsMode")} value={display.pvVisitsMode} onChange={event => update({ pvVisitsMode: event.target.value as GranularDisplay["pvVisitsMode"] })}>
      <option value="off">{t("display.pvOff")}</option><option value="last">{t("display.last")}</option><option value="all">{t("display.all")}</option>
    </select></label>
    {(["pvVisitsThreshold", "pvLength"] as const).map(key => <label key={key}>{t(`display.${key}`)}<input aria-label={t(`display.${key}`)} type="number" step="1" value={Number.isNaN(display[key]) ? "" : display[key]} onChange={event => update({ [key]: event.target.value === "" ? NaN : Number(event.target.value) })} /></label>)}
    <label>{t("display.scoreMode")}<select aria-label={t("display.scoreMode")} value={display.scoreMode} onChange={event => update({ scoreMode: event.target.value as GranularDisplay["scoreMode"] })}>
      <option value="lead">{t("display.lead")}</option><option value="board">{t("display.board")}</option>
    </select></label>
    {display.textOrder.map((field, index) => <label key={index}>{t("display.textOrder")} {index + 1}<select aria-label={`${t("display.textOrder")} ${index + 1}`} value={field} onChange={event => {
      const next = [...display.textOrder] as GranularDisplay["textOrder"];
      const target = event.target.value as CandidateTextField;
      const old = next.indexOf(target); next[old] = field; next[index] = target; update({ textOrder: next });
    }}>{(["winrate", "visits", "score"] as const).map(value => <option key={value} value={value}>{t(`display.${value}`)}</option>)}</select></label>)}
    <label>{t("display.ownershipStyle")}<select aria-label={t("display.ownershipStyle")} value={display.ownershipStyle} onChange={event => update({ ownershipStyle: event.target.value as OwnershipStyle })}>
      {(["small", "normal", "notOnLive", "bigBelow", "bySize"] as const).map(value => <option key={value} value={value}>{t(`display.${value}`)}</option>)}
    </select></label>
    <p role="status">{t("display.rawUnavailable")}</p>
    {error ? <p role="alert">{t("display.invalid")}</p> : null}
    <button type="button" aria-label="应用显示设置" disabled={disabled || edited === null} onClick={() => {
      if (!validGranularDisplay(display)) { setError(true); return; }
      onChange({ ...preferences, ...draft });
    }}>{t("display.apply")}</button>
    <button type="button" aria-label="取消显示设置" disabled={disabled || edited === null} onClick={() => { setEdited(null); setError(false); }}>{t("display.cancel")}</button>
  </fieldset>;
}

function NewGameDefaultsEditor({ preferences, disabled, onChange }: Pick<Props, "preferences" | "disabled" | "onChange">) {
  const [edited, setEdited] = useState<{ width: string; height: string; komi: string } | null>(null);
  const [error, setError] = useState<string | null>(null);
  const draft = edited ?? {
    width: String(preferences.defaultBoardWidth),
    height: String(preferences.defaultBoardHeight),
    komi: String(preferences.defaultKomi)
  };
  const emptyKomiError = draft.komi.trim() === "" ? "贴目须为有限数值。" : null;
  function apply() {
    const next = {
      ...preferences,
      defaultBoardWidth: Number(draft.width),
      defaultBoardHeight: Number(draft.height),
      defaultKomi: Number(draft.komi)
    };
    const invalid = emptyKomiError ?? newGameDefaultsError(next);
    setError(invalid);
    if (invalid) return;
    onChange(next);
    setEdited(null);
  }
  return (
    <fieldset className="preferences-grid" disabled={disabled}>
      <legend>新建棋谱默认值</legend>
      <label>
        <span>默认宽度</span>
        <input type="number" min={2} max={25} step={1} value={draft.width}
          onChange={(event) => setEdited({ ...draft, width: event.target.value })} />
      </label>
      <label>
        <span>默认高度</span>
        <input type="number" min={2} max={25} step={1} value={draft.height}
          onChange={(event) => setEdited({ ...draft, height: event.target.value })} />
      </label>
      <label>
        <span>默认贴目</span>
        <input type="number" step="any" value={draft.komi}
          onChange={(event) => setEdited({ ...draft, komi: event.target.value })} />
      </label>
      <button type="button" disabled={disabled || edited === null} onClick={apply}>保存新建默认值</button>
      <p>新建表单可临时修改这些值，不会反向改写默认值。</p>
      {error ? <p role="alert">{error}</p> : null}
    </fieldset>
  );
}

function ContinuousBudgetEditor({ preferences, disabled, onChange }: Pick<Props, "preferences" | "disabled" | "onChange">) {
  const [edited, setEdited] = useState<{
    timeEnabled: boolean; seconds: string; visitsEnabled: boolean; visits: string; stopOnEmpty: boolean;
  } | null>(null);
  const [error, setError] = useState<string | null>(null);
  const draft = edited ?? {
    timeEnabled: preferences.continuousTimeLimitEnabled,
    seconds: String(preferences.continuousTimeLimitSeconds),
    visitsEnabled: preferences.continuousVisitsLimitEnabled,
    visits: String(preferences.continuousVisitsLimit),
    stopOnEmpty: preferences.continuousStopOnEmptyBoard
  };
  function apply() {
    const next = {
      ...preferences,
      continuousTimeLimitEnabled: draft.timeEnabled,
      continuousTimeLimitSeconds: Number(draft.seconds),
      continuousVisitsLimitEnabled: draft.visitsEnabled,
      continuousVisitsLimit: Number(draft.visits),
      continuousStopOnEmptyBoard: draft.stopOnEmpty
    };
    const invalid = continuousBudgetError(next);
    setError(invalid);
    if (invalid) return;
    onChange(next);
    setEdited(null);
  }
  return (
    <fieldset className="preferences-grid" disabled={disabled}>
      <legend>连续分析预算</legend>
      <Toggle label="限制连续分析时间" checked={draft.timeEnabled} disabled={disabled}
        onChange={(timeEnabled) => setEdited({ ...draft, timeEnabled })} />
      <label>
        <span>连续分析时间（秒）</span>
        <input type="number" min={1} max={4294967295} step={1} value={draft.seconds}
          disabled={disabled || !draft.timeEnabled}
          onChange={(event) => setEdited({ ...draft, seconds: event.target.value })} />
      </label>
      <Toggle label="限制连续分析 visits" checked={draft.visitsEnabled} disabled={disabled}
        onChange={(visitsEnabled) => setEdited({ ...draft, visitsEnabled })} />
      <label>
        <span>连续分析 visits 上限</span>
        <input type="number" min={1} max={4294967295} step={1} value={draft.visits}
          disabled={disabled || !draft.visitsEnabled}
          onChange={(event) => setEdited({ ...draft, visits: event.target.value })} />
      </label>
      <Toggle label="空棋盘停止连续分析" checked={draft.stopOnEmpty} disabled={disabled}
        onChange={(stopOnEmpty) => setEdited({ ...draft, stopOnEmpty })} />
      <button type="button" disabled={disabled || edited === null} onClick={apply}>应用连续预算</button>
      <p>任一启用上限先到即停止。关闭上限保留数值；保存成功后才重新开始当前连续搜索，不修改有限分析或整盘请求。</p>
      {error ? <p role="alert">{error}</p> : null}
    </fieldset>
  );
}

function Toggle({ label, checked, disabled, onChange }: { label: string; checked: boolean; disabled?: boolean; onChange: (checked: boolean) => void }) {
  return (
    <label className="toggle-row">
      <span>{label}</span>
      <input type="checkbox" checked={checked} disabled={disabled} onChange={(event) => onChange(event.target.checked)} />
    </label>
  );
}
