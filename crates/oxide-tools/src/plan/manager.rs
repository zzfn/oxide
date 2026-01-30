//! 计划管理器

use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// 计划状态
#[derive(Debug, Clone)]
pub struct PlanState {
    /// 当前计划 ID
    pub current_plan_id: Option<Uuid>,
    /// 计划标题
    pub plan_title: Option<String>,
    /// 是否处于计划模式
    pub is_plan_mode: bool,
}

impl Default for PlanState {
    fn default() -> Self {
        Self {
            current_plan_id: None,
            plan_title: None,
            is_plan_mode: false,
        }
    }
}

/// 计划管理器
#[derive(Clone)]
pub struct PlanManager {
    state: Arc<RwLock<PlanState>>,
}

impl PlanManager {
    /// 创建新的计划管理器
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(PlanState::default())),
        }
    }

    /// 进入计划模式
    pub async fn enter_plan_mode(&self, title: Option<String>) -> Uuid {
        let mut state = self.state.write().await;
        let plan_id = Uuid::new_v4();
        state.current_plan_id = Some(plan_id);
        state.plan_title = title;
        state.is_plan_mode = true;
        plan_id
    }

    /// 退出计划模式
    pub async fn exit_plan_mode(&self) -> Option<Uuid> {
        let mut state = self.state.write().await;
        let plan_id = state.current_plan_id;
        state.current_plan_id = None;
        state.plan_title = None;
        state.is_plan_mode = false;
        plan_id
    }

    /// 检查是否处于计划模式
    pub async fn is_plan_mode(&self) -> bool {
        let state = self.state.read().await;
        state.is_plan_mode
    }

    /// 获取当前计划 ID
    pub async fn current_plan_id(&self) -> Option<Uuid> {
        let state = self.state.read().await;
        state.current_plan_id
    }
}

impl Default for PlanManager {
    fn default() -> Self {
        Self::new()
    }
}
