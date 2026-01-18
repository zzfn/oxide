# tui Specification

## ADDED Requirements

### Requirement: TUI 界面初始化
系统 SHALL 在启动时初始化 TUI 终端界面。

#### Scenario: TUI 模式启动
- **WHEN** 用户执行 `oxide` 命令（不带 --no-tui 参数）
- **THEN** 初始化 TUI 终端界面
- **AND** 显示欢迎界面（包含版本、模型信息）
- **AND** 显示主布局（状态栏、消息区、输入区）
- **AND** 进入事件循环等待用户输入

#### Scenario: 非 TUI 模式启动
- **WHEN** 用户执行 `oxide --no-tui` 命令
- **THEN** 使用简单终端模式
- **AND** 保持原有交互逻辑不变

### Requirement: 多面板布局
系统 SHALL 提供分面板的 TUI 布局。

#### Scenario: 主布局显示
- **WHEN** TUI 初始化完成
- **THEN** 顶部显示状态栏（当前模型、消息计数）
- **AND** 中间区域显示消息历史（可滚动）
- **AND** 底部显示输入框和提示符
- **AND** 右侧显示工具调用状态（可选）

### Requirement: 消息渲染
系统 SHALL 以丰富格式渲染各类消息。

#### Scenario: 用户消息渲染
- **WHEN** 显示用户输入的消息
- **THEN** 使用绿色显示用户消息
- **AND** 保持原始文本格式
- **AND** 显示消息序号

#### Scenario: AI 文本响应渲染
- **WHEN** 显示 AI 的文本响应
- **THEN** 使用青色显示 AI 消息
- **AND** 支持 Markdown 基本格式（标题、列表、代码块）
- **AND** 保持段落间距

#### Scenario: 代码块语法高亮
- **WHEN** AI 响应包含代码块
- **THEN** 识别代码语言标识符
- **AND** 应用相应的语法高亮方案
- **AND** 保留代码缩进和格式

#### Scenario: 工具调用显示
- **WHEN** AI 调用工具
- **THEN** 使用黄色显示工具名称
- **AND** 显示工具参数摘要
- **AND** 显示执行状态（执行中、完成、失败）

### Requirement: 可滚动历史
系统 SHALL 支持滚动查看历史消息。

#### Scenario: 消息滚动
- **WHEN** 用户使用上下箭头或 PageUp/PageDown
- **THEN** 消息列表相应滚动
- **AND** 高亮显示当前可见区域
- **AND** 保持输入焦点在底部

#### Scenario: 自动滚动到底部
- **WHEN** 收到新的 AI 响应
- **THEN** 自动滚动到最新消息
- **AND** 高亮显示新消息区域

### Requirement: 实时状态指示
系统 SHALL 显示实时操作状态。

#### Scenario: API 请求中状态
- **WHEN** 发送 API 请求
- **THEN** 在状态栏显示"请求中..."提示
- **AND** 使用动态加载动画
- **AND** 请求完成后移除提示

#### Scenario: 工具执行状态
- **WHEN** 执行工具调用
- **THEN** 显示工具执行进度
- **AND** 区分执行中、成功、失败状态
- **AND** 使用不同颜色标识状态

### Requirement: 键盘交互
系统 SHALL 支持丰富的键盘交互。

#### Scenario: 基本输入
- **WHEN** 用户在输入框输入文本
- **THEN** 字符正常显示
- **AND** 支持退格、删除、光标移动
- **AND** 回车发送消息

#### Scenario: 快捷键支持
- **WHEN** 用户按 Ctrl+C
- **THEN** 优雅退出程序
- **WHEN** 用户按 / 键
- **THEN** 提示可用的斜杠命令
- **WHEN** 用户按上下箭头（在命令模式）
- **THEN** 浏览命令历史

#### Scenario: 多行输入支持
- **WHEN** 用户输入超过一行
- **THEN** 自动换行显示
- **AND** 支持手动换行（Shift+Enter）
- **AND** 回车发送消息

### Requirement: 错误处理
系统 SHALL 在 TUI 界面中优雅处理错误。

#### Scenario: API 错误显示
- **WHEN** API 请求失败
- **THEN** 在消息区显示红色错误信息
- **AND** 提供错误详情
- **AND** 保持程序继续运行

#### Scenario: 网络连接问题
- **WHEN** 检测到网络连接问题
- **THEN** 在状态栏显示警告图标
- **AND** 提示用户检查网络
- **AND** 支持重试机制

### Requirement: 终端大小自适应
系统 SHALL 响应终端大小变化。

#### Scenario: 终端调整大小
- **WHEN** 用户调整终端窗口大小
- **THEN** 自动重新计算布局
- **AND** 重绘所有面板
- **AND** 保持消息显示完整性
