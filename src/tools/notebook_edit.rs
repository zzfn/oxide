//! NotebookEdit 工具
//!
//! 编辑 Jupyter notebook 文件。

#![allow(dead_code)]

use super::FileToolError;
use colored::*;
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Notebook 单元类型
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CellType {
    Code,
    Markdown,
}

/// Notebook 单元
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NotebookCell {
    /// 单元类型
    pub cell_type: CellType,

    /// 单元内容
    pub source: String,

    /// 执行次数(仅代码单元)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_count: Option<u32>,

    /// 输出(仅代码单元)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outputs: Option<Vec<serde_json::Value>>,
}

impl NotebookCell {
    /// 创建新的代码单元
    pub fn new_code(source: String) -> Self {
        Self {
            cell_type: CellType::Code,
            source,
            execution_count: None,
            outputs: None,
        }
    }

    /// 创建新的 Markdown 单元
    pub fn new_markdown(source: String) -> Self {
        Self {
            cell_type: CellType::Markdown,
            source,
            execution_count: None,
            outputs: None,
        }
    }
}

/// NotebookEdit 工具输入参数
#[derive(Deserialize)]
pub struct NotebookEditArgs {
    /// notebook 文件路径
    pub notebook_path: String,

    /// 要编辑的单元索引
    pub cell_index: usize,

    /// 单元 ID (可选,用于指定特定单元)
    #[serde(default)]
    pub cell_id: Option<String>,

    /// 新的单元内容
    pub new_source: String,

    /// 编辑模式
    #[serde(default)]
    pub edit_mode: String, // "replace", "insert", "delete"

    /// 单元类型(用于 insert 模式)
    #[serde(default)]
    pub cell_type: Option<String>,
}

/// NotebookEdit 工具输出
#[derive(Serialize, Deserialize, Debug)]
pub struct NotebookEditOutput {
    /// notebook 文件路径
    pub notebook_path: String,

    /// 是否成功
    pub success: bool,

    /// 消息
    pub message: String,

    /// 修改的单元索引
    pub cell_index: usize,

    /// 编辑模式
    pub edit_mode: String,

    /// notebook 中的单元总数
    pub total_cells: usize,
}

/// Jupyter Notebook 结构
#[derive(Debug, Deserialize, Serialize)]
struct JupyterNotebook {
    #[serde(rename = "nbformat")]
    format_version: u32,
    #[serde(rename = "nbformat_minor")]
    format_minor: u32,
    cells: Vec<serde_json::Value>,
}

/// NotebookEdit 工具
#[derive(Deserialize, Serialize)]
pub struct NotebookEditTool;

impl NotebookEditTool {
    /// 读取 notebook 文件
    fn read_notebook(path: &str) -> Result<JupyterNotebook, FileToolError> {
        let content = fs::read_to_string(path)?;

        serde_json::from_str(&content).map_err(|e| {
            FileToolError::InvalidInput(format!("无法解析 notebook 文件: {}", e))
        })
    }

    /// 写入 notebook 文件
    fn write_notebook(path: &str, notebook: &JupyterNotebook) -> Result<(), FileToolError> {
        let json = serde_json::to_string_pretty(notebook).map_err(|e| {
            FileToolError::InvalidInput(format!("序列化 notebook 失败: {}", e))
        })?;

        fs::write(path, json)?;
        Ok(())
    }

    /// 从 JSON 提取单元格类型
    fn extract_cell_type(cell_value: &serde_json::Value) -> Result<CellType, FileToolError> {
        cell_value
            .get("cell_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| FileToolError::InvalidInput("单元格缺少 cell_type 字段".to_string()))
            .and_then(|t| match t {
                "code" => Ok(CellType::Code),
                "markdown" => Ok(CellType::Markdown),
                _ => Err(FileToolError::InvalidInput(format!("未知的单元格类型: {}", t))),
            })
    }

    /// 从 JSON 提取源代码
    fn extract_cell_source(cell_value: &serde_json::Value) -> Result<String, FileToolError> {
        cell_value
            .get("source")
            .ok_or_else(|| FileToolError::InvalidInput("单元格缺少 source 字段".to_string()))
            .and_then(|v| {
                if v.is_string() {
                    Ok(v.as_str().unwrap().to_string())
                } else if v.is_array() {
                    // Jupyter 格式中 source 可能是字符串数组
                    let lines: Vec<&str> = v
                        .as_array()
                        .unwrap()
                        .iter()
                        .filter_map(|line| line.as_str())
                        .collect();

                    Ok(lines.join(""))
                } else {
                    Err(FileToolError::InvalidInput(
                        "source 字段格式无效".to_string(),
                    ))
                }
            })
    }

    /// 将源代码转换为 Jupyter 格式
    fn source_to_jupyter_format(source: &str) -> serde_json::Value {
        // 如果源代码包含换行符,将其拆分为数组
        if source.contains('\n') {
            serde_json::Value::Array(
                source
                    .split('\n')
                    .map(|line| serde_json::Value::String(format!("{}\n", line)))
                    .collect(),
            )
        } else {
            serde_json::Value::String(source.to_string())
        }
    }

    /// 替换单元内容
    fn replace_cell(
        notebook: &mut JupyterNotebook,
        cell_index: usize,
        new_source: &str,
    ) -> Result<(), FileToolError> {
        if cell_index >= notebook.cells.len() {
            return Err(FileToolError::InvalidInput(format!(
                "单元索引 {} 超出范围 (总共 {} 个单元)",
                cell_index,
                notebook.cells.len()
            )));
        }

        let cell = notebook
            .cells
            .get_mut(cell_index)
            .ok_or_else(|| FileToolError::InvalidInput("无法获取单元格".to_string()))?;

        // 更新 source 字段
        if let Some(source_obj) = cell.get_mut("source") {
            *source_obj = Self::source_to_jupyter_format(new_source);
        }

        Ok(())
    }

    /// 插入新单元
    fn insert_cell(
        notebook: &mut JupyterNotebook,
        cell_index: usize,
        new_source: &str,
        cell_type: &str,
    ) -> Result<(), FileToolError> {
        if cell_index > notebook.cells.len() {
            return Err(FileToolError::InvalidInput(format!(
                "单元索引 {} 超出范围 (总共 {} 个单元)",
                cell_index,
                notebook.cells.len()
            )));
        }

        let cell_type = match cell_type.to_lowercase().as_str() {
            "code" => CellType::Code,
            "markdown" => CellType::Markdown,
            _ => {
                return Err(FileToolError::InvalidInput(format!(
                    "未知的单元类型: {}",
                    cell_type
                )))
            }
        };

        let new_cell = serde_json::json!({
            "cell_type": if matches!(cell_type, CellType::Code) { "code" } else { "markdown" },
            "source": Self::source_to_jupyter_format(new_source),
            "metadata": {},
        });

        notebook.cells.insert(cell_index, new_cell);

        Ok(())
    }

    /// 删除单元
    fn delete_cell(notebook: &mut JupyterNotebook, cell_index: usize) -> Result<(), FileToolError> {
        if cell_index >= notebook.cells.len() {
            return Err(FileToolError::InvalidInput(format!(
                "单元索引 {} 超出范围 (总共 {} 个单元)",
                cell_index,
                notebook.cells.len()
            )));
        }

        notebook.cells.remove(cell_index);

        Ok(())
    }
}

impl Tool for NotebookEditTool {
    const NAME: &'static str = "notebook_edit";

    type Error = FileToolError;
    type Args = NotebookEditArgs;
    type Output = NotebookEditOutput;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "notebook_edit".to_string(),
            description: "Edit Jupyter notebook (.ipynb) files. This tool allows you to replace, insert, or delete cells in a notebook. Cell indices are 0-based. The tool preserves the notebook structure and metadata.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "notebook_path": {
                        "type": "string",
                        "description": "Path to the .ipynb file"
                    },
                    "cell_index": {
                        "type": "number",
                        "description": "Index of the cell to edit (0-based)"
                    },
                    "cell_id": {
                        "type": "string",
                        "description": "Optional cell ID for more precise targeting"
                    },
                    "new_source": {
                        "type": "string",
                        "description": "New cell content (for replace/insert modes)"
                    },
                    "edit_mode": {
                        "type": "string",
                        "description": "Edit mode: 'replace', 'insert', or 'delete'",
                        "enum": ["replace", "insert", "delete"]
                    },
                    "cell_type": {
                        "type": "string",
                        "description": "Cell type for insert mode: 'code' or 'markdown'",
                        "enum": ["code", "markdown"]
                    }
                },
                "required": ["notebook_path", "cell_index", "edit_mode"]
            })
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let path = Path::new(&args.notebook_path);

        // 验证文件存在
        if !path.exists() {
            return Err(FileToolError::FileNotFound(args.notebook_path.clone()));
        }

        // 验证文件扩展名
        if path.extension().and_then(|s| s.to_str()) != Some("ipynb") {
            return Err(FileToolError::InvalidInput(
                "文件必须是 .ipynb 格式".to_string(),
            ));
        }

        // 读取 notebook
        let mut notebook = Self::read_notebook(&args.notebook_path)?;
        let _original_cell_count = notebook.cells.len();

        // 执行编辑操作
        let result = match args.edit_mode.as_str() {
            "replace" => {
                if args.new_source.is_empty() {
                    return Err(FileToolError::InvalidInput(
                        "replace 模式需要提供 new_source".to_string(),
                    ));
                }
                Self::replace_cell(&mut notebook, args.cell_index, &args.new_source)
            }
            "insert" => {
                if args.new_source.is_empty() {
                    return Err(FileToolError::InvalidInput(
                        "insert 模式需要提供 new_source".to_string(),
                    ));
                }

                let cell_type = args.cell_type.unwrap_or_else(|| "code".to_string());
                Self::insert_cell(&mut notebook, args.cell_index, &args.new_source, &cell_type)
            }
            "delete" => Self::delete_cell(&mut notebook, args.cell_index),
            _ => Err(FileToolError::InvalidInput(format!(
                "未知的编辑模式: {}",
                args.edit_mode
            ))),
        };

        if let Err(e) = result {
            return Err(e);
        }

        // 写回文件
        Self::write_notebook(&args.notebook_path, &notebook)?;

        Ok(NotebookEditOutput {
            notebook_path: args.notebook_path.clone(),
            success: true,
            message: format!(
                "成功 {} 单元 {} (notebook 共有 {} 个单元)",
                if args.edit_mode == "delete" {
                    "删除"
                } else if args.edit_mode == "insert" {
                    "插入"
                } else {
                    "替换"
                },
                args.cell_index,
                notebook.cells.len()
            ),
            cell_index: args.cell_index,
            edit_mode: args.edit_mode.clone(),
            total_cells: notebook.cells.len(),
        })
    }
}

/// NotebookEdit 工具包装器
#[derive(Deserialize, Serialize)]
pub struct WrappedNotebookEditTool {
    inner: NotebookEditTool,
}

impl WrappedNotebookEditTool {
    pub fn new() -> Self {
        Self {
            inner: NotebookEditTool,
        }
    }
}

impl Default for WrappedNotebookEditTool {
    fn default() -> Self {
        Self::new()
    }
}

impl Tool for WrappedNotebookEditTool {
    const NAME: &'static str = "notebook_edit";

    type Error = FileToolError;
    type Args = <NotebookEditTool as Tool>::Args;
    type Output = <NotebookEditTool as Tool>::Output;

    async fn definition(&self, prompt: String) -> ToolDefinition {
        self.inner.definition(prompt).await
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        println!();
        println!(
            "{} {} ({})",
            "●".bright_green(),
            "NotebookEdit",
            args.notebook_path
        );

        let result = self.inner.call(args).await;

        match &result {
            Ok(output) => {
                println!(
                    "  └─ {}",
                    format!(
                        "成功 {} 单元 {} (总计 {} 个单元)",
                        if output.edit_mode == "delete" {
                            "删除"
                        } else if output.edit_mode == "insert" {
                            "插入"
                        } else {
                            "替换"
                        },
                        output.cell_index.to_string().bright_yellow(),
                        output.total_cells.to_string().bright_green()
                    )
                    .dimmed()
                );
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

    #[test]
    fn test_cell_type_serialization() {
        let code_type = CellType::Code;
        let json = serde_json::to_string(&code_type).unwrap();
        assert_eq!(json, "\"code\"");

        let md_type = CellType::Markdown;
        let json = serde_json::to_string(&md_type).unwrap();
        assert_eq!(json, "\"markdown\"");
    }

    #[test]
    fn test_notebook_cell_creation() {
        let code_cell = NotebookCell::new_code("print('hello')".to_string());
        assert!(matches!(code_cell.cell_type, CellType::Code));
        assert_eq!(code_cell.source, "print('hello')");

        let md_cell = NotebookCell::new_markdown("# Title".to_string());
        assert!(matches!(md_cell.cell_type, CellType::Markdown));
        assert_eq!(md_cell.source, "# Title");
    }

    #[test]
    fn test_notebook_edit_args_deserialization() {
        let json = r#"{
            "notebook_path": "/path/to/notebook.ipynb",
            "cell_index": 0,
            "new_source": "print('updated')",
            "edit_mode": "replace"
        }"#;

        let args: NotebookEditArgs = serde_json::from_str(json).unwrap();
        assert_eq!(args.notebook_path, "/path/to/notebook.ipynb");
        assert_eq!(args.cell_index, 0);
        assert_eq!(args.new_source, "print('updated')");
        assert_eq!(args.edit_mode, "replace");
    }

    #[test]
    fn test_source_to_jupyter_format() {
        // 单行文本应该转为字符串
        let single_line = NotebookEditTool::source_to_jupyter_format("single line");
        assert!(single_line.is_string());

        // 多行文本应该转为数组
        let multi_line = NotebookEditTool::source_to_jupyter_format("line1\nline2");
        assert!(multi_line.is_array());
    }

    #[test]
    fn test_notebook_edit_output_serialization() {
        let output = NotebookEditOutput {
            notebook_path: "/path/to/notebook.ipynb".to_string(),
            success: true,
            message: "成功替换单元 0".to_string(),
            cell_index: 0,
            edit_mode: "replace".to_string(),
            total_cells: 5,
        };

        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("notebook_path"));
        assert!(json.contains("success"));

        let deserialized: NotebookEditOutput = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.cell_index, 0);
        assert_eq!(deserialized.total_cells, 5);
        assert!(deserialized.success);
    }
}
