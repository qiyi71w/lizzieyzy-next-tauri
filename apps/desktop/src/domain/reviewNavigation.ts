import type { NodePath, ProblemMarkerDto } from "./types";
import type { ChartPoint } from "./winrateChart";
import { classifyPlayedMove, isBlunderBarRank } from "./moveRank";

export type ReviewProblem = ProblemMarkerDto & { path?: NodePath };

// Only adjacent analyzed positions establish a played move's loss. A gap,
// setup or annotation must not be blamed on an unrelated move number.
export function reviewLineProblems(points: ChartPoint[]): ReviewProblem[] {
  const problems: ReviewProblem[] = [];
  for (let index = 1; index < points.length; index += 1) {
    const parent = points[index - 1];
    const child = points[index];
    if (!child.isMove || !parent.analysis || !child.analysis) continue;
    const severity = classifyPlayedMove(parent.analysis, child.analysis, parent.toPlay);
    if (!severity || !isBlunderBarRank(severity)) continue;
    const sign = parent.toPlay === "black" ? 1 : -1;
    problems.push({
      path: child.path,
      turn: child.moveNumber,
      severity,
      label: severity,
      winrate_loss: Math.max(0, sign * (parent.analysis.winrateBlack - child.analysis.winrateBlack)),
      score_loss: parent.analysis.scoreMeanBlack == null || child.analysis.scoreMeanBlack == null
        ? null : Math.max(0, sign * (parent.analysis.scoreMeanBlack - child.analysis.scoreMeanBlack))
    });
  }
  return problems;
}
