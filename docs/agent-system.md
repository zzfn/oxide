# Agent 系统实现详解

## 目录

- [系统概述](#系统概述)
- [架构设计](#架构设计)
- [Agent 类型](#agent-类型)
- [Agent 构建](#agent-构建)
- [LLM 集成](#llm-集成)
- [工具管理](#工具管理)
- [使用指南](#使用指南)
- [扩展开发](#扩展开发)

## 系统概述

Oxide 的 Agent 系统是一个灵活、类型安全的 AI 助手架构，支持多种专用 Agent 类型，每种针对特定场景优化。系统采用构建器模式和多提供商支持，提供完整的工具调用能力。

### 核心特性

- **多 Agent 类型**: 6 种预定义 Agent 类型，满足不同场景需求
- **多提供商支持**: 同时支持 Anthropic Claude 和 OpenAI 兼容 API
- **权限控制**: 细粒度的工具权限管理（只读、读写、执行）
- **可扩展性**: 易于添加新的 Agent 类型和工具
- **类型安全**: 使用 Rust 类型系统确保配置正确性

## 架构设计

### 分层架构

```
┌─────────────────────────────────────┐
│         CLI 层                      │
│  - 用户命令处理                      │
│  - Agent 切换                        │
└─────────────────────────────────────┘
                ↓
┌─────────────────────────────────────┐
│      Agent 管理层                   │
│  - SubagentManager                  │
│  - Agent 注册和切换                 │
└─────────────────────────────────────┘
                ↓
┌─────────────────────────────────────┐
│      Agent 构建层                   │
│  - AgentBuilder                     │
│  - 工具配置                         │
│  - 提示词管理                       │
└─────────────────────────────────────┘
                ↓
┌─────────────────────────────────────┐
│      LLM 提供商层                   │
│  - Anthropic Client                 │
│  - OpenAI Client                    │
└─────────────────────────────────────┘
```

## Agent 类型

### 类型定义

`src/agent/types.rs` 中定义了 6 种 Agent 类型：

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentType {
    /// 主对话 Agent - 完整权限
    Main,

    /// 代码库探索 Agent - 只读
    Explore,

    /// 架构规划 Agent
    Plan,

    /// 代码审查 Agent - 只读
    CodeReviewer,

    /// 前端开发 Agent
    FrontendDeveloper,

    /// 通用 Agent
    General,
}
```

### Agent 能力矩阵

| Agent 类型 | 文件读取 | 文件写入 | 文件编辑 | Shell 执行 | 代码搜索 | 只读模式 |
|-----------|---------|---------|---------|-----------|---------|---------|
| Main | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ |
| Explore | ✅ | ❌ | ❌ | ❌ | ✅ | ✅ |
| Plan | ✅ | ✅ | ❌ | ❌ | ✅ | ❌ |
| CodeReviewer | ✅ | ❌ | ❌ | ❌ | ✅ | ✅ |
| FrontendDeveloper | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ |
| General | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ |

### 系统提示词

每个 Agent 类型都有专门的系统提示词，定义其：

- **专业领域**: 明确职责范围
- **行为准则**: 指导决策和行动
- **工具使用规范**: 如何正确使用工具
- **输出风格**: 回答的格式和语气

示例（Main Agent）：

```rust
const MAIN_AGENT_PROMPT: &str = r#"You are a powerful AI programming assistant.
You have access to various tools for file operations, code search, and command execution.
Always prioritize safety and correctness.
Use tools when necessary to accomplish tasks."#;
```

## Agent 构建

### AgentBuilder

`src/agent/builder.rs` 实现了构建器模式：

```rust
pub struct AgentBuilder {
    base_url: String,
    auth_token: String,
    model: Option<String>,
}

impl AgentBuilder {
    pub fn new(base_url: String, auth_token: String, model: Option<String>) -> Self {
        Self {
            base_url,
            auth_token,
            model,
        }
    }

    /// 构建具有完整权限的 Main Agent
    pub fn build_main(&self) -> Result<AgentEnum> {
        // 创建所有工具
        let tools = self.create_all_tools();

        // 根据提供商选择客户端
        if self.is_anthropic() {
            self.build_anthropic_agent(MAIN_AGENT_PROMPT, tools)
        } else {
            self.build_openai_agent(MAIN_AGENT_PROMPT, tools)
        }
    }

    /// 构建只读的 Explore Agent
    pub fn build_explore(&self) -> Result<AgentEnum> {
        // 只创建只读工具
        let tools = self.create_read_only_tools();

        // 使用专门的系统提示词
        if self.is_anthropic() {
            self.build_anthropic_agent(EXPLORE_AGENT_PROMPT, tools)
        } else {
            self.build_openai_agent(EXPLORE_AGENT_PROMPT, tools)
        }
    }

    // ... 其他构建方法
}
```

### 工具权限控制

不同的 Agent 类型获得不同的工具集合：

```rust
impl AgentBuilder {
    fn create_all_tools(&self) -> AllTools {
        AllTools {
            read_file: WrappedReadFileTool::new(),
            write_file: WrappedWriteFileTool::new(),
            edit_file: WrappedEditFileTool::new(),
            delete_file: WrappedDeleteFileTool::new(),
            create_directory: WrappedCreateDirectoryTool::new(),
            shell_execute: WrappedShellExecuteTool::new(),
            grep_search: WrappedGrepSearchTool::new(),
            glob: WrappedGlobTool::new(),
            scan_codebase: WrappedScanCodebaseTool::new(),
        }
    }

    fn create_read_only_tools(&self) -> ReadOnlyTools {
        ReadOnlyTools {
            read_file: WrappedReadFileTool::new(),
            grep_search: WrappedGrepSearchTool::new(),
            scan_codebase: WrappedScanCodebaseTool::new(),
            glob: WrappedGlobTool::new(),
        }
    }

    fn create_plan_tools(&self) -> PlanTools {
        PlanTools {
            read_file: WrappedReadFileTool::new(),
            write_file: WrappedWriteFileTool::new(),
            grep_search: WrappedGrepSearchTool::new(),
            scan_codebase: WrappedScanCodebaseTool::new(),
            glob: WrappedGlobTool::new(),
            todo_write: WrappedTodoWriteTool::new(),
        }
    }
}
```

## LLM 集成

### 多提供商支持

系统支持多种 LLM 提供商，通过 `AgentEnum` 统一管理：

```rust
pub enum AgentEnum {
    Anthropic(Agent<anthropic::completion::CompletionModel>),
    OpenAI(Agent<openai::responses_api::ResponsesCompletionModel>),
}
```

### 提供商识别

根据 `base_url` 自动选择提供商：

```rust
impl AgentBuilder {
    fn is_anthropic(&self) -> bool {
        self.base_url.contains("/anthropic") ||
        self.base_url.contains("anthropic.com")
    }

    fn build_agent(&self, prompt: &str, tools: AllTools) -> Result<AgentEnum> {
        if self.is_anthropic() {
            Ok(AgentEnum::Anthropic(
                self.build_anthropic_agent(prompt, tools)?
            ))
        } else {
            Ok(AgentEnum::OpenAI(
                self.build_openai_agent(prompt, tools)?
            ))
        }
    }
}
```

### Anthropic 集成

```rust
fn build_anthropic_agent(&self, prompt: &str, tools: AllTools) -> Result<Agent<...>> {
    let client = anthropic::Client::builder()
        .api_key(&self.auth_token)
        .base_url(&self.base_url)
        .build()?;

    let agent = Agent::builder(client, self.model.clone().unwrap_or_default())
        .preamble(prompt)
        .tool(tools.read_file)
        .tool(tools.write_file)
        // ... 注册其他工具
        .build();

    Ok(agent)
}
```

### OpenAI 集成

```rust
fn build_openai_agent(&self, prompt: &str, tools: AllTools) -> Result<Agent<...>> {
    let client = openai::Client::builder()
        .api_key(&self.auth_token)
        .base_url(&self.base_url)
        .build()?;

    let agent = Agent::builder(client, self.model.clone().unwrap_or_default())
        .preamble(prompt)
        .tool(tools.read_file)
        .tool(tools.write_file)
        // ... 注册其他工具
        .build();

    Ok(agent)
}
```

### 流式响应

Agent 支持流式响应，提供实时反馈：

```rust
let mut stream = agent
    .stream_prompt(user_input)
    .with_hook(hook.clone())
    .multi_turn(20)
    .with_history(context_manager.get_messages().to_vec())
    .await;

while let Some(chunk) = stream.next().await {
    print!("{}", chunk);
    stdout().flush()?;
}
```

## 工具管理

### 工具包装

所有工具都使用包装器模式添加可视化反馈：

```rust
pub struct WrappedReadFileTool {
    inner: ReadFileTool,
}

impl WrappedReadFileTool {
    pub fn new() -> Self {
        Self {
            inner: ReadFileTool,
        }
    }
}

#[async_trait]
impl Tool for WrappedReadFileTool {
    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // 显示工具调用
        println!(
            "{} {}({})",
            "●".bright_green(),
            "Read".bold(),
            args.file_path
        );

        // 执行实际操作
        let result = self.inner.call(args).await;

        // 显示结果
        match &result {
            Ok(output) => println!(
                "{} {} → {} bytes",
                "✓".green(),
                "Success".green(),
                output.size
            ),
            Err(e) => println!("{} {}", "✗".red(), e),
        }

        result
    }
}
```

### 工具注册流程

1. **创建工具实例**
2. **注册到 Agent**
3. **生成工具定义**（JSON Schema）
4. **LLM 调用工具**
5. **执行工具逻辑**
6. **返回结果给 LLM**

## 使用指南

### 基本使用

```rust
use oxide::agent::{AgentBuilder, AgentType};

// 1. 创建构建器
let builder = AgentBuilder::new(
    config.base_url.clone(),
    config.auth_token.clone(),
    config.model.clone(),
);

// 2. 构建 Agent
let agent = builder.build_main()?;

// 3. 发送消息
let response = agent.prompt("Help me understand this codebase").await?;
```

### Agent 切换

CLI 支持运行时切换 Agent：

```bash
# 查看所有可用的 Agent
/agent list

# 切换到 Explore Agent
/agent switch explore

# 切换到 Plan Agent
/agent switch plan

# 查看 Agent 能力
/agent capabilities
```

### 查看当前 Agent

```bash
# 显示当前配置
/config
```

输出示例：

```
==================================================
Oxide CLI 0.1.0 - DeepSeek Agent
==================================================
模型: deepseek-chat
会话: whole-comfort-1234
当前 Agent: Main (完整权限)
```

## 扩展开发

### 添加新 Agent 类型

1. **定义新类型** (`src/agent/types.rs`):

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentType {
    // ... 现有类型

    /// 数据库管理 Agent
    DatabaseAdmin,
}
```

2. **添加系统提示词** (`src/agent/builder.rs`):

```rust
const DATABASE_ADMIN_PROMPT: &str = r#"You are a database administration expert.
You can help with schema design, query optimization, and data migration.
Always prioritize data integrity and performance."#;
```

3. **实现构建方法** (`src/agent/builder.rs`):

```rust
impl AgentBuilder {
    pub fn build_database_admin(&self) -> Result<AgentEnum> {
        let tools = self.create_database_tools();

        if self.is_anthropic() {
            self.build_anthropic_agent(DATABASE_ADMIN_PROMPT, tools)
        } else {
            self.build_openai_agent(DATABASE_ADMIN_PROMPT, tools)
        }
    }

    fn create_database_tools(&self) -> DatabaseTools {
        DatabaseTools {
            read_file: WrappedReadFileTool::new(),
            shell_execute: WrappedShellExecuteTool::new(),
            // 添加数据库相关工具
        }
    }
}
```

4. **更新 CLI 命令** (`src/cli/mod.rs`):

```rust
fn handle_agent_command(&mut self, args: &str) -> Result<()> {
    match args {
        "list" => self.list_agents(),
        "switch database_admin" => self.switch_agent(AgentType::DatabaseAdmin),
        // ... 其他命令
    }
}
```

### 添加新工具

参见 [工具系统文档](./tool-system.md)

### 自定义 Agent 能力

可以通过配置文件自定义 Agent 行为：

```toml
# .oxide/config.toml
[agent.database_admin]
model = "claude-opus-4"
temperature = 0.3
max_tokens = 8192

[agent.database_admin.tools]
read_file = true
write_file = true
shell_execute = true
query_database = true
```

## 最佳实践

### 选择合适的 Agent

- **代码探索**: 使用 `Explore` Agent（安全、快速）
- **架构设计**: 使用 `Plan` Agent（专注规划）
- **代码审查**: 使用 `CodeReviewer` Agent（只读、专注质量）
- **前端开发**: 使用 `FrontendDeveloper` Agent（专业工具）
- **通用任务**: 使用 `Main` Agent（完整权限）

### 安全性考虑

1. **只读模式**: 探索和审查任务使用只读 Agent
2. **权限最小化**: 只授予任务所需的工具权限
3. **API 密钥保护**: 使用环境变量存储敏感信息
4. **输入验证**: 验证文件路径和命令参数

### 性能优化

1. **模型选择**: 简单任务使用较小模型（如 Haiku）
2. **工具缓存**: 缓存常用工具的结果
3. **流式响应**: 启用流式输出提升用户体验
4. **会话管理**: 定期清理过长会话

## 相关文档

- [工具系统](./tool-system.md) - 深入了解工具实现
- [配置管理](./config-management.md) - 配置 Agent 行为
- [会话管理](./session-management.md) - 管理对话历史
- [整体架构](./architecture.md) - 项目架构总览
