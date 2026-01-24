## MODIFIED Requirements

### Requirement: 斜杠命令系统
CLI SHALL 支持斜杠命令执行特殊操作。

#### Scenario: /bridge 命令 - 启用桥接模式
- **WHEN** 用户输入 "/bridge on"
- **THEN** 系统验证 Claude Code 是否已安装
- **AND** 如果已安装，启用桥接模式并显示确认消息
- **AND** 在命令提示符中显示 "[Bridge]" 标识
- **AND** 后续任务将通过 Claude Code 执行

#### Scenario: /bridge 命令 - 禁用桥接模式
- **WHEN** 用户输入 "/bridge off"
- **THEN** 禁用桥接模式并显示确认消息
- **AND** 从命令提示符中移除 "[Bridge]" 标识
- **AND** 后续任务将使用 Oxide 自己的 Agent

#### Scenario: /bridge 命令 - 查看状态
- **WHEN** 用户输入 "/bridge"（不带参数）
- **THEN** 显示当前桥接模式状态（启用/禁用）
- **AND** 显示 Claude Code 可执行文件路径
- **AND** 显示结构化输出保存目录
- **AND** 如果 Claude Code 未安装，显示提示信息

#### Scenario: /bridge 命令 - Claude Code 未安装
- **WHEN** 用户尝试启用桥接模式但 Claude Code 未安装
- **THEN** 显示错误消息（使用着色）
- **AND** 提供安装 Claude Code 的指引
- **AND** 提供配置 Claude Code 路径的指引
- **AND** 保持当前模式不变

#### Scenario: /export 命令 - 导出 JSON
- **WHEN** 用户输入 "/export json"
- **THEN** 将当前会话的结构化输出保存为 JSON 文件
- **AND** 显示保存的文件路径
- **AND** 如果没有结构化输出，显示提示信息

#### Scenario: /export 命令 - 导出 Markdown
- **WHEN** 用户输入 "/export markdown"
- **THEN** 将当前会话的工具调用结果格式化为 Markdown
- **AND** 保存到文件并显示文件路径
- **AND** 如果没有结构化输出，显示提示信息

#### Scenario: /export 命令 - 无效格式
- **WHEN** 用户输入 "/export <无效格式>"
- **THEN** 显示错误提示
- **AND** 列出支持的导出格式（json、markdown）
- **AND** 提示使用 /help 查看更多信息

#### Scenario: /help 命令 - 显示新增命令
- **WHEN** 用户输入 "/help"
- **THEN** 在命令列表中包含 /bridge 和 /export
- **AND** 显示每个命令的简短描述
- **AND** 显示命令的用法示例

## ADDED Requirements

### Requirement: Prompt 增强
CLI SHALL 在命令提示符中显示当前桥接模式状态。

#### Scenario: 桥接模式启用时的 Prompt
- **WHEN** 桥接模式已启用
- **THEN** 命令提示符 SHALL 包含 "[Bridge]" 标识
- **AND** 标识 SHALL 使用特殊颜色（如黄色）显示
- **AND** 提示符格式示例：`你>[Bridge][0] `

#### Scenario: 普通模式时的 Prompt
- **WHEN** 桥接模式未启用
- **THEN** 命令提示符 SHALL 不包含 "[Bridge]" 标识
- **AND** 提示符保持原有格式：`你>[0] `

### Requirement: 桥接模式切换反馈
CLI SHALL 在用户切换桥接模式时提供清晰的视觉反馈。

#### Scenario: 启用桥接成功
- **WHEN** 用户成功启用桥接模式
- **THEN** 显示成功消息（使用绿色着色）
- **AND** 显示 Claude Code 版本信息（如可获取）
- **AND** 提示任务将委托给 Claude Code

#### Scenario: 禁用桥接成功
- **WHEN** 用户成功禁用桥接模式
- **THEN** 显示确认消息（使用青色着色）
- **AND** 提示任务将使用 Oxide Agent

#### Scenario: 桥接模式切换失败
- **WHEN** 桥接模式切换失败
- **THEN** 显示错误消息（使用红色着色）
- **AND** 提供失败原因
- **AND** 保持当前模式不变

### Requirement: 导出反馈
CLI SHALL 在用户导出结构化输出时提供反馈。

#### Scenario: 导出成功
- **WHEN** 成功导出结构化输出
- **THEN** 显示成功消息（使用绿色着色）
- **AND** 显示导出文件的完整路径
- **AND** 显示文件大小（如适用）

#### Scenario: 导出目录创建
- **WHEN** 导出目录不存在
- **THEN** 自动创建目录（包括父目录）
- **AND** 显示目录创建消息（使用青色着色）

#### Scenario: 导出失败
- **WHEN** 导出失败（如权限不足、磁盘已满）
- **THEN** 显示错误消息（使用红色着色）
- **AND** 提供具体的失败原因
- **AND** 建议解决方案（如检查权限、清理磁盘）
