import { useState } from "react";
import { continuousBudgetError, type AppPreferences, type BoardTheme, type GraphPerspective, type NextMoveReviewMarkerMode, type ReviewMode, type SubBoardContentMode } from "../domain/preferences";
import { parsePositiveScoreLeadScale } from "../domain/winrateChart";

type Props = {
  preferences: AppPreferences;
  status: string;
  disabled?: boolean;
  scoreLeadAvailable?: boolean;
  onChange: (preferences: AppPreferences) => void;
};

export function PreferencesPanel({ preferences, status, disabled = false, scoreLeadAvailable = true, onChange }: Props) {
  function update(patch: Partial<AppPreferences>) {
    onChange({ ...preferences, ...patch });
  }

  return (
    <section className="preferences-panel" aria-label="设置">
      <div className="preferences-header">
        <h2>设置</h2>
        <span>{status}</span>
      </div>
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
          <select value={preferences.boardTheme} disabled={disabled} onChange={(event) => update({ boardTheme: event.target.value as BoardTheme })}>
            <option value="classic">浅色</option>
            <option value="high-contrast">高对比</option>
          </select>
        </label>
      </fieldset>
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
