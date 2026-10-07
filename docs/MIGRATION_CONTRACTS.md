# Migration source behavior contracts

本附录保留 149 份历史来源合同的实质内容：B01–B73、C01–C23、D01–D15、E01–E12、F01–F14 与 R1101–R1112。小写 source ID 是稳定链接；它们是来源身份，不是活跃实施任务。能力的当前阶段与唯一责任以 [公开路由](MIGRATION_ROUTES.json) 和 [来源意图](MIGRATION_SOURCE_MAP.md) 为准，阶段目标、具名待决门与集成出口见 R12–R18 计划。

## Reading and precedence

- 保留各来源的行为/决定问题、已知不变量、required/conditional/field-scoped inputs、原始 defaults/save、失败/Cancel/identity、完整 Source/Delta 条款、最强 Observable acceptance 与真实环境义务。仅履行调查不意味着对应功能已交付。
- 历史 A/B/C/D/E/F 是来源分组；A 对应当前 R11，B 的运行管理与分析分别由 R12/R13 承担，C/D/E/F 对应 R14/R15/R16/R17；历史 R11 正式发行来源由当前 R18 承担。历史 source ID 的引用只消费指定通过结果，不把来源全文或整个阶段变为开工前置。批准后的 scoped-result 规则优先于历史排期。
- 原调查：`qiyi71w/lizzieyzy-next-tauri@55795fab49b80a681a9fa544950e18f5f2da4cc6`；来源细化快照：`15b71e8bd89349961b7cf25f2fd2a6928e991e40`（历史，不是未来产品实施起点）。Java Migration Baseline v1：`7b4027531c2b26062d0bfc27a040cc550cfbea4d`；已冻结审计端点：`af0e07a7386483f3bfc8a15780de72ffc2f0de4c`。最终追加审计从该端点之后开始，不移动 v1 或既有区间。
- 正式阶段启动时按实际完成记录冻结包含全部所消费结果的精确 full SHA；多个未合并结果由阶段串行集成负责人先在隔离候选整合。不得自动改为 latest main、编造未来提交或凭文档目录选择工作树。证据实体必须可取得；相同 HEAD 不证明实体存在。
- 继承验收始终保留 original candidate/platform/engine/helper/service/resource/version、通过 scope 与 Not run。behavior/DTO/protocol/Run/job/document/account/resource/platform/service 改变仅使受影响证据失效。仓库/fixture/browser、native/真实服务、installed/signed release 与上游引擎编译分开，缺条件的门保持 needs-info/Not run。
- 首 UI 本地化消费 [R11 key/locale/fallback](R11_PLAN.md#behavior-a01)，按请求消费 [R11 actual chart output](R11_PLAN.md#behavior-a16)，安装支持只消费 [R11 identity/routing](R11_PLAN.md#behavior-a02) 和 R12 bounded diagnostics，不重建共享 owner。
- 非公开历史证据记录在正文仅以不可点击名称作来源 provenance；行为、判定条件和实际 evidence tuples 随合同保留。记录名称不构成需要本地文件才能读懂的功能合同。每项永久非等价仍需真实的逐项批准。

## Source-to-stage directory

| Historical source | Current capability section |
| --- | --- |
| [B01](#b01) | [R12 — 合格资源、受管维护与显式修复](R12_PLAN.md#resources) |
| [B02](#b02) | [R12 — 合格资源、受管维护与显式修复](R12_PLAN.md#resources) |
| [B03](#b03) | [R12 — 合格资源、受管维护与显式修复](R12_PLAN.md#resources) |
| [B04](#b04) | [R12 — 合格资源、受管维护与显式修复](R12_PLAN.md#resources) |
| [B05](#b05) | [R12 — 合格资源、受管维护与显式修复](R12_PLAN.md#resources) |
| [B06](#b06) | [R12 — 合格资源、受管维护与显式修复](R12_PLAN.md#resources) |
| [B07](#b07) | [R12 — 模型身份与已安装候选保留](R12_PLAN.md#models) |
| [B08](#b08) | [R12 — 模型身份与已安装候选保留](R12_PLAN.md#models) |
| [B09](#b09) | [R12 — 首消费者诊断与响应式控制台](R12_PLAN.md#diagnostics) |
| [B10](#b10) | [R12 — 首消费者诊断与响应式控制台](R12_PLAN.md#diagnostics) |
| [B11](#b11) | [R13 — 颗粒化显示、悬停、自定义评级与可访问候选](R13_PLAN.md#display) |
| [B12](#b12) | [R13 — 颗粒化显示、悬停、自定义评级与可访问候选](R13_PLAN.md#display) |
| [B13](#b13) | [R12 — 启动、默认引擎、预加载与评估让路](R12_PLAN.md#startup) |
| [B14](#b14) | [R12 — 启动、默认引擎、预加载与评估让路](R12_PLAN.md#startup) |
| [B15](#b15) | [R12 — 启动、默认引擎、预加载与评估让路](R12_PLAN.md#startup) |
| [B16](#b16) | [R12 — 启动、默认引擎、预加载与评估让路](R12_PLAN.md#startup) |
| [B17](#b17) | [R13 — 缓存、闪电及自动整局保留能力](R13_PLAN.md#retained-analysis) |
| [B18](#b18) | [R13 — 缓存、闪电及自动整局保留能力](R13_PLAN.md#retained-analysis) |
| [B19](#b19) | [R13 — 缓存、闪电及自动整局保留能力](R13_PLAN.md#retained-analysis) |
| [B20](#b20) | [R13 — 缓存、闪电及自动整局保留能力](R13_PLAN.md#retained-analysis) |
| [B21](#b21) | [R13 — 缓存、闪电及自动整局保留能力](R13_PLAN.md#retained-analysis) |
| [B22](#b22) | [R13 — 缓存、闪电及自动整局保留能力](R13_PLAN.md#retained-analysis) |
| [B23](#b23) | [R13 — Tracking 区间、目标与清除](R13_PLAN.md#tracking) |
| [B24](#b24) | [R13 — Tracking 区间、目标与清除](R13_PLAN.md#tracking) |
| [B25](#b25) | [R13 — Tracking 区间、目标与清除](R13_PLAN.md#tracking) |
| [B26](#b26) | [R13 — 多文件分析队列与安全输出](R13_PLAN.md#batch) |
| [B27](#b27) | [R13 — 多文件分析队列与安全输出](R13_PLAN.md#batch) |
| [B28](#b28) | [R13 — 具名丰富分析适配与缺字段显示](R13_PLAN.md#adapters) |
| [B29](#b29) | [R13 — 具名丰富分析适配与缺字段显示](R13_PLAN.md#adapters) |
| [B30](#b30) | [R13 — 自动载入快析与任务交还](R13_PLAN.md#autoquick) |
| [B31](#b31) | [R13 — 自动载入快析与任务交还](R13_PLAN.md#autoquick) |
| [B32](#b32) | [R13 — 颗粒化显示、悬停、自定义评级与可访问候选](R13_PLAN.md#display) |
| [B33](#b33) | [R13 — 颗粒化显示、悬停、自定义评级与可访问候选](R13_PLAN.md#display) |
| [B34](#b34) | [R13 — 颗粒化显示、悬停、自定义评级与可访问候选](R13_PLAN.md#display) |
| [B35](#b35) | [R13 — 颗粒化显示、悬停、自定义评级与可访问候选](R13_PLAN.md#display) |
| [B36](#b36) | [R13 — 两类引擎续走与唯一落子权威](R13_PLAN.md#continuation) |
| [B37](#b37) | [R13 — 两类引擎续走与唯一落子权威](R13_PLAN.md#continuation) |
| [B38](#b38) | [R13 — 独立 Benchmark、保存建议与实测调优](R13_PLAN.md#performance) |
| [B39](#b39) | [R13 — 独立 Benchmark、保存建议与实测调优](R13_PLAN.md#performance) |
| [B40](#b40) | [R13 — 独立 Benchmark、保存建议与实测调优](R13_PLAN.md#performance) |
| [B41](#b41) | [R13 — 独立 Benchmark、保存建议与实测调优](R13_PLAN.md#performance) |
| [B42](#b42) | [R12 — 手动运行线程来源、临时覆盖与实际值](R12_PLAN.md#manual-threads) |
| [B43](#b43) | [R12 — 手动运行线程来源、临时覆盖与实际值](R12_PLAN.md#manual-threads) |
| [B44](#b44) | [R12 — 手动运行线程来源、临时覆盖与实际值](R12_PLAN.md#manual-threads) |
| [B45](#b45) | [R13 — 独立 Benchmark、保存建议与实测调优](R13_PLAN.md#performance) |
| [B46](#b46) | [R13 — 独立 Benchmark、保存建议与实测调优](R13_PLAN.md#performance) |
| [B47](#b47) | [R12 — 启动、默认引擎、预加载与评估让路](R12_PLAN.md#startup) |
| [B48](#b48) | [R12 — 启动、默认引擎、预加载与评估让路](R12_PLAN.md#startup) |
| [B49](#b49) | [R13 — 同树重点分析与精确局面确认](R13_PLAN.md#focus) |
| [B50](#b50) | [R12 — 运行生命周期、规则确认、恢复与身份隔离](R12_PLAN.md#lifecycle) |
| [B51](#b51) | [R12 — 运行生命周期、规则确认、恢复与身份隔离](R12_PLAN.md#lifecycle) |
| [B52](#b52) | [R12 — 运行生命周期、规则确认、恢复与身份隔离](R12_PLAN.md#lifecycle) |
| [B53](#b53) | [R13 — 自动载入快析与任务交还](R13_PLAN.md#autoquick) |
| [B54](#b54) | [R12 — 运行生命周期、规则确认、恢复与身份隔离](R12_PLAN.md#lifecycle) |
| [B55](#b55) | [R13 — 同树重点分析与精确局面确认](R13_PLAN.md#focus) |
| [B56](#b56) | [R13 — 同树重点分析与精确局面确认](R13_PLAN.md#focus) |
| [B57](#b57) | [R13 — 自动载入快析与任务交还](R13_PLAN.md#autoquick) |
| [B58](#b58) | [R12 — 运行生命周期、规则确认、恢复与身份隔离](R12_PLAN.md#lifecycle) |
| [B59](#b59) | [R12 — 运行生命周期、规则确认、恢复与身份隔离](R12_PLAN.md#lifecycle) |
| [B60](#b60) | [R12 — 运行生命周期、规则确认、恢复与身份隔离](R12_PLAN.md#lifecycle) |
| [B61](#b61) | [R12 — 首消费者诊断与响应式控制台](R12_PLAN.md#diagnostics) |
| [B62](#b62) | [R13 — 自动载入快析与任务交还](R13_PLAN.md#autoquick) |
| [B63](#b63) | [R12 — 运行生命周期、规则确认、恢复与身份隔离](R12_PLAN.md#lifecycle) |
| [B64](#b64) | [R12 — 运行生命周期、规则确认、恢复与身份隔离](R12_PLAN.md#lifecycle) |
| [B65](#b65) | [R12 — 运行生命周期、规则确认、恢复与身份隔离](R12_PLAN.md#lifecycle) |
| [B66](#b66) | [R13 — 颗粒化显示、悬停、自定义评级与可访问候选](R13_PLAN.md#display) |
| [B67](#b67) | [R12 — PDA/WRN 成对只读回读](R12_PLAN.md#runtime-readback) |
| [B68](#b68) | [R12 — PDA/WRN 成对只读回读](R12_PLAN.md#runtime-readback) |
| [B69](#b69) | [R13 — 颗粒化显示、悬停、自定义评级与可访问候选](R13_PLAN.md#display) |
| [B70](#b70) | [R13 — 颗粒化显示、悬停、自定义评级与可访问候选](R13_PLAN.md#display) |
| [B71](#b71) | [R13 — 颗粒化显示、悬停、自定义评级与可访问候选](R13_PLAN.md#display) |
| [B72](#b72) | [R12 — 运行生命周期、规则确认、恢复与身份隔离](R12_PLAN.md#lifecycle) |
| [B73](#b73) | [R12 — 合格资源、受管维护与显式修复](R12_PLAN.md#resources) |
| [C01](#c01) | [R14 — SSH 全部已准入 stdio 与 Autoload](R14_PLAN.md#ssh) |
| [C02](#c02) | [R14 — SSH 全部已准入 stdio 与 Autoload](R14_PLAN.md#ssh) |
| [C03](#c03) | [R14 — Zhizi/Custom 远程算力与统一网络](R14_PLAN.md#compute-network) |
| [C04](#c04) | [R14 — Zhizi/Custom 远程算力与统一网络](R14_PLAN.md#compute-network) |
| [C05](#c05) | [R14 — readboard 只读生命周期、重点分析与外部恢复](R14_PLAN.md#readboard) |
| [C06](#c06) | [R14 — 既有服务证据与狐狸身份边界](R14_PLAN.md#providers) |
| [C07](#c07) | [R14 — 腾讯直播](R14_PLAN.md#tencent) |
| [C08](#c08) | [R14 — 腾讯直播](R14_PLAN.md#tencent) |
| [C09](#c09) | [R14 — 弈客 Personal 与受支持原生账号对弈](R14_PLAN.md#yike) |
| [C10](#c10) | [R14 — 弈客 Personal 与受支持原生账号对弈](R14_PLAN.md#yike) |
| [C11](#c11) | [R14 — 弈客 Personal 与受支持原生账号对弈](R14_PLAN.md#yike) |
| [C12](#c12) | [R14 — 弈客 Personal 与受支持原生账号对弈](R14_PLAN.md#yike) |
| [C13](#c13) | [R14 — 弈客 Personal 与受支持原生账号对弈](R14_PLAN.md#yike) |
| [C14](#c14) | [R14 — 弈客 Personal 与受支持原生账号对弈](R14_PLAN.md#yike) |
| [C15](#c15) | [R14 — GAME-10 双模式精确权威、Stop 与续走](R14_PLAN.md#external-match) |
| [C16](#c16) | [R14 — GAME-10 双模式精确权威、Stop 与续走](R14_PLAN.md#external-match) |
| [C17](#c17) | [R14 — readboard 只读生命周期、重点分析与外部恢复](R14_PLAN.md#readboard) |
| [C18](#c18) | [R14 — readboard 只读生命周期、重点分析与外部恢复](R14_PLAN.md#readboard) |
| [C19](#c19) | [R14 — readboard 只读生命周期、重点分析与外部恢复](R14_PLAN.md#readboard) |
| [C20](#c20) | [R14 — GAME-10 双模式精确权威、Stop 与续走](R14_PLAN.md#external-match) |
| [C21](#c21) | [R14 — GAME-10 双模式精确权威、Stop 与续走](R14_PLAN.md#external-match) |
| [C22](#c22) | [R14 — 两服务显式和自动快析交还](R14_PLAN.md#remote-handback) |
| [C23](#c23) | [R14 — 既有服务证据与狐狸身份边界](R14_PLAN.md#providers) |
| [D01](#d01) | [R15 — 批量 PK、开局库、换色和手动干预](R15_PLAN.md#pk) |
| [D02](#d02) | [R15 — 保留 Human 控件、竞赛时钟与认输](R15_PLAN.md#human-clock) |
| [D03](#d03) | [R15 — 保留 Human 控件、竞赛时钟与认输](R15_PLAN.md#human-clock) |
| [D04](#d04) | [R15 — 保留 Human 控件、竞赛时钟与认输](R15_PLAN.md#human-clock) |
| [D05](#d05) | [R15 — 保留 Human 控件、竞赛时钟与认输](R15_PLAN.md#human-clock) |
| [D06](#d06) | [R15 — 兼容 HumanSL 资源与完整 Coach](R15_PLAN.md#humansl) |
| [D07](#d07) | [R15 — 兼容 HumanSL 资源与完整 Coach](R15_PLAN.md#humansl) |
| [D08](#d08) | [R15 — 兼容 HumanSL 资源与完整 Coach](R15_PLAN.md#humansl) |
| [D09](#d09) | [R15 — 五规则族、自定义、古中国及运行规则消费](R15_PLAN.md#rules) |
| [D10](#d10) | [R15 — 五规则族、自定义、古中国及运行规则消费](R15_PLAN.md#rules) |
| [D11](#d11) | [R15 — 五规则族、自定义、古中国及运行规则消费](R15_PLAN.md#rules) |
| [D12](#d12) | [R15 — 同一不可变规则快照保存分享](R15_PLAN.md#rule-storage) |
| [D13](#d13) | [R15 — 同一不可变规则快照保存分享](R15_PLAN.md#rule-storage) |
| [D14](#d14) | [R15 — 当前 Match 双参与者运行贴目](R15_PLAN.md#komi) |
| [D15](#d15) | [R15 — 当前 Match 双参与者运行贴目](R15_PLAN.md#komi) |
| [E01](#e01) | [R16 — 真实贡献上传、consent 与历史保留能力](R16_PLAN.md#contribution) |
| [E02](#e02) | [R16 — 真实贡献上传、consent 与历史保留能力](R16_PLAN.md#contribution) |
| [E03](#e03) | [R16 — 真实贡献上传、consent 与历史保留能力](R16_PLAN.md#contribution) |
| [E04](#e04) | [R16 — 贡献棋盘观看与精确交还](R16_PLAN.md#watch) |
| [E05](#e05) | [R16 — 局域网棋盘发布](R16_PLAN.md#lan) |
| [E06](#e06) | [R16 — 具名教育生产者、永久关闭与重置](R16_PLAN.md#guidance) |
| [E07](#e07) | [R16 — 具名教育生产者、永久关闭与重置](R16_PLAN.md#guidance) |
| [E08](#e08) | [R16 — 生成信息与个人评论分离的历史闭环](R16_PLAN.md#generated) |
| [E09](#e09) | [R16 — 真实 AI 解说服务、凭据、偏好与 grounded 工作区](R16_PLAN.md#ai) |
| [E10](#e10) | [R16 — 真实 AI 解说服务、凭据、偏好与 grounded 工作区](R16_PLAN.md#ai) |
| [E11](#e11) | [R16 — 真实 AI 解说服务、凭据、偏好与 grounded 工作区](R16_PLAN.md#ai) |
| [E12](#e12) | [R16 — 真实 AI 解说服务、凭据、偏好与 grounded 工作区](R16_PLAN.md#ai) |
| [F01](#f01) | [R17 — 保留初始化和主机变更用户目标](R17_PLAN.md#initialization) |
| [F02](#f02) | [R17 — 保留初始化和主机变更用户目标](R17_PLAN.md#initialization) |
| [F03](#f03) | [R17 — 字体、主题编辑与布局工具栏预设](R17_PLAN.md#appearance) |
| [F04](#f04) | [R17 — 字体、主题编辑与布局工具栏预设](R17_PLAN.md#appearance) |
| [F05](#f05) | [R17 — 字体、主题编辑与布局工具栏预设](R17_PLAN.md#appearance) |
| [F06](#f06) | [R17 — 字体、主题编辑与布局工具栏预设](R17_PLAN.md#appearance) |
| [F07](#f07) | [R17 — 完整语言清单与翻译](R17_PLAN.md#localization) |
| [F08](#f08) | [R17 — 完整语言清单与翻译](R17_PLAN.md#localization) |
| [F09](#f09) | [R17 — 逐字段白名单、预览与子集原子迁入](R17_PLAN.md#import) |
| [F10](#f10) | [R17 — 逐字段白名单、预览与子集原子迁入](R17_PLAN.md#import) |
| [F11](#f11) | [R17 — UI-02 与原生工作台可访问性](R17_PLAN.md#accessibility) |
| [F12](#f12) | [R17 — 139 heading/20 route 入口默认保存闭环](R17_PLAN.md#entries) |
| [F13](#f13) | [R17 — 跨功能组合验收](R17_PLAN.md#integration) |
| [F14](#f14) | [R17 — 冻结端点之后全量上游复核](R17_PLAN.md#upstream) |
| [R1101](#r1101) | [R18 — 正式仓库、渠道、签名与组件目标决定](R18_PLAN.md#channel) |
| [R1102](#r1102) | [R18 — 正式仓库、渠道、签名与组件目标决定](R18_PLAN.md#channel) |
| [R1103](#r1103) | [R18 — 精确候选 Preflight 与正式产物信任](R18_PLAN.md#preflight-trust) |
| [R1104](#r1104) | [R18 — 精确候选 Preflight 与正式产物信任](R18_PLAN.md#preflight-trust) |
| [R1105](#r1105) | [R18 — 手动更新发现与签名组件获取](R18_PLAN.md#discovery-components) |
| [R1106](#r1106) | [R18 — 手动更新发现与签名组件获取](R18_PLAN.md#discovery-components) |
| [R1107](#r1107) | [R18 — Windows 更新应用与反向回滚](R18_PLAN.md#windows-update) |
| [R1108](#r1108) | [R18 — Windows 更新应用与反向回滚](R18_PLAN.md#windows-update) |
| [R1109](#r1109) | [R18 — macOS/Linux 已验证包交接](R18_PLAN.md#handoff) |
| [R1110](#r1110) | [R18 — Canonical Artifact 安装便携路径与卸载](R18_PLAN.md#installed) |
| [R1111](#r1111) | [R18 — 安装态 SGF/GIB 文件激活](R18_PLAN.md#activation) |
| [R1112](#r1112) | [R18 — 安装态支持日志与身份路由](R18_PLAN.md#support) |


## Full contract references

<a id="b01"></a>
### B01 — Qualified local runtime resources at Start and Switch

[完整来源契约](migration-contracts/R12.md#b01)。

<a id="b02"></a>
### B02 — Freeze managed catalog and explicit resource-maintenance admission

[完整来源契约](migration-contracts/R12.md#b02)。

<a id="b03"></a>
### B03 — Managed resource maintenance and version-qualified GPU guidance

[完整来源契约](migration-contracts/R12.md#b03)。

<a id="b04"></a>
### B04 — Determine ownership of target-directed TensorRT repair

[完整来源契约](migration-contracts/R12.md#b04)。

<a id="b05"></a>
### B05 — Bound NVIDIA hardware qualification for admitted repair targets

[完整来源契约](migration-contracts/R12.md#b05)。

<a id="b06"></a>
### B06 — Reachable acceleration setup actions and status layout

[完整来源契约](migration-contracts/R12.md#b06)。

<a id="b07"></a>
### B07 — Map model headers, ownership and installed retention

[完整来源契约](migration-contracts/R12.md#b07)。

<a id="b08"></a>
### B08 — Model identity and installed-candidate retention

[完整来源契约](migration-contracts/R12.md#b08)。

<a id="b09"></a>
### B09 — Define bounded first-consumer diagnostics and export budgets

[完整来源契约](migration-contracts/R12.md#b09)。

<a id="b10"></a>
### B10 — First-consumer bounded runtime diagnostics and pinned export

[完整来源契约](migration-contracts/R12.md#b10)。

<a id="b11"></a>
### B11 — Decide hover delay and manual reveal preservation

[完整来源契约](migration-contracts/R13.md#b11)。

<a id="b12"></a>
### B12 — Retained hover delay and manual reveal successor

[完整来源契约](migration-contracts/R13.md#b12)。

<a id="b13"></a>
### B13 — Decide last-primary-engine startup policy preservation

[完整来源契约](migration-contracts/R12.md#b13)。

<a id="b14"></a>
### B14 — Retained last-primary-engine startup policy successor

[完整来源契约](migration-contracts/R12.md#b14)。

<a id="b15"></a>
### B15 — Decide background extra-engine preload preservation

[完整来源契约](migration-contracts/R12.md#b15)。

<a id="b16"></a>
### B16 — Retained background extra-engine preload successor

[完整来源契约](migration-contracts/R12.md#b16)。

<a id="b17"></a>
### B17 — Decide user-controlled in-tree result reuse preservation

[完整来源契约](migration-contracts/R13.md#b17)。

<a id="b18"></a>
### B18 — Retained user-controlled in-tree result reuse successor

[完整来源契约](migration-contracts/R13.md#b18)。

<a id="b19"></a>
### B19 — Decide lightning, part and all-branches retained flows preservation

[完整来源契约](migration-contracts/R13.md#b19)。

<a id="b20"></a>
### B20 — Retained lightning, part and all-branches retained flows successor

[完整来源契约](migration-contracts/R13.md#b20)。

<a id="b21"></a>
### B21 — Decide automatic current-game retained controls preservation

[完整来源契约](migration-contracts/R13.md#b21)。

<a id="b22"></a>
### B22 — Retained automatic current-game retained controls successor

[完整来源契约](migration-contracts/R13.md#b22)。

<a id="b23"></a>
### B23 — Decide tracking job historical preservation preservation

[完整来源契约](migration-contracts/R13.md#b23)。

<a id="b24"></a>
### B24 — Freeze tracking Run/job ownership and interval compatibility

[完整来源契约](migration-contracts/R13.md#b24)。

<a id="b25"></a>
### B25 — Tracking points and clear-to-ordinary analysis

[完整来源契约](migration-contracts/R13.md#b25)。

<a id="b26"></a>
### B26 — Freeze batch intake, analysis conditions and safe file output

[完整来源契约](migration-contracts/R13.md#b26)。

<a id="b27"></a>
### B27 — Session-only ordered SGF batch analysis

[完整来源契约](migration-contracts/R13.md#b27)。

<a id="b28"></a>
### B28 — Admit named engine/version rich-analysis adapters

[完整来源契约](migration-contracts/R13.md#b28)。

<a id="b29"></a>
### B29 — Named rich-analysis adapters and absent-field handling

[完整来源契约](migration-contracts/R13.md#b29)。

<a id="b30"></a>
### B30 — Freeze automatic-on-load quick-analysis admission and saved defaults

[完整来源契约](migration-contracts/R13.md#b30)。

<a id="b31"></a>
### B31 — Automatic load quick analysis and identity-safe handback

[完整来源契约](migration-contracts/R13.md#b31)。

<a id="b32"></a>
### B32 — Map retained granular candidate and territory display fields

[完整来源契约](migration-contracts/R13.md#b32)。

<a id="b33"></a>
### B33 — Granular candidate and territory presentation

[完整来源契约](migration-contracts/R13.md#b33)。

<a id="b34"></a>
### B34 — Locate and decide custom review-grade thresholds

[完整来源契约](migration-contracts/R13.md#b34)。

<a id="b35"></a>
### B35 — Custom grade threshold settings and review consumers

[完整来源契约](migration-contracts/R13.md#b35)。

<a id="b36"></a>
### B36 — Decide leaf-only and every-beat Engine Continuation authority

[完整来源契约](migration-contracts/R13.md#b36)。

<a id="b37"></a>
### B37 — Approved Engine Continuation move creation

[完整来源契约](migration-contracts/R13.md#b37)。

<a id="b38"></a>
### B38 — Isolated saved-profile custom benchmark runner

[完整来源契约](migration-contracts/R13.md#b38)。

<a id="b39"></a>
### B39 — Distinct NN and search-speed benchmark results

[完整来源契约](migration-contracts/R13.md#b39)。

<a id="b40"></a>
### B40 — Accessible benchmark outcomes and recommendation distinction

[完整来源契约](migration-contracts/R13.md#b40)。

<a id="b41"></a>
### B41 — Saved-entry CFG/BENCHMARK recommendation policy and launch integration

[完整来源契约](migration-contracts/R13.md#b41)。

<a id="b42"></a>
### B42 — Freeze manual runtime thread source, effective and temporary scope

[完整来源契约](migration-contracts/R12.md#b42)。

<a id="b43"></a>
### B43 — Manual dynamic thread Apply and confirmed readback

[完整来源契约](migration-contracts/R12.md#b43)。

<a id="b44"></a>
### B44 — Determine managed config/include thread-alias applicability

[完整来源契约](migration-contracts/R12.md#b44)。

<a id="b45"></a>
### B45 — Design compatible measured-scene overlays and report admission

[完整来源契约](migration-contracts/R13.md#b45)。

<a id="b46"></a>
### B46 — Measured report Import, Review, Apply and Restore

[完整来源契约](migration-contracts/R13.md#b46)。

<a id="b47"></a>
### B47 — Decide startup performance evaluation entry and foreground priority

[完整来源契约](migration-contracts/R12.md#b47)。

<a id="b48"></a>
### B48 — Approved startup evaluation and non-disruptive yield

[完整来源契约](migration-contracts/R12.md#b48)。

<a id="b49"></a>
### B49 — Local same-tree move focus, ordinary cache and display

[完整来源契约](migration-contracts/R13.md#b49)。

<a id="b50"></a>
### B50 — Ordinary engine rules confirmation and immutable Match consumer result

[完整来源契约](migration-contracts/R12.md#b50)。

<a id="b51"></a>
### B51 — Bound Windows lifecycle UI/automatic-task applicability

[完整来源契约](migration-contracts/R12.md#b51)。

<a id="b52"></a>
### B52 — Bound lifecycle owner-lock and callback deadlock applicability

[完整来源契约](migration-contracts/R12.md#b52)。

<a id="b53"></a>
### B53 — Bound whole-game budget terminal-to-ordinary ownership

[完整来源契约](migration-contracts/R13.md#b53)。

<a id="b54"></a>
### B54 — Bound Match streaming genmove retirement and terminal fences

[完整来源契约](migration-contracts/R12.md#b54)。

<a id="b55"></a>
### B55 — Bound ordinary analysis target/reader/slot confirmation

[完整来源契约](migration-contracts/R13.md#b55)。

<a id="b56"></a>
### B56 — Bound SGF RU/KM import-to-analysis confirmation

[完整来源契约](migration-contracts/R13.md#b56)。

<a id="b57"></a>
### B57 — Bound existing response framing and handback pause ownership

[完整来源契约](migration-contracts/R13.md#b57)。

<a id="b58"></a>
### B58 — Bound exact restore path, quoting and incarnation admission

[完整来源契约](migration-contracts/R12.md#b58)。

<a id="b59"></a>
### B59 — Bound target Cancel versus explicit Run Stop/exit

[完整来源契约](migration-contracts/R12.md#b59)。

<a id="b60"></a>
### B60 — Bound restart reader/ACK/final authority transfer

[完整来源契约](migration-contracts/R12.md#b60)。

<a id="b61"></a>
### B61 — Responsive bounded console output and load status

[完整来源契约](migration-contracts/R12.md#b61)。

<a id="b62"></a>
### B62 — Bound current-game task completion/Cancel/Pause and close/reopen handback

[完整来源契约](migration-contracts/R13.md#b62)。

<a id="b63"></a>
### B63 — Bound failed/timeout position synchronization confirmation

[完整来源契约](migration-contracts/R12.md#b63)。

<a id="b64"></a>
### B64 — Bound failed-Switch deferred reader fence and same-A recovery

[完整来源契约](migration-contracts/R12.md#b64)。

<a id="b65"></a>
### B65 — Verify supported local SGF/node exact position restoration

[完整来源契约](migration-contracts/R12.md#b65)。

<a id="b66"></a>
### B66 — Verify current review-tree display and click-hit identity

[完整来源契约](migration-contracts/R13.md#b66)。

<a id="b67"></a>
### B67 — Admit paired PDA/WRN current-Run readback capability

[完整来源契约](migration-contracts/R12.md#b67)。

<a id="b68"></a>
### B68 — Paired PDA/WRN current-Run readback

[完整来源契约](migration-contracts/R12.md#b68)。

<a id="b69"></a>
### B69 — Verify preview input ownership and decide any navigation difference

[完整来源契约](migration-contracts/R13.md#b69)。

<a id="b70"></a>
### B70 — Accessible candidate-list navigation during live updates

[完整来源契约](migration-contracts/R13.md#b70)。

<a id="b71"></a>
### B71 — Verify bounded preview calculation/publication and replay retirement

[完整来源契约](migration-contracts/R13.md#b71)。

<a id="b72"></a>
### B72 — Verify local failed-handback→import→explicit Restart recovery

[完整来源契约](migration-contracts/R12.md#b72)。

<a id="b73"></a>
### B73 — Approved explicit target-directed TensorRT resource repair

[完整来源契约](migration-contracts/R12.md#b73)。

<a id="c01"></a>
### C01 — Freeze SSH adapter, trust and cancellable deadline admission

[完整来源契约](migration-contracts/R14.md#c01)。

<a id="c02"></a>
### C02 — Implement SSH Engine Profiles across admitted stdio adapters

[完整来源契约](migration-contracts/R14.md#c02)。

<a id="c03"></a>
### C03 — Establish both remote-compute service and deadline contracts

[完整来源契约](migration-contracts/R14.md#c03)。

<a id="c04"></a>
### C04 — Deliver both remote-compute modes with actual policy routes

[完整来源契约](migration-contracts/R14.md#c04)。

<a id="c05"></a>
### C05 — Integrate existing readboard functional and retirement evidence

[完整来源契约](migration-contracts/R14.md#c05)。

<a id="c06"></a>
### C06 — Record reusable public provider import and sync outcomes

[完整来源契约](migration-contracts/R14.md#c06)。

<a id="c07"></a>
### C07 — Decide supported Tencent/huanle live read protocol

[完整来源契约](migration-contracts/R14.md#c07)。

<a id="c08"></a>
### C08 — Implement admitted Tencent/huanle ongoing synchronization

[完整来源契约](migration-contracts/R14.md#c08)。

<a id="c09"></a>
### C09 — Establish real guest Yike Personal semantics

[完整来源契约](migration-contracts/R14.md#c09)。

<a id="c10"></a>
### C10 — Deliver guest Personal discovery and public handoff

[完整来源契约](migration-contracts/R14.md#c10)。

<a id="c11"></a>
### C11 — Freeze provider-supported Yike account read/write admission

[完整来源契约](migration-contracts/R14.md#c11)。

<a id="c12"></a>
### C12 — Implement one authorized Yike account and authoritative human play

[完整来源契约](migration-contracts/R14.md#c12)。

<a id="c13"></a>
### C13 — Resolve retained embedded Yike page and hall user goals

[完整来源契约](migration-contracts/R14.md#c13)。

<a id="c14"></a>
### C14 — Resolve registered no-op share shortcuts without empty actions

[完整来源契约](migration-contracts/R14.md#c14)。

<a id="c15"></a>
### C15 — Freeze readboard bidirectional capabilities and confirmation deadlines

[完整来源契约](migration-contracts/R14.md#c15)。

<a id="c16"></a>
### C16 — Implement both externally confirmed engine move modes

[完整来源契约](migration-contracts/R14.md#c16)。

<a id="c17"></a>
### C17 — Extend passed same-tree focus to readboard evidence lifecycle

[完整来源契约](migration-contracts/R14.md#c17)。

<a id="c18"></a>
### C18 — Check late Fox title and frame ordering without speculative fixes

[完整来源契约](migration-contracts/R14.md#c18)。

<a id="c19"></a>
### C19 — Check exact readboard handicap and rebuild Save/reopen semantics

[完整来源契约](migration-contracts/R14.md#c19)。

<a id="c20"></a>
### C20 — Preserve only current authorized external Engine Continuation intent

[完整来源契约](migration-contracts/R14.md#c20)。

<a id="c21"></a>
### C21 — Fence actual GAME-10 mode-switch and navigation results

[完整来源契约](migration-contracts/R14.md#c21)。

<a id="c22"></a>
### C22 — Validate required remote automatic-quick and explicit-analysis handback

[完整来源契约](migration-contracts/R14.md#c22)。

<a id="c23"></a>
### C23 — Check Fox numeric nickname, explicit UID and keyboard identity

[完整来源契约](migration-contracts/R14.md#c23)。

<a id="d01"></a>
### D01 — Session-only PK batches, opening catalog and intervention

[完整来源契约](migration-contracts/R15.md#d01)。

<a id="d02"></a>
### D02 — Decide retained legacy Human-vs-Engine control goals

[完整来源契约](migration-contracts/R15.md#d02)。

<a id="d03"></a>
### D03 — Implement approved retained Human match controls

[完整来源契约](migration-contracts/R15.md#d03)。

<a id="d04"></a>
### D04 — Approve competitive clock and auto-resign policy

[完整来源契约](migration-contracts/R15.md#d04)。

<a id="d05"></a>
### D05 — Competitive clocks and evidence-based auto-resign

[完整来源契约](migration-contracts/R15.md#d05)。

<a id="d06"></a>
### D06 — Freeze compatible HumanSL profile and Coach setup contract

[完整来源契约](migration-contracts/R15.md#d06)。

<a id="d07"></a>
### D07 — Acquire qualified HumanSL resources without changing prior set

[完整来源契约](migration-contracts/R15.md#d07)。

<a id="d08"></a>
### D08 — Independent HumanSL Coach with tactical verification and exact handback

[完整来源契约](migration-contracts/R15.md#d08)。

<a id="d09"></a>
### D09 — Freeze five-family/custom and ancient rule adapter mapping

[完整来源契约](migration-contracts/R15.md#d09)。

<a id="d10"></a>
### D10 — Preserving rule-family/custom selection and ancient-rule play

[完整来源契约](migration-contracts/R15.md#d10)。

<a id="d11"></a>
### D11 — Consume confirmed rules for immutable Match admission and exact restoration

[完整来源契约](migration-contracts/R15.md#d11)。

<a id="d12"></a>
### D12 — Approve immutable Match-rule generated details storage/share

[完整来源契约](migration-contracts/R15.md#d12)。

<a id="d13"></a>
### D13 — Save/share approved immutable Match-rule details

[完整来源契约](migration-contracts/R15.md#d13)。

<a id="d14"></a>
### D14 — Freeze current-Match runtime-komi entry and transaction interface

[完整来源契约](migration-contracts/R15.md#d14)。

<a id="d15"></a>
### D15 — Atomic two-participant runtime-komi updates

[完整来源契约](migration-contracts/R15.md#d15)。

<a id="e01"></a>
### E01 — Resolve each historical contribution/watch reduction

[完整来源契约](migration-contracts/R16.md#e01)。

<a id="e02"></a>
### E02 — Official Contribution Run: consent, credentials, lifecycle, upload and auto-save

[完整来源契约](migration-contracts/R16.md#e02)。

<a id="e03"></a>
### E03 — Implement individually admitted contribution/watch residual goals

[完整来源契约](migration-contracts/R16.md#e03)。

<a id="e04"></a>
### E04 — Optional transient Contribution board watcher with exact restoration

[完整来源契约](migration-contracts/R16.md#e04)。

<a id="e05"></a>
### E05 — LAN board publishing with bounded slow-client state

[完整来源契约](migration-contracts/R16.md#e05)。

<a id="e06"></a>
### E06 — Decide named Educational Tip producer and historical hint goals

[完整来源契约](migration-contracts/R16.md#e06)。

<a id="e07"></a>
### E07 — Named Educational Tips with durable dismissal and dedicated reset

[完整来源契约](migration-contracts/R16.md#e07)。

<a id="e08"></a>
### E08 — Reconcile generated-stat disposition without losing personal comments

[完整来源契约](migration-contracts/R16.md#e08)。

<a id="e09"></a>
### E09 — Approve AI service/auth, model and funding/grounding contract

[完整来源契约](migration-contracts/R16.md#e09)。

<a id="e10"></a>
### E10 — Official AI login/API-key connection and credential lifecycle

[完整来源契约](migration-contracts/R16.md#e10)。

<a id="e11"></a>
### E11 — Separate AI connection and commentary preferences with real model catalog

[完整来源契约](migration-contracts/R16.md#e11)。

<a id="e12"></a>
### E12 — Grounded AI commentary workspace with explicit charge consent and freshness

[完整来源契约](migration-contracts/R16.md#e12)。

<a id="f01"></a>
### F01 — Decide hostname-reset and initialization user goals individually

[完整来源契约](migration-contracts/R17.md#f01)。

<a id="f02"></a>
### F02 — Approved retained initialization and host-change behaviors

[完整来源契约](migration-contracts/R17.md#f02)。

<a id="f03"></a>
### F03 — Approve retained font, theme and layout goals with item-specific dispositions

[完整来源契约](migration-contracts/R17.md#f03)。

<a id="f04"></a>
### F04 — Approved application font/accessibility controls

[完整来源契约](migration-contracts/R17.md#f04)。

<a id="f05"></a>
### F05 — Approved theme presets, custom assets and editing with safe preview

[完整来源契约](migration-contracts/R17.md#f05)。

<a id="f06"></a>
### F06 — Approved workspace/toolbar presets and large-panel goals

[完整来源契约](migration-contracts/R17.md#f06)。

<a id="f07"></a>
### F07 — Approve full supported locale and translation/fallback scope

[完整来源契约](migration-contracts/R17.md#f07)。

<a id="f08"></a>
### F08 — Complete translations and durable locale selection without partial mixing

[完整来源契约](migration-contracts/R17.md#f08)。

<a id="f09"></a>
### F09 — Freeze per-owner Java settings whitelist and conflict semantics

[完整来源契约](migration-contracts/R17.md#f09)。

<a id="f10"></a>
### F10 — Read-only Java settings preview, confirmation and atomic import

[完整来源契约](migration-contracts/R17.md#f10)。

<a id="f11"></a>
### F11 — Finish non-blocking board intent and workbench accessibility residuals

[完整来源契约](migration-contracts/R17.md#f11)。

<a id="f12"></a>
### F12 — Close all139 frozen headings and20 routes with entry/default/save dispositions

[完整来源契约](migration-contracts/R17.md#f12)。

<a id="f13"></a>
### F13 — Integrate A–F functional workflows with original evidence and terminal gates

[完整来源契约](migration-contracts/R17.md#f13)。

<a id="f14"></a>
### F14 — Append final frozen upstream interval and resolve remaining user-visible deltas

[完整来源契约](migration-contracts/R17.md#f14)。

<a id="r1101"></a>
### R1101 — Approve final release repository, signing/update/support configuration

[完整来源契约](migration-contracts/R18.md#r1101)。

<a id="r1102"></a>
### R1102 — Resolve legacy flavor/JRE/JCEF component user goals

[完整来源契约](migration-contracts/R18.md#r1102)。

<a id="r1103"></a>
### R1103 — Exact-commit non-mutating release preflight and dry-run

[完整来源契约](migration-contracts/R18.md#r1103)。

<a id="r1104"></a>
### R1104 — Trusted canonical production artifacts and feed exclusion

[完整来源契约](migration-contracts/R18.md#r1104)。

<a id="r1105"></a>
### R1105 — Manual stable/beta signed update discovery without install mutation

[完整来源契约](migration-contracts/R18.md#r1105)。

<a id="r1106"></a>
### R1106 — Signed installed manifest and explicit component acquisition residual

[完整来源契约](migration-contracts/R18.md#r1106)。

<a id="r1107"></a>
### R1107 — Verified Windows component download/helper apply without data changes

[完整来源契约](migration-contracts/R18.md#r1107)。

<a id="r1108"></a>
### R1108 — Windows update reverse rollback, durable result journal and repair

[完整来源契约](migration-contracts/R18.md#r1108)。

<a id="r1109"></a>
### R1109 — Verified macOS/Linux package acquisition and non-overwriting handoff

[完整来源契约](migration-contracts/R18.md#r1109)。

<a id="r1110"></a>
### R1110 — Per-artifact installed/unpacked lifecycle, trust, data and associations

[完整来源契约](migration-contracts/R18.md#r1110)。

<a id="r1111"></a>
### R1111 — Finish APP-01 installed association activation without rewriting native scope

[完整来源契约](migration-contracts/R18.md#r1111)。

<a id="r1112"></a>
### R1112 — Finish installed identity/support routing and bounded support-bundle evidence

[完整来源契约](migration-contracts/R18.md#r1112)。
