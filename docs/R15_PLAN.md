# R15 — 高级对局

Status: 滚动阶段计划；产品能力验收尚未完成。

## 目标、完整范围与非目标

批量PK、开局库/换色/干预、竞赛clock/认输、HumanSL Coach和规则/komi用户行为完整交付。

非目标：不把Compute Budget代替clock；不强迫Coach等待下载/全部资源/发行；不复制R12规则snapshot owner。

本阶段包括以下具名能力章节及其引用的所有 source/Delta/用户断言、默认与保存、错误与取消、原生或实际服务义务。详细契约通过权威历史全文稳定引用保留，不以摘要替换契约；本文件与公开路由的 scoped-result 规则优先于历史来源排期。

<a id="pk"></a>
## 批量 PK、开局库、换色和手动干预

session-only queue/catalog/color swap/manual intervention与completed SGF durable输出；只有请求batch winrate images才消费R11 T02-EXPORT-03 actual chart-output，不等待board export/整R11。单Match保留rollback/rebuild/reservation，保存/reopen不恢复livePK。

唯一能力责任：`T02-GAME-06`。本节设置/技术调查与后续实现是同一完整能力；跨节消费不改变来源owner。

详细来源全文（归档合同，不是活跃任务）：

- [D01 — Session-only PK batches, opening catalog and intervention](MIGRATION_CONTRACTS.md#d01)：Extend explicit PK start and live PK controls to an ordered session-only batch without weakening single-game Match ownership.

最小验收面与用户断言：上述全文的 Acceptance/Concrete acceptance 和 Frozen source/Canonical Delta 条款全部保留；本节 owner 用实际 changed domain/owner/protocol/persistence/UI 面证明成功、失败/Cancel、身份失效、适用 defaults/save/reopen；只读调查以有来源的可判定结论结束，不能宣称功能已完成。每个受影响设置 map 归该消费者。

<a id="human-clock"></a>
## 保留 Human 控件、竞赛时钟与认输

保留每项legacy Human控件、竞赛clock/auto-resign政策；Compute Budget不是竞赛计时，clock决定只阻塞GAME-07，不阻塞Coach think-time budget。失败/cancel/late回合不破坏现局。

唯一能力责任：`T02-CLOCK-DECISION`, `T02-GAME-07`, `T02-H25`。本节设置/技术调查与后续实现是同一完整能力；跨节消费不改变来源owner。

详细来源全文（归档合同，不是活跃任务）：

- [D02 — Decide retained legacy Human-vs-Engine control goals](MIGRATION_CONTRACTS.md#d02)：Resolve each non-equivalent legacy control separately, without reopening the accepted one-Match-session core.
- [D03 — Implement approved retained Human match controls](MIGRATION_CONTRACTS.md#d03)：Expose the retained Human-vs-Engine control goals approved by D02 through the existing sole Match Session and adapter capability path.
- [D04 — Approve competitive clock and auto-resign policy](MIGRATION_CONTRACTS.md#d04)：Approve a clock/auto-resign policy before GAME-07 implementation, keeping application time distinct from Compute Budget.
- [D05 — Competitive clocks and evidence-based auto-resign](MIGRATION_CONTRACTS.md#d05)：Implement application-owned competitive remaining time and visible clocks exactly under D04's approved policy.

最小验收面与用户断言：上述全文的 Acceptance/Concrete acceptance 和 Frozen source/Canonical Delta 条款全部保留；本节 owner 用实际 changed domain/owner/protocol/persistence/UI 面证明成功、失败/Cancel、身份失效、适用 defaults/save/reopen；只读调查以有来源的可判定结论结束，不能宣称功能已完成。每个受影响设置 map 归该消费者。

<a id="humansl"></a>
## 兼容 HumanSL 资源与完整 Coach

GAME-08兼容实际HumanSL binary/model/profile setup门仍必需；qualified-local可开完整Coach，不等待download、managed catalog或发行。acquisition-backed分支才等真实获取，borrowed remote foreground才等对应R14已准入Run。完整tactical/time/deepen/handback/恢复foreground及预算安全（含final c1ec182d来源）仍需真实支持engine/model证据；与clock独立。

唯一能力责任：`T02-GAME-08`。本节设置/技术调查与后续实现是同一完整能力；跨节消费不改变来源owner。

详细来源全文（归档合同，不是活跃任务）：

- [D06 — Freeze compatible HumanSL profile and Coach setup contract](MIGRATION_CONTRACTS.md#d06)：Name the compatible HumanSL engine binary/model/profile and complete Coach setup contract required by GAME-08.
- [D07 — Acquire qualified HumanSL resources without changing prior set](MIGRATION_CONTRACTS.md#d07)：Provide explicit HumanSL model/resource acquisition for the D06-admitted profile through B's qualified resource and C's network-policy seams.
- [D08 — Independent HumanSL Coach with tactical verification and exact handback](MIGRATION_CONTRACTS.md#d08)：Deliver GAME-08 as an independent HumanSL AI Coach Match, not a hidden option of Human-vs-Engine or competitive-clock GAME-07.

最小验收面与用户断言：上述全文的 Acceptance/Concrete acceptance 和 Frozen source/Canonical Delta 条款全部保留；本节 owner 用实际 changed domain/owner/protocol/persistence/UI 面证明成功、失败/Cancel、身份失效、适用 defaults/save/reopen；只读调查以有来源的可判定结论结束，不能宣称功能已完成。每个受影响设置 map 归该消费者。

<a id="rules"></a>
## 五规则族、自定义、古中国及运行规则消费

五family/custom/ancient参数和公开选择需精确source mapping；R12 immutable operational snapshot/result是Match admission/restoration/read-only详情的实际前置，不等待storage或benchmark。

唯一能力责任：`T04-RULES`, `T05-ANCIENT-RULES`。本节设置/技术调查与后续实现是同一完整能力；跨节消费不改变来源owner。

详细来源全文（归档合同，不是活跃任务）：

- [D09 — Freeze five-family/custom and ancient rule adapter mapping](MIGRATION_CONTRACTS.md#d09)：Approve the Next public display/exact parameter mapping for five rule families, custom values and ancient Chinese rules.
- [D10 — Preserving rule-family/custom selection and ancient-rule play](MIGRATION_CONTRACTS.md#d10)：Expose D09-approved five-family/custom and ancient-Chinese choices in Match setup without rewriting saved parameters implicitly.
- [D11 — Consume confirmed rules for immutable Match admission and exact restoration](MIGRATION_CONTRACTS.md#d11)：Implement the D Match consumer of B's T03-MATCH-RULES-LIFECYCLE result: freeze rules at admission, restore originals and expose read-only immutable details.

最小验收面与用户断言：上述全文的 Acceptance/Concrete acceptance 和 Frozen source/Canonical Delta 条款全部保留；本节 owner 用实际 changed domain/owner/protocol/persistence/UI 面证明成功、失败/Cancel、身份失效、适用 defaults/save/reopen；只读调查以有来源的可判定结论结束，不能宣称功能已完成。每个受影响设置 map 归该消费者。

<a id="rule-storage"></a>
## 同一不可变规则快照保存分享

同一immutable Match-rule snapshot的结构化save/share归R15唯一owner；其材料性data ownership/representation决定只阻塞save/share，不写personal C或ADR禁止的第二generated channel。

唯一能力责任：`T03-MATCH-RULES-STORAGE-DECISION`。本节设置/技术调查与后续实现是同一完整能力；跨节消费不改变来源owner。

详细来源全文（归档合同，不是活跃任务）：

- [D12 — Approve immutable Match-rule generated details storage/share](MIGRATION_CONTRACTS.md#d12)：Decide data ownership and structured representation for saving/sharing the same immutable Match-rule snapshot without writing personal C or creating ADR0003's forbidden parallel generated-text channel.
- [D13 — Save/share approved immutable Match-rule details](MIGRATION_CONTRACTS.md#d13)：Save/reopen/share immutable generated Match-rule details exactly under D12's approved ownership and representation.

最小验收面与用户断言：上述全文的 Acceptance/Concrete acceptance 和 Frozen source/Canonical Delta 条款全部保留；本节 owner 用实际 changed domain/owner/protocol/persistence/UI 面证明成功、失败/Cancel、身份失效、适用 defaults/save/reopen；只读调查以有来源的可判定结论结束，不能宣称功能已完成。每个受影响设置 map 归该消费者。

<a id="komi"></a>
## 当前 Match 双参与者运行贴目

当前Match user entry/revision及双participant capability/transaction按实际可达path验证，distinct SGF-13 editor/offline图。失败/rollback保留原一致局面，已有GAME-01/03/04 Accepted不扩写。

唯一能力责任：`T04-KOMI`。本节设置/技术调查与后续实现是同一完整能力；跨节消费不改变来源owner。

详细来源全文（归档合同，不是活跃任务）：

- [D14 — Freeze current-Match runtime-komi entry and transaction interface](MIGRATION_CONTRACTS.md#d14)：Freeze the user entry and current Match owner/revision/participant capability contract for runtime komi before GAME-12 implementation.
- [D15 — Atomic two-participant runtime-komi updates](MIGRATION_CONTRACTS.md#d15)：Apply a runtime komi edit transactionally to the D14-admitted current Match participants.

最小验收面与用户断言：上述全文的 Acceptance/Concrete acceptance 和 Frozen source/Canonical Delta 条款全部保留；本节 owner 用实际 changed domain/owner/protocol/persistence/UI 面证明成功、失败/Cancel、身份失效、适用 defaults/save/reopen；只读调查以有来源的可判定结论结束，不能宣称功能已完成。每个受影响设置 map 归该消费者。

## 真实 scoped prerequisites 与跨阶段消费

阶段编号是 Delivery Order，不是 whole-stage hard dependency。原来源中已经 Accepted 的所需功能直接按原 evidence tuple 消费；ledger-only 汇总不是等待门。未来 source 的 required/conditional/field scope 在 [公开路由](MIGRATION_ROUTES.json) 和 [来源合同](MIGRATION_CONTRACTS.md) 中保留，不能扁平化成全票或全阶段开工门。

- 消费 `qualified-local-resource`（R12 / `T01-RESOURCE`）：Actual compatible HumanSL binary/model/profile; qualified-local Coach independent of acquisition。前置仅该已验证结果及适用条件。
- 消费 `bounded-runtime-diagnostics`（R12 / `T01-DIAGNOSTICS`）：Match/Coach actual errors。前置仅该已验证结果及适用条件。
- 消费 `ordinary-rules-snapshot`（R12 / `T03-MATCH-RULES-LIFECYCLE`）：Operational Match admission/original-rule restoration/read-only detail; storage choice only gates save/share。前置仅该已验证结果及适用条件。
- 消费 `first-ui-locale-foundation`（[R11](R11_PLAN.md#behavior-a01) / `T02-I18N-FOUNDATION`）：Actual new UI integration only; settings owner adds Java field map。前置仅该已验证结果及适用条件。
- 消费 `actual-chart-output`（[R11](R11_PLAN.md#behavior-a16) / `T02-EXPORT-03`）：Only requested PK batch winrate images; queue/catalog/color/intervention/durable SGFs independent。前置仅该已验证结果及适用条件。

## 具名待决事项、事实输入与受阻行为

以下 owner 是阶段内具名能力责任角色，实施细化时将阶段责任落到实际执行人；不是要求本轮解决所有未来选择。来源/协议/能力未证明保持 needs-info。普通工程设计不造审批；永久非等价才要逐项产品批准。

<a id="r15-gate-d02"></a>
### D02 来源问题（非新任务）

- Owner：R15 / 保留 Human 控件、竞赛时钟与认输 owner（T02-H25）；阶段细化负责人具名落实实际执行人。
- 待决定/取得证据：Resolve each non-equivalent legacy control separately, without reopening the accepted one-Match-session core.
- 受阻行为：Implement approved retained Human match controls
- 所需结果：Approved executable contract or bounded evidence-backed individual disposition; no silent new product choice.
- 原事实输入与边界全文：[来源](MIGRATION_CONTRACTS.md#d02)；以下保留该事实/合同输入，不表示已经选定新答案：

> Resolve each non-equivalent legacy control separately, without reopening the accepted one-Match-session core.
> Inputs are Inventory GM-HUMAN-GENMOVE/GM-HUMAN-ANA, Ticket12 original disposition and approval source, adapter capabilities, raw timing surfaces, pure-net toggle and play-mode overlay. Identify what user goal each control supplied and whether an existing Next control meets it; a mode label or an adapter enum is not equivalence. Output approved retained controls with entry/default/persistence/failure/cancel and engine-capability mapping, or the exact item-specific approval and reason for a non-equivalent disposition. Existing defaults and accepted role-based live-analysis policy stay unchanged until a decision is approved. Do not copy the Java pre-validation occupancy defect or create a second Human Match owner.

<a id="r15-gate-d04"></a>
### D04 来源问题（非新任务）

- Owner：R15 / 保留 Human 控件、竞赛时钟与认输 owner（T02-CLOCK-DECISION）；阶段细化负责人具名落实实际执行人。
- 待决定/取得证据：Approve a clock/auto-resign policy before GAME-07 implementation, keeping application time distinct from Compute Budget.
- 受阻行为：Competitive clocks and evidence-based auto-resign
- 所需结果：Approved executable contract or bounded evidence-backed individual disposition; no silent new product choice.
- 原事实输入与边界全文：[来源](MIGRATION_CONTRACTS.md#d04)；以下保留该事实/合同输入，不表示已经选定新答案：

> Approve a clock/auto-resign policy before GAME-07 implementation, keeping application time distinct from Compute Budget.
> Use Inventory GM-MATCH-RULES-START and GAME-07 to decide admitted timing systems (main time/byoyomi/Fischer or other choices remain unresolved), application-owned remaining-time authority, start defaults, participant/adapter capability mapping, human-versus-engine charging, pause/resume, deadline/result ordering, timeout/resign outcomes, save and recovery boundaries. Define auto-resign conditions and threshold evidence, including when evidence is missing/stale and when resignation is disabled. Java PK baseline minMove0/consecutive2/winrate10.0 is historical input, not newly approved Next thresholds. Output a separately approved policy and required observable boundary cases, with no arbitrarily expanded platform/timing matrix.

<a id="r15-gate-d06"></a>
### D06 来源问题（非新任务）

- Owner：R15 / 兼容 HumanSL 资源与完整 Coach owner（T02-GAME-08）；阶段细化负责人具名落实实际执行人。
- 待决定/取得证据：Name the compatible HumanSL engine binary/model/profile and complete Coach setup contract required by GAME-08.
- 受阻行为：Acquire qualified HumanSL resources without changing prior set; Independent HumanSL Coach with tactical verification and exact handback
- 所需结果：Approved executable contract or bounded evidence-backed individual disposition; no silent new product choice.
- 原事实输入与边界全文：[来源](MIGRATION_CONTRACTS.md#d06)；以下保留该事实/合同输入，不表示已经选定新答案：

> Name the compatible HumanSL engine binary/model/profile and complete Coach setup contract required by GAME-08.
> The core GAME-01/02/05 Accepted evidence is not HumanSL evidence. Establish actual engine/version/model identity and capability, resource provenance/integrity and supported preset/rank/color configuration. Preserve source defaults as decision inputs: POST_GAME_REVIEW, opponent RANK, 3 dan, RANDOM color, moveTime10s, handicap0, komi7.5 and fromCurrent false; source dialog fields were not written into uiConfig. Decide Next entry/last-successful setting behavior and from-current eligibility explicitly; do not silently add persistence or new presets. Freeze tactical fixture identity and minimum-gain/return-reserve policy from the final source safeguards rather than inventing new quality promises. Name admitted local and any existing remote foreground restoration consumer; remote integration requires that consumer's actual functionality, not all C work.

<a id="r15-gate-d09"></a>
### D09 来源问题（非新任务）

- Owner：R15 / 五规则族、自定义、古中国及运行规则消费 owner（T04-RULES, T05-ANCIENT-RULES）；阶段细化负责人具名落实实际执行人。
- 待决定/取得证据：Approve the Next public display/exact parameter mapping for five rule families, custom values and ancient Chinese rules.
- 受阻行为：Preserving rule-family/custom selection and ancient-rule play
- 所需结果：Approved executable contract or bounded evidence-backed individual disposition; no silent new product choice.
- 原事实输入与边界全文：[来源](MIGRATION_CONTRACTS.md#d09)；以下保留该事实/合同输入，不表示已经选定新答案：

> Approve the Next public display/exact parameter mapping for five rule families, custom values and ancient Chinese rules.
> Inputs are final PR531 semantic parameters and PR556 ancient-Chinese change; Chinese, Japanese/Korean, AGA/BGA, New Zealand and Tromp-Taylor are not Ing and not merely command strings. Only exact semantic matching labels a standard family; any differing or additional parameter stays custom with raw parameters retained. Name actual backend capability/version mappings, including ancient rule choice, save/reopen representation and unsupported behavior. Existing Chinese/GTP evidence does not prove every family. Opening/cancelling/unchanged save may not canonicalize legacy preset parameters. Output executable mapping and native obligations; no arbitrary requirement for five unsupported Generic modes.

<a id="r15-gate-d12"></a>
### D12 来源问题（非新任务）

- Owner：R15 / 同一不可变规则快照保存分享 owner（T03-MATCH-RULES-STORAGE-DECISION）；阶段细化负责人具名落实实际执行人。
- 待决定/取得证据：Decide data ownership and structured representation for saving/sharing the same immutable Match-rule snapshot without writing personal C or creating ADR0003's forbidden parallel generated-text channel.
- 受阻行为：Save/share approved immutable Match-rule details
- 所需结果：Approved executable contract or bounded evidence-backed individual disposition; no silent new product choice.
- 原事实输入与边界全文：[来源](MIGRATION_CONTRACTS.md#d12)；以下保留该事实/合同输入，不表示已经选定新答案：

> Decide data ownership and structured representation for saving/sharing the same immutable Match-rule snapshot without writing personal C or creating ADR0003's forbidden parallel generated-text channel.
> Read ADR0003, GAME-05/SGF-05 save/reopen, PR435 frozen detail fields and final PR531 custom parameters. Output each field's owner, structured save/reopen/share same-value assertion, compatibility read boundaries, unknown-field retention/disposition and byte-identical personal C contract. If this changes ADR or observable product scope obtain item-specific approval before enabling the residual. Stop with an executable approved contract, not a chosen speculative schema implemented in this planning ticket. No runner/clock/policy dependency; block only D13 save/share, never B confirmation or D11 operational lifecycle.

<a id="r15-gate-d14"></a>
### D14 来源问题（非新任务）

- Owner：R15 / 当前 Match 双参与者运行贴目 owner（T04-KOMI）；阶段细化负责人具名落实实际执行人。
- 待决定/取得证据：Freeze the user entry and current Match owner/revision/participant capability contract for runtime komi before GAME-12 implementation.
- 受阻行为：Atomic two-participant runtime-komi updates
- 所需结果：Approved executable contract or bounded evidence-backed individual disposition; no silent new product choice.
- 原事实输入与边界全文：[来源](MIGRATION_CONTRACTS.md#d14)；以下保留该事实/合同输入，不表示已经选定新答案：

> Freeze the user entry and current Match owner/revision/participant capability contract for runtime komi before GAME-12 implementation.
> Existing PK makes the path reachable but does not prove the Java defect in Next. Name supported dual-participant profiles, ACK/final-fence interfaces, cancellation and owner loss behavior; distinguish SGF-13's ordinary komi editor and frozen opening/batch defaults. Define latest-pending-edit and pause-intent interaction without adding a parallel Match owner. Output an implementable admission/revision contract and required real-engine boundary evidence.

## 集成、实际验收与环境义务

- 具名 owner：**R15 高级Match集成/实际验收 owner**；阶段细化时明确实际负责人，拥有本阶段 changed composed workflow 的实际验收，不能交给 read-only Closeout。每个能力owner仍对其局部断言和真实surface负责。
- 环境：实际两participatingengine/admittedrules能力与nativeMatch；兼容HumanSLbinary/model/profile（qualifiedlocal允许）；借remote仅对应分支服务与凭据；clock按approvedpolicy。
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
