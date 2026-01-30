//! 任务管理器

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

use super::errors::TaskError;
use super::types::{Task, TaskStatus};

/// 后台任务信息（向后兼容）
#[derive(Debug, Clone)]
pub struct BackgroundTask {
    pub _id: String,
    pub _command: String,
    pub output: String,
    pub is_running: bool,
    pub exit_code: Option<i32>,
}

impl BackgroundTask {
    /// 创建新的后台任务
    pub fn new(id: String, command: String) -> Self {
        Self {
            _id: id,
            _command: command,
            output: String::new(),
            is_running: true,
            exit_code: None,
        }
    }
}

/// 任务管理器
#[derive(Clone)]
pub struct TaskManager {
    /// 任务存储（新的任务系统）
    tasks: Arc<RwLock<HashMap<String, Task>>>,
    /// 后台任务存储（向后兼容 Bash 后台任务）
    background_tasks: Arc<RwLock<HashMap<String, BackgroundTask>>>,
    /// 任务计数器（用于生成简短的任务 ID）
    task_counter: Arc<RwLock<u32>>,
}

impl TaskManager {
    /// 创建新的任务管理器
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            background_tasks: Arc::new(RwLock::new(HashMap::new())),
            task_counter: Arc::new(RwLock::new(0)),
        }
    }

    /// 生成下一个任务 ID
    async fn next_task_id(&self) -> String {
        let mut counter = self.task_counter.write().await;
        *counter += 1;
        counter.to_string()
    }

    // ==================== 任务 CRUD 操作 ====================

    /// 创建新任务
    pub async fn create_task(
        &self,
        subject: String,
        description: String,
        active_form: Option<String>,
    ) -> Result<String, TaskError> {
        let task_id = self.next_task_id().await;
        let mut task = Task::new(subject, description, active_form);
        task.id = task_id.clone();

        let mut tasks = self.tasks.write().await;
        tasks.insert(task_id.clone(), task);

        Ok(task_id)
    }

    /// 获取任务
    pub async fn get_task(&self, task_id: &str) -> Result<Task, TaskError> {
        let tasks = self.tasks.read().await;
        tasks
            .get(task_id)
            .cloned()
            .ok_or_else(|| TaskError::not_found(task_id))
    }

    /// 列出所有任务
    pub async fn list_tasks(&self) -> Vec<Task> {
        let tasks = self.tasks.read().await;
        let mut task_list: Vec<Task> = tasks.values().cloned().collect();
        // 按 ID 排序
        task_list.sort_by(|a, b| {
            a.id.parse::<u32>()
                .unwrap_or(0)
                .cmp(&b.id.parse::<u32>().unwrap_or(0))
        });
        task_list
    }

    /// 更新任务状态
    pub async fn update_task_status(
        &self,
        task_id: &str,
        new_status: TaskStatus,
    ) -> Result<(), TaskError> {
        let mut tasks = self.tasks.write().await;
        let task = tasks
            .get_mut(task_id)
            .ok_or_else(|| TaskError::not_found(task_id))?;

        task.update_status(new_status)
            .map_err(TaskError::InvalidStatusTransition)?;

        Ok(())
    }

    /// 更新任务所有者
    pub async fn update_task_owner(
        &self,
        task_id: &str,
        owner: Option<String>,
    ) -> Result<(), TaskError> {
        let mut tasks = self.tasks.write().await;
        let task = tasks
            .get_mut(task_id)
            .ok_or_else(|| TaskError::not_found(task_id))?;

        task.set_owner(owner);
        Ok(())
    }

    /// 更新任务元数据
    pub async fn update_task_metadata(
        &self,
        task_id: &str,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Result<(), TaskError> {
        let mut tasks = self.tasks.write().await;
        let task = tasks
            .get_mut(task_id)
            .ok_or_else(|| TaskError::not_found(task_id))?;

        for (key, value) in metadata {
            if value.is_null() {
                task.remove_metadata(&key);
            } else {
                task.update_metadata(key, value);
            }
        }

        Ok(())
    }

    /// 更新任务主题和描述
    pub async fn update_task_content(
        &self,
        task_id: &str,
        subject: Option<String>,
        description: Option<String>,
        active_form: Option<String>,
    ) -> Result<(), TaskError> {
        let mut tasks = self.tasks.write().await;
        let task = tasks
            .get_mut(task_id)
            .ok_or_else(|| TaskError::not_found(task_id))?;

        if let Some(subject) = subject {
            task.subject = subject;
        }
        if let Some(description) = description {
            task.description = description;
        }
        if let Some(active_form) = active_form {
            task.active_form = Some(active_form);
        }

        task.updated_at = chrono::Utc::now();
        Ok(())
    }

    // ==================== 依赖关系管理 ====================

    /// 添加任务依赖关系
    pub async fn add_dependency(
        &self,
        task_id: &str,
        blocks: Vec<String>,
        blocked_by: Vec<String>,
    ) -> Result<(), TaskError> {
        let mut tasks = self.tasks.write().await;

        // 验证所有任务都存在
        for id in blocks.iter().chain(blocked_by.iter()) {
            if !tasks.contains_key(id) {
                return Err(TaskError::not_found(id));
            }
        }

        // 添加 blocks 关系
        for blocked_task_id in &blocks {
            if let Some(task) = tasks.get_mut(task_id) {
                task.add_blocks(blocked_task_id.clone());
            }
            if let Some(blocked_task) = tasks.get_mut(blocked_task_id) {
                blocked_task.add_blocked_by(task_id.to_string());
            }
        }

        // 添加 blocked_by 关系
        for blocking_task_id in &blocked_by {
            if let Some(task) = tasks.get_mut(task_id) {
                task.add_blocked_by(blocking_task_id.clone());
            }
            if let Some(blocking_task) = tasks.get_mut(blocking_task_id) {
                blocking_task.add_blocks(task_id.to_string());
            }
        }

        // 检测循环依赖
        drop(tasks); // 释放写锁
        self.detect_circular_dependency().await?;

        Ok(())
    }

    /// 检测循环依赖（使用 DFS）
    async fn detect_circular_dependency(&self) -> Result<(), TaskError> {
        let tasks = self.tasks.read().await;

        for task_id in tasks.keys() {
            let mut visited = HashSet::new();
            let mut rec_stack = HashSet::new();

            if self.has_cycle_dfs(task_id, &tasks, &mut visited, &mut rec_stack) {
                return Err(TaskError::circular_dependency(format!(
                    "任务 {} 存在循环依赖",
                    task_id
                )));
            }
        }

        Ok(())
    }

    /// DFS 检测循环
    fn has_cycle_dfs(
        &self,
        task_id: &str,
        tasks: &HashMap<String, Task>,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
    ) -> bool {
        if rec_stack.contains(task_id) {
            return true; // 发现循环
        }

        if visited.contains(task_id) {
            return false; // 已经访问过，没有循环
        }

        visited.insert(task_id.to_string());
        rec_stack.insert(task_id.to_string());

        if let Some(task) = tasks.get(task_id) {
            for blocked_id in &task.blocks {
                if self.has_cycle_dfs(blocked_id, tasks, visited, rec_stack) {
                    return true;
                }
            }
        }

        rec_stack.remove(task_id);
        false
    }

    /// 获取可执行的任务（没有被阻塞的 pending 任务）
    pub async fn get_ready_tasks(&self) -> Vec<Task> {
        let tasks = self.tasks.read().await;
        tasks
            .values()
            .filter(|task| task.is_ready())
            .cloned()
            .collect()
    }

    // ==================== 后台任务管理（向后兼容）====================

    /// 获取后台任务管理器（用于 Bash 工具）
    pub fn background_tasks(&self) -> Arc<RwLock<HashMap<String, BackgroundTask>>> {
        self.background_tasks.clone()
    }

    /// 添加后台任务
    pub async fn add_background_task(&self, task_id: String, command: String) {
        let mut tasks = self.background_tasks.write().await;
        tasks.insert(task_id.clone(), BackgroundTask::new(task_id, command));
    }

    /// 获取后台任务
    pub async fn get_background_task(&self, task_id: &str) -> Option<BackgroundTask> {
        let tasks = self.background_tasks.read().await;
        tasks.get(task_id).cloned()
    }

    /// 更新后台任务
    pub async fn update_background_task<F>(&self, task_id: &str, f: F)
    where
        F: FnOnce(&mut BackgroundTask),
    {
        let mut tasks = self.background_tasks.write().await;
        if let Some(task) = tasks.get_mut(task_id) {
            f(task);
        }
    }
}

impl Default for TaskManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_and_get_task() {
        let manager = TaskManager::new();

        let task_id = manager
            .create_task(
                "测试任务".to_string(),
                "这是一个测试".to_string(),
                Some("正在测试".to_string()),
            )
            .await
            .unwrap();

        let task = manager.get_task(&task_id).await.unwrap();
        assert_eq!(task.subject, "测试任务");
        assert_eq!(task.status, TaskStatus::Pending);
    }

    #[tokio::test]
    async fn test_list_tasks() {
        let manager = TaskManager::new();

        manager
            .create_task("任务1".to_string(), "描述1".to_string(), None)
            .await
            .unwrap();
        manager
            .create_task("任务2".to_string(), "描述2".to_string(), None)
            .await
            .unwrap();

        let tasks = manager.list_tasks().await;
        assert_eq!(tasks.len(), 2);
    }

    #[tokio::test]
    async fn test_update_task_status() {
        let manager = TaskManager::new();

        let task_id = manager
            .create_task("测试".to_string(), "描述".to_string(), None)
            .await
            .unwrap();

        manager
            .update_task_status(&task_id, TaskStatus::InProgress)
            .await
            .unwrap();

        let task = manager.get_task(&task_id).await.unwrap();
        assert_eq!(task.status, TaskStatus::InProgress);
    }

    #[tokio::test]
    async fn test_task_dependencies() {
        let manager = TaskManager::new();

        let task1 = manager
            .create_task("任务1".to_string(), "描述1".to_string(), None)
            .await
            .unwrap();
        let task2 = manager
            .create_task("任务2".to_string(), "描述2".to_string(), None)
            .await
            .unwrap();

        // task2 依赖 task1
        manager
            .add_dependency(&task2, vec![], vec![task1.clone()])
            .await
            .unwrap();

        let task = manager.get_task(&task2).await.unwrap();
        assert_eq!(task.blocked_by.len(), 1);
        assert!(!task.is_ready());

        let ready_tasks = manager.get_ready_tasks().await;
        assert_eq!(ready_tasks.len(), 1);
        assert_eq!(ready_tasks[0].id, task1);
    }

    #[tokio::test]
    async fn test_circular_dependency_detection() {
        let manager = TaskManager::new();

        let task1 = manager
            .create_task("任务1".to_string(), "描述1".to_string(), None)
            .await
            .unwrap();
        let task2 = manager
            .create_task("任务2".to_string(), "描述2".to_string(), None)
            .await
            .unwrap();

        // task1 blocks task2
        manager
            .add_dependency(&task1, vec![task2.clone()], vec![])
            .await
            .unwrap();

        // task2 blocks task1 (会形成循环)
        let result = manager
            .add_dependency(&task2, vec![task1.clone()], vec![])
            .await;

        assert!(result.is_err());
    }
}
