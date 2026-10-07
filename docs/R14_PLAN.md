# R14 — 远程与外部棋盘

Status: 滚动阶段计划；产品能力验收尚未完成。

## 目标、完整范围与非目标

真实SSH、双远程算力、readboard双向、腾讯直播和弈客剩余能力在唯一current-game/Match权威内可用。

非目标：无私有协议/浏览器凭据绕路/fake endpoint；不实现R13基础focus/autoquick或复制R12local恢复。

本阶段包括以下具名能力章节及其引用的所有 source/Delta/用户断言、默认与保存、错误与取消、原生或实际服务义务。详细契约通过权威历史全文稳定引用保留，不以摘要替换契约；本文件与公开路由的 scoped-result 规则优先于历史来源排期。

<a id="ssh"></a>
## SSH 全部已准入 stdio 与 Autoload

混合local/SSH catalog稳定ID，全部验收时已准入stdio adapters各有真实SSH证据；可显式sole Autoload Default，新profile默认不标。设置不Start/Switch，live snapshots不变；host trust/auth/deadline需source/real证据，不抄Java时限。System Credential Store及明确session-only fallback、断连取消/显式Restart、failed B保留A、无local fallback或auto reconnect；SSH不套HTTP网络。

唯一能力责任：`T01-SSH`。本节设置/技术调查与后续实现是同一完整能力；跨节消费不改变来源owner。

详细来源全文（归档合同，不是活跃任务）：

- [C01 — Freeze SSH adapter, trust and cancellable deadline admission](MIGRATION_CONTRACTS.md#c01)：Produce the promotion contract for SSH Engine Profiles; no SSH transport implementation belongs here.
- [C02 — Implement SSH Engine Profiles across admitted stdio adapters](MIGRATION_CONTRACTS.md#c02)：Deliver user-managed SSH profiles as standard Foreground Engine Runs for every stdio adapter admitted at acceptance.

最小验收面与用户断言：上述全文的 Acceptance/Concrete acceptance 和 Frozen source/Canonical Delta 条款全部保留；本节 owner 用实际 changed domain/owner/protocol/persistence/UI 面证明成功、失败/Cancel、身份失效、适用 defaults/save/reopen；只读调查以有来源的可判定结论结束，不能宣称功能已完成。每个受影响设置 map 归该消费者。

<a id="compute-network"></a>
## Zhizi/Custom 远程算力与统一网络

Zhizi账户/catalog与Custom ws/wss是两种都必须证明的真实服务，独立于profile/Autoload；显式Start产出标准Foreground Run。兼容protocol、auth/deadline/service实例证据不足保留needs-info；不fake endpoint。跳过本地benchmark，正确服务model refresh和setup guidance保持完整。统一Direct/System/Manual默认Direct、policy/request revision、env/platform/NO_PROXY/redirect/trust/no-Direct-fallback随实际transport取得证据；Tencent/Yike只在新增transport确实需要时消费对应已验扩展，不等待整个remote feature。

唯一能力责任：`T01-NETWORK`, `T01-REMOTE`。本节设置/技术调查与后续实现是同一完整能力；跨节消费不改变来源owner。

详细来源全文（归档合同，不是活跃任务）：

- [C03 — Establish both remote-compute service and deadline contracts](MIGRATION_CONTRACTS.md#c03)：Freeze the separate Zhizi account/catalog and Custom ws/wss service contracts without implementing a fake remote-compute service.
- [C04 — Deliver both remote-compute modes with actual policy routes](MIGRATION_CONTRACTS.md#c04)：Deliver explicit Zhizi account/catalog and Custom ws/wss remote-compute starts into the standard Foreground Engine Run, including the first new remote NET-01 consumers.

最小验收面与用户断言：上述全文的 Acceptance/Concrete acceptance 和 Frozen source/Canonical Delta 条款全部保留；本节 owner 用实际 changed domain/owner/protocol/persistence/UI 面证明成功、失败/Cancel、身份失效、适用 defaults/save/reopen；只读调查以有来源的可判定结论结束，不能宣称功能已完成。每个受影响设置 map 归该消费者。

<a id="readboard"></a>
## readboard 只读生命周期、重点分析与外部恢复

READ-01 readiness/owned process/socket与READ-02唯一external-authoritative read-only来源，READ-03仅OCR明确不支持，不吸收sidecar lifecycle。首次frame走一次SGF-07，B/W/rebuild、可信PL/marker缺失限制、last-good/disconnect/新Ready Retry/Stop authority退休各自验证。readboard focus消费R13 local054a–c/兼容binary result；pending/pure syncing不清已接受semantic focus、identical frame不重发、return重新校验完整棋谱/history/node/turn/size/rules/komi/revision/incarnation。

唯一能力责任：`T01-READBOARD`, `T03-FOCUS-READBOARD`, `T05-SYNC-LAG-CHECK`。本节设置/技术调查与后续实现是同一完整能力；跨节消费不改变来源owner。

详细来源全文（归档合同，不是活跃任务）：

- [C05 — Integrate existing readboard functional and retirement evidence](MIGRATION_CONTRACTS.md#c05)：Integrate existing READ-01/02 and read-only retirement evidence without blocking consumers or prescribing an unevidenced runtime repair.
- [C17 — Extend passed same-tree focus to readboard evidence lifecycle](MIGRATION_CONTRACTS.md#c17)：Deliver the readboard evidence/admission extension of the B-owned ANA-17 same-tree focus contract, not a new Parity Item or remote-engine focus.
- [C18 — Check late Fox title and frame ordering without speculative fixes](MIGRATION_CONTRACTS.md#c18)：Answer whether the supported real Fox spectator path rolls back confirmed moves when the window title/recognition frame lags.
- [C19 — Check exact readboard handicap and rebuild Save/reopen semantics](MIGRATION_CONTRACTS.md#c19)：Deliver the current read-only external-board portion of exact-position restoration without blocking it on remote credentials or future GAME-10.

最小验收面与用户断言：上述全文的 Acceptance/Concrete acceptance 和 Frozen source/Canonical Delta 条款全部保留；本节 owner 用实际 changed domain/owner/protocol/persistence/UI 面证明成功、失败/Cancel、身份失效、适用 defaults/save/reopen；只读调查以有来源的可判定结论结束，不能宣称功能已完成。每个受影响设置 map 归该消费者。

<a id="providers"></a>
## 既有服务证据与狐狸身份边界

既有public import/sync证据按原tuple整合，ledger不是开工门；public browser handoff不等于auth。狐狸numeric nickname/UID/Tab/Enter边界按source-exact真实样本核查，无样本保持gap，不把规划summary误写为产品bug。Tencent fixture/native/production timeout差异完整保留。

唯一能力责任：`T01-PROVIDERS`, `T05-FOX-IDENTITY-CHECK`。本节设置/技术调查与后续实现是同一完整能力；跨节消费不改变来源owner。

详细来源全文（归档合同，不是活跃任务）：

- [C06 — Record reusable public provider import and sync outcomes](MIGRATION_CONTRACTS.md#c06)：Consolidate source-exact public-provider functional evidence without blocking direct consumption of existing PROV functionality.
- [C23 — Check Fox numeric nickname, explicit UID and keyboard identity](MIGRATION_CONTRACTS.md#c23)：Answer whether supported Fox numeric-nickname/explicit-UID lookup and keyboard choice paths preserve distinct identities.

最小验收面与用户断言：上述全文的 Acceptance/Concrete acceptance 和 Frozen source/Canonical Delta 条款全部保留；本节 owner 用实际 changed domain/owner/protocol/persistence/UI 面证明成功、失败/Cancel、身份失效、适用 defaults/save/reopen；只读调查以有来源的可判定结论结束，不能宣称功能已完成。每个受影响设置 map 归该消费者。

<a id="tencent"></a>
## 腾讯直播

腾讯/huanle live同步不同于kifu分页导入；supportedURL/protocol/auth/interval/default/settings先具来源核实，不假定WebSocket。真实取消/错误/last-good与当前服务证据不得用fixture替代。

唯一能力责任：`T02-PROV-05`, `T02-TENCENT-PROTOCOL`。本节设置/技术调查与后续实现是同一完整能力；跨节消费不改变来源owner。

详细来源全文（归档合同，不是活跃任务）：

- [C07 — Decide supported Tencent/huanle live read protocol](MIGRATION_CONTRACTS.md#c07)：Answer one question: which supported Tencent/huanle room URL families, live read protocol and authorization boundary can Next actually admit?
- [C08 — Implement admitted Tencent/huanle ongoing synchronization](MIGRATION_CONTRACTS.md#c08)：Deliver ongoing Tencent/huanle live-room synchronization for exactly the C07-supported protocol and URL families.

最小验收面与用户断言：上述全文的 Acceptance/Concrete acceptance 和 Frozen source/Canonical Delta 条款全部保留；本节 owner 用实际 changed domain/owner/protocol/persistence/UI 面证明成功、失败/Cancel、身份失效、适用 defaults/save/reopen；只读调查以有来源的可判定结论结束，不能宣称功能已完成。每个受影响设置 map 归该消费者。

<a id="yike"></a>
## 弈客 Personal 与受支持原生账号对弈

Personal guest official=4有实际page/since/filter/populated/empty/failure证据；Recommend默认，category/page session-only。若auth-required，需要显式后续plan revision，不自动偷用账号。native play仅supported authorization和唯一GAME-01 Match，auth/assigned-side/exact-position/rules/turn/dirty准入、provider clock与RE权威；Pending Provider Move精确authenticated successor、write timeout Reconciling/no retry，secret仅system store，不能抄浏览器Cookie/token/password。embedded hall/share no-op保留具名历史处置。

唯一能力责任：`T02-H26`, `T02-H27`, `T02-PERSONAL-EVIDENCE`, `T02-PROV-06`, `T02-PROV-07`, `T02-YIKE-AUTH`。本节设置/技术调查与后续实现是同一完整能力；跨节消费不改变来源owner。

详细来源全文（归档合同，不是活跃任务）：

- [C09 — Establish real guest Yike Personal semantics](MIGRATION_CONTRACTS.md#c09)：Determine whether guest signed Yike Personal official=4 yields the frozen discovery behavior without account authorization.
- [C10 — Deliver guest Personal discovery and public handoff](MIGRATION_CONTRACTS.md#c10)：Add the evidence-supported guest Personal category to the existing Yike live center, without duplicating import or ongoing sync.
- [C11 — Freeze provider-supported Yike account read/write admission](MIGRATION_CONTRACTS.md#c11)：Obtain a supportable Yike authorization/read/write contract under ADR0005, or document why native account play remains Deferred.
- [C12 — Implement one authorized Yike account and authoritative human play](MIGRATION_CONTRACTS.md#c12)：Deliver one provider-supported authorized Yike account with native human Move/Pass/Resign through the sole GAME-01 Match Session.
- [C13 — Resolve retained embedded Yike page and hall user goals](MIGRATION_CONTRACTS.md#c13)：Decide the item-specific retained user goals behind Java embedded Yike page/hall without silently removing them or implementing a new embedding stack.
- [C14 — Resolve registered no-op share shortcuts without empty actions](MIGRATION_CONTRACTS.md#c14)：Resolve the retained user goal/approval for baseline registered share shortcuts Ctrl+E and Alt+B, whose shareSGF body was a no-op.

最小验收面与用户断言：上述全文的 Acceptance/Concrete acceptance 和 Frozen source/Canonical Delta 条款全部保留；本节 owner 用实际 changed domain/owner/protocol/persistence/UI 面证明成功、失败/Cancel、身份失效、适用 defaults/save/reopen；只读调查以有来源的可判定结论结束，不能宣称功能已完成。每个受影响设置 map 归该消费者。

<a id="external-match"></a>
## GAME-10 双模式精确权威、Stop 与续走

GAME-10同时保留Final-decision completed decision与Leading-candidate first-positive-OR rank-one；双向target/semantic能力和实际两模式证明、current session/turn/Run/target、coordinate/pass/resign都仅exact authoritative successor确认才commit，ACK/同坐标不够。无额外arming/固定预览准入；second request/mode取消前者。Stop关闭admission/seal queue，已sent仅限定identity confirmation window，失败/发后divergence/stale/timeout结束且不retry/undo/rollback/auto restart。

唯一能力责任：`T01-EXTERNAL`, `T03-READ02-STOP-AND-GAME10-TURN-RETIREMENT`, `T05-EXTERNAL-CONTINUATION`, `T05-EXTERNAL-EXACT-RESTORE`, `T05-EXTERNAL-MODE-SWITCH`。本节设置/技术调查与后续实现是同一完整能力；跨节消费不改变来源owner。

详细来源全文（归档合同，不是活跃任务）：

- [C15 — Freeze readboard bidirectional capabilities and confirmation deadlines](MIGRATION_CONTRACTS.md#c15)：Produce the exact real-sidecar/target promotion contract for both GAME-10 move modes before enabling external writes.
- [C16 — Implement both externally confirmed engine move modes](MIGRATION_CONTRACTS.md#c16)：Deliver one External-board Engine Match lifecycle with both Final-decision and Leading-candidate modes and exact external authority.
- [C20 — Preserve only current authorized external Engine Continuation intent](MIGRATION_CONTRACTS.md#c20)：Deliver the external owner/continuation interleaving boundary without turning READ-02 into an engine-play owner.
- [C21 — Fence actual GAME-10 mode-switch and navigation results](MIGRATION_CONTRACTS.md#c21)：Deliver safe transitions between Final-decision and Leading-candidate external engine-turn policies under the one C16 Match.

最小验收面与用户断言：上述全文的 Acceptance/Concrete acceptance 和 Frozen source/Canonical Delta 条款全部保留；本节 owner 用实际 changed domain/owner/protocol/persistence/UI 面证明成功、失败/Cancel、身份失效、适用 defaults/save/reopen；只读调查以有来源的可判定结论结束，不能宣称功能已完成。每个受影响设置 map 归该消费者。

<a id="remote-handback"></a>
## 两服务显式和自动快析交还

两required legs：explicit-analysis与automatic-quick/task；每一腿Zhizi和Custom都需真实Run/transport/task/credentials。显式腿仅消费R12 local import-restart ownership及本阶段both-mode Run能力，自动腿另外消费R13自动admission/owner/handoff。external read-only replacement、reader lease/cache/curve/result slot、新document/Run/rules/position、disconnect/user explicitRestart/queuedcancel全部保留；explicit可先推进但不足全intent关闭。GAME-10只有实际适用腿才消费exact-confirmation；分析不能place。

唯一能力责任：`T05-REMOTE-QUICK-HANDBACK`。本节设置/技术调查与后续实现是同一完整能力；跨节消费不改变来源owner。

详细来源全文（归档合同，不是活跃任务）：

- [C22 — Validate required remote automatic-quick and explicit-analysis handback](MIGRATION_CONTRACTS.md#c22)：Deliver both mandatory remote automatic-quick/task and explicit-analysis handback outcomes with both real compute modes; independently advancing partial analysis cannot close the full intent.

最小验收面与用户断言：上述全文的 Acceptance/Concrete acceptance 和 Frozen source/Canonical Delta 条款全部保留；本节 owner 用实际 changed domain/owner/protocol/persistence/UI 面证明成功、失败/Cancel、身份失效、适用 defaults/save/reopen；只读调查以有来源的可判定结论结束，不能宣称功能已完成。每个受影响设置 map 归该消费者。

## 真实 scoped prerequisites 与跨阶段消费

阶段编号是 Delivery Order，不是 whole-stage hard dependency。原来源中已经 Accepted 的所需功能直接按原 evidence tuple 消费；ledger-only 汇总不是等待门。未来 source 的 required/conditional/field scope 在 [公开路由](MIGRATION_ROUTES.json) 和 [来源合同](MIGRATION_CONTRACTS.md) 中保留，不能扁平化成全票或全阶段开工门。

- 消费 `qualified-local-resource`（R12 / `T01-RESOURCE`）：Actual SSH/compute resources only。前置仅该已验证结果及适用条件。
- 消费 `bounded-runtime-diagnostics`（R12 / `T01-DIAGNOSTICS`）：SSH/service/account/turn secrets and failure extensions。前置仅该已验证结果及适用条件。
- 消费 `automatic-load-quick-handoff`（R13 / `T02-AUTOLOAD-QUICK`）：Required automatic quick/task leg in both Zhizi and Custom; explicit remote leg can advance without it, complete intent cannot。前置仅该已验证结果及适用条件。
- 消费 `local-import-explicit-restart`（R12 / `T05-LOCAL-IMPORT-RESTART-CHECK`）：Both required real remote handback legs; no local investigation rerun or remote credential prerequisite to local work。前置仅该已验证结果及适用条件。
- 消费 `local-focus-result`（R13 / `T03-ANALYSIS-SAME-TREE-MOVE-FOCUS`）：054d readboard extension only。前置仅该已验证结果及适用条件。
- 本阶段唯一交付 `unified-network-transport`（`T01-NETWORK`）：Passed policy transport result for actual admitted HTTP(S)/WS(S) consumer。其他消费者只取其通过scope；不接管本owner。
- 消费 `unified-network-transport`（R14 / `T01-NETWORK`）：Tencent/Yike only if admitted transport needs new extension; existing compatible NET-01 functional direct input otherwise。前置仅该已验证结果及适用条件。
- 消费 `first-ui-locale-foundation`（[R11](R11_PLAN.md#behavior-a01) / `T02-I18N-FOUNDATION`）：Actual new UI integration only; settings owner adds Java field map。前置仅该已验证结果及适用条件。

## 具名待决事项、事实输入与受阻行为

以下 owner 是阶段内具名能力责任角色，实施细化时将阶段责任落到实际执行人；不是要求本轮解决所有未来选择。来源/协议/能力未证明保持 needs-info。普通工程设计不造审批；永久非等价才要逐项产品批准。

<a id="r14-gate-c01"></a>
### C01 来源问题（非新任务）

- Owner：R14 / SSH 全部已准入 stdio 与 Autoload owner（T01-SSH）；阶段细化负责人具名落实实际执行人。
- 待决定/取得证据：SSH real adapter/platform/host trust/auth and evidence-derived bounded deadlines
- 受阻行为：Implement SSH Engine Profiles across admitted stdio adapters
- 所需结果：具来源/实际能力证据的可执行合同；涉及非等价处置保留逐项批准。
- 原事实输入与边界全文：[来源](MIGRATION_CONTRACTS.md#c01)；以下保留该事实/合同输入，不表示已经选定新答案：

> Produce the promotion contract for SSH Engine Profiles; no SSH transport implementation belongs here.
> Read the resolved historical Ticket23 and current SSH-01 row. Inventory every admitted standard-input/output adapter at the future acceptance candidate, its immutable capabilities and a real compatible remote program. SSH changes location, not adapter semantics. Record actual remote host/platform, safe credential/key-reference source, host identity/trust handling and typed connect/auth/readiness/command/disconnect failures. Use supported SSH behavior and operator-controlled hosts, not a credential or trust bypass.
> Choose bounded, cancellable connection/authentication/readiness/command/teardown deadlines from actual host/adapter evidence, stating timeout outcome and cancellation ownership. Java 3s/20s/60s constants are not defaults. Name the per-admitted-platform credential-store/native scenarios and exact available remote versions. Missing host access is a bounded unsupported/unavailable finding, not an invented acceptance.
> Output either an approved, complete promotion contract consumed by C02 or a named unavailable prerequisite retaining SSH-01 Deferred. Do not prune an adapter merely to make admission easy. New adapters later own their own SSH compatibility evidence.

<a id="r14-gate-c03"></a>
### C03 来源问题（非新任务）

- Owner：R14 / Zhizi/Custom 远程算力与统一网络 owner（T01-REMOTE）；阶段细化负责人具名落实实际执行人。
- 待决定/取得证据：Supported Zhizi and Custom protocols/credential sources/deadlines and real service availability
- 受阻行为：Deliver both remote-compute modes with actual policy routes
- 所需结果：具来源/实际能力证据的可执行合同；涉及非等价处置保留逐项批准。
- 原事实输入与边界全文：[来源](MIGRATION_CONTRACTS.md#c03)；以下保留该事实/合同输入，不表示已经选定新答案：

> Freeze the separate Zhizi account/catalog and Custom ws/wss service contracts without implementing a fake remote-compute service.
> Use historical Ticket23 and supported provider documentation/operator-authorized service access. For each mode record endpoints and protocol/version, supported authorization and credential provenance, catalog/model identifiers, connect/readiness/analysis/cancel/Stop semantics, supported adapter capabilities, network-policy routing limits, failures and real service availability. Custom retains ws/wss scope; the Zhizi catalog/auth protocol must come from actual evidence, not a guessed common API.
> From real supported services choose bounded cancellable auth/catalog/connect/readiness/protocol and stop deadlines, with explicit failure outcomes and approval. Name what credentials/authorization the future native operator needs without collecting secrets into artifacts. Provide independently testable contracts for both modes; no mode may substitute for the other.
> Output a supported two-mode promotion result or named per-mode missing evidence with RCOMP-01 retained Deferred. A unavailable service is a terminal investigation outcome, not permission to narrow permanently. No browser Cookie/token extraction, provider-secret paste workaround, private unofficial endpoint or local-only benchmark proves service admission.

<a id="r14-gate-c07"></a>
### C07 来源问题（非新任务）

- Owner：R14 / 腾讯直播 owner（T02-TENCENT-PROTOCOL）；阶段细化负责人具名落实实际执行人。
- 待决定/取得证据：Supported live URL/read protocol/auth and update/settings/default contract without assuming WebSocket
- 受阻行为：Implement admitted Tencent/huanle ongoing synchronization
- 所需结果：具来源/实际能力证据的可执行合同；涉及非等价处置保留逐项批准。
- 原事实输入与边界全文：[来源](MIGRATION_CONTRACTS.md#c07)；以下保留该事实/合同输入，不表示已经选定新答案：

> Answer one question: which supported Tencent/huanle room URL families, live read protocol and authorization boundary can Next actually admit?
> Use frozen CAP-06-ONLINE-URL and provider-supported documentation plus lawful operator-authorized service evidence. Record endpoint/protocol/version and locator families, exact game/turn/result normalization, initial fetch and ongoing-update semantics, shared timeout/cancellation/retry/recovery behavior, trust/policy representability, credential/privacy requirements and whether real service testing is possible. Do not select WebSocket merely because the legacy description says live websocket/qipu; the output protocol is evidence-derived.
> Output the supported family/protocol/promotion contract or an explicit no-supported-contract/unavailable-evidence result, preserving PROV-05 Deferred and naming exactly what would unblock C08. If official authorization is needed, record provider-supported mechanism and authorization prerequisite; no browser-secret scraping or alternate unofficial protocol. Do not implement streaming or use a fixture as real service proof.

<a id="r14-gate-c09"></a>
### C09 来源问题（非新任务）

- Owner：R14 / 弈客 Personal 与受支持原生账号对弈 owner（T02-PERSONAL-EVIDENCE）；阶段细化负责人具名落实实际执行人。
- 待决定/取得证据：Actual guest Personal populated/empty/failure/page/since/filter/handoff versus auth-required
- 受阻行为：Deliver guest Personal discovery and public handoff
- 所需结果：具来源/实际能力证据的可执行合同；涉及非等价处置保留逐项批准。
- 原事实输入与边界全文：[来源](MIGRATION_CONTRACTS.md#c09)；以下保留该事实/合同输入，不表示已经选定新答案：

> Determine whether guest signed Yike Personal official=4 yields the frozen discovery behavior without account authorization.
> Use the existing guest request with usertoken=-1 and provider-supported access. Capture sanitized populated/empty/failure outcomes, page/since continuation, client-filter semantics, duplicate/end behavior and public locator-handoff viability; distinguish authentication-required from a legitimate empty category. Never fabricate a populated result or treat Java opaque-category support as live evidence.
> Output a real guest category contract (including exact filter/page/since behavior) or a named unavailable result. If actual service proves authentication required, retain PROV-06 Deferred and require a later explicit plan revision adding T02-YIKE-AUTH/PROV-07 rather than silently acquiring account semantics. Investigation ends at that promotion decision; it does not import/sync, implement login or read browser credentials.

<a id="r14-gate-c11"></a>
### C11 来源问题（非新任务）

- Owner：R14 / 弈客 Personal 与受支持原生账号对弈 owner（T02-YIKE-AUTH）；阶段细化负责人具名落实实际执行人。
- 待决定/取得证据：Provider-supported official authorization/read/write/clock/result/initial locator families and test account
- 受阻行为：Implement one authorized Yike account and authoritative human play
- 所需结果：具来源/实际能力证据的可执行合同；涉及非等价处置保留逐项批准。
- 原事实输入与边界全文：[来源](MIGRATION_CONTRACTS.md#c11)；以下保留该事实/合同输入，不表示已经选定新答案：

> Obtain a supportable Yike authorization/read/write contract under ADR0005, or document why native account play remains Deferred.
> Use provider-supported authorization documentation and an authorized test account. No Yike password collection/persistence, pasted Cookie/token, browser-cookie reading, browser-devtools capture, guest bootstrap treated as user auth, or unofficial credential workaround. For each independently admissible recognized locator family, record official account/side/position/rules/turn reads, Move/Pass/Resign commands, exact authoritative successor/result reads, provider clock/deadline visibility, supported revocation/expiry and test-account ability.
> Freeze initial supported family set, unsupported-family explanation, secret provenance/System Credential Store reference/removal behavior, account reconnect/reauthorize protocol, shared10s request bounds and no-write-retry read-only reconciliation semantics. If an official mechanism changes these invariants, return a decision blocker requiring explicit contract revision, not an implementation bypass. No native authorization fixture counts as real account evidence.
> Output one approved provider-supported auth/read/write and family admission contract or named missing provider support/service/account/clock evidence. Stop there. Browser-mediated official authorization may be a provider-supported flow, but browser credentials never enter Next. Public system-browser Play & Sync remains available separately without proving account-native writes.

<a id="r14-gate-c13"></a>
### C13 来源问题（非新任务）

- Owner：R14 / 弈客 Personal 与受支持原生账号对弈 owner（T02-H26）；阶段细化负责人具名落实实际执行人。
- 待决定/取得证据：Item-specific approved embedded page/hall retained goal or permanent non-equivalence
- 受阻行为：仅该来源的未决保留能力/确实观察到的差异；既有行为不重开、不永久删除。
- 所需结果：具来源/实际能力证据的可执行合同；涉及非等价处置保留逐项批准。
- 原事实输入与边界全文：[来源](MIGRATION_CONTRACTS.md#c13)；以下保留该事实/合同输入，不表示已经选定新答案：

> Decide the item-specific retained user goals behind Java embedded Yike page/hall without silently removing them or implementing a new embedding stack.
> Read historical Ticket13 embedded-hosting abandonment, CAP-06-YIKE-WEB/HALL, their original approval record if present, and actual PROV-03 canonical browser handoff/retention limitations. Distinguish embedded viewing/hall navigation/observed-room auto-sync goals from system-browser login/play plus explicit read-only sync and retained hall/game/unite locators. The recorded historical redesign is not automatically full functional equivalence or sufficient item-specific approval.
> Produce either (a) an approved retained-function contract with entry/default/persistence/cancel/failure/privacy/evidence and stable owner links, or (b) item-specific approved permanent non-equivalence rationale citing actual approving source and remaining user goals. If evidence of approval or user choice is absent, record needs-info decision output and keep the goal outstanding. No JCEF port, browser credentials, page hosting implementation or arbitrary automatic-sync semantics is authorized by this ticket.
> Known public Play & Sync keeps browser authentication outside Next and Stop leaves browser open. Do not conflate that with native PROV-07 or claim the browser handoff proved website game loading.

<a id="r14-gate-c14"></a>
### C14 来源问题（非新任务）

- Owner：R14 / 弈客 Personal 与受支持原生账号对弈 owner（T02-H27）；阶段细化负责人具名落实实际执行人。
- 待决定/取得证据：Item-specific share-shortcut retained goal/destination or approved no-op abandonment
- 受阻行为：仅该来源的未决保留能力/确实观察到的差异；既有行为不重开、不永久删除。
- 所需结果：具来源/实际能力证据的可执行合同；涉及非等价处置保留逐项批准。
- 原事实输入与边界全文：[来源](MIGRATION_CONTRACTS.md#c14)；以下保留该事实/合同输入，不表示已经选定新答案：

> Resolve the retained user goal/approval for baseline registered share shortcuts Ctrl+E and Alt+B, whose shareSGF body was a no-op.
> Read CAP-06-SHARE-CURRENT, historical Ticket13 abandonment and exact approving source. Separate reachable no-op shortcuts from unregistered batch/private/public upload/menu/toolbar internals, which are not this capability. Do not treat a callable empty body as usable sharing, but do not use that fact to approve permanent deletion on the user's behalf.
> Output either item-specific approved non-equivalence/abandonment citing approval and no empty Next share actions, or a retained functional sharing contract with destinations/data privacy/entry/default/persistence/cancel/failure and evidence boundaries and parent-approved owner assignment. If approval or intended share target is unavailable, retain a named decision output as needs-info rather than inventing a clipboard/cloud/upload service.
> No product key registration/removal/upload or new sharing implementation belongs here; website/auth credentials remain out of scope. A existing independent SGF export is not automatically an approved sharing replacement.

<a id="r14-gate-c15"></a>
### C15 来源问题（非新任务）

- Owner：R14 / GAME-10 双模式精确权威、Stop 与续走 owner（T01-EXTERNAL）；阶段细化负责人具名落实实际执行人。
- 待决定/取得证据：Supported real bidirectional target/request/semantic/identity capabilities and confirmation/Stop/restoration deadlines
- 受阻行为：Implement both externally confirmed engine move modes
- 所需结果：具来源/实际能力证据的可执行合同；涉及非等价处置保留逐项批准。
- 原事实输入与边界全文：[来源](MIGRATION_CONTRACTS.md#c15)；以下保留该事实/合同输入，不表示已经选定新答案：

> Produce the exact real-sidecar/target promotion contract for both GAME-10 move modes before enabling external writes.
> Read original Ticket25, ADR0004 and existing READ-01::functional/READ-02::functional directly at their original conditions; C05 ledger completion is not a gate. Current wire220430 readiness and read-only frames do not prove placement, semantic Pass/Resign or shared turn identity. For every proposed admitted target/platform record sidecar version, target version, bidirectional coordinate/pass/resign command/events, external engine-turn request source, mode-specific engine capabilities and exact authoritative successor/terminal result evidence. Both modes remain required overall; each profile may support only one.
> Choose from real supported protocol evidence bounded cancellable computation/engine-restoration, sent-turn confirmation and Stop windows. Distinguish Stop-before-place retirement from Stop-after-place's one bounded authoritative-confirmation window. No engine output/empty ACK/command handling or matching coordinate alone proves successor. Freeze strict external single-turn legal transition criteria and unsupported semantic/identity capability rejection without guessed protocol bytes or Java3s constants.
> Output an approved supported bidirectional/identity/deadline contract plus real-target evidence plan, or exact missing sidecar/protocol/target/capability result retaining GAME-10 Deferred. No private client manipulation, credential workaround, fake target or shipped-platform invention. If current runtime does not expose required capabilities, say what supported runtime/protocol result is needed, not that GAME-10 is satisfied by one-way sync.

`T02-CONTINUATION` 材料性决定唯一 owner 在 R13/continuation（B36来源）；R14 external-match只消费其外部适用性，C20保留同一未决门，不复制决定。

## 集成、实际验收与环境义务

- 具名 owner：**R14 外部服务与权威集成/实际验收 owner**；阶段细化时明确实际负责人，拥有本阶段 changed composed workflow 的实际验收，不能交给 read-only Closeout。每个能力owner仍对其局部断言和真实surface负责。
- 环境：真实SSH host/admittedstdio adapters、Zhizi和Custom两服务及授权凭据；Windowsreadboard/实际bidirectionaltarget、支持腾讯/弈客协议账号与system store；缺支持protocol保持needs-info。
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
