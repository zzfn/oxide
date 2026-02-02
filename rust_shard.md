# 用 Rust 复刻 Claude Code

## 1. 项目目标

通过复刻 Claude Code 理解 AI Agent 的核心机制：

- **Agent 决策循环**：思考-行动-观察的闭环设计
- **工具调用协调**：Tool 接口定义、LLM 调用、错误处理
- **权限与安全**：自动化与用户控制的平衡
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
