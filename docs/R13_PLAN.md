# R13 — 分析任务与展示

Status: 滚动阶段计划；2026-10-08 已按用户批准将显式 KataGo GTP 主分析纳入与 R12 的联合安排，其他本阶段决定仍按具名门滚动细化；产品能力验收尚未完成。

## 目标、完整范围与非目标

从多文件/自动快析/tracking到重点分析、颗粒化显示与性能工作流形成可独立验证的用户分析能力。

非目标：不替R12 runtime管理、R14readboard扩展或R15Match落子；不扩写ANA-06/16 Accepted显式任务为自动任务。

本阶段包括以下具名能力章节及其引用的所有 source/Delta/用户断言、默认与保存、错误与取消、原生或实际服务义务。详细契约通过权威历史全文稳定引用保留，不以摘要替换契约；本文件与公开路由的 scoped-result 规则优先于历史来源排期。

<a id="retained-analysis"></a>
## 缓存、闪电及自动整局保留能力

保留用户选择in-tree reuse、lightning/part/all-branches与automatic-current controls完整目标；现Accepted显式task不能代替未准入的自动触发。每项历史非等价保留具名决定，无批准不删除。

唯一能力责任：`T02-H19`, `T02-H20`, `T02-H21`。本节设置/技术调查与后续实现是同一完整能力；跨节消费不改变来源owner。

详细来源全文（归档合同，不是活跃任务）：

- [B17 — Decide user-controlled in-tree result reuse preservation](MIGRATION_CONTRACTS.md#b17)：Close the named historical non-equivalence with a retained executable behavior contract or item-specific approved disposition.
- [B18 — Retained user-controlled in-tree result reuse successor](MIGRATION_CONTRACTS.md#b18)：Deliver only the approved retained user behavior or evidence-backed no-change closure produced by the preservation decision.
- [B19 — Decide lightning, part and all-branches retained flows preservation](MIGRATION_CONTRACTS.md#b19)：Close the named historical non-equivalence with a retained executable behavior contract or item-specific approved disposition.
- [B20 — Retained lightning, part and all-branches retained flows successor](MIGRATION_CONTRACTS.md#b20)：Deliver only the approved retained user behavior or evidence-backed no-change closure produced by the preservation decision.
- [B21 — Decide automatic current-game retained controls preservation](MIGRATION_CONTRACTS.md#b21)：Close the named historical non-equivalence with a retained executable behavior contract or item-specific approved disposition.
- [B22 — Retained automatic current-game retained controls successor](MIGRATION_CONTRACTS.md#b22)：Deliver only the approved retained user behavior or evidence-backed no-change closure produced by the preservation decision.

最小验收面与用户断言：上述全文的 Acceptance/Concrete acceptance 和 Frozen source/Canonical Delta 条款全部保留；本节 owner 用实际 changed domain/owner/protocol/persistence/UI 面证明成功、失败/Cancel、身份失效、适用 defaults/save/reopen；只读调查以有来源的可判定结论结束，不能宣称功能已完成。每个受影响设置 map 归该消费者。

<a id="tracking"></a>
## Tracking 区间、目标与清除

interval/job历史目标、指定tracking点、target修改和clear-to-ordinary含单Run ownership/取消/恢复完整合同；不以same-tree focus、hover cache eviction或readboard自动替代。

唯一能力责任：`T02-H24`, `T02-TRACKING`。本节设置/技术调查与后续实现是同一完整能力；跨节消费不改变来源owner。

详细来源全文（归档合同，不是活跃任务）：

- [B23 — Decide tracking job historical preservation preservation](MIGRATION_CONTRACTS.md#b23)：Close the named historical non-equivalence with a retained executable behavior contract or item-specific approved disposition.
- [B24 — Freeze tracking Run/job ownership and interval compatibility](MIGRATION_CONTRACTS.md#b24)：Choose one tracking Run/job concurrency contract and preserve retained interval/visits/clear goals without duplicating same-tree focus.
- [B25 — Tracking points and clear-to-ordinary analysis](MIGRATION_CONTRACTS.md#b25)：Deliver the retained tracking workflow under the approved one-Run contract.

最小验收面与用户断言：上述全文的 Acceptance/Concrete acceptance 和 Frozen source/Canonical Delta 条款全部保留；本节 owner 用实际 changed domain/owner/protocol/persistence/UI 面证明成功、失败/Cancel、身份失效、适用 defaults/save/reopen；只读调查以有来源的可判定结论结束，不能宣称功能已完成。每个受影响设置 map 归该消费者。

<a id="batch"></a>
## 多文件分析队列与安全输出

session-only有序多SGF intake、各file admission/state/conditions/budget/停止当前或全部/队列策略与显式安全输出。cancel/失败保留已完成文件与current-game，不替换/dirty当前谱；不恢复旧batch/进程。

唯一能力责任：`T02-ANA-07`。本节设置/技术调查与后续实现是同一完整能力；跨节消费不改变来源owner。

详细来源全文（归档合同，不是活跃任务）：

- [B26 — Freeze batch intake, analysis conditions and safe file output](MIGRATION_CONTRACTS.md#b26)：Freeze missing batch settings/output choices without changing the session-only ordered queue requirement.
- [B27 — Session-only ordered SGF batch analysis](MIGRATION_CONTRACTS.md#b27)：Implement explicit batch/batch-deep intake and safe per-file execution without replacing the current game.

最小验收面与用户断言：上述全文的 Acceptance/Concrete acceptance 和 Frozen source/Canonical Delta 条款全部保留；本节 owner 用实际 changed domain/owner/protocol/persistence/UI 面证明成功、失败/Cancel、身份失效、适用 defaults/save/reopen；只读调查以有来源的可判定结论结束，不能宣称功能已完成。每个受影响设置 map 归该消费者。

<a id="adapters"></a>
## 具名丰富分析适配与缺字段显示

富分析以具名engine/version/capability及真实协议样本准入；缺字段显示unavailable不能fake0。generic GTP或已accepted基本分析不证明丰富能力。

2026-10-08 用户确认 R12 Q6，并要求主分析的 GTP 接入与本阶段共同安排。保留现有 JSONL，新增可显式选择的 KataGo GTP 主分析；不是只准入 move-only 或参数控件。交付与验收见[联合合同](#gtp-main-analysis)。本节 owner 不接管 R12 的进程、规则、线程和回读权威。

唯一能力责任：`T02-ADAPTER-EVIDENCE`, `T02-ANA-09`。本节设置/技术调查与后续实现是同一完整能力；跨节消费不改变来源owner。

详细来源全文（归档合同，不是活跃任务）：

- [B28 — Admit named engine/version rich-analysis adapters](MIGRATION_CONTRACTS.md#b28)：Produce named product engine/version admission and controlled protocol fixtures before an adapter feature is executable.
- [B29 — Named rich-analysis adapters and absent-field handling](MIGRATION_CONTRACTS.md#b29)：Implement precisely the engine/version capabilities admitted by B27 through actual Next analysis owners.

最小验收面与用户断言：上述全文的 Acceptance/Concrete acceptance 和 Frozen source/Canonical Delta 条款全部保留；本节 owner 用实际 changed domain/owner/protocol/persistence/UI 面证明成功、失败/Cancel、身份失效、适用 defaults/save/reopen；只读调查以有来源的可判定结论结束，不能宣称功能已完成。每个受影响设置 map 归该消费者。

<a id="autoquick"></a>
## 自动载入快析与任务交还

成功load trigger、default/save和一次性task admission/owner/Run handoff；既有explicit current-game quick/all/swing不证明auto-on-load。包含whole-game terminal→ordinary、当前response framing/pause、close/reopen/Cancel/Pause恢复调查；身份/document/Run/rules变化不能由旧Ready/stop/curve/cache解锁。R14自动remote leg必需消费本节已验证admission/owner/handoff结果，R12本地显式Restart独立。

唯一能力责任：`T02-AUTOLOAD-QUICK`, `T03-ANA16-WHOLE-GAME-LIMITS`, `T03-AUTOLOAD-RESUME-FRAMING-INVESTIGATION`, `T04-HANDOFF`。本节设置/技术调查与后续实现是同一完整能力；跨节消费不改变来源owner。

详细来源全文（归档合同，不是活跃任务）：

- [B30 — Freeze automatic-on-load quick-analysis admission and saved defaults](MIGRATION_CONTRACTS.md#b30)：Freeze actual load trigger/default/settings/ownership for automatic quick analysis, not just an explicit ANA16 task.
- [B31 — Automatic load quick analysis and identity-safe handback](MIGRATION_CONTRACTS.md#b31)：Deliver successful-load automatic quick analysis with safe pause, cancellation, visible mode and user-position preservation.
- [B53 — Bound whole-game budget terminal-to-ordinary ownership](MIGRATION_CONTRACTS.md#b53)：Answer one bounded applicability/correctness question and stop with an evidence-backed disposition, not a speculative implementation.
- [B57 — Bound existing response framing and handback pause ownership](MIGRATION_CONTRACTS.md#b57)：Answer one bounded applicability/correctness question and stop with an evidence-backed disposition, not a speculative implementation.
- [B62 — Bound current-game task completion/Cancel/Pause and close/reopen handback](MIGRATION_CONTRACTS.md#b62)：Answer one bounded applicability/correctness question and stop with an evidence-backed disposition, not a speculative implementation.

最小验收面与用户断言：上述全文的 Acceptance/Concrete acceptance 和 Frozen source/Canonical Delta 条款全部保留；本节 owner 用实际 changed domain/owner/protocol/persistence/UI 面证明成功、失败/Cancel、身份失效、适用 defaults/save/reopen；只读调查以有来源的可判定结论结束，不能宣称功能已完成。每个受影响设置 map 归该消费者。

<a id="display"></a>
## 颗粒化显示、悬停、自定义评级与可访问候选

颗粒candidate/territory开关、B/W筛选、PV visits/branch/text/order与10秒candidate delay默认关，以及retained hover delay/manual reveal、custom grade threshold/equality/missing analysis。保护ADR单series、主board heat与personal C分离。live candidate键盘导航/updates、tree显示/click-hit身份、bounded preview/replay retirement与初始两步导航差异都按实际来源处理，不伪造全局产品审批。

唯一能力责任：`T02-CUSTOM-GRADE`, `T02-GRANULAR-DISPLAY`, `T02-H16`, `T05-CANDIDATE-LIST-ACCESS`, `T05-PREVIEW-ASYNC-CHECK`, `T05-TREE-PUBLICATION-CHECK`, `T05-VARIATION-NAV-CHECK`。本节设置/技术调查与后续实现是同一完整能力；跨节消费不改变来源owner。

详细来源全文（归档合同，不是活跃任务）：

- [B11 — Decide hover delay and manual reveal preservation](MIGRATION_CONTRACTS.md#b11)：Close the named historical non-equivalence with a retained executable behavior contract or item-specific approved disposition.
- [B12 — Retained hover delay and manual reveal successor](MIGRATION_CONTRACTS.md#b12)：Deliver only the approved retained user behavior or evidence-backed no-change closure produced by the preservation decision.
- [B32 — Map retained granular candidate and territory display fields](MIGRATION_CONTRACTS.md#b32)：Produce per-field retained defaults/ranges/persistence/render consumption rather than restore an undefined preferences bundle.
- [B33 — Granular candidate and territory presentation](MIGRATION_CONTRACTS.md#b33)：Implement all retained granular fields on identity-valid ordinary analysis presentation.
- [B34 — Locate and decide custom review-grade thresholds](MIGRATION_CONTRACTS.md#b34)：Resolve a source-backed custom-grade contract instead of claiming Java already has an unverified implementation.
- [B35 — Custom grade threshold settings and review consumers](MIGRATION_CONTRACTS.md#b35)：Deliver approved custom-grade editing and consistent next-move/chart grading without rewriting fixed-grade historical acceptance.
- [B66 — Verify current review-tree display and click-hit identity](MIGRATION_CONTRACTS.md#b66)：Answer one bounded applicability/correctness question and stop with an evidence-backed disposition, not a speculative implementation.
- [B69 — Verify preview input ownership and decide any navigation difference](MIGRATION_CONTRACTS.md#b69)：Answer one bounded applicability/correctness question and stop with an evidence-backed disposition, not a speculative implementation.
- [B70 — Accessible candidate-list navigation during live updates](MIGRATION_CONTRACTS.md#b70)：Keep candidate rows readable and navigable across streaming updates and supported layout.
- [B71 — Verify bounded preview calculation/publication and replay retirement](MIGRATION_CONTRACTS.md#b71)：Answer one bounded applicability/correctness question and stop with an evidence-backed disposition, not a speculative implementation.

最小验收面与用户断言：上述全文的 Acceptance/Concrete acceptance 和 Frozen source/Canonical Delta 条款全部保留；本节 owner 用实际 changed domain/owner/protocol/persistence/UI 面证明成功、失败/Cancel、身份失效、适用 defaults/save/reopen；只读调查以有来源的可判定结论结束，不能宣称功能已完成。每个受影响设置 map 归该消费者。

<a id="continuation"></a>
## 两类引擎续走与唯一落子权威

leaf-only与every-beat两类Engine Continuation必须分别保留真正创建新棋子及唯一Match/current-game写权威；Review Autoplay遍历和Variation Replay预览不替代。外部适用性由本节owner决定，R14仅消费并执行其实际外部权威边界。

唯一能力责任：`T02-CONTINUATION`。本节设置/技术调查与后续实现是同一完整能力；跨节消费不改变来源owner。

详细来源全文（归档合同，不是活跃任务）：

- [B36 — Decide leaf-only and every-beat Engine Continuation authority](MIGRATION_CONTRACTS.md#b36)：Deliver approved user semantics and writing authority for both Engine Continuation goals.
- [B37 — Approved Engine Continuation move creation](MIGRATION_CONTRACTS.md#b37)：Implement both retained continuation modes only after approved writing/Run authority.

最小验收面与用户断言：上述全文的 Acceptance/Concrete acceptance 和 Frozen source/Canonical Delta 条款全部保留；本节 owner 用实际 changed domain/owner/protocol/persistence/UI 面证明成功、失败/Cancel、身份失效、适用 defaults/save/reopen；只读调查以有来源的可判定结论结束，不能宣称功能已完成。每个受影响设置 map 归该消费者。

<a id="performance"></a>
## 独立 Benchmark、保存建议与实测调优

isolated custom Benchmark直接从既有saved profile identity与合格资源运行；固定target/revision，NN/search speed区分，可访问反馈区分measurement与recommendation。saved CFG/BENCHMARK policy单向消费runner结果；manual dynamic线程不是runner前置。measured Import→Review→Apply/Restore保留所有fingerprint/scene overlay/nonmutation/busy/stale与固定v1 measurement 3/5pairs、speed/CV门，不自动apply，技术overlay/schema设计不是审批。

唯一能力责任：`T03-PERFORMANCE-A11Y-BENCHMARK`, `T03-PERFORMANCE-CUSTOM-BENCHMARK-RUNNER`, `T03-PERFORMANCE-SPEED-METRICS`, `T03-THREAD-SAVED-ENTRY-POLICY`, `T04-MEASURED-TUNING`。本节设置/技术调查与后续实现是同一完整能力；跨节消费不改变来源owner。

详细来源全文（归档合同，不是活跃任务）：

- [B38 — Isolated saved-profile custom benchmark runner](MIGRATION_CONTRACTS.md#b38)：Run explicit local custom benchmarks for a frozen existing saved-profile target without changing tuning policy.
- [B39 — Distinct NN and search-speed benchmark results](MIGRATION_CONTRACTS.md#b39)：Present two truthful benchmark metrics for the same frozen saved target.
- [B40 — Accessible benchmark outcomes and recommendation distinction](MIGRATION_CONTRACTS.md#b40)：Integrate truthful benchmark/recommendation feedback into actual keyboard and assistive-technology workflow.
- [B41 — Saved-entry CFG/BENCHMARK recommendation policy and launch integration](MIGRATION_CONTRACTS.md#b41)：Persist and consume per-entry measured thread recommendations without global contamination or rewriting user commands.
- [B45 — Design compatible measured-scene overlays and report admission](MIGRATION_CONTRACTS.md#b45)：Translate the fixed measured-report schema to admitted Next scene/settings/one-Run boundaries before feature implementation.
- [B46 — Measured report Import, Review, Apply and Restore](MIGRATION_CONTRACTS.md#b46)：Implement complete measured-report review and scene-specific explicit launch overlay workflow, not merely thread argv.

最小验收面与用户断言：上述全文的 Acceptance/Concrete acceptance 和 Frozen source/Canonical Delta 条款全部保留；本节 owner 用实际 changed domain/owner/protocol/persistence/UI 面证明成功、失败/Cancel、身份失效、适用 defaults/save/reopen；只读调查以有来源的可判定结论结束，不能宣称功能已完成。每个受影响设置 map 归该消费者。

<a id="focus"></a>
## 同树重点分析与精确局面确认

local same-tree focus包含054a/b/c/e/f：compatible binary真实probe、普通ownership、exact-root whole-slot adoption、user-intent/active set、strict8s progress、renderer/settings。局面target/reader/slot以及SGF RU/KM confirmation是功能内可达调查；本地验收不等待remote/readboard。054d仅R14扩展。

唯一能力责任：`T03-ANALYSIS-CONFIRMED-POSITION-INVESTIGATION`, `T03-ANALYSIS-SAME-TREE-MOVE-FOCUS`, `T03-ANALYSIS-SGF-RULES-SYNC-INVESTIGATION`。本节设置/技术调查与后续实现是同一完整能力；跨节消费不改变来源owner。

详细来源全文（归档合同，不是活跃任务）：

- [B49 — Local same-tree move focus, ordinary cache and display](MIGRATION_CONTRACTS.md#b49)：Deliver the complete local same-tree focus workflow and compatible-engine evidence without waiting for C readboard integration.
- [B55 — Bound ordinary analysis target/reader/slot confirmation](MIGRATION_CONTRACTS.md#b55)：Answer one bounded applicability/correctness question and stop with an evidence-backed disposition, not a speculative implementation.
- [B56 — Bound SGF RU/KM import-to-analysis confirmation](MIGRATION_CONTRACTS.md#b56)：Investigate current SGF RU/KM import→dispatch→result confirmation using frozen UD-03-029 rules semantics and B55 applicable target result; changed B50 paths receive future revalidation.

最小验收面与用户断言：上述全文的 Acceptance/Concrete acceptance 和 Frozen source/Canonical Delta 条款全部保留；本节 owner 用实际 changed domain/owner/protocol/persistence/UI 面证明成功、失败/Cancel、身份失效、适用 defaults/save/reopen；只读调查以有来源的可判定结论结束，不能宣称功能已完成。每个受影响设置 map 归该消费者。

<a id="gtp-main-analysis"></a>
## R12/R13 联合合同：KataGo GTP 主分析

本节继承 [R12 本轮基线与批准记录](R12_PLAN.md#refinement-20261008)：仓库 `qiyi71w/lizzieyzy-next-tauri`，调查基线 `9d2ccf3ba6881c755013b82948e5da1a01f7fcd4`。此处仅细化被明确要求的 GTP 主分析协调，不替其他 R13 产品决定作答，不提前发布实施票。

| 责任与交付 | 所需结果及消费边界 |
| --- | --- |
| R12 资源与运行管理原 owner | 合格 binary/model/config/profile、单 manager 下的显式 Start/Switch/Restart、同 Run 的 rules/position 确认、reader/command retirement、取消/失败隔离与诊断。GTP 基础运行资格不以 R13 主分析完成为前置。 |
| R13 `T02-ADAPTER-EVIDENCE`（B28） | 具名 KataGo binary/version 的真实分析流与控制响应样本；逐项声明 candidates、winrate、PV、ownership、streaming、selected-node、whole-game 能力及 absent/error/terminal/cancel 行为。运行版本尚未获证时保持资格门，不从旧 CPU 参数探测推定主分析资格。 |
| R13 `T02-ANA-09`（B29） | 从真实 GTP 分析流到既有分析模型、DTO/API 和主界面的完整接入；通过 R12 的同一 Run/reader 调度与身份边界。交付 `katago-gtp-main-analysis`，保留 JSONL 当前行为，不建立第二套 current-game/分析发布权威。 |
| R12 线程/回读 owner 与集成验收 owner | 线程 Apply＋确认、PDA/WRN pair 绑定上述同一 GTP 主分析 Run；消费 R13 scoped result，在集成候选完成运行控制与分析共同工作的验收。R13 分析 owner 负责字段和展示断言，R12 集成 owner 负责最终组合出口。 |

### 顺序与阻塞边界

1. B28 协议/能力取证与 R12 资源/参数取证可分别推进；共享 profile/Run/reader 合同先对齐，由具名串行集成负责人落地，不能同时建立互不兼容的 reader 或命令调度权威。
2. R12 提供已验证的对应本地资源、普通规则/局面与 Run 边界；R13 B29 消费后完成可用主分析。B29 的基础分析成功不依赖 R12 参数面板先完成，参数局部验证也不等待全部分析 UI。
3. R12 集成消费 `katago-gtp-main-analysis` 后验证组合场景。反向消费仅是组合验收依赖，不把 R12 全阶段完成设为 R13 开工门；R13 的 batch、tracking、自动快析、focus、benchmark 与建议保存不因此成为主分析的整体前置。
4. 切换模式由用户明确选择与启动；既有 JSONL profile 不自动转换，配置浏览/保存不触发切换。GTP 必需能力未通过时可见拒绝，不退回 JSONL 或另启隐藏 companion engine 冒充成功。

### 交付证据与验收

- 在真实具名 KataGo GTP Run 对当前所选局面持续分析，候选、胜率和 PV 来自有效当前结果；ownership 等其他字段按逐项准入映射，缺失显示 unavailable 而非零。与现有主分析逐项比较：局面/颜色/规则/KM、流式更新、Pause/Continue、节点和分支切换、已有结果保留；必需能力缺失不能用“可选字段”规避，协议差异须明确说明并按原门处理。
- 同一 reader 上分析流与编号线程/参数响应交错时，控制结果不能被当作分析，分析输出不能满足控制 ACK。线程确认/pair 按 Run/round 失效；用户 Pause、导航、Restart、Switch 和失败恢复后的旧输出不能回写新局面或恢复旧请求。
- 在实际 UI 验证 JSONL→GTP→JSONL 的显式切换及失败保留健康 A；同一个 GTP 主分析进程完成线程 Apply/确认和 PDA/WRN 成对回读。只读 pair 不扩成配对写入，线程临时值不自动跨 Run reapply。
- scoped result 记录完整应用提交、binary/model/config/resource 身份、平台和具体能力；仓库受控协议、真实引擎、原生 UI 分栏。R12 与 R13 消费同一集成候选的证据或明确 unchanged-boundary 继承，两个分别通过的孤立候选不能充当组合通过。
- `katago-gtp-main-analysis` 未获证时，R12 其他能力可推进，但其主分析联合要求及完整完成声明保持未完成；不改变 R13 其他来源 owner/合同、既有 Accepted 范围或未验资格状态。

## 真实 scoped prerequisites 与跨阶段消费

阶段编号是 Delivery Order，不是 whole-stage hard dependency。原来源中已经 Accepted 的所需功能直接按原 evidence tuple 消费；ledger-only 汇总不是等待门。未来 source 的 required/conditional/field scope 在 [公开路由](MIGRATION_ROUTES.json) 和 [来源合同](MIGRATION_CONTRACTS.md) 中保留，不能扁平化成全票或全阶段开工门。

- 消费 `qualified-local-resource`（R12 / `T01-RESOURCE`）：Actual local analysis/benchmark resource only。前置仅该已验证结果及适用条件。
- 本阶段唯一交付 `isolated-benchmark-result`（`T03-PERFORMANCE-CUSTOM-BENCHMARK-RUNNER`）：Passed frozen target/profile/revision custom-runner results。其他消费者只取其通过scope；不接管本owner。
- 消费 `isolated-benchmark-result`（R13 / `T03-PERFORMANCE-CUSTOM-BENCHMARK-RUNNER`）：Saved recommendation policy and measured tuning one-way only。前置仅该已验证结果及适用条件。
- 本阶段唯一交付 `automatic-load-quick-handoff`（`T02-AUTOLOAD-QUICK`）：Passed automatic-task admission/owner/handoff for successful-load consumer。其他消费者只取其通过scope；不接管本owner。
- 本阶段唯一交付 `local-focus-result`（`T03-ANALYSIS-SAME-TREE-MOVE-FOCUS`）：Passed 054a-c local ordinary/focus/root-slot identity and compatible binary result。其他消费者只取其通过scope；不接管本owner。
- 消费 `first-ui-locale-foundation`（[R11](R11_PLAN.md#behavior-a01) / `T02-I18N-FOUNDATION`）：Actual new UI integration only; settings owner adds Java field map。前置仅该已验证结果及适用条件。
- 消费 `ordinary-rules-snapshot`（R12 / `T03-MATCH-RULES-LIFECYCLE`）：仅同一 KataGo GTP 主分析 Run 的普通规则确认，不等待 Match 存储或整 R12。
- 本阶段唯一交付 `katago-gtp-main-analysis`（`T02-ANA-09`）：[显式 GTP 主分析的完整 scoped result](#gtp-main-analysis)；R12 只消费用于同实例运行控制与分析组合验收，不迁移本阶段来源归属。

## 具名待决事项、事实输入与受阻行为

以下 owner 是阶段内具名能力责任角色，实施细化时将阶段责任落到实际执行人；不是要求本轮解决所有未来选择。来源/协议/能力未证明保持 needs-info。普通工程设计不造审批；永久非等价才要逐项产品批准。

<a id="r13-gate-b11"></a>
### B11 来源问题（非新任务）

- Owner：R13 / 颗粒化显示、悬停、自定义评级与可访问候选 owner（T02-H16）；阶段细化负责人具名落实实际执行人。
- 待决定/取得证据：Retain which non-equivalent user behavior for Hover delay and manual reveal, or where is the item-specific approval for each non-equivalent disposition?
- 受阻行为：Retained hover delay and manual reveal successor
- 所需结果：具来源/实际能力证据的可执行合同；涉及非等价处置保留逐项批准。
- 原事实输入与边界全文：[来源](MIGRATION_CONTRACTS.md#b11)；以下保留该事实/合同输入，不表示已经选定新答案：

> Java 200ms plus delay dialog/manual reveal differs from accepted Next 120ms hover. Candidate ten-second delay is unrelated. Current 120ms and non-navigating preview remain unchanged until item-specific decision. Inspect original corresponding disposition ticket and approval source already named in frozen Inventory/audits, not new Java investigation. Missing approval is not permission to delete. Decide Hover entry, adjustable/default delay, manual-reveal behavior, persistence, focus and cancellation.

<a id="r13-gate-b17"></a>
### B17 来源问题（非新任务）

- Owner：R13 / 缓存、闪电及自动整局保留能力 owner（T02-H19）；阶段细化负责人具名落实实际执行人。
- 待决定/取得证据：Retain which non-equivalent user behavior for User-controlled in-tree result reuse, or where is the item-specific approval for each non-equivalent disposition?
- 受阻行为：Retained user-controlled in-tree result reuse successor
- 所需结果：具来源/实际能力证据的可执行合同；涉及非等价处置保留逐项批准。
- 原事实输入与边界全文：[来源](MIGRATION_CONTRACTS.md#b17)；以下保留该事实/合同输入，不表示已经选定新答案：

> Java enable-lizzie-cache defaults true and controls computed node reuse, not Next SQLite persistence. Existing SQLite remains accepted and cannot be cited as equivalent toggle without evidence. Inspect original corresponding disposition ticket and approval source already named in frozen Inventory/audits, not new Java investigation. Missing approval is not permission to delete. Decide Cache-toggle entry/default/save, result freshness/reuse/disable semantics and distinction from SGF exchange/durable SQLite.

<a id="r13-gate-b19"></a>
### B19 来源问题（非新任务）

- Owner：R13 / 缓存、闪电及自动整局保留能力 owner（T02-H20）；阶段细化负责人具名落实实际执行人。
- 待决定/取得证据：Retain which non-equivalent user behavior for Lightning, part and all-branches retained flows, or where is the item-specific approval for each non-equivalent disposition?
- 受阻行为：Retained lightning, part and all-branches retained flows successor
- 所需结果：具来源/实际能力证据的可执行合同；涉及非等价处置保留逐项批准。
- 原事实输入与边界全文：[来源](MIGRATION_CONTRACTS.md#b19)；以下保留该事实/合同输入，不表示已经选定新答案：

> Explicit accepted ANA-16 tasks do not prove automatic-on-load; automatic quick analysis remains separately owned T02-AUTOLOAD-QUICK. Java whole/part/all-branches, settings, Ctrl+B and right-click-this-move goals must be compared individually. Inspect original corresponding disposition ticket and approval source already named in frozen Inventory/audits, not new Java investigation. Missing approval is not permission to delete. Decide Each explicit flash entry/scope/default/budget/save/cancel and branch identity; automatic-on-load remainder routed to B31.

<a id="r13-gate-b21"></a>
### B21 来源问题（非新任务）

- Owner：R13 / 缓存、闪电及自动整局保留能力 owner（T02-H21）；阶段细化负责人具名落实实际执行人。
- 待决定/取得证据：Retain which non-equivalent user behavior for Automatic current-game retained controls, or where is the item-specific approval for each non-equivalent disposition?
- 受阻行为：Retained automatic current-game retained controls successor
- 所需结果：具来源/实际能力证据的可执行合同；涉及非等价处置保留逐项批准。
- 原事实输入与边界全文：[来源](MIGRATION_CONTRACTS.md#b21)；以下保留该事实/合同输入，不表示已经选定新答案：

> Accepted ANA-16 already covers interval/color/budget/Pause/Continue/two-stage tasks. That is not proof of all former menu/table/start/stop/default controls; educational pause-exit guidance stays E GUIDE-01. Inspect original corresponding disposition ticket and approval source already named in frozen Inventory/audits, not new Java investigation. Missing approval is not permission to delete. Decide Entry/default/save compatibility, current-game range/color/budgets and pause-exit task behavior; guidance remains independent.

<a id="r13-gate-b23"></a>
### B23 来源问题（非新任务）

- Owner：R13 / Tracking 区间、目标与清除 owner（T02-H24）；阶段细化负责人具名落实实际执行人。
- 待决定/取得证据：Retain which non-equivalent user behavior for Tracking job historical preservation, or where is the item-specific approval for each non-equivalent disposition?
- 受阻行为：Freeze tracking Run/job ownership and interval compatibility
- 所需结果：具来源/实际能力证据的可执行合同；涉及非等价处置保留逐项批准。
- 原事实输入与边界全文：[来源](MIGRATION_CONTRACTS.md#b23)；以下保留该事实/合同输入，不表示已经选定新答案：

> Tracking point set/interval/target visits/display identity/clear-to-ordinary goals are not hover eviction or readboard. Java tracking defaults max visits500 and has interval controls; same-tree focus is later evolution and must not silently delete legacy interval goal. Inspect original corresponding disposition ticket and approval source already named in frozen Inventory/audits, not new Java investigation. Missing approval is not permission to delete. Decide Exact retained tracking goals versus old exclusion and later same-tree focus, with any non-equivalent remainder individually approved.

<a id="r13-gate-b24"></a>
### B24 来源问题（非新任务）

- Owner：R13 / Tracking 区间、目标与清除 owner（T02-TRACKING）；阶段细化负责人具名落实实际执行人。
- 待决定/取得证据：How does retained tracking interval/job behavior coexist with ordinary analysis and same-tree focus under one Run owner?
- 受阻行为：Tracking points and clear-to-ordinary analysis
- 所需结果：具来源/实际能力证据的可执行合同；涉及非等价处置保留逐项批准。
- 原事实输入与边界全文：[来源](MIGRATION_CONTRACTS.md#b24)；以下保留该事实/合同输入，不表示已经选定新答案：

> Inputs are original TrackingAnalysisController interval/target-visits immutable display contract, H24 disposition result, ordinary current-engine lanes, and frozen 054 same-tree focus. Decide whether retained tracking is fully served by focus or has a distinct interval user workflow; any non-equivalent cut requires item approval. No second lifecycle or synthetic interval is chosen here. Preserve clear restoring ordinary analysis, no hover/readboard substitution, exact point/current-context/result identity, no auto-resume after retirement.

<a id="r13-gate-b26"></a>
### B26 来源问题（非新任务）

- Owner：R13 / 多文件分析队列与安全输出 owner（T02-ANA-07）；阶段细化负责人具名落实实际执行人。
- 待决定/取得证据：What analysis/output/default/queue-edit and stop-current/all contract is admitted for session-only batch SGF analysis?
- 受阻行为：Session-only ordered SGF batch analysis
- 所需结果：具来源/实际能力证据的可执行合同；涉及非等价处置保留逐项批准。
- 原事实输入与边界全文：[来源](MIGRATION_CONTRACTS.md#b26)；以下保留该事实/合同输入，不表示已经选定新答案：

> Consume ANA02/03 and APP02 multi-file intake plus source batch/batch-deep menus/table/default100 visits and PR529 startup/admission. Decide supported input/output SGF paths, per-file analysis scope/stages/budgets/defaults, stop-current versus stop-all and edit-queue rules, output overwrite/atomic preservation and explicit admission while foreground owner is occupied. No restoring queue/history from restart. Batch work must never replace/dirty current game.

<a id="r13-gate-b28"></a>
### B28 来源问题（非新任务）

- Owner：R13 / 具名丰富分析适配与缺字段显示 owner（T02-ADAPTER-EVIDENCE）；阶段细化负责人具名落实实际执行人。
- 已决定（2026-10-08）：用户要求显式 KataGo GTP 主分析与 R12 共同安排，保留现有 JSONL；[联合合同](#gtp-main-analysis)为当前范围。B28 仍须取得具名版本与逐字段协议证据，产品方向批准不等于 B29 或实际主分析验收通过。
- 待决定/取得证据：Which named engine/version rich-analysis capabilities and protocol samples are actually admitted?
- 受阻行为：Named rich-analysis adapters and absent-field handling
- 所需结果：具来源/实际能力证据的可执行合同；涉及非等价处置保留逐项批准。
- 原事实输入与边界全文：[来源](MIGRATION_CONTRACTS.md#b28)；以下保留该事实/合同输入，不表示已经选定新答案：

> Identify each requested engine/version and exact supported candidates/winrate/PV/ownership/streaming/selected-node/whole-game fields. Record raw protocol samples, valid/absent/error/terminal/cancel and identity behavior. Each capability is independent; missing fields unavailable, not invented. Decide precise admitted set and unsupported outcomes; no empty generic interface or automatic engine build. Future SSH adapters obtain additional compatibility/service evidence under C.

<a id="r13-gate-b30"></a>
### B30 来源问题（非新任务）

- Owner：R13 / 自动载入快析与任务交还 owner（T02-AUTOLOAD-QUICK）；阶段细化负责人具名落实实际执行人。
- 待决定/取得证据：Which successful-load triggers, default/persistence and single-Run handback policy are admitted for automatic quick analysis?
- 受阻行为：Automatic load quick analysis and identity-safe handback
- 所需结果：具来源/实际能力证据的可执行合同；涉及非等价处置保留逐项批准。
- 原事实输入与边界全文：[来源](MIGRATION_CONTRACTS.md#b30)；以下保留该事实/合同输入，不表示已经选定新答案：

> Input source auto-quick-analyze-on-load default true, analysis-max-visits default1, flash controls and current SGF07 transaction/ENG06/ANA16 ownership. Decide Next first-use default/persistence and supported trigger surfaces, late engine-ready admission, task conditions, user browsing/pause/Cancel and safe handback. No hidden start/switch/fallback after failed load or no engine. Retain separate explicit tasks and remote lifecycle extensions; source process topology is not imported.

<a id="r13-gate-b34"></a>
### B34 来源问题（非新任务）

- Owner：R13 / 颗粒化显示、悬停、自定义评级与可访问候选 owner（T02-CUSTOM-GRADE）；阶段细化负责人具名落实实际执行人。
- 待决定/取得证据：What source/product-approved custom threshold semantics are intended, including equality boundaries and missing analysis?
- 受阻行为：Custom grade threshold settings and review consumers
- 所需结果：具来源/实际能力证据的可执行合同；涉及非等价处置保留逐项批准。
- 原事实输入与边界全文：[来源](MIGRATION_CONTRACTS.md#b34)；以下保留该事实/合同输入，不表示已经选定新答案：

> Bounded question: frozen SET-NEXT-MOVE/SET-WINRATE-GRAPH custom grade entry/threshold/default/save/consumer versus existing Next fixed six-grade behavior. Record exact source or source-not-found without deleting spec user goal. Decide approved units/threshold order/equality/boundaries/missing-data/common perspective and atomic persistence. Do not fabricate thresholds or replace accepted fixed grades before approval.

<a id="r13-gate-b36"></a>
### B36 来源问题（非新任务）

- Owner：R13 / 两类引擎续走与唯一落子权威 owner（T02-CONTINUATION）；阶段细化负责人具名落实实际执行人。
- 待决定/取得证据：What approved move-creation semantics and Match/current-game authority apply independently to leaf-only and every-beat continuation?
- 受阻行为：Approved Engine Continuation move creation
- 所需结果：具来源/实际能力证据的可执行合同；涉及非等价处置保留逐项批准。
- 原事实输入与边界全文：[来源](MIGRATION_CONTRACTS.md#b36)；以下保留该事实/合同输入，不表示已经选定新答案：

> Use SGF03ADJAUTOPLAY/Ticket28 and glossary. Name leaf-only and every-beat creation semantics separately. Freeze entries/default/settings, selected-branch behavior, one Run/job scheduling, Match/current-game write authority, stop/cancel/failure/illegal decision/save boundaries and conflicts with review navigation. Engine Continuation creates moves; Review Autoplay traverses existing moves and Variation Replay only previews. Neither substitutes for continuation. No inherited historical exclusion authorizes deletion.

<a id="r13-gate-b69"></a>
### B69 来源问题（非新任务）

- Owner：R13 / 颗粒化显示、悬停、自定义评级与可访问候选 owner（T05-VARIATION-NAV-CHECK）；阶段细化负责人具名落实实际执行人。
- 待决定/取得证据：Any source two-move initial-step interaction difference requires explicit decision; existing preview navigation remains unchanged.
- 受阻行为：仅该来源的未决保留能力/确实观察到的差异；既有行为不重开、不永久删除。
- 所需结果：具来源/实际能力证据的可执行合同；涉及非等价处置保留逐项批准。
- 原事实输入与边界全文：[来源](MIGRATION_CONTRACTS.md#b69)；以下保留该事实/合同输入，不表示已经选定新答案：

> Question: Do first/last/longPV preview inputs retain ownership without changing SGF cursor, and is any Java first-step difference actually wanted?
>
> Preserve existing UI03 120ms and current variation/ReviewAutoplay/VariationReplay distinct input contracts. Freeze PV for navigation; hover replacement retires old input/timers. At first/last move wheel/Page belongs preview, never falls through and moves recorded game. Separately name product decision for any source new two-move initial-step versus existing full-step behavior; current accepted default remains until explicit decision. Stop at actual supported interaction matrix and approved/no-change difference, not automatic new shortcut implementation.
>
> Outcome must be one of precise unchanged/narrow inherited evidence, no corresponding reachable mechanism with source rationale, or named reachable gap with deterministic assertion and separately bounded successor. Do not implement a hypothetical Java defect here. Preserve original Accepted scope/candidate; no long stress run, arbitrary repeat/threshold matrix or architecture-immunity claim. Current supported paths can be assessed independently; a not-yet-built feature becomes that feature’s future evidence gate, not a blocker for current-path investigation.

## 集成、实际验收与环境义务

- 具名 owner：**R13 分析工作流集成/实际验收 owner**；阶段细化时明确实际负责人，拥有本阶段 changed composed workflow 的实际验收，不能交给 read-only Closeout。每个能力owner仍对其局部断言和真实surface负责。
- 环境：具名兼容本地真实引擎版本/协议/资源与原生UI；严格source focus/measurement阈值保持，GPU能力仅按实际准入。
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
