pub mod command;
pub mod render;

use anyhow::Result;
use colored::*;
use dialoguer::FuzzySelect;
use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
use rustyline::hint::{Hinter, HistoryHinter};
use rustyline::validate::{self, MatchingBracketValidator, Validator};
use rustyline::{Cmd, CompletionType, Config, Editor, EventHandler, KeyCode, KeyEvent, Modifiers};
use rustyline::{ConditionalEventHandler, Event, EventContext, RepeatCount};
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
        commands.insert(
            "/tasks".to_string(),
            CommandInfo::new("/tasks [list|show <id>]", "管理后台任务"),
        );
        commands.insert(
            "/skills".to_string(),
            CommandInfo::new("/skills [list|show <name>]", "管理技能"),
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

            // 添加内置命令
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

            // 添加动态技能
            if let Ok(skill_manager) = crate::skill::SkillManager::new() {
                for skill in skill_manager.list_skills() {
                    let cmd = format!("/{}", skill.name);
                    // 如果输入匹配这个技能命令
                    if input.is_empty() || input == "/" || cmd.starts_with(input) {
                        matches.push(Pair {
                            display: format!("{} - {}", cmd, skill.description),
                            replacement: cmd,
                        });
                    }
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
        // 如果输入以 / 开头，提供命令补全提示
        if line.starts_with('/') && !line.is_empty() {
            let input = &line[..pos];
            // 找到第一个匹配的命令，返回剩余部分作为提示
            let mut matched_commands: Vec<_> = self
                .commands
                .keys()
                .filter(|cmd| cmd.starts_with(input) && *cmd != input)
                .collect();
            matched_commands.sort();

            if let Some(cmd) = matched_commands.first() {
                return Some(cmd[input.len()..].to_string());
            }
        }

        // 否则使用历史提示
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

// ============================================================================
// 触发符处理器 - 拦截 /、@、# 按键
// ============================================================================

/// 特殊字符触发处理器
/// 当用户在空行输入 /、@、# 时，立即结束输入并弹出对应选择器
struct TriggerHandler {
    trigger_char: char,
}

impl TriggerHandler {
    fn new(trigger_char: char) -> Self {
        Self { trigger_char }
    }
}

impl ConditionalEventHandler for TriggerHandler {
    fn handle(
        &self,
        _evt: &Event,
        _n: RepeatCount,
        _positive: bool,
        ctx: &EventContext<'_>,
    ) -> Option<Cmd> {
        // 只在空行时触发
        if ctx.line().is_empty() {
            // 先插入触发字符，然后立即接受输入
            Some(Cmd::Insert(1, self.trigger_char.to_string()))
        } else {
            // 非空行时，使用默认行为（插入字符）
            None
        }
    }
}

/// 触发符类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TriggerType {
    /// / - 命令菜单
    Command,
    /// @ - 上下文/文件引用
    Context,
    /// # - 标签/话题
    Tag,
}

impl TriggerType {
    fn from_char(c: char) -> Option<Self> {
        match c {
            '/' => Some(TriggerType::Command),
            '@' => Some(TriggerType::Context),
            '#' => Some(TriggerType::Tag),
            _ => None,
        }
    }

    #[allow(dead_code)]
    fn prompt(&self) -> &'static str {
        match self {
            TriggerType::Command => "选择命令",
            TriggerType::Context => "选择上下文",
            TriggerType::Tag => "选择标签",
        }
    }
}

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

    /// 显示命令选择器（支持模糊搜索）
    fn show_command_selector(&self) -> Result<String> {
        // 获取 OxideHelper 中的命令信息
        let helper = OxideHelper::default();

        // 准备命令列表（带描述）
        let mut command_items: Vec<String> = helper
            .commands
            .iter()
            .map(|(name, info)| format!("{} - {}", name, info.description))
            .collect();

        // 添加技能到命令列表
        if let Ok(skill_manager) = crate::skill::SkillManager::new() {
            for skill in skill_manager.list_skills() {
                let cmd = format!("/{}", skill.name);
                command_items.push(format!("{} - {}", cmd, skill.description));
            }
        }

        // 按命令名称排序
        command_items.sort();

        // 使用 FuzzySelect 支持模糊搜索
        let selection = FuzzySelect::new()
            .with_prompt("选择命令 (输入过滤)")
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

    /// 显示上下文选择器（@ 触发）
    fn show_context_selector(&self) -> Result<String> {
        // TODO: 实现文件/上下文选择
        let context_items = vec![
            "@file - 引用文件",
            "@codebase - 搜索代码库",
            "@web - 搜索网页",
            "@docs - 搜索文档",
        ];

        let selection = FuzzySelect::new()
            .with_prompt("选择上下文 (输入过滤)")
            .items(&context_items)
            .default(0)
            .interact()?;

        let selected = context_items[selection]
            .split(" - ")
            .next()
            .unwrap_or("@")
            .to_string();

        Ok(selected)
    }

    /// 显示标签选择器（# 触发）
    fn show_tag_selector(&self) -> Result<String> {
        // TODO: 实现标签选择
        let tag_items = vec![
            "#bug - 问题修复",
            "#feature - 新功能",
            "#refactor - 重构",
            "#docs - 文档",
        ];

        let selection = FuzzySelect::new()
            .with_prompt("选择标签 (输入过滤)")
            .items(&tag_items)
            .default(0)
            .interact()?;

        let selected = tag_items[selection]
            .split(" - ")
            .next()
            .unwrap_or("#")
            .to_string();

        Ok(selected)
    }

    /// 根据触发符类型显示对应选择器
    fn show_trigger_selector(&self, trigger: TriggerType) -> Result<String> {
        match trigger {
            TriggerType::Command => self.show_command_selector(),
            TriggerType::Context => self.show_context_selector(),
            TriggerType::Tag => self.show_tag_selector(),
        }
    }

    async fn run_input_loop(&mut self) -> Result<()> {
        // 配置 rustyline：使用 List 类型补全，显示所有候选项
        let config = Config::builder()
            .completion_type(CompletionType::List) // 按 Tab 显示完整列表
            .completion_prompt_limit(20) // 超过 20 个候选项时询问是否显示
            .build();

        let mut rl = Editor::with_config(config)?;
        rl.set_helper(Some(OxideHelper::default()));

        // 绑定触发符按键处理器
        // 当在空行输入 /、@、# 时，立即插入字符（用户需要按回车确认）
        rl.bind_sequence(
            KeyEvent(KeyCode::Char('/'), Modifiers::NONE),
            EventHandler::Conditional(Box::new(TriggerHandler::new('/'))),
        );
        rl.bind_sequence(
            KeyEvent(KeyCode::Char('@'), Modifiers::NONE),
            EventHandler::Conditional(Box::new(TriggerHandler::new('@'))),
        );
        rl.bind_sequence(
            KeyEvent(KeyCode::Char('#'), Modifiers::NONE),
            EventHandler::Conditional(Box::new(TriggerHandler::new('#'))),
        );

        loop {
            self.print_separator()?;
            let readline = rl.readline("❯ ");

            match readline {
                Ok(line) => {
                    let input = line.trim();
                    if input.is_empty() {
                        continue;
                    }

                    // 检测触发符：/、@、#
                    let first_char = input.chars().next().unwrap_or(' ');
                    let trigger = TriggerType::from_char(first_char);

                    let final_input = if let Some(trigger_type) = trigger {
                        // 如果只输入了单个触发符，弹出选择器
                        if input.len() == 1 {
                            match self.show_trigger_selector(trigger_type) {
                                Ok(selected) => {
                                    println!("\n{} {}", "✓".green(), selected.bright_green());
                                    selected
                                }
                                Err(e) => {
                                    // 用户取消选择（如按 Esc）
                                    println!(
                                        "\n{} {}",
                                        "⚠️".yellow(),
                                        format!("选择取消: {}", e)
                                    );
                                    continue;
                                }
                            }
                        } else {
                            // 已经输入了完整内容，直接使用
                            input.to_string()
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
