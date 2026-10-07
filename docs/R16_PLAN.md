# R16 — 其他业务能力

Status: 滚动阶段计划；产品能力验收尚未完成。

## 目标、完整范围与非目标

真实贡献上传与观看、LAN发布、具名上下文引导和grounded AI棋理解说。

非目标：不假定AIservice/model/funding，不绕统一network/consent/credential，不把generated信息写personalC；无新增trial服务。

本阶段包括以下具名能力章节及其引用的所有 source/Delta/用户断言、默认与保存、错误与取消、原生或实际服务义务。详细契约通过权威历史全文稳定引用保留，不以摘要替换契约；本文件与公开路由的 scoped-result 规则优先于历史来源排期。

<a id="contribution"></a>
## 真实贡献上传、consent 与历史保留能力

official katagotraining.org客户端真实Start、版本化Consent（公开用户名/电力compute/上传/只停本地）、system credentials或明示session-only fallback、secret保护、统一network representability拒绝/no silent Direct、全局互斥、真实上传与取消。60秒transient reconnect、30秒graceful→forceStop、explicit repair、default-off complete-service-identity once-per-game autosave/窄clear-local-data完整保留。历史cuts逐项核对，正式signed client独立R18。

唯一能力责任：`T01-CONTRIBUTION`, `T02-CONTRIBUTION-CUTS`。本节设置/技术调查与后续实现是同一完整能力；跨节消费不改变来源owner。

详细来源全文（归档合同，不是活跃任务）：

- [E01 — Resolve each historical contribution/watch reduction](MIGRATION_CONTRACTS.md#e01)：Resolve each historical reduction against GM-CONTRIBUTE, full CONTRIB-01/GAME-09 and Ticket24, with item-specific approval rather than a blanket preservation exception.
- [E02 — Official Contribution Run: consent, credentials, lifecycle, upload and auto-save](MIGRATION_CONTRACTS.md#e02)：Provide the complete official katagotraining.org Contribution Run independently of the optional main-board watcher and signed installed-client delivery.
- [E03 — Implement individually admitted contribution/watch residual goals](MIGRATION_CONTRACTS.md#e03)：Implement only the item-specific retained contribution/watch successors approved by E01, preserving all unapproved historical user goals as gaps.

最小验收面与用户断言：上述全文的 Acceptance/Concrete acceptance 和 Frozen source/Canonical Delta 条款全部保留；本节 owner 用实际 changed domain/owner/protocol/persistence/UI 面证明成功、失败/Cancel、身份失效、适用 defaults/save/reopen；只读调查以有来源的可判定结论结束，不能宣称功能已完成。每个受影响设置 map 归该消费者。

<a id="watch"></a>
## 贡献棋盘观看与精确交还

消费GAME-01及已验证Contribution Run功能，不等signed delivery；temporary board watch selection/navigation/follow与exact prior game/cursor restore，close不stopRun、stop/failure/exit恢复，不replace/dirty authoritative current game、不恢复livewatch。

唯一能力责任：`T01-WATCH`。本节设置/技术调查与后续实现是同一完整能力；跨节消费不改变来源owner。

详细来源全文（归档合同，不是活跃任务）：

- [E04 — Optional transient Contribution board watcher with exact restoration](MIGRATION_CONTRACTS.md#e04)：Open an optional transient board view only on an active passed Contribution Run without replacing or dirtying the authoritative current game.

最小验收面与用户断言：上述全文的 Acceptance/Concrete acceptance 和 Frozen source/Canonical Delta 条款全部保留；本节 owner 用实际 changed domain/owner/protocol/persistence/UI 面证明成功、失败/Cancel、身份失效、适用 defaults/save/reopen；只读调查以有来源的可判定结论结束，不能宣称功能已完成。每个受影响设置 map 归该消费者。

<a id="lan"></a>
## 局域网棋盘发布

inbound-LAN Start/Stop、actual URL、bounded最新state delivery及源默认/失败契约；不添加凭据、proxy或无来源trial服务，实际LAN client证明不是loopback截图。

唯一能力责任：`T02-PUB-01`。本节设置/技术调查与后续实现是同一完整能力；跨节消费不改变来源owner。

详细来源全文（归档合同，不是活跃任务）：

- [E05 — LAN board publishing with bounded slow-client state](MIGRATION_CONTRACTS.md#e05)：Start/stop LAN board publishing and copy the URL of the actual bound endpoint without credentials or trial mechanics.

最小验收面与用户断言：上述全文的 Acceptance/Concrete acceptance 和 Frozen source/Canonical Delta 条款全部保留；本节 owner 用实际 changed domain/owner/protocol/persistence/UI 面证明成功、失败/Cancel、身份失效、适用 defaults/save/reopen；只读调查以有来源的可判定结论结束，不能宣称功能已完成。每个受影响设置 map 归该消费者。

<a id="guidance"></a>
## 具名教育生产者、永久关闭与重置

具名Educational Tip producer及H10/H11目标先来源/产品核实；permanent dismissal/durable save/dedicated Reset Guidance，Safety Confirmation不可永久取消，reset不改geometry/layout/settings。已Accepted分析不是自动教育producer。

唯一能力责任：`T02-GUIDANCE-PRODUCER`, `T02-GUIDE-01`, `T02-H10`, `T02-H11`。本节设置/技术调查与后续实现是同一完整能力；跨节消费不改变来源owner。

详细来源全文（归档合同，不是活跃任务）：

- [E06 — Decide named Educational Tip producer and historical hint goals](MIGRATION_CONTRACTS.md#e06)：Name at least one admitted Educational Tip producer before building GUIDE-01 dismissal/reset, and resolve historical hint goals individually.
- [E07 — Named Educational Tips with durable dismissal and dedicated reset](MIGRATION_CONTRACTS.md#e07)：Deliver only E06-admitted named Educational Tips at their actual producing entry and a dedicated Reset Guidance action.

最小验收面与用户断言：上述全文的 Acceptance/Concrete acceptance 和 Frozen source/Canonical Delta 条款全部保留；本节 owner 用实际 changed domain/owner/protocol/persistence/UI 面证明成功、失败/Cancel、身份失效、适用 defaults/save/reopen；只读调查以有来源的可判定结论结束，不能宣称功能已完成。每个受影响设置 map 归该消费者。

<a id="generated"></a>
## 生成信息与个人评论分离的历史闭环

ADR0003已批准generated info不写personal C不重开；剩余display/structured export目标与实际owner逐项对齐，不新增parallel generated-text/C channel。

唯一能力责任：`T02-H23`。本节设置/技术调查与后续实现是同一完整能力；跨节消费不改变来源owner。

详细来源全文（归档合同，不是活跃任务）：

- [E08 — Reconcile generated-stat disposition without losing personal comments](MIGRATION_CONTRACTS.md#e08)：Record ADR0003's approved generated-stat versus personal-C separation and reconcile the remaining generated display/export goals.

最小验收面与用户断言：上述全文的 Acceptance/Concrete acceptance 和 Frozen source/Canonical Delta 条款全部保留；本节 owner 用实际 changed domain/owner/protocol/persistence/UI 面证明成功、失败/Cancel、身份失效、适用 defaults/save/reopen；只读调查以有来源的可判定结论结束，不能宣称功能已完成。每个受影响设置 map 归该消费者。

<a id="ai"></a>
## 真实 AI 解说服务、凭据、偏好与 grounded 工作区

official service/auth/eligible registration/model catalog/grounding/funding/correction-call consent仍材料性未决。完整connection/preferences/grounded teaching含actual service、credential/revocation/cancel/stream与工作区source-semantic边界；禁止私有proxy/borrowed client/API-key猜测/automatic paid retry；personal C不存AI生成解说。

唯一能力责任：`T05-AI-CONNECTION`, `T05-AI-GROUNDED-TEACHING`, `T05-AI-PREFERENCES`。本节设置/技术调查与后续实现是同一完整能力；跨节消费不改变来源owner。

详细来源全文（归档合同，不是活跃任务）：

- [E09 — Approve AI service/auth, model and funding/grounding contract](MIGRATION_CONTRACTS.md#e09)：Approve the actual supported AI connection/auth/model and charge/grounding contracts before AI-01 implementation.
- [E10 — Official AI login/API-key connection and credential lifecycle](MIGRATION_CONTRACTS.md#e10)：Implement the E09-approved official authorization and independent API-key connections without unsupported identity shortcuts or chargeable fallbacks.
- [E11 — Separate AI connection and commentary preferences with real model catalog](MIGRATION_CONTRACTS.md#e11)：Expose separate connection and commentary preference surfaces using E09-approved actual account model/reasoning capability.
- [E12 — Grounded AI commentary workspace with explicit charge consent and freshness](MIGRATION_CONTRACTS.md#e12)：Deliver user-initiated grounded AI teaching for an explicit node/range/whole-game plus follow-ups bound to the original frozen node and factual basis.

最小验收面与用户断言：上述全文的 Acceptance/Concrete acceptance 和 Frozen source/Canonical Delta 条款全部保留；本节 owner 用实际 changed domain/owner/protocol/persistence/UI 面证明成功、失败/Cancel、身份失效、适用 defaults/save/reopen；只读调查以有来源的可判定结论结束，不能宣称功能已完成。每个受影响设置 map 归该消费者。

## 真实 scoped prerequisites 与跨阶段消费

阶段编号是 Delivery Order，不是 whole-stage hard dependency。原来源中已经 Accepted 的所需功能直接按原 evidence tuple 消费；ledger-only 汇总不是等待门。未来 source 的 required/conditional/field scope 在 [公开路由](MIGRATION_ROUTES.json) 和 [来源合同](MIGRATION_CONTRACTS.md) 中保留，不能扁平化成全票或全阶段开工门。

- 消费 `qualified-local-resource`（R12 / `T01-RESOURCE`）：Actual official contribution client resource qualification only。前置仅该已验证结果及适用条件。
- 消费 `bounded-runtime-diagnostics`（R12 / `T01-DIAGNOSTICS`）：Official contribution/AI real consumer extensions。前置仅该已验证结果及适用条件。
- 消费 `unified-network-transport`（R14 / `T01-NETWORK`）：Actual contribution subprocess representability and AI service requests。前置仅该已验证结果及适用条件。
- 消费 `first-ui-locale-foundation`（[R11](R11_PLAN.md#behavior-a01) / `T02-I18N-FOUNDATION`）：Actual new UI integration only; settings owner adds Java field map。前置仅该已验证结果及适用条件。

## 具名待决事项、事实输入与受阻行为

以下 owner 是阶段内具名能力责任角色，实施细化时将阶段责任落到实际执行人；不是要求本轮解决所有未来选择。来源/协议/能力未证明保持 needs-info。普通工程设计不造审批；永久非等价才要逐项产品批准。

<a id="r16-gate-e01"></a>
### E01 来源问题（非新任务）

- Owner：R16 / 真实贡献上传、consent 与历史保留能力 owner（T02-CONTRIBUTION-CUTS）；阶段细化负责人具名落实实际执行人。
- 待决定/取得证据：Resolve each historical reduction against GM-CONTRIBUTE, full CONTRIB-01/GAME-09 and Ticket24, with item-specific approval rather than a blanket preservation exception.
- 受阻行为：Implement individually admitted contribution/watch residual goals
- 所需结果：Approved executable contract or bounded evidence-backed individual disposition; no silent new product choice.
- 原事实输入与边界全文：[来源](MIGRATION_CONTRACTS.md#e01)；以下保留该事实/合同输入，不表示已经选定新答案：

> Resolve each historical reduction against GM-CONTRIBUTE, full CONTRIB-01/GAME-09 and Ticket24, with item-specific approval rather than a blanket preservation exception.
> Inputs include custom service/command/SSH, excluded backends (ONNX,+bs50,ROCm,OpenCL), Java automatic rotation/next-game playback, non-19 filtering, result/rules/console controls, manual batch Save All and general menu visibility. For every target recover original decision/approval and user goal, then output retained successor contract (entry/default/persistence/cancel/failure and compatible environment) or approved non-equivalent reason. Unknown approval is an unresolved retained obligation, not permission to execute new behavior. Official E02 Run and optional E04 watch retain their frozen contracts; E04 requires the passed Contribution Run functionality, not full signed component delivery.

<a id="r16-gate-e06"></a>
### E06 来源问题（非新任务）

- Owner：R16 / 具名教育生产者、永久关闭与重置 owner（T02-GUIDANCE-PRODUCER, T02-H10, T02-H11）；阶段细化负责人具名落实实际执行人。
- 待决定/取得证据：Name at least one admitted Educational Tip producer before building GUIDE-01 dismissal/reset, and resolve historical hint goals individually.
- 受阻行为：Named Educational Tips with durable dismissal and dedicated reset
- 所需结果：Approved executable contract or bounded evidence-backed individual disposition; no silent new product choice.
- 原事实输入与边界全文：[来源](MIGRATION_CONTRACTS.md#e06)；以下保留该事实/合同输入，不表示已经选定新答案：

> Name at least one admitted Educational Tip producer before building GUIDE-01 dismissal/reset, and resolve historical hint goals individually.
> Inputs: Ticket21 and its approval, SET-HINT-COMMENT-CTRL (load/reset/write only; no constructed consumer), SET-HINT-AUTOANALYZE (pause-exit tip), restored ANA-16 and SET-RESET-HINTS. Restored ANA-16 controls do not auto-admit guidance; ANA-06 ponder-limit and Ticket25 GMA/readboard notices also do not auto-start this item. Distinguish actual educational goal from unused Java flag and dirty replacement Safety Confirmations. For H10/H11 record item-specific approval or retained producer/entry contract rather than blanket deletion. Output stable producer/tip identities, triggering owner and numbered-phase status (Missing/Partial/Accepted), exact trigger, dismissal first-use/persistence/reset and cancellation/failure behavior. Safety Confirmations are never dismissible and unnamed persist-dismiss remains with its producing capability. No empty Reset Guidance implementation.

<a id="r16-gate-e08"></a>
### E08 来源问题（非新任务）

- Owner：R16 / 生成信息与个人评论分离的历史闭环 owner（T02-H23）；阶段细化负责人具名落实实际执行人。
- 待决定/取得证据：Record ADR0003's approved generated-stat versus personal-C separation and reconcile the remaining generated display/export goals.
- 受阻行为：仅该来源的未决保留能力/确实观察到的差异；既有行为不重开、不永久删除。
- 所需结果：Approved executable contract or bounded evidence-backed individual disposition; no silent new product choice.
- 原事实输入与边界全文：[来源](MIGRATION_CONTRACTS.md#e08)；以下保留该事实/合同输入，不表示已经选定新答案：

> Record ADR0003's approved generated-stat versus personal-C separation and reconcile the remaining generated display/export goals.
> Read Ticket20/ADR0003, SET-MAIN-PANEL append-WR and SGF-05/ANA-08/GAME-05. Generated winrate/score/playouts must not serialize or overwrite personal C; a Java file's existing whole C stays personal text. The analysis pane remains the numeric display and structured analysis-header exchange stays ANA-08 with its own acceptance. Output each retained display/export route and existing approval for the prohibited write-into-C behavior; do not delete every generated-information goal or introduce a second generated-text channel. This is disposition reconciliation, not duplicate SGF-05 or ANA-08 implementation.

<a id="r16-gate-e09"></a>
### E09 来源问题（非新任务）

- Owner：R16 / 真实 AI 解说服务、凭据、偏好与 grounded 工作区 owner（T05-AI-CONNECTION, T05-AI-PREFERENCES, T05-AI-GROUNDED-TEACHING）；阶段细化负责人具名落实实际执行人。
- 待决定/取得证据：Approve the actual supported AI connection/auth/model and charge/grounding contracts before AI-01 implementation.
- 受阻行为：Official AI login/API-key connection and credential lifecycle; Separate AI connection and commentary preferences with real model catalog; Grounded AI commentary workspace with explicit charge consent and freshness
- 所需结果：Approved executable contract or bounded evidence-backed individual disposition; no silent new product choice.
- 原事实输入与边界全文：[来源](MIGRATION_CONTRACTS.md#e09)；以下保留该事实/合同输入，不表示已经选定新答案：

> Approve the actual supported AI connection/auth/model and charge/grounding contracts before AI-01 implementation.
> Use PR575/c0455839311b73548e479e6ef26f5722af31bdf2 and UD-03-012 as behavior sources, not Next operational proof. Decide officially authorized ChatGPT login availability and independent API-key mode, eligible application/client registration and supported platforms; official access must exist and no borrowing another app's client/private proxy or automatic API-key fallback is permitted. Freeze PKCE/loopback/JWKS/refresh/revocation contracts only for the actually supported authorization route. Define System Credential Store/fallback, workspace/account ownership, remote network consumer and invalidation.
> Approve actual account-supported model/reasoning catalog, manual-model persistence and unavailable catalog semantics; unknown options are not guessed. Freeze user-driven single-node/range/whole-game/follow-up scope, immutable position/rules/PV/capture/liberty/analysis facts, missing/unknown loss representation, generated output ownership outside personal C, stream completion/failure/cancel and context freshness. Explicitly decide cost disclosure, funding responsibility and consent before chargeable requests or additional correction calls; no automatic paid retries or scope-change request. Output executable service/auth/model/grounded input and funding contracts with genuine credential/service environment gates; no hardcoded arbitrary provider/model/price.

## 集成、实际验收与环境义务

- 具名 owner：**R16 业务服务集成/实际验收 owner**；阶段细化时明确实际负责人，拥有本阶段 changed composed workflow 的实际验收，不能交给 read-only Closeout。每个能力owner仍对其局部断言和真实surface负责。
- 环境：官方贡献client/真实authorizedupload与consent/systemstore或明确sessionfallback、实际LANclient；AI官方service/model/权限/费用/correctionconsent；受影响nativeUI与引擎。
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
