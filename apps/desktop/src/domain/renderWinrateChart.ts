import { t } from "../i18n/resources";
import { chartBaseline, displayedScore, displayedWinrate, scoreLeadText, type ChartPoint, type WinrateChartModel } from "./winrateChart";

const COLORS = { background: "#16181d", grid: "#363d49", text: "#e1e6ee", muted: "#aeb9c9", winrate: "#60a5fa", score: "#34d399", current: "#fbbf24" };
const BAR_COLORS = { inaccuracy: "#d4a017", mistake: "#d03232", blunder: "#8b2a9b" };

export function chartPointText(point: ChartPoint, model: WinrateChartModel): string {
  const winrate = displayedWinrate(point, model.perspective, model.selectedToPlay);
  const score = scoreLeadText(displayedScore(point, model.perspective, model.selectedToPlay), model.perspective, model.selectedToPlay, t("chart.blackPrefix"), t("chart.whitePrefix"));
  const parts = [`${point.moveNumber}${t("chart.move")}`];
  if (model.showWinrate && winrate != null) parts.push(`${(winrate * 100).toFixed(1)}%`);
  if (model.showScore && score != null) parts.push(score);
  if (parts.length === 1) parts.push(t("chart.noAnalysis"));
  return parts.join(" · ");
}

/** One renderer owns both the measured workbench canvas and the export image.
 * Labels occupy reserved bands; they never cover a curve or the zero baseline. */
export function renderWinrateChart(ctx: CanvasRenderingContext2D, model: WinrateChartModel, width: number, height: number, exported = false): void {
  const unit = exported ? 2 : 1;
  const font = exported ? 22 : 10;
  const left = 30 * unit;
  const right = width - 30 * unit;
  const top = (exported ? 42 : 26) * unit;
  const bottom = height - (exported ? 44 : 26) * unit;
  const plotHeight = Math.max(bottom - top, 1);
  const plotWidth = Math.max(right - left, 1);
  const mid = top + plotHeight / 2;
  const nodeToX = (index: number) => left + (index / Math.max(model.points.length - 1, 1)) * plotWidth;
  const winrateY = (point: ChartPoint) => {
    const value = displayedWinrate(point, model.perspective, model.selectedToPlay);
    return value == null ? null : bottom - value * plotHeight;
  };
  const scoreY = (point: ChartPoint) => {
    const value = displayedScore(point, model.perspective, model.selectedToPlay);
    return value == null ? null : mid - (value / model.scoreScale) * plotHeight / 2;
  };
  const baseline = chartBaseline(model);
  ctx.clearRect(0, 0, width, height);
  ctx.fillStyle = COLORS.background;
  ctx.fillRect(0, 0, width, height);
  ctx.font = `${font}px "Noto Sans SC", "Microsoft YaHei", sans-serif`;
  ctx.textBaseline = "middle";
  ctx.textAlign = "left";
  ctx.lineWidth = unit;
  ctx.strokeStyle = COLORS.grid;
  ctx.setLineDash([3 * unit, 3 * unit]);
  // The emphasized baseline is the sole midline, not a duplicate grid stroke.
  for (const fraction of [0.25, 0.75]) {
    const y = top + plotHeight * fraction;
    ctx.beginPath(); ctx.moveTo(left, y); ctx.lineTo(right, y); ctx.stroke();
  }
  if (baseline.visible) {
    ctx.strokeStyle = COLORS.muted;
    ctx.lineWidth = 1.5 * unit;
    ctx.beginPath(); ctx.moveTo(left, mid); ctx.lineTo(right, mid); ctx.stroke();
    if (baseline.mark) {
      ctx.fillStyle = COLORS.text;
      ctx.textAlign = baseline.mark === "0" ? "right" : "left";
      ctx.fillText(baseline.mark, baseline.mark === "0" ? width - 2 * unit : 2 * unit, mid);
    }
  }
  ctx.setLineDash([]);
  ctx.strokeStyle = COLORS.grid;
  ctx.lineWidth = unit;
  ctx.beginPath(); ctx.moveTo(left, top); ctx.lineTo(left, bottom); ctx.lineTo(right, bottom); ctx.lineTo(right, top); ctx.stroke();
  ctx.fillStyle = COLORS.muted;
  if (model.showWinrate && model.points.some((point) => winrateY(point) != null)) {
    ctx.textAlign = "left";
    ctx.fillText("100%", 2 * unit, top);
    ctx.fillText("0%", 2 * unit, bottom);
  }
  if (model.showScore && model.points.some((point) => scoreY(point) != null)) {
    ctx.textAlign = "right";
    ctx.fillText(`+${model.scoreScale.toFixed(1)}`, width - 2 * unit, top);
    ctx.fillText(`−${model.scoreScale.toFixed(1)}`, width - 2 * unit, bottom);
  }
  for (const bar of model.bars) {
    const x1 = nodeToX(bar.fromIndex), x2 = nodeToX(bar.toIndex);
    const barHeight = plotHeight * (bar.rank === "blunder" ? 0.5 : bar.rank === "mistake" ? 0.35 : 0.2);
    ctx.fillStyle = BAR_COLORS[bar.rank];
    ctx.globalAlpha = 0.45;
    ctx.fillRect(x1, bottom - barHeight, Math.max(2 * unit, x2 - x1), barHeight);
    ctx.globalAlpha = 1;
  }
  if (model.showWinrate) strokeSegments(ctx, model.points, nodeToX, winrateY, COLORS.winrate, unit);
  if (model.showScore) strokeSegments(ctx, model.points, nodeToX, scoreY, COLORS.score, unit);
  const selectedIndex = model.points.findIndex((point) => point.path.indices.length === model.selectedPath.indices.length && point.path.indices.every((index, position) => index === model.selectedPath.indices[position]));
  if (selectedIndex >= 0) {
    const x = nodeToX(selectedIndex);
    ctx.strokeStyle = COLORS.current;
    ctx.lineWidth = 1.5 * unit;
    ctx.setLineDash([2 * unit, 2 * unit]);
    ctx.beginPath(); ctx.moveTo(x, top); ctx.lineTo(x, bottom); ctx.stroke();
    ctx.setLineDash([]);
    for (const y of [model.showWinrate ? winrateY(model.points[selectedIndex]) : null, model.showScore ? scoreY(model.points[selectedIndex]) : null]) {
      if (y == null) continue;
      ctx.fillStyle = COLORS.current;
      ctx.beginPath(); ctx.arc(x, y, 3.5 * unit, 0, Math.PI * 2); ctx.fill();
    }
    ctx.textAlign = "center";
    ctx.fillStyle = COLORS.current;
    ctx.fillText(`${t("chart.current")} ${chartPointText(model.points[selectedIndex], model)}`, width / 2, exported ? 62 : 12, width - 8 * unit);
  }
  const endpoints = [model.points[0], model.points.at(-1)];
  ctx.fillStyle = COLORS.text;
  endpoints.forEach((point, index) => {
    if (!point || (index === 1 && model.points.length === 1)) return;
    ctx.textAlign = index === 0 ? "left" : "right";
    ctx.fillText(chartPointText(point, model), index === 0 ? left : right, bottom + 14 * unit, plotWidth / 2 - 4 * unit);
  });
  if (exported) {
    const perspective = model.perspective === "black" ? t("chart.blackPerspective") : model.selectedToPlay === "white" ? t("chart.whitePerspective") : t("chart.selectedBlackPerspective");
    const labels = [perspective];
    if (model.showWinrate) labels.push(t("chart.winrate"));
    if (model.showScore) labels.push(t("chart.score"));
    if (model.bars.length) labels.push(t("chart.blunderBar"));
    ctx.textAlign = "left";
    ctx.fillStyle = COLORS.text;
    ctx.fillText(labels.join(" · "), left, 26);
    ctx.textAlign = "center";
    ctx.fillStyle = COLORS.muted;
    ctx.fillText(t("chart.axis"), width / 2, height - 22);
  }
}

function strokeSegments(ctx: CanvasRenderingContext2D, points: ChartPoint[], nodeToX: (index: number) => number, yFor: (point: ChartPoint) => number | null, color: string, unit: number): void {
  ctx.strokeStyle = color;
  ctx.fillStyle = color;
  ctx.lineWidth = 2 * unit;
  let drawing = false;
  ctx.beginPath();
  for (const [index, point] of points.entries()) {
    const y = yFor(point);
    if (y == null) {
      if (drawing) ctx.stroke();
      ctx.beginPath(); drawing = false;
      continue;
    }
    const x = nodeToX(index);
    if (drawing) ctx.lineTo(x, y);
    else ctx.moveTo(x, y);
    drawing = true;
    // A single valid point must still be visible, including isolated gap islands.
    ctx.fillRect(x - unit, y - unit, 2 * unit, 2 * unit);
  }
  if (drawing) ctx.stroke();
}
