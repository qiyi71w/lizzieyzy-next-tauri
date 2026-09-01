# 01 — Application Shell Interaction Census

Independent Domain 01 census for [Ticket 01](../issues/01-audit-application-shell-interaction.md). Destination writes land in `docs/JAVA_CAPABILITY_INVENTORY.md` Domain 01, `docs/PARITY_MATRIX.md` APP-01..APP-05 owned fields, and `docs/MIGRATION_PLAN.md` R5 / APP-01 split / R11 membership.

## Sources

| Tree | Path | Commit |
| --- | --- | --- |
| Java Migration Baseline v1 | `/home/dev/dev/weiqi/worktrees/lizzieyzy-next/java-baseline-v1` | `7b4027531c2b26062d0bfc27a040cc550cfbea4d` (verified `git rev-parse HEAD`) |
| Next inventory baseline | this worktree | `18c6d189b8b01069975c4c40ead63a010249cb8c` (Accepted-field comparison only) |
| Current destination docs | this worktree | `1136b8b8fa65b80f1b01c44707a4e4e13b01a120` at census time |

Read-only Java source census. No production edits, tests, builds, or runtime launches. Prior coverage-audit research 01 and Ticket 08 dispositions are product-decision sources, not a stop condition for this count.

## Method

1. Trace `Lizzie.main` → `start` → `shutdown` / `shutdownLoggingThenExit`.
2. Count a Capability only when it is one observable user goal. Count an Entry Point only when a control is constructed **and** added, shown, or registered.
3. Inventory File/window/drop/argv/dynamic chip/`Input` hold-X and dispatcher surfaces from `Lizzie.java`, `LizzieFrame.java`, `Menu.java`, `Input.java`, `MenuPresentationMode.java`, `Config.java`.
4. After the census completed, compare the computed count with reference 13. 13 was not used as a stop condition.
5. Runtime-check is allowed only for a named dynamic-visibility, default, persistence, or failure fact not recoverable from frozen source. Packaged/installed association live evidence is a `REL-04` link, not a Domain 01 Java runtime check.

Key Java files: `Lizzie.java`, `gui/Menu.java`, `gui/MenuPresentationMode.java`, `gui/LizzieFrame.java`, `gui/Input.java`, `Config.java`.

## Computed count versus reference

**Computed Domain 01 shell-owned Capabilities: 13.**

Reference 13. Extra rows: none. Missing rows: none.

IDs: `SHELL-01` … `SHELL-13`.

## Shell-owned Capabilities

| Frozen ID | Observable user goal | Registered Entry Points | Default / persistence | Failure / non-mutation | Source | Runtime-check | Mapping |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `SHELL-01` | Launch the desktop process and show the main window | OS executable / `java … featurecat.lizzie.Lizzie`; installer launch linked to 07 | `appName=LizzieYzy Next`; `lizzieVersion=2.5.3`; `nextVersion` from `-Dlizzie.next.version` / `LIZZIE_NEXT_VERSION` else `next-dev`. Config load; first-run may write a profile. | Hostname lookup timeout 500ms never treats unknown host as machine change. Logging bootstrap failure prints stderr and continues. Engine-manager construct failure → empty manager. Profile save failure → `startupProfileSaveFailed`. | `Lizzie.java:320-460,967-1045` | Not required | Equivalent: `UI-01` chrome, `UI-04` no-engine launch, `REL-04` installed launch. No new shell item. |
| `SHELL-02` | Choose whether to autoload an engine, start empty, or show the engine picker | Startup only. Flags that *set* autoload belong to 02/04. | Missing keys: all autoload flags false. New bundled KataGo profile writes `autoload-default=true`. Persist `uiConfig` `autoload-default/last/empty`, `last-engine`, `default-engine`. | `shouldOfferEngineRepair` when no configured engine and not autoload-empty. | `Lizzie.java:415-454,1129-1133`; `Config.java:762-790` | LoadEngine dialog visibility and empty-engine first paint (dynamic visibility) | Next redesign, domain 04: `ENG-06` Autoload Default (first use unmarked, starts with no engine). No Java modal picker. Empty/failed start is `ENG-02`. |
| `SHELL-03` | Open a path passed as the single CLI argument | `args[0]` when it is not `"read"` and `mainArgs.length == 1` | N/A if no argv. Recents via 03. | `loadFile` errors owned by 03. Extra argv after the first is ignored. | `Lizzie.java:1012-1017` | Not required. Warm single-instance activation has no Java `OpenFilesHandler` / IPC (Next redesign under `APP-01`). Installed Open With is `REL-04` live. | Next redesign: `APP-01` Native File Activation for `.sgf`/`.gib` (cold/warm). Replacement uses `SGF-07`. |
| `SHELL-04` | `read` launch mode (hide status and engine menus) | argv exactly `read` | `readMode=false`. Mode is not persisted. | N/A | `Lizzie.java:432-437`; `Menu.java:6547-6550` | Remaining analysis UI visibility under `readMode` (dynamic visibility) | Abandoned. Exclusion. No-engine review remains `UI-04`. |
| `SHELL-05` | Reload last auto-saved SGF when no CLI file | Startup if `config.autoResume` | `resume-previous-game` default **false**. Files `save/autoGame1.sgf` else `autoGame2.sgf`. File › Resume is commented (`Menu.java:412-416`) and is not an Entry Point. | Missing file → skip. | `Lizzie.java:1018-1020`; `LizzieFrame.java:4523-4534` | Dirty autosave index shuffle when `save-auto-game-index2 == -5` (persistence) | Next redesign: `APP-04` Current-game Session Recovery. Prompt after unclean exit; restore-at-launch default off. |
| `SHELL-06` | First-run engine setup and hostname-change persist wipe | Startup when `firstTimeLoad` or resolved hostname differs | New profile / `firstTimeLoad`. Persist `host-name`, `first-time-load`. | Save IOException → `startupProfileSaveFailed` and repair chip. Unknown host never treated as machine change. | `Lizzie.java:374-386,518-627,894-964` | Not required | Split: first-run engine/profile → `ENG-06` / `REL-05` / domain 04 failure. Hostname-based deletion **Abandoned**. Invalid geometry → `WINDOW-01`. Does not auto-open FirstUse. |
| `SHELL-07` | Overlay chip to repair a failed/missing engine at launch | `engineStartupStatusButton` click when snapshot `isActionable`; constructed and `basePanel.add` | Hidden until actionable. Not persisted. | Click opens KataGo autosetup; further repair is 04. | `LizzieFrame.java:1898-1945` | `engineStartupStatusButton` visibility and copy when snapshot `isActionable` (dynamic visibility) | Next redesign, domain 04: fold into Engine Switcher / Engine Settings and `ENG-07`. No duplicate shell identity. |
| `SHELL-08` | Persist, stop engines/sidecars, then exit 0 | File › Exit; window close (`WINDOW_CLOSING`) | `auto-save-exit` default **true**. Persist file + config; optional autosave SGF. | Persist/save errors show modal with path; resource failures print stack; process still exits 0. Java graceful path does **not** confirm dirty documents. `uiConfig` `confirm-exit` default false is written in `Config.createDefaultConfig` (`Config.java:2880`) and is **never read** by `shutdown()` — not a Capability. | `Lizzie.java:1428-1516`; `LizzieFrame.java:1820-1825`; `Menu.java:443-454` | Not required | Next redesign: `APP-03` Safe Graceful Shutdown. Dirty game: Save / Discard / Cancel. Failed or cancelled save aborts exit. Timeout names the stuck resource; Retry or contextual Exit anyway. |
| `SHELL-09` | Exit immediately without persist | File › Force Exit (`fileMenu.add(forceExit)`) | N/A. No persist. | Unsaved work lost. | `Menu.java:429-441` | Not required | Abandoned. Exclusion. Contextual timeout fallback lives only inside `APP-03` after graceful teardown fails. |
| `SHELL-10` | Present `Menu` as custom strip or native bar | Automatic at frame construct; `-Dlizzie.menu.presentation` | Default `auto` (Wayland Linux → native; else custom). Process property only. | N/A | `MenuPresentationMode.java:32-48`; `LizzieFrame.java:861-862,1270,1974-1976` | Native vs custom menu-strip focus and presentation (dynamic visibility) | Swing-only exclusion. Next HTML chrome is `UI-01`. |
| `SHELL-11` | Route focused main-board keys/mouse/wheel to Capabilities | `mainPanel` `Input` via `addInput(false)`; independent-board / subboard inputs | Main board focus after start. Not persisted. | Unregistered keys are no-ops. Board `Input` is registered on `mainPanel`; comment editor and GTP console are separate focus owners. | `LizzieFrame.java:1889-1891,2859-2896,8718,9905`; `Input.java:12-880` | Not required | Next redesign: `APP-05` Shortcut Registry and Reference. Does **not** expand accepted `UI-05`. Per-action semantics stay with owner domains. |
| `SHELL-12` | Drop files on the board to open or batch-analyze | OS drag-drop onto `mainPanel` `TransferHandler` | N/A | Exception print, return false. One file `loadFile`; multiple `isBatchAna` + `StartAnaDialog`. | `LizzieFrame.java:1277-1348` | `DataFlavor.javaFileListFlavor` `toString()` split parse (failure / non-mutation of drop) | Next redesign: `APP-02`. One `.sgf`/`.gib` follows `APP-01`/`SGF-07`. Multiple files → `ANA-07` batch intake; never silently replace the current game. |
| `SHELL-13` | Show shortcut cheat-sheet while X is held | `VK_X` press/release in `Input` | Off. Not persisted. | N/A. Shift+X is settings (domain 02). | `Input.java:583-611,828-833`; `LizzieFrame.commands.*` | Not required | Next redesign: `APP-05` searchable Shortcut Reference from Help or `?`. Input controls do not surrender typed `?`. |

## Explicit non-rows (not extra Domain 01 Capabilities)

| Surface | Why it is not a Domain 01 Capability |
| --- | --- |
| `confirm-exit` | Default-config key never read by `shutdown()`. |
| File › Resume | Commented `fileMenu.add(resume)`. Startup `autoResume` remains `SHELL-05`. |
| Help › Diagnostics / Stop Full Trace / About / Check Update / Clear personal data | Registered Help chrome; semantic owners are Domain 07 (`REL-09`, `REL-10`, recents split). |
| Share kifu menubar | `this.add(shareKifu)` commented (`Menu.java:4047`). Share keys are Domain 06 no-ops. |
| Contribute visibility / `engineMenu2` | Domain 02/04/06 owners. |
| `readBoard` menu | `live.add` only on Windows; `Alt+O` no-ops off Windows → Domain 06. |
| OS file-association packaging | `REL-C09` / `REL-04`. Semantics stay `APP-01`. |
| Smoke probes `-Dlizzie.smoke.*`, Apple Silicon auto-opt, first-run benchmark, `Discribe`, diagnostic internals | Lab/internal, not ordinary user Capabilities. |
| `forceExitTrial` | WebBoard trial control (Domain 06), not File › Force Exit. |
| Java single-instance / `OpenFilesHandler` | Absent in frozen source. Warm activation is Next redesign under `APP-01`, not a missing Java census row. |

## Owner routing (this ticket records links only)

| Concern | Owner | Domain 01 records |
| --- | --- | --- |
| Parse / replace | `SGF-07` (`SGF-08` GIB) | Link from `SHELL-03` / `SHELL-12` / `APP-01` / `APP-02` |
| Recents / New | `SGF-09` / `SGF-10` | Cross-domain route, not a SHELL row |
| Multi-file batch | `ANA-07` | Link from `SHELL-12` / `APP-02` |
| Autoload / picker / repair | `ENG-06` / `ENG-02` / `ENG-07` | Link from `SHELL-02` / `SHELL-07` |
| Geometry | `WINDOW-01` / `LAYOUT-*` | Link from `SHELL-06` |
| Persist mechanism | `PREF-01` | Link from `APP-03` / `APP-04` |
| Installed activation evidence | `REL-04` | Link from `SHELL-01` / `SHELL-03` / `APP-01`. Not an Item Start Prerequisite for `APP-01` semantic work. |
| Engine/sidecar teardown | 04 / 06 / 07 | Link from `APP-03` |

## APP-01..APP-05 authority split

| Item | Matrix-owned | Plan-owned |
| --- | --- | --- |
| `APP-01` Native File Activation | Status Missing. Repository: no `fileAssociations` / cold-warm path. Live: Installed Live via `REL-04` (pending). Remaining gap and acceptance: cold/warm `.sgf`/`.gib`, one-window focus, `SGF-07` preserve-on-cancel. **Depends on: `SGF-07` only** (Item Start Prerequisite). | R5 semantic gate; remains unaccepted until R11 + `REL-04`. Delivery Order after `SGF-07`. `REL-04` is the additional R11 final-acceptance gate, not a start prerequisite. |
| `APP-02` Window file-drop | Status Missing. Depends on `SGF-07` and the passed R5 `APP-01` semantic gate (never `APP-01` Accepted). | R5 after the recorded `APP-01` semantic gate. |
| `APP-03` Safe Graceful Shutdown | Status Missing. Depends on `SGF-06`, `PREF-01`. | R5. |
| `APP-04` Current-game Session Recovery | Status Missing. Depends on `PREF-01`, `SGF-01`, `SGF-02`. Recover tree, personal `C`, `NodePath`, source path, dirty. Never engines/jobs/Match/provider. Restore-at-launch default off. | R5. |
| `APP-05` Shortcut Registry and Reference | Status Missing. Depends on `UI-05`. Does not expand `UI-05`. | R5. |

Plan R5 Delivery Order (Plan only; not a second Matrix start-prerequisite set): `PREF-01` → `SGF-07` → `APP-03`/`APP-04`/`APP-05` → `APP-01` semantic gate → `APP-02`.

## Remainder and Accepted-field preservation

Every Domain 01 Capability maps to one supported Parity Item, an explicit exclusion, or an absorbed redesign. There is no application-shell remainder.

Original sixteen Accepted items versus Next `18c6d189b8b01069975c4c40ead63a010249cb8c` keep ID, observable scope, status, evidence, remaining gap, and acceptance: `BASE-01`, `BASE-02`, `SGF-01`–`SGF-06`, `RULE-01`, `UI-01`, `UI-03`, `UI-04`, `UI-05`, `ENG-01`, `ANA-05`, `READ-03`. `UI-02` remains Partial. This ticket does not expand `UI-01` / `UI-04` / `UI-05` or edit foreign `SGF-07` / `ENG-*` / `LAYOUT-*` / `REL-*` contracts.

Repository evidence is never written as Installed Live. `APP-01` live evidence class remains Installed Live through `REL-04`.
