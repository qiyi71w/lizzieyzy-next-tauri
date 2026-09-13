# Migration Coverage

This context defines the language used to audit the frozen Java product and plan deliberate behavior coverage in LizzieYzy Next.

## Language

**Capability（用户能力）**:
A distinct observable user goal or behavior from Migration Baseline v1. Multiple controls that invoke the same behavior remain one Capability.
_Avoid_: Widget, Java class, menu item

**Entry Point（入口）**:
A menu item, toolbar control, shortcut, dialog action, startup path, or other user-reachable way to invoke a Capability.
_Avoid_: Capability

**Capability Inventory（能力清单）**:
The exhaustive trace from each user-reachable baseline behavior and Entry Point to one Parity Item or an explicit exclusion disposition.
_Avoid_: Parity Matrix, roadmap

**Parity Item（对等项）**:
A stable, acceptance-sized migration claim with status, disposition, evidence, remaining gap, and acceptance conditions. It describes observable behavior rather than Swing implementation.
_Avoid_: Ticket, Java feature

**Disposition（迁移处置）**:
The deliberate product decision for a Capability: equivalent migration, Next redesign, deferred, abandoned, or Swing-only implementation.
_Avoid_: Status, phase

**Successor Item（后继项）**:
A new Parity Item created when audit findings materially extend an already Accepted item. It preserves the original item’s ID, scope, evidence, and historical acceptance.
_Avoid_: Reopened item, expanded accepted item

**Migration Phase（迁移阶段）**:
An ordered group of dependent Parity Items that forms a complete user workflow. It owns sequencing and exit criteria, not item-level evidence.
_Avoid_: Capability, ticket

**Item Start Prerequisite（条目开工前置）**:
A capability dependency that must be satisfied before work on a Parity Item may begin. It is intrinsic to that item, not a scheduling preference.
_Avoid_: Depends on, Delivery Order, Deferred Promotion Gate

**Deferred Promotion Gate（延期项晋级门槛）**:
A product or decision condition that must be satisfied before a Deferred Parity Item enters executable migration scope. It does not govern an item already admitted for execution.
_Avoid_: Admission gate, Item Start Prerequisite, Delivery Order

**Delivery Order（交付顺序）**:
The planned sequence among Parity Items for coherent delivery. It does not prohibit parallel starts unless the earlier item is also an Item Start Prerequisite.
_Avoid_: Depends on, Item Start Prerequisite, critical dependency

**Migration Phase Gate（迁移阶段门槛）**:
A phase-wide condition required to enter or exit a Migration Phase. It is not an item-level dependency or a Deferred Promotion Gate.
_Avoid_: Depends on, Item Start Prerequisite, admission gate

**Review Autoplay（复盘自动播放）**:
Automatic traversal of existing moves along the selected continuation of the current game. It never creates a move.
_Avoid_: Main-line autoplay, Variation Replay, Engine Continuation

**Variation Replay（变化回放）**:
Progressive presentation of a candidate or Sub-Board variation without changing the selected SGF node or the game tree.
_Avoid_: Candidate autoplay, Review Autoplay, Engine Continuation

**Engine Continuation（引擎续走）**:
Automatic creation of new game-tree moves from engine decisions during an autoplay flow.
_Avoid_: Engine-best-move replay, Review Autoplay, Variation Replay

**Next-move Review Marker（下一手复盘标记）**:
A review-board overlay that marks already-recorded child moves of the selected SGF node. It is not an engine prediction; an optional grade may compare analysis for the selected node and its primary child.
_Avoid_: Engine suggestion, predicted next move

**Graph Perspective（胜率图视角）**:
The encoding of the single win-rate series: always Black, or the entire series converted to the selected node's side to play.
_Avoid_: Dual black/white curves, per-move sawtooth, always-black candidate overlay, win-rate-always-black

**Winrate Line（胜率线）**:
The optional chart series that plots win rate against move number.

**Score Lead Line（目差线）**:
The optional chart series that plots score lead against move number.

**Blunder Bar（失误条）**:
A per-move vertical mark on the chart for a sudden win-rate or score swing.
_Avoid_: Next-move Review Marker, board blunder overlay

**Graph Hover（胜率图悬停）**:
A non-navigating readout of chart series at the pointer. It does not change the selected node.
_Avoid_: Click-to-jump, REVIEW-01

**Score Lead Scale（目差刻度）**:
The persisted floor of the score-lead axis full-scale magnitude. Session growth to larger absolute leads on the current line is not persisted.

**Next-move Marker Mode（下一手标记模式）**:
The single state of the Next-move Review Marker: Off, Variations, or Graded. Variations marks all existing child moves; Graded adds an analysis grade for the primary child.
_Avoid_: Simple/info booleans, marker toggles

**Primary Child（主子节点）**:
The first child of an SGF node, representing the current main-line continuation at that branch point.
_Avoid_: Best move, engine suggestion

**Analysis Context（分析口径）**:
The immutable engine/profile, model/config, rules, komi, and request-option set that makes two position analyses comparable. Completed visit counts may differ; missing or mismatched context makes a pair non-comparable.
_Avoid_: Profile ID, cache hit, analysis job

**Analysis Task（分析任务）**:
A user-started, session-only analysis of an explicit set of positions in the current game, with fixed analysis conditions, stage budgets, and a resumable completed set. Navigation and ordinary Pause/Continue preserve its identity; edits to position semantics or tree structure, or a new Foreground Engine Run, require a new task.

**Position Interval（局面区间）**:
An inclusive range of positions after numbered moves; zero denotes the initial position. Pass counts as a move, while setup changes the position without increasing the move count.

**Selected Review Line（当前复盘线路）**:
The root-to-leaf line determined by the user's remembered branch choices, with the Primary Child used at an unchosen branch. An analysis task fixes this line when it starts.

**Supporting Position（辅助局面）**:
A position needed to compare a selected move's before-and-after analysis, even when it lies outside the user's position interval or color filter. It is listed separately from the requested targets and retains its own game-tree location.

**Analysis Task Completed Set（分析任务完成集）**:
The positions that have met a stage's requested search conditions within the same Analysis Task. Existing SGF visit counts alone do not establish membership.

**Native File Activation（原生文件激活）**:
An operating-system request to open an SGF or GIB in the existing application window or to launch the application for that file. It uses the same current-game replacement safety as an in-application open.
_Avoid_: Command-line import, second window

**Safe Graceful Shutdown（安全退出）**:
The application-owned exit flow that resolves dirty game state, persists owned state, and gives each running resource a bounded opportunity to stop before process exit.
_Avoid_: Force exit, window close

**Current-game Session Recovery（当前棋谱会话恢复）**:
A recoverable snapshot of the authoritative current game and cursor after an unclean exit or an explicit restore preference. It never represents restoration of live engines, analysis jobs, or external sessions.
_Avoid_: Full workspace restore, auto-resume process

**Shortcut Registry（快捷键注册表）**:
The single product-owned catalog of actions, primary keys, supported aliases, focus rules, and help labels.
_Avoid_: Component-local shortcuts, Java key map

**Shortcut Reference（快捷键参考）**:
The searchable in-application view generated from the Shortcut Registry.
_Avoid_: Hold-X overlay, static shortcut document

**Shipped Platform（已准入发布平台）**:
A platform and its Canonical Artifacts whose production trust requirements and installed live acceptance have passed. A CI build or unsigned validation artifact does not make a platform shipped.
_Avoid_: Build target, release candidate

**Canonical Artifact（标准发布包）**:
A package format that the product commits to install, update or hand off, and exercise as an installed product before admitting its platform. Extra CI bundle formats are not Canonical Artifacts unless deliberately promoted.
_Avoid_: Any generated bundle, workflow artifact

**Portable Installation（便携安装）**:
A distribution whose application files and persistent user state can move together as one directory and that has no registered uninstall lifecycle. Deleting that directory removes both the application and its colocated state.
_Avoid_: Installer, hermetic runtime

**Managed Installed Component（受管安装组件）**:
A release-owned, independently versioned application or optional runtime payload declared by the installed manifest and delivered through the signed component-update contract. Operating-system WebViews and obsolete Java runtimes are not Managed Installed Components.
_Avoid_: User file, system dependency, Java package flavor

**Repository Release Evidence（仓库发布证据）**:
Deterministic source, fixture, validator, test, and dry-run evidence that proves a release contract or artifact shape without proving production publication or installed behavior.
_Avoid_: Installed Live Evidence, production release

**Installed Live Evidence（安装态实证）**:
A recorded exercise of a specific Canonical Artifact on its target operating system, including its trust state, install or unpack path, launch, update or handoff, failure recovery, and removal behavior.
_Avoid_: Build success, repository validation

**Panel Magnification（面板放大）**:
A mutually exclusive workspace emphasis that enlarges either the sub-board or the winrate graph. It is not rail-width dragging and not collapsing a whole rail.
_Avoid_: ExtraMode, large-subboard, large-winrate-graph, LAYOUT-01, LAYOUT-04, UI-01

**Sub-Board（副棋盘）**:
The dedicated second board in the analysis/reference rail. It is not the main board and not a detached window.
_Avoid_: OverlayMode, ExtraMode four-sub, independent sub-board window

**Sub-Board Mode（副盘内容模式）**:
The mutually exclusive content of the Sub-Board: Variation or Raw. It is not presence and not whether PV can be drawn.
_Avoid_: mini-board as a mode, OverlayMode, heatmap-on-sub

**Variation（变化图）**:
The Sub-Board Mode that shows the current position plus the active candidate PV. `ANA-04` owns that drawing when this mode is selected.
_Avoid_: a second PV widget

**Raw（纯棋子）**:
The Sub-Board Mode that shows only the current-position stones, with no PV, branch, or move numbers, even when candidates exist.
_Avoid_: rail collapse, LAYOUT-04

**Generated Comment Append（生成评论写入）**:
Persisting engine-generated winrate, score, and playouts text into the selected node's comment, which serializes as SGF `C`.
_Avoid_: Personal comment, analysis header, TeacherCommentCodec

**Player Names on Review（复盘棋手名）**:
The current game's black and white names as shown in review chrome. Missing or blank names use the generic 黑棋/白棋 labels.
_Avoid_: SGF-13, board overlay, name-visibility preference

**Main Window Always-on-top（主窗口置顶）**:
Keeping the main application window above other windows.
_Avoid_: Window geometry, analysis-frame always-on-top

**Educational Tip（教育提示）**:
A one-time, permanently dismissible instructional notice about a product behavior. Reset Guidance can restore it.
_Avoid_: Hint, tooltip, Safety Confirmation

**Safety Confirmation（安全确认）**:
A blocking confirmation that protects against data loss or irreversible replacement. It cannot be permanently dismissed.
_Avoid_: Educational Tip, do not show again

**Guidance Producer（引导生产者）**:
A user-facing capability that emits an Educational Tip. Dismissal storage and Reset Guidance are not themselves producers.
_Avoid_: GUIDE-01 as a tip, onboarding wizard

**Reset Guidance（重置引导）**:
The action that re-enables dismissed Educational Tips. It does not change Safety Confirmations, window geometry, layout, or other preferences.
_Avoid_: Reset window position, restore panel sizes, reset all settings

**Provider Network Policy（服务商网络策略）**:
The shared outbound routing contract for Next-owned remote-provider HTTP(S) and WebSocket(S): per-request Windows/macOS platform resolution (including system-owned PAC/WPAD), process environment overrides, Linux proxy environment variables, and `NO_PROXY`. It excludes browser-owned traffic, local readboard, inbound WebBoard, and updates; Next never collects proxy credentials, executes PAC itself, or falls back to direct after proxy failure.
_Avoid_: Update proxy, Java proxy UI, readboard proxy

**SSH Engine Profile（SSH 引擎配置）**:
An Engine Profile whose engine program runs on a user-managed SSH host and whose protocol input/output travels through that connection. It owns the remote host, authentication reference, and remote command; it is not a provider account or compute catalog.
_Avoid_: Remote Compute Provider, local Engine Profile, SSH implementation

**Remote Compute Provider（远程算力服务商）**:
An external compute service selected through an account-backed catalog or an explicit service endpoint to supply engine computation. It is not a user-managed SSH host or a game-record provider.
_Avoid_: SSH Engine Profile, provider sync, remote-compute transport

**Contribution Service Integration（贡献服务集成）**:
A user-authorized relationship that supplies local computation to one admitted distributed-training service and reports its participation state. It owns service identity, account consent, and the remote-service lifecycle; it does not own current-board presentation or ordinary game-record providers.
_Avoid_: Contribute Board Session, Provider, Engine Profile

**Contribution Client Component（贡献客户端组件）**:
An optional Managed Installed Component admitted for one Contribution Service Integration. Its official client version, distributed-training capability, compute backend, and service eligibility are independent of every Foreground Engine Profile and backend component.
_Avoid_: Engine Profile, KataGo analysis backend, user-selected executable

**Contribution Network Policy（贡献网络策略）**:
The outbound routing contract of a Contribution Run: a fixed, unauthenticated proxy supplied through lowercase `https_proxy` or `http_proxy`, or a direct connection when neither is present. It does not claim Provider Network Policy, platform proxy resolution, PAC/WPAD, `NO_PROXY`, proxy credentials, or route fallback.
_Avoid_: Provider Network Policy, update proxy, application proxy setting

**Contribution Run（贡献运行）**:
One explicit execution of a Contribution Service Integration, from Start until Stop, failure, or application exit. At most one exists, it is globally exclusive with every local Engine Run, Analysis Job, and Match Session, and it is never recovered or automatically recreated.
_Avoid_: Contribution Service Integration, Contribute Board Session, Engine Run

**Contribute Board Session（贡献棋盘会话）**:
A board-occupying session opened explicitly for an active Contribution Run. It temporarily presents contributed games without replacing or dirtying the authoritative current game and excludes every Match Session while open. Close Watch restores the exact prior game and cursor without stopping the Contribution Run; Stop Contribution, failure, or exit closes the session and restores them.
_Avoid_: Contribution Service Integration, Contribution Run, Match Session, provider synchronization

**Contribution Consent（贡献同意）**:
A versioned acknowledgement required before the first Contribution Service Integration run and again whenever its disclosed contract changes. It states that the username is public, local compute and electricity are consumed, training games and data are uploaded to an external service, and stopping ends only the local contribution run.
_Avoid_: Educational Tip, Safety Confirmation, service terms

**System Credential Store（系统凭据库）**:
Operating-system-owned storage that holds user secrets for Next outside application settings, portable state, and logs. Next persists only a non-secret reference with the owning configuration.
_Avoid_: Encrypted application settings, session credential, preference storage

**External-board Engine Match（外部棋盘引擎对局）**:
A Match Session whose authoritative board is an external readboard target and whose engine move advances that external game.
_Avoid_: GMA, readboard synchronization, Analysis Job

**Final-decision Move Mode（最终决策落子模式）**:
An External-board Engine Match move policy that advances the external game only with the engine's completed move decision.
_Avoid_: GMA, live leading candidate

**Leading-candidate Move Mode（首选候选落子模式）**:
An External-board Engine Match move policy that advances the external game with the rank-one analysis candidate when the first requested positive time or visit limit is reached.
_Avoid_: 一选落子, final engine decision

**Pending External Turn（待确认外部回合）**:
The single engine turn whose move has been sent to the external target but has not yet been confirmed by the exact authoritative successor position. It does not advance the Match or current game.
_Avoid_: Committed move, place acknowledgement

**Yike Personal Category（弈客个人分类）**:
The frozen Yike Live Center category labeled Personal. The label identifies a provider category; it does not by itself establish user-authenticated or private behavior.
_Avoid_: Provider-Authenticated Read, private room, personal account session

**Provider-Authenticated Read（服务商认证读取）**:
A remote-provider read whose authorized result depends on the end user's provider identity or account session. It is distinct from anonymous, guest, or application-signed provider reads.
_Avoid_: Yike Personal Category, public signed read, system-browser page access

**Review-board Image Export（复盘棋盘图像导出）**:
A file-producing Capability whose visual subject is the current main-board review view. It does not own analysis encoding or current-game persistence.
_Avoid_: Winrate Chart Image Export, Sub-Board Image Export, Save

**Sub-Board Image Export（副棋盘图像导出）**:
A file-producing Capability whose visual subject is the dedicated Sub-Board. It is distinct from Sub-Board presence and Sub-Board Mode.
_Avoid_: Review-board Image Export, Winrate Chart Image Export, rail capture

**Winrate Chart Image Export（胜率图图像导出）**:
A file-producing Capability whose visual subject is the current `ANA-11` chart encoding for the selected line. It does not own that encoding.
_Avoid_: Review-board Image Export, full analysis workspace capture

**Image Export Snapshot（图像导出快照）**:
The immutable visual state bound when an image-export action is invoked. Choosing a destination does not refresh the snapshot.
_Avoid_: Current widget pixels, live analysis stream

**Recent Image Export Directory（最近图像导出目录）**:
The parent directory of the last successfully completed Review-board or Winrate Chart Image Export, shared by both Capabilities.
_Avoid_: Current-game source directory, recent kifu directory

**Armed Engine Turn（已登记引擎回合）**:
A single admitted External-board Engine Match request whose move mode and limits are fixed while it waits for an exact authoritative position to show its engine side to play.
_Avoid_: Pending External Turn, queued replacement request

**Yike Authenticated Read and Play（弈客认证读取与对弈）**:
A Yike-specific capability in which an authorized account may read permitted room state and play its assigned side. It is distinct from public read-only synchronization, system-browser play, and engine-driven external matches.
_Avoid_: Provider-Authenticated Read, Play & Sync, External-board Engine Match

**Pending Provider Move（待确认服务商落子）**:
The single human move submitted during Yike Authenticated Read and Play but not yet confirmed by the exact authoritative successor position. It does not advance the Match Session or current game.
_Avoid_: Committed move, request acknowledgement, Pending External Turn
