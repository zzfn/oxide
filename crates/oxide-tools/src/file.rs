//! 文件操作工具: Read, Write, Edit

use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

use crate::registry::{Tool, ToolResult, ToolSchema};

/// Read 工具参数
#[derive(Debug, Deserialize)]
struct ReadParams {
    file_path: String,
    #[serde(default)]
    offset: Option<usize>,
    #[serde(default)]
    limit: Option<usize>,
}

/// Write 工具参数
#[derive(Debug, Deserialize)]
struct WriteParams {
    file_path: String,
    content: String,
}

/// Edit 工具参数
#[derive(Debug, Deserialize)]
struct EditParams {
    file_path: String,
    old_string: String,
    new_string: String,
    #[serde(default)]
    replace_all: bool,
}

/// Read 工具 - 读取文件内容
pub struct ReadTool {
    working_dir: PathBuf,
}

impl ReadTool {
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

    fn read_file_with_range(
        &self,
        path: &Path,
        offset: Option<usize>,
        limit: Option<usize>,
    ) -> anyhow::Result<String> {
        let content = fs::read_to_string(path)?;
        let lines: Vec<&str> = content.lines().collect();

        let start = offset.unwrap_or(0);
        let end = if let Some(limit) = limit {
            (start + limit).min(lines.len())
        } else {
            lines.len()
        };

        if start >= lines.len() {
            return Ok(String::new());
        }

        // 格式化输出，带行号
        let mut result = String::new();
        for (idx, line) in lines[start..end].iter().enumerate() {
            let line_num = start + idx + 1;
            result.push_str(&format!("{:6}→{}\n", line_num, line));
        }

        Ok(result)
    }
}

#[async_trait]
impl Tool for ReadTool {
    fn name(&self) -> &str {
        "Read"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: "Read".to_string(),
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

    async fn execute(&self, input: Value) -> anyhow::Result<ToolResult> {
        let params: ReadParams = serde_json::from_value(input)?;
        let path = self.resolve_path(&params.file_path);

        // 检查文件是否存在
        if !path.exists() {
            return Ok(ToolResult::error(format!(
                "文件不存在: {}",
                path.display()
            )));
        }

        // 检查是否是文件
        if !path.is_file() {
            return Ok(ToolResult::error(format!(
                "路径不是文件: {}",
                path.display()
            )));
        }

        // 读取文件
        match self.read_file_with_range(&path, params.offset, params.limit) {
            Ok(content) => {
                let total_lines = fs::read_to_string(&path)?.lines().count();
                let range_info = if params.offset.is_some() || params.limit.is_some() {
                    let start = params.offset.unwrap_or(0) + 1;
                    let end = if let Some(limit) = params.limit {
                        (start + limit - 1).min(total_lines)
                    } else {
                        total_lines
                    };
                    format!(" (显示第 {}-{} 行，共 {} 行)", start, end, total_lines)
                } else {
                    format!(" (共 {} 行)", total_lines)
                };

                Ok(ToolResult::success(format!(
                    "文件: {}{}\n\n{}",
                    path.display(),
                    range_info,
                    content
                )))
            }
            Err(e) => Ok(ToolResult::error(format!("读取文件失败: {}", e))),
        }
    }
}

/// Write 工具 - 写入文件
pub struct WriteTool {
    working_dir: PathBuf,
}

impl WriteTool {
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

#[async_trait]
impl Tool for WriteTool {
    fn name(&self) -> &str {
        "Write"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: "Write".to_string(),
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

    async fn execute(&self, input: Value) -> anyhow::Result<ToolResult> {
        let params: WriteParams = serde_json::from_value(input)?;
        let path = self.resolve_path(&params.file_path);

        // 检查文件是否已存在
        let existed = path.exists();

        // 创建父目录（如果不存在）
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        // 写入文件
        match fs::write(&path, &params.content) {
            Ok(_) => {
                let lines = params.content.lines().count();
                let bytes = params.content.len();
                let action = if existed { "覆盖" } else { "创建" };

                Ok(ToolResult::success(format!(
                    "✓ {} 文件: {}\n  {} 行，{} 字节",
                    action,
                    path.display(),
                    lines,
                    bytes
                )))
            }
            Err(e) => Ok(ToolResult::error(format!("写入文件失败: {}", e))),
        }
    }
}

/// Edit 工具 - 精确字符串替换
pub struct EditTool {
    working_dir: PathBuf,
}

impl EditTool {
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

#[async_trait]
impl Tool for EditTool {
    fn name(&self) -> &str {
        "Edit"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: "Edit".to_string(),
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

    async fn execute(&self, input: Value) -> anyhow::Result<ToolResult> {
        let params: EditParams = serde_json::from_value(input)?;
        let path = self.resolve_path(&params.file_path);

        // 检查文件是否存在
        if !path.exists() {
            return Ok(ToolResult::error(format!(
                "文件不存在: {}",
                path.display()
            )));
        }

        // 读取文件内容
        let content = fs::read_to_string(&path)?;

        // 检查 old_string 是否存在
        if !content.contains(&params.old_string) {
            return Ok(ToolResult::error(format!(
                "未找到要替换的字符串: \"{}\"",
                params.old_string
            )));
        }

        // 执行替换
        let new_content = if params.replace_all {
            content.replace(&params.old_string, &params.new_string)
        } else {
            // 只替换第一个匹配项
            content.replacen(&params.old_string, &params.new_string, 1)
        };

        // 检查唯一性（如果不是 replace_all 模式）
        if !params.replace_all {
            let count = content.matches(&params.old_string).count();
            if count > 1 {
                return Ok(ToolResult::error(format!(
                    "找到 {} 个匹配项，但 replace_all=false。请提供更具体的字符串或设置 replace_all=true",
                    count
                )));
            }
        }

        // 写入文件
        match fs::write(&path, &new_content) {
            Ok(_) => {
                let count = if params.replace_all {
                    content.matches(&params.old_string).count()
                } else {
                    1
                };

                Ok(ToolResult::success(format!(
                    "✓ 编辑文件: {}\n  替换了 {} 处",
                    path.display(),
                    count
                )))
            }
            Err(e) => Ok(ToolResult::error(format!("写入文件失败: {}", e))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_read_tool() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "line1\nline2\nline3\n").unwrap();

        let tool = ReadTool::new(temp_dir.path().to_path_buf());
        let result = tool
            .execute(json!({
                "file_path": "test.txt"
            }))
            .await
            .unwrap();

        assert!(!result.is_error);
        assert!(result.content.contains("line1"));
    }

    #[tokio::test]
    async fn test_write_tool() {
        let temp_dir = TempDir::new().unwrap();
        let tool = WriteTool::new(temp_dir.path().to_path_buf());

        let result = tool
            .execute(json!({
                "file_path": "new.txt",
                "content": "Hello, World!"
            }))
            .await
            .unwrap();

        assert!(!result.is_error);
        let content = fs::read_to_string(temp_dir.path().join("new.txt")).unwrap();
        assert_eq!(content, "Hello, World!");
    }

    #[tokio::test]
    async fn test_edit_tool() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("edit.txt");
        fs::write(&file_path, "Hello, World!").unwrap();

        let tool = EditTool::new(temp_dir.path().to_path_buf());
        let result = tool
            .execute(json!({
                "file_path": "edit.txt",
                "old_string": "World",
                "new_string": "Rust"
            }))
            .await
            .unwrap();

        assert!(!result.is_error);
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "Hello, Rust!");
    }
}
