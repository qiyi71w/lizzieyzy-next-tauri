# 01 — Application Shell Entry Point Routing Census

## Sources

| Tree | Path | Commit |
| --- | --- | --- |
| Java Migration Baseline v1 | `/home/dev/dev/weiqi/worktrees/lizzieyzy-next/java-baseline-v1` | `7b4027531c2b26062d0bfc27a040cc550cfbea4d` |
| Next | `/home/dev/dev/weiqi/worktrees/lizzieyzy-next-tauri/migration-coverage-audit` | `18c6d189b8b01069975c4c40ead63a010249cb8c` |

Read-only source census. No edits, tests, builds, or runtime launches. Behaviors that cannot be proven from these frozen bytes are marked `需定向运行核验`.

Domain 01 owns **global Entry Point routing and shell-owned process/chrome lifecycle only**. Domain-owned Capabilities are routed to 02–07; this report does not duplicate their deep inventories.

## Method

1. Trace `Lizzie.main` → `start` → `shutdown` / `shutdownLoggingThenExit`.
2. Confirm every menu/toolbar/status/context control from **construction plus parent add/show**, not from class existence. A constructed object that is never added is not an Entry Point.
3. Inventory `Menu` constructor (`this.add`), `WindowMenuStrip` wrapping, `BottomToolbar` fields, `RightClickMenu` / `RightClickMenu2` `this.add`, `Input` `keyPressed`/`mouse*`/`mouseWheelMoved`, `LizzieFrame` window close + `TransferHandler` drop, CLI `mainArgs`.
4. Map Next chrome from `AppChrome.tsx`, `App.tsx` `onKey`, `BoardCanvas.tsx` `handleKeyDown`, `tauri.conf.json`.
5. Match existing Parity Items from `docs/PARITY_MATRIX.md` without inventing dispositions.

Key Java files: `Lizzie.java`, `gui/Menu.java`, `gui/MenuPresentationMode.java`, `gui/WindowMenuStrip.java`, `gui/LizzieFrame.java`, `gui/Input.java`, `gui/BottomToolbar.java`, `gui/RightClickMenu.java`, `gui/RightClickMenu2.java`, `gui/DiagnosticsDialog.java`, `update/WindowsUpdateController.java`, `Config.java`.

Key Next files: `apps/desktop/src/App.tsx`, `apps/desktop/src/components/AppChrome.tsx`, `apps/desktop/src/components/BoardCanvas.tsx`, `apps/desktop/src/App.test.tsx`, `apps/desktop/src-tauri/tauri.conf.json`, `docs/PARITY_MATRIX.md`.

## Reachability rules applied

| Constructed | Added / registered | Verdict |
| --- | --- | --- |
| `JFontMenu file/view/game/analyze/edit/live/help/quickLinks/settings/contribute/engine/engine2` + AI Coach/Commentary buttons | `Menu.this.add(...)` | Reachable menubar Entry Points |
| `shareKifu` + share items | `// this.add(shareKifu)` at `Menu.java:4047` | **Not** a menubar Entry Point |
| File › resume previous game | commented `fileMenu.add(resume)` | **Not** a menu Entry Point; startup `autoResume` remains a separate path |
| `readBoard` item | `live.add(readBoard)` only if `OS.isWindows()` (`Menu.java:4195-4198`) | Windows-only menu item; `Alt+O` in `Input` still calls `openBoardSync()` on all platforms, which then no-ops off Windows |
| Contribute menu | added, then `setVisible(Lizzie.config.showContribute)` | Conditional |
| `engineMenu2` | added; hidden unless double-engine / not `readMode` | Conditional |
| Help › Diagnostics / Stop Full Trace / About / Check Update / Clear personal data | all `helpMenu.add(...)` | Reachable |
| `DiagnosticsDialog` internals (`DiagnosticBundleExporter`, modules, export worker) | opened only from Help item | Not separate Capabilities |
| `-Dlizzie.smoke.open*` probes | require JVM flags | Lab-only; not ordinary user Entry Points |
| `Discribe` dialog class | no Help menu add | Not a Java Help Entry Point |

Menu presentation: `MenuPresentationMode.detectCurrent()` (`lizzie.menu.presentation` = `native`/`custom`/`auto`). Default is custom in-window strip except Linux Wayland → native `JMenuBar`. Custom path paints `WindowMenuStrip(menu)` on `basePanel` (`LizzieFrame.java:861-862,1954`). macOS keeps in-window menus (`apple.laf.useScreenMenuBar=false`, `Lizzie.java:1075-1076`).

Most shortcuts are **not** `JMenuItem.setAccelerator`. Confirmed accelerators: Yike live `Shift+O`; Windows readboard `Alt+O`. All other keys go through `Input.keyPressed` on `mainPanel` (`LizzieFrame.addInput`).

---

## Shell-owned Capabilities (01)

### SHELL-01 Launch application process

| Field | Value |
| --- | --- |
| Name | Launch the desktop process and show the main window |
| Entry Points | OS executable / `java ... featurecat.lizzie.Lizzie`; installer launch (07) |
| Frozen baseline | `main` bootstraps logging, `Config`, locale, LAF, icon, then one of the startup branches below, then `start` builds `Board`+`LizzieFrame`, `toolbar.setPopupMenu()`, `menu.doubleMenu(true)`, `showMainPanel`, `logApplicationReady`, deferred `EngineManager` |
| Defaults | `appName="LizzieYzy Next"`, `lizzieVersion="2.5.3"`, `nextVersion` from `-Dlizzie.next.version` / `LIZZIE_NEXT_VERSION` else `next-dev` |
| Persistence | `Config` load; first-run may write profile |
| Failure/recovery | Hostname lookup timeout 500ms never treats unknown host as machine change. Logging bootstrap failure prints stderr and continues. Engine manager construct failure → `engineStartupStatus.failed` then `EngineManager(config,-1,false)` empty. Profile save failure → `startupProfileSaveFailed` |
| Evidence | `Lizzie.java:320-460,967-1045` |
| Next mapping | Tauri window `apps/desktop/src-tauri/tauri.conf.json` 1440×900 `LizzieYzy Next`; React `App()` loads health + preferences + sample SGF in Tauri |
| Parity Item | UI-01 (chrome), UI-04 (no-engine launch), REL-04 (installed launch, Missing) |
| Ambiguity | Packaged argv / file-association launch `需定向运行核验` (07). |

### SHELL-02 Choose startup engine path

| Field | Value |
| --- | --- |
| Name | Decide whether to autoload an engine, start empty, or show the engine picker |
| Entry Points | Startup only (no menu). Preferences that *set* these flags → 02/04 |
| Frozen baseline | Order: `autoload-default` → `startConfiguredEngine(-1,true)`; else `autoload-last` → `startConfiguredEngine(last-engine,-1 default)`; else `autoload-empty` → `start(-1,false)`; else CLI `read` (SHELL-04); else no engines → `start(-1,false)`; else first-launch session → default engine index; else **modal** `LoadEngine.createDialog().setVisible(true)` |
| Defaults | Missing keys: all autoload flags false. **New profile** with bundled KataGo writes `autoload-default=true` (`Config.java:786-790`) |
| Persistence | `uiConfig` `autoload-default/last/empty`, `last-engine`, `default-engine` |
| Failure | `shouldOfferEngineRepair` when no configured engine and not autoload-empty; chip “click to repair” |
| Evidence | `Lizzie.java:415-454,1129-1133`; `Config.java:762-790` |
| Next mapping | No equivalent picker. Engine chip opens engine sheet. ENG-02 lifecycle Missing |
| Parity Item | ENG-01 (profiles), ENG-02 (lifecycle Missing), PREF-01 |
| Ambiguity | Whether `LoadEngine` remains visible until a choice; empty-engine first paint `需定向运行核验`. |

Owner of engine identity after start: **04**. Owner of the autoload *settings*: **02**. 01 only routes the startup decision.

### SHELL-03 Open file from process argv

| Field | Value |
| --- | --- |
| Name | Open a path passed as the single CLI argument |
| Entry Points | `args[0]` when it is not `"read"` |
| Frozen baseline | After `EngineManager` on EDT: `frame.loadFile(file,true,true)` and `LizzieFrame.curFile=file` |
| Defaults | N/A if no argv |
| Persistence | Recent files via 03 |
| Failure | `loadFile` errors → 03 |
| Evidence | `Lizzie.java:1012-1017` |
| Next mapping | None in `tauri.conf.json` (no `fileAssociations`, no CLI). Open is in-app dialog |
| Parity Item | SGF-06/UI-04 for in-app open; argv/OS assoc unclaimed |
| Ambiguity | Multi-arg, relative path, GIB vs SGF, Windows “Open with” `需定向运行核验`. Route capability body to **03**. |

### SHELL-04 Read-only launch (`read`)

| Field | Value |
| --- | --- |
| Name | Launch with status hidden and engine menus hidden |
| Entry Points | argv exactly `read` |
| Frozen baseline | `readMode=true`, `config.showStatus=false`, `start(-1,false)`, **return** before autoReplay/smoke probes. Later `engineMenu`/`engineMenu2` hidden |
| Defaults | `readMode=false` |
| Persistence | Does not persist the mode |
| Failure | N/A |
| Evidence | `Lizzie.java:432-437`; `Menu.java:6547-6550` |
| Next mapping | Absent |
| Parity Item | None |
| Ambiguity | How much of analysis UI remains `需定向运行核验`. |

### SHELL-05 Resume previous game on launch

| Field | Value |
| --- | --- |
| Name | Reload last auto-saved SGF when no CLI file |
| Entry Points | Startup if `config.autoResume` |
| Frozen baseline | `frame.resumeFile()` loads `autoGame1.sgf` else `autoGame2.sgf`, walks to end |
| Defaults | `resume-previous-game` default **false** (`Config.java:1703`) |
| Persistence | Flag in uiConfig; files under save/ |
| Failure | Missing file → skip |
| Evidence | `Lizzie.java:1018-1020`; `LizzieFrame.java:4523-4534` |
| Next mapping | Absent (sample SGF load is Next-only) |
| Parity Item | None dedicated; file recovery → **03** |
| Ambiguity | Interaction with dirty autosave index `-5` shuffle (`Lizzie.java:1022-1038`) `需定向运行核验`. |

### SHELL-06 First-run / host-change recovery

| Field | Value |
| --- | --- |
| Name | Automatic first-run engine profile setup; wipe persist on machine change |
| Entry Points | Startup when `firstTimeLoad` or resolved hostname differs from stored |
| Frozen baseline | Known-host change → `config.deletePersist(false)` + `resetAllHints()`. Then `completeAutomaticFirstRunSetup()`: try bundled KataGo autosetup, `first-time-load=false`, `config.save()`. **Does not** auto-open `FirstUseSettings` (that is Settings › Init, 02) |
| Defaults | New profile / `firstTimeLoad` |
| Persistence | `host-name`, `first-time-load`; persist wipe |
| Failure | Save IOException → `startupProfileSaveFailed` and repair chip |
| Evidence | `Lizzie.java:374-386,894-964,518-627` |
| Next mapping | `loadAppPreferences` only; no host-change wipe |
| Parity Item | PREF-01 Partial; ENG-01 |
| Ambiguity | What `deletePersist(false)` deletes vs `true` (View › reset UI positions) → **02**. |

### SHELL-07 Engine startup status chip

| Field | Value |
| --- | --- |
| Name | Overlay chip to repair a failed/missing engine at launch |
| Entry Points | `engineStartupStatusButton` click when snapshot `isActionable` |
| Frozen baseline | Hidden until listener; click → `openKataGoAutoSetup()` |
| Defaults | Hidden |
| Persistence | N/A |
| Failure | Opens autosetup; further repair → 04 |
| Evidence | `LizzieFrame.java:1898-1945` |
| Next mapping | Engine chip opens engine sheet; no repair overlay |
| Parity Item | ENG-02/ENG-04 Missing; REL-05 Partial |
| Ambiguity | Exact visible copy `需定向运行核验`. Route click target to **04**. |

### SHELL-08 Graceful shutdown

| Field | Value |
| --- | --- |
| Name | Persist, stop engines/sidecars, then exit 0 |
| Entry Points | File › Exit; window close (`WINDOW_CLOSING`) |
| Frozen baseline | `shutdown`: log; if `autoSaveOnExit` `frame.saveAutoGame(1)`; if `autoload-last` store `last-engine`; store `is-ctrl-opened`; `config.persist()` then `config.save()` (modal on failure, then continue); close contribute engine; `forceKillAllEngines`; `readBoard.shutdown`; clock helper; destroy analysis engine; `webBoardManager.stop`; `shutdownLoggingThenExit` → `System.exit(0)` |
| Defaults | `auto-save-exit` default **true** (`Config.java:1704`) |
| Persistence | persist file + config file; optional autosave SGF |
| Failure | Persist/save errors show modal with path; engines/readboard failures print stack; process still exits 0 |
| Evidence | `Lizzie.java:1428-1516`; `LizzieFrame.java:1820-1825`; `Menu.java:443-454` |
| Next mapping | File › `退出` **disabled** `title=尚未接入`. No App-level close handler. OS window chrome uses Tauri default. REL-03 updater Missing |
| Parity Item | PREF-01 (persist), UI-01 (window), REL-04 |
| Ambiguity | Dirty-document prompt: Java graceful path does not confirm; autosave is silent. Next has `window.confirm` only for replacing dirty SGF, not for window close. `需定向运行核验` Tauri close. |

### SHELL-09 Force exit

| Field | Value |
| --- | --- |
| Name | Exit immediately without persist |
| Entry Points | File › Force Exit |
| Frozen baseline | `System.exit(0)` |
| Defaults | N/A |
| Persistence | None |
| Failure | Unsaved work lost |
| Evidence | `Menu.java:429-441` |
| Next mapping | Absent |
| Parity Item | None |
| Ambiguity | None from source. |

### SHELL-10 Host menu presentation

| Field | Value |
| --- | --- |
| Name | Present the constructed `Menu` as custom strip or native bar |
| Entry Points | Automatic at frame construct; override `-Dlizzie.menu.presentation` |
| Frozen baseline | Wayland Linux → native; else custom `WindowMenuStrip` |
| Defaults | `auto` |
| Persistence | None (process property) |
| Failure | N/A |
| Evidence | `MenuPresentationMode.java:32-48`; `LizzieFrame.java:861-862,1270,1974-1976` |
| Next mapping | Always custom HTML `nav.menu-bar` (`AppChrome.tsx`) |
| Parity Item | UI-01 |
| Ambiguity | Native bar vs strip focus/overflow `需定向运行核验`. Swing-only presentation. |

### SHELL-11 Keyboard dispatcher

| Field | Value |
| --- | --- |
| Name | Route focused main-board keys/mouse/wheel to Capabilities |
| Entry Points | `mainPanel` listeners `Input`; independent boards have `InputIndependent*`; subboard `InputSubboard` |
| Frozen baseline | `addInput(false)` at construct; engine-game may swap `input2` + GTP `VK_E` |
| Defaults | Main board focus after start (`setMainPanelFocus`) |
| Persistence | N/A |
| Failure | Keys while comment/GTP focused `需定向运行核验` |
| Evidence | `LizzieFrame.java:1889-1891,2859-2896`; `Input.java:12-880` |
| Next mapping | `window` `keydown` in `App.tsx:214-347` with `shouldIgnoreApplicationShortcut` for inputs/textarea/select/contenteditable **and** `aria-label=棋盘`. Board canvas separately consumes arrows/Enter/Space for stone cursor (`BoardCanvas.tsx:144-170`) |
| Parity Item | UI-02, UI-05 |
| Ambiguity | See shortcut conflict table. |

### SHELL-12 OS file drop onto the main board

| Field | Value |
| --- | --- |
| Name | Drop files on the board to open or batch-analyze |
| Entry Points | OS drag-drop onto `mainPanel` |
| Frozen baseline | `TransferHandler` `javaFileListFlavor`. One file → `loadFile`. Multiple → `isBatchAna`, load first, open `StartAnaDialog` |
| Defaults | N/A |
| Persistence | N/A |
| Failure | Exception print, return false |
| Evidence | `LizzieFrame.java:1277-1348` |
| Next mapping | Sheet “导入棋谱” `<input type=file>` only; no window drop |
| Parity Item | SGF open → **03**; batch analyze dialog → **04** |
| Ambiguity | Flavor parsing via `toString()` split `需定向运行核验`. |

### SHELL-13 Hold-X controls overlay

| Field | Value |
| --- | --- |
| Name | Show shortcut cheat-sheet while X is held (no modifiers) |
| Entry Points | `VK_X` press/release in `Input` |
| Frozen baseline | Hides var tree/list/comment/blunder, `drawControls()`; release `stopShowingControl()` |
| Defaults | Off |
| Persistence | N/A |
| Failure | N/A |
| Evidence | `Input.java:583-611,828-833`; strings `LizzieFrame.commands.*` |
| Next mapping | Absent |
| Parity Item | None (Swing overlay). |

---

## Routing table (reachable → owner 02–07)

Capability keys are stable census IDs. Multiple controls that invoke the same user goal are one row.

### 02 Settings / layout / window / persistence

| Key | Capability | Java Entry Points (reachable) | Next now | Parity |
| --- | --- | --- | --- | --- |
| SET-LANG | UI language | Settings › language radios | Absent in chrome (prefs sheet exists) | PREF-01 Partial |
| SET-FONT | Frame font size | Settings › font size | Absent | PREF-01 |
| SET-LOOKS | Java vs system LAF | Settings › looks; restart hint | N/A Swing | PREF-01 |
| SET-SOUND | Play sound / mute in sync | Settings checkboxes | Absent | PREF-01 |
| SET-CONTRIBUTE-VIS | Show Contribute menu | Settings checkbox | Contribute menu always visible, items disabled | PREF-01 |
| SET-INIT | First-use settings dialog | Settings › init | Engine sheet / prefs | PREF-01 |
| SET-COMPREHENSIVE | Comprehensive settings | Settings › comprehensive; `Shift+X` | Prefs sheet | PREF-01 |
| SET-THEME | Theme dialog | Settings › theme → `openConfigDialog2(1)` | Prefs `boardTheme` | PREF-01 |
| SET-APPEARANCE | Apple style / classic colors / custom board images | View › appearance | Limited prefs | PREF-01 |
| SET-BOARD-STYLE | Japanese vs Chinese-classic board | View › appearance › board style | Absent | PREF-01 |
| LAY-PANELS | Show/hide subboard, winrate, comment, variation, list, info, status, GTP, controller, hawk-eye, suggestion list | View › Panels; many Alt/letter keys | Fixed 228/260 rails; no GTP/controller | LAYOUT-01 Missing; UI-01 Accepted chrome |
| LAY-TOOLBARS | Top strip combined/separated/hidden; wrap; bottom toolbar vis/detail; custom button sets | View › Toolbar | Always-on tool-strip + param-strip + folio-nav | LAYOUT-01 |
| LAY-BOARD-POS | Nudge main board `[` `]` | View › main board pos; keys | Absent | LAYOUT-01 |
| LAY-MODES | Default/classic/min/thinking/four-sub/double-engine/float/custom1/2 | View › layout; `Alt+1..9` | Absent | LAYOUT-01 |
| LAY-INDEP | Independent main/sub boards | View panels + independent submenu; `Alt+Q/A` | Absent | LAYOUT-01 |
| LAY-LARGE | Large subboard / large winrate | View › main panel settings; `Ctrl+F` / `Ctrl+W` | Absent | LAYOUT-01 |
| LAY-ONTOP | Always on top | View setting; `Ctrl+Z` | Absent | PREF-01 |
| LAY-RESET-POS | Reset stored UI positions + hints | View › delete persist file | Absent | LAYOUT-03 Missing |
| LAY-RESTORE-SIZES | Restore default panel sizes | View › panels | LAYOUT-03 Missing |
| LAY-COORDS | Coordinates | View checkbox; `C` | View check + `C` + bottom “坐标” | UI-05 |
| LAY-MOVENUM | Move-number modes | View › move submenu | Binary “手数” + `M` | UI-05 Partial vs Java richness |
| LAY-NEXTHINT | Next-move hint modes + min playouts | View › next hint | Absent | PREF-01 |
| LAY-AUTOPLAY-DLG | Auto-play dialog | View item; `Ctrl+A` | `Ctrl+A` **toggles** 800ms autoplay, no dialog | UI-05 conflict |
| WIN-BOUNDS | Window size/maximize restore | persist on shutdown | Tauri 1440×900 default | LAYOUT-02 Missing |

### 03 SGF / board / non-engine review

| Key | Capability | Java Entry Points | Next now | Parity |
| --- | --- | --- | --- | --- |
| SGF-NEW | Empty/new board | File › new; Game › new (also 05); toolbar new; `N` (also starts genmove game — 05) | File › 新建; toolbar; `N`; 清空棋盘 | SGF-04, UI-05 |
| SGF-OPEN | Open SGF/GIB | File › open; `O`; toolbar; drop (SHELL-12); argv (SHELL-03) | File › 打开; `O`; file input | SGF-01/06, UI-04 |
| SGF-RECENT | Open recent | File › recent (dynamic `updateRecentFileMenu`) | Disabled 尚未接入 | — |
| SGF-SAVE | Save original | File › save; `Ctrl+S` | File › 保存; `Ctrl+S`; dirty-only | SGF-06 |
| SGF-SAVEAS | Save As | File › save as; `S` | File › 另存为; `S` | SGF-06 |
| SGF-SAVE-MORE | Raw/comment-raw/branch/screenshots | File › more save; `Ctrl+Shift+S`, `Ctrl+Alt+S`, `Alt+S`, `Shift+S`, `Shift+Alt+S` | Submenu all disabled | — |
| SGF-TEMPSLOT | Save/load slots | File › 存档与读档 | Disabled | — |
| SGF-COPY | Copy SGF | File; `Ctrl+C` | Wired | UI-05 |
| SGF-PASTE | Paste SGF | File; `Ctrl+V` | Wired | UI-05 |
| SGF-COPY-BOARD | Copy main/sub screenshots | File items; `Shift+C` / `Alt+C` | Absent | — |
| SGF-READKOMI | Auto-load komi from SGF | File checkbox | N/A in UI | PREF-01 |
| NAV-MOVE | Undo/redo 1/10, first/last | Arrows/Page/Home/End; wheel; edit jump items; bottom toolbar | **Different keys** (see conflicts). BottomBar 首/上一/下一/末 | SGF-03, UI-05 |
| NAV-BRANCH | Prev/next branch, move branch, main trunk | `Left`/`Right`/`Shift+Left/Right`; `B`/`L`; edit items | Arrows remapped; 设为主分支 disabled; 返回主分支 disabled | SGF-03 |
| EDIT-PASS | Pass | Game › pass; `P`; toolbar pass | Wired `P` / 虚手 | SGF-04 |
| EDIT-CLEAR | Clear board | Edit; `Ctrl+Home` | `Ctrl+Home` / 清空棋盘 = **new game** | conflict |
| EDIT-DELETE | Delete move / branch / undo-redo edit | Edit; Delete / Shift+Delete | Shift+Delete removes variation only | SGF-04 |
| EDIT-SETUP | Setup mode tools | Game › starting position | Disabled in Next 游戏 menu | SGF-04 |
| EDIT-TRANSFORM | Swap colors / rotate / mirror | Edit; Ctrl+Alt+arrows | Disabled | — |
| EDIT-INFO | Game info | Edit; `I` | Disabled | — |
| EDIT-SIZE | Board size | Edit; `Ctrl+I` | Disabled | — |
| EDIT-STONE | Add black/white/alternate; insert modes; drag; double-click find; click-review | Edit; toolbar icons; RMB | Most disabled; pointer play wired | UI-02 |
| EDIT-MARKUP | Markup tools | doubleMenu markup buttons | Disabled 标记工具 | — |
| REVIEW-COMMENT | Personal comments | comment pane click-to-edit | AnalysisPanel comment | SGF-05 |
| CTX-EMPTY | Empty-point context menu | RMB empty intersection if `showRightMenu` or hovering suggestion | Absent | 03/04 mix |
| CTX-STONE | Stone context menu | RMB on stone | Absent | 03 |

RMB empty (`RightClickMenu`, added items): allow/avoid/track/priority, previous move, find, insert B/W, reedit, add suggestion as branch. RMB stone (`RightClickMenu2`): move stone, switch color, delete, review, previous, find. During human games many items hide (悔棋).

### 04 Foreground engine / analysis / cache

| Key | Capability | Java Entry Points | Next now | Parity |
| --- | --- | --- | --- | --- |
| ENG-SELECT | Select/switch engine | Engine menu dynamic 21 slots; `Ctrl+1-9`/`Ctrl+0` (switch fall-through) | Engine sheet profiles | ENG-01/02 |
| ENG-STOP | Shutdown current/other/all; restart | Engine submenu | Absent lifecycle | ENG-02 |
| ENG-CFG | Engine config dialog | Settings › engine; `Alt+X` | Engine sheet | ENG-01 |
| ENG-RULES | Live KataGo rules | Settings; `Shift+D` | Disabled | — |
| ENG-PARAM | Live engine params | Settings; `Alt+D` | Prefs/engine sheet partial | — |
| ENG-AUTOSETUP | KataGo one-click setup | Settings; startup chip | Engine sheet “一键设置” | ENG-01, REL-05 |
| ENG-REMOTE | Remote compute | Settings item | Disabled | — |
| ANA-PONDER | Start/stop analysis | Analyze › toggle; Space; toolbar analyse | Analyze › 开始/停止 → one-shot; no Space ponder | ENG-05, ANA-01 Partial |
| ANA-ONCE | Analyze this position | Analyze items / bottom “继续分析” | Wired if engineReady | ANA-01 |
| ANA-AUTO | Auto / batch / lightning / deep / part / branches | Analyze menu; `A`; `Ctrl+O` batch; `Ctrl+B` flash; `Ctrl+Shift+B` deep | Analyze game + fake flash; batch/deep disabled | ANA-02 Partial |
| ANA-STOP | Stop auto/batch | Analyze item; toolbar | Cancel when running | ANA-03 |
| ANA-HAWK | Blunder / 超级鹰眼 | Analyze; View panel; `Y` (Java hawk is Y; T is policy) | Disabled hawk; Next `T`/`H` = policy | ANA-04 |
| ANA-POLICY | Policy / heatmap | Analyze; `T` policy, `H` heatmap, `.` kata estimate | Overlay buttons; `H`/`T` both policy-ish | ANA-04 |
| ANA-ESTIMATE | Kata estimate modes | View estimate submenu | 形势判断 overlay | ANA-04 |
| ANA-TSUME | Tsumego / capture tsumego | Analyze; `Shift+E` / `Shift+T` | Disabled 死活 | — |
| ANA-CACHE | Clear Lizzie cache / this / bestmoves | Analyze items; `Shift+A`; `Alt+Delete` | Cache badge + ANA-05; menu “清除 Lizzie 缓存” disabled | ANA-05 |
| ANA-TEACHER | AI commentary / teacher | Analyze item; AI Commentary button | Fake analyze / AI 解说 | unclaimed |
| ANA-GTP | GTP console | View panel; `E` | Absent | — |
| ANA-FORCE | Allow/avoid regions | Toolbar select allow/avoid; RMB; Alt-drag | Disabled force-region icons | — |
| ANA-KOMI-PDA-WRN | Komi/PDA/WRN/time/playout limits | Menu komi panel fields | Komi readonly; PDA/WRN disabled | PREF-01/04 |
| ANA-SUGGEST | Suggestion display options | View › Suggestions | Prefs candidates/ownership/policy | UI-03, ANA-04 |

### 05 Match / game sessions

| Key | Capability | Java Entry Points | Next now | Parity |
| --- | --- | --- | --- | --- |
| GAME-NEW-GENMOVE | Human vs AI genmove | Game › new genmove; `N` | 新建 is empty SGF, not a match | GAME-01 Missing |
| GAME-NEW-ANA | Analyze-mode game | `createAnalyzeModeGameItem`; `Alt+N` | Disabled submenu | GAME-01 |
| GAME-NEW-ENGINE | Engine vs engine | Game; `Alt+E` | Disabled | GAME-01 |
| GAME-HUMANSL | Human SL / AI Coach | Game item; AI Coach button | Absent | GAME-01 |
| GAME-CONTINUE | Continue as B/W genmove or analyze | Game submenu; Enter / Alt+Enter | Disabled | GAME-01 |
| GAME-STOP | Stop human/engine game; pause; revise batch limit | Game items; Space (also ponder); Alt+R/T | Pause button cancels analysis | GAME-01/02 |
| GAME-SCORE | Score mode | Game checkbox; `Ctrl+Q` | Absent | GAME-03 |
| GAME-BEST | Play best/variation | Game; `,` / Alt+, genmove | Absent | GAME-03 |
| GAME-TIME | Set AI time | Game item | Absent | GAME-01 |
| GAME-INTERVENE | Manual intervene | Game item | Absent | GAME-01 |
| GAME-LADDER | Continue ladder | Game item | Absent | — |

### 06 Providers / live sync / readboard / publishing

| Key | Capability | Java Entry Points | Next now | Parity |
| --- | --- | --- | --- | --- |
| SYNC-YIKE-LIVE | Yike live dialog | Sync; `Shift+O` | Disabled | PROV-01 Partial |
| SYNC-YIKE-WEB | Built-in Yike page | Sync item | Disabled | PROV-01 |
| SYNC-YIKE-ROOM | Yike lobby browser | Sync → URL | Disabled | PROV-01 |
| SYNC-FOX | Fox kifu | Sync item | Sync sheet Fox | PROV-02 Partial |
| SYNC-TENCENT | Tencent kifu | Sync item | Sync sheet | unclaimed |
| SYNC-ONLINE-URL | Open online link | File › open URL; `Q` | Disabled | — |
| SYNC-READBOARD | Board sync sidecar | Sync item **Windows menu**; `Alt+O` all OS (no-op non-Windows) | Sync sheet readboard | READ-01/02 Partial |
| SYNC-READBOARD-OPT | Always sync / focus on wheel | Sync › readboard settings | Absent | READ-02 |
| SYNC-WEBBOARD | Local web board server | Sync › web board start/stop/copy URL | Absent | unclaimed |
| SYNC-LIVE-OPTS | Open HTML on live; always goto last | Sync checkboxes | Absent | PREF-01/06 |
| SHARE-SGF | Share current SGF | **Not in menubar**. `Ctrl+E`, `Alt+B`; toolbar `share` if shown | Absent | unclaimed |
| SHARE-HIST/SEARCH | Share history / public search | Only under unadded `shareKifu` | Absent | not an Entry Point |

### 07 Install / update / release / platform

| Key | Capability | Java Entry Points | Next now | Parity |
| --- | --- | --- | --- | --- |
| REL-ABOUT | About | Help › About → `openConfigDialog2(2)` (settings tab 2) | Help › 关于 sets status `LizzieYzy Next 0.1.0 · 桌面复盘工作区` | UI-01; not REL |
| REL-UPDATE | Check for update | Help › Check Update → `WindowsUpdateController.openCheckUpdatePage` | Help item **disabled** 尚未接入 | REL-03 Missing |
| REL-DIAG | Diagnostics and logs dialog | Help › Diagnostics (`DiagnosticsDialog.open`) | Absent from Help | unclaimed; platform |
| REL-TRACE | Stop full trace | Help item, enabled iff `LoggingRuntime.fullTraceActive` | Absent | unclaimed |
| REL-CLEAR-PII | Clear listed personal keys | Help › clear personal data (confirm) removes `fox-recent-searches`, `recent-files`, `batch-analysis-history`, `share-history` only | Absent | PREF-01 / 02 persistence; **not** full profile wipe |
| REL-CONTRIB | KataGo distributed training UI / settings / website | Contribute menu (if visible) | Disabled stubs | unclaimed |
| REL-PACK | Installers / file assoc / updater feed | Outside this shell (packaging) | `tauri.conf.json` bundle, no updater, no fileAssociations | REL-01..05 |

**Exclude from inventory:** `DiagnosticsDialog` export/module/trace checkboxes as separate Capabilities; `DiagnosticBundleExporter` internals; smoke `-D` probes; `Discribe` class.

---

## Dynamic actions (reachable)

| Generator | Registration | Owner |
| --- | --- | --- |
| `updateRecentFileMenu()` | File › recent, up to 5 paths | 03 |
| `updateFastLinks()` | Quick Links menu | 02/07 (external programs) |
| Engine `JFontMenuItem[21]` | rebuilt with engine list + shutdown/restart | 04 |
| `doubleMenu(boolean)` | rebuilds icon strip from config flags | 02 hosts; actions 03–05 |
| BottomToolbar `setPopupMenu()` | called from `Lizzie.start` | 02/03/04/05/06 |
| Contribute visibility | `showContribute` | 02 |
| `engineMenu2` | double-engine mode | 04 |

---

## Mouse / wheel / drag (non-menu)

| Gesture | Java | Next | Owner |
| --- | --- | --- | --- |
| Left click empty | Play / setup / score / markup | `playAt` | 03 |
| Double-click | Find stone if `allowDoubleClick` | Absent | 03 |
| Right click | Undo or context menu | Absent | 03/04 |
| Middle click release | Play current variation | Absent | 04 |
| Alt-drag | Select allow/avoid region | Absent | 04 |
| Drag stone | If `allowDrag` | Absent | 03 |
| Wheel on board | Undo/redo or PV branch step | Absent (jump input / buttons) | 03/04 |
| Wheel on subboard | Subboard branch | Absent | 04 |
| Wheel on var tree | Undo/redo | Absent | 03 |
| Hover suggestion | PV preview | 120ms hover preview | 04 / UI-03 |
| File drop | SHELL-12 | File input only | 03/04 |

---

## Shortcut conflicts (Java `Input` vs Next `App.tsx` `onKey`)

Next ignores app shortcuts when focus is input/textarea/select/contenteditable **or the board canvas**. Java mostly listens on `mainPanel`. Board canvas in Next uses arrows/Enter/Space for a **keyboard stone cursor**, which also collides with Java move navigation.

| Key | Java (`Input.java`) | Next (`App.tsx` / `BoardCanvas`) | Severity |
| --- | --- | --- | --- |
| ArrowLeft/Right | Previous/next **branch** | Parent / next **child**; on board: move cursor | **Conflict** |
| ArrowUp/Down | Undo/redo 1 (or PV) | Prev/next **sibling**; on board: move cursor | **Conflict** |
| Space | Toggle ponder / stop game | Board: play cursor point; no ponder | **Conflict** |
| Enter | Continue AI (analyze vs genmove with Alt) | Board: play cursor point | **Conflict** |
| Ctrl+Home | Clear board | New game | Related but not identical |
| N | `startNewGame()` (genmove) | Empty/new document | Related |
| Ctrl+A | Open AutoPlay **dialog** | Toggle interval autoplay | **Conflict** |
| O | Open file | Open file | OK |
| Ctrl+O | Batch analyze | Unbound | Java-only |
| S | Save As | Save As | OK |
| Ctrl+S | Save original | Save | OK |
| C | Toggle coordinates | Toggle coordinates | OK |
| Ctrl+C | Copy SGF | Copy SGF | OK |
| Shift/Alt+C | Copy board images | Unbound | Java-only |
| V | Try-play | Unbound | Java-only |
| Ctrl+V | Paste SGF | Paste SGF | OK |
| P | Pass | Pass | OK |
| H | Heatmap | Policy overlay | **Conflict** |
| T | Policy overlay | Toggle `showPolicy` pref | Near-conflict |
| M | Cycle move numbers | Toggle move numbers | Partial |
| Y | Hawk-eye / bad moves | Unbound | Java-only |
| E | GTP console | Unbound | Java-only |
| Q | Open online URL | Unbound | Java-only |
| 1–9 | Mouse-over candidate / engine Ctrl-switch | Select candidate index | Partial overlap |
| Shift+O | Yike live | Unbound | Java-only |
| Alt+O | Readboard | Unbound | Java-only |
| Shift+X | Comprehensive settings | Unbound | Java-only |
| Alt+X | Engine dialog | Unbound (engine chip) | — |
| Shift+Delete | Delete branch | Remove variation | Close |

UI-05 Accepted covers only the **claimed Next** subset. Unbound Java keys remain unclaimed, not abandoned.

---

## Unreachable / implementation-only (not classified abandoned)

| Item | Why |
| --- | --- |
| `shareKifu` menu | Constructed; `this.add` commented |
| File › resume | Commented add; startup flag remains |
| EditToolbar | Methods commented |
| `openConfigDialog()` old | Commented |
| `Discribe` | Helper dialog, no Help add |
| Diagnostic exporter/module UI | Internals of REL-DIAG |
| `-Dlizzie.smoke.*` | Lab probes |
| `Menu.intro` / Next “简介” | Next-only disabled stub; not in frozen Help |

---

## Cross-domain leaks (entries 01 must not swallow)

- Window close and File › Exit are 01, but autosave/persist fields are 02 and engine kill is 04.
- `N` / 新建 is 03 empty board in Next and 05 genmove in Java.
- Space is 04 ponder and 05 stop-game in Java.
- Drop of many files opens 04 `StartAnaDialog` after 03 load.
- Help › About opens **02** `ConfigDialog2` tab 2 in Java, not a dedicated about box.
- Clear personal data lives in Help (07 chrome) but mutates 02/03/06 persisted lists.
- Engine chip (01 overlay) opens 04 autosetup.

---

## Corrections vs invalid first-round reports

- Evidence is only `/home/dev/dev/weiqi/worktrees/lizzieyzy-next/java-baseline-v1` @ `7b4027531c2b26062d0bfc27a040cc550cfbea4d` and `/home/dev/dev/weiqi/worktrees/lizzieyzy-next-tauri/migration-coverage-audit` @ `18c6d189b8b01069975c4c40ead63a010249cb8c`.
- **No** citation of `42c92e3`, `/home/dev/dev/weiqi/lizzieyzy-next`, or `/mnt/d/dev/weiqi/tmp/lizzie-java-baseline-r2a`.
- Help Diagnostics, Stop Full Trace, About, Check Update, and Clear personal data **are reachable** (`helpMenu.add` at `Menu.java:5164-5254`). They are not missing menu items; Next currently shows only About (wired) + disabled Check Update + extra disabled “简介”.
- Share **menu** is not reachable; share **keys** are.
- readBoard **menu** is Windows-only; `Alt+O` is registered in `Input` on all OS then refused off Windows.
- Diagnostic package internals are not Capabilities.
- Ctrl+1–9 engine switch is real because `VK_1..VK_8` fall through to `VK_9` (`Input.java:768-814`), not because of a separate case per digit.
