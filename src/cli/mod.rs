pub mod command;
pub mod file_resolver;
pub mod render;
pub mod statusbar;

use anyhow::Result;
use colored::*;
use inquire::Select;
use crossterm::{
    cursor::{self, MoveToColumn},
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    queue,
    style::{Color as CtColor, Print, ResetColor, SetForegroundColor, SetBackgroundColor},
    terminal::{self, Clear, ClearType},
};
use std::collections::HashMap;
use std::io::{stdout, Write, IsTerminal};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::context::ContextManager;

const PROMPT_CYCLE_COMMAND: &str = "__oxide_prompt_cycle__";

// ============================================================================
// crossterm 行编辑器实现
// ============================================================================

/// 输入信号类型
enum Signal {
    /// 成功提交输入
    Success(String),
    /// Ctrl+C 中断
    CtrlC,
    /// Ctrl+D 退出
    CtrlD,
}

/// 文本范围
#[derive(Clone, Copy)]
struct Span {
    start: usize,
    end: usize,
}

impl Span {
    fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

/// 补全建议
struct Suggestion {
    value: String,
    description: Option<String>,
    span: Span,
}

/// 行编辑器
struct LineEditor {
    buffer: String,
    /// 光标位置（字符索引，不是字节索引）
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

    /// 获取光标位置对应的字节索引
    fn cursor_byte_pos(&self) -> usize {
        self.buffer
            .char_indices()
            .nth(self.cursor_pos)
            .map(|(i, _)| i)
            .unwrap_or(self.buffer.len())
    }

    /// 获取字符数量
    fn char_count(&self) -> usize {
        self.buffer.chars().count()
    }

    fn insert_char(&mut self, c: char) {
        let byte_pos = self.cursor_byte_pos();
        self.buffer.insert(byte_pos, c);
        self.cursor_pos += 1;
    }

    fn delete_char(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
            let byte_pos = self.cursor_byte_pos();
            self.buffer.remove(byte_pos);
        }
    }

    fn delete_char_forward(&mut self) {
        if self.cursor_pos < self.char_count() {
            let byte_pos = self.cursor_byte_pos();
            self.buffer.remove(byte_pos);
        }
    }

    fn move_left(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
        }
    }

    fn move_right(&mut self) {
        if self.cursor_pos < self.char_count() {
            self.cursor_pos += 1;
        }
    }

    fn move_home(&mut self) {
        self.cursor_pos = 0;
    }

    fn move_end(&mut self) {
        self.cursor_pos = self.char_count();
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
            self.cursor_pos = self.char_count();
        }
    }

    fn history_down(&mut self) {
        match self.history_index {
            Some(i) if i < self.history.len() - 1 => {
                self.history_index = Some(i + 1);
                self.buffer = self.history[i + 1].clone();
                self.cursor_pos = self.char_count();
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

    /// 应用补全建议
    fn apply_suggestion(&mut self, suggestion: &Suggestion) {
        // span 使用的是字节索引，需要转换为字符索引
        let before = &self.buffer[..suggestion.span.start];
        let after = &self.buffer[suggestion.span.end..];
        let before_char_count = before.chars().count();
        let value_char_count = suggestion.value.chars().count();
        self.buffer = format!("{}{}{}", before, suggestion.value, after);
        self.cursor_pos = before_char_count + value_char_count;
    }
}

/// 补全菜单
struct CompletionMenu {
    suggestions: Vec<Suggestion>,
    selected_index: usize,
    visible: bool,
}

impl CompletionMenu {
    const MAX_VISIBLE: usize = 8;

    fn new() -> Self {
        Self {
            suggestions: Vec::new(),
            selected_index: 0,
            visible: false,
        }
    }

    fn update(&mut self, suggestions: Vec<Suggestion>) {
        self.suggestions = suggestions;
        self.selected_index = 0;
        self.visible = !self.suggestions.is_empty();
    }

    fn select_next(&mut self) {
        if !self.suggestions.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.suggestions.len();
        }
    }

    fn select_previous(&mut self) {
        if !self.suggestions.is_empty() {
            self.selected_index = if self.selected_index == 0 {
                self.suggestions.len() - 1
            } else {
                self.selected_index - 1
            };
        }
    }

    fn get_selected(&self) -> Option<&Suggestion> {
        if self.visible && !self.suggestions.is_empty() {
            Some(&self.suggestions[self.selected_index])
        } else {
            None
        }
    }

    fn hide(&mut self) {
        self.visible = false;
        self.suggestions.clear();
        self.selected_index = 0;
    }

    fn render(&self, prompt_len: u16) -> anyhow::Result<()> {
        if !self.visible || self.suggestions.is_empty() {
            return Ok(());
        }

        let mut stdout = stdout();

        // 计算显示范围
        let total = self.suggestions.len();
        let visible_count = total.min(Self::MAX_VISIBLE);
        let scroll_offset = if self.selected_index >= visible_count {
            self.selected_index - visible_count + 1
        } else {
            0
        };

        // 保存光标位置
        queue!(stdout, cursor::SavePosition)?;

        // 渲染每个建议项
        for i in 0..visible_count {
            let idx = scroll_offset + i;
            if idx >= total {
                break;
            }

            let suggestion = &self.suggestions[idx];
            let is_selected = idx == self.selected_index;

            // 移动到下一行
            queue!(stdout, Print("\r\n"), MoveToColumn(prompt_len))?;

            if is_selected {
                // 选中项：青色背景
                queue!(
                    stdout,
                    SetBackgroundColor(CtColor::Cyan),
                    SetForegroundColor(CtColor::Black),
                )?;
            } else {
                queue!(stdout, SetForegroundColor(CtColor::White))?;
            }

            // 打印建议值
            let display_value = if suggestion.value.len() > 30 {
                format!("{}...", &suggestion.value[..27])
            } else {
                suggestion.value.clone()
            };
            queue!(stdout, Print(&display_value))?;

            // 打印描述
            if let Some(ref desc) = suggestion.description {
                queue!(stdout, ResetColor, SetForegroundColor(CtColor::DarkGrey))?;
                let desc_display = if desc.len() > 30 {
                    format!(" - {}...", &desc[..27])
                } else {
                    format!(" - {}", desc)
                };
                queue!(stdout, Print(&desc_display))?;
            }

            queue!(stdout, ResetColor, Clear(ClearType::UntilNewLine))?;
        }

        // 恢复光标位置
        queue!(stdout, cursor::RestorePosition)?;
        stdout.flush()?;

        Ok(())
    }

    fn clear_display(&self, prompt_len: u16) -> anyhow::Result<()> {
        if self.suggestions.is_empty() {
            return Ok(());
        }

        let mut stdout = stdout();
        let visible_count = self.suggestions.len().min(Self::MAX_VISIBLE);

        queue!(stdout, cursor::SavePosition)?;

        for _ in 0..visible_count {
            queue!(
                stdout,
                Print("\r\n"),
                MoveToColumn(prompt_len),
                Clear(ClearType::UntilNewLine)
            )?;
        }

        queue!(stdout, cursor::RestorePosition)?;
        stdout.flush()?;

        Ok(())
    }
}

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
        CommandInfo::new("/agent [list|capabilities]", "查看 Agent 类型与能力"),
    );
    commands.insert(
        "/tasks".to_string(),
        CommandInfo::new("/tasks [list|show <id>]", "管理后台任务"),
    );
    commands.insert(
        "/skills".to_string(),
        CommandInfo::new("/skills [list|show <name>]", "管理技能"),
    );
    commands.insert(
        "/workflow".to_string(),
        CommandInfo::new("/workflow [status|on|off]", "PAOR 工作流设置"),
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
#[allow(dead_code)]
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
        // 移除 @ 符号用于模糊匹配
        let search_token = token.strip_prefix('@').unwrap_or(token);

        let mut suggestions: Vec<Suggestion> = entries
            .iter()
            .filter(|(value, _)| {
                // 检查完整路径是否以 token 开头（精确匹配）
                if value.starts_with(token) {
                    return true;
                }

                // 如果不是精确匹配，尝试模糊匹配文件名部分
                // 例如：@mod 应该匹配 @src/cli/mod.rs
                let value_path = value.strip_prefix('@').unwrap_or(value);
                let value_name = value_path
                    .split('/')
                    .last()
                    .unwrap_or(value_path);

                // 不区分大小写模糊匹配文件名
                value_name.to_lowercase().contains(&search_token.to_lowercase())
            })
            .map(|(value, description)| Suggestion {
                value: value.clone(),
                description: Some(description.clone()),
                span,
            })
            .collect();
        suggestions.sort_by(|a, b| a.value.cmp(&b.value));
        suggestions
    }

    /// 递归列出目录下的所有文件
    ///
    /// # 参数
    /// - `base_dir`: 基础目录
    ///
    /// # 返回
    /// - 目录下所有文件的路径列表
    #[allow(dead_code)]
    fn list_files_recursive(base_dir: &Path) -> Vec<PathBuf> {
        use std::fs;

        let mut files = Vec::new();

        // 需要忽略的目录
        let ignored_dirs = [
            ".git",
            "node_modules",
            "target",
            "dist",
            "build",
            ".venv",
            "venv",
            "__pycache__",
            ".pytest_cache",
            "vendor",
            ".cache",
        ];

        if let Ok(read_dir) = fs::read_dir(base_dir) {
            for entry in read_dir.filter_map(|e| e.ok()) {
                let path = entry.path();
                let file_name = entry.file_name();

                // 跳过隐藏文件和目录
                if file_name.to_string_lossy().starts_with('.') {
                    continue;
                }

                // 跳过忽略的目录
                if path.is_dir() {
                    let dir_name = file_name.to_string_lossy();
                    if ignored_dirs.iter().any(|&ignored| ignored == dir_name) {
                        continue;
                    }

                    // 递归扫描子目录
                    files.extend(Self::list_files_recursive(&path));
                } else if path.is_file() {
                    files.push(path);
                }
            }
        }

        files
    }

    /// 构建文件路径补全项
    fn build_file_entries(&self, path_str: &str) -> std::io::Result<Vec<(String, String)>> {
        use std::fs;

        let mut entries = Vec::new();

        // 解析路径：判断是否包含目录分隔符
        let has_directory_separator = path_str.contains('/') || path_str.contains('\\');

        if has_directory_separator {
            // 包含目录：扫描指定目录
            let path = PathBuf::from(path_str);
            let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            let full_path = current_dir.join(&path);

            let (scan_dir, file_prefix) = if full_path.exists() && full_path.is_dir() {
                (full_path, String::new())
            } else {
                // 尝试分离目录和文件部分
                if let Some(parent) = path.parent() {
                    let parent_path = if parent.as_os_str().is_empty() {
                        current_dir.clone()
                    } else {
                        current_dir.join(parent)
                    };
                    (parent_path, path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default())
                } else {
                    (current_dir, String::new())
                }
            };

            // 扫描指定目录
            if let Ok(read_dir) = fs::read_dir(&scan_dir) {
                for entry in read_dir.filter_map(|e| e.ok()) {
                    let file_name = entry.file_name();
                    let name = file_name.to_string_lossy().to_string();

                    if name.starts_with('.') {
                        continue;
                    }

                    // 应用文件名过滤
                    if !file_prefix.is_empty() && !name.to_lowercase().contains(&file_prefix.to_lowercase()) {
                        continue;
                    }

                    let file_type = entry.file_type();
                    let display_path = if let Some(parent) = path.parent() {
                        if parent.as_os_str().is_empty() {
                            format!("@{}", name)
                        } else {
                            format!("@{}/{}", parent.display(), name)
                        }
                    } else {
                        format!("@{}", name)
                    };

                    let description = if file_type.as_ref().map_or(false, |ft| ft.is_dir()) {
                        "目录/".to_string()
                    } else if file_type.as_ref().map_or(false, |ft| ft.is_file()) {
                        if let Ok(metadata) = entry.metadata() {
                            format_file_size(metadata.len())
                        } else {
                            "文件".to_string()
                        }
                    } else {
                        "其他".to_string()
                    };

                    entries.push((display_path, description));
                }
            }
        } else {
            // 不包含目录：递归扫描当前目录下的所有文件
            let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

            if path_str.is_empty() {
                // 输入为空：只显示当前目录的直接子项
                if let Ok(read_dir) = fs::read_dir(&current_dir) {
                    for entry in read_dir.filter_map(|e| e.ok()) {
                        let file_name = entry.file_name();
                        let name = file_name.to_string_lossy().to_string();

                        if name.starts_with('.') {
                            continue;
                        }

                        let file_type = entry.file_type();
                        let display_path = format!("@{}", name);

                        let description = if file_type.as_ref().map_or(false, |ft| ft.is_dir()) {
                            "目录/".to_string()
                        } else if file_type.as_ref().map_or(false, |ft| ft.is_file()) {
                            if let Ok(metadata) = entry.metadata() {
                                format_file_size(metadata.len())
                            } else {
                                "文件".to_string()
                            }
                        } else {
                            "其他".to_string()
                        };

                        entries.push((display_path, description));
                    }
                }
            } else {
                // 输入不为空：递归扫描所有文件进行模糊匹配
                let all_files = Self::list_files_recursive(&current_dir);

                for file_path in all_files {
                    let file_name = file_path.file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();

                    if file_name.starts_with('.') {
                        continue;
                    }

                    // 模糊匹配文件名
                    if !file_name.to_lowercase().contains(&path_str.to_lowercase()) {
                        continue;
                    }

                    // 获取相对路径
                    let relative_path = file_path.strip_prefix(&current_dir)
                        .unwrap_or(&file_path);
                    let display_path = format!("@{}", relative_path.display());

                    // 获取文件大小
                    let description = if let Ok(metadata) = fs::metadata(&file_path) {
                        format_file_size(metadata.len())
                    } else {
                        "文件".to_string()
                    };

                    entries.push((display_path, description));
                }

                // 限制结果数量，避免太多
                if entries.len() > 50 {
                    entries.truncate(50);
                }
            }
        }

        // 排序：目录优先，然后按名称
        entries.sort_by(|a, b| {
            let a_is_dir = a.1.ends_with('/');
            let b_is_dir = b.1.ends_with('/');
            if a_is_dir && !b_is_dir {
                std::cmp::Ordering::Less
            } else if !a_is_dir && b_is_dir {
                std::cmp::Ordering::Greater
            } else {
                a.0.cmp(&b.0)
            }
        });

        Ok(entries)
    }
}

/// 格式化文件大小
fn format_file_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

impl OxideCompleter {
    fn complete(&self, line: &str, pos: usize) -> Vec<Suggestion> {
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
                    // 动态生成文件路径补全
                    let path_str = &token[1..]; // 移除 @ 符号
                    if let Ok(file_entries) = self.build_file_entries(path_str) {
                        return self.match_entries(&file_entries, token, span);
                    }
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

/// 自定义 Prompt
#[derive(Clone)]
struct OxidePrompt {
    /// 左侧提示符标签
    label: PromptLabel,
}

impl OxidePrompt {
    fn new(label: PromptLabel) -> Self {
        Self { label }
    }

    fn render(&self) -> String {
        format!("{}> ", self.label.as_str())
    }

    fn len(&self) -> u16 {
        (self.label.as_str().len() + 2) as u16
    }
}

/// 左侧提示符标签
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PromptLabel {
    Oxide,
    Fast,
    Plan,
}

impl PromptLabel {
    fn as_str(self) -> &'static str {
        match self {
            PromptLabel::Oxide => "oxide",
            PromptLabel::Fast => "fast",
            PromptLabel::Plan => "plan",
        }
    }

    fn next(self) -> Self {
        match self {
            PromptLabel::Oxide => PromptLabel::Fast,
            PromptLabel::Fast => PromptLabel::Plan,
            PromptLabel::Plan => PromptLabel::Oxide,
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

use crate::agent::HitlIntegration;
use crate::agent::AgentType;
use crate::agent::SubagentManager;
use crate::agent::workflow::ComplexityEvaluator;
use crate::cli::render::Spinner;
use crate::cli::statusbar::StatusBar;
use crate::config::secret::Secret;

pub struct OxideCli {
    pub api_key: Secret<String>,
    pub model_name: String,
    pub agent: AgentType,
    pub context_manager: ContextManager,
    pub _hitl: Arc<HitlIntegration>,
    prompt_label: PromptLabel,
    spinner: Spinner,
    total_tokens: Arc<AtomicU64>,
    /// 子 agent 管理器（用于工作流）
    subagent_manager: Arc<SubagentManager>,
    /// 复杂度评估器
    complexity_evaluator: ComplexityEvaluator,
    /// 状态栏
    statusbar: Option<StatusBar>,
}

// 手动实现 Debug，防止 api_key 泄露
impl std::fmt::Debug for OxideCli {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OxideCli")
            .field("api_key", &self.api_key) // Secret 的 Debug 实现会输出 "***"
            .field("model_name", &self.model_name)
            .field("agent", &self.agent)
            .field("context_manager", &self.context_manager)
            .finish()
    }
}

impl OxideCli {
    pub fn new(
        api_key: Secret<String>,
        model_name: String,
        agent: AgentType,
        context_manager: ContextManager,
        hitl: Arc<HitlIntegration>,
    ) -> Self {
        let total_tokens = Arc::new(AtomicU64::new(0));
        let session_id = context_manager.session_id().to_string();
        let statusbar = StatusBar::new(
            total_tokens.clone(),
            session_id,
            model_name.clone(),
        );

        Self {
            api_key,
            model_name,
            agent,
            context_manager,
            _hitl: hitl,
            prompt_label: PromptLabel::Oxide,
            spinner: Spinner::new(),
            total_tokens,
            subagent_manager: Arc::new(SubagentManager::new()),
            complexity_evaluator: ComplexityEvaluator::new(),
            statusbar: Some(statusbar),
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        // 确保终端处于正常模式并重置
        let _ = terminal::disable_raw_mode();
        // 重置终端属性
        print!("\x1b[0m");
        let _ = stdout().flush();

        println!("{}", LOGO);
        self.show_welcome()?;
        self.show_tips()?;

        // 初始化状态栏（在欢迎信息之后）
        if let Some(ref mut statusbar) = self.statusbar {
            statusbar.init()?;
            statusbar.start_refresh();
        }

        let result = self.run_input_loop().await;

        // 清理状态栏
        if let Some(ref mut statusbar) = self.statusbar {
            statusbar.cleanup()?;
        }

        // 确保退出时终端恢复正常
        let _ = terminal::disable_raw_mode();

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

        let selection = Select::new("选择命令 (输入过滤)", command_items).prompt()?;

        // 提取命令名称（去除描述部分）
        let selected = selection
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

        let selection = Select::new("选择上下文 (输入过滤)", context_items).prompt()?;

        let selected = selection
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

        let selection = Select::new("选择标签 (输入过滤)", tag_items).prompt()?;

        let selected = selection
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
        // 检查是否是 TTY 终端
        if !std::io::stdin().is_terminal() {
            return Err(anyhow::anyhow!("Oxide CLI 需要在交互式终端中运行"));
        }

        let mut editor = LineEditor::new();
        let completer = OxideCompleter;
        let mut menu = CompletionMenu::new();
        let mut last_ctrl_c: Option<Instant> = None;
        let mut skip_separator = false;

        loop {
            let prompt = OxidePrompt::new(self.prompt_label);
            let prompt_len = prompt.len();

            if skip_separator {
                skip_separator = false;
            } else {
                self.print_separator()?;
            }

            // 启用 raw mode 进行输入
            terminal::enable_raw_mode()?;

            // 渲染提示符和当前输入
            self.render_input(&editor, &prompt)?;

            // 事件循环
            let signal = loop {
                if event::poll(Duration::from_millis(50))? {
                    if let Event::Key(KeyEvent { code, modifiers, .. }) = event::read()? {
                        match self.handle_key(&mut editor, code, modifiers, &completer, &mut menu, prompt_len)? {
                            Some(signal) => {
                                // 清除补全菜单
                                menu.clear_display(prompt_len)?;
                                menu.hide();
                                break signal;
                            }
                            None => {
                                // 继续渲染
                                self.render_input(&editor, &prompt)?;
                                menu.render(prompt_len)?;
                            }
                        }
                    }
                }
            };

            // 退出 raw mode
            terminal::disable_raw_mode()?;
            println!(); // 换行

            match signal {
                Signal::Success(line) => {
                    if line == PROMPT_CYCLE_COMMAND {
                        let old_label = self.prompt_label;
                        self.prompt_label = self.prompt_label.next();
                        self.show_mode_switch_hint(old_label, self.prompt_label);
                        skip_separator = true;
                        continue;
                    }

                    let input = line.trim().to_string();
                    if input.is_empty() {
                        continue;
                    }

                    last_ctrl_c = None;
                    self.print_separator()?;

                    let should_continue = self.handle_command(&input).await?;
                    if !should_continue {
                        break;
                    }
                }
                Signal::CtrlC => {
                    let now = Instant::now();
                    let should_exit = last_ctrl_c
                        .map(|prev| now.duration_since(prev) <= Duration::from_secs(1))
                        .unwrap_or(false);
                    println!("{}", "^C".dimmed());
                    if should_exit {
                        break;
                    }
                    last_ctrl_c = Some(now);
                    editor.clear();
                }
                Signal::CtrlD => {
                    break;
                }
            }
        }

        Ok(())
    }

    fn render_input(&self, editor: &LineEditor, prompt: &OxidePrompt) -> Result<()> {
        let mut stdout = stdout();
        let prompt_str = prompt.render();
        let prompt_len = prompt.len();

        // 计算光标前文本的显示宽度（中文字符占 2 个宽度）
        let cursor_display_pos: u16 = editor.buffer
            .chars()
            .take(editor.cursor_pos)
            .map(|c| if c.is_ascii() { 1 } else { 2 })
            .sum();

        queue!(
            stdout,
            MoveToColumn(0),
            Clear(ClearType::CurrentLine),
            SetForegroundColor(CtColor::Green),
            Print(&prompt_str),
            ResetColor,
            Print(&editor.buffer),
            MoveToColumn(prompt_len + cursor_display_pos)
        )?;
        stdout.flush()?;
        Ok(())
    }

    fn handle_key(
        &self,
        editor: &mut LineEditor,
        code: KeyCode,
        modifiers: KeyModifiers,
        completer: &OxideCompleter,
        menu: &mut CompletionMenu,
        prompt_len: u16,
    ) -> Result<Option<Signal>> {
        match (code, modifiers) {
            // 退出
            (KeyCode::Char('d'), KeyModifiers::CONTROL) => {
                return Ok(Some(Signal::CtrlD));
            }
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                return Ok(Some(Signal::CtrlC));
            }

            // 提交输入
            (KeyCode::Enter, _) => {
                // 如果菜单可见且有选中项，应用补全
                if menu.visible {
                    if let Some(suggestion) = menu.get_selected() {
                        editor.apply_suggestion(suggestion);
                        menu.clear_display(prompt_len)?;
                        menu.hide();
                        return Ok(None);
                    }
                }
                let input = editor.submit();
                return Ok(Some(Signal::Success(input)));
            }

            // Tab 切换模式
            (KeyCode::Tab, KeyModifiers::NONE)
            | (KeyCode::Tab, KeyModifiers::SHIFT)
            | (KeyCode::BackTab, _) => {
                // 如果菜单可见，Tab 应用选中项
                if menu.visible {
                    if let Some(suggestion) = menu.get_selected() {
                        editor.apply_suggestion(suggestion);
                        menu.clear_display(prompt_len)?;
                        menu.hide();
                        return Ok(None);
                    }
                }
                // 否则切换模式
                editor.clear();
                return Ok(Some(Signal::Success(PROMPT_CYCLE_COMMAND.to_string())));
            }

            // 菜单导航
            (KeyCode::Down, _) if menu.visible => {
                menu.select_next();
            }
            (KeyCode::Up, _) if menu.visible => {
                menu.select_previous();
            }

            // Escape 关闭菜单
            (KeyCode::Esc, _) => {
                if menu.visible {
                    menu.clear_display(prompt_len)?;
                    menu.hide();
                }
            }

            // 编辑操作
            (KeyCode::Backspace, _) => {
                editor.delete_char();
                self.update_completions(editor, completer, menu);
            }
            (KeyCode::Delete, _) => {
                editor.delete_char_forward();
                self.update_completions(editor, completer, menu);
            }
            (KeyCode::Left, _) => {
                if menu.visible {
                    menu.clear_display(prompt_len)?;
                    menu.hide();
                }
                editor.move_left();
            }
            (KeyCode::Right, _) => {
                if menu.visible {
                    menu.clear_display(prompt_len)?;
                    menu.hide();
                }
                editor.move_right();
            }
            (KeyCode::Home, _) | (KeyCode::Char('a'), KeyModifiers::CONTROL) => {
                if menu.visible {
                    menu.clear_display(prompt_len)?;
                    menu.hide();
                }
                editor.move_home();
            }
            (KeyCode::End, _) | (KeyCode::Char('e'), KeyModifiers::CONTROL) => {
                if menu.visible {
                    menu.clear_display(prompt_len)?;
                    menu.hide();
                }
                editor.move_end();
            }
            (KeyCode::Up, _) => {
                editor.history_up();
            }
            (KeyCode::Down, _) => {
                editor.history_down();
            }

            // 清除当前行
            (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
                if menu.visible {
                    menu.clear_display(prompt_len)?;
                    menu.hide();
                }
                editor.clear();
            }

            // 普通字符输入
            (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                // 先清除旧菜单
                if menu.visible {
                    menu.clear_display(prompt_len)?;
                }

                editor.insert_char(c);

                // 触发补全
                if c == '/' || c == '@' || c == '#' {
                    self.update_completions(editor, completer, menu);
                } else if menu.visible {
                    // 继续过滤补全
                    self.update_completions(editor, completer, menu);
                }
            }

            _ => {}
        }

        Ok(None)
    }

    fn update_completions(
        &self,
        editor: &LineEditor,
        completer: &OxideCompleter,
        menu: &mut CompletionMenu,
    ) {
        // complete 函数需要字节索引
        let suggestions = completer.complete(&editor.buffer, editor.cursor_byte_pos());
        menu.update(suggestions);
    }

    pub fn print_separator(&self) -> Result<()> {
        let width = crossterm::terminal::size()
            .map(|(width, _)| width as usize)
            .unwrap_or(80)
            .max(1);
        let separator = "-".repeat(width);
        println!("{}", separator.dimmed());
        Ok(())
    }

    #[allow(dead_code)]
    pub fn session_id(&self) -> &str {
        self.context_manager.session_id()
    }

    fn reset_session_tokens(&self) {
        self.total_tokens.store(0, Ordering::Relaxed);
    }

    fn add_session_tokens(&self, tokens: u64) {
        self.total_tokens.fetch_add(tokens, Ordering::Relaxed);
    }

    /// 显示模式切换提示
    fn show_mode_switch_hint(&self, _old_label: PromptLabel, new_label: PromptLabel) {
        use colored::Colorize;

        match new_label {
            PromptLabel::Plan => {
                println!();
                println!("{}", "📋 已切换到 Plan 模式".bright_cyan().bold());
                println!("{}", "   所有任务将使用 PAOR 工作流处理".bright_white());
                println!("{}", "   Planning → Acting → Observing → Reflecting".dimmed());
                println!();
            }
            PromptLabel::Fast => {
                println!();
                println!("{}", "⚡ 已切换到 Fast 模式".bright_yellow().bold());
                println!("{}", "   使用快速响应模式".bright_white());
                println!();
            }
            PromptLabel::Oxide => {
                println!();
                println!("{}", "🦀 已切换到 Oxide 模式".bright_green().bold());
                println!("{}", "   使用标准对话模式".bright_white());
                println!();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_list_files_recursive() {
        // 创建临时目录结构
        let temp_dir = TempDir::new().unwrap();
        let base = temp_dir.path();

        // 创建测试文件和目录
        fs::create_dir_all(base.join("src")).unwrap();
        fs::create_dir_all(base.join("tests")).unwrap();
        fs::create_dir_all(base.join("target")).unwrap(); // 应该被忽略
        fs::create_dir_all(base.join(".git")).unwrap(); // 应该被忽略

        File::create(base.join("Cargo.toml")).unwrap();
        File::create(base.join("README.md")).unwrap();
        File::create(base.join("src/main.rs")).unwrap();
        File::create(base.join("src/lib.rs")).unwrap();
        File::create(base.join("tests/integration.rs")).unwrap();
        File::create(base.join("target/test")).unwrap(); // 应该被忽略
        File::create(base.join(".git/config")).unwrap(); // 应该被忽略

        // 测试递归扫描
        let files = OxideCompleter::list_files_recursive(base);

        // 验证：应该找到非忽略目录下的文件
        let file_names: Vec<_> = files
            .iter()
            .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
            .collect();

        assert!(file_names.contains(&"Cargo.toml".to_string()));
        assert!(file_names.contains(&"README.md".to_string()));
        assert!(file_names.contains(&"main.rs".to_string()));
        assert!(file_names.contains(&"lib.rs".to_string()));
        assert!(file_names.contains(&"integration.rs".to_string()));

        // 验证：不应该包含被忽略目录下的文件
        assert!(!file_names.contains(&"test".to_string())); // target/
        assert!(!file_names.contains(&"config".to_string())); // .git/
    }

    #[test]
    fn test_list_files_recursive_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let base = temp_dir.path();

        let files = OxideCompleter::list_files_recursive(base);
        assert!(files.is_empty());
    }

    #[test]
    fn test_list_files_recursive_nested_structure() {
        let temp_dir = TempDir::new().unwrap();
        let base = temp_dir.path();

        // 创建深层嵌套结构
        fs::create_dir_all(base.join("a/b/c/d")).unwrap();
        File::create(base.join("a/file1.rs")).unwrap();
        File::create(base.join("a/b/file2.rs")).unwrap();
        File::create(base.join("a/b/c/file3.rs")).unwrap();
        File::create(base.join("a/b/c/d/file4.rs")).unwrap();

        let files = OxideCompleter::list_files_recursive(base);

        // 应该找到所有嵌套文件
        assert_eq!(files.len(), 4);

        let file_names: Vec<_> = files
            .iter()
            .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
            .collect();

        assert!(file_names.contains(&"file1.rs".to_string()));
        assert!(file_names.contains(&"file2.rs".to_string()));
        assert!(file_names.contains(&"file3.rs".to_string()));
        assert!(file_names.contains(&"file4.rs".to_string()));
    }

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(0), "0 B");
        assert_eq!(format_file_size(512), "512 B");
        assert_eq!(format_file_size(1024), "1.0 KB");
        assert_eq!(format_file_size(1536), "1.5 KB");
        assert_eq!(format_file_size(1024 * 1024), "1.0 MB");
        assert_eq!(format_file_size(5 * 1024 * 1024), "5.0 MB");
    }
}
