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
- Provider runtime path checks: verify `provider_fetch_yike` and `provider_fetch_fox` return structured success/error results through the Rust/TypeScript boundary. Record offline contract status vs live external network verification separately.
- Readboard domain tests: run Rust tests owning readboard parsing and command handling via focused filters.
- Readboard sidecar path checks: verify `readboard_sidecar_probe` and `readboard_sidecar_sync_snapshot` return structured status results. Live sidecar checks require an active sidecar process and must be documented separately.
- UI path checks: ProviderPanel is the expected UI surface for provider fetch, readboard probe, and readboard sync controls. If controls are absent or preview-only, mark live UI support as pending.
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

### 1. Yike Runtime Fetch

- Start the Tauri desktop runtime and configure the Yike provider inputs required by the implementation.
- Trigger `provider_fetch_yike` from ProviderPanel or the intended debug/test harness for a real Yike game, game list, or supported provider resource.
- Record account/session type, network path, request target, result count, and latency.
- Repeat with an expired or missing session if the implementation supports that state.

Expected result: successful fetches return normalized DTOs, and auth/network failures return structured errors without stale cached data being presented as live data.

### 2. Fox Runtime Fetch

- Start the Tauri desktop runtime with the real Fox prerequisites in place.
- Trigger `provider_fetch_fox` from ProviderPanel or the intended debug/test harness for each supported command shape: `chessid`, `uid`, and `user_name`.
- Record target client/session state, result count, latency, and any provider-specific prerequisites.
- Repeat with the target client unavailable or credentials/session invalid.

Expected result: successful fetches return normalized DTOs, and unavailable client/session/network states produce actionable errors.

### 3. readboard Sidecar Probe

- Start the readboard sidecar expected by the current implementation.
- Probe the sidecar from the Tauri runtime.
- Record sidecar version, port/path, process state, and probe latency.
- Stop the sidecar and repeat the probe.

Expected result: the app distinguishes ready, not running, incompatible, and timeout states.

### 4. readboard Sync

- With the sidecar running, sync against a real target board/client state.
- Exercise protocol line sync through `readboard_sidecar_sync_snapshot` using the supported protocol-line input or sidecar response path.
- Confirm board size, stones, move state, coordinates, and player-to-play if available.
- Restart the sidecar or change the target board state and sync again.

Expected result: sync responses normalize into app DTOs and do not leave stale board state after restart, target change, or timeout.

### 5. Image OCR Unsupported Path

- Request a sync with image-only input when image OCR is not available in the current runtime.
- Confirm the response is a structured unsupported/not-implemented error that names the readboard/OCR boundary.
- Confirm no board state is replaced by stale or guessed data.

Expected result: image OCR unavailability is explicit and recoverable. It is not reported as a successful sidecar sync.

### 6. Failure Modes

- Exercise missing provider configuration, bad credentials/session, network loss, provider timeout, malformed provider payload, missing sidecar, sidecar crash, sidecar timeout, and cancellation/retry.
- Confirm logs and UI messages identify the failing boundary: provider auth, provider network, sidecar process, sidecar protocol, Tauri command, or DTO normalization.

Expected result: failure states are structured, recoverable where expected, and not described as successful live support.

## Provider And Sidecar Manual Matrix

| Scenario | Repository/Local Expected Result | Live Environment Expected Result |
| --- | --- | --- |
| Yike fetch success | `provider_fetch_yike` validates provider/timeout and reaches the Yike runtime path with typed success, typed error, or runtime unavailable results. | Real Yike resource fetch returns normalized DTOs with source metadata, result count, and latency recorded. |
| Yike fetch failure | Missing URL, wrong provider, timeout, malformed payload, or unavailable runtime returns `ProviderError` without stale success data. | Bad auth/session, blocked network, provider timeout, or malformed live response returns structured auth/network/payload errors. |
| Fox `chessid` fetch success | `provider_fetch_fox` accepts the `chessid` command shape and reaches the Fox runtime path with typed success, typed error, or runtime unavailable results. | Real `chessid` fetch returns normalized SGF/provider DTOs and records prerequisites and latency. |
| Fox `uid` fetch success | `provider_fetch_fox` accepts the `uid` command shape and reaches the Fox runtime path with typed success, typed error, or runtime unavailable results. | Real `uid` fetch resolves the expected game/list payload and normalizes it without UI-only shortcuts. |
| Fox `user_name` fetch success | `provider_fetch_fox` accepts the `user_name` command shape and reaches the Fox runtime path with typed success, typed error, or runtime unavailable results. | Real nickname lookup resolves to the expected account/game payload and records ambiguous/not-found behavior. |
| Fox fetch failure | Missing URL/command, wrong provider, timeout, bad command, malformed payload, or unavailable runtime returns `ProviderError`. | Unavailable client/session, bad account, blocked network, timeout, or provider payload change returns structured errors. |
| readboard probe missing | `readboard_sidecar_probe` validates timeout and reports not-ready/runtime unavailable as a structured result or error. | Stopped or unreachable sidecar reports not running/unreachable/timeout without changing board state. |
| readboard probe present | Probe path is callable through the intended boundary. | Running sidecar reports ready with version/path/latency recorded. |
| readboard protocol line sync | `readboard_sidecar_sync_snapshot` validates input and protocol-line parsing returns DTOs or typed protocol errors. | Sidecar sync reflects board size, stones, move state, and player-to-play from the real target board. |
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
