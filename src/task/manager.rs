//! 任务管理器
//!
//! 管理后台任务的创建、执行和追踪。

use crate::agent::types::AgentType;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::task::JoinHandle;
use uuid::Uuid;

/// 任务 ID 类型
pub type TaskId = String;

/// 任务状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    /// 等待中
    Pending,

    /// 进行中
    InProgress,

    /// 已完成
    Completed,

    /// 失败
    Failed,
}

/// 任务信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// 任务 ID
    pub id: TaskId,

    /// 任务名称
    pub name: String,

    /// 任务描述
    pub description: String,

    /// 提示词
    pub prompt: String,

    /// 任务状态
    pub status: TaskStatus,

    /// Agent 类型
    pub agent_type: AgentType,

    /// 创建时间
    pub created_at: DateTime<Utc>,

    /// 开始时间
    pub started_at: Option<DateTime<Utc>>,

    /// 完成时间
    pub completed_at: Option<DateTime<Utc>>,

    /// 输出文件路径
    pub output_file: Option<PathBuf>,

    /// 错误信息
    pub error: Option<String>,
}

impl Task {
    /// 创建新任务
    #[allow(dead_code)]
    pub fn new(name: String, description: String, prompt: String, agent_type: AgentType) -> Self {
        let id = Uuid::new_v4().to_string();
        Self {
            id,
            name,
            description,
            prompt,
            status: TaskStatus::Pending,
            agent_type,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            output_file: None,
            error: None,
        }
    }

    /// 标记任务为进行中
    #[allow(dead_code)]
    pub fn mark_in_progress(&mut self) {
        self.status = TaskStatus::InProgress;
        self.started_at = Some(Utc::now());
    }

    /// 标记任务为已完成
    #[allow(dead_code)]
    pub fn mark_completed(&mut self, output_file: Option<PathBuf>) {
        self.status = TaskStatus::Completed;
        self.completed_at = Some(Utc::now());
        self.output_file = output_file;
    }

    /// 标记任务为失败
    #[allow(dead_code)]
    pub fn mark_failed(&mut self, error: String) {
        self.status = TaskStatus::Failed;
        self.completed_at = Some(Utc::now());
        self.error = Some(error);
    }

    /// 获取任务运行时长
    pub fn duration(&self) -> Option<chrono::Duration> {
        match (self.started_at, self.completed_at) {
            (Some(start), Some(end)) => Some(end.signed_duration_since(start)),
            (Some(start), None) => Some(Utc::now().signed_duration_since(start)),
            _ => None,
        }
    }
}

/// 任务管理器
pub struct TaskManager {
    /// 任务存储
    tasks: Arc<Mutex<HashMap<TaskId, Task>>>,

    /// 活跃的异步任务句柄
    active_handles: Arc<Mutex<HashMap<TaskId, JoinHandle<()>>>>,

    /// 存储目录
    storage_dir: PathBuf,
}

impl TaskManager {
    /// 创建新的任务管理器
    pub fn new(storage_dir: PathBuf) -> Result<Self> {
        // 确保存储目录存在
        fs::create_dir_all(&storage_dir)
            .context(format!("无法创建任务存储目录: {}", storage_dir.display()))?;

        Ok(Self {
            tasks: Arc::new(Mutex::new(HashMap::new())),
            active_handles: Arc::new(Mutex::new(HashMap::new())),
            storage_dir,
        })
    }

    /// 创建任务存储目录路径
    fn task_storage_path(&self, task_id: &TaskId) -> PathBuf {
        self.storage_dir.join(format!("{}.json", task_id))
    }

    /// 创建任务输出文件路径
    fn task_output_path(&self, task_id: &TaskId) -> PathBuf {
        self.storage_dir.join(format!("{}.output.txt", task_id))
    }

    /// 保存任务到磁盘
    fn save_task(&self, task: &Task) -> Result<()> {
        let path = self.task_storage_path(&task.id);
        let json = serde_json::to_string_pretty(task)
            .context("序列化任务失败")?;
        fs::write(&path, json)
            .context(format!("无法写入任务文件: {}", path.display()))?;
        Ok(())
    }

    /// 从磁盘加载任务
    fn load_task(&self, task_id: &TaskId) -> Result<Option<Task>> {
        let path = self.task_storage_path(task_id);
        if !path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&path)
            .context(format!("无法读取任务文件: {}", path.display()))?;
        let task: Task = serde_json::from_str(&content)
            .context(format!("解析任务文件失败: {}", path.display()))?;
        Ok(Some(task))
    }

    /// 创建新任务
    #[allow(dead_code)]
    pub fn create_task(
        &self,
        name: String,
        description: String,
        prompt: String,
        agent_type: AgentType,
    ) -> Result<TaskId> {
        let task = Task::new(name, description, prompt, agent_type);
        let task_id = task.id.clone();

        // 保存任务到磁盘
        self.save_task(&task)?;

        // 添加到内存存储
        let mut tasks = self.tasks.lock().unwrap();
        tasks.insert(task_id.clone(), task);

        Ok(task_id)
    }

    /// 获取任务信息
    pub fn get_task(&self, task_id: &TaskId) -> Result<Option<Task>> {
        // 首先尝试从内存获取
        {
            let tasks = self.tasks.lock().unwrap();
            if let Some(task) = tasks.get(task_id) {
                return Ok(Some(task.clone()));
            }
        }

        // 如果内存中没有，尝试从磁盘加载
        if let Some(task) = self.load_task(task_id)? {
            // 加载到内存
            let mut tasks = self.tasks.lock().unwrap();
            tasks.insert(task_id.clone(), task.clone());
            Ok(Some(task))
        } else {
            Ok(None)
        }
    }

    /// 列出所有任务
    pub fn list_tasks(&self) -> Result<Vec<Task>> {
        // 读取存储目录中的所有任务文件
        let mut tasks = Vec::new();

        if self.storage_dir.exists() {
            for entry in fs::read_dir(&self.storage_dir)
                .context(format!("无法读取存储目录: {}", self.storage_dir.display()))?
            {
                let entry = entry?;
                let path = entry.path();

                // 只处理 .json 文件
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(task) = serde_json::from_str::<Task>(&content) {
                            tasks.push(task);
                        }
                    }
                }
            }
        }

        // 按创建时间倒序排序
        tasks.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok(tasks)
    }

    /// 更新任务状态
    pub fn update_task_status(&self, task_id: &TaskId, status: TaskStatus) -> Result<()> {
        let mut tasks = self.tasks.lock().unwrap();
        if let Some(task) = tasks.get_mut(task_id) {
            task.status = status;
            self.save_task(task)?;
        }
        Ok(())
    }

    /// 获取任务输出
    pub fn get_task_output(&self, task_id: &TaskId) -> Result<Option<String>> {
        let output_path = self.task_output_path(task_id);

        if output_path.exists() {
            let content = fs::read_to_string(&output_path)
                .context(format!("无法读取任务输出: {}", output_path.display()))?;
            Ok(Some(content))
        } else {
            Ok(None)
        }
    }

    /// 取消任务
    pub fn cancel_task(&self, task_id: &TaskId) -> Result<bool> {
        // 尝试取消活跃的异步任务
        let mut handles = self.active_handles.lock().unwrap();

        if let Some(handle) = handles.remove(task_id) {
            handle.abort();
            drop(handles);

            // 更新任务状态
            self.update_task_status(task_id, TaskStatus::Failed)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// 清理已完成的任务
    #[allow(dead_code)]
    pub fn cleanup_completed_tasks(&self, older_than: chrono::Duration) -> Result<usize> {
        let tasks = self.list_tasks()?;
        let cutoff_time = Utc::now() - older_than;
        let mut cleaned = 0;

        for task in tasks {
            if task.status == TaskStatus::Completed {
                if let Some(completed_at) = task.completed_at {
                    if completed_at < cutoff_time {
                        // 删除任务文件
                        let task_path = self.task_storage_path(&task.id);
                        if task_path.exists() {
                            fs::remove_file(task_path)?;
                        }

                        // 删除输出文件
                        if let Some(output_path) = task.output_file {
                            if output_path.exists() {
                                fs::remove_file(output_path)?;
                            }
                        }

                        // 从内存中移除
                        let mut tasks = self.tasks.lock().unwrap();
                        tasks.remove(&task.id);

                        cleaned += 1;
                    }
                }
            }
        }

        Ok(cleaned)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_task_creation() {
        let task = Task::new(
            "测试任务".to_string(),
            "测试描述".to_string(),
            "测试提示词".to_string(),
            AgentType::Explore,
        );

        assert_eq!(task.status, TaskStatus::Pending);
        assert!(task.started_at.is_none());
        assert!(task.completed_at.is_none());
    }

    #[test]
    fn test_task_status_transitions() {
        let mut task = Task::new(
            "测试任务".to_string(),
            "测试描述".to_string(),
            "测试提示词".to_string(),
            AgentType::Explore,
        );

        task.mark_in_progress();
        assert_eq!(task.status, TaskStatus::InProgress);
        assert!(task.started_at.is_some());

        task.mark_completed(Some(PathBuf::from("/tmp/output.txt")));
        assert_eq!(task.status, TaskStatus::Completed);
        assert!(task.completed_at.is_some());
        assert!(task.output_file.is_some());
    }

    #[test]
    fn test_task_manager() {
        let temp_dir = TempDir::new().unwrap();
        let manager = TaskManager::new(temp_dir.path().to_path_buf()).unwrap();

        // 创建任务
        let task_id = manager
            .create_task(
                "测试任务".to_string(),
                "测试描述".to_string(),
                "测试提示词".to_string(),
                AgentType::Explore,
            )
            .unwrap();

        // 获取任务
        let task = manager.get_task(&task_id).unwrap().unwrap();
        assert_eq!(task.name, "测试任务");

        // 列出任务
        let tasks = manager.list_tasks().unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].id, task_id);
    }

    #[test]
    fn test_task_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().to_path_buf();

        // 创建第一个管理器并添加任务
        let manager1 = TaskManager::new(storage_path.clone()).unwrap();
        let task_id = manager1
            .create_task(
                "持久化测试".to_string(),
                "测试描述".to_string(),
                "测试提示词".to_string(),
                AgentType::Plan,
            )
            .unwrap();

        // 创建第二个管理器（模拟重启）
        let manager2 = TaskManager::new(storage_path).unwrap();
        let task = manager2.get_task(&task_id).unwrap().unwrap();

        assert_eq!(task.name, "持久化测试");
        assert_eq!(task.agent_type, AgentType::Plan);
    }
}
