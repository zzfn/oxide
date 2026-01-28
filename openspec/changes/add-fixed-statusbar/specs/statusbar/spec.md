# Capability: 固定底部状态栏

## ADDED Requirements

### Requirement: 状态栏初始化与显示

The statusbar SHALL be initialized when the CLI starts and SHALL be fixed at the bottom of the terminal, not scrolling with conversation content.

#### Scenario: CLI 启动时初始化状态栏

**Given** 用户启动 Oxide CLI
**When** CLI 完成初始化
**Then** 终端底部显示状态栏，包含初始状态信息（Token: 0, Session ID, Model, CWD）
**And** 对话输入区域位于状态栏上方的滚动区域

#### Scenario: 状态栏在对话过程中保持固定

**Given** 状态栏已初始化并显示
**When** 用户发送多条消息，对话历史超过一屏
**Then** 对话内容向上滚动
**And** 状态栏保持固定在终端底部，不随内容滚动

### Requirement: 实时状态更新

The statusbar SHALL display real-time session information including token usage, session ID, model name, and working directory.

#### Scenario: Token 使用量实时更新

**Given** 状态栏显示当前 Token 使用量为 100
**When** AI 响应流式输出，消耗了 50 个 token
**Then** 状态栏中的 Token 计数更新为 150
**And** 更新过程无闪烁或撕裂

#### Scenario: 会话信息正确显示

**Given** 当前会话 ID 为 "abc-123-def-456"
**And** 使用模型为 "claude-sonnet-4"
**And** 工作目录为 "/Users/user/project"
**When** 状态栏渲染
**Then** 状态栏显示 "Session: abc-123..." (截断显示)
**And** 显示 "Model: claude-sonnet-4"
**And** 显示 "CWD: /Users/user/project"

### Requirement: 终端尺寸自适应

The statusbar SHALL adapt to terminal window size changes and automatically adjust display position and content width.

#### Scenario: 终端窗口调整大小

**Given** 状态栏在 80x24 终端中正常显示
**When** 用户将终端调整为 120x30
**Then** 状态栏自动移动到新的底部位置（第 30 行）
**And** 滚动区域调整为 1-29 行
**And** 状态栏内容根据新宽度重新格式化

#### Scenario: 终端高度不足时禁用状态栏

**Given** 终端高度小于 5 行
**When** CLI 尝试初始化状态栏
**Then** 状态栏自动禁用
**And** CLI 以传统模式运行（无固定状态栏）

### Requirement: 优雅降级

The statusbar SHALL automatically disable in terminals that do not support ANSI escape sequences or in non-interactive environments, allowing the CLI to run normally.

#### Scenario: 非 TTY 环境中禁用状态栏

**Given** Oxide CLI 的 stdout 被重定向到文件或管道
**When** CLI 启动
**Then** 状态栏自动禁用
**And** 所有输出以纯文本形式写入，无 ANSI 转义序列

#### Scenario: 不支持 ANSI 的终端中禁用状态栏

**Given** 环境变量 TERM="dumb"
**When** CLI 启动
**Then** 状态栏自动禁用
**And** CLI 正常运行，无错误或警告

### Requirement: 退出时清理终端状态

The CLI SHALL properly restore terminal state on exit, clearing the statusbar and resetting the scroll region.

#### Scenario: 正常退出时清理状态栏

**Given** 状态栏正在显示
**When** 用户输入 /quit 退出 CLI
**Then** 状态栏内容被清除
**And** 终端滚动区域重置为全屏
**And** 光标移动到终端底部

#### Scenario: 异常退出时清理状态栏

**Given** 状态栏正在显示
**When** CLI 因错误或 Ctrl+C 中断退出
**Then** 状态栏清理逻辑仍然执行
**And** 终端状态正确恢复

### Requirement: 性能与用户体验

Statusbar updates SHALL be efficient, not affecting streaming output performance, and SHALL not cause visual flickering.

#### Scenario: 状态栏更新不阻塞流式输出

**Given** AI 正在流式输出长文本响应
**When** 状态栏每 100ms 更新一次 Token 计数
**Then** 文本输出流畅，无明显延迟或卡顿
**And** 状态栏更新不干扰用户阅读

#### Scenario: 状态栏更新无闪烁

**Given** 状态栏显示 Token 计数为 1000
**When** Token 计数更新为 1050
**Then** 状态栏内容平滑更新
**And** 无整行闪烁或光标跳动现象
