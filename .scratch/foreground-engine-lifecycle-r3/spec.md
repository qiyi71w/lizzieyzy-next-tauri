# Foreground Engine Lifecycle R3（ENG-02–ENG-07）实现规格

**Status:** ready-for-agent

## Problem Statement

当前产品把 Engine Profile 的编辑选择、分析命令的进程启动和用户眼中的“已加载引擎”混在一起。选择或填写一个 Profile 只代表配置可用，却会被界面当作引擎已就绪；一次选点分析或全局分析会直接从可变的 Profile DTO 启动各自的 KataGo 进程，系统没有一个权威、常驻且可观察的 Foreground Engine Run。

因此，用户无法可靠地知道当前到底是哪一个引擎实例在服务，也无法安全地执行 Start、Stop、Restart 或 A → B 切换。编辑活动 Profile、引擎异常退出、切换失败、迟到的进程事件以及迟到的分析结果，都缺少稳定身份和事务边界。Autoload Default 也不存在，错误主要以无类型字符串暴露，用户无法在保留当前工作区的前提下进行明确的手动恢复。

R3 必须在不重新裁决既有 ADR、Disposition、Accepted Parity Item 或迁移合同的前提下，建立 Foreground Engine Run、Analysis Job、Autoload Default、事务化切换和手动恢复的统一生命周期，并用真实 `KataGoAnalysis` 路径证明它。

## Solution

由 `engine-manager` 成为 Foreground Engine Run 和其 Analysis Job 的唯一所有者。每个 Run 从一个已保存 Engine Profile 创建，持有不可变的 adapter、profile 和 capability snapshot，并通过权威生命周期快照对外呈现 No-engine、Starting、Ready、Switching、Stopping 或 Error。

主工作区新增独立的 Engine Switcher。用户在其中选择 Profile 时立即 Start 或发起事务化 Switch；Engine Settings 只编辑 Profile catalog 和零或一个 Autoload Default，不隐式改变当前 Run。专用 Stop 先取消该 Run 的所有 Job，再终止进程；显式 Restart 终止旧 Run，并从当前已保存的 Profile 数据创建全新的 Run identity。

A → B 切换期间 A 始终保持 primary，直到 B 在当前 switch identity 下完成资产校验、启动和 adapter readiness。只有当前且 Ready 的 B 才能一次性晋升；随后取消 A 的 Job、把新工作绑定到 B，再停止 A。B 失败时回滚到 A；如果 A 已不存在，则进入显式 No-engine，而不是晋升 B。所有迟到的 switch、run 和 job 事件都按身份拒绝。

selected-node Analysis Job 由 manager 分配并绑定 run、job、document generation 与精确 `NodePath`。显式取消和新请求 supersession 都在该身份边界内完成，只有当前 Run、当前 Job、当前棋谱 generation 和当前 `NodePath` 完全匹配的结果才可发布。

Autoload Default 与 Profile catalog 一起原子持久化，首次使用默认关闭。启动失败产生 typed failure 并保持 No-engine，不回退到其他 Profile。引擎异常退出会取消其 Job，保留可诊断的 Error 状态，并等待用户修复 Profile 后显式 Restart；R3 不自动重启。

## User Stories

1. 作为一名复盘用户，我希望应用在没有引擎时仍显示权威的 No-engine 状态，从而不会把一个已选配置误认为正在运行的引擎。
2. 作为一名复盘用户，我希望在主工作区看到当前 Foreground Engine Run 的真实状态，从而知道它正在启动、已就绪、正在切换、正在停止还是需要恢复。
3. 作为一名复盘用户，我希望从 Engine Switcher 选择一个已保存 Profile 后立即启动引擎，从而不必进入 Engine Settings 才能开始使用引擎。
4. 作为一名复盘用户，我希望 Engine Settings 只负责配置，从而浏览、选择或编辑 Profile 时不会意外启动或切换引擎。
5. 作为一名复盘用户，我希望 Start 自动检查所需程序、模型和配置资产，从而在启动前得到明确失败，而不是依赖一次单独的手工检查。
6. 作为一名复盘用户，我希望引擎只有在 adapter readiness 被证明后才显示 Ready，从而不会把仅仅成功 spawn 的进程当作可用引擎。
7. 作为一名复盘用户，我希望当前 Run 使用不可变的 Profile snapshot，从而运行期间的配置编辑不会偷偷改变现有进程。
8. 作为一名引擎配置用户，我希望编辑活动 Profile 后看到“存在待应用更改”，从而知道当前 Run 仍在使用旧 snapshot。
9. 作为一名引擎配置用户，我希望待应用更改只在显式 Restart 后生效，从而能够控制何时中断当前分析。
10. 作为一名复盘用户，我希望 Restart 创建新的 Run identity，从而旧进程和新进程的事件不会混在一起。
11. 作为一名复盘用户，我希望 Restart 使用当前已保存的 Profile 数据，从而修复后的路径、模型或配置可以生效。
12. 作为一名复盘用户，我希望 Restart 先取消旧 Run 的全部 Job 并终止旧进程，从而不会遗留仍可发布结果的旧工作。
13. 作为一名复盘用户，我希望主工作区有专用 Stop 操作，从而可以结束引擎而不影响棋谱复盘。
14. 作为一名复盘用户，我希望 Stop 先取消所有 Run-owned Job 再终止进程，从而不会在停止后收到有效分析结果。
15. 作为一名复盘用户，我希望 Stop 完成后回到 No-engine，从而明确知道没有常驻引擎仍在运行。
16. 作为一名引擎配置用户，我希望活动 Profile 在 Run 存在时不能删除，从而当前 Run 始终能追溯到稳定的 Profile identity。
17. 作为一名引擎配置用户，我希望停止当前 Run 或成功切换到另一 Profile 后才能删除旧活动 Profile，从而删除行为与生命周期一致。
18. 作为一名引擎配置用户，我希望可以标记零或一个 Autoload Default，从而明确控制启动应用时是否自动加载引擎。
19. 作为一名首次使用者，我希望 Autoload Default 初始为空，从而应用不会未经选择就启动本地计算资源。
20. 作为一名复盘用户，我希望应用启动时只尝试已标记的 Autoload Default，从而启动行为可预测。
21. 作为一名复盘用户，我希望 Autoload 启动失败后保持 No-engine 且显示 typed failure，从而不会悄悄回退到另一个 Profile。
22. 作为一名引擎配置用户，我希望更改 Autoload Default 不影响当前 Run，从而持久化的下次启动选择不会改变本次会话。
23. 作为一名引擎配置用户，我希望删除被标记但未活动的 Profile 时同时清除 Autoload Default，从而不会留下悬空引用。
24. 作为一名引擎配置用户，我希望 Autoload Default 持久化失败时保留先前 durable mark，从而界面不会宣称一个未保存的默认值已经生效。
25. 作为一名复盘用户，我希望从引擎 A 切换到引擎 B 时 A 在 B Ready 前继续保持 primary，从而切换准备期间仍有明确的权威引擎。
26. 作为一名复盘用户，我希望 B 只有在资产、进程和 readiness 全部成功后才晋升，从而半启动状态不会替换 A。
27. 作为一名复盘用户，我希望 B 晋升后新 Analysis Job 只绑定 B，从而新工作不会误发给即将停止的 A。
28. 作为一名复盘用户，我希望成功切换时取消 A 的未完成 Job，然后停止 A，从而旧结果不会污染新引擎上下文。
29. 作为一名复盘用户，我希望 B 启动失败时 A 保持 Ready 和 primary，从而失败切换不会破坏可用会话。
30. 作为一名复盘用户，我希望失败切换给出与 B 和本次 switch identity 关联的 typed failure，从而能够修复正确的 Profile。
31. 作为一名复盘用户，我希望当失败切换发生且 A 已不存在时看到显式 No-engine，从而系统不会把未验证的 B 当作回退方案。
32. 作为一名快速连续切换引擎的用户，我希望被后续选择取代的 switch completion 被忽略，从而最后一次用户意图获胜。
33. 作为一名复盘用户，我希望迟到的旧 Run 事件不能改变当前状态，从而已停止或已替换的进程无法复活。
34. 作为一名复盘用户，我希望“分析此手”创建 manager-owned selected-node Analysis Job，从而每次分析都有稳定身份。
35. 作为一名复盘用户，我希望 selected-node Job 同时绑定当前 Run、棋谱 generation 和精确 `NodePath`，从而结果只属于发起时的局面。
36. 作为一名复盘用户，我希望可以显式取消当前 selected-node Job，从而不必停止整个 Foreground Engine Run。
37. 作为一名复盘用户，我希望新的 selected-node 请求只 supersede 该 lane 的旧请求，从而 latest-request-wins 且不误伤其他工作。
38. 作为一名复盘用户，我希望取消或 supersede 后的迟到结果被拒绝，从而旧候选点、PV、ownership 或 policy 不会再次出现。
39. 作为一名复盘用户，我希望切换棋谱、generation 或 `NodePath` 后旧 Job 结果不再发布，从而分析呈现始终对应当前选择。
40. 作为一名复盘用户，我希望分析动作由当前 immutable capability snapshot 准入，从而系统不会为不支持的动作启动隐藏的专用引擎。
41. 作为一名复盘用户，我希望 selected-node Job 的取消或普通超时以 typed terminal outcome 结束，从而可以区分用户取消、超时和引擎崩溃。
42. 作为一名复盘用户，我希望引擎异常退出时其全部 Job 被取消，从而已经失去进程所有权的结果不能发布。
43. 作为一名复盘用户，我希望异常退出后系统停留在 Error 并等待显式 Restart，从而不会发生不可预测的自动重启。
44. 作为一名引擎配置用户，我希望 Error 状态能引导我进入 Engine Settings 修复 Profile，从而可以从明确入口恢复。
45. 作为一名复盘用户，我希望应用重启不恢复旧 Run、capability snapshot 或 Analysis Job，从而不会把已经不存在的进程身份当作会话状态。
46. 作为一名启用了 Autoload 的用户，我希望应用重启后创建全新的 Run identity，从而 Autoload 是新的 Start 而不是旧进程恢复。
47. 作为一名复盘用户，我希望真实 `KataGoAnalysis` 可以完成 Ready、selected-node 分析、取消、Stop、Restart 和两 Profile 切换，从而 R3 的主路径由真实引擎而不是纯 mock 证明。
48. 作为一名维护者，我希望所有分析调用都不再携带可变 Profile DTO 直接启动进程，从而 Foreground Engine Run 成为唯一进程身份来源。
49. 作为一名维护者，我希望 typed lifecycle/job DTO 在 Rust、Tauri 和 TypeScript 边界保持一致，从而 UI 不需要从字符串或 Profile 选择推断状态。
50. 作为一名维护者，我希望 repository evidence、真实 KataGo smoke 和 native desktop smoke 分开记录，从而不会把 fixture 结果误报为真实环境验证。

## Implementation Decisions

- `engine-manager` 是 Foreground Engine Run、候选 Run、switch identity、Analysis Job registry、进程句柄和取消顺序的唯一所有者。Tauri 只负责命令/事件适配，React 只观察 snapshot 并发送用户 intent。
- R3 只证明 `KataGoAnalysis`。Engine Adapter 抽象必须允许 immutable capability snapshot，但不得借此接受 Multi-backend Engine Profiles、Generic GTP、命名 rich-analysis adapter、SSH Engine Profile 或 Remote Compute Provider。
- 每一次 Start、Restart 或 switch candidate 都获得新的 opaque Run identity。Restart 同一 Profile 也绝不复用旧 Run identity。
- 对外 lifecycle snapshot 使用显式 tagged state：No-engine、Starting、Ready、Switching、Stopping、Error。Snapshot 还带单调 revision，供订阅者拒绝乱序 snapshot。
- Starting 中的 candidate 数据包含稳定 Profile identity、adapter kind 和完整 immutable Profile snapshot，但 capability 尚未获准。Ready 在 adapter readiness 后冻结 verified capability snapshot；Stopping 和 Error 保留该 Run 已获准的 capability snapshot。UI 不从 catalog 当前值重新构造 Run 信息。
- Switching snapshot 同时暴露仍为 primary 的 A 和正在准备的 B，以及当前 switch identity。对用户可用的 active capability 在 B 晋升前仍来自 A。
- Error 用于已经成为当前 Run 后发生的不可继续运行失败，并保留失败 Run 的 immutable snapshot 以支持诊断和 Restart。初始 Start 或 Autoload 在没有 primary 时失败，则权威状态为 No-engine，并另行发布与该尝试 identity 关联的 typed failure。Switch B 失败则回到 Ready A 并发布 B 的 typed failure。
- `KataGoAnalysis` readiness 不能由资产存在或 spawn 成功代替。资产校验和 spawn 后，manager 必须在同一常驻子进程上完成一个有界、不会进入 UI 或 cache 的最小合法 JSONL probe，并成功解析响应，才能发布 Ready。
- Start 从保存后的 Profile record 创建 snapshot，自动验证必需资产并执行 readiness。诊断性的手工 Check Assets 保留，但不是 Start 前置步骤。
- Stop 会先将当前 Run 标记为 Stopping，使新 Job 无法准入；随后发出该 Run 全部 Job 的取消，给予有界的 terminal drain，再终止 primary 和仍属于当前操作的 candidate process，最终发布 No-engine。Job 或协议无响应不得让 Stop 无限等待。
- Restart 不是 Switch。它先阻止新 Job、取消旧 Run 的 Job、终止旧进程，再按相同稳定 Profile identity 读取当前已保存 record 并创建新 Run。Restart 失败不回滚到旧进程；旧进程已经结束，结果进入带 typed failure 的 No-engine 或可恢复 Error 边界。
- Stop、Restart、新 Switch 或应用 teardown 会使较早 lifecycle operation token 失效。任何失效 operation 的完成事件只能触发其私有 candidate 清理，不能改变 primary、snapshot、错误呈现或分析呈现。
- Engine Settings 保留 `selected_profile_id` 作为编辑 identity，并新增独立的零或一个 Autoload Default identity。二者都不是 Run identity，也不因 Engine Switcher 的选择而自动重写。
- Engine Settings 不再承载 Start、Switch、Stop 或分析执行。主工作区 Engine Switcher 读取保存后的 catalog；选择非活动 Profile 立即发起 Start 或 Switch，选择当前活动 Profile 不隐式 Restart。
- 保存对活动 Profile 的修改后，当前 Run snapshot 保持不变。UI 通过保存 record 与 Run snapshot 的差异显示 pending changes，并提供显式 Restart；未保存的表单内容不参与 Start、Switch 或 Restart。
- 删除活动 Profile 的约束必须由后端 catalog owner 强制执行，不能只禁用按钮。Error 中仍持有当前 Run identity 时同样适用；用户必须 Stop 或成功让另一 Profile 成为 primary 后才能删除。
- Autoload Default 与 catalog 使用同一持久化事务。首次使用、旧格式迁移或字段缺失均解释为无 mark。写入必须采用临时文件加原子替换；任何序列化、写入或替换失败都保留先前 durable catalog 和 UI 可见 mark。
- 标记一个 Autoload Default 会在同一事务中清除旧 mark。清除 mark 不停止当前 Run；修改 mark 不 Start、Restart 或 Switch 当前 Run。
- 删除未活动但被标记的 Profile，会在同一 catalog 事务中删除 Profile 并清除 mark。若事务失败，两者都保持原状。
- 应用初始化先装载 catalog，再初始化 manager。无 Autoload Default 时保持 No-engine；存在 mark 时只尝试该 Profile，成功后创建全新 Run，失败时报告 typed failure 并保持 No-engine，不尝试 catalog 中的其他 Profile。
- Foreground Engine Run、candidate Run、switch token、capability snapshot 和 Analysis Job 都是 session-only；持久化层不得写入这些对象。应用重启后的 Autoload 是新的 Start。
- A → B Switch 使用单调 switch identity。A 在 B 的资产校验、spawn 和 readiness probe 期间保持 primary。期间旧 A Job 可以自然完成，但新 Job 仍绑定 A，直到 B 原子晋升。
- 只有当前 switch identity 的 Ready B 可以晋升。晋升事务先把 primary identity 和 capability admission 切到 B，再拒绝 A 的新 Job、取消所有 A-owned Job，最后停止 A。对外 snapshot 不得出现 B 已 primary 而新 Job 仍绑定 A 的中间状态。
- B 的资产、spawn、readiness、protocol、timeout 或 exit 失败都清理 B candidate 并保留 Ready A。若 A 在事务结束前已不再存在，结果为 No-engine；任何失败分支都不得晋升 B。
- 后续 Switch 会 supersede 较早 Switch。被 supersede 的 B 即使之后 Ready、失败或退出，也只能按旧 switch identity 被拒绝和清理。
- selected-node Analysis Job 的 start contract 捕获 opaque Run identity、opaque Job identity、当前 authoritative game generation 和精确 `NodePath`。Tauri current-game owner提供并校验 generation/`NodePath` scope；React 不以 SGF 文本或 move number 代替该 scope。
- manager 为 selected-node lane 维护至多一个当前非 terminal Job。新请求先 supersede 并取消旧 Job，再建立新 Job identity。显式 Cancel 必须按 Run 和 Job identity 定位，不能只依靠“当前正在运行”布尔值。
- `KataGoAnalysis` 的 Job cancel 使用 adapter 的协议级取消能力，使正常取消不会终止健康的 Foreground Engine Run。若协议或进程状态已不可信，则以 typed process/protocol failure 使 Run 进入 Error，而不是悄悄启动替代进程。
- 每个 Job 事件都携带 run、job、document generation、`NodePath`、terminal outcome 和必要的进度/结果 payload。manager 首先拒绝 stale run/job；Tauri 与 React 的 publication fence 再要求当前 game generation 和精确 `NodePath` 匹配。
- cancelled、superseded、failed 或属于旧 Run 的 Job 永远不能发布候选点、PV、ownership、policy、cache 写入或状态回跳。现有 accepted `UI-03` hover/stale 行为保持不变。
- selected-node supersession 只影响 selected-node lane，不得静默取消 whole-game 工作。R3 要把现有 whole-game caller 从 profile-to-process spawn 迁到当前 Run 和 manager-owned registry，以便 Stop/Restart/Switch/Crash 能取消 Run-owned Job；独立 lane、完整进度和呈现验收仍归 R4。
- 所有分析动作先读取当前 Ready Run 的 immutable capability snapshot。能力不足时在进程调用前返回 typed unsupported outcome；不得启动隐藏 dedicated engine，也不得从 Engine Settings 当前 Profile 临时 spawn。
- failure DTO 至少区分 start、asset、readiness/protocol、nonzero exit、timeout、cancellation 和 unsupported capability，并携带 operation、相关 run/switch/job/profile identity、稳定 kind、用户可读 message 与可选诊断摘要。Cancellation 是正常 terminal outcome，不应伪装成进程 crash。
- unexpected process exit 只影响与该 process identity 匹配的当前 Run。它取消所有 Run-owned Job，发布 typed failure，进入 Error，并等待显式 Restart。旧 Run 或 candidate 的 exit 不得覆盖当前状态。
- UI 在 Error 中提供 Restart、Stop 和打开 Engine Settings 的恢复入口。R3 不提供自动重启、指数退避或自动 fallback。
- lifecycle API 提供初始 snapshot 查询、intent commands 和 snapshot/event subscription。Frontend 采用 subscribe-then-read，并以 snapshot revision 与 identities 合并结果，避免订阅建立期间漏掉状态变化。
- Rust 先定义 lifecycle、run、capability、job、failure 和 catalog DTO；TypeScript 使用同构 snake_case wire shape。UI 状态不得再由配置字段完整性、`canRun`、`engineLabel` 或本地 request token 推断进程身份。
- 现有 profile DTO 仅用于 catalog 编辑和 manager 内部 snapshot 建立。完成 caller cutover 后，删除接受 Profile DTO 并直接 spawn 的 one-shot/batch Tauri 命令、batch-only Job registry 以及对应兼容 wrapper；不得留下双重进程所有权路径。
- 浏览器 preview 的 fake review/analysis 保持非权威且不创建 Foreground Engine Run。它不能作为 R3 repository 或 live KataGo evidence。
- 实现按 Migration Phase 的 Delivery Order 交付：先完成 `ENG-02`；随后可推进 `ENG-05`、`ENG-06`、`ENG-07`；`ENG-03` 在 `ENG-02` 后即可开工，其排位仅是 Delivery Order；`ENG-04` 必须在 `ENG-03` 后完成。

## Testing Decisions

- 好的测试只观察公开 lifecycle intent、snapshot/event、typed outcome、process I/O 顺序和最终用户可见状态，不断言 mutex、线程、私有 enum、内部容器或具体函数拆分。
- 首选且主要的 repository seam 是 `engine-manager` 的公开 Foreground Engine coordinator，背后使用一个可脚本化的 `KataGoAnalysis` JSONL 子进程。测试通过真实 stdin/stdout/process-exit 边界驱动 readiness、延迟响应、取消、崩溃和迟到 completion，并只断言公开 snapshot/event。这一 seam 覆盖绝大多数 ENG-02–ENG-07 行为，避免为每个状态新增低层 seam。
- 复用现有受控 fake executable 的 prior art：当前已有 JSONL response、batch progress、cancel token、timeout、nonzero exit、stderr 和 asset path 测试。将同一种进程 fixture 扩展为可阻塞 readiness、记录命令、延迟 terminal event、接受协议取消以及模拟 A/B 两个进程，不建立第二套 mock process framework。
- coordinator 测试覆盖 No-engine → Starting → Ready、Ready → Stopping → No-engine、Ready/Error → Restart、新 Run identity、immutable snapshot、pending Profile 修改不影响 Run，以及活动 Profile 删除拒绝。
- coordinator 测试显式验证顺序：Stop/Restart/Switch/Crash 先取消 Run-owned Job，再结束或替换进程；B 晋升后新 Job 绑定 B，A Job 被取消，之后才停止 A。
- switch 测试覆盖成功 A → B、B asset failure、spawn failure、readiness/protocol failure、timeout、B exit、被后续 switch supersede 的迟到 Ready/Failure、旧 A exit，以及 A 已不存在时失败返回 No-engine。
- selected-node fixture 覆盖 start、explicit cancel、completion、supersession、protocol cancel 保持 Run Ready、旧 Job 迟到结果、旧 Run 迟到结果，以及 run/job/generation/`NodePath` 任一不匹配时不发布。
- typed failure 测试逐类证明 start、asset、protocol/readiness、nonzero exit、timeout、cancellation 与 unsupported capability 的 kind、scope identity 和状态效果；不要只匹配人类可读 message。
- catalog persistence 测试覆盖旧格式无 Autoload 字段、首次使用 off、设置/替换/清除 mark、删除 inactive marked Profile、活动 Profile 删除 guard，以及序列化/写入/原子替换失败时 durable 与可见状态保持原值。
- wire boundary 只保留小而必要的 DTO serialization tests，延续现有 Rust snake_case DTO round-trip prior art；它们证明 Run/Job/failure/catalog identity 不在 Rust/TypeScript 边界丢失，不重复 coordinator 行为测试。
- Tauri gateway 测试验证 command 只委托 manager、初始 snapshot 加订阅不会漏事件、事件携带完整 identity，以及 obsolete profile-to-process commands 不再注册。Tauri 测试不重新实现 manager 状态机。
- 最高 UI seam 使用 rendered application 配合 mocked frontend API，延续现有 application-level test prior art。它覆盖 Engine Switcher 与 Engine Settings 分离、状态标签、Stop/Restart 可用性、pending changes、活动 Profile 删除反馈、typed recovery 入口、Autoload mark 保存失败回滚，以及 stale Job 事件不恢复分析呈现。
- 不为纯视觉文案建立脆弱快照；UI 测试优先使用 role、可操作状态和用户结果。
- `katago-protocol` 既有 query/response normalization 测试继续负责 JSONL 正确性。R3 测试只验证 resident adapter 使用该协议、readiness 和 identity/cancellation，不重复 `ANA-01` 的分析正确性。
- 真实 KataGo smoke 与 repository fixture 分开记录。真实 smoke 至少证明：单 Profile 达到 Ready、selected-node Job 可完成与取消、Stop、修复/Restart 产生新 Run，以及两个真实 Profile 完成 A → B Switch。
- native desktop smoke 使用真实 KataGo assets，至少覆盖 No-engine 启动、单引擎 Start/Stop/Restart、Engine Settings 编辑不改变当前 Run、Autoload off/on 与失败、两 Profile 成功/失败切换、Error 后 manual recovery，以及活动 Profile 删除 guard。
- fixture-only 通过不能被描述为真实 KataGo 或 native evidence；真实 KataGo smoke 也不能替代 deterministic rollback/stale-identity repository coverage。
- R3 exit 前保持 accepted `ENG-01` profile/asset evidence 和 accepted `UI-03`/`UI-04` 行为继续通过。共享 DTO 或命令边界确有必要时才运行一次广域集成验证；每个切片优先运行最窄的 crate、wire 和 rendered-app 测试。

## Out of Scope

- 不重新打开或扩展 Accepted `ENG-01` 的 Engine Profile 与 asset-check 合同。
- 不实现 `ENG-08` Profile catalog 手工排序。
- 不接受 `ENG-09` Multi-backend Engine Profiles 或 `ENG-10` Generic GTP Game Adapter；已有 enum/interface 不能被当作这些 Parity Item 的证据。
- 不实现 SSH Engine Profile、Remote Compute Provider、Contribution Run 或其凭据/网络策略。
- 不实现 Match Session、Match Reservation、Human-vs-Engine、Engine-vs-Engine、Compute Budget 或任何对局落子所有权。
- 不完成 R4 的 `ANA-01`–`ANA-04` 产品验收，包括 exact-position admission 的完整分析合同、whole-game 独立 lane、完整进度模型、cache/presentation 绑定和候选呈现能力；R3 只建立身份、取消、能力准入基础并迁移旧 spawn caller。
- 不实现 continuous current-node analysis、batch SGF queue、SGF analysis exchange 或 named rich-analysis adapters。
- 不恢复或持久化 Foreground Engine Run、candidate、capability snapshot、switch identity、Analysis Job 或 live analysis state。
- 不记忆或自动加载“上次 primary”；Autoload 只认零或一个显式 mark。
- 不提供自动重启、自动 fallback、后台预加载额外 Profile 或隐藏 dedicated analysis engine。
- 不复刻 Swing 的线程结构、进程表、诊断窗口、quarantine presentation、GTP restore 分类或配置文件格式。
- 不改变 personal comment、winrate chart、Sub-Board、provider/readboard、release 或 current-game replacement 的既有 ADR/Disposition。
- 不把 R5 的完整 Safe Graceful Shutdown 纳入 R3；R3 只要求 manager 提供可由应用 teardown 调用的取消/停止边界，并保证退出时不恢复 Run/Job。
- 不把 browser preview 或 fake analysis 描述为 native Foreground Engine Run。

## Further Notes

- 本规格中的 Foreground Engine Run、Engine Adapter、Engine Profile、Analysis Job、Autoload Default、`NodePath`、Parity Item、Migration Phase、Item Start Prerequisite 和 Delivery Order 均沿用项目既有领域含义，不使用 Profile selection、move number 或 UI boolean 代替 identity。
- R3 的 Migration Phase Gate 已由 R2 退出满足；`UI-02` 尚余的“board mutation promise 未完成时接收 engine event”证据缺口不阻塞 R3，也不属于本规格的重新验收范围。
- R3 只有在 `ENG-02`–`ENG-07` 全部 Accepted 后退出。Repository evidence 必须覆盖生命周期、immutable snapshot、Autoload、事务切换/回滚、删除 guard、typed failure、取消和 stale rejection；此外还必须分别保留真实 KataGo 与 native desktop evidence。
- `KataGoAnalysis` 是 R3 的真实证明路径，不代表未来所有 Engine Adapter 已被接受。
- 完成实现后，应更新各 Parity Item 的 repository evidence、live/environment evidence 和 remaining gap；任何一类证据都不能代替另一类。
