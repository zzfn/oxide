## ADDED Requirements

### Requirement: CLI 启动
CLI 工具 SHALL 成功启动并显示欢迎信息。

#### Scenario: 正常启动
- **WHEN** 用户执行 `oxide` 命令
- **THEN** 显示欢迎信息，包含项目名称和版本
- **AND** 显示当前使用的模型信息
- **AND** 显示可用斜杠命令提示（/help 查看帮助）
- **AND** 显示命令提示符，准备接收用户输入

#### Scenario: 缺少 API Key
- **WHEN** DEEPSEEK_API_KEY 环境变量未设置
- **THEN** 显示友好的错误信息
- **AND** 提供设置 API Key 的指引
- **AND** 程序退出并返回非零状态码

### Requirement: 交互式对话
CLI SHALL 支持多轮对话式交互，提供良好的视觉反馈。

#### Scenario: 发送消息
- **WHEN** 用户输入文本并按回车
- **THEN** 显示加载状态提示
- **AND** 将消息发送到 AI API
- **AND** 显示 AI 响应（使用青色着色）
- **AND** 等待下一次用户输入

#### Scenario: 响应格式化
- **WHEN** AI 响应包含 Markdown 格式
- **THEN** 保留代码块格式
- **AND** 支持基本 Markdown 语法高亮（标题、列表、代码块）
- **AND** 保持段落间距

#### Scenario: 错误处理
- **WHEN** API 请求失败
- **THEN** 显示清晰的错误信息（红色）
- **AND** 提供错误原因
- **AND** 保持对话会话，允许用户继续输入

### Requirement: 斜杠命令系统
CLI SHALL 支持斜杠命令执行特殊操作。

#### Scenario: /help 命令
- **WHEN** 用户输入 "/help"
- **THEN** 显示所有可用斜杠命令列表
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

#### Scenario: 未知命令
- **WHEN** 用户输入未知的斜杠命令
- **THEN** 显示错误提示
- **AND** 提示用户使用 /help 查看可用命令

### Requirement: 输入验证
CLI SHALL 对用户输入进行合理验证。

#### Scenario: 空输入
- **WHEN** 用户输入仅包含空白字符
- **THEN** 忽略该输入
- **AND** 继续显示提示符等待输入

#### Scenario: 超长输入
- **WHEN** 用户输入超过合理长度（如 10000 字符）
- **THEN** 显示警告信息
- **AND** 询问用户是否继续发送
