# R11 — 常用复盘与操作迁移计划

Status: 已完成批准范围的实现、共享状态集成、统一审查与只读 Closeout；经 [PR #20](https://github.com/qiyi71w/lizzieyzy-next-tauri/pull/20) 合入 `9d2ccf3ba6881c755013b82948e5da1a01f7fcd4`。本文保留批准合同；当前条目状态以 [Parity Matrix](PARITY_MATRIX.md) 为准，原候选、后续修复和证据限制见 [R11 完成证据](DEVELOPMENT.md#r11-completion-evidence)，不声称本次文档回填产生新的运行通过结论。

## 1. 目标与基线

R11 将常用复盘与棋谱操作组织为七个完整能力组，配套一次实际共享状态集成验收和单独只读 Closeout。目标是从真实入口完成用户工作流，保留精确棋谱语义、非破坏失败和来源限定的证据责任。发行、安装态信任和更新由 [R18](R18_PLAN.md) 承担；R12–R18 按实际前置滚动细化，阶段编号表示交付顺序，不构成整阶段硬依赖。

- 仓库：`qiyi71w/lizzieyzy-next-tauri`。
- 固定调查基线：`55795fab49b80a681a9fa544950e18f5f2da4cc6`。
- Java Migration Baseline v1：`7b4027531c2b26062d0bfc27a040cc550cfbea4d`；冻结增量终点：`af0e07a7386483f3bfc8a15780de72ffc2f0de4c`。后续上游审计由 R17 追加冻结区间，不移动 v1。
- 权威边界：[Architecture](ARCHITECTURE_NEXT.md)、[Migration Plan](MIGRATION_PLAN.md)、[Parity Matrix](PARITY_MATRIX.md)、[上游 Delta](UPSTREAM_DELTA.md) 的原条目、Accepted 窄范围和候选证据保持不变。
- 本阶段对应历史 A 来源中的 31 个唯一责任意图、35 个来源意图、30 个 `behavior-aXX` 行为；数量是覆盖索引，不是新功能配额。详见 [公开责任路线](MIGRATION_ROUTES.json) 和 [完整来源图](MIGRATION_SOURCE_MAP.md)。
- 原产品开工须另获授权并消费最终规划成果，该门已经由 R11 正式执行满足；不再把本阶段列为待开工。后续改动仍须选择包含所需成果的完整提交，并核对实际已验证、已集成的证据材料；历史规划候选、共同旧 HEAD 或阶段完成标签不替代该检查。

### 完成记录与保留边界

- R11-01–08 在 `4fe5e6a3aa1383d7e5c643f2339f5d105317a8f2` 完成批准范围、统一 Spec/Standards 审查及修复；R11-09 只读 Closeout 核对 90 条原验收条款。原生验收分别绑定原功能候选、`b4492f1a1f22c494264d84c134c5e7d9b501b8ad` 的 17 个集成场景和最终修复后的 6 个受影响场景，不是全部场景无条件 PASS。
- PR 后续 `5c2fc5b5f5ed0c99154a3e9d1ed1f8cceb2fc5e3` 修复 `RU[cn]` 并取得真实 Windows/KataGo 证据；最终 PR head `03c8180115c083bfb31110930a846ff13c46f9ef` 的 Add/图表/Unix 保存修复另有受影响验证，不沿用旧正文“最终仅文档差异”的说法。详细版本关系与各证据类见 DEVELOPMENT。
- `EXPORT-03` 的 Windows 功能证据可供后续 owner 按范围消费；整项仍为 Partial，保留每个 Shipped Platform 原生准入强门。R11 记录时准入平台集合为空；这不否定已完成的批准 R11 范围，也不产生安装态/全平台通过结论。
- SPEC-F02 公共导入差异、SPEC-F03 图表 drag/click 差异及 SPEC-F04 真实音频设备失败仍是具名后续；有界调查完成不等于修复或未来能力验收。R17 全量语言/入口闭环、R18 安装/发行义务和原 Accepted 窄范围不变。
- 下文的“Parity IDs（不改原状态）”及“尚无/仍须取得证据”等表述保留原规划时点和验收要求，不作为当前状态公告；当前 disposition 只由 Matrix 维护。行为、Acceptance、scoped results、历史决定、依赖和原生强门未在本次回填中改写。

## 2. 共同合同与结果归属

Rust 是棋谱/会话权威，领域逻辑留在 crates，Tauri gateway 和前端 API wrappers 不复制领域 owner。单引擎生命周期、无隐式 switch，generation/NodePath/Run/job 身份隔离，Cancel/失败非破坏。五项既有 ADR、个人 C 与生成信息分离、统一网络与凭据保护保持不变；改变受影响契约时同步文档和 changelog。

每个设置 owner 随能力交付 Java 字段、值转换、默认、失败持久化及重启读回映射。R17 消费这些映射完成白名单、完整翻译和 139 入口/20 路由闭环，不建立第二套语义 owner。历史 read、常驻强退、raw、manual slots、第二 exit-save、忽略文件 KM、副棋盘图像各自已决处置保留完整非等价理由；相近功能不构成原目标等价验收。新的实际范围变化由对应 owner 另行取得决定。

### Scoped 结果消费

| 唯一 provider | 具名结果 | 实际消费边界与释放条件 |
| --- | --- | --- |
| R11-01 / [搜索首消费者](#behavior-a01) | `T02-I18N-FOUNDATION` | resource keys、effective locale、确定性 fallback、加载入口、缺键和真实搜索消费者；R11-02–07及后续新增 UI 仅在实际 UI 接入时消费。具名串行集成 owner 提前验证合入，记录完整 containing SHA、验证证据与所有所需材料后释放；不等待整个搜索或 R11 最终验收。 |
| R11-05 / [主盘真实消费者](#behavior-a15) | `shared-imagefolder-atomic-writer` | 唯一 imagefolder/schema/atomic-writer；R11-06 仅在图像文件接线时消费已验证已集成子结果。R11-05 不自依赖，两组无相互整组前置；图像编码/快照工作独立，释放不等待 R11-05 整组或 R11-08 最终验收。 |
| R11-06 / [胜率图输出](#behavior-a16) | actual chart output | R15 仅所请求批量 PK 胜率图消费；queue/catalog/换色/干预/durable SGF 独立。 |
| R11-01 / [构建身份](#behavior-a02) | actual identity routing | R18 安装态支持/反馈/help 接线消费；生产渠道仍须单独决定，不阻塞已知 About 漂移修复。 |
| R11-08 / [集成验收](#integration-acceptance) | `T06-A-INTEGRATION` | R17 综合闭环消费 R11 共享状态实际验收；R17 独占跨功能全部入口、默认、保存及最终上游核对。 |

Game Info exact-target/focus 是 R11-01 内的结果顺序，不是自依赖。R13 `T02-AUTOLOAD-QUICK` 只约束未来准入自动换谱/handoff 路径，当前 replacement 调查先行；R14 `PROV-03::functional` 仅约束实际 live provider 响应性结论，不等待全部远程功能。UD-04-039-g model identity 仍属 R12；R11 review 音频设备残余与 R15 Match/countdown 扩展分别负责。功能组开工没有整组前置，不自动证明共享 state/interface/lifecycle 可并行：读写重叠由集成 owner 串行安排，不据此制造阶段依赖。

## 3. 七个能力组

| 来源能力组 | 完整用户目标 | 行为锚点 |
| --- | --- | --- |
| R11-01 / [全局功能搜索、准确入口与棋局信息焦点](#capability-01) | 离线搜索从菜单、工具栏和Ctrl/Command+K到真实功能/设置目标，取消与原生焦点返回安全；同步修正About及N/人机入口事实，继承shell已决处置。 | [A01](#behavior-a01)、[A02](#behavior-a02)、[A03](#behavior-a03)、[A04](#behavior-a04)、[A18](#behavior-a18)、[A23](#behavior-a23) |
| R11-02 / [完整棋谱编辑、全树变换与征子续走](#capability-02) | 强制黑白/交替落子、保留属性的列表插入与拖动、全树旋转/镜像/交换黑白及合法征子续走，完整Undo/Redo与保存重开。 | [A09](#behavior-a09)、[A10](#behavior-a10)、[A13](#behavior-a13) |
| R11-03 / [安全导入、完整快照保存与棋谱语义保留](#capability-03) | 当前谱替换及导入边界有可追溯结果，完整调用快照以非破坏原子写保存且不阻塞交互；贴目/日期/结果/评论保留，不重新打开历史raw/slots决定。 | [A05](#behavior-a05)、[A07](#behavior-a07)、[A08](#behavior-a08)、[A20](#behavior-a20)、[A26](#behavior-a26)、[A27](#behavior-a27)、[A28](#behavior-a28)、[A29](#behavior-a29) |
| R11-04 / [按点找手、可保存自动播放与无引擎复盘](#capability-04) | 按源顺序精确找手，主棋盘播放间隔可重启保存小数秒且不重定时当前timer；补齐无引擎与音频失败的有界复盘证据。 | [A11](#behavior-a11)、[A12](#behavior-a12)、[A24](#behavior-a24)、[A25](#behavior-a25) |
| R11-05 / [所选分支SGF与当前主棋盘图像导出](#capability-05) | 所选线路导出独立SGF，主棋盘调用时快照导出真实图像；目标安全、成功才更新共享imagefolder，并明确副棋盘历史非等价处置。 | [A06](#behavior-a06)、[A14](#behavior-a14)、[A15](#behavior-a15) |
| R11-06 / [胜率图可读性、未分析节点导航与PNG导出](#capability-06) | 目差领导方/近零与基线可读；未分析节点精确导航证据/gesture差异明确；当前所选线路调用快照导出1600×600 PNG。 | [A16](#behavior-a16)、[A19](#behavior-a19)、[A21](#behavior-a21) |
| R11-07 / [保持引擎身份的档案排序](#capability-07) | 置首/上移/下移/置尾持久化stable ID顺序，选择、Autoload Default、pending edits与真实Run均不变化。 | [A17](#behavior-a17) |

以下行为保留冻结正式合同的全部行为、Acceptance、候选条件和原生残余；A 编号只标识历史来源。

<a id="capability-01"></a>
## R11-01 — 全局功能搜索、准确入口与棋局信息焦点

<a id="behavior-a01"></a>
### 离线搜索与首消费者本地化

#### 行为与边界

Ctrl/Command+K、菜单搜索与常驻工具栏入口共用一个离线功能 catalog 和 panel，显示本平台快捷键。catalog 由已注册 action/实际设置 target 组成，保留来源的多语言关键词、拼音和英文检索；检索本身不执行命令、不启动引擎、不联网。上下键选择、Enter 只执行当前可用结果，disabled 原因可见，调用原 action owner，不能旁路 dirty departure、Match 或资源占用守卫。设置结果指向具体 target，而非任意第一个输入。
记录打开 panel 前的合法 component/window。Escape 或关闭取消不执行选中 action；原窗口激活后恢复仍 showing/focusable 的原 component，否则同 owner 的合法 anchor，最后 main workspace；owner 已销毁不夺焦点。设置窗口完成激活/挂载才聚焦指定非首 target；unsupported target 有可见理由/合法回退，过时回调不夺新 owner。panel 的 query/selection/focus 属会话，不新增跨重启持久化。
本能力组在这个真实新增 UI 内交付资源键、effective locale 解析和确定性 fallback，不先建空平台。保持当前有效语言作为未配置时兼容行为；基础资源缺键回落到该完整基础资源，不显示裸 key，不把局部翻译宣称 I18N-01 完成。每个 catalog label/reason/tooltip/action 使用资源键；来源关键词与显示翻译区分。交付可供 集成验收 owner 单独验证的 foundation 结果：资源键约定、locale/fallback 规则、加载入口、缺键断言、首消费者调用。其他批次只消费这一已验证结果，不等待整个搜索功能结束。完整语言名单/全部翻译/语言设置持久化由 R17 的 T02-I18N-01 冻结，R11 不决定名单。

#### Acceptance

- 一个 panel 从菜单、常驻工具栏及 registry-owned Ctrl/Command+K 打开，离线关键词/拼音/英文命中实际目标且键盘选择可执行。
- 禁用结果展示真实原因；执行沿原 action owner，dirty/Match/input-focus 边界不能被搜索绕过；未知 target 不假成功。
- 由搜索打开设置中的非首 target，原生窗口可接收键盘后输入只作用于指定 target；旧激活回调不抢新窗口。
- 从非默认 component 取消后原合法 component 再次收到键盘；销毁/失效目标按合法 owner 回退，workspace/tree/path/dirty/jobs 不变。
- 交付 集成验收 owner 可独立验收的资源键/effective-locale/fallback 基础结果和首搜索消费者；后续新增 UI 复用，无第二套 foundation。
- catalog/action labels、提示、失败理由使用资源键；fallback 不露 key；不虚称完整语言支持或提前决定 R17 的语言名单。

#### 最小证据面、继承及原生残余

APP-05 原 registry/reference 证据只证明旧键和 reference，不证明本搜索。UD-03-040/047 exact-target 与040a原生 focus 尚无本新增场景 pass；实现 owner 提供 catalog/registry/rendered fixtures 及当前受支持原生窗口实际键盘/取消/迟到焦点证据。浏览器不能替原生 activation。

<a id="behavior-a02"></a>
### About构建身份

#### 行为与边界

Help → About 展示实际构建版本（及已有可靠 commit/channel 信息），不再以固定0.1.0伪装本构建。修正指向 Java repository 的构建元数据，明确源码仓库 qiyi71w/lizzieyzy-next-tauri 与最终 release repository 是不同身份。产品 LizzieYzy Next、identifier org.lizzieyzy.next、binary lizzieyzy-next-desktop 和用户 app-data/settings identity 保持不变。已有可信配置的 Issues/help/build-source 地址集中读取；尚未配置或待决的 Releases/update/support channel 显示不可用/不提供可执行入口，不能伪造可用链接或选择最终仓库。失败信息真实可见，无 fallback 到 Java release/update 源。此票只修运行构建身份漂移，不迁移用户数据。

#### Acceptance

- About 的版本来自当前构建，repository 元数据不再指向 Java；源码地址与未决发行渠道有明确区分。
- 重启后原配置/数据路径、identifier/binary/product identity 不变；未配置渠道无伪可用入口。
- 功能原生 About 证据与未来 installed SemVer/channel/link 义务分列；不提升 REL-10 全项状态。

#### 最小证据面、继承及原生残余

Matrix REL-10旧固定状态串没有 installed identity pass。实现者提供元数据/链接分支 fixtures 与原生当前构建 About 观察；R18 T01-RELEASE/T06-RELEASE-CHANNEL 决定最终渠道，不阻塞本已知漂移修复。

<a id="behavior-a03"></a>
### read模式既有决定

#### 行为与边界

已决：Ticket08 §Answer/SHELL-04放弃argv恰为read时只隐藏status/engine menus但仍可编辑的含糊模式。Java readMode默认false且进程内不持久化；Next保留No-engine Mode。这是具名历史非等价cut，不是UI-04功能等价证明。本能力组将源目标、批准、当前入口/说明一致绑定，不恢复read模式、不重复设批准门。

批准来源：Ticket08 §Answer（历史 Ticket08，resolvedTickets08）

#### Acceptance

- 源目标→具名原resolved Answer/批准理由→当前非等价处置完整；沿用精确已决cut，不重新空开审批。
- 对应现保留功能/入口与本节边界一致；不把近似替代或决定记录当已实施源功能，不扩大删除或Accepted范围。

#### 最小证据面、继承及原生残余

最小面为固定 Inventory census SHELL-04、02 audit对应历史决定与原处置票；是产品合同/批准证据，不需要当前跑全套或原生来代替审批。若保留，后继实现者拥有受影响原生/文件/持久化验收。

<a id="behavior-a04"></a>
### 安全退出既有决定

#### 行为与边界

已决：Ticket08 §Answer/SHELL-09拒绝常驻File→Force Exit立即无persist退出。APP-03安全退出仍用Save/Discard/Cancel：Cancel或Save失败保留当前谱及运行资源，中止离开；成功Save或明确Discard后才持久化并teardown，timeout具名资源，仅该上下文给Retry/Exit anyway。不可扩大为任意时刻强退，也不可称其等价于常驻强退。

批准来源：Ticket08 §Answer（历史 Ticket08，resolvedTickets08）

#### Acceptance

- 源目标→具名原resolved Answer/批准理由→当前非等价处置完整；沿用精确已决cut，不重新空开审批。
- 对应现保留功能/入口与本节边界一致；不把近似替代或决定记录当已实施源功能，不扩大删除或Accepted范围。

#### 最小证据面、继承及原生残余

最小面为固定 Inventory census SHELL-09、02 audit对应历史决定与原处置票；是产品合同/批准证据，不需要当前跑全套或原生来代替审批。若保留，后继实现者拥有受影响原生/文件/持久化验收。

<a id="behavior-a18"></a>
### N及真实人机入口

#### 行为与边界

读取source/registry/菜单/reference的实际行为，统一陈旧“未接入”人机入口与已Accepted GAME-02真实新局/继续菜单事实。保持Ctrl+Home为New、N当前不New；不新增Ctrl+N，不把Java N(genmove)误写为New，不把human新局/继续当未实现。菜单用已真实接通的GAME-02 owner，reference和explanatory copy反映现键是否可执行；旧不可执行入口不能伪装可用，也不得因为stale copy隐藏真实可用菜单。
本能力组是已知事实校正与明确N状态记录；若产品要求给N新操作，必须另记N-action decision（action、dirty/Match/focus适用性、别名冲突、批准），决定前本能力组仅维持现N不New/explanatory no-op。既有registry single owner与text focus排除不改。无新设置/持久化；New仍走SGF-07 replacement。

#### Acceptance

- registry/menu/reference一致声明Ctrl+Home New、N不New，无Ctrl+N；GAME-02真实新局/继续入口不再被旧未接入说明混淆。
- editable输入typed N不触发New/Match，New cancel保持document；真实human菜单沿既有Matchdialog和占用guard。
- 交付N现状态/原源语义对照及任何新N语义的明确未批准记录，未借本次校正批准新绑定。

#### 最小证据面、继承及原生残余

02 audit App/shortcuts/AppChrome精确事实；SGF-10原6781a50e600c4283ed42cb57f0273ea03fb7ec74及H-SAVE-DEPARTURE只继承New/replacement原范围。future owner registry/rendered/reference检查加原生focus/真实human菜单可达性；不重测GAME-02全部对局。

<a id="behavior-a23"></a>
### 棋局信息owner及Komi焦点

#### 行为与边界

搜索打开Game Info时dialog属于真实调用owner，首次目标为komi输入（不是PB首input），可直接键盘编辑；显式其他search target仍由A01exact-target规则决定。Cancel不提交draft/不改document并返回合法source/searchfocus；owner退出或target销毁后late callback不夺新window。
保留SGF-13 PB/PW/KM validation、Apply一次history、Undo/Redo、unchanged raw KM bytes、name-onlyanalysis continuity和komi generation失效合同。这里只补owner/focus，不新增default komi、rootKM覆盖、日期更新或设置持久化。共享输入是behavior-a01 已验证 exact-target/focus 结果，不依赖导出/排序/全searchcatalog全部完工。

#### Acceptance

- 搜索Game Info激活后初始Komi能收键；真实owner/return-search focus正确，与取消search恢复旧component流程分开验证。
- Cancel/invalid input无document修改；Apply/Undo/Redo仍沿SGF-13，unchanged KM[6.50]不改字节。
- owner退出/target销毁或新dialog出现，旧focus回调不抢新window；nativefocus证据真实取得。

#### 最小证据面、继承及原生残余

H-KOMI-EDITOR01429efcaf1fa732c1c0f2c2dcf50d367d478ea8 Windows isolatedappdata 原validation/Cancel/Apply/Undo/Save PB/PW/KM与KataGo name-only/komi restart可继承不变边界，不是本search/focus pass。owner rendered identity和实际nativeactivation/keyboard/return证据。

### 来源覆盖

唯一责任意图：[T03-NAV-GLOBAL-FUNCTION-SEARCH](MIGRATION_SOURCE_MAP.md#t03-nav-global-function-search), [T02-I18N-FOUNDATION](MIGRATION_SOURCE_MAP.md#t02-i18n-foundation), [T01-IDENTITY](MIGRATION_SOURCE_MAP.md#t01-identity), [T02-H01](MIGRATION_SOURCE_MAP.md#t02-h01), [T02-H02](MIGRATION_SOURCE_MAP.md#t02-h02), [T02-N-ENTRY](MIGRATION_SOURCE_MAP.md#t02-n-entry), [T03-SGF13-GAMEINFO-KOMI-FOCUS](MIGRATION_SOURCE_MAP.md#t03-sgf13-gameinfo-komi-focus)。

来源意图：[T03-NAV-GLOBAL-FUNCTION-SEARCH](MIGRATION_SOURCE_MAP.md#t03-nav-global-function-search), [T03-UI01-TOOLBAR-SEARCH-ENTRY](MIGRATION_SOURCE_MAP.md#t03-ui01-toolbar-search-entry), [T03-UI02-DIALOG-FOCUS-NAV](MIGRATION_SOURCE_MAP.md#t03-ui02-dialog-focus-nav), [T03-NAV-SEARCH-CANCEL-FOCUS](MIGRATION_SOURCE_MAP.md#t03-nav-search-cancel-focus), [T02-I18N-FOUNDATION](MIGRATION_SOURCE_MAP.md#t02-i18n-foundation), [T01-IDENTITY](MIGRATION_SOURCE_MAP.md#t01-identity), [T02-H01](MIGRATION_SOURCE_MAP.md#t02-h01), [T02-H02](MIGRATION_SOURCE_MAP.md#t02-h02), [T02-N-ENTRY](MIGRATION_SOURCE_MAP.md#t02-n-entry), [T03-SGF13-GAMEINFO-KOMI-FOCUS](MIGRATION_SOURCE_MAP.md#t03-sgf13-gameinfo-komi-focus)。

Parity IDs（不改原状态）：APP-03, APP-05, GAME-02, I18N-01, PREF-01, REL-10, SGF-10, SGF-13, UI-04, UI-06, UI-07。

Delta IDs：UD-03-040, UD-03-040a, UD-03-043, UD-03-047, UD-03-049。

<a id="capability-02"></a>
## R11-02 — 完整棋谱编辑、全树变换与征子续走

<a id="behavior-a09"></a>
### 直接棋子编辑

#### 行为与边界

Edit/右键的添加黑、添加白、交替添加、插入列表及移动棋子目标都保留，不能仅交付普通合法play或既有root setup editor。Rust current-game owner 在精确generation/NodePath上一次提交、一次Undo/Redo；bounds/occupancy与后继位置合法性在提交前验证，拒绝、gesture cancel、Match占用或stale generation不改tree/path/dirty/analysis。保留无关branches、unknown properties与个人评论；保存/重开与编辑后投影一致。
冻结 Java v1 `7b4027531c2b26062d0bfc27a040cc550cfbea4d` 的操作表如下；是已有目标的行为来源，不是新产品批准。Java重放机制可能丢属性/非move节点，Next不得复制该损失，必须满足本能力组完整保留与非破坏拒绝约束。

| 操作 | SGF/颜色与光标/history合同 | 来源 |
| --- | --- | --- |
| 添加黑/白/交替 | Menu设 `Input.insert=0`、`blackorwhite=1/2/0`；Board.placeForManual写实际B/W move，交替用当前side-to-play，不改为AB/AW setup或另写PL来伪造forced颜色。新move成为精确当前节点，保留旧branch和properties，单步Undo/Redo。例 `(;SZ[19];B[aa])` 在B节点添加黑bb得新B[bb]后继，添加白得W[bb]；交替取当前turn。 | [Menu L3705–3748](https://github.com/wimi321/lizzieyzy-next/blob/7b4027531c2b26062d0bfc27a040cc550cfbea4d/src/main/java/featurecat/lizzie/gui/Menu.java#L3705-L3748)、[Board L2618–2624](https://github.com/wimi321/lizzieyzy-next/blob/7b4027531c2b26062d0bfc27a040cc550cfbea4d/src/main/java/featurecat/lizzie/rules/Board.java#L2618-L2624) |
| list insertion，selected recorded move/pass | insertMove取得包含selected节点的linked list；把新B/W节点接在selected后、原所有children前，稳定顺序转为新节点children；Next选择新节点精确NodePath，整笔history可逆。不得删掉不合法后继，先验证全部受影响投影，失败整笔拒绝。 | [LizzieFrame L19076–19109](https://github.com/wimi321/lizzieyzy-next/blob/7b4027531c2b26062d0bfc27a040cc550cfbea4d/src/main/java/featurecat/lizzie/gui/LizzieFrame.java#L19076-L19109)、[Board L6174–6250](https://github.com/wimi321/lizzieyzy-next/blob/7b4027531c2b26062d0bfc27a040cc550cfbea4d/src/main/java/featurecat/lizzie/rules/Board.java#L6174-L6250) |
| recorded-move drag | 修改原记录B/W的coordinate，保留颜色、该node其他properties及child order，不制造setup/新move；光标恢复到拖前选中节点identity（被拖节点就是当前时仍同一节点）。例 `(;SZ[19];B[aa]C[x];W[bb])` 当前W，把aa拖cc得到 `(;SZ[19];B[cc]C[x];W[bb])`，仍选W。单步Undo还原tree/cursor。 | [DraggedReleased L10293–10362](https://github.com/wimi321/lizzieyzy-next/blob/7b4027531c2b26062d0bfc27a040cc550cfbea4d/src/main/java/featurecat/lizzie/gui/LizzieFrame.java#L10293-L10362) |
| starting-stone drag | distinct starting-stone path，修改setup石子位置而非生成move；Next改原AB/AW coordinate，保留其他setup/PL/metadata与children及拖前cursor。例 `(;SZ[19]AB[aa]PL[W]C[x];W[bb])` aa拖cc只将AB[aa]改AB[cc]。occupied/offboard/same-coordinate（no-op）与非法后继遵守本能力组原子约束。 | [L10363–10413](https://github.com/wimi321/lizzieyzy-next/blob/7b4027531c2b26062d0bfc27a040cc550cfbea4d/src/main/java/featurecat/lizzie/gui/LizzieFrame.java#L10363-L10413) |
| permission/persistence | `ui.allow-double-click`缺失true、`ui.allow-drag`缺失false；Menu写各ui键，Next沿PREF-01持久化成功/失败读回合同。`ui.enable-click-review`缺失false是独立REVIEW-01权限，不是前两者别名。Match/试下/score/setup先由现有input owner路由，drag还拒绝urlSgf与human/analysis play。 | [Config L2083–2096](https://github.com/wimi321/lizzieyzy-next/blob/7b4027531c2b26062d0bfc27a040cc550cfbea4d/src/main/java/featurecat/lizzie/Config.java#L2083-L2096)、[Menu L3750–3779](https://github.com/wimi321/lizzieyzy-next/blob/7b4027531c2b26062d0bfc27a040cc550cfbea4d/src/main/java/featurecat/lizzie/gui/Menu.java#L3750-L3779)、[Input L40–154](https://github.com/wimi321/lizzieyzy-next/blob/7b4027531c2b26062d0bfc27a040cc550cfbea4d/src/main/java/featurecat/lizzie/gui/Input.java#L40-L154) |

Insertion可执行树例：`(;SZ[19];B[aa]C[keep]XX[v](;W[bb];B[cc])(;W[dd]))` 当前 `[0]`，插白ee应得 `(;SZ[19];B[aa]C[keep]XX[v];W[ee](;W[bb];B[cc])(;W[dd]))`，新cursor `[0,0]`，两旧children依次为 `[0,0,0]` / `[0,0,1]`。空root `(;SZ[19])` 插Baa得 `(;SZ[19];B[aa])`。非空普通root的源算法跳root、以first recorded descendant为anchor：`(;SZ[19];B[aa];W[bb])` 当前 `[]` 插Wee得 `(;SZ[19];B[aa];W[ee];W[bb])`；不可自行把源root规则换成first move之前。

已批准A-INSERT-NONMOVE-ANCHOR（已批准滚动规划决定）：选中非根setup/comment/PL节点时，新节点插在所选节点之后、原孩子之前，全部旧节点/属性/分支保留并选择新节点；无新审批门。例 `(;SZ[19];B[aa];AB[bb]C[setup]PL[W];W[cc])`，selected `[0,0]`，插Wee得到 `(;SZ[19];B[aa];AB[bb]C[setup]PL[W];W[ee];W[cc])`，新cursor `[0,0,0]`，旧Wcc移至 `[0,0,0,0]`。有多个children时依原顺序成为新节点children；完整后继合法性必须预验证，失败整笔拒绝；不能以源删除重放丢节点。无recorded descendant时同样接在selected后。普通非空root的已知source锚点保持首个recorded descendant，空root直接新建后继；本批准不改变root规则。

#### Acceptance

- 上述源知operation/default/key及insertion move/pass/root树例成为fixture断言；按已批准non-root setup/comment/PL后插合同与本节before/after/cursor覆盖该场景，黑/白/交替、drag与invalid-descendant preservation不降级为可选项。
- 每项在矩形边界/occupied/非法后继/旧generation/Cancel场景非破坏；合法编辑单步Undo/Redo还原完整树和光标。
- 无关branch/property/C不丢失；native authoring与Save/reopen语义匹配，Match安全owner未被绕过。

#### 最小证据面、继承及原生残余

SGF-11/12只提供既有setup/history seam；SGF-12原候选 ddb31bb7b66450d339075ab4f4d87c53f8c7ee73 的结构编辑不证明本强制插入/拖动。实现 owner 获得Rust树/投影/history fixtures与真实原生drag/edit/Undo/Save-reopen。

<a id="behavior-a10"></a>
### 全树变换

#### 行为与边界

Edit transform入口共用Rust whole-tree owner，按冻结Inventory的旋转/镜像/交换黑白操作，不实现只旋转主盘像素的替代品。继承非方棋盘拒绝旋转与Match占用拒绝，镜像覆盖实际支持矩形维度。遍历所有branches，对每个已支持坐标属性逐字段变换：B/W moves含pass、AB/AW/AE setup及LB/CR/SQ/MA/TR markup；解析器实际支持的其他coordinate-bearing属性也列清单并一致处理（包括压缩坐标/线段属性如已支持），未知非坐标metadata原值不被猜改。NodePath结构/child order保留。交换黑白同时交换B/W、AB/AW、PL及已支持color-owned元数据/结果语义，不把字符串全文替换成另一颜色；不可支持/不可安全解释的属性拒绝而不部分commit。
第一真实手不跳过，leading W pass保留记录颜色或按color-swap交换；完整变换一次history unit，可Undo/Redo。几何/颜色改动沿现有generation与analysis invalidation owner，不搬用旧position分析。取消/任何解析或验证错误保持整棵原树。

#### Acceptance

- rotate/mirror/color-swap对所有branch的move/setup/markup逐字段覆盖，含first move与leading White pass，NodePath结构及无关metadata稳定。
- square/rectangular/source支持边界与Match守卫可解释；出错/取消无partial mutation。
- 单步Undo/Redo与serialize/reparse/native Save-reopen保持结果，旧position分析不伪装有效。

#### 最小证据面、继承及原生残余

H-KOMI-EDITOR/H-OFFLINE不证明transform。实现者提供全树Rust/property fixtures与实际原生transform/Undo/Save-reopen；保留SGF-01/11/14原前置，无installed/engine-build门。

<a id="behavior-a13"></a>
### 征子续走

#### 行为与边界

Game → 继续征子按固定Java v1 Board.continueLadderByOne用户predicate运行，无engine依赖。阈值为5个可取的最近coordinate moves、period4；从选中历史向前采样（遇pass/无法取得坐标拒绝），比较第5手与相隔4手的位移dx/dy，必须abs(dx)=abs(dy)=1；下一点为过去第4手+dx/dy，且该点及(x+dx,y)、(x,y+dy)都在盘内且空。每步仍经过Rust Go合法性，重复上述predicate直到几何/occupancy不再满足。不复制Java clear/replay或进程机制。
在隔离candidate tree先完成连续合法moves，再一次history commit到精确选中NodePath；不足5手、不匹配、stale/Match/实际规则验证失败给具体理由，不留部分tree。正常predicate终止是完成合法前缀，不是运行异常；若零步则非mutating refusal。保留其他branches/properties，Undo/Redo恢复整次continuation，Save/reopen一致；分析遵循现有position identity invalidation。

#### Acceptance

- 5手/period4/对角位移与三个空点predicate按固定source可复现，below-threshold/pass/非对角/blocked/off-board给解释且零步不变。
- 所有生成手经现有Rust rules；异常验证/stale/Match无部分树，正常终止保留全部已验证continuation。
- selected branch以外数据不丢；单步Undo/Redo及原生Continue ladder/Save-reopen结果一致，不启动engine。

#### 最小证据面、继承及原生残余

固定Board.java@7b4027531c2b26062d0bfc27a040cc550cfbea4d:6793–6849 predicate已读取；source事实不是Next native pass。实现owner提供Rust legality/tree/history fixtures及原生Game入口/Undo/Save-reopen。

### 来源覆盖

唯一责任意图：[T02-SGF-15](MIGRATION_SOURCE_MAP.md#t02-sgf-15), [T02-SGF-16](MIGRATION_SOURCE_MAP.md#t02-sgf-16), [T02-REVIEW-06](MIGRATION_SOURCE_MAP.md#t02-review-06)。

来源意图：[T02-SGF-15](MIGRATION_SOURCE_MAP.md#t02-sgf-15), [T02-SGF-16](MIGRATION_SOURCE_MAP.md#t02-sgf-16), [T04-TRANSFORM](MIGRATION_SOURCE_MAP.md#t04-transform), [T02-REVIEW-06](MIGRATION_SOURCE_MAP.md#t02-review-06)。

Parity IDs（不改原状态）：REVIEW-01, REVIEW-06, RULE-01, SGF-01, SGF-04, SGF-11, SGF-12, SGF-14, SGF-15, SGF-16。

Delta IDs：UD-04-027。

<a id="capability-03"></a>
## R11-03 — 安全导入、完整快照保存与棋谱语义保留

<a id="behavior-a05"></a>
### raw保存既有决定与源差别

#### 行为与边界

已决：Ticket10 §Answer/SGF-03-ADJ-SAVE-MORE舍弃有损raw/raw-with-comment；不是保留原文件bytes，也不是SGF-05等价于独立内容选择。普通Save不暴露raw flags，取消不泄漏模式；个人C与生成信息分离的ADR0003不变。保留来源逐模式差别供保存边界核对，不恢复有损writer，不重新审批：

| 内容 | raw | raw-with-comment | 固定来源 |
| --- | --- | --- | --- |
| 树/move/pass/setup/分支 | 重新序列化整个树；不保持原文件字节 | 相同 | [SGFParser L1323–1430](https://github.com/wimi321/lizzieyzy-next/blob/7b4027531c2b26062d0bfc27a040cc550cfbea4d/src/main/java/featurecat/lizzie/rules/SGFParser.java#L1323-L1430)、[L1540–1588](https://github.com/wimi321/lizzieyzy-next/blob/7b4027531c2b26062d0bfc27a040cc550cfbea4d/src/main/java/featurecat/lizzie/rules/SGFParser.java#L1540-L1588) |
| 根 metadata | 重写 KM/PW/PB/DT/RE/SZ/CA 和 HA；不新增普通writer的AP/DZ，不能推断既有AP/DZ或unknown被删除 | 相同 | [L1203–1313](https://github.com/wimi321/lizzieyzy-next/blob/7b4027531c2b26062d0bfc27a040cc550cfbea4d/src/main/java/featurecat/lizzie/rules/SGFParser.java#L1203-L1313) |
| 原节点 properties/unknown | propertiesString仍输出；非history/setup节点的跳过也沿用writer，非任意unknown清理规则 | 相同 | L1323–1362、L1543–1583 |
| 根 C 与有效 LZOP/LZOP2 | 普通本地保存两种raw仍输出；并非“所有评论/analysis全去除” | 相同 | [L1364–1392](https://github.com/wimi321/lizzieyzy-next/blob/7b4027531c2b26062d0bfc27a040cc550cfbea4d/src/main/java/featurecat/lizzie/rules/SGFParser.java#L1364-L1392) |
| 非根 C | 不追加 C | 追加当前C；有analysis时formatter可改变C，不能称只保留个人评论 | [L1763–1773](https://github.com/wimi321/lizzieyzy-next/blob/7b4027531c2b26062d0bfc27a040cc550cfbea4d/src/main/java/featurecat/lizzie/rules/SGFParser.java#L1763-L1773)、[L1859–1880](https://github.com/wimi321/lizzieyzy-next/blob/7b4027531c2b26062d0bfc27a040cc550cfbea4d/src/main/java/featurecat/lizzie/rules/SGFParser.java#L1859-L1880) |
| 非根新生成 analysis tags | 两模式都跳过standard comment/analysis追加；已有properties不因此成为已删除 | 相同 | L1763–1792 |



批准来源：Ticket10 §Answer（历史 Ticket10，resolvedTickets10）

#### Acceptance

- 源目标→具名原resolved Answer/批准理由→当前非等价处置完整；沿用精确已决cut，不重新空开审批。
- 对应现保留功能/入口与本节边界一致；不把近似替代或决定记录当已实施源功能，不扩大删除或Accepted范围。

#### 最小证据面、继承及原生残余

最小面为固定 Inventory census SGF-03-ADJ-SAVE-MORE、02 audit对应历史决定与原处置票；是产品合同/批准证据，不需要当前跑全套或原生来代替审批。若保留，后继实现者拥有受影响原生/文件/持久化验收。

<a id="behavior-a07"></a>
### 手动存档及第二退出保存既有决定

#### 行为与边界

已决：Ticket10 §Answer分别舍弃manual thumbnail slots/slot files及第二autosave-on-exit，两目标分别记录，不拿一个批准替另一个。Java主动save/load/named slots及thumbnail目标不同于APP-04自动恢复；auto-save-exit默认true、resume-previous-game默认false、slot位于save/，注释File Resume不是入口。Next APP-04仅恢复tree/个人C/精确NodePath/source/dirty，normal-launch恢复默认off，不恢复engine/jobs/provider；不宣称manual slots等价、不臆定1–9 slots、不新增第二exit-save。

批准来源：Ticket10 §Answer（历史 Ticket10，resolvedTickets10）

#### Acceptance

- 源目标→具名原resolved Answer/批准理由→当前非等价处置完整；沿用精确已决cut，不重新空开审批。
- 对应现保留功能/入口与本节边界一致；不把近似替代或决定记录当已实施源功能，不扩大删除或Accepted范围。

#### 最小证据面、继承及原生残余

最小面为固定 Inventory census SGF-03-ADJ-TEMP、02 audit对应历史决定与原处置票；是产品合同/批准证据，不需要当前跑全套或原生来代替审批。若保留，后继实现者拥有受影响原生/文件/持久化验收。

<a id="behavior-a08"></a>
### root KM既有决定

#### 行为与边界

已决：Ticket10 §Answer按parsed root KM权威，SGF/GIB缺KM走New默认，舍弃conditional-ignore-file-komi pref。Java uiConfig.read-komi缺失true且createDefaultConfig不含键，普通SGF/GIB可忽略KM而setup/handicap仍用root KM。这些源区别保留为非等价cut理由，不新建忽略开关、不弱化root权威。改变root KM仍沿现generation/analysis失效、Undo与Save-reopen，不以相近功能假称忽略目标已实现。

批准来源：Ticket10 §Answer（历史 Ticket10，resolvedTickets10）

#### Acceptance

- 源目标→具名原resolved Answer/批准理由→当前非等价处置完整；沿用精确已决cut，不重新空开审批。
- 对应现保留功能/入口与本节边界一致；不把近似替代或决定记录当已实施源功能，不扩大删除或Accepted范围。

#### 最小证据面、继承及原生残余

最小面为固定 Inventory census SGF-03-ADJ-KOMI、02 audit对应历史决定与原处置票；是产品合同/批准证据，不需要当前跑全套或原生来代替审批。若保留，后继实现者拥有受影响原生/文件/持久化验收。

<a id="behavior-a20"></a>
### 快速换谱的有界安全调查

#### 行为与边界

唯一问题：当前替换请求、generation与旧任务cleanup能否一致封存，避免旧结果写新game？先消费SGF-07 prepare/resolve owner与App现有有限任务handoff事实；对一个被覆盖请求、一个坏候选/取消及最终有效请求保留可控late result，比较原source latest-request-wins用户目标和现明确busy拒绝合同。不要擅把现在的拒绝改为新队列/50ms debounce，也不指定五文件矩阵。
坏file/Cancel保留原tree/path/dirty；最终有效replace只有新identity能publish，旧cleanup/analysis不attach新game。T02-AUTOLOAD-QUICK由R13拥有，尚未准入自动快析与已Accepted ANA-16显式task不同；现supported安全调查立即进行，不等整自动feature。auto-load路径相关结论只在B提供实际handoff artifact后补，未实现则记录其future实施native门，不能虚称其已覆盖。
停止于每实际路径“不适用/原窄证据/具名缺口”的source/evidence tuple表，附失败最小repro或理由、明确受影响功能owner及后继验收；结论前不授权新修复或改变请求策略。

#### Acceptance

- 一个被覆盖/拒绝请求、坏candidate/Cancel和最终有效replace，受控late结果保留原/新document正确identity；记录现busy合同与Java latest-request差别。
- current-supported与B未来auto-load路径分列，未准入auto-load留具名B门；既有SGF-07/ANA-16Accepted不扩大。
- 输出bounded结论、原candidate/platform/task条件与invalidations、缺口/后继owner；无固定时序或推测性实现。

#### 最小证据面、继承及原生残余

H-SAVE-DEPARTURE4fd710e1c24a991665c2ed47f58bbb178b8c1c82（Windows New Cancel保持whole-game；confirmed departure SaveAs Cancel需显式restart）；H-ANALYSIS5e593537f702af0c651a2c3bf0f0fd77758052e2（Windows standalone KataGo1.16.4 EigenCPU显式task/Pause/Continue/lane Cancel）仅原范围。调查owner用owner/rendered受控迟到fixtures；发现unsupported自动路径只交B，不造native pass。

<a id="behavior-a26"></a>
### 普通Save目标及原Run意图调查

#### 行为与边界

当前ordinarySave/SaveAs调查先行，raw/raw-with-comment历史比较按本能力组[behavior-a05](#behavior-a05)已批准处置记录覆盖或不适用结果。分解UD-04-039-b至f：无engineSave chooser；历史raw/raw-comment取消后的mode隔离结论按已决cut记录；overwriteCancel保留captured原sameRun的先前runninganalysis意图，不resume用户paused/replaced/newRun；最终.sgftarget先normalize再existingcheck/prompt；game.SGF不重复追加。源四save modes（ordinary/raw/raw-comment/branch）分别留覆盖/处置结果，rawflagsJava机制不复制。
existingtargetCancel必须保护所有原byte/sourcepath/dirty，不可prompt检查basename而写另一后缀target。用protected-sgf-中文对应existing.sgftarget及game.SGF为来源具体case，不新加任意suffix矩阵。比较Next既有departure明确停止/需manualrestart合同，不把ordinary-save与New/departureCancel合并为一条自动resume。
停止于save相关四项（rawmode隔离、sameRunintent、normalizedtarget、case-insensitive suffix）逐项原覆盖或具名可达差异，加noenginechooser关联结果；raw比较直接引用本能力组behavior-a05已批准cut；UD039-gmodel归R12 T04-MODEL-IDENTITY，不吞入本能力组。尚未证明的安全修复需结论后具名；只有真实发现来源未覆盖的新目标才列具名差异owner，普通Save调查按冻结合同完成。

#### Acceptance

- currentordinarySave原entry/target与sameRun原intent查清；raw历史/当前比较沿本能力组behavior-a05已批准处置给覆盖或不适用结果，普通Save无需等待新raw决定。
- 具体normalizedexistingtargetCancel保护byte/path/dirty，game.SGF写入/overwritecheck同target且无重复后缀；modeflags不泄漏。
- ordinarysameRunrunning/paused/replaced与Nextdeparture停止边界分列，所有save子项有窄tuple或named successor/decision和native责任。

#### 最小证据面、继承及原生残余

H-SAVE-DEPARTURE4fd710e1c24a991665c2ed47f58bbb178b8c1c82/R5 48db2b2833f9deb45bd7dcd47181f5da77348a0c不证明ordinarySavewhileanalyzing；H-OFFLINE66c906f3117cc1b8274673a0382456b91f02e39f仅旧SaveAs。futureowner先source/history，缺实际chooser/overwrite/sameRun则本地nativeSave与兼容已qualified真实engine获证，不compiledengine/installer门。

<a id="behavior-a27"></a>
### 完整快照原子保存

#### 行为与边界

继续沿SGF-06/07authoritativeSave owner捕获调用时完整tree（所有branch/setup/C/unknownproperties）与identity-validanalysis再serialize，后续edit/analysis不混入已捕获快照。为现directstd::fs::write增加atomicreplacement用户合同：existingtarget在准备/encode/write/replace任一失败保留全部原byte，sourcepath/dirty/currenttree不变；不得回退directoverwrite。确认overwrite在实际normalizedtarget进行。写盘成功仅记录此snapshot revision的savepoint；后续edit或newdocument不能被late成功标clean/改path，结果可见且绑定原document/snapshot。
实际UI save从capture→encode/write/replaceoff-UI调度与最终结果区分：picker offload不是写盘offload，证明write工作不在交互线程并维持cancel/progress/结果入口响应，无任意ms阈值。Cancel若发生于提交前无targetreplace；既有SGF-07 departurecancel合同不变。平台已有file权限/目标存在/replace失败语义按以上non-destructive合同冻结，不能把catalog atomic证据代SGF；temp/rename是可选技术方式，任何无法atomicreplace平台/target可见拒绝而不降级unsafe写。
无需等待rawdecision或全ordinarysave调查，新数据正确性可以提前；不宣称已观察runtime数据丢失。

#### Acceptance

- 捕获完整structure+analysis快照，serialize/reparse/reopen一致，late edits不混入，saved snapshot成功不能clean后续revision或newgame。
- existingbyte在prepare/encode/write/replace故障全保留，path/dirty/currenttree不变，无unsafefallback；成功atomicreplace与结果visible。
- 真实nativeSave展示off-UIwrite而不只picker，等待时合法input/cancel/result可达，latecompletion不夺newgame；SGF-06/07原history不改。

#### 最小证据面、继承及原生残余

UD045现snapshot源码真、persist_save_snapshotdirectstdwrite与syncSave/SaveAspickeroffload区分；没有atomic或大analysis off-UI原生pass。ownerRust faultfilesystem/revision/desktop scheduling/renderedfixtures及实际nativeSave/reopen/失败，noinstaller/noenginecompile。

<a id="behavior-a28"></a>
### DT表达式边界调查

#### 行为与边界

原生打开完整日期、部分日期、多日期及无DT的固定SGF，经元数据编辑和Save/reopen比较rootDT；导入日期表达式不被“今天”改写，无DT不凭空填日期。New日期独立，不借本调查定义新默认datepolicy。当前任意rootproperty保留与已有一个DT fixture不是全部边界native覆盖；JavaDate→String变更不是Next已丢日期证明。
停止于四类边界的source/entry/native结果表，若失败仅具名可达损失路径后继与最小输入，保留其他root字段与SGF-13metadata合同，不在本调查广改dateparser。

#### Acceptance

- full/partial/multiple/absentDT由nativeOpen→metadataedit→Save/reopen表达式相等，absence保持absence或精确namedgap。
- 原candidate/platform/fixture与未跑native分别记录，不把Java缺陷判Next故障；发现损失只定向具名后继。

#### 最小证据面、继承及原生残余

UD0511source a49c10c77f0a28c0ad802fc9d3905d1dcabb03de/238d74a46dc7f9217f55d7a2d23708049279dd42/8c43fdd24b01acd6485b8b3911708290ae18ab75；CurrentSgfDocumentroot保留源码已有DTtest未跑此边界。owner最小repositoryfixture+真实localnativeOpen/edit/Save/reopen，不依providerlive或installer。

<a id="behavior-a29"></a>
### 公共导入转义及RE/C边界调查

#### 行为与边界

相同来源样本经localfile、clipboard和当前supportedproviderpayload进入公共CurrentSgfDocument::open/SGF-07；对比tree、detachedRE和personalC。属性值之外escapedLF/CRLF可按来源normalize，值内部的SGF转义/换行不得全局replace破坏；含ambiguous/malicious truncation先parse拒绝，当前tree/path/dirty/selection不变。detached RE保持rootresultmetadata，不写入personalC，保留ADR0003。
Fox专用escapednewline已有覆盖不等于本地/clipboard所有入口。调查限同sample跨入口语义/安全，provider用现可达payloadfixture不必账号写/live抓凭据。停止于源/入口等价或具体可达差异和named后继；没有差异不批量重实现parser。

#### Acceptance

- 相同sample跨local/clipboard/provider当前入口tree/RE/C一致，structural escapedLF/CRLF兼容但value内部转义原义保留。
- 歧义/截断拒绝且currentdocument非破坏，RE不塞C；Fox专用历史scope与新增公共边界分列。
- 输出每entry source/原candidate/conditions或缺口，只有实证可达差异建立定向后继；local/clipboard原生拒绝/成功获证。

#### 最小证据面、继承及原生残余

UD0518 ec19f8b847ca719c0189eaaa0177592094a8208a/1c829ab080f604da019c5ab14f6e6c27782b8319/65cc93842eb26e0dd0ee911f3d16493b52cd6399/0fe5b3769e5f1402013df5520643466ff021f5e6 source。ownerparser/publicreplacementfixture+nativefile/clipboard；provider payloadfixture只是payload面，不伪装online服务。

### 来源覆盖

唯一责任意图：[T02-H12](MIGRATION_SOURCE_MAP.md#t02-h12), [T02-H14](MIGRATION_SOURCE_MAP.md#t02-h14), [T02-H15](MIGRATION_SOURCE_MAP.md#t02-h15), [T03-SGF07-RAPID-SWITCH-TRANSACTION](MIGRATION_SOURCE_MAP.md#t03-sgf07-rapid-switch-transaction), [T04-SAVE-TARGET](MIGRATION_SOURCE_MAP.md#t04-save-target), [T04-SAVE-ATOMIC](MIGRATION_SOURCE_MAP.md#t04-save-atomic), [T05-SGF-DATE-CHECK](MIGRATION_SOURCE_MAP.md#t05-sgf-date-check), [T05-SGF-IMPORT-CHECK](MIGRATION_SOURCE_MAP.md#t05-sgf-import-check)。

来源意图：[T02-H12](MIGRATION_SOURCE_MAP.md#t02-h12), [T02-H14](MIGRATION_SOURCE_MAP.md#t02-h14), [T02-H15](MIGRATION_SOURCE_MAP.md#t02-h15), [T03-SGF07-RAPID-SWITCH-TRANSACTION](MIGRATION_SOURCE_MAP.md#t03-sgf07-rapid-switch-transaction), [T04-SAVE-TARGET](MIGRATION_SOURCE_MAP.md#t04-save-target), [T04-SAVE-ATOMIC](MIGRATION_SOURCE_MAP.md#t04-save-atomic), [T05-SGF-DATE-CHECK](MIGRATION_SOURCE_MAP.md#t05-sgf-date-check), [T05-SGF-IMPORT-CHECK](MIGRATION_SOURCE_MAP.md#t05-sgf-import-check)。

Parity IDs（不改原状态）：ANA-06, ANA-14, ANA-16, ANA-19, APP-04, SGF-01, SGF-03, SGF-05, SGF-06, SGF-07, SGF-08, SGF-09, SGF-10, SGF-13, SGF-17。

Delta IDs：UD-03-014, UD-04-039, UD-04-045, UD-05-11, UD-05-18。

<a id="capability-04"></a>
## R11-04 — 按点找手、可保存自动播放与无引擎复盘

<a id="behavior-a11"></a>
### 按点找手

#### 行为与边界

右键找手/点搜索指向coordinate-bearing recorded moves，跨所有branches使用精确NodePath，经REVIEW-01选择而不是用move number或position hash近似。查询不改tree/child order/source path/dirty，不产生engine工作。无匹配/occupied但非recorded move/离盘点/stale generation显示非破坏结果；setup/pass不是matching move，同坐标多次与不同branches不能误当唯一手。
冻结 Java v1 [Board.findMove L6497–6538](https://github.com/wimi321/lizzieyzy-next/blob/7b4027531c2b26062d0bfc27a040cc550cfbea4d/src/main/java/featurecat/lizzie/rules/Board.java#L6497-L6538) 顺序：当前matching recorded node立即no-op；否则最近matching ancestor；否则当前next()所选continuation逐手向前第一个match；否则从root按稳定child index顺序DFS，命中即选。再次查询留在已matching节点，不cycle/wrap，不要求新result list。无match不改cursor。例树 `(;SZ[19](;B[aa];W[bb])(;B[aa];W[cc])(;B[dd];W[ee]))`：当前 `[1,0]` 查aa→`[1]`；再查→仍`[1]`；当前 `[2]` 查aa→root DFS `[0]`；当前 `[2]` 查ee→continuation `[2,0]`；当前 `[]` 查aa→continuation `[0]`；查不存在ff原path保持。
Double-click源入口并非full-root search：[LizzieFrame.onDoubleClicked L8477–8489](https://github.com/wimi321/lizzieyzy-next/blob/7b4027531c2b26062d0bfc27a040cc550cfbea4d/src/main/java/featurecat/lizzie/gui/LizzieFrame.java#L8477-L8489) 调 [Board.gotoAnyMoveByCoords L6150–6172](https://github.com/wimi321/lizzieyzy-next/blob/7b4027531c2b26062d0bfc27a040cc550cfbea4d/src/main/java/featurecat/lizzie/rules/Board.java#L6150-L6172)，只有current/nearest ancestor/current continuation三段，没有root DFS fallback。因此上述树从 `[2]` 双击aa不选另一branch，显式右键搜索才选 `[0]`；不静默扩展double-click scope。
`ui.allow-double-click`缺失true；Menu写该键，经Next PREF-01成功持久化/失败保持合同；独立于默认false的 `ui.allow-drag` 和 `ui.enable-click-review`（不是该搜索开关）。[Config L2083–2096](https://github.com/wimi321/lizzieyzy-next/blob/7b4027531c2b26062d0bfc27a040cc550cfbea4d/src/main/java/featurecat/lizzie/Config.java#L2083-L2096)、[Menu L3750–3779](https://github.com/wimi321/lizzieyzy-next/blob/7b4027531c2b26062d0bfc27a040cc550cfbea4d/src/main/java/featurecat/lizzie/gui/Menu.java#L3750-L3779)、[Input L40–135](https://github.com/wimi321/lizzieyzy-next/blob/7b4027531c2b26062d0bfc27a040cc550cfbea4d/src/main/java/featurecat/lizzie/gui/Input.java#L40-L135) 优先路由HumanSL/setup/score/engine Match；double-click要求left/count2、!trying、!human play、!analysis play、allowDoubleClick且合法board坐标，float-board由其owner路由。不开重复placement、不开analysis，也不绕过Next input/Match authority。源码顺序/默认不是新产品approval门。

#### Acceptance

- fixture断言current no-op、nearest ancestor、continuation优先于稳定rootDFS及上述精确NodePath/重复查询/no-match；原生double-click只沿源current-line范围，显式右键才跨branch，不发明cycle/wrap/list。
- 实现经REVIEW-01选到精确NodePath，相同手数/位置的branch不混淆，no-match不改tree/path/dirty。
- 原生右键/double-click入口遵守board/input/Match守卫，默认true独立持久化与重启读回，drag/click-review无权限串扰；无engine也工作、不启动分析。

#### 最小证据面、继承及原生残余

REVIEW-01 original 88e7b8c3ae42ad3fbb6bd13135c6678b8524256a Windows十root/setup/PL/comment/pass/branch节点只证明exact selection seam；本点搜索顺序/gesture须fixture与原生新证据。

<a id="behavior-a12"></a>
### 主棋盘自动播放间隔

#### 行为与边界

在现有Review Autoplay设置入口编辑main-board interval，registry-owned Ctrl+A/toolbar继续控制同一timer。保留当前800ms兼容默认与UI-05历史accepted toggle范围；不移植ANA-13的500ms/100–5000ms或Javabranch replay0.9秒。Review Autoplay只沿所选continuation导航现有节点，不能生成move/启动引擎，leaf和document/generation/scope change停止，timer最多一个。
设置是draft输入→Validate→persist/apply，Cancel保留runtime/durable值，write/replace failure保留旧间隔并可见失败；重启恢复已成功值。Java main输入为AutoPlay.txtAutoPlayMain/BottomToolbar.txtAutoMain，单位秒；[BottomToolbar.autoPlayMain L4397–4444](https://github.com/wimi321/lizzieyzy-next/blob/7b4027531c2b26062d0bfc27a040cc550cfbea4d/src/main/java/featurecat/lizzie/gui/BottomToolbar.java#L4397-L4444) 普通播放start时 `1000 * Double.parseDouble` 捕获一次，沿既有continuation推进后用该interval sleep；不每gap读新设置。运行中成功保存只更新下次start所用值，不重定时active timer，不引入Variation Replay next-gap语义。源leaf engine continuation重新parse路径不在本能力组范围。
实际durable映射为Java `persistedUi`（JSON `ui-persist`）`toolbar-parameter[40]`：[Config L3361–3380](https://github.com/wimi321/lizzieyzy-next/blob/7b4027531c2b26062d0bfc27a040cc550cfbea4d/src/main/java/featurecat/lizzie/Config.java#L3361-L3380) 用Integer.parseInt，不能parse时存-1；[BottomToolbar L2243–2250 / L2371–2372](https://github.com/wimi321/lizzieyzy-next/blob/7b4027531c2b26062d0bfc27a040cc550cfbea4d/src/main/java/featurecat/lizzie/gui/BottomToolbar.java#L2371-L2372) length51 array只恢复positive index40。runtime可接受小数秒但durable只存整数是明确source mismatch，不是单位/key/live-timing缺失。Next缺字段800ms是既定兼容默认，不把它误称Java整数durable默认。replay-branch-interval-seconds/continue-with-best-move/directly-with-best-move不映射为main interval。

已批准A-AUTOPLAY-FRACTIONAL-PERSISTENCE（已批准滚动规划决定）：按有效计时精度持久保存小数秒，0.8s保存并重启后仍为800ms；不能写-1却报告成功、截断为整数或改成integer-only。新Next存储应保存有效scheduler精度且精确转换，Java ui-persist.toolbar-parameter[40]映射仍列出旧integer限制，不照搬损失。非finite/≤0/overflow/不可安全表示非破坏技术validation，不新增任意产品上限。active timer保持本次Start捕获值，运行中成功保存仅改变下一次Start；Cancel或write/replace失败保留旧runtime/durable值。

#### Acceptance

- 正整数秒通过保存→restart值相同，精确映射ui-persist.toolbar-parameter[40]而非branch setting；缺字段800ms保持。小数秒0.8s保存→restart后精确800ms，拒绝不能保存的非安全表示值，invalid/nonfinite/overflow非破坏。
- 成功值restart持久；invalid/Cancel/write failure保留原runtime/durable值，不多开timer。
- ordinary播放只在start捕获interval；运行中保存不改变当前timer，stop/restart才使用新成功值；controlled clock证明唯一timer及无Variation Replay next-gap混入。
- chosen NodePath continuation确定性前进；leaf/stop/scope replacement关闭timer，旧tick不导航新game；既有UI-05历史范围不重写。

#### 最小证据面、继承及原生残余

以上固定BottomToolbar/Config源码只证明main runtime与durable行为，UI-05原toggle不是新增setting pass。未来owner提供controlled-clock/rendered/persistence fixtures与原生设置/restart/leaf/scope-change；不需要engine，fractional范围按本节已批准精确合同验收。

<a id="behavior-a24"></a>
### 音频设备失败边界调查

#### 行为与边界

重建E50已有review音效、once-only/silence、缺资源与failedprefwrite、mutedrestart证据，以及当前asyncplay rejection catch保持document事实。音频设备/播放失败不否定已接受move、不改用户soundEnabled、不阻塞后续navigation/operation，应是可见非致命错误。
仅真实devicefailure与未证countdown consumer是剩余问题；当前review与D未来match/countdown分列，不因device缺口重实现AcceptedREVIEW-08。若机器/consumer无可达device场景，说明边界和nativegate，不宣称无声卡全部通过。停止于已有场景不变表、真实设备残余/consumerowner和bounded后继，不移植Javadevicecapture内部设计。

#### Acceptance

- E50原候选/Windows场景与当前asynccatch边界重建，已有accepted/rejected/review静音行为不扩大。
- device/play失败不改soundEnabled、accepted move仍成功且后续操作可达；未证设备/倒计时保持精确nativegap。
- D消费者只接收其实际countdown音效边界，不以本review调查声称全部对局声音pass；缺口有具名owner/stop结果。

#### 最小证据面、继承及原生残余

E50cc937c9d2bda0f0d7af513679328ebac774affb1与bc28fc79ea27251997a88f9e000151bf8c1154bf，Windows playback/silence、prefwrite/resourcefailure/mutedrestart；更新isolatedappdata普通/pass/1与3capture、accepted/rejected/review方向静音。调查owner获取实际device门，R15 仅扩展其未来实际 Match/countdown consumer。

<a id="behavior-a25"></a>
### 无引擎实际支持路径调查

#### 行为与边界

保持原noengineopen/edit/comment/history/save/reopen，先查actual entry/default/savedbehavior，不重跑全部UI-04。原七修复中的ownership display15类实际菜单若Next有对应entry应可不需engine修改，engine-only动作明确disabled且不妨碍SGF；不假设15项全已实现。
对offline clear核对KM输入/现clear行为，区分Javaclear保留贴目与Next已批准explicitNew/board-size form用户参数；H-KOMI-EDITOR metadata不证明clear。对noengineANA-11读取SGF已有identity-validanalysis，chart可导航，不制造engine结果。对应source与原run已证明则窄闭合，否则逐子行为具名后继/决定，不能批量Missing。
输出每子behavior source、currententry/default/saved值、原candidate/conditions或缺口，clearKM和offlinechart不得以metadataeditor/架构说明替证据；R13 只接实际 display consumer，不等benchmark/远程功能。

#### Acceptance

- ownership实际supported入口noengine可设置，engine-only拒绝理由可见；unsupported未实现菜单明确列出。
- clearKM与explicitNewform/default区分，noenginechart真实SGFanalysis与exactnavigation，不启动engine/虚造结果。
- 每子行为得到窄原tuple覆盖或具名后继/决定及native责任，原UI-04原范围维持。

#### 最小证据面、继承及原生残余

H-OFFLINE66c906f3117cc1b8274673a0382456b91f02e39f Windows R2 runA/PID51344 golden branchingSGF，C4/B3/comment/deleteA2/SaveAsreopen仅旧workflow；H-KOMI-EDITOR01429efcaf1fa732c1c0f2c2dcf50d367d478ea8仅metadata。调查owner只补本reachable细分native。

### 来源覆盖

唯一责任意图：[T02-REVIEW-04](MIGRATION_SOURCE_MAP.md#t02-review-04), [T02-REVIEW-05](MIGRATION_SOURCE_MAP.md#t02-review-05), [T03-REVIEW08-AUDIO-BOUNDARY-INVESTIGATION](MIGRATION_SOURCE_MAP.md#t03-review08-audio-boundary-investigation), [T04-OFFLINE](MIGRATION_SOURCE_MAP.md#t04-offline)。

来源意图：[T02-REVIEW-04](MIGRATION_SOURCE_MAP.md#t02-review-04), [T02-REVIEW-05](MIGRATION_SOURCE_MAP.md#t02-review-05), [T03-REVIEW08-AUDIO-BOUNDARY-INVESTIGATION](MIGRATION_SOURCE_MAP.md#t03-review08-audio-boundary-investigation), [T04-OFFLINE](MIGRATION_SOURCE_MAP.md#t04-offline)。

Parity IDs（不改原状态）：ANA-11, PREF-01, REVIEW-01, REVIEW-04, REVIEW-05, REVIEW-08, SGF-10, SGF-13, UI-04, UI-05。

Delta IDs：UD-03-050, UD-04-027, UD-04-033, UD-04-039。

<a id="capability-05"></a>
## R11-05 — 所选分支SGF与当前主棋盘图像导出

<a id="behavior-a06"></a>
### 副棋盘图像既有决定

#### 行为与边界

已决：Ticket27 §Answer舍弃Sub-Board专用图像菜单及Shift+S；此为明确可逆产品cut，不需独立ADR才有效。主盘、Sub-Board Variation/Raw与胜率图是三个不同subject；主盘export不证明副盘图像等价，ANA-12显示也不是文件输出。保留该非等价来源/理由，不重复审批、不恢复Shift+S，不削减ANA-04/12或副盘presence。历史要求若将来经批准改变，完整Raw/Variation内容、格式/尺寸/命名/入口合同需由实际改变owner另行冻结，不是本R11隐藏的新实现义务。

批准来源：Ticket27 §Answer（历史 Ticket27；本节保留完整批准合同）

#### Acceptance

- 源目标→具名原resolved Answer/批准理由→当前非等价处置完整；沿用精确已决cut，不重新空开审批。
- 对应现保留功能/入口与本节边界一致；不把近似替代或决定记录当已实施源功能，不扩大删除或Accepted范围。

#### 最小证据面、继承及原生残余

最小面为固定 Inventory census SGF-03-ADJ-SAVE-MORE、02 audit对应历史决定与原处置票；是产品合同/批准证据，不需要当前跑全套或原生来代替审批。若保留，后继实现者拥有受影响原生/文件/持久化验收。

<a id="behavior-a14"></a>
### 分支SGF导出

#### 行为与边界

File → 更多保存 → 保存当前分支及registry-owned Ctrl+Alt+S调用同一export owner。冻结当前选中root-to-leaf continuation：root到selected NodePath的祖先加当前chosen continuation直到leaf，输出standalone main line，不promotion原tree，不夹带siblings。保留该路径的root metadata、setup、move/pass、markup、personal C及identity-valid持久化analysis；遵守SGF serializer和ADR0003，不临时改global raw/comment flags。
目的地与current Save path分离；规范化最终.sgftarget后才检测存在/确认覆盖（已有.SGF不双加后缀），Cancel/拒绝/serialize/write/replace失败保护原目标字节、当前source path/cursor/tree order/dirty。成功只产出文件，不把current game标clean或改变current writable path。临时chooser输入不持久；本能力组不借用Image目录。共享serializer和非破坏文件owner来自既有SGF-06；不要等普通save调查结束才开始已冻结export语义。

#### Acceptance

- 输出重开只有所选root-to-leaf主线，semantic tree/metadata/comment/analysis与冻结路径一致，root或leaf选中有明确路径结果。
- 原tree order/path/cursor/dirty及engine/jobs从调用到成功均不因export改变；取消、存在目标overwrite拒绝及encode/write/replace故障保护目标字节。
- nativechooser输入protected-sgf-中文及game.SGF：检查与写入同一最终target，不生成双后缀；Ctrl+Alt+S/menu走同owner且input-focus安全。

#### 最小证据面、继承及原生残余

H-SAVE-DEPARTURE 4fd710e1c24a991665c2ed47f58bbb178b8c1c82 / R5 48db2b2833f9deb45bd7dcd47181f5da77348a0c只支持原普通save/departure边界，不是本export pass。owner提供branch serializer/file fault fixtures及真实nativechooser/overwrite/Save-reopen。

<a id="behavior-a15"></a>
### 主棋盘图像导出

#### 行为与边界

File 更多保存的保存棋盘图像及registry来源Alt+S冻结调用时当前main-board review view后选择目的地；Shift+S原本是已批准cut的Sub-Board入口，不能串用。不得把whole-window chrome或Sub-Board替代主盘图。输出与调用时实际显示board/stone/coordinate/markup/review overlay一致；currentgame/tree/NodePath/dirty/jobs/review settings不因导出改变。
冻结v1 [Input L506–529](https://github.com/wimi321/lizzieyzy-next/blob/7b4027531c2b26062d0bfc27a040cc550cfbea4d/src/main/java/featurecat/lizzie/gui/Input.java#L506-L529)、[saveMainBoardPicture L10735–10741](https://github.com/wimi321/lizzieyzy-next/blob/7b4027531c2b26062d0bfc27a040cc550cfbea4d/src/main/java/featurecat/lizzie/gui/LizzieFrame.java#L10735-L10741)、[saveImage L10829–10918](https://github.com/wimi321/lizzieyzy-next/blob/7b4027531c2b26062d0bfc27a040cc550cfbea4d/src/main/java/featurecat/lizzie/gui/LizzieFrame.java#L10829-L10918) 给出PNG（默认首filter）、JPG/JPEG、GIF、BMP格式与当前view裁剪；源普通主盘crop尺寸是当时maxSize×maxSize，float-board由独立mainboard surface导出（[L11086–11117](https://github.com/wimi321/lizzieyzy-next/blob/7b4027531c2b26062d0bfc27a040cc550cfbea4d/src/main/java/featurecat/lizzie/gui/LizzieFrame.java#L11086-L11117)）。Next保留current-view输出：调用时mainboard实际rendered surface像素width×height及aspect，不强行固定square或照搬chart1600×600，不新增DPI/size control；实施owner记录renderer到output像素尺寸映射及矩形盘结果。已在该surface内绘制的overlay属于快照；surface外tooltip/hover/chrome不加入，不凭空承诺重新生成隐藏layer。native chooser不预填新命名规则（源没有selected filename），格式选择控制真实encoding/extension，unsupported拒绝不写伪后缀。
与EXPORT-03同一个Recent Image Export Directory：最后成功图像目录；缺失时existing general file-directory preference（Java filesystem.last-folder映射，不新增独立image状态）；缺失/路径不可用时existing native-dialog fallback，不指定开发机路径。原Ticket27 §Answer（历史 Ticket27；本节保留完整批准合同） invocation-time snapshot和success-only durable规则覆盖源dialog后抓图/chooser approve即写last-image-folder的缺陷。仅成功write/confirmed atomic overwrite更新；Cancel/overwrite拒绝/encode/write/replace失败保护target全部bytes、durable目录和game并可见报真实失败。

#### Acceptance

- 主盘menu/Alt+S同owner，PNG/JPG/JPEG/GIF/BMP按format真实decode，default PNG且不发明预填name；output width×height对应调用时rendered mainboard surface（含矩形盘），view内实际overlay包含、外部hover/chrome不加入，不宣称Sub-Board等价。
- 导出像素/语义吻合调用时review snapshot，chooser等待时导航/新数据不混入；没有chrome替代，currentgame/review不变。
- 本consumer原生成功更新唯一durable目录，restart读回；无image目录时用general file-directory preference，缺失/不可用遵existing native-dialog行为。Cancel/overwrite拒绝/encoding/write/replace失败保持target bytes/目录且可见失败；提供唯一writer的独立验证子结果，不以R11-06整票完成为本能力组验收门，真实交替由R11-08拥有。

#### 最小证据面、继承及原生残余

目前disabled export无新native证据。R11-05实现owner提供renderer/snapshot/image-decode/file-fault/preferences fixtures与nativechooser/overwrite/目录restart；同一真实主盘消费者唯一建设共享imagefolder/schema/atomic writer。子结果完成记录必须列唯一状态/接口、成功才更新及fallback规则、文件故障/Cancel/restart证据、实际containing SHA及所需完整证据材料清单；具名串行集成 owner提前验证合入后释放R11-06接线，不等待R11-08最终验收。

### 来源覆盖

唯一责任意图：[T02-H13](MIGRATION_SOURCE_MAP.md#t02-h13), [T02-EXPORT-01](MIGRATION_SOURCE_MAP.md#t02-export-01), [T02-EXPORT-02](MIGRATION_SOURCE_MAP.md#t02-export-02)。

来源意图：[T02-H13](MIGRATION_SOURCE_MAP.md#t02-h13), [T02-EXPORT-01](MIGRATION_SOURCE_MAP.md#t02-export-01), [T02-EXPORT-02](MIGRATION_SOURCE_MAP.md#t02-export-02)。

Parity IDs（不改原状态）：ANA-12, EXPORT-01, EXPORT-02, EXPORT-03, SGF-06, UI-01。

Delta IDs：无新增 Delta ID。

<a id="capability-06"></a>
## R11-06 — 胜率图可读性、未分析节点导航与PNG导出

<a id="behavior-a16"></a>
### 胜率图PNG导出

#### 行为与边界

至少一个identity-valid selected-line point时，File → 更多保存 → 保存胜率图截图及registry-owned Shift+Alt+S冻结当前ANA-11所选line、encoding/perspective、series visibility、gaps、axis/scale、current marker和Blunder Bar，再打开native destination dialog。无data拒绝可见，不开chooser、不启动analysis；score absence与stale point不伪造数据。
重画1600×600 PNG：包含可见胜率/目差series、Blunder Bar、axes/scales、current marker和固定perspective/series标签，不含hover/application chrome，ADR0001整条single-series perspective不改。dialog prefill精确为 `<sgf-stem>-winrate-m<selectedMove>.png`；无source SGF时字面值 `untitled-winrate-m<selectedMove>.png`（原Ticket27 §Answer（历史 Ticket27；本节保留完整批准合同）），不采用localized/application untitled stem。chooser等待期间新分析/节点变化不改冻结payload。仅成功write更新共享Recent Image Export Directory（Java last-image-folder映射）；初始目录是最后成功image目录，缺失时existing general file-directory preference（Java last-folder映射，不新增独立image状态），缺失/路径不可用时existing native-dialog fallback，不指定开发机路径。确认overwrite必须atomic；Cancel/overwrite拒绝/encode/write/replace失败保护已有target bytes、目录、currentgame、jobs及chart prefs并可见报错。不调用普通Save，不改变source path/dirty/selection。
共享imagefolder/schema/atomic writer唯一由R11-05建设；本能力组复用该同一状态和接口，不另建writer。共享图像文件接线前消费集成 owner 已释放的R11-05子结果及其完整containing SHA/验证证据/所需完整证据材料；独立chart编码及快照工作不等待R11-05 整能力组。两消费者真实交替完成验收由R11-08拥有。

#### Acceptance

- menu/Shift+Alt+S相同owner，no-data/stale-only拒绝不分析；一个valid点即可导出，数据缺口保留。
- PNG decode确认1600×600及调用时line/perspective/visible series/gaps/axes/marker/bar/固定label，无hover/chrome，不因chooser后变化混入新数据。
- native精确预填 `<sgf-stem>-winrate-m<selectedMove>.png` / `untitled-winrate-m<selectedMove>.png`；atomic overwrite success更新唯一Image目录，Cancel/overwrite拒绝/encode/write/replace failure保持原bytes/目录/game/jobs/settings且可见失败。
- registry/editable-input排除与本consumer共享目录restart/初始fallback/成功失败fixtures及native证据交付；共享接线已消费R11-05 writer子结果记录，不等待R11-05整票或本能力组readability子行为全部完成。真实R11-05/R11-06交替success/cancel/failure/restart集成仅R11-08拥有，不作为本能力组完成门。
- 保留原Ticket27 §Evidence and promotion boundary（历史 Ticket27；本节保留完整批准合同）明确的EXPORT-03强门：Native smoke on every Shipped Platform，逐平台实际菜单/Shift+Alt+S、save dialog、overwrite确认、成功PNG重开和可见失败报告；记录实际平台集合/候选/条件，仓库fixture不替代该门。

#### 最小证据面、继承及原生残余

ANA-11原native图encoding只继承原范围，完整SHA/条件重建为本能力组behavior-a21；本新增PNG无历史pass。R11-06 owner提供encoder/snapshot/filefault/rendered/registry fixtures及真实native destination/overwrite/failure/目录restart，平台范围遵循本behavior所引Ticket27 §Evidence and promotion boundary的每Shipped Platform强门；该范围不自动扩为behavior-a19/readability或a21点击/drag的全平台矩阵。

<a id="behavior-a19"></a>
### 目差及基线可读性

#### 行为与边界

保留ANA-11既有编码/defaults/gaps/score fallback与ADR0001整条single-series perspective，仅补readability successor。按e6ad02af fixed tests，Black-frame score +7.3→B+7.3、-4.6→W+4.6；+0/-0/±0.04→0.0无leader/negativezero，+0.05→B+0.1、-0.05→W+0.1。当前selected-side perspective需从被编码series正确反推出实际leader，不能显示与整条视角冲突的color/符号。
基线可辨：只有winrate可render显示50%，只有score可render显示0，两项均render不附误导单指标mark；score未有效而winrate可render仍50%，无renderable metrics无baseline/mark。baseline突出而ordinary midline不再重复叠绘。正score在zero上方、负在下方；current/hover/endpoints标签和baseline mark同时可读不遮挡。B+/W+与语义标签用资源键，各维护locale的已提供资源一致可读，未完成语言仍按foundation确定fallback；R17拥有全部翻译名单闭环，不在A偷删语言目标。
无新增chart产品开关/default变化或persist影响，hover不导航，不从missing/stale数据造零值。本能力组behavior-a16导出合同不以readability完成作前置；若实现共同renderer变更，由R11-08核对快照labels不漂移。

#### Acceptance

- source格式例覆盖±7.3/4.6、±0/0.04/0.05；实际leader/perspective一致，无negativezero，absent不是0。
- renderable metrics矩阵得到上述baseline/mark结果，零基线可辨且current/hover/endpoint labels不覆盖彼此或baseline。
- resource-key labels在维护locale/fallback可读，原生实际chart观察与编码fixtures一致，原ANA-11settings/dirty/jobs不变。

#### 最小证据面、继承及原生残余

WinrateGraphScoreLeadDisplayTest.java@e6ad02af0454413512bce6f2102f0c66e8424144 source已读取，例值为source非新阈值。最小fixture按真实标签碰撞风险选择，不新增任意窗口尺寸矩阵；native chart视觉/readability owner取得。原ANA-11完整native tuple仍A21重建，不虚称本后继已pass。

<a id="behavior-a21"></a>
### 未分析图导航证据与gesture差异

#### 行为与边界

仅重建原ANA-11 native完整candidate/条件并核对chart点击未分析节点的selected-line exact identity；基线按node序列点击已有可达实现，不能把全部导航判Missing。比较Java源drag/click与Next已有click，分列输入gesture覆盖，不把clicked analyzed point证据当empty point证据。
无分析值节点仍可导航到精确NodePath/branch，不造winrate/score/visits、不自动engine，tree/dirty不因navigation改变。找到原记录时仅继承其中明确点击scope；没有完整SHA或该gesture场景仍保留未来native gate。输出click窄证据tuple和drag差异/具名后继，不自己发明drag产品语义或实现hypotheticalbug。

#### Acceptance

- 重建原ANA-11候选/平台/分析载体/branch/gesture确切条件，查不到则明示缺记录并保留native gate，不用planningHEAD代替。
- 对未分析selected-line节点点击核对exactNodePath/branch与no-analysis/noengine；已有点击不被错误判全Missing。
- Java drag/Next click差异表和未证拖动的具名后继contract/owner齐备，UD-03-021不重复创造ParityID或扩大ANA-11。

#### 最小证据面、继承及原生残余

Matrix ANA-11给原Windows branch native但audit明确未定位本scene完整tuple；本能力组就是重建门。owner取得原记录或有界当前native点击证据，drags未跑保持缺口。

### 来源覆盖

唯一责任意图：[T02-EXPORT-03](MIGRATION_SOURCE_MAP.md#t02-export-03), [T03-ANA11-SCORE-LEAD-DISPLAY](MIGRATION_SOURCE_MAP.md#t03-ana11-score-lead-display), [T03-ANA11-UNANALYZED-SCRUBBING](MIGRATION_SOURCE_MAP.md#t03-ana11-unanalyzed-scrubbing)。

来源意图：[T02-EXPORT-03](MIGRATION_SOURCE_MAP.md#t02-export-03), [T03-ANA11-SCORE-LEAD-DISPLAY](MIGRATION_SOURCE_MAP.md#t03-ana11-score-lead-display), [T03-ANA11-UNANALYZED-SCRUBBING](MIGRATION_SOURCE_MAP.md#t03-ana11-unanalyzed-scrubbing)。

Parity IDs（不改原状态）：ANA-11, APP-05, EXPORT-03, REVIEW-01, REVIEW-10。

Delta IDs：UD-03-013, UD-03-021。

<a id="capability-07"></a>
## R11-07 — 保持引擎身份的档案排序

<a id="behavior-a17"></a>
### 引擎档案排序

#### 行为与边界

Engine Settings在保存的profile catalog行提供置首/上移/下移/置尾，按stable profile ID原子持久化顺序。边界置首/上移、末尾下移/置尾为不改变顺序的disabled/no-op；当前selected ID不因index改变，Autoload Default ID、active run、pending edits、job binding、loaded launch config都不变，不stop/restart/switch引擎。
无新排序默认：首次沿现catalog order；正常restart用成功durableorder。失败/拒绝（stale catalog、unknown/duplicate IDs、不完整集合、storage failure）保留原durable order并可见错误，不能先写局部丢失profile。Java映射随交付：MoreEngines move controls与leelaz.engine-settings-list[]顺序；Java ui.default-engine index仅映射stableDefault identity，不能把index当Next run ID或重新解释autoload值。新排序UI使用R11-01 locale foundation 结果，但不依赖benchmark/threads/remote 或其他 R11 settings。

#### Acceptance

- 四方向对stable IDs持久，restart恢复；边界及stale/invalid完整集合非破坏。
- selection、default ID、activeRun、pending edits及job profile bindings在成功/失败排序均不变，无隐式engine lifecycle操作。
- 故障注入persist/replace失败后durable/runtime展示一致旧顺序，原生settings排序/restart证据与Java字段映射齐备。

#### 最小证据面、继承及原生残余

既有ENG-01/09 catalog/autoload seam与H-SWITCH90508df1d74e94d11036a8955707f8399927655e仅证明旧lifecycle，not排序。owner提供catalog/persistence/rendered fixtures；native选择/default/pending edits/Run identity保护，若声称真实Run不变则用兼容既有KataGo实际run，无引擎编译要求。

### 来源覆盖

唯一责任意图：[T02-ENG-08](MIGRATION_SOURCE_MAP.md#t02-eng-08)。

来源意图：[T02-ENG-08](MIGRATION_SOURCE_MAP.md#t02-eng-08)。

Parity IDs（不改原状态）：ENG-01, ENG-08, ENG-09。

Delta IDs：无新增 Delta ID。

<a id="integration-acceptance"></a>
## R11-08 — R11共享状态集成与实际跨功能验收

本集成结果等待七能力组的已验证已集成最终成果；已交付子集可以先验，完整完成仍受全部真实结果门约束。

<a id="behavior-a22"></a>
### 实际消费者响应性调查

#### 行为与边界

唯一问题：当前受支持analysis publication、PROV-03同步和已可达设置是否同步阻塞UI或无限排队？按实际owner/事件面描述bounded workload来源、payload边界、queue/cancel/progress可观察结果；固定原source的Windows场景，而非凭Rust/WebView推断不卡顿或固定500手阈值。只有实际supported入口调查，未实现自动设置等把响应性验收留在其B功能票，不将missing功能当当前bug。
大analysis payload与live同步交互要保持cancel/进度/有效input可达，旧identity不能堆积后publish新game。每路径记录source/原candidate/条件、是否同因适用/原窄证据/具名残余；没有实测latency不发明ms预算。停止于覆盖表与必要repro/明确native门，不在此调查复制SwingEDT机制或实现抽象background平台。
R11拥有总体调查结果；R13负责analysis/setup实际consumer，R14负责PROV-03真实服务消费者native evidence。独立本地入口先完成；完整线上结论需要C现PROV-03::functional tuple/对应新native observation，而不是等待C所有新远程feature。

#### Acceptance

- analysis/current设置/PROV-03各路径有可达性、bounded工负载、cancel/progress/队列结论；未实现设置转其owner而不批量Missing。
- Windows真实native响应性不足的路径留精确环境/证据责任，无法以browser/fixture/Rust所有权替代。
- 输出同因适用性、原窄证据和具名残余/后继，只有有observedrepro才授权另票修复。

#### 最小证据面、继承及原生残余

PROV-03::functional141dd0e47ef973b4a141b46b7283d8f408cd283b Windows Yike79496703 129moves读同步/Save/Error/Retry/Stop；初cd8ae4fa50a584589daf1b673aaecf7ce5433a21，只可继承原功能不替compoundperformance。owner协调B/C actualconsumer artifact，不等待全批。

<a id="behavior-a30"></a>
### 跨功能集成验收

#### 行为与边界

本能力组是实际产品集成验收owner，不是read-onlyCloseout，也不是本轮planning完成声称。先消费R11-01–R11-07最终completion/artifact、已决历史处置记录/明确未过验收门与所选功能原candidate tuple，由具名集成 owner在独立integration包含所有实际前置finalcommits后冻结SHA；不从共同旧baseline假设新实现存在。
限定共享边界的端到端组合：branchedgame edit/transform→Undo/Redo→ordinarySave/reopen与selected-branchexport→chart/main-boardimage snapshot；export不改currentpath/dirty，chooser等待时navigation/analysis/新edit与snapshot不混，失败不破坏原file。共享Image目录在两consumer成功交错/restart保持唯一且失败不推进；resourcekeys/locale fallback只有一套。sorting在pendingprofileedits/activeRun时只改order；autoplayscope/scrub/pointsearch/komifocus/searchcancel与registryeditable-focus共享不越权、不多开timer、不夺新owner；旧job/documentgeneration不能污染替换新谱。具体组合按实际已交付consumer选，不列无关所有尺寸/platform/repetition矩阵。
本能力组独占两真实图像consumer原生交替完成验收：先R11-05主盘成功到directory X，再R11-06 chart native chooser从X起并成功到Y，反向R11-05从Y起；restart两者仍同一最后成功目录。穿插Cancel、overwrite拒绝、encode/write/atomic-replace failure不推进directory、不改已有target bytes/game/jobs/settings；image目录缺失时两者同消费existing general file-directory preference，缺失/不可用同遵existing native-dialog fallback。R11-05/06各自完成只需本consumer及唯一R11-05 writer子结果验收；该子结果已由具名串行集成 owner提前验证合入并释放，不以本能力组最终七组验收自我阻塞。
实际nativegap或新证据缺口不借Closeout认通过；可先做已交付组合，但R11 integration整体completion保留真实terminalblockers，批准的非等价决定只关闭对应goal不伪claim功能passed。所有新property/default/settings entry有本能力组消费验证及owner记录，R17 139-entry全跨功能与finalupstream仍由 R17 拥有。

#### Acceptance

- 从所有实际R11 capability owner最终成果选择verified-containingSHA，多个未合前置由具名集成 owner独立集成；decision/native未解表不被默默当通过。
- currentgame/export/Save/Undo/lateanalysis/snapshot组合有实际原生/文件结果；真实R11-05→R11-06→R11-05成功、反向起始目录与restart保持唯一success-onlyImage目录，交错Cancel/overwrite拒绝/encode/write/replace失败保持directory/target bytes，两个consumer初始general-directory/native fallback一致，非空壳接线检查。
- sorting selection/default/Run/pendingedit、autoplaytimer、searchfocus/registry/komi、resourcefallback边界按真实changedconsumer验收，不启动非needed工作。
- 每claim分repository/controlled/native及原fullSHA/条件/invalidations；安装态与engine编译义务不冒充，本能力组不替R17综合139入口或read-onlyCloseout。

#### 最小证据面、继承及原生残余

继承H-OFFLINE66c906f3117cc1b8274673a0382456b91f02e39f Windows noengine；H-SAVE-DEPARTURE4fd710e1c24a991665c2ed47f58bbb178b8c1c82/R5 48db2b2833f9deb45bd7dcd47181f5da77348a0c Windowsdeparture；H-KOMI-EDITOR01429efcaf1fa732c1c0f2c2dcf50d367d478ea8 Windows/KataGo旧editor；E50原audio两完整SHA仅原范围。新增组合由本integrationowner取得native/currentfile/qualifiedengine实际必要证据，不重跑无关原生matrix。

#### 来源验收继承表（原条件与剩余集成缺口）

| 输入与来源 | 继承条件 / 仍须取得的证据 | 本能力组责任边界 |
| --- | --- | --- |
| R11-06::behavior-a16 / 原Ticket27 §Evidence and promotion boundary（历史 Ticket27；本节保留完整批准合同） | EXPORT-03原准入明确Native smoke on every Shipped Platform：菜单/Shift+Alt+S、save dialog、overwrite确认、PNG重开、可见失败；消费06实际平台集合、完整candidate及上述证据，变化失效须重验。 | 保留该强门并补两image消费者共享目录/故障/快照交替实际缺口；不重复未变前置smoke，不将该强门套到全部R11行为。 |
| R11-01/02/03/04/05各behavior原生focus/gesture/file/chooser边界 | 仅按对应原审计来源、记录平台/窗口/文件条件和实际风险消费；fixture/浏览器不替真实激活、文件或设备面，原Windows tuple不自动外推。 | 验证实际共享焦点/谱状态/文件与timer新组合，按改变的边界补证，不列全平台尺寸/重复矩阵。 |
| R11-07排序与R11-03 ordinary sameRun意图 | 声称Run/analysis持续的部分须兼容合格真实KataGo及确切Run identity/先前running或paused条件；原catalog/mock与departure记录不替新论断。 | 只补排序/保存共享Run状态缺口，不重验引擎全生命周期或编译上游引擎。 |
| behavior-a22 / PROV-03::functional141dd0e47ef973b4a141b46b7283d8f408cd283b | 原Windows/Yike79496703/129moves只证明既有读同步/Save/Error/Retry/Stop；新live provider响应性论断仍需真实当前服务/原生交互及版本条件，payload fixture仅证明输入面。 | 现有本地路径先行；真实线上结论保持对应环境门，不等R14整阶段，也不以原tuple假称compound performance通过。 |

### 来源覆盖

唯一责任意图：[T03-PERF-BACKGROUND-OFFLOAD](MIGRATION_SOURCE_MAP.md#t03-perf-background-offload), [T06-A-INTEGRATION](MIGRATION_SOURCE_MAP.md#t06-a-integration)。

来源意图：[T03-PERF-BACKGROUND-OFFLOAD](MIGRATION_SOURCE_MAP.md#t03-perf-background-offload), [T06-A-INTEGRATION](MIGRATION_SOURCE_MAP.md#t06-a-integration)。

Parity IDs（不改原状态）：ENG-08, EXPORT-01, EXPORT-02, EXPORT-03, PROV-01, PROV-03, REVIEW-04, REVIEW-05, REVIEW-06, REVIEW-10, SGF-01, SGF-15, SGF-16, SGF-17, UI-02, UI-07。

Delta IDs：UD-03-023。

<a id="read-only-closeout"></a>
## R11-09 — 只读 Closeout

开始条件：R11-08 实际集成验收通过；统一最终 Spec/Standards 双轴和集中修复成功；全部适用 native/真实环境验收通过。Closeout 不在触发最终审查的实施 barrier 内，不要求 tracker 提前关闭，也不占 31 owner/35 source 中任何意图。

### 输入

R11-01、R11-02、R11-03、R11-04、R11-05、R11-06、R11-07、R11-08全部Completion records（含非终端前置）、每个记录的实际candidate/条件及未过门；R11 spec；root共享最终审查repository/base/final target、覆盖01–08的成功verdict和Follow-up ledger或明确zero-follow-up verdict。共享审查引用不代表每票单独审过。

### 只读边界

只读最终HEAD、完成/审查记录、既有ledger及其引用证据。不改代码，不做broad review、不重开原spec、不创建follow-up票、不关闭tracker。缺少任一实现/验收记录或完整final review target/coverage/verdict即阻塞。不会把native不可用或功能差异认作通过。每个distinct review ledger消费一次，ledger内部stable finding ID去重；不同ledger相同ID保持独立。统一结果与原历史source routes都保留。

### Acceptance

- 八个输入的实际完成记录和同一最终审查覆盖/target/成功verdict齐备，不伪造逐票review；全部适用验收由R11-08/feature owners满足。
- 既有票ownership已核对；在最终HEAD已修复、重复或无效finding用记录证据核对，每个distinct finding只消费一次。
- 剩余FIX及DIAGNOSE候选连冻结handoff呈用户选择，或明确`No follow-up candidates`；没有新票/新实现/新审查/外部关闭。

## 4. 阶段验收与证据继承

所有七能力组及 R11-08 实际集成完成后，统一最终 Spec/Standards 双轴审查与集中修复；不逐组重复最终全量审查。每项 claim 区分 repository、controlled、native、真实 engine/service 证据及未过门，记录实际起点/最终完整 SHA、消费结果及材料、候选和运行条件；规划准备度不是产品验收。继承须绑定原完整候选、行为范围、平台/窗口/文件、引擎/服务/资源版本和未变边界。行为、DTO、协议、身份、资源、平台或服务变化由对应 owner 重验；缺环境仍阻塞对应验收，不由共同 HEAD 或 Closeout 代认通过。

原生/真实环境门按各 behavior 的已批准来源和可达风险限定：

- R11-01：当前受支持原生窗口 actual activation、exact-target、取消恢复和 late-focus；当前构建 About、焦点安全快捷键和真实人机菜单可达性。shell 已决处置消费批准记录，不以全平台运行替审批。
- R11-02：原生 edit/drag/transform/征子、Undo/Redo、Save/reopen；树/规则 fixture 不能替 gesture/文件面。工具不依赖引擎。
- R11-03：本地/剪贴板导入、DT Open→metadata edit→Save/reopen、Save chooser/overwrite/failure/off-UI 写盘；普通 sameRun running-analysis 持续论断须兼容合格真实引擎，departure 证据不替该场景。provider payload fixture 只证明输入语义。
- R11-04：右键/double-click、独立权限持久化/restart，Review Autoplay 设置/restart/scope-change；真实 audio device 失败保持具名残余，不用 async catch 或 fixture 外推硬件通过，R15 countdown 分列。
- R11-05：实际菜单/快捷键、chooser/overwrite/可见失败、分支 SGF 和各真实图像 decode/reopen、唯一 writer 成功/失败/目录 restart。只按 a14/a15 来源范围，不套 chart 专属每平台门。
- R11-06：**EXPORT-03 保留历史 Ticket27 §Evidence and promotion boundary 的 Native smoke on every Shipped Platform**：实际菜单/Shift+Alt+S、save dialog、overwrite 确认、PNG 重开、可见失败，记录实际平台集合/候选/条件。a19 readability 与 a21 点击/drag 按各自来源和原 Windows tuple，不由 PNG 强门扩成全部 chart 手势全平台矩阵。
- R11-07：原生 settings 排序/restart，selection/Default/pending edits/Run 身份保护；声称真实 Run 不变须兼容合格实际 KataGo Run。
- R11-08：只补实际共享状态改变的集成缺口，独占两个真实图像 consumer 的交替成功/反向目录/restart/交错故障验收。live provider 新响应性论断须真实当前服务/原生交互；旧 PROV-03 tuple 不替 compound performance。

浏览器不是原生，fixture 不是真实服务；既有 Windows tuple 不自动外推。新增平台或组合门须具体来源/风险理由，不添加任意窗口尺寸、重复次数或 Cartesian 矩阵。无安装包、生产发布或上游 engine build 前置；安装态责任留 R18。调查交付有界结论、窄证据、具名可达差异和后继验收 owner，不据此授权推测性修复或新功能。

## 5. 后续阶段与公共来源

[R12](R12_PLAN.md) · [R13](R13_PLAN.md) · [R14](R14_PLAN.md) · [R15](R15_PLAN.md) · [R16](R16_PLAN.md) · [R17](R17_PLAN.md) · [R18](R18_PLAN.md) 保留滚动阶段范围，开始前按实际前置细化并取得相应批准。公共来源合同见 [MIGRATION_SOURCE_MAP](MIGRATION_SOURCE_MAP.md) 与 [后续来源合同附录](MIGRATION_CONTRACTS.md)，实际唯一责任、scoped results 与 named gates 见 [MIGRATION_ROUTES](MIGRATION_ROUTES.json)。
