# R12 — 引擎资源与运行管理

Status: 已完成批准的 R12 spec 范围及 R12/R13 同实例联合验收；17 张票、85 项验收在最终集成 `ca1b6f33ec822931fb51501c3e24c6e1924ee62e` 完成共享 Spec/Standards 审查和只读 Closeout，后续修复随 PR #21 合并于 `7731e72f8918956d98e76abe6456552357cb6ef1`。完整条目状态仍按 Matrix 的跨阶段剩余义务判断；候选、平台及硬件限制见下文。

<a id="r12-completion"></a>
## 完成记录与保留边界

- 调查与实施起点为 `9d2ccf3ba6881c755013b82948e5da1a01f7fcd4`。唯一原始执行记录位于 `/home/dev/dev/weiqi/worktrees/lizzieyzy-next-tauri/r12-planning-20261008/.scratch/engine-runtime-r12/` 的 `tracker.md`、`issues/01–17` 和共享 `review.md`；以下批准合同及历史调查保持原有归属。
- 具名 gate/决定记录中的“实施/验收未完成”“仍未验收”“尚未获证”等状态描述保留 2026-10-08 批准时点的历史上下文；合同与资格要求继续有效，当前完成结果以本节和 Matrix 为准。
- [公开完成证据](DEVELOPMENT.md#r12-completion-evidence) 分列局部实现、真实引擎、Windows 原生与集成验收；[后续修复和合并](DEVELOPMENT.md#r12-combined-candidate-evidence) 保留精确候选与实际 CI。原始原生证据不重标为合并提交的新运行。
- R12-16 已在 `d2462d3d60d5b9c293d55e106dd77cf3323fcf5b` 完成同一 GTP Run 的主分析、线程 Apply/确认、PDA/WRN pair，以及 JSONL 切换、失败保留和导入后显式恢复。预加载修复在 `ab9f8184daddc00a8a214336c1d7eb85210f8cb8` 补充真实 TRT 长启动与任务 Continue 让路；诊断收敛在 `ca1b6f33ec822931fb51501c3e24c6e1924ee62e` 补充受影响原生证据。R12-17 仅核账，不承担这些运行验收。
- R13 提供的 `katago-gtp-main-analysis` 和 `isolated-benchmark-result` 已按批准范围交付。保存式 CFG/BENCHMARK 策略、双性能指标与无障碍反馈、实测报告 Import/Review/Apply/Restore 仍归 [R13](R13_PLAN.md#performance)，不因 runner 或手动线程完成而视为完成。
- 真实 GPU 证据仅覆盖记录的 RTX 5070 Ti Laptop／驱动 591.66／TRT 10.9.0.34、CUDA 12.8、cuDNN 9.8.0.87／KataGo 1.18.2／B11 资源元组。15 个目录目标不是全硬件准入；B10 保留的受控证据不等于真实 B10 原生资格。OpenFolder 实际结果是 TimedOut，不是可见 Explorer 成功。
- R17 设置迁移/完整语言、R18 签名/安装/更新/安装态支持路径、其他平台及后续消费者的实际资格仍保留各自门禁。后续功能只消费已验证 scope，变更行为、资源或平台时由消费者重验受影响边界。

## 目标、完整范围与非目标

能以合格资源可靠启动、维护、诊断、显式管理线程及恢复本地运行，保持单引擎权威与可见失败。

非目标：不实现R13任务/benchmark展示、R14远程feature或R18组件分发；不把15来源目标当全GPU准入。

本阶段包括以下具名能力章节及其引用的所有 source/Delta/用户断言、默认与保存、错误与取消、原生或实际服务义务。详细契约通过权威历史全文稳定引用保留，不以摘要替换契约；本文件与公开路由的 scoped-result 规则优先于历史来源排期。

<a id="resources"></a>
## 合格资源、受管维护与显式修复

Start/Switch 本地合格资源必须有 origin/version/path/digest/协议能力与可见拒绝，失败保留健康 Ready A/棋谱。managed catalog、15个来源目标、B11默认型号采纳、后台维护、source identity与GPU说明不冒称所有硬件已认证；实际目标TensorRT repair为显式动作且重新确认原目标，不隐式换后端。下载消费已通过统一网络对应transport；本地资格无需managed/生产Release/固定开发机路径。维护进度/取消/失败保留last-good；schema2 static-zlib例外只适用精确catalog/manifest未修改资源。

唯一能力责任：`T01-RESOURCE`, `T03-RESOURCE-ACCEL-LAYOUT`, `T03-RESOURCE-NVIDIA-HARDWARE-GATE`, `T03-RESOURCE-TRT-REPAIR-INVESTIGATION`。本节设置/技术调查与后续实现是同一完整能力；跨节消费不改变来源owner。

详细来源全文（归档合同，不是活跃任务）：

- [B01 — Qualified local runtime resources at Start and Switch](MIGRATION_CONTRACTS.md#b01)：Establish the first-consumer resource qualification result for actual engine Start/Switch without a managed catalog or release prerequisite.
- [B02 — Freeze managed catalog and explicit resource-maintenance admission](MIGRATION_CONTRACTS.md#b02)：Produce the missing managed-resource product admission contract, without changing the frozen qualified-local-resource boundary.
- [B03 — Managed resource maintenance and version-qualified GPU guidance](MIGRATION_CONTRACTS.md#b03)：Implement the admitted managed resource acquisition/maintenance surface and version-aware guidance.
- [B04 — Determine ownership of target-directed TensorRT repair](MIGRATION_CONTRACTS.md#b04)：Decide whether admitted resources own TRT-directed repair, external responsibility, or a specific product decision.
- [B05 — Bound NVIDIA hardware qualification for admitted repair targets](MIGRATION_CONTRACTS.md#b05)：Produce supported/unsupported/unknown hardware qualification and evidence responsibility for each admitted target.
- [B06 — Reachable acceleration setup actions and status layout](MIGRATION_CONTRACTS.md#b06)：Present only admitted acceleration setup actions with grouped, readable, keyboard-reachable states.
- [B73 — Approved explicit target-directed TensorRT resource repair](MIGRATION_CONTRACTS.md#b73)：Deliver admitted TRT-directed repair or explicitly approved external-owner outcome while preserving original resource/user choice.

最小验收面与用户断言：上述全文的 Acceptance/Concrete acceptance 和 Frozen source/Canonical Delta 条款全部保留；本节 owner 用实际 changed domain/owner/protocol/persistence/UI 面证明成功、失败/Cancel、身份失效、适用 defaults/save/reopen；只读调查以有来源的可判定结论结束，不能宣称功能已完成。每个受影响设置 map 归该消费者。

<a id="models"></a>
## 模型身份与已安装候选保留

模型头/content/catalog/选择与已安装保留身份完整呈现；异步refresh与busy/旧snapshot不能误enable或覆盖新身份。来源样本与技术映射是功能内步骤，已解析的模型表示不是产品审批。

唯一能力责任：`T04-MODEL-IDENTITY`。本节设置/技术调查与后续实现是同一完整能力；跨节消费不改变来源owner。

详细来源全文（归档合同，不是活跃任务）：

- [B07 — Map model headers, ownership and installed retention](MIGRATION_CONTRACTS.md#b07)：Deliver a bounded source/local-sample inventory and technical model-header/ownership/catalog-retention contract for B08.
- [B08 — Model identity and installed-candidate retention](MIGRATION_CONTRACTS.md#b08)：Expose real model identity and preserve installed choices independently of display-name and catalog refresh.

最小验收面与用户断言：上述全文的 Acceptance/Concrete acceptance 和 Frozen source/Canonical Delta 条款全部保留；本节 owner 用实际 changed domain/owner/protocol/persistence/UI 面证明成功、失败/Cancel、身份失效、适用 defaults/save/reopen；只读调查以有来源的可判定结论结束，不能宣称功能已完成。每个受影响设置 map 归该消费者。

<a id="startup"></a>
## 启动、默认引擎、预加载与评估让路

保留last-primary startup与额外引擎preload目标并逐项协调单引擎权威；未经批准不以现实现宣称等价或静默删去。startup performance入口/default/foreground yield属于本节，测量仅消费R13独立runner的已验证输入/结果，不等待saved-policy或全部性能工作。启动策略不隐式切换、重启或应用临时线程。

当前产品决定见 [B13](#r12-gate-b13)、[B15](#r12-gate-b15)、[B47](#r12-gate-b47)，批准来源为本轮用户对 Q1–Q3 的“都按推荐来”。三个开关只授权各自行为；本次批准不扩大任何 adapter 的实际协议能力，不等于已有实现或整个 R12 开工授权。

唯一能力责任：`T02-H17`, `T02-H18`, `T04-STARTUP-PERFORMANCE`。本节设置/技术调查与后续实现是同一完整能力；跨节消费不改变来源owner。

详细来源全文（归档合同，不是活跃任务）：

- [B13 — Decide last-primary-engine startup policy preservation](MIGRATION_CONTRACTS.md#b13)：Close the named historical non-equivalence with a retained executable behavior contract or item-specific approved disposition.
- [B14 — Retained last-primary-engine startup policy successor](MIGRATION_CONTRACTS.md#b14)：Deliver only the approved retained user behavior or evidence-backed no-change closure produced by the preservation decision.
- [B15 — Decide background extra-engine preload preservation](MIGRATION_CONTRACTS.md#b15)：Close the named historical non-equivalence with a retained executable behavior contract or item-specific approved disposition.
- [B16 — Retained background extra-engine preload successor](MIGRATION_CONTRACTS.md#b16)：Deliver only the approved retained user behavior or evidence-backed no-change closure produced by the preservation decision.
- [B47 — Decide startup performance evaluation entry and foreground priority](MIGRATION_CONTRACTS.md#b47)：Close missing item-specific startup evaluation/default/persistence/yield product policy.
- [B48 — Approved startup evaluation and non-disruptive yield](MIGRATION_CONTRACTS.md#b48)：Deliver only the approved startup performance behavior and foreground yield policy.

最小验收面与用户断言：上述全文的 Acceptance/Concrete acceptance 和 Frozen source/Canonical Delta 条款全部保留；本节 owner 用实际 changed domain/owner/protocol/persistence/UI 面证明成功、失败/Cancel、身份失效、适用 defaults/save/reopen；只读调查以有来源的可判定结论结束，不能宣称功能已完成。每个受影响设置 map 归该消费者。

<a id="diagnostics"></a>
## 首消费者诊断与响应式控制台

首实际Run/resource消费者承担visible error、startup stderr/probe隔离、runtime/thread/cache trace、脱敏、pinned bounded cancellable export与shutdown cleanup；full trace默认关，无自动上传。预算/原生采集表示为普通技术步骤。控制台限界/响应性与加载状态同属实际用户能力，C/D/E扩展各自秘密与失败字段；R18仅承接安装日志路径/支持包。

唯一能力责任：`T01-DIAGNOSTICS`, `T03-CONSOLE-OUTPUT-RESPONSIVENESS`。本节设置/技术调查与后续实现是同一完整能力；跨节消费不改变来源owner。

详细来源全文（归档合同，不是活跃任务）：

- [B09 — Define bounded first-consumer diagnostics and export budgets](MIGRATION_CONTRACTS.md#b09)：Make the frozen diagnostics behavior executable by naming actual roles, source limits and collection/export deadlines.
- [B10 — First-consumer bounded runtime diagnostics and pinned export](MIGRATION_CONTRACTS.md#b10)：Deliver required startup/resource/analysis diagnostics at their first actual Run consumer, not after release.
- [B61 — Responsive bounded console output and load status](MIGRATION_CONTRACTS.md#b61)：Keep actual console output and load status responsive, identity-correct and bounded.

最小验收面与用户断言：上述全文的 Acceptance/Concrete acceptance 和 Frozen source/Canonical Delta 条款全部保留；本节 owner 用实际 changed domain/owner/protocol/persistence/UI 面证明成功、失败/Cancel、身份失效、适用 defaults/save/reopen；只读调查以有来源的可判定结论结束，不能宣称功能已完成。每个受影响设置 map 归该消费者。

<a id="manual-threads"></a>
## 手动运行线程来源、临时覆盖与实际值

区分saved CFG/launch override/current actual/temp requested/pending与未提交编辑；manual Apply/readback绑定Run/profile revision/request。实际能力/version/合法域需证据，不抄Java1..1024。Restart/Switch/disconnect过期、reset-to-config、初始化不以legacy numSearchThreads覆写已解析启动值；unknown/failure保留last-valid且不覆盖未提交编辑。不依赖benchmark、建议保存或measured报告；不自动reapply/reconnect。

唯一能力责任：`T03-THREAD-ALIAS-APPLICABILITY`, `T04-THREAD-CONTROL`, `T05-RUNTIME-THREADS`。本节设置/技术调查与后续实现是同一完整能力；跨节消费不改变来源owner。

详细来源全文（归档合同，不是活跃任务）：

- [B42 — Freeze manual runtime thread source, effective and temporary scope](MIGRATION_CONTRACTS.md#b42)：Deliver source/effective/pending/temporary precedence and admitted dynamic protocol lifetime independent of benchmarking.
- [B43 — Manual dynamic thread Apply and confirmed readback](MIGRATION_CONTRACTS.md#b43)：Apply explicitly requested temporary thread changes on an admitted current Run and show actual confirmed value.
- [B44 — Determine managed config/include thread-alias applicability](MIGRATION_CONTRACTS.md#b44)：Decide whether actual admitted analysis launch uses conflicting thread aliases, without imposing Java launch machinery.

最小验收面与用户断言：上述全文的 Acceptance/Concrete acceptance 和 Frozen source/Canonical Delta 条款全部保留；本节 owner 用实际 changed domain/owner/protocol/persistence/UI 面证明成功、失败/Cancel、身份失效、适用 defaults/save/reopen；只读调查以有来源的可判定结论结束，不能宣称功能已完成。每个受影响设置 map 归该消费者。

<a id="runtime-readback"></a>
## PDA/WRN 成对只读回读

只读PDA/WRN pair必须同current reader/Run/round/numbered-response，两项均finite才成功；缺半/乱序/超时/旧Run不能合成当前值。默认unknown，失败显示last-valid+failed/unknown且保护未提交编辑。具名实际引擎支持两个readback命令及原生UI证明仍是准入门；不新增paired write/Apply/retry/reapply。

唯一能力责任：`T05-PARAMETER-READBACK`。本节设置/技术调查与后续实现是同一完整能力；跨节消费不改变来源owner。

详细来源全文（归档合同，不是活跃任务）：

- [B67 — Admit paired PDA/WRN current-Run readback capability](MIGRATION_CONTRACTS.md#b67)：Establish actual supported current-Run PDA/WRN readback commands/value domains and reader/round/numbered-response pairing evidence.
- [B68 — Paired PDA/WRN current-Run readback](MIGRATION_CONTRACTS.md#b68)：Deliver current-reader paired runtime readback with truthful failure/unknown states and protection of unsubmitted edits.

最小验收面与用户断言：上述全文的 Acceptance/Concrete acceptance 和 Frozen source/Canonical Delta 条款全部保留；本节 owner 用实际 changed domain/owner/protocol/persistence/UI 面证明成功、失败/Cancel、身份失效、适用 defaults/save/reopen；只读调查以有来源的可判定结论结束，不能宣称功能已完成。每个受影响设置 map 归该消费者。

<a id="lifecycle"></a>
## 运行生命周期、规则确认、恢复与身份隔离

普通运行rules query/capability/确认和不可变operational snapshot唯一归R12；R15消费Match admission/restoration/read-only详情，storage仅限其save/share。包含Windows适用性/锁与callback、genmove retirement、target Cancel≠Run Stop、restart reader/ACK fence、failed synchronization、failed Switch同A恢复、路径quoting/incarnation与本地SGF/node exact restore、本地失败handoff→import→显式Restart→新rules/position→user continue。普通技术调查按当前可达路径取得结论，非源Bug机械移植；没有remote credentials/readboard/benchmark硬前置。

唯一能力责任：`T03-LIFECYCLE-CONCURRENCY-INVESTIGATION`, `T03-LIFECYCLE-RESTART-FENCE-INVESTIGATION`, `T03-LIFECYCLE-TARGET-CANCEL-RUN-STOP`, `T03-LIFECYCLE-WINDOWS-APPLICABILITY`, `T03-MATCH-GENMOVE-RETIREMENT-INVESTIGATION`, `T03-MATCH-RULES-LIFECYCLE`, `T03-RESTORE-GTP-PATH-INVESTIGATION`, `T04-ROLLBACK`, `T04-SYNC-CONFIRM`, `T05-LOCAL-EXACT-RESTORE-CHECK`, `T05-LOCAL-IMPORT-RESTART-CHECK`。本节设置/技术调查与后续实现是同一完整能力；跨节消费不改变来源owner。

详细来源全文（归档合同，不是活跃任务）：

- [B50 — Ordinary engine rules confirmation and immutable Match consumer result](MIGRATION_CONTRACTS.md#b50)：Provide actual ordinary engine instance rules/confirmation and immutable rule-capability result consumed by D Match lifecycle.
- [B51 — Bound Windows lifecycle UI/automatic-task applicability](MIGRATION_CONTRACTS.md#b51)：Answer one bounded applicability/correctness question and stop with an evidence-backed disposition, not a speculative implementation.
- [B52 — Bound lifecycle owner-lock and callback deadlock applicability](MIGRATION_CONTRACTS.md#b52)：Answer one bounded applicability/correctness question and stop with an evidence-backed disposition, not a speculative implementation.
- [B54 — Bound Match streaming genmove retirement and terminal fences](MIGRATION_CONTRACTS.md#b54)：Answer one bounded applicability/correctness question and stop with an evidence-backed disposition, not a speculative implementation.
- [B58 — Bound exact restore path, quoting and incarnation admission](MIGRATION_CONTRACTS.md#b58)：Answer one bounded applicability/correctness question and stop with an evidence-backed disposition, not a speculative implementation.
- [B59 — Bound target Cancel versus explicit Run Stop/exit](MIGRATION_CONTRACTS.md#b59)：Answer one bounded applicability/correctness question and stop with an evidence-backed disposition, not a speculative implementation.
- [B60 — Bound restart reader/ACK/final authority transfer](MIGRATION_CONTRACTS.md#b60)：Answer one bounded applicability/correctness question and stop with an evidence-backed disposition, not a speculative implementation.
- [B63 — Bound failed/timeout position synchronization confirmation](MIGRATION_CONTRACTS.md#b63)：Answer one bounded applicability/correctness question and stop with an evidence-backed disposition, not a speculative implementation.
- [B64 — Bound failed-Switch deferred reader fence and same-A recovery](MIGRATION_CONTRACTS.md#b64)：Answer one bounded applicability/correctness question and stop with an evidence-backed disposition, not a speculative implementation.
- [B65 — Verify supported local SGF/node exact position restoration](MIGRATION_CONTRACTS.md#b65)：Answer one bounded applicability/correctness question and stop with an evidence-backed disposition, not a speculative implementation.
- [B72 — Verify local failed-handback→import→explicit Restart recovery](MIGRATION_CONTRACTS.md#b72)：Answer one bounded applicability/correctness question and stop with an evidence-backed disposition, not a speculative implementation.

最小验收面与用户断言：上述全文的 Acceptance/Concrete acceptance 和 Frozen source/Canonical Delta 条款全部保留；本节 owner 用实际 changed domain/owner/protocol/persistence/UI 面证明成功、失败/Cancel、身份失效、适用 defaults/save/reopen；只读调查以有来源的可判定结论结束，不能宣称功能已完成。每个受影响设置 map 归该消费者。

## 真实 scoped prerequisites 与跨阶段消费

阶段编号是 Delivery Order，不是 whole-stage hard dependency。原来源中已经 Accepted 的所需功能直接按原 evidence tuple 消费；ledger-only 汇总不是等待门。未来 source 的 required/conditional/field scope 在 [公开路由](MIGRATION_ROUTES.json) 和 [来源合同](MIGRATION_CONTRACTS.md) 中保留，不能扁平化成全票或全阶段开工门。

- 本阶段唯一交付 `qualified-local-resource`（`T01-RESOURCE`）：Passed origin/version/path/integrity/capability and visible-failure qualification for the actual resource。其他消费者只取其通过scope；不接管本owner。
- 本阶段唯一交付 `bounded-runtime-diagnostics`（`T01-DIAGNOSTICS`）：Typed redaction/errors/bounded cancellable pinned export for first actual consumer。其他消费者只取其通过scope；不接管本owner。
- 消费 `isolated-benchmark-result`（R13 / `T03-PERFORMANCE-CUSTOM-BENCHMARK-RUNNER`）：Startup evaluation only when it actually consumes measurement。前置仅该已验证结果及适用条件。
- 本阶段唯一交付 `manual-runtime-threads`（`T05-RUNTIME-THREADS`）：Real admitted current-Run manual source/effective/pending/temp Apply/readback。其他消费者只取其通过scope；不接管本owner。
- 本阶段唯一交付 `local-import-explicit-restart`（`T05-LOCAL-IMPORT-RESTART-CHECK`）：Passed local failure/import/explicit Restart/new-rules-position/user-continue ownership。其他消费者只取其通过scope；不接管本owner。
- 本阶段唯一交付 `ordinary-rules-snapshot`（`T03-MATCH-RULES-LIFECYCLE`）：Passed ordinary capability/query/confirmation and same immutable operational snapshot。其他消费者只取其通过scope；不接管本owner。
- 消费 `unified-network-transport`（R14 / `T01-NETWORK`）：Managed remote downloads, not qualified local resource。前置仅该已验证结果及适用条件。
- 消费 `first-ui-locale-foundation`（[R11](R11_PLAN.md#behavior-a01) / `T02-I18N-FOUNDATION`）：Actual new UI integration only; settings owner adds Java field map。前置仅该已验证结果及适用条件。
- 消费 `katago-gtp-main-analysis`（R13 / `T02-ANA-09`）：仅[可显式选择的 GTP 主分析与运行控制的联合验收](#gtp-main-analysis-coordination)，不等待整个 R13；R12 本地资源/规则/Run 与独立参数能力不以该结果为开工前置。

## 具名待决事项、事实输入与受阻行为

以下 owner 是阶段内具名能力责任角色，实施细化时将阶段责任落到实际执行人；不是要求本轮解决所有未来选择。来源/协议/能力未证明保持 needs-info。普通工程设计不造审批；永久非等价才要逐项产品批准。

<a id="r12-gate-b02"></a>
### B02 来源问题（非新任务）

- Owner：R12 / 合格资源、受管维护与显式修复 owner（T01-RESOURCE）；阶段细化负责人具名落实实际执行人。
- 已决定（2026-10-08，用户确认 Q4 推荐方案）：以冻结端点 `af0e07a7386483f3bfc8a15780de72ffc2f0de4c` 的 schema2 目录作为受管身份基线，新装默认采用 B11-12002M；版本、文件名与 digest 见下方事实记录。15 个目标保留来源，只有对应资格成立的操作才开放，不自动下载/升级。
- 保留合同：既有 B11-11750M、其他已安装模型、自定义模型和当前选择不因新装默认变化而替换。历史默认版本明确由本次决定替代，不依赖浮动 latest。
- 所需结果：逐目标可取得性、archive/executable/model/config 完整性及实际协议/平台/硬件资格；维护进度、Cancel/失败和 last-good 保留。产品选择已关闭，B03/B06/B73 实施与验收未完成。
- 原事实输入与边界全文：[来源](MIGRATION_CONTRACTS.md#b02)；以下原来源作为 provenance 保留，以上本轮批准合同是当前决定：

> Freeze catalog origin/source commit/release tag/target/archive/executable/model/config identity, admitted platform/backend/hardware set, explicit maintenance actions, compatibility and offline behavior. Retain all frozen 15 source targets as enumerated sources, not 15 automatically certified supported environments. Decide the specific managed catalog and user repair surface, not production repository, credentials, release signing or channels. Keep public GPU guidance tied to selected resource version; RTX30/40/50 CUDA preference, TRT RTX20/GTX16 and GTX10 rejection are source-version guidance, not permanent universal hardware law. Decide default B11-11750M adoption or non-adoption with named version/reason; active custom selection stays untouched.

<a id="r12-gate-b13"></a>
### B13 来源问题（非新任务）

- Owner：R12 / 启动、默认引擎、预加载与评估让路 owner（T02-H17）；阶段细化负责人具名落实实际执行人。
- 已决定（2026-10-08，本轮用户对 Q1 回复“都按推荐来”）：提供互斥的“不自动启动／固定默认引擎／上次主引擎”三模式。既有配置保持原行为；新安装默认不自动启动。
- 保存合同：上次主引擎使用稳定 profile ID，在正常退出时记录；`selected_profile_id` 仍只是编辑身份。异常终止不承诺保存本次选择，沿用最后成功写入的记录；取消退出不写成一次正常退出。
- 失败合同：记录缺失、损坏、profile 已删除或启动失败都不选择其他 profile 兜底；启动失败保持 No-engine 并给出可见原因。持久化失败可见，不伪称已保存。切换启动模式或保存设置不隐式 Start/Switch/Restart。
- 所需结果：B14 所代表的完整启动能力及成功、保存失败、退出取消、异常退出、无可用身份和重开验收；产品选择已关闭，实施和真实验收未完成。
- 原事实输入与边界全文：[来源](MIGRATION_CONTRACTS.md#b13)；以下原来源作为 provenance 保留，以上本轮批准合同是当前决定：

> Java autoload-last remembers the last primary on normal shutdown; missing last-engine=-1 and corrupt index must not choose another profile. Next selected_profile_id is editing identity, not startup identity; existing zero-or-one Autoload Default remains accepted. Inspect original corresponding disposition ticket and approval source already named in frozen Inventory/audits, not new Java investigation. Missing approval is not permission to delete. Decide Startup mode/default, stable last-run profile identity, normal-shutdown/crash persistence, deleted/corrupt identity and failure no-fallback.

<a id="r12-gate-b15"></a>
### B15 来源问题（非新任务）

- Owner：R12 / 启动、默认引擎、预加载与评估让路 owner（T02-H18）；阶段细化负责人具名落实实际执行人。
- 已决定（2026-10-08，本轮用户对 Q2 回复“都按推荐来”）：保留额外引擎后台预加载；每个配置显式开关、默认关闭并持久化。收益是缩短切换等待，代价是额外进程、内存和显存占用。
- 所有实例由同一个 engine-manager 管理；后台实例有可见准备/失败状态并可取消，不获得当前棋谱、分析或 Match 的权威。只有用户显式切换且重新核对资源/profile 身份与 Ready 条件后，才允许成为主引擎。
- 前台用户工作优先；资源不足或资格失败时可见拒绝/失败，保留当前健康主引擎和棋谱，不自动改用其他后端。预加载退出、取消与失效后的子进程清理属于本能力，不建立独立调度器或隐藏自动重启。
- 所需结果：B16 所代表的完整预加载能力及持久化、取消、资源失败、显式提升、过期事件和正常退出清理验收；产品选择已关闭，实际协议/资源准入和验收未完成。
- 原事实输入与边界全文：[来源](MIGRATION_CONTRACTS.md#b15)；以下原来源作为 provenance 保留，以上本轮批准合同是当前决定：

> Java saved non-primary preload defaults false and is not foreground promotion. Next processes currently exist only under Switch/Match Reservation. Preserve preload user goal without approving hidden parallel lifecycle by assumption. Inspect original corresponding disposition ticket and approval source already named in frozen Inventory/audits, not new Java investigation. Missing approval is not permission to delete. Decide Opt-in entry/default/save, background readiness display, single-manager reservation ownership, cancellation/resource failure and foreground priority.

<a id="r12-gate-b42"></a>
### B42 来源问题（非新任务）

- Owner：R12 / 手动运行线程来源、临时覆盖与实际值 owner（T04-THREAD-CONTROL, T05-RUNTIME-THREADS）；阶段细化负责人具名落实实际执行人。
- 已决定（Q6）：动态线程确认绑定显式选择的合格 KataGo GTP Run；保留 JSONL，并将 GTP 主分析与 R13 联合安排，见[联合交付边界](#gtp-main-analysis-coordination)。不以另一个进程、静态启动值或查询覆盖冒称当前实例回读。
- 待决定/取得证据：Which real dynamic capability and manual source/effective/temporary precedence, input domain and expiry/reset semantics are admitted?
- 受阻行为：Manual dynamic thread Apply and confirmed readback
- 所需结果：具来源/实际能力证据的可执行合同；涉及非等价处置保留逐项批准。
- 原事实输入与边界全文：[来源](MIGRATION_CONTRACTS.md#b42)；以下保留该事实/合同输入，不表示已经选定新答案：

> Inputs distinguish saved config, launch policy/explicit override, current Run actual value, temporary requested value and user unsubmitted edits. Decide manual dynamic capability/version/valid input domain from actual protocol evidence, not Java1..1024. Freeze mouse/keyboard explicit Apply, confirm-readback failure/timeout, pending versus effective, expiry on Restart/Switch/disconnect, reset-to-config and no automatic reapply on reconnect. Initialization cannot kata-set-param legacy numSearchThreads over resolved launch override. Dynamic manual path must be executable without runner/policy recommendation. Consuming recommendation is separate B41 integration; H-STATIC proves only explicit Restart argv2.

<a id="r12-gate-b47"></a>
### B47 来源问题（非新任务）

- Owner：R12 / 启动、默认引擎、预加载与评估让路 owner（T04-STARTUP-PERFORMANCE）；阶段细化负责人具名落实实际执行人。
- 已决定（2026-10-08，本轮用户对 Q3 回复“都按推荐来”）：保留明确的性能评估入口和“启动时评估”持久开关，默认关闭；只有用户开启后才允许启动触发。
- 用户分析或对局优先；让路与用户取消不是引擎故障，不丢失 Pause 意图，不隐式切换/重启引擎，不自动应用线程建议。评估与额外引擎预加载是不同能力，各自开关不能互相授权。
- 所需结果：B48 所代表的明确引擎选择、启动触发、取消/让路和可见结果的完整能力。测量仅消费 R13 `isolated-benchmark-result`；不等待 saved policy 或整阶段 R13，也不以产品决定冒充实际 benchmark 证据。
- 原事实输入与边界全文：[来源](MIGRATION_CONTRACTS.md#b47)；以下原来源作为 provenance 保留，以上本轮批准合同是当前决定：

> Frozen PR500/504 and preload H18 have retained performance user goal; single-lifecycle constraint is not approval to delete it. Decide explicit entry or admitted startup trigger/default/save behavior, engine selection, user analysis/Pause priority and yield versus true failure feedback. Cite approved individual disposition or retained successor. No actual benchmark required in decision and no Java independent process scheduler migration. This does not block manual Benchmark/Threads or existing current-analysis handoff investigations.

<a id="r12-gate-b67"></a>
### B67 来源问题（非新任务）

- Owner：R12 / PDA/WRN 成对只读回读 owner（T05-PARAMETER-READBACK）；阶段细化负责人具名落实实际执行人。
- 已决定（Q6）：成对回读绑定同一个合格 KataGo GTP 主分析 Run；[联合交付](#gtp-main-analysis-coordination)包含真实分析与控件的组合验收。已取得独立 binary 成功回读证据，Next adapter/reader/round 及原生显示仍未验收。
- 待决定/取得证据：Which named actual local-engine binary/version supports both PDA/WRN readback commands, as proved by B67 minimal real protocol evidence for current reader/Run/round/numbered responses?
- 受阻行为：Paired PDA/WRN current-Run readback
- 所需结果：具来源/实际能力证据的可执行合同；涉及非等价处置保留逐项批准。
- 原事实输入与边界全文：[来源](MIGRATION_CONTRACTS.md#b67)；以下保留该事实/合同输入，不表示已经选定新答案：

> Bind PDA and WRN readback to current reader/Run/request round/numbered responses; both valid finite values are required for current pair success. Identify actual compatible engine binary/version, command/value domains and round retirement on cancel/reconnect/new Run from minimal real protocol evidence, not static argv or chart encoding. Default display is unknown until a valid current pair; invalidation preserves the last valid values with visible failed/unknown state. Current response must not overwrite unsent edits. Preserve existing editing behavior; this ticket neither pairs writes nor adds mutation/retry/reapply semantics.

<a id="r12-gate-b04"></a>
### B04 来源问题（非新任务）

- Owner：R12 / 合格资源、受管维护与显式修复 owner（T03-RESOURCE-TRT-REPAIR-INVESTIGATION）；阶段细化负责人具名落实实际执行人。
- 已决定（2026-10-08，用户确认 Q5 推荐方案）：应用承担已准入 TensorRT 目标的显式受管修复；动作前展示资源版本、下载量、磁盘需求与影响，过程可取消，失败保留 last-good 和原目标。
- 边界：不自动安装系统驱动，不替换成其他后端；外部或未准入资源给出明确责任/资格缺口，不宣称受管修复成功。前台使用非 TRT 引擎不改变用户选定 TRT 目标的身份。
- 所需结果：B04 的逐目标维护所有权和可修复边界、B05 的实际 NVIDIA 资格及 B73 的修复/回滚验收；产品责任已确定，具体目标可修复性尚未获证。
- 原事实输入与边界全文：[来源](MIGRATION_CONTRACTS.md#b04)；以下原来源作为 provenance 保留，以上本轮批准合同是当前决定：

> Use frozen resource acquisition/replacement boundaries, not new Java diagnosis. Compare selectable TRT identity to maintenance ownership even with non-TRT foreground engine. Stop at supported/external-owner/needs-decision table and exact retained-resource failure/cancel contract. No repair implementation before this result; a supported repair successor must be linked to managed-resources and retain last-known-good, original user target and explicit action. No new engine compile.

<a id="r12-gate-b05"></a>
### B05 来源问题（非新任务）

- Owner：R12 / 合格资源、受管维护与显式修复 owner（T03-RESOURCE-NVIDIA-HARDWARE-GATE）；阶段细化负责人具名落实实际执行人。
- 待决定/取得证据：Actual NVIDIA qualification/unknown and supported hardware responsibility remains unresolved until investigation result.
- 受阻行为：Reachable acceleration setup actions and status layout; Approved explicit target-directed TensorRT resource repair
- 所需结果：具来源/实际能力证据的可执行合同；涉及非等价处置保留逐项批准。
- 原事实输入与边界全文：[来源](MIGRATION_CONTRACTS.md#b05)；以下保留该事实/合同输入，不表示已经选定新答案：

> Compare target metadata to actual available probe and hardware evidence, including a non-TRT foreground selection. Unknown or absent GPU remains pending, never pass. Determine whether resource repair needs NVIDIA qualification and transaction rollback from the TRT ownership result; do not broaden to general engine builds or all GPUs.

## 集成、实际验收与环境义务

- 具名 owner：**R12 运行管理集成/实际验收 owner**；阶段细化时明确实际负责人，拥有本阶段 changed composed workflow 的实际验收，不能交给 read-only Closeout。每个能力owner仍对其局部断言和真实surface负责。
- 环境：合格local binary/model与实际已准入backend/平台；Windows原生lifecycle/credential-independent local restore；真实NVIDIA目标仅对应TRT准入，不以CPU代证。
- 集成消费已经归属的 original candidate/条件证据，只执行最终 changed boundary gap；各全文适用 required modes/协议/失败/身份/恢复断言完整保留，不新增无理由次数、平台或全组合。没有兼容真实引擎/服务/凭据/目标硬件，完成reachable独立工作但对应门保持受阻。
- 阶段退出：本阶段全部 required capability/Delta 有实际结果或逐项批准处置，required local/integration/native/service assertions 获证，保留残余发行或独立引擎编译 Not run。阶段实现与集成完成后统一 Spec/Standards 审查和集中修复、实际验收，再只读 Closeout；不逐票重复双轴审查。
- 验收由对应能力与阶段负责人取得；本节保留原阶段门禁，[完成记录](#r12-completion) 只引用其已执行证据，不由计划文字代替产品/native/真实服务/installed trust 验收。

## Baseline 与阶段启动细化规则

- 仓库 `qiyi71w/lizzieyzy-next-tauri`；历史调查 `55795fab49b80a681a9fa544950e18f5f2da4cc6`；历史来源细化快照 `15b71e8bd89349961b7cf25f2fd2a6928e991e40`（不是未来实施起点）。Java v1 `7b4027531c2b26062d0bfc27a040cc550cfbea4d`、冻结审计端点 `af0e07a7386483f3bfc8a15780de72ffc2f0de4c` 不变。
- 本文件是阶段计划，不是未来已发布任务组；历史草案仅详细来源合同，不构成第二活跃 tracker，也不把来源意图数当任务数。未决问题无需本轮全部决定，但受阻行为不能遗漏。
- 阶段启动前由本阶段负责人消费 [Architecture](ARCHITECTURE_NEXT.md)、[Migration Plan](MIGRATION_PLAN.md)、[公开路由](MIGRATION_ROUTES.json)、本阶段计划及实际 scoped validated prerequisite 的最终完成记录；无前置按完成记录冻结精确实施提交，不自行替换 latest main；有前置读取实际完成/集成 full SHA 和条件，用已验证包含所需结果的提交。多个未合并成果由具名串行集成 owner 在隔离树合成并验证后冻结起点，不能编造未来 SHA；相同 HEAD 不证明证据实体已经可取得。
- 将普通技术调查/字段映射/实现步骤收进完整垂直能力；只有真实材料性产品/服务/引擎准入问题保留具名 owner 与门。按阶段滚动细化完整垂直能力、fresh reader 可执行性冷读、呈现实施清单并另获发布批准；本阶段结构批准不授权产品实施或未来全部票发布。
- Rust current-game/session 权威、单 engine lifecycle、无隐式 Switch/Restart/fallback、Cancel/失败不破坏当前谱、统一网络和凭据保护及五 ADR 保持。新 UI 消费 [R11 首 search UI 本地化 key/locale/fallback 结果](R11_PLAN.md#behavior-a01)；各设置 owner 同步 Java 字段语义 map，R17 集合闭环，不等待整 R11 或完整翻译。
- 继承证据必须保留原 candidate full SHA、behavior/platform/engine/helper/service/resource/version 与未验条件。仅文档归类不重验产品；相关 behavior/DTO/protocol/Run/job/document/account/resource/platform/service 改变使受影响证据失效，由实际消费者取得 focused real revalidation。仓库/fixture/browser 与原生/真实服务不同；应用运行、发行打包、上游引擎编译三者独立。没有实际执行的门保持 Not run/needs-info。

<a id="refinement-20261008"></a>
## 2026-10-08 阶段细化：起点、能力分组与验收

历史上下文：本节保留阶段细化时的调查与授权边界；其中“未授权发布实施票”“本轮不自动进入”等表述描述当时的规划步骤。后续批准 spec 的实施、required acceptance、共享审查及 Closeout 已完成，见[完成记录](#r12-completion)。历史 HEAD、路径存在性检查与协议探测不是当前实现或验收结论；合格本地资源的现行实现合同见 [Architecture](ARCHITECTURE_NEXT.md#cratesengine-manager)。

### 调查起点与批准边界

- 仓库 `qiyi71w/lizzieyzy-next-tauri`；本轮已核对 origin 并 fetch `main`，调查基线及当前 HEAD 固定为 `9d2ccf3ba6881c755013b82948e5da1a01f7fcd4`，含 [R11 PR #20](https://github.com/qiyi71w/lizzieyzy-next-tauri/pull/20) 的合并结果。
- 独立规划工作树 `/home/dev/dev/weiqi/worktrees/lizzieyzy-next-tauri/r12-planning-20261008`，分支 `docs/r12-migration-plan`；原工作树未提交内容不作本轮基线、不覆盖。后续 spec/拆票继承本轮调查基线；产品实施起点另核对所消费 scoped result 的完整包含 SHA 和完成记录，不自动追新 main。
- R11 PR 报告其原候选验收与集成审查，不能据“已合并”扩大为 R12 的资源/参数/预加载验收。仅按实际 unchanged boundary 消费 locale、原 profile 保存/排序、现有生命周期等前置；遗留局部 tracker 的 waiting 文本与最终交付证据不一致时，以具名最终记录核对，不复制旧状态或凭 HEAD 推定实体存在。
- 本轮保留原七组能力、25 个唯一 owner intent、46 个 source intent 和 34 份 B 来源合同；这些是覆盖索引，不是票数。历史合同、原 Accepted 范围及 source/Delta 引用不删除。普通字段映射、源码调查和故障边界设计收进对应完整能力。
- 用户已批准 Q1–Q6，并明确要求“将主分析的 GTP 接入纳入协调范围，与 R13 分析能力共同安排”。这是阶段产品范围与联合依赖决定；未通过的资源、实际协议与硬件资格保持具名门，不以本地 probe 或本计划代替产品验收，未授权发布实施票或开始产品实现。

### 资源与协议事实：2026-10-08

- **目录身份。** 冻结 Java 端点的 [schema2 目录](https://github.com/wimi321/lizzieyzy-next/blob/af0e07a7386483f3bfc8a15780de72ffc2f0de4c/src/main/resources/katago-assets.json) 声明 KataGo `1.18.2`、源码 `47aadc08518b3e121f22539796c911002f699584`、engine release `wimi321/lizzieyzy-next@next-2026-10-07.1`、model release `v1.17.1`。这是来源身份；尚未证明 Release 附件可取得或 Next 对这些资源具备运行资格，也不决定 Next 产品的发布仓库。
- **实施批准的资源 tag。** 2026-10-08 用户批准使用已发布的 `next-2026-10-08.1` 作为受管引擎资源 tag。冻结端点仍是来源 provenance；源码、版本、15 个目标、模型、文件名、大小及全部摘要不变。已核对该发布的 CPU/TRT 附件元数据摘要与冻结目录一致，实际下载后仍须校验；这不是原生资格证明、代码基线更新或 Next 产品发布渠道选择。
- **15 个来源目标。** Windows：`windows-cpu`、`windows-opencl`、`windows-nvidia`、`windows-tensorrt`、`windows-directml`、`windows-openvino`、`windows-rocm-gfx103x`、`windows-rocm-gfx110x`、`windows-rocm-gfx1151`、`windows-rocm-gfx120x`；Linux：`linux-cpu`、`linux-opencl`、`linux-nvidia`；macOS：`macos-arm64`、`macos-amd64`。目录列举、资源下载可用、实际硬件支持、Next 原生验收是四个不同状态。每个实际开放的维护动作须消费自己的资格结果；未验目标保留来源与 unknown 状态。
- **默认模型差异。** [UD-04-037 对应历史目录](https://github.com/wimi321/lizzieyzy-next/blob/498f5f05119e7a7cbab6ed124d8656fcd7940196/src/main/resources/katago-assets.json) 的 B11 为 `kata1-tf3-b11c768-s11750M-d6216M.bin.gz`；冻结端点已为 `kata1-tf3-b11c768-s12002M-d6304M.bin.gz`，SHA256 `4a6312e80faadee7b7dd28689a2e87a1efb4640c10132f16290da7a17b4c6d9e`，262017809 bytes，目录 minimum KataGo `1.17.0`。因此 B02 的“11750M 采纳”不能未经决定就被“最新版”替换。目录元数据不证明棋力、速度或本机兼容。
- **调查基线的 Next 接入（历史）。** `crates/engine-manager/src/lib.rs` 的 `build_command_spec` 为 `KataGoAnalysis` 启动 `analysis` JSONL；当时 `check_assets` 主要检查路径存在性。`lifecycle.rs` 的 GTP readiness 是基础命令握手，既有具名运行证据为 GNU Go 3.8 move-only。`crates/katago-protocol/src/lib.rs` 的查询设置尚未暴露运行参数控制。因此独立 binary 支持命令不等于 Next adapter 已支持。当前本地资源准入已增加绑定保存 profile revision 的启动前/Ready 后实际内容校验；这不证明 GTP 主分析或运行参数能力已完成。
- **协议区别。** [KataGo v1.16.0 GTP 扩展](https://github.com/lightvector/KataGo/blob/v1.16.0/docs/GTP_Extensions.md) 中，回读命令为 `kata-get-param numSearchThreads`、`kata-get-param playoutDoublingAdvantage`、`kata-get-param analysisWideRootNoise`。[Analysis JSONL](https://github.com/lightvector/KataGo/blob/v1.16.0/docs/Analysis_Engine.md) 的 `overrideSettings` 是单次查询配置，WRN 字段为 `wideRootNoise`；其成功分析响应不能冒称独立的实例参数回读。不得向 JSONL stdin 注入 GTP 文本或用另一个隐藏进程的值冒充前台 Run。
- **线程能力按版本判断。** [v1.16.0 查询参数加载](https://github.com/lightvector/KataGo/blob/v1.16.0/cpp/command/analysis.cpp) 调用 [SearchParams 可变性检查](https://github.com/lightvector/KataGo/blob/v1.16.0/cpp/search/searchparams.cpp)；该版本检查并未把 `numThreads` 列为不可变参数，不能一概声称所有 JSONL 版本都禁止调整线程。查询覆盖、运行中搜索调整、独立实际值回读是不同能力；B42 要按实际准入版本分别取得证据，不能把查询接受当作 B43 已确认实际值。

本轮真实协议探测：Windows CPU `D:\katago\yzy\katago_cpu_avx2\katago.exe`，引擎报告 `1.12.3`，binary SHA256 `56008013aec6e6c7491970ad8fa740ee7ac4df169778e98a6ee65eb335d6d0b4`；模型 `D:\katago\yzy\weights\kata1-b20c256x2-s5303129600-d1228401921.bin.gz`。使用隔离临时 cwd/config，`maxVisits=1`、搜索线程初始值 1、缓存参数 16/12、关闭搜索日志；独立 owned child 正常退出，临时目录清理，用户配置未修改。模型内容未另取 digest，因此证据仅绑定本次所用路径，不作为未来资格凭据。

| 模式与请求 | 实际结果 | 能证明的范围 |
| --- | --- | --- |
| GTP `known_command kata-get-param` / `kata-set-param` | 两项 `true`，但 `kata-get-param numSearchThreads` → `Invalid parameter` | 命令存在不证明每个参数可用；该 binary 不能用于完整线程 Apply＋确认回读验收 |
| 同一 GTP 进程 `101 kata-get-param playoutDoublingAdvantage`、`102 kata-get-param analysisWideRootNoise` | `=101 0`、`=102 0.04`，随后 `103 quit` 成功、exit 0 | 两个正确参数名及有限值成功回读；尚未证明 Next reader/round retirement、失败分支或 UI |
| JSONL 初始 `numSearchThreads=1`、`numAnalysisThreads=1`，单查询覆盖 `numSearchThreads=2` | `field=overrideSettings`，错误 `Cannot increase number of search threads after initialization since this is used to initialize neural net buffer capacity`；exit 0 | 仅该旧 binary 的增加线程限制；不推导新版本、减少线程或动态回读能力 |

### 已批准产品决定：Q4–Q6

- **Q4／B02：目录与新装默认模型。** 冻结端点目录为受管身份基线，新装默认 B11-12002M；已安装 11750M、自定义模型和当前选择保持。15 目标按实际资格开放操作，不自动下载/升级。完整合同见 [B02](#r12-gate-b02)。
- **Q5／B04：TensorRT 修复责任。** 应用内显式受管修复原 TRT 目标，展示下载/磁盘需求、可取消并保留 last-good；不自动安装系统驱动或切后端。完整合同见 [B04](#r12-gate-b04)。
- **Q6／B42、B67：主分析与真实参数回读。** 保留现有 JSONL 主分析与配置，增加用户显式选择、由同一 lifecycle manager 管理的 KataGo GTP 运行能力。GTP 接入必须覆盖主分析及其动态线程确认、PDA/WRN 成对回读的组合场景，与 R13 的分析能力共同安排。既有 JSONL 配置不自动转换；两个协议按各自真实能力显示状态，不能互相冒充回读。

批准来源为本轮用户“确认 Q4–Q6 推荐方案，并将主分析的 GTP 接入纳入协调范围，与 R13 分析能力共同安排”。具体 DTO、模块划分、数值预算和参数探测归执行者；B05 实际 GPU 资格及 B42/B67 具名 adapter/版本验证仍为执行门。没有新增未决产品取舍；发现协议无法满足上述已批准目标时报告具体差异，不自行缩减。

<a id="gtp-main-analysis-coordination"></a>
### R12/R13 联合交付：可显式选择的 GTP 主分析

- **职责。** R12 保持资源、profile/Run/process 生命周期、规则/局面确认、临时线程与参数回读的唯一运行管理责任；R13 的 `T02-ADAPTER-EVIDENCE` / `T02-ANA-09` 负责具名 KataGo GTP 分析能力准入、流解析、领域结果和实际主分析展示，详见 [R13 联合合同](R13_PLAN.md#gtp-main-analysis)。不另设第二 manager、隐藏 companion engine 或重复来源 owner。
- **前置。** R13 的真实分析接入消费 R12 已验证的对应 `qualified-local-resource`、Run/reader 边界和 `ordinary-rules-snapshot`；资源、普通规则与 Run 就绪本身不依赖 GTP 分析展示。协议样本调查与 R12 参数能力调查可独立推进。R12 的组合验收再消费 R13 `katago-gtp-main-analysis` 结果，不等待整个 R13 的 batch、tracking、自动分析或性能工作流。
- **完成条件。** 同一个实际 GTP 主分析 Run 在当前局面产生可信候选/胜率/PV及准入字段，用户可 Pause/Continue、导航、显式切换与 Restart；同时线程 Apply 获确认、PDA/WRN 成对回读且控制响应与分析流互不串包。旧 Run/reader/round 输出不能污染新局面或释放 Pause。缺失可选字段显示 unavailable；必需主分析能力缺失时该路径不得被标记完成。
- **组合出口。** R13 提供具名 binary/model/config/平台、完整应用 SHA 和主分析结果；R12 集成验收负责人在同一集成候选上负责运行控制＋分析组合场景，R13 分析 owner 负责字段/展示断言。变更过的边界重验，保留 JSONL 当前能力并验证来回显式切换/失败保留原 Ready A。仅命令握手、move-only GTP 或独立参数控件不能结束该合同；R12 完整完成声明必须包含这份 scoped 联合证据。

### 垂直能力分组与真实前置

每组完成到 Rust 领域行为、DTO、Tauri/API、实际 UI、持久化（如适用）和用户可观察验证，不按后端/前端/测试拆成不能单独使用的阶段。阶段集成负责人统一协调共享 Run/process/profile schema 的修改。

| 分组（非正式票号） | 完整交付与来源覆盖 | 真正前置／可独立推进的部分 | 必须提供的结束证据 |
| --- | --- | --- | --- |
| 合格资源、维护与修复 | B01–B06、B73：本地 origin/version/path/integrity/capability；受管获取/维护/取消、版本对应 GPU 提示、保持原目标的 TensorRT 修复 | 本地资格直接消费现有 ENG-02，不等 managed catalog。远程获取只等实际下载 transport；managed/repair 等 B02/B04/B05 适用结果，不等 R18 签名分发 | 实际合格本地 Start；missing/incompatible Start→No-engine 与失败 Switch→原 Ready A；维护中断/Cancel/坏归档不破坏 last-good；原 TRT 目标及真实硬件结果，unknown 不视为通过 |
| 模型身份与已安装保留 | B07–B08：真实 header/content、catalog、saved selection 和 installed ownership；同路径替换重验，B11→B10→B11 保留已安装候选 | header/本地样本与资源身份合同可先行；目录集成只消费对应已验证资源结果。解析格式的调查不是独立产品审批 | 已安装和自定义候选不因刷新消失；rename/manual command 不改变 ownership；旧 snapshot/busy/失败不能误启用或重定向选择；真实模型/引擎选择 |
| 启动、预加载与评估 | B13–B16、B47–B48：已批准三模式、opt-in preload 和 startup evaluation | 启动策略不等 preload 或 benchmark；预加载只等所用资源/adapter 的准入与 manager ownership；评估只等 R13 `isolated-benchmark-result`，不等 saved recommendation/measured report | 旧配置等价迁移、正常退出/取消/异常结束/重开；后台实例无前台权威、显式提升和退出清理；评估让路/取消不丢 Pause，不自动应用建议 |
| 首消费者诊断与控制台 | B09–B10、B61：有界 ordinary log/WARN/stdout/stderr、原 attempt 身份、可读详情/Copy、脱敏 pinned export、响应式控制台及 shutdown cleanup | 数值预算与采集表示由执行者根据真实来源定界。必要错误/脱敏随首消费者交付；其他能力不等待整个导出 UI 或 installed support 包 | 延迟 A 输出不进入新 Run B；显示/Copy/ZIP 同脱敏；导出固定所见 attempt、可取消且原子发布；真实 Windows 长文/DPI、缺指标、写失败、关闭及有界 cleanup |
| 手动线程来源与临时 Apply | B42–B44：saved CFG/argv、effective launch、pending request、current actual、temp 与 draft 分离 | ENG-01/PREF-01 与真实具名动态协议；不依赖 ENG-14 benchmark 或建议保存。仅消费别名调查中实际可达的配置/include precedence | mouse/keyboard 显式 Apply 后真实确认；超时/错误/旧 Run 保留最后有效显示并标失败/unknown；保护 draft；Restart/Switch/disconnect 失效，不自动 reapply，不改用户 CFG |
| PDA/WRN 成对只读回读 | B67–B68：同 reader/Run/round/numbered response 的两个 finite 值；未知、失败与 last-valid 明确区分 | 支持两个参数的实际 binary/version/adapter；不依赖线程 Apply、benchmark 或 paired write | 初始 unknown；缺半、乱序、超时、取消、旧 reader/Run 不合成当前 pair；真实本地协议和原生显示证据；不覆盖未提交编辑，不新增写入/retry/reapply |
| 生命周期、规则与本地恢复 | B50–B52、B54、B58–B60、B63–B65、B72：普通规则确认与 immutable operational result；锁/callback、target Cancel、genmove retirement、reader/ACK、失败同步/回滚、精确局面与 import→Restart→Continue | 当前已支持本地路径无需远程凭据/readboard/benchmark。新 rules 消费合格资源；R15 只消费 ordinary-rules-snapshot，保存/共享决定不倒置为普通确认前置 | 有界路径/适配器/证据/残余表；规则请求不是确认，旧 ACK 不解锁新局面；真实引擎核对 stones/to-play/KM/真实尾手；发现真实差异才安排有界修复 |

### 集成顺序与共享边界

1. **事实与准入先行，但只阻塞消费者。** 固定每个实际资源及其能力；源支持、独立 binary 探测、Next adapter 支持、原生验收分别记录。Q1–Q6 产品决定已关闭；B02/B04/B05 资源目标资格、B42/B67 参数证据和 R13 B28 分析准入按各自真实缺口推进，不重新作为同一产品访谈门。
2. **先形成可用的本地启动/失败诊断组合。** 复用现有 profile catalog、manager、typed failure 和 locale；资格检查、最小诊断与三模式启动各自产出可验结果。旧路径不能留作未校验 fallback，保存或浏览资源不触发 Start。
3. **按真实前置推进其他能力。** 模型与受管资源对齐身份；预加载、普通规则/恢复、动态线程、成对回读分别消费同一 manager 的已验证边界。它们没有按表格顺序等待的硬边；同一状态机/schema 的变更由串行集成负责人落地，不能由多个 owner 建各自进程权威。
4. **跨阶段只提前消费所需结果。** 下载复用现有 NET-01 的已验证 HTTP 能力；确需补齐的 transport 由 R14 `T01-NETWORK` 唯一负责。R13 runner 可先消费 `qualified-local-resource`，再交给 R12 启动评估；GTP 主分析按上述联合合同先消费 R12 运行边界，再提供 `katago-gtp-main-analysis` 给组合验收。两条路径都不把消费者的最终完成反设为 provider 的开工前置。
5. **一次组合出口。** 全部能力实现与适用集成检查后，统一 Spec/Standards 审查、集中修复及实际验收，再只读 Closeout。执行者名称/候选 SHA 在正式任务分配时落实，不在规划中编造。若用户选择整阶段实施，可由 `/implement-spec` 按批准票整票负责人和独立串行 merger 执行；本轮不自动进入该流程。

实现放置遵守现有边界：`crates/engine-manager/src/catalog.rs` 管 profile 持久化，`lifecycle.rs` 及其内部模块管 Run/process/retirement；`crates/app-model` 先定义 wire 类型再改 TS 副本；协议逻辑留在 crate。`apps/desktop/src-tauri/src/document_departure.rs` 只接入正常退出/取消和资源 teardown，不能另造退出保存器；UI 通过 `apps/desktop/src/api` 消费领域状态。新增模块按真正依赖压力决定，不先建通用资源/任务框架。

### 设置映射、升级和验收保护

- 每个设置 owner 给出 Java key、旧默认/保存时机、Next 对应语义、缺失/损坏处理与实际读写消费；R17 只收集这些映射做明确白名单导入，本阶段不读写用户 Java 配置。
- Q1 从现有零或一 `autoload_profile_id` 映射“不自动／固定默认”，不从编辑中的 `selected_profile_id` 猜启动对象；新增“上次”单独持久身份。配置排序不能改变这些 ID，保存失败不能改变已公布的 durable setting 或当前 Run。
- Q2 的持久开关与瞬时进程/准备状态分离；恢复棋谱不恢复 job/Match/运行状态。旧 profile 没有 preload 字段时为关闭，保存草稿本身不授权启动。
- Q3 的 opt-in 与一次评估请求/结果分离；没有 identity-valid 结果就显示尚未评估，而非默认“推荐值”。手动线程不自动消费这个结果，R13 建议策略仍归原 owner。
- 共享 schema 切换一次迁移所有读写调用方及 browser fallback，不留下双写/旧接口兼容层；保留既有独立 Autoload、settings selection、foreground Run 和 pending draft 身份。

### 组合验收清单与停止条件

- **启动与退出：** 旧固定默认/关闭行为、新模式及坏身份；退出 Cancel/SaveAs Cancel 不记录虚假正常退出；正常退出失败写盘可见；应用重开仅按持久策略启动，不恢复历史 job。修改已选但未启动的编辑配置不误记为 last-primary。
- **资源／模型／预加载：** 前台 A 运行时维护或准备 B，Cancel/失败/旧 refresh 不影响 A 与当前谱；显式切换只提升当前有效 B；同路径内容变更使旧资格失效；取消与正常退出不残留 owned child。TRT 实际验收只对应具名 target/backend/GPU/driver，不推导所有15来源都合格。
- **线程与参数：** 请求在 Run A 发出后切换/Restart 到 B，A 的晚回读不能成为 B 的 actual/pair；未提交 draft 保留；Apply 失败不伪装生效；单参数成功不足以完成 pair。无实际兼容引擎时保留 blocked，而非用 static argv 或 mock 结束功能。
- **GTP 主分析联合场景：** 按 [R13 合同](R13_PLAN.md#gtp-main-analysis)执行分析流与编号控制响应交错、线程 Apply/回读、参数 pair、Pause/Continue、节点切换、Restart、失败 Switch 与 JSONL 往返切换；同一实例、局面和身份链获得真实引擎及原生证据后，才关闭主分析接入要求。
- **规则与局面：** 现有 supported SGF/node 路径的 root PL、alternating/PASS、KataGo 黑方 root handicap、前后导航与分支；未支持的 setup 按既有合同拒绝且不改谱/Run。失败 handback→新 SGF→显式 Restart→新 rules/position 确认→用户 Continue；旧 callback/ACK/permission 不得解除暂停或向新局面发布旧结果。
- **诊断与评估：** 当前显示失败在 Retry/Switch 后导出仍固定原 attempt；secret/control-character 脱敏、取消/写失败/估算过期/发布后打开目录失败按完整合同验证。启动评估遇到用户分析/对局让路，保留 Pause；不自动修改 CFG、线程建议或 active profile。
- 仓库/受控子进程、真实引擎协议、Windows 原生、GPU 目标、installed/signed release 分栏，记录完整应用 SHA 与 binary/model/config/resource/平台条件。只重验改变的边界；静态样例、浏览器、审查通过和 PR merged 都不是实际运行资格。全部 required 断言或逐项批准处置齐备才退出 R12；环境未具备不静默缩小本阶段范围。

## 覆盖与来源导航

[公开路由](MIGRATION_ROUTES.json) 记录唯一阶段归属；[来源意图](MIGRATION_SOURCE_MAP.md) 保留来源含义；[来源行为合同](MIGRATION_CONTRACTS.md) 提供本阶段所引用的完整静态合同。R12–R18 来源集包含 124 owner intents、163 source intents、149 历史合同和 44 具名未来门；数量用于来源覆盖，不是实施任务配额。历史 50 needs-info 的实际待决/普通技术步骤按本阶段正文处理；具名未来门保留原 ID，未证明的能力不自动完成。
