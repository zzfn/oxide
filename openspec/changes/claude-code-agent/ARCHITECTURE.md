# Claude Code 集成架构总结

## 设计原则

**简洁、可扩展、符合现有架构**

不引入额外的抽象层（如 Provider），直接扩展现有的 `AgentEnum` 枚举，使用共享的基础设施。

## 目录结构

```
src/
├── external/                    # 通用外部工具基础设施（可复用）
│   ├── mod.rs
│   ├── process.rs               # 通用进程管理器
│   ├── pty.rs                   # 跨平台 PTY 管理
│   └── output.rs                # 通用输出结构定义
│
├── parsers/                     # 各工具的解析器实现
│   ├── mod.rs
│   ├── claude_code.rs           # Claude Code 输出解析
│   ├── cursor.rs                # 未来：Cursor 输出解析
│   └── copilot.rs               # 未来：Copilot 输出解析
│
├── agent/
│   ├── claude_code.rs           # ClaudeCodeAgent 实现（新建）
│   ├── builder.rs               # 添加 build_claude_code() 方法
│   └── types.rs                 # 添加 ClaudeCode 类型
│
└── cli/
    ├── command.rs               # 添加 /bridge, /export 命令
    └── ...
```

## 核心组件

### 1. external::process::ExternalProcess（通用进程管理器）

```rust
pub struct ExternalProcess {
    command: Command,
    child: Option<Child>,
    pty: Option<PtyPair>,
}

impl ExternalProcess {
    pub fn new(command: &str) -> Self;
    pub fn start(&mut self) -> Result<()>;
    pub fn send_input(&mut self, input: &str) -> Result<()>;
    pub fn read_output(&mut self) -> Result<String>;
    pub fn is_running(&self) -> bool;
    pub fn terminate(&mut self) -> Result<()>;
}
```

**职责**：
- 使用 `tokio::process` 启动和管理进程
- 连接到 `portable-pty` 创建的虚拟终端
- 提供异步 I/O 接口
- 可被任何外部工具 Agent 复用

### 2. external::pty::PtyManager（跨平台 PTY）

```rust
pub struct PtyManager {
    pty: Box<dyn MasterPty + Send>,
}

impl PtyManager {
    pub fn new() -> Result<Self>;
    pub fn writer(&self) -> Box<dyn Write + Send>;
    pub fn reader(&self) -> Box<dyn Read + Send>;
    pub fn resize(&self, rows: u16, cols: u16) -> Result<()>;
}
```

**职责**：
- 创建跨平台虚拟终端
- 提供读写句柄
- 保留 ANSI 转义字符
- 可被任何外部工具 Agent 复用

### 3. parsers::claude_code::ClaudeCodeParser（Claude Code 特定解析器）

```rust
pub struct ClaudeCodeParser {
    output: StructuredOutput,
}

impl ClaudeCodeParser {
    pub fn parse(&mut self, line: &str) -> ParseResult;
    pub fn extract_tool_call(&self, line: &str) -> Option<ToolCall>;
    pub fn extract_file_operation(&self, line: &str) -> Option<FileOperation>;
    pub fn extract_shell_command(&self, line: &str) -> Option<ShellCommand>;
}
```

**职责**：
- 解析 Claude Code 特定的输出格式
- 提取工具调用、文件操作、命令执行
- 过滤 ANSI 转义字符用于解析
- 特定于 Claude Code，不可复用

### 4. agent::claude_code::ClaudeCodeAgent

```rust
pub struct ClaudeCodeAgent {
    process: ExternalProcess,
    parser: ClaudeCodeParser,
    output_manager: OutputManager,
}

impl ClaudeCodeAgent {
    pub fn new(config: &ClaudeCodeConfig) -> Result<Self>;
    pub async fn chat(&mut self, message: &str) -> Result<String>;
    pub fn export_output(&self, format: ExportFormat) -> Result<()>;
}
```

**职责**：
- 组合通用基础设施和 Claude Code 特定解析器
- 实现 Agent 接口（chat 方法）
- 管理会话状态
- 处理输出捕获和显示

### 5. AgentEnum 扩展

```rust
pub enum AgentEnum {
    Anthropic(Agent<anthropic::completion::CompletionModel>),
    OpenAI(Agent<openai::responses_api::ResponsesCompletionModel>),
    ClaudeCode(ClaudeCodeAgent),  // 新增
}
```

**好处**：
- 符合现有架构
- 统一的 Agent 接口
- 易于切换 Agent 类型

## 数据流

```
用户输入
    ↓
CLI (/bridge on 启用 ClaudeCode Agent)
    ↓
AgentEnum::ClaudeCode
    ↓
ClaudeCodeAgent::chat(message)
    ↓
ExternalProcess::send_input(message)
    ↓
Claude Code 进程（通过 PTY）
    ↓
产生输出（带 ANSI）
    ↓
OutputSplitter:
├─→ TerminalRenderer → 终端显示（彩色输出）
└─→ ClaudeCodeParser → 解析工具调用
        ↓
    StructuredOutput (JSON)
        ↓
    OutputManager::export()
        ↓
    JSON 文件
```

## 未来扩展示例：添加 Cursor Agent

```rust
// 1. 创建 parsers/cursor.rs
pub struct CursorParser { /* Cursor 特定解析逻辑 */ }

// 2. 创建 agent/cursor.rs
pub struct CursorAgent {
    process: ExternalProcess,      // 复用！
    parser: CursorParser,           // Cursor 特定
    output_manager: OutputManager,  // 复用！
}

// 3. 扩展 AgentType
pub enum AgentType {
    // ... 现有类型
    ClaudeCode,
    Cursor,  // 新增
}

// 4. 扩展 AgentEnum
pub enum AgentEnum {
    // ... 现有变体
    ClaudeCode(ClaudeCodeAgent),
    Cursor(CursorAgent),  // 新增
}
```

**复用率**：约 70-80%
- `external::process` - 100% 复用
- `external::pty` - 100% 复用
- `external::output` - 100% 复用
- 只需实现 `CursorParser`（类似 ClaudeCodeParser）

## 关键优势

1. **符合现有架构**：没有额外的 Provider 层，直接扩展现有枚举
2. **高复用性**：通用基础设施（process、pty、output）可复用 70-80%
3. **易于扩展**：添加新工具只需实现解析器，约 200-300 行代码
4. **清晰分离**：通用逻辑与工具特定逻辑分开
5. **可测试**：每个模块都可以独立测试

## 规范映射

| 规范 | 描述 | 文件 |
|------|------|------|
| **cli-core** | CLI 命令扩展 | `specs/cli-core/spec.md` |
| **agent-system** | Agent 类型扩展 | `specs/agent-system/spec.md` |
| **external-tool** | 通用基础设施 | `specs/external-tool/spec.md` |
| **claude-code-parser** | Claude Code 解析 | `specs/claude-code-parser/spec.md` |

## 实施优先级

1. **阶段 1**：通用基础设施（external/）
2. **阶段 2**：Claude Code 解析器（parsers/claude_code.rs）
3. **阶段 3**：ClaudeCode Agent 实现（agent/claude_code.rs）
4. **阶段 4**：Agent 系统集成（agent/types.rs, agent/builder.rs）
5. **阶段 5**：CLI 命令（cli/command.rs）

每个阶段都可独立验证，降低风险。
