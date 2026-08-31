# Inventory: Foreground Engine and Analysis Capabilities (Domain 04)

**Java unique source:** `/home/dev/dev/weiqi/worktrees/lizzieyzy-next/java-baseline-v1`<br>
**Java commit (worktree HEAD):** `7b4027531c2b26062d0bfc27a040cc550cfbea4d` (Migration Baseline v1)<br>
**Next unique source:** `/home/dev/dev/weiqi/worktrees/lizzieyzy-next-tauri/migration-coverage-audit`<br>
**Next commit:** `18c6d189b8b01069975c4c40ead63a010249cb8c` (`docs/migration-coverage-audit`)

No file from `/home/dev/dev/weiqi/lizzieyzy-next` or `/mnt/d/dev/weiqi/tmp/lizzie-java-baseline-r2a` is used as evidence. Commit `42c92e3` is not referenced.

Ticket: `.scratch/migration-coverage-audit/issues/04-inventory-engine-analysis.md`<br>
Standing R3 input: `.scratch/migration-coverage-audit/context/r3-foreground-engine-decisions.md`<br>
Also read: `CONTEXT.md`, `docs/JAVA_BASELINE.md`, `docs/PARITY_MATRIX.md`, `docs/MIGRATION_PLAN.md`, `docs/ARCHITECTURE_NEXT.md`.

## Census method

Read-only source census of the two frozen worktrees. Workspace `Grep` returned empty against these worktrees; file lists came from glob, then targeted `Read` of Config, Lizzie, EngineManager/tests, Menu, analysis GUI, AnalysisEngine, WholeGameAnalysis*, Next crates, Tauri gateway, and React analysis/cache UI.

Covered generation of user-reachable engine/analysis entry points:

| Surface | Key files |
| --- | --- |
| App startup / autoload | `Lizzie.java` `main`/`start`/`startConfiguredEngine`/`shutdown` |
| First-run repair status | `EngineStartupStatus.java`, `Lizzie.completeAutomaticFirstRunSetup` |
| Engine catalog settings | `MoreEngines.java`, `EngineData.java`, `Utils.engineDataToJson` (via `EngineDataJsonTest`) |
| Launch chooser | `LoadEngine.java` |
| Overflow chooser | `ChooseMoreEngine.java` |
| Menu switch / stop / restart / ponder | `Menu.java` (`engineMenu`, `shutdownEngine`, `pondering`) |
| Analysis settings | `AnalysisSettings.java` |
| Flash / auto / batch menus | `FlashAnalyzeMenu.java`, `AutoAnalyzeMenu.java`, `StartAnaDialog.java`, `AnalysisTable.java` |
| Whole-game session | `WholeGameAnalysisSession.java`, `WholeGameAnalysisPlan.java` |
| Analysis process | `AnalysisEngine.java`, `AnalysisRequestBuilder.java`, `AnalysisResourceCoordinator.java` |
| Foreground GTP | `Leelaz.java` (via tests/`initializeAfterVersionCheck`) |
| Switch identity | `EngineManager.EngineSwitchUiTracker` + tests |
| Candidate/stale | `AnalysisCandidateValidator.java`, `BoardData`, `MoveData` |
| Next engine/analysis | `EngineSetupPanel.tsx`, `App.tsx`, `backend.ts`, `crates/engine-manager`, `crates/katago-protocol`, `apps/desktop/src-tauri/src/lib.rs`, SQLite cache |

Profile **file format** (`leelaz.engine-settings-list`, `config.txt`, `persist`) is persistence evidence, not a Capability.

`EngineStartupBootstrap` is **absent** from this frozen Java tree. The observable startup owner is `EngineStartupStatus` (`READY` / `CHECKING` / `NEEDS_REPAIR` / `START_FAILED`) plus `Lizzie.engineStartupStatus`.

Legend for mapping columns: **Standing R3** = already decided Next redesign/input. **Java fact** = frozen baseline. **Uncovered** = must not be silently extended into R3.

---

## 1. Engine profile fields and catalog CRUD

### CAP-04-ENG-01 — Edit saved engine profiles

| Field | Value |
| --- | --- |
| Name | Create/edit/delete/reorder saved engine profiles |
| Entry Points | Engine Settings dialog `MoreEngines.createDialog`; table add/save/cancel/delete/move first/up/down/last; command scan `GetEngineLine`; remote SSH fields; GTP config button `GtpEngineConfigDialog`; encrypt command |
| Frozen baseline | User-visible profile fields: `name`, `commands` (launch line), `preload`, `width`, `height`, `komi`, `isDefault`, `useJavaSSH` + `ip`/`port`/`userName`/`password`/`useKeyGen`/`keyGenPath`, `initialCommand`, optional `gtpConfigurationProtocol`/`gtpConfigurationProfile`. Table shows 9 columns including preload/default/SSH. Catalog refresh `refreshEngineCatalog` / `updateEngines` on close if dirty. Deleting a row does **not** check whether that engine is currently primary/running (uncovered vs R3 delete-guard). Live GTP profile apply: if selected engine is started and `supportsGtpConfiguration()`, `applyGtpConfigurationProfile` runs immediately; otherwise saved for next start. |
| Defaults | New row: empty command, name localized “new engine”, `preload=false`, 19×19, komi 7.5, not default, not SSH. Bundled slot (`KataGo Bundled` / `KataGo Auto Setup`): command from package binary+weight+`gtp.cfg`; `preload` default false; komi 7.5; 19×19. |
| Persistence | `config.txt` → `leelaz.engine-settings-list[]` objects (`command`, `name`, `preload`, `komi`, `width`, `height`, `useJavaSSH`, `useKeyGen`, `keyGenPath`, `ip`, `port`, `userName`, `password`, `initialCommand`, `isDefault`, optional GTP profile). `ui.default-engine` index. Not a Capability. |
| Failure / recovery | Save/scan/GTP load failures show dialogs (`GtpEngineConfig.loadFailed`). Empty command on dirty check offers delete. Remote passwords encrypted. |
| Frozen evidence | `gui/EngineData.java:5-34`; `gui/MoreEngines.java:69-234,516-567,777-806,1202-1226,1288-1331`; `Config.java:645-760`; `util/EngineDataJsonTest.java:13-59` |
| Current Next mapping | `EngineSetupPanel` CRUD of `{id, profile:{name,engine_path,model_path,config_path,working_dir,backend:kata_go_analysis}, max_visits}`. Selecting a profile **saves selected_profile_id** and applies fields locally; it does **not** start a foreground process. No SSH, GTP profile, board size/komi, preload, initialCommand, default flag. Backend `kata_go_gtp`/`generic_gtp` exist on Rust DTO but TS only allows `kata_go_analysis`. Persistence `lizzieyzy-next-engine-profile.json`. |
| Next evidence | `apps/desktop/src/components/EngineSetupPanel.tsx:26-229`; `apps/desktop/src/domain/types.ts:48-52`; `crates/app-model/src/lib.rs:189-205`; `apps/desktop/src-tauri/src/lib.rs:36,213-238` |
| Existing Parity Item | **ENG-01 Accepted** (profile persist + asset checks). Successor needed for missing Java fields if kept. |
| Ambiguity / 核验 | Whether catalog save/`updateEngines` restarts a running matching engine. Whether delete of the running profile is blocked at runtime. Whether GTP live-apply is in R3 scope. |

Standing R3: Engine Settings must not implicitly change the Foreground Engine. Java `MoreEngines` can refresh catalog and apply GTP to a running engine — that is a **Java fact** that conflicts with standing R3 unless a later disposition splits “edit catalog” from “apply to run”.

---

## 2. Autoload, last-engine, no-engine launch

### CAP-04-ENG-02 — Four-state engine autoload

| Field | Value |
| --- | --- |
| Name | Choose what happens to the foreground engine at application startup |
| Entry Points | `LoadEngine` radio group; `MoreEngines` same radios (`rdoDefault` / `rdoLast` / `rdoMannul` / `rdoNone`); first-run bundled preference |
| Frozen baseline | Exclusive four-state: **autoload default engine**, **autoload last engine**, **manual chooser**, **no engine**. Conflicting true flags normalize to default-first (`updateStartupMode`). `Lizzie.main`: `autoload-default` → `startConfiguredEngine(-1,true)`; `autoload-last` → `startConfiguredEngine(last-engine,false)`; `autoload-empty` → `start(-1,false)`; else if args `read` → no engine; else if no configured engine → no engine; else if first launch session → start default; else show `LoadEngine` modal. New bundled profile sets `autoload-default=true` (others false). Existing profiles that already chose manual/empty are preserved (`shouldPreferBundled` comment). Missing keys: all three flags default **false** → **manual**. |
| Defaults | Existing `config.txt`: manual (all flags false). New complete package profile: autoload default. |
| Persistence | `ui.autoload-default`, `ui.autoload-last`, `ui.autoload-empty` in `config.txt`. Written by `MoreEngines.updateStartupMode` + `config.save()`, and by `LoadEngine` OK/no-engine/double-click. |
| Failure / recovery | If default engine command empty/broken Java JAR and autoload-default, bundled repair may retarget `default-engine`. Profile save failure → `EngineStartupStatus.failed` (`profileSaveFailed`). No bundled engine and not autoload-empty → `needsRepair`. |
| Frozen evidence | `Lizzie.java:415-453,916-940,967-1045,1129-1132`; `LoadEngine.java:77-189,226-278,382-399`; `MoreEngines.java:777-806`; `MoreEnginesStartupModeTest.java:10-88`; `Config.java:762-795` |
| Current Next mapping | **None.** Profiles load into the setup panel; no autoload, no last-engine, no startup chooser. Standing R3: autoload is an **option defaulting off**. That is a Next redesign vs Java new-profile autoload-default-on and vs four-state UI. |
| Existing Parity Item | ENG-02 Missing (lifecycle). PREF-01 Partial (settings inventory). Autoload itself is **uncovered** as a dedicated item. |
| Ambiguity | Whether Next keeps four states or boolean on/off; whether first-run bundled autoload-default is equivalent or redesign. **Do not decide here.** |

### CAP-04-ENG-03 — Remember last foreground engine

| Field | Value |
| --- | --- |
| Name | Persist the last primary engine index for next autoload-last start |
| Entry Points | Implicit on shutdown when autoload-last is set |
| Frozen baseline | On `Lizzie.shutdown`, if `autoload-last`, write `ui.last-engine = EngineManager.currentEngineNo`. Startup reads `uiConfig.optInt("last-engine", -1)`. |
| Defaults | `-1` if missing |
| Persistence | `ui.last-engine` via `config.persist()` then `config.save()` on shutdown |
| Failure / recovery | Persist failure shows modal with persist path; does not block exit after the dialog. |
| Frozen evidence | `Lizzie.java:426-428,1432-1460` |
| Current Next mapping | None. Next persists `selected_profile_id` in engine-profile JSON (settings selection, not a started run). |
| Existing Parity Item | None. Overlaps PREF-01 / ENG-02. |
| Ambiguity | Meaningful only if autoload-last is kept. Uncovered. |

### CAP-04-ENG-04 — Start with no engine

| Field | Value |
| --- | --- |
| Name | Use the desktop without a foreground engine |
| Entry Points | Autoload-empty; `LoadEngine` “不加载引擎”; `start(-1,false)` after failed start fallback; `read` CLI mode |
| Frozen baseline | `EngineManager` constructed with index `-1`. UI remains usable (JAVA_BASELINE `#348`). Repair chip if no configured engine and not autoload-empty. |
| Defaults | N/A |
| Persistence | Autoload-empty flag only |
| Failure / recovery | Failed `new EngineManager(config,index)` → `engineStartupStatus.failed` then retry `new EngineManager(config,-1,false)` |
| Frozen evidence | `Lizzie.java:429-430,440-441,996-1010`; JAVA_BASELINE `#348` |
| Current Next mapping | UI-04 Accepted: `未加载引擎`, SGF workflow without engine. No Engine Run object. |
| Existing Parity Item | **UI-04 Accepted**; ENG-02 still Missing for identity of the empty state. |

---

## 3. Start / stop / restart / switch / crash

### CAP-04-ENG-05 — Start a chosen foreground engine

| Field | Value |
| --- | --- |
| Name | Start the selected saved profile as the primary GTP engine |
| Entry Points | Autoload paths; `LoadEngine` OK/double-click; menu engine items (`Menu.engine[0..20]`); `ChooseMoreEngine` for engines beyond the first 21; `EngineManager(config,index,loadDefault)` |
| Frozen baseline | Primary identity is `Lizzie.leelaz` + monotonic `primaryEngineGeneration`. Startup restore uses a frozen route/barrier (`EngineManagerInitialStartupSynchronizationTest`): live `play` dropped until restore; handshake/`name`/`boardsize` flow; then one analyze + one ponder if `startPondering && !notStartPondering`. Bundled primary publishes `CHECKING`. Stale generation cannot publish checking/ready. After version check, PDA/UI/menu update only if generation still current. Remote compute may remap startup index (`RemoteComputeConfig.resolveStartupSelection`) — **domain 06**. |
| Defaults | Ponder starts unless `notStartPondering` (runtime, cleared after first ready). `playponder` field default `true`. |
| Persistence | Last-engine on shutdown if autoload-last. |
| Failure / recovery | Primary failures stay on accessible repair status (`Leelaz.shouldOpenInteractiveDiagnostic(true,*)==false`). Secondary may open diagnostics off first-launch. `EngineFailedMessage` redacts secrets. |
| Frozen evidence | `Lizzie.java:77-118,967-1045,1048-1177,1210-1260`; `EngineStartupStatus.java:8-147`; `EngineStartupDialogPolicyTest.java:28-150`; `EngineManagerInitialStartupSynchronizationTest.java:69-99` |
| Current Next mapping | `katago_analyze_once` / `katago_start_analyze_game` spawn a **one-shot/batch process from a profile DTO**, then the process exits. No ready handshake, no GTP, no Engine Run snapshot. ENG-02/03 Missing. Standing R3 wants manager-owned no-engine/starting/ready/stopping/error. |
| Existing Parity Item | ENG-02 Missing |
| Ambiguity | Foreground protocol is GTP in Java vs analysis JSONL in Next — standing R3 does not record this protocol cut. Uncovered product decision. |

### CAP-04-ENG-06 — Stop / restart foreground engine(s)

| Field | Value |
| --- | --- |
| Name | Stop or restart the current (or other/all) engine run |
| Entry Points | `Menu.shutdownCurrentEngine`, `restartCurrentEngine`, `shutdownOtherEngine`, `shutdownAllEngine`; secondary menu `shutdownCurrentEngine2`/`restartCurrentEngine2`; pondering button (pause analysis, not process kill) |
| Frozen baseline | Dedicated shutdown/restart menu items exist. Standing R3 “main workspace dedicated Stop” is a Next chrome decision; Java Stop is menu-based plus ponder toggle. |
| Defaults | N/A |
| Persistence | N/A |
| Failure / recovery | Failed restart uses same failed-switch/quarantine paths as start. |
| Frozen evidence | `Menu.java:53-78` |
| Current Next mapping | Cancel analysis job only (`katago_cancel_analysis`). No stop/restart of a long-lived engine. Standing R3 requires Stop. |
| Existing Parity Item | ENG-02 Missing |
| Ambiguity | Exact user-visible copy and whether restart auto-replays board — 需定向运行核验 (Menu.java body not fully walked; items are declared). |

### CAP-04-ENG-07 — Switch A → B with rollback and stale-token rejection

| Field | Value |
| --- | --- |
| Name | Switch primary engine to another profile without letting a failed or late B replace A |
| Entry Points | `Menu.engine[]` / `ChooseMoreEngine`; `EngineManager.switchEngine(index, ...)` |
| Frozen baseline | Switch UI phases: SWITCHING / ACTIVE / FAILED / IDLE. `begin` keeps **A as activeIndex** while target is B. `succeed(token)` promotes B only for current token. Late success/fail of superseded token ignored. `fail` keeps A name/index; target recorded as rejected. `abandonPending` → IDLE and rejects stale completion. Same token can succeed then fail (final lifecycle failure rolls back a ready target). `rollbackEngineSelectionAfterFailedSwitch`: if captured A incarnation is gone, primary becomes empty (`leelaz=null`, `currentEngineNo=-1`) until resync — not a silent B promotion. Setup-mode board **rejects** switch. Switch reserves current and frozen target (lifecycle reservation test). `ForegroundAnalysisPause` pauses pondering of A while B starts and restores if B fails. JAVA_BASELINE `#367`. |
| Defaults | N/A |
| Persistence | N/A |
| Failure / recovery | FAILED snapshot; EngineFailedMessage; quarantine of failed incarnation (`FAILED_ENGINE_QUARANTINES`). Engine-game participant crash recovery is **domain 05**. |
| Frozen evidence | `EngineSwitchUiTrackerTest.java:18-123`; `MenuEngineSwitchUiStateTest.java:50-150`; `EngineManagerLifecycleReservationTest.java:81-113`; `ForegroundAnalysisPause.java:8-50`; JAVA_BASELINE `#367` |
| Current Next mapping | None. Standing R3 matches Java A-stays-primary-until-B-ready and stale rejection; also says cancel unfinished A jobs then stop A. Java pause/restore of pondering vs cancel-jobs is a nuance for R3 implementation, not a new product fork unless jobs are analysis-engine leases. |
| Existing Parity Item | **ENG-03 Missing**, **ENG-04 Missing** |

### CAP-04-ENG-08 — Unexpected engine exit

| Field | Value |
| --- | --- |
| Name | Report unexpected exit and recover without promoting stale identity |
| Entry Points | Process exit / SSH disconnect / OpenCL native exit (engine-game enum also lists these) |
| Frozen baseline | Primary: repair/failed status, not interactive diagnostic on first failure. `autoCheckEngineAlive` default **true** (PREF overlap). Failed incarnations quarantined until stop completes. Standing R3: **no automatic restart** of foreground engine — this is a **Next redesign decision**, not proven as Java’s only path. Java engine-game **does** recover participants after exit (**domain 05**). |
| Defaults | `auto-check-engine-alive` true |
| Persistence | `ui.auto-check-engine-alive` |
| Failure / recovery | START_FAILED / NEEDS_REPAIR; click-to-repair (status `isActionable`). |
| Frozen evidence | `Config.java:1006,1719`; `EngineManager.java:96-100,187-191`; `EngineStartupDialogPolicyTest.java:28-31` |
| Current Next mapping | Spawn errors become command/event strings; no crash UI for a resident engine. Standing R3: report and wait for manual restart. |
| Existing Parity Item | ENG-02/04 Missing |
| Ambiguity | Whether `autoCheckEngineAlive` auto-restarts foreground GTP. **需定向运行核验** of Leelaz exit handler. Do not treat R3 “no auto restart” as a Java fact. |

### CAP-04-ENG-09 — Per-profile preload of extra GTP engines

| Field | Value |
| --- | --- |
| Name | Preload non-primary saved engines in the background |
| Entry Points | `MoreEngines` preload checkbox (column 3) |
| Frozen baseline | `EngineData.preload` persisted on each profile. `EngineManager.InitialEnginePreloadScheduler` starts daemon threads. Distinct from analysis-engine preload. |
| Defaults | `false` (including bundled slot) |
| Persistence | `engine-settings-list[].preload` |
| Failure / recovery | Secondary bundled startup must **not** publish primary CHECKING (`EngineStartupDialogPolicyTest`). |
| Frozen evidence | `EngineData.java:9`; `EngineManager.java:101-119`; `Config.java:735-748`; `EngineStartupDialogPolicyTest.java:87-101` |
| Current Next mapping | None |
| Existing Parity Item | None (not ENG-01). Uncovered vs R3. |

---

## 4. Analysis-engine lifecycle preferences (owned here)

These are user-visible and **lifecycle-affecting**; standing R3 does not mention them. Uncovered — do not silently fold into ENG-01/02.

### CAP-04-ANA-01 — Reuse current foreground engine for analysis JSONL work

| Field | Value |
| --- | --- |
| Name | Run lightning/whole-game analysis on the loaded GTP engine instead of a dedicated analysis process |
| Entry Points | `AnalysisSettings.chkReuseCurrentEngine`; used by `WholeGameAnalysisSession.awaitForegroundLeaseHandoff` |
| Frozen baseline | Default **false**. When true, dedicated command/generate/saved/remote/preload/auto-exit controls disable (`controlState`). Status line shows current engine name + exclusive GTP lease availability. Whole-game waits until lease AVAILABLE or engine changes. Empty dedicated command cannot save unless reuse is on. Exclusive lease blocks deferred HumanSL start. |
| Defaults | `false` |
| Persistence | `ui.analysis-reuse-current-engine` via `saveConfigSections` |
| Failure / recovery | Failed save leaves runtime flags and pending flash request unchanged. |
| Frozen evidence | `Config.java:1037,1826`; `AnalysisSettings.java:86-88,492-527,644-721`; `WholeGameAnalysisSession.java:189-207`; `AnalysisSettingsSaveConfigTest.java:48-91,125-204` |
| Current Next mapping | None. Next always spawns analysis JSONL from profile paths. |
| Existing Parity Item | None. Overlaps ENG-02 (lease) and ANA-01/02. |

### CAP-04-ANA-02 — Preload dedicated analysis engine

| Field | Value |
| --- | --- |
| Name | Keep a dedicated analysis process warm after app start |
| Entry Points | `AnalysisSettings.chkPreLoad`; `Lizzie.start` after EngineManager |
| Frozen baseline | Default **false**. If true, `frame.preloadConfiguredAnalysisEngineAfterStartup()`. Preload `AnalysisEngine(true)` uses purpose `PRELOADED_QUICK_ANALYSIS`. Preload must not show generated-config notice. Disabled in UI when reuse-current is on. |
| Defaults | `false` |
| Persistence | `ui.analysis-engine-preload` |
| Failure / recovery | Same save-failure atomicity as other analysis settings. |
| Frozen evidence | `Config.java:1036,1825`; `Lizzie.java:1039-1041`; `AnalysisEngine.java:170-180,197-200`; `EngineStartupDialogPolicyTest.java:80-85`; `AnalysisSettings.java:669-694` |
| Current Next mapping | None |
| Existing Parity Item | None. Distinct from per-GTP `preload`. |

### CAP-04-ANA-03 — Auto-quit dedicated analysis engine after job

| Field | Value |
| --- | --- |
| Name | Exit the dedicated analysis process when the current request finishes |
| Entry Points | `AnalysisSettings.chkAutoExit` |
| Frozen baseline | Default **true**. Disabled in UI when reuse-current is on. Command line template includes `-quit-without-waiting`. |
| Defaults | `true` |
| Persistence | `ui.analysis-auto-quit` |
| Failure / recovery | Save failure does not flip the flag. |
| Frozen evidence | `Config.java:1188,1993`; `AnalysisSettings.java:669-694`; `AnalysisEngineCommandHelper.java:21-22,68-70` |
| Current Next mapping | Next analysis processes already exit after stdout/timeout/cancel (one-shot/batch). No user toggle. |
| Existing Parity Item | None. Behavior is closer to Next’s current spawn-per-job than to a resident engine. |

### CAP-04-PREF-LIZZIE-CACHE — Enable in-tree Lizzie analysis cache

| Field | Value |
| --- | --- |
| Name | Reuse previously computed candidates/winrate on a node (Java “Lizzie cache”) |
| Entry Points | Not found as a dedicated menu in sampled files; setting is loaded from leelaz config. Toolbar/help may exist — **需定向运行核验** of the toggle UI. |
| Frozen baseline | `enableLizzieCache` default **true**, loaded from `leelaz.enable-lizzie-cache`. Node payloads live on `BoardData` (`bestMoves`, visits, ownership `estimateArray`, engineName). Whole-game skip already-complete nodes via `hasCompletePrimaryAnalysis`. This is **not** Next SQLite game-key cache. |
| Defaults | `true` |
| Persistence | `leelaz.enable-lizzie-cache` in `config.txt` |
| Failure / recovery | Incomplete payloads (empty candidate list) are not treated as terminal success. |
| Frozen evidence | `Config.java:244,1855`; `BoardData.java:703-770` |
| Current Next mapping | ANA-05 SQLite `analysis-cache.sqlite3` keyed by SGF hash + optional profile/engineKind; prefs `autoLoadCache`/`autoSaveAnalysis` default true. Different mechanism. |
| Existing Parity Item | **ANA-05 Accepted** for Next SQLite MVP. Java enable-lizzie-cache is a **separate** Capability / possible successor, not proven covered. |

---

## 5. Interactive pondering and one-shot jobs

### CAP-04-ANA-04 — Interactive pondering on the current node

| Field | Value |
| --- | --- |
| Name | Continuously analyze the current board with the foreground engine |
| Entry Points | `Menu.pondering`; toolbar `analyse` visibility pref; after engine ready (`engine.ponder()`); `AnalysisTable.stopGo` calls `togglePonder`; `playponder` (ponder while playing); `notStartPondering` skips first auto-ponder |
| Frozen baseline | Ready path starts ponder unless `notStartPondering`. `MoveData` parses GTP `info move … pv`. Limits: `limit-time` default true, `maxAnalyzeTimeMillis` 600000; `limit-playout` default false. `showPonderLimitedTips` default true. |
| Defaults | Auto-ponder on ready: on. `playponder` field default true. Toolbar `analyse` default true. |
| Persistence | `ui.analyse`, `ui.limit-time`, `ui.limit-playout`, `ui.limit-playouts`, `ui.show-ponder-limited-tips`. `playponder` persist key **需定向运行核验** (field exists; load line not in sampled block). |
| Failure / recovery | Pause/restore via `ForegroundAnalysisPause` during switch/game start. |
| Frozen evidence | `Lizzie.java:1250-1258`; `Menu.java:83,1884`; `MoveData.java:12-51`; `Config.java:64,1107-1110,1182,1884,1907-1909,2021` |
| Current Next mapping | `katago_analyze_once` is a **finite JSONL job** (60s timeout), not resident ponder. ENG-05 Partial: cancellation exists for full-game, not authoritative one-shot job. ANA-01 Partial. |
| Existing Parity Item | **ENG-05 Partial**, **ANA-01 Partial** |

### CAP-04-ANA-05 — One-shot “分析此手” (Next current path)

| Field | Value |
| --- | --- |
| Name | Analyze the selected node once |
| Entry Points | Next: EngineSetupPanel “分析此手”, bound toolbar commands. Java closest: ponder/flash-part, not a named one-shot JSONL job. |
| Frozen baseline | Java does not expose a separate “analyze this move once then quit GTP” control of the Next shape; interactive analysis is ponder. Lightning/whole-game use AnalysisEngine JSONL. |
| Defaults | Next max_visits UI default 800; Java `analysis-max-visits` default 1 (min 1). |
| Persistence | Next: profile `max_visits` + prefs `defaultMaxVisits`. Java: `ui.analysis-max-visits`. |
| Failure / recovery | Next: command error string; one-shot has **no** job-id cancel in ENG-05 sense. Stale: `shouldPublishReviewPresentation({generation,NodePath,requestToken})`. |
| Frozen evidence | Contrast `AnalysisEngine` vs Next `App.tsx:654-680`, `lib.rs:740-771` |
| Current Next mapping | Implemented as profile-spawned JSONL. |
| Existing Parity Item | ANA-01 Partial, ENG-05 Partial |

---

## 6. Flash, whole-game, auto, batch, tracking

### CAP-04-ANA-06 — Lightning (flash) analysis

| Field | Value |
| --- | --- |
| Name | Fast overview analysis of game / part / all branches + settings |
| Entry Points | Top toolbar `FlashAnalyzeMenu`: whole game Ctrl+B; part game; all branches; settings. Also `AutoAnalyzeMenu.wholeGameLightningItem` (same Ctrl+B). Missing command → settings dialog; OK resumes pending request, cancel discards. |
| Frozen baseline | `flashAnalyzeGame(true,false)` / `flashAnalyzePart` / `flashAnalyzeGame(false,true)` / `flashAnalyzeSettings`. Toolbar button visibility `flash-analyze` defaults **false** when double menu + top toolbar, else true. |
| Defaults | See above. `analysis-max-visits` default 1. `auto-quick-analyze-on-load` default **true**. |
| Persistence | `ui.flash-analyze`, analysis engine command/visits/rules, `ui.auto-quick-analyze-on-load`, `ui.quick-analysis-lightweight-model-enabled` |
| Failure / recovery | Missing command confirmation; failed settings save does not start analysis; `WaitForAnalysis` UI. |
| Frozen evidence | `FlashAnalyzeMenu.java:11-46`; `FlashAnalyzeMenuTest.java:16-38`; `AutoAnalyzeMenu.java:75-84`; `Config.java:1900-1901,1830-1832`; `AnalysisSettingsSaveConfigTest.java:95-123` |
| Current Next mapping | BottomBar `onFlashAnalyze` → `handleFakeAnalyze` (fake frames, not KataGo lightning). No part/all-branches. |
| Existing Parity Item | ANA-01/02 Partial only as generic analysis. Flash is **uncovered split**. |

### CAP-04-ANA-07 — Whole-game deep analysis

| Field | Value |
| --- | --- |
| Name | Two-stage baseline+deep analysis of the main line without blocking navigation |
| Entry Points | `AutoAnalyzeMenu` first item “整盘精析（推荐）” Shift+Ctrl+B → `openWholeGameDeepAnalysis` |
| Frozen baseline | Session states: IDLE, PREPARING, BASELINE, DEEP, PAUSING, PAUSED, COMPLETE, CANCELLED, FAILED. Baseline weight 20%, deep 80%. Default baseline visits 32, minimum deep visits 500. Engine start generation + dispatch generation drop stale starts. Pause closes engine; resume fails if game identity changed. Cancel → CANCELLED. Optional reuse of current engine with lease wait. Max stage attempts 3. |
| Defaults | Baseline 32, deep ≥500 |
| Persistence | Visits from analysis settings / request; plan itself is not a file |
| Failure / recovery | `fail("WholeGameAnalysis.error.engine")` / `gameChanged`; progress snapshot to UI |
| Frozen evidence | `AutoAnalyzeMenu.java:14-72`; `AutoAnalyzeMenuTest.java:20-38`; `WholeGameAnalysisSession.java:16-26,68-70,131-187,214-279`; `WholeGameAnalysisPlan.java:14-54` |
| Current Next mapping | `katago_start_analyze_game` one JSONL batch over mainline turns, progress events, cancel token. No two-stage baseline/deep, no pause/resume, no lease handoff, mainline-only (`project_current_game_mainline`). ANA-02 Partial. |
| Existing Parity Item | **ANA-02 Partial**, **ANA-03 Partial** |

### CAP-04-ANA-08 — Automatic analysis of the current game

| Field | Value |
| --- | --- |
| Name | Auto-step analysis with start/end move, time/playouts, color filters |
| Entry Points | `AutoAnalyzeMenu` “自动分析” → `StartAnaDialog`; bottom toolbar; `Menu.stopAutoAnalyze`; `AnalysisTable` stop/start |
| Frozen baseline | Runtime flags `isAutoAna` / `isStartingAutoAna`. Prefs: start/end move default -1; time/playouts/first-playouts 0; anaBlack/anaWhite true; `analyzeAllBranch` default false; diff-analysis optional; `exitAutoAnalyzeByPause` default true. Stop: `toolbar.stopAutoAna(true,true)`. |
| Defaults | See Config 941-950, 1282-1283, 1700 |
| Persistence | Many `ui.auto-ana-*` keys (sampled: diff-* ; start/end also fields — full key list 需定向运行核验 if needed for PREF-01) |
| Failure / recovery | Dialog cancel resets auto-ana if cancelled |
| Frozen evidence | `AutoAnalyzeMenu.java:26-54,99-104`; `StartAnaDialog.java:27-68`; `Config.java:941-950,1151-1163,1282-1283` |
| Current Next mapping | None (UI copy “开启自动分析” is empty-state text only). |
| Existing Parity Item | None. Not ANA-02. |

### CAP-04-ANA-09 — Batch file analysis

| Field | Value |
| --- | --- |
| Name | Analyze a queue of SGF files (ordinary or deep/analysis-mode) |
| Entry Points | `AutoAnalyzeMenu` batch / batch-deep → `openFileWithAna(false/true)`; `AnalysisTable` add files, stop/start, analysis-mode stop (`destroyAnalysisEngine`) |
| Frozen baseline | File dialog from persisted filesystem paths. Analysis-mode uses `StartAnaDialog(true)` and `AnalysisEngine`. `batch-analysis-playouts` default 100 (used when settings context is BATCH). |
| Defaults | `batch-analysis-playouts=100` |
| Persistence | `ui.batch-analysis-playouts`; filesystem persisted object |
| Failure / recovery | Destroy engine on stop in analysis-mode |
| Frozen evidence | `AutoAnalyzeMenu.java:31-59`; `AnalysisTable.java:26-119`; `Config.java:1128,1621` |
| Current Next mapping | None |
| Existing Parity Item | None. Uncovered; do not fold into ANA-02. |

### CAP-04-ANA-10 — Tracking analysis

| Field | Value |
| --- | --- |
| Name | Single-stream tracking requests with immutable display state (hover/track points) |
| Entry Points | Tracking UI (not fully enumerated here); parameters interval + target visits; default max visits 500 |
| Frozen baseline | AddResult: ADDED/DUPLICATE/ILLEGAL/CONTEXT_MISMATCH/LEASE_UNAVAILABLE. Progress timeout 8000ms. Appearance prefs for tracking points. |
| Defaults | `tracking-analysis-max-visits` migrated; 500 |
| Persistence | tracking appearance + max visits in `ui` |
| Failure / recovery | Lease unavailable; context mismatch |
| Frozen evidence | `TrackingAnalysisController.java:14-50`; `Config.java:1039-1045,1828-1829` |
| Current Next mapping | UI-03 candidate hover 120ms is review presentation, not tracking analysis jobs. |
| Existing Parity Item | UI-03 Accepted covers hover eviction only. Tracking jobs uncovered. Possible 06 overlap if tied to readboard. |

---

## 7. Candidates / PV / ownership / policy / stale results

### CAP-04-ANA-11 — Show candidates, PV, ownership, policy

| Field | Value |
| --- | --- |
| Name | Render engine suggestions and heat/territory on the review surface |
| Entry Points | Always-on when analysis payload exists; prefs: suggestion winrate/playouts/score, PV visits, estimate/heatmap, delay candidates, black/white candidates, `useMovesOwnership` |
| Frozen baseline | `MoveData`: coordinate, visits, winrate, variation (PV), policy/prior, scoreMean/stdev, ownership via `estimateArray` on BoardData. `limit-max-suggestion` default 10, `limit-branch-length` 15. `useMovesOwnership` default true. `showKataGoEstimate` default false. `delayShowCandidates` default false, 10s. `showBlackCandidates`/`showWhiteCandidates` default true. |
| Defaults | As Config fields 89-92, 199-200, 1013-1016, 1231-1235, 1268 |
| Persistence | matching `ui.*` / `leelaz.limit-*` keys |
| Failure / recovery | Occupied-point candidates stripped (`AnalysisCandidateValidator`). |
| Frozen evidence | `MoveData.java:12-35,52-110`; `AnalysisCandidateValidator.java:10-65`; `Config.java:199-200,1013-1016,1231-1235,1268,1852-1853`; `AnalysisRequestBuilder.java:11-23` |
| Current Next mapping | `AnalysisFrameDto` candidates + optional ownership/policy. Prefs showOwnership/showPolicy/showCandidates, candidateLimit 8. Overlay modes candidates/policy/ownership. Mini-board PV. UI-03 hover 120ms. |
| Existing Parity Item | **ANA-04 Partial**, **UI-03 Accepted** |

### CAP-04-ANA-12 — Cancel, supersede, and drop stale results

| Field | Value |
| --- | --- |
| Name | Ensure later completions cannot publish onto a different engine/node/job |
| Entry Points | Implicit in switch, navigation, cancel, whole-game pause/cancel |
| Frozen baseline | Switch tokens; `primaryEngineGeneration`; `EngineStartupStatus.Snapshot.isCurrent`; exclusive GTP lease; whole-game `engineStartGeneration`/`activeDispatchGeneration`; AnalysisEngine `analyzeMap` node ids; candidate validator vs older board. JAVA_BASELINE `#326` stale candidate eviction. |
| Defaults | N/A |
| Persistence | N/A |
| Failure / recovery | Ignore stale; rollback switch; cancel session |
| Frozen evidence | `EngineStartupStatus.java:44-47`; `Lizzie.java:104-118`; `WholeGameAnalysisSession.java:90-97,155-181`; `AnalysisEngine.java:85-86`; JAVA_BASELINE `#326` |
| Current Next mapping | Full-game: `AnalysisJobRegistry` + `AnalysisCancelToken`; UI `activeJobId` + `requestToken` + document `generation`. One-shot: new UUID per invoke, **no cancel token**. Stale UI gated by `shouldPublishReviewPresentation`. Standing R3: manager-owned job/switch identities. |
| Existing Parity Item | **ANA-03 Partial**, **ENG-05 Partial**, **ENG-04 Missing** |

Concurrency summary:

| Java | Next |
| --- | --- |
| One primary GTP (`leelaz`) + optional `leelaz2` | No resident engine |
| Dedicated AnalysisEngine or shared via exclusive lease | Spawn per job from profile |
| Preloaded GTP extras + preloaded analysis | Neither |
| Whole-game generation fence | Job id + review presentation scope |
| Switch: A primary until B ready | Not implemented |

---

## 8. Cache import/export and protocol errors

### CAP-04-ANA-13 — Persist analysis inside SGF and reopen it

| Field | Value |
| --- | --- |
| Name | Carry analysis headers/payloads through SGF comments/properties |
| Entry Points | Save/open SGF; `appendWinrateToComment` default true |
| Frozen baseline | `BoardData.analysisHeaderSlots`; comment notes imported SGFs may contain only serialized analysis headers; live engine results require at least one candidate to graph. This is **SGF round-trip of analysis**, overlapping domain 03. |
| Defaults | `append-winrate-to-comment` true |
| Persistence | SGF file |
| Failure / recovery | Placeholder 50% winrate + visits must not look like a complete curve |
| Frozen evidence | `BoardData.java:49-50,712-721`; `Config.java:196,1797` |
| Current Next mapping | Personal comments separate (SGF-05). Generated information field exists. No proven Java-compatible analysis-header round-trip. Cache is SQLite sidecar. |
| Existing Parity Item | SGF-05 Accepted (comments only). Analysis-in-SGF uncovered. |

### CAP-04-ANA-14 — Next analysis cache (current implementation)

| Field | Value |
| --- | --- |
| Name | Load/save/delete cached review frames for a game |
| Entry Points | Auto on open if `autoLoadCache`; auto after analysis if `autoSaveAnalysis`; `CacheStatusBadge` |
| Frozen baseline | N/A as SQLite. Java analogue is enable-lizzie-cache + SGF headers. |
| Defaults | autoLoad/autoSave true |
| Persistence | `analysis-cache.sqlite3`; browser localStorage prefix `lizzieyzy-next-analysis-cache` |
| Failure / recovery | Status error; incompatible payload message |
| Frozen evidence | N/A |
| Next evidence | `analysisCache.ts`; `lib.rs:701-736,1409-1502`; `App.tsx:1082-1193`; `preferences.ts:9-26` |
| Existing Parity Item | **ANA-05 Accepted** |

No Java **export-analysis-to-file** dialog was found in sampled files. `AnalysisTable.addFile` imports SGFs into a batch queue, not an analysis dump. Share/contribute are other domains.

### CAP-04-ANA-15 — Engine/protocol failure reporting

| Field | Value |
| --- | --- |
| Name | Show start/protocol/JSONL failures without leaking secrets |
| Entry Points | Start failure status; `EngineFailedMessage`; analysis settings fromError; KataGo JSON `error` field |
| Frozen baseline | Status CHECKING/NEEDS_REPAIR/START_FAILED. `EngineFailedMessage` redacts password/token/secret. Analysis JSONL `error` in protocol. ExactSnapshot restore categories: ADMISSION_STALE, UNSUPPORTED_REMOTE_POSITION, SEND_FAILED, GTP_ERROR, TIMEOUT, TAIL_REJECTED (implementation of restore, not a separate user Capability). |
| Defaults | N/A |
| Persistence | N/A |
| Failure / recovery | Click-to-repair; settings after missing command |
| Frozen evidence | `EngineFailedMessage.java:43-59`; `EngineStartupStatus.java:10-14,118-124`; `ExactSnapshotEngineRestore.java:33-40` |
| Current Next mapping | `katago-protocol` `ProtocolError::Engine`; `EngineManagerError` spawn/timeout/cancel/nonzero/missing stdout; events `katago://analysis-error`. Asset checks on demand. |
| Existing Parity Item | ENG-01 asset checks Accepted; protocol errors part of ANA-01/02 remaining gap (need controlled KataGo smoke). |

---

## 9. Implementation-only / not classified as abandon

Do **not** treat as abandoned. Record as non-Capability or cross-domain:

| Candidate | Why it is not a Capability |
| --- | --- |
| `EngineStartupStatus` / generation fences | Identity machinery |
| `EngineSwitchUiTracker` / switch tokens | Identity machinery |
| `InitialEnginePreloadScheduler` | Threading for CAP-04-ENG-09 |
| `AnalysisResourceCoordinator` | Process accounting/diagnostics |
| `ExactSnapshotEngineRestore` | GTP snapshot restore helper |
| `EngineFollowController` | Try-play follow; likely domain 03/05 |
| `HumanSlAnalysisRunner` | Training/human-SL; not inventoried as review analysis |
| Profile JSON schema | Persistence evidence |
| `ContributeEngine` | Domain 06/07 |
| Zhizi/remote compute catalog | Domain 06 |
| Engine-game occupancy/recovery | Domain 05 |
| Bundled KataGo assets/auto-setup | Domain 07 + first-run repair overlap |

---

## 10. Cross-domain dependencies and unowned entries

| Entry / behavior | Owner | Notes |
| --- | --- | --- |
| Menu/toolbar routing of engine and analysis items | 01 | This report lists the behaviors; 01 owns the census of *all* controls |
| Window/layout of MoreEngines/LoadEngine/AnalysisSettings | 02 | Dialog bounds/fonts |
| SGF tree navigation while analysis runs | 03 | Whole-game claims non-blocking navigation |
| Analysis stored in SGF comments | 03+04 | CAP-04-ANA-13 |
| Engine-game start/stop/pause/batch | 05 | `EngineGame*` ; do not duplicate |
| Remote SSH engines, Zhizi, Remote Compute startup remap | 06 | `useJavaSSH`, `RemoteComputeConfig.resolveStartupSelection` |
| Readboard ponder / GMA | 06 | `readBoardPonder`, tracking context |
| Bundled engine binary/model/config, first-run auto setup | 07 | `applyBundledKataGoDefaults`, REL-05 |
| No-engine desktop | 02/UI-04 | Already accepted; ENG empty-state still Missing |

Unowned by 04 if 01 does not list them: CLI `read` mode, smoke `-D` probes, GTP console visibility, contribute watch.

---

## 11. ENG / ANA / PREF overlaps

| Concern | Parity items | Domain 04 note |
| --- | --- | --- |
| Profile fields + asset check | ENG-01 Accepted | Successor if SSH/GTP/preload/komi/size kept |
| Foreground identity/start/stop | ENG-02 Missing | R3 standing applies |
| A→B switch | ENG-03 Missing | Java tracker already matches standing R3 promotion rule |
| Failed switch / stale token | ENG-04 Missing | Java tests exist; Next absent |
| Cancellable one-shot job | ENG-05 Partial | Java ponder ≠ Next one-shot |
| One-shot analysis | ANA-01 Partial | Bypasses manager |
| Whole-game batch | ANA-02 Partial | Missing two-stage/pause/lease |
| Cancel/supersede | ANA-03 Partial | Full-game only in Next |
| Candidates/PV/ownership/policy | ANA-04 Partial | Display exists; lifecycle binding incomplete |
| SQLite cache | ANA-05 Accepted | Not Java enable-lizzie-cache |
| Autoload / last-engine / analysis lifecycle prefs | PREF-01 Partial | Lifecycle-affecting prefs listed above; 02 should not re-own them |
| No-engine chrome | UI-04 Accepted | |
| Hover stale eviction | UI-03 Accepted | |

---

## 12. Standing R3 vs Java facts vs uncovered (do not decide)

| Topic | Standing R3 (Next redesign input) | Frozen Java fact | Uncovered (no silent extension) |
| --- | --- | --- | --- |
| Switcher vs Settings | Distinct; switcher starts/switches; settings do not change run | Menu/LoadEngine/ChooseMoreEngine start; MoreEngines edits catalog and **can** apply GTP live / `updateEngines` | Whether live GTP apply is equivalent or must be pending-restart |
| Autoload | Option, **defaults off** | Four-state; new bundled profile **autoload-default on**; missing keys = manual | Keep four-state vs boolean; first-run default |
| last-engine | Not mentioned | Written on shutdown if autoload-last | Keep iff autoload-last kept |
| Edit active profile | Pending changes; apply only on explicit restart | GTP profile may apply immediately | R3 vs Java live-apply |
| Crash | Report; **manual** restart | Repair status; quarantine; engine-game **does** auto-recover (05) | Foreground auto-restart via `autoCheckEngineAlive` |
| A→B | A primary while B starts; cancel A jobs; stop A; A stays if B fails | A stays in UI snapshot; ponder pause/restore; stale tokens ignored | Cancel-jobs vs pause-ponder |
| Stop | Dedicated main-workspace Stop | Shutdown/restart **menu** items + ponder button | Chrome placement |
| Delete active profile | Blocked until run stopped or other primary | Delete in MoreEngines has **no** running-engine guard in sampled handler | Whether R3 guard is new constraint |
| Asset check | Auto on start/switch; manual diagnostic | Repair status + bundled defaults; Next manual `检查资源` is diagnostic today | Auto-validate on start is R3, not current Next |
| Job/switch identity | Manager-owned | Java already has tokens/generations/leases | Next cutover |
| GTP vs analysis JSONL resident engine | Not decided in R3 doc | Foreground is GTP `Leelaz`; analysis is JSONL `AnalysisEngine` | Protocol of the R3 Foreground Engine |
| `EngineStartupBootstrap` | N/A | **Absent**; `EngineStartupStatus` is the status bus | Do not invent a Bootstrap type |

---

## 13. R3 vs later dependency list

**R3 (this domain’s lifecycle slice, using standing decisions as constraints):**

1. Foreground Engine identity snapshot (no-engine/starting/ready/switching/stopping/error) — ENG-02.
2. Start / dedicated Stop / explicit Restart applying pending profile edits — ENG-02 + standing R3.
3. Successful A→B and failed rollback + stale tokens — ENG-03/04.
4. Manager-owned cancellable one-shot job identity (so ANA-01 can bind) — ENG-05.
5. Autoload **as an option defaulting off** (standing R3). Mapping of Java four-state and last-engine is **blocked on uncovered disposition** (ticket 11).
6. Auto asset validation on start/switch; keep ENG-01 manual check diagnostic.
7. Lifecycle-affecting prefs **inventory only** until ticket 11: four-state autoload, last-engine, `analysisReuseCurrentEngine`, `analysisAutoQuit`, `enableLizzieCache`, `analysisEnginePreLoad`, per-profile GTP `preload`.

**Must not silently enter R3:** flash/part/branches, whole-game two-stage pause/resume, auto-ana, batch files, tracking, SGF analysis headers, HumanSL, SSH/remote, engine-game recovery, contribute.

**R4 (depends on R3 manager):** ANA-01 one-shot through manager; ANA-02 whole-game session/request (Java two-stage is uncovered extra); ANA-03 shared cancel; ANA-04 node/job-scoped candidates; variation-aware cache before claiming ANA-05 beyond MVP.

**PREF-01 / R5:** remaining analysis display prefs (estimate styles, delay candidates, PV visits, suggestion column order). Not lifecycle.

**R6:** engine-game sessions, batch limit, generated moves — JAVA_BASELINE `#365`.

**R7:** remote compute/SSH/Zhizi/readboard ponder.

**R8:** bundled KataGo resolution (REL-05) which Java first-run already uses for default profile.

---

## 14. Corrections vs invalid first-round reports

- Unique Java evidence is worktree `java-baseline-v1` at `7b4027531c2b26062d0bfc27a040cc550cfbea4d` only.
- Unique Next evidence is `migration-coverage-audit` at `18c6d189b8b01069975c4c40ead63a010249cb8c` only.
- **No** citation of `42c92e3` or of `/mnt/d/dev/weiqi/tmp/lizzie-java-baseline-r2a`.
- **No** `EngineStartupBootstrap` type/class in the frozen tree; previous reports that assumed that name as Java API are wrong. Use `EngineStartupStatus`.
- Autoload is **four-state**, not a single boolean; new bundled profiles default autoload-default **on**, which **contradicts** standing R3 default-off if treated as Java equivalence.
- `last-engine` is shutdown persistence for autoload-last, not a general “selected profile” analogue of Next `selected_profile_id`.
- `enableLizzieCache` is `leelaz.enable-lizzie-cache` in-tree node cache, not Next SQLite ANA-05.
- Profile format is persistence, not a Capability.
- Standing R3 is labeled as Next redesign input, not as frozen Java behavior.
- No product dispositions are made.
