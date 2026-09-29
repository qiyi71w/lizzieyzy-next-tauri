import { useEffect, useRef, useState, type CSSProperties } from "react";
import { useElementSize } from "../workspace/useElementSize";
import type { AnalysisFrameDto, CandidateMoveDto, NodePath, PositionDto, ProblemMarkerDto } from "../domain/types";
import type { ChartPoint } from "../domain/winrateChart";
import type { ReviewProblem } from "../domain/reviewNavigation";
import { isPoint, vertexLabel } from "../domain/board";
import { variationReplayPointSteps } from "../domain/variationReplay";

type PolicyPoint = { x: number; y: number; value: number };
type Pane = "commentary" | "reference";
type SubBoardContentMode = "variation" | "raw";

type Props = {
  frame?: AnalysisFrameDto;
  problems: ReviewProblem[];
  reviewLine?: ChartPoint[];
  selectedPath?: NodePath;
  onSelectNode?: (path: NodePath) => void;
  boardWidth: number;
  boardHeight: number;
  currentMove: number;
  currentPosition?: PositionDto;
  blackName?: string | null;
  whiteName?: string | null;
  personalComment?: string;
  generatedInformation?: string | null;
  commentEditorEnabled?: boolean;
  onCommitPersonalComment?: (comment: string) => void;
  selectedCandidateIndex: number | null;
  previewCandidateIndex?: number | null;
  onSelectCandidate: (index: number) => void;
  onSelectProblem: (problem: ReviewProblem) => void;
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
  reviewLine = [],
  selectedPath,
  onSelectNode,
  boardWidth,
  boardHeight,
  currentMove,
  blackName,
  whiteName,
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
  const commentNodeKey = selectedPath?.indices.join(",");
  useEffect(() => {
    setCommentDraft(personalComment);
  }, [personalComment, commentNodeKey]);
  const hasOwnership = (frame?.ownership?.length ?? 0) >= boardWidth * boardHeight;
  const rootScore = frame?.score_mean_black;

  const candidates = frame?.candidates ?? [];
  const activeCandidateIndex = previewCandidateIndex ?? selectedCandidateIndex ?? 0;
  const activeCandidate = candidates[activeCandidateIndex] ?? candidates[0] ?? null;

  if (pane === "reference") {
    return (
      <aside className="analysis-panel reference-panel" aria-label="参考图与选点">
        <section className="variation-tree" aria-label="变化">
          {reviewLine.map((point) => {
            const key = point.path.indices.join(",");
            return (
              <button
                key={key}
                type="button"
                className={selectedPath?.indices.join(",") === key ? "is-current" : undefined}
                data-node-path={key}
                onClick={(event) => {
                  if (event.currentTarget.isConnected && !event.currentTarget.closest("[hidden]")) {
                    onSelectNode?.(point.path);
                  }
                }}
              >
                {point.path.indices.length === 0 ? "根" : `${point.moveNumber} ${point.isMove ? "棋步" : "非棋步"}`} · {key || "root"}
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
                    const moveText = vertexLabel(candidate.vertex, boardHeight);
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
            <p className="muted commentary" style={{ padding: "12px 10px", margin: 0 }}>
              暂无候选点。点击“分析当前节点”或“分析第一子主线”。
            </p>
          )}
        </section>
        <section className="subboard-section" aria-label="副棋盘">
          <div className="subboard-card">
            <SubBoardCanvas
              boardWidth={boardWidth}
              boardHeight={boardHeight}
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
          <span className="player-name" title={blackName?.trim() ? blackName : "黑棋"}>{blackName?.trim() ? blackName : "黑棋"}</span>
          <span className="captures">提子: {currentPosition?.captures_black ?? 0}</span>
        </div>
        <div className="score-lead-box">
          <div className="score-lead-val">
            {rootScore != null ? (rootScore >= 0 ? `+${rootScore.toFixed(1)}` : rootScore.toFixed(1)) : "—"}
          </div>
          <div className="score-lead-desc">
            {rootScore != null
              ? (rootScore > 0.5 ? "黑稍优" : rootScore < -0.5 ? "白稍优" : "形势接近")
              : frame ? "目数不可用" : "待评估"}
          </div>
        </div>
        <div className="player-stat">
          <span className="stone-dot white-dot"></span>
          <span className="player-name" title={whiteName?.trim() ? whiteName : "白棋"}>{whiteName?.trim() ? whiteName : "白棋"}</span>
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
                  {" · "}上一手 {lastMoveLabel(currentPosition, boardHeight)}
                  {" · "}下一手 {nextPlayerLabel(currentPosition)}
                  {" · "}提子 黑 {currentPosition?.captures_black ?? 0} 白 {currentPosition?.captures_white ?? 0}
                </p>
                {commentEditorEnabled ? (
                  <>
                    <label className="personal-comment-editor-label">
                      个人评论
                      <textarea
                        className="personal-comment-editor"
                        value={commentDraft}
                        onChange={(event) => setCommentDraft(event.target.value)}
                        spellCheck={false}
                        aria-label="个人评论"
                        placeholder="为当前选中节点写下个人评论"
                      />
                    </label>
                    <button
                      type="button"
                      className="personal-comment-apply-button"
                      onClick={() => onCommitPersonalComment?.(commentDraft)}
                    >
                      应用评论
                    </button>
                  </>
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
                    第 {currentMove} 手 KataGo 推荐首选 {activeCandidate && isPoint(activeCandidate.vertex) ? vertexLabel(activeCandidate.vertex, boardHeight) : "推荐点"}，胜率 {(frame.winrate_black * 100).toFixed(1)}%，{rootScore != null ? `目差 ${rootScore >= 0 ? "+" : ""}${rootScore.toFixed(1)} 目` : "目差不可用"}。
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
                    const isCurrent = p.path ? selectedPath?.indices.join(",") === p.path.indices.join(",") : currentMove === p.turn;
                    return (
                      <li key={p.path?.indices.join(",") ?? p.turn} className={`severity-${p.severity}${isCurrent ? " is-current" : ""}`}>
                        <button
                          type="button"
                          className="problem-button"
                          onClick={(event) => {
                            if (event.currentTarget.isConnected && !event.currentTarget.closest("[hidden]")) {
                              onSelectProblem(p);
                            }
                          }}
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
  boardWidth,
  boardHeight,
  position,
  candidate,
  contentMode,
  pvPrefixLength
}: {
  boardWidth: number;
  boardHeight: number;
  position?: PositionDto;
  candidate: CandidateMoveDto | null;
  contentMode: SubBoardContentMode;
  pvPrefixLength?: number;
}) {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const size = useElementSize(canvasRef);

  useEffect(() => {
    const canvas = canvasRef.current;
    const ctx = canvas?.getContext("2d");
    if (!canvas || !ctx) return;

    const { width: cssWidth, height: cssHeight, dpr } = size;
    if (cssWidth <= 0 || cssHeight <= 0) return;
    canvas.width = Math.floor(cssWidth * dpr);
    canvas.height = Math.floor(cssHeight * dpr);
    ctx.scale(dpr, dpr);
    ctx.clearRect(0, 0, cssWidth, cssHeight);

    const padding = Math.min(cssWidth, cssHeight) * 0.08;
    const grid = Math.min(
      (cssWidth - padding * 2) / Math.max(boardWidth - 1, 1),
      (cssHeight - padding * 2) / Math.max(boardHeight - 1, 1)
    );
    const offsetX = (cssWidth - grid * (boardWidth - 1)) / 2;
    const offsetY = (cssHeight - grid * (boardHeight - 1)) / 2;
    const coordX = (n: number) => offsetX + n * grid;
    const coordY = (n: number) => offsetY + n * grid;

    ctx.fillStyle = "#dfb77d";
    ctx.fillRect(0, 0, cssWidth, cssHeight);
    ctx.strokeStyle = "rgba(42, 28, 14, 0.75)";
    ctx.lineWidth = 0.8;
    for (let y = 0; y < boardHeight; y += 1) {
      ctx.beginPath(); ctx.moveTo(coordX(0), coordY(y)); ctx.lineTo(coordX(boardWidth - 1), coordY(y)); ctx.stroke();
    }
    for (let x = 0; x < boardWidth; x += 1) {
      ctx.beginPath(); ctx.moveTo(coordX(x), coordY(0)); ctx.lineTo(coordX(x), coordY(boardHeight - 1)); ctx.stroke();
    }

    ctx.fillStyle = "rgba(42, 28, 14, 0.85)";
    for (const x of starCoordinates(boardWidth)) for (const y of starCoordinates(boardHeight)) {
      ctx.beginPath(); ctx.arc(coordX(x), coordY(y), Math.max(1.5, grid * 0.07), 0, Math.PI * 2); ctx.fill();
    }

    for (const stone of position?.stones ?? []) {
      const cx = coordX(stone.x);
      const cy = coordY(stone.y);
      const radius = grid * 0.44;
      ctx.beginPath(); ctx.arc(cx, cy, radius, 0, Math.PI * 2);
      ctx.fillStyle = stone.color === "black" ? "rgba(20,20,22,0.92)" : "rgba(248,246,240,0.94)";
      ctx.fill();
      ctx.strokeStyle = "rgba(30,25,20,0.6)";
      ctx.lineWidth = 0.6;
      ctx.stroke();
    }

    if (contentMode === "variation" && candidate) {
      let turnColor: "black" | "white" = position?.to_play ?? "black";
      const visibleSteps = variationReplayPointSteps(candidate).slice(0, pvPrefixLength ?? Number.POSITIVE_INFINITY);
      for (const [index, point] of visibleSteps.entries()) {
        if (index > 0) turnColor = turnColor === "black" ? "white" : "black";
        const cx = coordX(point.x);
        const cy = coordY(point.y);
        const radius = grid * 0.44;
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
  }, [size, boardWidth, boardHeight, position, candidate, contentMode, pvPrefixLength]);

  return (
    <canvas
      ref={canvasRef}
      className="subboard-canvas"
      style={{
        aspectRatio: `${boardWidth} / ${boardHeight}`,
        width: `${Math.min(1, boardWidth / boardHeight) * 100}%`,
        height: `${Math.min(1, boardHeight / boardWidth) * 100}%`
      } as CSSProperties}
      aria-label={contentMode === "raw" ? "纯棋子副棋盘" : "参考图变化副棋盘"}
    />
  );
}

function starCoordinates(length: number): number[] {
  if (length === 19) return [3, 9, 15];
  if (length === 13) return [3, 6, 9];
  if (length === 9) return [2, 4, 6];
  if (length < 7) return [];
  return [3, length - 4].filter((value, index, values) => value >= 0 && value < length && values.indexOf(value) === index);
}

function lastMoveLabel(position: PositionDto | undefined, boardHeight: number): string {
  const last = position?.last_move;
  if (!last) return "无";
  const color = last.color === "black" ? "黑" : "白";
  if (!isPoint(last.vertex)) return `${color} 虚手`;
  return `${color} ${vertexLabel(last.vertex, boardHeight)}`;
}

function nextPlayerLabel(position: PositionDto | undefined): string {
  if (!position) return "—";
  return position.to_play === "black" ? "黑" : "白";
}
