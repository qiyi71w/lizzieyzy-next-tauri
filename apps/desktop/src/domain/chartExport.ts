import { t } from "../i18n/resources";
import { displayedScore, displayedWinrate, type WinrateChartModel } from "./winrateChart";
import { renderWinrateChart } from "./renderWinrateChart";

export type ChartExportSnapshot = {
  model: WinrateChartModel;
  defaultFileName: string;
};

/** Freeze at invocation, before any destination selection or asynchronous work.
 * Only the document's attached selected-line model enters this read-only path. */
export function captureChartExport(model: WinrateChartModel, sourceSgfPath: string | null): ChartExportSnapshot {
  if (!model.points.some((point) =>
    (model.showWinrate && displayedWinrate(point, model.perspective, model.selectedToPlay) != null) ||
    (model.showScore && displayedScore(point, model.perspective, model.selectedToPlay) != null)
  )) {
    throw new Error(t("chart.export.noData"));
  }
  const basename = sourceSgfPath?.split(/[\\/]/).at(-1);
  const stem = basename && /\.sgf$/i.test(basename) ? basename.slice(0, -4) : "untitled";
  return {
    model: {
      ...model,
      selectedPath: { indices: [...model.selectedPath.indices] },
      points: model.points.map((point) => ({ ...point, path: { indices: [...point.path.indices] }, analysis: point.analysis ? { ...point.analysis } : null })),
      bars: model.bars.map((bar) => ({ ...bar })),
      hoverEnabled: false
    },
    defaultFileName: `${stem}-winrate-m${model.currentMove}.png`
  };
}

/** Capture actual canvas pixels; the shared Rust image writer owns PNG encoding,
 * destination confirmation, atomic replacement and the sole directory state. */
export function renderChartExport(snapshot: ChartExportSnapshot): { width: number; height: number; rgba: number[] } {
  const canvas = document.createElement("canvas");
  canvas.width = 1600;
  canvas.height = 600;
  const ctx = canvas.getContext("2d");
  if (!ctx) throw new Error(t("chart.export.canvasUnavailable"));
  renderWinrateChart(ctx, snapshot.model, canvas.width, canvas.height, true);
  return { width: canvas.width, height: canvas.height, rgba: Array.from(ctx.getImageData(0, 0, canvas.width, canvas.height).data) };
}
