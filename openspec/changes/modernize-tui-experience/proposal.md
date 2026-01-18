# Change: 现代化 TUI 体验

## Why
当前 Oxide 的 TUI 实现使用传统的终端界面设计，包含大量边框、分割线和装饰元素，与 Claude Code 等现代化工具的简洁设计风格相比显得过时。用户体验存在以下问题：

1. **界面设计过于传统**：使用大量边框和分割线，视觉上显得杂乱
2. **Markdown 渲染不流畅**：AI 响应需要完整输出后才渲染，缺乏流式输出体验
3. **工具状态展示不直观**：工具执行状态更新不够清晰，缺乏实时视觉反馈
4. **交互体验卡顿**：界面更新存在轻微闪烁，滚动不够流畅

用户期望更现代、更流畅的 TUI 体验，类似 Claude Code 的设计风格。

## What Changes
- **MODIFIED**: 重构 TUI 渲染逻辑，实现流式 Markdown 渲染
- **MODIFIED**: 简化界面设计，减少不必要的边框和装饰元素
- **MODIFIED**: 改进工具状态展示，提供实时、清晰的视觉反馈
- **MODIFIED**: 优化交互体验，消除闪烁，实现流畅滚动
- **ADDED**: 新增消息卡片设计（使用 `╭─╮│╰─╯` 轻量边框）
- **ADDED**: 新增主题系统（支持 dark/light/high_contrast 和自定义主题）
- **ADDED**: 新增布局模式切换（Standard/Compact/Minimal/Split）
- **ADDED**: 新增交互功能（帮助系统、命令历史、多行输入、消息搜索）
- **ADDED**: 新增实时打字效果（typewriter effect）用于 AI 响应
- **ADDED**: 新增进度指示器和加载动画
- **ADDED**: 新增工具状态内嵌到消息流（替代独立面板）

## Impact
- Affected specs: `cli-core` (交互式对话需求), `tui` (TUI 体验需求)
- Affected code: `src/tui/ui.rs` (渲染逻辑), `src/tui/app.rs` (状态管理), `src/tui/theme.rs` (新增主题系统), `src/main.rs` (主循环)
- 新增依赖: `toml` (主题配置), 可能需要 `pulldown-cmark` (增强 Markdown 渲染)
- **BREAKING**: 移除非 TUI 模式支持，TUI 成为唯一 UI
- 移除 `--no-tui` 参数及相关逻辑
- 新增配置文件: `~/.config/oxide/theme.toml` (用户自定义主题)
