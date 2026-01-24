# cli-core Specification

## MODIFIED Requirements

### Requirement: 斜杠命令系统
CLI SHALL 支持斜杠命令执行特殊操作，包括内置命令和用户自定义 Skills。

#### Scenario: /help 命令
- **WHEN** 用户输入 "/help"
- **THEN** 显示所有可用斜杠命令列表（包括内置命令和自定义 Skills）
- **AND** 显示每个命令的简短描述

#### Scenario: /clear 命令
- **WHEN** 用户输入 "/clear"
- **THEN** 清空对话历史
- **AND** 显示确认消息
- **AND** 重置为新的对话会话

#### Scenario: /exit 命令
- **WHEN** 用户输入 "/exit"
- **THEN** 显示告别消息（青色）
- **AND** 程序正常退出并返回状态码 0

#### Scenario: /skills list 命令
- **WHEN** 用户输入 "/skills" 或 "/skills list"
- **THEN** 显示所有可用 Skills 列表
- **AND** 标记每个 Skill 的来源（内置、全局、本地）
- **AND** 显示 Skill 名称和描述

#### Scenario: /skills show 命令
- **WHEN** 用户输入 "/skills show <name>"
- **THEN** 显示指定 Skill 的详细信息
- **AND** 包括 Skill 的来源、参数列表、模板内容

#### Scenario: Skill 命令执行
- **WHEN** 用户输入 "/<skill-name> [args]"
- **AND** Skill 存在于技能库中
- **THEN** 加载 Skill 模板
- **AND** 使用提供的参数渲染模板
- **AND** 将渲染后的内容作为用户消息发送给 AI

#### Scenario: 未知命令
- **WHEN** 用户输入未知的斜杠命令
- **THEN** 显示错误提示
- **AND** 提示用户使用 /help 查看可用命令
- **AND** 建议相似的可用命令（模糊匹配）

## ADDED Requirements

### Requirement: Skill 自动补全
CLI SHALL 在用户输入斜杠命令时提供自动补全建议，包括内置命令和自定义 Skills。

#### Scenario: 命令补全
- **WHEN** 用户输入 "/" 并按 Tab 键
- **THEN** 显示所有可用命令和 Skills
- **AND** 命令按名称排序
- **AND** 包含命令描述

#### Scenario: 模糊补全
- **WHEN** 用户输入部分命令（如 "/com"）并按 Tab 键
- **THEN** 显示匹配的命令和 Skills（如 /commit, /compact）
- **AND** 允许用户选择完整命令

### Requirement: Skill 自动加载
CLI SHALL 在启动时自动加载所有可用的 Skills。

#### Scenario: 正常加载
- **WHEN** Oxide 启动
- **THEN** 扫描 `.oxide/skills/` 目录（本地）
- **AND** 扫描 `~/.oxide/skills/` 目录（全局）
- **AND** 加载内置 Skills
- **AND** 本地 Skills 覆盖同名全局 Skills
- **AND** 缓存所有 Skills 到内存

#### Scenario: 目录不存在
- **WHEN** Skills 目录不存在
- **THEN** 静默跳过该目录
- **AND** 继续加载其他位置的 Skills
- **AND** 不显示错误或警告

#### Scenario: Skill 文件格式错误
- **WHEN** 某个 Skill 文件格式错误
- **THEN** 跳过该 Skill 文件
- **AND** 在启动时显示警告（文件路径和错误原因）
- **AND** 继续加载其他有效 Skills
