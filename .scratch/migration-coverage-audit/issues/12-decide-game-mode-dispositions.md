# Choose Dispositions for Game-Mode Capabilities

Type: grilling
Status: resolved
Blocked by: 01, 05

## Question

Given the cross-cutting Entry Point census and [Inventory Game-Mode Capabilities](05-inventory-game-modes.md), which modes and controls are equivalent migrations, deliberate Next redesigns, deferred, abandoned, or implementation-only; how should oversized GAME items split into acceptance-sized claims; and what engine-lifecycle, SGF, persistence, failure/recovery, and native-evidence dependencies must be explicit before a game-mode phase can begin?

## Answer

### Frozen boundary and supersession

Accepted R0-R2 claims keep their original IDs, scopes, evidence, and status. Existing `GAME-01` through `GAME-03` are all Missing, so their prospective boundaries may be replaced rather than preserving an unaccepted PK-shaped split.

This decision supersedes the KataGo-only profile, run, and protocol assumptions in [Choose Dispositions for Foreground Engine and Analysis Capabilities](11-decide-engine-analysis-dispositions.md), plus that ticket's match-session application of abandoned black/white candidate filtering. The role policy below does not restore a granular review preference. Ticket 11's zero-or-one Autoload Default, immutable run snapshots, transactional switch, lane-scoped jobs, cancellation, stale-identity rejection, typed failure, manual recovery, and no-run/job-recovery contracts remain authoritative.

Ticket 15 owns rebuilding `docs/JAVA_CAPABILITY_INVENTORY.md`, `docs/PARITY_MATRIX.md`, and `docs/MIGRATION_PLAN.md` from these decisions. Ticket 16 owns final traceability. This ticket changes no production Rust, TypeScript, Tauri, or React behavior and does not edit those three tracked destination documents.

### Canonical model

- An **Engine Adapter** translates manager lifecycle, game-play, and analysis operations to exactly one engine protocol and declares the capabilities that protocol and run can actually provide.
- An **Engine Profile** is durable adapter-specific launch configuration with a stable identity. Shared fields are name, program, argument vector, and working directory; an adapter owns its additional required assets and settings. Capabilities are not user promises inferred from a generic command string.
- An **Engine Run** is one manager-owned process created from an immutable Engine Profile and capability snapshot. One run speaks one protocol. A KataGo process launched in `analysis` mode and one launched in `gtp` mode are separate runs even when they use the same binary.
- A **Match Session** is the only owner allowed to advance an actual match turn on the current game. At most one Match Session exists globally.
- A **Match Reservation** transactionally binds a Match Session to the current-game generation and all required Engine Runs. It commits only after every profile, capability, exact-position admission, and run-ready check succeeds.
- A **Compute Budget** is a per-engine move limit expressed as max visits when the adapter supports it plus a manager-owned hard wall deadline. It is not a competitive clock.
- A **Competitive Clock** owns authoritative remaining player time and a timeout game result. It remains Deferred and cannot be inferred from a Compute Budget.

Opening an SGF, navigating its tree, and reviewing moves without engine output are engine-independent. One-shot analysis, whole-game analysis, candidates, winrate/PV, ownership, and actual match play are separate adapter capabilities; support for one never implies the others.

### Capability dispositions

| Frozen Capability | Disposition | Decision |
| --- | --- | --- |
| `GM-HUMAN-GENMOVE` and `GM-HUMAN-ANA` | Merged Next redesign | Replace the two GTP-versus-analysis-mode products with one Human-vs-Engine Match Session. It supports a new game and exact continuation from the current legal position, engine/profile selection, human color, human moves, pass, resign, Stop, and a capability-gated live-analysis policy. The adapter, not a second product mode, determines how an engine move is obtained. |
| `GM-PK-SESSION` | Split Next redesign / Deferred remainder | `GAME-03` owns one engine-vs-engine game from an empty or exactly admissible current position, with two immutable profile snapshots, pause/resume, Stop, pass handling, max moves, terminal handoff, and typed failure. `GAME-06` defers batch count and live revision, SGF opening catalogs, sequential/random openings, color exchange, manual intervention, batch folders and durable per-game output, and batch winrate images. |
| `GM-HUMANSL` | Deferred Next redesign | `GAME-08` keeps AI Coach as an independent Match Session after core Human-vs-Engine and SGF handoff are Accepted and a HumanSL-compatible model/profile is proven. It is not a hidden option of `GAME-02`. |
| `GM-MATCH-PASS` | Split successor migration | Match-scoped pass belongs to the active Human or PK session and advances only through its current game/run/job identity. Existing review-board pass remains in domain 03 and gains no match acceptance history. |
| `GM-MATCH-STOP` | Next redesign | Stop cancels every session-owned job, releases the reservation, preserves the last committed board, and returns to review. It does not double as a generic pondering toggle. |
| `GM-CONTRIBUTE` | Cross-domain Deferred redesign | `GAME-09` owns board occupancy and Match Session exclusion. Domain 06 owns service support, credentials, privacy, network progress, and network failure. Scheduling requires both owners' dependencies and an approved supported service. |
| `GM-MATCH-RULES-START` | Shared Next redesign | `GAME-04` owns one game-level start contract for board size, komi, handicap, side assignments, Compute Budgets, and PK max moves. New-game defaults are 19 x 19, komi 7.5, and handicap 0; continuation inherits the exact current position and game-level rules. |

The legacy genmove/analysis selector, GTP-versus-WebSocket/raw timing surfaces, pure-net toggle, and Swing play-mode overlay are Abandoned as product controls. Legacy match autosave is absorbed by normal SGF save plus `APP-04` Current-game Session Recovery. Human countdowns, main time/byoyomi/Fischer/engine-owned clocks, and configurable winrate auto-resign remain Deferred under `GAME-07`.

Legacy ponder, analysis-mode play, and black/white candidate filters become one default-off, session-only live-analysis policy. When the active adapter declares the required analysis capability, candidate visibility is `off`, `human-turn`, `engine-turn`, or `both`; opening live analysis defaults to `both`. The policy follows roles rather than stone colors, is not persisted, and reuses `ANA-04` presentation. Review-mode candidate preferences remain separate.

### Engine adapter and capability boundary

- `KataGoAnalysis` is the default full-capability adapter. It launches `katago analysis`, sends complete-position JSONL queries, chooses the response move with `order = 0` for match play, and supplies whichever structured analysis capabilities its query/response contract declares.
- `GenericGtp` is a first-game-phase adapter for standard game play. It validates its required handshake and commands before readiness, synchronizes only positions it can represent exactly, and uses `play`/`genmove` plus supported board/rules commands. It does not claim candidates, winrate/PV, ownership, selected-node analysis, or whole-game analysis merely because `genmove` exists.
- For a GTP run that supports `time_settings` and `time_left`, the adapter maps the per-move hard deadline to `time_settings 0 <seconds> 1` and the corresponding `time_left` before `genmove`. The manager remains authoritative for the wall deadline. This mapping does not create a user-visible competitive clock, and a deadline remains a typed session failure rather than a timeout loss.
- A current position is admissible only if every participating adapter can reproduce its board size, rules, komi, side to move, setup stones, and move path exactly. Unsupported `AB`/`AW`/`AE`/`PL`, rules, sizes, or protocol commands reject before any board, foreground-run, or durable-default mutation. No setup may be simplified or silently dropped.
- Leela Zero, Zen, or other engine-specific rich-analysis protocols require named adapters with a fixed supported version and controlled fixtures. `ANA-09` keeps those adapters Deferred. A profile and every action surface show the actual capability set; an unavailable action explains the missing capability before invocation.

### Stable Parity Item boundaries

- `ENG-09` — **Multi-backend Engine Profiles and Capabilities**: extend the accepted KataGo-only `ENG-01` boundary without changing its history. One catalog persists stable profile identity, adapter kind, shared executable/argument/working-directory fields, and adapter-owned settings. Save validates the selected adapter contract atomically. A run snapshots the profile and verified capabilities; profile edits remain pending until explicit Restart or switch. Unsupported actions are disabled or rejected before process or current-game mutation.
- `ENG-10` — **Generic GTP Game Adapter**: start a profile, complete a bounded protocol/name/version/list-commands handshake, publish only verified game-play and clock capabilities, synchronize an exactly admissible position, map Compute Budgets, and expose typed command, parse, timeout, exit, and unsupported-capability outcomes. Cancellation and stale run/job/game identities cannot publish a move. Rich-analysis extensions are not part of this item.
- `ANA-09` — **Named Rich-analysis Engine Adapters**: stay Deferred. Each admitted engine/version needs a protocol fixture mapping its candidate, winrate/PV, ownership, streaming, selected-node, and whole-game fields into the existing Next analysis model. Each capability is independently declared; missing fields remain unavailable rather than fabricated.
- `GAME-01` — **Match Ownership and Transactional Start**: one coordinator owns Idle/Starting/Playing/Paused/Ending/Error and the sole current-game Match Session. Start validates durable inputs and exact position, acquires all run snapshots, cancels conflicting analysis jobs, and commits the board/session only after every run is ready for the current transaction. Failure preserves the prior board, dirty state, source path, selected `NodePath`, foreground run, and durable defaults. No second match or ordinary analysis job can enter an active reservation.
- `GAME-02` — **Human-vs-Engine Match**: start a new game or continue the exact current position; bind one human role and one engine run; accept only a legal human move or pass on the human turn; request and publish only the current engine job's move on the engine turn; support explicit resign and Stop; and expose the role-based live-analysis policy only when capabilities permit. Review navigation and structural editing cannot mutate the live match behind the session owner.
- `GAME-03` — **Single-game Engine-vs-Engine Match**: bind two distinct run identities and alternate only current, legal engine responses. New and exact-current-position starts, pause/resume, Stop, double pass, max moves, and per-side typed failure are observable. Pause cancels the in-flight job while preserving the board and side to move; resume creates a new job identity for that turn. Late results cannot move, resume, finish, or write generated information.
- `GAME-04` — **Shared Match Rules and Compute Budgets**: one validated start model owns new-versus-current position, board size, komi, handicap, side/profile assignments, per-side max visits and hard deadlines, and PK max moves. Successful durable-default writes precede visible setting changes; failure preserves the prior durable values. Live turn, pause, job, run, clock, and remaining-budget state never persist.
- `GAME-05` — **Match SGF and Review Handoff**: every committed move updates the current tree and dirty state through the domain-03 current-game owner. Player/profile labels, rules, komi, handicap, and explainable terminal result use standard root metadata; generated session information never overwrites personal `C`. Human resign writes standard `RE`; double pass ends the session and enters `REVIEW-03` scoring/confirmation. Stop, max moves, Compute Budget timeout, and engine/protocol failure return to review without fabricating `RE`. Save/reopen preserves committed data, and `APP-04` recovery restores only review state, never a Match Session or Engine Run.
- `GAME-06` — **PK Batch and Advanced Controls**: stay Deferred until `GAME-03` and `GAME-05` are Accepted. Acceptance requires a session-only ordered batch, SGF opening catalog and deterministic sequential/random selection, color exchange, live revision of remaining count, manual intervention, visible per-game state, and durable completed-game output. Application restart does not restore an unfinished queue; completed files remain durable.
- `GAME-07` — **Competitive Clocks and Automatic Resign**: stay Deferred until retained Human and PK sessions are Accepted and a clock policy is separately approved. Acceptance requires an application-owned remaining-time model, visible clocks, adapter mapping, deterministic pause/resume and timeout result semantics, save/recovery boundaries, and explicit auto-resign thresholds and evidence. It cannot reuse a Compute Budget timeout as acceptance.
- `GAME-08` — **HumanSL AI Coach Match**: stay Deferred until `GAME-01`, `GAME-02`, `GAME-05`, and a compatible HumanSL profile are Accepted. Acceptance requires independent preset/rank/color setup, pass, retry AI, finish/review, exact start rollback, typed AI failure, and teardown. Live analysis consumes `ANA-04`; post-game reporting consumes the review/SGF contracts.
- `GAME-09` — **Contribute Board Session**: stay Deferred until domain 06 admits a supported service, credential/privacy flow, progress/error model, and resource teardown. GAME acceptance covers explicit start/stop, board occupancy, global match exclusion, release on failure/exit, and no restart recovery; it does not duplicate provider/network claims.

Engine process classes, protocol enum names, JSON/GTP parser structure, thread topology, run-table representation, command dispatch, and state-enum implementation are implementation-only. Their observable readiness, capability, admission, cancellation, failure, and stale-publication effects are accepted through the items above.

### Session, persistence, and failure contract

- A Human match reserves one adapter-backed run. PK reserves two run identities; the second may be session-scoped rather than another resident foreground identity. The manager may temporarily hold additional candidate processes during a transaction, but only a committed reservation may advance the board.
- Starting a match cancels both ordinary analysis lanes for every reserved foreground run. No automatic analysis resumes after Stop, failure, switch, or review recovery. A user may explicitly start a new analysis job after the session releases its reservation.
- Persist only stable start defaults: profile IDs, game-level rules, Human color, per-side Compute Budgets, and PK max moves. The live-analysis and role-visibility policy, current turn, pause, run/job identities, and remaining budgets are session state.
- Startup validation or run readiness failure is all-or-nothing. A runtime process, protocol, synchronization, or Compute Budget failure cancels every session job, identifies the failed side/profile, preserves the last committed board, releases the reservation, and returns to review. There is no automatic retry, run resurrection, match resumption, loss result, or stale-event publication.
- Explicit application restart never restores a Match Session. `APP-04` may recover the SGF tree, personal comments, source path, selected `NodePath`, and dirty state; the user must repair or choose engines and invoke Continue from Current Position to start a new session identity.

### Phase and dependency boundary

No GAME implementation phase may begin until `ENG-02`, `ENG-03`, `ENG-04`, `ENG-07`, `ENG-09`, `ENG-10`, `SGF-07`, `APP-04`, `REVIEW-03`, `PREF-01`, and `ANA-04` are Accepted. These prerequisites establish run ownership and switch rollback, stale-event rejection, manual recovery, adapter capabilities, safe current-game ownership, review-only restart recovery, scoring, durable defaults, and capability-gated presentation. Temporary game-only process, tree, preference, or presentation paths are not acceptable substitutes.

Within the first GAME phase, implement `GAME-01`, `GAME-04`, and `GAME-05` before the product sessions; then accept Human `GAME-02`; then accept PK `GAME-03`. Ticket 15 owns the actual phase number and may split the phase only along these dependencies. It must not pull `GAME-06` through `GAME-09` or `ANA-09` forward merely because their interfaces exist.

### Evidence boundary

- Deterministic controlled JSONL and GTP process fixtures must cover profile/capability handshake, exact-position admission and rejection, Compute Budget mapping, all-or-nothing start, one-session exclusion, pause/resume resubmission, Stop, cancellation, process/protocol/timeout failure, and stale run/job/game-generation rejection.
- Deterministic state-transition fixtures must prove that every failed start preserves the complete prior current game and foreground identity, every runtime failure preserves the last committed move and returns to review, and a late engine response cannot move, write metadata, or revive a released session.
- Deterministic SGF fixtures must cover new and continued positions, handicap/setup admission, player/profile labels, pass, resign `RE`, double-pass scoring handoff, generated-information separation from personal `C`, save/reopen, and `APP-04` review-only recovery.
- Deterministic preference fixtures must cover first-use defaults, atomic durable start-default updates, persistence failure preservation, and the non-persistence of live session state.
- Real-engine smoke must prove one Human-vs-Engine move and Stop with KataGo JSONL, a complete two-process KataGo PK start/pause/resume/Stop path, and one supported non-KataGo GTP engine completing handshake, exact position sync, `genmove`, budget mapping, and Stop.
- Native desktop evidence must exercise the visible new/continue/pass/resign/Stop and PK start/pause/resume/Stop paths, capability labels, pre-invocation unsupported-analysis explanation, failure-to-review presentation, save/reopen, and restart recovery without a live session. Fixture-only backend evidence cannot mark a GAME item Accepted.

### Acceptance criteria

- [x] Every frozen game-mode family and shared control has an explicit migration, redesign, Deferred, Abandoned, absorbed, or cross-domain disposition.
- [x] KataGo JSONL, Generic GTP game play, named rich-analysis adapters, capability admission, and one-protocol-per-run semantics are explicit.
- [x] Oversized `GAME-01` through `GAME-03` have acceptance-sized successor boundaries, with Deferred products kept separate.
- [x] Engine lifecycle, current-game/SGF, preferences, recovery, scoring, presentation, and cross-domain dependencies are explicit before GAME work.
- [x] Defaults, durable-versus-session state, failure/recovery, result writing, stale-event rejection, and native evidence are explicit.
- [x] Ticket 15 and Ticket 16 own destination-document rebuilding and final traceability rather than this decision ticket.
