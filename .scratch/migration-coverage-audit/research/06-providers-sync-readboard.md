# 06 — Provider, Synchronization, readboard, and External Publishing Census

## Sources (authoritative only)

| Tree | Path | Commit |
| --- | --- | --- |
| Java Migration Baseline v1 | `/home/dev/dev/weiqi/worktrees/lizzieyzy-next/java-baseline-v1` | detached HEAD `7b4027531c2b26062d0bfc27a040cc550cfbea4d` (`.git/worktrees/java-baseline-v1/HEAD`) |
| Next | `/home/dev/dev/weiqi/worktrees/lizzieyzy-next-tauri/migration-coverage-audit` | `18c6d189b8b01069975c4c40ead63a010249cb8c` (`refs/heads/docs/migration-coverage-audit`) |

Read: `CONTEXT.md`, `docs/JAVA_BASELINE.md`, `docs/PARITY_MATRIX.md` R7 (`PROV-01`/`PROV-02`/`READ-01`–`READ-03`), `docs/MIGRATION_PLAN.md` R7, `docs/ARCHITECTURE_NEXT.md` provider/sidecar section.

Not used as evidence: `/home/dev/dev/weiqi/lizzieyzy-next`, `/mnt/d/dev/weiqi/tmp/lizzie-java-baseline-r2a`, GitHub compare, later commits, or first-round reports.

## Census method

Static source census of reachable Entry Points only (registered menu/toolbar/shortcut/dialog actions). Generation surfaces: `Menu.java` File / Sync / bottom-toolbar customizer / top basic-btn strip; `BottomToolbar.java` live + share popups; `Input.java` Q / O / E / B; `Config.java` defaults + uiConfig keys; Fox/Yike/Tencent/Online/ReadBoard/WebBoard/Share classes + focused tests; Next `ProviderPanel` / `AppChrome` sync sheet / provider crates / Tauri gateway.

Capability = one user goal. Multiple controls that invoke the same behavior are one Capability. Swing class/widget/thread/file format is not a Capability. Constructed-but-unregistered Share menubar and share-toolbar-toggle are not Entry Points. Secrets (Yike signing material, upload password values) are not quoted. Preview vs import vs ongoing sync is called out per Capability.

---

## Capability inventory

### CAP-06-ONLINE-URL — Paste an online room/live URL and synchronize it into the current game

- **Entry Points:** File → Open online link (`Menu.openUrl`); shortcut Q (`Input` VK_Q without Ctrl/Alt); Yike Live Center Sync/double-click/Enter (`YikeLiveDialog.syncSelectedGame` → `Lizzie.frame.syncOnline`).
- **Modes:** `YikeUrlParser` kinds: new live room, old live room, old live board, game/hall room, **unite room**. OnlineDialog also contains compiled Tencent/huanle websocket and qipu URL bytes (b/b2/b3/b4/c1) for non-Yike live streams.
- **Preview / import / sync:** Ongoing synchronization into the authoritative current board (`urlSgf` / history apply). Not a one-shot preview. Yike session promotion requires both sync-ready and geometry-ready (`OnlineDialogYikeSessionStateTest`).
- **Defaults:** Refresh interval field `txtRefreshTime` exists; numeric default 需定向运行核验. `alwaysGotoLastOnLive` default false; `openHtmlOnLive` default true.
- **Persistence:** `always-gotolast-onlive`, `open-html-onlive` in uiConfig. No URL credential store observed for this dialog.
- **Failure / recovery:** Network via AjaxHttpRequest / NetworkProxy / websocket (io.socket + Java-WebSocket). Yike waiting-page URLs invalidate placement geometry without clearing display session. Session switch is deferred until pending session is fully ready. Timeout/reconnect numeric policy 需定向运行核验.
- **Frozen evidence:** Menu.java 212–222; Input.java 701–707; OnlineDialog.java 86–204, 137–156; YikeUrlParser.java 8–129; YikeUrlInfo.java 1–37; YikeLiveDialog.java 284–293; OnlineDialogYikeSessionStateTest.java.
- **Next mapping:** File menu item 打开在线链接(Q) is visible-disabled title=尚未接入. Sync sheet can one-shot fetch+import a Yike URL after Preview, not periodic live sync. Unite-room URLs are not in Next parseYikeUrl (old_live_room / old_live_board / game_room / new_live_room only).
- **Parity Item:** Partial PROV-01 (Yike fetch/import only). No parity item for ongoing URL sync, Tencent websocket live, or unite-room.
- **Ambiguity:** Exact refresh-seconds default, Tencent/huanle URL acceptance at runtime, reconnect backoff: 需定向运行核验.

### CAP-06-YIKE-LIVE-CENTER — Browse Yike live games, then start sync

- **Entry Points:** Sync → 弈客直播 (Menu.yikeLive, Shift+O); BottomToolbar 直播 popup → 弈客直播; Input Shift+O.
- **Modes:** Categories recommend "1", local "0", personal "4". Pagination page/since. Client filter regex.
- **Preview / import / sync:** List is browse/preview. Sync/double-click/Enter starts ongoing sync via CAP-06-ONLINE-URL (`syncOnline(game.toRoomUrl())`). Web button opens CAP-06-YIKE-WEB.
- **Defaults:** Dialog 920×420; first category Recommend; HTTP timeout 10_000 ms (YikeApiClient.HTTP_TIMEOUT_MS). Live list Status must be 1200.
- **Persistence:** None for last category/page.
- **Failure / recovery:** loadFailed status with exception message; buttons disabled while loading. Signed GET with compiled-in AppKey/signing headers (values not listed). No retry loop in fetchLiveList.
- **Frozen evidence:** Menu.java 4134–1444; Input.java 532–538; BottomToolbar.java 1050–1059; YikeLiveDialog.java 41–163, 231–293; YikeApiClient.java 26–76.
- **Next mapping:** Chrome 弈客直播(Shift+O) disabled 尚未接入. provider-yike has fetch_live_list / fetch_live_detail_import / same 10s timeout, not wired to a list UI. Provider panel is URL-in, not category browser.
- **Parity Item:** PROV-01 Partial covers fetch/import of a known id, not the live-center workflow.
- **Ambiguity:** Personal category auth vs anonymous: 需定向运行核验.

### CAP-06-YIKE-WEB — Open Yike in the embedded browser (optional auto-sync)

- **Entry Points:** Sync → 打开弈客网页版; Live Center Web; BottomToolbar same item → openYikeLiveWeb.
- **Modes:** BrowserFrame(startURL, title, yike=true). Observes page URL; can issue yikeBrowserSyncStop; observed-URL freshness 30s.
- **Preview / import / sync:** Browser is navigation. If openHtmlOnLive, live/hall entry may open HTML. Auto-sync into current game is a sidecar of this browser, not a separate import.
- **Defaults:** JCEF release tag and jcef-bundle directory required. openHtmlOnLive default true.
- **Persistence:** open-html-onlive.
- **Failure / recovery:** BrowserFrame.startFailedTitle/Message; Retry or Open in system browser. JCEF init exceptions.
- **Frozen evidence:** Menu.java 4146–1455, 4273–4283; BottomToolbar.java 1061–1068; BrowserFrame.java 42–80; DisplayStrings.properties BrowserFrame.*.
- **Next mapping:** Chrome item disabled. No JCEF/browser crate.
- **Parity Item:** None.
- **Environment:** Native JCEF bundle; network to yikeweiqi.com. 需定向运行核验 auto-sync from browser vs OnlineDialog.

### CAP-06-YIKE-HALL — Open Yike hall in the embedded browser

- **Entry Points:** Sync → 弈客大厅; BottomToolbar 弈客大厅 → bowser("https://home.yikeweiqi.com/#/game", …, true).
- **Modes:** Hall URL; room ids parsed as game_room when later pasted into OnlineDialog (room= + hall).
- **Preview / import / sync:** Navigation only until the user syncs a room URL.
- **Defaults:** Fixed hall URL.
- **Persistence:** N/A.
- **Failure / recovery:** Same JCEF failure path as CAP-06-YIKE-WEB.
- **Frozen evidence:** Menu.java 4157–4168; BottomToolbar.java 1071–1081; YikeApiClient.YIKE_GAME_URL.
- **Next mapping:** Chrome 弈客大厅 disabled.
- **Parity Item:** None.

### CAP-06-FOX-KIFU — Search Fox games by nickname/UID and import a selected kifu

- **Entry Points:** Sync → 野狐(腾讯)棋谱; BottomToolbar 直播 popup → same; top basic-btn btnFoxKifu when showBasicBtn (default true).
- **Modes:** user_name <nick>; numeric input treated as UID; uid <id> [last_code] pagination (lastcode, 25/page UI, API pages ~100); chessid <id> SGF fetch. CGI POST then H5 GET fallback.
- **Preview / import / sync:** Search list is preview. Open button imports SGF into the current game (loadFoxKifu → chessid). Not live sync. After-open: foxAfterGet 0 minimize / 1 close / 2 none.
- **Defaults:** Connect 20s, read 25s, 3 retries with 350ms×attempt sleep. Mobile UA for H5; okhttp/3.12.12 for CGI. foxAfterGet default 0. lastFoxName default empty.
- **Persistence:** last-fox-name, fox-after-get, fox-recent-searches (up to 8 uid/nickname pairs, no passwords). Help → clear personal data removes fox-recent-searches.
- **Failure / recovery:** Empty list message; result!=0 / errcode empty payload (isFoxPayloadWithoutContent); nickname not found / empty UID RuntimeException surfaced as error emit; CGI failures fall through to next endpoint then H5; stale GetFoxRequest results ignored.
- **Frozen evidence:** GetFoxRequest.java 27–40, 67–177, 218–266; FoxKifuDownload.java 54–187, 470–489, 621–641, 849–914, 1084–1106; tests GetFoxRequestTest, FoxKifuDownloadResponsePayloadTest, FoxKifuDownloadPaginationCursorTest.
- **Next mapping:** Sync sheet Fox command box: numeric → chessid; supports chessid/uid/user_name. Fetch+import replaces current game after dirty confirm (handleProviderImport → applyReplacement). UI timeout 15s vs Java 20/25s and vs Tauri transport default 30s. No search list, pagination, recent searches, after-get, or Fox SGF normalize of consecutive-B handicap in the panel (Rust normalize_fox_sgf exists in crate). Direct http(s) Fox URLs rejected by the panel.
- **Parity Item:** PROV-02 Partial. Remaining: live Fox, list UI, pagination, recent searches, after-get, handicap normalize parity.
- **Environment:** Network to foxwq.com / huanle.qq.com. No Fox login cookie observed (srcuid=0).

### CAP-06-TENCENT-KIFU — Search Tencent games and import a selected kifu

- **Entry Points:** Sync → Tencent kifu (Menu.tencentKifu → openTencentKifu); top basic-btn btnTencentKifu. Not in BottomToolbar live popup.
- **Modes:** Username search; lastCode pagination; chessId SGF fetch. Session token default lizzieyzy-next. Fetch num 100; UI 25/tab.
- **Preview / import / sync:** List preview; open imports into current game. Not live sync.
- **Defaults:** Connect 20s, read 25s, 3 retries. Desktop Chrome UA.
- **Persistence:** Recent Tencent searches in dialog (same pattern as Fox). Exact uiConfig key 需定向运行核验 (dialog field recentSearches; Fox uses fox-recent-searches).
- **Failure / recovery:** Same retry/timeout class as Fox; emitError on exception.
- **Frozen evidence:** Menu.java 4182–4190, 7901–7916; GetTencentRequest.java 17–70; TencentKifuDownload.java 53–80.
- **Next mapping:** Chrome 腾讯棋谱 opens the generic sync sheet (Fox/Yike only). No Tencent provider crate or command.
- **Parity Item:** None.

### CAP-06-READBOARD — Synchronize an external client board via local sidecar

- **Entry Points:** Sync → 棋盘识别工具 only if OS.isWindows(), accelerator Alt+O; BottomToolbar 棋盘同步 Windows only; Input Alt+O is not OS-gated.
- **Modes:** Launch readboard.exe (pipe) or readboard.bat (socket fallback). use-pipeline-readboard default false. Protocol commands include yikeSyncStart/yikeSyncStop, readboardUpdateSupported, GMA (gma; 该模式仅支持 KataGo). Settings: always keep board identical; wheel-focus.
- **Preview / import / sync:** Ongoing sync of the external position into the current board (not a one-shot SGF import). GMA can place local engine moves back to the target (engine play → domain 04/05). Image OCR is not this path (Capture Tsumego is 03).
- **Defaults:** alwaysSyncBoardStat true; readBoardGetFocus true; notPlaySoundInSync true; usePipeReadBoard false; process exit wait 1000ms; destroy 200ms; pending local-move ACK 3000ms; sync analysis resume delay 200ms.
- **Persistence:** always-sync-boardstat, read-board-get-focus, use-pipeline-readboard, suppress-readboard-websocket-pondering-notice.
- **Failure / recovery:** Missing exe/bat; place failed error place failed; GMA unsupported without KataGo; sidecar update installer; logging handshake. Shutdown waits.
- **Frozen evidence:** Menu.java 4193–4324; BottomToolbar.java 1094–1102; Input.java 532–536; ReadBoard.java 43–73, 81–131; Config.java 101, 1636, 1743, 2001; tests ReadBoardLaunchPathTest, ReadBoardYikeSyncControlTest, ReadBoardGmaSessionContractTest.
- **Next mapping:** Probe + protocol-line snapshot preview only (readboard_sidecar_probe / readboard_sidecar_sync_snapshot). Default socket 127.0.0.1:39081; jar name readboard-1.6.2-shaded.jar; UI timeout 5s. Preview does not replace_current_game. Image OCR returns structured unsupported (READ-03 Accepted).
- **Parity Item:** READ-01 Partial, READ-02 Partial (preview ≠ import), READ-03 Accepted.
- **Environment:** Windows native sidecar (or Next Java jar fallback). Live target client. Alt+O on non-Windows: 需定向运行核验.

### CAP-06-WEBBOARD — Publish the current board to a LAN web viewer (trial play)

- **Entry Points:** Sync → WebBoard submenu Start/Stop + Copy URL; BottomToolbar live popup Start/Stop + Force-exit trial.
- **Modes:** HTTP default port 9998 (try +0..9); WS 9999 (skip in-use); max connections 20; bind 0.0.0.0. Access URL http://<LAN IPv4>:<httpPort>. Trial session: one owner, 5 min idle, dummy mainline node, engine-busy / in-use denial.
- **Preview / import / sync:** Outbound publish of current board + optional remote trial that overrides display node. Does not import an external provider game.
- **Defaults:** web-board.http-port 9998, ws-port 9999, max-connections 20. Idle 5 min.
- **Persistence:** Optional web-board object in config (not uiConfig keys observed as required).
- **Failure / recovery:** Start returns false if no HTTP/WS port; WS fail stops HTTP; trial denied in_use / engine_busy; copy URL no-ops if not running.
- **Frozen evidence:** Menu.java 4208–4269; BottomToolbar.java 1105–1168; WebBoardManager.java 24–227; WebBoardServer.java 10–73; WebBoardHttpServer.java 16–70; WebBoardManagerTest.java.
- **Next mapping:** None.
- **Parity Item:** None.
- **Environment:** LAN firewall; browser clients. Engine-busy interaction → 04.

### CAP-06-SHARE-CURRENT — Registered share shortcuts with no working action

- **Entry Points:** `Ctrl+E` and `Alt+B` are registered by `Input` and `InputIndependentMainBoard` and call `shareSGF()`.
- **Frozen behavior:** `LizzieFrame.shareSGF()` has a fully commented body, so both shortcuts are observable no-ops: they do not open `ShareFrame`, upload a game, mutate the current game, or report an error. `batchShareSGF()` is likewise fully commented.
- **Defaults:** N/A.
- **Persistence:** No observable state is written by the registered no-op. Legacy upload fields/classes are unreachable implementation candidates, not evidence of a working workflow.
- **Failure / recovery:** No feedback and no recovery action; the registered action simply returns.
- **Frozen evidence:** `Input.java:317-319,663-666`; `InputIndependentMainBoard.java` registered share paths; `LizzieFrame.java:11570-11578` commented `shareSGF()` and `batchShareSGF()` bodies.
- **Next mapping:** No share command or matching dead shortcut.
- **Parity Item:** None. This is a reachable dead Entry Point, not a working external-publishing Capability.
- **Ambiguity:** N/A from frozen source; disposition belongs to the external-domain decision ticket.

### CAP-06-SYNC-SETTINGS — Live/readboard preference toggles

- **Entry Points:** Sync menu checkboxes: allow Yike live/hall to open room HTML; always jump to last move on a new live move; readboard always-sync board; readboard get-focus.
- **Frozen behavior:** Each toggle changes the corresponding live/readboard behavior without creating a separate sync session.
- **Defaults:** `openHtmlOnLive=true`; `alwaysGotoLastOnLive=false`; `alwaysSyncBoardStat=true`; `readBoardGetFocus=true`.
- **Persistence:** `open-html-onlive`, `always-gotolast-onlive`, `always-sync-boardstat`, and `read-board-get-focus` in `uiConfig`.
- **Failure / recovery:** No independent setting-error channel is visible. Runtime unavailability and protocol failures remain owned by the Yike/readboard capabilities above; persistence-write failure presentation is **需定向运行核验**.
- **Frozen evidence:** `Menu.java:4146-4324`; `Config.java:101,1636,1743,2001`; the corresponding Yike and readboard capability evidence above shows each flag's use site.
- **Next mapping:** No equivalent settings exist.
- **Parity Item:** `PREF-01` currently aggregates the setting inventory; behavior acceptance belongs with the relevant R7 successor items.
- **Ambiguity / runtime check:** Verify whether toggle persistence failure is surfaced and whether each change applies immediately to an already-running session.

---

## Unreachable / implementation-only candidates (not abandoned)

| Candidate | Why it is not an Entry Point | Notes |
| --- | --- | --- |
| Menubar shareKifu | this.add(shareKifu) commented out | Same actions as keyboard share / hidden toolbar |
| Batch/private/public share actions | Only exposed by the unreachable menubar Share items or the hidden share-toolbar popup | No reachable shortcut or registered dynamic Entry Point in the frozen baseline. |
| Custom-toolbar share checkbox | customToolbarItem.add(shareButton) commented out | Cannot turn share button on from that menu |
| assets/foxReq/FoxRequest.jar | GetFoxRequest talks HTTP directly | Leftover binary; not a user control |
| SyncDiagnosticsDialog | No menu/toolbar/shortcut registration found in scanned constructors | May be opened from a sync internals path; 需定向运行核验. Diagnostics, not a user workflow. |
| Capture Tsumego | Screenshot region → tsumego frame | Domain 03, not provider sync |
| Remote compute / SSH engine | Settings → 远程算力 | Domain 04 |
| File Save / clipboard SGF | Local file/clipboard | Domain 03 |

---

## Cross-domain dependencies and unowned entries

| Surface | Owner if not 06 |
| --- | --- |
| File → Open online link, Sync menu, live toolbar, Shift+O / Alt+O / Q, and dead Ctrl+E / Alt+B share shortcuts | 01 indexes routing only; 06 owns semantics and records the share shortcuts as no-ops |
| Imported SGF tree/board after Fox/Yike/Tencent/Online apply | 03 owns subsequent review/edit/save |
| ReadBoard GMA / WebBoard trial vs engine busy / analysis resume | 04 engine identity; 05 if it becomes a match session |
| openHtmlOnLive / jump-to-last / always-sync-board | Settings persistence 02; behavior 06 |
| Contribute training | 04 engine + 07 if packaged katago-contribute |
| Help → check update / clear personal data (clears fox-recent/share-history) | 07 / 02; 06 only notes the cleared keys |
| Next File 打开在线链接(Q) disabled row | 01 shell inventory; 06 records no workflow behind it |

---

## Next plumbing vs Java user workflow

| Java user workflow | Next at 18c6d189 | Gap |
| --- | --- | --- |
| Yike live center → ongoing sync | Signed list/detail in provider-yike; UI is URL Preview + Fetch&import | No list UI; no periodic/websocket sync; no unite-room parse |
| Paste URL Q live sync | Disabled menu; one-shot import if user uses sync sheet | No OnlineDialog |
| JCEF Yike web/hall | Disabled chrome items | No browser |
| Fox search list + import | Command-box fetch+import to current game | No list/pagination/recents/after-get; timeout mismatch 15s vs 20/25s |
| Tencent kifu | Chrome item opens Fox/Yike sheet | No Tencent path |
| ReadBoard Windows sidecar ongoing sync | Probe + paste protocol line preview | No launch/session; no import into current game (READ-02) |
| WebBoard LAN publish + trial | None | Missing |
| Registered `Ctrl+E` / `Alt+B` share shortcuts | None | Frozen Java action is also a no-op; no working upload parity claim exists |
| Sync settings | None | Missing |

Parity (from PARITY_MATRIX.md, unchanged by this census):

- PROV-01 Partial — Yike fetch/import wired; live Yike unvalidated
- PROV-02 Partial — Fox chessid/uid/user_name wired; live Fox unvalidated
- READ-01 Partial — sidecar probe typed; live sidecar unvalidated
- READ-02 Partial — snapshot preview, not current-game import
- READ-03 Accepted — OCR explicitly unsupported

No disposition decisions.

---

## Corrections vs invalid first-round report

- Java evidence is only java-baseline-v1@7b402753. This report does not cite 42c92e3, /home/dev/dev/weiqi/lizzieyzy-next working tree, or /mnt/d/dev/weiqi/tmp/lizzie-java-baseline-r2a.
- Share menubar, share-toolbar-toggle, and BottomToolbar share popup are not Entry Points (`this.add(shareKifu)` and `customToolbarItem.add(shareButton)` are commented; `Config.share` defaults false and its load is commented).
- Registered `Ctrl+E` / `Alt+B` call a commented `shareSGF()` body and therefore do not provide a working publish workflow; public/batch/private share actions remain unreachable.
- Tencent is a separate reachable Menu/top-toolbar Capability, not folded into Fox.
- Unite-room is a frozen Yike URL mode; Next parser omits it.
- WebBoard is a reachable LAN publish Capability, not a Fox/Yike provider.
- ReadBoard menu/toolbar registration is Windows-only; Alt+O is global in Input.
- Contribute is reachable (`Menu.java:5579`) but is owned by the match/engine/release domains (05/04/07), not 06.
- Next Provider Fetch&import does replace the current game (import), but readboard sync is preview-only; Java Online/Yike/ReadBoard are ongoing sync, not one-shot import.
