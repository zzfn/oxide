//! 文件工具的 rig Tool trait 适配: Read, Write, Edit

use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};

use super::errors::FileError;

// Read 工具

/// Read 工具参数
#[derive(Debug, Deserialize)]
pub struct ReadArgs {
    /// 文件路径
    pub file_path: String,
    /// 起始行号（从 0 开始）
    #[serde(default)]
    pub offset: Option<usize>,
    /// 读取的行数
    #[serde(default)]
    pub limit: Option<usize>,
}

/// Read 工具输出
#[derive(Debug, Serialize)]
pub struct ReadOutput {
    /// 文件内容（带行号）
    pub content: String,
    /// 文件路径
    pub path: String,
    /// 总行数
    pub total_lines: usize,
    /// 显示的起始行
    pub start_line: usize,
    /// 显示的结束行
    pub end_line: usize,
}

/// Read 工具 - 读取文件内容
#[derive(Clone, Serialize, Deserialize)]
pub struct RigReadTool {
    working_dir: PathBuf,
}

impl RigReadTool {
    pub fn new(working_dir: PathBuf) -> Self {
        Self { working_dir }
    }

    fn resolve_path(&self, path: &str) -> PathBuf {
        let path = Path::new(path);
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.working_dir.join(path)
        }
    }
}

impl Tool for RigReadTool {
    const NAME: &'static str = "Read";

    type Error = FileError;
    type Args = ReadArgs;
    type Output = ReadOutput;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "读取文件内容，支持行范围读取".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "文件路径（绝对路径或相对于工作目录的路径）"
                    },
                    "offset": {
                        "type": "integer",
                        "description": "起始行号（从 0 开始，可选）"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "读取的行数（可选，不指定则读取到文件末尾）"
                    }
                },
                "required": ["file_path"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let path = self.resolve_path(&args.file_path);

        if !path.exists() {
            return Err(FileError::FileNotFound(path.display().to_string()));
        }

        if !path.is_file() {
            return Err(FileError::NotFile(path.display().to_string()));
        }

        let file_content = fs::read_to_string(&path)?;
        let lines: Vec<&str> = file_content.lines().collect();
        let total_lines = lines.len();

        let start = args.offset.unwrap_or(0);
        let end = if let Some(limit) = args.limit {
            (start + limit).min(total_lines)
        } else {
            total_lines
        };

        let content = if start >= total_lines {
            String::new()
        } else {
            let mut result = String::new();
            for (idx, line) in lines[start..end].iter().enumerate() {
                let line_num = start + idx + 1;
                result.push_str(&format!("{:6}→{}\n", line_num, line));
            }
            result
        };

        Ok(ReadOutput {
            content,
            path: path.display().to_string(),
            total_lines,
            start_line: start + 1,
            end_line: end,
        })
    }
}

// Write 工具

/// Write 工具参数
#[derive(Debug, Deserialize)]
pub struct WriteArgs {
    /// 文件路径
    pub file_path: String,
    /// 要写入的内容
    pub content: String,
}

/// Write 工具输出
#[derive(Debug, Serialize)]
pub struct WriteOutput {
    /// 文件路径
    pub path: String,
    /// 写入的行数
    pub lines: usize,
    /// 写入的字节数
    pub bytes: usize,
    /// 是否是新创建的文件
    pub created: bool,
}

/// Write 工具 - 写入文件
#[derive(Clone, Serialize, Deserialize)]
pub struct RigWriteTool {
    working_dir: PathBuf,
}

impl RigWriteTool {
    pub fn new(working_dir: PathBuf) -> Self {
        Self { working_dir }
    }

    fn resolve_path(&self, path: &str) -> PathBuf {
        let path = Path::new(path);
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.working_dir.join(path)
        }
    }
}

impl Tool for RigWriteTool {
    const NAME: &'static str = "Write";

    type Error = FileError;
    type Args = WriteArgs;
    type Output = WriteOutput;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "写入文件内容，如果文件存在则覆盖".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "文件路径（绝对路径或相对于工作目录的路径）"
                    },
                    "content": {
                        "type": "string",
                        "description": "要写入的内容"
                    }
                },
                "required": ["file_path", "content"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let path = self.resolve_path(&args.file_path);
        let created = !path.exists();

        // 创建父目录（如果不存在）
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        let lines = args.content.lines().count();
        let bytes = args.content.len();

        fs::write(&path, &args.content)?;

        Ok(WriteOutput {
            path: path.display().to_string(),
            lines,
            bytes,
            created,
        })
    }
}

// Edit 工具

/// Edit 工具参数
#[derive(Debug, Deserialize)]
pub struct EditArgs {
    /// 文件路径
    pub file_path: String,
    /// 要替换的字符串
    pub old_string: String,
    /// 替换后的字符串
    pub new_string: String,
    /// 是否替换所有匹配项
    #[serde(default)]
    pub replace_all: bool,
}

/// Edit 工具输出
#[derive(Debug, Serialize)]
pub struct EditOutput {
    /// 文件路径
    pub path: String,
    /// 替换的次数
    pub replacements: usize,
}

/// Edit 工具 - 精确字符串替换
#[derive(Clone, Serialize, Deserialize)]
pub struct RigEditTool {
    working_dir: PathBuf,
}

impl RigEditTool {
    pub fn new(working_dir: PathBuf) -> Self {
        Self { working_dir }
    }

    fn resolve_path(&self, path: &str) -> PathBuf {
        let path = Path::new(path);
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.working_dir.join(path)
        }
    }
}

impl Tool for RigEditTool {
    const NAME: &'static str = "Edit";

    type Error = FileError;
    type Args = EditArgs;
    type Output = EditOutput;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "精确字符串替换，支持单次或批量替换".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "文件路径（绝对路径或相对于工作目录的路径）"
                    },
                    "old_string": {
                        "type": "string",
                        "description": "要替换的字符串"
                    },
                    "new_string": {
                        "type": "string",
                        "description": "替换后的字符串"
                    },
                    "replace_all": {
                        "type": "boolean",
                        "description": "是否替换所有匹配项（默认 false，只替换第一个）"
                    }
                },
                "required": ["file_path", "old_string", "new_string"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let path = self.resolve_path(&args.file_path);

        if !path.exists() {
            return Err(FileError::FileNotFound(path.display().to_string()));
        }

        if !path.is_file() {
            return Err(FileError::NotFile(path.display().to_string()));
        }

        let content = fs::read_to_string(&path)?;

        if !content.contains(&args.old_string) {
            return Err(FileError::StringNotFound(args.old_string.clone()));
        }

        let count = content.matches(&args.old_string).count();

        // 检查唯一性（如果不是 replace_all 模式）
        if !args.replace_all && count > 1 {
            return Err(FileError::MultipleMatches { count });
        }

        let new_content = if args.replace_all {
            content.replace(&args.old_string, &args.new_string)
        } else {
            content.replacen(&args.old_string, &args.new_string, 1)
        };

        fs::write(&path, &new_content)?;

        let replacements = if args.replace_all { count } else { 1 };

        Ok(EditOutput {
            path: path.display().to_string(),
            replacements,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_rig_read_tool() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "line1\nline2\nline3\n").unwrap();

        let tool = RigReadTool::new(temp_dir.path().to_path_buf());
        let result = tool
            .call(ReadArgs {
                file_path: "test.txt".to_string(),
                offset: None,
                limit: None,
            })
            .await
            .unwrap();

        assert_eq!(result.total_lines, 3);
        assert!(result.content.contains("line1"));
    }

    #[tokio::test]
    async fn test_rig_write_tool() {
        let temp_dir = TempDir::new().unwrap();
        let tool = RigWriteTool::new(temp_dir.path().to_path_buf());

        let result = tool
            .call(WriteArgs {
                file_path: "new.txt".to_string(),
                content: "Hello, World!".to_string(),
            })
            .await
            .unwrap();

        assert!(result.created);
        let content = fs::read_to_string(temp_dir.path().join("new.txt")).unwrap();
        assert_eq!(content, "Hello, World!");
    }

    #[tokio::test]
    async fn test_rig_edit_tool() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("edit.txt");
        fs::write(&file_path, "Hello, World!").unwrap();

        let tool = RigEditTool::new(temp_dir.path().to_path_buf());
        let result = tool
            .call(EditArgs {
                file_path: "edit.txt".to_string(),
                old_string: "World".to_string(),
                new_string: "Rust".to_string(),
                replace_all: false,
            })
            .await
            .unwrap();

        assert_eq!(result.replacements, 1);
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "Hello, Rust!");
    }

    #[tokio::test]
    async fn test_rig_edit_tool_replace_all() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("edit.txt");
        fs::write(&file_path, "foo bar foo baz foo").unwrap();

        let tool = RigEditTool::new(temp_dir.path().to_path_buf());
        let result = tool
            .call(EditArgs {
                file_path: "edit.txt".to_string(),
                old_string: "foo".to_string(),
                new_string: "qux".to_string(),
                replace_all: true,
            })
            .await
            .unwrap();

        assert_eq!(result.replacements, 3);
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "qux bar qux baz qux");
    }
}
