import { describe, expect, it } from "vitest";
import { projectWorkspace, resizeWorkspace, type WorkspaceShares, type WorkspaceVisibility } from "./projection";

describe("projectWorkspace", () => {
  const visibleBoth: WorkspaceVisibility = { left: true, right: true };
  const visibleLeftOnly: WorkspaceVisibility = { left: true, right: false };
  const visibleRightOnly: WorkspaceVisibility = { left: false, right: true };
  const hiddenBoth: WorkspaceVisibility = { left: false, right: false };

  it("applies default wide sizes (228 / 260) when input width > 1180 with null shares", () => {
    const result = projectWorkspace(1200, null, visibleBoth);
    expect(result.left).toBe(228);
    expect(result.right).toBe(260);
    expect(result.separator).toBe(8);
    expect(result.width).toBe(1200);
    expect(result.center).toBe(1200 - 16 - 228 - 260);
    expect(result.center).toBeGreaterThanOrEqual(360);
    expect(result.left + 8 + result.center + 8 + result.right).toBe(result.width);
  });

  it("applies default narrow sizes (210 / 240) when input width <= 1180 with null shares", () => {
    const at1180 = projectWorkspace(1180, null, visibleBoth);
    expect(at1180.left).toBe(210);
    expect(at1180.right).toBe(240);
    expect(at1180.center).toBe(1180 - 16 - 210 - 240);
    expect(at1180.left + 8 + at1180.center + 8 + at1180.right).toBe(at1180.width);

    const at1000 = projectWorkspace(1000, null, visibleBoth);
    expect(at1000.left).toBe(210);
    expect(at1000.right).toBe(240);
    expect(at1000.center).toBe(1000 - 16 - 210 - 240);
  });

  it("clamps output width to minimum total including separators when input width is too small", () => {
    // Both visible: minTotal = 210 + 240 + 360 + 16 = 826
    const small = projectWorkspace(500, null, visibleBoth);
    expect(small.width).toBe(826);
    expect(small.left).toBe(210);
    expect(small.right).toBe(240);
    expect(small.center).toBe(360);
    expect(small.left + 8 + small.center + 8 + small.right).toBe(small.width);

    // Negative / NaN width handling
    const neg = projectWorkspace(-100, null, visibleBoth);
    expect(neg.width).toBe(826);
    expect(Number.isFinite(neg.left)).toBe(true);
    expect(Number.isFinite(neg.center)).toBe(true);
    expect(Number.isFinite(neg.right)).toBe(true);

    const nanWidth = projectWorkspace(NaN, null, visibleBoth);
    expect(nanWidth.width).toBe(826);
    expect(Number.isNaN(nanWidth.center)).toBe(false);
  });

  it("sets hidden sides to 0 without mutating shares object", () => {
    const shares: WorkspaceShares = Object.freeze({ left: 0.25, right: 0.25 });

    const leftOnly = projectWorkspace(1200, shares, visibleLeftOnly);
    expect(leftOnly.left).toBeGreaterThan(0);
    expect(leftOnly.right).toBe(0);
    expect(leftOnly.left + 8 + leftOnly.center).toBe(leftOnly.width);

    const rightOnly = projectWorkspace(1200, shares, visibleRightOnly);
    expect(rightOnly.left).toBe(0);
    expect(rightOnly.right).toBeGreaterThan(0);
    expect(rightOnly.center + 8 + rightOnly.right).toBe(rightOnly.width);

    const bothHidden = projectWorkspace(1200, shares, hiddenBoth);
    expect(bothHidden.left).toBe(0);
    expect(bothHidden.right).toBe(0);
    expect(bothHidden.center).toBe(bothHidden.width);
  });

  it("falls back to default policy for invalid shares without propagating NaN", () => {
    const invalidSum = projectWorkspace(1200, { left: 0.6, right: 0.5 }, visibleBoth);
    expect(invalidSum.left).toBe(228);
    expect(invalidSum.right).toBe(260);

    const negativeShare = projectWorkspace(1200, { left: -0.1, right: 0.2 }, visibleBoth);
    expect(negativeShare.left).toBe(228);
    expect(negativeShare.right).toBe(260);

    const nanShare = projectWorkspace(1200, { left: NaN, right: 0.2 }, visibleBoth);
    expect(nanShare.left).toBe(228);
    expect(nanShare.right).toBe(260);
  });

  it("projects feasible custom shares within bounds and respects visible minima", () => {
    // 1200 width - 16 separators = 1184 available column width
    const shares: WorkspaceShares = { left: 0.25, right: 0.25 };
    const result = projectWorkspace(1200, shares, visibleBoth);
    expect(result.left).toBeCloseTo(0.25 * 1184, 5);
    expect(result.right).toBeCloseTo(0.25 * 1184, 5);
    expect(result.center).toBeCloseTo(1184 - result.left - result.right, 5);
    expect(result.center).toBeGreaterThanOrEqual(360);
    expect(result.left + 8 + result.center + 8 + result.right).toBe(1200);

    // Shares requesting below minima are clamped up to 210/240
    const tinyShares: WorkspaceShares = { left: 0.05, right: 0.05 };
    const clampedTiny = projectWorkspace(1000, tinyShares, visibleBoth);
    expect(clampedTiny.left).toBe(210);
    expect(clampedTiny.right).toBe(240);
    expect(clampedTiny.center).toBe(1000 - 16 - 210 - 240);
  });

  it("fairly allocates excess proportional to requested excess when both exceed feasible space", () => {
    // Narrow width 900: available 884. minSides = 450. maxSides = 884 - 360 = 524. feasibleExcess = 74.
    const greedyShares: WorkspaceShares = { left: 0.45, right: 0.45 };
    const result = projectWorkspace(900, greedyShares, visibleBoth);

    expect(result.left).toBeGreaterThanOrEqual(210);
    expect(result.right).toBeGreaterThanOrEqual(240);
    expect(result.center).toBeCloseTo(360, 5);
    expect(result.left + 8 + result.center + 8 + result.right).toBe(900);
  });
});

describe("resizeWorkspace", () => {
  const visibleBoth: WorkspaceVisibility = { left: true, right: true };
  const visibleLeftOnly: WorkspaceVisibility = { left: true, right: false };
  it("keeps the opposite visible width when resizing an already constrained preference", () => {
    const shares = { left: 0.45, right: 0.45 };
    const before = projectWorkspace(1000, shares, visibleBoth);
    const after = projectWorkspace(1000, resizeWorkspace(before, "left", -30, shares, visibleBoth), visibleBoth);
    expect(after.right).toBeCloseTo(before.right);
    expect(after.left).toBeCloseTo(before.left - 30);
    expect(after.center).toBeCloseTo(before.center + 30);
  });

  it("retains a large hidden share even at the visible drag limit", () => {
    const shares = { left: 0.1, right: 0.8 };
    const before = projectWorkspace(1100, shares, visibleLeftOnly);
    const after = resizeWorkspace(before, "left", 5000, shares, visibleLeftOnly);
    expect(after.right).toBe(shares.right);
    expect(after.left + after.right).toBeLessThan(1);
    expect(projectWorkspace(1100, after, visibleLeftOnly).center).toBeGreaterThanOrEqual(360);
  });


  it("increases requested side on positive delta and converts to share of column width", () => {
    const initial = projectWorkspace(1200, null, visibleBoth);
    const updated = resizeWorkspace(initial, "left", 50, null, visibleBoth);

    expect(updated.left).toBeGreaterThan(initial.left / (1200 - 16));
    expect(updated.right).toBeCloseTo(initial.right / (1200 - 16), 5);
    expect(updated.left + updated.right).toBeLessThan(1);

    const projected = projectWorkspace(1200, updated, visibleBoth);
    expect(projected.left).toBe(228 + 50);
    expect(projected.right).toBe(initial.right);
    expect(projected.center).toBeGreaterThanOrEqual(360);
  });

  it("clamps affected side so opposite visible side retains projected width and center >= 360 under extreme positive delta", () => {
    const initial = projectWorkspace(1000, null, visibleBoth);
    // Extreme delta +5000: left cannot push center below 360
    const updated = resizeWorkspace(initial, "left", 5000, null, visibleBoth);

    const projected = projectWorkspace(1000, updated, visibleBoth);
    expect(projected.right).toBe(initial.right);
    expect(projected.center).toBeCloseTo(360, 5);
    expect(projected.left + 8 + projected.center + 8 + projected.right).toBe(1000);
    expect(updated.left + updated.right).toBeLessThan(1);
  });

  it("clamps affected side to minimum width under extreme negative delta", () => {
    const initial = projectWorkspace(1200, null, visibleBoth);
    // Extreme delta -5000: left cannot shrink below 210
    const updated = resizeWorkspace(initial, "left", -5000, null, visibleBoth);

    const projected = projectWorkspace(1200, updated, visibleBoth);
    expect(projected.left).toBe(210);
    expect(projected.right).toBe(initial.right);
  });

  it("preserves stored share of other side and handles hidden side reservation through roundtrip", () => {
    // User starts with custom shares: left 0.25, right 0.3
    const customShares: WorkspaceShares = { left: 0.25, right: 0.3 };

    // 1. Right side is hidden
    const projectedHidden = projectWorkspace(1200, customShares, visibleLeftOnly);
    expect(projectedHidden.left).toBeGreaterThan(0);
    expect(projectedHidden.right).toBe(0);

    // 2. User resizes left side while right side is hidden
    const resizedShares = resizeWorkspace(projectedHidden, "left", 40, customShares, visibleLeftOnly);

    // Stored share for hidden right side must be preserved
    expect(resizedShares.right).toBe(customShares.right);
    expect(resizedShares.left + resizedShares.right).toBeLessThan(1);

    // 3. User unhides right side: preference intent is retained and restores feasible projection
    const restored = projectWorkspace(1200, resizedShares, visibleBoth);
    expect(restored.right).toBeCloseTo(0.3 * (1200 - 16), 4);
    expect(restored.center).toBeGreaterThanOrEqual(360);
    expect(restored.left + 8 + restored.center + 8 + restored.right).toBe(1200);
  });

  it("preserves preference intent across temporary viewport shrink and restore", () => {
    // 1. Custom preference at 1200px
    const initialShares: WorkspaceShares = { left: 0.28, right: 0.28 };
    const normal = projectWorkspace(1200, initialShares, visibleBoth);
    expect(normal.left).toBeCloseTo(0.28 * 1184, 4);

    // 2. Viewport shrinks to 826px (minimum total), forcing both sides into minima
    const shrunk = projectWorkspace(826, initialShares, visibleBoth);
    expect(shrunk.left).toBe(210);
    expect(shrunk.right).toBe(240);
    expect(shrunk.center).toBe(360);

    // 3. Viewport expands back to 1200px without mutating stored shares: preference restored
    const expanded = projectWorkspace(1200, initialShares, visibleBoth);
    expect(expanded.left).toBeCloseTo(0.28 * 1184, 4);
    expect(expanded.right).toBeCloseTo(0.28 * 1184, 4);
  });
});
