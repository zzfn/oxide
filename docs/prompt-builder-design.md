# PromptBuilder 设计文档

## 概述

PromptBuilder 是 Oxide 的提示词管理模块，负责：
1. 加载和组织系统提示词（使用 rust-embed 嵌入）
2. 动态注入运行时环境信息
3. 合并用户配置（全局 + 项目 OXIDE.md）
4. 生成最终发送给 API 的完整 prompt

## 架构设计

### 整体结构

```
┌─────────────────────────────────────────────────────────────┐
│                      PromptBuilder                          │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │ SystemParts │  │ ToolDefs    │  │ RuntimeContext      │  │
│  │ (静态指令)   │  │ (工具定义)   │  │ (运行时信息)         │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
│         │                │                   │              │
│         └────────────────┼───────────────────┘              │
│                          ▼                                  │
│                   ┌─────────────┐                           │
│                   │ build()     │                           │
│                   └─────────────┘                           │
│                          │                                  │
│                          ▼                                  │
│              ┌───────────────────────┐                      │
│              │ BuiltPrompt           │                      │
│              │ (system + tools)      │                      │
│              └───────────────────────┘                      │
└─────────────────────────────────────────────────────────────┘
```

### 目录结构

```
crates/oxide-core/
├── src/prompt/
│   ├── mod.rs              # 模块入口，导出公共类型
│   ├── builder.rs          # PromptBuilder 实现
│   ├── parts.rs            # 提示词片段定义 (rust-embed)
│   ├── context.rs          # 运行时上下文
│   └── tool.rs             # 工具定义
└── prompts/                # 提示词资源文件 (编译时嵌入)
    └── system/
        ├── core.md             # 核心行为指令
        ├── tone.md             # 语气风格
        ├── objectivity.md      # 专业客观性
        ├── security.md         # 安全指令
        ├── task-management.md  # 任务管理
        ├── doing-tasks.md      # 执行任务指南
        ├── tool-policy.md      # 工具使用策略
        └── git-operations.md   # Git 操作指南
```

## 核心类型

### PromptBuilder

提示词构建器，使用 Builder 模式组装最终的系统提示词。

```rust
pub struct PromptBuilder {
    system_parts: Vec<PromptPart>,      // 系统提示词片段
    tools: Vec<ToolDefinition>,          // 工具定义列表
    context: Option<RuntimeContext>,     // 运行时上下文
    user_instructions: Option<String>,   // 用户指令
}
```

**主要方法：**

| 方法 | 说明 |
|------|------|
| `new()` | 创建空构建器 |
| `default_agent()` | 预设：包含所有标准指令 |
| `lightweight()` | 预设：仅核心 + 安全指令 |
| `with_core_instructions()` | 添加 Core + Tone + Objectivity |
| `with_security()` | 添加安全指令 |
| `with_task_management()` | 添加任务管理指令 |
| `with_doing_tasks()` | 添加执行任务指南 |
| `with_tool_policy()` | 添加工具使用策略 |
| `with_git_operations()` | 添加 Git 操作指南 |
| `with_part(PromptPart)` | 添加指定片段 |
| `with_custom(String)` | 添加自定义内容 |
| `with_tool(ToolDefinition)` | 添加单个工具 |
| `with_tools(Vec<ToolDefinition>)` | 批量添加工具 |
| `with_context(RuntimeContext)` | 设置运行时上下文 |
| `with_user_instructions(&PathBuf)` | 从目录加载用户指令 |
| `with_user_instructions_content(String)` | 直接设置用户指令 |
| `build()` | 构建最终提示词 |

### PromptPart

提示词片段枚举，每个变体对应一个嵌入的 markdown 文件。

```rust
pub enum PromptPart {
    Core,           // system/core.md
    Tone,           // system/tone.md
    Objectivity,    // system/objectivity.md
    Security,       // system/security.md
    TaskManagement, // system/task-management.md
    DoingTasks,     // system/doing-tasks.md
    ToolPolicy,     // system/tool-policy.md
    GitOperations,  // system/git-operations.md
    Custom(String), // 自定义内容
}
```

**方法：**
- `file_path()` - 获取对应的文件路径
- `content()` - 获取片段内容
- `standard_parts()` - 获取所有标准片段（按推荐顺序）

### RuntimeContext

运行时上下文，包含环境信息。

```rust
pub struct RuntimeContext {
    pub working_dir: PathBuf,   // 工作目录
    pub is_git_repo: bool,      // 是否是 Git 仓库
    pub platform: String,       // 操作系统平台
    pub os_version: String,     // 操作系统版本
    pub today: String,          // 当前日期
    pub model_name: String,     // 模型名称
    pub model_id: String,       // 模型 ID
}
```

**方法：**
- `default()` - 使用默认值创建
- `from_env(PathBuf)` - 从当前环境创建
- `with_model(name, id)` - 设置模型信息
- `to_env_section()` - 生成 `<env>` 格式的环境信息

### ToolDefinition

工具定义，用于描述 Agent 可用的工具。

```rust
pub struct ToolDefinition {
    pub name: String,           // 工具名称
    pub description: String,    // 工具描述
    pub input_schema: JsonValue, // 参数 JSON Schema
}
```

**方法：**
- `new(name, description, schema)` - 创建新定义
- `to_prompt_section()` - 生成提示词中的工具段落
- `to_api_format()` - 转换为 API 格式

### BuiltPrompt

构建完成的提示词结果。

```rust
pub struct BuiltPrompt {
    pub system: String,              // 完整系统提示词
    pub tools: Vec<ToolDefinition>,  // 工具定义列表
}
```

**方法：**
- `system_prompt()` - 获取系统提示词
- `tools_for_api()` - 获取 API 格式的工具列表
- `estimated_tokens()` - 估算 token 数量
- `system_len()` - 获取字符数
- `has_tools()` / `tool_count()` - 工具相关查询

## 构建流程

`build()` 方法按以下顺序组装最终提示词：

```
1. 系统指令片段 (PromptPart)
   ├── Core
   ├── Tone
   ├── Objectivity
   ├── Security
   ├── TaskManagement
   ├── DoingTasks
   ├── ToolPolicy
   ├── GitOperations
   └── Custom(...)

2. 工具定义 (# Tools)
   ├── ## ToolName1
   ├── ## ToolName2
   └── ...

3. 运行时上下文 (<env>...</env>)

4. 用户指令 (<system-reminder>...</system-reminder>)
```

## 使用示例

### 基本用法

```rust
use oxide_core::prompt::{PromptBuilder, RuntimeContext};
use std::path::PathBuf;

let project_dir = PathBuf::from("/path/to/project");

// 创建运行时上下文
let context = RuntimeContext::from_env(project_dir.clone())
    .with_model("Claude Sonnet 4", "claude-sonnet-4-20250514");

// 使用默认配置构建
let prompt = PromptBuilder::default_agent()
    .with_context(context)
    .with_user_instructions(&project_dir)
    .build();

println!("System prompt: {} chars", prompt.system_len());
println!("Estimated tokens: {}", prompt.estimated_tokens());
```

### 轻量模式

```rust
// 仅包含核心指令和安全指令
let prompt = PromptBuilder::lightweight()
    .with_context(context)
    .build();
```

### 自定义构建

```rust
// 选择性添加片段
let prompt = PromptBuilder::new()
    .with_core_instructions()
    .with_security()
    .with_tool_policy()
    .with_custom("# 项目特定指令\n\n请使用中文回复。")
    .with_context(context)
    .build();
```

### 添加工具

```rust
let read_tool = ToolDefinition::new(
    "Read",
    "读取文件内容",
    serde_json::json!({
        "type": "object",
        "properties": {
            "file_path": { "type": "string", "description": "文件路径" }
        },
        "required": ["file_path"]
    }),
);

let prompt = PromptBuilder::default_agent()
    .with_tool(read_tool)
    .build();

// 获取 API 格式的工具列表
let api_tools = prompt.tools_for_api();
```

## 配置层级

```
优先级（从低到高）：
1. 内置系统提示词 (prompts/system/*.md)
2. 全局用户配置 (~/.oxide/OXIDE.md)
3. 项目配置 (./OXIDE.md)
4. 运行时参数

合并策略：
- 系统提示词：按顺序拼接
- 用户指令：追加到 <system-reminder> 中
- 工具定义：根据配置过滤
- 环境信息：运行时动态生成
```

## 预设配置对比

| 配置 | 包含的片段 |
|------|-----------|
| `default_agent()` | Core, Tone, Objectivity, Security, TaskManagement, DoingTasks, ToolPolicy, GitOperations |
| `lightweight()` | Core, Tone, Objectivity, Security |

## 测试

```rust
#[test]
fn test_builder_default_agent() {
    let prompt = PromptBuilder::default_agent().build();
    assert!(!prompt.system.is_empty());
    assert!(prompt.system.contains("interactive CLI tool"));
}

#[test]
fn test_builder_with_context() {
    let context = RuntimeContext {
        working_dir: PathBuf::from("/test/project"),
        is_git_repo: true,
        platform: "darwin".to_string(),
        os_version: "Darwin 24.0".to_string(),
        today: "2025-01-31".to_string(),
        model_name: "Claude".to_string(),
        model_id: "claude-sonnet-4".to_string(),
    };

    let prompt = PromptBuilder::new()
        .with_core_instructions()
        .with_context(context)
        .build();

    assert!(prompt.system.contains("/test/project"));
    assert!(prompt.system.contains("git repo: Yes"));
}

#[test]
fn test_builder_with_tools() {
    let tool = ToolDefinition::new(
        "TestTool",
        "A test tool",
        serde_json::json!({"type": "object"}),
    );

    let prompt = PromptBuilder::new().with_tool(tool).build();

    assert!(prompt.system.contains("# Tools"));
    assert!(prompt.system.contains("## TestTool"));
    assert_eq!(prompt.tool_count(), 1);
}
```

## 扩展点

### 1. 添加新的提示词片段

1. 在 `prompts/system/` 下创建新的 `.md` 文件
2. 在 `PromptPart` 枚举中添加新变体
3. 在 `file_path()` 方法中添加映射
4. 可选：在 `PromptBuilder` 中添加便捷方法

### 2. 动态工具注册

```rust
pub struct ToolRegistry {
    tools: HashMap<String, ToolDefinition>,
}

impl ToolRegistry {
    pub fn register(&mut self, tool: ToolDefinition) {
        self.tools.insert(tool.name.clone(), tool);
    }

    pub fn get_enabled(&self, config: &Config) -> Vec<ToolDefinition> {
        self.tools.values()
            .filter(|t| config.is_tool_allowed(&t.name))
            .cloned()
            .collect()
    }
}
```

### 3. 提示词缓存

```rust
use std::sync::OnceLock;

static CACHED_BASE_PROMPT: OnceLock<String> = OnceLock::new();

impl PromptBuilder {
    pub fn cached_base() -> &'static str {
        CACHED_BASE_PROMPT.get_or_init(|| {
            Self::default_agent().build().system
        })
    }
}
```

## 参考

- [Anthropic API Tool Use 文档](https://docs.anthropic.com/en/docs/build-with-claude/tool-use)
- [rust-embed 文档](https://docs.rs/rust-embed)
