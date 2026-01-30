//! 工具权限管理

use oxide_core::config::PermissionsConfig;
use std::collections::HashSet;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 需要用户确认的危险工具
const DANGEROUS_TOOLS: &[&str] = &["Edit", "Write", "Bash"];

/// 用户确认结果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmationResult {
    /// 允许本次执行
    Allow,
    /// 拒绝执行
    Deny,
    /// 始终允许（本次会话）
    AllowSession,
    /// 始终允许（持久化）
    AllowAlways,
}

/// 确认回调类型
pub type ConfirmationCallback = Arc<
    dyn Fn(String) -> Pin<Box<dyn Future<Output = ConfirmationResult> + Send>> + Send + Sync,
>;

/// 权限管理器
#[derive(Clone)]
pub struct PermissionManager {
    config: Arc<RwLock<PermissionsConfig>>,
    /// 本次会话中用户已批准的工具
    approved_tools: Arc<RwLock<HashSet<String>>>,
    /// 是否启用运行时确认
    require_confirmation: bool,
    /// 确认回调
    confirmation_callback: Option<ConfirmationCallback>,
}

impl PermissionManager {
    pub fn new(config: PermissionsConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            approved_tools: Arc::new(RwLock::new(HashSet::new())),
            require_confirmation: true,
            confirmation_callback: None,
        }
    }

    /// 禁用运行时确认（用于测试或自动化场景）
    pub fn without_confirmation(mut self) -> Self {
        self.require_confirmation = false;
        self
    }

    /// 设置确认回调
    pub fn with_confirmation_callback(mut self, callback: ConfirmationCallback) -> Self {
        self.confirmation_callback = Some(callback);
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

    /// 请求用户确认
    /// 返回 Ok(ConfirmationResult) 表示用户的选择
    /// 返回 Err 表示没有配置确认回调
    pub async fn request_confirmation(&self, tool_name: &str) -> Result<ConfirmationResult, ()> {
        if let Some(callback) = &self.confirmation_callback {
            let result = callback(tool_name.to_string()).await;
            match result {
                ConfirmationResult::Allow => {
                    // 仅本次允许，不记录
                }
                ConfirmationResult::AllowSession => {
                    // 本次会话内始终允许
                    self.approve_tool(tool_name).await;
                }
                ConfirmationResult::AllowAlways => {
                    // 持久化允许（TODO: 写入配置文件）
                    self.approve_tool(tool_name).await;
                }
                ConfirmationResult::Deny => {
                    // 拒绝，不做任何操作
                }
            }
            Ok(result)
        } else {
            Err(())
        }
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

    /// 检查是否有确认回调
    pub fn has_confirmation_callback(&self) -> bool {
        self.confirmation_callback.is_some()
    }
}

impl Default for PermissionManager {
    fn default() -> Self {
        Self::new(PermissionsConfig::default())
    }
}
