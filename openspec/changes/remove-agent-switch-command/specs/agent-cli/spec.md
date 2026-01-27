## ADDED Requirements

### Requirement: Agent 命令仅支持查看能力

CLI 中 `/agent` 命令 MUST 仅支持 `list` 与 `capabilities` 子命令，用于查看可用的 Agent 类型与能力信息。

#### Scenario: 查看 agent 列表

- **WHEN** 用户输入 `/agent` 或 `/agent list`
- **THEN** 系统显示所有可用 Agent 类型与能力说明

#### Scenario: 查看 agent 能力

- **WHEN** 用户输入 `/agent capabilities`
- **THEN** 系统显示各 Agent 类型的能力与工具列表

### Requirement: 不支持手动切换 Agent

CLI MUST 不提供 `/agent switch` 子命令，且帮助文档中不得提示可手动切换 Agent 类型。

#### Scenario: 用户尝试使用 /agent switch

- **WHEN** 用户输入 `/agent switch <type>`
- **THEN** 系统提示该命令不被支持或视为未知命令
