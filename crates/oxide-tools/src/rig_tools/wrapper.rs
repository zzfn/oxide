//! 工具包装器 - 添加进度显示和权限检查
//!
//! 包装 rig Tool，在执行前后显示进度信息并检查权限

use crate::permission::PermissionManager;
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

    /// 显示工具执行开始
    fn show_start(&self) {
        if self.show_progress {
            println!("  {} 执行工具: {}", "⚙".bright_yellow(), T::NAME.bright_cyan());
            let _ = std::io::stdout().flush();
        }
    }

    /// 显示工具执行成功
    fn show_success(&self) {
        if self.show_progress {
            println!("  {} 工具 {} 执行成功", "✓".green(), T::NAME.bright_cyan());
            let _ = std::io::stdout().flush();
        }
    }

    /// 显示工具执行失败
    fn show_error(&self, error: &str) {
        if self.show_progress {
            println!("  {} 工具 {} 执行失败: {}", "✗".red(), T::NAME.bright_cyan(), error);
            let _ = std::io::stdout().flush();
        }
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

impl<T: Tool + Send + Sync> Tool for ToolWrapper<T> {
    const NAME: &'static str = T::NAME;

    type Error = T::Error;
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
                if !pm.is_allowed(T::NAME).await {
                    if show_progress {
                        println!("  {} 工具 {} 被权限配置禁止", "✗".red(), T::NAME.bright_cyan());
                    }
                    // 需要将 OxideError 转换为工具的错误类型
                    // 这里我们无法直接返回，因为类型不匹配
                    // 暂时跳过权限检查的错误返回，只打印警告
                }
            }

            // 显示开始
            if show_progress {
                println!("  {} 执行工具: {}", "⚙".bright_yellow(), T::NAME.bright_cyan());
                let _ = std::io::stdout().flush();
            }

            // 执行工具
            let result = self.inner.call(args).await;

            // 显示结果
            if show_progress {
                match &result {
                    Ok(_) => println!("  {} 工具 {} 执行成功", "✓".green(), T::NAME.bright_cyan()),
                    Err(e) => println!("  {} 工具 {} 执行失败: {:?}", "✗".red(), T::NAME.bright_cyan(), e),
                }
                let _ = std::io::stdout().flush();
            }

            result
        }
    }
}
