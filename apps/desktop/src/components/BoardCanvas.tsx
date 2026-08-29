import { useEffect, useMemo, useRef, useState, type KeyboardEvent } from "react";
import { createPortal } from "react-dom";
import type { AnalysisFrameDto, MoveDto, PointDto, PositionDto } from "../domain/types";
import { isPoint } from "../domain/board";

export type OverlayMode = "candidates" | "ownership" | "policy";

type Props = {
  position: PositionDto;
  analysis?: AnalysisFrameDto;
  selectedCandidateIndex?: number | null;
  moves?: MoveDto[];
  showCoordinates?: boolean;
  showMoveNumbers?: boolean;
  overlayMode?: OverlayMode;
  onOverlayModeChange?: (mode: OverlayMode) => void;
  hideCandidates?: boolean;
  onPointClick?: (point: PointDto) => void;
};
type PolicyPoint = { x: number; y: number; value: number };

export function BoardCanvas({
  position,
  analysis,
  selectedCandidateIndex,
  moves = [],
  showCoordinates = true,
  showMoveNumbers = false,
  overlayMode: overlayModeProp,
  onOverlayModeChange,
  hideCandidates = false,
  onPointClick
}: Props) {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const [keyboardPoint, setKeyboardPoint] = useState<PointDto | null>(null);
  const [overlayModeLocal, setOverlayModeLocal] = useState<OverlayMode>("candidates");
  const overlayMode = overlayModeProp ?? overlayModeLocal;
  function setOverlayMode(mode: OverlayMode) {
    onOverlayModeChange?.(mode);
    if (overlayModeProp === undefined) setOverlayModeLocal(mode);
  }
  const boardPointCount = position.board_size * position.board_size;
  const hasOwnership = (analysis?.ownership?.length ?? 0) >= boardPointCount;
  const policyPoints = useMemo(() => getTopPolicyPoints(analysis?.policy, position.board_size, 12), [analysis?.policy, position.board_size]);
  const hasPolicy = policyPoints.length > 0;
  const effectiveOverlayMode = overlayMode === "ownership" && !hasOwnership ? "candidates" : overlayMode === "policy" && !hasPolicy ? "candidates" : overlayMode;

  useEffect(() => {
    setKeyboardPoint((current) => {
      if (!current) return null;
      const lastCoordinate = Math.max(position.board_size - 1, 0);
      const next = {
        x: Math.min(current.x, lastCoordinate),
        y: Math.min(current.y, lastCoordinate)
      };
      return next.x === current.x && next.y === current.y ? current : next;
    });
  }, [position.board_size]);

  function handleKeyDown(event: KeyboardEvent<HTMLCanvasElement>) {
    const isSubmit = event.key === "Enter" || event.key === " ";
    const isArrow = event.key === "ArrowLeft" ||
      event.key === "ArrowRight" ||
      event.key === "ArrowUp" ||
      event.key === "ArrowDown";
    if (!isSubmit && !isArrow) return;

    event.preventDefault();
    event.stopPropagation();

    const center = Math.floor(position.board_size / 2);
    const current = keyboardPoint ?? { x: center, y: center };
    if (isSubmit) {
      onPointClick?.(current);
      return;
    }

    const lastCoordinate = Math.max(position.board_size - 1, 0);
    const xDelta = event.key === "ArrowLeft" ? -1 : event.key === "ArrowRight" ? 1 : 0;
    const yDelta = event.key === "ArrowUp" ? -1 : event.key === "ArrowDown" ? 1 : 0;
    setKeyboardPoint({
      x: Math.max(0, Math.min(lastCoordinate, current.x + xDelta)),
      y: Math.max(0, Math.min(lastCoordinate, current.y + yDelta))
    });
  }

  useEffect(() => {
    const canvas = canvasRef.current;
    const ctx = canvas?.getContext("2d");
    if (!canvas || !ctx) return;
    const dpr = window.devicePixelRatio || 1;
    const cssSize = Math.min(canvas.clientWidth || 720, canvas.clientHeight || 720);
    canvas.width = Math.floor(cssSize * dpr);
    canvas.height = Math.floor(cssSize * dpr);
    ctx.scale(dpr, dpr);
    ctx.clearRect(0, 0, cssSize, cssSize);

    const boardSize = position.board_size;
    const padding = cssSize * 0.09;
    const grid = (cssSize - padding * 2) / (boardSize - 1);
    const coord = (n: number) => padding + n * grid;

    ctx.fillStyle = "#e4c27a";
    ctx.fillRect(0, 0, cssSize, cssSize);
    ctx.strokeStyle = "rgba(74,53,24,.85)";
    ctx.lineWidth = 1;
    for (let i = 0; i < boardSize; i += 1) {
      ctx.beginPath(); ctx.moveTo(coord(0), coord(i)); ctx.lineTo(coord(boardSize - 1), coord(i)); ctx.stroke();
      ctx.beginPath(); ctx.moveTo(coord(i), coord(0)); ctx.lineTo(coord(i), coord(boardSize - 1)); ctx.stroke();
    }

    const stars = boardSize === 19 ? [3, 9, 15] : boardSize === 13 ? [3, 6, 9] : [2, boardSize - 3];
    ctx.fillStyle = "rgba(42,28,14,.88)";
    for (const x of stars) for (const y of stars) { ctx.beginPath(); ctx.arc(coord(x), coord(y), Math.max(2, grid * 0.08), 0, Math.PI * 2); ctx.fill(); }

    if (showCoordinates) {
      const letters = "ABCDEFGHJKLMNOPQRST";
      ctx.fillStyle = "#4a3518";
      ctx.font = `${Math.max(9, grid * 0.28)}px "Noto Sans SC", sans-serif`;
      ctx.textAlign = "center";
      ctx.textBaseline = "middle";
      for (let i = 0; i < boardSize; i += 1) {
        const label = letters[i] ?? String(i + 1);
        ctx.fillText(label, coord(i), padding * 0.38);
        ctx.fillText(label, coord(i), cssSize - padding * 0.38);
        ctx.fillText(String(boardSize - i), padding * 0.38, coord(i));
        ctx.fillText(String(boardSize - i), cssSize - padding * 0.38, coord(i));
      }
    }

    if (effectiveOverlayMode === "ownership" && hasOwnership && analysis?.ownership) {
      const cellSize = Math.max(2, grid * 0.94);
      for (let y = 0; y < boardSize; y += 1) {
        for (let x = 0; x < boardSize; x += 1) {
          const value = normalizeOwnershipValue(analysis.ownership[y * boardSize + x]);
          const magnitude = Math.abs(value);
          if (magnitude < 0.015) continue;
          const alpha = 0.1 + magnitude * 0.38;
          ctx.fillStyle = value >= 0 ? `rgba(33,86,199,${alpha})` : `rgba(194,65,12,${alpha})`;
          ctx.fillRect(coord(x) - cellSize / 2, coord(y) - cellSize / 2, cellSize, cellSize);
        }
      }
    }

    const moveByPoint = new Map<string, number>();
    if (showMoveNumbers) {
      for (const move of moves) {
        if (move.move_number > position.move_number || !isPoint(move.vertex)) continue;
        moveByPoint.set(`${move.vertex.point.x}:${move.vertex.point.y}`, move.move_number);
      }
    }
    for (const stone of position.stones) {
      const cx = coord(stone.x); const cy = coord(stone.y); const radius = grid * 0.45;
      ctx.beginPath(); ctx.arc(cx, cy, radius, 0, Math.PI * 2);
      ctx.fillStyle = stone.color === "black" ? "#161616" : "#f7f4ee"; ctx.fill();
      ctx.strokeStyle = "#1c1915";
      ctx.lineWidth = 1;
      ctx.stroke();
      const moveNumber = moveByPoint.get(`${stone.x}:${stone.y}`);
      if (moveNumber !== undefined) {
        ctx.fillStyle = stone.color === "black" ? "#f7f4ee" : "#161616";
        ctx.font = `${Math.max(9, grid * 0.32)}px "Noto Sans SC", sans-serif`;
        ctx.textAlign = "center";
        ctx.textBaseline = "middle";
        ctx.fillText(String(moveNumber), cx, cy);
      }
    }

    if (position.last_move && isPoint(position.last_move.vertex)) {
      const { x, y } = position.last_move.vertex.point;
      const cx = coord(x); const cy = coord(y);
      ctx.fillStyle = "#2156c7";
      ctx.beginPath(); ctx.arc(cx, cy, Math.max(2.5, grid * 0.12), 0, Math.PI * 2); ctx.fill();
    }

    if (effectiveOverlayMode === "policy" && hasPolicy) {
      drawPolicyOverlay(ctx, policyPoints, boardSize, coord, grid);
    } else if (!hideCandidates) {
      const topCandidates = analysis?.candidates.slice(0, 8) ?? [];
      for (const [index, candidate] of topCandidates.entries()) {
        if (!isPoint(candidate.vertex)) continue;
        const cx = coord(candidate.vertex.point.x); const cy = coord(candidate.vertex.point.y);
        const radius = grid * (0.18 + Math.min(candidate.visits / Math.max(analysis?.visits ?? 1, 1), 1) * 0.22);
        const isSelected = selectedCandidateIndex === index;
        ctx.beginPath(); ctx.arc(cx, cy, radius, 0, Math.PI * 2);
        ctx.fillStyle = isSelected ? "#2156c7" : "rgba(255,255,255,.88)";
        ctx.fill();
        ctx.strokeStyle = isSelected ? "#163f96" : "rgba(42,28,14,.55)";
        ctx.lineWidth = isSelected ? Math.max(1.5, grid * 0.06) : 1;
        ctx.stroke();
        ctx.fillStyle = isSelected ? "#fff" : "#1a1d21";
        ctx.font = `${Math.max(10, grid * 0.28)}px "Noto Sans SC", sans-serif`;
        ctx.textAlign = "center";
        ctx.textBaseline = "middle";
        ctx.fillText(String(index + 1), cx, cy);
      }
    }

    if (keyboardPoint) {
      const cx = coord(keyboardPoint.x);
      const cy = coord(keyboardPoint.y);
      const radius = grid * 0.48;
      ctx.beginPath();
      ctx.arc(cx, cy, radius, 0, Math.PI * 2);
      ctx.strokeStyle = "#ffffff";
      ctx.lineWidth = Math.max(3, grid * 0.12);
      ctx.stroke();
      ctx.beginPath();
      ctx.arc(cx, cy, radius, 0, Math.PI * 2);
      ctx.strokeStyle = "#2156c7";
      ctx.lineWidth = Math.max(1.5, grid * 0.06);
      ctx.stroke();
    }
  }, [position, analysis, selectedCandidateIndex, effectiveOverlayMode, hasOwnership, hasPolicy, policyPoints, moves, showCoordinates, showMoveNumbers, hideCandidates, keyboardPoint]);

  const layerHost = document.getElementById("board-layers");
  const overlays = (
    <div className="board-overlays" aria-label="棋盘图层">
      <OverlayButton label="候选" active={effectiveOverlayMode === "candidates"} onClick={() => setOverlayMode("candidates")} />
      <OverlayButton label="领地" active={effectiveOverlayMode === "ownership"} disabled={!hasOwnership} onClick={() => setOverlayMode("ownership")} />
      <OverlayButton label="策略" active={effectiveOverlayMode === "policy"} disabled={!hasPolicy} onClick={() => setOverlayMode("policy")} />
    </div>
  );

  return <div className="board-canvas">
    <canvas
      ref={canvasRef}
      aria-label="棋盘"
      role="application"
      tabIndex={0}
      onFocus={() => {
        const center = Math.floor(position.board_size / 2);
        setKeyboardPoint((current) => current ?? { x: center, y: center });
      }}
      onKeyDown={handleKeyDown}
      onClick={(event) => {
        if (!onPointClick) return;
        const rect = event.currentTarget.getBoundingClientRect();
        const cssSize = Math.min(rect.width, rect.height);
        const boardSize = position.board_size;
        const padding = cssSize * 0.09;
        const grid = (cssSize - padding * 2) / Math.max(boardSize - 1, 1);
        const x = Math.round((event.clientX - rect.left - padding) / grid);
        const y = Math.round((event.clientY - rect.top - padding) / grid);
        if (x < 0 || y < 0 || x >= boardSize || y >= boardSize) return;
        onPointClick({ x, y });
      }}
    />
    {layerHost ? createPortal(overlays, layerHost) : overlays}
  </div>;
}

function OverlayButton({ label, active, disabled, onClick }: { label: string; active: boolean; disabled?: boolean; onClick: () => void }) {
  return <button type="button" disabled={disabled} aria-pressed={active} onClick={onClick}>{label}</button>;
}

function normalizeOwnershipValue(value: number | undefined): number {
  if (typeof value !== "number" || !Number.isFinite(value)) return 0;
  const normalized = Math.abs(value) > 1 ? value / 100 : value;
  return Math.max(-1, Math.min(1, normalized));
}

function getTopPolicyPoints(policy: number[] | null | undefined, boardSize: number, limit: number): PolicyPoint[] {
  if (!policy || policy.length < boardSize * boardSize) return [];
  const points: PolicyPoint[] = [];
  for (let index = 0; index < boardSize * boardSize; index += 1) {
    const value = policy[index];
    if (!Number.isFinite(value) || value <= 0) continue;
    points.push({ x: index % boardSize, y: Math.floor(index / boardSize), value });
  }
  return points.sort((a, b) => b.value - a.value).slice(0, limit);
}

function drawPolicyOverlay(ctx: CanvasRenderingContext2D, points: PolicyPoint[], boardSize: number, coord: (n: number) => number, grid: number) {
  const maxPolicy = Math.max(points[0]?.value ?? 1, 1e-6);
  for (const [rank, point] of points.entries()) {
    const weight = Math.sqrt(point.value / maxPolicy);
    const cx = coord(point.x);
    const cy = coord(point.y);
    const radius = grid * (0.12 + weight * 0.32);
    ctx.beginPath();
    ctx.arc(cx, cy, radius, 0, Math.PI * 2);
    ctx.fillStyle = rank === 0 ? "#9a2a1f" : "#efe6d2";
    ctx.fill();
    ctx.strokeStyle = "#1c1915";
    ctx.lineWidth = 1;
    ctx.stroke();
    if (rank < 8) {
      ctx.fillStyle = rank === 0 ? "#efe6d2" : "#1c1915";
      ctx.font = `${Math.max(10, grid * 0.26)}px "Noto Sans SC", sans-serif`;
      ctx.textAlign = "center";
      ctx.textBaseline = "middle";
      ctx.fillText(String(rank + 1), cx, cy);
    }
  }
}
