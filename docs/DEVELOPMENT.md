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
- Pick or type the KataGo engine binary path.
- Pick or type the model path.
- Pick or type the analysis config path.
- Optionally set a working directory.
- Set a positive max visits value.
- Save the profile.
- Add a second profile if Autoload or Switch coverage is part of the change.

Expected result: profiles reload after app restart. Saving Autoload Default or Settings selection does not change the current Foreground Engine Run.

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
