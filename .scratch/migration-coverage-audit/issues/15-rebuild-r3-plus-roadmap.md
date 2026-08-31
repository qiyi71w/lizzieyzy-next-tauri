# Rebuild the R3+ Dependency Roadmap

Type: grilling
Status: resolved
Blocked by: 08, 09, 10, 11, 12, 13, 14

## Question

Given all resolved domain dispositions and their acceptance-sized Parity Items, what dependency graph, phase boundaries, order, and exit criteria should replace the current R3+ roadmap while keeping completed R0–R2 history fixed, keeping foreground engine lifecycle as the next executable phase, moving lifecycle-affecting settings into their owning phases, and leaving the complete settings inventory traceable rather than concentrated in one catch-all phase?

## Answer

The rebuilt roadmap is recorded in the three destination documents with non-overlapping authority:

- `docs/JAVA_CAPABILITY_INVENTORY.md` owns the exhaustive frozen census: 139 unique Capability rows across domains 01–07, Entry Points, defaults, persistence, failures, evidence provenance, and final owner/disposition.
- `docs/PARITY_MATRIX.md` owns 95 stable Parity Items, status, evidence, remaining gap, acceptance, phase/deferred ownership, and start dependencies.
- `docs/MIGRATION_PLAN.md` owns phase membership, dependency order, gates, exit criteria, the Deferred queue, and the next executable batch.

Completed R0–R2 history remains fixed. The replacement R3+ topology is:

| Phase | Workflow | Owned acceptance boundary |
| --- | --- | --- |
| R3 | Foreground Engine Lifecycle | `ENG-02`–`ENG-07`; `ENG-06` Autoload Default is off at first use |
| R4 | Analysis | `ANA-01`–`ANA-04`; `ANA-05` remains frozen Accepted |
| R5 | Safe Current Game / Application Shell | `PREF-01`, `SGF-07`, `APP-02`–`APP-05`, and the `APP-01` semantic gate |
| R6 | SGF Authoring / Review | `SGF-08`–`SGF-14`, `REVIEW-01`–`REVIEW-03`, `REVIEW-07`, `REVIEW-08` |
| R7 | Adaptive Workspace | `LAYOUT-01`–`LAYOUT-04`, `WINDOW-01`, `APPEAR-01`, `GUIDE-01` |
| R8 | Engine Adapters | `ENG-09`, `ENG-10` |
| R9 | Game Modes | `GAME-01`–`GAME-05` |
| R10 | Providers / readboard | `PROV-01`–`PROV-04`, `READ-01`–`READ-03` |
| R11 | Release | `REL-01`–`REL-10` and final `APP-01` acceptance |

Phase numbers group work and express priority; named dependencies are authoritative. R10 and R11 have no hard edge between them and may overlap after their own gates. R3 remains the named next executable phase. Slice R3-A establishes the manager-owned foreground run identity and lifecycle through `KataGoAnalysis`; it does not pull R4–R11 integration, `ENG-09`, or `ENG-10` into that batch.

The critical graph is explicit:

- R3 establishes run identity before switch transactions, lane jobs, autoload, and recovery.
- R4 binds selected-node and whole-game work to manager-owned run/job/node identity before presentation.
- R5 establishes `PREF-01` as mechanism/surface only, `SGF-07` as the shared parse-before-replace seam, `APP-03` as the single graceful-shutdown seam, and `APP-04` as review-only recovery that never resurrects engines, jobs, matches, or synchronization.
- `APP-01` is the sole split item: `SGF-07` permits its R5 semantic gate; `APP-02` follows that passed gate; `REL-04` later supplies the additional association and Installed Live Evidence required for final R11 acceptance.
- R9 starts only after `ENG-02`, `ENG-03`, `ENG-04`, `ENG-07`, `ENG-09`, `ENG-10`, `SGF-07`, `APP-04`, `REVIEW-03`, `PREF-01`, and `ANA-04` are Accepted. Its internal order is `GAME-01` + `GAME-04` + `GAME-05`, then `GAME-02`, then `GAME-03`.
- R10 requires the current-game, preference, and shutdown seams; `PROV-03` follows `PROV-01`, and `READ-02` follows `READ-01`.
- R11 keeps repository release evidence, release-environment evidence, and Installed Live Evidence separate. Platform admission is independent: global release items apply globally, Windows additionally requires its apply/rollback branch, and macOS/Linux require their package-handoff branch.

Semantic settings stay with behavior owners instead of a catch-all phase. `PREF-01` does not absorb them. The 19 explicitly Deferred items remain in one unnumbered queue and are never numbered-phase exits until a later plan revision admits them.

The census also exposes thirteen owner-route/conflict records for which Tickets 08–14 supplied no stable Parity ID. They remain visible and unscheduled for Ticket 16: the next-move marker and related SGF hint, winrate-graph controls, sub-board mode, main-panel extras, the `GUIDE-01` producer conflict, provider proxy, SSH/remote compute, contribute-service ownership, readboard GMA, personal/auth Yike reads, analysis-panel image export, and domain-04 autoplay remainder. No existing Accepted, Missing, or Deferred item silently absorbs them, and no unsupported ID is invented.

Ticket 16 is therefore the final traceability audit, not another roadmap-design phase.
