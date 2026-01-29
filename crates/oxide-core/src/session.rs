//! 会话状态持久化
//!
//! 管理会话历史、任务状态和计划文件。

use crate::config::{history_path, plans_dir, session_env_dir, tasks_dir};
use crate::error::Result;
use crate::types::{Conversation, Message};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use uuid::Uuid;

/// 历史记录条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub input: String,
    pub session_id: Uuid,
}

impl HistoryEntry {
    pub fn new(input: String, session_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            input,
            session_id,
        }
    }
}

/// 历史记录管理器
pub struct History;

impl History {
    /// 追加历史记录
    pub fn append(entry: &HistoryEntry) -> Result<()> {
        let path = history_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;
        let line = serde_json::to_string(entry)?;
        writeln!(file, "{}", line)?;
        Ok(())
    }

    /// 读取最近的历史记录
    pub fn recent(limit: usize) -> Result<Vec<HistoryEntry>> {
        let path = history_path()?;
        if !path.exists() {
            return Ok(vec![]);
        }
        let file = File::open(&path)?;
        let reader = BufReader::new(file);
        let entries: Vec<HistoryEntry> = reader
            .lines()
            .filter_map(|line| line.ok())
            .filter_map(|line| serde_json::from_str(&line).ok())
            .collect();

        // 返回最后 limit 条
        let start = entries.len().saturating_sub(limit);
        Ok(entries[start..].to_vec())
    }

    /// 清空历史记录
    pub fn clear() -> Result<()> {
        let path = history_path()?;
        if path.exists() {
            fs::remove_file(&path)?;
        }
        Ok(())
    }
}

/// 会话状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub id: Uuid,
    pub conversation: Conversation,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub working_dir: PathBuf,
}

impl SessionState {
    pub fn new(working_dir: PathBuf) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            conversation: Conversation::new(),
            created_at: now,
            updated_at: now,
            working_dir,
        }
    }

    /// 获取会话文件路径
    fn session_path(&self) -> Result<PathBuf> {
        Ok(session_env_dir()?.join(format!("{}.json", self.id)))
    }

    /// 保存会话状态
    pub fn save(&self) -> Result<()> {
        let path = self.session_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(&path, content)?;
        Ok(())
    }

    /// 加载会话状态
    pub fn load(id: Uuid) -> Result<Self> {
        let path = session_env_dir()?.join(format!("{}.json", id));
        let content = fs::read_to_string(&path)?;
        let state: SessionState = serde_json::from_str(&content)?;
        Ok(state)
    }

    /// 列出所有会话
    pub fn list_all() -> Result<Vec<Uuid>> {
        let dir = session_env_dir()?;
        if !dir.exists() {
            return Ok(vec![]);
        }
        let mut sessions = vec![];
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Some(stem) = path.file_stem() {
                    if let Ok(id) = Uuid::parse_str(&stem.to_string_lossy()) {
                        sessions.push(id);
                    }
                }
            }
        }
        Ok(sessions)
    }

    /// 添加消息
    pub fn add_message(&mut self, message: Message) {
        self.conversation.add_message(message);
        self.updated_at = Utc::now();
    }
}

/// 任务状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

/// 持久化任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedTask {
    pub id: Uuid,
    pub subject: String,
    pub description: String,
    pub status: TaskStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub session_id: Uuid,
    pub blocked_by: Vec<Uuid>,
    pub blocks: Vec<Uuid>,
}

impl PersistedTask {
    pub fn new(subject: String, description: String, session_id: Uuid) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            subject,
            description,
            status: TaskStatus::Pending,
            created_at: now,
            updated_at: now,
            session_id,
            blocked_by: vec![],
            blocks: vec![],
        }
    }

    /// 获取任务文件路径
    fn task_path(&self) -> Result<PathBuf> {
        Ok(tasks_dir()?.join(format!("{}.json", self.id)))
    }

    /// 保存任务
    pub fn save(&self) -> Result<()> {
        let path = self.task_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(&path, content)?;
        Ok(())
    }

    /// 加载任务
    pub fn load(id: Uuid) -> Result<Self> {
        let path = tasks_dir()?.join(format!("{}.json", id));
        let content = fs::read_to_string(&path)?;
        let task: PersistedTask = serde_json::from_str(&content)?;
        Ok(task)
    }

    /// 删除任务
    pub fn delete(&self) -> Result<()> {
        let path = self.task_path()?;
        if path.exists() {
            fs::remove_file(&path)?;
        }
        Ok(())
    }
}

/// 计划文件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub session_id: Uuid,
}

impl Plan {
    pub fn new(title: String, content: String, session_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            title,
            content,
            created_at: Utc::now(),
            session_id,
        }
    }

    /// 保存计划
    pub fn save(&self) -> Result<()> {
        let path = plans_dir()?.join(format!("{}.json", self.id));
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(&path, content)?;
        Ok(())
    }

    /// 加载计划
    pub fn load(id: Uuid) -> Result<Self> {
        let path = plans_dir()?.join(format!("{}.json", id));
        let content = fs::read_to_string(&path)?;
        let plan: Plan = serde_json::from_str(&content)?;
        Ok(plan)
    }
}
