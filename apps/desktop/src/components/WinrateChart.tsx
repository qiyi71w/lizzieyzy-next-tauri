import { useEffect, useRef, useState, type ChangeEvent, type MouseEvent } from "react";
import { useElementSize } from "../workspace/useElementSize";
import { type ChartPoint, type WinrateChartModel } from "../domain/winrateChart";
import { chartPointText, renderWinrateChart } from "../domain/renderWinrateChart";
import type { NodePath } from "../domain/types";

type Props = {
  model: WinrateChartModel;
  onSelectNode?: (path: NodePath) => void;
};


export function WinrateChart({ model, onSelectNode }: Props) {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const size = useElementSize(canvasRef);
  const [hover, setHover] = useState<string | null>(null);

  useEffect(() => {
    if (!model.hoverEnabled) setHover(null);
  }, [model.hoverEnabled]);

  useEffect(() => {
    const canvas = canvasRef.current;
    const ctx = canvas?.getContext("2d");
    if (!canvas || !ctx) return;

    const { width, height, dpr } = size;
    if (width <= 0 || height <= 0) return;
    canvas.width = Math.floor(width * dpr);
    canvas.height = Math.floor(height * dpr);
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    renderWinrateChart(ctx, model, width, height);
  }, [model, size]);

  function pointAt(clientX: number): ChartPoint | null {
    const canvas = canvasRef.current;
    if (!canvas || model.points.length === 0) return null;
    const width = canvas.getBoundingClientRect().width;
    if (width <= 0) return null;
    const ratio = Math.min(1, Math.max(0, (clientX - 30) / Math.max(width - 60, 1)));
    const nodeIndex = Math.round(ratio * Math.max(model.points.length - 1, 0));
    return model.points[nodeIndex] ?? null;
  }

  function onMove(event: MouseEvent<HTMLCanvasElement>) {
    if (!model.hoverEnabled) return;
    const rect = event.currentTarget.getBoundingClientRect();
    const point = pointAt(event.clientX - rect.left);
    if (!point) {
      setHover(null);
      return;
    }
    setHover(chartPointText(point, model));
  }

  function selectPoint(point: ChartPoint, target: HTMLElement) {
    if (!onSelectNode || isChartHidden(target)) return;
    onSelectNode({ indices: [...point.path.indices] });
  }

  function onCanvasClick(event: MouseEvent<HTMLCanvasElement>) {
    const rect = event.currentTarget.getBoundingClientRect();
    const point = pointAt(event.clientX - rect.left);
    if (point) selectPoint(point, event.currentTarget);
  }

  function onSelectChange(event: ChangeEvent<HTMLSelectElement>) {
    const point = model.points.find(({ path }) => JSON.stringify(path.indices) === event.currentTarget.value);
    if (point) selectPoint(point, event.currentTarget);
  }

  const gapCount = model.points.filter((point) => point.analysis == null).length;

  return (
    <div
      className="winrate-chart-shell"
      data-perspective={model.perspective}
      data-show-winrate={model.showWinrate ? "true" : "false"}
      data-show-score={model.showScore ? "true" : "false"}
      data-score-available={model.scoreAvailable ? "true" : "false"}
      data-score-scale={String(model.scoreScale)}
      data-current-move={String(model.currentMove)}
      data-gap-count={String(gapCount)}
      data-bar-count={String(model.bars.length)}
      data-bar-ranks={model.bars.map((bar) => bar.rank).join(",")}
      data-hover={model.hoverEnabled ? "true" : "false"}
      data-selected-path={model.selectedPath.indices.join(",")}
    >
      <canvas
        ref={canvasRef}
        className="winrate-chart"
        aria-label="胜率走势"
        onMouseMove={model.hoverEnabled ? onMove : undefined}
        onMouseLeave={model.hoverEnabled ? () => setHover(null) : undefined}
        onClick={onSelectNode ? onCanvasClick : undefined}
      />
      {hover ? <div className="winrate-chart-hover" role="status">{hover}</div> : null}
      <select
        className="winrate-chart-node-select"
        aria-label="选择图表节点"
        value={JSON.stringify(model.selectedPath.indices)}
        disabled={!onSelectNode}
        onChange={onSelectChange}
      >
        {model.points.map((point) => {
          const pathLabel = point.path.indices.length === 0 ? "根" : point.path.indices.join(".");
          const nodeLabel = point.path.indices.length === 0
            ? "根节点"
            : point.isMove ? `第 ${point.moveNumber} 手` : `第 ${point.moveNumber} 手后节点`;
          return <option key={JSON.stringify(point.path.indices)} value={JSON.stringify(point.path.indices)}>{nodeLabel} · 路径 {pathLabel}</option>;
        })}
      </select>
    </div>
  );
}



function isChartHidden(element: HTMLElement): boolean {
  let current: HTMLElement | null = element;
  while (current) {
    if (current.hidden || current.getAttribute("aria-hidden") === "true") return true;
    const style = window.getComputedStyle(current);
    if (style.display === "none" || style.visibility === "hidden") return true;
    current = current.parentElement;
  }
  return false;
}
