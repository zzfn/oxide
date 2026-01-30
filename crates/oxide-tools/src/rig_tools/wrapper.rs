//! 工具包装器 - 添加进度显示
//!
//! 包装 rig Tool，在执行前后显示进度信息

use colored::Colorize;
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::io::Write;

/// 工具包装器 - 添加进度显示
pub struct ToolWrapper<T: Tool> {
    inner: T,
    show_progress: bool,
}

impl<T: Tool> ToolWrapper<T> {
    /// 创建新的工具包装器
    pub fn new(tool: T) -> Self {
        Self {
            inner: tool,
            show_progress: true,
        }
    }

    /// 设置是否显示进度
    pub fn with_progress(mut self, show: bool) -> Self {
        self.show_progress = show;
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
        async move {
            // 显示开始
            self.show_start();

            // 执行工具
            let result = self.inner.call(args).await;

            // 显示结果
            match &result {
                Ok(_) => self.show_success(),
                Err(e) => self.show_error(&format!("{:?}", e)),
            }

            result
        }
    }
}
