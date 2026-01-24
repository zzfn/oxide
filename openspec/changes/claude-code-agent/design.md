# Design: Claude Code 桥接架构

## Context

Oxide 是一个基于 Rust 的 AI Agent CLI，当前使用 rig-core 库与多种 LLM 提供商交互。Claude Code 是 Anthropic 官方的 AI 编程助手，拥有更强大的工具生态和模型能力。用户希望能够在 Oxide 中选择性地使用 Claude Code，同时保留 Oxide 的灵活性和定制能力。

关键约束：
1. 必须保留 ANSI 转义字符，用户需要看到 Claude Code 的原生彩色输出和进度条
2. 必须解析工具调用结果为结构化 JSON，供其他系统调用
3. 不能通过 Shell 调用，必须使用 `tokio::process` 直接管理进程
4. 桥接功能是可选的，不影响现有 Oxide 功能

## Goals / Non-Goals

### Goals
- 创建 Claude Code 进程管理器，使用 `tokio::process::Command` 启动和控制
- 使用 `portable-pty` 创建虚拟终端，保留 ANSI 转义字符
- 实时捕获输出，同时进行流式显示和结构化解析
- 解析工具调用结果（Read、Write、Bash、Task 等）为 JSON
- 提供桥接模式开关，用户可选择是否使用 Claude Code
- 支持导出结构化输出为 JSON 文件

### Non-Goals
- 不实现双向通信协议（Oxide → Claude Code 单向委托）
- 不实现完整的 Claude Code 输出解析，优先处理常见工具调用
- 不替换或修改现有的 Oxide Agent 系统
- 不实现 Claude Code 的会话管理（由 Claude Code 自己处理）

## Decisions

### 1. 使用 portable-pty 库

**决策**: 使用 `portable-pty` 库创建跨平台虚拟终端

**原因**:
- 支持 macOS、Linux、Windows
- 提供原生 PTY 实现，完美保留 ANSI 转义字符
- 可以获取原始输出，同时用于显示和解析

**替代方案**:
- 使用 `pty` crate（仅 Unix）
- 使用 `conpty`（仅 Windows）
- 不使用 PTY，直接使用 stdin/stdout（会丢失 ANSI 序列）

### 2. 进程管理架构

**决策**: 使用 `tokio::process::Command` + `portable-pty` 组合

**架构**:
```
Oxide CLI
  │
  ├─→ ClaudeCodeAgent (src/agent/claude_code.rs)
  │     │
  │     ├─→ external::process::ExternalProcess (通用进程管理)
  │     │     │
  │     │     ├─→ external::pty::PtyManager (创建虚拟终端)
  │     │     │     │
  │     │     │     └─→ Claude Code 进程
  │     │     │           │
  │     │     │           └─→ stdout (带 ANSI 的原始输出)
  │     │     │                 │
  │     │     ├─→ OutputSplitter (同时处理显示和解析)
  │     │     │     ├─→ TerminalRenderer (显示带 ANSI 的输出)
  │     │     │     └─→ parsers::claude_code::Parser (解析工具调用)
  │     │     │           │
  │     │     │           └─→ StructuredOutput (JSON)
  │     │     │                 │
  │     │     └─→ ResultExporter (导出 JSON)
  │     │
  │     └─→ 实现 Agent 接口（chat 方法等）
  │
  └─→ 未来可复用同样架构添加 CursorAgent、CopilotAgent 等
```

**原因**:
- `tokio::process` 提供异步进程管理，不阻塞事件循环
- PTY 确保 Claude Code 认为自己在真实终端中，正常输出彩色内容
- OutputSplitter 同时处理显示和解析，一次处理完成两项任务

### 3. 输出解析策略

**决策**: 使用正则表达式 + 状态机解析工具调用

**原因**:
- Claude Code 的输出格式相对稳定，工具调用有明确模式
- 正则表达式性能好，适合实时流式解析
- 状态机可以处理跨行的工具调用

**解析目标**:
1. **Tool Use**: `<tool_name>` 工具调用开始标记
2. **Tool Result**: 工具执行结果（JSON、文本）
3. **File Operations**: 文件路径、修改类型
4. **Shell Commands**: 命令本身、退出码、输出
5. **Progress Updates**: 进度条、状态更新（保留原始 ANSI）

**输出结构**:
```json
{
  "version": "1.0",
  "timestamp": "2025-01-24T12:00:00Z",
  "session_id": "abc123",
  "tool_calls": [
    {
      "tool": "read_file",
      "parameters": { "path": "src/main.rs" },
      "result": { "status": "success", "content": "..." },
      "timestamp": "2025-01-24T12:00:01Z"
    }
  ],
  "file_operations": [
    {
      "operation": "edit",
      "path": "src/main.rs",
      "changes": "..."
    }
  ],
  "shell_commands": [
    {
      "command": "cargo test",
      "exit_code": 0,
      "output": "..."
    }
  ],
  "raw_output": "原始 ANSI 输出（base64 编码）"
}
```

### 4. Agent 类型设计

**决策**: 新增 `ClaudeCodeBridge` Agent 类型，而非修改现有 Agent

**Agent 类型定义**:
```rust
pub enum AgentType {
    Main,           // 现有：完整工具权限
    Explore,        // 现有：只读
    Plan,           // 现有：规划
    CodeReviewer,   // 现有：代码审查
    FrontendDeveloper, // 现有：前端开发
    General,        // 现有：通用

    ClaudeCodeBridge, // 新增：委托给 Claude Code
}
```

**原因**:
- 保持向后兼容，不影响现有代码
- 清晰的职责分离
- 用户可以明确选择是否使用桥接模式

### 5. 配置管理

**决策**: 在配置文件中添加独立的 `[bridge]` 节

**配置结构**:
```toml
[bridge]
enabled = false              # 默认关闭
claude_code_path = "claude"  # Claude Code 可执行文件路径
output_format = "json"       # 输出格式: json, markdown, both
save_raw_output = true       # 是否保存原始 ANSI 输出
output_dir = ".oxide/bridge" # 结构化输出保存目录
```

**环境变量**:
```bash
CLAUDE_CODE_PATH=/usr/local/bin/claude
BRIDGE_ENABLED=true
BRIDGE_OUTPUT_DIR=/path/to/output
```

**原因**:
- 独立配置节，便于管理
- 支持环境变量覆盖，适合 CI/CD 场景
- 默认关闭，不影响现有用户

### 6. 错误处理

**决策**: 分层错误处理，优雅降级

**错误场景**:
1. **Claude Code 未安装**: 提示安装路径，回退到 Oxide Agent
2. **PTY 创建失败**: 记录错误，回退到普通进程模式（无彩色输出）
3. **输出解析失败**: 保留原始输出，记录解析错误
4. **进程崩溃**: 显示错误信息，清理资源，不崩溃 Oxide

**原因**:
- 桥接是增强功能，不应影响 Oxide 核心功能
- 用户应始终能使用 Oxide，即使 Claude Code 不可用

## Risks / Trade-offs

### Risk 1: Claude Code 输出格式变化

**风险**: Claude Code 更新可能导致输出格式变化，解析失败

**缓解措施**:
- 解析器设计宽松，匹配关键模式而非严格格式
- 提供解析回退模式，失败时保留原始输出
- 定期测试 Claude Code 不同版本

### Risk 2: 跨平台 PTY 兼容性

**风险**: `portable-pty` 在某些平台上可能有问题

**缓解措施**:
- 充分测试 macOS、Linux、Windows
- 提供配置项禁用 PTY（牺牲彩色输出）
- 记录平台特定问题和解决方案

### Risk 3: 性能开销

**风险**: 实时解析可能增加 CPU 使用率

**缓解措施**:
- 使用高效的正则表达式引擎
- 异步解析，不阻塞主线程
- 提供配置项禁用解析（仅保留原始输出）

### Trade-off: 解析完整性 vs 性能

**选择**: 优先解析常见工具调用（Read、Write、Bash），不追求 100% 覆盖

**原因**:
- 常见工具调用覆盖 90%+ 使用场景
- 保持解析器简单可维护
- 新工具可以后续添加支持

## Migration Plan

### 阶段 1: 基础桥接（MVP）
- 实现 Claude Code 进程管理
- 实现 PTY 创建和输出捕获
- 实现基础终端显示（带 ANSI）

### 阶段 2: 输出解析
- 实现工具调用解析器（Read、Write、Bash）
- 实现结构化 JSON 生成
- 实现输出导出功能

### 阶段 3: CLI 集成
- 添加 `/bridge` 命令
- 添加 `/export` 命令
- 集成到 AgentBuilder

### 阶段 4: 测试和优化
- 跨平台测试
- 性能优化
- 文档完善

### Rollback 策略

每个阶段独立，可以随时回滚：
- 通过配置 `[bridge.enabled] = false` 完全禁用
- 删除 `src/claude_code/` 目录即可移除
- 不影响现有代码，无迁移成本

## Open Questions

1. **Claude Code 路径检测**:
   - Q: 如何自动检测 Claude Code 安装路径？
   - A: 优先使用环境变量，其次 `which claude`，最后配置文件

2. **会话管理**:
   - Q: 是否需要同步 Oxide 和 Claude Code 的会话？
   - A: MVP 不需要，Claude Code 自己管理会话。未来可以考虑。

3. **双向通信**:
   - Q: 是否需要实现 Oxide → Claude Code 的实时交互？
   - A: MVP 不需要。用户通过 Oxide 提交任务，Claude Code 执行完成后返回结果。

4. **输出格式版本化**:
   - Q: 如何处理 Claude Code 输出格式变化？
   - A: 在 JSON 结构中包含 `version` 字段，解析器支持多版本。

## Implementation Notes

### 关键代码模块

**src/external/mod.rs** - 外部工具基础设施入口
**src/external/process.rs** - 通用进程管理器
**src/external/pty.rs** - 跨平台 PTY 管理
**src/external/output.rs** - 通用输出结构定义

**src/parsers/mod.rs** - 解析器模块入口
**src/parsers/claude_code.rs** - Claude Code 输出解析器

**src/agent/claude_code.rs** - ClaudeCodeAgent 实现
- 使用 external::process 管理进程
- 使用 external::pty 创建虚拟终端
- 使用 parsers::claude_code 解析输出
- 实现 Agent 接口（chat 方法等）

### 依赖更新

```toml
[dependencies]
portable-pty = "0.8"  # 跨平台 PTY 支持
async-trait = "0.1"   # 异步 trait（如需要）
```

### 测试策略

- 单元测试：解析器逻辑
- 集成测试：真实的 Claude Code 进程
- Mock 测试：模拟输出，验证解析
- 跨平台测试：GitHub Actions 多平台运行
