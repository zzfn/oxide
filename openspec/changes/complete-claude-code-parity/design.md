# Design: Claude Code 功能对等实现

## 架构概述

Claude Code 的核心是 **Agent 系统**，它将不同的能力封装为专门的 Subagent。Oxide 需要实现类似的架构。

### Agent 层次结构

```
Oxide (主 Agent)
├── Explore Agent (代码库探索)
├── Plan Agent (架构规划)
├── Code Reviewer Agent (代码审查)
├── Frontend Developer Agent (前端开发)
└── ... (更多专业 Agent)
```

### 组件交互

```
用户输入
  ↓
CLI/TUI 层
  ↓
主 Agent (路由决策)
  ↓
┌─────────────┬───────────────┬────────────────┐
│ Subagent    │ Tools         │ Context        │
│ (专业能力)   │ (工具调用)     │ (会话管理)      │
└─────────────┴───────────────┴────────────────┘
  ↓
输出渲染 + 任务管理
```

## 模块设计

### 1. Agent System (`src/agent/`)

#### 新增结构

```rust
// src/agent/types.rs
pub enum AgentType {
    Main,                    // 主对话 Agent
    Explore,                 // 代码库探索
    Plan,                    // 架构规划
    CodeReviewer,            // 代码审查
    FrontendDeveloper,       // 前端开发
    // ... 更多类型
}

pub struct AgentCapability {
    pub name: String,
    pub description: String,
    pub tools: Vec<String>,
    pub system_prompt: String,
}

// src/agent/subagent.rs
pub struct SubagentManager {
    agents: HashMap<AgentType, AgentType>,
    current_agent: AgentType,
}

impl SubagentManager {
    pub fn new() -> Self;
    pub fn register(&mut self, agent_type: AgentType, agent: AgentType);
    pub fn switch_to(&mut self, agent_type: AgentType) -> Result<()>;
    pub fn current(&self) -> &AgentType;
    pub fn list_capabilities(&self) -> Vec<AgentCapability>;
}
```

#### 关键决策

1. **Agent 路由**：
   - 用户可以直接指定：`使用 Explore agent 分析这个文件`
   - 主 Agent 自动路由：根据任务类型自动选择合适的 Subagent
   - 使用 LLM 进行路由决策（需要额外的路由 Agent）

2. **Agent 生命周期**：
   - 每个会话维护独立的 Agent 实例
   - Subagent 可以创建嵌套的子任务
   - 任务完成后返回主 Agent

3. **工具权限**：
   - Main Agent：访问所有工具
   - Explore Agent：只读工具（Read, Glob, Grep）
   - Plan Agent：只读 + TodoWrite
   - CodeReviewer Agent：只读工具
   - FrontendDeveloper Agent：所有前端相关工具

### 2. Task Management (`src/task/`)

#### 新增模块

```rust
// src/task/manager.rs
pub struct TaskManager {
    tasks: HashMap<TaskId, Task>,
    storage_dir: PathBuf,
}

pub struct Task {
    pub id: TaskId,
    pub name: String,
    pub status: TaskStatus,
    pub agent_type: AgentType,
    pub created_at: DateTime<Utc>,
    pub output_file: Option<PathBuf>,
}

pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

#[async_trait]
impl TaskManager {
    pub async fn spawn(
        &mut self,
        name: String,
        agent_type: AgentType,
        prompt: String,
    ) -> Result<TaskId>;

    pub async fn get_output(&self, id: TaskId) -> Result<String>;

    pub async fn wait_for(&self, id: TaskId, timeout: Duration) -> Result<TaskStatus>;

    pub fn list(&self) -> Vec<Task>;
}
```

#### Task 工具实现

```rust
// src/tools/task.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskToolInput {
    pub description: String,
    pub prompt: String,
    pub agent_type: String,  // "explore", "plan", etc.
    pub run_in_background: bool,
}

impl Tool for TaskTool {
    fn execute(&self, input: TaskToolInput) -> Result<String> {
        // 1. 解析 agent_type
        // 2. 创建或获取 Subagent
        // 3. 执行任务
        // 4. 返回任务 ID 或直接结果
    }
}
```

#### 后台任务实现

- 使用 `tokio::task::spawn` 创建异步任务
- 任务输出写入临时文件
- 主进程定期检查任务状态
- 支持 `/tasks` 命令查看所有任务

### 3. Advanced Tools (`src/tools/`)

#### Glob 工具

```rust
// src/tools/glob.rs
pub struct GlobTool {
    base_dir: PathBuf,
}

#[derive(Debug, Deserialize)]
pub struct GlobInput {
    pub pattern: String,      // "**/*.rs"
    pub path: Option<String>, // 默认当前目录
}

impl Tool for GlobTool {
    fn execute(&self, input: GlobInput) -> Result<Vec<PathBuf>> {
        let pattern = input.pattern;
        let base = self.base_dir.join(input.path.unwrap_or("."));

        // 使用 glob crate
        let full_pattern = base.join(pattern).to_string_lossy().to_string();
        let paths: Vec<PathBuf> = glob(&full_pattern)?
            .filter_map(|p| p.ok())
            .collect();

        Ok(paths)
    }
}
```

#### MultiEdit 工具

```rust
// src/tools/multiedit.rs
#[derive(Debug, Deserialize)]
pub struct MultiEditInput {
    pub edits: Vec<EditOperation>,
}

pub struct EditOperation {
    pub file_path: PathBuf,
    pub old_string: String,
    pub new_string: String,
}

impl Tool for MultiEditTool {
    fn execute(&self, input: MultiEditInput) -> Result<()> {
        // 批量执行多个 Edit 操作
        // 使用 Edit 工具的逻辑
        for edit in input.edits {
            self.apply_edit(edit)?;
        }
        Ok(())
    }
}
```

#### AskUserQuestion 工具

```rust
// src/tools/ask.rs
#[derive(Debug, Deserialize)]
pub struct QuestionInput {
    pub questions: Vec<Question>,
}

pub struct Question {
    pub question: String,
    pub header: String,
    pub options: Vec<Option>,
    pub multi_select: bool,
}

#[async_trait]
impl Tool for AskUserQuestionTool {
    async fn execute(&self, input: QuestionInput) -> Result<HashMap<String, String>> {
        // 1. 暂停 TUI 渲染
        // 2. 使用 dialoguer 显示交互式选择器
        // 3. 收集用户答案
        // 4. 恢复 TUI
        // 5. 返回答案映射
    }
}
```

### 4. Configuration System (`src/config/`)

#### 配置层次

```
全局配置 (~/.oxide/config.toml)
  ↓
项目配置 (.oxide/config.toml)
  ↓
会话配置 (内存中，运行时)
```

#### 配置文件结构

```toml
# ~/.oxide/config.toml
[default]
model = "claude-sonnet-4-20250514"
max_tokens = 4096
temperature = 0.7

[agent.explore]
model = "claude-haiku-4-20250514"
max_tokens = 2048

[theme]
mode = "dark"
custom_theme = "~/.oxide/theme.toml"

[tui]
layout_mode = "standard"
streaming_enabled = true
typewriter_effect = true

[features]
enable_mcp = false
enable_multimodal = false
```

```markdown
# .oxide/CONFIG.md (项目配置)

# Project Instructions

## Language
- 思考和回复：中文
- 代码注释：中文

## Git Workflow
- Commit: Conventional Commits
- Branch: feat/, fix/, refactor/

## Tech Stack
- Rust 2021
- Tokio
```

#### 配置加载逻辑

```rust
// src/config/loader.rs
pub struct ConfigLoader {
    global_config: PathBuf,  // ~/.oxide/config.toml
    project_config: PathBuf, // .oxide/config.toml
    project_instructions: PathBuf, // .oxide/CONFIG.md
}

impl ConfigLoader {
    pub fn load() -> Result<Config> {
        let mut config = Config::default();

        // 1. 加载全局配置
        if self.global_config.exists() {
            config.merge(self.load_toml(&self.global_config)?);
        }

        // 2. 加载项目配置（覆盖全局）
        if self.project_config.exists() {
            config.merge(self.load_toml(&self.project_config)?);
        }

        // 3. 加载项目指令（系统提示词）
        if self.project_instructions.exists() {
            config.system_prompt = self.read_instructions(&self.project_instructions)?;
        }

        // 4. 环境变量覆盖
        config.apply_env_vars();

        Ok(config)
    }
}
```

### 5. Git Integration Enhancement

#### Git 安全检查

```rust
// src/tools/git_guard.rs
pub struct GitGuard {
    repo: git2::Repository,
}

impl GitGuard {
    pub fn check_safe(&self) -> Result<GitSafety> {
        // 1. 检查是否有未提交的更改
        // 2. 检查是否在主分支
        // 3. 检查远程分支状态
    }

    pub fn warn_if_pushing_to_main(&self) {
        if self.is_main_branch() && self.has_remote() {
            println!("⚠️  警告：即将推送到 main 分支");
        }
    }
}
```

#### Commit 规范验证

```rust
// src/tools/commit_linter.rs
pub struct CommitLinter;

impl CommitLinter {
    pub fn validate(message: &str) -> Result<CommitValidity> {
        let conventional = Regex::new(r"^(feat|fix|docs|style|refactor|test|chore)(\(.+\))?: ")?;

        if !conventional.is_match(message) {
            return Err(anyhow!("Commit message 不符合 Conventional Commits 规范"));
        }

        Ok(CommitValidity::Valid)
    }
}
```

### 6. MCP Support (可选)

#### MCP 客户端架构

```rust
// src/mcp/client.rs
pub struct McpClient {
    server_url: String,
    tools: Vec<McpTool>,
}

#[derive(Debug, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

#[async_trait]
impl McpClient {
    pub async fn connect(url: &str) -> Result<Self>;
    pub async fn list_tools(&self) -> Vec<McpTool>;
    pub async fn call_tool(&self, name: &str, args: Value) -> Result<Value>;
}

// src/mcp/wrapper.rs
pub struct McpToolWrapper {
    client: Arc<McpClient>,
    tool_name: String,
}

impl Tool for McpToolWrapper {
    fn execute(&self, input: Value) -> Result<String> {
        let result = self.client.call_tool(&self.tool_name, input).await?;
        Ok(serde_json::to_string(&result)?)
    }
}
```

#### MCP 配置

```toml
# ~/.oxide/mcp_servers.toml
[servers.filesystem]
command = "npx"
args = ["-y", "@modelcontextprotocol/server-filesystem", "/Users/c.chen/dev"]

[servers.github]
command = "npx"
args = ["-y", "@modelcontextprotocol/server-github"]
env = { GITHUB_TOKEN = "${GITHUB_TOKEN}" }
```

## 实现顺序

### Phase 1: 核心基础设施（2-3 周）
1. ✅ 配置系统重构
2. ✅ Agent 类型定义
3. ✅ Task Manager 基础实现
4. ✅ Glob 工具

### Phase 2: Agent System（3-4 周）
5. ✅ Subagent Manager
6. ✅ Explore Agent 实现
7. ✅ Plan Agent 实现
8. ✅ Task 工具集成
9. ✅ `/agent` 命令

### Phase 3: Advanced Tools（2-3 周）
10. ✅ MultiEdit 工具
11. ✅ AskUserQuestion 工具
12. ✅ NotebookEdit 工具
13. ✅ Git 增强

### Phase 4: TUI 完善（2 周）
14. 完成未完成的 TUI 任务
15. 主题系统
16. 性能优化

### Phase 5: 可选功能（按需）
17. MCP 支持
18. 多模态支持
19. Web 工具

## 技术挑战

### 1. Agent 路由决策
**挑战**：如何自动选择合适的 Agent？

**方案**：
- 使用 LLM 进行分类（需要额外 API 调用）
- 基于关键词规则匹配（快速但不够智能）
- 混合方案：规则 + LLM fallback

### 2. 后台任务管理
**挑战**：如何在异步环境中追踪任务状态？

**方案**：
- 使用 `tokio::task::JoinHandle` 管理任务
- 文件系统持久化状态
- 定期轮询任务状态（或使用 channels 通知）

### 3. TUI 交互中断
**挑战**：AskUserQuestion 需要暂停 TUI 渲染

**方案**：
- 使用 `Alternate Screen` 切换
- 保存和恢复 TUI 状态
- 临时退出 raw mode

### 4. 配置优先级
**挑战**：多个配置源如何正确合并？

**方案**：
- 定义清晰的优先级：环境变量 > 项目配置 > 全局配置
- 使用 `merge` 策略，后者覆盖前者
- 提供配置验证和错误提示

## 测试策略

### 单元测试
- 每个 Agent 类型的能力测试
- 每个工具的输入输出测试
- 配置加载逻辑测试
- Git 安全检查测试

### 集成测试
- 完整的 Agent 对话流程
- 任务创建和执行
- 多 Agent 协作

### 手动测试
- 用户体验测试
- 性能基准测试
- 错误恢复测试

## 性能考虑

- Agent 创建开销：缓存 Agent 实例
- 任务并发限制：限制同时运行的后台任务数量
- 配置热加载：避免频繁文件 I/O
- TUI 渲染优化：使用虚拟滚动
