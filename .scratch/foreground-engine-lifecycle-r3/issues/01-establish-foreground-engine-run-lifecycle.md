# 01 — 建立 Foreground Engine Run 生命周期

**What to build:** 让用户从主工作区的 Engine Switcher 启动、停止和重启一个真实 `KataGoAnalysis` Foreground Engine Run，并始终看到由 manager 发布的权威状态。该切片贯通 manager、wire DTO、Tauri gateway、frontend API 与 rendered application，建立后续 R3 行为共用的 Run identity、不可变 snapshot、typed failure envelope 和 Run-owned Analysis Job registry。

**Blocked by:** None — can start immediately.

**Status:** done

**Guardrails:** `engine-manager` 是进程、Run identity、lifecycle operation、job registry 和取消顺序的唯一所有者。Lifecycle tagged union 从本票起固定为 No-engine、Starting、Ready、Switching、Stopping、Error，并携带单调 revision；本票只产生除 Switching 外的状态，Switching 留给事务切换。R3 只证明 `KataGoAnalysis`；不得据此接受 `ENG-09`、`ENG-10` 或 R4 分析合同。Engine Settings 只编辑 catalog，不能 Start、Switch、Stop 或执行分析；Engine Switcher 是 lifecycle intent 的主工作区入口。遗留 profile-to-process 分析调用可暂时共存，但不得拥有或伪造 Foreground Engine Run。Run、snapshot、operation 和 job identity 都是 session-only。

- [x] No-engine、Starting、Ready、Stopping 和 Error 通过同一个公开 lifecycle snapshot/event 合同贯通到 UI；snapshot revision 单调递增，UI 不再从 Profile 完整性、`canRun`、标签或本地布尔值推断进程状态。
- [x] Frontend 采用 subscribe-then-read，并用 revision 与 identity 合并初始读取和事件，订阅建立期间不会丢失或倒序应用状态。
- [x] Start 只读取已保存的 Engine Profile record；未保存表单内容不参与 Start 或 Restart，选择当前 primary Profile 不隐式 Restart，选择其他 Profile 的 Switch 行为留给 06。
- [x] Start 自动验证所需资产；手工 Check Assets 仍是诊断入口而非 Start 前置。资产存在或 spawn 成功都不能发布 Ready。
- [x] `KataGoAnalysis` 在同一常驻子进程上完成有界、最小、合法且可解析的 JSONL readiness probe 后才进入 Ready；probe 数据不进入 UI、Analysis Job 结果或 cache。
- [x] Starting 包含稳定 Profile identity、adapter kind 和不可变 Profile snapshot，但尚未准入 capability；Ready 冻结 verified capability snapshot，Stopping 与 Error 保留已经获准的 snapshot。
- [x] 保存对活动 Profile 的修改不会改变当前 Run snapshot；UI 对比保存 record 与 Run snapshot 显示 pending changes，只有显式 Restart 应用它们。
- [x] Stop 先发布 Stopping 并拒绝新 Job，再通过共用 Run-owned registry 取消该 Run 的全部 Job、执行有界 terminal drain、终止 primary 与当前 operation 所属 candidate，最后发布 No-engine；无响应的 Job 或协议不会无限阻塞。
- [x] Restart 不是 Switch：先阻止准入、取消旧 Run Job、终止旧进程，再从当前已保存 record 创建新的 opaque Run identity；同一稳定 Profile identity 也不得复用旧 Run identity，失败不得复活旧进程。
- [x] 当前 primary 进程意外退出会保留失败 Run 的不可变诊断 snapshot 并进入 Error，不自动重启；初始 Start 在尚无 primary 时失败则保持 No-engine，并发布 attempt-scoped typed failure。
- [x] 建立唯一 failure DTO，至少携带 operation、适用的 run/switch/job/profile identities、稳定 kind、用户可读 message 和可选已清理诊断摘要；后续 Autoload、manual recovery 与 switch failure 必须复用它。
- [x] 建立唯一 lane-aware Run-owned Analysis Job registry 和 cancel-all-by-run 边界；selected-node supersession 只属于该 lane，02 与 03 必须注册到此 registry，不得创建第二个 job owner。
- [x] 活动 Profile 删除约束由 catalog 后端强制，而非只禁用按钮；Ready、Stopping 与仍持有当前 Run identity 的 Error 都拒绝删除，Stop 后允许删除。
- [x] manager 暴露供应用 teardown 调用的有界取消/停止边界，但本票不实现 R5 Safe Graceful Shutdown；应用重启不恢复 Run、Error、snapshot 或 Job。
- [x] repository tests 复用现有可脚本化 JSONL 子进程 seam，黑盒证明公开 snapshot/event、readiness I/O、Stop/Restart 取消顺序、pending changes、删除 guard、Start failure、Error 且无自动重启；不建立第二套 mock process framework。
- [x] 小型 wire DTO 与 rendered application tests 证明 snake_case identity 无丢失、Switcher 与 Settings 分离、状态与可用操作真实，以及 accepted `ENG-01`、`UI-03`、`UI-04` 行为未回退。


## Answer

Foreground Engine Run lifecycle is owned by `engine-manager` and published through snake_case snapshot/event DTOs, Tauri gateway commands/events, and the Engine Switcher.

- Tagged union is No-engine / Starting / Ready / Switching / Stopping / Error plus monotonic revision. Ticket 01 produces every state except Switching.
- Start reads the saved catalog record, probes JSONL readiness on one resident process, then admits a capability snapshot. Asset miss or probe timeout stays No-engine with a typed failure. Unexpected Ready exit enters Error and does not auto-restart.
- Stop/Restart/Teardown cancel Run-owned jobs, drain, then kill. Restart mints a new run id from the current saved record. Catalog delete is rejected while Starting/Ready/Stopping/Error still hold that profile, and allowed after Stop/Teardown.
- Engine Settings is catalog/Check Assets only. Engine Switcher Start/Stop/Restart live in the main workspace. Selecting the current primary is a no-op; other-profile Switch is ticket 06.

Evidence: `cargo test -p app-model -p engine-manager`, `crates/engine-manager/tests/foreground_engine_run.rs`, `apps/desktop` vitest (`foregroundEngine`, `EngineLifecycle`, `App`). Desktop crate compile still needs host WebKit/pkg-config.
