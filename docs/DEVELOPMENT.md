# Development Guide

This guide is for contributors working on the LizzieYzy Next Tauri 2 + Rust + TypeScript workspace. The existing Java/Swing application remains the maintenance release line; this document focuses on validating the Next scaffold and its current desktop workflows.

## Scope

Use this guide when changing:

- `README.md` and Next architecture/migration docs,
- scaffold validation and smoke documentation,
- the Tauri desktop app under `apps/desktop`,
- Rust crates under `crates/*`,
- SGF, KataGo, Foreground Engine Run, engine profile, Autoload Default, durable preferences, and cache behavior.

Do not treat a passing Next smoke run as full legacy parity. Provider/readboard work in this batch may provide offline contracts and runtime path plumbing, but live Fox/Yike network behavior, live readboard sidecar operation, and Tauri production release packaging still require environment-specific validation.

## Required Local Baseline

From the repository root, always run:

```bash
python3 scripts/validate_scaffold.py --verbose
python3 scripts/validate_release_assets.py --verbose
```

For code changes, also run the relevant checks:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

For frontend or Tauri UI changes:

```bash
cd apps/desktop
npm ci
npm run build
```

For documentation-only changes, scaffold validation is still required because it is the shared acceptance gate for the current handoff package.

Provider/readboard acceptance entry points:

- Provider contract tests: run the Rust workspace tests that own the provider contract modules and record the exact package or filter. `cargo test --workspace` is the baseline; a focused filter only counts if it runs non-zero provider tests.
- Provider runtime path checks: verify `provider_fetch_yike` and `provider_fetch_fox` return structured success/error results through the intended Rust/TypeScript boundary. This is still offline/local evidence unless real provider services are contacted.
- Readboard domain tests: run the Rust workspace tests that own readboard parsing/command behavior and record the exact package or filter.
- Readboard sidecar path checks: verify `readboard_sidecar_probe` and `readboard_sidecar_sync_snapshot` return structured ready/not-ready/sync results through the intended boundary. Live sidecar checks require a real readboard environment and should be listed separately.
- UI path checks: ProviderPanel is the expected UI surface for provider fetch, readboard probe, and readboard sync controls. If a batch only exposes payload import or URL preview, record fetch/probe/sync UI as pending rather than implying live support.
- Release dry-run: run `python3 scripts/validate_release_assets.py --verbose` locally and use `.github/workflows/release-dry-run.yml` for the cross-platform compile-only dry-run.

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

The browser preview is useful for layout and fallback checks. Real KataGo execution, native file dialogs, app-data engine profile persistence, asset inspection, and SQLite cache commands require `npm run tauri:dev`.

## Repository Structure

- `apps/desktop`: React + TypeScript frontend.
- `apps/desktop/src/api`: frontend wrappers around Tauri commands and browser fallbacks.
- `apps/desktop/src/components`: board, analysis, chart, cache, Engine Switcher, and Engine Settings UI.
- `apps/desktop/src-tauri`: Tauri 2 command gateway and native app integration.
- `crates/app-model`: shared DTOs.
- `crates/go-core`: board/rules logic.
- `crates/sgf`: SGF parsing, replay, and serialization.
- `crates/katago-protocol`: KataGo analysis JSONL query/response modeling.
- `crates/analysis-core`: derived analysis markers.
- `crates/engine-manager`: engine profiles, Autoload Default, Foreground Engine Run lifecycle, Analysis Jobs, process execution, and cancellation.
- `crates/app-preferences`: durable app preference load/save, unreadable isolation, and replace-safe persist.
- `crates/storage`: SQLite storage/cache helpers.
- `tests/golden`: SGF fixtures for migration and regression checks.

## Local Smoke Flow

Use the desktop runtime for this flow:

```bash
cd apps/desktop
npm run tauri:dev
```

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
- Click `分析此手`.
- Confirm the job stays on that Ready run, the current-game generation, and the selected `NodePath`.
- Confirm candidates, PV, winrate/score, ownership overlay (`领地`), and policy overlay (`策略`) update for that node only.
- Click `取消此手` while the job is still running. Confirm the Engine Switcher stays Ready and whole-game (if running) continues.
- Start a second `分析此手` and confirm only the latest matching identity publishes.
- Confirm a timeout or protocol failure does not leave candidates or overlays, and does not write SQLite analysis cache.

Expected result: selected-node analysis runs on the current Ready Foreground Engine Run, not a one-shot profile process. Publication is identity-bound to run/job/generation/`NodePath`.

Repository evidence (does not substitute for native GUI or live KataGo):

```bash
cargo test -p sgf selected_node --offline
cargo test -p lizzieyzy-next-desktop selected_node --offline
cargo test -p engine-manager --test foreground_engine_run selected_node_ --offline -- --test-threads=1
cd apps/desktop && npx vitest run src/EngineLifecycle.test.tsx src/SelectedNodeAnalysis.test.tsx
```

Optional real KataGo on a resident Run (ignored by default):

```bash
LIZZIEYZY_REAL_KATAGO=1 cargo test -p engine-manager --test real_katago_lifecycle_smoke -- --ignored --test-threads=1
```

### 5. Run Full-Game Analysis

- With a Ready run, click `自动分析`.
- Watch the whole-game lane show `整局 completed/expected 剩余 remaining` as first-child mainline nodes finish. Each completed node should publish its own `NodePath` result immediately; do not wait for the batch to end, and do not treat turn as job identity.
- Navigate with `上一手` / `下一手` while the lane is running and confirm review navigation stays enabled and does not cancel the job.
- Optionally click `分析此手` / `继续分析` on the same Ready run and confirm both lanes stay active together.
- A second `自动分析` while that lane is occupied must be rejected without cancelling the selected-node lane.
- Cancel or fail after at least one node has completed and confirm the current-session result for that node remains visible.

Expected result: whole-game analysis occupies only the Ready Run's whole-game lane, walks `[]` / `[0]` / `[0,0]`…, and keeps completed node results after cancel/fail. Selected-node can run concurrently. Durable SGF attachment is not part of this smoke.

### 5.1 Bind Analysis Presentation To Exact Nodes

- After selected-node completes, navigate away with `下一变化` and back with `父节点`. Confirm that node's candidates, PV Sub-Board, winrate, and score return, and that candidate hover does not return.
- Start `继续分析` while a whole-game result already exists for a child node, then open that child. Confirm the child's presentation appears, `取消整局` stays available, and navigation does not cancel whole-game.
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
- Confirm new encoded primary uses root `LZOP` and non-root `LZ`. This smoke does not attach live engine results (ticket 08) and does not remove the SQLite cache (ticket 09).

Expected result: opening a branching Java SGF shows selected-node analysis through the exact-node presentation path. Save/reopen preserves secondary, unknown properties, personal comments, and malformed payloads. Native KataGo/GTK is not required for this repository evidence.

Repository evidence (does not substitute for native GUI or live KataGo):

```bash
cargo test -p sgf --test java_analysis_payloads --offline
cd apps/desktop && npx vitest run src/AnalysisPresentation.test.tsx
```

### 6. Cancel Analysis

- On one resident Ready run, start both `分析此手` and `自动分析`.
- Click `取消此手` and confirm the whole-game lane continues.
- Click `取消整局` and confirm selected-node (if restarted) is unaffected.
- Confirm the Engine Switcher is still Ready (`停止` / `重启` enabled).
- Confirm ordinary `保存` / `另存为` stay enabled while analysis is running.
- Start another analysis afterwards to ensure the job registry recovered.

Expected result: each cancel is lane-local, does not Stop the run, does not lock Save/Save As, and does not prevent a later start on that lane.

### 7. Verify Cache Hit

- Analyze a game.
- Reparse or reopen the same SGF.
- Confirm cache status changes to hit for the same game/profile/engine kind.
- If needed, clear the cache path through the UI or a targeted cache command and verify miss behavior.

Expected result: repeated loading of the same SGF can reuse cached analysis instead of starting from an empty review state.

### 8. Save SGF

- Use Save or Save As under the Tauri desktop runtime.
- Reopen the saved file.
- Confirm parse/replay still succeeds and move count remains stable.

Expected result: SGF write validates parseability and can round-trip through native open.

### 9. Durable Preferences

- Open Preferences from `参数`, `棋盘`, Settings → `首选项…`, or Settings → `综合设置(Shift+X)`.
- Confirm the sheet is categorized (`分析呈现`, `复盘`, `棋盘`, `缓存`) and that View-menu `候选` / `领地` edit the same values as the sheet.
- Change a visible preference (for example uncheck `候选`) and wait until the sheet reports that it is saved.
- Quit the Tauri app and start `npm run tauri:dev` again.
- Confirm the saved value is still applied after restart.

Repository equivalent: `cargo test -p app-preferences successful_write_is_reloadable_as_restart`.

Expected result: missing preference storage loads owner defaults; a successful write survives native restart; View-menu and Preferences-sheet edits share one durable store.

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

## Documentation Acceptance

When updating docs for this handoff package, keep these claims accurate:

- Do not claim a Tauri release has been published unless release artifacts exist.
- Do not claim full Java/Swing parity.
- Do not claim Fox/Yike/readboard live migration is complete without external-network or sidecar evidence.
- Distinguish browser preview behavior from native Tauri desktop behavior.
- Report the exact validation command and result in the handoff.

## Suggested Reading

- [Next architecture](ARCHITECTURE_NEXT.md)
- [Migration plan](MIGRATION_PLAN.md)
- [Release checklist](RELEASE_CHECKLIST.md)
- [Troubleshooting](TROUBLESHOOTING.md)
