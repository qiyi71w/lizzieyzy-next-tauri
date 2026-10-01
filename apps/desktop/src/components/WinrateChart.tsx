import { useEffect, useRef, useState, type ChangeEvent, type MouseEvent } from "react";
import { useElementSize } from "../workspace/useElementSize";
import {
  displayedScore,
  displayedWinrate,
  type ChartPoint,
  type WinrateChartModel
} from "../domain/winrateChart";
import type { NodePath } from "../domain/types";

type Props = {
  model: WinrateChartModel;
  onSelectNode?: (path: NodePath) => void;
};

const BAR_COLORS = {
  inaccuracy: "#d4a017",
  mistake: "#d03232",
  blunder: "#8b2a9b"
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
    ctx.clearRect(0, 0, width, height);

    ctx.fillStyle = "#16181d";
    ctx.fillRect(0, 0, width, height);

    ctx.strokeStyle = "#282d37";
    ctx.lineWidth = 1;
    ctx.setLineDash([3, 3]);
    for (let i = 1; i <= 3; i += 1) {
      const y = (height / 4) * i;
      ctx.beginPath();
      ctx.moveTo(0, y);
      ctx.lineTo(width, y);
      ctx.stroke();
    }
    ctx.setLineDash([]);

    const lastNodeIndex = Math.max(model.points.length - 1, 1);
    const nodeToX = (nodeIndex: number) => (
      Math.min(Math.max(nodeIndex, 0), lastNodeIndex) / lastNodeIndex
    ) * width;

    for (const bar of model.bars) {
      const x1 = nodeToX(bar.fromIndex);
      const x2 = nodeToX(bar.toIndex);
      const barHeight = bar.rank === "blunder" ? height * 0.5 : bar.rank === "mistake" ? height * 0.35 : height * 0.2;
      ctx.fillStyle = BAR_COLORS[bar.rank];
      ctx.globalAlpha = 0.45;
      ctx.fillRect(Math.min(x1, x2), height - barHeight, Math.max(2, Math.abs(x2 - x1)), barHeight);
      ctx.globalAlpha = 1;
    }

    if (model.showWinrate) {
      strokeSegments(ctx, model.points, nodeToX, (point) => {
        const winrate = displayedWinrate(point, model.perspective, model.selectedToPlay);
        return winrate == null ? null : height - winrate * height;
      }, "#60a5fa");
    }
    if (model.showScore) {
      strokeSegments(ctx, model.points, nodeToX, (point) => {
        const score = displayedScore(point, model.perspective, model.selectedToPlay);
        if (score == null) return null;
        return height / 2 - (score / model.scoreScale) * (height / 2);
      }, "#34d399");
    }

    const selectedIndex = model.points.findIndex((point) => (
      point.path.indices.length === model.selectedPath.indices.length
      && point.path.indices.every((index, position) => index === model.selectedPath.indices[position])
    ));
    if (selectedIndex >= 0) {
      const markerX = nodeToX(selectedIndex);
      const current = model.points[selectedIndex];
      const markerWinrate = displayedWinrate(current, model.perspective, model.selectedToPlay);
      const markerY = markerWinrate == null ? height * 0.5 : height - markerWinrate * height;
      ctx.strokeStyle = "#f59e0b";
      ctx.lineWidth = 1.5;
      ctx.setLineDash([2, 2]);
      ctx.beginPath();
      ctx.moveTo(markerX, 0);
      ctx.lineTo(markerX, height);
      ctx.stroke();
      ctx.setLineDash([]);
      ctx.fillStyle = "#f59e0b";
      ctx.beginPath();
      ctx.arc(markerX, markerY, 3.5, 0, Math.PI * 2);
      ctx.fill();
    }
  }, [model, size]);

  function pointAt(clientX: number): ChartPoint | null {
    const canvas = canvasRef.current;
    if (!canvas || model.points.length === 0) return null;
    const width = canvas.getBoundingClientRect().width;
    if (width <= 0) return null;
    const ratio = Math.min(1, Math.max(0, clientX / width));
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
    const parts = [point.path.indices.length === 0
      ? "根节点"
      : point.isMove ? `第 ${point.moveNumber} 手` : `第 ${point.moveNumber} 手后节点`];
    const winrate = displayedWinrate(point, model.perspective, model.selectedToPlay);
    const score = displayedScore(point, model.perspective, model.selectedToPlay);
    if (model.showWinrate && winrate != null) parts.push(`胜率 ${(winrate * 100).toFixed(1)}%`);
    if (model.showScore && score != null) parts.push(`目差 ${score.toFixed(1)}`);
    if (winrate == null && score == null) parts.push("无分析");
    setHover(parts.join(" · "));
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

function strokeSegments(
  ctx: CanvasRenderingContext2D,
  points: ChartPoint[],
  nodeToX: (nodeIndex: number) => number,
  yFor: (point: ChartPoint) => number | null,
  color: string
) {
  ctx.strokeStyle = color;
  ctx.lineWidth = 2;
  let drawing = false;
  ctx.beginPath();
  for (const [nodeIndex, point] of points.entries()) {
    const y = yFor(point);
    if (y == null) {
      if (drawing) {
        ctx.stroke();
        ctx.beginPath();
        drawing = false;
      }
      continue;
    }
    const x = nodeToX(nodeIndex);
    if (!drawing) {
      ctx.moveTo(x, y);
      drawing = true;
    } else {
      ctx.lineTo(x, y);
    }
  }
  if (drawing) ctx.stroke();
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
