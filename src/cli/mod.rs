pub mod command;
pub mod render;

use anyhow::Result;
use colored::*;
use dialoguer::Select;
use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
use rustyline::hint::{Hinter, HistoryHinter};
use rustyline::validate::{self, MatchingBracketValidator, Validator};
use rustyline::Editor;
use rustyline::{Context, Helper};
use std::borrow::Cow::{self, Borrowed, Owned};
use std::collections::HashMap;

use crate::context::ContextManager;

// 命令信息结构
#[derive(Clone, Debug)]
struct CommandInfo {
    #[allow(dead_code)]
    name: String,
    description: String,
}

impl CommandInfo {
    fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
        }
    }
}

// 自定义补全器
pub struct OxideHelper {
    #[allow(dead_code)]
    completer: FilenameCompleter,
    highlighter: MatchingBracketHighlighter,
    validator: MatchingBracketValidator,
    hinter: HistoryHinter,
    commands: HashMap<String, CommandInfo>,
}

impl Default for OxideHelper {
    fn default() -> Self {
        let mut commands = HashMap::new();
        commands.insert("/quit".to_string(), CommandInfo::new("/quit", "退出程序"));
        commands.insert("/exit".to_string(), CommandInfo::new("/exit", "退出程序"));
        commands.insert("/clear".to_string(), CommandInfo::new("/clear", "清除屏幕"));
        commands.insert("/config".to_string(), CommandInfo::new("/config", "显示当前配置"));
        commands.insert("/help".to_string(), CommandInfo::new("/help", "显示帮助信息"));
        commands.insert(
            "/toggle-tools".to_string(),
            CommandInfo::new("/toggle-tools", "切换工具显示"),
        );
        commands.insert(
            "/history".to_string(),
            CommandInfo::new("/history", "显示对话历史"),
        );
        commands.insert(
            "/load".to_string(),
            CommandInfo::new("/load <session_id>", "加载指定会话"),
        );
        commands.insert(
            "/sessions".to_string(),
            CommandInfo::new("/sessions", "列出所有会话"),
        );
        commands.insert(
            "/delete".to_string(),
            CommandInfo::new("/delete <session_id>", "删除指定会话"),
        );
        commands.insert(
            "/agent".to_string(),
            CommandInfo::new("/agent [list|switch <type>]", "管理 Agent 类型"),
        );

        Self {
            completer: FilenameCompleter::new(),
            highlighter: MatchingBracketHighlighter::new(),
            validator: MatchingBracketValidator::new(),
            hinter: HistoryHinter {},
            commands,
        }
    }
}

impl Completer for OxideHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        if line.starts_with('/') {
            let input = &line[..pos];
            let mut matches = Vec::new();

            for (cmd_name, cmd_info) in &self.commands {
                // 如果输入为空或者是 "/" 的子串，显示所有命令
                // 否则只显示匹配的命令
                if input.is_empty() || input == "/" || cmd_name.starts_with(input) {
                    // 显示格式：命令名称 - 描述
                    matches.push(Pair {
                        display: format!("{} - {}", cmd_name, cmd_info.description),
                        replacement: cmd_name.clone(),
                    });
                }
            }

            // 按字母顺序排序
            matches.sort_by(|a, b| a.display.cmp(&b.display));

            Ok((0, matches))
        } else {
            Ok((pos, vec![]))
        }
    }
}

impl Hinter for OxideHelper {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<String> {
        self.hinter.hint(line, pos, ctx)
    }
}

impl Highlighter for OxideHelper {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        default: bool,
    ) -> Cow<'b, str> {
        if default {
            Borrowed(prompt)
        } else {
            Owned(prompt.to_string())
        }
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Owned(format!("{}", hint.dimmed()))
    }

    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        if line.starts_with('/') {
            if let Some(space_pos) = line.find(' ') {
                let command = &line[..space_pos];
                let rest = &line[space_pos..];
                if self.commands.contains_key(command) {
                    return Owned(format!("{}{}", command.bright_green(), rest));
                }
            } else if self.commands.contains_key(line) {
                return Owned(line.bright_green().to_string());
            }
        }

        self.highlighter.highlight(line, pos)
    }

    fn highlight_char(&self, line: &str, pos: usize, forced: bool) -> bool {
        self.highlighter.highlight_char(line, pos, forced)
    }
}

impl Validator for OxideHelper {
    fn validate(
        &self,
        ctx: &mut validate::ValidationContext,
    ) -> rustyline::Result<validate::ValidationResult> {
        self.validator.validate(ctx)
    }

    fn validate_while_typing(&self) -> bool {
        self.validator.validate_while_typing()
    }
}

impl Helper for OxideHelper {}

pub const LOGO: &str = r#"
 _______          _________ ______   _______
(  ___  )|\     /|\__   __/(  __  \ (  ____ \
| (   ) |( \   / )   ) (   | (  \  )| (    \/
| |   | | \ (_) /    | |   | |   ) || (__
| |   | |  ) _ (     | |   | |   | ||  __)
| |   | | / ( ) \    | |   | |   ) || (
| (___) |( /   \ )___) (___| (__/  )| (____/\
(_______)|/     \|\_______/(______/ (_______/
"#;

use crate::agent::AgentType;
use crate::cli::render::Spinner;

pub struct OxideCli {
    pub api_key: String,
    pub model_name: String,
    pub agent: AgentType,
    pub context_manager: ContextManager,
    spinner: Spinner,
}

impl OxideCli {
    pub fn new(
        api_key: String,
        model_name: String,
        agent: AgentType,
        context_manager: ContextManager,
    ) -> Self {
        Self {
            api_key,
            model_name,
            agent,
            context_manager,
            spinner: Spinner::new(),
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        println!("{}", LOGO);
        self.show_welcome()?;
        self.show_tips()?;

        let result = self.run_input_loop().await;

        match result {
            Ok(_) => println!("\n{}", "👋 Goodbye!".bright_cyan()),
            Err(e) => {
                println!("\n{} {}", "❌ Error:".red(), e);
                return Err(e);
            }
        }

        Ok(())
    }

    /// 显示命令选择器
    fn show_command_selector(&self) -> Result<String> {
        // 获取 OxideHelper 中的命令信息
        let helper = OxideHelper::default();

        // 准备命令列表（带描述）
        let mut command_items: Vec<String> = helper
            .commands
            .iter()
            .map(|(name, info)| format!("{} - {}", name, info.description))
            .collect();

        // 按命令名称排序
        command_items.sort();

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

    async fn run_input_loop(&mut self) -> Result<()> {
        let mut rl = Editor::new()?;
        rl.set_helper(Some(OxideHelper::default()));

        loop {
            self.print_separator()?;
            let readline = rl.readline("❯ ");

            match readline {
                Ok(line) => {
                    let input = line.trim();
                    if input.is_empty() {
                        continue;
                    }

                    // 检测是否输入了 "/"，如果是则弹出命令选择器
                    let final_input = if input == "/" {
                        println!("{}", "检测到 / 命令，正在打开选择器...".dimmed());
                        match self.show_command_selector() {
                            Ok(selected) => {
                                println!("\n{} {}", "✓".green(), selected.bright_green());
                                selected
                            }
                            Err(e) => {
                                // 用户取消选择（如按 Esc）
                                println!("\n{} {}", "⚠️".yellow(), format!("选择失败: {}", e));
                                continue;
                            }
                        }
                    } else {
                        input.to_string()
                    };

                    let _ = rl.add_history_entry(&final_input);

                    self.print_separator()?;

                    let should_continue = self.handle_command(&final_input).await?;
                    if !should_continue {
                        break;
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    println!("{}", "^C".dimmed());
                    break;
                }
                Err(ReadlineError::Eof) => {
                    break;
                }
                Err(err) => {
                    println!("{} {:?}", "Error:".red(), err);
                    break;
                }
            }
        }

        Ok(())
    }

    pub fn print_separator(&self) -> Result<()> {
        let width = 80;
        let separator = "-".repeat(width);
        println!("{}", separator.dimmed());
        Ok(())
    }

    #[allow(dead_code)]
    pub fn session_id(&self) -> &str {
        self.context_manager.session_id()
    }
}
