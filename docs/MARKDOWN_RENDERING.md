# Markdown 渲染功能

Oxide CLI 支持实时渲染 AI 回复中的 Markdown 格式，提供更好的阅读体验。

## 概述

当 AI 助手回复包含 Markdown 格式的内容时，Oxide 会自动将其渲染为彩色格式，包括：
- 彩色标题
- 高亮粗体文本
- 彩色斜体文本
- 高亮行内代码
- 带背景色的代码块
- 格式化的列表

## 支持的 Markdown 元素

### 标题（Headers）

```markdown
# 一级标题
## 二级标题
### 三级标题
```

**渲染效果**：青色显示，有明显的视觉层次

### 粗体（Bold）

```markdown
这是 **粗体文本**
```

**渲染效果**：白色高亮显示

### 斜体（Italic）

```markdown
这是 *斜体文本*
```

**渲染效果**：黄色显示

### 行内代码（Inline Code）

```markdown
这是 `行内代码`
```

**渲染效果**：绿色显示

### 代码块（Code Blocks）

````markdown
```rust
fn main() {
    println!("Hello, World!");
}
```
````

**渲染效果**：灰色背景，代码原色显示

### 列表（Lists）

```markdown
- 无序列表项 1
- 无序列表项 2
  - 嵌套项
```

**渲染效果**：带有列表符号，缩进正确

## 实现细节

### 渲染库

Oxide 使用 [termimad](https://github.com/Canop/termimad) 库进行 Markdown 渲染：
- 轻量级，专为 CLI 设计
- 支持 CommonMark 子集
- 与 `colored` 库完美配合
- 支持自定义颜色主题

### 流式渲染

为了保持响应速度，Oxide 实现了智能流式渲染：

1. **逐字符缓冲** - 累积字符直到遇到换行符
2. **逐行渲染** - 每行独立渲染 Markdown 格式
3. **代码块检测** - 自动检测代码块边界
4. **实时输出** - 边接收边渲染，无延迟

### 自定义样式

```rust
// src/cli/render.rs 中的配置
skin.set_headers_fg(Color::Cyan);        // 标题青色
skin.bold.set_fg(Color::White);           // 粗体白色
skin.italic.set_fg(Color::Yellow);        // 斜体黄色
skin.inline_code.set_fg(Color::Green);    // 行内代码绿色
skin.code_block.set_bg(Color::AnsiValue(233)); // 代码块灰色背景
```

## 不支持的格式

以下 Markdown 格式**不支持**渲染（会显示为原始文本）：

- ❌ **表格** - 由于终端宽度限制和中文字符宽度计算问题，表格以原始文本显示
- ❌ **删除线** - `~~删除线~~` 不支持
- ❌ **链接** - 链接文本会渲染，但不可点击
- ❌ **图片** - 图片语法会显示为原始文本
- ❌ **HTML** - HTML 标签不渲染

## 最佳实践

### 编写 AI 提示词

为了让 AI 输出更好的渲染效果，可以引导 AI 使用支持的格式：

```
请使用以下格式回复：
- 使用 ## 或 ### 标记重要章节
- 使用 **粗体** 强调关键信息
- 使用 `行内代码` 标记变量名和函数名
- 使用代码块展示代码示例
- 使用列表组织多个要点
```

### 终端配置

推荐使用以下终端以获得最佳渲染效果：

- **macOS**: iTerm2, Terminal.app
- **Linux**: GNOME Terminal, Alacritty, Kitty
- **Windows**: Windows Terminal, PowerShell 7+

**终端要求**：
- 支持 ANSI 颜色代码
- 支持 Unicode 字符（U+2500-U+259F）
- 建议宽度至少 80 列
- 建议使用等宽字体

### 颜色主题

如果你不喜欢默认的颜色方案，可以修改 `src/cli/render.rs` 中的颜色配置：

```rust
// 修改标题颜色为蓝色
skin.set_headers_fg(Color::Blue);

// 修改粗体颜色为亮绿色
skin.bold.set_fg(Color::BrightGreen);
```

然后重新编译：

```bash
cargo build --release
```

## 故障排除

### 问题 1：颜色显示不正常

**症状**：所有文本都是白色或颜色混乱

**解决方案**：
1. 检查终端是否支持 ANSI 颜色
2. 尝试不同的终端程序
3. 确保 TERM 环境变量正确（如 `xterm-256color`）

### 问题 2：代码块背景色显示异常

**症状**：代码块没有背景色或背景色覆盖文字

**解决方案**：
1. 更新终端到最新版本
2. 在终端设置中启用"使用背景色"选项
3. 尝试不同的配色方案

### 问题 3：表格格式错位

**症状**：表格的列对齐不正确

**说明**：这是正常现象，Oxide 不渲染表格为表格格式，而是显示原始 Markdown 文本

## 技术实现

如果你想了解或修改渲染逻辑，请查看：

- `src/cli/render.rs` - 主要的渲染实现
- `MarkdownStreamRenderer` - 流式渲染器结构
- `get_mad_skin()` - 颜色主题配置
- `stream_with_animation()` - 集成渲染的流式输出函数

## 未来改进

可能的未来改进方向：

1. ✨ 添加语法高亮（使用 `syntect`）
2. 📊 支持简单的表格渲染（ASCII 格式）
3. 🎨 支持自定义颜色主题文件
4. 🌓 支持亮色/暗色主题切换
5. 📱 改进移动端终端的显示效果

## 参考资源

- [termimad 文档](https://docs.rs/termimad/)
- [CommonMark 规范](https://commonmark.org/)
- [ANSI 颜色代码](https://en.wikipedia.org/wiki/ANSI_escape_code)
