# 09 — Reconcile Matrix Corpus and Preserve Accepted Contracts

Independent Matrix corpus reconstruction for [Ticket 09](../issues/09-reconcile-matrix-accepted-contracts.md). Destination writes land only in `docs/PARITY_MATRIX.md`. Inventory mappings and Plan sequencing were not edited. No Parity IDs were invented.

## Sources

| Tree | Path | Commit |
| --- | --- | --- |
| Java Migration Baseline v1 | `/home/dev/dev/weiqi/worktrees/lizzieyzy-next/java-baseline-v1` | `7b4027531c2b26062d0bfc27a040cc550cfbea4d` (not re-censused) |
| Next inventory baseline | this worktree | `18c6d189b8b01069975c4c40ead63a010249cb8c` (Accepted-field comparison only) |
| Current destination docs | this worktree | `c1a307db1f6cb81b758244f5b69dbfc43fa600b1` at reconstruction time |
| Inventory mappings | `docs/JAVA_CAPABILITY_INVENTORY.md` | Ticket 08 corpus plus Tickets 01–07 domain tables |
| Prior coverage-audit totals | `.scratch/migration-coverage-audit/research/30-final-traceability-reaudit.md` | Reference 108 / 16, 19, 46, 27 only |

Java source was not re-censused. Completeness is the unique item set reconstructed from Matrix tables after Inventory mappings were reconciled. Historical 108 / 16, 19, 46, 27 are comparison evidence, not stop conditions. Phases, Promotion Gates, Delivery Order, and Phase Gates stay with the Plan.

## Method

1. Parse every Matrix table row whose status is Accepted, Partial, Missing, or Deferred. Count unique IDs, then count by status. Duplicate IDs stay open. Compare 108 and 16, 19, 46, 27 afterward.
2. Parse every Frozen-ID table under Inventory `## Domain 01` … `## Domain 07`. Extract backtick Parity IDs (`[A-Z]+-digits`) from Mapping cells. A mapping target is a Parity ID that exists in the Matrix. Frozen IDs named as owner links are not mapping targets.
3. For each Matrix row, require status, repository evidence, live/environment evidence, remaining gap, and observable acceptance. Require `Depends on` where Matrix authority puts that column (R3 onward, including Deferred). R0–R2 keep historical columns.
4. Diff the sixteen Accepted rows at `18c6d189:docs/PARITY_MATRIX.md` against the current Matrix on ID, Capability (observable scope), Status, both evidence columns, Remaining gap, and Acceptance. Named permitted history context or permitted R4+ phase movement cannot alter those contracts.
5. Classify current-only Matrix IDs as Successor Items. Confirm Exclusion / owner-route / implementation wording is not an extra Accepted claim.

## Computed item set

**Computed unique Parity Items: 108.** Duplicate IDs: **none**.

Reference 108. Extra item rows: none. Missing item rows: none. 108 was compared afterward and was not a stop condition.

| Status | Computed | Reference | Extra / missing IDs |
| --- | --- | --- | --- |
| Accepted | 16 | 16 | none |
| Partial | 19 | 19 | none |
| Missing | 46 | 46 | none |
| Deferred | 27 | 27 | none |
| **Total** | **108** | **108** | **none** |

16 + 19 + 46 + 27 = 108.

### Accepted (16)

`BASE-01` `BASE-02` `SGF-01` `SGF-02` `SGF-03` `SGF-04` `SGF-05` `SGF-06` `RULE-01` `UI-01` `UI-03` `UI-04` `UI-05` `ENG-01` `ANA-05` `READ-03`

### Partial (19)

`UI-02` `ENG-05` `ANA-01` `ANA-02` `ANA-03` `ANA-04` `PREF-01` `SGF-07` `SGF-10` `REVIEW-01` `REVIEW-07` `APPEAR-01` `PROV-01` `PROV-02` `READ-01` `READ-02` `REL-01` `REL-05` `REL-10`

### Missing (46)

`ENG-02` `ENG-03` `ENG-04` `ENG-06` `ENG-07` `ANA-10` `ANA-11` `ANA-12` `ANA-13` `APP-01` `APP-02` `APP-03` `APP-04` `APP-05` `SGF-08` `SGF-09` `SGF-11` `SGF-12` `SGF-13` `SGF-14` `REVIEW-02` `REVIEW-03` `REVIEW-08` `REVIEW-09` `LAYOUT-01` `LAYOUT-02` `LAYOUT-03` `LAYOUT-04` `WINDOW-01` `WINDOW-02` `ENG-09` `ENG-10` `GAME-01` `GAME-02` `GAME-03` `GAME-04` `GAME-05` `PROV-03` `PROV-04` `REL-02` `REL-03` `REL-04` `REL-06` `REL-07` `REL-08` `REL-09`

### Deferred (27)

`I18N-01` `GUIDE-01` `SGF-15` `SGF-16` `REVIEW-04` `REVIEW-05` `REVIEW-06` `EXPORT-01` `EXPORT-02` `EXPORT-03` `ENG-08` `SSH-01` `CONTRIB-01` `ANA-06` `ANA-07` `ANA-08` `ANA-09` `GAME-06` `GAME-07` `GAME-08` `GAME-09` `GAME-10` `PROV-05` `PROV-06` `PROV-07` `RCOMP-01` `PUB-01`

## Inventory mapping coverage

Every Inventory domain-table Mapping cell that names a Parity ID names an ID that exists **exactly once** in the Matrix. Mapping Parity IDs absent from the Matrix: **none**. Unresolved Inventory mapping remainder: **empty**. No Parity IDs were invented.

`SET-FIRST-LAUNCH` backticks Frozen `SHELL-06` as a Domain 01 engine-bootstrap link. `SHELL-06` is not a Parity Item and is not a missing Matrix row. Surviving mapping targets on that cell remain `PREF-01`, `WINDOW-01`, `LAYOUT-04`, and `APPEAR-01`.

`REVIEW-02`, `REVIEW-03`, and `REVIEW-06` are owned by Frozen `SGF-03-ADJ-TRYPLAY / SCORE / LADDER` (split keys `SGF-03-ADJ-TRYPLAY`, `SGF-03-ADJ-SCORE`, `SGF-03-ADJ-LADDER`).

Matrix IDs never cited from an Inventory mapping (not Inventory remainders; each has an implementation-gate rationale on the Matrix row):

| ID | Why this is not an Inventory remainder |
| --- | --- |
| `BASE-01`, `BASE-02` | R0 meta-inventory, not Java user-reachable Capabilities |
| `REL-01` | Maintainer preflight/dry-run; Inventory Maintainer-only section |
| `ANA-09` | Ticket 11 Deferred successor (named rich-analysis adapters); census `CAP-04-ANA-09` maps to `ANA-07` |

## Required fields

Every Matrix row has a non-empty Status, repository evidence, live/environment evidence, Remaining gap, and Acceptance.

`Depends on` under Matrix authority:

| Rows | `Depends on` |
| --- | --- |
| R0–R2 (14): `BASE-01` `BASE-02` `SGF-01`–`SGF-06` `RULE-01` `UI-01`–`UI-05` | Column absent. Matrix rule: R0–R2 keep historical columns. |
| R3 onward and Deferred (94) | Present on every row. Empty cells: none. `—` on `ENG-01`, `ANA-05`, `PREF-01`, `READ-03`, `REL-01`, `REL-02`, `REL-04`, `REL-09`. |

Named Item Start Prerequisite phrasing that is not a Parity ID, left unchanged:

- `APP-02`: `SGF-07` plus the passed R5 `APP-01` semantic gate, never `APP-01` Accepted.
- `SSH-01`: admitted adapter at promotion time.
- `ANA-08`: `not ANA-05`.

`GAME-03` Item Start Prerequisites remain `{GAME-01, GAME-04, GAME-05}`. `GAME-02` before `GAME-03` stays Plan Delivery Order.

Phase membership, Promotion Gates, Delivery Order, and Phase Gates were not assigned.

## Evidence classification

Repository evidence and live/environment evidence stay separate columns. No repository cell presents installed, release-environment, provider-live, engine-live, or other live evidence as completion.

Live/environment cells, classified from current wording:

| Class | Count | Meaning |
| --- | --- | --- |
| Not required | 33 | Deterministic repository behavior, or Deferred `Not required until admitted` |
| Recorded live | 9 | Named Ticket native cases / PIDs on Accepted R1/R2 rows |
| Environment note | 2 | `ENG-01` machine-specific assets; `ANA-05` native app-data path |
| Pending or Not run | 64 | Pending native/provider/engine/sidecar smoke, or **Not run** release-environment / Installed Live Evidence |
| **Total** | **108** | |

## Original Accepted contracts

The 16 Accepted IDs at `18c6d189b8b01069975c4c40ead63a010249cb8c` equal the current Accepted set. No extra item is Accepted. None of the original 16 left Accepted.

Protected fields ID, Capability, Status, both evidence columns, Remaining gap, and Acceptance are unchanged, with one named permitted history-context edit:

- **`BASE-02` repository evidence** still records stable items without percentages and still separates repository evidence from live/environment evidence. It now notes that the historical R0 inventory listed 41 items before the Tickets 08–14 successor expansion. Remaining gap and acceptance are unchanged.

Permitted non-protected movement:

- **`READ-03`** Phase R7 → R10 (map-allowed R4+ reorder). Capability, evidence, remaining gap, and acceptance are unchanged.
- **`ENG-01`**, **`ANA-05`**, **`READ-03`** gained `Depends on` `—` when Ticket 29 added that column from R3 onward. Protected fields are unchanged.

R0–R2 Accepted rows still have no `Depends on` column.

## Successor Items and false-parity check

The pre-audit Matrix at `18c6d189` listed 41 items. Every current ID absent from that set is a Successor Item from Tickets 08–14 or remainder Tickets 17–28. No original Accepted ID, scope, evidence, remaining gap, or acceptance was expanded in place.

Exclusion, owner-route, and implementation-only Inventory rows are not extra Accepted claims. Abandoned Swing/share/preload/flash/auto-analyze/hostname-wipe surfaces stay Exclusion or absorbed `ENG-02`. `PUB-01` keeps trial counters out of scope. `READ-03` remains the OCR-unsupported claim.

## Authority (not edited)

- `docs/JAVA_CAPABILITY_INVENTORY.md` mappings, Frozen IDs, and runtime-check status were not changed.
- `docs/MIGRATION_PLAN.md` phase membership, Promotion Gates, Delivery Order, and Phase Gates were not changed.
- No new Parity IDs. No new Accepted items.

## Destination writes

- `docs/PARITY_MATRIX.md`: Ticket 09 corpus counts, mapping-coverage / Matrix-only rationale, Accepted-preservation note, and item completeness index.
- This research file.
