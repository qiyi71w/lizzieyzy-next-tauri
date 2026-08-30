# Choose Dispositions for Provider, Synchronization, readboard, and External Publishing Capabilities

Type: grilling
Status: resolved
Blocked by: 01, 06

## Question

Given the cross-cutting Entry Point census and [Inventory Provider, Synchronization, readboard, and External Publishing Capabilities](06-inventory-providers-sync-readboard.md), which external workflows are equivalent migrations, deliberate Next redesigns, deferred, abandoned, or implementation-only; what stable Parity Items distinguish preview/import/synchronization and each supported provider mode; and what credential, timeout, recovery, privacy, repository-evidence, and live-evidence contracts are required?

## Answer

### Frozen claim boundary

`PROV-01` remains Partial **Yike preview/one-shot import** at its existing identity, status, and evidence. `PROV-02` remains Partial **Fox preview/one-shot import**. `READ-01` remains Partial **sidecar readiness**. `READ-02` remains Partial **snapshot/import**, with its remaining contract advancing to equivalent readboard ongoing synchronization under the same ID. Accepted `READ-03` **OCR unsupported** stays complete at original scope, status, and evidence; adding image OCR requires a new item. This decision does not mark any capability implemented or Accepted.

Yike `personal` lists, private rooms, and any read path that requires user authentication stay outside `PROV-01` and `PROV-03` until a later successor has a secure integration and live auth evidence. Fully native authenticated Yike move submission is not a claim.

### Canonical preview, import, and synchronization model

- **Preview** fetches, decodes, and displays remote or sidecar data. It never changes the authoritative current game and is dismissible.
- **One-shot import** fully fetches and parses first, asks one dirty-game confirmation, then atomically opens or replaces the current game once through `SGF-07`. It creates no background relationship. Cancel, parse failure, or owner rejection preserves the complete current game.
- **Ongoing synchronization** starts only from an explicit Start sync or Play & Sync action. While active, the external source is authoritative: one dirty-game confirmation on entry, no per-refresh prompts, visible cancellable state, ignored stale results, last-good board preserved on failure, and an editable current game when stopped.

Domain 06 owns locators, fetch, credentials, timeouts, retries, and provider/sidecar failures. Successful trees enter through `SGF-07`. Domain 03 owns later review, edit, and save. Provider sessions are not recovered by `APP-04`.

### Capability dispositions

| Frozen Capability | Disposition | Decision |
| --- | --- | --- |
| `CAP-06-ONLINE-URL` paste room/live URL | Split redesign / Deferred | A native provider center accepts recognized Yike locators without rendering the page: new/old live, old live board, game/hall room, and unite room. Selection previews only (`PROV-01`). Explicit Import is `PROV-01`. Explicit Start sync is `PROV-03`. Tencent/huanle live URLs belong to deferred `PROV-05`, not to Yike or Tencent kifu import. |
| `CAP-06-YIKE-LIVE-CENTER` browse live games then sync | Next redesign | The same provider center lists public `recommend` and `local` modes as preview (`PROV-01`). Import stays `PROV-01`; Start sync and Play & Sync stay `PROV-03`. `personal` is deferred as above. |
| `CAP-06-YIKE-WEB` embedded Yike page with optional auto-sync | Split redesign / Abandoned | Abandon embedded browser hosting. Preserve website play through `PROV-03` Play & Sync: open that public room in the system browser and start the matching Next read-only sync. Stopping Next sync does not close the browser. |
| `CAP-06-YIKE-HALL` embedded Yike hall | Split redesign / Abandoned | Abandon embedded hall hosting. Hall and game-room URLs remain import/sync locators under `PROV-01` / `PROV-03`. |
| `CAP-06-FOX-KIFU` search and import Fox kifu | Equivalent migration | `PROV-02` owns nickname, UID, and `chessid` preview, pagination, bounded recents, and one-shot import through `SGF-07`. Fox is not live synchronization and is not combined with Tencent. |
| `CAP-06-TENCENT-KIFU` search and import Tencent kifu | Equivalent migration | `PROV-04` owns username/`chessId` preview, pagination, bounded recents, and one-shot import through `SGF-07`. It is a separate equivalent migration from `PROV-02` and from deferred live protocols. |
| `CAP-06-READBOARD` sidecar board sync | Split equivalent migration / Deferred | `READ-01` owns sidecar readiness. `READ-02` owns equivalent one-way external-authoritative ongoing sidecar sync, advancing from snapshot/import. GMA (engine moves back to the target) is deferred to domains 04/05. `READ-03` stays the frozen OCR-unsupported claim. |
| `CAP-06-WEBBOARD` LAN web publish and trial play | Deferred | `PUB-01` records LAN board publishing as a stable deferred item. Trial counters and internal trial mechanics are implementation-only and excluded from that item. |
| `CAP-06-SHARE-CURRENT` registered share shortcuts | Abandoned | `Ctrl+E` / `Alt+B` are baseline no-ops. Do not create empty Next share actions. |
| `CAP-06-SYNC-SETTINGS` live/readboard toggles | Route to owner items | Sync intervals, locators, recents, retries, jump-to-last, mute-during-sync, and readboard always-sync/focus belong to `PROV-01`/`PROV-02`/`PROV-03`/`PROV-04` and `READ-02`. They are not duplicate generic `PREF-01` capabilities. Persistence uses `PREF-01` as the durable-preference mechanism only. |
| Unregistered share menubar, toolbar, batch/private/public upload | Implementation-only | Not Entry Points and not Next actions. |
| WebBoard trial internals, leftover Fox jar, sync diagnostics | Implementation-only | Not Parity Items. |

Swing/JCEF widgets, Java dialog close-after-import, and Java config-file keys remain implementation details rather than additional Capabilities.

### Stable Parity Item boundaries

- `PROV-01` — **Yike Preview and One-shot Import**: keep the Partial identity. The native provider center previews public `recommend`/`local` results and recognized Yike URL families without mutating the current game. Explicit Import follows the one-shot model through `SGF-07`. Unite-room locators are in scope. Existing fetch/import plumbing is evidence toward this item, not completion.
- `PROV-02` — **Fox Preview and One-shot Import**: keep the Partial identity and migrate the Fox kifu workflow as equivalent: `chessid` / `uid` / `user_name` lookup, list pagination, bounded recents, and one-shot `SGF-07` import. No Fox live-sync item is created.
- `PROV-03` — **Yike Ongoing Synchronization**: new item, not combined with `PROV-01`. Explicit Start sync applies the ongoing-synchronization model to a public Yike room. **Play & Sync** is the dual-channel redesign: the system browser owns login, cookies, and webpage move submission for that room; Next starts the matching public read-only sync and uses only its signed read path; Next does not read, copy, or log browser authentication. Stopping Next sync leaves the browser open. Native authenticated move submission is out of scope. Embedded web/hall hosting is not this item.
- `PROV-04` — **Tencent Kifu Preview and Import**: new item for the equivalent Tencent kifu workflow (username/`chessId`, `lastCode` pagination, recents, one-shot `SGF-07` import). Not combined with `PROV-02` or with live Tencent/huanle protocols.
- `READ-01` — **readboard Sidecar Readiness**: keep the Partial identity. Probe/ready, incompatible, unavailable, timeout, and restart remain this item; they do not imply a synchronized current game.
- `READ-02` — **readboard Ongoing Synchronization**: keep the Partial identity. Complete acceptance is equivalent one-way external-authoritative sidecar sync under the canonical ongoing-synchronization model, including coordinates, move state, repeat refresh, disconnect, and stop-to-editable. Current snapshot preview is existing Partial evidence, not completion. GMA is not this item.
- `READ-03` — **Explicit OCR Limitation**: keep Accepted. Product/docs do not claim image OCR.
- `PROV-05` — **Tencent/huanle Live Synchronization**: Deferred. Non-Yike live websocket/qipu protocols stay a separate item from `PROV-04`. Current live evidence is not required.
- `PUB-01` — **WebBoard LAN Publishing**: Deferred. Start/stop LAN publish and copy-access URL are in scope later; trial counters/internal trial mechanics stay excluded.

### Shared contracts

**Credentials.** Browser authentication stays in the system browser. Next persists no user secrets, browser cookies, auth material, or signing values in settings or logs. Public Yike read uses the signed read path only.

**Timeouts.** Yike requests use a 10s deadline. Fox and Tencent use 20s connect and 25s read. Visible UI cancellation remains available and is not an earlier hidden timeout.

**Retry.** Retry only idempotent transient reads, at most three retries after the initial attempt. Do not retry auth failures, not-found, malformed or invalid provider content, or user cancellation.

**Recovery.** Ignore stale results. Preserve the last-good board on failure. Stop or cancel leaves an editable current game. Preview never requires recovery of the current game because it does not mutate it.

**Privacy.** Persist only bounded, clearable, non-secret recents (URLs, usernames, UIDs). Clearing provider/search history is owned by these items, not by a broad reset.

**Repository evidence.** Parser and URL-family fixtures; atomic import and sync state transitions through `SGF-07`; cancellation; stale-result suppression; timeout and retry classification; last-good recovery; stop-to-editable behavior.

**Live evidence.** Separate and per mode. Required for in-scope work: Yike public categories; every accepted Yike room URL family; Play & Sync handoff (system-browser room, web login/move, Next public sync); Fox and Tencent lookup modes and pagination; upstream failure and latency; readboard readiness, sync, disconnect, and restart. Deferred personal/auth Yike, `PROV-05`, and `PUB-01` do not need current live evidence.

Existing Partial items remain Partial until both their repository evidence and in-scope live evidence pass. Ticket 15 owns phase ordering; Ticket 16 owns inventory, matrix, and roadmap traceability. Deferred acceptance is recorded now so later work cannot silently broaden it.
