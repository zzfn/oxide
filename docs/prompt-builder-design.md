# PromptBuilder 设计文档

## 概述

PromptBuilder 是 Oxide 的提示词管理模块，负责：
1. 加载和组织系统提示词
2. 动态注入环境信息
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
│              │ SystemPrompt + Tools  │                      │
│              └───────────────────────┘                      │
└─────────────────────────────────────────────────────────────┘
```

### 目录结构

```
crates/oxide-core/src/
├── prompt/
│   ├── mod.rs              # 模块入口
│   ├── builder.rs          # PromptBuilder 实现
│   ├── parts.rs            # 提示词片段定义
│   ├── context.rs          # 运行时上下文
│   └── templates.rs        # 模板渲染
└── ...

prompts/                    # 提示词资源文件
├── system/
│   ├── core.md             # 核心行为指令
│   ├── tone.md             # 语气风格
│   ├── task-management.md  # 任务管理
│   ├── tool-policy.md      # 工具使用策略
│   ├── git-operations.md   # Git 操作指南
│   └── security.md         # 安全指令
├── tools/
│   ├── bash.md             # Bash 工具描述
│   ├── read.md             # Read 工具描述
│   ├── edit.md             # Edit 工具描述
│   └── ...
└── templates/
    └── env.md              # 环境信息模板
```

## 核心类型定义

### PromptBuilder

```rust
use std::path::PathBuf;

/// 提示词构建器
pub struct PromptBuilder {
    /// 系统提示词片段（按顺序拼接）
    system_parts: Vec<PromptPart>,
    /// 工具定义列表
    tools: Vec<ToolDefinition>,
    /// 运行时上下文
    context: RuntimeContext,
    /// 用户指令（OXIDE.md）
    user_instructions: Option<String>,
}

impl PromptBuilder {
    /// 创建新的构建器
    pub fn new() -> Self {
        Self {
            system_parts: Vec::new(),
            tools: Vec::new(),
            context: RuntimeContext::default(),
            user_instructions: None,
        }
    }

    /// 添加核心系统指令
    pub fn with_core_instructions(mut self) -> Self {
        self.system_parts.push(PromptPart::Core);
        self.system_parts.push(PromptPart::Tone);
        self.system_parts.push(PromptPart::Objectivity);
        self
    }

    /// 添加任务管理指令
    pub fn with_task_management(mut self) -> Self {
        self.system_parts.push(PromptPart::TaskManagement);
        self
    }

    /// 添加工具使用策略
    pub fn with_tool_policy(mut self) -> Self {
        self.system_parts.push(PromptPart::ToolPolicy);
        self
    }

    /// 添加 Git 操作指南
    pub fn with_git_operations(mut self) -> Self {
        self.system_parts.push(PromptPart::GitOperations);
        self
    }

    /// 添加安全指令
    pub fn with_security(mut self) -> Self {
        self.system_parts.push(PromptPart::Security);
        self
    }

    /// 添加工具定义
    pub fn with_tool(mut self, tool: ToolDefinition) -> Self {
        self.tools.push(tool);
        self
    }

    /// 批量添加工具
    pub fn with_tools(mut self, tools: Vec<ToolDefinition>) -> Self {
        self.tools.extend(tools);
        self
    }

    /// 设置运行时上下文
    pub fn with_context(mut self, context: RuntimeContext) -> Self {
        self.context = context;
        self
    }

    /// 加载用户指令（全局 + 项目 OXIDE.md）
    pub fn with_user_instructions(mut self, project_dir: &PathBuf) -> Self {
        if let Ok(instructions) = crate::config::load_instructions(project_dir) {
            if !instructions.is_empty() {
                self.user_instructions = Some(instructions);
            }
        }
        self
    }

    /// 使用默认配置（包含所有标准指令）
    pub fn default_agent() -> Self {
        Self::new()
            .with_core_instructions()
            .with_task_management()
            .with_tool_policy()
            .with_git_operations()
            .with_security()
    }

    /// 构建最终的系统提示词
    pub fn build(self) -> BuiltPrompt {
        let mut system_prompt = String::new();

        // 1. 拼接系统指令片段
        for part in &self.system_parts {
            system_prompt.push_str(&part.content());
            system_prompt.push_str("\n\n");
        }

        // 2. 添加工具定义
        if !self.tools.is_empty() {
            system_prompt.push_str("# Tools\n\n");
            for tool in &self.tools {
                system_prompt.push_str(&tool.to_prompt_section());
                system_prompt.push_str("\n\n---\n\n");
            }
        }

        // 3. 注入环境信息
        system_prompt.push_str(&self.context.to_env_section());

        // 4. 添加用户指令
        if let Some(ref instructions) = self.user_instructions {
            system_prompt.push_str("\n\n");
            system_prompt.push_str("<system-reminder>\n");
            system_prompt.push_str("As you answer the user's questions, you can use the following context:\n");
            system_prompt.push_str("# User Instructions\n\n");
            system_prompt.push_str(instructions);
            system_prompt.push_str("\n</system-reminder>");
        }

        BuiltPrompt {
            system: system_prompt,
            tools: self.tools,
        }
    }
}
```

### PromptPart（提示词片段）

```rust
/// 提示词片段类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptPart {
    /// 核心行为指令
    Core,
    /// 语气风格
    Tone,
    /// 专业客观性
    Objectivity,
    /// 任务管理
    TaskManagement,
    /// 工具使用策略
    ToolPolicy,
    /// Git 操作指南
    GitOperations,
    /// 安全指令
    Security,
    /// 自定义片段
    Custom(&'static str),
}

impl PromptPart {
    /// 获取片段内容
    pub fn content(&self) -> &'static str {
        match self {
            Self::Core => include_str!("../../../prompts/system/core.md"),
            Self::Tone => include_str!("../../../prompts/system/tone.md"),
            Self::Objectivity => include_str!("../../../prompts/system/objectivity.md"),
            Self::TaskManagement => include_str!("../../../prompts/system/task-management.md"),
            Self::ToolPolicy => include_str!("../../../prompts/system/tool-policy.md"),
            Self::GitOperations => include_str!("../../../prompts/system/git-operations.md"),
            Self::Security => include_str!("../../../prompts/system/security.md"),
            Self::Custom(content) => content,
        }
    }
}
```

### RuntimeContext（运行时上下文）

```rust
use std::path::PathBuf;

/// 运行时上下文
#[derive(Debug, Clone)]
pub struct RuntimeContext {
    /// 工作目录
    pub working_dir: PathBuf,
    /// 是否是 Git 仓库
    pub is_git_repo: bool,
    /// 操作系统平台
    pub platform: String,
    /// 操作系统版本
    pub os_version: String,
    /// 当前日期
    pub today: String,
    /// 模型名称
    pub model_name: String,
    /// 模型 ID
    pub model_id: String,
}

impl Default for RuntimeContext {
    fn default() -> Self {
        Self {
            working_dir: std::env::current_dir().unwrap_or_default(),
            is_git_repo: false,
            platform: std::env::consts::OS.to_string(),
            os_version: String::new(),
            today: chrono::Local::now().format("%Y-%m-%d").to_string(),
            model_name: "Claude".to_string(),
            model_id: "claude-sonnet-4-20250514".to_string(),
        }
    }
}

impl RuntimeContext {
    /// 从当前环境创建上下文
    pub fn from_env(working_dir: PathBuf) -> Self {
        let is_git_repo = working_dir.join(".git").exists();
        let os_version = Self::get_os_version();

        Self {
            working_dir,
            is_git_repo,
            platform: std::env::consts::OS.to_string(),
            os_version,
            today: chrono::Local::now().format("%Y-%m-%d").to_string(),
            ..Default::default()
        }
    }

    /// 设置模型信息
    pub fn with_model(mut self, name: &str, id: &str) -> Self {
        self.model_name = name.to_string();
        self.model_id = id.to_string();
        self
    }

    /// 生成环境信息段落
    pub fn to_env_section(&self) -> String {
        format!(
            r#"Here is useful information about the environment you are running in:
<env>
Working directory: {}
Is directory a git repo: {}
Platform: {}
OS Version: {}
Today's date: {}
</env>
You are powered by the model named {}. The exact model ID is {}."#,
            self.working_dir.display(),
            if self.is_git_repo { "Yes" } else { "No" },
            self.platform,
            self.os_version,
            self.today,
            self.model_name,
            self.model_id
        )
    }

    fn get_os_version() -> String {
        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("sw_vers")
                .arg("-productVersion")
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .map(|s| format!("macOS {}", s.trim()))
                .unwrap_or_else(|| "macOS".to_string())
        }
        #[cfg(target_os = "linux")]
        {
            std::process::Command::new("uname")
                .arg("-r")
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .map(|s| format!("Linux {}", s.trim()))
                .unwrap_or_else(|| "Linux".to_string())
        }
        #[cfg(target_os = "windows")]
        {
            "Windows".to_string()
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            "Unknown".to_string()
        }
    }
}
```

### ToolDefinition（工具定义）

```rust
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// 工具定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// 工具名称
    pub name: String,
    /// 工具描述
    pub description: String,
    /// 参数 JSON Schema
    pub input_schema: JsonValue,
}

impl ToolDefinition {
    /// 创建新的工具定义
    pub fn new(name: &str, description: &str, schema: JsonValue) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            input_schema: schema,
        }
    }

    /// 生成提示词中的工具段落
    pub fn to_prompt_section(&self) -> String {
        format!(
            "## {}\n\n{}\n\n```json\n{}\n```",
            self.name,
            self.description,
            serde_json::to_string_pretty(&self.input_schema).unwrap_or_default()
        )
    }

    /// 转换为 API 格式
    pub fn to_api_format(&self) -> JsonValue {
        serde_json::json!({
            "name": self.name,
            "description": self.description,
            "input_schema": self.input_schema
        })
    }
}
```

### BuiltPrompt（构建结果）

```rust
/// 构建完成的提示词
#[derive(Debug, Clone)]
pub struct BuiltPrompt {
    /// 系统提示词
    pub system: String,
    /// 工具定义列表
    pub tools: Vec<ToolDefinition>,
}

impl BuiltPrompt {
    /// 获取系统提示词
    pub fn system_prompt(&self) -> &str {
        &self.system
    }

    /// 获取工具定义（API 格式）
    pub fn tools_for_api(&self) -> Vec<serde_json::Value> {
        self.tools.iter().map(|t| t.to_api_format()).collect()
    }

    /// 估算 token 数量（粗略估计）
    pub fn estimated_tokens(&self) -> usize {
        // 粗略估计：每 4 个字符约 1 个 token
        self.system.len() / 4
    }
}
```

## 使用示例

### 基本用法

```rust
use oxide_core::prompt::{PromptBuilder, RuntimeContext};
use std::path::PathBuf;

fn main() {
    let project_dir = PathBuf::from("/path/to/project");

    // 创建运行时上下文
    let context = RuntimeContext::from_env(project_dir.clone())
        .with_model("Claude Sonnet 4", "claude-sonnet-4-20250514");

    // 构建提示词
    let prompt = PromptBuilder::default_agent()
        .with_context(context)
        .with_user_instructions(&project_dir)
        .with_tools(get_default_tools())
        .build();

    println!("System prompt length: {} chars", prompt.system.len());
    println!("Estimated tokens: {}", prompt.estimated_tokens());
}
```

### 自定义构建

```rust
// 只包含部分指令（例如轻量级模式）
let prompt = PromptBuilder::new()
    .with_core_instructions()
    .with_tool_policy()
    .with_context(context)
    .build();

// 添加自定义片段
let prompt = PromptBuilder::default_agent()
    .with_part(PromptPart::Custom("# Custom Instructions\n\nDo something special."))
    .build();
```

### 与现有代码集成

```rust
// 在 oxide-cli/src/agent.rs 中使用
impl Agent {
    pub fn new(project_dir: PathBuf, config: Config) -> Self {
        let context = RuntimeContext::from_env(project_dir.clone())
            .with_model(&config.model.default_model, &config.model.default_model);

        let prompt = PromptBuilder::default_agent()
            .with_context(context)
            .with_user_instructions(&project_dir)
            .with_tools(Self::get_enabled_tools(&config))
            .build();

        Self {
            system_prompt: prompt.system,
            tools: prompt.tools,
            // ...
        }
    }
}
```

## 提示词文件示例

### prompts/system/core.md

```markdown
You are an interactive CLI tool that helps users with software engineering tasks.
Use the instructions below and the tools available to you to assist the user.

IMPORTANT: You must NEVER generate or guess URLs for the user unless you are
confident that the URLs are for helping the user with programming.
```

### prompts/system/tone.md

```markdown
## Tone and style

- Only use emojis if the user explicitly requests it.
- Your output will be displayed on a command line interface.
- Your responses should be short and concise.
- You can use Github-flavored markdown for formatting.
- Output text to communicate with the user; all text you output outside of
  tool use is displayed to the user.
- NEVER create files unless they're absolutely necessary for achieving your goal.
```

### prompts/system/task-management.md

```markdown
## Task Management

You have access to the TodoWrite tools to help you manage and plan tasks.
Use these tools VERY frequently to ensure that you are tracking your tasks
and giving the user visibility into your progress.

It is critical that you mark todos as completed as soon as you are done with a task.
Do not batch up multiple tasks before marking them as completed.
```

## 配置层级合并

```
优先级（从低到高）：
1. 内置系统提示词 (prompts/system/*.md)
2. 全局用户配置 (~/.oxide/OXIDE.md)
3. 项目配置 (./OXIDE.md)
4. 运行时参数

合并策略：
- 系统提示词：按顺序拼接
- 用户指令：追加到 <system-reminder> 中
- 工具定义：根据权限配置过滤
- 环境信息：运行时动态生成
```

## 扩展点

### 1. 技能系统（Skills）

```rust
/// 技能定义
pub struct Skill {
    pub name: String,
    pub description: String,
    pub prompt: String,
}

impl PromptBuilder {
    /// 添加可用技能列表
    pub fn with_skills(mut self, skills: Vec<Skill>) -> Self {
        // 生成技能列表提示
        let skills_prompt = skills.iter()
            .map(|s| format!("- {}: {}", s.name, s.description))
            .collect::<Vec<_>>()
            .join("\n");

        self.system_parts.push(PromptPart::Custom(
            Box::leak(format!(
                "<system-reminder>\nThe following skills are available:\n{}\n</system-reminder>",
                skills_prompt
            ).into_boxed_str())
        ));
        self
    }
}
```

### 2. 动态工具注册

```rust
/// 工具注册表
pub struct ToolRegistry {
    tools: HashMap<String, ToolDefinition>,
}

impl ToolRegistry {
    pub fn register(&mut self, tool: ToolDefinition) {
        self.tools.insert(tool.name.clone(), tool);
    }

    pub fn get_enabled(&self, config: &PermissionsConfig) -> Vec<ToolDefinition> {
        self.tools.values()
            .filter(|t| config.is_allowed(&t.name))
            .cloned()
            .collect()
    }
}
```

### 3. 提示词缓存

```rust
use std::sync::OnceLock;

static CACHED_SYSTEM_PROMPT: OnceLock<String> = OnceLock::new();

impl PromptBuilder {
    /// 获取缓存的系统提示词（静态部分）
    pub fn cached_system_prompt() -> &'static str {
        CACHED_SYSTEM_PROMPT.get_or_init(|| {
            Self::default_agent().build().system
        })
    }
}
```

## 测试策略

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_default() {
        let prompt = PromptBuilder::default_agent().build();
        assert!(!prompt.system.is_empty());
        assert!(prompt.system.contains("software engineering"));
    }

    #[test]
    fn test_builder_with_context() {
        let context = RuntimeContext {
            working_dir: PathBuf::from("/test/project"),
            is_git_repo: true,
            platform: "darwin".to_string(),
            os_version: "macOS 14.0".to_string(),
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

        let prompt = PromptBuilder::new()
            .with_tool(tool)
            .build();

        assert!(prompt.system.contains("## TestTool"));
        assert_eq!(prompt.tools.len(), 1);
    }

    #[test]
    fn test_user_instructions_merge() {
        // 测试用户指令合并
        let temp_dir = std::env::temp_dir().join("oxide_test");
        std::fs::create_dir_all(&temp_dir).unwrap();
        std::fs::write(temp_dir.join("OXIDE.md"), "# Test Instructions").unwrap();

        let prompt = PromptBuilder::new()
            .with_user_instructions(&temp_dir)
            .build();

        assert!(prompt.system.contains("Test Instructions"));
        assert!(prompt.system.contains("<system-reminder>"));

        std::fs::remove_dir_all(&temp_dir).ok();
    }
}
```

## 迁移计划

1. **Phase 1**: 创建 `prompts/` 目录，拆分现有 `prompt.md` 内容
2. **Phase 2**: 实现 `PromptBuilder` 核心类型
3. **Phase 3**: 集成到 `oxide-cli` 的 Agent 初始化流程
4. **Phase 4**: 添加技能系统支持
5. **Phase 5**: 性能优化（缓存、懒加载）

## 参考

- Claude Code prompt.md 结构分析
- Anthropic API Tool Use 文档
- 现有 `oxide-core/src/config.rs` 实现
