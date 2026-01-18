## MODIFIED Requirements

### Requirement: 交互式对话
CLI SHALL 支持多轮对话式交互，提供丰富的 TUI 用户界面。

#### Scenario: 发送消息（TUI 模式）
- **WHEN** 用户在输入框输入文本并按回车
- **THEN** 在状态栏显示"思考中..."提示
- **AND** 将消息添加到消息历史
- **AND** 发送 API 请求
- **AND** 收到响应后在消息区显示（青色着色）
- **AND** 自动滚动到最新消息
- **AND** 输入框保持焦点等待下一次输入

#### Scenario: 发送消息（非 TUI 模式）
- **WHEN** 用户在简单终端模式输入文本并按回车
- **THEN** 显示加载状态提示（"思考中..."）
- **AND** 将消息发送到 AI API
- **AND** 显示 AI 响应（使用青色着色）
- **AND** 等待下一次用户输入
- **AND** 保持原有行为不变

#### Scenario: 响应格式化（TUI 模式）
- **WHEN** AI 响应包含 Markdown 格式
- **THEN** 在消息区渲染格式化内容
- **AND** 支持代码块语法高亮
- **AND** 支持标题、列表、粗体等基本 Markdown
- **AND** 保持原始代码缩进和格式
- **AND** 支持可滚动查看长消息

#### Scenario: 响应格式化（非 TUI 模式）
- **WHEN** AI 响应包含 Markdown 格式
- **THEN** 保留代码块格式
- **AND** 支持基本 Markdown 语法高亮（标题、列表、代码块）
- **AND** 保持段落间距
- **AND** 保持原有行为不变

#### Scenario: 错误处理（TUI 模式）
- **WHEN** API 请求失败
- **THEN** 在消息区显示清晰的错误信息（红色）
- **AND** 提供错误原因和可能的解决方案
- **AND** 在状态栏显示错误指示
- **AND** 保持程序继续运行，允许用户继续输入

#### Scenario: 错误处理（非 TUI 模式）
- **WHEN** API 请求失败
- **THEN** 显示清晰的错误信息（红色）
- **AND** 提供错误原因
- **AND** 保持对话会话，允许用户继续输入
- **AND** 保持原有行为不变

## ADDED Requirements

### Requirement: 模式切换
CLI SHALL 支持在 TUI 模式和非 TUI 模式之间切换。

#### Scenario: 启用 TUI 模式
- **WHEN** 用户执行 `oxide` 命令（不带 --no-tui 参数）
- **THEN** 启动 TUI 界面
- **AND** 提供丰富的交互体验

#### Scenario: 启用非 TUI 模式
- **WHEN** 用户执行 `oxide --no-tui` 命令
- **THEN** 使用简单终端交互
- **AND** 保持原有功能和体验
- **AND** 适用于不支持 TUI 的环境

### Requirement: 界面自适应
CLI SHALL 根据终端能力自动选择合适的交互模式。

#### Scenario: 检测终端能力
- **WHEN** 启动程序
- **THEN** 检测终端是否支持 TUI
- **AND** 如果不支持，自动回退到非 TUI 模式
- **AND** 显示提示信息说明使用的模式
