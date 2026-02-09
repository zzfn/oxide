//! 自动补全
//!
//! 支持三种触发符补全：
//! - `/` - 命令补全
//! - `@` - 文件路径补全
//! - `#` - 标签补全

use std::path::PathBuf;
use std::sync::Arc;

use crate::commands::CommandRegistry;

/// 补全建议
#[derive(Debug, Clone)]
pub struct Suggestion {
    pub value: String,
    pub description: Option<String>,
}

/// Oxide 自动补全器
pub struct OxideCompleter {
    commands: Arc<CommandRegistry>,
    working_dir: PathBuf,
    tags: Vec<String>,
}

impl OxideCompleter {
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

    pub fn set_working_dir(&mut self, dir: PathBuf) {
        self.working_dir = dir;
    }

    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    /// 根据当前输入和光标位置生成补全建议
    pub fn complete(&self, line: &str, pos: usize) -> Vec<Suggestion> {
        let line_to_pos = &line[..pos];
        let word_start = line_to_pos
            .rfind(|c: char| c.is_whitespace())
            .map(|i| i + 1)
            .unwrap_or(0);
        let current_word = &line_to_pos[word_start..];

        if current_word.starts_with('/') {
            self.complete_commands(current_word)
        } else if current_word.starts_with('@') {
            self.complete_files(current_word)
        } else if current_word.starts_with('#') {
            self.complete_tags(current_word)
        } else {
            Vec::new()
        }
    }

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
            })
            .collect()
    }

    fn complete_files(&self, prefix: &str) -> Vec<Suggestion> {
        let prefix = prefix.trim_start_matches('@');
        let search_path = if prefix.is_empty() {
            self.working_dir.clone()
        } else if prefix.starts_with('/') {
            PathBuf::from(prefix)
        } else {
            self.working_dir.join(prefix)
        };

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
                    });
                }
            }
        }

        suggestions
    }

    fn complete_tags(&self, prefix: &str) -> Vec<Suggestion> {
        let prefix = prefix.trim_start_matches('#');
        self.tags
            .iter()
            .filter(|tag| tag.starts_with(prefix))
            .map(|tag| Suggestion {
                value: format!("#{}", tag),
                description: Some("标签".to_string()),
            })
            .collect()
    }
}
