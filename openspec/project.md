# Project Context

## Purpose
Oxide 是一个支持任意 AI 模型的通用 AI Agent CLI 工具，提供对话式编程助手功能。类似于 claude code、codex、gemini-cli 等工具，用户可以通过命令行界面与 AI 交互，AI 可以使用文件操作和 Shell 命令执行工具来完成任务。设计目标是支持多种 AI 提供商的模型，当前实现使用 DeepSeek API 作为示例。

## Tech Stack
- **Rust** (Edition 2021) - 主要编程语言
- **Tokio** (v1.40) - 异步运行时
- **rig-core** (v0.28.0) - AI Agent 框架，支持多种 LLM 提供商
- **reqwest** (v0.12) - HTTP 客户端，支持与多种 AI 提供商 API 交互
- **serde/serde_json** (v1.0) - 序列化和反序列化，用于 API 请求响应
- **toml** (v0.8) - TOML 配置文件解析
- **dotenv** (v0.15) - 环境变量加载（API Keys、配置等）
- **colored** (v3.0) - 终端颜色输出
- **anyhow** (v1.0) - 错误处理

## Project Conventions

### Code Style
- 使用 Rust 2021 edition 规范
- 函数和变量使用 `snake_case`
- 结构体和枚举使用 `PascalCase`
- 错误处理使用 `anyhow::Result` 和 `.context()` 方法
- 终端输出使用 `colored` 库进行美化（.cyan(), .yellow(), .green() 等）
- 模块分离：工具实现放在 `tools.rs` 中

### Architecture Patterns
- **单体 CLI 应用** - 单一可执行文件
- **Provider 抽象** - 支持 Anthropic Claude 和 OpenAI 兼容 API
- **Agent 模式** - 使用 AgentBuilder + AgentEnum 构建，支持多种 Agent 类型（Main, Explore, Plan, CodeReviewer, FrontendDeveloper）
- **消息驱动** - 基于 rig 库的 Message 类型系统
- **工具执行循环** - 异步处理用户输入 → 发送消息 → 执行工具 → 返回结果
- **模块化工具** - 工具实现分离到独立模块
- **可扩展性** - 易于添加新的 AI 提供商（通过 OpenAI 兼容接口）和工具

### Testing Strategy
- 使用 Rust 标准测试框架
- 单元测试覆盖工具模块功能
- 集成测试覆盖 Agent 消息流
- Mock API 响应以避免实际 API 调用

### Git Workflow
- **分支策略**: 主分支 `main`，功能分支使用 `feat/` 前缀
- **Commit 规范**: 遵循 Conventional Commits
  - `feat:` - 新功能
  - `fix:` - Bug 修复
  - `refactor:` - 重构
  - `docs:` - 文档更新
  - `test:` - 测试相关
  - `chore:` - 构建/配置变更
- **变更管理**: 使用 OpenSpec 进行提案、规范和变更管理

## Domain Context
Oxide 是一个通用 AI 编程助手 CLI，核心概念包括：
- **消息循环** - 用户输入 → AI 响应 → 工具执行 → 结果返回
- **工具系统** - AI 可以调用预定义的工具（9 个核心工具 + 额外工具）
- **模型抽象** - 使用 rig 库支持 Anthropic 和 OpenAI 兼容 API
- **状态管理** - Agent 维护对话历史，支持多轮对话
- **彩色输出** - 使用不同颜色区分用户输入、AI 响应、工具调用等
- **多 Agent 架构** - 支持 Main, Explore, Plan, CodeReviewer, FrontendDeveloper 等 Agent 类型

## Important Constraints
- **API 限制**: 受所选 AI 提供商的速率限制和配额约束
- **模型兼容性**: 需要支持不同提供商的 API 格式和响应结构
- **单机部署**: 设计为本地 CLI 工具，不涉及服务器部署
- **环境变量**: 必须设置相应提供商的 API Key 环境变量
- **异步运行**: 使用 Tokio 异步运行时，所有 I/O 操作必须异步

## External Dependencies
- **AI 提供商 API** - 当前实现支持 Anthropic Claude 和 OpenAI 兼容 API
  - 当前默认模型: `claude-sonnet-4-20250514`
  - API 端点: `https://api.anthropic.com` (默认) 或自定义 OpenAI 兼容端点
  - 认证: 需要提供商特定的 API Key（推荐使用 `OXIDE_AUTH_TOKEN` 环境变量）
  - Provider 判断: 基于 `OXIDE_BASE_URL` 中是否包含 "anthropic" 字符串
  - 未来计划: 通过 OpenAI 兼容层支持更多提供商（如 DeepSeek、Google Gemini 等）
