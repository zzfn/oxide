# 用 Rust 复刻 Claude Code

## 1. 项目目标

通过复刻 Claude Code 理解 AI Agent 的核心机制：

- **Agent 决策循环**：思考-行动-观察的闭环设计
- **工具系统**：让 LLM 从"顾问"变成"执行者"（Read/Write/Edit/Bash/Grep/Glob）
- **权限与 Human-in-the-loop**：自动化与用户控制的平衡
- **任务管理**：复杂任务的规划、分解与依赖跟踪
- **流式响应处理**：实时渲染与界面响应性
- **上下文管理**：对话历史组织与 Token 优化

选择 Rust 的原因：

1. **验证 AI 辅助开发模式**：完全不会 Rust，通过 AI 从零实现复杂系统，验证这种开发方式的可行性
2. **技术契合度高**：CLI 工具的最佳选择

## 2. 系统架构

### Agent 核心循环

```
用户输入
    │
    ▼
┌─────────────────────────────────────┐
│  Input Processor                     │
│  - 解析指令                          │
│  - 检查权限                          │
│  - 构建上下文                        │
└─────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────┐
│  Orchestrator (大脑)                 │
│  1. 调用 LLM (Anthropic API)         │
│  2. 接收流式响应                     │
│  3. 解析 Tool Call                   │
└─────────────────────────────────────┘
    │
    ▼
    有 Tool Call？
    │
    ├─ NO ──▶ 输出响应 ──▶ 结束
    │
    └─ YES ──▶ Toolbox
                │
                ├─ 权限确认
                ├─ 执行操作
                └─ 收集结果
                │
                ▼
            注入上下文 ──▶ 回到 Orchestrator
                            │
                            ▼
                    达到目标/最大迭代？
                            │
                    ┌───────┴───────┐
                   YES             NO
                    │               │
                输出结果        继续循环
```

### Agent Loop vs LangGraph

**核心区别**：

LangGraph（节点图模式）：

```
用户输入 → 规划节点 → 工具节点 → 总结节点 → 输出
         ↑_____________________________↓
```

- 预定义的节点和边
- 显式的状态转移
- 需要设计图结构

Claude Code（Agent Loop 模式）：

```rust
while !done {
    response = llm.call(messages + tools)
    if response.has_tool_calls() {
        results = execute_tools(response.tool_calls)
        messages.push(results)
    } else {
        done = true
    }
}
```

- LLM 自主决定是否调用工具
- 没有预定义的流程图
- 循环直到 LLM 不再请求工具

### Tools：从"顾问"到"执行者"

普通 Chatbot 与 Claude Code 的本质区别在于**能否动手执行**。

| 普通 Chatbot | Claude Code |
|-------------|-------------|
| 只能给建议（"你应该修改 A 文件"） | 直接执行（`Read` 看代码 → `Edit` 修改 → `Bash` 跑测试） |
| 依赖用户手动操作 | 自主完成端到端任务 |
| 上下文靠对话历史 | 上下文靠**工具执行结果**动态获取 |

**核心工具集**：

- **文件操作**: `Read` / `Write` / `Edit` —— 直接读写代码
- **代码搜索**: `Grep` / `Glob` —— 在代码库中定位目标
- **系统交互**: `Bash` —— 执行任意命令（`git status`、`cargo test`、`npm install`）

**Bash 工具尤其关键**，它让 Agent 能：
- 实时探索项目结构（`find`、`ls`）
- 获取环境信息（OS 版本、依赖版本）
- 运行测试验证修改
- 执行构建和部署

工具的本质是**让 LLM 触达外部环境**，形成"思考 → 行动 → 观察 → 再思考"的闭环。没有工具，LLM 只能基于训练数据猜测；有了工具，它能**实时获取信息、验证假设、完成实际操作**。

### 权限与人类介入（Human-in-the-loop）

工具带来能力的同时，也带来风险（误删文件、执行危险命令）。Claude Code 的核心设计哲学是：**Agent 自主决策，但关键操作必须人类确认**。

**危险操作分级**：

| 级别 | 操作类型 | 处理方式 |
|------|---------|---------|
| 🟢 安全 | `Read`、`Grep`、`Glob` | 直接执行 |
| 🟡 敏感 | `Edit`（修改现有文件） | 首次确认，同一会话可记忆 |
| 🔴 危险 | `Write`（创建新文件）、`Bash` | 每次确认，除非显式允许 |

**权限确认交互**：

```
┌─────────────────────────────────────────────┐
│ ⚠️  需要权限确认                              │
├─────────────────────────────────────────────┤
│ 工具: Bash                                   │
│ 参数: {"command": "rm -rf target"}          │
├─────────────────────────────────────────────┤
│ [Y] 允许本次  [A] 本次会话允许  [N] 拒绝    │
└─────────────────────────────────────────────┘
```

**为什么需要 Human-in-the-loop**：

1. **防止幻觉**：LLM 可能产生幻觉，生成危险或不正确的命令
2. **边界确认**：Agent 不知道哪些文件是敏感数据（如 `.env`）
3. **信任建立**：逐步授权让用户对 Agent 的行为有掌控感

这不是阻碍效率，而是**在自主与可控之间取得平衡**——Agent 负责繁琐的读写执行，人类负责关键决策把关。

### 任务管理系统

复杂任务（如"实现一个登录功能"）需要拆分为多个子任务。Claude Code 内置任务管理系统，让 Agent 能够规划、跟踪和执行多步骤工作。

**任务数据结构**：

```rust
pub struct Task {
    pub id: String,                    // 自增数字 ID
    pub subject: String,               // 任务标题（祈使句）
    pub description: String,           // 详细描述
    pub active_form: Option<String>,   // 进行中显示文本
    pub status: TaskStatus,            // Pending / InProgress / Completed
    pub owner: Option<String>,         // 子代理 ID
    pub blocks: Vec<String>,           // 此任务阻塞的任务列表
    pub blocked_by: Vec<String>,       // 阻塞此任务的任务列表
}
```

**任务依赖管理**：

```
任务 1: 设计数据库表
    │
    ▼ (blocks)
任务 2: 实现 API 接口
    │
    ▼ (blocks)
任务 3: 编写单元测试
```

系统使用 **DFS 算法** 检测循环依赖，确保任务图无环。

**任务工具集**：

| 工具 | 功能 |
|------|------|
| `TaskCreate` | 创建新任务，指定标题、描述、依赖关系 |
| `TaskList` | 查看所有任务摘要 |
| `TaskGet` | 获取任务详情（包括被哪些任务阻塞）|
| `TaskUpdate` | 更新状态、指定所有者、建立依赖 |

**典型工作流**：

```
用户: "帮我实现 JWT 认证"
    │
    ▼
Agent 创建任务:
  - 任务 1: 添加 jwt 依赖
  - 任务 2: 实现 Token 生成（依赖任务 1）
  - 任务 3: 实现 Token 验证（依赖任务 1）
  - 任务 4: 添加中间件（依赖任务 2,3）
    │
    ▼
Agent 按依赖顺序执行，每完成一个更新状态
    │
    ▼
遇到阻塞：创建子代理处理独立任务
```

任务管理系统让 Agent 具备**复杂任务规划能力**，不是一次性执行所有操作，而是有结构地推进工作，并能在中断后恢复上下文。

### 关键特性

- **流式响应**：LLM 输出实时渲染
- **权限中断**：Tool 执行前暂停等待确认
- **错误恢复**：Tool 失败信息反馈给 LLM 自我修正
- **上下文累积**：对话历史和 Tool 结果注入下一轮

### 模块划分

- **oxide-core**: 核心类型、配置、错误处理
- **oxide-provider**: LLM 提供商适配（Anthropic API）
- **oxide-tools**: 工具系统（Read/Write/Edit/Bash/Grep/Glob）
- **oxide-agent**: 代理和子代理系统
- **oxide-cli**: CLI 界面（Reedline + 流式渲染）

## 3. 开发环境

### AI 辅助工具

- **Claude Code**: 主力开发工具
  - Sonnet: 基础任务（文件读写、代码搜索、Git 操作）
  - Opus: 复杂架构设计（核心数据结构、异步流程编排、性能优化）

- **Codex**: Bug 修复
  - 借用检查器报错分析
  - 类型推断和生命周期标注
  - 编译器警告修复

## 4. 核心技术栈

| 类别     | 依赖                                           | 用途                                          |
| -------- | ---------------------------------------------- | --------------------------------------------- |
| 异步运行 | `tokio`                                        | 高性能 I/O                                    |
| LLM 交互 | `rig-core`                                     | 统一 LLM 抽象层                               |
| 界面 UI  | `termimad`, `crossterm`, `inquire`, `reedline` | Markdown 渲染、终端控制、交互提示、命令编辑   |
| 代码处理 | `walkdir`, `ignore`, `grep-searcher`, `regex`  | 目录遍历、.gitignore 过滤、代码搜索、正则匹配 |
| 序列化   | `serde` 系列                                   | JSON/YAML/TOML 配置                           |
| 错误处理 | `anyhow`, `thiserror`                          | 错误传播和自定义错误                          |
| 其他     | `chrono`, `git2`, `tiktoken-rs`                | 时间处理、Git 集成、Token 计数                |

## 5. 核心挑战与解决方案

### 流式渲染与状态管理

**挑战**: LLM token 流式输出 + Tool 执行日志混杂导致终端闪烁

**解法**:

- 设计 `StreamState` 状态机管理渲染优先级
- 区分 `Text`（文本流）、`Reasoning`（思考过程）、`ToolStatus`（工具状态）
- 使用 `crossterm` 光标回退和局部重绘

### 权限控制与安全性

**挑战**: Agent 能力强但潜在风险大（误删文件等）

**解法**: 三层防御体系

1. **静态配置**: `config.toml` 白名单/黑名单
2. **运行时拦截**: `PermissionManager` 劫持 Tool Call，危险操作强制确认
3. **会话记忆**: 支持"本次会话允许"，平衡安全性与易用性

### Prompt 动态构建

**挑战**: System Prompt 需根据 OS、Git 状态、用户规则动态变化

**解法**:

- `Builder` 模式实现 `PromptBuilder`
- 模块化组装 `Core`、`Security`、`Context`
- 动态注入 `RuntimeContext`（OS 版本、CWD 信息）

### 异步任务编排

**挑战**: 同时处理 stdin 输入（`reedline` 阻塞）、后台 Tool 执行、LLM 流式响应

**解法**:

- 基于 `tokio` Actor 模式
- 拆分 UI 渲染、LLM 交互、工具执行为独立异步任务
- 通过 Channel 通信，避免主线程阻塞

### 编辑操作的精准控制（待完善）

**当前局限**: `Edit` 工具基于字符串替换实现，存在以下问题：
- 多行修改时难以保证位置精准
- 缺少 diff 预览，用户无法确认变更范围
- 修改后 LSP 语义分析未更新，可能导致后续编辑基于过时信息

**施工中方案**:
- **Diff 预览**: 执行前展示统一的 diff 格式变更对比
- **LSP 集成**: 编辑后触发 LSP `didChange` 通知，同步语义信息
- **结构化编辑**: 探索基于 AST 的精准修改（替代字符串替换）

## 6. Langfuse 可观测性集成

- **全链路追踪**: 基于 `opentelemetry` + `tracing` 记录 Prompt 输入输出、Latency、Token 消耗
- **按需开启**: 仅在 Debug 模式 + 配置环境变量时启用
- **深度调试**: `tracing-opentelemetry` 将异步 Span 映射为 Langfuse Trace，可视化分析 Tool 执行耗时

## 7. 项目进度

### 已完成

- ✅ Phase 0: 基础设施（配置系统、错误处理、会话管理）
- ✅ Phase 1: LLM 集成（Anthropic API、流式响应、工具调用）
- ✅ Phase 2: 核心工具（Read/Write/Edit/Bash/Grep/Glob）
- ✅ Phase 4: CLI 界面（Reedline、Markdown 渲染、状态栏）
- 🚧 Phase 3: 高级功能（任务管理、计划模式、用户交互已完成，子代理系统待开发）

### 待开发

- ⏳ 子代理系统（Task Tool）
- ⏳ Hooks 系统（事件驱动 shell 命令）
- ⏳ 自动摘要与上下文管理
- ⏳ Git 集成（智能 commit、PR 审查）
- ⏳ MCP 服务器支持
- ⏳ 技能系统（Skills）

## 8. 核心收获

- **AI 辅助学习新语言**: 不需要先学完 Rust 再动手，遇到问题直接问 AI
- **类型系统的价值**: 编译器强制处理所有分支，避免运行时错误
- **性能优势明显**: 单一二进制、启动快、内存占用低
- **Agent 架构理解**: 通过实现理解"思考-行动-观察"循环的设计哲学
