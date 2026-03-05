<p align="center">
  <img src="assets/caipi-logo-source.png" alt="Caipi" width="128" height="128">
</p>

<h1 align="center">Caipi</h1>

<p align="center">
  一个快速、轻量级的 AI 编码 CLI 桌面应用程序。
  <br>
  <a href="https://caipi.ai">网站</a> &middot; <a href="https://github.com/pietz/caipi/releases/latest">下载</a>
</p>

<p align="center">
  <a href="https://github.com/pietz/caipi/releases/latest"><img src="https://img.shields.io/github/v/release/pietz/caipi?label=version" alt="Latest Release"></a>
  <a href="https://github.com/pietz/caipi/releases/latest"><img src="https://img.shields.io/github/downloads/pietz/caipi/total" alt="Downloads"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-BSL--1.1-blue" alt="License"></a>
</p>

---

<!-- TODO: 在此处添加产品截图 -->

聊天应用只是对话。Caipi 为 [Claude Code](https://docs.anthropic.com/en/docs/claude-code) 和 [Codex CLI](https://github.com/openai/codex) 提供了适当的桌面界面——这样 AI 可以读取您的文件、运行命令并进行更改，所有这些都通过一个视觉层向您展示发生了什么。

无需 API 密钥。Caipi 封装了您已经安装的 CLI，使用您现有的订阅。

## 功能

- **文件资源管理器** — 在侧边栏中浏览您的项目树，具有实时文件监视功能。双击在默认编辑器中打开文件。
- **会话历史** — 从中断的地方继续。会话从 CLI 自己的日志中加载，按项目文件夹分组。
- **权限模式** — 控制 AI 可以做什么。*默认*模式提示危险操作，*编辑*模式自动允许文件更改，*全部允许*模式绕过所有检查。
- **模型切换** — 在模型之间循环（Claude 的 Opus、Sonnet、Haiku；Codex 的 GPT-5.x），而无需离开聊天。
- **扩展思考** — 切换支持此功能的模型的思考深度（低/中/高）。
- **上下文跟踪** — 一眼看到使用了多少上下文窗口。
- **任务和技能侧边栏** — 在可折叠面板中跟踪代理待办事项和活动技能。
- **流式传输** — 当 AI 工作时实时显示文本和工具调用。
- **工具可见性** — 内联可折叠工具调用堆栈，显示 AI 正在读取、写入和运行的内容。
- **自动更新** — 内置更新机制使您保持最新版本。
- **浅色和深色模式** — 遵循您的系统偏好或手动设置。

## 安装

### 先决条件

您需要至少安装和验证以下 CLI 之一：

- **[Claude Code](https://docs.anthropic.com/en/docs/claude-code)** — 需要 Claude Pro 或 Max 订阅。
- **[Codex CLI](https://github.com/openai/codex)**（可选）— 需要 OpenAI API 密钥或订阅。

Caipi 在启动时会自动检测已安装的后端。

### 下载

| 平台 | 下载 | 要求 |
|----------|----------|--------------|
| **macOS** | [Apple Silicon (.dmg)](https://github.com/pietz/caipi/releases/latest/download/caipi_aarch64.dmg) | macOS 12+ |
| **Windows** | [x64 (.exe)](https://github.com/pietz/caipi/releases/latest/download/caipi_x64.exe) | Windows 10+ |

或者从[发布页面](https://github.com/pietz/caipi/releases/latest)获取最新版本。

## 入门

1. **安装**支持的 CLI（Claude Code 或 Codex）并登录。
2. **下载并打开** Caipi。
3. **设置向导**将检测您已安装的后端。选择您的默认后端。
4. **选择一个项目文件夹**以便在其中工作。
5. **开始聊天** — AI 可以在您的机器上读取您的文件、运行命令并进行更改。

## 键盘快捷键

| 快捷键 | 操作 |
|----------|--------|
| `Enter` | 发送消息 / 允许待处理的权限 |
| `Shift+Enter` | 在消息输入中换行 |
| `Escape` | 拒绝待处理的权限 |

## 支持的后端

### Claude Code

| 模型 | 级别 | 思考 |
|-------|------|----------|
| Opus 4.6 | 大 | 低 / 中 / 高 |
| Sonnet 4.5 | 中 | 低 / 中 / 高 |
| Haiku 4.5 | 小 | -- |

### Codex CLI

| 模型 | 级别 | 思考 |
|-------|------|----------|
| GPT-5.3 Codex | 大 | 低 / 中 / 高 |
| GPT-5.2 | 中 | 低 / 中 / 高 |
| GPT-5.1 Codex Mini | 小 | -- |

## 路线图

**下一步**
- 多窗口支持
- 附件支持
- 斜杠命令支持

**探索中**
- 计划模式
- Linux 支持
- GitHub Copilot CLI 支持
- Gemini CLI 支持

## 技术栈

- **前端**：Svelte 5、TypeScript、Tailwind CSS
- **后端**：Rust、Tauri 2.0
- **通信**：通过 JSON stdin/stdout 的 CLI 子进程，通过 Tauri 事件流式传输

## 许可证

[Business Source License 1.1](LICENSE) — 个人和非商业使用免费。商业使用需要付费许可证。四年后转换为 Apache 2.0。

联系方式：[caipi@plpp.de](mailto:caipi@plpp.de)
