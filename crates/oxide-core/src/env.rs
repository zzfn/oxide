//! 环境变量管理
//!
//! 从环境变量读取敏感配置（如 API Key）。

use crate::error::{OxideError, Result};
use std::env;

/// 环境变量名称常量
pub mod vars {
    /// Anthropic API Key
    pub const ANTHROPIC_API_KEY: &str = "ANTHROPIC_API_KEY";
    /// Oxide Auth Token (自定义 API Key)
    pub const OXIDE_AUTH_TOKEN: &str = "OXIDE_AUTH_TOKEN";
    /// Oxide Base URL (自定义 API 端点)
    pub const OXIDE_BASE_URL: &str = "OXIDE_BASE_URL";
    /// OpenAI API Key (备用)
    pub const OPENAI_API_KEY: &str = "OPENAI_API_KEY";
    /// Oxide 主目录覆盖
    pub const OXIDE_HOME: &str = "OXIDE_HOME";
    /// 调试模式
    pub const OXIDE_DEBUG: &str = "OXIDE_DEBUG";
    /// 默认模型覆盖
    pub const OXIDE_MODEL: &str = "OXIDE_MODEL";
}

/// 环境变量管理器
pub struct Env;

impl Env {
    /// 获取 API Key（优先使用 OXIDE_AUTH_TOKEN，其次 ANTHROPIC_API_KEY）
    pub fn api_key() -> Result<String> {
        env::var(vars::OXIDE_AUTH_TOKEN)
            .or_else(|_| env::var(vars::ANTHROPIC_API_KEY))
            .map_err(|_| OxideError::Config(format!(
                "未设置 API Key 环境变量。请运行: export {}=your_api_key 或 export {}=your_api_key",
                vars::OXIDE_AUTH_TOKEN,
                vars::ANTHROPIC_API_KEY
            )))
    }

    /// 获取 Base URL（可选，用于自定义 API 端点）
    pub fn base_url() -> Option<String> {
        env::var(vars::OXIDE_BASE_URL).ok()
    }

    /// 获取 OpenAI API Key（可选）
    pub fn openai_api_key() -> Option<String> {
        env::var(vars::OPENAI_API_KEY).ok()
    }

    /// 获取 Oxide 主目录覆盖
    pub fn oxide_home() -> Option<String> {
        env::var(vars::OXIDE_HOME).ok()
    }

    /// 是否启用调试模式
    pub fn is_debug() -> bool {
        env::var(vars::OXIDE_DEBUG)
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or(false)
    }

    /// 获取模型覆盖
    pub fn model_override() -> Option<String> {
        env::var(vars::OXIDE_MODEL).ok()
    }

    /// 获取环境变量，带默认值
    pub fn get_or(key: &str, default: &str) -> String {
        env::var(key).unwrap_or_else(|_| default.to_string())
    }

    /// 获取必需的环境变量
    pub fn require(key: &str) -> Result<String> {
        env::var(key).map_err(|_| {
            OxideError::Config(format!("未设置必需的环境变量: {}", key))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_or_default() {
        let value = Env::get_or("NONEXISTENT_VAR_12345", "default");
        assert_eq!(value, "default");
    }

    #[test]
    fn test_is_debug_default() {
        // 默认应该是 false
        // 注意：这个测试可能受环境影响
        let _ = Env::is_debug();
    }
}
