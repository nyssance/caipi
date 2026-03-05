# 后端统一 - CLI 协议参考

Claude Code 和 Codex CLI 的完整协议文档，基于实验验证（2026 年 2 月）。

## 快速参考

| 方面 | Claude Code | Codex CLI |
|------|-------------|-----------|
| 非交互模式 | `claude -p` | `codex exec` |
| JSON 流式输出 | `--output-format stream-json --verbose` | `--json` |
| 输入格式 | `--input-format stream-json`（stdin）| 提示词作为参数或 stdin 使用 `-` |
| 权限控制 | `--permission-mode` + 双向 JSON 回调 | `--sandbox` + 预设（无程序化回调）|
| 会话 ID 来源 | init 事件中的 `session_id` | thread.started 事件中的 `thread_id` |

---

## 1. 进程启动

### Claude Code

```bash
claude -p \
  --output-format stream-json \
  --verbose \
  --input-format stream-json \
  [--model <model>] \
  [--permission-mode <mode>] \
  [--resume <session-id>]
```

**GUI 包装器必需标志：**
- `-p`（打印模式）- 非交互式
- `--output-format stream-json` - 结构化事件输出
- `--verbose` - 使用 stream-json 时必需

**可选标志：**
- `--input-format stream-json` - 启用双向 JSON 通信
- `--model sonnet|opus|haiku` - 模型选择
- `--permission-mode default|acceptEdits|bypassPermissions` - 权限处理
- `--resume <session-id>` - 继续之前的会话

### Codex CLI

```bash
codex exec \
  --json \
  [--model <model>] \
  [--sandbox <mode>] \
  [--full-auto] \
  [-C <directory>] \
  "prompt"
```

**GUI 包装器必需标志：**
- `--json` - JSONL 事件输出

**可选标志：**
- `--model gpt-5-codex|gpt-5|gpt-5.1-codex-max` - 模型选择
- `--sandbox read-only|workspace-write|danger-full-access` - 沙箱级别
- `--full-auto` - 便捷预设（workspace-write + 按需批准）
- `-C <dir>` - 工作目录

---

## 2. 事件流格式

### Claude Code 事件

所有事件都是 JSON 对象，每行一个在 stdout 上。

#### 系统初始化事件
```json
{
  "type": "system",
  "subtype": "init",
  "cwd": "/path/to/project",
  "session_id": "uuid",
  "tools": ["Read", "Write", "Bash", ...],
  "model": "claude-opus-4-5-20251101",
  "permissionMode": "default",
  "apiKeySource": "none|api_key",
  "claude_code_version": "2.1.31"
}
```

#### 助手消息事件
```json
{
  "type": "assistant",
  "message": {
    "model": "claude-opus-4-5-20251101",
    "id": "msg_xxx",
    "role": "assistant",
    "content": [
      {"type": "text", "text": "响应文本..."},
      {"type": "tool_use", "id": "toolu_xxx", "name": "Write", "input": {...}}
    ],
    "usage": {
      "input_tokens": 100,
      "output_tokens": 50,
      "cache_read_input_tokens": 1000,
      "cache_creation_input_tokens": 500
    }
  },
  "session_id": "uuid"
}
```

#### 工具结果事件（用户消息）
```json
{
  "type": "user",
  "message": {
    "role": "user",
    "content": [
      {
        "tool_use_id": "toolu_xxx",
        "type": "tool_result",
        "content": "工具输出文本"
      }
    ]
  },
  "tool_use_result": {
    "type": "create|update|delete",
    "filePath": "/path/to/file",
    "content": "适用的文件内容"
  }
}
```

#### 结果事件
```json
{
  "type": "result",
  "subtype": "success|error",
  "is_error": false,
  "duration_ms": 5000,
  "num_turns": 3,
  "result": "最终响应文本",
  "session_id": "uuid",
  "total_cost_usd": 0.05,
  "usage": {...}
}
```

### Codex CLI 事件

所有事件都是 JSON 对象，每行一个在 stdout 上。

#### 线程启动事件
```json
{
  "type": "thread.started",
  "thread_id": "uuid"
}
```

#### 轮次启动事件
```json
{
  "type": "turn.started"
}
```

#### 推理事件（思考）
```json
{
  "type": "item.completed",
  "item": {
    "id": "item_0",
    "type": "reasoning",
    "text": "思考方法..."
  }
}
```

#### 命令执行事件
```json
// 已启动
{
  "type": "item.started",
  "item": {
    "id": "item_1",
    "type": "command_execution",
    "command": "/bin/zsh -lc 'ls -la'",
    "aggregated_output": "",
    "exit_code": null,
    "status": "in_progress"
  }
}

// 已完成（成功）
{
  "type": "item.completed",
  "item": {
    "id": "item_1",
    "type": "command_execution",
    "command": "/bin/zsh -lc 'ls -la'",
    "aggregated_output": "file1.txt\nfile2.txt\n",
    "exit_code": 0,
    "status": "completed"
  }
}

// 已完成（失败）
{
  "type": "item.completed",
  "item": {
    "id": "item_1",
    "type": "command_execution",
    "command": "/bin/zsh -lc 'cat nonexistent.txt'",
    "aggregated_output": "cat: nonexistent.txt: No such file or directory\n",
    "exit_code": 1,
    "status": "failed"
  }
}
```

#### 文件更改事件
```json
{
  "type": "item.completed",
  "item": {
    "id": "item_3",
    "type": "file_change",
    "changes": [
      {"path": "/path/to/file.txt", "kind": "update"}
    ],
    "status": "completed"
  }
}
```

#### 代理消息事件
```json
{
  "type": "item.completed",
  "item": {
    "id": "item_5",
    "type": "agent_message",
    "text": "我已完成任务..."
  }
}
```

#### 轮次完成事件
```json
{
  "type": "turn.completed",
  "usage": {
    "input_tokens": 14172,
    "cached_input_tokens": 4352,
    "output_tokens": 127
  }
}
```

---

## 3. 事件到 ChatEvent 的映射

| ChatEvent | Claude Code 来源 | Codex CLI 来源 |
|-----------|-----------------|----------------|
| `SessionInit` | `system` + `subtype: init` | `thread.started` |
| `Text` | 带文本内容块的 `assistant` 消息 | 带有 `type: agent_message` 的 `item.completed` |
| `ThinkingStart/End` | 带思考块的 `assistant` 消息 | 带有 `type: reasoning` 的 `item.completed` |
| `ToolStart` | PreToolUse 钩子回调* 或 `tool_use` 块 | 带有 `type: command_execution/file_change` 的 `item.started` |
| `ToolEnd` | 带有 `tool_result` 的 `user` 消息 | 带状态的 `item.completed` |
| `TokenUsage` | `assistant.message.usage` 或 `result.usage` | `turn.completed.usage` |
| `Complete` | 带有 `subtype: success` 的 `result` | `turn.completed` |
| `Error` | 带有 `subtype: error` 的 `result` | 带有 `status: failed` 的 `item.completed`（部分）|

*当前 Caipi 从 PreToolUse 钩子发出 `ToolStart`（不是从解析 `tool_use` 块），以便在执行前显示 `awaiting_permission`/`running` 状态转换。

---

## 4. 权限系统

### Claude Code：双向 JSON 协议

Claude Code 通过双向 JSON 通信支持**运行时权限请求**。

#### 初始化（发送到 stdin）
```json
{
  "type": "control_request",
  "request_id": "init_001",
  "request": {
    "subtype": "initialize",
    "hooks": {
      "PreToolUse": [
        {
          "matcher": "*",
          "hookCallbackIds": ["hook_0"]
        }
      ]
    }
  }
}
```

**注意：** `request_id` 是关联响应所必需的。CLI 将发回带有相同 `request_id` 的 `control_response`。

#### 权限请求（从 stdout 接收）
```json
{
  "type": "control_request",
  "request_id": "req_123",
  "request": {
    "subtype": "hook_callback",
    "callback_id": "hook_0",
    "input": {
      "hook_event_name": "PreToolUse",
      "tool_name": "Bash",
      "tool_input": {"command": "rm -rf /tmp/test"}
    }
  }
}
```

#### 权限响应（发送到 stdin）
```json
{
  "type": "control_response",
  "response": {
    "subtype": "success",
    "request_id": "req_123",
    "response": {
      "hookSpecificOutput": {
        "hookEventName": "PreToolUse",
        "permissionDecision": "allow"
      }
    }
  }
}
```

**权限决策：** `"allow"`、`"deny"`、`"ask"`（升级到用户）

### Codex CLI：无程序化权限回调

Codex CLI 在 exec 模式下**不**支持程序化运行时权限回调。虽然批准策略存在（`-a/--ask-for-approval`），但它们表现为 TTY 提示，会导致 GUI 包装器挂起。

**沙箱模式：**
- `read-only`（默认）- 无文件修改
- `workspace-write` - 可以修改工作区中的文件
- `danger-full-access` - 完全系统访问

**批准策略（可用但无法程序化使用）：**
- `untrusted` - 询问所有内容（TTY 提示，导致挂起）
- `on-failure` - 仅在失败时询问（TTY 提示）
- `on-request` - 模型决定何时询问（TTY 提示）
- `never` - 从不询问（自动化安全）

**对于 GUI 包装器：** 使用 `--full-auto`（workspace-write + on-request）或预先配置沙箱级别。无法程序化服务每操作权限提示 - CLI 将挂起等待 TTY 输入。

---

## 5. 会话管理

### Claude Code

```bash
# 新会话
claude -p "prompt"

# 按 ID 恢复
claude -p --resume "session-id" "follow-up"

# 继续上一个会话
claude -p --continue "follow-up"
```

**会话 ID 提取：**
```javascript
// 从 init 事件
const sessionId = event.session_id; // 当 event.type === "system" && event.subtype === "init"

// 或从结果事件
const sessionId = event.session_id; // 当 event.type === "result"
```

### Codex CLI

```bash
# 新会话
codex exec "prompt"

# 按 ID 恢复
codex exec resume <thread-id> "follow-up"

# 继续上一个会话
codex exec resume --last "follow-up"
```

**线程 ID 提取：**
```javascript
// 从 thread.started 事件
const threadId = event.thread_id; // 当 event.type === "thread.started"
```

---

## 6. 中止/中断

### Claude Code

向进程发送 SIGINT 或使用 SDK 的中止机制。CLI 响应带有指示中断的 `stop_reason` 的结果事件。

### Codex CLI

向进程发送 SIGINT。轮次将以当前状态完成。

---

## 7. 错误处理

### Claude Code

错误出现在结果事件中：
```json
{
  "type": "result",
  "subtype": "error",
  "is_error": true,
  "result": "错误消息"
}
```

### Codex CLI

工具失败出现在 item.completed 事件中：
```json
{
  "type": "item.completed",
  "item": {
    "type": "command_execution",
    "status": "failed",
    "exit_code": 1,
    "aggregated_output": "错误输出"
  }
}
```

会话级错误可能显示为单独的错误事件（未完全记录）。

---

## 8. 模型选择

### Claude Code

| 模型 | 标志值 | 备注 |
|------|--------|------|
| Claude Opus 4.5 | `opus` 或 `claude-opus-4-5-20251101` | 最强能力 |
| Claude Sonnet 4.5 | `sonnet` 或 `claude-sonnet-4-5-20250929` | 默认 |
| Claude Haiku 4.5 | `haiku` 或 `claude-haiku-4-5-20250929` | 最快 |

### Codex CLI

| 模型 | 标志值 | 备注 |
|------|--------|------|
| GPT-5 Codex | `gpt-5-codex` | 默认（macOS/Linux）|
| GPT-5 | `gpt-5` | 默认（Windows）|
| GPT-5.1 Codex Max | `gpt-5.1-codex-max` | 最强能力 |
| GPT-4.1 Mini | `gpt-4.1-mini` | 轻量级 |

---

## 9. 已知差异

| 功能 | Claude Code | Codex CLI |
|------|-------------|-----------|
| 程序化权限 | 是（control_request/response 回调）| 否（批准策略存在但需要 TTY）|
| 思考/推理事件 | 在助手消息内容中 | 单独的 `reasoning` 项目类型 |
| 工具类型 | 统一的 tool_use 块 | 不同的项目类型（command_execution、file_change 等）|
| ToolStart 来源 | PreToolUse 钩子或 tool_use 解析 | item.started 事件 |
| 会话持久化 | 可选（`--no-session-persistence`）| 始终启用 |
| 工作目录 | 从 shell 继承 | 可以使用 `-C` 标志设置 |
| MCP 集成 | 内置 `--mcp-config` | 通过 `codex mcp` 命令 |

---

## 10. 验证的 CLI 版本

- Claude Code: 2.1.31
- Codex CLI: 0.96.0

最后验证：2026 年 2 月
