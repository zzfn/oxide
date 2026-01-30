//! 工具包装器 - 添加进度显示和权限检查
//!
//! 包装 rig Tool，在执行前后显示进度信息并检查权限

use crate::permission::{ConfirmationResult, PermissionManager};
use crate::rig_tools::errors::{PermissionError, WrappedError};
use colored::Colorize;
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::io::Write;

/// 工具包装器 - 添加进度显示和权限检查
pub struct ToolWrapper<T: Tool> {
    inner: T,
    show_progress: bool,
    permission_manager: Option<PermissionManager>,
}

impl<T: Tool> ToolWrapper<T> {
    /// 创建新的工具包装器
    pub fn new(tool: T) -> Self {
        Self {
            inner: tool,
            show_progress: true,
            permission_manager: None,
        }
    }

    /// 设置是否显示进度
    pub fn with_progress(mut self, show: bool) -> Self {
        self.show_progress = show;
        self
    }

    /// 设置权限管理器
    pub fn with_permission_manager(mut self, manager: PermissionManager) -> Self {
        self.permission_manager = Some(manager);
        self
    }

}

// 实现 Clone（如果内部工具支持）
impl<T: Tool + Clone> Clone for ToolWrapper<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            show_progress: self.show_progress,
            permission_manager: self.permission_manager.clone(),
        }
    }
}

// 实现 Serialize（如果内部工具支持）
impl<T: Tool + Serialize> Serialize for ToolWrapper<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.inner.serialize(serializer)
    }
}

// 实现 Deserialize（如果内部工具支持）
impl<'de, T: Tool + Deserialize<'de>> Deserialize<'de> for ToolWrapper<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self {
            inner: T::deserialize(deserializer)?,
            show_progress: true,
            permission_manager: None,
        })
    }
}

impl<T: Tool + Send + Sync> Tool for ToolWrapper<T>
where
    T::Error: Send,
{
    const NAME: &'static str = T::NAME;

    type Error = WrappedError<T::Error>;
    type Args = T::Args;
    type Output = T::Output;

    fn definition(
        &self,
        prompt: String,
    ) -> impl Future<Output = ToolDefinition> + Send + Sync {
        self.inner.definition(prompt)
    }

    fn call(
        &self,
        args: Self::Args,
    ) -> impl Future<Output = Result<Self::Output, Self::Error>> + Send {
        let permission_manager = self.permission_manager.clone();
        let show_progress = self.show_progress;

        async move {
            // 权限检查
            if let Some(pm) = &permission_manager {
                // 1. 检查工具是否被配置禁止
                if !pm.is_allowed(T::NAME).await {
                    if show_progress {
                        println!(
                            "  {} 工具 {} 被权限配置禁止",
                            "🚫".red(),
                            T::NAME.bright_cyan()
                        );
                        let _ = std::io::stdout().flush();
                    }
                    return Err(WrappedError::Permission(PermissionError::ToolDenied(
                        T::NAME.to_string(),
                    )));
                }

                // 2. 检查是否需要用户确认
                if pm.requires_confirmation(T::NAME).await {
                    match pm.request_confirmation(T::NAME).await {
                        Ok(ConfirmationResult::Allow)
                        | Ok(ConfirmationResult::AllowSession)
                        | Ok(ConfirmationResult::AllowAlways) => {
                            // 用户同意，继续执行
                            if show_progress {
                                println!(
                                    "  {} 用户已授权执行工具 {}",
                                    "✓".green(),
                                    T::NAME.bright_cyan()
                                );
                                let _ = std::io::stdout().flush();
                            }
                        }
                        Ok(ConfirmationResult::Deny) => {
                            // 用户拒绝
                            if show_progress {
                                println!(
                                    "  {} 用户拒绝执行工具 {}",
                                    "🚫".red(),
                                    T::NAME.bright_cyan()
                                );
                                let _ = std::io::stdout().flush();
                            }
                            return Err(WrappedError::Permission(PermissionError::UserRejected(
                                T::NAME.to_string(),
                            )));
                        }
                        Err(()) => {
                            // 没有配置确认回调
                            if show_progress {
                                println!(
                                    "  {} 工具 {} 需要用户确认，但未配置确认处理器",
                                    "⚠".yellow(),
                                    T::NAME.bright_cyan()
                                );
                                let _ = std::io::stdout().flush();
                            }
                            return Err(WrappedError::Permission(
                                PermissionError::NoConfirmationHandler(T::NAME.to_string()),
                            ));
                        }
                    }
                }
            }

            // 显示开始
            if show_progress {
                println!(
                    "  {} 执行工具: {}",
                    "⚙".bright_yellow(),
                    T::NAME.bright_cyan()
                );
                let _ = std::io::stdout().flush();
            }

            // 执行工具
            let result = self.inner.call(args).await;

            // 显示结果
            if show_progress {
                match &result {
                    Ok(_) => println!(
                        "  {} 工具 {} 执行成功",
                        "✓".green(),
                        T::NAME.bright_cyan()
                    ),
                    Err(e) => println!(
                        "  {} 工具 {} 执行失败: {:?}",
                        "✗".red(),
                        T::NAME.bright_cyan(),
                        e
                    ),
                }
                let _ = std::io::stdout().flush();
            }

            result.map_err(WrappedError::Inner)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::permission::ConfirmationCallback;
    use oxide_core::config::PermissionsConfig;
    use rig::completion::ToolDefinition;
    use serde_json::json;
    use std::sync::Arc;

    /// 测试用的简单工具
    #[derive(Clone, Serialize, Deserialize)]
    struct MockEditTool;

    #[derive(Debug, thiserror::Error)]
    #[error("mock error")]
    struct MockError;

    impl Tool for MockEditTool {
        const NAME: &'static str = "Edit";
        type Error = MockError;
        type Args = serde_json::Value;
        type Output = String;

        async fn definition(&self, _prompt: String) -> ToolDefinition {
            ToolDefinition {
                name: "Edit".to_string(),
                description: "Mock edit tool".to_string(),
                parameters: json!({}),
            }
        }

        async fn call(&self, _args: Self::Args) -> Result<Self::Output, Self::Error> {
            Ok("executed".to_string())
        }
    }

    /// 测试用的非危险工具
    #[derive(Clone, Serialize, Deserialize)]
    struct MockReadTool;

    impl Tool for MockReadTool {
        const NAME: &'static str = "Read";
        type Error = MockError;
        type Args = serde_json::Value;
        type Output = String;

        async fn definition(&self, _prompt: String) -> ToolDefinition {
            ToolDefinition {
                name: "Read".to_string(),
                description: "Mock read tool".to_string(),
                parameters: json!({}),
            }
        }

        async fn call(&self, _args: Self::Args) -> Result<Self::Output, Self::Error> {
            Ok("read content".to_string())
        }
    }

    #[tokio::test]
    async fn test_wrapper_without_permission_manager() {
        // 没有权限管理器时，工具应该正常执行
        let tool = MockEditTool;
        let wrapper = ToolWrapper::new(tool).with_progress(false);

        let result = wrapper.call(json!({})).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "executed");
    }

    #[tokio::test]
    async fn test_wrapper_tool_denied_by_config() {
        // 工具被配置禁止时应该返回错误
        let mut config = PermissionsConfig::default();
        config.deny = vec!["Edit".to_string()];

        let pm = PermissionManager::new(config);
        let tool = MockEditTool;
        let wrapper = ToolWrapper::new(tool)
            .with_progress(false)
            .with_permission_manager(pm);

        let result = wrapper.call(json!({})).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            WrappedError::Permission(PermissionError::ToolDenied(name)) => {
                assert_eq!(name, "Edit");
            }
            _ => panic!("Expected ToolDenied error"),
        }
    }

    #[tokio::test]
    async fn test_wrapper_dangerous_tool_needs_confirmation() {
        // 危险工具需要确认，但没有配置回调时应该返回错误
        let config = PermissionsConfig::default();
        let pm = PermissionManager::new(config);

        let tool = MockEditTool;
        let wrapper = ToolWrapper::new(tool)
            .with_progress(false)
            .with_permission_manager(pm);

        let result = wrapper.call(json!({})).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            WrappedError::Permission(PermissionError::NoConfirmationHandler(name)) => {
                assert_eq!(name, "Edit");
            }
            _ => panic!("Expected NoConfirmationHandler error"),
        }
    }

    #[tokio::test]
    async fn test_wrapper_user_approves_dangerous_tool() {
        // 用户同意执行危险工具
        let config = PermissionsConfig::default();
        let callback: ConfirmationCallback = Arc::new(|_tool_name| {
            Box::pin(async move { ConfirmationResult::AllowSession })
        });
        let pm = PermissionManager::new(config).with_confirmation_callback(callback);

        let tool = MockEditTool;
        let wrapper = ToolWrapper::new(tool)
            .with_progress(false)
            .with_permission_manager(pm);

        let result = wrapper.call(json!({})).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "executed");
    }

    #[tokio::test]
    async fn test_wrapper_user_rejects_dangerous_tool() {
        // 用户拒绝执行危险工具
        let config = PermissionsConfig::default();
        let callback: ConfirmationCallback = Arc::new(|_tool_name| {
            Box::pin(async move { ConfirmationResult::Deny })
        });
        let pm = PermissionManager::new(config).with_confirmation_callback(callback);

        let tool = MockEditTool;
        let wrapper = ToolWrapper::new(tool)
            .with_progress(false)
            .with_permission_manager(pm);

        let result = wrapper.call(json!({})).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            WrappedError::Permission(PermissionError::UserRejected(name)) => {
                assert_eq!(name, "Edit");
            }
            _ => panic!("Expected UserRejected error"),
        }
    }

    #[tokio::test]
    async fn test_wrapper_non_dangerous_tool_no_confirmation() {
        // 非危险工具不需要确认
        let config = PermissionsConfig::default();
        let pm = PermissionManager::new(config);

        let tool = MockReadTool;
        let wrapper = ToolWrapper::new(tool)
            .with_progress(false)
            .with_permission_manager(pm);

        let result = wrapper.call(json!({})).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "read content");
    }

    #[tokio::test]
    async fn test_wrapper_confirmation_disabled() {
        // 禁用确认后，危险工具也不需要确认
        let config = PermissionsConfig::default();
        let pm = PermissionManager::new(config).without_confirmation();

        let tool = MockEditTool;
        let wrapper = ToolWrapper::new(tool)
            .with_progress(false)
            .with_permission_manager(pm);

        let result = wrapper.call(json!({})).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "executed");
    }

    #[tokio::test]
    async fn test_wrapper_approval_remembered_in_session() {
        // 用户批准后，同一会话内不再询问
        let config = PermissionsConfig::default();
        let call_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let call_count_clone = call_count.clone();

        let callback: ConfirmationCallback = Arc::new(move |_tool_name| {
            call_count_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Box::pin(async move { ConfirmationResult::AllowSession })
        });
        let pm = PermissionManager::new(config).with_confirmation_callback(callback);

        let tool = MockEditTool;
        let wrapper = ToolWrapper::new(tool)
            .with_progress(false)
            .with_permission_manager(pm);

        // 第一次调用
        let result1 = wrapper.call(json!({})).await;
        assert!(result1.is_ok());

        // 第二次调用
        let result2 = wrapper.call(json!({})).await;
        assert!(result2.is_ok());

        // 确认回调只应该被调用一次
        assert_eq!(call_count.load(std::sync::atomic::Ordering::SeqCst), 1);
    }
}
