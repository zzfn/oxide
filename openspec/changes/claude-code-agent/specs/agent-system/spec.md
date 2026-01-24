## MODIFIED Requirements

### Requirement: Agent 类型定义
系统 SHALL 定义多种 Agent 类型以适应不同的使用场景。

#### Scenario: Agent 类型枚举扩展
- **WHEN** 定义 `AgentType` 枚举
- **THEN** SHALL 包含 `ClaudeCode` 变体
- **AND** 变体值 SHALL 为 `AgentType::ClaudeCode`
- **AND** SHALL 支持通过字符串解析（"claude-code"、"claude"）

#### Scenario: ClaudeCode Agent 显示名称
- **WHEN** 调用 `AgentType::ClaudeCode.display_name()`
- **THEN** 返回 "Claude Code"
- **AND** 名称 SHALL 用于 CLI 显示和日志

#### Scenario: ClaudeCode Agent 描述
- **WHEN** 调用 `AgentType::ClaudeCode.description()`
- **THEN** 返回描述："外部工具 Agent，使用 Claude Code 执行任务"
- **AND** 描述 SHALL 说明这是委托给外部工具

## ADDED Requirements

### Requirement: ClaudeCode Agent
系统 SHALL 支持 `ClaudeCode` Agent 类型，该 Agent 将任务委托给 Claude Code CLI 工具执行。

#### Scenario: 创建 ClaudeCode Agent
- **WHEN** 使用 `AgentBuilder::build_claude_code()` 方法
- **THEN** 系统 SHALL 创建 `ClaudeCodeAgent` 实例
- **AND** Agent SHALL 配置为委托模式
- **AND** Agent SHALL 包含系统提示词

#### Scenario: ClaudeCode Agent 系统提示词
- **WHEN** ClaudeCode Agent 被创建
- **THEN** 系统提示词 SHALL 说明此 Agent 会委托任务给 Claude Code
- **AND** 提示词 SHALL 告知用户任务将由 Claude Code 执行
- **AND** 提示词 SHALL 说明输出将被解析为结构化 JSON

#### Scenario: ClaudeCode Agent 工具配置
- **WHEN** 使用 ClaudeCode Agent
- **THEN** Agent SHALL 不激活 Oxide 的工具系统
- **AND** Agent SHALL 依赖 Claude Code 的工具执行
- **AND** 避免 Oxide 和 Claude Code 之间的工具冲突

### Requirement: AgentEnum 扩展
`AgentEnum` SHALL 包含 ClaudeCode Agent 变体。

#### Scenario: AgentEnum 定义
- **WHEN** 定义 `AgentEnum` 枚举
- **THEN** SHALL 包含 `ClaudeCode(ClaudeCodeAgent)` 变体
- **AND** `ClaudeCodeAgent` SHALL 管理 Claude Code 进程和输出解析

#### Scenario: 获取 ClaudeCode Agent 类型名称
- **WHEN** 调用 `AgentEnum::type_name()` 方法
- **AND** Agent 是 ClaudeCode 类型
- **THEN** 返回 "claude-code"
- **AND** 名称 SHALL 可用于日志、配置和显示

### Requirement: Agent 构建
系统 SHALL 支持构建不同类型的 Agent。

#### Scenario: 构建指定类型的 Agent
- **WHEN** 调用 `AgentBuilder::build_with_type(AgentType::ClaudeCode)`
- **THEN** 系统 SHALL 调用 `build_claude_code()` 方法
- **AND** 返回配置好的 ClaudeCode Agent
- **AND** 如果 Claude Code 不可用，返回错误

#### Scenario: 直接构建 ClaudeCode Agent
- **WHEN** 调用 `AgentBuilder::build_claude_code()` 方法
- **THEN** 系统 SHALL 验证 Claude Code 是否已安装
- **AND** 系统 SHALL 创建 `ClaudeCodeAgent` 实例
- **AND** 系统 SHALL 配置必要的参数（工作目录、环境变量等）

### Requirement: Agent 状态管理
系统 SHALL 管理当前使用的 Agent 类型。

#### Scenario: 切换到 ClaudeCode Agent
- **WHEN** 用户执行 `/agent switch claude-code` 或类似命令
- **THEN** 系统 SHALL 创建 ClaudeCode Agent
- **AND** 系统 SHALL 替换当前活动的 Agent
- **AND** 系统 SHALL 更新内部状态记录

#### Scenario: 从 ClaudeCode Agent 切换回其他 Agent
- **WHEN** 用户切换到其他 Agent 类型
- **THEN** 系统 SHALL 停止 Claude Code 进程（如正在运行）
- **AND** 系统 SHALL 清理 ClaudeCode Agent 资源
- **AND** 系统 SHALL 创建新的 Agent 实例

### Requirement: ClaudeCode Agent 验证
系统 SHALL 在创建 ClaudeCode Agent 前验证必要条件。

#### Scenario: 验证 Claude Code 可用性
- **WHEN** 尝试创建 ClaudeCode Agent
- **THEN** 系统 SHALL 检查 Claude Code 是否已安装
- **AND** 系统 SHALL 验证 Claude Code 可执行文件路径
- **AND** 如果验证失败，返回错误并说明原因

#### Scenario: 验证配置有效性
- **WHEN** 尝试创建 ClaudeCode Agent
- **THEN** 系统 SHALL 验证相关配置
- **AND** 系统 SHALL 检查输出目录是否可写（如配置了导出）
- **AND** 如果配置无效，返回错误并说明原因

#### Scenario: 验证失败时的提示
- **WHEN** ClaudeCode Agent 验证失败
- **THEN** 系统 SHALL 提供清晰的错误消息
- **AND** 系统 SHALL 建议解决方案（如安装 Claude Code）
- **AND** 系统 SHALL 提供配置指引
