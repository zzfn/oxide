# cli-core Spec Delta

## ADDED Requirements

### Requirement: 流式 Markdown 渲染
TUI SHALL 支持流式实时渲染 AI 的 Markdown 响应。

#### Scenario: 实时打字效果
- **WHEN** AI 开始流式输出响应
- **THEN** 逐字符显示响应内容（打字机效果）
- **AND** 实时解析和渲染 Markdown 格式
- **AND** 保持流畅的视觉体验

#### Scenario: 增量 Markdown 解析
- **WHEN** 接收到新的响应片段
- **THEN** 增量更新已解析的 Markdown AST
- **AND** 重新渲染受影响的区域
- **AND** 避免闪烁和跳动

## MODIFIED Requirements

### Requirement: 交互式对话（更新）
CLI SHALL 支持多轮对话式交互，提供流式 Markdown 渲染和流畅的视觉反馈。

#### Scenario: 发送消息（更新）
- **WHEN** 用户输入文本并按回车
- **THEN** 显示加载状态提示
- **AND** 将消息发送到 AI API
- **AND** 开始流式显示 AI 响应（打字机效果）
- **AND** 实时渲染 Markdown 格式
- **AND** 等待下一次用户输入

#### Scenario: 响应格式化（更新）
- **WHEN** AI 响应包含 Markdown 格式
- **THEN** 流式渲染 Markdown 内容
- **AND** 支持代码块语法高亮（带语言标识）
- **AND** 支持标题、列表、链接、粗体等格式
- **AND** 保持段落间距和格式
- **AND** 渲染过程流畅无卡顿

## REMOVED Requirements

无
