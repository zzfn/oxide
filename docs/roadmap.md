# Oxide 实现路线图

> 用 Rust 复刻 Claude Code 的完整实现计划

## 📋 项目概览

Oxide 旨在成为一个高性能、可扩展的 AI 编程助手，复刻 Claude Code 的核心功能，同时利用 Rust 的性能优势。

## 🎯 核心目标

- **完整的工具系统**：实现 Claude Code 的所有核心工具
- **高性能执行**：利用 Rust 的并发能力优化响应速度
- **可扩展架构**：支持自定义工具和插件
- **优秀的用户体验**：流畅的 CLI 交互和清晰的输出

---

## 🗺️ 实现阶段

### Phase 0: 基础设施 (2-3 周)

**目标**: 搭建项目基础架构和核心模块

**当前状态**: ✅ 基本完成

#### 0.1 项目结构设计
- [x] 定义清晰的模块划分
  - `oxide-core`: 核心类型和 trait
  - `oxide-provider`: LLM 提供商适配
  - `oxide-tools`: 工具系统实现
  - `oxide-cli`: 命令行界面
  - `oxide-agent`: 代理和子代理系统
- [x] 设置 workspace 结构
- [ ] 配置 CI/CD 流程

#### 0.2 配置系统
- [x] 实现配置文件解析 (`~/.oxide/config.toml`)
  - 权限管理 (allow/deny 列表)
  - 插件启用/禁用
  - 行为设置 (thinking mode, cleanup period 等)
  - 模型选择和参数（temperature, max_tokens 等）
- [x] 环境变量管理 (API Key 从环境变量读取)
- [x] 会话状态持久化 (`~/.oxide/` 目录结构)
  - `history.jsonl`: 命令历史
  - `session-env/`: 会话环境
  - `tasks/`: 任务状态
  - `plans/`: 计划文件

#### 0.3 错误处理
- [x] 定义统一的错误类型 (`OxideError`)
- [x] 实现错误传播和上下文
- [x] 用户友好的错误消息

#### 0.4 日志系统
- [ ] 集成 `tracing` 框架
- [ ] 结构化日志输出
- [ ] 调试模式支持

---

### Phase 1: LLM 集成 (1-2 周)

**目标**: 实现与 Claude API 的基础交互

**当前状态**: ✅ 已完成

#### 1.1 Provider 抽象
- [x] 定义 `LLMProvider` trait
- [x] 实现 Anthropic API 客户端
- [x] 支持流式响应
- [x] 处理 rate limiting 和重试（通过 reqwest 超时）

#### 1.2 消息管理
- [x] 实现 `Message` 和 `Conversation` 类型
- [x] 支持多模态内容（文本、图片）
- [x] 上下文窗口管理（通过 max_tokens 控制）
- [x] Token 计数和预算控制（API 返回 usage 信息）

#### 1.3 工具调用协议
- [x] 实现 Anthropic 工具调用格式
- [x] 工具结果序列化/反序列化
- [x] 多轮工具调用循环（支持 ToolUse 和 ToolResult）

---

### Phase 2: 核心工具系统 (3-4 周)

**目标**: 实现 Claude Code 的核心工具集

**当前状态**: 🚧 进行中（文件操作工具已完成）

#### 2.1 工具框架
- [x] 定义 `Tool` trait
- [x] 工具注册和发现机制
- [x] 参数验证和类型转换
- [ ] 工具执行沙箱（可选）

#### 2.2 文件操作工具
- [x] **Read**: 读取文件内容
  - 支持行范围读取
  - 处理大文件
  - 带行号格式化输出
- [x] **Write**: 写入文件
  - 覆盖检测
  - 自动创建父目录
- [x] **Edit**: 精确字符串替换
  - 唯一性检查
  - 批量替换模式
- [x] **Glob**: 文件模式匹配
  - 支持 glob 语法
  - 按修改时间排序
- [x] **Grep**: 代码搜索
  - 集成 ripgrep
  - 支持正则表达式
  - 上下文行显示

#### 2.3 执行工具
- [x] **Bash**: 命令执行
  - 工作目录持久化
  - 超时控制
  - 后台任务支持
  - 输出流式传输
- [x] **TaskOutput**: 获取后台任务输出
- [x] **TaskStop**: 停止后台任务

#### 2.4 网络工具
- [ ] **WebFetch**: 网页内容获取
  - HTML 转 Markdown
  - 缓存机制
  - 重定向处理

---

### Phase 3: 高级功能 (4-5 周)

**目标**: 实现代理系统和高级交互功能

**当前状态**: ⏳ 未开始（基础结构已搭建）

#### 3.1 子代理系统 (Task Tool)
- [ ] 定义子代理类型和能力
  - `Bash`: 命令执行专家
  - `Explore`: 代码库探索
  - `Plan`: 实现规划
  - `general-purpose`: 通用代理
- [ ] 子代理生命周期管理
- [ ] 父子代理通信协议
- [ ] 后台代理支持
- [ ] 代理恢复机制 (resume)

#### 3.2 计划模式 (Plan Mode)
- [x] **EnterPlanMode**: 进入计划模式
- [x] **ExitPlanMode**: 退出并请求批准
- [x] 计划文件管理
- [x] 权限请求系统（声明）
- [x] 工具执行权限控制（基础实现）

#### 3.3 任务管理系统
- [x] **TaskCreate**: 创建任务
- [x] **TaskList**: 列出任务
- [x] **TaskGet**: 获取任务详情
- [x] **TaskUpdate**: 更新任务状态
- [x] 任务依赖关系管理
- [x] 循环依赖检测
- [ ] 任务持久化

#### 3.4 用户交互
- [x] **AskUserQuestion**: 询问用户
  - 单选/多选支持
  - 选项推荐
  - 自定义输入

---

### Phase 4: CLI 界面 (2-3 周)

**目标**: 打造流畅的命令行交互体验

**当前状态**: ✅ 基本完成

#### 4.1 基础 CLI
- [x] 集成 Reedline 编辑器
- [x] 命令历史记录
- [x] 自动补全
- [x] 语法高亮

#### 4.2 输出渲染
- [x] Markdown 渲染（使用 `termimad`）
- [x] 代码块语法高亮
- [x] 进度指示器
- [x] 流式输出显示

#### 4.3 状态栏
- [x] 显示当前模式
- [x] Token 使用统计
- [x] 任务进度
- [x] 后台任务状态

#### 4.4 快捷命令
- [x] `/help`: 帮助信息
- [x] `/clear`: 清空会话
- [ ] `/compact`: 压缩上下文
- [ ] `/tasks`: 任务列表
- [ ] `/config`: 配置管理

---

### Phase 5: Git 集成 (1-2 周)

**目标**: 深度集成 Git 工作流

**当前状态**: ⏳ 未开始

#### 5.1 Git 操作
- [ ] 自动检测 Git 仓库
- [ ] 读取 git status
- [ ] 智能 commit 消息生成
- [ ] Pre-commit hook 处理
- [ ] 分支管理

---

### Phase 6: 扩展功能 (3-4 周)

**目标**: 实现高级特性和集成

**当前状态**: ⏳ 未开始

#### 6.1 技能系统 (Skills)
- [ ] 技能定义格式
- [ ] 技能加载和执行
- [ ] 内置技能实现
  - `commit`: Git 提交工作流
  - `review-pr`: PR 审查
  - `feature-dev`: 功能开发引导
- [ ] 自定义技能支持

#### 6.2 MCP 服务器支持
- [ ] MCP 协议实现
- [ ] 服务器发现和连接
- [ ] 工具代理
- [ ] 资源访问

#### 6.3 IDE 集成
- [ ] LSP 诊断获取 (`getDiagnostics`)
- [ ] Jupyter 内核支持 (`executeCode`)
- [ ] 编辑器状态同步

#### 6.4 会话管理
- [ ] 会话持久化
- [ ] 会话恢复
- [ ] 上下文自动压缩
- [ ] 多会话支持

---

### Phase 7: 优化与完善 (持续)

**目标**: 性能优化和用户体验提升

**当前状态**: ⏳ 未开始

#### 7.1 性能优化
- [ ] 并行工具执行
- [ ] 缓存策略优化
- [ ] 内存使用优化
- [ ] 启动时间优化

#### 7.2 测试覆盖
- [ ] 单元测试
- [ ] 集成测试
- [ ] 端到端测试
- [ ] 性能基准测试

#### 7.3 文档完善
- [ ] API 文档
- [ ] 用户指南
- [ ] 开发者文档
- [ ] 示例和教程

#### 7.4 安全性
- [ ] 沙箱执行环境
- [ ] 敏感信息过滤
- [ ] 权限系统
- [ ] 审计日志

---

## 🏗️ 技术架构

### 核心依赖

```toml
[dependencies]
# AI 框架
rig-core = "0.x"

# 异步运行时
tokio = { version = "1", features = ["full"] }

# CLI
reedline = "0.x"
clap = { version = "4", features = ["derive"] }

# 序列化
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# 错误处理
anyhow = "1"
thiserror = "1"

# 日志
tracing = "0.1"
tracing-subscriber = "0.3"

# 文件操作
walkdir = "2"
ignore = "0.4"  # 遵循 .gitignore

# 代码搜索
grep = "0.3"  # ripgrep 库

# HTTP
reqwest = { version = "0.11", features = ["json", "stream"] }

# Markdown 渲染
termimad = "0.x"

# 配置
config = "0.13"
directories = "5"
```

### 模块结构

```
oxide/
├── crates/
│   ├── oxide-core/          # 核心类型和 trait
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── error.rs
│   │   │   ├── config.rs
│   │   │   └── types.rs
│   │   └── Cargo.toml
│   │
│   ├── oxide-provider/      # LLM 提供商
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── anthropic.rs
│   │   │   └── traits.rs
│   │   └── Cargo.toml
│   │
│   ├── oxide-tools/         # 工具系统
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── registry.rs
│   │   │   ├── file.rs      # Read, Write, Edit
│   │   │   ├── search.rs    # Glob, Grep
│   │   │   ├── exec.rs      # Bash
│   │   │   └── web.rs       # WebFetch
│   │   └── Cargo.toml
│   │
│   ├── oxide-agent/         # 代理系统
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── agent.rs
│   │   │   ├── subagent.rs
│   │   │   └── task.rs
│   │   └── Cargo.toml
│   │
│   └── oxide-cli/           # CLI 界面
│       ├── src/
│       │   ├── main.rs
│       │   ├── repl.rs
│       │   ├── render.rs
│       │   └── commands.rs
│       └── Cargo.toml
│
├── docs/                    # 文档
│   ├── roadmap.md          # 本文档
│   ├── architecture.md     # 架构设计
│   └── api.md              # API 文档
│
├── Cargo.toml              # Workspace 配置
└── README.md
```

---

## 🎨 设计原则

### 1. 模块化设计
- 每个功能模块独立开发和测试
- 清晰的接口定义
- 最小化模块间依赖

### 2. 性能优先
- 利用 Rust 的零成本抽象
- 异步 I/O 处理所有网络和文件操作
- 并行执行独立的工具调用

### 3. 可扩展性
- 插件式工具系统
- 自定义技能支持
- 灵活的配置机制

### 4. 用户体验
- 清晰的错误消息
- 流畅的交互反馈
- 智能的默认行为

---

## 📊 里程碑

| 阶段 | 目标 | 预期成果 |
|------|------|----------|
| Phase 0 | 基础设施 | 可运行的项目骨架 |
| Phase 1 | LLM 集成 | 能与 Claude API 对话 |
| Phase 2 | 核心工具 | 基本的文件和代码操作 |
| Phase 3 | 高级功能 | 子代理和计划模式 |
| Phase 4 | CLI 界面 | 完整的交互体验 |
| Phase 5 | Git 集成 | 智能的版本控制 |
| Phase 6 | 扩展功能 | 技能和 MCP 支持 |
| Phase 7 | 优化完善 | 生产就绪 |

---

## 🚀 快速启动建议

### 第一步：从 Phase 0 开始
1. 设置 workspace 结构
2. 实现基础配置系统
3. 建立错误处理框架

### 第二步：实现最小可用产品 (MVP)
专注于核心功能：
- Anthropic API 集成
- Read, Write, Edit 工具
- 基础 CLI 界面

### 第三步：迭代增强
根据实际使用反馈，逐步添加高级功能。

---

## 📝 注意事项

### 安全性
- 永远不要在代码中硬编码 API Key
- 实现工具执行的权限检查
- 过滤敏感信息输出

### 兼容性
- 保持与 Claude Code 工具格式的兼容
- 支持跨平台运行（macOS, Linux, Windows）

### 性能
- 避免阻塞主线程
- 合理使用缓存
- 监控内存使用

---

## 🤝 贡献指南

欢迎贡献！请遵循以下原则：
- 保持代码简洁和可读
- 编写测试覆盖新功能
- 更新相关文档
- 遵循 Rust 最佳实践

---

**最后更新**: 2026-01-30

## 📈 当前进度总结

### ✅ 已完成
- **Phase 0 (基础设施)**: 90% 完成
  - 项目结构、配置系统、错误处理、会话管理已完成
  - 待完成: CI/CD 流程、日志系统

- **Phase 1 (LLM 集成)**: ✅ 100% 完成
  - Provider 抽象、Anthropic API 客户端、流式响应已完成
  - 支持自定义 Base URL (OXIDE_BASE_URL) 和 API Key (OXIDE_AUTH_TOKEN)
  - 消息类型、工具调用格式、多模态内容支持已完成

- **Phase 4 (CLI 界面)**: 85% 完成
  - Reedline 编辑器、命令系统、渲染器、状态栏已完成
  - 待完成: 部分快捷命令 (/compact, /tasks, /config)

### 🚧 进行中
- **Phase 2 (核心工具)**: 95% 完成
  - 工具框架已完成
  - 文件操作工具已完成 (Read, Write, Edit)
  - 搜索工具已完成 (Glob, Grep)
  - 执行工具已完成 (Bash, TaskOutput, TaskStop)
  - 代理主循环已完成（工具调用、流式输出）
  - 待完成: 网页获取 (WebFetch)

- **Phase 3 (高级功能)**: 40% 完成
  - 任务管理系统已完成 (TaskCreate, TaskList, TaskGet, TaskUpdate)
  - 任务依赖关系管理和循环依赖检测已完成
  - 计划模式已完成 (EnterPlanMode, ExitPlanMode, 权限请求系统)
  - 待完成: 子代理系统、用户交互工具完善

### ⏳ 未开始
- **Phase 3 (高级功能 - 剩余部分)**: 子代理系统、计划模式、用户交互
- **Phase 5 (Git 集成)**: Git 操作、GitHub 集成
- **Phase 6 (扩展功能)**: 技能系统、MCP 支持、IDE 集成
- **Phase 7 (优化完善)**: 性能优化、测试、文档、安全性

### 🎯 下一步重点
1. ✅ ~~完成 Anthropic API 实际调用和流式响应~~
2. ✅ ~~实现核心文件操作工具 (Read, Write, Edit)~~
3. ✅ ~~实现命令执行工具 (Bash)~~
4. ✅ ~~实现搜索工具 (Glob, Grep)~~
5. ✅ ~~完成代理主循环，实现工具调用~~
6. ✅ ~~实现任务管理系统 (TaskCreate, TaskList, TaskGet, TaskUpdate)~~
7. ✅ ~~实现计划模式 (EnterPlanMode, ExitPlanMode)~~
8. 完善用户交互工具 (AskUserQuestion) - 已有基础实现
9. 实现子代理系统 (Task Tool)
10. 实现网页获取工具 (WebFetch)
11. 端到端测试和优化

