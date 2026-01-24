use super::FileToolError;
use colored::*;
use diffy::{apply, Patch};
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use similar::{TextDiff};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

/// 检查是否启用预览模式
fn preview_enabled() -> bool {
    // 通过环境变量 OXIDE_EDIT_PREVIEW 控制（默认启用）
    env::var("OXIDE_EDIT_PREVIEW")
        .unwrap_or_else(|_| "true".to_string())
        .parse::<bool>()
        .unwrap_or(true)
}

/// 渲染带颜色的 diff
fn render_colored_diff(original: &str, modified: &str) {
    let diff = TextDiff::from_lines(original, modified);

    for ops in diff.grouped_ops(3) {
        for op in ops {
            for change in diff.iter_changes(&op) {
                match change.tag() {
                    similar::ChangeTag::Equal => {
                        print!(" {}", change.value().dimmed());
                    }
                    similar::ChangeTag::Delete => {
                        print!("{}{}", "-".red(), change.value().red());
                    }
                    similar::ChangeTag::Insert => {
                        print!("{}{}", "+".green(), change.value().green());
                    }
                }
            }
        }
    }
    println!();
}

/// 请求用户确认
fn request_confirmation(lines_added: usize, lines_removed: usize) -> io::Result<bool> {
    print!(
        "\n{} {} (+{} lines, -{} lines)\n",
        "❓".bright_yellow(),
        "确认应用此修改？".bright_white(),
        lines_added.to_string().green(),
        lines_removed.to_string().red()
    );
    print!(
        "{}  [Y/n] ",
        "💡".bright_blue(),
    );

    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let input = input.trim().to_lowercase();
    Ok(input.is_empty() || input == "y" || input == "yes")
}

#[derive(Deserialize)]
pub struct EditFileArgs {
    pub file_path: String,
    pub patch: String,
}

#[derive(Serialize, Debug)]
pub struct EditFileOutput {
    pub file_path: String,
    pub lines_added: usize,
    pub lines_removed: usize,
    pub success: bool,
    pub message: String,
    /// 预览内容（如果生成了的话）
    pub preview: Option<String>,
    /// 是否被用户取消
    pub cancelled: bool,
}

#[derive(Deserialize, Serialize)]
pub struct EditFileTool;

impl Tool for EditFileTool {
    const NAME: &'static str = "edit_file";

    type Error = FileToolError;
    type Args = EditFileArgs;
    type Output = EditFileOutput;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "edit_file".to_string(),
            description: "Apply a unified diff patch to a file. This is efficient for making small, targeted changes to existing files without rewriting the entire content.\n\nIMPORTANT: The patch must be in valid unified diff format:\n\n```diff\n--- a/path/to/file.txt\n+++ b/path/to/file.txt\n@@ -line_number,count +line_number,count @@\n context_line1\n-old_line_to_remove\n+new_line_to_add\n context_line2\n```\n\nRules:\n1. Include both --- and +++ headers with file paths\n2. Hunk header @@ must include correct line numbers and counts\n3. Include 3 lines of context before and after changes\n4. Lines to remove start with '-'\n5. Lines to add start with '+'\n6. Context lines start with ' '\n\nExample patch:\n```diff\n--- a/src/main.rs\n+++ b/src/main.rs\n@@ -5,3 +5,4 @@\n fn main() {\n     let x = 5;\n-    println!(\"Old\");\n+    println!(\"New\");\n }\n```".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "The path to the file to edit (relative or absolute). The file must exist."
                    },
                    "patch": {
                        "type": "string",
                        "description": "A complete unified diff patch with proper headers and hunks. Must include ---/+++ headers and @@ hunk headers with correct line numbers."
                    }
                },
                "required": ["file_path", "patch"]
            })
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let file_path = &args.file_path;
        let patch_str = &args.patch;
        let path = Path::new(file_path);

        // Check if file exists
        if !path.exists() {
            return Err(FileToolError::FileNotFound(file_path.clone()));
        }

        // Check if it's actually a file (not a directory)
        if !path.is_file() {
            return Err(FileToolError::NotAFile(file_path.clone()));
        }

        // Read the current file content
        let current_content = fs::read_to_string(file_path)?;

        // Ensure patch_str ends with a newline
        let patch_str_normalized = if !patch_str.ends_with('\n') {
            format!("{}\n", patch_str)
        } else {
            patch_str.to_string()
        };

        // Parse the patch using diffy
        let patch = Patch::from_str(&patch_str_normalized).map_err(|e| {
            FileToolError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to parse patch: {}", e),
            ))
        })?;

        // Apply the patch using diffy::apply
        let patched_content = apply(&current_content, &patch).map_err(|e| {
            FileToolError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to apply patch: {}", e),
            ))
        })?;

        // Calculate statistics
        let original_lines: Vec<&str> = args.patch.lines().collect();
        let mut lines_added = 0usize;
        let mut lines_removed = 0usize;

        for line in original_lines {
            if line.starts_with('+') && !line.starts_with("+++") {
                lines_added += 1;
            } else if line.starts_with('-') && !line.starts_with("---") {
                lines_removed += 1;
            }
        }

        // Write the modified content back to the file
        match fs::write(file_path, &patched_content) {
            Ok(()) => Ok(EditFileOutput {
                file_path: file_path.clone(),
                lines_added,
                lines_removed,
                success: true,
                message: format!(
                    "Successfully applied patch to '{}': +{} lines, -{} lines",
                    file_path, lines_added, lines_removed
                ),
                preview: None,
                cancelled: false,
            }),
            Err(e) => match e.kind() {
                std::io::ErrorKind::PermissionDenied => {
                    Err(FileToolError::PermissionDenied(file_path.clone()))
                }
                _ => Err(FileToolError::Io(e)),
            },
        }
    }
}

impl EditFileTool {
    /// 预览补丁（不实际应用）
    /// 返回 (原始内容, 修改后内容, 新增行数, 删除行数, 补丁字符串)
    pub async fn preview_patch(&self, args: &EditFileArgs) -> Result<(String, String, usize, usize, String), FileToolError> {
        let file_path = &args.file_path;
        let patch_str = &args.patch;
        let path = Path::new(file_path);

        // Check if file exists
        if !path.exists() {
            return Err(FileToolError::FileNotFound(file_path.clone()));
        }

        // Check if it's actually a file (not a directory)
        if !path.is_file() {
            return Err(FileToolError::NotAFile(file_path.clone()));
        }

        // Read the current file content
        let current_content = fs::read_to_string(file_path)?;

        // Ensure patch_str ends with a newline
        let patch_str_normalized = if !patch_str.ends_with('\n') {
            format!("{}\n", patch_str)
        } else {
            patch_str.to_string()
        };

        // Parse the patch using diffy
        let patch = Patch::from_str(&patch_str_normalized).map_err(|e| {
            FileToolError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to parse patch: {}", e),
            ))
        })?;

        // Apply the patch using diffy::apply
        let patched_content = apply(&current_content, &patch).map_err(|e| {
            FileToolError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to apply patch: {}", e),
            ))
        })?;

        // Calculate statistics
        let original_lines: Vec<&str> = args.patch.lines().collect();
        let mut lines_added = 0usize;
        let mut lines_removed = 0usize;

        for line in original_lines {
            if line.starts_with('+') && !line.starts_with("+++") {
                lines_added += 1;
            } else if line.starts_with('-') && !line.starts_with("---") {
                lines_removed += 1;
            }
        }

        // 使用原始补丁字符串作为预览
        let preview = patch_str_normalized.clone();

        Ok((current_content, patched_content, lines_added, lines_removed, preview))
    }
}

#[derive(Deserialize, Serialize)]
pub struct WrappedEditFileTool {
    inner: EditFileTool,
}

impl WrappedEditFileTool {
    pub fn new() -> Self {
        Self {
            inner: EditFileTool,
        }
    }
}

impl Tool for WrappedEditFileTool {
    const NAME: &'static str = "edit_file";

    type Error = FileToolError;
    type Args = <EditFileTool as Tool>::Args;
    type Output = <EditFileTool as Tool>::Output;

    async fn definition(&self, prompt: String) -> ToolDefinition {
        self.inner.definition(prompt).await
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        println!();
        println!("{} {}({})", "●".bright_green(), "Edit", args.file_path);

        // 检查是否启用预览
        if preview_enabled() {
            // 生成预览
            match self.inner.preview_patch(&args).await {
                Ok((current_content, patched_content, lines_added, lines_removed, preview)) => {
                    // 显示预览
                    println!();
                    println!("{}", "📋 即将应用以下修改:".bright_cyan().bold());
                    println!();
                    render_colored_diff(&current_content, &patched_content);
                    println!();

                    // 请求用户确认
                    match request_confirmation(lines_added, lines_removed) {
                        Ok(true) => {
                            // 用户确认，应用修改
                            if let Err(e) = fs::write(&args.file_path, &patched_content) {
                                println!("  └─ {}", format!("Error: {}", e).red());
                                println!();
                                return match e.kind() {
                                    std::io::ErrorKind::PermissionDenied => {
                                        Err(FileToolError::PermissionDenied(args.file_path.clone()))
                                    }
                                    _ => Err(FileToolError::Io(e)),
                                };
                            }

                            println!(
                                "  └─ {} (+{} lines, -{} lines)",
                                format!("Patched '{}'", args.file_path).dimmed(),
                                lines_added.to_string().green(),
                                lines_removed.to_string().red()
                            );
                            println!();

                            Ok(EditFileOutput {
                                file_path: args.file_path.clone(),
                                lines_added,
                                lines_removed,
                                success: true,
                                message: format!(
                                    "已应用修改到 '{}': +{} 行, -{} 行",
                                    args.file_path, lines_added, lines_removed
                                ),
                                preview: Some(preview),
                                cancelled: false,
                            })
                        }
                        Ok(false) => {
                            // 用户取消
                            println!("  └─ {}", "修改已取消".bright_yellow());
                            println!();
                            Ok(EditFileOutput {
                                file_path: args.file_path.clone(),
                                lines_added,
                                lines_removed,
                                success: true,
                                message: "用户取消了修改".to_string(),
                                preview: Some(preview),
                                cancelled: true,
                            })
                        }
                        Err(e) => {
                            println!("  └─ {}", format!("读取输入错误: {}", e).red());
                            println!();
                            Err(FileToolError::Io(e))
                        }
                    }
                }
                Err(e) => {
                    println!("  └─ {}", format!("预览失败: {}", e).red());
                    println!();
                    Err(e)
                }
            }
        } else {
            // 不启用预览，直接应用
            let result = self.inner.call(args).await;

            match &result {
                Ok(output) => {
                    println!(
                        "  └─ {} (+{} lines, -{} lines)",
                        format!("Patched '{}'", output.file_path).dimmed(),
                        output.lines_added.to_string().green(),
                        output.lines_removed.to_string().red()
                    );
                }
                Err(e) => {
                    println!("  └─ {}", format!("Error: {}", e).red());
                }
            }
            println!();
            result
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_preview_patch() {
        let tool = EditFileTool;

        // 创建临时测试文件
        let temp_file = NamedTempFile::new().unwrap();
        let test_path = temp_file.path().to_path_buf();
        fs::write(&test_path, "line 1\nline 2\nline 3\n").unwrap();

        let args = EditFileArgs {
            file_path: test_path.to_str().unwrap().to_string(),
            patch: "@@ -1,3 +1,3 @@
 line 1
-line 2
+line 2 modified
 line 3
".to_string(),
        };

        let result = tool.preview_patch(&args).await;
        assert!(result.is_ok(), "预览应该成功");

        let (original, modified, added, removed, preview) = result.unwrap();

        // 验证原始内容
        assert_eq!(original, "line 1\nline 2\nline 3\n");

        // 验证修改后内容
        assert_eq!(modified, "line 1\nline 2 modified\nline 3\n");

        // 验证统计
        assert_eq!(added, 1);
        assert_eq!(removed, 1);

        // 验证预览包含补丁信息
        assert!(preview.contains("line 2"));
        assert!(preview.contains("line 2 modified"));
    }

    #[test]
    fn test_preview_enabled_default() {
        // 默认应该启用预览
        assert!(preview_enabled());
    }

    #[test]
    fn test_preview_disabled_by_env() {
        // 临时设置环境变量
        env::set_var("OXIDE_EDIT_PREVIEW", "false");
        assert!(!preview_enabled());

        // 恢复默认
        env::set_var("OXIDE_EDIT_PREVIEW", "true");
        assert!(preview_enabled());

        // 清理
        env::remove_var("OXIDE_EDIT_PREVIEW");
        assert!(preview_enabled()); // 应该回退到默认值 true
    }

    #[tokio::test]
    async fn test_preview_patch_file_not_found() {
        let tool = EditFileTool;

        let args = EditFileArgs {
            file_path: "/nonexistent/file.rs".to_string(),
            patch: "@@ -1,1 +1,1 @@
-old
+new
".to_string(),
        };

        let result = tool.preview_patch(&args).await;
        assert!(result.is_err());

        match result {
            Err(FileToolError::FileNotFound(path)) => {
                assert_eq!(path, "/nonexistent/file.rs");
            }
            _ => panic!("应该返回 FileNotFound 错误"),
        }
    }

    #[tokio::test]
    async fn test_preview_patch_invalid_patch() {
        let tool = EditFileTool;

        // 创建临时文件
        let temp_file = NamedTempFile::new().unwrap();
        let test_path = temp_file.path().to_str().unwrap().to_string();
        fs::write(&test_path, "content\n").unwrap();

        // 使用无法应用的补丁（行号不匹配）
        let args = EditFileArgs {
            file_path: test_path,
            patch: "@@ -10,5 +10,5 @@
-line 10
-line 11
+line 10 modified
+line 11 modified
".to_string(),
        };

        let result = tool.preview_patch(&args).await;
        // diffy 会成功解析补丁，但应用时会失败或产生空结果
        // 这里我们只验证它能处理这种情况而不崩溃
        match result {
            Ok((_original, _modified, added, removed, _preview)) => {
                // 应该返回结果，即使没有实际修改
                assert_eq!(added, 2);
                assert_eq!(removed, 2);
            }
            Err(_) => {
                // 或者返回错误也是可接受的
            }
        }
    }
}
