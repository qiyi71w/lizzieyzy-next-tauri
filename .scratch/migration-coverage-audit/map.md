# Audit the Full Migration Baseline v1 Capability Surface

## Destination

Establish three authoritative, mutually traceable migration documents: an exhaustive `docs/JAVA_CAPABILITY_INVENTORY.md`, an acceptance-sized `docs/PARITY_MATRIX.md`, and a dependency-ordered `docs/MIGRATION_PLAN.md`. Every user-reachable behavior and Entry Point in Migration Baseline v1 must map exactly once to a Parity Item or an explicit exclusion disposition; completed R0–R2 history and Accepted item scopes remain intact.

## Notes

- Domain: observable desktop-product behavior from frozen Java Migration Baseline v1 at `7b4027531c2b26062d0bfc27a040cc550cfbea4d`, compared with the Next baseline at `18c6d189b8b01069975c4c40ead63a010249cb8c`.
- Frozen Java source worktree: `/home/dev/dev/weiqi/worktrees/lizzieyzy-next/java-baseline-v1`; detached HEAD must equal `7b4027531c2b26062d0bfc27a040cc550cfbea4d` before a census result is accepted.
- Read `CONTEXT.md`, `docs/JAVA_BASELINE.md`, `docs/ARCHITECTURE_NEXT.md`, `docs/PARITY_MATRIX.md`, and `docs/MIGRATION_PLAN.md` before working a ticket.
- Use `/research` for research tickets and `/grilling` plus `/domain-modeling` for disposition and roadmap tickets.
- Completeness uses a static census of all user-reachable source surfaces plus targeted baseline runtime checks where dynamic visibility, defaults, persistence, or failure behavior cannot be established from source.
- The Capability Inventory owns baseline Entry Points, defaults, persistence, failure behavior, source references, and runtime-check status. The Parity Matrix owns Next disposition, status, evidence, remaining gap, and acceptance. The Migration Plan owns phases, dependencies, order, and exit criteria.
- This effort carries documentation execution inside the map: research assets and resolved domain decisions may update the three destination documents. Production Rust/TypeScript behavior changes remain out of scope.
- R0–R2 phase numbers and historical acceptance remain frozen. R3 remains the next phase but may absorb lifecycle-affecting settings; R4 and later may be split or reordered.
- Preserve an Accepted Parity Item’s ID, scope, evidence, and status. Materially new behavior becomes a Successor Item.
- Equivalent and Swing-only classifications may follow clear evidence. Any Next redesign, defer, or abandon disposition requires live user confirmation.
- R3 implementation is paused during this audit. Preserve the pre-map foreground-engine decisions in [R3 foreground-engine standing decisions](context/r3-foreground-engine-decisions.md).
- Local Markdown is the issue tracker for this effort. Open tickets are child files under `issues/`; blocking and claim state follow the tracker headers.

## Decisions so far

- [Inventory Update, Packaging, Release, and Platform Capabilities](issues/07-inventory-update-packaging-release.md) — The clean frozen census identifies thirteen install, startup, bundled-runtime, update, diagnostics, OS-integration, support, privacy-reset, and product-identity capabilities while separating maintainer-only release mechanics.
- [Inventory SGF, Board, and Review Capabilities](issues/03-inventory-sgf-board-review.md) — The clean frozen census preserves Accepted R1/R2 scopes while separating adjacent file, tree, board, metadata, recovery, and review-presentation capabilities for explicit successor decisions.
- [Inventory Foreground Engine and Analysis Capabilities](issues/04-inventory-engine-analysis.md) — The clean frozen census identifies twenty-five lifecycle, preference, analysis, cache, and protocol capabilities and exposes the exact R3-versus-later decision boundary.
- [Inventory Application Shell, Startup, Exit, and Interaction Entry Points](issues/01-inventory-application-shell-entry-points.md) — The clean frozen census records thirteen shell capabilities and routes every actually registered UI, shortcut, pointer, OS, and dynamic Entry Point to its semantic owner domain.
- [Inventory Provider, Synchronization, readboard, and External Publishing Capabilities](issues/06-inventory-providers-sync-readboard.md) — The clean frozen census distinguishes ongoing sync, import, embedded browsing, readboard, LAN publishing, and sync preferences; registered share shortcuts are no-ops, while the remaining share surfaces have no registered Entry Point.
- [Inventory Game-Mode Capabilities](issues/05-inventory-game-modes.md) — The clean frozen census identifies eight match/start capabilities and proves that current GAME items over-aggregate PK while omitting human genmove, analysis-mode play, HumanSL, contribute, and shared rules/limits.
- [Inventory Settings, Layout, and Persistence Capabilities](issues/02-inventory-settings-layout-persistence.md) — The clean frozen census defines thirty-five general settings, layout, reset, hint, first-use, and persistence capabilities while routing lifecycle-affecting engine preferences to domain 04.
- [Choose Dispositions for Application Shell and Interaction Capabilities](issues/08-decide-application-shell-dispositions.md) — Adopt Next-native SGF/GIB activation and drop, safe shutdown, current-game recovery, and a discoverable shortcut registry; abandon legacy `read`, hostname wiping, and visible force-exit behavior while excluding Swing menu hosting.
- [Choose Dispositions for Settings, Layout, and Persistence Capabilities](issues/09-decide-settings-layout-dispositions.md) — Adopt durable native preferences, a single persisted adaptive workspace, curated appearance, and contextual guidance; defer complete localization, abandon Java/Swing configuration structure, and route semantic settings to their behavior owners.
- [Choose Dispositions for SGF, Board, and Review Capabilities](issues/10-decide-sgf-board-review-dispositions.md) — Preserve Accepted R1/R2 claims; adopt parse-before-replace SGF/GIB/recent/clipboard intake, root-KM authority, automatic session recovery, core tree/setup/metadata/markup editing, Next-native click review/try-play/dual-rule scoring, and persisted coordinate/move-number booleans; abandon Swing raw saves, manual temp slots, the load-komi/hover-delay/granular move-number settings, and defer destructive power tools, configurable autoplay, ladder, branch export, and board-image export.
- [Choose Dispositions for Provider, Synchronization, readboard, and External Publishing Capabilities](issues/13-decide-provider-sync-readboard-dispositions.md) — Split external workflows by provider and preview/import/synchronization mode; preserve Yike web play through system-browser Play & Sync; migrate Fox/Tencent kifu intake and readboard ongoing sync; defer Yike personal/private/auth-required Next read modes, Tencent/huanle live, GMA, and WebBoard; and abandon embedded Yike hosting and no-op share shortcuts under explicit timeout, retry, recovery, privacy, and evidence contracts.
- [Choose Dispositions for Foreground Engine and Analysis Capabilities](issues/11-decide-engine-analysis-dispositions.md) — Adopt an adapter-backed resident Foreground Engine Run, zero-or-one profile autoload default, transactional lifecycle and lane-scoped cancellable jobs, single-stage mainline analysis, capability-gated Next-native presentation, and typed failures; preserve accepted KataGo profile history while adding multi-backend successor boundaries, defer named rich-analysis adapters, profile ordering, continuous analysis, batch files, and SGF analysis exchange, and abandon hidden dedicated/preloaded processes and overlapping Java analysis modes.
- [Choose Dispositions for Game-Mode Capabilities](issues/12-decide-game-mode-dispositions.md) — Merge legacy genmove and analysis-mode play into one Human-vs-Engine session; adopt single-game PK, transactional match reservations, shared rules and Compute Budgets, exact-position admission, SGF/review-only recovery, KataGo JSONL full capability and Generic GTP game adapters; redesign candidate visibility by human/engine role; defer PK batch, competitive clocks/auto-resign, HumanSL, contribute, and named rich-analysis adapters under explicit dependencies and native evidence.
- [Choose Dispositions for Update, Packaging, Release, and Platform Capabilities](issues/14-decide-update-release-dispositions.md) — Admit Windows NSIS/portable, macOS DMG, and Linux AppImage independently; rebuild the signed Java component-updater behavior around Next-native components, platform-specific apply/handoff, transactional Windows rollback, native diagnostics/support, and strict repository-versus-installed evidence gates.


## Not yet specified

- Prototype or targeted native-baseline tickets needed only when a domain disposition decision depends on runtime behavior that static frozen source cannot establish.

## Out of scope

- Implementing or closing newly discovered migration gaps in production Rust, Tauri, TypeScript, React, provider, engine, packaging, or release code.
- Moving Migration Baseline v1 to a newer Java commit; post-baseline changes continue to use the existing severity-based governance rule.
- Importing Java configuration formats, Git history, Swing widget structure, thread structure, or other implementation details without a separately approved user-facing need.
- Obtaining provider credentials, signing/notarization credentials, or production release infrastructure merely to complete this documentation audit.
