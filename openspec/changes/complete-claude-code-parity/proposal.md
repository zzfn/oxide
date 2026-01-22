# Change: 完成 Claude Code 功能对等

## Why

Oxide 项目的目标是用 Rust 复刻 Claude Code 的功能。经过代码库分析，虽然已经实现了基础功能，但与 Claude Code 相比还缺少许多关键特性。需要系统性地识别并实现这些缺失功能，以达到功能对等。

当前已实现的功能：
- ✅ 基础 CLI 界面（rustyline + colored）
- ✅ 8 个核心工具（read, write, edit, delete, shell, grep, scan, mkdir）
- ✅ 会话管理（保存/加载/删除/列表）
- ✅ 多模型支持（Anthropic + OpenAI API）
- ✅ 流式响应输出
- ✅ 斜杠命令系统
- ✅ 命令历史和补全
- ✅ TUI 框架基础（ratatui，但功能不完整）

当前缺失的关键功能：
1. **Agent 系统**：Claude Code 的 Subagent（Explore、Plan、Code Reviewer 等）
2. **Task 工具**：后台任务管理和输出追踪
3. **完整 TUI 体验**：现代化的终端界面（已有部分实现）
4. **高级工具**：Glob、NotebookEdit、MultiEdit、AskUserQuestion
5. **Git 集成**：增强的 Git 工作流支持
6. **系统提示管理**：可配置的系统提示词
7. **配置文件系统**：持久化配置（类似 `.claude/CLAUDE.md`）
8. **MCP 服务器支持**：模型上下文协议集成
9. **图片分析**：多模态图片读取和分析
10. **Web 抓取**：WebFetch 和 WebSearch 工具

## What Changes

本变更包含以下主要改进：

### 1. Agent 系统（第一阶段）
- **MODIFIED**: `src/agent/` 模块扩展
- **ADDED**: Subagent 架构支持（Explore, Plan, Code Reviewer, Frontend Developer 等）
- **ADDED**: Agent 能力描述系统
- **ADDED**: Agent 生命周期管理
- **ADDED**: `/agent` 斜杠命令用于切换和管理 Agent

### 2. 任务管理工具
- **ADDED**: `Task` 工具实现，支持异步后台任务执行
- **ADDED**: `TaskOutput` 工具用于追踪后台任务输出
- **ADDED**: 任务状态持久化和恢复
- **ADDED**: `/tasks` 斜杠命令查看和管理任务

### 3. 高级文件工具
- **ADDED**: `Glob` 工具（文件模式匹配）
- **ADDED**: `MultiEdit` 工具（批量文件编辑）
- **ADDED**: `NotebookEdit` 工具（Jupyter notebook 编辑）
- **ADDED**: `AskUserQuestion` 工具（交互式用户确认）

### 4. 增强 Git 集成
- **MODIFIED**: `shell_execute` 工具添加 Git 安全检查
- **ADDED**: Git commit 规范验证
- **ADDED**: Git hook 集成支持
- **ADDED**: PR 创建命令集成

### 5. 配置和提示词系统
- **ADDED**: 项目级配置文件（`.oxide/CONFIG.md`）
- **ADDED**: 用户全局配置（`~/.oxide/config.toml`）
- **ADDED**: 可自定义的系统提示词模板
- **ADDED**: 多项目配置隔离

### 6. MCP 服务器支持（可选）
- **ADDED**: MCP 客户端基础实现
- **ADDED**: MCP 工具包装器
- **ADDED**: MCP 服务器配置管理
- **ADDED**: `/mcp` 斜杠命令管理 MCP 连接

### 7. 多模态支持（可选）
- **ADDED**: `Read` 工具支持图片读取（PNG, JPG, PDF）
- **ADDED**: `AnalyzeImage` 工具集成视觉模型
- **ADDED**: 图片显示和预览（TUI 模式）

### 8. Web 工具（可选）
- **ADDED**: `WebFetch` 工具（HTTP 请求和内容提取）
- **ADDED**: `WebSearch` 工具（搜索引擎集成）
- **ADDED**: HTML 转 Markdown 渲染

### 9. TUI 完善和优化
- **MODIFIED**: 完成现有 TUI 变更中的未完成任务
- **ADDED**: 主题系统完整实现
- **ADDED**: 虚拟滚动和性能优化
- **ADDED**: 帮助系统和快捷键

### 10. 测试和文档
- **ADDED**: 全面的单元测试覆盖
- **ADDED**: 集成测试套件
- **ADDED**: 用户文档更新
- **ADDED**: API 文档生成

## Impact

- Affected specs: `cli-core`（扩展交互需求）, `agent-system`（新增）, `advanced-tools`（新增）, `config-system`（新增）
- Affected code:
  - `src/agent/` - 大幅扩展
  - `src/tools/` - 新增 10+ 个工具
  - `src/config.rs` - 重构为完整配置系统
  - `src/cli/command.rs` - 新增斜杠命令
  - `src/tui/` - 完成未完成任务
- 新增依赖:
  - `glob` - 文件模式匹配
  - `reqwest` (已有) - HTTP 请求
  - `image` - 图片处理
  - `toml` - 配置文件解析
  - `serde` (已有) - 配置序列化
  - `tokio` (已有) - 异步任务管理
- **BREAKING**: 配置系统重构可能需要用户迁移配置
- 向后兼容: 保持现有 API 和命令行接口兼容

## Dependencies

- 依赖于 `modernize-tui-experience` 变更完成
- 可以与 `add-tui-interface` 并行开发部分功能
- 某些高级功能（MCP、多模态）可以延后实现

## Priority

分为三个优先级阶段：

**P0（必须完成）**：
1. Agent 系统基础架构
2. Task 工具
3. 高级文件工具（Glob, MultiEdit, AskUserQuestion）
4. 配置系统

**P1（重要功能）**：
5. 增强 Git 集成
6. TUI 体验完善
7. 测试覆盖

**P2（可选增强）**：
8. MCP 服务器支持
9. 多模态支持
10. Web 工具
