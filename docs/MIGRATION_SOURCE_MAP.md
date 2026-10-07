# Migration Source Map — 冻结来源与当前公开责任

本文保留 Issue18 冻结来源分配中的 **203 个 source-intent 合同**：199 个审计意图和 4 个集成/决定意图。这是来源覆盖索引，不是 203 个新功能，也不提升产品状态。实际当前路由以 [MIGRATION_ROUTES.json](MIGRATION_ROUTES.json) 为准：155 个唯一实施责任意图承接 198 个来源，另有 5 个证据继承/排除来源；63 个未完成条目均保留具体责任与 scoped/named gates。

## 基线与阅读方式

- 仓库：`qiyi71w/lizzieyzy-next-tauri`；固定调查基线 `55795fab49b80a681a9fa544950e18f5f2da4cc6`。
- Java Migration Baseline v1 `7b4027531c2b26062d0bfc27a040cc550cfbea4d`；冻结增量终点 `af0e07a7386483f3bfc8a15780de72ffc2f0de4c`。
- 来源 T01/T02/T03/T04/T05 分别是范围、入口/目标、首段增量、后段增量及追加增量审计；T06 是 spec 级集成/决定。原分组 A/B/C/D/E/F/历史 R11 只标历史来源：当前常用复盘为 R11，历史发行阶段由 R18 承担。
- 各意图的原义、非等价目标、来源/默认/失败/证据门保持可读；本文历史来源中的“待决定”“未分配”等只说明冻结当时事实，不覆盖当前已批准决定或公开路线。R11 的已批准 nonmove insertion、小数秒持久化及 shell/raw/slots/KM/sub-board dispositions 以 [R11_PLAN](R11_PLAN.md) 对应精确 behavior 为当前合同；后续阶段具体批准门见 R12–R18 与 routes。
- 原 Parity IDs、Accepted 窄范围、原完整候选和五项 ADR 保留；source alias 不建立第二个 owner。历史来源完整行为/非目标/实际阻塞/证据门继续连接仓库内 [UPSTREAM_DELTA](UPSTREAM_DELTA.md)、[JAVA_CAPABILITY_INVENTORY](JAVA_CAPABILITY_INVENTORY.md) 和 [后续合同附录](MIGRATION_CONTRACTS.md)。

## 分配与依赖合同

首个真实 UI/设置消费者在 R11 交付本地化 foundation，后续新 UI 消费这一已验证结果，每个设置 owner 同步 Java 字段语义映射。一个最终能力可消费多个同目标来源，但不能将独立功能人为串行。实际依赖按 state/interface/lifecycle 边界，阶段次序不是整阶段硬前置。

investigation/decision 先交付可判定结论和所需批准；feature 只等其真实残余。缺协议、凭据或产品决定的受影响行为保持具名阻塞；继承/内部排除只保留证据，不造 feature。benchmark runner 消费既有 profile/资源，saved policy 消费 runner，不能反向循环。Match rules storage 决定只阻塞保存/共享，普通确认、准入、恢复和只读详情独立。本地 SGF 恢复及显式 Restart 调查不等远程凭据；同树 focus 本地准入/数据/显示不等 readboard 扩展。必要诊断由首个 Run/资源消费者交付，每个新 consumer 补脱敏/取消/错误，不统一等安装态支持包。

## 全集来源与当前路由

| 原来源意图 | 唯一责任意图 | 来源任务性质 | 审计来源 | 当前公开合同 |
| --- | --- | --- | --- | --- |
| [T01-RESOURCE](#t01-resource) | T01-RESOURCE | feature | 01 | [R12 / resources](R12_PLAN.md#resources) |
| [T01-NETWORK](#t01-network) | T01-NETWORK | feature | 01 | [R14 / compute-network](R14_PLAN.md#compute-network) |
| [T01-SSH](#t01-ssh) | T01-SSH | feature | 01 | [R14 / ssh](R14_PLAN.md#ssh) |
| [T01-REMOTE](#t01-remote) | T01-REMOTE | feature | 01 | [R14 / compute-network](R14_PLAN.md#compute-network) |
| [T01-CONTRIBUTION](#t01-contribution) | T01-CONTRIBUTION | feature | 01 | [R16 / contribution](R16_PLAN.md#contribution) |
| [T01-WATCH](#t01-watch) | T01-WATCH | feature | 01 | [R16 / watch](R16_PLAN.md#watch) |
| [T01-EXTERNAL](#t01-external) | T01-EXTERNAL | feature | 01 | [R14 / external-match](R14_PLAN.md#external-match) |
| [T01-READBOARD](#t01-readboard) | T01-READBOARD | feature | 01 | [R14 / readboard](R14_PLAN.md#readboard) |
| [T01-PROVIDERS](#t01-providers) | T01-PROVIDERS | feature | 01 | [R14 / providers](R14_PLAN.md#providers) |
| [T01-DIAGNOSTICS](#t01-diagnostics) | T01-DIAGNOSTICS | feature | 01 | [R12 / diagnostics](R12_PLAN.md#diagnostics) |
| [T01-IDENTITY](#t01-identity) | T01-IDENTITY | feature | 01 | [R11 / behavior-a02](R11_PLAN.md#behavior-a02) |
| [T01-ACTIVATION](#t01-activation) | T01-ACTIVATION | feature | 01 | [R18 / activation](R18_PLAN.md#activation) |
| [T01-RELEASE](#t01-release) | T01-RELEASE | feature | 01 | [R18 / discovery-components](R18_PLAN.md#discovery-components) |
| [T02-H01](#t02-h01) | T02-H01 | decision | 02 | [R11 / behavior-a03](R11_PLAN.md#behavior-a03) |
| [T02-H02](#t02-h02) | T02-H02 | decision | 02 | [R11 / behavior-a04](R11_PLAN.md#behavior-a04) |
| [T02-H03](#t02-h03) | T02-H03 | decision | 02 | [R17 / initialization](R17_PLAN.md#initialization) |
| [T02-H04](#t02-h04) | T02-H04 | decision | 02 | [R17 / initialization](R17_PLAN.md#initialization) |
| [T02-H05](#t02-h05) | T02-H05 | decision | 02 | [R17 / appearance](R17_PLAN.md#appearance) |
| [T02-H06](#t02-h06) | T02-H06 | decision | 02 | [R17 / appearance](R17_PLAN.md#appearance) |
| [T02-H07](#t02-h07) | T02-H07 | decision | 02 | [R17 / appearance](R17_PLAN.md#appearance) |
| [T02-H08](#t02-h08) | T02-H08 | decision | 02 | [R17 / appearance](R17_PLAN.md#appearance) |
| [T02-H09](#t02-h09) | T02-H09 | decision | 02 | [R17 / appearance](R17_PLAN.md#appearance) |
| [T02-H10](#t02-h10) | T02-H10 | decision | 02 | [R16 / guidance](R16_PLAN.md#guidance) |
| [T02-H11](#t02-h11) | T02-H11 | decision | 02 | [R16 / guidance](R16_PLAN.md#guidance) |
| [T02-H12](#t02-h12) | T02-H12 | decision | 02 | [R11 / behavior-a05](R11_PLAN.md#behavior-a05) |
| [T02-H13](#t02-h13) | T02-H13 | decision | 02 | [R11 / behavior-a06](R11_PLAN.md#behavior-a06) |
| [T02-H14](#t02-h14) | T02-H14 | decision | 02 | [R11 / behavior-a07](R11_PLAN.md#behavior-a07) |
| [T02-H15](#t02-h15) | T02-H15 | decision | 02 | [R11 / behavior-a08](R11_PLAN.md#behavior-a08) |
| [T02-H16](#t02-h16) | T02-H16 | decision | 02 | [R13 / display](R13_PLAN.md#display) |
| [T02-H17](#t02-h17) | T02-H17 | decision | 02 | [R12 / startup](R12_PLAN.md#startup) |
| [T02-H18](#t02-h18) | T02-H18 | decision | 02 | [R12 / startup](R12_PLAN.md#startup) |
| [T02-H19](#t02-h19) | T02-H19 | decision | 02 | [R13 / retained-analysis](R13_PLAN.md#retained-analysis) |
| [T02-H20](#t02-h20) | T02-H20 | decision | 02 | [R13 / retained-analysis](R13_PLAN.md#retained-analysis) |
| [T02-H21](#t02-h21) | T02-H21 | decision | 02 | [R13 / retained-analysis](R13_PLAN.md#retained-analysis) |
| [T02-H22](#t02-h22) | T02-H22 | decision | 02 | [R17 / appearance](R17_PLAN.md#appearance) |
| [T02-H23](#t02-h23) | T02-H23 | decision | 02 | [R16 / generated](R16_PLAN.md#generated) |
| [T02-H24](#t02-h24) | T02-H24 | decision | 02 | [R13 / tracking](R13_PLAN.md#tracking) |
| [T02-H25](#t02-h25) | T02-H25 | decision | 02 | [R15 / human-clock](R15_PLAN.md#human-clock) |
| [T02-H26](#t02-h26) | T02-H26 | decision | 02 | [R14 / yike](R14_PLAN.md#yike) |
| [T02-H27](#t02-h27) | T02-H27 | decision | 02 | [R14 / yike](R14_PLAN.md#yike) |
| [T02-H28](#t02-h28) | T02-H28 | decision | 02 | [R18 / channel](R18_PLAN.md#channel) |
| [T02-UI-02](#t02-ui-02) | T02-UI-02 | feature | 02 | [R17 / accessibility](R17_PLAN.md#accessibility) |
| [T02-REL-01](#t02-rel-01) | T02-REL-01 | feature | 02 | [R18 / preflight-trust](R18_PLAN.md#preflight-trust) |
| [T02-REL-02](#t02-rel-02) | T02-REL-02 | feature | 02 | [R18 / preflight-trust](R18_PLAN.md#preflight-trust) |
| [T02-REL-03](#t02-rel-03) | T02-REL-03 | feature | 02 | [R18 / discovery-components](R18_PLAN.md#discovery-components) |
| [T02-REL-04](#t02-rel-04) | T02-REL-04 | feature | 02 | [R18 / installed](R18_PLAN.md#installed) |
| [T02-REL-06](#t02-rel-06) | T02-REL-06 | feature | 02 | [R18 / windows-update](R18_PLAN.md#windows-update) |
| [T02-REL-07](#t02-rel-07) | T02-REL-07 | feature | 02 | [R18 / handoff](R18_PLAN.md#handoff) |
| [T02-REL-08](#t02-rel-08) | T02-REL-08 | feature | 02 | [R18 / windows-update](R18_PLAN.md#windows-update) |
| [T02-I18N-01](#t02-i18n-01) | T02-I18N-01 | feature | 02 | [R17 / localization](R17_PLAN.md#localization) |
| [T02-GUIDE-01](#t02-guide-01) | T02-GUIDE-01 | feature | 02 | [R16 / guidance](R16_PLAN.md#guidance) |
| [T02-SGF-15](#t02-sgf-15) | T02-SGF-15 | feature | 02 | [R11 / behavior-a09](R11_PLAN.md#behavior-a09) |
| [T02-SGF-16](#t02-sgf-16) | T02-SGF-16 | feature | 02 | [R11 / behavior-a10](R11_PLAN.md#behavior-a10) |
| [T02-REVIEW-04](#t02-review-04) | T02-REVIEW-04 | feature | 02 | [R11 / behavior-a11](R11_PLAN.md#behavior-a11) |
| [T02-REVIEW-05](#t02-review-05) | T02-REVIEW-05 | feature | 02 | [R11 / behavior-a12](R11_PLAN.md#behavior-a12) |
| [T02-REVIEW-06](#t02-review-06) | T02-REVIEW-06 | feature | 02 | [R11 / behavior-a13](R11_PLAN.md#behavior-a13) |
| [T02-EXPORT-01](#t02-export-01) | T02-EXPORT-01 | feature | 02 | [R11 / behavior-a14](R11_PLAN.md#behavior-a14) |
| [T02-EXPORT-02](#t02-export-02) | T02-EXPORT-02 | feature | 02 | [R11 / behavior-a15](R11_PLAN.md#behavior-a15) |
| [T02-EXPORT-03](#t02-export-03) | T02-EXPORT-03 | feature | 02 | [R11 / behavior-a16](R11_PLAN.md#behavior-a16) |
| [T02-ENG-08](#t02-eng-08) | T02-ENG-08 | feature | 02 | [R11 / behavior-a17](R11_PLAN.md#behavior-a17) |
| [T02-ANA-07](#t02-ana-07) | T02-ANA-07 | feature | 02 | [R13 / batch](R13_PLAN.md#batch) |
| [T02-ANA-09](#t02-ana-09) | T02-ANA-09 | feature | 02 | [R13 / adapters](R13_PLAN.md#adapters) |
| [T02-GAME-06](#t02-game-06) | T02-GAME-06 | feature | 02 | [R15 / pk](R15_PLAN.md#pk) |
| [T02-GAME-07](#t02-game-07) | T02-GAME-07 | feature | 02 | [R15 / human-clock](R15_PLAN.md#human-clock) |
| [T02-GAME-08](#t02-game-08) | T02-GAME-08 | feature | 02 | [R15 / humansl](R15_PLAN.md#humansl) |
| [T02-PROV-05](#t02-prov-05) | T02-PROV-05 | feature | 02 | [R14 / tencent](R14_PLAN.md#tencent) |
| [T02-PROV-06](#t02-prov-06) | T02-PROV-06 | feature | 02 | [R14 / yike](R14_PLAN.md#yike) |
| [T02-PROV-07](#t02-prov-07) | T02-PROV-07 | feature | 02 | [R14 / yike](R14_PLAN.md#yike) |
| [T02-PUB-01](#t02-pub-01) | T02-PUB-01 | feature | 02 | [R16 / lan](R16_PLAN.md#lan) |
| [T02-AUTOLOAD-QUICK](#t02-autoload-quick) | T02-AUTOLOAD-QUICK | feature | 02 | [R13 / autoquick](R13_PLAN.md#autoquick) |
| [T02-TRACKING](#t02-tracking) | T02-TRACKING | feature | 02 | [R13 / tracking](R13_PLAN.md#tracking) |
| [T02-GRANULAR-DISPLAY](#t02-granular-display) | T02-GRANULAR-DISPLAY | feature | 02 | [R13 / display](R13_PLAN.md#display) |
| [T02-CUSTOM-GRADE](#t02-custom-grade) | T02-CUSTOM-GRADE | decision | 02 | [R13 / display](R13_PLAN.md#display) |
| [T02-N-ENTRY](#t02-n-entry) | T02-N-ENTRY | decision | 02 | [R11 / behavior-a18](R11_PLAN.md#behavior-a18) |
| [T02-JAVA-IMPORT](#t02-java-import) | T02-JAVA-IMPORT | feature | 02 | [R17 / import](R17_PLAN.md#import) |
| [T02-I18N-FOUNDATION](#t02-i18n-foundation) | T02-I18N-FOUNDATION | feature | 02 | [R11 / behavior-a01](R11_PLAN.md#behavior-a01) |
| [T02-CLOCK-DECISION](#t02-clock-decision) | T02-CLOCK-DECISION | decision | 02 | [R15 / human-clock](R15_PLAN.md#human-clock) |
| [T02-TENCENT-PROTOCOL](#t02-tencent-protocol) | T02-TENCENT-PROTOCOL | investigation/decision | 02 | [R14 / tencent](R14_PLAN.md#tencent) |
| [T02-YIKE-AUTH](#t02-yike-auth) | T02-YIKE-AUTH | investigation/decision | 02 | [R14 / yike](R14_PLAN.md#yike) |
| [T02-PERSONAL-EVIDENCE](#t02-personal-evidence) | T02-PERSONAL-EVIDENCE | investigation/decision | 02 | [R14 / yike](R14_PLAN.md#yike) |
| [T02-ADAPTER-EVIDENCE](#t02-adapter-evidence) | T02-ADAPTER-EVIDENCE | investigation/decision | 02 | [R13 / adapters](R13_PLAN.md#adapters) |
| [T02-GUIDANCE-PRODUCER](#t02-guidance-producer) | T02-GUIDANCE-PRODUCER | investigation/decision | 02 | [R16 / guidance](R16_PLAN.md#guidance) |
| [T02-ENTRY-CLOSEOUT](#t02-entry-closeout) | T02-ENTRY-CLOSEOUT | feature | 02 | [R17 / entries](R17_PLAN.md#entries) |
| [T02-CONTINUATION](#t02-continuation) | T02-CONTINUATION | decision | 02 | [R13 / continuation](R13_PLAN.md#continuation) |
| [T02-CONTRIBUTION-CUTS](#t02-contribution-cuts) | T02-CONTRIBUTION-CUTS | decision | 02 | [R16 / contribution](R16_PLAN.md#contribution) |
| [T03-REL09-DIAG-BUNDLE-EXPORT](#t03-rel09-diag-bundle-export) | T01-DIAGNOSTICS | feature | UD-03-001 | [R12 / diagnostics](R12_PLAN.md#diagnostics) |
| [T03-REL09-THREAD-SNAPSHOT](#t03-rel09-thread-snapshot) | T01-DIAGNOSTICS | feature | UD-03-002 | [R12 / diagnostics](R12_PLAN.md#diagnostics) |
| [T03-ENG01-BOOTSTRAP-DIAGNOSTICS](#t03-eng01-bootstrap-diagnostics) | T01-DIAGNOSTICS | feature | UD-03-003 | [R12 / diagnostics](R12_PLAN.md#diagnostics) |
| [T03-REL09-RUNTIME-SNAPSHOT](#t03-rel09-runtime-snapshot) | T01-DIAGNOSTICS | feature | UD-03-004 | [R12 / diagnostics](R12_PLAN.md#diagnostics) |
| [T03-ENG01-GTP-PROBE-STDERR](#t03-eng01-gtp-probe-stderr) | T01-DIAGNOSTICS | feature | UD-03-005 | [R12 / diagnostics](R12_PLAN.md#diagnostics) |
| [T03-RESOURCE-MAINTENANCE-LIFECYCLE](#t03-resource-maintenance-lifecycle) | T01-RESOURCE | feature | UD-03-006 | [R12 / resources](R12_PLAN.md#resources) |
| [T03-GAME08-HUMANSL-SAFETY](#t03-game08-humansl-safety) | T02-GAME-08 | feature | UD-03-007 | [R15 / humansl](R15_PLAN.md#humansl) |
| [T03-RESOURCE-TRT-REPAIR-INVESTIGATION](#t03-resource-trt-repair-investigation) | T03-RESOURCE-TRT-REPAIR-INVESTIGATION | investigation | UD-03-008 | [R12 / resources](R12_PLAN.md#resources) |
| [T03-GAME08-RESTORE-FOREGROUND](#t03-game08-restore-foreground) | T02-GAME-08 | feature | UD-03-009 | [R15 / humansl](R15_PLAN.md#humansl) |
| [T03-RESOURCE-NVIDIA-HARDWARE-GATE](#t03-resource-nvidia-hardware-gate) | T03-RESOURCE-NVIDIA-HARDWARE-GATE | investigation | UD-03-010 | [R12 / resources](R12_PLAN.md#resources) |
| [T03-LIFECYCLE-WINDOWS-APPLICABILITY](#t03-lifecycle-windows-applicability) | T03-LIFECYCLE-WINDOWS-APPLICABILITY | investigation | UD-03-011 | [R12 / lifecycle](R12_PLAN.md#lifecycle) |
| [T03-GAME-AI-COMMENTARY-WORKSPACE](#t03-game-ai-commentary-workspace) | T05-AI-GROUNDED-TEACHING | feature | UD-03-012 | [R16 / ai](R16_PLAN.md#ai) |
| [T03-ANA11-SCORE-LEAD-DISPLAY](#t03-ana11-score-lead-display) | T03-ANA11-SCORE-LEAD-DISPLAY | feature | UD-03-013 | [R11 / behavior-a19](R11_PLAN.md#behavior-a19) |
| [T03-SGF07-RAPID-SWITCH-TRANSACTION](#t03-sgf07-rapid-switch-transaction) | T03-SGF07-RAPID-SWITCH-TRANSACTION | investigation | UD-03-014 | [R11 / behavior-a20](R11_PLAN.md#behavior-a20) |
| [T03-PERFORMANCE-SPEED-METRICS](#t03-performance-speed-metrics) | T03-PERFORMANCE-SPEED-METRICS | feature | UD-03-015 | [R13 / performance](R13_PLAN.md#performance) |
| [T03-RCOMP-SKIP-LOCAL-BENCHMARK](#t03-rcomp-skip-local-benchmark) | T01-REMOTE | feature | UD-03-016 | [R14 / compute-network](R14_PLAN.md#compute-network) |
| [T03-PERFORMANCE-A11Y-BENCHMARK](#t03-performance-a11y-benchmark) | T03-PERFORMANCE-A11Y-BENCHMARK | feature | UD-03-017 | [R13 / performance](R13_PLAN.md#performance) |
| [T03-LIFECYCLE-CONCURRENCY-INVESTIGATION](#t03-lifecycle-concurrency-investigation) | T03-LIFECYCLE-CONCURRENCY-INVESTIGATION | investigation | UD-03-018 | [R12 / lifecycle](R12_PLAN.md#lifecycle) |
| [T03-REL09-LOGGING-GENERATION](#t03-rel09-logging-generation) | T01-DIAGNOSTICS | feature | UD-03-019 | [R12 / diagnostics](R12_PLAN.md#diagnostics) |
| [T03-ANA06-LIVE-LIMITS-INHERITANCE](#t03-ana06-live-limits-inheritance) | T03-ANA06-LIVE-LIMITS-INHERITANCE | inheritance-or-exclusion | UD-03-020 | 证据继承/排除；无新功能责任（见 [routes evidence_only](MIGRATION_ROUTES.json)） |
| [T03-ANA11-UNANALYZED-SCRUBBING](#t03-ana11-unanalyzed-scrubbing) | T03-ANA11-UNANALYZED-SCRUBBING | investigation | UD-03-021 | [R11 / behavior-a21](R11_PLAN.md#behavior-a21) |
| [T03-AUTOLOAD-PAUSE-BOUNDARIES](#t03-autoload-pause-boundaries) | T02-AUTOLOAD-QUICK | feature | UD-03-022 | [R13 / autoquick](R13_PLAN.md#autoquick) |
| [T03-PERF-BACKGROUND-OFFLOAD](#t03-perf-background-offload) | T03-PERF-BACKGROUND-OFFLOAD | investigation | UD-03-023 | [R11 / behavior-a22](R11_PLAN.md#behavior-a22) |
| [T03-ANA16-WHOLE-GAME-LIMITS](#t03-ana16-whole-game-limits) | T03-ANA16-WHOLE-GAME-LIMITS | investigation | UD-03-024 | [R13 / autoquick](R13_PLAN.md#autoquick) |
| [T03-REL09-ANALYSIS-CACHE-TRACE](#t03-rel09-analysis-cache-trace) | T01-DIAGNOSTICS | feature | UD-03-025 | [R12 / diagnostics](R12_PLAN.md#diagnostics) |
| [T03-GAME01-RULES-DIALOG-GUARD](#t03-game01-rules-dialog-guard) | T03-GAME01-RULES-DIALOG-GUARD | inheritance-or-exclusion | UD-03-026 | 证据继承/排除；无新功能责任（见 [routes evidence_only](MIGRATION_ROUTES.json)） |
| [T03-MATCH-GENMOVE-RETIREMENT-INVESTIGATION](#t03-match-genmove-retirement-investigation) | T03-MATCH-GENMOVE-RETIREMENT-INVESTIGATION | investigation | UD-03-027 | [R12 / lifecycle](R12_PLAN.md#lifecycle) |
| [T03-PERFORMANCE-CUSTOM-BENCHMARK-RUNNER](#t03-performance-custom-benchmark-runner) | T03-PERFORMANCE-CUSTOM-BENCHMARK-RUNNER | feature | UD-03-028 | [R13 / performance](R13_PLAN.md#performance) |
| [T03-MATCH-RULES-LIFECYCLE](#t03-match-rules-lifecycle) | T03-MATCH-RULES-LIFECYCLE | feature | UD-03-029 | [R12 / lifecycle](R12_PLAN.md#lifecycle) |
| [T03-MATCH-RULES-STORAGE-DECISION](#t03-match-rules-storage-decision) | T03-MATCH-RULES-STORAGE-DECISION | decision | UD-03-029 | [R15 / rule-storage](R15_PLAN.md#rule-storage) |
| [T03-REL09-SANITIZER-BOUNDARIES](#t03-rel09-sanitizer-boundaries) | T01-DIAGNOSTICS | feature | UD-03-030 | [R12 / diagnostics](R12_PLAN.md#diagnostics) |
| [T03-REL09-EXPORT-BOUNDED-IO](#t03-rel09-export-bounded-io) | T01-DIAGNOSTICS | feature | UD-03-031 | [R12 / diagnostics](R12_PLAN.md#diagnostics) |
| [T03-ANALYSIS-CONFIRMED-POSITION-INVESTIGATION](#t03-analysis-confirmed-position-investigation) | T03-ANALYSIS-CONFIRMED-POSITION-INVESTIGATION | investigation | UD-03-032 | [R13 / focus](R13_PLAN.md#focus) |
| [T03-RCOMP01-SETUP-GUIDANCE](#t03-rcomp01-setup-guidance) | T01-REMOTE | feature | UD-03-033 | [R14 / compute-network](R14_PLAN.md#compute-network) |
| [T03-READ02-STOP-AND-GAME10-TURN-RETIREMENT](#t03-read02-stop-and-game10-turn-retirement) | T03-READ02-STOP-AND-GAME10-TURN-RETIREMENT | feature | UD-03-034 | [R14 / external-match](R14_PLAN.md#external-match) |
| [T03-REL05-HUMANSL-R2-DOWNLOAD](#t03-rel05-humansl-r2-download) | T02-GAME-08 | feature | UD-03-035 | [R15 / humansl](R15_PLAN.md#humansl) |
| [T03-THREAD-SAVED-ENTRY-POLICY](#t03-thread-saved-entry-policy) | T03-THREAD-SAVED-ENTRY-POLICY | feature | UD-03-036 | [R13 / performance](R13_PLAN.md#performance) |
| [T03-REL05-B11-MODEL-UPDATE](#t03-rel05-b11-model-update) | T01-RESOURCE | feature | UD-03-037 | [R12 / resources](R12_PLAN.md#resources) |
| [T03-ANALYSIS-SGF-RULES-SYNC-INVESTIGATION](#t03-analysis-sgf-rules-sync-investigation) | T03-ANALYSIS-SGF-RULES-SYNC-INVESTIGATION | investigation | UD-03-038 | [R13 / focus](R13_PLAN.md#focus) |
| [T03-RESOURCE-ACCEL-LAYOUT](#t03-resource-accel-layout) | T03-RESOURCE-ACCEL-LAYOUT | feature | UD-03-039 | [R12 / resources](R12_PLAN.md#resources) |
| [T03-NAV-GLOBAL-FUNCTION-SEARCH](#t03-nav-global-function-search) | T03-NAV-GLOBAL-FUNCTION-SEARCH | feature | UD-03-040 | [R11 / behavior-a01](R11_PLAN.md#behavior-a01) |
| [T03-AUTOLOAD-RESUME-FRAMING-INVESTIGATION](#t03-autoload-resume-framing-investigation) | T03-AUTOLOAD-RESUME-FRAMING-INVESTIGATION | investigation | UD-03-041 | [R13 / autoquick](R13_PLAN.md#autoquick) |
| [T03-ENG01-ISOLATED-STARTUP-FAILURE](#t03-eng01-isolated-startup-failure) | T01-DIAGNOSTICS | feature | UD-03-042 | [R12 / diagnostics](R12_PLAN.md#diagnostics) |
| [T03-UI01-TOOLBAR-SEARCH-ENTRY](#t03-ui01-toolbar-search-entry) | T03-NAV-GLOBAL-FUNCTION-SEARCH | feature | UD-03-043 | [R11 / behavior-a01](R11_PLAN.md#behavior-a01) |
| [T03-RESTORE-GTP-PATH-INVESTIGATION](#t03-restore-gtp-path-investigation) | T03-RESTORE-GTP-PATH-INVESTIGATION | investigation | UD-03-044 | [R12 / lifecycle](R12_PLAN.md#lifecycle) |
| [T03-LIFECYCLE-TARGET-CANCEL-RUN-STOP](#t03-lifecycle-target-cancel-run-stop) | T03-LIFECYCLE-TARGET-CANCEL-RUN-STOP | investigation | UD-03-045 | [R12 / lifecycle](R12_PLAN.md#lifecycle) |
| [T03-LIFECYCLE-RESTART-FENCE-INVESTIGATION](#t03-lifecycle-restart-fence-investigation) | T03-LIFECYCLE-RESTART-FENCE-INVESTIGATION | investigation | UD-03-046 | [R12 / lifecycle](R12_PLAN.md#lifecycle) |
| [T03-UI02-DIALOG-FOCUS-NAV](#t03-ui02-dialog-focus-nav) | T03-NAV-GLOBAL-FUNCTION-SEARCH | feature | UD-03-047 | [R11 / behavior-a01](R11_PLAN.md#behavior-a01) |
| [T03-CONSOLE-OUTPUT-RESPONSIVENESS](#t03-console-output-responsiveness) | T03-CONSOLE-OUTPUT-RESPONSIVENESS | feature | UD-03-048 | [R12 / diagnostics](R12_PLAN.md#diagnostics) |
| [T03-SGF13-GAMEINFO-KOMI-FOCUS](#t03-sgf13-gameinfo-komi-focus) | T03-SGF13-GAMEINFO-KOMI-FOCUS | feature | UD-03-049 | [R11 / behavior-a23](R11_PLAN.md#behavior-a23) |
| [T03-REVIEW08-AUDIO-BOUNDARY-INVESTIGATION](#t03-review08-audio-boundary-investigation) | T03-REVIEW08-AUDIO-BOUNDARY-INVESTIGATION | investigation | UD-03-050 | [R11 / behavior-a24](R11_PLAN.md#behavior-a24) |
| [T03-ANA16-EXECUTION-MODE-INDICATOR](#t03-ana16-execution-mode-indicator) | T02-AUTOLOAD-QUICK | feature | UD-03-051 | [R13 / autoquick](R13_PLAN.md#autoquick) |
| [T03-REL09-SHUTDOWN-CLEANUP](#t03-rel09-shutdown-cleanup) | T01-DIAGNOSTICS | feature | UD-03-052 | [R12 / diagnostics](R12_PLAN.md#diagnostics) |
| [T03-THREAD-ALIAS-APPLICABILITY](#t03-thread-alias-applicability) | T03-THREAD-ALIAS-APPLICABILITY | investigation | UD-03-053 | [R12 / manual-threads](R12_PLAN.md#manual-threads) |
| [T03-ANALYSIS-SAME-TREE-MOVE-FOCUS](#t03-analysis-same-tree-move-focus) | T03-ANALYSIS-SAME-TREE-MOVE-FOCUS | feature | UD-03-054 | [R13 / focus](R13_PLAN.md#focus) |
| [T03-RESOURCE-PUBLIC-GPU-GUIDANCE](#t03-resource-public-gpu-guidance) | T01-RESOURCE | feature | UD-03-010a | [R12 / resources](R12_PLAN.md#resources) |
| [T03-RESOURCE-TRUSTED-ORIGIN-IDENTITY](#t03-resource-trusted-origin-identity) | T01-RESOURCE | feature | UD-03-035a | [R12 / resources](R12_PLAN.md#resources) |
| [T03-NAV-SEARCH-CANCEL-FOCUS](#t03-nav-search-cancel-focus) | T03-NAV-GLOBAL-FUNCTION-SEARCH | feature | UD-03-040a | [R11 / behavior-a01](R11_PLAN.md#behavior-a01) |
| [T03-FOCUS-ADMISSION](#t03-focus-admission) | T03-ANALYSIS-SAME-TREE-MOVE-FOCUS | feature | UD-03-054a | [R13 / focus](R13_PLAN.md#focus) |
| [T03-FOCUS-USER-SET-PROGRESS](#t03-focus-user-set-progress) | T03-ANALYSIS-SAME-TREE-MOVE-FOCUS | feature | UD-03-054b | [R13 / focus](R13_PLAN.md#focus) |
| [T03-FOCUS-TREE-CACHE-SGF](#t03-focus-tree-cache-sgf) | T03-ANALYSIS-SAME-TREE-MOVE-FOCUS | feature | UD-03-054c | [R13 / focus](R13_PLAN.md#focus) |
| [T03-FOCUS-READBOARD](#t03-focus-readboard) | T03-FOCUS-READBOARD | feature | UD-03-054d | [R14 / readboard](R14_PLAN.md#readboard) |
| [T03-FOCUS-DISPLAY-SETTINGS](#t03-focus-display-settings) | T03-ANALYSIS-SAME-TREE-MOVE-FOCUS | feature | UD-03-054e | [R13 / focus](R13_PLAN.md#focus) |
| [T03-FOCUS-COMPATIBLE-ENGINE-EVIDENCE](#t03-focus-compatible-engine-evidence) | T03-ANALYSIS-SAME-TREE-MOVE-FOCUS | evidence-gate | UD-03-054f | [R13 / focus](R13_PLAN.md#focus) |
| [T04-RESOURCE](#t04-resource) | T01-RESOURCE | 功能增量（复用T01-RESOURCE；不复制别名） | UD-04-001,UD-04-002,UD-04-005,UD-04-006,UD-04-008,UD-04-010,UD-04-014,UD-04-016,UD-04-018,UD-04-021,UD-04-024,UD-04-026,UD-04-028,UD-04-035,UD-04-037,UD-04-038,UD-04-043 | [R12 / resources](R12_PLAN.md#resources) |
| [T04-HANDOFF](#t04-handoff) | T04-HANDOFF | 有界调查；必要时产生范围明确后继 | UD-04-004,UD-04-013 | [R13 / autoquick](R13_PLAN.md#autoquick) |
| [T04-STARTUP-PERFORMANCE](#t04-startup-performance) | T04-STARTUP-PERFORMANCE | 产品决策（未批准非等价） | UD-04-009,UD-04-013 | [R12 / startup](R12_PLAN.md#startup) |
| [T04-SYNC-CONFIRM](#t04-sync-confirm) | T04-SYNC-CONFIRM | 有界正确性调查 | UD-04-019 | [R12 / lifecycle](R12_PLAN.md#lifecycle) |
| [T04-ROLLBACK](#t04-rollback) | T04-ROLLBACK | 有界调查（普通failed-switch reader-fence） | UD-04-026 | [R12 / lifecycle](R12_PLAN.md#lifecycle) |
| [T04-HUMANSL](#t04-humansl) | T02-GAME-08 | GAME-08功能增量（T02-GAME-08来源细化） | UD-04-012,UD-04-015,UD-04-017 | [R15 / humansl](R15_PLAN.md#humansl) |
| [T04-TRANSFORM](#t04-transform) | T02-SGF-16 | SGF-16功能来源增量（T02-SGF-16） | UD-04-027 | [R11 / behavior-a10](R11_PLAN.md#behavior-a10) |
| [T04-OFFLINE](#t04-offline) | T04-OFFLINE | 有界细分调查；不重验全部no-engine功能 | UD-04-027,UD-04-033,UD-04-039 | [R11 / behavior-a25](R11_PLAN.md#behavior-a25) |
| [T04-MODEL-IDENTITY](#t04-model-identity) | T04-MODEL-IDENTITY | 本地resource/model用户功能后继 | UD-04-030,UD-04-031,UD-04-039,UD-04-043 | [R12 / models](R12_PLAN.md#models) |
| [T04-BATCH](#t04-batch) | T02-ANA-07 | ANA-07功能来源增量（T02-ANA-07） | UD-04-032 | [R13 / batch](R13_PLAN.md#batch) |
| [T04-KOMI](#t04-komi) | T04-KOMI | 具名runtime-komi正确性后继（未分配新Parity ID） | UD-04-033 | [R15 / komi](R15_PLAN.md#komi) |
| [T04-RULES](#t04-rules) | T04-RULES | 五family/custom规则功能后继 | UD-04-034 | [R15 / rules](R15_PLAN.md#rules) |
| [T04-SAVE-TARGET](#t04-save-target) | T04-SAVE-TARGET | 有界调查及raw-scope决策输入 | UD-04-039 | [R11 / behavior-a26](R11_PLAN.md#behavior-a26) |
| [T04-SAVE-ATOMIC](#t04-save-atomic) | T04-SAVE-ATOMIC | SGF保存durability后继；async scheduling细分调查 | UD-04-045 | [R11 / behavior-a27](R11_PLAN.md#behavior-a27) |
| [T04-LAN](#t04-lan) | T02-PUB-01 | PUB-01功能增量（T02-PUB-01） | UD-04-044 | [R16 / lan](R16_PLAN.md#lan) |
| [T04-DIAGNOSTICS](#t04-diagnostics) | T01-DIAGNOSTICS | 功能增量（T01-DIAGNOSTICS/REL-09功能部分） | UD-04-040,UD-04-046,UD-04-048 | [R12 / diagnostics](R12_PLAN.md#diagnostics) |
| [T04-MEASURED-TUNING](#t04-measured-tuning) | T04-MEASURED-TUNING | 具名measured-report用户功能后继 | UD-04-042,UD-04-047,UD-04-048 | [R13 / performance](R13_PLAN.md#performance) |
| [T04-THREAD-CONTROL](#t04-thread-control) | T04-THREAD-CONTROL | 有界来源/政策决策，随后功能后继 | UD-04-047 | [R12 / manual-threads](R12_PLAN.md#manual-threads) |
| [T05-RELEASE-PROVENANCE](#t05-release-provenance) | T05-RELEASE-PROVENANCE | inheritance-or-exclusion | UD-05-01 | 证据继承/排除；无新功能责任（见 [routes evidence_only](MIGRATION_ROUTES.json)） |
| [T05-REMOTE-MODEL-REFRESH](#t05-remote-model-refresh) | T01-REMOTE | see-source-disposition | UD-05-02 | [R14 / compute-network](R14_PLAN.md#compute-network) |
| [T05-WORKBENCH-ACCESS](#t05-workbench-access) | T05-WORKBENCH-ACCESS | see-source-disposition | UD-05-03 | [R17 / accessibility](R17_PLAN.md#accessibility) |
| [T05-INTERNAL-EVIDENCE](#t05-internal-evidence) | T05-INTERNAL-EVIDENCE | inheritance-or-exclusion | UD-05-04 | 证据继承/排除；无新功能责任（见 [routes evidence_only](MIGRATION_ROUTES.json)） |
| [T05-MODEL-IDENTITY](#t05-model-identity) | T04-MODEL-IDENTITY | see-source-disposition | UD-05-05 | [R12 / models](R12_PLAN.md#models) |
| [T05-SYNC-LAG-CHECK](#t05-sync-lag-check) | T05-SYNC-LAG-CHECK | see-source-disposition | UD-05-06 | [R14 / readboard](R14_PLAN.md#readboard) |
| [T05-ANCIENT-RULES](#t05-ancient-rules) | T05-ANCIENT-RULES | see-source-disposition | UD-05-07 | [R15 / rules](R15_PLAN.md#rules) |
| [T05-STARTUP-DIAGNOSTICS](#t05-startup-diagnostics) | T01-DIAGNOSTICS | see-source-disposition | UD-05-08 | [R12 / diagnostics](R12_PLAN.md#diagnostics) |
| [T05-RUNTIME-THREADS](#t05-runtime-threads) | T05-RUNTIME-THREADS | see-source-disposition | UD-05-09 | [R12 / manual-threads](R12_PLAN.md#manual-threads) |
| [T05-SECRET-DIAGNOSTICS](#t05-secret-diagnostics) | T01-DIAGNOSTICS | see-source-disposition | UD-05-10 | [R12 / diagnostics](R12_PLAN.md#diagnostics) |
| [T05-SGF-DATE-CHECK](#t05-sgf-date-check) | T05-SGF-DATE-CHECK | see-source-disposition | UD-05-11 | [R11 / behavior-a28](R11_PLAN.md#behavior-a28) |
| [T05-THEME-PRESETS](#t05-theme-presets) | T05-THEME-PRESETS | see-source-disposition | UD-05-12 | [R17 / appearance](R17_PLAN.md#appearance) |
| [T05-EXTERNAL-EXACT-RESTORE](#t05-external-exact-restore) | T05-EXTERNAL-EXACT-RESTORE | see-source-disposition | UD-05-13 | [R14 / external-match](R14_PLAN.md#external-match) |
| [T05-LOCAL-EXACT-RESTORE-CHECK](#t05-local-exact-restore-check) | T05-LOCAL-EXACT-RESTORE-CHECK | see-source-disposition | UD-05-13 | [R12 / lifecycle](R12_PLAN.md#lifecycle) |
| [T05-EXTERNAL-CONTINUATION](#t05-external-continuation) | T05-EXTERNAL-CONTINUATION | see-source-disposition | UD-05-14 | [R14 / external-match](R14_PLAN.md#external-match) |
| [T05-EXTERNAL-MODE-SWITCH](#t05-external-mode-switch) | T05-EXTERNAL-MODE-SWITCH | see-source-disposition | UD-05-15 | [R14 / external-match](R14_PLAN.md#external-match) |
| [T05-FOX-RANK-EVIDENCE](#t05-fox-rank-evidence) | T05-FOX-RANK-EVIDENCE | inheritance-or-exclusion | UD-05-16 | 证据继承/排除；无新功能责任（见 [routes evidence_only](MIGRATION_ROUTES.json)） |
| [T05-AUTO-QUICK-OWNER](#t05-auto-quick-owner) | T02-AUTOLOAD-QUICK | see-source-disposition | UD-05-17,UD-05-25 | [R13 / autoquick](R13_PLAN.md#autoquick) |
| [T05-SGF-IMPORT-CHECK](#t05-sgf-import-check) | T05-SGF-IMPORT-CHECK | see-source-disposition | UD-05-18 | [R11 / behavior-a29](R11_PLAN.md#behavior-a29) |
| [T05-TREE-PUBLICATION-CHECK](#t05-tree-publication-check) | T05-TREE-PUBLICATION-CHECK | see-source-disposition | UD-05-19 | [R13 / display](R13_PLAN.md#display) |
| [T05-PARAMETER-READBACK](#t05-parameter-readback) | T05-PARAMETER-READBACK | see-source-disposition | UD-05-20 | [R12 / runtime-readback](R12_PLAN.md#runtime-readback) |
| [T05-VARIATION-NAV-CHECK](#t05-variation-nav-check) | T05-VARIATION-NAV-CHECK | see-source-disposition | UD-05-21 | [R13 / display](R13_PLAN.md#display) |
| [T05-UPDATE-CHANNEL-DECISION](#t05-update-channel-decision) | T05-UPDATE-CHANNEL-DECISION | see-source-disposition | UD-05-22 | [R18 / channel](R18_PLAN.md#channel) |
| [T05-CANDIDATE-LIST-ACCESS](#t05-candidate-list-access) | T05-CANDIDATE-LIST-ACCESS | see-source-disposition | UD-05-23 | [R13 / display](R13_PLAN.md#display) |
| [T05-PREVIEW-ASYNC-CHECK](#t05-preview-async-check) | T05-PREVIEW-ASYNC-CHECK | see-source-disposition | UD-05-24 | [R13 / display](R13_PLAN.md#display) |
| [T05-REMOTE-QUICK-HANDBACK](#t05-remote-quick-handback) | T05-REMOTE-QUICK-HANDBACK | see-source-disposition | UD-05-25 | [R14 / remote-handback](R14_PLAN.md#remote-handback) |
| [T05-LOCAL-IMPORT-RESTART-CHECK](#t05-local-import-restart-check) | T05-LOCAL-IMPORT-RESTART-CHECK | see-source-disposition | UD-05-25 | [R12 / lifecycle](R12_PLAN.md#lifecycle) |
| [T05-FOX-IDENTITY-CHECK](#t05-fox-identity-check) | T05-FOX-IDENTITY-CHECK | see-source-disposition | UD-05-26 | [R14 / providers](R14_PLAN.md#providers) |
| [T05-RUNTIME-COMPATIBILITY](#t05-runtime-compatibility) | T01-RESOURCE | see-source-disposition | UD-05-27 | [R12 / resources](R12_PLAN.md#resources) |
| [T05-AI-CONNECTION](#t05-ai-connection) | T05-AI-CONNECTION | see-source-disposition | UD-05-28 | [R16 / ai](R16_PLAN.md#ai) |
| [T05-AI-GROUNDED-TEACHING](#t05-ai-grounded-teaching) | T05-AI-GROUNDED-TEACHING | see-source-disposition | UD-05-29 | [R16 / ai](R16_PLAN.md#ai) |
| [T05-AI-PREFERENCES](#t05-ai-preferences) | T05-AI-PREFERENCES | see-source-disposition | UD-05-30 | [R16 / ai](R16_PLAN.md#ai) |
| [T06-A-INTEGRATION](#t06-a-integration) | T06-A-INTEGRATION | integration | spec | [R11 / behavior-a30](R11_PLAN.md#behavior-a30) |
| [T06-F-INTEGRATION](#t06-f-integration) | T06-F-INTEGRATION | integration | spec | [R17 / integration](R17_PLAN.md#integration) |
| [T06-UPSTREAM-FINAL](#t06-upstream-final) | T06-UPSTREAM-FINAL | decision/audit | spec | [R17 / upstream](R17_PLAN.md#upstream) |
| [T06-RELEASE-CHANNEL](#t06-release-channel) | T06-RELEASE-CHANNEL | decision/audit | spec | [R18 / channel](R18_PLAN.md#channel) |

## 意图合同与验收输入

以下正文保留冻结审计的原始实质语义与来源 grouping；实际阶段、唯一责任和当前批准门使用上表/公开 routes，不把历史 grouping 作为活跃阶段分配。Delta 全文在公开仓库内，引用可直接读取。

<a id="t01-resource"></a>
### T01-RESOURCE

当前公开合同：[R12 / resources](R12_PLAN.md#resources)。

唯一责任意图 `T01-RESOURCE`；性质 feature。历史来源 grouping/context：B首个资源消费者；跨批共享合同。

合格本地资源身份/完整性/兼容与可见失败；保留Ready A；取得真实引擎证据，正式组件分发另验。

<a id="t01-network"></a>
### T01-NETWORK

当前公开合同：[R14 / compute-network](R14_PLAN.md#compute-network)。

唯一责任意图 `T01-NETWORK`；性质 feature。历史来源 grouping/context：C及每个新网络消费者。

不支持选定策略可见拒绝，不静默Direct；只在消费者准入范围取得真实请求/取消/秘密保护证据。

<a id="t01-ssh"></a>
### T01-SSH

当前公开合同：[R14 / ssh](R14_PLAN.md#ssh)。

唯一责任意图 `T01-SSH`；性质 feature。历史来源 grouping/context：C。

完整SSH合同、实际远端和准入adapter/平台证据；固定deadline后才实施。

<a id="t01-remote"></a>
### T01-REMOTE

当前公开合同：[R14 / compute-network](R14_PLAN.md#compute-network)。

唯一责任意图 `T01-REMOTE`；性质 feature。历史来源 grouping/context：C。

Zhizi/Custom各自服务协议与凭据来源核实，双模式标准Run；缺协议先有界调查，不能用fake endpoint验收。

<a id="t01-contribution"></a>
### T01-CONTRIBUTION

当前公开合同：[R16 / contribution](R16_PLAN.md#contribution)。

唯一责任意图 `T01-CONTRIBUTION`；性质 feature。历史来源 grouping/context：E。

官方真实客户端/上传、同意/凭据/互斥/网络/生命周期/自动存谱的功能记录；资源发行残余单列。

<a id="t01-watch"></a>
### T01-WATCH

当前公开合同：[R16 / watch](R16_PLAN.md#watch)。

唯一责任意图 `T01-WATCH`；性质 feature。历史来源 grouping/context：E。

GAME-01 + Contribution Run功能通过后，观看/导航/精确恢复真实证据。

<a id="t01-external"></a>
### T01-EXTERNAL

当前公开合同：[R14 / external-match](R14_PLAN.md#external-match)。

唯一责任意图 `T01-EXTERNAL`；性质 feature。历史来源 grouping/context：C。

双向能力准入、两模式、精确权威后继确认与Stop/失败真实验收；同步Partial不能仅靠状态直接通过。

<a id="t01-readboard"></a>
### T01-READBOARD

当前公开合同：[R14 / readboard](R14_PLAN.md#readboard)。

唯一责任意图 `T01-READBOARD`；性质 feature。历史来源 grouping/context：C。

原READ-01/02功能记录与安装残余分列；READ-03仅OCR明确不支持，不吸收sidecar生命周期。

<a id="t01-providers"></a>
### T01-PROVIDERS

当前公开合同：[R14 / providers](R14_PLAN.md#providers)。

唯一责任意图 `T01-PROVIDERS`；性质 feature。历史来源 grouping/context：C。

公共导入/同步继承按候选逐场景记录；认证读写、腾讯直播等后继另取真实证据。

<a id="t01-diagnostics"></a>
### T01-DIAGNOSTICS

当前公开合同：[R12 / diagnostics](R12_PLAN.md#diagnostics)。

唯一责任意图 `T01-DIAGNOSTICS`；性质 feature。历史来源 grouping/context：B（首个Run/资源消费者）；C/D/E各消费者扩展；R11仅安装支持包。

必要日志/脱敏/受限可取消导出/可见错误随功能，保留full trace默认关和无自动上传。

<a id="t01-identity"></a>
### T01-IDENTITY

当前公开合同：[R11 / behavior-a02](R11_PLAN.md#behavior-a02)。

唯一责任意图 `T01-IDENTITY`；性质 feature。历史来源 grouping/context：A（身份漂移）；R11渠道决定。

About版本及repository漂移具名修复；产品数据身份不变，未配置渠道不伪造入口，最终仓库待决。

<a id="t01-activation"></a>
### T01-ACTIVATION

当前公开合同：[R18 / activation](R18_PLAN.md#activation)。

唯一责任意图 `T01-ACTIVATION`；性质 feature。历史来源 grouping/context：R11发行负责人。

继承原native激活范围；保留安装关联与最终产物未执行义务。

<a id="t01-release"></a>
### T01-RELEASE

当前公开合同：[R18 / discovery-components](R18_PLAN.md#discovery-components)。

唯一责任意图 `T01-RELEASE`；性质 feature。历史来源 grouping/context：R11。

签名获取、安装路径、信任、关联、升级卸载、最终产物及支持包逐平台验收；不阻塞独立已通过功能，不伪造通过。

<a id="t02-h01"></a>
### T02-H01

当前公开合同：[R11 / behavior-a03](R11_PLAN.md#behavior-a03)。

唯一责任意图 `T02-H01`；性质 decision。历史来源 grouping/context：A。

`read` launch mode / SHELL-04 | No-engine review `UI-04`；[固定来源](https://github.com/qiyi71w/lizzieyzy-next-tauri/blob/55795fab49b80a681a9fa544950e18f5f2da4cc6/docs/JAVA_CAPABILITY_INVENTORY.md#L426) | 保留未等价用户目标；核对原对应处置票及逐项批准来源，产出保留功能合同或经批准非等价理由；未决不能删除或自动实施新语义。

<a id="t02-h02"></a>
### T02-H02

当前公开合同：[R11 / behavior-a04](R11_PLAN.md#behavior-a04)。

唯一责任意图 `T02-H02`；性质 decision。历史来源 grouping/context：A。

Visible Force Exit / SHELL-09 | Contextual Exit anyway only inside `APP-03` after teardown timeout；[固定来源](https://github.com/qiyi71w/lizzieyzy-next-tauri/blob/55795fab49b80a681a9fa544950e18f5f2da4cc6/docs/JAVA_CAPABILITY_INVENTORY.md#L427) | 保留未等价用户目标；核对原对应处置票及逐项批准来源，产出保留功能合同或经批准非等价理由；未决不能删除或自动实施新语义。

<a id="t02-h03"></a>
### T02-H03

当前公开合同：[R17 / initialization](R17_PLAN.md#initialization)。

唯一责任意图 `T02-H03`；性质 decision。历史来源 grouping/context：F。

Hostname-triggered persist/preference deletion / SET-FIRST-LAUNCH (hostname-wipe half) | `WINDOW-01` invalid-geometry reset only；[固定来源](https://github.com/qiyi71w/lizzieyzy-next-tauri/blob/55795fab49b80a681a9fa544950e18f5f2da4cc6/docs/JAVA_CAPABILITY_INVENTORY.md#L428) | 保留未等价用户目标；核对原对应处置票及逐项批准来源，产出保留功能合同或经批准非等价理由；未决不能删除或自动实施新语义。

<a id="t02-h04"></a>
### T02-H04

当前公开合同：[R17 / initialization](R17_PLAN.md#initialization)。

唯一责任意图 `T02-H04`；性质 decision。历史来源 grouping/context：F。

Forced initialize-settings wizard / SET-FIRST-USE | `PREF-01` surface；[固定来源](https://github.com/qiyi71w/lizzieyzy-next-tauri/blob/55795fab49b80a681a9fa544950e18f5f2da4cc6/docs/JAVA_CAPABILITY_INVENTORY.md#L429) | 保留未等价用户目标；核对原对应处置票及逐项批准来源，产出保留功能合同或经批准非等价理由；未决不能删除或自动实施新语义。

<a id="t02-h05"></a>
### T02-H05

当前公开合同：[R17 / appearance](R17_PLAN.md#appearance)。

唯一责任意图 `T02-H05`；性质 decision。历史来源 grouping/context：F。

Application font-size slider / SET-FRAME-FONT | System DPI；[固定来源](https://github.com/qiyi71w/lizzieyzy-next-tauri/blob/55795fab49b80a681a9fa544950e18f5f2da4cc6/docs/JAVA_CAPABILITY_INVENTORY.md#L430) | 保留未等价用户目标；核对原对应处置票及逐项批准来源，产出保留功能合同或经批准非等价理由；未决不能删除或自动实施新语义。

<a id="t02-h06"></a>
### T02-H06

当前公开合同：[R17 / appearance](R17_PLAN.md#appearance)。

唯一责任意图 `T02-H06`；性质 decision。历史来源 grouping/context：F。

Apple/Morandi/custom board assets / SET-THEME-APPLE-CLASSIC-CUSTOM | `APPEAR-01` curated pair；[固定来源](https://github.com/qiyi71w/lizzieyzy-next-tauri/blob/55795fab49b80a681a9fa544950e18f5f2da4cc6/docs/JAVA_CAPABILITY_INVENTORY.md#L431) | 保留未等价用户目标；核对原对应处置票及逐项批准来源，产出保留功能合同或经批准非等价理由；未决不能删除或自动实施新语义。

<a id="t02-h07"></a>
### T02-H07

当前公开合同：[R17 / appearance](R17_PLAN.md#appearance)。

唯一责任意图 `T02-H07`；性质 decision。历史来源 grouping/context：F。

Separate theme editor / SET-THEME-DIALOG | `APPEAR-01` in Preferences；[固定来源](https://github.com/qiyi71w/lizzieyzy-next-tauri/blob/55795fab49b80a681a9fa544950e18f5f2da4cc6/docs/JAVA_CAPABILITY_INVENTORY.md#L432) | 保留未等价用户目标；核对原对应处置票及逐项批准来源，产出保留功能合同或经批准非等价理由；未决不能删除或自动实施新语义。

<a id="t02-h08"></a>
### T02-H08

当前公开合同：[R17 / appearance](R17_PLAN.md#appearance)。

唯一责任意图 `T02-H08`；性质 decision。历史来源 grouping/context：F。

ExtraMode / classic / custom layouts / SET-LAYOUT-MODE | One adaptive workspace；[固定来源](https://github.com/qiyi71w/lizzieyzy-next-tauri/blob/55795fab49b80a681a9fa544950e18f5f2da4cc6/docs/JAVA_CAPABILITY_INVENTORY.md#L433) | 保留未等价用户目标；核对原对应处置票及逐项批准来源，产出保留功能合同或经批准非等价理由；未决不能删除或自动实施新语义。

<a id="t02-h09"></a>
### T02-H09

当前公开合同：[R17 / appearance](R17_PLAN.md#appearance)。

唯一责任意图 `T02-H09`；性质 decision。历史来源 grouping/context：F。

Toolbar wrap/order/visibility prefs / SET-LAYOUT-TOOLBAR | Fixed Next toolbar；[固定来源](https://github.com/qiyi71w/lizzieyzy-next-tauri/blob/55795fab49b80a681a9fa544950e18f5f2da4cc6/docs/JAVA_CAPABILITY_INVENTORY.md#L434) | 保留未等价用户目标；核对原对应处置票及逐项批准来源，产出保留功能合同或经批准非等价理由；未决不能删除或自动实施新语义。

<a id="t02-h10"></a>
### T02-H10

当前公开合同：[R16 / guidance](R16_PLAN.md#guidance)。

唯一责任意图 `T02-H10`；性质 decision。历史来源 grouping/context：E。

Comment-control instructional popup / SET-HINT-COMMENT-CTRL | —；[固定来源](https://github.com/qiyi71w/lizzieyzy-next-tauri/blob/55795fab49b80a681a9fa544950e18f5f2da4cc6/docs/JAVA_CAPABILITY_INVENTORY.md#L435) | 保留未等价用户目标；核对原对应处置票及逐项批准来源，产出保留功能合同或经批准非等价理由；未决不能删除或自动实施新语义。

<a id="t02-h11"></a>
### T02-H11

当前公开合同：[R16 / guidance](R16_PLAN.md#guidance)。

唯一责任意图 `T02-H11`；性质 decision。历史来源 grouping/context：E。

Auto-analyze pause-exit educational tip / SET-HINT-AUTOANALYZE | Separate guidance obligation; `CAP-04-ANA-08` restores through `ANA-16`, while `GUIDE-01` remains Deferred pending producer admission；[固定来源](https://github.com/qiyi71w/lizzieyzy-next-tauri/blob/55795fab49b80a681a9fa544950e18f5f2da4cc6/docs/JAVA_CAPABILITY_INVENTORY.md#L436) | 保留未等价用户目标；核对原对应处置票及逐项批准来源，产出保留功能合同或经批准非等价理由；未决不能删除或自动实施新语义。

<a id="t02-h12"></a>
### T02-H12

当前公开合同：[R11 / behavior-a05](R11_PLAN.md#behavior-a05)。

唯一责任意图 `T02-H12`；性质 decision。历史来源 grouping/context：A。

Lossy Swing raw / raw-with-comment saves / SGF-03-ADJ-SAVE-MORE (raw subset) | `SGF-05` comment separation; Save remains `SGF-06`；[固定来源](https://github.com/qiyi71w/lizzieyzy-next-tauri/blob/55795fab49b80a681a9fa544950e18f5f2da4cc6/docs/JAVA_CAPABILITY_INVENTORY.md#L437) | 保留未等价用户目标；核对原对应处置票及逐项批准来源，产出保留功能合同或经批准非等价理由；未决不能删除或自动实施新语义。

<a id="t02-h13"></a>
### T02-H13

当前公开合同：[R11 / behavior-a06](R11_PLAN.md#behavior-a06)。

唯一责任意图 `T02-H13`；性质 decision。历史来源 grouping/context：A。

Sub-Board Image Export / SGF-03-ADJ-SAVE-MORE (sub-board image subset) | —；[固定来源](https://github.com/qiyi71w/lizzieyzy-next-tauri/blob/55795fab49b80a681a9fa544950e18f5f2da4cc6/docs/JAVA_CAPABILITY_INVENTORY.md#L438) | Ticket27 §Answer明确可逆产品决定而非ADR缺失违规；本轮仍保留Sub-Board图像用户目标与具名决定，确定与EXPORT02/03共享写入规则后交A，不能声称主盘导出等价。

<a id="t02-h14"></a>
### T02-H14

当前公开合同：[R11 / behavior-a07](R11_PLAN.md#behavior-a07)。

唯一责任意图 `T02-H14`；性质 decision。历史来源 grouping/context：A。

Thumbnail slots / second autosave-on-exit / SGF-03-ADJ-TEMP (manual slots) | `APP-04`；[固定来源](https://github.com/qiyi71w/lizzieyzy-next-tauri/blob/55795fab49b80a681a9fa544950e18f5f2da4cc6/docs/JAVA_CAPABILITY_INVENTORY.md#L439) | 保留未等价用户目标；核对原对应处置票及逐项批准来源，产出保留功能合同或经批准非等价理由；未决不能删除或自动实施新语义。

<a id="t02-h15"></a>
### T02-H15

当前公开合同：[R11 / behavior-a08](R11_PLAN.md#behavior-a08)。

唯一责任意图 `T02-H15`；性质 decision。历史来源 grouping/context：A。

Conditional ignore of file `KM` / SGF-03-ADJ-KOMI (ignore-file-komi pref) | Root `KM` authoritative；[固定来源](https://github.com/qiyi71w/lizzieyzy-next-tauri/blob/55795fab49b80a681a9fa544950e18f5f2da4cc6/docs/JAVA_CAPABILITY_INVENTORY.md#L440) | 保留未等价用户目标；核对原对应处置票及逐项批准来源，产出保留功能合同或经批准非等价理由；未决不能删除或自动实施新语义。

<a id="t02-h16"></a>
### T02-H16

当前公开合同：[R13 / display](R13_PLAN.md#display)。

唯一责任意图 `T02-H16`；性质 decision。历史来源 grouping/context：B。

Java 200 ms + delay dialog / manual reveal / SGF-03-ADJ-HOVER-DELAY | `UI-03` 120 ms；[固定来源](https://github.com/qiyi71w/lizzieyzy-next-tauri/blob/55795fab49b80a681a9fa544950e18f5f2da4cc6/docs/JAVA_CAPABILITY_INVENTORY.md#L441) | 保留未等价用户目标；核对原对应处置票及逐项批准来源，产出保留功能合同或经批准非等价理由；未决不能删除或自动实施新语义。

<a id="t02-h17"></a>
### T02-H17

当前公开合同：[R12 / startup](R12_PLAN.md#startup)。

唯一责任意图 `T02-H17`；性质 decision。历史来源 grouping/context：B。

Last-engine autoload / CAP-04-ENG-03 | `ENG-06` Autoload Default；[固定来源](https://github.com/qiyi71w/lizzieyzy-next-tauri/blob/55795fab49b80a681a9fa544950e18f5f2da4cc6/docs/JAVA_CAPABILITY_INVENTORY.md#L442) | 保留未等价用户目标；核对原对应处置票及逐项批准来源，产出保留功能合同或经批准非等价理由；未决不能删除或自动实施新语义。

<a id="t02-h18"></a>
### T02-H18

当前公开合同：[R12 / startup](R12_PLAN.md#startup)。

唯一责任意图 `T02-H18`；性质 decision。历史来源 grouping/context：B。

Background preload of extra GTP engines / CAP-04-ENG-09 | Switch / Match Reservation processes only；[固定来源](https://github.com/qiyi71w/lizzieyzy-next-tauri/blob/55795fab49b80a681a9fa544950e18f5f2da4cc6/docs/JAVA_CAPABILITY_INVENTORY.md#L443) | 保留未等价用户目标；核对原对应处置票及逐项批准来源，产出保留功能合同或经批准非等价理由；未决不能删除或自动实施新语义。

<a id="t02-h19"></a>
### T02-H19

当前公开合同：[R13 / retained-analysis](R13_PLAN.md#retained-analysis)。

唯一责任意图 `T02-H19`；性质 decision。历史来源 grouping/context：B。

Java in-tree cache toggle / CAP-04-PREF-LIZZIE-CACHE | `ANA-05` SQLite；[固定来源](https://github.com/qiyi71w/lizzieyzy-next-tauri/blob/55795fab49b80a681a9fa544950e18f5f2da4cc6/docs/JAVA_CAPABILITY_INVENTORY.md#L444) | 保留未等价用户目标；核对原对应处置票及逐项批准来源，产出保留功能合同或经批准非等价理由；未决不能删除或自动实施新语义。

<a id="t02-h20"></a>
### T02-H20

当前公开合同：[R13 / retained-analysis](R13_PLAN.md#retained-analysis)。

唯一责任意图 `T02-H20`；性质 decision。历史来源 grouping/context：B。

Lightning / part / all-branches flash / CAP-04-ANA-06 | `ANA-16` explicit task; automatic-on-load remainder stays a separate obligation；[固定来源](https://github.com/qiyi71w/lizzieyzy-next-tauri/blob/55795fab49b80a681a9fa544950e18f5f2da4cc6/docs/JAVA_CAPABILITY_INVENTORY.md#L445) | 保留未等价用户目标；核对原对应处置票及逐项批准来源，产出保留功能合同或经批准非等价理由；未决不能删除或自动实施新语义。

<a id="t02-h21"></a>
### T02-H21

当前公开合同：[R13 / retained-analysis](R13_PLAN.md#retained-analysis)。

唯一责任意图 `T02-H21`；性质 decision。历史来源 grouping/context：B。

Automatic current-game analysis / CAP-04-ANA-08 | `ANA-16` range/color/budgets/Pause/Continue/two-stage task; guidance remains separately owned；[固定来源](https://github.com/qiyi71w/lizzieyzy-next-tauri/blob/55795fab49b80a681a9fa544950e18f5f2da4cc6/docs/JAVA_CAPABILITY_INVENTORY.md#L446) | 保留未等价用户目标；核对原对应处置票及逐项批准来源，产出保留功能合同或经批准非等价理由；未决不能删除或自动实施新语义。

<a id="t02-h22"></a>
### T02-H22

当前公开合同：[R17 / appearance](R17_PLAN.md#appearance)。

唯一责任意图 `T02-H22`；性质 decision。历史来源 grouping/context：F。

Mutually exclusive large-sub / large-WR enlarge presets / SET-MAIN-PANEL (large extras) | `LAYOUT-01` / `LAYOUT-04` adaptive workspace; `UI-01` chart/mini-board presence；[固定来源](https://github.com/qiyi71w/lizzieyzy-next-tauri/blob/55795fab49b80a681a9fa544950e18f5f2da4cc6/docs/JAVA_CAPABILITY_INVENTORY.md#L447) | 保留未等价用户目标；核对原对应处置票及逐项批准来源，产出保留功能合同或经批准非等价理由；未决不能删除或自动实施新语义。

<a id="t02-h23"></a>
### T02-H23

当前公开合同：[R16 / generated](R16_PLAN.md#generated)。

唯一责任意图 `T02-H23`；性质 decision。历史来源 grouping/context：E。

Write generated winrate/score/playouts into personal `C` / SET-MAIN-PANEL (append-WR extra) | `SGF-05` personal comments; analysis pane display; deferred `ANA-08` structured headers；[固定来源](https://github.com/qiyi71w/lizzieyzy-next-tauri/blob/55795fab49b80a681a9fa544950e18f5f2da4cc6/docs/JAVA_CAPABILITY_INVENTORY.md#L448) | ADR0003批准生成信息不写个人C；保留个人C与生成信息分离，结构化分析头ANA-08另验，不据此删除所有生成信息显示/导出目标。

<a id="t02-h24"></a>
### T02-H24

当前公开合同：[R13 / tracking](R13_PLAN.md#tracking)。

唯一责任意图 `T02-H24`；性质 decision。历史来源 grouping/context：B。

Tracking analysis jobs / CAP-04-ANA-10 | `UI-03` hover eviction unchanged; readboard stays 06；[固定来源](https://github.com/qiyi71w/lizzieyzy-next-tauri/blob/55795fab49b80a681a9fa544950e18f5f2da4cc6/docs/JAVA_CAPABILITY_INVENTORY.md#L449) | 保留未等价用户目标；核对原对应处置票及逐项批准来源，产出保留功能合同或经批准非等价理由；未决不能删除或自动实施新语义。

<a id="t02-h25"></a>
### T02-H25

当前公开合同：[R15 / human-clock](R15_PLAN.md#human-clock)。

唯一责任意图 `T02-H25`；性质 decision。历史来源 grouping/context：D。

Legacy genmove/analysis selector, raw timing, pure-net, play-mode overlay / GM-HUMAN-GENMOVE / GM-HUMAN-ANA (controls, not extra census IDs) | Adapter capability + `GAME-02`；[固定来源](https://github.com/qiyi71w/lizzieyzy-next-tauri/blob/55795fab49b80a681a9fa544950e18f5f2da4cc6/docs/JAVA_CAPABILITY_INVENTORY.md#L450) | 保留未等价用户目标；核对原对应处置票及逐项批准来源，产出保留功能合同或经批准非等价理由；未决不能删除或自动实施新语义。

<a id="t02-h26"></a>
### T02-H26

当前公开合同：[R14 / yike](R14_PLAN.md#yike)。

唯一责任意图 `T02-H26`；性质 decision。历史来源 grouping/context：C。

JCEF embedded Yike page/hall / CAP-06-YIKE-WEB / CAP-06-YIKE-HALL (embedded hosting half) | `PROV-03` Play & Sync; locators remain；[固定来源](https://github.com/qiyi71w/lizzieyzy-next-tauri/blob/55795fab49b80a681a9fa544950e18f5f2da4cc6/docs/JAVA_CAPABILITY_INVENTORY.md#L451) | 保留未等价用户目标；核对原对应处置票及逐项批准来源，产出保留功能合同或经批准非等价理由；未决不能删除或自动实施新语义。

<a id="t02-h27"></a>
### T02-H27

当前公开合同：[R14 / yike](R14_PLAN.md#yike)。

唯一责任意图 `T02-H27`；性质 decision。历史来源 grouping/context：C。

Reachable share shortcuts that do nothing / CAP-06-SHARE-CURRENT | No empty Next share actions；[固定来源](https://github.com/qiyi71w/lizzieyzy-next-tauri/blob/55795fab49b80a681a9fa544950e18f5f2da4cc6/docs/JAVA_CAPABILITY_INVENTORY.md#L452) | 保留未等价用户目标；核对原对应处置票及逐项批准来源，产出保留功能合同或经批准非等价理由；未决不能删除或自动实施新语义。

<a id="t02-h28"></a>
### T02-H28

当前公开合同：[R18 / channel](R18_PLAN.md#channel)。

唯一责任意图 `T02-H28`；性质 decision。历史来源 grouping/context：R11。

Flavor guessing, bundled JRE, JCEF as updater components / REL-C04 remainder | `REL-05` manifest of `app-core` + acquired KataGo components；[固定来源](https://github.com/qiyi71w/lizzieyzy-next-tauri/blob/55795fab49b80a681a9fa544950e18f5f2da4cc6/docs/JAVA_CAPABILITY_INVENTORY.md#L453) | 保留未等价用户目标；核对原对应处置票及逐项批准来源，产出保留功能合同或经批准非等价理由；未决不能删除或自动实施新语义。

<a id="t02-ui-02"></a>
### T02-UI-02

当前公开合同：[R17 / accessibility](R17_PLAN.md#accessibility)。

唯一责任意图 `T02-UI-02`；性质 feature。历史来源 grouping/context：F。

Pointer/keyboard move intent updates the selected node without blocking rendering or engine events; illegal intent receives visible feedback.

<a id="t02-rel-01"></a>
### T02-REL-01

当前公开合同：[R18 / preflight-trust](R18_PLAN.md#preflight-trust)。

唯一责任意图 `T02-REL-01`；性质 feature。历史来源 grouping/context：R11（运行必需部分随首消费者）。

Validators and non-mutating dry-run execute on the exact release commit, identify tag/commit/platform/signing state, and state their limits. This is a maintainer gate, not an end-user Capability or publication proof.

<a id="t02-rel-02"></a>
### T02-REL-02

当前公开合同：[R18 / preflight-trust](R18_PLAN.md#preflight-trust)。

唯一责任意图 `T02-REL-02`；性质 feature。历史来源 grouping/context：R11（运行必需部分随首消费者）。

Every artifact offered through stable or beta update channels has a trusted signed envelope and verified payload; Windows production artifacts are Authenticode-signed and timestamped; macOS artifacts are signed, notarized, and stapled. Unsigned public-validation prereleases are labeled as such and never enter an updater feed. Linux may publish an AppImage without a repository/package-manager signature only when updater envelope/payload verification and release notes state the exact trust contract.

<a id="t02-rel-03"></a>
### T02-REL-03

当前公开合同：[R18 / discovery-components](R18_PLAN.md#discovery-components)。

唯一责任意图 `T02-REL-03`；性质 feature。历史来源 grouping/context：R11（运行必需部分随首消费者）。

Help-triggered manual discovery implements stable/beta and official/GitHub policy, persists valid choices, compares SemVer, shows release identity/notes, follows the system proxy, and reports unpackaged, unsupported, no-update, no-package, fetch, schema, key, signature, and version failures without mutating the install.

<a id="t02-rel-04"></a>
### T02-REL-04

当前公开合同：[R18 / installed](R18_PLAN.md#installed)。

唯一责任意图 `T02-REL-04`；性质 feature。历史来源 grouping/context：R11（运行必需部分随首消费者）。

Each Canonical Artifact independently satisfies its own runtime, trust, state, update/handoff, and removal contracts: Windows NSIS (Evergreen WebView2 bootstrap when absent; Authenticode+timestamp; OS app-data; `REL-06` apply; uninstall preserves OS app-data by default); Windows portable (system Evergreen WebView2 present and absent; Authenticode+timestamp; package-local `user-data/`; `REL-06` without changing `user-data/`; directory delete is destructive); macOS DMG (platform WebKit; launch from Applications, not the mounted image; signed/notarized/stapled; OS app-data; `REL-07` opens the DMG; deleting the app preserves OS app-data by default); Linux AppImage (documented distribution/runtime with user-visible failure when missing; envelope/payload verification plus release notes stating the exact trust and dependency contract; OS app-data; `REL-07` opens the containing folder; deleting the AppImage preserves OS app-data by default). Each also passes install/unpack, primary launch, and upgrade. Packaging supplies `APP-01` association evidence without owning file-open semantics. Platforms are admitted independently. Extra CI MSI/deb/rpm formats are not Canonical Artifacts.

<a id="t02-rel-06"></a>
### T02-REL-06

当前公开合同：[R18 / windows-update](R18_PLAN.md#windows-update)。

唯一责任意图 `T02-REL-06`；性质 feature。历史来源 grouping/context：R11（运行必需部分随首消费者）。

NSIS and portable installs select newer managed components, resume/cancel verified downloads, fall back between supported sources, launch the helper with elevation only when required, close through `APP-03` only after successful helper launch, replace the declared paths, update the manifest, and restart without changing package-local or OS app-data.

<a id="t02-rel-07"></a>
### T02-REL-07

当前公开合同：[R18 / handoff](R18_PLAN.md#handoff)。

唯一责任意图 `T02-REL-07`；性质 feature。历史来源 grouping/context：R11（运行必需部分随首消费者）。

The selected channel/source downloads and verifies the matching DMG/AppImage with resume/cancel/fallback, then opens the DMG or containing folder and gives the platform-specific next step. The running install is not overwritten and failed acquisition leaves it usable.

<a id="t02-rel-08"></a>
### T02-REL-08

当前公开合同：[R18 / windows-update](R18_PLAN.md#windows-update)。

唯一责任意图 `T02-REL-08`；性质 feature。历史来源 grouping/context：R11（运行必需部分随首消费者）。

Invalid metadata/payload and helper-launch failures leave the current application running. Windows apply creates a complete replacement backup, rolls back in reverse, journals the result, and restarts the restored version. The application reports successful restoration with diagnostics/retry; incomplete restoration retains the backup and provides an explicit repair/reinstall path.

<a id="t02-i18n-01"></a>
### T02-I18N-01

当前公开合同：[R17 / localization](R17_PLAN.md#localization)。

唯一责任意图 `T02-I18N-01`；性质 feature。历史来源 grouping/context：F。

基础结构首消费者T02-I18N-FOUNDATION，完整翻译F。Complete resources for every supported locale, persisted locale selection, deterministic fallback behavior, and no mixed partial-language state. Admit only after functional migration.

<a id="t02-guide-01"></a>
### T02-GUIDE-01

当前公开合同：[R16 / guidance](R16_PLAN.md#guidance)。

唯一责任意图 `T02-GUIDE-01`；性质 feature。历史来源 grouping/context：E。

先T02-GUIDANCE-PRODUCER，不交空Reset。Persist dismissals only for named Educational Tips; dedicated Reset Guidance re-enables those tips; Safety Confirmations stay non-dismissible. Leave Deferred only after a later disposition names a Guidance Producer whose owner is in a numbered phase (`Missing`/`Partial`/`Accepted`). The same ticket may admit the owner and name the producer. `ANA-06` ponder-limit and Ticket 25 GMA/readboard notices do not auto-start this item; later naming is allowed after those owners are in a numbered phase. Unnamed persist-dismiss stays with the producing capability.

<a id="t02-sgf-15"></a>
### T02-SGF-15

当前公开合同：[R11 / behavior-a09](R11_PLAN.md#behavior-a09)。

唯一责任意图 `T02-SGF-15`；性质 feature。历史来源 grouping/context：A。

Explicit SGF setup/move semantics, bounds/occupancy validation, preservation of unrelated branches/properties, reversible edits, non-mutating rejection, and save/reopen coverage.

<a id="t02-sgf-16"></a>
### T02-SGF-16

当前公开合同：[R11 / behavior-a10](R11_PLAN.md#behavior-a10)。

唯一责任意图 `T02-SGF-16`；性质 feature。历史来源 grouping/context：A。

Every supported coordinate-bearing move, setup, and markup property transforms consistently across all branches while `NodePath` structure and unrelated metadata remain stable; color swap also exchanges color-owned properties; serialize/reparse preserves the result.

<a id="t02-review-04"></a>
### T02-REVIEW-04

当前公开合同：[R11 / behavior-a11](R11_PLAN.md#behavior-a11)。

唯一责任意图 `T02-REVIEW-04`；性质 feature。历史来源 grouping/context：A。

Deterministic traversal to matching moves across branches, exact `NodePath` selection through `REVIEW-01`, and a non-mutating no-match result.

<a id="t02-review-05"></a>
### T02-REVIEW-05

当前公开合同：[R11 / behavior-a12](R11_PLAN.md#behavior-a12)。

唯一责任意图 `T02-REVIEW-05`；性质 feature。历史来源 grouping/context：A。

A validated persisted interval, one active timer, deterministic selected-`NodePath` progression along the chosen continuation, stop at a leaf or scope change, and no change to the accepted fixed-toggle history in `UI-05`.

<a id="t02-review-06"></a>
### T02-REVIEW-06

当前公开合同：[R11 / behavior-a13](R11_PLAN.md#behavior-a13)。

唯一责任意图 `T02-REVIEW-06`；性质 feature。历史来源 grouping/context：A。

A reproducible ladder predicate, legal continuation from the selected node, explanatory refusal below the supported threshold, and no partial tree mutation on failure.

<a id="t02-export-01"></a>
### T02-EXPORT-01

当前公开合同：[R11 / behavior-a14](R11_PLAN.md#behavior-a14)。

唯一责任意图 `T02-EXPORT-01`；性质 feature。历史来源 grouping/context：A。

Export of the selected root-to-leaf branch as a standalone main line whose semantic reopen matches that path, without mutating current tree order, source path, cursor, or dirty state.

<a id="t02-export-02"></a>
### T02-EXPORT-02

当前公开合同：[R11 / behavior-a15](R11_PLAN.md#behavior-a15)。

唯一责任意图 `T02-EXPORT-02`；性质 feature。历史来源 grouping/context：A。

Export of the current main-board review view to a documented image format and size without changing current-game or review state. It shares the Recent Image Export Directory with `EXPORT-03`; only a successful image write updates that directory.

<a id="t02-export-03"></a>
### T02-EXPORT-03

当前公开合同：[R11 / behavior-a16](R11_PLAN.md#behavior-a16)。

唯一责任意图 `T02-EXPORT-03`；性质 feature。历史来源 grouping/context：A。

With at least one identity-valid selected-line point, File → 更多保存 → 保存胜率图截图 and registry-owned `Shift+Alt+S` freeze the current `ANA-11` line, encoding, gaps, axis, and marker before destination selection, then redraw a 1600 × 600 PNG containing visible series, Blunder Bar, axes/scales, marker, and fixed perspective/series labels but no hover or application chrome. The native dialog pre-fills `<sgf-stem>-winrate-m<selectedMove>.png` or the untitled fallback. Success alone updates the shared Recent Image Export Directory. Confirmed overwrite is atomic; cancel or encode/write/replace failure preserves the target, directory, current-game state, jobs, and chart settings and reports the failure. No-data refusal never starts analysis.

<a id="t02-eng-08"></a>
### T02-ENG-08

当前公开合同：[R11 / behavior-a17](R11_PLAN.md#behavior-a17)。

唯一责任意图 `T02-ENG-08`；性质 feature。历史来源 grouping/context：A。

Reordering persists stable profile identities without changing Engine Settings selection, Autoload Default, active run, pending edits, or job binding. Rejected or failed persistence leaves the durable order unchanged.

<a id="t02-ana-07"></a>
### T02-ANA-07

当前公开合同：[R13 / batch](R13_PLAN.md#batch)。

唯一责任意图 `T02-ANA-07`；性质 feature。历史来源 grouping/context：B。

A session-only ordered SGF queue, visible per-file pending/running/completed/failed/cancelled state, explicit cancellation, and run/job/file identity that blocks stale publication. Queue state is not persisted or restored, and batch work cannot replace or dirty the current game.

<a id="t02-ana-09"></a>
### T02-ANA-09

当前公开合同：[R13 / adapters](R13_PLAN.md#adapters)。

唯一责任意图 `T02-ANA-09`；性质 feature。历史来源 grouping/context：B。

先T02-ADAPTER-EVIDENCE具名版本产品证据。Each admitted engine/version requires a controlled protocol fixture mapping its candidates, winrate/PV, ownership, streaming, selected-node, and whole-game fields into the Next analysis model. Capabilities are declared independently; missing fields remain unavailable rather than fabricated.

<a id="t02-game-06"></a>
### T02-GAME-06

当前公开合同：[R15 / pk](R15_PLAN.md#pk)。

唯一责任意图 `T02-GAME-06`；性质 feature。历史来源 grouping/context：D。

A session-only ordered batch, SGF opening catalog and deterministic sequential/random selection, color exchange, live revision of remaining count, manual intervention, visible per-game state, and durable completed-game output. Application restart does not restore an unfinished queue; completed files remain durable.

<a id="t02-game-07"></a>
### T02-GAME-07

当前公开合同：[R15 / human-clock](R15_PLAN.md#human-clock)。

唯一责任意图 `T02-GAME-07`；性质 feature。历史来源 grouping/context：D。

先T02-CLOCK-DECISION单独批准政策。An application-owned remaining-time model, visible clocks, adapter mapping, deterministic pause/resume and timeout result semantics, save/recovery boundaries, and explicit auto-resign thresholds and evidence. A Compute Budget timeout cannot satisfy this item.

<a id="t02-game-08"></a>
### T02-GAME-08

当前公开合同：[R15 / humansl](R15_PLAN.md#humansl)。

唯一责任意图 `T02-GAME-08`；性质 feature。历史来源 grouping/context：D。

Independent preset/rank/color setup, pass, retry AI, finish/review, exact start rollback, typed AI failure, and teardown. Live analysis consumes `ANA-04`; post-game reporting consumes the review/SGF contracts.

<a id="t02-prov-05"></a>
### T02-PROV-05

当前公开合同：[R14 / tencent](R14_PLAN.md#tencent)。

唯一责任意图 `T02-PROV-05`；性质 feature。历史来源 grouping/context：C。

先T02-TENCENT-PROTOCOL协议证据。Ongoing-synchronization of Tencent/huanle live rooms as a separate item from `PROV-04`, with the shared timeout, retry, recovery, privacy, and evidence contracts.

<a id="t02-prov-06"></a>
### T02-PROV-06

当前公开合同：[R14 / yike](R14_PLAN.md#yike)。

唯一责任意图 `T02-PROV-06`；性质 feature。历史来源 grouping/context：C。

先T02-PERSONAL-EVIDENCE真实guest语义；若需auth，按原晋级门阻塞。Keep Recommend as default and do not persist category/page. Personal discovery supports pagination/`since`, client filtering, typed populated/empty/failure outcomes, and handoff to `PROV-01`/`PROV-03` without owning import/sync. If live evidence proves authentication is required, remain Deferred until a later plan revision explicitly adds `PROV-07`.

<a id="t02-prov-07"></a>
### T02-PROV-07

当前公开合同：[R14 / yike](R14_PLAN.md#yike)。

唯一责任意图 `T02-PROV-07`；性质 feature。历史来源 grouping/context：C。

先T02-YIKE-AUTH受支持协议和授权证据；ADR0005不变。One provider-supported account stores its secret only in the System Credential Store and bounded non-secret locator recents through `PREF-01`. Explicit Start for each independently admitted locator family transactionally reuses `PROV-03`, `GAME-01`, and `GAME-05`; exact account/side/position/turn and visible provider clock admit writes. Move/Pass/confirmed Resign create one Pending Provider Move and commit only on exact authoritative readback; writes never auto-retry, transient reads retry at most three times, an alternate legal successor wins, and Error/Reconcile retains the reservation until read-only recovery or warned Stop. Yike owns terminal `RE`; restart restores the account reference and last-confirmed review state, never the live session. Repository and per-Shipped-Platform live evidence cover authorization/credential lifecycle, every admitted family, switch rollback, conflicts, timeout/expiry, Stop/Disconnect, and Provider Network Policy.

<a id="t02-pub-01"></a>
### T02-PUB-01

当前公开合同：[R16 / lan](R16_PLAN.md#lan)。

唯一责任意图 `T02-PUB-01`；性质 feature。历史来源 grouping/context：E。

Start/stop LAN publish and copy-access URL. No credentials. Trial counters/internal trial mechanics are out of scope. Inbound LAN is excluded from Provider Network Policy.

<a id="t02-autoload-quick"></a>
### T02-AUTOLOAD-QUICK

当前公开合同：[R13 / autoquick](R13_PLAN.md#autoquick)。

唯一责任意图 `T02-AUTOLOAD-QUICK`；性质 feature。历史来源 grouping/context：B。

CAP-04-ANA-06 | 载入成功后自动快析的触发/默认/保存/取消及Run ownership；ANA-16显式任务不证明自动触发；依据冻结Java入口/字段和新增设置映射，失败不隐式启动/切换引擎。

<a id="t02-tracking"></a>
### T02-TRACKING

当前公开合同：[R13 / tracking](R13_PLAN.md#tracking)。

唯一责任意图 `T02-TRACKING`；性质 feature。历史来源 grouping/context：B。

CAP-04-ANA-10 | 保留跟踪点集合、间隔/目标visits、显示身份及clear后恢复普通分析用户目标；独立于hover与readboard。先冻结单一Run/job并发合同，再用真实引擎验证取消/过期发布隔离。

<a id="t02-granular-display"></a>
### T02-GRANULAR-DISPLAY

当前公开合同：[R13 / display](R13_PLAN.md#display)。

唯一责任意图 `T02-GRANULAR-DISPLAY`；性质 feature。历史来源 grouping/context：B。

SET-SUGGESTION-INFO, SET-KATA-DISPLAY, CAP-04-ANA-11 | 逐字段保留候选十秒延迟（默认关）、黑/白过滤、PV-visits、分支长度、WR/visits/score显示、order/max-red和领地估算样式/位置目标；该候选延迟不是hover120ms。ADR0002主盘热图约束不变；不预设新的长度范围。

<a id="t02-custom-grade"></a>
### T02-CUSTOM-GRADE

当前公开合同：[R13 / display](R13_PLAN.md#display)。

唯一责任意图 `T02-CUSTOM-GRADE`；性质 decision。历史来源 grouping/context：B。

SET-NEXT-MOVE, SET-WINRATE-GRAPH | 有界来源/产品决策：查冻结Java的自定义评级入口、阈值字段、默认值、保存及消费；区分Next既有六级固定grade。交付具名来源与可观察阈值/边界合同后，才让自定义评级功能票就绪；找不到来源也不能捏造现有Java实现或删除spec目标。

<a id="t02-n-entry"></a>
### T02-N-ENTRY

当前公开合同：[R11 / behavior-a18](R11_PLAN.md#behavior-a18)。

唯一责任意图 `T02-N-ENTRY`；性质 decision。历史来源 grouping/context：A。

SHELL-11, SGF-03-ADJ-NEW, GM-HUMAN-GENMOVE | 修正旧N/新建映射与陈旧未接入入口：保留Ctrl+Home新建、N当前不New；GAME-02已经Accepted且真实人机新局/继续菜单已接通。统一registry/菜单/reference的真实可用行为，任何新N产品语义先明确决定；不新增Ctrl+N。

<a id="t02-java-import"></a>
### T02-JAVA-IMPORT

当前公开合同：[R17 / import](R17_PLAN.md#import)。

唯一责任意图 `T02-JAVA-IMPORT`；性质 feature。历史来源 grouping/context：F。

SET-PERSIST-CONFIG, REL-C02 | 只读Java源配置，白名单字段映射/预览/确认/原子应用；未知字段可见跳过；失败不部分应用；不修改源、不自动启动引擎。never imported是旧当前实现，不是永久禁止。

<a id="t02-i18n-foundation"></a>
### T02-I18N-FOUNDATION

当前公开合同：[R11 / behavior-a01](R11_PLAN.md#behavior-a01)。

唯一责任意图 `T02-I18N-FOUNDATION`；性质 feature。历史来源 grouping/context：A（首个新增UI内交付）。

SET-LANG | 首个新增UI消费者先建资源键/locale/fallback基础，不能继续硬编码后以F补救；字段映射随设置交付。I18N-01完整各支持语言、持久化与无混合语言仍由F验收。

<a id="t02-clock-decision"></a>
### T02-CLOCK-DECISION

当前公开合同：[R15 / human-clock](R15_PLAN.md#human-clock)。

唯一责任意图 `T02-CLOCK-DECISION`；性质 decision。历史来源 grouping/context：D。

GM-MATCH-RULES-START | GAME-07功能票阻塞于单独批准的clock/auto-resign政策：计时制、暂停/超时、adapter映射、保存恢复和阈值证据；不擅自冻结Fischer/读秒等新组合。

<a id="t02-tencent-protocol"></a>
### T02-TENCENT-PROTOCOL

当前公开合同：[R14 / tencent](R14_PLAN.md#tencent)。

唯一责任意图 `T02-TENCENT-PROTOCOL`；性质 investigation/decision。历史来源 grouping/context：C。

CAP-06-ONLINE-URL | 有界核实Tencent/huanle受支持实时读协议、URL族与可取得真实服务证据；给出协议/授权/错误边界后PROV-05方可实施，不因名字预定WebSocket。

<a id="t02-yike-auth"></a>
### T02-YIKE-AUTH

当前公开合同：[R14 / yike](R14_PLAN.md#yike)。

唯一责任意图 `T02-YIKE-AUTH`；性质 investigation/decision。历史来源 grouping/context：C。

CAP-06-YIKE-LIVE-CENTER | 有界获取provider-supported授权与读写协议、准入locator家族和测试账号能力；禁止抓取浏览器凭据替代；PROV-07依赖该证据及ADR0005。

<a id="t02-personal-evidence"></a>
### T02-PERSONAL-EVIDENCE

当前公开合同：[R14 / yike](R14_PLAN.md#yike)。

唯一责任意图 `T02-PERSONAL-EVIDENCE`；性质 investigation/decision。历史来源 grouping/context：C。

CAP-06-YIKE-LIVE-CENTER | 取得guest Personal分类真实pagination/filter/outcome语义；如实际需要认证，PROV-06按原门保持Deferred并显式依赖授权工作，不伪造guest等价。

<a id="t02-adapter-evidence"></a>
### T02-ADAPTER-EVIDENCE

当前公开合同：[R13 / adapters](R13_PLAN.md#adapters)。

唯一责任意图 `T02-ADAPTER-EVIDENCE`；性质 investigation/decision。历史来源 grouping/context：B。

CAP-04-ENG-01 | ANA-09先具名engine/version产品证据及协议fixture，不以空接口拉起；missingfields unavailable；SSH后续adapters另补兼容证据。

<a id="t02-guidance-producer"></a>
### T02-GUIDANCE-PRODUCER

当前公开合同：[R16 / guidance](R16_PLAN.md#guidance)。

唯一责任意图 `T02-GUIDANCE-PRODUCER`；性质 investigation/decision。历史来源 grouping/context：E。

SET-HINT-AUTOANALYZE, SET-RESET-HINTS | 具名准入教育提示producer（现ANA-16已Accepted但不自动准入提示），保存dismissal及专用reset仅作用已准入tips；安全确认不可dismiss，不交空reset。

<a id="t02-entry-closeout"></a>
### T02-ENTRY-CLOSEOUT

当前公开合同：[R17 / entries](R17_PLAN.md#entries)。

唯一责任意图 `T02-ENTRY-CLOSEOUT`；性质 feature。历史来源 grouping/context：F。

全部139条及20路由 | 逐入口、默认、持久化、取消/失败与字段映射最后核对；动态无证据仍缺口；不把源Java缺陷复制为Next行为，也不重写冻结事实。

<a id="t02-continuation"></a>
### T02-CONTINUATION

当前公开合同：[R13 / continuation](R13_PLAN.md#continuation)。

唯一责任意图 `T02-CONTINUATION`；性质 decision。历史来源 grouping/context：B（先决定，涉及Match权威）。

SGF-03-ADJ-AUTOPLAY / Ticket28 | 两个Engine Continuation目标（leaf-only/every-beat）具名保留；先交付用户语义/单一Run及Match/current-game写入权威、停止/失败边界和逐项批准，功能票显式受阻塞；不可直接沿用历史排除，也不可把它实现成Review Autoplay/Variation Replay。

<a id="t02-contribution-cuts"></a>
### T02-CONTRIBUTION-CUTS

当前公开合同：[R16 / contribution](R16_PLAN.md#contribution)。

唯一责任意图 `T02-CONTRIBUTION-CUTS`；性质 decision。历史来源 grouping/context：E。

输入GM-CONTRIBUTE和固定CONTRIB-01/GAME-09全文、Ticket24；输出每个上述历史缩减的原决定/批准来源、保留用户目标与后继合同或逐项批准非等价处置。GAME-09本身依赖GAME-01及通过的Contribution Run功能，不依赖CONTRIB-01全项Accepted。

<a id="t03-rel09-diag-bundle-export"></a>
### T03-REL09-DIAG-BUNDLE-EXPORT

当前公开合同：[R12 / diagnostics](R12_PLAN.md#diagnostics)。

唯一责任意图 `T01-DIAGNOSTICS`；性质 feature。历史来源 grouping/context：B（首个Run/资源消费者）；C/D/E各消费者扩展；R11仅安装支持包。

以UD-03-001的“实际阻塞与未来证据门”及“功能断言/调查停止条件”为完整合同；该意图拥有结果。

完整行为/非目标/实际阻塞/证据门：[UD-03-001](UPSTREAM_DELTA.md#ud-03-001).

<a id="t03-rel09-thread-snapshot"></a>
### T03-REL09-THREAD-SNAPSHOT

当前公开合同：[R12 / diagnostics](R12_PLAN.md#diagnostics)。

唯一责任意图 `T01-DIAGNOSTICS`；性质 feature。历史来源 grouping/context：B（首个Run/资源消费者）；C/D/E各消费者扩展；R11仅安装支持包。

以UD-03-002的“实际阻塞与未来证据门”及“功能断言/调查停止条件”为完整合同；该意图拥有结果。

完整行为/非目标/实际阻塞/证据门：[UD-03-002](UPSTREAM_DELTA.md#ud-03-002).

<a id="t03-eng01-bootstrap-diagnostics"></a>
### T03-ENG01-BOOTSTRAP-DIAGNOSTICS

当前公开合同：[R12 / diagnostics](R12_PLAN.md#diagnostics)。

唯一责任意图 `T01-DIAGNOSTICS`；性质 feature。历史来源 grouping/context：B（首个Run/资源消费者）；C/D/E各消费者扩展；R11仅安装支持包。

以UD-03-003的“实际阻塞与未来证据门”及“功能断言/调查停止条件”为完整合同；该意图拥有结果。

完整行为/非目标/实际阻塞/证据门：[UD-03-003](UPSTREAM_DELTA.md#ud-03-003).

<a id="t03-rel09-runtime-snapshot"></a>
### T03-REL09-RUNTIME-SNAPSHOT

当前公开合同：[R12 / diagnostics](R12_PLAN.md#diagnostics)。

唯一责任意图 `T01-DIAGNOSTICS`；性质 feature。历史来源 grouping/context：B（首个Run/资源消费者）；C/D/E各消费者扩展；R11仅安装支持包。

以UD-03-004的“实际阻塞与未来证据门”及“功能断言/调查停止条件”为完整合同；该意图拥有结果。

完整行为/非目标/实际阻塞/证据门：[UD-03-004](UPSTREAM_DELTA.md#ud-03-004).

<a id="t03-eng01-gtp-probe-stderr"></a>
### T03-ENG01-GTP-PROBE-STDERR

当前公开合同：[R12 / diagnostics](R12_PLAN.md#diagnostics)。

唯一责任意图 `T01-DIAGNOSTICS`；性质 feature。历史来源 grouping/context：B（首个Run/资源消费者）；C/D/E各消费者扩展；R11仅安装支持包。

以UD-03-005的“实际阻塞与未来证据门”及“功能断言/调查停止条件”为完整合同；该意图拥有结果。

完整行为/非目标/实际阻塞/证据门：[UD-03-005](UPSTREAM_DELTA.md#ud-03-005).

<a id="t03-resource-maintenance-lifecycle"></a>
### T03-RESOURCE-MAINTENANCE-LIFECYCLE

当前公开合同：[R12 / resources](R12_PLAN.md#resources)。

唯一责任意图 `T01-RESOURCE`；性质 feature。历史来源 grouping/context：B首个资源消费者；跨批共享合同。

以UD-03-006的“实际阻塞与未来证据门”及“功能断言/调查停止条件”为完整合同；该意图拥有结果。

完整行为/非目标/实际阻塞/证据门：[UD-03-006](UPSTREAM_DELTA.md#ud-03-006).

<a id="t03-game08-humansl-safety"></a>
### T03-GAME08-HUMANSL-SAFETY

当前公开合同：[R15 / humansl](R15_PLAN.md#humansl)。

唯一责任意图 `T02-GAME-08`；性质 feature。历史来源 grouping/context：D。

以UD-03-007的“实际阻塞与未来证据门”及“功能断言/调查停止条件”为完整合同；该意图拥有结果。

完整行为/非目标/实际阻塞/证据门：[UD-03-007](UPSTREAM_DELTA.md#ud-03-007).

<a id="t03-resource-trt-repair-investigation"></a>
### T03-RESOURCE-TRT-REPAIR-INVESTIGATION

当前公开合同：[R12 / resources](R12_PLAN.md#resources)。

唯一责任意图 `T03-RESOURCE-TRT-REPAIR-INVESTIGATION`；性质 investigation。历史来源 grouping/context：B。

以UD-03-008的“实际阻塞与未来证据门”及“功能断言/调查停止条件”为完整合同；该意图拥有结果。结论前不派实现。

完整行为/非目标/实际阻塞/证据门：[UD-03-008](UPSTREAM_DELTA.md#ud-03-008).

<a id="t03-game08-restore-foreground"></a>
### T03-GAME08-RESTORE-FOREGROUND

当前公开合同：[R15 / humansl](R15_PLAN.md#humansl)。

唯一责任意图 `T02-GAME-08`；性质 feature。历史来源 grouping/context：D。

以UD-03-009的“实际阻塞与未来证据门”及“功能断言/调查停止条件”为完整合同；该意图拥有结果。

完整行为/非目标/实际阻塞/证据门：[UD-03-009](UPSTREAM_DELTA.md#ud-03-009).

<a id="t03-resource-nvidia-hardware-gate"></a>
### T03-RESOURCE-NVIDIA-HARDWARE-GATE

当前公开合同：[R12 / resources](R12_PLAN.md#resources)。

唯一责任意图 `T03-RESOURCE-NVIDIA-HARDWARE-GATE`；性质 investigation。历史来源 grouping/context：B。

以UD-03-010的“实际阻塞与未来证据门”及“功能断言/调查停止条件”为完整合同；该意图拥有结果。结论前不派实现。

完整行为/非目标/实际阻塞/证据门：[UD-03-010](UPSTREAM_DELTA.md#ud-03-010).

<a id="t03-lifecycle-windows-applicability"></a>
### T03-LIFECYCLE-WINDOWS-APPLICABILITY

当前公开合同：[R12 / lifecycle](R12_PLAN.md#lifecycle)。

唯一责任意图 `T03-LIFECYCLE-WINDOWS-APPLICABILITY`；性质 investigation。历史来源 grouping/context：B。

以UD-03-011的“实际阻塞与未来证据门”及“功能断言/调查停止条件”为完整合同；该意图拥有结果。结论前不派实现。

完整行为/非目标/实际阻塞/证据门：[UD-03-011](UPSTREAM_DELTA.md#ud-03-011).

<a id="t03-game-ai-commentary-workspace"></a>
### T03-GAME-AI-COMMENTARY-WORKSPACE

当前公开合同：[R16 / ai](R16_PLAN.md#ai)。

唯一责任意图 `T05-AI-GROUNDED-TEACHING`；性质 feature。历史来源 grouping/context：E。

以UD-03-012的“实际阻塞与未来证据门”及“功能断言/调查停止条件”为完整合同；该意图拥有结果。

完整行为/非目标/实际阻塞/证据门：[UD-03-012](UPSTREAM_DELTA.md#ud-03-012).

<a id="t03-ana11-score-lead-display"></a>
### T03-ANA11-SCORE-LEAD-DISPLAY

当前公开合同：[R11 / behavior-a19](R11_PLAN.md#behavior-a19)。

唯一责任意图 `T03-ANA11-SCORE-LEAD-DISPLAY`；性质 feature。历史来源 grouping/context：A。

以UD-03-013的“实际阻塞与未来证据门”及“功能断言/调查停止条件”为完整合同；该意图拥有结果。

完整行为/非目标/实际阻塞/证据门：[UD-03-013](UPSTREAM_DELTA.md#ud-03-013).

<a id="t03-sgf07-rapid-switch-transaction"></a>
### T03-SGF07-RAPID-SWITCH-TRANSACTION

当前公开合同：[R11 / behavior-a20](R11_PLAN.md#behavior-a20)。

唯一责任意图 `T03-SGF07-RAPID-SWITCH-TRANSACTION`；性质 investigation。历史来源 grouping/context：A/B（数据安全A，快析B）。

以UD-03-014的“实际阻塞与未来证据门”及“功能断言/调查停止条件”为完整合同；该意图拥有结果。结论前不派实现。

完整行为/非目标/实际阻塞/证据门：[UD-03-014](UPSTREAM_DELTA.md#ud-03-014).

<a id="t03-performance-speed-metrics"></a>
### T03-PERFORMANCE-SPEED-METRICS

当前公开合同：[R13 / performance](R13_PLAN.md#performance)。

唯一责任意图 `T03-PERFORMANCE-SPEED-METRICS`；性质 feature。历史来源 grouping/context：B。

以UD-03-015的“实际阻塞与未来证据门”及“功能断言/调查停止条件”为完整合同；该意图拥有结果。

完整行为/非目标/实际阻塞/证据门：[UD-03-015](UPSTREAM_DELTA.md#ud-03-015).

<a id="t03-rcomp-skip-local-benchmark"></a>
### T03-RCOMP-SKIP-LOCAL-BENCHMARK

当前公开合同：[R14 / compute-network](R14_PLAN.md#compute-network)。

唯一责任意图 `T01-REMOTE`；性质 feature。历史来源 grouping/context：C。

以UD-03-016的“实际阻塞与未来证据门”及“功能断言/调查停止条件”为完整合同；该意图拥有结果。

完整行为/非目标/实际阻塞/证据门：[UD-03-016](UPSTREAM_DELTA.md#ud-03-016).

<a id="t03-performance-a11y-benchmark"></a>
### T03-PERFORMANCE-A11Y-BENCHMARK

当前公开合同：[R13 / performance](R13_PLAN.md#performance)。

唯一责任意图 `T03-PERFORMANCE-A11Y-BENCHMARK`；性质 feature。历史来源 grouping/context：B（F完整语言/无障碍收尾）。

以UD-03-017的“实际阻塞与未来证据门”及“功能断言/调查停止条件”为完整合同；该意图拥有结果。

完整行为/非目标/实际阻塞/证据门：[UD-03-017](UPSTREAM_DELTA.md#ud-03-017).

<a id="t03-lifecycle-concurrency-investigation"></a>
### T03-LIFECYCLE-CONCURRENCY-INVESTIGATION

当前公开合同：[R12 / lifecycle](R12_PLAN.md#lifecycle)。

唯一责任意图 `T03-LIFECYCLE-CONCURRENCY-INVESTIGATION`；性质 investigation。历史来源 grouping/context：B（当前路径风险优先）。

以UD-03-018的“实际阻塞与未来证据门”及“功能断言/调查停止条件”为完整合同；该意图拥有结果。结论前不派实现。

完整行为/非目标/实际阻塞/证据门：[UD-03-018](UPSTREAM_DELTA.md#ud-03-018).

<a id="t03-rel09-logging-generation"></a>
### T03-REL09-LOGGING-GENERATION

当前公开合同：[R12 / diagnostics](R12_PLAN.md#diagnostics)。

唯一责任意图 `T01-DIAGNOSTICS`；性质 feature。历史来源 grouping/context：B（首个Run/资源消费者）；C/D/E各消费者扩展；R11仅安装支持包。

以UD-03-019的“实际阻塞与未来证据门”及“功能断言/调查停止条件”为完整合同；该意图拥有结果。

完整行为/非目标/实际阻塞/证据门：[UD-03-019](UPSTREAM_DELTA.md#ud-03-019).

<a id="t03-ana06-live-limits-inheritance"></a>
### T03-ANA06-LIVE-LIMITS-INHERITANCE

当前公开合同：证据继承/排除；无新功能责任（见 [routes evidence_only](MIGRATION_ROUTES.json)）。

唯一责任意图 `T03-ANA06-LIVE-LIMITS-INHERITANCE`；性质 inheritance-or-exclusion。历史来源 grouping/context：B（仅继承，无新feature）。

以UD-03-020的“实际阻塞与未来证据门”及“功能断言/调查停止条件”为完整合同；该意图拥有结果。仅原范围继承，不派新feature。

完整行为/非目标/实际阻塞/证据门：[UD-03-020](UPSTREAM_DELTA.md#ud-03-020).

<a id="t03-ana11-unanalyzed-scrubbing"></a>
### T03-ANA11-UNANALYZED-SCRUBBING

当前公开合同：[R11 / behavior-a21](R11_PLAN.md#behavior-a21)。

唯一责任意图 `T03-ANA11-UNANALYZED-SCRUBBING`；性质 investigation。历史来源 grouping/context：A。

以UD-03-021的“实际阻塞与未来证据门”及“功能断言/调查停止条件”为完整合同；该意图拥有结果。结论前不派实现。

完整行为/非目标/实际阻塞/证据门：[UD-03-021](UPSTREAM_DELTA.md#ud-03-021).

<a id="t03-autoload-pause-boundaries"></a>
### T03-AUTOLOAD-PAUSE-BOUNDARIES

当前公开合同：[R13 / autoquick](R13_PLAN.md#autoquick)。

唯一责任意图 `T02-AUTOLOAD-QUICK`；性质 feature。历史来源 grouping/context：B。

以UD-03-022的“实际阻塞与未来证据门”及“功能断言/调查停止条件”为完整合同；该意图拥有结果。

完整行为/非目标/实际阻塞/证据门：[UD-03-022](UPSTREAM_DELTA.md#ud-03-022).

<a id="t03-perf-background-offload"></a>
### T03-PERF-BACKGROUND-OFFLOAD

当前公开合同：[R11 / behavior-a22](R11_PLAN.md#behavior-a22)。

唯一责任意图 `T03-PERF-BACKGROUND-OFFLOAD`；性质 investigation。历史来源 grouping/context：A/B/C 按可达消费者。

以UD-03-023的“实际阻塞与未来证据门”及“功能断言/调查停止条件”为完整合同；该意图拥有结果。结论前不派实现。

完整行为/非目标/实际阻塞/证据门：[UD-03-023](UPSTREAM_DELTA.md#ud-03-023).

<a id="t03-ana16-whole-game-limits"></a>
### T03-ANA16-WHOLE-GAME-LIMITS

当前公开合同：[R13 / autoquick](R13_PLAN.md#autoquick)。

唯一责任意图 `T03-ANA16-WHOLE-GAME-LIMITS`；性质 investigation。历史来源 grouping/context：B。

以UD-03-024的“实际阻塞与未来证据门”及“功能断言/调查停止条件”为完整合同；该意图拥有结果。结论前不派实现。

完整行为/非目标/实际阻塞/证据门：[UD-03-024](UPSTREAM_DELTA.md#ud-03-024).

<a id="t03-rel09-analysis-cache-trace"></a>
### T03-REL09-ANALYSIS-CACHE-TRACE

当前公开合同：[R12 / diagnostics](R12_PLAN.md#diagnostics)。

唯一责任意图 `T01-DIAGNOSTICS`；性质 feature。历史来源 grouping/context：B（首个Run/资源消费者）；C/D/E各消费者扩展；R11仅安装支持包。

以UD-03-025的“实际阻塞与未来证据门”及“功能断言/调查停止条件”为完整合同；该意图拥有结果。

完整行为/非目标/实际阻塞/证据门：[UD-03-025](UPSTREAM_DELTA.md#ud-03-025).

<a id="t03-game01-rules-dialog-guard"></a>
### T03-GAME01-RULES-DIALOG-GUARD

当前公开合同：证据继承/排除；无新功能责任（见 [routes evidence_only](MIGRATION_ROUTES.json)）。

唯一责任意图 `T03-GAME01-RULES-DIALOG-GUARD`；性质 inheritance-or-exclusion。历史来源 grouping/context：D（仅现有守卫继承）。

以UD-03-026的“实际阻塞与未来证据门”及“功能断言/调查停止条件”为完整合同；该意图拥有结果。仅原范围继承，不派新feature。

完整行为/非目标/实际阻塞/证据门：[UD-03-026](UPSTREAM_DELTA.md#ud-03-026).

<a id="t03-match-genmove-retirement-investigation"></a>
### T03-MATCH-GENMOVE-RETIREMENT-INVESTIGATION

当前公开合同：[R12 / lifecycle](R12_PLAN.md#lifecycle)。

唯一责任意图 `T03-MATCH-GENMOVE-RETIREMENT-INVESTIGATION`；性质 investigation。历史来源 grouping/context：B/D（当前Match风险优先）。

以UD-03-027的“实际阻塞与未来证据门”及“功能断言/调查停止条件”为完整合同；该意图拥有结果。结论前不派实现。

完整行为/非目标/实际阻塞/证据门：[UD-03-027](UPSTREAM_DELTA.md#ud-03-027).

<a id="t03-performance-custom-benchmark-runner"></a>
### T03-PERFORMANCE-CUSTOM-BENCHMARK-RUNNER

当前公开合同：[R13 / performance](R13_PLAN.md#performance)。

唯一责任意图 `T03-PERFORMANCE-CUSTOM-BENCHMARK-RUNNER`；性质 feature。历史来源 grouping/context：B。

以UD-03-028的“实际阻塞与未来证据门”及“功能断言/调查停止条件”为完整合同；该意图拥有结果。

完整行为/非目标/实际阻塞/证据门：[UD-03-028](UPSTREAM_DELTA.md#ud-03-028).

<a id="t03-match-rules-lifecycle"></a>
### T03-MATCH-RULES-LIFECYCLE

当前公开合同：[R12 / lifecycle](R12_PLAN.md#lifecycle)。

唯一责任意图 `T03-MATCH-RULES-LIFECYCLE`；性质 feature。历史来源 grouping/context：B/D（普通确认B；Match D）。

以UD-03-029的“实际阻塞与未来证据门”及“功能断言/调查停止条件”为完整合同；该意图拥有结果。

完整行为/非目标/实际阻塞/证据门：[UD-03-029](UPSTREAM_DELTA.md#ud-03-029).

<a id="t03-match-rules-storage-decision"></a>
### T03-MATCH-RULES-STORAGE-DECISION

当前公开合同：[R15 / rule-storage](R15_PLAN.md#rule-storage)。

唯一责任意图 `T03-MATCH-RULES-STORAGE-DECISION`；性质 decision。历史来源 grouping/context：D。

UD-03-029具名决定拥有generated details保存/共享表示、字段归属、兼容读取与个人C保护合同；适用批准后只开放save残余，不阻塞规则准入/恢复/只读详情。

完整行为/非目标/实际阻塞/证据门：[UD-03-029](UPSTREAM_DELTA.md#ud-03-029).

<a id="t03-rel09-sanitizer-boundaries"></a>
### T03-REL09-SANITIZER-BOUNDARIES

当前公开合同：[R12 / diagnostics](R12_PLAN.md#diagnostics)。

唯一责任意图 `T01-DIAGNOSTICS`；性质 feature。历史来源 grouping/context：B（首个Run/资源消费者）；C/D/E各消费者扩展；R11仅安装支持包。

以UD-03-030的“实际阻塞与未来证据门”及“功能断言/调查停止条件”为完整合同；该意图拥有结果。

完整行为/非目标/实际阻塞/证据门：[UD-03-030](UPSTREAM_DELTA.md#ud-03-030).

<a id="t03-rel09-export-bounded-io"></a>
### T03-REL09-EXPORT-BOUNDED-IO

当前公开合同：[R12 / diagnostics](R12_PLAN.md#diagnostics)。

唯一责任意图 `T01-DIAGNOSTICS`；性质 feature。历史来源 grouping/context：B（首个Run/资源消费者）；C/D/E各消费者扩展；R11仅安装支持包。

以UD-03-031的“实际阻塞与未来证据门”及“功能断言/调查停止条件”为完整合同；该意图拥有结果。

完整行为/非目标/实际阻塞/证据门：[UD-03-031](UPSTREAM_DELTA.md#ud-03-031).

<a id="t03-analysis-confirmed-position-investigation"></a>
### T03-ANALYSIS-CONFIRMED-POSITION-INVESTIGATION

当前公开合同：[R13 / focus](R13_PLAN.md#focus)。

唯一责任意图 `T03-ANALYSIS-CONFIRMED-POSITION-INVESTIGATION`；性质 investigation。历史来源 grouping/context：B/C。

以UD-03-032的“实际阻塞与未来证据门”及“功能断言/调查停止条件”为完整合同；该意图拥有结果。结论前不派实现。

完整行为/非目标/实际阻塞/证据门：[UD-03-032](UPSTREAM_DELTA.md#ud-03-032).

<a id="t03-rcomp01-setup-guidance"></a>
### T03-RCOMP01-SETUP-GUIDANCE

当前公开合同：[R14 / compute-network](R14_PLAN.md#compute-network)。

唯一责任意图 `T01-REMOTE`；性质 feature。历史来源 grouping/context：C。

以UD-03-033的“实际阻塞与未来证据门”及“功能断言/调查停止条件”为完整合同；该意图拥有结果。

完整行为/非目标/实际阻塞/证据门：[UD-03-033](UPSTREAM_DELTA.md#ud-03-033).

<a id="t03-read02-stop-and-game10-turn-retirement"></a>
### T03-READ02-STOP-AND-GAME10-TURN-RETIREMENT

当前公开合同：[R14 / external-match](R14_PLAN.md#external-match)。

唯一责任意图 `T03-READ02-STOP-AND-GAME10-TURN-RETIREMENT`；性质 feature。历史来源 grouping/context：C。

以UD-03-034的“实际阻塞与未来证据门”及“功能断言/调查停止条件”为完整合同；该意图拥有结果。

完整行为/非目标/实际阻塞/证据门：[UD-03-034](UPSTREAM_DELTA.md#ud-03-034).

<a id="t03-rel05-humansl-r2-download"></a>
### T03-REL05-HUMANSL-R2-DOWNLOAD

当前公开合同：[R15 / humansl](R15_PLAN.md#humansl)。

唯一责任意图 `T02-GAME-08`；性质 feature。历史来源 grouping/context：D。

以UD-03-035的“实际阻塞与未来证据门”及“功能断言/调查停止条件”为完整合同；该意图拥有结果。

完整行为/非目标/实际阻塞/证据门：[UD-03-035](UPSTREAM_DELTA.md#ud-03-035).

<a id="t03-thread-saved-entry-policy"></a>
### T03-THREAD-SAVED-ENTRY-POLICY

当前公开合同：[R13 / performance](R13_PLAN.md#performance)。

唯一责任意图 `T03-THREAD-SAVED-ENTRY-POLICY`；性质 feature。历史来源 grouping/context：B。

以UD-03-036的“实际阻塞与未来证据门”及“功能断言/调查停止条件”为完整合同；该意图拥有结果。

完整行为/非目标/实际阻塞/证据门：[UD-03-036](UPSTREAM_DELTA.md#ud-03-036).

<a id="t03-rel05-b11-model-update"></a>
### T03-REL05-B11-MODEL-UPDATE

当前公开合同：[R12 / resources](R12_PLAN.md#resources)。

唯一责任意图 `T01-RESOURCE`；性质 feature。历史来源 grouping/context：B首个资源消费者；跨批共享合同。

以UD-03-037的“实际阻塞与未来证据门”及“功能断言/调查停止条件”为完整合同；该意图拥有结果。

完整行为/非目标/实际阻塞/证据门：[UD-03-037](UPSTREAM_DELTA.md#ud-03-037).

<a id="t03-analysis-sgf-rules-sync-investigation"></a>
### T03-ANALYSIS-SGF-RULES-SYNC-INVESTIGATION

当前公开合同：[R13 / focus](R13_PLAN.md#focus)。

唯一责任意图 `T03-ANALYSIS-SGF-RULES-SYNC-INVESTIGATION`；性质 investigation。历史来源 grouping/context：B。

以UD-03-038的“实际阻塞与未来证据门”及“功能断言/调查停止条件”为完整合同；该意图拥有结果。结论前不派实现。

完整行为/非目标/实际阻塞/证据门：[UD-03-038](UPSTREAM_DELTA.md#ud-03-038).

<a id="t03-resource-accel-layout"></a>
### T03-RESOURCE-ACCEL-LAYOUT

当前公开合同：[R12 / resources](R12_PLAN.md#resources)。

唯一责任意图 `T03-RESOURCE-ACCEL-LAYOUT`；性质 feature。历史来源 grouping/context：B。

以UD-03-039的“实际阻塞与未来证据门”及“功能断言/调查停止条件”为完整合同；该意图拥有结果。

完整行为/非目标/实际阻塞/证据门：[UD-03-039](UPSTREAM_DELTA.md#ud-03-039).

<a id="t03-nav-global-function-search"></a>
### T03-NAV-GLOBAL-FUNCTION-SEARCH

当前公开合同：[R11 / behavior-a01](R11_PLAN.md#behavior-a01)。

唯一责任意图 `T03-NAV-GLOBAL-FUNCTION-SEARCH`；性质 feature。历史来源 grouping/context：A。

以UD-03-040的“实际阻塞与未来证据门”及“功能断言/调查停止条件”为完整合同；该意图拥有结果。

完整行为/非目标/实际阻塞/证据门：[UD-03-040](UPSTREAM_DELTA.md#ud-03-040).

<a id="t03-autoload-resume-framing-investigation"></a>
### T03-AUTOLOAD-RESUME-FRAMING-INVESTIGATION

当前公开合同：[R13 / autoquick](R13_PLAN.md#autoquick)。

唯一责任意图 `T03-AUTOLOAD-RESUME-FRAMING-INVESTIGATION`；性质 investigation。历史来源 grouping/context：B（当前parser/handoff风险优先）。

以UD-03-041的“实际阻塞与未来证据门”及“功能断言/调查停止条件”为完整合同；该意图拥有结果。结论前不派实现。

完整行为/非目标/实际阻塞/证据门：[UD-03-041](UPSTREAM_DELTA.md#ud-03-041).

<a id="t03-eng01-isolated-startup-failure"></a>
### T03-ENG01-ISOLATED-STARTUP-FAILURE

当前公开合同：[R12 / diagnostics](R12_PLAN.md#diagnostics)。

唯一责任意图 `T01-DIAGNOSTICS`；性质 feature。历史来源 grouping/context：B（首个Run/资源消费者）；C/D/E各消费者扩展；R11仅安装支持包。

以UD-03-042的“实际阻塞与未来证据门”及“功能断言/调查停止条件”为完整合同；该意图拥有结果。

完整行为/非目标/实际阻塞/证据门：[UD-03-042](UPSTREAM_DELTA.md#ud-03-042).

<a id="t03-ui01-toolbar-search-entry"></a>
### T03-UI01-TOOLBAR-SEARCH-ENTRY

当前公开合同：[R11 / behavior-a01](R11_PLAN.md#behavior-a01)。

唯一责任意图 `T03-NAV-GLOBAL-FUNCTION-SEARCH`；性质 feature。历史来源 grouping/context：A。

以UD-03-043的“实际阻塞与未来证据门”及“功能断言/调查停止条件”为完整合同；该意图拥有结果。

完整行为/非目标/实际阻塞/证据门：[UD-03-043](UPSTREAM_DELTA.md#ud-03-043).

<a id="t03-restore-gtp-path-investigation"></a>
### T03-RESTORE-GTP-PATH-INVESTIGATION

当前公开合同：[R12 / lifecycle](R12_PLAN.md#lifecycle)。

唯一责任意图 `T03-RESTORE-GTP-PATH-INVESTIGATION`；性质 investigation。历史来源 grouping/context：B/D。

以UD-03-044的“实际阻塞与未来证据门”及“功能断言/调查停止条件”为完整合同；该意图拥有结果。结论前不派实现。

完整行为/非目标/实际阻塞/证据门：[UD-03-044](UPSTREAM_DELTA.md#ud-03-044).

<a id="t03-lifecycle-target-cancel-run-stop"></a>
### T03-LIFECYCLE-TARGET-CANCEL-RUN-STOP

当前公开合同：[R12 / lifecycle](R12_PLAN.md#lifecycle)。

唯一责任意图 `T03-LIFECYCLE-TARGET-CANCEL-RUN-STOP`；性质 investigation。历史来源 grouping/context：B。

以UD-03-045的“实际阻塞与未来证据门”及“功能断言/调查停止条件”为完整合同；该意图拥有结果。结论前不派实现。

完整行为/非目标/实际阻塞/证据门：[UD-03-045](UPSTREAM_DELTA.md#ud-03-045).

<a id="t03-lifecycle-restart-fence-investigation"></a>
### T03-LIFECYCLE-RESTART-FENCE-INVESTIGATION

当前公开合同：[R12 / lifecycle](R12_PLAN.md#lifecycle)。

唯一责任意图 `T03-LIFECYCLE-RESTART-FENCE-INVESTIGATION`；性质 investigation。历史来源 grouping/context：B（当前restart风险优先）。

以UD-03-046的“实际阻塞与未来证据门”及“功能断言/调查停止条件”为完整合同；该意图拥有结果。结论前不派实现。

完整行为/非目标/实际阻塞/证据门：[UD-03-046](UPSTREAM_DELTA.md#ud-03-046).

<a id="t03-ui02-dialog-focus-nav"></a>
### T03-UI02-DIALOG-FOCUS-NAV

当前公开合同：[R11 / behavior-a01](R11_PLAN.md#behavior-a01)。

唯一责任意图 `T03-NAV-GLOBAL-FUNCTION-SEARCH`；性质 feature。历史来源 grouping/context：A。

以UD-03-047的“实际阻塞与未来证据门”及“功能断言/调查停止条件”为完整合同；该意图拥有结果。

完整行为/非目标/实际阻塞/证据门：[UD-03-047](UPSTREAM_DELTA.md#ud-03-047).

<a id="t03-console-output-responsiveness"></a>
### T03-CONSOLE-OUTPUT-RESPONSIVENESS

当前公开合同：[R12 / diagnostics](R12_PLAN.md#diagnostics)。

唯一责任意图 `T03-CONSOLE-OUTPUT-RESPONSIVENESS`；性质 feature。历史来源 grouping/context：B。

以UD-03-048的“实际阻塞与未来证据门”及“功能断言/调查停止条件”为完整合同；该意图拥有结果。

完整行为/非目标/实际阻塞/证据门：[UD-03-048](UPSTREAM_DELTA.md#ud-03-048).

<a id="t03-sgf13-gameinfo-komi-focus"></a>
### T03-SGF13-GAMEINFO-KOMI-FOCUS

当前公开合同：[R11 / behavior-a23](R11_PLAN.md#behavior-a23)。

唯一责任意图 `T03-SGF13-GAMEINFO-KOMI-FOCUS`；性质 feature。历史来源 grouping/context：A。

以UD-03-049的“实际阻塞与未来证据门”及“功能断言/调查停止条件”为完整合同；该意图拥有结果。

完整行为/非目标/实际阻塞/证据门：[UD-03-049](UPSTREAM_DELTA.md#ud-03-049).

<a id="t03-review08-audio-boundary-investigation"></a>
### T03-REVIEW08-AUDIO-BOUNDARY-INVESTIGATION

当前公开合同：[R11 / behavior-a24](R11_PLAN.md#behavior-a24)。

唯一责任意图 `T03-REVIEW08-AUDIO-BOUNDARY-INVESTIGATION`；性质 investigation。历史来源 grouping/context：A（现有review）；D如对局音效消费者。

以UD-03-050的“实际阻塞与未来证据门”及“功能断言/调查停止条件”为完整合同；该意图拥有结果。结论前不派实现。

完整行为/非目标/实际阻塞/证据门：[UD-03-050](UPSTREAM_DELTA.md#ud-03-050).

<a id="t03-ana16-execution-mode-indicator"></a>
### T03-ANA16-EXECUTION-MODE-INDICATOR

当前公开合同：[R13 / autoquick](R13_PLAN.md#autoquick)。

唯一责任意图 `T02-AUTOLOAD-QUICK`；性质 feature。历史来源 grouping/context：B。

以UD-03-051的“实际阻塞与未来证据门”及“功能断言/调查停止条件”为完整合同；该意图拥有结果。

完整行为/非目标/实际阻塞/证据门：[UD-03-051](UPSTREAM_DELTA.md#ud-03-051).

<a id="t03-rel09-shutdown-cleanup"></a>
### T03-REL09-SHUTDOWN-CLEANUP

当前公开合同：[R12 / diagnostics](R12_PLAN.md#diagnostics)。

唯一责任意图 `T01-DIAGNOSTICS`；性质 feature。历史来源 grouping/context：B（首个Run/资源消费者）；C/D/E各消费者扩展；R11仅安装支持包。

以UD-03-052的“实际阻塞与未来证据门”及“功能断言/调查停止条件”为完整合同；该意图拥有结果。

完整行为/非目标/实际阻塞/证据门：[UD-03-052](UPSTREAM_DELTA.md#ud-03-052).

<a id="t03-thread-alias-applicability"></a>
### T03-THREAD-ALIAS-APPLICABILITY

当前公开合同：[R12 / manual-threads](R12_PLAN.md#manual-threads)。

唯一责任意图 `T03-THREAD-ALIAS-APPLICABILITY`；性质 investigation。历史来源 grouping/context：B。

以UD-03-053的“实际阻塞与未来证据门”及“功能断言/调查停止条件”为完整合同；该意图拥有结果。结论前不派实现。

完整行为/非目标/实际阻塞/证据门：[UD-03-053](UPSTREAM_DELTA.md#ud-03-053).

<a id="t03-analysis-same-tree-move-focus"></a>
### T03-ANALYSIS-SAME-TREE-MOVE-FOCUS

当前公开合同：[R13 / focus](R13_PLAN.md#focus)。

唯一责任意图 `T03-ANALYSIS-SAME-TREE-MOVE-FOCUS`；性质 feature。历史来源 grouping/context：B（ReadBoard部分C）。

以UD-03-054的“实际阻塞与未来证据门”及“功能断言/调查停止条件”为完整合同；该意图拥有结果。

完整行为/非目标/实际阻塞/证据门：[UD-03-054](UPSTREAM_DELTA.md#ud-03-054).

<a id="t03-resource-public-gpu-guidance"></a>
### T03-RESOURCE-PUBLIC-GPU-GUIDANCE

当前公开合同：[R12 / resources](R12_PLAN.md#resources)。

唯一责任意图 `T01-RESOURCE`；性质 feature。历史来源 grouping/context：B首个资源消费者；跨批共享合同。

以UD-03-010a“实际阻塞”“有界可观察断言/stop”为完整合同；同父Delta去重，不分配新Parity ID。

完整行为/非目标/实际阻塞/证据门：[UD-03-010a](UPSTREAM_DELTA.md#ud-03-010a).

<a id="t03-resource-trusted-origin-identity"></a>
### T03-RESOURCE-TRUSTED-ORIGIN-IDENTITY

当前公开合同：[R12 / resources](R12_PLAN.md#resources)。

唯一责任意图 `T01-RESOURCE`；性质 feature。历史来源 grouping/context：B首个资源消费者；跨批共享合同。

以UD-03-035a“实际阻塞”“有界可观察断言/stop”为完整合同；同父Delta去重，不分配新Parity ID。

完整行为/非目标/实际阻塞/证据门：[UD-03-035a](UPSTREAM_DELTA.md#ud-03-035a).

<a id="t03-nav-search-cancel-focus"></a>
### T03-NAV-SEARCH-CANCEL-FOCUS

当前公开合同：[R11 / behavior-a01](R11_PLAN.md#behavior-a01)。

唯一责任意图 `T03-NAV-GLOBAL-FUNCTION-SEARCH`；性质 feature。历史来源 grouping/context：A。

以UD-03-040a“实际阻塞”“有界可观察断言/stop”为完整合同；同父Delta去重，不分配新Parity ID。

完整行为/非目标/实际阻塞/证据门：[UD-03-040a](UPSTREAM_DELTA.md#ud-03-040a).

<a id="t03-focus-admission"></a>
### T03-FOCUS-ADMISSION

当前公开合同：[R13 / focus](R13_PLAN.md#focus)。

唯一责任意图 `T03-ANALYSIS-SAME-TREE-MOVE-FOCUS`；性质 feature。历史来源 grouping/context：B（ReadBoard部分C）。

以UD-03-054a“实际阻塞”“有界可观察断言/stop”为完整合同；同父Delta去重，不分配新Parity ID。

完整行为/非目标/实际阻塞/证据门：[UD-03-054a](UPSTREAM_DELTA.md#ud-03-054a).

<a id="t03-focus-user-set-progress"></a>
### T03-FOCUS-USER-SET-PROGRESS

当前公开合同：[R13 / focus](R13_PLAN.md#focus)。

唯一责任意图 `T03-ANALYSIS-SAME-TREE-MOVE-FOCUS`；性质 feature。历史来源 grouping/context：B（ReadBoard部分C）。

以UD-03-054b“实际阻塞”“有界可观察断言/stop”为完整合同；同父Delta去重，不分配新Parity ID。

完整行为/非目标/实际阻塞/证据门：[UD-03-054b](UPSTREAM_DELTA.md#ud-03-054b).

<a id="t03-focus-tree-cache-sgf"></a>
### T03-FOCUS-TREE-CACHE-SGF

当前公开合同：[R13 / focus](R13_PLAN.md#focus)。

唯一责任意图 `T03-ANALYSIS-SAME-TREE-MOVE-FOCUS`；性质 feature。历史来源 grouping/context：B（ReadBoard部分C）。

以UD-03-054c“实际阻塞”“有界可观察断言/stop”为完整合同；同父Delta去重，不分配新Parity ID。

完整行为/非目标/实际阻塞/证据门：[UD-03-054c](UPSTREAM_DELTA.md#ud-03-054c).

<a id="t03-focus-readboard"></a>
### T03-FOCUS-READBOARD

当前公开合同：[R14 / readboard](R14_PLAN.md#readboard)。

唯一责任意图 `T03-FOCUS-READBOARD`；性质 feature。历史来源 grouping/context：C。

以UD-03-054d“实际阻塞”“有界可观察断言/stop”为完整合同；同父Delta去重，不分配新Parity ID。

完整行为/非目标/实际阻塞/证据门：[UD-03-054d](UPSTREAM_DELTA.md#ud-03-054d).

<a id="t03-focus-display-settings"></a>
### T03-FOCUS-DISPLAY-SETTINGS

当前公开合同：[R13 / focus](R13_PLAN.md#focus)。

唯一责任意图 `T03-ANALYSIS-SAME-TREE-MOVE-FOCUS`；性质 feature。历史来源 grouping/context：B（ReadBoard部分C）。

以UD-03-054e“实际阻塞”“有界可观察断言/stop”为完整合同；同父Delta去重，不分配新Parity ID。

完整行为/非目标/实际阻塞/证据门：[UD-03-054e](UPSTREAM_DELTA.md#ud-03-054e).

<a id="t03-focus-compatible-engine-evidence"></a>
### T03-FOCUS-COMPATIBLE-ENGINE-EVIDENCE

当前公开合同：[R13 / focus](R13_PLAN.md#focus)。

唯一责任意图 `T03-ANALYSIS-SAME-TREE-MOVE-FOCUS`；性质 evidence-gate。历史来源 grouping/context：B（ReadBoard部分C）。

以UD-03-054f“实际阻塞”“有界可观察断言/stop”为完整合同；同父Delta去重，不分配新Parity ID。

完整行为/非目标/实际阻塞/证据门：[UD-03-054f](UPSTREAM_DELTA.md#ud-03-054f).

<a id="t04-resource"></a>
### T04-RESOURCE

当前公开合同：[R12 / resources](R12_PLAN.md#resources)。

唯一责任意图 `T01-RESOURCE`；性质 功能增量（复用T01-RESOURCE；不复制别名）。历史来源 grouping/context：B首个资源消费者；跨批共享合同。

- **kind**: 功能增量（复用T01-RESOURCE；不复制别名）
- **历史来源 grouping / 行为范围**: B；随受影响consumer提前；REL-05功能部分/ENG-01原profile边界；签名安装仍T01-RELEASE。
- **sourceDelta**: UD-04-001、UD-04-002、UD-04-005、UD-04-006、UD-04-008、UD-04-010、UD-04-014、UD-04-016、UD-04-018、UD-04-021、UD-04-024、UD-04-026、UD-04-028、UD-04-035、UD-04-037、UD-04-038、UD-04-043。
- **actual blockers**: 需要源/版本/路径/digest及兼容adapter能力的明确准入合同；可用合格本地资源。具体managed catalog/受控修复产品界面须先冻结；无生产Release/上游编译/签名开工前置。
- **bounded observable assertions**:
1. 原profile可继续用本地资源，不自动获取或切profile。目录资源按origin、source commit、backend、archive/executable与model/config身份验证；schema2的verified managed static-zlib例外只对catalog及manifest精确匹配且未修改的可执行文件适用。external/official/unknown不能借receipt去掉自身dynamic DLL合同，modified拒绝trusted-ready。
2. 15目标来源可枚举但按实际准入平台/硬件获取真实能力，不用CPU/fixture替GPU认证；源build、cloud transfer、JRE/installer实现不迁移。新版engineReleaseTag只改变来源联接，不证明已发布/URL可下载。离线/缺失/不兼容失败可见，既存current-game/ready A保留，autoload失败不fallback。
3. TensorRT repair只由明确用户动作触发，重新核对原选择目标/资源identity，不静默转CUDA/OpenCL；GPU提示不冒称真实GPU认证。保存/选择/刷新不隐式启动引擎。
4. B11-11750M默认变化记录实际采纳/未采纳原因与版本证据；不以default upgrade覆盖custom active selection。model/setup后台refresh失败不误enable，busy动作不可重入，旧snapshot不能覆盖当前identity。
- **验证环境/责任**: 以后用本地qualified binary/model、source fixture与真实已准入引擎/平台；组件签名/安装路径单独Not run。 对应实际能力 owner 取得证据，集成验收 owner 消费已验证结果。
- **stop condition**: 每个上述resource子合同都有来源、失败断言与责任，consumer功能证据可引用；未准入平台保持有名缺口，不能用catalog存在结束。

完整行为/非目标/实际阻塞/证据门：[UD-04-001](UPSTREAM_DELTA.md#ud-04-001), [UD-04-002](UPSTREAM_DELTA.md#ud-04-002), [UD-04-005](UPSTREAM_DELTA.md#ud-04-005), [UD-04-006](UPSTREAM_DELTA.md#ud-04-006), [UD-04-008](UPSTREAM_DELTA.md#ud-04-008), [UD-04-010](UPSTREAM_DELTA.md#ud-04-010), [UD-04-014](UPSTREAM_DELTA.md#ud-04-014), [UD-04-016](UPSTREAM_DELTA.md#ud-04-016), [UD-04-018](UPSTREAM_DELTA.md#ud-04-018), [UD-04-021](UPSTREAM_DELTA.md#ud-04-021), [UD-04-024](UPSTREAM_DELTA.md#ud-04-024), [UD-04-026](UPSTREAM_DELTA.md#ud-04-026), [UD-04-028](UPSTREAM_DELTA.md#ud-04-028), [UD-04-035](UPSTREAM_DELTA.md#ud-04-035), [UD-04-037](UPSTREAM_DELTA.md#ud-04-037), [UD-04-038](UPSTREAM_DELTA.md#ud-04-038), [UD-04-043](UPSTREAM_DELTA.md#ud-04-043).

<a id="t04-handoff"></a>
### T04-HANDOFF

当前公开合同：[R13 / autoquick](R13_PLAN.md#autoquick)。

唯一责任意图 `T04-HANDOFF`；性质 有界调查；必要时产生范围明确后继。历史来源 grouping/context：B；当前correctness差异可提前；ANA-06/16保持Accepted；T02-AUTOLOAD-QUICK独立。。

- **kind**: 有界调查；必要时产生范围明确后继
- **历史来源 grouping / 行为范围**: B；当前correctness差异可提前；ANA-06/16保持Accepted；T02-AUTOLOAD-QUICK独立。
- **sourceDelta**: UD-04-004、UD-04-013。
- **actual blockers**: 需要比对Java实际whole-game window与Next task panel的可达close/reopen/terminal语义；H-ANALYSIS不变边界足够的部分无需重跑，未证明细分不标Covered。
- **bounded observable assertions**:
冻结完成、Cancel、Pause、close/reopen的用户意图和原task/Run/job身份；只交还一次，暂停不被恢复覆盖，旧terminal不能恢复新任务的普通analysis。复用原Windows handoff/lane-local Cancel证据并单列即时window时序余项，不用DTO定义/Start调用作为结果证据。无对应新可达路径时以入口/source比对闭合该Java机制。
- **验证环境/责任**: 固定源码/原acceptance记录先行；动态余项由后续实际Windows/real-KataGo验证，不在本审计执行。 对应实际能力 owner 取得证据，集成验收 owner 消费已验证结果。
- **stop condition**: 输出原覆盖与新差异表、可达入口和bounded failure assertions；有缺口生成后继，无缺口给窄范围理由，均不修改原Accepted含义。

完整行为/非目标/实际阻塞/证据门：[UD-04-004](UPSTREAM_DELTA.md#ud-04-004), [UD-04-013](UPSTREAM_DELTA.md#ud-04-013).

<a id="t04-startup-performance"></a>
### T04-STARTUP-PERFORMANCE

当前公开合同：[R12 / startup](R12_PLAN.md#startup)。

唯一责任意图 `T04-STARTUP-PERFORMANCE`；性质 产品决策（未批准非等价）。历史来源 grouping/context：B；未分配后继；与T04-MEASURED-TUNING及T02-H18/preload目标去重，不取代原意图。。

- **kind**: 产品决策（未批准非等价）
- **历史来源 grouping / 行为范围**: B；未分配后继；与T04-MEASURED-TUNING及T02-H18/preload目标去重，不取代原意图。
- **sourceDelta**: UD-04-009、UD-04-013。
- **actual blockers**: 缺少逐项startup评估/默认/让位的历史批准和Next对应用户合同；普通single-lifecycle原则不能充当批准。
- **bounded observable assertions**:
保留性能评估用户目标、分析优先、不误报、pause intent与明确引擎选择；调查freeze source的entry/default/persistence/失败，决定显式入口/是否启动评估/如何让位，注明批准人/来源。Java调度可排除但非等价剩余不得静默删。
- **验证环境/责任**: 固定源及approved decision记录；没有真实benchmark义务在本票。 对应实际能力 owner 取得证据，集成验收 owner 消费已验证结果。
- **stop condition**: 交付具名决定、逐项批准引用或保留功能后继合同；依赖该政策的未来功能在此前不标ready-for-agent。

完整行为/非目标/实际阻塞/证据门：[UD-04-009](UPSTREAM_DELTA.md#ud-04-009), [UD-04-013](UPSTREAM_DELTA.md#ud-04-013).

<a id="t04-sync-confirm"></a>
### T04-SYNC-CONFIRM

当前公开合同：[R12 / lifecycle](R12_PLAN.md#lifecycle)。

唯一责任意图 `T04-SYNC-CONFIRM`；性质 有界正确性调查。历史来源 grouping/context：B；现支持路径差异提前；ENG-02/04、ANA-03现同步边界。。

- **kind**: 有界正确性调查
- **历史来源 grouping / 行为范围**: B；现支持路径差异提前；ENG-02/04、ANA-03现同步边界。
- **sourceDelta**: UD-04-019。
- **actual blockers**: 实际current position同步/restore与identity路径需对比；Rust数据权威不能豁免protocol确认。
- **bounded observable assertions**:
错误响应/发送失败/timeout/旧Run迟到事件不能将当前target标已同步或发布分析；confirmed success必须是捕获position/Run/job的证据。区分原fixture、real asset switch和新race，若缺口形成具体后继。
- **验证环境/责任**: 固定source及fixture证据，新增dynamic proof交实际engine功能票。 对应实际能力 owner 取得证据，集成验收 owner 消费已验证结果。
- **stop condition**: 所有实际支持入口的确认/失败/late-identity断言可判；无差异须给原测试/候选条件引用而非architecture免疫。

完整行为/非目标/实际阻塞/证据门：[UD-04-019](UPSTREAM_DELTA.md#ud-04-019).

<a id="t04-rollback"></a>
### T04-ROLLBACK

当前公开合同：[R12 / lifecycle](R12_PLAN.md#lifecycle)。

唯一责任意图 `T04-ROLLBACK`；性质 有界调查（普通failed-switch reader-fence）。历史来源 grouping/context：B；ENG-03/04原合同；TensorRT显式repair归T04-RESOURCE。。

- **kind**: 有界调查（普通failed-switch reader-fence）
- **历史来源 grouping / 行为范围**: B；ENG-03/04原合同；TensorRT显式repair归T04-RESOURCE。
- **sourceDelta**: UD-04-026。
- **actual blockers**: 原H-SWITCH只证明具体真实asset失败和fixture stale类；Javadeferred reader时序是否存在Next对应path未证。
- **bounded observable assertions**:
188c925d只恢复原A：precise incarnation可以settle deferred initialization，但先前analysis在new physical-write前不发布。B失败、superseded B、A退出、old events分别按ENG-04终态；不得默启不同backend/profile。对缺少reader的路径核对有界rollback与恢复analysis权威，而不是以guidance message替代事务。
- **验证环境/责任**: 源码/原fixture和Windows候选证据先引用；新的reader动态余项之后由实际Run场景验。 对应实际能力 owner 取得证据，集成验收 owner 消费已验证结果。
- **stop condition**: 产出路径/identity/fence可达对比；需新行为时后继保留same-user-choice，不产生自动后端fallback任务。

完整行为/非目标/实际阻塞/证据门：[UD-04-026](UPSTREAM_DELTA.md#ud-04-026).

<a id="t04-humansl"></a>
### T04-HUMANSL

当前公开合同：[R15 / humansl](R15_PLAN.md#humansl)。

唯一责任意图 `T02-GAME-08`；性质 GAME-08功能增量（T02-GAME-08来源细化）。历史来源 grouping/context：D。

- **kind**: GAME-08功能增量（T02-GAME-08来源细化）
- **历史来源 grouping / 行为范围**: D；GAME-08 Deferred；原GAME-01/02/05及ANA-04消费边界保留。
- **sourceDelta**: UD-04-012、UD-04-015、UD-04-017。
- **actual blockers**: HumanSL core功能与兼容profile按T02-GAME-08；实际compatible HumanSL binary/model必需，已Accepted core不意味着HumanSL已实现。无安装/编译前置。
- **bounded observable assertions**:
启动overlay只清GTP thread alias并给JSON analysis有效线程，不改用户config字节/保存命令，也不禁绝必要继承。policy/verify/deepen每个request使用overrideSettings.maxTime按剩余deadline及return/delivery reserve；maxVisits不作clock。
最终c1ec182d规则：用prior requested limit+elapsed+remaining估deepen，不因low-root visits推断time-limit；low-root/high-child不压制deepen，足够更多child证据可采用，空/已知更弱child结果保留之前verified结果；不足时间minimum gain则停止，不以raw policy伪造验证。preset/rank/color/pass/retry/finish/teardown和stale request按T02-GAME-08完整合同，不用概率分布测试替这些状态/证据断言。
- **验证环境/责任**: source fixtures/预算边界与真实compatible HumanSL进程；原config/command字节比较，actual deadline/result proof。 对应实际能力 owner 取得证据，集成验收 owner 消费已验证结果。
- **stop condition**: 三source safeguards及既有T02完整HumanSL验收都具可观察证据，未执行的真实能力不得升级。

完整行为/非目标/实际阻塞/证据门：[UD-04-012](UPSTREAM_DELTA.md#ud-04-012), [UD-04-015](UPSTREAM_DELTA.md#ud-04-015), [UD-04-017](UPSTREAM_DELTA.md#ud-04-017).

<a id="t04-transform"></a>
### T04-TRANSFORM

当前公开合同：[R11 / behavior-a10](R11_PLAN.md#behavior-a10)。

唯一责任意图 `T02-SGF-16`；性质 SGF-16功能来源增量（T02-SGF-16）。历史来源 grouping/context：A。

- **kind**: SGF-16功能来源增量（T02-SGF-16）
- **历史来源 grouping / 行为范围**: A；SGF-16 Deferred；SGF-01/11/14原prereqs不改。
- **sourceDelta**: UD-04-027。
- **actual blockers**: whole-tree transform尚未实现；使用T02-SGF-16冻结语义，不用absence声明当前bug。
- **bounded observable assertions**:
rotate/mirror/color swap覆盖所有branch的moves/setup/markup，第一真实手不跳，leading W pass颜色保持/交换一致；NodePath结构和无关metadata保留，serialize/reparse一致，cancel/error非破坏。颜色拥有的属性一起交换，实际支持coordinate properties逐字段有覆盖，不只变主盘图像。
- **验证环境/责任**: SGF fixtures与authoring/native save/reopen由实现票获取。 对应实际能力 owner 取得证据，集成验收 owner 消费已验证结果。
- **stop condition**: 完整whole-tree语义合同通过；不升级其他Accepted scope、不捏造当前transform失效runtime。

完整行为/非目标/实际阻塞/证据门：[UD-04-027](UPSTREAM_DELTA.md#ud-04-027).

<a id="t04-offline"></a>
### T04-OFFLINE

当前公开合同：[R11 / behavior-a25](R11_PLAN.md#behavior-a25)。

唯一责任意图 `T04-OFFLINE`；性质 有界细分调查；不重验全部no-engine功能。历史来源 grouping/context：A/B相关现支持路径；UI-04、SGF-10/13、ANA-11原Accepted；H-OFFLINE/H-KOMI-EDITOR。。

- **kind**: 有界细分调查；不重验全部no-engine功能
- **历史来源 grouping / 行为范围**: A/B相关现支持路径；UI-04、SGF-10/13、ANA-11原Accepted；H-OFFLINE/H-KOMI-EDITOR。
- **sourceDelta**: UD-04-027、UD-04-033、UD-04-039。
- **actual blockers**: ownership prefs、offline clear KM、offline chart原候选细分归属未完整闭合；必须先读实际entry/default/saved behavior。
- **bounded observable assertions**:
维持原no-engine open/edit/save；检七项中ownership显示15类实际菜单若Next有相应入口应不需engine；不支持engine-only动作明确disabled。clear保留贴目与explicit New/board-size form用户参数合同区分；no-engine chart显示已有SGF分析且不制造engine结果。若当前source和原run证明原覆盖就窄闭合，否则产生具名后继而非批量Missing。
- **验证环境/责任**: 固定source/既有native record，细分runtime缺口由affected owner获取。 对应实际能力 owner 取得证据，集成验收 owner 消费已验证结果。
- **stop condition**: 交付各子行为source/candidate/conditions或后继，尤其clear KM和chart不得以metadata/editor架构替代。

完整行为/非目标/实际阻塞/证据门：[UD-04-027](UPSTREAM_DELTA.md#ud-04-027), [UD-04-033](UPSTREAM_DELTA.md#ud-04-033), [UD-04-039](UPSTREAM_DELTA.md#ud-04-039).

<a id="t04-model-identity"></a>
### T04-MODEL-IDENTITY

当前公开合同：[R12 / models](R12_PLAN.md#models)。

唯一责任意图 `T04-MODEL-IDENTITY`；性质 本地resource/model用户功能后继。历史来源 grouping/context：B；ENG-01原profile；T01-RESOURCE；未分配扩展。。

- **kind**: 本地resource/model用户功能后继
- **历史来源 grouping / 行为范围**: B；ENG-01原profile；T01-RESOURCE；未分配扩展。
- **sourceDelta**: UD-04-030、UD-04-031、UD-04-039、UD-04-043。
- **actual blockers**: 模型header、managed ownership、retention/catalog snapshot的产品合同与可用本地样本；不以RCOMP-01/ENG-03占位。
- **bounded observable assertions**:
解析header获得真实模型identity，损坏/unknown可见且custom候选不消失；同路径replacement重新validate。profile名称≠ownership，rename/manual command/restart不丢原origin或覆写custom inputs。B11→B10→B11都保持installed候选，不变为download-only、不需重复下载；refresh后台snapshot+busy gate不retarget saved identity、失败不误enable。
- **验证环境/责任**: header/catalog/persistence fixtures与合格本地真实模型/引擎选择；网络下载仅若consumer实现需真实统一策略，不签名Release。 对应实际能力 owner 取得证据，集成验收 owner 消费已验证结果。
- **stop condition**: metadata/content/selection identity和installed availability的每条断言有证据；原profile存储Accepted不扩大。

完整行为/非目标/实际阻塞/证据门：[UD-04-030](UPSTREAM_DELTA.md#ud-04-030), [UD-04-031](UPSTREAM_DELTA.md#ud-04-031), [UD-04-039](UPSTREAM_DELTA.md#ud-04-039), [UD-04-043](UPSTREAM_DELTA.md#ud-04-043).

<a id="t04-batch"></a>
### T04-BATCH

当前公开合同：[R13 / batch](R13_PLAN.md#batch)。

唯一责任意图 `T02-ANA-07`；性质 ANA-07功能来源增量（T02-ANA-07）。历史来源 grouping/context：B。

- **kind**: ANA-07功能来源增量（T02-ANA-07）
- **历史来源 grouping / 行为范围**: B；ANA-07 Deferred；ANA-02/03 prereqs与APP-02Promotion语义分开。
- **sourceDelta**: UD-04-032。
- **actual blockers**: 按T02-ANA-07准入ordered queue/output合同；ANA-16只是单current-game，不替队列。已available file/run能力可引用，无R11。
- **bounded observable assertions**:
各实际菜单/toolbar/multi-file入口启动同session-only队列；显示per-file状态、waiting admission不消失、顺序和Cancel可判；run/job/file stale fence，旧completion不改new batch/current-game。batch不能replace/dirty current-game，queue不恢复；显式output按源快照并在失败保护原文件。
- **验证环境/责任**: queue/identity fixtures、真实qualified engine与实际文件失败/取消；不执行本票。 对应实际能力 owner 取得证据，集成验收 owner 消费已验证结果。
- **stop condition**: T02完整batch+PR529startup/admission增量都有责任和observable结果。

完整行为/非目标/实际阻塞/证据门：[UD-04-032](UPSTREAM_DELTA.md#ud-04-032).

<a id="t04-komi"></a>
### T04-KOMI

当前公开合同：[R15 / komi](R15_PLAN.md#komi)。

唯一责任意图 `T04-KOMI`；性质 具名runtime-komi正确性后继（未分配新Parity ID）。历史来源 grouping/context：D；现PK可达风险可提前插相关功能批；GAME-01/03/04既有Accepted不变；SGF-13 editor独立。。

- **kind**: 具名runtime-komi正确性后继（未分配新Parity ID）
- **历史来源 grouping / 行为范围**: D；现PK可达风险可提前插相关功能批；GAME-01/03/04既有Accepted不变；SGF-13 editor独立。
- **sourceDelta**: UD-04-033。
- **actual blockers**: 先冻结Next current Match Session修改KM的用户entry/owner合同；已有PK是可达性，不是此Java defect已复现。需要dual-participant兼容profile/明确revision接口后才ready。
- **bounded observable assertions**:
双参与者ACK+最终fence成功前KM/cache不可commit；pending edits只latest，当前genmove须先合法结算/对手接受；期间不重复resume，最终按latest pause意图。失去owner/revision/instance、拒绝/send failure/timeout保持last confirmed KM，终止当前批次不造result/next game；partial/cancel退役可能含未确认KM的原实例，late ACK/move/analysis不污染new instance/session。frozen opening与后续batch defaults不改。
- **验证环境/责任**: owner/协议fixture和真实已准入dual-engine match；取消/partial failure/替换身份需实际状态证据。 对应实际能力 owner 取得证据，集成验收 owner 消费已验证结果。
- **stop condition**: 完整transaction断言闭合，不允许仅观察komi命令就验收；没有runtime证明时不声称Next已有产品故障。

完整行为/非目标/实际阻塞/证据门：[UD-04-033](UPSTREAM_DELTA.md#ud-04-033).

<a id="t04-rules"></a>
### T04-RULES

当前公开合同：[R15 / rules](R15_PLAN.md#rules)。

唯一责任意图 `T04-RULES`；性质 五family/custom规则功能后继。历史来源 grouping/context：D；GAME-04 Shared rules/budgets原Accepted；GAME-01–03原范围。。

- **kind**: 五family/custom规则功能后继
- **历史来源 grouping / 行为范围**: D；GAME-04 Shared rules/budgets原Accepted；GAME-01–03原范围。
- **sourceDelta**: UD-04-034。
- **actual blockers**: 需明确Next family display与exact parameter映射合同、adapter准入；不重写已有saved parameters。既有Chinese/GTP证据不证明五family都验。
- **bounded observable assertions**:
Chinese、Japanese/Korean、AGA/BGA、New Zealand、Tromp-Taylor均按exact semantic parameter matching；其他custom且保留全部raw参数。打开/取消/无改动保存不canonicalize旧preset或删除custom；explicit用户selection才改。saved/reopen和真实participant同规则；不支持adapter可见拒绝，不悄改规则。
- **验证环境/责任**: 参数fixture/保存重开+准入真实engines；不要求五种未经支持的Generic模式。 对应实际能力 owner 取得证据，集成验收 owner 消费已验证结果。
- **stop condition**: 每family/custom/保存非变行为有assertion，新scope用后继而不把原GAME-04判未完成。

完整行为/非目标/实际阻塞/证据门：[UD-04-034](UPSTREAM_DELTA.md#ud-04-034).

<a id="t04-save-target"></a>
### T04-SAVE-TARGET

当前公开合同：[R11 / behavior-a26](R11_PLAN.md#behavior-a26)。

唯一责任意图 `T04-SAVE-TARGET`；性质 有界调查及raw-scope决策输入。历史来源 grouping/context：A；current save安全差异可提前；SGF-06 save；ANA-06 analysis intent；T02-H12 raw目标未决。。

- **kind**: 有界调查及raw-scope决策输入
- **历史来源 grouping / 行为范围**: A；current save安全差异可提前；SGF-06 save；ANA-06 analysis intent；T02-H12 raw目标未决。
- **sourceDelta**: UD-04-039。
- **actual blockers**: 当前普通Save/SaveAs target与engine-intent source/原run需核对；raw用户等价先等T02-H12决定，不能延迟current普通save调查。
- **bounded observable assertions**:
四Java save modes先normalize最终.sgftarget再overwrite prompt；已存在目标Cancel原字节、source path、dirty不变；game.SGF不追加第二后缀。原raw/comment flags在所有返回路径恢复，此机制不移植但未来raw模式不能泄漏设置。ordinary Save取消需保持捕获原same-engine此前analysis意图，不resume新Run或用户paused；比较Next既有departure明确停止合同，不偷偷改变该批准边界。
- **验证环境/责任**: 固定save source/历史native记录先行；需要补实际chooser/overwrite/analysis-scoped proof的交付后继获取，不在本票执行。 对应实际能力 owner 取得证据，集成验收 owner 消费已验证结果。
- **stop condition**: 七行为中save四项每项都闭合原覆盖或named successor/decision，记录不同cancel contracts及target byte safety。

完整行为/非目标/实际阻塞/证据门：[UD-04-039](UPSTREAM_DELTA.md#ud-04-039).

<a id="t04-save-atomic"></a>
### T04-SAVE-ATOMIC

当前公开合同：[R11 / behavior-a27](R11_PLAN.md#behavior-a27)。

唯一责任意图 `T04-SAVE-ATOMIC`；性质 SGF保存durability后继；async scheduling细分调查。历史来源 grouping/context：A；数据正确性可提前；SGF-06原save与SGF-07安全replacement保持；未分配后继。。

- **kind**: SGF保存durability后继；async scheduling细分调查
- **历史来源 grouping / 行为范围**: A；数据正确性可提前；SGF-06原save与SGF-07安全replacement保持；未分配后继。
- **sourceDelta**: UD-04-045。
- **actual blockers**: 原persist_save_snapshot是directstd::fs::write；新增atomic replacement合同及platform失败语义须冻结。off-UI调度是独立未证子门，不用picker offload替写盘。无runtime数据丢失报告。
- **bounded observable assertions**:
调用snapshot冻结全部tree与analysis再serialize，后续编辑不混入；既存target在准备/写/replace失败保留全部原bytes、path/dirty/current tree不变；成功只清本saved snapshot revision，不吞后续edit。异步write在实际UI保存中保持响应/结果可见，late snapshot成功不clean新revision，取消/失败不replace当前game。temp/rename是可选实现，不把catalog原atomic evidence移作SGF证据。
- **验证环境/责任**: Rust/file fault fixtures与实际本地native Save/reopen/失败证据；无installer/engine编译门。 对应实际能力 owner 取得证据，集成验收 owner 消费已验证结果。
- **stop condition**: snapshot/byte-preservation/dirty和真实scheduling各自产证；不改历史SGF-06/07验收含义。

完整行为/非目标/实际阻塞/证据门：[UD-04-045](UPSTREAM_DELTA.md#ud-04-045).

<a id="t04-lan"></a>
### T04-LAN

当前公开合同：[R16 / lan](R16_PLAN.md#lan)。

唯一责任意图 `T02-PUB-01`；性质 PUB-01功能增量（T02-PUB-01）。历史来源 grouping/context：E。

- **kind**: PUB-01功能增量（T02-PUB-01）
- **历史来源 grouping / 行为范围**: E；PUB-01 Deferred；APP-03 teardown。
- **sourceDelta**: UD-04-044。
- **actual blockers**: LAN publishing按T02-PUB-01准入start/stop/URL合同；无account/proxy credentials，入站LAN不属provider policy。
- **bounded observable assertions**:
每slow client pending state队列有明确上限/最新完整状态coalesce，快client仍推进，不要求Java同队列实现；Stop关闭owned sockets/jobs，Start session身份阻断late delivery。copy URL对应实际端点，不造trial counters或账号义务。
- **验证环境/责任**: local真实client/slow socket场景及取消；无需外网server或正式Release。 对应实际能力 owner 取得证据，集成验收 owner 消费已验证结果。
- **stop condition**: 原PUB-01完整用户功能与本源backpressure断言均满足。

完整行为/非目标/实际阻塞/证据门：[UD-04-044](UPSTREAM_DELTA.md#ud-04-044).

<a id="t04-diagnostics"></a>
### T04-DIAGNOSTICS

当前公开合同：[R12 / diagnostics](R12_PLAN.md#diagnostics)。

唯一责任意图 `T01-DIAGNOSTICS`；性质 功能增量（T01-DIAGNOSTICS/REL-09功能部分）。历史来源 grouping/context：B（首个Run/资源消费者）；C/D/E各消费者扩展；R11仅安装支持包。

- **kind**: 功能增量（T01-DIAGNOSTICS/REL-09功能部分）
- **历史来源 grouping / 行为范围**: 随引擎/分析等受影响功能交付，非统一等E/R11；REL-09 Missing原scope；ENG-02原typed failure。
- **sourceDelta**: UD-04-040、UD-04-046、UD-04-048。
- **actual blockers**: 需要bounded attempt/output/export identity与redaction合同；相关启动路径真实证据以后取得，不以PEscanner为prereq。安装日志路径/support package另gate。
- **bounded observable assertions**:
最终PR545只有per-attempt visible failure/command、details、copy、pinned displayed-error export；普通WARN、late stdout/stderr保留原engine/launch identity，有限bytes/lines/deadline，不把新Run输出接到旧attempt。Copy/export通过同redaction，包含明显source/status/exit与缺失可见，control chars/credential/token不得泄漏。export受限可取消，close/retry/switch不改变被导出的displayed snapshot，failure可见且不自动上传/fulltrace。新diag窗口长文本/buttons在实际Windowswork-area/DPI可达；PEscanner/所有internal launcher统一session-history已删除，不列最终migration功能。
- **验证环境/责任**: source/fixtures与真实Startup error/window/导出；Linux/browser不是Windowsnative。功能资源可本地，installed日志路径单列。 对应实际能力 owner 取得证据，集成验收 owner 消费已验证结果。
- **stop condition**: 每个受影响功能已有必要可见错误/日志/可取消脱敏导出门，最终source cutover和对应native残余不丢。

完整行为/非目标/实际阻塞/证据门：[UD-04-040](UPSTREAM_DELTA.md#ud-04-040), [UD-04-046](UPSTREAM_DELTA.md#ud-04-046), [UD-04-048](UPSTREAM_DELTA.md#ud-04-048).

<a id="t04-measured-tuning"></a>
### T04-MEASURED-TUNING

当前公开合同：[R13 / performance](R13_PLAN.md#performance)。

唯一责任意图 `T04-MEASURED-TUNING`；性质 具名measured-report用户功能后继。历史来源 grouping/context：B；新scope未分配Parity；不重定义ENG-06；T04-RESOURCE/模型identity只消费所需部分。。

- **kind**: 具名measured-report用户功能后继
- **历史来源 grouping / 行为范围**: B；新scope未分配Parity；不重定义ENG-06；T04-RESOURCE/模型identity只消费所需部分。
- **sourceDelta**: UD-04-042、UD-04-047、UD-04-048。
- **actual blockers**: 先冻结Next scene overlays/settings与已有单Run/lane身份兼容设计和schema准入；3/5 rounds等以已冻结source合同为依据，不在本审计跑benchmark。静态argv不满足该前置。
- **bounded observable assertions**:
1. scene为live或whole-game，report import→后台验证→review展示saved engine identity、参数/paired speed gain→独立explicit Apply；只review/import不改变settings。resolve engine/model/config只能从saved local command，不执行report paths/commandSemantics。verify engine/model/config SHA256、ordered recursively included configs、GPU/hardware/driver/memory及有效non-tuning command settings；不忽略visit/time/precision/PV/search参数。include不可用/unsupported是failure，同尺寸/时间替换也必须失效。
2. live参数numSearchThreads/nnMaxBatchSize；whole-game参数numAnalysisThreads/numSearchThreadsPerAnalysisThread/nnMaxBatchSize。接受的scene launch overlay可以优先该scene手动参数，但保存原command原字节/明确overrides不被rewrite，下次对应engine启动生效；Restore移除两个accepted overlay且保留后续manual command edits及legacy Apple tuning key。whole-game overlay不spill quick/HumanSL；separate SSH whole-game或analysisReuseCurrentEngine在review/apply/launch均unsupported，slow hash后重验，不禁local live。
3. asset/hardware/driver/command变化使recommendation失效并保留defaults；setup discovery/model change/benchmark/import/restore共享busy gate，连已queued动作和late callbacks也不能绕过。close/hide取消worker、invalidate generation；reopen后old success/error/confirmation不得显示或Apply。current saved engine identity不被later discovery retarget。model hashing不在render/event loop，opt-in verify在review/apply/launch，cold verification成本分记。
4. version1的measurement limits保持：engine-only输出不能替app固定budget completion/response/event-loop；whole-game rootVisitsByTurn每position达到budget，Cancel metric是active worker实际process exit，不是JSON termination ACK。3或5独立warm pairs，cold不进runs，有限metrics/scene allowed keys/unsupported schema拒绝；paired speed每pair更快且median至少1.03；三轮CV>10%须五轮，五轮CV>15%拒绝；latency median/max、memory余量阈值按固定schema。sampled total GPU memory不是continuous peak/per-process VRAM，也不证明RAM/heap/RSS无回归；未决RAM regression只能experimental，保留原config，不给unconditional recommendation。
- **验证环境/责任**: source schema/状态/设置fault fixtures；后续真实controlled machine/driver/workload/one-KataGo measurement与native review/apply/restore，样例数值不算证据。 对应实际能力 owner 取得证据，集成验收 owner 消费已验证结果。
- **stop condition**: 完整workflow/identity/nonmutation/busy/stale/measurement boundaries均有结果；不能仅threads argv通过。

完整行为/非目标/实际阻塞/证据门：[UD-04-042](UPSTREAM_DELTA.md#ud-04-042), [UD-04-047](UPSTREAM_DELTA.md#ud-04-047), [UD-04-048](UPSTREAM_DELTA.md#ud-04-048).

<a id="t04-thread-control"></a>
### T04-THREAD-CONTROL

当前公开合同：[R12 / manual-threads](R12_PLAN.md#manual-threads)。

唯一责任意图 `T04-THREAD-CONTROL`；性质 有界来源/政策决策，随后功能后继。历史来源 grouping/context：B；静态argvH-STATIC-THREAD保留；dynamic来源/有效值/临时覆盖是未分配新scope。。

- **kind**: 有界来源/政策决策，随后功能后继
- **历史来源 grouping / 行为范围**: B；静态argvH-STATIC-THREAD保留；dynamic来源/有效值/临时覆盖是未分配新scope。
- **sourceDelta**: UD-04-047。
- **actual blockers**: 需freeze CFG/BENCHMARK/effective launch值/temporary override与已Run更新边界；当前真正dynamic runtime控制没有继承证据，需实际adapter支持/用户明确动作。
- **bounded observable assertions**:
saved config、source policy、effective launch override各有明确precedence和identity；explicit static overrides保留，初始化不通过kata-set-param numSearchThreads回写legacy值覆盖resolved override。展示effective与pending、temporary override结束/Restart的归还规则独立决定；不自动重启/切换，没支持dynamic capability可见拒绝。H-STATIC仅Restart2threads，不外推此工作流。
- **验证环境/责任**: 固定source政策与真实已准入engine后续证据；无需编译新upstream引擎。 对应实际能力 owner 取得证据，集成验收 owner 消费已验证结果。
- **stop condition**: 产出source/effective/temp/run作用域可观察合同和dynamic可达性证明后才使功能ready；原static证据不被降级或伪扩。

完整行为/非目标/实际阻塞/证据门：[UD-04-047](UPSTREAM_DELTA.md#ud-04-047).

<a id="t05-release-provenance"></a>
### T05-RELEASE-PROVENANCE

当前公开合同：证据继承/排除；无新功能责任（见 [routes evidence_only](MIGRATION_ROUTES.json)）。

唯一责任意图 `T05-RELEASE-PROVENANCE`；性质 inheritance-or-exclusion。历史来源 grouping/context：R11。

- 后续验收/调查停止条件：未来选定托管后验证实际发布资产、清单/版本一致与失败可见；本轮只验来源和包含关系。

完整行为/非目标/实际阻塞/证据门：[UD-05-01](UPSTREAM_DELTA.md#ud-05-01).

<a id="t05-remote-model-refresh"></a>
### T05-REMOTE-MODEL-REFRESH

当前公开合同：[R14 / compute-network](R14_PLAN.md#compute-network)。

唯一责任意图 `T01-REMOTE`；性质 see-source-disposition。历史来源 grouping/context：C。

- 后续验收/调查停止条件：真实支持端点刷新、空列表/过期结果/断线与键盘操作；无凭据则Blocked。

完整行为/非目标/实际阻塞/证据门：[UD-05-02](UPSTREAM_DELTA.md#ud-05-02).

<a id="t05-workbench-access"></a>
### T05-WORKBENCH-ACCESS

当前公开合同：[R17 / accessibility](R17_PLAN.md#accessibility)。

唯一责任意图 `T05-WORKBENCH-ACCESS`；性质 see-source-disposition。历史来源 grouping/context：F。

- 后续验收/调查停止条件：各新入口键盘/取消/焦点/语言溢出；最终UI-02实际桌面主题、字体、缩放、屏幕阅读组合。

完整行为/非目标/实际阻塞/证据门：[UD-05-03](UPSTREAM_DELTA.md#ud-05-03).

<a id="t05-internal-evidence"></a>
### T05-INTERNAL-EVIDENCE

当前公开合同：证据继承/排除；无新功能责任（见 [routes evidence_only](MIGRATION_ROUTES.json)）。

唯一责任意图 `T05-INTERNAL-EVIDENCE`；性质 inheritance-or-exclusion。历史来源 grouping/context：R11。

- 后续验收/调查停止条件：源索引每提交有归属，相关用户义务仍在其他行；无Java构建/测试执行。

完整行为/非目标/实际阻塞/证据门：[UD-05-04](UPSTREAM_DELTA.md#ud-05-04).

<a id="t05-model-identity"></a>
### T05-MODEL-IDENTITY

当前公开合同：[R12 / models](R12_PLAN.md#models)。

唯一责任意图 `T04-MODEL-IDENTITY`；性质 see-source-disposition。历史来源 grouping/context：B；ENG-01原profile；T01-RESOURCE；未分配扩展。。

- 后续验收/调查停止条件：选择不同模型/间接启动时标识真实来源版本与路径；不伪报棋力；缺资源可见失败，无引擎照常复盘。

完整行为/非目标/实际阻塞/证据门：[UD-05-05](UPSTREAM_DELTA.md#ud-05-05).

<a id="t05-sync-lag-check"></a>
### T05-SYNC-LAG-CHECK

当前公开合同：[R14 / readboard](R14_PLAN.md#readboard)。

唯一责任意图 `T05-SYNC-LAG-CHECK`；性质 see-source-disposition。历史来源 grouping/context：C。

- 后续验收/调查停止条件：重放来自受支持Fox场景的落后标题/新帧顺序，确认已落子不回滚；若不同则按唯一外部权威契约修复并实际桌面复验。

完整行为/非目标/实际阻塞/证据门：[UD-05-06](UPSTREAM_DELTA.md#ud-05-06).

<a id="t05-ancient-rules"></a>
### T05-ANCIENT-RULES

当前公开合同：[R15 / rules](R15_PLAN.md#rules)。

唯一责任意图 `T05-ANCIENT-RULES`；性质 see-source-disposition。历史来源 grouping/context：D。

- 后续验收/调查停止条件：支持引擎可建立同规则棋局、保存/重开一致；不支持明确拒绝而非换规则。

完整行为/非目标/实际阻塞/证据门：[UD-05-07](UPSTREAM_DELTA.md#ud-05-07).

<a id="t05-startup-diagnostics"></a>
### T05-STARTUP-DIAGNOSTICS

当前公开合同：[R12 / diagnostics](R12_PLAN.md#diagnostics)。

唯一责任意图 `T01-DIAGNOSTICS`；性质 see-source-disposition。历史来源 grouping/context：B（首个Run/资源消费者）；C/D/E各消费者扩展；R11仅安装支持包。

- 后续验收/调查停止条件：缺路径/坏配置/版本不兼容/子进程退出均显示真实原因，普通日志有界，敏感值不出屏或导出。

完整行为/非目标/实际阻塞/证据门：[UD-05-08](UPSTREAM_DELTA.md#ud-05-08).

<a id="t05-runtime-threads"></a>
### T05-RUNTIME-THREADS

当前公开合同：[R12 / manual-threads](R12_PLAN.md#manual-threads)。

唯一责任意图 `T05-RUNTIME-THREADS`；性质 see-source-disposition。历史来源 grouping/context：B。

- 后续验收/调查停止条件：真实支持引擎修改并确认实际值，失效/超时/旧run响应不覆盖编辑；重启/切换/断线后的临时值寿命明确，不能假定Java1..1024就是Next协议。

完整行为/非目标/实际阻塞/证据门：[UD-05-09](UPSTREAM_DELTA.md#ud-05-09).

<a id="t05-secret-diagnostics"></a>
### T05-SECRET-DIAGNOSTICS

当前公开合同：[R12 / diagnostics](R12_PLAN.md#diagnostics)。

唯一责任意图 `T01-DIAGNOSTICS`；性质 see-source-disposition。历史来源 grouping/context：B（首个Run/资源消费者）；C/D/E各消费者扩展；R11仅安装支持包。

- 后续验收/调查停止条件：假凭据覆盖空格/引号/换行/URL及fallback显示，UI/日志/支持包无泄漏；实际账号只在授权现场验证，不自动账号写入重试。

完整行为/非目标/实际阻塞/证据门：[UD-05-10](UPSTREAM_DELTA.md#ud-05-10).

<a id="t05-sgf-date-check"></a>
### T05-SGF-DATE-CHECK

当前公开合同：[R11 / behavior-a28](R11_PLAN.md#behavior-a28)。

唯一责任意图 `T05-SGF-DATE-CHECK`；性质 see-source-disposition。历史来源 grouping/context：A。

- 后续验收/调查停止条件：原生打开含完整/部分/多日期/无DT谱，元数据编辑及Save/reopen比较DT；失败时只修可达损失路径。

完整行为/非目标/实际阻塞/证据门：[UD-05-11](UPSTREAM_DELTA.md#ud-05-11).

<a id="t05-theme-presets"></a>
### T05-THEME-PRESETS

当前公开合同：[R17 / appearance](R17_PLAN.md#appearance)。

唯一责任意图 `T05-THEME-PRESETS`；性质 see-source-disposition。历史来源 grouping/context：F。

- 后续验收/调查停止条件：预设/用户主题切换可预览、取消不写盘，重启保持；旧字段白名单映射不覆盖用户现值。

完整行为/非目标/实际阻塞/证据门：[UD-05-12](UPSTREAM_DELTA.md#ud-05-12).

<a id="t05-external-exact-restore"></a>
### T05-EXTERNAL-EXACT-RESTORE

当前公开合同：[R14 / external-match](R14_PLAN.md#external-match)。

唯一责任意图 `T05-EXTERNAL-EXACT-RESTORE`；性质 see-source-disposition。历史来源 grouping/context：C。

- 后续验收/调查停止条件：本地不等待远程凭据：受支持让子、主线前后及分支端点逐项比较 stones/to-play/komi/真实 tail，拒绝输入不改变当前棋谱或既有 Run；记录实际引擎版本/适配器及范围。外部 READ-02 另以真实让子/重建帧、Save/reopen比较局面；GAME-10 两种落子模式各由权威精确后继确认，ACK不推进回合。

完整行为/非目标/实际阻塞/证据门：[UD-05-13](UPSTREAM_DELTA.md#ud-05-13).

<a id="t05-local-exact-restore-check"></a>
### T05-LOCAL-EXACT-RESTORE-CHECK

当前公开合同：[R12 / lifecycle](R12_PLAN.md#lifecycle)。

唯一责任意图 `T05-LOCAL-EXACT-RESTORE-CHECK`；性质 see-source-disposition。历史来源 grouping/context：B。

- 后续验收/调查停止条件：本地不等待远程凭据：受支持让子、主线前后及分支端点逐项比较 stones/to-play/komi/真实 tail，拒绝输入不改变当前棋谱或既有 Run；记录实际引擎版本/适配器及范围。外部 READ-02 另以真实让子/重建帧、Save/reopen比较局面；GAME-10 两种落子模式各由权威精确后继确认，ACK不推进回合。

完整行为/非目标/实际阻塞/证据门：[UD-05-13](UPSTREAM_DELTA.md#ud-05-13).

<a id="t05-external-continuation"></a>
### T05-EXTERNAL-CONTINUATION

当前公开合同：[R14 / external-match](R14_PLAN.md#external-match)。

唯一责任意图 `T05-EXTERNAL-CONTINUATION`；性质 see-source-disposition。历史来源 grouping/context：C。

- 后续验收/调查停止条件：同步/取消/新局/手动导航穿插时无旧owner落子；仅用户已开启且当前条件合法时恢复Engine Continuation。

完整行为/非目标/实际阻塞/证据门：[UD-05-14](UPSTREAM_DELTA.md#ud-05-14).

<a id="t05-external-mode-switch"></a>
### T05-EXTERNAL-MODE-SWITCH

当前公开合同：[R14 / external-match](R14_PLAN.md#external-match)。

唯一责任意图 `T05-EXTERNAL-MODE-SWITCH`；性质 see-source-disposition。历史来源 grouping/context：C。

- 后续验收/调查停止条件：两模式切换中旧结果不落子，局面/会话变化立即失效；无精确权威确认则结束而非重试落子、undo或restart。

完整行为/非目标/实际阻塞/证据门：[UD-05-15](UPSTREAM_DELTA.md#ud-05-15).

<a id="t05-fox-rank-evidence"></a>
### T05-FOX-RANK-EVIDENCE

当前公开合同：证据继承/排除；无新功能责任（见 [routes evidence_only](MIGRATION_ROUTES.json)）。

唯一责任意图 `T05-FOX-RANK-EVIDENCE`；性质 inheritance-or-exclusion。历史来源 grouping/context：C。

- 后续验收/调查停止条件：后续PROV02变更时保持未知值可见和职业码边界；本轮不新增live声明。

完整行为/非目标/实际阻塞/证据门：[UD-05-16](UPSTREAM_DELTA.md#ud-05-16).

<a id="t05-auto-quick-owner"></a>
### T05-AUTO-QUICK-OWNER

当前公开合同：[R13 / autoquick](R13_PLAN.md#autoquick)。

唯一责任意图 `T02-AUTOLOAD-QUICK`；性质 see-source-disposition。历史来源 grouping/context：B。

- 后续验收/调查停止条件：打开谱后引擎迟到启动、手动浏览、换谱/切引擎/取消/关闭均不拉回旧节点、不写旧局面；真实引擎完整曲线与交还状态。
- 后续验收/调查停止条件：本地受支持引擎验证任务交还失败→换谱→用户显式 Restart→新规则/位置确认→用户继续，以及失败、取消、又换谱/规则/引擎、迟到旧回调；新谱/Run/暂停意图不被旧结果污染，无问题记录所测边界，有问题才修复。远程另验真实端点停止/断线/重连、排队重启/取消与外部同步：无旧曲线/旧落子、不隐藏失败，不以loopback代替真实服务。

完整行为/非目标/实际阻塞/证据门：[UD-05-17](UPSTREAM_DELTA.md#ud-05-17), [UD-05-25](UPSTREAM_DELTA.md#ud-05-25).

<a id="t05-sgf-import-check"></a>
### T05-SGF-IMPORT-CHECK

当前公开合同：[R11 / behavior-a29](R11_PLAN.md#behavior-a29)。

唯一责任意图 `T05-SGF-IMPORT-CHECK`；性质 see-source-disposition。历史来源 grouping/context：A。

- 后续验收/调查停止条件：local/clipboard/provider相同样本比对树/RE/C；恶意截断拒绝且当前棋谱不变；不得全局替换破坏属性值。

完整行为/非目标/实际阻塞/证据门：[UD-05-18](UPSTREAM_DELTA.md#ud-05-18).

<a id="t05-tree-publication-check"></a>
### T05-TREE-PUBLICATION-CHECK

当前公开合同：[R13 / display](R13_PLAN.md#display)。

唯一责任意图 `T05-TREE-PUBLICATION-CHECK`；性质 see-source-disposition。历史来源 grouping/context：B。

- 后续验收/调查停止条件：快速换谱/编辑/缩放/浏览后点击只选当前显示节点；旧结果不覆盖新上下文。

完整行为/非目标/实际阻塞/证据门：[UD-05-19](UPSTREAM_DELTA.md#ud-05-19).

<a id="t05-parameter-readback"></a>
### T05-PARAMETER-READBACK

当前公开合同：[R12 / runtime-readback](R12_PLAN.md#runtime-readback)。

唯一责任意图 `T05-PARAMETER-READBACK`；性质 see-source-disposition。历史来源 grouping/context：B。

- 后续验收/调查停止条件：两值乱序、错误、非有限值、超时/重连均保持最后有效值并显示未知/失败；当前确认不得覆盖用户未提交编辑。

完整行为/非目标/实际阻塞/证据门：[UD-05-20](UPSTREAM_DELTA.md#ud-05-20).

<a id="t05-variation-nav-check"></a>
### T05-VARIATION-NAV-CHECK

当前公开合同：[R13 / display](R13_PLAN.md#display)。

唯一责任意图 `T05-VARIATION-NAV-CHECK`；性质 see-source-disposition。历史来源 grouping/context：B。

- 后续验收/调查停止条件：首手/末手/长PV/hover替换与Review Autoplay、Variation Replay分别验证，实际谱游标不被预览输入改写。

完整行为/非目标/实际阻塞/证据门：[UD-05-21](UPSTREAM_DELTA.md#ud-05-21).

<a id="t05-update-channel-decision"></a>
### T05-UPDATE-CHANNEL-DECISION

当前公开合同：[R18 / channel](R18_PLAN.md#channel)。

唯一责任意图 `T05-UPDATE-CHANNEL-DECISION`；性质 see-source-disposition。历史来源 grouping/context：R11。

- 后续验收/调查停止条件：选定渠道契约后离线签名清单边界加实际发布更新验证；本轮不复制端点或虚报渠道就绪。

完整行为/非目标/实际阻塞/证据门：[UD-05-22](UPSTREAM_DELTA.md#ud-05-22).

<a id="t05-candidate-list-access"></a>
### T05-CANDIDATE-LIST-ACCESS

当前公开合同：[R13 / display](R13_PLAN.md#display)。

唯一责任意图 `T05-CANDIDATE-LIST-ACCESS`；性质 see-source-disposition。历史来源 grouping/context：B。

- 后续验收/调查停止条件：实时分析不断刷新时可浏览选中行；键盘/滚动不串到棋盘或锁定预览，窄高/缩放仍完整可操作。

完整行为/非目标/实际阻塞/证据门：[UD-05-23](UPSTREAM_DELTA.md#ud-05-23).

<a id="t05-preview-async-check"></a>
### T05-PREVIEW-ASYNC-CHECK

当前公开合同：[R13 / display](R13_PLAN.md#display)。

唯一责任意图 `T05-PREVIEW-ASYNC-CHECK`；性质 see-source-disposition。历史来源 grouping/context：B。

- 后续验收/调查停止条件：长PV快速切换/隐藏/换谱/重放停止无stale发布；测量当前支持面交互与取消，保留既有延时及导航契约。

完整行为/非目标/实际阻塞/证据门：[UD-05-24](UPSTREAM_DELTA.md#ud-05-24).

<a id="t05-remote-quick-handback"></a>
### T05-REMOTE-QUICK-HANDBACK

当前公开合同：[R14 / remote-handback](R14_PLAN.md#remote-handback)。

唯一责任意图 `T05-REMOTE-QUICK-HANDBACK`；性质 see-source-disposition。历史来源 grouping/context：C。

- 后续验收/调查停止条件：本地受支持引擎验证任务交还失败→换谱→用户显式 Restart→新规则/位置确认→用户继续，以及失败、取消、又换谱/规则/引擎、迟到旧回调；新谱/Run/暂停意图不被旧结果污染，无问题记录所测边界，有问题才修复。远程另验真实端点停止/断线/重连、排队重启/取消与外部同步：无旧曲线/旧落子、不隐藏失败，不以loopback代替真实服务。

完整行为/非目标/实际阻塞/证据门：[UD-05-25](UPSTREAM_DELTA.md#ud-05-25).

<a id="t05-local-import-restart-check"></a>
### T05-LOCAL-IMPORT-RESTART-CHECK

当前公开合同：[R12 / lifecycle](R12_PLAN.md#lifecycle)。

唯一责任意图 `T05-LOCAL-IMPORT-RESTART-CHECK`；性质 see-source-disposition。历史来源 grouping/context：B。

- 后续验收/调查停止条件：本地受支持引擎验证任务交还失败→换谱→用户显式 Restart→新规则/位置确认→用户继续，以及失败、取消、又换谱/规则/引擎、迟到旧回调；新谱/Run/暂停意图不被旧结果污染，无问题记录所测边界，有问题才修复。远程另验真实端点停止/断线/重连、排队重启/取消与外部同步：无旧曲线/旧落子、不隐藏失败，不以loopback代替真实服务。

完整行为/非目标/实际阻塞/证据门：[UD-05-25](UPSTREAM_DELTA.md#ud-05-25).

<a id="t05-fox-identity-check"></a>
### T05-FOX-IDENTITY-CHECK

当前公开合同：[R14 / providers](R14_PLAN.md#providers)。

唯一责任意图 `T05-FOX-IDENTITY-CHECK`；性质 see-source-disposition。历史来源 grouping/context：C。

- 后续验收/调查停止条件：同数字值分别以nickname/uid查询不会串身份；真实数字昵称需存在样本，缺则保留现场缺口；未知账号与网络失败不混淆。

完整行为/非目标/实际阻塞/证据门：[UD-05-26](UPSTREAM_DELTA.md#ud-05-26).

<a id="t05-runtime-compatibility"></a>
### T05-RUNTIME-COMPATIBILITY

当前公开合同：[R12 / resources](R12_PLAN.md#resources)。

唯一责任意图 `T01-RESOURCE`；性质 see-source-disposition。历史来源 grouping/context：B首个资源消费者；跨批共享合同。

- 后续验收/调查停止条件：用户本地合法资源含缺/错版本库、损坏模型、错误来源；显示实际失败exe及修复指引；真实HumanSL与目标硬件另验，缺硬件Blocked。

完整行为/非目标/实际阻塞/证据门：[UD-05-27](UPSTREAM_DELTA.md#ud-05-27).

<a id="t05-ai-connection"></a>
### T05-AI-CONNECTION

当前公开合同：[R16 / ai](R16_PLAN.md#ai)。

唯一责任意图 `T05-AI-CONNECTION`；性质 see-source-disposition。历史来源 grouping/context：E。

- 后续验收/调查停止条件：授权登录/刷新/登出/取消/账号切换、迟到回调与secret storage失败；无凭据不上线，不自动产生付费重试；真实撤销及支持平台各验。

完整行为/非目标/实际阻塞/证据门：[UD-05-28](UPSTREAM_DELTA.md#ud-05-28).

<a id="t05-ai-grounded-teaching"></a>
### T05-AI-GROUNDED-TEACHING

当前公开合同：[R16 / ai](R16_PLAN.md#ai)。

唯一责任意图 `T05-AI-GROUNDED-TEACHING`；性质 see-source-disposition。历史来源 grouping/context：E。

- 后续验收/调查停止条件：切谱/浏览/账号变化与流式取消不污染新节点；无分析事实不编数字，费用/可能追加校正调用需明确同意；个人C原样保留。

完整行为/非目标/实际阻塞/证据门：[UD-05-29](UPSTREAM_DELTA.md#ud-05-29).

<a id="t05-ai-preferences"></a>
### T05-AI-PREFERENCES

当前公开合同：[R16 / ai](R16_PLAN.md#ai)。

唯一责任意图 `T05-AI-PREFERENCES`；性质 see-source-disposition。历史来源 grouping/context：E。

- 后续验收/调查停止条件：键盘/屏幕阅读/多语言长标签、保存/取消、手动模型不被目录覆盖；scope变化不自动发请求。

完整行为/非目标/实际阻塞/证据门：[UD-05-30](UPSTREAM_DELTA.md#ud-05-30).

<a id="t06-a-integration"></a>
### T06-A-INTEGRATION

当前公开合同：[R11 / behavior-a30](R11_PLAN.md#behavior-a30)。

唯一责任意图 `T06-A-INTEGRATION`；性质 integration。历史来源 grouping/context：A。

A票共享当前棋谱/导出目录/设置/快捷键边界的集成验收；继承前置原候选，聚焦组合改动，不重跑无关全套或原生矩阵；不得由Closeout代做。

<a id="t06-f-integration"></a>
### T06-F-INTEGRATION

当前公开合同：[R17 / integration](R17_PLAN.md#integration)。

唯一责任意图 `T06-F-INTEGRATION`；性质 integration。历史来源 grouping/context：F。

六批跨功能结果、139入口/默认/持久化和真实功能门最后整合；原候选与失效条件逐项保留，缺环境保持未验。

<a id="t06-upstream-final"></a>
### T06-UPSTREAM-FINAL

当前公开合同：[R17 / upstream](R17_PLAN.md#upstream)。

唯一责任意图 `T06-UPSTREAM-FINAL`；性质 decision/audit。历史来源 grouping/context：F。

从af0e07a7386483f3bfc8a15780de72ffc2f0de4c追加冻结审计区间，严重当前路径风险及时调查，其余收尾复核；不移动v1，不把提交数当功能数。

<a id="t06-release-channel"></a>
### T06-RELEASE-CHANNEL

当前公开合同：[R18 / channel](R18_PLAN.md#channel)。

唯一责任意图 `T06-RELEASE-CHANNEL`；性质 decision/audit。历史来源 grouping/context：R11。

明确最终发行仓库/签名/更新源/公钥/反馈帮助配置及责任，未配置渠道不展示伪可用入口；保持产品identifier/数据身份，未经批准不执行生产发布。

## 能力与剩余条目全集接点

[能力清单](JAVA_CAPABILITY_INVENTORY.md)保留全部139 frozen headings及20路由；其本轮责任表连接这里的意图。44个原非Accepted条目均有01/02责任，Accepted的未等价目标仍由H01–H28和具名后继承接。不得以原项Accepted删除这些目标。

## 分片到语义Delta

| 分片记录 | 最终Delta | 语义目标 |
| --- | --- | --- |
| [UD-03-001](UPSTREAM_DELTA.md#ud-03-001) | UDX-001 | 必要诊断、脱敏与有界导出 |
| [UD-03-002](UPSTREAM_DELTA.md#ud-03-002) | UDX-001 | 必要诊断、脱敏与有界导出 |
| [UD-03-003](UPSTREAM_DELTA.md#ud-03-003) | UDX-001 | 必要诊断、脱敏与有界导出 |
| [UD-03-004](UPSTREAM_DELTA.md#ud-03-004) | UDX-001 | 必要诊断、脱敏与有界导出 |
| [UD-03-005](UPSTREAM_DELTA.md#ud-03-005) | UDX-001 | 必要诊断、脱敏与有界导出 |
| [UD-03-019](UPSTREAM_DELTA.md#ud-03-019) | UDX-001 | 必要诊断、脱敏与有界导出 |
| [UD-03-025](UPSTREAM_DELTA.md#ud-03-025) | UDX-001 | 必要诊断、脱敏与有界导出 |
| [UD-03-030](UPSTREAM_DELTA.md#ud-03-030) | UDX-001 | 必要诊断、脱敏与有界导出 |
| [UD-03-031](UPSTREAM_DELTA.md#ud-03-031) | UDX-001 | 必要诊断、脱敏与有界导出 |
| [UD-03-042](UPSTREAM_DELTA.md#ud-03-042) | UDX-001 | 必要诊断、脱敏与有界导出 |
| [UD-03-052](UPSTREAM_DELTA.md#ud-03-052) | UDX-001 | 必要诊断、脱敏与有界导出 |
| [UD-04-040](UPSTREAM_DELTA.md#ud-04-040) | UDX-001 | 必要诊断、脱敏与有界导出 |
| [UD-04-046](UPSTREAM_DELTA.md#ud-04-046) | UDX-001 | 必要诊断、脱敏与有界导出 |
| [UD-05-08](UPSTREAM_DELTA.md#ud-05-08) | UDX-001 | 必要诊断、脱敏与有界导出 |
| [UD-05-10](UPSTREAM_DELTA.md#ud-05-10) | UDX-001 | 必要诊断、脱敏与有界导出 |
| [UD-03-006](UPSTREAM_DELTA.md#ud-03-006) | UDX-002 | 本地运行资源、来源与模型兼容 |
| [UD-03-008](UPSTREAM_DELTA.md#ud-03-008) | UDX-002 | 本地运行资源、来源与模型兼容 |
| [UD-03-010](UPSTREAM_DELTA.md#ud-03-010) | UDX-002 | 本地运行资源、来源与模型兼容 |
| [UD-03-010a](UPSTREAM_DELTA.md#ud-03-010a) | UDX-002 | 本地运行资源、来源与模型兼容 |
| [UD-03-035](UPSTREAM_DELTA.md#ud-03-035) | UDX-002 | 本地运行资源、来源与模型兼容 |
| [UD-03-035a](UPSTREAM_DELTA.md#ud-03-035a) | UDX-002 | 本地运行资源、来源与模型兼容 |
| [UD-03-037](UPSTREAM_DELTA.md#ud-03-037) | UDX-002 | 本地运行资源、来源与模型兼容 |
| [UD-03-039](UPSTREAM_DELTA.md#ud-03-039) | UDX-002 | 本地运行资源、来源与模型兼容 |
| [UD-04-001](UPSTREAM_DELTA.md#ud-04-001) | UDX-002 | 本地运行资源、来源与模型兼容 |
| [UD-04-002](UPSTREAM_DELTA.md#ud-04-002) | UDX-002 | 本地运行资源、来源与模型兼容 |
| [UD-04-006](UPSTREAM_DELTA.md#ud-04-006) | UDX-002 | 本地运行资源、来源与模型兼容 |
| [UD-04-022](UPSTREAM_DELTA.md#ud-04-022) | UDX-002 | 本地运行资源、来源与模型兼容 |
| [UD-04-026](UPSTREAM_DELTA.md#ud-04-026) | UDX-002 | 本地运行资源、来源与模型兼容 |
| [UD-04-030](UPSTREAM_DELTA.md#ud-04-030) | UDX-002 | 本地运行资源、来源与模型兼容 |
| [UD-04-031](UPSTREAM_DELTA.md#ud-04-031) | UDX-002 | 本地运行资源、来源与模型兼容 |
| [UD-04-037](UPSTREAM_DELTA.md#ud-04-037) | UDX-002 | 本地运行资源、来源与模型兼容 |
| [UD-04-038](UPSTREAM_DELTA.md#ud-04-038) | UDX-002 | 本地运行资源、来源与模型兼容 |
| [UD-04-043](UPSTREAM_DELTA.md#ud-04-043) | UDX-002 | 本地运行资源、来源与模型兼容 |
| [UD-05-05](UPSTREAM_DELTA.md#ud-05-05) | UDX-002 | 本地运行资源、来源与模型兼容 |
| [UD-05-27](UPSTREAM_DELTA.md#ud-05-27) | UDX-002 | 本地运行资源、来源与模型兼容 |
| [UD-03-007](UPSTREAM_DELTA.md#ud-03-007) | UDX-003 | HumanSL独立AI Coach与恢复 |
| [UD-03-009](UPSTREAM_DELTA.md#ud-03-009) | UDX-003 | HumanSL独立AI Coach与恢复 |
| [UD-04-012](UPSTREAM_DELTA.md#ud-04-012) | UDX-003 | HumanSL独立AI Coach与恢复 |
| [UD-04-015](UPSTREAM_DELTA.md#ud-04-015) | UDX-003 | HumanSL独立AI Coach与恢复 |
| [UD-04-017](UPSTREAM_DELTA.md#ud-04-017) | UDX-003 | HumanSL独立AI Coach与恢复 |
| [UD-03-011](UPSTREAM_DELTA.md#ud-03-011) | UDX-004 | 现有Run/reader/取消/重启可达性调查 |
| [UD-03-018](UPSTREAM_DELTA.md#ud-03-018) | UDX-004 | 现有Run/reader/取消/重启可达性调查 |
| [UD-03-045](UPSTREAM_DELTA.md#ud-03-045) | UDX-004 | 现有Run/reader/取消/重启可达性调查 |
| [UD-03-046](UPSTREAM_DELTA.md#ud-03-046) | UDX-004 | 现有Run/reader/取消/重启可达性调查 |
| [UD-04-029](UPSTREAM_DELTA.md#ud-04-029) | UDX-004 | 现有Run/reader/取消/重启可达性调查 |
| [UD-03-012](UPSTREAM_DELTA.md#ud-03-012) | UDX-005 | AI解说连接、依据与工作区 |
| [UD-05-28](UPSTREAM_DELTA.md#ud-05-28) | UDX-005 | AI解说连接、依据与工作区 |
| [UD-05-29](UPSTREAM_DELTA.md#ud-05-29) | UDX-005 | AI解说连接、依据与工作区 |
| [UD-05-30](UPSTREAM_DELTA.md#ud-05-30) | UDX-005 | AI解说连接、依据与工作区 |
| [UD-03-013](UPSTREAM_DELTA.md#ud-03-013) | UDX-006 | 目差显示与未分析节点图导航 |
| [UD-03-021](UPSTREAM_DELTA.md#ud-03-021) | UDX-006 | 目差显示与未分析节点图导航 |
| [UD-03-014](UPSTREAM_DELTA.md#ud-03-014) | UDX-007 | 换谱与后台响应性调查 |
| [UD-03-023](UPSTREAM_DELTA.md#ud-03-023) | UDX-007 | 换谱与后台响应性调查 |
| [UD-03-015](UPSTREAM_DELTA.md#ud-03-015) | UDX-008 | 独立benchmark与性能反馈 |
| [UD-03-017](UPSTREAM_DELTA.md#ud-03-017) | UDX-008 | 独立benchmark与性能反馈 |
| [UD-03-028](UPSTREAM_DELTA.md#ud-03-028) | UDX-008 | 独立benchmark与性能反馈 |
| [UD-03-016](UPSTREAM_DELTA.md#ud-03-016) | UDX-009 | 远程算力的本地隔离与配置体验 |
| [UD-03-033](UPSTREAM_DELTA.md#ud-03-033) | UDX-009 | 远程算力的本地隔离与配置体验 |
| [UD-05-02](UPSTREAM_DELTA.md#ud-05-02) | UDX-009 | 远程算力的本地隔离与配置体验 |
| [UD-03-020](UPSTREAM_DELTA.md#ud-03-020) | UDX-010 | 连续预算历史范围继承 |
| [UD-03-022](UPSTREAM_DELTA.md#ud-03-022) | UDX-011 | 自动快析、交还与本地Restart调查 |
| [UD-03-024](UPSTREAM_DELTA.md#ud-03-024) | UDX-011 | 自动快析、交还与本地Restart调查 |
| [UD-03-041](UPSTREAM_DELTA.md#ud-03-041) | UDX-011 | 自动快析、交还与本地Restart调查 |
| [UD-03-051](UPSTREAM_DELTA.md#ud-03-051) | UDX-011 | 自动快析、交还与本地Restart调查 |
| [UD-04-004](UPSTREAM_DELTA.md#ud-04-004) | UDX-011 | 自动快析、交还与本地Restart调查 |
| [UD-05-17](UPSTREAM_DELTA.md#ud-05-17) | UDX-011 | 自动快析、交还与本地Restart调查 |
| [UD-05-25](UPSTREAM_DELTA.md#ud-05-25) | UDX-011 | 自动快析、交还与本地Restart调查 |
| [UD-03-026](UPSTREAM_DELTA.md#ud-03-026) | UDX-012 | Match既有编辑守卫继承 |
| [UD-03-027](UPSTREAM_DELTA.md#ud-03-027) | UDX-013 | 现有Match流式终态屏障调查 |
| [UD-03-029](UPSTREAM_DELTA.md#ud-03-029) | UDX-014 | 普通规则确认与Match冻结规则生命周期 |
| [UD-04-034](UPSTREAM_DELTA.md#ud-04-034) | UDX-014 | 普通规则确认与Match冻结规则生命周期 |
| [UD-05-07](UPSTREAM_DELTA.md#ud-05-07) | UDX-014 | 普通规则确认与Match冻结规则生命周期 |
| [UD-03-032](UPSTREAM_DELTA.md#ud-03-032) | UDX-015 | 普通本地位置/规则/reader精确恢复调查 |
| [UD-03-038](UPSTREAM_DELTA.md#ud-03-038) | UDX-015 | 普通本地位置/规则/reader精确恢复调查 |
| [UD-03-044](UPSTREAM_DELTA.md#ud-03-044) | UDX-015 | 普通本地位置/规则/reader精确恢复调查 |
| [UD-04-019](UPSTREAM_DELTA.md#ud-04-019) | UDX-015 | 普通本地位置/规则/reader精确恢复调查 |
| [UD-04-020](UPSTREAM_DELTA.md#ud-04-020) | UDX-015 | 普通本地位置/规则/reader精确恢复调查 |
| [UD-05-13](UPSTREAM_DELTA.md#ud-05-13) | UDX-015 | 普通本地位置/规则/reader精确恢复调查 |
| [UD-03-034](UPSTREAM_DELTA.md#ud-03-034) | UDX-016 | 只读同步退休与迟到帧调查 |
| [UD-04-041](UPSTREAM_DELTA.md#ud-04-041) | UDX-016 | 只读同步退休与迟到帧调查 |
| [UD-05-06](UPSTREAM_DELTA.md#ud-05-06) | UDX-016 | 只读同步退休与迟到帧调查 |
| [UD-03-036](UPSTREAM_DELTA.md#ud-03-036) | UDX-017 | 保存条目的线程来源与动态读回 |
| [UD-03-053](UPSTREAM_DELTA.md#ud-03-053) | UDX-017 | 保存条目的线程来源与动态读回 |
| [UD-04-048](UPSTREAM_DELTA.md#ud-04-048) | UDX-017 | 保存条目的线程来源与动态读回 |
| [UD-05-09](UPSTREAM_DELTA.md#ud-05-09) | UDX-017 | 保存条目的线程来源与动态读回 |
| [UD-03-040](UPSTREAM_DELTA.md#ud-03-040) | UDX-018 | 全局离线功能搜索与精确焦点 |
| [UD-03-040a](UPSTREAM_DELTA.md#ud-03-040a) | UDX-018 | 全局离线功能搜索与精确焦点 |
| [UD-03-043](UPSTREAM_DELTA.md#ud-03-043) | UDX-018 | 全局离线功能搜索与精确焦点 |
| [UD-03-047](UPSTREAM_DELTA.md#ud-03-047) | UDX-018 | 全局离线功能搜索与精确焦点 |
| [UD-03-048](UPSTREAM_DELTA.md#ud-03-048) | UDX-019 | 有界可响应引擎console |
| [UD-03-049](UPSTREAM_DELTA.md#ud-03-049) | UDX-020 | 棋局详情焦点与窗口归属 |
| [UD-03-050](UPSTREAM_DELTA.md#ud-03-050) | UDX-021 | 音频失败隔离调查 |
| [UD-03-054](UPSTREAM_DELTA.md#ud-03-054) | UDX-022 | 同树用户重点分析 |
| [UD-03-054a](UPSTREAM_DELTA.md#ud-03-054a) | UDX-022 | 同树用户重点分析 |
| [UD-03-054b](UPSTREAM_DELTA.md#ud-03-054b) | UDX-022 | 同树用户重点分析 |
| [UD-03-054c](UPSTREAM_DELTA.md#ud-03-054c) | UDX-022 | 同树用户重点分析 |
| [UD-03-054d](UPSTREAM_DELTA.md#ud-03-054d) | UDX-022 | 同树用户重点分析 |
| [UD-03-054e](UPSTREAM_DELTA.md#ud-03-054e) | UDX-022 | 同树用户重点分析 |
| [UD-03-054f](UPSTREAM_DELTA.md#ud-03-054f) | UDX-022 | 同树用户重点分析 |
| [UD-03-EXC-001](UPSTREAM_DELTA.md#ud-03-exc-001) | UDX-023 | 上游内部mechanics与发布来源排除 |
| [UD-03-EXC-002](UPSTREAM_DELTA.md#ud-03-exc-002) | UDX-023 | 上游内部mechanics与发布来源排除 |
| [UD-03-EXC-003](UPSTREAM_DELTA.md#ud-03-exc-003) | UDX-023 | 上游内部mechanics与发布来源排除 |
| [UD-03-EXC-004](UPSTREAM_DELTA.md#ud-03-exc-004) | UDX-023 | 上游内部mechanics与发布来源排除 |
| [UD-03-EXC-005](UPSTREAM_DELTA.md#ud-03-exc-005) | UDX-023 | 上游内部mechanics与发布来源排除 |
| [UD-03-EXC-006](UPSTREAM_DELTA.md#ud-03-exc-006) | UDX-023 | 上游内部mechanics与发布来源排除 |
| [UD-03-EXC-007](UPSTREAM_DELTA.md#ud-03-exc-007) | UDX-023 | 上游内部mechanics与发布来源排除 |
| [UD-04-003](UPSTREAM_DELTA.md#ud-04-003) | UDX-023 | 上游内部mechanics与发布来源排除 |
| [UD-04-005](UPSTREAM_DELTA.md#ud-04-005) | UDX-023 | 上游内部mechanics与发布来源排除 |
| [UD-04-007](UPSTREAM_DELTA.md#ud-04-007) | UDX-023 | 上游内部mechanics与发布来源排除 |
| [UD-04-008](UPSTREAM_DELTA.md#ud-04-008) | UDX-023 | 上游内部mechanics与发布来源排除 |
| [UD-04-010](UPSTREAM_DELTA.md#ud-04-010) | UDX-023 | 上游内部mechanics与发布来源排除 |
| [UD-04-014](UPSTREAM_DELTA.md#ud-04-014) | UDX-023 | 上游内部mechanics与发布来源排除 |
| [UD-04-016](UPSTREAM_DELTA.md#ud-04-016) | UDX-023 | 上游内部mechanics与发布来源排除 |
| [UD-04-018](UPSTREAM_DELTA.md#ud-04-018) | UDX-023 | 上游内部mechanics与发布来源排除 |
| [UD-04-021](UPSTREAM_DELTA.md#ud-04-021) | UDX-023 | 上游内部mechanics与发布来源排除 |
| [UD-04-023](UPSTREAM_DELTA.md#ud-04-023) | UDX-023 | 上游内部mechanics与发布来源排除 |
| [UD-04-024](UPSTREAM_DELTA.md#ud-04-024) | UDX-023 | 上游内部mechanics与发布来源排除 |
| [UD-04-025](UPSTREAM_DELTA.md#ud-04-025) | UDX-023 | 上游内部mechanics与发布来源排除 |
| [UD-04-028](UPSTREAM_DELTA.md#ud-04-028) | UDX-023 | 上游内部mechanics与发布来源排除 |
| [UD-04-035](UPSTREAM_DELTA.md#ud-04-035) | UDX-023 | 上游内部mechanics与发布来源排除 |
| [UD-05-01](UPSTREAM_DELTA.md#ud-05-01) | UDX-023 | 上游内部mechanics与发布来源排除 |
| [UD-05-04](UPSTREAM_DELTA.md#ud-05-04) | UDX-023 | 上游内部mechanics与发布来源排除 |
| [UD-04-009](UPSTREAM_DELTA.md#ud-04-009) | UDX-024 | 启动自检产品决定及measured tuning |
| [UD-04-013](UPSTREAM_DELTA.md#ud-04-013) | UDX-024 | 启动自检产品决定及measured tuning |
| [UD-04-042](UPSTREAM_DELTA.md#ud-04-042) | UDX-024 | 启动自检产品决定及measured tuning |
| [UD-04-047](UPSTREAM_DELTA.md#ud-04-047) | UDX-024 | 启动自检产品决定及measured tuning |
| [UD-04-011](UPSTREAM_DELTA.md#ud-04-011) | UDX-025 | 工作区可访问性与主题/字体目标 |
| [UD-05-03](UPSTREAM_DELTA.md#ud-05-03) | UDX-025 | 工作区可访问性与主题/字体目标 |
| [UD-04-027](UPSTREAM_DELTA.md#ud-04-027) | UDX-026 | 全树坐标/色彩变换 |
| [UD-04-032](UPSTREAM_DELTA.md#ud-04-032) | UDX-027 | 多文件分析准入与会话队列 |
| [UD-04-033](UPSTREAM_DELTA.md#ud-04-033) | UDX-028 | 双参与者runtime-komi事务 |
| [UD-04-036](UPSTREAM_DELTA.md#ud-04-036) | UDX-029 | No-engine支持路径调查 |
| [UD-04-039](UPSTREAM_DELTA.md#ud-04-039) | UDX-030 | 保存目标/快照/原子替换 |
| [UD-04-045](UPSTREAM_DELTA.md#ud-04-045) | UDX-030 | 保存目标/快照/原子替换 |
| [UD-04-044](UPSTREAM_DELTA.md#ud-04-044) | UDX-031 | LAN发布和慢客户端有界状态 |
| [UD-05-11](UPSTREAM_DELTA.md#ud-05-11) | UDX-032 | 导入日期保真调查 |
| [UD-05-12](UPSTREAM_DELTA.md#ud-05-12) | UDX-033 | 主题纹理与预设目标决定 |
| [UD-05-14](UPSTREAM_DELTA.md#ud-05-14) | UDX-034 | 外部落子意图与模式切换 |
| [UD-05-15](UPSTREAM_DELTA.md#ud-05-15) | UDX-034 | 外部落子意图与模式切换 |
| [UD-05-16](UPSTREAM_DELTA.md#ud-05-16) | UDX-035 | Fox段位继承与显式昵称/UID调查 |
| [UD-05-26](UPSTREAM_DELTA.md#ud-05-26) | UDX-035 | Fox段位继承与显式昵称/UID调查 |
| [UD-05-18](UPSTREAM_DELTA.md#ud-05-18) | UDX-036 | SGF导入容错与结果元数据调查 |
| [UD-05-19](UPSTREAM_DELTA.md#ud-05-19) | UDX-037 | 树图/变例/预览异步身份调查 |
| [UD-05-21](UPSTREAM_DELTA.md#ud-05-21) | UDX-037 | 树图/变例/预览异步身份调查 |
| [UD-05-24](UPSTREAM_DELTA.md#ud-05-24) | UDX-037 | 树图/变例/预览异步身份调查 |
| [UD-05-20](UPSTREAM_DELTA.md#ud-05-20) | UDX-038 | PDA/WRN成对参数读回 |
| [UD-05-22](UPSTREAM_DELTA.md#ud-05-22) | UDX-039 | 更新版本选择政策 |
| [UD-05-23](UPSTREAM_DELTA.md#ud-05-23) | UDX-040 | 候选列表可访问性 |

## 来源条目与当前阶段接点

Matrix新建19个Missing后继，不改变原113状态；命名决定不等于功能开工。各组的全部原 source 意图仍按公开 routes 唯一 owner 负责；下表保留条目/来源接点，当前阶段取 routes，不另造任务。

| Item | 当前阶段合同 | Source/owner coverage |
| --- | --- | --- |
| ENG-11 | [R12](R12_PLAN.md#resources)、[R12](R12_PLAN.md#models) | [完整来源组](UPSTREAM_DELTA.md#udx-002) |
| ENG-12 | [R13](R13_PLAN.md#performance)、[R12](R12_PLAN.md#manual-threads) | [完整来源组](UPSTREAM_DELTA.md#udx-017) |
| ENG-13 | [R12](R12_PLAN.md#manual-threads) | [完整来源组](UPSTREAM_DELTA.md#udx-017) |
| ENG-14 | [R13](R13_PLAN.md#performance)、[R12](R12_PLAN.md#startup) | [完整来源组](UPSTREAM_DELTA.md#udx-008) |
| ENG-15 | [R13](R13_PLAN.md#performance)、[R12](R12_PLAN.md#startup) | [完整来源组](UPSTREAM_DELTA.md#udx-024) |
| AI-01 | [R16](R16_PLAN.md#ai) | [完整来源组](UPSTREAM_DELTA.md#udx-005) |
| REVIEW-10 | [R11](R11_PLAN.md#capability-06)、[R11](R11_PLAN.md#integration-acceptance) | [完整来源组](UPSTREAM_DELTA.md#udx-006) |
| ANA-17 | [R13](R13_PLAN.md#tracking)、[R13](R13_PLAN.md#focus)、[R14](R14_PLAN.md#readboard) | [完整来源组](UPSTREAM_DELTA.md#udx-022) |
| ANA-18 | [R13](R13_PLAN.md#tracking)、[R13](R13_PLAN.md#display) | T02-TRACKING、T02-GRANULAR-DISPLAY |
| ANA-19 | [R11](R11_PLAN.md#capability-03)、[R13](R13_PLAN.md#retained-analysis)、[R13](R13_PLAN.md#autoquick)、[R12](R12_PLAN.md#lifecycle) | [完整来源组](UPSTREAM_DELTA.md#udx-011) |
| ANA-20 | [R13](R13_PLAN.md#display) | [完整来源组](UPSTREAM_DELTA.md#udx-040) |
| ANA-22 | [R12](R12_PLAN.md#runtime-readback) | [完整来源组](UPSTREAM_DELTA.md#udx-038) |
| GAME-11 | [R12](R12_PLAN.md#lifecycle)、[R15](R15_PLAN.md#rules) | [完整来源组](UPSTREAM_DELTA.md#udx-014) |
| GAME-12 | [R15](R15_PLAN.md#komi) | [完整来源组](UPSTREAM_DELTA.md#udx-028) |
| GAME-13 | [R15](R15_PLAN.md#rules)、[R15](R15_PLAN.md#rule-storage) | [完整来源组](UPSTREAM_DELTA.md#udx-014) |
| UI-07 | [R11](R11_PLAN.md#capability-01)、[R11](R11_PLAN.md#integration-acceptance) | [完整来源组](UPSTREAM_DELTA.md#udx-018) |
| UI-08 | [R12](R12_PLAN.md#diagnostics) | [完整来源组](UPSTREAM_DELTA.md#udx-019) |
| SGF-17 | [R11](R11_PLAN.md#capability-03)、[R11](R11_PLAN.md#integration-acceptance) | [完整来源组](UPSTREAM_DELTA.md#udx-030) |
| PREF-02 | [R17](R17_PLAN.md#import)、[R17](R17_PLAN.md#entries) | T02-JAVA-IMPORT |
