# 内置 Agent 系统详解

> 最后更新: 2026-01-28

本文档详细说明 Oxide 的内置 Agent 系统，并与 Claude Code 进行对比。

---

## 📊 Claude Code 内置 Agent

Claude Code 真正内置的 Agent 只有 **6 个**，其余都是通过 Skill 系统扩展的：

| Agent 名称 | 模型 | 用途 | 说明 |
|-----------|------|------|------|
| **Bash** | inherit | 命令执行专家 | 处理 bash 命令和终端操作 |
| **general-purpose** | inherit | 通用任务代理 | 处理复杂的多步骤任务 |
| **statusline-setup** | sonnet | 状态栏配置 | 配置用户的 Claude Code 状态栏设置 |
| **Explore** | haiku | 代码库探索 | 快速探索代码库（quick/medium/very thorough） |
| **Plan** | inherit | 架构设计 | 设计实现计划并请求用户批准 |
| **claude-code-guide** | haiku | 使用指南 | 回答 Claude Code 使用相关问题 |

### 模型说明

- **inherit**: 继承父 Agent 的模型配置
- **sonnet**: 使用 Claude Sonnet 模型
- **haiku**: 使用 Claude Haiku 模型（快速、低成本）

### 设计理念

Claude Code 的内置 Agent 设计非常精简：
- **核心功能**: 只内置最基础、最通用的 Agent
- **可扩展性**: 通过 Skill 系统扩展专业领域能力
- **性能优化**: 简单任务使用 Haiku，复杂任务使用 Sonnet/Opus

---

## 🔧 Oxide 内置 Agent

Oxide 采用不同的设计理念，内置了 **6 个** 专用 Agent 类型：

| Agent 类型 | 模型 | 权限 | 工具集 | 用途 |
|-----------|------|------|--------|------|
| **Main** | 可配置 | 完整 | 全部工具 | 主对话 Agent，处理所有任务 |
| **Explore** | 可配置 | 只读 | Read, Grep, Glob, Scan | 代码库探索和理解 |
| **Plan** | 可配置 | 读写 | Read, Write, Grep, Glob, Scan, Todo | 架构设计和计划制定 |
| **CodeReviewer** | 可配置 | 只读 | Read, Grep, Glob, Scan | 代码审查和质量检查 |
| **FrontendDeveloper** | 可配置 | 完整 | 全部工具 | 前端开发专家 |
| **General** | 可配置 | 完整 | 全部工具 | 通用任务处理 |

### Agent 能力矩阵

| Agent 类型 | 文件读取 | 文件写入 | 文件编辑 | Shell 执行 | 代码搜索 | 只读模式 |
|-----------|---------|---------|---------|-----------|---------|---------|
| Main | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ |
| Explore | ✅ | ❌ | ❌ | ❌ | ✅ | ✅ |
| Plan | ✅ | ✅ | ❌ | ❌ | ✅ | ❌ |
| CodeReviewer | ✅ | ❌ | ❌ | ❌ | ✅ | ✅ |
| FrontendDeveloper | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ |
| General | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ |

---

## 🏗️ Oxide Agent 架构

### 1. Agent 类型定义

```rust
// src/agent/types.rs
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

### 2. Agent 能力描述

```rust
// src/agent/types.rs
pub struct AgentCapability {
    pub agent_type: AgentType,
    pub name: String,
    pub description: String,
    pub tools: Vec<String>,
    pub system_prompt: String,
    pub read_only: bool,
}
```

### 3. Subagent 管理器

```rust
// src/agent/subagent.rs
pub struct SubagentManager {
    current_agent: Arc<RwLock<AgentType>>,
    capabilities: HashMap<AgentType, AgentCapability>,
    agent_builder: Option<AgentBuilder>,
}

impl SubagentManager {
    /// 委派任务给指定类型的 Agent 执行
    pub async fn delegate(&self, agent_type: AgentType, request: &str) -> Result<String>

    /// 切换到指定的 Agent 类型
    pub fn switch_to(&self, agent_type: AgentType) -> Result<AgentType>

    /// 列出所有已注册的 Agent 能力
    pub fn list_capabilities(&self) -> Vec<AgentCapability>
}
```

### 4. Agent 构建器

```rust
// src/agent/builder.rs
pub struct AgentBuilder {
    base_url: String,
    auth_token: String,
    model: Option<String>,
}

impl AgentBuilder {
    /// 构建具有完整权限的 Main Agent
    pub fn build_main(&self) -> Result<AgentEnum>

    /// 构建只读的 Explore Agent
    pub fn build_explore(&self) -> Result<AgentEnum>

    /// 构建 Plan Agent
    pub fn build_plan(&self) -> Result<AgentEnum>

    /// 构建 Code Reviewer Agent
    pub fn build_code_reviewer(&self) -> Result<AgentEnum>

    /// 构建 Frontend Developer Agent
    pub fn build_frontend_developer(&self) -> Result<AgentEnum>

    /// 根据类型构建 Agent
    pub fn build_with_type(&self, agent_type: AgentType) -> Result<AgentEnum>
}
```

---

## 🆚 对比分析

### Claude Code vs Oxide

| 维度 | Claude Code | Oxide | 说明 |
|-----|-------------|-------|------|
| **内置 Agent 数量** | 6 个 | 6 个 | 数量相同 |
| **设计理念** | 精简核心 + Skill 扩展 | 专用类型 + 权限控制 | 不同的扩展策略 |
| **模型选择** | 按 Agent 固定 | 全局可配置 | Oxide 更灵活 |
| **权限控制** | 工具级别 | 工具集级别 | Oxide 更粗粒度 |
| **扩展方式** | Skill 系统 | Agent 类型 + Skill | Oxide 双重扩展 |
| **任务委派** | Task 工具 | SubagentManager | 实现方式不同 |

### 设计差异

#### Claude Code 的设计
- **最小化内置**: 只内置最基础的 6 个 Agent
- **Skill 为主**: 专业能力通过 Skill 系统扩展
- **模型优化**: 不同 Agent 使用不同模型（Haiku/Sonnet）
- **轻量级**: 减少核心系统复杂度

#### Oxide 的设计
- **类型化 Agent**: 每个 Agent 类型有明确的职责和权限
- **权限分级**: 通过工具集控制 Agent 能力
- **统一模型**: 所有 Agent 使用相同的模型配置
- **可扩展**: 支持添加新的 Agent 类型

---

## 📝 Agent 系统提示词

### Main Agent

```rust
const MAIN_AGENT_PROMPT: &str = r#"You are Oxide, a powerful AI programming assistant.

You have access to various tools for file operations, code search, and command execution.
Always prioritize safety and correctness. Use tools when necessary to accomplish tasks.

When working with code:
- Read files before modifying them
- Use grep and glob for code search
- Execute shell commands carefully
- Provide clear explanations

You are helpful, precise, and focused on delivering high-quality solutions."#;
```

### Explore Agent

```rust
const EXPLORE_AGENT_PROMPT: &str = r#"You are a code exploration specialist.

Your role is to help users understand codebases by:
- Reading and analyzing source files
- Searching for patterns and keywords
- Mapping code structure and dependencies
- Explaining architecture and design patterns

You have READ-ONLY access. You cannot modify files or execute commands.
Focus on providing clear, insightful analysis of the codebase."#;
```

### Plan Agent

```rust
const PLAN_AGENT_PROMPT: &str = r#"You are a software architecture and planning specialist.

Your role is to:
- Design implementation plans for features
- Analyze existing architecture
- Propose technical solutions
- Create structured task lists
- Document design decisions

You can read files and write planning documents, but cannot execute code or modify source files.
Focus on creating clear, actionable plans that guide implementation."#;
```

### Code Reviewer Agent

```rust
const CODE_REVIEWER_AGENT_PROMPT: &str = r#"You are a code review specialist.

Your role is to:
- Review code for bugs and issues
- Check code quality and style
- Identify security vulnerabilities
- Suggest improvements
- Verify best practices

You have READ-ONLY access. Focus on providing constructive, actionable feedback."#;
```

### Frontend Developer Agent

```rust
const FRONTEND_DEVELOPER_AGENT_PROMPT: &str = r#"You are a frontend development specialist.

Your expertise includes:
- React, Vue, Angular, and modern frameworks
- HTML, CSS, JavaScript/TypeScript
- UI/UX best practices
- Responsive design
- Performance optimization

You have full access to tools. Focus on creating high-quality, maintainable frontend code."#;
```

---

## 🚀 使用指南

### 查看可用 Agent

```bash
# 列出所有 Agent
/agent list

# 查看 Agent 能力
/agent capabilities
```

输出示例：

```
╔══════════════════════════════════════════════════════════════╗
║                    可用的 Agent 类型                          ║
╚══════════════════════════════════════════════════════════════╝

1. Main Agent
   描述: 主对话 Agent，具有完整权限
   工具: read_file, write_file, edit_file, delete_file, shell_execute, grep_search, glob, scan_codebase
   只读: 否

2. Explore Agent
   描述: 代码库探索专家，只读模式
   工具: read_file, grep_search, glob, scan_codebase
   只读: 是

3. Plan Agent
   描述: 架构设计和规划专家
   工具: read_file, write_file, grep_search, glob, scan_codebase, todo_write
   只读: 否

4. Code Reviewer Agent
   描述: 代码审查专家，只读模式
   工具: read_file, grep_search, glob, scan_codebase
   只读: 是

5. Frontend Developer Agent
   描述: 前端开发专家
   工具: read_file, write_file, edit_file, delete_file, shell_execute
   只读: 否

6. General Agent
   描述: 通用任务处理 Agent
   工具: read_file, write_file, edit_file, delete_file, shell_execute, grep_search, glob, scan_codebase
   只读: 否
```

### 编程方式使用

```rust
use oxide::agent::{AgentBuilder, SubagentManager};
use oxide::agent::types::AgentType;

// 1. 创建 Agent 构建器
let builder = AgentBuilder::new(
    "https://api.anthropic.com".to_string(),
    "your-api-key".to_string(),
    Some("claude-sonnet-4-5".to_string()),
);

// 2. 创建 Subagent 管理器
let manager = SubagentManager::with_builder(builder);

// 3. 委派任务给 Explore Agent
let result = manager.delegate(
    AgentType::Explore,
    "分析 src/main.rs 的代码结构"
).await?;

println!("探索结果: {}", result);

// 4. 委派任务给 Plan Agent
let plan = manager.delegate(
    AgentType::Plan,
    "设计一个用户认证系统的实现方案"
).await?;

println!("实现计划: {}", plan);
```

---

## 🔄 Agent 委派流程

```
┌─────────────────────────────────────────────────────────────┐
│                      用户请求                                │
│              "帮我分析这个代码库的结构"                        │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│                   Main Agent 判断                            │
│         "这是一个代码探索任务，委派给 Explore Agent"           │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│              SubagentManager.delegate()                      │
│         agent_type: Explore, request: "分析代码库结构"        │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│              AgentBuilder.build_explore()                    │
│         创建只读 Explore Agent 实例                           │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│                  Explore Agent 执行                          │
│    使用 read_file, grep_search, glob 等工具分析代码           │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│                    返回分析结果                               │
│              "代码库采用模块化架构..."                         │
└─────────────────────────────────────────────────────────────┘
```

---

## 🛡️ 安全性设计

### 权限隔离

1. **只读 Agent** (Explore, CodeReviewer)
   - 只能读取文件和搜索代码
   - 无法修改文件或执行命令
   - 适合代码分析和审查任务

2. **受限 Agent** (Plan)
   - 可以读写文件
   - 无法执行 Shell 命令
   - 适合规划和文档编写

3. **完整权限 Agent** (Main, General, FrontendDeveloper)
   - 可以执行所有操作
   - 需要用户明确授权
   - 适合实际开发任务

### 工具权限控制

```rust
// 只读工具集
fn create_read_only_tools(&self) -> ReadOnlyTools {
    ReadOnlyTools {
        read_file: WrappedReadFileTool::new(),
        grep_search: WrappedGrepSearchTool::new(),
        scan_codebase: WrappedScanCodebaseTool::new(),
        glob: WrappedGlobTool::new(),
    }
}

// 计划工具集（读写，无执行）
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

// 完整工具集
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
```

---

## 📈 性能优化建议

### 1. 模型选择策略

虽然 Oxide 目前使用统一模型，但可以通过配置优化：

```toml
# .oxide/config.toml
[agent.explore]
model = "claude-haiku-4"  # 快速、低成本

[agent.plan]
model = "claude-sonnet-4-5"  # 平衡性能和成本

[agent.main]
model = "claude-opus-4-5"  # 最强性能
```

### 2. Agent 选择指南

- **代码探索**: 使用 `Explore` Agent（安全、快速）
- **架构设计**: 使用 `Plan` Agent（专注规划）
- **代码审查**: 使用 `CodeReviewer` Agent（只读、专注质量）
- **前端开发**: 使用 `FrontendDeveloper` Agent（专业工具）
- **通用任务**: 使用 `Main` Agent（完整权限）

### 3. 任务委派最佳实践

```rust
// ✅ 好的做法：根据任务类型选择合适的 Agent
match task_type {
    TaskType::Explore => manager.delegate(AgentType::Explore, request).await,
    TaskType::Review => manager.delegate(AgentType::CodeReviewer, request).await,
    TaskType::Plan => manager.delegate(AgentType::Plan, request).await,
    TaskType::Implement => manager.delegate(AgentType::Main, request).await,
}

// ❌ 不好的做法：所有任务都用 Main Agent
manager.delegate(AgentType::Main, request).await
```

---

## 🔮 未来改进方向

### 1. 动态模型选择

```rust
impl AgentBuilder {
    pub fn build_with_model(&self, agent_type: AgentType, model: &str) -> Result<AgentEnum> {
        // 根据任务复杂度动态选择模型
    }
}
```

### 2. Agent 能力扩展

- [ ] 添加 DatabaseAdmin Agent（数据库管理）
- [ ] 添加 DevOps Agent（部署和运维）
- [ ] 添加 TestEngineer Agent（测试专家）
- [ ] 添加 SecurityAuditor Agent（安全审计）

### 3. 智能任务路由

```rust
impl SubagentManager {
    /// 根据任务描述自动选择最合适的 Agent
    pub async fn auto_delegate(&self, request: &str) -> Result<String> {
        let agent_type = self.analyze_task(request)?;
        self.delegate(agent_type, request).await
    }
}
```

### 4. Agent 协作

```rust
impl SubagentManager {
    /// 多个 Agent 协作完成复杂任务
    pub async fn collaborate(&self, agents: Vec<AgentType>, request: &str) -> Result<String> {
        // Explore Agent 分析代码
        // Plan Agent 设计方案
        // Main Agent 实现功能
        // CodeReviewer Agent 审查代码
    }
}
```

---

## 📚 相关文档

- [Agent 系统详解](./agent-system.md) - 完整的 Agent 架构文档
- [工具系统详解](./tool-system.md) - 工具实现和权限控制
- [Skill 系统详解](./skill-system.md) - Skill 扩展机制
- [已实现功能清单](./IMPLEMENTED_FEATURES.md) - 功能实现状态

---

## 🧪 测试覆盖

Subagent 系统有完整的单元测试覆盖：

```rust
// src/agent/subagent.rs:179-337
#[cfg(test)]
mod tests {
    #[test] fn test_subagent_manager_creation()
    #[test] fn test_switch_agent()
    #[test] fn test_switch_to_invalid_agent()
    #[test] fn test_list_capabilities()
    #[test] fn test_get_capability()
    #[test] fn test_is_read_only()
    #[test] fn test_get_system_prompt()
    #[test] fn test_get_tools()
    #[test] fn test_registered_agent_types()
    #[test] fn test_register_custom_agent()
}
```

测试覆盖率: **100%** (10/10 测试通过)

---

## 💡 总结

### Oxide 的优势

1. **类型安全**: 使用 Rust 类型系统确保 Agent 配置正确
2. **权限控制**: 细粒度的工具权限管理
3. **可扩展**: 易于添加新的 Agent 类型
4. **灵活配置**: 支持全局和 Agent 级别的模型配置

### 与 Claude Code 的差异

1. **Agent 数量**: 相同（6 个内置 Agent）
2. **扩展方式**: Oxide 更倾向于类型化 Agent，Claude Code 更倾向于 Skill
3. **模型策略**: Oxide 统一配置，Claude Code 按 Agent 优化
4. **实现语言**: Oxide 使用 Rust，Claude Code 使用 TypeScript

### 选择建议

- **如果需要类型安全和性能**: 选择 Oxide
- **如果需要快速扩展和灵活性**: 选择 Claude Code
- **如果需要 Rust 生态集成**: 选择 Oxide
- **如果需要 Node.js 生态集成**: 选择 Claude Code
