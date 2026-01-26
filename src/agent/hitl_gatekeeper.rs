//! Human-in-the-Loop Gatekeeper
//!
//! 在工具调用前进行智能决策，判断是否需要人工确认。

use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// HITL Gatekeeper 配置
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HitlConfig {
    /// 信任度设置
    pub trust: TrustConfig,
}

impl Default for HitlConfig {
    fn default() -> Self {
        Self {
            trust: TrustConfig::default(),
        }
    }
}



/// 信任度配置
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TrustConfig {
    /// 初始信任分数 (0.0 - 1.0)
    pub initial_score: f32,

    /// 自动批准低风险操作的信任阈值
    pub auto_approve_threshold: f32,

    /// 确认一次后提高的信任分数
    pub increment: f32,

    /// 拒绝一次后降低的信任分数
    pub decrement: f32,
}

impl Default for TrustConfig {
    fn default() -> Self {
        Self {
            initial_score: 0.5,
            auto_approve_threshold: 0.8,
            increment: 0.02,
            decrement: 0.05,
        }
    }
}

/// 工具调用请求
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize)]
pub struct ToolCallRequest {
    /// 工具名称
    pub tool_name: String,

    /// 工具参数
    pub args: serde_json::Value,

    /// 上下文信息
    pub context: OperationContext,
}

/// 操作上下文
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize)]
pub struct OperationContext {
    /// 用户最近的历史操作
    pub recent_operations: Vec<String>,

    /// 当前任务描述
    pub current_task: Option<String>,

    /// 是否有 git 仓库
    pub has_git: bool,

    /// 当前 git 分支
    pub git_branch: Option<String>,
}

/// HITL 决策结果
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HitlDecision {
    /// 直接执行，不需要确认
    ExecuteDirectly {
        reason: String,
    },

    /// 需要用户确认（是/否）
    RequireConfirmation {
        reason: String,
        warning_level: WarningLevel,
    },

    /// 需要用户选择（多选一）
    RequireChoice {
        question: String,
        options: Vec<UserChoice>,
        default: String,
    },

    /// 拒绝执行
    Reject {
        reason: String,
        suggestion: Option<String>,
    },
}

/// 警告级别
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WarningLevel {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

/// 用户选项
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserChoice {
    pub label: String,
    pub description: String,
}

/// HITL Gatekeeper
#[allow(dead_code)]
///
/// 轻量级的人机交互决策层，不需要额外的 AI 模型，
/// 直接使用规则引擎和信任度系统进行智能决策。
pub struct HitlGatekeeper {
    config: HitlConfig,
    trust_score: Arc<tokio::sync::Mutex<f32>>,
    operation_history: Arc<tokio::sync::Mutex<Vec<String>>>,
}

impl HitlGatekeeper {
    /// 创建新的 HITL Gatekeeper
    #[allow(dead_code)]
    pub fn new(config: HitlConfig) -> Result<Self, HitlError> {
        let initial_score = config.trust.initial_score;
        Ok(Self {
            config,
            trust_score: Arc::new(tokio::sync::Mutex::new(initial_score)),
            operation_history: Arc::new(tokio::sync::Mutex::new(Vec::new())),
        })
    }

    #[allow(dead_code)]
    /// 评估工具调用是否需要人工确认
    pub async fn evaluate_tool_call(
        &self,
        request: ToolCallRequest,
    ) -> Result<HitlDecision, HitlError> {
        // 1. 快速路径：已知的低风险操作
        if let Some(decision) = self.quick_path(&request).await {
            return Ok(decision);
        }

        // 2. 检查信任分数
        let trust_score = *self.trust_score.lock().await;
        if trust_score >= self.config.trust.auto_approve_threshold {
            // 高信任度：对于中低风险操作自动批准
            if self.is_low_risk_tool(&request.tool_name) {
                return Ok(HitlDecision::ExecuteDirectly {
                    reason: format!("信任分数较高 ({:.2})，自动批准低风险操作", trust_score),
                });
            }
        }

        // 3. 使用规则判断（简化版，暂时不用 AI）
        Ok(self.rule_based_decision(request))
    }

    #[allow(dead_code)]
    /// 记录操作成功（提高信任分数）
    pub async fn record_success(&self, operation: String) {
        let mut score = self.trust_score.lock().await;
        *score = (*score + self.config.trust.increment).min(1.0);
        drop(score); // 释放锁

        let mut history = self.operation_history.lock().await;
        history.push(operation);

        // 只保留最近 100 条
        if history.len() > 100 {
            let len = history.len();
            history.drain(0..len - 100);
        }
    }

    #[allow(dead_code)]
    /// 记录用户拒绝（降低信任分数）
    pub async fn record_rejection(&self) {
        let mut score = self.trust_score.lock().await;
        *score = (*score - self.config.trust.decrement).max(0.0);
    }

    /// 快速路径：已知的低风险操作
    async fn quick_path(&self, request: &ToolCallRequest) -> Option<HitlDecision> {
        match request.tool_name.as_str() {
            "read_file" | "glob" | "grep_search" | "scan_codebase" => {
                Some(HitlDecision::ExecuteDirectly {
                    reason: "只读操作，无风险".to_string(),
                })
            }
            "shell_execute" => {
                // 检查是否是安全的只读命令
                if let Some(cmd) = request.args.get("command").and_then(|c| c.as_str()) {
                    if self.is_safe_readonly_command(cmd) {
                        return Some(HitlDecision::ExecuteDirectly {
                            reason: "安全的只读命令".to_string(),
                        });
                    }
                }
                None
            }
            _ => None,
        }
    }

    /// 检查是否是安全的只读命令
    fn is_safe_readonly_command(&self, cmd: &str) -> bool {
        let safe_commands = [
            "git status",
            "git diff",
            "git log",
            "git show",
            "ls",
            "pwd",
            "cat",
            "echo",
            "which",
            "rustc --version",
            "cargo --version",
            "node --version",
            "python --version",
        ];

        let cmd_lower = cmd.trim().to_lowercase();
        for safe in &safe_commands {
            if cmd_lower.starts_with(safe) {
                return true;
            }
        }
        false
    }

    /// 检查是否是低风险工具
    fn is_low_risk_tool(&self, tool_name: &str) -> bool {
        matches!(tool_name,
            "read_file" | "write_file" | "edit_file" |
            "glob" | "grep_search" | "scan_codebase"
        )
    }

    /// 基于规则的决策（简化版）
    fn rule_based_decision(&self, request: ToolCallRequest) -> HitlDecision {
        match request.tool_name.as_str() {
            "delete_file" => {
                // 删除文件总是需要确认
                HitlDecision::RequireConfirmation {
                    reason: "即将删除文件".to_string(),
                    warning_level: WarningLevel::High,
                }
            }
            "shell_execute" => {
                // 检查是否是危险命令
                if let Some(cmd) = request.args.get("command").and_then(|c| c.as_str()) {
                    if self.is_dangerous_command(cmd) {
                        HitlDecision::Reject {
                            reason: "检测到危险命令".to_string(),
                            suggestion: Some("请考虑使用更安全的替代方案".to_string()),
                        }
                    } else {
                        HitlDecision::RequireConfirmation {
                            reason: format!("即将执行命令: {}", cmd),
                            warning_level: WarningLevel::Medium,
                        }
                    }
                } else {
                    HitlDecision::RequireConfirmation {
                        reason: "即将执行命令".to_string(),
                        warning_level: WarningLevel::Medium,
                    }
                }
            }
            "edit_file" => {
                // 编辑文件 (Wrapper 版本) 已经内置了 diff 预览和确认
                // 因此这里不需要再次确认，避免双重确认
                HitlDecision::ExecuteDirectly {
                    reason: "工具内置确认".to_string(),
                }
            }
            "write_file" | "multiedit" => {
                // 其他修改文件的工具需要确认
                HitlDecision::RequireConfirmation {
                    reason: "即将修改文件".to_string(),
                    warning_level: WarningLevel::Low,
                }
            }
            _ => {
                // 其他工具：根据上下文判断
                HitlDecision::ExecuteDirectly {
                    reason: "未知工具，默认执行".to_string(),
                }
            }
        }
    }

    /// 检查是否是危险命令
    fn is_dangerous_command(&self, cmd: &str) -> bool {
        let cmd_lower = cmd.trim().to_lowercase();
        let dangerous_patterns = [
            "rm -rf",
            "rm -fr",
            ":(){:|:&};:", // fork bomb
            "dd if=/dev/zero",
            "mkfs",
            "format",
            "shutdown",
            "reboot",
            "kill -9",
        ];

        for pattern in &dangerous_patterns {
            if cmd_lower.contains(pattern) {
                return true;
            }
        }
        false
    }

    /// 获取当前信任分数
    #[allow(dead_code)]
    pub async fn trust_score(&self) -> f32 {
        *self.trust_score.lock().await
    }
}

/// HITL 错误类型
#[derive(Debug, thiserror::Error)]
pub enum HitlError {
    #[error("配置错误: {0}")]
    ConfigError(String),

    #[error("评估错误: {0}")]
    EvaluationError(String),

    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),
}

impl From<HitlError> for String {
    fn from(err: HitlError) -> String {
        err.to_string()
    }
}
