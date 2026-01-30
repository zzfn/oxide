//! 任务类型定义

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 任务状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    /// 待处理
    Pending,
    /// 进行中
    InProgress,
    /// 已完成
    Completed,
    /// 已删除
    Deleted,
}

impl TaskStatus {
    /// 检查状态转换是否合法
    pub fn can_transition_to(&self, new_status: TaskStatus) -> bool {
        use TaskStatus::*;
        match (self, new_status) {
            // pending 可以转换到任何状态
            (Pending, _) => true,
            // in_progress 可以转换到 completed 或 deleted
            (InProgress, Completed) | (InProgress, Deleted) => true,
            // completed 只能转换到 deleted
            (Completed, Deleted) => true,
            // deleted 不能转换到其他状态
            (Deleted, _) => false,
            // 其他转换不允许
            _ => false,
        }
    }
}

/// 任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// 任务 ID
    pub id: String,
    /// 任务标题（祈使句）
    pub subject: String,
    /// 详细描述
    pub description: String,
    /// 进行中显示文本（现在进行时）
    pub active_form: Option<String>,
    /// 任务状态
    pub status: TaskStatus,
    /// 任务所有者（子代理 ID）
    pub owner: Option<String>,
    /// 此任务阻塞的任务列表
    pub blocks: Vec<String>,
    /// 阻塞此任务的任务列表
    pub blocked_by: Vec<String>,
    /// 元数据（用于上下文传递）
    pub metadata: HashMap<String, serde_json::Value>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

impl Task {
    /// 创建新任务
    pub fn new(subject: String, description: String, active_form: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            subject,
            description,
            active_form,
            status: TaskStatus::Pending,
            owner: None,
            blocks: Vec::new(),
            blocked_by: Vec::new(),
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// 更新任务状态
    pub fn update_status(&mut self, new_status: TaskStatus) -> Result<(), String> {
        if !self.status.can_transition_to(new_status) {
            return Err(format!(
                "无法从 {:?} 转换到 {:?}",
                self.status, new_status
            ));
        }
        self.status = new_status;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// 添加阻塞关系（此任务阻塞其他任务）
    pub fn add_blocks(&mut self, task_id: String) {
        if !self.blocks.contains(&task_id) {
            self.blocks.push(task_id);
            self.updated_at = Utc::now();
        }
    }

    /// 添加被阻塞关系（此任务被其他任务阻塞）
    pub fn add_blocked_by(&mut self, task_id: String) {
        if !self.blocked_by.contains(&task_id) {
            self.blocked_by.push(task_id);
            self.updated_at = Utc::now();
        }
    }

    /// 移除阻塞关系
    pub fn remove_blocks(&mut self, task_id: &str) {
        self.blocks.retain(|id| id != task_id);
        self.updated_at = Utc::now();
    }

    /// 移除被阻塞关系
    pub fn remove_blocked_by(&mut self, task_id: &str) {
        self.blocked_by.retain(|id| id != task_id);
        self.updated_at = Utc::now();
    }

    /// 设置所有者
    pub fn set_owner(&mut self, owner: Option<String>) {
        self.owner = owner;
        self.updated_at = Utc::now();
    }

    /// 更新元数据
    pub fn update_metadata(&mut self, key: String, value: serde_json::Value) {
        self.metadata.insert(key, value);
        self.updated_at = Utc::now();
    }

    /// 删除元数据
    pub fn remove_metadata(&mut self, key: &str) {
        self.metadata.remove(key);
        self.updated_at = Utc::now();
    }

    /// 检查任务是否可以开始（没有被阻塞）
    pub fn is_ready(&self) -> bool {
        self.blocked_by.is_empty() && self.status == TaskStatus::Pending
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_creation() {
        let task = Task::new(
            "测试任务".to_string(),
            "这是一个测试任务".to_string(),
            Some("正在测试".to_string()),
        );

        assert_eq!(task.subject, "测试任务");
        assert_eq!(task.status, TaskStatus::Pending);
        assert!(task.owner.is_none());
        assert!(task.blocks.is_empty());
        assert!(task.blocked_by.is_empty());
    }

    #[test]
    fn test_status_transitions() {
        use TaskStatus::*;

        // pending 可以转换到任何状态
        assert!(Pending.can_transition_to(InProgress));
        assert!(Pending.can_transition_to(Completed));
        assert!(Pending.can_transition_to(Deleted));

        // in_progress 可以转换到 completed 或 deleted
        assert!(InProgress.can_transition_to(Completed));
        assert!(InProgress.can_transition_to(Deleted));
        assert!(!InProgress.can_transition_to(Pending));

        // completed 只能转换到 deleted
        assert!(Completed.can_transition_to(Deleted));
        assert!(!Completed.can_transition_to(Pending));
        assert!(!Completed.can_transition_to(InProgress));

        // deleted 不能转换到其他状态
        assert!(!Deleted.can_transition_to(Pending));
        assert!(!Deleted.can_transition_to(InProgress));
        assert!(!Deleted.can_transition_to(Completed));
    }

    #[test]
    fn test_task_update_status() {
        let mut task = Task::new(
            "测试".to_string(),
            "描述".to_string(),
            None,
        );

        // pending -> in_progress
        assert!(task.update_status(TaskStatus::InProgress).is_ok());
        assert_eq!(task.status, TaskStatus::InProgress);

        // in_progress -> completed
        assert!(task.update_status(TaskStatus::Completed).is_ok());
        assert_eq!(task.status, TaskStatus::Completed);

        // completed -> pending (不允许)
        assert!(task.update_status(TaskStatus::Pending).is_err());
    }

    #[test]
    fn test_task_dependencies() {
        let mut task = Task::new(
            "测试".to_string(),
            "描述".to_string(),
            None,
        );

        // 添加依赖
        task.add_blocked_by("task-1".to_string());
        task.add_blocked_by("task-2".to_string());
        assert_eq!(task.blocked_by.len(), 2);
        assert!(!task.is_ready());

        // 移除依赖
        task.remove_blocked_by("task-1");
        assert_eq!(task.blocked_by.len(), 1);

        task.remove_blocked_by("task-2");
        assert_eq!(task.blocked_by.len(), 0);
        assert!(task.is_ready());
    }
}
