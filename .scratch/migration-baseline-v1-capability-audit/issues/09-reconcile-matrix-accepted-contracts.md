# 09 — Reconcile Matrix Corpus and Preserve Accepted Contracts

**What to build:** Build the complete Parity Matrix from the reconciled Inventory mappings so every supported Capability reaches an acceptance-sized item with status, evidence, gap, acceptance, and Item Start Prerequisites, while every original Accepted contract remains intact.

**Blocked by:** 08 — Reconcile Inventory Corpus and Cross-domain Routing.

**Status:** done

**Guardrails:** The Matrix owns status, evidence, remaining gap, acceptance, and `Depends on`. Compare against Next revision `18c6d189b8b01069975c4c40ead63a010249cb8c` only for the original Accepted contracts. New material uses Successor Items. Do not assign phases, Promotion Gates, Delivery Order, or Phase Gates here. Counts are observations, not targets.

- [x] Reconstruct unique Accepted, Partial, Missing, and Deferred item rows from Matrix content, report computed totals, then compare 108 and 16, 19, 46, and 27 as reference evidence; mismatches stay open with exact rows.
- [x] Validate every Inventory supported mapping target exists exactly once in the Matrix and every Matrix item has an owning Capability or explicit implementation-gate rationale.
- [x] Verify every row has status, repository and live evidence classification, remaining gap, observable acceptance, and `Depends on` under Matrix authority.
- [x] Compare the original sixteen Accepted contracts field-for-field across ID, observable scope, status, evidence, remaining gap, and acceptance; named permitted history context or phase movement cannot alter those contracts.
- [x] Confirm every materially new behavior is a Successor Item and no exclusion, owner-route, or implementation detail is promoted into a false parity claim.

## Answer

Independent reconstruction: [research 09](../research/09-reconcile-matrix-accepted-contracts.md). Destination: corpus counts, mapping-coverage and Matrix-only rationale, Accepted-preservation note, and the item completeness index in `docs/PARITY_MATRIX.md`. `docs/JAVA_CAPABILITY_INVENTORY.md` and `docs/MIGRATION_PLAN.md` were not edited.

- **Computed unique Parity Items: 108.** Duplicate IDs: none. Reference 108. Extra/missing item rows: none. Status split: 16 Accepted, 19 Partial, 46 Missing, 27 Deferred. 108 and 16, 19, 46, 27 were compared afterward and were not stop conditions.
- Every Inventory supported mapping target exists exactly once. Mapping Parity IDs absent from the Matrix: none. Unresolved mapping remainder: empty. `SET-FIRST-LAUNCH` names Frozen `SHELL-06` as a Domain 01 link, not a Matrix target. Matrix-only `BASE-01`, `BASE-02`, `REL-01`, and `ANA-09` each keep an implementation-gate rationale.
- Every row has status, both evidence columns, remaining gap, and acceptance. R0–R2 omit `Depends on` under Matrix historical-column rule. Every R3+ and Deferred row has `Depends on`.
- The original sixteen Accepted contracts at `18c6d189b8b01069975c4c40ead63a010249cb8c` remain Accepted at original ID, observable scope, status, evidence, remaining gap, and acceptance. `BASE-02` repository evidence only adds historical successor-inventory context. `READ-03` only moves from R7 to R10 under permitted R4+ reordering.
- Materially new behavior uses Successor Items. No extra item is Accepted. Exclusions and owner routes stay non-Accepted.
