# R7 — Adaptive Workspace Execution Plan

Status: exited within the approved R7 acceptance scope. All seven owner items are Accepted; ticket07 is CLOSED/ACCEPTED. Windows A01–A18 pass18/18, required repository checks pass, and final bounded Standards/Spec reviews are CLEAN on `406250408043e68fe67f6082c583429600768c0c`. macOS native remains user-approved SKIPPED (2026-09-30), not PASS or a support change. Original native `baa9747…` and affected-document `f4e478e…` evidence retain their exact source attribution. Ticket08 is unblocked but has not run; no later feature phase starts here.

## Baseline and authority

- Remote fetched: `https://github.com/qiyi71w/lizzieyzy-next-tauri`, `origin/main` at `5d44760f172867c520adb9c57adcd86678f72c54` (`feat(review): 完成 R6 作者工具与本地复盘迁移 (#13)`).
- Planning branch: `docs/r7-migration-plan`.
- Independent checkout: `/home/dev/dev/weiqi/worktrees/lizzieyzy-next-tauri/r7-planning-20260926`.
- The original checkout remains on `wip/main-before-migration-audit`; its modified/untracked files were not transferred or edited.
- Java reference remains Migration Baseline v1 `7b4027531c2b26062d0bfc27a040cc550cfbea4d`; this plan uses its frozen [capability inventory](JAVA_CAPABILITY_INVENTORY.md), not the sibling's latest HEAD.
- Current main records R6 exited at Windows candidate `be6951bf57afa07c4a1ae6d57075573b308f5dfd`. Historical local R6 foundation checkpoints are not this plan's baseline.
- [Architecture](ARCHITECTURE_NEXT.md) remains authoritative for module/DTO/command ownership; [Migration Plan](MIGRATION_PLAN.md#r7--adaptive-workspace) owns phase scope/order; [Parity Matrix](PARITY_MATRIX.md#r7--adaptive-workspace) owns status and Item Start Prerequisites.

## Deliverables and approval boundary

1. [Complete specification](../.scratch/adaptive-workspace-r7/spec.md): behavior, state, persistence, concurrency, resets, native geometry, DPI, errors and non-goals.
2. [Implementation tracker](../.scratch/adaptive-workspace-r7/tracker.md): six feature slices, one integrated acceptance gate, one read-only Closeout, each published as an individual issue.
3. This plan: source map, preservation delta, delivery DAG, native acceptance cases and execution rules.

Historical publication: all eight tickets were published as `ready-for-agent`;01 and05 were the initial frontier. Implementation is now integrated and applicable native/repository checks pass. Ticket07 records final review and owner/phase seal;08 remains the separate read-only Closeout. Prerequisite edges and cold-read publication evidence are unchanged.

## Preserved scope and delta

| Item | Preserve | Added execution detail |
| --- | --- | --- |
| LAYOUT-01 | Left/board/right adjustment and practical minimums | Pointer capture, focused-key operation, ratio projection, resize-aware Canvas and rectangular board checks |
| LAYOUT-02 | Immediate changes, debounced atomic save, visible failed-save retry, restart | 500 ms coalescing, committed/pending separation, owner-scoped merges and bounded exit flush |
| LAYOUT-03 | Restore panel sizes and board proportion only | One action in menu/Preferences, works while rails hidden, uses layout durability |
| LAYOUT-04 | Two visible-by-default independently durable rails | Release grid space, preserve drafts/replay semantics, migrate the current transient right-rail path |
| WINDOW-01 | OS/Next first-use default, valid saved geometry, invalid-only auto-recovery, geometry-only reset | Units, normal/maximized/minimized, work-area validation, DPI, startup and exit fences |
| WINDOW-02 | Default off, main only, durable value after failure, no Ctrl+Z takeover | Native apply/persist/rollback failure contract and visible retry |
| APPEAR-01 | Existing curated pair, durable choice, system DPI | Reuse existing theme persistence; readable controls, DPR redraw and exact-candidate DPI evidence |

No owner, accepted historical evidence, excluded capability or prerequisite is dropped. `GUIDE-01` remains Deferred; R7 does not promote later phases. `WINDOW-01` continues to depend on `LAYOUT-02`; `LAYOUT-04`, `WINDOW-02`, and `APPEAR-01` do not gain new Matrix dependencies merely because implementation files overlap.

## Current-source evidence and placement

Paths below are investigation anchors at the pinned baseline, not a request to grow the gateway or App monolith. Executors locate current definitions and references before changes.

| Concern | Source anchor | Evidence / intended seam |
| --- | --- | --- |
| Current rails and defaults | `apps/desktop/src/styles.css:39-43,445-463,1316-1328` | body min 1100×720; fixed 228/260, narrow 210/240; theme variable override. Replace only layout width ownership; preserve default appearance. |
| Workspace composition | `apps/desktop/src/App.tsx:3320-3655` | AppChrome, three-column spread, separate task area/BottomBar/Preferences. A bounded workspace module owns projection/drag; parent retains SGF/analysis ownership. |
| Transient right rail | `apps/desktop/src/App.tsx:270,520-521,3373-3374,3509-3537`; `components/AppChrome.tsx:239-241` | referenceRailCollapsed also gates replay and selection. Cut over every consumer; this is not a new unrelated toggle. |
| Canvas sizing | `components/BoardCanvas.tsx:200-209,423`; `components/WinrateChart.tsx:29-38,108`; `components/AnalysisPanel.tsx:310-320,384` | Canvas reads client size/DPR during content-dependent effects; size-only/DPI-only changes need measured redraw and shared hit geometry. |
| Existing theme | `components/PreferencesPanel.tsx:129-133`; `src/domain/preferences.ts`; `src/api/preferences.ts` | boardTheme already persists through normal preferences; do not introduce a duplicate theme field/store. |
| Preference file owner | `crates/app-preferences/src/lib.rs` | Existing defaults, validation, unreadable-file isolation and replace-safe persistence. Extend, do not replace. New shared wire structures begin in `crates/app-model`. |
| Preference transaction | `apps/desktop/src-tauri/src/continuous_analysis.rs::PreferencesState` | Existing mutex and preservation of recent_game_paths on full saves. Add bounded R7 field updates under this same transaction; pure preference/store behavior remains in crates. |
| Frontend write ordering | `App.tsx::queuePreferencesSave`, `runPreferencesSaveLoop`, `handlePreferencesChange` | Existing committed/pending queue and analysis admission protections. R7 continuous drafts must not roll back on write failure or be overwritten by stale full snapshots. |
| Native main window | `apps/desktop/src-tauri/tauri.conf.json:16-24`; `src-tauri/src/lib.rs::run` | Configured 1440×900, resizable, no native min-size; current events handle close/drop, not geometry/always-on-top. Extract window OS adaptation into a gateway module, not domain parsing/storage in lib.rs. |
| Native activation and exit | `src-tauri/src/file_activation.rs`; `src-tauri/src/document_departure.rs`; `App.tsx::handleApplicationExit` | Preserve focus/unminimize, activation intake, dirty decisions and confirmed teardown. Flush R7 writes before final native exit. |
| Test seams | `src/App.preferences.test.tsx`, `src/api/preferences.test.ts`, preference crate tests, existing departure tests | Extend consumer-visible failure/concurrency/restart behavior, not structural/source-text checks. |

The worktree's CodeGraph and code-review-graph were initialized at baseline. They are local indexes, not delivery artifacts. No source changed during planning; future implementation must refresh indexes before graph-assisted review.

### Exercised baseline evidence

A Vite browser preview was launched from this checkout on loopback port 1437; isolated Chromium exercised the existing Display → Panels → candidate-list panel toggle and viewport resize. No native API or user profile was used.

- At 1440×960 CSS viewport, grid columns were `228px 952px 260px`.
- After disabling the right-panel entry, the right aside had `hidden=true` but computed `display:flex`, width `260`, and the same grid allocation; the screenshot still showed the right reference surface. LAYOUT-04 must fix actual presentation, not just persist the existing flag.
- After resize-only to 1100×720, grid was `210px 650px 240px`. Main Canvas CSS size was 261×261 while its bitmap remained 705×705; chart CSS width was 170 while bitmap width remained 203. Browser DPR reported 1.25. These are observed stale backing sizes, not a native-DPI verdict.
- Screenshots were inspected during the run. Preview was stopped and the managed tab closed. The native window, multi-monitor, restart and system-DPI cases below are **NOT RUN** in this planning session.

## Delivery DAG and ownership

| Ticket | Delivers | Blocked by | Parity owner |
| --- | --- | --- | --- |
| 01 | Live draggable workspace and measured rendering | None | LAYOUT-01 |
| 02 | Durable proportions, visible Retry and narrow panel restore | 01 | LAYOUT-02, LAYOUT-03 |
| 03 | Independent durable rail visibility | 01 | LAYOUT-04 |
| 04 | Native window geometry/recovery/reset | 02 | WINDOW-01 |
| 05 | Transactional main-window always-on-top | None | WINDOW-02 |
| 06 | Curated appearance and DPI completion | 01 | APPEAR-01 |
| 07 | Integrated exact-candidate R7 acceptance and owner evidence | 03, 04, 05, 06 | All seven |
| 08 | Read-only review-follow-up Closeout | 07 | No new parity owner |

01 supplies the measured workspace and projection used by 03/06. 02 supplies the continuous-save and exit-flush contract needed by 04. These are implementation-ticket prerequisites, not changes to canonical Matrix edges. 07's terminal blockers transitively cover 01/02. 08 consumes records from **every** ticket 01–07, not only terminal nodes.

Execution follows the DAG: after 01, tickets 02/03/06 may proceed; 04 waits for 02; 05 has no ticket prerequisite; 07 waits for 03/04/05/06. Prefer the roadmap's delivery order when integrating, but it creates no extra start gate. Shared-file serialization is a merge constraint, not an invented product dependency.

- One integration owner controls shared DTO/TS copies, preference transaction, App/AppChrome, CSS shell and exit integration.
- Independent work may develop behind these frozen interfaces; do not have writers edit the shared same file simultaneously. Native window adaptation and appearance can be separate slices; merge through the owner.
- Every feature slice is demonstrated end-to-end on its required surface before its completion record. A repository pass alone does not close a native case.
- Follow the project's implementation flow: focused behavior checks, actual smoke, Standards + Spec review against a fixed point, then commit when authorized. Record complete review results or explicit zero-follow-up verdict for Closeout.
- Remove the obsolete fixed-column override and transient rail owner at the cutover. Keep browser fallback explicitly non-authoritative for window commands.

## Acceptance plan

The approved cases below retain their original requirements. Ticket07 now records each result against final Windows candidate `baa9747d11ee9e2130904ff057adbe7cfd968153`: 14 PASS and 4 BLOCKED. Each record distinguishes expected/actual, fixture scope, inherited evidence, source SHA, artifact, OS, monitor topology, scale, profile, PID and evidence paths. A screenshot alone does not prove persistence or OS z-order.

| Case | Scenario and observable result | Owning ticket | Required surface |
| --- | --- | --- | --- |
| A01 | First use: default proportions, both rails, Classic, pin off; missing R7 fields preserve old analysis/display/Recent settings | 01/02/03/05/06 | Fixtures + native fresh isolated app-data |
| A02 | Drag each divider to both extremes; pointer release outside/cancel; keyboard adjustment consumes its arrows; no accidental stone or navigation | 01 | Browser + native pointer |
| A03 | Resize at unchanged SGF/frame; canvases redraw at new size/DPR; points/hover/chart selection align; 19×19, 9×9 and rectangular 13×9 SGF | 01/06 | Browser measurements/screenshots + native |
| A04 | Two rapid adjustments while write A is pending; B is the final layout; old success/failure cannot clear B's pending state; 500 ms debounce coalesces | 02 | Deterministic async tests + native restart |
| A05 | Isolated write denial: session layout usable, durable file unchanged, visible Retry saves newest value; no save on every pointer frame | 02 | Temporary-dir failure + native permission fault |
| A06 | Toggle each rail and both; hidden content loses grid space/focus; reopen from fixed toolbar; restart restores; comment draft and selected branch survive | 03 | Browser + native restart |
| A07 | Variation Replay with right rail hidden: continue on eligible main surface; hide last eligible surface to pause; restore at retained prefix, no new identity | 03 | Rendered App + native representative PV |
| A08 | Change all R7 and representative engine/analysis/review settings; Restore panel sizes changes only proportions, including when rails hidden | 02/07 | State comparisons + native restart |
| A09 | Move/resize normal window, restart; maximize, restart, unmaximize returns prior normal bounds; minimize does not poison saved bounds or hide next launch | 04 | Native OS state + restart |
| A10 | Negative-coordinate secondary screen remains valid; remove that screen or use an off-screen isolated saved record to recover; valid partly off-screen reachable title stays put | 04 | Monitor fixtures + native changed topology / invalid record |
| A11 | Small work-area and DPI changes: fit without inaccessible title/tools; no auto-reset merely because primary monitor/hostname changes; monitor-query failure preserves durable record | 04/06 | Fixtures + native system DPI/topology |
| A12 | Explicit window reset unmaximizes/fits/centers only geometry; proportions, rails, pin, theme and SGF remain unchanged | 04/07 | Native state + preference comparison |
| A13 | Pin on/off with another application covering main; restart restores; disk-write denial restores old native flag; after apply/rollback/startup failure query actual flag and distinguish changed, unchanged and unreadable state; Retry targets the old durable value | 05 | Native z-order + isolated fault; deterministic adapter faults cover all three readback outcomes |
| A14 | Theme switch success/failure/restart; menu, input, dialog, rail, error and focus are readable; system DPI at ≥2 real scales plus dynamic/cross-screen change | 06 | Browser contrast inspection + Windows DPI |
| A15 | Rapid drag then Close/File Exit; pending save flushes; fault blocks before document departure; retry/cancel remains usable; ordinary SGF Save/Discard/Cancel intact | 02/04/07 | Controlled ordering + native dirty-document exit |
| A16 | Concurrent geometry/layout, theme/coordinates, Recent update and analysis budget save retain all committed fields; no stale full-response rollback | 02/03/04/05/06 | Serialized owner/consumer tests + native rapid sequence |
| A17 | Ready real KataGo with current-node and current-game task: adjust layout/rails/pin/theme/window without changing semantic identity or cancelling work; no-engine edit/Save still works | 07 | Real KataGo + native SGF round-trip |
| A18 | Cold/warm file activation and drop after restored window: same single main window and replacement/recovery gate; reset/pin did not alter these routes | 04/07 | Native activation/drop |

### Native candidate procedure

Use the existing `prepare-windows-candidate` helper with the **exact committed implementation SHA**, project `lizzieyzy-next-tauri`, source integration checkout and a fresh independent destination. Read `candidate.json`, then launch with that candidate's `windows-desktop-session.ps1`, unique run directory and isolated acceptance app identifier/profile. Record the override; it is source-candidate acceptance, not Canonical Artifact installation proof. Do not modify the daily Windows clone or user's app-data.

Run one supervised candidate at a time. Restart scenarios reuse the same isolated durable data intentionally; unrelated fault scenarios get separate data. Genuine multi-monitor/DPI changes and z-order observations need native system evidence; lack of the required display environment leaves the corresponding case unaccepted. Do not replace a native check with browser zoom.

### Verification and final exit

- During feature work use focused crate/consumer tests and the changed path's actual smoke; parent owns checks, writing agents skip mid-flight builds/tests/formatters.
- At final integrated gate follow [DEVELOPMENT.md](DEVELOPMENT.md): scaffold/release-asset validators, frontend build, workspace tests/Clippy and formatting **check** where the project requires them. No repository-wide formatting rewrite.
- 07 updates architecture/DEVELOPMENT/changelog for implemented contracts and each Matrix row with both repository and native evidence. Only all seven Accepted plus A01–A18's applicable evidence permits R7 exit. A partially available environment is not phase completion.
- 08 only reconciles final review records and existing follow-up ledgers; it cannot create acceptance, fix code, broaden review or promote a missing owner.

## Planning validation record

- `python3 scripts/validate_scaffold.py --verbose`: 10 passed, 0 failed.
- `python3 scripts/validate_release_assets.py --verbose`: 4 passed, 0 failed.
- Post-publication documentation check: all eight issues are `ready-for-agent`; their approved bodies and the specification's behavioral sections are unchanged. All 42 relative links across 11 planning Markdown files resolve, line endings are LF, and no obsolete draft links remain. Parsed blockers match the tracker; the DAG is acyclic, 07 transitively covers 01–06 and 08 consumes all 01–07 records. The eligible frontier is 01/05. Both validators above passed again after publication.
- Independent ticket-readiness cold read: 01–08 individually PASS; one scheduling sentence needed clarification so 03/06 do not falsely wait for 02. The sentence now follows the existing DAG without changing any edge.
- Independent scope-preservation cold read: all seven owners, Matrix prerequisites, narrow resets and exclusions preserved. One failure-reporting clarification required native pin readback (or an explicit unknown state) after a setter error; spec D7, ticket 05 and A13 now cover it.
- The preceding entries are historical planning checks. Implementation and native integration results are recorded separately below; planning readiness is not acceptance.

## Integrated verification record

- Source candidate: `baa9747d11ee9e2130904ff057adbe7cfd968153`; isolated Windows build `D:\dev\weiqi\worktrees\lizzieyzy-next-tauri\r7-integrated-baa9747\candidate.json`, identifier `org.lizzieyzy.next.acceptance.r61dc6694855c4defab9face18ecc83f0`. Runs intentionally reuse only that private profile. No release packaging or installed-artifact claim.
- Six predecessor worktrees were merged into the planning branch. Integration repaired omitted legacy DTO defaults, all-owner exit drain/freeze, and Windows pin cache-based false success. The latter now applies checked Win32 state and reads the actual flag before persistence.
- Repository checks: scaffold 10/10 and release assets 4/4; Rust workspace 592 passed / 2 ignored on the integrated legacy repair; Linux workspace clippy passed. Frontend build and full suite 368/368 passed after exit repair. Subsequent Windows-only pin changes passed Linux pin tests 3/3 and desktop-lib clippy; final Windows candidate built and pin tests passed 4/4. These are attributable runs, not a claimed full rerun on each SHA.
- Authorized repository follow-up `f4e478ea15781b7a84ee1ec27a0f74683e909e2e`: formatting-only changes in five files and a classifier import scoped to the existing non-Windows Save As helper. `cargo fmt --all --check`, affected-owner Linux clippy and strict Windows lib/tests clippy pass. SGF94 and Recent4 tests pass; Windows pin tests4/4 pass. The fresh isolated native document smoke passes five actual startup/Open/navigation/Open Cancel/Save As/reopen/exit cases on PID3200, with exit0; its declared exclusions remain unchanged. Evidence: `D:\dev\weiqi\acceptance\r7-gates-f4e478e-native-smoke-2\smoke.json`.
- Original source review remains SUCCESS / CLEAN on tree `37dd0f64aa1a0b9a67f7662884ca65d2ae84bb15`, exact object `baa9747…`; all four admitted R7 findings resolved. The follow-up's bounded Standards/Spec reviews are CLEAN on tree `70cfc1b51bcebd46330839100ec71154a6d0bafe`, fixed HEAD `5f05737b7466e50c8b202f3639b5cfe0cfbabbd2`, exact source object `f4e478e…`. Both historical handoffs are resolved with full envelopes retained. Subsequent desktop-lib failures were reproduced in parallel and serial runs, then repaired only in controlled fixtures/Unix test setup. Final default-parallel and serial runs each pass146/146, shared-fixture analysis-task checks pass20/20 (107 filtered), formatting check and Linux strict desktop lib/tests clippy pass; `artifact://678` / `artifact://677`. Production analysis behavior and the attributed native binary are unchanged. Final bounded Standards/Spec FULL_REVIEW is CLEAN on exact object `406250408043e68fe67f6082c583429600768c0c`, tree `266dcc14e3bb993d982fa76a6c869441e81a9137`; parent confirmed the frozen target unchanged. Zero open IN_SCOPE, zero open DEFERRED and zero new follow-up items.
- Final Windows native results: A01–A18 PASS: 18 PASS / 0 BLOCKED. A11 completes an authorized temporary native `1280×720 @180Hz` mode change with work area `1280×672` at DPI96: the window fits at `0,0`, outer `1280×672`, client `1264×633`; page/workspace/left-rail scrolling reaches controls without hiding either rail or changing SGF. Restart retains the full preference document, both runs exit0, and actual mode/registry readback confirms restoration to `2560×1440 @180Hz`. `r7-integrated-baa9747-small-auto/A11-continuation-native-final-verdict.json` links the measured records and inspected surfaces. A10 passes actual DISPLAY2 negative-coordinate movement/restart at `-2210,64 / 1145.3333333333333×722.6666666666666 @1.5` and recovery after Windows “PC screen only” removes that output: `552,226 / 1440×900 @1`, with only `windowGeometry` changed and normal exit0. `A10-continuation-native-final-verdict.json` links those records. A14 includes final run7's same-process system DPI96→144→96, both themes, keyboard focus, native K10 hit and exact state retention; supplemental125%/200% CDP layouts are not native system-DPI evidence. A18 passes actual Explorer dirty Cancel/Discard and exact9×9/two-variation import while retaining the same single PID13212/HWND264354. This completes native acceptance within the approved scope; required repository checks now pass; final fixture-review seal is recorded in ticket07.
- Evidence root: `D:\dev\weiqi\acceptance\r7-integrated-baa9747-run1` through `-run7`, plus `r7-integrated-baa9747-monitor-enum` and `-monitor-both`. Run1 contains pointer/canvas/reset/real-engine/concurrency/exit evidence; runs2–3 normal/max/min restart; run4 recovery and post-Discard final-flush denial/Retry; run5 legacy invalid geometry; run6 partial-caption restart and native theme screenshots; run7 both hidden rails retained after normal restart. Monitor-both proves same-PID recovery after removing both process-local native hooks. Enumeration-only controller termination is not live-Retry or normal-close proof.
- Native small-work-area automation: `r7-integrated-baa9747-small-auto/display-mode` records exact original DEVMODE, successful `CDS_TEST`, an independent restore watchdog, flags0 real mode switch, actual monitor work area, and verified restoration with registry unchanged. `run1`/`run2` retain exact source/PID/native HWND and normal exit0. The explicit restore path was exercised; watchdog timeout fallback was armed but not triggered.
- Ticket04's Linux/X11/Wayland and actual negative monitor/removal, and ticket06's original 100% → 150% → 100% retain their own SHAs/PIDs. Final run7 independently completes the changed integrated chrome's actual DPI coverage on PID13212 / HWND264354; `A14-continuation-native-final-verdict.json` links its measured native and supplemental records. macOS AppKit LTR/RTL is SKIPPED (user-approved 2026-09-30, owners04/07); no native Mac PASS or build is claimed. The unverified RTL candidate and original review attribution remain recorded. This execution-scope change does not alter macOS implementation/support or other phases' platform gates.
- Full case results, final fixed review target, resolved findings and both complete historical follow-up envelopes: [ticket07 completion record](../.scratch/adaptive-workspace-r7/issues/07-integrated-native-acceptance.md). Both acceptance and review gates pass; seven R7 owners are Accepted and R7 has exited. Ticket08 is unblocked and remains unexecuted.

