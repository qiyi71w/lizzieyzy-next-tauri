import type { MoveSoundKind } from "./moveSound";
import type { SelectedNodeSnapshotDto } from "./types";

/** Classify an accepted forward move using authoritative replay captures. */
export function acceptedMoveSound(before: SelectedNodeSnapshotDto, after: SelectedNodeSnapshotDto): MoveSoundKind | null {
  if (after.path.indices.length <= before.path.indices.length
    || !before.path.indices.every((index, depth) => after.path.indices[depth] === index)
    || after.position.move_number !== before.position.move_number + 1) return null;
  const move = after.position.last_move;
  if (!move || move.move_number !== after.position.move_number) return null;
  if (move.vertex === "pass") return "pass";
  const captures = move.color === "black"
    ? after.position.captures_black - before.position.captures_black
    : after.position.captures_white - before.position.captures_white;
  return captures >= 3 ? "capture-large" : captures >= 1 ? "capture-small" : "move";
}
