# Domain 07 — Install, Update, Packaging, Release, and Platform Census

## Sources

| Tree | Path | Commit |
| --- | --- | --- |
| Java (Migration Baseline v1) | `/home/dev/dev/weiqi/worktrees/lizzieyzy-next/java-baseline-v1` | `7b4027531c2b26062d0bfc27a040cc550cfbea4d` (detached HEAD, parent-verified) |
| Next | `/home/dev/dev/weiqi/worktrees/lizzieyzy-next-tauri/migration-coverage-audit` | `18c6d189b8b01069975c4c40ead63a010249cb8c` (`docs/migration-coverage-audit`) |

Docs read: `CONTEXT.md`, `docs/JAVA_BASELINE.md`, `docs/PARITY_MATRIX.md` (R8 REL-01–05), `docs/MIGRATION_PLAN.md` §R8, `docs/ARCHITECTURE_NEXT.md`.

This census does **not** cite `42c92e3`, `/mnt/d/dev/weiqi/tmp/lizzie-java-baseline-r2a`, or `/home/dev/dev/weiqi/lizzieyzy-next` source. Old reports were treated as candidates only.

## Method

Static source census of user-reachable install/first-launch/update/about/diagnostics/OS-integration surfaces. Capability = observable user goal; multiple controls that do the same thing are one Capability.

**Java entry-generation surfaces covered**

- Help menu construction: `src/main/java/featurecat/lizzie/gui/Menu.java` (`helpMenu` at 5164–2554)
- Startup: `Lizzie.main` / `completeAutomaticFirstRunSetup` / `start`
- Work-dir / portable: `WorkDirectoryResolver`, `Config`
- Bundled engine discovery: `KataGoAutoSetupHelper.inspectLocalSetup` / `inspectLocalKataGo` (called from first-run)
- Update stack: `featurecat.lizzie.update.*`
- Diagnostics: `DiagnosticsDialog`, `DiagnosticBundleExporter`, `CrashHandlers`, `LoggingSettings`
- About: `ConfigDialog2` about tab (`openConfigDialog2(2)`)
- User docs: `docs/INSTALL.md`, `docs/PACKAGES.md`, `docs/TROUBLESHOOTING.md`, `SUPPORT.md`
- Packaging scripts/workflows treated as **maintainer-only** unless they define an end-user install artifact the user actually downloads

**Next surfaces covered**

- Help chrome: `apps/desktop/src/components/AppChrome.tsx`
- About handler: `apps/desktop/src/App.tsx`
- Tauri bundle: `apps/desktop/src-tauri/tauri.conf.json`, `Cargo.toml`, capabilities
- Engine asset checks: `crates/engine-manager`
- Health DTO: `crates/app-model` `AppHealthDto`
- Release workflows/docs: evidence for REL-01, **not** user Capabilities

No validation commands, builds, or live updater/installer smokes were run. Runtime-only claims are marked **需定向运行核验**.

Domain ownership: 01 shell routing, 02 settings/layout, 03 SGF/board, 04 engine/analysis, 05 match sessions, 06 providers/readboard, **07 this census**. Cross-refs do not duplicate those inventories.

---

## End-user Capabilities

### REL-C01 — Install a platform package and launch the app

| Field | Value |
| --- | --- |
| Name | Install / unpack a release package and start the product |
| Entry Points | Download from [goagent.top/download](https://goagent.top/download/) or GitHub Releases; Windows portable unzip + double-click flavor EXE; Windows `installer.exe` wizard then Start Menu / desktop shortcut; macOS DMG drag to Applications then launch from Applications; Linux unzip + `./start-linux64.sh` |
| Frozen baseline behavior | User-facing package matrix is documented, not inferred from widgets. Windows prefers portable zip; installer is optional. Flavor EXEs: `LizzieYzy Next NVIDIA.exe`, `LizzieYzy Next OpenCL.exe`, `LizzieYzy Next.exe` (CPU / without-engine). macOS: do not run from the mounted DMG. Linux: chmod +x then `start-linux64.sh`. |
| Defaults | Windows ordinary users: `.portable.zip`. NVIDIA RTX 20/30/40/50: `windows64.nvidia.*`. AMD/Intel/older NVIDIA: `opencl`. No GPU / GPU fail: `with-katago` CPU. macOS by chip. Linux `linux64.with-katago.zip`. |
| Persistence | Portable Windows writes config/logs/saves under package `user-data/` when `.lizzie-portable` is present. Installed Windows uses shared work dir `LizzieYzyNext` / fallback (see REL-C02). |
| Failure / recovery | Wrong package / incomplete unzip / Gatekeeper: `docs/TROUBLESHOOTING.md`. Linux: launch from terminal to see errors. |
| Platform | Windows x64 portable+installer (nvidia/opencl/with-katago/without.engine); macOS Apple Silicon + Intel DMG; Linux x64 zip (cpu/opencl/nvidia). Experimental Windows DirectML/OpenVINO/ROCm portable only. |
| Frozen evidence | `docs/INSTALL.md` L1–228; `docs/PACKAGES.md` L1–150; `docs/TROUBLESHOOTING.md` L1–37; `scripts/package_windows_exe.sh` L29–34, L13–14 (flavor names, upgrade UUID); `scripts/package_macos_dmg.sh` L1–80 |
| Next mapping | Repository packaging exists: `tauri.conf.json` bundle `targets: all`, identifier `org.lizzieyzy.next`, product `LizzieYzy Next`, binary `lizzieyzy-next-desktop`, version `0.1.0`. Workflow `.github/workflows/release.yml` builds unsigned CI bundles (macOS dmg, Windows exe/msi, Linux AppImage/deb/rpm) and publishes a **prerelease** GitHub Release. **No live install/launch/uninstall evidence.** |
| Next evidence | `apps/desktop/src-tauri/tauri.conf.json` L1–69; `.github/workflows/release.yml` L1–150; `.github/RELEASE_NOTES_v0.1.0.md`; `docs/RELEASE_CHECKLIST.md` Packaging Checklist; `docs/PARITY_MATRIX.md` REL-04 Missing |
| Existing Parity Item | **REL-04** Missing — platform installer smoke pending |
| Ambiguity | Whether a given CI artifact is actually installable on a clean machine is live evidence, not repository evidence. Java 13-asset matrix vs Next Tauri `targets: all` is a product-shape difference, not a defect. |

### REL-C02 — First launch work-directory / portable vs installed layout

| Field | Value |
| --- | --- |
| Name | Resolve a writable work directory on first launch (portable folder vs installed profile) |
| Entry Points | Process start (`Lizzie.main` → `bootstrapLogging` → `WorkDirectoryResolver.resolve`) |
| Frozen baseline behavior | Explicit `lizzie.work.dir` override if set. On Windows, if a portable package root with `.lizzie-portable` is found, work dir is package `user-data/` (with v2 state migration key `migrated-windows-portable-user-state-v2` and recovery backup `config.before-portable-recovery.txt`). Else Windows falls back to writable shared dir `LizzieYzyNext`. Non-Windows: writable cwd, else `~/.lizzieyzy-next` (legacy `~/.lizzieyzy-next-foxuid`). If cwd is not writable, `user.dir` is switched to the resolved directory. |
| Defaults | Portable if marker present; otherwise platform fallback. |
| Persistence | Work dir holds config, logs, saves, update staging. Portable credentials also under that tree. |
| Failure / recovery | Resolver records `WorkDirectoryDiagnostic`s; portable recovery backup name is defined. If first-run profile save fails, engine chip shows repair (`EngineStartup.profileSaveFailed`). |
| Frozen evidence | `WorkDirectoryResolver.java` L28–180, L32–36, L114–180; `Lizzie.java` L334–336, L381–386, L462–476, L894–964 |
| Next mapping | Tauri app-data owns `lizzieyzy-next-engine-profile.json`, `lizzieyzy-next-app-preferences.json`, `analysis-cache.sqlite3`. No portable-marker / `user-data/` beside the binary. No Java work-dir migration. |
| Next evidence | `apps/desktop/src-tauri/src/lib.rs` L36–38; `docs/ARCHITECTURE_NEXT.md` Persistence |
| Existing Parity Item | **REL-04** (installed layout) + **PREF-01** (02 owns preference files). No dedicated portable-mode item. |
| Ambiguity | Exact Windows shared-dir path (`%LOCALAPPDATA%` vs `%APPDATA%`) is in `resolveWindowsWorkDir` not fully read here — **需定向运行核验**. Interaction of `isNewProfile()` vs `firstTimeLoad` when only one is true: **需定向运行核验**. |

### REL-C03 — First-launch bundled engine auto-setup

| Field | Value |
| --- | --- |
| Name | On first run (or host change), detect bundled KataGo + configs + weight and write a usable engine profile without a setup wizard |
| Entry Points | Startup when `config.firstTimeLoad` or machine hostname change; not a Help-menu action. Interactive `KataGo 一键设置` is **04** (cross-ref). |
| Frozen baseline behavior | `completeAutomaticFirstRunSetup`: if no usable engine config and not `autoload-empty`, call `KataGoAutoSetupHelper.inspectLocalSetup()`; if engine+configs+weight exist, `applyAutoSetup`. Then `first-time-load=false` is persisted. Failure to save profile → repair chip. Still no complete bundled setup → `EngineStartup.noBundledEngine`. Host change deletes persist (except as coded) and re-runs first-run setup. Hostname lookup is local-only with timeout so DNS/LAN prompt cannot block launch or fake a machine change. |
| Defaults | Bundled `with-katago` / nvidia / opencl packages expected to auto-write default engine. `without.engine` stays unconfigured. |
| Persistence | Engine list + `first-time-load` in UI config. |
| Failure / recovery | Auto-setup exception is logged; user can open **04** one-click setup or engine dialog. Docs: confirm `weights/default.bin.gz` and `engines/katago/` still exist, restart. |
| Frozen evidence | `Lizzie.java` L275–336, L381–428, L758–773, L894–964, L522–615; `docs/INSTALL.md` L187–196; `docs/TROUBLESHOOTING.md` L39–54; `KataGoAutoSetupHelper.java` `SetupSnapshot` / `LocalKataGoDiscoveryResult` L270–406 |
| Next mapping | None. Engine profiles are user-entered paths + `check_assets`. No first-run bundled discovery. App first paint loads a demo/sample SGF (`App.tsx` useEffect). |
| Next evidence | `crates/engine-manager/src/lib.rs` `AssetCheck` / missing-path errors L81–93; `App.tsx` L120–134 |
| Existing Parity Item | **REL-05** Partial (bundled runtime assets) + **ENG-01** (04 profiles). First-launch auto-profile is **not** an accepted Next item. |
| Ambiguity | NVIDIA first-launch CUDA lib staging and driver-probe UX (`BundledEngineStartup.*` strings, INSTALL NVIDIA notes) is tightly coupled to **04** engine start. Treat GPU runtime prepare as part of REL-C04; live timing is **需定向运行核验**. |

### REL-C04 — Bundled runtime / flavor discovery

| Field | Value |
| --- | --- |
| Name | Discover which packaged flavor and bundled runtimes this install has (Java runtime, KataGo backend, weights, JCEF, readboard) |
| Entry Points | Implicit at update-check planning and first-run setup; flavor marker `lizzieyzy-next-engine-backend.txt`; installed manifest `lizzieyzy-next-installed-manifest.json` |
| Frozen baseline behavior | Windows flavor: installed manifest `flavor`, else backend marker (`cpu`→`with-katago`, `opencl`, `nvidia`, legacy `nvidia50-cuda`→`nvidia`), else presence of `engines/`. macOS/Linux package flavor from same markers / `containsKataGo`. Bundled Java: `appRoot/runtime/bin/java.exe` for updater helper. Update components include `core`, `katago-*`, `weight-default`, `readboard`, `jcef`, `java-runtime`. |
| Defaults | macOS updater flavor default `with-katago`. |
| Persistence | `lizzieyzy-next-installed-manifest.json` under `appDir` after a successful Windows apply. |
| Failure / recovery | Missing matching package → check result `NO_PACKAGE`. Missing bundled engine → REL-C03 repair chip. |
| Frozen evidence | `WindowsUpdatePaths.java` L20–178; `PlatformUpdateService.java` L92–231; `InstalledUpdateState.java` L14–43; `WindowsUpdateService.PathMapping` L245–262; `docs/PACKAGES.md` L83–145 |
| Next mapping | `engine-manager` checks **configured** profile paths only. `tauri.conf.json` has icons/bundle metadata, **no** bundled KataGo/JRE/readboard layout. REL-05 remaining gap is packaged-app resolution. |
| Next evidence | `engine-manager/src/lib.rs` L81–93; `PARITY_MATRIX.md` REL-05 Partial |
| Existing Parity Item | **REL-05** Partial |
| Ambiguity | Next has no equivalent flavor matrix. Do not treat Java nvidia/opencl/cpu packages as implied Next artifacts. |

### REL-C05 — Check for updates (channel + source)

| Field | Value |
| --- | --- |
| Name | Manually check whether a newer signed release exists for this install |
| Entry Points | Help → 检查更新 (`Menu.checkUpdate`) → `WindowsUpdateController.openCheckUpdatePage` → `CheckUpdateDialog`. Opening the page **does not** use the network; Check button starts discovery. |
| Frozen baseline behavior | Shows current `Lizzie.nextVersion`. Channel radio: 正式/`stable` (default) vs 测试/`beta`. Source radio (stable only): 官网 `https://download.goagent.top/channels/stable/update-envelope.json` vs GitHub latest envelope. Beta forces GitHub test pointer `.../releases/download/channel-beta/lizzieyzy-next-update-envelope.json` and requires signature + schema v2 + `prerelease=true`. Unpackaged/`next-dev` → cannot check. Discovery is single-flight. Newer tag than installed `next-YYYY-MM-DD.N` required. |
| Defaults | Channel `stable`; source `official`. Version if unset: `next-dev` (`-Dlizzie.next.version` / `LIZZIE_NEXT_VERSION`). |
| Persistence | `uiConfig["update-channel"]`, `uiConfig["update-source"]` via `Config.save()`. Changing channel does not change installed files. Beta does not persist a non-GitHub source. |
| Failure / recovery | Stay on page with localized result: unpackaged, unsupported platform, no update (stable/beta), no matching package, fetch failed (stable vs beta copy), invalid test pointer, generic check failed. Close blocked while check in flight. |
| Platform | Windows adapter if Windows runtime; else macOS/Linux package adapter; other OS → unsupported. |
| Frozen evidence | `Menu.java` L5164–5222; `CheckUpdateDialog.java` L28–273; `UpdateChannel.java` L8–54; `UpdateSource.java` L8–54; `UpdateDiscovery.java` L11–112; `UpdateManifestClient.java` L16–137; `UpdateVersion.java` L8–68; `UpdateCheckFeedback.java` L1–84; `TrustedUpdateKeys` resource `src/main/resources/update/trusted-update-keys.properties` (`stable-2026-08`) |
| Next mapping | Help → 检查更新 is **visible-disabled** `title=尚未接入`. No `tauri-plugin-updater` in `Cargo.toml`. Capabilities: `core`, `opener`, `dialog` only. |
| Next evidence | `AppChrome.tsx` L207–210; `Cargo.toml` L16–36; `capabilities/default.json`; `PARITY_MATRIX.md` REL-03 Missing |
| Existing Parity Item | **REL-03** Missing |
| Ambiguity | `UpdateVersion.shouldSkipAutomaticCheck` exists; no startup auto-check caller found on the Help/startup paths read. Treat in-app check as **manual-only** unless a later read finds a timer. **需定向运行核验** that the Help item is always enabled on unpackaged JAR (UI still opens; Check then shows unpackaged message). |

### REL-C06 — Download and apply a Windows in-place update

| Field | Value |
| --- | --- |
| Name | After a Windows offer, download selected components, quit, apply via helper, restart |
| Entry Points | Offer dialog from successful REL-C05 (`WindowsUpdateDialog`): 立即更新, 暂停/继续, 取消, 查看 Release, 关闭 |
| Frozen baseline behavior | Downloads to staging under work-dir `update/staging/<tag>/` with resume (`.part`), SHA-256 + size verify, R2 then GitHub fallback. Writes `update-request.json`, copies helper jar, launches `WindowsUpdateApplier` with bundled `runtime/bin/java.exe` if present. If not portable and app dir not writable → elevated PowerShell `RunAs`. Then `Lizzie.shutdown()`. Helper waits for main PID (≤30s), extracts zips, replace-core jar + launcher `.cfg`, replace-app-path for other components, writes installed manifest, restarts first `*.exe` in appRoot. User data / TensorRT / unchanged large assets are not re-downloaded (dialog copy). |
| Defaults | Selected components = those newer than installed manifest. Core-only is the small path; `core-update.zip` is the offline equivalent documented for already-installed portable trees. |
| Persistence | Staging files; installed manifest after success. |
| Failure / recovery | See REL-C08. |
| Frozen evidence | `WindowsUpdateDialog.java` L28–356; `WindowsUpdateService.java` L13–264; `WindowsUpdateApplier.java` L23–238; `ResumableDownloader.java` L22–38; `docs/PACKAGES.md` L18, L64 |
| Next mapping | Absent. |
| Existing Parity Item | **REL-03** Missing |
| Ambiguity | Helper restart command is “first exe in appRoot” — **需定向运行核验** which EXE is chosen when multiple flavor launchers exist. |

### REL-C07 — Download a macOS/Linux full package (no in-place replace)

| Field | Value |
| --- | --- |
| Name | After a non-Windows offer, download the matching signed package and open it / its folder |
| Entry Points | `PackageUpdateDialog` from REL-C05: 下载新版, pause/resume/cancel, View release |
| Frozen baseline behavior | macOS `open-dmg`: after verify, `Desktop.open(dmg)`. User must drag to Applications and launch from Applications. Linux: never overwrites current dir; opens containing Downloads (or `workDir/update/downloads`). Flavor matching via REL-C04. |
| Defaults | Download dir: `~/Downloads` if it is a directory, else work-dir `update/downloads`. Override `-Dlizzie.update.downloadDir`. |
| Persistence | Downloaded file remains; app install files unchanged until user replaces them. |
| Failure / recovery | Cancel keeps `.part` for resume. Network fallback R2→GitHub. Desktop unsupported → IOException. |
| Frozen evidence | `PackageUpdateDialog.java` L26–220; `PlatformUpdateService.java` L14–126, L53–65; `PackageUpdateAdapter.java` L1–64 |
| Next mapping | Absent as in-app flow. Users would download GitHub Release assets manually if a tag exists. |
| Existing Parity Item | **REL-03** Missing |

### REL-C08 — Update apply failure and rollback

| Field | Value |
| --- | --- |
| Name | Survive a failed Windows apply without leaving a half-replaced install |
| Entry Points | Automatic inside `WindowsUpdateApplier.apply`; download failures stay in REL-C06 dialog |
| Frozen baseline behavior | Before replace, current files are moved to `staging/backup-<timestamp>/`. On any apply exception, backups are restored in reverse. Zip-slip blocked. Apply waits ≤30s for main PID; timeout is failure → rollback. Download SHA mismatch / cancel does not start apply. Launch-helper failure shows message and does **not** shut down if exception is caught after failed launch (shutdown only after successful `launchHelper`). |
| Defaults | N/A |
| Persistence | Backup dir is under staging; not a user setting. |
| Failure / recovery | Rollback best-effort; rollback IOExceptions print to stderr. If main process never exits, apply fails. |
| Frozen evidence | `WindowsUpdateApplier.java` L39–82, L210–238, L266–283, L305–318; `WindowsUpdateDialog.java` L335–355 |
| Next mapping | `docs/RELEASE_PROCESS.md` rollback is **maintainer GitHub-release** policy (withdraw tag, publish patch), not an installed-app rollback UX. `tauri.conf.json` `windows.allowDowngrades: false` only if an updater existed. |
| Next evidence | `docs/RELEASE_PROCESS.md` L130–143; `tauri.conf.json` L49–50 |
| Existing Parity Item | **REL-03** (app updater rollback) vs maintainer rollback (not a user Capability) |
| Ambiguity | Whether a failed helper after the app has already quit leaves the user with a closed app + intact files is **需定向运行核验**. macOS/Linux have no apply rollback because they never overwrite. |

### REL-C09 — OS integration (shortcuts, dock, Gatekeeper, OS runtime)

| Field | Value |
| --- | --- |
| Name | Integrate the installed app with the host OS enough to launch and not be blocked |
| Entry Points | Installer Start Menu / desktop shortcuts (Windows installer packages); macOS Dock icon at launch; macOS Privacy & Security “Open Anyway”; Linux `.desktop` not found as a first-class user control |
| Frozen baseline behavior | macOS: `Lizzie.installApplicationIcon` sets Taskbar/Dock icon from `/assets/logo.png`. Official DMGs are documented as signed+notarized; unsigned/cache cases use System Settings → Privacy & Security → Open Anyway. Windows portable: no installer shortcuts; run EXE in unzip folder. Windows installer: wizard, install dir, Start Menu/desktop (INSTALL.md). CLI: `main(args[0])` loads that file if it is not `read`. |
| Defaults | N/A |
| Persistence | N/A |
| Failure / recovery | Gatekeeper/SmartScreen: TROUBLESHOOTING. |
| Frozen evidence | `Lizzie.java` L783–861, L1012–1017; `docs/INSTALL.md` L69–171; `docs/TROUBLESHOOTING.md` L11–37 |
| Next mapping | Tauri window title/size; Windows `webviewInstallMode.downloadBootstrapper` silent WebView2 bootstrap; macOS `hardenedRuntime: true`, `minimumSystemVersion: 10.13`. **No** `fileAssociations` in `tauri.conf.json`. Signing/notarization secrets listed in `RELEASE_PROCESS.md` but workflow builds `--no-sign`. |
| Next evidence | `tauri.conf.json` L13–69; `docs/RELEASE_PROCESS.md` L66–128; `PARITY_MATRIX.md` REL-02 Missing |
| Existing Parity Item | **REL-02** Missing (signing/notarization); **REL-04** Missing (installer smoke) |
| Ambiguity | No frozen `file-associations` / `.sgf` registration file was found under `packaging/` or `scripts/`. Double-click `.sgf` → app is **需定向运行核验** (CLI path load exists; OS association may be installer-only). Cross-ref **01/03** for the open-file behavior itself. |

### REL-C10 — Diagnostics, logs, and support bundle

| Field | Value |
| --- | --- |
| Name | Turn diagnostic logging on/off, export a sanitized zip, open the log folder |
| Entry Points | Help → Diagnostics and Logs → `DiagnosticsDialog.open` (modeless, singleton); Help → Stop Full Trace (enabled only while full-trace active); dialog: Apply, Export package, Cancel export, Open log folder; module checkboxes Engine/GTP/ReadBoard/Network; Full Logs + scopes |
| Frozen baseline behavior | Defaults: diagnostics **enabled**, all `DiagnosticModule`s, all `TraceScope`s (`LoggingSettings.defaults`). Apply persists via `config.saveLoggingSettings` under `uiConfig.logging`. Export writes a size-capped zip (app logs 24h/50MiB, crash 24h/10MiB, raw 50MiB) with sanitizer, then opens parent folder. Full Logs requires confirm. Crash path: `CrashHandlers.install` on bootstrap records uncaught exceptions to crash log. |
| Persistence | `logging.diagnostics-enabled`, `diagnostic-modules`, `preferred-full-trace-scopes` |
| Failure / recovery | Apply failure reverts UI from runtime and shows message. Export cancel via flag. |
| Frozen evidence | `Menu.java` L5170–5205; `DiagnosticsDialog.java` L56–134, L375–464; `LoggingSettings.java` L10–101; `DiagnosticBundleExporter.java` L55–64; `CrashHandlers.java` L16–80; `Lizzie.java` L462–506 |
| Next mapping | **No** Help diagnostics item, log export, or crash-bundle UI. Native errors surface as chrome `message` strings. |
| Existing Parity Item | None dedicated. Not REL-01–05. |
| Cross-ref | `SyncDiagnosticsDialog` (copy/export sync/Yike/readboard traces) is **06**. Help Diagnostics export **includes** a sync snapshot (`currentRequest` → `SyncDiagnosticsRecorder.exportSnapshot`) — do not inventory the sync dialog here. |
| Ambiguity | Exact on-disk log folder path per OS is resolver-dependent — **需定向运行核验**. |

### REL-C11 — Version / About

| Field | Value |
| --- | --- |
| Name | See product name, release tag, short intro, and project links |
| Entry Points | Help → About (`Menu.about`) → `Lizzie.frame.openConfigDialog2(2)` about tab. Same About content is also the settings modern nav “About”. Check-update page header also shows `Lizzie.nextVersion`. |
| Frozen baseline behavior | Hero: `LizzieYzy Next` + `versionLine` formatted with `Lizzie.nextVersion` + intro copy. Links: GitHub repo, Releases, Issues. Three marketing cards (kifu / KataGo / review). Maven artifact version `yzy2.5.3` / `lizzieVersion = "2.5.3"` is **not** the user-facing Next tag. User-facing tags are `next-YYYY-MM-DD.N`. |
| Defaults | `next-dev` when property/env unset. Packaged builds inject `LIZZIE_NEXT_VERSION` / `-Dlizzie.next.version` (`package_windows_exe.sh` L12). |
| Persistence | N/A |
| Failure / recovery | N/A |
| Frozen evidence | `Menu.java` L5208–5216; `ConfigDialog2.java` L146–162, L2331–2458; `Lizzie.java` L275–294, L754–773; `pom.xml` L9 |
| Next mapping | Help → About sets status text `LizzieYzy Next 0.1.0 · 桌面复盘工作区` (no dialog, no links, no commit/channel). Help → 简介 is disabled `尚未接入` (Java intro lives **inside** About, not a separate item). `getHealth()` / `AppHealthDto` is an internal API (`app`, `architecture`, `rust_backend_ready`, `notes`), not a user About surface. |
| Next evidence | `AppChrome.tsx` L207–210; `App.tsx` L1256; `app-model` `AppHealthDto` L208–213; `tauri.conf.json` `version: 0.1.0` |
| Existing Parity Item | None. Version policy is mentioned under REL-03 (“version policy are not accepted”). |

### REL-C12 — Support routing (docs / issues / QQ)

| Field | Value |
| --- | --- |
| Name | Find the right support channel for install/package/bug |
| Entry Points | About links (REL-C11); GitHub issue templates (`bug_report.yml`, `installation_report.yml`); `SUPPORT.md`; Discussions; QQ group `299419120` (docs only, not an in-app control) |
| Frozen baseline behavior | Docs tell users: package guide, INSTALL, TROUBLESHOOTING, TESTED_PLATFORMS, Installation Report vs Bug Report. |
| Defaults | N/A |
| Persistence | N/A |
| Frozen evidence | `SUPPORT.md`; `.github/ISSUE_TEMPLATE/*`; About link buttons |
| Next mapping | `.github/RELEASE_NOTES_v0.1.0.md` known limitations; no in-app support menu beyond disabled 简介. |
| Existing Parity Item | N/A (docs, not a matrix row) |

### REL-C13 — Clear listed personal recents from Help

| Field | Value |
| --- | --- |
| Name | Clear a fixed set of personal recents (Fox searches, recent files, batch-analysis history, share history) |
| Entry Points | Help → `Menu.clearAllPersonalData` with OK/Cancel confirm |
| Frozen baseline behavior | Removes four `uiConfig` keys only, then `config.save()`, then done dialog. Does **not** wipe engine profiles, work dir, or logs. |
| Persistence | Those keys deleted from UI config. |
| Failure / recovery | Save IOException is printed; no further recovery UI. |
| Frozen evidence | `Menu.java` L5225–2552 |
| Next mapping | Absent. |
| Existing Parity Item | Closer to **02 PREF-01** than REL-*. Listed here because the only Entry Point is Help. |
| Cross-ref | 02 should own whether Next wants a privacy-reset action. |

---

## Capability → Parity Item map

| Temp key | Existing item | Next status vs Java |
| --- | --- | --- |
| REL-C01 Install/launch | REL-04 | Missing live installer smoke; repo can emit unsigned bundles |
| REL-C02 Work dir / portable | REL-04 + PREF-01 | Tauri app-data only; no portable marker |
| REL-C03 First-launch auto-setup | REL-05 / ENG-01 | Missing |
| REL-C04 Bundled flavor/runtime | REL-05 | Partial: configured-path checks only |
| REL-C05 Check update / channel | REL-03 | Missing (disabled chrome) |
| REL-C06 Windows in-place apply | REL-03 | Missing |
| REL-C07 macOS/Linux package download | REL-03 | Missing |
| REL-C08 Apply rollback | REL-03 | Missing (docs describe **maintainer** rollback) |
| REL-C09 OS integration / signing | REL-02, REL-04 | Missing signing; WebView2 bootstrapper configured |
| REL-C10 Diagnostics bundle | (none) | Missing |
| REL-C11 About / version | REL-03 version policy | Partial chrome status string only |
| REL-C12 Support routing | (none) | Docs only |
| REL-C13 Clear recents | PREF-01 adjacent | Missing |

**REL-01** (release preflight/dry-run) is **maintainer evidence**, not an end-user Capability. It remains a valid Parity Item for R8 engineering gates.

---

## Unreachable or implementation-only (not abandoned)

Do not treat these as user Capabilities. Do not mark abandoned.

| Candidate | Why it is not a Capability |
| --- | --- |
| `UpdateCheckCoordinator`, adapters, `SignedUpdateEnvelope` | Implementation of REL-C05–C07 |
| `WindowsUpdateApplier.main` | Helper process of REL-C06 |
| `UpdateVersion.shouldSkipAutomaticCheck` | No user Entry Point found |
| `-Dlizzie.smoke.open*` probes | Maintainer/smoke hooks |
| `ReadBoardUpdateInstaller` / `readboardUpdateReady` | **06** sidecar self-update |
| `SyncDiagnosticsDialog` | **06** sync diagnostics UI |
| `FirstUseSettings` / Settings → 初始化设置 | **02** first-use preferences |
| Settings → KataGo 一键设置 | **04** interactive engine setup |
| JCEF / Yike embedded browser | **06** (JCEF zip **is** an update component in REL-C04/C06) |
| `AppHealthDto` / `health` command | Internal Next API, not About |
| Swing `CheckUpdateDialog` widgets | Entry Points listed above |

---

## Maintainer-only build / release mechanics

These are repository/live **evidence** for REL-01/02/04, not user Capabilities.

### Java (frozen)

- `scripts/package_windows_exe.sh`, `package_macos_dmg.sh`, `package_release.sh`, `prepare_bundled_*.sh/py`, `sign_macos_release*.sh`
- `scripts/r2_release.py`, `publish_release_request.py`, `generate_release_notes.py`, `validate_release_*.py/sh`
- Workflows: `build-*-release.yml`, `promote-stable-release.yml` (R2 + `UPDATE_SIGNING_PRIVATE_KEY` + `stable-2026-08`), `publish-test-channel-pointer.yml`, `publish-requested-pre-release.yml`
- `docs/RELEASE_CHECKLIST.md` (Java), `docs/R2_RELEASES.md`
- Windows installer upgrade UUIDs in `package_windows_exe.sh` L14, L62–63

### Next (current repo)

- `python3 scripts/validate_scaffold.py` / `validate_release_assets.py` / `validate_release_workflow.py` (REL-01 Partial)
- `.github/workflows/release-dry-run.yml` (contents:read, `--no-sign`, no GitHub Release mutation)
- `.github/workflows/release.yml` (tag `v*`, `--ci --no-sign`, prerelease=true, unsigned notice in notes)
- `scripts/collect_release_assets.py`
- `docs/RELEASE_PROCESS.md`, `docs/RELEASE_CHECKLIST.md`, `.github/RELEASE_NOTES_v0.1.0.md`
- Required secrets listed for a **future** signed release; dry-run must not be described as production publication

---

## Cross-domain Entry Points this census does not own

| Entry | Owner |
| --- | --- |
| File Open / `args[0]` SGF load | 01 + 03 |
| Engine profile UI, one-click setup dialog, TensorRT install | 04 |
| First-use mouse/language/looks dialog | 02 |
| Fox/Yike/readboard menus | 06 |
| Readboard zip update protocol | 06 |
| Sync diagnostics window | 06 |
| Layout/window persistence | 02 |

---

## Environment checks still required (not run)

Marked **需定向运行核验** in Capabilities; summary:

1. Clean-machine install/launch/uninstall for each Java flavor and each Next unsigned bundle.
2. First-launch auto-setup success/fail on with-katago vs without.engine vs NVIDIA driver bands.
3. Manual update check on packaged `next-YYYY-MM-DD.N` vs `next-dev`.
4. Windows apply + rollback (kill helper mid-apply; deny elevation; SHA mismatch).
5. macOS DMG-from-disk vs Applications; Gatekeeper on signed vs unsigned.
6. Whether `.sgf` is registered with the OS.
7. Diagnostics zip contents and redaction on a real log dir.
8. Next: WebView2 bootstrapper on a machine without WebView2.

Repository evidence must not be substituted for those.

---

## Corrections vs invalid first-round reports

- Java source is **only** `/home/dev/dev/weiqi/worktrees/lizzieyzy-next/java-baseline-v1` @ `7b4027531c2b26062d0bfc27a040cc550cfbea4d`. Next is **only** `migration-coverage-audit` @ `18c6d189b8b01069975c4c40ead63a010249cb8c`.
- **No** citation of `42c92e3` or the dirty `/mnt/d` checkout.
- End-user Capabilities are separated from maintainer packaging/CI. Next `release.yml` / validators are REL-01/02 **evidence**, not “user can update from Help”.
- Help Diagnostics is a real Java Capability (`DiagnosticsDialog`), not guessed from class names.
- In-app updater is **manual**, signed-envelope, channel-aware; Windows in-place vs macOS/Linux download-and-open are distinct Capabilities.
- Next Help already **shows** 检查更新/简介 as disabled `尚未接入`; About is a status string, not parity.
- `REL-01` is not reclassified as a user install Capability.
