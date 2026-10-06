# QA Report

## Batch Scope

This QA report covers the documentation and acceptance contract for the current Tauri 2 + Rust + TypeScript migration batch. It separates repository-level evidence from live external environment evidence:

- Implemented repository claim: provider/readboard contracts and runtime paths are documented where owning code and tests land. The command boundaries are `provider_yike_list` / `provider_yike_preview`, `provider_fox_list` / `provider_fox_list_more` / `provider_fox_preview`, `provider_tencent_list` / `provider_tencent_preview`, `readboard_runtime_*`, and the shared external-sync owner. `readboard_sidecar_sync_snapshot` is an offline preview.
- Live evidence: R10 predecessor runs document actual public Yike/Fox/Tencent reads and Windows readboard target synchronization in DEVELOPMENT. Ticket08's integrated Windows source candidate proves cross-source handoff, failure/Cancel retention, policy cancellation, native Save and exit/recovery. Tickets01–09 are complete, with zero follow-up candidates in ticket09's closeout of final integration `91b4e17`. Per-platform installed-network evidence remains Not run, explicitly non-blocking for R10 completion under the approved 2026-10-06 exception. DEVELOPMENT retains exact candidate attribution and limits; installed/platform acceptance is not claimed.
- Release claim limit: do not describe the Next app as a 100% legacy replacement or as fully live-provider-ready from scaffold/preflight checks alone.
- UI surfaces: ProviderPanel hosts the three provider centers and ReadboardPanel lifecycle; YikeSyncPanel and ReadboardSyncPanel consume the shared external-sync owner. Browser preview cannot establish native IPC or resource cleanup.

## Ownership Check

- Current branch ownership is multi-worker: Rust workspace, Tauri backend, TypeScript frontend, provider crates, readboard sidecar, release assets, and docs may all appear in the branch diff.
- Worker-C-Fix ownership for this pass is limited to QA/docs claim accuracy in `QA_REPORT.md` and `docs/**`.
- Code changes under `crates/**`, `apps/desktop/**`, package/build files, and workflows are treated as other-worker work; this pass documents their acceptance boundaries without reviewing, editing, or reverting them.

## Validation Matrix

| Area | Repository Evidence | Live Environment Evidence |
| --- | --- | --- |
| Scaffold contract | `python3 scripts/validate_scaffold.py --verbose` | Not applicable. |
| Release metadata/preflight | `python3 scripts/validate_release_assets.py --verbose` | Platform packaging/signing still requires real artifact builds. |
| Yike provider | Public-center/locator contract tests and `provider_yike_list` / `provider_yike_preview` runtime checks. | Ticket03 public reads and ticket06 ongoing updates, browser handoff and terminal transition are recorded in DEVELOPMENT with original candidates; installed network admission remains separate. |
| Fox provider | Lookup/list/cursor/preview/recents contract tests and `provider_fox_*` runtime command checks for nickname, UID and chessid. | Ticket04 native list/preview/import and provider-live upstream failure/latency passed on `f1270ba`; DEVELOPMENT records the original candidate and limitations. |
| Tencent provider | Independent username/chessId, cursor, history and explicit import workflow. | Ticket05 native/provider-live evidence on `844fa4c` / `e201634` is recorded in DEVELOPMENT; installed network admission remains separate. |
| readboard readiness | Controlled Child/socket lifecycle and `readboard_runtime_*` checks cover readiness, typed faults, generations and cleanup. | Fixed Windows readboard `cdcc7b3` readiness/Stop/Restart/exit passed on Next `eadb577`, then revalidated on `f75cacf` after the ordered-stream change (DEVELOPMENT §3). |
| readboard sync | Frame/Go/SGF/current-game tests cover the shared owner, turn recovery, stale frames and last-good pause/Retry/Stop. | Next `f75cacf` real Fox spectator-target run and `eb5736a` room-change turn repair evidence are recorded in DEVELOPMENT §4; offline snapshot preview is a separate boundary. |
| image OCR | Image-only sync returns a structured unsupported/not-implemented error when OCR is unavailable. | OCR is SKIPPED/UNSUPPORTED unless an OCR-capable runtime and image fixture evidence are available. |
| Legacy parity | Documented as partial migration only. | Full parity requires explicit evidence for providers, settings, packaging, and advanced review workflows. |

## Manual Smoke Additions

The release and development docs now require explicit manual smoke entries for:

- Yike runtime fetch success and auth/network failure modes.
- Fox nickname, UID and chessid lookups, list continuation, preview/import, and network/upstream failure modes.
- Readboard actual readiness, incompatible/unavailable/timeout, Stop/Restart and owned process/socket cleanup.
- Readboard continuous real-target synchronization, including room changes, disconnect, Retry, Stop-to-edit and Save/reopen.
- image OCR unavailable as a structured unsupported/not-implemented error.
- Boundary-specific failure reporting for provider auth, provider network, sidecar process, sidecar protocol, Tauri command, cache, engine, and DTO normalization errors.

## Manual Smoke Matrix

| Scenario | Required Result |
| --- | --- |
| Yike fetch success | `provider_yike_list` / `provider_yike_preview` return normalized DTOs for real public Yike resources; locator, candidate SHA, route, result count and latency are recorded. |
| Yike fetch failure | Missing/expired auth, blocked network, timeout, malformed payload, or invalid request returns structured errors and no stale live-success claim. |
| Fox `chessid` preview | `provider_fox_preview` returns a normalized SGF preview for a real `chessid`; Import goes through SGF-07. |
| Fox `uid` list | `provider_fox_list` / `provider_fox_list_more` return real batches that continue without repeats to the end. |
| Fox nickname list | `provider_fox_list` resolves a real nickname and records found/not-found behavior. |
| Fox fetch failure | Unavailable client/session, blocked network, timeout, bad command, malformed payload, or invalid request returns structured errors. |
| readboard missing/incompatible runtime | Native lifecycle Start reports typed unavailable/incompatible/timeout states without changing the current game; controlled fault evidence is labelled separately. |
| readboard fixed-runtime readiness | ReadboardPanel Browse/Save/Start reaches ready220430; candidate/path/PID/endpoint are recorded, and Stop/Restart reclaim the old owned process/socket. |
| readboard live shared-owner sync | Explicit Start admits the first real-target frame through SGF-07; dirty Cancel retains the old owner, later frames follow one session, disconnect pauses last-good, Retry uses a new Ready generation, Stop permits editing and exit releases resources. |
| readboard offline snapshot preview | `readboard_sidecar_sync_snapshot` normalizes a pasted protocol line only; it does not establish live recognition, synchronization or cleanup. |
| image OCR unavailable | Image-only sync returns structured unsupported/not-implemented error; OCR is not claimed live unless separately validated. |

## Local verification

- `python3 scripts/validate_scaffold.py --verbose`: PASS, 10 passed, 0 failed.
- `python3 scripts/validate_release_assets.py --verbose`: PASS, 4 passed, 0 failed.

## Unverified External Conditions

- Real Yike service access, credentials/session behavior, and provider rate-limit/network behavior.
- Real Fox client/session/capture prerequisites and provider network behavior.
- Real readboard sidecar installation, compatible version, target client/window state, and protocol behavior.
- Platform release packaging, signing, notarization, installer behavior, and clean-machine startup.
