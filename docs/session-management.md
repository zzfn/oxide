# 会话管理实现详解

## 目录

- [系统概述](#系统概述)
- [数据结构](#数据结构)
- [会话 ID](#会话-id)
- [消息管理](#消息管理)
- [持久化存储](#持久化存储)
- [会话操作](#会话操作)
- [自动清理](#自动清理)
- [CLI 集成](#cli-集成)
- [使用指南](#使用指南)
- [最佳实践](#最佳实践)

## 系统概述

Oxide 的会话管理系统负责管理用户与 AI Agent 的对话历史，提供持久化存储、多会话管理和自动清理功能。系统设计简洁高效，支持长时间的多轮对话。

### 核心特性

- **自动保存**: 每次交互后自动保存会话状态
- **多会话管理**: 支持创建、切换、删除多个会话
- **消息持久化**: JSON 格式存储，易于查看和迁移
- **自动清理**: 超过限制时自动移除旧消息
- **快速恢复**: 随时加载历史会话继续对话
- **安全性**: 防止删除活跃会话

## 数据结构

### ContextManager

会话管理的核心组件：

```rust
use std::path::PathBuf;
use std::sync::RwLock;

pub struct ContextManager {
    /// 存储目录路径
    storage_dir: PathBuf,

    /// 当前会话 ID
    session_id: String,

    /// 消息历史
    messages: Vec<Message>,

    /// 最大消息数限制
    max_messages: usize,
}
```

### Session

会话数据结构：

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// 会话元数据
    pub metadata: SessionMetadata,

    /// 消息历史
    pub messages: Vec<SerializableMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    /// 会话唯一标识
    pub session_id: String,

    /// 创建时间（RFC3339 格式）
    pub created_at: String,

    /// 最后更新时间（RFC3339 格式）
    pub last_updated: String,

    /// 消息数量
    pub message_count: usize,
}
```

### Message

消息类型定义：

```rust
use rig::messages::{Message, Role};

/// 可序列化的消息格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableMessage {
    /// 消息角色（user/assistant/system）
    pub role: String,

    /// 消息内容
    pub content: String,
}

impl From<Message> for SerializableMessage {
    fn from(msg: Message) -> Self {
        SerializableMessage {
            role: match msg.role {
                Role::User => "user".to_string(),
                Role::Assistant => "assistant".to_string(),
                Role::System => "system".to_string(),
            },
            content: msg.content,
        }
    }
}

impl From<SerializableMessage> for Message {
    fn from(msg: SerializableMessage) -> Self {
        let role = match msg.role.as_str() {
            "user" => Role::User,
            "assistant" => Role::Assistant,
            "system" => Role::System,
            _ => Role::User,
        };

        Message { role, content: msg.content }
    }
}
```

## 会话 ID

### 生成机制

使用 `names` crate 生成随机的、易读的会话 ID：

```rust
use names::{Generator, Name};

pub fn generate_session_id() -> String {
    let mut generator = Generator::default();
    generator
        .next()
        .unwrap_or_else(|| "unknown-session".to_string())
}
```

**生成的 ID 示例**:
- `whole-comfort`
- `violet-sky`
- `happy-river`
- `brave-mountain`

### ID 特点

- **唯一性**: 基于 `rand` 随机数生成器
- **可读性**: 使用形容词-名词组合
- **无冲突**: 生成器保证不会重复
- **易记忆**: 比随机字符串更友好

## 消息管理

### 添加消息

```rust
impl ContextManager {
    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);

        // 自动清理：超过最大数量时移除最旧的消息
        if self.messages.len() > self.max_messages {
            self.messages.remove(0);
        }
    }
}
```

### 获取消息

```rust
impl ContextManager {
    pub fn get_messages(&self) -> &[Message] {
        &self.messages
    }

    pub fn get_messages_count(&self) -> usize {
        self.messages.len()
    }

    pub fn clear_messages(&mut self) {
        self.messages.clear();
    }
}
```

### 消息过滤

```rust
impl ContextManager {
    /// 获取最近 N 条消息
    pub fn get_recent_messages(&self, n: usize) -> Vec<Message> {
        let start = if self.messages.len() > n {
            self.messages.len() - n
        } else {
            0
        };

        self.messages[start..].to_vec()
    }

    /// 获取特定角色的消息
    pub fn get_messages_by_role(&self, role: Role) -> Vec<Message> {
        self.messages
            .iter()
            .filter(|m| m.role == role)
            .cloned()
            .collect()
    }
}
```

## 持久化存储

### 存储位置

```
.oxide/
└── sessions/
    ├── whole-comfort.json
    ├── violet-sky.json
    └── happy-river.json
```

### 存储格式

JSON 格式，易于人类阅读和机器解析：

```json
{
  "metadata": {
    "session_id": "whole-comfort",
    "created_at": "2026-01-24T05:12:11.710311+00:00",
    "last_updated": "2026-01-24T05:15:32.123456+00:00",
    "message_count": 4
  },
  "messages": [
    {
      "role": "user",
      "content": "hello"
    },
    {
      "role": "assistant",
      "content": "Hello! 👋 How can I help you today?"
    },
    {
      "role": "user",
      "content": "帮我查看当前目录的文件"
    },
    {
      "role": "assistant",
      "content": "[工具] scan_codebase\n..."
    }
  ]
}
```

### 保存会话

```rust
use std::fs::File;
use std::io::Write;

impl ContextManager {
    pub fn save(&self) -> Result<()> {
        // 确保存储目录存在
        fs::create_dir_all(&self.storage_dir)?;

        // 创建文件路径
        let file_path = self.storage_dir.join(format!("{}.json", self.session_id));

        // 序列化会话
        let session = Session {
            metadata: SessionMetadata {
                session_id: self.session_id.clone(),
                created_at: self.get_created_time(),
                last_updated: Utc::now().to_rfc3339(),
                message_count: self.messages.len(),
            },
            messages: self.messages
                .iter()
                .map(|m| SerializableMessage::from(m.clone()))
                .collect(),
        };

        // 写入文件
        let json = serde_json::to_string_pretty(&session)?;
        let mut file = File::create(file_path)?;
        file.write_all(json.as_bytes())?;

        Ok(())
    }
}
```

### 加载会话

```rust
impl ContextManager {
    pub fn load(&mut self, session_id: &str) -> Result<()> {
        // 构建文件路径
        let file_path = self.storage_dir.join(format!("{}.json", session_id));

        // 检查文件是否存在
        if !file_path.exists() {
            bail!("Session not found: {}", session_id);
        }

        // 读取文件
        let content = fs::read_to_string(file_path)?;
        let session: Session = serde_json::from_str(&content)?;

        // 更新状态
        self.session_id = session.metadata.session_id;
        self.messages = session
            .messages
            .into_iter()
            .map(|m| Message::from(m))
            .collect();

        Ok(())
    }
}
```

## 会话操作

### 列出会话

```rust
impl ContextManager {
    pub fn list_sessions(&self) -> Result<Vec<SessionMetadata>> {
        let mut sessions = Vec::new();

        // 读取存储目录中的所有 JSON 文件
        for entry in fs::read_dir(&self.storage_dir)? {
            let entry = entry?;
            let path = entry.path();

            // 只处理 .json 文件
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                // 读取并解析文件
                let content = fs::read_to_string(&path)?;
                let session: Session = serde_json::from_str(&content)?;

                sessions.push(session.metadata);
            }
        }

        // 按最后更新时间排序（最新的在前）
        sessions.sort_by(|a, b| {
            b.last_updated
                .cmp(&a.last_updated)
        });

        Ok(sessions)
    }
}
```

### 删除会话

```rust
impl ContextManager {
    pub fn delete_session(&self, session_id: &str) -> Result<()> {
        // 防止删除当前活跃会话
        if session_id == self.session_id {
            bail!("Cannot delete active session");
        }

        // 构建文件路径
        let file_path = self.storage_dir.join(format!("{}.json", session_id));

        // 检查文件是否存在
        if !file_path.exists() {
            bail!("Session not found: {}", session_id);
        }

        // 删除文件
        fs::remove_file(file_path)?;

        Ok(())
    }
}
```

### 切换会话

```rust
impl ContextManager {
    pub fn switch_session(&mut self, new_session_id: String) -> Result<()> {
        // 保存当前会话
        self.save()?;

        // 如果会话不存在，创建新会话
        let file_path = self.storage_dir.join(format!("{}.json", new_session_id));
        if file_path.exists() {
            // 加载现有会话
            self.load(&new_session_id)?;
        } else {
            // 创建新会话
            self.session_id = new_session_id;
            self.messages.clear();
        }

        Ok(())
    }
}
```

## 自动清理

### 消息数量限制

```rust
impl ContextManager {
    pub fn new(storage_dir: PathBuf, max_messages: usize) -> Self {
        Self {
            storage_dir,
            session_id: generate_session_id(),
            messages: Vec::new(),
            max_messages,
        }
    }

    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);

        // 自动清理：超过最大数量时移除最旧的消息
        if self.messages.len() > self.max_messages {
            let removed = self.messages.remove(0);

            // 可选：记录被移除的消息
            if cfg!(debug_assertions) {
                eprintln!(
                    "Removed old message (role: {:?}) to stay within limit",
                    removed.role
                );
            }
        }
    }
}
```

**默认限制**: 100 条消息

### 配置限制

可以通过配置调整限制（未来功能）：

```toml
# .oxide/config.toml
[session]
max_messages = 200  # 增加到 200 条消息
auto_cleanup = true
```

## CLI 集成

### 自动保存

每次 AI 响应完成后自动保存：

```rust
impl OxideCli {
    async fn process_ai_response(&mut self) -> Result<()> {
        // 发送消息给 AI
        let response = self.agent.prompt(&user_input).await?;

        // 添加到上下文
        self.context_manager.add_message(Message::user(&user_input));
        self.context_manager.add_message(Message::assistant(&response));

        // 自动保存上下文
        if let Err(e) = self.context_manager.save() {
            println!(
                "{} Failed to save context: {}",
                "⚠️".yellow(),
                e
            );
        }

        Ok(())
    }
}
```

### 会话命令

```rust
impl OxideCli {
    async fn handle_session_command(&mut self, args: &str) -> Result<()> {
        let parts: Vec<&str> = args.splitn(2, ' ').collect();
        let command = parts[0];
        let arg = parts.get(1).unwrap_or(&"");

        match command {
            "list" => self.list_sessions(),
            "load" => self.load_session(arg),
            "delete" => self.delete_session(arg),
            _ => self.show_session_help(),
        }
    }

    fn list_sessions(&self) -> Result<()> {
        let sessions = self.context_manager.list_sessions()?;

        println!("\n📁 会话列表:\n");

        for session in sessions {
            // 标记当前会话
            let current_marker = if session.session_id == self.context_manager.session_id() {
                " (当前)"
            } else {
                ""
            };

            println!(
                "  {}{} - {} 条消息",
                session.session_id.bold(),
                current_marker,
                session.message_count
            );
            println!(
                "    创建: {}\n    更新: {}",
                Self::format_time(&session.created_at),
                Self::format_time(&session.last_updated)
            );
            println!();
        }

        Ok(())
    }

    fn format_time(rfc3339: &str) -> String {
        // 解析 RFC3339 时间并格式化为本地时间
        // 实现略...
    }
}
```

### 命令列表

| 命令 | 说明 | 示例 |
|-----|------|------|
| `/sessions` | 列出所有会话 | `/sessions` |
| `/load <id>` | 加载指定会话 | `/load whole-comfort` |
| `/delete <id>` | 删除会话 | `/delete violet-sky` |
| `/history` | 显示当前会话历史 | `/history` |
| `/clear` | 清除当前会话消息 | `/clear` |

## 使用指南

### 基本使用

```bash
# 启动 Oxide（自动创建新会话）
oxide

# 会话 ID 会显示在提示符中
==================================================
Oxide CLI 0.1.0 - DeepSeek Agent
==================================================
模型: deepseek-chat
会话: whole-comfort
提示: 输入 /help 查看帮助

你> 你好
...
```

### 查看会话列表

```bash
你> /sessions

📁 会话列表:

  whole-comfort (当前) - 8 条消息
    创建: 2026-01-24 13:12:11
    更新: 2026-01-24 13:25:33

  violet-sky - 15 条消息
    创建: 2026-01-23 10:05:42
    更新: 2026-01-23 10:30:18

  happy-river - 3 条消息
    创建: 2026-01-22 16:20:55
    更新: 2026-01-22 16:22:10
```

### 切换会话

```bash
# 加载之前的会话
你> /load violet-sky

✓ 已加载会话: violet-sky
会话包含 15 条消息

# 继续对话
你> 我们之前讨论了什么？
（AI 会根据历史消息回答）
```

### 删除会话

```bash
# 删除不需要的会话
你> /delete happy-river

✓ 已删除会话: happy-river

# 注意：不能删除当前活跃的会话
你> /delete whole-comfort
✗ 不能删除当前活跃会话
```

### 清空历史

```bash
# 清除当前会话的所有消息
你> /clear

✓ 已清除会话消息
（会话 ID 保持不变，但消息历史被清空）
```

### 查看消息历史

```bash
# 显示当前会话的消息历史
你> /history

[0] user: 你好
[1] assistant: 你好！我是 Oxide 助手...
[2] user: 帮我查看文件
[3] assistant: [工具] scan_codebase ...
```

## 最佳实践

### 会话组织

1. **按任务分类**: 不同任务使用不同会话
2. **定期清理**: 删除不需要的旧会话
3. **有意义的名称**: 会话 ID 自动生成，但可以在描述中记录主题

### 长时间对话

```bash
# 启动新会话进行长时间任务
oxide

# 记录会话 ID
会话: brave-mountain

# 工作一段时间...

# 退出
/exit

# 稍后恢复
oxide
/load brave-mountain
```

### Token 管理

```rust
// 监控消息数量，避免超出限制
if context_manager.get_messages_count() > 80 {
    println!("⚠️  会话消息接近限制，考虑使用 /compact 压缩");
}
```

### 数据备份

```bash
# 手动备份会话
cp -r .oxide/sessions ~/.oxide/backup/

# 或导出特定会话
cat .oxide/sessions/whole-comfort.json | jq .
```

## 相关文档

- [Agent 系统](./agent-system.md) - Agent 使用会话的方式
- [配置管理](./config-management.md) - 配置会话参数
- [整体架构](./architecture.md) - 项目架构总览
