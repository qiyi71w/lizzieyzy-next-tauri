import type { CandidateMoveDto, PositionDto } from "./types";
import { candidateDisplayScore, type GranularDisplay, type CandidateTextField } from "./granularDisplay";
import { t } from "../i18n/resources";

export function drawOwnership(ctx: CanvasRenderingContext2D, ownership: number[], position: PositionDto, preferences: GranularDisplay, grid: number, coordX: (x: number) => number, coordY: (y: number) => number) {
  const style = preferences.ownershipStyle;
  for (let y = 0; y < position.board_height; y++) for (let x = 0; x < position.board_width; x++) {
    const value = ownership[y * position.board_width + x];
    if (!Number.isFinite(value) || value === 0 || Math.abs(value) > 1) continue;
    const magnitude = Math.abs(value);
    const black = value > 0;
    const stone = position.stones.find(stone => stone.x === x && stone.y === y);
    if (style === "notOnLive" && stone?.color === (black ? "black" : "white")) continue;
    if (style === "bySize" && magnitude <= 0.2) continue;
    const width = style === "bigBelow" ? grid : style === "small" ? grid * (black ? 0.25 : 1 / 3)
      : style === "bySize" ? grid * magnitude * 0.6 : grid * 0.5;
    const alpha = style === "bySize" ? 180 / 255 : magnitude * (style === "bigBelow" ? (black ? 105 : 165) / 255 : 1);
    if (black && stone?.color === "black" && style !== "bigBelow" && style !== "bySize") {
      const gray = Math.round(127 - (Math.floor(magnitude * 255) - 1) / 2);
      ctx.fillStyle = `rgb(${gray},${gray},${gray})`;
    } else ctx.fillStyle = black ? `rgba(0,0,0,${alpha})` : `rgba(255,255,255,${alpha})`;
    ctx.fillRect(coordX(x) - width / 2, coordY(y) - width / 2, width, width);
  }
}

export function drawCandidateText(ctx: CanvasRenderingContext2D, candidate: CandidateMoveDto, candidates: CandidateMoveDto[], index: number, selected: boolean, preferences: GranularDisplay, perspective: "black" | "sideToPlay", position: PositionDto, komi: number | undefined, grid: number, x: number, y: number) {
  const flip = perspective === "sideToPlay" && position.to_play === "white";
  const winrate = flip ? 1 - candidate.winrate_black : candidate.winrate_black;
  const score = candidateDisplayScore(candidate, preferences, perspective, position.to_play, komi);
  const enabled: Record<CandidateTextField, boolean> = { winrate: preferences.showWinrate, visits: preferences.showVisits, score: preferences.showScore };
  const values: Record<CandidateTextField, string> = { winrate: `${(winrate * 100).toFixed(1)}%`, visits: String(candidate.visits), score: score === null ? t("display.unavailable") : `${score > 0 ? "+" : ""}${score.toFixed(1)}` };
  const fields = preferences.textOrder.filter(field => enabled[field]);
  ctx.textAlign = "center";
  ctx.textBaseline = "middle";
  const normalColor = selected ? "#fff" : "#1a1d21";
  if (preferences.showOrder) {
    ctx.fillStyle = normalColor;
    ctx.font = `bold ${Math.max(9, grid * 0.23)}px sans-serif`;
    ctx.fillText(String((candidate.order ?? index) + 1), x - grid * 0.32, y - grid * 0.32);
  }
  const maxVisits = Math.max(...candidates.map(move => move.visits));
  const maxWinrate = Math.max(...candidates.map(move => flip ? 1 - move.winrate_black : move.winrate_black));
  const maxScore = Math.max(...candidates.map(move => candidateDisplayScore(move, preferences, perspective, position.to_play, komi) ?? -Infinity));
  const extrema: Record<CandidateTextField, boolean> = { winrate: winrate === maxWinrate, visits: candidate.visits === maxVisits, score: score !== null && score === maxScore };
  fields.forEach((field, row) => {
    const emphasize = preferences.emphasizeMax && !selected && extrema[field];
    ctx.fillStyle = emphasize ? (index === 0 ? "#163f96" : candidate.visits / Math.max(maxVisits, 1) > 0.375 ? "#b42318" : "#00685c") : normalColor;
    ctx.font = `${emphasize ? "bold " : ""}${Math.max(8, grid * 0.23)}px sans-serif`;
    ctx.fillText(values[field], x, y + (row - (fields.length - 1) / 2) * grid * 0.25, grid * 0.86);
  });
}
