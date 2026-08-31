# Choose Disposition for Readboard GMA

Type: grilling
Status: resolved
Blocked by: 16

## Question

Ticket 13 deferred the `CAP-06-READBOARD` GMA remainder to domains 04/05 without an ID. Should GMA become a named analysis or match Parity Item, a Deferred item with a stable ID, or an exclusion from Next readboard? One-way sidecar synchronization in `READ-02` is not GMA.

## Answer

### Disposition and ownership

- Retain the capability as an independent Match capability, not an Analysis Item and not an expansion of `READ-02`.
- Canonical term: **External-board Engine Match（外部棋盘引擎对局）**. `GMA` remains a legacy protocol token, not the product term.
- Retain both legacy engine play-back semantics under that capability: **Final-decision Move Mode（最终决策落子模式）** and **Leading-candidate Move Mode（首选候选落子模式）**.
- Represent the capability as one Parity Item with both move modes required for acceptance, sharing one Match Session lifecycle and failure contract.
- Assign stable ID `GAME-10` with disposition Deferred; do not add it to an existing numbered migration phase or expand the R10 `READ-02` exit.

### Admission and request lifecycle

- Admit Engine Profiles per requested move mode: a profile may support one or both modes; `GAME-10` acceptance must prove both modes but does not require every profile to support both.
- Start only by transactionally promoting an active `READ-02` ongoing-sync session with a last-good exact position, trusted side to play, and no pending local placement; static previews, one-shot imports, and ordinary local games are not eligible.
- Select the move mode and its thresholds per external engine-turn request; an unsupported request is rejected without advancing the game or ending the active Match Session.

### External-authoritative turn transaction

- Treat one engine turn as a single **Pending External Turn（待确认外部回合）** after `place`; neither engine output nor sidecar ACK advances the Match. Commit and append the engine `MOVE` only when `READ-02` supplies the exact authoritative successor snapshot.
- On `place` failure, confirmation timeout, sidecar disconnect, or any non-matching authoritative snapshot, terminate `GAME-10` without retry or external rollback; preserve `READ-02` at its healthy or disconnected last-good state and require explicit Match restart.
- Stop closes admission immediately and gives an already-sent external move one bounded confirmation window; commit it if its exact successor snapshot arrives, otherwise terminate under the same failure rule.
- Require every `GAME-10` target to declare bidirectional semantic `pass` and `resign` capabilities, commands/events, and authoritative confirmation; never encode either as a coordinate click.
- While no engine move is pending, keep the same Match only for an unchanged heartbeat or one strictly proven legal external `MOVE`/semantic `PASS`; append that turn. Setup edits, corrections, multi-move jumps, ambiguous turn, board-size changes, or rules changes terminate `GAME-10` and fall back to ordinary `READ-02`.
- Allow exactly one **Armed Engine Turn（已登记引擎回合）** to arrive before the qualifying external turn. Its requested color must match authoritative side to play and cannot override it; a second request or policy change must explicitly cancel the armed request rather than silently replace it.
- Let the first admissible external engine-turn request transactionally start `GAME-10`; expose active Match state and Stop in Next, but require no separate in-app arming action. A rejected start leaves `READ-02` unchanged.
- Allow per-turn cancellation before `place`: clear an Armed request directly, or cancel/discard computation and restore the engine exactly. Keep the Match only after proven restoration; once `place` is sent, use the bounded pending-turn rule instead.
- On engine error or timeout before `place`, never auto-retry. Fail only that request and keep `GAME-10` when exact engine restoration succeeds; otherwise terminate the Match and preserve `READ-02`.

### Move-mode limits

- Make each request limit set self-contained. Final-decision requires at least one positive `maxTime` or `maxVisits`; Leading-candidate requires at least one positive `time`, `totalVisits`, or `leadingCandidateVisits`, with multiple conditions combined as OR. Missing/zero disables that condition, negative values or an all-disabled request are rejected, and no runtime or toolbar value carries over.
- In Leading-candidate mode, commit the rank-one candidate at the instant the first limit fires; random-opening and global candidate-selection preferences do not affect `GAME-10`.

### Protocol, terminal, presentation, and evidence

- Require `READ-01` readiness to declare the target capabilities needed by the requested mode and require a shared session/turn identity across armed request, semantic command, ACK, and authoritative confirmation; ignore stale identities after cancellation, restart, or turn completion.
- Treat one confirmed `pass` as a committed turn; complete `GAME-10` on the second consecutive confirmed pass or a confirmed `resign`, release the engine, and leave `READ-02` active. Record a known resign winner, but do not invent an `RE` or run scoring after two passes without an authoritative result.
- In Next, visibly expose current move mode, Armed/thinking/Pending state, completion or last failure, and Stop on the Match status surface. Sidecar presentation is supplemental; these states do not produce `GUIDE-01` tips or re-own `READ-02` focus/sound preferences.
- Require both deterministic repository evidence and Installed Live Evidence before promotion from Deferred: prove both modes and all state/failure contracts in-repo, then exercise the real sidecar/external target on every platform where `READ-01/02` admits the capability.

## Inherited constraints

- `GAME-01` remains the only Match reservation/turn-ownership gate; at most one Match Session exists globally, and ordinary analysis cannot advance the game.
- Graceful shutdown uses the application-owned bounded stop path. Recovery may preserve only the committed current game; it never restores or auto-restarts `READ-02`, `GAME-10`, an Armed Engine Turn, a Pending External Turn, an engine process, or a sidecar session.
- `GAME-10` inherits platform admission from `READ-01/02`; it creates no independent platform promise.

## Traceability updates

- `CONTEXT.md` defines External-board Engine Match, both Move Modes, Armed Engine Turn, and Pending External Turn.
- `docs/PARITY_MATRIX.md` adds Deferred `GAME-10`, includes it in the Deferred phase map, and removes the readboard GMA no-ID gap.
- `docs/MIGRATION_PLAN.md` adds the dependency edge and Deferred admission contract and keeps `GAME-10` outside the R10 exit.
- `docs/JAVA_CAPABILITY_INVENTORY.md` maps the full `CAP-06-READBOARD` engine-play-back remainder to `GAME-10` and removes its open decision-gap row.
- [ADR 0004](../../../docs/adr/0004-external-board-turns-require-authoritative-confirmation.md) records the authoritative confirmation boundary and rejected optimistic/ACK commit alternatives.
- No production behavior is implemented by this decision ticket.
