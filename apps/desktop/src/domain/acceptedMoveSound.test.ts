import { describe, expect, it } from "vitest";
import { acceptedMoveSound } from "./acceptedMoveSound";
import type { SelectedNodeSnapshotDto } from "./types";

function snapshot(indices: number[], moveNumber: number, captures = 0, pass = false): SelectedNodeSnapshotDto {
  return {
    path: { indices }, personal_comment: "", markup: [], stone_move_numbers: [],
    position: { board_width: 9, board_height: 9, move_number: moveNumber, to_play: "white", stones: [],
      captures_black: captures, captures_white: 0, errors: [],
      last_move: { color: "black", vertex: pass ? "pass" : { point: { x: 2, y: 2 } }, move_number: moveNumber } }
  };
}

describe("accepted review move sound", () => {
  it("uses the actual branch capture delta and includes pass", () => {
    const before = snapshot([1], 4, 5);
    expect(acceptedMoveSound(before, snapshot([1, 2], 5, 5))).toBe("move");
    expect(acceptedMoveSound(before, snapshot([1, 2], 5, 7))).toBe("capture-small");
    expect(acceptedMoveSound(before, snapshot([1, 2], 5, 8))).toBe("capture-large");
    expect(acceptedMoveSound(before, snapshot([1, 2], 5, 5, true))).toBe("pass");
    expect(acceptedMoveSound(before, snapshot([1, 0, 2], 5, 5))).toBe("move");
  });
  it("rejects duplicate, backward, sibling, jump and non-move transitions", () => {
    const before = snapshot([1], 4);
    for (const after of [before, snapshot([], 3), snapshot([2], 5), snapshot([1, 0, 0], 6), snapshot([1, 0], 4)]) {
      expect(acceptedMoveSound(before, after)).toBeNull();
    }
  });
});
