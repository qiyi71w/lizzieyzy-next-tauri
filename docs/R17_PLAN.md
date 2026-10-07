# R17 — 功能对等收尾

Status: 滚动阶段计划；产品能力验收尚未完成。

## 目标、完整范围与非目标

全语言、Java设置安全迁入、保留appearance初始化、入口默认保存、可访问性与跨功能完整闭环。

非目标：不改v1/原Accepted；skip不等永久cut；不以全部owner未完阻断approved-field子集，不以子集冒称fullimport。

本阶段包括以下具名能力章节及其引用的所有 source/Delta/用户断言、默认与保存、错误与取消、原生或实际服务义务。详细契约通过权威历史全文稳定引用保留，不以摘要替换契约；本文件与公开路由的 scoped-result 规则优先于历史来源排期。

<a id="initialization"></a>
## 保留初始化和主机变更用户目标

hostname-triggered deletion和initialization/wizard逐项保留/批准，不静默wipe；invalid-geometry恢复独立既有合同。

唯一能力责任：`T02-H03`, `T02-H04`。本节设置/技术调查与后续实现是同一完整能力；跨节消费不改变来源owner。

详细来源全文（归档合同，不是活跃任务）：

- [F01 — Decide hostname-reset and initialization user goals individually](MIGRATION_CONTRACTS.md#f01)：Resolve hostname-triggered deletion and initialization/settings-wizard goals from original decisions and approvals without silently wiping preferences.
- [F02 — Approved retained initialization and host-change behaviors](MIGRATION_CONTRACTS.md#f02)：Implement only the initialization/host-change goals explicitly retained and approved by F01.

最小验收面与用户断言：上述全文的 Acceptance/Concrete acceptance 和 Frozen source/Canonical Delta 条款全部保留；本节 owner 用实际 changed domain/owner/protocol/persistence/UI 面证明成功、失败/Cancel、身份失效、适用 defaults/save/reopen；只读调查以有来源的可判定结论结束，不能宣称功能已完成。每个受影响设置 map 归该消费者。

<a id="appearance"></a>
## 字体、主题编辑与布局工具栏预设

retained font/theme preset/custom editor/texture及toolbar/layout/large panel全目标，curated APPEAR-01/adaptive现实现不能证明全等价。Next Classic/System-DPI/adaptive默认不因收集source自动改变，各字段map跟实际owner交付。

唯一能力责任：`T02-H05`, `T02-H06`, `T02-H07`, `T02-H08`, `T02-H09`, `T02-H22`, `T05-THEME-PRESETS`。本节设置/技术调查与后续实现是同一完整能力；跨节消费不改变来源owner。

详细来源全文（归档合同，不是活跃任务）：

- [F03 — Approve retained font, theme and layout goals with item-specific dispositions](MIGRATION_CONTRACTS.md#f03)：Resolve each historical font/theme/layout reduction and the newer texture/default migration goal without treating curated APPEAR-01 or adaptive layout as full equivalence.
- [F04 — Approved application font/accessibility controls](MIGRATION_CONTRACTS.md#f04)：Implement the F03-approved application font-size user goal alongside system DPI/accessibility scaling.
- [F05 — Approved theme presets, custom assets and editing with safe preview](MIGRATION_CONTRACTS.md#f05)：Implement F03-retained preset/custom-theme/editor goals without copying Swing assets or changing defaults implicitly.
- [F06 — Approved workspace/toolbar presets and large-panel goals](MIGRATION_CONTRACTS.md#f06)：Implement F03-approved retained workspace preset, toolbar visibility/order/wrap and large-sub/large-WR goals within the Next workspace.

最小验收面与用户断言：上述全文的 Acceptance/Concrete acceptance 和 Frozen source/Canonical Delta 条款全部保留；本节 owner 用实际 changed domain/owner/protocol/persistence/UI 面证明成功、失败/Cancel、身份失效、适用 defaults/save/reopen；只读调查以有来源的可判定结论结束，不能宣称功能已完成。每个受影响设置 map 归该消费者。

<a id="localization"></a>
## 完整语言清单与翻译

消费R11首search UI locale/key/fallback基础而非全部global search；支持locale清单/selection/switch/fallback需具名决定，随后每admitted locale全资源翻译，不以混杂部分语言宣称完整。F08不等Java import。

唯一能力责任：`T02-I18N-01`。本节设置/技术调查与后续实现是同一完整能力；跨节消费不改变来源owner。

详细来源全文（归档合同，不是活跃任务）：

- [F07 — Approve full supported locale and translation/fallback scope](MIGRATION_CONTRACTS.md#f07)：Approve the complete supported-locale list and translation scope for I18N-01 from A's actual first-consumer resource/fallback result.
- [F08 — Complete translations and durable locale selection without partial mixing](MIGRATION_CONTRACTS.md#f08)：Complete every F07-admitted locale across migrated functional UI, typed errors/status/help and accessibility labels using A's same resource/locale/fallback seam.

最小验收面与用户断言：上述全文的 Acceptance/Concrete acceptance 和 Frozen source/Canonical Delta 条款全部保留；本节 owner 用实际 changed domain/owner/protocol/persistence/UI 面证明成功、失败/Cancel、身份失效、适用 defaults/save/reopen；只读调查以有来源的可判定结论结束，不能宣称功能已完成。每个受影响设置 map 归该消费者。

<a id="import"></a>
## 逐字段白名单、预览与子集原子迁入

每字段Java key/type/default/save→Next owner/key/type/transform/default/eligibility/conflict，未知unsupported invalid secret可见skip。map-only collection按字段，approved子集不等其他字段；explicit preview原/译/current值与选择、read-only source、confirm全选子集atomic memory+durable应用、失败全不改、restart持久；不搬workdir、不导入secret/arbitrary runnable command、不start/download引擎。skip保留原owner decision与implementation gap，所有retained fields有实际behavior/import覆盖或逐项批准处置前禁止complete-import/PREF-02/full-function/final-integration claim。

唯一能力责任：`T02-JAVA-IMPORT`。本节设置/技术调查与后续实现是同一完整能力；跨节消费不改变来源owner。

详细来源全文（归档合同，不是活跃任务）：

- [F09 — Freeze per-owner Java settings whitelist and conflict semantics](MIGRATION_CONTRACTS.md#f09)：Collect and approve an explicit field-by-field whitelist for Java settings import from completed behavior-owner maps.
- [F10 — Read-only Java settings preview, confirmation and atomic import](MIGRATION_CONTRACTS.md#f10)：Import F09-whitelisted Java settings from an explicitly selected source through read-only parse, visible preview and explicit confirmation followed by all-or-nothing atomic application.

最小验收面与用户断言：上述全文的 Acceptance/Concrete acceptance 和 Frozen source/Canonical Delta 条款全部保留；本节 owner 用实际 changed domain/owner/protocol/persistence/UI 面证明成功、失败/Cancel、身份失效、适用 defaults/save/reopen；只读调查以有来源的可判定结论结束，不能宣称功能已完成。每个受影响设置 map 归该消费者。

<a id="accessibility"></a>
## UI-02 与原生工作台可访问性

UI-02 pendingmutation/engine-event responsiveness及实际原生accessibility集成；每新增UI owner仍负责键盘/cancel/focus/localized overflow，不把1280×800Chromium窄proof变全DPI/native证明。

唯一能力责任：`T02-UI-02`, `T05-WORKBENCH-ACCESS`。本节设置/技术调查与后续实现是同一完整能力；跨节消费不改变来源owner。

详细来源全文（归档合同，不是活跃任务）：

- [F11 — Finish non-blocking board intent and workbench accessibility residuals](MIGRATION_CONTRACTS.md#f11)：Finish UI-02's actual board-mutation/engine-event concurrency residual and integrate admitted workbench accessibility cases without reopening narrow chrome history.

最小验收面与用户断言：上述全文的 Acceptance/Concrete acceptance 和 Frozen source/Canonical Delta 条款全部保留；本节 owner 用实际 changed domain/owner/protocol/persistence/UI 面证明成功、失败/Cancel、身份失效、适用 defaults/save/reopen；只读调查以有来源的可判定结论结束，不能宣称功能已完成。每个受影响设置 map 归该消费者。

<a id="entries"></a>
## 139 heading/20 route 入口默认保存闭环

139-heading/20-route ledger（135 capabilities+4 non-capability headings）入口/default/save/failure/field逐项核对唯一归R17；不是新增139任务。先消费各owner结果，未决差异保留owner与gap。

唯一能力责任：`T02-ENTRY-CLOSEOUT`。本节设置/技术调查与后续实现是同一完整能力；跨节消费不改变来源owner。

详细来源全文（归档合同，不是活跃任务）：

- [F12 — Close all139 frozen headings and20 routes with entry/default/save dispositions](MIGRATION_CONTRACTS.md#f12)：Complete the entry/default/persistence/cancel/failure/field-map ledger for all139 frozen headings and20 cross-domain routes using actual per-owner results.

最小验收面与用户断言：上述全文的 Acceptance/Concrete acceptance 和 Frozen source/Canonical Delta 条款全部保留；本节 owner 用实际 changed domain/owner/protocol/persistence/UI 面证明成功、失败/Cancel、身份失效、适用 defaults/save/reopen；只读调查以有来源的可判定结论结束，不能宣称功能已完成。每个受影响设置 map 归该消费者。

<a id="integration"></a>
## 跨功能组合验收

跨R11–R17真实composed workflows/data/lifecycle验收唯一归R17集成owner，消费各已验证terminal scope、entry ledger与upstream result；不为所有前票重跑同一原验收、不让Closeout代做产品验收。其他阶段各自集成责任仍在其exit门。

唯一能力责任：`T06-F-INTEGRATION`。本节设置/技术调查与后续实现是同一完整能力；跨节消费不改变来源owner。

详细来源全文（归档合同，不是活跃任务）：

- [F13 — Integrate A–F functional workflows with original evidence and terminal gates](MIGRATION_CONTRACTS.md#f13)：Own final cross-functional product acceptance for the six functional batches, consuming rather than duplicating per-feature/entry/upstream acceptance.

最小验收面与用户断言：上述全文的 Acceptance/Concrete acceptance 和 Frozen source/Canonical Delta 条款全部保留；本节 owner 用实际 changed domain/owner/protocol/persistence/UI 面证明成功、失败/Cancel、身份失效、适用 defaults/save/reopen；只读调查以有来源的可判定结论结束，不能宣称功能已完成。每个受影响设置 map 归该消费者。

<a id="upstream"></a>
## 冻结端点之后全量上游复核

全量追加af0e07a7386483f3bfc8a15780de72ffc2f0de4c之后区间，source/版本contains与处置/用户case；不移动Java v1，不把严重当前可达correctness拖到这里，提前归实际consumer。

唯一能力责任：`T06-UPSTREAM-FINAL`。本节设置/技术调查与后续实现是同一完整能力；跨节消费不改变来源owner。

详细来源全文（归档合同，不是活跃任务）：

- [F14 — Append final frozen upstream interval and resolve remaining user-visible deltas](MIGRATION_CONTRACTS.md#f14)：Append a frozen upstream audit interval beginning after af0e07a7386483f3bfc8a15780de72ffc2f0de4c and ending at a full upstream commit fixed at audit start.

最小验收面与用户断言：上述全文的 Acceptance/Concrete acceptance 和 Frozen source/Canonical Delta 条款全部保留；本节 owner 用实际 changed domain/owner/protocol/persistence/UI 面证明成功、失败/Cancel、身份失效、适用 defaults/save/reopen；只读调查以有来源的可判定结论结束，不能宣称功能已完成。每个受影响设置 map 归该消费者。

## 真实 scoped prerequisites 与跨阶段消费

阶段编号是 Delivery Order，不是 whole-stage hard dependency。原来源中已经 Accepted 的所需功能直接按原 evidence tuple 消费；ledger-only 汇总不是等待门。未来 source 的 required/conditional/field scope 在 [公开路由](MIGRATION_ROUTES.json) 和 [来源合同](MIGRATION_CONTRACTS.md) 中保留，不能扁平化成全票或全阶段开工门。

- 本阶段唯一交付 `per-field-java-map`（`T02-JAVA-IMPORT`）：Executable approved whitelist entries/conflict policy plus visible retained-owner/implementation gap record。其他消费者只取其通过scope；不接管本owner。
- 消费 `per-field-java-map`（R17 / `T02-JAVA-IMPORT`）：Read-only preview and all-or-nothing confirmed import of approved selected subset。前置仅该已验证结果及适用条件。
- 消费 `first-ui-locale-foundation`（[R11](R11_PLAN.md#behavior-a01) / `T02-I18N-FOUNDATION`）：Actual new UI integration only; settings owner adds Java field map。前置仅该已验证结果及适用条件。

## 具名待决事项、事实输入与受阻行为

以下 owner 是阶段内具名能力责任角色，实施细化时将阶段责任落到实际执行人；不是要求本轮解决所有未来选择。来源/协议/能力未证明保持 needs-info。普通工程设计不造审批；永久非等价才要逐项产品批准。

<a id="r17-gate-f01"></a>
### F01 来源问题（非新任务）

- Owner：R17 / 保留初始化和主机变更用户目标 owner（T02-H03, T02-H04）；阶段细化负责人具名落实实际执行人。
- 待决定/取得证据：Resolve hostname-triggered deletion and initialization/settings-wizard goals from original decisions and approvals without silently wiping preferences.
- 受阻行为：Approved retained initialization and host-change behaviors
- 所需结果：Approved executable contract or bounded evidence-backed individual disposition; no silent new product choice.
- 原事实输入与边界全文：[来源](MIGRATION_CONTRACTS.md#f01)；以下保留该事实/合同输入，不表示已经选定新答案：

> Resolve hostname-triggered deletion and initialization/settings-wizard goals from original decisions and approvals without silently wiping preferences.
> Inputs: Inventory SET-FIRST-LAUNCH hostname-wipe half and SET-FIRST-USE, Ticket09, WINDOW-01 invalid-geometry reset, PREF-01 categorized preferences and engine bootstrap owner. Hostname lookup failure must not fake a machine change. Determine retained user goals, initial/default/explicit action, data ownership and cancel/failure semantics, or item-specific approved non-equivalent reason. Do not invent a forced wizard or expand geometry reset to semantic settings; no automatic engine start/download. Output one independently traceable disposition per H03/H04 and an executable retained setup contract if applicable.

<a id="r17-gate-f03"></a>
### F03 来源问题（非新任务）

- Owner：R17 / 字体、主题编辑与布局工具栏预设 owner（T02-H05, T02-H06, T02-H07, T02-H08, T02-H09, T02-H22, T05-THEME-PRESETS）；阶段细化负责人具名落实实际执行人。
- 待决定/取得证据：Resolve each historical font/theme/layout reduction and the newer texture/default migration goal without treating curated APPEAR-01 or adaptive layout as full equivalence.
- 受阻行为：Approved application font/accessibility controls; Approved theme presets, custom assets and editing with safe preview; Approved workspace/toolbar presets and large-panel goals; Finish non-blocking board intent and workbench accessibility residuals
- 所需结果：Approved executable contract or bounded evidence-backed individual disposition; no silent new product choice.
- 原事实输入与边界全文：[来源](MIGRATION_CONTRACTS.md#f03)；以下保留该事实/合同输入，不表示已经选定新答案：

> Resolve each historical font/theme/layout reduction and the newer texture/default migration goal without treating curated APPEAR-01 or adaptive layout as full equivalence.
> Inputs: H05 app font-size (Java12/16/20 and custom12–20), H06 Apple/Morandi/custom image, H07 separate theme editor, H08 ExtraMode/classic/custom layouts, H09 toolbar wrap/order/visibility, H22 mutually exclusive large-sub/large-WR; Ticket09/Ticket20 approvals and UD-05-12 texture change. Preserve user goals and original approval per item. Decide retained controls versus approved non-equivalent treatment, defaults and system-DPI interaction, supported assets/preview/apply/clear, layout/toolbar reachability, atomic persistence and Java whitelist fields. Do not copy Swing components/resources or arbitrarily switch Next Classic default.
> For retained preset/custom theme switching require preview, Cancel without disk write, restart durability and import not overwriting existing user values. A separate editor is a user goal to resolve, not a mandate for a Swing dialog. Output executable F04–F06 slices and named approvals; no blanket deletion and no all-DPI claim from existing1280×800 narrow chrome evidence.

<a id="r17-gate-f07"></a>
### F07 来源问题（非新任务）

- Owner：R17 / 完整语言清单与翻译 owner（T02-I18N-01）；阶段细化负责人具名落实实际执行人。
- 待决定/取得证据：Approve the complete supported-locale list and translation scope for I18N-01 from A's actual first-consumer resource/fallback result.
- 受阻行为：Complete translations and durable locale selection without partial mixing
- 所需结果：Approved executable contract or bounded evidence-backed individual disposition; no silent new product choice.
- 原事实输入与边界全文：[来源](MIGRATION_CONTRACTS.md#f07)；以下保留该事实/合同输入，不表示已经选定新答案：

> Approve the complete supported-locale list and translation scope for I18N-01 from A's actual first-consumer resource/fallback result.
> Do not infer languages from the Java enum or whichever hardcoded Chinese UI exists. Inventory SET-LANG records OS/SYSTEM baseline and ui.use-language mapping, not a new Next language set. Decide each supported locale, deterministic system/unavailable fallback, complete key/surface ownership, whether switching is live or restart-bound, persisted selection and failed-write/cancel behavior. Unsupported/untranslated locale cannot be advertised as complete support. Output approved locale list, key inventory, completeness criteria and translation responsibility consumed by F08; do not rebuild A foundation or block other independent functionality on complete translation.

<a id="r17-gate-f09"></a>
### F09 来源问题（非新任务）

- Owner：R17 / 逐字段白名单、预览与子集原子迁入 owner（T02-JAVA-IMPORT）；阶段细化负责人具名落实实际执行人。
- 待决定/取得证据：Collect and approve an explicit field-by-field whitelist for Java settings import from completed behavior-owner maps.
- 受阻行为：Read-only Java settings preview, confirmation and atomic import
- 所需结果：Approved executable contract or bounded evidence-backed individual disposition; no silent new product choice.
- 原事实输入与边界全文：[来源](MIGRATION_CONTRACTS.md#f09)；以下保留该事实/合同输入，不表示已经选定新答案：

> Collect and approve an explicit field-by-field whitelist for Java settings import from completed behavior-owner maps.
> Read-only source config fields and legacy work-directory clues are inputs, not automatic directory migration or a permanent never-imported prohibition. Each admitted field names original Java key/type/default/save semantics, Next owner/key/type/value transform/default, capability prerequisite and conflict/unsupported/unknown/secret handling. Preserve current user values, especially preset/custom theme; replacement of an existing value requires explicit previewed user selection. Include declared language, review, engine/match/non-secret network settings only where corresponding behavior is admitted; no automatic credentials/process commands/startup admission by broad JSON merge. Output the exact executable whitelist and preview/conflict policy for F10, or name unapproved owner fields as visibly skipped gaps.
> Collect and admit fields incrementally: an approved owner map and its actual admitted behavior allow that field's executable whitelist entry and F10 import to proceed while unrelated owner decisions remain unresolved. For every unresolved retained field, keep its Java key/user goal, original owner decision and retained implementation obligation in the visible gap record; a skipped field is not a permanent cut. F09 supplies F10 the approved entries plus conflict policy and that gap record. Do not claim complete Java-settings import, PREF-02/full-function completion or final integration until every required retained field has its approved map and actual behavior/import evidence (or an item-specific approved non-equivalent disposition).

## 集成、实际验收与环境义务

- 具名 owner：**R17 功能对等集成/实际验收 owner**；阶段细化时明确实际负责人，拥有本阶段 changed composed workflow 的实际验收，不能交给 read-only Closeout。每个能力owner仍对其局部断言和真实surface负责。
- 环境：全部admitted语言原生UI/keyboard/focus/overflow；只读Java配置与Next persistence/nativeRestart；actual cross-feature engine/helper/service条件仅按changed组合复用，不新增全笛卡尔/平台矩阵。
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
