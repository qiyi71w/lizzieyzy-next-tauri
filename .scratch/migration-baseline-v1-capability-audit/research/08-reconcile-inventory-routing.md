# 08 — Reconcile Inventory Corpus and Cross-domain Routing

Independent corpus reconstruction for [Ticket 08](../issues/08-reconcile-inventory-routing.md). Destination writes land only in `docs/JAVA_CAPABILITY_INVENTORY.md`. Matrix statuses and Plan sequencing were not edited.

## Sources

| Tree | Path | Commit |
| --- | --- | --- |
| Java Migration Baseline v1 | `/home/dev/dev/weiqi/worktrees/lizzieyzy-next/java-baseline-v1` | `7b4027531c2b26062d0bfc27a040cc550cfbea4d` (not re-censused) |
| Next inventory baseline | this worktree | `18c6d189b8b01069975c4c40ead63a010249cb8c` (Accepted-field comparison only; not edited) |
| Current destination docs | this worktree | `760c78b962098ef73b02c8a1a67f4be9ea738979` at reconstruction time |
| Domain censuses | `.scratch/migration-baseline-v1-capability-audit/research/01` … `07` | Tickets 01–07 independent counts |
| Prior coverage-audit census | `.scratch/migration-coverage-audit/research/` | Reference 139 headings only |

Java source was not re-censused. Completeness is the unique set reconstructed from the seven Inventory domain tables after Tickets 01–07. Historical 139 is comparison evidence, not a stop condition.

## Method

1. Parse every Frozen-ID table under `## Domain 01` … `## Domain 07` in `docs/JAVA_CAPABILITY_INVENTORY.md`. Count unique Frozen IDs, then count Capabilities using each domain's independent definition (headings and “not a Java Capability” rows are named, not forced into the Capability set).
2. Reconstruct registered Entry Points from each Capability row's Entry Point cell plus the Cross-domain routes table and the seven research owner-routing tables. A shared physical control is a duplicate only when two rows claim semantic ownership of the same user goal. Link-only rows, headings, and context-routed keys are named.
3. Check every Capability row for defaults (the Default / persistence cell), persistence in that same cell, failure / recovery, frozen evidence, and runtime-check status. A runtime check is accepted only when it names dynamic-visibility, default, persistence, or failure, or is `Not required` / not a Java Capability.
4. Classify each Mapping cell as supported Parity Item coverage, explicit exclusion, absorbed redesign, or unresolved remainder. Mapping targets must already exist in `docs/PARITY_MATRIX.md`; this ticket does not invent IDs.
5. Rebuild the census completeness index from the computed Capability set. Compare 139 afterward.

## Computed unique Capability set

**Computed unique Java Capabilities: 135.**

Reference 139. Extra Capability rows: none.

Exact rows that are Frozen IDs or completeness headings but **not** Capabilities (the mismatch versus 139-as-Capabilities):

| Frozen ID | Domain table | Why it is not a Capability |
| --- | --- | --- |
| `CAP-04-ANA-05` | Domain 04 | Next one-shot JSONL job; Java has no named one-shot. Kept for Ticket 11 mapping continuity. |
| `CAP-04-ANA-14` | Domain 04 | Next SQLite cache; no Java analogue. Kept for Ticket 11 mapping continuity. |
| `REL-C12` | Domain 07 heading table | Maintainer/docs. In-app links are Entry Points of `REL-C11`. |
| `REL-C13` | Domain 07 heading table | Help clear-personal-data is Domain 02 `SET-CLEAR-PERSONAL-HISTORY`. |

Unique Frozen IDs in domain tables: **139** (no duplicates). Unique Frozen ID set equals reference 139. Unique Capability set is 135 = 13+35+35+23+8+10+11 from Tickets 01–07. 139 was compared afterward and was not a stop condition.

Domain roll-up:

| Domain | Computed Capabilities | Frozen IDs in table | Reference (coverage-audit headings) | Extra / missing as Capabilities |
| --- | --- | --- | --- | --- |
| 01 | 13 | 13 | 13 | none |
| 02 | 35 | 35 | 35 | none |
| 03 | 35 | 35 | 35 | none |
| 04 | 23 | 25 | 25 | `CAP-04-ANA-05`, `CAP-04-ANA-14` not Java Capabilities |
| 05 | 8 | 8 | 8 | none |
| 06 | 10 | 10 | 10 | none |
| 07 | 11 | 13 (11 + 2 headings) | 13 | `REL-C12`, `REL-C13` headings |
| **Corpus** | **135** | **139** | **139** | those four rows |

## Entry Point routing

Registered Entry Points were reconstructed from domain-table cells. Duplicate Frozen IDs: **none**. Missing registered Entry Points versus the seven completed domain tables: **none**. Java was not re-walked.

Shared physical controls that are **not** duplicate Capability ownership:

| Surface | Census rows | Semantic owner | Classification |
| --- | --- | --- | --- |
| File › 打开在线链接 / `Q` / `OnlineDialog` | `SGF-03-ADJ-URL`, `CAP-06-ONLINE-URL` | Domain 06 `CAP-06-ONLINE-URL` (`PROV-01` / `PROV-03` / deferred `PROV-05`); install `SGF-07` | Foreign-owned Domain 03 link row |
| Provider / readboard `loadFile` / `loadSgfString*` | `SGF-03-ADJ-PROVIDER`, Domain 06 `CAP-06-*` | Domain 06 fetch; `SGF-07` install | Foreign-owned Domain 03 link row |
| Autoload radios / startup engine choice | `SHELL-02`, `CAP-04-ENG-02` | Domain 04 `CAP-04-ENG-02` → `ENG-06`; Domain 01 records the process-start Entry Point | Routed pair; one semantic owner |
| Overlay repair chip | `SHELL-07`, `CAP-04-ENG-05`, `CAP-04-ANA-15` | `SHELL-07` owns the click; Domain 04 owns start-failure copy | Split, not two owners of the chip |
| Space / `togglePonderMannul` / Game › breakGame | `SGF-03-BOARD-INTENT`, `CAP-04-ANA-04`, `GM-MATCH-STOP`, `CAP-04-ENG-06` | Match live → `GAME-01` / `GAME-02` Stop; otherwise ponder `CAP-04-ANA-04` → Deferred `ANA-06`. Space is not engine kill | Context route (already in Cross-domain routes) |
| `P` / Game 停一手 / `board.pass()` | `SGF-03-EDIT`, `GM-MATCH-PASS`, `GM-HUMANSL` | Review pass `SGF-04`; match pass `GAME-02` / `GAME-03`; HumanSL `humanPass()` | Context route |
| Java `N` | `SGF-03-ADJ-NEW` (explicitly not `N`), `GM-HUMAN-GENMOVE` | Java `N` is Domain 05 genmove; File New / toolbar New is `SGF-10` | Named non-duplicate |
| Ctrl+I / 设置棋盘大小 | `SET-BOARD-SIZE`, `SGF-03-ADJ-META` | Default / New Document size `SGF-10` / `GAME-04`; GameInfo names/komi `SGF-13` | Split |
| View 下一手 / `J` | `SET-NEXT-MOVE`, `SGF-03-ADJ-NEXT-HINT` | `ANA-10` | Settings census vs overlay census; one semantic owner |
| Help › About / ConfigDialog2 tab 2 | `SET-CONFIG-DIALOG-DISPLAY`, `REL-C11` | `REL-C11` → `REL-10`; the dialog surface stays `PREF-01` chrome | Routed chrome vs identity |
| Help › clear personal data | `SET-CLEAR-PERSONAL-HISTORY`, heading `REL-C13` | Domain 02 | Heading, not a second Capability |
| `firstTimeLoad` / hostname | `SHELL-06`, `SET-FIRST-LAUNCH`, `SET-RESET-HINTS`, `REL-C03` | Engine/profile bootstrap vs persist wipe vs hint reset vs bundled-engine detect | Distinct goals on one startup path |
| Contribute menu visibility / occupancy | `SET-CONTRIBUTE-MENU-VIS`, `GM-CONTRIBUTE` | Deferred `CONTRIB-01` service; Deferred `GAME-09` watch; no general visibility preference | Routed |
| One-file drop / argv | `SHELL-12`, `SHELL-03` | `APP-02` / `APP-01` → `SGF-07` (`SGF-08` for GIB) | Unique shell owners; File Open chooser is `SGF-03-ADJ-OPEN` |
| Help Check Update / Diagnostics / Stop Full Trace | Domain 07 `REL-C05` / `REL-C10` | Domain 07 | Domain 01 records chrome only |

No cross-domain route creates two semantic Capability owners for one user goal. The foreign-owned rows above remain in their original census tables (split rule: keep the original row and list the owner in Mapping). The Inventory Cross-domain table keeps one canonical row per physical surface; Frozen IDs named there are census or link labels, not a second owner.

## Required fields

Every Capability row (`135`) has a non-empty Entry Points cell, Default / persistence cell, Failure / recovery cell, Frozen evidence cell, and Runtime-check cell.

`REL-C12` and `REL-C13` use the Domain 07 heading table (not Capability columns). `CAP-04-ANA-05` and `CAP-04-ANA-14` keep continuity columns; Runtime-check records that they are not Java Capabilities.

Runtime-check classes after this ticket:

- `Not required` (or not a Java Capability): remainder of the 135 + the two Next-only attachments
- Named permitted fact: dynamic visibility, default (including “dynamic default”), persistence, or failure

Nineteen Domain 04/06/07 cells named a source gap but omitted the permitted class word (including `CAP-04-PREF-LIZZIE-CACHE` “visibility”). Inventory owns runtime-check status; those cells now name the class. After review, `CAP-04-ENG-07` and `CAP-04-ANA-12` live switch-key facts are tagged failure, and `CAP-06-SYNC-SETTINGS` live apply-to-running-session is tagged default. No runtime check was executed.

## Mapping classification

Computed from Mapping cells after the three absorbed Domain 04 rows name `ENG-02`. Unresolved remainder set: **empty**. No Parity IDs were invented.

| Kind | Count | Meaning |
| --- | --- | --- |
| Supported Parity Item coverage (including explicit splits) | 125 | Mapping names one or more existing Matrix IDs |
| Explicit exclusion (Abandoned / Swing-only; no surviving ID required) | 7 | `SET-LOOKS`, `SET-FRAME-FONT`, `SET-THEME-APPLE-CLASSIC-CUSTOM`, `SET-HINT-COMMENT-CTRL`, `CAP-04-ENG-03`, `CAP-04-ENG-09`, `CAP-06-SHARE-CURRENT` |
| Absorbed redesign | 3 | `CAP-04-ANA-01`, `CAP-04-ANA-02`, `CAP-04-ANA-03` → `ENG-02` |
| Unresolved remainder | 0 | — |
| **Capability rows** | **135** | |
| Continuity Frozen IDs (not Capabilities) | 2 | `CAP-04-ANA-05` → `ANA-01`; `CAP-04-ANA-14` → `ANA-05` |
| Headings (not Capabilities) | 2 | `REL-C12` absorbed `REL-09`/`REL-10`; `REL-C13` routes to Domain 02 |

Every mapping target exists in `docs/PARITY_MATRIX.md`. Mapping IDs absent from the Matrix: **none**.

Matrix IDs never cited from an Inventory mapping (not Inventory remainders; Ticket 09 owns Matrix-side ownership):

| ID | Why this is not an Inventory remainder |
| --- | --- |
| `BASE-01`, `BASE-02` | R0 meta-inventory, not Java user-reachable Capabilities |
| `REL-01` | Maintainer preflight/dry-run; Inventory Maintainer-only section |
| `ANA-09` | Ticket 11 Deferred successor (named rich-analysis adapters); census `CAP-04-ANA-09` maps to `ANA-07` |

## Census completeness index

Rebuilt from the computed Capability set (135) plus the four named non-Capability Frozen IDs (139 Frozen IDs). The index lists each Frozen ID once. Domain 07 headings and Domain 04 Next-only attachments stay in the index as **not counted** so reference 139 remains comparable without being treated as the Capability total.

## Authority (not edited)

- `docs/PARITY_MATRIX.md` status, evidence, remaining gap, acceptance, and `Depends on` were not changed.
- `docs/MIGRATION_PLAN.md` phase membership, Promotion Gates, Delivery Order, and Phase Gates were not changed.
- No new Parity IDs.

## Destination writes

- `docs/JAVA_CAPABILITY_INVENTORY.md`: corpus counts, Cross-domain routes named pairs, runtime-check class words, absorbed `ENG-02` labels, rebuilt census completeness index.
- This research file.
