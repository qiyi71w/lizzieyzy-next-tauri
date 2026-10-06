# Release Checklist

This checklist tracks release readiness for the LizzieYzy Next Tauri 2 + Rust + TypeScript workspace. It is not a statement that a public Tauri release has already shipped.

This checklist governs full release candidate qualification, distribution packaging, and milestone readiness. It is **not** a per-ticket manual gate for day-to-day development or localized changes. Per-change development validation follows the affected-surface development gate in [Development Guide](DEVELOPMENT.md). Refer to that policy for focused validation, parent-executor ownership, acceptance mini-contracts, and runtime isolation rules rather than duplicating them here.

The existing Java/Swing maintenance line may have its own release process. For the Next workspace, do not publish or describe a release as ready until the checks below pass on the intended platform and the artifact set exists.
## Release Readiness Rules

- Do not claim full legacy parity unless Fox/Yike/readboard, legacy settings, and advanced review workflows have explicit acceptance evidence.
- Do not claim Fox, Yike, or readboard live support in the Next app from offline contracts alone. Repository-level offline contract and runtime path evidence can be reported as implemented, but live support requires the environment smoke checks below.
- Do not claim production packaging is complete until platform artifacts are built and verified.
- Keep README, migration plan, architecture doc, and release notes aligned with the actual state.
- Every release candidate must include scaffold validation output.

## Required Automated Checks

From the repository root:

```bash
python3 scripts/validate_scaffold.py --verbose
python3 scripts/validate_release_assets.py --verbose
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Frontend automated checks and build:

```bash
cd apps/desktop
npm ci
npm test
npm run build
```

Tauri build, once release packaging is in scope:

```bash
cd apps/desktop
npm run tauri:build
```

If `npm run tauri:build` is not being run for the current handoff, document that packaging was not validated.

Release engineering dry-run:

```bash
cd apps/desktop
npm run tauri:build -- --no-bundle --ci --no-sign
```

The GitHub Actions dry-run workflow is `.github/workflows/release-dry-run.yml`. It can be triggered manually or by pushing a `v*` tag, validates macOS/Linux/Windows setup, runs release preflight, and uploads dry-run summaries without creating a GitHub release.

Provider/readboard acceptance:

- Provider contract tests must run through the Rust workspace test gate, with the exact package or filter recorded in the handoff. A zero-test focused filter is not evidence.
- Provider runtime path checks must record whether `provider_yike_list`, `provider_yike_preview`, `provider_fox_*` and `provider_tencent_*` were exercised offline only or against real services.
- Fox runtime path checks must cover nickname and UID lists (with continuation) and chessid preview/import.
- Readboard domain tests must run through the Rust workspace test gate, with `readboard_runtime_*` lifecycle and shared external-sync checks recorded separately from the offline `readboard_sidecar_sync_snapshot` preview.
- ProviderPanel hosts the three provider centers and ReadboardPanel lifecycle; the sync sheet hosts YikeSyncPanel and ReadboardSyncPanel. Native scenarios must exercise these product entry points.
- Offline provider/readboard contracts and runtime path checks are not the same as live Fox/Yike external-network or readboard sidecar validation.

## Native Desktop Scenarios

These native desktop scenarios follow an automation-first approach (using headless/isolated runners, controlled engine harnesses, or automated integration probes where available). Only unavoidable physical hardware checks (such as physical display removal or physical device state changes) and OS authorization/permission prompts remain manual checks.

Per-change development executes only the specific scenarios covering the affected surface as defined in [DEVELOPMENT.md](DEVELOPMENT.md); execute the full matrix below only for release candidates.

Run under the native desktop runtime:

```bash
cd apps/desktop
npm run tauri:dev
```

Record the OS, CPU architecture, KataGo version, model path, config path, and whether the engine is bundled or local.
### SGF Open

- Open an SGF through native file open.
- Also paste or load a fixture from `tests/golden`.
- Confirm board size, move count, players, komi/result where present, and replay positions.

Pass: the game loads without falling back to stale demo data.

### Engine Profile

- Create or update a profile with engine, model, config, optional working directory, and visits.
- Save it.
- Restart the app.
- Confirm the profile list and selected profile persist.
- Add a second profile and switch selection if multi-profile behavior is part of the candidate.

Pass: profile data survives restart in app data.

### Asset Check

- Run `Check assets`.
- Confirm engine, model, and config are present.
- Test one missing required path and confirm analysis is blocked with an actionable message.

Pass: asset checks distinguish present and missing required inputs.

### One-Position Analysis

- Select a mid-game move.
- Run one-position KataGo analysis.
- Confirm candidates, PV, winrate, score, ownership/policy-backed data paths, and markers update.

Pass: the app receives normalized analysis frames from KataGo JSONL and updates the review UI.

### Full-Game Analysis

- Run full-game analysis.
- Observe progress events.
- Confirm completion produces frames across the game and the winrate/candidate views update.

Pass: the batch job completes and validates response turns.

### Cancellation

- Start a full-game run with enough visits to observe progress.
- Cancel it.
- Confirm cancellation message appears and a subsequent analysis can start.

Pass: cancellation releases the active job and does not leave the UI permanently disabled.

### Cache Hit

- Complete an analysis run.
- Reopen or reparse the same SGF with the same profile/engine kind.
- Confirm cache status reports a hit and cached frames/problems load.

Pass: the SQLite analysis cache can be reused for the same game key.

### SGF Save

- Save or Save As the current SGF.
- Reopen the saved file.
- Confirm parse/replay and move count match expectations.

Pass: saved SGF is parseable and round-trips through native open.

## Provider And Sidecar Manual Smoke

These checks are required before release notes claim live Fox, Yike, or readboard support. Mark them `SKIPPED` when the environment is unavailable and keep the release claim limited to offline contract/runtime path status.

### Yike Runtime Fetch

- Record Yike account/session type, network environment, provider endpoint or resource type, and app build.
- Fetch Recommend/Local or a real supported Yike locator through `provider_yike_list` / `provider_yike_preview` from ProviderPanel or the intended test/debug harness.
- Confirm normalized DTOs match the fetched resource and are not stale fallback data.
- Repeat with missing/expired auth or blocked network.

Pass: real fetch succeeds when the environment is valid, and auth/network failures return structured errors.

### Fox Runtime Fetch

- Record Fox account/session or client prerequisites, target client state, network environment, and app build.
- Look up a real nickname, UID and chessid through the Fox kifu center (`provider_fox_list` / `provider_fox_list_more` / `provider_fox_preview`), continue a list, and import a previewed game.
- Confirm normalized DTOs match the fetched/captured resource and are not stale fallback data.
- Repeat with unavailable client/session or blocked network.

Pass: real fetch/capture succeeds when prerequisites are valid, and unavailable-client/session/network failures return structured errors.

### readboard Runtime Readiness

- Record exact Next/readboard candidates, executable path, Windows run identity, target client/window and evidence directory.
- Use ReadboardPanel Browse, Save path and Start (`readboard_runtime_start`); verify `readboard_runtime_snapshot` reaches Ready only after wire `220430` handshake. Readiness must not replace the current game or start an engine.
- Exercise a missing executable and the applicable controlled incompatible/timeout/early-exit cases; keep controlled-process evidence separate from the fixed real runtime.
- Stop and Restart through the lifecycle controls; observe the old owned PID/socket release and the new runtime generation before another Ready. A retained cleanup failure must remain visible.

Pass: actual process/protocol readiness and typed failure states agree with the UI, and owned resources are reclaimed. This is not yet live-board synchronization evidence.

### readboard Live Sync

- With the fixed runtime Ready, select a real target board and Start from ReadboardSyncPanel (`begin_readboard_sync` / `prepare_readboard_sync`) or the sidecar. Both explicit entry points can switch from Yike. The first provable frame passes the shared SGF-07 decision; dirty Cancel retains the old document/session.
- Compare accepted stones, dimensions, source move metadata and side to play with the target; observe later frames and target changes in the same authoritative document. Reject old runtime/session frames rather than allowing a competing writer.
- Disconnect the sidecar and verify ErrorPaused retains the last-good read-only game. Retry must obtain a new Ready generation before resuming; Stop leaves an editable document. Observe actual owned process/socket cleanup on runtime Stop and application exit.
- Save/reopen the accepted position and record candidate, target, session/generation, screenshots and runtime evidence. Inherit equivalent prior cases with their original candidates; do not relabel them as new measurements.

Pass: actual target updates and recovery cross the shared owner boundary with the required identity fences. Separately, `readboard_sidecar_sync_snapshot` may preview a pasted protocol line offline; its normalized DTO is not evidence of live target recognition, session ownership or resource cleanup.

### Image OCR Unsupported Path

- Request image-only readboard sync when the current runtime does not provide OCR.
- Confirm the result is a structured unsupported/not-implemented error that names OCR or readboard image sync.
- Confirm the board is not replaced with guessed, stale, or partial data.

Pass: OCR absence is explicit and recoverable; it is not counted as successful live readboard sync.

### Failure Modes

- Exercise bad provider credentials/session, network loss, provider timeout, malformed provider payload, missing sidecar, sidecar crash, sidecar timeout, cancellation, and retry.
- Confirm logs and UI distinguish provider auth, provider network, sidecar process, sidecar protocol, Tauri command, engine, cache, and DTO normalization failures.

Pass: failures are explicit, recoverable where expected, and never reported as successful live provider/sidecar support.

## Packaging Checklist

When production packaging becomes in scope, verify:

- App identifier remains `org.lizzieyzy.next`.
- Frontend output is built from `apps/desktop/dist`.
- Required icons and metadata are present.
- `python3 scripts/validate_release_assets.py --verbose` passes.
- `.github/workflows/release-dry-run.yml` passes on macOS, Linux, and Windows.
- `.github/workflows/release.yml` is validated by `python3 scripts/validate_release_workflow.py --verbose`.
- A `v*` tag release produces macOS, Windows, and Linux assets plus checksum files.
- Missing signing secrets are reported as unsigned dry-run state, not treated as a publish failure.
- Bundled KataGo/runtime assets, if included, match documented paths.
- The app starts without a development server.
- Windows installer or portable package opens on a clean machine.
- macOS app handles Gatekeeper/signing/notarization according to the documented release policy.
- Linux package includes required runtime dependencies or clearly documents them.
- Logs and error messages distinguish UI errors, Tauri command errors, engine errors, and storage/cache errors.
- The full release process, secrets, artifact policy, and rollback plan are recorded in `docs/RELEASE_PROCESS.md`.
- GitHub Release notes include English and Chinese summaries, signing state, checksum guidance, and known limitations.

## Release Notes Guardrails

Release notes for the Next workspace should state:

- The exact release candidate or tag.
- The platform artifacts included.
- The validation commands run and their results.
- The native desktop scenarios result, including OS and KataGo details.
- Known limitations.

Release notes should not state:

- that the Tauri app is a full replacement for the Java/Swing app,
- that Fox/Yike/readboard are migrated,
- that a platform package exists when it was not built,
- that cache/profile persistence covers every legacy setting.

## Handoff Template

Use this shape when handing off a release candidate:

```text
Candidate:
Commit:
Platform:
KataGo:
Model:
Config:

Automated checks:
- python3 scripts/validate_scaffold.py --verbose: PASS/FAIL
- python3 scripts/validate_release_assets.py --verbose: PASS/FAIL
- cargo fmt --all --check: PASS/FAIL/SKIPPED
- cargo clippy --workspace --all-targets -- -D warnings: PASS/FAIL/SKIPPED
- cargo test --workspace: PASS/FAIL/SKIPPED
- provider contract tests: PASS/FAIL/SKIPPED, package/filter:
- Yike runtime fetch: PASS/FAIL/SKIPPED, offline/live:
- Fox chessid fetch: PASS/FAIL/SKIPPED, offline/live:
- Fox uid fetch: PASS/FAIL/SKIPPED, offline/live:
- Fox nickname list: PASS/FAIL/SKIPPED, offline/live:
- readboard domain tests: PASS/FAIL/SKIPPED, package/filter:
- readboard controlled process/socket lifecycle: PASS/FAIL/SKIPPED, package/filter:
- readboard offline protocol snapshot preview: PASS/FAIL/SKIPPED, package/filter:
- image OCR unavailable structured error: PASS/FAIL/SKIPPED:
- npm ci: PASS/FAIL/SKIPPED
- npm test: PASS/FAIL/SKIPPED
- npm run build: PASS/FAIL/SKIPPED
- npm run tauri:build: PASS/FAIL/SKIPPED

Native desktop scenarios:
- SGF open: PASS/FAIL
- Engine profile persistence: PASS/FAIL
- Asset check: PASS/FAIL
- One-position analysis: PASS/FAIL
- Full-game analysis: PASS/FAIL
- Cancel analysis: PASS/FAIL
- Cache hit: PASS/FAIL
- SGF save: PASS/FAIL
- Yike live fetch: PASS/FAIL/SKIPPED
- Fox chessid live fetch: PASS/FAIL/SKIPPED
- Fox uid live fetch: PASS/FAIL/SKIPPED
- Fox nickname live list: PASS/FAIL/SKIPPED
- readboard fixed-runtime ready220430/Stop/Restart and PID/socket cleanup: PASS/FAIL/BLOCKED, candidate/run:
- readboard real-target live owner/frames/Retry/Stop/Save/reopen: PASS/FAIL/BLOCKED, candidate/run:
- image OCR unavailable structured error: PASS/FAIL/SKIPPED

Known limitations:
```
