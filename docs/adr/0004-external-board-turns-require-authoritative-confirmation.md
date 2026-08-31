# Commit external-board engine turns only after authoritative confirmation

`GAME-10` External-board Engine Match treats the readboard target as the authoritative game. An engine result and a successful sidecar command acknowledgement create a Pending External Turn but do not advance the current game; Next commits the turn only when the current session/turn identity receives the exact authoritative successor position. This rule is shared by Final-decision and Leading-candidate move modes.

A missing, stale, divergent, or timed-out confirmation ends the Match without retrying, undoing the external target, or restoring the live session after restart; ordinary `READ-02` synchronization remains the fallback authority. Coordinate moves and explicit `pass`/`resign` commands use the same identity and confirmation boundary.

## Considered options

- **Commit locally before commanding the target.** Lower apparent latency, but placement failure creates two competing game states and requires rollback.
- **Commit on the sidecar acknowledgement.** Avoids local optimism, but an acknowledgement proves command handling rather than the external board's resulting position.
