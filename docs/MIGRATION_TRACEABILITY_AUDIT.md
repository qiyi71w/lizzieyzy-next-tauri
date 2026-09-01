# Migration Baseline v1 Traceability Audit

This file is the Ticket 11 corpus-wide traceability record. It is not a fourth migration authority. [`JAVA_CAPABILITY_INVENTORY.md`](JAVA_CAPABILITY_INVENTORY.md) owns Entry Points, defaults, persistence, failure behavior, source references, and runtime-check status. [`PARITY_MATRIX.md`](PARITY_MATRIX.md) owns status, evidence, remaining gap, acceptance, and Item Start Prerequisites. [`MIGRATION_PLAN.md`](MIGRATION_PLAN.md) owns phase membership, Deferred Promotion Gates, Delivery Order, Migration Phase Gates, and exits.

Independent reconstruction for [Ticket 11](../.scratch/migration-baseline-v1-capability-audit/issues/11-reaudit-final-migration-traceability.md). The three authorities were not rewritten. Java was not re-censused. Historical 139 / 108 / 16 / 19 / 46 / 27 figures were compared only after the sets below were rebuilt.

## Verdict

**Destination reached.**

Every total was recomputed from the current working-tree destination documents at `306e1db7b11541828c8eb3ae6e6a603f2b695f53`. Ticket 16’s 95 Parity Items and 19 Deferred items, and Ticket 08–10 reconstruction write-ups, were not used as current input.

| # | Check | Result |
| --- | --- | --- |
| 1 | Unique Frozen IDs in Inventory domain tables equal the census completeness index; remainder set empty; then compare 139. Registered Entry Points route once to one semantic owner | **Pass** (139 Frozen IDs, 135 Java Capabilities, remainder empty, 20 unique Cross-domain surfaces) |
| 2 | Every Capability row has default, persistence, failure, source-reference, and runtime-check-status; each named runtime check is a permitted dynamic-visibility / default / persistence / failure fact | **Pass** |
| 3 | Matrix statuses recomputed; Plan Deferred IDs parsed independently; then compare 108 and 16 / 19 / 46 / 27 and equal Deferred sets | **Pass** |
| 4 | Original sixteen Accepted contracts at `18c6d189b8b01069975c4c40ead63a010249cb8c` preserved across protected fields | **Pass** |
| 5 | Item Start Prerequisites, Promotion Gates, Delivery Order, Phase Gates, phase membership, and exits sit under their designated authority | **Pass** |
| 6 | Every evidence cell classified; repository references are not presented as installed, release-environment, provider-live, engine-live, or other live evidence | **Pass** |

## Sources

| Artifact | Path | Role |
| --- | --- | --- |
| Inventory | `docs/JAVA_CAPABILITY_INVENTORY.md` | Domain 01–07 Frozen-ID tables; Cross-domain routes; census completeness index |
| Parity Matrix | `docs/PARITY_MATRIX.md` | Status, evidence, remaining gap, acceptance, `Depends on`; Current Phase Map; Deferred table |
| Migration Plan | `docs/MIGRATION_PLAN.md` | Phase membership / Owns; Migration Phase Gates; Delivery Order; exits; Deferred Promotion Gates |
| Java Migration Baseline v1 | `/home/dev/dev/weiqi/worktrees/lizzieyzy-next/java-baseline-v1` | Detached HEAD `7b4027531c2b26062d0bfc27a040cc550cfbea4d` (identity check only; not re-censused) |
| Next inventory baseline | `18c6d189b8b01069975c4c40ead63a010249cb8c:docs/PARITY_MATRIX.md` | Original 16 Accepted contracts |
| Ticket 16 remainders | `.scratch/migration-coverage-audit/research/16-final-traceability-audit.md` | Named thirteen Frozen IDs to re-check closure; not a count source |
| Ticket 29 | `.scratch/migration-coverage-audit/issues/29-reconcile-start-dependencies.md` | Designated ISP / Promotion Gate / Delivery Order pairs |
| Spec | `.scratch/migration-coverage-audit/spec.md` | Authority split; reference passing result compared afterward |

Repository paths, tests, scripts, and workflows cited in the destination documents remain repository evidence only.

## Reproducible Method

1. Parse every markdown table under Inventory `## Domain 01` … `## Domain 07`. A Frozen ID is the first backtick token of the Frozen-ID column. Count unique primaries, then compare that set with backtick Frozen IDs under `## Census completeness index`. Duplicate IDs stay open.
2. Classify a Frozen-ID row as a Java Capability unless it sits in the Domain 07 heading table or the Entry-Point / Runtime-check cell names `Not a Java Entry Point` / `Not a Java Capability`. Report the Capability count, then compare reference 139 against the Frozen-ID set.
3. Reconstruct registered Entry Points from each Capability row’s Entry Point cell plus the Cross-domain routes table. A duplicate Frozen ID, a duplicate Cross-domain surface, or two semantic owners for one Cross-domain surface stays open.
4. For every Capability row, require non-empty Entry Points, Default / persistence, Failure / recovery, Frozen evidence, and Runtime-check. Accept a runtime check only when it is `Not required` or names dynamic visibility, default, persistence, or failure. Heading rows use the Domain 07 heading table.
5. Classify each Mapping cell as supported Parity Item coverage (names one or more Matrix IDs), explicit exclusion (cell starts `Exclusion`), absorbed redesign (cell starts `Absorbed redesign`), or unresolved remainder. Mapping Parity IDs must exist in the Matrix. Then compare remainder = ∅.
6. Parse every Matrix table whose Status is Accepted, Partial, Missing, or Deferred. Count unique IDs, then count by status. Parse the Plan Roadmap `### Deferred Queue` table independently. Compare Deferred ID sets. Then compare 108 and 16 / 19 / 46 / 27.
7. Diff the 16 Accepted rows at `18c6d189:docs/PARITY_MATRIX.md` against the current Matrix on ID, Capability, Status, both evidence columns, Remaining gap, and Acceptance. Phase movement is permitted only for named R4+ reordering.
8. Rebuild phase membership from Matrix `Phase` cells and the Current Phase Map, and from Plan `**Owns:**` / historical `Exit when:` lists. Confirm every numbered phase has a Migration Phase Gate and an exit. Confirm every Deferred Promotion Gate is present. Compare Ticket 29’s nine ISP / Promotion Gate / Delivery Order pairs with current cells. A Plan gate that restates Matrix `Depends on` as a competing start set stays open.
9. Classify every Matrix live/environment cell. Treat source, test, and workflow paths as repository evidence only; reject any such path presented as installed, release-environment, provider-live, engine-live, or other live proof.

## Quantitative Reconciliation

| Claim | How counted | Evidence now | Verdict |
| --- | --- | --- | --- |
| Inventory Frozen IDs | Unique first-column Frozen IDs in Domain 01–07 tables | **139**; no duplicates; census index lists the same 139 once | Pass |
| Java Capabilities | Frozen IDs except heading / “not a Java Capability” rows | **135** = 13+35+35+23+8+10+11 | Pass (see non-Capability rows) |
| Reference 139 | Compared after the Frozen-ID set was rebuilt | Frozen-ID set equals 139 | Pass |
| Mapping remainder | Mapping cells with no Matrix ID, no `Exclusion`, no `Absorbed redesign` | **0** | Pass |
| Parity Items | Matrix rows with status Accepted / Partial / Missing / Deferred | **108** unique IDs | Pass |
| Status split | Unique IDs by Status | 16 Accepted, 19 Partial, 46 Missing, 27 Deferred | Pass |
| Deferred queue | Matrix Deferred IDs vs Plan Deferred Queue IDs | Both **27**; sets equal; only-Matrix none; only-Plan none | Pass |
| Original Accepted set | Baseline `18c6d189` vs current Matrix | Same 16 IDs still Accepted; no extra Accepted | Pass |
| Thirteen historical remainders | Named Frozen IDs from Ticket 16, re-read from current Mapping cells | All 13 closed | Pass |

### Domain Frozen-ID roll-up

| Domain | Frozen IDs | Java Capabilities | Non-Capability Frozen IDs |
| --- | --- | --- | --- |
| 01 | 13 | 13 | — |
| 02 | 35 | 35 | — |
| 03 | 35 | 35 | — |
| 04 | 25 | 23 | `CAP-04-ANA-05`, `CAP-04-ANA-14` |
| 05 | 8 | 8 | — |
| 06 | 10 | 10 | — |
| 07 | 13 | 11 | `REL-C12`, `REL-C13` (heading table) |
| **Corpus** | **139** | **135** | **4** |

`CAP-04-ANA-05` and `CAP-04-ANA-14` name **Not a Java Entry Point** / **Not a Java Capability** in-row. `REL-C12` and `REL-C13` sit in the Domain 07 heading table (`Why it is not a Domain 07 Capability`). 139 was compared afterward and was not a stop condition.

### Mapping classification (135 Capability rows)

Unresolved remainder set: **empty**. Mapping Parity IDs absent from the Matrix: **none**. No Parity IDs were invented.

| Kind | Count | Rule used in this audit |
| --- | --- | --- |
| Supported Parity Item coverage, including splits | 114 | Mapping names one or more Matrix IDs and is not an `Exclusion` / `Absorbed redesign` lead-in |
| Explicit exclusion | 18 | Mapping cell starts with `Exclusion` |
| Absorbed redesign | 3 | Mapping cell starts with `Absorbed redesign` (`CAP-04-ANA-01`, `CAP-04-ANA-02`, `CAP-04-ANA-03` → `ENG-02`) |
| Unresolved remainder | 0 | — |
| **Capability rows** | **135** | |

Explicit exclusion rows: `SHELL-04` `SHELL-09` `SHELL-10` `SET-LOOKS` `SET-FRAME-FONT` `SET-THEME-APPLE-CLASSIC-CUSTOM` `SET-THEME-DIALOG` `SET-LAYOUT-MODE` `SET-LAYOUT-TOOLBAR` `SET-HINT-COMMENT-CTRL` `SET-HINT-AUTOANALYZE` `CAP-04-ENG-03` `CAP-04-ENG-09` `CAP-04-PREF-LIZZIE-CACHE` `CAP-04-ANA-06` `CAP-04-ANA-08` `CAP-04-ANA-10` `CAP-06-SHARE-CURRENT`.

Continuity Frozen IDs (not Capabilities) still name Matrix targets: `CAP-04-ANA-05` → `ANA-01` / `ENG-05`; `CAP-04-ANA-14` → `ANA-05`; `REL-C12` → `REL-09` / `REL-10`; `REL-C13` routes to Domain 02 `SET-CLEAR-PERSONAL-HISTORY` / `SGF-09`.

Matrix IDs never cited from an Inventory Capability mapping (not Inventory remainders):

| ID | Why this is not an Inventory remainder |
| --- | --- |
| `BASE-01`, `BASE-02` | R0 meta-inventory, not Java user-reachable Capabilities |
| `REL-01` | Maintainer preflight/dry-run; Inventory Maintainer-only section |
| `ANA-09` | Deferred named rich-analysis adapters; census `CAP-04-ANA-09` maps to `ANA-07` |

### Status roll-up (current Matrix)

| Status | Count | IDs |
| --- | --- | --- |
| Accepted | 16 | `BASE-01` `BASE-02` `SGF-01` `SGF-02` `SGF-03` `SGF-04` `SGF-05` `SGF-06` `RULE-01` `UI-01` `UI-03` `UI-04` `UI-05` `ENG-01` `ANA-05` `READ-03` |
| Partial | 19 | `UI-02` `ENG-05` `ANA-01` `ANA-02` `ANA-03` `ANA-04` `PREF-01` `SGF-07` `SGF-10` `REVIEW-01` `REVIEW-07` `APPEAR-01` `PROV-01` `PROV-02` `READ-01` `READ-02` `REL-01` `REL-05` `REL-10` |
| Missing | 46 | `ENG-02` `ENG-03` `ENG-04` `ENG-06` `ENG-07` `ANA-10` `ANA-11` `ANA-12` `ANA-13` `APP-01` `APP-02` `APP-03` `APP-04` `APP-05` `SGF-08` `SGF-09` `SGF-11` `SGF-12` `SGF-13` `SGF-14` `REVIEW-02` `REVIEW-03` `REVIEW-08` `REVIEW-09` `LAYOUT-01` `LAYOUT-02` `LAYOUT-03` `LAYOUT-04` `WINDOW-01` `WINDOW-02` `ENG-09` `ENG-10` `GAME-01` `GAME-02` `GAME-03` `GAME-04` `GAME-05` `PROV-03` `PROV-04` `REL-02` `REL-03` `REL-04` `REL-06` `REL-07` `REL-08` `REL-09` |
| Deferred | 27 | listed below |
| **Total** | **108** | |

16 + 19 + 46 + 27 = 108. Matrix item completeness index lists the same 108 IDs once. Duplicate Matrix IDs: **none**.

**Matrix Deferred (27):** `I18N-01` `GUIDE-01` `SGF-15` `SGF-16` `REVIEW-04` `REVIEW-05` `REVIEW-06` `EXPORT-01` `EXPORT-02` `EXPORT-03` `ENG-08` `SSH-01` `CONTRIB-01` `ANA-06` `ANA-07` `ANA-08` `ANA-09` `GAME-06` `GAME-07` `GAME-08` `GAME-09` `GAME-10` `PROV-05` `PROV-06` `PROV-07` `RCOMP-01` `PUB-01`

**Plan Deferred (27):** the same 27 IDs, same order in the Roadmap Deferred Queue table.

Only-Matrix: **none**. Only-Plan: **none**. Sets are equal. Reference 27 was compared afterward.

## Entry Point routing

Duplicate Frozen IDs: **none**. Every Capability row has a non-empty Entry Points cell. Heading rows `REL-C12` / `REL-C13` are not Capabilities.

Cross-domain routes table: **20** unique surfaces, **0** duplicate surfaces. Each row has one Routing owner and one Semantic owner / mapping cell. Shared physical controls therefore have one canonical Cross-domain row; census rows that only point at the owner remain link rows, not a second semantic Capability.

| Surface | Semantic owner |
| --- | --- |
| Native file activation / argv / drop of one kifu | `APP-01` / `APP-02` → `SGF-07` (`SGF-08` for GIB) |
| Multi-file drop | `ANA-07` intake |
| Autoload / engine picker / repair chip | `ENG-06` / `ENG-02` / `ENG-07`; `SHELL-02` / `SHELL-07` record process-start |
| Window close / File › Exit | `APP-03` |
| N / 新建 | Next empty document `SGF-10`; Java genmove is `GM-HUMAN-GENMOVE` |
| Space | Match Stop `GAME-01`/`GAME-02`; ponder is Deferred `ANA-06` |
| P | Review pass `SGF-04`; match pass `GAME-02`/`GAME-03` |
| Help › About / Check Update / Diagnostics / Clear personal data | `REL-10` / `REL-03` / `REL-09`; clear-personal-data is Domain 02 |
| Coordinates / move numbers | `REVIEW-07` |
| Suggestion / Kata overlay prefs | `ANA-04` |
| Contribute entry and actions | `CONTRIB-01` / `GAME-09` |
| Network proxy | `REL-03` update OS proxy; `PROV-01`–`PROV-07` Provider Network Policy |
| Remote engine execution / compute center | Deferred `SSH-01` / `RCOMP-01` |
| Default board size / Ctrl+I | `SGF-10` / `GAME-04`; names/komi `SGF-13` |
| URL / provider / readboard; File › 打开在线链接 / `Q` | Domain 06 `CAP-06-ONLINE-URL` → `PROV-01` / `PROV-03` / Deferred `PROV-05` |
| Bundled engine auto-setup | `REL-05` |
| File-association packaging evidence | `APP-01` semantics; packaging is `REL-04` / `REL-C09` |
| Match SGF handoff | `GAME-05` through current-game owner |
| `APP-04` recovery | Never restores engine runs, jobs, matches, or provider sessions |
| View 下一手 / `J` | `ANA-10` |

No Cross-domain row creates two semantic Capability owners for one user goal.

## Required Inventory fields and runtime checks

Every Capability row (135) has non-empty Entry Points, Default / persistence, Failure / recovery, Frozen evidence, and Runtime-check cells. Empty required fields: **none**.

`REL-C12` and `REL-C13` use the Domain 07 heading table. `CAP-04-ANA-05` and `CAP-04-ANA-14` keep continuity columns; Runtime-check records that they are not Java Capabilities.

Runtime-check classes (139 Frozen IDs):

| Class | Count |
| --- | --- |
| `Not required` | 97 |
| Named permitted fact | 38 |
| Not a Java Capability | 2 |
| Heading table (no runtime column) | 2 |
| Unclassified | 0 |
| **Total Frozen IDs** | **139** |

Named permitted facts stay inside dynamic visibility, default (including dynamic default), persistence, or failure. No runtime check was executed. Packaged/installed, provider-live, sidecar-live, and `GAME-10` / `READ-02` sidecar-confirmation claims remain Matrix / `REL-*` evidence classes.

## Original Accepted contracts

The 16 Accepted IDs at `18c6d189b8b01069975c4c40ead63a010249cb8c` equal the current Accepted set: `BASE-01` `BASE-02` `SGF-01`–`SGF-06` `RULE-01` `UI-01` `UI-03` `UI-04` `UI-05` `ENG-01` `ANA-05` `READ-03`.

Protected fields ID, Capability, Status, both evidence columns, Remaining gap, and Acceptance are unchanged, with one named permitted history-context edit:

- **`BASE-02` repository evidence** still records stable items without percentages and still separates repository evidence from live/environment evidence. It now notes that the historical R0 inventory listed 41 items before the Tickets 08–14 successor expansion. Remaining gap and acceptance are unchanged.

Permitted non-protected movement:

- **`READ-03`** Phase R7 → R10 (map-allowed R4+ reorder). Capability, evidence, remaining gap, and acceptance are unchanged.
- **`ENG-01`**, **`ANA-05`**, **`READ-03`** carry `Depends on` `—` on R3+ rows. Protected fields are unchanged.

No extra item is Accepted. None of the original 16 left Accepted.

## Thirteen historical remainders

Re-read from current Mapping cells. Open remainder count: **zero**.

| Frozen ID | Current mapping |
| --- | --- |
| `SET-NETWORK-PROXY` | Update OS proxy `REL-03`; Provider Network Policy on `PROV-01`–`PROV-07`; Java proxy UI/keys Abandoned; no proxy Parity Item |
| `SET-NEXT-MOVE` | `ANA-10` |
| `SET-WINRATE-GRAPH` | `ANA-11`; chart presence `UI-01` |
| `SET-SUBBOARD` | `ANA-12`; heatmap-on-sub and mouse-over freeze Abandoned |
| `SET-MAIN-PANEL` | `REVIEW-09` and `WINDOW-02`; large-sub/large-WR presets and write-into-`C` Abandoned |
| `SET-HINT-AUTOANALYZE` | Exclusion with abandoned `CAP-04-ANA-08`; `GUIDE-01` is not bound to this path |
| `SGF-03-ADJ-SAVE-MORE` | Deferred `EXPORT-01` / `EXPORT-02` / `EXPORT-03`; Swing raw saves and Sub-Board Image Export Abandoned |
| `SGF-03-ADJ-AUTOPLAY` | `ANA-13`; Engine Continuation excluded; `UI-05` / Deferred `REVIEW-05` remain Review Autoplay |
| `SGF-03-ADJ-NEXT-HINT` | `ANA-10` |
| `CAP-04-ENG-01` | `ENG-01` / `ENG-09` plus Deferred `SSH-01` / `RCOMP-01` |
| `GM-CONTRIBUTE` | Deferred `CONTRIB-01` plus Deferred `GAME-09` |
| `CAP-06-YIKE-LIVE-CENTER` | Public `PROV-01` / `PROV-03`; Deferred `PROV-06` / `PROV-07` |
| `CAP-06-READBOARD` | `READ-01` / `READ-02` / `READ-03`; Deferred `GAME-10` for engine play-back |

Inventory `## Parity decision-gap closure after Tickets 17–28`, Matrix `## Ticket 16 Disposition-To-Item Closure`, and Plan `### Traceability Gap Closure` all state that no gap remains. Those prose closures match the rebuilt remainder set.

## Dependency authority

Matrix `Depends on` is the sole Item Start Prerequisite authority. The Plan owns Deferred Promotion Gates, Delivery Order, Migration Phase Gates, phase membership, and exits.

R0–R2 (14 rows) have no `Depends on` column. Every R3+ and Deferred row (94) has a non-empty `Depends on` cell. `—` on `ENG-01` `ANA-05` `PREF-01` `READ-03` `REL-01` `REL-02` `REL-04` `REL-09`. Named non-ID phrasing left in place: `APP-02` (passed R5 `APP-01` semantic gate); `SSH-01` (admitted adapter); `ANA-08` (`not ANA-05`).

Required Matrix fields (status, capability, repository evidence, live/environment evidence, remaining gap, acceptance, phase): empty cells **none**.

### Nine Ticket 29 boundaries

| Item | Matrix `Depends on` (ISP) | Plan authority | Verdict |
| --- | --- | --- | --- |
| `GAME-03` | `{GAME-01, GAME-04, GAME-05}` | `GAME-02` before `GAME-03` is Delivery Order only (Critical Edges + R9) | Pass |
| `SGF-15` | `{SGF-04, SGF-11, SGF-12}` | Promotion Gate `SGF-07` | Pass |
| `SGF-16` | `{SGF-01, SGF-11, SGF-14}` | — | Pass |
| `REVIEW-06` | `{SGF-04, REVIEW-01, RULE-01}` | — | Pass |
| `ANA-06` | `{ANA-01, ANA-03, ENG-05}` | — | Pass |
| `ANA-07` | `{ANA-02, ANA-03}` | Promotion Gate `APP-02` | Pass |
| `ANA-09` | `{ENG-09, ANA-04}` | Named engine/version product evidence; interface existence is insufficient | Pass |
| `PROV-05` | `{SGF-07}` | —; `PROV-04` is an exclusion, not a gate | Pass |
| `GAME-09` | `{GAME-01, CONTRIB-01}` | Promotion Gate: accepted `GAME-01` and accepted `CONTRIB-01` | Pass |

`GAME-03` may start when `{GAME-01, GAME-04, GAME-05}` are satisfied. Delivering Human-vs-Engine first is sequencing, not a start blocker.

### Promotion Gates and Delivery Order

Every Deferred item has an explicit Promotion Gate. Empty Promotion Gate cells: **none**. `—` means no additional condition. No Promotion Gate restates Matrix `Depends on` as a competing start set.

Additional gates (not ISP duplicates): `I18N-01` after functional migration; `GUIDE-01` later Guidance Producer; `SGF-15` `SGF-07`; `REVIEW-05` does not reopen accepted `UI-05`; `ANA-07` `APP-02`; `ANA-09` named product evidence; `GAME-07` separately approved clock policy; `GAME-08` HumanSL-compatible profile; `GAME-09` accepted `GAME-01` and accepted `CONTRIB-01`; `GAME-10` does not expand the R10 exit; `PROV-05` exclusion note.

R3: `ENG-03` may start once Matrix `ENG-02` is satisfied; listing it after `ENG-05` / `ENG-06` / `ENG-07` is Delivery Order only. `ENG-04` still waits for `ENG-03`.

### Phase membership, Phase Gates, and exits

Matrix `Phase` cells, Current Phase Map, and Plan `**Owns:**` (R3–R11) agree:

| Phase | Membership | Count |
| --- | --- | --- |
| R0 | `BASE-01` `BASE-02` | 2 |
| R1 | `SGF-01`–`SGF-06` `RULE-01` | 7 |
| R2 | `UI-01`–`UI-05` | 5 |
| R3 | `ENG-01`–`ENG-07` | 7 |
| R4 | `ANA-01`–`ANA-04` `ANA-10`–`ANA-13`; frozen `ANA-05` | 9 |
| R5 | `PREF-01` `SGF-07` `APP-02`–`APP-05`; `APP-01` semantic gate | 6 + split `APP-01` |
| R6 | `SGF-08`–`SGF-14` `REVIEW-01`–`REVIEW-03` `REVIEW-07`–`REVIEW-09` | 13 |
| R7 | `LAYOUT-01`–`LAYOUT-04` `WINDOW-01` `WINDOW-02` `APPEAR-01` | 7 |
| R8 | `ENG-09` `ENG-10` | 2 |
| R9 | `GAME-01`–`GAME-05` | 5 |
| R10 | `PROV-01`–`PROV-04` `READ-01`–`READ-03` | 7 |
| R11 | `REL-01`–`REL-10`; final `APP-01` acceptance | 10 |
| Deferred | Plan Deferred Queue | 27 |
| **Total** | | **108** |

`APP-01` is the split `R5 / R11` row. Frozen Accepted foundations stay in their phases: `ENG-01` in R3, `ANA-05` in R4, `READ-03` in R10. `GUIDE-01` and `ENG-08` stay Deferred.

Every numbered phase has a Migration Phase Gate and an exit. R0–R2 gates are historical and already exited; their exits are the `Exit when:` lists (`BASE-01`; `SGF-01`–`SGF-06` + `RULE-01`; `UI-01` `UI-03`–`UI-05` with `UI-02` residual recorded). R3–R11 use `**Migration Phase Gate:**` and `**Exit when:**`. R3 remains the next executable phase and owns Foreground Engine lifecycle.

## Evidence classification

Matrix live/environment cells, recomputed from current wording:

| Class | Count | Meaning |
| --- | --- | --- |
| Not required | 5 | Deterministic repository behavior on Accepted R0/R1 rows (`BASE-01` `BASE-02` `SGF-01` `SGF-02` `RULE-01`) |
| Not required until admitted | 27 | All Deferred rows |
| Conditional not required | 1 | `READ-03`: live OCR smoke is not required until an OCR-capable runtime is supported |
| Recorded Ticket / native | 9 | Named Ticket cases / PIDs on `SGF-03`–`SGF-06` and `UI-01`–`UI-05` |
| Environment note | 2 | `ENG-01` machine-specific assets; `ANA-05` native app-data path |
| Pending | 54 | Pending native / provider / engine / sidecar smoke |
| Not run | 10 | Release-environment or Installed Live Evidence **Not run**, including `REL-04` per Canonical Artifact |
| **Total** | **108** | |

No source, test, or workflow path appears in a live/environment cell. No repository-evidence cell presents installed, release-environment, provider-live, engine-live, or other live completion. Plan Evidence Model keeps repository evidence, native/provider live evidence, Repository Release Evidence, release-environment evidence, and Installed Live Evidence non-interchangeable.

## Authority (not rewritten)

- Inventory Frozen IDs, Entry Points, defaults, persistence, failure behavior, source references, runtime-check status, and Mapping cells were not changed.
- Matrix item rows, status, evidence, remaining gap, acceptance, and `Depends on` were not changed.
- Plan phase membership, Promotion Gates, Delivery Order, Phase Gates, and exits were not changed.
- No new Parity IDs. No new Accepted items. No Deferred item entered a numbered phase.

## Destination writes

- This file. Inventory mappings, Matrix item rows, and Plan sequencing were not rewritten.

## Conclusion

All Ticket 11 checks pass on the current working-tree destination documents.

- Frozen census: **139** unique Frozen IDs (135 Java Capabilities plus four named non-Capability rows); each maps to a supported Parity Item, an explicit exclusion, or an absorbed redesign; remainder **0**.
- Parity corpus: **108** items (16 / 19 / 46 / 27); Deferred sets identical in Matrix and Plan. Counts were recomputed; Ticket 16’s 95/19 were not reused.
- Original 16 Accepted contracts preserved against `18c6d189b8b01069975c4c40ead63a010249cb8c`; `BASE-02` evidence adds historical successor context; `READ-03` phase move is named.
- Ticket 29 authorities hold for the nine Matrix/Plan comparisons. Phase Gates, exits, Promotion Gates, and Delivery Order sit on the Plan; Item Start Prerequisites sit on Matrix `Depends on`.
- Evidence classes stay unmixed.
