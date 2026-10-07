# R18 — 正式发行

Status: 滚动阶段计划；产品能力验收尚未完成。

## 目标、完整范围与非目标

保留旧未执行R11正式渠道、签名组件、canonical包安装更新回滚卸载、文件注册和支持路径义务。

非目标：本轮不选择生产repo/channel/platform/key/secret，不启动发行；不将runtime证据伪装installedlive，不增加CanonicalArtifact。

本阶段包括以下具名能力章节及其引用的所有 source/Delta/用户断言、默认与保存、错误与取消、原生或实际服务义务。详细契约通过权威历史全文稳定引用保留，不以摘要替换契约；本文件与公开路由的 scoped-result 规则优先于历史来源排期。

<a id="channel"></a>
## 正式仓库、渠道、签名与组件目标决定

保留product identifier/data identity，决定final repo/stable-beta/channel/source/key/signing/feedback/help与version-selection。历史flavor/JRE/JCEF用户目标对既有app-core/acquired-KataGo manifest逐项协调；不因文档排期自行选择生产endpoint/platform/schema/secret。

唯一能力责任：`T02-H28`, `T05-UPDATE-CHANNEL-DECISION`, `T06-RELEASE-CHANNEL`。本节设置/技术调查与后续实现是同一完整能力；跨节消费不改变来源owner。

详细来源全文（归档合同，不是活跃任务）：

- [R1101 — Approve final release repository, signing/update/support configuration](MIGRATION_CONTRACTS.md#r1101)：Decide final release repository, platform admission/signing responsibility, stable/beta update sources/trusted public keys and feedback/help routing while preserving product identity and user data.
- [R1102 — Resolve legacy flavor/JRE/JCEF component user goals](MIGRATION_CONTRACTS.md#r1102)：Resolve historical REL-C04 flavor-guessing/JRE/JCEF user goals against the existing REL-05 app-core/acquired-KataGo manifest.

最小验收面与用户断言：上述全文的 Acceptance/Concrete acceptance 和 Frozen source/Canonical Delta 条款全部保留；本节 owner 用实际 changed domain/owner/protocol/persistence/UI 面证明成功、失败/Cancel、身份失效、适用 defaults/save/reopen；只读调查以有来源的可判定结论结束，不能宣称功能已完成。每个受影响设置 map 归该消费者。

<a id="preflight-trust"></a>
## 精确候选 Preflight 与正式产物信任

maintainer exact-commit nonmutating preflight不是Capability/publication；真正AuthentiCode timestamp、macOS signing/notarization/staple、Linux exact trust/dependency证据与授权。CI/unsigned build不代表Shipped。

唯一能力责任：`T02-REL-01`, `T02-REL-02`。本节设置/技术调查与后续实现是同一完整能力；跨节消费不改变来源owner。

详细来源全文（归档合同，不是活跃任务）：

- [R1103 — Exact-commit non-mutating release preflight and dry-run](MIGRATION_CONTRACTS.md#r1103)：Extend existing release validators/non-mutating dry-run to identify exact tag/commit/platform/signing state and evidence limits.
- [R1104 — Trusted canonical production artifacts and feed exclusion](MIGRATION_CONTRACTS.md#r1104)：Produce trusted artifacts under the approved existing release configuration without introducing a new platform design.

最小验收面与用户断言：上述全文的 Acceptance/Concrete acceptance 和 Frozen source/Canonical Delta 条款全部保留；本节 owner 用实际 changed domain/owner/protocol/persistence/UI 面证明成功、失败/Cancel、身份失效、适用 defaults/save/reopen；只读调查以有来源的可判定结论结束，不能宣称功能已完成。每个受影响设置 map 归该消费者。

<a id="discovery-components"></a>
## 手动更新发现与签名组件获取

手动更新trusted envelope/payload与installed signed manifest/acquisition，用户explicit动作、cancel/failure保留last-good；仅app-owned managed runtime，OS WebView/obsoleteJava不是managed component。实际network消费已验证相应result，不依赖全部远程算力功能。

唯一能力责任：`T01-RELEASE`, `T02-REL-03`。本节设置/技术调查与后续实现是同一完整能力；跨节消费不改变来源owner。

详细来源全文（归档合同，不是活跃任务）：

- [R1105 — Manual stable/beta signed update discovery without install mutation](MIGRATION_CONTRACTS.md#r1105)：Implement Help-triggered manual update discovery under R1101-approved stable/beta official/GitHub policy and SemVer.
- [R1106 — Signed installed manifest and explicit component acquisition residual](MIGRATION_CONTRACTS.md#r1106)：Finish REL-05 signed installed-manifest and explicit acquisition residual, consuming B's qualified runtime resource functionality rather than re-owning it.

最小验收面与用户断言：上述全文的 Acceptance/Concrete acceptance 和 Frozen source/Canonical Delta 条款全部保留；本节 owner 用实际 changed domain/owner/protocol/persistence/UI 面证明成功、失败/Cancel、身份失效、适用 defaults/save/reopen；只读调查以有来源的可判定结论结束，不能宣称功能已完成。每个受影响设置 map 归该消费者。

<a id="windows-update"></a>
## Windows 更新应用与反向回滚

Windows helper apply、backup/reverse rollback/journal/restart按既有完整契约真实安装候选，拒绝伪装atomic/完成，保留秘密与user state，production动作独立授权。

唯一能力责任：`T02-REL-06`, `T02-REL-08`。本节设置/技术调查与后续实现是同一完整能力；跨节消费不改变来源owner。

详细来源全文（归档合同，不是活跃任务）：

- [R1107 — Verified Windows component download/helper apply without data changes](MIGRATION_CONTRACTS.md#r1107)：Implement the existing NSIS and portable update-apply contract using selected newer managed components.
- [R1108 — Windows update reverse rollback, durable result journal and repair](MIGRATION_CONTRACTS.md#r1108)：Handle pre-apply and post-quit update failure with complete replacement backup, reverse rollback, durable result journal and restored-version restart.

最小验收面与用户断言：上述全文的 Acceptance/Concrete acceptance 和 Frozen source/Canonical Delta 条款全部保留；本节 owner 用实际 changed domain/owner/protocol/persistence/UI 面证明成功、失败/Cancel、身份失效、适用 defaults/save/reopen；只读调查以有来源的可判定结论结束，不能宣称功能已完成。每个受影响设置 map 归该消费者。

<a id="handoff"></a>
## macOS/Linux 已验证包交接

macOS DMG/Linux AppImage verified包hand-off在对应native平台验trust/launch/update/失败恢复，不用Windows运行或CI替代。

唯一能力责任：`T02-REL-07`。本节设置/技术调查与后续实现是同一完整能力；跨节消费不改变来源owner。

详细来源全文（归档合同，不是活跃任务）：

- [R1109 — Verified macOS/Linux package acquisition and non-overwriting handoff](MIGRATION_CONTRACTS.md#r1109)：Download and verify matching DMG/AppImage for the approved selected channel/source with resume/pause/cancel and supported fallback, then hand off without overwriting the running application.

最小验收面与用户断言：上述全文的 Acceptance/Concrete acceptance 和 Frozen source/Canonical Delta 条款全部保留；本节 owner 用实际 changed domain/owner/protocol/persistence/UI 面证明成功、失败/Cancel、身份失效、适用 defaults/save/reopen；只读调查以有来源的可判定结论结束，不能宣称功能已完成。每个受影响设置 map 归该消费者。

<a id="installed"></a>
## Canonical Artifact 安装便携路径与卸载

Windows NSIS+portable、macOS DMG、Linux AppImage独立artifact准入/evidence，不把缺其他平台变已验平台hard gate；其他MSI/deb/rpm不自动Canonical。installation/resource path/file注册/update/removal与portable colocated state完整验；已功能资源资格不等待此门。

唯一能力责任：`T02-REL-04`。本节设置/技术调查与后续实现是同一完整能力；跨节消费不改变来源owner。

详细来源全文（归档合同，不是活跃任务）：

- [R1110 — Per-artifact installed/unpacked lifecycle, trust, data and associations](MIGRATION_CONTRACTS.md#r1110)：Accept each existing Canonical Artifact independently for install/unpack, primary launch, runtime/trust/state, upgrade/update/handoff and removal, including packaging-owned file association registration.

最小验收面与用户断言：上述全文的 Acceptance/Concrete acceptance 和 Frozen source/Canonical Delta 条款全部保留；本节 owner 用实际 changed domain/owner/protocol/persistence/UI 面证明成功、失败/Cancel、身份失效、适用 defaults/save/reopen；只读调查以有来源的可判定结论结束，不能宣称功能已完成。每个受影响设置 map 归该消费者。

<a id="activation"></a>
## 安装态 SGF/GIB 文件激活

继承APP-01 native coldSGF/warmGIB/single-window/dirtyCancel原candidate，新增安装association/registration与实际激活gap，语义与注册分开。

唯一能力责任：`T01-ACTIVATION`。本节设置/技术调查与后续实现是同一完整能力；跨节消费不改变来源owner。

详细来源全文（归档合同，不是活跃任务）：

- [R1111 — Finish APP-01 installed association activation without rewriting native scope](MIGRATION_CONTRACTS.md#r1111)：Complete APP-01 final installed association gate using R1110 registration/artifacts and the inherited native activation/open/replace owner.

最小验收面与用户断言：上述全文的 Acceptance/Concrete acceptance 和 Frozen source/Canonical Delta 条款全部保留；本节 owner 用实际 changed domain/owner/protocol/persistence/UI 面证明成功、失败/Cancel、身份失效、适用 defaults/save/reopen；只读调查以有来源的可判定结论结束，不能宣称功能已完成。每个受影响设置 map 归该消费者。

<a id="support"></a>
## 安装态支持日志与身份路由

消费 [R11 identity](R11_PLAN.md#behavior-a02) / [R12 bounded diagnostics](R12_PLAN.md#diagnostics) 结果，只追加installed logs/support path/routing；不得整体把功能诊断推迟到R18或自动上传。

本节由 **R18 installed support/identity owner** 承担安装态增量；首消费者 identity 与 diagnostics 的唯一来源责任分别保留在 R11/R12。

详细来源全文（归档合同，不是活跃任务）：

- [R1112 — Finish installed identity/support routing and bounded support-bundle evidence](MIGRATION_CONTRACTS.md#r1112)：Finish only REL-09/10 installed support/identity residuals, consuming A build identity and B diagnostic functionality rather than rebuilding either.

最小验收面与用户断言：上述全文的 Acceptance/Concrete acceptance 和 Frozen source/Canonical Delta 条款全部保留；本节 owner 用实际 changed domain/owner/protocol/persistence/UI 面证明成功、失败/Cancel、身份失效、适用 defaults/save/reopen；只读调查以有来源的可判定结论结束，不能宣称功能已完成。每个受影响设置 map 归该消费者。

## 真实 scoped prerequisites 与跨阶段消费

阶段编号是 Delivery Order，不是 whole-stage hard dependency。原来源中已经 Accepted 的所需功能直接按原 evidence tuple 消费；ledger-only 汇总不是等待门。未来 source 的 required/conditional/field scope 在 [公开路由](MIGRATION_ROUTES.json) 和 [来源合同](MIGRATION_CONTRACTS.md) 中保留，不能扁平化成全票或全阶段开工门。

- 消费 `bounded-runtime-diagnostics`（R12 / `T01-DIAGNOSTICS`）：Installed log/support paths only。前置仅该已验证结果及适用条件。
- 消费 `unified-network-transport`（R14 / `T01-NETWORK`）：Actual update/acquisition transport。前置仅该已验证结果及适用条件。
- 消费 `first-ui-locale-foundation`（[R11](R11_PLAN.md#behavior-a01) / `T02-I18N-FOUNDATION`）：Actual new UI integration only; settings owner adds Java field map。前置仅该已验证结果及适用条件。
- 消费 `actual-identity-routing`（[R11](R11_PLAN.md#behavior-a02) / `T01-IDENTITY`）：Installed support/feedback/help routing plus separately approved production channel。前置仅该已验证结果及适用条件。

## 具名待决事项、事实输入与受阻行为

以下 owner 是阶段内具名能力责任角色，实施细化时将阶段责任落到实际执行人；不是要求本轮解决所有未来选择。来源/协议/能力未证明保持 needs-info。普通工程设计不造审批；永久非等价才要逐项产品批准。

<a id="r18-gate-r1101"></a>
### R1101 来源问题（非新任务）

- Owner：R18 / 正式仓库、渠道、签名与组件目标决定 owner（T06-RELEASE-CHANNEL, T05-UPDATE-CHANNEL-DECISION）；阶段细化负责人具名落实实际执行人。
- 待决定/取得证据：Decide final release repository, platform admission/signing responsibility, stable/beta update sources/trusted public keys and feedback/help routing while preserving product identity and user data.
- 受阻行为：Trusted canonical production artifacts and feed exclusion; Manual stable/beta signed update discovery without install mutation; Signed installed manifest and explicit component acquisition residual; Finish installed identity/support routing and bounded support-bundle evidence
- 所需结果：Approved executable contract or bounded evidence-backed individual disposition; no silent new product choice.
- 原事实输入与边界全文：[来源](MIGRATION_CONTRACTS.md#r1101)；以下保留该事实/合同输入，不表示已经选定新答案：

> Decide final release repository, platform admission/signing responsibility, stable/beta update sources/trusted public keys and feedback/help routing while preserving product identity and user data.
> Inputs: current RELEASE_PROCESS/CHECKLIST and Matrix REL-02/03/10, A build-identity correction, UD-05-22 version-selection policy. Product identifier org.lizzieyzy.next, binary lizzieyzy-next-desktop and user-data identity are independent of final repository; Cargo repository drift is metadata, not identity migration. Output the approved configured source/channel/key/support contract and administrator/credential owners, or exact unavailable infrastructure blockers. Unconfigured channels/links are honestly unavailable, never fabricated available UI. No production publication is authorized by this decision.
> Existing stable/beta, official/GitHub and SemVer discovery are inherited contracts; freeze how they map to actual supported sources. UD-05-22 beta policy (latest valid stable/pre candidate, stable tie preference, selected-version NO_PACKAGE without old-version downgrade, explicit malformed-list failure) is a source behavior to approve/adapt explicitly, not automatic reuse of Java endpoints/date versions. Name updater envelope/payload trust and unsigned-validation-feed exclusion. Do not design a new distribution platform, select endpoints or provision credentials in this drafting ticket.

<a id="r18-gate-r1102"></a>
### R1102 来源问题（非新任务）

- Owner：R18 / 正式仓库、渠道、签名与组件目标决定 owner（T02-H28）；阶段细化负责人具名落实实际执行人。
- 待决定/取得证据：Resolve historical REL-C04 flavor-guessing/JRE/JCEF user goals against the existing REL-05 app-core/acquired-KataGo manifest.
- 受阻行为：仅该来源的未决保留能力/确实观察到的差异；既有行为不重开、不永久删除。
- 所需结果：Approved executable contract or bounded evidence-backed individual disposition; no silent new product choice.
- 原事实输入与边界全文：[来源](MIGRATION_CONTRACTS.md#r1102)；以下保留该事实/合同输入，不表示已经选定新答案：

> Resolve historical REL-C04 flavor-guessing/JRE/JCEF user goals against the existing REL-05 app-core/acquired-KataGo manifest.
> Read Ticket14/Ticket24 original disposition and item-specific approval. Distinguish pure Java runtime implementation from real bundled-resource/acquisition user goal, and retain any unmet goal with a declared successor contract. A readboard/contribution component joins only if its domain accepts that update behavior; no Deferred optional component becomes an earlier R11 exit gate. Output individual retained contract or approved non-equivalent reason for flavor guessing, JRE and JCEF remainder; do not infer blanket permission from the manifest redesign or implement new platform infrastructure.

## 集成、实际验收与环境义务

- 具名 owner：**R18 artifact-specific 发行集成/Installed Live Evidence owner**；阶段细化时明确实际负责人，拥有本阶段 changed composed workflow 的实际验收，不能交给 read-only Closeout。每个能力owner仍对其局部断言和真实surface负责。
- 环境：WindowsNSIS+portable、macOSDMG、LinuxAppImage各自机器/权限/signing/notarization/trust；实际update渠道/key与artifact授权；platform独立准入。
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
