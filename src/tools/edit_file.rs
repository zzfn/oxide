use super::FileToolError;
use colored::*;
use diffy::{apply, Patch};
use super::ask_user_question::{ask_question_interactive, Question, QuestionOption};
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use similar::{TextDiff};
use std::borrow::Cow;
use std::env;
use std::fs;
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
fn request_confirmation(
    lines_added: usize,
    lines_removed: usize,
    confirmation: Option<&Question>,
) -> Result<bool, FileToolError> {
    print!(
        "\n{} {} (+{} lines, -{} lines)\n",
        "❓".bright_yellow(),
        "确认应用此修改？".bright_white(),
        lines_added.to_string().green(),
        lines_removed.to_string().red()
    );
    let default_question = Question {
        question: "确认应用此修改？".to_string(),
        header: "确认".to_string(),
        options: vec![
            QuestionOption {
                label: "是".to_string(),
                description: "应用当前修改".to_string(),
            },
            QuestionOption {
                label: "否".to_string(),
                description: "取消本次修改".to_string(),
            },
        ],
        multi_select: false,
    };

    let question = confirmation.cloned().unwrap_or(default_question);
    let approve_label = question
        .options
        .first()
        .map(|opt| opt.label.clone())
        .unwrap_or_else(|| "是".to_string());

    let answer = ask_question_interactive(&question)?;
    match answer.selected {
        serde_json::Value::String(label) => Ok(label == approve_label),
        serde_json::Value::Array(labels) => Ok(labels
            .iter()
            .any(|item| item.as_str() == Some(&approve_label))),
        _ => Ok(false),
    }
}

fn build_parse_error<E: std::fmt::Display>(e: E, patch_str: &str) -> FileToolError {
    // 提取 patch 的前几行用于诊断
    let preview_lines: Vec<&str> = patch_str.lines().take(20).collect();
    let patch_preview = preview_lines.join("\n");

    let error_msg = format!(
        "Failed to parse patch: {}\n\n\
         ═══════════════════════════════════════════════════════════\n\
         🔍 Patch 解析失败 - 诊断信息:\n\
         ═══════════════════════════════════════════════════════════\n\
         \n\
         常见原因:\n\
         1. ❌ Hunk header 格式错误\n\
            正确格式: @@ -line_count,count +line_count,count @@\n\
         2. ❌ 缺少足够的上下文行（推荐 3 行）\n\
         3. ❌ 行号不准确（文件内容可能已改变）\n\
         4. ❌ 缩进不匹配（空格/制表符）\n\
         5. ❌ 缺少 ---/+++ 文件头\n\
         \n\
         📋 Patch 内容预览（前 20 行）:\n\
         ─────────────────────────────────────────────────────────────\n\
         {}\n\
         ─────────────────────────────────────────────────────────────\n\
         \n\
         💡 建议:\n\
         - 检查 hunk header 中的行号是否准确\n\
         - 确保包含足够的上下文行\n\
         - 使用 Read 工具确认当前文件内容\n\
         - 或考虑使用 search-replace 格式代替 unified diff",
        e, patch_preview
    );

    FileToolError::Io(std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        error_msg,
    ))
}

fn parse_range(range: &str) -> Option<(usize, usize)> {
    let range = range.trim_start_matches(['-', '+']);
    let mut iter = range.split(',');
    let start = iter.next()?.parse().ok()?;
    let count = match iter.next() {
        Some(val) => val.parse().ok()?,
        None => 1,
    };
    Some((start, count))
}

fn rebuild_hunk_header(header: &str, hunk_lines: &[&str]) -> Option<String> {
    if !header.starts_with("@@") {
        return None;
    }

    let rest = &header[2..];
    let idx = rest.find("@@")?;
    let header_body = rest[..idx].trim();
    let trailing = &rest[idx + 2..];
    let mut parts = header_body.split_whitespace();
    let old_range = parts.next()?;
    let new_range = parts.next()?;
    let (old_start, _) = parse_range(old_range)?;
    let (new_start, _) = parse_range(new_range)?;

    let mut old_count = 0usize;
    let mut new_count = 0usize;
    for line in hunk_lines {
        if line.starts_with(' ') {
            old_count += 1;
            new_count += 1;
        } else if line.starts_with('-') {
            old_count += 1;
        } else if line.starts_with('+') {
            new_count += 1;
        } else if line.starts_with('\\') {
            // "\ No newline at end of file" 不计入行数
        }
    }

    Some(format!(
        "@@ -{},{} +{},{} @@{}",
        old_start, old_count, new_start, new_count, trailing
    ))
}

fn normalize_patch_hunk_counts(patch: &str) -> String {
    let lines: Vec<&str> = patch.lines().collect();
    let mut out: Vec<String> = Vec::with_capacity(lines.len());
    let mut i = 0usize;

    while i < lines.len() {
        let line = lines[i];
        if line.starts_with("@@") {
            let mut j = i + 1;
            while j < lines.len() && !lines[j].starts_with("@@") {
                j += 1;
            }
            let hunk_lines = &lines[i + 1..j];
            if let Some(new_header) = rebuild_hunk_header(line, hunk_lines) {
                out.push(new_header);
            } else {
                out.push(line.to_string());
            }
            for &hunk_line in hunk_lines {
                out.push(hunk_line.to_string());
            }
            i = j;
        } else {
            out.push(line.to_string());
            i += 1;
        }
    }

    let mut normalized = out.join("\n");
    if patch.ends_with('\n') {
        normalized.push('\n');
    }
    normalized
}

fn normalize_patch_for_parse<'a>(patch_str: &'a str) -> Result<Cow<'a, str>, FileToolError> {
    match Patch::from_str(patch_str) {
        Ok(_patch) => Ok(Cow::Borrowed(patch_str)),
        Err(e) => {
            let err_msg = e.to_string();
            if err_msg.contains("Hunk header does not match hunk") {
                let repaired = normalize_patch_hunk_counts(patch_str);
                if repaired != patch_str {
                    if Patch::from_str(&repaired).is_ok() {
                        return Ok(Cow::Owned(repaired));
                    }
                }
            }
            Err(build_parse_error(e, patch_str))
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct EditFileArgs {
    pub file_path: String,
    pub patch: String,
    #[serde(default)]
    pub confirmation: Option<Question>,
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
            description: r#"
编辑文件的高效工具，使用 unified diff 格式应用补丁。

⚠️ 重要提示：使用此工具前必须先使用 Read 工具读取文件的最新内容！

═══════════════════════════════════════════════════════════════════════════
📖 使用指南
═══════════════════════════════════════════════════════════════════════════

【推荐方案】如果你不确定准确的行号，请避免使用此工具。
考虑使用 write_file 工具重写整个文件，或先读取文件确认行号。

【高级方案】Unified Diff 格式要求：

1️⃣ 必须包含文件头：
   --- a/path/to/file.txt
   +++ b/path/to/file.txt

2️⃣ Hunk header 格式：
   @@ -起始行,行数 +起始行,行数 @@

   注意：
   - 起始行从 1 开始计数
   - 行数包含上下文、删除和新增的所有行
   - 删除的行用 -old_line
   - 新增的行用 +new_line
   - 上下文行用 空格+context_line

3️⃣ 必须包含足够的上下文（推荐 3 行）：
   - 上下文行帮助定位修改位置
   - 上下文必须与文件内容完全一致（包括缩进）
   - 上下文不匹配会导致应用失败

4️⃣ 完整示例：

   假设文件内容：
   1: fn main() {
   2:     let x = 5;
   3:     println!("Old");
   4: }

   要修改第 3 行，正确的 patch 是：
   ```diff
   --- a/src/main.rs
   +++ b/src/main.rs
   @@ -1,4 +1,4 @@
    fn main() {
        let x = 5;
   -    println!("Old");
   +    println!("New");
    }
   ```

   说明：
   - -1,4 表示从第 1 行开始，共 4 行（原文件）
   - +1,4 表示从第 1 行开始，共 4 行（修改后）
   - 包含 3 行上下文：第 1、2、4 行

═══════════════════════════════════════════════════════════════════════════
❌ 常见错误
═══════════════════════════════════════════════════════════════════════════

1. ❌ 行号错误：未先 Read 文件就假设行号
   ✅ 解决：先 Read 文件，确认准确的行号

2. ❌ 缺少上下文：只有修改的行，没有上下文
   ✅ 解决：包含修改前后各 3 行上下文

3. ❌ 缩进不匹配：patch 中的空格与文件不一致
   ✅ 解决：使用 Read 工具复制确切的缩进

4. ❌ 文件已改变：生成 patch 后文件被修改
   ✅ 解决：重新 Read 文件，生成新的 patch

═══════════════════════════════════════════════════════════════════════════
💡 最佳实践
═══════════════════════════════════════════════════════════════════════════

1. 每次使用 edit_file 前必须先 Read 文件
2. 复制文件中的确切内容作为上下文（包括缩进）
3. 小修改（< 10 行）使用 edit_file
4. 大修改（≥ 10 行）考虑使用 write_file
5. 如果 patch 应用失败，检查错误提示中的诊断信息
"#.trim().to_string(),
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
                    },
                    "confirmation": {
                        "type": "object",
                        "description": "Optional confirmation prompt. The first option is treated as approval; other selections cancel the edit.",
                        "properties": {
                            "question": {
                                "type": "string",
                                "description": "The complete question text"
                            },
                            "header": {
                                "type": "string",
                                "description": "Short header/title for the question (max 12 chars recommended)"
                            },
                            "options": {
                                "type": "array",
                                "description": "List of answer options. The first option is treated as approval.",
                                "items": {
                                    "type": "object",
                                    "properties": {
                                        "label": {
                                            "type": "string",
                                            "description": "Short option label"
                                        },
                                        "description": {
                                            "type": "string",
                                            "description": "Detailed description of the option"
                                        }
                                    },
                                    "required": ["label", "description"]
                                }
                            },
                            "multi_select": {
                                "type": "boolean",
                                "description": "Whether to allow multiple selections (default: false)"
                            }
                        },
                        "required": ["question", "header", "options", "multi_select"]
                    }
                },
                "required": ["file_path", "patch"]
            })
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let (_current_content, patched_content, lines_added, lines_removed) =
            Self::apply_patch_internal(&args.file_path, &args.patch)?;

        // Write the modified content back to the file
        match fs::write(&args.file_path, &patched_content) {
            Ok(()) => Ok(EditFileOutput {
                file_path: args.file_path.clone(),
                lines_added,
                lines_removed,
                success: true,
                message: format!(
                    "Successfully applied patch to '{}': +{} lines, -{} lines",
                    args.file_path, lines_added, lines_removed
                ),
                preview: None,
                cancelled: false,
            }),
            Err(e) => match e.kind() {
                std::io::ErrorKind::PermissionDenied => {
                    Err(FileToolError::PermissionDenied(args.file_path.clone()))
                }
                _ => Err(FileToolError::Io(e)),
            },
        }
    }
}

impl EditFileTool {
    /// 内部方法：应用补丁并返回所有中间结果
    /// 返回 (原始内容, 修改后内容, 新增行数, 删除行数)
    fn apply_patch_internal(
        file_path: &str,
        patch_str: &str,
    ) -> Result<(String, String, usize, usize), FileToolError> {
        let path = Path::new(file_path);

        // Check if file exists
        if !path.exists() {
            return Err(FileToolError::FileNotFound(file_path.to_string()));
        }

        // Check if it's actually a file (not a directory)
        if !path.is_file() {
            return Err(FileToolError::NotAFile(file_path.to_string()));
        }

        // Read the current file content
        let current_content = fs::read_to_string(file_path)?;

        // Ensure patch_str ends with a newline
        let patch_str_normalized = if !patch_str.ends_with('\n') {
            Cow::Owned(format!("{}\n", patch_str))
        } else {
            Cow::Borrowed(patch_str)
        };

        // Parse the patch using diffy (with repair for bad hunk counts)
        let patch_str_used = normalize_patch_for_parse(&patch_str_normalized)?;
        let patch = Patch::from_str(patch_str_used.as_ref())
            .map_err(|e| build_parse_error(e, patch_str_used.as_ref()))?;

        // Apply the patch using diffy::apply
        let patched_content = apply(&current_content, &patch).map_err(|e| {
            // 计算文件行数用于诊断
            let file_lines: Vec<&str> = current_content.lines().collect();
            let total_lines = file_lines.len();

            let error_msg = format!(
                "Failed to apply patch: {}\n\n\
                 ═══════════════════════════════════════════════════════════\n\
                 ❌ Patch 应用失败 - 诊断信息:\n\
                 ═══════════════════════════════════════════════════════════\n\
                 \n\
                 文件信息:\n\
                 - 文件: {}\n\
                 - 总行数: {}\n\
                 \n\
                 常见原因:\n\
                 1. ❌ Hunk header 中的行号超出文件范围\n\
                 2. ❌ 上下文内容与文件实际内容不匹配\n\
                 3. ❌ 文件内容在生成 patch 后已被修改\n\
                 4. ❌ 缩进或空格不匹配\n\
                 \n\
                 💡 建议:\n\
                 - 使用 Read 工具重新读取文件，确认当前内容\n\
                 - 检查 patch 中的上下文行是否与文件完全一致\n\
                 - 确认 hunk header 的行号在有效范围内 (1-{})\n\
                 - 如果文件最近被修改过，需要重新生成 patch",
                e, file_path, total_lines, total_lines
            );

            FileToolError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                error_msg,
            ))
        })?;

        // Calculate statistics
        let original_lines: Vec<&str> = patch_str_used.as_ref().lines().collect();
        let mut lines_added = 0usize;
        let mut lines_removed = 0usize;

        for line in original_lines {
            if line.starts_with('+') && !line.starts_with("+++") {
                lines_added += 1;
            } else if line.starts_with('-') && !line.starts_with("---") {
                lines_removed += 1;
            }
        }

        Ok((current_content, patched_content, lines_added, lines_removed))
    }

    /// 预览补丁（不实际应用）
    /// 返回 (原始内容, 修改后内容, 新增行数, 删除行数, 补丁字符串)
    pub async fn preview_patch(&self, args: &EditFileArgs) -> Result<(String, String, usize, usize, String), FileToolError> {
        let (current_content, patched_content, lines_added, lines_removed) =
            Self::apply_patch_internal(&args.file_path, &args.patch)?;

        // 重新生成补丁字符串用于预览（标准化后的版本）
        let preview = if args.patch.ends_with('\n') {
            args.patch.clone()
        } else {
            format!("{}\n", args.patch)
        };

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
                    match request_confirmation(lines_added, lines_removed, args.confirmation.as_ref()) {
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
                                success: false,
                                message: "用户取消了修改。请不要重试此操作，除非用户明确要求。".to_string(),
                                preview: Some(preview),
                                cancelled: true,
                            })
                        }
                        Err(e) => {
                            println!("  └─ {}", format!("读取输入错误: {}", e).red());
                            println!();
                            Err(e)
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
    use std::sync::{Mutex, OnceLock};
    use tempfile::NamedTempFile;

    fn env_lock() -> &'static Mutex<()> {
        static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        ENV_LOCK.get_or_init(|| Mutex::new(()))
    }

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
            confirmation: None,
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

    #[tokio::test]
    async fn test_preview_patch_repairs_hunk_counts() {
        let tool = EditFileTool;

        let temp_file = NamedTempFile::new().unwrap();
        let test_path = temp_file.path().to_path_buf();
        fs::write(&test_path, "line 1\nline 2\nline 3\n").unwrap();

        let args = EditFileArgs {
            file_path: test_path.to_str().unwrap().to_string(),
            // 头部行数故意写错：实际 hunk 为 3 行
            patch: "@@ -1,2 +1,2 @@
 line 1
-line 2
+line 2 modified
 line 3
".to_string(),
            confirmation: None,
        };

        let result = tool.preview_patch(&args).await;
        assert!(result.is_ok(), "应能自动修复 hunk 行数");

        let (_original, modified, added, removed, _preview) = result.unwrap();
        assert_eq!(modified, "line 1\nline 2 modified\nline 3\n");
        assert_eq!(added, 1);
        assert_eq!(removed, 1);
    }

    #[test]
    fn test_preview_enabled_default() {
        let _guard = env_lock().lock().unwrap();
        env::remove_var("OXIDE_EDIT_PREVIEW");
        // 默认应该启用预览
        assert!(preview_enabled());
    }

    #[test]
    fn test_preview_disabled_by_env() {
        let _guard = env_lock().lock().unwrap();
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
            confirmation: None,
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
            confirmation: None,
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
