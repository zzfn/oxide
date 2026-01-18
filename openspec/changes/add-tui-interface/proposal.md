# Change: 添加 TUI（终端用户界面）支持

## Why
当前 Oxide CLI 使用简单的终端输入输出流，用户体验受限。TUI 可以提供更丰富的交互体验，包括代码块语法高亮、进度显示、历史消息滚动、实时状态更新等功能，类似于 Claude Code 的用户体验。

## What Changes
- **BREAKING**: 重构主要的交互循环架构，从简单的输入输出流升级到 TUI 事件驱动模式
- 添加 TUI 框架支持（ratatui + crossterm）
- 实现多面板布局（消息历史、输入区域、工具状态、帮助信息）
- 添加代码块语法高亮显示
- 实现可滚动的历史消息列表
- 添加实时状态指示器（API 请求、工具执行、思考状态）
- 保持与现有 CLI 功能的兼容性（支持无 TUI 模式）

## Impact
- Affected specs: `cli-core` (交互式对话需求), `tui` (新增)
- Affected code: `src/main.rs` (主交互循环), 新增 `src/tui/` 模块
- 新增依赖: `ratatui`, `crossterm`, `syntect` (代码高亮)
- 向后兼容: 保留 `--no-tui` 参数以支持简单终端模式
