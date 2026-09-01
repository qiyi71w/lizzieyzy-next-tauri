# 10 — Reconcile the R3+ Dependency Roadmap

**What to build:** Produce one dependency-ordered Migration Plan from the reconciled Matrix so an implementation agent can identify the next executable phase, genuine start prerequisites, delivery sequencing, Deferred promotion conditions, phase gates, exits, and evidence classes without conflicting authorities.

**Blocked by:** 09 — Reconcile Matrix Corpus and Preserve Accepted Contracts.

**Status:** done

**Guardrails:** Preserve R0 through R2; R3 remains next and owns Foreground Engine lifecycle. Matrix `Depends on` is the sole Item Start Prerequisite authority. The Plan owns phase membership and exits, Deferred Promotion Gates, Delivery Order, and Migration Phase Gates. ADRs and recorded Dispositions are settled inputs. Do not move Deferred work into numbered phases merely because an interface exists.

- [x] Rebuild phase membership and exits from the complete Matrix set, preserving R0 through R2 and ordering R3 and later phases from real dependencies.
- [x] Reconcile all nine disputed boundaries under their designated authorities, including GAME-02 before GAME-03 as Delivery Order and CONTRIB-01 as a GAME-09 Item Start Prerequisite.
- [x] Independently parse Matrix and Plan Deferred sets, report computed counts, then compare the reference 27-item equality; mismatches stay open with exact IDs.
- [x] Every Deferred item has an explicit Promotion Gate; every numbered phase has a Phase Gate and exit; Delivery Order never duplicates or strengthens Matrix `Depends on`.
- [x] Semantic settings remain with behavior owners, all thirteen historical remainders are represented by their resolved item or exclusion decisions, and repository and live evidence gates remain non-interchangeable.

## Answer

Independent reconstruction: [research 10](../research/10-reconcile-r3-dependency-roadmap.md). Destination: rebuilt phase membership, Promotion Gates, Delivery Order, Phase Gates, and exits in `docs/MIGRATION_PLAN.md`, plus a membership pointer in `docs/PARITY_MATRIX.md`. Inventory mappings and Matrix item rows, status, evidence, and `Depends on` were not rewritten.

- Phase membership is the complete Matrix set. R0–R2 stay historical. R3 remains next and owns Foreground Engine lifecycle (`ENG-02`–`ENG-07`, with frozen `ENG-01`). R4–R11 follow Matrix `Depends on` and recorded dispositions. Deferred items stay in the unnumbered queue.
- Nine Ticket 29 boundaries sit under designated authorities. `GAME-02` before `GAME-03` is Delivery Order only. `CONTRIB-01` remains a `GAME-09` Item Start Prerequisite in the Matrix. The Plan-owned `GAME-09` Promotion Gate remains Accepted `GAME-01` and Accepted `CONTRIB-01`.
- **Computed unique Parity Items: 108.** Status split: 16 Accepted, 19 Partial, 46 Missing, 27 Deferred. Independently parsed Plan Deferred IDs: **27**. Matrix and Plan Deferred sets are equal. Only-Matrix: none. Only-Plan: none. Reference 27-item equality was compared afterward and was not a stop condition.
- Every Deferred item has an explicit Promotion Gate (`—` means no additional condition). Duplicate Matrix-ISP restatements were stripped from Promotion Gates except the Ticket 29 designated `GAME-09` Accepted-status gate. Every numbered phase has a Migration Phase Gate and an exit. `ENG-03` may start once Matrix `ENG-02` is satisfied; later list position is Delivery Order only.
- Semantic settings remain with behavior owners. All thirteen historical remainders have a resolved item or exclusion. Repository evidence, native/provider live evidence, Repository Release Evidence, release-environment evidence, and Installed Live Evidence stay non-interchangeable.
