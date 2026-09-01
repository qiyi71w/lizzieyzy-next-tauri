# Domain 04 — Foreground engine and analysis (independent census)

Status: complete
Owner: Ticket 04
Java tree: `/home/dev/dev/weiqi/worktrees/lizzieyzy-next/java-baseline-v1` (`7b4027531c2b26062d0bfc27a040cc550cfbea4d`)
Next baseline: `18c6d189b8b01069975c4c40ead63a010249cb8c`
Method: Domain 01 census — registered, user-reachable Entry Points only. Constructed-but-unadded widgets are not Entry Points. One observable user goal is one Capability; extra controls are Entry Points of that Capability.

## Sources

- Java `Menu`, `LoadEngine`, `MoreEngines`, `EngineManager`, `Leelaz`, `LizzieFrame`, `Config`, `ConfigDialog2`, `AnalysisSettings`, `ContributeSettings`, `AutomaticAnalysisSettings`, `AnalysisTable`, `SetAnalysisTable`, `AutoPlay`, `RightClickMenu`, `BottomToolbar`, `WinratePane`, `ContributeView`, `HawkEyeDialog`, `PlayerStrengthEstimator`, `EngineStartupStatus`, `EngineFailedMessage`
- Java tests cited per row under Frozen evidence
- Standing R3 (`docs/decisions/standing-r3-engine-lifecycle.md`) and ADRs `0001`–`0003` are Next product law, not Java census facts
- Ticket 03 owns SGF comment mutation / EXPORT; this census only records engine-produced comment data

## Method notes

- Capability = one observable user goal. Scan, encrypt, GTP profile fields, reorder, default-mark, LoadEngine chooser, overlay repair chip, lightning whole/part/all-branches, ownership/policy overlays, GTP console, and first-run bundled write are Entry Points or other-domain rows, not extra Domain 04 Capabilities.
- `EngineStartupBootstrap` is absent from this Java tree.
- Secondary Engine menu (`engineMenu2`) is ExtraMode / Domain 05, not Domain 04.
- Computed count first; reference 25 is comparison evidence only.

## Owner routing (not Domain 04 Capabilities)

| Surface | Owner |
| --- | --- |
| First-run bundled KataGo write (`Lizzie.ensureDefaultEngineReady`) | `SHELL-06` |
| Overlay engine-repair chip (`engineStartupStatusButton`) | `SHELL-07` (repair); Domain 04 start-failure presentation remains `CAP-04-ANA-15` |
| GTP console show/hide (`View` › `gtpPanel`) | Domain 02 `SET-LAYOUT-PANELS`; sending live GTP is abandoned with Ticket 11 live GTP |
| Next-move / winrate-graph / variation-board settings | Domain 02 `SET-NEXT-MOVE`, `SET-WINRATE-GRAPH`, `SET-SUBBOARD` |
| Autoplay variation replay | Domain 03 `CAP-03-SGF-03-ADJ-AUTOPLAY` |
| SGF comment (`C`) mutation, persistence, and export | Ticket 03 (`SGF-05`, `EXPORT-01`, `EXPORT-02`); Domain 04 only records engine-produced comment data |
| Match / PK / Contribute / ExtraMode dual engine | Domain 05 |
| Teacher / aiCoach | Domain 07 |
| Tsumego | Domain 06 |
| Hawk-eye (`T`) | `UI-05` (Ticket 19), not analysis |
| Player-strength report | Analyze menu is registered, but the dialog reads cached mainline estimates and does not start analysis; not a Domain 04 workflow |

## Computed Domain 04 Capabilities: 23

Reference 25 compared afterward. Extra Java rows vs 25: none. Reference-only rows that are not frozen Java Capabilities:

- Previous `CAP-04-ANA-05` “Java-closest one-shot” — Java has no named one-shot JSONL job. Next `ANA-01` is a Successor redesign of ponder + flash-analyze-this-move, not a Java census row.
- Previous `CAP-04-ANA-14` Next SQLite cache — no Java analogue. Next `ANA-05` stays the Accepted cache item and maps from `CAP-04-PREF-LIZZIE-CACHE` abandonment.

| Frozen ID | Name | User-reachable Entry Points | Default / persistence | Failure / recovery | Frozen evidence | Runtime-check |
| --- | --- | --- | --- | --- | --- | --- |
| CAP-04-ENG-01 | Edit saved engine profiles | Settings › `engineConfig` (`MoreEngines`); Scan; encrypt; SSH fields; GTP profile name/args/`initialCommand`; reorder; default checkbox; LoadEngine catalog is read-only of this store | New profile: blank command, `preload=false`, `isDefault=false`. Persist `engine-settings-list` / `engineSettings.txt`. Scan writes `.settings` then reload. | Scan/save/encrypt failures stay in the dialog. Empty command is allowed at save. Active-profile delete is unguarded in Java (`MoreEngines.delete` then `updateEngines`; missing running name → `killAllEngines`). | `MoreEngines.java:528-823,850-888`; `EngineProfileSettingsStore.java`; `LoadEngine.java:282-293`; `Menu.java:6108-6111`. Tests: `EngineProfileSettingsStoreTest`; `MoreEnginesSettingsPersistenceTest`. | Scan of a live directory; encrypt round-trip on a real password. |
| CAP-04-ENG-02 | Choose how the next launch starts an engine (four-state autoload) | LoadEngine radios `rdoLastEngine` / `rdoDefaultEngine` / `rdoNoEngine` / none; Settings last-engine checkbox is an Entry Point of `CAP-04-ENG-03` | Missing keys: all false (manual picker). Radios persist mutually exclusive `autoload-last` / `autoload-default` / `autoload-empty`. | Invalid saved index falls through EngineManager empty/failed start (`CAP-04-ENG-05` / `CAP-04-ANA-15`). | `LoadEngine.java:169-279`; `Lizzie.java:967-1011`; `EngineManager.java:529-690`. Tests: `EngineManagerInitialStartupSynchronizationTest`. | First-launch picker vs autoload when `engine-settings-list` is empty (also SHELL-06). |
| CAP-04-ENG-03 | Remember last-used engine and restore it on next launch | Settings › last-engine checkbox; LoadEngine `rdoLastEngine`; shutdown writes last index when the flag is on | Default off. Persist `autoload-last` plus last engine index. | Corrupt index → empty/failed start, not a different profile. | `Config.java` last-engine keys; `LoadEngine.java` radio persist; `Lizzie` shutdown persist. | Whether shutdown persist runs after a crash kill (source is normal shutdown). |
| CAP-04-ENG-04 | Start with no engine | LoadEngine `rdoNoEngine` / empty OK; `Lizzie.start(-1)` | `autoload-empty` or `index==-1`. Menu “no engine”. | Empty primary is success, not failure. | `LoadEngine.java`; `EngineManager.java` empty primary (`isEmpty=true`). | Overlay chip hidden when empty-by-choice vs start-failed. |
| CAP-04-ENG-05 | Start the chosen catalog profile as the primary GTP engine | LoadEngine OK / double-click → `Lizzie.start(curIndex, false)`; startup `EngineManager(Config, index, loadDefault)`; overlay chip click is SHELL-07 | Selected slot `preload=true`, `firstLoad=true`. Run does not persist across process restart. | Constructor exception → `engineStartupStatus.failed` + empty manager. Primary start → `NEEDS_REPAIR` or `START_FAILED`; overlay; `EngineFailedMessage`. Exclusive lease → `existing_lease`. | `LoadEngine.java:169-279`; `Lizzie.java:967-1011`; `EngineManager.java:529-690`; `EngineStartupStatus.java:10-42`; `Leelaz.java:21620-21661`; `EngineFailedMessage.java:106-131`. Tests: `EngineStartupStatusTest`; `EngineManagerInitialStartupSynchronizationTest`; `EngineManagerLifecycleReservationTest.initializationCommandFailureCannotPublishReadyOrQueueReadyUi`. | Overlay copy when `isActionable`; whether first-session suppresses the interactive diagnostic (`shouldOpenInteractiveDiagnostic`). |
| CAP-04-ENG-06 | Stop or restart the current engine | Engine › Restart (`reStartEngine`); Shutdown current / other / all (`killThisEngines` / `killOtherEngines` / `killAllEngines`). Space / toolbar analyse is ponder (`CAP-04-ANA-04`), not stop. | In-session. Stop current → `currentEngineNo=-1`, `isEmpty=true`. Restart restores ponder intent after 200 ms shutdown sleep. | Setup-mode reject. Lease in use → not killed. Empty primary restart no-ops. `forceKillAllEngines` is SHELL-08, not this menu. | `Menu.java:5734-5791`; `EngineManager.java:4025-4380`. Tests: `EngineManagerInitialStartupSynchronizationTest`; `Ticket07RestartBootstrapProductionEntryTest`. | User-visible difference of `forceQuit` vs `normalQuit`. Click Restart during SWITCHING (lease message). |
| CAP-04-ENG-07 | Switch from engine A to engine B | Engine menu `engine[0..19]`; overflow ChooseMoreEngine; Ctrl+1..9 / Ctrl+0. Title shows switching. | Last committed primary is in-memory. Autoload-last is not this control. | Same-slot dual-mode hint. Failed B: “Switch failed, rolled back”; recovered identity stays A (`CAP-04-ENG-08` for crash; this row for switch-B failure). Stale token success/fail ignored. Out-of-range no-op. Setup-mode reject. | `Menu.java:5621-5627,6932-6968,9884-10074`; `ChooseMoreEngine.java:150-164`; `Input.java:768-818`; `EngineManager.java:10344-10426`. Tests: `MenuEngineSwitchUiStateTest`; `EngineSwitchUiTrackerTest`; `EngineManagerLifecycleReservationTest` switch reservation / frozen catalog. | Live key-repeat while SWITCHING. |
| CAP-04-ENG-08 | Recover when the current engine exits unexpectedly | No dedicated button. 5 s timer `autoCheckEngineAlive` → `checkEngineAlive`. Settings `chkCheckEngineAlive` toggles live. | Default **on**: `uiConfig` `auto-check-engine-alive` fallback **true**. Skipped if empty, setup mode, or engine-game occupying. | Dead primary → Java **automatic** local restart or remote restart in background. SSH `javaSSHClosed` → same auto-restart. Next `ENG-07` redesign waits for explicit Restart and does not auto-restart. | `EngineManager.java:685,1251-1269,3248-3272`; `Config.java:1719`; `ConfigDialog2.java:6104-6106`. Tests: `LeelazAutomaticRestartConvergenceTest`. | Timer while the window is backgrounded. Remote-compute restart UX (Domain 06 / `RCOMP-01`). |
| CAP-04-ENG-09 | Preload extra (non-primary) catalog GTP engines | Not a menu click. Per-profile `MoreEngines.preload`; EngineManager starts every non-selected `preload==true` on `lizzie-initial-engine-preload-N`. | Default false on new profile. Persisted with engine settings. | Scheduling failure rethrows. Preloaded engines are not promoted to primary. | `EngineManager.java:632-710`; `EngineData.java:9`; `MoreEngines.java:528,595,823`. | Whether a preloaded extra appears ready in the Engine menu before any switch. |
| CAP-04-ANA-01 | Reuse the current engine for analysis | `AnalysisSettings.chkReuseCurrentEngine` (added to the analysis settings dialog); used by whole-game lease handoff. Not a separate Analyze-menu workflow. | Default **false**. Persist `ui.analysis-reuse-current-engine`. | Failed save leaves runtime flags and pending flash request unchanged. Exclusive lease blocks deferred HumanSL start. Empty primary → analysis unavailable. Next absorbs reuse into manager-owned run + capability snapshot (`ANA-03` / `ENG-10`). | `Config.java:1037,1826`; `AnalysisSettings.java:86-88,219-224,492-527,644-721`; `WholeGameAnalysisSession.java:189-207` | Not required. |
| CAP-04-ANA-02 | Preload a dedicated analysis engine | Settings › `chkPreloadLizzieCache` plus analysis-engine command/preload in ContributeSettings / AnalysisSettings | Separate analysis `Leelaz` when preload is on. | Start failure is analysis-engine specific; primary stays up. Next forbids a second hidden analysis process. | `ContributeSettings.java`; `AnalysisSettings.java`; analysis-engine preload fields. | Whether the dedicated process is visible in Engine menu (source: separate from catalog table). |
| CAP-04-ANA-03 | Auto-quit the dedicated analysis engine | Same settings: quit after analysis / keep alive | Persist with analysis-engine settings. | Process exit after job. Next: job completion must not kill the resident run. | `ContributeSettings.java`; `AnalysisSettings.java`. | Not required. |
| CAP-04-PREF-LIZZIE-CACHE | Cache analysis in the Java tree | Settings `chkPreloadLizzieCache`; analysis writes beside the SGF | Default follows Java settings. Next `ANA-05` SQLite is the only accepted runtime cache. | Corrupt cache is a Java-tree concern; Next does not migrate it. | `ContributeSettings.java` / analysis cache settings; Ticket 11 Java-tree cache evidence. | Not required. |
| CAP-04-ANA-04 | Interactive pondering (continuous analysis of the current node) | Space `togglePonderMannul`; BottomToolbar `analyse` (default visible `Config.analyse=true`); detail `stopGo`; Game › breakGame; FloatBoard Space. `Menu.pondering` is declared and **not added** — not an Entry Point. | Ponder in-memory. `playponder` default true is play-mode, not this toggle. | `stopAiPlayingAndPolicy` can consume the action. Null `leelaz` no-op. Manual start syncs kifu first. Process stays up (`nameCmd` vs `ponder()`). | `Input.java:419-421`; `LizzieFrame.java:12357-12367`; `BottomToolbar.java:81,226,1258-1264,1584-1593`; `Menu.java:83,3034-3043`; `Leelaz.java:21113-21137`. Tests: `LizzieFrameRegressionTest.manualPonderStartSyncsCurrentKifuBeforeAnalysis`; `LeelazPonderStateTest`. | Confirm `Menu.pondering` is never assigned after `doubleMenu` (none found through game-control construction). |
| CAP-04-ANA-06 | Lightning analysis | Analyze › `flashAnalyzeGame` / `flashAnalyzePart`; right-click `flashAnalyzeThisMove`; Analyze › `flashAnalyzeSettings`; `chkAutoAnaAfterReopen` / `chkAutoAnaNewGame` (silent load-flash is this Capability, not a new row) | Persist flash settings (max-move, time, threads, first-ponder). Auto-on-load is a setting. | Cancel via `flashMode` / `tryStopFlashModeForManualGame`. Load-flash is silent (no dialog). | `Menu.java:3280-3347,3378-3385`; `RightClickMenu.java:185-191`; `LizzieFrame.java:12599+,12720+,12755+`; `Config.java:2693-2700`. Tests: `LizzieFrameRegressionTest` flash/load-flash. | Whether load-flash runs on a file without a catalog engine. |
| CAP-04-ANA-07 | Whole-game deep analysis | Analyze › `analyzeGame` → `AnalysisSettings` (two-stage, `estimateAz` / `estimateLz`) | Dialog defaults; not a one-click. | `waitForAnalysisEngineReady` timeout; `analysisEngine.startEngineAnalysis`. | `Menu.java:3273-3278`; `AnalysisSettings.java:40-47,80-84`. | Timeout copy when the analysis engine never becomes ready. |
| CAP-04-ANA-08 | Automatic current-game analysis | Analyze › `autoAnalyze` → `AutomaticAnalysisSettings` | Start/end/time/play-mode persisted in that dialog. | Stop control in the dialog. | `Menu.java:3350-3352`; `AutomaticAnalysisSettings.java:23`. | Not required. |
| CAP-04-ANA-09 | Batch file analysis | Analyze › `batchAnalysis` (GTP) / `batchFlashAnalyze` (lightning); `AnalysisTable` / `SetAnalysisTable` | Queue in `AnalysisTable`. GTP vs lightning are modes of this Capability. | Stop current / stop all (`shutdownEngine` / `stopAllAnalyzing`). | `Menu.java:3354-3376`; `AnalysisTable.java:84-91,328-350`; `SetAnalysisTable.java:155-161`. | Queue progress while another engine occupies the primary. |
| CAP-04-ANA-10 | Track / untrack points for continuous analysis | Right-click `trackPoint` / `untrackPoint` / `clearAllTracked` | In-session tracked points. | Clearing tracking resumes ordinary ponder. Next abandons tracking; hover (`UI-03`) is not tracking. | `RightClickMenu.java` track items; `Leelaz` tracking pause/resume. | Not required. |
| CAP-04-ANA-11 | Show candidates, PV, ownership, and policy on the board | Board paint of engine suggestions; Analyze › `showPolicyHeatmap`; hawk-eye is UI-05, not this row | Persist heatmap/suggestion display settings. Ownership/policy are presentation of the same analysis publication. | Empty / failed engine → no overlay. | `LizzieFrame` suggestion paint; `Menu.java:3388-3396`; `WinratePane` related display. | Heatmap vs candidate interaction at runtime (ADR 0002 is Next law). |
| CAP-04-ANA-12 | Cancel in-flight analysis and ignore stale results | Flash stop; batch stop; ponder toggle off; node change supersede; engine switch/stop cancels in-flight work | In-session. Java generation / occupancy fences are machinery of this goal. | Stale token / superseded completion must not publish on a different engine or node. Switch FAILED keeps identity A. | `ForegroundAnalysisPause.java`; `EngineSwitchUiTrackerTest`; `MenuEngineSwitchUiStateTest`; flash/batch stop paths. | Live Ctrl+digit while SWITCHING (also ENG-07). |
| CAP-04-ANA-13 | Persist analysis into SGF headers / append WR to `C` | Analysis completion writes headers; append-WR-to-comment setting | Persist with SGF when the user saves. Engine-produced comment **data** is this row. **Mutating `C`, persistence, and export** are Ticket 03 (`SGF-05`, `EXPORT-01`, `EXPORT-02`). ADR 0003: Next will not write engine text into `C`. | Save failure is Domain 03. | Analysis SGF header writers; append-WR setting. | Not required for engine-produced data. Ticket 03 runtime-checks comment persistence. |
| CAP-04-ANA-15 | Show engine/protocol failure to the user | Overlay chip; `EngineFailedMessage`; menu FAILED / rolled-back; `existing_lease` | In-session status. | Typed Java statuses: `NEEDS_REPAIR`, `START_FAILED`, switch FAILED, lease conflict. Recovery is Restart / repair chip / LoadEngine, not Settings. | `EngineStartupStatus.java`; `EngineFailedMessage.java`; `Menu.java` FAILED snapshot. Tests: `EngineStartupStatusTest`; `EngineSwitchUiTrackerTest`. | First-session diagnostic suppression (also ENG-05). |

## Explicit non-rows

- Settings live `setRules` / `setLzSaiEngine` / engine parameters: Ticket 11 abandoned live GTP; they are Entry Points of `CAP-04-ENG-01`, not new Capabilities.
- Remote Compute Center: Settings Entry Point of `CAP-04-ENG-01` remainder → Deferred `RCOMP-01` (Ticket 23). Not a 24th Domain 04 census row.
- SSH profile fields: Entry Points of `CAP-04-ENG-01` remainder → Deferred `SSH-01`. Not a new census row.
- GTP console, Teacher LLM, Match/PK/Contribute, ExtraMode secondary engine, hawk-eye, player-strength, tsumego, first-run write, overlay chip ownership: routed above.
- Switch tokens, reservations, `ExactSnapshotEngineRestore`, `ForegroundAnalysisPause`, catalog file format: machinery.
- `Menu.pondering` unadded field: not an Entry Point.
- Previous inventory `CAP-04-ANA-05` (Next one-shot) and `CAP-04-ANA-14` (Next SQLite): not Java Capabilities.

## Ticket 11 / 23 / 03 mapping (product law, not census)

Preserve Accepted `ENG-01` and `ANA-05`. Keep Deferred `ENG-08`, `ANA-06`–`ANA-09`, `SSH-01`, `RCOMP-01`. Analysis-owned `ANA-10`–`ANA-13` stay on analysis; they do not expand engine start.

| Frozen ID | Next mapping |
| --- | --- |
| CAP-04-ENG-01 | `ENG-01` local catalog; SSH remainder `SSH-01`; Remote Compute remainder `RCOMP-01`; live GTP / `initialCommand` / default-index / per-profile preload abandoned (`ENG-09` covers preload) |
| CAP-04-ENG-02 | `ENG-06` Autoload Default (zero or one; first run unmarked) |
| CAP-04-ENG-03 | Abandoned (no last-run autoload) |
| CAP-04-ENG-04 | `UI-04` empty-board; `ENG-02` no-engine run |
| CAP-04-ENG-05 | `ENG-02` start selected as primary |
| CAP-04-ENG-06 | `ENG-02` dedicated Stop + explicit Restart |
| CAP-04-ENG-07 | `ENG-03` + `ENG-04` |
| CAP-04-ENG-08 | `ENG-07` typed failure, **wait for explicit Restart** (Java auto-restart does not migrate) |
| CAP-04-ENG-09 | Abandoned hidden extra processes; switch and Match Reservation may keep a process |
| CAP-04-ANA-01 | Absorbed: jobs use the manager-owned run + capability snapshot |
| CAP-04-ANA-02 | Absorbed: no second hidden analysis process |
| CAP-04-ANA-03 | Absorbed: job completion does not kill the resident run |
| CAP-04-PREF-LIZZIE-CACHE | Abandoned; `ANA-05` is the only runtime cache |
| CAP-04-ANA-04 | `ANA-06` Deferred continuous; Next `ANA-01` is the supported selected-node path (Successor, not a Java row) |
| CAP-04-ANA-06 | Abandoned; `ANA-01` / `ANA-02` are the supported paths |
| CAP-04-ANA-07 | `ANA-02` single-stage mainline |
| CAP-04-ANA-08 | Abandoned |
| CAP-04-ANA-09 | `ANA-07` Deferred |
| CAP-04-ANA-10 | Abandoned; do not treat `UI-03` hover as tracking |
| CAP-04-ANA-11 | `ANA-04`; `UI-03` hover remains frozen |
| CAP-04-ANA-12 | `ANA-03` lanes + `ENG-05` identity |
| CAP-04-ANA-13 | `ANA-08` Deferred; ADR 0003 no `C` write; Ticket 03 owns `SGF-05` / `EXPORT-01` / `EXPORT-02` |
| CAP-04-ANA-15 | `ENG-07` |

## Standing R3 / ADR reminders (Next, not Java)

- Switcher ≠ Settings; Settings never starts or switches a run.
- Autoload Default off until explicit mark; first run has no mark.
- Editing the active profile is pending until Restart.
- Unexpected exit waits for explicit Restart (Java auto-restart is not Next).
- A stays primary until B is ready; ready B promotes in one transaction and must replace A.
- Dedicated Stop. Active profile cannot be deleted.
- ADR 0001 one published score/winrate series; ADR 0002 heatmap on main board; ADR 0003 no engine text in `C`.
