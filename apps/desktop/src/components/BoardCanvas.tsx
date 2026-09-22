import { useEffect, useMemo, useRef, useState, type CSSProperties, type KeyboardEvent, type PointerEvent } from "react";
import { createPortal } from "react-dom";
import type { AnalysisFrameDto, MoveDto, PointDto, PositionDto, SgfMarkupDto } from "../domain/types";
import { isPoint } from "../domain/board";
import type { NextMoveReviewMarker, NextMoveReviewMarkerMode } from "../domain/nextMoveReviewMarker";
import { variationReplayPointSteps } from "../domain/variationReplay";
import {
  shouldPublishReviewPresentation,
  type ReviewPresentationScope
} from "../domain/reviewPresentation";

export type OverlayMode = "candidates" | "ownership" | "policy";

type Props = {
  position: PositionDto;
  markup?: SgfMarkupDto[];
  analysis?: AnalysisFrameDto;
  selectedCandidateIndex?: number | null;
  stoneMoveNumbers?: MoveDto[];
  showCoordinates?: boolean;
  showMoveNumbers?: boolean;
  overlayMode?: OverlayMode;
  onOverlayModeChange?: (mode: OverlayMode) => void;
  hideCandidates?: boolean;
  onPointClick?: (point: PointDto) => void;
  keyboardPlacement?: boolean;
  onCandidatePreview?: (index: number | null) => void;
  previewScope?: ReviewPresentationScope;
  nextMoveMode?: NextMoveReviewMarkerMode;
  nextMoveMarkers?: NextMoveReviewMarker[];
  pvPrefixLength?: number;
  replayCandidateIndex?: number | null;
};
type PolicyPoint = { x: number; y: number; value: number };

function samePreviewScope(
  current: ReviewPresentationScope | undefined,
  scheduled: ReviewPresentationScope | undefined
): boolean {
  if (!current || !scheduled) return current === scheduled;
  return shouldPublishReviewPresentation(current, scheduled);
}

export function BoardCanvas({
  position,
  markup = [],
  analysis,
  selectedCandidateIndex,
  stoneMoveNumbers = [],
  showCoordinates = true,
  showMoveNumbers = false,
  overlayMode: overlayModeProp,
  onOverlayModeChange,
  hideCandidates = false,
  onPointClick,
  keyboardPlacement = false,
  onCandidatePreview,
  previewScope,
  nextMoveMode = "off",
  nextMoveMarkers = [],
  pvPrefixLength,
  replayCandidateIndex
}: Props) {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const [keyboardPoint, setKeyboardPoint] = useState<PointDto | null>(null);
  const [overlayModeLocal, setOverlayModeLocal] = useState<OverlayMode>("candidates");
  const previewTimerRef = useRef<number | undefined>(undefined);
  const hoveredCandidateIndexRef = useRef<number | null>(null);
  const publishedCandidateIndexRef = useRef<number | null>(null);
  const onCandidatePreviewRef = useRef(onCandidatePreview);
  const previewScopeRef = useRef(previewScope);
  onCandidatePreviewRef.current = onCandidatePreview;
  previewScopeRef.current = previewScope;
  const previewScopeKey = previewScope
    ? JSON.stringify([previewScope.generation, previewScope.requestToken, previewScope.selectedPath])
    : "";
  const overlayMode = overlayModeProp ?? overlayModeLocal;
  function setOverlayMode(mode: OverlayMode) {
    onOverlayModeChange?.(mode);
    if (overlayModeProp === undefined) setOverlayModeLocal(mode);
  }
  const boardPointCount = position.board_width * position.board_height;
  const hasOwnership = (analysis?.ownership?.length ?? 0) >= boardPointCount;
  const policyPoints = useMemo(
    () => getTopPolicyPoints(analysis?.policy, position.board_width, position.board_height, 12),
    [analysis?.policy, position.board_width, position.board_height]
  );
  const hasPolicy = policyPoints.length > 0;
  const effectiveOverlayMode = overlayMode === "ownership" && !hasOwnership ? "candidates" : overlayMode === "policy" && !hasPolicy ? "candidates" : overlayMode;

  function cancelCandidatePreview() {
    if (previewTimerRef.current !== undefined) {
      clearTimeout(previewTimerRef.current);
      previewTimerRef.current = undefined;
    }
    hoveredCandidateIndexRef.current = null;
    if (publishedCandidateIndexRef.current !== null) {
      publishedCandidateIndexRef.current = null;
      onCandidatePreviewRef.current?.(null);
    }
  }

  function handleCandidatePointerMove(event: PointerEvent<HTMLCanvasElement>) {
    if (effectiveOverlayMode !== "candidates" || hideCandidates) {
      cancelCandidatePreview();
      return;
    }
    const rect = event.currentTarget.getBoundingClientRect();
    const padding = Math.min(rect.width, rect.height) * 0.09;
    const grid = Math.min(
      (rect.width - padding * 2) / Math.max(position.board_width - 1, 1),
      (rect.height - padding * 2) / Math.max(position.board_height - 1, 1)
    );
    const offsetX = (rect.width - grid * (position.board_width - 1)) / 2;
    const offsetY = (rect.height - grid * (position.board_height - 1)) / 2;
    const pointerX = event.clientX - rect.left;
    const pointerY = event.clientY - rect.top;
    const candidateIndex = (analysis?.candidates ?? []).findIndex((candidate) => {
      if (!isPoint(candidate.vertex)) return false;
      const centerX = offsetX + candidate.vertex.point.x * grid;
      const centerY = offsetY + candidate.vertex.point.y * grid;
      const radius = grid * (0.18 + Math.min(candidate.visits / Math.max(analysis?.visits ?? 1, 1), 1) * 0.22);
      return Math.hypot(pointerX - centerX, pointerY - centerY) <= radius;
    });
    if (candidateIndex < 0) {
      cancelCandidatePreview();
      return;
    }
    if (hoveredCandidateIndexRef.current === candidateIndex) return;

    cancelCandidatePreview();
    hoveredCandidateIndexRef.current = candidateIndex;
    const scheduledPreviewScope = previewScope;
    previewTimerRef.current = setTimeout(() => {
      previewTimerRef.current = undefined;
      if (!samePreviewScope(previewScopeRef.current, scheduledPreviewScope)) {
        hoveredCandidateIndexRef.current = null;
        return;
      }
      publishedCandidateIndexRef.current = candidateIndex;
      onCandidatePreviewRef.current?.(candidateIndex);
    }, 120);
  }

  useEffect(() => {
    cancelCandidatePreview();
  }, [analysis, position, selectedCandidateIndex, effectiveOverlayMode, hideCandidates, previewScopeKey]);

  useEffect(() => () => {
    clearTimeout(previewTimerRef.current);
  }, []);
  useEffect(() => {
    setKeyboardPoint((current) => {
      if (!current) return null;
      const next = {
        x: Math.min(current.x, Math.max(position.board_width - 1, 0)),
        y: Math.min(current.y, Math.max(position.board_height - 1, 0))
      };
      return next.x === current.x && next.y === current.y ? current : next;
    });
  }, [position.board_width, position.board_height]);

  useEffect(() => {
    if (!keyboardPlacement) {
      setKeyboardPoint(null);
      return;
    }
    const center = { x: Math.floor(position.board_width / 2), y: Math.floor(position.board_height / 2) };
    setKeyboardPoint((current) => current ?? center);
    canvasRef.current?.focus();
  }, [keyboardPlacement, position.board_width, position.board_height]);

  function handleKeyDown(event: KeyboardEvent<HTMLCanvasElement>) {
    if (!keyboardPlacement) return;
    if (event.nativeEvent.isComposing || event.key === "Process") return;
    const modified = event.altKey || event.ctrlKey || event.metaKey || event.shiftKey;
    if (modified) return;
    const isEnter = event.key === "Enter";
    const isArrow = event.key === "ArrowLeft" ||
      event.key === "ArrowRight" ||
      event.key === "ArrowUp" ||
      event.key === "ArrowDown";
    if (!isEnter && !isArrow) return;

    event.preventDefault();
    event.stopPropagation();

    const center = { x: Math.floor(position.board_width / 2), y: Math.floor(position.board_height / 2) };
    const current = keyboardPoint ?? center;
    if (isEnter) {
      cancelCandidatePreview();
      onPointClick?.(current);
      return;
    }

    const xDelta = event.key === "ArrowLeft" ? -1 : event.key === "ArrowRight" ? 1 : 0;
    const yDelta = event.key === "ArrowUp" ? -1 : event.key === "ArrowDown" ? 1 : 0;
    setKeyboardPoint({
      x: Math.max(0, Math.min(position.board_width - 1, current.x + xDelta)),
      y: Math.max(0, Math.min(position.board_height - 1, current.y + yDelta))
    });
  }

  useEffect(() => {
    const canvas = canvasRef.current;
    const ctx = canvas?.getContext("2d");
    if (!canvas || !ctx) return;
    const dpr = window.devicePixelRatio || 1;
    const cssWidth = canvas.clientWidth || 720;
    const cssHeight = canvas.clientHeight || 720;
    canvas.width = Math.floor(cssWidth * dpr);
    canvas.height = Math.floor(cssHeight * dpr);
    ctx.scale(dpr, dpr);
    ctx.clearRect(0, 0, cssWidth, cssHeight);

    const boardWidth = position.board_width;
    const boardHeight = position.board_height;
    const padding = Math.min(cssWidth, cssHeight) * 0.09;
    const grid = Math.min(
      (cssWidth - padding * 2) / Math.max(boardWidth - 1, 1),
      (cssHeight - padding * 2) / Math.max(boardHeight - 1, 1)
    );
    const offsetX = (cssWidth - grid * (boardWidth - 1)) / 2;
    const offsetY = (cssHeight - grid * (boardHeight - 1)) / 2;
    const coordX = (n: number) => offsetX + n * grid;
    const coordY = (n: number) => offsetY + n * grid;

    ctx.fillStyle = "#e4c27a";
    ctx.fillRect(0, 0, cssWidth, cssHeight);
    ctx.strokeStyle = "rgba(74,53,24,.85)";
    ctx.lineWidth = 1;
    for (let y = 0; y < boardHeight; y += 1) {
      ctx.beginPath(); ctx.moveTo(coordX(0), coordY(y)); ctx.lineTo(coordX(boardWidth - 1), coordY(y)); ctx.stroke();
    }
    for (let x = 0; x < boardWidth; x += 1) {
      ctx.beginPath(); ctx.moveTo(coordX(x), coordY(0)); ctx.lineTo(coordX(x), coordY(boardHeight - 1)); ctx.stroke();
    }

    ctx.fillStyle = "rgba(42,28,14,.88)";
    for (const x of starCoordinates(boardWidth)) for (const y of starCoordinates(boardHeight)) {
      ctx.beginPath(); ctx.arc(coordX(x), coordY(y), Math.max(2, grid * 0.08), 0, Math.PI * 2); ctx.fill();
    }

    if (showCoordinates) {
      const letters = "ABCDEFGHJKLMNOPQRSTUVWXYZ";
      ctx.fillStyle = "#4a3518";
      ctx.font = `${Math.max(9, grid * 0.28)}px "Noto Sans SC", sans-serif`;
      ctx.textAlign = "center";
      ctx.textBaseline = "middle";
      for (let x = 0; x < boardWidth; x += 1) {
        const label = letters[x] ?? String(x + 1);
        ctx.fillText(label, coordX(x), padding * 0.38);
        ctx.fillText(label, coordX(x), cssHeight - padding * 0.38);
      }
      for (let y = 0; y < boardHeight; y += 1) {
        const label = String(boardHeight - y);
        ctx.fillText(label, padding * 0.38, coordY(y));
        ctx.fillText(label, cssWidth - padding * 0.38, coordY(y));
      }
    }

    if (effectiveOverlayMode === "ownership" && hasOwnership && analysis?.ownership) {
      const cellWidth = Math.max(2, grid * 0.94);
      const cellHeight = cellWidth;
      for (let y = 0; y < boardHeight; y += 1) {
        for (let x = 0; x < boardWidth; x += 1) {
          const value = normalizeOwnershipValue(analysis.ownership[y * boardWidth + x]);
          const magnitude = Math.abs(value);
          if (magnitude < 0.015) continue;
          const alpha = 0.1 + magnitude * 0.38;
          ctx.fillStyle = value >= 0 ? `rgba(33,86,199,${alpha})` : `rgba(194,65,12,${alpha})`;
          ctx.fillRect(coordX(x) - cellWidth / 2, coordY(y) - cellHeight / 2, cellWidth, cellHeight);
        }
      }
    }

    const moveByPoint = new Map<string, number>();
    if (showMoveNumbers) {
      for (const move of stoneMoveNumbers) {
        if (!isPoint(move.vertex)) continue;
        moveByPoint.set(`${move.vertex.point.x}:${move.vertex.point.y}`, move.move_number);
      }
    }
    for (const stone of position.stones) {
      const cx = coordX(stone.x); const cy = coordY(stone.y); const radius = grid * 0.45;
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
      ctx.fillStyle = "#2156c7";
      ctx.beginPath(); ctx.arc(coordX(x), coordY(y), Math.max(2.5, grid * 0.12), 0, Math.PI * 2); ctx.fill();
    }

    if (effectiveOverlayMode === "policy" && hasPolicy) {
      drawPolicyOverlay(ctx, policyPoints, coordX, coordY, grid);
    } else if (!hideCandidates) {
      const topCandidates = analysis?.candidates ?? [];
      const replayCandidate = topCandidates[replayCandidateIndex ?? selectedCandidateIndex ?? 0] ?? topCandidates[0];
      const replaySteps = pvPrefixLength == null ? [] : variationReplayPointSteps(replayCandidate).slice(0, pvPrefixLength);
      if (replaySteps.length === 0) {
        for (const [index, candidate] of topCandidates.entries()) {
          if (!isPoint(candidate.vertex)) continue;
          const cx = coordX(candidate.vertex.point.x); const cy = coordY(candidate.vertex.point.y);
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
      } else {
        let turnColor: "black" | "white" = position.to_play;
        for (const [index, point] of replaySteps.entries()) {
          if (index > 0) turnColor = turnColor === "black" ? "white" : "black";
          const cx = coordX(point.x);
          const cy = coordY(point.y);
          const radius = grid * 0.45;
          ctx.beginPath(); ctx.arc(cx, cy, radius, 0, Math.PI * 2);
          ctx.fillStyle = turnColor === "black" ? "#1a1a1e" : "#ffffff";
          ctx.fill();
          ctx.strokeStyle = index === 0 ? "#2563eb" : "rgba(42,28,14,0.8)";
          ctx.lineWidth = index === 0 ? 1.5 : 1;
          ctx.stroke();
          ctx.fillStyle = turnColor === "black" ? "#ffffff" : "#1a1a1e";
          ctx.font = `bold ${Math.max(9, grid * 0.4)}px "Noto Sans SC", sans-serif`;
          ctx.textAlign = "center";
          ctx.textBaseline = "middle";
          ctx.fillText(String(index + 1), cx, cy);
        }
      }
    }

    for (const marker of nextMoveMode === "off" ? [] : nextMoveMarkers) {
      const cx = coordX(marker.point.x);
      const cy = coordY(marker.point.y);
      const radius = grid * (marker.primary ? 0.22 : 0.16);
      ctx.beginPath();
      ctx.arc(cx, cy, radius, 0, Math.PI * 2);
      ctx.fillStyle = markerFill(marker);
      ctx.fill();
      ctx.strokeStyle = marker.primary ? "#163f96" : "rgba(33,86,199,.55)";
      ctx.lineWidth = marker.primary ? Math.max(2, grid * 0.08) : 1;
      ctx.stroke();
    }

    for (const mark of markup) {
      const cx = coordX(mark.point.x);
      const cy = coordY(mark.point.y);
      const radius = grid * 0.25;
      const stone = position.stones.find((entry) => entry.x === mark.point.x && entry.y === mark.point.y);
      ctx.strokeStyle = stone?.color === "black" ? "#ffffff" : "#161616";
      ctx.fillStyle = ctx.strokeStyle;
      ctx.lineWidth = Math.max(1.5, grid * 0.06);
      ctx.beginPath();
      if (mark.kind === "label") {
        ctx.font = `${Math.max(9, grid * 0.4)}px sans-serif`;
        ctx.textAlign = "center";
        ctx.textBaseline = "middle";
        ctx.fillText(mark.text, cx, cy, grid * 0.85);
      } else if (mark.kind === "circle") {
        ctx.arc(cx, cy, radius, 0, Math.PI * 2);
      } else if (mark.kind === "square") {
        ctx.rect(cx - radius, cy - radius, radius * 2, radius * 2);
      } else if (mark.kind === "triangle") {
        ctx.moveTo(cx, cy - radius);
        ctx.lineTo(cx + radius, cy + radius);
        ctx.lineTo(cx - radius, cy + radius);
        ctx.lineTo(cx, cy - radius);
      } else {
        ctx.moveTo(cx - radius, cy - radius);
        ctx.lineTo(cx + radius, cy + radius);
        ctx.moveTo(cx + radius, cy - radius);
        ctx.lineTo(cx - radius, cy + radius);
      }
      ctx.stroke();
    }

    if (keyboardPoint) {
      const cx = coordX(keyboardPoint.x);
      const cy = coordY(keyboardPoint.y);
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
  }, [position, markup, analysis, selectedCandidateIndex, effectiveOverlayMode, hasOwnership, hasPolicy, policyPoints, stoneMoveNumbers, showCoordinates, showMoveNumbers, hideCandidates, keyboardPoint, nextMoveMode, nextMoveMarkers, pvPrefixLength, replayCandidateIndex]);

  const layerHost = document.getElementById("board-layers");
  const overlays = (
    <div className="board-overlays" aria-label="棋盘图层">
      <OverlayButton label="候选" active={effectiveOverlayMode === "candidates"} onClick={() => setOverlayMode("candidates")} />
      <OverlayButton label="领地" active={effectiveOverlayMode === "ownership"} disabled={!hasOwnership} onClick={() => setOverlayMode("ownership")} />
      <OverlayButton label="策略" active={effectiveOverlayMode === "policy"} disabled={!hasPolicy} onClick={() => setOverlayMode("policy")} />
    </div>
  );

  const boardStyle = {
    "--board-ratio": position.board_width / position.board_height,
    aspectRatio: `${position.board_width} / ${position.board_height}`
  } as CSSProperties;
  return <div
    className="board-canvas"
    style={boardStyle}
    data-next-move-mode={nextMoveMode}
    data-next-move-markers={JSON.stringify(nextMoveMarkers)}
  >
    <canvas
      ref={canvasRef}
      aria-label="棋盘"
      role="application"
      tabIndex={0}
      onKeyDown={handleKeyDown}
      onPointerMove={handleCandidatePointerMove}
      onPointerLeave={cancelCandidatePreview}
      onClick={(event) => {
        cancelCandidatePreview();
        if (!onPointClick) return;
        const rect = event.currentTarget.getBoundingClientRect();
        const padding = Math.min(rect.width, rect.height) * 0.09;
        const grid = Math.min(
          (rect.width - padding * 2) / Math.max(position.board_width - 1, 1),
          (rect.height - padding * 2) / Math.max(position.board_height - 1, 1)
        );
        const offsetX = (rect.width - grid * (position.board_width - 1)) / 2;
        const offsetY = (rect.height - grid * (position.board_height - 1)) / 2;
        const x = Math.round((event.clientX - rect.left - offsetX) / grid);
        const y = Math.round((event.clientY - rect.top - offsetY) / grid);
        if (x < 0 || y < 0 || x >= position.board_width || y >= position.board_height) return;
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

function getTopPolicyPoints(policy: number[] | null | undefined, boardWidth: number, boardHeight: number, limit: number): PolicyPoint[] {
  const pointCount = boardWidth * boardHeight;
  if (!policy || policy.length < pointCount) return [];
  const points: PolicyPoint[] = [];
  for (let index = 0; index < pointCount; index += 1) {
    const value = policy[index];
    if (!Number.isFinite(value) || value <= 0) continue;
    points.push({ x: index % boardWidth, y: Math.floor(index / boardWidth), value });
  }
  return points.sort((a, b) => b.value - a.value).slice(0, limit);
}

function drawPolicyOverlay(ctx: CanvasRenderingContext2D, points: PolicyPoint[], coordX: (n: number) => number, coordY: (n: number) => number, grid: number) {
  const maxPolicy = Math.max(points[0]?.value ?? 1, 1e-6);
  for (const [rank, point] of points.entries()) {
    const weight = Math.sqrt(point.value / maxPolicy);
    const cx = coordX(point.x);
    const cy = coordY(point.y);
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

function starCoordinates(length: number): number[] {
  if (length === 19) return [3, 9, 15];
  if (length === 13) return [3, 6, 9];
  if (length === 9) return [2, 4, 6];
  if (length < 7) return [];
  return [3, length - 4].filter((value, index, values) => value >= 0 && value < length && values.indexOf(value) === index);
}

function markerFill(marker: NextMoveReviewMarker): string {
  if (marker.rank) return NEXT_MOVE_RANK_FILL[marker.rank];
  return marker.primary ? "rgba(33,86,199,.22)" : "rgba(33,86,199,.10)";
}

const NEXT_MOVE_RANK_FILL = {
  best: "#1b7f3a",
  good: "#5aa862",
  normal: "#c7a21a",
  inaccuracy: "#d67a12",
  mistake: "#c4451c",
  blunder: "#8e1515"
} as const;
