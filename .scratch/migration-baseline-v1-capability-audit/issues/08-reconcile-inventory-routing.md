# 08 — Reconcile Inventory Corpus and Cross-domain Routing

**What to build:** Integrate the seven completed domain inventories into one exhaustive Capability Inventory in which every frozen Capability and registered Entry Point has exactly one semantic owner and one authoritative mapping or exclusion.

**Blocked by:** 01 — Audit Application Shell and Interaction Capabilities; 02 — Audit Settings, Layout, and Persistence Capabilities; 03 — Audit SGF, Board, and Review Capabilities; 04 — Audit Foreground Engine and Analysis Capabilities; 05 — Audit Game-Mode Capabilities; 06 — Audit Providers, Synchronization, readboard, and Publishing; 07 — Audit Update, Packaging, Release, and Platform Capabilities.

**Status:** done

**Guardrails:** The Inventory alone owns baseline Entry Points, defaults, persistence, failure behavior, source references, and runtime-check status. Do not decide Matrix statuses or roadmap sequencing here. Rebuild sets; never normalize toward historical totals.

- [x] Reconstruct the unique Capability set from all seven domain tables, report its computed count, then compare 139 as reference evidence; any mismatch stays open with exact rows.
- [x] Reconstruct registered Entry Points by domain and prove each routes once to one semantic Capability owner; duplicate, missing, and foreign-owned rows are named exactly.
- [x] Verify every Inventory row has required defaults, persistence, failure, source, and runtime-check-status fields, and every runtime check names a dynamic-visibility, default, persistence, or failure fact unavailable statically.
- [x] Validate every mapping target as supported Parity Item coverage or explicit exclusion or absorbed redesign; report the exact unresolved remainder set rather than inventing IDs.
- [x] Rebuild the census completeness index from the computed set and confirm no cross-domain route creates duplicate Capability ownership.

## Answer

Independent reconstruction: [research 08](../research/08-reconcile-inventory-routing.md). Destination: corpus counts, Cross-domain routes, runtime-check class words, absorbed `ENG-02` labels, and the census completeness index in `docs/JAVA_CAPABILITY_INVENTORY.md`. `docs/PARITY_MATRIX.md` and `docs/MIGRATION_PLAN.md` were not edited.

- **Computed unique Java Capabilities: 135.** Unique Frozen IDs in domain tables: 139 (no duplicates). Reference 139. Extra Capability rows: none. Exact Frozen IDs that are not Capabilities: `CAP-04-ANA-05`, `CAP-04-ANA-14`, `REL-C12`, `REL-C13`. 139 was compared afterward and was not a stop condition.
- Duplicate Frozen IDs: none. Missing registered Entry Points versus the seven domain tables: none. Foreign-owned / link-only rows are named in one canonical Cross-domain row per physical surface (`SGF-03-ADJ-URL`, `SGF-03-ADJ-PROVIDER`, `SHELL-02` process-start radios, About chrome / `REL-C11`, Ctrl+I size split, View 下一手 / `J`). No cross-domain route creates two semantic owners for one user goal.
- Every Capability row has defaults, persistence, failure, source, and runtime-check status. Runtime checks now name dynamic visibility, default, persistence, or failure, or are `Not required`.
- Mapping remainder: **empty**. 125 supported coverage (including splits), 7 explicit exclusions, 3 absorbed redesigns (`CAP-04-ANA-01`/`02`/`03` → `ENG-02`). No new Parity IDs. Matrix-only `BASE-01`/`BASE-02`/`REL-01`/`ANA-09` are not Inventory remainders.
- Census completeness index rebuilt: 135 Capabilities listed once; four non-Capability Frozen IDs listed once as not counted.
