# Choose Disposition for Domain-04 Autoplay Remainder

Type: grilling
Status: resolved
Blocked by: 16

## Question

Ticket 10 kept the claimed `UI-05` fixed-interval main-board toggle, deferred the configurable main-board interval as `REVIEW-05`, and routed candidate-variation, sub-board, and engine-best-move replay from `SGF-03-ADJ-AUTOPLAY` to domain 04 without an ID. Should that remainder become a domain-04 item, a Deferred item with a stable ID, or an exclusion?

## Answer

### Capability boundary

The frozen row combined three different user goals because Swing hosted them in one dialog:

- **Review Autoplay** advances existing moves along the selected continuation. Accepted `UI-05` owns its fixed 800 ms toggle; Deferred `REVIEW-05` owns only a configurable main-board interval.
- **Variation Replay** progressively reveals an active candidate PV without changing the selected SGF node or game tree. Candidate replay and Sub-Board replay are two presentations of this one Capability.
- **Engine Continuation** generates and writes engine moves into the game tree. It is not replay.

The shared domain language is recorded in [`CONTEXT.md`](../../../CONTEXT.md).

### Dispositions

| Frozen behavior | Disposition | Decision |
| --- | --- | --- |
| Fixed-interval main-board Review Autoplay | Frozen Accepted boundary | `UI-05` remains unchanged. `Ctrl+A` continues to toggle Review Autoplay only. |
| Configurable main-board Review Autoplay interval | Deferred boundary | `REVIEW-05` remains unchanged and separate from Variation Replay timing. |
| Candidate-variation and Sub-Board Variation replay | Supported Next redesign | New Missing R4 item `ANA-13` — **Variation Replay**. |
| Show the complete variation before replaying it | Abandoned | Do not preview the full PV and pause before restarting from its first move. |
| Continue from a leaf with an engine best move | Abandoned | Review Autoplay stops at a leaf. |
| Generate an engine best move on every autoplay beat | Abandoned | Review Autoplay never creates moves. Human/engine and engine/engine play remain owned by `GAME-*` Match Sessions. |

### `ANA-13` — Variation Replay

`ANA-13` consumes the active candidate and PV from `ANA-04`; it does not select candidates, start Analysis Jobs, or own candidate visibility. It presents the same replay prefix on the main-board candidate variation and on a visible `ANA-12` Sub-Board in Variation mode. Raw never draws a PV. `PREF-01` supplies atomic durable storage and the categorized Preferences surface only.

Observable contract:

- One persisted enablement, one replay progress, and one active timer drive every eligible visible surface. First use is off.
- `显示 → 变化回放` is the checked entry point. `设置 → 首选项 → 分析` exposes one independent interval. There is no shortcut; `Ctrl+A` remains `UI-05`.
- The interval defaults to 500 ms and accepts 100–5000 ms. A successfully persisted live change applies to the next gap without resetting the current replay.
- Replay identity is the selected `NodePath`, active candidate move, and complete ordered PV move sequence. Changing any of those starts a new replay. Visits, winrate, score, and other statistics do not change the identity.
- A new identity reveals its first move immediately, then reveals one additional move per interval. At the end, the full PV remains visible, the timer stops, and enablement remains armed for the next identity.
- When no visible surface can present the active PV, timing pauses and the current prefix is retained. If an eligible surface returns with the same identity, replay resumes from that prefix. If the PV is absent, enablement remains armed without a timer.
- Disabling replay stops timing and immediately restores ordinary full-PV presentation. It never hides the PV or leaves a truncated prefix.
- An explicit enablement or interval change becomes visible only after its atomic `PREF-01` write succeeds. Failure reports the persistence error and preserves the prior durable value and replay state. Missing storage loads the first-use defaults.
- Replay never changes the selected `NodePath`, current-game tree, dirty state, engine run, or Analysis Job. It does not recreate Java's separate candidate/Sub-Board threads, seconds-versus-milliseconds fields, full-PV preview pause, end-of-PV busy loop, or Engine Continuation.

### Evidence, remaining gap, acceptance, and phase

Current repository evidence provides identity-scoped active candidates and PV presentation through `ANA-04`, plus the Sub-Board widget that `ANA-12` will switch between Variation and Raw. `AppChrome` has the existing `显示 → 自动播放(Ctrl+A)` action for `UI-05`, but no Variation Replay control or interval. There is no shared replay state or timer.

Acceptance requires:

1. Deterministic repository tests with a controlled clock cover first-use off, atomic enablement and interval persistence, the 500 ms default and 100–5000 ms validation, immediate first move, one-move cadence, exact identity resets, statistics-only updates, synchronized main-board/Sub-Board prefixes, Raw suppression, pause/resume with no eligible surface, end-of-PV armed idle, disable-to-full-PV, and live speed changes without a reset.
2. Controlled or real KataGo evidence on a branching SGF supplies an identity-valid active candidate whose PV can change while statistics also update.
3. Native R4 smoke covers `显示 → 变化回放`, the Analysis preference, synchronized main-board and Sub-Board Variation replay, Raw, restart persistence, and proves unchanged `NodePath`, dirty state, current-game tree, engine run, and Analysis Job.

`ANA-13` depends on `ANA-04` and `ANA-12`. It belongs to R4 after `ANA-12` and becomes an R4 exit criterion. `PREF-01` is the persistence mechanism, not an R4 start or exit gate.

This ticket changes no production Rust, TypeScript, React, or Tauri behavior. No ADR is added: the reversible migration disposition and acceptance contract are explicit here.
