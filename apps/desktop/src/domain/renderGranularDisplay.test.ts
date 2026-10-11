import { expect, it, vi } from "vitest";
import { drawOwnership } from "./renderGranularDisplay";
import { defaultGranularDisplay, type OwnershipStyle } from "./granularDisplay";
import type { PositionDto } from "./types";
it("draws five source styles with strict by-size threshold, alpha and same-color-stone exclusion", () => {
  const position: PositionDto = { board_width: 2, board_height: 2, move_number: 0, to_play: "black", stones: [{ x: 0, y: 0, color: "black" }], captures_black: 0, captures_white: 0, last_move: null, errors: [] };
  const colors: string[] = [];
  const widths: number[] = [];
  const context = { fillStyle: "", fillRect: vi.fn((_x: number, _y: number, width: number) => { colors.push(context.fillStyle); widths.push(width); }) };
  const expected: Record<OwnershipStyle, number[]> = { small: [25, 100 / 3, 25, 25], normal: [50, 50, 50, 50], notOnLive: [50, 50, 50], bigBelow: [100, 100, 100, 100], bySize: [48, 30, 12.06] };
  for (const ownershipStyle of ["small", "normal", "notOnLive", "bigBelow", "bySize"] as const) {
    widths.length = 0; colors.length = 0;
    drawOwnership(context as unknown as CanvasRenderingContext2D, [0.8, -0.5, 0.2, 0.201], position, { ...defaultGranularDisplay, ownershipStyle }, 100, x => x * 100, y => y * 100);
    expect(widths.length).toBe(expected[ownershipStyle].length);
    widths.forEach((value, index) => expect(value).toBeCloseTo(expected[ownershipStyle][index]));
    if (ownershipStyle === "bySize") expect(colors[0]).toBe(`rgba(0,0,0,${180 / 255})`);
    if (ownershipStyle === "bigBelow") expect(colors[0]).toBe(`rgba(0,0,0,${0.8 * 105 / 255})`);
  }
});
