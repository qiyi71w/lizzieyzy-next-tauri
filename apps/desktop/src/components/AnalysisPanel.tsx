import { useEffect, useRef, useState } from "react";
import type { AnalysisFrameDto, CandidateMoveDto, MoveDto, PositionDto, ProblemMarkerDto } from "../domain/types";
import { isPoint, vertexLabel } from "../domain/board";
import { variationReplayPointSteps } from "../domain/variationReplay";

type PolicyPoint = { x: number; y: number; value: number };
type Pane = "commentary" | "reference";
type SubBoardContentMode = "variation" | "raw";

type Props = {
  frame?: AnalysisFrameDto;
  problems: ProblemMarkerDto[];
  moves?: MoveDto[];
  boardSize: number;
  currentMove: number;
  currentPosition?: PositionDto;
  personalComment?: string;
  generatedInformation?: string | null;
  commentEditorEnabled?: boolean;
  onCommitPersonalComment?: (comment: string) => void;
  selectedCandidateIndex: number | null;
  previewCandidateIndex?: number | null;
  onSelectCandidate: (index: number) => void;
  onSelectProblem: (moveNumber: number) => void;
  pane?: Pane;
  contentMode?: SubBoardContentMode;
  pvPrefixLength?: number;
};

const severityLabel: Record<ProblemMarkerDto["severity"], string> = {
  info: "提示",
  inaccuracy: "疑问",
  mistake: "恶手",
  blunder: "败着"
};

export function AnalysisPanel({
  frame,
  problems,
  moves = [],
  boardSize,
  currentMove,
  currentPosition,
  personalComment = "",
  generatedInformation = null,
  commentEditorEnabled = false,
  onCommitPersonalComment,
  selectedCandidateIndex,
  previewCandidateIndex = null,
  onSelectCandidate,
  onSelectProblem,
  pane = "commentary",
  contentMode = "variation",
  pvPrefixLength
}: Props) {
  const [leftTab, setLeftTab] = useState<"commentary" | "problems">("commentary");
  const [commentDraft, setCommentDraft] = useState(personalComment);
  useEffect(() => {
    setCommentDraft(personalComment);
  }, [personalComment]);
  const hasOwnership = (frame?.ownership?.length ?? 0) >= boardSize * boardSize;

  const candidates = frame?.candidates ?? [];
  const activeCandidateIndex = previewCandidateIndex ?? selectedCandidateIndex ?? 0;
  const activeCandidate = candidates[activeCandidateIndex] ?? candidates[0] ?? null;

  if (pane === "reference") {
    return (
      <aside className="analysis-panel reference-panel" aria-label="参考图与选点">
        <section className="variation-tree" aria-label="变化">
          <button type="button" className={currentMove === 0 ? "is-current" : undefined} onClick={() => onSelectProblem(0)}>
            根
          </button>
          {moves.map((move) => {
            const label = isPoint(move.vertex) ? vertexLabel(move.vertex, boardSize) : "虚手";
            return (
              <button
                key={move.move_number}
                type="button"
                className={currentMove === move.move_number ? "is-current" : undefined}
                onClick={() => onSelectProblem(move.move_number)}
              >
                {move.move_number} {move.color === "black" ? "黑" : "白"} {label}
              </button>
            );
          })}
        </section>
        <section className="candidate-table-section" aria-label="候选点">
          <div className="candidate-title-bar">
            <span>候选点</span>
          </div>
          {candidates.length > 0 ? (
            <div className="table-wrapper">
              <table className="candidate-table">
                <thead>
                  <tr>
                    <th>#</th>
                    <th>选点</th>
                    <th>胜率</th>
                    <th>计算量</th>
                    <th>目数</th>
                  </tr>
                </thead>
                <tbody>
                  {candidates.map((candidate, index) => {
                    const isSelected = selectedCandidateIndex === index || (selectedCandidateIndex === null && index === 0);
                    const moveText = vertexLabel(candidate.vertex, boardSize);
                    const winrateText = `${(candidate.winrate_black * 100).toFixed(1)}%`;
                    const scoreText = (candidate.score_mean_black >= 0 ? "+" : "") + candidate.score_mean_black.toFixed(1);
                    return (
                      <tr
                        key={index}
                        className={`cand-row${isSelected ? " is-selected" : ""}`}
                        onClick={() => onSelectCandidate(index)}
                      >
                        <td>
                          <span className={`rank-badge rank-${index + 1}`}>{index + 1}</span>
                        </td>
                        <td className="cand-coord">{moveText}</td>
                        <td className="cand-winrate">{winrateText}</td>
                        <td className="cand-visits">{candidate.visits.toLocaleString()}</td>
                        <td className="cand-score">{scoreText}</td>
                      </tr>
                    );
                  })}
                </tbody>
              </table>
            </div>
          ) : (
            <p className="muted commentary" style={{ padding: "12px 10px", margin: 0, color: "#8b919c" }}>
              暂无候选点。点击“分析当前节点”或“分析第一子主线”。
            </p>
          )}
        </section>
        <section className="subboard-section" aria-label="副棋盘">
          <div className="subboard-card">
            <SubBoardCanvas
              boardSize={boardSize}
              position={currentPosition}
              candidate={activeCandidate}
              contentMode={contentMode}
              pvPrefixLength={pvPrefixLength}
            />
          </div>
        </section>
      </aside>
    );
  }

  // pane === "commentary" (Left HUD Column)
  return (
    <aside className="analysis-panel commentary-panel" aria-label="对局分析与评注">
      {/* 比分与提子概览 */}
      <div className="score-summary-card">
        <div className="player-stat">
          <span className="stone-dot black-dot"></span>
          <span className="player-name">黑棋</span>
          <span className="captures">提子: {currentPosition?.captures_black ?? 0}</span>
        </div>
        <div className="score-lead-box">
          <div className="score-lead-val">
            {frame ? (frame.score_mean_black >= 0 ? `+${frame.score_mean_black.toFixed(1)}` : frame.score_mean_black.toFixed(1)) : "—"}
          </div>
          <div className="score-lead-desc">
            {frame ? (frame.score_mean_black > 0.5 ? "黑稍优" : frame.score_mean_black < -0.5 ? "白稍优" : "形势接近") : "待评估"}
          </div>
        </div>
        <div className="player-stat">
          <span className="stone-dot white-dot"></span>
          <span className="player-name">白棋</span>
          <span className="captures">提子: {currentPosition?.captures_white ?? 0}</span>
        </div>
      </div>

      {/* 评论与问题手 Tab 切换 */}
      <div className="tabs-container">
        <div className="hud-tabs">
          <button
            type="button"
            className={`hud-tab${leftTab === "commentary" ? " active" : ""}`}
            onClick={() => setLeftTab("commentary")}
          >
            评论与解说
          </button>
          <button
            type="button"
            className={`hud-tab${leftTab === "problems" ? " active" : ""}`}
            onClick={() => setLeftTab("problems")}
          >
            问题手 ({problems.length})
          </button>
        </div>

        <div className="hud-tab-content">
          {leftTab === "commentary" ? (
            <div className="commentary-body">
              <div className="selected-node-facts">
                <p>
                  手数 {currentPosition?.move_number ?? currentMove}
                  {" · "}上一手 {lastMoveLabel(currentPosition, boardSize)}
                  {" · "}下一手 {nextPlayerLabel(currentPosition)}
                  {" · "}提子 黑 {currentPosition?.captures_black ?? 0} 白 {currentPosition?.captures_white ?? 0}
                </p>
                {commentEditorEnabled ? (
                  <label className="personal-comment-editor-label">
                    个人评论
                    <textarea
                      className="personal-comment-editor"
                      value={commentDraft}
                      onChange={(event) => setCommentDraft(event.target.value)}
                      onBlur={() => onCommitPersonalComment?.(commentDraft)}
                      spellCheck={false}
                      aria-label="个人评论"
                      placeholder="为当前选中节点写下个人评论"
                    />
                  </label>
                ) : personalComment ? (
                  <p className="personal-comment">{personalComment}</p>
                ) : null}
                {generatedInformation ? (
                  <p className="generated-information">{generatedInformation}</p>
                ) : null}
              </div>
              {frame ? (
                <div className="commentary-metrics">
                  <p className="commentary-text">
                    第 {currentMove} 手 KataGo 推荐首选 {activeCandidate && isPoint(activeCandidate.vertex) ? vertexLabel(activeCandidate.vertex, boardSize) : "推荐点"}，胜率 {(frame.winrate_black * 100).toFixed(1)}%，目差 {(frame.score_mean_black >= 0 ? "+" : "") + frame.score_mean_black.toFixed(1)} 目。
                  </p>
                  <div className="stats-badges">
                    <span>计算量: {frame.visits.toLocaleString()} visits</span>
                    <span>{hasOwnership ? "领地已评估" : ""}</span>
                  </div>
                </div>
              ) : (
                <p className="muted commentary">点击“分析当前节点”查看详细评注与局势分析。</p>
              )}
            </div>
          ) : (
            <div className="problems-body">
              {problems.length === 0 ? (
                <p className="muted">当前对局未标出明显问题手。</p>
              ) : (
                <ol className="problem-list">
                  {problems.slice(0, 16).map((p) => {
                    const isCurrent = currentMove === p.turn;
                    return (
                      <li key={p.turn} className={`severity-${p.severity}${isCurrent ? " is-current" : ""}`}>
                        <button
                          type="button"
                          className="problem-button"
                          onClick={() => onSelectProblem(p.turn)}
                        >
                          <span className="problem-turn">第 {p.turn} 手</span>
                          <strong className="problem-tag">{severityLabel[p.severity] ?? p.label}</strong>
                          <span className="problem-loss">胜率 -{(p.winrate_loss * 100).toFixed(1)}%</span>
                        </button>
                      </li>
                    );
                  })}
                </ol>
              )}
            </div>
          )}
        </div>
      </div>
    </aside>
  );
}

/**
 * SubBoardCanvas: 绘制参考图变化的木质副棋盘
 */
function SubBoardCanvas({
  boardSize,
  position,
  candidate,
  contentMode,
  pvPrefixLength
}: {
  boardSize: number;
  position?: PositionDto;
  candidate: CandidateMoveDto | null;
  contentMode: SubBoardContentMode;
  pvPrefixLength?: number;
}) {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    const ctx = canvas?.getContext("2d");
    if (!canvas || !ctx) return;

    const dpr = window.devicePixelRatio || 1;
    const cssSize = Math.min(canvas.clientWidth || 240, canvas.clientHeight || 240);
    canvas.width = Math.floor(cssSize * dpr);
    canvas.height = Math.floor(cssSize * dpr);
    ctx.scale(dpr, dpr);
    ctx.clearRect(0, 0, cssSize, cssSize);

    const padding = cssSize * 0.08;
    const grid = (cssSize - padding * 2) / (boardSize - 1);
    const coord = (n: number) => padding + n * grid;

    // 1. 木质底色
    ctx.fillStyle = "#dfb77d";
    ctx.fillRect(0, 0, cssSize, cssSize);

    // 2. 网格线
    ctx.strokeStyle = "rgba(42, 28, 14, 0.75)";
    ctx.lineWidth = 0.8;
    for (let i = 0; i < boardSize; i += 1) {
      ctx.beginPath(); ctx.moveTo(coord(0), coord(i)); ctx.lineTo(coord(boardSize - 1), coord(i)); ctx.stroke();
      ctx.beginPath(); ctx.moveTo(coord(i), coord(0)); ctx.lineTo(coord(i), coord(boardSize - 1)); ctx.stroke();
    }

    // 3. 星位点
    const stars = boardSize === 19 ? [3, 9, 15] : boardSize === 13 ? [3, 6, 9] : [2, boardSize - 3];
    ctx.fillStyle = "rgba(42, 28, 14, 0.85)";
    for (const x of stars) {
      for (const y of stars) {
        ctx.beginPath(); ctx.arc(coord(x), coord(y), Math.max(1.5, grid * 0.07), 0, Math.PI * 2); ctx.fill();
      }
    }

    // 4. 当前棋盘上的已有棋子 (半透明淡化作为背景)
    if (position?.stones) {
      for (const stone of position.stones) {
        const cx = coord(stone.x);
        const cy = coord(stone.y);
        const radius = grid * 0.44;
        ctx.beginPath(); ctx.arc(cx, cy, radius, 0, Math.PI * 2);
        ctx.fillStyle = stone.color === "black" ? "rgba(20,20,22,0.92)" : "rgba(248,246,240,0.94)";
        ctx.fill();
        ctx.strokeStyle = "rgba(30,25,20,0.6)";
        ctx.lineWidth = 0.6;
        ctx.stroke();
      }
    }

    // 5. PV 变化步骤绘制 (1, 2, 3, 4, 5...)
    if (contentMode === "variation" && candidate) {
      const pvSteps: { x: number; y: number; step: number; color: "black" | "white" }[] = [];
      let turnColor: "black" | "white" = position?.to_play ?? "black";
      const visibleSteps = variationReplayPointSteps(candidate).slice(
        0,
        pvPrefixLength ?? Number.POSITIVE_INFINITY
      );

      for (const [index, point] of visibleSteps.entries()) {
        if (index > 0) turnColor = turnColor === "black" ? "white" : "black";
        pvSteps.push({
          x: point.x,
          y: point.y,
          step: index + 1,
          color: turnColor
        });
      }

      // 绘制 PV 棋子与序号
      for (const item of pvSteps) {
        const cx = coord(item.x);
        const cy = coord(item.y);
        const radius = grid * 0.44;

        ctx.beginPath(); ctx.arc(cx, cy, radius, 0, Math.PI * 2);
        ctx.fillStyle = item.color === "black" ? "#1a1a1e" : "#ffffff";
        ctx.fill();
        ctx.strokeStyle = item.step === 1 ? "#2563eb" : "rgba(42,28,14,0.8)";
        ctx.lineWidth = item.step === 1 ? 1.5 : 1;
        ctx.stroke();

        // 数字序号
        ctx.fillStyle = item.color === "black" ? "#ffffff" : "#1a1a1e";
        ctx.font = `bold ${Math.max(9, grid * 0.4)}px "Noto Sans SC", sans-serif`;
        ctx.textAlign = "center";
        ctx.textBaseline = "middle";
        ctx.fillText(String(item.step), cx, cy);
      }
    }
  }, [boardSize, position, candidate, contentMode, pvPrefixLength]);

  return (
    <canvas
      ref={canvasRef}
      className="subboard-canvas"
      aria-label={contentMode === "raw" ? "纯棋子副棋盘" : "参考图变化副棋盘"}
    />
  );
}

function lastMoveLabel(position: PositionDto | undefined, boardSize: number): string {
  const last = position?.last_move;
  if (!last) return "无";
  const color = last.color === "black" ? "黑" : "白";
  if (!isPoint(last.vertex)) return `${color} 虚手`;
  return `${color} ${vertexLabel(last.vertex, boardSize)}`;
}

function nextPlayerLabel(position: PositionDto | undefined): string {
  if (!position) return "—";
  return position.to_play === "black" ? "黑" : "白";
}
