# Migration Plan

## Goal

LizzieYzy Next remains the Tauri 2, Rust, and TypeScript product already implemented in this repository. Migration means reproducing or deliberately replacing observable behavior from the Java/Swing maintenance line; it does not mean translating Java source, preserving Swing internals, importing Java Git history, or restarting the Tauri implementation.

The migration is planned as complete user workflows:

1. Open an SGF, navigate and edit its tree, then save the edited game.
2. Review a game in the core desktop UI without requiring an engine.
3. Configure, start, switch, and stop a foreground KataGo engine safely.
4. Run interactive and whole-game analysis against the selected tree node.
5. Persist settings and a user-adjustable review layout.
6. Run supported engine-game modes.
7. Fetch or synchronize games through Fox, Yike, and readboard.
8. Install, update, and release the desktop product on supported platforms.

Each workflow crosses the Rust domain, Rust wire DTO, Tauri gateway, TypeScript API wrapper, React state, UI, and focused evidence that it actually needs. A layer-only milestone is not a migrated workflow.

## Sources Of Truth

- [ARCHITECTURE_NEXT.md](ARCHITECTURE_NEXT.md) owns architecture, module boundaries, DTO conventions, and command boundaries.
- [JAVA_BASELINE.md](JAVA_BASELINE.md) fixes the Java behavior reference at Migration Baseline v1.
- [PARITY_MATRIX.md](PARITY_MATRIX.md) owns item-level status, gaps, and acceptance.
- This document owns phase order, cross-item seams, and the next executable slices.
- [DEVELOPMENT.md](DEVELOPMENT.md) and [RELEASE_CHECKLIST.md](RELEASE_CHECKLIST.md) own environment smoke and release procedures.

Later Java `main` changes do not automatically change this plan. Data, SGF, Go-rule, and engine-correctness defects must be assessed; UX changes are judged individually; Swing-only implementation changes are ignored. Baseline changes follow the successor policy in `JAVA_BASELINE.md`.

## Evidence Model

Progress is parity-driven, not percentage-driven.

| Evidence column | What it proves | What it does not prove |
| --- | --- | --- |
| Repository evidence | A focused fixture, test, build, or exercised command path proves deterministic behavior in this repository. | Real provider sessions, sidecar installation, native file dialogs, packaged asset paths, platform signing, or installer behavior. |
| Live/environment evidence | A recorded native or external-environment smoke proves the environment-dependent path under named conditions. | General correctness outside the exercised conditions or repository regressions. |

An item is accepted only when the acceptance statement in `PARITY_MATRIX.md` is met. Wiring a command plus focused offline evidence is valid repository proof. It must not be described as live Fox/Yike/readboard, real KataGo, native desktop, updater, signing, or installer proof until that environment column is also satisfied.

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

- UI-02 remains Partial: there is still no evidence that engine events are delivered while a board mutation promise is pending. That residual does not block R3.
- Selecting an engine profile does not create an authoritative foreground engine identity or lifecycle.
- A → B engine switching, failed-switch rollback, stale switch token handling, and cancellable one-shot job identity are absent.
- Layout rails are fixed at `228px` and `260px`; splitters, persistence, and narrow reset semantics are absent.
- Engine-game session controls and state are absent.
- Provider/readboard repository plumbing exists, but live provider sessions and live sidecar behavior remain unvalidated; readboard snapshots are previewed rather than imported into the active game.
- Signing, notarization, production updater behavior, bundled installed-runtime resolution, and platform installer smoke remain incomplete.

These gaps are tracked individually in `PARITY_MATRIX.md`. Existing provider/readboard plumbing remains supported but is scheduled after the core local workflows rather than treated as the next migration frontier.

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

### Desktop Interaction

`BoardCanvas` should emit coordinate, pass, and hover intent through explicit component callbacks. The owning feature state decides whether the intent navigates, edits, or previews. Components continue to call wrappers in `apps/desktop/src/api`; no raw Tauri `invoke` is added to UI components.

Candidate and PV state is scoped to the selected `NodePath` and analysis job identity. Changing node, game, or job clears stale hover and candidate state before new results are presented.

### Foreground Engine And Analysis Jobs

`crates/engine-manager` becomes the sole owner of:

- Foreground engine identity and process lifecycle.
- No-engine, starting, ready, switching, stopping, and error state.
- Monotonic switch tokens and failed-switch rollback.
- One-shot and whole-game analysis job identity, cancellation, and stale-event rejection.

Tauri remains a command/event adapter. React observes lifecycle and job snapshots; it does not infer process identity from the selected profile.

For A → B switching:

1. Keep A authoritative while B starts and completes its handshake.
2. Promote B only after readiness is proven.
3. Bind new work to B, then stop A.
4. If B fails, keep or restore A as ready and primary.
5. Ignore completion or events whose switch token is no longer current.

All analysis callers migrate to the manager-owned lifecycle before the raw profile-to-process path is removed. No compatibility wrapper remains after the cutover.

### Preferences And Layout

Next-native preferences own panel proportions. The fixed left/right rail widths become user-adjustable layout state with practical minimums that preserve the board as the primary surface.

Restore Default is intentionally narrow: it resets panel sizes and board proportion only. It does not change panel visibility, window position/size, engine settings, theme, or unrelated preferences.

### Engine-Game State

Start inside `engine-manager` as a small internal state module. Its public behavior is:

- `start`
- `stop`
- `pause`
- `resume`
- `revise_batch_limit`
- `snapshot`

The module owns one current session and binds generated moves to the Rust-owned game tree. Do not create a new crate unless real dependency pressure appears.

### Providers And readboard

Keep `provider_fetch_yike`, `provider_fetch_fox`, `readboard_sidecar_probe`, and `readboard_sidecar_sync_snapshot` and their typed error boundaries. R7 integrates their successful output into the same Rust-owned current-game workflow completed in R1.

Image OCR remains unsupported unless an OCR-capable runtime, explicit parity item, and live evidence are added. Preview-only readboard sync is not complete import parity.

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

- `UI-01` through `UI-05` are accepted.
- Native smoke passes both without an engine and with engine-only actions unavailable.
- Visual evidence follows the reference rules in `DESIGN.md`.

Current state: Ticket 07 closeout on native Windows candidate `66c906f` / PID 51344 accepted `UI-01`, `UI-03`, `UI-04`, and `UI-05`. `UI-02` remains Partial solely for engine-event delivery while a board mutation promise is pending. That residual is tracked in `PARITY_MATRIX.md` and is not the next executable slice.

### R3 — KataGo And Foreground Engine Lifecycle

**Goal:** Replace profile-as-selection with one authoritative foreground engine lifecycle.

Deliver:

- Manager-owned engine identity/state snapshot and Tauri command/event adapter.
- Start, stop, readiness handshake, and no-engine behavior.
- Successful A → B promotion.
- Failed B rollback to A.
- Stale switch token and stale engine-event rejection.
- Manager-owned cancellable one-shot jobs.
- Clean migration of analysis callers away from raw profile process startup.

Exit when:

- `ENG-01` through `ENG-05` are accepted.
- Controlled process tests prove successful switch, failed switch rollback, cancellation, and delayed stale completion.
- Native smoke proves no-engine, one-engine, and two-profile switch behavior with real KataGo assets.

### R4 — Analysis Workflows

**Goal:** Bind interactive and whole-game analysis to the selected SGF node and authoritative engine/job identity.

Deliver:

- One-shot result/error/cancel/supersede behavior.
- Whole-game session/request state, progress, cancellation, and current-node integration.
- Node/job-scoped candidates, PV, ownership, policy, and problem markers.
- Variation-aware cache decisions before cached results are claimed across branches.

Exit when:

- `ANA-01` through `ANA-05` are accepted.
- Controlled KataGo evidence covers response validation, timeout, stderr error, cancellation, and stale results.
- Native smoke exercises one-shot and whole-game analysis against a branching edited SGF.

### R5 — Settings And Layout

**Goal:** Persist Next-native user choices and make the review layout adjustable.

Deliver:

- Inventory and deliberate mapping of supported Java-visible settings to Next owners/defaults.
- Draggable left/board/right splitters.
- Persisted layout proportions.
- Narrow Restore Default semantics.

Exit when:

- `PREF-01` and `LAYOUT-01` through `LAYOUT-03` are accepted.
- Native restart smoke proves persistence.
- Reset smoke proves unrelated visibility, window, engine, and preference state is unchanged.

### R6 — Game Modes

**Goal:** Add authoritative engine-game session behavior without mixing it into review-state inference.

Deliver:

- Start/stop/pause/resume state transitions and snapshot.
- Revisable batch limit.
- SGF tree integration for generated moves.
- Separate generated game information and personal comments.

Exit when:

- `GAME-01` through `GAME-03` are accepted.
- Focused state tests cover normal transitions and limit revision.
- Native real-engine smoke covers a short session, pause/resume, limit revision, stop, save, and reopen.

### R7 — Fox, Yike, And readboard

**Goal:** Complete external import/synchronization workflows on top of the current-game model.

Deliver:

- Yike fetch/import and recorded live behavior.
- Fox `chessid`, `uid`, and `user_name` fetch/import and recorded live behavior.
- readboard readiness and protocol-line sync under a real sidecar.
- Snapshot import into the active Rust-owned game, not preview only.
- Typed, user-visible failure behavior for unavailable/expired sessions, network errors, timeouts, incompatible sidecars, crashes, and malformed payloads.

Exit when:

- `PROV-01`, `PROV-02`, and `READ-01` through `READ-03` are accepted.
- Repository evidence names concrete non-zero focused tests.
- Live evidence records prerequisites, versions/session type, network/target state, result, latency where relevant, and recovery behavior.

### R8 — Update, Packaging, And Release

**Goal:** Turn repository builds into installable, updateable, supportable desktop releases.

Deliver:

- Release asset/workflow preflight and dry-run on the release commit.
- Bundled runtime asset resolution from packaged application paths.
- Windows signing and macOS signing/notarization where those platforms are shipped.
- Production updater feed/signature/version/failure behavior.
- Platform installer install/launch/primary-flow/upgrade-or-uninstall smoke.

Exit when:

- `REL-01` through `REL-05` are accepted for every shipped platform.
- Unsigned artifacts remain explicitly labeled unsigned.
- Dry-run evidence is not substituted for publication or installer evidence.

## Next Executable Batch

R2 desktop review is closed except the UI-02 residual (engine events during a pending board mutation). Start R3 foreground engine lifecycle; do not mix R4–R8 integration work into this batch, and do not treat the UI-02 residual as the next slice.

### Slice R3-A — Foreground engine identity and lifecycle

Scope:

- Give `engine-manager` an authoritative no-engine / starting / ready / stopping / error snapshot.
- Expose that snapshot through Tauri commands and events.
- Stop treating profile selection as process identity.

Acceptance:

- `ENG-02` is accepted with repository evidence and native no-engine versus one-engine smoke.
- Analysis callers observe manager-owned identity rather than inferring a process from the selected profile.

## Verification Strategy

For each slice:

1. Run the narrowest fixture, crate test, frontend build/type check, or controlled process check that exercises the changed contract.
2. Run native desktop smoke when the changed behavior depends on Tauri, app data, file dialogs, process lifecycle, or visual interaction.
3. Update the affected parity rows with the exact evidence and remaining gap.
4. Broaden to workspace-wide validation only when a shared DTO or command boundary makes scoped checks insufficient; use a full suite at most once as the integration gate for that batch.

Structural and release validators remain bounded gates, not substitutes for feature evidence:

```bash
python3 scripts/validate_scaffold.py --verbose
python3 scripts/validate_release_assets.py --verbose
python3 scripts/validate_release_workflow.py --verbose
```

Use the scaffold validator when workspace/command/document structure changes. Use the release validators in R8 or when release-support files change.

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

## Migration Principle

The Java/Swing codebase is a frozen behavior reference, not the implementation skeleton for Next. Begin each item with observable behavior and the smallest evidence that would catch its loss. Implement that behavior behind the established Tauri/Rust/TypeScript boundaries, migrate every caller, remove the obsolete path, and record repository and live evidence separately.
