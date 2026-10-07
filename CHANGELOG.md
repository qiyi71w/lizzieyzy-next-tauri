# Changelog

## [Unreleased]

### English

- Added offline Function Search shared by Help, the fixed toolbar and registry-owned Ctrl/Command+K. It searches registered actions and exact settings targets with platform shortcuts, visible disabled reasons and original-owner guards, and restores source focus on cancellation.
- Added activation-fenced exact-target navigation, including initial Komi focus in Game Info and return to the retained search session on Cancel; metadata/history semantics are unchanged.
- Added current-build About identity and centralized source/Issues/Help addresses; unconfigured release/update/support channels remain unavailable. Corrected Cargo's source repository without changing product or app-data identity.
- Corrected N's explanatory state while preserving Ctrl+Home New, no Ctrl+N, and the existing Human New/Continue menu owners. Search resources use the complete Chinese compatibility bundle and deterministic per-key locale fallback; no final language list or persistence was introduced.
- Added an independent Tencent kifu center with username/chessId lookup, cancellable previews, cursor continuation across 25-row pages, eight-entry query history, and explicit SGF-07 import. Complete-tree normalization preserves game metadata and effective komi through Save/reopen; requests consume the shared network policy.
- Preserved accepted Tencent history writes across provider-panel changes and retained failed clear/query intent for visible retry after reopening the center.
- Added explicit Yike Start sync and Play & Sync with a Rust-owned read-only session, bounded failure pause/Retry, Stop-to-edit, same-count source reconciliation, independent sync preferences and browser handoff. Save As preserves its invocation snapshot while later source frames remain dirty.
- Added read-only readboard live sync through the same external-sync owner: Start passes one SGF-07 decision on the first frame, later frames follow the frozen Java recovery/turn rules with real moves and static-setup rebuilds, disconnect pauses with the last-good board, Retry restarts the runtime and Stop restores editing. Always-sync, focus, mute and jump-to-last persist independently. Historical Windows source-runtime evidence covers real Fox spectator rooms and R10 cross-source integration on the original candidates recorded in DEVELOPMENT; it is not a new native acceptance run on the PR candidate. Installed-network evidence remains separate.
- Fixed readboard room changes keeping the previous room's side to play: a rebuilt root now takes its side to play from the frame's trusted last-move marker, including when an earlier sync left a root PL. Frames without a trusted marker keep the frozen Java baseline.
- Fixed active-sync handoff to a one-shot provider import: validated previews now reach the shared dirty decision, Cancel retains the source session, and committed imports seal its old requests/frames before restoring editing.
- Fixed readboard's explicit Sync request being ignored while Yike owned the game; both Start entry points now use the shared source-switch transaction and retain the original session on dirty Cancel.
- Kept an admitted external-sync candidate owned by its protected replacement transaction: late Start cancellation cannot remove it during commit, and the readboard Cancel control is disabled throughout that phase.
- Reset discarded pending readboard snapshot state while preserving Sync candidates; retain the original Start error when cancellation cleanup fails, and reserve the cleanup-failure diagnostic for the matching runtime phase.
- Treat Windows WPAD-only discovery absence (`12180`) as permission to evaluate static system proxy/bypass settings; configured PAC and selected-route failures remain terminal. Corrected native request-error callback dispatch so WinHTTP failures reach this decision.
- Corrected the controlled-engine handshake-cancellation test to exercise bounded Retry Stop when cleanup cannot yet be confirmed, without changing its 150ms cleanup budget or production behavior. Added zero-budget coverage for retained reservation/profile protection and release only after confirmed reaping.

- Added the Yike public center with Recommend/Local pagination, cancellable record and URL previews, complete-tree SGF-07 imports, and a persisted public locator. Current public-source business acceptance is complete under the approved legacy-link scope; platform and installed-network admission remain separate.
- Fixed unite public previews with operation-local anonymous access and canonical home-site links; reject non-Go games and mismatched room responses before import. Preserve separately supplied player/result metadata through SGF import and Save/reopen without replacing existing source properties.

- Added the Fox kifu center: nickname/UID game lists (25 per page, 100-game batches continued by `lastCode`, correct end/empty/query-switch handling with stale responses ignored) and chessid preview; selecting a game only previews it and explicit Import goes through SGF-07. Up to eight Fox-only recents and the last lookup persist atomically and can be cleared; no responses or credentials are stored. The free-form Fox command fetch path was removed.
- Added atomic Direct/System/Manual provider network settings, native per-URL proxy resolution, cancellable Yike/Fox previews with route diagnostics, and explicit identity-fenced SGF-07 import. Provider requests share bounded retry/timeout and exit-cleanup ownership; platform and installed-network acceptance remain separate gates.
- Added native Windows readboard EXE selection/persistence and crate-owned Start/Stop/Restart with exact ready/version220430 handshake, cancellable startup, typed failures, stale-generation rejection and shared application-exit cleanup. Historical fixed-runtime readiness/restart/exit smoke passed on the original Windows candidate `eadb577` recorded in DEVELOPMENT, without an engine or current-game mutation; this is not a new native run on the PR candidate. Ongoing target sync remains separate and OCR remains unsupported.
- Integrated draggable workspace proportions, independently persistent rails, native window geometry/reset, atomic window pinning, and the Classic/High Contrast appearance controls. Restored legacy preference defaults after combining the visibility and pin DTO changes; integration acceptance remains distinct from inherited feature evidence.
- Fixed the integrated exit fence to wait for theme, rail visibility and window pin writes as well as layout and geometry, and keep all R7 controls frozen through dirty confirmation and final-save failures.
- Fixed Windows pin readback to use actual Win32 state and restore saved intent even when Tao's cached flag already matches; native apply now checks the OS result before persistence.
- Scoped the Save As classifier import to its non-Windows dialog helper and removed the reported Rust formatting differences without changing document, trial, analysis, or SGF behavior. Both-platform strict lint and the actual Windows Open/Save As/reopen smoke pass. Synchronized the cleanup-failure test with actual controlled-engine search receipt; desktop-lib passes all146 cases in parallel and serial runs without changing production analysis behavior.
- Completed R7 integration acceptance: all seven workspace/window/appearance owners are Accepted, Windows A01–A18 pass, repository checks and final dual-axis review pass. macOS native acceptance remains user-approved SKIPPED; platform support and release gates are unchanged.
- Added rectangular 2–25-axis boards across SGF, rules, main/sub-board rendering and KataGo analysis, plus shared New/Clear/Set Board Size parameters and durable new-document dimension/komi defaults.
- Added complete SGF tree navigation and exact-path links from variations, charts and problem lists. Move jumps count pass while skipping setup/comment nodes; selected SGF labels and marks follow the cursor, and stale-document navigation is rejected.
- Added five-entry durable native SGF/GIB history with safe reopen, full-path tooltips, narrow Clear, and explicit persistence-failure Retry. Successful opens remain installed when history persistence fails.
- Added shared, bounded Undo/Redo for moves, passes, personal comments and variation deletion, with exact cursor restoration, Save-aware dirty state and preservation of unrelated analysis.
- Added confirmed subtree deletion, reversible promotion through every branching ancestor and navigation back to the main trunk. Root deletion uses the safe New flow; structural changes fence stale analysis by generation and preserve attached analysis with the moved nodes.
- Added childless-root black/white/erase/clear/player-to-play setup drafts and confirmed conversion of the selected board into a root position. Each commit is one Undo/Redo step; Cancel preserves the original tree and source path.
- Added durable coordinate and all-move-number controls shared by menus, C/M shortcuts and Preferences. Numbering follows surviving stones on the selected branch, including captures, setup and pass; cancelled or failed preference writes retain the saved display state.
- Trial move and pass actions now use the same accepted-move sound classification as document play; scoring corrections remain silent.
- Removed the legacy force-replace IPC and frontend wrapper. Native current-game replacement uses the shared candidate-validation and Save / Discard / Cancel workflow.
- Added manual continuous current-node analysis with real KataGo progress, fixed 600-second search budgets, shared-Run queue liveness, target-final cancellation and bounded Run-failure cleanup. Accepted snapshots remain available to ordinary SGF Save and current-game recovery.
- Added durable default-on continuous-analysis intent, manager-owned latest-node following, and contextual Start/Stop/Resume with limit, finite-job, error and document-departure inhibition.
- Added durable independent continuous time/visits budgets and empty-board stopping, with explicit limit causes, retained results, fresh-budget Resume and write-before-replacement safety.
- Personal comments preserve active analysis admission. Document snapshot ordering protects newer annotations and Save state from delayed analysis events.
- Added explicit current-game analysis scopes with authoritative previews, finite total-visit tasks, progress, terminal cancellation and semantic/Run invalidation. Ctrl+B starts a one-visit overview; analysis results remain attached to their SGF nodes.
- Analysis tasks support immediate Pause with target-final cleanup and Continue on the same Run with a fresh Job, retaining completed positions and restarting interrupted work at its full budget. Paused tasks reserve their lane; Cancel, semantic edits, Run replacement and confirmed departure prevent continuation.
- Added independent, durable task search-time, total-visit and leading-candidate-visit conditions with OR stopping, observed ending causes, per-position query identity and target-final cleanup. Preset writes leave active task conditions unchanged.
- Added an all-position two-stage task strategy: a 32-visit overview of every target precedes an independently persisted deep pass with a 500-visit minimum, stage-local Pause/Continue progress, retained overview summaries, and Ctrl+Shift+B access.
- KataGo JSONL parsing now clamps only epsilon-sized winrate roundoff at 0/1 before strict frame validation; material out-of-range values remain invalid.
- Grouped swing-analysis admission into a named Rust request while preserving stage budgets, target identities and continuous no-result failure handling. Simplified equivalent protocol/history checks and lifecycle test result handling for strict lint checks.
- Boxed desktop engine-command errors and grouped replacement/exit requests for strict workspace lint checks, preserving serialized errors and Save / Discard / Cancel ordering.

### 中文

- 新增离线功能搜索：帮助菜单、常驻工具栏和 registry-owned Ctrl/Command+K 共用 catalog，注册动作与精确设置目标显示本平台快捷键及禁用理由，调用原 owner，取消恢复合法来源焦点。
- 新增激活后精确目标聚焦，包括 Game Info 默认贴目输入和取消返回原搜索会话；棋谱元数据及历史合同保持不变。
- About 读取当前构建身份，集中配置源码／Issues／Help 地址；未配置发行／更新／支持渠道不可执行。Cargo repository 改为 Next 源码仓库，产品及 app-data identity 不变。
- 校正 N 的提示状态，保留 Ctrl+Home New、无 Ctrl+N 和真实人机新局／续弈 owner。搜索首消费者使用完整中文基础资源与确定性逐键 locale fallback，不新增最终语言名单或语言持久化。
- 新增独立 Tencent 棋谱中心：username／chessId 查询、可取消预览、25局分页与游标续取、最多8条查询历史及显式 SGF-07 导入。完整树正规化保留棋谱元数据和有效贴目，保存重开一致；远程请求消费共享网络策略。
- Tencent 查询历史写入跨来源面板切换保持顺序；清除或查询保存失败后，重开中心仍可见并重试原操作。
- 新增 Yike Start sync／Play & Sync：Rust 唯一只读会话、有界失败暂停与 Retry、Stop 后恢复编辑、同手数来源修订、独立同步偏好及系统浏览器交接；Save As 保存调用时快照，期间新帧仍保持未保存。
- 新增 readboard 只读持续同步，复用同一外部同步 owner：首帧经一次 SGF-07 决策开始，后续帧按冻结 Java 的恢复/手番规则追加真实着手或重建静态布子；断线暂停并保留最后正确棋盘，Retry 重启 runtime，Stop 恢复编辑。always-sync、focus、mute、jump-to-last 独立持久化。Windows 源码运行历史证据覆盖真实野狐观战房间和 R10 跨来源集成，保留 DEVELOPMENT 中各原候选的归属，不代表 PR 候选新增 native 验收；installed 网络证据单独归账。
- 修复 readboard 换房间后沿用旧房间手番：重建根节点按帧内可信最后一手标记确定轮次，前一次同步留下的根 PL 也会被覆盖；无可信标记的帧仍按冻结 Java 规则保留基线。
- 修复同步启动取消与受保护棋局替换之间的竞态：进入提交阶段后保留待安装候选，readboard 的“取消启动”在该阶段禁用。
- 丢弃 readboard 待安装候选时重置快照上下文，Sync 控制保留候选；取消清理失败不覆盖原始启动错误，清理失败诊断仅用于对应 runtime 阶段。
- Windows 仅在启用 WPAD、未配置 PAC URL 且返回 `12180` 时继续判断静态系统代理及 bypass；显式 PAC 和已选路由失败仍为终止错误。修正原生 request-error 回调常量，使 WinHTTP 错误进入该决策。
- 修正受控引擎握手取消测试：首次未确认回收时验证有界 Retry Stop，保留150ms回收预算及生产行为；新增零预算用例，检查 reservation／profile 占用保护与确认回收后才释放的契约。

- 新增 Yike 公共中心：Recommend／Local 分页、可取消的选局与 URL 预览、完整来源树的 SGF-07 一次性导入及公开 locator 持久化。按批准的历史链接范围修订完成当前公开来源业务验收；平台与 installed 网络仍单独准入。
- 修复 unite 公开预览：使用操作内匿名访问及主站网页链接，在导入前拒绝非围棋与房间身份不匹配的响应；将独立返回的棋手与结果补入 SGF，导入和保存重开后仍保留，且不覆盖来源已有字段。
- 新增野狐棋谱中心：按昵称、UID 查询棋谱列表（每页 25 局、每批最多 100 局按 `lastCode` 续取，尾页／空结果／换查询均正确，旧响应不覆盖），按 chessid 直接预览；选局只预览，显式导入经 SGF-07。最多 8 条野狐专属最近查询及最后查询值原子保存、可清除，不保存响应或认证信息。移除旧的 Fox 命令字符串获取路径。
- 新增原子持久化 Direct／System／Manual 网络设置、原生按 URL 代理解析、可取消的 Yike／Fox 预览及脱敏路由诊断；显式导入经身份校验进入 SGF-07，请求统一受重试、超时及退出清理边界约束。平台与 installed 网络验收仍单独准入。
- 新增原生 Windows readboard EXE 选择/持久保存及 crate-owned 启动/停止/重启；以 ready/version220430 握手确认就绪，支持取消启动、typed 故障、旧代次隔离和共享应用退出清理。固定真实 runtime 的就绪/重启/退出 smoke 历史结果归属 DEVELOPMENT 中的 Windows 原候选 `eadb577`，无需引擎且不改变当前棋谱，不代表 PR 候选新增 native 运行；持续目标同步仍单独准入，OCR 保持不支持。
- 集成可拖动工作区比例、两侧栏独立持久显隐、原生窗口几何与独立重置、原子置顶及 Classic／High Contrast 外观；修复显隐与置顶 DTO 合并时遗漏的旧配置默认值。集成验收与前置票据历史证据分别记录。
- 修复集成退出门禁：除布局和窗口几何外，还等待主题、侧栏显隐和置顶写入；脏棋谱确认及最终保存失败期间，所有 R7 控件保持冻结。
- 修复 Windows 置顶读回：读取 Win32 实际状态，并在 Tao 缓存标志已匹配时仍恢复已保存意图；持久化之前检查原生设置操作的结果。
- 将 Save As classifier import 限定在非 Windows 对话框函数内，并消除已报告的 Rust 格式差异，不改变棋谱、试下、分析或 SGF 行为。两端 strict lint 与真实 Windows Open／Save As／reopen smoke 通过；cleanup failure测试等待可控引擎实际接到search再注入取消故障，桌面库并发／串行均146项通过，未改生产分析行为。
- 完成 R7 集成验收：七个工作区／窗口／外观 owner 全部 Accepted，Windows A01–A18、repository 检查和最终双轴审查通过；macOS 原生验收按用户批准保持 SKIPPED，平台支持和发布门禁不变。
- 新增宽高各 2–25 的矩形棋盘，贯通 SGF、规则、主副棋盘与 KataGo 分析；新建／清空／设置棋盘大小共用参数表单，并持久化新建宽高与贴目默认值。
- 新增完整 SGF 树及变化、图表、问题列表的精确路径导航；跳手计入 pass、跳过 setup/注释节点，标签与标记随选点同步，过期文档请求不会改变选择。
- 新增最近五个原生 SGF/GIB 的持久历史，支持安全重开、完整路径提示、单独清空和写失败显式重试；历史写入失败不撤销已打开的棋谱。
- 落子、pass、个人评论和变化删除共用最多 100 条撤销／重做历史，恢复精确游标；保存保留历史并建立 dirty 保存点，反转保留无关的新分析。
- 新增后续节点删除确认、多级祖先主线提升及返回主干导航；根节点删除走安全新建流程。结构编辑通过 generation 拦截旧分析，提升保留节点附带的分析并可撤销／重做。
- 新增无后续根局面的黑白子、擦除、清空、执色草稿及当前局面确认转换；一次提交对应一步撤销／重做，取消不改变原树与源路径。
- 坐标与全部手数设置现已持久化，菜单、C/M 快捷键与设置面板共享保存值；编号按当前真实分支的存活棋子来源显示，正确处理提子、setup 和 pass，取消或写入失败保留原设置。
- 试下落子与虚手复用正式落子的成功动作声音分类；计分修正保持静默。
- 移除旧强制替换 IPC 及前端 wrapper；原生当前棋谱替换统一使用候选验证与保存/放弃/取消流程。
- 新增手动连续当前节点分析：接入真实 KataGo 进度、固定 600 秒搜索预算、共享 Run 排队保活、目标 final 取消与有界故障清理；已接纳快照可普通保存到 SGF，并纳入当前棋谱恢复。
- 连续分析意图默认开启并持久化；由 manager 跟随最新节点，统一开始/停止/继续动作，并保留到限、有限请求、错误及离开棋谱后的自动工作抑制。
- 连续分析支持独立持久化的时间/visits 预算及空棋盘停止；显示到限原因、保留结果，继续时创建新预算，设置落盘成功后才替换搜索。
- 个人评论保留正在运行的分析准入；文档快照排序防止延迟分析事件覆盖新批注与保存状态。
- 新增当前棋谱显式分析范围、Rust 权威预览、总访问预算任务、真实进度、终态取消及语义/Run 失效状态；Ctrl+B 启动一访问概览，结果保存在对应 SGF 节点。
- 分析任务支持暂停／继续：等待目标 final 清理后进入 Paused，在同一 Run 上以新 Job 保留完成位置并为中断位置恢复完整预算。暂停任务继续占有整局通道；取消、语义编辑、Run 替换及确认离开会终止继续资格。
- 分析任务新增独立持久化的搜索时间、总 visits 与首选候选 visits 条件，任一到限即停止实际查询；记录观测到的结束原因，以逐位置查询身份和目标 final 清理保护完成进度。预设写入不改变当前任务预算。
- 新增全位置两阶段分析任务：先以 32 visits 概览全部目标，再按独立持久化且不低于 500 visits 的深度预算逐点分析；暂停／继续保留阶段进度与概览摘要，并支持 Ctrl+Shift+B 启动。
- KataGo JSONL 解析在严格帧校验前仅校正 0/1 附近的微小胜率浮点误差；明显越界值仍被拒绝。
- swing 分析准入改用具名 Rust 请求，保留阶段预算、目标身份与连续分析无结果失败处理；对协议／历史检查及生命周期测试结果处理做等价 lint 修正。
- 桌面引擎命令错误采用 Rust 装箱返回，替换／退出使用具名请求，通过严格工作区 lint 检查；保留错误序列化内容与保存／放弃／取消顺序。

## [0.1.0] - 2026-05-01

### English

Initial public Tauri 2 release candidate for the LizzieYzy Next desktop mainline.

Added:

- Tauri 2 + Rust + React/TypeScript desktop workspace.
- SGF import, open, parse, replay, edit, serialize, save, and Save As flows.
- KataGo engine profile setup, asset checks, one-position analysis, full-game analysis, progress events, and cancellation.
- Board, winrate, candidate move, PV, ownership, policy, and review mark UI surfaces.
- SQLite-backed analysis cache with browser-preview fallback behavior.
- Yike/Fox provider and readboard sidecar runtime contracts with offline validation.
- Multi-platform release workflow for macOS, Windows, and Linux CI-built assets.
- Bilingual release notes in English and Chinese.

Fixed before release:

- Invalid SGF parsing now clears stale review data instead of showing old candidate/review marks.
- Cache status no longer reports frame count as impossible move count such as `21/20 moves`.
- Provider and readboard warnings are visible in the UI instead of hidden behind counts or hover-only text.
- English UI no longer contains the previous Chinese-only full-game analysis tooltip.

Verified:

- Scaffold validation.
- Release asset preflight.
- Production release workflow contract validation.
- Frontend production build.
- Rust formatting, clippy, and workspace tests.
- Local macOS Tauri bundle build.

Known limits:

- CI release assets are unsigned unless repository signing/notarization secrets are configured.
- Live Yike/Fox external-service checks and real readboard sidecar hardware checks require maintainer environments.
- This release establishes the new Tauri mainline baseline; it is not a blanket claim of full Java/Swing legacy parity.

### 中文

LizzieYzy Next 桌面主线的首个公开 Tauri 2 发布候选版本。

新增：

- Tauri 2 + Rust + React/TypeScript 桌面工作区。
- SGF 导入、打开、解析、回放、编辑、序列化、保存和另存为。
- KataGo 引擎配置、资源检查、单点分析、全局分析、进度事件和取消。
- 棋盘、胜率图、候选点、PV、ownership、policy 和问题手标记界面。
- SQLite 分析缓存，并支持浏览器预览 fallback。
- Yike/Fox provider 和 readboard sidecar 运行时契约与离线验证。
- 面向 macOS、Windows、Linux 的多平台 CI 发布 workflow。
- 中英双语 release notes。

发布前修复：

- 无效 SGF 解析失败后会清空旧复盘数据，不再继续显示上一局候选点和问题手。
- 缓存状态不再把 frame 数错误显示为 `21/20 moves` 这类不可能的手数。
- Provider 和 readboard warning 会直接显示在界面中，不再只显示数量或依赖 hover。
- 英文界面中移除了原先中文-only 的全局分析 tooltip。

已验证：

- scaffold 校验。
- release asset preflight。
- production release workflow 契约校验。
- 前端生产构建。
- Rust format、clippy 和 workspace tests。
- 本机 macOS Tauri bundle 构建。

已知限制：

- 除非仓库配置签名/公证 secrets，否则 CI release 资产是未签名包。
- 真实 Yike/Fox 外部服务验证和真实 readboard sidecar 设备验证需要维护者环境。
- 本版本建立新的 Tauri 主线基线，不等于对 Java/Swing 旧主线全部细节作 100% 等价承诺。
