# 10 — Reconcile the R3+ Dependency Roadmap

Independent Plan reconstruction for [Ticket 10](../issues/10-reconcile-r3-dependency-roadmap.md). Destination writes land in `docs/MIGRATION_PLAN.md`, with a membership pointer in `docs/PARITY_MATRIX.md`. Inventory mappings and Matrix `Depends on` cells were not edited. No Parity IDs were invented. Deferred items were not moved into numbered phases because an interface exists.

## Sources

| Tree | Path | Commit |
| --- | --- | --- |
| Java Migration Baseline v1 | `/home/dev/dev/weiqi/worktrees/lizzieyzy-next/java-baseline-v1` | `7b4027531c2b26062d0bfc27a040cc550cfbea4d` (not re-censused) |
| Next inventory baseline | this worktree | `18c6d189b8b01069975c4c40ead63a010249cb8c` (Accepted-field comparison only; not edited) |
| Current destination docs | this worktree | `c9f5749fa175fe01c95587c0aa5a9fd2fa835f2c` at reconstruction time |
| Reconciled Matrix corpus | `docs/PARITY_MATRIX.md` | Ticket 09 reconstruction: 108 items, 16 / 19 / 46 / 27 |
| Prior coverage-audit Deferred equality | `.scratch/migration-coverage-audit/research/30-final-traceability-reaudit.md` | Reference 27-item Matrix/Plan equality only |
| Nine disputed boundaries | `.scratch/migration-coverage-audit/issues/29-reconcile-start-dependencies.md` | Designated authorities |

Java source was not re-censused. Completeness is phase membership, Promotion Gates, Delivery Order, Phase Gates, and exits rebuilt from the Ticket 09 Matrix set. Historical 27-item Deferred equality is comparison evidence, not a stop condition. Item Start Prerequisites stay with Matrix `Depends on`.

## Method

1. Parse every Matrix table row whose status is Accepted, Partial, Missing, or Deferred. Count unique IDs, then count by status and by owner phase. Duplicate IDs stay open.
2. Parse the Plan Deferred table independently. Compare ID sets. Report computed counts, then compare the reference 27-item equality. Mismatches stay open with exact IDs.
3. Rebuild Plan phase membership from the complete Matrix set. Preserve R0–R2. Keep R3 next and owning Foreground Engine lifecycle. Order R4+ from Matrix `Depends on` and recorded dispositions. Do not promote Deferred items because an interface exists.
4. Compare the nine Ticket 29 boundaries against Matrix `Depends on` and Plan Delivery Order / Promotion Gates.
5. Classify every Plan Promotion Gate as additional to Matrix `Depends on`, or as a duplicate restatement. Duplicate restatements are Plan defects: replace them with `—` or with the additional product condition only.
6. Confirm every numbered phase has a Phase Gate and an exit; Delivery Order does not duplicate or strengthen Matrix `Depends on`.
7. Confirm semantic settings remain with behavior owners, the thirteen historical remainders have resolved item or exclusion decisions, and repository / live / release-environment / Installed Live evidence stay non-interchangeable.

## Computed Matrix set

**Computed unique Parity Items: 108.** Duplicate IDs: **none**.

Reference 108. Extra item rows: none. Missing item rows: none.

| Status | Computed | Reference | Extra / missing IDs |
| --- | --- | --- | --- |
| Accepted | 16 | 16 | none |
| Partial | 19 | 19 | none |
| Missing | 46 | 46 | none |
| Deferred | 27 | 27 | none |
| **Total** | **108** | **108** | **none** |

16 + 19 + 46 + 27 = 108.

### Matrix owner-phase membership

| Phase | Computed IDs | Count |
| --- | --- | --- |
| R0 | `BASE-01` `BASE-02` | 2 |
| R1 | `SGF-01`–`SGF-06` `RULE-01` | 7 |
| R2 | `UI-01`–`UI-05` | 5 |
| R3 | `ENG-01`–`ENG-07` | 7 |
| R4 | `ANA-01`–`ANA-05` `ANA-10`–`ANA-13` | 9 |
| R5 | `PREF-01` `SGF-07` `APP-02`–`APP-05` | 6 |
| R5 / R11 | `APP-01` | 1 |
| R6 | `SGF-08`–`SGF-14` `REVIEW-01`–`REVIEW-03` `REVIEW-07`–`REVIEW-09` | 13 |
| R7 | `LAYOUT-01`–`LAYOUT-04` `WINDOW-01` `WINDOW-02` `APPEAR-01` | 7 |
| R8 | `ENG-09` `ENG-10` | 2 |
| R9 | `GAME-01`–`GAME-05` | 5 |
| R10 | `PROV-01`–`PROV-04` `READ-01`–`READ-03` | 7 |
| R11 | `REL-01`–`REL-10` | 10 |
| Deferred | listed below | 27 |
| **Total** | | **108** |

R0–R2 = 14 frozen items. Numbered R3–R11 plus the split `APP-01` row = 67. Deferred = 27. 14 + 67 + 27 = 108.

Frozen Accepted foundations stay in their phases and are not reopened as exits: `ENG-01` in R3, `ANA-05` in R4, `READ-03` in R10.

`GUIDE-01` is Deferred, not an R7 member. `ENG-08` stays Deferred, not R3/R8. `ANA-09` stays Deferred, not R8. `CONTRIB-01`, `GAME-10`, `PROV-05`–`PROV-07`, and `PUB-01` stay Deferred and are not R10/R11 exits.

## Independent Deferred parse

**Matrix Deferred (27):** `I18N-01` `GUIDE-01` `SGF-15` `SGF-16` `REVIEW-04` `REVIEW-05` `REVIEW-06` `EXPORT-01` `EXPORT-02` `EXPORT-03` `ENG-08` `SSH-01` `CONTRIB-01` `ANA-06` `ANA-07` `ANA-08` `ANA-09` `GAME-06` `GAME-07` `GAME-08` `GAME-09` `GAME-10` `PROV-05` `PROV-06` `PROV-07` `RCOMP-01` `PUB-01`

**Plan Deferred (27):** the same 27 IDs, same order in the Deferred Queue table.

Only-Matrix: **none**. Only-Plan: **none**. Sets are equal. Reference 27 was compared afterward and was not a stop condition.

## Nine disputed boundaries

Matrix `Depends on` is the Item Start Prerequisite authority. The Plan owns Delivery Order and Promotion Gates.

| Item | Matrix `Depends on` (ISP) | Plan authority | Verdict |
| --- | --- | --- | --- |
| `GAME-03` | `{GAME-01, GAME-04, GAME-05}` | `GAME-02` before `GAME-03` is Delivery Order only | Pass |
| `SGF-15` | `{SGF-04, SGF-11, SGF-12}` | Promotion Gate `SGF-07` | Pass |
| `SGF-16` | `{SGF-01, SGF-11, SGF-14}` | — | Pass |
| `REVIEW-06` | `{SGF-04, REVIEW-01, RULE-01}` | — | Pass |
| `ANA-06` | `{ANA-01, ANA-03, ENG-05}` | — | Pass |
| `ANA-07` | `{ANA-02, ANA-03}` | Promotion Gate `APP-02` | Pass |
| `ANA-09` | `{ENG-09, ANA-04}` | Named engine/version product evidence; interface existence is insufficient | Pass |
| `PROV-05` | `{SGF-07}` | —; `PROV-04` is an exclusion, not a gate | Pass |
| `GAME-09` | `{GAME-01, CONTRIB-01}` | Promotion Gate: accepted `GAME-01` and accepted `CONTRIB-01`. `CONTRIB-01` is a `GAME-09` Item Start Prerequisite in the Matrix | Pass |

`GAME-03` may start when `{GAME-01, GAME-04, GAME-05}` are satisfied. Delivering Human-vs-Engine first is sequencing, not a start blocker.

## Promotion Gate reconciliation

Plan intro already states that a Promotion Gate is additional to Matrix `Depends on`, and that `—` means no additional condition. At reconstruction time, these Plan cells restated Matrix `Depends on` and were therefore a competing set:

| ID | Matrix ISP | Pre-reconciliation Plan gate | Disposition |
| --- | --- | --- | --- |
| `REVIEW-04` | `REVIEW-01` | `REVIEW-01` | `—` |
| `EXPORT-01` | `SGF-06` | current-game owner | `—` |
| `EXPORT-02` | `UI-01` | `UI-01` | `—` |
| `EXPORT-03` | `ANA-11`, `APP-05` | `ANA-11`, `APP-05` | `—` |
| `ENG-08` | `ENG-01`, `ENG-09` | `ENG-01`, `ENG-09` | `—` |
| `SSH-01` | `ENG-02`, `ENG-06`, `ENG-07`, `ENG-09`, admitted adapter | same | `—` |
| `CONTRIB-01` | `ENG-02`, `PREF-01`, `APP-03`, `REL-05` | same | `—` |
| `ANA-08` | `ANA-04`, `SGF-01`, `SGF-05`; not `ANA-05` | same | `—` |
| `GAME-06` | `GAME-03`, `GAME-05` | same | `—` |
| `GAME-07` | `GAME-02`, `GAME-03` | those plus a separate clock policy | additional only: a separately approved clock policy |
| `GAME-08` | `GAME-01`, `GAME-02`, `GAME-05` | those plus HumanSL-compatible profile | additional only: a HumanSL-compatible profile |
| `GAME-10` | `GAME-01`, `GAME-05`, `READ-01`, `READ-02`, `ENG-02`, `ENG-09`, `APP-03` | same | additional only: does not expand the R10 exit |
| `PROV-06` | `PROV-01`, `PROV-03` | same | `—` |
| `PROV-07` | `PROV-03`, `GAME-01`, `GAME-05`, `PREF-01`, `APP-03` | same | `—` |
| `RCOMP-01` | `ENG-02`, `ENG-07`, `ANA-01`, `ANA-03`, `PREF-01` | same | `—` |
| `PUB-01` | `APP-03` | shutdown/teardown owner | `—` |
| `REVIEW-05` | `UI-05` | accepted `UI-05` toggle | additional only: does not reopen accepted `UI-05` history |

Retained additional gates: `I18N-01` after functional migration; `GUIDE-01` later Guidance Producer; `SGF-15` `SGF-07`; `ANA-07` `APP-02`; `ANA-09` named product evidence; `GAME-09` accepted `GAME-01` and accepted `CONTRIB-01`; `PROV-05` exclusion note.

Every Deferred item has an explicit Promotion Gate after this reconciliation.

## Delivery Order versus Matrix `Depends on`

R9 already states that `GAME-02` before `GAME-03` is Delivery Order and does not block `GAME-03` once its Matrix prerequisites are satisfied.

R3 listed `ENG-03` after `ENG-05` / `ENG-06` / `ENG-07`. Matrix `ENG-03` depends only on `ENG-02`. Treating that list as a start blocker would strengthen Matrix `Depends on`. Destination write: `ENG-03` may start once `ENG-02` is satisfied; the later list position is Delivery Order only. `ENG-04` still waits for `ENG-03`.

How To Choose The Next Slice already picks work from Matrix ISPs plus Plan gates, not from Delivery Order as a hard start set. Critical Edges may cite Matrix ISPs for readability; those citations are not a second `Depends on` set.

## Phase Gates and exits

Every numbered phase has a Migration Phase Gate and an exit:

| Phase | Phase Gate | Exit |
| --- | --- | --- |
| R0 | Historical start; this phase has exited | `BASE-01` accepted; claims mapped |
| R1 | Historical; `BASE-01` accepted; this phase has exited | `SGF-01`–`SGF-06`, `RULE-01` accepted |
| R2 | Historical; R1 has exited; `UI-02` residual is not this gate | `UI-01`, `UI-03`–`UI-05` accepted; `UI-02` residual recorded |
| R3 | R2 has exited; `UI-02` residual is not this gate | `ENG-02`–`ENG-07` accepted |
| R4 | `ENG-02`–`ENG-07` accepted | `ANA-01`–`ANA-04`, `ANA-10`–`ANA-13` accepted |
| R5 | R1 and R2 exited; legal beside R3 | `PREF-01`, `SGF-07`, `APP-02`–`APP-05` accepted; `APP-01` semantic gate recorded |
| R6 | `SGF-07`, `PREF-01` accepted | `SGF-08`–`SGF-14`, `REVIEW-01`–`REVIEW-03`, `REVIEW-07`–`REVIEW-09` accepted |
| R7 | `PREF-01` accepted; `GUIDE-01` not a member | `LAYOUT-01`–`LAYOUT-04`, `WINDOW-01`, `WINDOW-02`, `APPEAR-01` accepted |
| R8 | `ENG-02`–`ENG-07` accepted | `ENG-09`, `ENG-10` accepted |
| R9 | `ENG-02`, `ENG-03`, `ENG-04`, `ENG-07`, `ENG-09`, `ENG-10`, `SGF-07`, `APP-04`, `REVIEW-03`, `PREF-01`, `ANA-04` accepted | `GAME-01`–`GAME-05` accepted |
| R10 | `SGF-07`, `PREF-01`, `APP-03` accepted | `PROV-01`–`PROV-04`, `READ-01`–`READ-03` accepted |
| R11 | `REL-01` anytime as maintainer gate; apply/handoff and `APP-01` final-acceptance follow named item gates | global `REL-01`–`REL-05`, `REL-09`, `REL-10`, and `APP-01` after both gates; platform branches independently |

R3 remains the named Next Executable Batch (Slice R3-A = `ENG-02`). Later phases that become dependency-legal must not be mixed into that batch.

`ANA-10` is R4's cross-phase exception (`PREF-01`, `APP-05`). R4 may execute other items while R5 proceeds and cannot exit before `ANA-10`. `ANA-11`–`ANA-13` persist through `PREF-01` and do not wait for `PREF-01` Accepted.

## Semantic settings

Owner-routed settings stay on behavior-owner rows. `PREF-01` is the durable file, atomic write, and categorized Preferences surface in R5. Autoload stays `ENG-06`. Analysis presentation/marker/chart/sub-board/replay stay `ANA-04` / `ANA-10`–`ANA-13`. Match start defaults stay `GAME-04`. Contribute watch stays `GAME-09`. No catch-all settings phase.

## Thirteen historical remainders

| Frozen ID | Resolved item or exclusion |
| --- | --- |
| `SET-NETWORK-PROXY` | Abandoned Java proxy UI/keys. Provider Network Policy on `PROV-01`–`PROV-07`; no proxy Parity Item |
| `SET-NEXT-MOVE` | Missing `ANA-10` |
| `SET-WINRATE-GRAPH` | Missing `ANA-11` |
| `SET-SUBBOARD` | Missing `ANA-12`; heatmap-on-sub and mouse-over freeze Abandoned |
| `SET-MAIN-PANEL` | Missing `REVIEW-09` and `WINDOW-02`; large-sub/large-WR presets and write-into-`C` Abandoned |
| `SET-HINT-AUTOANALYZE` | Excluded with abandoned `CAP-04-ANA-08`; `GUIDE-01` remains Deferred until a Guidance Producer is named |
| `SGF-03-ADJ-SAVE-MORE` | Deferred `EXPORT-03`; Sub-Board Image Export Abandoned |
| `SGF-03-ADJ-AUTOPLAY` | Missing `ANA-13`; Engine Continuation excluded |
| `SGF-03-ADJ-NEXT-HINT` | Missing `ANA-10` |
| `CAP-04-ENG-01` | Deferred `SSH-01` and `RCOMP-01`; neither expands `ENG-01` / `ENG-09` |
| `GM-CONTRIBUTE` | Deferred `CONTRIB-01` service plus Deferred `GAME-09` watcher |
| `CAP-06-YIKE-LIVE-CENTER` | Deferred `PROV-06` and `PROV-07`; public paths stay `PROV-01` / `PROV-03` |
| `CAP-06-READBOARD` | Deferred `GAME-10` for engine play-back; `READ-02` remains sync-only |

Open remainder count: **zero**.

## Evidence classes

Plan Evidence Model keeps five non-interchangeable classes: repository evidence; native/provider live evidence; Repository Release Evidence; release-environment evidence; Installed Live Evidence. Matrix live/environment cells stay a separate column. Source, test, and workflow paths are repository proof only. Dry-run and unsigned validation do not satisfy release-environment or Installed Live Evidence.

## Authority (not edited)

- `docs/JAVA_CAPABILITY_INVENTORY.md` mappings, Frozen IDs, and runtime-check status were not changed.
- `docs/PARITY_MATRIX.md` item rows, status, evidence, remaining gap, acceptance, and `Depends on` were not changed. The Current Phase Map gained a Ticket 10 membership pointer only.
- No new Parity IDs. No new Accepted items. No Deferred item entered a numbered phase.

## Destination writes

- `docs/MIGRATION_PLAN.md`: Ticket 10 reconciliation, additional-only Promotion Gates, R3 Delivery Order clarification for `ENG-03`, Critical Edges citation rule, named remainder closures.
- `docs/PARITY_MATRIX.md`: one membership pointer that Plan owns owner-phase membership and that Matrix `Depends on` remains the only Item Start Prerequisite set.
- This research file.
