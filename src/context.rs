use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub session_id: String,
    pub created_at: String,
    pub last_updated: String,
    pub message_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct SessionData {
    pub metadata: SessionMetadata,
    pub messages: Vec<SerializableMessage>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SerializableMessage {
    pub role: String,
    pub content: Option<String>,
    pub tool_call_id: Option<String>,
    pub tool_calls: Option<Vec<SerializableToolCall>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SerializableToolCall {
    pub id: String,
    pub call_type: String,
    pub function: SerializableFunctionCall,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SerializableFunctionCall {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: Option<String>,
    pub tool_call_id: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String,
    pub function: FunctionCall,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

impl Message {
    pub fn user(text: &str) -> Self {
        Message {
            role: "user".to_string(),
            content: Some(text.to_string()),
            tool_call_id: None,
            tool_calls: None,
        }
    }

    pub fn assistant_with_text(text: &str) -> Self {
        Message {
            role: "assistant".to_string(),
            content: Some(text.to_string()),
            tool_call_id: None,
            tool_calls: None,
        }
    }

    pub fn assistant_with_tool_calls(tool_calls: Vec<ToolCall>) -> Self {
        Message {
            role: "assistant".to_string(),
            content: None,
            tool_call_id: None,
            tool_calls: Some(tool_calls),
        }
    }

    pub fn tool_result(tool_use_id: &str, content: &str) -> Self {
        Message {
            role: "tool".to_string(),
            content: Some(content.to_string()),
            tool_call_id: Some(tool_use_id.to_string()),
            tool_calls: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ContextManager {
    storage_dir: PathBuf,
    session_id: String,
    messages: Vec<Message>,
    max_messages: usize,
}

impl ContextManager {
    pub fn new<P: AsRef<Path>>(storage_dir: P, session_id: String) -> Result<Self> {
        let storage_dir = storage_dir.as_ref().to_path_buf();
        if !storage_dir.exists() {
            fs::create_dir_all(&storage_dir)
                .with_context(|| format!("无法创建存储目录: {:?}", storage_dir))?;
        }
        Ok(Self {
            storage_dir,
            session_id,
            messages: Vec::new(),
            max_messages: 100,
        })
    }

    pub fn with_max_messages(mut self, max_messages: usize) -> Self {
        self.max_messages = max_messages;
        self
    }

    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
        if self.messages.len() > self.max_messages {
            self.messages.remove(0);
        }
    }

    pub fn get_messages(&self) -> &[Message] {
        &self.messages
    }

    pub fn get_messages_mut(&mut self) -> &mut Vec<Message> {
        &mut self.messages
    }

    pub fn clear(&mut self) {
        self.messages.clear();
    }

    pub fn save(&self) -> Result<()> {
        let file_path = self.get_session_file_path();
        let now = chrono::Utc::now().to_rfc3339();
        let metadata = SessionMetadata {
            session_id: self.session_id.clone(),
            created_at: now.clone(),
            last_updated: now,
            message_count: self.messages.len(),
        };
        let serializable_messages: Vec<SerializableMessage> = self
            .messages
            .iter()
            .map(|msg| SerializableMessage {
                role: msg.role.clone(),
                content: msg.content.clone(),
                tool_call_id: msg.tool_call_id.clone(),
                tool_calls: msg.tool_calls.clone().map(|calls| {
                    calls
                        .into_iter()
                        .map(|call| SerializableToolCall {
                            id: call.id,
                            call_type: call.call_type,
                            function: SerializableFunctionCall {
                                name: call.function.name,
                                arguments: call.function.arguments,
                            },
                        })
                        .collect()
                }),
            })
            .collect();
        let session_data = SessionData {
            metadata,
            messages: serializable_messages,
        };
        let json_data =
            serde_json::to_string_pretty(&session_data).context("序列化会话数据失败")?;
        fs::write(&file_path, json_data)
            .with_context(|| format!("无法写入会话文件: {:?}", file_path))?;
        Ok(())
    }

    pub fn load(&mut self) -> Result<bool> {
        let file_path = self.get_session_file_path();
        if !file_path.exists() {
            return Ok(false);
        }
        let json_data = fs::read_to_string(&file_path)
            .with_context(|| format!("无法读取会话文件: {:?}", file_path))?;
        let session_data: SessionData =
            serde_json::from_str(&json_data).context("反序列化会话数据失败")?;
        self.messages = session_data
            .messages
            .into_iter()
            .map(|msg| Message {
                role: msg.role,
                content: msg.content,
                tool_call_id: msg.tool_call_id,
                tool_calls: msg.tool_calls.map(|calls| {
                    calls
                        .into_iter()
                        .map(|call| ToolCall {
                            id: call.id,
                            call_type: call.call_type,
                            function: FunctionCall {
                                name: call.function.name,
                                arguments: call.function.arguments,
                            },
                        })
                        .collect()
                }),
            })
            .collect();
        Ok(true)
    }

    pub fn list_sessions(&self) -> Result<Vec<SessionMetadata>> {
        let mut sessions = Vec::new();
        if !self.storage_dir.exists() {
            return Ok(sessions);
        }
        for entry in fs::read_dir(&self.storage_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(json_data) = fs::read_to_string(&path) {
                    if let Ok(session_data) = serde_json::from_str::<SessionData>(&json_data) {
                        sessions.push(session_data.metadata);
                    }
                }
            }
        }
        sessions.sort_by(|a, b| b.last_updated.cmp(&a.last_updated));
        Ok(sessions)
    }

    pub fn delete_session(&self) -> Result<bool> {
        let file_path = self.get_session_file_path();
        if file_path.exists() {
            fs::remove_file(&file_path)
                .with_context(|| format!("无法删除会话文件: {:?}", file_path))?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn get_session_file_path(&self) -> PathBuf {
        self.storage_dir.join(format!("{}.json", self.session_id))
    }

    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    pub fn switch_session(&mut self, new_session_id: String) {
        self.session_id = new_session_id;
        self.messages.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_context_manager_new() {
        let temp_dir = TempDir::new().unwrap();
        let context_manager =
            ContextManager::new(temp_dir.path(), "test_session".to_string()).unwrap();
        assert_eq!(context_manager.session_id(), "test_session");
    }

    #[test]
    fn test_add_message() {
        let temp_dir = TempDir::new().unwrap();
        let mut context_manager =
            ContextManager::new(temp_dir.path(), "test_session".to_string()).unwrap();
        context_manager.add_message(Message::user("test message"));
        assert_eq!(context_manager.get_messages().len(), 1);
    }

    #[test]
    fn test_max_messages_limit() {
        let temp_dir = TempDir::new().unwrap();
        let mut context_manager = ContextManager::new(temp_dir.path(), "test_session".to_string())
            .unwrap()
            .with_max_messages(2);

        context_manager.add_message(Message::user("msg1"));
        context_manager.add_message(Message::user("msg2"));
        context_manager.add_message(Message::user("msg3"));

        assert_eq!(context_manager.get_messages().len(), 2);
        assert_eq!(
            context_manager.get_messages()[0].content,
            Some("msg2".to_string())
        );
    }

    #[test]
    fn test_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let mut context_manager =
            ContextManager::new(temp_dir.path(), "test_session".to_string()).unwrap();

        context_manager.add_message(Message::user("test message"));
        context_manager.save().unwrap();

        let mut new_context_manager =
            ContextManager::new(temp_dir.path(), "test_session".to_string()).unwrap();
        let loaded = new_context_manager.load().unwrap();
        assert!(loaded);
        assert_eq!(new_context_manager.get_messages().len(), 1);
    }

    #[test]
    fn test_clear() {
        let temp_dir = TempDir::new().unwrap();
        let mut context_manager =
            ContextManager::new(temp_dir.path(), "test_session".to_string()).unwrap();

        context_manager.add_message(Message::user("test message"));
        assert_eq!(context_manager.get_messages().len(), 1);

        context_manager.clear();
        assert_eq!(context_manager.get_messages().len(), 0);
    }

    #[test]
    fn test_switch_session() {
        let temp_dir = TempDir::new().unwrap();
        let mut context_manager =
            ContextManager::new(temp_dir.path(), "session1".to_string()).unwrap();

        context_manager.add_message(Message::user("msg1"));
        assert_eq!(context_manager.session_id(), "session1");

        context_manager.switch_session("session2".to_string());
        assert_eq!(context_manager.session_id(), "session2");
        assert_eq!(context_manager.get_messages().len(), 0);
    }
}
