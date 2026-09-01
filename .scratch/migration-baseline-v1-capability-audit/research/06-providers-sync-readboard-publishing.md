# 06 — Providers, Synchronization, readboard, and Publishing Census

Independent Domain 06 census for [Ticket 06](../issues/06-audit-providers-sync-readboard-publishing.md). Destination writes land in `docs/JAVA_CAPABILITY_INVENTORY.md` Domain 06, Matrix-owned fields for `PROV-01`–`PROV-07`, `READ-01`–`READ-03`, `PUB-01`, and `CONTRIB-01`, and existing Plan R10 / Deferred membership. This ticket owns the `CONTRIB-01` Matrix contract; Domain 05 `GM-CONTRIBUTE` remains the occupancy row; `GAME-09` records only its dependency.

## Sources

| Tree | Path | Commit |
| --- | --- | --- |
| Java Migration Baseline v1 | `/home/dev/dev/weiqi/worktrees/lizzieyzy-next/java-baseline-v1` | `7b4027531c2b26062d0bfc27a040cc550cfbea4d` (verified `git rev-parse HEAD`) |
| Next inventory baseline | this worktree | `18c6d189b8b01069975c4c40ead63a010249cb8c` (Accepted-field comparison only) |
| Current destination docs | this worktree | `f2c56e7c1f37249334d0acb0da9ada3cfb9fcc7c` at census time |

Read-only Java source census. No production edits, tests, builds, or runtime launches. Prior coverage-audit research 06 and Tickets 13, 22, 24, 25, 26 are product-decision sources, not a stop condition for this count.

## Method

1. Walk File → Open online link, Sync menu, BottomToolbar live popup, top `showBasicBtn` strip, and `Input` / `InputIndependentMainBoard` keys from `Menu.java`, `BottomToolbar.java`, `Input.java`, `LizzieFrame.java`.
2. Count a Capability only when it is one observable user goal. Count an Entry Point only when a control is constructed **and** added, shown, or registered.
3. Keep Yike public browse/URL, Fox kifu, and Tencent kifu as separate goals. Preview, one-shot import, and ongoing sync are modes of those goals, not extra Capability IDs.
4. After the census completed, compare the computed count with reference 10. 10 was not used as a stop condition.
5. Runtime-check is allowed only for a named dynamic-visibility, default, persistence, or failure fact not recoverable from frozen source. Provider-live, sidecar-live, credential, and install-state evidence are Matrix / `REL-*` classes, not Domain 06 Java runtime checks.
6. Do not list compiled Yike AppKey, AppSecret, or signing material.

Key Java files: `gui/Menu.java`, `gui/BottomToolbar.java`, `gui/Input.java`, `gui/InputIndependentMainBoard.java`, `gui/LizzieFrame.java`, `gui/OnlineDialog.java`, `gui/YikeLiveDialog.java`, `gui/YikeUrlParser.java`, `gui/YikeApiClient.java`, `gui/BrowserFrame.java`, `gui/FoxKifuDownload.java`, `gui/TencentKifuDownload.java`, `analysis/GetFoxRequest.java`, `analysis/GetTencentRequest.java`, `analysis/ReadBoard.java`, `gui/WebBoardManager.java`, `util/NetworkProxy.java`, `Config.java`.

## Computed count versus reference

**Computed Domain 06 Capabilities: 10.**

Reference 10. Extra rows: none. Missing rows: none.

IDs: `CAP-06-ONLINE-URL` `CAP-06-YIKE-LIVE-CENTER` `CAP-06-YIKE-WEB` `CAP-06-YIKE-HALL` `CAP-06-FOX-KIFU` `CAP-06-TENCENT-KIFU` `CAP-06-READBOARD` `CAP-06-WEBBOARD` `CAP-06-SHARE-CURRENT` `CAP-06-SYNC-SETTINGS`.

Corrections versus the prior coverage-audit write-up (same 10 IDs; not extra/missing rows):

- `OnlineDialog.txtRefreshTime` field default is `"1"` (`OnlineDialog.java:348-358`). `proc()` falls back to 10 when the parsed value is not `> 0` (`834-836`).
- Fox recents persist `fox-recent-searches` (≤8). Tencent recents persist `tencent-recent-searches` (≤8) plus `last-tencent-name` and `tencent-after-get`.
- Off-Windows `Alt+O` is source-complete: `Input` is not OS-gated, but `LizzieFrame.openBoardSync()` returns after stderr when `!isNativeBoardSyncSupported()` (`2958-2965`).
- Share toolbar checkbox is constructed; `customToolbarItem.add(shareButton)` is commented (`Menu.java:4858-4860`). Share menubar `this.add(shareKifu)` is commented (`4043-4047`).

## Provider Network Policy and Java proxy occupancy

Java `NetworkProxy.DEFAULT_MODE = direct`, host `127.0.0.1`, port `7897` (`NetworkProxy.java:19-27`). Yike/Fox/Tencent HTTP uses `NetworkProxy.openConnection`. That Java UI and `network-proxy-mode/host/port` are Abandoned with `SET-NETWORK-PROXY` (Domain 02). Next has **no** proxy Parity Item and **no** application proxy preference.

Next-owned remote HTTP(S)/WebSocket(S) for `PROV-01`–`PROV-07` uses Provider Network Policy: process-environment override, then Windows/macOS platform resolution including system-owned PAC/WPAD, or Linux proxy environment variables; honor `NO_PROXY`; re-resolve redirects; no direct fallback; system trust and authentication; sanitized Retry. Browser-owned traffic, local readboard, inbound WebBoard, SSH, and updates are excluded. Contribution uses a separate Contribution Network Policy on `CONTRIB-01`.

## Provider / sidecar / publishing Capabilities

| Frozen ID | Observable user goal | Registered Entry Points | Default / persistence | Failure / non-mutation | Source | Runtime-check | Mapping |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `CAP-06-ONLINE-URL` | Paste a recognized room/live URL and synchronize it into the current game | File → Open online link constructed and `fileMenu.add` (`Menu.java:212-222`); `Input` `Q` without Ctrl/Alt (`Input.java:701-707`); Live Center Sync/double-click/Enter → `syncOnline` (`YikeLiveDialog.java:284-293`) | `txtRefreshTime` UI default `"1"`; `proc()` uses that value when `> 0`, else 10. `alwaysGotoLastOnLive` field/load false; `openHtmlOnLive` field/load true. Persist those ui keys. No URL credential store. | Applying a URL starts ongoing synchronization via `proc()` (`OnlineDialog.java:834-859`); Java has no separate non-mutating URL preview control. Pending Yike session promotes only when `syncReady` **and** `geometryReady` (`OnlineDialog.shouldPromotePendingSession` `3717-3724`). Waiting-page URLs invalidate placement geometry without clearing the display session. Tencent/huanle live uses compiled handshake bytes (`OnlineDialog.java:137-156`) — Deferred `PROV-05`, not this public Yike locator set. Yike HTTP timeout 10s. | `Menu.java:212-222`; `Input.java:701-707`; `OnlineDialog.java:137-156,348-358,834-836,3717-3724`; `YikeUrlParser.java:8-129`; `YikeLiveDialog.java:284-293`; `YikeApiClient.java:26-35` | Not required. Tencent/huanle live start from a pasted URL is Deferred `PROV-05` provider-live, not a Domain 06 Java runtime check. | Split: recognized Yike locators (new/old live, old live board, game/hall room, unite room) Preview/Import → `PROV-01`; explicit Start sync → `PROV-03`. Tencent/huanle live URLs → Deferred `PROV-05`. Successful trees enter `SGF-07`. |
| `CAP-06-YIKE-LIVE-CENTER` | Browse public Yike live games, then start sync | Sync → 弈客直播 constructed and `live.add` + Shift+O (`Menu.java:4134-4144`); `Input` Shift+O (`532-538`); `InputIndependentMainBoard` Shift+O (`328-334`); BottomToolbar live popup (`1050-1059`) | Dialog 920×420; first category Recommend `official="1"`; Local `"0"`; Personal `"4"`. Guest `usertoken=-1`. HTTP 10s. Status must be 1200. No persist for category/page. Compiled-in AppKey/signing headers exist; values are not listed. | `loadFailed` status; buttons disabled while loading. No retry loop in `fetchLiveList`. All three categories use the same guest signed GET. | `Menu.java:4134-4144`; `Input.java:532-538`; `BottomToolbar.java:1050-1059`; `YikeLiveDialog.java:41-163,231-293`; `YikeApiClient.java:26-76` | Not required. Whether guest `official=4` returns meaningful Personal results is `PROV-06` provider-live. | Public `recommend`/`local` → `PROV-01` / `PROV-03`. Frozen Personal-category discovery → Deferred `PROV-06`. Java proves no Yike login, private-room type, or native move API; authorized read/play is Deferred `PROV-07`. |
| `CAP-06-YIKE-WEB` | Open Yike in the embedded browser (optional auto-sync) | Sync → 打开弈客网页版 (`Menu.java:4146-4155`); Live Center Web; BottomToolbar same item (`1061-1068`) | JCEF bundle required. `openHtmlOnLive` default true. Observed-URL freshness 30s (`BrowserFrame.YIKE_OBSERVED_URL_FRESH_MILLIS`). | `BrowserFrame` startFailed; Retry or Open in system browser. | `Menu.java:4146-4155`; `BottomToolbar.java:1061-1068`; `BrowserFrame.java:42-80` | Not required. Observed-URL auto-sync is JCEF implementation of Abandoned embedding. | Abandon embedded hosting. Preserve website play through `PROV-03` Play & Sync (system browser + matching Next read-only sync). Stopping Next sync does not close the browser. Hall/game URLs remain locators under `PROV-01` / `PROV-03`. |
| `CAP-06-YIKE-HALL` | Open Yike hall in the embedded browser | Sync → 弈客大厅 (`Menu.java:4157-4168`); BottomToolbar 弈客大厅 (`1071-1081`) | Fixed `https://home.yikeweiqi.com/#/game`. Persistence N/A. `YikeUrlParser` hall → `TYPE_GAME_ROOM`; unite → `TYPE_UNITE_ROOM` (`101-125`). | Same JCEF failure path as `CAP-06-YIKE-WEB`. | `Menu.java:4157-4168`; `BottomToolbar.java:1071-1081`; `YikeApiClient.YIKE_GAME_URL`; `YikeUrlParser.java:101-125` | Not required | Abandon embedded hall hosting. Hall and game-room URLs remain import/sync locators under `PROV-01` / `PROV-03`. |
| `CAP-06-FOX-KIFU` | Search Fox games by nickname/UID/`chessid` and one-shot import | Sync → 野狐(腾讯)棋谱 (`Menu.java:4171-4180`); BottomToolbar live popup (`1084-1092`); top `btnFoxKifu` added under `showBasicBtn` default true (`8177`, `Config.showBasicBtn` `927,1793`) | Connect 20s, read 25s, `HTTP_MAX_RETRIES=3`, sleep `350ms×attempt`. `foxAfterGet` default 0; `last-fox-name` default empty. Persist those plus `fox-recent-searches` (≤8 uid/nickname, no passwords). Help clear-personal-data removes that recents key. UI 25/tab. | Empty list message; CGI then H5 fallback; `receiveResult` ignores `source != foxReq` (stale-safe). Preview does not mutate; import is one `SGF-07` replacement. Not live sync. | `GetFoxRequest.java:27-40,150-266`; `FoxKifuDownload.java:470-478,849-877`; `Menu.java:4171-4180,8177`; `BottomToolbar.java:1084-1092` | Not required | Equivalent `PROV-02`. Not combined with Tencent. Not live synchronization. |
| `CAP-06-TENCENT-KIFU` | Search Tencent games by username/`chessId` and one-shot import | Sync → Tencent kifu (`Menu.java:4182-4191`); top `btnTencentKifu` (`8178`). **Not** in BottomToolbar live popup. | Connect 20s, read 25s, 3 retries, sleep `350ms×attempt`. Session default `lizzieyzy-next`. Fetch num 100; UI 25/tab. Persist `tencent-recent-searches` (≤8), `last-tencent-name`, `tencent-after-get` (load fallback `foxAfterGet`, default 0). | Same timeout/retry class as Fox; `emitError` on exception. Preview does not mutate; import is one `SGF-07` replacement. Not live sync. | `GetTencentRequest.java:17-29,150-186`; `TencentKifuDownload.java:84-98,825-854`; `Menu.java:4182-4191,8178` | Not required | Equivalent `PROV-04`. Separate from `PROV-02` and from Deferred `PROV-05`. |
| `CAP-06-READBOARD` | Synchronize an external client board via local sidecar | Sync → 棋盘识别工具 **Windows `live.add` only** + Alt+O accelerator (`Menu.java:4193-4205`); BottomToolbar 棋盘同步 Windows only (`1094-1102`); `Input` Alt+O (`532-536`) and `InputIndependentMainBoard` Alt+O (`328-332`) are **not** OS-gated | Sidecar `readboard.exe` / `.bat`. `use-pipeline-readboard` default false; `alwaysSyncBoardStat` true; `readBoardGetFocus` true; `notPlaySoundInSync` true. Process exit 1000ms / destroy 200ms; pending ACK 3000ms. Persist always-sync, get-focus, pipeline, suppress-notice. GMA token `gma` is not this Capability. | Missing exe/bat; place failed; GMA unsupported without KataGo. Off-Windows Alt+O: `openBoardSync()` prints stderr and returns (`LizzieFrame.java:2958-2965`). Local sidecar is excluded from Provider Network Policy. | `Menu.java:4193-4205`; `BottomToolbar.java:1094-1102`; `Input.java:532-536`; `LizzieFrame.java:2958-2965`; `ReadBoard.java:43-131`; `Config.java:101,1636,1743,2001` | Not required | `READ-01` sidecar readiness. `READ-02` one-way external-authoritative ongoing sidecar sync. `READ-03` stays the frozen OCR-unsupported claim. Both legacy engine play-back modes → Deferred `GAME-10` (link only; do not edit `GAME-10`). |
| `CAP-06-WEBBOARD` | Publish the current board to a LAN web viewer | Sync → WebBoard Start/Stop + Copy URL (`Menu.java:4208-4269`); BottomToolbar Start/Stop + Force-exit trial (`1105-1148`). Trial is an Entry Point of this Capability, not a new ID. | HTTP 9998 try +0..9; WS 9999 skip in-use; max-conn 20; bind `0.0.0.0`; idle 5 min. Optional config `web-board` object. | Start returns false if no HTTP/WS port; WS fail stops HTTP; trial denied `in_use` / `engine_busy`; Copy URL no-ops if not running. Inbound WebBoard is excluded from Provider Network Policy. | `Menu.java:4208-4269`; `BottomToolbar.java:1105-1148`; `WebBoardManager.java:24-227`; `WebBoardServer.java:10-73`; `WebBoardHttpServer.java:16-70` | Not required | Deferred `PUB-01` start/stop/copy URL. Trial counters and internal trial mechanics are excluded from that item. |
| `CAP-06-SHARE-CURRENT` | Registered share shortcuts with no working action | `Ctrl+E` and `Alt+B` in `Input` (`317-319,663-666`) and `InputIndependentMainBoard` (`122,461`) call `shareSGF()` | N/A. No observable state is written. | `shareSGF()` / `batchShareSGF()` bodies are fully commented (`LizzieFrame.java:11570-11578`). No feedback. Reachable no-op. | `Input.java:317-319,663-666`; `InputIndependentMainBoard.java:122,461`; `LizzieFrame.java:11570-11578`; `Menu.java:4043-4047,4858-4860` | Not required | Exclusion — Abandoned product capability. Do not create empty Next share actions. Unregistered share menubar/toolbar/batch/private/public upload are **not** this Capability. |
| `CAP-06-SYNC-SETTINGS` | Live/readboard preference toggles | Sync checkboxes constructed and `live.add`: open HTML on live; always jump to last live move; readboard always-sync; readboard get-focus (`Menu.java:4273-4324`) | Field/load: `openHtmlOnLive=true`; `alwaysGotoLastOnLive=false`; `alwaysSyncBoardStat=true`; `readBoardGetFocus=true`. Persist those ui keys. Listeners do **not** `config.save()` (SC-01/SC-03). Mute-during-sync is Domain 02 `SET-SOUND`, routed here. | No independent setting-error channel. Persist-write failure presentation is SC-01/SC-03, not extra UI. | `Menu.java:4273-4324`; `Config.java:95-103,1743,2001` | Whether each toggle applies to an already-running sync or readboard session (dynamic) | Route to owners: locators/recents/retries/jump-to-last/`openHtmlOnLive` → `PROV-01`–`PROV-07`; always-sync/focus/mute-during-sync → `READ-02` / `PROV-03`. `PREF-01` is mechanism only. `PROV-07` secrets use the System Credential Store. |

## Explicit non-rows (not extra Domain 06 Capabilities)

| Surface | Why it is not a Domain 06 Capability |
| --- | --- |
| `GM-CONTRIBUTE` occupancy / Contribute menu | Domain 05 census row. This ticket owns Parity Item `CONTRIB-01` only. |
| `SET-NETWORK-PROXY` Java Direct/System/Manual UI | Domain 02 Abandoned occupancy. Provider Network Policy applies on `PROV-01`–`PROV-07`. |
| GMA / `gma` sidecar token / “该模式仅支持 KataGo” | Deferred `GAME-10` External-board Engine Match (Ticket 25 / ADR 0004). `READ-02` stays sync-only. |
| Capture Tsumego Shift+E | Domain 03 puzzle, not provider sync. |
| `shareKifu` menubar / share toolbar checkbox | Constructed but not added. Unregistered, not Abandoned-by-themselves. |
| Batch/private/public share / `PublicKifuSearch` | Only behind the unregistered Share menu. |
| `SyncDiagnosticsDialog` | No Menu / toolbar / shortcut registration found. |
| `EnableEnterYikeGame` | Entry Point of `CAP-06-SYNC-SETTINGS` (`openHtmlOnLive`), not a second URL Capability. |
| `fox-after-get` / `tencent-after-get` / recents keys | Persistence of Fox/Tencent, not extra Capabilities. |
| WebBoard trial internals / `forceExitTrial` | Entry Point of `CAP-06-WEBBOARD`; excluded from `PUB-01` contract. |
| JCEF `BrowserFrame` class / bundle | Implementation of `CAP-06-YIKE-WEB` / `HALL`. Next abandons embedding. |
| Leftover `FoxRequest.jar` | Unused binary. |
| Remote compute / SSH engine | Domain 04 / Deferred `SSH-01` / `RCOMP-01`. |
| File Save / clipboard SGF | Domain 03. |

## Owner routing (this ticket records links only)

| Concern | Owner | Domain 06 records |
| --- | --- | --- |
| Parse / replace after fetch | `SGF-07` | Link from Preview/Import/sync success. One dirty confirmation. Cancelled/failed replace preserves the current game. |
| Durable preference mechanism | `PREF-01` | Link from recents, jump-to-last, always-sync, focus, `openHtmlOnLive`. Not semantic owner. |
| Graceful shutdown / sidecar teardown | `APP-03` | Link from ongoing sync stop and readboard process wait. |
| Session recovery | `APP-04` | Provider, readboard, Match, and contribution runs are **not** restored. |
| Match Session turns | `GAME-01` / `GAME-05` | Link from Deferred `PROV-07` and `GAME-10` only. Do not edit those contracts. |
| Sidecar confirmation / engine play-back | `GAME-10` | Link from GMA. Do not expand `READ-02`. |
| Histories / recents clear | `SGF-09` / Help clear-personal-data | Link: Fox/Tencent recents keys. |
| Update OS proxy | `REL-03` | Not Provider Network Policy. |
| Contribution client component | `REL-05` | `CONTRIB-01` uses the shared manifest/acquisition protocol after acceptance. |
| Contribute occupancy | `GM-CONTRIBUTE` (Domain 05) | `CONTRIB-01` Matrix owned here; `GAME-09` depends on Accepted `CONTRIB-01` + `GAME-01`. |
| Mute during sync | `SET-SOUND` (Domain 02) | Behavior owners `PROV-03` / `READ-02`. |

## PROV / READ / PUB / CONTRIB authority split

| Item | Matrix-owned | Plan-owned |
| --- | --- | --- |
| `PROV-01` Yike preview and one-shot import | Status Partial. Repository: normalized fetch/import plumbing. Live: **provider-live** public Yike smoke (pending). Remaining gap / acceptance: non-mutating preview; fetch/parse then one `SGF-07` import; unite rooms; public `recommend`/`local`; Yike 10s; ≤3 idempotent-read retries; no provider secret; Provider Network Policy. **Depends on: `SGF-07`, `PREF-01`, `APP-03`.** Sidecar-live and credential evidence are not this item. | R10 Delivery Order 1 with `READ-01`. R10 exit member. Phase Gate `SGF-07`, `PREF-01`, `APP-03`. |
| `PROV-02` Fox preview and one-shot import | Status Partial. Live: **provider-live** Fox smoke (pending). Acceptance: nickname/UID/`chessid`, pagination, bounded recents, 20s/25s, 3 retries, stale ignore, one `SGF-07` import, not live sync. Depends on `SGF-07`, `PREF-01`, `APP-03`. | R10 Delivery Order 2 with `PROV-04`. |
| `PROV-03` Yike ongoing synchronization | Status Missing. Live: **provider-live** public-room and Play & Sync smoke (pending). Acceptance: one dirty confirmation at Start; no per-refresh prompts; cancel; stale ignored; last-good preserved; Stop leaves editable game; system-browser Play & Sync without reading browser auth; Yike 10s; ≤3 transient-read retries; writes never auto-retry; Provider Network Policy. Depends on `PROV-01`, `SGF-07`, `APP-03`. | R10 Delivery Order 3 after `PROV-01`. |
| `PROV-04` Tencent kifu preview and import | Status Missing. Live: **provider-live** Tencent kifu smoke (pending). Acceptance: username/`chessId`, `lastCode` pagination, bounded recents, 20s/25s, 3 retries, one `SGF-07` import, not combined with Fox or live Tencent/huanle. Depends on `SGF-07`, `PREF-01`, `APP-03`. | R10 Delivery Order 2 with `PROV-02`. |
| `READ-01` readboard sidecar readiness | Status Partial. Live: **sidecar-live** smoke (pending). Acceptance: ready/incompatible/unavailable/timeout/restart. Depends on `APP-03`. Readiness does not imply a synchronized current game. | R10 Delivery Order 1 with `PROV-01`. |
| `READ-02` readboard ongoing synchronization | Status Partial. Live: **sidecar-live** target smoke (pending). Acceptance: one-way external-authoritative sync; first apply through `SGF-07`; disconnect; stop-to-editable; mute-during-sync; local sidecar excluded from Provider Network Policy. GMA is not this item. Depends on `READ-01`, `SGF-07`. | R10 Delivery Order 4 after `READ-01` and `SGF-07`. |
| `READ-03` Explicit OCR limitation | Status **Accepted**. Do not mutate ID, scope, status, evidence, remaining gap, or acceptance. | R10 OCR-unsupported foundation. Not a live-OCR exit. |
| `PROV-05` Tencent/huanle live synchronization | Status Deferred. Live: not required until admitted (**provider-live** when admitted). Shared timeout/retry/recovery/privacy. Depends on `SGF-07`. | Unnumbered Deferred. Promotion Gate: —; `PROV-04` is an exclusion, not a gate. |
| `PROV-06` Yike Personal-category Discovery | Status Deferred. Live: guest `official=4` unverified (**provider-live**). No persist category/page. Depends on `PROV-01`, `PROV-03`. | Deferred. Promotion Gate: `PROV-01`, `PROV-03`. |
| `PROV-07` Yike Authenticated Read and Play | Status Deferred. Live: **credential** + **provider-live** per admitted family when admitted. Secrets only in System Credential Store. Depends on `PROV-03`, `GAME-01`, `GAME-05`, `PREF-01`, `APP-03`. | Deferred. Promotion Gate matches Matrix Depends on. |
| `PUB-01` WebBoard LAN publishing | Status Deferred. Live: not required until admitted. Inbound LAN excluded from Provider Network Policy. No credentials. Trial internals excluded. Depends on `APP-03`. | Deferred. Promotion Gate: shutdown/teardown owner. |
| `CONTRIB-01` Contribution Service Integration | Status Deferred. **This ticket owns the Matrix contract.** Live: production `katagotraining.org` **provider-live** plus **credential-store** evidence when admitted. Contribution Network Policy ≠ Provider Network Policy. Depends on `ENG-02`, `PREF-01`, `APP-03`, `REL-05`. `GAME-09` records only its dependency. | Deferred. Promotion Gate: `ENG-02`, `PREF-01`, `APP-03`, `REL-05`. Not an R10 exit. |

Plan R10 Delivery Order (Plan only; not a second Matrix start-prerequisite set): `PROV-01`+`READ-01` → `PROV-02`+`PROV-04` → `PROV-03` → `READ-02`. `PROV-05`–`PROV-07`, `PUB-01`, `GAME-10`, and `CONTRIB-01` do not block R10 exit.

## Evidence classes (kept distinct)

| Class | Used for | Not implied by |
| --- | --- | --- |
| Repository | Parsers, URL families, `SGF-07` transitions, cancellation, stale ignore, timeout/retry classification, last-good, stop-to-editable, Provider Network Policy fixtures | Any live column |
| Provider-live | Public Yike categories/URL families, Play & Sync handoff, Fox/Tencent lookup, Deferred live Tencent/huanle, Personal guest semantics, authorized Yike, production `katagotraining.org` | Repository plumbing; sidecar-live; credential store |
| Sidecar-live | `READ-01` ready/timeout/restart; `READ-02` target sync/disconnect | Provider-live; `GAME-10` engine play-back |
| Credential | System Credential Store for Deferred `PROV-07` and `CONTRIB-01` only | Bounded non-secret recents; proxy secrets (never persisted) |

## Remainder and Accepted-field preservation

Every Domain 06 Capability maps to one supported Parity Item, an explicit split list, an owner-routed mapping, or an explicit exclusion. There is no external-workflow remainder. `CONTRIB-01` is owned here as a Parity Item, not an 11th Domain 06 Capability.

Original sixteen Accepted items versus Next `18c6d189b8b01069975c4c40ead63a010249cb8c` keep ID, observable scope, status, evidence, remaining gap, and acceptance: `BASE-01`, `BASE-02`, `SGF-01`–`SGF-06`, `RULE-01`, `UI-01`, `UI-03`, `UI-04`, `UI-05`, `ENG-01`, `ANA-05`, `READ-03`. This ticket does not expand `READ-03` or edit foreign `SGF-07` / `PREF-01` / `APP-03` / `APP-04` / `REL-03` / `REL-05` / `GAME-01`–`GAME-10` / `ENG-*` / `REVIEW-03` contracts.

Repository evidence is never written as provider-live, sidecar-live, or credential evidence.
