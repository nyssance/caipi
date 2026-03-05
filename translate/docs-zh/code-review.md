# Caipi 代码审查 — 统一报告

**日期：** 2026-02-18
**范围：** 完整代码库（前端 + 后端），排除参考目录
**代理：** 5 个 Opus 审查者（Rust 后端、前端-后端契约、前端状态/响应式、跨领域、模式一致性）

## 执行摘要

**70 条原始发现**，5 个代理，去重后约 **46 条唯一发现**。代码库总体结构良好 — 后端抽象、事件系统和存储架构合理。主要主题：

1. **资源生命周期差距** — 生成的任务没有中止句柄，即发即弃的清理
2. **Svelte 5 响应式陷阱** — `$state` 中的原生 Map/Set，$effect 模式
3. **安全面** — CSP 已禁用，通配符 fs 范围
4. **代码重复** — Codex 工具解析存在于两个位置
5. **不一致的模式** — Claude vs Codex 适配器对称性，错误处理策略

三个发现由 3 个代理独立发现（Codex 重复、死命令、存储 TOCTOU），对此有很高的信心。

---

## 按优先级划分的发现

### P0 — 关键（1）

| # | 发现 | 风险 | 工作量 | 客观性 | 领域 |
|---|---------|------|--------|--------|------|
| 1 | ~~**Codex 工具解析在 `cli_protocol.rs` 和 `sessions.rs` 之间重复** — 3 个逐字辅助函数 + 1 个结构重复。协议更改必须在 2 个地方进行；忘记一个会导致静默历史/实时差异。~~ ✅ | M | L | H | Rust |

*由 3 个代理独立发现。*

---

### P1 — 高（10）

| # | 发现 | 风险 | 工作量 | 客观性 | 领域 |
|---|---------|------|--------|--------|------|
| 2 | ~~**生成的 tokio 任务没有中止句柄** — stdout/stderr 读取器和进程监视器是 `tokio::spawn`ed 而不存储 `JoinHandle`。中止后，过时的事件仍可能发出。~~ ✅ | M | M | H | Rust |
| 3 | ~~**窗口关闭清理是即发即弃** — 清理任务已生成但应用程序立即退出。孤立的 CLI 进程继续运行并消耗 API 配额。~~ ✅ | M | M | H | Rust |
| 4 | ~~**CSP 已禁用**（`"csp": null`）— 无内容安全策略。结合 `@html` markdown 渲染，XSS 将拥有完整的 Tauri IPC 访问权限。~~ ✅ | M | M | H | 安全 |
| 5 | ~~**通配符文件系统范围**（`"path": "**"`）— 前端对系统上的每个文件都有读/写访问权限。应限定为 `$HOME`/`$APPDATA`。~~ ✅ | M | M | H | 安全 |
| 6 | ~~**`$state` 与原生 `Map`** — 聊天存储中的 `tools` 使用 `new Map()` 而不是 `SvelteMap`。通过 `.set()` 的突变不会触发响应式。目前偶然安全（完整替换模式），但脆弱。~~ ✅ | M | M | H | 前端 |
| 7 | ~~**`$state` 与原生 `Set`** — 文件存储中的 `expanded` 和 `SessionPicker` 使用 `new Set()` 而不是 `SvelteSet`。与上述相同的陷阱。~~ ✅ | M | L | H | 前端 |
| 8 | ~~**`SessionPicker.$effect` 无条件运行 `loadSessions()`** — 无显式依赖，应该是 `onMount`。可能会在不相关的状态更改上导致虚假重新获取。~~ ✅ | L | L | H | 前端 |
| 9 | ~~**`cleanup()` 语义在后端之间不同** — Claude 静默终止进程；Codex 发出 `AbortComplete` 事件并休眠 500ms。调用者期望静默拆除但 Codex 触发 UI 状态更改。~~ ✅ | M | M | H | Rust |
| 10 | ~~**同一个模块树中有两个不同的 `PermissionDecision` 枚举** — `hooks.rs` 和 `cli_protocol.rs` 都定义具有不同形状的 `PermissionDecision`。需要在导入站点消除歧义。~~ ✅ | L | L | H | Rust |
| 11 | ~~**`send_control_response` 返回 `String` 错误** 而同一适配器中的所有其他 I/O 返回 `BackendError`。强制调用站点进行不一致的错误处理。~~ ✅ | L | L | H | Rust |

---

### P2 — 中（22）

| # | 发现 | 风险 | 工作量 | 客观性 | 领域 |
|---|---------|------|--------|--------|------|
| 12 | ~~**11 个 Tauri 命令已注册但从未从前端调用** — `check_cli_status`、`check_cli_installed`、`check_cli_authenticated`、`check_backend_cli_*`、`reset_onboarding`、`set_default_folder`、`get_default_backend`、`get_cli_path`、`set_cli_path`、`get_session_messages`。不必要地扩展了 IPC 攻击面约 30%。~~ ✅ | L | L | H | 跨领域 |
| 13 | ~~**损坏的 `data.json` 破坏应用程序** — 解析错误返回 `Err` 而不是 `AppData::default()`。无自愈；用户必须手动删除文件。~~ ✅ | L | L | M | Rust |
| 14 | ~~**Codex `send_message` 泄漏单次通道** — `turn/start` 响应接收器立即被丢弃但发送者永远留在 `pending_requests` 中。每轮次轻微内存泄漏。~~ ✅ | L | L | H | Rust |
| 15 | **权限模式和工具状态是裸 `String`** — 与像 `"bypassPermissions"` 这样的魔术字面量进行比较。拼写错误是静默错误。应该是枚举。 | L | M | M | Rust |
| 16 | ~~**存储读取不持有锁** — 几个读取操作跳过互斥锁而写入持有它。`get_license()` 迁移有 TOCTOU 窗口。~~ ✅ | L | L | H | Rust |
| 17 | **Claude 中止阻塞 tokio 任务 500ms** — 中止路径中的顺序休眠延迟 UI 响应。Codex 具有相同的模式。 | L | L | H | Rust |
| 18 | ~~**Codex 进程监视器在休眠期间持有互斥锁** — 在 `tokio::time::sleep` 之前未显式删除锁。Claude 适配器确实会删除它。~~ ✅ | L | L | H | Rust |
| 19 | **Codex `agent_message` 完成可能会双重发出文本** — 注释说"如果我们尚未接收增量"但不存在实际检查。 | M | L | M | Rust |
| 20 | **`respond_permission` 忽略 `sessionId`** — 参数被接受但未使用（`_session_id`）。没有验证请求属于声称的会话。 | M | M | H | 契约 |
| 21 | ~~**`StartupInfo.backendCliPaths` 类型为可选但始终存在** — Rust 字段始终序列化（为 `{}`），TS 标记为 `?`。~~ ✅ | L | L | H | 契约 |
| 22 | **`ToolCallStack` $effect 读取和写入 `revealedIds`** — 需要 `untrack()` 以防止不必要的重新运行。两个实例。 | M | M | M | 前端 |
| 23 | ~~**`MessageInput` 中的权限/模型/思考 API 调用上缺少 `.catch()`** — 后端失败时未处理的 promise 拒绝。~~ ✅ | L | L | H | 前端 |
| 24 | **不稳定的 `{#each}` 键** — `groupedStreamItems` 按数组索引键控。插入导致错误的 DOM 重用。 | L | L | H | 前端 |
| 25 | **没有针对 `files.svelte.ts` 存储的测试覆盖** — `updateChildren` 进行递归树遍历，容易出现边缘情况错误。 | L | M | H | 前端 |
| 26 | ~~**功能中缺少 OS 插件权限** — 使用了 `@tauri-apps/plugin-os` 的 `platform()` 但未声明 `os:` 权限。~~ ✅ | M | L | H | 配置 |
| 27 | ~~**打开器范围缺少 Windows 路径** — `/tmp/**` 在 Windows 上无效；`$HOME` 外的项目被阻止。~~ ✅ | M | L | H | 平台 |
| 28 | **后端特征上广泛的 `#[allow(dead_code)]`** — 11 个注释位于设计但未使用的抽象层上。`setup.rs` 中存在并行代码路径。 | L | L | H | Rust |
| 29 | **SessionInit 发出一次（Claude）vs 每轮次（Codex）** — 相同事件类型的不同生命周期语义。 | M | M | H | 模式 |
| 30 | ~~**Codex stderr 仅在调试构建中记录** — Claude 始终记录。生产 Codex 问题被静默吞没。~~ ✅ | L | L | H | 模式 |
| 31 | **Updater 存储使用闭包模式** 而所有其他存储使用类模式 — 唯一异常。 | L | M | L | 模式 |
| 32 | **模型注册表在前端和后端中定义** — `BackendCapabilities.available_models` 从未被前端使用，前端有自己的配置。 | L | L | M | 模式 |
| 33 | **Codex 批准响应静默吞没 I/O 错误** — 手动序列化，使用 `let _ = ...` 而不是使用 `write_line` 方法。 | L | L | H | 模式 |

---

### P3 — 低（13）

| # | 发现 | 风险 | 工作量 | 客观性 | 领域 |
|---|---------|------|--------|--------|------|
| 34 | **`SystemSubtype`/`ResultSubtype` 枚举已定义但从未使用** — 改为使用字符串比较。 | L | L | H | Rust |
| 35 | **`ChatEvent` serde 在 `AbortComplete` 上有冗余重命名** — 重命名为它已经是的样子。缺少显式 `rename_all`。 | L | L | M | Rust |
| 36 | **`lib.rs` 中不必要的 `pub mod claude`** — 仅在内部使用。 | L | L | L | Rust |
| 37 | **`Option<T>` 序列化为 `null` 但 TS 使用 `T \| undefined`** — 通过真值检查工作但在 `strictNullChecks` 下不正确。 | L | L | M | 契约 |
| 38 | **`AbortComplete` 在信封和负载中携带冗余 `sessionId`** — 负载副本从未被读取。 | L | L | M | 契约 |
| 39 | **`list_directory` 绕过集中式 API 包装器** — 仅从组件直接调用的命令。 | L | L | M | 契约 |
| 40 | **中止时双重状态重置** — `finalize()` 然后 `setStreaming(false)` 执行相同的清理两次。 | L | L | H | 前端 |
| 41 | **`ThemeStore.destroy()` 是死代码** — 存在但从未在单例上调用。 | L | L | H | 前端 |
| 42 | **`ToolConfig` 中未使用的 `className` 属性** — 在 20+ 配置上定义，从未被任何组件读取。 | L | L | H | 前端 |
| 43 | **多个 `setTimeout` 调用没有清理** — `LicenseEntry`、`SetupWizard`、`ToolCallStack` 设置计时器但在销毁时不清除。 | L | L | M | 前端 |
| 44 | **使用 `Date.now()` 作为流项目 ID** — 如果在同一毫秒内创建了两个项目，理论上有冲突。 | L | L | M | 前端 |
| 45 | **发布脚本中的硬编码路径** — `/Users/pietz/Private/caipi.ai` 仅在一台机器上工作。 | L | L | H | 构建 |
| 46 | **不一致的错误显示模式** — 每个屏幕以不同方式处理错误（内联文本、图标、样式容器）。 | L | M | L | 前端 |

---

## 建议行动计划

### 第 1 阶段：快速见效（< 1 天，高影响）

这些是低工作量、高客观性的修复：

| # | 操作 | 工作量 | 状态 |
|---|--------|--------|--------|
| 1 | 将 Codex 工具解析去重到共享模块 | L | ✅ 完成 |
| 6-7 | 将存储中的原生 `Map`/`Set` 替换为 `SvelteMap`/`SvelteSet` | L-M | ✅ 完成 |
| 8 | 将 `SessionPicker.$effect` 更改为 `onMount` | L | ✅ 完成 |
| 12 | 从 `generate_handler!` 中删除 11 个未使用的 Tauri 命令 | L | ✅ 完成 |
| 14 | 不为 `turn/start` 注册 `pending_request`（不需要响应）| L | ✅ 完成 |
| 16 | 向存储读取添加锁，特别是 `get_license` 迁移 | L | ✅ 完成 |
| 18 | 在 Codex 进程监视器中添加显式 `drop(guard)` | L | ✅ 完成 |
| 23 | 在 `MessageInput` 中的即发即弃 API 调用上添加 `.catch()` | L | ✅ 完成 |
| 10 | 将 `cli_protocol::PermissionDecision` 重命名为 `CliPermissionDecision` | L | ✅ 完成 |
| 11 | 将 `send_control_response` 更改为返回 `BackendError` | L | ✅ 完成 |

### 第 2 阶段：重要改进（1-2 天）

| # | 操作 | 工作量 |
|---|--------|--------|
| 4-5 | 添加 CSP 并限定文件系统权限 ✅ | M |
| 2 | 存储 `JoinHandle` 并在清理时中止生成的任务 ✅ | M |
| 3 | 实现正确的窗口关闭清理（等待会话）✅ | M |
| 9 | 在 Codex 适配器中分离 `cleanup()` 和 `abort()` ✅ | M |
| 15 | 定义 `PermissionMode` 和 `ToolStatus` 枚举 | M |
| 13 | 在损坏的 `data.json` 上返回 `AppData::default()` ✅ | L |
| 26 | 向功能添加 `os:default` 权限 ✅ | L |
| 27 | 修复 Windows 路径的打开器范围 ✅ | L |
| 30 | 使 Codex stderr 记录与 Claude 匹配（始终开启）✅ | L |

### 第 3 阶段：润色（方便时）

所有 P3，加上较低优先级的 P2 项目，如测试覆盖、updater 存储模式、模型注册表清理。

---

## 跨代理收敛

三个发现由多个代理独立发现，表明高置信度：

| 发现 | 代理 | 一致性 |
|---------|--------|-----------|
| Codex 工具解析重复 | Rust、跨领域、模式 | 3/5 |
| 死 Tauri 命令 | 契约、跨领域 | 2/5 |
| 存储读取 TOCTOU | Rust、跨领域 | 2/5 |
| `pub mod claude` | Rust、跨领域 | 2/5 |

---

## 哪些运作良好

代理还注意到优势领域：

- **ChatEvent 契约稳固** — 所有变体在 Rust 枚举和 TypeScript 联合之间匹配，包括字段名称、类型和可选性
- **所有 Tauri 调用参数名称匹配** — 针对 20+ 命令验证了自动 camelCase-to-snake_case 转换
- **后端模型名称一致** — 前端配置和 Rust 功能定义匹配的模型 ID
- **权限往返正确** — 基于 UUID 的请求 ID、60 秒超时、中止处理全部实现
- **事件信封过滤工作** — 会话/轮次 ID 阻止过时事件
- **存储原子写入** — 临时文件 + 持久化模式防止部分写入
- **测试基础设施** — 基于重放的测试、行为测试以及事件系统的良好固件覆盖
