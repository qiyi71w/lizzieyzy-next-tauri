# 16 — Final Traceability and Evidence Completeness Audit

## Verdict

**Destination not reached.**

The three destination documents form a mutually indexed, authority-split corpus whose **counts match Ticket 15**: 139 census Capabilities, 95 Parity Items, 19 Deferred items, and 13 owner-route/conflict records. Research 01–07 headings match the inventory 1:1. Every original Accepted item keeps its ID, status, capability text, remaining-gap, and acceptance; no new item was marked Accepted during the audit.

The closed-trace question still fails. Thirteen census remainders have a final owner-route or conflict but **no supported Parity Item ID and no explicit exclusion** from Tickets 08–14. Ticket 15 carried those records into this audit without inventing IDs. They remain unscheduled. That is unresolved in-scope fog, not a documentation typo.

A second, smaller failure is **start-dependency disagreement** between `PARITY_MATRIX.md` `Depends on` and `MIGRATION_PLAN.md` after Ticket 15 already chose an order. That does not invent product behavior, but it does make start gates non-authoritative.

No destination document was edited. Each failed check maps to a proposed Wayfinder ticket below.

## Sources

| Tree / artifact | Path | Commit / role |
| --- | --- | --- |
| Java Migration Baseline v1 | `/home/dev/dev/weiqi/worktrees/lizzieyzy-next/java-baseline-v1` | Detached HEAD `7b4027531c2b26062d0bfc27a040cc550cfbea4d` (verified this audit) |
| Next inventory baseline | this worktree | `18c6d189b8b01069975c4c40ead63a010249cb8c` |
| Current worktree HEAD | this worktree | `71654d97feb0f41a5f128916347053b99a3ae68d` (destination docs evolved after the inventory baseline; that is expected) |
| Language | `CONTEXT.md` | Capability, Entry Point, Disposition, Parity Item, Successor Item, phase/evidence terms |
| Java behavior reference | `docs/JAVA_BASELINE.md` | Fixes v1 at `7b402753…` |
| Architecture | `docs/ARCHITECTURE_NEXT.md` | Module/DTO/command boundaries; not item status |
| Inventory | `docs/JAVA_CAPABILITY_INVENTORY.md` | Census, Entry Points, defaults, persistence, failure, mapping |
| Parity Matrix | `docs/PARITY_MATRIX.md` | Status, evidence, remaining gap, acceptance, item `Depends on` |
| Migration Plan | `docs/MIGRATION_PLAN.md` | Phases, gates, exits, Deferred queue, next batch |
| Map | `.scratch/migration-coverage-audit/map.md` | Destination, authority split, Decisions so far, R0–R2 freeze |
| Ticket 15 | `.scratch/migration-coverage-audit/issues/15-rebuild-r3-plus-roadmap.md` | Resolved roadmap + the 13 records |
| Ticket 16 | `.scratch/migration-coverage-audit/issues/16-audit-traceability-completeness.md` | This question |
| Census | `.scratch/migration-coverage-audit/research/01-…07-….md` | Named Capabilities and Entry Points |
| Dispositions | `.scratch/migration-coverage-audit/issues/08-…14-….md` | Equivalent / redesign / defer / abandon / Swing-only |
| Frozen pre-audit matrix | `git show 18c6d189:docs/PARITY_MATRIX.md` | 41 items; 16 Accepted |

Java source was **not** re-censused. Completeness is the trace from research 01–07 through Tickets 08–15 into the three destination documents. The frozen Java worktree identity was checked so cited baseline commits are not a different tree.

## Method

1. Reconstruct every Frozen ID, Entry Point cell, mapping, Parity ID, status, evidence/gap/acceptance, phase, and `Depends on` from the destination files.
2. Diff research 01–07 capability headings against the inventory census index and domain tables.
3. Diff Ticket 15’s 139 / 95 / 19 / 13 claims and R3–R11 topology against the destination files and the inventory Ticket 16 ledger.
4. Diff the 16 Accepted rows at Next baseline `18c6d189` against the current matrix (ID, capability, status, evidence, remaining gap, acceptance, phase).
5. Check authority split: Inventory owns baseline contracts; Matrix owns Next status/evidence; Plan owns order/exits. Do not demand duplicate prose.
6. Classify each Frozen ID as `mapped` (named supported ID or explicit split), `exclusion`, or `gap` (Ticket 16 ledger language).
7. Treat explicit splits and Ticket 11 “Absorbed redesign” of Java process-preferences as closed when Tickets 08–14 named the surviving item or stated the Java control is not retained. Treat owner-routed remainders with no ID as **open**.
8. Report failures as proposed tickets. Do not invent Parity IDs or repair destination documents.

## Quantitative reconciliation

Ticket 15 claimed 139 Capabilities, 95 Parity Items, 19 Deferred items, and 13 owner-route/conflict records. **Those counts are correct.** They are claims that this audit verified, not assumptions.

| Claim | Ticket 15 / prior roadmap | Evidence now | Verdict |
| --- | --- | --- | --- |
| Inventory Capabilities | 139 unique rows, domains 01–07 | Inventory tables: 13+35+35+25+8+10+13 = **139**; census index lists the same 139 once; no duplicate Frozen IDs | Pass |
| Research 01–07 headings | implied by 139 | 13+35+35+25+8+10+13 headings; set equality with inventory Frozen IDs (SHELL headings differ only by title text) | Pass |
| Parity Items | 95 | Matrix item rows: **95** unique IDs. Status: 16 Accepted, 19 Partial, 41 Missing, 19 Deferred | Pass |
| Deferred queue | 19, unnumbered | Matrix Deferred table and Plan Deferred table both list the same 19 IDs | Pass |
| Owner-route / conflict records | 13, no stable Parity ID | Inventory “Parity decision gaps (Ticket 16)” table: **13** rows; Matrix “Ticket 16 Disposition-To-Item Gap” 13 rows; Plan “Traceability Open Gaps” 13 rows; 13 inventory domain-table mappings contain `Parity decision gap` | Pass (count); **Fail (closed route)** |
| Frozen pre-audit matrix | 41 items | `18c6d189:docs/PARITY_MATRIX.md` has 41 IDs. Current matrix keeps all 41 and adds 54 successors = 95 | Pass |
| Original Accepted set | 16 | Same 16 IDs still Accepted; no extra Accepted | Pass |
| Java HEAD | `7b4027531c2b26062d0bfc27a040cc550cfbea4d` | `git -C java-baseline-v1 rev-parse HEAD` equals that commit | Pass |

### Status roll-up (current matrix)

| Status | Count | IDs |
| --- | --- | --- |
| Accepted | 16 | `BASE-01` `BASE-02` `SGF-01`–`SGF-06` `RULE-01` `UI-01` `UI-03` `UI-04` `UI-05` `ENG-01` `ANA-05` `READ-03` |
| Partial | 19 | `UI-02` `ENG-05` `ANA-01`–`ANA-04` `PREF-01` `SGF-07` `SGF-10` `REVIEW-01` `REVIEW-07` `APPEAR-01` `PROV-01` `PROV-02` `READ-01` `READ-02` `REL-01` `REL-05` `REL-10` |
| Missing | 41 | R3–R11 remainder (`ENG-02`–`ENG-04` `ENG-06` `ENG-07` `APP-01`–`APP-05` `SGF-08` `SGF-09` `SGF-11`–`SGF-14` `REVIEW-02` `REVIEW-03` `REVIEW-08` `LAYOUT-01`–`LAYOUT-04` `WINDOW-01` `GUIDE-01` `ENG-09` `ENG-10` `GAME-01`–`GAME-05` `PROV-03` `PROV-04` `REL-02`–`REL-04` `REL-06`–`REL-09`) |
| Deferred | 19 | `I18N-01` `SGF-15` `SGF-16` `REVIEW-04`–`REVIEW-06` `EXPORT-01` `EXPORT-02` `ENG-08` `ANA-06`–`ANA-09` `GAME-06`–`GAME-09` `PROV-05` `PUB-01` |

### Inventory mapping kinds (139)

| Kind | Count | Meaning |
| --- | --- | --- |
| `mapped` | 109 | Mapping names one or more supported Parity IDs, or an explicit split/merge into those IDs |
| `exclusion` | 17 | Mapping starts with `Exclusion —` (Abandoned or Swing-only) |
| `gap` | 13 | Mapping contains `Parity decision gap (Ticket 16)` |

109 + 17 + 13 = 139. Several `gap` rows are **partially** mapped (public Yike, readboard sync, `GAME-09` board occupancy, update OS proxy, `ENG-01`/`EXPORT-*`/`UI-05` slices) with an **un-IDed remainder**. Those remainders are the 13 records, not extra Capabilities.

### Matrix IDs with no census mapping

| ID | Why this is not an orphan census row |
| --- | --- |
| `BASE-01` `BASE-02` | R0 meta-inventory; not Java user-reachable Capabilities. Inventory Authority table points at them. |
| `REL-01` | Maintainer preflight/dry-run. Inventory “Maintainer-only”: not one of the 13 `REL-C*` Capabilities (`docs/JAVA_CAPABILITY_INVENTORY.md` §Maintainer-only). |
| `ANA-09` | Ticket 11 Deferred successor “Named Rich-analysis Engine Adapters”, not a research-04 heading. Census `CAP-04-ANA-09` maps to `ANA-07`. |

## Reconstructed indexes

### Authority split (do not demand duplicate prose)

| Concern | Owner | Where it lives |
| --- | --- | --- |
| Entry Points, Java defaults, persistence, failure/recovery, frozen source, `需定向运行核验` | Inventory | Domain tables + “Defaults, persistence, and failure ownership” |
| Disposition (08–14) and mapping to IDs / exclusion / gap | Inventory | Mapping column; Abandoned / Swing-only / unregistered tables |
| Status, repository evidence, live/environment evidence, remaining gap, acceptance, item `Depends on` | Parity Matrix | Item rows from R3 onward include `Depends on`; R0–R2 keep historical columns |
| Phase membership, start gates, exits, Deferred admission, next executable batch | Migration Plan | Roadmap + Deferred table + Slice R3-A |
| Module/DTO/command boundaries | `ARCHITECTURE_NEXT.md` | Not a substitute for item status |

Shared cross-cutting contracts (`PREF-01` atomic write, `SGF-07` parse-before-replace, `APP-03` shutdown, `APP-04` review-only recovery, provider timeouts) are stated once in Inventory’s ownership table and referenced by Matrix/Plan. That is the intended split, not missing item prose.

### Census Capabilities and Entry Points

Full cells are in `docs/JAVA_CAPABILITY_INVENTORY.md` at the cited lines. Route = `mapped` / `exclusion` / `gap`.

| Frozen ID | Inv. line | Route | Mapped / surviving Parity IDs | Entry Points | Default / persistence | Failure / recovery |
| --- | --- | --- | --- | --- | --- | --- |
| `SHELL-01` | 73 | mapped | `UI-01`, `UI-04`, `REL-04` | OS executable / `java … featurecat.lizzie.Lizzie`; installer launch (07) | `appName=LizzieYzy Next`; `lizzieVersion=2.5.3`; `nextVersion` from `-Dlizzie.next.version` / `LIZZ… | Hostname lookup timeout 500ms never treats unknown host as machine change. Logging bootst… |
| `SHELL-02` | 74 | mapped | `ENG-06`, `ENG-02` | Startup only. Flags that *set* autoload belong to 02/04. | Missing keys: all autoload flags false (manual). New bundled KataGo profile writes `autoload-defaul… | `shouldOfferEngineRepair` when no configured engine and not autoload-empty. |
| `SHELL-03` | 75 | mapped | `APP-01`, `SGF-07` | `args[0]` when it is not `"read"` | N/A if no argv. Recent files via 03. | `loadFile` errors owned by 03. |
| `SHELL-04` | 76 | exclusion | `UI-04` | argv exactly `read` | `readMode=false`. Mode is not persisted. | N/A |
| `SHELL-05` | 77 | mapped | `APP-04` | Startup if `config.autoResume` | `resume-previous-game` default **false**. Files `save/autoGame1.sgf` else `autoGame2.sgf`. | Missing file → skip. |
| `SHELL-06` | 78 | mapped | `ENG-06`, `REL-05`, `WINDOW-01` | Startup when `firstTimeLoad` or resolved hostname differs | New profile / `firstTimeLoad`. Persist `host-name`, `first-time-load`. | Save IOException → `startupProfileSaveFailed` and repair chip. |
| `SHELL-07` | 79 | mapped | `ENG-07` | `engineStartupStatusButton` click when snapshot `isActionable` | Hidden until actionable. Not persisted. | Click opens KataGo autosetup; further repair is 04. |
| `SHELL-08` | 80 | mapped | `APP-03` | File › Exit; window close (`WINDOW_CLOSING`) | `auto-save-exit` default **true**. Persist file + config; optional autosave SGF. | Persist/save errors show modal with path; resource failures print stack; process still ex… |
| `SHELL-09` | 81 | exclusion | `APP-03` | File › Force Exit | N/A. No persist. | Unsaved work lost. |
| `SHELL-10` | 82 | exclusion | `UI-01` | Automatic at frame construct; `-Dlizzie.menu.presentation` | Default `auto` (Wayland Linux → native; else custom). Process property only. | N/A |
| `SHELL-11` | 83 | mapped | `APP-05`, `UI-05` | `mainPanel` `Input`; independent-board / subboard inputs | Main board focus after start. Not persisted. | Keys while comment/GTP focused 需定向运行核验. |
| `SHELL-12` | 84 | mapped | `APP-02`, `APP-01`, `SGF-07`, `ANA-07` | OS drag-drop onto `mainPanel` | N/A | Exception print, return false. |
| `SHELL-13` | 85 | mapped | `APP-05` | `VK_X` press/release in `Input` | Off. Not persisted. | N/A |
| `SET-RESET-WINDOW-POS` | 97 | mapped | `WINDOW-01` | View `Menu.deletePersistFile` | Destructive. Deletes `persist`. Does not rewrite theme/language/engine in `config.txt`. Also calls … | After this action, shutdown persist write is skipped until restart. Dialog always shown w… |
| `SET-RESTORE-PANEL-SIZES` | 98 | mapped | `LAYOUT-03` | View → 面板 `restoreDefaultPanelSizes` | N/A (restore action). Distinct from window reset. | Body not fully extracted. Whether persist is immediate 需定向运行核验. |
| `SET-RESET-HINTS` | 99 | mapped | `GUIDE-01` | Not a menu. Callers: `SET-RESET-WINDOW-POS`; `Lizzie.main` on firstTimeLoad / hostname change | Four flags load-default true. Does **not** reset `showNewBoardHint`. Persist ui keys; file save fol… | No dedicated save; relies on later SC-01/SC-03. |
| `SET-FIRST-LAUNCH` | 100 | mapped | `PREF-01`, `WINDOW-01`, `LAYOUT-04`, `APPEAR-01` | Startup when `firstTimeLoad` or hostname differs | `first-time-load` load fallback true. Persist `first-time-load`, `host-name`. Host change wipes per… | Save IOException → repair chip. Hostname lookup 500ms never fakes a machine change. |
| `SET-FIRST-USE` | 101 | mapped | `PREF-01` | Settings → `Menu.initSettings` → `openFirstUseSettings(false)` | Load-defaults button fills the dialog only. Confirm writes listed ui/leelaz keys and `config.save()… | Confirm parse errors → hint, no write. Save IOException printed, not modal. |
| `SET-PERSIST-CONFIG` | 102 | mapped | `PREF-01` | Any ui/leelaz write; Settings/FirstUse Confirm; many View toggles; shutdown `config.save()` | `createDefaultConfig` plus `opt*` plus first-launch. Atomic temp write. | Missing → write defaults + `newProfile`. Unreadable → backup + rebuild. Shutdown save fai… |
| `SET-PERSIST-WINDOW` | 103 | mapped | `WINDOW-01`, `LAYOUT-02` | Implicit on close via `config.persist()`. Reset via `SET-RESET-WINDOW-POS`. | Persist default: empty positions, `window-maximized=false`. Field `startMaximized=true` is **not** … | Load fail → in-memory empty persist. After window reset, writes skipped until restart. |
| `SET-LANG` | 104 | mapped | `I18N-01` | Settings language submenu; FirstUse language combo | New profile: OS locale. Load fallback SYSTEM (`use-language=0`). Persist `ui.use-language`. | SC-01 |
| `SET-LOOKS` | 105 | exclusion | — | Settings 界面外观; FirstUse radios | Load `optBoolean(..., !OS.isWindows())`. Persist `ui.use-java-looks`. Restart required. | SC-01. Settings-menu writes rely on later save. |
| `SET-FRAME-FONT` | 106 | exclusion | — | Settings 界面字体大小; other → `SetFrameFontSize` | Load fallback 12. Persist `ui.frame-font-size`. | SC-01 |
| `SET-SOUND` | 107 | mapped | `REVIEW-08`, `PROV-03`, `READ-02`, `PREF-01` | Settings `playSound`, `notPlaySoundInSync` | Both load-fallback true. `play-sound` absent from default JSON. Persist `ui.play-sound`, `ui.not-pl… | SC-01 |
| `SET-CONTRIBUTE-MENU-VIS` | 108 | mapped | `GAME-09` | Settings `showContribute`; menu `setVisible` | Load `show-Contribute` default true. Persist `ui.show-Contribute`. | SC-01 |
| `SET-THEME-BOARD-STYLE` | 109 | mapped | `APPEAR-01` | View → 外观主题 → 棋盘风格 | JSON / load default Japanese. Persist `ui.board-style`. | SC-01 |
| `SET-THEME-APPLE-CLASSIC-CUSTOM` | 110 | exclusion | — | View Apple / classic color / custom board cluster | JSON `is-apple-style=false`, `theme=default`. Persist those plus `use-morandi-colors`. | SC-01 |
| `SET-THEME-DIALOG` | 111 | exclusion | `APPEAR-01`, `PREF-01` | Settings → 主题 → `openConfigDialog2(1)` | Theme `default`. Shadows JSON 85 vs field 75. Persist theme object + ui keys. | SC-01 |
| `SET-CONFIG-DIALOG-DISPLAY` | 112 | mapped | `PREF-01` | Settings 综合设置 (`Shift+X`) → `openConfigDialog2(0)`; Help About opens tab 2 | N/A as a dialog. Display checkboxes live-toggle View flags; other tabs OK-apply. | SC-01 |
| `SET-NETWORK-PROXY` | 113 | gap | `REL-03` | ConfigDialog2 Advanced; keys owned by `NetworkProxy` | Default mode `direct`; host `127.0.0.1` port `7897`. Persist mode/host/port. | SC-01 |
| `SET-BOARD-SIZE` | 114 | mapped | `SGF-10`, `GAME-04` | ConfigDialog2 Display; `SetBoardSize`; `saveOtherBoardSize` | Default 19. Other size 21×21. Persist `board-size`, `other-size-width/height`. Commented `<2` clamp… | SC-01 |
| `SET-LAYOUT-MODE` | 115 | exclusion | `LAYOUT-01`, `LAYOUT-02`, `LAYOUT-04` | View 布局模式 Alt+1..9 | JSON ExtraMode Normal. Unknown → Normal. Persist `extra-mode`, `is-classic-mode`, `custom-layout-1/… | SC-01 |
| `SET-LAYOUT-PANELS` | 116 | mapped | `LAYOUT-04`, `UI-01` | View 面板; ConfigDialog2 Display live toggles | Default JSON: status/comment/variation/subboard/captured/winrate on. Persist show-* keys; independe… | SC-01 + SC-02 |
| `SET-LAYOUT-TOOLBAR` | 117 | exclusion | `UI-05` | View 工具栏; `ToolbarPositionConfig` | `showTopToolBar=true`, `autoWrapToolBar=true`. Persist flags; height in persist. | SC-01 + SC-02 |
| `SET-BOARD-POS` | 118 | mapped | `LAYOUT-01`, `LAYOUT-02`, `LAYOUT-03` | View 主棋盘位置 `[` `]` | Field default 4. Persist typo `board-postion-propotion` vs load `board-postion-proportion`. | SC-01 + SC-02 |
| `SET-COORDS` | 119 | mapped | `REVIEW-07`, `PREF-01` | View 坐标(C); ConfigDialog2 Display | JSON / load default true. Persist `ui.show-coordinates`. Next session-only is not this baseline. | SC-01 |
| `SET-MOVE-NUMBERS` | 120 | mapped | `REVIEW-07` | View 手数(M) modes; ConfigDialog2 radios | JSON `show-move-number=false`, `only-last-move-number=1`. Persist those plus branch/from-one flags. | SC-01 |
| `SET-SUGGESTION-INFO` | 121 | mapped | `ANA-04`, `UI-03` | View 选点信息; FirstUse WR/visits/score | JSON `show-best-moves=true`, `limit-max-suggestion=10`. Persist ui + leelaz keys. Search/limit visi… | SC-01 |
| `SET-NEXT-MOVE` | 122 | gap | `ANA-04`, `UI-01`, `PREF-01`, `SGF-03` | View 下一手 including `minPlayoutsForNextMove` | JSON `show-next-moves=true`. Load blunder true; min playouts 30. Persist those ui keys. | SC-01 |
| `SET-WINRATE-GRAPH` | 123 | gap | `UI-01`, `ANA-04`, `PREF-01` | View 胜率图设置 | Blunder bar JSON false plus one-time hide migration. Lines default true. Persist ui flags; persist … | SC-01 + SC-02 |
| `SET-SUBBOARD` | 124 | gap | `UI-01`, `ANA-04` | View 小棋盘设置; FirstUse mouse-over-subboard radios | JSON `show-subboard=true`. Heat flags load false. Persist those ui keys. Heatmap **data** is 04; mo… | SC-01 |
| `SET-MAIN-PANEL` | 125 | gap | `UI-01` | View 主界面设置 cluster; large-sub / large-WR toggles | JSON large flags false. `append-winrate-to-comment=true`. Persist ui keys. Hint flags also `SET-HIN… | SC-01 |
| `SET-KATA-DISPLAY` | 126 | mapped | `ANA-04`, `UI-03` | View KataGo settings; FirstUse score+komi radios | Estimate off; on-main/on-sub true. Persist ui keys. Computation is 04; overlay prefs were 02 census. | SC-01 |
| `SET-HINT-NEWBOARD` | 127 | mapped | `SGF-07`, `SGF-10`, `GUIDE-01` | 主界面设置 plus dialog `noNoticeAgain` | Load default true. Persist `ui.show-new-board-hint`. **Not** reset by `resetAllHints`. | SC-01 |
| `SET-HINT-REPLACE` | 128 | mapped | `SGF-07` | File replace dialog `noNoticeAgain`; re-enabled by `SET-RESET-HINTS` | Default true. Persist `ui.show-replace-file-hint`. Reset **yes**. | SC-01 |
| `SET-HINT-COMMENT-CTRL` | 129 | exclusion | — | Comment-control UI; reset via `SET-RESET-HINTS` | Default true. Persist `ui.allow-close-comment-control-hint`. Reset **yes**. | SC-01 |
| `SET-HINT-AUTOANALYZE` | 130 | gap | `GUIDE-01` | Auto-analyze exit path (04); reset via `SET-RESET-HINTS` | Default true. Persist `ui.exit-auto-analyze-tip`. Related `exit-auto-analyze-by-pause` is a separat… | SC-01 |
| `SET-CLEAR-PERSONAL-HISTORY` | 131 | mapped | `SGF-09`, `PROV-02`, `PROV-04`, `PROV-01`, `PROV-03`, `ANA-07` | Help → `clearAllPersonalData` with confirm | N/A. Removes `fox-recent-searches`, `recent-files`, `batch-analysis-history`, `share-history` then … | Save IOException printed. Cancel → no-op. |
| `SGF-03-RT` | 143 | mapped | `SGF-01` | File Open / Save / Save As / clipboard copy serialize / Next parse-textarea / sample load | FF[4]-family SGF; board size from `SZ` (parser default 19); `loadSgfLast` walks to last move | Unreadable/empty: `SgfObservation` + `false`; Next malformed: `MalformedSgf`, holder unch… |
| `SGF-03-DTO` | 144 | mapped | `SGF-02` | N/A (wire contract consumed by current-game UI) | Root `[]`; default selected path first-child mainline to leaf | Invalid path → `InvalidNodePath`; no document → `NoCurrentGame` |
| `SGF-03-NAV` | 145 | mapped | `SGF-03` | Java: arrows / Page / Home / End; Edit jump items; variation tree; wheel. Next: arrows / Home / End / Page*; Edit 跳转到最前 | Session cursor. Java `loadSgfLast` persists whether to land on last move | Illegal path rejected; Next select does not bump generation |
| `SGF-03-EDIT` | 146 | mapped | `SGF-04`, `UI-05` | Game 停一手(P) `board.pass()`; `Input` VK_P (HumanSL → 05); board left-click; Shift+Delete / Edit 删除分支 | Color = selected node’s player-to-play. Dirty until Save | Occupied/suicide/simple-ko reject without mutation. Root cannot be removed |
| `SGF-03-CMT` | 147 | mapped | `SGF-05`, `SGF-03` | Java comment pane `setCommentEditable`; Next AnalysisPanel personal textarea | Empty comment. Persist `C` in serialized SGF | Next no-op if unchanged; empty trim removes `C` |
| `SGF-03-SAVE` | 148 | mapped | `SGF-06` | File Save / Save As; Ctrl+S / S (and Java extra save shortcuts that belong to ADJ-SAVE-MORE) | Timestamp-ish filename from GameInfo or `yyyyMMddHHmmss`; last-folder in `filesystem` | Java: `saveFileFailed` toast. Next: directory-as-file leaves path/dirty; cancel/denied wr… |
| `SGF-03-RULE` | 149 | mapped | `RULE-01` | Same as `SGF-03-EDIT` | Simple ko; suicide forbidden in Next `go-core` | Atomic reject |
| `SGF-03-CHROME` | 150 | mapped | `UI-01`, `LAYOUT-01` | Main window after open | Next fixed 228px/260px rails (`UI-01`). Layout visibility is 02 | N/A |
| `SGF-03-BOARD-INTENT` | 151 | mapped | `UI-02` | Board click; board-focused arrows+Enter/Space; outside-board arrows navigate | N/A | Occupied: Next status text, position unchanged |
| `SGF-03-HOVER` | 152 | mapped | `UI-03`, `SGF-03` | Pointer over candidate; leave/click/scope change cancels | Java default **200 ms** plus optional delay dialog. Next accepted contract is **120 ms**. Delay set… | Scope mismatch refuses publish |
| `SGF-03-NOENGINE` | 153 | mapped | `UI-04` | All File/SGF/board/review controls; Next chip `未加载引擎` | Next `engineLabel` default `未加载引擎` | Engine-only actions unavailable/explanatory |
| `SGF-03-ACTIONS` | 154 | mapped | `UI-05` | `App.tsx` onKey; `AppChrome` File/View/Game/Edit/toolbar. Java `Input.keyPressed` is a larger map | Coordinate/move-number toggles are session in current Next (persist via `REVIEW-07`) | `shouldIgnoreApplicationShortcut` skips when typing in fields |
| `SGF-03-ADJ-OPEN` | 160 | mapped | `SGF-07`, `APP-01`, `APP-02` | File 打开, `O`, toolbar; `loadFile` after chooser | last-folder from persisted filesystem | IO / parse false → restore sound flag + later open-failed message. Java has **no** dirty-… |
| `SGF-03-ADJ-GIB` | 161 | mapped | `SGF-08`, `SGF-07` | Same Open chooser (`*.gib`); `loadFile` extension branch | Names “Player 1/2”; komi 7.5 if not parsed. Save as SGF | Missing/unreadable/empty → false; must not clear a seeded board |
| `SGF-03-ADJ-RECENT` | 162 | mapped | `SGF-09`, `SGF-07`, `PREF-01` | File → 最近打开 (`updateRecentFileMenu`) | List length / persist key 需定向运行核验 | Missing file 需定向运行核验 |
| `SGF-03-ADJ-CLIP` | 163 | mapped | `UI-05`, `SGF-07` | File copy/paste; Ctrl+C / Ctrl+V | N/A | Copy exception logged; paste ignore or confirm-cancel leaves game. Java `isSGF` gate + co… |
| `SGF-03-ADJ-SAVE-MORE` | 164 | gap | `EXPORT-01`, `EXPORT-02` | File → 更多保存; Ctrl+Shift+S, Ctrl+Alt+S, Alt+S, Shift+S, Shift+Alt+S | Same timestamp naming as Save As. Persist `last-image-folder` | Write/format unsupported dialogs |
| `SGF-03-ADJ-TEMP` | 165 | mapped | `APP-04` | File → 存档与读档 → `showTempGamePanel`; panel checkboxes. File menu Resume item is **commented out**; startup `resumeFile` remains | Checkboxes from `autoSaveOnExit` / `autoResume`. Persist those ui keys; slot files under `save/` | Missing bmp/sgf: stack trace / skip. Startup resume wiring 需定向运行核验 |
| `SGF-03-ADJ-NEW` | 166 | mapped | `SGF-10`, `SGF-07` | File 新建棋盘; Edit 清空棋盘 Ctrl+Home; `N` (Java also starts genmove — 05). Next File 新建 / `N` / Ctrl+Home / 清空棋盘 | Next empty `(;GM[1]FF[4]SZ[19]KM[7.5]PB[黑]PW[白])`. Java File-new vs clear-in-place 需定向运行核验 | Confirm cancel leaves game |
| `SGF-03-ADJ-KOMI` | 167 | mapped | `SGF-07`, `SGF-08`, `SGF-10` | File checkbox 自动加载棋谱中的贴目 | Config default 需定向运行核验. Persist `uiConfig.read-komi`. Even when off, setup/handicap games still app… | N/A |
| `SGF-03-ADJ-SETUP` | 168 | mapped | `SGF-11`, `SGF-01`, `SGF-04` | Game → 起始局面设置; board clicks in setup | Tools black/white/erase. Persist setup stones in tree. AB/AW/AE/PL **parse** already in `SGF-01` | Cancel/rejection 需定向运行核验 |
| `SGF-03-ADJ-INSERT` | 169 | mapped | `REVIEW-01`, `SGF-15`, `REVIEW-04` | Edit menu; right-click add black/white; toolbar icons; `allowDrag` / `allowDoubleClick` / `enableClickReview` | Those booleans in uiConfig | Drag onto occupied: abort |
| `SGF-03-ADJ-DELETE-MOVE` | 170 | mapped | `SGF-12`, `SGF-04`, `UI-05` | Edit 删除一手; Delete/Backspace; right-click `deleteone`; undo/redo delete | N/A. In-tree | Empty-mainline delete 需定向运行核验 |
| `SGF-03-ADJ-MAIN` | 171 | mapped | `SGF-12`, `SGF-03` | Edit 设为主分支(L) / 返回主分支(B); `L` / `B`; toolbar | N/A. Tree child order | Already-main is no-op |
| `SGF-03-ADJ-XFORM` | 172 | mapped | `SGF-16` | Edit exchange/spin/mirror; Ctrl+Shift+Alt+Right, Ctrl+Alt+arrows | N/A. Mutates tree | 需定向运行核验 |
| `SGF-03-ADJ-META` | 173 | mapped | `SGF-13`, `SGF-10`, `SGF-01` | Edit 编辑棋局信息(I) / 设置棋盘大小(Ctrl+I) | Current GameInfo. Handicap field read-only. Persist root SGF properties on save | Whole-game analysis blocks edit |
| `SGF-03-ADJ-MARKUP` | 174 | mapped | `SGF-14`, `SGF-01` | Markup tools; `tryToMarkup` / `tryToRemoveMarkup` | Markup off. Persist node properties; `SGF-01` already preserves unknown/list props in frozen set | Click off-board: false |
| `SGF-03-ADJ-AUTOPLAY` | 175 | gap | `UI-05`, `REVIEW-05`, `ANA-04` | View 自动播放(Ctrl+A) → `AutoPlay` dialog. Next Ctrl+A toggles 800 ms | Toolbar main/sub values; persist branch-preview and engine-continuation defaults; JSON `replay-bran… | Next main-board autoplay stops at a leaf. Java candidate-variation replay stops when disa… |
| `SGF-03-ADJ-NEXT-HINT` | 176 | gap | `ANA-04`, `UI-01`, `PREF-01` | View 下一手; `J` toggles | Defaults 需定向运行核验. Persist `showNextMoves` / `showNextMoveBlunder` | N/A |
| `SGF-03-ADJ-TREE-CLICK` | 177 | mapped | `REVIEW-01` | `VariationTree` clickPoint; blunder table mouseClicked | N/A | N/A |
| `SGF-03-ADJ-TRYPLAY / SCORE / LADDER` | 178 | mapped | `REVIEW-02`, `REVIEW-03`, `REVIEW-06`, `SGF-03` | Try-play: `V` `tryPlay(false)`. Score: Ctrl+Q; board left-click in score; rule toggle and Confirm Result. Ladder: Game 继续征子 | Score defaults to area; selected rule persists `use-territory-in-score`. Ladder refuses below `MINI… | Score preview is transient. Ladder failure must not partially mutate. Try-play must not m… |
| `SGF-03-ADJ-URL` | 179 | mapped | `PROV-01`, `PROV-03`, `PROV-05`, `SGF-07` | File 打开在线链接(Q); `Q`; `OnlineDialog` | Refresh / live flags owned by 06 | Fetch/parse failure owned by 06; current-game install is 03 |
| `SGF-03-ADJ-PROVIDER` | 180 | mapped | `SGF-07` | Owned by 06; Java loaders end in `loadFile` / `loadSgfString*`; Next `handleProviderImport` | N/A as a 03 setting | Dirty confirm / owner rejection preserves current game |
| `SGF-03-ADJ-HOVER-DELAY` | 181 | mapped | `UI-03` | `SetDelayShowCandidates`; `delay-show-candidates`, `delay-candidates-seconds` | Java `DEFAULT_DELAY_MS = 200` | N/A |
| `SGF-03-ADJ-COMMENT-LAYERS` | 182 | mapped | `GAME-05`, `SGF-05` | Comment pane display path; teacher codec; engine-game generated comments | `append-winrate-to-comment` default true (02/04 overlap) | N/A for the personal field |
| `CAP-04-ENG-01` | 192 | gap | `ENG-01`, `ENG-09`, `ENG-06`, `ENG-08`, `GAME-04` | `MoreEngines` table add/save/cancel/delete/move; command scan; remote SSH fields; GTP config; encrypt command | New row: empty command, localized “new engine”, `preload=false`, 19×19, komi 7.5. Persist `leelaz.e… | Save/scan/GTP load failures show dialogs. Empty command on dirty check offers delete. Del… |
| `CAP-04-ENG-02` | 193 | mapped | `ENG-06` | `LoadEngine` radios; `MoreEngines` same radios; first-run bundled preference | Existing `config.txt`: manual (all flags false). New complete package profile: autoload default. Pe… | Broken default + autoload-default may retarget bundled engine. Profile save failure → `En… |
| `CAP-04-ENG-03` | 194 | exclusion | — | Implicit on shutdown when autoload-last is set | Missing → `-1`. Persist `ui.last-engine` | Persist failure shows modal; does not block exit after the dialog |
| `CAP-04-ENG-04` | 195 | mapped | `UI-04`, `ENG-02` | Autoload-empty; `LoadEngine` 不加载引擎; failed-start fallback `start(-1,false)`; `read` CLI (Abandoned as `SHELL-04`) | N/A. Autoload-empty flag only | Failed `EngineManager` construct → retry empty manager |
| `CAP-04-ENG-05` | 196 | mapped | `ENG-02`, `ENG-10` | Autoload paths; `LoadEngine` OK/double-click; menu engine items; `ChooseMoreEngine`; `EngineManager(config,index,loadDefault)` | Ponder starts unless `notStartPondering`. `playponder` field default true | Primary failures stay on repair status. Stale generation cannot publish checking/ready. R… |
| `CAP-04-ENG-06` | 197 | mapped | `ENG-02` | `Menu.shutdownCurrentEngine`, `restartCurrentEngine`, `shutdownOtherEngine`, `shutdownAllEngine`; secondary equivalents; pondering button p… | N/A | Failed restart uses failed-switch/quarantine paths |
| `CAP-04-ENG-07` | 198 | mapped | `ENG-03`, `ENG-04` | `Menu.engine[]` / `ChooseMoreEngine`; `EngineManager.switchEngine` | N/A | FAILED snapshot; quarantine; if captured A is gone, primary becomes empty rather than sil… |
| `CAP-04-ENG-08` | 199 | mapped | `ENG-07` | Process exit / SSH disconnect / OpenCL native exit | `auto-check-engine-alive` default true. Persist that ui key | START_FAILED / NEEDS_REPAIR; click-to-repair. Whether this auto-restarts foreground GTP 需… |
| `CAP-04-ENG-09` | 200 | exclusion | — | `MoreEngines` preload checkbox | Default false, including bundled slot. Persist `engine-settings-list[].preload` | Secondary bundled startup must not publish primary CHECKING |
| `CAP-04-ANA-01` | 201 | mapped | — | `AnalysisSettings.chkReuseCurrentEngine`; used by whole-game lease handoff | Default **false**. Persist `ui.analysis-reuse-current-engine` | Failed save leaves runtime flags and pending flash request unchanged. Exclusive lease blo… |
| `CAP-04-ANA-02` | 202 | mapped | — | `AnalysisSettings.chkPreLoad`; `Lizzie.start` after EngineManager | Default **false**. Persist `ui.analysis-engine-preload` | Same save-failure atomicity as other analysis settings |
| `CAP-04-ANA-03` | 203 | mapped | — | `AnalysisSettings.chkAutoExit` | Default **true**. Persist `ui.analysis-auto-quit` | Save failure does not flip the flag |
| `CAP-04-PREF-LIZZIE-CACHE` | 204 | exclusion | `ANA-05` | Loaded from leelaz config; dedicated menu 需定向运行核验 | Default **true**. Persist `leelaz.enable-lizzie-cache`. **Not** Next SQLite | Incomplete payloads are not terminal success |
| `CAP-04-ANA-04` | 205 | mapped | `ANA-06` | `Menu.pondering`; toolbar `analyse`; after engine ready; `AnalysisTable.stopGo`; `playponder` | Auto-ponder on ready: on. Toolbar `analyse` default true. Persist `ui.analyse`, time/playout limits… | Pause/restore via `ForegroundAnalysisPause` during switch/game start |
| `CAP-04-ANA-05` | 206 | mapped | `ANA-01`, `ENG-05` | Next: EngineSetupPanel 分析此手. Java closest: ponder/flash-part, not a named one-shot JSONL job | Next max_visits UI default 800; Java `analysis-max-visits` default 1. Persist profile `max_visits` … | Next: command error string; one-shot has no job-id cancel in the ENG-05 sense at the inve… |
| `CAP-04-ANA-06` | 207 | exclusion | `ANA-01`, `ANA-02` | `FlashAnalyzeMenu` whole/part/all-branches/settings; `AutoAnalyzeMenu` lightning; Ctrl+B | `analysis-max-visits` default 1. `auto-quick-analyze-on-load` default true. Toolbar `flash-analyze`… | Missing command → settings dialog; failed settings save does not start analysis |
| `CAP-04-ANA-07` | 208 | mapped | `ANA-02` | `AutoAnalyzeMenu` 整盘精析 Shift+Ctrl+B | Baseline visits 32, minimum deep 500. Plan is not a file | `fail` engine / gameChanged; pause closes engine; resume fails if game identity changed |
| `CAP-04-ANA-08` | 209 | exclusion | `ANA-01`, `ANA-02`, `GUIDE-01` | `AutoAnalyzeMenu` 自动分析 → `StartAnaDialog`; bottom toolbar; `Menu.stopAutoAnalyze`; `AnalysisTable` | start/end move default -1; anaBlack/anaWhite true; `analyzeAllBranch` false; `exitAutoAnalyzeByPaus… | Dialog cancel resets auto-ana if cancelled |
| `CAP-04-ANA-09` | 210 | mapped | `ANA-07`, `APP-02` | `AutoAnalyzeMenu` batch / batch-deep; `AnalysisTable` add files, stop/start | `batch-analysis-playouts` default 100. Persist that ui key + filesystem object | Destroy engine on stop in analysis-mode |
| `CAP-04-ANA-10` | 211 | exclusion | `UI-03` | Tracking UI (not fully enumerated); interval + target visits | Default max visits 500. Persist tracking appearance + max visits | Lease unavailable; context mismatch |
| `CAP-04-ANA-11` | 212 | mapped | `ANA-04`, `UI-03` | Always-on when analysis payload exists; View suggestion/estimate prefs; delay candidates; black/white candidates | `limit-max-suggestion` 10, `limit-branch-length` 15, `useMovesOwnership` true, estimate off, delay … | Occupied-point candidates stripped |
| `CAP-04-ANA-12` | 213 | mapped | `ANA-03`, `ENG-05` | Implicit in switch, navigation, cancel, whole-game pause/cancel | N/A | Ignore stale; rollback switch; cancel session |
| `CAP-04-ANA-13` | 214 | mapped | `ANA-08`, `SGF-05`, `ANA-05` | Save/open SGF; `appendWinrateToComment` default true | Persist in SGF file | Placeholder 50% winrate + visits must not look like a complete curve |
| `CAP-04-ANA-14` | 215 | mapped | `ANA-05`, `ANA-08` | Auto on open if `autoLoadCache`; auto after analysis if `autoSaveAnalysis`; `CacheStatusBadge` | autoLoad/autoSave true. Persist `analysis-cache.sqlite3` (browser localStorage prefix in preview) | Status error; incompatible payload message |
| `CAP-04-ANA-15` | 216 | mapped | `ENG-07`, `ENG-04` | Start failure status; `EngineFailedMessage`; analysis settings fromError; KataGo JSON `error` | N/A | Click-to-repair; settings after missing command. `EngineFailedMessage` redacts secrets |
| `GM-HUMAN-GENMOVE` | 226 | mapped | `GAME-01`, `GAME-02`, `GAME-04`, `GAME-05`, `APP-04` | 棋局 → 新对局 genmove (`N`); double-menu NewGame; 人机续弈 as White/Black; Enter continue | Komi fallback 7.5. Time mode from genmove flags else normal. Ponder from `playponder` (field defaul… | No engine selected → modal. Engine switch failure aborts. WebSocket advanced clock unsupp… |
| `GM-HUMAN-ANA` | 227 | mapped | `ANA-04`, `GAME-07` | Alt+N `startAnalyzeGameDialog`; menu 人机对局(分析模式); analyze-mode continue items; Alt+Enter. Engines with `noAnalyze` redirect to genmove | Komi 7.5. Resign fields persist. Shares continue/color/komi/handicap keys with genmove. Apply requi… | No engine; wrong AI-move settings; contribute block; HumanSL teardown deferral; lease con… |
| `GM-PK-SESSION` | 228 | mapped | `GAME-03`, `GAME-06`, `GAME-05`, `SGF-05` | 棋局 → 新对局 engine game (Alt+E); `NewEngineGameDialog` OK; bottom toolbar PK panel. Stop/pause/revise are **this** Capability, not separate ce… | Komi 7.5, handicap 0, batchLimit 1, maxMoves 450, autosave true, resign minMove 0 / consecutive 2 /… | Same-engine message; invalid analysis limits; occupied; startFailed unless user cancel. G… |
| `GM-HUMANSL` | 229 | mapped | `GAME-08`, `GAME-01`, `GAME-02`, `GAME-05` | 棋局 → 新对局 HumanSL; toolbar AI Coach; training bar Pass / Retry AI / Finish and review; keyboard P when live controller exists | Mode POST_GAME_REVIEW, opponent RANK, rank 3 dan, color RANDOM, moveTime 10s, handicap 0, komi 7.5,… | URL-SGF live sync blocks start. Failed start rolls back StartBoardSnapshot. AI no-respons… |
| `GM-MATCH-PASS` | 230 | mapped | `GAME-02`, `GAME-03`, `SGF-04`, `UI-05` | Keyboard P (HumanSL → `humanPass()`, else `board.pass()`); Menu 停一手 always `board.pass()`; double-menu pass; PK engines pass as `GameOutcom… | N/A | HumanSL ignores pass if finished or not human turn. Menu Pass during HumanSL vs keyboard … |
| `GM-MATCH-STOP` | 231 | mapped | `GAME-01`, `GAME-02` | Space / `Menu.breakGame` / `togglePonderMannul`; double-menu Stop. HumanSL also: training-bar Finish. PK stop is `GM-PK-SESSION` | `autoSavePlayedGame` gates Java autosave under MyGames | HumanSL abort is async/retryable. If no match is active, Space falls through to ponder to… |
| `GM-CONTRIBUTE` | 232 | gap | `GAME-09` | Contribute menu / `ContributeView` (visible if `SET-CONTRIBUTE-MENU-VIS`). Constructor starts the process, sets `isContributing`, clears th… | Server https://katagotraining.org/ unless overridden. Persist contribute* command/path/user/batch (… | SSH login failure leaves javaSSHClosed. Benchmark-sync suppression can skip start. Pause/… |
| `GM-MATCH-RULES-START` | 233 | mapped | `GAME-04`, `GAME-07` | All four start dialogs; GameInfo dialog `I`; live komi spinner (02 overlap); PK max-move in EnginePkConfig | Human/PK/HumanSL default komi 7.5. Human handicap combo; PK empty handicap = even 0. Persist family… | Illegal komi/handicap strings. WebSocket engines reject advanced clocks. Mid-match txtKom… |
| `CAP-06-ONLINE-URL` | 243 | mapped | `PROV-01`, `PROV-03`, `PROV-05` | File → Open online link; shortcut Q; Yike Live Center Sync/double-click/Enter → `syncOnline` | Refresh interval numeric default 需定向运行核验. `alwaysGotoLastOnLive` default false; `openHtmlOnLive` de… | Network via AjaxHttpRequest / NetworkProxy / websocket. Yike waiting-page URLs invalidate… |
| `CAP-06-YIKE-LIVE-CENTER` | 244 | gap | `PROV-01`, `PROV-03` | Sync → 弈客直播 (Shift+O); BottomToolbar 直播 popup; Input Shift+O | Dialog 920×420; first category Recommend; HTTP timeout 10_000 ms. Categories recommend `"1"`, local… | loadFailed status; buttons disabled while loading. Signed GET. No retry loop in fetchLive… |
| `CAP-06-YIKE-WEB` | 245 | mapped | `PROV-03` | Sync → 打开弈客网页版; Live Center Web; BottomToolbar same item | JCEF bundle required. `openHtmlOnLive` default true. Observed-URL freshness 30s | BrowserFrame startFailed; Retry or Open in system browser. Auto-sync from browser vs Onli… |
| `CAP-06-YIKE-HALL` | 246 | mapped | `PROV-01`, `PROV-03` | Sync → 弈客大厅; BottomToolbar 弈客大厅 | Fixed hall URL. Persistence N/A | Same JCEF failure path as `CAP-06-YIKE-WEB` |
| `CAP-06-FOX-KIFU` | 247 | mapped | `PROV-02`, `SGF-07` | Sync → 野狐(腾讯)棋谱; BottomToolbar live popup; top `btnFoxKifu` when `showBasicBtn` (default true) | Connect 20s, read 25s, 3 retries with 350ms×attempt. `foxAfterGet` default 0. Persist `last-fox-nam… | Empty list message; CGI then H5 fallback; stale GetFoxRequest results ignored |
| `CAP-06-TENCENT-KIFU` | 248 | mapped | `PROV-04`, `SGF-07`, `PROV-02`, `PROV-05` | Sync → Tencent kifu; top `btnTencentKifu`. Not in BottomToolbar live popup | Connect 20s, read 25s, 3 retries. Session token default lizzieyzy-next. Fetch num 100; UI 25/tab. E… | Same retry/timeout class as Fox; emitError on exception |
| `CAP-06-READBOARD` | 249 | gap | `READ-01`, `READ-02`, `READ-03` | Sync → 棋盘识别工具 **Windows menu only**, accelerator Alt+O; BottomToolbar 棋盘同步 Windows only; Input Alt+O is **not** OS-gated | `alwaysSyncBoardStat` true; `readBoardGetFocus` true; `notPlaySoundInSync` true; `usePipeReadBoard`… | Missing exe/bat; place failed; GMA unsupported without KataGo; sidecar update installer. … |
| `CAP-06-WEBBOARD` | 250 | mapped | `PUB-01` | Sync → WebBoard Start/Stop + Copy URL; BottomToolbar live popup Start/Stop + Force-exit trial | HTTP 9998 (try +0..9); WS 9999; max connections 20; bind 0.0.0.0; idle 5 min. Optional `web-board` … | Start returns false if no HTTP/WS port; WS fail stops HTTP; trial denied in_use / engine_… |
| `CAP-06-SHARE-CURRENT` | 251 | exclusion | — | `Ctrl+E` and `Alt+B` call `shareSGF()` | N/A. No observable state is written | No feedback; the registered action returns |
| `CAP-06-SYNC-SETTINGS` | 252 | mapped | `PROV-01`, `PROV-02`, `PROV-03`, `PROV-04`, `READ-02`, `PREF-01` | Sync menu checkboxes: open HTML on live; always jump to last live move; readboard always-sync; readboard get-focus | `openHtmlOnLive=true`; `alwaysGotoLastOnLive=false`; `alwaysSyncBoardStat=true`; `readBoardGetFocus… | No independent setting-error channel. Persistence-write failure presentation 需定向运行核验. Whe… |
| `REL-C01` | 262 | mapped | `REL-04` | Download from goagent.top/download or GitHub Releases; Windows portable unzip + flavor EXE; Windows `installer.exe`; macOS DMG drag to Appl… | Windows ordinary users: `.portable.zip`. Flavor by GPU. Portable writes under package `user-data/` … | Wrong package / incomplete unzip / Gatekeeper: TROUBLESHOOTING. Clean-machine install 需定向… |
| `REL-C02` | 263 | mapped | `REL-04` | Process start (`Lizzie.main` → `WorkDirectoryResolver.resolve`) | Portable if `.lizzie-portable` marker; else platform fallback (`LizzieYzyNext` / `~/.lizzieyzy-next… | Resolver records diagnostics; portable recovery backup defined. Profile save failure → re… |
| `REL-C03` | 264 | mapped | `ENG-02`, `UI-04`, `REL-05` | Startup when `firstTimeLoad` or hostname change. Interactive 一键设置 is 04 | Bundled with-katago / nvidia / opencl expected to auto-write default engine. `without.engine` stays… | Auto-setup exception logged; user can open 04 one-click setup. NVIDIA CUDA staging 需定向运行核验 |
| `REL-C04` | 265 | mapped | `REL-05` | Implicit at update-check planning and first-run setup; flavor marker; installed manifest | macOS updater flavor default `with-katago`. Persist `lizzieyzy-next-installed-manifest.json` after … | Missing matching package → `NO_PACKAGE`. Missing bundled engine → `REL-C03` repair chip |
| `REL-C05` | 266 | mapped | `REL-03` | Help → 检查更新 → `CheckUpdateDialog`. Opening the page does **not** use the network; Check starts discovery | Channel `stable`; source `official`. Unset version `next-dev`. Persist `update-channel`, `update-so… | Stay on page with localized unpackaged / unsupported / no-update / no-package / fetch / i… |
| `REL-C06` | 267 | mapped | `REL-06`, `APP-03` | Offer dialog from successful `REL-C05`: 立即更新, pause/resume/cancel, View Release, close | Selected components = those newer than installed manifest. Staging under work-dir `update/staging/<… | See `REL-C08`. Shutdown only after successful helper launch. Helper restart “first exe in… |
| `REL-C07` | 268 | mapped | `REL-07` | `PackageUpdateDialog`: 下载新版, pause/resume/cancel, View release | Download dir `~/Downloads` if a directory, else work-dir `update/downloads` | Cancel keeps `.part` for resume. Network fallback R2→GitHub. Desktop unsupported → IOExce… |
| `REL-C08` | 269 | mapped | `REL-08` | Automatic inside `WindowsUpdateApplier.apply`; download failures stay in `REL-C06` dialog | Backup dir under staging; not a user setting | Before replace, current files move to `staging/backup-<timestamp>/`. On exception, restor… |
| `REL-C09` | 270 | mapped | `REL-02`, `REL-04`, `APP-01` | Installer Start Menu / desktop shortcuts; macOS Dock icon; macOS Privacy & Security Open Anyway. Linux `.desktop` not a first-class user co… | N/A | Gatekeeper/SmartScreen: TROUBLESHOOTING. `.sgf` OS association 需定向运行核验 |
| `REL-C10` | 271 | mapped | `REL-09` | Help → Diagnostics and Logs; Help → Stop Full Trace (enabled iff full-trace active); dialog Apply / Export / Cancel export / Open log folder | Defaults: diagnostics **enabled**, all modules, all scopes. Persist `uiConfig.logging`. Full Logs r… | Apply failure reverts UI from runtime. Export cancel via flag. Exact on-disk log folder p… |
| `REL-C11` | 272 | mapped | `REL-10` | Help → About → `openConfigDialog2(2)`; settings modern nav About; check-update page header shows `Lizzie.nextVersion` | Packaged builds inject `LIZZIE_NEXT_VERSION`. Unset → `next-dev`. Maven `2.5.3` is **not** the user… | N/A |
| `REL-C12` | 273 | mapped | `REL-09`, `REL-10` | About links (`REL-C11`); GitHub issue templates; `SUPPORT.md`; Discussions; QQ group in docs only (not an in-app control) | N/A | N/A |
| `REL-C13` | 274 | mapped | `SGF-09` | Help → `clearAllPersonalData` with OK/Cancel | Removes four `uiConfig` keys only, then `config.save()`. Does **not** wipe engine profiles, work di… | Save IOException printed |

Domain line ranges: 01 `SHELL-*` L73–85; 02 `SET-*` L97–131; 03 Accepted L143–155 and adjacent L160–182; 04 L192–216; 05 L226–233; 06 L243–252; 07 `REL-C*` L262–274. Cross-domain routing (not extra Capabilities): L278–301. Unregistered/dead surfaces: L351–368. Abandoned summary: L370–399.

### Parity Items (status, phase, depends, remaining gap)

R0–R2 rows have no `Depends on` column (historical). From R3 onward every row has `Depends on`.

| ID | Status | Capability | Phase | Depends on | Remaining gap (abbrev.) |
| --- | --- | --- | --- | --- | --- |
| `BASE-01` | Accepted | Fixed Java behavior baseline | R0 | historical (R0–R2) | None. |
| `BASE-02` | Accepted | Stable parity inventory | R0 | historical (R0–R2) | None; maintain the inventory as slices land and add successor items when scope changes ma… |
| `SGF-01` | Accepted | Semantic SGF tree round-trip | R1 | historical (R0–R2) | None for the frozen R1 semantic set. |
| `SGF-02` | Accepted | Tree-shaped wire DTO and `NodePath` | R1 | historical (R0–R2) | None. |
| `SGF-03` | Accepted | Variation navigation | R1 | historical (R0–R2) | None. |
| `SGF-04` | Accepted | Board and branch editing | R1 | historical (R0–R2) | None. |
| `SGF-05` | Accepted | Personal comment editing and separation | R1 | historical (R0–R2) | None. |
| `SGF-06` | Accepted | Save the current edited game | R1 | historical (R0–R2) | None. |
| `RULE-01` | Accepted | Go-rule behavior used by SGF editing | R1 | historical (R0–R2) | None for the rules required by the frozen R1 fixtures. |
| `UI-01` | Accepted | Baseline desktop chrome and review layout | R2 | historical (R0–R2) | None. |
| `UI-02` | Partial | Non-blocking board interaction | R2 | historical (R0–R2) | No evidence yet demonstrates engine-event delivery while a board mutation promise is pend… |
| `UI-03` | Accepted | Candidate selection, hover intent, and stale eviction | R2 | historical (R0–R2) | None for the frozen R2 hover/stale-publication contract. Engine-manager job lifecycle and… |
| `UI-04` | Accepted | No-engine desktop workflow | R2 | historical (R0–R2) | None. |
| `UI-05` | Accepted | Core review actions and shortcuts | R2 | historical (R0–R2) | None for claimed R2 rows. Remaining Java actions stay unclaimed. |
| `ENG-01` | Accepted | Engine profiles and asset checks | R3 | — | No migration gap for profile storage itself. |
| `ENG-02` | Missing | Foreground Engine Run | R3 | ENG-01 | There is no authoritative no-engine/starting/ready/stopping/error identity, immutable ada… |
| `ENG-03` | Missing | Transactional A → B foreground switch | R3 | ENG-02 | A new engine is not started and proven ready before becoming primary. |
| `ENG-04` | Missing | Failed switch and stale identity rejection | R3 | ENG-03 | A failed or late B start cannot be modeled while preserving A as primary. |
| `ENG-05` | Partial | Cancellable selected-node job identity | R3 | ENG-02 | One-shot work has no manager-owned selected-node job start, explicit cancellation, comple… |
| `ENG-06` | Missing | Default foreground autoload | R3 | ENG-01, ENG-02 | There is no zero-or-one Autoload Default, first-use off mark, or startup validation that … |
| `ENG-07` | Missing | Foreground failure and manual recovery | R3 | ENG-02 | Start, asset, protocol, nonzero-exit, timeout, and cancellation outcomes are not typed an… |
| `ANA-01` | Partial | One-shot position analysis | R4 | ANA-03, ENG-05 | The request bypasses the manager-owned selected-node lane, adapter capability admission, … |
| `ANA-02` | Partial | Mainline whole-game analysis | R4 | ANA-03 | The path still spawns from a profile and lacks the accepted manager session and navigatio… |
| `ANA-03` | Partial | Analysis job lanes | R4 | ENG-02, ENG-05, ENG-07 | Selected-node work has no independent manager-owned identity and UI state can outlive its… |
| `ANA-04` | Partial | Next-native analysis presentation | R4 | ANA-01, ANA-03, UI-03 | Results are not fully bound to adapter capability and run/job/node identity. |
| `ANA-05` | Accepted | Analysis cache basics | R4 | — | Schema migration tests wait until the schema changes beyond the current MVP. |
| `PREF-01` | Partial | Durable preferences and Preferences surface | R5 | — | Missing-file defaults, unreadable-file isolation, atomic write, and one categorized surfa… |
| `SGF-07` | Partial | Safe current-game replacement | R5 | SGF-01, SGF-02, SGF-06 | Parse/validate-before-commit and full-state preservation for every caller are incomplete. |
| `APP-01` | Missing | Native File Activation | R5 / R11 | SGF-07 (semantic work); REL-04 additional R11 final-acceptance gate only | Cold/warm `.sgf`/`.gib` activation, one-window focus, and shared replacement semantics ar… |
| `APP-02` | Missing | Window file-drop dispatch | R5 | SGF-07, passed R5 APP-01 semantic gate (never APP-01 Accepted) | One-file drop through `SGF-07` and explicit multi-file routing are absent. |
| `APP-03` | Missing | Safe Graceful Shutdown | R5 | SGF-06, PREF-01 | Save / Discard / Cancel, failed-save abort, ordered teardown, timeout, Retry, and context… |
| `APP-04` | Missing | Current-game Session Recovery | R5 | PREF-01, SGF-01, SGF-02 | Review-only recovery of current game identity and dirty state is absent. |
| `APP-05` | Missing | Shortcut Registry and Reference | R5 | UI-05 | Registry-owned labels, aliases, tests, conflicts, and reference are absent. |
| `SGF-08` | Missing | GIB current-game import | R6 | SGF-07 | Tygem GIB intake through `SGF-07` is absent. |
| `SGF-09` | Missing | Recent kifu intake | R6 | SGF-07, PREF-01 | Persisted successful SGF/GIB opens, reopen, and narrow clear-history are absent. |
| `SGF-10` | Partial | New game document | R6 | SGF-07 | New/Clear entry points are not one `SGF-07` flow with chosen size, default komi, and clea… |
| `SGF-11` | Missing | Starting-position editor | R6 | SGF-01, SGF-04 | `AB`/`AW`/`AE`/`PL` editing and conversion are absent. |
| `SGF-12` | Missing | Structural tree editing | R6 | SGF-04 | Delete-current, undo/redo, promote-to-main, and return-to-main are absent. |
| `SGF-13` | Missing | Game metadata editor | R6 | SGF-01, SGF-10 | Atomic player-name and komi editing is absent. |
| `SGF-14` | Missing | Interactive markup | R6 | SGF-01 | Create/replace/remove labels and standard marks is absent. |
| `REVIEW-01` | Partial | Direct review navigation | R6 | SGF-03 | Exact complete-tree `NodePath` selection and owner-scoped presentation follow are incompl… |
| `REVIEW-02` | Missing | Try-play sandbox | R6 | SGF-04, REVIEW-01 | An isolated trial line is absent. |
| `REVIEW-03` | Missing | Position scoring | R6 | PREF-01, SGF-13 | Area/territory scoring with persisted rule and explicit root `RE` commit is absent. |
| `REVIEW-07` | Partial | Board display preferences | R6 | PREF-01 | The two booleans are not durably synchronized across controls and rendering. |
| `REVIEW-08` | Missing | Review move sound | R6 | PREF-01 | Default-on durable move sound is absent. |
| `LAYOUT-01` | Missing | Draggable workspace proportions | R7 | UI-01 | Main proportions cannot be adjusted. |
| `LAYOUT-02` | Missing | Persisted workspace proportions | R7 | LAYOUT-01, PREF-01 | Debounced atomic persistence and visible unsaved retry are absent. |
| `LAYOUT-03` | Missing | Restore panel sizes | R7 | LAYOUT-01 | Panel-size/board-proportion restore is absent. |
| `LAYOUT-04` | Missing | Rail visibility | R7 | PREF-01 | Independent durable collapse is absent. |
| `WINDOW-01` | Missing | Window geometry and reset | R7 | PREF-01, LAYOUT-02 | Valid bounds persistence, invalid-display recovery, and explicit narrow reset are absent. |
| `APPEAR-01` | Partial | Curated appearance | R7 | PREF-01 | Durable contract and system DPI confirmation are incomplete. |
| `GUIDE-01` | Missing | Contextual guidance | R7 | PREF-01, unresolved producer decision | Ticket 09 requires the domain-04 auto-analyze education path, but Ticket 11 abandons that… |
| `ENG-09` | Missing | Multi-backend engine profiles and capabilities | R8 | ENG-01, ENG-02 | Adapter kind, shared fields, adapter-owned settings, declared capabilities, and run snaps… |
| `ENG-10` | Missing | Generic GTP game adapter | R8 | ENG-09, ENG-02 | Bounded handshake, exact-position sync, Compute Budget mapping, and typed outcomes are ab… |
| `GAME-01` | Missing | Match ownership and transactional start | R9 | ENG-02, ENG-03, ENG-04, ENG-07, ENG-09, ENG-10, SGF-07, APP-04, REVIEW-03, PREF… | Authoritative states, one global session, and all-or-nothing reservation are absent. |
| `GAME-02` | Missing | Human-vs-Engine match | R9 | GAME-01, GAME-04, GAME-05, ANA-04 | New/continue, role, pass, resign, Stop, and capability-gated presentation are absent. |
| `GAME-03` | Missing | Single-game engine-vs-engine match | R9 | GAME-01, GAME-04, GAME-05 | Two-run single-game PK with pause/resume/Stop and terminal rules is absent. |
| `GAME-04` | Missing | Shared match rules and Compute Budgets | R9 | PREF-01, ENG-09, ENG-10 | One validated durable start model is absent. |
| `GAME-05` | Missing | Match SGF and review handoff | R9 | SGF-07, APP-04, REVIEW-03 | Committed tree updates, metadata, personal/generated separation, terminal `RE`, and revie… |
| `PROV-01` | Partial | Yike preview and one-shot import | R10 | SGF-07, PREF-01, APP-03 | Native public category/URL preview, unite-room parsing, complete one-shot import, and liv… |
| `PROV-02` | Partial | Fox preview and one-shot import | R10 | SGF-07, PREF-01, APP-03 | Pagination, bounded recents, required timeouts, and live failures are incomplete. |
| `PROV-03` | Missing | Yike ongoing synchronization | R10 | PROV-01, SGF-07, APP-03 | Explicit external-authoritative sync, stale suppression, last-good recovery, and browser … |
| `PROV-04` | Missing | Tencent kifu preview and import | R10 | SGF-07, PREF-01, APP-03 | Username/`chessId` preview, `lastCode` pagination, recents, and one-shot `SGF-07` import … |
| `READ-01` | Partial | readboard sidecar readiness | R10 | APP-03 | Real process/version/path/timeout/restart behavior is unvalidated. Readiness does not imp… |
| `READ-02` | Partial | readboard ongoing synchronization | R10 | READ-01, SGF-07 | Preview is not equivalent one-way external-authoritative ongoing sidecar sync into the Ru… |
| `READ-03` | Accepted | Explicit OCR limitation | R10 | — | None within current scope. |
| `REL-01` | Partial | Release preflight and dry-run | R11 | — | Current evidence does not prove installable production assets. Validators still need to i… |
| `REL-02` | Missing | Trusted production artifacts | R11 | — | Signed envelopes, verified payloads, Authenticode-signed Windows artifacts, and signed/no… |
| `REL-03` | Missing | Update discovery and offer | R11 | REL-02 | Manual Help discovery, stable/beta and official/GitHub policy, SemVer comparison, OS prox… |
| `REL-04` | Missing | Platform package lifecycle | R11 | — | Each Canonical Artifact (Windows NSIS plus portable, macOS DMG, Linux AppImage) has not b… |
| `REL-05` | Partial | Managed installed components | R11 | REL-02 | A signed installed manifest identifying `app-core` and acquired KataGo backend/default-mo… |
| `REL-06` | Missing | Windows component download and apply | R11 | REL-03, REL-05, APP-03 | Installed-component diffing, resume/cancel, official-to-GitHub fallback, helper launch, e… |
| `REL-07` | Missing | macOS/Linux package update handoff | R11 | REL-03 | Verified full-package download, pause/resume/cancel/fallback, and open-DMG or containing-… |
| `REL-08` | Missing | Update failure, rollback, and recovery | R11 | REL-06 | Pre-apply non-mutation, Windows reverse rollback, durable result journal, restored-versio… |
| `REL-09` | Missing | Diagnostics and support bundle | R11 | — | Bounded ordinary logs, explicit full trace, sanitized cancellable export, open-folder acc… |
| `REL-10` | Partial | Product identity and support routing | R11 | REL-04 | Packaged SemVer and channel, plus working Releases, Issues, and support-document links, a… |
| `I18N-01` | Deferred | Complete localization | Deferred | PREF-01 | Complete resources for every supported locale, persisted locale selection, deterministic … |
| `SGF-15` | Deferred | Direct stone manipulation | Deferred | SGF-11, SGF-04 | Forced colored insertion, list insertion, and stone drag remain unscheduled. |
| `SGF-16` | Deferred | Whole-tree transforms | Deferred | SGF-01, SGF-14 | Whole-tree color swap, rotate, and mirror remain unscheduled. |
| `REVIEW-04` | Deferred | Board-point move search | Deferred | REVIEW-01 | Double-click/search navigation remains unscheduled. |
| `REVIEW-05` | Deferred | Configurable main-board autoplay | Deferred | UI-05 | A validated persisted main-board interval remains unscheduled. Candidate-variation, sub-b… |
| `REVIEW-06` | Deferred | Ladder continuation | Deferred | SGF-04, REVIEW-01 | Automatic ladder continuation remains unscheduled. |
| `EXPORT-01` | Deferred | Current-branch SGF export | Deferred | SGF-06 | Selected-branch SGF export remains unscheduled. |
| `EXPORT-02` | Deferred | Review-board image export | Deferred | UI-01 | Review-board image export remains unscheduled. Analysis/sub-board/win-rate image exports … |
| `ENG-08` | Deferred | Engine profile catalog ordering | Deferred | ENG-01, ENG-09 | User-directed first/up/down/last reordering remains unscheduled. |
| `ANA-06` | Deferred | Continuous current-node analysis | Deferred | ANA-01, ANA-03, ENG-02 | Explicit default-off, run-scoped continuous current-node analysis remains unscheduled. |
| `ANA-07` | Deferred | Batch SGF analysis | Deferred | ANA-02, ANA-03 | A session-only SGF file queue remains unscheduled. |
| `ANA-08` | Deferred | SGF analysis exchange | Deferred | ANA-04, SGF-01, SGF-05; not ANA-05 | Structured SGF analysis import/export remains unscheduled. |
| `ANA-09` | Deferred | Named rich-analysis engine adapters | Deferred | ENG-09, ANA-04 | Named engine/version rich-analysis adapters remain unscheduled. |
| `GAME-06` | Deferred | PK batch and advanced controls | Deferred | GAME-03, GAME-05 | Batch count, opening catalogs, color exchange, live revision, manual intervention, and du… |
| `GAME-07` | Deferred | Competitive clocks and automatic resign | Deferred | GAME-02, GAME-03 | Application-owned remaining-time clocks and auto-resign remain unscheduled and require a … |
| `GAME-08` | Deferred | HumanSL AI Coach match | Deferred | GAME-01, GAME-02, GAME-05 | Independent HumanSL Match Session remains unscheduled until core Human-vs-Engine, SGF han… |
| `GAME-09` | Deferred | Contribute board session | Deferred | GAME-01 | Board occupancy and Match Session exclusion remain unscheduled until domain 06 admits a s… |
| `PROV-05` | Deferred | Tencent/huanle live synchronization | Deferred | SGF-07, APP-03 | Non-Yike live websocket/qipu protocols remain unscheduled. |
| `PUB-01` | Deferred | WebBoard LAN publishing | Deferred | APP-03 | Start/stop LAN publish and copy-access URL remain unscheduled. Trial counters and interna… |

### Migration Plan phases, dependencies, exits

| Phase | Workflow | Owns (Plan) | Start gate (Plan) | Exit (Plan, abbreviated) |
| --- | --- | --- | --- | --- |
| R0 | Baseline and inventory | `BASE-01` `BASE-02` | — | `BASE-01` accepted; claims map to items |
| R1 | SGF and board semantics | `SGF-01`–`SGF-06` `RULE-01` | — | those IDs accepted |
| R2 | Core desktop review UI | `UI-01`–`UI-05` | — | `UI-01` `UI-03` `UI-04` `UI-05` accepted; `UI-02` residual recorded |
| R3 | Foreground Engine Lifecycle | `ENG-02`–`ENG-07`; `ENG-01` frozen | R2 exited | `ENG-02`–`ENG-07` accepted; KataGo smoke |
| R4 | Analysis | `ANA-01`–`ANA-04`; `ANA-05` frozen | `ENG-02`–`ENG-07` accepted | `ANA-01`–`ANA-04` accepted |
| R5 | Safe Current Game / shell | `PREF-01` `SGF-07` `APP-02`–`APP-05` + `APP-01` semantic gate | R1 and R2 exited | those accepted; `APP-01` semantic gate recorded, item unaccepted pending `REL-04` |
| R6 | SGF authoring / review | `SGF-08`–`SGF-14` `REVIEW-01`–`REVIEW-03` `REVIEW-07` `REVIEW-08` | `SGF-07` `PREF-01` accepted | those accepted |
| R7 | Adaptive workspace | `LAYOUT-01`–`LAYOUT-04` `WINDOW-01` `APPEAR-01` `GUIDE-01` | `PREF-01` accepted; `GUIDE-01` blocked on producer | layout/window/appearance accepted; `GUIDE-01` only after Ticket 16 producer |
| R8 | Engine adapters | `ENG-09` `ENG-10` | `ENG-02`–`ENG-07` accepted | those accepted |
| R9 | Game modes | `GAME-01`–`GAME-05` | 11 named Accepted IDs (see Ticket 15) | `GAME-01`–`GAME-05` accepted |
| R10 | Providers / readboard | `PROV-01`–`PROV-04` `READ-01`–`READ-03` | `SGF-07` `PREF-01` `APP-03` | those accepted; `PROV-05` `PUB-01` not exits |
| R11 | Release | `REL-01`–`REL-10` + final `APP-01` | `REL-01` any time; `REL-06` needs `APP-03`; `APP-01` needs R5 semantic gate + `REL-04` | global REL items + per-platform branch; `APP-01` accepted |
| Deferred | Unnumbered queue | 19 IDs | later plan revision | never R3–R11 exits until started |

**Next executable batch:** Slice R3-A = `ENG-02` identity/lifecycle via `KataGoAnalysis` only (`docs/MIGRATION_PLAN.md` §Next Executable Batch). Matches Ticket 15.

Named critical edges: `docs/MIGRATION_PLAN.md` §Critical Edges (L215–236) and `APP-01` split graph (L238–258). `R10`/`R11` numbers are priority, not a hard edge.

## Check-by-check

### C1. Closed route: every census Capability / Entry Point → exactly one supported Parity Item or explicit exclusion

**Fail.**

- 109 rows name supported IDs (including explicit splits such as `SHELL-01` → `UI-01`/`UI-04`/`REL-04`, `GM-HUMAN-GENMOVE`+`GM-HUMAN-ANA` merge into `GAME-01`/`GAME-02`/`GAME-04`/`GAME-05`).
- 17 rows are explicit exclusions. Abandoned halves of split rows are listed again in the Abandoned summary so they are not confused with unregistered chrome.
- 13 rows/remainders have **no** supported Parity ID. Inventory, Matrix, and Plan all say Ticket 16 must close them and forbid silent absorption into `ANA-04`, `UI-01`, `PREF-01`, `READ-02`, `PROV-01`/`PROV-03`, `EXPORT-02`, `UI-05`/`REVIEW-05`, or `GAME-09`.
- Domain 01 routing keys are not extra Capabilities (`docs/JAVA_CAPABILITY_INVENTORY.md` L28, L278–301).
- `CAP-04-ANA-01`/`02`/`03` are **not** among the 13. Ticket 11 “Absorbed redesign” states the Java preferences are not retained (reuse is invariant; no hidden preload; no auto-quit). That is a closed product decision into the `ENG-02` resident-run model, not an owner-route-without-ID.

Unregistered/dead surfaces (Share menubar never added, commented resume item, lab JVM flags, etc.) are indexed as non-Entry-Points (L351–368). Reachable no-ops that *were* Capabilities (`SHELL-04`, `SHELL-09`, `CAP-06-SHARE-CURRENT`) are exclusions, not this table.

### C2. Ambiguous / multiple ownership, duplicate claims, orphaned IDs

**Pass for duplicate census IDs and Accepted-scope expansion; Fail for the 13 remainders (same as C1).**

- No duplicate Frozen IDs. No duplicate Parity IDs.
- Explicit splits/merges are labeled. Inventory forbids expanding Accepted `UI-01` `UI-03` `UI-05` `SGF-01`–`SGF-06` `ENG-01` `ANA-05`.
- `GAME-01`–`GAME-03` were **Missing** at `18c6d189` (`Engine-game session state` / `Revisable batch limit` / `Game-mode board and comment integration`). Ticket 12 replaced those Missing identities. That is allowed successor/replacement of non-accepted history, not a silent Accepted rewrite. Original batch-limit behavior now lives on Deferred `GAME-06`.
- `REL-C12` is absorbed into `REL-09`/`REL-10` acceptance surfaces, not a second item.

### C3. Original Accepted items preserved

**Pass**, with two allowed non-scope edits.

Compared `git show 18c6d189:docs/PARITY_MATRIX.md` to the current matrix for the 16 Accepted IDs.

| ID | ID/status/capability/gap/acceptance | Evidence | Phase |
| --- | --- | --- | --- |
| `BASE-01` | Unchanged | Unchanged | R0 |
| `BASE-02` | Unchanged except evidence sentence | “41 stable parity items” → “stable parity items … historical R0 inventory listed 41 items before the Tickets 08–14 successor expansion” | R0 |
| `SGF-01`–`SGF-06` `RULE-01` | Unchanged | Unchanged | R1 |
| `UI-01` `UI-03` `UI-04` `UI-05` | Unchanged | Unchanged | R2 |
| `ENG-01` | Unchanged | Unchanged | R3 (new `Depends on` column is `—`) |
| `ANA-05` | Unchanged | Unchanged | R4 |
| `READ-03` | Unchanged | Unchanged | R7 → **R10** |

`BASE-02`’s own remaining gap already required maintaining the inventory and adding successors. The evidence rewrite records that expansion. It does not change acceptance (“every claimed migration capability maps to a stable item”).

`READ-03` phase R7→R10 follows the map rule that R4 and later may be split or reordered (`.scratch/migration-coverage-audit/map.md` L16). Status, OCR-unsupported scope, evidence, remaining gap, and acceptance are unchanged.

No unsupported completion: no new Accepted IDs. Partial rows that gained plumbing still say existing paths are not completion (`PROV-01`, `READ-02`, `REL-05`, `SGF-07`, `ANA-01`–`ANA-04`). `UI-02` remains Partial solely for engine-event delivery during a pending mutation.

### C4. Applicable contracts present under the right owner

**Pass.**

Spot-checked every inventory row for non-empty Entry Points, Default/persistence, Failure/recovery, Frozen evidence, Disposition, Mapping (zero empty cells). Every matrix item has Status, Capability, both evidence columns, remaining gap, acceptance, phase. From R3 onward `Depends on` is present (empty only as explicit `—`). Plan states start gates, order, and exits per phase, plus Deferred admission.

Defaults/persistence/failure for semantic settings travel with behavior owners (`ENG-06` autoload first-use off, `REVIEW-07` coords on / move numbers off, provider timeouts, `REL-03` OS proxy). `PREF-01` is mechanism-only. That matches Ticket 15 and the Inventory ownership table (L44–61).

Repository vs live/environment vs (for R11) Repository Release / release-environment / Installed Live Evidence stay distinct in Matrix rules and Plan Evidence Model. Partial provider/readboard/release rows do not treat wiring as live proof.

### C5. Counts, cross-document references, Deferred coverage, roadmap order, Decisions so far

**Pass on counts, Deferred membership, Decisions so far, and R3-as-next-batch. Fail on Matrix vs Plan `Depends on` for several Deferred/active edges.**

Decisions so far in `map.md` L24–38 lists resolved Tickets 01–15. Ticket 16 is claimed, not yet a decision. That is complete for resolved work.

Deferred 19 IDs are identical in Matrix Phase Map, Matrix Deferred table, Plan Deferred table, and Inventory’s “Deferred items are not Abandoned” sentence (L401).

**Dependency mismatches** (Plan Deferred/Critical Edges vs Matrix `Depends on`):

| ID | Matrix `Depends on` | Plan | Relation to Ticket 15 |
| --- | --- | --- | --- |
| `GAME-03` | `GAME-01`, `GAME-04`, `GAME-05` | Critical Edge `GAME-02` → `GAME-03`; R9 order is `GAME-01`+`04`+`05`, then `GAME-02`, then `GAME-03` | Ticket 15 L41 states that internal order. Matrix omits `GAME-02`. **Contradiction.** |
| `SGF-15` | `SGF-11`, `SGF-04` | `SGF-07`, `SGF-11`, `SGF-12` | Not unified |
| `SGF-16` | `SGF-01`, `SGF-14` | `SGF-11`, `SGF-14` | Not unified |
| `REVIEW-06` | `SGF-04`, `REVIEW-01` | `REVIEW-01`, `RULE-01` | Not unified |
| `ANA-06` | `ANA-01`, `ANA-03`, `ENG-02` | `ENG-05`, `ANA-01`, `ANA-03` | Complementary pieces on each side |
| `ANA-07` | `ANA-02`, `ANA-03` | `ANA-02`, `APP-02` | Plan vs matrix disagree on intake vs lanes |
| `ANA-09` | `ENG-09`, `ANA-04` | `ENG-09` | Ticket 11 names both adapter fixtures and analysis-model mapping |
| `PROV-05` | `SGF-07`, `APP-03` | `SGF-07`; not `PROV-04` | Complementary |
| `GAME-09` | `GAME-01` | domain-06 supported service | Plan points at the 13th gap; Matrix does not |

Phase **Owns** lists otherwise match Ticket 15 and Matrix Phase Map (including `APP-01` split and `GUIDE-01` blocked on producer).

### C6. Remaining in-scope fog

**Fail** (the 13 records). Additional non-blocking fog is recorded.

- The 13 owner-route/conflict records are in-scope fog by Ticket 15 L47 and Inventory L407–425.
- Inventory contains **79** `需定向运行核验` notes. The file states those runtime checks were not executed (L20). Map “Not yet specified” still allows prototype tickets only when a disposition depends on unobservable runtime. No additional disposition beyond the 13 is identified as blocked solely by an unrun check.
- `GUIDE-01` cannot be Accepted until a producer is chosen (`docs/PARITY_MATRIX.md` L23–24, L67–68; Plan R7 exit). That is gap 6, not a second class of fog.

## Failure evidence (the 13 Ticket 15 records)

Census pointers match across Inventory L411–425, Matrix L49–63, and Plan L310–324.

| # | Gap (destination wording) | Census pointer | Inventory line | What *is* mapped | What has no Parity ID | Forbidden silent absorption |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | Next-move marker | `SET-NEXT-MOVE` | L122 | none | none/simple/info marker + min playouts | `ANA-04` `UI-01` `PREF-01` |
| 2 | Same next-move cluster | `SGF-03-ADJ-NEXT-HINT` | L176 | none | SGF next-move hint | same |
| 3 | Graph perspective/lines/blunder-bar | `SET-WINRATE-GRAPH` | L123 | chart *presence* stays `UI-01` | perspective, WR/score lines, blunder bar | `ANA-04` `UI-01` |
| 4 | Mini-board *mode* | `SET-SUBBOARD` | L124 | presence `UI-01`; PV mini-board `ANA-04` | variation/raw/heatmap mode | `ANA-04` `UI-01` |
| 5 | Main-panel extras | `SET-MAIN-PANEL` extras | L125 | hints/coords/move numbers have other rows | large-sub / large-WR / append-WR / names-on-board / always-on-top | `UI-01` |
| 6 | `GUIDE-01` producer conflict | `SET-HINT-AUTOANALYZE` vs `CAP-04-ANA-08` | L130, L209 | `GUIDE-01` Reset Guidance from `SET-RESET-HINTS` (L99); `CAP-04-ANA-08` Abandoned (L209) | replacement educational producer | binding `GUIDE-01` to abandoned auto-analyze; substituting an unconfirmed tip |
| 7 | Provider-network proxy | `SET-NETWORK-PROXY` (06 edge) | L113 | update consumer `REL-03` OS proxy | application provider proxy | treating OS update proxy as provider proxy |
| 8 | SSH/remote compute | `CAP-04-ENG-01` SSH/remote remainder | L192 | `ENG-01` `ENG-09` `ENG-06` `ENG-08` `GAME-04` | SSH/remote compute | inventing a 06 ID |
| 9 | Contribute-service half | `GM-CONTRIBUTE` domain-06 service half | L232 | `GAME-09` board occupancy / match exclusion | 06 service, credentials, privacy, network | `GAME-09` |
| 10 | Readboard GMA | `CAP-06-READBOARD` GMA remainder | L249 | `READ-01` `READ-02` `READ-03` | GMA | `READ-02` |
| 11 | Personal/auth Yike reads | `CAP-06-YIKE-LIVE-CENTER` personal/auth | L244 | public `recommend`/`local` → `PROV-01`/`PROV-03` | personal/private/auth-required Next reads | `PROV-01` `PROV-03` |
| 12 | Analysis-panel images | `SGF-03-ADJ-SAVE-MORE` analysis-panel images | L164 | `EXPORT-01` branch SGF; `EXPORT-02` review-board image; Swing raw abandoned | analysis/sub-board/win-rate image export | `EXPORT-02` |
| 13 | Autoplay remainder | `SGF-03-ADJ-AUTOPLAY` domain-04 remainder | L175 | `UI-05` fixed toggle; `REVIEW-05` main-board interval | candidate-variation / sub-board / engine-best replay | `UI-05` `REVIEW-05` |

These are the same 13 phrases Ticket 15 L47 listed (next-move marker and related SGF hint, winrate-graph controls, sub-board mode, main-panel extras, `GUIDE-01` producer, provider proxy, SSH/remote compute, contribute-service, readboard GMA, personal/auth Yike reads, analysis-panel image export, domain-04 autoplay remainder).

## Proposed decision tickets

Do not create these files here. Parent owns create-then-wire. Each is `wayfinder:grilling` because map L18 requires live user confirmation for Next redesign, defer, or abandon.

Records 1–2 are one cluster (Inventory L122 and L176; Matrix L51–52).

### 1. Choose Disposition for the Next-Move Marker Cluster

- **Type:** `wayfinder:grilling`
- **Blocked by:** 16
- **Affects:** `SET-NEXT-MOVE`, `SGF-03-ADJ-NEXT-HINT`; must not expand `ANA-04` `UI-01` `PREF-01`
- **## Question**

> Should the frozen Java next-move marker (none/simple/info plus min playouts) and the adjacent SGF next-move hint become a new Parity Item (and if so with what ID, defaults, persistence, failure/recovery, evidence, remaining gap, acceptance, and phase), a Deferred item, or an Abandoned/Swing-only exclusion? Tickets 09–11 routed both census rows to domain 04 and named no ID. Do not expand accepted `ANA-04`, `UI-01`, or `PREF-01`.

- **Why a live product-scope decision is required:** Redesign/defer/abandon cannot be inferred from static routing. Ticket 15 forbade inventing an ID.

### 2. Choose Disposition for Winrate-Graph Controls

- **Type:** `wayfinder:grilling`
- **Blocked by:** 16
- **Affects:** `SET-WINRATE-GRAPH`; chart presence remains `UI-01`
- **## Question**

> Should Java winrate-graph perspective, WR/score lines, and blunder-bar controls become a new Parity Item, a Deferred item, or an exclusion? Chart *presence* in accepted `UI-01` is not this Capability. Tickets 09–14 named no ID.

- **Why a live product-scope decision is required:** Same map rule; presence vs controls is a product cut.

### 3. Choose Disposition for Sub-Board Mode

- **Type:** `wayfinder:grilling`
- **Blocked by:** 16
- **Affects:** `SET-SUBBOARD`; not mini-board presence (`UI-01`) or PV mini-board (`ANA-04`)
- **## Question**

> Should sub-board *mode* (variation / raw / heatmap) become a new Parity Item, a Deferred item, or an exclusion? Mini-board presence and PV mini-board already have owners.

- **Why a live product-scope decision is required:** Mode preference is a distinct user-reachable Capability with no 08–14 ID.

### 4. Choose Disposition for Main-Panel Extras

- **Type:** `wayfinder:grilling`
- **Blocked by:** 16
- **Affects:** `SET-MAIN-PANEL` extras (large-sub, large-WR, append-WR, names-on-board, always-on-top)
- **## Question**

> Should the Java main-panel extras (large sub-board, large winrate graph, append winrate to comment, names on board, always-on-top) become one or more successor Parity Items, Deferred items, or exclusions? Ticket 10 mapped related hints/coords/move numbers only. Do not expand accepted `UI-01` or personal-comment `SGF-05`.

- **Why a live product-scope decision is required:** Several extras, one census row; grouping vs abandon is a product cut.

### 5. Choose the GUIDE-01 Producer After Auto-Analyze Abandonment

- **Type:** `wayfinder:grilling`
- **Blocked by:** 16
- **Blocks:** `GUIDE-01` acceptance and R7 guidance exit
- **Affects:** `SET-HINT-AUTOANALYZE`, `CAP-04-ANA-08` (Abandoned), `GUIDE-01`, `SET-RESET-HINTS`
- **## Question**

> Ticket 09 routed educational-tip persistence and Reset Guidance to `GUIDE-01` and named domain-04 auto-analyze education (`SET-HINT-AUTOANALYZE`) as a required producer. Ticket 11 Abandoned `CAP-04-ANA-08` automatic current-game analysis, so that producer cannot satisfy `GUIDE-01`. What replacement producer, if any, is in scope, or should `GUIDE-01` be deferred, narrowed to Reset Guidance with no auto-analyze example, or otherwise re-dispositioned? Do not silently substitute an unconfirmed tip or bind `GUIDE-01` to the abandoned path.

- **Why a live product-scope decision is required:** Two resolved tickets conflict. Only the user can replace or drop the producer. Matrix L67–68 forbids this audit from substituting one.

### 6. Choose Disposition for Provider-Network Proxy

- **Type:** `wayfinder:grilling`
- **Blocked by:** 16
- **Affects:** `SET-NETWORK-PROXY` 06 edge; update edge already `REL-03`
- **## Question**

> Ticket 14 mapped update traffic to the OS/system proxy with no application-specific update proxy. Ticket 13 did not decide provider-network proxy. Should Next provider HTTP/WebSocket traffic use the OS proxy only, a separate application proxy item, a Deferred item, or an Abandoned Java proxy UI?

- **Why a live product-scope decision is required:** The 06 remainder is a network/privacy product cut, not implied by `REL-03`.

### 7. Choose Disposition for SSH/Remote Compute

- **Type:** `wayfinder:grilling`
- **Blocked by:** 16
- **Affects:** `CAP-04-ENG-01` SSH/remote remainder; not `ENG-01`/`ENG-09` local catalog
- **## Question**

> Ticket 11 routed SSH/remote compute to domain 06. Ticket 13 created no remote-compute item. Should remote/SSH engine compute become a new Parity Item, a Deferred item, or an Abandoned Java-only path?

- **Why a live product-scope decision is required:** Owner-route without an ID is not a disposition.

### 8. Choose the Contribute-Service Owner and Parity Item

- **Type:** `wayfinder:grilling`
- **Blocked by:** 16
- **Affects:** `GM-CONTRIBUTE` 06 half; `GAME-09` remains Deferred board-occupancy mapping
- **## Question**

> Ticket 12 assigned `GAME-09` board occupancy and match exclusion, and assigned domain 06 the contribute service, credentials, privacy, and network progress/failure. Ticket 13 named no contribute-service Parity ID. What is the 06 item (or exclusion/deferral) for that service half, and may `GAME-09` be scheduled only after it exists?

- **Why a live product-scope decision is required:** Scheduling `GAME-09` without a service owner would invent 06 coverage. Plan Deferred admission already requires a supported 06 service.

### 9. Choose Disposition for Readboard GMA

- **Type:** `wayfinder:grilling`
- **Blocked by:** 16
- **Affects:** `CAP-06-READBOARD` GMA remainder; not `READ-01`/`READ-02`/`READ-03`
- **## Question**

> Ticket 13 deferred GMA to domains 04/05 without an ID. Should GMA become a named Parity Item (analysis or match), a Deferred item with a stable ID, or an exclusion from Next readboard?

- **Why a live product-scope decision is required:** One-way sidecar sync (`READ-02`) is not GMA. Deferral without an ID is not a closed Deferred queue member.

### 10. Choose Disposition for Personal and Auth-Required Yike Reads

- **Type:** `wayfinder:grilling`
- **Blocked by:** 16
- **Affects:** `CAP-06-YIKE-LIVE-CENTER` personal/auth; public modes stay `PROV-01`/`PROV-03`
- **## Question**

> Ticket 13 kept public `recommend`/`local` (and Play & Sync) on `PROV-01`/`PROV-03` and left personal/private/auth-required Next read modes outside those items without a successor ID. Should those modes become a new item, a Deferred item with a stable ID, or an Abandoned/out-of-scope exclusion?

- **Why a live product-scope decision is required:** Auth/privacy integration is a product cut. Ticket 13 deferred without an ID.

### 11. Choose Dispositions for Sub-Board and Winrate-Chart Image Exports

- **Type:** `wayfinder:grilling`
- **Blocked by:** 16
- **Affects:** `SGF-03-ADJ-SAVE-MORE` sub-board and winrate-chart image outputs; `EXPORT-02` stays review-board only
- **## Question**

> Ticket 10 deferred review-board image export as `EXPORT-02` and left sub-board and winrate-chart image outputs without stable dispositions. Should either output become a domain-04 item, join the `EXPORT-*` family as Deferred, or be abandoned?

- **Why a live product-scope decision is required:** `EXPORT-02` owns only the main-board review view; the other two baseline outputs still need independent dispositions.

### 12. Choose Disposition for Domain-04 Autoplay Remainder

- **Type:** `wayfinder:grilling`
- **Blocked by:** 16
- **Affects:** `SGF-03-ADJ-AUTOPLAY` remainder; `UI-05` toggle and `REVIEW-05` interval stay as named
- **## Question**

> Ticket 10 kept the claimed `UI-05` fixed-interval main-board toggle, deferred configurable main-board interval as `REVIEW-05`, and routed candidate-variation, sub-board, and engine-best-move replay to domain 04 without an ID. Should that remainder become a domain-04 item, a Deferred item with a stable ID, or an exclusion?

- **Why a live product-scope decision is required:** Two named items do not cover the remainder. Ticket 10 left it owner-routed.

### 13. Reconcile Parity Matrix and Migration Plan Start Dependencies

- **Type:** `wayfinder:grilling`
- **Blocked by:** 16
- **Affects:** `GAME-03` start; Deferred `SGF-15` `SGF-16` `REVIEW-06` `ANA-06` `ANA-07` `ANA-09` `PROV-05` `GAME-09`
- **## Question**

> After Ticket 15, which document is canonical for item `Depends on` when `PARITY_MATRIX.md` and `MIGRATION_PLAN.md` disagree? In particular: must `GAME-03` wait for `GAME-02` as Ticket 15 and the Plan critical edge state, or may PK start from `GAME-01`/`GAME-04`/`GAME-05` only as the Matrix row states? Which of the Deferred Depends-on sets (see C5 table) is the admission gate?

- **Why a live product-scope / planning decision is required:** Start gates decide when work may begin. The two sources of truth currently give different gates. This audit cannot silently pick one.

## What is not a failure

- R0–R2 historical acceptance and IDs.
- Replacing Missing `GAME-01`–`GAME-03` identities (Ticket 12).
- `APP-01` split semantic-gate vs R11 association (Ticket 15; Plan L238–258).
- `GUIDE-01` remaining Missing while the producer is open (the item ID is stable; gap 6 is the producer).
- Maintainer `REL-01` not appearing as a `REL-C*` Capability.
- Successor `ANA-09` with no census heading.
- 79 unrun `需定向运行核验` notes, except as recorded non-blocking fog.
- Architecture/current-game evidence in `ARCHITECTURE_NEXT.md` lagging the rebuilt R3+ topology: that file is not the item-status owner.

## Final answer to Ticket 16

The Inventory, Parity Matrix, and Migration Plan **do not** yet form a closed trace from every frozen-baseline user-reachable behavior and Entry Point to exactly one supported Parity Item or explicit exclusion. They **do** preserve original Accepted IDs/scopes/status/evidence, state owner-split contracts, and avoid unsupported completion claims. Counts 139 / 95 / 19 / 13 are confirmed. The destination remains blocked on the 13 Ticket 15 records (proposed tickets 1–12) and on Matrix/Plan start-dependency authority (proposed ticket 13).
