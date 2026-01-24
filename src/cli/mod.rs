pub mod command;
pub mod render;

use anyhow::Result;
use colored::*;
use dialoguer::FuzzySelect;
use reedline::{
    default_emacs_keybindings, Completer, DefaultPrompt, DescriptionMode, EditCommand, Emacs,
    IdeMenu, KeyCode, KeyModifiers, MenuBuilder, Reedline, ReedlineEvent, ReedlineMenu, Signal,
    Span, Suggestion,
};
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

fn build_commands() -> HashMap<String, CommandInfo> {
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
    commands
}

fn build_command_entries() -> Vec<(String, String)> {
    let mut entries: Vec<(String, String)> = build_commands()
        .into_iter()
        .map(|(name, info)| (name, info.description))
        .collect();

    if let Ok(skill_manager) = crate::skill::SkillManager::new() {
        for skill in skill_manager.list_skills() {
            entries.push((format!("/{}", skill.name), skill.description));
        }
    }

    entries.sort_by(|a, b| a.0.cmp(&b.0));
    entries
}

fn build_context_entries() -> Vec<(String, String)> {
    vec![
        ("@file".to_string(), "引用文件".to_string()),
        ("@codebase".to_string(), "搜索代码库".to_string()),
        ("@web".to_string(), "搜索网页".to_string()),
        ("@docs".to_string(), "搜索文档".to_string()),
    ]
}

fn build_tag_entries() -> Vec<(String, String)> {
    vec![
        ("#bug".to_string(), "问题修复".to_string()),
        ("#feature".to_string(), "新功能".to_string()),
        ("#refactor".to_string(), "重构".to_string()),
        ("#docs".to_string(), "文档".to_string()),
    ]
}

fn token_start(line: &str, pos: usize) -> usize {
    let mut start = 0;
    for (idx, ch) in line[..pos].char_indices().rev() {
        if ch.is_whitespace() {
            start = idx + ch.len_utf8();
            break;
        }
    }
    start
}

fn token_end(line: &str, pos: usize) -> usize {
    let mut end = line.len();
    for (idx, ch) in line[pos..].char_indices() {
        if ch.is_whitespace() {
            end = pos + idx;
            break;
        }
    }
    end
}

fn is_line_start(line: &str, start: usize) -> bool {
    line[..start].trim().is_empty()
}

struct OxideCompleter;

impl OxideCompleter {
    fn match_entries(
        &self,
        entries: &[(String, String)],
        token: &str,
        span: Span,
    ) -> Vec<Suggestion> {
        let mut suggestions: Vec<Suggestion> = entries
            .iter()
            .filter(|(value, _)| value.starts_with(token))
            .map(|(value, description)| Suggestion {
                value: value.clone(),
                description: Some(description.clone()),
                style: None,
                extra: None,
                span,
                append_whitespace: false,
            })
            .collect();
        suggestions.sort_by(|a, b| a.value.cmp(&b.value));
        suggestions
    }
}

impl Completer for OxideCompleter {
    fn complete(&mut self, line: &str, pos: usize) -> Vec<Suggestion> {
        let start = token_start(line, pos);
        let token = &line[start..pos];
        let end = token_end(line, pos);
        let span = Span::new(start, end);

        if let Some(first_char) = token.chars().next() {
            match first_char {
                '/' => {
                    if is_line_start(line, start) {
                        return self.match_entries(&build_command_entries(), token, span);
                    }
                }
                '@' => {
                    return self.match_entries(&build_context_entries(), token, span);
                }
                '#' => {
                    if is_line_start(line, start) {
                        return self.match_entries(&build_tag_entries(), token, span);
                    }
                }
                _ => {}
            }
        }

        Vec::new()
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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    fn show_command_selector(&self) -> Result<String> {
        // 获取 OxideHelper 中的命令信息
        let commands = build_commands();

        // 准备命令列表（带描述）
        let mut command_items: Vec<String> = commands
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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    fn show_trigger_selector(&self, trigger: TriggerType) -> Result<String> {
        match trigger {
            TriggerType::Command => self.show_command_selector(),
            TriggerType::Context => self.show_context_selector(),
            TriggerType::Tag => self.show_tag_selector(),
        }
    }

    async fn run_input_loop(&mut self) -> Result<()> {
        let mut keybindings = default_emacs_keybindings();
        keybindings.add_binding(
            KeyModifiers::NONE,
            KeyCode::Char('/'),
            ReedlineEvent::Multiple(vec![
                ReedlineEvent::Edit(vec![EditCommand::InsertChar('/')]),
                ReedlineEvent::Menu("oxide_completion".to_string()),
            ]),
        );
        keybindings.add_binding(
            KeyModifiers::NONE,
            KeyCode::Char('@'),
            ReedlineEvent::Multiple(vec![
                ReedlineEvent::Edit(vec![EditCommand::InsertChar('@')]),
                ReedlineEvent::Menu("oxide_completion".to_string()),
            ]),
        );
        keybindings.add_binding(
            KeyModifiers::NONE,
            KeyCode::Char('#'),
            ReedlineEvent::Multiple(vec![
                ReedlineEvent::Edit(vec![EditCommand::InsertChar('#')]),
                ReedlineEvent::Menu("oxide_completion".to_string()),
            ]),
        );

        let edit_mode = Box::new(Emacs::new(keybindings));
        let completion_menu = IdeMenu::default()
            .with_name("oxide_completion")
            .with_default_border()
            .with_description_mode(DescriptionMode::PreferRight)
            .with_max_completion_height(8)
            .with_max_description_height(6)
            .with_max_description_width(48)
            .with_correct_cursor_pos(true);

        let mut rl = Reedline::create()
            .with_edit_mode(edit_mode)
            .with_completer(Box::new(OxideCompleter))
            .with_menu(ReedlineMenu::EngineCompleter(Box::new(completion_menu)));
        let prompt = DefaultPrompt::default();

        loop {
            self.print_separator()?;
            let readline = rl.read_line(&prompt);
            let final_input = match readline {
                Ok(Signal::Success(line)) => {
                    let input = line.trim().to_string();
                    if input.is_empty() {
                        continue;
                    }
                    input
                }
                Ok(Signal::CtrlC) => {
                    println!("{}", "^C".dimmed());
                    break;
                }
                Ok(Signal::CtrlD) => break,
                Err(err) => {
                    println!("{} {:?}", "Error:".red(), err);
                    break;
                }
            };

            self.print_separator()?;

            let should_continue = self.handle_command(&final_input).await?;
            if !should_continue {
                break;
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
