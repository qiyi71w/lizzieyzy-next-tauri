# 03 — File / Edit / Game / View Entry Point Proof

Independent Domain 03 entry-point census for Ticket 03 (SGF / board / review). Parent synthesizes destination docs. This file proves registered vs constructed-but-unadded controls and lists remainder candidates. It does not rewrite `docs/JAVA_CAPABILITY_INVENTORY.md`.

## Sources

| Tree | Path | Commit |
| --- | --- | --- |
| Java Migration Baseline v1 | `/home/dev/dev/weiqi/worktrees/lizzieyzy-next/java-baseline-v1` | `7b4027531c2b26062d0bfc27a040cc550cfbea4d` (worktree HEAD file; `gitdir` → `lizzieyzy-next/.git/worktrees/java-baseline-v1`) |
| Completeness index | `docs/JAVA_CAPABILITY_INVENTORY.md` lines 440–444 | Compared **after** independent reconstruction |

Read-only Java source census. No production edits, tests, builds, or runtime launches. Next docs/parity are product-decision sources, not a stop condition for counting. An Entry Point exists only when a control is constructed **and** added, shown, or registered (`Input.keyPressed`, `buttonPane.add`, `topPanel.add`, `this.add`). Provider/URL fetch and engine-produced analysis are routes, not Domain 03 Capabilities.

Key Java files: `gui/Menu.java` constructor (`fileMenu` 181–454, `viewMenu` 479+, `gameMenu` 2900+, `editMenu` 3553+), `gui/RightClickMenu.java`, `gui/RightClickMenu2.java`, `gui/Input.java` `keyPressed` 312–823, `gui/BottomToolbar.java` constructor 600+, `gui/LizzieFrame.java` board/comment/markup.

## Method

1. Walk every `fileMenu.add`, `editMenu.add`, `gameMenu.add`, and relevant `viewMenu.add` (coords, move numbers, autoplay, next-move) in `Menu()`.
2. Walk `RightClickMenu` / `RightClickMenu2` `this.add`, BottomToolbar `buttonPane.add`, and `Menu.doubleMenu` top-strip icons that invoke load/save/new/nav/edit.
3. For each candidate: constructed? added/shown? shortcut in `Input.keyPressed`? Then classify Entry Point or non-row.
4. Independently bucket remainder user goals, then compare to the 35 Domain 03 headings.

---

## 1. Registered Domain 03 Entry Point inventory

Control → handler → Capability bucket. Shortcut column is `Input.keyPressed` unless noted.

### 1.1 File menu (`fileMenu.add`, `Menu.java` 181–454)

| Control | Added | Handler | Shortcut | Bucket |
| --- | --- | --- | --- | --- |
| `newBoard` | `fileMenu.add` 194 | `Lizzie.frame.newEmptyBoard()` | none (Java `N` is **05** genmove) | `SGF-03-ADJ-NEW` |
| `open` | `fileMenu.add` 205 | `Lizzie.frame.openFile()` | `O` 532–541 | `SGF-03-ADJ-OPEN` (`SGF-03-ADJ-GIB` same chooser) |
| `openRecent` | `fileMenu.add` 208; items via `updateRecentFileMenu` 6721–6744 | `Lizzie.frame.loadFile(recentF, true, false)` | none | `SGF-03-ADJ-RECENT` |
| `openUrl` | `fileMenu.add` 222 | `Lizzie.frame.openOnlineDialog()` | `Q` 701–707 | Route **06**; current-game replace is `SGF-03-ADJ-URL` |
| `save` | `fileMenu.add` 234 | `Lizzie.frame.saveOriFile()` | `Ctrl+S` 524–526 | `SGF-03-SAVE` |
| `saveAs` | `fileMenu.add` 245 | `LizzieFrame.saveFile(false)` | `S` 527 | `SGF-03-SAVE` |
| `saveMore` / `saveRaw` | `fileMenu.add` 249; `saveMore.add` 262 | `LizzieFrame.saveFile(true)` | `Ctrl+Shift+S` 507–508 | `SGF-03-ADJ-SAVE-MORE` |
| `saveCommentRaw` | `saveMore.add` 274 | `Lizzie.frame.saveRawFileComment()` | none | `SGF-03-ADJ-SAVE-MORE` |
| `saveBranchRaw` | `saveMore.add` 286 | `LizzieFrame.saveCurrentBranch()` | `Ctrl+Alt+S` 509–510 | `SGF-03-ADJ-SAVE-MORE` |
| `saveMainBoardScreen` | `saveMore.add` 297 | `saveMainBoardPicture()` | `Alt+S` 521–522 | `SGF-03-ADJ-SAVE-MORE` |
| `saveSubBoardScreen` | `saveMore.add` 308 | `saveSubBoardPicture()` | `Shift+S` 518–519 | `SGF-03-ADJ-SAVE-MORE` |
| `saveWinrate` | `saveMore.add` 325 | `saveImage(statx,staty,…)` | `Shift+Alt+S` 511–517 | `SGF-03-ADJ-SAVE-MORE` |
| `saveAndLoad` | `fileMenu.add` 339 | `showTempGamePanel()` | none | `SGF-03-ADJ-TEMP` |
| `copyBoardScreen` | `fileMenu.add` 346 | `saveMainBoardToClipboard()` | `Shift+C` 646–647 | `SGF-03-ADJ-SAVE-MORE` (clipboard image) |
| `copySubBoardScreen` | `fileMenu.add` 360 | `copySubBoard()` | `Alt+C` 644–645 | `SGF-03-ADJ-SAVE-MORE` |
| `copySgf` | `fileMenu.add` 374 | `copySgf()` | `Ctrl+C` 648–649 | `SGF-03-ADJ-CLIP` / `SGF-03-ACTIONS` |
| `pasteSgf` | `fileMenu.add` 387 | `pasteSgf()` | `Ctrl+V` 546–550 | `SGF-03-ADJ-CLIP` |
| `loadKomi` checkbox | `fileMenu.add` 410; state sync 463–464 | `Lizzie.config.readKomi` + `uiConfig.read-komi` | none | `SGF-03-ADJ-KOMI` |
| `forceExit` | `fileMenu.add` 441 | `System.exit(0)` | none | **01** `SHELL-09`, not 03 |
| `exit` | `fileMenu.add` 454 | `Lizzie.shutdown()` | window close | **01** `SHELL-08`; autosave payload is `SGF-03-ADJ-TEMP` |

### 1.2 Edit menu (`editMenu.add`, `Menu.java` 3553–4041)

| Control | Added | Handler | Shortcut | Bucket |
| --- | --- | --- | --- | --- |
| `addBlack` / `addWhite` / `alternatelyMoves` | 3717 / 3732 / 3747 | `Input.insert=0`; `blackorwhite` 1/2/0 | none (top-strip icons) | `SGF-03-ADJ-INSERT` |
| `allowDoubleClick` | 3759 | `allow-double-click` | double-click `Input` 104–109 | `SGF-03-ADJ-INSERT` |
| `allowDrag` | 3770 | `allow-drag` | drag `Input` 147–187 | `SGF-03-ADJ-INSERT` |
| `allowClickReview` | 3781 | `enable-click-review` | RightClickMenu2 `review` | `SGF-03-ADJ-INSERT` |
| `insertBlack` / `insertWhite` | 3794 / 3805 | `Input.insert` 1/2 | left-click insert path 114–127 | `SGF-03-ADJ-INSERT` |
| `clearBoard` | 3818 | `Lizzie.board.clear(false)` | `Ctrl+Home` 565–569 | `SGF-03-ADJ-NEW` (in-place clear; File new-board is a different handler) |
| `backToMainBranch` | 3828 | `moveToMainTrunk()` | `B` 663–675 (not `T`) | `SGF-03-ADJ-MAIN` |
| `setAsMain` | 3832 | `setAsMain()` | `L` 423–428 | `SGF-03-ADJ-MAIN` |
| `jumpToFirst` / `jumpToLast` | 3850 / 3860 | `firstMove()` / `lastMove()` | `Home` 560–572 / `End` 575–581 | `SGF-03-NAV` |
| `jumpToLeft` / `jumpToRight` | 3870 / 3880 | `Input.previousBranch()` / `nextBranch()` | `Left` / `Right` 326–356 | `SGF-03-NAV` |
| `delete` | 3886 | `Lizzie.board.deleteMove()` | `Delete` / `Backspace` 678–688 | `SGF-03-ADJ-DELETE-MOVE` |
| `deleteBranch` | 3896 | `Lizzie.board.deleteBranch()` | `Shift+Delete` 685–686 | `SGF-03-EDIT` |
| `undoDelete` / `redoDelete` | 3905 / 3917; visibility 4025–4028 | `cleanedit()` / `reedit()` | none | `SGF-03-ADJ-DELETE-MOVE` |
| `setInfo` | 3938 | `LizzieFrame.editGameInfo()` | `I` 497–503 | `SGF-03-ADJ-META` |
| `setBoard` | 3949 | `new SetBoardSize().setVisible(true)` | `Ctrl+I` 500–502 | `SGF-03-ADJ-META` (default SZ persist is **02** `SET-BOARD-SIZE`) |
| `exchange` / `spinRight` / `spinLeft` / `mirrorVertical` / `mirrorHorizon` | 3955–3997 | `exchangeBlackWhite()` / `SpinAndMirror(1..4)` | `Ctrl+Shift+Alt+Right`; `Ctrl+Alt` arrows 330–401 | `SGF-03-ADJ-XFORM` |

Undo/redo-delete items stay added; `menuSelected` only hides them when `boardstatbeforeedit` / `boardstatafteredit` are empty.

### 1.3 Game menu Domain 03 rows (`gameMenu.add`, `Menu.java` 2900–3287)

| Control | Added | Handler | Shortcut | Bucket |
| --- | --- | --- | --- | --- |
| `scoreGame` | 3011 | `toggleScoreMode()` | `Ctrl+Q` 704 | `SGF-03-ADJ-TRYPLAY / SCORE / LADDER` (score) |
| `playPassMove` | 3150 | `Lizzie.board.pass()` | `P` 431–437 (Human SL → **05**) | `SGF-03-EDIT` |
| `startingPositionSetup` + tools | 3155–3254 | `toggleSetupMode` / `selectSetupTool` / `setupClearAllCommand` / `setupSetSideToPlayCommand` / `convertCurrentPositionToStartingPositionCommand` | board click in setup `Input` 46–62 | `SGF-03-ADJ-SETUP` |
| `continueLadder` | 3287 | `Lizzie.board.continueLadder()` | none | `SGF-03-ADJ-TRYPLAY / SCORE / LADDER` (ladder) |

All other `gameMenu.add` rows in this constructor (new genmove/analyze/engine/HumanSL, continue-vs-AI, AI time, break/pause engine game, intervention, play-best) are **05** or **04**. See §3.

### 1.4 View menu Domain 03-relevant rows (`viewMenu.add`)

| Control | Added | Handler | Shortcut | Bucket |
| --- | --- | --- | --- | --- |
| `coordsMenu` | `viewMenu.add` 529 | `Lizzie.config.toggleCoordinates()` | `C` 651 | **02** `SET-COORDS` (registered EP; not a Domain 03 Capability) |
| `moveMenu` (no/last-1/5/10/all/custom/from-one/branch) | `viewMenu.add` 563; children `moveMenu.add` 744–899 | `setMoveNumber` / `MovenumberDialog` / branch flags | `M` 450–457; `Ctrl+M` all-in-branch | **02** `SET-MOVE-NUMBERS` |
| `nextMoveHint` none/simple/info + min playouts | `viewMenu.add` 576; children 599–658 | `setShowNextMoves` / `minPlayoutsForNextMove` | `J` 460–462 | `SGF-03-ADJ-NEXT-HINT` (persist also **02** `SET-NEXT-MOVE`) |
| `makeAutoPlay` | `viewMenu.add` 587 | `new AutoPlay().setVisible(true)` | `Ctrl+A` 711–713 | `SGF-03-ADJ-AUTOPLAY` |
| `setCandidatesDelay` | `Suggestions.add` 1090 (`Suggestions` is `viewMenu.add` 572) | `openCandidatesDelaySettings` | none | `SGF-03-ADJ-HOVER-DELAY` (also **02** suggestion delay) |

Other View children (panel/toolbar/appearance/winrate graph/layout modes) are **02**. Analyze-suggestion content is **04**.

### 1.5 Right-click (`this.add`)

**Empty-point menu** `RightClickMenu.java` 267–287 — all listed items are added:

    | Control | Handler | Bucket |
    | --- | --- | --- |
    | `previousMove` | `undoForRightClick()` 317–322 | `SGF-03-NAV` |
    | `findMove` | `Lizzie.board.findMove(coords)` 387–392 | `SGF-03-ADJ-INSERT` |
    | `addblack` / `addwhite` | `insertMove(coords, true/false)` 395–413, 464–497 | `SGF-03-ADJ-INSERT` |
    | `reedit` / `cleanupedit` / `cleanedittemp` | `reedit` / `cleanedit` / `cleanedittemp` 340–368 | `SGF-03-ADJ-DELETE-MOVE` |
    | `allow` / `allow2` / `allow3` / `avoid` / `avoid2` / `cancelavoid` | `analyzeAvoid` / force coords 415–445 | **04** |
    | `priority` / `clearPriority` | KataGo `setmaxpolicy` / `clearpolicy` 297–315 | **04** |
    | `trackPoint` / `untrackPoint` / `clearAllTracked` | tracking points 447–449 | **04** |
    | `addSuggestionAsBranch` | `addSuggestionAsBranch()` 325–337 | **04** route (engine PV → tree); not an extra 03 Capability |

Popup `setVisible` toggles hide analysis/insert items during match play (115–184). Hidden-but-added remains an Entry Point with dynamic visibility.

**Occupied-stone menu** `RightClickMenu2.java` 110–117 — all added:

    | Control | Handler | Bucket |
    | --- | --- | --- |
    | `moveStone` | `setDragStartInfo(coords, true)` 150–156 | `SGF-03-ADJ-INSERT` |
    | `switchone` | `editMove(coords, true, false)` 170–172 | `SGF-03-ADJ-INSERT` |
    | `deleteone` | `editMove(coords, false, true)` 174–176 | `SGF-03-ADJ-DELETE-MOVE` |
    | `review` | `setPressStoneInfo(coords, true)` 159–166 | `SGF-03-ADJ-INSERT` (click-review) |
    | `previousMove` | `undoForRightClick()` 126–132 | `SGF-03-NAV` |
    | `findStone` | `findMove(coords)` 118–124 | `SGF-03-ADJ-INSERT` |

### 1.6 Bottom toolbar (`BottomToolbar` constructor 600–752)

`this.add(buttonPane)` 713; Domain 03 icons `buttonPane.add` 726–751.

| Control | Added | Handler | Bucket |
| --- | --- | --- | --- |
| `openfile` | 738 | `openFile()` 1347–1352 | `SGF-03-ADJ-OPEN` |
| `savefile` | 733 | `saveOriFile()` 1354–1359 | `SGF-03-SAVE` |
| `clearButton` | 728 | `newEmptyBoard()` 1361–1366 | `SGF-03-ADJ-NEW` (same handler as File new-board, not Edit `clear`) |
| `firstButton` / `lastButton` | 730 / 729 | `firstMove` / `lastMove` 1368–1384 | `SGF-03-NAV` |
| `backward1` / `forward1` / `backward10` / `forward10` | 736–732 | `previousMove` / `nextMove` x1 or x10 1266–1307 | `SGF-03-NAV` |
| `gotomove` + `txtMoveNumber` | 735 / 878 | `goToMoveNumberBeyondBranch` 880–894, 1309–1323 | `SGF-03-NAV` |
| `deleteMove` | 726 | `deleteMoveNoHint()` 1179–1183 | `SGF-03-ADJ-DELETE-MOVE` |
| `tryPlay` | 746 | `tryPlay(true)` 1193–1197 | `SGF-03-ADJ-TRYPLAY / SCORE / LADDER` |
| `backMain` / `setMain` | 742 / 743 | `moveToMainTrunk` / `setAsMain` 1228–1238 | `SGF-03-ADJ-MAIN` |
| `autoPlay` | 751 | `new AutoPlay().setVisible(true)` 1171–1176 | `SGF-03-ADJ-AUTOPLAY` |
| `finalScore` | 731 | `toggleScoreMode()` 1386–1391 | `SGF-03-ADJ-TRYPLAY / SCORE / LADDER` |
| `coords` / `move` | 750 / 749 | `toggleCoordinates` / `toggleShowMoveNumber` 1205–1217 | **02** |
| `analyse` / `heatMap` / `kataEstimate` / `batchOpen` / `analyzeList` / `badMoves` / `refresh` / `moveRank` | added | ponder / heatmap / estimate / AutoAnalyze / best-moves / hawk-eye / refresh / rank mark | **04** |
| `share` / `liveButton` / `remoteComputeButton` / `downloadWeightButton` | added | share popup / Yike popup / remote / weights | **06** |

Overflow `moreActionsButton` 752 re-invokes the same `doClick` handlers (525–546): still the same Entry Points.

### 1.7 Top strip (`Menu.doubleMenu`, shown when `showDoubleMenu`)

`doubleMenu` 7783+ rebuilds `topPanel`. Load/save/new/nav/edit icons are added only inside `if (Lizzie.config.showBasicBtn)` 7794–8200:

    | Control | Added | Handler | Bucket |
    | --- | --- | --- | --- |
    | `btnNewFile` | `leftArea.add` 8174 | `newEmptyBoard()` 7840–7844 | `SGF-03-ADJ-NEW` |
    | `btnOpen` | 8175 | `openFile()` 7851–7855 | `SGF-03-ADJ-OPEN` |
    | `btnSave` | 8176 | `saveOriFile()` 7862–7866 | `SGF-03-SAVE` |
    | `btnSetMain` / `btnBackMain` | `rightArea.add` 8186–8187 | `setAsMain` / `moveToMainTrunk` 7981–8002 | `SGF-03-ADJ-MAIN` |
    | `btnChangeTurn` | 8185 | `Lizzie.board.changeNextTurn()` 8005–8014 | `SGF-03-ADJ-SETUP` |
    | `btnMarkup` + types | 8189; types 8190–8198 when `isShowingMarkupTools` | `setMarkupType` / clear properties / `drawPainting` 8028–8170 | `SGF-03-ADJ-MARKUP` |
    | `black` / `white` / `blackwhite` / `playPass` | `topPanel.add` 8202–8207 | insert color / `board.pass()` 7084–7174 | `SGF-03-ADJ-INSERT` / `SGF-03-EDIT` |
    | `saveLoad` | `toolPanel.add` 9709; visible if `showSaveLoadMenu` 9732–9735 | `showTempGamePanel()` 6118–6124 | `SGF-03-ADJ-TEMP` |
    | `setBoardSize` button | `toolPanel.add` 9708; visible if `showGobanMenu` 9727–9730 | `SetBoardSize` 6108–6115 | `SGF-03-ADJ-META` |
    | `doubleMenuNewGame` | `topPanel.add` 8695 if `showDoubleMenuGameControl` | popup → `startNewGame` / analyze / engine game 8602–8644 | **05** |

`toggleShowEditbar` 10381–10387 only changes visibility of already-added color/pass buttons.

### 1.8 Pointer / wheel / comment (not menus, still registered)

| Control | Registered | Handler | Bucket |
| --- | --- | --- | --- |
| Board left click | `Input.mousePressed` 102–129 | `onClicked` / insert / `tryToMarkup` | `SGF-03-BOARD-INTENT` / `SGF-03-EDIT` / `SGF-03-ADJ-MARKUP` |
| Board right click | 131–135 | `openRightClickMenu` or `undoForRightClick` | `SGF-03-NAV` + menus above |
| Wheel | `mouseWheelMoved` 847–878 | undo/redo or `doBranch` | `SGF-03-NAV` |
| Comment pane | `LizzieFrame.setCommentEditable` (cited by inventory `SGF-03-CMT`) | writes `BoardData.comment` | `SGF-03-CMT` |
| Variation tree / blunder table click | `VariationTree.inNode`; blunder `goToMoveNumber` `LizzieFrame` 1431–1448 | node jump | `SGF-03-ADJ-TREE-CLICK` |
| Hover preview | `SuggestionHoverIntent` from board move | preview + eviction | `SGF-03-HOVER` |

### 1.9 Accepted buckets with no extra File/Edit/Game/View identity

These are Domain 03 Capabilities whose Entry Points are the same registered controls above, or non-menu contracts:

    | Heading | How it is reached from this census |
    | --- | --- |
    | `SGF-03-RT` | Open/Save/clipboard serialize through `SGFParser` |
    | `SGF-03-DTO` | No UI control (wire). Not an Entry Point. |
    | `SGF-03-NAV` | Edit jumps, arrows/Home/End/Page, toolbar nav, wheel, right-click previous |
    | `SGF-03-EDIT` | Board click, Game/toolbar pass, Shift+Delete |
    | `SGF-03-CMT` | Comment pane, not a File/Edit/Game item |
    | `SGF-03-SAVE` | File Save / Save As / Ctrl+S / S / toolbar save |
    | `SGF-03-RULE` | Same play/pass/setup rejects as EDIT |
    | `SGF-03-CHROME` | Main window after open; panel visibility is **02** |
    | `SGF-03-BOARD-INTENT` | `Input.mousePressed` |
    | `SGF-03-HOVER` | Pointer over candidates |
    | `SGF-03-NOENGINE` | Same File/board/review controls with engine absent |
    | `SGF-03-ACTIONS` | Claimed shortcut subset of `Input.keyPressed` 312–823 |

---

## 2. Constructed-not-added / commented (non-rows)

| Surface | Evidence | Why not an Entry Point |
| --- | --- | --- |
| File › Resume | `resume` constructed in comments; `fileMenu.add(resume)` commented `Menu.java` 412–425 | Handler `resumeFile()` is not reachable from File. Startup `autoResume` is **01** `SHELL-05`. Temp-panel load remains `SGF-03-ADJ-TEMP`. |
| Share menubar | `shareKifu` constructed 4043–4046; children added 4049–4126; `this.add(shareKifu)` commented 4047 | Menubar share is not an Entry Point. BottomToolbar `share` **is** added (`BottomToolbar.java` 724, 1333–1337) → **06**. |
| `this.add(black/white/blackwhite/playPass)` | commented 7105, 7129, 7154, 7175 | Not on the `JMenuBar`. They become Entry Points only via `topPanel.add` in `doubleMenu` 8202–8207. |
| `this.add(btnDoubleMenu)` | commented 7083 | Strip toggle lives on `topPanel` (`doubleMenu` 7791). Layout **02**. |
| `komiPanel.add(setRules/setLzSaiParam/setBoardSize/saveLoad)` | commented 6449–6452 | Those four are not on the komi row. `saveLoad` / `setBoardSize` reappear on `toolPanel` when `showSaveLoadMenu` / `showGobanMenu` (`9704–9736`). Rules/params are **04**. |
| `btnMarkupCircle` | constructed 8074–8087; **not** in `rightArea.add` 8190–8198 | Circle markup tool is constructed-not-added. Label/number/X/square/triangle/eraser/clear/paint **are** added when `isShowingMarkupTools`. Circle is not a Capability. |
| Right-click `insertmode` / `quitinsert` | commented fields `RightClickMenu.java` 221–222; `RightClickMenu2.java` 21 | Dead comments. Insert is Edit menu + `Input.insert`. |
| BottomToolbar `shareHistory` | commented 987–1021, 1048 | Local share-history file open is not an Entry Point. Remote share items are added → **06**. |
| Analyze `clearsave` / `clearthis` visibility comments | `Menu.java` 4000–4003 | Unrelated commented cache items; not File/Edit. |
| File `autoSave.setState` in `menuSelected` | commented 460–462 | No File autosave checkbox. Exit autosave is **01**. |

Hidden-but-added (match-play `setVisible(false)` on right-click insert items, `undoDelete` empty-stack hide, markup types behind `isShowingMarkupTools`, `saveLoad` behind `showSaveLoadMenu`) stay Entry Points with dynamic visibility. Runtime-check is not required: the add/show calls are in frozen source.

---

## 3. Cross-domain routes (must not become extra Domain 03 Capabilities)

| Surface | Owner | Why it looks like 03 | 03 role |
| --- | --- | --- | --- |
| File › Exit / Force Exit | **01** | On File menu | Shutdown / no persist. Autosave file is TEMP payload. |
| CLI `args[0]` open; board file-drop | **01** `SHELL-03` / `SHELL-12` | Loads a kifu | Parse/replace stays OPEN/GIB. |
| Startup `autoResume` | **01** `SHELL-05` | Reloads autosave SGF | File Resume is **not** an EP. |
| View coords / move numbers / panel / theme / toolbar pos | **02** | On View next to autoplay | Persist presentation. Next-move overlay remainder stays 03; the View toggle persist is `SET-NEXT-MOVE`. |
| Analyze menu; hawk-eye; flash; heatmap; policy; cache; Teacher generate; allow/avoid/track; play-best; `VK_COMMA` | **04** | Board/review chrome | Hover *eviction* and comment *field split* stay 03. |
| Game › 新对局 / 人机续弈 / AI time / break/pause PK / intervention | **05** | Game menu | Review pass `board.pass()` is 03. `P` during Human SL is 05. Java `N` is genmove, not File new-board. |
| File › 打开在线链接; Sync/Yike/Fox/Tencent/readboard; toolbar live/share | **06** | Replaces current game | Success install is OPEN-shaped; fetch is 06. `SGF-03-ADJ-URL` / `SGF-03-ADJ-PROVIDER` record the replace, not extra fetch Capabilities. |
| Help diagnostics / About / Check Update / clear personal data | **07** / **02** recents split | Recents wipe | `SET-CLEAR-PERSONAL-HISTORY` / `SGF-03-ADJ-RECENT` owner split. |

---

## 4. Extra / missing vs the 35-heading reference

Independent reconstruction from registered Entry Points, **then** compared to inventory lines 440–444.

**Reference 35 headings**

- Accepted (12): `SGF-03-RT` `SGF-03-DTO` `SGF-03-NAV` `SGF-03-EDIT` `SGF-03-CMT` `SGF-03-SAVE` `SGF-03-RULE` `SGF-03-CHROME` `SGF-03-BOARD-INTENT` `SGF-03-HOVER` `SGF-03-NOENGINE` `SGF-03-ACTIONS`
- Adjacent (23): `SGF-03-ADJ-OPEN` `SGF-03-ADJ-GIB` `SGF-03-ADJ-RECENT` `SGF-03-ADJ-CLIP` `SGF-03-ADJ-SAVE-MORE` `SGF-03-ADJ-TEMP` `SGF-03-ADJ-NEW` `SGF-03-ADJ-KOMI` `SGF-03-ADJ-SETUP` `SGF-03-ADJ-INSERT` `SGF-03-ADJ-DELETE-MOVE` `SGF-03-ADJ-MAIN` `SGF-03-ADJ-XFORM` `SGF-03-ADJ-META` `SGF-03-ADJ-MARKUP` `SGF-03-ADJ-AUTOPLAY` `SGF-03-ADJ-NEXT-HINT` `SGF-03-ADJ-TREE-CLICK` `SGF-03-ADJ-TRYPLAY / SCORE / LADDER` `SGF-03-ADJ-URL` `SGF-03-ADJ-PROVIDER` `SGF-03-ADJ-HOVER-DELAY` `SGF-03-ADJ-COMMENT-LAYERS`

**Independent remainder candidates (user goals with at least one registered EP, excluding the 12 accepted):** OPEN, GIB, RECENT, CLIP, SAVE-MORE, TEMP, NEW, KOMI, SETUP, INSERT, DELETE-MOVE, MAIN, XFORM, META, MARKUP, AUTOPLAY, NEXT-HINT, TREE-CLICK, TRYPLAY, SCORE, LADDER, URL (06 route), PROVIDER (06 route), HOVER-DELAY, COMMENT-LAYERS (comment pane, not a menu item).

**Extra vs 35:** none as Capability rows. Independently TRYPLAY, SCORE, and LADDER are three user goals; the index already stores them as one heading. `btnChangeTurn` folds into SETUP. Clipboard board images fold into SAVE-MORE. `addSuggestionAsBranch` is a 04 route.

**Missing vs 35:** none. Every adjacent heading has a registered EP or an explicit 06 route. `SGF-03-DTO` has no UI Entry Point by design. File Resume is missing *as an EP* and is correctly absent from the 35.

**Domain 03 remainder for Ticket 03:** the 23 adjacent headings. Do not add coords/move-numbers (02), File exit (01), Game new-match (05), Analyze (04), URL/provider fetch (06), or Help (07) as extra Domain 03 Capabilities.

---

## 5. Line citations (index)

| File | Lines |
| --- | --- |
| Worktree HEAD | `7b4027531c2b26062d0bfc27a040cc550cfbea4d` |
| `Menu.java` File | 181–454 (Resume 412–425 commented) |
| `Menu.java` View coords/move/autoplay/next | 518–658, 734–899, 1081–1090 |
| `Menu.java` Game 03 | 3009–3287 |
| `Menu.java` Edit | 3553–4041 |
| `Menu.java` share bar commented | 4043–4047 |
| `Menu.java` recents | 207–210, 6721–6744 |
| `Menu.java` top strip | 7783–8210, 8694–8712, 9704–9736; circle constructed 8074–8087 not added 8190–8198 |
| `Menu.java` color/pass construct | 7084–7175 (`this.add` commented; `topPanel.add` 8202–8207) |
| `RightClickMenu.java` `this.add` | 267–287 |
| `RightClickMenu2.java` `this.add` | 110–117 |
| `BottomToolbar.java` construct/add/handlers | 600–752, 1171–1391 |
| `Input.java` `keyPressed` | 312–823; mouse 28–136; wheel 847–878 |
| Inventory Domain 03 index | `docs/JAVA_CAPABILITY_INVENTORY.md` 440–444 |
