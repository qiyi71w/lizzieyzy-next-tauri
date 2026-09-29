import type { WorkspaceSharesDto } from "../domain/types";
export type WorkspaceShares = WorkspaceSharesDto;

export type WorkspaceVisibility = {
  left: boolean;
  right: boolean;
};

export type ProjectedWorkspace = {
  left: number;
  center: number;
  right: number;
  separator: number;
  width: number;
};

const MIN_LEFT = 210;
const MIN_RIGHT = 240;
const MIN_CENTER = 360;
const SEPARATOR_WIDTH = 8;


function isValidShares(shares: WorkspaceShares | null): shares is WorkspaceShares {
  if (!shares || typeof shares !== "object") return false;
  const leftValid = typeof shares.left === "number" && Number.isFinite(shares.left) && shares.left >= 0 && shares.left < 1;
  const rightValid = typeof shares.right === "number" && Number.isFinite(shares.right) && shares.right >= 0 && shares.right < 1;
  return leftValid && rightValid && Number.isFinite(shares.left + shares.right) && shares.left + shares.right < 1;
}

export function projectWorkspace(
  width: number,
  shares: WorkspaceShares | null,
  visibility: WorkspaceVisibility
): ProjectedWorkspace {
  const visibleLeft = Boolean(visibility?.left);
  const visibleRight = Boolean(visibility?.right);

  const totalSeparators = (visibleLeft ? SEPARATOR_WIDTH : 0) + (visibleRight ? SEPARATOR_WIDTH : 0);
  const minLeft = visibleLeft ? MIN_LEFT : 0;
  const minRight = visibleRight ? MIN_RIGHT : 0;
  const minTotal = minLeft + minRight + MIN_CENTER + totalSeparators;

  const outputWidth = Math.max(Number.isFinite(width) ? width : 0, minTotal);
  const availableColumnWidth = outputWidth - totalSeparators;

  if (!isValidShares(shares)) {
    const defaultLeft = width > 1180 ? 228 : 210;
    const defaultRight = width > 1180 ? 260 : 240;

    const left = visibleLeft ? defaultLeft : 0;
    const right = visibleRight ? defaultRight : 0;
    const center = availableColumnWidth - left - right;

    return {
      left,
      center,
      right,
      separator: SEPARATOR_WIDTH,
      width: outputWidth,
    };
  }

  const reqL = visibleLeft ? shares.left * availableColumnWidth : 0;
  const reqR = visibleRight ? shares.right * availableColumnWidth : 0;

  const reqExcessL = visibleLeft ? Math.max(0, reqL - minLeft) : 0;
  const reqExcessR = visibleRight ? Math.max(0, reqR - minRight) : 0;
  const totalReqExcess = reqExcessL + reqExcessR;

  const maxSidesTotal = availableColumnWidth - MIN_CENTER;
  const feasibleExcess = Math.max(0, maxSidesTotal - (minLeft + minRight));

  let left: number;
  let right: number;

  if (totalReqExcess <= feasibleExcess) {
    left = visibleLeft ? minLeft + reqExcessL : 0;
    right = visibleRight ? minRight + reqExcessR : 0;
  } else {
    const allocatedExcessL = visibleLeft && totalReqExcess > 0
      ? feasibleExcess * (reqExcessL / totalReqExcess)
      : 0;
    const allocatedExcessR = visibleRight
      ? (feasibleExcess - allocatedExcessL)
      : 0;

    left = visibleLeft ? minLeft + allocatedExcessL : 0;
    right = visibleRight ? minRight + allocatedExcessR : 0;
  }

  const center = availableColumnWidth - left - right;

  return {
    left,
    center,
    right,
    separator: SEPARATOR_WIDTH,
    width: outputWidth,
  };
}

export function resizeWorkspace(
  projected: ProjectedWorkspace,
  side: "left" | "right",
  delta: number,
  shares: WorkspaceShares | null,
  visibility: WorkspaceVisibility
): WorkspaceShares {
  const visibleLeft = Boolean(visibility?.left);
  const visibleRight = Boolean(visibility?.right);
  const totalSeparators = (visibleLeft ? SEPARATOR_WIDTH : 0) + (visibleRight ? SEPARATOR_WIDTH : 0);
  const availableColumnWidth = Math.max(projected.width - totalSeparators, 1);

  const otherSide = side === "left" ? "right" : "left";
  const otherStored = isValidShares(shares) ? shares[otherSide] : null;

  let otherShare: number;
  if (visibility[otherSide]) {
    otherShare = projected[otherSide] / availableColumnWidth;
  } else if (otherStored !== null) {
    otherShare = otherStored;
  } else {
    const defaultOtherWidth = otherSide === "left"
      ? (projected.width > 1180 ? 228 : 210)
      : (projected.width > 1180 ? 260 : 240);
    otherShare = defaultOtherWidth / availableColumnWidth;
  }

  const currentWidth = side === "left" ? projected.left : projected.right;
  const safeDelta = Number.isFinite(delta) ? delta : 0;
  const targetWidth = currentWidth + safeDelta;

  const minSide = visibility[side] ? (side === "left" ? MIN_LEFT : MIN_RIGHT) : 0;

  const oppositeSpace = visibility[otherSide] ? projected[otherSide] : 0;
  const maxSideWidth = Math.max(minSide, availableColumnWidth - oppositeSpace - MIN_CENTER);
  const clampedWidth = Math.max(minSide, Math.min(targetWidth, maxSideWidth));
  // A hidden preference reserves share, not visible pixels. Keep it unchanged.
  const newSideShare = Math.min(clampedWidth / availableColumnWidth, 1 - otherShare - Number.EPSILON);
  const newLeftShare = side === "left" ? newSideShare : otherShare;
  const newRightShare = side === "right" ? newSideShare : otherShare;

  return {
    left: Number.isFinite(newLeftShare) && newLeftShare >= 0 ? newLeftShare : 0,
    right: Number.isFinite(newRightShare) && newRightShare >= 0 ? newRightShare : 0,
  };
}
