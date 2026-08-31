# Reconcile Parity Matrix and Migration Plan Start Dependencies

Type: grilling
Status: resolved
Blocked by: 16

## Question

After Ticket 15, which document is canonical for item `Depends on` when `docs/PARITY_MATRIX.md` and `docs/MIGRATION_PLAN.md` disagree? In particular, must `GAME-03` wait for `GAME-02` as Ticket 15 and the Plan critical edge state, or may PK start from `GAME-01`/`GAME-04`/`GAME-05` only as the Matrix row states? Which dependency sets are the admission gates for Deferred `SGF-15`, `SGF-16`, `REVIEW-06`, `ANA-06`, `ANA-07`, `ANA-09`, `PROV-05`, and `GAME-09`?

## Answer

### Dependency language and authority

The overloaded dependency language is split into four domain concepts, recorded in [`CONTEXT.md`](../../../CONTEXT.md):

- **Item Start Prerequisite** is an intrinsic capability dependency that must be satisfied before work on a Parity Item begins. `docs/PARITY_MATRIX.md` is its sole authority; each R3+ row's `Depends on` cell contains only this set.
- **Deferred Promotion Gate** is an additional product or decision condition that must pass before a Deferred item enters executable migration scope.
- **Delivery Order** sequences coherent delivery but does not prohibit an earlier parallel start.
- **Migration Phase Gate** controls entry to or exit from a phase rather than one item.

`docs/MIGRATION_PLAN.md` is the sole authority for Deferred Promotion Gates, Delivery Order, Migration Phase Gates, phase membership, and exit criteria. It may reference Matrix-owned Item Start Prerequisites but does not maintain a competing set. Ticket 15 remains historical; this ticket supersedes its dependency wording where the destination documents previously disagreed.

### `GAME-03`

`GAME-03` has Item Start Prerequisites `{GAME-01, GAME-04, GAME-05}`. It may begin when those are satisfied without waiting for `GAME-02`.

The Plan still delivers `GAME-02` before integrated `GAME-03`, but `GAME-02 → GAME-03` is Delivery Order only. It is not an Item Start Prerequisite.

### Deferred items

Promotion Gates below are additional to the Matrix-owned Item Start Prerequisites. `—` means no additional promotion condition.

| Item | Item Start Prerequisites — Matrix | Deferred Promotion Gate — Plan |
| --- | --- | --- |
| `SGF-15` | `SGF-04`, `SGF-11`, `SGF-12` | `SGF-07` |
| `SGF-16` | `SGF-01`, `SGF-11`, `SGF-14` | — |
| `REVIEW-06` | `SGF-04`, `REVIEW-01`, `RULE-01` | — |
| `ANA-06` | `ANA-01`, `ANA-03`, `ENG-05` | — |
| `ANA-07` | `ANA-02`, `ANA-03` | `APP-02` |
| `ANA-09` | `ENG-09`, `ANA-04` | Controlled product evidence for a named engine/version; interface existence alone is insufficient. |
| `PROV-05` | `SGF-07` | —; `PROV-04` remains an exclusion, not a positive gate. |
| `GAME-09` | `GAME-01`, `CONTRIB-01` | Accepted `GAME-01` and Accepted `CONTRIB-01`, unchanged from Ticket 24. |

`APP-03` is not an Item Start Prerequisite or Deferred Promotion Gate for `PROV-05`; it may remain relevant only through Plan-owned Delivery Order or a Migration Phase Gate.

This ticket changes documentation authority and dependency classification only. It adds no production implementation and no ADR: the audit ticket and destination documents fully record the reversible planning decision.
