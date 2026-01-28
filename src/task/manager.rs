//! 任务管理器
//!
//! 管理后台任务的创建、执行和追踪。

use crate::agent::types::AgentType;
use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::task::JoinHandle;
use uuid::Uuid;

/// 任务 ID 类型
pub type TaskId = String;

/// 任务状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    /// 等待中
    Pending,

    /// 进行中
    InProgress,

    /// 已完成
    Completed,

    /// 失败
    Failed,

    /// 已删除
    Deleted,
}

/// 任务信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// 任务 ID
    pub id: TaskId,

    /// 任务名称（内部名称）
    pub name: String,

    /// 简短标题（必需，用于显示）
    pub subject: String,

    /// 任务描述
    pub description: String,

    /// 提示词
    pub prompt: String,

    /// 进行中显示文本（如 "Running tests"）
    pub active_form: Option<String>,

    /// 任务状态
    pub status: TaskStatus,

    /// Agent 类型
    pub agent_type: AgentType,

    /// 任务所有者
    pub owner: Option<String>,

    /// 阻塞的任务 ID（本任务完成后这些任务才能开始）
    pub blocks: Vec<TaskId>,

    /// 被阻塞的任务 ID（这些任务完成后本任务才能开始）
    pub blocked_by: Vec<TaskId>,

    /// 自定义元数据
    pub metadata: HashMap<String, serde_json::Value>,

    /// 创建时间
    pub created_at: DateTime<Utc>,

    /// 最后更新时间
    pub updated_at: DateTime<Utc>,

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
    /// 创建新任务（简化版，用于任务管理工具）
    pub fn new(subject: String, description: String, active_form: Option<String>) -> Self {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        Self {
            id,
            name: subject.clone(),
            subject,
            description,
            prompt: String::new(),
            active_form,
            status: TaskStatus::Pending,
            agent_type: AgentType::Main,
            owner: None,
            blocks: Vec::new(),
            blocked_by: Vec::new(),
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
            started_at: None,
            completed_at: None,
            output_file: None,
            error: None,
        }
    }

    /// 创建新任务（完整版，用于后台 Agent 任务）
    #[allow(dead_code)]
    pub fn new_with_agent(
        name: String,
        description: String,
        prompt: String,
        agent_type: AgentType,
    ) -> Self {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        Self {
            id,
            subject: name.clone(),
            name,
            description,
            prompt,
            active_form: None,
            status: TaskStatus::Pending,
            agent_type,
            owner: None,
            blocks: Vec::new(),
            blocked_by: Vec::new(),
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
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
        self.updated_at = Utc::now();
    }

    /// 标记任务为已完成
    #[allow(dead_code)]
    pub fn mark_completed(&mut self, output_file: Option<PathBuf>) {
        self.status = TaskStatus::Completed;
        self.completed_at = Some(Utc::now());
        self.updated_at = Utc::now();
        self.output_file = output_file;
    }

    /// 标记任务为失败
    #[allow(dead_code)]
    pub fn mark_failed(&mut self, error: String) {
        self.status = TaskStatus::Failed;
        self.completed_at = Some(Utc::now());
        self.updated_at = Utc::now();
        self.error = Some(error);
    }

    /// 标记任务为已删除
    #[allow(dead_code)]
    pub fn mark_deleted(&mut self) {
        self.status = TaskStatus::Deleted;
        self.updated_at = Utc::now();
    }

    /// 检查任务是否被阻塞（所有 blocked_by 任务都必须已完成）
    pub fn is_blocked(&self, tasks: &HashMap<TaskId, Task>) -> bool {
        for blocking_id in &self.blocked_by {
            if let Some(blocking_task) = tasks.get(blocking_id) {
                // 如果阻塞任务未完成且未删除，则本任务被阻塞
                if blocking_task.status != TaskStatus::Completed
                    && blocking_task.status != TaskStatus::Deleted
                {
                    return true;
                }
            }
        }
        false
    }

    /// 获取未完成的阻塞任务 ID 列表
    pub fn get_open_blockers(&self, tasks: &HashMap<TaskId, Task>) -> Vec<TaskId> {
        self.blocked_by
            .iter()
            .filter(|id| {
                tasks.get(*id).map_or(false, |t| {
                    t.status != TaskStatus::Completed && t.status != TaskStatus::Deleted
                })
            })
            .cloned()
            .collect()
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

/// 全局任务管理器单例
static TASK_MANAGER: Lazy<TaskManager> = Lazy::new(|| {
    let storage_dir = PathBuf::from(".oxide/tasks");
    TaskManager::new(storage_dir).expect("无法初始化任务管理器")
});

/// 获取全局任务管理器
pub fn get_task_manager() -> &'static TaskManager {
    &TASK_MANAGER
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

    /// 创建新任务（简化版，用于任务管理工具）
    pub fn create_task_simple(
        &self,
        subject: String,
        description: String,
        active_form: Option<String>,
        metadata: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<Task> {
        let mut task = Task::new(subject, description, active_form);
        if let Some(meta) = metadata {
            task.metadata = meta;
        }
        let task_id = task.id.clone();

        // 保存任务到磁盘
        self.save_task(&task)?;

        // 添加到内存存储
        let mut tasks = self.tasks.lock().unwrap();
        tasks.insert(task_id, task.clone());

        Ok(task)
    }

    /// 创建新任务（完整版，用于后台 Agent 任务）
    #[allow(dead_code)]
    pub fn create_task(
        &self,
        name: String,
        description: String,
        prompt: String,
        agent_type: AgentType,
    ) -> Result<TaskId> {
        let task = Task::new_with_agent(name, description, prompt, agent_type);
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
            task.updated_at = Utc::now();

            // 根据状态更新时间戳
            match status {
                TaskStatus::InProgress => {
                    if task.started_at.is_none() {
                        task.started_at = Some(Utc::now());
                    }
                }
                TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Deleted => {
                    if task.completed_at.is_none() {
                        task.completed_at = Some(Utc::now());
                    }
                }
                _ => {}
            }

            self.save_task(task)?;
        }
        Ok(())
    }

    /// 更新任务（通用方法）
    pub fn update_task<F>(&self, task_id: &TaskId, updater: F) -> Result<Option<Task>>
    where
        F: FnOnce(&mut Task),
    {
        // 先尝试从磁盘加载（如果内存中没有）
        let _ = self.get_task(task_id)?;

        let mut tasks = self.tasks.lock().unwrap();
        if let Some(task) = tasks.get_mut(task_id) {
            updater(task);
            task.updated_at = Utc::now();
            self.save_task(task)?;
            Ok(Some(task.clone()))
        } else {
            Ok(None)
        }
    }

    /// 添加依赖关系：task_id 阻塞 blocked_task_id
    /// 即 blocked_task_id 必须等待 task_id 完成
    pub fn add_blocks(&self, task_id: &TaskId, blocked_task_id: &TaskId) -> Result<()> {
        // 检查循环依赖
        if self.would_create_cycle(blocked_task_id, task_id)? {
            return Err(anyhow!(
                "添加依赖关系会导致循环依赖: {} -> {}",
                task_id,
                blocked_task_id
            ));
        }

        // 更新 task_id 的 blocks 列表
        self.update_task(task_id, |task| {
            if !task.blocks.contains(blocked_task_id) {
                task.blocks.push(blocked_task_id.clone());
            }
        })?;

        // 更新 blocked_task_id 的 blocked_by 列表
        self.update_task(blocked_task_id, |task| {
            if !task.blocked_by.contains(task_id) {
                task.blocked_by.push(task_id.clone());
            }
        })?;

        Ok(())
    }

    /// 添加依赖关系：task_id 被 blocking_task_id 阻塞
    /// 即 task_id 必须等待 blocking_task_id 完成
    pub fn add_blocked_by(&self, task_id: &TaskId, blocking_task_id: &TaskId) -> Result<()> {
        // 这实际上是 add_blocks 的反向操作
        self.add_blocks(blocking_task_id, task_id)
    }

    /// 检查添加依赖是否会导致循环
    fn would_create_cycle(&self, from_id: &TaskId, to_id: &TaskId) -> Result<bool> {
        // 如果 from_id 已经（直接或间接）依赖于 to_id，
        // 那么添加 to_id -> from_id 的依赖会导致循环
        let mut visited = HashSet::new();
        let mut path = HashSet::new();
        self.detect_cycle_from(from_id, to_id, &mut visited, &mut path)
    }

    /// 从指定任务开始检测是否能到达目标任务（DFS）
    fn detect_cycle_from(
        &self,
        current_id: &TaskId,
        target_id: &TaskId,
        visited: &mut HashSet<TaskId>,
        path: &mut HashSet<TaskId>,
    ) -> Result<bool> {
        if current_id == target_id {
            return Ok(true);
        }

        if visited.contains(current_id) {
            return Ok(false);
        }

        visited.insert(current_id.clone());
        path.insert(current_id.clone());

        if let Some(task) = self.get_task(current_id)? {
            for blocked_id in &task.blocks {
                if self.detect_cycle_from(blocked_id, target_id, visited, path)? {
                    return Ok(true);
                }
            }
        }

        path.remove(current_id);
        Ok(false)
    }

    /// 获取未被阻塞的待处理任务
    pub fn get_available_tasks(&self) -> Result<Vec<Task>> {
        let all_tasks = self.list_tasks()?;
        let tasks_map: HashMap<TaskId, Task> = all_tasks
            .iter()
            .cloned()
            .map(|t| (t.id.clone(), t))
            .collect();

        Ok(all_tasks
            .into_iter()
            .filter(|task| {
                task.status == TaskStatus::Pending
                    && task.owner.is_none()
                    && !task.is_blocked(&tasks_map)
            })
            .collect())
    }

    /// 删除任务（标记为已删除）
    pub fn delete_task(&self, task_id: &TaskId) -> Result<bool> {
        if let Some(_) = self.get_task(task_id)? {
            self.update_task_status(task_id, TaskStatus::Deleted)?;
            Ok(true)
        } else {
            Ok(false)
        }
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
            Some("Testing".to_string()),
        );

        assert_eq!(task.status, TaskStatus::Pending);
        assert_eq!(task.subject, "测试任务");
        assert!(task.started_at.is_none());
        assert!(task.completed_at.is_none());
    }

    #[test]
    fn test_task_with_agent() {
        let task = Task::new_with_agent(
            "Agent 任务".to_string(),
            "测试描述".to_string(),
            "测试提示词".to_string(),
            AgentType::Explore,
        );

        assert_eq!(task.status, TaskStatus::Pending);
        assert_eq!(task.agent_type, AgentType::Explore);
        assert_eq!(task.prompt, "测试提示词");
    }

    #[test]
    fn test_task_status_transitions() {
        let mut task = Task::new(
            "测试任务".to_string(),
            "测试描述".to_string(),
            None,
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
    fn test_task_deleted_status() {
        let mut task = Task::new(
            "测试任务".to_string(),
            "测试描述".to_string(),
            None,
        );

        task.mark_deleted();
        assert_eq!(task.status, TaskStatus::Deleted);
    }

    #[test]
    fn test_task_manager_simple() {
        let temp_dir = TempDir::new().unwrap();
        let manager = TaskManager::new(temp_dir.path().to_path_buf()).unwrap();

        // 创建简化任务
        let task = manager
            .create_task_simple(
                "测试任务".to_string(),
                "测试描述".to_string(),
                Some("Testing".to_string()),
                None,
            )
            .unwrap();

        assert_eq!(task.subject, "测试任务");

        // 获取任务
        let fetched = manager.get_task(&task.id).unwrap().unwrap();
        assert_eq!(fetched.subject, "测试任务");

        // 列出任务
        let tasks = manager.list_tasks().unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].id, task.id);
    }

    #[test]
    fn test_task_manager_with_agent() {
        let temp_dir = TempDir::new().unwrap();
        let manager = TaskManager::new(temp_dir.path().to_path_buf()).unwrap();

        // 创建 Agent 任务
        let task_id = manager
            .create_task(
                "Agent 任务".to_string(),
                "测试描述".to_string(),
                "测试提示词".to_string(),
                AgentType::Explore,
            )
            .unwrap();

        // 获取任务
        let task = manager.get_task(&task_id).unwrap().unwrap();
        assert_eq!(task.name, "Agent 任务");
        assert_eq!(task.agent_type, AgentType::Explore);
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

    #[test]
    fn test_task_dependencies() {
        let temp_dir = TempDir::new().unwrap();
        let manager = TaskManager::new(temp_dir.path().to_path_buf()).unwrap();

        // 创建两个任务
        let task1 = manager
            .create_task_simple("任务1".to_string(), "描述1".to_string(), None, None)
            .unwrap();
        let task2 = manager
            .create_task_simple("任务2".to_string(), "描述2".to_string(), None, None)
            .unwrap();

        // 添加依赖：task2 被 task1 阻塞
        manager.add_blocked_by(&task2.id, &task1.id).unwrap();

        // 验证依赖关系
        let updated_task1 = manager.get_task(&task1.id).unwrap().unwrap();
        let updated_task2 = manager.get_task(&task2.id).unwrap().unwrap();

        assert!(updated_task1.blocks.contains(&task2.id));
        assert!(updated_task2.blocked_by.contains(&task1.id));

        // 验证 task2 被阻塞
        let all_tasks = manager.list_tasks().unwrap();
        let tasks_map: HashMap<TaskId, Task> = all_tasks
            .iter()
            .cloned()
            .map(|t| (t.id.clone(), t))
            .collect();
        assert!(updated_task2.is_blocked(&tasks_map));
    }

    #[test]
    fn test_cycle_detection() {
        let temp_dir = TempDir::new().unwrap();
        let manager = TaskManager::new(temp_dir.path().to_path_buf()).unwrap();

        // 创建三个任务
        let task1 = manager
            .create_task_simple("任务1".to_string(), "描述1".to_string(), None, None)
            .unwrap();
        let task2 = manager
            .create_task_simple("任务2".to_string(), "描述2".to_string(), None, None)
            .unwrap();
        let task3 = manager
            .create_task_simple("任务3".to_string(), "描述3".to_string(), None, None)
            .unwrap();

        // 添加依赖链：task1 -> task2 -> task3
        manager.add_blocks(&task1.id, &task2.id).unwrap();
        manager.add_blocks(&task2.id, &task3.id).unwrap();

        // 尝试添加循环依赖：task3 -> task1（应该失败）
        let result = manager.add_blocks(&task3.id, &task1.id);
        assert!(result.is_err());
    }

    #[test]
    fn test_available_tasks() {
        let temp_dir = TempDir::new().unwrap();
        let manager = TaskManager::new(temp_dir.path().to_path_buf()).unwrap();

        // 创建两个任务
        let task1 = manager
            .create_task_simple("任务1".to_string(), "描述1".to_string(), None, None)
            .unwrap();
        let task2 = manager
            .create_task_simple("任务2".to_string(), "描述2".to_string(), None, None)
            .unwrap();

        // task2 被 task1 阻塞
        manager.add_blocked_by(&task2.id, &task1.id).unwrap();

        // 获取可用任务（只有 task1）
        let available = manager.get_available_tasks().unwrap();
        assert_eq!(available.len(), 1);
        assert_eq!(available[0].id, task1.id);

        // 完成 task1
        manager
            .update_task_status(&task1.id, TaskStatus::Completed)
            .unwrap();

        // 现在 task2 也可用了
        let available = manager.get_available_tasks().unwrap();
        assert_eq!(available.len(), 1);
        assert_eq!(available[0].id, task2.id);
    }
}
