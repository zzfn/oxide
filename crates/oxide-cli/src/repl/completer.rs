//! 自动补全
//!
//! 支持三种触发符补全：
//! - `/` - 命令补全
//! - `@` - 文件路径补全
//! - `#` - 标签补全

use reedline::{Completer, Span, Suggestion};
use std::path::PathBuf;
use std::sync::Arc;

use crate::commands::CommandRegistry;

/// Oxide 自动补全器
pub struct OxideCompleter {
    /// 命令注册表
    commands: Arc<CommandRegistry>,
    /// 工作目录
    working_dir: PathBuf,
    /// 标签列表
    tags: Vec<String>,
}

impl OxideCompleter {
    /// 创建新的补全器
    pub fn new(commands: Arc<CommandRegistry>, working_dir: PathBuf) -> Self {
        Self {
            commands,
            working_dir,
            tags: vec![
                "bug".to_string(),
                "feature".to_string(),
                "refactor".to_string(),
                "docs".to_string(),
                "test".to_string(),
            ],
        }
    }

    /// 设置工作目录
    pub fn set_working_dir(&mut self, dir: PathBuf) {
        self.working_dir = dir;
    }

    /// 添加标签
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    /// 命令补全
    fn complete_commands(&self, prefix: &str) -> Vec<Suggestion> {
        let prefix = prefix.trim_start_matches('/');
        self.commands
            .command_names()
            .into_iter()
            .filter(|name| name.starts_with(prefix))
            .map(|name| Suggestion {
                value: format!("/{}", name),
                description: self
                    .commands
                    .get(name)
                    .map(|cmd| cmd.description().to_string()),
                style: None,
                extra: None,
                span: Span::new(0, 0), // 将在 complete 中设置
                append_whitespace: true,
            })
            .collect()
    }

    /// 文件路径补全
    fn complete_files(&self, prefix: &str) -> Vec<Suggestion> {
        let prefix = prefix.trim_start_matches('@');
        let search_path = if prefix.is_empty() {
            self.working_dir.clone()
        } else if prefix.starts_with('/') {
            PathBuf::from(prefix)
        } else {
            self.working_dir.join(prefix)
        };

        // 获取目录和文件名前缀
        let (dir, file_prefix) = if search_path.is_dir() {
            (search_path, String::new())
        } else {
            let parent = search_path.parent().unwrap_or(&self.working_dir);
            let file_name = search_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            (parent.to_path_buf(), file_name.to_string())
        };

        // 读取目录内容
        let mut suggestions = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with(&file_prefix) && !name.starts_with('.') {
                    let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
                    let display = if is_dir {
                        format!("{}/", name)
                    } else {
                        name.clone()
                    };

                    let full_path = if prefix.is_empty() {
                        display.clone()
                    } else if prefix.ends_with('/') {
                        format!("{}{}", prefix, display)
                    } else {
                        let parent_prefix = prefix
                            .rfind('/')
                            .map(|i| &prefix[..=i])
                            .unwrap_or("");
                        format!("{}{}", parent_prefix, display)
                    };

                    suggestions.push(Suggestion {
                        value: format!("@{}", full_path),
                        description: Some(if is_dir {
                            "目录".to_string()
                        } else {
                            "文件".to_string()
                        }),
                        style: None,
                        extra: None,
                        span: Span::new(0, 0),
                        append_whitespace: !is_dir,
                    });
                }
            }
        }

        suggestions
    }

    /// 标签补全
    fn complete_tags(&self, prefix: &str) -> Vec<Suggestion> {
        let prefix = prefix.trim_start_matches('#');
        self.tags
            .iter()
            .filter(|tag| tag.starts_with(prefix))
            .map(|tag| Suggestion {
                value: format!("#{}", tag),
                description: Some("标签".to_string()),
                style: None,
                extra: None,
                span: Span::new(0, 0),
                append_whitespace: true,
            })
            .collect()
    }
}

impl Completer for OxideCompleter {
    fn complete(&mut self, line: &str, pos: usize) -> Vec<Suggestion> {
        // 找到当前正在输入的词
        let line_to_pos = &line[..pos];
        let word_start = line_to_pos
            .rfind(|c: char| c.is_whitespace())
            .map(|i| i + 1)
            .unwrap_or(0);
        let current_word = &line_to_pos[word_start..];

        let span = Span::new(word_start, pos);

        let mut suggestions = if current_word.starts_with('/') {
            self.complete_commands(current_word)
        } else if current_word.starts_with('@') {
            self.complete_files(current_word)
        } else if current_word.starts_with('#') {
            self.complete_tags(current_word)
        } else {
            Vec::new()
        };

        // 更新 span
        for suggestion in &mut suggestions {
            suggestion.span = span;
        }

        suggestions
    }
}
