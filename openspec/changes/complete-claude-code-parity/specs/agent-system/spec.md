# agent-system Specification

## Purpose
定义 Oxide 的 Agent 系统，实现类似 Claude Code 的 Subagent 架构。Agent 系统提供专业化的 AI 能力，每个 Agent 针对特定任务类型优化。

## ADDED Requirements

### Requirement: Agent 类型定义
系统 SHALL 定义多种 Agent 类型，每种类型具有特定的能力和工具权限。

#### Scenario: 基础 Agent 类型
- **GIVEN** Oxide 系统启动
- **THEN** 系统应支持以下 Agent 类型：
  - `Main` - 主对话 Agent，访问所有工具
  - `Explore` - 代码库探索 Agent，只读工具
  - `Plan` - 架构规划 Agent，只读 + 计划工具
  - `CodeReviewer` - 代码审查 Agent，只读工具
  - `FrontendDeveloper` - 前端开发 Agent，完整工具集
  - `GeneralPurpose` - 通用任务 Agent

#### Scenario: Agent 能力描述
- **WHEN** 查询 Agent 能力
- **THEN** 返回 Agent 的以下信息：
  - 名称和描述
  - 可用工具列表
  - 系统提示词
  - 适用场景

### Requirement: Subagent 管理
系统 SHALL 提供 SubagentManager 来管理多个 Agent 实例。

#### Scenario: 注册 Subagent
- **GIVEN** SubagentManager 实例
- **WHEN** 注册新的 Agent
- **THEN** Agent 被添加到管理器
- **AND** 可以通过类型检索该 Agent

#### Scenario: 切换 Agent
- **GIVEN** 当前活跃的 Agent
- **WHEN** 切换到另一个 Agent 类型
- **THEN** 当前 Agent 被暂停
- **AND** 新 Agent 成为活跃 Agent
- **AND** 上下文被保留

#### Scenario: 列出 Agent 能力
- **WHEN** 用户执行 `/agent list` 命令
- **THEN** 显示所有可用的 Agent 类型
- **AND** 显示每个 Agent 的简短描述

### Requirement: Agent 创建和初始化
系统 SHALL 能够根据配置动态创建 Agent 实例。

#### Scenario: 使用默认配置创建 Agent
- **GIVEN** 有效的 API 配置
- **WHEN** 创建 Main Agent
- **THEN** Agent 使用系统默认配置
- **AND** Agent 可以正常响应消息

#### Scenario: 使用自定义配置创建 Agent
- **GIVEN** Agent 特定配置
- **WHEN** 创建 Explore Agent
- **THEN** Agent 使用 Explore 特定的模型和提示词
- **AND** Agent 只能访问只读工具

### Requirement: 工具权限管理
每个 Agent SHALL 只能访问其权限范围内的工具。

#### Scenario: Explore Agent 工具限制
- **GIVEN** Explore Agent 实例
- **WHEN** Agent 尝试写入文件
- **THEN** 操作被拒绝
- **AND** 返回权限错误

#### Scenario: Main Agent 完整权限
- **GIVEN** Main Agent 实例
- **WHEN** Agent 执行任何工具
- **THEN** 操作被允许
- **AND** 工具正常执行

### Requirement: Agent 路由决策
系统 SHALL 能够根据任务类型自动选择合适的 Agent。

#### Scenario: 自动路由到 Explore Agent
- **GIVEN** 用户输入："探索这个项目的目录结构"
- **WHEN** 系统分析任务类型
- **THEN** 自动选择 Explore Agent
- **AND** 使用 Explore Agent 执行任务

#### Scenario: 自动路由到 Plan Agent
- **GIVEN** 用户输入："帮我规划如何实现用户认证"
- **WHEN** 系统分析任务类型
- **THEN** 自动选择 Plan Agent
- **AND** 使用 Plan Agent 创建实施计划

#### Scenario: 手动指定 Agent
- **GIVEN** 用户输入："使用 Code Reviewer 审查这段代码"
- **WHEN** 系统检测到明确的 Agent 指定
- **THEN** 使用指定的 Code Reviewer Agent
- **AND** 执行代码审查任务

### Requirement: Agent 系统提示词
每个 Agent SHALL 有专门定制的系统提示词。

#### Scenario: Explore Agent 提示词
- **GIVEN** Explore Agent
- **WHEN** 查询系统提示词
- **THEN** 提示词包含：
  - 角色定义（代码库探索专家）
  - 工具使用指导
  - 输出格式要求
  - 只读约束说明

#### Scenario: Plan Agent 提示词
- **GIVEN** Plan Agent
- **WHEN** 查询系统提示词
- **THEN** 提示词包含：
  - 角色定义（架构规划专家）
  - 计划制定方法论
  - 风险评估指导
  - 实施步骤格式要求

### Requirement: /agent 斜杠命令
系统 SHALL 提供 `/agent` 命令来管理 Agent。

#### Scenario: /agent list
- **WHEN** 用户输入 `/agent list`
- **THEN** 显示所有可用的 Agent
- **AND** 显示当前活跃的 Agent
- **AND** 显示每个 Agent 的状态

#### Scenario: /agent switch
- **WHEN** 用户输入 `/agent switch explore`
- **THEN** 切换到 Explore Agent
- **AND** 显示确认消息
- **AND** 更新提示符显示当前 Agent

#### Scenario: /agent capabilities
- **WHEN** 用户输入 `/agent capabilities`
- **THEN** 显示当前 Agent 的能力
- **AND** 列出可用的工具
- **AND** 显示系统提示词摘要

### Requirement: Agent 生命周期管理
系统 SHALL 正确管理 Agent 的创建、切换和销毁。

#### Scenario: Agent 创建
- **GIVEN** 首次请求特定 Agent 类型
- **WHEN** 创建 Agent 实例
- **THEN** Agent 被正确初始化
- **AND** 资源被分配
- **AND** Agent 准备就绪

#### Scenario: Agent 缓存
- **GIVEN** 已经创建过的 Agent
- **WHEN** 再次请求该 Agent 类型
- **THEN** 返回缓存的 Agent 实例
- **AND** 避免重复创建开销

#### Scenario: Agent 上下文隔离
- **GIVEN** 两个不同的 Agent
- **WHEN** 各自执行任务
- **THEN** Agent 之间的上下文相互独立
- **AND** 不会相互干扰

### Requirement: Agent 任务执行
Agent SHALL 能够执行任务并返回结果。

#### Scenario: 同步任务执行
- **GIVEN** 用户输入和活跃的 Agent
- **WHEN** 执行同步任务
- **THEN** Agent 处理任务
- **AND** 返回结果给用户
- **AND** 等待任务完成后才接受新输入

#### Scenario: 异步任务执行
- **GIVEN** 用户请求后台任务
- **WHEN** 使用 Task 工具创建后台任务
- **THEN** Agent 在后台执行任务
- **AND** 立即返回任务 ID
- **AND** 主进程继续接受用户输入

### Requirement: Agent 配置
Agent 配置 SHALL 支持多级覆盖。

#### Scenario: 全局默认配置
- **GIVEN** ~/.oxide/config.toml 中的默认配置
- **WHEN** 创建 Agent
- **THEN** Agent 使用全局默认配置

#### Scenario: 项目特定配置
- **GIVEN** .oxide/config.toml 中的项目配置
- **WHEN** 创建 Agent
- **THEN** 项目配置覆盖全局配置
- **AND** Agent 使用项目特定设置

#### Scenario: 运行时配置
- **GIVEN** 环境变量设置
- **WHEN** 创建 Agent
- **THEN** 环境变量优先级最高
- **AND** Agent 使用运行时配置
