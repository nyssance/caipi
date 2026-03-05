# Claude CLI 后端统一评审

日期：2026-02-05
范围：分阶段的 Claude CLI 后端推出（`claudecli`）及后续修复。

## 方法
- 比较 SDK 后端（`src-tauri/src/claude/agent.rs`）与 CLI 后端（`src-tauri/src/backends/claude/cli_adapter.rs`）。
- 根据当前源代码验证声明，而不仅仅是暂存的差异。
- 使用真实 CLI 输出运行协议合理性检查。
- 在 `src-tauri` 中运行 `cargo test -q`（通过）。

## 执行摘要
- 架构稳固且可扩展。
- 之前两个关键问题已修复（中止上下文丢失、重复 ToolEnd）。
- `claudecli` 的令牌处理现在使用助手使用字段来表示上下文使用。

## 发现（按优先级排序）

### P1

### P2

2. 无进程崩溃信号 / 过时会话进程状态风险
- 状态：开放
- 来源：两个评审
- 证据：
  - 读取器任务在 EOF/解析路径上静默退出，无崩溃事件：`src-tauri/src/backends/claude/cli_adapter.rs:295`
  - `self.process` 仅在显式中止/清理路径中清除：`src-tauri/src/backends/claude/cli_adapter.rs:919`、`src-tauri/src/backends/claude/cli_adapter.rs:940`
- 影响：
  - 静默后端死亡；下一次发送可能会模糊地失败。
- 建议：
  - 监控子进程退出（`wait`）并发出 `ChatEvent::Error`；在意外退出时原子性地清除进程/stdin。

3. 思考级别切换在 CLI 后端中是无操作
- 状态：开放
- 来源：两个评审
- 证据：
  - CLI 后端无操作：`src-tauri/src/backends/claude/cli_adapter.rs:969`
  - UI 全局公开思考切换：`src/lib/components/chat/MessageInput.svelte:57`
- 影响：
  - 用户可见的平价差距；设置看起来有效但实际上没有作用。
- 建议：
  - 实现真正的控制，或者为 `claudecli` 禁用/隐藏切换并传达限制。

4. 交互式工具在运行时被拒绝，而不是在模型规划时被禁止
- 状态：开放
- 来源：外部评审 + 已验证
- 证据：
  - SDK 提前禁止：`src-tauri/src/claude/agent.rs:227`
  - CLI 在权限钩子路径中拒绝：`src-tauri/src/claude/hooks.rs:59`
- 影响：
  - 额外的工具尝试和令牌浪费；潜在的模型混淆。
- 建议：
  - 如果可用，传递 CLI 级别的禁止标志；否则保持拒绝但调整提示/系统指令。

5. 会话中间的权限模式更新可能不完全匹配生成时的绕过语义
- 状态：开放（边缘情况风险）
- 来源：外部评审 + 已验证
- 证据：
  - `set_permission_mode` 仅更新本地状态：`src-tauri/src/backends/claude/cli_adapter.rs:955`
  - `--dangerously-skip-permissions` 仅基于模式在生成时设置：`src-tauri/src/backends/claude/cli_adapter.rs:209`
- 影响：
  - 模式切换后 CLI 内部权限行为的潜在差异。
- 建议：
  - 选项 A：模式切换时重新生成进程以实现强平价。
  - 选项 B：将模式更改语义记录为仅钩子级别。

### P3

6. 结果错误上的通用错误消息
- 状态：开放
- 来源：外部评审 + 已验证
- 证据：
  - 发出硬编码消息：`src-tauri/src/backends/claude/cli_adapter.rs:832`
  - `ResultEvent` 省略潜在的 `result/errors` 负载字段：`src-tauri/src/claude/cli_protocol.rs:252`
- 影响：
  - 排除 UX 差。
- 建议：
  - 扩展 `ResultEvent` 解析以获取实际错误文本并显示它。

7. 会话 ID 回退发送字面量 `"default"`
- 状态：开放（低置信度）
- 来源：外部评审 + 已验证
- 证据：
  - `session_id: ...unwrap_or("default")`：`src-tauri/src/backends/claude/cli_adapter.rs:409`
- 影响：
  - 如果 CLI 特殊对待此值，则协议模糊。
- 建议：
  - 未知时省略 `session_id`，而不是发送合成值。

8. 跨模块重复的 `PermissionDecision` 类型名称
- 状态：开放（可维护性）
- 来源：外部评审 + 已验证
- 证据：
  - 钩子中的语义权限决策枚举：`src-tauri/src/claude/hooks.rs:32`
  - CLI 协议遗留部分中的协议枚举：`src-tauri/src/claude/cli_protocol.rs:698`
- 影响：
  - 开发人员混淆/导入错误。
- 建议：
  - 重命名一个类型或从活动模块中删除遗留协议类型。

9. `setting_sources` 平价差距（SDK 显式，CLI 隐式）
- 状态：开放（低严重性）
- 来源：外部评审 + 已验证
- 证据：
  - SDK 设置 `SettingSource::User, Project`：`src-tauri/src/claude/agent.rs:234`
  - CLI 路径没有等效的显式标志。
- 影响：
  - 如果 CLI 默认值更改，主要是文档/平价风险。
- 建议：
  - 记录预期的假设或如果支持则传递显式等效项。

10. `--verbose` 解析风险
- 状态：未重现，保留为观察项
- 来源：外部评审
- 证据：
  - CLI 适配器需要 `--verbose`：`src-tauri/src/backends/claude/cli_adapter.rs:199`
  - 真实探针在测试运行中仅发出 JSON stdout。
- 影响：
  - 低；已经容忍解析错误。
- 建议：
  - 保留解析错误遥测；仅当生产日志显示频繁解析删除时才重新访问。

## 已解决的发现

1. 中止不再在下一条消息上丢弃上下文
- 状态：已解决
- 证据：
  - 中止后的重新生成现在在存在先前 CLI 会话 id 时恢复：`src-tauri/src/backends/claude/cli_adapter.rs:890`

2. 来自 `PostToolUse` + `ToolResult` 的重复 `ToolEnd`
- 状态：已解决
- 证据：
  - `PostToolUse` 现在仅 ACK：`src-tauri/src/backends/claude/cli_adapter.rs:635`

3. Opus 映射/版本平价
- 状态：已解决
- 证据：
  - SDK 将 `opus` 映射到 `claude-opus-4-6`：`src-tauri/src/claude/agent.rs:95`
  - CLI 探针确认模型可用性（`model":"claude-opus-4-6"`）。

4. 通过 `user.tool_result` 完成工具时缺少 `ToolEnd`
- 状态：已解决
- 证据：
  - `CliEvent::User` 现已处理：`src-tauri/src/backends/claude/cli_adapter.rs`
  - 通过 `active_tools` 在用户/助手 tool_result 变体之间对 `ToolEnd` 进行去重。

5. `claudecli` 上下文指示器的令牌使用语义
- 状态：已解决
- 决策：
  - `claudecli` 现在根据每次 API 调用的助手 `usage` 发出令牌使用：
    `input_tokens + cache_read_input_tokens + cache_creation_input_tokens`。
  - 它不再用累积的 `result` 总数覆盖此值。
- 证据：
  - CLI 适配器现在从 `AssistantEvent.message.usage` 计算使用情况。
  - 结果事件令牌发出已在 `src-tauri/src/backends/claude/cli_adapter.rs` 中移除。

## 已拒绝 / 不适用

1. 缺少 `claude:permission_request` 事件
- 状态：已拒绝
- 原因：
  - 当前架构在 SDK 和 CLI 流中都使用 `ChatEvent::ToolStatusUpdate { status: "awaiting_permission" }`，前端已为此路径连接。
  - 证据 SDK 钩子：`src-tauri/src/claude/hooks.rs:229`
  - 证据 CLI 钩子：`src-tauri/src/backends/claude/cli_adapter.rs:759`

2. `--replay-user-messages` 的缺席本身是坏的
- 状态：作为独立声明被拒绝
- 原因：
  - 在测试的 CLI 版本中，`user` tool_result 事件在正常的 `-p` stream-json 输出中没有该标志的情况下出现。
  - 真正的问题是 `CliEvent::User` 当前在适配器逻辑中被忽略。

## 建议修复顺序
1. 添加子进程退出/错误传播并清除过时的进程句柄（P2）。
2. 决定并实现后端特定的思考切换行为（P2）。
