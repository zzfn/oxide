//! 工作流状态定义
//!
//! 定义工作流编排器的状态机和相关类型。

use serde::{Deserialize, Serialize};
use std::fmt;

/// 工作流阶段
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkflowPhase {
    /// 空闲状态，等待任务
    Idle,
    
    /// 规划阶段 - 分析任务并制定计划
    Planning,
    
    /// 执行阶段 - 执行工具调用或委派子任务
    Acting,
    
    /// 观察阶段 - 收集执行结果和观察数据
    Observing,
    
    /// 反思阶段 - 评估进展并决定下一步
    Reflecting,
    
    /// 完成状态 - 目标已达成
    Complete,
    
    /// 失败状态 - 遇到不可恢复错误
    Failed,
}

impl fmt::Display for WorkflowPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WorkflowPhase::Idle => write!(f, "Idle"),
            WorkflowPhase::Planning => write!(f, "Planning"),
            WorkflowPhase::Acting => write!(f, "Acting"),
            WorkflowPhase::Observing => write!(f, "Observing"),
            WorkflowPhase::Reflecting => write!(f, "Reflecting"),
            WorkflowPhase::Complete => write!(f, "Complete"),
            WorkflowPhase::Failed => write!(f, "Failed"),
        }
    }
}

impl WorkflowPhase {
    /// 判断是否为终止状态
    pub fn is_terminal(&self) -> bool {
        matches!(self, WorkflowPhase::Complete | WorkflowPhase::Failed)
    }
    
    /// 获取下一个阶段
    pub fn next(&self) -> Option<WorkflowPhase> {
        match self {
            WorkflowPhase::Idle => Some(WorkflowPhase::Planning),
            WorkflowPhase::Planning => Some(WorkflowPhase::Acting),
            WorkflowPhase::Acting => Some(WorkflowPhase::Observing),
            WorkflowPhase::Observing => Some(WorkflowPhase::Reflecting),
            WorkflowPhase::Reflecting => None, // Reflect 之后需要根据结果决定
            WorkflowPhase::Complete | WorkflowPhase::Failed => None,
        }
    }
}

/// 工作流状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowState {
    /// 当前阶段
    pub phase: WorkflowPhase,
    
    /// 当前迭代次数
    pub iteration: u32,
    
    /// 最大迭代次数
    pub max_iterations: u32,
    
    /// 用户请求的原始描述
    pub user_request: String,
    
    /// 工作流开始时间
    pub started_at: std::time::SystemTime,
    
    /// 最后更新时间
    pub updated_at: std::time::SystemTime,
    
    /// 是否需要用户干预
    pub requires_user_intervention: bool,
    
    /// 失败原因（如果失败）
    pub failure_reason: Option<String>,
}

impl WorkflowState {
    /// 创建新的工作流状态
    pub fn new(user_request: String, max_iterations: u32) -> Self {
        let now = std::time::SystemTime::now();
        Self {
            phase: WorkflowPhase::Idle,
            iteration: 0,
            max_iterations,
            user_request,
            started_at: now,
            updated_at: now,
            requires_user_intervention: false,
            failure_reason: None,
        }
    }
    
    /// 转换到下一个阶段
    pub fn transition_to(&mut self, phase: WorkflowPhase) {
        self.phase = phase;
        self.updated_at = std::time::SystemTime::now();
        
        // 从 Reflecting 回到 Planning 时增加迭代次数
        if phase == WorkflowPhase::Planning && self.iteration > 0 {
            self.iteration += 1;
        } else if phase == WorkflowPhase::Planning {
            self.iteration = 1;
        }
    }
    
    /// 检查是否已达到最大迭代次数
    pub fn has_reached_max_iterations(&self) -> bool {
        self.iteration >= self.max_iterations
    }
    
    /// 检查是否应该终止
    pub fn should_terminate(&self) -> bool {
        self.phase.is_terminal() || self.has_reached_max_iterations()
    }
    
    /// 标记需要用户干预
    pub fn mark_requires_intervention(&mut self, reason: String) {
        self.requires_user_intervention = true;
        self.transition_to(WorkflowPhase::Failed);
        self.failure_reason = Some(reason);
    }
    
    /// 标记完成
    pub fn mark_complete(&mut self) {
        self.transition_to(WorkflowPhase::Complete);
    }
    
    /// 标记失败
    pub fn mark_failed(&mut self, reason: String) {
        self.transition_to(WorkflowPhase::Failed);
        self.failure_reason = Some(reason);
    }
    
    /// 获取经过的时间（毫秒）
    pub fn elapsed_ms(&self) -> u128 {
        self.updated_at
            .duration_since(self.started_at)
            .unwrap_or_default()
            .as_millis()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_workflow_phase_terminal() {
        assert!(!WorkflowPhase::Idle.is_terminal());
        assert!(!WorkflowPhase::Planning.is_terminal());
        assert!(WorkflowPhase::Complete.is_terminal());
        assert!(WorkflowPhase::Failed.is_terminal());
    }
    
    #[test]
    fn test_workflow_phase_next() {
        assert_eq!(WorkflowPhase::Idle.next(), Some(WorkflowPhase::Planning));
        assert_eq!(WorkflowPhase::Planning.next(), Some(WorkflowPhase::Acting));
        assert_eq!(WorkflowPhase::Acting.next(), Some(WorkflowPhase::Observing));
        assert_eq!(WorkflowPhase::Observing.next(), Some(WorkflowPhase::Reflecting));
        assert_eq!(WorkflowPhase::Reflecting.next(), None);
        assert_eq!(WorkflowPhase::Complete.next(), None);
    }
    
    #[test]
    fn test_workflow_state_creation() {
        let state = WorkflowState::new("Test request".to_string(), 10);
        assert_eq!(state.phase, WorkflowPhase::Idle);
        assert_eq!(state.iteration, 0);
        assert_eq!(state.max_iterations, 10);
        assert!(!state.should_terminate());
    }
    
    #[test]
    fn test_workflow_state_transitions() {
        let mut state = WorkflowState::new("Test".to_string(), 5);
        
        state.transition_to(WorkflowPhase::Planning);
        assert_eq!(state.phase, WorkflowPhase::Planning);
        assert_eq!(state.iteration, 1);
        
        state.transition_to(WorkflowPhase::Acting);
        state.transition_to(WorkflowPhase::Observing);
        state.transition_to(WorkflowPhase::Reflecting);
        
        // 回到 Planning 应该增加迭代
        state.transition_to(WorkflowPhase::Planning);
        assert_eq!(state.iteration, 2);
    }
    
    #[test]
    fn test_max_iterations() {
        let mut state = WorkflowState::new("Test".to_string(), 2);
        
        state.transition_to(WorkflowPhase::Planning);
        assert!(!state.has_reached_max_iterations());
        
        state.transition_to(WorkflowPhase::Reflecting);
        state.transition_to(WorkflowPhase::Planning);
        assert!(state.has_reached_max_iterations());
        assert!(state.should_terminate());
    }
}
