import type { AppPreferences, BoardTheme, ReviewMode } from "../domain/preferences";

type Props = {
  preferences: AppPreferences;
  status: string;
  disabled?: boolean;
  onChange: (preferences: AppPreferences) => void;
};

export function PreferencesPanel({ preferences, status, disabled = false, onChange }: Props) {
  function update(patch: Partial<AppPreferences>) {
    onChange({ ...preferences, ...patch });
  }

  return (
    <section className="preferences-panel" aria-label="设置">
      <div className="preferences-header">
        <h2>设置</h2>
        <span>{status}</span>
      </div>
      <div className="preferences-grid">
        <Toggle label="候选" checked={preferences.showCandidates} disabled={disabled} onChange={(checked) => update({ showCandidates: checked })} />
        <Toggle label="领地" checked={preferences.showOwnership} disabled={disabled} onChange={(checked) => update({ showOwnership: checked })} />
        <Toggle label="策略" checked={preferences.showPolicy} disabled={disabled} onChange={(checked) => update({ showPolicy: checked })} />
        <Toggle label="自动载入缓存" checked={preferences.autoLoadCache} disabled={disabled} onChange={(checked) => update({ autoLoadCache: checked })} />
        <Toggle label="自动保存分析" checked={preferences.autoSaveAnalysis} disabled={disabled} onChange={(checked) => update({ autoSaveAnalysis: checked })} />
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
        <label>
          <span>棋盘对比</span>
          <select value={preferences.boardTheme} disabled={disabled} onChange={(event) => update({ boardTheme: event.target.value as BoardTheme })}>
            <option value="classic">浅色</option>
            <option value="high-contrast">高对比</option>
          </select>
        </label>
      </div>
    </section>
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
