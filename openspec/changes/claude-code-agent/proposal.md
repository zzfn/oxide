# Change: 添加 Claude Code 桥接功能

## Why

当前 Oxide 是一个独立的 AI Agent CLI，但 Claude Code 拥有更强大的工具生态和更先进的模型能力。通过添加桥接功能，Oxide 可以选择性地将任务委托给 Claude Code 执行，捕获其输出并转换为结构化 JSON，同时保留原生彩色输出和进度条显示。这样既保留了 Oxide 的灵活性，又能利用 Claude Code 的优势。

## What Changes

- **新增外部工具基础设施** (`src/external/`)
  - `process.rs` - 通用的外部进程管理器（可复用于 Cursor、Copilot 等）
  - `pty.rs` - 跨平台 PTY 管理（可复用）
  - `output.rs` - 通用输出结构定义

- **新增解析器模块** (`src/parsers/`)
  - `claude_code.rs` - Claude Code 输出解析器
  - 未来可添加 `cursor.rs`、`copilot.rs` 等

- **新增 Claude Code Agent** (`src/agent/claude_code.rs`)
  - `ClaudeCodeAgent` 结构体，实现 Agent 接口
  - 使用通用基础设施（external::process、external::pty）
  - 集成 Claude Code 特定的解析器（parsers::claude_code）

- **扩展 Agent 系统**
  - 在 `AgentType` 枚举中添加 `ClaudeCode` 类型
  - 在 `AgentEnum` 中添加 `ClaudeCode(ClaudeCodeAgent)` 变体
  - 修改 `AgentBuilder` 添加 `build_claude_code()` 方法

- **结构化输出**
  - 定义工具调用结果的 JSON Schema（ToolCall、FileOperation、ShellOutput 等）
  - 实时输出流式处理，同时显示彩色输出和构建 JSON
  - 支持将结果导出为 JSON 文件供其他系统调用

- **CLI 增强**
  - 新增 `/bridge [on|off]` 命令控制桥接模式
  - 新增 `/export <format>` 命令导出结构化输出（JSON、Markdown）
  - 在 Prompt 中显示当前桥接模式状态

## Impact

- 受影响的规范:
  - **cli-core** - 新增桥接相关斜杠命令
  - **agent-system** - 新增 ClaudeCode Agent 类型
  - 新增 **external-tool-integration** 规范（通用外部工具集成）
  - 新增 **claude-code-parser** 规范（Claude Code 特定解析）

- 受影响的代码:
  - `src/agent/builder.rs` - 添加 `build_claude_code()` 方法
  - `src/agent/types.rs` - 添加 `ClaudeCode` 类型
  - `src/agent/claude_code.rs` - 新增 ClaudeCodeAgent 实现
  - `src/cli/` - 添加桥接模式相关命令
  - 新增 `src/external/` 模块（通用基础设施）
  - 新增 `src/parsers/` 模块（各工具的解析器）

- 新增依赖:
  - `portable-pty` - 跨平台伪终端支持
  - `async-trait` - 异步 trait 支持（如需要）

- 配置变更:
  - `.env.example` 添加 `CLAUDE_CODE_PATH` 配置项
  - 新增 `bridge` 配置节，包含桥接模式相关设置

## 非目标 (Out of Scope)

- 不替换现有的 Oxide Agent 功能，桥接是可选功能
- 不实现完整的 Claude Code CLI，仅作为委托执行器
- 不处理 Claude Code 的所有输出类型，优先处理工具调用结果
- 不实现双向通信（仅 Oxide → Claude Code 单向委托）
