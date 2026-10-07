# R12 — 引擎资源与运行管理

Status: 滚动阶段计划；产品能力验收尚未完成。

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

## 具名待决事项、事实输入与受阻行为

以下 owner 是阶段内具名能力责任角色，实施细化时将阶段责任落到实际执行人；不是要求本轮解决所有未来选择。来源/协议/能力未证明保持 needs-info。普通工程设计不造审批；永久非等价才要逐项产品批准。

<a id="r12-gate-b02"></a>
### B02 来源问题（非新任务）

- Owner：R12 / 合格资源、受管维护与显式修复 owner（T01-RESOURCE）；阶段细化负责人具名落实实际执行人。
- 待决定/取得证据：Which managed catalog targets and explicit maintenance UI are admitted, with what immutable trust/default/resource transaction contract?
- 受阻行为：Managed resource maintenance and version-qualified GPU guidance; Reachable acceleration setup actions and status layout; Approved explicit target-directed TensorRT resource repair
- 所需结果：具来源/实际能力证据的可执行合同；涉及非等价处置保留逐项批准。
- 原事实输入与边界全文：[来源](MIGRATION_CONTRACTS.md#b02)；以下保留该事实/合同输入，不表示已经选定新答案：

> Freeze catalog origin/source commit/release tag/target/archive/executable/model/config identity, admitted platform/backend/hardware set, explicit maintenance actions, compatibility and offline behavior. Retain all frozen 15 source targets as enumerated sources, not 15 automatically certified supported environments. Decide the specific managed catalog and user repair surface, not production repository, credentials, release signing or channels. Keep public GPU guidance tied to selected resource version; RTX30/40/50 CUDA preference, TRT RTX20/GTX16 and GTX10 rejection are source-version guidance, not permanent universal hardware law. Decide default B11-11750M adoption or non-adoption with named version/reason; active custom selection stays untouched.

<a id="r12-gate-b13"></a>
### B13 来源问题（非新任务）

- Owner：R12 / 启动、默认引擎、预加载与评估让路 owner（T02-H17）；阶段细化负责人具名落实实际执行人。
- 待决定/取得证据：Retain which non-equivalent user behavior for Last-primary-engine startup policy, or where is the item-specific approval for each non-equivalent disposition?
- 受阻行为：Retained last-primary-engine startup policy successor
- 所需结果：具来源/实际能力证据的可执行合同；涉及非等价处置保留逐项批准。
- 原事实输入与边界全文：[来源](MIGRATION_CONTRACTS.md#b13)；以下保留该事实/合同输入，不表示已经选定新答案：

> Java autoload-last remembers the last primary on normal shutdown; missing last-engine=-1 and corrupt index must not choose another profile. Next selected_profile_id is editing identity, not startup identity; existing zero-or-one Autoload Default remains accepted. Inspect original corresponding disposition ticket and approval source already named in frozen Inventory/audits, not new Java investigation. Missing approval is not permission to delete. Decide Startup mode/default, stable last-run profile identity, normal-shutdown/crash persistence, deleted/corrupt identity and failure no-fallback.

<a id="r12-gate-b15"></a>
### B15 来源问题（非新任务）

- Owner：R12 / 启动、默认引擎、预加载与评估让路 owner（T02-H18）；阶段细化负责人具名落实实际执行人。
- 待决定/取得证据：Retain which non-equivalent user behavior for Background extra-engine preload, or where is the item-specific approval for each non-equivalent disposition?
- 受阻行为：Retained background extra-engine preload successor
- 所需结果：具来源/实际能力证据的可执行合同；涉及非等价处置保留逐项批准。
- 原事实输入与边界全文：[来源](MIGRATION_CONTRACTS.md#b15)；以下保留该事实/合同输入，不表示已经选定新答案：

> Java saved non-primary preload defaults false and is not foreground promotion. Next processes currently exist only under Switch/Match Reservation. Preserve preload user goal without approving hidden parallel lifecycle by assumption. Inspect original corresponding disposition ticket and approval source already named in frozen Inventory/audits, not new Java investigation. Missing approval is not permission to delete. Decide Opt-in entry/default/save, background readiness display, single-manager reservation ownership, cancellation/resource failure and foreground priority.

<a id="r12-gate-b42"></a>
### B42 来源问题（非新任务）

- Owner：R12 / 手动运行线程来源、临时覆盖与实际值 owner（T04-THREAD-CONTROL, T05-RUNTIME-THREADS）；阶段细化负责人具名落实实际执行人。
- 待决定/取得证据：Which real dynamic capability and manual source/effective/temporary precedence, input domain and expiry/reset semantics are admitted?
- 受阻行为：Manual dynamic thread Apply and confirmed readback
- 所需结果：具来源/实际能力证据的可执行合同；涉及非等价处置保留逐项批准。
- 原事实输入与边界全文：[来源](MIGRATION_CONTRACTS.md#b42)；以下保留该事实/合同输入，不表示已经选定新答案：

> Inputs distinguish saved config, launch policy/explicit override, current Run actual value, temporary requested value and user unsubmitted edits. Decide manual dynamic capability/version/valid input domain from actual protocol evidence, not Java1..1024. Freeze mouse/keyboard explicit Apply, confirm-readback failure/timeout, pending versus effective, expiry on Restart/Switch/disconnect, reset-to-config and no automatic reapply on reconnect. Initialization cannot kata-set-param legacy numSearchThreads over resolved launch override. Dynamic manual path must be executable without runner/policy recommendation. Consuming recommendation is separate B41 integration; H-STATIC proves only explicit Restart argv2.

<a id="r12-gate-b47"></a>
### B47 来源问题（非新任务）

- Owner：R12 / 启动、默认引擎、预加载与评估让路 owner（T04-STARTUP-PERFORMANCE）；阶段细化负责人具名落实实际执行人。
- 待决定/取得证据：Which startup performance evaluation/default/yield policy is retained or item-specifically approved for replacement?
- 受阻行为：Approved startup evaluation and non-disruptive yield
- 所需结果：具来源/实际能力证据的可执行合同；涉及非等价处置保留逐项批准。
- 原事实输入与边界全文：[来源](MIGRATION_CONTRACTS.md#b47)；以下保留该事实/合同输入，不表示已经选定新答案：

> Frozen PR500/504 and preload H18 have retained performance user goal; single-lifecycle constraint is not approval to delete it. Decide explicit entry or admitted startup trigger/default/save behavior, engine selection, user analysis/Pause priority and yield versus true failure feedback. Cite approved individual disposition or retained successor. No actual benchmark required in decision and no Java independent process scheduler migration. This does not block manual Benchmark/Threads or existing current-analysis handoff investigations.

<a id="r12-gate-b67"></a>
### B67 来源问题（非新任务）

- Owner：R12 / PDA/WRN 成对只读回读 owner（T05-PARAMETER-READBACK）；阶段细化负责人具名落实实际执行人。
- 待决定/取得证据：Which named actual local-engine binary/version supports both PDA/WRN readback commands, as proved by B67 minimal real protocol evidence for current reader/Run/round/numbered responses?
- 受阻行为：Paired PDA/WRN current-Run readback
- 所需结果：具来源/实际能力证据的可执行合同；涉及非等价处置保留逐项批准。
- 原事实输入与边界全文：[来源](MIGRATION_CONTRACTS.md#b67)；以下保留该事实/合同输入，不表示已经选定新答案：

> Bind PDA and WRN readback to current reader/Run/request round/numbered responses; both valid finite values are required for current pair success. Identify actual compatible engine binary/version, command/value domains and round retirement on cancel/reconnect/new Run from minimal real protocol evidence, not static argv or chart encoding. Default display is unknown until a valid current pair; invalidation preserves the last valid values with visible failed/unknown state. Current response must not overwrite unsent edits. Preserve existing editing behavior; this ticket neither pairs writes nor adds mutation/retry/reapply semantics.

<a id="r12-gate-b04"></a>
### B04 来源问题（非新任务）

- Owner：R12 / 合格资源、受管维护与显式修复 owner（T03-RESOURCE-TRT-REPAIR-INVESTIGATION）；阶段细化负责人具名落实实际执行人。
- 待决定/取得证据：Admitted TRT target repair ownership remains supported/external/needs-decision until investigation result.
- 受阻行为：Reachable acceleration setup actions and status layout; Approved explicit target-directed TensorRT resource repair
- 所需结果：具来源/实际能力证据的可执行合同；涉及非等价处置保留逐项批准。
- 原事实输入与边界全文：[来源](MIGRATION_CONTRACTS.md#b04)；以下保留该事实/合同输入，不表示已经选定新答案：

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
- 本计划不记录产品/native/真实服务/installed trust 验收通过；实际证据由对应能力与阶段验收负责人取得。

## Baseline 与阶段启动细化规则

- 仓库 `qiyi71w/lizzieyzy-next-tauri`；历史调查 `55795fab49b80a681a9fa544950e18f5f2da4cc6`；历史来源细化快照 `15b71e8bd89349961b7cf25f2fd2a6928e991e40`（不是未来实施起点）。Java v1 `7b4027531c2b26062d0bfc27a040cc550cfbea4d`、冻结审计端点 `af0e07a7386483f3bfc8a15780de72ffc2f0de4c` 不变。
- 本文件是阶段计划，不是未来已发布任务组；历史草案仅详细来源合同，不构成第二活跃 tracker，也不把来源意图数当任务数。未决问题无需本轮全部决定，但受阻行为不能遗漏。
- 阶段启动前由本阶段负责人消费 [Architecture](ARCHITECTURE_NEXT.md)、[Migration Plan](MIGRATION_PLAN.md)、[公开路由](MIGRATION_ROUTES.json)、本阶段计划及实际 scoped validated prerequisite 的最终完成记录；无前置按完成记录冻结精确实施提交，不自行替换 latest main；有前置读取实际完成/集成 full SHA 和条件，用已验证包含所需结果的提交。多个未合并成果由具名串行集成 owner 在隔离树合成并验证后冻结起点，不能编造未来 SHA；相同 HEAD 不证明证据实体已经可取得。
- 将普通技术调查/字段映射/实现步骤收进完整垂直能力；只有真实材料性产品/服务/引擎准入问题保留具名 owner 与门。按阶段滚动细化完整垂直能力、fresh reader 可执行性冷读、呈现实施清单并另获发布批准；本阶段结构批准不授权产品实施或未来全部票发布。
- Rust current-game/session 权威、单 engine lifecycle、无隐式 Switch/Restart/fallback、Cancel/失败不破坏当前谱、统一网络和凭据保护及五 ADR 保持。新 UI 消费 [R11 首 search UI 本地化 key/locale/fallback 结果](R11_PLAN.md#behavior-a01)；各设置 owner 同步 Java 字段语义 map，R17 集合闭环，不等待整 R11 或完整翻译。
- 继承证据必须保留原 candidate full SHA、behavior/platform/engine/helper/service/resource/version 与未验条件。仅文档归类不重验产品；相关 behavior/DTO/protocol/Run/job/document/account/resource/platform/service 改变使受影响证据失效，由实际消费者取得 focused real revalidation。仓库/fixture/browser 与原生/真实服务不同；应用运行、发行打包、上游引擎编译三者独立。没有实际执行的门保持 Not run/needs-info。

## 覆盖与来源导航

[公开路由](MIGRATION_ROUTES.json) 记录唯一阶段归属；[来源意图](MIGRATION_SOURCE_MAP.md) 保留来源含义；[来源行为合同](MIGRATION_CONTRACTS.md) 提供本阶段所引用的完整静态合同。R12–R18 来源集包含 124 owner intents、163 source intents、149 历史合同和 44 具名未来门；数量用于来源覆盖，不是实施任务配额。历史 50 needs-info 的实际待决/普通技术步骤按本阶段正文处理；具名未来门保留原 ID，未证明的能力不自动完成。
