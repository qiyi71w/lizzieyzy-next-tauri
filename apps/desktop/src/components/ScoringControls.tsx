import type { AreaCompensationDto, ScoringRuleDto, ScoringSessionDto } from "../domain/types";

type Props = {
  scoring: ScoringSessionDto;
  busy: boolean;
  onRule: (rule: ScoringRuleDto) => void;
  onSettings: (compensation: AreaCompensationDto, handicap: number) => void;
  onExit: (confirm: boolean) => void;
};

export function ScoringControls({ scoring, busy, onRule, onSettings, onExit }: Props) {
  const bonus = scoring.rule === "territory" ? 0 : scoring.compensation === "handicap"
    ? scoring.handicap : scoring.compensation === "handicap_minus_one" ? Math.max(scoring.handicap - 1, 0) : 0;
  return <div className="root-setup-tools scoring-tools" role="group" aria-label="本地计分">
    <strong>本地计分 · {scoring.result === "0" ? "和棋 0" : scoring.result}</strong>
    <label>规则 <select aria-label="计分规则" disabled={busy} value={scoring.rule}
      onChange={(event) => onRule(event.target.value as ScoringRuleDto)}>
      <option value="area">面积（活子＋围空）</option>
      <option value="territory">地盘（围空＋实际提子＋对方死子）</option>
    </select></label>
    <label>让子数 h <input aria-label="计分让子数" type="number" min="0" max="625" step="1" disabled={busy}
      value={scoring.handicap} onChange={(event) => {
        const handicap = Number(event.target.value);
        if (Number.isInteger(handicap) && handicap >= 0 && handicap <= 625) onSettings(scoring.compensation, handicap);
      }} /></label>
    <label>面积白方补偿 <select aria-label="面积让子补偿" disabled={busy || scoring.rule === "territory"}
      value={scoring.compensation} onChange={(event) => onSettings(event.target.value as AreaCompensationDto, scoring.handicap)}>
      <option value="none">无 (0)</option>
      <option value="handicap">N (h)</option>
      <option value="handicap_minus_one">N−1 (max(h−1, 0))</option>
    </select></label>
    <span>黑：活子 {scoring.black_stones} · 围空 {scoring.black_territory} · 实际提子 {scoring.position.captures_black} · 对方死子 {scoring.white_dead} = {scoring.black_total}</span>
    <span>白：活子 {scoring.white_stones} · 围空 {scoring.white_territory} · 实际提子 {scoring.position.captures_white} · 对方死子 {scoring.black_dead} · 贴目 {scoring.komi} · 面积补偿 {bonus} = {scoring.white_total}</span>
    <span>点击棋子切换整组死活，点击空点切换中立；预览不写棋谱。</span>
    <button type="button" disabled={busy} onClick={() => onExit(false)}>取消计分</button>
    <button type="button" disabled={busy} onClick={() => onExit(true)}>确认结果 {scoring.result}</button>
  </div>;
}
