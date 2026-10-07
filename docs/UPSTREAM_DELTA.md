# Upstream Delta Ledger

本台账是固定上游增量的来源与处置权威。Matrix拥有状态/验收，Inventory拥有v1能力/入口，Plan拥有R阶段；这里不提升任何产品状态。

## 冻结范围与使用规则

- Tauri调查/实施基线：`55795fab49b80a681a9fa544950e18f5f2da4cc6`，仓库 `qiyi71w/lizzieyzy-next-tauri`。
- Java主源：[wimi321/lizzieyzy-next](https://github.com/wimi321/lizzieyzy-next)，Migration Baseline v1 `7b4027531c2b26062d0bfc27a040cc550cfbea4d`不移动；本轮终点`af0e07a7386483f3bfc8a15780de72ffc2f0de4c`。
- 三段可达差集231/197/145互斥，并集573；包含合并支线。90/38/17第一父事件是来源核验数，不是功能数。
- 148个分片来源记录归40个语义Delta族（UDX）；一族可含多个有实际依赖的任务。相同用户目标只由一个责任意图/后继项承接；来源修复、merge和测试不重复计功能。复合来源中的其他目标由逐记录任务引用交叉归属，不能仅看族标题删去。
- 每个分片ID保留为源记录锚点；组内原始验收条款共同约束后续任务，不以族摘要替代。T01–T05与[冻结分配表](../.scratch/issue18-function-first/evidence/06-delivery-map.md)保留审计输入、责任键和意图来源；当前[阶段索引](../.scratch/issue18-function-first/replanned/stages/index.json)按 owner_routes 逐意图分配，07只细化[R11草案组](../.scratch/issue18-function-first/replanned/R11/)，08保留R12–R18完整阶段计划。忽略草案材料已实体集成、仍未发布，新R11票单须另经用户批准；历史草案不是第二套活跃tracker，也不是正式实施票。
- 源记录引用的Java文件均按其完整source commit读取，不采用维护线当前工作区。E/H证据别名见文末；源码支持、仓库测试、受控smoke、原生/真实引擎服务分列，新增目标未取得运行证据即为缺口。
- 分片原文中的章节号、“06分配”、Txx与A–F/旧R11建议完整保留为来源语境。历史A→R11、B→R12/R13、C→R14、D→R15、E→R16、F→R17、未执行旧R11发行→R18；R0–R10证据不改归属。混合族标题/阶段列只导航，不将整族机械划给R12或R13；唯一责任owner、真实前置和其他阶段消费结果均查意图索引。手动线程/只读PDA-WRN/运行恢复归R12，benchmark/saved-policy/分析任务展示归R13；startup消费者可引用独立benchmark结果，不创建整阶段依赖。没有第二套接受状态。

## 发布事实

以下API事实冻结于本轮审计；tag/preparation标题不证明发布。正式4个、预发布8个；10-07.1为draft，published_at=null，不计公开版本。源索引列出实际包含该源的全部已发布版本。

| Release | 类型 | 完整tag target |
| --- | --- | --- |
| [next-2026-08-30.1](https://github.com/wimi321/lizzieyzy-next/releases/tag/next-2026-08-30.1) | prerelease | `aa77b7f437aab9603baf2a066d7614027ed3336a` |
| [next-2026-08-31.1](https://github.com/wimi321/lizzieyzy-next/releases/tag/next-2026-08-31.1) | prerelease | `f533b9109195a8f610276b1179dfa460979c0ed1` |
| [next-2026-08-31.2](https://github.com/wimi321/lizzieyzy-next/releases/tag/next-2026-08-31.2) | prerelease | `fbac79e5ef839f656bafc70ded3cab119842d141` |
| [next-2026-09-01.1](https://github.com/wimi321/lizzieyzy-next/releases/tag/next-2026-09-01.1) | prerelease | `cecbc4328b7f4cce2b47a77910949f64c7df9ac5` |
| [next-2026-09-03.1](https://github.com/wimi321/lizzieyzy-next/releases/tag/next-2026-09-03.1) | prerelease | `de4855ea5902d4871207c42ba2e2e52cb465d9ff` |
| [next-2026-09-04.1](https://github.com/wimi321/lizzieyzy-next/releases/tag/next-2026-09-04.1) | formal | `e23cf300ae65ce0729c0fc589c52492c89961b6c` |
| [next-2026-09-13.2](https://github.com/wimi321/lizzieyzy-next/releases/tag/next-2026-09-13.2) | formal | `c1857c2ea017ef446105c6446ea1e555978c912c` |
| [next-2026-09-18.2](https://github.com/wimi321/lizzieyzy-next/releases/tag/next-2026-09-18.2) | prerelease | `5930a09d979daac41eb0451dcd6fa8ccf5f2db1f` |
| [next-2026-09-22.2](https://github.com/wimi321/lizzieyzy-next/releases/tag/next-2026-09-22.2) | prerelease | `8e6b76bd54c6a6927214bccb258ae86b33788603` |
| [next-2026-09-26.1](https://github.com/wimi321/lizzieyzy-next/releases/tag/next-2026-09-26.1) | formal | `50898233f5cce7d952b37cdb9ee005f780ea948f` |
| [next-2026-09-26.2](https://github.com/wimi321/lizzieyzy-next/releases/tag/next-2026-09-26.2) | formal | `3f58c0b62160bb0e46f7e9765da100371bb78b2d` |
| [next-2026-10-01.1](https://github.com/wimi321/lizzieyzy-next/releases/tag/next-2026-10-01.1) | prerelease | `c013d8ce66afc186df5af96c83a61cbb3e4a14ba` |

## 语义Delta索引

| Delta | 用户目标 | 当前R阶段导航 / 历史来源组 | 条目/后继（状态归Matrix） | 分片来源 |
| --- | --- | --- | --- | --- |
| [UDX-001](#udx-001) | 必要诊断、脱敏与有界导出 | R12/R13（逐意图）；来源B | REL-09 | UD-03-001, UD-03-002, UD-03-003, UD-03-004, UD-03-005, UD-03-019, UD-03-025, UD-03-030, UD-03-031, UD-03-042, UD-03-052, UD-04-040, UD-04-046, UD-05-08, UD-05-10 |
| [UDX-002](#udx-002) | 本地运行资源、来源与模型兼容 | R12/R13（逐意图）；来源B | ENG-11 | UD-03-006, UD-03-008, UD-03-010, UD-03-010a, UD-03-035, UD-03-035a, UD-03-037, UD-03-039, UD-04-001, UD-04-002, UD-04-006, UD-04-022, UD-04-026, UD-04-030, UD-04-031, UD-04-037, UD-04-038, UD-04-043, UD-05-05, UD-05-27 |
| [UDX-003](#udx-003) | HumanSL独立AI Coach与恢复 | R15；来源D | GAME-08 | UD-03-007, UD-03-009, UD-04-012, UD-04-015, UD-04-017 |
| [UDX-004](#udx-004) | 现有Run/reader/取消/重启可达性调查 | R12/R13（逐意图）；来源B | ENG-02, ENG-04 | UD-03-011, UD-03-018, UD-03-045, UD-03-046, UD-04-029 |
| [UDX-005](#udx-005) | AI解说连接、依据与工作区 | R16；来源E | AI-01 | UD-03-012, UD-05-28, UD-05-29, UD-05-30 |
| [UDX-006](#udx-006) | 目差显示与未分析节点图导航 | R11；来源A | REVIEW-10 | UD-03-013, UD-03-021 |
| [UDX-007](#udx-007) | 换谱与后台响应性调查 | R11；来源A | SGF-07, UI-02 | UD-03-014, UD-03-023 |
| [UDX-008](#udx-008) | 独立benchmark与性能反馈 | R12/R13（逐意图）；来源B | ENG-14 | UD-03-015, UD-03-017, UD-03-028 |
| [UDX-009](#udx-009) | 远程算力的本地隔离与配置体验 | R14；来源C | RCOMP-01 | UD-03-016, UD-03-033, UD-05-02 |
| [UDX-010](#udx-010) | 连续预算历史范围继承 | R12/R13（逐意图）；来源B | ANA-06 | UD-03-020 |
| [UDX-011](#udx-011) | 自动快析、交还与本地Restart调查 | R12/R13（逐意图）；来源B | ANA-19 | UD-03-022, UD-03-024, UD-03-041, UD-03-051, UD-04-004, UD-05-17, UD-05-25 |
| [UDX-012](#udx-012) | Match既有编辑守卫继承 | R15；来源D | GAME-01 | UD-03-026 |
| [UDX-013](#udx-013) | 现有Match流式终态屏障调查 | R12/R13（逐意图）；来源B | GAME-01, ENG-10 | UD-03-027 |
| [UDX-014](#udx-014) | 普通规则确认与Match冻结规则生命周期 | R15；来源D | GAME-11, GAME-13 | UD-03-029, UD-04-034, UD-05-07 |
| [UDX-015](#udx-015) | 普通本地位置/规则/reader精确恢复调查 | R12/R13（逐意图）；来源B | ENG-02, ENG-09 | UD-03-032, UD-03-038, UD-03-044, UD-04-019, UD-04-020, UD-05-13 |
| [UDX-016](#udx-016) | 只读同步退休与迟到帧调查 | R14；来源C | READ-01, READ-02 | UD-03-034, UD-04-041, UD-05-06 |
| [UDX-017](#udx-017) | 保存条目的线程来源与动态读回 | R12/R13（逐意图）；来源B | ENG-12, ENG-13 | UD-03-036, UD-03-053, UD-04-048, UD-05-09 |
| [UDX-018](#udx-018) | 全局离线功能搜索与精确焦点 | R11；来源A | UI-07 | UD-03-040, UD-03-040a, UD-03-043, UD-03-047 |
| [UDX-019](#udx-019) | 有界可响应引擎console | R12/R13（逐意图）；来源B | UI-08 | UD-03-048 |
| [UDX-020](#udx-020) | 棋局详情焦点与窗口归属 | R11；来源A | SGF-13 | UD-03-049 |
| [UDX-021](#udx-021) | 音频失败隔离调查 | R11；来源A | REVIEW-08 | UD-03-050 |
| [UDX-022](#udx-022) | 同树用户重点分析 | R12/R13（逐意图）；来源B | ANA-17 | UD-03-054, UD-03-054a, UD-03-054b, UD-03-054c, UD-03-054d, UD-03-054e, UD-03-054f |
| [UDX-023](#udx-023) | 上游内部mechanics与发布来源排除 | R18；来源旧R11发行 | — | UD-03-EXC-001, UD-03-EXC-002, UD-03-EXC-003, UD-03-EXC-004, UD-03-EXC-005, UD-03-EXC-006, UD-03-EXC-007, UD-04-003, UD-04-005, UD-04-007, UD-04-008, UD-04-010, UD-04-014, UD-04-016, UD-04-018, UD-04-021, UD-04-023, UD-04-024, UD-04-025, UD-04-028, UD-04-035, UD-05-01, UD-05-04 |
| [UDX-024](#udx-024) | 启动自检产品决定及measured tuning | R12/R13（逐意图）；来源B | ENG-15 | UD-04-009, UD-04-013, UD-04-042, UD-04-047 |
| [UDX-025](#udx-025) | 工作区可访问性与主题/字体目标 | R17；来源F | UI-02, APPEAR-01 | UD-04-011, UD-05-03 |
| [UDX-026](#udx-026) | 全树坐标/色彩变换 | R11；来源A | SGF-16 | UD-04-027 |
| [UDX-027](#udx-027) | 多文件分析准入与会话队列 | R12/R13（逐意图）；来源B | ANA-07 | UD-04-032 |
| [UDX-028](#udx-028) | 双参与者runtime-komi事务 | R15；来源D | GAME-12 | UD-04-033 |
| [UDX-029](#udx-029) | No-engine支持路径调查 | R11；来源A | UI-04, SGF-13 | UD-04-036 |
| [UDX-030](#udx-030) | 保存目标/快照/原子替换 | R11；来源A | SGF-17 | UD-04-039, UD-04-045 |
| [UDX-031](#udx-031) | LAN发布和慢客户端有界状态 | R16；来源E | PUB-01 | UD-04-044 |
| [UDX-032](#udx-032) | 导入日期保真调查 | R11；来源A | SGF-13 | UD-05-11 |
| [UDX-033](#udx-033) | 主题纹理与预设目标决定 | R17；来源F | APPEAR-01 | UD-05-12 |
| [UDX-034](#udx-034) | 外部落子意图与模式切换 | R14；来源C | GAME-10 | UD-05-14, UD-05-15 |
| [UDX-035](#udx-035) | Fox段位继承与显式昵称/UID调查 | R14；来源C | PROV-02 | UD-05-16, UD-05-26 |
| [UDX-036](#udx-036) | SGF导入容错与结果元数据调查 | R11；来源A | SGF-01, SGF-07 | UD-05-18 |
| [UDX-037](#udx-037) | 树图/变例/预览异步身份调查 | R12/R13（逐意图）；来源B | UI-02, UI-03, ANA-13 | UD-05-19, UD-05-21, UD-05-24 |
| [UDX-038](#udx-038) | PDA/WRN成对参数读回 | R12/R13（逐意图）；来源B | ANA-22 | UD-05-20 |
| [UDX-039](#udx-039) | 更新版本选择政策 | R18；来源旧R11发行 | REL-03 | UD-05-22 |
| [UDX-040](#udx-040) | 候选列表可访问性 | R12/R13（逐意图）；来源B | ANA-20 | UD-05-23 |

<a id="udx-001"></a>
## UDX-001 — 必要诊断、脱敏与有界导出

当前阶段导航 **R12/R13（逐意图）**（历史来源组 **B**；混合归属以意图索引为准）；条目/后继 **REL-09**。各来源的有界调查/决定仍须先输出结论，不能把族导航视为新增实现或全部Accepted。明确的其他阶段消费者按其任务合同单独交付；其存在不阻塞独立本地工作。

<a id="ud-03-001"></a>
### UD-03-001 — 诊断入口、启动失败回退与支持指引

- **来源 PR 与提交**:
  - 主来源: `PR #379` (包含被集成 PR: #378)
  - 完整 40 位 Commit SHA (2 个):
    - `a96cbf45e83f99aad32faf3249739adec56665ea` (next-2026-08-30.1) — Merge pull request #379 from wimi321/cloud/unify-diagnostic-pack-docs-5eec
    - `45a111f16a952274e46059bc1ee9d8478ae64c42` (next-2026-08-30.1) — docs: unify diagnostic pack guidance in troubleshooting and issue templates
- **核心文件出处**: `.github/ISSUE_TEMPLATE/bug_report.yml`, `.github/ISSUE_TEMPLATE/installation_report.yml`, `CONTRIBUTING.md`, `SUPPORT.md` 等共 5 文件
- **用户可见行为与边界**: 帮助菜单导出脱敏诊断包；无法启动时有可定位的普通/崩溃日志回退。不是新增工单平台。
- **对等项 / 任务映射**: REL-09（Missing）；T01-DIAGNOSTICS；`T03-REL09-DIAG-BUNDLE-EXPORT`。所有历史项仅引用Matrix原范围，最终后继ID由06分配。
- **Tauri 现状 / 证据与残余**: 基线没有 Help 诊断导出入口或支持包 UI。
- **处置结论**: 需后继功能/修复。任务 kind=`feature`；不把Source差异等同Next缺陷。
- **功能断言 / 调查停止条件**: 导出或取消有可见结果；ZIP 内秘密不出现；无法启动时指引能定位实际回退日志。
- **实际阻塞与未来证据门**: T01-DIAGNOSTICS 的功能日志/脱敏合同；原生目录与正式安装路径分别验收。
- **唯一批次建议 / 责任**: B/C/D 随消费者；公共入口 E；`T03-REL09-DIAG-BUNDLE-EXPORT` 拥有此Delta的结果。07/08冻结具体实施合同，06负责同源去重。

<a id="ud-03-002"></a>
### UD-03-002 — 有界线程/任务快照与包内敏感身份别名

- **来源 PR 与提交**:
  - 主来源: `PR #381` (包含被集成 PR: #373)
  - 完整 40 位 Commit SHA (5 个):
    - `12d6786e30af2c1a6d40cee394d62e6f205b321f` (next-2026-08-30.1) — Merge pull request #381 from wimi321/cloud/thread-snapshot-diagnostics-ff5f
    - `a3424b91814dae00e8b4aa4168c14b8f1880122c` (next-2026-08-30.1) — Merge branch 'main' into cloud/thread-snapshot-diagnostics-ff5f
    - `db96fbc36111f5ac59934943fad1b539846e6601` (next-2026-08-30.1) — docs(diagnostics): note why thread dumps use threadId= (#373)
    - `a61f66607f60b491367d40b41bebca1d78183d5b` (next-2026-08-30.1) — fix(diagnostics): avoid sanitizer aliasing thread ids as yike rooms (#373)
    - `1912a9bc7730033a7aeebca1ed86876eb4ca0a7a` (next-2026-08-30.1) — feat(diagnostics): include thread snapshots in diagnostic ZIP (#373)
- **核心文件出处**: `src/main/java/featurecat/lizzie/Lizzie.java`, `src/main/java/featurecat/lizzie/logging/DiagnosticBundleExporter.java`, `src/main/java/featurecat/lizzie/logging/EdtHangWatchdog.java`, `src/main/java/featurecat/lizzie/logging/ThreadSnapshot.java`
- **用户可见行为与边界**: 快照保留 threadId= 的数字线程标识、线程状态/栈/角色；采集失败不破坏支持包。线程标识不等于弈客 room/session 身份。后者同一 ZIP 内只使用稳定 route-kind+序号别名，不能导出原值或可枚举数字哈希。
- **对等项 / 任务映射**: REL-09（Missing）；T01-DIAGNOSTICS；`T03-REL09-THREAD-SNAPSHOT`。所有历史项仅引用Matrix原范围，最终后继ID由06分配。
- **Tauri 现状 / 证据与残余**: 原生线程/任务诊断面尚缺；不要求移植 JVM/EDT 实现。
- **额外来源 / 继承条件**: a61f6660 patch；0a0d44f0 的 docs/specs/2026-05-31-sync-diagnostics-export-design.md:199–206,284–295,401–402。
- **处置结论**: 需后继功能/修复。任务 kind=`feature`；不把Source差异等同Next缺陷。
- **功能断言 / 调查停止条件**: 导出同一敏感身份至多个文件，关联别名一致且原 room ID/session key/token/URL/用户名路径均不存在；合法 threadId 保留；采集不可用时报告 missing，导出可结束/取消。
- **实际阻塞与未来证据门**: T01-DIAGNOSTICS；先定义原生可采集角色与有界采集失败表示。
- **唯一批次建议 / 责任**: B/C/D 随消费者；`T03-REL09-THREAD-SNAPSHOT` 拥有此Delta的结果。07/08冻结具体实施合同，06负责同源去重。

<a id="ud-03-003"></a>
### UD-03-003 — 结构化启动身份与结果诊断

- **来源 PR 与提交**:
  - 主来源: `PR #382` (包含被集成 PR: #376)
  - 完整 40 位 Commit SHA (3 个):
    - `56cc54bc1c8d88de6093fc63659f519abdd2c472` (next-2026-08-30.1) — Merge pull request #382 from wimi321/cloud/engine-startup-bootstrap-376-9424
    - `2a7b5e36a2acca723f157f9a958ec382a9a706b7` (next-2026-08-30.1) — Merge branch 'main' into cloud/engine-startup-bootstrap-376-9424
    - `291ab979b3e4743b6723a0c23ea4771b501146b4` (next-2026-08-30.1) — feat(engine): log structured startup bootstrap diagnostics (#376)
- **核心文件出处**: `src/main/java/featurecat/lizzie/analysis/AnalysisResourceCoordinator.java`, `src/main/java/featurecat/lizzie/analysis/EngineStartupBootstrap.java`, `src/main/java/featurecat/lizzie/analysis/Leelaz.java`, `src/main/java/featurecat/lizzie/logging/EngineBootstrapFacts.java` 等共 5 文件
- **用户可见行为与边界**: 启动类型、用途、来源、后端、ONNX provider、脱敏模型/配置身份与 ready/failed 共用启动 identity。来源不保证模型一定有 SHA，不能把路径脱敏替换成未提供的哈希事实。
- **对等项 / 任务映射**: ENG-01（Accepted，仅 profile/assets）；ENG-02（Accepted，Run）；REL-09（Missing）；`T03-ENG01-BOOTSTRAP-DIAGNOSTICS`。所有历史项仅引用Matrix原范围，最终后继ID由06分配。
- **Tauri 现状 / 证据与残余**: 已有 profile/Run 并不覆盖结构化诊断输出。
- **处置结论**: 需后继功能/修复。任务 kind=`feature`；不把Source差异等同Next缺陷。
- **功能断言 / 调查停止条件**: 同一启动的成功或失败记录可关联其脱敏资源/后端身份；旧启动不能归入新 Run；绝对用户名路径和秘密不出现。
- **实际阻塞与未来证据门**: T01-RESOURCE 与 T01-DIAGNOSTICS；真实资源/引擎证据随首个启动消费者。
- **唯一批次建议 / 责任**: B；`T03-ENG01-BOOTSTRAP-DIAGNOSTICS` 拥有此Delta的结果。07/08冻结具体实施合同，06负责同源去重。

<a id="ud-03-004"></a>
### UD-03-004 — 原生内存/磁盘诊断及缺失指标

- **来源 PR 与提交**:
  - 主来源: `PR #383` (包含被集成 PR: #374)
  - 完整 40 位 Commit SHA (2 个):
    - `c74eec0a51d090630b7516a610b8133606a0d9c4` (next-2026-08-30.1) — Merge pull request #383 from wimi321/cloud/runtime-snapshot-diagnostics-b51b
    - `588e18beb9d1776516f644c257ed79a38c533827` (next-2026-08-30.1) — feat(diagnostics): add JVM memory and disk snapshot to diagnostic pack (#374)
- **核心文件出处**: `src/main/java/featurecat/lizzie/logging/DiagnosticBundleExporter.java`, `src/main/java/featurecat/lizzie/logging/LoggingRuntime.java`, `src/main/java/featurecat/lizzie/logging/RuntimeSnapshot.java`
- **用户可见行为与边界**: Java 导出 JVM heap/max 与工作/临时目录磁盘快照；适用用户目标是资源压力可见、单指标失败隔离。JVM heap 实现内部排除；Next 选择对应原生进程与实际应用/临时目录指标。
- **对等项 / 任务映射**: REL-09（Missing）；T01-DIAGNOSTICS；`T03-REL09-RUNTIME-SNAPSHOT`。所有历史项仅引用Matrix原范围，最终后继ID由06分配。
- **Tauri 现状 / 证据与残余**: 尚无运行时快照导出证据。
- **处置结论**: 需后继功能/修复。任务 kind=`feature`；不把Source差异等同Next缺陷。
- **功能断言 / 调查停止条件**: 可取得的指标有单位和采集时间；不可取得项明确 missing；其他指标/ZIP 不因单项失败丢失；取消有终态。
- **实际阻塞与未来证据门**: 原生采集可用性合同；不复制 JVM 数值或固定开发机路径。
- **唯一批次建议 / 责任**: B/C/D 随消费者；`T03-REL09-RUNTIME-SNAPSHOT` 拥有此Delta的结果。07/08冻结具体实施合同，06负责同源去重。

<a id="ud-03-005"></a>
### UD-03-005 — 有界 probe stderr 与提前退出分类

- **来源 PR 与提交**:
  - 主来源: `PR #384` (包含被集成 PR: #375)
  - 完整 40 位 Commit SHA (4 个):
    - `26808a5291991e9647b91a08dec686ba7bfefbc2` (next-2026-08-30.1) — Merge pull request #384 from wimi321/cloud/gtp-probe-stderr-diagnostics-1cf0
    - `424e2d1d5755fdf69b3c7e0a014a7427fead40ca` (next-2026-08-30.1) — Merge branch 'main' into cloud/gtp-probe-stderr-diagnostics-1cf0
    - `10ba23424f3cf381c019c07bc81f759da2ef3d67` (next-2026-08-30.1) — fix(diagnostics): classify probe early-exit after stdout EOF
    - `34f395bc2c2061d431c4443af6ebf00659d1152e` (next-2026-08-30.1) — feat(diagnostics): keep bounded GTP probe stderr (#375)
- **核心文件出处**: `src/main/java/featurecat/lizzie/analysis/gtpconfig/GtpConfigurationProbe.java`, `src/main/java/featurecat/lizzie/logging/EngineObservation.java`
- **用户可见行为与边界**: 配置探针有界保存最近 stderr；stdout EOF/提前退出区别于 timeout，观测记录不能使探针失控。
- **对等项 / 任务映射**: ENG-01（Accepted，资产/profile 基础）；具名 probe 诊断后继；`T03-ENG01-GTP-PROBE-STDERR`。所有历史项仅引用Matrix原范围，最终后继ID由06分配。
- **Tauri 现状 / 证据与残余**: 现有异步输出结构不是此探针边界的验收。
- **处置结论**: 需后继功能/修复。任务 kind=`feature`；不把Source差异等同Next缺陷。
- **功能断言 / 调查停止条件**: 错误依赖、配置失败、配置未完成 EOF 与 timeout 有区别；摘要受来源/字节限额约束并脱敏，额外输出不会无限增长；无任意“最后50行”产品要求。
- **实际阻塞与未来证据门**: 定位实际受支持配置探测消费者；T01-DIAGNOSTICS 必需诊断。
- **唯一批次建议 / 责任**: B；`T03-ENG01-GTP-PROBE-STDERR` 拥有此Delta的结果。07/08冻结具体实施合同，06负责同源去重。

<a id="ud-03-019"></a>
### UD-03-019 — 日志恢复后旧工作隔离

- **来源 PR 与提交**:
  - 主来源: `PR #411`
  - 完整 40 位 Commit SHA (1 个):
    - `3327b7c46f66bc745ba7e7bf118b3c454d1c1676` (next-2026-09-03.1) — fix: make logging recovery generation-aware (#411)
- **核心文件出处**: `src/main/java/featurecat/lizzie/logging/BoundedAsyncAppender.java`, `src/main/java/featurecat/lizzie/logging/LoggingRuntime.java`
- **用户可见行为与边界**: 恢复/重新初始化日志后旧异步重试不得污染新会话；保留可见失败和有界队列。Java generation实现内部机制不强制复制。
- **对等项 / 任务映射**: REL-09（Missing）；T01-DIAGNOSTICS；`T03-REL09-LOGGING-GENERATION`。所有历史项仅引用Matrix原范围，最终后继ID由06分配。
- **Tauri 现状 / 证据与残余**: 基础tracing并非已验收动态日志恢复。
- **处置结论**: 需后继功能/修复。任务 kind=`feature`；不把Source差异等同Next缺陷。
- **功能断言 / 调查停止条件**: 故障恢复后旧pending写入不能归入新会话；当前会话可用或明确失败；队列/重试受已定义预算约束，不自动无限恢复。
- **实际阻塞与未来证据门**: 诊断运行时实际支持的恢复模型；不新增无关动态重配置。
- **唯一批次建议 / 责任**: B/C/D 随消费者；`T03-REL09-LOGGING-GENERATION` 拥有此Delta的结果。07/08冻结具体实施合同，06负责同源去重。

<a id="ud-03-025"></a>
### UD-03-025 — Full Trace 解释节点结果写入/拒绝

- **来源 PR 与提交**:
  - 主来源: `PR #425` (包含被集成 PR: #424)
  - 完整 40 位 Commit SHA (1 个):
    - `237cbc0073da3bcc6b6115b95fe4f9a0f62c0b93` (next-2026-09-13.2) — feat(diagnostics): analysis-cache Full Trace decisions (#424) (#425)
- **核心文件出处**: `src/main/java/featurecat/lizzie/logging/EngineObservation.java`, `src/main/java/featurecat/lizzie/rules/BoardData.java`
- **用户可见行为与边界**: 最终analysis-cache ACCEPT/REJECT解释新引擎结果为何写入/跳过节点槽；incoming/cached visits、winrate/score、reason/identity用于关联；不是缓存命中读/重用。仅显式Full Trace启用，失败不影响publication。
- **对等项 / 任务映射**: REL-09（Missing）；ANA-04/ANA-14现有结果基础；T01-DIAGNOSTICS；`T03-REL09-ANALYSIS-CACHE-TRACE`。所有历史项仅引用Matrix原范围，最终后继ID由06分配。
- **Tauri 现状 / 证据与残余**: 是否SQLite无关：Next内存/节点结果写入仍有采用/拒绝边界。
- **额外来源 / 继承条件**: 237cbc00 patch:11–13,39–86,117以后BoardData写入判断。
- **处置结论**: 需后继功能/修复。任务 kind=`feature`；不把Source差异等同Next缺陷。
- **功能断言 / 调查停止条件**: 受控较强/较弱/ownership回填或无效identity输入的最终采用/拒绝输出原因与实际结果一致；Trace off无此行，Trace故障不阻断结果；脱敏受限。
- **实际阻塞与未来证据门**: 实际Next结果采用owner；036/054等受影响分析诊断随功能。
- **唯一批次建议 / 责任**: B（随结果publication）；`T03-REL09-ANALYSIS-CACHE-TRACE` 拥有此Delta的结果。07/08冻结具体实施合同，06负责同源去重。

<a id="ud-03-030"></a>
### UD-03-030 — 脱敏保留日志记录边界

- **来源 PR 与提交**:
  - 主来源: `PR #436`
  - 完整 40 位 Commit SHA (1 个):
    - `1d6bb41ffa40b0efc8a31c5a348f8805d4324d0a` (next-2026-09-13.2) — fix(logging): preserve exported log record boundaries (#436)
- **核心文件出处**: `src/main/java/featurecat/lizzie/analysis/SyncDiagnosticsExportSanitizer.java`, `src/main/java/featurecat/lizzie/logging/DiagnosticBundleExporter.java`
- **用户可见行为与边界**: 脱敏替换必须保留记录/换行和时间/level关联，包内敏感身份别名与002共用；不让超限输入无限工作。
- **对等项 / 任务映射**: REL-09（Missing）；T01-DIAGNOSTICS；`T03-REL09-SANITIZER-BOUNDARIES`。所有历史项仅引用Matrix原范围，最终后继ID由06分配。
- **Tauri 现状 / 证据与残余**: 日志导出能力缺口。
- **处置结论**: 需后继功能/修复。任务 kind=`feature`；不把Source差异等同Next缺陷。
- **功能断言 / 调查停止条件**: 包含多行/敏感token/session样本的导出记录边界未合并，秘密原文不存在、别名跨文件一致；超限/取消有明确终态；不冻结任意1000行数量。
- **实际阻塞与未来证据门**: 002/031导出和隐私合同；不改普通日志记录语义。
- **唯一批次建议 / 责任**: B/C/D 随消费者；`T03-REL09-SANITIZER-BOUNDARIES` 拥有此Delta的结果。07/08冻结具体实施合同，06负责同源去重。

<a id="ud-03-031"></a>
### UD-03-031 — 冻结导出请求、有界估算/发布与安全归档剪枝

- **来源 PR 与提交**:
  - 主来源: `PR #438`
  - 完整 40 位 Commit SHA (1 个):
    - `0a0d44f0906bbd79b462b79ef540caff295092ba` (next-2026-09-13.2) — fix(diagnostics): bound export work and unblock completion (#438)
- **核心文件出处**: docs/specs/2026-05-31-sync-diagnostics-export-design.md；logging/DiagnosticBundleRequest.java, LogArchiveBoundary.java, DiagnosticBundleExporter.java；gui/DiagnosticsDialog.java
- **用户可见行为与边界**: action冻结host/session/scope/settings/time/helper/config；后台枚举读取。有界估算未压缩候选MiB，不预测ZIP/CRC/压缩率；一个估算执行+一个latest pending并隔离旧generation。旧归档仅path+非空identity+creation time全部匹配才剪枝；Windows卷号+128bit file ID区别同creation time替换档案，不解决锁阻塞。成功以ZIP close→force→atomic publication为界，folder opener后台失败不撤销成功。
- **对等项 / 任务映射**: REL-09（Missing）；T01-DIAGNOSTICS；`T03-REL09-EXPORT-BOUNDED-IO`。所有历史项仅引用Matrix原范围，最终后继ID由06分配。
- **Tauri 现状 / 证据与残余**: 源目标适用于未来诊断导出；不标Covered。
- **额外来源 / 继承条件**: 0a0d44f0 docs/specs/...:451–495；DiagnosticBundleRequest,LogArchiveBoundary,DiagnosticsDialog,DiagnosticBundleExporter。
- **处置结论**: 需后继功能/修复。任务 kind=`feature`；不把Source差异等同Next缺陷。
- **功能断言 / 调查停止条件**: 同路径同创建时间但不同file ID的新档案保留，unknown identity保留且有界筛选；真旧档可prune。更改UI后旧请求结果不覆盖新估算；未知量标unknown。取消/close/force失败无成功成品；已成功原子发布后打开目录失败仍显示导出成功；记录真实各stage与大小上限。
- **实际阻塞与未来证据门**: 有界/可取消导出合同；平台metadata能力；source1024枚举边界为原来源事实，具体Next限额由功能合同冻结，不继承无条件Windows延迟保证。
- **唯一批次建议 / 责任**: B/C/D 随消费者；`T03-REL09-EXPORT-BOUNDED-IO` 拥有此Delta的结果。07/08冻结具体实施合同，06负责同源去重。

<a id="ud-03-042"></a>
### UD-03-042 — 启动失败诊断按当前Run隔离

- **来源 PR 与提交**:
  - 主来源: `PR #461`
  - 完整 40 位 Commit SHA (3 个):
    - `86ee29510b605896a46543970f64a8a94d0e6f15` (next-2026-09-18.2) — Merge pull request #461 from qiyi71w/fix/engine-startup-diagnostics
    - `08d6d1b39dd6793c8e04fe09ead8e3bc0ad52b95` (next-2026-09-18.2) — chore: merge main into startup diagnostics
    - `1ba79d4d27338a170f7088acbdc86a729481a417` (next-2026-09-18.2) — fix(engine): restore fenced startup diagnostics
- **核心文件出处**: `src/main/java/featurecat/lizzie/analysis/Leelaz.java`, `src/test/java/featurecat/lizzie/analysis/EngineStartupDialogPolicyTest.java`
- **用户可见行为与边界**: 非first-launch仍显示当前失败详情；已换engine/run时丢弃旧启动失败dialog，不能打断新对局。
- **对等项 / 任务映射**: ENG-02 / ENG-04（Accepted，身份/切换）；REL-09（Missing）；`T03-ENG01-ISOLATED-STARTUP-FAILURE`。所有历史项仅引用Matrix原范围，最终后继ID由06分配。
- **Tauri 现状 / 证据与残余**: 已有stale identity基础不等于此UI失败记录Covered。
- **处置结论**: 需后继功能/修复。任务 kind=`feature`；不把Source差异等同Next缺陷。
- **功能断言 / 调查停止条件**: 启动A失败通知延迟，切至B成功/Match后A通知不覆盖B；当前失败仍可见并脱敏，failure解释有原Run identity。
- **实际阻塞与未来证据门**: T01-DIAGNOSTICS、ENG02/04现有owner；证据按repository/realengine/native区分。
- **唯一批次建议 / 责任**: B；`T03-ENG01-ISOLATED-STARTUP-FAILURE` 拥有此Delta的结果。07/08冻结具体实施合同，06负责同源去重。

<a id="ud-03-052"></a>
### UD-03-052 — 中断下有界日志关闭/资源清理

- **来源 PR 与提交**:
  - 主来源: `PR #482`
  - 完整 40 位 Commit SHA (2 个):
    - `9e0ca19e1e0e929d42b368755bcfa891bdf6c906` (next-2026-09-18.2) — Merge pull request #482 from wimi321/fix/logging-shutdown-cleanup
    - `b0c905238453b0eacd367699e7821555da864c42` (next-2026-09-18.2) — fix(logging): reserve time for interrupted resource cleanup
- **核心文件出处**: `src/main/java/featurecat/lizzie/logging/Deadline.java`
- **另有导入来源（不重复计能力）**: `59ebb4c129f074b778af766442ef7e08112e6d67`。这些源同时保留其内部分组。
- **用户可见行为与边界**: Deadline为被interrupt的关闭保留cleanup/flush时间；在界内完成或报告未完成，不承诺强杀一定flush。
- **对等项 / 任务映射**: REL-09（Missing）；T01-DIAGNOSTICS；`T03-REL09-SHUTDOWN-CLEANUP`。所有历史项仅引用Matrix原范围，最终后继ID由06分配。
- **Tauri 现状 / 证据与残余**: Rust process teardown不证明日志队列的bounded interrupted cleanup。
- **处置结论**: 需后继功能/修复。任务 kind=`feature`；不把Source差异等同Next缺陷。
- **功能断言 / 调查停止条件**: 正常退出与可控中断下有界结束；最终记录完成或可见缺失，不因interrupt无限等待/复活Run；强制杀进程不伪造落盘。
- **实际阻塞与未来证据门**: 实际日志owner、source cap和deadline契约；正式安装日志路径另验。
- **唯一批次建议 / 责任**: B/C/D 随诊断消费者；`T03-REL09-SHUTDOWN-CLEANUP` 拥有此Delta的结果。07/08冻结具体实施合同，06负责同源去重。

<a id="ud-04-040"></a>
### UD-04-040 诊断窗口layout与scratch清理

- **Source / event**: PR #538 (集成事件 38) | 共 3 个提交；第一父E38。
- **完整source commits（与§6反向对应）**:
  - [085fba504918aebe26366216f918e18d4b15669c](https://github.com/wimi321/lizzieyzy-next/commit/085fba504918aebe26366216f918e18d4b15669c) — chore(repo): untrack local scratch evidence
  - [558db3043e56bffc2fe364cac6e6c66ac77792f8](https://github.com/wimi321/lizzieyzy-next/commit/558db3043e56bffc2fe364cac6e6c66ac77792f8) — fix(gui): stabilize diagnostics dialog layout
  - [7a64b2b20e194e24a1e9ec822e7ff63d17f82fe4](https://github.com/wimi321/lizzieyzy-next/commit/7a64b2b20e194e24a1e9ec822e7ff63d17f82fe4) — QA integrate PR 538
- **发布包含**: 最早有API/ancestry证据收录 `next-2026-09-26.1`（正式发布）；全部source也含于正式09-26.1/.2。准备target仅记prepared，不由本字段推断准备版本已发布。
- **实际behavior / sourceDelta**: 085fba50清理local scratch，558db304调整diagnostics dialog布局，7a64b2b2整合。没有React全DPI免疫；未实现的diagnostic窗口不能继承48db R5。
- **Canonical owner / scope**: REL-09 Missing/T01-DIAGNOSTICS；诊断后继布局。
- **候选/覆盖与边界**: 无此窗口Tauri native证据；保留R7 WINDOW-01原baa9747d11ee9e2130904ff057adbe7cfd968153范围，不扩成diag通过。
- **Disposition**: Java scratch机制排除；新增窗口layout为功能gate。
- **责任意图**: T04-DIAGNOSTICS。
- **Observable assertions**: 诊断后继在适用Windows显示条件保持详情/Copy/Export/Cancel均可见可达，长文本不遮buttons；原layout evidence只在原window条件适用，不新增任意多平台矩阵。

<a id="ud-04-046"></a>
### UD-04-046 最终startup failure、bounded output、WARN与pinned export

- **Source / event**: PR #545 (集成事件 38) | 共 14 个提交；第一父E38。
- **完整source commits（与§6反向对应）**:
  - [7f2dc02f53eb7b17f12e4c48df87d95bcfa50936](https://github.com/wimi321/lizzieyzy-next/commit/7f2dc02f53eb7b17f12e4c48df87d95bcfa50936) — feat(engine): focus startup failure diagnostics
  - [80940479be71be37e9da4e4cb137438e4d71f240](https://github.com/wimi321/lizzieyzy-next/commit/80940479be71be37e9da4e4cb137438e4d71f240) — test(engine): fix startup diagnostic CI regressions
  - [cc47b2cde023a1ccc0399958ad1cf25c9c49b2c7](https://github.com/wimi321/lizzieyzy-next/commit/cc47b2cde023a1ccc0399958ad1cf25c9c49b2c7) — fix(gui): preserve engine failure window layout
  - [138e0d17d8ddbe608c667e970281f447a73ad00c](https://github.com/wimi321/lizzieyzy-next/commit/138e0d17d8ddbe608c667e970281f447a73ad00c) — test(engine): retain integrated native diagnostic artifacts
  - [b0e0e07cc79b4ba71ef9e5f1f40d420d5c73adf1](https://github.com/wimi321/lizzieyzy-next/commit/b0e0e07cc79b4ba71ef9e5f1f40d420d5c73adf1) — test(engine): exercise native search and runtime evidence in desktop
  - [b82833a592fa6567cf817ef8ec4c93b63a54266c](https://github.com/wimi321/lizzieyzy-next/commit/b82833a592fa6567cf817ef8ec4c93b63a54266c) — test(engine): compare sanitized probe export evidence
  - [b3c4eec18c421317c321418a7900dee67bc1a84b](https://github.com/wimi321/lizzieyzy-next/commit/b3c4eec18c421317c321418a7900dee67bc1a84b) — test(engine): cover native probe dependency failure
  - [2a7507ac901800652a686190a598ae4474c1f397](https://github.com/wimi321/lizzieyzy-next/commit/2a7507ac901800652a686190a598ae4474c1f397) — feat(engine): integrate startup diagnostics
  - [fedeb66cd182fe6e9532ea5a032fe1f0db790560](https://github.com/wimi321/lizzieyzy-next/commit/fedeb66cd182fe6e9532ea5a032fe1f0db790560) — feat(engine): trace native PE startup dependencies
  - [5001b532ba194014b7fe8376daa08a2c3e7f819d](https://github.com/wimi321/lizzieyzy-next/commit/5001b532ba194014b7fe8376daa08a2c3e7f819d) — feat(engine): diagnose independent launchers
  - [064ed9c4e0de90967eb1dd4d00a6a859bad7c1c1](https://github.com/wimi321/lizzieyzy-next/commit/064ed9c4e0de90967eb1dd4d00a6a859bad7c1c1) — feat(engine): explain startup failure evidence
  - [9e7f1942b356596501f31303668ca6d4fc95f370](https://github.com/wimi321/lizzieyzy-next/commit/9e7f1942b356596501f31303668ca6d4fc95f370) — fix(engine): refine startup failure diagnostics
  - [4cb8e7530146b73955cb8050b6bc88f7b0d1e352](https://github.com/wimi321/lizzieyzy-next/commit/4cb8e7530146b73955cb8050b6bc88f7b0d1e352) — feat(engine): retain startup failure diagnostics
  - [75f1863a15fa8a8e0ccec29ab126c156ec99a92f](https://github.com/wimi321/lizzieyzy-next/commit/75f1863a15fa8a8e0ccec29ab126c156ec99a92f) — QA integrate PR 545
- **发布包含**: 最早有API/ancestry证据收录 `next-2026-09-26.1`（正式发布）；全部source也含于正式09-26.1/.2。准备target仅记prepared，不由本字段推断准备版本已发布。
- **实际behavior / sourceDelta**: PR545保留全部14来源，包括早期fedeb66c的PeDependencyScanner/PeImportReader和独立launcher/history接入；7f2dc02f最后删除这两production classes及tests，移除internal probes/session-history集成。最终是per-attempt GUI error/command、Windows status/explicit DLL/runtime evidence、bounded stdout/stderr tail、ordinary WARN原engine+launch identity、details/copy及导出当前displayed failure。不是PE扫描/全launcher历史保留功能。
- **Canonical owner / scope**: REL-09 Missing；T01-DIAGNOSTICS；ENG-02原typed failures。
- **候选/覆盖与边界**: ENG-02原typed error范围保留；REL-09仍Missing，已有chrome错误不等于普通日志/详情/导出全部实现。
- **Disposition**: 需诊断功能后继；中间机制显式撤回。
- **责任意图**: T04-DIAGNOSTICS。
- **Observable assertions**: 原attempt/engine/launch身份绑定，后到output不重定向new run；bounded lines/bytes/deadline与取消可见；retry/switch后打开旧failure导出必须pin displayed snapshot，不混当前错误；普通WARN与Copy/export均redact secrets/control chars、来源明确，受限可取消导出失败可见。不必复制已删除PE scanner，不做自动upload。

<a id="ud-05-08"></a>
### UD-05-08 

- 来源：[020](https://github.com/wimi321/lizzieyzy-next/commit/84c7e00d83f0f78cdd7619f76f89281120e4c8fa), [021](https://github.com/wimi321/lizzieyzy-next/commit/d017a5a07f37699b8d0e29cfe17e39b65f8125c1), [022](https://github.com/wimi321/lizzieyzy-next/commit/7a244119c6225f4ad583ef25b47f9415703e6b41), [047](https://github.com/wimi321/lizzieyzy-next/commit/8c43fdd24b01acd6485b8b3911708290ae18ab75)（完整SHA/发布包含见源索引）。
- 用户行为/边界：启动失败诊断区分配置/运行时/进程失败，输出可操作信息。
- Tauri映射/具名任务意图：ENG-01、REL-09功能诊断；T05-STARTUP-DIAGNOSTICS（B）
- 覆盖证据或缺口：PR557；Tauri已有typed lifecycle failure仅作已知基础，不等于全日志/导出完成。
- 处置：补齐功能诊断；与05-10敏感信息约束合并，发行崩溃包后置。
- 后续验收/调查停止条件：缺路径/坏配置/版本不兼容/子进程退出均显示真实原因，普通日志有界，敏感值不出屏或导出。

<a id="ud-05-10"></a>
### UD-05-10 

- 来源：[028](https://github.com/wimi321/lizzieyzy-next/commit/e8f38d76ab16f0cbfe9e92644dc29482f0a4de1e), [029](https://github.com/wimi321/lizzieyzy-next/commit/2ccf9995b33646361226989b1d2bfa3380b24b93), [030](https://github.com/wimi321/lizzieyzy-next/commit/cda3977afb04bf5f2e2fb06af83602728b85ecc1), [035](https://github.com/wimi321/lizzieyzy-next/commit/cc91b449220c8569e0753108b86a584b3fab09c6), [036](https://github.com/wimi321/lizzieyzy-next/commit/2d0c829308ae41db7e1ac17c82f9ba44e9741456), [037](https://github.com/wimi321/lizzieyzy-next/commit/d376bf54c2d33fc2aac5b6c15a79679dc4b7f331), [044](https://github.com/wimi321/lizzieyzy-next/commit/567d71378d9bf9671a6151566b5d5cf78f1414e1), [047](https://github.com/wimi321/lizzieyzy-next/commit/8c43fdd24b01acd6485b8b3911708290ae18ab75), [069](https://github.com/wimi321/lizzieyzy-next/commit/e9f11ac5a66584111fbf8d461294841bf1e3c608), [070](https://github.com/wimi321/lizzieyzy-next/commit/bc4c318f462924462f03853494cc6aa454ae2770), [071](https://github.com/wimi321/lizzieyzy-next/commit/65cc93842eb26e0dd0ee911f3d16493b52cd6399), [077](https://github.com/wimi321/lizzieyzy-next/commit/0fe5b3769e5f1402013df5520643466ff021f5e6)（完整SHA/发布包含见源索引）。
- 用户行为/边界：SSH plink、Contrib、远程连接与失败回退argv的密码脱敏；保留参数边界和CR/LF；线性扫描避免诊断拖慢交互。
- Tauri映射/具名任务意图：SSH-01、CONTRIB-01、RCOMP-01、REL-09；T05-SECRET-DIAGNOSTICS（B基础，C/E消费）
- 覆盖证据或缺口：PR562/565/568/572。源代码修复含带空格密码和原命令缺失fallback；不证明当前Tauri同缺陷。
- 处置：未来接入的安全不变量；现有普通诊断路径先做有界调查，不照搬Java扫描器。
- 后续验收/调查停止条件：假凭据覆盖空格/引号/换行/URL及fallback显示，UI/日志/支持包无泄漏；实际账号只在授权现场验证，不自动账号写入重试。

<a id="udx-002"></a>
## UDX-002 — 本地运行资源、来源与模型兼容

当前阶段导航 **R12/R13（逐意图）**（历史来源组 **B**；混合归属以意图索引为准）；条目/后继 **ENG-11**。各来源的有界调查/决定仍须先输出结论，不能把族导航视为新增实现或全部Accepted。明确的其他阶段消费者按其任务合同单独交付；其存在不阻塞独立本地工作。

<a id="ud-03-006"></a>
### UD-03-006 — 维护操作生命周期与诊断

- **来源 PR 与提交**:
  - 主来源: `PR #385` (包含被集成 PR: #377)
  - 完整 40 位 Commit SHA (1 个):
    - `6c82562434c195bf4ba5ed6659f4396805c31e44` (next-2026-08-30.1) — Merge pull request #385 from wimi321/cloud/maintenance-lifecycle-diagnostics-be61
- **核心文件出处**: `src/main/java/featurecat/lizzie/logging/MaintenanceObservation.java`, `src/main/java/featurecat/lizzie/util/KataGoAutoSetupHelper.java`, `src/main/java/featurecat/lizzie/util/KataGoRuntimeHelper.java`
- **用户可见行为与边界**: 配置、校验、修复、下载的开始/阶段/成功/失败按同一维护身份关联。
- **对等项 / 任务映射**: REL-05（Partial）；REL-09（Missing）；T01-RESOURCE / T01-DIAGNOSTICS；`T03-RESOURCE-MAINTENANCE-LIFECYCLE`。所有历史项仅引用Matrix原范围，最终后继ID由06分配。
- **Tauri 现状 / 证据与残余**: ENG-04 是失败切换/旧身份拒绝，不是维护向导；本维护观测缺口独立命名。
- **处置结论**: 需后继功能/修复。任务 kind=`feature`；不把Source差异等同Next缺陷。
- **功能断言 / 调查停止条件**: 一次维护的开始、阶段与唯一成功/失败/取消终态可关联；迟到旧操作不能覆盖新操作；失败可见且保留原有效资源。
- **实际阻塞与未来证据门**: 实际维护消费者与其资源合同；非正式签名组件发布。
- **唯一批次建议 / 责任**: B；`T03-RESOURCE-MAINTENANCE-LIFECYCLE` 拥有此Delta的结果。07/08冻结具体实施合同，06负责同源去重。

<a id="ud-03-008"></a>
### UD-03-008 — 独立 TRT 修复及失败恢复产物

- **来源 PR 与提交**:
  - 主来源: `PR #392` (包含被集成 PR: #380)
  - 完整 40 位 Commit SHA (4 个):
    - `0752ce50a78d52f57da9388240142340f590f0c2` (next-2026-08-31.1) — Merge pull request #392 from qiyi71w/fix/issue-380-tensorrt-repair
    - `6ebd2b41266ac52a8cff70bec221a97e59790404` (next-2026-08-31.1) — Merge branch 'main' into fix/issue-380-tensorrt-repair
    - `9199c3e5b5c0c13ea9fee55c4d19f4b39657ca85` (next-2026-08-31.1) — fix(tensorrt): preserve recovery artifacts
    - `8d56d7df8d1566e4a0f931341ef14a288d29e809` (next-2026-08-31.1) — fix(tensorrt): unblock repair from DirectML profiles
- **核心文件出处**: `src/main/java/featurecat/lizzie/analysis/AnalysisEngine.java`, `src/main/java/featurecat/lizzie/analysis/Leelaz.java`, `src/main/java/featurecat/lizzie/gui/HumanSlTensorRtRepairView.java`, `src/main/java/featurecat/lizzie/util/KataGoRuntimeHelper.java`
- **用户可见行为与边界**: 修复目标与前台 DirectML 等后端解耦；失败保留 last-known-good 和恢复产物，避免破坏当前可用配置。
- **对等项 / 任务映射**: REL-05（Partial）；T01-RESOURCE；具名 TRT applicability 调查；`T03-RESOURCE-TRT-REPAIR-INVESTIGATION`。所有历史项仅引用Matrix原范围，最终后继ID由06分配。
- **Tauri 现状 / 证据与残余**: Next 可用外部二进制/合格本地资源；是否提供内置 TRT 修复需决策，不能据无向导永久删除目标。
- **处置结论**: 需有界调查。任务 kind=`investigation`；不把Source差异等同Next缺陷。
- **功能断言 / 调查停止条件**: 问：准入资源体系是否拥有 TRT 定向修复？检查资源获取/替换面并产出受支持/外部负责/需决策清单；停止于路径、责任与失败保留合同明确。若实施，在非TRT前台可检查TRT目标，失败不损毁 last-known-good。
- **实际阻塞与未来证据门**: T01-RESOURCE 的资源身份/完整性；内置修复功能明确阻塞于此调查决策，不要求编译引擎。
- **唯一批次建议 / 责任**: B；`T03-RESOURCE-TRT-REPAIR-INVESTIGATION` 拥有此Delta的结果。07/08冻结具体实施合同，06负责同源去重。

<a id="ud-03-010"></a>
### UD-03-010 — 硬件资格、事务修复与恢复

- **来源 PR 与提交**:
  - 主来源: `PR #396`
  - 完整 40 位 Commit SHA (1 个):
    - `36535440c58bc4085c4309a3d99254589240e9b0` (next-2026-08-31.1) — fix(tensorrt): gate repair by NVIDIA hardware (#396)
- **核心文件出处**: `src/main/java/featurecat/lizzie/analysis/EngineManager.java`, `src/main/java/featurecat/lizzie/gui/BottomToolbar.java`, `src/main/java/featurecat/lizzie/gui/KataGoAutoSetupDialog.java`, `src/main/java/featurecat/lizzie/gui/TensorRtAccelerationView.java`
- **用户可见行为与边界**: NVIDIA 硬件资格与当前活动后端不同；选项/修复准入依据硬件，组件替换失败保留有效资源，成功后安全恢复。
- **对等项 / 任务映射**: REL-05（Partial）；T01-RESOURCE；`T03-RESOURCE-NVIDIA-HARDWARE-GATE`。所有历史项仅引用Matrix原范围，最终后继ID由06分配。
- **Tauri 现状 / 证据与残余**: ENG-04 不是 GPU 配置；探测/修复是否产品内提供需有界决策。
- **处置结论**: 需有界调查。任务 kind=`investigation`；不把Source差异等同Next缺陷。
- **功能断言 / 调查停止条件**: 问：各准入资源是否需要/可证明 NVIDIA 资格？检查目标元数据、探测可用性及 unknown；停止于 supported/unsupported/unknown 与责任表。后继验证非TRT前台仍按硬件准入，失败回滚资源，unknown不能宣称通过。
- **实际阻塞与未来证据门**: 008的修复所有权决策与合格GPU环境；无GPU项目保持待验。
- **唯一批次建议 / 责任**: B；`T03-RESOURCE-NVIDIA-HARDWARE-GATE` 拥有此Delta的结果。07/08冻结具体实施合同，06负责同源去重。

<a id="ud-03-010a"></a>
### UD-03-010a — 发布脚本产生的公开GPU兼容/下载说明

- **来源 PR / 完整提交**: `1f37094360690ac59eefb038067beeaf38d90d84`, `2f00081d1cff16a3625a4b1c44078b375bf28fdf`。PR和最早发布/full包含见§6/§2；小写子记录来源不超出03区间。
- **行为 / 源码边界**: scripts/r2_release.py 的catalog_label/render_index输出用户指南：RTX30/40/50优先CUDA通常更快，TensorRT仅建议RTX20/GTX16，GTX10不支持，分卷都需下载。这是该来源资源版本的公开指导，不是所有未来TRT版本永久硬件规律。
- **对等项 / 意图**: REL-05（Partial）；T01-RESOURCE；010资格调查关联；`T03-RESOURCE-PUBLIC-GPU-GUIDANCE`。
- **现状 / 处置 / kind**: 未将本新目标记Covered；需后继功能/修复；kind=`feature`。
- **实际阻塞**: 所准入资源版本/平台/硬件compatibility表；没有对应GPU保留待验，不要求当前compile/release。
- **有界可观察断言 / stop**: 未来资源获取/帮助文案与所选binary版本资格一致，不推荐已知不支持项；unknown明确；源指南以对应版本记录，新版变化须重新来源核实。发布脚本技术保留EXC-005。
- **唯一批次 / 责任**: B（资源帮助）；`T03-RESOURCE-PUBLIC-GPU-GUIDANCE`；子记录与本体由06去重为一个相关能力，不能当独立ParityID。

<a id="ud-03-035"></a>
### UD-03-035 — HumanSL 下载来源/镜像与完整性

- **来源 PR 与提交**:
  - 主来源: `PR #447`
  - 完整 40 位 Commit SHA (3 个):
    - `a06be4cad8aba89d3d705561874bd0a36cc49875` (next-2026-09-13.2) — Merge pull request #447 from wimi321/feat/humansl-r2-download
    - `c5f533e93d1a74d03660abed016db6ff471422fd` (next-2026-09-13.2) — Keep unrelated engine tests formatting unchanged
    - `f9e00b9e202b40350cc9246b5a64e71d3fe60af8` (next-2026-09-13.2) — Accelerate HumanSL downloads with verified R2 mirror
- **核心文件出处**: `src/main/java/featurecat/lizzie/util/KataGoAutoSetupHelper.java`, `docs/R2_RELEASES.md`
- **用户可见行为与边界**: 通过经过校验的R2镜像获取HumanSL权重；runtime identity、SHA256、兼容、失败可见是保留目标，镜像不豁免统一network。
- **对等项 / 任务映射**: REL-05（Partial）；GAME-08（Deferred）；T01-RESOURCE / T01-NETWORK；`T03-REL05-HUMANSL-R2-DOWNLOAD`。所有历史项仅引用Matrix原范围，最终后继ID由06分配。
- **Tauri 现状 / 证据与残余**: 运行资源功能与签名安装态分离；不硬编码新的任意可配置网络源产品设计。
- **处置结论**: 需后继功能/修复。任务 kind=`feature`；不把Source差异等同Next缺陷。
- **功能断言 / 调查停止条件**: 准入origin/version/路径/完整性清楚；wrong digest/不可用/取消不安装或改变原资源；选定网络策略不支持时可见失败；真实兼容HumanSL得到资源。
- **实际阻塞与未来证据门**: T01-RESOURCE与HumanSL准入及网络policy；不要求当前仓库release资产。
- **唯一批次建议 / 责任**: D（资源门随HumanSL）；`T03-REL05-HUMANSL-R2-DOWNLOAD` 拥有此Delta的结果。07/08冻结具体实施合同，06负责同源去重。

<a id="ud-03-035a"></a>
### UD-03-035a — production修复下载的可信origin/identity/asset边界

- **来源 PR / 完整提交**: `2061865fce2e9ec4a44633fe86f1dd6bde9112d2`, `4019a7e15aa4deaf2a1e5b6e49ae691f1f7db74d`, `e35b01c14e2fb423f09de05311bb68dd1f6d1c1c`。PR和最早发布/full包含见§6/§2；小写子记录来源不超出03区间。
- **行为 / 源码边界**: KataGoAssetCatalog.assetDownloadUrl消费可信catalog：非catalog同ID伪asset拒绝；official-origin不得改download repository，project-source-build origin只准来源允许仓库/完整sourcecommit/合法release tag；source archive名含commit前缀+target，archive名安全。上界KataGoRuntimeHelper:167–170,6196–6199用于TRT/CUDA companion repair，不是仅构建script。Next资源发布仓库待决，不机械硬编码Java仓库。
- **对等项 / 意图**: REL-05（Partial）；T01-RESOURCE / T01-NETWORK；008/010/035相关资源功能；`T03-RESOURCE-TRUSTED-ORIGIN-IDENTITY`。
- **现状 / 处置 / kind**: 未将本新目标记Covered；需后继功能/修复；kind=`feature`。
- **实际阻塞**: T01-RESOURCE集中可信origin/版本/路径/完整性/capability合同及渠道未决；合格本地binary允许；正式签名发行独立。
- **有界可观察断言 / stop**: 不可信origin、不可变identity不全、同ID篡改asset、目录穿越/不安全archive名在下载/安装前可见拒绝；合法本地/可信catalog资源可准入；digest/兼容失败不更改有效资源。选择network policy不能静默Direct。EXC-007仍排除compile/packaging技术。
- **唯一批次 / 责任**: B（及D/C资源消费者）；`T03-RESOURCE-TRUSTED-ORIGIN-IDENTITY`；子记录与本体由06去重为一个相关能力，不能当独立ParityID。

<a id="ud-03-037"></a>
### UD-03-037 — 默认 B11 资源身份更新

- **来源 PR 与提交**:
  - 主来源: `PR #451`
  - 完整 40 位 Commit SHA (1 个):
    - `fe95700e739484cc8dcf843943c2d19c212ed164` (next-2026-09-13.2) — feat(models): 更新默认 B11 为 2026-09-07 官方模型 (#451)
- **核心文件出处**: src/main/resources/katago-assets.json；util/KataGoAssetCatalog.java, KataGoAutoSetupHelper.java；scripts/katago_asset_catalog.py；docs/INSTALL.md / PACKAGES.md
- **用户可见行为与边界**: 默认打包/推荐B11改为2026-09-07官方网络；manifest/catalog、下载与文档一致，既有用户外部模型不被自动覆盖。
- **对等项 / 任务映射**: REL-05（Partial）；T01-RESOURCE；`T03-REL05-B11-MODEL-UPDATE`。所有历史项仅引用Matrix原范围，最终后继ID由06分配。
- **Tauri 现状 / 证据与残余**: 运行资源catalog更新，与HumanSL不同，不归BatchD。
- **处置结论**: 需后继功能/修复。任务 kind=`feature`；不把Source差异等同Next缺陷。
- **功能断言 / 调查停止条件**: 默认模型ID、版本与digest由源catalog明确关联；获取失败/不兼容可见，旧外部配置不被隐式修改；正式安装交付门保持未验。
- **实际阻塞与未来证据门**: T01-RESOURCE的版本/完整性/兼容合同。
- **唯一批次建议 / 责任**: B；`T03-REL05-B11-MODEL-UPDATE` 拥有此Delta的结果。07/08冻结具体实施合同，06负责同源去重。

<a id="ud-03-039"></a>
### UD-03-039 — 加速设置分组、状态文本与窄宽可达性

- **来源 PR 与提交**:
  - 主来源: `PR #453`
  - 完整 40 位 Commit SHA (1 个):
    - `a204afe2c6a1749b51cf4586e9ca59018d1a7f7f` (next-2026-09-13.2) — fix(autosetup): restore responsive TensorRT groups (#453)
- **核心文件出处**: `src/main/java/featurecat/lizzie/gui/KataGoAccelerationLayout.java`
- **用户可见行为与边界**: repair/enable、experimental backend、maintenance分组分离且wrap；完整状态/维护文案与变化label仍可见；Swing caret-margin/布局技术排除。
- **对等项 / 任务映射**: 具名 acceleration-layout successor；UI-02（Partial）相关交互基础；`T03-RESOURCE-ACCEL-LAYOUT`。所有历史项仅引用Matrix原范围，最终后继ID由06分配。
- **Tauri 现状 / 证据与残余**: Next尚未有此setupUI，CSS不证明已可达；用户目标归修复向导而非Java-only全排除。
- **处置结论**: 需后继功能/修复。任务 kind=`feature`；不把Source差异等同Next缺陷。
- **功能断言 / 调查停止条件**: 在未来设置支持的窗口/DPI/语言边界，所有动作可达/标签完整/状态可读，组不混淆；键盘操作可用。尺寸由功能UI合同/真实风险决定，不任意600px。
- **实际阻塞与未来证据门**: 008/010决定哪些setup动作产品保留；它们明确前置，Java布局类不迁移。
- **唯一批次建议 / 责任**: B；`T03-RESOURCE-ACCEL-LAYOUT` 拥有此Delta的结果。07/08冻结具体实施合同，06负责同源去重。

<a id="ud-04-001"></a>
### UD-04-001 ROCm源码构建与可见兼容义务

- **Source / event**: PR #487 (集成事件 1) | 共 9 个提交；第一父E01。
- **完整source commits（与§6反向对应）**:
  - [6e43939a3a864100730af42b5906e6b73a68128d](https://github.com/wimi321/lizzieyzy-next/commit/6e43939a3a864100730af42b5906e6b73a68128d) — Merge pull request #487 from wimi321/build/katago-source-rocm
  - [8ab44dc21d520c5c0bfb8f937c3fd87c28267631](https://github.com/wimi321/lizzieyzy-next/commit/8ab44dc21d520c5c0bfb8f937c3fd87c28267631) — Measure HTTP timeout independently of Windows interpreter startup
  - [dd184f5125a9f80a695761497193fa885269dd2e](https://github.com/wimi321/lizzieyzy-next/commit/dd184f5125a9f80a695761497193fa885269dd2e) — Merge remote-tracking branch 'origin/main' into build/katago-source-rocm
  - [6b31e28564a302722fcc8b01607ff0ddc826dcaa](https://github.com/wimi321/lizzieyzy-next/commit/6b31e28564a302722fcc8b01607ff0ddc826dcaa) — Merge remote-tracking branch 'origin/main' into build/katago-source-rocm
  - [c5545229703000f3183e907815f6863bbb4601e5](https://github.com/wimi321/lizzieyzy-next/commit/c5545229703000f3183e907815f6863bbb4601e5) — Preserve hidden dependency license files in source artifacts
  - [3803a62cb5b64e28f78dd6f780a8da05c4f271fc](https://github.com/wimi321/lizzieyzy-next/commit/3803a62cb5b64e28f78dd6f780a8da05c4f271fc) — Normalize HIP compiler paths before CMake serialization
  - [86a8a862ce0f8a13a907a122097f81ae01a746f7](https://github.com/wimi321/lizzieyzy-next/commit/86a8a862ce0f8a13a907a122097f81ae01a746f7) — Supply pinned HIP device libraries to the Windows compiler
  - [8e44c9010618322a95715cb6c9457afdb461c9ad](https://github.com/wimi321/lizzieyzy-next/commit/8e44c9010618322a95715cb6c9457afdb461c9ad) — Retain ROCm compiler preflight diagnostics before configure
  - [31ab10c39f9d6e1297ea4d63b94427b921e63e05](https://github.com/wimi321/lizzieyzy-next/commit/31ab10c39f9d6e1297ea4d63b94427b921e63e05) — Build pinned ROCm engines with unchanged family runtimes
- **发布包含**: 最早有API/ancestry证据收录 `next-2026-09-18.2`（预发布）；全部source也含于正式09-26.1/.2。准备target仅记prepared，不由本字段推断准备版本已发布。
- **实际behavior / sourceDelta**: PR487保留HIP/CMake路径、设备库、预检诊断、源码归档许可证及HTTP与Python冷启动测量分离。构建工具本身不移植；AMD/ROCm用户选择可用引擎及看到不兼容原因的目标保留。15目标不是本票新承诺全GPU实际通过。
- **Canonical owner / scope**: T01-RESOURCE；本地资源后继（不占RCOMP-01）。
- **候选/覆盖与边界**: 尚无此新增行为的可继承运行证据；不把Java修复判为当前Next缺陷。
- **Disposition**: 混合：Java构建内部排除；用户兼容需功能后继。
- **责任意图**: T04-RESOURCE。
- **Observable assertions**: 按准入本地ROCm资源核对来源/版本/路径/完整性与adapter能力；缺失依赖或能力拒绝可见，棋谱不变、不隐式换后端。真实兼容由功能票获取；CMake/CI无需Tauri功能票。

<a id="ud-04-002"></a>
### UD-04-002 Pinned 15目标目录激活、资源来源与离线选择

- **Source / event**: PR #493 (集成事件 2) | 共 10 个提交；第一父E02。
- **完整source commits（与§6反向对应）**:
  - [2c379b4f511f17fdc5bed2c6a4d184f766e8ee80](https://github.com/wimi321/lizzieyzy-next/commit/2c379b4f511f17fdc5bed2c6a4d184f766e8ee80) — Merge pull request #493 from wimi321/release/activate-pinned-katago
  - [b9b80a59f0f1868cabe8884abf26af3c08ecdaf6](https://github.com/wimi321/lizzieyzy-next/commit/b9b80a59f0f1868cabe8884abf26af3c08ecdaf6) — Merge remote-tracking branch 'origin/main' into release/activate-pinned-katago
  - [6cf94ee206948709e81e90f7031c35a727ae9e40](https://github.com/wimi321/lizzieyzy-next/commit/6cf94ee206948709e81e90f7031c35a727ae9e40) — Test valid official catalog rejection without network access
  - [00465a1d93f4abc9d25badf5c4b0abd00a2921fe](https://github.com/wimi321/lizzieyzy-next/commit/00465a1d93f4abc9d25badf5c4b0abd00a2921fe) — Keep the official-catalog rejection fixture independent of release activation
  - [1b7029d147b60600afe5eda831654ab10d75afbb](https://github.com/wimi321/lizzieyzy-next/commit/1b7029d147b60600afe5eda831654ab10d75afbb) — Isolate Draft asset provisioning from unprivileged native acceptance
  - [418f5f3cbbc5018437915a761ba82c8acf29c895](https://github.com/wimi321/lizzieyzy-next/commit/418f5f3cbbc5018437915a761ba82c8acf29c895) — Merge remote-tracking branch 'origin/main' into release/activate-pinned-katago
  - [dfd7362a3b1bf5e76ae1327190e5a9aa286da036](https://github.com/wimi321/lizzieyzy-next/commit/dfd7362a3b1bf5e76ae1327190e5a9aa286da036) — Match installer assertions to the reviewed engine catalog
  - [255028f76e33dcd0d9f3236bd9447fba89054680](https://github.com/wimi321/lizzieyzy-next/commit/255028f76e33dcd0d9f3236bd9447fba89054680) — Activate all 15 audited focus-capable engine archives behind a draft acceptance gate
  - [e67c5a3b47f45c4bb27d334f4163f8c8dced1f0f](https://github.com/wimi321/lizzieyzy-next/commit/e67c5a3b47f45c4bb27d334f4163f8c8dced1f0f) — Merge remote-tracking branch 'origin/main' into release/activate-pinned-katago
  - [0a6546f4a7e5e671c9b414eb8eb736c15eb151dc](https://github.com/wimi321/lizzieyzy-next/commit/0a6546f4a7e5e671c9b414eb8eb736c15eb151dc) — Prepare source-engine activation contracts and installation guidance
- **发布包含**: 最早有API/ancestry证据收录 `next-2026-09-18.2`（预发布）；全部source也含于正式09-26.1/.2。准备target仅记prepared，不由本字段推断准备版本已发布。
- **实际behavior / sourceDelta**: PR493激活15个平台/后端资源目录与固定源构建身份、安装/文档契约和离线网络拒绝。目录“存在”不同于资源可用、实际GPU认证；项目构建和官方binary须区别。
- **Canonical owner / scope**: ENG-01原profile资产检查；REL-05功能部分/T01-RESOURCE。
- **候选/覆盖与边界**: 尚无此新增行为的可继承运行证据；不把Java修复判为当前Next缺陷。
- **Disposition**: 需资源功能后继，原ENG-01范围不变。
- **责任意图**: T04-RESOURCE。
- **Observable assertions**: 目录来源/版本/后端/archive/executable身份可追踪；合格本地选取不需Release，离线获取失败可见且不错误标Ready；模型/配置/协议能力不兼容时拒绝并保持当前棋谱。

<a id="ud-04-006"></a>
### UD-04-006 macOS打包二进制完整性

- **Source / event**: PR #497 (集成事件 6) | 共 2 个提交；第一父E06。
- **完整source commits（与§6反向对应）**:
  - [1a70ca2e5468ff324f8459f73a5ad51f7c83bf2c](https://github.com/wimi321/lizzieyzy-next/commit/1a70ca2e5468ff324f8459f73a5ad51f7c83bf2c) — Merge pull request #497 from wimi321/fix/macos-source-integrity
  - [2a77ca55bd8a5281dfd1e8b3e4f7d8dfe4b84e31](https://github.com/wimi321/lizzieyzy-next/commit/2a77ca55bd8a5281dfd1e8b3e4f7d8dfe4b84e31) — Preserve reviewed KataGo bytes across macOS app packaging
- **发布包含**: 最早有API/ancestry证据收录 `next-2026-09-18.2`（预发布）；全部source也含于正式09-26.1/.2。准备target仅记prepared，不由本字段推断准备版本已发布。
- **实际behavior / sourceDelta**: PR497在macOS packager/audit中核对最终KataGo二进制与源收据，防止bundle静默污染。shell/DMG布局不复制；用户资源被修改不能仍宣称可信的目标保留。
- **Canonical owner / scope**: Java打包内部；T01-RESOURCE共用完整性目标。
- **候选/覆盖与边界**: 尚无此新增行为的可继承运行证据；不把Java修复判为当前Next缺陷。
- **Disposition**: 内部机制排除；功能完整性由资源合同承接。
- **责任意图**: T04-RESOURCE。
- **Observable assertions**: 安装前后的managed executable变更不能复用原可信身份；功能资源验证可用合格本地文件。正式签名导致的身份变化及最终包验收留发行，不要求本票运行macOS或签名。

<a id="ud-04-022"></a>
### UD-04-022 Pinned archive云端中转校验

- **Source / event**: PR #513 (集成事件 22) | 共 2 个提交；第一父E22。
- **完整source commits（与§6反向对应）**:
  - [cb987e2fce767af89b708653af15b01ede403a9d](https://github.com/wimi321/lizzieyzy-next/commit/cb987e2fce767af89b708653af15b01ede403a9d) — Merge pull request #513 from wimi321/fix/source-archive-cloud-transfer-20260918
  - [a4e1872bab549401b82548a682a5e97ebc9e40eb](https://github.com/wimi321/lizzieyzy-next/commit/a4e1872bab549401b82548a682a5e97ebc9e40eb) — Add verified cloud transfer for pinned release engine archives
- **发布包含**: 最早有API/ancestry证据收录 `next-2026-09-18.2`（预发布）；全部source也含于正式09-26.1/.2。准备target仅记prepared，不由本字段推断准备版本已发布。
- **实际behavior / sourceDelta**: PR513的transfer-pinned-source-assets workflow转移核准archive、核对身份；不增加桌面入口。T01-RESOURCE已保留消费者来源/完整性义务，不复制upstream云构建/中转。
- **Canonical owner / scope**: Java CI/源资产转移。
- **候选/覆盖与边界**: 静态文件/来源排除。
- **Disposition**: 仅Java内部不适用。
- **责任意图**: 无独立产品任务。
- **Observable assertions**: 记录transfer源与资源consumer分界；无Taurifeature任务、无upstream编译/真实上传义务。

<a id="ud-04-026"></a>
### UD-04-026 TensorRT受控修复、schema2资源身份及原引擎rollback

- **Source / event**: PR #519 (集成事件 26) | 共 19 个提交；第一父E26。
- **完整source commits（与§6反向对应）**:
  - [6ec5df985be570c6b03f65b00b1b08944d9bb1e6](https://github.com/wimi321/lizzieyzy-next/commit/6ec5df985be570c6b03f65b00b1b08944d9bb1e6) — Merge pull request #519 from qiyi71w/integration/tensorrt-candidate
  - [8e876f251983361b7d68bd13c4120b0f2430b8cb](https://github.com/wimi321/lizzieyzy-next/commit/8e876f251983361b7d68bd13c4120b0f2430b8cb) — chore: merge distribution runtime acceptance
  - [f2645998827b0593447f82ed8c390640037f4454](https://github.com/wimi321/lizzieyzy-next/commit/f2645998827b0593447f82ed8c390640037f4454) — fix(test): await Windows process teardown
  - [188c925d5a91c429bb7701b128088df75db88dc2](https://github.com/wimi321/lizzieyzy-next/commit/188c925d5a91c429bb7701b128088df75db88dc2) — fix(engine): complete deferred KataGo rollback
  - [1164bdabd6684e3b8921abedbef159b60a9afbe4](https://github.com/wimi321/lizzieyzy-next/commit/1164bdabd6684e3b8921abedbef159b60a9afbe4) — fix(acceptance): preserve candidate identity
  - [a13665dd49d50bb8b5c1e760755494e1dbb339df](https://github.com/wimi321/lizzieyzy-next/commit/a13665dd49d50bb8b5c1e760755494e1dbb339df) — ci: make KataGo source builds manual-only
  - [78dc2c247067f6b00a72745db4b88c3905292cfc](https://github.com/wimi321/lizzieyzy-next/commit/78dc2c247067f6b00a72745db4b88c3905292cfc) — ci: scope source builds to platform inputs
  - [c8b3d5b9b7d6cb97d210636ed203fecbbc04e930](https://github.com/wimi321/lizzieyzy-next/commit/c8b3d5b9b7d6cb97d210636ed203fecbbc04e930) — test(release): align source zlib fixture
  - [3a7ea04501f6b85d943f2f160e4f412fd230e64d](https://github.com/wimi321/lizzieyzy-next/commit/3a7ea04501f6b85d943f2f160e4f412fd230e64d) — fix(tensorrt): clarify modern GPU support
  - [258303be3c105e8a2d3d872250808c75721ae7ea](https://github.com/wimi321/lizzieyzy-next/commit/258303be3c105e8a2d3d872250808c75721ae7ea) — fix(tensorrt): trust static zlib provenance
  - [d5a0e2ed648df262b9e8f82cf6ef999c131ba9de](https://github.com/wimi321/lizzieyzy-next/commit/d5a0e2ed648df262b9e8f82cf6ef999c131ba9de) — ci: consume existing product artifacts
  - [41f4cff8961d1138854a0ce449bcba64f9df0787](https://github.com/wimi321/lizzieyzy-next/commit/41f4cff8961d1138854a0ce449bcba64f9df0787) — fix(release): validate packaged JVM host
  - [a5d1e8ecae41c418612926d04855aca02a14f0cd](https://github.com/wimi321/lizzieyzy-next/commit/a5d1e8ecae41c418612926d04855aca02a14f0cd) — test: keep comment worker Java 17 compatible
  - [ed64305b5a8d352ccc19d1cec807f83322e57f22](https://github.com/wimi321/lizzieyzy-next/commit/ed64305b5a8d352ccc19d1cec807f83322e57f22) — fix(tensorrt): complete controlled repair flow
  - [c1808af0e62367989d51f6c8f81c7a8ab50c5998](https://github.com/wimi321/lizzieyzy-next/commit/c1808af0e62367989d51f6c8f81c7a8ab50c5998) — ci(release): add candidate acceptance workflows
  - [58cdb5512d3de1ccccf17cc01311354ed66e3345](https://github.com/wimi321/lizzieyzy-next/commit/58cdb5512d3de1ccccf17cc01311354ed66e3345) — feat(runtime): verify standalone Java 17
  - [72ff67ebac41ac4ab924ee0e2d2e6be51b63dde0](https://github.com/wimi321/lizzieyzy-next/commit/72ff67ebac41ac4ab924ee0e2d2e6be51b63dde0) — feat(release): add macOS product acceptance
  - [80133df74f62938366fa3444aa1862b8a8465b76](https://github.com/wimi321/lizzieyzy-next/commit/80133df74f62938366fa3444aa1862b8a8465b76) — feat(release): add Linux product acceptance
  - [29ec26e316c642f5b70386f71d98f315c5926bd9](https://github.com/wimi321/lizzieyzy-next/commit/29ec26e316c642f5b70386f71d98f315c5926bd9) — feat(release): add Windows product acceptance
- **发布包含**: 最早有API/ancestry证据收录 `next-2026-09-22.2`（预发布）；全部source也含于正式09-26.1/.2。准备target仅记prepared，不由本字段推断准备版本已发布。
- **实际behavior / sourceDelta**: 188c925d修复ordinary failed-switch恢复原引擎的exact deferred reader incarnation：completion释放endpoint后可继续探测，但旧analysis在新的physical analysis write前quarantine。不是TensorRT失败自动选择CUDA/OpenCL。ed64305b等保留显式controlled repair、GPU/backend适配提示，a13665dd/78dc2c24等CI/JVM属于内部。258303be的managed catalog+installed manifest schema2绑定origin、source commit、asset ID/name/archive digest、executable digest、backend、zlib linkage；仅精确verified managed static-zlib身份省略dynamic zlib DLL group，official/external/unknown/modified仍遵守自身依赖合同。
- **Canonical owner / scope**: ENG-03/04原switch；T01-RESOURCE功能；未分配controlled-repair后继。
- **候选/覆盖与边界**: H-SWITCH对应实际asset失败与原stale fixture；无TensorRT修复/GPU或Java deferred reader完整native证明。
- **Disposition**: 原switch窄覆盖；资源/修复后继；reader差异调查。
- **责任意图**: T04-RESOURCE、T04-ROLLBACK。
- **Observable assertions**: B失败保留仍Ready A，A已退出按终态No-engine，不implicit profile/backend promotion；exact reader允许settle但旧analysis不能跨incarnation或physical-write fence。显式修复需重新验证原目标身份；modified/external不能借managed receipt绕过DLL要求；catalog/manifests不一致可见拒绝。

<a id="ud-04-030"></a>
### UD-04-030 模型header身份与同路径替换

- **Source / event**: PR #527 (集成事件 30) | 共 1 个提交；第一父E30。
- **完整source commits（与§6反向对应）**:
  - [6846db3aba385a6eb051f558d856dfe37eedebae](https://github.com/wimi321/lizzieyzy-next/commit/6846db3aba385a6eb051f558d856dfe37eedebae) — fix(katago): read weight identity from headers (#527)
- **发布包含**: 最早有API/ancestry证据收录 `next-2026-09-22.2`（预发布）；全部source也含于正式09-26.1/.2。准备target仅记prepared，不由本字段推断准备版本已发布。
- **实际behavior / sourceDelta**: 6846db3a从KataGo权重header取得模型身份，不靠file name猜猜；更新目录/配置展示，同路径替换须重新识别，未知/custom模型保留，不静默改saved launch inputs。
- **Canonical owner / scope**: ENG-01原profile；资源识别未分配后继。
- **候选/覆盖与边界**: 尚无此新增行为的可继承运行证据；不把Java修复判为当前Next缺陷。
- **Disposition**: 需模型识别功能后继。
- **责任意图**: T04-MODEL-IDENTITY。
- **Observable assertions**: 有效gzip/header身份识别；损坏/未知可见处理，不能删custom模型；同路径同尺寸/时间等替换不继续用旧identity；显示名和saved identity/实际路径不同，原命令不被嗅探改写。

<a id="ud-04-031"></a>
### UD-04-031 Bundled profile所有权与显示名称解耦

- **Source / event**: PR #528 (集成事件 31) | 共 1 个提交；第一父E31。
- **完整source commits（与§6反向对应）**:
  - [aee0e708b534789f0f46c4fd9bd7300c33ccdb49](https://github.com/wimi321/lizzieyzy-next/commit/aee0e708b534789f0f46c4fd9bd7300c33ccdb49) — fix(katago): preserve renamed bundled profiles (#528)
- **发布包含**: 最早有API/ancestry证据收录 `next-2026-09-22.2`（预发布）；全部source也含于正式09-26.1/.2。准备target仅记prepared，不由本字段推断准备版本已发布。
- **实际behavior / sourceDelta**: aee0e708记录bundle ownership独立于用户名称，保护重命名及修改的命令、旧配置来源迁移，防rediscovery静默覆盖custom化。不是新增ENG-03配置定义，也不从源码强定Next ID格式/UUID或新增Reset UI。
- **Canonical owner / scope**: ENG-01原profile；REL-05 managed resource后继。
- **候选/覆盖与边界**: 尚无此新增行为的可继承运行证据；不把Java修复判为当前Next缺陷。
- **Disposition**: 需managed ownership后继，Accepted存储不重定义。
- **责任意图**: T04-MODEL-IDENTITY。
- **Observable assertions**: 重命名+修改命令+重开仍保留ownership/参数；rediscovery不能因name判managed或覆盖manual command；无法证明旧ownership时显式未知而不是自动接管。

<a id="ud-04-037"></a>
### UD-04-037 默认B11升级11750M资源身份

- **Source / event**: PR #534 (集成事件 37) | 共 2 个提交；第一父E37。
- **完整source commits（与§6反向对应）**:
  - [0bc7707a44a2e6545299dbff174afde0d744c6db](https://github.com/wimi321/lizzieyzy-next/commit/0bc7707a44a2e6545299dbff174afde0d744c6db) — Merge pull request #534 from wimi321/codex/upgrade-b11-11750m-20260922
  - [498f5f05119e7a7cbab6ed124d8656fcd7940196](https://github.com/wimi321/lizzieyzy-next/commit/498f5f05119e7a7cbab6ed124d8656fcd7940196) — chore(models): upgrade default B11 to 11750M checkpoint
- **发布包含**: 最早有API/ancestry证据收录 `next-2026-09-26.1`（正式发布）；全部source也含于正式09-26.1/.2。准备target仅记prepared，不由本字段推断准备版本已发布。
- **实际behavior / sourceDelta**: 498f5f05将默认B11 checkpoint升级至11750M并改model目录/元数据。保留实际默认模型变化，不用“任意等价稳定版”抹掉上游变化；是否采用默认版本由资源功能合同记录。任意默认替换不是已批准。
- **Canonical owner / scope**: T01-RESOURCE；REL-05功能/发行分立。
- **候选/覆盖与边界**: 尚无此新增行为的可继承运行证据；不把Java修复判为当前Next缺陷。
- **Disposition**: 需功能资源来源与兼容后继。
- **责任意图**: T04-RESOURCE。
- **Observable assertions**: 指明选择的B11-11750M来源、digest、路径/引擎协议兼容；合格本地模型可验，无Release要求；已有custom active选择不被默认catalog升级覆写，missing/incompatible可见且不假Ready。

<a id="ud-04-038"></a>
### UD-04-038 PR534二次进入Windows integration

- **Source / event**: PR #534 (集成事件 38) | 共 1 个提交；第一父E38。
- **完整source commits（与§6反向对应）**:
  - [886bd81d0c01c8a3b57940dfab15c228a2e12146](https://github.com/wimi321/lizzieyzy-next/commit/886bd81d0c01c8a3b57940dfab15c228a2e12146) — QA integrate PR 534
- **发布包含**: 最早有API/ancestry证据收录 `next-2026-09-26.1`（正式发布）；全部source也含于正式09-26.1/.2。准备target仅记prepared，不由本字段推断准备版本已发布。
- **实际behavior / sourceDelta**: 886bd81d仅把PR534已拥有的model升级合入E38。源merge保留、模型能力不重复计数。
- **Canonical owner / scope**: UD-04-037同一资源能力。
- **候选/覆盖与边界**: 没有独立运行继承。
- **Disposition**: 仅integration归属；不重复feature。
- **责任意图**: 关联T04-RESOURCE，无独立产品任务。
- **Observable assertions**: event E38/source full SHA引用解得UD-04-037模型合同，merge无额外accepted/result。

<a id="ud-04-043"></a>
### UD-04-043 模型目录immutable snapshot、busy gate与refresh

- **Source / event**: PR #541 (集成事件 38) | 共 6 个提交；第一父E38。
- **完整source commits（与§6反向对应）**:
  - [b7da149c6c8dc30828321b2c509d576727cec9a7](https://github.com/wimi321/lizzieyzy-next/commit/b7da149c6c8dc30828321b2c509d576727cec9a7) — test(gui): initialize completion buttons in catalog fixture
  - [b19808838c10112962fdd6c338a1e21463f454a0](https://github.com/wimi321/lizzieyzy-next/commit/b19808838c10112962fdd6c338a1e21463f454a0) — fix(gui): keep failed catalog model actions disabled
  - [5c96e1979583173c4ba3ac0f086e0a94eb910d89](https://github.com/wimi321/lizzieyzy-next/commit/5c96e1979583173c4ba3ac0f086e0a94eb910d89) — fix(gui): share busy controls during catalog refresh
  - [5414286e949204f01a077ce772566103eb69541e](https://github.com/wimi321/lizzieyzy-next/commit/5414286e949204f01a077ce772566103eb69541e) — fix(gui): restore refresh controls and revalidate weight selection
  - [94cdfc191e88ac404d9d7666ec645e15eccc6660](https://github.com/wimi321/lizzieyzy-next/commit/94cdfc191e88ac404d9d7666ec645e15eccc6660) — perf(gui): reuse immutable background weight catalog snapshots
  - [e91904197ca0ef922b2650299c0862d70e7275a8](https://github.com/wimi321/lizzieyzy-next/commit/e91904197ca0ef922b2650299c0862d70e7275a8) — QA integrate PR 541
- **发布包含**: 最早有API/ancestry证据收录 `next-2026-09-26.1`（正式发布）；全部source也含于正式09-26.1/.2。准备target仅记prepared，不由本字段推断准备版本已发布。
- **实际behavior / sourceDelta**: PR541后台目录snapshot复用、失败模型动作不误enable、busy gate跨refresh/setup/model操作、恢复refresh并revalidate选择。源码含test-only支撑，不让“test”吞生产用户goal。
- **Canonical owner / scope**: 本地resource/setup后继；ENG-01原存储不扩大。
- **候选/覆盖与边界**: 尚无此新增行为的可继承运行证据；不把Java修复判为当前Next缺陷。
- **Disposition**: 需功能后继。
- **责任意图**: T04-RESOURCE、T04-MODEL-IDENTITY。
- **Observable assertions**: slow refresh不在render重hash/model扫描；冲突动作和已排队callbacks不会绕busy；失败模型不应用，当前saved engine identity固定；refresh不丢installed candidates或custom inputs，旧snapshot可见失效。

<a id="ud-05-05"></a>
### UD-05-05 

- 来源：[011](https://github.com/wimi321/lizzieyzy-next/commit/7b4ebcb87a5341efa97ae24c7e3c605e1d81b9a7), [012](https://github.com/wimi321/lizzieyzy-next/commit/e9481f9a43ad817d80292cc1819a23c10d0d72d1), [048](https://github.com/wimi321/lizzieyzy-next/commit/410f36dc9fe58a099785cf0f5fc1701cb9ca179b), [049](https://github.com/wimi321/lizzieyzy-next/commit/f59c59232119cfc47ad97a21ede94e6747d05b19), [051](https://github.com/wimi321/lizzieyzy-next/commit/dcd4803586328dd71419eb301a217fec5d20a7a5), [052](https://github.com/wimi321/lizzieyzy-next/commit/91ba01d3bd542a1ab14f03c8b970ee5a38c6db36), [053](https://github.com/wimi321/lizzieyzy-next/commit/01c7eafdd50b698c6d53e9515e355180829a4e39), [054](https://github.com/wimi321/lizzieyzy-next/commit/2a9a10ddd0ac7b28fda63e110c2de428128891fc), [055](https://github.com/wimi321/lizzieyzy-next/commit/d21aa344a339ed4b1a87f0135c23c92a3f5a30d7), [056](https://github.com/wimi321/lizzieyzy-next/commit/0abe772bac0494517de33f61c39dc839ec7f8246), [077](https://github.com/wimi321/lizzieyzy-next/commit/0fe5b3769e5f1402013df5520643466ff021f5e6)（完整SHA/发布包含见源索引）。
- 用户行为/边界：B11模型更新、性能/棋力提示、间接launcher模型身份识别与基准报告滚动/布局。
- Tauri映射/具名任务意图：ENG-09（多配置）与 REL-05（功能资源）为受影响现有边界，不代表模型身份/性能提示已验收；新增责任 T05-MODEL-IDENTITY（B），最终后继 ID 由06分配。
- 覆盖证据或缺口：PR552/569/576。Tauri资源路径已有Partial，不能据此声称B11/间接启动识别、性能提示已完成。
- 处置：后继/适配调查；只要求识别实际支持资源，不强制Java打包版本或内建benchmark布局。
- 后续验收/调查停止条件：选择不同模型/间接启动时标识真实来源版本与路径；不伪报棋力；缺资源可见失败，无引擎照常复盘。

<a id="ud-05-27"></a>
### UD-05-27 

- 来源：[127](https://github.com/wimi321/lizzieyzy-next/commit/41f9d9476492e5d104c971191e17d654b92f2809), [128](https://github.com/wimi321/lizzieyzy-next/commit/a4520557ebae17026e19d4ffb1f0a4417a936987), [131](https://github.com/wimi321/lizzieyzy-next/commit/ba42fe4673cbf4498222500c93e705a1b6c96307), [137](https://github.com/wimi321/lizzieyzy-next/commit/504cfbcf32d6d1d8bcb52d02b01b508fc82fe408), [141](https://github.com/wimi321/lizzieyzy-next/commit/03202192ab183a5c0a0e71e803127fbfca8d6e17), [143](https://github.com/wimi321/lizzieyzy-next/commit/b82b6611b7dc4ce92d1a0f5e2d2063b585201617)（完整SHA/发布包含见源索引）。
- 用户行为/边界：HumanSL companion static-zlib信任必须有固定清单+精确摘要；实际启动的伴随进程也检查CUDA/cuDNN/NVRTC；TensorRT额外parser DLL缺失可见且不误报CUDA。
- Tauri映射/具名任务意图：GAME-08、ENG-09、REL-05功能资源；T05-RUNTIME-COMPATIBILITY（B/D）
- 覆盖证据或缺口：PR593/596，KataGoRuntimeHelper只免除已验证static zlib，不免除其他依赖；Java打包旁路不是Next准入方案。
- 处置：保留真实资源兼容功能，Java伴随文件名/打包结构不适用；不得因为发行后置删掉运行时义务。
- 后续验收/调查停止条件：用户本地合法资源含缺/错版本库、损坏模型、错误来源；显示实际失败exe及修复指引；真实HumanSL与目标硬件另验，缺硬件Blocked。

<a id="udx-003"></a>
## UDX-003 — HumanSL独立AI Coach与恢复

当前阶段导航 **R15**（历史来源组 **D**；混合归属以意图索引为准）；条目/后继 **GAME-08**。各来源的有界调查/决定仍须先输出结论，不能把族导航视为新增实现或全部Accepted。明确的其他阶段消费者按其任务合同单独交付；其存在不阻塞独立本地工作。

<a id="ud-03-007"></a>
### UD-03-007 — HumanSL 战术候选守卫与时间自适应

- **来源 PR 与提交**:
  - 主来源: `PR #388`
  - 完整 40 位 Commit SHA (3 个):
    - `89cf4731016c01fa1d7deffb45ba769ee3064c30` (next-2026-08-30.1) — Merge pull request #388 from wimi321/fix/humansl-tactical-quality-guard
    - `b164ad6ceea1e9866184d83d2d20abb2fa08cb16` (next-2026-08-30.1) — Adapt AI Coach search depth to think time
    - `714e032279996616f2fe087e3db6c59e0dc95b6d` (next-2026-08-30.1) — Improve AI Coach tactical move safety
- **核心文件出处**: `src/main/java/featurecat/lizzie/analysis/HumanLikeMoveSelector.java`, `src/main/java/featurecat/lizzie/analysis/HumanSlAnalysisRunner.java`
- **用户可见行为与边界**: 保留战术着手安全与依据 think time 调整候选搜索深度；不是通过一句“不会犯错”承诺所有棋力。
- **对等项 / 任务映射**: GAME-08（Deferred）；T02-GAME-08；`T03-GAME08-HUMANSL-SAFETY`。所有历史项仅引用Matrix原范围，最终后继ID由06分配。
- **Tauri 现状 / 证据与残余**: 尚未准入 HumanSL。
- **处置结论**: 需后继功能/修复。任务 kind=`feature`；不把Source差异等同Next缺陷。
- **功能断言 / 调查停止条件**: 来源测试的提子/征子/死活候选守卫可复现；不同有效 think time 按来源策略选择搜索预算，返回合法落子或可见失败；不以棋力主观评分代替策略断言。
- **实际阻塞与未来证据门**: T02-GAME-08 的准入 HumanSL 引擎/权重与真实对局；合并04 #506/#508预算演进。
- **唯一批次建议 / 责任**: D；`T03-GAME08-HUMANSL-SAFETY` 拥有此Delta的结果。07/08冻结具体实施合同，06负责同源去重。

<a id="ud-03-009"></a>
### UD-03-009 — HumanSL 结束恢复、成功设置与局后复盘

- **来源 PR 与提交**:
  - 主来源: `PR #371`
  - 完整 40 位 Commit SHA (1 个):
    - `22128b6e2f9cba91701f157e8c6af2d205640a4f` (next-2026-08-31.1) — fix(ai-coach): restore analysis and settings safely (#371)
- **核心文件出处**: `src/main/java/featurecat/lizzie/analysis/ExactSnapshotEngineRestore.java`, `src/main/java/featurecat/lizzie/gui/HumanSlGameController.java`, `src/main/java/featurecat/lizzie/gui/Menu.java`, `src/main/java/featurecat/lizzie/rules/Board.java`
- **用户可见行为与边界**: AI Coach 结束先完成本地/知子云前台交接，成功才触发局后复盘；保留最近成功设置和稳定工具栏，失败不能记作成功恢复。
- **对等项 / 任务映射**: GAME-08（Deferred）；ENG-02（Accepted，基础 Run）；`T03-GAME08-RESTORE-FOREGROUND`。所有历史项仅引用Matrix原范围，最终后继ID由06分配。
- **Tauri 现状 / 证据与残余**: 已有其他 Match 交接不证明 Coach 专用恢复。
- **处置结论**: 需后继功能/修复。任务 kind=`feature`；不把Source差异等同Next缺陷。
- **功能断言 / 调查停止条件**: 结束/取消成功恢复原配置后才开始复盘；恢复失败可见且不继续复盘；失败/取消设置不替换 last-successful；工具栏仍可操作。
- **实际阻塞与未来证据门**: T02-GAME-08；所借前台/远程能力及T01-REMOTE真实服务条件。
- **唯一批次建议 / 责任**: D；`T03-GAME08-RESTORE-FOREGROUND` 拥有此Delta的结果。07/08冻结具体实施合同，06负责同源去重。

<a id="ud-04-012"></a>
### UD-04-012 HumanSL继承线程alias的launch overlay

- **Source / event**: PR #503 (集成事件 12) | 共 3 个提交；第一父E12。
- **完整source commits（与§6反向对应）**:
  - [e30018411b2ab54e6844ec567ff6fb5e2bf47aeb](https://github.com/wimi321/lizzieyzy-next/commit/e30018411b2ab54e6844ec567ff6fb5e2bf47aeb) — Merge pull request #503 from wimi321/fix/humansl-thread-alias-20260917
  - [eb7f8ab12139739c2d07b0983b0f90e2f2f5151a](https://github.com/wimi321/lizzieyzy-next/commit/eb7f8ab12139739c2d07b0983b0f90e2f2f5151a) — Synchronize diagnostic and socket test cleanup on completion
  - [0492a9367bfeac569b1e9dcb5ba04d7a12c4d4ee](https://github.com/wimi321/lizzieyzy-next/commit/0492a9367bfeac569b1e9dcb5ba04d7a12c4d4ee) — Clear the inherited GTP thread alias for HumanSL launches
- **发布包含**: 最早有API/ancestry证据收录 `next-2026-09-18.2`（预发布）；全部source也含于正式09-26.1/.2。准备target仅记prepared，不由本字段推断准备版本已发布。
- **实际behavior / sourceDelta**: 0492a936只清除继承numSearchThreads GTP alias，并以HumanSL launch overlay设置numAnalysisThreads=1、numSearchThreadsPerAnalysisThread=8等；不改用户配置文件，也不是禁止全部继承主命令。
- **Canonical owner / scope**: GAME-08 Deferred；T02-GAME-08。
- **候选/覆盖与边界**: 尚无此新增行为的可继承运行证据；不把Java修复判为当前Next缺陷。
- **Disposition**: 需HumanSL功能增量。
- **责任意图**: T04-HUMANSL。
- **Observable assertions**: 在config已有alias和inline override两种源下，HumanSL有效thread precedence明确且不会二次被GTP alias夺回；config原字节与saved command不变，普通Run参数不被HumanSL overlay污染。

<a id="ud-04-015"></a>
### UD-04-015 HumanSL JSON逐阶段剩余时间与return reserve

- **Source / event**: PR #506 (集成事件 15) | 共 2 个提交；第一父E15。
- **完整source commits（与§6反向对应）**:
  - [8705f20827eb434f11cc00070ff6d61c716e2463](https://github.com/wimi321/lizzieyzy-next/commit/8705f20827eb434f11cc00070ff6d61c716e2463) — Merge pull request #506 from wimi321/fix/humansl-engine-time-budget-20260917
  - [9599cfac1a2be63de31a506de5b283136e04bae7](https://github.com/wimi321/lizzieyzy-next/commit/9599cfac1a2be63de31a506de5b283136e04bae7) — Respect remaining HumanSL move time in KataGo searches
- **发布包含**: 最早有API/ancestry证据收录 `next-2026-09-18.2`（预发布）；全部source也含于正式09-26.1/.2。准备target仅记prepared，不由本字段推断准备版本已发布。
- **实际behavior / sourceDelta**: 9599cfac将policy probe/verification/deepen各JSON analysis request的overrideSettings.maxTime绑定剩余move期限，并给final batch、JSON输出和controller delivery留reserve。maxVisits不是clock；其早期低root推断被UD-04-017后续修正。
- **Canonical owner / scope**: GAME-08 Deferred；T02-GAME-08。
- **候选/覆盖与边界**: 尚无此新增行为的可继承运行证据；不把Java修复判为当前Next缺陷。
- **Disposition**: 需HumanSL功能增量。
- **责任意图**: T04-HUMANSL。
- **Observable assertions**: 每阶段用实时剩余deadline扣return/delivery reserve，maxTime在overrideSettings内；短/零预算不负值，无预算时不继续搜索；未验证policy不能变合法AI结果；c1ec182d最终规则优先于早期root-timeout推断。

<a id="ud-04-017"></a>
### UD-04-017 HumanSL weightless root计数与已验证child evidence

- **Source / event**: PR #508 (集成事件 17) | 共 2 个提交；第一父E17。
- **完整source commits（与§6反向对应）**:
  - [e504299224559eaa96468facee0f70630e5e96a6](https://github.com/wimi321/lizzieyzy-next/commit/e504299224559eaa96468facee0f70630e5e96a6) — Merge pull request #508 from wimi321/fix/humansl-weightless-budget-20260917
  - [c1ec182d3ac0a02c8e9bae191ae03c8863b7677c](https://github.com/wimi321/lizzieyzy-next/commit/c1ec182d3ac0a02c8e9bae191ae03c8863b7677c) — Preserve adaptive HumanSL search with weightless visit accounting
- **发布包含**: 最早有API/ancestry证据收录 `next-2026-09-18.2`（预发布）；全部source也含于正式09-26.1/.2。准备target仅记prepared，不由本字段推断准备版本已发布。
- **实际behavior / sourceDelta**: c1ec182d的最终deepen依据prior requested limit、已用与剩余时间，不因low root visits推断超时或压制deepen；child visits包括weightless探索。deeper返回空或已知更低child evidence时保留原verified结果；更低root但更多child evidence可以采用。不是网络尺寸/概率漂移修复。
- **Canonical owner / scope**: GAME-08 Deferred；T02-GAME-08。
- **候选/覆盖与边界**: 尚无此新增行为的可继承运行证据；不把Java修复判为当前Next缺陷。
- **Disposition**: 需HumanSL功能增量。
- **责任意图**: T04-HUMANSL。
- **Observable assertions**: low-root/high-child仍可deepen；elapsed+remaining不足minimum gain时停止；empty/weaker deeper结果不覆盖已验证结果；lower-root/stronger-child可更新；保持已验证move集合和deadline，无raw policy假fallback。

<a id="udx-004"></a>
## UDX-004 — 现有Run/reader/取消/重启可达性调查

当前阶段导航 **R12/R13（逐意图）**（历史来源组 **B**；混合归属以意图索引为准）；条目/后继 **ENG-02, ENG-04**。各来源的有界调查/决定仍须先输出结论，不能把族导航视为新增实现或全部Accepted。明确的其他阶段消费者按其任务合同单独交付；其存在不阻塞独立本地工作。

<a id="ud-03-011"></a>
### UD-03-011 — Windows 启停提示、自动任务停止与早期布局

- **来源 PR 与提交**:
  - 主来源: `PR #397`
  - 完整 40 位 Commit SHA (1 个):
    - `ef2319488868b42e4cb3041bf1ded1936f5abb90` (next-2026-08-31.1) — fix: harden Windows user acceptance regressions (#397)
- **核心文件出处**: `src/main/java/featurecat/lizzie/analysis/EngineManager.java`, `src/main/java/featurecat/lizzie/gui/BottomToolbar.java`, `src/main/java/featurecat/lizzie/gui/LizzieFrame.java`, `src/main/java/featurecat/lizzie/gui/Menu.java` 等共 5 文件
- **用户可见行为与边界**: 源含引擎启停 UI、手动自动分析停止准确提示、早期布局回调容忍；log rollover 的测试稳定性不能扩大成已证明全部日志竞态修复。
- **对等项 / 任务映射**: ENG-02 / ENG-05（Accepted）；ANA-06 / ANA-16（Accepted）；REL-09（Missing）；`T03-LIFECYCLE-WINDOWS-APPLICABILITY`。所有历史项仅引用Matrix原范围，最终后继ID由06分配。
- **Tauri 现状 / 证据与残余**: 现有生命周期与目标取消存在；复合 Windows 提示/布局/自动载入任务差异需核对，非笼统架构覆盖。
- **处置结论**: 需有界调查。任务 kind=`investigation`；不把Source差异等同Next缺陷。
- **功能断言 / 调查停止条件**: 逐项比较实际 Run/analysis-task UI 路径；停止于已有有限证据与剩余提示/布局/自动任务场景表；旧事件不改变新Run，停止提示与操作相符。日志 fixture稳定性独立内部排除。
- **实际阻塞与未来证据门**: 原ENG生命周期历史候选完整身份若未定位则不得继承其动态范围；自动载入 T02-AUTOLOAD-QUICK。
- **唯一批次建议 / 责任**: B；`T03-LIFECYCLE-WINDOWS-APPLICABILITY` 拥有此Delta的结果。07/08冻结具体实施合同，06负责同源去重。

<a id="ud-03-018"></a>
### UD-03-018 — 展示与生命周期死锁适用性

- **来源 PR 与提交**:
  - 主来源: `PR #410`
  - 完整 40 位 Commit SHA (1 个):
    - `bfa4759805296b6107e7720810b3195957459c53` (next-2026-09-03.1) — Prevent engine presentation deadlocks (#410)
- **核心文件出处**: `src/main/java/featurecat/lizzie/analysis/EngineManager.java`, `src/main/java/featurecat/lizzie/analysis/Leelaz.java`
- **用户可见行为与边界**: Java展示回调与生命周期 reservation 并发可能相互等待；只移植可达用户目标，不复制Java锁结构。源另含Bash3 CI默认值，内部排除。
- **对等项 / 任务映射**: ENG-02（Accepted）；具名 concurrency investigation；`T03-LIFECYCLE-CONCURRENCY-INVESTIGATION`。所有历史项仅引用Matrix原范围，最终后继ID由06分配。
- **Tauri 现状 / 证据与残余**: Next通过manager/events，但不能据Tokio/WebView免疫断言。
- **处置结论**: 需有界调查。任务 kind=`investigation`；不把Source差异等同Next缺陷。
- **功能断言 / 调查停止条件**: 问：受支持Start/Stop/Restart/analysis状态发布是否在持owner锁时等待UI/回调？检查engine-manager生命周期与App消费者，记录锁/等待/回调边界；停止于无同因路径的证据或命名可达缺口和确定性交错。禁止“长时间压力”充当问题。
- **实际阻塞与未来证据门**: 固定Next路径证据；若查到可达风险由06优先派独立修复，先调查后实施。
- **唯一批次建议 / 责任**: B（当前路径风险优先）；`T03-LIFECYCLE-CONCURRENCY-INVESTIGATION` 拥有此Delta的结果。07/08冻结具体实施合同，06负责同源去重。

<a id="ud-03-045"></a>
### UD-03-045 — target cancel 与明确 Run Stop/退出分开

- **来源 PR 与提交**:
  - 主来源: `PR #469`
  - 完整 40 位 Commit SHA (1 个):
    - `365363d6ac420ef79495823d0fe59b692427ca5a` (next-2026-09-18.2) — fix(engine): finalize stopped transport state
- **核心文件出处**: `src/main/java/featurecat/lizzie/analysis/Leelaz.java`, `src/test/java/featurecat/lizzie/gui/ControlledGtpPeer.java`
- **另有导入来源（不重复计能力）**: `2dff9fd2b7577d4727bf2304938c1cb257a0fd68`。这些源同时保留其内部分组。
- **用户可见行为与边界**: 源退役当前binding时清pondering；健康resident KataGo目标cancel/Pause只清目标、不退出Run，不影响另lane。明确Run Stop/应用退出才teardown owned process；GenericGTP move cancel/failure自身规则保持。
- **对等项 / 任务映射**: ENG-02 / ENG-05 / ANA-06（Accepted，历史区分）；`T03-LIFECYCLE-TARGET-CANCEL-RUN-STOP`。所有历史项仅引用Matrix原范围，最终后继ID由06分配。
- **Tauri 现状 / 证据与残余**: 已有E20目标取消/Ready与Run teardown分别保留；特定Javaponder flag非Next有此字段的证明，残余binding清理需比较。
- **处置结论**: 需有界调查。任务 kind=`investigation`；不把Source差异等同Next缺陷。
- **功能断言 / 调查停止条件**: 按操作分别问/记录：ordinary cancel仅本目标final且healthyRun保留，另lane继续；Run Stop/exit封存binding及ownedprocess被reap；故障按既有状态处理。停止于操作×adapter证据表/具名残余，禁止CPU立即归零保证。
- **实际阻塞与未来证据门**: E20的Windows/KataGo范围；更广adapter绑定清理证据由功能票取得。
- **唯一批次建议 / 责任**: B；`T03-LIFECYCLE-TARGET-CANCEL-RUN-STOP` 拥有此Delta的结果。07/08冻结具体实施合同，06负责同源去重。

<a id="ud-03-046"></a>
### UD-03-046 — 重启权限与迟到 ACK/final 适用性

- **来源 PR 与提交**:
  - 主来源: `PR #470` (包含被集成 PR: #445)
  - 完整 40 位 Commit SHA (2 个):
    - `be242060ae81f5cec9fbad07b564ac5b8bf8cfb2` (next-2026-09-18.2) — Merge pull request #470 from wimi321/fix/restart-confirmation-445
    - `7afe28aa4b7121d5d33742ec157e7db4bc49581c` (next-2026-09-18.2) — Fix restart confirmation authority handoff and delayed acknowledgements
- **核心文件出处**: `src/main/java/featurecat/lizzie/analysis/EngineManager.java`, `src/main/java/featurecat/lizzie/analysis/Leelaz.java`
- **另有导入来源（不重复计能力）**: `9d05032ecfcb1e63e3a5bf62f3718fc53397af51`。这些源同时保留其内部分组。
- **用户可见行为与边界**: 旧退出过程不得复活bootstrap authority；迟到旧ACK只能结清旧lineage，不绑定新engine；保持final fence。
- **对等项 / 任务映射**: ENG-02 / ENG-04（Accepted，原scope）；具名 restart-fence investigation；`T03-LIFECYCLE-RESTART-FENCE-INVESTIGATION`。所有历史项仅引用Matrix原范围，最终后继ID由06分配。
- **Tauri 现状 / 证据与残余**: Next Run identity/stale拒绝已有，但未证明Java相同ACK/final通道。
- **处置结论**: 需有界调查。任务 kind=`investigation`；不把Source差异等同Next缺陷。
- **功能断言 / 调查停止条件**: 问：各adapter restart teardown会否共享reader/ACK并提前授新owner？检查Run/reader/numberedresponse与final boundary；controlled old ACK/final到新Run后不改变新Ready/能力/结果。停止于适用表+有限原证据/具名缺口，不用快速点击代替确定性交错。
- **实际阻塞与未来证据门**: 各adapter实际协议证据；#445最终compatibleengine gate保持未来。
- **唯一批次建议 / 责任**: B（当前restart风险优先）；`T03-LIFECYCLE-RESTART-FENCE-INVESTIGATION` 拥有此Delta的结果。07/08冻结具体实施合同，06负责同源去重。

<a id="ud-04-029"></a>
### UD-04-029 Unicode/延迟启动验收harness身份

- **Source / event**: PR #526 (集成事件 29) | 共 4 个提交；第一父E29。
- **完整source commits（与§6反向对应）**:
  - [7898fe556529170659f30eee5b13c840dbda5e02](https://github.com/wimi321/lizzieyzy-next/commit/7898fe556529170659f30eee5b13c840dbda5e02) — Merge pull request #526 from wimi321/codex/windows-acceptance-startup-20260922
  - [1215d5a7adf6adb13883da38163c38f47cf9d0fb](https://github.com/wimi321/lizzieyzy-next/commit/1215d5a7adf6adb13883da38163c38f47cf9d0fb) — fix(qa): make native acceptance fixtures Unicode-safe
  - [e7391cfa930168a685a9f751fa9af5db8880100c](https://github.com/wimi321/lizzieyzy-next/commit/e7391cfa930168a685a9f751fa9af5db8880100c) — fix(qa): preserve startup failure when launcher already exited
  - [275f31c6e995c8fee2b9df0f7aff9e2a7b283e48](https://github.com/wimi321/lizzieyzy-next/commit/275f31c6e995c8fee2b9df0f7aff9e2a7b283e48) — fix(qa): capture delayed Windows engine startup before freezing process identity
- **发布包含**: 最早有API/ancestry证据收录 `next-2026-09-22.2`（预发布）；全部source也含于正式09-26.1/.2。准备target仅记prepared，不由本字段推断准备版本已发布。
- **实际behavior / sourceDelta**: PR526的1215d5a7/e7391cfa/275f31c6改native acceptance fixtures/scripts，捕获延迟/已退出launcher身份与失败，避免fixture错误丢证据。不是Rust/宽字符API证明所有真实引擎路径安全。
- **Canonical owner / scope**: Java QA内部；APP-01原路径证据不变。
- **候选/覆盖与边界**: 无新增Tauri运行证据；源QA断言不能升级功能Accepted。
- **Disposition**: 仅QA机制排除；无新runtime覆盖。
- **责任意图**: 无独立产品任务；错误展示由T04-DIAGNOSTICS消费。
- **Observable assertions**: 按manifest标QA-only。未来startup诊断的late-output/attempt身份验收见T04-DIAGNOSTICS；APP-01既有非ASCII文件激活证据保留原H-SAVE-DEPARTURE/T01-ACTIVATION，不自动外推engine paths。

<a id="udx-005"></a>
## UDX-005 — AI解说连接、依据与工作区

当前阶段导航 **R16**（历史来源组 **E**；混合归属以意图索引为准）；条目/后继 **AI-01**。各来源的有界调查/决定仍须先输出结论，不能把族导航视为新增实现或全部Accepted。明确的其他阶段消费者按其任务合同单独交付；其存在不阻塞独立本地工作。

<a id="ud-03-012"></a>
### UD-03-012 — AI 棋理解说工作区

- **来源 PR 与提交**:
  - 主来源: `PR #399`
  - 完整 40 位 Commit SHA (1 个):
    - `7542c6ff8e537937851b1c159c84f403f980abec` (next-2026-08-31.2) — ui: redesign AI commentary workspace (#399)
- **核心文件出处**: src/main/java/featurecat/lizzie/teacher/TeacherDialog.java, TeacherDialogStyle.java, TeacherDialogView.java（7542c6ff生产文件）；design-qa.md / docs/qa/ai-commentary-redesign
- **用户可见行为与边界**: 紧凑模式导航、主要阅读区、请求/完成状态、多语言与缩放可读性。
- **对等项 / 任务映射**: 具名 AI commentary workspace successor（无现有Parity Item拥有）；05 #575相关连接后继；`T03-GAME-AI-COMMENTARY-WORKSPACE`。所有历史项仅引用Matrix原范围，最终后继ID由06分配。
- **Tauri 现状 / 证据与残余**: GAME-09 是贡献观察，不是解说；基线未有此解说工作区。
- **处置结论**: 需后继功能/修复。任务 kind=`feature`；不把Source差异等同Next缺陷。
- **功能断言 / 调查停止条件**: 模式可键盘切换；请求/失败/完成状态明确；维护语言文案完整；缩放后内容和操作仍可达；失败不丢个人评论。
- **实际阻塞与未来证据门**: AI解说连接/授权与生成数据分离合同；05 #575后续真实登录/服务门。
- **唯一批次建议 / 责任**: E；`T03-GAME-AI-COMMENTARY-WORKSPACE` 拥有此Delta的结果。07/08冻结具体实施合同，06负责同源去重。

<a id="ud-05-28"></a>
### UD-05-28 

- 来源：[144](https://github.com/wimi321/lizzieyzy-next/commit/c0455839311b73548e479e6ef26f5722af31bdf2)（完整SHA/发布包含见源索引）。
- 用户行为/边界：AI解说支持ChatGPT官方登录与独立API Key配置；PKCE/loopback/JWKS、OS凭据库、workspace/account隔离、刷新/取消、临时故障保留凭据，明确授权失败才失效；不私用其他应用客户端。
- Tauri映射/具名任务意图：新增AI解说连接/认证后继；T05-AI-CONNECTION（E）
- 覆盖证据或缺口：PR575现已squash合并c0455839；保留Java实际自然过期刷新证据的候选/平台边界。Tauri尚无此功能，LinuxSecretService/远端撤销等仍缺现场。
- 处置：后继/支持方式决策；官方接入/凭据前置必须可用，不降格私有代理或自动APIKey兜底。
- 后续验收/调查停止条件：授权登录/刷新/登出/取消/账号切换、迟到回调与secret storage失败；无凭据不上线，不自动产生付费重试；真实撤销及支持平台各验。

<a id="ud-05-29"></a>
### UD-05-29 

- 来源：[144](https://github.com/wimi321/lizzieyzy-next/commit/c0455839311b73548e479e6ef26f5722af31bdf2)（完整SHA/发布包含见源索引）。
- 用户行为/边界：AI解说依据冻结局面/规则/PV/提子气及真实分析事实生成，top3外实战手和未知损失诚实表达；用户明确发起单步/范围/全局，后续追问继续绑定原节点与依据。
- Tauri映射/具名任务意图：新增AI解说依据与会话后继；T05-AI-GROUNDED-TEACHING（E）
- 覆盖证据或缺口：PR575 docs/CHATGPT_COMMENTARY.md与教学报告；响应完成/取消/失败区分，不把未完流写成完成。Tauri ADR0003禁止自动写生成统计/解说进个人C。
- 处置：后继；Java自动SGF写入若冲突须适配为分离生成信息，不宣称批准过等价替换。
- 后续验收/调查停止条件：切谱/浏览/账号变化与流式取消不污染新节点；无分析事实不编数字，费用/可能追加校正调用需明确同意；个人C原样保留。

<a id="ud-05-30"></a>
### UD-05-30 

- 来源：[144](https://github.com/wimi321/lizzieyzy-next/commit/c0455839311b73548e479e6ef26f5722af31bdf2)（完整SHA/发布包含见源索引）。
- 用户行为/边界：连接与解说偏好分开，模型/思考档位来自实际账户支持目录，未知不猜；键盘卡片、字体语言回退，取消不保存或触发查询/计费。
- Tauri映射/具名任务意图：AI解说设置及I18N-01；T05-AI-PREFERENCES（E）
- 覆盖证据或缺口：PR575最终settings与输入焦点修复；systemDPI/Linux/撤销余缺不能从Java源码提升为现场通过。
- 处置：后继，第一批新增入口就外置字符串与旧字段映射；最终完整翻译不阻塞所有功能。
- 后续验收/调查停止条件：键盘/屏幕阅读/多语言长标签、保存/取消、手动模型不被目录覆盖；scope变化不自动发请求。

<a id="udx-006"></a>
## UDX-006 — 目差显示与未分析节点图导航

当前阶段导航 **R11**（历史来源组 **A**；混合归属以意图索引为准）；条目/后继 **REVIEW-10**。各来源的有界调查/决定仍须先输出结论，不能把族导航视为新增实现或全部Accepted。明确的其他阶段消费者按其任务合同单独交付；其存在不阻塞独立本地工作。

<a id="ud-03-013"></a>
### UD-03-013 — 目差领导方/近零格式、基线与标签可读性

- **来源 PR 与提交**:
  - 主来源: `PR #401` (包含被集成 PR: #387)
  - 完整 40 位 Commit SHA (1 个):
    - `e6ad02af0454413512bce6f2102f0c66e8424144` (next-2026-09-01.1) — Merge pull request #401 from qiyi71w/feat/issue-387-winrate-graph-readability
- **核心文件出处**: `src/main/java/featurecat/lizzie/Config.java`, `src/main/java/featurecat/lizzie/analysis/MoveData.java`, `src/main/java/featurecat/lizzie/gui/ConfigDialog2.java`, `src/main/java/featurecat/lizzie/gui/WinrateGraph.java`
- **用户可见行为与边界**: 除曲线对比度外，保留 leader-prefix、near-zero、维护 locales、突出且有语义的零基线/mark，以及 current/hover/endpoint 标签不重叠。
- **对等项 / 任务映射**: ANA-11（Accepted，原图编码范围）；ADR0001；具名 readability successor；`T03-ANA11-SCORE-LEAD-DISPLAY`。所有历史项仅引用Matrix原范围，最终后继ID由06分配。
- **Tauri 现状 / 证据与残余**: 基线 WinrateChart 的三条同样虚线与 signed toFixed(1) hover 不等于上述新增格式/突出基线。原图能力保留，不宣称新目标已Covered。
- **额外来源 / 继承条件**: e6ad02af 的 WinrateGraphScoreLeadDisplayTest.java:19–119；基线 apps/desktop/src/components/WinrateChart.tsx。
- **处置结论**: 需后继功能/修复。任务 kind=`feature`；不把Source差异等同Next缺陷。
- **功能断言 / 调查停止条件**: 领导方与近零文本按来源语义区分；零基线可辨且标记清晰；当前/hover/端点同时显示不遮挡；各维护语言可读；整条单序列视角仍遵守ADR0001。
- **实际阻塞与未来证据门**: ANA-11原编码；新文字/标记的本地化与视觉验收；没有逐项批准删除这些目标。
- **唯一批次建议 / 责任**: A；`T03-ANA11-SCORE-LEAD-DISPLAY` 拥有此Delta的结果。07/08冻结具体实施合同，06负责同源去重。

<a id="ud-03-021"></a>
### UD-03-021 — 未分析节点图点击导航与额外手势差异

- **来源 PR 与提交**:
  - 主来源: `PR #413`
  - 完整 40 位 Commit SHA (1 个):
    - `69843b834a5a0e2b087f332073f40feab7dfd5d1` (next-2026-09-03.1) — fix(gui): restore graph navigation for unanalyzed moves (#413)
- **核心文件出处**: `src/main/java/featurecat/lizzie/gui/WinrateGraph.java`
- **用户可见行为与边界**: 图导航目标不依赖有分析值；保留选中分支/节点身份；Java源拖动/点击与Next已有点击分别记录。
- **对等项 / 任务映射**: ANA-11（Accepted）；具名 graph navigation evidence investigation；`T03-ANA11-UNANALYZED-SCRUBBING`。所有历史项仅引用Matrix原范围，最终后继ID由06分配。
- **Tauri 现状 / 证据与残余**: WinrateChart 基线点击按节点序列计算，未分析节点可达，不能将其全部判Missing；未定位原native完整候选/拖动等价证据。
- **处置结论**: 需有界调查。任务 kind=`investigation`；不把Source差异等同Next缺陷。
- **功能断言 / 调查停止条件**: 只重建原ANA-11候选/证据tuple并检查未分析节点点击分支identity；停止于可继承窄点击证据+拖动差异表，缺原记录则保持未来原生门，不发明新native通过。
- **实际阻塞与未来证据门**: 历史图native记录的完整候选/具体条件；新增拖动行为如未支持由06归入具名后继。
- **唯一批次建议 / 责任**: A；`T03-ANA11-UNANALYZED-SCRUBBING` 拥有此Delta的结果。07/08冻结具体实施合同，06负责同源去重。

<a id="udx-007"></a>
## UDX-007 — 换谱与后台响应性调查

当前阶段导航 **R11**（历史来源组 **A**；混合归属以意图索引为准）；条目/后继 **SGF-07, UI-02**。各来源的有界调查/决定仍须先输出结论，不能把族导航视为新增实现或全部Accepted。明确的其他阶段消费者按其任务合同单独交付；其存在不阻塞独立本地工作。

<a id="ud-03-014"></a>
### UD-03-014 — 换谱事务、最新请求与自动快析交接

- **来源 PR 与提交**:
  - 主来源: `PR #402`
  - 完整 40 位 Commit SHA (1 个):
    - `81152eeaade28e114aeb414114e1684c829d9595` (next-2026-09-01.1) — fix: stabilize kifu switching and automatic analysis (#402)
- **核心文件出处**: `src/main/java/featurecat/lizzie/Lizzie.java`, `src/main/java/featurecat/lizzie/analysis/AnalysisEngine.java`, `src/main/java/featurecat/lizzie/gui/KifuEngineSyncCoordinator.java`, `src/main/java/featurecat/lizzie/rules/Board.java`
- **用户可见行为与边界**: latest-request-wins；坏文件/取消保留当前棋谱；延迟载入须等旧自动任务恢复结束，不让旧结果写新棋谱。
- **对等项 / 任务映射**: SGF-07（Accepted）；ANA-16（Accepted，显式任务）；T02-AUTOLOAD-QUICK；`T03-SGF07-RAPID-SWITCH-TRANSACTION`。所有历史项仅引用Matrix原范围，最终后继ID由06分配。
- **Tauri 现状 / 证据与残余**: 原子替换已有；自动快析未被ANA-16吸收，高频时序需有界比较。
- **处置结论**: 需有界调查。任务 kind=`investigation`；不把Source差异等同Next缺陷。
- **功能断言 / 调查停止条件**: 问：替换请求、generation和旧任务cleanup能否一致封存？追踪SGF-07 owner+App自动/有限handoff；对一个被覆盖请求、坏候选及最终有效请求保留可控迟到结果。停止于不适用/已有窄证据/具名缺口；不固定50ms或五文件。
- **实际阻塞与未来证据门**: SGF-07有效原范围与T02-AUTOLOAD-QUICK；调查结论前不授权新修复。
- **唯一批次建议 / 责任**: A/B（数据安全A，快析B）；`T03-SGF07-RAPID-SWITCH-TRANSACTION` 拥有此Delta的结果。07/08冻结具体实施合同，06负责同源去重。

<a id="ud-03-023"></a>
### UD-03-023 — Windows 分析/在线同步/设置响应性适用性

- **来源 PR 与提交**:
  - 主来源: `PR #416`
  - 完整 40 位 Commit SHA (1 个):
    - `e883d575d7534717dba30fe6b3b1fc3eaaf260d1` (next-2026-09-03.1) — 修复 Windows 棋谱分析、弈客同步与一键设置卡顿 (#416)
- **核心文件出处**: `src/main/java/featurecat/lizzie/analysis/AnalysisEngine.java`, `src/main/java/featurecat/lizzie/gui/KataGoAutoSetupDialog.java`, `src/main/java/featurecat/lizzie/gui/LizzieFrame.java`
- **用户可见行为与边界**: 大分析结果、Yike同步与自动设置交互应保持可响应；Swing EDT 调度技术排除但不能删除用户目标。
- **对等项 / 任务映射**: SGF-01（Accepted）；PROV-01（Partial，导入）；PROV-03（Partial，在线同步）；具名 performance investigation；`T03-PERF-BACKGROUND-OFFLOAD`。所有历史项仅引用Matrix原范围，最终后继ID由06分配。
- **Tauri 现状 / 证据与残余**: 没有Next复合Windows场景证据；不以Rust/WebView或任意500手保证响应性。
- **处置结论**: 需有界调查。任务 kind=`investigation`；不把Source差异等同Next缺陷。
- **功能断言 / 调查停止条件**: 问：受支持分析publication、PROV-03同步、将来的设置是否同步阻塞UI或无限排队？按各现存owner/事件面记录bounded工负载和取消/进度结果；停止于每路径无同因/已证据/残余清单。未实现设置留其功能票。
- **实际阻塞与未来证据门**: PROV-03真实同步证据与设置准入；原生Windows性能由对应消费者取得。
- **唯一批次建议 / 责任**: A/B/C 按可达消费者；`T03-PERF-BACKGROUND-OFFLOAD` 拥有此Delta的结果。07/08冻结具体实施合同，06负责同源去重。

<a id="udx-008"></a>
## UDX-008 — 独立benchmark与性能反馈

当前阶段导航 **R12/R13（逐意图）**（历史来源组 **B**；混合归属以意图索引为准）；条目/后继 **ENG-14**。各来源的有界调查/决定仍须先输出结论，不能把族导航视为新增实现或全部Accepted。明确的其他阶段消费者按其任务合同单独交付；其存在不阻塞独立本地工作。

<a id="ud-03-015"></a>
### UD-03-015 — benchmark NN 与搜索速度双指标

- **来源 PR 与提交**:
  - 主来源: `PR #403`
  - 完整 40 位 Commit SHA (1 个):
    - `b5050a2937292de073b1de660aef57ec7c04058f` (next-2026-09-01.1) — feat: show NN and search speed in performance optimization (#403)
- **核心文件出处**: `src/main/java/featurecat/lizzie/gui/KataGoAutoSetupDialog.java`, `src/main/java/featurecat/lizzie/util/KataGoRuntimeHelper.java`
- **用户可见行为与边界**: NN eval/s 与 visits/playouts/s 分开，附当前后端/模型/线程目标；不能把指标互换。
- **对等项 / 任务映射**: 具名 benchmark/performance successor；ENG-01仅profile基础；`T03-PERFORMANCE-SPEED-METRICS`。所有历史项仅引用Matrix原范围，最终后继ID由06分配。
- **Tauri 现状 / 证据与残余**: ENG-08（Deferred）仅引擎目录排序；并非性能调优 Partial。
- **处置结论**: 需后继功能/修复。任务 kind=`feature`；不把Source差异等同Next缺陷。
- **功能断言 / 调查停止条件**: 同一受支持benchmark的两指标有明确标签/单位与目标身份，缺项显示unknown而非伪造另一指标；成功/失败有终态。
- **实际阻塞与未来证据门**: 028独立benchmark执行及036目标所有权；真实兼容引擎证据。
- **唯一批次建议 / 责任**: B；`T03-PERFORMANCE-SPEED-METRICS` 拥有此Delta的结果。07/08冻结具体实施合同，06负责同源去重。

<a id="ud-03-017"></a>
### UD-03-017 — benchmark 结果与线程推荐无障碍反馈

- **来源 PR 与提交**:
  - 主来源: `PR #409`
  - 完整 40 位 Commit SHA (1 个):
    - `55384424f797bbf7d39f2215a173b106ef175b00` (next-2026-09-03.1) — fix: make benchmark feedback accurate and accessible (#409)
- **核心文件出处**: `src/main/java/featurecat/lizzie/gui/AccessibilitySupport.java`, `src/main/java/featurecat/lizzie/gui/KataGoAutoSetupDialog.java`
- **用户可见行为与边界**: 结果、推荐线程、硬件类型和失败状态可被读屏/键盘理解；不要求复制Swing AccessibleDescription或指定HTML标签形状。
- **对等项 / 任务映射**: 具名 benchmark successor；UI-02（Partial）仅相关交互基础；`T03-PERFORMANCE-A11Y-BENCHMARK`。所有历史项仅引用Matrix原范围，最终后继ID由06分配。
- **Tauri 现状 / 证据与残余**: 无benchmark结果动态验收，Web框架不证明无障碍等价。
- **处置结论**: 需后继功能/修复。任务 kind=`feature`；不把Source差异等同Next缺陷。
- **功能断言 / 调查停止条件**: 完成/失败反馈准确且可读屏，推荐与实际值区别明确；焦点不丢失；无需视觉颜色才能判断结果。
- **实际阻塞与未来证据门**: 015/028/036的语义结果合同。
- **唯一批次建议 / 责任**: B（F完整语言/无障碍收尾）；`T03-PERFORMANCE-A11Y-BENCHMARK` 拥有此Delta的结果。07/08冻结具体实施合同，06负责同源去重。

<a id="ud-03-028"></a>
### UD-03-028 — 独立 slot-owned custom benchmark

- **来源 PR 与提交**:
  - 主来源: `PR #434`
  - 完整 40 位 Commit SHA (1 个):
    - `554fdebd4f4bb3c2aaf2a22cd1e8ef1650d02883` (next-2026-09-13.2) — fix(engine): support custom KataGo benchmark tasks (#434)
- **核心文件出处**: `src/main/java/featurecat/lizzie/analysis/BenchmarkExecution.java`, `src/main/java/featurecat/lizzie/analysis/EngineManager.java`
- **用户可见行为与边界**: 直接本地KataGo benchmark保留自定义参数、独立可取消slot、流输出/退出码与compact running/failed状态；GTP capability/同步与benchmark分离。
- **对等项 / 任务映射**: 具名 benchmark runner successor；ENG-02仅Run基础；`T03-PERFORMANCE-CUSTOM-BENCHMARK-RUNNER`。所有历史项仅引用Matrix原范围，最终后继ID由06分配。
- **Tauri 现状 / 证据与残余**: ENG-08是排序，与runner不同；没有独立benchmark验收。
- **处置结论**: 需后继功能/修复。任务 kind=`feature`；不把Source差异等同Next缺陷。
- **功能断言 / 调查停止条件**: 选定持久目标/完整argv/workdir执行；状态和退出码正确，取消只清本任务slot；失败可见，前台GTP不被混写；不隐式保存推荐或修改用户配置。
- **实际阻塞与未来证据门**: 消费既有ENG-01的稳定profile ID、持久command/config layers/workdir及T01-RESOURCE所需目标identity；明确本地benchmark capability与真实兼容二进制。runner可先独立交付，不依赖036推荐来源/保存策略实现；它返回绑定目标ID与冻结输入revision的结果，不写policy。036之后消费runner结果完成推荐保存/启动采用集成；此单向边不改变用户配置或真实引擎证据门。
- **唯一批次建议 / 责任**: B；`T03-PERFORMANCE-CUSTOM-BENCHMARK-RUNNER` 拥有此Delta的结果。07/08冻结具体实施合同，06负责同源去重。

<a id="udx-009"></a>
## UDX-009 — 远程算力的本地隔离与配置体验

当前阶段导航 **R14**（历史来源组 **C**；混合归属以意图索引为准）；条目/后继 **RCOMP-01**。各来源的有界调查/决定仍须先输出结论，不能把族导航视为新增实现或全部Accepted。明确的其他阶段消费者按其任务合同单独交付；其存在不阻塞独立本地工作。

<a id="ud-03-016"></a>
### UD-03-016 — 远端线程所有权与本地 benchmark 排除

- **来源 PR 与提交**:
  - 主来源: `PR #408`
  - 完整 40 位 Commit SHA (1 个):
    - `b9b3fc88c5c3bdfd399214410c5053486f1b099b` (next-2026-09-03.1) — fix: skip automatic benchmark for remote engines (#408)
- **核心文件出处**: `src/main/java/featurecat/lizzie/util/KataGoRuntimeHelper.java`, `src/test/java/featurecat/lizzie/util/KataGoRuntimeHelperTest.java`
- **用户可见行为与边界**: Remote Compute / Java SSH /外部SSH远端引擎不触发本地硬件benchmark；可见说明远端管理。
- **对等项 / 任务映射**: SSH-01 / RCOMP-01（Deferred）；T01-SSH / T01-REMOTE；`T03-RCOMP-SKIP-LOCAL-BENCHMARK`。所有历史项仅引用Matrix原范围，最终后继ID由06分配。
- **Tauri 现状 / 证据与残余**: 是远端后继的不变量，不覆盖036本地policy。
- **处置结论**: 需后继功能/修复。任务 kind=`feature`；不把Source差异等同Next缺陷。
- **功能断言 / 调查停止条件**: 远端选择不运行本地benchmark/本地线程注入；不支持目标有说明；本地独立入口不受误伤。
- **实际阻塞与未来证据门**: T01-SSH/REMOTE准入adapter与服务；统一网络/凭据合同。
- **唯一批次建议 / 责任**: C；`T03-RCOMP-SKIP-LOCAL-BENCHMARK` 拥有此Delta的结果。07/08冻结具体实施合同，06负责同源去重。

<a id="ud-03-033"></a>
### UD-03-033 — Custom compute 配置指引与受限环境帮助

- **来源 PR 与提交**:
  - 主来源: `PR #427`
  - 完整 40 位 Commit SHA (3 个):
    - `92dc357e41cd63ced1c69603139507494822d8fe` (next-2026-09-13.2) — Merge pull request #427 from wimi321/feat/remote-compute-one-click-help
    - `4477b7c23bf37bd43077e84d9f0d9c2125b3bc49` (next-2026-09-13.2) — Polish constrained custom compute guidance
    - `8e0d33a67c9dd7f94c02ae7df7aed9136fef6ab1` (next-2026-09-13.2) — Add one-click setup guidance for custom compute
- **核心文件出处**: `src/main/java/featurecat/lizzie/gui/RemoteComputeDialog.java`, `README.md`
- **用户可见行为与边界**: 一键配置入口/脚本指导及多语言帮助，受限compute环境有清楚限制；帮助入口不等于远程功能或服务验收。
- **对等项 / 任务映射**: RCOMP-01（Deferred）；T01-REMOTE；`T03-RCOMP01-SETUP-GUIDANCE`。所有历史项仅引用Matrix原范围，最终后继ID由06分配。
- **Tauri 现状 / 证据与残余**: 未来远程配置UI/真实服务待实现。
- **处置结论**: 需后继功能/修复。任务 kind=`feature`；不把Source差异等同Next缺陷。
- **功能断言 / 调查停止条件**: 准入协议下用户可由指南配置节点并见连接/不支持/失败说明；限制与网络策略不支持可见拒绝；不静默Direct或取得浏览器秘密。
- **实际阻塞与未来证据门**: T01-REMOTE正式协议/授权调查结果；真实远端/凭据/断连取消证据。
- **唯一批次建议 / 责任**: C；`T03-RCOMP01-SETUP-GUIDANCE` 拥有此Delta的结果。07/08冻结具体实施合同，06负责同源去重。

<a id="ud-05-02"></a>
### UD-05-02 

- 来源：[004](https://github.com/wimi321/lizzieyzy-next/commit/ee774cac8df2150c5349492059f83a99327d9ab2), [008](https://github.com/wimi321/lizzieyzy-next/commit/a0e7b211d253ec09fa1bfe662f7a0a7c544c1238)（完整SHA/发布包含见源索引）。
- 用户行为/边界：远程模型列表刷新图标及可访问名称；刷新保留当前选择，失败可见。
- Tauri映射/具名任务意图：RCOMP-01；T05-REMOTE-MODEL-REFRESH（C）
- 覆盖证据或缺口：Java PR550；Tauri RCOMP-01仍Deferred，不能以本地模型选择替代远程查询。
- 处置：后继功能，与远程端点/凭据/取消契约一起交付。
- 后续验收/调查停止条件：真实支持端点刷新、空列表/过期结果/断线与键盘操作；无凭据则Blocked。

<a id="udx-010"></a>
## UDX-010 — 连续预算历史范围继承

当前阶段导航 **R12/R13（逐意图）**（历史来源组 **B**；混合归属以意图索引为准）；条目/后继 **ANA-06**。各来源的有界调查/决定仍须先输出结论，不能把族导航视为新增实现或全部Accepted。明确的其他阶段消费者按其任务合同单独交付；其存在不阻塞独立本地工作。

<a id="ud-03-020"></a>
### UD-03-020 — 已支持 continuous budget 偏好与 Limit/Resume

- **来源 PR 与提交**:
  - 主来源: `PR #412` (包含被集成 PR: #407)
  - 完整 40 位 Commit SHA (1 个):
    - `9a1fd30c0e4d45efccb5d44ac178e6d1fc14c8e2` (next-2026-09-03.1) — fix(gui): restore live analysis limits on Engine settings (#412)
- **核心文件出处**: `src/main/java/featurecat/lizzie/gui/ConfigDialog2.java`
- **用户可见行为与边界**: 普通连续分析时间/visits enable flags、值校验、Apply、持久化及达到预算后的Limit/Resume，区别于有限AnalysisStageConditions。
- **对等项 / 任务映射**: ANA-06 / PREF-01（Accepted）；`T03-ANA06-LIVE-LIMITS-INHERITANCE`。所有历史项仅引用Matrix原范围，最终后继ID由06分配。
- **Tauri 现状 / 证据与残余**: ContinuousBudgetEditor、AppChrome入口和App消费已存在；不再拆新接线功能。
- **额外来源 / 继承条件**: E20见§4.1；原候选 a06d600b0f16bd6bb61415a2907c5d839e53a0b1。
- **处置结论**: 已有证据覆盖（限定下述原范围）。任务 kind=`inheritance`；不把Source差异等同Next缺陷。
- **功能断言 / 调查停止条件**: 引用E20：保存开关/合法值并限额停止、Resume、重开按原范围；保留默认600秒soak与macOS/Linux未运行，不扩大历史Windows范围。
- **实际阻塞与未来证据门**: 无新增实现阻塞；环境扩大时由关联任务重验。
- **唯一批次建议 / 责任**: B（仅继承，无新feature）；`T03-ANA06-LIVE-LIMITS-INHERITANCE` 拥有此Delta的结果。07/08冻结具体实施合同，06负责同源去重。

<a id="udx-011"></a>
## UDX-011 — 自动快析、交还与本地Restart调查

当前阶段导航 **R12/R13（逐意图）**（历史来源组 **B**；混合归属以意图索引为准）；条目/后继 **ANA-19**。各来源的有界调查/决定仍须先输出结论，不能把族导航视为新增实现或全部Accepted。明确的其他阶段消费者按其任务合同单独交付；其存在不阻塞独立本地工作。

<a id="ud-03-022"></a>
### UD-03-022 — Pause、当前任务取消、Resume 与新载入边界

- **来源 PR 与提交**:
  - 主来源: `PR #415`
  - 完整 40 位 Commit SHA (1 个):
    - `d7edb3296ca688ecf716aa68a19f6475929cf541` (next-2026-09-03.1) — fix(gui): keep manual analysis pause authoritative (#415)
- **核心文件出处**: `src/main/java/featurecat/lizzie/analysis/AnalysisEngine.java`, `src/main/java/featurecat/lizzie/analysis/Leelaz.java`, `src/main/java/featurecat/lizzie/gui/BottomToolbar.java`
- **用户可见行为与边界**: Pause取消当前自动棋谱任务；专用worker结束、共享借用恢复且不退出进程；Resume仅当前局面；较晚的新kifu load可开始新自动任务。旧restore不得绕过暂停。
- **对等项 / 任务映射**: ANA-06 / ANA-16（Accepted，原范围）；T02-AUTOLOAD-QUICK；`T03-AUTOLOAD-PAUSE-BOUNDARIES`。所有历史项仅引用Matrix原范围，最终后继ID由06分配。
- **Tauri 现状 / 证据与残余**: 已有连续暂停/显式有限任务不等于自动载入快析；保留source新载入边界，不新增永久暂停策略。
- **处置结论**: 需后继功能/修复。任务 kind=`feature`；不把Source差异等同Next缺陷。
- **功能断言 / 调查停止条件**: 当前自动任务取消后其迟到完成不启动ponder；Resume只分析当前局面；下一次独立有效载入按自动快析设置准入，等待旧cleanup；不退出健康共享Run、不影响独立lane。
- **实际阻塞与未来证据门**: T02-AUTOLOAD-QUICK的准入/交接合同；源d7edb329 patch:11–19,30–32。
- **唯一批次建议 / 责任**: B；`T03-AUTOLOAD-PAUSE-BOUNDARIES` 拥有此Delta的结果。07/08冻结具体实施合同，06负责同源去重。

<a id="ud-03-024"></a>
### UD-03-024 — 已有整盘预算与快析转普通分析竞态

- **来源 PR 与提交**:
  - 主来源: `PR #418`
  - 完整 40 位 Commit SHA (1 个):
    - `f64c2767cbe80f6aa92d041eae408003b68e5186` (next-2026-09-04.1) — 优化自动分析切换与整盘精析搜索设置 (#418)
- **核心文件出处**: `src/main/java/featurecat/lizzie/Config.java`, `src/main/java/featurecat/lizzie/analysis/WholeGameAnalysisOptions.java`, `src/main/java/featurecat/lizzie/analysis/WholeGameAnalysisPlan.java`, `src/main/java/featurecat/lizzie/gui/LizzieFrame.java`
- **用户可见行为与边界**: 整盘自定义visits/playouts和快析到普通分析有序交接是两目标。ANA-16显式任务条件已支持，自动载入交接不被覆盖。
- **对等项 / 任务映射**: ANA-16（Accepted）；T02-AUTOLOAD-QUICK；`T03-ANA16-WHOLE-GAME-LIMITS`。所有历史项仅引用Matrix原范围，最终后继ID由06分配。
- **Tauri 现状 / 证据与残余**: 已有total/leading/time OR预算引用E16；过渡identity/owner仍需有界调查。
- **额外来源 / 继承条件**: E16原候选5e593537f702af0c651a2c3bf0f0fd77758052e2，Windows/KataGo1.16.4EigenCPU。
- **处置结论**: 需有界调查。任务 kind=`investigation`；不把Source差异等同Next缺陷。
- **功能断言 / 调查停止条件**: 问：有限/自动完成后是否只将当前有效、未暂停目标交给普通owner？列出generation/lease/结果发布顺序；停止于已有E16有限范围和自动残余。预算按用户合法值，无任意2000visits冻结要求。
- **实际阻塞与未来证据门**: T02-AUTOLOAD-QUICK；E16不证明其自动触发。
- **唯一批次建议 / 责任**: B；`T03-ANA16-WHOLE-GAME-LIMITS` 拥有此Delta的结果。07/08冻结具体实施合同，06负责同源去重。

<a id="ud-03-041"></a>
### UD-03-041 — 自动快析恢复、命令完整分帧与失败呈现适用性

- **来源 PR 与提交**:
  - 主来源: `PR #460`
  - 完整 40 位 Commit SHA (4 个):
    - `88715db53c546d750ebba65370da0db54319f686` (next-2026-09-18.2) — Merge pull request #460 from qiyi71w/fix/automatic-sgf-quick-analysis
    - `2f4b216aa5d876c08443e0c4a0d5f3a01bb983f6` (next-2026-09-18.2) — fix(engine): avoid failure presentation deadlock
    - `c6eb5b8bfb268f6b2827fdccdba019f39ba2fe72` (next-2026-09-18.2) — fix(analysis): resume after automatic quick scan
    - `0b858f9562ba246f448a11d30ede7786a0c7b9b8` (next-2026-09-18.2) — fix(gtp): preserve command-list framing
- **核心文件出处**: `src/main/java/featurecat/lizzie/analysis/Leelaz.java`, `src/main/java/featurecat/lizzie/gui/LizzieFrame.java`
- **用户可见行为与边界**: 自动快析成功后仅对当前未暂停/idle目标恢复普通分析，不重复盘面 replay；list_commands payload直到空行完整响应消费；失败呈现不死锁。
- **对等项 / 任务映射**: T02-AUTOLOAD-QUICK；ANA-16仅显式任务；ENG-02 / ENG-10现有基础；`T03-AUTOLOAD-RESUME-FRAMING-INVESTIGATION`。所有历史项仅引用Matrix原范围，最终后继ID由06分配。
- **Tauri 现状 / 证据与残余**: Next普通analysis与GTP handshake不同；需查各实际parser/handoff，不自动认作Java缺陷。
- **处置结论**: 需有界调查。任务 kind=`investigation`；不把Source差异等同Next缺陷。
- **功能断言 / 调查停止条件**: 问：实际handshake是否误把payload下一行当新响应，交接是否让旧task覆盖pause/newtarget？检查各adapter framing与owner；确定性split/latefinal/failurepresentation结果不串包、不复活旧分析；停止于不适用/已有窄证据/具名修复。自动快析新增行为等T02准入后实施。
- **实际阻塞与未来证据门**: 018/032问题结果、T02-AUTOLOAD-QUICK；真实兼容engine future门。
- **唯一批次建议 / 责任**: B（当前parser/handoff风险优先）；`T03-AUTOLOAD-RESUME-FRAMING-INVESTIGATION` 拥有此Delta的结果。07/08冻结具体实施合同，06负责同源去重。

<a id="ud-03-051"></a>
### UD-03-051 — 整盘执行模式及所有权说明

- **来源 PR 与提交**:
  - 主来源: `PR #478`
  - 完整 40 位 Commit SHA (2 个):
    - `4c11cf264d855cd576c12183ccd97173a9372b29` (next-2026-09-18.2) — Merge pull request #478 from wimi321/fix/whole-game-execution-mode
    - `f178b22f8d98b8b41e708fc363cc14ab2c98a25b` (next-2026-09-18.2) — fix: distinguish local shared analysis from remote execution
- **核心文件出处**: `src/main/java/featurecat/lizzie/analysis/AnalysisEngine.java`, `src/main/java/featurecat/lizzie/gui/WholeGameAnalysisDialog.java`
- **另有导入来源（不重复计能力）**: `e03490000b596e60639da1f61be120b4563cc7d8`。这些源同时保留其内部分组。
- **用户可见行为与边界**: 区分本地共享前台和独立remote执行，显示并发/性能预期；标签不授予未验证remote能力。
- **对等项 / 任务映射**: ANA-16（Accepted，显式tasks）；RCOMP-01（Deferred）；`T03-ANA16-EXECUTION-MODE-INDICATOR`。所有历史项仅引用Matrix原范围，最终后继ID由06分配。
- **Tauri 现状 / 证据与残余**: 本地任务既有，remote/service门保持未来。
- **处置结论**: 需后继功能/修复。任务 kind=`feature`；不把Source差异等同Next缺陷。
- **功能断言 / 调查停止条件**: 显示实际task执行目标/模式，不以用户选项冒充成功准入；共享lease/独立remote限制说明一致，remote不可用可见拒绝。
- **实际阻塞与未来证据门**: T01-REMOTE准入/服务证据只阻塞remote部分，非所有本地显示。
- **唯一批次建议 / 责任**: B（本地）；C（remote）；`T03-ANA16-EXECUTION-MODE-INDICATOR` 拥有此Delta的结果。07/08冻结具体实施合同，06负责同源去重。

<a id="ud-04-004"></a>
### UD-04-004 整局分析终态交还与窗口close/reopen时序

- **Source / event**: PR #495 (集成事件 4) | 共 3 个提交；第一父E04。
- **完整source commits（与§6反向对应）**:
  - [c8eea60bbdc9cd9dcd69a1b081d9c8f51cb9575d](https://github.com/wimi321/lizzieyzy-next/commit/c8eea60bbdc9cd9dcd69a1b081d9c8f51cb9575d) — Merge pull request #495 from wimi321/fix/whole-game-terminal-handoff
  - [6ea79ccc2c251389f5a5edb63f5b89ddb026642c](https://github.com/wimi321/lizzieyzy-next/commit/6ea79ccc2c251389f5a5edb63f5b89ddb026642c) — Preserve foreground intent across transient snapshot restoration
  - [6e2e386a7976826d927124b88cfe2aefbd549741](https://github.com/wimi321/lizzieyzy-next/commit/6e2e386a7976826d927124b88cfe2aefbd549741) — Preserve whole-game analysis ownership through foreground handoff
- **发布包含**: 最早有API/ancestry证据收录 `next-2026-09-18.2`（预发布）；全部source也含于正式09-26.1/.2。准备target仅记prepared，不由本字段推断准备版本已发布。
- **实际behavior / sourceDelta**: PR495修复完成/停止后立即关闭或重开whole-game窗口的普通分析交还竞争与重复触发；不是只存在DTO或任务启动入口就算交还已验。
- **Canonical owner / scope**: ANA-06/ANA-16（均Accepted）。
- **候选/覆盖与边界**: H-ANALYSIS保留真实Windows/KataGo范围；Java专属窗口销毁机制不复制。立即close/reopen差异尚未被这些原场景全量证明。
- **Disposition**: 窄历史覆盖；窗口时序差异有界调查。
- **责任意图**: T04-HANDOFF。
- **Observable assertions**: 原有限任务交还、lane独立取消和暂停意图按H-ANALYSIS继承；对Next实际对应的关闭/重开入口比较完成/Cancel前后，交还只一次、保持显式暂停、旧task不能重绑新Run/任务。若无额外可达窗口时序，给出源和入口对比闭合，不造假故障。

<a id="ud-05-17"></a>
### UD-05-17 

- 来源：[064](https://github.com/wimi321/lizzieyzy-next/commit/aaef6f12f4c440b70bf1cf141c55d5065640675e), [071](https://github.com/wimi321/lizzieyzy-next/commit/65cc93842eb26e0dd0ee911f3d16493b52cd6399), [075](https://github.com/wimi321/lizzieyzy-next/commit/17fe30c1c049f35bf5eaeeab4f70a57221139ddc), [077](https://github.com/wimi321/lizzieyzy-next/commit/0fe5b3769e5f1402013df5520643466ff021f5e6)（完整SHA/发布包含见源索引）。
- 用户行为/边界：自动快析完成/引擎异步启动或预加载期间保持用户当前浏览位置；只对当前game/run/rules恢复任务及前台暂停状态。
- Tauri映射/具名任务意图：ANA-16（分析任务）、ANA-06（持续分析）、ENG-02（Run）为受影响边界，ENG-06（自动载入）为相关前置；自动打开谱快析新增责任 T05-AUTO-QUICK-OWNER（B），不扩大既有手动快析或 ANA-11 胜率图范围。
- 覆盖证据或缺口：PR572/580直接修复17fe30c1；Tauri手动one-visit quick task App.tsx:3101存在，不代表打开谱自动快析。
- 处置：后继功能，串行冻结打开/切换/预加载/交还生命周期；不同文件不代表可并行实现。
- 后续验收/调查停止条件：打开谱后引擎迟到启动、手动浏览、换谱/切引擎/取消/关闭均不拉回旧节点、不写旧局面；真实引擎完整曲线与交还状态。

<a id="ud-05-25"></a>
### UD-05-25 

- 来源：[106](https://github.com/wimi321/lizzieyzy-next/commit/0178f55f51ba7b179674884979a286cefdb40d84), [107](https://github.com/wimi321/lizzieyzy-next/commit/8b37616e20401349961def86cb9a08003bf6cdf9), [108](https://github.com/wimi321/lizzieyzy-next/commit/7c16ae92c06bdc92148073eeb1019341415f6518), [109](https://github.com/wimi321/lizzieyzy-next/commit/6a5b21b068d069b2b4682dc907f72ca4fc0cc177), [110](https://github.com/wimi321/lizzieyzy-next/commit/8395cc174bf9e2b15847b92c212b28c32252f49d), [111](https://github.com/wimi321/lizzieyzy-next/commit/41142498a95ba36c77b531613b0f856f4229e498), [112](https://github.com/wimi321/lizzieyzy-next/commit/702512f320e84ffbdb05d477ae9600462f041a0b), [114](https://github.com/wimi321/lizzieyzy-next/commit/b768c7707b68bf1d966c5019cede4a7143999042), [118](https://github.com/wimi321/lizzieyzy-next/commit/07dc9f919886005c97a87a1fc76d8ff3d35c3b5e), [119](https://github.com/wimi321/lizzieyzy-next/commit/e6d6026f3bd05902ecf1f733b4c6a5cd19ac2000), [120](https://github.com/wimi321/lizzieyzy-next/commit/f360decb0440abfa82303ad9fe69cdd67a1070c5), [121](https://github.com/wimi321/lizzieyzy-next/commit/90afca6be97cd52245f815b1f1ffa89c137021e2), [122](https://github.com/wimi321/lizzieyzy-next/commit/5b6a1333da8ec06e0fb219b70fbfd9e67bd9bda6), [123](https://github.com/wimi321/lizzieyzy-next/commit/ac0f7d4d38c48a7a8bf9d0c9aebf0e5ff78694e4), [124](https://github.com/wimi321/lizzieyzy-next/commit/2b45468f144cf17793c83e5c0b44c3b29b74cb92), [125](https://github.com/wimi321/lizzieyzy-next/commit/0e58160a3d270f726797f0d07bf67d9b85ead0c1), [126](https://github.com/wimi321/lizzieyzy-next/commit/5b7c7aaef7acd993053dfda0682a9bb8513fd9e8), [133](https://github.com/wimi321/lizzieyzy-next/commit/470ee88d5ba348ec7cb946a84740ea6168402d29), [135](https://github.com/wimi321/lizzieyzy-next/commit/00b38da747322665d378058eaef721f3242f3430), [139](https://github.com/wimi321/lizzieyzy-next/commit/4dcbeeb679a59841fd3942779e020cbbe545a387), [141](https://github.com/wimi321/lizzieyzy-next/commit/03202192ab183a5c0a0e71e803127fbfca8d6e17), [143](https://github.com/wimi321/lizzieyzy-next/commit/b82b6611b7dc4ce92d1a0f5e2d2063b585201617)（完整SHA/发布包含见源索引）。
- 用户行为/边界：本组包含两条不同责任链。本地：自动快析交还失败后导入另一棋谱，用户显式 Restart 当前本地引擎，在新 reader/generation 重新确认该棋谱的规则和位置后才允许 Space 继续；保持暂停/失败门禁，旧局面、旧规则许可、旧回调不能解锁新局面。远程：remote-only 快析/停止响应 reader lease、断线重连、曲线和外部同步交还；不能因同组提交而混同本地恢复。
- Tauri映射/具名任务意图：本地 T05-LOCAL-IMPORT-RESTART-CHECK（B），受影响 ENG-02（Restart/Run）、ANA-16（任务）、ANA-06（持续分析）、SGF-01/SGF-07（导入/替换）；远程 T05-REMOTE-QUICK-HANDBACK（C），依赖本地所有权契约及 RCOMP-01，涉及 GAME-10 时另消费其确认契约。ANA-11不拥有自动快析。
- 覆盖证据或缺口：PR591的 `f360decb0440abfa82303ad9fe69cdd67a1070c5` 明确仅本地显式重启（`!targetEngine.useRemoteCompute`），不改变远程重连；`90afca6be97cd52245f815b1f1ffa89c137021e2`、`5b6a1333da8ec06e0fb219b70fbfd9e67bd9bda6` 保留比较退出/导入恢复交接。固定 Tauri lifecycle.rs:644–696 已在 Restart 退休旧任务并建立新 Run，document_departure.rs:418–490 拥有本地替换；这是调查起点而非新原生证据或已证缺陷。远程 PR584/587/592/595/596 中 stop response 先核对 reader binding；不复制 Java remoteGTP。
- 处置：本地已支持导入/显式 Restart/继续路径先做有界调查，不等 RCOMP 或远程账号；自动打开快析的新增部分与 T05-AUTO-QUICK-OWNER 串行整合。不得自动重启、自动恢复暂停意图或仅凭 Ready 清除局面确认。远程后继等本地任务所有权和远程协议稳定后接入，无真实服务证据不Passed。
- 后续验收/调查停止条件：本地受支持引擎验证任务交还失败→换谱→用户显式 Restart→新规则/位置确认→用户继续，以及失败、取消、又换谱/规则/引擎、迟到旧回调；新谱/Run/暂停意图不被旧结果污染，无问题记录所测边界，有问题才修复。远程另验真实端点停止/断线/重连、排队重启/取消与外部同步：无旧曲线/旧落子、不隐藏失败，不以loopback代替真实服务。

<a id="udx-012"></a>
## UDX-012 — Match既有编辑守卫继承

当前阶段导航 **R15**（历史来源组 **D**；混合归属以意图索引为准）；条目/后继 **GAME-01**。各来源的有界调查/决定仍须先输出结论，不能把族导航视为新增实现或全部Accepted。明确的其他阶段消费者按其任务合同单独交付；其存在不阻塞独立本地工作。

<a id="ud-03-026"></a>
### UD-03-026 — Match 工作区规则编辑守卫

- **来源 PR 与提交**:
  - 主来源: `PR #426`
  - 完整 40 位 Commit SHA (1 个):
    - `8750dfb4b82b24ef94c59d3861b124ce981ce4f3` (next-2026-09-13.2) — fix(gui): prevent rules dialogs from interrupting engine games (#426)
- **核心文件出处**: `src/main/java/featurecat/lizzie/gui/LizzieFrame.java`, `src/main/java/featurecat/lizzie/gui/SetKataRules.java`
- **用户可见行为与边界**: Match占用时禁止规则编辑入口与应用，防止中途改变盘面上下文；并非#435规则freeze/restore。
- **对等项 / 任务映射**: GAME-01（Accepted）；RULE-01（Accepted，仅编辑Go规则）；`T03-GAME01-RULES-DIALOG-GUARD`。所有历史项仅引用Matrix原范围，最终后继ID由06分配。
- **Tauri 现状 / 证据与残余**: 基线App matchOwnsWorkspace及Match owner保护相关入口/应用；继承E26限定既有守卫，不宣称新引擎规则窗口已实现。
- **额外来源 / 继承条件**: E26原集成93b8410fde3bd0795662470b4d663c8b506cc4e6；DEVELOPMENT§2.10。
- **处置结论**: 已有证据覆盖（限定下述原范围）。任务 kind=`inheritance`；不把Source差异等同Next缺陷。
- **功能断言 / 调查停止条件**: 当前Match期间菜单/快捷键/应用请求不可变更rules/komi；结束后编辑恢复；命名§4.1 E26的fixture与native类各自范围。
- **实际阻塞与未来证据门**: 未来新增规则入口必须接同一owner并自取证据。
- **唯一批次建议 / 责任**: D（仅现有守卫继承）；`T03-GAME01-RULES-DIALOG-GUARD` 拥有此Delta的结果。07/08冻结具体实施合同，06负责同源去重。

<a id="udx-013"></a>
## UDX-013 — 现有Match流式终态屏障调查

当前阶段导航 **R12/R13（逐意图）**（历史来源组 **B**；混合归属以意图索引为准）；条目/后继 **GAME-01, ENG-10**。各来源的有界调查/决定仍须先输出结论，不能把族导航视为新增实现或全部Accepted。明确的其他阶段消费者按其任务合同单独交付；其存在不阻塞独立本地工作。

<a id="ud-03-027"></a>
### UD-03-027 — 流式genmove退役/终态屏障适用性

- **来源 PR 与提交**:
  - 主来源: `PR #428`
  - 完整 40 位 Commit SHA (1 个):
    - `0c75fd540ab93f0e6734e13b777d15451d1b71cf` (next-2026-09-13.2) — fix(engine): retire streaming genmove safely and restore foreground state (#428)
- **核心文件出处**: `docs/SNAPSHOT_NODE_KIND.md`, `src/main/java/featurecat/lizzie/analysis/EngineManager.java`, `src/main/java/featurecat/lizzie/gui/NewEngineGameDialog.java`
- **用户可见行为与边界**: 空ACK不算final；迟到着手仅结清旧请求、不修改新局面；未完成对局停止后稳定恢复才归还前台owner，首手前正确移交dialog reservation。
- **对等项 / 任务映射**: GAME-01 / ENG-02（Accepted）；具名 genmove retirement investigation；`T03-MATCH-GENMOVE-RETIREMENT-INVESTIGATION`。所有历史项仅引用Matrix原范围，最终后继ID由06分配。
- **Tauri 现状 / 证据与残余**: Next KataGoAnalysis不是直接复制Java GTP流；GenericGTP/Match均实际支持，需分adapter检查终态/restore而非认定缺陷。
- **处置结论**: 需有界调查。任务 kind=`investigation`；不把Source差异等同Next缺陷。
- **功能断言 / 调查停止条件**: 问：哪些Nextadapter接收空ACK/流式final，Stop能否在final前release？追踪Match/engine-manager target-final路径；交错旧final、Stop、新Run，断言旧结果不写棋谱，失败保持封存且不假Ready；停止于adapter适用表+已有证据/具名修复。
- **实际阻塞与未来证据门**: 现有GAME-01有限证据；兼容真实引擎future门随可达修复，不要求上游编译。
- **唯一批次建议 / 责任**: B/D（当前Match风险优先）；`T03-MATCH-GENMOVE-RETIREMENT-INVESTIGATION` 拥有此Delta的结果。07/08冻结具体实施合同，06负责同源去重。

<a id="udx-014"></a>
## UDX-014 — 普通规则确认与Match冻结规则生命周期

当前阶段导航 **R15**（历史来源组 **D**；混合归属以意图索引为准）；条目/后继 **GAME-11, GAME-13**。各来源的有界调查/决定仍须先输出结论，不能把族导航视为新增实现或全部Accepted。明确的其他阶段消费者按其任务合同单独交付；其存在不阻塞独立本地工作。

<a id="ud-03-029"></a>
### UD-03-029 — 实际引擎规则确认、Match 冻结/准入/恢复与只读详情

- **来源 PR 与提交**:
  - 主来源: `PR #435`
  - 完整 40 位 Commit SHA (1 个):
    - `fe4af8678dab4bc6bfca72faac71d533dd4ff584` (next-2026-09-13.2) — feat(enginegame): verify and restore match rules (#435)
- **核心文件出处**: src/main/java/featurecat/lizzie/analysis/KataGoRules.java, EngineRulesResult.java；enginegame/MatchRulesAdmission.java, MatchRulesSnapshot.java, MatchRuleOption.java；对应MatchRulesSnapshotTest.java
- **用户可见行为与边界**: 普通窗口按实例setting/query能力报告pending/confirmed/failed；list_commands只证能力不证实际规则。Match接收时冻结目标和双方身份，overlay前捕获双方original；capability matrix支持confirmed、明确unverified consent及reject。仅exact preset归标准，扩展字段/Custom保留。restore originals成功才release；caption/Shift+D/details/save读取immutable snapshot不查询live引擎。
- **对等项 / 任务映射**: GAME-01（Accepted，既有Match）；具名 match-rules successor；RULE-01仅SGF编辑规则；`T03-MATCH-RULES-LIFECYCLE`。所有历史项仅引用Matrix原范围，最终后继ID由06分配。
- **Tauri 现状 / 证据与残余**: 此规则生命周期/保存详情不在RULE-01或026入口guard历史范围。
- **额外来源 / 继承条件**: fe4af867 body；enginegame/MatchRulesAdmission.java, MatchRulesSnapshotTest.java, MatchRuleOption.java；local准备合同。
- **处置结论**: 需后继功能/修复。任务 kind=`feature`；不把Source差异等同Next缺陷。
- **功能断言 / 调查停止条件**: 双方confirmed目标一致才confirmed准入；可query失败/mismatch/incomplete拒绝，unsupported按明确且身份/target/causes绑定consent；取消/恢复失败不放行、不修改当前棋谱。Custom参数保真；更换引擎后旧query无权更新；结束后只读详情与save仍同一冻结值。generated details独立于个人C且C字节不变。
- **实际阻塞与未来证据门**: engine-rules能力/身份及Match owner；普通规则确认、Match准入/恢复和只读详情可按已冻结行为推进。仅generated details的保存/共享残余依赖`T03-MATCH-RULES-STORAGE-DECISION`结果；04#531选项演进交06合并。
- **具名决定 T03-MATCH-RULES-STORAGE-DECISION**（kind=`decision`，D，owner为规则保存责任人）：问题是如何保存/共享同一immutable match-rule snapshot且不写个人C、不建立ADR0003原范围禁止的第二生成文本通道。输入为ADR0003、GAME-05/SGF-05既有保存与重开合同、PR435冻结详情字段及04#531规则参数。输出一份明确的数据归属/结构化表示/兼容读取边界决定，逐字段说明保存、重开和共享的同值断言、未知字段处置及个人C字节不变；若需要改变现有ADR或产品可观察范围，记录逐项批准后才开放该save残余。停止于可执行且经适用批准的保存合同，不在本审计选择schema或实现产品。此决定不等待runner/policy，也不阻塞已冻结的准入、恢复或只读详情。
- **唯一批次建议 / 责任**: B/D（普通确认B；Match D）；`T03-MATCH-RULES-LIFECYCLE` 拥有此Delta的结果。07/08冻结具体实施合同，06负责同源去重。

<a id="ud-04-034"></a>
### UD-04-034 五rule families、custom精确参数与saved preservation

- **Source / event**: PR #531 (集成事件 34) | 共 3 个提交；第一父E34。
- **完整source commits（与§6反向对应）**:
  - [84bfcd5f8e45916f6b58027ed44a3d2f127e4c94](https://github.com/wimi321/lizzieyzy-next/commit/84bfcd5f8e45916f6b58027ed44a3d2f127e4c94) — Merge pull request #531 from qiyi71w/fix/issue-516-match-rules
  - [028b8b0fa48a4d4e6424cabaafd05813cd39ca99](https://github.com/wimi321/lizzieyzy-next/commit/028b8b0fa48a4d4e6424cabaafd05813cd39ca99) — Merge verified main into match rule picker fix
  - [88d6b0bb4f7ea87930bf5b8297ef97cdca1b43b6](https://github.com/wimi321/lizzieyzy-next/commit/88d6b0bb4f7ea87930bf5b8297ef97cdca1b43b6) — fix(enginegame): simplify match rule choices
- **发布包含**: 最早有API/ancestry证据收录 `next-2026-09-22.2`（预发布）；全部source也含于正式09-26.1/.2。准备target仅记prepared，不由本字段推断准备版本已发布。
- **实际behavior / sourceDelta**: 88d6b0bb与endpoint MatchRuleOption定义Chinese、Japanese/Korean、AGA/BGA、New Zealand、Tromp-Taylor；只有exact semantic parameter match才归standard family，其余custom。打开selector/旧配置迁移不得默认重写已有preset参数，直到用户明确更改。不是Ing，也不只是两个命令字符串。
- **Canonical owner / scope**: GAME-04 Accepted原rules/budgets；GAME-01/02/03 Accepted原matches。
- **候选/覆盖与边界**: 尚无此新增行为的可继承运行证据；不把Java修复判为当前Next缺陷。
- **Disposition**: 具名rules后继，原Accepted不改Partial/Deferred。
- **责任意图**: T04-RULES。
- **Observable assertions**: 五family逐参数匹配；custom差一个参数仍custom；加载/打开/取消/无改动保存保留saved rule parameters字节/语义；explicit choice才变更，match参与者实际准入规则一致，adapter不支持可见拒绝。

<a id="ud-05-07"></a>
### UD-05-07 

- 来源：[018](https://github.com/wimi321/lizzieyzy-next/commit/133d9ca6294ceaa52fb747c2c32c53c54de7bfe9), [019](https://github.com/wimi321/lizzieyzy-next/commit/1c33a7862dabfbf6d04a2f544db293f80ec47e78), [047](https://github.com/wimi321/lizzieyzy-next/commit/8c43fdd24b01acd6485b8b3911708290ae18ab75)（完整SHA/发布包含见源索引）。
- 用户行为/边界：古代中国规则加入对局选择与引擎规则传递。
- Tauri映射/具名任务意图：GAME-04（Rules & komi）为规则受影响项，GAME-01/02仅为会话/开始对局前置；T05-ANCIENT-RULES（D）承接新增规则准入，不扩大既有规则 Accepted 范围。
- 覆盖证据或缺口：PR556，现有基础对局只在既有规则范围Accepted。
- 处置：后继适配；先冻结Next公开规则与后端能力映射，不复制Java字段。
- 后续验收/调查停止条件：支持引擎可建立同规则棋局、保存/重开一致；不支持明确拒绝而非换规则。

<a id="udx-015"></a>
## UDX-015 — 普通本地位置/规则/reader精确恢复调查

当前阶段导航 **R12/R13（逐意图）**（历史来源组 **B**；混合归属以意图索引为准）；条目/后继 **ENG-02, ENG-09**。各来源的有界调查/决定仍须先输出结论，不能把族导航视为新增实现或全部Accepted。明确的其他阶段消费者按其任务合同单独交付；其存在不阻塞独立本地工作。

<a id="ud-03-032"></a>
### UD-03-032 — 普通分析位置/reader/slot 与只读同步边界

- **来源 PR 与提交**:
  - 主来源: `PR #439`
  - 完整 40 位 Commit SHA (1 个):
    - `860caa60c8173920ec9456dc6e405f1108f83a16` (next-2026-09-13.2) — fix(analysis): confirm positions before sync analysis (#439)
- **核心文件出处**: `src/main/java/featurecat/lizzie/analysis/Leelaz.java`, `src/main/java/featurecat/lizzie/analysis/ReadBoard.java`, `src/main/java/featurecat/lizzie/gui/LizzieFrame.java`
- **用户可见行为与边界**: ordinary分析捕获board target、reader incarnation、engine slot；确认compound restore后启动，failed/retired lineage隔离，导航/SGF reload保留兼容节点缓存；ReadBoard一次完整confirmed snapshot只恢复一次，须当前target/engine/用户policy。
- **对等项 / 任务映射**: ANA-06 / ENG-05（Accepted）；READ-02（Partial）；具名 position-confirmation investigation；`T03-ANALYSIS-CONFIRMED-POSITION-INVESTIGATION`。所有历史项仅引用Matrix原范围，最终后继ID由06分配。
- **Tauri 现状 / 证据与残余**: Next普通KataGoAnalysis请求携带target快照/identity，与JavaLeelaz GTP确认不同；READ-02只读且GAME-10未实现。ADR0004不是已覆盖证据。
- **处置结论**: 需有界调查。任务 kind=`investigation`；不把Source差异等同Next缺陷。
- **功能断言 / 调查停止条件**: 问：各现有adapter的位置dispatch与publication是否同一target/run/generation/slot，restore失败如何封存？检查请求/结果owner与READ-02generation；停止于adapter适用表、有限历史证据与具名残余。旧reader/slot/board不得采纳结果；只读sync不发外部着手。
- **实际阻塞与未来证据门**: 实际Nextordinary路径证据；外部落子另归034/T01-EXTERNAL，不用ADR作实现。
- **唯一批次建议 / 责任**: B/C；`T03-ANALYSIS-CONFIRMED-POSITION-INVESTIGATION` 拥有此Delta的结果。07/08冻结具体实施合同，06负责同源去重。

<a id="ud-03-038"></a>
### UD-03-038 — SGF 导入后的规则确认与分析恢复适用性

- **来源 PR 与提交**:
  - 主来源: `PR #452`
  - 完整 40 位 Commit SHA (1 个):
    - `e044191a0ed28b398fcfcefe6c539415e4060f3c` (next-2026-09-13.2) — fix(sgf): confirm rules before resuming analysis (#452)
- **核心文件出处**: `src/main/java/featurecat/lizzie/analysis/EngineManager.java`, `src/main/java/featurecat/lizzie/rules/Board.java`
- **用户可见行为与边界**: 载入后按generation同步规则/komi，确认成功再恢复；新载入/引擎替换使旧确认无效，错误不分析错上下文。
- **对等项 / 任务映射**: SGF-01 / RULE-01（Accepted，解析/编辑）；ANA-06 / ANA-16（Accepted）；具名 rules-position investigation；`T03-ANALYSIS-SGF-RULES-SYNC-INVESTIGATION`。所有历史项仅引用Matrix原范围，最终后继ID由06分配。
- **Tauri 现状 / 证据与残余**: 已有SGF RU/KM解析不能证明引擎动态query/restore顺序；RULE-01不是引擎规则窗口。
- **处置结论**: 需有界调查。任务 kind=`investigation`；不把Source差异等同Next缺陷。
- **功能断言 / 调查停止条件**: 问：实际KataGoAnalysis快照与GenericGTP位置同步各如何携带RU/KM并封存失败？检查导入→dispatch→result目标；停止于支持adapter证据/差异表；不同KM/RU旧generation结果不得采纳，不以胜率数值“看起来正确”作为唯一证明。
- **实际阻塞与未来证据门**: 029engine-rules合同与032target调查；如果源query概念不适用记录精确等价请求边界。
- **唯一批次建议 / 责任**: B；`T03-ANALYSIS-SGF-RULES-SYNC-INVESTIGATION` 拥有此Delta的结果。07/08冻结具体实施合同，06负责同源去重。

<a id="ud-03-044"></a>
### UD-03-044 — exact snapshot 的路径/化身准入适用性

- **来源 PR 与提交**:
  - 主来源: `PR #463`
  - 完整 40 位 Commit SHA (1 个):
    - `be31e18b152c509ee4b9e52e737f1cc5f7b5235a` (next-2026-09-18.2) — fix(engine): use GTP-safe paths for exact snapshot restore (#463)
- **核心文件出处**: `src/main/java/featurecat/lizzie/analysis/ExactSnapshotEngineRestore.java`
- **用户可见行为与边界**: 暂存SGF必须当前engine incarnation可读且GTP-safe，拒unsupported虚拟/包装FS；失败不错误restore或宣称Ready。
- **对等项 / 任务映射**: ENG-02 / ENG-10 / SGF-07（Accepted，各原范围）；具名 restore-path investigation；`T03-RESTORE-GTP-PATH-INVESTIGATION`。所有历史项仅引用Matrix原范围，最终后继ID由06分配。
- **Tauri 现状 / 证据与残余**: KataGoAnalysis参数快照与GenericGTP重建不同；绝对路径本身不是本化身准入/quoted-safe动态证据。
- **处置结论**: 需有界调查。任务 kind=`investigation`；不把Source差异等同Next缺陷。
- **功能断言 / 调查停止条件**: 问：哪个受支持adapter真的发送SGF路径？检查其physical path/编码/quote/reader与cleanup；停止于无该路径的证据或受支持路径边界与缺口；不支持路径有typed拒绝，替换Run旧restore不可写新盘。
- **实际阻塞与未来证据门**: GAME-01现有exactContinue/GenericGTP证据有限；新resource/engine变化需要真实证据。
- **唯一批次建议 / 责任**: B/D；`T03-RESTORE-GTP-PATH-INVESTIGATION` 拥有此Delta的结果。07/08冻结具体实施合同，06负责同源去重。

<a id="ud-04-019"></a>
### UD-04-019 引擎同步失败不能假确认

- **Source / event**: PR #510 (集成事件 19) | 共 3 个提交；第一父E19。
- **完整source commits（与§6反向对应）**:
  - [1b7f44a4b696c2bcc2c98316e1fc2c38306f5c64](https://github.com/wimi321/lizzieyzy-next/commit/1b7f44a4b696c2bcc2c98316e1fc2c38306f5c64) — Merge pull request #510 from wimi321/fix/sync-timeout-race-20260918
  - [50d6ff2783863efa0444a0e33de567f92145190e](https://github.com/wimi321/lizzieyzy-next/commit/50d6ff2783863efa0444a0e33de567f92145190e) — Keep headless rules fixture from scheduling uninitialized menu updates
  - [2e2d3dac15cd9da04086edf4e10f3e86168b4539](https://github.com/wimi321/lizzieyzy-next/commit/2e2d3dac15cd9da04086edf4e10f3e86168b4539) — Reject failed position responses during synchronization confirmation
- **发布包含**: 最早有API/ancestry证据收录 `next-2026-09-18.2`（预发布）；全部source也含于正式09-26.1/.2。准备target仅记prepared，不由本字段推断准备版本已发布。
- **实际behavior / sourceDelta**: PR510的2e2d3dac拒绝把失败/error-response同步序列标成功，另含无头规则test fixture菜单初始化。同步确认是用户结果/位置正确性，不能因Rust权威而全覆盖。
- **Canonical owner / scope**: ENG-02/04、ANA-03同步边界（既有范围）。
- **候选/覆盖与边界**: H-SWITCH只保留原失败和stale fixture范围；没有本Java同步race的新增native等价证据。
- **Disposition**: 源机制保留；差异有界调查。
- **责任意图**: T04-SYNC-CONFIRM。
- **Observable assertions**: 识别当前Next相应位置同步/确认路径：失败、timeout、late/replaced identity不能发已同步/分析结果；成功必须绑定目标位置和原Run。若现fixture充分，精确引用后闭合；需新runtime证据则交受影响功能，不宣称Java故障在Next已复现。

<a id="ud-04-020"></a>
### UD-04-020 Release upload恢复与同步修复整合

- **Source / event**: PR #511 (集成事件 20) | 共 5 个提交；第一父E20。
- **完整source commits（与§6反向对应）**:
  - [c96e672fc5ade639d2bb454b86fc6fe99a9df9db](https://github.com/wimi321/lizzieyzy-next/commit/c96e672fc5ade639d2bb454b86fc6fe99a9df9db) — Merge pull request #511 from wimi321/fix/release-upload-recovery-20260918
  - [cfa115d4d4b5cba67f68fc0e88217320234b6f43](https://github.com/wimi321/lizzieyzy-next/commit/cfa115d4d4b5cba67f68fc0e88217320234b6f43) — Merge branch 'fix/sync-timeout-race-20260918' into fix/release-upload-recovery-20260918
  - [c892a1c46f387b6b1cc4dc0f5ecda44a3c47c698](https://github.com/wimi321/lizzieyzy-next/commit/c892a1c46f387b6b1cc4dc0f5ecda44a3c47c698) — Merge branch 'fix/sync-timeout-race-20260918' into fix/release-upload-recovery-20260918
  - [5825960ce0214eeda63877ead54f27a5388bb4fd](https://github.com/wimi321/lizzieyzy-next/commit/5825960ce0214eeda63877ead54f27a5388bb4fd) — Keep HTTP status in bounded upload retry diagnostics
  - [dddaeeef56afc34eed394ed9c9720605f7675d94](https://github.com/wimi321/lizzieyzy-next/commit/dddaeeef56afc34eed394ed9c9720605f7675d94) — Make release asset uploads retryable without overwriting completed files
- **发布包含**: 最早有API/ancestry证据收录 `next-2026-09-18.2`（预发布）；全部source也含于正式09-26.1/.2。准备target仅记prepared，不由本字段推断准备版本已发布。
- **实际behavior / sourceDelta**: PR511仅release asset上传重试/HTTP诊断和已有同步修复merge；不覆盖已上传完成资产。重试权限是发行asset流程，不是账号写或引擎命令自动重试许可。
- **Canonical owner / scope**: Java上传机制；同步归UD-04-019。
- **候选/覆盖与边界**: 没有独立Tauri行为覆盖声称。
- **Disposition**: 内部上传排除；同步来源去重。
- **责任意图**: 无独立产品任务；关联T04-SYNC-CONFIRM。
- **Observable assertions**: 上传与同步源分别保留：同步行为只由UD-04-019拥有，merge不新增feature；未来账号写不继承此重试策略。

<a id="ud-05-13"></a>
### UD-05-13 

- 来源：[038](https://github.com/wimi321/lizzieyzy-next/commit/dcc71136f547ed3fb6157cbdb14b4c8c23a7bd8f), [040](https://github.com/wimi321/lizzieyzy-next/commit/542e8b0e2904849cbf078ec92b601ea1b0bde53a), [047](https://github.com/wimi321/lizzieyzy-next/commit/8c43fdd24b01acd6485b8b3911708290ae18ab75)（完整SHA/发布包含见源索引）。
- 用户行为/边界：外部 GTP 引擎的局面恢复也覆盖本地导入 SGF/选中节点，不等于外部棋盘同步。PR566以五个原始 SGF 比较 direct `loadsgf` 与 shell launcher 的 in-band 路径，主线正反导航及分支端点须保持 stones、to-play、komi 和真实 move tail；初始让子来源与当前 reader 能力须在任何棋盘修改前确认，不能伪造 PASS 或丢弃 setup。
- Tauri映射/具名任务意图：本地 ENG-02/ENG-10（Run/协议）及 SGF-01、REVIEW-01（导入/浏览）是现有受影响面；T05-LOCAL-EXACT-RESTORE-CHECK（B）独立调查。READ-02 只对应外部同步，T05-EXTERNAL-EXACT-RESTORE（C）与未来 GAME-10 分开承接，不成为本地调查的前置。
- 覆盖证据或缺口：固定 DEVELOPMENT §2.3–2.4 的当前契约限定 square 2–19、已准入规则/komi、空根/PL、交替落子/PASS、KataGo 黑子根让子；每次受支持请求新同步，白/混合/中途 setup 等未准入输入明确拒绝。这是源码/契约范围，不是本轮 native pass，也不要求移植 Java SNAPSHOT、文件命令或进程拓扑。E05-READBOARD 仅证明外部帧重建，不证明本地 SGF→引擎恢复。
- 处置：本地已支持路径做有界正确性调查，使用冻结导入谱/选中节点与实际支持适配器核对引擎位置；没有失败则记录所测范围等价，有失败才提出修复。保留不支持 setup 的可见拒绝，不借上游变化扩充当前协议准入。外部同步仍保留既有偏离说明及 GAME-10 后继。
- 后续验收/调查停止条件：本地不等待远程凭据：受支持让子、主线前后及分支端点逐项比较 stones/to-play/komi/真实 tail，拒绝输入不改变当前棋谱或既有 Run；记录实际引擎版本/适配器及范围。外部 READ-02 另以真实让子/重建帧、Save/reopen比较局面；GAME-10 两种落子模式各由权威精确后继确认，ACK不推进回合。

<a id="udx-016"></a>
## UDX-016 — 只读同步退休与迟到帧调查

当前阶段导航 **R14**（历史来源组 **C**；混合归属以意图索引为准）；条目/后继 **READ-01, READ-02**。各来源的有界调查/决定仍须先输出结论，不能把族导航视为新增实现或全部Accepted。明确的其他阶段消费者按其任务合同单独交付；其存在不阻塞独立本地工作。

<a id="ud-03-034"></a>
### UD-03-034 — Stop sync 退休确认与外部写回未来义务

- **来源 PR 与提交**:
  - 主来源: `PR #440`
  - 完整 40 位 Commit SHA (1 个):
    - `3c63881300f6d670237651e31aa6958028d3fd41` (next-2026-09-13.2) — fix(sync): retire local confirmations on stop (#440)
- **核心文件出处**: `src/main/java/featurecat/lizzie/analysis/ReadBoard.java`, `src/main/java/featurecat/lizzie/analysis/ReadBoardStream.java`, `src/main/java/featurecat/lizzie/rules/Board.java`
- **用户可见行为与边界**: 同步Stop retire ACK权限/queued placement，按Board history序列化local admission/failure；endsync与frame完成不同。
- **对等项 / 任务映射**: READ-02（Partial，只读）；GAME-10（Deferred）；T01-READBOARD / T01-EXTERNAL；`T03-READ02-STOP-AND-GAME10-TURN-RETIREMENT`。所有历史项仅引用Matrix原范围，最终后继ID由06分配。
- **Tauri 现状 / 证据与残余**: 现有只读sync有generation Stop隔离（E34）；没有engine-turn请求/reserve/外部command/turn confirmation实现。外部部分保留futurefeature而非Covered。
- **额外来源 / 继承条件**: E34见01 T01-READBOARD原候选三段证据；ADR0004是约束而非native通过。
- **处置结论**: 需后继功能/修复。任务 kind=`feature`；不把Source差异等同Next缺陷。
- **功能断言 / 调查停止条件**: 只读Stop后旧frame不能改last-good/current，留可编辑棋谱；未来双向Stop封存pending turn/队列，迟到ACK不得commit或重发。两种Final-decision/Leading-candidate均须exact authoritative successor，ACK/坐标相同不足。
- **实际阻塞与未来证据门**: GAME-10所需双向协议/capability调查、唯一Match/session/turn identity；停止/失败不得自动重试写操作。
- **唯一批次建议 / 责任**: C；`T03-READ02-STOP-AND-GAME10-TURN-RETIREMENT` 拥有此Delta的结果。07/08冻结具体实施合同，06负责同源去重。

<a id="ud-04-041"></a>
### UD-04-041 PR539 retirement/producer确认测试

- **Source / event**: PR #539 (集成事件 38) | 共 3 个提交；第一父E38。
- **完整source commits（与§6反向对应）**:
  - [4ef8bbf5dad119728045a9cbf8720207c4ef6820](https://github.com/wimi321/lizzieyzy-next/commit/4ef8bbf5dad119728045a9cbf8720207c4ef6820) — test: await retirement before restarting engine game
  - [c1308069813aa6091aceda751b24c612d372dcda](https://github.com/wimi321/lizzieyzy-next/commit/c1308069813aa6091aceda751b24c612d372dcda) — test(sync): verify producer snapshot confirmation
  - [10b1781ed3483ebc554fc5118ed9ee00d19537c3](https://github.com/wimi321/lizzieyzy-next/commit/10b1781ed3483ebc554fc5118ed9ee00d19537c3) — QA integrate PR 539
- **发布包含**: 最早有API/ancestry证据收录 `next-2026-09-26.1`（正式发布）；全部source也含于正式09-26.1/.2。准备target仅记prepared，不由本字段推断准备版本已发布。
- **实际behavior / sourceDelta**: 4ef8bbf5与c1308069分别只改src/test：restart对局test等待retirement，ReadBoardSyncDecisionTest验证producer snapshot confirmation。没有production change；10b1781e为merge。测试名字不能说明引擎OOM或Tauri修复已发生。
- **Canonical owner / scope**: GAME-01/03原lifecycle；READ-02/T01-READBOARD。
- **候选/覆盖与边界**: H-READBOARD可引用真实producer/PL条件，不是泛engine lifecycle；本测试源不是额外运行证明。
- **Disposition**: test-only来源，不新增feature或runtime Covered。
- **责任意图**: 无独立产品任务；T01-READBOARD与现match owners原证据。
- **Observable assertions**: 源码角色和source ownership保持；真实READ-02确认按H-READBOARD及其Partial残余承接，外部写GAME-10独立。match现Accepted保留原scope，不造额外重复feature任务。

<a id="ud-05-06"></a>
### UD-05-06 

- 来源：[016](https://github.com/wimi321/lizzieyzy-next/commit/41121557a82d6f447300671207c69c4edb95a6cc), [017](https://github.com/wimi321/lizzieyzy-next/commit/7c48b1c01fe07311f028809cae716fe145e1b341), [047](https://github.com/wimi321/lizzieyzy-next/commit/8c43fdd24b01acd6485b8b3911708290ae18ab75)（完整SHA/发布包含见源索引）。
- 用户行为/边界：Fox窗口标题/帧暂时落后时保持已确认落子，不能把迟到识别当撤销。
- Tauri映射/具名任务意图：READ-02；T05-SYNC-LAG-CHECK（C，前置正确性调查）
- 覆盖证据或缺口：PR554；继承下方 E05-READBOARD 的 Windows 修复候选、目标与限制，不继承初始候选的错误重建 PL；落后标题/新帧排序仍是本行待调查边界，不是 Java 同实现或当前缺陷证明。
- 处置：有界调查：现有readboard帧支持路径可达；未证明缺陷，不回退已验收状态。
- 后续验收/调查停止条件：重放来自受支持Fox场景的落后标题/新帧顺序，确认已落子不回滚；若不同则按唯一外部权威契约修复并实际桌面复验。

<a id="udx-017"></a>
## UDX-017 — 保存条目的线程来源与动态读回

当前阶段导航 **R12/R13（逐意图）**（历史来源组 **B**；混合归属以意图索引为准）；条目/后继 **ENG-12, ENG-13**。各来源的有界调查/决定仍须先输出结论，不能把族导航视为新增实现或全部Accepted。明确的其他阶段消费者按其任务合同单独交付；其存在不阻塞独立本地工作。

<a id="ud-03-036"></a>
### UD-03-036 — 按保存条目的 CFG/BENCHMARK 与有效override

- **来源 PR 与提交**:
  - 主来源: `PR #450` (包含被集成 PR: #437)
  - 完整 40 位 Commit SHA (1 个):
    - `d25e18355dba14b47f082c6c0b351c6f1bb4b439` (next-2026-09-13.2) — feat(engine): scope KataGo tuning to saved engines (#450)
- **核心文件出处**: src/main/java/featurecat/lizzie/util/EngineThreadPolicy.java；gui/EngineData.java, SetKataEngines.java；util/katago/tuning；对应EngineThreadPolicyPersistenceTest.java
- **用户可见行为与边界**: 以稳定saved-entry ID持久保存CFG/BENCHMARK、推荐和sourceRevision，undecided默认CFG；benchmark用选定条目持久command、所有config layers、working directory，结果互不污染。explicit有效override优先，不重写userconfig。Save才提交policy；cancel/failure保留。engine/config/model/command/workdir环境changed/unknown可见。remote/javaSSH/externalSSH不本地调优；无全局推荐覆盖。
- **对等项 / 任务映射**: ENG-01（Accepted，profiles）；具名 saved-thread-policy successor；SSH-01/RCOMP-01仅远端所有权关联；`T03-THREAD-SAVED-ENTRY-POLICY`。所有历史项仅引用Matrix原范围，最终后继ID由06分配。
- **Tauri 现状 / 证据与残余**: 静态argv历史证据仅基础；本条新增本地source/effective/selected-entry benchmark，不归ENG-08排序。
- **额外来源 / 继承条件**: d25e183 body；util/EngineThreadPolicy.java:25–85,87–128,131–213；EngineThreadPolicyPersistenceTest.java。
- **处置结论**: 需后继功能/修复。任务 kind=`feature`；不把Source差异等同Next缺陷。
- **功能断言 / 调查停止条件**: 两个saved entries benchmark/保存/重载保持ID隔离；CFG不注入推荐，合法BENCHMARK只本条目launch生效；explicit override不被替换；无推荐不可切BENCHMARK；deleted/changed entry和stale revision结果不写错目标；环境unknown/changed可见；cancel/失败durable值不变；远端无本地benchmark。
- **实际阻塞与未来证据门**: 推荐保存与启动采用的集成消费`T03-PERFORMANCE-CUSTOM-BENCHMARK-RUNNER`交付的目标ID/冻结输入revision结果，以及T01-RESOURCE目标identity、既有profile/persistence owner；CFG/显式override等合同可先冻结，runner不反向等待本策略实现。05runtime controls独立，不据本条提升动态控制。
- **唯一批次建议 / 责任**: B；`T03-THREAD-SAVED-ENTRY-POLICY` 拥有此Delta的结果。07/08冻结具体实施合同，06负责同源去重。

<a id="ud-03-053"></a>
### UD-03-053 — 实际 cfg 线程别名/临时 override 适用性

- **来源 PR 与提交**:
  - 主来源: `PR #492`
  - 完整 40 位 Commit SHA (2 个):
    - `73ac3e422463c06b8b94b1ab9bf1b8fac58f63a6` (next-2026-09-18.2) — Merge pull request #492 from wimi321/fix/analysis-thread-alias
    - `2fed2f5b4057d46373ed2287f7ba742c24cffca0` (next-2026-09-18.2) — Avoid conflicting KataGo search thread aliases during analysis
- **核心文件出处**: `src/main/java/featurecat/lizzie/util/KataGoRuntimeHelper.java`
- **另有导入来源（不重复计能力）**: `7c88a25c031ddf633fc37706902d11be9ab10b1b`。这些源同时保留其内部分组。
- **用户可见行为与边界**: managed analysis launch注入numSearchThreadsPerAnalysisThread前，用临时empty numSearchThreads=消除cfg/include旧alias；KataGo两者共存会拒绝。explicit numSearchThreads/numAnalysisThreads overrides优先且用户config bytes不变。不是-threads/-search-threads。
- **对等项 / 任务映射**: ENG-01（Accepted，仅profiles）；036saved-policy successor；具名 alias applicability investigation；`T03-THREAD-ALIAS-APPLICABILITY`。所有历史项仅引用Matrix原范围，最终后继ID由06分配。
- **Tauri 现状 / 证据与残余**: 历史静态argv只证明原引擎参数，未证此cfg/include冲突；Next actual analysis launch path须比较。
- **额外来源 / 继承条件**: 2fed2f5b patch:47–52,85–126；KataGoRuntimeHelper。
- **处置结论**: 需有界调查。任务 kind=`investigation`；不把Source差异等同Next缺陷。
- **功能断言 / 调查停止条件**: 问：Next准入adapter是否使用此managed alias且config layers存在冲突？解析实际effective launch而非只搜生成键；停止于无注入路径证据或明确冲突/precedence合同；cfg/include两alias、explicit两override分别保真，文件字节不改。
- **实际阻塞与未来证据门**: 036策略/真实launch evidence；兼容真实KataGo门，后续04#503HumanSL同源交叉。
- **唯一批次建议 / 责任**: B；`T03-THREAD-ALIAS-APPLICABILITY` 拥有此Delta的结果。07/08冻结具体实施合同，06负责同源去重。

<a id="ud-04-048"></a>
### UD-04-048 顶层Windows layout与portable QA路径

- **Source / event**: PR #547 (集成事件 38) | 共 5 个提交；第一父E38。
- **完整source commits（与§6反向对应）**:
  - [c58919c4257886baac9b9e4b7387a08643edf65c](https://github.com/wimi321/lizzieyzy-next/commit/c58919c4257886baac9b9e4b7387a08643edf65c) — fix(qa): honor explicit Python and portable Windows test paths
  - [b6eeeef64ae77a093468b7b47d7059835fa8282c](https://github.com/wimi321/lizzieyzy-next/commit/b6eeeef64ae77a093468b7b47d7059835fa8282c) — fix(gui): size performance actions before native layout and verify Windows saves
  - [1baa60749cd255ed2e7ece4a827f709422a8b764](https://github.com/wimi321/lizzieyzy-next/commit/1baa60749cd255ed2e7ece4a827f709422a8b764) — Merge current main into Windows integration candidate
  - [5aad75550e3e4a4877dd560905d704c1353fec86](https://github.com/wimi321/lizzieyzy-next/commit/5aad75550e3e4a4877dd560905d704c1353fec86) — fix(gui): keep measured tuning confirmation controls visible on Windows
  - [37b41f6686c1db56a4ea06d86648d74d3642197b](https://github.com/wimi321/lizzieyzy-next/commit/37b41f6686c1db56a4ea06d86648d74d3642197b) — Merge Windows-validated PR integration #547
- **发布包含**: 最早有API/ancestry证据收录 `next-2026-09-26.1`（正式发布）；全部source也含于正式09-26.1/.2。准备target仅记prepared，不由本字段推断准备版本已发布。
- **实际behavior / sourceDelta**: PR547 direct修复5aad7555确保measured controls不clipped，b6eeeef6在realization前保留performance action sizes；c58919c4使QA尊重explicit Python/portable Windows路径，1baa6074及37b41f66是merge。不是SGF dirty更新修复，也不是WebView天然全DPI覆盖。
- **Canonical owner / scope**: 诊断/调优后继的可达布局；Java QA内部。
- **候选/覆盖与边界**: 原R7layout/window范围保留；本新增窗口尚无Next native证据。
- **Disposition**: 功能布局gate；QA-only部分排除。
- **责任意图**: T04-DIAGNOSTICS、T04-MEASURED-TUNING。
- **Observable assertions**: 后继调优/诊断controls在实际适用work-area/DPI可见可达、长内容有scroll、Apply/Restore/Cancel键盘可到；具体平台条件由实际UI风险/原合同给出，不凭48db泛化。portable QA仅保留来源，不复制Python paths。

<a id="ud-05-09"></a>
### UD-05-09 

- 来源：[023](https://github.com/wimi321/lizzieyzy-next/commit/89e85d721f6f4a2b8dbaa2f89fb4223de33e3e72), [024](https://github.com/wimi321/lizzieyzy-next/commit/5b447d22017296a7cbd543d038880402e313f7ac), [025](https://github.com/wimi321/lizzieyzy-next/commit/c49e299c14bdab1cb0fe94a8bd96901e49c1fbd7), [026](https://github.com/wimi321/lizzieyzy-next/commit/0812ab88b037cc0eb6387faf9b75c1b50ef6c98a), [027](https://github.com/wimi321/lizzieyzy-next/commit/d1577fefde3a29f50982392fdf864922e080cb33), [047](https://github.com/wimi321/lizzieyzy-next/commit/8c43fdd24b01acd6485b8b3911708290ae18ab75)（完整SHA/发布包含见源索引）。
- 用户行为/边界：动态线程输入、确认读回、临时覆盖与恢复配置来源；鼠标/键盘生效一致；远程重连不擅自重放本地覆盖。
- Tauri映射/具名任务意图：ENG-01（配置/加载）、ENG-09（多配置）为保存配置引用，ENG-02（生命周期）为当前 Run 前置；动态控制新增责任 T05-RUNTIME-THREADS（B），不是 ANA-11 胜率图能力。
- 覆盖证据或缺口：PR561与Leelaz/SetKataEngines改动。固定 [DEVELOPMENT §2.1](https://github.com/qiyi71w/lizzieyzy-next-tauri/blob/55795fab49b80a681a9fa544950e18f5f2da4cc6/docs/DEVELOPMENT.md#L199-L207)：候选 `b23f3eb16c16ad63bf4570d7ecc9bf22c274ea05`，Windows 原生、KataGo v1.12.3 Eigen CPU AVX2/FMA、b20模型；显式 Restart 后进程 argv 观察到 `-override-config numSearchThreads=2`。只继承静态启动参数，不证明动态命令/读回；适用失效规则见 E05-INHERITANCE。
- 处置：后继功能/协议准入决策；区分保存配置、启动参数、当前run实际值，Next支持范围需协议验证。
- 后续验收/调查停止条件：真实支持引擎修改并确认实际值，失效/超时/旧run响应不覆盖编辑；重启/切换/断线后的临时值寿命明确，不能假定Java1..1024就是Next协议。

<a id="udx-018"></a>
## UDX-018 — 全局离线功能搜索与精确焦点

当前阶段导航 **R11**（历史来源组 **A**；混合归属以意图索引为准）；条目/后继 **UI-07**。各来源的有界调查/决定仍须先输出结论，不能把族导航视为新增实现或全部Accepted。明确的其他阶段消费者按其任务合同单独交付；其存在不阻塞独立本地工作。

<a id="ud-03-040"></a>
### UD-03-040 — 全局离线搜索与精确导航

- **来源 PR 与提交**:
  - 主来源: `PR #458`
  - 完整 40 位 Commit SHA (1 个):
    - `839f9976fa32310afe378038b6caf7ea03f47cf1` (next-2026-09-18.2) — Merge pull request #458 from qiyi71w/plan/function-search-navigation
- **核心文件出处**: `src/main/java/featurecat/lizzie/gui/FunctionSearchController.java`
- **用户可见行为与边界**: Ctrl/Command+K、菜单/工具栏入口共用本地功能catalog；多语言keyword及拼音/英文搜索、键盘选择执行；按可用性禁止非法命令，设置定位特定target。
- **对等项 / 任务映射**: 具名 function-search successor；APP-05（Accepted，registry）基础；`T03-NAV-GLOBAL-FUNCTION-SEARCH`。所有历史项仅引用Matrix原范围，最终后继ID由06分配。
- **Tauri 现状 / 证据与残余**: 基线无全局搜索；UI-01只桌面chrome，不扩大Accepted。
- **处置结论**: 需后继功能/修复。任务 kind=`feature`；不把Source差异等同Next缺陷。
- **功能断言 / 调查停止条件**: 离线按各来源关键词检索目标；上下/回车执行当前可用命令；禁用项理由可见；打开设置聚焦指定target不是任意首输入；取消见040a保留原workspace与焦点。
- **实际阻塞与未来证据门**: APP-05 registry/action owner、PREF-01/SGF-13target；不能以搜索旁路dirty/Match安全。
- **唯一批次建议 / 责任**: A；`T03-NAV-GLOBAL-FUNCTION-SEARCH` 拥有此Delta的结果。07/08冻结具体实施合同，06负责同源去重。

<a id="ud-03-040a"></a>
### UD-03-040a — 取消搜索后恢复原component/window

- **来源 PR / 完整提交**: `741586a4d639321ef45f8dd3eb22561e2a514789`。PR和最早发布/full包含见§6/§2；小写子记录来源不超出03区间。
- **行为 / 源码边界**: FunctionSearchController取消搜索，在anchor window激活后恢复仍showing/focusable的previousFocus；否则合法anchor，最后mainPanel。owner销毁不夺焦点。不同于049的game-info打开/返回流程。
- **对等项 / 意图**: 040同一search successor；APP-05registry与workspace基础；`T03-NAV-SEARCH-CANCEL-FOCUS`。
- **现状 / 处置 / kind**: 未将本新目标记Covered；需后继功能/修复；kind=`feature`。
- **实际阻塞**: 040search与047native activation/target合同；原生platform focus门。
- **有界可观察断言 / stop**: 从非默认component打开并取消搜索，原可用component重新收到键盘；target/window销毁时安全回退，不触发任何搜索动作/dirty变化，迟到activation不抢新owner；Robot/CI部分仍EXC-006。
- **唯一批次 / 责任**: A；`T03-NAV-SEARCH-CANCEL-FOCUS`；子记录与本体由06去重为一个相关能力，不能当独立ParityID。

<a id="ud-03-043"></a>
### UD-03-043 — 搜索工具栏常驻入口与统一快捷键

- **来源 PR 与提交**:
  - 主来源: `PR #462`
  - 完整 40 位 Commit SHA (5 个):
    - `ba6783cfc89a158248ab7a781800c446faa40081` (next-2026-09-18.2) — Merge pull request #462 from qiyi71w/fix/toolbar-search-entry
    - `6bac01c3507c38b3ab8ae7018ac7cfe27d4bc4d0` (next-2026-09-18.2) — Merge remote-tracking branch 'upstream/main' into fix/toolbar-search-entry
    - `92aa33430479e7906c002d726328298af4d50483` (next-2026-09-18.2) — chore: merge main into search entry cleanup
    - `bcde6ac92e1d0f4b04a0c701a456b1555a6c4d82` (next-2026-09-18.2) — docs(search): align localized entry points
    - `8e7eebcb6a362d9ca9f4ffd168fa1a366d8999e7` (next-2026-09-18.2) — fix(ui): keep function search in the toolbar
- **核心文件出处**: `src/main/java/featurecat/lizzie/gui/Menu.java`, `README.md`
- **用户可见行为与边界**: 常驻工具栏图标与Ctrl/Command+K同一搜索能力；清理冗余偏好入口不删除检索目标。
- **对等项 / 任务映射**: 040同一search successor；APP-05 / UI-01（Accepted，原chrome/registry）；`T03-UI01-TOOLBAR-SEARCH-ENTRY`。所有历史项仅引用Matrix原范围，最终后继ID由06分配。
- **Tauri 现状 / 证据与残余**: 是入口子变化，不单独创造搜索Capability。
- **处置结论**: 需后继功能/修复。任务 kind=`feature`；不把Source差异等同Next缺陷。
- **功能断言 / 调查停止条件**: 工具栏和快捷键打开同一panel，提示本平台键；关闭/重新打开保持合法focus/可用性，不绕dirty/Match。
- **实际阻塞与未来证据门**: 040搜索能力；无额外安装/发行阻塞。
- **唯一批次建议 / 责任**: A；`T03-UI01-TOOLBAR-SEARCH-ENTRY` 拥有此Delta的结果。07/08冻结具体实施合同，06负责同源去重。

<a id="ud-03-047"></a>
### UD-03-047 — 窗口就绪后精确target键盘导航

- **来源 PR 与提交**:
  - 主来源: `PR #472`
  - 完整 40 位 Commit SHA (2 个):
    - `ec88d370a4fc4dcc5965d6d964090fae05d6ab0e` (next-2026-09-18.2) — Merge pull request #472 from wimi321/fix/settings-navigation-focus
    - `ee08f5e98b4d99ca251904d3754776ad8f17c036` (next-2026-09-18.2) — fix: defer settings navigation until native dialogs can accept focus
- **核心文件出处**: `src/main/java/featurecat/lizzie/gui/ConfigDialog2.java`, `src/main/java/featurecat/lizzie/gui/GameInfoDialog.java`
- **另有导入来源（不重复计能力）**: `9dcc17c35faf756052ba60633aaa23feb004ba3a`。这些源同时保留其内部分组。
- **用户可见行为与边界**: 打开/激活设置窗口后才聚焦指定组件，不替换成首输入；关闭后调用来源可恢复合法focus。
- **对等项 / 任务映射**: PREF-01（Accepted，preference owner）；SGF-13（Accepted，metadata）；040search successor；`T03-UI02-DIALOG-FOCUS-NAV`。所有历史项仅引用Matrix原范围，最终后继ID由06分配。
- **Tauri 现状 / 证据与残余**: Swing激活/EDT技术内部排除；Next exact target focus尚无本场景验收。
- **处置结论**: 需后继功能/修复。任务 kind=`feature`；不把Source差异等同Next缺陷。
- **功能断言 / 调查停止条件**: 由搜索指定非首target打开设置，native激活后键盘输入作用于该target；target不支持时可见理由/安全回退；旧dialog回调不夺取新window焦点。
- **实际阻塞与未来证据门**: 040target catalog和实际dialog可达性；原生平台焦点证据。
- **唯一批次建议 / 责任**: A；`T03-UI02-DIALOG-FOCUS-NAV` 拥有此Delta的结果。07/08冻结具体实施合同，06负责同源去重。

<a id="udx-019"></a>
## UDX-019 — 有界可响应引擎console

当前阶段导航 **R12/R13（逐意图）**（历史来源组 **B**；混合归属以意图索引为准）；条目/后继 **UI-08**。各来源的有界调查/决定仍须先输出结论，不能把族导航视为新增实现或全部Accepted。明确的其他阶段消费者按其任务合同单独交付；其存在不阻塞独立本地工作。

<a id="ud-03-048"></a>
### UD-03-048 — 可响应 console 输出与载入提示

- **来源 PR 与提交**:
  - 主来源: `PR #473`
  - 完整 40 位 Commit SHA (3 个):
    - `5805c5b35e37c1d80794239388ca9b22aeb02b51` (next-2026-09-18.2) — Merge pull request #473 from wimi321/fix/console-edt-rendering
    - `8dec1f98416c7796a8d49fa4a068ba4f171e7c4a` (next-2026-09-18.2) — Merge remote-tracking branch 'origin/main' into fix/console-edt-rendering
    - `2a902ae61bccb506499af728c65fecb678300159` (next-2026-09-18.2) — fix(console): render engine output and loading comments on the EDT
- **核心文件出处**: `src/main/java/featurecat/lizzie/gui/GtpConsolePane.java`, `src/main/java/featurecat/lizzie/gui/GtpConsoleUpdatePump.java`
- **另有导入来源（不重复计能力）**: `e5bb35c13307b37b97f409d09855aef6d4c34a73`。这些源同时保留其内部分组。
- **用户可见行为与边界**: 用户可见高频engineoutput及loading提示应有界刷新、保持顺序/可读性/操作响应；GtpConsoleUpdatePump/EDT技术不迁移。
- **对等项 / 任务映射**: 具名 console-output successor；REL-09必要运行诊断相关；`T03-CONSOLE-OUTPUT-RESPONSIVENESS`。所有历史项仅引用Matrix原范围，最终后继ID由06分配。
- **Tauri 现状 / 证据与残余**: ENG-09仅multi-backend profile/capability；React/debounce不等于已运行console。
- **处置结论**: 需后继功能/修复。任务 kind=`feature`；不把Source差异等同Next缺陷。
- **功能断言 / 调查停止条件**: 密集输出按冻结保留/截断/滚动规则有序可见，loading/running/failed提示可辨；取消/其他UI仍可操作，内存受限；后台事件不修改retiredview。
- **实际阻塞与未来证据门**: 明确console用户面与有界输出策略；T01-DIAGNOSTICS输出脱敏，不新建无限调试UI。
- **唯一批次建议 / 责任**: B；`T03-CONSOLE-OUTPUT-RESPONSIVENESS` 拥有此Delta的结果。07/08冻结具体实施合同，06负责同源去重。

<a id="udx-020"></a>
## UDX-020 — 棋局详情焦点与窗口归属

当前阶段导航 **R11**（历史来源组 **A**；混合归属以意图索引为准）；条目/后继 **SGF-13**。各来源的有界调查/决定仍须先输出结论，不能把族导航视为新增实现或全部Accepted。明确的其他阶段消费者按其任务合同单独交付；其存在不阻塞独立本地工作。

<a id="ud-03-049"></a>
### UD-03-049 — metadata Komi 初始focus与窗口归属

- **来源 PR 与提交**:
  - 主来源: `PR #475`
  - 完整 40 位 Commit SHA (4 个):
    - `c3e088ac3a4389f06c462d55475eae62b74bc524` (next-2026-09-18.2) — Merge pull request #475 from wimi321/fix/game-info-window-owner
    - `09a17018fc4f544833393d87c09e45907b7d9278` (next-2026-09-18.2) — fix: select komi as the initial focus target for navigation
    - `d09da1971cd5a3339f6b4d78408ffebe66f98734` (next-2026-09-18.2) — fix: restore search focus after native window activation
    - `48bd3e56966279f47e8a118449974f59828ff8cc` (next-2026-09-18.2) — fix: attach game info dialog to its real owner
- **核心文件出处**: `src/main/java/featurecat/lizzie/gui/GameInfoDialog.java`, `src/main/java/featurecat/lizzie/gui/FunctionSearchController.java`
- **另有导入来源（不重复计能力）**: `15c575777f2656999e8a2e747c822f7c6f9cb0e0`。这些源同时保留其内部分组。
- **用户可见行为与边界**: game-info真正owner、默认komi输入、返回searchfocus；与040a取消搜索恢复旧focus不是同一次操作。
- **对等项 / 任务映射**: SGF-13（Accepted，metadata editor）；040search successor；`T03-SGF13-GAMEINFO-KOMI-FOCUS`。所有历史项仅引用Matrix原范围，最终后继ID由06分配。
- **Tauri 现状 / 证据与残余**: SGF-02是tree DTO/NodePath，不拥有metadatafocus；原编辑能力不扩张。
- **处置结论**: 需后继功能/修复。任务 kind=`feature`；不把Source差异等同Next缺陷。
- **功能断言 / 调查停止条件**: 搜索打开metadata可直接编辑komi；Cancel无document变动并回合法sourcefocus；owner退出/target销毁后迟到focus不夺新window。
- **实际阻塞与未来证据门**: 040/047focus导航；SGF-13原编辑/验证/Undo不变。
- **唯一批次建议 / 责任**: A；`T03-SGF13-GAMEINFO-KOMI-FOCUS` 拥有此Delta的结果。07/08冻结具体实施合同，06负责同源去重。

<a id="udx-021"></a>
## UDX-021 — 音频失败隔离调查

当前阶段导航 **R11**（历史来源组 **A**；混合归属以意图索引为准）；条目/后继 **REVIEW-08**。各来源的有界调查/决定仍须先输出结论，不能把族导航视为新增实现或全部Accepted。明确的其他阶段消费者按其任务合同单独交付；其存在不阻塞独立本地工作。

<a id="ud-03-050"></a>
### UD-03-050 — 音频失败不阻塞、用户开关不重置

- **来源 PR 与提交**:
  - 主来源: `PR #476`
  - 完整 40 位 Commit SHA (2 个):
    - `e38202c541070d0ca30b6d05b3589333670273ea` (next-2026-09-18.2) — Merge pull request #476 from wimi321/test/native-window-evidence
    - `bda503fead372fd2618ebad5a468d76f8aecae5d` (next-2026-09-18.2) — fix: keep audio device failures nonblocking and preserve sound settings
- **核心文件出处**: `src/main/java/featurecat/lizzie/util/SoundPlayer.java`, `src/main/java/featurecat/lizzie/util/Utils.java`
- **另有导入来源（不重复计能力）**: `baf63ef7f7490f51cbf8b0c7e5bdb0749bff98a7`。这些源同时保留其内部分组。
- **用户可见行为与边界**: 音频设备/播放失败不影响已接受落子、提示非致命，用户soundEnabled不被失败改写；Java设备捕获方式内部排除。
- **对等项 / 任务映射**: REVIEW-08 / PREF-01（Accepted）；具名 audio-boundary investigation；`T03-REVIEW08-AUDIO-BOUNDARY-INVESTIGATION`。所有历史项仅引用Matrix原范围，最终后继ID由06分配。
- **Tauri 现状 / 证据与残余**: E50已证once-only/silence、缺资源失败与开关持久化；基线App拒绝play promise不回滚棋谱。不把无声卡或所有倒计时路径自动Covered。
- **额外来源 / 继承条件**: E50原cc937c9d2bda0f0d7af513679328ebac774affb1与bc28fc79ea27251997a88f9e000151bf8c1154bf。
- **处置结论**: 需有界调查。任务 kind=`investigation`；不把Source差异等同Next缺陷。
- **功能断言 / 调查停止条件**: 重建E50缺资源/偏好失败和现有asynccatch范围；只对未证真实devicefailure/countdown保留有界问题与native门。停止于已有场景不变+残余表；失败不改soundEnabled、落子状态仍成功。
- **实际阻塞与未来证据门**: 适用真实device/音效consumer；现有REVIEW-08不重新实现。
- **唯一批次建议 / 责任**: A（现有review）；D如对局音效消费者；`T03-REVIEW08-AUDIO-BOUNDARY-INVESTIGATION` 拥有此Delta的结果。07/08冻结具体实施合同，06负责同源去重。

<a id="udx-022"></a>
## UDX-022 — 同树用户重点分析

当前阶段导航 **R12/R13（逐意图）**（历史来源组 **B**；混合归属以意图索引为准）；条目/后继 **ANA-17**。各来源的有界调查/决定仍须先输出结论，不能把族导航视为新增实现或全部Accepted。明确的其他阶段消费者按其任务合同单独交付；其存在不阻塞独立本地工作。

<a id="ud-03-054"></a>
### UD-03-054 — 同树用户 add/remove/clear 重点分析

- **来源 PR 与提交**:
  - 主来源: `PR #449` (包含被集成 PR: #414)
  - 完整 40 位 Commit SHA (18 个):
    - `076cc4ad127609231645d614cdeb61881bd6e458` (next-2026-09-18.2) — Merge pull request #449 from qiyi71w/feat/414-move-focus
    - `e23eddfddf94af2aa7fed1f985727c22ef4f9715` (next-2026-09-18.2) — Merge remote-tracking branch 'origin/main' into fix/focus-probe-resume
    - `1cfc050f25aaee6aa03c30004b523e27d273e681` (next-2026-09-18.2) — Merge remote-tracking branch 'origin/main' into fix/focus-probe-resume
    - `d3833d60204d540cc5b523be7c934b312114f77d` (next-2026-09-18.2) — Merge remote-tracking branch 'origin/main' into fix/focus-probe-resume
    - `7e72e32fd6f76ce0b7fef1e63304ffc45e6a20c0` (next-2026-09-18.2) — test: verify same-tree focus in a native window with the pinned engine
    - `bd228cccecc0f53448bf5f5da0377d3c1ea833e3` (next-2026-09-18.2) — test: compare native settings navigation with active catalog
    - `94c0f4901ec8269898b123c29308707a60d4d7bc` (next-2026-09-18.2) — Merge main into same-tree focus and preserve rule synchronization
    - `7865da5c5d1b004690e3cab1e1a555090e353e89` (next-2026-09-18.2) — fix: dispatch UI capability probes off the event thread
    - `8cb4153fd12850ac3068f6118353dc25a8951115` (next-2026-09-18.2) — fix: serialize deferred focus probes without blocking pause
    - `b56540aaa5a1750209cbcd0939d08f8c6fbe68dd` (next-2026-09-18.2) — fix: retry focus capability detection after pause or board changes
    - `e0dd75541994724ce1a18e6a3c25dae36f32f1f8` (next-2026-09-18.2) — test(engine): serialize late rollback acknowledgements and timeout checks
    - `ae535babca34b7a3a35931c7a35731e6b840c9dd` (next-2026-09-18.2) — test(engine): deliver rollback responses on an independent reader
    - `19b6d9877fa2b0e7c55aeb646f06b1c7836a2c42` (next-2026-09-18.2) — Merge main into move-focus integration and preserve user pause intent
    - `25a984dd6f4ed7282092970d32f087da5847ad57` (next-2026-09-18.2) — fix(i18n): simplify point evaluation menu labels
    - `deb208d6f45f020c9b30b6033c780e213bd70347` (next-2026-09-18.2) — feat(analysis): add same-tree move focus
    - `7451ddc1dddbf4650be221da2fe303bccb9e4f33` (next-2026-09-18.2) — Merge commit '5a33cf560d2563a94e537f7501d088030e3adcfd' into feat/414-move-focus
    - `01c259083b30b3e721242998fd9c582b8b11de1e` (next-2026-09-18.2) — fix(analysis): preserve exact KataGo root metrics
    - `5a33cf560d2563a94e537f7501d088030e3adcfd` (next-2026-09-18.2) — fix(readboard): restore evaluation after navigation
- **核心文件出处**: `src/main/java/featurecat/lizzie/analysis/Leelaz.java`, `src/main/java/featurecat/lizzie/analysis/KataGoAnalysisPayload.java`, `docs/TRACKING_ANALYSIS_CONTRACT.md`, `docs/TRACKING_ANALYSIS_DEVELOPER_GUIDE.md`
- **用户可见行为与边界**: 当前普通本地直连KataGo树的用户关注点研究；不是ANA-16整盘swing-selected。完整契约分054a–054f记录，保留原18source与exactroot修复。
- **对等项 / 任务映射**: 具名 same-tree-focus successor；T02-TRACKING；ANA-16历史不变；`T03-ANALYSIS-SAME-TREE-MOVE-FOCUS`。所有历史项仅引用Matrix原范围，最终后继ID由06分配。
- **Tauri 现状 / 证据与残余**: 基线ANA-16的overview/deep/swing已Accepted(E16)，不证明当前局面focus；新准入/缓存/ReadBoard/显示需独立实现/证据。
- **额外来源 / 继承条件**: 076cc4ad@docs/TRACKING_ANALYSIS_CONTRACT.md；local://issue18-focus-contract.txt全文1–103。
- **处置结论**: 需后继功能/修复。任务 kind=`feature`；不把Source差异等同Next缺陷。
- **功能断言 / 调查停止条件**: 054a–f逐项满足；真实兼容binary证明same-tree preservation且保留final-fence/platform门。无上游编译/正式package前置，无10%或>500visits要求。
- **实际阻塞与未来证据门**: focus-capable二进制的真实能力证据、ordinary owner/结果slot及T01-READBOARD；各子目标明确前置见下。
- **唯一批次建议 / 责任**: B（ReadBoard部分C）；`T03-ANALYSIS-SAME-TREE-MOVE-FOCUS` 拥有此Delta的结果。07/08冻结具体实施合同，06负责同源去重。

<a id="ud-03-054a"></a>
### UD-03-054a — ADMISSION

- **来源 PR / 完整提交**: `deb208d6f45f020c9b30b6033c780e213bd70347`, `7865da5c5d1b004690e3cab1e1a555090e353e89`, `8cb4153fd12850ac3068f6118353dc25a8951115`, `b56540aaa5a1750209cbcd0939d08f8c6fbe68dd`。PR和最早发布/full包含见§6/§2；小写子记录来源不超出03区间。
- **行为 / 源码边界**: 普通local直连当前engine incarnation受控probe；位置confirmed、idleowner、ordinary ponder前合法pass/probability0/rootInfo analyze，消费startup及完整stop/final才SUPPORTED。list_commands/version不证明支持；语法拒绝UNSUPPORTED但普通分析可用；通信不确定按binding失败、不能缓存不支持。旧probe不改变新binding或pause。remote/SSH/WebSocket、双engine、Match/GMA/webtrial/foreground-exclusive、unrestored position拒绝新focus。无旧allow回退。
- **对等项 / 意图**: 054同一same-tree-focus successor；不扩ANA-16；`T03-FOCUS-ADMISSION`。
- **现状 / 处置 / kind**: 未将本新目标记Covered；需后继功能/修复；kind=`feature`。
- **实际阻塞**: 受控probe协议和兼容binary真实支持证据；ordinary owner确认
- **有界可观察断言 / stop**: 替换Run/暂停时迟到probe不启ponder/改变新capability；unsupported仅禁用focus并说明兼容完整engine包，external资源不自动覆盖；active stream不被探针打断。
- **唯一批次 / 责任**: B；`T03-FOCUS-ADMISSION`；子记录与本体由06去重为一个相关能力，不能当独立ParityID。

<a id="ud-03-054b"></a>
### UD-03-054b — USER-SET-PROGRESS

- **来源 PR / 完整提交**: `deb208d6f45f020c9b30b6033c780e213bd70347`。PR和最早发布/full包含见§6/§2；小写子记录来源不超出03区间。
- **行为 / 源码边界**: 唯一controller拥有用户关注集合、active需增益集合、每点累计N visits、high-water/8秒no-progress与immutable display；多点等权总probability0.5。只当前合法未裁剪ordinarypayload可证明达标；strict该点增长才续8秒，other/root增长不续。达标仅退出active且保留关注/圈，不自动再加。remove/clear即时取消关注并安全撤运行增益，不回滚已采纳结果；请求取消≠确认撤去。pause/限额/timeout/error/context退休不重试；落子/离开/换谱/换engine清意图返回不复活。
- **对等项 / 意图**: 054同一same-tree-focus successor；不扩ANA-16；`T03-FOCUS-USER-SET-PROGRESS`。
- **现状 / 处置 / kind**: 未将本新目标记Covered；需后继功能/修复；kind=`feature`。
- **实际阻塞**: 054a支持与ordinarybudget/owner合同
- **有界可观察断言 / stop**: add/remove/clear、多个同时达标合并、无progresstimeout、失败取消分别可观察；blocked/illegal点不标完成，撤去在途不显示confirmed；累计目标读当前树不借旧SGF。
- **唯一批次 / 责任**: B；`T03-FOCUS-USER-SET-PROGRESS`；子记录与本体由06去重为一个相关能力，不能当独立ParityID。

<a id="ud-03-054c"></a>
### UD-03-054c — TREE-CACHE-SGF

- **来源 PR / 完整提交**: `deb208d6f45f020c9b30b6033c780e213bd70347`, `01c259083b30b3e721242998fd9c582b8b11de1e`。PR和最早发布/full包含见§6/§2；小写子记录来源不超出03区间。
- **行为 / 源码边界**: focus只重发ordinary analyze并保持position/turn/allowavoid/非focus输出参数，不clear/loadSGF/replay/改允许集合或重置普通start/total预算。先旧response结束再绑定新输出，未写出的queued旧更新不可复活removedpoint。parser和publication复验incarnation/outputowner/lineage/boardrevision/displaynode/slot，要求exactroot而非candidatevisits求和。合法focus第一完整payload整槽接管新root1000可替旧root10000；不拼两树。首份前reject/remove/pause/context变保留旧cache。主副slot独立，same-tree map保留、真实新树ownership清理。SGF和display同一普通nodevalue，不保存/恢复focusintent。
- **对等项 / 意图**: 054同一same-tree-focus successor；不扩ANA-16；`T03-FOCUS-TREE-CACHE-SGF`。
- **现状 / 处置 / kind**: 未将本新目标记Covered；需后继功能/修复；kind=`feature`。
- **实际阻塞**: 054a支持，ordinary response/final fence、exactroot与node/slot结果owner；真实兼容engine保树证据
- **有界可观察断言 / stop**: 受控真实writer/parser/publication观察无clear/replay、budget不重置；旧queue/update/reader不发布；firstcomplete整槽新树采用、旧值失效前保留、same-tree maps有效；Save/reopen普通root/order/edge结果保留而focus集合/目标/timer不复活。
- **唯一批次 / 责任**: B；`T03-FOCUS-TREE-CACHE-SGF`；子记录与本体由06去重为一个相关能力，不能当独立ParityID。

<a id="ud-03-054d"></a>
### UD-03-054d — READBOARD

- **来源 PR / 完整提交**: `deb208d6f45f020c9b30b6033c780e213bd70347`, `5a33cf560d2563a94e537f7501d088030e3adcfd`。PR和最早发布/full包含见§6/§2；小写子记录来源不超出03区间。
- **行为 / 源码边界**: accepted remote evidence、新requestadmission、已有requestvalidity分开。FRAME_PENDING/纯SYNCING关闭新admission但保留最后accepted semanticfocus；相同完整帧不清/不重发。坏帧/Stop/helper退休/真实语义变化retire。导航离开清关注但保留可重验remote evidence；返回完整重验board/history/node/turn/size/ruleskomi/revision/confirmed incarnation，无新helperframe仍可新add。epoch隔离lateaccept，LF/CRLF保持、非法cell整帧拒绝，adapter失效不是userclear。
- **对等项 / 意图**: 054同一same-tree-focus successor；不扩ANA-16；`T03-FOCUS-READBOARD`。
- **现状 / 处置 / kind**: 未将本新目标记Covered；需后继功能/修复；kind=`feature`。
- **实际阻塞**: 054a–c、READ-01/02准入真实helper/平台功能证据，T01-READBOARD；C部分不阻独立localfocus
- **有界可观察断言 / stop**: 相同frame/导航来回/pending/newframe/badframe/stop和lateepoch分别观察；离开旧focus不复活，返回可新建但完整revalidate失败拒绝；不实现外部playback来替代只读evidence。
- **唯一批次 / 责任**: C（ReadBoard）；`T03-FOCUS-READBOARD`；子记录与本体由06去重为一个相关能力，不能当独立ParityID。

<a id="ud-03-054e"></a>
### UD-03-054e — DISPLAY-SETTINGS

- **来源 PR / 完整提交**: `deb208d6f45f020c9b30b6033c780e213bd70347`, `25a984dd6f4ed7282092970d32f087da5847ad57`。PR和最早发布/full包含见§6/§2；小写子记录来源不超出03区间。
- **行为 / 源码边界**: 候选本体/文字/填充/字体/order复用ordinary绘制，无第二结果circle；关注qualityoutline按同payload该候选与order0的MoveRank loss，缺候选/基准中性，不代表还在计算；completed/samepositionpause保留圈，remove清圈；仍关注豁免数目/低比例filter包括completed和outlineoff，remove恢复filter不删analysis。show-tracking-point-outline只圈，开回不发focus；保留maxvisits/outlineopacity与旧positivevisits迁入，删独立resultfill/text设置；全部维护语言keys/order/placeholders一致，提示会更新普通分析。
- **对等项 / 意图**: 054同一same-tree-focus successor；不扩ANA-16；`T03-FOCUS-DISPLAY-SETTINGS`。
- **现状 / 处置 / kind**: 未将本新目标记Covered；需后继功能/修复；kind=`feature`。
- **实际阻塞**: 054b/c关注与ordinaryrenderer合同；T02-GRANULAR-DISPLAY关联不合并成同能力，native绘制证据
- **有界可观察断言 / stop**: 完成/pause/clear、outlineoff/on、filter与missingbaseline场景使用同ordinaryslot正确绘制；toggle不重算/复活；有效旧visits迁入且obsolete设置不读写；各语言操作/说明一致。
- **唯一批次 / 责任**: B；`T03-FOCUS-DISPLAY-SETTINGS`；子记录与本体由06去重为一个相关能力，不能当独立ParityID。

<a id="ud-03-054f"></a>
### UD-03-054f — COMPATIBLE-ENGINE-EVIDENCE

- **来源 PR / 完整提交**: `7e72e32fd6f76ce0b7fef1e63304ffc45e6a20c0`, `e0dd75541994724ce1a18e6a3c25dae36f32f1f8`, `ae535babca34b7a3a35931c7a35731e6b840c9dd`。PR和最早发布/full包含见§6/§2；小写子记录来源不超出03区间。
- **行为 / 源码边界**: source T1用production add/remove/clear、queue/writer/response/parser/publication；T2真实ReadBoard帧/navigation；T3 ordinarypayload/renderer/SGFparseadoptsave。controlledtransport只替进程，不伪造准入/cache/owner。source pinned engine upstream47aadc08518b3e121f22539796c911002f699584为原engine事实，不是Next自动验收。
- **对等项 / 意图**: 054同一same-tree-focus successor；不扩ANA-16；`T03-FOCUS-COMPATIBLE-ENGINE-EVIDENCE`。
- **现状 / 处置 / kind**: 未将本新目标记Covered；未来功能所需证据门；kind=`evidence-gate`。
- **实际阻塞**: 054a–e实现和合格兼容二进制；此kind为evidence-gate，禁止标新功能全通过
- **有界可观察断言 / stop**: 未来owner记录exact Nextcandidate、binarysource/version/hash/platform和真实same-tree结果；受控自动化/native绘制/真实engine分列，#445finalfence及未有GPU项目各自pending。没有新编译或release前置，旧实验engine结果不顶替最终兼容binary。
- **唯一批次 / 责任**: B；`T03-FOCUS-COMPATIBLE-ENGINE-EVIDENCE`；子记录与本体由06去重为一个相关能力，不能当独立ParityID。

<a id="udx-023"></a>
## UDX-023 — 上游内部mechanics与发布来源排除

当前阶段导航 **R18**（历史来源组 **旧R11发行**；混合归属以意图索引为准）；条目/后继 **—**。各来源的有界调查/决定仍须先输出结论，不能把族导航视为新增实现或全部Accepted。明确的其他阶段消费者按其任务合同单独交付；其存在不阻塞独立本地工作。

<a id="ud-03-exc-001"></a>
### UD-03-EXC-001 — README维护措辞

- **kind / 处置**: `internal-exclusion`；仅Java/上游内部mechanics不适用（不排除下面具名用户子记录）。
- **成员源数**: 1（源清单保留；能力不按此计数）。
- **源码依据 / 保留边界**: 仅README与贡献维护文档不改变桌面runtime；不直接导入Java措辞属于仓库维护mechanics。没有逐项用户goal删除approval，故不称approved replacement。
- **实际阻塞**: 无客户端实现；若拟永久缩减用户goal需具名decision与逐项approval，未批准保留。
- **排除停止条件**: 文件/body中技术部分可定位，production/public输出全部已有用户映射；父冷读确认，不宣称已跑原CI/原nativegate。
- **完整 40 位 Commit SHA 清单**:
  - `5df03ac20a4112b51aa375de57cc6b0eeddf4159` (next-2026-08-30.1) — docs(readme): streamline multilingual project guides (#386)

<a id="ud-03-exc-002"></a>
### UD-03-EXC-002 — CI职责与维护脚本

- **kind / 处置**: `internal-exclusion`；仅Java/上游内部mechanics不适用（不排除下面具名用户子记录）。
- **成员源数**: 14（源清单保留；能力不按此计数）。
- **源码依据 / 保留边界**: 这些manifest列出的workflow/localCI/文档gate是Java开发基础设施，不迁移其作业结构或Maven脚本。gate要求不证明产品功能pass，不替代source关联的userbehavior。
- **实际阻塞**: 无客户端实现；若拟永久缩减用户goal需具名decision与逐项approval，未批准保留。
- **排除停止条件**: 文件/body中技术部分可定位，production/public输出全部已有用户映射；父冷读确认，不宣称已跑原CI/原nativegate。
- **完整 40 位 Commit SHA 清单**:
  - `4fd12153edf6da20174b9fa1d8e62bffbab19bb8` (next-2026-08-31.1) — ci: share local and hosted preflight gates (#394)
  - `062a8cf461bc2adf3e91112e542c8feb7ed3e463` (next-2026-09-13.2) — Merge pull request #454 from qiyi71w/plan/ci-responsibility-split
  - `1a075a9bbd116d08ce443c02aac6e67d36254080` (next-2026-09-13.2) — docs(qa): use public candidate build instructions
  - `73ff458877cfab79a2f15107c821c536a7ac0333` (next-2026-09-13.2) — docs(qa): define specialized acceptance gates
  - `c1408902f27cd1d41bf543a4af05e9161991ee1e` (next-2026-09-13.2) — ci: split checks into responsibility groups
  - `e3b6d918e203dfb97312013191bc8e3364886680` (next-2026-09-13.2) — Merge pull request #455 from qiyi71w/plan/ci-responsibility-split
  - `6caf46b26f7486e6ffb63ffe20c384032ac31e36` (next-2026-09-13.2) — chore: sync main for CI gate cleanup
  - `8dfe02b6f67159ee5f3805ab84067801e604063c` (next-2026-09-13.2) — ci: remove legacy aggregate gates
  - `75736abf7d4726701c34590dac59f7a3b06b7a04` (next-2026-09-18.2) — Merge pull request #459 from qiyi71w/ci/windows-stall-evidence
  - `a1dcc8c992ed2691924bad747cb06bad98d89acb` (next-2026-09-18.2) — ci(windows): capture stalled test diagnostics
  - `714a31d18ddd173f0fbd1069c2dcdc5fa70bb0d3` (next-2026-09-18.2) — ci: require execution proof and desktop smoke (#464)
  - `5f7b18cb3f404786831acffe39983f32156a9ef2` (next-2026-09-18.2) — docs(ci): restore desktop probe diagnostics (#467)
  - `5c048712ea51c24a8bdc012759fc7fa4b291a1ff` (next-2026-09-18.2) — ci(engine): require native process gate
  - `ba4d8b8db11b5e40ac5f9ae7c6890d3a35c96372` (next-2026-09-18.2) — ci: exercise desktop focus under a real X11 window manager

<a id="ud-03-exc-003"></a>
### UD-03-EXC-003 — fixture修复及混合导入

- **kind / 处置**: `internal-exclusion`；仅Java/上游内部mechanics不适用（不排除下面具名用户子记录）。
- **成员源数**: 14（源清单保留；能力不按此计数）。
- **源码依据 / 保留边界**: JUnit/符号链接/WebSocket fixture等待与teardown修复不迁移；integration/test branch含production导入时不把整SHA视为纯test。e35b01c14e2fb423f09de05311bb68dd1f6d1c1c 的KataGoAssetCatalog导入链接035a，仍仅计同一资源trust能力。
- **实际阻塞**: 无客户端实现；若拟永久缩减用户goal需具名decision与逐项approval，未批准保留。
- **排除停止条件**: 文件/body中技术部分可定位，production/public输出全部已有用户映射；父冷读确认，不宣称已跑原CI/原nativegate。
- **完整 40 位 Commit SHA 清单**:
  - `b2227aaa086f8fda797646bc9bb23c71e0a384a7` (next-2026-08-31.1) — test(logging): stabilize crash persistence barrier coverage (#393)
  - `3dea2ffbabdd5735fe499098296672853c0a0a33` (next-2026-08-31.1) — test(windows): handle unavailable symbolic links (#395)
  - `e5ea09bc633c2312af86382f5904389d5daac359` (next-2026-09-13.2) — Merge pull request #441 from wimi321/test/localized-sgf-rule-markers
  - `d92ede3ab9e36c3ed130064f556db43f56f872be` (next-2026-09-13.2) — test(enginegame): handle localized SGF rule markers
  - `f4df03062f7662ca7979217d5f16915f6fcd7659` (next-2026-09-13.2) — Merge pull request #442 from wimi321/test/startup-cancel-response-gate
  - `a22d252647c2363c7b3b4e0d3fcc3bb1f94b977d` (next-2026-09-13.2) — test(engine): gate startup cancellation deterministically
  - `34b6dbb9a69f59f5493d79ab65328f02d8865ab5` (next-2026-09-13.2) — test(engine): await failed startup settlement before retry
  - `56268282f3793c1d6fbcfadd921b9ced78ae041f` (next-2026-09-13.2) — test(logging): await durable recovery before publishing pre-release
  - `42898b2a97656219814ce15aeee1ffc98ff46138` (next-2026-09-18.2) — test: await native setting focus instead of a fixed paint delay
  - `a139645c8df72124e7675dc5120790efae8b7bc0` (next-2026-09-18.2) — Merge pull request #474 from wimi321/test/windows-peer-evidence
  - `df2dee4dcfdb011e7b21e858b67f65cfe5c190cc` (next-2026-09-18.2) — test: tolerate bounded Windows evidence-file sharing locks
  - `34986b9359d77fa3b7b1d8c597ab4d742f4d95d5` (next-2026-09-18.2) — Merge pull request #491 from wimi321/test/webboard-owned-cleanup
  - `e35b01c14e2fb423f09de05311bb68dd1f6d1c1c` (next-2026-09-18.2) — Merge remote-tracking branch 'origin/main' into test/webboard-owned-cleanup
  - `60e65e22e06a390f45ba174aa471090b49e4566d` (next-2026-09-18.2) — Stop owned WebSocket server before awaiting fixture clients

<a id="ud-03-exc-004"></a>
### UD-03-EXC-004 — release request/notes维护

- **kind / 处置**: `internal-exclusion`；仅Java/上游内部mechanics不适用（不排除下面具名用户子记录）。
- **成员源数**: 10（源清单保留；能力不按此计数）。
- **源码依据 / 保留边界**: request/notes格式和bot触发mechanics内部；不要求迁移发布bot。公开说明关联功能原Delta，版本是否formal/prerelease只能看§2API，不将prepare/title算release。
- **实际阻塞**: 无客户端实现；若拟永久缩减用户goal需具名decision与逐项approval，未批准保留。
- **排除停止条件**: 文件/body中技术部分可定位，production/public输出全部已有用户映射；父冷读确认，不宣称已跑原CI/原nativegate。
- **完整 40 位 Commit SHA 清单**:
  - `aa77b7f437aab9603baf2a066d7614027ed3336a` (next-2026-08-30.1) — Merge pull request #389 from wimi321/release/next-2026-08-30.1
  - `7dad1f1525a1c33a3709460bbd38ad9b98566cf6` (next-2026-08-30.1) — release: request next-2026-08-30.1 pre-release
  - `f533b9109195a8f610276b1179dfa460979c0ed1` (next-2026-08-31.1) — release: request next-2026-08-31.1 pre-release (#398)
  - `fbac79e5ef839f656bafc70ded3cab119842d141` (next-2026-08-31.2) — release: request next-2026-08-31.2 pre-release (#400)
  - `cecbc4328b7f4cce2b47a77910949f64c7df9ac5` (next-2026-09-01.1) — release: request next-2026-09-01.1 pre-release (#404)
  - `de4855ea5902d4871207c42ba2e2e52cb465d9ff` (next-2026-09-03.1) — 发布 next-2026-09-03.1 多平台预发布版 (#417)
  - `e23cf300ae65ce0729c0fc589c52492c89961b6c` (next-2026-09-04.1) — 发布 next-2026-09-04.1 预览版 (#419)
  - `e8d0dc9434d03b1bc3c20ee67a5fa8e4c9fda42c` (next-2026-09-13.2) — Merge pull request #456 from wimi321/release/next-2026-09-13.1
  - `441cf0357848ceedae4666afeb283e872b99d71d` (next-2026-09-13.2) — release: prepare next-2026-09-13.1 pre-release
  - `c1857c2ea017ef446105c6446ea1e555978c912c` (next-2026-09-13.2) — Merge pull request #457 from wimi321/release/next-2026-09-13.2

<a id="ud-03-exc-005"></a>
### UD-03-EXC-005 — 发布script内部与公开输出分开

- **kind / 处置**: `internal-exclusion`；仅Java/上游内部mechanics不适用（不排除下面具名用户子记录）。
- **成员源数**: 2（源清单保留；能力不按此计数）。
- **源码依据 / 保留边界**: scripts/r2_release.py发布目录/脚本技术内部排除；其catalog_label/render_index publicGPUguide保留UD010a，1f37094360690ac59eefb038067beeaf38d90d84与2f00081d1cff16a3625a4b1c44078b375bf28fdf同时映射。
- **实际阻塞**: 无客户端实现；若拟永久缩减用户goal需具名decision与逐项approval，未批准保留。
- **排除停止条件**: 文件/body中技术部分可定位，production/public输出全部已有用户映射；父冷读确认，不宣称已跑原CI/原nativegate。
- **完整 40 位 Commit SHA 清单**:
  - `2f00081d1cff16a3625a4b1c44078b375bf28fdf` (next-2026-09-13.2) — Merge pull request #420 from wimi321/fix/stable-tensorrt-catalog-labels
  - `1f37094360690ac59eefb038067beeaf38d90d84` (next-2026-09-13.2) — Correct TensorRT guidance in stable catalog

<a id="ud-03-exc-006"></a>
### UD-03-EXC-006 — Robot/runner内部与productionfocus/Stop分开

- **kind / 处置**: `internal-exclusion`；仅Java/上游内部mechanics不适用（不排除下面具名用户子记录）。
- **成员源数**: 20（源清单保留；能力不按此计数）。
- **源码依据 / 保留边界**: Robot/probe/localrunner/CI receipt与发布artifact QA内部；741586a4d639321ef45f8dd3eb22561e2a514789 FunctionSearchController取消恢复焦点链接040a；2dff9fd2b7577d4727bf2304938c1cb257a0fd68导入Leelaz退休状态链接045，不复制Robot或误称整组无runtime。
- **实际阻塞**: 无客户端实现；若拟永久缩减用户goal需具名decision与逐项approval，未批准保留。
- **排除停止条件**: 文件/body中技术部分可定位，production/public输出全部已有用户映射；父冷读确认，不宣称已跑原CI/原nativegate。
- **完整 40 位 Commit SHA 清单**:
  - `741586a4d639321ef45f8dd3eb22561e2a514789` (next-2026-09-18.2) — ci(desktop): require real search input evidence (#465)
  - `2698994a5b31c58c82629362223655699e9d1286` (next-2026-09-18.2) — test(engine): require process lifecycle smoke (#466)
  - `7173e05f01d292623c18ee30cf88a4a7c50bd9a3` (next-2026-09-18.2) — test(desktop): add local acceptance runner (#468)
  - `2dff9fd2b7577d4727bf2304938c1cb257a0fd68` (next-2026-09-18.2) — Merge pull request #469 from qiyi71w/plan/ci-engine-failure
  - `cb3310cb98e98879191a6820bffe321674ab0919` (next-2026-09-18.2) — test(engine): cover process failure recovery
  - `c421f5b0f77d8d530fd0f03a6f75452897971a38` (next-2026-09-18.2) — test: synchronize native board focus before keyboard setup
  - `c327cf1f5c628996ded0e7b85fbf1ca63c8a0fd1` (next-2026-09-18.2) — test: require all five native navigation repetitions in CI receipts
  - `12b2895e85af6097c6d5e137acd7eb7b9051fd26` (next-2026-09-18.2) — test: retain unexpected modal sources during repeated native navigation
  - `62ea06de304226db5993b8040ad88964539b44ff` (next-2026-09-18.2) — Merge pull request #483 from qiyi71w/integrate/remaining-acceptance
  - `4d63b53c3a7ceec1f84c1e8daa74db29dafef0ee` (next-2026-09-18.2) — Merge remote-tracking branch 'origin/main' into review/acceptance-483
  - `eda99b488e5a5c6e82a9d2cf7b971316df13522a` (next-2026-09-18.2) — test: distinguish bounded CPU cold inference from missing analysis
  - `f1c0515d78a1e8885f5d0388bfb7c619bad35068` (next-2026-09-18.2) — test: isolate WebSocket fixtures and retain CPU failure traces
  - `1d11b7ea217df6ced5b145fc1ec295e62294dd29` (next-2026-09-18.2) — test: execute acceptance helpers on Windows and Linux UI gates
  - `7b5f027b8d0809fa20f307994f693e0c9fe7e68a` (next-2026-09-18.2) — Merge remote-tracking branch 'origin/main' into review/acceptance-483
  - `52902e341effd32bc299cc04d5ac3d30aba4c4bd` (next-2026-09-18.2) — test: enforce acceptance download size and cross-platform gates
  - `5e0496c837fb97f741591dd9ed0a07e684a03816` (next-2026-09-18.2) — fix(acceptance): bound CPU asset downloads
  - `564545739af3073a58079f4cc02af27947755836` (next-2026-09-18.2) — test(acceptance): integrate SGF and CPU gates
  - `a83a1785d5c1821ceac28d89ca84e2d95e2cea1a` (next-2026-09-18.2) — test(gui): add SGF UI acceptance coverage
  - `bb1ca7832d57465acb895bc213132eb86cb7a017` (next-2026-09-18.2) — test(gui): add real CPU engine acceptance
  - `0dbe0b0130c54c0ce659af7abbcd4417a36da51f` (next-2026-09-18.2) — feat(release): verify final artifact identity

<a id="ud-03-exc-007"></a>
### UD-03-EXC-007 — engine compile/packaging内部与runtime/import分开

- **kind / 处置**: `internal-exclusion`；仅Java/上游内部mechanics不适用（不排除下面具名用户子记录）。
- **成员源数**: 59（源清单保留；能力不按此计数）。
- **源码依据 / 保留边界**: macOSlinker/WindowsCPU/OpenCL/DirectML/CUDA/TRT/LinuxABI/SDK封印、编译/依赖打包机械不迁移；不执行compile/release是审计边界而非永久产品替换批准。2061865fce2e9ec4a44633fe86f1dd6bde9112d2与4019a7e15aa4deaf2a1e5b6e49ae691f1f7db74d productiontrust链接035a。混合import源各自链接046/047/048/049/050/051/052/053，不重复建capability；可用兼容二进制、来源/identity/完整性/硬件指导仍是消费者future gate。
- **实际阻塞**: 无客户端实现；若拟永久缩减用户goal需具名decision与逐项approval，未批准保留。
- **排除停止条件**: 文件/body中技术部分可定位，production/public输出全部已有用户映射；父冷读确认，不宣称已跑原CI/原nativegate。
- **完整 40 位 Commit SHA 清单**:
  - `4c86a9ececc0a4cac5d6e3ea475dc9b7c67fcb51` (next-2026-09-18.2) — Merge pull request #471 from wimi321/build/katago-focus-source
  - `baf63ef7f7490f51cbf8b0c7e5bdb0749bff98a7` (next-2026-09-18.2) — Integrate nonblocking sound fix and strict repeated desktop acceptance
  - `15c575777f2656999e8a2e747c822f7c6f9cb0e0` (next-2026-09-18.2) — Merge remote-tracking branch 'origin/main' into build/katago-focus-source
  - `e5bb35c13307b37b97f409d09855aef6d4c34a73` (next-2026-09-18.2) — Merge remote-tracking branch 'origin/main' into build/katago-focus-source
  - `6e1e971f34b6518ab5226124b405d105cf7df2ec` (next-2026-09-18.2) — test: honor explicit cold-start probe deadline and retain timing
  - `9dcc17c35faf756052ba60633aaa23feb004ba3a` (next-2026-09-18.2) — Merge tested native focus fixes into pinned engine builds
  - `ed2d3a0ae617e1a13c23508b5355359addb52d2c` (next-2026-09-18.2) — fix: reserve portable library header space in the Swift linker
  - `65424a44b5e17084132ede8db5fab556c9d33a49` (next-2026-09-18.2) — ci: rerun source build acceptance for KataGo test changes
  - `61cedcf2f46e58e541b00e9fc55372553f90b9f0` (next-2026-09-18.2) — test: keep SDK environment assertions platform independent
  - `91bfa9bd43f893003b1ba58ce94a9b99be38c1a4` (next-2026-09-18.2) — fix: lock macOS source dependencies and audit deployment compatibility
  - `9d05032ecfcb1e63e3a5bf62f3718fc53397af51` (next-2026-09-18.2) — Merge restart confirmation fix into source build validation
  - `3f0eb53208c3ccf40c0fb6d9f8d6e9ff3882d8d8` (next-2026-09-18.2) — build: retain macOS deployment target through Swift linking
  - `4cc507210a1ed12f832096cb50d182cac78f8116` (next-2026-09-18.2) — build: require executable identity and genuine hardware deferrals
  - `bc5df09ec927d258abc26fbb6d379e8e408ee1d8` (next-2026-09-18.2) — Add pinned KataGo source build receipts and real focus protocol probes
  - `a5663b14096a6072688cac84bcbb940229963047` (next-2026-09-18.2) — Merge pull request #479 from wimi321/build/katago-source-windows
  - `4f86f57d3491019060fbe50e0cdcd4a73b9b2228` (next-2026-09-18.2) — fix: normalize Windows environment keys for portable engine checks
  - `35df875894942ab4e8fe2bead43576ce7465d845` (next-2026-09-18.2) — fix: install Eigen headers without unrelated Fortran configuration
  - `e03490000b596e60639da1f61be120b4563cc7d8` (next-2026-09-18.2) — Merge remote-tracking branch 'origin/main' into build/katago-source-windows
  - `bba2bafda72468f7c4d5d5cde3db9434f664a8a4` (next-2026-09-18.2) — build: add locked Windows CPU and OpenCL source evidence
  - `e146e78cc7edb5919a8fc3d7925eb597edfa61c6` (next-2026-09-18.2) — fix: use verified zlib release and preserve static link dependencies
  - `5af5113bffaf8200f3bb3b0d449a4eb8463c195a` (next-2026-09-18.2) — Integrate reviewed source-build foundation from main
  - `c83deb372aaf9d2ec53a0c3d0afaac3cc3e9d4ae` (next-2026-09-18.2) — build: add locked Linux CPU and OpenCL source evidence pipeline
  - `9989c718597188709baee44c576684fd19121ef2` (next-2026-09-18.2) — Merge pull request #480 from wimi321/build/katago-source-directml
  - `eb245718b31fb54b76157251e6e2477120b9e2e4` (next-2026-09-18.2) — build: lock and verify DirectML source-engine dependencies
  - `f7312a9f0ce0baad9bfba6c301e8631c714bb8fb` (next-2026-09-18.2) — Merge pull request #481 from wimi321/build/katago-source-openvino
  - `e1c8bc603d9a8f0c10533bf9e3967ea658fafbef` (next-2026-09-18.2) — docs: clarify source-build target coverage
  - `59ebb4c129f074b778af766442ef7e08112e6d67` (next-2026-09-18.2) — Merge remote-tracking branch 'origin/main' into build/katago-source-openvino
  - `6a996bae16000a6a0bc8afce876ae0c2ade76d62` (next-2026-09-18.2) — build: lock and audit OpenVINO source-engine artifacts
  - `d6646b1541936566c8db43f2cba770e67306ba27` (next-2026-09-18.2) — Merge pull request #484 from wimi321/build/katago-source-cuda
  - `7af73741a020ca12f7ffc51159d718fd1f8d07fb` (next-2026-09-18.2) — build: preserve complete CUDA runtime archive inventory
  - `7c7985ad5d852a12a167086e668ef3ddfeb2c124` (next-2026-09-18.2) — build: add locked Windows CUDA source artifacts
  - `d0f339849481529d94feeb44b2f486ae8a5a17e8` (next-2026-09-18.2) — Merge pull request #485 from wimi321/build/katago-source-linux-cuda-v2
  - `66670b3853ac3721a26c3325f7dc73d4e023437c` (next-2026-09-18.2) — build: bundle sealed zlib required by Linux cuDNN
  - `3b2ec98edb4be85f03b91053734f10c3852b3496` (next-2026-09-18.2) — build: preserve exact Linux runtime audit failure details
  - `b45e65be76b95b8176f7c31b1b45623690e65405` (next-2026-09-18.2) — fix(build): resolve Linux nvcc runtime archive layout
  - `9f5ec287c3297f477149f0ebc4a0271d82a9ca54` (next-2026-09-18.2) — test: keep Linux CUDA audit tests portable
  - `f7589969108770ddae76ab5c435a446ae9415c0a` (next-2026-09-18.2) — build: add pinned Linux CUDA source evidence
  - `8a3f54fd45b343fb08659870e90e1a43c162c3f1` (next-2026-09-18.2) — Merge pull request #486 from wimi321/build/katago-source-tensorrt
  - `d0aff16ec9333d8948052eb8543015e689da9773` (next-2026-09-18.2) — Merge verified Linux CUDA source foundation
  - `15b299c6b1a5cf0c0a67008f5b5bb5addea0612d` (next-2026-09-18.2) — build: seal static protobuf for TensorRT ONNX generation
  - `44ad3474f7ad430096ba23c5c8f41a5d33cebd0f` (next-2026-09-18.2) — build: seal TensorRT 10.9 source SDK and runtime audit
  - `4019a7e15aa4deaf2a1e5b6e49ae691f1f7db74d` (next-2026-09-18.2) — Merge pull request #488 from wimi321/build/katago-source-delivery
  - `37e922aa35a01d8d41019c77fe962cd57e049771` (next-2026-09-18.2) — Verify source archives before installing them into release build trees
  - `a4ab1ff337c7e99ac96409f81f7c7f0c0da24356` (next-2026-09-18.2) — Include pinned configs and reverify source archive contents
  - `2061865fce2e9ec4a44633fe86f1dd6bde9112d2` (next-2026-09-18.2) — Seal audited source engines and share trusted repair download origins
  - `2b6e31da53f3e805338eb3062fc71dd6a7f4f912` (next-2026-09-18.2) — Merge pull request #489 from wimi321/build/katago-linux-compatibility
  - `630866cb25e41b4ac59baab298e8cc49356952c1` (next-2026-09-18.2) — Merge remote-tracking branch 'origin/main' into build/katago-linux-compatibility
  - `59e95d11fc7b17563e0f8b84087e50a428449b3d` (next-2026-09-18.2) — Merge commit '37e922aa' into build/katago-linux-compatibility
  - `280f7ebf78c5c65e948bbcd79dcf7ec0b03bd769` (next-2026-09-18.2) — Supply complete GTP configuration for isolated inference acceptance
  - `0cc9ef8fe43003a67b3995755e6fc34482d50d00` (next-2026-09-18.2) — Merge commit 'a4ab1ff3' into build/katago-linux-compatibility
  - `29b8985a330068b62ab92cddc9ca080c8efa5d78` (next-2026-09-18.2) — Isolate new loader checks from historical baseline dependencies
  - `30dceff6cf0363e4d3025d7e1fdac021dc80e672` (next-2026-09-18.2) — Verify source Linux engines against production ABI and clean distributions
  - `08b6908dfdbed86f83f95e43e454d35c497add26` (next-2026-09-18.2) — Merge pull request #490 from wimi321/build/katago-source-packaging
  - `7c88a25c031ddf633fc37706902d11be9ab10b1b` (next-2026-09-18.2) — Merge remote-tracking branch 'origin/main' into build/katago-source-packaging
  - `c62a0fb778185e8510636f558664c02a2c135b19` (next-2026-09-18.2) — Await asynchronous application exit in native SGF acceptance
  - `3ef5fa3778e44bff360fefc12c286105a5d6f878` (next-2026-09-18.2) — Merge remote-tracking branch 'origin/main' into build/katago-source-packaging
  - `33f0996f4db411910a6db3c5b2f2d58aa2f4a912` (next-2026-09-18.2) — Integrate verified Linux compatibility gates into final packaging
  - `4c474f3a66d473061c0b190b8b92b5f659c35a00` (next-2026-09-18.2) — Verify source-built CPU acceptance archives and exact executable revision
  - `6053dc3401aee9426847ae08e695310917c36046` (next-2026-09-18.2) — Install and audit reviewed source engines in final release packages

<a id="ud-04-003"></a>
### UD-04-003 next-2026-09-17.1准备与来源联接

- **Source / event**: PR #494 (集成事件 3) | 共 4 个提交；第一父E03。
- **完整source commits（与§6反向对应）**:
  - [aa07f6e07b16add08b271e7b3994b731216d5e0d](https://github.com/wimi321/lizzieyzy-next/commit/aa07f6e07b16add08b271e7b3994b731216d5e0d) — Merge pull request #494 from wimi321/release/prerelease-focus-20260917
  - [e8e5171b1ee009af81c1991bad361909bcb29dc9](https://github.com/wimi321/lizzieyzy-next/commit/e8e5171b1ee009af81c1991bad361909bcb29dc9) — Merge remote-tracking branch 'origin/main' into release/prerelease-focus-20260917
  - [c0d15d8cb0cf4f06ade6aaa3718ac3d4e70a871b](https://github.com/wimi321/lizzieyzy-next/commit/c0d15d8cb0cf4f06ade6aaa3718ac3d4e70a871b) — Merge branch 'release/activate-pinned-katago' into release/prerelease-focus-20260917
  - [b318dbf9952701a9c59b7cb50256f5bf1b242e31](https://github.com/wimi321/lizzieyzy-next/commit/b318dbf9952701a9c59b7cb50256f5bf1b242e31) — Prepare next-2026-09-17.1 same-tree analysis prerelease
- **发布包含**: 最早有API/ancestry证据收录 `next-2026-09-18.2`（预发布）；全部source也含于正式09-26.1/.2。准备target仅记prepared，不由本字段推断准备版本已发布。
- **实际behavior / sourceDelta**: 本来源已合并same-tree/focus准备及notes/requests，不证明准备tag已发布。源notes只整合早段与本段既有功能，不重复计新能力。
- **Canonical owner / scope**: 无独立发行功能；来源整合记录。
- **候选/覆盖与边界**: 无Tauri运行继承；notes/requests和Java测试不要求产品复刻。源变化由§6索引保留。
- **Disposition**: Java发布准备内部排除。
- **责任意图**: 无独立产品任务。
- **Observable assertions**: 逐项使用§2 API/ancestry确定发布归属；保持准备与publication区分；无独立feature实现验收。

<a id="ud-04-005"></a>
### UD-04-005 next-2026-09-17.2准备与来源联接

- **Source / event**: PR #496 (集成事件 5) | 共 3 个提交；第一父E05。
- **完整source commits（与§6反向对应）**:
  - [815c4a90040dc9a7412823200a49513f272de0ec](https://github.com/wimi321/lizzieyzy-next/commit/815c4a90040dc9a7412823200a49513f272de0ec) — Merge pull request #496 from wimi321/release/prerelease-focus-20260917-v2
  - [c0b18f0d978126f9091721532e0081256f07fbe4](https://github.com/wimi321/lizzieyzy-next/commit/c0b18f0d978126f9091721532e0081256f07fbe4) — Merge remote-tracking branch 'origin/main' into release/prerelease-focus-20260917-v2
  - [cae8b21fb7d635210f1fc9fe88f667388c40078d](https://github.com/wimi321/lizzieyzy-next/commit/cae8b21fb7d635210f1fc9fe88f667388c40078d) — Prepare audited same-tree analysis prerelease with final handoff fixes
- **发布包含**: 最早有API/ancestry证据收录 `next-2026-09-18.2`（预发布）；全部source也含于正式09-26.1/.2。准备target仅记prepared，不由本字段推断准备版本已发布。
- **实际behavior / sourceDelta**: 本来源已合并terminal handoff准备及notes/requests，不证明准备tag已发布。manifest列出src/main/resources/katago-assets.json；engineReleaseTag等资源定位字段更新必须保留为source linkage，不是notes-only。
- **Canonical owner / scope**: 无独立发行功能；T01-RESOURCE/REL-05功能source linkage。
- **候选/覆盖与边界**: 无Tauri运行继承；notes/requests和Java测试不要求产品复刻。源变化由§6索引保留。
- **Disposition**: 混合：准备/测试内部排除；catalog linkage随资源功能。
- **责任意图**: T04-RESOURCE。
- **Observable assertions**: 逐项使用§2 API/ancestry确定发布归属；资源定位变化须能解析明确来源并在缺失/离线时可见失败，不能因准备URL存在判下载可用或已验收。

<a id="ud-04-007"></a>
### UD-04-007 Windows安装器staging隔离与失败传播

- **Source / event**: PR #498 (集成事件 7) | 共 3 个提交；第一父E07。
- **完整source commits（与§6反向对应）**:
  - [e5788680ea62f0202c639846ae19d700e2c336d4](https://github.com/wimi321/lizzieyzy-next/commit/e5788680ea62f0202c639846ae19d700e2c336d4) — Merge pull request #498 from wimi321/fix/windows-installer-staging
  - [8bdcd39f5bd1e66e18a6a46e7185bdbb617d115a](https://github.com/wimi321/lizzieyzy-next/commit/8bdcd39f5bd1e66e18a6a46e7185bdbb617d115a) — Merge remote-tracking branch 'origin/main' into fix/windows-installer-staging
  - [665f4f232d2f441004af5b4b3a233ad718dca597](https://github.com/wimi321/lizzieyzy-next/commit/665f4f232d2f441004af5b4b3a233ad718dca597) — Keep Windows installer staging clean and propagate packaging failures
- **发布包含**: 最早有API/ancestry证据收录 `next-2026-09-18.2`（预发布）；全部source也含于正式09-26.1/.2。准备target仅记prepared，不由本字段推断准备版本已发布。
- **实际behavior / sourceDelta**: PR498修正安装器暂存、重复引擎副本和打包非零退出传播；没有独立新增桌面用户操作。正式安装资源验证继续保留在发行合同。
- **Canonical owner / scope**: Java安装器内部；后置发行参考。
- **候选/覆盖与边界**: manifest文件/提交来源是静态排除依据，不是Tauri运行证据。
- **Disposition**: 仅Java内部不适用。
- **责任意图**: 无独立产品任务。
- **Observable assertions**: 来源指向安装脚本/CI且不把失败发布成成功；不复制Java packager、不替未来Tauri installer声称已验。

<a id="ud-04-008"></a>
### UD-04-008 next-2026-09-17.3准备与来源联接

- **Source / event**: PR #499 (集成事件 8) | 共 4 个提交；第一父E08。
- **完整source commits（与§6反向对应）**:
  - [eb5d245275cb1d106801d11e62abb617a9ba3ff4](https://github.com/wimi321/lizzieyzy-next/commit/eb5d245275cb1d106801d11e62abb617a9ba3ff4) — Merge pull request #499 from wimi321/release/prerelease-focus-20260917-v3
  - [2585443492345e42436345ca803b0c26bfc58320](https://github.com/wimi321/lizzieyzy-next/commit/2585443492345e42436345ca803b0c26bfc58320) — Separate probe schema timeout from child JVM startup in acceptance
  - [2ebe7397cf7aac3e3dd7ec7d5b681a2c57816648](https://github.com/wimi321/lizzieyzy-next/commit/2ebe7397cf7aac3e3dd7ec7d5b681a2c57816648) — Merge remote-tracking branch 'origin/main' into release/prerelease-focus-20260917-v3
  - [29b923a2e38a69a989a3c0b60634f82f1d659d56](https://github.com/wimi321/lizzieyzy-next/commit/29b923a2e38a69a989a3c0b60634f82f1d659d56) — Prepare validated source-engine prerelease next-2026-09-17.3
- **发布包含**: 最早有API/ancestry证据收录 `next-2026-09-18.2`（预发布）；全部source也含于正式09-26.1/.2。准备target仅记prepared，不由本字段推断准备版本已发布。
- **实际behavior / sourceDelta**: 本来源已合并GTP probe超时与child JVM启动分离及notes/requests，不证明准备tag已发布。manifest列出src/main/resources/katago-assets.json；engineReleaseTag等资源定位字段更新必须保留为source linkage，不是notes-only。
- **Canonical owner / scope**: 无独立发行功能；T01-RESOURCE/REL-05功能source linkage。
- **候选/覆盖与边界**: 无Tauri运行继承；notes/requests和Java测试不要求产品复刻。源变化由§6索引保留。
- **Disposition**: 混合：准备/测试内部排除；catalog linkage随资源功能。
- **责任意图**: T04-RESOURCE。
- **Observable assertions**: 逐项使用§2 API/ancestry确定发布归属；资源定位变化须能解析明确来源并在缺失/离线时可见失败，不能因准备URL存在判下载可用或已验收。

<a id="ud-04-010"></a>
### UD-04-010 next-2026-09-17.4准备与来源联接

- **Source / event**: PR #501 (集成事件 10) | 共 3 个提交；第一父E10。
- **完整source commits（与§6反向对应）**:
  - [c870d9f35d195500caf17d63c4a90203eeae8e4f](https://github.com/wimi321/lizzieyzy-next/commit/c870d9f35d195500caf17d63c4a90203eeae8e4f) — Merge pull request #501 from wimi321/release/prerelease-focus-20260917-v4
  - [4509ace7486e2749d6c425da46f6956272a041e4](https://github.com/wimi321/lizzieyzy-next/commit/4509ace7486e2749d6c425da46f6956272a041e4) — Merge branch 'fix/startup-benchmark-yield-20260917' into release/prerelease-focus-20260917-v4
  - [bf73c6c3614776906f6ac8183cb7bdd4a7805473](https://github.com/wimi321/lizzieyzy-next/commit/bf73c6c3614776906f6ac8183cb7bdd4a7805473) — Prepare next-2026-09-17.4 with verified benchmark handoff
- **发布包含**: 最早有API/ancestry证据收录 `next-2026-09-18.2`（预发布）；全部source也含于正式09-26.1/.2。准备target仅记prepared，不由本字段推断准备版本已发布。
- **实际behavior / sourceDelta**: 本来源已合并startup benchmark交接准备及notes/requests，不证明准备tag已发布。manifest列出src/main/resources/katago-assets.json；engineReleaseTag等资源定位字段更新必须保留为source linkage，不是notes-only。
- **Canonical owner / scope**: 无独立发行功能；T01-RESOURCE/REL-05功能source linkage。
- **候选/覆盖与边界**: 无Tauri运行继承；notes/requests和Java测试不要求产品复刻。源变化由§6索引保留。
- **Disposition**: 混合：准备/测试内部排除；catalog linkage随资源功能。
- **责任意图**: T04-RESOURCE。
- **Observable assertions**: 逐项使用§2 API/ancestry确定发布归属；资源定位变化须能解析明确来源并在缺失/离线时可见失败，不能因准备URL存在判下载可用或已验收。

<a id="ud-04-014"></a>
### UD-04-014 next-2026-09-17.5准备与来源联接

- **Source / event**: PR #505 (集成事件 14) | 共 3 个提交；第一父E14。
- **完整source commits（与§6反向对应）**:
  - [beb498f5707972f5e6bc7c014a60d8f2baa7b52b](https://github.com/wimi321/lizzieyzy-next/commit/beb498f5707972f5e6bc7c014a60d8f2baa7b52b) — Merge pull request #505 from wimi321/release/prerelease-focus-20260917-v5
  - [a18cd792f47e76db5de7da972da3b9ffac16938d](https://github.com/wimi321/lizzieyzy-next/commit/a18cd792f47e76db5de7da972da3b9ffac16938d) — Merge remote-tracking branch 'origin/main' into release/prerelease-focus-20260917-v5
  - [841f90b5ac65ef65673d79d57e04ab349ff51285](https://github.com/wimi321/lizzieyzy-next/commit/841f90b5ac65ef65673d79d57e04ab349ff51285) — Prepare verified pinned-source prerelease and audit static zlib correctly
- **发布包含**: 最早有API/ancestry证据收录 `next-2026-09-18.2`（预发布）；全部source也含于正式09-26.1/.2。准备target仅记prepared，不由本字段推断准备版本已发布。
- **实际behavior / sourceDelta**: 本来源已合并静态zlib来源审计对齐及notes/requests，不证明准备tag已发布。manifest列出src/main/resources/katago-assets.json；engineReleaseTag等资源定位字段更新必须保留为source linkage，不是notes-only。
- **Canonical owner / scope**: 无独立发行功能；T01-RESOURCE/REL-05功能source linkage。
- **候选/覆盖与边界**: 无Tauri运行继承；notes/requests和Java测试不要求产品复刻。源变化由§6索引保留。
- **Disposition**: 混合：准备/测试内部排除；catalog linkage随资源功能。
- **责任意图**: T04-RESOURCE。
- **Observable assertions**: 逐项使用§2 API/ancestry确定发布归属；资源定位变化须能解析明确来源并在缺失/离线时可见失败，不能因准备URL存在判下载可用或已验收。

<a id="ud-04-016"></a>
### UD-04-016 next-2026-09-17.6准备与来源联接

- **Source / event**: PR #507 (集成事件 16) | 共 3 个提交；第一父E16。
- **完整source commits（与§6反向对应）**:
  - [a99a69ab99656421b9a09148b68021f0700a7042](https://github.com/wimi321/lizzieyzy-next/commit/a99a69ab99656421b9a09148b68021f0700a7042) — Merge pull request #507 from wimi321/release/prerelease-focus-20260917-v6
  - [9a93cb3b70477454cc345817c3e859e4026aa328](https://github.com/wimi321/lizzieyzy-next/commit/9a93cb3b70477454cc345817c3e859e4026aa328) — Merge remote-tracking branch 'origin/main' into release/prerelease-focus-20260917-v6
  - [ecea2cdf9aa3845340e5fb961566b75be63156a8](https://github.com/wimi321/lizzieyzy-next/commit/ecea2cdf9aa3845340e5fb961566b75be63156a8) — Prepare time-budgeted same-tree analysis prerelease candidate
- **发布包含**: 最早有API/ancestry证据收录 `next-2026-09-18.2`（预发布）；全部source也含于正式09-26.1/.2。准备target仅记prepared，不由本字段推断准备版本已发布。
- **实际behavior / sourceDelta**: 本来源已合并HumanSL时间预算准备及notes/requests，不证明准备tag已发布。manifest列出src/main/resources/katago-assets.json；engineReleaseTag等资源定位字段更新必须保留为source linkage，不是notes-only。
- **Canonical owner / scope**: 无独立发行功能；T01-RESOURCE/REL-05功能source linkage。
- **候选/覆盖与边界**: 无Tauri运行继承；notes/requests和Java测试不要求产品复刻。源变化由§6索引保留。
- **Disposition**: 混合：准备/测试内部排除；catalog linkage随资源功能。
- **责任意图**: T04-RESOURCE。
- **Observable assertions**: 逐项使用§2 API/ancestry确定发布归属；资源定位变化须能解析明确来源并在缺失/离线时可见失败，不能因准备URL存在判下载可用或已验收。

<a id="ud-04-018"></a>
### UD-04-018 next-2026-09-17.7准备与来源联接

- **Source / event**: PR #509 (集成事件 18) | 共 3 个提交；第一父E18。
- **完整source commits（与§6反向对应）**:
  - [1054141b0ccc530d68a1c0701cc7745f968afbf2](https://github.com/wimi321/lizzieyzy-next/commit/1054141b0ccc530d68a1c0701cc7745f968afbf2) — Merge pull request #509 from wimi321/release/prerelease-focus-20260917-v7
  - [d01d27172ac9237d36dfda965e243978d29639da](https://github.com/wimi321/lizzieyzy-next/commit/d01d27172ac9237d36dfda965e243978d29639da) — Merge remote-tracking branch 'origin/main' into release/prerelease-focus-20260917-v7
  - [ecd0414ae6aa76d1bd5f0d6c15f039b3d82534ab](https://github.com/wimi321/lizzieyzy-next/commit/ecd0414ae6aa76d1bd5f0d6c15f039b3d82534ab) — Prepare final same-tree prerelease with bounded HumanSL verification
- **发布包含**: 最早有API/ancestry证据收录 `next-2026-09-18.2`（预发布）；全部source也含于正式09-26.1/.2。准备target仅记prepared，不由本字段推断准备版本已发布。
- **实际behavior / sourceDelta**: 本来源已合并HumanSL weightless验证准备及notes/requests，不证明准备tag已发布。manifest列出src/main/resources/katago-assets.json；engineReleaseTag等资源定位字段更新必须保留为source linkage，不是notes-only。
- **Canonical owner / scope**: 无独立发行功能；T01-RESOURCE/REL-05功能source linkage。
- **候选/覆盖与边界**: 无Tauri运行继承；notes/requests和Java测试不要求产品复刻。源变化由§6索引保留。
- **Disposition**: 混合：准备/测试内部排除；catalog linkage随资源功能。
- **责任意图**: T04-RESOURCE。
- **Observable assertions**: 逐项使用§2 API/ancestry确定发布归属；资源定位变化须能解析明确来源并在缺失/离线时可见失败，不能因准备URL存在判下载可用或已验收。

<a id="ud-04-021"></a>
### UD-04-021 next-2026-09-18.1准备与来源联接

- **Source / event**: PR #512 (集成事件 21) | 共 2 个提交；第一父E21。
- **完整source commits（与§6反向对应）**:
  - [d1683e229f84cbe6ffd239f9a4de6fac76b1f2a6](https://github.com/wimi321/lizzieyzy-next/commit/d1683e229f84cbe6ffd239f9a4de6fac76b1f2a6) — Merge pull request #512 from wimi321/release/prerelease-focus-20260918-v1
  - [a39e0b9f65cc74c31ad10a4022d6db10b1f88f09](https://github.com/wimi321/lizzieyzy-next/commit/a39e0b9f65cc74c31ad10a4022d6db10b1f88f09) — Prepare same-tree prerelease with verified synchronization and upload recovery
- **发布包含**: 最早有API/ancestry证据收录 `next-2026-09-18.2`（预发布）；全部source也含于正式09-26.1/.2。准备target仅记prepared，不由本字段推断准备版本已发布。
- **实际behavior / sourceDelta**: 本来源已合并同步与upload恢复准备及notes/requests，不证明准备tag已发布。manifest列出src/main/resources/katago-assets.json；engineReleaseTag等资源定位字段更新必须保留为source linkage，不是notes-only。
- **Canonical owner / scope**: 无独立发行功能；T01-RESOURCE/REL-05功能source linkage。
- **候选/覆盖与边界**: 无Tauri运行继承；notes/requests和Java测试不要求产品复刻。源变化由§6索引保留。
- **Disposition**: 混合：准备/测试内部排除；catalog linkage随资源功能。
- **责任意图**: T04-RESOURCE。
- **Observable assertions**: 逐项使用§2 API/ancestry确定发布归属；资源定位变化须能解析明确来源并在缺失/离线时可见失败，不能因准备URL存在判下载可用或已验收。

<a id="ud-04-023"></a>
### UD-04-023 Windows upload恢复与ReadBoard测试终态等待

- **Source / event**: PR #514 (集成事件 23) | 共 3 个提交；第一父E23。
- **完整source commits（与§6反向对应）**:
  - [55b113e70a5f4d47b080c1a7d6e31d81e08c2c47](https://github.com/wimi321/lizzieyzy-next/commit/55b113e70a5f4d47b080c1a7d6e31d81e08c2c47) — Merge pull request #514 from wimi321/fix/windows-release-transfer-20260918
  - [e37bf9f3e269fbb58ff6eb14ddbeadd049b4d8a6](https://github.com/wimi321/lizzieyzy-next/commit/e37bf9f3e269fbb58ff6eb14ddbeadd049b4d8a6) — Wait for terminal restore workers before disposing test fixtures
  - [6c39d0a3973565704cbdf78c7735430ceece6572](https://github.com/wimi321/lizzieyzy-next/commit/6c39d0a3973565704cbdf78c7735430ceece6572) — Retain verified Windows packages for upload-only recovery
- **发布包含**: 最早有API/ancestry证据收录 `next-2026-09-18.2`（预发布）；全部source也含于正式09-26.1/.2。准备target仅记prepared，不由本字段推断准备版本已发布。
- **实际behavior / sourceDelta**: PR514保留已验证安装包供upload-only重试；e37bf9f3只在ReadBoardEngineResumeTest等待restore worker终态。不是新增生产恢复修复。
- **Canonical owner / scope**: Java release/test机制；READ-02/T01-READBOARD原范围。
- **候选/覆盖与边界**: H-READBOARD指实际Windows/Fox证据，不把Java test改动当它的实现或通过。
- **Disposition**: 内部/test排除；不制造runtime覆盖。
- **责任意图**: 无独立产品任务；T01-READBOARD为原证据责任。
- **Observable assertions**: 源码角色标test；producer/同步目标仍由READ-02原功能合同负责，不因测试等待声明native确认已新增通过。

<a id="ud-04-024"></a>
### UD-04-024 next-2026-09-18.2准备与来源联接

- **Source / event**: PR #515 (集成事件 24) | 共 2 个提交；第一父E24。
- **完整source commits（与§6反向对应）**:
  - [5930a09d979daac41eb0451dcd6fa8ccf5f2db1f](https://github.com/wimi321/lizzieyzy-next/commit/5930a09d979daac41eb0451dcd6fa8ccf5f2db1f) — Merge pull request #515 from wimi321/release/prerelease-focus-20260918-v2
  - [a935ec227ce667a364ff08cf7e0d97ebc4e3cdd4](https://github.com/wimi321/lizzieyzy-next/commit/a935ec227ce667a364ff08cf7e0d97ebc4e3cdd4) — Prepare verified same-tree analysis pre-release with recoverable uploads
- **发布包含**: 最早有API/ancestry证据收录 `next-2026-09-18.2`（预发布）；全部source也含于正式09-26.1/.2。准备target仅记prepared，不由本字段推断准备版本已发布。
- **实际behavior / sourceDelta**: 本来源已合并首批中段整合准备及notes/requests，不证明准备tag已发布。manifest列出src/main/resources/katago-assets.json；engineReleaseTag等资源定位字段更新必须保留为source linkage，不是notes-only。
- **Canonical owner / scope**: 无独立发行功能；T01-RESOURCE/REL-05功能source linkage。
- **候选/覆盖与边界**: 无Tauri运行继承；notes/requests和Java测试不要求产品复刻。源变化由§6索引保留。
- **Disposition**: 混合：准备/测试内部排除；catalog linkage随资源功能。
- **责任意图**: T04-RESOURCE。
- **Observable assertions**: 逐项使用§2 API/ancestry确定发布归属；资源定位变化须能解析明确来源并在缺失/离线时可见失败，不能因准备URL存在判下载可用或已验收。

<a id="ud-04-025"></a>
### UD-04-025 Java17独立distribution/runtime与产物验收框架

- **Source / event**: PR #522 (集成事件 25) | 共 10 个提交；第一父E25。
- **完整source commits（与§6反向对应）**:
  - [e072e143cb848e054347e6c5157241567755915a](https://github.com/wimi321/lizzieyzy-next/commit/e072e143cb848e054347e6c5157241567755915a) — Merge pull request #522 from qiyi71w/impl/distribution-runtime
  - [09453697968c0dc95fe87309a40448210e2ecbae](https://github.com/wimi321/lizzieyzy-next/commit/09453697968c0dc95fe87309a40448210e2ecbae) — ci: consume existing product artifacts
  - [6e0ebd2918839dbbc65d5e168ab83d954e29010f](https://github.com/wimi321/lizzieyzy-next/commit/6e0ebd2918839dbbc65d5e168ab83d954e29010f) — fix(acceptance): harden runtime evidence
  - [c610b36b6f446b461677c04f49f7d998413988b8](https://github.com/wimi321/lizzieyzy-next/commit/c610b36b6f446b461677c04f49f7d998413988b8) — fix(test): keep executor cleanup Java 17 compatible
  - [9f2270cfdf830d5506641407b812c07954c46c62](https://github.com/wimi321/lizzieyzy-next/commit/9f2270cfdf830d5506641407b812c07954c46c62) — docs(acceptance): reconcile runtime evidence
  - [6ce3e5d703b72ae481ce9de2f73fbafb213ae37d](https://github.com/wimi321/lizzieyzy-next/commit/6ce3e5d703b72ae481ce9de2f73fbafb213ae37d) — ci(release): add candidate acceptance workflows
  - [14a2348512830f69b95a6aa8f2d094b47edf75e8](https://github.com/wimi321/lizzieyzy-next/commit/14a2348512830f69b95a6aa8f2d094b47edf75e8) — feat(runtime): verify standalone Java 17
  - [0915462ca991d0f7aff15f484d58c300c5055e9e](https://github.com/wimi321/lizzieyzy-next/commit/0915462ca991d0f7aff15f484d58c300c5055e9e) — feat(release): add macOS product acceptance
  - [d1a9a8de23f69e7d76207889564e804c3d06b999](https://github.com/wimi321/lizzieyzy-next/commit/d1a9a8de23f69e7d76207889564e804c3d06b999) — feat(release): add Linux product acceptance
  - [fcc5f71fc9af245314e7ef3aecc1cb887b6104a4](https://github.com/wimi321/lizzieyzy-next/commit/fcc5f71fc9af245314e7ef3aecc1cb887b6104a4) — feat(release): add Windows product acceptance
- **发布包含**: 最早有API/ancestry证据收录 `next-2026-09-22.2`（预发布）；全部source也含于正式09-26.1/.2。准备target仅记prepared，不由本字段推断准备版本已发布。
- **实际behavior / sourceDelta**: PR522建立standalone Java17/host verification、cross-platform product acceptance/provenance框架。Tauri不绑定JRE；解压/启动/推理的实际资源可用目标已有T01-RESOURCE功能与发行边界。
- **Canonical owner / scope**: Java JVM/distribution内部；发行证据保留。
- **候选/覆盖与边界**: manifest提供构建/脚本/文档角色，无产品验收声称。
- **Disposition**: Java机制排除；不删用户资源义务。
- **责任意图**: 无独立产品任务；资源目标由T04-RESOURCE承接。
- **Observable assertions**: 不移植JVM框架，不把Java框架success当Tauri资源/真实引擎/安装证据；混合资源消费者继续有owner。

<a id="ud-04-028"></a>
### UD-04-028 next-2026-09-22.1准备与来源联接

- **Source / event**: PR #525 (集成事件 28) | 共 2 个提交；第一父E28。
- **完整source commits（与§6反向对应）**:
  - [3df8606879fa3a767f2925caa6e7933d4a4d6d19](https://github.com/wimi321/lizzieyzy-next/commit/3df8606879fa3a767f2925caa6e7933d4a4d6d19) — Merge pull request #525 from wimi321/codex/release-next-2026-09-22-1
  - [12440af64a784837d4887d43c41b2b7a99b4bbbf](https://github.com/wimi321/lizzieyzy-next/commit/12440af64a784837d4887d43c41b2b7a99b4bbbf) — chore(release): prepare Windows stability pre-release next-2026-09-22.1
- **发布包含**: 最早有API/ancestry证据收录 `next-2026-09-22.2`（预发布）；全部source也含于正式09-26.1/.2。准备target仅记prepared，不由本字段推断准备版本已发布。
- **实际behavior / sourceDelta**: 本来源已合并Windows稳定性准备及notes/requests，不证明准备tag已发布。manifest列出src/main/resources/katago-assets.json；engineReleaseTag等资源定位字段更新必须保留为source linkage，不是notes-only。
- **Canonical owner / scope**: 无独立发行功能；T01-RESOURCE/REL-05功能source linkage。
- **候选/覆盖与边界**: 无Tauri运行继承；notes/requests和Java测试不要求产品复刻。源变化由§6索引保留。
- **Disposition**: 混合：准备/测试内部排除；catalog linkage随资源功能。
- **责任意图**: T04-RESOURCE。
- **Observable assertions**: 逐项使用§2 API/ancestry确定发布归属；资源定位变化须能解析明确来源并在缺失/离线时可见失败，不能因准备URL存在判下载可用或已验收。

<a id="ud-04-035"></a>
### UD-04-035 next-2026-09-22.2准备与来源联接

- **Source / event**: PR #532 (集成事件 35) | 共 4 个提交；第一父E35。
- **完整source commits（与§6反向对应）**:
  - [8e6b76bd54c6a6927214bccb258ae86b33788603](https://github.com/wimi321/lizzieyzy-next/commit/8e6b76bd54c6a6927214bccb258ae86b33788603) — Merge pull request #532 from wimi321/codex/release-next-2026-09-22-2
  - [5d43f3a8b2a4655a6f7b7370a447ff5f649a9bf3](https://github.com/wimi321/lizzieyzy-next/commit/5d43f3a8b2a4655a6f7b7370a447ff5f649a9bf3) — Integrate reviewed match rule choices and final source verification
  - [9cc51e3b6f00f2982d9666190906d5eb1f8df833](https://github.com/wimi321/lizzieyzy-next/commit/9cc51e3b6f00f2982d9666190906d5eb1f8df833) — Merge branch 'codex/pr531-integration-20260922' into codex/release-next-2026-09-22-2
  - [aec4f4435e12c082504298e1a3e291ac19646a4a](https://github.com/wimi321/lizzieyzy-next/commit/aec4f4435e12c082504298e1a3e291ac19646a4a) — chore(release): prepare fully integrated Windows stability pre-release .2
- **发布包含**: 最早有API/ancestry证据收录 `next-2026-09-22.2`（预发布）；全部source也含于正式09-26.1/.2。准备target仅记prepared，不由本字段推断准备版本已发布。
- **实际behavior / sourceDelta**: 本来源已合并规则整合/source verification准备及notes/requests，不证明准备tag已发布。manifest列出src/main/resources/katago-assets.json；engineReleaseTag等资源定位字段更新必须保留为source linkage，不是notes-only。集成带入规则的合并不重复拥有UD-04-034源码行为。
- **Canonical owner / scope**: 无独立发行功能；T01-RESOURCE/REL-05功能source linkage。
- **候选/覆盖与边界**: 无Tauri运行继承；notes/requests和Java测试不要求产品复刻。源变化由§6索引保留。
- **Disposition**: 混合：准备/测试内部排除；catalog linkage随资源功能。
- **责任意图**: T04-RESOURCE。
- **Observable assertions**: 逐项使用§2 API/ancestry确定发布归属；资源定位变化须能解析明确来源并在缺失/离线时可见失败，不能因准备URL存在判下载可用或已验收。

<a id="ud-05-01"></a>
### UD-05-01 

- 来源：[001](https://github.com/wimi321/lizzieyzy-next/commit/f211a22087c95f589067191fd017fccb5edfc516), [002](https://github.com/wimi321/lizzieyzy-next/commit/8e808201eefd5056f917807514dc51bebf1a448c), [003](https://github.com/wimi321/lizzieyzy-next/commit/50898233f5cce7d952b37cdb9ee005f780ea948f), [013](https://github.com/wimi321/lizzieyzy-next/commit/89ff0200c20b7007e5ccd2e5d5c020b349041418), [014](https://github.com/wimi321/lizzieyzy-next/commit/74afda436fdcb45b7eece1a0776f498e9ce562d4), [015](https://github.com/wimi321/lizzieyzy-next/commit/3f58c0b62160bb0e46f7e9765da100371bb78b2d), [078](https://github.com/wimi321/lizzieyzy-next/commit/87cdedc37dc87e48fd0f270a5722dbb5cfb5b351), [082](https://github.com/wimi321/lizzieyzy-next/commit/40bb84eb6acdd7ec40811ddcd446cf8c52b49ddb), [083](https://github.com/wimi321/lizzieyzy-next/commit/8600b61586dc99532bb32306519a1cc9001c6999), [084](https://github.com/wimi321/lizzieyzy-next/commit/c013d8ce66afc186df5af96c83a61cbb3e4a14ba), [145](https://github.com/wimi321/lizzieyzy-next/commit/af0e07a7386483f3bfc8a15780de72ffc2f0de4c)（完整SHA/发布包含见源索引）。
- 用户行为/边界：发布请求、固定引擎工件复用与多语言发布说明；终点仅将资源目录定位 tag 从10-01.1改为10-07.1，并非新翻译实现或已发布证明。
- Tauri映射/具名任务意图：REL-01–REL-10；T05-RELEASE-PROVENANCE（发行阶段）
- 覆盖证据或缺口：Java release request/workflow/catalog 与已观察 Release 主源；Tauri当前发行仓库未定。
- 处置：Java CI脚本不照搬；保留发行来源/版本/校验与支持链接义务，运行本地资源不依赖当前 Releases。
- 后续验收/调查停止条件：未来选定托管后验证实际发布资产、清单/版本一致与失败可见；本轮只验来源和包含关系。

<a id="ud-05-04"></a>
### UD-05-04 

- 来源：[007](https://github.com/wimi321/lizzieyzy-next/commit/36ce7e546c3dc4c77d12bd26b9f9bd81846eadf3), [039](https://github.com/wimi321/lizzieyzy-next/commit/caff4fc1420670d01f058b2717c5081eadfda24e), [045](https://github.com/wimi321/lizzieyzy-next/commit/04b5306336c08c5143f4482780381994a17e5b44), [046](https://github.com/wimi321/lizzieyzy-next/commit/7d30c8f41ddabb3d6723faff8337d3b51477ebed), [047](https://github.com/wimi321/lizzieyzy-next/commit/8c43fdd24b01acd6485b8b3911708290ae18ab75), [050](https://github.com/wimi321/lizzieyzy-next/commit/efa54a3700f7f137c56fbd130250ece77979914c), [060](https://github.com/wimi321/lizzieyzy-next/commit/8d3cc68a790812a568ea8d460d32fc50145d98e0), [073](https://github.com/wimi321/lizzieyzy-next/commit/22b948c814acb29240a5153d12b48b88dbf24614), [076](https://github.com/wimi321/lizzieyzy-next/commit/adb74114377fafdbb09433641791554b4967aded), [077](https://github.com/wimi321/lizzieyzy-next/commit/0fe5b3769e5f1402013df5520643466ff021f5e6), [113](https://github.com/wimi321/lizzieyzy-next/commit/0469dd5b5287f13a74660f89fdcdbd0c435d8011), [130](https://github.com/wimi321/lizzieyzy-next/commit/94e4a2bdf97d0ba6e04e3138d7f8ac522fd46288), [132](https://github.com/wimi321/lizzieyzy-next/commit/d9812785023444b92afbf982c817b63f44d3a7bc), [138](https://github.com/wimi321/lizzieyzy-next/commit/fc399c0ff755165b7fcae33de8d24276af5e7164), [140](https://github.com/wimi321/lizzieyzy-next/commit/219d47ec9c2307f6ce6bb6976b2475883619bab3), [141](https://github.com/wimi321/lizzieyzy-next/commit/03202192ab183a5c0a0e71e803127fbfca8d6e17), [142](https://github.com/wimi321/lizzieyzy-next/commit/8b0d9bda872f79a5beb0194c0036b9dc25f95d24), [143](https://github.com/wimi321/lizzieyzy-next/commit/b82b6611b7dc4ce92d1a0f5e2d2063b585201617)（完整SHA/发布包含见源索引）。
- 用户行为/边界：测试夹具隔离、模拟平台/无头截图门禁、格式与Java现场QA记录；不添加独立Tauri用户功能。
- Tauri映射/具名任务意图：T05-INTERNAL-EVIDENCE（排除独立产品任务）
- 覆盖证据或缺口：changed-files与PR568/580/596内容核对：受测产品变化分别归05-05等对应行；历史Java Windows结果仍只属于其候选。
- 处置：Java内部不适用；测试/QA证据归相关行为，不能转作TauriPassed。
- 后续验收/调查停止条件：源索引每提交有归属，相关用户义务仍在其他行；无Java构建/测试执行。

<a id="udx-024"></a>
## UDX-024 — 启动自检产品决定及measured tuning

当前阶段导航 **R12/R13（逐意图）**（历史来源组 **B**；混合归属以意图索引为准）；条目/后继 **ENG-15**。各来源的有界调查/决定仍须先输出结论，不能把族导航视为新增实现或全部Accepted。明确的其他阶段消费者按其任务合同单独交付；其存在不阻塞独立本地工作。

<a id="ud-04-009"></a>
### UD-04-009 启动benchmark让位用户分析

- **Source / event**: PR #500 (集成事件 9) | 共 3 个提交；第一父E09。
- **完整source commits（与§6反向对应）**:
  - [1c0b6bd7e3f2a6c2771457552715efe2ea3aa339](https://github.com/wimi321/lizzieyzy-next/commit/1c0b6bd7e3f2a6c2771457552715efe2ea3aa339) — Merge pull request #500 from wimi321/fix/startup-benchmark-yield-20260917
  - [7cb1a41df48c57e6e3215dad0d63f7ad6896e8ca](https://github.com/wimi321/lizzieyzy-next/commit/7cb1a41df48c57e6e3215dad0d63f7ad6896e8ca) — Await persisted log events in engine identity assertions
  - [162d50debb153978f602217f452901aff9fe065b](https://github.com/wimi321/lizzieyzy-next/commit/162d50debb153978f602217f452901aff9fe065b) — Avoid startup benchmark failure dialogs when analysis takes priority
- **发布包含**: 最早有API/ancestry证据收录 `next-2026-09-18.2`（预发布）；全部source也含于正式09-26.1/.2。准备target仅记prepared，不由本字段推断准备版本已发布。
- **实际behavior / sourceDelta**: PR500区分启动benchmark被用户分析占用/让位与真实失败，避免误导错误；当前Next没有因此获得性能调优等价或批准永久删除。单一生命周期/无隐式切换约束不是逐项产品批准。
- **Canonical owner / scope**: 用户性能目标未分配后继；ANA-06原范围不变。
- **候选/覆盖与边界**: 尚无此新增行为的可继承运行证据；不把Java修复判为当前Next缺陷。
- **Disposition**: Java调度机制排除；用户目标具名决策。
- **责任意图**: T04-STARTUP-PERFORMANCE。
- **Observable assertions**: 先决定性能评估入口/默认/保存/让位边界；产出可判定用户合同及批准引用。保留用户主动分析/暂停优先，无隐式Run/backend；未找到批准时保持具名余项，不闭合为approved replacement。

<a id="ud-04-013"></a>
### UD-04-013 startup性能自检与前台资源让位

- **Source / event**: PR #504 (集成事件 13) | 共 5 个提交；第一父E13。
- **完整source commits（与§6反向对应）**:
  - [9f086cebfe235dde813128152903c77e546fe9b9](https://github.com/wimi321/lizzieyzy-next/commit/9f086cebfe235dde813128152903c77e546fe9b9) — Merge pull request #504 from wimi321/fix/startup-benchmark-user-work-20260917
  - [e07bb0eb31b5b53fb11fc2c2fd67da9955e563d2](https://github.com/wimi321/lizzieyzy-next/commit/e07bb0eb31b5b53fb11fc2c2fd67da9955e563d2) — Merge remote-tracking branch 'origin/main' into fix/startup-benchmark-user-work-20260917
  - [94706b7be6199678e2bb04aa42e8bc284c08b588](https://github.com/wimi321/lizzieyzy-next/commit/94706b7be6199678e2bb04aa42e8bc284c08b588) — Merge branch 'fix/humansl-thread-alias-20260917' into fix/startup-benchmark-user-work-20260917
  - [c984eccdec892b4b28bfe80471add25b27d76462](https://github.com/wimi321/lizzieyzy-next/commit/c984eccdec892b4b28bfe80471add25b27d76462) — Distinguish the primary engine from auxiliary compute for startup tuning
  - [24b181b4d6d285716b932c954d4e63723c06fe46](https://github.com/wimi321/lizzieyzy-next/commit/24b181b4d6d285716b932c954d4e63723c06fe46) — Keep delayed startup tuning from interrupting user analysis and training
- **发布包含**: 最早有API/ancestry证据收录 `next-2026-09-18.2`（预发布）；全部source也含于正式09-26.1/.2。准备target仅记prepared，不由本字段推断准备版本已发布。
- **实际behavior / sourceDelta**: PR504调整AnalysisResourceCoordinator与foreground/user-work调度，让benchmark不抢正在工作的用户analysis。Java独立线程/进程机制可不复制，避让/不误报/不丢用户意图的目标保留。
- **Canonical owner / scope**: ANA-06/16窄范围；未分配性能后继。
- **候选/覆盖与边界**: H-ANALYSIS原Windows范围可引用；无startup性能评估等价证据，不能标全部Covered。
- **Disposition**: 原分析证据窄继承；startup性能决策保留。
- **责任意图**: T04-STARTUP-PERFORMANCE、T04-HANDOFF。
- **Observable assertions**: 按T04-STARTUP-PERFORMANCE冻结显式行为；若重用现分析能力，H-ANALYSIS仅证明有限交还等原范围，不替startup benchmark等价。后台性能工作让位后不得弹伪失败、启动额外隐式Run或覆盖暂停。

<a id="ud-04-042"></a>
### UD-04-042 性能probe engine/app测量分界

- **Source / event**: PR #540 (集成事件 38) | 共 5 个提交；第一父E38。
- **完整source commits（与§6反向对应）**:
  - [55c2d54b3b9beeb331a8d6a877c40b027fee0601](https://github.com/wimi321/lizzieyzy-next/commit/55c2d54b3b9beeb331a8d6a877c40b027fee0601) — docs(perf): record v2 probe validation and measurement boundaries
  - [b7ba99e579044961e417ea9681565017cf8bafdc](https://github.com/wimi321/lizzieyzy-next/commit/b7ba99e579044961e417ea9681565017cf8bafdc) — test(perf): drain ordinary restore stages without superseding their lineage
  - [a9d36e5f19ff665b0a7f1bef4153972aeef0f73c](https://github.com/wimi321/lizzieyzy-next/commit/a9d36e5f19ff665b0a7f1bef4153972aeef0f73c) — test(perf): confirm restoration and actual process evidence before timing
  - [e7c032830fad95e7f319655d792e91a845979302](https://github.com/wimi321/lizzieyzy-next/commit/e7c032830fad95e7f319655d792e91a845979302) — test(perf): measure fixed-budget engine and application analysis separately
  - [8a6630e8b9898f1b8fda6dab21230d54799dc6ec](https://github.com/wimi321/lizzieyzy-next/commit/8a6630e8b9898f1b8fda6dab21230d54799dc6ec) — QA integrate PR 540
- **发布包含**: 最早有API/ancestry证据收录 `next-2026-09-26.1`（正式发布）；全部source也含于正式09-26.1/.2。准备target仅记prepared，不由本字段推断准备版本已发布。
- **实际behavior / sourceDelta**: PR540的docs/test probe v2把engine-only固定预算推理与application analysis分开；等待restore stages/drain及production process exit证据。其Java probe/EDT计时实现不是必复制模块，但报告不能偷换测量对象。
- **Canonical owner / scope**: 未分配measured tuning后继；非性能Parity新ID。
- **候选/覆盖与边界**: 尚无此新增行为的可继承运行证据；不把Java修复判为当前Next缺陷。
- **Disposition**: test/docs机制排除；用户measurement限制随调优。
- **责任意图**: T04-MEASURED-TUNING。
- **Observable assertions**: measured report必须标engine/app mode与预算/fixture/进程证据；engine-only benchmark不替application latency/cancel证据。whole-game Cancel用实际worker退出，不用JSON ack；cold verification成本和search gain分开，未测范围可见。

<a id="ud-04-047"></a>
### UD-04-047 live/whole-game measured report review与安全overlay

- **Source / event**: PR #546 (集成事件 38) | 共 10 个提交；第一父E38。
- **完整source commits（与§6反向对应）**:
  - [186a4d48b945f17653b6a508176f89f0bbfc6a31](https://github.com/wimi321/lizzieyzy-next/commit/186a4d48b945f17653b6a508176f89f0bbfc6a31) — docs(perf): publish controlled Windows evidence and acceptance limits
  - [b29d2765e362197b9f027193703b7093caecd35b](https://github.com/wimi321/lizzieyzy-next/commit/b29d2765e362197b9f027193703b7093caecd35b) — fix(tuning): qualify sampled GPU memory at explicit apply confirmation
  - [24752abd28fff870c32dfdb3a6d97e5db7a2f182](https://github.com/wimi321/lizzieyzy-next/commit/24752abd28fff870c32dfdb3a6d97e5db7a2f182) — test(tuning): assert startup honors resolved launch thread override
  - [437e3b6c84dfa60a1693400bb65f7c9fefefc793](https://github.com/wimi321/lizzieyzy-next/commit/437e3b6c84dfa60a1693400bb65f7c9fefefc793) — fix(tuning): share setup task gate and bind confirmation identity
  - [c281b67db69a5c674bdc8cd278cc5f8dcb7d9b10](https://github.com/wimi321/lizzieyzy-next/commit/c281b67db69a5c674bdc8cd278cc5f8dcb7d9b10) — test(perf): gate measured exports on reliable process evidence
  - [daa76f86200a0dd8e7c87672aa117c0f636577cc](https://github.com/wimi321/lizzieyzy-next/commit/daa76f86200a0dd8e7c87672aa117c0f636577cc) — feat(perf): summarize and export measured analysis evidence
  - [8bf99cca5ffb322f6b6955f841278cb009408270](https://github.com/wimi321/lizzieyzy-next/commit/8bf99cca5ffb322f6b6955f841278cb009408270) — fix(tuning): reject unsupported scenes and invalidate closed reviews
  - [f4ccaef5470b8da87ee5ccd0ca30fefa2a9bded4](https://github.com/wimi321/lizzieyzy-next/commit/f4ccaef5470b8da87ee5ccd0ca30fefa2a9bded4) — fix(tuning): normalize per-run log output locations
  - [73f025926b59e26ce8826e6b886ef606b5849f0a](https://github.com/wimi321/lizzieyzy-next/commit/73f025926b59e26ce8826e6b886ef606b5849f0a) — feat(tuning): review and confirm measured scene-specific profiles
  - [123e168e4730f4655a0b0b2618afa32077789486](https://github.com/wimi321/lizzieyzy-next/commit/123e168e4730f4655a0b0b2618afa32077789486) — QA integrate PR 546
- **发布包含**: 最早有API/ancestry证据收录 `next-2026-09-26.1`（正式发布）；全部source也含于正式09-26.1/.2。准备target仅记prepared，不由本字段推断准备版本已发布。
- **实际behavior / sourceDelta**: PR546十来源全部归E38。73f02592与endpoint KATAGO_MEASURED_TUNING_SCHEMA定义live / whole-game，不是开中终盘。report import在后台review，以saved engine identity、engine/model/config SHA256、ordered recursive include hashes、hardware/GPU/driver及effective non-tuning command fingerprint匹配；数据不是命令也不执行report路径。明确Apply/Restore，scene-specific launch overlays不写saved command；whole-game不spill到quick/HumanSL，SSH independent analysis与analysisReuseCurrentEngine不支持。import/restore和setup/model/benchmark共享busy gate；close/hide取消并使旧callback/confirm失效，即使reopen亦然。24752abd是test-only，初始化不发kata-set-param numSearchThreads覆盖resolved launch override；b29d2765仅copy/test澄清sampled total GPU memory，非calibration算法。
- **Canonical owner / scope**: 未分配measured-tuning后继；静态argv原证据单列。
- **候选/覆盖与边界**: H-STATIC-THREAD真实Windows/KataGo1.12.3显式Restart证据保留；测量schema样例不是实际benchmark结果。未执行新的benchmark/engine/native验收。
- **Disposition**: 需measured功能后继；dynamic-thread有界决策。
- **责任意图**: T04-MEASURED-TUNING、T04-THREAD-CONTROL。
- **Observable assertions**: §5 measured合同覆盖完整fingerprint/import-review/单独Apply/Restore、stale/close/hide/busy与limits。static argv按H-STATIC-THREAD窄继承；dynamic来源/有效值/temporary override尚未验，不让只出现threads argv通过measured功能。

<a id="udx-025"></a>
## UDX-025 — 工作区可访问性与主题/字体目标

当前阶段导航 **R17**（历史来源组 **F**；混合归属以意图索引为准）；条目/后继 **UI-02, APPEAR-01**。各来源的有界调查/决定仍须先输出结论，不能把族导航视为新增实现或全部Accepted。明确的其他阶段消费者按其任务合同单独交付；其存在不阻塞独立本地工作。

<a id="ud-04-011"></a>
### UD-04-011 评论渲染与主题刷新EDT调度

- **Source / event**: PR #502 (集成事件 11) | 共 2 个提交；第一父E11。
- **完整source commits（与§6反向对应）**:
  - [c07a45a4f142b2d8a7560d175ec317e752271031](https://github.com/wimi321/lizzieyzy-next/commit/c07a45a4f142b2d8a7560d175ec317e752271031) — Merge pull request #502 from wimi321/fix/comment-render-edt-20260917
  - [d445977043df41f1b683559d75cff7f87e40aa7d](https://github.com/wimi321/lizzieyzy-next/commit/d445977043df41f1b683559d75cff7f87e40aa7d) — Serialize comment rendering and theme refresh on the Swing EDT
- **发布包含**: 最早有API/ancestry证据收录 `next-2026-09-18.2`（预发布）；全部source也含于正式09-26.1/.2。准备target仅记prepared，不由本字段推断准备版本已发布。
- **实际behavior / sourceDelta**: PR502将评论解析/主题刷新投递Swing EDT，防Java组件并发修改。没有改个人comment或生成分析数据语义；React不是任何layout/并发的runtime免疫证明。
- **Canonical owner / scope**: SGF-05既有comment能力，Swing内部调度。
- **候选/覆盖与边界**: 保留SGF-05原Accepted，不增加候选运行声称。
- **Disposition**: Java EDT机制不适用；不另造feature任务。
- **责任意图**: 无独立产品任务。
- **Observable assertions**: 冻结source证明范围是Swing scheduling且不删除comment/主题入口目标；未来实际consumer修改时保留personal/generated分离，不从此源声明新native通过。

<a id="ud-05-03"></a>
### UD-05-03 

- 来源：[005](https://github.com/wimi321/lizzieyzy-next/commit/98a7610831330b4a3b08975fda092a2f4c80cafe), [006](https://github.com/wimi321/lizzieyzy-next/commit/1a186485e17d9961fccbe180f234e326718a168e), [009](https://github.com/wimi321/lizzieyzy-next/commit/849ef5cb51c513c7f994f26fc1dc81f2029ab627), [010](https://github.com/wimi321/lizzieyzy-next/commit/b7285159f2eaf94771d1d41f2a028cc40e1d9bcc)（完整SHA/发布包含见源索引）。
- 用户行为/边界：桌面样式、主题/字体、键盘与屏幕阅读：焦点归还、插入符滚动、长标签/紧凑工具栏、预览高度与语义控件；保留用户布局习惯，不移植Swing组件。
- Tauri映射/具名任务意图：UI-02及主题/字体/布局后继；T05-WORKBENCH-ACCESS（F，新增入口各批同步）
- 覆盖证据或缺口：PR551含源码及Windows记录；不是Tauri DPI/主题验收。
- 处置：后继覆盖；现有窄布局Accepted不扩成全主题全DPI。
- 后续验收/调查停止条件：各新入口键盘/取消/焦点/语言溢出；最终UI-02实际桌面主题、字体、缩放、屏幕阅读组合。

<a id="udx-026"></a>
## UDX-026 — 全树坐标/色彩变换

当前阶段导航 **R11**（历史来源组 **A**；混合归属以意图索引为准）；条目/后继 **SGF-16**。各来源的有界调查/决定仍须先输出结论，不能把族导航视为新增实现或全部Accepted。明确的其他阶段消费者按其任务合同单独交付；其存在不阻塞独立本地工作。

<a id="ud-04-027"></a>
### UD-04-027 PR524 offline可用、首手/pass色与全树transform

- **Source / event**: PR #524 (集成事件 27) | 共 2 个提交；第一父E27。
- **完整source commits（与§6反向对应）**:
  - [2a487ced3bdf0d0557478313a77758492cbb6bc9](https://github.com/wimi321/lizzieyzy-next/commit/2a487ced3bdf0d0557478313a77758492cbb6bc9) — Merge pull request #524 from wimi321/codex/windows-qa-20260922
  - [d128c029d70b648faaf571deeb33713bda83af7d](https://github.com/wimi321/lizzieyzy-next/commit/d128c029d70b648faaf571deeb33713bda83af7d) — fix(windows): keep board usable after engine startup failure
- **发布包含**: 最早有API/ancestry证据收录 `next-2026-09-22.2`（预发布）；全部source也含于正式09-26.1/.2。准备target仅记prepared，不由本字段推断准备版本已发布。
- **实际behavior / sourceDelta**: d128c029包括no-engine import/place/pass/history/komi；transform replay从实际sentinel起，不跳第一真实手；pass保持记录颜色（包括leading White pass与交换黑白）；offline clear保持贴目。另修cross-host absolute POSIX/Windows evidence path validator与desktop CI/IME测试说明，不改生产IME。
- **Canonical owner / scope**: UI-04 Accepted；SGF-10/13既有；SGF-16 Deferred/T02-SGF-16。
- **候选/覆盖与边界**: H-OFFLINE仅支持原no-engine打谱/导航/Save；H-KOMI-EDITOR仅metadata，不证明transform和clear保持。SGF-16仍Deferred。
- **Disposition**: offline窄覆盖；transform功能增量；clear-komi调查。
- **责任意图**: T04-TRANSFORM、T04-OFFLINE。
- **Observable assertions**: T02-SGF-16实现时检首真实手、leading W pass、color swap、所有branches、setup/markup及serialize/reparse；无transform入口不等于Next当前缺陷。对offline clear的当前可达语义核对贴目输入/清盘保留/显式New参数边界，不把H-KOMI-EDITOR扩成clear全证据。

<a id="udx-027"></a>
## UDX-027 — 多文件分析准入与会话队列

当前阶段导航 **R12/R13（逐意图）**（历史来源组 **B**；混合归属以意图索引为准）；条目/后继 **ANA-07**。各来源的有界调查/决定仍须先输出结论，不能把族导航视为新增实现或全部Accepted。明确的其他阶段消费者按其任务合同单独交付；其存在不阻塞独立本地工作。

<a id="ud-04-032"></a>
### UD-04-032 ordinary batch启动与nested sync准入

- **Source / event**: PR #529 (集成事件 32) | 共 4 个提交；第一父E32。
- **完整source commits（与§6反向对应）**:
  - [7b116c09b274aa82c1174519bd66ba4275c5bdc8](https://github.com/wimi321/lizzieyzy-next/commit/7b116c09b274aa82c1174519bd66ba4275c5bdc8) — Merge pull request #529 from qiyi71w/fix/523-batch-analysis
  - [15d0cc3dabad8611edd9e00ae8a1432a19cfbbfc](https://github.com/wimi321/lizzieyzy-next/commit/15d0cc3dabad8611edd9e00ae8a1432a19cfbbfc) — test(analysis): await nested batch synchronization admission
  - [05391234f7107610b432b2304a6b81fdebadf17c](https://github.com/wimi321/lizzieyzy-next/commit/05391234f7107610b432b2304a6b81fdebadf17c) — Merge remote-tracking branch 'origin/main' into codex/batch-analysis-main-sync-20260922
  - [ff2bb9de50ce82c78e54003dd93721c1f439b54d](https://github.com/wimi321/lizzieyzy-next/commit/ff2bb9de50ce82c78e54003dd93721c1f439b54d) — fix(analysis): restore ordinary batch startup
- **发布包含**: 最早有API/ancestry证据收录 `next-2026-09-22.2`（预发布）；全部source也含于正式09-26.1/.2。准备target仅记prepared，不由本字段推断准备版本已发布。
- **实际behavior / sourceDelta**: PR529 ff2bb9de恢复菜单/toolbar的ordinary multi-file batch startup，15d0cc3d测试等待nested batch sync admission。显式当前棋谱ANA-16不是多文件队列；merged upstream test等待不照搬Java线程。
- **Canonical owner / scope**: ANA-07 Deferred；T02-ANA-07（B）。
- **候选/覆盖与边界**: 尚无此新增行为的可继承运行证据；不把Java修复判为当前Next缺陷。
- **Disposition**: 需既有Deferred功能与来源增量。
- **责任意图**: T04-BATCH。
- **Observable assertions**: session-only有序queue、所有真实入口可见pending/running/completed/failed/cancelled；等待准入不静默消失，Cancel使旧file/run/job无法发布；不replace/dirty current-game，不持久恢复queue。输出SGF显式保存，源文件失败保护按共享写入合同。

<a id="udx-028"></a>
## UDX-028 — 双参与者runtime-komi事务

当前阶段导航 **R15**（历史来源组 **D**；混合归属以意图索引为准）；条目/后继 **GAME-12**。各来源的有界调查/决定仍须先输出结论，不能把族导航视为新增实现或全部Accepted。明确的其他阶段消费者按其任务合同单独交付；其存在不阻塞独立本地工作。

<a id="ud-04-033"></a>
### UD-04-033 双参与者runtime-komi事务与offline图表

- **Source / event**: PR #530 (集成事件 33) | 共 4 个提交；第一父E33。
- **完整source commits（与§6反向对应）**:
  - [a0135a8a6fa57606141326fb2f92b230fddaea6a](https://github.com/wimi321/lizzieyzy-next/commit/a0135a8a6fa57606141326fb2f92b230fddaea6a) — Merge pull request #530 from qiyi71w/fix/issue-518-runtime-komi
  - [750530649d31787e9c5dacfa75f8540cfbaccc76](https://github.com/wimi321/lizzieyzy-next/commit/750530649d31787e9c5dacfa75f8540cfbaccc76) — Keep komi editing and score graphs usable without an engine
  - [b1b38bf3858b72365bb2ad20c7d6fe9bf1e031d0](https://github.com/wimi321/lizzieyzy-next/commit/b1b38bf3858b72365bb2ad20c7d6fe9bf1e031d0) — Merge latest main into runtime komi fix
  - [6b7944e266ee01c1fbee7c6456d755bbb4d83786](https://github.com/wimi321/lizzieyzy-next/commit/6b7944e266ee01c1fbee7c6456d755bbb4d83786) — fix(engine-game): synchronize runtime komi
- **发布包含**: 最早有API/ancestry证据收录 `next-2026-09-22.2`（预发布）；全部source也含于正式09-26.1/.2。准备target仅记prepared，不由本字段推断准备版本已发布。
- **实际behavior / sourceDelta**: 6b7944e2是current engine-game owner的双参与者事务：先结清已发genmove并由对手接受；合并pending edits仅保留latest target；两侧komi ACK与final fence成功后才原子提交current GameInfo/analysis缓存。保持latest pause意图、不改frozen opening plan/批次defaults；拒绝旧komi revision；部分写后失败/Cancel退役不确定原实例，不能污染replacement/new match。75053064另保持no-engine komi controls及既存winrate/score graph可用。
- **Canonical owner / scope**: GAME-01–04 Accepted原scope；SGF-13/ANA-11/UI-04原scope。
- **候选/覆盖与边界**: H-KOMI-EDITOR/H-OFFLINE是不同原用户范围，ANA-11 Accepted保留但本票未补该源offline-chart候选/动态等价；T04-OFFLINE负责有限核对。runtime事务未证明实现，无观察到Next故障的断言。
- **Disposition**: runtime-komi后继；offline细分调查。
- **责任意图**: T04-KOMI、T04-OFFLINE。
- **Observable assertions**: 单侧ACK不能commit；连续edit不得中间resume；rejection/send failure/timeout/identity loss保留last confirmed KM，无伪结果；Cancel/partial failure旧实例cleanup、late ACK/analysis/move隔离；offline graph仅显示SGF真实analysis，无engine仍可navigation。

<a id="udx-029"></a>
## UDX-029 — No-engine支持路径调查

当前阶段导航 **R11**（历史来源组 **A**；混合归属以意图索引为准）；条目/后继 **UI-04, SGF-13**。各来源的有界调查/决定仍须先输出结论，不能把族导航视为新增实现或全部Accepted。明确的其他阶段消费者按其任务合同单独交付；其存在不阻塞独立本地工作。

<a id="ud-04-036"></a>
### UD-04-036 QA接受ready no-engine live session

- **Source / event**: PR #533 (集成事件 36) | 共 1 个提交；第一父E36。
- **完整source commits（与§6反向对应）**:
  - [2e1e08c9e90f22738cad525f3bd4fbb5813bb0a7](https://github.com/wimi321/lizzieyzy-next/commit/2e1e08c9e90f22738cad525f3bd4fbb5813bb0a7) — fix(qa): accept ready no-engine live sessions (#533)
- **发布包含**: 最早有API/ancestry证据收录 `next-2026-09-26.1`（正式发布）；全部source也含于正式09-26.1/.2。准备target仅记prepared，不由本字段推断准备版本已发布。
- **实际behavior / sourceDelta**: 2e1e08c9仅修scripts/test_windows_product_acceptance.py与powershell等对ready no-engine状态的判定；不把Java验收脚本修订叫Tauri生产修复。
- **Canonical owner / scope**: Java acceptance脚本；UI-04既有。
- **候选/覆盖与边界**: H-OFFLINE是原功能证据；本commit没有新增Tauri Covered行为。
- **Disposition**: QA内部排除，no-engine原证据保留。
- **责任意图**: 无独立产品任务。
- **Observable assertions**: 源角色为QA。UI-04按H-OFFLINE原Windows无引擎范围保留；打包no-engine installed live仍REL-05 Not run，不能被本row关闭。

<a id="udx-030"></a>
## UDX-030 — 保存目标/快照/原子替换

当前阶段导航 **R11**（历史来源组 **A**；混合归属以意图索引为准）；条目/后继 **SGF-17**。各来源的有界调查/决定仍须先输出结论，不能把族导航视为新增实现或全部Accepted。明确的其他阶段消费者按其任务合同单独交付；其存在不阻塞独立本地工作。

<a id="ud-04-039"></a>
### UD-04-039 PR537七个真实用户修复

- **Source / event**: PR #537 (集成事件 38) | 共 2 个提交；第一父E38。
- **完整source commits（与§6反向对应）**:
  - [2d4787e63c57a0c77b43199791f7484d6195ff12](https://github.com/wimi321/lizzieyzy-next/commit/2d4787e63c57a0c77b43199791f7484d6195ff12) — fix(gui): resolve Windows user-flow QA regressions
  - [3a0b27e6fd9200f1e00bd25800ff31262ed2862c](https://github.com/wimi321/lizzieyzy-next/commit/3a0b27e6fd9200f1e00bd25800ff31262ed2862c) — QA integrate PR 537
- **发布包含**: 最早有API/ancestry证据收录 `next-2026-09-26.1`（正式发布）；全部source也含于正式09-26.1/.2。准备target仅记prepared，不由本字段推断准备版本已发布。
- **实际behavior / sourceDelta**: 2d4787e6及WINDOWS_USER_QA_20260922.md的七项，非focus修复：(1)无engine仍可改ownership-display preferences，engine-dependent action有guard；(2)无engine可Save弹chooser；(3)raw/raw-with-comments Cancel在finally恢复两global flags；(4)overwrite Cancel只恢复原same engine此前running analysis，原paused/replaced engine保持；(5)普通/raw/raw-comment/branch四save modes先normalize实际sgf target再查existing，Cancel保护目标字节；(6).SGF case-insensitive后缀不追加重复；(7)B11→B10切换后installed B11仍local candidate、saved-entry snapshot不改launch inputs、不需重新下载。
- **Canonical owner / scope**: UI-04/SGF-06/ANA-06原scope；T02-H12 raw目标；资源后继。
- **候选/覆盖与边界**: H-OFFLINE仅已有no-engineSave/编辑；H-SAVE-DEPARTURE保留不同departure Cancel行为；没有整个PR537全量native覆盖。
- **Disposition**: 窄历史覆盖；save/菜单调查；model retention后继。
- **责任意图**: T04-SAVE-TARGET、T04-OFFLINE、T04-MODEL-IDENTITY。
- **Observable assertions**: 七项逐条处置与断言见下表；ordinary target、same-engine intent、suffix和local model不能被笼统reactive UI覆盖。raw flags机制可不复制，但T02-H12未决raw用户目标仍保留，不能自称批准排除。


#### UD-04-039 七项subrecord（共同源2d4787e6，不重复算commit）

| subrecord | source实际行为 | Next处置/继承 | 责任与observable断言 |
| --- | --- | --- | --- |
| 039-a | No-engine ownership-display菜单guard | H-OFFLINE不涵盖所有ownership preferences；有界entry调查 | T04-OFFLINE：实际已支持显示prefs无engine可操作，engine-only明确disabled、不妨碍SGF。 |
| 039-b | No-engine Save chooser | H-OFFLINE的Save As/reopen窄继承；不同chooser入口未证仍调查 | T04-OFFLINE/T04-SAVE-TARGET：原no-engineSave保留，当前支持save入口不依engine。 |
| 039-c | raw/raw-comment Cancel恢复两flags | Next不存在这些Java globals不需照搬；raw用户目标仍T02-H12未决，非批准永久删除 | T04-SAVE-TARGET提供T02-H12输入：任何已准入raw选择取消后不泄漏mode，原command/config/comments不变。 |
| 039-d | Overwrite Cancel恢复原same-engine running analysis | H-SAVE-DEPARTURE是不同departure合同且普通Save while analyzing未native捕获；有界调查 | T04-SAVE-TARGET：捕获same Run与原intent，不恢复replaced/paused/new Run；保留Next明确departure后需manual restart的历史批准边界。 |
| 039-e | normalize最终target后overwrite confirm | 缺此具体继承，不以SGF-06Accepted自动close | T04-SAVE-TARGET：`protected-sgf-中文`对应existing `.sgf`；Cancel保持原目标所有bytes/当前path/dirty；普通/raw/raw-comment/branch按未来支持scope。 |
| 039-f | `.SGF`后缀case-insensitive | 缺具体inheritance，bounded target调查 | T04-SAVE-TARGET：`game.SGF`不变`game.SGF.sgf`，existing check与actual write为同一target。 |
| 039-g | installed B11切B10仍local，saved-entry snapshot不改launch inputs | local model retention后继，不是菜单focus或generic reactive Covered | T04-MODEL-IDENTITY：B11→B10→B11均可用、不重复下载，snapshot保留所有local candidates与实际active identity。 |

<a id="ud-04-045"></a>
### UD-04-045 SGF调用快照、异步调度和atomic文件替换分开

- **Source / event**: PR #543 (集成事件 38) | 共 4 个提交；第一父E38。
- **完整source commits（与§6反向对应）**:
  - [9298e061290f72f3e62a0b66dc69b26b71d1a120](https://github.com/wimi321/lizzieyzy-next/commit/9298e061290f72f3e62a0b66dc69b26b71d1a120) — test(save): type into the standard chooser filename field
  - [6bf3129d7cfc89d3bfdd9e585ba0e004d716f12a](https://github.com/wimi321/lizzieyzy-next/commit/6bf3129d7cfc89d3bfdd9e585ba0e004d716f12a) — fix(save): freeze tree structure before cloning analysis payloads
  - [948f653d04bb92409849c6dbb16fcea69618b3d8](https://github.com/wimi321/lizzieyzy-next/commit/948f653d04bb92409849c6dbb16fcea69618b3d8) — refactor(save): capture SGF snapshots and atomically write off the EDT
  - [f35af66eb7c43ecf348d3d8c26293e5e76f87598](https://github.com/wimi321/lizzieyzy-next/commit/f35af66eb7c43ecf348d3d8c26293e5e76f87598) — QA integrate PR 543
- **发布包含**: 最早有API/ancestry证据收录 `next-2026-09-26.1`（正式发布）；全部source也含于正式09-26.1/.2。准备target仅记prepared，不由本字段推断准备版本已发布。
- **实际behavior / sourceDelta**: PR543 6bf3129d先freeze结构再clone analysis负载，948f653d capture immutable SGF snapshot并off-EDT atomic write。当前Next current_game_state.rs:263–305 capture/serialize invocation snapshot为真，但persist_save_snapshot直接std::fs::write(&target,&serialized)，没有temp/rename atomic replace；lib.rs:240–247 sync Save命令，249–262仅SaveAs picker offload，不证明写盘async。
- **Canonical owner / scope**: SGF-06 Accepted save；SGF-07 Accepted safe replacement；保存后继。
- **候选/覆盖与边界**: 保留SGF-06/07原save/取消/dirty证据和H-SAVE-DEPARTURE。源码absence不是观察到数据丢失；没有atomic/large-analysis off-UI native pass。
- **Disposition**: 源码快照支持；async未证；atomic后继。
- **责任意图**: T04-SAVE-ATOMIC。
- **Observable assertions**: 后继在write/replace failure保护原existing file bytes与source path/dirty；调用时完整branch/setup/comment/analysis快照一致，后续编辑不混入；saved revision只对本快照成功更新；UI scheduling响应及实际写盘是否off-thread单独核对，不从Rust所有权推出。

<a id="udx-031"></a>
## UDX-031 — LAN发布和慢客户端有界状态

当前阶段导航 **R16**（历史来源组 **E**；混合归属以意图索引为准）；条目/后继 **PUB-01**。各来源的有界调查/决定仍须先输出结论，不能把族导航视为新增实现或全部Accepted。明确的其他阶段消费者按其任务合同单独交付；其存在不阻塞独立本地工作。

<a id="ud-04-044"></a>
### UD-04-044 WebBoard慢客户端有界state与通知合并

- **Source / event**: PR #542 (集成事件 38) | 共 3 个提交；第一父E38。
- **完整source commits（与§6反向对应）**:
  - [55ac71af0de1fdcec1288c53091cb2b2dd0fe347](https://github.com/wimi321/lizzieyzy-next/commit/55ac71af0de1fdcec1288c53091cb2b2dd0fe347) — perf(web): bound state buffering for slow websocket clients
  - [4d7f93ed6aff3b57745c445d3d40bf0697cee95a](https://github.com/wimi321/lizzieyzy-next/commit/4d7f93ed6aff3b57745c445d3d40bf0697cee95a) — perf(web): coalesce board notifications into one pending update
  - [f5f47ce6195b775aaaec3495e600ae84a70fe865](https://github.com/wimi321/lizzieyzy-next/commit/f5f47ce6195b775aaaec3495e600ae84a70fe865) — QA integrate PR 542
- **发布包含**: 最早有API/ancestry证据收录 `next-2026-09-26.1`（正式发布）；全部source也含于正式09-26.1/.2。准备target仅记prepared，不由本字段推断准备版本已发布。
- **实际behavior / sourceDelta**: 55ac71af限制慢WebSocket client state buffering，4d7f93ed coalesce board notifications；不是remote compute，不生成LAN伪Parity ID。inbound LAN不属Provider Network Policy，不扩成凭据服务。
- **Canonical owner / scope**: PUB-01 Deferred/T02-PUB-01（E）。
- **候选/覆盖与边界**: 尚无此新增行为的可继承运行证据；不把Java修复判为当前Next缺陷。
- **Disposition**: 需既有LAN功能增量。
- **责任意图**: T04-LAN。
- **Observable assertions**: 高速state更新+slow client保持每client有界pending state、最新完整board最终送达而非无限队列；快client不被慢client阻塞；Stop关闭相关连接/工作，Start/Copy URL按PUB-01，无trial counters/credentials。

<a id="udx-032"></a>
## UDX-032 — 导入日期保真调查

当前阶段导航 **R11**（历史来源组 **A**；混合归属以意图索引为准）；条目/后继 **SGF-13**。各来源的有界调查/决定仍须先输出结论，不能把族导航视为新增实现或全部Accepted。明确的其他阶段消费者按其任务合同单独交付；其存在不阻塞独立本地工作。

<a id="ud-05-11"></a>
### UD-05-11 

- 来源：[031](https://github.com/wimi321/lizzieyzy-next/commit/a49c10c77f0a28c0ad802fc9d3905d1dcabb03de), [032](https://github.com/wimi321/lizzieyzy-next/commit/238d74a46dc7f9217f55d7a2d23708049279dd42), [047](https://github.com/wimi321/lizzieyzy-next/commit/8c43fdd24b01acd6485b8b3911708290ae18ab75)（完整SHA/发布包含见源索引）。
- 用户行为/边界：导入的SGF DT保留日期表达式（部分日期、多日期、缺失），编辑/保存快照不改成今天；新建日期独立。
- Tauri映射/具名任务意图：SGF-01/SGF-09既有范围的边界调查；T05-SGF-DATE-CHECK（A）
- 覆盖证据或缺口：a49c10c7 GameInfo由Date改字符串；Tauri CurrentSgfDocument保留任意root属性，current_game.rs:3650已有DT保留测试源码；尚未运行本边界场景。
- 处置：有界调查优先A：导入/编辑/保存为当前支持路径；不能凭Java缺陷宣布Tauri丢日期，也不能凭一个DT用例关闭全部边界。
- 后续验收/调查停止条件：原生打开含完整/部分/多日期/无DT谱，元数据编辑及Save/reopen比较DT；失败时只修可达损失路径。

<a id="udx-033"></a>
## UDX-033 — 主题纹理与预设目标决定

当前阶段导航 **R17**（历史来源组 **F**；混合归属以意图索引为准）；条目/后继 **APPEAR-01**。各来源的有界调查/决定仍须先输出结论，不能把族导航视为新增实现或全部Accepted。明确的其他阶段消费者按其任务合同单独交付；其存在不阻塞独立本地工作。

<a id="ud-05-12"></a>
### UD-05-12 

- 来源：[033](https://github.com/wimi321/lizzieyzy-next/commit/7bcef4dafc9ad6360c7f891ad0c2c3929e964f63), [034](https://github.com/wimi321/lizzieyzy-next/commit/90ff66f4f12c90c4dbebe41b593f14bcc0131bb5), [047](https://github.com/wimi321/lizzieyzy-next/commit/8c43fdd24b01acd6485b8b3911708290ae18ab75)（完整SHA/发布包含见源索引）。
- 用户行为/边界：默认工作区棋盘纹理变化。
- Tauri映射/具名任务意图：主题/字体/布局后继；T05-THEME-PRESETS（F）
- 覆盖证据或缺口：PR564，资源/配置默认值变更；Tauri不采用Swing主题资源。
- 处置：保留自定义主题和默认值迁移决策，不强制照抄纹理；与02主题能力合并。
- 后续验收/调查停止条件：预设/用户主题切换可预览、取消不写盘，重启保持；旧字段白名单映射不覆盖用户现值。

<a id="udx-034"></a>
## UDX-034 — 外部落子意图与模式切换

当前阶段导航 **R14**（历史来源组 **C**；混合归属以意图索引为准）；条目/后继 **GAME-10**。各来源的有界调查/决定仍须先输出结论，不能把族导航视为新增实现或全部Accepted。明确的其他阶段消费者按其任务合同单独交付；其存在不阻塞独立本地工作。

<a id="ud-05-14"></a>
### UD-05-14 

- 来源：[041](https://github.com/wimi321/lizzieyzy-next/commit/fc94364b47f2becc8696b2294124ce6adca8ed94), [042](https://github.com/wimi321/lizzieyzy-next/commit/332bc2a43d4eb44d1fcce52ded111a76433c7114), [043](https://github.com/wimi321/lizzieyzy-next/commit/66c4e2160eca86c99718c667cdda00da45c50d42), [047](https://github.com/wimi321/lizzieyzy-next/commit/8c43fdd24b01acd6485b8b3911708290ae18ab75)（完整SHA/发布包含见源索引）。
- 用户行为/边界：同步/回滚后只恢复最新owner的引擎自动落子意图，不让旧restore重启自动行为。
- Tauri映射/具名任务意图：GAME-10、READ-02；T05-EXTERNAL-CONTINUATION（C）
- 覆盖证据或缺口：PR567；术语上Engine Continuation不是Review Autoplay或Variation Replay。
- 处置：GAME10后继，与05-13/15同权威链，不能改变纯READ02只读同步范围。
- 后续验收/调查停止条件：同步/取消/新局/手动导航穿插时无旧owner落子；仅用户已开启且当前条件合法时恢复Engine Continuation。

<a id="ud-05-15"></a>
### UD-05-15 

- 来源：[057](https://github.com/wimi321/lizzieyzy-next/commit/f14b928908d9036655cb1eef9ffe66e11d96e093), [058](https://github.com/wimi321/lizzieyzy-next/commit/53215d3f2545d88d3e9bf90f8553b3275f4a850e), [059](https://github.com/wimi321/lizzieyzy-next/commit/4f9e1964d167a35e2d009598eccaffb9d10c883f), [061](https://github.com/wimi321/lizzieyzy-next/commit/f9a0b0cced0d64909038e52a3d67b6aaf349a101), [077](https://github.com/wimi321/lizzieyzy-next/commit/0fe5b3769e5f1402013df5520643466ff021f5e6)（完整SHA/发布包含见源索引）。
- 用户行为/边界：GMA模式切换等待旧搜索输出排空、隔离在途结果、还原正确目标；手动导航取消。
- Tauri映射/具名任务意图：GAME-10；T05-EXTERNAL-MODE-SWITCH（C）
- 覆盖证据或缺口：PR570；Tauri两模式（引擎最终决策、首选候选）均是未来功能，现有READ02不提供对弈完成证据。
- 处置：后继，遵守ADR0004；不将Java drain实现当架构要求。
- 后续验收/调查停止条件：两模式切换中旧结果不落子，局面/会话变化立即失效；无精确权威确认则结束而非重试落子、undo或restart。

<a id="udx-035"></a>
## UDX-035 — Fox段位继承与显式昵称/UID调查

当前阶段导航 **R14**（历史来源组 **C**；混合归属以意图索引为准）；条目/后继 **PROV-02**。各来源的有界调查/决定仍须先输出结论，不能把族导航视为新增实现或全部Accepted。明确的其他阶段消费者按其任务合同单独交付；其存在不阻塞独立本地工作。

<a id="ud-05-16"></a>
### UD-05-16 

- 来源：[062](https://github.com/wimi321/lizzieyzy-next/commit/de4e3db7f488298f4301b8bc75c7daf10f1e78e1), [063](https://github.com/wimi321/lizzieyzy-next/commit/bb076b3b5b8fe3d99a3145e8948f40288c2e8ecf), [077](https://github.com/wimi321/lizzieyzy-next/commit/0fe5b3769e5f1402013df5520643466ff021f5e6)（完整SHA/发布包含见源索引）。
- 用户行为/边界：Fox棋谱列表职业段位编码100..108显示职业1..9段。
- Tauri映射/具名任务意图：PROV-02；T05-FOX-RANK-EVIDENCE（保留）
- 覆盖证据或缺口：Tauri crates/provider-fox/src/lib.rs:903–926已有100/108/27/16映射测试源码；DEVELOPMENT候选f1270ba713b8b6d12ef66045951d37d21f620575原生柯洁列表/158手预览。
- 处置：已有仓库实现与原生业务证据；沿用原条件不扩大至所有未知段位码。
- 后续验收/调查停止条件：后续PROV02变更时保持未知值可见和职业码边界；本轮不新增live声明。

<a id="ud-05-26"></a>
### UD-05-26 

- 来源：[115](https://github.com/wimi321/lizzieyzy-next/commit/35555e5e610c21abbe4b402d2e95be0941d39182), [116](https://github.com/wimi321/lizzieyzy-next/commit/0557fe352100b006e885977e6c58445eb5d40700), [117](https://github.com/wimi321/lizzieyzy-next/commit/2b6077c44fd637e79b6267a74c1b5bd6280d66ea), [129](https://github.com/wimi321/lizzieyzy-next/commit/1003e95a7670bedbc04bea6b9a946e34973517f9), [134](https://github.com/wimi321/lizzieyzy-next/commit/757792abafda582b8b215299b3c2ed90a9e42fbe), [136](https://github.com/wimi321/lizzieyzy-next/commit/e607c706404c0e60db115eac2ac5b39c661137b5), [141](https://github.com/wimi321/lizzieyzy-next/commit/03202192ab183a5c0a0e71e803127fbfca8d6e17), [143](https://github.com/wimi321/lizzieyzy-next/commit/b82b6611b7dc4ce92d1a0f5e2d2063b585201617)（完整SHA/发布包含见源索引）。
- 用户行为/边界：Fox全数字昵称始终按昵称解析；UID为显式独立入口；not-found与传输失败区分，Tab/Enter可选记录/历史。
- Tauri映射/具名任务意图：PROV-02；T05-FOX-IDENTITY-CHECK（C）
- 覆盖证据或缺口：35555e5e GetFoxRequest移除数字猜UID。Tauri FoxKifuCenter kind明确nickname/uid/chessid；kifu.rs parse_lookup:26–44与fetch_account_list:47–68分路，现有live记录不含数字昵称边界。
- 处置：已有结构不标Missing；有界身份/键盘边界调查，不扩大原Accepted/Partial现场范围。
- 后续验收/调查停止条件：同数字值分别以nickname/uid查询不会串身份；真实数字昵称需存在样本，缺则保留现场缺口；未知账号与网络失败不混淆。

<a id="udx-036"></a>
## UDX-036 — SGF导入容错与结果元数据调查

当前阶段导航 **R11**（历史来源组 **A**；混合归属以意图索引为准）；条目/后继 **SGF-01, SGF-07**。各来源的有界调查/决定仍须先输出结论，不能把族导航视为新增实现或全部Accepted。明确的其他阶段消费者按其任务合同单独交付；其存在不阻塞独立本地工作。

<a id="ud-05-18"></a>
### UD-05-18 

- 来源：[065](https://github.com/wimi321/lizzieyzy-next/commit/ec19f8b847ca719c0189eaaa0177592094a8208a), [066](https://github.com/wimi321/lizzieyzy-next/commit/1c829ab080f604da019c5ab14f6e6c27782b8319), [071](https://github.com/wimi321/lizzieyzy-next/commit/65cc93842eb26e0dd0ee911f3d16493b52cd6399), [077](https://github.com/wimi321/lizzieyzy-next/commit/0fe5b3769e5f1402013df5520643466ff021f5e6)（完整SHA/发布包含见源索引）。
- 用户行为/边界：各SGF入口兼容属性值之外escaped LF/CRLF，保留值内部转义；歧义/截断先拒绝；detached RE保持元数据不塞C。
- Tauri映射/具名任务意图：SGF-01/SGF-03/SGF-07；T05-SGF-IMPORT-CHECK（A）
- 覆盖证据或缺口：ec19f8b8 SGFParser normalizeStructuralNewlines与RE改动；Tauri Fox专用escapednewlines覆盖不等于本地/剪贴板所有入口，CurrentSgfDocument::open为公共支持路径。
- 处置：有界正确性调查A；保留ADR0003个人评论边界，未证明Tauri缺陷。
- 后续验收/调查停止条件：local/clipboard/provider相同样本比对树/RE/C；恶意截断拒绝且当前棋谱不变；不得全局替换破坏属性值。

<a id="udx-037"></a>
## UDX-037 — 树图/变例/预览异步身份调查

当前阶段导航 **R12/R13（逐意图）**（历史来源组 **B**；混合归属以意图索引为准）；条目/后继 **UI-02, UI-03, ANA-13**。各来源的有界调查/决定仍须先输出结论，不能把族导航视为新增实现或全部Accepted。明确的其他阶段消费者按其任务合同单独交付；其存在不阻塞独立本地工作。

<a id="ud-05-19"></a>
### UD-05-19 

- 来源：[067](https://github.com/wimi321/lizzieyzy-next/commit/ac18ae1a2b620c465aba34798dab1890a8400b6f), [068](https://github.com/wimi321/lizzieyzy-next/commit/ca74519d00672680ccdf101d47801f558da5cf20), [071](https://github.com/wimi321/lizzieyzy-next/commit/65cc93842eb26e0dd0ee911f3d16493b52cd6399), [077](https://github.com/wimi321/lizzieyzy-next/commit/0fe5b3769e5f1402013df5520643466ff021f5e6)（完整SHA/发布包含见源索引）。
- 用户行为/边界：变例树异步图像只发布当前history/node/viewport结果，点击命中与显示一致。
- Tauri映射/具名任务意图：REVIEW-01（Direct review navigation）、UI-02（Non-blocking board interaction）为现有受影响面；T05-TREE-PUBLICATION-CHECK（B）负责调查，Analysis job lanes仍归ANA-03。
- 覆盖证据或缺口：PR572；Tauri React ReviewTree与Java离屏Swing不同，已有树浏览Accepted不推断同race。
- 处置：有界调查已支持树浏览路径；无问题则记录等价边界，不复制EDT方案。
- 后续验收/调查停止条件：快速换谱/编辑/缩放/浏览后点击只选当前显示节点；旧结果不覆盖新上下文。

<a id="ud-05-21"></a>
### UD-05-21 

- 来源：[079](https://github.com/wimi321/lizzieyzy-next/commit/ce4f6e6b5af26a1e95a759fd14e2f7eab00ae608), [080](https://github.com/wimi321/lizzieyzy-next/commit/070db3f5354044b3169752acb237030910f02cdf), [081](https://github.com/wimi321/lizzieyzy-next/commit/6e2f9102a545cf0549f98521fbdc52a14d21a922), [101](https://github.com/wimi321/lizzieyzy-next/commit/8ec30daae57785f0570e279bc266a41fc1940560)（完整SHA/发布包含见源索引）。
- 用户行为/边界：候选变例第一手时仍保有输入所有权，wheel/Page不倒退实战棋谱；长变例按冻结PV导航。
- Tauri映射/具名任务意图：ANA-04（候选/PV）、UI-03（悬停预览）、ANA-13（Variation Replay）为现有受影响面；T05-VARIATION-NAV-CHECK（B）调查输入归属，不扩大既有范围。
- 覆盖证据或缺口：PR577/581/586；UI-03既有120ms等窄验收保持。Java新异步算法首步由旧整段跳到2手的兼容观察未形成Next决策。
- 处置：有界调查 + 显式交互决策，不默改旧快捷键/默认。
- 后续验收/调查停止条件：首手/末手/长PV/hover替换与Review Autoplay、Variation Replay分别验证，实际谱游标不被预览输入改写。

<a id="ud-05-24"></a>
### UD-05-24 

- 来源：[095](https://github.com/wimi321/lizzieyzy-next/commit/d57e9233e51520f982fdc6bc503f0a03b157b39b), [096](https://github.com/wimi321/lizzieyzy-next/commit/ac9b103eb08474b412dfd89966298f29fd2b865c), [097](https://github.com/wimi321/lizzieyzy-next/commit/258cfdcb7f920320123d5a8af9217bc2081f81ea), [098](https://github.com/wimi321/lizzieyzy-next/commit/3a8f4e6dfefb566a1b9fe4e1a3da626c77229d3a), [099](https://github.com/wimi321/lizzieyzy-next/commit/2f2809f50d9797e162e6ab56041570e2c9ddfbf2), [100](https://github.com/wimi321/lizzieyzy-next/commit/5ac06934b3888b17756b91152443c2b14f5d11de), [102](https://github.com/wimi321/lizzieyzy-next/commit/873b652dc1fce3812e1781d6fcb05dc65a1cc91d), [103](https://github.com/wimi321/lizzieyzy-next/commit/aa65f97e58b0397677717bea9dac33932bfe610e), [104](https://github.com/wimi321/lizzieyzy-next/commit/a361f59aae8989d9008589feb312a1bb9bc9185a), [105](https://github.com/wimi321/lizzieyzy-next/commit/dfd40d3fc0b282c3e26a100952c15a13e87ad248)（完整SHA/发布包含见源索引）。
- 用户行为/边界：主/独立/浮动预览使用冻结输入与有界后台计算；立即展示完整结果、保留兼容可见结果、owner更换撤旧，publication与replay计时器一致。
- Tauri映射/具名任务意图：ANA-04（候选/PV）、UI-03（悬停预览）、ANA-13（Variation Replay）为现有受影响面；T05-PREVIEW-ASYNC-CHECK（B）负责可达路径调查，ANA-03不是预览呈现项。
- 覆盖证据或缺口：PR586源码与修复；Java独立窗口/EDT布局不是Next架构要求，不能把Java性能提升变Tauri必需新线程。
- 处置：当前可达预览性能/所有权有界调查；仅实际证据决定是否实现。
- 后续验收/调查停止条件：长PV快速切换/隐藏/换谱/重放停止无stale发布；测量当前支持面交互与取消，保留既有延时及导航契约。

<a id="udx-038"></a>
## UDX-038 — PDA/WRN成对参数读回

当前阶段导航 **R12/R13（逐意图）**（历史来源组 **B**；混合归属以意图索引为准）；条目/后继 **ANA-22**。各来源的有界调查/决定仍须先输出结论，不能把族导航视为新增实现或全部Accepted。明确的其他阶段消费者按其任务合同单独交付；其存在不阻塞独立本地工作。

<a id="ud-05-20"></a>
### UD-05-20 

- 来源：[072](https://github.com/wimi321/lizzieyzy-next/commit/7c458bb358a8a845673a712bd7cbb9032936fe64), [074](https://github.com/wimi321/lizzieyzy-next/commit/c956c727c168e5694157b807fc7187ca111fbaf3), [077](https://github.com/wimi321/lizzieyzy-next/commit/0fe5b3769e5f1402013df5520643466ff021f5e6)（完整SHA/发布包含见源索引）。
- 用户行为/边界：PDA和WRN读回按当前reader/轮次/编号响应配对，只有两值都有效才发布；旧run/超时不伪造0。
- Tauri映射/具名任务意图：ENG-02（当前 Run/协议归属）为前置；PDA/WRN运行时读回新增责任 T05-PARAMETER-READBACK（B），不归 ANA-11 胜率图编码。
- 覆盖证据或缺口：PR574，Tauri配置/静态argv不证明runtime readback。
- 处置：后继/协议能力准入；是否可支持对应命令由真实引擎验证。
- 后续验收/调查停止条件：两值乱序、错误、非有限值、超时/重连均保持最后有效值并显示未知/失败；当前确认不得覆盖用户未提交编辑。

<a id="udx-039"></a>
## UDX-039 — 更新版本选择政策

当前阶段导航 **R18**（历史来源组 **旧R11发行**；混合归属以意图索引为准）；条目/后继 **REL-03**。各来源的有界调查/决定仍须先输出结论，不能把族导航视为新增实现或全部Accepted。明确的其他阶段消费者按其任务合同单独交付；其存在不阻塞独立本地工作。

<a id="ud-05-22"></a>
### UD-05-22 

- 来源：[085](https://github.com/wimi321/lizzieyzy-next/commit/47d0a3e84849a1672ae9688bd0ce62c5af322158), [086](https://github.com/wimi321/lizzieyzy-next/commit/e260147d59f1ec3c91e8df19c492868a04fc6588), [087](https://github.com/wimi321/lizzieyzy-next/commit/f59239e706d58d8aeb76477616491dee748867f3)（完整SHA/发布包含见源索引）。
- 用户行为/边界：Beta选择合法stable/pre候选中最新版本，稳定同版优先；选中无平台包报告NO_PACKAGE，不能降级旧版本；无效清单显式错误。
- Tauri映射/具名任务意图：REL-02；T05-UPDATE-CHANNEL-DECISION（发行阶段）
- 覆盖证据或缺口：PR583；Java已发布与更新器算法不是Next当前channel承诺。
- 处置：发行决策任务；托管与渠道未定则needs-info，功能批不被自动更新阻塞。
- 后续验收/调查停止条件：选定渠道契约后离线签名清单边界加实际发布更新验证；本轮不复制端点或虚报渠道就绪。

<a id="udx-040"></a>
## UDX-040 — 候选列表可访问性

当前阶段导航 **R12/R13（逐意图）**（历史来源组 **B**；混合归属以意图索引为准）；条目/后继 **ANA-20**。各来源的有界调查/决定仍须先输出结论，不能把族导航视为新增实现或全部Accepted。明确的其他阶段消费者按其任务合同单独交付；其存在不阻塞独立本地工作。

<a id="ud-05-23"></a>
### UD-05-23 

- 来源：[088](https://github.com/wimi321/lizzieyzy-next/commit/acb8be03dc7249d81a7c2f9a4b1893472384b704), [089](https://github.com/wimi321/lizzieyzy-next/commit/b4ab3ba7f599145dadfa5e8277e6a25b565bc53e), [090](https://github.com/wimi321/lizzieyzy-next/commit/20efc9ef68c298dc09fb15216187475abdfb376c), [091](https://github.com/wimi321/lizzieyzy-next/commit/baa677bd2b12523d5bee9ce4b252a3b07122ce33), [092](https://github.com/wimi321/lizzieyzy-next/commit/12d9fb1440a7a73916f546fe1baa3d5b6f6334a0), [093](https://github.com/wimi321/lizzieyzy-next/commit/6d601ec540d267a6832a4b495b474e6a78bbb1ff), [094](https://github.com/wimi321/lizzieyzy-next/commit/f831ad936fc5be1a0b67ba3dc4431c9c30b71049)（完整SHA/发布包含见源索引）。
- 用户行为/边界：候选列表完整行、实时刷新保留滚动位置、缩放收缩合法clamp、键盘焦点与scrollbar导航。
- Tauri映射/具名任务意图：UI-02；T05-CANDIDATE-LIST-ACCESS（F，相关新入口同步）
- 覆盖证据或缺口：PR585；Java像素行高不作为Tauri默认值。
- 处置：后继验收补面；与05-03统一UI职责。
- 后续验收/调查停止条件：实时分析不断刷新时可浏览选中行；键盘/滚动不串到棋盘或锁定预览，窄高/缩放仍完整可操作。

## 全量来源索引

每行一个唯一源commit；PR与集成来源见标题和上述source记录。全部版本列为真实已发布tag包含关系；空集合只证明本次观察未见公开包含，不从draft造出正式发行。

| Commit / 原标题 | 分片记录 | 最终Delta | 已发布包含 |
| --- | --- | --- | --- |
| [`291ab979b3e4743b6723a0c23ea4771b501146b4`](https://github.com/wimi321/lizzieyzy-next/commit/291ab979b3e4743b6723a0c23ea4771b501146b4) feat(engine): log structured startup bootstrap diagnostics (#376) | UD-03-003 | UDX-001 | next-2026-08-30.1, next-2026-08-31.1, next-2026-08-31.2, next-2026-09-01.1, next-2026-09-03.1, next-2026-09-04.1, next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`1912a9bc7730033a7aeebca1ed86876eb4ca0a7a`](https://github.com/wimi321/lizzieyzy-next/commit/1912a9bc7730033a7aeebca1ed86876eb4ca0a7a) feat(diagnostics): include thread snapshots in diagnostic ZIP (#373) | UD-03-002 | UDX-001 | next-2026-08-30.1, next-2026-08-31.1, next-2026-08-31.2, next-2026-09-01.1, next-2026-09-03.1, next-2026-09-04.1, next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`a61f66607f60b491367d40b41bebca1d78183d5b`](https://github.com/wimi321/lizzieyzy-next/commit/a61f66607f60b491367d40b41bebca1d78183d5b) fix(diagnostics): avoid sanitizer aliasing thread ids as yike rooms (#373) | UD-03-002 | UDX-001 | next-2026-08-30.1, next-2026-08-31.1, next-2026-08-31.2, next-2026-09-01.1, next-2026-09-03.1, next-2026-09-04.1, next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`db96fbc36111f5ac59934943fad1b539846e6601`](https://github.com/wimi321/lizzieyzy-next/commit/db96fbc36111f5ac59934943fad1b539846e6601) docs(diagnostics): note why thread dumps use threadId= (#373) | UD-03-002 | UDX-001 | next-2026-08-30.1, next-2026-08-31.1, next-2026-08-31.2, next-2026-09-01.1, next-2026-09-03.1, next-2026-09-04.1, next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`45a111f16a952274e46059bc1ee9d8478ae64c42`](https://github.com/wimi321/lizzieyzy-next/commit/45a111f16a952274e46059bc1ee9d8478ae64c42) docs: unify diagnostic pack guidance in troubleshooting and issue templates | UD-03-001 | UDX-001 | next-2026-08-30.1, next-2026-08-31.1, next-2026-08-31.2, next-2026-09-01.1, next-2026-09-03.1, next-2026-09-04.1, next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`a96cbf45e83f99aad32faf3249739adec56665ea`](https://github.com/wimi321/lizzieyzy-next/commit/a96cbf45e83f99aad32faf3249739adec56665ea) Merge pull request #379 from wimi321/cloud/unify-diagnostic-pack-docs-5eec | UD-03-001 | UDX-001 | next-2026-08-30.1, next-2026-08-31.1, next-2026-08-31.2, next-2026-09-01.1, next-2026-09-03.1, next-2026-09-04.1, next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`a3424b91814dae00e8b4aa4168c14b8f1880122c`](https://github.com/wimi321/lizzieyzy-next/commit/a3424b91814dae00e8b4aa4168c14b8f1880122c) Merge branch 'main' into cloud/thread-snapshot-diagnostics-ff5f | UD-03-002 | UDX-001 | next-2026-08-30.1, next-2026-08-31.1, next-2026-08-31.2, next-2026-09-01.1, next-2026-09-03.1, next-2026-09-04.1, next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`12d6786e30af2c1a6d40cee394d62e6f205b321f`](https://github.com/wimi321/lizzieyzy-next/commit/12d6786e30af2c1a6d40cee394d62e6f205b321f) Merge pull request #381 from wimi321/cloud/thread-snapshot-diagnostics-ff5f | UD-03-002 | UDX-001 | next-2026-08-30.1, next-2026-08-31.1, next-2026-08-31.2, next-2026-09-01.1, next-2026-09-03.1, next-2026-09-04.1, next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`2a7b5e36a2acca723f157f9a958ec382a9a706b7`](https://github.com/wimi321/lizzieyzy-next/commit/2a7b5e36a2acca723f157f9a958ec382a9a706b7) Merge branch 'main' into cloud/engine-startup-bootstrap-376-9424 | UD-03-003 | UDX-001 | next-2026-08-30.1, next-2026-08-31.1, next-2026-08-31.2, next-2026-09-01.1, next-2026-09-03.1, next-2026-09-04.1, next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`56cc54bc1c8d88de6093fc63659f519abdd2c472`](https://github.com/wimi321/lizzieyzy-next/commit/56cc54bc1c8d88de6093fc63659f519abdd2c472) Merge pull request #382 from wimi321/cloud/engine-startup-bootstrap-376-9424 | UD-03-003 | UDX-001 | next-2026-08-30.1, next-2026-08-31.1, next-2026-08-31.2, next-2026-09-01.1, next-2026-09-03.1, next-2026-09-04.1, next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`34f395bc2c2061d431c4443af6ebf00659d1152e`](https://github.com/wimi321/lizzieyzy-next/commit/34f395bc2c2061d431c4443af6ebf00659d1152e) feat(diagnostics): keep bounded GTP probe stderr (#375) | UD-03-005 | UDX-001 | next-2026-08-30.1, next-2026-08-31.1, next-2026-08-31.2, next-2026-09-01.1, next-2026-09-03.1, next-2026-09-04.1, next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`10ba23424f3cf381c019c07bc81f759da2ef3d67`](https://github.com/wimi321/lizzieyzy-next/commit/10ba23424f3cf381c019c07bc81f759da2ef3d67) fix(diagnostics): classify probe early-exit after stdout EOF | UD-03-005 | UDX-001 | next-2026-08-30.1, next-2026-08-31.1, next-2026-08-31.2, next-2026-09-01.1, next-2026-09-03.1, next-2026-09-04.1, next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`588e18beb9d1776516f644c257ed79a38c533827`](https://github.com/wimi321/lizzieyzy-next/commit/588e18beb9d1776516f644c257ed79a38c533827) feat(diagnostics): add JVM memory and disk snapshot to diagnostic pack (#374) | UD-03-004 | UDX-001 | next-2026-08-30.1, next-2026-08-31.1, next-2026-08-31.2, next-2026-09-01.1, next-2026-09-03.1, next-2026-09-04.1, next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`c74eec0a51d090630b7516a610b8133606a0d9c4`](https://github.com/wimi321/lizzieyzy-next/commit/c74eec0a51d090630b7516a610b8133606a0d9c4) Merge pull request #383 from wimi321/cloud/runtime-snapshot-diagnostics-b51b | UD-03-004 | UDX-001 | next-2026-08-30.1, next-2026-08-31.1, next-2026-08-31.2, next-2026-09-01.1, next-2026-09-03.1, next-2026-09-04.1, next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`424e2d1d5755fdf69b3c7e0a014a7427fead40ca`](https://github.com/wimi321/lizzieyzy-next/commit/424e2d1d5755fdf69b3c7e0a014a7427fead40ca) Merge branch 'main' into cloud/gtp-probe-stderr-diagnostics-1cf0 | UD-03-005 | UDX-001 | next-2026-08-30.1, next-2026-08-31.1, next-2026-08-31.2, next-2026-09-01.1, next-2026-09-03.1, next-2026-09-04.1, next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`26808a5291991e9647b91a08dec686ba7bfefbc2`](https://github.com/wimi321/lizzieyzy-next/commit/26808a5291991e9647b91a08dec686ba7bfefbc2) Merge pull request #384 from wimi321/cloud/gtp-probe-stderr-diagnostics-1cf0 | UD-03-005 | UDX-001 | next-2026-08-30.1, next-2026-08-31.1, next-2026-08-31.2, next-2026-09-01.1, next-2026-09-03.1, next-2026-09-04.1, next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`6c82562434c195bf4ba5ed6659f4396805c31e44`](https://github.com/wimi321/lizzieyzy-next/commit/6c82562434c195bf4ba5ed6659f4396805c31e44) Merge pull request #385 from wimi321/cloud/maintenance-lifecycle-diagnostics-be61 | UD-03-006 | UDX-002 | next-2026-08-30.1, next-2026-08-31.1, next-2026-08-31.2, next-2026-09-01.1, next-2026-09-03.1, next-2026-09-04.1, next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`5df03ac20a4112b51aa375de57cc6b0eeddf4159`](https://github.com/wimi321/lizzieyzy-next/commit/5df03ac20a4112b51aa375de57cc6b0eeddf4159) docs(readme): streamline multilingual project guides (#386) | UD-03-EXC-001 | UDX-023 | next-2026-08-30.1, next-2026-08-31.1, next-2026-08-31.2, next-2026-09-01.1, next-2026-09-03.1, next-2026-09-04.1, next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`714e032279996616f2fe087e3db6c59e0dc95b6d`](https://github.com/wimi321/lizzieyzy-next/commit/714e032279996616f2fe087e3db6c59e0dc95b6d) Improve AI Coach tactical move safety | UD-03-007 | UDX-003 | next-2026-08-30.1, next-2026-08-31.1, next-2026-08-31.2, next-2026-09-01.1, next-2026-09-03.1, next-2026-09-04.1, next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`b164ad6ceea1e9866184d83d2d20abb2fa08cb16`](https://github.com/wimi321/lizzieyzy-next/commit/b164ad6ceea1e9866184d83d2d20abb2fa08cb16) Adapt AI Coach search depth to think time | UD-03-007 | UDX-003 | next-2026-08-30.1, next-2026-08-31.1, next-2026-08-31.2, next-2026-09-01.1, next-2026-09-03.1, next-2026-09-04.1, next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`89cf4731016c01fa1d7deffb45ba769ee3064c30`](https://github.com/wimi321/lizzieyzy-next/commit/89cf4731016c01fa1d7deffb45ba769ee3064c30) Merge pull request #388 from wimi321/fix/humansl-tactical-quality-guard | UD-03-007 | UDX-003 | next-2026-08-30.1, next-2026-08-31.1, next-2026-08-31.2, next-2026-09-01.1, next-2026-09-03.1, next-2026-09-04.1, next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`7dad1f1525a1c33a3709460bbd38ad9b98566cf6`](https://github.com/wimi321/lizzieyzy-next/commit/7dad1f1525a1c33a3709460bbd38ad9b98566cf6) release: request next-2026-08-30.1 pre-release | UD-03-EXC-004 | UDX-023 | next-2026-08-30.1, next-2026-08-31.1, next-2026-08-31.2, next-2026-09-01.1, next-2026-09-03.1, next-2026-09-04.1, next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`aa77b7f437aab9603baf2a066d7614027ed3336a`](https://github.com/wimi321/lizzieyzy-next/commit/aa77b7f437aab9603baf2a066d7614027ed3336a) Merge pull request #389 from wimi321/release/next-2026-08-30.1 | UD-03-EXC-004 | UDX-023 | next-2026-08-30.1, next-2026-08-31.1, next-2026-08-31.2, next-2026-09-01.1, next-2026-09-03.1, next-2026-09-04.1, next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`8d56d7df8d1566e4a0f931341ef14a288d29e809`](https://github.com/wimi321/lizzieyzy-next/commit/8d56d7df8d1566e4a0f931341ef14a288d29e809) fix(tensorrt): unblock repair from DirectML profiles | UD-03-008 | UDX-002 | next-2026-08-31.1, next-2026-08-31.2, next-2026-09-01.1, next-2026-09-03.1, next-2026-09-04.1, next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`9199c3e5b5c0c13ea9fee55c4d19f4b39657ca85`](https://github.com/wimi321/lizzieyzy-next/commit/9199c3e5b5c0c13ea9fee55c4d19f4b39657ca85) fix(tensorrt): preserve recovery artifacts | UD-03-008 | UDX-002 | next-2026-08-31.1, next-2026-08-31.2, next-2026-09-01.1, next-2026-09-03.1, next-2026-09-04.1, next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`b2227aaa086f8fda797646bc9bb23c71e0a384a7`](https://github.com/wimi321/lizzieyzy-next/commit/b2227aaa086f8fda797646bc9bb23c71e0a384a7) test(logging): stabilize crash persistence barrier coverage (#393) | UD-03-EXC-003 | UDX-023 | next-2026-08-31.1, next-2026-08-31.2, next-2026-09-01.1, next-2026-09-03.1, next-2026-09-04.1, next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`4fd12153edf6da20174b9fa1d8e62bffbab19bb8`](https://github.com/wimi321/lizzieyzy-next/commit/4fd12153edf6da20174b9fa1d8e62bffbab19bb8) ci: share local and hosted preflight gates (#394) | UD-03-EXC-002 | UDX-023 | next-2026-08-31.1, next-2026-08-31.2, next-2026-09-01.1, next-2026-09-03.1, next-2026-09-04.1, next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`6ebd2b41266ac52a8cff70bec221a97e59790404`](https://github.com/wimi321/lizzieyzy-next/commit/6ebd2b41266ac52a8cff70bec221a97e59790404) Merge branch 'main' into fix/issue-380-tensorrt-repair | UD-03-008 | UDX-002 | next-2026-08-31.1, next-2026-08-31.2, next-2026-09-01.1, next-2026-09-03.1, next-2026-09-04.1, next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`0752ce50a78d52f57da9388240142340f590f0c2`](https://github.com/wimi321/lizzieyzy-next/commit/0752ce50a78d52f57da9388240142340f590f0c2) Merge pull request #392 from qiyi71w/fix/issue-380-tensorrt-repair | UD-03-008 | UDX-002 | next-2026-08-31.1, next-2026-08-31.2, next-2026-09-01.1, next-2026-09-03.1, next-2026-09-04.1, next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`3dea2ffbabdd5735fe499098296672853c0a0a33`](https://github.com/wimi321/lizzieyzy-next/commit/3dea2ffbabdd5735fe499098296672853c0a0a33) test(windows): handle unavailable symbolic links (#395) | UD-03-EXC-003 | UDX-023 | next-2026-08-31.1, next-2026-08-31.2, next-2026-09-01.1, next-2026-09-03.1, next-2026-09-04.1, next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`22128b6e2f9cba91701f157e8c6af2d205640a4f`](https://github.com/wimi321/lizzieyzy-next/commit/22128b6e2f9cba91701f157e8c6af2d205640a4f) fix(ai-coach): restore analysis and settings safely (#371) | UD-03-009 | UDX-003 | next-2026-08-31.1, next-2026-08-31.2, next-2026-09-01.1, next-2026-09-03.1, next-2026-09-04.1, next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`36535440c58bc4085c4309a3d99254589240e9b0`](https://github.com/wimi321/lizzieyzy-next/commit/36535440c58bc4085c4309a3d99254589240e9b0) fix(tensorrt): gate repair by NVIDIA hardware (#396) | UD-03-010 | UDX-002 | next-2026-08-31.1, next-2026-08-31.2, next-2026-09-01.1, next-2026-09-03.1, next-2026-09-04.1, next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`ef2319488868b42e4cb3041bf1ded1936f5abb90`](https://github.com/wimi321/lizzieyzy-next/commit/ef2319488868b42e4cb3041bf1ded1936f5abb90) fix: harden Windows user acceptance regressions (#397) | UD-03-011 | UDX-004 | next-2026-08-31.1, next-2026-08-31.2, next-2026-09-01.1, next-2026-09-03.1, next-2026-09-04.1, next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`f533b9109195a8f610276b1179dfa460979c0ed1`](https://github.com/wimi321/lizzieyzy-next/commit/f533b9109195a8f610276b1179dfa460979c0ed1) release: request next-2026-08-31.1 pre-release (#398) | UD-03-EXC-004 | UDX-023 | next-2026-08-31.1, next-2026-08-31.2, next-2026-09-01.1, next-2026-09-03.1, next-2026-09-04.1, next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`7542c6ff8e537937851b1c159c84f403f980abec`](https://github.com/wimi321/lizzieyzy-next/commit/7542c6ff8e537937851b1c159c84f403f980abec) ui: redesign AI commentary workspace (#399) | UD-03-012 | UDX-005 | next-2026-08-31.2, next-2026-09-01.1, next-2026-09-03.1, next-2026-09-04.1, next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`fbac79e5ef839f656bafc70ded3cab119842d141`](https://github.com/wimi321/lizzieyzy-next/commit/fbac79e5ef839f656bafc70ded3cab119842d141) release: request next-2026-08-31.2 pre-release (#400) | UD-03-EXC-004 | UDX-023 | next-2026-08-31.2, next-2026-09-01.1, next-2026-09-03.1, next-2026-09-04.1, next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`e6ad02af0454413512bce6f2102f0c66e8424144`](https://github.com/wimi321/lizzieyzy-next/commit/e6ad02af0454413512bce6f2102f0c66e8424144) Merge pull request #401 from qiyi71w/feat/issue-387-winrate-graph-readability | UD-03-013 | UDX-006 | next-2026-09-01.1, next-2026-09-03.1, next-2026-09-04.1, next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`81152eeaade28e114aeb414114e1684c829d9595`](https://github.com/wimi321/lizzieyzy-next/commit/81152eeaade28e114aeb414114e1684c829d9595) fix: stabilize kifu switching and automatic analysis (#402) | UD-03-014 | UDX-007 | next-2026-09-01.1, next-2026-09-03.1, next-2026-09-04.1, next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`b5050a2937292de073b1de660aef57ec7c04058f`](https://github.com/wimi321/lizzieyzy-next/commit/b5050a2937292de073b1de660aef57ec7c04058f) feat: show NN and search speed in performance optimization (#403) | UD-03-015 | UDX-008 | next-2026-09-01.1, next-2026-09-03.1, next-2026-09-04.1, next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`cecbc4328b7f4cce2b47a77910949f64c7df9ac5`](https://github.com/wimi321/lizzieyzy-next/commit/cecbc4328b7f4cce2b47a77910949f64c7df9ac5) release: request next-2026-09-01.1 pre-release (#404) | UD-03-EXC-004 | UDX-023 | next-2026-09-01.1, next-2026-09-03.1, next-2026-09-04.1, next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`b9b3fc88c5c3bdfd399214410c5053486f1b099b`](https://github.com/wimi321/lizzieyzy-next/commit/b9b3fc88c5c3bdfd399214410c5053486f1b099b) fix: skip automatic benchmark for remote engines (#408) | UD-03-016 | UDX-009 | next-2026-09-03.1, next-2026-09-04.1, next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`55384424f797bbf7d39f2215a173b106ef175b00`](https://github.com/wimi321/lizzieyzy-next/commit/55384424f797bbf7d39f2215a173b106ef175b00) fix: make benchmark feedback accurate and accessible (#409) | UD-03-017 | UDX-008 | next-2026-09-03.1, next-2026-09-04.1, next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`bfa4759805296b6107e7720810b3195957459c53`](https://github.com/wimi321/lizzieyzy-next/commit/bfa4759805296b6107e7720810b3195957459c53) Prevent engine presentation deadlocks (#410) | UD-03-018 | UDX-004 | next-2026-09-03.1, next-2026-09-04.1, next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`3327b7c46f66bc745ba7e7bf118b3c454d1c1676`](https://github.com/wimi321/lizzieyzy-next/commit/3327b7c46f66bc745ba7e7bf118b3c454d1c1676) fix: make logging recovery generation-aware (#411) | UD-03-019 | UDX-001 | next-2026-09-03.1, next-2026-09-04.1, next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`9a1fd30c0e4d45efccb5d44ac178e6d1fc14c8e2`](https://github.com/wimi321/lizzieyzy-next/commit/9a1fd30c0e4d45efccb5d44ac178e6d1fc14c8e2) fix(gui): restore live analysis limits on Engine settings (#412) | UD-03-020 | UDX-010 | next-2026-09-03.1, next-2026-09-04.1, next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`69843b834a5a0e2b087f332073f40feab7dfd5d1`](https://github.com/wimi321/lizzieyzy-next/commit/69843b834a5a0e2b087f332073f40feab7dfd5d1) fix(gui): restore graph navigation for unanalyzed moves (#413) | UD-03-021 | UDX-006 | next-2026-09-03.1, next-2026-09-04.1, next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`d7edb3296ca688ecf716aa68a19f6475929cf541`](https://github.com/wimi321/lizzieyzy-next/commit/d7edb3296ca688ecf716aa68a19f6475929cf541) fix(gui): keep manual analysis pause authoritative (#415) | UD-03-022 | UDX-011 | next-2026-09-03.1, next-2026-09-04.1, next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`e883d575d7534717dba30fe6b3b1fc3eaaf260d1`](https://github.com/wimi321/lizzieyzy-next/commit/e883d575d7534717dba30fe6b3b1fc3eaaf260d1) 修复 Windows 棋谱分析、弈客同步与一键设置卡顿 (#416) | UD-03-023 | UDX-007 | next-2026-09-03.1, next-2026-09-04.1, next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`de4855ea5902d4871207c42ba2e2e52cb465d9ff`](https://github.com/wimi321/lizzieyzy-next/commit/de4855ea5902d4871207c42ba2e2e52cb465d9ff) 发布 next-2026-09-03.1 多平台预发布版 (#417) | UD-03-EXC-004 | UDX-023 | next-2026-09-03.1, next-2026-09-04.1, next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`f64c2767cbe80f6aa92d041eae408003b68e5186`](https://github.com/wimi321/lizzieyzy-next/commit/f64c2767cbe80f6aa92d041eae408003b68e5186) 优化自动分析切换与整盘精析搜索设置 (#418) | UD-03-024 | UDX-011 | next-2026-09-04.1, next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`e23cf300ae65ce0729c0fc589c52492c89961b6c`](https://github.com/wimi321/lizzieyzy-next/commit/e23cf300ae65ce0729c0fc589c52492c89961b6c) 发布 next-2026-09-04.1 预览版 (#419) | UD-03-EXC-004 | UDX-023 | next-2026-09-04.1, next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`1f37094360690ac59eefb038067beeaf38d90d84`](https://github.com/wimi321/lizzieyzy-next/commit/1f37094360690ac59eefb038067beeaf38d90d84) Correct TensorRT guidance in stable catalog | UD-03-010a, UD-03-EXC-005 | UDX-002, UDX-023 | next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`2f00081d1cff16a3625a4b1c44078b375bf28fdf`](https://github.com/wimi321/lizzieyzy-next/commit/2f00081d1cff16a3625a4b1c44078b375bf28fdf) Merge pull request #420 from wimi321/fix/stable-tensorrt-catalog-labels | UD-03-010a, UD-03-EXC-005 | UDX-002, UDX-023 | next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`237cbc0073da3bcc6b6115b95fe4f9a0f62c0b93`](https://github.com/wimi321/lizzieyzy-next/commit/237cbc0073da3bcc6b6115b95fe4f9a0f62c0b93) feat(diagnostics): analysis-cache Full Trace decisions (#424) (#425) | UD-03-025 | UDX-001 | next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`8750dfb4b82b24ef94c59d3861b124ce981ce4f3`](https://github.com/wimi321/lizzieyzy-next/commit/8750dfb4b82b24ef94c59d3861b124ce981ce4f3) fix(gui): prevent rules dialogs from interrupting engine games (#426) | UD-03-026 | UDX-012 | next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`0c75fd540ab93f0e6734e13b777d15451d1b71cf`](https://github.com/wimi321/lizzieyzy-next/commit/0c75fd540ab93f0e6734e13b777d15451d1b71cf) fix(engine): retire streaming genmove safely and restore foreground state (#428) | UD-03-027 | UDX-013 | next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`554fdebd4f4bb3c2aaf2a22cd1e8ef1650d02883`](https://github.com/wimi321/lizzieyzy-next/commit/554fdebd4f4bb3c2aaf2a22cd1e8ef1650d02883) fix(engine): support custom KataGo benchmark tasks (#434) | UD-03-028 | UDX-008 | next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`fe4af8678dab4bc6bfca72faac71d533dd4ff584`](https://github.com/wimi321/lizzieyzy-next/commit/fe4af8678dab4bc6bfca72faac71d533dd4ff584) feat(enginegame): verify and restore match rules (#435) | UD-03-029 | UDX-014 | next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`1d6bb41ffa40b0efc8a31c5a348f8805d4324d0a`](https://github.com/wimi321/lizzieyzy-next/commit/1d6bb41ffa40b0efc8a31c5a348f8805d4324d0a) fix(logging): preserve exported log record boundaries (#436) | UD-03-030 | UDX-001 | next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`0a0d44f0906bbd79b462b79ef540caff295092ba`](https://github.com/wimi321/lizzieyzy-next/commit/0a0d44f0906bbd79b462b79ef540caff295092ba) fix(diagnostics): bound export work and unblock completion (#438) | UD-03-031 | UDX-001 | next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`860caa60c8173920ec9456dc6e405f1108f83a16`](https://github.com/wimi321/lizzieyzy-next/commit/860caa60c8173920ec9456dc6e405f1108f83a16) fix(analysis): confirm positions before sync analysis (#439) | UD-03-032 | UDX-015 | next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`d92ede3ab9e36c3ed130064f556db43f56f872be`](https://github.com/wimi321/lizzieyzy-next/commit/d92ede3ab9e36c3ed130064f556db43f56f872be) test(enginegame): handle localized SGF rule markers | UD-03-EXC-003 | UDX-023 | next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`e5ea09bc633c2312af86382f5904389d5daac359`](https://github.com/wimi321/lizzieyzy-next/commit/e5ea09bc633c2312af86382f5904389d5daac359) Merge pull request #441 from wimi321/test/localized-sgf-rule-markers | UD-03-EXC-003 | UDX-023 | next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`a22d252647c2363c7b3b4e0d3fcc3bb1f94b977d`](https://github.com/wimi321/lizzieyzy-next/commit/a22d252647c2363c7b3b4e0d3fcc3bb1f94b977d) test(engine): gate startup cancellation deterministically | UD-03-EXC-003 | UDX-023 | next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`f4df03062f7662ca7979217d5f16915f6fcd7659`](https://github.com/wimi321/lizzieyzy-next/commit/f4df03062f7662ca7979217d5f16915f6fcd7659) Merge pull request #442 from wimi321/test/startup-cancel-response-gate | UD-03-EXC-003 | UDX-023 | next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`8e0d33a67c9dd7f94c02ae7df7aed9136fef6ab1`](https://github.com/wimi321/lizzieyzy-next/commit/8e0d33a67c9dd7f94c02ae7df7aed9136fef6ab1) Add one-click setup guidance for custom compute | UD-03-033 | UDX-009 | next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`4477b7c23bf37bd43077e84d9f0d9c2125b3bc49`](https://github.com/wimi321/lizzieyzy-next/commit/4477b7c23bf37bd43077e84d9f0d9c2125b3bc49) Polish constrained custom compute guidance | UD-03-033 | UDX-009 | next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`92dc357e41cd63ced1c69603139507494822d8fe`](https://github.com/wimi321/lizzieyzy-next/commit/92dc357e41cd63ced1c69603139507494822d8fe) Merge pull request #427 from wimi321/feat/remote-compute-one-click-help | UD-03-033 | UDX-009 | next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`3c63881300f6d670237651e31aa6958028d3fd41`](https://github.com/wimi321/lizzieyzy-next/commit/3c63881300f6d670237651e31aa6958028d3fd41) fix(sync): retire local confirmations on stop (#440) | UD-03-034 | UDX-016 | next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`01c259083b30b3e721242998fd9c582b8b11de1e`](https://github.com/wimi321/lizzieyzy-next/commit/01c259083b30b3e721242998fd9c582b8b11de1e) fix(analysis): preserve exact KataGo root metrics | UD-03-054, UD-03-054c | UDX-022 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`5a33cf560d2563a94e537f7501d088030e3adcfd`](https://github.com/wimi321/lizzieyzy-next/commit/5a33cf560d2563a94e537f7501d088030e3adcfd) fix(readboard): restore evaluation after navigation | UD-03-054, UD-03-054d | UDX-022 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`7451ddc1dddbf4650be221da2fe303bccb9e4f33`](https://github.com/wimi321/lizzieyzy-next/commit/7451ddc1dddbf4650be221da2fe303bccb9e4f33) Merge commit '5a33cf560d2563a94e537f7501d088030e3adcfd' into feat/414-move-focus | UD-03-054 | UDX-022 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`deb208d6f45f020c9b30b6033c780e213bd70347`](https://github.com/wimi321/lizzieyzy-next/commit/deb208d6f45f020c9b30b6033c780e213bd70347) feat(analysis): add same-tree move focus | UD-03-054, UD-03-054a, UD-03-054b, UD-03-054c, UD-03-054d, UD-03-054e | UDX-022 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`25a984dd6f4ed7282092970d32f087da5847ad57`](https://github.com/wimi321/lizzieyzy-next/commit/25a984dd6f4ed7282092970d32f087da5847ad57) fix(i18n): simplify point evaluation menu labels | UD-03-054, UD-03-054e | UDX-022 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`f9e00b9e202b40350cc9246b5a64e71d3fe60af8`](https://github.com/wimi321/lizzieyzy-next/commit/f9e00b9e202b40350cc9246b5a64e71d3fe60af8) Accelerate HumanSL downloads with verified R2 mirror | UD-03-035 | UDX-002 | next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`c5f533e93d1a74d03660abed016db6ff471422fd`](https://github.com/wimi321/lizzieyzy-next/commit/c5f533e93d1a74d03660abed016db6ff471422fd) Keep unrelated engine tests formatting unchanged | UD-03-035 | UDX-002 | next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`a06be4cad8aba89d3d705561874bd0a36cc49875`](https://github.com/wimi321/lizzieyzy-next/commit/a06be4cad8aba89d3d705561874bd0a36cc49875) Merge pull request #447 from wimi321/feat/humansl-r2-download | UD-03-035 | UDX-002 | next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`d25e18355dba14b47f082c6c0b351c6f1bb4b439`](https://github.com/wimi321/lizzieyzy-next/commit/d25e18355dba14b47f082c6c0b351c6f1bb4b439) feat(engine): scope KataGo tuning to saved engines (#450) | UD-03-036 | UDX-017 | next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`fe95700e739484cc8dcf843943c2d19c212ed164`](https://github.com/wimi321/lizzieyzy-next/commit/fe95700e739484cc8dcf843943c2d19c212ed164) feat(models): 更新默认 B11 为 2026-09-07 官方模型 (#451) | UD-03-037 | UDX-002 | next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`19b6d9877fa2b0e7c55aeb646f06b1c7836a2c42`](https://github.com/wimi321/lizzieyzy-next/commit/19b6d9877fa2b0e7c55aeb646f06b1c7836a2c42) Merge main into move-focus integration and preserve user pause intent | UD-03-054 | UDX-022 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`ae535babca34b7a3a35931c7a35731e6b840c9dd`](https://github.com/wimi321/lizzieyzy-next/commit/ae535babca34b7a3a35931c7a35731e6b840c9dd) test(engine): deliver rollback responses on an independent reader | UD-03-054, UD-03-054f | UDX-022 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`e0dd75541994724ce1a18e6a3c25dae36f32f1f8`](https://github.com/wimi321/lizzieyzy-next/commit/e0dd75541994724ce1a18e6a3c25dae36f32f1f8) test(engine): serialize late rollback acknowledgements and timeout checks | UD-03-054, UD-03-054f | UDX-022 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`b56540aaa5a1750209cbcd0939d08f8c6fbe68dd`](https://github.com/wimi321/lizzieyzy-next/commit/b56540aaa5a1750209cbcd0939d08f8c6fbe68dd) fix: retry focus capability detection after pause or board changes | UD-03-054, UD-03-054a | UDX-022 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`8cb4153fd12850ac3068f6118353dc25a8951115`](https://github.com/wimi321/lizzieyzy-next/commit/8cb4153fd12850ac3068f6118353dc25a8951115) fix: serialize deferred focus probes without blocking pause | UD-03-054, UD-03-054a | UDX-022 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`7865da5c5d1b004690e3cab1e1a555090e353e89`](https://github.com/wimi321/lizzieyzy-next/commit/7865da5c5d1b004690e3cab1e1a555090e353e89) fix: dispatch UI capability probes off the event thread | UD-03-054, UD-03-054a | UDX-022 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`e044191a0ed28b398fcfcefe6c539415e4060f3c`](https://github.com/wimi321/lizzieyzy-next/commit/e044191a0ed28b398fcfcefe6c539415e4060f3c) fix(sgf): confirm rules before resuming analysis (#452) | UD-03-038 | UDX-015 | next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`a204afe2c6a1749b51cf4586e9ca59018d1a7f7f`](https://github.com/wimi321/lizzieyzy-next/commit/a204afe2c6a1749b51cf4586e9ca59018d1a7f7f) fix(autosetup): restore responsive TensorRT groups (#453) | UD-03-039 | UDX-002 | next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`c1408902f27cd1d41bf543a4af05e9161991ee1e`](https://github.com/wimi321/lizzieyzy-next/commit/c1408902f27cd1d41bf543a4af05e9161991ee1e) ci: split checks into responsibility groups | UD-03-EXC-002 | UDX-023 | next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`73ff458877cfab79a2f15107c821c536a7ac0333`](https://github.com/wimi321/lizzieyzy-next/commit/73ff458877cfab79a2f15107c821c536a7ac0333) docs(qa): define specialized acceptance gates | UD-03-EXC-002 | UDX-023 | next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`1a075a9bbd116d08ce443c02aac6e67d36254080`](https://github.com/wimi321/lizzieyzy-next/commit/1a075a9bbd116d08ce443c02aac6e67d36254080) docs(qa): use public candidate build instructions | UD-03-EXC-002 | UDX-023 | next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`8dfe02b6f67159ee5f3805ab84067801e604063c`](https://github.com/wimi321/lizzieyzy-next/commit/8dfe02b6f67159ee5f3805ab84067801e604063c) ci: remove legacy aggregate gates | UD-03-EXC-002 | UDX-023 | next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`062a8cf461bc2adf3e91112e542c8feb7ed3e463`](https://github.com/wimi321/lizzieyzy-next/commit/062a8cf461bc2adf3e91112e542c8feb7ed3e463) Merge pull request #454 from qiyi71w/plan/ci-responsibility-split | UD-03-EXC-002 | UDX-023 | next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`6caf46b26f7486e6ffb63ffe20c384032ac31e36`](https://github.com/wimi321/lizzieyzy-next/commit/6caf46b26f7486e6ffb63ffe20c384032ac31e36) chore: sync main for CI gate cleanup | UD-03-EXC-002 | UDX-023 | next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`e3b6d918e203dfb97312013191bc8e3364886680`](https://github.com/wimi321/lizzieyzy-next/commit/e3b6d918e203dfb97312013191bc8e3364886680) Merge pull request #455 from qiyi71w/plan/ci-responsibility-split | UD-03-EXC-002 | UDX-023 | next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`441cf0357848ceedae4666afeb283e872b99d71d`](https://github.com/wimi321/lizzieyzy-next/commit/441cf0357848ceedae4666afeb283e872b99d71d) release: prepare next-2026-09-13.1 pre-release | UD-03-EXC-004 | UDX-023 | next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`34b6dbb9a69f59f5493d79ab65328f02d8865ab5`](https://github.com/wimi321/lizzieyzy-next/commit/34b6dbb9a69f59f5493d79ab65328f02d8865ab5) test(engine): await failed startup settlement before retry | UD-03-EXC-003 | UDX-023 | next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`e8d0dc9434d03b1bc3c20ee67a5fa8e4c9fda42c`](https://github.com/wimi321/lizzieyzy-next/commit/e8d0dc9434d03b1bc3c20ee67a5fa8e4c9fda42c) Merge pull request #456 from wimi321/release/next-2026-09-13.1 | UD-03-EXC-004 | UDX-023 | next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`56268282f3793c1d6fbcfadd921b9ced78ae041f`](https://github.com/wimi321/lizzieyzy-next/commit/56268282f3793c1d6fbcfadd921b9ced78ae041f) test(logging): await durable recovery before publishing pre-release | UD-03-EXC-003 | UDX-023 | next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`c1857c2ea017ef446105c6446ea1e555978c912c`](https://github.com/wimi321/lizzieyzy-next/commit/c1857c2ea017ef446105c6446ea1e555978c912c) Merge pull request #457 from wimi321/release/next-2026-09-13.2 | UD-03-EXC-004 | UDX-023 | next-2026-09-13.2, next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`a1dcc8c992ed2691924bad747cb06bad98d89acb`](https://github.com/wimi321/lizzieyzy-next/commit/a1dcc8c992ed2691924bad747cb06bad98d89acb) ci(windows): capture stalled test diagnostics | UD-03-EXC-002 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`75736abf7d4726701c34590dac59f7a3b06b7a04`](https://github.com/wimi321/lizzieyzy-next/commit/75736abf7d4726701c34590dac59f7a3b06b7a04) Merge pull request #459 from qiyi71w/ci/windows-stall-evidence | UD-03-EXC-002 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`839f9976fa32310afe378038b6caf7ea03f47cf1`](https://github.com/wimi321/lizzieyzy-next/commit/839f9976fa32310afe378038b6caf7ea03f47cf1) Merge pull request #458 from qiyi71w/plan/function-search-navigation | UD-03-040 | UDX-018 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`8e7eebcb6a362d9ca9f4ffd168fa1a366d8999e7`](https://github.com/wimi321/lizzieyzy-next/commit/8e7eebcb6a362d9ca9f4ffd168fa1a366d8999e7) fix(ui): keep function search in the toolbar | UD-03-043 | UDX-018 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`bcde6ac92e1d0f4b04a0c701a456b1555a6c4d82`](https://github.com/wimi321/lizzieyzy-next/commit/bcde6ac92e1d0f4b04a0c701a456b1555a6c4d82) docs(search): align localized entry points | UD-03-043 | UDX-018 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`0b858f9562ba246f448a11d30ede7786a0c7b9b8`](https://github.com/wimi321/lizzieyzy-next/commit/0b858f9562ba246f448a11d30ede7786a0c7b9b8) fix(gtp): preserve command-list framing | UD-03-041 | UDX-011 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`c6eb5b8bfb268f6b2827fdccdba019f39ba2fe72`](https://github.com/wimi321/lizzieyzy-next/commit/c6eb5b8bfb268f6b2827fdccdba019f39ba2fe72) fix(analysis): resume after automatic quick scan | UD-03-041 | UDX-011 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`2f4b216aa5d876c08443e0c4a0d5f3a01bb983f6`](https://github.com/wimi321/lizzieyzy-next/commit/2f4b216aa5d876c08443e0c4a0d5f3a01bb983f6) fix(engine): avoid failure presentation deadlock | UD-03-041 | UDX-011 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`88715db53c546d750ebba65370da0db54319f686`](https://github.com/wimi321/lizzieyzy-next/commit/88715db53c546d750ebba65370da0db54319f686) Merge pull request #460 from qiyi71w/fix/automatic-sgf-quick-analysis | UD-03-041 | UDX-011 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`92aa33430479e7906c002d726328298af4d50483`](https://github.com/wimi321/lizzieyzy-next/commit/92aa33430479e7906c002d726328298af4d50483) chore: merge main into search entry cleanup | UD-03-043 | UDX-018 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`1ba79d4d27338a170f7088acbdc86a729481a417`](https://github.com/wimi321/lizzieyzy-next/commit/1ba79d4d27338a170f7088acbdc86a729481a417) fix(engine): restore fenced startup diagnostics | UD-03-042 | UDX-001 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`08d6d1b39dd6793c8e04fe09ead8e3bc0ad52b95`](https://github.com/wimi321/lizzieyzy-next/commit/08d6d1b39dd6793c8e04fe09ead8e3bc0ad52b95) chore: merge main into startup diagnostics | UD-03-042 | UDX-001 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`86ee29510b605896a46543970f64a8a94d0e6f15`](https://github.com/wimi321/lizzieyzy-next/commit/86ee29510b605896a46543970f64a8a94d0e6f15) Merge pull request #461 from qiyi71w/fix/engine-startup-diagnostics | UD-03-042 | UDX-001 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`6bac01c3507c38b3ab8ae7018ac7cfe27d4bc4d0`](https://github.com/wimi321/lizzieyzy-next/commit/6bac01c3507c38b3ab8ae7018ac7cfe27d4bc4d0) Merge remote-tracking branch 'upstream/main' into fix/toolbar-search-entry | UD-03-043 | UDX-018 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`ba6783cfc89a158248ab7a781800c446faa40081`](https://github.com/wimi321/lizzieyzy-next/commit/ba6783cfc89a158248ab7a781800c446faa40081) Merge pull request #462 from qiyi71w/fix/toolbar-search-entry | UD-03-043 | UDX-018 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`be31e18b152c509ee4b9e52e737f1cc5f7b5235a`](https://github.com/wimi321/lizzieyzy-next/commit/be31e18b152c509ee4b9e52e737f1cc5f7b5235a) fix(engine): use GTP-safe paths for exact snapshot restore (#463) | UD-03-044 | UDX-015 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`714a31d18ddd173f0fbd1069c2dcdc5fa70bb0d3`](https://github.com/wimi321/lizzieyzy-next/commit/714a31d18ddd173f0fbd1069c2dcdc5fa70bb0d3) ci: require execution proof and desktop smoke (#464) | UD-03-EXC-002 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`741586a4d639321ef45f8dd3eb22561e2a514789`](https://github.com/wimi321/lizzieyzy-next/commit/741586a4d639321ef45f8dd3eb22561e2a514789) ci(desktop): require real search input evidence (#465) | UD-03-040a, UD-03-EXC-006 | UDX-018, UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`2698994a5b31c58c82629362223655699e9d1286`](https://github.com/wimi321/lizzieyzy-next/commit/2698994a5b31c58c82629362223655699e9d1286) test(engine): require process lifecycle smoke (#466) | UD-03-EXC-006 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`5f7b18cb3f404786831acffe39983f32156a9ef2`](https://github.com/wimi321/lizzieyzy-next/commit/5f7b18cb3f404786831acffe39983f32156a9ef2) docs(ci): restore desktop probe diagnostics (#467) | UD-03-EXC-002 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`7173e05f01d292623c18ee30cf88a4a7c50bd9a3`](https://github.com/wimi321/lizzieyzy-next/commit/7173e05f01d292623c18ee30cf88a4a7c50bd9a3) test(desktop): add local acceptance runner (#468) | UD-03-EXC-006 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`cb3310cb98e98879191a6820bffe321674ab0919`](https://github.com/wimi321/lizzieyzy-next/commit/cb3310cb98e98879191a6820bffe321674ab0919) test(engine): cover process failure recovery | UD-03-EXC-006 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`365363d6ac420ef79495823d0fe59b692427ca5a`](https://github.com/wimi321/lizzieyzy-next/commit/365363d6ac420ef79495823d0fe59b692427ca5a) fix(engine): finalize stopped transport state | UD-03-045 | UDX-004 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`5c048712ea51c24a8bdc012759fc7fa4b291a1ff`](https://github.com/wimi321/lizzieyzy-next/commit/5c048712ea51c24a8bdc012759fc7fa4b291a1ff) ci(engine): require native process gate | UD-03-EXC-002 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`2dff9fd2b7577d4727bf2304938c1cb257a0fd68`](https://github.com/wimi321/lizzieyzy-next/commit/2dff9fd2b7577d4727bf2304938c1cb257a0fd68) Merge pull request #469 from qiyi71w/plan/ci-engine-failure | UD-03-045, UD-03-EXC-006 | UDX-004, UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`bc5df09ec927d258abc26fbb6d379e8e408ee1d8`](https://github.com/wimi321/lizzieyzy-next/commit/bc5df09ec927d258abc26fbb6d379e8e408ee1d8) Add pinned KataGo source build receipts and real focus protocol probes | UD-03-EXC-007 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`4cc507210a1ed12f832096cb50d182cac78f8116`](https://github.com/wimi321/lizzieyzy-next/commit/4cc507210a1ed12f832096cb50d182cac78f8116) build: require executable identity and genuine hardware deferrals | UD-03-EXC-007 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`3f0eb53208c3ccf40c0fb6d9f8d6e9ff3882d8d8`](https://github.com/wimi321/lizzieyzy-next/commit/3f0eb53208c3ccf40c0fb6d9f8d6e9ff3882d8d8) build: retain macOS deployment target through Swift linking | UD-03-EXC-007 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`7afe28aa4b7121d5d33742ec157e7db4bc49581c`](https://github.com/wimi321/lizzieyzy-next/commit/7afe28aa4b7121d5d33742ec157e7db4bc49581c) Fix restart confirmation authority handoff and delayed acknowledgements | UD-03-046 | UDX-004 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`42898b2a97656219814ce15aeee1ffc98ff46138`](https://github.com/wimi321/lizzieyzy-next/commit/42898b2a97656219814ce15aeee1ffc98ff46138) test: await native setting focus instead of a fixed paint delay | UD-03-EXC-003 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`be242060ae81f5cec9fbad07b564ac5b8bf8cfb2`](https://github.com/wimi321/lizzieyzy-next/commit/be242060ae81f5cec9fbad07b564ac5b8bf8cfb2) Merge pull request #470 from wimi321/fix/restart-confirmation-445 | UD-03-046 | UDX-004 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`9d05032ecfcb1e63e3a5bf62f3718fc53397af51`](https://github.com/wimi321/lizzieyzy-next/commit/9d05032ecfcb1e63e3a5bf62f3718fc53397af51) Merge restart confirmation fix into source build validation | UD-03-046, UD-03-EXC-007 | UDX-004, UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`91bfa9bd43f893003b1ba58ce94a9b99be38c1a4`](https://github.com/wimi321/lizzieyzy-next/commit/91bfa9bd43f893003b1ba58ce94a9b99be38c1a4) fix: lock macOS source dependencies and audit deployment compatibility | UD-03-EXC-007 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`61cedcf2f46e58e541b00e9fc55372553f90b9f0`](https://github.com/wimi321/lizzieyzy-next/commit/61cedcf2f46e58e541b00e9fc55372553f90b9f0) test: keep SDK environment assertions platform independent | UD-03-EXC-007 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`65424a44b5e17084132ede8db5fab556c9d33a49`](https://github.com/wimi321/lizzieyzy-next/commit/65424a44b5e17084132ede8db5fab556c9d33a49) ci: rerun source build acceptance for KataGo test changes | UD-03-EXC-007 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`ed2d3a0ae617e1a13c23508b5355359addb52d2c`](https://github.com/wimi321/lizzieyzy-next/commit/ed2d3a0ae617e1a13c23508b5355359addb52d2c) fix: reserve portable library header space in the Swift linker | UD-03-EXC-007 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`ee08f5e98b4d99ca251904d3754776ad8f17c036`](https://github.com/wimi321/lizzieyzy-next/commit/ee08f5e98b4d99ca251904d3754776ad8f17c036) fix: defer settings navigation until native dialogs can accept focus | UD-03-047 | UDX-018 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`ec88d370a4fc4dcc5965d6d964090fae05d6ab0e`](https://github.com/wimi321/lizzieyzy-next/commit/ec88d370a4fc4dcc5965d6d964090fae05d6ab0e) Merge pull request #472 from wimi321/fix/settings-navigation-focus | UD-03-047 | UDX-018 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`9dcc17c35faf756052ba60633aaa23feb004ba3a`](https://github.com/wimi321/lizzieyzy-next/commit/9dcc17c35faf756052ba60633aaa23feb004ba3a) Merge tested native focus fixes into pinned engine builds | UD-03-047, UD-03-EXC-007 | UDX-018, UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`6e1e971f34b6518ab5226124b405d105cf7df2ec`](https://github.com/wimi321/lizzieyzy-next/commit/6e1e971f34b6518ab5226124b405d105cf7df2ec) test: honor explicit cold-start probe deadline and retain timing | UD-03-EXC-007 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`2a902ae61bccb506499af728c65fecb678300159`](https://github.com/wimi321/lizzieyzy-next/commit/2a902ae61bccb506499af728c65fecb678300159) fix(console): render engine output and loading comments on the EDT | UD-03-048 | UDX-019 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`df2dee4dcfdb011e7b21e858b67f65cfe5c190cc`](https://github.com/wimi321/lizzieyzy-next/commit/df2dee4dcfdb011e7b21e858b67f65cfe5c190cc) test: tolerate bounded Windows evidence-file sharing locks | UD-03-EXC-003 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`a139645c8df72124e7675dc5120790efae8b7bc0`](https://github.com/wimi321/lizzieyzy-next/commit/a139645c8df72124e7675dc5120790efae8b7bc0) Merge pull request #474 from wimi321/test/windows-peer-evidence | UD-03-EXC-003 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`8dec1f98416c7796a8d49fa4a068ba4f171e7c4a`](https://github.com/wimi321/lizzieyzy-next/commit/8dec1f98416c7796a8d49fa4a068ba4f171e7c4a) Merge remote-tracking branch 'origin/main' into fix/console-edt-rendering | UD-03-048 | UDX-019 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`5805c5b35e37c1d80794239388ca9b22aeb02b51`](https://github.com/wimi321/lizzieyzy-next/commit/5805c5b35e37c1d80794239388ca9b22aeb02b51) Merge pull request #473 from wimi321/fix/console-edt-rendering | UD-03-048 | UDX-019 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`e5bb35c13307b37b97f409d09855aef6d4c34a73`](https://github.com/wimi321/lizzieyzy-next/commit/e5bb35c13307b37b97f409d09855aef6d4c34a73) Merge remote-tracking branch 'origin/main' into build/katago-focus-source | UD-03-048, UD-03-EXC-007 | UDX-019, UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`48bd3e56966279f47e8a118449974f59828ff8cc`](https://github.com/wimi321/lizzieyzy-next/commit/48bd3e56966279f47e8a118449974f59828ff8cc) fix: attach game info dialog to its real owner | UD-03-049 | UDX-020 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`c421f5b0f77d8d530fd0f03a6f75452897971a38`](https://github.com/wimi321/lizzieyzy-next/commit/c421f5b0f77d8d530fd0f03a6f75452897971a38) test: synchronize native board focus before keyboard setup | UD-03-EXC-006 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`ba4d8b8db11b5e40ac5f9ae7c6890d3a35c96372`](https://github.com/wimi321/lizzieyzy-next/commit/ba4d8b8db11b5e40ac5f9ae7c6890d3a35c96372) ci: exercise desktop focus under a real X11 window manager | UD-03-EXC-002 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`d09da1971cd5a3339f6b4d78408ffebe66f98734`](https://github.com/wimi321/lizzieyzy-next/commit/d09da1971cd5a3339f6b4d78408ffebe66f98734) fix: restore search focus after native window activation | UD-03-049 | UDX-020 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`09a17018fc4f544833393d87c09e45907b7d9278`](https://github.com/wimi321/lizzieyzy-next/commit/09a17018fc4f544833393d87c09e45907b7d9278) fix: select komi as the initial focus target for navigation | UD-03-049 | UDX-020 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`c3e088ac3a4389f06c462d55475eae62b74bc524`](https://github.com/wimi321/lizzieyzy-next/commit/c3e088ac3a4389f06c462d55475eae62b74bc524) Merge pull request #475 from wimi321/fix/game-info-window-owner | UD-03-049 | UDX-020 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`15c575777f2656999e8a2e747c822f7c6f9cb0e0`](https://github.com/wimi321/lizzieyzy-next/commit/15c575777f2656999e8a2e747c822f7c6f9cb0e0) Merge remote-tracking branch 'origin/main' into build/katago-focus-source | UD-03-049, UD-03-EXC-007 | UDX-020, UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`12b2895e85af6097c6d5e137acd7eb7b9051fd26`](https://github.com/wimi321/lizzieyzy-next/commit/12b2895e85af6097c6d5e137acd7eb7b9051fd26) test: retain unexpected modal sources during repeated native navigation | UD-03-EXC-006 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`bda503fead372fd2618ebad5a468d76f8aecae5d`](https://github.com/wimi321/lizzieyzy-next/commit/bda503fead372fd2618ebad5a468d76f8aecae5d) fix: keep audio device failures nonblocking and preserve sound settings | UD-03-050 | UDX-021 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`c327cf1f5c628996ded0e7b85fbf1ca63c8a0fd1`](https://github.com/wimi321/lizzieyzy-next/commit/c327cf1f5c628996ded0e7b85fbf1ca63c8a0fd1) test: require all five native navigation repetitions in CI receipts | UD-03-EXC-006 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`baf63ef7f7490f51cbf8b0c7e5bdb0749bff98a7`](https://github.com/wimi321/lizzieyzy-next/commit/baf63ef7f7490f51cbf8b0c7e5bdb0749bff98a7) Integrate nonblocking sound fix and strict repeated desktop acceptance | UD-03-050, UD-03-EXC-007 | UDX-021, UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`c83deb372aaf9d2ec53a0c3d0afaac3cc3e9d4ae`](https://github.com/wimi321/lizzieyzy-next/commit/c83deb372aaf9d2ec53a0c3d0afaac3cc3e9d4ae) build: add locked Linux CPU and OpenCL source evidence pipeline | UD-03-EXC-007 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`e38202c541070d0ca30b6d05b3589333670273ea`](https://github.com/wimi321/lizzieyzy-next/commit/e38202c541070d0ca30b6d05b3589333670273ea) Merge pull request #476 from wimi321/test/native-window-evidence | UD-03-050 | UDX-021 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`4c86a9ececc0a4cac5d6e3ea475dc9b7c67fcb51`](https://github.com/wimi321/lizzieyzy-next/commit/4c86a9ececc0a4cac5d6e3ea475dc9b7c67fcb51) Merge pull request #471 from wimi321/build/katago-focus-source | UD-03-EXC-007 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`5af5113bffaf8200f3bb3b0d449a4eb8463c195a`](https://github.com/wimi321/lizzieyzy-next/commit/5af5113bffaf8200f3bb3b0d449a4eb8463c195a) Integrate reviewed source-build foundation from main | UD-03-EXC-007 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`e146e78cc7edb5919a8fc3d7925eb597edfa61c6`](https://github.com/wimi321/lizzieyzy-next/commit/e146e78cc7edb5919a8fc3d7925eb597edfa61c6) fix: use verified zlib release and preserve static link dependencies | UD-03-EXC-007 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`bba2bafda72468f7c4d5d5cde3db9434f664a8a4`](https://github.com/wimi321/lizzieyzy-next/commit/bba2bafda72468f7c4d5d5cde3db9434f664a8a4) build: add locked Windows CPU and OpenCL source evidence | UD-03-EXC-007 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`f178b22f8d98b8b41e708fc363cc14ab2c98a25b`](https://github.com/wimi321/lizzieyzy-next/commit/f178b22f8d98b8b41e708fc363cc14ab2c98a25b) fix: distinguish local shared analysis from remote execution | UD-03-051 | UDX-011 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`4c11cf264d855cd576c12183ccd97173a9372b29`](https://github.com/wimi321/lizzieyzy-next/commit/4c11cf264d855cd576c12183ccd97173a9372b29) Merge pull request #478 from wimi321/fix/whole-game-execution-mode | UD-03-051 | UDX-011 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`e03490000b596e60639da1f61be120b4563cc7d8`](https://github.com/wimi321/lizzieyzy-next/commit/e03490000b596e60639da1f61be120b4563cc7d8) Merge remote-tracking branch 'origin/main' into build/katago-source-windows | UD-03-051, UD-03-EXC-007 | UDX-011, UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`35df875894942ab4e8fe2bead43576ce7465d845`](https://github.com/wimi321/lizzieyzy-next/commit/35df875894942ab4e8fe2bead43576ce7465d845) fix: install Eigen headers without unrelated Fortran configuration | UD-03-EXC-007 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`4f86f57d3491019060fbe50e0cdcd4a73b9b2228`](https://github.com/wimi321/lizzieyzy-next/commit/4f86f57d3491019060fbe50e0cdcd4a73b9b2228) fix: normalize Windows environment keys for portable engine checks | UD-03-EXC-007 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`a5663b14096a6072688cac84bcbb940229963047`](https://github.com/wimi321/lizzieyzy-next/commit/a5663b14096a6072688cac84bcbb940229963047) Merge pull request #479 from wimi321/build/katago-source-windows | UD-03-EXC-007 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`eb245718b31fb54b76157251e6e2477120b9e2e4`](https://github.com/wimi321/lizzieyzy-next/commit/eb245718b31fb54b76157251e6e2477120b9e2e4) build: lock and verify DirectML source-engine dependencies | UD-03-EXC-007 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`9989c718597188709baee44c576684fd19121ef2`](https://github.com/wimi321/lizzieyzy-next/commit/9989c718597188709baee44c576684fd19121ef2) Merge pull request #480 from wimi321/build/katago-source-directml | UD-03-EXC-007 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`b0c905238453b0eacd367699e7821555da864c42`](https://github.com/wimi321/lizzieyzy-next/commit/b0c905238453b0eacd367699e7821555da864c42) fix(logging): reserve time for interrupted resource cleanup | UD-03-052 | UDX-001 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`9e0ca19e1e0e929d42b368755bcfa891bdf6c906`](https://github.com/wimi321/lizzieyzy-next/commit/9e0ca19e1e0e929d42b368755bcfa891bdf6c906) Merge pull request #482 from wimi321/fix/logging-shutdown-cleanup | UD-03-052 | UDX-001 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`0dbe0b0130c54c0ce659af7abbcd4417a36da51f`](https://github.com/wimi321/lizzieyzy-next/commit/0dbe0b0130c54c0ce659af7abbcd4417a36da51f) feat(release): verify final artifact identity | UD-03-EXC-006 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`bb1ca7832d57465acb895bc213132eb86cb7a017`](https://github.com/wimi321/lizzieyzy-next/commit/bb1ca7832d57465acb895bc213132eb86cb7a017) test(gui): add real CPU engine acceptance | UD-03-EXC-006 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`a83a1785d5c1821ceac28d89ca84e2d95e2cea1a`](https://github.com/wimi321/lizzieyzy-next/commit/a83a1785d5c1821ceac28d89ca84e2d95e2cea1a) test(gui): add SGF UI acceptance coverage | UD-03-EXC-006 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`564545739af3073a58079f4cc02af27947755836`](https://github.com/wimi321/lizzieyzy-next/commit/564545739af3073a58079f4cc02af27947755836) test(acceptance): integrate SGF and CPU gates | UD-03-EXC-006 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`5e0496c837fb97f741591dd9ed0a07e684a03816`](https://github.com/wimi321/lizzieyzy-next/commit/5e0496c837fb97f741591dd9ed0a07e684a03816) fix(acceptance): bound CPU asset downloads | UD-03-EXC-006 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`52902e341effd32bc299cc04d5ac3d30aba4c4bd`](https://github.com/wimi321/lizzieyzy-next/commit/52902e341effd32bc299cc04d5ac3d30aba4c4bd) test: enforce acceptance download size and cross-platform gates | UD-03-EXC-006 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`6a996bae16000a6a0bc8afce876ae0c2ade76d62`](https://github.com/wimi321/lizzieyzy-next/commit/6a996bae16000a6a0bc8afce876ae0c2ade76d62) build: lock and audit OpenVINO source-engine artifacts | UD-03-EXC-007 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`59ebb4c129f074b778af766442ef7e08112e6d67`](https://github.com/wimi321/lizzieyzy-next/commit/59ebb4c129f074b778af766442ef7e08112e6d67) Merge remote-tracking branch 'origin/main' into build/katago-source-openvino | UD-03-052, UD-03-EXC-007 | UDX-001, UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`e1c8bc603d9a8f0c10533bf9e3967ea658fafbef`](https://github.com/wimi321/lizzieyzy-next/commit/e1c8bc603d9a8f0c10533bf9e3967ea658fafbef) docs: clarify source-build target coverage | UD-03-EXC-007 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`f7312a9f0ce0baad9bfba6c301e8631c714bb8fb`](https://github.com/wimi321/lizzieyzy-next/commit/f7312a9f0ce0baad9bfba6c301e8631c714bb8fb) Merge pull request #481 from wimi321/build/katago-source-openvino | UD-03-EXC-007 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`7b5f027b8d0809fa20f307994f693e0c9fe7e68a`](https://github.com/wimi321/lizzieyzy-next/commit/7b5f027b8d0809fa20f307994f693e0c9fe7e68a) Merge remote-tracking branch 'origin/main' into review/acceptance-483 | UD-03-EXC-006 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`1d11b7ea217df6ced5b145fc1ec295e62294dd29`](https://github.com/wimi321/lizzieyzy-next/commit/1d11b7ea217df6ced5b145fc1ec295e62294dd29) test: execute acceptance helpers on Windows and Linux UI gates | UD-03-EXC-006 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`f1c0515d78a1e8885f5d0388bfb7c619bad35068`](https://github.com/wimi321/lizzieyzy-next/commit/f1c0515d78a1e8885f5d0388bfb7c619bad35068) test: isolate WebSocket fixtures and retain CPU failure traces | UD-03-EXC-006 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`eda99b488e5a5c6e82a9d2cf7b971316df13522a`](https://github.com/wimi321/lizzieyzy-next/commit/eda99b488e5a5c6e82a9d2cf7b971316df13522a) test: distinguish bounded CPU cold inference from missing analysis | UD-03-EXC-006 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`7c7985ad5d852a12a167086e668ef3ddfeb2c124`](https://github.com/wimi321/lizzieyzy-next/commit/7c7985ad5d852a12a167086e668ef3ddfeb2c124) build: add locked Windows CUDA source artifacts | UD-03-EXC-007 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`7af73741a020ca12f7ffc51159d718fd1f8d07fb`](https://github.com/wimi321/lizzieyzy-next/commit/7af73741a020ca12f7ffc51159d718fd1f8d07fb) build: preserve complete CUDA runtime archive inventory | UD-03-EXC-007 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`d6646b1541936566c8db43f2cba770e67306ba27`](https://github.com/wimi321/lizzieyzy-next/commit/d6646b1541936566c8db43f2cba770e67306ba27) Merge pull request #484 from wimi321/build/katago-source-cuda | UD-03-EXC-007 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`4d63b53c3a7ceec1f84c1e8daa74db29dafef0ee`](https://github.com/wimi321/lizzieyzy-next/commit/4d63b53c3a7ceec1f84c1e8daa74db29dafef0ee) Merge remote-tracking branch 'origin/main' into review/acceptance-483 | UD-03-EXC-006 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`62ea06de304226db5993b8040ad88964539b44ff`](https://github.com/wimi321/lizzieyzy-next/commit/62ea06de304226db5993b8040ad88964539b44ff) Merge pull request #483 from qiyi71w/integrate/remaining-acceptance | UD-03-EXC-006 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`94c0f4901ec8269898b123c29308707a60d4d7bc`](https://github.com/wimi321/lizzieyzy-next/commit/94c0f4901ec8269898b123c29308707a60d4d7bc) Merge main into same-tree focus and preserve rule synchronization | UD-03-054 | UDX-022 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`bd228cccecc0f53448bf5f5da0377d3c1ea833e3`](https://github.com/wimi321/lizzieyzy-next/commit/bd228cccecc0f53448bf5f5da0377d3c1ea833e3) test: compare native settings navigation with active catalog | UD-03-054 | UDX-022 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`7e72e32fd6f76ce0b7fef1e63304ffc45e6a20c0`](https://github.com/wimi321/lizzieyzy-next/commit/7e72e32fd6f76ce0b7fef1e63304ffc45e6a20c0) test: verify same-tree focus in a native window with the pinned engine | UD-03-054, UD-03-054f | UDX-022 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`44ad3474f7ad430096ba23c5c8f41a5d33cebd0f`](https://github.com/wimi321/lizzieyzy-next/commit/44ad3474f7ad430096ba23c5c8f41a5d33cebd0f) build: seal TensorRT 10.9 source SDK and runtime audit | UD-03-EXC-007 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`15b299c6b1a5cf0c0a67008f5b5bb5addea0612d`](https://github.com/wimi321/lizzieyzy-next/commit/15b299c6b1a5cf0c0a67008f5b5bb5addea0612d) build: seal static protobuf for TensorRT ONNX generation | UD-03-EXC-007 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`f7589969108770ddae76ab5c435a446ae9415c0a`](https://github.com/wimi321/lizzieyzy-next/commit/f7589969108770ddae76ab5c435a446ae9415c0a) build: add pinned Linux CUDA source evidence | UD-03-EXC-007 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`9f5ec287c3297f477149f0ebc4a0271d82a9ca54`](https://github.com/wimi321/lizzieyzy-next/commit/9f5ec287c3297f477149f0ebc4a0271d82a9ca54) test: keep Linux CUDA audit tests portable | UD-03-EXC-007 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`b45e65be76b95b8176f7c31b1b45623690e65405`](https://github.com/wimi321/lizzieyzy-next/commit/b45e65be76b95b8176f7c31b1b45623690e65405) fix(build): resolve Linux nvcc runtime archive layout | UD-03-EXC-007 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`3b2ec98edb4be85f03b91053734f10c3852b3496`](https://github.com/wimi321/lizzieyzy-next/commit/3b2ec98edb4be85f03b91053734f10c3852b3496) build: preserve exact Linux runtime audit failure details | UD-03-EXC-007 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`66670b3853ac3721a26c3325f7dc73d4e023437c`](https://github.com/wimi321/lizzieyzy-next/commit/66670b3853ac3721a26c3325f7dc73d4e023437c) build: bundle sealed zlib required by Linux cuDNN | UD-03-EXC-007 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`d0f339849481529d94feeb44b2f486ae8a5a17e8`](https://github.com/wimi321/lizzieyzy-next/commit/d0f339849481529d94feeb44b2f486ae8a5a17e8) Merge pull request #485 from wimi321/build/katago-source-linux-cuda-v2 | UD-03-EXC-007 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`d0aff16ec9333d8948052eb8543015e689da9773`](https://github.com/wimi321/lizzieyzy-next/commit/d0aff16ec9333d8948052eb8543015e689da9773) Merge verified Linux CUDA source foundation | UD-03-EXC-007 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`8a3f54fd45b343fb08659870e90e1a43c162c3f1`](https://github.com/wimi321/lizzieyzy-next/commit/8a3f54fd45b343fb08659870e90e1a43c162c3f1) Merge pull request #486 from wimi321/build/katago-source-tensorrt | UD-03-EXC-007 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`d3833d60204d540cc5b523be7c934b312114f77d`](https://github.com/wimi321/lizzieyzy-next/commit/d3833d60204d540cc5b523be7c934b312114f77d) Merge remote-tracking branch 'origin/main' into fix/focus-probe-resume | UD-03-054 | UDX-022 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`60e65e22e06a390f45ba174aa471090b49e4566d`](https://github.com/wimi321/lizzieyzy-next/commit/60e65e22e06a390f45ba174aa471090b49e4566d) Stop owned WebSocket server before awaiting fixture clients | UD-03-EXC-003 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`2061865fce2e9ec4a44633fe86f1dd6bde9112d2`](https://github.com/wimi321/lizzieyzy-next/commit/2061865fce2e9ec4a44633fe86f1dd6bde9112d2) Seal audited source engines and share trusted repair download origins | UD-03-035a, UD-03-EXC-007 | UDX-002, UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`30dceff6cf0363e4d3025d7e1fdac021dc80e672`](https://github.com/wimi321/lizzieyzy-next/commit/30dceff6cf0363e4d3025d7e1fdac021dc80e672) Verify source Linux engines against production ABI and clean distributions | UD-03-EXC-007 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`29b8985a330068b62ab92cddc9ca080c8efa5d78`](https://github.com/wimi321/lizzieyzy-next/commit/29b8985a330068b62ab92cddc9ca080c8efa5d78) Isolate new loader checks from historical baseline dependencies | UD-03-EXC-007 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`a4ab1ff337c7e99ac96409f81f7c7f0c0da24356`](https://github.com/wimi321/lizzieyzy-next/commit/a4ab1ff337c7e99ac96409f81f7c7f0c0da24356) Include pinned configs and reverify source archive contents | UD-03-EXC-007 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`0cc9ef8fe43003a67b3995755e6fc34482d50d00`](https://github.com/wimi321/lizzieyzy-next/commit/0cc9ef8fe43003a67b3995755e6fc34482d50d00) Merge commit 'a4ab1ff3' into build/katago-linux-compatibility | UD-03-EXC-007 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`280f7ebf78c5c65e948bbcd79dcf7ec0b03bd769`](https://github.com/wimi321/lizzieyzy-next/commit/280f7ebf78c5c65e948bbcd79dcf7ec0b03bd769) Supply complete GTP configuration for isolated inference acceptance | UD-03-EXC-007 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`37e922aa35a01d8d41019c77fe962cd57e049771`](https://github.com/wimi321/lizzieyzy-next/commit/37e922aa35a01d8d41019c77fe962cd57e049771) Verify source archives before installing them into release build trees | UD-03-EXC-007 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`59e95d11fc7b17563e0f8b84087e50a428449b3d`](https://github.com/wimi321/lizzieyzy-next/commit/59e95d11fc7b17563e0f8b84087e50a428449b3d) Merge commit '37e922aa' into build/katago-linux-compatibility | UD-03-EXC-007 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`4019a7e15aa4deaf2a1e5b6e49ae691f1f7db74d`](https://github.com/wimi321/lizzieyzy-next/commit/4019a7e15aa4deaf2a1e5b6e49ae691f1f7db74d) Merge pull request #488 from wimi321/build/katago-source-delivery | UD-03-035a, UD-03-EXC-007 | UDX-002, UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`630866cb25e41b4ac59baab298e8cc49356952c1`](https://github.com/wimi321/lizzieyzy-next/commit/630866cb25e41b4ac59baab298e8cc49356952c1) Merge remote-tracking branch 'origin/main' into build/katago-linux-compatibility | UD-03-EXC-007 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`2b6e31da53f3e805338eb3062fc71dd6a7f4f912`](https://github.com/wimi321/lizzieyzy-next/commit/2b6e31da53f3e805338eb3062fc71dd6a7f4f912) Merge pull request #489 from wimi321/build/katago-linux-compatibility | UD-03-EXC-007 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`e35b01c14e2fb423f09de05311bb68dd1f6d1c1c`](https://github.com/wimi321/lizzieyzy-next/commit/e35b01c14e2fb423f09de05311bb68dd1f6d1c1c) Merge remote-tracking branch 'origin/main' into test/webboard-owned-cleanup | UD-03-035a, UD-03-EXC-003 | UDX-002, UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`34986b9359d77fa3b7b1d8c597ab4d742f4d95d5`](https://github.com/wimi321/lizzieyzy-next/commit/34986b9359d77fa3b7b1d8c597ab4d742f4d95d5) Merge pull request #491 from wimi321/test/webboard-owned-cleanup | UD-03-EXC-003 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`2fed2f5b4057d46373ed2287f7ba742c24cffca0`](https://github.com/wimi321/lizzieyzy-next/commit/2fed2f5b4057d46373ed2287f7ba742c24cffca0) Avoid conflicting KataGo search thread aliases during analysis | UD-03-053 | UDX-017 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`73ac3e422463c06b8b94b1ab9bf1b8fac58f63a6`](https://github.com/wimi321/lizzieyzy-next/commit/73ac3e422463c06b8b94b1ab9bf1b8fac58f63a6) Merge pull request #492 from wimi321/fix/analysis-thread-alias | UD-03-053 | UDX-017 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`1cfc050f25aaee6aa03c30004b523e27d273e681`](https://github.com/wimi321/lizzieyzy-next/commit/1cfc050f25aaee6aa03c30004b523e27d273e681) Merge remote-tracking branch 'origin/main' into fix/focus-probe-resume | UD-03-054 | UDX-022 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`6053dc3401aee9426847ae08e695310917c36046`](https://github.com/wimi321/lizzieyzy-next/commit/6053dc3401aee9426847ae08e695310917c36046) Install and audit reviewed source engines in final release packages | UD-03-EXC-007 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`4c474f3a66d473061c0b190b8b92b5f659c35a00`](https://github.com/wimi321/lizzieyzy-next/commit/4c474f3a66d473061c0b190b8b92b5f659c35a00) Verify source-built CPU acceptance archives and exact executable revision | UD-03-EXC-007 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`33f0996f4db411910a6db3c5b2f2d58aa2f4a912`](https://github.com/wimi321/lizzieyzy-next/commit/33f0996f4db411910a6db3c5b2f2d58aa2f4a912) Integrate verified Linux compatibility gates into final packaging | UD-03-EXC-007 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`3ef5fa3778e44bff360fefc12c286105a5d6f878`](https://github.com/wimi321/lizzieyzy-next/commit/3ef5fa3778e44bff360fefc12c286105a5d6f878) Merge remote-tracking branch 'origin/main' into build/katago-source-packaging | UD-03-EXC-007 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`c62a0fb778185e8510636f558664c02a2c135b19`](https://github.com/wimi321/lizzieyzy-next/commit/c62a0fb778185e8510636f558664c02a2c135b19) Await asynchronous application exit in native SGF acceptance | UD-03-EXC-007 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`7c88a25c031ddf633fc37706902d11be9ab10b1b`](https://github.com/wimi321/lizzieyzy-next/commit/7c88a25c031ddf633fc37706902d11be9ab10b1b) Merge remote-tracking branch 'origin/main' into build/katago-source-packaging | UD-03-053, UD-03-EXC-007 | UDX-017, UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`08b6908dfdbed86f83f95e43e454d35c497add26`](https://github.com/wimi321/lizzieyzy-next/commit/08b6908dfdbed86f83f95e43e454d35c497add26) Merge pull request #490 from wimi321/build/katago-source-packaging | UD-03-EXC-007 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`e23eddfddf94af2aa7fed1f985727c22ef4f9715`](https://github.com/wimi321/lizzieyzy-next/commit/e23eddfddf94af2aa7fed1f985727c22ef4f9715) Merge remote-tracking branch 'origin/main' into fix/focus-probe-resume | UD-03-054 | UDX-022 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`076cc4ad127609231645d614cdeb61881bd6e458`](https://github.com/wimi321/lizzieyzy-next/commit/076cc4ad127609231645d614cdeb61881bd6e458) Merge pull request #449 from qiyi71w/feat/414-move-focus | UD-03-054 | UDX-022 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`0a6546f4a7e5e671c9b414eb8eb736c15eb151dc`](https://github.com/wimi321/lizzieyzy-next/commit/0a6546f4a7e5e671c9b414eb8eb736c15eb151dc) Prepare source-engine activation contracts and installation guidance | UD-04-002 | UDX-002 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`e67c5a3b47f45c4bb27d334f4163f8c8dced1f0f`](https://github.com/wimi321/lizzieyzy-next/commit/e67c5a3b47f45c4bb27d334f4163f8c8dced1f0f) Merge remote-tracking branch 'origin/main' into release/activate-pinned-katago | UD-04-002 | UDX-002 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`255028f76e33dcd0d9f3236bd9447fba89054680`](https://github.com/wimi321/lizzieyzy-next/commit/255028f76e33dcd0d9f3236bd9447fba89054680) Activate all 15 audited focus-capable engine archives behind a draft acceptance gate | UD-04-002 | UDX-002 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`dfd7362a3b1bf5e76ae1327190e5a9aa286da036`](https://github.com/wimi321/lizzieyzy-next/commit/dfd7362a3b1bf5e76ae1327190e5a9aa286da036) Match installer assertions to the reviewed engine catalog | UD-04-002 | UDX-002 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`418f5f3cbbc5018437915a761ba82c8acf29c895`](https://github.com/wimi321/lizzieyzy-next/commit/418f5f3cbbc5018437915a761ba82c8acf29c895) Merge remote-tracking branch 'origin/main' into release/activate-pinned-katago | UD-04-002 | UDX-002 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`b318dbf9952701a9c59b7cb50256f5bf1b242e31`](https://github.com/wimi321/lizzieyzy-next/commit/b318dbf9952701a9c59b7cb50256f5bf1b242e31) Prepare next-2026-09-17.1 same-tree analysis prerelease | UD-04-003 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`1b7029d147b60600afe5eda831654ab10d75afbb`](https://github.com/wimi321/lizzieyzy-next/commit/1b7029d147b60600afe5eda831654ab10d75afbb) Isolate Draft asset provisioning from unprivileged native acceptance | UD-04-002 | UDX-002 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`00465a1d93f4abc9d25badf5c4b0abd00a2921fe`](https://github.com/wimi321/lizzieyzy-next/commit/00465a1d93f4abc9d25badf5c4b0abd00a2921fe) Keep the official-catalog rejection fixture independent of release activation | UD-04-002 | UDX-002 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`6cf94ee206948709e81e90f7031c35a727ae9e40`](https://github.com/wimi321/lizzieyzy-next/commit/6cf94ee206948709e81e90f7031c35a727ae9e40) Test valid official catalog rejection without network access | UD-04-002 | UDX-002 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`31ab10c39f9d6e1297ea4d63b94427b921e63e05`](https://github.com/wimi321/lizzieyzy-next/commit/31ab10c39f9d6e1297ea4d63b94427b921e63e05) Build pinned ROCm engines with unchanged family runtimes | UD-04-001 | UDX-002 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`8e44c9010618322a95715cb6c9457afdb461c9ad`](https://github.com/wimi321/lizzieyzy-next/commit/8e44c9010618322a95715cb6c9457afdb461c9ad) Retain ROCm compiler preflight diagnostics before configure | UD-04-001 | UDX-002 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`86a8a862ce0f8a13a907a122097f81ae01a746f7`](https://github.com/wimi321/lizzieyzy-next/commit/86a8a862ce0f8a13a907a122097f81ae01a746f7) Supply pinned HIP device libraries to the Windows compiler | UD-04-001 | UDX-002 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`3803a62cb5b64e28f78dd6f780a8da05c4f271fc`](https://github.com/wimi321/lizzieyzy-next/commit/3803a62cb5b64e28f78dd6f780a8da05c4f271fc) Normalize HIP compiler paths before CMake serialization | UD-04-001 | UDX-002 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`c5545229703000f3183e907815f6863bbb4601e5`](https://github.com/wimi321/lizzieyzy-next/commit/c5545229703000f3183e907815f6863bbb4601e5) Preserve hidden dependency license files in source artifacts | UD-04-001 | UDX-002 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`6b31e28564a302722fcc8b01607ff0ddc826dcaa`](https://github.com/wimi321/lizzieyzy-next/commit/6b31e28564a302722fcc8b01607ff0ddc826dcaa) Merge remote-tracking branch 'origin/main' into build/katago-source-rocm | UD-04-001 | UDX-002 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`dd184f5125a9f80a695761497193fa885269dd2e`](https://github.com/wimi321/lizzieyzy-next/commit/dd184f5125a9f80a695761497193fa885269dd2e) Merge remote-tracking branch 'origin/main' into build/katago-source-rocm | UD-04-001 | UDX-002 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`8ab44dc21d520c5c0bfb8f937c3fd87c28267631`](https://github.com/wimi321/lizzieyzy-next/commit/8ab44dc21d520c5c0bfb8f937c3fd87c28267631) Measure HTTP timeout independently of Windows interpreter startup | UD-04-001 | UDX-002 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`6e43939a3a864100730af42b5906e6b73a68128d`](https://github.com/wimi321/lizzieyzy-next/commit/6e43939a3a864100730af42b5906e6b73a68128d) Merge pull request #487 from wimi321/build/katago-source-rocm | UD-04-001 | UDX-002 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`b9b80a59f0f1868cabe8884abf26af3c08ecdaf6`](https://github.com/wimi321/lizzieyzy-next/commit/b9b80a59f0f1868cabe8884abf26af3c08ecdaf6) Merge remote-tracking branch 'origin/main' into release/activate-pinned-katago | UD-04-002 | UDX-002 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`c0d15d8cb0cf4f06ade6aaa3718ac3d4e70a871b`](https://github.com/wimi321/lizzieyzy-next/commit/c0d15d8cb0cf4f06ade6aaa3718ac3d4e70a871b) Merge branch 'release/activate-pinned-katago' into release/prerelease-focus-20260917 | UD-04-003 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`2c379b4f511f17fdc5bed2c6a4d184f766e8ee80`](https://github.com/wimi321/lizzieyzy-next/commit/2c379b4f511f17fdc5bed2c6a4d184f766e8ee80) Merge pull request #493 from wimi321/release/activate-pinned-katago | UD-04-002 | UDX-002 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`e8e5171b1ee009af81c1991bad361909bcb29dc9`](https://github.com/wimi321/lizzieyzy-next/commit/e8e5171b1ee009af81c1991bad361909bcb29dc9) Merge remote-tracking branch 'origin/main' into release/prerelease-focus-20260917 | UD-04-003 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`aa07f6e07b16add08b271e7b3994b731216d5e0d`](https://github.com/wimi321/lizzieyzy-next/commit/aa07f6e07b16add08b271e7b3994b731216d5e0d) Merge pull request #494 from wimi321/release/prerelease-focus-20260917 | UD-04-003 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`6e2e386a7976826d927124b88cfe2aefbd549741`](https://github.com/wimi321/lizzieyzy-next/commit/6e2e386a7976826d927124b88cfe2aefbd549741) Preserve whole-game analysis ownership through foreground handoff | UD-04-004 | UDX-011 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`6ea79ccc2c251389f5a5edb63f5b89ddb026642c`](https://github.com/wimi321/lizzieyzy-next/commit/6ea79ccc2c251389f5a5edb63f5b89ddb026642c) Preserve foreground intent across transient snapshot restoration | UD-04-004 | UDX-011 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`cae8b21fb7d635210f1fc9fe88f667388c40078d`](https://github.com/wimi321/lizzieyzy-next/commit/cae8b21fb7d635210f1fc9fe88f667388c40078d) Prepare audited same-tree analysis prerelease with final handoff fixes | UD-04-005 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`c8eea60bbdc9cd9dcd69a1b081d9c8f51cb9575d`](https://github.com/wimi321/lizzieyzy-next/commit/c8eea60bbdc9cd9dcd69a1b081d9c8f51cb9575d) Merge pull request #495 from wimi321/fix/whole-game-terminal-handoff | UD-04-004 | UDX-011 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`c0b18f0d978126f9091721532e0081256f07fbe4`](https://github.com/wimi321/lizzieyzy-next/commit/c0b18f0d978126f9091721532e0081256f07fbe4) Merge remote-tracking branch 'origin/main' into release/prerelease-focus-20260917-v2 | UD-04-005 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`815c4a90040dc9a7412823200a49513f272de0ec`](https://github.com/wimi321/lizzieyzy-next/commit/815c4a90040dc9a7412823200a49513f272de0ec) Merge pull request #496 from wimi321/release/prerelease-focus-20260917-v2 | UD-04-005 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`29b923a2e38a69a989a3c0b60634f82f1d659d56`](https://github.com/wimi321/lizzieyzy-next/commit/29b923a2e38a69a989a3c0b60634f82f1d659d56) Prepare validated source-engine prerelease next-2026-09-17.3 | UD-04-008 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`665f4f232d2f441004af5b4b3a233ad718dca597`](https://github.com/wimi321/lizzieyzy-next/commit/665f4f232d2f441004af5b4b3a233ad718dca597) Keep Windows installer staging clean and propagate packaging failures | UD-04-007 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`2a77ca55bd8a5281dfd1e8b3e4f7d8dfe4b84e31`](https://github.com/wimi321/lizzieyzy-next/commit/2a77ca55bd8a5281dfd1e8b3e4f7d8dfe4b84e31) Preserve reviewed KataGo bytes across macOS app packaging | UD-04-006 | UDX-002 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`1a70ca2e5468ff324f8459f73a5ad51f7c83bf2c`](https://github.com/wimi321/lizzieyzy-next/commit/1a70ca2e5468ff324f8459f73a5ad51f7c83bf2c) Merge pull request #497 from wimi321/fix/macos-source-integrity | UD-04-006 | UDX-002 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`8bdcd39f5bd1e66e18a6a46e7185bdbb617d115a`](https://github.com/wimi321/lizzieyzy-next/commit/8bdcd39f5bd1e66e18a6a46e7185bdbb617d115a) Merge remote-tracking branch 'origin/main' into fix/windows-installer-staging | UD-04-007 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`e5788680ea62f0202c639846ae19d700e2c336d4`](https://github.com/wimi321/lizzieyzy-next/commit/e5788680ea62f0202c639846ae19d700e2c336d4) Merge pull request #498 from wimi321/fix/windows-installer-staging | UD-04-007 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`2ebe7397cf7aac3e3dd7ec7d5b681a2c57816648`](https://github.com/wimi321/lizzieyzy-next/commit/2ebe7397cf7aac3e3dd7ec7d5b681a2c57816648) Merge remote-tracking branch 'origin/main' into release/prerelease-focus-20260917-v3 | UD-04-008 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`2585443492345e42436345ca803b0c26bfc58320`](https://github.com/wimi321/lizzieyzy-next/commit/2585443492345e42436345ca803b0c26bfc58320) Separate probe schema timeout from child JVM startup in acceptance | UD-04-008 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`eb5d245275cb1d106801d11e62abb617a9ba3ff4`](https://github.com/wimi321/lizzieyzy-next/commit/eb5d245275cb1d106801d11e62abb617a9ba3ff4) Merge pull request #499 from wimi321/release/prerelease-focus-20260917-v3 | UD-04-008 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`162d50debb153978f602217f452901aff9fe065b`](https://github.com/wimi321/lizzieyzy-next/commit/162d50debb153978f602217f452901aff9fe065b) Avoid startup benchmark failure dialogs when analysis takes priority | UD-04-009 | UDX-024 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`7cb1a41df48c57e6e3215dad0d63f7ad6896e8ca`](https://github.com/wimi321/lizzieyzy-next/commit/7cb1a41df48c57e6e3215dad0d63f7ad6896e8ca) Await persisted log events in engine identity assertions | UD-04-009 | UDX-024 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`1c0b6bd7e3f2a6c2771457552715efe2ea3aa339`](https://github.com/wimi321/lizzieyzy-next/commit/1c0b6bd7e3f2a6c2771457552715efe2ea3aa339) Merge pull request #500 from wimi321/fix/startup-benchmark-yield-20260917 | UD-04-009 | UDX-024 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`bf73c6c3614776906f6ac8183cb7bdd4a7805473`](https://github.com/wimi321/lizzieyzy-next/commit/bf73c6c3614776906f6ac8183cb7bdd4a7805473) Prepare next-2026-09-17.4 with verified benchmark handoff | UD-04-010 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`4509ace7486e2749d6c425da46f6956272a041e4`](https://github.com/wimi321/lizzieyzy-next/commit/4509ace7486e2749d6c425da46f6956272a041e4) Merge branch 'fix/startup-benchmark-yield-20260917' into release/prerelease-focus-20260917-v4 | UD-04-010 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`c870d9f35d195500caf17d63c4a90203eeae8e4f`](https://github.com/wimi321/lizzieyzy-next/commit/c870d9f35d195500caf17d63c4a90203eeae8e4f) Merge pull request #501 from wimi321/release/prerelease-focus-20260917-v4 | UD-04-010 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`d445977043df41f1b683559d75cff7f87e40aa7d`](https://github.com/wimi321/lizzieyzy-next/commit/d445977043df41f1b683559d75cff7f87e40aa7d) Serialize comment rendering and theme refresh on the Swing EDT | UD-04-011 | UDX-025 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`c07a45a4f142b2d8a7560d175ec317e752271031`](https://github.com/wimi321/lizzieyzy-next/commit/c07a45a4f142b2d8a7560d175ec317e752271031) Merge pull request #502 from wimi321/fix/comment-render-edt-20260917 | UD-04-011 | UDX-025 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`0492a9367bfeac569b1e9dcb5ba04d7a12c4d4ee`](https://github.com/wimi321/lizzieyzy-next/commit/0492a9367bfeac569b1e9dcb5ba04d7a12c4d4ee) Clear the inherited GTP thread alias for HumanSL launches | UD-04-012 | UDX-003 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`24b181b4d6d285716b932c954d4e63723c06fe46`](https://github.com/wimi321/lizzieyzy-next/commit/24b181b4d6d285716b932c954d4e63723c06fe46) Keep delayed startup tuning from interrupting user analysis and training | UD-04-013 | UDX-024 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`c984eccdec892b4b28bfe80471add25b27d76462`](https://github.com/wimi321/lizzieyzy-next/commit/c984eccdec892b4b28bfe80471add25b27d76462) Distinguish the primary engine from auxiliary compute for startup tuning | UD-04-013 | UDX-024 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`eb7f8ab12139739c2d07b0983b0f90e2f2f5151a`](https://github.com/wimi321/lizzieyzy-next/commit/eb7f8ab12139739c2d07b0983b0f90e2f2f5151a) Synchronize diagnostic and socket test cleanup on completion | UD-04-012 | UDX-003 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`94706b7be6199678e2bb04aa42e8bc284c08b588`](https://github.com/wimi321/lizzieyzy-next/commit/94706b7be6199678e2bb04aa42e8bc284c08b588) Merge branch 'fix/humansl-thread-alias-20260917' into fix/startup-benchmark-user-work-20260917 | UD-04-013 | UDX-024 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`e30018411b2ab54e6844ec567ff6fb5e2bf47aeb`](https://github.com/wimi321/lizzieyzy-next/commit/e30018411b2ab54e6844ec567ff6fb5e2bf47aeb) Merge pull request #503 from wimi321/fix/humansl-thread-alias-20260917 | UD-04-012 | UDX-003 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`e07bb0eb31b5b53fb11fc2c2fd67da9955e563d2`](https://github.com/wimi321/lizzieyzy-next/commit/e07bb0eb31b5b53fb11fc2c2fd67da9955e563d2) Merge remote-tracking branch 'origin/main' into fix/startup-benchmark-user-work-20260917 | UD-04-013 | UDX-024 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`841f90b5ac65ef65673d79d57e04ab349ff51285`](https://github.com/wimi321/lizzieyzy-next/commit/841f90b5ac65ef65673d79d57e04ab349ff51285) Prepare verified pinned-source prerelease and audit static zlib correctly | UD-04-014 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`9f086cebfe235dde813128152903c77e546fe9b9`](https://github.com/wimi321/lizzieyzy-next/commit/9f086cebfe235dde813128152903c77e546fe9b9) Merge pull request #504 from wimi321/fix/startup-benchmark-user-work-20260917 | UD-04-013 | UDX-024 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`a18cd792f47e76db5de7da972da3b9ffac16938d`](https://github.com/wimi321/lizzieyzy-next/commit/a18cd792f47e76db5de7da972da3b9ffac16938d) Merge remote-tracking branch 'origin/main' into release/prerelease-focus-20260917-v5 | UD-04-014 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`beb498f5707972f5e6bc7c014a60d8f2baa7b52b`](https://github.com/wimi321/lizzieyzy-next/commit/beb498f5707972f5e6bc7c014a60d8f2baa7b52b) Merge pull request #505 from wimi321/release/prerelease-focus-20260917-v5 | UD-04-014 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`9599cfac1a2be63de31a506de5b283136e04bae7`](https://github.com/wimi321/lizzieyzy-next/commit/9599cfac1a2be63de31a506de5b283136e04bae7) Respect remaining HumanSL move time in KataGo searches | UD-04-015 | UDX-003 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`ecea2cdf9aa3845340e5fb961566b75be63156a8`](https://github.com/wimi321/lizzieyzy-next/commit/ecea2cdf9aa3845340e5fb961566b75be63156a8) Prepare time-budgeted same-tree analysis prerelease candidate | UD-04-016 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`8705f20827eb434f11cc00070ff6d61c716e2463`](https://github.com/wimi321/lizzieyzy-next/commit/8705f20827eb434f11cc00070ff6d61c716e2463) Merge pull request #506 from wimi321/fix/humansl-engine-time-budget-20260917 | UD-04-015 | UDX-003 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`9a93cb3b70477454cc345817c3e859e4026aa328`](https://github.com/wimi321/lizzieyzy-next/commit/9a93cb3b70477454cc345817c3e859e4026aa328) Merge remote-tracking branch 'origin/main' into release/prerelease-focus-20260917-v6 | UD-04-016 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`a99a69ab99656421b9a09148b68021f0700a7042`](https://github.com/wimi321/lizzieyzy-next/commit/a99a69ab99656421b9a09148b68021f0700a7042) Merge pull request #507 from wimi321/release/prerelease-focus-20260917-v6 | UD-04-016 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`c1ec182d3ac0a02c8e9bae191ae03c8863b7677c`](https://github.com/wimi321/lizzieyzy-next/commit/c1ec182d3ac0a02c8e9bae191ae03c8863b7677c) Preserve adaptive HumanSL search with weightless visit accounting | UD-04-017 | UDX-003 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`ecd0414ae6aa76d1bd5f0d6c15f039b3d82534ab`](https://github.com/wimi321/lizzieyzy-next/commit/ecd0414ae6aa76d1bd5f0d6c15f039b3d82534ab) Prepare final same-tree prerelease with bounded HumanSL verification | UD-04-018 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`e504299224559eaa96468facee0f70630e5e96a6`](https://github.com/wimi321/lizzieyzy-next/commit/e504299224559eaa96468facee0f70630e5e96a6) Merge pull request #508 from wimi321/fix/humansl-weightless-budget-20260917 | UD-04-017 | UDX-003 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`d01d27172ac9237d36dfda965e243978d29639da`](https://github.com/wimi321/lizzieyzy-next/commit/d01d27172ac9237d36dfda965e243978d29639da) Merge remote-tracking branch 'origin/main' into release/prerelease-focus-20260917-v7 | UD-04-018 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`1054141b0ccc530d68a1c0701cc7745f968afbf2`](https://github.com/wimi321/lizzieyzy-next/commit/1054141b0ccc530d68a1c0701cc7745f968afbf2) Merge pull request #509 from wimi321/release/prerelease-focus-20260917-v7 | UD-04-018 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`2e2d3dac15cd9da04086edf4e10f3e86168b4539`](https://github.com/wimi321/lizzieyzy-next/commit/2e2d3dac15cd9da04086edf4e10f3e86168b4539) Reject failed position responses during synchronization confirmation | UD-04-019 | UDX-015 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`50d6ff2783863efa0444a0e33de567f92145190e`](https://github.com/wimi321/lizzieyzy-next/commit/50d6ff2783863efa0444a0e33de567f92145190e) Keep headless rules fixture from scheduling uninitialized menu updates | UD-04-019 | UDX-015 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`1b7f44a4b696c2bcc2c98316e1fc2c38306f5c64`](https://github.com/wimi321/lizzieyzy-next/commit/1b7f44a4b696c2bcc2c98316e1fc2c38306f5c64) Merge pull request #510 from wimi321/fix/sync-timeout-race-20260918 | UD-04-019 | UDX-015 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`dddaeeef56afc34eed394ed9c9720605f7675d94`](https://github.com/wimi321/lizzieyzy-next/commit/dddaeeef56afc34eed394ed9c9720605f7675d94) Make release asset uploads retryable without overwriting completed files | UD-04-020 | UDX-015 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`5825960ce0214eeda63877ead54f27a5388bb4fd`](https://github.com/wimi321/lizzieyzy-next/commit/5825960ce0214eeda63877ead54f27a5388bb4fd) Keep HTTP status in bounded upload retry diagnostics | UD-04-020 | UDX-015 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`c892a1c46f387b6b1cc4dc0f5ecda44a3c47c698`](https://github.com/wimi321/lizzieyzy-next/commit/c892a1c46f387b6b1cc4dc0f5ecda44a3c47c698) Merge branch 'fix/sync-timeout-race-20260918' into fix/release-upload-recovery-20260918 | UD-04-020 | UDX-015 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`cfa115d4d4b5cba67f68fc0e88217320234b6f43`](https://github.com/wimi321/lizzieyzy-next/commit/cfa115d4d4b5cba67f68fc0e88217320234b6f43) Merge branch 'fix/sync-timeout-race-20260918' into fix/release-upload-recovery-20260918 | UD-04-020 | UDX-015 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`c96e672fc5ade639d2bb454b86fc6fe99a9df9db`](https://github.com/wimi321/lizzieyzy-next/commit/c96e672fc5ade639d2bb454b86fc6fe99a9df9db) Merge pull request #511 from wimi321/fix/release-upload-recovery-20260918 | UD-04-020 | UDX-015 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`a39e0b9f65cc74c31ad10a4022d6db10b1f88f09`](https://github.com/wimi321/lizzieyzy-next/commit/a39e0b9f65cc74c31ad10a4022d6db10b1f88f09) Prepare same-tree prerelease with verified synchronization and upload recovery | UD-04-021 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`d1683e229f84cbe6ffd239f9a4de6fac76b1f2a6`](https://github.com/wimi321/lizzieyzy-next/commit/d1683e229f84cbe6ffd239f9a4de6fac76b1f2a6) Merge pull request #512 from wimi321/release/prerelease-focus-20260918-v1 | UD-04-021 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`a4e1872bab549401b82548a682a5e97ebc9e40eb`](https://github.com/wimi321/lizzieyzy-next/commit/a4e1872bab549401b82548a682a5e97ebc9e40eb) Add verified cloud transfer for pinned release engine archives | UD-04-022 | UDX-002 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`cb987e2fce767af89b708653af15b01ede403a9d`](https://github.com/wimi321/lizzieyzy-next/commit/cb987e2fce767af89b708653af15b01ede403a9d) Merge pull request #513 from wimi321/fix/source-archive-cloud-transfer-20260918 | UD-04-022 | UDX-002 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`6c39d0a3973565704cbdf78c7735430ceece6572`](https://github.com/wimi321/lizzieyzy-next/commit/6c39d0a3973565704cbdf78c7735430ceece6572) Retain verified Windows packages for upload-only recovery | UD-04-023 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`e37bf9f3e269fbb58ff6eb14ddbeadd049b4d8a6`](https://github.com/wimi321/lizzieyzy-next/commit/e37bf9f3e269fbb58ff6eb14ddbeadd049b4d8a6) Wait for terminal restore workers before disposing test fixtures | UD-04-023 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`55b113e70a5f4d47b080c1a7d6e31d81e08c2c47`](https://github.com/wimi321/lizzieyzy-next/commit/55b113e70a5f4d47b080c1a7d6e31d81e08c2c47) Merge pull request #514 from wimi321/fix/windows-release-transfer-20260918 | UD-04-023 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`a935ec227ce667a364ff08cf7e0d97ebc4e3cdd4`](https://github.com/wimi321/lizzieyzy-next/commit/a935ec227ce667a364ff08cf7e0d97ebc4e3cdd4) Prepare verified same-tree analysis pre-release with recoverable uploads | UD-04-024 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`5930a09d979daac41eb0451dcd6fa8ccf5f2db1f`](https://github.com/wimi321/lizzieyzy-next/commit/5930a09d979daac41eb0451dcd6fa8ccf5f2db1f) Merge pull request #515 from wimi321/release/prerelease-focus-20260918-v2 | UD-04-024 | UDX-023 | next-2026-09-18.2, next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`29ec26e316c642f5b70386f71d98f315c5926bd9`](https://github.com/wimi321/lizzieyzy-next/commit/29ec26e316c642f5b70386f71d98f315c5926bd9) feat(release): add Windows product acceptance | UD-04-026 | UDX-002 | next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`80133df74f62938366fa3444aa1862b8a8465b76`](https://github.com/wimi321/lizzieyzy-next/commit/80133df74f62938366fa3444aa1862b8a8465b76) feat(release): add Linux product acceptance | UD-04-026 | UDX-002 | next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`72ff67ebac41ac4ab924ee0e2d2e6be51b63dde0`](https://github.com/wimi321/lizzieyzy-next/commit/72ff67ebac41ac4ab924ee0e2d2e6be51b63dde0) feat(release): add macOS product acceptance | UD-04-026 | UDX-002 | next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`58cdb5512d3de1ccccf17cc01311354ed66e3345`](https://github.com/wimi321/lizzieyzy-next/commit/58cdb5512d3de1ccccf17cc01311354ed66e3345) feat(runtime): verify standalone Java 17 | UD-04-026 | UDX-002 | next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`c1808af0e62367989d51f6c8f81c7a8ab50c5998`](https://github.com/wimi321/lizzieyzy-next/commit/c1808af0e62367989d51f6c8f81c7a8ab50c5998) ci(release): add candidate acceptance workflows | UD-04-026 | UDX-002 | next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`ed64305b5a8d352ccc19d1cec807f83322e57f22`](https://github.com/wimi321/lizzieyzy-next/commit/ed64305b5a8d352ccc19d1cec807f83322e57f22) fix(tensorrt): complete controlled repair flow | UD-04-026 | UDX-002 | next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`a5d1e8ecae41c418612926d04855aca02a14f0cd`](https://github.com/wimi321/lizzieyzy-next/commit/a5d1e8ecae41c418612926d04855aca02a14f0cd) test: keep comment worker Java 17 compatible | UD-04-026 | UDX-002 | next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`41f4cff8961d1138854a0ce449bcba64f9df0787`](https://github.com/wimi321/lizzieyzy-next/commit/41f4cff8961d1138854a0ce449bcba64f9df0787) fix(release): validate packaged JVM host | UD-04-026 | UDX-002 | next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`d5a0e2ed648df262b9e8f82cf6ef999c131ba9de`](https://github.com/wimi321/lizzieyzy-next/commit/d5a0e2ed648df262b9e8f82cf6ef999c131ba9de) ci: consume existing product artifacts | UD-04-026 | UDX-002 | next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`258303be3c105e8a2d3d872250808c75721ae7ea`](https://github.com/wimi321/lizzieyzy-next/commit/258303be3c105e8a2d3d872250808c75721ae7ea) fix(tensorrt): trust static zlib provenance | UD-04-026 | UDX-002 | next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`3a7ea04501f6b85d943f2f160e4f412fd230e64d`](https://github.com/wimi321/lizzieyzy-next/commit/3a7ea04501f6b85d943f2f160e4f412fd230e64d) fix(tensorrt): clarify modern GPU support | UD-04-026 | UDX-002 | next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`c8b3d5b9b7d6cb97d210636ed203fecbbc04e930`](https://github.com/wimi321/lizzieyzy-next/commit/c8b3d5b9b7d6cb97d210636ed203fecbbc04e930) test(release): align source zlib fixture | UD-04-026 | UDX-002 | next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`78dc2c247067f6b00a72745db4b88c3905292cfc`](https://github.com/wimi321/lizzieyzy-next/commit/78dc2c247067f6b00a72745db4b88c3905292cfc) ci: scope source builds to platform inputs | UD-04-026 | UDX-002 | next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`a13665dd49d50bb8b5c1e760755494e1dbb339df`](https://github.com/wimi321/lizzieyzy-next/commit/a13665dd49d50bb8b5c1e760755494e1dbb339df) ci: make KataGo source builds manual-only | UD-04-026 | UDX-002 | next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`1164bdabd6684e3b8921abedbef159b60a9afbe4`](https://github.com/wimi321/lizzieyzy-next/commit/1164bdabd6684e3b8921abedbef159b60a9afbe4) fix(acceptance): preserve candidate identity | UD-04-026 | UDX-002 | next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`188c925d5a91c429bb7701b128088df75db88dc2`](https://github.com/wimi321/lizzieyzy-next/commit/188c925d5a91c429bb7701b128088df75db88dc2) fix(engine): complete deferred KataGo rollback | UD-04-026 | UDX-002 | next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`f2645998827b0593447f82ed8c390640037f4454`](https://github.com/wimi321/lizzieyzy-next/commit/f2645998827b0593447f82ed8c390640037f4454) fix(test): await Windows process teardown | UD-04-026 | UDX-002 | next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`fcc5f71fc9af245314e7ef3aecc1cb887b6104a4`](https://github.com/wimi321/lizzieyzy-next/commit/fcc5f71fc9af245314e7ef3aecc1cb887b6104a4) feat(release): add Windows product acceptance | UD-04-025 | UDX-023 | next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`d1a9a8de23f69e7d76207889564e804c3d06b999`](https://github.com/wimi321/lizzieyzy-next/commit/d1a9a8de23f69e7d76207889564e804c3d06b999) feat(release): add Linux product acceptance | UD-04-025 | UDX-023 | next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`0915462ca991d0f7aff15f484d58c300c5055e9e`](https://github.com/wimi321/lizzieyzy-next/commit/0915462ca991d0f7aff15f484d58c300c5055e9e) feat(release): add macOS product acceptance | UD-04-025 | UDX-023 | next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`14a2348512830f69b95a6aa8f2d094b47edf75e8`](https://github.com/wimi321/lizzieyzy-next/commit/14a2348512830f69b95a6aa8f2d094b47edf75e8) feat(runtime): verify standalone Java 17 | UD-04-025 | UDX-023 | next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`6ce3e5d703b72ae481ce9de2f73fbafb213ae37d`](https://github.com/wimi321/lizzieyzy-next/commit/6ce3e5d703b72ae481ce9de2f73fbafb213ae37d) ci(release): add candidate acceptance workflows | UD-04-025 | UDX-023 | next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`9f2270cfdf830d5506641407b812c07954c46c62`](https://github.com/wimi321/lizzieyzy-next/commit/9f2270cfdf830d5506641407b812c07954c46c62) docs(acceptance): reconcile runtime evidence | UD-04-025 | UDX-023 | next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`c610b36b6f446b461677c04f49f7d998413988b8`](https://github.com/wimi321/lizzieyzy-next/commit/c610b36b6f446b461677c04f49f7d998413988b8) fix(test): keep executor cleanup Java 17 compatible | UD-04-025 | UDX-023 | next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`6e0ebd2918839dbbc65d5e168ab83d954e29010f`](https://github.com/wimi321/lizzieyzy-next/commit/6e0ebd2918839dbbc65d5e168ab83d954e29010f) fix(acceptance): harden runtime evidence | UD-04-025 | UDX-023 | next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`09453697968c0dc95fe87309a40448210e2ecbae`](https://github.com/wimi321/lizzieyzy-next/commit/09453697968c0dc95fe87309a40448210e2ecbae) ci: consume existing product artifacts | UD-04-025 | UDX-023 | next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`e072e143cb848e054347e6c5157241567755915a`](https://github.com/wimi321/lizzieyzy-next/commit/e072e143cb848e054347e6c5157241567755915a) Merge pull request #522 from qiyi71w/impl/distribution-runtime | UD-04-025 | UDX-023 | next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`8e876f251983361b7d68bd13c4120b0f2430b8cb`](https://github.com/wimi321/lizzieyzy-next/commit/8e876f251983361b7d68bd13c4120b0f2430b8cb) chore: merge distribution runtime acceptance | UD-04-026 | UDX-002 | next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`6ec5df985be570c6b03f65b00b1b08944d9bb1e6`](https://github.com/wimi321/lizzieyzy-next/commit/6ec5df985be570c6b03f65b00b1b08944d9bb1e6) Merge pull request #519 from qiyi71w/integration/tensorrt-candidate | UD-04-026 | UDX-002 | next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`d128c029d70b648faaf571deeb33713bda83af7d`](https://github.com/wimi321/lizzieyzy-next/commit/d128c029d70b648faaf571deeb33713bda83af7d) fix(windows): keep board usable after engine startup failure | UD-04-027 | UDX-026 | next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`2a487ced3bdf0d0557478313a77758492cbb6bc9`](https://github.com/wimi321/lizzieyzy-next/commit/2a487ced3bdf0d0557478313a77758492cbb6bc9) Merge pull request #524 from wimi321/codex/windows-qa-20260922 | UD-04-027 | UDX-026 | next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`12440af64a784837d4887d43c41b2b7a99b4bbbf`](https://github.com/wimi321/lizzieyzy-next/commit/12440af64a784837d4887d43c41b2b7a99b4bbbf) chore(release): prepare Windows stability pre-release next-2026-09-22.1 | UD-04-028 | UDX-023 | next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`3df8606879fa3a767f2925caa6e7933d4a4d6d19`](https://github.com/wimi321/lizzieyzy-next/commit/3df8606879fa3a767f2925caa6e7933d4a4d6d19) Merge pull request #525 from wimi321/codex/release-next-2026-09-22-1 | UD-04-028 | UDX-023 | next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`275f31c6e995c8fee2b9df0f7aff9e2a7b283e48`](https://github.com/wimi321/lizzieyzy-next/commit/275f31c6e995c8fee2b9df0f7aff9e2a7b283e48) fix(qa): capture delayed Windows engine startup before freezing process identity | UD-04-029 | UDX-004 | next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`e7391cfa930168a685a9f751fa9af5db8880100c`](https://github.com/wimi321/lizzieyzy-next/commit/e7391cfa930168a685a9f751fa9af5db8880100c) fix(qa): preserve startup failure when launcher already exited | UD-04-029 | UDX-004 | next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`1215d5a7adf6adb13883da38163c38f47cf9d0fb`](https://github.com/wimi321/lizzieyzy-next/commit/1215d5a7adf6adb13883da38163c38f47cf9d0fb) fix(qa): make native acceptance fixtures Unicode-safe | UD-04-029 | UDX-004 | next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`7898fe556529170659f30eee5b13c840dbda5e02`](https://github.com/wimi321/lizzieyzy-next/commit/7898fe556529170659f30eee5b13c840dbda5e02) Merge pull request #526 from wimi321/codex/windows-acceptance-startup-20260922 | UD-04-029 | UDX-004 | next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`6846db3aba385a6eb051f558d856dfe37eedebae`](https://github.com/wimi321/lizzieyzy-next/commit/6846db3aba385a6eb051f558d856dfe37eedebae) fix(katago): read weight identity from headers (#527) | UD-04-030 | UDX-002 | next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`aee0e708b534789f0f46c4fd9bd7300c33ccdb49`](https://github.com/wimi321/lizzieyzy-next/commit/aee0e708b534789f0f46c4fd9bd7300c33ccdb49) fix(katago): preserve renamed bundled profiles (#528) | UD-04-031 | UDX-002 | next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`88d6b0bb4f7ea87930bf5b8297ef97cdca1b43b6`](https://github.com/wimi321/lizzieyzy-next/commit/88d6b0bb4f7ea87930bf5b8297ef97cdca1b43b6) fix(enginegame): simplify match rule choices | UD-04-034 | UDX-014 | next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`6b7944e266ee01c1fbee7c6456d755bbb4d83786`](https://github.com/wimi321/lizzieyzy-next/commit/6b7944e266ee01c1fbee7c6456d755bbb4d83786) fix(engine-game): synchronize runtime komi | UD-04-033 | UDX-028 | next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`ff2bb9de50ce82c78e54003dd93721c1f439b54d`](https://github.com/wimi321/lizzieyzy-next/commit/ff2bb9de50ce82c78e54003dd93721c1f439b54d) fix(analysis): restore ordinary batch startup | UD-04-032 | UDX-027 | next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`05391234f7107610b432b2304a6b81fdebadf17c`](https://github.com/wimi321/lizzieyzy-next/commit/05391234f7107610b432b2304a6b81fdebadf17c) Merge remote-tracking branch 'origin/main' into codex/batch-analysis-main-sync-20260922 | UD-04-032 | UDX-027 | next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`15d0cc3dabad8611edd9e00ae8a1432a19cfbbfc`](https://github.com/wimi321/lizzieyzy-next/commit/15d0cc3dabad8611edd9e00ae8a1432a19cfbbfc) test(analysis): await nested batch synchronization admission | UD-04-032 | UDX-027 | next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`7b116c09b274aa82c1174519bd66ba4275c5bdc8`](https://github.com/wimi321/lizzieyzy-next/commit/7b116c09b274aa82c1174519bd66ba4275c5bdc8) Merge pull request #529 from qiyi71w/fix/523-batch-analysis | UD-04-032 | UDX-027 | next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`b1b38bf3858b72365bb2ad20c7d6fe9bf1e031d0`](https://github.com/wimi321/lizzieyzy-next/commit/b1b38bf3858b72365bb2ad20c7d6fe9bf1e031d0) Merge latest main into runtime komi fix | UD-04-033 | UDX-028 | next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`750530649d31787e9c5dacfa75f8540cfbaccc76`](https://github.com/wimi321/lizzieyzy-next/commit/750530649d31787e9c5dacfa75f8540cfbaccc76) Keep komi editing and score graphs usable without an engine | UD-04-033 | UDX-028 | next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`a0135a8a6fa57606141326fb2f92b230fddaea6a`](https://github.com/wimi321/lizzieyzy-next/commit/a0135a8a6fa57606141326fb2f92b230fddaea6a) Merge pull request #530 from qiyi71w/fix/issue-518-runtime-komi | UD-04-033 | UDX-028 | next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`028b8b0fa48a4d4e6424cabaafd05813cd39ca99`](https://github.com/wimi321/lizzieyzy-next/commit/028b8b0fa48a4d4e6424cabaafd05813cd39ca99) Merge verified main into match rule picker fix | UD-04-034 | UDX-014 | next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`84bfcd5f8e45916f6b58027ed44a3d2f127e4c94`](https://github.com/wimi321/lizzieyzy-next/commit/84bfcd5f8e45916f6b58027ed44a3d2f127e4c94) Merge pull request #531 from qiyi71w/fix/issue-516-match-rules | UD-04-034 | UDX-014 | next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`aec4f4435e12c082504298e1a3e291ac19646a4a`](https://github.com/wimi321/lizzieyzy-next/commit/aec4f4435e12c082504298e1a3e291ac19646a4a) chore(release): prepare fully integrated Windows stability pre-release .2 | UD-04-035 | UDX-023 | next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`9cc51e3b6f00f2982d9666190906d5eb1f8df833`](https://github.com/wimi321/lizzieyzy-next/commit/9cc51e3b6f00f2982d9666190906d5eb1f8df833) Merge branch 'codex/pr531-integration-20260922' into codex/release-next-2026-09-22-2 | UD-04-035 | UDX-023 | next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`5d43f3a8b2a4655a6f7b7370a447ff5f649a9bf3`](https://github.com/wimi321/lizzieyzy-next/commit/5d43f3a8b2a4655a6f7b7370a447ff5f649a9bf3) Integrate reviewed match rule choices and final source verification | UD-04-035 | UDX-023 | next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`8e6b76bd54c6a6927214bccb258ae86b33788603`](https://github.com/wimi321/lizzieyzy-next/commit/8e6b76bd54c6a6927214bccb258ae86b33788603) Merge pull request #532 from wimi321/codex/release-next-2026-09-22-2 | UD-04-035 | UDX-023 | next-2026-09-22.2, next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`2e1e08c9e90f22738cad525f3bd4fbb5813bb0a7`](https://github.com/wimi321/lizzieyzy-next/commit/2e1e08c9e90f22738cad525f3bd4fbb5813bb0a7) fix(qa): accept ready no-engine live sessions (#533) | UD-04-036 | UDX-029 | next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`498f5f05119e7a7cbab6ed124d8656fcd7940196`](https://github.com/wimi321/lizzieyzy-next/commit/498f5f05119e7a7cbab6ed124d8656fcd7940196) chore(models): upgrade default B11 to 11750M checkpoint | UD-04-037 | UDX-002 | next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`886bd81d0c01c8a3b57940dfab15c228a2e12146`](https://github.com/wimi321/lizzieyzy-next/commit/886bd81d0c01c8a3b57940dfab15c228a2e12146) QA integrate PR 534 | UD-04-038 | UDX-002 | next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`2d4787e63c57a0c77b43199791f7484d6195ff12`](https://github.com/wimi321/lizzieyzy-next/commit/2d4787e63c57a0c77b43199791f7484d6195ff12) fix(gui): resolve Windows user-flow QA regressions | UD-04-039 | UDX-030 | next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`3a0b27e6fd9200f1e00bd25800ff31262ed2862c`](https://github.com/wimi321/lizzieyzy-next/commit/3a0b27e6fd9200f1e00bd25800ff31262ed2862c) QA integrate PR 537 | UD-04-039 | UDX-030 | next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`558db3043e56bffc2fe364cac6e6c66ac77792f8`](https://github.com/wimi321/lizzieyzy-next/commit/558db3043e56bffc2fe364cac6e6c66ac77792f8) fix(gui): stabilize diagnostics dialog layout | UD-04-040 | UDX-001 | next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`085fba504918aebe26366216f918e18d4b15669c`](https://github.com/wimi321/lizzieyzy-next/commit/085fba504918aebe26366216f918e18d4b15669c) chore(repo): untrack local scratch evidence | UD-04-040 | UDX-001 | next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`7a64b2b20e194e24a1e9ec822e7ff63d17f82fe4`](https://github.com/wimi321/lizzieyzy-next/commit/7a64b2b20e194e24a1e9ec822e7ff63d17f82fe4) QA integrate PR 538 | UD-04-040 | UDX-001 | next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`c1308069813aa6091aceda751b24c612d372dcda`](https://github.com/wimi321/lizzieyzy-next/commit/c1308069813aa6091aceda751b24c612d372dcda) test(sync): verify producer snapshot confirmation | UD-04-041 | UDX-016 | next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`4ef8bbf5dad119728045a9cbf8720207c4ef6820`](https://github.com/wimi321/lizzieyzy-next/commit/4ef8bbf5dad119728045a9cbf8720207c4ef6820) test: await retirement before restarting engine game | UD-04-041 | UDX-016 | next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`10b1781ed3483ebc554fc5118ed9ee00d19537c3`](https://github.com/wimi321/lizzieyzy-next/commit/10b1781ed3483ebc554fc5118ed9ee00d19537c3) QA integrate PR 539 | UD-04-041 | UDX-016 | next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`e7c032830fad95e7f319655d792e91a845979302`](https://github.com/wimi321/lizzieyzy-next/commit/e7c032830fad95e7f319655d792e91a845979302) test(perf): measure fixed-budget engine and application analysis separately | UD-04-042 | UDX-024 | next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`a9d36e5f19ff665b0a7f1bef4153972aeef0f73c`](https://github.com/wimi321/lizzieyzy-next/commit/a9d36e5f19ff665b0a7f1bef4153972aeef0f73c) test(perf): confirm restoration and actual process evidence before timing | UD-04-042 | UDX-024 | next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`b7ba99e579044961e417ea9681565017cf8bafdc`](https://github.com/wimi321/lizzieyzy-next/commit/b7ba99e579044961e417ea9681565017cf8bafdc) test(perf): drain ordinary restore stages without superseding their lineage | UD-04-042 | UDX-024 | next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`55c2d54b3b9beeb331a8d6a877c40b027fee0601`](https://github.com/wimi321/lizzieyzy-next/commit/55c2d54b3b9beeb331a8d6a877c40b027fee0601) docs(perf): record v2 probe validation and measurement boundaries | UD-04-042 | UDX-024 | next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`8a6630e8b9898f1b8fda6dab21230d54799dc6ec`](https://github.com/wimi321/lizzieyzy-next/commit/8a6630e8b9898f1b8fda6dab21230d54799dc6ec) QA integrate PR 540 | UD-04-042 | UDX-024 | next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`94cdfc191e88ac404d9d7666ec645e15eccc6660`](https://github.com/wimi321/lizzieyzy-next/commit/94cdfc191e88ac404d9d7666ec645e15eccc6660) perf(gui): reuse immutable background weight catalog snapshots | UD-04-043 | UDX-002 | next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`5414286e949204f01a077ce772566103eb69541e`](https://github.com/wimi321/lizzieyzy-next/commit/5414286e949204f01a077ce772566103eb69541e) fix(gui): restore refresh controls and revalidate weight selection | UD-04-043 | UDX-002 | next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`5c96e1979583173c4ba3ac0f086e0a94eb910d89`](https://github.com/wimi321/lizzieyzy-next/commit/5c96e1979583173c4ba3ac0f086e0a94eb910d89) fix(gui): share busy controls during catalog refresh | UD-04-043 | UDX-002 | next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`b19808838c10112962fdd6c338a1e21463f454a0`](https://github.com/wimi321/lizzieyzy-next/commit/b19808838c10112962fdd6c338a1e21463f454a0) fix(gui): keep failed catalog model actions disabled | UD-04-043 | UDX-002 | next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`b7da149c6c8dc30828321b2c509d576727cec9a7`](https://github.com/wimi321/lizzieyzy-next/commit/b7da149c6c8dc30828321b2c509d576727cec9a7) test(gui): initialize completion buttons in catalog fixture | UD-04-043 | UDX-002 | next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`e91904197ca0ef922b2650299c0862d70e7275a8`](https://github.com/wimi321/lizzieyzy-next/commit/e91904197ca0ef922b2650299c0862d70e7275a8) QA integrate PR 541 | UD-04-043 | UDX-002 | next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`4d7f93ed6aff3b57745c445d3d40bf0697cee95a`](https://github.com/wimi321/lizzieyzy-next/commit/4d7f93ed6aff3b57745c445d3d40bf0697cee95a) perf(web): coalesce board notifications into one pending update | UD-04-044 | UDX-031 | next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`55ac71af0de1fdcec1288c53091cb2b2dd0fe347`](https://github.com/wimi321/lizzieyzy-next/commit/55ac71af0de1fdcec1288c53091cb2b2dd0fe347) perf(web): bound state buffering for slow websocket clients | UD-04-044 | UDX-031 | next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`f5f47ce6195b775aaaec3495e600ae84a70fe865`](https://github.com/wimi321/lizzieyzy-next/commit/f5f47ce6195b775aaaec3495e600ae84a70fe865) QA integrate PR 542 | UD-04-044 | UDX-031 | next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`948f653d04bb92409849c6dbb16fcea69618b3d8`](https://github.com/wimi321/lizzieyzy-next/commit/948f653d04bb92409849c6dbb16fcea69618b3d8) refactor(save): capture SGF snapshots and atomically write off the EDT | UD-04-045 | UDX-030 | next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`6bf3129d7cfc89d3bfdd9e585ba0e004d716f12a`](https://github.com/wimi321/lizzieyzy-next/commit/6bf3129d7cfc89d3bfdd9e585ba0e004d716f12a) fix(save): freeze tree structure before cloning analysis payloads | UD-04-045 | UDX-030 | next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`9298e061290f72f3e62a0b66dc69b26b71d1a120`](https://github.com/wimi321/lizzieyzy-next/commit/9298e061290f72f3e62a0b66dc69b26b71d1a120) test(save): type into the standard chooser filename field | UD-04-045 | UDX-030 | next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`f35af66eb7c43ecf348d3d8c26293e5e76f87598`](https://github.com/wimi321/lizzieyzy-next/commit/f35af66eb7c43ecf348d3d8c26293e5e76f87598) QA integrate PR 543 | UD-04-045 | UDX-030 | next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`4cb8e7530146b73955cb8050b6bc88f7b0d1e352`](https://github.com/wimi321/lizzieyzy-next/commit/4cb8e7530146b73955cb8050b6bc88f7b0d1e352) feat(engine): retain startup failure diagnostics | UD-04-046 | UDX-001 | next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`9e7f1942b356596501f31303668ca6d4fc95f370`](https://github.com/wimi321/lizzieyzy-next/commit/9e7f1942b356596501f31303668ca6d4fc95f370) fix(engine): refine startup failure diagnostics | UD-04-046 | UDX-001 | next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`064ed9c4e0de90967eb1dd4d00a6a859bad7c1c1`](https://github.com/wimi321/lizzieyzy-next/commit/064ed9c4e0de90967eb1dd4d00a6a859bad7c1c1) feat(engine): explain startup failure evidence | UD-04-046 | UDX-001 | next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`fedeb66cd182fe6e9532ea5a032fe1f0db790560`](https://github.com/wimi321/lizzieyzy-next/commit/fedeb66cd182fe6e9532ea5a032fe1f0db790560) feat(engine): trace native PE startup dependencies | UD-04-046 | UDX-001 | next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`5001b532ba194014b7fe8376daa08a2c3e7f819d`](https://github.com/wimi321/lizzieyzy-next/commit/5001b532ba194014b7fe8376daa08a2c3e7f819d) feat(engine): diagnose independent launchers | UD-04-046 | UDX-001 | next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`2a7507ac901800652a686190a598ae4474c1f397`](https://github.com/wimi321/lizzieyzy-next/commit/2a7507ac901800652a686190a598ae4474c1f397) feat(engine): integrate startup diagnostics | UD-04-046 | UDX-001 | next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`b3c4eec18c421317c321418a7900dee67bc1a84b`](https://github.com/wimi321/lizzieyzy-next/commit/b3c4eec18c421317c321418a7900dee67bc1a84b) test(engine): cover native probe dependency failure | UD-04-046 | UDX-001 | next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`b82833a592fa6567cf817ef8ec4c93b63a54266c`](https://github.com/wimi321/lizzieyzy-next/commit/b82833a592fa6567cf817ef8ec4c93b63a54266c) test(engine): compare sanitized probe export evidence | UD-04-046 | UDX-001 | next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`b0e0e07cc79b4ba71ef9e5f1f40d420d5c73adf1`](https://github.com/wimi321/lizzieyzy-next/commit/b0e0e07cc79b4ba71ef9e5f1f40d420d5c73adf1) test(engine): exercise native search and runtime evidence in desktop | UD-04-046 | UDX-001 | next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`138e0d17d8ddbe608c667e970281f447a73ad00c`](https://github.com/wimi321/lizzieyzy-next/commit/138e0d17d8ddbe608c667e970281f447a73ad00c) test(engine): retain integrated native diagnostic artifacts | UD-04-046 | UDX-001 | next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`cc47b2cde023a1ccc0399958ad1cf25c9c49b2c7`](https://github.com/wimi321/lizzieyzy-next/commit/cc47b2cde023a1ccc0399958ad1cf25c9c49b2c7) fix(gui): preserve engine failure window layout | UD-04-046 | UDX-001 | next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`80940479be71be37e9da4e4cb137438e4d71f240`](https://github.com/wimi321/lizzieyzy-next/commit/80940479be71be37e9da4e4cb137438e4d71f240) test(engine): fix startup diagnostic CI regressions | UD-04-046 | UDX-001 | next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`7f2dc02f53eb7b17f12e4c48df87d95bcfa50936`](https://github.com/wimi321/lizzieyzy-next/commit/7f2dc02f53eb7b17f12e4c48df87d95bcfa50936) feat(engine): focus startup failure diagnostics | UD-04-046 | UDX-001 | next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`75f1863a15fa8a8e0ccec29ab126c156ec99a92f`](https://github.com/wimi321/lizzieyzy-next/commit/75f1863a15fa8a8e0ccec29ab126c156ec99a92f) QA integrate PR 545 | UD-04-046 | UDX-001 | next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`73f025926b59e26ce8826e6b886ef606b5849f0a`](https://github.com/wimi321/lizzieyzy-next/commit/73f025926b59e26ce8826e6b886ef606b5849f0a) feat(tuning): review and confirm measured scene-specific profiles | UD-04-047 | UDX-024 | next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`f4ccaef5470b8da87ee5ccd0ca30fefa2a9bded4`](https://github.com/wimi321/lizzieyzy-next/commit/f4ccaef5470b8da87ee5ccd0ca30fefa2a9bded4) fix(tuning): normalize per-run log output locations | UD-04-047 | UDX-024 | next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`8bf99cca5ffb322f6b6955f841278cb009408270`](https://github.com/wimi321/lizzieyzy-next/commit/8bf99cca5ffb322f6b6955f841278cb009408270) fix(tuning): reject unsupported scenes and invalidate closed reviews | UD-04-047 | UDX-024 | next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`daa76f86200a0dd8e7c87672aa117c0f636577cc`](https://github.com/wimi321/lizzieyzy-next/commit/daa76f86200a0dd8e7c87672aa117c0f636577cc) feat(perf): summarize and export measured analysis evidence | UD-04-047 | UDX-024 | next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`c281b67db69a5c674bdc8cd278cc5f8dcb7d9b10`](https://github.com/wimi321/lizzieyzy-next/commit/c281b67db69a5c674bdc8cd278cc5f8dcb7d9b10) test(perf): gate measured exports on reliable process evidence | UD-04-047 | UDX-024 | next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`437e3b6c84dfa60a1693400bb65f7c9fefefc793`](https://github.com/wimi321/lizzieyzy-next/commit/437e3b6c84dfa60a1693400bb65f7c9fefefc793) fix(tuning): share setup task gate and bind confirmation identity | UD-04-047 | UDX-024 | next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`24752abd28fff870c32dfdb3a6d97e5db7a2f182`](https://github.com/wimi321/lizzieyzy-next/commit/24752abd28fff870c32dfdb3a6d97e5db7a2f182) test(tuning): assert startup honors resolved launch thread override | UD-04-047 | UDX-024 | next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`b29d2765e362197b9f027193703b7093caecd35b`](https://github.com/wimi321/lizzieyzy-next/commit/b29d2765e362197b9f027193703b7093caecd35b) fix(tuning): qualify sampled GPU memory at explicit apply confirmation | UD-04-047 | UDX-024 | next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`186a4d48b945f17653b6a508176f89f0bbfc6a31`](https://github.com/wimi321/lizzieyzy-next/commit/186a4d48b945f17653b6a508176f89f0bbfc6a31) docs(perf): publish controlled Windows evidence and acceptance limits | UD-04-047 | UDX-024 | next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`123e168e4730f4655a0b0b2618afa32077789486`](https://github.com/wimi321/lizzieyzy-next/commit/123e168e4730f4655a0b0b2618afa32077789486) QA integrate PR 546 | UD-04-047 | UDX-024 | next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`c58919c4257886baac9b9e4b7387a08643edf65c`](https://github.com/wimi321/lizzieyzy-next/commit/c58919c4257886baac9b9e4b7387a08643edf65c) fix(qa): honor explicit Python and portable Windows test paths | UD-04-048 | UDX-017 | next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`b6eeeef64ae77a093468b7b47d7059835fa8282c`](https://github.com/wimi321/lizzieyzy-next/commit/b6eeeef64ae77a093468b7b47d7059835fa8282c) fix(gui): size performance actions before native layout and verify Windows saves | UD-04-048 | UDX-017 | next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`0bc7707a44a2e6545299dbff174afde0d744c6db`](https://github.com/wimi321/lizzieyzy-next/commit/0bc7707a44a2e6545299dbff174afde0d744c6db) Merge pull request #534 from wimi321/codex/upgrade-b11-11750m-20260922 | UD-04-037 | UDX-002 | next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`1baa60749cd255ed2e7ece4a827f709422a8b764`](https://github.com/wimi321/lizzieyzy-next/commit/1baa60749cd255ed2e7ece4a827f709422a8b764) Merge current main into Windows integration candidate | UD-04-048 | UDX-017 | next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`5aad75550e3e4a4877dd560905d704c1353fec86`](https://github.com/wimi321/lizzieyzy-next/commit/5aad75550e3e4a4877dd560905d704c1353fec86) fix(gui): keep measured tuning confirmation controls visible on Windows | UD-04-048 | UDX-017 | next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`37b41f6686c1db56a4ea06d86648d74d3642197b`](https://github.com/wimi321/lizzieyzy-next/commit/37b41f6686c1db56a4ea06d86648d74d3642197b) Merge Windows-validated PR integration #547 | UD-04-048 | UDX-017 | next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`f211a22087c95f589067191fd017fccb5edfc516`](https://github.com/wimi321/lizzieyzy-next/commit/f211a22087c95f589067191fd017fccb5edfc516) release: request next-2026-09-26.1 with Windows acceptance evidence | UD-05-01 | UDX-023 | next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`8e808201eefd5056f917807514dc51bebf1a448c`](https://github.com/wimi321/lizzieyzy-next/commit/8e808201eefd5056f917807514dc51bebf1a448c) release: bind unchanged pinned engines to the new candidate | UD-05-01 | UDX-023 | next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`50898233f5cce7d952b37cdb9ee005f780ea948f`](https://github.com/wimi321/lizzieyzy-next/commit/50898233f5cce7d952b37cdb9ee005f780ea948f) release: prepare next-2026-09-26.1 after Windows acceptance (#548) | UD-05-01 | UDX-023 | next-2026-09-26.1, next-2026-09-26.2, next-2026-10-01.1 |
| [`ee774cac8df2150c5349492059f83a99327d9ab2`](https://github.com/wimi321/lizzieyzy-next/commit/ee774cac8df2150c5349492059f83a99327d9ab2) fix(ui): refine remote model refresh icon | UD-05-02 | UDX-009 | next-2026-09-26.2, next-2026-10-01.1 |
| [`98a7610831330b4a3b08975fda092a2f4c80cafe`](https://github.com/wimi321/lizzieyzy-next/commit/98a7610831330b4a3b08975fda092a2f4c80cafe) Unify Swing workbench UI and fix Windows layout regressions | UD-05-03 | UDX-025 | next-2026-09-26.2, next-2026-10-01.1 |
| [`1a186485e17d9961fccbe180f234e326718a168e`](https://github.com/wimi321/lizzieyzy-next/commit/1a186485e17d9961fccbe180f234e326718a168e) fix(ui): complete Windows keyboard and screen-reader acceptance | UD-05-03 | UDX-025 | next-2026-09-26.2, next-2026-10-01.1 |
| [`36ce7e546c3dc4c77d12bd26b9f9bd81846eadf3`](https://github.com/wimi321/lizzieyzy-next/commit/36ce7e546c3dc4c77d12bd26b9f9bd81846eadf3) test(ui): isolate graph navigation fixtures and record final acceptance | UD-05-04 | UDX-023 | next-2026-09-26.2, next-2026-10-01.1 |
| [`a0e7b211d253ec09fa1bfe662f7a0a7c544c1238`](https://github.com/wimi321/lizzieyzy-next/commit/a0e7b211d253ec09fa1bfe662f7a0a7c544c1238) Merge PR #550: refine remote model refresh icon | UD-05-02 | UDX-009 | next-2026-09-26.2, next-2026-10-01.1 |
| [`849ef5cb51c513c7f994f26fc1dc81f2029ab627`](https://github.com/wimi321/lizzieyzy-next/commit/849ef5cb51c513c7f994f26fc1dc81f2029ab627) Merge remote-tracking branch 'origin/main' into ui/unified-analysis-workbench | UD-05-03 | UDX-025 | next-2026-09-26.2, next-2026-10-01.1 |
| [`b7285159f2eaf94771d1d41f2a028cc40e1d9bcc`](https://github.com/wimi321/lizzieyzy-next/commit/b7285159f2eaf94771d1d41f2a028cc40e1d9bcc) Merge PR #551: unify Swing analysis workbench UI | UD-05-03 | UDX-025 | next-2026-09-26.2, next-2026-10-01.1 |
| [`7b4ebcb87a5341efa97ae24c7e3c605e1d81b9a7`](https://github.com/wimi321/lizzieyzy-next/commit/7b4ebcb87a5341efa97ae24c7e3c605e1d81b9a7) Update bundled B11 to verified September 25 network | UD-05-05 | UDX-002 | next-2026-09-26.2, next-2026-10-01.1 |
| [`e9481f9a43ad817d80292cc1819a23c10d0d72d1`](https://github.com/wimi321/lizzieyzy-next/commit/e9481f9a43ad817d80292cc1819a23c10d0d72d1) Merge pull request #552 from wimi321/upgrade/b11-20260925 | UD-05-05 | UDX-002 | next-2026-09-26.2, next-2026-10-01.1 |
| [`89ff0200c20b7007e5ccd2e5d5c020b349041418`](https://github.com/wimi321/lizzieyzy-next/commit/89ff0200c20b7007e5ccd2e5d5c020b349041418) Request next-2026-09-26.2 workbench and B11 pre-release | UD-05-01 | UDX-023 | next-2026-09-26.2, next-2026-10-01.1 |
| [`74afda436fdcb45b7eece1a0776f498e9ce562d4`](https://github.com/wimi321/lizzieyzy-next/commit/74afda436fdcb45b7eece1a0776f498e9ce562d4) Allow CI to bootstrap release requests from identical pinned archives | UD-05-01 | UDX-023 | next-2026-09-26.2, next-2026-10-01.1 |
| [`3f58c0b62160bb0e46f7e9765da100371bb78b2d`](https://github.com/wimi321/lizzieyzy-next/commit/3f58c0b62160bb0e46f7e9765da100371bb78b2d) Merge pull request #553 from wimi321/release/next-2026-09-26.2 | UD-05-01 | UDX-023 | next-2026-09-26.2, next-2026-10-01.1 |
| [`41121557a82d6f447300671207c69c4edb95a6cc`](https://github.com/wimi321/lizzieyzy-next/commit/41121557a82d6f447300671207c69c4edb95a6cc) fix(sync): retain moves while Fox titles lag | UD-05-06 | UDX-016 | next-2026-10-01.1 |
| [`7c48b1c01fe07311f028809cae716fe145e1b341`](https://github.com/wimi321/lizzieyzy-next/commit/7c48b1c01fe07311f028809cae716fe145e1b341) Merge remote-tracking branch 'origin/pr/554' into review/all-pr-20260928 | UD-05-06 | UDX-016 | next-2026-10-01.1 |
| [`133d9ca6294ceaa52fb747c2c32c53c54de7bfe9`](https://github.com/wimi321/lizzieyzy-next/commit/133d9ca6294ceaa52fb747c2c32c53c54de7bfe9) fix(engine-game): restore ancient Chinese rules | UD-05-07 | UDX-014 | next-2026-10-01.1 |
| [`1c33a7862dabfbf6d04a2f544db293f80ec47e78`](https://github.com/wimi321/lizzieyzy-next/commit/1c33a7862dabfbf6d04a2f544db293f80ec47e78) Merge remote-tracking branch 'origin/pr/556' into review/all-pr-20260928 | UD-05-07 | UDX-014 | next-2026-10-01.1 |
| [`84c7e00d83f0f78cdd7619f76f89281120e4c8fa`](https://github.com/wimi321/lizzieyzy-next/commit/84c7e00d83f0f78cdd7619f76f89281120e4c8fa) fix(engine): unify startup failure diagnostics | UD-05-08 | UDX-001 | next-2026-10-01.1 |
| [`d017a5a07f37699b8d0e29cfe17e39b65f8125c1`](https://github.com/wimi321/lizzieyzy-next/commit/d017a5a07f37699b8d0e29cfe17e39b65f8125c1) test(engine): await final fence cleanup before assertions | UD-05-08 | UDX-001 | next-2026-10-01.1 |
| [`7a244119c6225f4ad583ef25b47f9415703e6b41`](https://github.com/wimi321/lizzieyzy-next/commit/7a244119c6225f4ad583ef25b47f9415703e6b41) Merge remote-tracking branch 'origin/pr/557' into review/all-pr-20260928 | UD-05-08 | UDX-001 | next-2026-10-01.1 |
| [`89e85d721f6f4a2b8dbaa2f89fb4223de33e3e72`](https://github.com/wimi321/lizzieyzy-next/commit/89e85d721f6f4a2b8dbaa2f89fb4223de33e3e72) fix(katago): restore runtime thread controls | UD-05-09 | UDX-017 | next-2026-10-01.1 |
| [`5b447d22017296a7cbd543d038880402e313f7ac`](https://github.com/wimi321/lizzieyzy-next/commit/5b447d22017296a7cbd543d038880402e313f7ac) fix(gui): unblock thread checkbox mouse input | UD-05-09 | UDX-017 | next-2026-10-01.1 |
| [`c49e299c14bdab1cb0fe94a8bd96901e49c1fbd7`](https://github.com/wimi321/lizzieyzy-next/commit/c49e299c14bdab1cb0fe94a8bd96901e49c1fbd7) fix(katago): preserve process thread overrides | UD-05-09 | UDX-017 | next-2026-10-01.1 |
| [`0812ab88b037cc0eb6387faf9b75c1b50ef6c98a`](https://github.com/wimi321/lizzieyzy-next/commit/0812ab88b037cc0eb6387faf9b75c1b50ef6c98a) fix(gui): show confirmed threads immediately | UD-05-09 | UDX-017 | next-2026-10-01.1 |
| [`d1577fefde3a29f50982392fdf864922e080cb33`](https://github.com/wimi321/lizzieyzy-next/commit/d1577fefde3a29f50982392fdf864922e080cb33) Merge remote-tracking branch 'origin/pr/561' into review/all-pr-20260928 | UD-05-09 | UDX-017 | next-2026-10-01.1 |
| [`e8f38d76ab16f0cbfe9e92644dc29482f0a4de1e`](https://github.com/wimi321/lizzieyzy-next/commit/e8f38d76ab16f0cbfe9e92644dc29482f0a4de1e) fix(logging): redact plink password arguments | UD-05-10 | UDX-001 | next-2026-10-01.1 |
| [`2ccf9995b33646361226989b1d2bfa3380b24b93`](https://github.com/wimi321/lizzieyzy-next/commit/2ccf9995b33646361226989b1d2bfa3380b24b93) fix(logging): preserve snapshot argument boundaries | UD-05-10 | UDX-001 | next-2026-10-01.1 |
| [`cda3977afb04bf5f2e2fb06af83602728b85ecc1`](https://github.com/wimi321/lizzieyzy-next/commit/cda3977afb04bf5f2e2fb06af83602728b85ecc1) Merge remote-tracking branch 'origin/pr/562' into review/all-pr-20260928 | UD-05-10 | UDX-001 | next-2026-10-01.1 |
| [`a49c10c77f0a28c0ad802fc9d3905d1dcabb03de`](https://github.com/wimi321/lizzieyzy-next/commit/a49c10c77f0a28c0ad802fc9d3905d1dcabb03de) fix(sgf): preserve imported game dates | UD-05-11 | UDX-032 | next-2026-10-01.1 |
| [`238d74a46dc7f9217f55d7a2d23708049279dd42`](https://github.com/wimi321/lizzieyzy-next/commit/238d74a46dc7f9217f55d7a2d23708049279dd42) Merge remote-tracking branch 'origin/pr/563' into review/all-pr-20260928 | UD-05-11 | UDX-032 | next-2026-10-01.1 |
| [`7bcef4dafc9ad6360c7f891ad0c2c3929e964f63`](https://github.com/wimi321/lizzieyzy-next/commit/7bcef4dafc9ad6360c7f891ad0c2c3929e964f63) fix(ui): restore default workspace texture | UD-05-12 | UDX-033 | next-2026-10-01.1 |
| [`90ff66f4f12c90c4dbebe41b593f14bcc0131bb5`](https://github.com/wimi321/lizzieyzy-next/commit/90ff66f4f12c90c4dbebe41b593f14bcc0131bb5) Merge remote-tracking branch 'origin/pr/564' into review/all-pr-20260928 | UD-05-12 | UDX-033 | next-2026-10-01.1 |
| [`cc91b449220c8569e0753108b86a584b3fab09c6`](https://github.com/wimi321/lizzieyzy-next/commit/cc91b449220c8569e0753108b86a584b3fab09c6) fix(logging): redact contribution passwords | UD-05-10 | UDX-001 | next-2026-10-01.1 |
| [`2d0c829308ae41db7e1ac17c82f9ba44e9741456`](https://github.com/wimi321/lizzieyzy-next/commit/2d0c829308ae41db7e1ac17c82f9ba44e9741456) fix(remote): hide credentials in connection notices | UD-05-10 | UDX-001 | next-2026-10-01.1 |
| [`d376bf54c2d33fc2aac5b6c15a79679dc4b7f331`](https://github.com/wimi321/lizzieyzy-next/commit/d376bf54c2d33fc2aac5b6c15a79679dc4b7f331) Merge remote-tracking branch 'origin/pr/565' into review/all-pr-20260928 | UD-05-10 | UDX-001 | next-2026-10-01.1 |
| [`dcc71136f547ed3fb6157cbdb14b4c8c23a7bd8f`](https://github.com/wimi321/lizzieyzy-next/commit/dcc71136f547ed3fb6157cbdb14b4c8c23a7bd8f) fix(engine): restore external handicap positions | UD-05-13 | UDX-015 | next-2026-10-01.1 |
| [`caff4fc1420670d01f058b2717c5081eadfda24e`](https://github.com/wimi321/lizzieyzy-next/commit/caff4fc1420670d01f058b2717c5081eadfda24e) test(engine): align restore fixtures with bindings | UD-05-04 | UDX-023 | next-2026-10-01.1 |
| [`542e8b0e2904849cbf078ec92b601ea1b0bde53a`](https://github.com/wimi321/lizzieyzy-next/commit/542e8b0e2904849cbf078ec92b601ea1b0bde53a) Merge remote-tracking branch 'origin/pr/566' into review/all-pr-20260928 | UD-05-13 | UDX-015 | next-2026-10-01.1 |
| [`fc94364b47f2becc8696b2294124ce6adca8ed94`](https://github.com/wimi321/lizzieyzy-next/commit/fc94364b47f2becc8696b2294124ce6adca8ed94) fix(readboard): resume autoplay after sync | UD-05-14 | UDX-034 | next-2026-10-01.1 |
| [`332bc2a43d4eb44d1fcce52ded111a76433c7114`](https://github.com/wimi321/lizzieyzy-next/commit/332bc2a43d4eb44d1fcce52ded111a76433c7114) fix(sync): preserve newest restore ownership | UD-05-14 | UDX-034 | next-2026-10-01.1 |
| [`66c4e2160eca86c99718c667cdda00da45c50d42`](https://github.com/wimi321/lizzieyzy-next/commit/66c4e2160eca86c99718c667cdda00da45c50d42) Merge remote-tracking branch 'origin/pr/567' into review/all-pr-20260928 | UD-05-14 | UDX-034 | next-2026-10-01.1 |
| [`567d71378d9bf9671a6151566b5d5cf78f1414e1`](https://github.com/wimi321/lizzieyzy-next/commit/567d71378d9bf9671a6151566b5d5cf78f1414e1) Fix fallback startup diagnostic credential boundaries | UD-05-10 | UDX-001 | next-2026-10-01.1 |
| [`04b5306336c08c5143f4482780381994a17e5b44`](https://github.com/wimi321/lizzieyzy-next/commit/04b5306336c08c5143f4482780381994a17e5b44) Publish restart fixture capabilities on the replacement reader | UD-05-04 | UDX-023 | next-2026-10-01.1 |
| [`7d30c8f41ddabb3d6723faff8337d3b51477ebed`](https://github.com/wimi321/lizzieyzy-next/commit/7d30c8f41ddabb3d6723faff8337d3b51477ebed) Avoid cached simulated platform in native process cleanup tests | UD-05-04 | UDX-023 | next-2026-10-01.1 |
| [`8c43fdd24b01acd6485b8b3911708290ae18ab75`](https://github.com/wimi321/lizzieyzy-next/commit/8c43fdd24b01acd6485b8b3911708290ae18ab75) Merge pull request #568 from wimi321/review/all-pr-20260928 | UD-05-04, UD-05-06, UD-05-07, UD-05-08, UD-05-09, UD-05-10, UD-05-11, UD-05-12, UD-05-13, UD-05-14 | UDX-001, UDX-014, UDX-015, UDX-016, UDX-017, UDX-023, UDX-032, UDX-033, UDX-034 | next-2026-10-01.1 |
| [`410f36dc9fe58a099785cf0f5fc1701cb9ca179b`](https://github.com/wimi321/lizzieyzy-next/commit/410f36dc9fe58a099785cf0f5fc1701cb9ca179b) Explain B11 speed and strength beside analysis metrics | UD-05-05 | UDX-002 | next-2026-10-01.1 |
| [`f59c59232119cfc47ad97a21ede94e6747d05b19`](https://github.com/wimi321/lizzieyzy-next/commit/f59c59232119cfc47ad97a21ede94e6747d05b19) Keep benchmark details readable on narrow high-DPI windows | UD-05-05 | UDX-002 | next-2026-10-01.1 |
| [`efa54a3700f7f137c56fbd130250ece77979914c`](https://github.com/wimi321/lizzieyzy-next/commit/efa54a3700f7f137c56fbd130250ece77979914c) Format narrow-window regression coverage | UD-05-04 | UDX-023 | next-2026-10-01.1 |
| [`dcd4803586328dd71419eb301a217fec5d20a7a5`](https://github.com/wimi321/lizzieyzy-next/commit/dcd4803586328dd71419eb301a217fec5d20a7a5) Preserve report scroll position and verify locale glyph coverage | UD-05-05 | UDX-002 | next-2026-10-01.1 |
| [`91ba01d3bd542a1ab14f03c8b970ee5a38c6db36`](https://github.com/wimi321/lizzieyzy-next/commit/91ba01d3bd542a1ab14f03c8b970ee5a38c6db36) Assert stable report navigation and honor metric preferred height | UD-05-05 | UDX-002 | next-2026-10-01.1 |
| [`01c7eafdd50b698c6d53e9515e355180829a4e39`](https://github.com/wimi321/lizzieyzy-next/commit/01c7eafdd50b698c6d53e9515e355180829a4e39) Merge PR #569 for Windows integration acceptance | UD-05-05 | UDX-002 | next-2026-10-01.1 |
| [`2a9a10ddd0ac7b28fda63e110c2de428128891fc`](https://github.com/wimi321/lizzieyzy-next/commit/2a9a10ddd0ac7b28fda63e110c2de428128891fc) Fix B11 model identity for indirect engine launchers | UD-05-05 | UDX-002 | next-2026-10-01.1 |
| [`d21aa344a339ed4b1a87f0135c23c92a3f5a30d7`](https://github.com/wimi321/lizzieyzy-next/commit/d21aa344a339ed4b1a87f0135c23c92a3f5a30d7) Guard benchmark B11 notice against indirect launchers | UD-05-05 | UDX-002 | next-2026-10-01.1 |
| [`0abe772bac0494517de33f61c39dc839ec7f8246`](https://github.com/wimi321/lizzieyzy-next/commit/0abe772bac0494517de33f61c39dc839ec7f8246) Merge PR #576 for Windows integration acceptance | UD-05-05 | UDX-002 | next-2026-10-01.1 |
| [`f14b928908d9036655cb1eef9ffe66e11d96e093`](https://github.com/wimi321/lizzieyzy-next/commit/f14b928908d9036655cb1eef9ffe66e11d96e093) fix(readboard): await GMA drain on mode switch | UD-05-15 | UDX-034 | next-2026-10-01.1 |
| [`53215d3f2545d88d3e9bf90f8553b3275f4a850e`](https://github.com/wimi321/lizzieyzy-next/commit/53215d3f2545d88d3e9bf90f8553b3275f4a850e) fix(readboard): isolate in-flight GMA decisions | UD-05-15 | UDX-034 | next-2026-10-01.1 |
| [`4f9e1964d167a35e2d009598eccaffb9d10c883f`](https://github.com/wimi321/lizzieyzy-next/commit/4f9e1964d167a35e2d009598eccaffb9d10c883f) fix(readboard): preserve GMA restore targets across mode changes | UD-05-15 | UDX-034 | next-2026-10-01.1 |
| [`8d3cc68a790812a568ea8d460d32fc50145d98e0`](https://github.com/wimi321/lizzieyzy-next/commit/8d3cc68a790812a568ea8d460d32fc50145d98e0) test(readboard): model user navigation cancellation | UD-05-04 | UDX-023 | next-2026-10-01.1 |
| [`f9a0b0cced0d64909038e52a3d67b6aaf349a101`](https://github.com/wimi321/lizzieyzy-next/commit/f9a0b0cced0d64909038e52a3d67b6aaf349a101) Merge PR #570 for Windows integration acceptance | UD-05-15 | UDX-034 | next-2026-10-01.1 |
| [`de4e3db7f488298f4301b8bc75c7daf10f1e78e1`](https://github.com/wimi321/lizzieyzy-next/commit/de4e3db7f488298f4301b8bc75c7daf10f1e78e1) fix(fox): decode professional ranks in game lists | UD-05-16 | UDX-035 | next-2026-10-01.1 |
| [`bb076b3b5b8fe3d99a3145e8948f40288c2e8ecf`](https://github.com/wimi321/lizzieyzy-next/commit/bb076b3b5b8fe3d99a3145e8948f40288c2e8ecf) Merge PR #571 for Windows integration acceptance | UD-05-16 | UDX-035 | next-2026-10-01.1 |
| [`aaef6f12f4c440b70bf1cf141c55d5065640675e`](https://github.com/wimi321/lizzieyzy-next/commit/aaef6f12f4c440b70bf1cf141c55d5065640675e) fix(analysis): preserve quick-analysis position | UD-05-17 | UDX-011 | next-2026-10-01.1 |
| [`ec19f8b847ca719c0189eaaa0177592094a8208a`](https://github.com/wimi321/lizzieyzy-next/commit/ec19f8b847ca719c0189eaaa0177592094a8208a) fix(sgf): import escaped structural newlines | UD-05-18 | UDX-036 | next-2026-10-01.1 |
| [`1c829ab080f604da019c5ab14f6e6c27782b8319`](https://github.com/wimi321/lizzieyzy-next/commit/1c829ab080f604da019c5ab14f6e6c27782b8319) fix(sgf): integrate structural newline import | UD-05-18 | UDX-036 | next-2026-10-01.1 |
| [`ac18ae1a2b620c465aba34798dab1890a8400b6f`](https://github.com/wimi321/lizzieyzy-next/commit/ac18ae1a2b620c465aba34798dab1890a8400b6f) fix(gui): publish only current variation trees | UD-05-19 | UDX-037 | next-2026-10-01.1 |
| [`ca74519d00672680ccdf101d47801f558da5cf20`](https://github.com/wimi321/lizzieyzy-next/commit/ca74519d00672680ccdf101d47801f558da5cf20) fix(gui): integrate current tree publication | UD-05-19 | UDX-037 | next-2026-10-01.1 |
| [`e9f11ac5a66584111fbf8d461294841bf1e3c608`](https://github.com/wimi321/lizzieyzy-next/commit/e9f11ac5a66584111fbf8d461294841bf1e3c608) perf(diagnostics): scan secret lines once | UD-05-10 | UDX-001 | next-2026-10-01.1 |
| [`bc4c318f462924462f03853494cc6aa454ae2770`](https://github.com/wimi321/lizzieyzy-next/commit/bc4c318f462924462f03853494cc6aa454ae2770) fix(sgf): integrate diagnostic export performance | UD-05-10 | UDX-001 | next-2026-10-01.1 |
| [`65cc93842eb26e0dd0ee911f3d16493b52cd6399`](https://github.com/wimi321/lizzieyzy-next/commit/65cc93842eb26e0dd0ee911f3d16493b52cd6399) Merge PR #572 for Windows integration acceptance | UD-05-10, UD-05-17, UD-05-18, UD-05-19 | UDX-001, UDX-011, UDX-036, UDX-037 | next-2026-10-01.1 |
| [`7c458bb358a8a845673a712bd7cbb9032936fe64`](https://github.com/wimi321/lizzieyzy-next/commit/7c458bb358a8a845673a712bd7cbb9032936fe64) fix(engine): synchronize PDA and WRN readback | UD-05-20 | UDX-038 | next-2026-10-01.1 |
| [`22b948c814acb29240a5153d12b48b88dbf24614`](https://github.com/wimi321/lizzieyzy-next/commit/22b948c814acb29240a5153d12b48b88dbf24614) test(engine): isolate shared fixture state | UD-05-04 | UDX-023 | next-2026-10-01.1 |
| [`c956c727c168e5694157b807fc7187ca111fbaf3`](https://github.com/wimi321/lizzieyzy-next/commit/c956c727c168e5694157b807fc7187ca111fbaf3) Merge PR #574 for Windows integration acceptance | UD-05-20 | UDX-038 | next-2026-10-01.1 |
| [`17fe30c1c049f35bf5eaeeab4f70a57221139ddc`](https://github.com/wimi321/lizzieyzy-next/commit/17fe30c1c049f35bf5eaeeab4f70a57221139ddc) Fix automatic SGF overview during asynchronous engine startup | UD-05-17 | UDX-011 | next-2026-10-01.1 |
| [`adb74114377fafdbb09433641791554b4967aded`](https://github.com/wimi321/lizzieyzy-next/commit/adb74114377fafdbb09433641791554b4967aded) Record final Windows regression and native EXE acceptance | UD-05-04 | UDX-023 | next-2026-10-01.1 |
| [`0fe5b3769e5f1402013df5520643466ff021f5e6`](https://github.com/wimi321/lizzieyzy-next/commit/0fe5b3769e5f1402013df5520643466ff021f5e6) Merge tested Windows PR acceptance and startup SGF fix (#580) | UD-05-04, UD-05-05, UD-05-10, UD-05-15, UD-05-16, UD-05-17, UD-05-18, UD-05-19, UD-05-20 | UDX-001, UDX-002, UDX-011, UDX-023, UDX-034, UDX-035, UDX-036, UDX-037, UDX-038 | next-2026-10-01.1 |
| [`87cdedc37dc87e48fd0f270a5722dbb5cfb5b351`](https://github.com/wimi321/lizzieyzy-next/commit/87cdedc37dc87e48fd0f270a5722dbb5cfb5b351) Prepare next-2026-10-01.1 verified multiplatform pre-release | UD-05-01 | UDX-023 | next-2026-10-01.1 |
| [`ce4f6e6b5af26a1e95a759fd14e2f7eab00ae608`](https://github.com/wimi321/lizzieyzy-next/commit/ce4f6e6b5af26a1e95a759fd14e2f7eab00ae608) Keep candidate-variation navigation at its first move (#577) | UD-05-21 | UDX-037 | next-2026-10-01.1 |
| [`070db3f5354044b3169752acb237030910f02cdf`](https://github.com/wimi321/lizzieyzy-next/commit/070db3f5354044b3169752acb237030910f02cdf) Merge branch 'main' into fix/577-branch-wheel-ownership | UD-05-21 | UDX-037 | next-2026-10-01.1 |
| [`6e2f9102a545cf0549f98521fbdc52a14d21a922`](https://github.com/wimi321/lizzieyzy-next/commit/6e2f9102a545cf0549f98521fbdc52a14d21a922) Keep candidate-variation navigation at its first move (#581) | UD-05-21 | UDX-037 | next-2026-10-01.1 |
| [`40bb84eb6acdd7ec40811ddcd446cf8c52b49ddb`](https://github.com/wimi321/lizzieyzy-next/commit/40bb84eb6acdd7ec40811ddcd446cf8c52b49ddb) Include verified candidate-preview navigation fix | UD-05-01 | UDX-023 | next-2026-10-01.1 |
| [`8600b61586dc99532bb32306519a1cc9001c6999`](https://github.com/wimi321/lizzieyzy-next/commit/8600b61586dc99532bb32306519a1cc9001c6999) Include preview navigation acceptance in multilingual release notes | UD-05-01 | UDX-023 | next-2026-10-01.1 |
| [`c013d8ce66afc186df5af96c83a61cbb3e4a14ba`](https://github.com/wimi321/lizzieyzy-next/commit/c013d8ce66afc186df5af96c83a61cbb3e4a14ba) Prepare verified next-2026-10-01.1 pre-release (#582) | UD-05-01 | UDX-023 | next-2026-10-01.1 |
| [`47d0a3e84849a1672ae9688bd0ce62c5af322158`](https://github.com/wimi321/lizzieyzy-next/commit/47d0a3e84849a1672ae9688bd0ce62c5af322158) fix(update): compare Beta release candidates | UD-05-22 | UDX-039 | 仅主线；无已核实已发布包含 |
| [`e260147d59f1ec3c91e8df19c492868a04fc6588`](https://github.com/wimi321/lizzieyzy-next/commit/e260147d59f1ec3c91e8df19c492868a04fc6588) Merge pull request #583 from qiyi71w/analysis/578-beta-candidates | UD-05-22 | UDX-039 | 仅主线；无已核实已发布包含 |
| [`f59239e706d58d8aeb76477616491dee748867f3`](https://github.com/wimi321/lizzieyzy-next/commit/f59239e706d58d8aeb76477616491dee748867f3) Merge remote-tracking branch 'origin/pr-583' into qa/windows-pr-acceptance-20261002 | UD-05-22 | UDX-039 | 仅主线；无已核实已发布包含 |
| [`acb8be03dc7249d81a7c2f9a4b1893472384b704`](https://github.com/wimi321/lizzieyzy-next/commit/acb8be03dc7249d81a7c2f9a4b1893472384b704) fix(gui): show complete suggestion rows | UD-05-23 | UDX-040 | 仅主线；无已核实已发布包含 |
| [`b4ab3ba7f599145dadfa5e8277e6a25b565bc53e`](https://github.com/wimi321/lizzieyzy-next/commit/b4ab3ba7f599145dadfa5e8277e6a25b565bc53e) fix(gui): stabilize live suggestion refresh | UD-05-23 | UDX-040 | 仅主线；无已核实已发布包含 |
| [`20efc9ef68c298dc09fb15216187475abdfb376c`](https://github.com/wimi321/lizzieyzy-next/commit/20efc9ef68c298dc09fb15216187475abdfb376c) fix(gui): enable suggestion scrollbar key input | UD-05-23 | UDX-040 | 仅主线；无已核实已发布包含 |
| [`baa677bd2b12523d5bee9ce4b252a3b07122ce33`](https://github.com/wimi321/lizzieyzy-next/commit/baa677bd2b12523d5bee9ce4b252a3b07122ce33) fix(gui): preserve game navigation focus | UD-05-23 | UDX-040 | 仅主线；无已核实已发布包含 |
| [`12d9fb1440a7a73916f546fe1baa3d5b6f6334a0`](https://github.com/wimi321/lizzieyzy-next/commit/12d9fb1440a7a73916f546fe1baa3d5b6f6334a0) chore: merge upstream main into suggestion fix | UD-05-23 | UDX-040 | 仅主线；无已核实已发布包含 |
| [`6d601ec540d267a6832a4b495b474e6a78bbb1ff`](https://github.com/wimi321/lizzieyzy-next/commit/6d601ec540d267a6832a4b495b474e6a78bbb1ff) Merge remote-tracking branch 'origin/pr-585' into qa/windows-pr-acceptance-20261002 | UD-05-23 | UDX-040 | 仅主线；无已核实已发布包含 |
| [`f831ad936fc5be1a0b67ba3dc4431c9c30b71049`](https://github.com/wimi321/lizzieyzy-next/commit/f831ad936fc5be1a0b67ba3dc4431c9c30b71049) Merge pull request #585 from qiyi71w/acceptance/579-windows | UD-05-23 | UDX-040 | 仅主线；无已核实已发布包含 |
| [`d57e9233e51520f982fdc6bc503f0a03b157b39b`](https://github.com/wimi321/lizzieyzy-next/commit/d57e9233e51520f982fdc6bc503f0a03b157b39b) perf(preview): isolate branch simulation inputs | UD-05-24 | UDX-037 | 仅主线；无已核实已发布包含 |
| [`ac9b103eb08474b412dfd89966298f29fd2b865c`](https://github.com/wimi321/lizzieyzy-next/commit/ac9b103eb08474b412dfd89966298f29fd2b865c) fix(preview): honor pending variation selections | UD-05-24 | UDX-037 | 仅主线；无已核实已发布包含 |
| [`258cfdcb7f920320123d5a8af9217bc2081f81ea`](https://github.com/wimi321/lizzieyzy-next/commit/258cfdcb7f920320123d5a8af9217bc2081f81ea) feat(preview): render all hosts asynchronously | UD-05-24 | UDX-037 | 仅主线；无已核实已发布包含 |
| [`3a8f4e6dfefb566a1b9fe4e1a3da626c77229d3a`](https://github.com/wimi321/lizzieyzy-next/commit/3a8f4e6dfefb566a1b9fe4e1a3da626c77229d3a) fix(preview): hide suggestions during replacement | UD-05-24 | UDX-037 | 仅主线；无已核实已发布包含 |
| [`2f2809f50d9797e162e6ab56041570e2c9ddfbf2`](https://github.com/wimi321/lizzieyzy-next/commit/2f2809f50d9797e162e6ab56041570e2c9ddfbf2) fix(preview): retain visible variation on replacement | UD-05-24 | UDX-037 | 仅主线；无已核实已发布包含 |
| [`5ac06934b3888b17756b91152443c2b14f5d11de`](https://github.com/wimi321/lizzieyzy-next/commit/5ac06934b3888b17756b91152443c2b14f5d11de) fix(preview): preserve floating hover targets | UD-05-24 | UDX-037 | 仅主线；无已核实已发布包含 |
| [`8ec30daae57785f0570e279bc266a41fc1940560`](https://github.com/wimi321/lizzieyzy-next/commit/8ec30daae57785f0570e279bc266a41fc1940560) fix(preview): integrate first-move variation navigation | UD-05-21 | UDX-037 | 仅主线；无已核实已发布包含 |
| [`873b652dc1fce3812e1781d6fcb05dc65a1cc91d`](https://github.com/wimi321/lizzieyzy-next/commit/873b652dc1fce3812e1781d6fcb05dc65a1cc91d) fix(preview): preserve candidate input ownership | UD-05-24 | UDX-037 | 仅主线；无已核实已发布包含 |
| [`aa65f97e58b0397677717bea9dac33932bfe610e`](https://github.com/wimi321/lizzieyzy-next/commit/aa65f97e58b0397677717bea9dac33932bfe610e) fix(preview): serialize publication and replay | UD-05-24 | UDX-037 | 仅主线；无已核实已发布包含 |
| [`a361f59aae8989d9008589feb312a1bb9bc9185a`](https://github.com/wimi321/lizzieyzy-next/commit/a361f59aae8989d9008589feb312a1bb9bc9185a) Merge remote-tracking branch 'origin/pr-586' into qa/windows-pr-acceptance-20261002 | UD-05-24 | UDX-037 | 仅主线；无已核实已发布包含 |
| [`dfd40d3fc0b282c3e26a100952c15a13e87ad248`](https://github.com/wimi321/lizzieyzy-next/commit/dfd40d3fc0b282c3e26a100952c15a13e87ad248) Merge pull request #586 from qiyi71w/plan/issue-347-hover-preview | UD-05-24 | UDX-037 | 仅主线；无已核实已发布包含 |
| [`0178f55f51ba7b179674884979a286cefdb40d84`](https://github.com/wimi321/lizzieyzy-next/commit/0178f55f51ba7b179674884979a286cefdb40d84) Fix automatic quick curves for Zhizi remote-only profiles | UD-05-25 | UDX-011 | 仅主线；无已核实已发布包含 |
| [`8b37616e20401349961def86cb9a08003bf6cdf9`](https://github.com/wimi321/lizzieyzy-next/commit/8b37616e20401349961def86cb9a08003bf6cdf9) Fix quick analysis result loss and remote foreground cache handback | UD-05-25 | UDX-011 | 仅主线；无已核实已发布包含 |
| [`7c16ae92c06bdc92148073eeb1019341415f6518`](https://github.com/wimi321/lizzieyzy-next/commit/7c16ae92c06bdc92148073eeb1019341415f6518) Keep automatic curve ownership until foreground restoration completes | UD-05-25 | UDX-011 | 仅主线；无已核实已发布包含 |
| [`6a5b21b068d069b2b4682dc907f72ca4fc0cc177`](https://github.com/wimi321/lizzieyzy-next/commit/6a5b21b068d069b2b4682dc907f72ca4fc0cc177) Merge remote-tracking branch 'origin/pr-584' into qa/windows-pr-acceptance-20261002 | UD-05-25 | UDX-011 | 仅主线；无已核实已发布包含 |
| [`8395cc174bf9e2b15847b92c212b28c32252f49d`](https://github.com/wimi321/lizzieyzy-next/commit/8395cc174bf9e2b15847b92c212b28c32252f49d) refactor(analysis): own automatic quick analysis tasks | UD-05-25 | UDX-011 | 仅主线；无已核实已发布包含 |
| [`41142498a95ba36c77b531613b0f856f4229e498`](https://github.com/wimi321/lizzieyzy-next/commit/41142498a95ba36c77b531613b0f856f4229e498) Integrate PR #587 with remote quick-analysis restore regressions | UD-05-25 | UDX-011 | 仅主线；无已核实已发布包含 |
| [`702512f320e84ffbdb05d477ae9600462f041a0b`](https://github.com/wimi321/lizzieyzy-next/commit/702512f320e84ffbdb05d477ae9600462f041a0b) Fix automatic analysis startup remaining pending after unchecked failure | UD-05-25 | UDX-011 | 仅主线；无已核实已发布包含 |
| [`0469dd5b5287f13a74660f89fdcdbd0c435d8011`](https://github.com/wimi321/lizzieyzy-next/commit/0469dd5b5287f13a74660f89fdcdbd0c435d8011) Document Windows PR integration acceptance and remaining release gates | UD-05-04 | UDX-023 | 仅主线；无已核实已发布包含 |
| [`b768c7707b68bf1d966c5019cede4a7143999042`](https://github.com/wimi321/lizzieyzy-next/commit/b768c7707b68bf1d966c5019cede4a7143999042) Merge pull request #587 from qiyi71w/review/architecture-20261001 | UD-05-25 | UDX-011 | 仅主线；无已核实已发布包含 |
| [`35555e5e610c21abbe4b402d2e95be0941d39182`](https://github.com/wimi321/lizzieyzy-next/commit/35555e5e610c21abbe4b402d2e95be0941d39182) fix(fox): resolve numeric nicknames as nicknames | UD-05-26 | UDX-035 | 仅主线；无已核实已发布包含 |
| [`0557fe352100b006e885977e6c58445eb5d40700`](https://github.com/wimi321/lizzieyzy-next/commit/0557fe352100b006e885977e6c58445eb5d40700) fix(fox): simplify the Fox search row | UD-05-26 | UDX-035 | 仅主线；无已核实已发布包含 |
| [`2b6077c44fd637e79b6267a74c1b5bd6280d66ea`](https://github.com/wimi321/lizzieyzy-next/commit/2b6077c44fd637e79b6267a74c1b5bd6280d66ea) test: integrate PR 590 for Windows acceptance | UD-05-26 | UDX-035 | 仅主线；无已核实已发布包含 |
| [`07dc9f919886005c97a87a1fc76d8ff3d35c3b5e`](https://github.com/wimi321/lizzieyzy-next/commit/07dc9f919886005c97a87a1fc76d8ff3d35c3b5e) fix(analysis): explain unrestored foreground engine after quick-analysis handback failure | UD-05-25 | UDX-011 | 仅主线；无已核实已发布包含 |
| [`e6d6026f3bd05902ecf1f733b4c6a5cd19ac2000`](https://github.com/wimi321/lizzieyzy-next/commit/e6d6026f3bd05902ecf1f733b4c6a5cd19ac2000) fix(analysis): keep unrestored-engine guidance readable and scoped to handback | UD-05-25 | UDX-011 | 仅主线；无已核实已发布包含 |
| [`f360decb0440abfa82303ad9fe69cdd67a1070c5`](https://github.com/wimi321/lizzieyzy-next/commit/f360decb0440abfa82303ad9fe69cdd67a1070c5) fix(analysis): resume kifu sync after local restart | UD-05-25 | UDX-011 | 仅主线；无已核实已发布包含 |
| [`90afca6be97cd52245f815b1f1ffa89c137021e2`](https://github.com/wimi321/lizzieyzy-next/commit/90afca6be97cd52245f815b1f1ffa89c137021e2) fix(analysis): preserve recovery handoffs | UD-05-25 | UDX-011 | 仅主线；无已核实已发布包含 |
| [`5b6a1333da8ec06e0fb219b70fbfd9e67bd9bda6`](https://github.com/wimi321/lizzieyzy-next/commit/5b6a1333da8ec06e0fb219b70fbfd9e67bd9bda6) fix(analysis): retain queued restart handoff | UD-05-25 | UDX-011 | 仅主线；无已核实已发布包含 |
| [`ac0f7d4d38c48a7a8bf9d0c9aebf0e5ff78694e4`](https://github.com/wimi321/lizzieyzy-next/commit/ac0f7d4d38c48a7a8bf9d0c9aebf0e5ff78694e4) test: integrate PR 591 for Windows acceptance | UD-05-25 | UDX-011 | 仅主线；无已核实已发布包含 |
| [`2b45468f144cf17793c83e5c0b44c3b29b74cb92`](https://github.com/wimi321/lizzieyzy-next/commit/2b45468f144cf17793c83e5c0b44c3b29b74cb92) fix(analysis): route initial stop responses to lease | UD-05-25 | UDX-011 | 仅主线；无已核实已发布包含 |
| [`0e58160a3d270f726797f0d07bf67d9b85ead0c1`](https://github.com/wimi321/lizzieyzy-next/commit/0e58160a3d270f726797f0d07bf67d9b85ead0c1) fix(analysis): capture the reader when reserving a lease | UD-05-25 | UDX-011 | 仅主线；无已核实已发布包含 |
| [`5b7c7aaef7acd993053dfda0682a9bb8513fd9e8`](https://github.com/wimi321/lizzieyzy-next/commit/5b7c7aaef7acd993053dfda0682a9bb8513fd9e8) test: integrate PR 592 for Windows acceptance | UD-05-25 | UDX-011 | 仅主线；无已核实已发布包含 |
| [`41f9d9476492e5d104c971191e17d654b92f2809`](https://github.com/wimi321/lizzieyzy-next/commit/41f9d9476492e5d104c971191e17d654b92f2809) fix(katago): trust static-zlib HumanSL companion | UD-05-27 | UDX-002 | 仅主线；无已核实已发布包含 |
| [`a4520557ebae17026e19d4ffb1f0a4417a936987`](https://github.com/wimi321/lizzieyzy-next/commit/a4520557ebae17026e19d4ffb1f0a4417a936987) test: integrate PR 593 for Windows acceptance | UD-05-27 | UDX-002 | 仅主线；无已核实已发布包含 |
| [`1003e95a7670bedbc04bea6b9a946e34973517f9`](https://github.com/wimi321/lizzieyzy-next/commit/1003e95a7670bedbc04bea6b9a946e34973517f9) fix(fox): preserve keyboard access to search choices | UD-05-26 | UDX-035 | 仅主线；无已核实已发布包含 |
| [`94e4a2bdf97d0ba6e04e3138d7f8ac522fd46288`](https://github.com/wimi321/lizzieyzy-next/commit/94e4a2bdf97d0ba6e04e3138d7f8ac522fd46288) test(desktop): keep headless diagnostics off the desktop | UD-05-04 | UDX-023 | 仅主线；无已核实已发布包含 |
| [`ba42fe4673cbf4498222500c93e705a1b6c96307`](https://github.com/wimi321/lizzieyzy-next/commit/ba42fe4673cbf4498222500c93e705a1b6c96307) fix(katago): detect missing Transformer TensorRT parser runtime | UD-05-27 | UDX-002 | 仅主线；无已核实已发布包含 |
| [`d9812785023444b92afbf982c817b63f44d3a7bc`](https://github.com/wimi321/lizzieyzy-next/commit/d9812785023444b92afbf982c817b63f44d3a7bc) style(test): format TensorRT parser assertions | UD-05-04 | UDX-023 | 仅主线；无已核实已发布包含 |
| [`470ee88d5ba348ec7cb946a84740ea6168402d29`](https://github.com/wimi321/lizzieyzy-next/commit/470ee88d5ba348ec7cb946a84740ea6168402d29) fix(analysis): preserve automatic curves during remote recovery | UD-05-25 | UDX-011 | 仅主线；无已核实已发布包含 |
| [`757792abafda582b8b215299b3c2ed90a9e42fbe`](https://github.com/wimi321/lizzieyzy-next/commit/757792abafda582b8b215299b3c2ed90a9e42fbe) fix(fox): resolve numeric Fox nicknames as nicknames (#590) | UD-05-26 | UDX-035 | 仅主线；无已核实已发布包含 |
| [`00b38da747322665d378058eaef721f3242f3430`](https://github.com/wimi321/lizzieyzy-next/commit/00b38da747322665d378058eaef721f3242f3430) Merge remote-tracking branch 'origin/main' into qa/pr591-final-integration-20261006 | UD-05-25 | UDX-011 | 仅主线；无已核实已发布包含 |
| [`e607c706404c0e60db115eac2ac5b39c661137b5`](https://github.com/wimi321/lizzieyzy-next/commit/e607c706404c0e60db115eac2ac5b39c661137b5) fix(fox): preserve keyboard access to search choices | UD-05-26 | UDX-035 | 仅主线；无已核实已发布包含 |
| [`504cfbcf32d6d1d8bcb52d02b01b508fc82fe408`](https://github.com/wimi321/lizzieyzy-next/commit/504cfbcf32d6d1d8bcb52d02b01b508fc82fe408) fix(katago): detect missing Transformer TensorRT parser runtime | UD-05-27 | UDX-002 | 仅主线；无已核实已发布包含 |
| [`fc399c0ff755165b7fcae33de8d24276af5e7164`](https://github.com/wimi321/lizzieyzy-next/commit/fc399c0ff755165b7fcae33de8d24276af5e7164) test(desktop): keep headless diagnostics off the desktop | UD-05-04 | UDX-023 | 仅主线；无已核实已发布包含 |
| [`4dcbeeb679a59841fd3942779e020cbbe545a387`](https://github.com/wimi321/lizzieyzy-next/commit/4dcbeeb679a59841fd3942779e020cbbe545a387) fix(analysis): preserve automatic curves during remote recovery | UD-05-25 | UDX-011 | 仅主线；无已核实已发布包含 |
| [`219d47ec9c2307f6ce6bb6976b2475883619bab3`](https://github.com/wimi321/lizzieyzy-next/commit/219d47ec9c2307f6ce6bb6976b2475883619bab3) test: make remote recovery regression standalone | UD-05-04 | UDX-023 | 仅主线；无已核实已发布包含 |
| [`03202192ab183a5c0a0e71e803127fbfca8d6e17`](https://github.com/wimi321/lizzieyzy-next/commit/03202192ab183a5c0a0e71e803127fbfca8d6e17) Merge branches 'qa/pr591-final-integration-20261006', 'fix/fox-keyboard-acceptance-20261006', 'fix/trt-parser-acceptance-20261006', 'test/headless-desktop-diagnostics' and 'fix/remote-curve-reconnect-20261006' into qa/windows-pr-acceptance-20261006 | UD-05-04, UD-05-25, UD-05-26, UD-05-27 | UDX-002, UDX-011, UDX-023, UDX-035 | 仅主线；无已核实已发布包含 |
| [`8b0d9bda872f79a5beb0194c0036b9dc25f95d24`](https://github.com/wimi321/lizzieyzy-next/commit/8b0d9bda872f79a5beb0194c0036b9dc25f95d24) docs(qa): record native Windows PR integration acceptance | UD-05-04 | UDX-023 | 仅主线；无已核实已发布包含 |
| [`b82b6611b7dc4ce92d1a0f5e2d2063b585201617`](https://github.com/wimi321/lizzieyzy-next/commit/b82b6611b7dc4ce92d1a0f5e2d2063b585201617) Merge Windows-verified PR integration (#596) | UD-05-04, UD-05-25, UD-05-26, UD-05-27 | UDX-002, UDX-011, UDX-023, UDX-035 | 仅主线；无已核实已发布包含 |
| [`c0455839311b73548e479e6ef26f5722af31bdf2`](https://github.com/wimi321/lizzieyzy-next/commit/c0455839311b73548e479e6ef26f5722af31bdf2) feat: add ChatGPT commentary login and grounded teaching (#575) | UD-05-28, UD-05-29, UD-05-30 | UDX-005 | 仅主线；无已核实已发布包含 |
| [`af0e07a7386483f3bfc8a15780de72ffc2f0de4c`](https://github.com/wimi321/lizzieyzy-next/commit/af0e07a7386483f3bfc8a15780de72ffc2f0de4c) release: prepare next-2026-10-07.1 multilingual preview (#597) | UD-05-01 | UDX-023 | 仅主线；无已核实已发布包含 |

## 原候选证据别名

这些是原审计引用的窄范围继承输入，不是本轮新native运行。完整来源、候选、环境和未变边界按各组原记录和DEVELOPMENT；行为、DTO/协议、Run身份、资源版本、服务或平台改变时，相应继承失效。

 有限历史证据 tuple 与未变边界

本repair只有文档，不改相关行为/DTO/protocol/run identity/资源/服务/平台，所以仅原有限范围可继承。未来实现改变这些边界就由受影响票重新验证。下面引用历史已报告记录，不是本repair运行的pass。

| 证据引用 | 原candidate/可定位来源 | 已报告行为 / evidence class / 条件 | 保留边界与缺口 |
| --- | --- | --- | --- |
| E20 | `a06d600b0f16bd6bb61415a2907c5d839e53a0b1`；固定Matrix ANA-06:L156、restoration tickets01–05 completion | 01–04 owner/rendered/repository有限验证，05 Windows native/KataGo1.16.4 EigenCPU/zhizi_hzy_b28_muonfd2：streaming、Stop、navigation/input、持久intent、limit/Resume、finite handoff、Save/reopen/recovery；ticket04同candidate一/双线程queue与独立cancel | continuous时间/visit enable/value/Apply/persistence已支持，不能重复拆feature。默认600秒uninterrupted soak和macOS/Linux native未跑；正常targetcancel保留健康Run，fault timeout/cleanup与显式Run Stop不同。未证任意自动快析/动态threadcontrols |
| E16 | `5e593537f702af0c651a2c3bf0f0fd77758052e2`；Matrix ANA-16:L164与其ticket07 completion | Windows standalone/KataGo1.16.4 EigenCPU：显式current-game scope、quick/all/swing stages、time/total/leading OR预算、Pause/Continue、双lane、finite handoff、identity、Save/reopen/departure、engineexit与target-final | 不扩大成自动载入快析或当前position用户same-tree focus；独立APP-03/APP-04/ANA-15保持原ownership |
| E26 | `93b8410fde3bd0795662470b4d663c8b506cc4e6`；固定DEVELOPMENT§2.10，Matrix GAME-01:L213；基线App `matchOwnsWorkspace`守卫 | combined owner/manager/rendered结果与受控real-process回滚/排除/rebuild，实际Windows Human/KataGo/qualified GTP运行单列A01–A13 | 只继承现有Match占用工作区拒绝编辑的owner/fixture范围以及原native操作；原native不是未来规则详情/每个新rules入口的通过。新增入口/query/restore需新证据；没有扩大RULE-01 |
| E50 | `cc937c9d2bda0f0d7af513679328ebac774affb1` 与更新 `bc28fc79ea27251997a88f9e000151bf8c1154bf`；Matrix REVIEW-08:L185、ticket12完成 | 原Windows native：playback-start/silence、failed preference write、missing-resource error、muted restart；更新隔离app-data一次普通/pass/1与3capture、accepted/rejected/review方向静音 | 继承已有review音效、资源失败/开关保存范围，基线App async播放失败捕获保持document；没证明所有真实音频设备失败/对局countdown；后者保留UD050调查，不能重做已有reviewfeature |
| E34 | `f75cacf7c21b3b2252a16dfcb5fa833caae79e95` 初验；`a5a44642de6e235440b6280a2ebd07a8cea157b1` run6；`eb5736a12d888c30c6bd5a1d80c71b75133879ce` run7；01报告T01-READBOARD / 固定DEVELOPMENT:918–945 | Windows实际readboard `cdcc7b382668ea09cb6f1c61bda021f378080eda` v3.1.0 wire220430与Fox观战。初验sync/Stop/save可继承，已知root PL错误不可继承；run6修4次rebuild仍暴露Start；run7 Start+9次rebuild中7trusted marker决定PL | READ-02仍Partial，只读，Stop/generation/last-good范围保留；不得凭它宣称外部engine-turn/playback/confirmation已实现；无其他平台或安装态通过 |

原13个Covered行逐项处置：011保留现有lifecycle但compoundWindows/automatic/layout调查；013保留ANA-11原编码并新readability；021承认基线未分析节点点击已有，原native fullcandidate/拖动范围待具名证据重建；023无compound性能继承→调查；026用E26限定现有Matchguard；032 ordinarytarget与read-only分开调查；034用E34只读子范围+futureGAME-10；038仅SGF解析已有、动态rules确认调查；044 path/incarnation调查；045 E20保留targetcancel/Ready且RunStop分别调查；050 E50保留review已证和device残余；052 REL-09日志cleanup仍feature；053 staticargv不证明alias冲突，调查。**没有将全部这些行判为Missing，也没有把未知dynamic等价判Covered。**

历史静态thread argv、ANA-11 native图记录和早期ENG lifecycle若Matrix只给短SHA/未附精确新场景，就保留既有Accepted原范围，不拿当前HEAD代替原验收candidate；本report不承诺已新查到原livepass。具名UD021/011/053问题只定位必要原记录和实际路径，不以缺tuple删除已存在实现。


 可继承历史证据与不变边界

历史结果不在本票重跑；以下全 SHA 只标原结果归属，不是给本次候选新通过。

| 引用 | 原候选/原出处 | 已观察的条件与行为 | 继承边界及残余 |
| --- | --- | --- | --- |
| H-ANALYSIS | `a06d600b0f16bd6bb61415a2907c5d839e53a0b1`，固定 Plan:594–608、Matrix ANA-06；`5e593537f702af0c651a2c3bf0f0fd77758052e2`，Plan:610–622、Matrix ANA-16 | 前者 Windows 原生、KataGo 1.16.4 EigenCPU、zhizi_hzy_b28_muonfd2，流式/Stop/有限请求交还/一二线程与独立取消/保存重开/目标终态清理；后者 Windows standalone、KataGo 1.16.4 EigenCPU，显式 quick/all-position/swing-selected、所有预算、Pause/Continue、有限交还、lane-local Cancel、身份失效、Save/reopen及departure失败保持均在原记录 | 固定基线包含该功能，文档修订不改变任务/Run/job/DTO/资源/平台边界。仅继承这些原场景；Java whole-game 窗口立即 close/reopen 的时序不自动等价为每种 Next 面板行为。自动载入快析归 T02-AUTOLOAD-QUICK，默认600秒soak及macOS/Linux未跑不变。 |
| H-SWITCH | `90508df1d74e94d11036a8955707f8399927655e`，Plan:475–512及 Matrix ENG-02/03/04 的 Ticket09 分类 | Windows KataGoAnalysis真实 v1.16.4 Eigen CPU；real smoke 71.37s验 Ready/Stop/Restart/A→B；26.48s失败类验 missing-model Start→No-engine、Switch→原Ready A；原native `R3 Smoke B` 切missing model保持B并可见asset错误 | 只继承成功切换和实际asset失败；spawn/protocol/timeout、late/stale是原fixture类，不伪装native。没有 TensorRT特定控制修复或Java reader细节的全量证据。相关profile/Run/协议边界若改变须由后继重验。 |
| H-OFFLINE | `66c906f3117cc1b8274673a0382456b91f02e39f`，Plan:452–473、Matrix UI-04，R2 Ticket07 run A/PID51344 | Windows原生，无引擎；golden branching SGF，C4/B3、comment、删A2、Save As/reopen，保留B3/comment且移除second continuation；`未加载引擎`而SGF controls可用 | 仅无引擎打谱/导航/保存既有路径。窗口、资源与current-game数据不变时可引用；并非每个ownership显示设置、崩溃后所有入口、transform或全DPI验证。 |
| H-SAVE-DEPARTURE | `4fd710e1c24a991665c2ed47f58bbb178b8c1c82`，Matrix SGF-07；R5集成 `48db2b2833f9deb45bd7dcd47181f5da77348a0c`，Plan:592 | Windows原生New Cancel保持运行整局分析；已确认departure后的Save As Cancel可见“Analysis is stopped; restart it explicitly”；R5 N1–N7包含GIB/activation/drop/recovery/departure/failure/exit | 保留原SGF-06/07的保存/取消/dirty/替换合同，不声称普通Save while analyzing另行native捕获，不把departure取消语义改成Java自动恢复。48db不是诊断/调优对话框全DPI或atomic-save证明。 |
| H-KOMI-EDITOR | `01429efcaf1fa732c1c0f2c2dcf50d367d478ea8`，DEVELOPMENT R6 ticket07、Matrix SGF-13 | Windows isolated app-data；finite-komi拒绝/Cancel不变，KM[6.50]无改动仍原字节；Apply/Undo/Redo、Save As/reopen PB/PW/KM；真实KataGo name-only continuity与komi失效/restart | metadata editor不是PK双参与者runtime-komi事务，也不是offline clear保持原贴目的自动证明。 |
| H-STATIC-THREAD | `b23f3eb16c16ad63bf4570d7ecc9bf22c274ea05`，DEVELOPMENT:199–207 | Windows private desktop/Tauri，KataGo1.12.3 EigenCPU AVX2/FMA；M02保存argv不改变原Run，显式Restart新Run采用 `-override-config numSearchThreads=2` / visits12，owned PID71276命令行确认 | 静态launch argv与明确Restart已观测；未验动态线程修改、CFG/BENCHMARK来源/有效值/临时覆盖或measured-report工作流。有关运行身份/资源版本变化使适用证据须重新评估。 |
| H-READBOARD | T01-READBOARD/01历史表：Next `f75cacf7c21b3b2252a16dfcb5fa833caae79e95`、修复 `a5a44642de6e235440b6280a2ebd07a8cea157b1` / `eb5736a12d888c30c6bd5a1d80c71b75133879ce`；readboard `cdcc7b382668ea09cb6f1c61bda021f378080eda` v3.1.0 wire220430 | Windows真实Fox spectator同步、Stop/保存；run7 trusted marker的PL修复及后续首手颜色 | READ-02仍Partial；初验已知PL错误不得继承为正确，外部写回GAME-10及其他平台未验。不能以engine lifecycle代替producer snapshot确认。 |

没有历史证据闭合的细分断言在 §4/§5 明确归调查。UI/CSS/Rust所有权只约束实现，不能证明未实现窗口或native可达性免疫。若以后继承条件变化，由实际受影响任务重验，不批量重开历史Accepted，也不禁止06建立有证据的后继。



## 追加与收尾责任

后续上游审计从本轮终点追加明确左开右闭区间，不移动v1、不重写本轮source归属或发布观察。新严重数据/结果/协议问题由相应当前支持路径owner立即作有界可达性调查，证实后在实际消费者阶段提前修复；Java缺陷不自动等于Next缺陷。R17（历史F）的最终上游复核任务负责其余新增区间和跨功能遗漏；R18承接未执行旧R11的全部发行/安装义务，必要运行资源与诊断不整体后置。

## 末段证据别名


- **E05-READBOARD**：固定 [DEVELOPMENT §4](https://github.com/qiyi71w/lizzieyzy-next-tauri/blob/55795fab49b80a681a9fa544950e18f5f2da4cc6/docs/DEVELOPMENT.md#L912-L946)；Windows x64 Next 原生 + readboard `cdcc7b382668ea09cb6f1c61bda021f378080eda`（v3.1.0 / wire220430），真实 Fox spectator rooms。初始 Next `f75cacf7c21b3b2252a16dfcb5fa833caae79e95` 有换房重建 PL 错误，不能继承为该修复通过。Run6 `a5a44642de6e235440b6280a2ebd07a8cea157b1` 的四次 White→Black 换房正确，但后来 Start 仍沿用旧 PL；Run7 `eb5736a12d888c30c6bd5a1d80c71b75133879ce` 的 SGF-07 Start 加九次重建中，七次可信 `foxCornerFlip` marker 重建与 marker 一致，四次改变旧根 PL；三次随后真实落子也匹配。只有 deviation 或无可信 marker 的帧保留旧 baseline，等下一可信帧修正。该证据不覆盖落后标题排序新调查、SGF→引擎恢复、OCR 或 GAME-10 写入。
- **E05-INHERITANCE**：以上和 UD-05-09 均保留原候选、平台、引擎/资源、服务版本及实际观察范围；文档后继不改记为本报告 HEAD 的运行通过。行为、DTO/协议、Run/owner identity、资源版本、服务或平台任一相关边界改变，须重验受影响面；无法建立条件时记缺口。本轮仅审计既有记录，没有原生/真实服务新验收。

