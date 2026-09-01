# 07 — Update, Packaging, Release, and Platform Census

Independent Domain 07 census for [Ticket 07](../issues/07-audit-update-packaging-release-platform.md). Destination writes land in `docs/JAVA_CAPABILITY_INVENTORY.md` Domain 07 and Matrix-owned fields for `REL-01`–`REL-10`. Plan R11 Delivery Order, Phase Gate, independent platform admission, and exit already exist and are not edited.

## Sources

| Tree | Path | Commit |
| --- | --- | --- |
| Java Migration Baseline v1 | `/home/dev/dev/weiqi/worktrees/lizzieyzy-next/java-baseline-v1` | `7b4027531c2b26062d0bfc27a040cc550cfbea4d` (verified `git rev-parse HEAD`) |
| Next inventory baseline | this worktree | `18c6d189b8b01069975c4c40ead63a010249cb8c` (Accepted-field comparison only) |
| Current destination docs | this worktree | `4eaaf6bc568aacbb57673df8348cacf387c226b0` at census time |

Read-only Java source census. No production edits, tests, builds, or runtime launches. Prior coverage-audit research 07 and Ticket 14 are product-decision sources, not a stop condition for this count.

## Method

1. Walk Help construction (`Menu.java` `this.add(helpMenu)` then each `helpMenu.add`), startup work-dir, first-run auto-setup, update package, diagnostics, About, OS icon, crash log, and `docs/INSTALL.md` / `PACKAGES.md` / `TROUBLESHOOTING.md` / `SUPPORT.md`.
2. Count a Capability only when it is one observable user goal. Count an Entry Point only when a control is constructed **and** added, shown, or registered. Nested dialog buttons stay Entry Points of the parent goal.
3. Maintainer packaging, signing, CI, validators, and `REL-01` remain excluded from the Capability count. History clearing, engine-profile creation, provider networking, and `.sgf`/`.gib` activation are links only.
4. After the census completed, compare the computed count with reference 13. 13 was not used as a stop condition.
5. Runtime-check is allowed only for a named dynamic-visibility, default, persistence, or failure fact not recoverable from frozen source. Repository Release Evidence, release-environment evidence, and Installed Live Evidence are Matrix classes, not Domain 07 Java runtime checks.

Key Java files: `gui/Menu.java`, `Lizzie.java`, `logging/WorkDirectoryResolver.java`, `logging/WorkDirectoryEnvironment.java`, `logging/LoggingRuntime.java`, `logging/LoggingSettings.java`, `logging/DiagnosticBundleExporter.java`, `logging/CrashHandlers.java`, `gui/DiagnosticsDialog.java`, `gui/ConfigDialog2.java`, `update/CheckUpdateDialog.java`, `update/UpdateDiscovery.java`, `update/UpdateCheckCoordinator.java`, `update/WindowsUpdateController.java`, `update/WindowsUpdateDialog.java`, `update/WindowsUpdateService.java`, `update/WindowsUpdateApplier.java`, `update/WindowsUpdatePaths.java`, `update/WindowsUpdatePlan.java`, `update/PlatformUpdateService.java`, `update/PackageUpdateDialog.java`, `update/InstalledUpdateState.java`, `update/UpdateManifestClient.java`, `update/UpdateVersion.java`.

## Computed count versus reference

**Computed Domain 07 end-user Capabilities: 11.**

Reference 13. Extra rows: none. Missing as Capabilities versus 13: `REL-C12` (docs/templates; in-app links are Entry Points of `REL-C11`) and `REL-C13` (Domain 02 `SET-CLEAR-PERSONAL-HISTORY`; Help item is a link only).

IDs: `REL-C01` `REL-C02` `REL-C03` `REL-C04` `REL-C05` `REL-C06` `REL-C07` `REL-C08` `REL-C09` `REL-C10` `REL-C11`.

`REL-01` is a maintainer gate, not an end-user Capability.

Help `this.add(helpMenu)` then adds, in order (`Menu.java:5164-5254`): Diagnostics and Logs; Stop Full Trace (enabled iff `fullTraceActive`); About → `openConfigDialog2(2)`; Check Update → `WindowsUpdateController.openCheckUpdatePage` (always added, never `setEnabled`); `clearAllPersonalData`. Stop Full Trace is an Entry Point of `REL-C10`. Clear personal data is Domain 02, not a twelfth Capability.

Source-complete corrections versus the prior coverage-audit write-up (not extra/missing IDs):

- Windows installed work-dir algorithm: first ASCII-safe writable of `{PUBLIC}/Documents/LizzieYzyNext`, `{PUBLIC}/LizzieYzyNext`, `{PROGRAMDATA}/LizzieYzyNext`, else `{userHome}/.lizzieyzy-next` (`WorkDirectoryResolver.windowsSharedWorkDirCandidates` 270–278; `WorkDirectoryEnvironment.system` `PUBLIC` / `PROGRAMDATA`). It is not APPDATA/LOCALAPPDATA. Allowed runtime-check: which candidate a typical machine actually uses.
- Logs directory is `{workDir}/logs` (`LoggingRuntime.java:67`). Export default is `{workDir}/diagnostics` (`DiagnosticBundleExporter.defaultOutputDirectory`; dialog passes `runtime.logsDirectory().getParent()`).
- No startup auto-check. `UpdateVersion.shouldSkipAutomaticCheck` is used only in `WindowsUpdatePlan.create` (unpackaged → empty item list) and tests. `UpdateDiscovery` returns `unavailableBuild()` when the installed version is not a packaged `next-YYYY-MM-DD.N` tag. Help Check is always registered; unpackaged is a Check result, not a hidden menu. Production caller of `openCheckUpdatePage` is only `Menu.java:5222`. Worker thread name `lizzie-update-manual`. `Config.autoCheckVersion` is commented out. `Lizzie.initializeAfterVersionCheck` is engine GTP version, not app update.
- Helper restart executable: `WindowsUpdateService.findAppExecutable` lists `appRoot` and takes the first `*.exe` (`163-171`). Unspecified `Files.list` order is an allowed runtime-check when multiple flavor launchers exist.

## Canonical Artifacts (Next; Ticket 14)

Java ships a 13-asset flavor matrix. Next Canonical Artifacts are four independent products. Extra CI MSI/deb/rpm formats are not Canonical. Each artifact states runtime, trust, state, update/handoff, and removal independently. Trust envelopes are owned by `REL-02`; discovery by `REL-03`; component identity by `REL-05`; Windows apply by `REL-06`; macOS/Linux handoff by `REL-07`; Windows rollback by `REL-08`. `REL-04` records the per-artifact lifecycle those items attach to. Inventory Domain 07 points at `REL-04` and does not restate this acceptance table.

| Artifact | Runtime | Trust | State | Update / handoff | Removal |
| --- | --- | --- | --- | --- | --- |
| Windows NSIS | Evergreen WebView2 bootstrap when absent | Authenticode-signed and timestamped | OS app-data | `REL-06` component apply | Uninstall preserves OS app-data by default |
| Windows portable | System Evergreen WebView2; absence fails before the web UI and routes to WebView2 or NSIS | Authenticode-signed and timestamped | Package-local `user-data/` | `REL-06` without changing `user-data/` | Deleting the directory destroys colocated state |
| macOS DMG | Platform WebKit; launch from Applications, not the mounted image | Signed, notarized, and stapled | OS app-data | `REL-07` opens the DMG | Deleting the app preserves OS app-data by default |
| Linux AppImage | Documented distribution/runtime; user-visible failure when missing | Envelope/payload verification; release notes state the exact trust and dependency contract (no repository/package-manager signature required) | OS app-data | `REL-07` opens the containing folder | Deleting the AppImage preserves OS app-data by default |

## Release and platform Capabilities

| Frozen ID | Observable user goal | Registered Entry Points | Default / persistence | Failure / non-mutation | Source | Runtime-check | Mapping |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `REL-C01` | Download, unpack or install a release package, and start the product | Download from goagent.top/download or GitHub Releases; Windows portable unzip + flavor EXE; Windows `installer.exe`; macOS DMG drag to Applications; Linux unzip + `chmod +x start-linux64.sh && ./start-linux64.sh`. `core-update.zip`, experimental packages, and TensorRT split archives are Entry Points of this goal / `REL-C06`, not new IDs. | Windows ordinary users: `.portable.zip`. Flavor by GPU in Java. Next Canonical Artifacts are the four-row table above (Matrix `REL-04`). | Wrong package / incomplete unzip / Gatekeeper: TROUBLESHOOTING. macOS: do not run from the mounted DMG. | `docs/INSTALL.md`; `docs/PACKAGES.md`; `docs/TROUBLESHOOTING.md`; `scripts/package_windows_exe.sh`; `scripts/package_macos_dmg.sh` | Not required. Clean-machine installability is `REL-04` Installed Live Evidence. | `REL-04` Platform Package Lifecycle. Extra CI formats are not Canonical Artifacts. |
| `REL-C02` | Resolve a writable work directory on first launch (portable vs installed) | Process start (`Lizzie.main` → `WorkDirectoryResolver.resolve`) | `-Dlizzie.work.dir` if set. Windows portable: walk ≤8 parents for `.lizzie-portable` → `{root}/user-data`. Else shared-dir algorithm above. Non-Windows: writable cwd else `~/.lizzieyzy-next` (legacy `.lizzieyzy-next-foxuid` migrates). | Resolver records diagnostics. Portable recovery backup `config.before-portable-recovery.txt`. Unwritable cwd may switch `user.dir`. Profile save failure → Domain 04 repair chip. | `WorkDirectoryResolver.java:28-180,226-278`; `WorkDirectoryEnvironment.java:12-68`; `Lizzie.java:334-336,381-386,462-476,894-964` | Which Windows shared-dir candidate a typical machine actually uses (dynamic default) | `REL-04` data-location contract. Installed variants use OS app-data. Windows portable uses package-local `user-data/`. |
| `REL-C03` | On first run or host change, detect bundled KataGo and write a usable engine profile | Startup when `firstTimeLoad` or hostname change. Interactive 一键设置 is Domain 04, not this Capability. | If no engine and not `autoload-empty`, `inspectLocalSetup` then `applyAutoSetup` when engine+configs+weight present; persist `first-time-load=false`. `without.engine` stays empty. | Exception logged; no bundle → `noBundledEngine`; save fail → profile-save repair. | `Lizzie.java:275-336,381-428,758-773,894-964,522-615`; `KataGoAutoSetupHelper` | Not required. NVIDIA CUDA staging is Domain 04. | Start in No-engine Mode (`ENG-02` / `UI-04`); never download an engine automatically. Explicit Engine Settings acquires signed backend/model (`REL-05`); Domain 04 creates/confirms the profile. |
| `REL-C04` | Discover packaged flavor and bundled runtimes | Implicit at update-check planning and first-run setup | Windows: `lizzieyzy-next-installed-manifest.json` flavor, else `engines/katago/windows-x64/lizzieyzy-next-engine-backend.txt` (`opencl`/`nvidia`/`nvidia50-cuda`→nvidia/`cpu`→with-katago), else `appDir/engines` → with-katago, else `without.engine`. Override `-Dlizzie.update.flavor`. macOS default `with-katago`. Portable marker `.lizzie-portable`. | Missing matching package → `NO_PACKAGE`. Missing bundled engine → `REL-C03` repair chip. | `WindowsUpdatePaths.java:20-178`; `PlatformUpdateService.java:92-231`; `InstalledUpdateState.java:14-43`; `docs/PACKAGES.md` | Not required | `REL-05` signed installed manifest of `app-core` and acquired KataGo backend/default-model. Abandon Java GPU flavor / JRE / JCEF. A later accepted owner, including admitted `CONTRIB-01`, may add a declared signed component through this seam without becoming an earlier R11 exit. |
| `REL-C05` | Manually check whether a newer signed release exists | Help → 检查更新 always added (`Menu.java:5219-5222`). Opening the page does **not** use the network; Check starts `UpdateDiscovery`. | Channel `stable`; source `official`. Unset version `next-dev`. Persist `update-channel`, `update-source`. Beta forces GitHub and does not persist a non-GitHub source. Envelopes: official `https://download.goagent.top/channels/stable/update-envelope.json`; GitHub latest `lizzieyzy-next-update-envelope.json`; beta `releases/download/channel-beta/lizzieyzy-next-update-envelope.json`. Java compares `next-YYYY-MM-DD.N`; Next uses SemVer. Single-flight `UpdateCheckCoordinator.inFlight`. | Stay on page with localized unpackaged / unsupported / no-update / no-package / fetch / invalid / generic failures. Close blocked while check in flight. Unpackaged = `unavailableBuild()`. | `Menu.java:5164-5222`; `CheckUpdateDialog.java`; `UpdateDiscovery.java`; `UpdateManifestClient.java`; `UpdateChannel.java`; `UpdateSource.java`; `UpdateVersion.java` | Not required | `REL-03` Update Discovery and Offer. SemVer; OS proxy; signed-envelope/schema validation; single-flight; stable default and beta; official/GitHub source policy. |
| `REL-C06` | After a Windows offer, download selected components, quit, apply via helper, restart | Offer dialog from successful `REL-C05`: 立即更新, pause/resume/cancel, View Release, close | Selected components = those newer than the installed manifest. Staging `{workDir}/update/staging/{tag}/`; helper `lizzieyzy-next-updater-helper.jar`; SHA-256 plus size; official then GitHub. | See `REL-C08`. `launchHelper` success then `dispose` + `Lizzie.shutdown()`; IOException leaves the process running. Non-portable and app dir not writable → PowerShell `RunAs`. SHA mismatch/cancel do not apply. | `WindowsUpdateDialog.java:28-356`; `WindowsUpdateService.java:13-190`; `WindowsUpdateApplier.java:23-238`; `ResumableDownloader.java:22-38` | Which `*.exe` `Files.list` returns first under `appRoot` when several flavor launchers exist | `REL-06` Windows Component Download and Apply. Covers NSIS and portable roots without changing user data. Close through `APP-03` only after successful helper launch. |
| `REL-C07` | After a non-Windows offer, download the matching signed package and open it or its folder | `PackageUpdateDialog`: 下载新版, pause/resume/cancel, View release | Download dir `~/Downloads` if that path is a directory, else `{workDir}/update/downloads` | Cancel keeps `.part` for resume. Network fallback official → GitHub. Desktop unsupported → IOException. Never overwrite the running tree. | `PackageUpdateDialog.java:26-220`; `PlatformUpdateService.java:38-50,112-126`; `PackageUpdateAdapter.java:1-64` | Not required | `REL-07` macOS/Linux Package Update Handoff for matching DMG/AppImage. |
| `REL-C08` | Survive a failed Windows apply without leaving a half-replaced install | Automatic inside `WindowsUpdateApplier.apply`; download failures stay in `REL-C06` | Backup dir `staging/backup-<timestamp>/`; not a user setting | Wait for main PID ≤30s. Current files move to backup; exception restores in reverse. Zip-slip blocked. Maintainer GitHub-release withdrawal is **not** this Capability. macOS/Linux have no apply rollback. | `WindowsUpdateApplier.java:39-82,210-238,266-283,305-318`; `WindowsUpdateDialog.java:335-355` | Failure presentation if the helper fails after the app has already quit (Installed Live Evidence for `REL-08`, named here because source cannot show post-quit UI) | `REL-08` Update Failure, Rollback, and Recovery. Pre-apply failures leave the running install untouched. Incomplete restoration retains the backup and provides repair/reinstall. |
| `REL-C09` | Integrate the installed app with the host OS enough to launch and not be blocked | Installer Start Menu / desktop shortcuts; macOS Dock icon (`Lizzie.installApplicationIcon`, Mac-only `/assets/logo.png`); macOS Privacy & Security Open Anyway. Linux `.desktop` is not a first-class user control. Frozen tree has no file-associations packaging script. argv file load is `SHELL-03`. | Taskbar `ICON_IMAGE` then Java 8 `setDockIconImage`. | Icon failures ignored/logged. Gatekeeper/SmartScreen: TROUBLESHOOTING. | `Lizzie.java:360,783-861,1012-1017`; `docs/INSTALL.md`; `docs/TROUBLESHOOTING.md` | Not required. OS file association and per-artifact launch/blockers are `REL-04` Installed Live Evidence. | Domain 07: `REL-02` package trust plus `REL-04` install/upgrade/removal and packaging evidence needed by `APP-01`. `APP-01` still owns `.sgf`/`.gib` activation and current-game replacement. |
| `REL-C10` | Turn diagnostic logging on/off, export a sanitized zip, open the log folder | Help → Diagnostics and Logs; Help → Stop Full Trace (enabled iff full-trace active); dialog Apply / Export / Cancel export / Open log folder. CrashHandlers installed at bootstrap. | Defaults: diagnostics **on**, all modules, all scopes (`LoggingSettings.defaults`). Persist `uiConfig.logging`. Full Logs requires confirm; default off. Logs `{workDir}/logs` (`app.log`, `crash.log`); export default `{workDir}/diagnostics`. Caps 24h/50MiB app, 24h/10MiB crash, 50MiB raw. | Apply failure reverts UI from runtime. Export cancel via flag; size-bounded and sanitized. Open-folder failures swallowed. No automatic upload. | `Menu.java:5170-5205`; `DiagnosticsDialog.java:56-134,375-464,955-1025`; `LoggingSettings.java:10-101`; `LoggingRuntime.java:64-67`; `DiagnosticBundleExporter.java`; `CrashHandlers.java:16-80` | Stop Full Trace enabled only while `fullTraceActive`; scopes lock during an active session | `REL-09` Diagnostics and Support Bundle. Bounded ordinary local logs on by default; full trace explicit and default off; sanitized cancellable export; open-folder access. Swing dialog structure is not the claim. |
| `REL-C11` | See product name, release tag, short intro, and project links | Help → About → `openConfigDialog2(2)`; settings modern nav About (`targetTabIndex=2`); check-update page header shows `Lizzie.nextVersion`. Repo/Releases/Issues via `createAboutLinkButton`. QQ `299419120` is copy, not a control. | Packaged builds inject `LIZZIE_NEXT_VERSION` / `-Dlizzie.next.version`. Unset → `next-dev`. Maven `2.5.3` is **not** the user-facing Next tag. | Browse no-ops if Desktop unsupported. | `Menu.java:5208-5216`; `ConfigDialog2.java:146-162,2331-2458,2670-2684`; `Lizzie.java:275-294,754-773` | Not required | `REL-10` Product Identity and Support Routing. Packaged SemVer and channel plus Releases, Issues, and support links. Abandon Java marketing-card presentation and the static `0.1.0` status string as evidence. |

## Explicit non-rows (not extra Domain 07 Capabilities)

| Surface | Why it is not a Domain 07 Capability |
| --- | --- |
| `REL-C12` SUPPORT.md / issue templates / Discussions / QQ in docs | Maintainer/docs. In-app links are Entry Points of `REL-C11`. Ticket 14 absorbed this heading into `REL-09` / `REL-10`. |
| `REL-C13` Help → `clearAllPersonalData` | Domain 02 `SET-CLEAR-PERSONAL-HISTORY`. Ticket 07 records a link only. |
| `REL-01`, `scripts/package_*.sh`, INSTALL/PACKAGES/TROUBLESHOOTING as documents, `lizzieyzy-next-update-manifest.json` | Maintainer gate / packaging evidence. |
| Java updater classes, Swing update widgets, helper `main`, `AppHealthDto` | Implementation of `REL-C05`–`REL-C08` / Next internal API. |
| `-Dlizzie.smoke.open*` | Lab-only JVM flags. |
| Stop Full Trace menu item; nested diagnostics Apply/Export/Open folder; channel/source radios | Entry Points of `REL-C10` / `REL-C05`. |
| `core-update.zip` / experimental / TensorRT split assets | Entry Points of `REL-C01` / `REL-C06`. |
| CrashHandlers uncaught persistence | Entry Point / persistence of `REL-C10`, not a separate goal. |
| KataGo 一键设置 / `initSettings` | Domain 04. |
| FirstUse wizard | Domain 02. |
| `SyncDiagnosticsDialog` / readboard zip update | Domain 06; no Help registration found. |
| INSTALL 野狐抓谱 | Domain 06. |
| Java GPU flavor / bundled JRE / JCEF | Abandoned as Next product; occupancy of `REL-C04`. |
| MSI / deb / rpm CI outputs | Not Canonical Artifacts. |
| Maintainer GitHub-release withdrawal | Cannot satisfy `REL-08`. |
| Quick Links, Theme tab, `UpdateCheckCoordinator` internals | Other menu, Swing internals. |
| `Config.autoCheckVersion` | Commented out. |

## Owner routing (this ticket records links only)

| Concern | Owner | Domain 07 records |
| --- | --- | --- |
| `.sgf` / `.gib` activation and current-game replacement | `APP-01` | Packaging evidence only; no duplicate open semantics. |
| Graceful shutdown before Windows apply | `APP-03` | Close only after successful helper launch. |
| No-engine launch / engine profile | `ENG-02` / `UI-04` / Domain 04 | `REL-C03` / `REL-05` acquisition handoff. |
| Durable preference mechanism | `PREF-01` | Channel/source and logging keys; not semantic owner. |
| Histories / recents clear | `SET-CLEAR-PERSONAL-HISTORY` → `SGF-09` / provider owners | Help item is a Domain 02 Entry Point. |
| Provider HTTP proxy | Domain 06 Provider Network Policy | Update uses OS proxy on `REL-03` only. |
| Contribution client component | `CONTRIB-01` + `REL-05` | Same manifest/acquisition seam after acceptance. |

## REL authority split

| Item | Matrix-owned | Plan-owned |
| --- | --- | --- |
| `REL-01` Release preflight and dry-run | Status Partial. Repository: validators and compile-oriented dry-run paths. Live/environment: **Not run** for production publication; repository dry-run is not Installed Live Evidence. Gap/acceptance: exact-commit validators that state tag/commit/platform/signing limits. **Depends on: —.** Maintainer gate, not a user Capability. | R11 Delivery Order 1. Phase Gate: may proceed at any time. Global R11 exit member. |
| `REL-02` Trusted production artifacts | Status Missing. Repository: workflow structure without production credentials. Release-environment: **Not run** (Windows Authenticode+timestamp; macOS sign/notarize/staple; Linux envelope/payload + notes). Gap/acceptance: signed envelope and verified payload on stable/beta; unsigned public-validation prereleases labeled and never in updater feeds. **Depends on: —.** | R11 Delivery Order 2 before `REL-03`/`REL-05`. Production-trust gate per Shipped Platform. |
| `REL-03` Update discovery and offer | Status Missing. Repository: Help → 检查更新 visible-disabled `尚未接入`; no updater plugin. Release-environment: **Not run** hosted feed. Installed Live: **Not run** Help-triggered discovery on an installed Canonical Artifact. Acceptance: manual Help; stable/beta; official/GitHub; persist valid choices; SemVer; OS proxy; typed failures without mutating the install. **Depends on: `REL-02`.** | Delivery Order 2 with `REL-05` after `REL-02`. |
| `REL-04` Platform package lifecycle | Status Missing. Repository: packaging preflight. Installed Live: **Not run** independently for each of the four Canonical Artifacts (runtime, trust, state, update/handoff, removal). Association evidence for `APP-01` is absent. **Depends on: —.** | Delivery Order 3 per Canonical Artifact, independently. `APP-01` final acceptance needs this association evidence plus the R5 semantic gate. Independent platform admission. |
| `REL-05` Managed installed components | Status Partial. Repository: configured-path checks. Installed Live: **Not run** packaged-app smoke. Acceptance: signed manifest of `app-core` + acquired KataGo; no-engine launch without optional components; explicit Engine Settings acquisition or preserve-and-explain. JRE/JCEF/file-guessed GPU/unaccepted domain components excluded. **Depends on: `REL-02`.** | Delivery Order 2 with `REL-03`. Deferred domain components, including admitted `CONTRIB-01`, do not become earlier R11 exits. |
| `REL-06` Windows component download and apply | Status Missing. Repository: absent. Release-environment: **Not run** signed component feed. Installed Live: **Not run** NSIS and portable helper/elevation/apply. **Depends on: `REL-03`, `REL-05`, `APP-03`.** | Delivery Order 4 after those three. Windows-only Shipped Platform branch. |
| `REL-07` macOS/Linux package update handoff | Status Missing. Repository: absent as in-app flow. Installed Live: **Not run** DMG/AppImage download and handoff. **Depends on: `REL-03`.** | Delivery Order 5 after `REL-03`. macOS/Linux Shipped Platform branch. |
| `REL-08` Update failure, rollback, and recovery | Status Missing. Repository: `docs/RELEASE_PROCESS.md` maintainer GitHub withdrawal, not installed rollback. Installed Live: **Not run** Windows reverse-rollback, journal, restored-version restart, incomplete-repair. **Depends on: `REL-06`.** | Delivery Order 6 after `REL-06`. Windows-only Shipped Platform branch. Maintainer withdrawal cannot satisfy this item. |
| `REL-09` Diagnostics and support bundle | Status Missing. Repository: no Help diagnostics item. Installed Live: **Not run** export/open-folder smoke. **Depends on: —.** | Delivery Order 7 in parallel with `REL-10`. Global R11 exit member. |
| `REL-10` Product identity and support routing | Status Partial. Repository: Help → About sets chrome status `LizzieYzy Next 0.1.0 · 桌面复盘工作区` with no dialog, commit/channel, or links. Installed Live: **Not run** packaged SemVer/channel and working support links. **Depends on: `REL-04`.** | Delivery Order 7 in parallel with `REL-09`. Global R11 exit member. |

Plan R11 Delivery Order (Plan only; not a second Matrix start-prerequisite set): `REL-01` → `REL-02` then `REL-03`/`REL-05` → `REL-04` per Canonical Artifact → `REL-06` → `REL-07` → `REL-08`; `REL-09`/`REL-10` in parallel. Do not copy Delivery Order into Matrix `Depends on`.

## Evidence classes (kept distinct)

| Class | Used for | Not implied by |
| --- | --- | --- |
| Repository Release Evidence | Validators, dry-run, updater fixtures, unsigned generated bundles, documentation | Any live column |
| Release-environment evidence | Exact tag/commit, CI run, produced Canonical Artifact, channel feed, signing identity, Windows timestamp, macOS notarization/stapling | Workflow files or secret *names* |
| Installed Live Evidence | Per Canonical Artifact: OS, artifact identity, versions, components, trust, install/unpack, primary launch, data location, update/handoff, uninstall/delete | Repository checks; Java baseline runs |
| Windows updater fault evidence | Invalid signature, missing component, interrupted download/resume, cancel, helper-launch failure, elevation denial, apply interruption, reverse rollback, journal, restored restart, incomplete-repair; portable WebView2 present/absent | Maintainer GitHub withdrawal |
| macOS/Linux update evidence | Verified download, cancel/resume, DMG/AppImage handoff, unchanged running install on failure, successful manual replacement | In-place overwrite (forbidden) |

Unavailable credentials, machines, feeds, or infrastructure are **`Not run`**. No repository check, unsigned dry-run, generated bundle, documentation, or Java baseline run substitutes for missing production or installed proof.

## Remainder and Accepted-field preservation

Every Domain 07 Capability maps to one supported Parity Item, an explicit split list, an owner-routed mapping, or an explicit absorption/exclusion. There is no release or platform remainder.

No Domain 07 mapping is Accepted. Original sixteen Accepted items versus Next `18c6d189b8b01069975c4c40ead63a010249cb8c` keep ID, observable scope, status, evidence, remaining gap, and acceptance. This ticket does not edit foreign `APP-01`/`APP-03`/`ENG-*`/`PREF-01`/`SGF-09`/`PROV-*`/`CONTRIB-01`/`GAME-*`/`SET-CLEAR-PERSONAL-HISTORY` contracts.

Plan R11 is unchanged: independent platform admission, Delivery Order, Phase Gate, and exit already exclusive to the Plan.
