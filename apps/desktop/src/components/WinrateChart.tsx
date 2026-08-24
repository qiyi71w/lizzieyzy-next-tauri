import { useEffect, useRef } from "react";
import type { AnalysisFrameDto } from "../domain/types";

type Props = { frames: AnalysisFrameDto[]; currentMove: number };
export function WinrateChart({ frames, currentMove }: Props) {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  useEffect(() => {
    const canvas = canvasRef.current;
    const ctx = canvas?.getContext("2d");
    if (!canvas || !ctx) return;

    const dpr = window.devicePixelRatio || 1;
    const width = canvas.clientWidth || 240;
    const height = canvas.clientHeight || 90;
    canvas.width = Math.floor(width * dpr);
    canvas.height = Math.floor(height * dpr);
    ctx.scale(dpr, dpr);
    ctx.clearRect(0, 0, width, height);

    // 1. 深色卡片背景
    ctx.fillStyle = "#16181d";
    ctx.fillRect(0, 0, width, height);

    // 2. 50% 胜率中轴与网格虚线
    ctx.strokeStyle = "#282d37";
    ctx.lineWidth = 1;
    ctx.setLineDash([3, 3]);
    for (let i = 1; i <= 3; i += 1) {
      const y = (height / 4) * i;
      ctx.beginPath(); ctx.moveTo(0, y); ctx.lineTo(width, y); ctx.stroke();
    }
    ctx.setLineDash([]);

    // 3. 绘制胜率折线
    const sortedFrames = [...frames].sort((a, b) => a.turn - b.turn);
    const maxTurn = Math.max(currentMove, ...sortedFrames.map((frame) => frame.turn), 1);
    const turnToX = (turn: number) => (Math.min(Math.max(turn, 0), maxTurn) / maxTurn) * width;

    if (sortedFrames.length > 1) {
      ctx.strokeStyle = "#60a5fa";
      ctx.lineWidth = 2;
      ctx.beginPath();
      sortedFrames.forEach((frame, index) => {
        const x = turnToX(frame.turn);
        const y = height - frame.winrate_black * height;
        if (index === 0) ctx.moveTo(x, y);
        else ctx.lineTo(x, y);
      });
      ctx.stroke();
    }

    // 4. 当前手数标记点与垂线
    const markerX = turnToX(currentMove);
    const curFrame = frames.find((f) => f.turn === currentMove) ?? frames.at(-1);
    const markerY = curFrame ? height - curFrame.winrate_black * height : height * 0.5;

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
  }, [frames, currentMove]);

  return <canvas ref={canvasRef} className="winrate-chart" aria-label="胜率走势" />;
}
