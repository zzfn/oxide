# Design: 现代化 TUI 体验

> 基于 Claude Code、Warp、Lazygit 等工具的最佳实践
> 设计版本: v2.0 | 更新日期: 2025-01-19

---

## 一、设计原则

### 核心哲学
1. **极简而不简陋**：界面干净，功能完整
2. **留白胜于边框**：使用间距和颜色区分，而非线条
3. **状态内嵌化**：工具状态融入消息流，减少独立面板
4. **渐进式信息披露**：需要时才显示详细信息

### 设计目标
- 减少视觉噪音（移除显式边框和分隔线）
- 提升信息层次（通过大小、颜色、位置）
- 改善呼吸感（合理留白）
- 增强可访问性（高对比度选项）

---

## 二、布局设计

### 2.1 推荐方案：极简无边框布局

```
┌─────────────────────────────────────────────────────────┐
│  Oxide  v0.1.0  claude-sonnet-4-5  ● 就绪               │  ← 单行状态栏（无边框）
│                                                         │
│  ╭─ User ───────────────────────────────────────────╮  │
│  │ 如何实现流式渲染？                                │  │
│  ╰──────────────────────────────────────────────────╯  │
│                                                         │
│  ╭─ Assistant ─────────────────────────────────────╮   │
│  │ 要实现流式渲染，你需要：                          │  │
│  │                                                   │  │
│  │ 1. 使用异步流式 API                              │  │
│  │ 2. 增量解析 Markdown                             │  │
│  │ 3. 实时更新 UI                                   │  │
│  │                                                   │  │
│  │ ```rust                                          │  │
│  │ async fn stream_response() {                     │  │
│  │     let mut stream = api.stream().await;        │  │
│  │     while let Some(chunk) = stream.next().await {│  │
│  │         update_ui(chunk);                       │  │
│  │     }                                            │  │
│  │ }                                                │  │
│  │ ```                                              │  │
│  │                                                   │  │
│  │ ⟳ Bash 读取文件...                               │  │  ← 内嵌工具状态
│  │ ✓ Read 完成 (42ms)                               │  │
│  │                                                   │  │
│  ╰──────────────────────────────────────────────────╯  │
│                                                         │
│  ╭─ Tool ──────────────────────────────────────────╮   │
│  │ Read · 完成读取 src/main.rs                     │  │
│  ╰──────────────────────────────────────────────────╯  │
│                                                         │
│                                                         │
│  > 如何优化渲染性能？                                   │  ← 极简输入提示符
│                                                         │
│  [?]帮助  [Ctrl+T]工具  [↑↓]历史  [Ctrl+C]退出           │  ← 底部快捷键提示
└─────────────────────────────────────────────────────────┘
```

### 2.2 四种布局模式

| 模式 | 消息边框 | 消息间距 | 工具面板 | 帮助栏 | 适用场景 |
|------|----------|----------|----------|--------|----------|
| **Standard** | ✓ | 宽 | 按需 | ✓ | 日常使用 |
| **Compact** | ✗ | 中 | 内嵌 | ✓ | 小终端 |
| **Minimal** | ✗ | 窄 | 内嵌 | ✗ | 演示 |
| **Split** | ✓ | 宽 | 常驻 | ✓ | 调试 |

### 2.3 布局模式切换

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LayoutMode {
    /// 标准模式：完整边框消息卡片
    Standard,
    /// 紧凑模式：简化消息格式
    Compact,
    /// 极简模式：最精简显示
    Minimal,
    /// 分屏模式：工具面板常驻
    Split,
}

// 快捷键：`M` 循环切换布局模式
```

---

## 三、消息卡片设计

### 3.1 卡片结构

```rust
pub struct MessageCard {
    pub msg_type: MessageType,
    pub title: String,        // "User", "Assistant", "Tool"
    pub content: String,
    pub tool_states: Vec<ToolState>,  // 内嵌的工具状态
}
```

### 3.2 卡片样式规范

| 消息类型 | 前缀格式 | 边框字符 | 颜色 | 用途 |
|----------|----------|----------|------|------|
| User | `╭─ User ─` | `─│╭╮╰╯` | Cyan | 用户输入 |
| Assistant | `╭─ Assistant ─` | `─│╭╮╰╯` | Blue | AI 响应 |
| Tool | `╭─ Tool ─` | `─│╭╮╰╯` | Yellow | 工具调用 |
| System | `╭─ System ─` | `─│╭╮╰╯` | Gray | 系统消息 |

### 3.3 工具状态内嵌设计

```rust
pub struct ToolState {
    pub name: String,
    pub state: ToolExecutionState,
    pub duration: Option<Duration>,
    pub error: Option<String>,
}

pub enum ToolExecutionState {
    Pending,    // • 等待中
    Running,    // ⟳ 执行中...
    Success,    // ✓ 成功 (42ms)
    Failed,     // ✗ 失败: 错误信息
}
```

**视觉效果示例：**
```
╭─ Assistant ─────────────────────────────╮
│ 分析代码结构...                          │
│                                          │
│ ⟳ Bash 读取文件...                       │
│ ✓ Read 完成 (42ms)                       │
│ • Grep 等待中                            │
╰──────────────────────────────────────────╯
```

---

## 四、主题系统

### 4.1 主题结构

```rust
pub struct Theme {
    // 基础颜色
    pub background: Color,
    pub foreground: Color,

    // 语义化颜色
    pub primary: Color,      // 主要信息（品牌、重要提示）
    pub secondary: Color,    // 次要信息
    pub success: Color,      // 成功状态
    pub warning: Color,      // 警告/处理中
    pub error: Color,        // 错误状态
    pub info: Color,         // 信息提示

    // 消息类型颜色
    pub user: Color,         // 用户消息
    pub assistant: Color,    // 助手消息
    pub tool: Color,         // 工具消息
    pub system: Color,       // 系统消息

    // UI 元素颜色
    pub border: Color,       // 边框
    pub hint: Color,         // 提示文字
    pub dim: Color,          // 暗淡文字
}
```

### 4.2 内置主题

```rust
impl Theme {
    /// 默认深色主题
    pub fn dark() -> Self {
        Theme {
            background: Color::Rgb(30, 30, 30),
            foreground: Color::Rgb(212, 212, 212),
            primary: Color::Rgb(97, 175, 239),
            secondary: Color::Rgb(144, 165, 174),
            success: Color::Rgb(152, 195, 121),
            warning: Color::Rgb(229, 192, 123),
            error: Color::Rgb(224, 108, 117),
            // ... 完整定义
        }
    }

    /// 高对比度主题（无障碍）
    pub fn high_contrast() -> Self {
        Theme {
            background: Color::Black,
            foreground: Color::White,
            primary: Color::Cyan,
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            // ... 更高对比度的颜色
        }
    }

    /// 浅色主题
    pub fn light() -> Self {
        Theme {
            background: Color::Rgb(255, 255, 255),
            foreground: Color::Rgb(40, 44, 52),
            // ... 浅色配色
        }
    }
}
```

### 4.3 主题配置文件

**`~/.config/oxide/theme.toml`**
```toml
[colors]
# 主题名称：dark, light, high_contrast, 或自定义
name = "dark"

# 基础颜色（支持 RGB hex 或命名颜色）
background = "#1e1e1e"
foreground = "#d4d4d4"

# 语义化颜色
primary = "#61afef"
success = "#98c379"
warning = "#e5c07b"
error = "#e06c75"

[style]
# 边框样式：single, double, rounded
border_style = "rounded"

# 消息间距：compact, normal, spacious
message_spacing = "normal"

# 是否显示图标
show_icons = true

# 动画效果
animations = true
```

### 4.4 主题切换命令

```bash
# 列出所有主题
oxide> /theme list

# 设置主题
oxide> /theme set dark
oxide> /theme set high_contrast

# 加载自定义主题
oxide> /theme load /path/to/custom.toml
```

---

## 五、交互功能增强

### 5.1 快捷键系统

| 快捷键 | 功能 | 说明 |
|--------|------|------|
| `?` | 显示帮助 | 全屏快捷键帮助 |
| `M` | 切换布局模式 | Standard → Compact → Minimal → Split |
| `Ctrl+T` | 工具面板 | 切换/调整工具面板 |
| `↑/↓` | 历史记录 | 浏览输入历史 |
| `Ctrl+R` | 搜索历史 | 向后搜索历史命令 |
| `Ctrl+L` | 清屏 | 清空消息历史 |
| `/` | 搜索消息 | 在消息中搜索 |
| `n/N` | 下/上匹配 | 跳转搜索结果 |
| `Ctrl+C` | 退出 | 确认后退出 |
| `Ctrl+G` | 开始多行 | 多行输入模式 |
| `Enter` | 发送 | 单行模式发送，多行模式换行 |

### 5.2 帮助屏幕

```
┌─────────────────────────────────────────────────────────┐
│                    Oxide 快捷键帮助                       │
├─────────────────────────────────────────────────────────┤
│  全局快捷键                                             │
│  ?         显示此帮助屏幕                                │
│  Ctrl+C    退出程序                                      │
│  Ctrl+L    清空消息历史                                  │
│  M         切换布局模式                                  │
│                                                         │
│  输入快捷键                                             │
│  Enter      发送消息                                     │
│  Ctrl+G     多行输入模式                                 │
│  ↑/↓        浏览输入历史                                 │
│  Ctrl+R     搜索历史                                     │
│                                                         │
│  导航快捷键                                             │
│  Ctrl+U    向上滚动                                      │
│  Ctrl+D    向下滚动                                      │
│  g/G       跳转到顶部/底部                               │
│  /         搜索消息内容                                  │
│                                                         │
│  按 `q` 或 `Esc` 关闭此帮助                             │
└─────────────────────────────────────────────────────────┘
```

### 5.3 命令历史

```rust
pub struct InputHistory {
    entries: Vec<String>,
    position: usize,
    filter: Option<String>,  // 搜索过滤器
}

impl InputHistory {
    // 上下键浏览
    pub fn up(&mut self) -> Option<&str>;
    pub fn down(&mut self) -> Option<&str>;

    // 搜索历史（Ctrl+R）
    pub fn search(&mut self, query: &str) -> Vec<&str>;

    // 持久化到文件
    pub fn save(&self, path: &Path) -> Result<()>;
    pub fn load(path: &Path) -> Result<Self>;
}
```

### 5.4 多行输入模式

```
┌─────────────────────────────────────────────────────────┐
│  Oxide · v0.1.0 · ● 就绪                                │
├─────────────────────────────────────────────────────────┤
│  [消息历史...]                                          │
│                                                         │
│  ╭─ 多行输入模式 ──────────────────────────────────╮   │
│  │ 请实现一个流式渲染函数：                         │  │
│  │                                                   │  │
│  │ ```rust                                          │  │
│  │ fn stream_render(text: &str) {                  │  │
│  │     for chunk in text.chunks(10) {              │  │
│  │         render(chunk);                          │  │
│  │     }                                            │  │
│  │ }                                                │  │
│  │ ```                                              │  │
│  │                                                   │  │
│  │ [Ctrl+G]完成 [Esc]取消                            │  │
│  ╰──────────────────────────────────────────────────╯  │
└─────────────────────────────────────────────────────────┘
```

---

## 六、流式渲染优化

### 6.1 增量 Markdown 解析

```rust
pub struct IncrementalMarkdownParser {
    buffer: String,
    ast: Option<MarkdownAST>,
}

impl IncrementalMarkdownParser {
    pub fn append(&mut self, chunk: &str) -> ParseResult {
        self.buffer.push_str(chunk);

        // 增量解析（避免每次全量解析）
        let new_ast = parse_markdown_partial(&self.buffer)?;

        // 只渲染变化的部分
        let delta = self.ast.diff(&new_ast);
        self.ast = Some(new_ast);

        Ok(delta)
    }
}
```

### 6.2 打字机效果优化

```rust
pub struct TypewriterEffect {
    full_content: String,
    visible_chars: usize,
    chars_per_tick: usize,
    adaptive: bool,  // 是否自适应速度
}

impl TypewriterEffect {
    pub fn tick(&mut self) -> bool {
        // 根据内容长度自适应速度
        let speed = if self.adaptive {
            (self.full_content.len() / 100).max(1)
        } else {
            self.chars_per_tick
        };

        let next = (self.visible_chars + speed).min(self.full_content.len());
        self.visible_chars = next;

        next < self.full_content.len()
    }

    // 按任意键跳过动画
    pub fn skip(&mut self) {
        self.visible_chars = self.full_content.len();
    }
}
```

---

## 七、性能优化

### 7.1 虚拟滚动

```rust
pub struct VirtualScroll<T> {
    items: Vec<T>,
    viewport_size: usize,
    offset: usize,
}

impl<T> VirtualScroll<T> {
    pub fn visible_items(&self) -> &[T] {
        let start = self.offset;
        let end = (start + self.viewport_size).min(self.items.len());
        &self.items[start..end]
    }

    pub fn scroll_up(&mut self, amount: usize) {
        self.offset = self.offset.saturating_sub(amount);
    }

    pub fn scroll_down(&mut self, amount: usize) {
        let max_offset = self.items.len().saturating_sub(self.viewport_size);
        self.offset = (self.offset + amount).min(max_offset);
    }
}
```

### 7.2 增量渲染

```rust
pub struct RenderCache {
    dirty_regions: Vec<Rect>,
    last_frame: Option<Frame>,
}

impl RenderCache {
    // 只重绘脏区域
    pub fn render_incremental(&mut self, frame: &mut Frame, app: &App) {
        if self.dirty_regions.is_empty() {
            // 全屏渲染
            render_full(frame, app);
        } else {
            // 增量渲染
            for region in &self.dirty_regions {
                render_region(frame, app, *region);
            }
        }

        self.dirty_regions.clear();
        self.last_frame = Some(frame.clone());
    }

    // 标记区域为脏
    pub fn mark_dirty(&mut self, region: Rect) {
        self.dirty_regions.push(region);
    }
}
```

---

## 八、实现路线图

### 阶段 1: 主题系统（2-3 天）
- [ ] 实现 `Theme` 结构和内置主题
- [ ] 实现主题配置文件加载（`toml` 解析）
- [ ] 重构所有渲染函数使用主题颜色
- [ ] 实现主题热切换
- [ ] 添加主题命令（`/theme list/set/load`）

### 阶段 2: 消息卡片（3-4 天）
- [ ] 实现消息卡片设计（`╭─╮│╰─╯` 边框）
- [ ] 内嵌工具状态到消息中
- [ ] 实现不同布局模式（Standard/Compact/Minimal/Split）
- [ ] 添加模式切换逻辑（`M` 键）
- [ ] 更新渲染逻辑支持布局模式

### 阶段 3: 交互功能（3-4 天）
- [ ] 实现快捷键帮助屏幕（`?` 键）
- [ ] 实现命令历史（`↑/↓` 键）
- [ ] 实现历史搜索（`Ctrl+R`）
- [ ] 实现多行输入模式（`Ctrl+G`）
- [ ] 实现消息搜索（`/` 键）

### 阶段 4: 流式渲染增强（2-3 天）
- [ ] 实现增量 Markdown 解析器
- [ ] 优化打字机效果（自适应速度）
- [ ] 实现跳过动画功能
- [ ] 添加流式渲染性能监控

### 阶段 5: 性能优化（2-3 天）
- [ ] 实现虚拟滚动
- [ ] 实现渲染缓存
- [ ] 优化增量渲染逻辑
- [ ] 添加性能监控和日志

### 阶段 6: 测试和文档（2-3 天）
- [ ] 编写单元测试
- [ ] 编写集成测试
- [ ] 更新用户文档
- [ ] 添加主题配置示例

---

## 九、向后兼容性

**此变更不保持向后兼容，会破坏以下内容：**
- 移除非 TUI 模式（`--no-tui` 参数）
- 移除简单终端模式
- TUI 成为唯一的用户界面
- 所有交互必须在支持 TUI 的终端中运行

**新增配置文件：**
- `~/.config/oxide/theme.toml`（可选，如不存在使用默认主题）
- `~/.config/oxide/history.toml`（命令历史，自动生成）

---

## 十、未来扩展

1. **动态主题**：根据终端背景自动调整
2. **主题市场**：用户贡献和分享主题
3. **插件系统**：支持自定义消息渲染器
4. **AI 布局优化**：根据使用习惯自动调整布局
5. **触摸支持**：支持移动端终端应用

---

## 参考资料

- [Claude Code UI Design](https://www.anthropic.com/engineering/claude-code-best-practices)
- [Warp Terminal Blocks](https://docs.warp.dev/terminal/blocks)
- [Lazygit UI Design](https://github.com/jesseduffield/lazygit)
- [Ratatui Examples](https://github.com/ratatui-org/ratatui/tree/master/examples)
- 详细设计文档: `docs/tui-redesign.md`
