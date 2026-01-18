# Project Context

## Purpose
Oxide 是一个支持任意 AI 模型的通用 AI Agent CLI 工具，提供对话式编程助手功能。类似于 claude code、codex、gemini-cli 等工具，用户可以通过命令行界面与 AI 交互，AI 可以使用文件操作和 Shell 命令执行工具来完成任务。设计目标是支持多种 AI 提供商的模型，当前实现使用 DeepSeek API 作为示例。

## Tech Stack
- **Rust** (Edition 2021) - 主要编程语言
- **Tokio** (v1.40) - 异步运行时
- **reqwest** (v0.12) - HTTP 客户端，支持与多种 AI 提供商 API 交互
- **serde/serde_json** (v1.0) - 序列化和反序列化，用于 API 请求响应
- **clap** (v4.5) - 命令行参数解析
- **dotenv** (v0.15) - 环境变量加载（API Keys、配置等）
- **colored** (v2.1) - 终端颜色输出
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
- **Provider 抽象** - 设计支持多种 AI 提供商，当前实现使用 Anthropic API
- **Agent 模式** - Agent 结构体管理状态（客户端、API 密钥、消息历史、工具定义）
- **消息驱动** - 基于 ContentBlock 类型系统（Text, ToolUse, ToolResult）
- **工具执行循环** - 异步处理用户输入 → 发送消息 → 执行工具 → 返回结果
- **模块化工具** - 工具实现分离到独立模块
- **可扩展性** - 易于添加新的 AI 提供商和工具

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
- **工具系统** - AI 可以调用预定义的工具（read_file, write_file, shell_execute）
- **模型抽象** - 设计为支持多种 AI 提供商（Anthropic、OpenAI、Google 等）
- **状态管理** - Agent 维护对话历史，支持多轮对话
- **彩色输出** - 使用不同颜色区分用户输入、AI 响应、工具调用等

## Important Constraints
- **API 限制**: 受所选 AI 提供商的速率限制和配额约束
- **模型兼容性**: 需要支持不同提供商的 API 格式和响应结构
- **单机部署**: 设计为本地 CLI 工具，不涉及服务器部署
- **环境变量**: 必须设置相应提供商的 API Key 环境变量
- **异步运行**: 使用 Tokio 异步运行时，所有 I/O 操作必须异步

## External Dependencies
- **AI 提供商 API** - 当前实现使用 DeepSeek API 作为示例
  - 当前模型: `deepseek-chat` 或 `deepseek-coder`
  - API 端点: `https://api.deepseek.com/v1/chat/completions`（OpenAI 兼容格式）
  - 认证: 需要提供商特定的 API Key（如 `DEEPSEEK_API_KEY`）
  - 未来计划支持: Anthropic Claude、OpenAI、Google Gemini、其他 AI 提供商
