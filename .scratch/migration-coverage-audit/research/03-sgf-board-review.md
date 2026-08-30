# Frozen 03 — SGF / Board / No-Engine Review Capability Census

**Java unique source:** `/home/dev/dev/weiqi/worktrees/lizzieyzy-next/java-baseline-v1`  
**Java commit (HEAD):** `7b4027531c2b26062d0bfc27a040cc550cfbea4d`  
**Next unique source:** `/home/dev/dev/weiqi/worktrees/lizzieyzy-next-tauri/migration-coverage-audit`  
**Next commit:** `18c6d189b8b01069975c4c40ead63a010249cb8c`

Not used: `/home/dev/dev/weiqi/lizzieyzy-next`, `/mnt/d/dev/weiqi/tmp/lizzie-java-baseline-r2a`, GitHub compare, or commit `42c92e3`. Old census text is not evidence. Runtime-only behavior is marked **需定向运行核验**.

Domain: 03 owns File/SGF/board/review Entry Points, review pass-as-edit, personal-vs-generated presentation, and candidate hover eviction as review presentation. 04 owns engine-generated analysis. 05 owns match-session pass. 06 is cross-referenced only where a provider/live import replaces the current game.

---

## 1. Census method

Static source walk of frozen Java user-reachable File/SGF/board/review surfaces, then mapping onto frozen Next current-game/review code and `docs/PARITY_MATRIX.md` R1/R2 items.

**Java generation mechanisms / files**

| Mechanism | Key files |
| --- | --- |
| File menu | `src/main/java/featurecat/lizzie/gui/Menu.java` (fileMenu 181–454) |
| Edit / Game / View review items | same `Menu.java` (view 479+, game 2900+, edit 3553+) |
| Keyboard / pointer | `Input.java`; board click/hover in `LizzieFrame.java` |
| Open/save/load/resume | `LizzieFrame.openFile` / `loadFile` / `saveOriFile` / `saveFile` / `resumeFile` |
| SGF / GIB | `rules/SGFParser.java`, `rules/GIBParser.java` |
| Board / tree | `rules/Board.java`, `VariationTree.java` |
| Comments | `LizzieFrame.setCommentEditable`, `teacher/TeacherCommentCodec.java`, `teacher/CommentDisplayRenderer.java` |
| Hover eviction | `SuggestionHoverIntent.java`, `analysis/AnalysisCandidateValidator.java` |
| Right-click | `RightClickMenu.java`, `RightClickMenu2.java` |
| Temp slots | `LizzieFrame.showTempGamePanel` / `addTempGameOne`, `TempGameData.java` |
| Strings | `src/main/resources/l10n/DisplayStrings*.properties` |
| Semantic tests | `SGFParserSemanticRoundTripTest.java`, `GIBParserTest.java`, `SGFParserEmptyEngineSaveTest.java` |

**Next generation mechanisms / files**

| Mechanism | Key files |
| --- | --- |
| Current game holder | `apps/desktop/src-tauri/src/current_game_state.rs` |
| SGF mutate/serialize | `crates/sgf/src/current_game.rs`, `crates/sgf/src/lib.rs` |
| Go rules | `crates/go-core/src/lib.rs` |
| Open/save API | `apps/desktop/src/api/backend.ts` |
| UI + shortcuts | `apps/desktop/src/App.tsx`, `components/AppChrome.tsx` |
| Board / hover | `components/BoardCanvas.tsx` |
| Review publication | `domain/reviewPresentation.ts` |
| Comments | `components/AnalysisPanel.tsx` |
| Fixtures / tests | `tests/golden/*.sgf`, `crates/sgf/tests/editable_workspace_roundtrip.rs`, `App.test.tsx` |

Open-file filter: Java AWT `*.sgf;*.gib;*.SGF;*.GIB` (`LizzieFrame.chooseKifuFilesWithAwt` 4490–498); Linux Swing `sgf,gib` (4501–509). Next native open: `sgf,txt` only (`backend.ts` 26).

---

## 2. Original Accepted R1/R2 scope (do not expand)

These capabilities already have Accepted (or Partial UI-02) Parity Items. Adjacent extras below are successor candidates, not silent expansions.

### SGF-03-RT — Semantic SGF tree round-trip

| Field | Content |
| --- | --- |
| Name | Parse → edit → serialize → reparse preserves tree meaning |
| Entry Points | File Open / Save / Save As / clipboard copy serialize / Next parse-textarea / sample load (same tree, not extra formats) |
| Frozen baseline | `SGFParser.load` / `loadFromString` / `save` / `saveToString`; semantic tests cover mainline order/color, sibling variations, AB/AW/PL setup, comments, pass |
| Defaults | FF[4]-family SGF; board size from `SZ` (default 19 in parser); `loadSgfLast` walks to last move after load |
| Persistence | Written SGF file / clipboard string; not byte-identical |
| Failure / recovery | Unreadable/empty file: `SgfObservation` + `false`; malformed Next: `MalformedSgf` and holder unchanged |
| Java evidence | `SGFParser.java` `load` 51–112, `loadFromString` 114–142; `SGFParserSemanticRoundTripTest.java` 28–80; `SGFParserEmptyEngineSaveTest.java` 14–51 |
| Next mapping | `CurrentSgfDocument::open/serialize/play/set_personal_comment/remove_variation`; fixture `tests/golden/editable-workspace-branching.sgf`; `editable_workspace_roundtrip.rs` 7–79 |
| Parity Item | **SGF-01 Accepted** |
| Adjacent / 核验 | Interactive setup-mode editing, GIB, raw-save stripping, unknown-property set beyond frozen R1 fixtures → successor, not SGF-01 expansion |

### SGF-03-DTO — Tree-shaped wire DTO and NodePath

| Field | Content |
| --- | --- |
| Name | Complete tree + `NodePath` as location |
| Entry Points | N/A (wire contract consumed by all current-game UI) |
| Frozen baseline | Java history is `BoardHistoryNode` tree; Next maps that observable tree to DTO |
| Defaults | Root `[]`; default selected path is first-child mainline to leaf (`current_game.rs` `default_selected_path` 35–45) |
| Persistence | N/A |
| Failure / recovery | Invalid path → `InvalidNodePath`; no document → `NoCurrentGame` |
| Java evidence | `BoardHistoryList` / `BoardHistoryNode` (tree children/variations); semantic test sibling structure `SGFParserSemanticRoundTripTest` 43–62 |
| Next mapping | `CurrentGameResultDto` in `current_game_state.rs` 23–117; `SgfTreeNodeDto` via `CurrentSgfDocument::tree` |
| Parity Item | **SGF-02 Accepted** |
| Adjacent | N/A |

### SGF-03-NAV — Variation navigation

| Field | Content |
| --- | --- |
| Name | Parent / child / sibling / first / last / page jumps; board+comment follow selected node |
| Entry Points | **Java:** arrows / PageUp/Down / Home / End (`Input.java` 326–410, 560–581); Edit menu jump items (`Menu.java` 3842–880); variation tree (`VariationTree.java`); wheel (`Input.mouseWheelMoved` 847+). **Next:** arrows / Home / End / Page* (`App.tsx` 230–276); Edit “跳转到最前”; `selectCurrentGameNode` |
| Frozen baseline | `LizzieFrame.undo/redo/firstMove/lastMove/moveToMainTrunk`; `Input.nextBranch/previousBranch`; `Board.nextMove/previousMove/nextBranch/previousBranch`. Match play blocks undo/redo. |
| Defaults | Next remembers chosen child index per parent (`chosenChildren`) so Next-child follows last taken sibling — Ticket 08 Case 2 |
| Persistence | N/A (session cursor). Java `loadSgfLast` persists *whether* to land on last move (`ui` config; used at `SGFParser` 89–91, `GIBParser` 117–119) |
| Failure / recovery | Illegal path rejected; Next select does not bump generation (`current_game_state.rs` 326–349) |
| Java evidence | `Input.java` 268–282, 326–410, 560–581; `LizzieFrame.java` 18882–19019 |
| Next mapping | `App.tsx` `selectNode` 909–935, `handleParent/NextChild/PrevSibling/NextSibling` 948–965 |
| Parity Item | **SGF-03 Accepted** |
| Adjacent | Click-on-tree, winrate-graph click, blunder-list click, try-play, comment-node jumps (Ctrl+Shift+Up/Down) → successor |

### SGF-03-EDIT — Legal move / pass-as-edit / variation removal

| Field | Content |
| --- | --- |
| Name | Play a legal point or **review pass** as a child; identical existing child is selected; illegal play is atomic no-op; remove selected non-root variation |
| Entry Points | **Java pass-as-edit (03, not 05):** Game “停一手(P)” `Menu.java` 3141–3149 `Lizzie.board.pass()`; `Input` VK_P 431–437 (if Human SL active → **05** `humanSlGame.humanPass()`). Board left-click `Input.mousePressed` → `onClicked`. Shift+Delete / Edit “删除分支” `deleteBranch`. **Next:** board pointer (`BoardCanvas.onPointClick` → `playAt`); toolbar 虛手; Game/Edit 停一手; `P`; Shift+Delete `handleRemoveVariation`. |
| Frozen baseline | Pass is a tree child (SGF `B[]`/`W[]`). Existing identical child selected without new node (Next `existing_child_index` 230–253). Occupied/suicide/simple-ko reject without mutation. Root cannot be removed. |
| Defaults | Color = selected node’s player-to-play |
| Persistence | Dirty tree until Save (Next `dirty`+`generation`; Java in-memory board until `SGFParser.save`) |
| Failure / recovery | Next typed errors Occupied/Suicide/SimpleKo; UI `role=status` (`App.tsx` 994–997, 1314–316). Java drag-to-occupied aborts (`LizzieFrame` 10318–322). |
| Java evidence | `Menu.java` 3141–3149; `Input.java` 431–437, 678–689; GIB `SKI` → `pass()` `GIBParser.java` 102–105 |
| Next mapping | `CurrentSgfDocument::play` 70–104; `CurrentGameHolder::play` 171–188; `remove_variation` 115–129 / 232–247; `App.tsx` `playAt` 967–1001, `handleRemoveVariation` 1003–1020 |
| Parity Item | **SGF-04 Accepted** (pass insertion in frozen fixtures); **UI-05** claims `P` and Shift+Delete |
| Adjacent | Delete-*one-move* (Delete without Shift), insert-black/white, setup-mode, stone drag, set-as-main → successor. **05** owns match-session pass. |

### SGF-03-CMT — Personal comments vs generated information

| Field | Content |
| --- | --- |
| Name | Edit/clear personal comment on selected node; generated info does not overwrite personal field |
| Entry Points | **Java:** comment pane `setCommentEditable` writes `BoardData.comment` (`LizzieFrame.java` 9890–9916); display via `CommentDisplayRenderer`. **Next:** AnalysisPanel personal textarea blur → `set_current_game_personal_comment`. |
| Frozen baseline | SGF `C` is the personal field. Teacher AI blocks live in the same `C` string behind markers but `removeBlocks` keeps user text (`TeacherCommentCodec`). `splitWinrateComment` separates match-info when `separateMatchInfo` (`CommentDisplayRenderer` 15–24). Next snapshot has `personal_comment` and `generated_information: None` (`current_game.rs` 47–55). |
| Defaults | Empty comment |
| Persistence | `C` in serialized SGF; Next dirty/generation bump only if serialize changed (`current_game_state.rs` 206–229) |
| Failure / recovery | Next no-op if text unchanged (`App.tsx` 877); empty trim removes `C` (`apply_personal_comment` 335+) |
| Java evidence | `TeacherCommentCodec.java` 7–75; `CommentDisplayRenderer.java` 11–49; `LizzieFrame.setCommentEditable` 9890–9916 |
| Next mapping | `set_personal_comment` 106–113; `personal_comment()` 328–333; `AnalysisPanel.tsx` 200–214; `App.tsx` 161–162, 871–907 |
| Parity Item | **SGF-05 Accepted** (“generated information does not overwrite or serialize into the personal-comment field”) |
| Adjacent | Teacher marker blocks, match-info divider, HTML comment rendering, engine-game generated text (05/GAME-03) → successor presentation items. Do not reopen SGF-05. |

### SGF-03-SAVE — Save / Save As current tree

| Field | Content |
| --- | --- |
| Name | Serialize current edited tree to the current path or a chosen path; reopen reproduces position/variations/setup/metadata/comments |
| Entry Points | **Java:** File Save → `saveOriFile`; Save As → `saveFile(false)`; Ctrl+S / S / Ctrl+Shift+S / Ctrl+Alt+S (`Input` 506–529). **Next:** 保存 / 另存为; Ctrl+S (no-op when clean); S Save As; toolbar save. |
| Frozen baseline | `saveOriFile`: if `curFile` exists and is not `.gib`, confirm replace (`show-replace-file-hint`, default on) then `SGFParser.save`; else Save As. Save As: last-folder chooser, `.sgf` suffix, exists confirm, write failure message. Cancel = dialog not APPROVE. No dirty flag. `.gib` current file cannot overwrite as GIB — falls through to SGF Save As. Empty-engine save still writes `AP[LizzieYzy Next]` (`SGFParserEmptyEngineSaveTest`). |
| Defaults | Timestamp-ish filename from GameInfo or `yyyyMMddHHmmss`; last-folder in persisted `filesystem` |
| Persistence | File + `filesystem.last-folder`; Next also `native_path`, `dirty=false` on success, generation **not** incremented |
| Failure / recovery | Java: `LizzieFrame.saveFileFailed` toast; Next: directory-as-file leaves path/dirty; Save As cancel `null`; denied/redirected write failure does not adopt path (`ARCHITECTURE_NEXT.md` / `save-as-dialog`) |
| Java evidence | `LizzieFrame.java` 4042–4250, 4256–4330; `Menu.java` 225–262 |
| Next mapping | `backend.ts` `saveCurrentGame` 159–172; `current_game_state.rs` `save_to_path` 50–82; `App.tsx` `handleSaveSgfDocument` 601–624 |
| Parity Item | **SGF-06 Accepted** |
| Adjacent | Raw save, raw+comment, current-branch flatten save, image exports → successor |

### SGF-03-RULE — Go rules used by SGF editing

| Field | Content |
| --- | --- |
| Name | Occupied, capture, suicide, pass, simple ko; rejected edit leaves tree unchanged |
| Entry Points | Same as SGF-03-EDIT |
| Frozen baseline | Java `Board.place/pass` + history; tests in `src/test/java/featurecat/lizzie/rules/*` |
| Defaults | Simple ko; suicide forbidden in Next `go-core` |
| Persistence | N/A |
| Failure / recovery | Atomic reject |
| Java evidence | `GIBParser.place`/`pass` 94–105; board tests listed under `src/test/java/featurecat/lizzie/rules/` |
| Next mapping | `go-core` `Board::play` 94–148; `RuleError` 47–58; mapped in `current_game.rs` `rule_error` 299–310 |
| Parity Item | **RULE-01 Accepted** for frozen R1 fixtures |
| Adjacent | Japanese/Chinese/Tromp-Taylor engine-rule UI (`SetKataRules`, `LizzieFrame` 8001–033) is **04/02**, not RULE-01 |

### SGF-03-CHROME — No-engine review layout

| Field | Content |
| --- | --- |
| Name | Workbench: board, analysis/comment rail, win-rate, candidates, mini-board |
| Entry Points | Main window after open |
| Frozen baseline | Java main panel + comment sidebar + variation graph + winrate + sub-board (visibility is **02**) |
| Defaults | Next fixed 228px/260px rails (`PARITY_MATRIX` UI-01) |
| Persistence | Layout → **02** |
| Failure / recovery | N/A |
| Java evidence | `LizzieFrame` layout ~6000+ `showComment` / `showVariationGraph` / `showSubBoard` |
| Next mapping | `App.tsx` shell 1222–1330; `DESIGN.md` cited by UI-01 |
| Parity Item | **UI-01 Accepted** |
| Adjacent | Resizable splitters → LAYOUT-01 (02/R5) |

### SGF-03-BOARD-INTENT — Non-blocking board interaction

| Field | Content |
| --- | --- |
| Name | Pointer/keyboard play intent updates selected node without blocking render |
| Entry Points | Board click; board-focused arrows+Enter/Space (`BoardCanvas.tsx` 144–170); outside-board arrows navigate |
| Frozen baseline | `Input.mousePressed` 28–80 routes setup / score / drag / comment / click; hover arms `SuggestionHoverIntent` |
| Defaults | N/A |
| Persistence | N/A |
| Failure / recovery | Occupied: Next status text, position unchanged |
| Java evidence | `Input.java` 19–80 |
| Next mapping | `BoardCanvas.tsx` 86–170; `App.tsx` `playAt` 967–1001 |
| Parity Item | **UI-02 Partial** — remaining gap is engine-event delivery while a mutation promise is pending (**04**, not 03 expansion) |

### SGF-03-HOVER — Candidate hover preview + stale eviction (review presentation)

| Field | Content |
| --- | --- |
| Name | Hover previews a candidate without committing; node/job/generation change drops stale hover and published review |
| Entry Points | Pointer over candidate on main board; leave/click/scope change cancels |
| Frozen baseline | Java `SuggestionHoverIntent` default **200 ms** (`SuggestionHoverIntent.java` 8–39); `armSuggestionHoverPreview` from `LizzieFrame` ~8814; `AnalysisCandidateValidator` drops occupied-point candidates (`AnalysisCandidateValidator.java` 10–65); `cancelPendingSuggestionHoverPreview` on click (`Input` 29) |
| Defaults | Java 200 ms; optional extra delay via `SetDelayShowCandidates` (`delay-show-candidates`, `delay-candidates-seconds`) |
| Persistence | Delay settings in `uiConfig` |
| Failure / recovery | Scope mismatch refuses publish (`reviewPresentation.ts` 11–19; `BoardCanvas` 114–122) |
| Java evidence | `SuggestionHoverIntent.java` 1–65; `AnalysisCandidateValidator.java` 10–65; `LizzieFrame.java` 8810–849, 12844 |
| Next mapping | `BoardCanvas.tsx` 74–127 (**120 ms**); `reviewPresentation.ts` 1–19; `App.tsx` `previewCandidate` 1196–1213, `visibleFrames` gated by `shouldPublishReviewPresentation` 171–183 |
| Parity Item | **UI-03 Accepted** at the **frozen 120 ms Next contract**. Java 200 ms + delay-dialog is **not** UI-03 expansion → successor if product wants Java delay |

### SGF-03-NOENGINE — Desktop usable without engine

| Field | Content |
| --- | --- |
| Name | Open, inspect, edit, save, exit without engine profile/process |
| Entry Points | All File/SGF/board/review controls above; Next chip `未加载引擎` |
| Frozen baseline | `SGFParserEmptyEngineSaveTest` save with `leelaz=null`; load/save paths do not require engine |
| Defaults | Next `engineLabel` default `未加载引擎` (`App.tsx` 96) |
| Persistence | N/A |
| Failure / recovery | Engine-only actions unavailable/explanatory |
| Java evidence | `SGFParserEmptyEngineSaveTest.java` 14–37 |
| Next mapping | `AppChrome` native vs `尚未接入`; UI-04 Ticket 08–09 |
| Parity Item | **UI-04 Accepted** |

### SGF-03-ACTIONS — Claimed R2 review actions / shortcuts

Frozen **UI-05** inventory (Ticket 07): candidate keys 1–9, `C`/`M`/`H`/`T`, arrows/Home/End/Page*, Ctrl+A autoplay, Ctrl+C copy SGF, Ctrl+S save (no-op if clean), `O`/`S` native dialogs, `P` pass, Shift+Delete remove variation, `N` new, Ctrl+Home new/clear, 载入示例. Unwired Java rows stay visible-disabled `title=尚未接入`.

| Field | Content |
| --- | --- |
| Name | One visible control + intended shortcut for the **claimed** R2 set |
| Entry Points | `App.tsx` 215–332; `AppChrome.tsx` File/View/Game/Edit/toolbar |
| Frozen baseline | Java has a much larger shortcut map (`Input.keyPressed` 312–823) |
| Defaults | See UI-05 |
| Persistence | Coordinate/move-number toggles are session in Next (not preferences) — **02** if they should persist |
| Failure / recovery | `shouldIgnoreApplicationShortcut` skips when typing in fields |
| Java evidence | `Input.java` 312–823 |
| Next mapping | `App.tsx` 215–332; `AppChrome.tsx` 98–270 |
| Parity Item | **UI-05 Accepted** for claimed rows only |
| Adjacent | Every disabled `尚未接入` File/Edit/Game row below |

---

## 3. Adjacent capabilities (successor candidates; not Accepted-scope expansion)

### SGF-03-ADJ-OPEN — Open SGF and replace current game

| Field | Content |
| --- | --- |
| Name | Choose a kifu file and install it as the current game |
| Entry Points | Java: File 打开, `O`, toolbar (Menu/toolbar), `loadFile` after chooser; Next: File 打开棋谱(O), toolbar 打开, `openSgfDocument` |
| Frozen baseline | Pauses ponder; `chooseKifuFiles`; `loadFile(file[0])`; sets `curFile`; resumes ponder. **No dirty-discard confirm on Open** (unlike paste). Rejects load during engine/human match (`loadFile` 4975–978). Overlay progress `beginKifuLoad`. After success: recent list, last-folder, title, delayed movelist refresh, optional analysis resume (**04**). |
| Defaults | last-folder from persisted filesystem |
| Persistence | `filesystem.last-folder`; recent paths via `saveRecentFilePaths` |
| Failure / recovery | IO / parse false → restore sound flag + later “open failed” message |
| Java evidence | `Menu.java` 196–205; `Input.java` 532–541; `LizzieFrame.java` 4332–4365, 4475–521, 4974–5030 |
| Next mapping | `backend.ts` 95–105, 110–115; `App.tsx` `handleOpenSgfDocument` 571–598 (dirty confirm **is** present) |
| Parity Item | Covered operationally by **SGF-01/06 + UI-04/05** for SGF open/save. **Dirty-confirm-on-open** and **in-game refuse** are adjacent. |
| 岐义 | Whether Next should refuse open during 05 sessions; whether Java-style no-confirm open is required |

### SGF-03-ADJ-GIB — Open GIB as current game

| Field | Content |
| --- | --- |
| Name | Load `.gib` (Tygem) into the current board |
| Entry Points | Same Open chooser (`*.gib`); `loadFile` extension branch |
| Frozen baseline | Encoding detect; parse `GAMEINFOMAIN/GONGJE`, names, `INI` handicap (max 9; 5/7 place tengen extra), `STO` moves, `SKI` pass; `readKomi` applies komi; rewind then optional `loadSgfLast`; **cannot Save over GIB** (`saveOriFile` 4115) |
| Defaults | Names “Player 1/2”, komi 7.5 if not parsed |
| Persistence | In-memory game; save as SGF |
| Failure / recovery | Missing/unreadable/empty → false; test: empty/missing must not clear a seeded board (`GIBParserTest.java` 19–22) |
| Java evidence | `GIBParser.java` 1–124; `LizzieFrame.loadFile` 4994–496; `GIBParserTest.java` 1–80 |
| Next mapping | **Missing.** Open filters `sgf,txt` only |
| Parity Item | None. Successor. |
| 岐义 | 需定向运行核验 handicap 5/7 placement and CRLF name stripping vs live Tygem files |

### SGF-03-ADJ-RECENT — Open recent file

| Field | Content |
| --- | --- |
| Name | Re-open a recently used kifu |
| Entry Points | File → 最近打开 (`Menu.openRecent` 207–210, `updateRecentFileMenu`) |
| Frozen baseline | Populated on successful non-temp `loadFile` (`saveRecentFilePaths`) |
| Defaults | 需定向运行核验 list length / persist key |
| Persistence | Config recent paths |
| Failure / recovery | Missing file: 需定向运行核验 |
| Java evidence | `Menu.java` 169, 207–210; `LizzieFrame.loadFile` 5005–5008 |
| Next mapping | `AppChrome.tsx` 101 `最近打开` disabled `尚未接入` |
| Parity Item | None (UI-05 explicitly unclaimed) |

### SGF-03-ADJ-CLIP — Clipboard SGF copy / paste replacing current game

| Field | Content |
| --- | --- |
| Name | Copy current tree as SGF text; paste SGF text as current game |
| Entry Points | File copy/paste items; Ctrl+C / Ctrl+V (`Input` 544–550, 643–650); Next same |
| Frozen baseline | Copy: `SGFParser.saveToString(false)` → clipboard (`copySgf` 9736–745). Paste: clipboard string → `isSGF` check → if board not empty, confirm replace → `loadSgfString` (`pasteSgf` 9748–773, `pasteSgfDecision` 9775–782). Empty / not-SGF ignored. |
| Defaults | N/A |
| Persistence | N/A |
| Failure / recovery | Copy exception logged; paste ignore or confirm-cancel leaves game |
| Java evidence | `LizzieFrame.java` 9736–801; `Menu.java` 371–396 |
| Next mapping | `handleCopySgf` 843–851; `handlePasteSgf` 853–868 (always `applyReplacement` after non-empty text; **no `isSGF` gate** in sampled code) |
| Parity Item | **Ctrl+C is in UI-05 claimed set.** Paste exists in Next but was **not** in Ticket 07 exercised list; Java isSGF/confirm-if-stones is extra → successor, do not widen UI-05 |

### SGF-03-ADJ-SAVE-MORE — Extra save / export flavors

| Field | Content |
| --- | --- |
| Name | Save raw SGF, raw+comments, current branch flattened, board/sub-board/winrate images |
| Entry Points | File → 更多保存 (`Menu.java` 247–325); shortcuts Ctrl+Shift+S, Ctrl+Alt+S, Alt+S, Shift+S, Shift+Alt+S (`Input` 506–529) |
| Frozen baseline | `saveFile(true)` / `isSavingRaw`; `saveRawFileComment` sets `isSavingRawComment`; `saveCurrentBranch` flattens via `setMoveListWithFlatten` then raw save; images via `saveMainBoardPicture` / `saveSubBoardPicture` / `saveImage` PNG/JPG/GIF/BMP, exists confirm, `last-image-folder` |
| Defaults | Same timestamp naming as Save As |
| Persistence | last-image-folder |
| Failure / recovery | Write/format unsupported dialogs |
| Java evidence | `Menu.java` 247–325; `LizzieFrame.java` 4042–4112, 4256–4330, 10652–10755 |
| Next mapping | `AppChrome.tsx` 106–112 all disabled `尚未接入`. Authoritative Save remains SGF-06 only |
| Parity Item | None |

### SGF-03-ADJ-TEMP — Temp slots, autosave-on-exit, resume-previous

| Field | Content |
| --- | --- |
| Name | Named slot save/load with thumbnail; optional resume at startup; autosave on exit |
| Entry Points | File → 存档与读档 → `showTempGamePanel`; checkboxes in that panel |
| Frozen baseline | Files `save/gameN.sgf` + `.bmp`, `save/autoGameN.sgf`; load then `playList` or `goToMoveNumber`. `auto-save-exit`, `resume-previous-game`. File menu **Resume item is commented out** (`Menu.java` 412–425); `resumeFile` still loads autoGame1 then autoGame2 (4523–534). |
| Defaults | checkboxes from `Lizzie.config.autoSaveOnExit` / `autoResume` |
| Persistence | `uiConfig` keys `auto-save-exit`, `resume-previous-game`; slot files under `save/` |
| Failure / recovery | Missing bmp/sgf: stack trace / skip; 需定向运行核验 startup resume wiring |
| Java evidence | `Menu.java` 327–339, 412–425; `LizzieFrame.java` 4523–4534, 13608–13670, 14000–14034; `TempGameData.java` |
| Next mapping | `AppChrome.tsx` 114 `存档与读档` disabled. No slot/resume files |
| Parity Item | None. **01** may own startup resume as an Entry Point; 03 owns the kifu payload |

### SGF-03-ADJ-NEW — New empty board / new game document

| Field | Content |
| --- | --- |
| Name | Replace current game with an empty document |
| Entry Points | Java File 新建棋盘 `newEmptyBoard` (`Menu.java` 186–194); Edit 清空棋盘 Ctrl+Home `Board.clear(false)` (3809–3817; `Input` 565–569). Game 新对局 is **05**. Next: File 新建, `N`, Ctrl+Home, 清空棋盘 → `handleNewGame` empty SGF |
| Frozen baseline | Ctrl+Home clears stones in place (`Board.clear`). File new-board method body not sampled in `LizzieFrame` slices — **需定向运行核验** whether it is clear-in-place vs new document + path reset |
| Defaults | Next empty `(;GM[1]FF[4]SZ[19]KM[7.5]PB[黑]PW[白])` (`App.tsx` 46, 834–840) |
| Persistence | Next dirty confirm; native_path cleared |
| Failure / recovery | Confirm cancel leaves game |
| Java evidence | `Menu.java` 186–194, 3809–3817; `Input.java` 565–569 |
| Next mapping | `handleNewGame` 834–840; UI-05 claims `N` and Ctrl+Home |
| Adjacent | Distinguishing Java 清空 vs 新建棋盘 |

### SGF-03-ADJ-KOMI — Load komi from kifu

| Field | Content |
| --- | --- |
| Name | Toggle applying SGF/GIB komi into the current game |
| Entry Points | File checkbox `自动加载棋谱中的贴目` (`Menu.java` 398–410) |
| Frozen baseline | `readKomi` persisted `read-komi`. Even when off, setup/handicap games still apply parsed root KM (`SGFParser.applySgfKomiForSetupGameWhenReadKomiDisabled` 150–158). GIB applies komi only if `readKomi` (107–109). |
| Defaults | 需定向运行核验 Config default |
| Persistence | `uiConfig.read-komi` |
| Failure / recovery | N/A |
| Java evidence | `Menu.java` 398–410, 459–464; `SGFParser.java` 150–174; `GIBParser.java` 107–109 |
| Next mapping | Open always takes KM from SGF (`KM[7.5]` empty game). No toggle |
| Parity Item | None. Komi *value* in frozen fixtures is inside SGF-01 |

### SGF-03-ADJ-SETUP — Interactive starting-position setup

| Field | Content |
| --- | --- |
| Name | Setup mode: place/erase black/white, side-to-play, clear all, convert current position to setup |
| Entry Points | Game → 起始局面设置 (`Menu.java` 3153–3254); board clicks in setup (`Input` 46–62) |
| Frozen baseline | AB/AW/AE/PL already **parse** in SGF-01 tests; this is the **interactive** tool |
| Defaults | Tools black/white/erase |
| Persistence | Setup stones in tree |
| Failure / recovery | 需定向运行核验 |
| Java evidence | `Menu.java` 3153–3254; `Input.java` 46–62 |
| Next mapping | `AppChrome.tsx` 149 `起始局面设置` disabled. Parse of AB/AW/PL remains SGF-01 |
| Parity Item | Successor; do not expand SGF-01/04 |

### SGF-03-ADJ-INSERT — Add/insert colored stones, drag, double-click find, click-review

| Field | Content |
| --- | --- |
| Name | Force black/white/alternating add; insert-into-list; drag a stone; double-click search; click-to-review |
| Entry Points | Edit menu (`Menu.java` 3705–3781); right-click add black/white (`RightClickMenu`); toolbar icons; `allowDrag` / `allowDoubleClick` / `enableClickReview` |
| Frozen baseline | `blackorwhite` 0/1/2; `Input.insert` 0/1/2; drag rewrite `DraggedReleased` 10300–3400 |
| Defaults | allow-drag / allow-double-click / enable-click-review in uiConfig |
| Persistence | those booleans |
| Failure / recovery | Drag onto occupied: abort |
| Java evidence | `Menu.java` 3705–3781; `RightClickMenu.java` 25–27, 133–138; `LizzieFrame.java` 10300–3400 |
| Next mapping | Add black/white/交替 / 标记 / 设为主分支 toolbar rows disabled (`AppChrome.tsx` 176–262) |
| Parity Item | None |

### SGF-03-ADJ-DELETE-MOVE — Delete one move vs undo/redo delete

| Field | Content |
| --- | --- |
| Name | Delete current move; undo/redo that edit |
| Entry Points | Edit 删除一手; Delete/Backspace (`Input` 678–689); right-click `deleteone` (`RightClickMenu2`); undo/redo delete (`Menu.java` 3904–3926) |
| Frozen baseline | `Board.deleteMove` vs `deleteBranch` (branch = SGF-04/UI-05). `cleanedit` / `reedit` restore `boardstatbeforeedit` |
| Defaults | N/A |
| Persistence | In-tree |
| Failure / recovery | 需定向运行核验 empty-mainline delete |
| Java evidence | `Menu.java` 3884–3926; `Input.java` 294–299, 678–689; `RightClickMenu2.java` 23–24 |
| Next mapping | Only non-root **variation** removal. No one-move delete, no edit-undo stack |
| Parity Item | Successor. Shift+Delete stays UI-05 |

### SGF-03-ADJ-MAIN — Set as main / return to main trunk

| Field | Content |
| --- | --- |
| Name | Promote current variation to main; walk back to main trunk |
| Entry Points | Edit 设为主分支(L) / 返回主分支(B); `L` / `B` (`Input` 423–428, 663–676); toolbar |
| Frozen baseline | `setAsMain`, `moveToMainTrunk` (18984–19019) |
| Defaults | N/A |
| Persistence | Tree child order |
| Failure / recovery | Already-main is no-op |
| Java evidence | `Menu.java` 3820–3838; `Input.java` 423–428, 663–676; `LizzieFrame.java` 18984–19019 |
| Next mapping | Disabled `尚未接入` (`AppChrome.tsx` 182–183, 256–257) |
| Parity Item | None. Navigation *along* mainline is SGF-03 |

### SGF-03-ADJ-XFORM — Color swap / rotate / mirror

| Field | Content |
| --- | --- |
| Name | Transform whole tree geometry/colors |
| Entry Points | Edit exchange/spin/mirror (`Menu.java` 3953–3997); Ctrl+Shift+Alt+Right, Ctrl+Alt+arrows (`Input` 330–371, 398–402) |
| Frozen baseline | `Board.exchangeBlackWhite`, `SpinAndMirror(1..4)` |
| Defaults | N/A |
| Persistence | Mutates tree |
| Failure / recovery | 需定向运行核验 |
| Java evidence | `Menu.java` 3953–3997; `Input.java` 330–402 |
| Next mapping | Disabled (`AppChrome.tsx` 189–193) |
| Parity Item | None |

### SGF-03-ADJ-META — Game info + board size

| Field | Content |
| --- | --- |
| Name | Edit PB/PW/KM; set SZ |
| Entry Points | Edit 编辑棋局信息(I) / 设置棋盘大小(Ctrl+I); `I` / Ctrl+I |
| Frozen baseline | `GameInfoDialog` applies names+komi and sends engine `komi` (150–159). Handicap field **read-only**. Whole-game analysis blocks edit (`editGameInfo` 4012–4016). |
| Defaults | Current GameInfo |
| Persistence | Root SGF properties on save |
| Failure / recovery | Analysis conflict message |
| Java evidence | `LizzieFrame.editGameInfo` 4012–4023; `GameInfoDialog.java` 20–159; `Menu.java` 3930–3949 |
| Next mapping | Disabled (`AppChrome.tsx` 153–154). KM/PB/PW **round-trip** is already SGF-01 |
| Parity Item | Successor for the **dialog/workflow**, not for property round-trip |

### SGF-03-ADJ-MARKUP — SGF marks (LB/CR/SQ/MA/TR)

| Field | Content |
| --- | --- |
| Name | Place/remove list-property marks on points |
| Entry Points | Markup tools; `tryToMarkup` / `tryToRemoveMarkup` (`LizzieFrame.java` 14419–14478) |
| Frozen baseline | Types 1–6 letters/circle/X/square/triangle/number; stored as SGF list props |
| Defaults | Markup off |
| Persistence | Node properties; SGF-01 already preserves unknown/list props in frozen set |
| Failure / recovery | Click off-board: false |
| Java evidence | `LizzieFrame.java` 14419–14478; `SGFParser` `listProps` 41–43 |
| Next mapping | Toolbar 标记工具 disabled |
| Parity Item | Interactive markup successor; parse preservation stays SGF-01 |

### SGF-03-ADJ-AUTOPLAY — Configurable autoplay

| Field | Content |
| --- | --- |
| Name | Timed main-board / sub-board / variation replay |
| Entry Points | View 自动播放(Ctrl+A) → `AutoPlay` dialog; Next Ctrl+A toggles 800 ms interval (`App.tsx` 309–371) |
| Frozen baseline | Main-board seconds; sub-board milliseconds; candidate-variation replay interval and optional entire-variation-first pause; continue/directly-with-engine-best-move controls (`AutoPlay.java` 44–149) |
| Defaults | Toolbar main/sub values; persisted branch-preview and engine-continuation defaults |
| Persistence | `auto-replay-display-entire-variations-first`, `display-entire-variations-first-seconds`, `replay-branch-interval-seconds`, `continue-with-best-move`, `directly-with-best-move`, and `auto-replay-branch` are written on confirm (`AutoPlay.java` 180–206) |
| Failure / recovery | Main-board Next autoplay stops at a leaf. Java candidate-variation replay stops when disabled and restarts its delay when hover selection changes (`LizzieFrame.autoReplayBranch` 10171–10223). |
| Java evidence | `Menu.java` 578–587; `Input.java` 710–713; `AutoPlay.java` 44–206; `LizzieFrame.java` 10171–10223; `BottomToolbar.java` 4397–4423 |
| Next mapping | Main-board toggle only, 800 ms hardcoded; no dialog |
| Parity Item | **UI-05 claims the main-board Ctrl+A toggle.** Configurable main-board interval is an adjacent review successor. Candidate-variation, sub-board, and engine-best-move replay are domain 04 analysis presentation. |

### SGF-03-ADJ-NEXT-HINT — Next-move hints on board

| Field | Content |
| --- | --- |
| Name | Show next-move marks: none / simple / with blunder info |
| Entry Points | View 下一手 (`Menu.java` 574–598, 2556–568); `J` toggles (`Input` 460–462) |
| Frozen baseline | `setShowNextMoves(false,false)` etc. |
| Defaults | 需定向运行核验 |
| Persistence | config showNextMoves / showNextMoveBlunder |
| Failure / recovery | N/A |
| Java evidence | `Menu.java` 574–598, 2556–568 |
| Next mapping | Not present as a distinct next-move overlay (candidates are 04/UI-03) |
| Parity Item | None |

### SGF-03-ADJ-TREE-CLICK — Variation tree / lists as navigation

| Field | Content |
| --- | --- |
| Name | Click variation graph, winrate graph, or blunder/suggestion table to change node |
| Entry Points | `VariationTree` clickPoint; blunder table mouseClicked → `goToMoveNumber` (`LizzieFrame.java` 1431–1448) |
| Frozen baseline | Tree draw + hit testing (`VariationTree.java` 11–80+) |
| Defaults | N/A |
| Persistence | N/A |
| Failure / recovery | N/A |
| Java evidence | `VariationTree.java`; `LizzieFrame.java` 12008–12300, 1431–1448 |
| Next mapping | AnalysisPanel move list buttons (`AnalysisPanel.tsx` 64–79) are a **redesigned** tree, not Java graph. Winrate click-nav not sampled as node jump |
| Parity Item | SGF-03 covers keyboard/parent-child. Click-graph successor |

### SGF-03-ADJ-TRYPLAY / SCORE / LADDER

| Key | Name | Java Entry | Next | Notes |
| --- | --- | --- | --- | --- |
| TRYPLAY | Try-play overlay | `V` without Ctrl (`Input` 554–557) `tryPlay(false)` | Missing | Review sandbox; not 05 |
| SCORE | Score mode | Ctrl+Q (`Input` 704); board left-click in score (`Input` 64–67); score-dialog rule toggle and Confirm Result | Missing | Dead-group marking; area/territory totals include komi; area is default and the selected rule persists; confirmation writes root `RE` |
| LADDER | Auto-continue ladder | Game 继续征子 (`Menu.java` 3273–3287) | Missing | Fails if shorter than `MINIMUM_LADDER_LENGTH_FOR_AUTO_CONTINUATION` |

#### Census correction for `SGF-03-ADJ-SCORE`

Score mode is one Capability with two user-selectable calculations and an explicit result commit, not dead-stone marking alone. `Board.showGroupResult` derives living stones, surrounded points, captures, and root komi. `ScoreResult.setScore` uses area scoring by default (`blackAlive + blackPoint` versus `whiteAlive + whitePoint + komi`) or territory scoring when `use-territory-in-score` is enabled; the score dialog toggles the rule and writes that boolean to `uiConfig`. Dead-group marks and preview totals are transient. Confirm Result calls `GameInfo.setResult`, and SGF serialization writes that value as root `RE`. Frozen evidence: `Board.java` 6540–6574 and 6577–6673; `ScoreResult.java` 178–242, 262–343, and 384–387; `SGFParser.java` 1192–1211 and 1288–1311; `Config.java` 1237 and 2098. Next has no scoring mode, dead-group interaction, rule selection, result confirmation, or persisted score-rule preference. This correction does not change an Accepted item; it completes the one adjacent `SGF-03-ADJ-SCORE` successor surface.

### SGF-03-ADJ-URL — Open online link replacing current game

| Field | Content |
| --- | --- |
| Name | Paste/open a URL and load SGF into the current game |
| Entry Points | File 打开在线链接(Q); `Q` (`Input` 701–707); `OnlineDialog.java` |
| Frozen baseline | Replaces current game via SGF parse after fetch |
| Next mapping | `AppChrome.tsx` 102 disabled. Live Fox/Yike → **06**. 03 only records that success **replaces** current game (`App.tsx` `handleProviderImport` 812–823) |
| Parity Item | Cross-ref **06 / PROV-***; current-game install = 03 |

### SGF-03-ADJ-PROVIDER — Provider/readboard import as current-game replacement

Owned by **06**. 03 fact: Next `handleProviderImport` calls `replaceCurrentGame` / `applyReplacement` with dirty confirm (`App.tsx` 812–823). Java Yike/Fox/readboard loaders also end in `loadFile` / `loadSgfString*` (`LizzieFrame` 4650–4856). Do not duplicate 06 capabilities.

### SGF-03-ADJ-HOVER-DELAY — Java 200 ms hover vs claimed 120 ms

UI-03 Accepted is 120 ms Next. Java `DEFAULT_DELAY_MS = 200` plus `SetDelayShowCandidates`. Successor only if product wants Java delay/manual-F reveal.

### SGF-03-ADJ-COMMENT-LAYERS — Teacher / match-info / engine-game text in the comment pane

SGF-05 Accepted: personal `C` vs `generated_information` field. Java also: Teacher markers in `C`, `splitWinrateComment` match-info HTML, engine-game generated comments (**05/GAME-03**). Successor presentation items; do not reopen SGF-05.

---

## 4. Unreachable or implementation-only (not dispositions)

| Candidate | Why listed | Not a user Capability |
| --- | --- | --- |
| File menu Resume item commented out (`Menu.java` 412–425) | UI entry gone; `resumeFile` still callable from temp-panel auto-resume | Implementation leftover |
| `KifuLoadFinisher` 120 ms paint delay | Load overlay timing | Widget |
| `isSavingRaw` / `isSavingRawComment` flags | Control what `SGFParser.save` writes | Implementation |
| `NodePath` / `generation` / `dirty` | Wire/state | Not user goals |
| `SgfObservation` logging | Diagnostics | Not user-facing |
| `FileFilterTest1/2` accept-all | Unused/odd filters; real chooser uses extension filters | Swing leftover |
| `chooseKifuFiles` AWT vs Swing Linux split | Platform widget | Same Capability |
| Browser preview current-game throw | Explains non-authoritative preview (`backend.ts` 107–108) | Runtime caveat, UI-04 |
| `fakeAnalyze` / cache restore | 04 analysis presentation; eviction uses 03 scope | Boundary only |

No abandon/defer decisions.

---

## 5. Cross-domain Entry Points

| Entry | Owner | 03 role |
| --- | --- | --- |
| Startup / OS file association / CLI path | **01** | If it loads a kifu, current-game replace is 03; wiring 需定向运行核验 (not sampled in `Lizzie.java` slices) |
| Exit / force-exit | **01** | File menu items 429–454; not SGF |
| Window/layout/theme/coords persistence | **02** | View coords/move-numbers as *review presentation* are 03/UI-05; persist → 02 |
| Engine analysis, flash, whole-game, heatmap, cache, teacher LLM generate | **04** | 03 owns hover *eviction* and comment *field* separation |
| VK_P during Human SL / engine game pass | **05** | Review `Lizzie.board.pass()` is 03 |
| New genmove/analyze/engine/HumanSL game | **05** | Game menu 2905–2949 |
| Fox/Yike/readboard/Tencent fetch | **06** | Success replaces current game (03) |
| Share SGF / upload | **06/07** as publishing | Copy-to-clipboard kifu is 03 |
| Install/update | **07** | N/A |
| In-game open refuse | 03×05 | `loadFile` 4975–978 |

---

## 6. Accepted vs uncovered (explicit)

**Keep Accepted as-is:** SGF-01..06, RULE-01, UI-01, UI-03, UI-04, UI-05 (claimed rows), UI-02 Partial only for engine-event-during-mutation (04).

**Do not silently add to those items:** GIB; recent files; URL open; raw/branch/image save; temp slots/resume; paste isSGF gating; setup-mode UI; insert/drag/markup; delete-one-move; set-as-main; transforms; game-info dialog; autoplay settings beyond toggle; next-move hints; tree-click; try-play; score mode including persisted area/territory selection; ladder; Java 200 ms hover; teacher/match-info comment layers; dirty-confirm-on-open vs Java no-confirm.

**Already implemented in Next but outside frozen acceptance text (still successor if claiming parity):** clipboard paste (`handlePasteSgf`); file `<input>` import (`handleImportFile` 801–810); txt in open filter; dirty confirm on Open/New/Paste/Sample; 800 ms autoplay interval.

---

## 7. Corrections vs invalid first-round reports

- Evidence is only `java-baseline-v1` @ `7b4027531c2b26062d0bfc27a040cc550cfbea4d` and `migration-coverage-audit` @ `18c6d189b8b01069975c4c40ead63a010249cb8c`.
- **No** citation of `42c92e3`, dirty `/mnt/d/dev/weiqi/tmp/lizzie-java-baseline-r2a`, or `/home/dev/dev/weiqi/lizzieyzy-next`.
- Pass on `P` is classified as **review pass-as-edit (03)** except Human SL / match session (**05**).
- Hover eviction is **review presentation (03 / UI-03)**; engine job lifecycle remains **04 / R3**.
- Personal vs generated: SGF-05 field split is Accepted; Java teacher markers + match-info HTML are adjacent, not a silent SGF-05 reopen.
- UI-05 disabled `尚未接入` rows are inventoried as adjacent, not treated as Accepted.
- GIB, extra saves, and temp resume are first-class adjacent capabilities, not “just File menu widgets.”
