# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Users

Primary users are the same people who already use Java LizzieYzy: Go (weiqi) players who review games with KataGo after a finished game or after obtaining an SGF.

Their job is to open a record, walk the mainline, run local engine analysis, and judge candidates, variations, winrate, and notable mistakes. Coaches, streamers, and first-time Next users are not a separate product audience unless that is confirmed later.

The primary language and community are Chinese. Fox, Yike, and readboard belong to that operating world. English is a parallel language for release notes and current in-app chrome, not the primary audience.

## Product Purpose

LizzieYzy Next is the next-generation desktop Go review workspace. It exists to become the user-facing LizzieYzy, replacing the Java/Swing line once the same review job is proven on Tauri 2 + Rust + TypeScript.

Success is a cooperating operator on their own machine completing the review loop: open or import SGF, configure a local KataGo profile, check assets, analyze a position or the whole game, inspect board overlays and review marks, reuse cached analysis, and save the record. It is not success to claim full Java parity, live Fox/Yike/readboard operation, or signed installers without the matching evidence.

## Positioning

Next is the future shipping product, not a permanent side experiment. Java LizzieYzy remains the behavior reference and the more complete feature set until Next proves the corresponding workflow.

The mechanism a neighboring KataGo GUI cannot truthfully copy is this lineage: LizzieYzy review habits, terminology, and Chinese-server/sidecar workflows, rebuilt behind small testable domains instead of a Swing monolith. The rewrite is an implementation strategy. The product job stays the LizzieYzy review workspace.

## Operating Context

The real product is the Tauri desktop app on Windows, macOS, and Linux (`org.lizzieyzy.next`, binary `lizzieyzy-next-desktop`, window title `LizzieYzy Next`). `npm run dev` is a browser preview for layout and fallbacks. It cannot stand in for native file dialogs, app-data profiles, asset checks, KataGo, SQLite cache, or provider/sidecar commands.

Typical session:

1. Open, import, paste, or load an SGF.
2. Replay with the move slider; optionally edit the SGF text and reparse.
3. Configure one or more local KataGo profiles (engine, model, config, working directory, visits) and persist them in app data.
4. Check that required assets exist, then run one-position or full-game analysis with progress and cancel.
5. Read winrate, candidates, PV, ownership, policy, and review marks; save or Save As the SGF.
6. Reopen the same record and expect the analysis cache to restore prior review frames when auto-load is on.

Optional later paths, expected by Java users but not claimed live from this repository: fetch a game from Yike or Fox; probe/sync a local readboard sidecar. Image OCR is unsupported unless a capable sidecar is separately proven.

Contributor evaluation uses the WSL tree as commit source of truth and a separate Windows clone for native desktop verify. The sibling Java tree is a behavior reference only and is not edited for Next work.

## Capabilities and Constraints

Confirmed in the Next workspace:

- SGF parse, replay, serialize, comments, variations, setup stones, pass, captures, and simple ko through Rust domain logic.
- Native Open / Save / Save As under Tauri; file import and sample SGF in the UI.
- Board, winrate chart, candidates, PV, ownership, policy, and review-mark surfaces.
- KataGo one-shot and full-game batch analysis via analysis JSONL, with progress, cancellation, timeouts, and asset checks.
- Multiple engine profiles in app data; SQLite analysis cache keyed from parsed SGF content and raw SGF hash.
- Preferences: candidate/ownership/policy visibility, candidate limit, auto-load cache, auto-save analysis, default visits, quick/deep review mode, classic/high-contrast board theme.
- Yike/Fox provider and readboard sidecar commands are wired as offline contracts plus runtime path plumbing (`provider_fetch_yike`, `provider_fetch_fox` for `chessid` / `uid` / `user_name`, `readboard_sidecar_probe`, `readboard_sidecar_sync_snapshot`).

Constraints and undecided product facts:

- Not full Java/Swing feature parity. Missing or unproven: live Fox/Yike network behavior, live readboard sidecar, legacy capture/import beyond current SGF flows, full settings migration, layout/theme parity, every analysis shortcut and advanced review workflow, bundled KataGo layout, signed/notarized installers.
- Browser preview may show fake review frames and local cache fallback. That is not the shipped analysis path.
- In-app chrome is currently English while the confirmed audience is Chinese-first. UI language strategy beyond that gap is undecided.
- Accessibility standard (for example WCAG target) is undecided; high-contrast board theme is a preference, not a stated compliance claim.
- Platform is recorded as `web` because the desktop shell hosts one Web UI. This is not an iOS/Android product and does not adapt its design language per OS.

Terminology to keep: SGF, KataGo, visits, PV, ownership, policy, review marks / problem markers, engine profile, analysis cache, Fox, Yike (易棋), readboard.

## Brand Commitments

- Product name: LizzieYzy Next.
- Bundle identity: `org.lizzieyzy.next`; binary `lizzieyzy-next-desktop`; publisher copy uses “LizzieYzy Next contributors”.
- Lineage: next generation of [yzyray/lizzieyzy](https://github.com/yzyray/lizzieyzy), with KataGo as the analysis engine. Historical Fox helpers are references, not the product name.
- Release notes are bilingual English/Chinese. User-facing product language is Chinese-first.
- One visual language on Windows, macOS, and Linux. Do not skin per OS.
- Default chrome is a light, contemporary desktop workbench. Additional themes are planned; do not ship a theme switcher until that work is requested.
- Do not invent a new product name, mascot, or marketing claim set.

## Evidence on Hand

- Product and contract docs: `README.md`, `docs/ARCHITECTURE_NEXT.md`, `docs/MIGRATION_PLAN.md`, `docs/DEVELOPMENT.md`, `docs/RELEASE_PROCESS.md`, `docs/RELEASE_CHECKLIST.md`, `CHANGELOG.md`, `.github/RELEASE_NOTES_v0.1.0.md`, `QA_REPORT.md`.
- Runnable desktop UI: `apps/desktop` (React/TypeScript) and `apps/desktop/src-tauri`.
- Golden SGF fixtures: `tests/golden`.
- Desktop icons: `apps/desktop/src-tauri/icons`.
- Authority when implementation and docs diverge: `docs/ARCHITECTURE_NEXT.md` > `docs/MIGRATION_PLAN.md` > matching development/release docs > `QA_REPORT.md`.

Do not fabricate: user research, testimonials, benchmarks, pricing, licensing, live Fox/Yike session evidence, live readboard device evidence, signed-store listings, or a claim that `docs/TROUBLESHOOTING.md` exists (it is linked and missing).

## Product Principles

1. Serve existing LizzieYzy reviewers. Do not redesign the job for a new audience.
2. Java observable behavior is the truth of the review task. Next may change architecture, not the meaning of open, analyze, mark, cache, save, Fox, Yike, or readboard.
3. The Tauri desktop runtime is the product. Browser preview is a development fallback and is not evidence of shipped UX.
4. Separate wired commands from live capability. Offline contracts and path plumbing are not Fox/Yike/readboard support, and a passing scaffold is not Java parity.
5. Write and design Chinese-first. English may accompany; it must not displace the primary users or their servers and sidecars.
