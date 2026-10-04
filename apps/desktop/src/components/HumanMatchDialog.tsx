import { useState } from "react";
import type { EngineProfileRecordDto, ForegroundEngineSnapshotDto, MatchDefaultsDto, MatchModeDto, PositionDto, SgfTreeNodeDto } from "../domain/types";
import { profileHasPendingChanges, verifiedGameMoveCapabilitiesLabel } from "../domain/foregroundEngine";

type Props = {
  mode?: MatchModeDto;
  defaults: MatchDefaultsDto;
  profiles: EngineProfileRecordDto[];
  engineSnapshot?: ForegroundEngineSnapshotDto;
  /** Exact position of the selected node when continuing; absent for a New game. */
  continuation?: PositionDto | null;
  continuationRoot?: SgfTreeNodeDto;
  pending: boolean;
  error?: string | null;
  onStart: (settings: MatchDefaultsDto) => void;
  onCancel: () => void;
};

export function HumanMatchDialog({ mode = "human", defaults, profiles, engineSnapshot, continuation, continuationRoot, pending, error, onStart, onCancel }: Props) {
  const [draft, setDraft] = useState(() => ({ ...defaults, pk_black: { ...defaults.pk_black }, pk_white: { ...defaults.pk_white } }));
  const [rootConfirmed, setRootConfirmed] = useState(false);
  const selected = profiles.find((profile) => profile.id === draft.profile_id);
  const generic = selected?.profile.adapter_kind === "generic_gtp";
  const pk = mode === "pk";
  const pkSides = ["pk_black", "pk_white"] as const;
  const positiveInteger = (value: number) => Number.isInteger(value) && value > 0;
  const pkValid = pkSides.every((side) => {
    const settings = draft[side];
    const saved = profiles.find((profile) => profile.id === settings.profile_id);
    return Boolean(saved) && positiveInteger(settings.deadline_ms)
      && (saved?.profile.adapter_kind === "generic_gtp" || positiveInteger(settings.kata_max_visits));
  }) && positiveInteger(draft.pk_max_moves);
  const verifiedSnapshot = selected && engineSnapshot?.lifecycle.state === "ready"
    && engineSnapshot.lifecycle.run.profile_id === selected.id
    && !profileHasPendingChanges(selected.profile, engineSnapshot) ? engineSnapshot : null;
  const positionValid = continuation ? rootConfirmed : Boolean(draft.rules)
    && Number.isInteger(draft.board_size) && draft.board_size >= 2 && draft.board_size <= 19
    && Number.isFinite(draft.komi) && Number.isInteger(draft.komi * 2)
    && (draft.handicap === 0 || ([9, 13, 19].includes(draft.board_size) && Number.isInteger(draft.handicap) && draft.handicap >= 2 && draft.handicap <= 9));
  const valid = positionValid && (pk ? pkValid : Boolean(selected)
    && positiveInteger(draft.deadline_ms) && (generic || positiveInteger(draft.kata_max_visits)));
  const title = `${pk ? "PK" : "人机"}${continuation ? "续弈" : "新局"}`;
  const inheritedRules = continuationRoot?.properties.find((property) => property.key === "RU")?.values.join(", ") ?? "未标注（启动时校验）";
  const inheritedKomi = continuationRoot?.properties.find((property) => property.key === "KM")?.values.join(", ") ?? "未标注（启动时校验）";
  const inheritedHandicap = continuationRoot?.properties.find((property) => property.key === "HA")?.values.join(", ") ?? "0（未标注）";
  return <div className="shortcut-reference-backdrop">
    <form className="new-document-dialog human-match-dialog" role="dialog" aria-modal="true" aria-label={title}
      onKeyDown={(event) => { if (event.key === "Escape" && !pending) { event.stopPropagation(); onCancel(); } }}
      onSubmit={(event) => { event.preventDefault(); if (valid && !pending) onStart({ ...draft }); }}>
      <div className="new-document-header"><h2>{title}</h2><p>{continuation
        ? `从当前节点精确续弈（第 ${continuation.move_number} 手后，${continuation.to_play === "white" ? "白" : "黑"}方行棋）。棋盘 ${continuation.board_width}×${continuation.board_height} · 规则 ${inheritedRules} · 贴目 ${inheritedKomi} · 让子 ${inheritedHandicap}。棋盘、规则、贴目、让子与摆子均继承当前棋谱，不可修改。`
        : "启动成功后替换当前棋谱。"}引擎 Ready 后验证规则和落子能力。</p></div>
      <fieldset className="new-document-fields" disabled={pending}>
        {continuation ? null : <>
          <label><span>棋盘大小</span><input autoFocus required type="number" min={2} max={19} step={1} value={draft.board_size} onChange={(e) => setDraft({ ...draft, board_size: e.target.valueAsNumber })} /></label>
          <label><span>贴目</span><input required type="number" step={0.5} value={draft.komi} onChange={(e) => setDraft({ ...draft, komi: e.target.valueAsNumber })} /></label>
          <label><span>让子（0 或 2–9）</span><input required type="number" min={0} max={9} step={1} value={draft.handicap} onChange={(e) => setDraft({ ...draft, handicap: e.target.valueAsNumber })} /></label>
        </>}
        {pk ? <>
          {pkSides.map((side) => {
            const label = side === "pk_black" ? "黑方" : "白方";
            const settings = draft[side];
            const sideGeneric = profiles.find((profile) => profile.id === settings.profile_id)?.profile.adapter_kind === "generic_gtp";
            return <div key={side}>
              <label><span>{label}已保存的引擎配置</span><select required value={settings.profile_id ?? ""} onChange={(e) => setDraft({ ...draft, [side]: { ...settings, profile_id: e.target.value || null } })}><option value="">请选择配置</option>{profiles.map((profile) => <option key={profile.id} value={profile.id}>{profile.profile.name}{profile.profile.adapter_kind === "generic_gtp" ? "（GTP）" : ""}</option>)}</select></label>
              <label><span>{label}每步请求截止（毫秒，非对局计时）</span><input required type="number" min={1} step={1} value={settings.deadline_ms} onChange={(e) => setDraft({ ...draft, [side]: { ...settings, deadline_ms: e.target.valueAsNumber } })} /></label>
              {sideGeneric
                ? <p className="message">{label} GTP：仅使用每步硬截止（引擎支持时映射为 GTP 时间设置），不设置 visits，不提供分析。</p>
                : <label><span>{label} KataGo 每步最大 visits</span><input required type="number" min={1} step={1} value={settings.kata_max_visits} onChange={(e) => setDraft({ ...draft, [side]: { ...settings, kata_max_visits: e.target.valueAsNumber } })} /></label>}
            </div>;
          })}
          <label><span>本场最大手数（含 pass，不含继承历史）</span><input required type="number" min={1} step={1} value={draft.pk_max_moves} onChange={(e) => setDraft({ ...draft, pk_max_moves: e.target.valueAsNumber })} /></label>
          <p className="message">支持 KataGo 与已合格 GNU Go 3.8（GTP）任意组合，可选择同一配置；双方始终使用独立 run。GTP 取步无法按协议取消：暂停时正在取步的 GTP 进程会被回收，只有点击恢复才按本场原始配置快照重建。</p>
        </> : <>
          <label><span>人类执子</span><select value={draft.human_color} onChange={(e) => setDraft({ ...draft, human_color: e.target.value === "white" ? "white" : "black" })}><option value="black">黑</option><option value="white">白</option></select></label>
          <label><span>已保存的引擎配置</span><select required value={draft.profile_id ?? ""} onChange={(e) => setDraft({ ...draft, profile_id: e.target.value || null })}><option value="">请选择配置</option>{profiles.map((profile) => <option key={profile.id} value={profile.id}>{profile.profile.name}</option>)}</select></label>
        </>}
        {continuation ? null : <label><span>精确规则</span><select required value={draft.rules ?? ""} onChange={(e) => setDraft({ ...draft, rules: e.target.value === "chinese" ? "chinese" : e.target.value === "chinese_kgs" ? "chinese_kgs" : null })}><option value="">请选择规则</option><option value="chinese">Chinese</option><option value="chinese_kgs">Chinese KGS</option></select></label>}
        {!pk ? <>
          <label><span>每步请求截止（毫秒，非对局计时）</span><input required type="number" min={1} step={1} value={draft.deadline_ms} onChange={(e) => setDraft({ ...draft, deadline_ms: e.target.valueAsNumber })} /></label>
          {!generic ? <label><span>KataGo 每步最大 visits</span><input required type="number" min={1} step={1} value={draft.kata_max_visits} onChange={(e) => setDraft({ ...draft, kata_max_visits: e.target.valueAsNumber })} /></label> : null}
        </> : null}
        {continuation ? <label className="human-match-root-confirmation"><input type="checkbox" required checked={rootConfirmed} onChange={(e) => setRootConfirmed(e.target.checked)} />
          <span>黑方/白方名称和结果属于整个棋谱：开始后根节点 PB/PW 改为本局双方，RE 清除。原后续着法和注释保留为变化，可撤销。</span></label> : null}
      </fieldset>
      {pk ? pkSides.map((side) => {
        const saved = profiles.find((profile) => profile.id === draft[side].profile_id);
        const verified = saved && engineSnapshot?.lifecycle.state === "ready"
          && engineSnapshot.lifecycle.run.profile_id === saved.id && !profileHasPendingChanges(saved.profile, engineSnapshot) ? engineSnapshot : null;
        return <p key={side} className="message" role="status" aria-label={`${side === "pk_black" ? "黑方" : "白方"}配置落子能力`}>{side === "pk_black" ? "黑方" : "白方"}：{verified
          ? verifiedGameMoveCapabilitiesLabel(verified)
          : "能力未验证；启动时分别按实际 Ready run 验证规则、局面与预算。"}</p>;
      }) : <p className="message" role="status" aria-label="所选配置落子能力">{verifiedSnapshot
        ? verifiedGameMoveCapabilitiesLabel(verifiedSnapshot)
        : "所选配置落子能力未验证；启动到 Ready 后按实际运行配置验证规则、局面与计算预算。"}</p>}
      {error ? <p className="new-document-error" role="alert">{error}</p> : null}
      {profiles.length === 0 ? <p role="status">请先在引擎设置中保存配置。</p> : null}
      <div className="document-departure-actions"><button className="primary" type="submit" disabled={pending || !valid}>{pending ? "正在启动…" : continuation ? "开始续弈" : "开始新局"}</button><button type="button" onClick={onCancel}>{pending ? "停止启动" : "取消"}</button></div>
    </form>
  </div>;
}
