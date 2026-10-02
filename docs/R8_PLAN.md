# R8 — Engine Adapters 迁移与验收计划

Status: R8 runtime acceptance complete on 2026-10-02; ENG-09 / ENG-10 Accepted. Exact runtime candidate `42cd91a657d2710b826cc1b94f21f168de64a0fa`; final review and documentation commit are recorded in ticket 05. Ticket 06 is the separate read-only closeout.

## 1. 结论与基线

R8 只完成 **ENG-09 多后端 Engine Profile / Capability Snapshot** 与 **ENG-10 Generic GTP Game Adapter**。目标是让现有引擎管理链路安全承载第二种协议，并给 R9 留下已验证的取步接口；不是提前做对局产品。

- 仓库：`qiyi71w/lizzieyzy-next-tauri`；WSL 为提交源。
- 调查基线与首个实现起点：`a0c8ed370f620341c863a3bb8fd4936710d82e72`，R7 最终收尾提交。
- 规划分支：`docs/r8-engine-adapters-plan`。
- 规划工作树：`/home/dev/dev/weiqi/worktrees/lizzieyzy-next-tauri/r8-planning-20260930`；规划时 HEAD 与上述调查基线相同，不是后继实现的当前起点。
- 本次继承 R7 完成记录，不将规划目录、历史 R6 分支或另一次解析的 `origin/main` 当成实现起点。原始工作树未提交内容不迁入。
- Java 参照仍是 Migration Baseline v1：`7b4027531c2b26062d0bfc27a040cc550cfbea4d`。后续 Java 变更不自动扩域。
- 权威顺序： [Architecture](ARCHITECTURE_NEXT.md) → [Migration Plan](MIGRATION_PLAN.md#r8--engine-adapters) → [Parity Matrix](PARITY_MATRIX.md#r8--engine-adapters) 的具体条目与验收。Matrix 独占 Item Start Prerequisites；本计划的工作包依赖不改写它们。
- 完整行为规格：[spec.md](../.scratch/engine-adapters-r8/spec.md)。本文保留代码定位、执行拆分和证据安排；规格不绑定易变实现文件。

### 调查基线时的进入条件（历史）

| 门禁 | 本基线状态 | 对 R8 的含义 |
| --- | --- | --- |
| R8 Phase Gate：ENG-02–ENG-07 | 全部 Accepted | 复用 run、switch、cancel、autoload、manual recovery；不再造第二个生命周期 owner |
| ENG-09 Item Start：ENG-01、ENG-02 | Accepted | 可以开始配置与 capability cutover |
| ENG-10 Item Start：ENG-09、ENG-02 | ENG-09 Missing，ENG-02 Accepted | ENG-10 的正式条目启动按 Matrix 等待 ENG-09；允许提前做不交付产品的协议资料/环境准备 |
| R7 收尾 | 已完成 | 保留 R7 用户数据、窗口与偏好行为；其 macOS SKIPPED 不是 R8 的豁免 |
| R9 Phase Gate | ENG-09/ENG-10 尚未 Accepted | R8 完成前不启动 Match Session 产品实现 |

上表记录规划时的 `Missing`，不覆盖后继验收。当前 ENG-09 已由 02 接受，ENG-10 的最终组合证据见 §10 与 DEVELOPMENT §2.4；ENG-01 的历史范围不变。

## 2. 保留范围与本轮增量

| 原有约束 | 保留 | R8 补充 |
| --- | --- | --- |
| ENG-01 的 KataGo profile 历史 | 不改 ID、Accepted 或历史证明范围 | 一份新目录格式承载多个 adapter，读取既有持久化数据 |
| 一个 run 只说一种协议 | KataGoAnalysis 继续 JSONL | GenericGtp 只说 GTP；同一 KataGo 二进制的两个模式也是两个 run |
| immutable snapshot | profile/catalog 编辑不原地改变 live run | args、adapter settings 和完整 capability 均进快照与 pending 比较 |
| A → B 事务切换 | A 保持权威到 B Ready；失败回滚；last intent wins | 跨 JSONL/GTP 同样成立，失败 B 不清空 A 的工作 |
| unsupported-before-mutation | 后端最终把关 | 按能力而非 Ready/引擎名称禁用各分析入口，并解释缺失能力 |
| exact position | board/rules/komi/side/setup/history 均不得简化 | 独立 admission plan；GTP 不可表达的 setup/规则明确拒绝 |
| Compute Budget | 每步 wall deadline，不是 Competitive Clock | 支持时映射 GTP 时间命令；不伪造 visits 或超时负结果 |
| stale identity | run/job/game 身份不匹配不得发布 | GTP command ID 与业务 job identity 分离；迟到 genmove 无效 |
| 不恢复运行态 | 恢复棋谱不恢复 run/job | 能力、协议连接、同步状态和剩余预算也不持久化 |

明确不做：R9 新局/续弈、人机/PK 界面、Match Reservation、对局 SGF/结果写入；ANA-09 rich-analysis；ENG-08 排序；SSH-01、RCOMP-01；Java 配置导入；引擎下载/打包/签名/安装器；竞技时钟、自动认输、HumanSL、批量 PK、贡献、外部棋盘回写。

这些都是各自 owner 的后续工作，不因添加 GTP 接口自动获得接受历史。

## 3. 当前缺口与代码落点

下面是固定基线的定位依据，不要求在原大文件继续堆逻辑。实施前检查当前定义和引用；导出类型变更先走可用 LSP。

| 关注点 | 现有定位 | 事实与改动方向 |
| --- | --- | --- |
| profile / backend / capability | `crates/app-model/src/lib.rs:378-394,453-468` | profile 无参数向量；KataGo model/config 在共享结构；enum 已有 GenericGtp 等值，但 capability 只有 selected-node、whole-game、root-score、protocol-cancel。存在 enum 不代表适配器可用 |
| 目录与旧数据 | `crates/engine-manager/src/catalog.rs:17-36,96-145` | 当前多 profile 与旧单 profile 均可读；默认 max_visits=800。沿用 atomic replace，不以新结构直接抛弃旧文件 |
| 启动命令 | `crates/engine-manager/src/lib.rs:363-420` | KataGoAnalysis 合成 analysis/config/model；GenericGtp args 为空。需让 adapter 生成正确 argv，而非 shell 字符串 |
| 生命周期 | `crates/engine-manager/src/lifecycle.rs` | readiness 仍发送 KataGo JSONL；默认 readiness_timeout=30s、stop_drain_timeout=2s。抽出协议差异，不重写 switch/job owner |
| 设置 UI | `apps/desktop/src/components/EngineSetupPanel.tsx:61-87` | buildProfile 固定 kata_go_analysis；新增 adapter 选择、argv 和 adapter-owned 表单，保存后才更新已持久化值 |
| 前端 admission | `apps/desktop/src/domain/foregroundEngine.ts:46-77` | ready/switching 判断不是能力判断；pending 比较漏 adapter/args。按 active run snapshot 判定，切换中仍看 A |
| gateway / wrappers | `apps/desktop/src-tauri`、`apps/desktop/src/api/backend.ts` | 迁移现有命令/DTO消费者；协议、解析、进程、存储逻辑留在 crates |
| SGF / Go 语义 | `crates/sgf`、`crates/go-core` | 从准确 NodePath 提取语义与完整路径，不复用压平后的分析初始棋子表示假装保留历史 |
| 既有可复用证据 | `crates/engine-manager/tests/foreground_engine_run.rs`、`apps/desktop/src/EngineLifecycle.test.tsx`、`apps/desktop/src/TruthfulAnalysisActions.test.tsx` | 覆盖生命周期、switch、stale 和 UI 真正可执行入口；新增断言围绕第二协议引入的差异 |

本树已初始化 CodeGraph 与 code-review-graph。索引只辅助定位；源码为准。规划不改生产源码，实施后索引由主执行者更新。

## 4. 技术方案

### 4.1 单目录与干净 DTO cutover

- 共享字段：稳定 ID、名称、program、argument vector、working directory、adapter kind。adapter settings 使用可区分的结构；KataGo 的 model/config/finite visits 留在其设置内，不强迫 GenericGtp 填无用模型路径。
- KataGoAnalysis 保留既有生成的 `analysis -config ... -model ...` 语义；共享 argv 作为额外参数，adapter 占有协议/模型/配置参数，冲突明确拒绝。GenericGtp 的 argv 是完整启动参数。两者均直接 spawn，不拼 shell、不自行按空格拆分用户路径。
- UI 用参数列表逐项编辑，保留有空格/中文/空参数的准确含义。切换 adapter 的草稿与已保存值分开；不携带失效的隐藏设置形成混合结构。
- 给持久化 envelope 明确格式版本。读取现有多 profile 和 legacy 单 profile，保留 ID、顺序、选中项、Autoload Default、路径和 visits；新格式只写一次标准形态。保留旧文件读取迁移，不保留两套运行时 DTO/别名。
- 读取迁移不自行启动进程或覆盖原文件；下一次成功显式保存原子写新格式。解析/版本/adapter 不支持时可诊断，原文件保留，不静默重置目录。
- Save 做结构、adapter、参数与数值校验；Start 重新做实际文件/资源校验。保留初次使用的未配置默认记录，不能把保存设置当作“已就绪”。保存失败不改变 durable catalog、Autoload、live snapshot 或已生效 UI 值。
- 仅暴露实际实现的 KataGoAnalysis、GenericGtp。已有未落地 enum/分支逐个迁移消费者；Readboard 不混入本地引擎目录，也不凭 enum 恢复一个独立 KataGoGtp 产品。

### 4.2 能力声明、验证与 UI

分清三层：adapter 的静态协议上限、保存的启动配置、当前 run 握手后验证的能力。未启动 profile 可显示“待验证”，不能伪造 live snapshot。

能力至少独立表达：game move、selected-node finite/continuous、whole-game/task、候选/PV、胜率、分数、ownership/policy、visits limit、GTP time mapping、取消方式和 exact-position 限制。只有真实消费者需要的能力才进入公共 DTO；内部握手细节不全部泄漏到 React。

- GenericGtp 不宣称任何 rich-analysis；`genmove`、`final_score` 或名称含 KataGo 都不能推导这些能力。
- KataGo 已接受的有限/连续/整谱/task 能力保持；已有 SGF 中的历史分析仍可查看，不因当前 GTP 无分析能力而删除。
- 所有入口覆盖：菜单、快捷键/Space、工具按钮、task preview/start/continue、continuous reconciler、Rust 公共入口。前端禁用与后端拒绝同时存在，不能只把按钮置灰。
- continuous intent 的持久值不因切到 GTP 被改写；在 GTP 上显示不可用且不提交查询。切回 KataGo 按既有独立 Ready、budget 和强 hold 契约重新判断，不越过 departure/error hold。
- Settings 展示 profile 草稿；主工作区展示当前 run 的实际能力。A→B 中用 A 的能力，B 成功 promotion 后原子切换。

### 4.3 进程与 GTP 协议

在 engine-manager 内形成两个真实 adapter 的内部 seam：启动准备、readiness、命令/响应编解码、协议取消/终止。manager 仍拥有进程、run/job identity、switch、deadlines、publication fence。优先现有 crate 内聚模块；不建插件框架、通用 RPC 总线或第二个进程管理器。

GenericGtp 使用 GTP v2：

1. 一个总 readiness deadline 内依次完成 `protocol_version`、`name`、`version`、`list_commands`，不能每收到一行就续期。
2. 必需标准动作包含 `boardsize`、`clear_board`、`komi`、`play`、`genmove`、`quit`；缺失则启动失败，不能 Ready。clock/setup 命令独立记录。
3. command ID 单调分配并校验响应 ID。协议按空行结束一个响应，不把单行读完当完整响应；处理分片、多行、CRLF、空成功体与 `?` 失败。
4. 单 run 串行执行改变 GTP 棋盘状态的命令。stdout 协议与 stderr 诊断分离；无界输出、未结束响应与退出均受有界资源/时间约束。
5. command rejection、framing/parse error、timeout、unexpected exit、unsupported capability 各有稳定类型及 run/profile/job 上下文，不让调用方解析英文错误文本。
6. 协议异常导致会话状态不可确定时，封闭发布并清理该 run；不继续发送命令猜测恢复，不自动重启。

### 4.4 Exact Position Admission

先在纯计算步骤产生完整 synchronization plan，再执行进程 I/O。计划包含尺寸、规则语义、komi、轮到谁、根 setup、按 NodePath 的历史 move/pass，以及实际所需命令。棋盘外观相同不代表劫争/重复历史相同。

投影必须显式处理每个回放错误，并按本次支持的完整规则/历史验证；不能把只保留 simple-ko 的裸 Board、最终 snapshot 或忽略回放错误的旧 helper 当成任意规则的合法性证明。新取步路径使用该投影；现有 selected/continuous 分析查询的改造不借此扩域。

| 输入类别 | GenericGtp 首轮策略 |
| --- | --- |
| 空根 + 完整合法普通 move/pass 路径 | 支持；从 clear board 顺序回放，明确颜色，不遗漏 pass |
| 正方形尺寸 | 仅已验证引擎支持的尺寸；不因 boardsize 存在就宣称所有尺寸 |
| 矩形棋盘 | 标准 GTP 路径拒绝；KataGo 现有可用行为不受影响 |
| 根黑棋让子 | 有已验证 `set_free_handicap` 且结果与请求完全一致时支持；否则提前拒绝 |
| 根白棋/混色任意摆子、路径中 AB/AW/AE | 没有经验证的精确命令与历史语义则拒绝；不能把 setup 改写为普通落子 |
| PL / side to move | 能用明确颜色与原历史精确表达的情形支持；无法保持局面/历史语义的路径中修改拒绝；不能插入假 pass |
| 未知/不匹配规则、不能精确表达的 komi | 结构化拒绝，不静默改成默认 Chinese 或 Japanese |
| 注释/标记节点 | 不创造落子；NodePath 仍精确绑定原节点 |

GTP v2 没有通用规则协商。支持规则必须来自 adapter 拥有且可验证的兼容性记录（引擎身份/版本、实际启动语义、命令与受支持规则），不能来自“用户勾选我支持”或单看 `list_commands`。未知引擎可以完成协议握手，但在规则无法证明时精确局面 admission 失败；界面区分协议就绪与具体局面可用。

实现同时记录两种失败边界：

- **admission 拒绝**：不得写协议、取消 A 的工作或改变 current game/durable defaults。
- **执行中引擎拒绝/退出**：可能已改变目标进程内部棋盘，必须使其同步状态失效，必要时终止；仍不得修改 current game。未来 R9 的全局预约/回滚由 R9 承担，R8 不伪造该证明。

### 4.5 取步、预算与取消

R8 的真实取步路径是 manager-owned operation，输入含 run/job/document generation/NodePath、准确局面和 Compute Budget；输出 typed move/pass/resign 或 typed failure。它不持有 current-game 写权限。R9 才把合法结果提交到 SGF。

同一 seam 交付两种映射：

- **KataGoAnalysis**：用 exact projection 构造完整局面 JSONL（真实根 setup、完整 move/pass 历史、规则/komi/尺寸和可精确表达的 initialPlayer），只分析请求目标。仅接受当前 query/turn 的完成响应，选择唯一 `order = 0` 候选返回 typed move/pass；不按数组首项、visits 或胜率猜测，不生成自动 resign。缺失/重复 order=0、非法结果或规则降级 warning 都是 typed failure，不使用旧结果兜底。
- **GenericGtp**：执行经 admission 的准确同步及预算映射后调用 `genmove`，返回 typed move/pass/resign。协议支持与具体局面可用分开。

两者共用身份 fence 与结果接口，但不混用协议或取消策略。取步期间只占有一个 run 的单次操作槽；已有 selected-node/whole-game/task 工作或另一个取步正在执行/清理时，先以 Busy 拒绝，不隐式取消它们。取步准入后该 run 的新分析与取步亦被拒绝，直到完成或 cleanup 确认；continuous reconciler 尊重这一暂时占用，不修改持久 intent。该槽不是 R9 的跨 run、跨回合 Match Reservation。

KataGo 取步要求显式正 `max_visits` 与 hard deadline；调用方可从 profile 的有限预算默认值预填。visits 映射 JSONL `maxVisits`，deadline 仍由 manager 执行；截止前收到有效完成响应才可成功。取消/超时先 seal，再 targeted terminate 并等待目标 final；清理成功保持 Ready，清理失败走既有 Error/强制回收。不能把 GTP 的正常取消即杀 run 策略套给 KataGo。取步能力只有在对应实现可用且请求通过参数/局面 admission 时才授予。

- 预算含正数 hard wall deadline；visits 仅 adapter 支持时接受。GenericGtp 不把 seconds 转成 visits，也不忽略请求中的不支持字段。
- 同时支持 `time_settings` 和 `time_left` 时，按冻结契约发送 `time_settings 0 <seconds> 1` 及当前颜色对应的 `time_left <color> <seconds> 1`，成功后才发 `genmove`。两命令不全时能力为“不支持引擎时间映射”，manager deadline 仍执行。
- hard deadline 从取步操作被 manager 接受开始计时，涵盖必要同步/时间命令/等待/生成；不是每条命令一个可无限延长的期限。seconds 使用正整数；time_left 使用剩余期限的保守可表达值，剩余不足一秒则不再发 genmove，直接 deadline failure。
- `genmove` 已在引擎内部落子，后续不可再向同一引擎重复 `play` 这一手。被上层拒绝/取消的结果使同步基点失效；重新使用前必须完整同步。
- 不将返回顺序、最后一行或最近选择的 profile 当身份；合法坐标转换、I 列跳过、pass/resign 与越界/非法走子均经类型化处理。
- 标准 GTP 没有通用 interrupt。首轮取消策略保守：立即 seal job，终止并确认回收对应 run，拒绝迟到输出；用户取消回到 No-engine，超时/协议失败进入现有 Error/manual recovery。不得自动重启/续算，也不得复用 KataGo 的 `terminateId`。
- cleanup 未确认不能假装资源已释放或接纳下一项。显式 Stop/Restart/switch/exit 仍走现有统一 teardown。

## 5. 真实非 KataGo 引擎准入

规划选择的验收候选为 **GNU Go 3.8（GTP v2）**：不需要神经网络模型。03 已完成 Windows 二进制资格探测，04 已通过生产 manager 真实取步，05 已通过最终原生集成；正式支持限制见 [DEVELOPMENT §2.3](DEVELOPMENT.md#23-exact-position-budgeted-moves-r8-ticket-04)。

正式实现/验收前，执行者完成一次最小资格探测并固定证据：二进制来源与版本、平台、argv、name/version/list_commands、规则启动语义、一个受支持尺寸、komi、move/pass 回放、time_settings/time_left、genmove、quit/停止。规则的计分/ko/suicide 差异必须与 Next 语义核对；单纯接受命令不是语义证明。

若 GNU Go 在目标环境不能提供所需时间映射或精确规则证明，不能删减 R8 出口；记录阻塞原因，选择另一款真实非 KataGo GTP 引擎，并用相同资格表重新固定支持范围。若更换导致产品范围/行为改变，再请求用户决定。控制进程 fixture 或 `katago gtp` 都不能替代非 KataGo 证据。

规划时未找到 WSL 的 `gnugo` 命令或标准安装目录中的可执行文件；这项历史环境观察已由后续 Windows GNU Go3.8 资格探测与实际验收补齐，不再是阻塞。上游构建/发布工程不属于本票交付。

协议依据：[GTP v2 draft 2](https://www.lysator.liu.se/~gunnar/gtp/gtp2-spec-draft2/gtp2-spec.html)、[GNU Go GTP 文档](https://www.gnu.org/software/gnugo/gnugo_19.html)、[GNU Go 启动文档](https://www.gnu.org/software/gnugo/gnugo_3.html)。实际二进制 transcript 优先于仅根据文档推断的支持。

KataGo 新取步映射依据：[Analysis Engine protocol](https://github.com/lightvector/KataGo/blob/master/docs/Analysis_Engine.md) 的 `moves`、`initialStones`、`initialPlayer`、`maxVisits`、完成响应与 `order`。上游提示旧模型可能自动改写不支持的 rules；新取步路径必须把此类 warning 视为语义不一致并失败，不能宣称 exact。

## 6. 已发布执行票与依赖

用户于 2026-10-01 确认 5 个实施工作包与 1 个只读 Closeout；以下正式票按本地 tracker 约定一票一文件发布。01–04 的最终成果已线性进入 05 起点，05 的实际验收、最终审查和 commit 以其 Completion record 为准；06 仅消费 01–05 的最终记录，不追加产品验收。

| 工作包 | 完整交付路径 | Blocked by | 完成证据 |
| --- | --- | --- | --- |
| [01 多后端目录与 KataGo clean cutover](../.scratch/engine-adapters-r8/issues/01-profile-catalog-cutover.md) | 旧文件加载 → adapter 表单 → argv/原子保存 → KataGo Start/Restart；能力 DTO 与所有现有消费者同步迁移 | 无；从固定 R7 SHA 开始 | 迁移/保存失败/argv 实际 spawn、不可变快照、现有 KataGo 原生焦点 smoke；ENG-09 的配置部分 |
| [02 能力驱动的操作与 ENG-09 接受](../.scratch/engine-adapters-r8/issues/02-capability-admission-eng09.md) | run/profile 能力显示 → 全入口 admission → continuous/task 边界 → §6 的 M01–M03 native 机制验收 | 01 | 汇总 01/02 的 ENG-09 repository/native 证据，由本票验收 owner 更新 ENG-09 为 Accepted 并交接候选；03 消费该完成记录后开始 |
| [03 GenericGtp 生命周期与就绪](../.scratch/engine-adapters-r8/issues/03-generic-gtp-lifecycle.md) | Generic profile Start → 有界握手 → 标准能力 snapshot → 跨协议 switch/rollback → Stop/Restart；真实非 KataGo 资格探测 | 02 | 确定性 GTP 进程及真实引擎启动/退出；缺命令、坏帧、timeout 和失败 switch 保留 A |
| [04 精确局面与双 adapter 受控取步](../.scratch/engine-adapters-r8/issues/04-exact-position-budgeted-move.md) | SGF NodePath → exact admission → 同一取步接口；KataGo 完整 JSONL/order=0 与 GTP sync/genmove；budget、单次占用、取消/身份/非法返回围栏 | 03 | 两条实际进程 harness 返回合法结果或 typed failure；current game 不变；真实 KataGo 新取步与非 KataGo sync/budget/genmove/Stop |
| [05 集成验收与 R8 封口](../.scratch/engine-adapters-r8/issues/05-native-integration-r8-acceptance.md) | 合并所有最终成果 → 固定候选 → Windows app-runtime、真实引擎、焦点回归 → 文档/Matrix 更新 | 04（因此包含 01–03） | 继承并复核 02 的 ENG-09 接受记录，补最终组合证据；§7 全部满足、Standards + Spec CLEAN 后接受 ENG-10 并交接 R9 |

另设 [**06 只读 Closeout**](../.scratch/engine-adapters-r8/issues/06-review-followup-closeout.md)，阻塞于 05，输入明确包含 01–05 每个最终 review completion record。只归并已有 follow-up 或明确 `No follow-up candidates`；不代替 05 的验收，不运行第二轮无边界审查，也不自动开范围外修复票。

### 关于 02 的 ENG-09 门禁

ENG-09 接受目录、快照和能力 admission 机制；ENG-10 接受具体 GTP 协议。02 在真实 Windows app-runtime 上执行下表，无须添加可被用户选择的验收 adapter，也不把 GenericGtp 草稿标成 Ready。

| 02 的 native 机制场景 | 实际观察 | 证据归属 |
| --- | --- | --- |
| M01 多 adapter 目录 | 同一设置页创建/保存/重载 KataGoAnalysis 与 GenericGtp；显示各自字段、合法 argv 和 GenericGtp 待验证状态；错误配置/保存失败不改变 durable 数据 | ENG-09 native multi-adapter catalog；复用 N01 |
| M02 已验证 run 与 pending | 真实 KataGo 启动后经 manager → IPC → UI 展示已验证能力；编辑 adapter/argv 后当前 run 仍为旧 snapshot，pending 可见；不因目录选择变化隐式启动 GTP | ENG-09 native snapshot；复用 N04 的本阶段部分 |
| M03 调用前解释及无副作用 | 此阶段 GenericGtp runtime 未交付，显式 Start/切换先解释“适配器运行支持尚未提供”并拒绝；原 KataGo/No-engine、棋谱、continuous intent 和目录均不变。真实 KataGo 的动作按自身能力正常执行 | ENG-09 native admission；不宣称 GTP readiness 或游戏能力 |

非分析 capability snapshot 的菜单/Space/task/continuous 全入口差异，由 repository 控制进程/状态证据覆盖；不能仅伪造 UI 状态作为 native 证明。02 更新 ENG-09 的候选与证据；03 消费这一接受记录。真实 GTP Ready 后的能力差异与跨协议 native 场景归 05 的 N02/N03/N05/N06，不能用 M01–M03 替代，也不能把此前未运行部分追记为通过。

### 实现基线与并行

- 01 的起点是 `a0c8ed370f620341c863a3bb8fd4936710d82e72`。
- 其余工作包使用前置最终完成记录中的 commit，或已验证包含其成果的 commit；不预填未来 SHA，不因新开工作树改用当日 main。
- 多个尚未合并的前置由该后继票主执行者先在隔离 integration tree 合并，记录 integration SHA 后再冻结实现起点。05 主执行者拥有最终集成与验收。
- app-model/镜像 TS DTO、目录格式、manager lifecycle 是共享边界，不能由多个 writing agents 同时改。可独立并行的是协议资料/真实引擎准备、纯 admission 设计与 fixture 数据、已冻结接口下的 UI 接入；共同文件由一名 integration owner 落地。
- 每票新会话，先读 spec 与前置完成记录；子代理跳过 build/lint/test，主执行者统一做必要验证。每票完成要求 Spec + Standards 审查以及该票指定的实际验收，不以写完代码封票。

## 7. 验收计划

### Repository 与真实进程

| 编号 | 受影响行为 → 必须观察到的结果 | 最小验证面 |
| --- | --- | --- |
| P01 | 两种旧文件读取后 ID/selection/autoload/paths/visits 不变；坏新版本/保存失败不覆盖原文件 | catalog 公共读写接口，隔离文件 |
| P02 | 带空格/中文/空值 argv 逐项到达子进程；无 shell 展开；adapter 所需参数准确 | 实际可控子进程回显收到的 argv，验证行为而非字符串构建实现 |
| P03 | profile 修改、Autoload 修改或设置页选中变化不改变 live snapshot；Restart 才用新配置 | manager + catalog/UI 行为 |
| P04 | 握手分片/多行/CRLF/错误 ID/缺命令/`?`/退出/永不结束均在规定边界 Ready 或 typed failure | 控制 GTP 子进程，实际 pipes |
| P05 | A→B失败保留 A；B过期不能 promotion；Stop 后迟到完成不能复活 run | manager 跨协议状态轨迹 |
| P06 | 普通路径、pass、合法让子及可表达 PL 准确同步；矩形/未知规则/不支持 setup 在 I/O 前拒绝 | SGF fixtures → admission → command transcript，确认 current game 不变 |
| P07 | time mapping 仅在双命令支持时执行；不支持 visits 提前拒绝；deadline 不被响应/排队重置 | 受控 monotonic clock + 真实子进程边界 |
| P08 | 同一接口区分 move/pass/resign；GTP不重复play；KataGo只取完成响应唯一order=0（刻意打乱数组顺序）；缺失/重复候选、非法坐标/走子、规则降级拒绝 | 取步公共接口 + 同一规则/完整历史的合法性验证；含同棋子图不同ko/history |
| P09 | busy 准入不取消旧lane；取步封闭新lane；cancel/switch/restart/exit/generation/NodePath变化使旧结果无效；GTP回收，KataGo targeted cancel/target-final后保持Ready | 两种adapter的result sink、资源占用与进程状态观察，不写SGF、不自动重启 |
| P10 | 菜单/快捷键/task/continuous 在 GTP 上不提交分析；保存的 continuous intent 与已有 SGF 分析不丢 | 渲染 UI + Rust admission；状态变化而非 mock forwarding 测试 |
| P11 | KataGo 的有限/连续/task 取消与 target-final、双 lane、departure hold 仍正确 | 只选 DTO/adapter seam 改动触及的既有测试与真实 KataGo smoke |

计时测试以可控时钟/受控响应为主，不靠长 sleep 猜测 race。测试过滤器执行零项不算通过。共享 DTO cutover 确实跨 crate，可在最终门禁运行覆盖这些消费者的完整 Rust/TS 检查；不每票机械重跑全仓或 release validators。

### Windows 原生 app-runtime

继承项目 Windows 候选流程：精确 committed SHA、独立目的目录、独立 app-data、private desktop。不得拿 `npm run dev` 代替 native，不接管用户日常桌面。此处验收的是本轮修改的原生应用行为，不是 NSIS/portable Installed Live Evidence。

| 编号 | 场景与可观察结果 | 证明边界 |
| --- | --- | --- |
| N01 | 已有 KataGo 目录升级、编辑 adapter/argv、保存失败保留已生效值、重启读取、Autoload 单选 | profile/UI/native persistence；用隔离数据 |
| N02 | 真实 KataGo Ready → 真实非 KataGo GTP Ready → 切回；显示当前 run 的能力；坏 GTP candidate 不取代 KataGo | app 内实际 child identity 与 UI，而非只跑协议脚本 |
| N03 | GTP 下分析菜单、Space、task/continuous 说明一致；不启动隐藏 KataGo；旧 SGF 分析可读；切回不破坏原有 hold | pre-invocation explanation 与无副作用 |
| N04 | 修改活跃 profile 显示 pending，Restart 新 run 才采用；Stop/窗口退出后只回收自己的 child；重启不恢复 live run，Autoload 按目录决定新建 | lifecycle、snapshot、teardown、persist/runtime 区分 |
| N05 | 真实非 KataGo 二进制经同一 manager harness：准确局面 → time mapping → legal genmove → Stop | 固定引擎/version/argv、完整 transcript、结果/退出。R8 不增加游戏按钮来做这项验收 |
| N06 | 最终候选上的真实 KataGo：准确局面 JSONL → order=0 typed move/pass，budget/cancel后current game不变；跨协议切换后再分析及受影响safe departure | 新取步与既有JSONL分别证明；04的同候选等价证据可继承，不重跑R6/R7无关全矩阵 |

Linux 控制进程证明跨平台纯逻辑；Windows 原生证明本轮原生表面。macOS/Linux GUI 未运行的部分明确 NOT RUN，不据此扩张平台支持，也不把 R7 的平台豁免复制过来。实际受改动影响的跨平台编译失败须修复。

### 证据继承与归属

- ENG-02–07、ANA 与 R6/R7 历史接受记录保留原候选/条件，不能改写为本次通过。
- 共享 profile、capability 或 dispatch 改动会使“这些 seam 未改”的继承条件失效，需补上述焦点证据；与 seam 无关的布局/DPI/标记/提供商不用重验。
- 01–04 各自交付 transcript、命令、实际计数、候选 SHA/版本、未覆盖边界和最终 review completion；05 只补最终组合及剩余 native gap，不重复所有前置证明。
- 截图只证明 UI。genmove 合法性、失败保存不变、没有孤儿进程需要对应机器证据。Repository、真实引擎、native UI 三栏分别记录 PASS/FAIL/BLOCKED/NOT RUN。

## 8. 风险与处理

| 风险 | 处理 / 停止条件 |
| --- | --- |
| 新 DTO 让旧目录不可读或丢 Autoload | 旧格式迁移先行，读取不重写，持久化失败全体回滚；迁移不通过不进入 runtime cutover |
| Ready 被误当 full-analysis | capability 与生命周期分离；覆盖自动 continuous 和快捷键，后端仍拒绝 |
| GTP rules/setup 不能精确表示 | 资格表与纯 admission；未知拒绝，不能用最终棋子图“近似等价” |
| 无协议取消却继续复用 run | seal → kill/reap；No-engine/Error；显式 Restart，迟到响应无权发布 |
| genmove 已改变引擎状态但外部未提交 | 同步基点失效；R8 返回结果不落子，R9 拥有 commit/重新同步 |
| 为 native gate 顺手造 R9 对局界面 | 使用同一 manager 的薄验收 harness，UI只测设置/能力/lifecycle |
| 用 fixture 或 KataGo GTP冒充非 KataGo | 真实外部引擎必需；没有环境只记 BLOCKED，ENG-10 不接受 |
| GTP抽象破坏 KataGo target-final/continuous | 保留 JSONL 实现与 owner，回归触及的 lane/departure/race，不统一成最弱协议 |
| 多前置未合并、HEAD 漂移 | 后继主执行者做隔离集成并冻结 SHA；不直接从最新 main重启 |

## 9. R8 出口与 R9 交接

必须全部成立：

1. ENG-09/ENG-10 满足 Matrix 验收并更新真实证据；ENG-01 历史不被扩大或覆盖。
2. 旧数据迁移、adapter catalog、verified snapshots、unsupported-before-mutation 完整落地；全部 Rust/TS/IPC 消费者完成 clean cutover。
3. GTP handshake、exact-position admit/reject、Compute Budget、typed outcomes、取消及 stale fencing 有确定性真实进程证据。
4. 至少一款固定版本真实非 KataGo GTP 完成 handshake、exact sync、time mapping、genmove、Stop；native capability 与 unsupported explanation 通过。
5. 本轮修改影响的 KataGo 生命周期/分析/departure 无回归；无残留临时验收脚手架或孤儿进程。
6. Architecture、Development、Parity Matrix、Migration Plan 同步；记最终 committed candidate 与 review CLEAN。没有证据的行明确未运行。
7. 给 R9 交接：profile schema/迁移、能力含义、exact admission 限制、取步输入输出、取消终止 run 的后果、genmove 内部已落子的语义、失败分类与原生证据位置。

R9 再实现全局 Match Session/Reservation、当前游戏 commit、人机/PK、持久开始默认值和 SGF终局归属。R8 不创建会自行推进棋谱的隐藏 match owner。

## 10. 当前交付状态

**产品验收：PASS。** ENG-09 保留 02 的 `b23f3eb16c16ad63bf4570d7ecc9bf22c274ea05` 接受记录；ENG-10 在 05 的最终组合矩阵中 Accepted。调查基线仍为 `a0c8ed370f620341c863a3bb8fd4936710d82e72`。01–04 的最终 SHA 依次为 `3c67c0f5d9bd227d08e0d758291b9541a4697cc4`、`eb34c204c8327639c0ca3038323ae87c0d9ebc9b`、`83b50f3046f4b9b6241eb7a3b3cac5b8fe16c240`、`42cd91a657d2710b826cc1b94f21f168de64a0fa`，全部在当前分支祖先链内。

05 实现起点与最终 app-runtime 候选为 `42cd91a657d2710b826cc1b94f21f168de64a0fa`；工作树 `/home/dev/dev/weiqi/worktrees/lizzieyzy-next-tauri/r8-profile-01-20261001`，分支 `feat/r8-profile-catalog-01`。本票只变更文档；代码、依赖与资源不变。04 的 `62841b149361603a5701170fd84179a25f306d73` 至此仅六行验收文档差异，真实双引擎取步保留原候选归属。最终 Windows 三次私有桌面会话补跨 adapter 切换/坏 B 回滚、GTP 调用前拒绝、历史分析与 safety hold、pending/Restart、Autoload/无 live recovery 和正常退出；全部 exit0、最终 owned process 为空。

P01–P11：PASS；M01–M03 保留 02 的原始机制范围；N01–N06：PASS（逐项继承条件、命令、计数、目录见 [DEVELOPMENT §2.4](DEVELOPMENT.md#24-r8-integrated-acceptance-and-r9-handoff-ticket-05)）。最终 Rust457 passed/3 opt-in ignored，前端135 passed。macOS/Linux GUI **NOT RUN**，不复制 R7 的平台特例；不宣称 installer/provider/readboard 或物理显示验收。Standards/Spec 的独立最终 verdict、审查目标与提交后 HEAD 由 [05 Completion record](../.scratch/engine-adapters-r8/issues/05-native-integration-r8-acceptance.md#completion-record) 保留；06 的只读归并仍是独立工作。

R9 交接采用 Architecture 的 read-only move API、version-1 catalog、verified primary-run capability、exact admission 限制、预算/typed failure/取消/重新同步契约。GTP 取消终止 run，genmove 已在引擎内落子且每次请求重新同步；运行态不恢复。R9 才拥有 Match Session/Reservation、当前棋谱 commit 与对局产品，不因 R8 接口已存在而省略 R9 的条目门禁。
