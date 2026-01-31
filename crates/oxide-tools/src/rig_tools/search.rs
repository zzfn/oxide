//! 搜索工具的 rig Tool trait 适配: Glob, Grep

use glob::Pattern;
use grep_regex::RegexMatcherBuilder;
use grep_searcher::{sinks::UTF8, SearcherBuilder};
use ignore::WalkBuilder;
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use super::errors::SearchError;

// Glob 工具

/// Glob 工具参数
#[derive(Debug, Deserialize)]
pub struct GlobArgs {
    /// glob 模式，如 "**/*.js" 或 "src/**/*.ts"
    pub pattern: String,
    /// 搜索目录（可选）
    #[serde(default)]
    pub path: Option<String>,
}

/// Glob 工具输出
#[derive(Debug, Serialize)]
pub struct GlobOutput {
    /// 匹配的文件列表
    pub files: Vec<String>,
    /// 匹配数量
    pub count: usize,
}

/// Glob 工具 - 文件模式匹配
#[derive(Clone, Serialize, Deserialize)]
pub struct RigGlobTool {
    working_dir: PathBuf,
}

impl RigGlobTool {
    pub fn new(working_dir: PathBuf) -> Self {
        Self { working_dir }
    }

    fn resolve_path(&self, path: Option<&str>) -> PathBuf {
        match path {
            Some(p) => {
                let path = Path::new(p);
                if path.is_absolute() {
                    path.to_path_buf()
                } else {
                    self.working_dir.join(path)
                }
            }
            None => self.working_dir.clone(),
        }
    }
}

impl Tool for RigGlobTool {
    const NAME: &'static str = "Glob";

    type Error = SearchError;
    type Args = GlobArgs;
    type Output = GlobOutput;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "快速文件模式匹配工具，支持 glob 模式（如 \"**/*.js\" 或 \"src/**/*.ts\"）。返回按修改时间排序的匹配文件路径。".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "pattern": {
                        "type": "string",
                        "description": "glob 模式，如 \"**/*.js\" 或 \"src/**/*.ts\""
                    },
                    "path": {
                        "type": "string",
                        "description": "搜索目录。如果不指定，使用当前工作目录。"
                    }
                },
                "required": ["pattern"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let search_path = self.resolve_path(args.path.as_deref());

        if !search_path.exists() {
            return Err(SearchError::PathNotFound(search_path.display().to_string()));
        }

        if !search_path.is_dir() {
            return Err(SearchError::NotDirectory(search_path.display().to_string()));
        }

        // 使用 ignore crate 遍历文件，自动遵循 .gitignore
        let mut matches: Vec<(PathBuf, SystemTime)> = Vec::new();
        let walker = WalkBuilder::new(&search_path)
            .hidden(false)
            .git_ignore(true)
            .build();

        let pattern = Pattern::new(&args.pattern)?;

        for entry in walker.flatten() {
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            if let Ok(relative_path) = path.strip_prefix(&search_path) {
                if pattern.matches_path(relative_path) {
                    if let Ok(metadata) = fs::metadata(path) {
                        if let Ok(modified) = metadata.modified() {
                            matches.push((path.to_path_buf(), modified));
                        }
                    }
                }
            }
        }

        // 按修改时间排序（最新的在前）
        matches.sort_by(|a, b| b.1.cmp(&a.1));

        let files: Vec<String> = matches
            .into_iter()
            .map(|(path, _)| path.display().to_string())
            .collect();

        let count = files.len();
        Ok(GlobOutput { files, count })
    }
}

// Grep 工具

/// Grep 工具参数
#[derive(Debug, Deserialize)]
pub struct GrepArgs {
    /// 要搜索的正则表达式模式
    pub pattern: String,
    /// 搜索路径（文件或目录）
    #[serde(default)]
    pub path: Option<String>,
    /// 文件过滤 glob 模式
    #[serde(default)]
    pub glob: Option<String>,
    /// 文件类型过滤
    #[serde(default)]
    pub r#type: Option<String>,
    /// 输出模式
    #[serde(default)]
    pub output_mode: Option<String>,
    /// 上下文行数
    #[serde(default)]
    pub context: Option<usize>,
    /// 匹配行后的行数
    #[serde(default, rename = "-A")]
    pub after: Option<usize>,
    /// 匹配行前的行数
    #[serde(default, rename = "-B")]
    pub before: Option<usize>,
    /// context 的别名
    #[serde(default, rename = "-C")]
    pub context_alias: Option<usize>,
    /// 忽略大小写
    #[serde(default, rename = "-i")]
    pub case_insensitive: bool,
    /// 显示行号
    #[serde(default, rename = "-n")]
    pub line_numbers: Option<bool>,
    /// 限制输出行数
    #[serde(default)]
    pub head_limit: Option<usize>,
    /// 跳过前 N 行
    #[serde(default)]
    pub offset: Option<usize>,
    /// 多行模式
    #[serde(default)]
    pub multiline: bool,
}

/// Grep 工具输出
#[derive(Debug, Serialize)]
pub struct GrepOutput {
    /// 搜索结果
    pub results: Vec<String>,
    /// 匹配数量
    pub count: usize,
}

/// Grep 工具 - 代码搜索
#[derive(Clone, Serialize, Deserialize)]
pub struct RigGrepTool {
    working_dir: PathBuf,
}

impl RigGrepTool {
    pub fn new(working_dir: PathBuf) -> Self {
        Self { working_dir }
    }

    fn resolve_path(&self, path: Option<&str>) -> PathBuf {
        match path {
            Some(p) => {
                let path = Path::new(p);
                if path.is_absolute() {
                    path.to_path_buf()
                } else {
                    self.working_dir.join(path)
                }
            }
            None => self.working_dir.clone(),
        }
    }
}

impl Tool for RigGrepTool {
    const NAME: &'static str = "Grep";

    type Error = SearchError;
    type Args = GrepArgs;
    type Output = GrepOutput;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "基于 ripgrep 的强大搜索工具。支持正则表达式、文件过滤、上下文显示等功能。".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "pattern": {
                        "type": "string",
                        "description": "要搜索的正则表达式模式"
                    },
                    "path": {
                        "type": "string",
                        "description": "搜索路径（文件或目录）。默认为当前工作目录。"
                    },
                    "glob": {
                        "type": "string",
                        "description": "文件过滤 glob 模式（如 \"*.js\", \"*.{ts,tsx}\"）"
                    },
                    "type": {
                        "type": "string",
                        "description": "文件类型过滤（如 \"js\", \"py\", \"rust\"）"
                    },
                    "output_mode": {
                        "type": "string",
                        "enum": ["content", "files_with_matches", "count"],
                        "description": "输出模式：content（显示匹配行）、files_with_matches（仅文件路径）、count（匹配计数）。默认 files_with_matches。"
                    },
                    "context": {
                        "type": "number",
                        "description": "显示匹配行前后的上下文行数"
                    },
                    "-A": {
                        "type": "number",
                        "description": "显示匹配行后的行数"
                    },
                    "-B": {
                        "type": "number",
                        "description": "显示匹配行前的行数"
                    },
                    "-C": {
                        "type": "number",
                        "description": "context 的别名"
                    },
                    "-i": {
                        "type": "boolean",
                        "description": "忽略大小写"
                    },
                    "-n": {
                        "type": "boolean",
                        "description": "显示行号（仅 content 模式）。默认 true。"
                    },
                    "head_limit": {
                        "type": "number",
                        "description": "限制输出的前 N 行/条目"
                    },
                    "offset": {
                        "type": "number",
                        "description": "跳过前 N 行/条目"
                    },
                    "multiline": {
                        "type": "boolean",
                        "description": "启用多行模式，允许 . 匹配换行符"
                    }
                },
                "required": ["pattern"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let search_path = self.resolve_path(args.path.as_deref());

        if !search_path.exists() {
            return Err(SearchError::PathNotFound(search_path.display().to_string()));
        }

        let output_mode = args.output_mode.as_deref().unwrap_or("files_with_matches");

        // 构建正则匹配器
        let mut matcher_builder = RegexMatcherBuilder::new();
        matcher_builder.case_insensitive(args.case_insensitive);
        if args.multiline {
            matcher_builder.multi_line(true).dot_matches_new_line(true);
        }
        let matcher = matcher_builder
            .build(&args.pattern)
            .map_err(|e| SearchError::RegexError(e.to_string()))?;

        // 确定上下文行数
        let context_lines = args.context.or(args.context_alias).unwrap_or(0);
        let before_lines = args.before.unwrap_or(context_lines);
        let after_lines = args.after.unwrap_or(context_lines);

        let mut results = Vec::new();

        // 遍历文件
        let walker = if search_path.is_file() {
            WalkBuilder::new(&search_path).build()
        } else {
            let mut builder = WalkBuilder::new(&search_path);
            builder.hidden(false).git_ignore(true);
            builder.build()
        };

        for entry in walker.flatten() {
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            // 应用 glob 过滤
            if let Some(glob_pattern) = &args.glob {
                if let Some(file_name) = path.file_name() {
                    let pattern = Pattern::new(glob_pattern)?;
                    if !pattern.matches(file_name.to_string_lossy().as_ref()) {
                        continue;
                    }
                }
            }

            // 应用类型过滤
            if let Some(file_type) = &args.r#type {
                if let Some(ext) = path.extension() {
                    let ext_str = ext.to_string_lossy();
                    let matches = match file_type.as_str() {
                        "js" => ext_str == "js" || ext_str == "jsx",
                        "ts" => ext_str == "ts" || ext_str == "tsx",
                        "py" => ext_str == "py",
                        "rust" => ext_str == "rs",
                        "go" => ext_str == "go",
                        "java" => ext_str == "java",
                        _ => ext_str == file_type.as_str(),
                    };
                    if !matches {
                        continue;
                    }
                } else {
                    continue;
                }
            }

            // 搜索文件内容
            match output_mode {
                "files_with_matches" => {
                    let mut found = false;
                    let mut searcher = SearcherBuilder::new().build();
                    let _ = searcher.search_path(
                        &matcher,
                        path,
                        UTF8(|_lnum, _line| {
                            found = true;
                            Ok(false)
                        }),
                    );
                    if found {
                        results.push(path.display().to_string());
                    }
                }
                "count" => {
                    let mut count = 0;
                    let mut searcher = SearcherBuilder::new().build();
                    let _ = searcher.search_path(
                        &matcher,
                        path,
                        UTF8(|_lnum, _line| {
                            count += 1;
                            Ok(true)
                        }),
                    );
                    if count > 0 {
                        results.push(format!("{}:{}", path.display(), count));
                    }
                }
                "content" => {
                    let show_line_numbers = args.line_numbers.unwrap_or(true);
                    let mut matches_in_file = Vec::new();

                    let mut searcher = SearcherBuilder::new()
                        .before_context(before_lines)
                        .after_context(after_lines)
                        .build();

                    let _ = searcher.search_path(
                        &matcher,
                        path,
                        UTF8(|lnum, line| {
                            let line_str = if show_line_numbers {
                                format!("{}:{}", lnum, line.trim_end())
                            } else {
                                line.trim_end().to_string()
                            };
                            matches_in_file.push(line_str);
                            Ok(true)
                        }),
                    );

                    if !matches_in_file.is_empty() {
                        results.push(format!(
                            "{}:\n{}",
                            path.display(),
                            matches_in_file.join("\n")
                        ));
                    }
                }
                _ => {
                    return Err(SearchError::SearchFailed(format!(
                        "不支持的输出模式: {}",
                        output_mode
                    )));
                }
            }
        }

        // 应用 offset 和 head_limit
        let offset = args.offset.unwrap_or(0);
        let results: Vec<String> = results.into_iter().skip(offset).collect();

        let results = if let Some(limit) = args.head_limit {
            results.into_iter().take(limit).collect()
        } else {
            results
        };

        let count = results.len();
        Ok(GrepOutput { results, count })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_rig_glob_tool() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("test.rs"), "fn main() {}").unwrap();
        fs::write(temp_dir.path().join("test.txt"), "hello").unwrap();

        let tool = RigGlobTool::new(temp_dir.path().to_path_buf());
        let result = tool
            .call(GlobArgs {
                pattern: "*.rs".to_string(),
                path: None,
            })
            .await
            .unwrap();

        assert_eq!(result.count, 1);
        assert!(result.files[0].contains("test.rs"));
    }

    #[tokio::test]
    async fn test_rig_grep_tool() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("test.rs"), "fn main() {\n    println!(\"hello\");\n}").unwrap();

        let tool = RigGrepTool::new(temp_dir.path().to_path_buf());
        let result = tool
            .call(GrepArgs {
                pattern: "println".to_string(),
                path: None,
                glob: None,
                r#type: None,
                output_mode: Some("content".to_string()),
                context: None,
                after: None,
                before: None,
                context_alias: None,
                case_insensitive: false,
                line_numbers: Some(true),
                head_limit: None,
                offset: None,
                multiline: false,
            })
            .await
            .unwrap();

        assert_eq!(result.count, 1);
        assert!(result.results[0].contains("println"));
    }
}
