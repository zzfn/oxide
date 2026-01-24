use anyhow::Result;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    queue,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, Clear, ClearType},
};
use dialoguer::Select;
use std::collections::HashMap;
use std::io::{self, Write};

// 命令信息结构
#[derive(Clone, Debug)]
pub struct CommandInfo {
    pub name: String,
    pub description: String,
}

impl CommandInfo {
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
        }
    }
}

// 即时输入处理器
pub struct InstantInput {
    commands: HashMap<String, CommandInfo>,
}

impl InstantInput {
    pub fn new() -> Self {
        let mut commands = HashMap::new();
        commands.insert("/quit".to_string(), CommandInfo::new("/quit", "退出程序"));
        commands.insert("/exit".to_string(), CommandInfo::new("/exit", "退出程序"));
        commands.insert("/clear".to_string(), CommandInfo::new("/clear", "清除屏幕"));
        commands.insert("/config".to_string(), CommandInfo::new("/config", "显示当前配置"));
        commands.insert("/help".to_string(), CommandInfo::new("/help", "显示帮助信息"));
        commands.insert(
            "/toggle-tools".to_string(),
            CommandInfo::new("/toggle-tools", "显示工具状态"),
        );
        commands.insert("/history".to_string(), CommandInfo::new("/history", "显示对话历史"));
        commands.insert("/load".to_string(), CommandInfo::new("/load <session_id>", "加载指定会话"));
        commands.insert("/sessions".to_string(), CommandInfo::new("/sessions", "列出所有会话"));
        commands.insert(
            "/delete".to_string(),
            CommandInfo::new("/delete <session_id>", "删除指定会话"),
        );

        Self { commands }
    }

    /// 读取用户输入，支持即时命令菜单
    pub fn read_line_with_instant_menu(&self, prompt: &str) -> Result<String> {
        terminal::enable_raw_mode()?;

        let mut input = String::new();
        let mut show_menu = false;
        let stdout = io::stdout();
        let mut stdout = stdout.lock();

        loop {
            // 清除当前行并显示提示符和输入
            queue!(stdout, Clear(ClearType::CurrentLine))?;
            queue!(stdout, cursor::MoveToColumn(0))?;
            queue!(stdout, SetForegroundColor(Color::Cyan))?;
            queue!(stdout, Print(prompt))?;
            queue!(stdout, ResetColor)?;
            queue!(stdout, Print(&input))?;
            stdout.flush()?;

            if show_menu {
                // 显示命令菜单（在当前行下方）
                self.show_command_menu(&mut stdout)?;
            }

            // 等待按键事件
            if let Event::Key(key) = event::read()? {
                // 先检查控制键组合
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    match key.code {
                        KeyCode::Char('c') => {
                            // Ctrl-C 退出
                            queue!(stdout, Print("^C"))?;
                            stdout.flush()?;
                            terminal::disable_raw_mode()?;
                            println!();
                            return Ok(String::new());
                        }
                        KeyCode::Char('d') => {
                            // Ctrl-D 退出
                            terminal::disable_raw_mode()?;
                            println!();
                            return Ok(String::new());
                        }
                        _ => {}
                    }
                    continue;
                }

                // 处理普通按键
                match key.code {
                    KeyCode::Char(c) => {
                        // 检测是否输入了 '/'
                        if c == '/' && input.is_empty() {
                            show_menu = true;
                            input.push(c);
                        } else {
                            // 如果菜单打开，需要先清除菜单
                            if show_menu {
                                self.clear_menu(&mut stdout)?;
                                show_menu = false;
                            }
                            input.push(c);
                        }
                    }
                    KeyCode::Backspace => {
                        if !input.is_empty() {
                            input.pop();
                            // 如果删除了 '/'，关闭菜单
                            if input.is_empty() || !input.starts_with('/') {
                                if show_menu {
                                    self.clear_menu(&mut stdout)?;
                                    show_menu = false;
                                }
                            }
                        }
                    }
                    KeyCode::Enter => {
                        // 如果菜单打开，先清除
                        if show_menu {
                            self.clear_menu(&mut stdout)?;
                        }
                        println!();
                        terminal::disable_raw_mode()?;
                        return Ok(input.trim().to_string());
                    }
                    KeyCode::Esc => {
                        // ESC 键关闭菜单
                        if show_menu {
                            self.clear_menu(&mut stdout)?;
                            show_menu = false;
                        } else {
                            // 如果菜单未打开，ESC 退出
                            println!();
                            terminal::disable_raw_mode()?;
                            return Ok(String::new());
                        }
                    }
                    KeyCode::Tab => {
                        // 如果菜单显示中，Tab 可以用于自动补全
                        if show_menu && input.starts_with('/') {
                            if let Some(completed) = self.try_complete(&input) {
                                self.clear_menu(&mut stdout)?;
                                input = completed;
                                show_menu = false;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    /// 显示命令菜单
    fn show_command_menu(&self, stdout: &mut io::StdoutLock) -> Result<()> {
        // 移动到下一行开始显示菜单
        queue!(stdout, cursor::MoveToNextLine(1))?;

        // 显示菜单标题
        queue!(stdout, SetForegroundColor(Color::Yellow))?;
        queue!(stdout, Print("可用命令:\n"))?;
        queue!(stdout, ResetColor)?;

        // 显示命令列表（按字母排序）
        let mut commands: Vec<_> = self.commands.iter().collect();
        commands.sort_by(|a, b| a.0.cmp(b.0));

        for (cmd_name, cmd_info) in commands {
            queue!(stdout, SetForegroundColor(Color::Green))?;
            queue!(stdout, Print(format!("  {}", cmd_name)))?;
            queue!(stdout, ResetColor)?;
            queue!(stdout, SetForegroundColor(Color::DarkGrey))?;
            queue!(stdout, Print(format!(" - {}\n", cmd_info.description)))?;
            queue!(stdout, ResetColor)?;
        }

        // 显示提示
        queue!(stdout, SetForegroundColor(Color::Cyan))?;
        queue!(stdout, Print("提示: 输入命令名或按 ESC 关闭菜单"))?;
        queue!(stdout, ResetColor)?;

        // 移动光标回到输入行
        let menu_height = self.commands.len() as u16 + 2; // 命令数 + 标题 + 提示
        queue!(stdout, cursor::MoveUp(menu_height))?;
        queue!(stdout, cursor::MoveToColumn(0))?;

        stdout.flush()?;
        Ok(())
    }

    /// 清除菜单显示
    fn clear_menu(&self, stdout: &mut io::StdoutLock) -> Result<()> {
        // 移动到菜单开始位置（下一行）
        queue!(stdout, cursor::MoveToNextLine(1))?;

        // 清除所有菜单行
        let menu_height = self.commands.len() as u16 + 2;
        for _ in 0..menu_height {
            queue!(stdout, Clear(ClearType::CurrentLine))?;
            queue!(stdout, cursor::MoveDown(1))?;
        }

        // 移动光标回到输入行
        queue!(stdout, cursor::MoveUp(menu_height))?;
        queue!(stdout, cursor::MoveToColumn(0))?;

        stdout.flush()?;
        Ok(())
    }

    /// 尝试自动补全命令
    fn try_complete(&self, input: &str) -> Option<String> {
        let matches: Vec<&String> = self
            .commands
            .keys()
            .filter(|cmd| cmd.starts_with(input))
            .collect();

        if matches.len() == 1 {
            Some(matches[0].clone())
        } else {
            None
        }
    }

    /// 显示交互式命令选择器（用于多行输入或复杂选择）
    pub fn show_command_selector(&self) -> Result<String> {
        // 准备命令列表（带描述）
        let mut command_items: Vec<String> = self
            .commands
            .iter()
            .map(|(name, info)| format!("{} - {}", name, info.description))
            .collect();

        // 按命令名称排序
        command_items.sort();

        // 临时退出原始模式以显示选择器
        terminal::disable_raw_mode()?;

        // 显示选择器
        let selection = Select::new()
            .with_prompt("请选择命令")
            .items(&command_items)
            .default(0)
            .interact()?;

        // 提取命令名称（去除描述部分）
        let selected = command_items[selection]
            .split(" - ")
            .next()
            .unwrap_or("/")
            .to_string();

        Ok(selected)
    }
}

impl Default for InstantInput {
    fn default() -> Self {
        Self::new()
    }
}
