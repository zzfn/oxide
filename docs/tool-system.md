# 工具系统实现详解

## 目录

- [系统概述](#系统概述)
- [Tool Trait](#tool-trait)
- [文件操作工具](#文件操作工具)
- [搜索工具](#搜索工具)
- [系统工具](#系统工具)
- [工具注册](#工具注册)
- [参数验证](#参数验证)
- [错误处理](#错误处理)
- [使用指南](#使用指南)
- [扩展开发](#扩展开发)

## 系统概述

Oxide 的工具系统是一个类型安全、高性能的工具调用框架，为 AI Agent 提供与文件系统、Shell 和代码库交互的能力。系统采用统一的 `Tool` trait 接口，支持异步执行、错误处理和可视化反馈。

### 核心特性

- **统一接口**: 所有工具实现相同的 `Tool` trait
- **类型安全**: 使用 Rust 类型系统确保参数正确性
- **异步执行**: 所有工具操作都是异步的
- **错误处理**: 统一的错误类型和详细的错误信息
- **可视化反馈**: 包装器模式提供彩色输出和进度显示
- **权限控制**: 细粒度的工具权限管理

## Tool Trait

### 核心定义

工具系统基于 `rig-core` crate 的 `Tool` trait：

```rust
use rig::tool::Tool;

pub trait Tool {
    /// 工具名称
    const NAME: &'static str;

    /// 错误类型
    type Error: std::error::Error + Send + Sync;

    /// 输入参数类型（必须实现 Serialize）
    type Args: Serialize;

    /// 输出结果类型（必须实现 Serialize）
    type Output: Serialize;

    /// 生成工具定义（用于 LLM 理解）
    async fn definition(&self, prompt: String) -> ToolDefinition;

    /// 执行工具
    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error>;
}
```

### 工具定义

`ToolDefinition` 包含工具的元数据，遵循 OpenAI Function Calling 规范：

```rust
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,  // JSON Schema
}
```

示例（ReadFileTool）：

```json
{
  "name": "read_file",
  "description": "Read the contents of a file",
  "parameters": {
    "type": "object",
    "properties": {
      "file_path": {
        "type": "string",
        "description": "The path to the file to read"
      }
    },
    "required": ["file_path"]
  }
}
```

## 文件操作工具

### ReadFileTool

读取文件内容，支持多种文件格式。

**参数**:
```rust
pub struct ReadFileArgs {
    pub file_path: String,
}
```

**输出**:
```rust
pub struct ReadFileOutput {
    pub content: String,    // 文件内容
    pub path: String,       // 文件路径
    pub size: usize,        // 字节数
    pub lines: usize,       // 行数
    pub status: String,     // 状态（success/error）
}
```

**实现**:
```rust
pub struct ReadFileTool;

#[async_trait]
impl Tool for ReadFileTool {
    const NAME: &'static str = "read_file";

    type Error = FileToolError;
    type Args = ReadFileArgs;
    type Output = ReadFileOutput;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "read_file".to_string(),
            description: "Read the contents of a file".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "The path to the file to read"
                    }
                },
                "required": ["file_path"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // 检查文件是否存在
        let path = Path::new(&args.file_path);
        if !path.exists() {
            return Err(FileToolError::FileNotFound(args.file_path));
        }

        // 检查是否为文件
        if !path.is_file() {
            return Err(FileToolError::NotAFile(args.file_path));
        }

        // 读取文件内容
        let content = fs::read_to_string(path)?;

        // 统计信息
        let size = content.len();
        let lines = content.lines().count();

        Ok(ReadFileOutput {
            content,
            path: args.file_path,
            size,
            lines,
            status: "success".to_string(),
        })
    }
}
```

### WriteFileTool

写入文件内容，自动创建不存在的目录。

**参数**:
```rust
pub struct WriteFileArgs {
    pub file_path: String,
    pub content: String,
}
```

**输出**:
```rust
pub struct WriteFileOutput {
    pub path: String,
    pub bytes_written: usize,
    pub lines_written: usize,
    pub status: String,
}
```

**实现**:
```rust
pub struct WriteFileTool;

#[async_trait]
impl Tool for WriteFileTool {
    const NAME: &'static str = "write_file";

    type Error = FileToolError;
    type Args = WriteFileArgs;
    type Output = WriteFileOutput;

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let path = Path::new(&args.file_path);

        // 创建父目录（如果不存在）
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        // 写入文件
        fs::write(&path, args.content)?;

        // 统计信息
        let bytes_written = args.content.len();
        let lines_written = args.content.lines().count();

        Ok(WriteFileOutput {
            path: args.file_path,
            bytes_written,
            lines_written,
            status: "success".to_string(),
        })
    }
}
```

### EditFileTool

使用 unified diff patch 编辑文件。

**参数**:
```rust
pub struct EditFileArgs {
    pub file_path: String,
    pub patch: String,  // Unified diff 格式
}
```

**输出**:
```rust
pub struct EditFileOutput {
    pub path: String,
    pub lines_added: usize,
    pub lines_removed: usize,
    pub status: String,
}
```

**实现**:
```rust
use patch::{Patch, PatchList};

pub struct EditFileTool;

#[async_trait]
impl Tool for EditFileTool {
    const NAME: &'static str = "edit_file";

    type Error = FileToolError;
    type Args = EditFileArgs;
    type Output = EditFileOutput;

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // 读取原始文件
        let original_content = fs::read_to_string(&args.file_path)?;

        // 解析 patch
        let patch: PatchList = Patch::from_multiple(&args.patch)
            .map_err(|e| FileToolError::InvalidInput(format!("Invalid patch: {}", e)))?;

        // 应用 patch
        let patched_content = patch
            .apply(&original_content)
            .map_err(|e| FileToolError::InvalidInput(format!("Failed to apply patch: {}", e)))?;

        // 写回文件
        fs::write(&args.file_path, patched_content)?;

        // 统计变更
        let lines_added = args.patch.lines().filter(|l| l.starts_with('+')).count();
        let lines_removed = args.patch.lines().filter(|l| l.starts_with('-')).count();

        Ok(EditFileOutput {
            path: args.file_path,
            lines_added,
            lines_removed,
            status: "success".to_string(),
        })
    }
}
```

**Patch 格式示例**:
```diff
--- a/main.rs
+++ b/main.rs
@@ -10,7 +10,9 @@
 fn main() {
     let x = 5;
     let y = 10;
+    // Calculate sum
     let sum = x + y;
+    println!("Sum: {}", sum);
 }
```

### DeleteFileTool

删除文件或目录。

**参数**:
```rust
pub struct DeleteFileArgs {
    pub path: String,
}
```

**输出**:
```rust
pub struct DeleteFileOutput {
    pub path: String,
    pub deleted: bool,
    pub status: String,
}
```

### CreateDirectoryTool

创建目录（包括父目录）。

**参数**:
```rust
pub struct CreateDirectoryArgs {
    pub path: String,
}
```

**输出**:
```rust
pub struct CreateDirectoryOutput {
    pub path: String,
    pub created: bool,
    pub status: String,
}
```

## 搜索工具

### GrepSearchTool

使用正则表达式搜索文件内容。

**参数**:
```rust
pub struct GrepSearchArgs {
    pub root_path: String,
    pub query: String,
    pub max_results: Option<usize>,
}
```

**输出**:
```rust
pub struct GrepSearchOutput {
    pub matches: Vec<Match>,
    pub total_matches: usize,
    pub files_searched: usize,
    pub status: String,
}

pub struct Match {
    pub file: String,
    pub line: usize,
    pub content: String,
}
```

**实现**:
```rust
use ignore::{Walk, WalkBuilder};
use regex::Regex;

pub struct GrepSearchTool;

#[async_trait]
impl Tool for GrepSearchTool {
    const NAME: &'static str = "grep_search";

    type Error = SearchError;
    type Args = GrepSearchArgs;
    type Output = GrepSearchOutput;

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // 编译正则表达式
        let regex = Regex::new(&args.query)
            .map_err(|e| SearchError::InvalidRegex(e.to_string()))?;

        let mut matches = Vec::new();

        // 遍历文件
        let walker = WalkBuilder::new(&args.root_path)
            .git_ignore(true)  // 尊重 .gitignore
            .build();

        for entry in walker {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                // 读取文件内容
                if let Ok(content) = fs::read_to_string(path) {
                    // 搜索匹配
                    for (line_num, line) in content.lines().enumerate() {
                        if regex.is_match(line) {
                            matches.push(Match {
                                file: path.to_string_lossy().to_string(),
                                line: line_num + 1,
                                content: line.to_string(),
                            });

                            // 限制结果数量
                            if let Some(max) = args.max_results {
                                if matches.len() >= max {
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(GrepSearchOutput {
            matches,
            total_matches: matches.len(),
            files_searched: walker.into_iter().count(),
            status: "success".to_string(),
        })
    }
}
```

### GlobTool

使用通配符模式匹配文件。

**参数**:
```rust
pub struct GlobArgs {
    pub pattern: String,
    pub search_path: Option<String>,
}
```

**输出**:
```rust
pub struct GlobOutput {
    pub matches: Vec<String>,
    pub count: usize,
    pub status: String,
}
```

**实现**:
```rust
use glob::glob;

pub struct GlobTool;

#[async_trait]
impl Tool for GlobTool {
    const NAME: &'static str = "glob";

    type Error = SearchError;
    type Args = GlobArgs;
    type Output = GlobOutput;

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let search_path = args.search_path.unwrap_or_else(|| ".".to_string());
        let full_pattern = Path::new(&search_path).join(&args.pattern);

        let mut matches = Vec::new();

        for entry in glob(full_pattern.to_str().unwrap())
            .map_err(|e| SearchError::InvalidPattern(e.to_string()))?
        {
            match entry {
                Ok(path) => {
                    matches.push(path.to_string_lossy().to_string());
                }
                Err(e) => {
                    eprintln!("Error reading path: {:?}", e);
                }
            }
        }

        // 排序结果
        matches.sort();

        Ok(GlobOutput {
            matches,
            count: matches.len(),
            status: "success".to_string(),
        })
    }
}
```

**模式示例**:
- `**/*.rs` - 所有 Rust 文件
- `src/**/*.rs` - src 目录下所有 Rust 文件
- `**/test_*.py` - 所有以 test_ 开头的 Python 文件

### ScanCodebaseTool

扫描并显示代码库目录结构。

**参数**:
```rust
pub struct ScanCodebaseArgs {
    pub root_path: Option<String>,
    pub max_depth: Option<usize>,
}
```

**输出**:
```rust
pub struct ScanCodebaseOutput {
    pub tree: String,  // 目录树（ASCII 艺术）
    pub total_files: usize,
    pub total_dirs: usize,
    pub status: String,
}
```

## 系统工具

### ShellExecuteTool

执行 Shell 命令。

**参数**:
```rust
pub struct ShellExecuteArgs {
    pub command: String,
}
```

**输出**:
```rust
pub struct ShellExecuteOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub status: String,
}
```

**实现**:
```rust
use tokio::process::Command;

pub struct ShellExecuteTool;

#[async_trait]
impl Tool for ShellExecuteTool {
    const NAME: &'static str = "shell_execute";

    type Error = ShellError;
    type Args = ShellExecuteArgs;
    type Output = ShellExecuteOutput;

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // Git 安全检查
        if args.command.starts_with("git") {
            // 防止破坏性 git 命令
            if args.command.contains("git push --force") ||
               args.command.contains("git reset --hard") {
                return Err(ShellError::DangerousCommand(
                    "Force push and hard reset are not allowed".to_string()
                ));
            }
        }

        // 选择 shell（跨平台）
        let (shell, shell_arg) = if cfg!(windows) {
            ("cmd", "/C")
        } else {
            ("sh", "-c")
        };

        // 执行命令
        let output = Command::new(shell)
            .arg(shell_arg)
            .arg(&args.command)
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code().unwrap_or(-1);

        Ok(ShellExecuteOutput {
            stdout,
            stderr,
            exit_code,
            status: if output.status.success() {
                "success".to_string()
            } else {
                "failed".to_string()
            },
        })
    }
}
```

## 工具注册

### 包装器模式

所有工具都使用包装器模式添加可视化反馈：

```rust
pub struct WrappedReadFileTool {
    inner: ReadFileTool,
}

impl WrappedReadFileTool {
    pub fn new() -> Self {
        Self {
            inner: ReadFileTool,
        }
    }
}

#[async_trait]
impl Tool for WrappedReadFileTool {
    const NAME: &'static str = "read_file";

    type Error = FileToolError;
    type Args = ReadFileArgs;
    type Output = ReadFileOutput;

    async fn definition(&self, prompt: String) -> ToolDefinition {
        self.inner.definition(prompt).await
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // 显示工具调用
        println!(
            "{} {}({})",
            "●".bright_green(),
            "Read".bold(),
            args.file_path
        );

        // 执行实际操作
        let result = self.inner.call(args).await;

        // 显示结果
        match &result {
            Ok(output) => println!(
                "{} {} → {} bytes, {} lines",
                "✓".green(),
                "Success".green(),
                output.size,
                output.lines
            ),
            Err(e) => println!("{} {}", "✗".red(), e),
        }

        result
    }
}
```

### 工具集合

```rust
pub struct AllTools {
    pub read_file: WrappedReadFileTool,
    pub write_file: WrappedWriteFileTool,
    pub edit_file: WrappedEditFileTool,
    pub delete_file: WrappedDeleteFileTool,
    pub create_directory: WrappedCreateDirectoryTool,
    pub shell_execute: WrappedShellExecuteTool,
    pub grep_search: WrappedGrepSearchTool,
    pub glob: WrappedGlobTool,
    pub scan_codebase: WrappedScanCodebaseTool,
}
```

### Agent 注册

```rust
impl AgentBuilder {
    fn build_main(&self) -> Result<AgentEnum> {
        let tools = self.create_tools();

        let agent = Agent::builder(client, model)
            .preamble(MAIN_AGENT_PROMPT)
            .tool(tools.read_file)
            .tool(tools.write_file)
            .tool(tools.edit_file)
            .tool(tools.delete_file)
            .tool(tools.create_directory)
            .tool(tools.shell_execute)
            .tool(tools.grep_search)
            .tool(tools.glob)
            .tool(tools.scan_codebase)
            .build();

        Ok(agent)
    }
}
```

## 参数验证

### 类型安全

使用 Rust 类型系统确保参数类型正确：

```rust
#[derive(Serialize, Deserialize, Debug)]
pub struct ReadFileArgs {
    #[serde(validate = "validate_path")]
    pub file_path: String,
}

fn validate_path(path: &str) -> Result<(), String> {
    if path.is_empty() {
        return Err("Path cannot be empty".to_string());
    }
    if path.contains("..") {
        return Err("Path traversal not allowed".to_string());
    }
    Ok(())
}
```

### 运行时验证

```rust
impl ReadFileTool {
    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // 验证路径格式
        let path = Path::new(&args.file_path);
        if !path.is_absolute() {
            return Err(FileToolError::InvalidInput(
                "Path must be absolute".to_string()
            ));
        }

        // 验证文件存在
        if !path.exists() {
            return Err(FileToolError::FileNotFound(args.file_path));
        }

        // 验证是文件而非目录
        if !path.is_file() {
            return Err(FileToolError::NotAFile(args.file_path));
        }

        // 继续处理...
    }
}
```

## 错误处理

### 统一错误类型

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FileToolError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Path is not a file: {0}")]
    NotAFile(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}
```

### 错误传播

```rust
async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
    let content = fs::read_to_string(&args.file_path)?;  // 自动转换 IO 错误
    Ok(ReadFileOutput { content, ... })
}
```

## 使用指南

### LLM 调用工具

LLM 通过以下步骤调用工具：

1. **理解工具定义**: LLM 接收工具的 JSON Schema 定义
2. **生成参数**: LLM 根据用户输入生成工具参数
3. **执行工具**: 系统执行工具并返回结果
4. **处理结果**: LLM 根据工具结果生成最终响应

示例对话：

```
用户: 帮我读取 src/main.rs 文件

LLM: 我来帮你读取文件。

[工具调用] read_file({
  "file_path": "src/main.rs"
})

[工具返回] {
  "content": "fn main() { ... }",
  "size": 1024,
  "lines": 42
}

LLM: 文件内容如下：
fn main() {
    ...
}
```

### 权限控制

不同的 Agent 类型获得不同的工具权限：

```rust
impl AgentBuilder {
    fn create_tools(&self, agent_type: AgentType) -> Result<Box<dyn ToolSet>> {
        match agent_type {
            AgentType::Main => Ok(Box::new(AllTools::new())),
            AgentType::Explore => Ok(Box::new(ReadOnlyTools::new())),
            AgentType::Plan => Ok(Box::new(PlanTools::new())),
            // ...
        }
    }
}
```

## 扩展开发

### 添加新工具

1. **定义工具结构**:

```rust
pub struct MyCustomTool;

#[async_trait]
impl Tool for MyCustomTool {
    const NAME: &'static str = "my_custom_tool";

    type Error = MyCustomError;
    type Args = MyCustomArgs;
    type Output = MyCustomOutput;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "my_custom_tool".to_string(),
            description: "My custom tool description".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "param1": {
                        "type": "string",
                        "description": "First parameter"
                    }
                },
                "required": ["param1"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // 实现工具逻辑
        Ok(MyCustomOutput {
            result: format!("Processed: {}", args.param1)
        })
    }
}
```

2. **创建包装器**:

```rust
pub struct WrappedMyCustomTool {
    inner: MyCustomTool,
}

impl WrappedMyCustomTool {
    pub fn new() -> Self {
        Self {
            inner: MyCustomTool,
        }
    }
}

#[async_trait]
impl Tool for WrappedMyCustomTool {
    const NAME: &'static str = "my_custom_tool";
    type Error = MyCustomError;
    type Args = MyCustomArgs;
    type Output = MyCustomOutput;

    async fn definition(&self, prompt: String) -> ToolDefinition {
        self.inner.definition(prompt).await
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        println!("{} {}", "●".bright_green(), "MyCustom".bold());
        let result = self.inner.call(args).await;
        match &result {
            Ok(_) => println!("{} {}", "✓".green(), "Success".green()),
            Err(e) => println!("{} {}", "✗".red(), e),
        }
        result
    }
}
```

3. **注册到 Agent**:

```rust
impl AgentBuilder {
    fn create_tools(&self) -> AllTools {
        AllTools {
            // ... 现有工具
            my_custom: WrappedMyCustomTool::new(),
        }
    }
}
```

## 最佳实践

### 性能优化

1. **异步操作**: 所有 I/O 操作都应该是异步的
2. **缓存结果**: 对于频繁调用的工具，考虑添加缓存
3. **限制输出**: 限制搜索和扫描的结果数量
4. **流式处理**: 对于大文件，使用流式读取而非一次性加载

### 安全考虑

1. **路径验证**: 防止路径遍历攻击
2. **命令过滤**: 限制危险 Shell 命令
3. **权限检查**: 验证文件系统权限
4. **输入清理**: 验证和清理所有用户输入

### 用户体验

1. **清晰反馈**: 提供彩色输出和进度指示
2. **错误信息**: 提供详细但易懂的错误信息
3. **结果格式**: 结构化输出便于 LLM 理解
4. **一致性**: 保持所有工具的接口一致性

## 相关文档

- [Agent 系统](./agent-system.md) - Agent 与工具的集成
- [配置管理](./config-management.md) - 配置工具权限
- [整体架构](./architecture.md) - 项目架构总览
