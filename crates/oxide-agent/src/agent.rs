//! 主代理实现

use oxide_core::types::Conversation;
use oxide_provider::LLMProvider;
use oxide_tools::ToolRegistry;
use std::sync::Arc;
use uuid::Uuid;

/// 代理状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentState {
    Idle,
    Running,
    WaitingForUser,
    Completed,
}

/// 主代理
pub struct Agent {
    pub id: Uuid,
    pub state: AgentState,
    pub conversation: Conversation,
    #[allow(dead_code)] // TODO: 将在代理主循环实现时使用
    provider: Arc<dyn LLMProvider>,
    #[allow(dead_code)] // TODO: 将在代理主循环实现时使用
    tools: Arc<ToolRegistry>,
}

impl Agent {
    pub fn new(provider: Arc<dyn LLMProvider>, tools: Arc<ToolRegistry>) -> Self {
        Self {
            id: Uuid::new_v4(),
            state: AgentState::Idle,
            conversation: Conversation::new(),
            provider,
            tools,
        }
    }

    /// 运行代理主循环
    pub async fn run(&mut self) -> anyhow::Result<()> {
        self.state = AgentState::Running;
        // TODO: 实现代理主循环
        self.state = AgentState::Completed;
        Ok(())
    }
}
