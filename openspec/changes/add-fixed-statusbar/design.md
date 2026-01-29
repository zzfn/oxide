# 设计文档：固定底部状态栏

## 架构概览

```
┌─────────────────────────────────────────┐
│  对话历史区域（滚动）                    │
│  ● oxide: 你好                          │
│  ● user: 帮我分析代码                   │
│  ● oxide: 好的，让我看看...             │
│  ...                                    │
│  [此区域可滚动]                          │
├─────────────────────────────────────────┤ ← 滚动区域边界
│ 📊 Tokens: 1234 | Session: abc-123 ... │ ← 固定状态栏
└─────────────────────────────────────────┘
```

## 核心组件

### 1. StatusBar 模块 (`src/cli/statusbar.rs`)

```rust
pub struct StatusBar {
    enabled: bool,
    terminal_height: u16,
    terminal_width: u16,
}

pub struct StatusData {
    pub total_tokens: u64,
    pub session_id: String,
    pub model_name: String,
    pub cwd: PathBuf,
}

impl StatusBar {
    /// 初始化状态栏，设置终端滚动区域（DECSTBM）
    pub fn init(&mut self) -> Result<()> {
        let (width, height) = crossterm::terminal::size()?;
        self.terminal_width = width;
        self.terminal_height = height;

        // 设置滚动区域：第 1 行到倒数第 2 行
        // 格式：\x1b[{top};{bottom}r
        print!("\x1b[1;{}r", height - 1);
        stdout().flush()?;
        Ok(())
    }

    /// 更新状态栏显示（使用 crossterm SavePosition/RestorePosition）
    pub fn update(&self, data: &StatusData) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        use crossterm::{cursor, execute};
        let mut sw = stdout();

        // 1. 保存输入行的光标位置
        execute!(sw, cursor::SavePosition)?;

        // 2. 瞬移到最后一行
        execute!(sw, cursor::MoveTo(0, self.terminal_height - 1))?;

        // 3. 渲染状态栏（带背景色 + 清除到行尾）
        let status_line = self.format_status(data);
        print!("\x1b[48;5;238m{}\x1b[0K\x1b[0m", status_line);

        // 4. 恢复光标到输入行位置
        execute!(sw, cursor::RestorePosition)?;
        sw.flush()?;
        Ok(())
    }

    /// 清理状态栏，恢复终端状态（CRITICAL）
    pub fn cleanup(&self) -> Result<()> {
        // 重置滚动区域到全屏
        print!("\x1b[r");

        // 清除状态栏行
        print!("\x1b[{};1H\x1b[2K", self.terminal_height);

        stdout().flush()?;
        Ok(())
    }

    /// 处理终端尺寸变化
    pub fn handle_resize(&mut self, width: u16, height: u16) -> Result<()> {
        self.terminal_width = width;
        self.terminal_height = height;
        self.init() // 重新设置滚动区域
    }

    fn format_status(&self, data: &StatusData) -> String {
        let session_short = if data.session_id.len() > 8 {
            &data.session_id[..8]
        } else {
            &data.session_id
        };

        format!(
            " 模型: {} | Token: {} | Session: {}... ",
            data.model_name,
            data.total_tokens,
            session_short
        )
    }
}
```

### 2. 集成到 OxideCli 与 Reedline

**关键原则：解耦**
- 状态栏不是 Prompt 的一部分，而是独立的终端控制层
- Reedline 在滚动区运行，状态栏在固定区独立渲染

```rust
// src/cli/mod.rs
pub struct OxideCli {
    // ... 现有字段
    statusbar: StatusBar,
}

impl OxideCli {
    pub fn new(...) -> Self {
        let mut statusbar = StatusBar::new();
        statusbar.init().ok(); // 设置滚动区域

        Self {
            // ...
            statusbar,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        // 确保退出时清理
        let _cleanup = StatusBarCleanup(&self.statusbar);

        // Reedline 事件循环
        loop {
            // 每次读取输入前刷新状态栏
            self.statusbar.update(&self.get_status_data()).ok();

            let sig = line_editor.read_line(&prompt);
            match sig {
                Ok(Signal::Success(buffer)) => {
                    // 处理命令...
                }
                Ok(Signal::CtrlC) | Ok(Signal::CtrlD) => break,
                _ => {}
            }
        }

        Ok(())
    }

    fn get_status_data(&self) -> StatusData {
        StatusData {
            total_tokens: self.total_tokens.load(Ordering::Relaxed),
            session_id: self.context_manager.session_id().to_string(),
            model_name: self.model_name.clone(),
            cwd: std::env::current_dir().unwrap_or_default(),
        }
    }
}

// RAII 清理辅助
struct StatusBarCleanup<'a>(&'a StatusBar);
impl Drop for StatusBarCleanup<'_> {
    fn drop(&mut self) {
        let _ = self.0.cleanup();
    }
}
```

**Reedline Hook 点**：
1. **循环开始前**：每次 `read_line()` 前刷新状态栏
2. **Validator 中**（可选）：实时响应用户输入
3. **ExternalPrinter**：流式输出时异步更新

### 3. 流式输出集成

**关键：流式输出时 Reedline 已交出控制权**

```rust
// src/cli/render.rs
pub async fn stream_with_animation<R>(
    stream: &mut StreamingResult<R>,
    statusbar: &StatusBar,
    status_data_fn: impl Fn() -> StatusData,
) -> Result<FinalResponse, std::io::Error> {
    let mut token_counter = 0;
    let mut last_update = Instant::now();

    while let Some(content) = stream.next().await {
        match content {
            Ok(MultiTurnStreamItem::StreamAssistantItem(...)) => {
                // 渲染文本（在滚动区）
                renderer.process_text(&text.text, skin);

                token_counter += estimate_tokens(&text.text);

                // 限流更新：每 100ms 或每 10 个 token
                if last_update.elapsed() > Duration::from_millis(100) || token_counter >= 10 {
                    statusbar.update(&status_data_fn()).ok();
                    last_update = Instant::now();
                    token_counter = 0;
                }
            }
            // ...
        }
    }

    // 最终更新
    statusbar.update(&status_data_fn()).ok();
    Ok(final_res)
}
```

## ANSI 转义序列参考

| 序列 | 功能 | 说明 |
|------|------|------|
| `\x1b[s` | 保存光标位置 | 保存当前光标坐标 |
| `\x1b[u` | 恢复光标位置 | 恢复到上次保存的位置 |
| `\x1b[{row};{col}H` | 移动光标 | 移动到指定行列（1-based） |
| `\x1b[2K` | 清除整行 | 清除光标所在行的所有内容 |
| `\x1b[{top};{bottom}r` | 设置滚动区域 | 限制滚动范围 |
| `\x1b[r` | 重置滚动区域 | 恢复全屏滚动 |
| `\x1b[48;5;{color}m` | 设置背景色 | 256 色模式 |
| `\x1b[0m` | 重置样式 | 清除所有颜色和样式 |

## 终端能力检测

```rust
fn is_ansi_supported() -> bool {
    // 检查 TERM 环境变量
    if let Ok(term) = std::env::var("TERM") {
        if term == "dumb" || term.is_empty() {
            return false;
        }
    }

    // 检查是否是 TTY
    if !crossterm::tty::IsTty::is_tty(&std::io::stdout()) {
        return false;
    }

    // Windows 需要额外检查
    #[cfg(windows)]
    {
        // Windows 10+ 支持 ANSI
        return crossterm::ansi_support::supports_ansi();
    }

    #[cfg(not(windows))]
    true
}
```

## 性能考虑

1. **更新频率限制**：
   - 不在每个 token 到达时都更新状态栏
   - 使用时间阈值（如每 100ms）或 token 阈值（如每 10 个 token）

2. **缓冲输出**：
   - 所有 ANSI 序列和状态文本一次性写入，减少系统调用

3. **避免闪烁**：
   - 使用 `\x1b[s` 和 `\x1b[u` 而非重复的绝对定位
   - 先清除再绘制，避免旧内容残留

## 边界情况处理

1. **终端高度不足**：
   - 如果 `height < 5`，禁用状态栏
   - 避免滚动区域过小导致不可用

2. **非交互式环境**：
   - 检测 stdout 是否是 TTY
   - 管道或重定向时自动禁用

3. **终端尺寸变化（CRITICAL）**：
   ```rust
   // 使用 crossterm 监听 Resize 事件
   use crossterm::event::{Event, EventStream};

   // 在后台任务中监听
   tokio::spawn(async move {
       let mut reader = EventStream::new();
       while let Some(Ok(Event::Resize(w, h))) = reader.next().await {
           statusbar.handle_resize(w, h); // 重新设置滚动区域
       }
   });
   ```

4. **退出清理（CRITICAL）**：
   - 必须执行 `print!("\x1b[r")` 重置滚动区域
   - 使用 Drop trait 或 signal handler 确保执行
   - 否则用户回到 Shell 后终端仍被限制

5. **并发安全**：
   - 状态栏更新与流式输出可能并发
   - 使用 `Mutex` 或确保单线程更新

## 测试策略

1. **单元测试**：
   - 测试 `format_status()` 的截断逻辑
   - 测试终端能力检测函数

2. **集成测试**：
   - 模拟终端环境（使用 pty）
   - 验证 ANSI 序列输出正确性

3. **手动测试**：
   - 在多种终端模拟器中测试
   - 测试极端尺寸（很小、很大）
   - 测试快速调整窗口大小

## 未来扩展

1. **多行状态栏**：支持显示更多信息（如最近的工具调用）
2. **可配置内容**：允许用户自定义状态栏显示项
3. **颜色主题**：支持自定义状态栏配色
4. **交互式元素**：支持点击状态栏项查看详情（需要鼠标支持）
