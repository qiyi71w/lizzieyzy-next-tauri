# Migration Plan

## Goal

LizzieYzy Next remains the Tauri 2, Rust, and TypeScript product already implemented in this repository. Migration means reproducing or deliberately replacing observable behavior from the Java/Swing maintenance line; it does not mean translating Java source, preserving Swing internals, importing Java Git history, or restarting the Tauri implementation.

The migration is planned as complete user workflows:

1. Open an SGF, navigate and edit its tree, then save the edited game.
2. Review a game in the core desktop UI without requiring an engine.
3. Configure, start, switch, and stop a foreground KataGo engine safely.
4. Run interactive and whole-game analysis against the selected tree node.
5. Replace the current game safely, shut down, recover the current game, and persist owner-routed settings.
6. Author SGF setup, tree structure, metadata, and markup, and review with scoring, try-play, and board display controls.
7. Persist one adaptive workspace, window geometry, appearance, and contextual guidance.
8. Admit multi-backend engine profiles and a Generic GTP game adapter.
9. Run Human-vs-Engine and single-game engine-vs-engine matches.
10. Preview, import, or synchronize games through Yike, Fox, Tencent kifu, and readboard.
11. Install, update, and release Canonical Artifacts on independently admitted platforms.

Each workflow crosses the Rust domain, Rust wire DTO, Tauri gateway, TypeScript API wrapper, React state, UI, and focused evidence that it actually needs. A layer-only milestone is not a migrated workflow.

## Sources Of Truth

- [JAVA_CAPABILITY_INVENTORY.md](JAVA_CAPABILITY_INVENTORY.md) is the exhaustive capability census: Entry Points, defaults, persistence, failure behavior, source references, and runtime-check status against Migration Baseline v1.
- [PARITY_MATRIX.md](PARITY_MATRIX.md) owns item-level disposition, status, evidence, remaining gap, and acceptance.
- This document owns phase membership, the dependency graph, order, and exit criteria. It does not restate item-level evidence.
- [ARCHITECTURE_NEXT.md](ARCHITECTURE_NEXT.md) owns architecture, module boundaries, DTO conventions, and command boundaries.
- [JAVA_BASELINE.md](JAVA_BASELINE.md) fixes the Java behavior reference at Migration Baseline v1 (`7b4027531c2b26062d0bfc27a040cc550cfbea4d`). The Next inventory baseline for this audit is `18c6d189b8b01069975c4c40ead63a010249cb8c`.
- [DEVELOPMENT.md](DEVELOPMENT.md) and [RELEASE_CHECKLIST.md](RELEASE_CHECKLIST.md) own environment smoke and release procedures.

Later Java `main` changes do not automatically change this plan. Data, SGF, Go-rule, and engine-correctness defects must be assessed; UX changes are judged individually; Swing-only implementation changes are ignored. Baseline changes follow the successor policy in `JAVA_BASELINE.md`.

Abandoned and Swing-only capabilities remain exclusions in the inventory and matrix. They are not phase work.

## Evidence Model

Progress is parity-driven, not percentage-driven.

| Evidence class | What it proves | What it does not prove |
| --- | --- | --- |
| Repository evidence | A focused fixture, test, build, or exercised command path proves deterministic behavior in this repository. | Real provider sessions, sidecar installation, native file dialogs, packaged asset paths, platform signing, or installer behavior. |
| Native/provider live evidence | A recorded native desktop or external-environment smoke proves the named path under named conditions. | General correctness outside those conditions, repository regressions, or release publication. |
| Repository Release Evidence | Envelope, channel, version, manifest, updater-state, helper, validator, and non-mutating unsigned dry-run proof of contract and artifact shape. | Production publication, signing identity, or installed behavior. |
| Release-environment evidence | The exact tag and commit, CI run, produced Canonical Artifact, channel feed, signing identity/status, Windows timestamp verification, and macOS notarization/stapling result. | Workflow configuration, the presence of secret names, or an installed product. |
| Installed Live Evidence | A recorded exercise of one Canonical Artifact on its target OS: trust state, install or unpack, primary launch, data location, update or handoff, failure recovery, and uninstall or delete. | Build success, repository validation, or another platform's admission. |

An item is accepted only when the acceptance statement in `PARITY_MATRIX.md` is met. Wiring a command plus focused offline evidence is valid repository proof. It must not be described as live Fox/Yike/Tencent/readboard, real KataGo, native desktop, updater, signing, or installer proof until that environment column is also satisfied.

Release items keep the three release classes independent. Repository Release Evidence, including unsigned dry-run, is never substituted for release-environment evidence or Installed Live Evidence. Unavailable credentials, machines, feeds, or infrastructure are `Not run`; the item stays `Missing` or `Partial`.

Screenshots prove only visible layout and presentation. Semantic behavior requires fixtures or exercised interactions. A focused test filter that runs zero tests is not evidence.

## Current Next Baseline

The existing Tauri implementation is the migration starting point and must be preserved.

### Repository-Proven Or Implemented Paths

- Rust workspace, Tauri desktop backend, React/TypeScript/Vite frontend, domain crates, structural validator, CI build paths, and golden SGF fixtures.
- Rust-owned current-game state, tree-shaped DTOs, `NodePath`, SGF mutation/serialization, Go-rule validation, and fresh projections for remaining analysis consumers.
- Native SGF Open/Save/Save As, including cancellation and failed-write preservation; browser preview remains explicitly non-authoritative.
- Variation navigation, legal move/pass editing, branch removal, personal comments, and semantic save/reopen without an engine.
- Board, analysis panel, win-rate chart, candidate/PV display, ownership/policy paths, problem markers, mini-board, cache state, and engine setup UI.
- Persisted engine profiles, asset checks, KataGo command construction, one-shot analysis, whole-game analysis, progress, full-game cancellation, timeout/error propagation, and response-turn validation.
- SQLite analysis cache with stable SGF cache keys and lookup/save/delete commands.
- Yike and Fox runtime fetch/import command paths with normalized DTO/error boundaries.
- readboard probe and protocol-line snapshot parsing/preview command paths.
- Release asset/workflow preflight and compile-oriented dry-run paths.

### Known Workflow Gaps

- `UI-02` remains Partial: there is still no evidence that engine events are delivered while a board mutation promise is pending. That residual does not block R3.
- Selecting an engine profile does not create an authoritative Foreground Engine Run, Autoload Default, transactional switch, or typed manual recovery.
- Safe current-game replacement, native file activation, window file-drop dispatch, graceful shutdown, and current-game session recovery are not yet the R5 seams.
- Layout rails are fixed at `228px` and `260px`; splitters, rail visibility, window-geometry reset, and narrow Restore Default are absent.
- Multi-backend profiles, Generic GTP, and Match Sessions are absent.
- Provider/readboard repository plumbing exists, but live sessions remain unvalidated; Yike ongoing sync and Tencent kifu import are unclaimed; readboard snapshots are previewed rather than synchronized into the active game.
- Signing, notarization, production updater apply/handoff/rollback, bundled installed-runtime resolution, and Canonical Artifact Installed Live Evidence remain incomplete.

These gaps are tracked individually in `PARITY_MATRIX.md`. Provider, adapter, game-mode, and release work stay behind their start gates rather than becoming the next migration frontier.

## Architecture Seams

### SGF Tree And Board Editing

`crates/sgf` continues to own SGF tree semantics, mutation, replay, and serialization. `crates/go-core` owns legal board transitions. React must not reimplement either rule set.

The wire cutover starts in `crates/app-model`, then updates the TypeScript copies in `apps/desktop/src/domain`. The shared location value is `NodePath`: a zero-based child-index path from the root (`[]`, `[0]`, `[0, 1]`). It is a location in the current tree, not a hash, fingerprint, or permanent node identity.

Required invariants:

- The DTO can represent every child, supported property, setup state, move, comment, and metadata value needed by the UI.
- React owns cursor and view state only. Rust remains authoritative for parse, legal edit, replay, and serialize operations.
- A mutation returns the resulting current tree data and a valid cursor path.
- Open creates the current game representation; Save serializes that current representation instead of the original input string.
- Personal comments have a distinct field and edit path from generated analysis or engine-game information.
- All callers move to the tree-shaped DTO in the same cutover; the flat mainline-only product path is removed when no longer used.

### Safe Current-Game Replacement

`SGF-07` is the only in-application replacement seam. Every file, clipboard, GIB, recent, New Document, native activation, drop, and provider/readboard import parses and validates a candidate before the current-game owner commits it. Parse failure, cancelled Save / Discard / Cancel, failed Save, or owner rejection preserves the complete current game: tree, selected `NodePath`, source path, dirty state, review presentation, and recent-file state. Success atomically installs one candidate and clears stale generated presentation.

`APP-01` and `APP-02` dispatch OS activation and drops into this seam. They do not duplicate replacement semantics. Domain-06 import and sync commit through the same seam.

### Desktop Interaction

`BoardCanvas` should emit coordinate, pass, and hover intent through explicit component callbacks. The owning feature state decides whether the intent navigates, edits, or previews. Components continue to call wrappers in `apps/desktop/src/api`; no raw Tauri `invoke` is added to UI components.

Candidate and PV state is scoped to the selected `NodePath` and analysis job identity. Changing node, game, or job clears stale hover and candidate state before new results are presented.

`APP-05` owns one Shortcut Registry: actions, primary keys, visible non-conflicting Java aliases, conflict checks, labels, and the Help / `?` reference. Per-action semantics remain with their owner domains and do not expand accepted `UI-05`.

### Foreground Engine And Analysis Jobs

`crates/engine-manager` becomes the sole owner of:

- Adapter-backed Foreground Engine Run identity and process lifecycle.
- No-engine, starting, ready, switching, stopping, and error state.
- Immutable adapter, profile, and capability snapshots for the current run.
- The zero-or-one Autoload Default persisted with the profile catalog.
- Monotonic switch tokens and failed-switch rollback.
- Selected-node and whole-game analysis job identity, cancellation, and stale-event rejection.

Tauri remains a command/event adapter. React observes lifecycle and job snapshots; it does not infer process identity from the selected profile. Engine Settings edits the catalog and Autoload Default without starting or switching a run. The main-workspace Engine Switcher starts or switches the run.

R3 must at least prove the `KataGoAnalysis` path. `ENG-09` and `ENG-10` extend the catalog and Generic GTP game adapter in R8; their interfaces must not pull that work into R3.

For A → B switching:

1. Keep A authoritative while B starts and completes its handshake.
2. Promote B only after readiness is proven for the current switch identity.
3. Cancel A-owned jobs, bind new work to B, then stop A.
4. If B fails, keep A as ready and primary. If A no longer exists, yield the explicit no-engine state rather than promoting B.
5. Ignore completion or events whose switch token is no longer current.

Unexpected exit publishes a typed error, cancels that run's jobs, and waits for explicit Restart. Analysis jobs use the current run only when its adapter declares the required capability. A game-only Generic GTP run leaves unsupported analysis actions unavailable instead of starting a hidden dedicated engine.

All analysis callers migrate to the manager-owned lifecycle before the raw profile-to-process path is removed. No compatibility wrapper remains after the cutover.

Foreground Engine Runs, capability snapshots, Analysis Jobs, Match Sessions, and provider sessions never persist across application restart. `APP-04` restores only the current game.

### Durable Preferences And Owner-Routed Settings

`PREF-01` owns the native durable-preference mechanism and one categorized Preferences surface plus contextual entry points. A missing file loads defaults. An unreadable file is isolated, defaults are loaded, and the recovery is reported. An explicit setting changes visible state only after an atomic write succeeds; failure preserves the last durable value and reports the error. Java `config.txt` is never imported.

Semantic effects stay with their behavior owners. `PREF-01` does not claim engine autoload, analysis presentation, board display, sound, scoring, layout, window geometry, appearance, guidance, match rules, provider locators, or update-channel behavior.

### Adaptive Workspace

Next-native preferences own panel proportions. The fixed left/right rail widths become user-adjustable layout state with practical minimums that preserve the board as the primary surface.

Restore Default is intentionally narrow: it resets panel sizes and board proportion only. It does not change rail visibility, window position/size, engine settings, theme, guidance, or unrelated preferences.

### Match Sessions

Start inside `engine-manager` as a small internal coordinator. Do not create a new crate unless real dependency pressure appears.

Public behavior is one global Match Session bound by a Match Reservation that commits only after every profile, capability, exact-position admission, and run-ready check succeeds. Human-vs-Engine and single-game engine-vs-engine matches share `GAME-04` start rules and Compute Budgets. Generated session information never overwrites personal `C`. Double pass hands off to `REVIEW-03`; Stop and protocol failure return to review without fabricating `RE`.

### Providers And readboard

Keep `provider_fetch_yike`, `provider_fetch_fox`, `readboard_sidecar_probe`, and `readboard_sidecar_sync_snapshot` and their typed error boundaries. R10 completes preview, one-shot import, and ongoing synchronization on top of `SGF-07`.

Preview never mutates the current game. One-shot import fetches and parses first, then replaces once through `SGF-07`. Ongoing synchronization starts only from explicit Start sync or Play & Sync; the external source is then authoritative until stop. Play & Sync opens the public Yike room in the system browser and starts matching Next read-only sync; Next does not read browser authentication; stopping Next sync does not close the browser.

Image OCR remains unsupported unless an OCR-capable runtime, explicit parity item, and live evidence are added. Preview-only readboard sync is not complete import parity.

### Release And Installed Runtime

Platform admission is independent. Canonical Artifacts are Windows NSIS and portable, macOS DMG, and Linux AppImage. Extra CI formats stay validation artifacts until a later disposition promotes them.

Installed variants use OS app-data and preserve it by default on removal. Windows portable uses package-local `user-data/`; deleting that directory is destructive. Packages start in No-engine Mode with `app-core` only. Engine Settings acquires signed KataGo backend and default-model components as Managed Installed Components; domain 04 owns the resulting profile and engine-start behavior.

## Dependency Graph

Phase numbers group work and express delivery priority. They are not the dependency graph. R10 and R11 may proceed in either order after their own start gates. Other later phases may also become dependency-legal beside a lower number once that number's hard edges are satisfied; they still must not be pulled into the named Next Executable Batch.

### How To Choose The Next Slice

1. If a Next Executable Batch is named below, do that slice. Do not mix later-phase work into it.
2. Otherwise pick an incomplete active item whose every named start dependency is Accepted. A named gate may be passed even when its cross-phase item is not yet Accepted. Final-acceptance gates are not start dependencies for that item's earlier semantic work.
3. Prefer the lowest-numbered incomplete phase, except R10 and R11 may proceed in either order after their own start gates.
4. Deferred queue items are never phase exits and are never the next slice unless a later plan revision starts them.

```mermaid
flowchart TB
  R2[R2 Core Review UI - done]
  R3[R3 Foreground Engine]
  R4[R4 Analysis]
  R5[R5 Safe Current Game]
  R6[R6 SGF Authoring / Review]
  R7[R7 Adaptive Workspace]
  R8[R8 Engine Adapters]
  R9[R9 Game Modes]
  R10[R10 Providers / readboard]
  R11[R11 Release]

  R2 --> R3
  R3 --> R4
  R2 --> R5
  R3 --> R8
  R5 --> R6
  R5 --> R7
  R5 --> R10
  R5 --> R11
  R4 --> R9
  R5 --> R9
  R6 --> R9
  R8 --> R9
```

Layout, window, and appearance in R7 wait only for `PREF-01`. `GUIDE-01` has no R4 hard edge: Ticket 09 named a domain-04 auto-analyze education path that Ticket 11 later Abandoned as `CAP-04-ANA-08`. That contradiction is a Ticket 16 disposition-to-item gap, not an R4→R7 dependency.

### Critical Edges

| Dependency | Dependent | Contract |
| --- | --- | --- |
| `ENG-02` | `ENG-03`, `ENG-04`, `ENG-05`, `ENG-06`, `ENG-07` | Manager-owned run identity exists before switch, jobs, autoload start, and recovery. |
| `ENG-03` | `ENG-04` | Failed switch and stale rejection are the failure mode of the same transaction. |
| `ENG-02`, `ENG-05`, `ANA-03` | `ANA-01`, `ANA-02` | One-shot and whole-game work use independent manager-owned lanes. |
| Identity-valid `ANA-01` / `ANA-02` results | `ANA-04` | Presentation consumes only current run/job/node results. |
| `ENG-09` | `ENG-10` | Multi-backend catalog and capability snapshot before Generic GTP. |
| `SGF-07` | `APP-01` semantic gate, `APP-02`, `SGF-08`, `SGF-09`, `SGF-10`, R10 import/sync | One replacement seam. Semantic `APP-01` work starts from this seam only. |
| `APP-01` R5 semantic gate | `APP-02` | One-file drop follows the recorded semantic open/replace contract, never final `APP-01` Accepted. |
| `PREF-01` | `SGF-09`, `REVIEW-03`, `REVIEW-07`, `REVIEW-08`, `LAYOUT-02`, `LAYOUT-04`, `WINDOW-01`, `APPEAR-01`, `GUIDE-01`, `GAME-04`, R10 recents | Durable write mechanism only. |
| `APP-03` | `REL-06` | Windows helper launch is followed by Safe Graceful Shutdown. |
| `REL-02` | `REL-03`, `REL-05` | Feeds and managed components consume trusted artifacts. |
| `REL-03`, `REL-05`, `APP-03` | `REL-06` | Windows download/apply. |
| `REL-03` | `REL-07` | macOS/Linux package handoff. |
| `REL-06` | `REL-08` | Rollback is the failure path of Windows apply. |
| `APP-01` R5 semantic gate plus `REL-04` Installed Live association | `APP-01` Accepted | Split graph: semantic gate is R5; `REL-04` is the additional R11 final-acceptance gate only. Not a start dependency for `APP-01` semantic work or `APP-02`. `APP-01` still owns open/replace semantics. |
| `PROV-01` | `PROV-03` | Public locator/center before ongoing Yike sync. |
| `READ-01` | `READ-02` | Sidecar readiness before ongoing sync. |
| `GAME-01`, `GAME-04`, `GAME-05` | `GAME-02` | Ownership, rules, and SGF handoff before Human-vs-Engine. |
| `GAME-02` | `GAME-03` | Human session before single-game PK. |

### APP-01 Cross-Phase Gate

`APP-01` is the only active item split across two phases. Semantic-gate order and final-acceptance order are separate graphs.

```mermaid
flowchart LR
  SGF07[SGF-07]
  APP01sem[APP-01 R5 semantic gate]
  APP02[APP-02]
  REL04[REL-04 association evidence]
  APP01acc[APP-01 Accepted]

  SGF07 --> APP01sem
  APP01sem --> APP02
  APP01sem --> APP01acc
  REL04 --> APP01acc
```

- **R5 semantic gate:** cold and warm `.sgf` / `.gib` activation focuses one window and uses `SGF-07`. Save / Discard / Cancel cancellation or failure preserves the current game. This gate may be recorded as passed while `APP-01` is not yet Accepted. `APP-02` one-file drop follows this gate, never final `APP-01` Accepted; multiple files route to deferred `ANA-07` intake and never silently replace the current game.
- **R11 association gate:** `REL-04` supplies per-artifact file association and installed-app launch evidence. It is the additional final-acceptance gate only. Packaging does not own open/replace semantics.
- `APP-01` stays unaccepted until both gates pass. R5 may exit with the semantic gate recorded and `APP-01` still unaccepted.

Ordinary launch evidence stays on accepted `UI-01` / `UI-04` plus `REL-04` installed launch. It is not a second shell item.

### GAME Admission Gate And Internal Order

R9 must not begin until all of the following are Accepted: `ENG-02`, `ENG-03`, `ENG-04`, `ENG-07`, `ENG-09`, `ENG-10`, `SGF-07`, `APP-04`, `REVIEW-03`, `PREF-01`, and `ANA-04`. Temporary game-only process, tree, preference, or presentation paths are not substitutes.

Inside R9: implement `GAME-01`, `GAME-04`, and `GAME-05` first; then `GAME-02`; then `GAME-03`.

### R10 / R11 Overlap

R10 and R11 numbers are delivery priority, not a hard edge between them. Either phase may start after its own start gate:

- R10 start gate: `SGF-07`, `PREF-01`, and `APP-03` Accepted.
- R11 start gate: repository release work may proceed at any time for `REL-01`. End-user apply/handoff requires `APP-03` for `REL-06`. `REL-04` association evidence requires the `APP-01` semantic gate. Trusted feeds require `REL-02`.

The only product coupling between the two phases is whether domain 06 later accepts a readboard payload into the `REL-05` installed manifest. That decision is not an R10 or R11 exit.

### Owner-Routed Settings

| Observable setting | Owner item | Phase |
| --- | --- | --- |
| Durable file, atomic write, categorized Preferences surface | `PREF-01` | R5 |
| Autoload Default, first-use off | `ENG-06` | R3 |
| Identity-scoped candidates, PV mini-board, ownership, policy, and candidate-limit rendering | `ANA-04` | R4 |
| Dirty-state replacement confirmations | `SGF-07` | R5 |
| Restore last session at launch, default off | `APP-04` | R5 |
| Shortcut primary keys and aliases | `APP-05` | R5 |
| Recent SGF/GIB list and clear-history | `SGF-09` | R6 |
| New Document board size and default komi | `SGF-10` | R6 |
| Editable player names and komi | `SGF-13` | R6 |
| Scoring rule, area default | `REVIEW-03` | R6 |
| Coordinates default on, move numbers default off | `REVIEW-07` | R6 |
| Review move sound, default on | `REVIEW-08` | R6 |
| Rail visibility | `LAYOUT-04` | R7 |
| Window geometry and reset | `WINDOW-01` | R7 |
| Classic / High Contrast appearance | `APPEAR-01` | R7 |
| Educational tip dismissal and Reset Guidance | `GUIDE-01` | R7 |
| Adapter kind and adapter-owned profile fields | `ENG-09` | R8 |
| Match start defaults, Compute Budgets, PK max moves | `GAME-04` | R9 |
| Provider locators, recents, sync interval, mute-during-sync | `PROV-01` … `PROV-04`, `READ-02` | R10 |
| Update channel and source | `REL-03` | R11 |
| Complete locale selection | `I18N-01` | Deferred |
| Contribute entry visibility | `GAME-09` | Deferred |

Update networking uses the OS proxy. Provider networking remains a domain-06 decision on those owner items.

### Traceability Open Gaps

These records have a final owner-route or conflict but no supported Parity Item ID from Tickets 08–14. They match the inventory Ticket 16 ledger. They are not R3–R11 members, not R4 or R6 exit criteria, and not Deferred-queue items. This plan does not invent IDs for them. Ticket 16 must open a concrete decision ticket before any of them can be scheduled.

| Gap | Census pointer | Why it is not scheduled |
| --- | --- | --- |
| Owner-routed next-move marker; not `ANA-04`/`UI-01`/`PREF-01` | SET-NEXT-MOVE | 09 → domain 04; 10–14 named no ID. Not `ANA-04`. |
| Same next-move cluster | SGF-03-ADJ-NEXT-HINT | 10 → domain 04; 11 named no ID. Not `ANA-04`. |
| Graph perspective/lines/blunder-bar. Chart *presence* in `UI-01` is not this Capability | SET-WINRATE-GRAPH | 09 → domain 04; 10–14 named no ID. Not `ANA-04`. |
| Mini-board *mode* (variation/raw/heatmap). Mini-board *presence* in `UI-01` and PV mini-board in `ANA-04` are not this Capability | SET-SUBBOARD | 09 → domain 04; 10–14 named no ID. Not `ANA-04`. |
| Large-sub / large-WR / append-WR / names-on-board / always-on-top. Do not expand `UI-01` | SET-MAIN-PANEL extras | 09 → domain 03; 10 mapped hints/coords/move-numbers only. Accepted `UI-01` and personal-comment `SGF-05` scopes cannot absorb these controls. |
| Record both the Ticket 09 educational-tip route and Ticket 11 abandonment of automatic current-game analysis. Do not bind `GUIDE-01` to the abandoned path or substitute an unconfirmed tip | SET-HINT-AUTOANALYZE vs CAP-04-ANA-08 | 09 routed the tip to `GUIDE-01`/domain 04; 11 Abandoned `CAP-04-ANA-08`. No replacement producer has been approved. |
| Provider-network proxy has no Parity ID | SET-NETWORK-PROXY (06 edge) | 09 → 06/07; 14 mapped the update edge to OS proxy; 13 did not decide provider proxy. |
| SSH/remote compute owner-routed to 06 without an ID | CAP-04-ENG-01 SSH/remote remainder | 11 → domain 06; 13 created no remote-compute item. |
| `GAME-09` is the GAME mapping; 06 service/credentials/privacy/network remains without an ID | GM-CONTRIBUTE domain-06 service half | 12 assigned `GAME-09` plus 06 service ownership; 13 did not name a contribute-service item. Not `GAME-09`. |
| One-way sync is `READ-02`; GMA has no Parity ID | CAP-06-READBOARD GMA remainder | 13 deferred GMA to 04/05 without an ID. Not `READ-02`. |
| Public modes are `PROV-01`/`PROV-03`; personal/auth has no successor ID | CAP-06-YIKE-LIVE-CENTER personal/auth | 13 deferred personal/private/auth-required Yike read without a successor ID. Not `PROV-01`/`PROV-03`. |
| `EXPORT-02` is review-board only; analysis-panel images have no domain-04 ID | SGF-03-ADJ-SAVE-MORE analysis-panel images | 10 left analysis-panel image exports to domain 04 without an ID. Not `EXPORT-02`. |
| `UI-05` toggle and `REVIEW-05` interval are named; candidate-variation / sub-board / engine-best replay have no domain-04 ID | SGF-03-ADJ-AUTOPLAY domain-04 remainder | 10 routed that remainder to 04 without an ID. Not `UI-05`/`REVIEW-05`. |

Ticket 16 must close these gaps with explicit decision tickets. Until then, they remain visible but unscheduled; `GUIDE-01` cannot be Accepted by substituting an unapproved tip.

## Roadmap

### R0 — Baseline And Inventory

**Goal:** Make the migration target immutable and progress auditable.

Deliver:

- Migration Baseline v1 with post-baseline governance.
- Stable parity item IDs with repository/live evidence columns.
- A fixture inventory mapping frozen behavior to existing or missing Next evidence.
- Explicit deliberate divergences rather than accidental omissions.

Exit when:

- `BASE-01` is accepted.
- Every active migration claim maps to a parity item.
- The first R1 fixture batch is named and has deterministic expected behavior.

Current state: the baseline and initial matrix are established; maintaining the inventory continues with each slice.

### R1 — SGF And Board Semantics

**Goal:** Complete the local open → navigate → edit → save workflow before more integration work.

Order:

1. Convert the frozen `#317` behavior set into minimal cross-language SGF fixtures and focused Rust expectations.
2. Add the tree-shaped Rust DTO and `NodePath`, then update TypeScript copies and every caller.
3. Replace mainline-only React state with current-game plus cursor/view state.
4. Add parent/child/sibling variation navigation and position replay.
5. Add legal board move/pass editing, branch creation/removal, and personal-comment editing through Rust domain operations.
6. Save the current edited tree and prove semantic reopen.

Exit when:

- `SGF-01` through `SGF-06` and `RULE-01` are accepted.
- A native no-engine smoke opens a branching SGF, navigates branches, edits a move and comment, saves, reopens, and observes the same tree state.
- No product path still treats the first-child move list or original `sgfText` as the authoritative current game.

Current state: accepted through Tickets 01–09. The Windows native run covered launch, navigation, edit/comment/removal, cancel, Save As/reopen, and ACL-denied Save As. Case 6 passed on PID 73980 using `f2c5896` plus the Windows build fix later committed as `8289391`.

### R2 — Core Desktop Review UI

**Goal:** Make the everyday review surface complete and usable without KataGo.

Deliver:

- Recorded baseline screenshots and same-state Next comparisons under named display conditions.
- Non-blocking board pointer/keyboard intent.
- Complete variation controls and baseline review shortcuts.
- Candidate hover-preview semantics and stale-state clearing.
- Clear engine-unavailable presentation that does not block SGF work.

Exit when:

- `UI-01`, `UI-03`, `UI-04`, and `UI-05` are accepted.
- Any remaining `UI-02` evidence gap is recorded with an explicit migration disposition.
- Native smoke passes both without an engine and with engine-only actions unavailable.
- Visual evidence follows the reference rules in `DESIGN.md`.

Current state: Ticket 07 closeout on native Windows candidate `66c906f` / PID 51344 accepted `UI-01`, `UI-03`, `UI-04`, and `UI-05`. `UI-02` remains Partial solely for engine-event delivery while a board mutation promise is pending. That residual is tracked in `PARITY_MATRIX.md` and is not the next executable slice.

### R3 — Foreground Engine Lifecycle

**Goal:** Replace profile-as-selection with one adapter-backed Foreground Engine Run. R3 is the next executable phase.

**Owns:** `ENG-02`, `ENG-03`, `ENG-04`, `ENG-05`, `ENG-06`, `ENG-07`. Accepted `ENG-01` remains the KataGo analysis profile and asset-check foundation and is not reopened.

**Start gate:** R2 has exited. `UI-02`'s residual engine-event gap is not this gate.

**Order:**

1. `ENG-02` Foreground Engine Run identity and lifecycle (Slice R3-A).
2. `ENG-05`, `ENG-06`, and `ENG-07` after `ENG-02`.
3. `ENG-03` transactional A → B switch.
4. `ENG-04` failed switch and stale-identity rejection.

**Deliver:**

- Manager-owned no-engine / starting / ready / switching / stopping / error snapshot, including an immutable adapter/profile/capability snapshot. The R3 proof path is `KataGoAnalysis`.
- Dedicated Stop and explicit Restart. Stop cancels every run-owned job, then terminates the process. Restart replaces the run from current saved profile data; it does not mutate a live run in place.
- Engine Settings versus Engine Switcher. Settings never start, switch, or mutate a run. Editing the active profile reports pending changes until Restart.
- Autoload Default as zero-or-one marked profile, first-use off, persisted with the catalog. Startup with a mark validates and starts that profile; failure leaves no engine without falling back to another profile. Changing the mark does not change the current run.
- Successful A → B promotion and failed-B rollback. The active profile cannot be deleted.
- Typed start, asset, protocol, nonzero-exit, timeout, and cancellation outcomes scoped to the current run. Unexpected exit waits for manual Restart.
- Manager-owned cancellable selected-node job identity with latest-request-wins publication and stale-result rejection.
- Clean migration of analysis callers away from raw profile process startup.

R3 acceptance is the `KataGoAnalysis` run lifecycle. `ENG-09`, `ENG-10`, analysis product items, Match Sessions, and Deferred engine/analysis items belong to later phases.

**Exit when:**

- `ENG-02` through `ENG-07` are accepted.
- Repository evidence covers start / Stop / Restart, immutable snapshots, Autoload Default persistence, A → B promotion and rollback, active-profile deletion guard, typed failure, cancellation, and stale identities, plus a controlled JSONL selected-node job start / cancel / supersede / stale-reject path.
- Real KataGo smoke proves ready, Stop, Restart, and two-profile switch.
- Native smoke proves no-engine, one-engine, and two-profile switch behavior with real KataGo assets.

### R4 — Analysis

**Goal:** Bind interactive and whole-game analysis to the selected SGF node and the R3 run/job identity.

**Owns:** `ANA-01`, `ANA-02`, `ANA-03`, `ANA-04`. Accepted `ANA-05` remains the SQLite cache foundation and is not reopened.

**Start gate:** `ENG-02` through `ENG-07` accepted.

**Order:**

1. `ANA-03` independent selected-node and whole-game lanes.
2. `ANA-01` and `ANA-02` in parallel on those lanes.
3. `ANA-04` Next-native presentation of identity-valid results.

**Deliver:**

- One-shot analysis of the selected position after adapter capability and exact-position admission, with result, typed failure, explicit cancellation, and supersession.
- One single-stage, mainline-only whole-game session with visible progress, node-linked results, and cancellation that does not block review navigation.
- Lane-local supersession: a new selected-node request does not silently cancel whole-game work. Stop, Restart, switch, crash, and exit cancel both lanes for that run.
- Candidates, PV mini-board, ownership, policy, and candidate-limit rendering only for the current adapter capability and run/job/node scope.
- Pre-invocation rejection of unsupported capabilities. No hidden dedicated analysis engine.

**Exit when:**

- `ANA-01` through `ANA-04` are accepted.
- Repository evidence covers exact-node publication, concurrent lanes, lane-local supersession, progress, explicit cancellation, typed failure, presentation scoping, and unsupported-capability rejection.
- Controlled or real KataGo evidence covers response validation, timeout, stderr error, cancellation, and stale results.
- Native smoke exercises one-shot and whole-game analysis against a branching edited SGF.

`ANA-05` variation-aware expansion, Java in-tree cache semantics, and SGF analysis exchange stay outside this exit.

### R5 — Safe Current Game And Application Shell

**Goal:** Make current-game replacement, shutdown, recovery, and the durable-preference mechanism safe before authoring, workspace, provider, and release apply work.

**Owns:** `PREF-01`, `SGF-07`, `APP-02`, `APP-03`, `APP-04`, `APP-05`, and the `APP-01` semantic gate.

**Start gate:** R1 and R2 have exited. This phase is dependency-legal beside R3, but the named next batch remains R3-A.

**Order:**

1. `PREF-01` durable mechanism and categorized surface.
2. `SGF-07` safe replacement.
3. `APP-03` Safe Graceful Shutdown, `APP-04` current-game recovery, and `APP-05` Shortcut Registry.
4. `APP-01` semantic gate after `SGF-07`. Record the gate as passed here; do not wait for final `APP-01` Accepted.
5. `APP-02` window file-drop dispatch after the recorded `APP-01` semantic gate.

**Deliver:**

- Native durable preferences without importing Java configuration or absorbing owner-domain behavior.
- Parse-before-replace current-game installation, including clipboard paste into `SGF-07`.
- One-file drop follows the passed `APP-01` semantic gate; multiple supported files are explanatory and do not mutate the current game until `ANA-07` is started.
- Shutdown uses Save / Discard / Cancel; cancelled or failed Save aborts exit and leaves resources running. After save or discard, persist owned state and stop owned resources. A teardown timeout names the stuck resource and offers Retry or contextual “Exit anyway.”
- Recovery restores SGF tree, personal comments, current `NodePath`, source path, and dirty state only. Layout persists separately. Engine processes, analysis jobs, Match Sessions, and provider sessions are never resurrected. Normal-launch restore stays default off.
- One Shortcut Registry and searchable Shortcut Reference.

**Exit when:**

- `PREF-01`, `SGF-07`, `APP-02`, `APP-03`, `APP-04`, and `APP-05` are accepted.
- The `APP-01` semantic gate is recorded as passed. `APP-01` itself remains unaccepted pending `REL-04`.
- Native restart smoke proves preference durability and review-only recovery.
- Shutdown smoke proves cancelled save aborts exit and successful teardown stops every currently owned resource.

### R6 — SGF Authoring And Review

**Goal:** Complete in-application SGF intake and the remaining no-engine authoring/review tools on top of `SGF-07`.

**Owns:** `SGF-08` through `SGF-14`, `REVIEW-01`, `REVIEW-02`, `REVIEW-03`, `REVIEW-07`, `REVIEW-08`.

**Start gate:** `SGF-07` and `PREF-01` accepted.

**Order:**

1. `SGF-08` GIB import, `SGF-09` recent kifu, `SGF-10` New Document.
2. `SGF-11` setup editor, `SGF-12` structural tree editing, `SGF-13` metadata, `SGF-14` markup.
3. `REVIEW-01` direct navigation, `REVIEW-02` try-play, `REVIEW-03` scoring, `REVIEW-07` board display, `REVIEW-08` move sound.

**Deliver:**

- Tygem GIB intake through `SGF-07`, then Save As SGF only.
- Deduplicated persisted recents for successful SGF/GIB file opens, reopened through `SGF-07`.
- File New / Clear Board as one untitled document at `NodePath []`.
- Interactive setup, delete/undo/redo/promote/return-to-main, player names and komi, and supported marks.
- Exact `NodePath` selection from the Next tree/list, an isolated try-play sandbox, and area/territory scoring with Confirm Result writing root `RE`.
- Persisted coordinate and move-number booleans, and default-on sound for successful local moves, passes, and forward review navigation.

**Exit when:**

- `SGF-08` through `SGF-14` and `REVIEW-01`, `REVIEW-02`, `REVIEW-03`, `REVIEW-07`, and `REVIEW-08` are accepted.
- Fixtures cover GIB handicap/pass/malformed preservation, New Document cancellation, setup/markup save/reopen, try-play non-mutation, and scoring Confirm Result.
- Native smoke covers GIB open, recent reopen, New Document, scoring confirm, and display/sound persistence across restart.

Accepted `SGF-01` through `SGF-06`, `RULE-01`, `UI-01`, `UI-03`, `UI-04`, and `UI-05` stay at their original scope.

### R7 — Adaptive Workspace

**Goal:** Persist one user-adjustable review workspace without concentrating semantic settings here.

**Owns:** `LAYOUT-01`, `LAYOUT-02`, `LAYOUT-03`, `LAYOUT-04`, `WINDOW-01`, `APPEAR-01`, `GUIDE-01`.

**Start gate:** `PREF-01` accepted. `GUIDE-01` remains blocked by the unresolved producer conflict recorded under Traceability Open Gaps; other R7 items do not wait for that decision.

**Order:**

1. `LAYOUT-01` draggable proportions.
2. `LAYOUT-02` persisted proportions and `LAYOUT-03` narrow restore.
3. `LAYOUT-04` rail visibility, `WINDOW-01` geometry, `APPEAR-01` appearance.
4. `GUIDE-01` only after Ticket 16 supplies a resolved producer disposition.

**Deliver:**

- Draggable left/board/right proportions with practical minimums.
- Debounced atomic layout persistence; save failure keeps the session usable and visibly retryable.
- Restore Default resets panel sizes and board proportion only.
- Left and right rails visible by default, independently collapsible, persisted through `PREF-01`.
- Window geometry validated against current displays; automatic reset only when saved geometry is invalid; explicit reset changes window geometry only.
- Curated Classic default and High Contrast alternate. Chrome and text follow system DPI.
- Dismissible educational tips only, with Reset Guidance. Dirty-state confirmations cannot be permanently disabled. The producer requires an explicit disposition because the originally named auto-analyze workflow was abandoned.

**Exit when:**

- `LAYOUT-01` through `LAYOUT-04`, `WINDOW-01`, and `APPEAR-01` are accepted.
- `GUIDE-01` is accepted only after its producer conflict is resolved; the abandoned automatic-analysis path cannot satisfy it.
- Native restart smoke proves proportions, rail visibility, window geometry, and appearance.
- Reset smoke proves unrelated visibility, engine, guidance, and owner-routed settings are unchanged.
- Guidance smoke covers the approved producer, dismissal, restart persistence, and reset.

### R8 — Engine Adapters

**Goal:** Extend accepted KataGo-only profiles into an adapter catalog and a bounded Generic GTP game adapter before Match Sessions.

**Owns:** `ENG-09`, `ENG-10`.

**Start gate:** `ENG-02` through `ENG-07` accepted. This phase is a GAME prerequisite, not part of R3 or R9.

**Order:**

1. `ENG-09` multi-backend profiles and declared capabilities.
2. `ENG-10` Generic GTP game adapter.

**Deliver:**

- One catalog with stable profile identity, adapter kind, shared executable/argument/working-directory fields, and adapter-owned settings. Save validates the adapter contract atomically. A run snapshots verified capabilities; profile edits remain pending until Restart or switch.
- Generic GTP handshake, exact-position synchronization, Compute Budget mapping, and typed command/parse/timeout/exit/unsupported-capability outcomes. Cancellation and stale run/job/game identities cannot publish a move.
- Unsupported actions disabled or rejected before process or current-game mutation. Rich-analysis extensions are not this phase.

**Exit when:**

- `ENG-09` and `ENG-10` are accepted.
- Repository evidence covers profile/capability handshake, GTP fixtures, exact-position admit/reject, budget mapping, and stale-identity non-publication.
- Real-engine evidence includes one supported non-KataGo GTP handshake, exact position sync, `genmove`, budget mapping, and Stop.
- Native smoke shows capability labels and pre-invocation unsupported-analysis explanation.

### R9 — Game Modes

**Goal:** Add Human-vs-Engine and single-game engine-vs-engine Match Sessions without mixing them into review-state inference.

**Owns:** `GAME-01`, `GAME-02`, `GAME-03`, `GAME-04`, `GAME-05`.

**Start gate:** `ENG-02`, `ENG-03`, `ENG-04`, `ENG-07`, `ENG-09`, `ENG-10`, `SGF-07`, `APP-04`, `REVIEW-03`, `PREF-01`, and `ANA-04` are Accepted.

**Order:**

1. `GAME-01` match ownership and transactional start, `GAME-04` shared rules and Compute Budgets, `GAME-05` SGF/review handoff.
2. `GAME-02` Human-vs-Engine.
3. `GAME-03` single-game engine-vs-engine.

**Deliver:**

- One coordinator owning Idle/Starting/Playing/Paused/Ending/Error and the sole current-game Match Session.
- All-or-nothing reservation: failure preserves the prior board, dirty state, source path, selected `NodePath`, foreground run, and durable defaults.
- Shared new-versus-continue start model. New-game defaults are 19×19, komi 7.5, handicap 0.
- Human match: one human role and one engine run, legal human move/pass, resign, Stop, and role-based live-analysis policy only when capabilities permit.
- PK match: two distinct run identities, pause/resume, Stop, double pass, max moves, and per-side typed failure.
- Committed moves update the domain-03 current game. Generated information stays out of personal `C`. `APP-04` recovery never restores a Match Session or Engine Run.

**Exit when:**

- `GAME-01` through `GAME-05` are accepted.
- Controlled JSONL and GTP fixtures cover handshake, exact-position admission, Compute Budgets, all-or-nothing start, one-session exclusion, pause/resume, Stop, failure, and stale rejection.
- SGF fixtures cover new and continued positions, handicap/setup admission, resign `RE`, double-pass scoring handoff, save/reopen, and review-only recovery.
- Real-engine smoke covers one KataGo JSONL Human-vs-Engine move and Stop, a two-process KataGo PK start/pause/resume/Stop path, and one supported non-KataGo GTP engine.
- Native smoke covers visible new/continue/pass/resign/Stop and PK start/pause/resume/Stop, failure-to-review, save/reopen, and restart without a live session.

Fixture-only backend evidence cannot mark a GAME item Accepted.

### R10 — Providers And readboard

**Goal:** Complete external preview, one-shot import, and ongoing synchronization on the R5 replacement and shutdown seams.

**Owns:** `PROV-01`, `PROV-02`, `PROV-03`, `PROV-04`, `READ-01`, `READ-02`, `READ-03`. Accepted `READ-03` remains the OCR-unsupported foundation.

**Start gate:** `SGF-07`, `PREF-01`, and `APP-03` accepted. Engine lifecycle is not required for `PROV-01`, `PROV-02`, `PROV-04`, or `READ-01`.

**Order:**

1. `PROV-01` Yike preview/import and `READ-01` sidecar readiness.
2. `PROV-02` Fox preview/import and `PROV-04` Tencent kifu preview/import.
3. `PROV-03` Yike ongoing sync / Play & Sync after `PROV-01`.
4. `READ-02` readboard ongoing sync after `READ-01` and `SGF-07`.

**Deliver:**

- Native provider center for public Yike `recommend` / `local` and recognized URL families, including unite rooms. Preview never mutates the current game; Import uses one-shot `SGF-07`.
- Fox `chessid` / `uid` / `user_name` and Tencent username/`chessId` preview, pagination, bounded recents, and one-shot import. Fox is not live sync and is not combined with Tencent.
- Yike Start sync and Play & Sync under the ongoing-synchronization model. Play & Sync is dual-channel: system browser for that room's login and webpage moves; Next public read-only signed path only.
- Sidecar probe/ready/incompatible/unavailable/timeout/restart as `READ-01`; equivalent one-way external-authoritative sidecar sync as `READ-02`, including disconnect and stop-to-editable.
- Shared contracts: no persisted provider secrets; Yike 10s deadline; Fox and Tencent 20s connect and 25s read; at most three retries after the initial attempt, and only for idempotent transient reads; ignore stale results; preserve last-good board on failure.

**Exit when:**

- `PROV-01` through `PROV-04` and `READ-01` through `READ-03` are accepted.
- Repository evidence names concrete non-zero focused tests for parsers, URL families, atomic import/sync transitions through `SGF-07`, cancellation, stale-result suppression, timeout/retry classification, last-good recovery, and stop-to-editable.
- Live evidence is separate per mode and records public Yike categories, every accepted Yike URL family, Play & Sync handoff, Fox and Tencent lookup plus pagination, upstream failure and latency, and readboard ready/sync/disconnect/restart.

`PROV-05` and `PUB-01` do not block this exit. Existing Partial plumbing is evidence toward these items, not completion.

### R11 — Release

**Goal:** Turn repository builds into independently admitted, updateable, supportable desktop releases.

**Owns:** `REL-01` through `REL-10` and final `APP-01` acceptance.

**Start gate:** `REL-01` may proceed at any time as a maintainer gate. `REL-06` requires `REL-03`, `REL-05`, and `APP-03`. `APP-01` final acceptance requires the R5 semantic gate plus `REL-04` association evidence.

**Order:**

1. `REL-01` preflight/dry-run.
2. `REL-02` trusted production artifacts, then `REL-03` discovery/offer and `REL-05` managed installed components.
3. `REL-04` platform package lifecycle per Canonical Artifact, independently.
4. `REL-06` Windows download/apply after `REL-03` + `REL-05` + `APP-03`.
5. `REL-07` macOS/Linux handoff after `REL-03`.
6. `REL-08` failure/rollback/recovery after `REL-06`.
7. `REL-09` diagnostics/support bundle and `REL-10` product identity in parallel.

```mermaid
flowchart LR
  REL02[REL-02 Trusted artifacts]
  REL03[REL-03 Discovery]
  REL05[REL-05 Components]
  APP03[APP-03 Shutdown]
  REL06[REL-06 Windows apply]
  REL07[REL-07 macOS/Linux handoff]
  REL08[REL-08 Windows rollback]
  REL04[REL-04 Platform lifecycle]
  APP01sem[APP-01 R5 semantic gate]
  APP01[APP-01 Accepted]

  REL02 --> REL03
  REL02 --> REL05
  REL03 --> REL06
  REL05 --> REL06
  APP03 --> REL06
  REL03 --> REL07
  REL06 --> REL08
  REL04 --> APP01
  APP01sem --> APP01
```

**Deliver:**

- Validators and non-mutating dry-run on the exact release commit, with their limits stated. This is not publication proof.
- Signed envelopes and verified payloads on stable/beta feeds. Windows production artifacts are Authenticode-signed and timestamped; macOS artifacts are signed, notarized, and stapled. Unsigned public-validation prereleases are labeled as such and never enter an updater feed.
- Help-triggered manual discovery: stable/beta, official/GitHub, persisted valid choices, SemVer comparison, OS proxy, and typed no-update/no-package/network/signature failures without mutating the install.
- Independent Installed Live Evidence for Windows NSIS, Windows portable, macOS DMG, and Linux AppImage, including declared runtime, data location, upgrade, and uninstall/delete. Installed removal preserves app-data by default. Portable launch covers present and absent WebView2.
- Installed manifest of `app-core` plus acquired KataGo backend/default-model components. No-engine launch works without optional components.
- Windows NSIS and portable component apply with resume/cancel/fallback, helper elevation when required, shutdown only after successful helper launch, manifest update, and restart without changing user data.
- macOS/Linux verified full-package download and DMG/folder handoff without overwriting the running tree.
- Windows reverse rollback, result journal, restored-version restart, and incomplete-restoration repair path. Maintainer GitHub-release withdrawal is not this item.
- Bounded ordinary logs default on, full trace default off, sanitized cancellable support-bundle export, and About that reads the packaged SemVer and channel.

**Exit when:**

- Global release items `REL-01` through `REL-05`, `REL-09`, and `REL-10` are accepted, and `APP-01` is accepted after its R5 semantic gate plus `REL-04` association evidence.
- Each Shipped Platform additionally accepts only its applicable branch and Installed Live Evidence row: Windows requires `REL-06` and `REL-08`; macOS and Linux require `REL-07`. Windows-only `REL-06`/`REL-08` and macOS/Linux-only `REL-07` are not required for every platform. Platform-inapplicable items are not required.
- Each Shipped Platform has its own production-trust gate and complete Installed Live Evidence row. An unshipped platform does not block admitting another.
- Unsigned artifacts remain explicitly labeled unsigned.
- Dry-run and other Repository Release Evidence are not substituted for release-environment or Installed Live Evidence.

Linux may publish an AppImage without a repository/package-manager signature only when updater envelope/payload verification and release notes state the exact trust and dependency contract.

### Deferred Queue

These items are stable and unnumbered. Each has an owner and admission conditions. None is an R3–R11 exit criterion until a later plan revision starts it.

| ID | Owner | Depends on | Admission |
| --- | --- | --- | --- |
| `I18N-01` | Preferences surface | `PREF-01` | After functional migration. Complete resources for every supported locale, persisted locale, deterministic fallback, no mixed partial-language state. |
| `SGF-15` | SGF authoring | `SGF-07`, `SGF-11`, `SGF-12` | Explicit setup/move semantics, bounds/occupancy validation, reversible edits, non-mutating rejection, save/reopen. |
| `SGF-16` | SGF authoring | `SGF-11`, `SGF-14` | Whole-tree color swap, rotate, and mirror with consistent coordinate-bearing properties. |
| `REVIEW-04` | Review | `REVIEW-01` | Deterministic branch traversal to matching moves; no-match is non-mutating. |
| `REVIEW-05` | Review | accepted `UI-05` toggle | Validated persisted main-board interval only. Does not reopen `UI-05` history. |
| `REVIEW-06` | Review | `REVIEW-01`, `RULE-01` | Reproducible ladder predicate; failure does not partially mutate the tree. |
| `EXPORT-01` | SGF authoring | current-game owner | Selected root-to-leaf branch as standalone SGF without mutating tree order, source, cursor, or dirty state. |
| `EXPORT-02` | Review | `UI-01` | Current main-board review view to a documented image format without changing current-game or review state. |
| `ENG-08` | Engine catalog | `ENG-01`, `ENG-09` | User-directed catalog reorder persists identities without changing Settings selection, Autoload Default, active run, pending edits, or job binding. |
| `ANA-06` | Analysis | `ENG-05`, `ANA-01`, `ANA-03` | Explicit default-off, run-scoped continuous current-node analysis. Never starts on launch, ready, restart, or switch. |
| `ANA-07` | Analysis | `ANA-02`, `APP-02` | Session-only SGF queue. Must not replace or dirty the current game. Queue state is not restored after restart. |
| `ANA-08` | Analysis | `ANA-04`, `SGF-01`, `SGF-05`; not `ANA-05` | Frozen Java analysis-header import and Next export/reopen. No false SQLite↔SGF sync claim. |
| `ANA-09` | Engine adapters | `ENG-09` | Named engine/version fixtures into the existing analysis model. Missing fields stay unavailable. Interface existence does not start this item. |
| `GAME-06` | Game modes | `GAME-03`, `GAME-05` | Session-only PK batch with durable completed-game output. Restart does not restore an unfinished queue. |
| `GAME-07` | Game modes | `GAME-02`, `GAME-03`, plus a separate clock policy | Application-owned remaining-time model. A Compute Budget timeout is not this item. |
| `GAME-08` | Game modes | `GAME-01`, `GAME-02`, `GAME-05`, HumanSL-compatible profile | Independent AI Coach Match Session, not a hidden option of `GAME-02`. |
| `GAME-09` | Game modes plus providers | domain-06 supported service, credentials, privacy, teardown | Board occupancy and global match exclusion only. Does not duplicate provider/network claims. |
| `PROV-05` | Providers | `SGF-07`; not `PROV-04` | Non-Yike Tencent/huanle live protocols as a separate item from kifu import. |
| `PUB-01` | Providers | shutdown/teardown owner | LAN board publish start/stop and copy-access URL. Trial internals stay excluded. |

## Next Executable Batch

R2 desktop review is closed except the `UI-02` residual (engine events during a pending board mutation). Start R3 foreground engine lifecycle. Do not mix R4–R11 integration work into this batch, do not accept `ENG-09` / `ENG-10` here, and do not treat the `UI-02` residual as the next slice.

### Slice R3-A — Foreground engine identity and lifecycle

Scope:

- Give `engine-manager` an authoritative no-engine / starting / ready / stopping / error snapshot with an immutable adapter/profile/capability snapshot.
- Expose that snapshot through Tauri commands and events.
- Stop treating profile selection as process identity.
- Keep Engine Settings edits from starting or switching a run.
- Prove this path with `KataGoAnalysis`. Leave Generic GTP and multi-backend catalog acceptance to R8.

Acceptance:

- `ENG-02` is accepted with repository evidence and native no-engine versus one-engine smoke.
- Analysis callers observe manager-owned identity rather than inferring a process from the selected profile.

`ENG-06` stays in R3 with default off and catalog persistence, but it is not this first slice.

## Verification Strategy

For each slice:

1. Run the narrowest fixture, crate test, frontend build/type check, or controlled process check that exercises the changed contract.
2. Run native desktop smoke when the changed behavior depends on Tauri, app data, file dialogs, process lifecycle, or visual interaction.
3. Run provider or sidecar live smoke only for the R10 modes in that item's acceptance statement, recorded separately from repository evidence.
4. For R11, record Repository Release Evidence, release-environment evidence, and Installed Live Evidence as distinct rows. Do not promote an item on dry-run or unsigned validation alone.
5. Update the affected parity rows with the exact evidence and remaining gap.
6. Broaden to workspace-wide validation only when a shared DTO or command boundary makes scoped checks insufficient; use a full suite at most once as the integration gate for that batch.

Structural and release validators remain bounded gates, not substitutes for feature evidence:

```bash
python3 scripts/validate_scaffold.py --verbose
python3 scripts/validate_release_assets.py --verbose
python3 scripts/validate_release_workflow.py --verbose
```

Use the scaffold validator when workspace, command, or document structure changes. Use the release validators in R11 or when release-support files change.

Provider/readboard acceptance must name the concrete Rust package or test filter and confirm that tests executed. Live environment results stay separate and follow `DEVELOPMENT.md`.

## Parallel Execution

When a slice is parallelized:

- One owner controls Rust domain/wire/Tauri changes.
- One owner controls TypeScript API/state/UI changes after the wire contract is fixed.
- One owner controls planning, smoke/release documentation, and parity evidence.
- Release workflow work remains a separate ownership area.
- Shared DTO and command contracts are decided before parallel edits.
- Parent integration owns focused validation; workers do not run repository-wide suites while sibling changes are incomplete.
- Reviewers may inspect all areas but do not silently rewrite another owner's files.

After R3 exits, R4, R5, and R8 may be assigned in parallel when their start gates are satisfied. R10 and R11 may overlap after theirs. R9 waits for the GAME admission gate.

## Migration Principle

The Java/Swing codebase is a frozen behavior reference, not the implementation skeleton for Next. Begin each item with observable behavior and the smallest evidence that would catch its loss. Implement that behavior behind the established Tauri/Rust/TypeScript boundaries, migrate every caller, remove the obsolete path, and record repository and live evidence separately.