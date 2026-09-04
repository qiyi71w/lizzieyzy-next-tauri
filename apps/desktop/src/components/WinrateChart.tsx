import { useEffect, useRef, useState, type MouseEvent } from "react";
import {
  displayedScore,
  displayedWinrate,
  type ChartPoint,
  type WinrateChartModel
} from "../domain/winrateChart";

type Props = { model: WinrateChartModel };

const BAR_COLORS = {
  inaccuracy: "#d4a017",
  mistake: "#d03232",
  blunder: "#8b2a9b"
};

export function WinrateChart({ model }: Props) {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const [hover, setHover] = useState<{ moveNumber: number; label: string } | null>(null);

  useEffect(() => {
    if (!model.hoverEnabled) setHover(null);
  }, [model.hoverEnabled]);

  useEffect(() => {
    const canvas = canvasRef.current;
    const ctx = canvas?.getContext("2d");
    if (!canvas || !ctx) return;

    const dpr = window.devicePixelRatio || 1;
    const width = canvas.clientWidth || 240;
    const height = canvas.clientHeight || 90;
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

    const maxMove = Math.max(model.currentMove, model.points.at(-1)?.moveNumber ?? 0, 1);
    const turnToX = (turn: number) => (Math.min(Math.max(turn, 0), maxMove) / maxMove) * width;

    for (const bar of model.bars) {
      const x1 = turnToX(bar.fromMove);
      const x2 = turnToX(bar.toMove);
      const barHeight = bar.rank === "blunder" ? height * 0.5 : bar.rank === "mistake" ? height * 0.35 : height * 0.2;
      ctx.fillStyle = BAR_COLORS[bar.rank];
      ctx.globalAlpha = 0.45;
      ctx.fillRect(Math.min(x1, x2), height - barHeight, Math.max(2, Math.abs(x2 - x1)), barHeight);
      ctx.globalAlpha = 1;
    }

    if (model.showWinrate) {
      strokeSegments(ctx, model.points, turnToX, (point) => {
        const winrate = displayedWinrate(point, model.perspective, model.selectedToPlay);
        return winrate == null ? null : height - winrate * height;
      }, "#60a5fa");
    }
    if (model.showScore) {
      strokeSegments(ctx, model.points, turnToX, (point) => {
        const score = displayedScore(point, model.perspective, model.selectedToPlay);
        if (score == null) return null;
        return height / 2 - (score / model.scoreScale) * (height / 2);
      }, "#34d399");
    }

    const markerX = turnToX(model.currentMove);
    const current = model.points.find((point) => point.moveNumber === model.currentMove);
    const markerWinrate = current
      ? displayedWinrate(current, model.perspective, model.selectedToPlay)
      : null;
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
  }, [model]);

  function pointAt(clientX: number): ChartPoint | null {
    const canvas = canvasRef.current;
    if (!canvas || model.points.length === 0) return null;
    const width = canvas.clientWidth || 240;
    const maxMove = Math.max(model.currentMove, model.points.at(-1)?.moveNumber ?? 0, 1);
    const ratio = Math.min(1, Math.max(0, clientX / width));
    const moveNumber = Math.round(ratio * maxMove);
    return model.points.find((point) => point.moveNumber === moveNumber)
      ?? model.points.reduce((closest, point) => (
        Math.abs(point.moveNumber - moveNumber) < Math.abs(closest.moveNumber - moveNumber) ? point : closest
      ));
  }

  function onMove(event: MouseEvent<HTMLCanvasElement>) {
    if (!model.hoverEnabled) return;
    const rect = event.currentTarget.getBoundingClientRect();
    const point = pointAt(event.clientX - rect.left);
    if (!point) {
      setHover(null);
      return;
    }
    const parts = [`第 ${point.moveNumber} 手`];
    const winrate = displayedWinrate(point, model.perspective, model.selectedToPlay);
    const score = displayedScore(point, model.perspective, model.selectedToPlay);
    if (model.showWinrate && winrate != null) parts.push(`胜率 ${(winrate * 100).toFixed(1)}%`);
    if (model.showScore && score != null) parts.push(`目差 ${score.toFixed(1)}`);
    if (winrate == null && score == null) parts.push("无分析");
    setHover({ moveNumber: point.moveNumber, label: parts.join(" · ") });
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
    >
      <canvas
        ref={canvasRef}
        className="winrate-chart"
        aria-label="胜率走势"
        onMouseMove={model.hoverEnabled ? onMove : undefined}
        onMouseLeave={model.hoverEnabled ? () => setHover(null) : undefined}
      />
      {hover ? <div className="winrate-chart-hover" role="status">{hover.label}</div> : null}
    </div>
  );
}

function strokeSegments(
  ctx: CanvasRenderingContext2D,
  points: ChartPoint[],
  turnToX: (turn: number) => number,
  yFor: (point: ChartPoint) => number | null,
  color: string
) {
  ctx.strokeStyle = color;
  ctx.lineWidth = 2;
  let drawing = false;
  ctx.beginPath();
  for (const point of points) {
    const y = yFor(point);
    if (y == null) {
      if (drawing) {
        ctx.stroke();
        ctx.beginPath();
        drawing = false;
      }
      continue;
    }
    const x = turnToX(point.moveNumber);
    if (!drawing) {
      ctx.moveTo(x, y);
      drawing = true;
    } else {
      ctx.lineTo(x, y);
    }
  }
  if (drawing) ctx.stroke();
}
