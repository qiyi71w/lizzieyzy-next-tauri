# Development Guide

This guide is for contributors working on the LizzieYzy Next Tauri 2 + Rust + TypeScript workspace. The existing Java/Swing application remains the maintenance release line; this document focuses on validating the Next scaffold and its current desktop workflows.

## Scope

Use this guide when changing:

- `README.md` and Next architecture/migration docs,
- scaffold validation and smoke documentation,
- the Tauri desktop app under `apps/desktop`,
- Rust crates under `crates/*`,
- SGF, KataGo, Foreground Engine Run, engine profile, Autoload Default, durable preferences, and SGF analysis persistence.

Do not treat a passing Next smoke run as full legacy parity. Provider/readboard work in this batch may provide offline contracts and runtime path plumbing, but live Fox/Yike network behavior, live readboard sidecar operation, and Tauri production release packaging still require environment-specific validation.

## Affected-Surface Development Gate

Local development validation is strictly scoped to the **affected surface** of each change. Running the full workspace test suite, workspace-wide clippy, or release-asset validation is not required—and should not be run unconditionally—on every local edit. Full workspace validation belongs to shared cross-cutting contracts, CI, and release qualification.

### Ownership and Acceptance Mini-Contract

- **Parent Executor Ownership**: The explicit parent executor owns validation execution, integration runs, and evidence collection. Subagents focus on scoped implementation and contract compliance, skipping project-wide builds, formatting, and linting mid-flight.
- **Acceptance Mini-Contract**: Before executing any acceptance operation, record an acceptance mini-contract capturing:
  1. *Affected behavior*: the exact functional delta or invariant being changed.
  2. *Assertion*: the concrete verification condition or test check.
  3. *Surface*: the touched layer (frontend unit/hook, Rust crate boundary, IPC command, or native desktop shell).
  4. *Inherited evidence*: provenance and technical equivalence of previously verified artifacts that remain valid.
  5. *Native gap*: what cannot be proven without native OS/hardware runtime and remains pending.
- **Evidence Reuse & Ticket Dimensions**: Preserve exact frozen ticket dimensions. Evidence reuse must record provenance and equivalence so that already-verified, unchanged contracts are not re-run redundantly, and final affected smoke does not duplicate existing proofs.

### Function-first Evidence And Planning Checks

The current R11–R17 functional route precedes R18 release acceptance (old, unexecuted R11 Release); see Matrix’s `ITEM::functional` records for exact inherited candidate/platform/service scope. A functional pass is not whole-item Accepted. Failed/unavailable required checks block the affected dependency; native/service acceptance cannot be replaced by repository fixtures or a review verdict. This documentation-only stage revision requires preservation/relationship checks and reader-navigation smoke, not repeated product/native runs. All planning implementation/integration tasks precede one integrated dual-axis review and centralized repair, then required actual acceptance and read-only Closeout; review does not publish the new R11 task group.

For a new runtime consumer, record resource source/version/path/integrity/capability compatibility, Start/Switch failure behavior, and applicable actual engine/service/credential conditions. Use qualified local binaries without requiring a current-repository Release. Native application validation, installed-product validation and upstream engine compilation are separate operations; perform only the one the affected contract requires.

For documentation-only Issue18 planning changes, validate current IDs/status totals, old-ID/status/Accepted-scope retention, complete 139-heading and unfinished-owner routing, intent resolution, functional dependency cycles, all 573 source objects/148 records and source release containment. Cold-read the whole-spec behavior and task contracts separately. For publication, exercise Markdown anchors and JSON routes using tracked repository files alone and compare source-intent, owner, decision and acceptance coverage against the approved local records. Reuse attributable scaffold validation while its product/configuration inputs remain unchanged; product/native/release builds do not prove future functionality. The [R11 plan](R11_PLAN.md), [later-stage plans](MIGRATION_PLAN.md#current-function-first-delivery), [source map](MIGRATION_SOURCE_MAP.md), [source contracts](MIGRATION_CONTRACTS.md) and [owner routes](MIGRATION_ROUTES.json) form the public reading surface. R11’s formal spec and nine tasks are prepared locally. Product implementation is not started and requires separate authorization plus the final Issue18 handoff. Operational Closeout cannot substitute for designated product integration acceptance.

Inherited R10 source-runtime records remain at their original candidates and conditions. In particular Tencent 60+60 is a working-tree production-crate probe, while native pagination is 25/25/10; initial readboard PL failures and later repaired candidates remain separate observations. Installation/network trust remains Not run where recorded.

Yike analysis-rule compatibility was verified on Windows source candidate `5c2fc5b5f5ed0c99154a3e9d1ed1f8cceb2fc5e3` in an isolated native Tauri session with the qualified KataGo CPU engine and b20 model. Both the preserved public-room 196315 SGF and a fresh Direct-network sync of that room produced real analysis at path `[0, 0]` from `RU[cn]`. Saving sync preferences retained the active Run/Job identity; Stop restored editing and remained idle beyond the prior polling interval. Native Save As/reopen retained raw `RU[cn]`, player metadata, personal comments and analysis. `RU[cn-unknown]` still produced `Could not parse rules: cn-unknown` with no attached analysis. The room was already ended (365 moves): this proves current public fetch/sync/settings/analysis composition, not newly arriving moves or queue latency. It does not extend installed-network, macOS, readboard or release acceptance.

Focused regression: `cargo test -p katago-protocol -p engine-manager --lib --test analysis_rules` passed 46 tests; the pre-existing ignored exact-match engine smoke is a separate path. `cargo clippy -p katago-protocol -p engine-manager --all-targets -- -D warnings` passed. The rule conversion is confined to analysis JSONL; exact-position match admission and local scoring inference retain their existing contracts.

### Per-Surface Validation Gates

- **Frontend / Tauri UI changes (`apps/desktop`)**:
  - Run focused unit and integration tests using `npm test` (or `npx vitest run <target-files>`) covering touched components, hooks, or domain modules.
  - Run `npm run build` only when relevant to the change (e.g. bundling configuration, TypeScript compilation/type-checking boundaries, packaging updates, or exported UI contract changes). Localized unit or styling fixes verified by Vitest do not require an unconditional UI build.
- **Rust crate changes (`crates/*`, `src-tauri`)**:
  - Run focused checks targeting the owning package and contract:
    ```bash
    cargo test -p <owning-crate> <test-filter> [--offline]
    cargo clippy -p <owning-crate> --all-targets -- -D warnings
    ```
  - Full workspace checks (`cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all --check`) are reserved for changes with shared cross-crate impact, CI pipelines, or pre-release candidate qualification.
- **Structural and Release Validators**:
  - Structural scaffold validation (`python3 scripts/validate_scaffold.py --verbose`) and release asset validation (`python3 scripts/validate_release_assets.py --verbose`) are run locally only when their corresponding contracts have changed (e.g. workspace structure, release asset manifests, or release packaging scripts). CI continues to run full validation; local documentation or component edits do not trigger release asset checks unless release contracts are modified.

### Human Interaction Rules

- **Human Involvement Gate**: Involve humans only for unavoidable authorization (e.g. OS permissions, credential prompts), physical hardware actions (e.g. physical monitor disconnects, hardware display changes), or subjective evaluation.
- **No Pixel Measurement Tasks**: Never assign pixel measurement tasks (such as measuring element bounding boxes, pixel heights, or drag margins) to humans. Use programmatic assertions or native OS inspection APIs (such as Win32 `WM_NCHITTEST` / `TITLEBARINFOEX` or X11/GTK geometry queries) instead.
- **Batching & Sequence**:
  - Batch independent actions and queries into a single response.
  - Group state-dependent actions into a single scenario.
  - Always preserve explicit sequential user requests.

### Runtime and Isolation Environment Rules

- **Browser Preview Is Not Native**: Running `npm run dev` in a browser is strictly for rapid layout prototyping and browser fallback checks. It does not provide real KataGo process management, native file dialogs, SQLite storage, app preferences persistence, or native window geometry. Native desktop behavior requires the native Tauri runtime (`npm run tauri:dev` or built desktop binaries).
- **Headless & Isolated Execution**:
  - **Linux**: Prefer `Xvfb` and private displays (isolated X11 display or headless Wayland compositor like Weston).
  - **Windows**: Prefer isolated desktop sessions (e.g. using `bin/windows-desktop-session.ps1` or separate desktop objects).
  - **No Claims of Windows Support Proven Yet**: Do not claim Windows native support is generally proven beyond the specific isolated candidate runs recorded with explicit evidence.
  - **No Desktop Hijacking**: Never switch the user's active foreground desktop or fall back to the interactive desktop without explicit user authorization.

### Provider and Readboard Acceptance Entry Points

When provider or readboard contracts are modified, validate through the owning surface:

- Provider contract tests: run Rust tests owning the provider contract modules using a focused package or filter (`cargo test -p <crate> <filter>`), ensuring non-zero tests are executed.
- Provider runtime path checks: verify `provider_yike_list`, `provider_yike_preview`, `provider_fox_list`, `provider_fox_list_more`, `provider_fox_preview`, `provider_tencent_list` and `provider_tencent_preview` return structured success/error results through the Rust/TypeScript boundary. Record offline contract status vs live external network verification separately.
- NET-01 transport: `cargo test -p provider-core --test network` exercises real loopback proxy/direct sends, redirect route resolution, NO_PROXY, terminal HTTP errors, cancellation, operation-wide retry limits, Yike10s/Fox25s read deadlines and bounded shutdown. `cargo test -p provider-fox --lib` covers Fox lookup validation, list/cursor normalization, CGI/H5 alternatives and terminal malformed responses. `cargo test -p lizzieyzy-next-desktop network_ --lib`, `fox_` and `provider_import_rejects` cover atomic PREF-01 writes/restart, Fox recents and SGF-07 policy/document fencing.
- Provider UI: `npm run desktop:test -- ProviderPanel TencentKifuCenter NetworkSettingsPanel` covers preview-before-import, paging/continuation, stale lookup suppression, failure/retry, recents restore/clear, and save failure/success. Native smoke must save a policy, fetch a real HTTPS game, observe the effective route, confirm the current game is unchanged until explicit Import, and restore the saved policy after restart. Browser tests alone do not qualify native routing, PAC or trust.
- Readboard domain tests: `cargo test -p readboard-sidecar -- --test-threads=1` runs actual controlled children/sockets (handshake, split/coalesced multi-frame streams, rejected frames, disconnect, restart generations, focus `loss`) plus frame-decoder and offline parser cases. `cargo test -p go-core` covers the remote-context fold and `cargo test -p sgf readboard` the Java-oracle sync engine. Fault fixtures are not evidence of the real Windows runtime.
- Readboard owner checks: `cargo test -p lizzieyzy-next-desktop --lib external_sync` covers readboard SGF-07 Start, stale generations, Error-paused/Retry/Stop, pending-start runtime loss, preferences/focus and analysis fencing alongside Yike; `continuous_analysis` covers durable `readboardSync` preferences.
- Readboard lifecycle checks: use `readboard_runtime_*` and `readboard_save_runtime_path` through the native panel; readiness requires `ready` plus the requested exact wire version. Keep `readboard_sidecar_sync_snapshot` as the offline preview boundary, not a readiness or current-game import substitute.
- UI path checks: ProviderPanel hosts ReadboardPanel with native Browse, Save path, Start, Stop/cancel startup and Restart; the sync sheet hosts ReadboardSyncPanel. Browser preview reports unavailable. Run `npm run desktop:test -- ReadboardPanel ReadboardSyncPanel YikeSyncPanel App.preferences` for ordering and preference regressions.
- Release dry-run: run `python3 scripts/validate_release_assets.py --verbose` and `.github/workflows/release-dry-run.yml` only during release-readiness qualification.

## Running The Next App

Desktop runtime:

```bash
cd apps/desktop
npm run tauri:dev
```

Browser preview:

```bash
cd apps/desktop
npm run dev
```

The browser preview is useful for layout and fallback checks. Real KataGo execution, native file dialogs, app-data engine profile persistence, asset inspection, and SGF analysis persistence require `npm run tauri:dev`.

### Repeatable Windows Native Smoke

From the repository root, `npm run desktop:test -- <affected-test-files>` forwards to the existing frontend test suite. Without a filter it runs the frontend suite, as CI does before the TypeScript build.

For the native document path, first let the executor prepare an exact committed candidate in a **new dedicated smoke destination** with `/home/dev/dev/weiqi/bin/prepare-windows-candidate`; uncommitted product changes are excluded. Preparation also deploys the matching Windows helpers. The runner reserves the candidate's isolated app-data identity for smoke reuse and refuses pre-existing data owned by another acceptance session; it never automatically discards a recovery prompt. Then run from this checkout (Windows or WSL, Node 22+):

```bash
npm run desktop:smoke:windows -- --candidate <candidate.json> --run-directory <fresh-directory>
```

The finite runner uses the real EXE/WebView2 on a private Win32 desktop. It verifies process identity, native Open cancellation, a 5×4 SGF with five moves including Pass, navigation to the last move, native Save As, reopening identical SGF content and clean exit. It creates its own fixtures and screenshots, records each case in `smoke.json`, and stops only its own recorded candidate. An occupied candidate is refused; use a separate prepared candidate rather than stopping another session. It neither sends global keys nor touches the clipboard. No foreground fallback is attempted.

Read `smoke.json`, inspect the screenshots and report uncovered boundaries separately. The fixed document smoke is not evidence for engine lifecycle, OS drag/drop, clipboard permissions, physical DPI/monitor changes, GPU/audio output or release installers. Those remain affected-surface scenarios, not an automatic request for the user to replay the complete release checklist.

## Repository Structure

- `apps/desktop`: React + TypeScript frontend.
- `apps/desktop/src/api`: frontend wrappers around Tauri commands and browser fallbacks.
- `apps/desktop/src/components`: board, analysis, chart, Engine Switcher, and Engine Settings UI.
- `apps/desktop/src-tauri`: Tauri 2 command gateway and native app integration.
- `crates/app-model`: shared DTOs.
- `crates/go-core`: board/rules logic.
- `crates/sgf`: SGF parsing, replay, and serialization.
- `crates/katago-protocol`: KataGo analysis JSONL query/response modeling.
- `crates/analysis-core`: derived analysis markers.
- `crates/engine-manager`: engine profiles, Autoload Default, Foreground Engine Run lifecycle, Analysis Jobs, process execution, and cancellation.
- `crates/app-preferences`: durable app preference load/save, unreadable isolation, and replace-safe persist.
- `crates/storage`: SQLite application storage helpers for games, nodes, engine profiles, and assets.
- `tests/golden`: SGF fixtures for migration and regression checks.

## Local Smoke Flow

This catalog provides desktop smoke scenarios for validating specific subsystems. Under the affected-surface development gate, **do not run this entire flow unconditionally for every change**. Instead, execute only the specific scenario(s) covering the affected surface of the change, recording the acceptance mini-contract and native gap. The complete smoke matrix is reserved for release candidates and milestone verification in [Release Checklist](RELEASE_CHECKLIST.md).

When an affected scenario requires the native desktop runtime:

Use the `分析` menu for the analysis start/cancel steps below. With a Ready run, `分析当前节点` starts finite selected-node analysis and `分析第一子主线` starts whole-game analysis. Space and the visible continuous action share contextual Start/Stop/Resume. Continuous intent defaults on and is durable; it follows accepted navigation and edits on an independently Ready engine. `分析 → 连续分析预算…` opens the categorized Preferences controls: independent enabled time/visits limits, retained numeric values and empty-board stop. Apply a 1-second or small visits limit to check normal limited status, retained results, same-node inhibition and explicit Resume. Time counts actual engine search, not queue wait; either enabled limit suffices. Failed/cancelled departure Save requires explicit Resume. Preferences writes must succeed before changing intent or replacing continuous work; checkbox/budget changes do not release safety/error holds. Finite and whole-game requests keep their own budgets.

```bash
cd apps/desktop
npm run tauri:dev
```

Analysis-task repository smokes (controlled Python engine, not real-KataGo/native acceptance):

```bash
unset DISPLAY && cargo test -p lizzieyzy-next-desktop --lib analysis_task_pause_continue_controllable_engine_smoke -- --nocapture
unset DISPLAY && cargo test -p lizzieyzy-next-desktop --lib all_positions_two_stage_pause_continue_controllable_engine_smoke -- --nocapture
cargo test -p engine-manager --test foreground_engine_run analysis_task_
cargo test -p engine-manager --test foreground_engine_run all_positions_two_stage_
```

The single-stage task controls show Pausing until target-final cleanup and Paused afterward. Continue retains the task's completed set on the same Run; it does not restore a task from SGF/recovery. The all-position strategy completes every 32-visit overview before starting the independently configured deep stage (minimum retained total-visits value 500). Pause/Continue keeps stage-local completions, restarts only the interrupted target with a fresh Job, and preserves task-owned overview summaries while deep results replace SGF primary analysis. One-thread queueing and two-thread lane independence use the controllable-engine checks. Exact Windows standalone/real-KataGo integrated evidence remains ticket 07's gate.

### 1. Open Or Load SGF

- Start from the bundled sample SGF or paste a fixture from `tests/golden/basic_19x19.sgf`.
- Click the parse/import action and confirm the board, move count, player names, and move slider update.
- Use native Open to load an `.sgf` file when running under Tauri.

Expected result: the status message reports a loaded/opened game and the board can move through replayed positions.

### 2. Configure Engine Profile

- Open Engine Settings (`引擎设置`). Settings only edit the catalog; they do not Start or Switch a run.
- Set a profile name.
- Select `KataGoAnalysis` or `GenericGtp`, then pick or type the executable program path.
- Enter additional argv as separate items. Empty items, spaces and non-ASCII text must remain single arguments, not shell text.
- For KataGo, pick or type the model and analysis config paths and set a positive finite max visits value. Do not add adapter-owned mode/model/config arguments to argv.
- GenericGtp exposes no KataGo settings. Supply the engine's GTP argv explicitly (for GNU Go, `--mode`, `gtp`); Start verifies its protocol before publishing Ready.
- Optionally set a working directory.
- Save the profile.
- Add a second profile if Autoload or Switch coverage is part of the change.

Expected result: successfully saved additions, renames and deletions immediately update the Engine Switcher; failed saves leave its prior catalog intact. Profiles also reload after app restart. Saving Autoload Default or Settings selection does not change the current Foreground Engine Run.

For migration coverage, seed an isolated app-data directory with an old unversioned collection or single-profile file. Loading must preserve values without rewriting the bytes. Explicit Save writes version 1. An unknown version or malformed settings must report an error without replacing the stored file. A denied write must retain the prior catalog and Run.

While KataGo is Ready, save changed argv or finite visits. The live snapshot and process remain unchanged and the profile is pending; Restart must create a new Run from the saved values. A GenericGtp candidate cannot replace that run until its complete handshake succeeds; a failed candidate leaves the original run and work intact.

During A-to-B switching, deleting either A or candidate B must fail at the backend, preserving the catalog and switching state. After B promotes, inactive A can be deleted and active B remains protected.

R11 profile ordering uses the existing catalog order as its initial order. The Rust `reorder_engine_profiles_settings` command persists a complete stable-ID permutation and reloads the latest records under the same write lock as Save. Stale order/membership, unknown/duplicate IDs and incomplete sets fail before writing; write/replace failures retain the previous bytes and returned/runtime order. Same-order requests do not write, even when storage is unavailable. Focused catalog, injected replacement, gateway late-consumer and browser-storage regressions cover these boundaries; native Settings/restart and real Run/job invariants remain separate acceptance gates.

Java mapping: `MoreEngines` first/up/down/last controls correspond to user-directed permutation of `leelaz.engine-settings-list[]`. Next stores that permutation as the `profiles[]` stable-ID sequence in catalog version 1. Java `ui.default-engine` is an index into its saved list; resolve that index to the original profile identity when interpreting the mapping. Next's `autoload_profile_id` keeps that stable identity through a reorder; an absent mark remains off. Neither Settings selection nor Autoload is a Run ID, and a new list index never changes the immutable active launch snapshot or job profile bindings. No alphabetical or other default sorting policy is introduced.

For order smoke, seed three saved profiles and keep one selected/marked while editing its unsaved name, program and argv. Move saved rows through all four controls; the dropdown and main-workspace catalog order update after each successful write while the editor and mark stay unchanged. First/up on the first row and down/last on the last row remain disabled. Reload/restart recovers the durable order. Inject a write/replace failure: old bytes/order and pending editor remain, and the full storage diagnostic is visible. A stale consumer must reject without overwriting newer storage. The localized labels use the shared effective-locale and missing/blank-resource fallback.

R11-07 local evidence covers 11 catalog tests, four persistence-failure tests, two gateway order/late-Save tests, one actual controlled-child Ready Run/held Job invariant case, and 81 affected frontend cases across editor, backend storage, resources and existing lifecycle UI. Production Rust reorder/persist/reload/write-failure and actual Chromium browser1437 four-controls/draft/failure/reload/fallback smoke passed. A late full Save retains the latest durable order of surviving IDs, appending newly saved IDs rather than restoring its stale list order. Chromium is browser fallback, and the controlled child is not qualified KataGo: native Settings/restart and compatible real KataGo Run/job identity acceptance remain root-scheduled gates.

R8 ticket 01 repository evidence: app-model 20 tests; engine-manager 22 unit, 9 catalog and 129 lifecycle tests; desktop gateway 148 tests; 13 affected frontend files with 284 passing cases, followed by the final 38-case lifecycle rerun (three added typed-refusal cases). TypeScript/Vite build passed. The actual-child argv case uses an isolated executable, Chinese/spaced paths and an empty argument; fixture lifecycle coverage includes finite/continuous/task/target-final, dual lanes and departure holds. These checks are not native acceptance.

Windows native evidence: exact application source `966e282b986c4e42a3f7bcdf73b1a21158ebfe33`, candidate `D:\dev\weiqi\worktrees\lizzieyzy-next-tauri\r8-profile-966e282\candidate.json`, private-desktop runs `D:\dev\weiqi\acceptance\r8-profile-966e282-run1` and `-run2`, isolated identifier `org.lizzieyzy.next.acceptance.r493403fda45f467d9c8fc5287caa80a7`. KataGo v1.12.3 Eigen CPU AVX2 passed legacy-read/no-rewrite, explicit version-1 save, Generic profile argv save/reload and readable Start/Switch refusal, immutable pending profile/capabilities, Restart adopting `-override-config numSearchThreads=2` and visits 12, real selected-node completion (13 reported visits), native read-only-file replacement denial preserving bytes/catalog/Run, and fresh Autoload without restored Jobs. Both runs exited 0; owned-child inspection after Stop found no KataGo. The real lifecycle smoke separately passed Start/finite/cancel/Stop/Restart/Switch on `aa9a6fa72470f296511307d0312572647dfcd389` (`D:\dev\weiqi\acceptance\r8-profile-aa9a6fa-real\real-lifecycle.log`); the subsequent application delta only formats typed UI errors. This is N01/N04's ticket-01 slice, not release packaging or GenericGtp runtime acceptance. ENG-01 historical evidence remains attributed to its original run; ENG-09/ENG-10 are not accepted by this ticket.

### 2.1 Verified Capability Admission (ENG-09)

Engine Settings distinguishes the saved/draft profile and static adapter ceiling from `当前 run 已验证`; `当前 run 能力` in the workspace always describes primary A, including during A→B switching. Saving adapter/argv/settings leaves A immutable and pending. Missing verification or an absent analysis group cannot authorize analysis. GenericGtp Start/Switch/Restart enters the shared lifecycle; game-move admission additionally requires the qualified launch and exact-position checks below.

Finite, continuous and whole-game/task admission are independent. Current query commands additionally request ownership/policy; visits limits, leading-candidate conditions and swing score/winrate filters require the respective capabilities. Preview submits no engine query. Start/Continue recheck after asynchronous work and before saving task presets/submitting work. Rust independently refuses before I/O, takeover or task mutation. Unsupported runs preserve historical SGF analysis and durable continuous intent, including existing holds; cancellation of owned work remains available.

Repository regression commands:

```bash
cargo test -p app-model
cargo test -p engine-manager -p lizzieyzy-next-desktop --lib --tests
cd apps/desktop
npx vitest run src/TruthfulAnalysisActions.test.tsx src/EngineLifecycle.test.tsx src/SelectedNodeAnalysis.test.tsx src/AnalysisPresentation.test.tsx src/SubBoardContentMode.test.tsx src/VariationReplay.test.tsx src/components/EngineSetupPanel.test.tsx src/domain/foregroundEngine.test.ts src/App.test.tsx src/App.preferences.test.tsx
npm run build
```

On application source `b23f3eb16c16ad63bf4570d7ecc9bf22c274ea05`, model20, manager22 unit+9 catalog+135 lifecycle and gateway151 checks passed. The affected frontend run passed249/250; its old NoEngine preference-race fixture was corrected to use a verified Ready run, then all24 preference tests and TypeScript/Vite build passed. All250 affected cases therefore passed across these runs. The new request-feature regressions first failed with unsupported selected-node IPC and a saved task preset, then passed with ownership/policy and candidate admission. Six new manager admission cases retain independent protocol-log/state evidence. Repository-controlled capability states are not native GenericGtp runs.

Windows M01–M03 passed on exact committed debug candidate `D:\dev\weiqi\worktrees\lizzieyzy-next-tauri\r8-capability-b23f3eb\candidate.json`, source SHA above. Evidence: `D:\dev\weiqi\acceptance\r8-capability-b23f3eb-run1\capability-evidence.json`, screenshots and process snapshots in the same directory. Private desktop PID71284/HWND47846250 used isolated app ID `org.lizzieyzy.next.acceptance.r1b73404ef1cc4b4882626d20e605390a`; the native WebView URL was `http://tauri.localhost/`. KataGo v1.12.3 Eigen CPU AVX2/FMA, model `D:\katago\yzy\weights\kata1-b20c256x2-s5303129600-d1228401921.bin.gz`, and config/workdir `D:\dev\weiqi\acceptance\r8-profile-01-assets` supplied the real runtime.

- M01: both adapters saved/reloaded through the same page; Generic argv `['', 'two words', '中文参数']` retained item boundaries. Native invalid visits refused without changes. A real read-only catalog caused `os error 5` while disk bytes, published catalog, run, game and preferences stayed unchanged; attributes were restored afterward.
- M02: real Ready run `6d0d2070-be5a-46bc-9f73-72127e6e73bd` published all11 verified flags through IPC/UI. Unsaved/saved adapter changes and saved argv remained pending without changing that run. Explicit Restart created `77a99cc5-aa62-4b93-b54a-af2189b4dedd`, adopting `-override-config numSearchThreads=2` and visits12; owned PID71276's command line confirmed the argv. Catalog selection did not start GTP.
- M03: Generic selection from NoEngine and from Ready A, plus Restart after saving A as Generic, explained refusal before lifecycle IPC. A CDP function-call breakpoint recorded two successful snapshot positive controls and no Start/Switch/Restart calls per refused action; full catalog/game/preferences/run snapshots were unchanged. Real continuous analysis reached its visits limit, finite analysis completed with17 visits, and a current-node task completed1/1. Stop reaped KataGo while SGF `LZ` history and durable intent remained readable; the desktop exited normally with code0.

The exact candidate also passed `real_katago_ready_job_stop_restart_and_switch` (1 passed,1 filtered,52.55s), with `LIZZIEYZY_REAL_KATAGO=1` and explicit engine/model/config/workdir environment paths. Log: `D:\dev\weiqi\acceptance\r8-capability-checks\real-katago.log`; repository transcripts are in that directory. Earlier profile-cutover evidence above retains its original candidate attribution for unchanged legacy migration/autoload paths. No Generic GTP readiness, game-move, cross-protocol N02/N03/N05/N06, provider or release-package claim follows from these checks; `ENG-10` remains Missing.

### 2.2 Generic GTP lifecycle (R8 ticket 03)

Repository checks: `cargo test -p app-model -p engine-manager` (189 passed, 2 real-KataGo tests ignored); `cargo test -p lizzieyzy-next-desktop --lib` (149 passed); the four affected capability/lifecycle frontend files (87 passed); `npm run build`. The five `gtp_` real-pipe scenarios cover split CRLF/multiline frames, stderr isolation, wrong IDs, malformed/oversized/flooded output, missing commands/v1, command rejection, exit/EOF, never-ending/trickle timeout, explicit recovery, cross-protocol promotion/rollback/latest intent and Stop during handshake. These fixture results are not native acceptance.

Real-engine qualification uses [Ben Lambrechts' Cygwin Windows GNU Go 3.8 build](https://gnugo.baduk.org/gnugo2/gnugo-3.8.zip), with argv `--mode gtp --chinese-rules --positional-superko --forbid-suicide --level 1 --seed 1`. `D:\dev\weiqi\acceptance\r8-gtp-assets\qualification.json` records the full 42-command transcript: GTP v2, GNU Go/3.8, 138 commands, board sizes 1/2/5/9/13/19 accepted and 20 rejected, komi round-trip, forbidden suicide, immediate ko and repetition after two passes rejected, play/pass, `time_settings 0 1 1`, `time_left b 1 1`, `genmove b` → `C4`, and clean `quit`. Separate scoring probes distinguish Chinese area (`B+25.0`) from Japanese territory (`B+23.0`) on the same one-stone 5×5 board. This qualifies the configured engine, not a manager game/clock API or all rule/history combinations. Ticket 04 owns exact-position/budget operations; ticket 05 owns final Windows N02/N04 integration. `ENG-10` remains Missing.

Native ticket-03 smoke passed on exact source candidate `ea89a48960e0e529508516e08d9cd12f52015ce2`, Windows private desktop / Tauri WebView2 (`http://tauri.localhost/`), isolated application identifier `org.lizzieyzy.next.acceptance.r1e5c305f57be41ee85645734f2e6a3f2`. Evidence: `D:\dev\weiqi\acceptance\r8-gtp-ea89a48-run1\native-evidence.json`, `gtp-ready.png`, `failed-switch.png`, `process-*.json`; fresh-launch evidence is in sibling `r8-gtp-ea89a48-run2`.

- The UI saved the qualified GNU Go profile, started it, displayed verified GTP v2 / GNU Go 3.8 / 138 commands without analysis admission, and stopped it; owned PID64416 was reaped.
- GNU Go with `--version` as a bad GTP candidate produced a visible framing failure; KataGo run `1ff43ab8-ed48-4a0b-93f9-5d31a48918f2` and continuous job `7b6f4eb8-7052-4724-8cf7-fc4d8ae4e1bb` remained searching. A subsequent valid GTP switch promoted a new run and reaped KataGo PID80468.
- Explicit GTP Restart replaced PID26724 with PID31688 and a new run ID. Normal app exit (Discard of isolated analysis changes) reaped the owned engine; a fresh app launch retained all three profiles but restored no live run. Both isolated sessions exited normally.
- The same candidate passed `real_katago_ready_job_stop_restart_and_switch` (1 passed, 1 filtered, 53.09s), using the ENG-09 CPU/model/config assets above. Full log: `D:\dev\weiqi\acceptance\r8-gtp-assets\real-katago.log`.

These are ticket-03 native lifecycle observations, not ticket-04 exact-position/game-move/time-budget implementation or the final ticket-05 combination matrix. `ENG-10` was Missing at that checkpoint; final acceptance is recorded in §2.4 below. Provider, release and physical-display acceptance are unchanged.

### 2.3 Exact-position budgeted moves (R8 ticket 04)

`computeGameMove` and `cancelGameMove` wrap native `foreground_engine_game_move` and `foreground_engine_cancel_game_move`; no game UI or SGF commit is introduced. Supply the current Ready run, semantic generation, selected NodePath and `ComputeBudgetDto { deadline_ms, max_visits }`. KataGo requires positive visits and a deadline; GTP requires a deadline and rejects visits. Watch `game_move_job` for the cancellation identity. A successful result reports whether both GTP time commands were mapped. The current game and durable preferences remain unchanged.

Exact admission supports square boards 2–19, explicit `RU[Chinese]` or `RU[Chinese-KGS]`, half-integer komi, empty root, root PL, alternating move/pass history and black root handicap for KataGo. Chinese-KGS uses positional superko; both rule dictionaries include AREA scoring, no suicide/tax/button, white handicap bonus N and friendly passes, following [KataGo's rule definitions](https://github.com/lightvector/KataGo/blob/v1.12.3/docs/GTP_Extensions.md). It rejects malformed replay, white/mixed/intermediate setup, unrepresentable PL, unknown rules and rectangles rather than flattening or inventing passes.

GenericGtp game-move qualification is GNU Go 3.8 / GTP v2 with exactly `--mode gtp --chinese-rules --positional-superko --forbid-suicide --level 1 --seed 1`, board size 2/5/9/13/19, Chinese-KGS and finite half-integer komi [-1000,1000]. Root setup is refused because handicap compensation has not been qualified. Other verified GTP engines remain lifecycle-only. Every admitted request sends a fresh board/komi/history synchronization. Time mapping requires both commands; manager deadlines apply in either case. Cancellation reaps GTP and leaves NoEngine; failure leaves Error; restart is explicit.

Reproducible real-process acceptance entrypoint (run on the intended platform with explicit asset paths):

```text
LIZZIEYZY_REAL_GAME_MOVE=1
LIZZIEYZY_MOVE_EVIDENCE=<fresh isolated directory>
LIZZIEYZY_GNUGO_PROGRAM=<qualified GNU Go 3.8 executable>
LIZZIEYZY_KATAGO_PROGRAM=<KataGo executable>
LIZZIEYZY_KATAGO_MODEL=<model>
LIZZIEYZY_KATAGO_CONFIG=<analysis config>
cargo test -p engine-manager --lib real_exact_move_smoke -- --ignored --nocapture
```

This opt-in harness uses the production manager for both actual processes, records exact projection, full move wire transcript, typed results, version/launch snapshot and owned-process cleanup in `move-transcript.json`. It covers move/pass history and root PL, plus KataGo setup, hard timeout and targeted cancellation. Real-engine and native evidence must identify their exact candidate; fixture passes alone do not accept N05/N06 or ENG-10.

Ticket-04 acceptance passed on committed source `62841b149361603a5701170fd84179a25f306d73` (Windows x64, private desktop, isolated app-data, actual Tauri WebView2):

- Production-manager smoke: **1 passed**, GNU Go **3.8** and KataGo **1.12.3 Eigen CPU AVX2/FMA**, using the qualified GNU argv above and the ENG-09 model/config assets. `D:\dev\weiqi\acceptance\r8-move-62841b1-manager\move-transcript.json` contains 81 records: 22 stdin, 37 stdout, 7 exact positions, 5 typed results, both version/run/PID records, timeout/cancel target-final confirmations and both exits. GNU mapped time for both history and root-PL requests; KataGo also handled true root handicap and returned Ready after timeout/cancel.
- Native gateway smoke: `D:\dev\weiqi\acceptance\r8-move-62841b1-native\gateway-evidence.json` records both real adapters through `foreground_engine_game_move`, typed budget refusal, explicit cancellation, and path round-trip cancellation. SGF bytes/comments, selected path, semantic generation, dirty/source and durable preferences/profiles were unchanged by move/refusal. This exercises the native IPC boundary, not a new game UI.
- Cleanup: manager PIDs 55324/4972 reaped; native app PID 84504 exited with code 0. `process-stopped.json` and `process-final.json` in the native evidence directory show no engine child and then no surviving tracked process. The IPC harness changed backend selection without updating the React view; its final window-close request therefore used a stale path. After verified engine Stop, `confirm_native_exit` closed this isolated fixture. This is not additional native close-UI acceptance; ticket-03 lifecycle evidence retains its original attribution.
- Final repair checks: desktop library **152 passed**, manager move **11 passed**, foreground lifecycle **138 passed**. A completed-result A→B→A publication regression failed before repair and passed afterward. Standards and Spec review both passed; no follow-up candidates. Earlier candidate `193f63cb70bf094671b2480185352b84b51abb28` evidence remains historical; final native evidence above was rerun after the publication repair.

Ticket 04 supplies the move evidence inherited by ticket 05 below; it did not independently close the broader integration or provider/release/display-matrix gates.

### 2.4 R8 integrated acceptance and R9 handoff (ticket 05)

R8 runtime acceptance passed on **2026-10-02**, exact source **`42cd91a657d2710b826cc1b94f21f168de64a0fa`**, repository `qiyi71w/lizzieyzy-next-tauri`, branch `feat/r8-profile-catalog-01`, WSL worktree `/home/dev/dev/weiqi/worktrees/lizzieyzy-next-tauri/r8-profile-01-20261001`. The investigation baseline remains `a0c8ed370f620341c863a3bb8fd4936710d82e72`; ticket 04's final HEAD is this integration start. Tickets 01–04 are already in its ancestry. No integration merge or new main baseline was needed, and ticket 05 changes documentation only.

**Evidence inheritance:** catalog migration and atomic save retain ticket 01's `966e282b986c4e42a3f7bcdf73b1a21158ebfe33` and ticket 02's `b23f3eb16c16ad63bf4570d7ecc9bf22c274ea05` attribution (§2/§2.1). The catalog writer is unchanged; current catalog/UI regressions and final reload/Autoload cover the subsequently changed consumers. Ticket 03's lifecycle evidence remains at `ea89a48960e0e529508516e08d9cd12f52015ce2` (§2.2); current manager tests and the final native switches cover later dispatch changes. Ticket 04's real manager and gateway evidence remains at `62841b149361603a5701170fd84179a25f306d73` (§2.3): its only delta to the integration source is six DEVELOPMENT lines, so source, lockfile, build inputs and move behavior are equivalent. Its retained GNU/KataGo assets and launch contracts are reused. None of these earlier runs is relabelled as a new ticket-05 run.

**Final repository checks** (Linux; actual command results, not native substitutes):

```bash
unset DISPLAY && cargo test -p app-model -p sgf -p engine-manager -p lizzieyzy-next-desktop --lib --tests
# apps/desktop working directory:
unset DISPLAY && npm test -- src/EngineLifecycle.test.tsx src/SelectedNodeAnalysis.test.tsx src/TruthfulAnalysisActions.test.tsx src/domain/foregroundEngine.test.ts src/api/backend.engineProfiles.test.ts src/components/EngineSetupPanel.test.tsx
```

Rust: **457 passed, 0 failed, 3 ignored**, 18 executables; app-model20, manager22/catalog9/lifecycle138/move11, gateway152 plus cache-cutover2, SGF103. The three opt-in real-engine tests were not run by this Linux command; their Windows evidence is attributed separately. Frontend: **135 passed**, six files. Logs are `rust-tests.log` and `frontend-tests.log` in the run1 directory below. The exact Windows candidate preparation also completed the frontend and native debug build. No release packaging was run.

| Repository gate | Current result and evidence |
| --- | --- |
| P01 | PASS: unchanged catalog migration/atomic writer; current catalog9 and manager unit checks, inherited native legacy/no-rewrite and denied-save evidence in §2/§2.1. |
| P02 | PASS: actual argv/cwd pipe coverage in current manager tests; inherited spaced/Chinese/empty argv native save/reload. Final native GNU command lines match saved argv. |
| P03 | PASS: current lifecycle/UI immutable snapshots; run1 `confirmed-gtp-pending-seed2` keeps seed1 until explicit Restart, run2 Autoload edits keep the same live run. |
| P04 | PASS: current lifecycle138 includes bounded GTP framing, IDs, missing commands, command errors, EOF, timeout/trickle/flood and recovery; ticket-03 pipe evidence retained. |
| P05 | PASS: current lifecycle cross-protocol promotion/rollback/stale/Stop regressions; final bad-GTP native rollback preserves KataGo. |
| P06 | PASS: current SGF103, manager move11 and gateway152; inherited exact transcripts and immutable-game observations for both actual engines. |
| P07 | PASS: move11 covers paired time commands, visits refusal and operation-wide deadline; inherited GNU real time mapping and KataGo budget/timeout. |
| P08 | PASS: move11 and manager unit checks cover typed/legal results, unique completed `order=0`, invalid candidates and no duplicate play; inherited real move results. |
| P09 | PASS: current lifecycle/move/gateway occupancy, identity and single-use publication regressions; inherited actual cancellation and process cleanup. |
| P10 | PASS: current rendered-UI tests plus final Ready-GTP native actions with verified zero-submission tracing, preserved SGF analysis and intent. |
| P11 | PASS: current lifecycle138/gateway152 and affected frontend tests; actual switch-back finite/continuous analysis, strong departure hold and normal window exit below. |

**Final native setup:** Windows x64, real Tauri/WebView2 **154.0.4258.48**, debug application; isolated identifier `org.lizzieyzy.next.acceptance.re06afb11e4794adf9734e823ef679e0f`, private Win32 desktop per session. Candidate: `D:\dev\weiqi\worktrees\lizzieyzy-next-tauri\r8-integrated-42cd91a\candidate.json`. GNU Go **3.8** uses the qualified executable/argv from §2.3; KataGo **1.12.3 Eigen CPU AVX2/FMA** uses `D:\katago\yzy\katago_cpu_avx2\katago.exe`, the `kata1-b20c256x2-s5303129600-d1228401921.bin.gz` model and `D:\dev\weiqi\acceptance\r8-profile-01-assets\analysis.cfg`. No network service or credential was required.

Evidence roots are `D:\dev\weiqi\acceptance\r8-integrated-42cd91a-run1`, `-run2`, and `-run3`. Each retains `run.json` and `process-final.json`; run1 also contains `native-evidence.json`, `gtp-history-surface.png`, `cancel-departure-save.json`, and intermediate process snapshots. Run2 contains `autoload-evidence.json`; run3 contains `cold-start-evidence.json` and `process-no-engine.json`.

| Native gate | Result, actual observation and attribution |
| --- | --- |
| N01 | PASS: inherit legacy migration and denied-save from 01/02, not rerun. Final GUI saves adapter-specific profiles/argv and reloads three profiles across fresh app processes; single GNU Autoload is saved, used, then cleared durably. M01 remains ticket 02's native catalog proof. |
| N02 | PASS on final candidate: real KataGo Ready → GNU Go Ready → KataGo Ready; verified analysis belongs to the primary run. GNU `--version` fails framing and preserves the original KataGo run/SGF/continuous state. GTP process snapshot contains only the owned GNU child, not a hidden KataGo. |
| N03 | PASS on final candidate: disabled analysis menu/continuous/task controls explain unavailable capability; actual disabled-menu clicks, board Space and task Start produce **zero IPC submissions**. A passive CDP function-call breakpoint on the immutable Tauri invoke function was positive-controlled with `foreground_engine_snapshot`; the earlier ineffective assignment observer is excluded. Before/after SGF, preferences and snapshot are equal; historical candidates/chart/comment remain visible. A cancelled native Save As leaves `safety_hold`, preserved across GTP/Space and return to KataGo; only explicit eligible Space releases it. This is Ready-GTP evidence, not ticket 02's runtime-unavailable M03. |
| N04 | PASS on final candidate: saved GNU seed2 is pending while the live run remains seed1; explicit Restart creates a new run and PID with seed2. Seed2 is deliberately outside game-move qualification (`game_move=false`), not an N05 claim. Stop removes the engine. Normal window close with actual dirty-game Discard reaps KataGo. Run2 creates a fresh GNU run from Autoload, no recovered jobs; clearing Autoload does not stop that run. After normal GNU exit, run3 is `no_engine` with no jobs or engine process and continuous intent retained. M02 remains attributed to ticket 02. |
| N05 | PASS by equivalent-source inheritance from ticket 04: production manager → real GNU Go3.8 → exact history/root-PL sync → paired time mapping → legal genmove → Stop; complete 81-record combined transcript and PIDs55324/4972 cleanup in §2.3. Final native lifecycle integration supplements, but does not replace, that protocol proof. |
| N06 | PASS: ticket-04 exact KataGo JSONL/order0/budget/cancel and immutable-game proof inherited under the same equivalence. Final candidate additionally completes actual continuous and finite selected-node analysis after cross-protocol return, preserves the strong hold, and closes through normal UI without the ticket-04 stale-React IPC harness workaround. |

Run1 app PID72104 stopped GNU PID81912 explicitly, then normally closed with KataGo PID83980. Run2 app PID63028 normally closed with GNU PID54424. Run3 app PID82436 started no engine. All three application exit codes are **0**, and all three final process lists are empty. Only these owned sessions were stopped. Temporary acceptance scripts stayed outside the repository; no test adapter, product scaffold or R9 game UI was added.

M01–M03 retain their original ENG-09 scope and candidate; N02/N03/N04/N06 add the final combination proof. ENG-01 and historical R3/R4/R6/R7 evidence are unchanged. **macOS and Linux GUI: NOT RUN**; no cross-platform GUI, installer, signing, provider/readboard or physical-display acceptance is claimed. The ticket-05 completion record owns the separate final Standards/Spec verdict and committed documentation HEAD.

**R9 contract:** use the version-1 catalog and read-only legacy migration; saved profiles and Autoload are durable, runs/jobs/capabilities/budgets are not recovered. Read verified primary-run capabilities, not adapter ceilings or the selected draft. `computeGameMove` takes Ready run identity, current generation/NodePath and a positive `ComputeBudgetDto`; cancellation uses the active `game_move_job` identity. Results are typed move/pass/resign with identities and `engine_time_mapped`; admission, occupancy, unsupported budget/position, command/protocol, timeout, process-exit and stale/cancel paths remain typed failures. Exact-position and qualified GNU restrictions are in §2.3 and Architecture; unknown semantics are rejected before I/O or mutation. KataGo needs positive visits plus deadline and target-final cleanup; GTP rejects visits and cancellation terminates its run, requiring explicit recovery. Every GTP request fully resynchronizes; `genmove` already changes the internal board, so never replay its result as another `play`. The read API never commits SGF. R9 alone owns global Match Session/Reservation, game commit, turn/terminal ownership and user-facing game modes.

### 2.5 Human/New match slice

The native `人机新局` action chooses a saved engine profile and explicit exact rules, then resolves the existing dirty-document Save/Discard/Cancel flow. Start prepares a Ready candidate and persists stable `matchDefaults` before replacing the document. During Playing, board input and Pass route through the match turn token; Stop and resign release ownership before review. Two passes enter the existing scoring confirmation. Save remains available; native close Cancel preserves the match. The browser preview displays this feature as unavailable.

Focused repository checks:

```bash
cargo test -p sgf --lib
cargo test -p engine-manager --test game_move
cargo test -p lizzieyzy-next-desktop current_game_state::human_match --lib
npm run desktop:build
npm run desktop:test -- api/match.test.ts components/HumanMatchDialog.test.tsx
```

The owner tests use actual `CurrentGameState`, the real preferences/recovery producers, a temporary filesystem and controlled child processes. They cover failed defaults persistence, Starting Stop/exit, black/white turns, illegal/stale/duplicate publication, two pass/resign, postcommit failure, history/savepoint and review-only recovery. They are controlled-process evidence, not real-engine or native-window proof. Native acceptance must separately pin the committed Windows candidate and record real KataGo New/move/Stop/save-reopen/restart, qualified GNU Go move/Stop, and actual close Save/Discard/Cancel. Continue, PK and role-based live analysis are not implemented by this slice. Installed-runtime and other-platform GUI evidence remain separate.

First runtime candidate `4179d832cdfc583b3c25a8faf5435542f58657fd` completed isolated Windows native runs with real KataGo 1.12.3 Eigen CPU (9×9 Chinese, 30,000 ms, explicitly reduced to 8 visits after the 800-visit run timed out) and qualified GNU Go 3.8 (Chinese-KGS, no visits). Actual SGF files record the human/engine moves, standard resignation, and two terminal passes without a generated result after scoring Cancel. Native Save/Open dialogs and close Save/Discard/Cancel, including SaveAs Cancel preserving the same session, were exercised. Restart reused stable defaults without a session. Exact candidate/config/run/PID/file evidence: `D:\dev\weiqi\acceptance\r9-human-new-01-4179d83-evidence.md`; this is debug-runtime evidence, not release/installed evidence.

The repair checks additionally cover unchanged display-preference writes during a reservation, rejection of continuous-setting changes before disk/cache mutation, failed preference persistence preserving manager intent, invalid saved match defaults using the existing quarantine/recovery path, and recovery with continuous intent enabled and the engine Ready before or after restore. Match runtime failures retain the committed profile/run/side. UI capability disclosure uses the actual Ready `game_move` qualification; another profile or changed saved spec remains unverified until preparation. Final review and the repaired native candidate are recorded in ticket 01's completion record; the first candidate's evidence applies only to unchanged paths.

Final reviewed runtime `b7898e51cba9ef11c68a46d631132f201b29260b` passed Standards and Spec final review with zero open blockers and zero follow-up items. Its exact Windows debug candidate completed real KataGo HA2 / human White Pass / engine move / Stop, real qualified GNU Go opening / Stop, native Save/Open, and recovery of an actual native-produced document with KataGo Autoload and continuous intent enabled: review only, no Match Session or automatic analysis. Both qualified capability disclosures rendered in full. The repaired candidate `9a89c6434fe9b801e16bf3037ea6a75e00697ecb` separately exercised durable display preferences during Playing, retained timeout side/profile/cause, unsupported GNU handicap rollback and enabled-intent restart. Original close/scoring evidence above remains attributable to its unchanged paths.

Complete identity, engine/config, run/PID, SGF, screenshot and inheritance record: `D:\dev\weiqi\acceptance\r9-human-new-01-final-evidence.md`; final run directory `D:\dev\weiqi\acceptance\r9-human-new-01-b7898e5-run1`. Repository results are separate runs: owner 15, continuous preferences 5, preferences 44, engine game-move 20 and SGF 61 tests passed; frontend original 257, repair 123 and final focused 23 passed, with TypeScript/Vite build passing. The foreground aggregate had 138/139 pass and one snapshot-wait timeout; that exact test passed in isolation, not a clean aggregate rerun. macOS/Linux GUI and installed runtime are NOT RUN; packaging and upstream builds are outside ticket 01. This completes the Human/New slice, not whole `GAME-01`–`GAME-05` acceptance.

### 2.6 Human/Continue match slice (R9 ticket 02)

`棋局 → 人机续弈 → 从当前节点续弈` opens the match dialog in Continue mode for the selected node. Board size, rules, komi, handicap and setup are shown as inherited and cannot be edited; the user chooses human color, saved profile, deadline and (KataGo) visits, and must confirm that PB/PW/RE are whole-document root metadata. A dirty document continues without a departure prompt; the native path is kept. The start uses the node and document generation captured when the dialog opened; a navigation or structural edit that lands later makes the start fail rather than move it, while continuous analysis accepted on the same position does not. Success makes the start node's path the mainline ending at a new structure node; old continuations stay variations. One Undo restores the original tree order, players, result and selection and keeps analysis accepted later on surviving nodes.

Focused repository checks:

```bash
cargo test -p sgf --lib continuation_staging
cargo test -p match-core
cargo test -p lizzieyzy-next-desktop current_game_state::human_match --lib
npm run desktop:test -- components/HumanMatchDialog.test.tsx App.test.tsx
```

Owner tests cover non-mainline Continue as an independent mainline, ignored New fields, merged defaults, stale NodePath rejection, a start surviving analysis accepted while the dialog was open, the same next move creating a new node, implicit-handicap setup agreeing on White to play, recovery envelope, undo/redo and save/reopen; zero-move Stop/resign; inherited trailing pass plus structure node completing a double pass; and unconfirmed, moved-selection, defaults-write-failure and unsupported-rules rejection preserving document, foreground and defaults. SGF tests cover startup undo reversing only continuation content while later analysis stays attached; App tests cover the pinned dialog baseline under a late navigation. These are controlled-process evidence.

Windows native acceptance used exact candidate `4c24de5edb4851e8bb7e3ed7675128746471b5e3` (isolated app identifier and private desktop) with the ticket 01 KataGo 1.12.3 Eigen CPU and GNU Go 3.8 profiles. The fixture had a non-mainline start node `W[ge]C[start node]` with an old continuation, old PB/PW and `RE[B+R]`. With continuous KataGo analysis attached to the start node and still advancing while the dialog was open, Continue showed inherited 9×9/Chinese-KGS/7.5, required the root-metadata confirmation and started. Human `B[ee]` got a real KataGo reply `W[fg]` (8 visits, 30,000 ms), then Stop returned to review. Native Save As wrote `PB[Human]PW[R9 Repaired KataGo]`, no RE, and the start path promoted first with `(;;B[ee];W[fg])` before the old `B[eg]C[old continuation]` and old main line; reopening showed that mainline. Three Undo steps (the two moves, then the start unit) restored the fixture order and `PB[Old Black]PW[Old White]RE[B+R]`, while the analysis accepted afterwards stayed attached. A second Continue from the reopened endpoint with GNU Go got a real reply `W[cc]` after human `B[gc]` and Stop. Predecessor candidate `5ebcbc5` failed the start under live analysis because the dialog pinned `snapshot_seq`; `4c24de5` is the repair.

### 2.7 KataGo single-game PK (R9 ticket 04)

`棋局` exposes PK New and Continue with separate Black/White saved KataGo profiles and request budgets, plus a positive move limit (default 450). Same-profile selection starts two independent processes. Continue inherits the exact selected position and requires whole-document metadata confirmation. Rust alternates the moves; board/pass/resign input and ordinary analysis cannot bypass PK ownership. Pause seals the current turn before targeted cancel/drain; Resume becomes available only after drain and uses a fresh job with the complete side budget. Stop, move limit and failure return to review without a generated result or automatic analysis. At this ticket GTP PK was unavailable; §2.8 adds qualified GTP sides. Qualified Human GTP is unchanged.

Repository `qiyi71w/lizzieyzy-next-tauri`; historical investigation `4c458ee208e03576e8ada60fed74ea9804716282`; implementation starts at ticket 02 final `fcb1142bd789e8615398c8545875e5d4a6c2c78e`, which contains ticket 01 final `0dfbc418fec22009ec2cf3a2161829341ea91c88`. Worktree `/home/dev/dev/weiqi/worktrees/lizzieyzy-next-tauri/r9-katago-pk-04-20261004`, branch `feat/r9-katago-pk-04`. No new main baseline or ticket-03 integration was used.

Focused repository evidence (separate from native acceptance):

```bash
CARGO_INCREMENTAL=0 cargo test -p lizzieyzy-next-desktop current_game_state::human_match --lib
CARGO_INCREMENTAL=0 cargo test -p engine-manager --test game_move
CARGO_INCREMENTAL=0 cargo test -p engine-manager --test foreground_engine_run
CARGO_INCREMENTAL=0 cargo test -p app-model -p match-core
# From apps/desktop:
npm test -- src/App.test.tsx
npx tsc --noEmit
npm test -- src/App.preferences.test.tsx src/components/HumanMatchDialog.test.tsx
```

Observed: owner **29/29**, engine game-move **28/28**, foreground lifecycle **139/139**, app-model **25/25**, match-core **9/9**, App **103/103**, preferences/dialog **40/40**, and TypeScript passed. An additional test, `pk_candidate_exit_before_commit_never_installs_or_retires_the_old_foreground`, subsequently passed for both candidate sides (**1/1**, 28 filtered); it adds coverage without changing the runtime candidate. A damaged local Rust incremental cache required `CARGO_INCREMENTAL=0`; this was not a product failure. Initial frontend fixture/type-check failures were corrected before the recorded passing runs.

The owner and manager tests use real temporary preferences/recovery files and controlled child processes. They cover distinct same-profile PIDs, both-side dispatch, second preparation failure, either candidate's precommit exit, cancellation/default-write rollback preserving the old foreground/document, backend exclusion, exact Continue history/setup/PL, stale/duplicate/sealed results, A→B→A, Pause/Stop drain races, fresh-job/full-budget Resume, inherited-history exclusion, double-pass/limit priority, typed resign, each side's EOF/protocol/deadline failure, idle-side exit and unconfirmed cleanup occupancy. Paused recovery reopens committed review data without live session identity. These fixtures do not stand in for real KataGo or a native window.

Windows native acceptance passed on **2026-10-04**, exact runtime commit **`ae2d12e56d6d7d97ca727a49a20b0463cf82f04e`**. Candidate: `D:\dev\weiqi\worktrees\lizzieyzy-next-tauri\r9-katago-pk-04-runtime1\candidate.json`; run: `D:\dev\weiqi\acceptance\r9-katago-pk-04-runtime1-run1\run.json`. The private-desktop Tauri debug process was PID **27496**, isolated identifier `org.lizzieyzy.next.acceptance.r52e4b7073dab4ef888bd5f81492ec89e`. Both participants used saved profile `default`, real KataGo **1.12.3 Eigen CPU AVX2**, `D:\katago\yzy\katago_cpu_avx2\katago.exe`, model `D:\katago\yzy\weights\kata1-b20c256x2-s5303129600-d1228401921.bin.gz`, and `D:\dev\weiqi\acceptance\r8-profile-01-assets\analysis.cfg`. Native smoke explicitly selected Black **30000 ms / 8 visits**, White **40000 ms / 12 visits**, 9×9 Chinese-KGS / komi 7.5; product visit defaults remain 800.

| Native scenario | Observed result | Evidence under `D:\dev\weiqi\acceptance\r9-katago-pk-04-assets` |
| --- | --- | --- |
| New, same profile | Two concurrent KataGo children **29388 / 84360**, distinct runs `ed073b69-896e-49dd-bcc7-779172d5859a` / `99ab4282-7022-4a33-bf06-e149dda022d0`; 20 alternating committed moves before Pause, Black to play. | `new-paused.json`, `new-paused-processes.json`, `new-paused.webp` |
| Paused native close → Cancel | Actual window close opened the unsaved-document dialog. Cancel preserved session `pk-1`, selected branch, count 20, side, runs and completed Paused state. | `paused-close-cancel.json` (`unchanged: true`) |
| Resume → Stop | Black job `12849569-cb60-4d4d-9248-1af9cf5429cb` was replaced by `deb18200-485e-4da3-9e22-c7b671acabc3` on the same run/generation/path. Two more alternating moves, Stop at count 22; both children reclaimed, NoEngine and continuous safety hold despite enabled intent. Native Save As produced SGF without RE. | `new-resume-stop.json`, `new-stopped-processes.json`, `new-stopped.sgf`, native Save dialog artifacts |
| Exact Continue → limit | Opened the non-mainline `B[ce];W[ge]C[start node]` at `[1,0]`; new runs `64929b4a-6987-4d1b-93d8-912d34725cf7` / `b0c0049a-29b0-40d4-b068-f862cc227ad6` alternated `B[fe];W[ff];B[gd];W[gf]`. Limit 4 ended after four new moves, not six inherited-plus-new moves. Both children reclaimed; no RE. Native Save/reopen preserved the independent first mainline, old continuation, old mainline and C. | `continue-limit.json`, `continue-limit.webp`, `continue-limit-processes.json`, `continue-input.sgf`, `continue-reopened.json`, native Open dialog artifacts |
| Paused idle White failure | A further Continue used runs `7931b740-b8d2-4506-be93-57a87a21428d` / `7253c047-0f5c-4d66-9fb8-562d2a73d0d1`, children **89452 / 84020**. After two committed moves, paused with Black to play; targeted termination of White produced visible White/profile `process_exit`, ended the whole match and reclaimed the other child. Committed board retained; no automatic analysis. | `white-exit-injection.json`, `white-paused-exit.json`, `white-paused-exit.webp`, `white-exit-reclaimed.json` |

The candidate then closed normally through the real unsaved-document dialog with Discard: `run.json` records **EXITED / exit_code 0**, no forced application termination. Durable preferences contain stable PK profile IDs/budgets/limit only; recovery contains SGF, selected path and document bookkeeping, not Paused/run/job/session state. The full local evidence record is `D:\dev\weiqi\acceptance\r9-katago-pk-04-final-evidence.md`.

Ticket 01/02 evidence retains its original candidate and scope (§2.5/§2.6); it is not relabelled as a ticket-04 run. Ticket 03 was absent from the ticket-04 candidate, so that run claims no role-based live-analysis combination. GTP PK belongs to ticket 05; GAME-03 remains **Partial**, not Accepted. macOS/Linux GUI, installed artifacts, packaging, signing, providers and readboard were **not run** for this slice. Final review and completion commit identities belong to the ticket's Completion record; documentation/test-only additions after the native SHA do not change its runtime evidence.

Final repair-enabled FULL_REVIEW: **Standards CLEAN; Spec CLEAN; zero blockers and zero follow-up items**. Both independent reviewers covered base `fcb1142bd789e8615398c8545875e5d4a6c2c78e` through runtime HEAD `ae2d12e56d6d7d97ca727a49a20b0463cf82f04e`, including the additional precommit-exit regression and four updated contract/evidence documents in the working tree. The parent verified that HEAD, complete tracked diff and preserved untracked environment configuration stayed unchanged during review. No repair was required. Native acceptance and this review are separately satisfied gates; the final commit adds test/documentation evidence without changing the accepted runtime.

### 2.8 Qualified GTP PK and explicit Resume rebuild (R9 ticket 05)

The PK dialog accepts KataGo or the qualified GNU Go 3.8 profile on either side, including GTP/GTP. A GTP side uses only its hard deadline (mapped to `time_settings`/`time_left` when both are listed); visits are hidden in the UI and refused by the manager before any process starts. Each GTP move resynchronizes `boardsize`/`clear_board`/`komi` and full `play` history, so an engine-applied generated move is never replayed. Pause cancels a KataGo move by targeted drain; an in-flight GTP move is killed and reaped and its side appears in `rebuild_sides`, explained in the match status. A side without an active move is never reclaimed. Only `恢复 PK` rebuilds that side from the session's immutable profile snapshot (saved-profile edits during Pause are ignored) with a new run, readiness/capability/exact-position admission and a new job with the full budget; `resume_pending` makes repeated Resume join one attempt. Stop or confirmed exit seals the rebuild; rebuild failure ends the match at the committed position with side/profile/typed cause and no retry or default write.

Repository `qiyi71w/lizzieyzy-next-tauri`; historical investigation `4c458ee208e03576e8ada60fed74ea9804716282`; implementation starts at ticket 04 final `a2f34a2b0deb71547e31f0d8a5b8ba4b6aeb91e0` (contains 01/02 finals and 04 runtime `ae2d12e`). Worktree `/home/dev/dev/weiqi/worktrees/lizzieyzy-next-tauri/r9-gtp-pk-05-20261004`, branch `feat/r9-gtp-pk-05`. No main sync; ticket 03 was not included in that predecessor candidate.

Focused repository evidence (separate from native acceptance):

```bash
CARGO_INCREMENTAL=0 cargo test -p lizzieyzy-next-desktop --lib human_match
CARGO_INCREMENTAL=0 cargo test -p engine-manager -p app-model -p match-core
# From apps/desktop:
npx tsc --noEmit
npx vitest run
```

Observed on runtime `746fa97dc0c917cae945e3c8da976f23726aae3f`: owner **35/35** (three consecutive runs), engine-manager lib **22**, foreground lifecycle **139/139**, game-move **29/29**, catalog **9**, app-model **25**, match-core **9**, TypeScript passed, full vitest **449/449**. Review repair commit `343e8f60a7a955626af512471219603979de1d4f` added two test-only cases; owner **37/37** on three consecutive runs.

The controlled fixture `crates/engine-manager/tests/fixtures/pk_process.py` speaks GTP when launched with the GNU Go argv and records per-PID argv, traces and genmove handshakes. Owner tests in `current_game_state/human_match/tests/pk/gtp.rs` cover: mixed PK reaping only the in-flight GTP side; Resume with an edited saved profile still launching the original argv, new run/job, same position/count, full budget (`time_settings 0 3 1`) and stale old token; GTP/GTP rebuild following whichever side Pause interrupted; rebuild failure without retry or default write; repeated Resume joining one attempt with Stop or exit during rebuild, including a late Ready after the owner seal; GTP-side Start failures (startup and unsupported rules) rolling back only candidates; resident GTP EOF/protocol/timeout and idle GTP exit ending the whole match; an unconfirmed GTP reap keeping occupancy and owned PIDs until an explicit Stop retry; and an unacknowledged KataGo drain in mixed PK not reclaiming the idle GTP side. The manager test refuses a GTP visits budget before spawn and dispatches both resident GTP runs. These fixtures do not stand in for real engines or a native window.

Windows native acceptance passed on **2026-10-04**, exact runtime commit **`746fa97dc0c917cae945e3c8da976f23726aae3f`**. Candidate `D:\dev\weiqi\worktrees\lizzieyzy-next-tauri\r9-gtp-pk-05-runtime1\candidate.json`; run `D:\dev\weiqi\acceptance\r9-gtp-pk-05-runtime1-run1\run.json`; private-desktop Tauri debug PID **58296**, isolated identifier `org.lizzieyzy.next.acceptance.r942c907326ef40769f66be49ddb651e4`. KataGo **1.12.3 Eigen CPU AVX2** (assets as §2.7, 30000 ms / 8 visits) and the R8-qualified GNU Go **3.8** (`D:\dev\weiqi\acceptance\r8-gtp-assets\gnugo-3.8\gnugo.exe`, qualified argv, 30000 ms).

| Native scenario | Observed result | Evidence under `D:\dev\weiqi\acceptance\r9-gtp-pk-05-assets` |
| --- | --- | --- |
| Exact Continue, KataGo Black / GNU Go White | Native Open; non-mainline `B[dd];W[pq]C[start node]` at `[1,0]`; runs `ce1dca01-…` (PID **692**) / `adf66257-…` (PID **67028**, qualified argv); legal alternation from the exact position. | `continue-input.sgf`, `open-continue-dialog.*`, `started-processes.json` |
| Pause during White GTP genmove | Clicked `暂停 PK` while job `8ebe5860-…` was active at count 25: Paused, `rebuild_sides=[white]`, no failure, reservation held; PID 67028 reaped, KataGo PID 692 retained; status explains the rebuild. | `paused.json`, `paused-processes.json`, `paused.webp` |
| Resume rebuild | `resume_pending` then Playing with new White run `c8a490b1-…`, same generation 28/path/count 25/Black run, new job `d099838f-…`; GNU Go moved legally. Ran to limit 40: `move_limit`, both children reaped, NoEngine/safety hold. | `resume-trace.json`, `first-session-limit.json`, `resumed-processes.json` |
| Swapped sides, Pause/Resume/Stop | Continue from move 42 with GNU Go Black (PID **65624**) / KataGo White (PID **6848**). Pause during Black's GTP job → `rebuild_sides=[black]`, 65624 reaped, 6848 kept. Resume → Black run `07abe3f3-…`, PID **40200** with the original argv, White unchanged, count 26 kept. Match Stop at count 67 → Idle/`stopped`, no failure, NoEngine/safety hold, no owned children. | `second-*.json`, `second-*.webp` |
| Review handoff | Native Save As SGF has no `RE`, final-session PB/PW and the old root/start-node structure; preferences hold only stable PK defaults; normal exit code 0. | `mixed-pk-stopped.sgf`, `save-as-dialog.*` |

The full local record is `D:\dev\weiqi\acceptance\r9-gtp-pk-05-final-evidence.md`. Unchanged R8 GNU Go qualification (§2.2–§2.4) keeps its original candidates; session rebuild and these mixed dispatch combinations are new evidence. A native GTP kill whose reap cannot be confirmed is not reproducible; it is covered by the controlled zero-cleanup-budget test only. Ticket 01/02/04 evidence keeps its candidates. GAME-03 remains **Partial** pending ticket 06 integration. macOS/Linux GUI, installed artifacts, packaging, signing, providers and readboard were **not run**.

Repair-enabled review: FULL_REVIEW (base `a2f34a2`, HEAD `746fa97`) found Standards clean and two Spec controlled-coverage gaps (GTP unconfirmed reap; resident GTP runtime failures). Test-only repair `343e8f6` resolved both in VERIFICATION; **Standards CLEAN; Spec CLEAN; zero follow-up items**.

### 2.9 Human/New session analysis (R9 ticket 03)

The evidence below belongs to ticket 03's Human/New candidates. The combined source supports session analysis in Human New and Continue only; PK retains unsupported/off analysis and rejects the policy command. Ticket 01–05 evidence in §2.5–§2.9 retains its original candidate attribution. Current combined-candidate observations and the A01–A13 inheritance matrix are in §2.10.

During a native Human/New match, `本场候选分析` starts off and initially enables both roles. `分析回合` selects human, engine or both turns for this session only. Verified KataGo supplies finite human-turn output through the match reservation and engine-turn output from the actual move job. Policy changes clear output; if an engine move is already running it continues, with new-policy analysis deferred to subsequently scheduled work. Qualified GNU Go exposes disabled analysis controls and a backend capability refusal. The ordinary display settings still control ANA-04 presentation; legacy black/white analysis filters do not gate session output.

Runtime candidate `37d393c1e97c446a0150cc6c94c1b2b1bbd18e66` passed an isolated Windows Tauri debug run using real KataGo 1.12.3 Eigen CPU, 9×9 Chinese, human Black, 30,000 ms and explicitly chosen 8 visits. In one session, native screenshots and correlated snapshots show human-turn candidates and candidates from the actual engine move job. Role switching, off/on, human-analysis cancellation before Pass, disabling output while the same engine job continued, committed engine moves, and Stop during human analysis were exercised. Stop left no engine/session frame or fabricated result; ordinary preferences were unchanged and SGF contained only actual moves. A subsequent real qualified GNU Go 3.8 session showed off/unsupported controls and returned `unsupported_capability` for a direct non-off policy request. Native File → Exit / Discard exited with code 0.

Exact candidate: `D:\dev\weiqi\worktrees\lizzieyzy-next-tauri\r9-analysis-03-37d393c\candidate.json`. Run/evidence: `D:\dev\weiqi\acceptance\r9-analysis-03-37d393c-native` (`run.json`, `analysis-evidence.json`, `gtp-evidence.json`, human/engine/clearing/Stop/GTP screenshots). PID 80980, start `2026-10-04T02:57:34.4097494Z`, isolated desktop, CDP 9463. These are real native/runtime observations, not controlled-process or browser-preview proof.

Repository checks: current-game Human owner 19 passed; manager `game_move` 26 passed; manager `foreground_engine_run` 139 passed; App plus five affected analysis/presentation/replay suites 164 passed; TypeScript/Vite build passed. Controlled owner/manager tests cover all policies, reservation exclusion/drain, capability refusal, sealed identity/epoch events, finite completion, bounded delivery, Stop and unchanged SGF/preferences. Rendered UI tests cover role controls, full envelope/revision rejection and nonpersistence. Existing ticket-01 transaction, recovery and Save/reopen evidence remains attributable to its original runs; changed analysis/move/Stop paths were rerun here. Continue+analysis belongs to ticket 06. macOS/Linux GUI, installed runtime and release packaging are NOT RUN for this slice.

Review repair runtime `a4e603132f13e665a7cf93a815fb7b4a818de59d` separately verified completion across departure gates and document-projection ordering. Deterministic rendered tests first reproduced stale visible komi after a newer analysis-only/same-document revision; controlled owner tests first reproduced an unretired successful job and swallowed deadline failure under exit. After repair, owner 21 and the six affected frontend suites 167 passed, including obsolete-document rejection; TypeScript/Vite build passed. Manager/protocol code is unchanged from the initial candidate, so its 26/139 results and qualified-GTP native evidence remain attributable to that run.

The repaired exact Windows candidate used the native-produced profile catalog from the initial run through the normal settings command. Real KataGo New (9×9 Chinese, komi 6.5, human Black, 30,000 ms / 8 visits) displayed the correct komi, finished human analysis while File → Exit awaited a decision, and retained the final frame with no active job after Cancel. An actual human Pass and engine move advanced to turn 3; the next analysis completed while the native Save As dialog was open. The run-bound native helper canceled Save As, preserving the same session and final frame; Stop then released ownership. A new session defaulted off. With an explicit 3,000 ms / 800 visits budget, its analysis timed out during the exit decision: typed timeout, error phase, cleared frame and released resources remained after Cancel, without a fabricated move or result. Final File → Exit / Discard exited 0.

Repaired candidate: `D:\dev\weiqi\worktrees\lizzieyzy-next-tauri\r9-analysis-03-a4e6031\candidate.json`; evidence: `D:\dev\weiqi\acceptance\r9-analysis-03-a4e6031-native` (`run.json`, `repair-evidence.json`, `save-as-cancel.json`, completion/Save-cancel/timeout screenshots). PID 32228, start `2026-10-04T03:27:13.1973476Z`, isolated desktop, CDP 9464. Initial candidate evidence remains limited to unchanged role scheduling, engine-job output, policy clearing and GTP capability behavior; repaired departure and projection claims use this run and the new deterministic regressions.

### 2.10 Integrated R9 runtime acceptance (ticket 06)

Integration/runtime source **`93b8410fde3bd0795662470b4d663c8b506cc4e6`** merges ticket 03 final `0fb1dbfebc44d94e6270c163f773769de2afd2b3` into ticket 05 final `6bfc13986f678422a40f46ab857df45bf515f7d3`; both finals are ancestors. Repository: `qiyi71w/lizzieyzy-next-tauri`; worktree `/home/dev/dev/weiqi/worktrees/lizzieyzy-next-tauri/r9-integration-06-20261004`, branch `feat/r9-integration-06`. Historical investigation SHA remains `4c458ee208e03576e8ada60fed74ea9804716282`.

The merged scheduler retains PK's per-side run/budget dispatch and Human reservation-authorized analysis. Continue frames use the newly promoted mainline's generation/path; frontend document projection ordering remains independent of match-only revisions. PK has no Human analysis control or frame and rejects policy requests through its backend owner. The shared controlled fixture now accepts both move requests with top-level `maxVisits` and ordinary continuous requests with `overrideSettings`.

Affected repository checks: current-game match owner **44**, engine-manager `game_move` **35**, `foreground_engine_run` **139**, app-model **24**, match-core **9**; rendered App **122**, dialog **16**, preferences **24**, AnalysisPresentation **17**, SelectedNodeAnalysis **19**; TypeScript/Vite build **PASS**. New owner/rendered regressions cover Continue+role analysis and Human→PK clearing/exclusion, including rebuild. Process suites passed serially after an initial concurrent ETXTBSY/readiness timeout; those failed attempts are retained. A deterministic rollback failure exposed the fixture query-shape assumption above; the original preservation test passes after repair.

Exact Windows debug candidate: `D:\dev\weiqi\worktrees\lizzieyzy-next-tauri\r9-integration-06-runtime1\candidate.json`; isolated application ID `org.lizzieyzy.next.acceptance.r4f54dbf65b9f46f1964e8dc66acad26a`. Private native run1 PID **64240**, CDP 9471, and restart run2 PID **66432**, CDP 9472, are recorded in `D:\dev\weiqi\acceptance\r9-integration-06-runtime1-run1` / `run2`. Real KataGo **1.12.3 Eigen CPU AVX2**, the b20 model/config from §2.8, and qualified **GNU Go 3.8** with the same exact argv were used. Smoke explicitly selected **30000 ms / 8 visits** for KataGo and deadline-only GTP; product defaults remain unchanged.

| Native case | Current-candidate result |
| --- | --- |
| Human New | Fresh 19×19/7.5/HA0/Black/default-budget form; explicit 9×9 smoke, actual E5/C5, both-role analysis, Stop/no RE. |
| Off-mainline Human Continue | Original two-move path `[1,0]`, inherited 9×9/6.5/Chinese-KGS, explicit root confirmation; new endpoint `[0,0,0]`. Real human-turn frame generation 6 and engine-turn frame generation 7/path `[0,0,0,0]` with rendered F3/F7/B5 candidates; reply, turn clearing and Stop. |
| Save/reopen | Actual native Save As/Open of `continued-human.sgf`; default first-child four-move session mainline, old main/continuation branches and root/start/old-node personal `C` preserved; no RE. |
| Human→dual KataGo PK | Independent runs and PIDs **2116/10640**, unsupported/off analysis and no candidates; Pause at 32, same path/generation/count Resume with a fresh job, Stop at 49/no RE. Foreground switch attempt did not replace either PK run. |
| Mixed exact Continue | GNU Go Black/KataGo White from an exact 20-move position; active-GTP Pause at 12 and 66 new moves. Resume preserves position/count and original argv; pinned GNU Go PID **66820** is reaped, then replaced by **55688** with a new run/job; KataGo **27356** remains. Stop at 109 new moves releases ownership/no RE. |
| Terminal/failure | Real mixed two passes → local scoring Confirm `W+26.5`; reopening scoring and Cancel leaves the tree/result unchanged. Another real mixed continuation resigns `W+R`. Explicit 1 ms Human request timeout produces typed failure, no RE and editable review. |
| Exit/restart | Paused PK File→Exit→Cancel retains session/reservation. Deliberate test-only abnormal process-tree disposal; actual restart/Restore reproduces the exact tree/path at move49 with null session/run/job, no Paused state, analysis off and zero engine children. Native dirty Save + Save As then normal exit code **0**. |

Full A01–A13 attribution, exact job/run identities, screenshots, process command lines, SGFs and snapshots: **`D:\dev\weiqi\acceptance\r9-integration-06-final-evidence.md`**, artifacts under **`D:\dev\weiqi\acceptance\r9-integration-06-assets`**. Shared dispatch/analysis/projection changes invalidated reliance on old combination smoke, so both PK compositions, Human New/Continue, role analysis and restart were rerun. Unchanged SGF/history/scoring algorithms, default/recovery schemas, atomic writer, GTP qualification/encoding/rebuild retain their predecessor candidates. Human Pass/Resign, direct double-pass Cancel, close Save/Discard/Cancel and one-unit continuation undo keep §2.5–§2.9 attribution; current cases supplement rather than relabel them.

Windows source-runtime and real-engine assertions above **PASS**. macOS/Linux GUI **NOT RUN**, not PASS or a platform exemption. Installed artifacts, packaging/signing, physical-display matrix, providers/readboard and upstream engine builds are outside this ticket and NOT RUN. Stale-event and unconfirmable-reap interleavings remain controlled-process evidence, not native claims.

Repair-enabled **FULL_REVIEW: Standards CLEAN; Spec SUCCESS; zero blockers and zero follow-up items**. Both independent axes covered all 19 changed files and A01–A13 at fixed point `6bfc13986f678422a40f46ab857df45bf515f7d3`, runtime HEAD `93b8410fde3bd0795662470b4d663c8b506cc4e6`, plus the four contract/evidence document updates. Parent reconciliation verified unchanged HEAD, complete diff, status and preserved untracked environment link/content. Batch 1 required no repair. GAME-01–05 are **Accepted**; the final documentation commit does not relabel runtime evidence or start ticket 07/later-phase work.

### 2.11 R9 PR transplant onto main

The `feat/r9-game-modes` PR preparation worktree starts at main **`acc8405b2b03931cb26700c6006c3260f311e53e`** and applies only the R9 delta from **`4c458ee208e03576e8ada60fed74ea9804716282`** to **`566d748cf63c4a36bd54c4b7c3d8aec65643cb8f`**. This avoids replaying the original R7/R8 history already squash-merged into main. All 49 R9-only files and three main-only repair files retain their respective source content; six automatically merged overlap files retain main's exact repair edits. The remaining overlaps are the move worker, its controlled process fixture, and move regression tests.

The move worker combines main's terminal KataGo `error`/`errors` handling with R9's reserved analysis stream: reject before rich-frame parsing, keep the Run Ready without sending terminate or waiting for target-final, seal the failed Match Reservation, and let explicit Stop release the owned resources. Ordinary move rejection remains usable for the next non-match request. The added `rejected_reserved_analysis_keeps_run_ready_but_seals_match_without_frames` regression covers both human-turn analysis and engine-turn move-with-analysis, both terminal fields, exact failure identity, absence of frames/terminate, retained process, blocked follow-on match work and confirmed Stop cleanup. Main's whitespace-only working-directory handling, nonblank-path preservation and explicit legacy-migration refusal are retained.

Working-tree checks on 2026-10-04 (serial Rust process suites):

- `cargo test -p engine-manager --test game_move -- --test-threads=1`: **37 passed**.
- `cargo test -p lizzieyzy-next-desktop -p app-model -p app-preferences -p sgf -p match-core -p engine-manager --lib -- --test-threads=1`: **361 passed, 1 ignored**; desktop 196, app-model 24, preferences 44, SGF 65, match-core 9, manager 23. The opt-in real-engine harness remains ignored.
- `cargo test -p engine-manager --test foreground_engine_run -- --test-threads=1`: **139 passed**. Rust total: **537 passed, 1 ignored**.
- Frontend TypeScript/Vite build and eight focused files (App, preferences, HumanMatchDialog, EngineSetupPanel, AnalysisPresentation, SelectedNodeAnalysis, EngineLifecycle and foregroundEngine): **259 passed**.
- Scaffold validator: **10 passed**; its fixed crate inventory is not a substitute for the workspace manifest or the checks above.

A temporary standalone executable exercised the production manager through real JSONL child pipes. Normal output produced D4 with visits 1/8 frames and a consume-once move permit. A terminal `errors` response produced Protocol, no frames or terminate, the same Ready Run and a sealed reservation. Explicit Stop reaped the owned child and released the reservation in both cases (observed PIDs 3716055 and 3716358). The executable and isolated temporary data were removed after the smoke.

These are repository and controlled-process results on the transplanted working tree, not a new committed native candidate. The unchanged frontend, SGF/history, default/recovery and GTP behavior retain their original §2.5–§2.10 evidence attribution. Real KataGo/GNU Go and Windows native GUI were not rerun for this PR preparation; the new terminal-rejection/analysis combination has controlled-process evidence only. Installed artifacts and other-platform GUI remain outside these results.

Independent bounded-transplant FULL_REVIEW: **Standards CLEAN, Spec SUCCESS; zero findings and zero follow-up items**. Both axes covered the 58-file staged diff, source-preservation evidence and the manually combined response boundary. Parent reconciliation confirmed unchanged HEAD, full diff, status and untracked configuration throughout review; no repair batch was needed.

### 2.12 PR #16 review repairs

Local repairs to PR head `ef703e7495be27f07c488ae45735c84fc137c410` cover review comments 1–6: intentional startup Stop/exit returns Idle/Stopped after confirmed cleanup; a previous idle session cannot consume a new startup cancellation; missing profiles release the empty reservation before process I/O; aborting an uncommitted reservation rearms ordinary cancellation supervision without resuming analysis. Genuine preparation/cleanup failures remain visible. The completeness index now matches all 112 parity rows (69 Accepted, 8 Partial, 10 Missing, 25 Deferred). CI-reported Rust formatting is corrected, and match-owner errors use the existing desktop boxed-error convention without changing wire DTOs.

The startup-cancellation, stale-session cancellation, missing-profile reservation and suspended-watchdog regressions failed before their corresponding fixes. Focused frontend checks passed: eight files, **260 tests**, plus TypeScript/Vite production build. Final `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings` passed. Rust match-owner regressions passed **46/46**, including readiness-held child cleanup for Stop/confirmed exit, exit admission refusing a prepared candidate, genuine failure retention and PK cancellation. Manager `game_move` passed **38/38**, including waiting for authoritative No-engine before restarting after watchdog cleanup.

The follow-on Pause/departure repair closes a race in which cancellation delivery failure removed the Job before departure collected its snapshot. The owner now propagates the same sealed task's Run failure before Save/replacement. `task_pause_cleanup_failure_aborts_departure_and_retains_game` fixes this ordering deterministically: wait for delivery failure and Job removal before departure, retain the original 9×9 game without opening/writing Save, then verify explicit Stop permits a fresh 13×13 replacement. The controlled-child regression failed before the source fix and passed afterward; its deadline-failure branch also passed. Final `CARGO_INCREMENTAL=0 cargo test --workspace -- --test-threads=1`: **730 passed, 0 failed, 3 ignored**, including all 198 desktop tests. Format and full-target Clippy checks passed. The three opt-in real-engine tests remain unexecuted; incremental compilation was disabled because runtime interruption had corrupted local build artifacts.

A browser smoke exercised the modified App with controlled Tauri IPC: previous idle `human-1`, delayed new startup, Stop before the new Starting event, then cancellation of **only `human-2`**. The final state was Idle/Stopped with no held resources or startup-failure banner; the match dialog reopened normally. This is browser presentation evidence, not native IPC acceptance.

A temporary executable exercised the production manager with real controlled JSONL child pipes. Missing-profile refusal released the reservation while the same old Ready Run still returned D4 (PID 10277). An unclean analysis drain preserved the foreground child across abort, then the restored ordinary watchdog reported Timeout (PID 10286). Explicit Stop/teardown confirmed both children reaped and reservations released. The executable, isolated data and browser/Vite session were removed after verification.

These results apply to the local review-repair working tree, not a pushed or newly committed native candidate. Real KataGo/GNU Go, Windows native GUI and installed artifacts were **not rerun**; §2.5–§2.10 retain their original candidate attribution.

### 3. Start From The Engine Switcher

- In the main-workspace Engine Switcher, choose the saved profile (`选择引擎` starts it).
- Confirm the chip leaves `未加载引擎` and `停止` / `重启` become enabled.
- Optional diagnostic: in Engine Settings click `检查资源`. Asset existence is not a Start gate; Start still validates assets and adapter readiness.
- Optional Autoload: mark `启动时自动加载`, restart the desktop app, and confirm only that profile is started. A missing model stays No-engine with a typed Autoload/asset banner and no fallback.
- Click `停止` and confirm the chip returns to `未加载引擎` with `停止` / `重启` disabled.
- Start again, then click `重启` and confirm a new Ready run.

Expected result: analysis actions stay disabled until a Ready run exists. Check Assets does not start a process.

### 4. Run One-Position Analysis

- With a Ready run, select a node on the board (or keep the current selected node).
- Click `分析当前节点`.
- Confirm the job stays on that Ready run, the current-game generation, and the selected `NodePath`.
- Confirm candidates, PV, winrate/score, ownership overlay (`领地`), and policy overlay (`策略`) update for that node only.
- Click `取消此手分析` while the job is still running. Confirm the Engine Switcher stays Ready and whole-game (if running) continues.
- Start a second `分析当前节点` and confirm only the latest matching identity publishes.
- Confirm a timeout or protocol failure does not leave candidates or overlays.

Expected result: selected-node analysis runs on the current Ready Foreground Engine Run, not a one-shot profile process. Publication is identity-bound to run/job/generation/`NodePath`.

Repository evidence (does not substitute for native GUI or live KataGo):

```bash
cargo test -p sgf selected_node --offline
cargo test -p lizzieyzy-next-desktop selected_node --offline
cargo test -p engine-manager --test foreground_engine_run selected_node_ --offline -- --test-threads=1
cd apps/desktop && npx vitest run src/EngineLifecycle.test.tsx src/SelectedNodeAnalysis.test.tsx
cd ../.. && cargo test -p engine-manager --test foreground_engine_run continuous_
```

Optional real KataGo on a resident Run (ignored by default):

```bash
LIZZIEYZY_REAL_KATAGO=1 cargo test -p engine-manager --test real_katago_lifecycle_smoke -- --ignored --test-threads=1
```

### 5. Run Full-Game Analysis

- With a Ready run, click `分析第一子主线`.
- Watch the whole-game lane show `整局 completed/expected 剩余 remaining` as first-child mainline nodes finish. Each completed node should publish its own `NodePath` result immediately; do not wait for the batch to end, and do not treat turn as job identity.
- Navigate with `上一手` / `下一手` while the lane is running and confirm review navigation stays enabled and does not cancel the job.
- Optionally click `分析当前节点` on the same Ready run and confirm both lanes stay active together.
- A second `分析第一子主线` while that lane is occupied must be rejected without cancelling the selected-node lane.
- Cancel or fail after at least one node has completed and confirm the current-session result for that node remains visible.

Expected result: whole-game analysis occupies only the Ready Run's whole-game lane, walks `[]` / `[0]` / `[0,0]`…, and keeps completed node results after cancel/fail. Selected-node can run concurrently. Durable SGF attachment is not part of this smoke.

### 5.1 Bind Analysis Presentation To Exact Nodes

- After selected-node completes, navigate away with `下一变化` and back with `父节点`. Confirm that node's candidates, PV Sub-Board, winrate, and score return, and that candidate hover does not return.
- Start `分析当前节点` while a whole-game result already exists for a child node, then open that child. Confirm the child's presentation appears, `取消整局分析` stays available, and navigation does not cancel whole-game.
- Click `停止` so the chip returns to `未加载引擎`. Confirm candidates, PV, ownership (`领地`), and policy (`策略`) clear rather than remaining from the previous Ready run.
- Confirm the candidate table follows the durable candidate limit without a second hardcoded cap, and that missing ownership/policy keep `领地` / `策略` disabled instead of showing default overlays.

Expected result: review presentation is a projection of identity-valid current-session results keyed by `NodePath`. Leaving Ready clears it. Unsupported fields stay absent. Native KataGo/GTK is not required for this repository evidence.

Repository evidence (does not substitute for native GUI or live KataGo):

```bash
cd apps/desktop && npx vitest run src/AnalysisPresentation.test.tsx src/EngineLifecycle.test.tsx src/SelectedNodeAnalysis.test.tsx
```

### 5.2 Round-Trip Java-Compatible SGF Analysis Payloads

- Use native Open to load `tests/golden/java-analysis-branching.sgf` (or another Java SGF with `LZ` / `LZOP`).
- Confirm the selected node shows that node's primary analysis (candidates, winrate, PV, and score when present) without starting KataGo.
- Navigate with `下一变化` / `父节点` and sibling controls. Each node must show only its own primary analysis; a node whose `LZ` is malformed or incomplete must stay empty.
- Save or Save As, reopen the file, and confirm personal `C`, unknown properties, secondary `LZ2` / `LZOP2`, and malformed payloads are still present. Next must not add an Analysis Context property or append generated statistics to `C`.
- Confirm new encoded primary uses root `LZOP` and non-root `LZ`. Live engine attachment is §5.3.

Expected result: opening a branching Java SGF shows selected-node analysis through the exact-node presentation path. Save/reopen preserves secondary, unknown properties, personal comments, and malformed payloads. Native KataGo/GTK is not required for this repository evidence.

Repository evidence (does not substitute for native GUI or live KataGo):

```bash
cargo test -p sgf --test java_analysis_payloads --offline
cd apps/desktop && npx vitest run src/AnalysisPresentation.test.tsx
```

### 5.3 Attach Live Analysis And Save Call-Time Snapshots

- Start a Ready Foreground Engine Run and click `分析第一子主线` on a game with at least two first-child mainline nodes.
- After at least one node completes, confirm the document is dirty (`未保存` / Save enabled) and that node's candidates appear. Click `另存为(S)` and write snapshot A. Do not stop the job.
- Wait for a later mainline node to complete. Confirm the document is dirty again. Click `另存为(S)` and write snapshot B.
- Reopen snapshot A: it must contain only the analysis attached at the first Save As. Reopen snapshot B: it must include the later node's primary `LZ` / `LZOP` as well.
- Confirm personal `C`, secondary `LZ2` / `LZOP2`, and unknown properties are still present. Confirm Save As remains available while analysis is still running.
- Repeat with `分析当前节点` on one node, then Save. Re-analysis of that node must replace only the primary payload.

Expected result: identity-valid completed selected-node and whole-game results attach to the exact `NodePath`, dirty the game without changing generation, and persist only through ordinary Save / Save As as the call-time snapshot. Later completions re-dirty. A failed Save keeps dirty and in-memory payloads.

Repository evidence (does not substitute for native GUI or live KataGo):

```bash
cargo test -p sgf --test replace_primary_analysis --offline
cargo test -p lizzieyzy-next-desktop current_game_ --offline
cargo test -p app-model admits_analysis_attachment --offline
cd apps/desktop && npx vitest run src/domain/analysisJob.test.ts src/AnalysisPresentation.test.tsx src/App.test.tsx src/SelectedNodeAnalysis.test.tsx src/EngineLifecycle.test.tsx
```

Native KataGo/GTK is required for the GUI Save As A/B reopen steps above. When that environment is unavailable, record the smoke as not run; repository tests cover attachment admission, generation stability, partial retention, replacement preservation, save-while-running, later-redirty, and failed-save recovery.

### 5.4 Persisted Sub-Board Variation And Raw

- Open `显示` → `小棋盘设置`. Confirm first-use is `变化图` and the Sub-Board aria-label is `参考图变化副棋盘`.
- With a live selected-node candidate list, confirm Variation shows the numbered active PV. Switch to `纯棋子` and confirm the Sub-Board keeps current stones but drops PV, branch, and move numbers, including after hovering a main-board candidate.
- Confirm `参数` / Preferences `小棋盘内容` edits the same durable mode, including while the analysis rail stays visible. Collapse/hide of the rail (if used) must not rewrite the mode and is not Raw.
- Quit and start `npm run tauri:dev` again. Confirm the last successfully saved mode is restored.
- A failed write must keep the previous visible mode and report the save failure.

Repository equivalent:

```bash
cd apps/desktop && npx vitest run src/SubBoardContentMode.test.tsx src/components/AnalysisPanel.test.tsx src/App.preferences.test.tsx src/api/preferences.test.ts
cargo test -p app-preferences
```

Expected result: Variation/Raw is one durable PREF-01 value, not an SGF field and not rail collapse. Missing storage loads Variation. Native KataGo/GTK is not required for this repository evidence. Native restart smoke of Raw and Variation remains a separate desktop column.

### 5.5 Synchronized Variation Replay

- Confirm first-use `显示 → 变化回放` is off and Preferences `回放间隔` is 500 ms. The Variation Sub-Board shows the complete active PV.
- Enable Variation Replay. Confirm the main-board candidate presentation and visible Variation Sub-Board immediately show the same first PV move, then advance together once per interval, and idle armed at the full PV without looping.
- Switch to `纯棋子` or collapse the analysis rail while the main board still shows candidates: replay continues on the remaining eligible surface. Hide candidates as well so no eligible surface remains: progress pauses. Restore a surface and confirm replay continues from the retained prefix.
- Disable replay and confirm both eligible surfaces restore full-PV presentation. A live interval change between 100 and 5000 ms applies to the next gap without resetting progress. A failed write keeps the previous enablement/interval.
- Replay must not change `NodePath`, dirty the SGF, create a move, or start an Analysis Job. `Ctrl+A` remains Review Autoplay.
- With a live KataGo PV, confirm a statistics-only update (visits/winrate) does not reset the prefix, and a candidate/PV sequence change immediately shows the new first move.

Repository equivalent:

```bash
cd apps/desktop && npx vitest run src/VariationReplay.test.tsx src/SubBoardContentMode.test.tsx src/App.preferences.test.tsx src/api/preferences.test.ts
cargo test -p app-preferences
```

Expected result: one application-owned timer and prefix are shared by the main board and visible Variation Sub-Board. Missing storage loads replay off / 500 ms. Native KataGo/GTK is not required for this repository evidence. Live PV statistics vs sequence updates and native restart remain a separate desktop column.

### 6. Cancel Analysis

- On one resident Ready run, start both `分析当前节点` and `分析第一子主线`.
- Click `取消此手分析` and confirm the whole-game lane continues.
- Click `取消整局分析` and confirm selected-node (if restarted) is unaffected.
- Confirm the Engine Switcher is still Ready (`停止` / `重启` enabled).
- Confirm ordinary `保存` / `另存为` stay enabled while analysis is running.
- Start another analysis afterwards to ensure the job registry recovered.

Expected result: each cancel is lane-local, does not Stop the run, does not lock Save/Save As, and does not prevent a later start on that lane.

### 7. Verify SGF Analysis Persistence

- Analyze a game, Save or Save As, and reopen the SGF. Confirm analysis returns from the file.
- Reparse or reopen the same SGF without saving: live attachments that were not saved must not reappear.
- Confirm Preferences has no cache category or automatic load/save cache controls, and that the chrome has no cache-status badge or cache action.

Expected result: attached analysis is only visible after Save/reopen of the SGF (or from the in-memory current game before Save). There is no second durable analysis authority.

### 8. Save SGF

- Use Save or Save As under the Tauri desktop runtime.
- Reopen the saved file.
- Confirm parse/replay still succeeds and move count remains stable.

Expected result: SGF write validates parseability and can round-trip through native open.

### 9. Durable Preferences

- Open Preferences from `参数`, `棋盘`, Settings → `首选项…`, or Settings → `综合设置(Shift+X)`.
- Confirm the sheet is categorized (`分析呈现`, `复盘`, `棋盘`, `胜率图`) and that View-menu `候选` / `领地` / 胜率图设置 edit the same values as the sheet.
- Change a visible preference (for example uncheck `候选`) and wait until the sheet reports that it is saved.
- Quit the Tauri app and start `npm run tauri:dev` again.
- Confirm the saved value is still applied after restart.

Repository equivalent: `cargo test -p app-preferences successful_write_is_reloadable_as_restart`.

Expected result: missing preference storage loads owner defaults; a successful write survives native restart; View-menu and Preferences-sheet edits share one durable store.

### 10. Winrate Chart Encoding And Move Rank

- Open a branching SGF whose selected line has `LZ` / `LZOP` on some nodes, including at least one node with `scoreMean` and one without (or a missing payload).
- Confirm the chart follows `下一变化` / `下一分支` rather than a hardcoded mainline, missing analysis stays a gap, and the current-move marker tracks the selected node.
- Toggle `显示` → `胜率图设置` and `参数` → `胜率图`: Black vs selected-node-side-to-play, winrate/score/both, Blunder Bar, Graph Hover, and Score Lead Scale. Confirm they share the PREF-01 keys and that a restart after a successful write keeps them.
- On a score-less line, `目差线` and `目差刻度` are unavailable. A persisted score-only choice still presents as winrate without rewriting the stored `scoreLeadLine`. Invalid scale input is ignored; a larger current-line peak may grow the axis for the session only.
- Enable Blunder Bar and confirm only Inaccuracy / Mistake / Blunder bars appear between adjacent displayed analyses. Hover shows move number and visible values and does not navigate; hover off is inert.

Native KataGo/GTK: required for the GUI branching selected-line steps above. When that environment is unavailable, record the smoke as not run.

Repository evidence (does not substitute for native GUI or live KataGo):

```bash
cargo test -p analysis-core -p app-preferences --offline
cd apps/desktop && npx vitest run src/domain/moveRank.test.ts src/domain/winrateChart.test.ts src/WinrateChartEncoding.test.tsx src/App.preferences.test.tsx
```

Expected result: first-use defaults are Black perspective, both series on, Blunder Bar off, Graph Hover on, and Score Lead Scale 15. The chart encodes the selected root-to-leaf variation with Java Auto Move Rank bars.

R11 chart readability checks use actual black-frame leaders in either whole-series perspective: ±7.3/±4.6, ±0/±0.04 → `0.0`, +0.05 → `B+0.1` and −0.05 → `W+0.1`. Check winrate-only `50%`, score-only `0`, both-series unmarked highlighted baseline and no-metric absence; ordinary grid lines must not duplicate the baseline. Current, hover and endpoint text must remain readable together. The 1600×600 export renderer has fixed perspective/series labels, scales, true gaps, visible bars and current marker, with hover and UI chrome omitted.

For chart export, capture before the chooser, then navigate and publish later analysis while it remains open: the PNG must retain the invocation model and `<sgf-stem>-winrate-m<selectedMove>.png` (literal `untitled` without an SGF source). No valid selected-line data must visibly refuse without a chooser or engine. Windows source-runtime acceptance must separately exercise PNG prefill, overwrite, visible write failure, reopen dimensions/readability and exact unanalyzed branch/path click with fake analysis and engine inactive. Browser renderer snapshots and controlled IPC fixtures do not satisfy those native gates. `EXPORT-03` retains its Shipped Platform installed-production-trust evidence boundary; an unsigned CI artifact is not automatically a Shipped Platform.

### 11. Next-move Review Marker

- Open a branching SGF with coordinate children, a pass child, and Java `LZ` / `LZOP` on the selected node and its Primary Child.
- Confirm first-use is Variations: every coordinate-bearing child is marked and the Primary Child is emphasized; pass / non-coordinate children stay in the tree without a board mark.
- Cycle `显示` → `下一手标记(J)` and `参数` → `下一手标记` through Off, Variations, and Graded. Confirm they share the PREF-01 key and that a restart after a successful write keeps the last saved mode.
- Press `J` when the board is not an editable control. Confirm it cycles the same three modes. Type `J` into 个人评论 and confirm the mode does not change.
- In Graded, confirm the Primary Child uses the shared Auto Move Rank when both payloads are parseable and positive-visit. Missing, malformed, zero-visit, pass-only, or absent Primary Child analysis keeps variation marks and does not start an Analysis Job. No uncertainty label appears.
- Native restart after saving Graded restores Graded without creating a new analysis request.

Native KataGo/GTK: required for live grading of a freshly analyzed branching game. When that environment is unavailable, record the smoke as not run; Java LZ reopen and preference restart can still be checked from a fixture SGF.

Repository evidence (does not substitute for native GUI or live KataGo):

```bash
cargo test -p app-preferences --offline
cd apps/desktop && npx vitest run src/domain/nextMoveReviewMarker.test.ts src/domain/shortcuts.test.ts src/components/BoardCanvas.test.tsx src/NextMoveReviewMarker.test.tsx src/App.preferences.test.tsx
```

Expected result: first-use is Variations. Off draws no marks. Graded grades only the Primary Child from attached tree analysis and never starts analysis.

## Provider And Sidecar Smoke Flow

Run this only in an environment with the required provider accounts, network access, target client state, and readboard sidecar installed. Record skipped items explicitly; skipped live checks do not invalidate offline contract work, but they do block live-support claims.

Repository-level checks may show that the commands and UI route exist and return typed success, typed error, or runtime unavailable results. Live checks require real services or processes. Keep those two result columns separate in the handoff.

### R10 ticket08 integrated Windows evidence (2026-10-06)

Integration starts at `f6cc553be3a336f9aaefe21b618bec7c80fb67e5`: ticket07 final `70c8031` plus ticket04 final `907af86`, with all01–07 final commits verified as ancestors. Investigation baseline remains `566d748cf63c4a36bd54c4b7c3d8aec65643cb8f`. Worktree: `r10-integrated-08-20261006`, branch `feat/r10-integrated-08`.

Native acceptance found two source-admission defects. `29c1048c700d191e5b3d36e0ba03e4cd8fd76beb` admits identity-fenced one-shot provider imports during active sync; its native run proves Yike→Fox205-move import. `bfdd28fb0b80aa946a603660f7d39f695aa21226` admits the sidecar's explicit Sync request during Yike through the existing SGF-07 transaction. The Windows validation snapshot `367a7ebac2958c1d49b4e754d6fbdf012779f97f` and `bfdd28f` have the identical complete tree `0c80f75054256590c6806d5dd62dfc24343e8611`; the off-branch snapshot preserved the review HEAD while the native repair ran.

Runs use Windows x64 source-runtime, an isolated acceptance application identifier, fixed readboard `cdcc7b382668ea09cb6f1c61bda021f378080eda` (wire220430), and the user's real Fox spectator room45511. The user selected the readboard target and started continuous synchronization; the parent operated Next and collected machine evidence. No engine or provider credentials were required. Evidence root: `D:\dev\weiqi\acceptance\r10-integrated08-367a7eb-run3`; restart runs: `r10-integrated08-367a7eb-restart4` and `r10-integrated08-367a7eb-restart5`.

| Integrated assertion | Observed result |
| --- | --- |
| Explicit Yike→readboard source switch | Yike/session1/doc2 became readboard/session2/doc3. All361 intersections matched the actual Fox screenshot:77 stones, zero differences after excluding the last-move marker from black-stone pixel sampling. Source76→83→91 remained under the same owner. |
| Failure and dirty Cancel preserve ownership | Actual Tencent HTTPS preview failed through an intentionally unavailable proxy; readboard/session2/doc3 continued. Restored Direct preview reached the shared dirty decision; Cancel retained that owner. |
| Active readboard→Tencent import and late-frame exclusion | After user-restored readboard/session5/doc6, explicit Discard imported Tencent chessId1778996477030103932 as79 moves and returned the owner to Idle. The original sidecar remained Ready and reported Fox210 moves; the imported SGF stayed byte-identical. |
| Native Save and browsing | Readboard Save As wrote the invocation-time SGF exactly while retaining synchronization. Yike remained at the browsed root through failure and Retry; the complete129-move tree remained intact. Earlier ticket06 moving-source/delayed-save evidence retains its original candidate attribution. |
| Error-paused, Retry, Stop and mutual exclusion | A real Yike read through unavailable127.0.0.1:1 paused with last-good intact. Restoring Direct did not auto-Retry; explicit Retry retained session3. Stop allowed a committed personal comment. Active Yike disabled Match/Trial entry points; existing owner tests cover the shared Rust admission boundary. |
| Policy invalidation and local transport separation | A controlled Windows loopback proxy held actual `CONNECT api-new.yikeweiqi.com:443` for sync request151/policy10/document8. Switching to Direct/policy11 closed that peer without a response and preserved the SGF. This is controlled transport-cancellation evidence, not installed routing or enterprise trust. Separately, readboard continued through an unusable Manual HTTP proxy. |
| Active exit and document-only recovery | Normal active-Yike exit released Next73556, readboard46040 and their owned sockets; only unowned TCP TIME_WAIT remained. With `restoreLastSession` enabled through Preferences, restart restored the identical129-move SGF, sync/readboard Idle, engine `no_engine`, continuous analysis `safety_hold`. All three final-candidate processes exited normally. |

The first restart used the default disabled restore preference and correctly opened the sample; that attempt does not satisfy recovery acceptance. A browse/Trial attempt after readboard had already become Idle is also NOT RUN for active-sync exclusion. The configured recovery and active-Yike exclusion runs above supply the required evidence. `integration-evidence.json`, `review.json`, `policy-sync/held-proxy-*.json`, resource snapshots and screenshots retain these distinctions.

Inherited01–07 business, browser and readboard protocol/recovery verdicts remain attached to their original candidates. Provider parsers/transport, the readboard decoder/runtime/frame engine, browser opener, resources, manifests and lockfiles are unchanged by the admission repairs. New shared-admission tests and this final integration smoke cover the affected seams. Browser-owned traffic and retention use ticket06's approved public-room/browser split; no website-authentication claim is added.

Parent checks: integration Rust desktop/model/preferences/Fox311, network23, controlled readboard37 and SGF89; original eight-file frontend260 and TypeScript/Vite. After the provider-admission repair: current_game_state159 and ProviderPanel39+App122=161. After explicit tool Start repair: App123+Yike14+Readboard3=140, desktop external_sync28 and TypeScript/Vite. Counts are separate runs with overlapping coverage, not an additive unique total. Both regressions have failed-before/passed-after evidence. Bounded Standards and Spec VERIFICATION returned SUCCESS; STD-01 and SPEC08-01 are resolved, zero new follow-ups. Inherited07 SPEC-06 remains Closed (Spec/DEFERRED/NON_BLOCKING; ClearBoard contract corrected to Java767–768, code unchanged, CLOSE).

**User-approved non-blocking exception (2026-10-06):** installed-network admission on every Shipped Platform remains **Not run**: Direct/Manual/System routes and platform sources, NO_PROXY, a real HTTPS provider, operator enterprise CA, Windows/macOS fixed proxy and PAC, Linux environment. This gate is explicitly non-blocking for R10 completion. Tickets01–09 are complete; ticket09 reconciled every final review against final integration `91b4e1756c509fcc36c38bcd892c5315620f7893` and recorded zero follow-up candidates. Source-runtime results retain their original candidate attribution; installed/platform acceptance is not claimed. NET-01/PROV-01–04/READ-01–02 stay Partial, READ-03 stays Accepted, and Deferred capabilities and release gates are unchanged.

### R10 PR integration checks (2026-10-06)

PR branch `feat/r10-providers-readboard`, worktree `r10-pr-20261006`, starts from merged `main@c1ab59afd138990fa0d30e9ed8e334f1d5d1cc23`. It transplants the R10 delta `566d748..91b4e17` while preserving main's R7–R9 repairs, including failed-Pause departure refusal and Match-start cancellation. The historical Windows candidates above retain their own attribution; this transplant is a different source tree.

- Initial combined candidate: `cargo test --workspace --locked` passed **866**, with **3** opt-in real-engine cases ignored; frontend **40 files / 572 tests** and TypeScript/Vite build passed. Scaffold10, release preflight4 and release-workflow validation passed. Formatting was applied only to R10-changed Rust files; `cargo fmt --all --check` passed.
- PR review exposed a protected readboard Start cancellation race. The owner regression failed with `admitted sync start`, and the rendered control regression dispatched Cancel despite the protected-departure input. The owner now retains the candidate until atomic commit or shared abort; the readboard Cancel control respects that phase. Both regressions pass after repair. The affected App/Yike/readboard frontend run passed **142**, followed by TypeScript/Vite build.
- Strict `cargo clippy --workspace --all-targets --locked -- -D warnings` passed after fixing controlled-read byte handling, using the managed preference state in IPC commands, compacting the internal nonzero start ID without boxing the candidate, and correcting one boolean assertion. Public IPC payloads are unchanged. Network integration **23** and desktop-lib serial **225** passed after repairs. One earlier parallel desktop rerun timed out in the unchanged `unconfirmed_gtp_reap_keeps_occupancy_and_never_authorizes_rebuild` fixture; its isolated rerun passed, as did the complete serial run. The aborted parallel run is not counted as a pass; its timeout cause was not established.
- Actual Linux production-crate probe: Direct HTTPS Yike `live/new-room/186031` returned **232 moves** in **998 ms** via `api-new.yikeweiqi.com:443`; normalized SGF file Save/reopen and selected snapshot matched exactly. Atomic Manual-policy persistence/reload passed; the previous request was cancelled before further I/O, retained its route list and left the saved SGF unchanged; shutdown drained within one second. The throwaway crate and data were removed.
- Real Chromium rendered the production `ReadboardSyncPanel` with controlled phase inputs: protected Cancel was disabled; returning to waiting enabled it and a real click invoked cancellation once. Screenshots were inspected. This is a component browser smoke, not native IPC, readboard-process or installed acceptance. The page and development server were closed; no test-only product entrypoint was retained.

The Matrix reconciles **113** unique IDs: **69 Accepted / 11 Partial / 8 Missing / 25 Deferred**, with each status index matching its rows. `NET-01` is the restored R10 successor. Installed-platform evidence remains Not run under the existing non-blocking exception; no new Windows/macOS or real-engine acceptance is claimed by these PR checks.

### PR #17 review repairs (2026-10-06)

Repair base: `ac78484ec5c8570c60046435c63103d1ef89c6fc`, on the same `r10-pr-20261006` worktree. These checks exercise the working-tree repair delta, not the historical Windows application candidates above.

- Linux CI/release runners now install `pkg-config`, `libcurl4-openssl-dev` and `libkrb5-dev` for the existing curl/GSS backend. Local strict workspace/all-targets Clippy, formatting check, network integration **23**, release preflight **4** and release-workflow validation passed. This local run is not a replacement for the remote Actions result.
- Pending readboard regressions failed before repair: Sync discarded the waiting candidate, and Clear retained stale source-move context (**4** instead of **2**). The owner now preserves Sync candidates and resets context only when discarding a candidate, retaining unconsumed ClearBoard intent across subsequent Start controls. The ClearBoard→Start regression first retained the old two-move history; after repair it commits a fresh setup-only root and removes old player metadata. External-sync owner **31** passed; admitted/active-session behavior is unchanged.
- A headless executable using the production desktop owner installed an initial two-move document through its replacement transaction, delivered ClearBoard→Start→frame, and committed via SGF-07. The original document remained unchanged before commit; afterward the serialized game contained only the new `AB[aa]AW[ba]PL[B]` setup root. The temporary executable source was removed; this controlled owner smoke does not claim a real readboard connection or native UI run.
- The App regression first displayed the cancellation IPC error instead of the original Start error; five runtime-phase cases first displayed a cleanup-failure diagnostic outside `cleanup_failed`. The affected App/readboard/Yike frontend run passed **155**, followed by TypeScript/Vite build. Real Chromium rendered the production readboard panel in stopping, timeout and cleanup-failed states; diagnostic text and Start/Stop availability matched each phase. Controlled snapshots were used, so this is not native IPC acceptance.
- Windows `provider-core --lib` passed **22**, including per-URL static proxy/bypass/direct selection after WPAD-only `12180`, configured-PAC refusal, terminal errors, PAC-selected route precedence and native request-error callback decoding. An actual WinHTTP probe initially timed out for both WPAD absence and a refused loopback PAC URL: the request-error callback constant differed from the Windows SDK. After correcting it to `0x00200000`, the same native probe observed WPAD discovery absence, selected the configured static proxy, returned `ProxyFailed` for the explicit PAC URL and preserved cancellation. The probe used production resolver code with isolated input configuration; it did not alter system proxy settings or send a provider request through the selected proxy. Existing Windows FFI naming warnings remain; this is not a Windows strict-lint pass.
- Installed-platform admission, enterprise trust/authentication, real readboard/Fox interaction and full native application acceptance remain separate. This repair does not upgrade Matrix statuses or claim those gates passed.

### STD-CI-11 controlled-engine cleanup regression (2026-10-06)

Repair base: `7211aefe5d5a43deeabf4d8df68946e33499d8a5`, same PR #17 worktree. Actions run `37536031659` failed because the handshake-cancellation test required its first abort to succeed. A controlled trace reproduced the same error: after SIGKILL the candidate remained runnable, while the caller spent its 150ms budget waiting for another reaper's Child mutex. Cleanup correctly returned Timeout and retained ownership; the old Ready engine survived. The trace establishes a reachable local mechanism, not an exact reconstruction of the original runner's scheduling.

- Only the test changes: accept the typed candidate cleanup Timeout through explicit, bounded Retry Stop, require preparation Cancellation, and still require the candidate reaped, reservation released and old Ready engine preserved. Production cleanup code and the existing 150ms per-call budget are unchanged; the three-second test bound limits the explicit retry sequence, not an individual cleanup call.
- A separate zero-observation-budget case covers both controlled GTP and KataGo adapters. The first abort must return Timeout with the candidate's identity; reservation, new-work refusal and profile-deletion protection remain until a later abort confirms reaping. Successful cleanup releases those protections without killing the old engine. The test explicitly reaps that old engine during final teardown as well.
- Focused abort tests **2/2**, the adjacent `game_move` suite **39/39**, and `cargo clippy -p engine-manager --test game_move -- -D warnings` passed. These counts overlap.
- A temporary standalone executable exercised the production manager with the original pipe fixture: unrestricted **2/2** cases and one CPU with 32 bounded load processes, four drivers and two GTP/KataGo iterations each, **16/16** cases passed. One pressured case returned Timeout on its first abort, then completed after **one** explicit retry. All cases confirmed candidate reaping, reservation release and preservation of the old Ready engine; all **36** owned engine PIDs were absent after teardown. The temporary executable source was removed. This is controlled-process evidence, not real-engine/native acceptance or a new remote CI result.

### R10 ticket05 Tencent source and native evidence (2026-10-05)

Implementation starts at `116199cc84341c57d29632771b51bbbc5d23daac` on `feat/r10-network-01`, containing ticket01 final `fcaf0f1420811214367b6a552f85649b5e5ab873`. `provider-tencent` owns the independent frozen CGI contract; `TencentKifuCenter` owns explicit username/chessId selection, cached 25-row pages and preview-before-import.

Focused checks: Tencent provider **16**, history transaction **1**, provider lease/document rejection **1**, app-preferences **44**, Tencent UI **34**, existing ProviderPanel **27**, NetworkSettingsPanel **5**, preference API **18**, shared network **16**, document-departure **20** passed (**182 distinct focused cases**). TypeScript/Vite build passed. Cases include raw-tail cursor before dedup, short-batch continuation, 60/25 partial-page fill, stale identity rejection, full-tree escape/setup/variation preservation, komi conversion, ordered history clear/write and failed persistence. Three lifecycle regressions failed before repair and passed afterward; accepted writes now survive center unmount, retain action order across remount, and expose failed intent for retry.

The production `NetworkState` + `provider-tencent` + SGF parser ran Direct HTTPS without credentials or an engine: username `br胸怀大志` returned **60** records in1488ms; `lastCode=1786452851030104419` returned the next **60** in1478ms (new tail1778950448030102634). Selection1786700326030101944 parsed140 moves in1056ms; direct chessId1778996477030103932 parsed79 moves in1038ms. Both retained komi7.5; routes were `cgi.huanle.qq.com:443` and `happyapp.huanle.qq.com:443`. Missing chessId0 returned terminal `invalid_payload` in1029ms without exposing the upstream body. Cancelled lease rejected before I/O; shutdown completed within its one-second budget. These are working-tree source-runtime observations, not a committed Windows candidate or installed admission.

Windows native source candidates ran on private Win32 desktops with dedicated acceptance identifiers and no engine or credentials:

- `844fa4c2ae6a414c4e3c10cbf7c84c2db7bc9b3c`: actual 同步 → 腾讯棋谱 entry; real username pages25/25/10, next batch fills page3 to25 before page4, first100 records distinct; selection preview preserves the current SGF, dirty Cancel preserves it, Discard imports140 moves. Direct chessId1778996477030103932 previews then imports79 moves; chessId0 shows a typed error without mutation. Native Save As and native reopen preserve the exact normalized79-move SGF, including komi7.5. Evidence: `D:\dev\weiqi\acceptance\r10-tencent05-844fa4c-run1\tencent-evidence.json`, screenshots, `save-as.json`, `reopen.json` and `tencent79.sgf`.
- Repair candidate `e201634b613e1ea766117e53df99a7a021fdc8a7`: real username and direct-chessId preview/import pass again. An owned loopback proxy held actual `CONNECT happyapp.huanle.qq.com:443`; Cancel returned the visible unchanged-game state in38ms, and restoring Direct allowed real preview/import. This is controlled latency, not an upstream outage. A Windows file handle denying replacement of this candidate's preference file produced a visible save failure; switching Fox/Tencent retained the error and Retry, releasing the handle let Retry clear durable history without changing the current game. Normal exit/restart restored only the saved query/recents, with no list, preview or active request restored; explicit Clear persisted empty history. Evidence: `D:\dev\weiqi\acceptance\r10-tencent05-e201634-run1\tencent-evidence.json`, screenshots and `...-run2\tencent-restart-evidence.json`.
- Only the Tencent frontend history lifetime/tests and contract prose changed between candidates. Provider/shared transport/parser/gateway/SGF-07, resources and build configuration are unchanged; the final candidate's affected-history and real-provider integration smoke supports reuse of the first candidate's pagination/dirty-decision/native-file verdicts without relabeling their SHA. All three isolated native processes exited normally.
- Two-axis repair review: `SUCCESS` after FULL_REVIEW and targeted VERIFICATION. `STD-01` (accepted history writes lost on unmount) is resolved; Spec has no findings; zero open blockers and zero follow-up items. Review base `116199cc84341c57d29632771b51bbbc5d23daac`; isolated review HEAD `844fa4c2ae6a414c4e3c10cbf7c84c2db7bc9b3c` plus the byte-equivalent two-file repair; target stability checked before/after both batches. Record: `...\r10-tencent05-844fa4c-run1\review.json`.

Ticket05 business acceptance is complete. PROV-04 stays Partial for ticket08 platform/installed network admission. Ticket01 transport/trust/settings ownership is unchanged in these Tencent candidates; its historical Windows routes remain attributed to `b36ea007f436ff4492b464f57fff7116852e7d84`. Native PAC/WPAD, enterprise trust/auth, other shipped platforms and installed admission are not claimed by these source-candidate runs.

### 1. Yike Runtime Fetch

- Start the Tauri desktop runtime and configure the Yike provider inputs required by the implementation.
- Open Yike Public Center: Recommend is the default; select Local, page forward/back, select a record or paste a supported public locator, then preview before explicit Import.
- Record public locator, exact candidate SHA, effective route, result count and latency. Preview and closing must preserve the current document; Import uses the existing Save/Discard/Cancel gate.
- Authentication and missing-SGF responses are terminal public-service failures. Do not supply browser credentials or call a different family to claim the requested family passed.

Expected result: successful fetches return normalized DTOs, and auth/network failures return structured errors without stale cached data being presented as live data.

NET-01 platform checks are separate: native Windows/macOS resolver code must compile for its target; portable resolver helper tests do not prove OS PAC/WPAD, authentication or installed trust. R10 ticket08 owns the final native-platform/enterprise-CA/installed matrix. Keep unavailable environments Not run and NET-01 Partial until that matrix is admitted.

#### R10 ticket01 network cutover evidence

Windows source-runtime candidate `b36ea007f436ff4492b464f57fff7116852e7d84` ran on a private Win32 desktop with no engine selected. Acceptance identifier: `org.lizzieyzy.next.acceptance.rca9640c1d270485d93835b5eb2bf218a`; candidate: `D:\dev\weiqi\worktrees\lizzieyzy-next-tauri\r10-net01-b36ea00\candidate.json`. Runs `D:\dev\weiqi\acceptance\r10-net01-b36ea00-run1` and `...-run2` retain process/window identities. `network-evidence.json`, `manual-preview.webp`, `manual-import.webp` (run1) and `direct-import.webp` (run2) retain the observations.

- Manual `127.0.0.1:17897`: the controlled loopback observer received `CONNECT api-new.yikeweiqi.com:443`; native UI showed the same route. Public signed Yike detail186031 preview parsed 232 moves. Serialized current SGF remained byte-for-byte unchanged before Import; explicit SGF-07 Import installed 232 moves and `yike-186031.sgf`.
- System: native resolver reported `windows-direct` for the same HTTPS target; preview preserved the imported game. Manual on unused port17898 failed with curl7 and no preview/direct fallback. Restoring port17897 and restarting retained Manual/host/port; revision restarted at1, while in-flight identities were not restored.
- After restart, Direct preview/import again installed 232 moves. A native `provider_fetch_fox` HTTPS request to `self-signed.badssl.com` returned `tls_failed` (curl60), with no certificate bypass. Both isolated processes exited normally.
- Repository checks: transport14, Fox14, network-preference1, provider gateway/import10, app-preferences44, document-departure20, provider/settings UI16. Shared App/preferences run passed157 and exposed seven incidental argument-arity assertions; removing those assertions retained the behavioral cases, and affected replacement/activation24 passed. TypeScript/Vite build passed. Both native adapter sources compiled for `aarch64-apple-darwin` and `x86_64-pc-windows-gnu` in an isolated check harness; their portable production-helper tests passed21. Windows full Tauri build and native runtime passed.
- After that Windows candidate, macOS-only resolution was corrected so CFNetwork's HTTPS proxy type selects an HTTP CONNECT endpoint, and the system auto-discovery flag explicitly invokes native WPAD via an empty PAC URL. These changes do not alter the Windows candidate's executed code; cross-compilation is not a native macOS smoke. Apple API interpretation follows Chromium's [proxy mapping](https://chromium.googlesource.com/chromium/src/+/main/net/proxy_resolution/proxy_chain_util_apple.cc) and [native resolver](https://chromium.googlesource.com/chromium/src/+/main/net/proxy_resolution/proxy_resolver_apple.cc).
- Review repairs preserve the native PAC input URL without injected default ports and bound shutdown's policy/lease lock waits. Public production-boundary probes changed from URL mismatch and a 30ms shutdown still blocked at200ms to preserved HTTP/HTTPS/nondefault-port URLs and timeout at30.1ms. Two new held-persistence/held-install regressions fail before the fix and pass afterward; the full transport suite now passes16, network persistence1 and document-departure20 pass again. Native PAC routing itself remains Not run, and the Windows runtime evidence above remains attributed to its original candidate.

Not run: native macOS, Windows PAC/WPAD network, enterprise authentication/CA, installed artifacts, and live Fox games. Resolver helper tests/cross-compilation are not native-platform admission. Those platform/environment rows stay with ticket08; provider-specific workflows stay with their own tickets. NET-01 remains Partial. The completion record owns final review and documentation commit identity; this runtime evidence keeps its exact candidate attribution.

#### R10 ticket03 public-center evidence

Repository checks on the ticket03 working tree from `fcaf0f1420811214367b6a552f85649b5e5ab873`: provider-yike **27 passed** (15 unit + 12 public-family tests), app-preferences **44 passed**, narrow locator persistence **1 passed**, provider gateway/identity **6 passed**, SGF-07 document-departure **20 passed**. Frontend ProviderPanel **27**, NetworkSettingsPanel **5**, preference API **18** passed; TypeScript/Vite build passed. Fixtures cover strict pre-I/O locator rejection, all five response mappings, complete source branches, pagination/end/malformed responses, stale responses and transactional cancellation/failure. They do not prove public upstream readability.

The actual production `NetworkState` + `provider-yike` + SGF parser ran Direct HTTPS on 2026-10-04, with no engine or user credentials. Recommend and Local each returned 20 records and successful page2; selections196107/196106 parsed150/119 moves. Known new-live186031 parsed232 moves in944ms; old-live `18328/1/15630642` and short old-board18328 each parsed166 moves in1220/1181ms. Routes were `direct` to `api.yikeweiqi.com:443` or `api-new.yikeweiqi.com:443`. Counts for ongoing games belong to this run, not a stable revision identifier.

Remaining live gates: game/watch room15630642 and equivalent hall URL returned a response without SGF (1131/917ms); unite186031 returned typed `authentication_failed` (1124ms) from `game-server.yikeweiqi.com:443`. Invalid `example.com` locator rejected in0ms with no routes/I/O. An initial separate Python probe rejected Recommend, but the production Rust list/selection/page2 succeeded; that probe is not the product verdict. PROV-01 stays Partial; game/hall and unite need publicly readable source evidence, not auth expansion or fixture substitution. Platform/installed admission remains ticket08's separate gate.

Windows native source-candidate smoke passed on **2026-10-05 UTC**, exact committed source **`443c3e322e6d3d2abcedadb76e8901ad778bf4f8`**, private Win32 desktop / Tauri WebView2, Direct, no engine. Candidate: `D:\dev\weiqi\worktrees\lizzieyzy-next-tauri\r10-yike03-443c3e3\candidate.json`; evidence: `D:\dev\weiqi\acceptance\r10-yike03-443c3e3-run1\yike-native-evidence.json`, screenshots, native file-dialog records and `run.json`. Isolated application identifier: `org.lizzieyzy.next.acceptance.r577c39f0ac174dada0786264ea1e1934`.

- Recommend196107 preview left the sample unchanged; explicit Import installed163 moves. Native Save As and Open round-tripped exactly the serialized SGF. Local returned20 records; selection196106 preview preserved the current game and Import installed121 moves.
- Old live `18328/1/15630642` and old board18328 each previewed/imported166 moves. A personal-comment edit made the original dirty: preview and Cancel retained its SGF exactly; a subsequent Save decision used the native Save As dialog, saved the original exactly and then installed the clean source tree.
- URL-input ArrowLeft retained the selected board move; closing the provider panel retained the current SGF. The last successful canonical locator remained old-board18328 after failed queries. The app exited normally with code0; no engine was started.
- Native game/hall reads remained blocked at990/998ms, and unite at1192ms with visible `Yike unite public access was rejected`; all retained the current SGF. These failures confirm the open ticket03 gates above, not successful five-family acceptance. Source-runtime results and native observations retain separate timestamps and move counts.

Standards and Spec FULL_REVIEW both returned no source findings against base `fcaf0f1420811214367b6a552f85649b5e5ab873` and candidate `443c3e322e6d3d2abcedadb76e8901ad778bf4f8`; the frozen target recheck matched. Repair-loop verdict: SUCCESS, zero follow-up items. Ticket03 remains blocked on game/hall and unite public preview/import; review success does not close those gates. Subsequent evidence-only documentation commits do not change this runtime candidate's attribution.

#### Ticket03 anonymous-unite repair (2026-10-05)

Starting HEAD: `544f97d8f6cb225445357f3446bae16b6241b594`, branch `feat/r10-network-01`. The user chose to resolve ticket03 before beginning ticket06; neither ticket is closed by this repair alone.

First-party `home.yikeweiqi.com` desktop/vendor bundles show public `POST /v1/user/tpop/authorizations/anonymous` followed by guest-authorized `game-server /game/info`. Independent cookie-free bootstrap and Go-list reads succeeded. The historical unite ID186031 is actually `game_type=fir` (five-in-a-row), not Go. Room79438407 is `territory`, 19×19, Chinese rules; its public SGF changed from11 moves during investigation to57 in the production Rust probe. This is live acquisition evidence, not a Next ongoing-sync session.

The initial repaired production `NetworkState` + `provider-yike` Direct path fetched room79438407 in2703ms and rejected unite186031 as `invalid_payload` in2022ms. After metadata normalization, the same path passed in2195ms with `PB[毛志方]PW[陈方明]`; non-Go rejection took2041ms. Both route lists contain only `api-new.yikeweiqi.com:443` and `game-server.yikeweiqi.com:443`; no user/browser credentials were read or persisted. Network shutdown completed within the probe's one-second budget. `cargo test -p provider-yike --offline`: **31 passed** (15 unit +16 integration), including failing-before/passing-after anonymous access, non-Go admission and SGF-only metadata roundtrip regressions, existing-source precedence, room mismatch and bootstrap-secret sanitization.

Spec review found that unite's separate player/result metadata would otherwise be lost at gateway enrichment and SGF-only import (`SPEC-C1`). A focused regression reproduced missing `black_name`; the provider now fills only absent root `PB/PW/RE` using the existing SGF parser/serializer. Save/reopen preserves escaped/team names, result, setup, comments and both variations; source metadata remains authoritative. Source probes before the exact native candidate are working-tree evidence based on the starting HEAD above.

Repair review against base `544f97d8f6cb225445357f3446bae16b6241b594`: FULL_REVIEW Standards found no issues; Spec admitted `SPEC-C1` after the focused failure. Bounded VERIFICATION on both axes resolved it with no new findings, and the frozen target recheck matched. Source repair-loop verdict: **SUCCESS**, zero open in-scope findings and zero follow-up items. Native acceptance and the game/hall gate remain separate.

Windows native repair candidate: **`6d41c36b925b615583c160796dcdcc582b95561f`**, built from the committed WSL source into `D:\dev\weiqi\worktrees\lizzieyzy-next-tauri\r10-yike03-unite-6d41c36`. Isolated application ID: `org.lizzieyzy.next.acceptance.r15e3407e1ef648979ffb496870423bbd`; no engine was loaded. This is source-candidate acceptance, not installed/release or cross-platform network admission.

- `npm run desktop:smoke:windows -- --candidate <candidate.json> --run-directory <fresh-directory>`: **5/5 PASS** (startup/identity, native Open/navigation, Open cancel, Save As/reopen, clean exit). Evidence: `D:\dev\weiqi\acceptance\r10-yike03-unite-6d41c36-files\smoke.json`; PID58280.
- Real unite native run: **7/7 PASS**, PID90600 on isolated desktop `weiqi-b9cd22b0ac1544fa8129c43782966365`: non-mutating live preview; dirty Import→Cancel; explicit Import→Discard; native Save As; replace with a distinct9×9 document then native reopen; non-Go rejection preserving document/selection/locator; clean exit0. Evidence: `D:\dev\weiqi\acceptance\r10-yike03-unite-6d41c36-live\unite-smoke.json`, `run.json`, native-dialog JSON/PNG records and four WebView screenshots.
- Room79438407 preview became ready in2334ms (UI-observed elapsed), with245 moves. Import/Save/reopen retained the complete serialized SGF, `PB[毛志方]PW[陈方明]RE[b+]`, 19×19 dimensions and komi7.5. Cancel retained the exact dirty sample SGF and selected move20. After saving/reopening, room186031 was rejected with no Import control, while the245-move document, selection and persisted canonical room79438407 locator remained unchanged. The real source's result spelling is preserved; offline regression separately covers `B+R`, escaping, team names, branches/setup/comments and source-property precedence.

Source-runtime artifacts: `D:\dev\weiqi\acceptance\r10-yike03-unite-source-live.json` (initial acquisition and legacy-service diagnosis) and `r10-yike03-metadata-source-live.json` (post-normalization probe). The native candidate, working-tree probes and inherited ticket03 evidence retain their own attribution. Runtime source is unchanged by subsequent evidence-only commits.

**Handoff:** unite acquisition/normalization/native preview/import/Save/reopen is now verified. Ticket03 remains **BLOCKED** only on its game/hall family gate; PROV-01 remains Partial. Ticket06 has not started because the user chose to close03 first. Its original investigation baseline remains `566d748cf63c4a36bd54c4b7c3d8aec65643cb8f`; no new implementation baseline or sync acceptance is claimed. This continuation is recorded here within the authorized repair paths; the planning worktree's local tickets are unchanged.

Game/hall remains unresolved: frozen Java `OnlineDialog.req2/login/initData` reads `game_info.sgf` over legacy `https://rtgame.yikeweiqi.com` Socket.IO. A room ID cannot be treated as a live-board ID to claim success. The current Next HTTP lookup still lacks SGF; this run's independent direct connection to legacy port443 failed in240ms. The anonymous current website's legacy room page rendered blank. A reachable genuine game/hall service/source is still needed before protocol repair and native success can be established.



### 2. Fox Runtime Fetch

- Start the Tauri desktop runtime; no Fox account, credential or engine is required.
- In ProviderPanel select Fox. Look up a nickname, a UID and a chessid. Page through 25-row pages until a new batch loads, and continue to the end of a short list.
- Select a listed game: preview only, current SGF unchanged. Import explicitly; verify the SGF-07 prompt, Cancel keeps the original game, and Import installs the previewed game.
- Record route, result counts, latency, recents (≤8, restart restores, Clear empties) and upstream failure/delay results.

Expected result: lists and previews are normalized DTOs, preview never changes the current game, and upstream failures are typed, visible and retryable.

#### R10 ticket04 Fox kifu evidence

Repository checks at the final candidate: provider-fox **19 passed** (lookup validation before I/O, nickname not-found, 100-row cap and `lastCode` continuation without cursor/duplicate rows, empty/short end, typed malformed lists, CGI miss/HTML fallthrough to H5, H5 miss `not_found`, malformed CGI terminal after one request, escaped line breaks between properties, professional rank codes, terminal classified failures, recents cap/dedupe/sanitize); full `sgf` suite passed; provider-core network **16 passed**; desktop `fox_`/`provider_`/`document_departure` **28 passed** (including Fox recents atomic write, stale full-save preservation, restart and Clear); app-preferences **44 passed**; ProviderPanel **18** + NetworkSettingsPanel **5** passed (including non-aligned continuation, back navigation during a continuation and empty-list recents); preferences API/App preferences **42** passed; TypeScript/Vite build passed.

Production-crate live probe (2026-10-05, Direct, actual `NetworkState` + `provider-fox` + SGF parser, no credentials or engine): nickname 绝艺 resolved UID 8772065 and returned 100 games in 4561ms via `newframe.foxwq.com:443` then `h5.foxwq.com:443`; two continuations returned 100/100 games with zero repeats (4422/2919ms). UID 3490876 returned nickname 姜小二 and 100 games (3035ms), continuations 100/100 without repeats. Unknown nickname returned `not_found` in 1001ms after one request. Chessid 1566713791010001299 parsed 246 moves from the first CGI endpoint (661ms); recent chessid 1791172882030077720 fell through a CGI miss and an HTML CGI page to H5 and parsed (3776ms); chessid 123 returned `not_found` after all three paths (3360ms). Upstream investigation found the list endpoint ignores the frozen Java `lastcode` parameter (it repeats the first batch); only `lastCode` advances. This probe is provider-live crate evidence, not native UI evidence.

Native Windows source-runtime evidence (Direct, isolated desktop, no engine or credentials) on final candidate `f1270ba713b8b6d12ef66045951d37d21f620575`, record `D:\dev\weiqi\acceptance\r10-fox04-f1270ba-run1\acceptance.json`: nickname 柯洁 listed page 1/4… with professional ranks (4238ms); the fourth Next requested a continuation and showed row 101 on page 5 (3184ms); a listed game previewed 158 moves through CGI miss → HTML → H5 with the current SGF byte-identical; UID 3000000 reached its end on page 3/3; chessid 1785337045010001403 previewed and imported 205 moves; a dirty document's SGF-07 Cancel kept its SGF byte-identical and Discard installed the previewed 13×13 game; an empty UID kept recents unchanged; restart (`-run3`) restored recents and the last lookup, a recent re-ran by UID and Clear emptied recents. Earlier candidates `6e2341b` and `cff5fb8` exposed and then confirmed the professional-rank, empty-list recents, page-fencing and escaped-line-break repairs. Not run: native macOS/Linux, System/Manual routes for Fox, installed artifacts and induced live timeouts; platform/installed network admission remains ticket08's.

### 3. readboard Native Readiness (R10 ticket 02)

- In native Windows Next, open Sync → Fox / Yike / readboard, Browse to the real `readboard.exe`, and Save path. The executable and its dependencies must remain together. Start uses only the saved path, not an unsaved draft.
- Start binds a dynamic loopback endpoint before launching the child. Record PID, endpoint, generation/revision and exact `220430` wire. Only `ready` followed by the requested version response counts as ready; file existence or a TCP connection does not.
- Stop/cancel startup must invalidate the old generation, close the socket and reap the owned child. Restart must produce a new PID/generation with no old child or connection remaining. Confirm no engine is required and SGF/dirty/selection remain unchanged.
- Close a dirty application, Cancel once, then confirm Discard or Save. Cancel preserves the ready runtime; confirmed exit shares APP-03's teardown deadline, with existing Retry / Exit anyway on failure. Check actual OS processes/connections, not only the UI. Reopen retains the saved path but does not relaunch readboard.
- Missing runtime/platform is `unavailable`; wrong/future wire is `incompatible`; no complete handshake reaches `timeout`; exit/disconnect are separate phases. Use controlled child/stream cases for unsafe-to-induce faults, separately from real-runtime evidence.

#### Recorded source-runtime evidence

Investigation/implementation start: `566d748cf63c4a36bd54c4b7c3d8aec65643cb8f`. Windows Next candidate: `eadb577257dd8b08359e2a91620026d377b76670`, built from the isolated `feat/r10-readboard-02` worktree. Actual readboard: `cdcc7b382668ea09cb6f1c61bda021f378080eda`, v3.1.0 / wire220430. Both were prepared with `/home/dev/dev/weiqi/bin/prepare-windows-candidate --source <owning WSL checkout> --commit <exact SHA> --project <project> --destination <fresh destination> --timeout 900`; no push or daily-clone changes.

Native run on 2026-10-05 UTC used private Win32 desktop `weiqi-40f21102d2384903b2029897d8c565f1`, application ID `org.lizzieyzy.next.acceptance.r6e1a666eb50b4e328ee5795111c85834`, and CDP port9497. Next PID79044 launched actual readboard PID22344 at `127.0.0.1:3800`, reaching ready220430. Stop removed PID22344 and all owned TCP connections. Start created PID32932; Restart reaped it and reached ready with PID85356/generation5 at `127.0.0.1:11726`. Dirty exit Cancel retained that exact runtime and document. File Exit → Discard ended Next with code0; OS observation found neither Next nor PID85356 nor their connections. Reopen PID16748 retained the saved path, was idle with no child, had no engine, and exited code0. SGF including an unsaved personal comment, dirty=true, selection, generation and snapshot sequence remained identical throughout readiness/Stop/Restart/Cancel.

Evidence: `D:\dev\weiqi\acceptance\r10-readboard-02-assets\` contains `browse.json/png`, `ready-ui.png`, `ready-state.json`, `ready-processes.json`, `stopped-state.json`, `stopped-processes.json`, `restarted-state.json`, `restarted-processes.json`, `exit-cancel.json`, `exited-processes.json`, and `reopened-state.json`. Run records: `D:\dev\weiqi\acceptance\r10-readboard-02-runtime1-run1\run.json` and `...-run2\run.json`. Candidate manifests are under `D:\dev\weiqi\worktrees\lizzieyzy-next-tauri\r10-readboard-02-runtime1\` and `D:\dev\weiqi\worktrees\readboard\r10-readiness-02-cdcc7b3\`.

Focused repository checks passed: readboard lib20 (including8 controlled lifecycle cases), desktop `document_departure`20, `continuous_analysis::tests`6 (including durable readboard path/stale-save/failure), `readboard_sidecar_sync`2, app-preferences44, frontend `ReadboardPanel App.preferences`31 (7+24), and `npm run desktop:build`. Controlled cases cover wrong/future/empty/missing wire, bare TCP, split lines, version-before-ready, no-connect timeout, exit23, disconnect, bounded line length, cancel, superseded restart and kill/reap of a child ignoring quit. These are Linux controlled-process/repository checks, not Windows fault or real target-board evidence.

Ticket 07 changed the runtime event subscription to one ordered stream, so this readiness path was re-run natively on Next `f75cacf7c21b3b2252a16dfcb5fa833caae79e95` with the same readboard `cdcc7b3…` (§4 recorded evidence): Browse/Save path, ready220430, Stop reaping the child and socket, Start, Restart to a new PID/generation, and saved path plus preferences retained after reopen all passed.

This proves READ-01 lifecycle only. Ticket 07 owns complete-frame delivery and target synchronization (§4), reusing the crate owner/generation/event seam in ARCHITECTURE_NEXT. OCR remains unsupported; no target recognition, GAME-10 play-back, installed/release, other-platform runtime or provider-live acceptance is claimed. Later changes to argv, wire, lifecycle, shared exit or build dependencies require affected native revalidation; documentation-only descendants retain this candidate's attribution.

### 4. readboard Sync (R10 ticket 07)

- Start readboard to Ready (§3), open a real target board, then press Start in the sync sheet (or readboard's own sync button). A dirty current game shows one Save/Discard/Cancel prompt before the first frame is installed; later frames do not prompt.
- Play several moves on the target and confirm coordinates, stones, move number, side to play and captures; the board stays read-only (no play, comment, Match or Trial) while browsing and existing analysis remain available.
- Change the target (new game/room) and confirm the rebuild keeps one static setup root without PASS nodes; Save and reopen the SGF.
- Close readboard or stop its runtime: the session enters Error-paused with the last-good board. Retry restarts the runtime and resumes on the new generation; Stop leaves an editable game.
- Toggle the four preferences (always-sync, focus, mute, jump-to-last) and confirm each persists across restart independently.

Expected result: only frames from the bound runtime generation change the current game; no stale board survives restart, target change or disconnect. Repository/controlled evidence does not substitute for this fixed Windows run.

#### Recorded Windows evidence (2026-10-06)

Next candidate `f75cacf7c21b3b2252a16dfcb5fa833caae79e95` (`feat/r10-readboard-07`, implementation start `7283b3f7579b5e8b96151eaf680516a5a1204769`) was prepared at `D:\dev\weiqi\worktrees\lizzieyzy-next-tauri\r10-readboard-07-f75cacf\` with `prepare-windows-candidate`; readboard is the §3 candidate `cdcc7b3…` (v3.1.0, wire220430). The target is the real Fox client `foxwq.exe`, with readboard in Fox mode on live spectator rooms 45009, 45305, 45260, 45388 and 45394. The user logged in and chose rooms; no credentials were handled. Runs used the interactive desktop, an isolated acceptance application id and CDP ports 9417–9420.

Harness finding: the acceptance launcher sets `WEBVIEW2_USER_DATA_FOLDER` for Next. Next's readboard child inherits it, and readboard's own WebView2 then fails with 0x8007139F; a probe showed the variable alone causes this. Runs 2–4 used a launcher copy that leaves the variable unset (the isolated application id still isolates Next's profile). This is an acceptance-environment issue, not candidate behavior.

- §3 revalidation (run1, Next PID43268): native Browse/Save, ready gen1 PID77284 at `127.0.0.1:3099`; Stop gen2 reaped the child and closed the socket; Start gen3 PID69960; Restart gen5 PID34248 at `127.0.0.1:4759` with the old child gone. Run2 (PID17396) and run3 (PID51148) reopened with the saved path and preferences, reached ready with one readboard WebView2 child, and repeated Stop/Restart.
- SGF-07 Start: a dirty game (unsaved personal comment) produced exactly one Save/Discard/Cancel prompt; Discard installed session 1 as a static setup root at Fox move 124 (`Rebuilt from a static readboard snapshot…`). A clean game installed without a prompt (run3, session 2). Pressing readboard's own sync button with no Next session followed the same SGF-07 prompt (run3, session 3).
- Continuous updates: room 45009 advanced from move 124 to 172 as real B/W child nodes with the source tip following. For each sampled position, every Fox intersection (sampled from a PrintWindow capture of the Fox window) matched Next's authoritative position, except the newest stone: the Fox last-move marker defeats the luminance sampler, and Next's `last_move` is that point. Captures and side to play were recorded with each sample.
- Target change: switching rooms rebuilt a single static AB/AW root with no PASS node, then appended live moves in the frame colour (45009→45305; →45260 root 92B/90W; →45394 root 55B/53W at Fox move 109). Readboard's empty loading frames did not change the game. On `f75cacf` the rebuilt root's `PL` was not reliable; the diagnosis and repair follow below.
- Read-only and disconnect: force-killing readboard moved the session to Error-paused (`runtime_unavailable`, os error 10054) with the last-good board, and a comment edit was refused (`External sync owns this read-only game…`). Retry started a new runtime (run2 gen8 PID78948; run3 gen3 PID53700) and resumed the same session on that generation, rebuilding after the moves missed during the gap. Closing the readboard window normally sends `stopsync`, which ended the session and left an editable game.
- Stop: the session went idle with the readboard runtime still ready, and a personal comment edit succeeded. Native Save As wrote `07-saved.sgf`; after New, native Open restored identical SGF text with the comment and a clean document.
- Preferences: always-sync, focus, mute and jump-to-last were toggled one at a time; each was saved independently, and run4 (PID60884) loaded `false/false/false/true` after restart. The defaults were then restored.

Evidence: `D:\dev\weiqi\acceptance\r10-readboard07-f75cacf-run1\` … `-run5\` (`run.json`, `evidence\`, the window-only `capture-screen.ps1` and `compare-board.ps1`); the harness copy is in `D:\dev\weiqi\acceptance\r10-readboard07-tools\`. Readboard frame diagnostics are under that build's `debug-diagnostics\`.

#### Room-change `PL` diagnosis (run5, 2026-10-06)

Run5 (Next PID63564, readboard gen1 PID49292) switched among Fox rooms 45394, 45388, 45312, 45518, 45350, 45347 and 45130 with readboard frame diagnostics enabled. An in-page 100 ms trace of the Next root and tip (`evidence\trace.json`) and a Fox/readboard window-title log (`titles.jsonl`) were matched to each diagnostic frame's payload (`evidence\pl-analysis.json`). Every room-change frame carried exactly one last-move marker with `lastMoveSource foxCornerFlip`, which is a trusted source.

| Rebuild (UTC) | Marker colour in frame | Source to play | Old-tip baseline | Next root `PL` |
| --- | --- | --- | --- | --- |
| 16:26:30 | B | W | W | W |
| 16:27:14 | B | W | W | W |
| 16:27:58 | W | B | W | **W** |
| 16:28:20 | B | W | B | **B** |

Cause on `f75cacf`: `Delta::summarize` (like frozen Java `SyncSnapshotClassifier.summarizeDelta`, `7b40275` lines 24–43) records the marker and the stone differences in one row-major pass and returns at the first point that changed between black and white. A different room's board almost always has such a point early (indices 21–30 here), before the marker (indices 45–216), so the marker was never recorded. Turn inference then fell back to the old room's side to play, so root `PL` copied the old room's turn. Run2's first switch is consistent with this but has no frame record: a snapshot 26 s before the switch showed 45009 at Fox move 172, and `PL[W]` follows if Black's move 173 arrived before the switch.

Repair (Next deviation from frozen Java, recorded in ARCHITECTURE_NEXT): the trusted-marker turn signal reads the frame's single marker over the whole frame (`a5a4464`). On a non-empty frame, that marker also decides the rebuilt root's `PL` ahead of a `PL` inherited from the old anchor, and the old `PL` is not carried forward (`eb5736a`). Without that second change, a later session reads the `PL` materialised by an earlier session's rebuild as explicit and keeps it after a room change. Delta classification, incremental and single-move recovery, and marker-based move numbers keep the Java scan. Regression tests: `room_change_rebuild_takes_the_side_to_play_from_a_trusted_marker_after_a_colour_flip` and `a_trusted_marker_supersedes_an_earlier_session_root_pl_without_rebuilding_again`, both failing before their fix; `cargo test -p sgf` lib 89 and desktop lib 224 pass. Replaying run5's real frames offline through the engine gives the wrong `PL` before the first fix and the frame-marker `PL` after it.

Windows re-check, with readboard frame diagnostics on and each rebuild matched to its frame (`evidence\trace.json`, `evidence\pl-check*.json`):

- Run6, candidate `a5a4464` (`D:\dev\weiqi\acceptance\r10-readboard07-a5a4464-run6\`, Next PID3680, readboard PID47964, rooms 45518, 45511, 45362, 45509, 45203 and 44356): four room changes where the old room had White to play and the new room Black. Before the fix these rebuilt as `PL[W]`; all four now wrote `PL[B]`, matching the frame's trusted marker and Black's next move. A later Start on the document left by session 1 kept its root `PL[B]` although the new room's trusted marker showed Black had just moved. That second case was fixed in `eb5736a`.
- Run7, candidate `eb5736a` (`D:\dev\weiqi\acceptance\r10-readboard07-eb5736a-run7\`, Next PID14684, readboard gen1 PID63548): the SGF-07 Start candidate plus nine rebuilds. All seven rebuilds whose frame had a trusted `foxCornerFlip` marker wrote the `PL` that marker implies; four of them changed the root's previous `PL`. The Start candidate and one rebuild had only a `deviation` marker and kept the baseline, as the frozen rule requires, and the next trusted frame corrected the root. The one frame without a marker (6 black stones, `lastMoveSource none`) also kept the baseline. Each of the three first moves appended after a rebuilt root was played by the side its `PL` named.

Engine play-back (`GAME-10`), OCR, installed/release and other-platform runtimes are not claimed. Readboard frames that lack a trusted marker still leave the root `PL` at the previous baseline until the next trusted frame, per the frozen Java rule.

### 5. Image OCR Unsupported Path

- Request a sync with image-only input when image OCR is not available in the current runtime.
- Confirm the response is a structured unsupported/not-implemented error that names the readboard/OCR boundary.
- Confirm no board state is replaced by stale or guessed data.

Expected result: image OCR unavailability is explicit and recoverable. It is not reported as a successful sidecar sync.

### 6. Failure Modes

- Exercise missing provider configuration, bad credentials/session, network loss, provider timeout, malformed provider payload, missing sidecar, sidecar crash, sidecar timeout, and cancellation/retry.
- Confirm logs and UI messages identify the failing boundary: provider auth, provider network, sidecar process, sidecar protocol, Tauri command, or DTO normalization.

Expected result: failure states are structured, recoverable where expected, and not described as successful live support.

## R10 Ticket06 Yike Ongoing Synchronization

Source candidates: `cd8ae4fa50a584589daf1b673aaecf7ce5433a21` (implementation) and `141dd0e47ef973b4a141b46b7283d8f408cd283b` (review repairs). Both built and ran as independently prepared Windows candidates; neither daily clone was edited. Ticket06 started from ticket03 handoff `116199cc84341c57d29632771b51bbbc5d23daac`. Concurrent Tencent commits through `28685afd1b085fa2f5a640b2bd6cf5752e74e6ce` were preserved and excluded from the ticket06 review delta.

Focused checks actually run (not a full release matrix):

- `cargo test -p provider-core --test network`: 23 passing controlled network cases, including separate leases, cancellation, policy revisions and bounded retries.
- `cargo test -p sgf --test external_sync`: 13 passing reconciliation cases, including same-count revisions, ancestry-safe C/LZ overlays and cursor fallback.
- Desktop `external_sync` owner filter: 18 passing cases; `current_game_save` filter: 7 passing cases. Save As captures an immutable snapshot before opening the picker; a later frame cannot be marked saved by that write.
- DTO historical interval decoding and narrow preference-owner merge/failure/restart cases passed (one focused case each).
- `npm --prefix apps/desktop run test -- src/components/YikeSyncPanel.test.tsx`: 14 passing cases. The delayed-preference-save Cancel regression failed before repair and passed afterward.
- Existing App/ProviderPanel focused suites: 149 passing cases; `npm --prefix apps/desktop run build` and desktop `cargo check` passed. App's mocked ProviderPanel does not prove its actual lifetime key; native evidence below covers that integration.

Windows run `/mnt/d/dev/weiqi/acceptance/r10-yike06-cd8ae4f-run1/` records actual public room `79496157` updating 67→73→76 moves with session1/document2 and user cursor61. Real native Save completed. Manual proxy `127.0.0.1:1` produced visible terminal `proxy_failed`/ErrorPaused while retaining the last-good game; restoring Direct did not silently resume. Retry/Stop and local comment editing were exercised, as was dirty Play & Sync cancellation. This terminal proxy fault does not prove the transient Retrying sequence, which is covered by controlled requests. Later admissions observed finished results but are not same-session terminal-transition evidence. File Exit with an active owner completed normally (exit0, approximately0.27s from the recorded command), without forced process termination.

Repaired Windows run `/mnt/d/dev/weiqi/acceptance/r10-yike06-141dd0e-run1/` used PID73736 and CDP9337 on its private Win32 desktop. Successful one-shot import removes the old preview/action. Current-hall room `79496703` updated under session2/document5 from processing to **ended,129 moves,result `b+`**, retaining cursor19. During the real Save As picker, the source advanced21→27: `invocation-snapshot.sgf` equals the invocation serialization, differs from the later frame, and the current27-move document remains dirty. A separate preview survives these same-document updates. A fault at49 moves paused the session; restoring Direct did not resume it, and explicit Retry successfully fetched54 moves under the same session/document. Stop then allowed a local comment. Holding a real preview IPC success until after Stop/New/Discard and releasing it left the replacement0-move game unchanged, with no old import action. The latency hook was removed. `sync-smoke.json`, `terminal-source.sgf`, `delayed-save-dialog.json` and `delayed-save-native.png` retain raw evidence. An automation wait timed out and reset its JS kernel after the first ended observation; reconnecting to the original PID confirmed unchanged session2/document5 and saved the terminal evidence. Play & Sync subsequently admitted the canonical room and reported no opener error, but browser dispatch alone is not browser-play evidence. File Exit with active session3 completed exit0 at2026-10-05T19:20:52.0962886Z; no forced app termination.

Review **SUCCESS**: frozen implementation delta `28685af..cd8ae4f`, independent Spec and Standards FULL_REVIEW, then bounded VERIFICATION. SPEC-01 (duplicate STD-02), delayed preference-save cancellation, and SPEC-02, picker-time snapshot capture, have failed-before/passed-after regression evidence. STD-01, stale preview lifetime after replacement, was reproduced natively before repair; final native import, same-document browsing and delayed-response replacement evidence close the verification-only gap. All three IN_SCOPE findings are resolved; zero open findings, no DEFERRED findings and zero follow-up items. No third reviewer batch or additional source repair was needed. Acceptance and review verdicts remain separate.

Ticket06 business acceptance is **complete**. PROV-03 remains **Partial** pending ticket08 platform/installed network admission. Public-room observations under session2/document5 prove source-move→Next-update and the terminal transition; browser-only trial moves are a separate local activity. Public guest acquisition remains independent of browser credentials/cookies.

Visible browser handoff passed on the same exact `141dd0e` candidate in `/mnt/d/dev/weiqi/acceptance/r10-yike06-141dd0e-run3/`, after explicit authorization to use the interactive Windows desktop. Play & Sync admitted room `79496703` in Next (PID12080); Windows UI Automation observed the canonical address `home.yikeweiqi.com/#/unite/79496703` in the actual default Chrome (PID10696/HWND15274274) at2026-10-05T19:58:16.0392483Z. After Next Stop returned to Idle, the same Chrome window retained that address and page at19:59:14.4033795Z. `browser-handoff.json`, `browser-auth-state.png` and `browser-stopped.png` record both PASS cases. The website was not logged in and displayed an empty board; this run proves URL handoff and browser retention, not website authentication or game loading. Continuous public-source evidence is independently recorded in run1 above. File Exit completed normally (exit0 at20:00:21.6424823Z); the user's browser was left open. Run2's isolated-desktop visibility limitation remains a historical blocked observation, resolved by run3. Documentation-only commits after `141dd0e` do not change the verified runtime.

Ticket07 integration: reuse `CurrentGameState`'s external-sync admission/session owner and `CurrentSgfDocument::reconcile_external_source`; provide source/target identity plus the complete normalized SGF tree. First installation goes through SGF-07 once; later source commits preserve document identity/history/savepoint and carry session/source/document/request/policy fences. Stop seals identity before cancelling work. Readboard must not add a competing current-game writer or use move count as revision identity. The shared owner rejects authoring/Match/Trial while preserving browsing and exact-position attachment checks; APP-03 owns bounded shutdown, APP-04 restores only the document.

## Provider And Sidecar Manual Matrix

| Scenario | Repository/Local Expected Result | Live Environment Expected Result |
| --- | --- | --- |
| Yike fetch success | `provider_yike_list` and `provider_yike_preview` validate public input before I/O and return normalized typed results under the shared network identity. | Real Recommend/Local and each accepted URL family independently returns readable SGF with public locator, route, result count and latency recorded. |
| Yike fetch failure | Missing URL, wrong provider, timeout, malformed payload, or unavailable runtime returns `ProviderError` without stale success data. | Bad auth/session, blocked network, provider timeout, or malformed live response returns structured auth/network/payload errors. |
| Fox nickname lookup | `provider_fox_list` validates input before I/O, resolves the account, and returns a normalized first batch; unknown nickname is `not_found` with no list request. | Real nickname resolves to UID/nickname and a list; unknown nickname shows not-found; route and latency recorded. |
| Fox UID list and continuation | `provider_fox_list` / `provider_fox_list_more` return ≤100 games per batch, continue via `lastCode`, drop cursor/duplicate rows and end on short/empty batches. | Real UID list continues across batches without repeats; end-of-list and empty results are observed. |
| Fox chessid preview/import | `provider_fox_preview` normalizes SGF via CGI/H5 alternatives under one retry budget; misses fall through, H5 miss is `not_found`. Import uses SGF-07 with the preview identity. | Real chessid previews without changing the current game; explicit Import installs it; Cancel/dirty Cancel keeps the original game. |
| Fox fetch failure | Invalid lookup, malformed list/record, classified HTTP failures, cancellation and stale identities return typed errors without overwriting newer results. | Blocked network, timeout, upstream failure or payload change returns visible structured errors with retry. |
| readboard unavailable/fault | Controlled children prove typed incompatible/timeout/exit/disconnect and actual cleanup; missing path/unsupported platform is unavailable. | Only real-runtime observations count as live readiness; controlled fault runs remain separately attributed. |
| readboard ready/stop/restart | Generation/revision fence prevents stale publication and owned Child/socket cleanup precedes another admission. | Fixed Windows evidence in §3 proves ready220430, Stop, Restart, dirty exit Cancel/Discard cleanup and saved-path reopen without changing the current game. |
| readboard live sync | Decoder rejects malformed frames; controlled children deliver multi-frame/disconnect/restart streams; owner tests prove SGF-07 Start, stale-generation ignore, Error-paused/Retry/Stop and the Java-oracle recovery. | §4 recorded evidence (Next `f75cacf…`, readboard `cdcc7b3…`, Fox spectator rooms) proves continuous updates, target change, disconnect, Retry on a new generation, Stop→edit, Save/reopen and preference persistence. |
| image OCR unavailable | Image-only sync returns a structured unsupported/not-implemented error when OCR is unavailable. | Live OCR may only be marked PASS with an OCR-capable runtime and image fixture evidence; otherwise mark SKIPPED/UNSUPPORTED. |

## Session Workspace Verification

From `apps/desktop`, run `npm run test -- src/workspace src/components/BoardCanvas.test.tsx src/components/WinrateChart.test.tsx src/components/AnalysisPanel.test.tsx src/App.test.tsx src/SubBoardContentMode.test.tsx src/VariationReplay.test.tsx src/AnalysisPresentation.test.tsx`, then `npm run build`.

Browser smoke: at 1100×720 and a larger viewport, drag both separators to their limits, release outside the handle, cancel capture, and use focused Left/Right (8px) and Shift+Left/Right (32px). Check all three canvas backing sizes against CSS content size × DPR after container-only resize, DPR changes and hide/show without new analysis. Check 19×19, 9×9 and 13×9 input geometry and scroll access to expanded controls. Preview remains non-authoritative; exact committed Windows candidate, PID, screenshots and native pointer/window behavior are a separate acceptance gate. Durable proportions, visibility, native geometry/reset and pinning are integrated; remaining item-level native evidence is tracked separately in R7.

For exit integration, run `npm test -- src/App.preferences.test.tsx src/App.test.tsx src/hooks/useMainWindowPin.test.tsx` and `npm run build`. Hold each theme, rail and pin transaction at the API seam and confirm document departure waits; confirm a dirty exit decision and a committed final-drain error reject new theme/rail/pin changes, while Cancel restores them. Repeat the affected native Close/File Exit, genuine preference replacement denial, Retry/Cancel and rapid-write/restart cases on the exact candidate. Fixtures prove the ordering boundary, not native disk, z-order or exit acceptance.

Windows pin acceptance must compare the command's `actual` result with Win32 `WS_EX_TOPMOST`. In an isolated, identity-checked candidate, deliberately clear that native flag without changing durable intent, then Retry the saved intent and verify the flag is restored. This controlled native-state drift is not a naturally occurring setter failure. Run the Windows-native `main_window_pin` tests as well: zero extended styles with a stale last-error value mean a valid unpinned window, while a destroyed HWND must report unreadable. A native Z-order witness must be non-topmost at measurement time; preparatory focus operations are not proof.

Final R7 source candidate `baa9747d11ee9e2130904ff057adbe7cfd968153` was built and exercised in isolated Windows runs `D:\dev\weiqi\acceptance\r7-integrated-baa9747-run1` through `-run7` and process-local monitor-fault runs. Evidence includes trusted native splitter cancellation, 9×9/13×9 input, no-frame canvas resize, both whole-preference reset comparisons, hidden-rail restart, real KataGo continuous/task identities, no-engine Save/reopen, genuine replacement denial, and both pre-departure and post-Discard final exit Retry. Final run7/PID13212/HWND264354 completes actual system DPI96→144→96, both themes, visible keyboard focus, native K10 hit and preserved21-move SGF/Classic. Two actual Explorer drops verify dirty replacement admission, exact SGF retention on Cancel, exact 9×9/two-variation import on Discard, and the same single main window. Actual DISPLAY2 negative-coordinate movement and restart retain `-2210,64 / 1145.3333333333333×722.6666666666666 @1.5`; Windows “PC screen only” removes the secondary output and recovery restores `552,226 / 1440×900 @1`, changing only the geometry owner. Both topology runs exit normally. `A10-continuation-native-final-verdict.json` and `A18-continuation-native-final-verdict.json` link measured records and inspected screenshots. Supplemental125%/200% initialized CDP layouts are separately scoped, not native system-DPI proof. The normal non-topmost witness establishes relative Z-order; witness foreground was not obtained. See [R7 integration record](R7_PLAN.md#integrated-verification-record) for source/check attribution and ticket07 for every case result.

Authorized automatic A11 acceptance used `ChangeDisplaySettingsExW` flags0 after `CDS_TEST`, with an independent timeout-restore process armed before switching. Actual mode was `1280×720 @180Hz`, work area `1280×672`, DPI96; both native runs fitted outer `0,0,1280×672` / client `1264×633`. Trusted scrolling reached page/workspace/left-rail controls, SGF and both visible rails stayed unchanged, and restart retained the complete preference document. Both runs exited0. Explicit restoration read back `2560×1440 @180Hz` and unchanged registry settings. `D:\dev\weiqi\acceptance\r7-integrated-baa9747-small-auto\A11-continuation-native-final-verdict.json` links16 measured records/screenshots. The timeout fallback was armed, not exercised; no browser emulation or persistent display update was used.

All18 Windows A01–A18 cases pass on exact native source `baa9747d11ee9e2130904ff057adbe7cfd968153`. This R7 run's macOS AppKit LTR/RTL native acceptance is SKIPPED by explicit user approval on 2026-09-30. Authorized follow-up `f4e478ea15781b7a84ee1ec27a0f74683e909e2e` passes formatting, affected-owner Linux clippy, strict Windows lib/tests clippy, SGF94, Recent4 and five actual native Open/Save As/reopen/exit cases. Its two complete historical handoff envelopes are retained as resolved in ticket07. Subsequent controlled-fixture repairs pass desktop-lib146/146 in default-parallel and serial runs, shared analysis-task fixture checks20/20 (107 filtered), formatting and strict Linux lib/tests clippy. Final bounded Standards/Spec reviews are CLEAN on exact object `406250408043e68fe67f6082c583429600768c0c`; all seven owners Accepted, ticket07 CLOSED/ACCEPTED and R7 exited. Earlier native/test counts keep their original source attribution; no Mac PASS, support change or additional release claim is made. Ticket08 is unblocked and remains unexecuted.


## Native Window Geometry Verification

Run `cargo test -p app-preferences` for geometry, monitor-failure and window-state fixtures, and `cargo test -p lizzieyzy-next-desktop continuous_analysis::tests::workspace_updates_merge_with_other_owners_and_fail_atomically` for narrow-save isolation and failure/retry. From `apps/desktop`, run `npm test -- src/App.test.tsx` and `npm run build` for lifecycle integration.

On an exact committed Windows candidate, record outer physical origin, client size, DPI, process identity and preference JSON while moving/resizing, restarting maximized, unmaximizing, and restarting after minimization. Validate a reachable off-center record unchanged, an invalid record fitted to the work area, reset isolation, blocked preference writes and Retry, immediate close after a move, dirty-close cancellation, file activation/drop and recovery. Real monitor removal/DPI changes require native hardware/session evidence; deterministic fixtures and browser rendering are supplementary only.

For caption boundaries, measure the actual native draggable region with `WM_NCHITTEST` (`HTCAPTION`), not just `TITLEBARINFOEX`: its rectangle can include resize borders and system-menu pixels. Verify a contiguous 100 logical pixel width and `min(32, native draggable caption height)` logical pixel height; exercise valid edge placement unchanged on restart and insufficient visible title area recovered into the work area.

On 2026-09-29, Windows snapshot `85a5cc12f876186c95b9961451364a46cf0a0ad7` at 96 DPI exposed 28 logical pixels in `TITLEBARINFOEX`, but only 22 contiguous `HTCAPTION` rows. A 100-pixel-wide scan verified all 22 rows. A saved `400,1361,1100×720` client record retained its position on restart with the caption ending exactly at work-area bottom 1392; `400,1375,1100×720` recovered to `552,226,1440×900`. Evidence is in `D:\dev\weiqi\acceptance\r7-window04-85a5cc1-run3` and `-run4`. This proves the Windows title-height boundary, not ticket-wide acceptance. Real display-removal and OS file-drop gates remain separate.

The same exact snapshot passed process-local native monitor-query fault probes at 96 DPI on a 2560×1392 work area. Reversible Frida 17.19.0 hooks, armed after main-window creation, made `EnumDisplayMonitors` fail; a second case also made `MonitorFromPoint` return null. With primary readable, the window fitted to `552,226,1440×900`; with both queries unavailable, the visible OS-default window remained at `260,260,1440×900`. Both cases showed an unsaved error and toolbar Retry while preserving the entire preference document, including durable `310,170,1100×720`. Close during the double fault showed Retry/Cancel, and Cancel retained the live window. Removing the hooks and clicking toolbar Retry restored the durable bounds and saved status in both cases; an uninstrumented restart after the enumeration-only case retained the same bounds. All three processes exited normally, and the isolated profile was restored. Evidence: `D:\dev\weiqi\acceptance\r7-window04-monitor-enum1`, `r7-window04-monitor-enum-restart`, and `r7-window04-monitor-both1` (`run.json`, native rectangles, status JSON, fault logs and screenshots). These are injected failures of the real native API path, not physical monitor removal or proof of double-failure reachability on every screen topology.

The non-Windows caption delta was exercised with the real Linux GTK/WebKit application, Xvfb and Openbox on an isolated 1600×1000 X11 display at scale 1. A compact `400,963,800×200` record retained a 32-pixel-high visible caption; native pointer drags at both corners of a 100×32 caption rectangle moved the window horizontally without changing its vertical position or client size. `400,964,800×200`, leaving only 31 pixels, recovered to `75,22,1440×900`. Native maximize retained the normal rectangle, minimize retained the maximized flag, restore recovered normal bounds, and GTK titlebar buttons and dragging both worked after the event surface was placed below its controls. A fresh isolated profile persisted the actual native `75,22` outer origin with a `1440×900` client. Evidence is under `/tmp/window04-linux-native`, including PID-bound `integrated-valid-caption.json`, `integrated-invalid-caption.json`, native-control records and screenshots. Openbox can constrain larger partially off-screen windows even when the policy accepts them; use compact fixtures to separate title visibility from compositor placement policy. This is native Linux X11 evidence, not physical multi-monitor or Wayland acceptance.

The AppKit adapter type-checks against `objc2-app-kit` for `aarch64-apple-darwin` in an isolated API harness. No macOS GUI session or Apple SDK is available here, so this is neither a full macOS application build nor native caption acceptance. Review identified an unverified RTL-control exclusion candidate in that adapter. On 2026-09-30 the user explicitly skipped this R7 run's macOS LTR/RTL native acceptance: SKIPPED, not PASS. The candidate and original review attribution remain unverified; macOS implementation/support and other phases' platform gates are unchanged.

Native Wayland uses the approved compositor-managed-position exception: persisted `x` and `y` are both null, while client size and maximization remain durable. This replaces the observed `Cannot query primary monitor; saved geometry was retained.` startup/Retry failure; GDK permits no primary monitor and GTK does not expose global Wayland coordinates ([GDK](https://docs.gtk.org/gdk3/method.Display.get_primary_monitor.html), [GTK](https://docs.gtk.org/gtk3/method.Window.get_position.html)). A real GTK/WebKit run on isolated Weston 14.0.2 restored a null-origin `1100×720` record, maximized through its native button without changing normal dimensions, closed normally, restarted maximized, and restored normal size. Display-menu reset saved `1440×876` after compositor constraints with both origins still null; all other preference fields were identical. Native Close and File Exit both exited 0 without a geometry-error prompt. Tao's Wayland event surface is placed below native controls so they receive clicks. Evidence screenshots are under `/tmp/window04-wayland-native/managed-*.png`. An earlier Alt+F4 closed the nested compositor instead of the application; that disconnected run is excluded from exit evidence. These nested-compositor results do not substitute for macOS or physical display-removal validation.

Wayland review found that mixed-null saved origins bypassed startup validation. The native before-repair fixture `{x:null,y:50,width:1000,height:650,scaleFactor:1,maximized:true}` incorrectly restored maximized and was rewritten as a valid null/null record. Shared sampling/restore validation now rejects that record: the identical native fixture restored normal `1440×900`, null/null origins, with all other preference fields unchanged. Policy regressions cover both mixed orientations and non-finite/sentinel numeric origins; 19 geometry tests passed. Before/after JSON and screenshots are in `/tmp/window04-wayland-native/mixed-origin-*`.

User-assisted Windows acceptance on exact snapshot `85a5cc12f876186c95b9961451364a46cf0a0ad7` imported `r7-window04-drop.sgf` from Explorer: the UI showed the matching filename, players `window04-drop` / `native`, 9×9 board, and one black move E5. Real DISPLAY2 at `-2560,-156` used 150% DPI; moving to `-2100,80` saved logical client `1101.3333333333333×722@1.5`. A normal exit/restart preserved that exact geometry and native physical client `1652×1083`. The user then selected Windows “PC screen only”; real enumeration removed DISPLAY2, and restart recovered to primary `552,226,1440×900@1`, saved and usable. This is a real OS topology removal, not a physical cable-unplug test or an API mock. All three processes exited 0. Evidence: `D:\dev\weiqi\acceptance\r7-window04-manual-drop1`, `r7-window04-negative-restart`, and `r7-window04-display-removed`. This snapshot predates the Linux/Wayland adapter delta; its Windows evidence does not establish macOS acceptance.

Linux and Wayland screenshots and geometry JSON have durable copies under `D:\dev\weiqi\acceptance\r7-window04-platform-evidence\linux` and `\wayland`. The Wayland repair passed independent Standards and Spec verification against the frozen repair delta; `SPEC-WAYLAND-1` and its duplicate `STD-WAYLAND-1` are resolved. This scoped result supplies no AppKit evidence; this R7 run's AppKit native acceptance is separately SKIPPED by user approval on 2026-09-30, with its unverified candidate and original review attribution retained.

The platform-adapter commit `f9b1dd82d66eb77c45b36d47206de66bada5862d` also built successfully as an isolated Windows candidate. Its `r7-window04-f9b1dd8-run2` restored numeric `310,170,1100×720@1`, showed saved status with no geometry error, rendered the native application, and exited 0. Native rectangles, status JSON and screenshot are retained in that run directory. Run1 used an invalid workspace-share fixture and exercised preference quarantine/default startup; it is excluded from numeric-restore evidence.

## Documentation Acceptance

When updating docs for this handoff package, keep these claims accurate:

- Do not claim a Tauri release has been published unless release artifacts exist.
- Do not claim full Java/Swing parity.
- Do not claim Fox/Yike/readboard live migration is complete without external-network or sidecar evidence.
- Distinguish browser preview behavior from native Tauri desktop behavior.
- Distinguish per-change focused validation from the full release matrix; do not make obsolete unconditional claims requiring full-workspace or full-smoke runs on every edit.
- Report the exact validation command, affected surface, evidence provenance, and result in the handoff.
## Suggested Reading

- [Next architecture](ARCHITECTURE_NEXT.md)
- [Migration plan](MIGRATION_PLAN.md)
- [Release checklist](RELEASE_CHECKLIST.md)
- [Troubleshooting](TROUBLESHOOTING.md)
