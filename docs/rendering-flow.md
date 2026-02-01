# AI 交互渲染流程

## 概述

AI 交互渲染分为三个层次：
1. **StatusLine（底部状态栏）** - 显示 AI 思考进度和 token 计数
2. **工具执行进度条** - 显示当前执行的工具
3. **流式文本输出** - AI 的文本响应和思考内容

## 状态显示时机

### StatusLine（底部状态栏）

**显示时机：**
- AI 开始响应时调用 `statusline.start()`
- 持续显示 spinner、经过时间、token 计数

**暂停时机：**
- 工具调用开始时调用 `statusline.suspend()`
- 人类确认对话框显示时（通过 `mp.suspend()`）

**恢复时机：**
- 工具执行完成时调用 `statusline.resume()`
- 人类确认完成后自动恢复

**完成时机：**
- AI 响应完全结束时调用 `statusline.finish()`

### 工具执行进度条

**创建时机：**
- 收到 `ToolCall` 事件时
- 显示格式：`⏺ ToolName(description)`
- 使用 spinner 动画表示执行中

**清除时机：**
- 收到 `ToolResult` 事件时
- 清除 spinner，输出永久性完成信息：`⏺ ToolName(description)` （绿色）

### 流式文本输出

**文本模式：**
- 按行缓冲输出
- 遇到换行符时立即输出当前行

**思考模式：**
- 收到 `Reasoning` 或 `ReasoningDelta` 时进入
- 显示 `💭 思考中:` 标题
- 缓冲思考内容，在模式切换时输出

**模式切换：**
- 从思考模式切换到文本模式时：刷新缓冲 + 输出空行
- 从任意模式切换到工具调用时：刷新缓冲

## 人类确认流程

### 触发时机

工具需要权限确认时（由 `PermissionManager` 判断）：
1. 工具不在白名单中
2. 工具不在会话临时白名单中

### 确认流程

```
1. 工具调用开始
   ↓
2. PermissionManager 检查权限
   ↓
3. 需要确认 → 调用 confirmation_callback
   ↓
4. mp.suspend() 暂停所有进度条
   ↓
5. 显示 dialoguer 确认对话框
   - 允许本次
   - 始终允许（本次会话）
   - 始终允许（记住选择）
   - 拒绝
   ↓
6. 用户选择
   ↓
7. 恢复进度条（mp.suspend 自动恢复）
   ↓
8. 返回确认结果
   ↓
9. 工具执行或拒绝
```

### 确认选项说明

- **允许本次**：仅本次工具调用允许
- **始终允许（本次会话）**：添加到会话临时白名单
- **始终允许（记住选择）**：添加到配置文件白名单，持久化
- **拒绝**：拒绝工具执行

## 代码结构

### StreamState（流式渲染状态管理器）

位置：`crates/oxide-cli/src/render/stream_state.rs`

**职责：**
- 管理流式输出的状态转换
- 协调 statusline 和工具进度条的显示
- 处理文本缓冲和输出

**关键方法：**
- `handle_text()` - 处理流式文本
- `handle_reasoning()` - 处理思考内容
- `start_tool()` - 开始工具调用（暂停 statusline，创建进度条）
- `finish_tool()` - 完成工具调用（清除进度条，恢复 statusline）
- `finish()` - 完成流式输出（刷新缓冲）

### RigAgentRunner::run_stream()

位置：`crates/oxide-cli/src/agent.rs:212-289`

**职责：**
- 处理流式响应事件
- 委托给 `StreamState` 处理渲染

**事件处理：**
- `Text` → `state.handle_text()`
- `Reasoning` / `ReasoningDelta` → `state.handle_reasoning()`
- `ToolCall` → `state.start_tool()`
- `ToolResult` → `state.finish_tool()`

### create_confirmation_callback()

位置：`crates/oxide-cli/src/agent.rs:18-59`

**职责：**
- 创建权限确认回调
- 使用 `mp.suspend()` 暂停进度条
- 显示确认对话框
- 返回确认结果

## 改进点

### 已完成
- ✅ 提取 `StreamState` 管理器，简化状态管理
- ✅ 从 200+ 行方法重构为 80 行 + 独立状态管理器
- ✅ 清晰的状态转换逻辑

### 待改进
- 考虑将 `extract_tool_description()` 移到 `StreamState` 内部
- 考虑为不同工具类型提供更丰富的描述格式
- 考虑添加工具执行时间统计
