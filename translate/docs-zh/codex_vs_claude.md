# 后端实现指南

在 Caipi 中实现基于 CLI 的后端的实用指南，基于协议分析和实验。

## 执行摘要

| 方面 | Claude Code | Codex CLI | 影响 |
|------|-------------|-----------|------|
| 权限流程 | 双向 JSON | 仅预设标志 | Codex 无法进行每操作提示 |
| 思考事件 | 在消息内容中 | 单独的项目类型 | 不同的解析逻辑 |
| 工具事件 | 统一的 tool_use | 不同的项目类型 | 需要单独的处理程序 |
| 会话恢复 | `--resume <id>` | `resume <id>` | 仅语法差异 |

**关键发现：** Codex exec 模式不支持运行时权限请求。所有权限必须在执行前通过 `--sandbox` 和批准标志配置。

---

## 第 1 阶段：Claude CLI 包装器

为 Claude Code 构建直接 CLI 包装器，在添加 Codex 之前验证模式。

### 1.1 模块结构

```
src-tauri/src/backends/
├── mod.rs                 # 公共导出
├── types.rs               # Backend/BackendSession 特征（存在）
├── session.rs             # BackendSession 特征（存在）
├── process.rs             # 新增：CliProcess 实用程序
├── claude/
│   ├── mod.rs
│   ├── adapter.rs         # ClaudeBackend（存在）
│   ├── sdk.rs             # 当前的基于 SDK（从 agent.rs 重命名）
│   └── cli/               # 新增
│       ├── mod.rs         # ClaudeCliSession
│       ├── process.rs     # 进程管理
│       ├── protocol.rs    # 消息类型
│       └── control.rs     # 控制协议
└── codex/                 # 未来
    ├── mod.rs
    ├── adapter.rs
    └── cli/
        ├── mod.rs
        └── protocol.rs
```

### 1.2 共享 CliProcess

```rust
// src-tauri/src/backends/process.rs

use tokio::process::{Child, Command};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};

pub struct CliProcess {
    child: Child,
    stdin: BufWriter<tokio::process::ChildStdin>,
    stdout_reader: tokio::io::Lines<BufReader<tokio::process::ChildStdout>>,
}

impl CliProcess {
    pub async fn spawn(
        program: &str,
        args: &[&str],
        cwd: &Path,
        env: &[(&str, &str)],
    ) -> Result<Self, Error>;

    pub async fn write_line(&mut self, json: &str) -> Result<(), Error>;

    pub async fn read_line(&mut self) -> Option<Result<String, Error>>;

    pub fn kill(&mut self) -> Result<(), Error>;

    pub fn try_wait(&mut self) -> Result<Option<ExitStatus>, Error>;
}
```

### 1.3 Claude 协议类型

```rust
// src-tauri/src/backends/claude/cli/protocol.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum ClaudeMessage {
    #[serde(rename = "system")]
    System(SystemMessage),
    #[serde(rename = "assistant")]
    Assistant(AssistantMessage),
    #[serde(rename = "user")]
    User(UserMessage),
    #[serde(rename = "result")]
    Result(ResultMessage),
    #[serde(rename = "control_request")]
    ControlRequest(IncomingControlRequest),
    #[serde(rename = "control_response")]
    ControlResponse(ControlResponseMessage),
}

#[derive(Debug, Deserialize)]
pub struct SystemMessage {
    pub subtype: String,  // "init"
    pub session_id: String,
    pub cwd: String,
    pub model: String,
    #[serde(rename = "permissionMode")]
    pub permission_mode: String,
    #[serde(rename = "apiKeySource")]
    pub api_key_source: String,
    pub tools: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct AssistantMessage {
    pub message: AssistantMessageContent,
    pub session_id: String,
}

#[derive(Debug, Deserialize)]
pub struct AssistantMessageContent {
    pub model: String,
    pub id: String,
    pub content: Vec<ContentBlock>,
    pub usage: Option<Usage>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    #[serde(rename = "thinking")]
    Thinking {
        thinking: String,
    },
}

// 注意：这些 UserMessage 类型是简化的占位符。
// 真实 JSONL 有更深的嵌套：message.content[].tool_use_id 等。
// 捕获实际 JSONL 固件以完善这些结构。
#[derive(Debug, Deserialize)]
pub struct UserMessage {
    pub message: UserMessageContent,
    pub tool_use_result: Option<ToolUseResult>,
}

#[derive(Debug, Deserialize)]
pub struct ToolUseResult {
    #[serde(rename = "type")]
    pub result_type: String,  // "create", "update", "delete"
    #[serde(rename = "filePath")]
    pub file_path: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ResultMessage {
    pub subtype: String,  // "success" 或 "error"
    pub is_error: bool,
    pub session_id: String,
    pub result: String,
    pub duration_ms: u64,
    pub num_turns: u32,
    pub total_cost_usd: Option<f64>,
}

// 控制协议类型
#[derive(Debug, Deserialize)]
pub struct IncomingControlRequest {
    pub request_id: String,
    pub request: ControlRequestData,
}

#[derive(Debug, Deserialize)]
pub struct ControlRequestData {
    pub subtype: String,  // "hook_callback"
    pub callback_id: Option<String>,
    pub input: Option<HookInput>,
}

#[derive(Debug, Deserialize)]
pub struct HookInput {
    pub hook_event_name: String,
    pub tool_name: Option<String>,
    pub tool_input: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct ControlResponse {
    #[serde(rename = "type")]
    pub msg_type: String,  // "control_response"
    pub response: ControlResponseData,
}

#[derive(Debug, Serialize)]
pub struct ControlResponseData {
    pub subtype: String,  // "success"
    pub request_id: String,
    pub response: HookResponse,
}

#[derive(Debug, Serialize)]
pub struct HookResponse {
    #[serde(rename = "hookSpecificOutput")]
    pub hook_specific_output: HookSpecificOutput,
}

#[derive(Debug, Serialize)]
pub struct HookSpecificOutput {
    #[serde(rename = "hookEventName")]
    pub hook_event_name: String,
    #[serde(rename = "permissionDecision")]
    pub permission_decision: String,  // "allow", "deny"
}
```

### 1.4 控制协议实现

```rust
// src-tauri/src/backends/claude/cli/control.rs

impl ClaudeCliSession {
    /// 使用权限处理的钩子初始化 CLI
    async fn initialize(&mut self) -> Result<(), Error> {
        let request_id = Uuid::new_v4().to_string();
        let init_request = json!({
            "type": "control_request",
            "request_id": request_id,  // 关联响应所必需
            "request": {
                "subtype": "initialize",
                "hooks": {
                    "PreToolUse": [{
                        "matcher": "*",
                        "hookCallbackIds": ["pretool_0"]
                    }],
                    "PostToolUse": [{
                        "matcher": "*",
                        "hookCallbackIds": ["posttool_0"]
                    }]
                }
            }
        });

        self.process.write_line(&init_request.to_string()).await?;

        // 等待初始化响应
        while let Some(line) = self.process.read_line().await {
            let msg: ClaudeMessage = serde_json::from_str(&line?)?;
            if matches!(msg, ClaudeMessage::ControlResponse(_)) {
                break;
            }
        }

        Ok(())
    }

    /// 处理传入的控制请求（权限回调）
    async fn handle_control_request(
        &mut self,
        request: IncomingControlRequest,
        permission_callback: impl Fn(&str, &serde_json::Value) -> BoxFuture<'static, bool>,
    ) -> Result<(), Error> {
        let input = request.request.input.ok_or(Error::MissingInput)?;
        let tool_name = input.tool_name.unwrap_or_default();
        let tool_input = input.tool_input.unwrap_or(json!({}));

        // 调用权限回调（这触发 UI 提示）
        let allowed = permission_callback(&tool_name, &tool_input).await;

        // 发送响应
        let response = ControlResponse {
            msg_type: "control_response".to_string(),
            response: ControlResponseData {
                subtype: "success".to_string(),
                request_id: request.request_id,
                response: HookResponse {
                    hook_specific_output: HookSpecificOutput {
                        hook_event_name: input.hook_event_name,
                        permission_decision: if allowed { "allow" } else { "deny" }.to_string(),
                    },
                },
            },
        };

        self.process.write_line(&serde_json::to_string(&response)?).await?;
        Ok(())
    }
}
```

### 1.5 事件转换

```rust
// src-tauri/src/backends/claude/cli/mod.rs

impl ClaudeCliSession {
    fn convert_to_chat_event(&self, msg: ClaudeMessage) -> Vec<ChatEvent> {
        match msg {
            ClaudeMessage::System(sys) if sys.subtype == "init" => {
                vec![ChatEvent::SessionInit {
                    auth_type: match sys.api_key_source.as_str() {
                        "none" => "Claude AI Subscription".to_string(),
                        _ => "Anthropic API Key".to_string(),
                    },
                }]
            }

            ClaudeMessage::Assistant(ast) => {
                let mut events = vec![];

                for block in ast.message.content {
                    match block {
                        ContentBlock::Text { text } => {
                            events.push(ChatEvent::Text { content: text });
                        }
                        ContentBlock::ToolUse { id, name, input } => {
                            // 注意：当前 Caipi SDK 集成从
                            // PreToolUse 钩子发出 ToolStart（不是这里）以显示 awaiting_permission/running
                            // 转换。对于 CLI 直接，您可能想要：
                            // 1. 从钩子回调发出（如 SDK 所做），或
                            // 2. 在此处发出但与钩子触发的事件去重
                            events.push(ChatEvent::ToolStart {
                                tool_use_id: id,
                                tool_type: name.clone(),
                                target: extract_tool_target(&name, &input),
                                status: "pending".to_string(),
                                input: Some(input),
                            });
                        }
                        ContentBlock::Thinking { thinking } => {
                            events.push(ChatEvent::ThinkingStart {
                                thinking_id: uuid::Uuid::new_v4().to_string(),
                                content: thinking,
                            });
                        }
                    }
                }

                if let Some(usage) = ast.message.usage {
                    events.push(ChatEvent::TokenUsage {
                        total_tokens: usage.input_tokens + usage.output_tokens,
                    });
                }

                events
            }

            ClaudeMessage::User(usr) => {
                let mut events = vec![];

                if let Some(result) = usr.tool_use_result {
                    // 从消息内容中提取 tool_use_id
                    if let Some(content) = usr.message.content.first() {
                        if let Some(id) = content.get("tool_use_id").and_then(|v| v.as_str()) {
                            events.push(ChatEvent::ToolEnd {
                                id: id.to_string(),
                                status: "completed".to_string(),
                            });
                        }
                    }
                }

                events
            }

            ClaudeMessage::Result(res) => {
                if res.is_error {
                    vec![ChatEvent::Error {
                        message: res.result,
                    }]
                } else {
                    vec![ChatEvent::Complete]
                }
            }

            _ => vec![],
        }
    }
}
```

---

## 第 2 阶段：验证

### 2.1 功能标志

```rust
// src-tauri/src/backends/claude/mod.rs

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ClaudeImplementation {
    Sdk,       // 当前：使用 claude-agent-sdk-rs
    CliDirect, // 新增：直接 CLI 包装器
}

impl ClaudeBackend {
    pub fn new(implementation: ClaudeImplementation) -> Self {
        Self { implementation }
    }
}
```

### 2.2 验证测试用例

使用相同输入运行两个实现：

| 测试 | 预期 |
|------|------|
| 简单文本响应 | 相同的文本输出 |
| 工具使用（写入文件）| 创建相同的文件，相同的事件 |
| 权限提示 | 两者都触发 UI，都正确响应 |
| 会话恢复 | 两者都从同一点继续 |
| 中止中途 | 两者都干净停止 |
| 错误处理 | 相同的错误类型 |

---

## 第 3 阶段：Codex 后端

### 3.1 协议类型

```rust
// src-tauri/src/backends/codex/cli/protocol.rs

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum CodexEvent {
    #[serde(rename = "thread.started")]
    ThreadStarted { thread_id: String },

    #[serde(rename = "turn.started")]
    TurnStarted,

    #[serde(rename = "item.started")]
    ItemStarted { item: CodexItem },

    #[serde(rename = "item.completed")]
    ItemCompleted { item: CodexItem },

    #[serde(rename = "turn.completed")]
    TurnCompleted { usage: CodexUsage },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum CodexItem {
    #[serde(rename = "reasoning")]
    Reasoning {
        id: String,
        text: String,
    },

    #[serde(rename = "command_execution")]
    CommandExecution {
        id: String,
        command: String,
        aggregated_output: String,
        exit_code: Option<i32>,
        status: String,  // "in_progress", "completed", "failed"
    },

    #[serde(rename = "file_change")]
    FileChange {
        id: String,
        changes: Vec<FileChangeEntry>,
        status: String,
    },

    #[serde(rename = "agent_message")]
    AgentMessage {
        id: String,
        text: String,
    },
}

#[derive(Debug, Deserialize)]
pub struct FileChangeEntry {
    pub path: String,
    pub kind: String,  // "create", "update", "delete"
}

#[derive(Debug, Deserialize)]
pub struct CodexUsage {
    pub input_tokens: u64,
    pub cached_input_tokens: Option<u64>,
    pub output_tokens: u64,
}
```

### 3.2 事件转换

```rust
// src-tauri/src/backends/codex/cli/mod.rs

impl CodexCliSession {
    fn convert_to_chat_event(&self, event: CodexEvent) -> Vec<ChatEvent> {
        match event {
            CodexEvent::ThreadStarted { thread_id } => {
                self.thread_id = Some(thread_id);
                vec![ChatEvent::SessionInit {
                    auth_type: "OpenAI API".to_string(),
                }]
            }

            CodexEvent::ItemStarted { item } => {
                match item {
                    CodexItem::CommandExecution { id, command, .. } => {
                        vec![ChatEvent::ToolStart {
                            tool_use_id: id,
                            tool_type: "Bash".to_string(),
                            target: command,
                            status: "running".to_string(),
                            input: None,
                        }]
                    }
                    _ => vec![],
                }
            }

            CodexEvent::ItemCompleted { item } => {
                match item {
                    CodexItem::Reasoning { id, text } => {
                        vec![
                            ChatEvent::ThinkingStart {
                                thinking_id: id.clone(),
                                content: text,
                            },
                            ChatEvent::ThinkingEnd { thinking_id: id },
                        ]
                    }

                    CodexItem::CommandExecution { id, status, .. } => {
                        vec![ChatEvent::ToolEnd {
                            id,
                            status: if status == "completed" { "completed" } else { "error" }.to_string(),
                        }]
                    }

                    CodexItem::FileChange { id, changes, .. } => {
                        let target = changes
                            .first()
                            .map(|c| c.path.clone())
                            .unwrap_or_default();

                        vec![
                            ChatEvent::ToolStart {
                                tool_use_id: id.clone(),
                                tool_type: "Edit".to_string(),
                                target,
                                status: "completed".to_string(),
                                input: None,
                            },
                            ChatEvent::ToolEnd {
                                id,
                                status: "completed".to_string(),
                            },
                        ]
                    }

                    CodexItem::AgentMessage { text, .. } => {
                        vec![ChatEvent::Text { content: text }]
                    }
                }
            }

            CodexEvent::TurnCompleted { usage } => {
                vec![
                    ChatEvent::TokenUsage {
                        total_tokens: usage.input_tokens + usage.output_tokens,
                    },
                    ChatEvent::Complete,
                ]
            }

            _ => vec![],
        }
    }
}
```

### 3.3 权限处理（有限）

```rust
// src-tauri/src/backends/codex/cli/mod.rs

impl CodexCliSession {
    /// 根据权限模式构建 CLI 参数
    fn build_args(&self, config: &SessionConfig) -> Vec<String> {
        let mut args = vec![
            "exec".to_string(),
            "--json".to_string(),
        ];

        // 将 Caipi 权限模式映射到 Codex 沙箱/批准设置
        match config.permission_mode.as_str() {
            "bypassPermissions" => {
                // 完全自动模式 - 无提示
                args.push("--dangerously-bypass-approvals-and-sandbox".to_string());
            }
            "acceptEdits" => {
                // 自动批准文件编辑，沙箱命令
                args.push("--full-auto".to_string());
            }
            "default" | _ => {
                // Workspace-write 沙箱 - 允许项目中的文件编辑
                // 注意：在 exec 模式下无法程序化地进行每操作提示
                // （批准策略存在但表现为 TTY 提示，导致挂起）
                args.push("--sandbox".to_string());
                args.push("workspace-write".to_string());
            }
        }

        if let Some(model) = &config.model {
            args.push("--model".to_string());
            args.push(model.clone());
        }

        if let Some(dir) = &config.folder_path {
            args.push("-C".to_string());
            args.push(dir.clone());
        }

        args
    }
}
```

**重要限制：** Codex 在 exec 模式下没有程序化权限回调机制。虽然批准策略（`-a/--ask-for-approval`）存在，但它们表现为 TTY 提示，会导致 GUI 包装器挂起。权限 UI 不会出现在 Codex 会话中 - 必须通过沙箱模式选择预先配置权限。

---

## 主要差异摘要

| 功能 | Claude CLI 包装器 | Codex CLI 包装器 |
|------|-----------------|-----------------|
| 权限提示 | 是（控制协议）| 否（程序化回调不可用）|
| 思考可见性 | 从内容块解析 | 单独的 `reasoning` 事件 |
| ToolStart 来源 | PreToolUse 钩子回调（首选）或 `tool_use` 块 | `item.started` 事件 |
| ToolEnd 来源 | 带有 `tool_result` 的 `user` 消息 | `item.completed` 事件 |
| 会话 ID | 消息中的 `session_id` | thread.started 中的 `thread_id` |
| 恢复命令 | `--resume <id>` | `resume <id>` 子命令 |

---

## 测试清单

- [ ] Claude CLI：简单文本响应
- [ ] Claude CLI：带权限提示的工具使用
- [ ] Claude CLI：权限被拒绝流程
- [ ] Claude CLI：会话恢复
- [ ] Claude CLI：中止处理
- [ ] Claude CLI：扩展思考
- [ ] Claude CLI：错误处理
- [ ] Codex CLI：简单文本响应
- [ ] Codex CLI：命令执行
- [ ] Codex CLI：文件更改事件
- [ ] Codex CLI：推理事件
- [ ] Codex CLI：会话恢复
- [ ] Codex CLI：不同的沙箱模式

---

## 风险缓解

| 风险 | 缓解措施 |
|------|----------|
| 控制协议更改 | 固定 CLI 版本，监控更新日志 |
| Codex 协议未记录 | 使用实验数据，广泛测试 |
| 权限 UX 不匹配 | 在 UI 中记录 Codex 限制 |
| 性能差异 | 基准测试，优化解析 |

---

## 参考资料

- [CLI 协议参考](./codex_vs_claude.md) - 详细事件格式
- [Claude Agent SDK 源代码](https://github.com/pietz/claude-agent-sdk-rs) - SDK 实现参考
