//! 计划模式工具
//!
//! 实现 EnterPlanMode 和 ExitPlanMode 工具，让 Agent 可以自主进入和退出计划模式。

use super::FileToolError;
use colored::*;
use inquire::{Confirm, Select};
use once_cell::sync::Lazy;
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use chrono::{Local, Utc};

// ============================================================================
// 权限管理系统 (AllowedPrompt)
// ============================================================================

/// 允许的权限提示
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllowedPrompt {
    /// 工具名称
    pub tool: String,

    /// 权限描述（语义化描述，如 "run tests", "install dependencies"）
    pub prompt: String,
}

impl AllowedPrompt {
    pub fn new(tool: &str, prompt: &str) -> Self {
        Self {
            tool: tool.to_string(),
            prompt: prompt.to_string(),
        }
    }

    /// 检查是否匹配给定的工具和操作
    pub fn matches(&self, tool: &str, operation: &str) -> bool {
        self.tool == tool && self.prompt.to_lowercase().contains(&operation.to_lowercase())
    }
}

// ============================================================================
// 计划模式状态管理
// ============================================================================

/// 计划模式状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanModeState {
    /// 是否处于计划模式
    pub active: bool,

    /// 当前计划 ID
    pub plan_id: Option<String>,

    /// 计划文件路径
    pub plan_file: Option<PathBuf>,

    /// 计划内容
    pub plan_content: Option<String>,

    /// 允许的权限列表
    pub allowed_prompts: Vec<AllowedPrompt>,

    /// 进入计划模式的时间
    pub entered_at: Option<chrono::DateTime<Utc>>,

    /// 用户是否已批准
    pub approved: bool,
}

impl Default for PlanModeState {
    fn default() -> Self {
        Self {
            active: false,
            plan_id: None,
            plan_file: None,
            plan_content: None,
            allowed_prompts: Vec::new(),
            entered_at: None,
            approved: false,
        }
    }
}

impl PlanModeState {
    /// 进入计划模式
    pub fn enter(&mut self) -> String {
        let plan_id = format!("plan_{}", Local::now().format("%Y%m%d_%H%M%S"));
        let plan_file = PathBuf::from(".oxide/plans").join(format!("{}.md", plan_id));

        self.active = true;
        self.plan_id = Some(plan_id.clone());
        self.plan_file = Some(plan_file);
        self.plan_content = None;
        self.allowed_prompts = Vec::new();
        self.entered_at = Some(Utc::now());
        self.approved = false;

        plan_id
    }

    /// 退出计划模式
    pub fn exit(&mut self) {
        self.active = false;
        self.approved = false;
    }

    /// 设置计划内容
    pub fn set_plan_content(&mut self, content: String) {
        self.plan_content = Some(content);
    }

    /// 添加允许的权限
    pub fn add_allowed_prompt(&mut self, prompt: AllowedPrompt) {
        self.allowed_prompts.push(prompt);
    }

    /// 检查权限是否被允许
    pub fn is_allowed(&self, tool: &str, operation: &str) -> bool {
        if !self.active || !self.approved {
            return false;
        }
        self.allowed_prompts.iter().any(|p| p.matches(tool, operation))
    }

    /// 批准计划
    pub fn approve(&mut self) {
        self.approved = true;
    }
}

/// 全局计划模式状态管理器
pub struct PlanModeManager {
    state: Arc<RwLock<PlanModeState>>,
}

impl PlanModeManager {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(PlanModeState::default())),
        }
    }

    pub fn get_state(&self) -> PlanModeState {
        self.state.read().unwrap().clone()
    }

    pub fn is_active(&self) -> bool {
        self.state.read().unwrap().active
    }

    pub fn is_approved(&self) -> bool {
        self.state.read().unwrap().approved
    }

    pub fn enter(&self) -> String {
        self.state.write().unwrap().enter()
    }

    pub fn exit(&self) {
        self.state.write().unwrap().exit()
    }

    pub fn set_plan_content(&self, content: String) {
        self.state.write().unwrap().set_plan_content(content);
    }

    pub fn add_allowed_prompt(&self, prompt: AllowedPrompt) {
        self.state.write().unwrap().add_allowed_prompt(prompt);
    }

    pub fn approve(&self) {
        self.state.write().unwrap().approve();
    }

    pub fn get_allowed_prompts(&self) -> Vec<AllowedPrompt> {
        self.state.read().unwrap().allowed_prompts.clone()
    }

    pub fn is_allowed(&self, tool: &str, operation: &str) -> bool {
        self.state.read().unwrap().is_allowed(tool, operation)
    }
}

impl Default for PlanModeManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for PlanModeManager {
    fn clone(&self) -> Self {
        Self {
            state: Arc::clone(&self.state),
        }
    }
}

// 全局单例
static PLAN_MODE_MANAGER: Lazy<PlanModeManager> = Lazy::new(|| PlanModeManager::new());

// ============================================================================
// EnterPlanMode 工具
// ============================================================================

/// EnterPlanMode 工具输入参数
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EnterPlanModeArgs {
    // 无参数，进入计划模式不需要额外参数
}

/// EnterPlanMode 工具输出
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterPlanModeOutput {
    /// 是否成功
    pub success: bool,

    /// 计划 ID
    pub plan_id: Option<String>,

    /// 计划文件路径
    pub plan_file: Option<String>,

    /// 消息
    pub message: String,
}

/// EnterPlanMode 工具
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EnterPlanModeTool;

impl Tool for EnterPlanModeTool {
    const NAME: &'static str = "enter_plan_mode";

    type Error = FileToolError;
    type Args = EnterPlanModeArgs;
    type Output = EnterPlanModeOutput;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "enter_plan_mode".to_string(),
            description: r#"Enter plan mode to design an implementation approach for user approval.

Use this tool proactively when you're about to start a non-trivial implementation task. Getting user sign-off on your approach before writing code prevents wasted effort and ensures alignment.

## When to Use This Tool

Use it when ANY of these conditions apply:
1. **New Feature Implementation**: Adding meaningful new functionality
2. **Multiple Valid Approaches**: The task can be solved in several different ways
3. **Code Modifications**: Changes that affect existing behavior or structure
4. **Architectural Decisions**: The task requires choosing between patterns or technologies
5. **Multi-File Changes**: The task will likely touch more than 2-3 files
6. **Unclear Requirements**: You need to explore before understanding the full scope

## When NOT to Use This Tool

Only skip for simple tasks:
- Single-line or few-line fixes (typos, obvious bugs, small tweaks)
- Adding a single function with clear requirements
- Tasks where the user has given very specific, detailed instructions
- Pure research/exploration tasks

## What Happens in Plan Mode

In plan mode, you'll:
1. Thoroughly explore the codebase using Glob, Grep, and Read tools
2. Understand existing patterns and architecture
3. Design an implementation approach
4. Present your plan to the user for approval using exit_plan_mode
5. Exit plan mode when ready to implement"#.to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        }
    }

    async fn call(&self, _args: Self::Args) -> Result<Self::Output, Self::Error> {
        // 检查是否已经在计划模式中
        if PLAN_MODE_MANAGER.is_active() {
            return Ok(EnterPlanModeOutput {
                success: false,
                plan_id: None,
                plan_file: None,
                message: "Already in plan mode. Use exit_plan_mode to exit first.".to_string(),
            });
        }

        // 创建计划目录
        let plans_dir = PathBuf::from(".oxide/plans");
        if let Err(e) = fs::create_dir_all(&plans_dir) {
            return Ok(EnterPlanModeOutput {
                success: false,
                plan_id: None,
                plan_file: None,
                message: format!("Failed to create plans directory: {}", e),
            });
        }

        // 进入计划模式
        let plan_id = PLAN_MODE_MANAGER.enter();
        let state = PLAN_MODE_MANAGER.get_state();
        let plan_file = state.plan_file.map(|p| p.display().to_string());

        // 显示进入计划模式的提示
        println!();
        println!("{}", "╔══════════════════════════════════════════════════════════════╗".bright_cyan());
        println!("{}", "║                    📋 进入计划模式                            ║".bright_cyan());
        println!("{}", "╚══════════════════════════════════════════════════════════════╝".bright_cyan());
        println!();
        println!("{} {}", "计划 ID:".bright_white(), plan_id.bright_yellow());
        if let Some(ref file) = plan_file {
            println!("{} {}", "计划文件:".bright_white(), file.bright_cyan());
        }
        println!();
        println!("{}", "在计划模式下，你可以：".bright_white());
        println!("  {} 探索代码库，了解现有架构", "•".bright_green());
        println!("  {} 设计实现方案", "•".bright_green());
        println!("  {} 使用 exit_plan_mode 提交计划并请求用户批准", "•".bright_green());
        println!();

        Ok(EnterPlanModeOutput {
            success: true,
            plan_id: Some(plan_id),
            plan_file,
            message: "Successfully entered plan mode. Design your implementation approach and use exit_plan_mode when ready for user approval.".to_string(),
        })
    }
}

// ============================================================================
// ExitPlanMode 工具
// ============================================================================

/// ExitPlanMode 工具输入参数
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExitPlanModeArgs {
    /// 需要的权限列表
    #[serde(default)]
    pub allowed_prompts: Vec<AllowedPromptArg>,
}

/// 权限参数
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AllowedPromptArg {
    /// 工具名称
    pub tool: String,

    /// 权限描述
    pub prompt: String,
}

/// ExitPlanMode 工具输出
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExitPlanModeOutput {
    /// 是否成功
    pub success: bool,

    /// 用户是否批准
    pub approved: bool,

    /// 消息
    pub message: String,

    /// 批准的权限列表
    pub approved_prompts: Vec<AllowedPrompt>,
}

/// ExitPlanMode 工具
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExitPlanModeTool;

impl ExitPlanModeTool {
    /// 请求用户批准计划
    fn request_approval(allowed_prompts: &[AllowedPrompt]) -> Result<(bool, String), FileToolError> {
        println!();
        println!("{}", "╔══════════════════════════════════════════════════════════════╗".bright_yellow());
        println!("{}", "║                    📋 计划审批请求                            ║".bright_yellow());
        println!("{}", "╚══════════════════════════════════════════════════════════════╝".bright_yellow());
        println!();

        // 显示计划内容
        let state = PLAN_MODE_MANAGER.get_state();
        if let Some(ref content) = state.plan_content {
            println!("{}", "📝 计划内容:".bright_cyan());
            println!("{}", "─".repeat(60).dimmed());
            // 限制显示长度
            let display_content = if content.len() > 2000 {
                format!("{}...\n\n(内容已截断)", &content[..2000])
            } else {
                content.clone()
            };
            println!("{}", display_content);
            println!("{}", "─".repeat(60).dimmed());
            println!();
        }

        // 显示需要的权限
        if !allowed_prompts.is_empty() {
            println!("{}", "🔐 需要的权限:".bright_cyan());
            for (i, prompt) in allowed_prompts.iter().enumerate() {
                println!(
                    "  {}. {} - {}",
                    (i + 1).to_string().bright_white(),
                    prompt.tool.bright_yellow(),
                    prompt.prompt.bright_white()
                );
            }
            println!();
        }

        // 请求用户批准
        let options = vec![
            "批准并执行 - Approve and execute the plan",
            "修改计划 - Request modifications to the plan",
            "取消 - Cancel and discard the plan",
        ];

        let selection = Select::new("请选择操作:", options)
            .with_help_message("↑↓ 移动，Enter 确认")
            .prompt();

        match selection {
            Ok(choice) => {
                if choice.starts_with("批准") {
                    Ok((true, "Plan approved by user.".to_string()))
                } else if choice.starts_with("修改") {
                    // 请求用户输入修改意见
                    println!();
                    println!("{}", "请输入修改意见 (按 Enter 提交):".bright_yellow());
                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input).map_err(|e| FileToolError::Io(e))?;
                    let feedback = input.trim().to_string();
                    Ok((false, format!("User requested modifications: {}", feedback)))
                } else {
                    Ok((false, "Plan cancelled by user.".to_string()))
                }
            }
            Err(_) => {
                // 用户取消或出错，使用简单的确认
                println!();
                let confirm = Confirm::new("是否批准此计划?")
                    .with_default(false)
                    .prompt();

                match confirm {
                    Ok(true) => Ok((true, "Plan approved by user.".to_string())),
                    Ok(false) => Ok((false, "Plan rejected by user.".to_string())),
                    Err(_) => Ok((false, "Plan approval cancelled.".to_string())),
                }
            }
        }
    }

    /// 保存计划到文件
    fn save_plan(plan_id: &str, content: &str, allowed_prompts: &[AllowedPrompt], approved: bool) -> Result<PathBuf, FileToolError> {
        let plans_dir = PathBuf::from(".oxide/plans");
        fs::create_dir_all(&plans_dir).map_err(|e| FileToolError::Io(e))?;

        let plan_file = plans_dir.join(format!("{}.md", plan_id));

        let mut full_content = String::new();
        full_content.push_str(&format!("# 计划: {}\n\n", plan_id));
        full_content.push_str(&format!("> 生成时间: {}\n", Local::now().format("%Y-%m-%d %H:%M:%S")));
        full_content.push_str(&format!("> 状态: {}\n\n", if approved { "✅ 已批准" } else { "❌ 未批准" }));

        if !allowed_prompts.is_empty() {
            full_content.push_str("## 🔐 权限列表\n\n");
            for prompt in allowed_prompts {
                full_content.push_str(&format!("- **{}**: {}\n", prompt.tool, prompt.prompt));
            }
            full_content.push_str("\n");
        }

        full_content.push_str("## 📋 计划内容\n\n");
        full_content.push_str(content);

        fs::write(&plan_file, full_content).map_err(|e| FileToolError::Io(e))?;

        Ok(plan_file)
    }
}

impl Tool for ExitPlanModeTool {
    const NAME: &'static str = "exit_plan_mode";

    type Error = FileToolError;
    type Args = ExitPlanModeArgs;
    type Output = ExitPlanModeOutput;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "exit_plan_mode".to_string(),
            description: r#"Exit plan mode and request user approval for the implementation plan.

Use this tool when you have finished writing your plan and are ready for user approval.

## How This Tool Works
- You should have already designed your implementation approach
- This tool will display the plan to the user and request approval
- The user can: approve, request modifications, or cancel
- If approved, you can proceed with implementation

## Before Using This Tool
Ensure your plan is complete and unambiguous:
- If you have unresolved questions about requirements, clarify first
- Once your plan is finalized, use THIS tool to request approval

## Parameters
- allowed_prompts: List of permissions needed to implement the plan
  - tool: The tool name (e.g., "Bash", "Write")
  - prompt: Description of the action (e.g., "run tests", "install dependencies")"#.to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "allowed_prompts": {
                        "type": "array",
                        "description": "Permissions needed to implement the plan",
                        "items": {
                            "type": "object",
                            "properties": {
                                "tool": {
                                    "type": "string",
                                    "description": "The tool this permission applies to (e.g., 'Bash', 'Write')"
                                },
                                "prompt": {
                                    "type": "string",
                                    "description": "Semantic description of the action (e.g., 'run tests', 'install dependencies')"
                                }
                            },
                            "required": ["tool", "prompt"]
                        }
                    }
                },
                "required": []
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // 检查是否在计划模式中
        if !PLAN_MODE_MANAGER.is_active() {
            return Ok(ExitPlanModeOutput {
                success: false,
                approved: false,
                message: "Not in plan mode. Use enter_plan_mode first.".to_string(),
                approved_prompts: Vec::new(),
            });
        }

        // 转换权限参数
        let allowed_prompts: Vec<AllowedPrompt> = args
            .allowed_prompts
            .iter()
            .map(|p| AllowedPrompt::new(&p.tool, &p.prompt))
            .collect();

        // 添加权限到状态
        for prompt in &allowed_prompts {
            PLAN_MODE_MANAGER.add_allowed_prompt(prompt.clone());
        }

        // 请求用户批准
        let (approved, message) = Self::request_approval(&allowed_prompts)?;

        // 获取计划信息
        let state = PLAN_MODE_MANAGER.get_state();
        let plan_id = state.plan_id.clone().unwrap_or_else(|| "unknown".to_string());
        let plan_content = state.plan_content.clone().unwrap_or_else(|| "No plan content provided.".to_string());

        // 保存计划到文件
        if let Err(e) = Self::save_plan(&plan_id, &plan_content, &allowed_prompts, approved) {
            eprintln!("{} 保存计划文件失败: {}", "⚠️".yellow(), e);
        }

        if approved {
            // 批准计划
            PLAN_MODE_MANAGER.approve();

            println!();
            println!("{}", "✅ 计划已批准！".bright_green().bold());
            println!("{}", "现在可以开始执行计划。".bright_white());
            println!();

            // 退出计划模式但保留批准状态
            // 注意：这里不调用 exit()，因为我们需要保留权限信息

            Ok(ExitPlanModeOutput {
                success: true,
                approved: true,
                message,
                approved_prompts: allowed_prompts,
            })
        } else {
            // 退出计划模式
            PLAN_MODE_MANAGER.exit();

            println!();
            println!("{}", "❌ 计划未批准".bright_red().bold());
            println!("{}", message.bright_white());
            println!();

            Ok(ExitPlanModeOutput {
                success: true,
                approved: false,
                message,
                approved_prompts: Vec::new(),
            })
        }
    }
}

// ============================================================================
// 包装器
// ============================================================================

/// EnterPlanMode 工具包装器
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WrappedEnterPlanModeTool {
    inner: EnterPlanModeTool,
}

impl WrappedEnterPlanModeTool {
    pub fn new() -> Self {
        Self {
            inner: EnterPlanModeTool,
        }
    }
}

impl Default for WrappedEnterPlanModeTool {
    fn default() -> Self {
        Self::new()
    }
}

impl Tool for WrappedEnterPlanModeTool {
    const NAME: &'static str = "enter_plan_mode";

    type Error = FileToolError;
    type Args = EnterPlanModeArgs;
    type Output = EnterPlanModeOutput;

    async fn definition(&self, prompt: String) -> ToolDefinition {
        self.inner.definition(prompt).await
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        self.inner.call(args).await
    }
}

/// ExitPlanMode 工具包装器
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WrappedExitPlanModeTool {
    inner: ExitPlanModeTool,
}

impl WrappedExitPlanModeTool {
    pub fn new() -> Self {
        Self {
            inner: ExitPlanModeTool,
        }
    }
}

impl Default for WrappedExitPlanModeTool {
    fn default() -> Self {
        Self::new()
    }
}

impl Tool for WrappedExitPlanModeTool {
    const NAME: &'static str = "exit_plan_mode";

    type Error = FileToolError;
    type Args = ExitPlanModeArgs;
    type Output = ExitPlanModeOutput;

    async fn definition(&self, prompt: String) -> ToolDefinition {
        self.inner.definition(prompt).await
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        self.inner.call(args).await
    }
}

// ============================================================================
// 辅助函数
// ============================================================================

/// 检查当前是否在计划模式中
pub fn is_in_plan_mode() -> bool {
    PLAN_MODE_MANAGER.is_active()
}

/// 检查计划是否已被批准
pub fn is_plan_approved() -> bool {
    PLAN_MODE_MANAGER.is_approved()
}

/// 检查操作是否被允许
pub fn is_operation_allowed(tool: &str, operation: &str) -> bool {
    PLAN_MODE_MANAGER.is_allowed(tool, operation)
}

/// 设置计划内容（供 Agent 在计划模式中使用）
pub fn set_plan_content(content: &str) {
    PLAN_MODE_MANAGER.set_plan_content(content.to_string());
}

/// 获取当前计划状态
pub fn get_plan_state() -> PlanModeState {
    PLAN_MODE_MANAGER.get_state()
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allowed_prompt_creation() {
        let prompt = AllowedPrompt::new("Bash", "run tests");
        assert_eq!(prompt.tool, "Bash");
        assert_eq!(prompt.prompt, "run tests");
    }

    #[test]
    fn test_allowed_prompt_matches() {
        let prompt = AllowedPrompt::new("Bash", "run tests");
        assert!(prompt.matches("Bash", "tests"));
        assert!(prompt.matches("Bash", "run"));
        assert!(!prompt.matches("Write", "tests"));
        assert!(!prompt.matches("Bash", "install"));
    }

    #[test]
    fn test_plan_mode_state_default() {
        let state = PlanModeState::default();
        assert!(!state.active);
        assert!(state.plan_id.is_none());
        assert!(state.allowed_prompts.is_empty());
        assert!(!state.approved);
    }

    #[test]
    fn test_plan_mode_state_enter() {
        let mut state = PlanModeState::default();
        let plan_id = state.enter();

        assert!(state.active);
        assert!(plan_id.starts_with("plan_"));
        assert!(state.plan_id.is_some());
        assert!(state.plan_file.is_some());
        assert!(!state.approved);
    }

    #[test]
    fn test_plan_mode_state_exit() {
        let mut state = PlanModeState::default();
        state.enter();
        state.approve();
        state.exit();

        assert!(!state.active);
        assert!(!state.approved);
    }

    #[test]
    fn test_plan_mode_state_is_allowed() {
        let mut state = PlanModeState::default();
        state.enter();
        state.add_allowed_prompt(AllowedPrompt::new("Bash", "run tests"));
        state.approve();

        assert!(state.is_allowed("Bash", "tests"));
        assert!(!state.is_allowed("Write", "tests"));
    }
}
