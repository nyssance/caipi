# 多窗口 / 多会话功能

在 Caipi 中运行多个并发聊天会话的设计文档。

## 当前架构（单会话）

```
用户输入 → Tauri 命令 → Rust BackendSession → CLI 进程 → stdout JSON 流
                                                                       ↓
                                                              emit_chat_event()
                                                                       ↓
                                                          app_handle.emit("chat:event")  ← 广播到所有窗口
                                                                       ↓
                                                          前端 listen("chat:event")
                                                                       ↓
                                                          shouldIgnoreEvent() 按 session_id 过滤
                                                                       ↓
                                                          单例聊天存储更新 UI
```

**关键组件：**
- `SessionStore` — `HashMap<String, Arc<dyn BackendSession>>`（已经支持按 ID 进行多个会话）
- `runtime.rs` 中的 `emit_chat_event()` — 使用 `app_handle.emit()`（全局广播）
- `ChatEventEnvelope` — 用 `session_id` 和 `turn_id` 元数据包装事件
- `events.ts` 中的 `shouldIgnoreEvent()` — 客户端过滤器，将 `event.sessionId` 与 `app.sessionId` 进行比较
- 单例存储 — `app`、`chat`、`files` 是模块级单例
- 模块级全局变量 — `events.ts` 中的 `lineBuffer` 和 `flushTimer`
- 事件监听器 — `ChatContainer.svelte` 中的全局 `listen<ChatEvent>('chat:event', ...)`

---

## 方法 A：多窗口（推荐）

每个会话在自己的原生操作系统窗口中运行。Tauri 的 `emit_to()` 将事件直接路由到正确的窗口。每个窗口都有一个独立的 JavaScript 运行时，因此所有存储和全局变量自然都是分离的。

### 为什么这样做有效

每个 Tauri webview 窗口都是一个**独立的 JS 执行上下文**。这意味着：
- 单例 `app`、`chat`、`files` 存储每个窗口都有自己的实例 — 不需要重构
- 模块级的 `lineBuffer` 和 `flushTimer` 自然隔离 — 不需要每个会话的映射
- `localStorage` 在窗口之间共享（同源），因此持久化设置会自动同步

### 架构

```
窗口 A (session-abc)                          窗口 B (session-xyz)
┌─────────────────────┐                        ┌─────────────────────┐
│ 自己的 JS 运行时    │                        │ 自己的 JS 运行时    │
│ 自己的 app/chat/files │                        │ 自己的 app/chat/files │
│ 自己的 lineBuffer    │                        │ 自己的 lineBuffer    │
│                      │                        │                      │
│ getCurrent().listen()│                        │ getCurrent().listen()│
│ "chat:event"         │                        │ "chat:event"         │
└────────▲─────────────┘                        └────────▲─────────────┘
         │ emit_to("session-abc")                        │ emit_to("session-xyz")
         │                                               │
┌────────┴───────────────────────────────────────────────┴─────────────┐
│                        Rust 后端                                   │
│                                                                       │
│  SessionStore: HashMap<String, Arc<dyn BackendSession>>               │
│  WindowRegistry: HashMap<String, String>  (session_id → window_label) │
│                                                                       │
│  emit_chat_event() → 查找 window_label → emit_to(label, ...)      │
└───────────────────────────────────────────────────────────────────────┘
```

### 所需更改

#### Rust 后端

**1. 窗口注册表**（新状态）

```rust
// 新类型：映射 session_id → window_label
pub type WindowRegistry = Arc<Mutex<HashMap<String, String>>>;
```

通过 `.manage(WindowRegistry::default())` 在 Tauri 构建器中注册。

**2. `emit_chat_event()` — 目标发射**（`runtime.rs`）

```rust
pub fn emit_chat_event(
    app_handle: &AppHandle,
    session_id: Option<&str>,
    turn_id: Option<&str>,
    event: &ChatEvent,
) {
    let payload = ChatEventEnvelope { session_id, turn_id, event };

    // 首先尝试目标发射，回退到广播
    if let Some(sid) = session_id {
        if let Some(registry) = app_handle.try_state::<WindowRegistry>() {
            if let Ok(map) = registry.lock() {
                if let Some(label) = map.get(sid) {
                    let _ = app_handle.emit_to(label, CHAT_EVENT_CHANNEL, &payload);
                    return;
                }
            }
        }
    }
    // 回退：广播（向后兼容）
    let _ = app_handle.emit(CHAT_EVENT_CHANNEL, &payload);
}
```

**3. `create_session` 命令 — 注册窗口映射**

创建会话时，前端传递其窗口标签。后端注册 `session_id → window_label` 映射。

**4. `create_window` 命令**（新增）

```rust
#[tauri::command]
async fn create_window(app: tauri::AppHandle) -> Result<String, String> {
    let label = format!("chat-{}", uuid::Uuid::new_v4());
    WebviewWindowBuilder::new(&app, &label, WebviewUrl::App("index.html".into()))
        .title("Caipi")
        .inner_size(900.0, 640.0)
        .hidden_title(true)
        .title_bar_style(TitleBarStyle::Overlay)
        // ... 匹配现有窗口配置
        .build()
        .map_err(|e| e.to_string())?;
    Ok(label)
}
```

**5. 窗口关闭清理**

监听窗口关闭事件以清理会话并从窗口注册表中取消注册。

#### 前端

**1. 事件监听器 — 作用于当前窗口**（`ChatContainer.svelte`）

```typescript
// 之前（广播到所有窗口）：
unlisten = await listen<ChatEvent>('chat:event', handler);

// 之后（仅接收此窗口的事件）：
import { getCurrent } from '@tauri-apps/api/webviewWindow';
unlisten = await getCurrent().listen<ChatEvent>('chat:event', handler);
```

**2. 会话创建 — 传递窗口标签**

```typescript
import { getCurrent } from '@tauri-apps/api/webviewWindow';
const windowLabel = getCurrent().label;
// 将 windowLabel 传递给 create_session，以便后端可以注册映射
```

**3. "新窗口"操作**

调用 `create_window` 命令的菜单项、键盘快捷键（Cmd+N）或按钮。新窗口加载相同的 SvelteKit 应用程序并经历正常的启动流程（文件夹选择器 → 会话创建）。

#### 功能（`default.json`）

```json
{
  "windows": ["main", "chat-*"],
  "permissions": [
    "core:webview:allow-create-webview-window",
    // ... 现有权限
  ]
}
```

### 窗口间的共享状态

某些状态确实需要在窗口之间同步：

| 状态 | 机制 | 备注 |
|-------|-----------|-------|
| 设置（模型、权限、主题）| `localStorage`（共享源）| 每个窗口在启动时读取；更改自动可见 |
| 许可证状态 | `localStorage` 或 Rust 管理的状态 | 窗口创建时检查 |
| 默认后端 / CLI 路径 | `localStorage` | 可以每个窗口覆盖 |
| "哪些会话打开？" | Rust `WindowRegistry` | 通过 Tauri 命令查询 |

对于实时同步（例如，在一个窗口中更改主题会更新所有窗口），使用 `app_handle.emit()`（广播）进行专门的 `"settings:changed"` 事件通道 — 与会话范围的聊天事件分开。

### 窗口生命周期

| 事件 | 行为 |
|-------|----------|
| 新窗口 | 生成 webview，经过文件夹选择器或使用指定的文件夹打开 |
| 窗口关闭 | 销毁会话，从 WindowRegistry 中删除，清理 CLI 进程 |
| 最后一个窗口关闭 | 应用程序退出（默认 macOS/Windows 行为）|
| 会话结束（完成）| 窗口保持打开，用户可以开始新会话或关闭 |
| 应用程序退出（Cmd+Q）| 所有窗口关闭，所有会话清理 |

### 优势

- **零存储重构** — 每个窗口隔离的 JS 运行时
- **零事件多路复用** — Rust 中的 `emit_to` 处理路由
- **现有 UI 不变** — 无标签栏、无组件参数化
- **操作系统级窗口管理** — 快照、平铺、多显示器、全屏
- **小的 Rust 差异** — 窗口注册表 + `emit` → `emit_to` + 新命令
- **增量式** — 可以仅使用"新窗口"发布并迭代

### 风险和缓解措施

| 风险 | 缓解措施 |
|------|------------|
| 每个 webview 的内存（每个约 50-100MB）| 对于桌面应用可以接受；大多数用户将有 2-4 个窗口 |
| 窗口之间的设置漂移 | 使用 `localStorage` + 广播事件进行实时同步 |
| 重启时窗口状态不持久化 | 将打开的窗口/文件夹保存到 localStorage；下次启动时恢复 |
| Windows 上的窗口死锁 | 始终使用异步命令创建窗口（Tauri 文档警告）|

---

## 方法 B：基于标签（单窗口）

带有标签栏的单个 Tauri 窗口。前端通过多路复用事件来管理多个会话。

### 架构

```
┌──────────────────────────────────────────────────┐
│                 单个窗口                      │
│  ┌──────────┬──────────┬──────────┐               │
│  │ 标签 A    │ 标签 B    │ 标签 C    │  ← 标签栏    │
│  └──────────┴──────────┴──────────┘               │
│  ┌────────────────────────────────────────────┐   │
│  │            活动标签内容               │   │
│  │     （呈现所选会话的聊天）            │   │
│  └────────────────────────────────────────────┘   │
│                                                    │
│  SessionManager: Map<sessionId, {                  │
│    chat: ChatState,                                │
│    files: FilesState,                              │
│    lineBuffer: string,                             │
│    flushTimer: number                              │
│  }>                                                │
│                                                    │
│  事件调度器：                                  │
│    listen("chat:event") → 按 sessionId 路由       │
│    → 更新正确的 ChatState                      │
└──────────────────────────────────────────────────┘
```

### 所需更改

#### 前端（主要重构）

**1. 每个会话的存储实例**

单例 `chat` 和 `files` 存储必须成为每个会话实例化的工厂或类：

```typescript
class SessionManager {
  sessions = $state<Map<string, SessionContext>>(new Map());
  activeSessionId = $state<string | null>(null);

  get activeSession(): SessionContext | undefined {
    return this.activeSessionId ? this.sessions.get(this.activeSessionId) : undefined;
  }
}

interface SessionContext {
  chat: ChatState;
  files: FilesState;
  lineBuffer: string;
  flushTimer: ReturnType<typeof setTimeout> | null;
}
```

**2. 事件调度器**

用路由器替换直接 `handleClaudeEvent`，该路由器查找正确的 `SessionContext`：

```typescript
listen<ChatEvent>('chat:event', (event) => {
  const sessionId = event.payload.sessionId;
  const ctx = sessionManager.sessions.get(sessionId);
  if (!ctx) return;
  handleClaudeEvent(event.payload, ctx.chat, ctx.lineBuffer, ctx.flushTimer, ...);
});
```

**3. 组件参数化**

`ChatContainer`、`MessageList` 和相关组件必须接受 `ChatState` 属性而不是导入全局单例。

**4. 标签栏组件**（新增）

用于在会话之间切换的新 UI 组件，显示会话名称、关闭按钮、拖放重新排序。

**5. 模块全局变量消除**

`events.ts` 中的 `lineBuffer`、`flushTimer`、`onContentChange` 必须移动到 `SessionContext` 中。

#### Rust 后端

最小更改 — 前端处理路由，因此广播 `app_handle.emit()` 仍然有效。可以选择保留 `shouldIgnoreEvent()` 作为额外的安全措施。

### 优势

- **单个 webview** — 更低的内存占用
- **更简单的标签间 UX** — 拖放会话，一目了然地查看所有标签
- **更简单的窗口生命周期** — 一个窗口 = 一个应用程序
- **不需要 Tauri 多窗口 API** — 纯前端工作

### 风险和缓解措施

| 风险 | 缓解措施 |
|------|------------|
| 大规模前端重构 | 分阶段进行：首先存储，然后 UI，最后润色 |
| 所有事件在单线程中处理 | `shouldIgnoreEvent` 很快；仅活动标签渲染 |
| 单例存储假设分散在代码库中 | 审计所有 `chat`/`files` 单例的导入 |
| `events.ts` 全局变量是隐式耦合 | 提取到 `SessionContext` 类 |
| 后台标签累积不可见的状态 | 限制为 N 个标签；显示内存警告 |

---

## 比较

| 维度 | 多窗口（A）| 标签（B）|
|-----------|-----------------|----------|
| **存储重构** | 无 | 主要（单例 → 每个会话）|
| **事件路由** | Rust 中的 `emit_to`（1 个函数更改）| 前端调度器（新层）|
| **UI 更改** | 仅"新窗口"按钮/快捷键 | 标签栏 + 组件参数化 |
| **Rust 更改** | 窗口注册表 + `emit_to` + 新命令 | 最小 |
| **前端更改** | 作用域监听器 + 窗口标签传递 | 广泛重构 |
| **JS 隔离** | 免费（独立运行时）| 手动（每个会话状态映射）|
| **每个会话的内存** | ~50-100MB（完整 webview）| ~5-10MB（仅 JS 状态）|
| **操作系统集成** | 原生窗口管理 | 无 |
| **并排会话** | 是（操作系统窗口平铺）| 否（除非内置拆分视图）|
| **实现工作量** | 小 | 大 |
| **回归风险** | 低（现有代码未触及）| 高（存储层重写）|

---

## 建议

**从方法 A（多窗口）开始。** 它以最小的代码更改提供多会话功能，并且对现有功能零风险。Rust 差异很小且包含在内，前端更改仅限于两行（作用域监听器 + 窗口标签传递）加上"新窗口"操作。

如果需要，方法 B（标签）可以稍后分层添加 — 它所需的每个会话存储重构是正交的，可以独立完成。未来的混合方法（窗口内的标签，如 VS Code）将建立在两者之上。

### 方法 A 的实现顺序

1. 将 `WindowRegistry` 添加到 Rust 状态，更新 `emit_chat_event` 以使用 `emit_to`
2. 更新 `create_session` 以接受和注册 `window_label`
3. 添加 `create_window` Tauri 命令
4. 更新功能以允许窗口创建并作用域到 `chat-*` 窗口
5. 前端：在 ChatContainer 中切换到 `getCurrent().listen()`
6. 前端：在会话创建期间传递窗口标签
7. 添加"新窗口"快捷键（Cmd+N）和/或菜单项
8. 处理窗口关闭 → 会话清理
9. 添加设置同步广播通道
10. 使用目标事件传递测试并发会话
