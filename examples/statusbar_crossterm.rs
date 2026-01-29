use crossterm::{
    cursor::{self, MoveTo, MoveToColumn},
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    queue,
    style::{Color, Print, SetBackgroundColor, SetForegroundColor, ResetColor},
    terminal::{self, Clear, ClearType},
};
use std::io::{stdout, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

struct LineEditor {
    buffer: String,
    cursor_pos: usize,
    history: Vec<String>,
    history_index: Option<usize>,
}

impl LineEditor {
    fn new() -> Self {
        Self {
            buffer: String::new(),
            cursor_pos: 0,
            history: Vec::new(),
            history_index: None,
        }
    }

    fn insert_char(&mut self, c: char) {
        self.buffer.insert(self.cursor_pos, c);
        self.cursor_pos += 1;
    }

    fn delete_char(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
            self.buffer.remove(self.cursor_pos);
        }
    }

    fn delete_char_forward(&mut self) {
        if self.cursor_pos < self.buffer.len() {
            self.buffer.remove(self.cursor_pos);
        }
    }

    fn move_left(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
        }
    }

    fn move_right(&mut self) {
        if self.cursor_pos < self.buffer.len() {
            self.cursor_pos += 1;
        }
    }

    fn move_home(&mut self) {
        self.cursor_pos = 0;
    }

    fn move_end(&mut self) {
        self.cursor_pos = self.buffer.len();
    }

    fn history_up(&mut self) {
        if self.history.is_empty() {
            return;
        }
        match self.history_index {
            None => {
                self.history_index = Some(self.history.len() - 1);
            }
            Some(i) if i > 0 => {
                self.history_index = Some(i - 1);
            }
            _ => return,
        }
        if let Some(i) = self.history_index {
            self.buffer = self.history[i].clone();
            self.cursor_pos = self.buffer.len();
        }
    }

    fn history_down(&mut self) {
        match self.history_index {
            Some(i) if i < self.history.len() - 1 => {
                self.history_index = Some(i + 1);
                self.buffer = self.history[i + 1].clone();
                self.cursor_pos = self.buffer.len();
            }
            Some(_) => {
                self.history_index = None;
                self.buffer.clear();
                self.cursor_pos = 0;
            }
            None => {}
        }
    }

    fn submit(&mut self) -> String {
        let result = self.buffer.clone();
        if !result.trim().is_empty() {
            self.history.push(result.clone());
        }
        self.buffer.clear();
        self.cursor_pos = 0;
        self.history_index = None;
        result
    }

    fn clear(&mut self) {
        self.buffer.clear();
        self.cursor_pos = 0;
        self.history_index = None;
    }
}

struct StatusBar {
    width: u16,
    height: u16,
    input_count: usize,
    refresh_count: u32,
}

impl StatusBar {
    fn new(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            input_count: 0,
            refresh_count: 0,
        }
    }

    fn render(&mut self) -> anyhow::Result<()> {
        let mut stdout = stdout();
        self.refresh_count = self.refresh_count.wrapping_add(1);

        // 保存光标位置
        queue!(stdout, cursor::SavePosition)?;

        // 移动到状态栏行（终端底部）
        queue!(stdout, MoveTo(0, self.height - 1))?;

        // 绘制状态栏内容
        let info = format!(
            " 输入: {} | 刷新: {} | 终端: {}x{} ",
            self.input_count, self.refresh_count, self.width, self.height
        );
        let padding_len = (self.width as usize).saturating_sub(info.chars().count());
        let padding = " ".repeat(padding_len);

        queue!(
            stdout,
            SetBackgroundColor(Color::Rgb { r: 68, g: 68, b: 68 }),
            SetForegroundColor(Color::White),
            Print(&info),
            Print(&padding),
            ResetColor
        )?;

        // 恢复光标位置
        queue!(stdout, cursor::RestorePosition)?;
        stdout.flush()?;

        Ok(())
    }

    fn increment_input(&mut self) {
        self.input_count += 1;
    }
}

fn render_input(editor: &LineEditor) -> anyhow::Result<()> {
    let mut stdout = stdout();
    let prompt_len = 2u16; // "> "

    queue!(
        stdout,
        MoveToColumn(0),
        Clear(ClearType::CurrentLine),
        SetForegroundColor(Color::Green),
        Print("> "),
        ResetColor,
        Print(&editor.buffer),
        MoveToColumn(prompt_len + editor.cursor_pos as u16)
    )?;
    stdout.flush()?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    println!("=== Crossterm 状态栏测试 ===");
    println!("按 Ctrl+D 或 Ctrl+C 退出");
    println!("支持: 左右方向键、Home/End、上下历史、Backspace/Delete\n");

    let (width, height) = terminal::size()?;
    if height < 5 {
        println!("错误: 终端高度不足");
        return Ok(());
    }

    // 启用 raw mode
    terminal::enable_raw_mode()?;

    let statusbar = Arc::new(Mutex::new(StatusBar::new(width, height)));
    let statusbar_clone = statusbar.clone();
    let running = Arc::new(Mutex::new(true));
    let running_clone = running.clone();

    // 后台线程：持续刷新状态栏
    let refresh_handle = thread::spawn(move || {
        while *running_clone.lock().unwrap() {
            thread::sleep(Duration::from_millis(100));
            if let Ok(mut sb) = statusbar_clone.lock() {
                let _ = sb.render();
            }
        }
    });

    // 主循环
    let mut editor = LineEditor::new();
    render_input(&editor)?;

    loop {
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(KeyEvent { code, modifiers, .. }) = event::read()? {
                match (code, modifiers) {
                    // 退出
                    (KeyCode::Char('d'), KeyModifiers::CONTROL) |
                    (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                        break;
                    }

                    // 提交输入
                    (KeyCode::Enter, _) => {
                        let input = editor.submit();
                        let mut stdout = stdout();

                        // 清除当前行，打印输出，换行
                        queue!(
                            stdout,
                            MoveToColumn(0),
                            Clear(ClearType::CurrentLine),
                        )?;

                        if !input.trim().is_empty() {
                            queue!(stdout, Print(format!("输入: {}\r\n", input)))?;
                            statusbar.lock().unwrap().increment_input();
                        } else {
                            queue!(stdout, Print("\r\n"))?;
                        }
                        stdout.flush()?;

                        render_input(&editor)?;
                    }

                    // 编辑操作
                    (KeyCode::Backspace, _) => {
                        editor.delete_char();
                        render_input(&editor)?;
                    }
                    (KeyCode::Delete, _) => {
                        editor.delete_char_forward();
                        render_input(&editor)?;
                    }
                    (KeyCode::Left, _) => {
                        editor.move_left();
                        render_input(&editor)?;
                    }
                    (KeyCode::Right, _) => {
                        editor.move_right();
                        render_input(&editor)?;
                    }
                    (KeyCode::Home, _) => {
                        editor.move_home();
                        render_input(&editor)?;
                    }
                    (KeyCode::End, _) => {
                        editor.move_end();
                        render_input(&editor)?;
                    }
                    (KeyCode::Up, _) => {
                        editor.history_up();
                        render_input(&editor)?;
                    }
                    (KeyCode::Down, _) => {
                        editor.history_down();
                        render_input(&editor)?;
                    }

                    // 清除当前行
                    (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
                        editor.clear();
                        render_input(&editor)?;
                    }

                    // 普通字符输入
                    (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                        editor.insert_char(c);
                        render_input(&editor)?;
                    }

                    _ => {}
                }
            }
        }
    }

    // 清理
    *running.lock().unwrap() = false;
    refresh_handle.join().ok();

    // 清除状态栏
    let mut stdout = stdout();
    queue!(
        stdout,
        cursor::SavePosition,
        MoveTo(0, height - 1),
        Clear(ClearType::CurrentLine),
        cursor::RestorePosition
    )?;
    stdout.flush()?;

    terminal::disable_raw_mode()?;
    println!("\r\n测试完成");

    Ok(())
}
