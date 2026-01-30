//! 工具权限管理

use oxide_core::config::PermissionsConfig;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 需要用户确认的危险工具
const DANGEROUS_TOOLS: &[&str] = &["Edit", "Write", "Bash"];

/// 权限管理器
#[derive(Clone)]
pub struct PermissionManager {
    config: Arc<RwLock<PermissionsConfig>>,
    /// 本次会话中用户已批准的工具
    approved_tools: Arc<RwLock<HashSet<String>>>,
    /// 是否启用运行时确认
    require_confirmation: bool,
}

impl PermissionManager {
    pub fn new(config: PermissionsConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            approved_tools: Arc::new(RwLock::new(HashSet::new())),
            require_confirmation: true,
        }
    }

    /// 禁用运行时确认（用于测试或自动化场景）
    pub fn without_confirmation(mut self) -> Self {
        self.require_confirmation = false;
        self
    }

    /// 检查工具是否被允许执行
    pub async fn is_allowed(&self, tool_name: &str) -> bool {
        let config = self.config.read().await;
        config.is_allowed(tool_name)
    }

    /// 检查工具是否需要用户确认
    pub async fn requires_confirmation(&self, tool_name: &str) -> bool {
        if !self.require_confirmation {
            return false;
        }

        // 检查是否是危险工具
        if !DANGEROUS_TOOLS.contains(&tool_name) {
            return false;
        }

        // 检查是否已经批准过
        let approved = self.approved_tools.read().await;
        !approved.contains(tool_name)
    }

    /// 批准工具执行（记录到本次会话）
    pub async fn approve_tool(&self, tool_name: &str) {
        let mut approved = self.approved_tools.write().await;
        approved.insert(tool_name.to_string());
    }

    /// 更新权限配置
    pub async fn update_config(&self, config: PermissionsConfig) {
        let mut current = self.config.write().await;
        *current = config;
    }
}

impl Default for PermissionManager {
    fn default() -> Self {
        Self::new(PermissionsConfig::default())
    }
}
