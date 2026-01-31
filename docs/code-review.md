# Code Review 记录

## 2026-01-31: 状态行显示功能 (feat/rebuild-from-scratch)

### 变更概述

使用 `indicatif` 库添加状态行显示功能，提供实时任务状态更新。

### 涉及文件

- `crates/oxide-cli/Cargo.toml` - 添加 indicatif 依赖
- `crates/oxide-cli/src/render/statusline.rs` - 新增状态行模块
- `crates/oxide-cli/src/render/mod.rs` - 集成 MultiProgress
- `crates/oxide-cli/src/render/tool_status.rs` - 重构为使用 indicatif
- `crates/oxide-cli/src/agent.rs` - 添加 MultiProgress 支持
- `crates/oxide-cli/src/repl/mod.rs` - 集成状态行显示
- `crates/oxide-cli/examples/dynamic_output.rs` - 已删除

### 发现的问题

#### 1. `tool_status.rs:update()` 方法丢失图标和颜色

**位置**: `crates/oxide-cli/src/render/tool_status.rs:117-124`

**问题**: 重构后 `update()` 方法只输出纯文本，丢失了原来的图标（🔧、⚙、✓、✗）和颜色。

**原代码**:
```rust
let (icon, text, color_fn) = match status {
    ToolStatus::Calling => ("🔧", format!("调用工具: {}", tool_name), |s| s.bright_yellow()),
    // ...
};
print!("{} {}", icon, color_fn(&text));
```

**现代码**:
```rust
let text = match status {
    ToolStatus::Calling => format!("调用工具: {}", tool_name),
    // ... 没有图标和颜色
};
self.mp.println(text)?;
```

**建议修复**:
```rust
pub fn update(&mut self, tool_name: &str, status: ToolStatus) -> io::Result<()> {
    let text = match status {
        ToolStatus::Calling => format!("{} 调用工具: {}", "🔧", tool_name.bright_yellow()),
        ToolStatus::Executing(ref desc) => format!("{} 执行工具: {} - {}", "⚙", tool_name.bright_cyan(), desc),
        ToolStatus::Success => format!("{} 工具 {} 执行成功", "✓".green(), tool_name.bright_cyan()),
        ToolStatus::Error(ref err) => format!("{} 工具 {} 执行失败: {}", "✗".red(), tool_name.bright_cyan(), err),
    };
    self.mp.println(text)?;
    Ok(())
}
```

#### 2. 未使用的方法 `start_tool_before()`

**位置**: `crates/oxide-cli/src/render/tool_status.rs:68-91`

**问题**: 新增的 `start_tool_before()` 方法没有被任何代码调用，属于多余代码。

**建议**: 删除该方法，或在需要时再添加。

#### 3. `statusline.rs` 中未使用的公开方法

**位置**:
- `crates/oxide-cli/src/render/statusline.rs:37-39` (`bar()`)
- `crates/oxide-cli/src/render/statusline.rs:127-129` (`start_time()`)

**问题**: 这两个方法没有被调用，增加了不必要的 API 表面。

**建议**: 删除未使用的方法，保持 API 精简。

#### 4. `assistant_header()` 行为改变

**位置**: `crates/oxide-cli/src/render/mod.rs:108-112`

**问题**: 原来使用 `print!` 不换行（流式输出会接在后面），现在改为 `mp.println` 会换行，改变了输出格式。

**原代码**:
```rust
print!("{} ", "Assistant".bright_blue().bold());
```

**现代码**:
```rust
let _ = self.mp.println(format!("{} ", "Assistant".bright_blue().bold()));
```

**影响**: 可能影响流式输出的显示效果。

#### 5. 移除了 `Default` trait 实现

**位置**: `crates/oxide-cli/src/render/tool_status.rs`

**问题**: `ToolStatusDisplay` 移除了 `Default` trait 实现，如果其他代码依赖 `ToolStatusDisplay::default()` 会导致编译错误。

### 非阻塞观察

1. 大量使用 `let _ = self.mp.println(...)` 忽略错误，可接受但不够优雅
2. `agent.rs` 中的行缓冲逻辑和最后的刷新代码有重复

### 建议优先级

| 优先级 | 问题 | 影响 |
|--------|------|------|
| 高 | `update()` 丢失图标和颜色 | 用户体验下降 |
| 中 | 未使用的 `start_tool_before()` | 代码冗余 |
| 低 | 未使用的 `bar()` 和 `start_time()` | API 冗余 |
| 低 | `assistant_header()` 行为改变 | 可能影响显示 |
