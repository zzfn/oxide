# tui Spec Delta

## ADDED Requirements

### Requirement: 简洁的界面设计
TUI SHALL 采用简洁现代的界面设计，避免过度装饰。

#### Scenario: 无边框布局
- **WHEN** TUI 初始化完成
- **THEN** 不使用外边框（Borders::ALL）
- **AND** 使用简洁的分隔线（`═════════════════`）
- **AND** 保持足够的留白和间距
- **AND** 专注于内容展示而非装饰

#### Scenario: 简化的状态栏
- **WHEN** 显示状态栏信息
- **THEN** 使用单行文本显示关键信息（版本、模型、会话 ID）
- **AND** 使用简短的状态指示器（✓ 就绪、⟳ 处理中、✗ 错误）
- **AND** 避免冗余的装饰元素

### Requirement: 流式交互体验
TUI SHALL 提供流畅的流式交互体验。

#### Scenario: 打字机效果
- **WHEN** AI 流式输出响应
- **THEN** 实现打字机效果逐字符显示
- **AND** 保持稳定的渲染速度（如 50ms/字符）
- **AND** 可配置打字速度或禁用效果

#### Scenario: 平滑滚动
- **WHEN** 用户滚动消息历史
- **THEN** 实现平滑滚动效果
- **AND** 支持逐行滚动和页面滚动
- **AND** 避免闪烁和跳跃

### Requirement: 增量渲染
TUI SHALL 支持增量渲染以优化性能和用户体验。

#### Scenario: 部分区域更新
- **WHEN** 只有部分内容发生变化
- **THEN** 只重新渲染变化的部分区域
- **AND** 避免不必要的全屏重绘
- **AND** 减少渲染开销

#### Scenario: 双缓冲渲染
- **WHEN** 渲染 TUI 界面
- **THEN** 使用双缓冲技术避免闪烁
- **AND** 在后台缓冲区构建完整帧
- **AND** 一次性更新到终端

### Requirement: 消息卡片设计
TUI SHALL 使用消息卡片设计，每个消息独立显示，使用轻量边框区分。

#### Scenario: 消息卡片结构
- **WHEN** 显示消息
- **THEN** 使用 `╭─╮│╰─╯` 字符创建轻量边框
- **AND** 边框颜色匹配消息类型（Cyan/Blue/Yellow/Gray）
- **AND** 保持卡片间距一致
- **AND** 支持无边框模式（Compact/Minimal 布局）

#### Scenario: 工具状态内嵌
- **WHEN** AI 调用工具
- **THEN** 工具状态显示在 Assistant 消息卡片底部
- **AND** 使用状态图标（• ⟳ ✓ ✗）+ 工具名
- **AND** 显示执行时间（如 `✓ Read 完成 (42ms)`）
- **AND** 支持独立面板模式（Split 布局）

### Requirement: 主题系统
TUI SHALL 支持可配置的主题系统，允许用户自定义颜色和样式。

#### Scenario: 内置主题
- **WHEN** 用户启动 TUI
- **THEN** 使用默认主题（dark 深色主题）
- **AND** 支持切换到内置主题（light/high_contrast）
- **AND** 主题包含完整颜色定义（背景、前景、语义化颜色）

#### Scenario: 自定义主题
- **WHEN** 用户创建 `~/.config/oxide/theme.toml`
- **THEN** 从配置文件加载自定义颜色
- **AND** 支持 RGB hex 格式（`#61afef`）和命名颜色
- **AND** 配置文件不存在时使用默认主题

#### Scenario: 主题热切换
- **WHEN** 用户执行 `/theme set light`
- **THEN** 立即应用新主题（无需重启）
- **AND** 所有已渲染内容使用新颜色
- **AND** 主题选择持久化到配置文件

#### Scenario: 高对比度模式
- **WHEN** 用户启用高对比度主题
- **THEN** 使用 Black/White 极致对比配色
- **AND** 所有颜色符合 WCAG 2.0 标准
- **AND** 保持良好的可读性

### Requirement: 布局模式切换
TUI SHALL 支持多种布局模式，适应不同终端尺寸和用户偏好。

#### Scenario: 标准布局
- **WHEN** 使用 Standard 布局模式
- **THEN** 消息使用完整卡片边框
- **AND** 消息间距宽（2-3 行）
- **AND** 显示帮助栏和状态栏
- **AND** 工具面板按需显示（Ctrl+T）

#### Scenario: 紧凑布局
- **WHEN** 使用 Compact 布局模式
- **THEN** 消息无边框，仅颜色区分
- **AND** 消息间距中等（1 行）
- **AND** 工具状态内嵌到消息中
- **AND** 保持帮助栏

#### Scenario: 极简布局
- **WHEN** 使用 Minimal 布局模式
- **THEN** 消息无边框，最小间距
- **AND** 不显示帮助栏
- **AND** 工具状态极简显示
- **AND** 最大化内容区域

#### Scenario: 分屏布局
- **WHEN** 使用 Split 布局模式
- **THEN** 右侧常驻工具面板（32 宽度）
- **AND** 消息使用卡片边框
- **AND** 支持调整面板宽度（`+`/`-` 键）
- **AND** 适合调试场景

#### Scenario: 模式切换
- **WHEN** 用户按 `M` 键
- **THEN** 循环切换布局模式（Standard → Compact → Minimal → Split → Standard）
- **AND** 保持当前消息历史
- **AND** 平滑过渡到新模式

### Requirement: 帮助系统
TUI SHALL 提供完整的快捷键帮助系统。

#### Scenario: 帮助屏幕
- **WHEN** 用户按 `?` 键
- **THEN** 显示全屏帮助屏幕
- **AND** 列出所有快捷键和功能
- **AND** 按类别组织（全局/输入/导航）
- **AND** 按 `q` 或 `Esc` 关闭

#### Scenario: 底部快捷键提示
- **WHEN** TUI 正常运行（非 Minimal 模式）
- **THEN** 底部显示最常用的快捷键
- **AND** 使用 `[?]帮助 [Ctrl+T]工具` 格式
- **AND** 使用低对比度颜色避免干扰

### Requirement: 命令历史
TUI SHALL 支持输入历史记录和搜索。

#### Scenario: 浏览历史
- **WHEN** 用户按 `↑` 键
- **THEN** 输入框显示上一条命令
- **AND** 连续按 `↑` 继续向上浏览
- **AND** 按 `↓` 键反向浏览

#### Scenario: 历史搜索
- **WHEN** 用户按 `Ctrl+R`
- **THEN** 进入历史搜索模式
- **AND** 输入关键词过滤历史
- **AND** 显示匹配的命令列表
- **AND** 按 `Enter` 选择命令

#### Scenario: 历史持久化
- **WHEN** 用户发送消息
- **THEN** 命令保存到 `~/.config/oxide/history.toml`
- **AND** 启动时自动加载历史
- **AND** 限制历史大小（默认 1000 条）

### Requirement: 多行输入
TUI SHALL 支持多行输入模式，用于复杂查询。

#### Scenario: 进入多行模式
- **WHEN** 用户按 `Ctrl+G`
- **THEN** 进入多行输入模式
- **AND** 显示全屏输入卡片
- **AND** 支持 Markdown 编辑

#### Scenario: 多行发送
- **WHEN** 在多行输入模式下按 `Ctrl+G`
- **THEN** 发送多行内容
- **AND** 返回正常输入模式
- **AND** 按 `Esc` 取消输入

### Requirement: 改进的工具状态展示
TUI SHALL 提供清晰直观的工具执行状态展示。

#### Scenario: 实时进度指示
- **WHEN** 工具正在执行
- **THEN** 显示实时进度条或加载动画
- **AND** 显示执行时间预估
- **AND** 使用动画效果增强视觉反馈

#### Scenario: 工具状态详情
- **WHEN** 查看工具执行状态
- **THEN** 显示工具名称和状态（⚡ 执行中、✓ 完成、✗ 失败）
- **AND** 显示工具执行的详细日志
- **AND** 使用颜色区分不同状态

#### Scenario: 多工具执行跟踪
- **WHEN** 多个工具同时执行
- **THEN** 显示每个工具的独立状态
- **AND** 显示整体进度（如 2/5 完成）
- **AND** 支持展开/折叠详细信息

### Requirement: 消息搜索
TUI SHALL 支持在消息历史中搜索内容。

#### Scenario: 搜索消息
- **WHEN** 用户按 `/` 键
- **THEN** 进入消息搜索模式
- **AND** 输入关键词实时过滤消息
- **AND** 高亮匹配的关键词

#### Scenario: 跳转搜索结果
- **WHEN** 在搜索模式下按 `n` 键
- **THEN** 跳转到下一个搜索结果
- **AND** 按 `N` 键跳转到上一个结果
- **AND** 按 `Esc` 退出搜索模式

### Requirement: 性能优化
TUI SHALL 通过多种技术优化性能。

#### Scenario: 虚拟滚动
- **WHEN** 消息历史超过一定长度（如 1000 行）
- **THEN** 实现虚拟滚动，只渲染可见区域
- **AND** 保持滚动位置平滑
- **AND** 避免内存溢出

#### Scenario: 渲染缓存
- **WHEN** 渲染相同或相似内容
- **THEN** 使用缓存机制避免重复计算
- **AND** 缓存解析后的 Markdown AST
- **AND** 缓存渲染后的行数据

#### Scenario: 异步渲染
- **WHEN** 进行复杂的渲染操作
- **THEN** 在后台线程异步执行
- **AND** 不阻塞用户输入和交互
- **AND** 使用消息传递更新 UI

## MODIFIED Requirements

### Requirement: TUI 界面初始化（更新）
系统 SHALL 在启动时初始化 TUI 终端界面，采用简洁现代的设计风格。

#### Scenario: TUI 模式启动（更新）
- **WHEN** 用户执行 `oxide` 命令
- **THEN** 初始化 TUI 终端界面
- **AND** 显示简洁的欢迎界面（无边框设计）
- **AND** 显示主布局（状态栏、消息区、输入区）- 采用简洁设计
- **AND** 进入事件循环等待用户输入
- **AND** 不支持非 TUI 模式或 `--no-tui` 参数

### Requirement: 多面板布局（更新）
系统 SHALL 提供简洁的分面板 TUI 布局，避免过度装饰。

#### Scenario: 主布局显示（更新）
- **WHEN** TUI 初始化完成
- **THEN** 顶部显示简洁的状态栏（当前模型、消息计数）- 无边框
- **AND** 中间区域显示消息历史（可滚动）- 无边框
- **AND** 底部显示输入框和提示符 - 无边框
- **AND** 右侧（可选）显示改进的工具调用状态

### Requirement: 消息渲染（更新）
系统 SHALL 以流畅的流式方式渲染各类消息，支持打字机效果和增量更新。

#### Scenario: AI 文本响应渲染（更新）
- **WHEN** 显示 AI 的文本响应
- **THEN** 使用青色显示 AI 消息
- **AND** 流式渲染 Markdown 基本格式（标题、列表、代码块）
- **AND** 实现打字机效果逐字符显示
- **AND** 保持段落间距和格式
- **AND** 渲染过程流畅无卡顿

#### Scenario: 工具调用显示（更新）
- **WHEN** AI 调用工具
- **THEN** 使用黄色显示工具名称
- **AND** 显示工具参数摘要
- **AND** 显示改进的执行状态（带动画的执行中、✓ 完成、✗ 失败）
- **AND** 提供实时进度反馈
- **WHEN** AI 调用工具
- **THEN** 使用黄色显示工具名称
- **AND** 显示工具参数摘要
- **AND** 显示改进的执行状态（带动画的执行中、✓ 完成、✗ 失败）
- **AND** 提供实时进度反馈

### Requirement: 实时状态指示（更新）
系统 SHALL 显示简洁清晰的实时操作状态，提供流畅的视觉反馈。

#### Scenario: API 请求中状态（更新）
- **WHEN** 发送 API 请求
- **THEN** 在状态栏显示简洁的"⟳ 处理中..."提示
- **AND** 使用平滑的加载动画
- **AND** 请求完成后移除提示

#### Scenario: 工具执行状态（更新）
- **WHEN** 执行工具调用
- **THEN** 显示工具执行进度（带进度条或动画）
- **AND** 区分执行中（⚡ 动画）、成功（✓ 绿色）、失败（✗ 红色）状态
- **AND** 使用不同颜色和动画标识状态
- **AND** 提供详细的执行日志
- **WHEN** 执行工具调用
- **THEN** 显示工具执行进度（带进度条或动画）
- **AND** 区分执行中（⚡ 动画）、成功（✓ 绿色）、失败（✗ 红色）状态
- **AND** 使用不同颜色和动画标识状态
- **AND** 提供详细的执行日志

## REMOVED Requirements

### Requirement: 非 TUI 模式支持（已移除）
系统不再支持非 TUI 模式，TUI 是唯一的用户界面。

#### Scenario: 非 TUI 模式启动（已移除）
- **REMOVED**: 不再支持 `oxide --no-tui` 命令
- **REMOVED**: 移除简单终端模式的所有相关代码
- **REMOVED**: 移除非 TUI 模式的交互逻辑
