# LizzieYzy Next

LizzieYzy Next is the in-progress next-generation desktop architecture for the LizzieYzy Go review application. The new line is being built in parallel with the existing Java/Swing maintenance line, using Tauri 2, Rust, and TypeScript so SGF handling, KataGo analysis, engine setup, and local persistence can be tested in smaller modules.

This repository currently contains a functional Next workspace that can be built and run locally, not a published full-parity replacement for the legacy application. The Java/Swing line remains the stable user-facing release path while the Tauri app gains coverage and parity.

## Current Status

Implemented in the Next workspace:

- Tauri 2 desktop shell under `apps/desktop/src-tauri`.
- React + TypeScript + Vite frontend under `apps/desktop`.
- Rust workspace crates for app DTOs, Go rules, SGF parsing/replay/serialization, native Save As outcomes, KataGo protocol normalization, analysis classification, engine management, and SQLite-backed storage/cache.
- One Rust-owned editable SGF workspace with complete-tree `NodePath` navigation, legal move/pass editing, variation removal, personal comments, setup/metadata preservation, and semantic save/reopen.
- Native SGF Open, Save, and Save As through the Tauri desktop backend, including cancellation and failed-write state preservation.
- A Windows-native no-engine workflow covering open, navigate, edit, comment, remove, save, reopen, and rejected ACL Save As.
- KataGo one-position analysis and full-game batch analysis through analysis JSONL.
- Analysis progress events, cancellation, candidate moves, ownership, policy, and winrate/progress overlays.
- Engine path/model/config pickers, asset checks, and multiple engine profiles persisted in app data.
- SQLite analysis cache with cache key computation, lookup, save, and delete commands.
- Scaffold validation, Rust tests, and frontend build checks wired for local and CI use.
- Release preflight validation for Tauri metadata and the safe dry-run workflow.
- Multi-platform GitHub Release workflow for macOS, Windows, and Linux CI-built assets.
- Bilingual English/Chinese release notes for `v0.1.0`.

Not yet claimed as complete in the Next workspace:

- Full legacy Java/Swing feature parity.
- Fox/Yike online game providers as live external-network integrations.
- readboard live sidecar integration in a real target environment.
- Production signing/notarization for macOS and Windows unless maintainer secrets are configured.
- End-to-end clean-machine installer smoke coverage across all target platforms.
- Complete migration of every legacy setting, layout preference, and analysis workflow.

Provider and readboard work in this batch should be treated as offline contract/domain-command coverage until the owning implementation has live environment evidence. Do not describe live provider login, external network capture, or readboard sidecar operation as shipped from this repository alone.

## Migration Overview

[The parity matrix](docs/PARITY_MATRIX.md) is the item-level source of truth. This table is a compact roll-up, not a second status tracker.

Status and evidence are separate. The evidence ladder is `Not started` → `Scaffolded` → `Behavior implemented` → `Repository tested` → `Native/live verified`. Environment-independent behavior can be accepted at `Repository tested`; native/live evidence is not required for those items.

| Capability | Parity items | Status | Highest completed evidence stage | Main remaining gap |
| --- | --- | --- | --- | --- |
| Baseline and migration inventory | `BASE-01`–`BASE-02` | Accepted | Repository tested | Maintain evidence as later slices land. |
| SGF document and tree-shaped wire model | `SGF-01`–`SGF-02` | Accepted | Repository tested | None for R1. |
| Native variation navigation, editing, comments, and save/reopen | `SGF-03`–`SGF-06` | Accepted | Native/live verified | None for R1. |
| Go rules required by SGF editing | `RULE-01` | Accepted | Repository tested | None for the frozen R1 fixtures. |
| Core review presentation and interaction | `UI-01`–`UI-03`, `UI-05` | Accepted / Partial | Native/live verified | `UI-02` still lacks engine-event delivery evidence while a board mutation promise is pending. |
| No-engine desktop workflow | `UI-04` | Accepted | Native/live verified | None. |
| Engine profiles and asset checks | `ENG-01` | Accepted | Repository tested | None within this item. |
| Foreground engine lifecycle, switching, rollback, and jobs | `ENG-02`–`ENG-05` | Missing / Partial | Scaffolded | Authoritative lifecycle/job identity, controlled tests, and real KataGo smoke. |
| Interactive and whole-game analysis | `ANA-01`–`ANA-04` | Partial | Repository tested | Manager-owned lifecycle binding and controlled/native KataGo evidence. |
| Analysis cache basics | `ANA-05` | Accepted | Repository tested | Branch-aware cache decisions remain in later R4 work. |
| Preferences | `PREF-01` | Partial | Repository tested | Complete settings inventory and native restart evidence. |
| Adjustable and persisted layout | `LAYOUT-01`–`LAYOUT-03` | Missing | Not started | Splitters, persistence, and narrow reset behavior. |
| Engine game modes | `GAME-01`–`GAME-03` | Missing | Not started | Session state, controls, batch revision, and SGF integration. |
| Yike and Fox providers | `PROV-01`–`PROV-02` | Partial | Repository tested | Live sessions, network behavior, and recovery evidence. |
| readboard probe and synchronization | `READ-01`–`READ-02` | Partial | Repository tested | Live sidecar evidence and active-game import. |
| Explicit OCR limitation | `READ-03` | Accepted | Repository tested | None until OCR support is deliberately introduced. |
| Release preflight | `REL-01` | Partial | Repository tested | Installable production artifact evidence. |
| Signing, updater, and installer workflows | `REL-02`–`REL-04` | Missing | Scaffolded | Production credentials, hosted update behavior, and platform smoke. |
| Bundled runtime assets | `REL-05` | Partial | Repository tested | Packaged-application resolution smoke. |

## Technology Stack

- Desktop runtime: Tauri 2.
- Backend: Rust workspace, Tauri commands, SQLite via `rusqlite`.
- Frontend: React, TypeScript, Vite, `@tauri-apps/api`.
- Core domains: SGF, Go rules, KataGo analysis JSONL, engine profiles, analysis cache.
- Validation: `scripts/validate_scaffold.py`, Rust unit tests, frontend build, and smoke checks.

## Repository Map

- `apps/desktop`: Next React desktop UI.
- `apps/desktop/src-tauri`: Tauri 2 command gateway and native desktop integration.
- `crates/app-model`: Shared DTOs used across Rust and TypeScript boundaries.
- `crates/go-core`: Board state and Go rule logic.
- `crates/sgf`: SGF parsing, tree mutation, replay, and serialization.
- `crates/save-as-dialog`: Platform-neutral Save As outcome and state-transition rules.
- `crates/katago-protocol`: KataGo analysis query/response models.
- `crates/analysis-core`: Candidate/problem classification helpers.
- `crates/engine-manager`: Engine command specs, asset checks, process execution, and cancellation.
- `crates/storage`: SQLite storage/cache schema helpers.
- `docs/ARCHITECTURE_NEXT.md`: Current Next architecture and module boundaries.
- `docs/JAVA_BASELINE.md`: Frozen Java behavior reference and successor policy.
- `docs/PARITY_MATRIX.md`: Stable parity items, status, evidence, gaps, and acceptance.
- `docs/MIGRATION_PLAN.md`: Migration phase order and next executable slices.
- `docs/DEVELOPMENT.md`: Local development and smoke validation commands.
- `docs/RELEASE_CHECKLIST.md`: Release-readiness checklist and manual acceptance flow.

## Development Commands

Run scaffold validation from the repository root:

```bash
python3 scripts/validate_scaffold.py --verbose
```

Run release asset preflight from the repository root:

```bash
python3 scripts/validate_release_assets.py --verbose
python3 scripts/validate_release_workflow.py --verbose
```

Run the Rust checks:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Run the frontend build:

```bash
cd apps/desktop
npm ci
npm run build
```

Run the Next app in development:

```bash
cd apps/desktop
npm run tauri:dev
```

For a non-authoritative browser-only preview without native Tauri commands:

```bash
cd apps/desktop
npm run dev
```

The browser preview can exercise presentation fallbacks, local SGF parsing, fake review frames, and browser-local cache. Rust-owned current-game editing and authoritative Save are unavailable there. Those behaviors, real KataGo execution, native file dialogs, asset inspection, and app-data profile persistence require the Tauri desktop runtime.

## Local Smoke Flow

Use `docs/DEVELOPMENT.md` for the full checklist. The core no-engine acceptance path is:

1. Run `python3 scripts/validate_scaffold.py --verbose`.
2. Start `npm run tauri:dev` in `apps/desktop`.
3. Open `tests/golden/editable-workspace-branching.sgf`.
4. Navigate both sibling variations and verify board, move data, captures, next player, and comments follow the selected node.
5. Add a legal move or pass and a personal comment, then remove a different sibling variation.
6. Cancel Save As once and verify the current path and dirty state do not change.
7. Save the edited game, reopen it, and verify the retained edit/comment, removed sibling, setup, and metadata.

Engine configuration, one-position analysis, whole-game analysis/cancellation, and cache checks are separate regression paths; they are not prerequisites for SGF workspace use.

## CI Status

CI should be read as scaffold and regression coverage for the Next workspace, not as proof of full legacy parity. The important gates are:

- scaffold validation,
- release asset preflight and production release workflow contract validation,
- frontend dependency install and build,
- Rust formatting,
- Rust clippy,
- Rust tests.

Provider contract tests and readboard domain tests are accepted through the Rust workspace test gate when those modules land; the handoff should name the exact package or test filter and must not count a zero-test filter as evidence. Release dry-run acceptance is `.github/workflows/release-dry-run.yml` plus `python3 scripts/validate_release_assets.py --verbose`; the workflow uploads diagnostic artifacts and must not create a GitHub release.

Passing CI means the current Tauri/Rust/TypeScript baseline is structurally healthy. It does not mean live Fox/Yike/readboard integrations, platform signing, notarization, or clean-machine installer smoke checks have completed.

## Releases

`v0.1.0` is the first public Tauri release candidate for this repository. Release notes are bilingual:

- [Release notes v0.1.0](.github/RELEASE_NOTES_v0.1.0.md)
- [Changelog](CHANGELOG.md)

The production release workflow is `.github/workflows/release.yml`. It runs on `v*` tags, builds macOS, Windows, and Linux Tauri bundles, collects assets, writes SHA-256 checksum files, and publishes a GitHub Release. Assets generated without signing or notarization secrets are clearly marked as unsigned release-candidate artifacts.

## Documentation

- [Next architecture](docs/ARCHITECTURE_NEXT.md)
- [Java behavior baseline](docs/JAVA_BASELINE.md)
- [Parity matrix](docs/PARITY_MATRIX.md)
- [Migration plan](docs/MIGRATION_PLAN.md)
- [Development guide](docs/DEVELOPMENT.md)
- [Release checklist](docs/RELEASE_CHECKLIST.md)
- [Troubleshooting](docs/TROUBLESHOOTING.md)

## Acknowledgements

- Original project: [yzyray/lizzieyzy](https://github.com/yzyray/lizzieyzy)
- KataGo: [lightvector/KataGo](https://github.com/lightvector/KataGo)
- Historical Fox references:
  - [yzyray/FoxRequest](https://github.com/yzyray/FoxRequest)
  - [FuckUbuntu/Lizzieyzy-Helper](https://github.com/FuckUbuntu/Lizzieyzy-Helper)
