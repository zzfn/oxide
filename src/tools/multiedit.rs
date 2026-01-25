//! MultiEdit 工具
//!
//! 批量编辑多个文件。

#![allow(dead_code)]

use super::{edit_file::EditFileTool, FileToolError};
use colored::*;
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};

/// 单个编辑操作
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EditOperation {
    /// 文件路径
    pub file_path: String,

    /// Unified diff patch
    pub patch: String,
}

/// MultiEdit 工具输入参数
#[derive(Deserialize)]
pub struct MultiEditArgs {
    /// 编辑操作列表
    pub edits: Vec<EditOperation>,
}

/// 单个文件编辑结果
#[derive(Serialize, Deserialize, Debug)]
pub struct EditResult {
    /// 文件路径
    pub file_path: String,

    /// 是否成功
    pub success: bool,

    /// 消息
    pub message: String,

    /// 添加的行数
    pub lines_added: Option<usize>,

    /// 删除的行数
    pub lines_removed: Option<usize>,

    /// 错误信息(如果失败)
    pub error: Option<String>,
}

/// MultiEdit 工具输出
#[derive(Serialize, Deserialize, Debug)]
pub struct MultiEditOutput {
    /// 总操作数
    pub total_operations: usize,

    /// 成功操作数
    pub successful_operations: usize,

    /// 失败操作数
    pub failed_operations: usize,

    /// 详细结果
    pub results: Vec<EditResult>,

    /// 整体成功状态
    pub success: bool,

    /// 总结消息
    pub summary: String,
}

/// MultiEdit 工具
#[derive(Deserialize, Serialize)]
pub struct MultiEditTool {
    /// 内部 Edit 工具
    edit_tool: EditFileTool,
}

impl MultiEditTool {
    /// 创建新的 MultiEdit 工具
    pub fn new() -> Self {
        Self {
            edit_tool: EditFileTool,
        }
    }

    /// 验证编辑操作
    fn validate_operation(operation: &EditOperation) -> Result<(), FileToolError> {
        if operation.file_path.is_empty() {
            return Err(FileToolError::InvalidInput(
                "file_path 不能为空".to_string(),
            ));
        }

        if operation.patch.is_empty() {
            return Err(FileToolError::InvalidInput(
                "patch 不能为空".to_string(),
            ));
        }

        Ok(())
    }
}

impl Default for MultiEditTool {
    fn default() -> Self {
        Self::new()
    }
}

impl Tool for MultiEditTool {
    const NAME: &'static str = "multi_edit";

    type Error = FileToolError;
    type Args = MultiEditArgs;
    type Output = MultiEditOutput;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "multi_edit".to_string(),
            description: "Apply multiple edit operations to multiple files in one call. This is useful when you need to make coordinated changes across several files. Each edit operation is independent and will be applied sequentially. If any edit fails, the overall operation will continue and report all successes and failures.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "edits": {
                        "type": "array",
                        "description": "List of edit operations to apply",
                        "items": {
                            "type": "object",
                            "properties": {
                                "file_path": {
                                    "type": "string",
                                    "description": "The path to the file to edit"
                                },
                                "patch": {
                                    "type": "string",
                                    "description": "A unified diff patch string in standard format"
                                }
                            },
                            "required": ["file_path", "patch"]
                        }
                    }
                },
                "required": ["edits"]
            })
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let total = args.edits.len();
        let mut results = Vec::with_capacity(total);
        let mut successful = 0usize;
        let mut failed = 0usize;

        // 逐个应用编辑操作
        for (_index, operation) in args.edits.iter().enumerate() {
            // 验证操作
            if let Err(e) = Self::validate_operation(operation) {
                results.push(EditResult {
                    file_path: operation.file_path.clone(),
                    success: false,
                    message: format!("验证失败: {}", e),
                    lines_added: None,
                    lines_removed: None,
                    error: Some(e.to_string()),
                });
                failed += 1;
                continue;
            }

            // 检查文件是否存在
            if !std::path::Path::new(&operation.file_path).exists() {
                results.push(EditResult {
                    file_path: operation.file_path.clone(),
                    success: false,
                    message: "文件不存在".to_string(),
                    lines_added: None,
                    lines_removed: None,
                    error: Some(format!("文件 '{}' 不存在", operation.file_path)),
                });
                failed += 1;
                continue;
            }

            // 应用编辑
            let edit_args = super::edit_file::EditFileArgs {
                file_path: operation.file_path.clone(),
                patch: operation.patch.clone(),
                confirmation: None,
            };

            match self.edit_tool.call(edit_args).await {
                Ok(output) => {
                    successful += 1;
                    results.push(EditResult {
                        file_path: operation.file_path.clone(),
                        success: true,
                        message: output.message.clone(),
                        lines_added: Some(output.lines_added),
                        lines_removed: Some(output.lines_removed),
                        error: None,
                    });
                }
                Err(e) => {
                    failed += 1;
                    results.push(EditResult {
                        file_path: operation.file_path.clone(),
                        success: false,
                        message: "编辑失败".to_string(),
                        lines_added: None,
                        lines_removed: None,
                        error: Some(e.to_string()),
                    });
                }
            }
        }

        let overall_success = failed == 0;
        let summary = if overall_success {
            format!(
                "成功编辑 {}/{} 个文件",
                successful,
                total
            )
        } else {
            format!(
                "完成 {}/{} 个文件编辑，{} 个失败",
                successful,
                total,
                failed
            )
        };

        Ok(MultiEditOutput {
            total_operations: total,
            successful_operations: successful,
            failed_operations: failed,
            results,
            success: overall_success,
            summary,
        })
    }
}

/// MultiEdit 工具包装器
#[derive(Deserialize, Serialize)]
pub struct WrappedMultiEditTool {
    inner: MultiEditTool,
}

impl WrappedMultiEditTool {
    pub fn new() -> Self {
        Self {
            inner: MultiEditTool::new(),
        }
    }
}

impl Default for WrappedMultiEditTool {
    fn default() -> Self {
        Self::new()
    }
}

impl Tool for WrappedMultiEditTool {
    const NAME: &'static str = "multi_edit";

    type Error = FileToolError;
    type Args = <MultiEditTool as Tool>::Args;
    type Output = <MultiEditTool as Tool>::Output;

    async fn definition(&self, prompt: String) -> ToolDefinition {
        self.inner.definition(prompt).await
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        println!();
        println!(
            "{} {} ({} 文件)",
            "●".bright_green(),
            "MultiEdit",
            args.edits.len()
        );

        let result = self.inner.call(args).await;

        match &result {
            Ok(output) => {
                println!(
                    "  └─ {}",
                    format!(
                        "完成: {} 成功, {} 失败",
                        output.successful_operations.to_string().green(),
                        output.failed_operations.to_string().red()
                    )
                    .dimmed()
                );

                // 显示详细结果
                for edit_result in &output.results {
                    if edit_result.success {
                        println!(
                            "    ✓ {} (+{} lines, -{} lines)",
                            edit_result.file_path.bright_green(),
                            edit_result
                                .lines_added
                                .unwrap_or(0)
                                .to_string()
                                .green(),
                            edit_result
                                .lines_removed
                                .unwrap_or(0)
                                .to_string()
                                .red()
                        );
                    } else {
                        println!(
                            "    ✗ {} - {}",
                            edit_result.file_path.bright_red(),
                            edit_result.error.as_ref().unwrap_or(&"未知错误".to_string()).red()
                        );
                    }
                }
            }
            Err(e) => {
                println!("  └─ {}", format!("错误: {}", e).red());
            }
        }
        println!();

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::write;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_edit_operation_serialization() {
        let operation = EditOperation {
            file_path: "/path/to/file.rs".to_string(),
            patch: "@@ -1,1 +1,1 @@\n-old\n+new\n".to_string(),
        };

        let json = serde_json::to_string(&operation).unwrap();
        assert!(json.contains("file_path"));
        assert!(json.contains("patch"));

        let deserialized: EditOperation = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.file_path, operation.file_path);
        assert_eq!(deserialized.patch, operation.patch);
    }

    #[test]
    fn test_multiedit_args_deserialization() {
        let json = r#"{
            "edits": [
                {
                    "file_path": "/path/to/file1.rs",
                    "patch": "@@ -1,1 +1,1 @@\n-old1\n+new1\n"
                },
                {
                    "file_path": "/path/to/file2.rs",
                    "patch": "@@ -1,1 +1,1 @@\n-old2\n+new2\n"
                }
            ]
        }"#;

        let args: MultiEditArgs = serde_json::from_str(json).unwrap();
        assert_eq!(args.edits.len(), 2);
        assert_eq!(args.edits[0].file_path, "/path/to/file1.rs");
        assert_eq!(args.edits[1].file_path, "/path/to/file2.rs");
    }

    #[test]
    fn test_validate_operation() {
        let valid_operation = EditOperation {
            file_path: "/path/to/file.rs".to_string(),
            patch: "@@ -1,1 +1,1 @@\n-old\n+new\n".to_string(),
        };
        assert!(MultiEditTool::validate_operation(&valid_operation).is_ok());

        let empty_file_path = EditOperation {
            file_path: String::new(),
            patch: "@@ -1,1 +1,1 @@\n-old\n+new\n".to_string(),
        };
        assert!(MultiEditTool::validate_operation(&empty_file_path).is_err());

        let empty_patch = EditOperation {
            file_path: "/path/to/file.rs".to_string(),
            patch: String::new(),
        };
        assert!(MultiEditTool::validate_operation(&empty_patch).is_err());
    }

    #[test]
    fn test_multiedit_output_serialization() {
        let output = MultiEditOutput {
            total_operations: 3,
            successful_operations: 2,
            failed_operations: 1,
            results: vec![],
            success: false,
            summary: "完成 2/3 个文件编辑，1 个失败".to_string(),
        };

        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("total_operations"));
        assert!(json.contains("successful_operations"));
        assert!(json.contains("failed_operations"));

        let deserialized: MultiEditOutput = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.total_operations, 3);
        assert_eq!(deserialized.successful_operations, 2);
        assert_eq!(deserialized.failed_operations, 1);
    }

    #[test]
    fn test_multiedit_tool_creation() {
        let _tool = MultiEditTool::new();
        let _default_tool = MultiEditTool::default();

        // 两者都应该有效创建
        assert_eq!(MultiEditTool::NAME, "multi_edit");
    }

    #[test]
    fn test_wrapped_multiedit_tool_creation() {
        let _wrapped = WrappedMultiEditTool::new();
        let _default_wrapped = WrappedMultiEditTool::default();

        // WrappedMultiEditTool 应该能成功创建
        // 内部工具有 NAME 常量
        assert_eq!(MultiEditTool::NAME, "multi_edit");
    }
}
