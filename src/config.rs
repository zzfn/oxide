//! 配置模块
//!
//! 支持多层次的配置系统：
//! 1. 全局配置：~/.oxide/config.toml
//! 2. 项目配置：.oxide/config.toml
//! 3. 项目指令：.oxide/CONFIG.md
//! 4. 环境变量（覆盖所有配置）
//!
//! 为了向后兼容，仍支持从环境变量直接加载

use anyhow::{Context, Result};
use std::env;

mod loader;
pub mod secret;
pub use loader::ConfigLoader;
pub use secret::Secret;

const DEFAULT_BASE_URL: &str = "https://api.anthropic.com";
#[allow(dead_code)]
const DEFAULT_MODEL: &str = "claude-sonnet-4-20250514";
const DEFAULT_MAX_TOKENS: u32 = 4096;
const DEFAULT_STREAM_CHARS_PER_TICK: usize = 8;

#[derive(Clone)]
pub struct Config {
    pub base_url: String,
    pub auth_token: Secret<String>,
    pub model: Option<String>,
    #[allow(dead_code)]
    pub max_tokens: u32,
    #[allow(dead_code)]
    pub stream_chars_per_tick: usize,
}

// 手动实现 Debug，防止 auth_token 泄露
impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("base_url", &self.base_url)
            .field("auth_token", &self.auth_token) // Secret 的 Debug 实现会输出 "***"
            .field("model", &self.model)
            .field("max_tokens", &self.max_tokens)
            .field("stream_chars_per_tick", &self.stream_chars_per_tick)
            .finish()
    }
}

impl Config {
    /// 使用新的配置加载器（推荐）
    pub fn load_with_loader() -> Result<Self> {
        let loader = ConfigLoader::new();
        let loaded = loader.load()?;

        Ok(Self {
            base_url: loaded.base_url,
            auth_token: loaded.auth_token, // 已经是 Secret<String>
            model: loaded.model,
            max_tokens: loaded.max_tokens,
            stream_chars_per_tick: loaded.stream_chars_per_tick,
        })
    }

    /// 从环境变量直接加载（向后兼容）
    pub fn load() -> Result<Self> {
        // 优先尝试使用新的配置加载器
        // 如果失败，回退到环境变量
        match Self::load_with_loader() {
            Ok(config) => Ok(config),
            Err(_) => Self::load_from_env(),
        }
    }

    /// 从环境变量加载配置
    fn load_from_env() -> Result<Self> {
        dotenv::dotenv().ok();

        let base_url = env::var("OXIDE_BASE_URL")
            .or_else(|_| env::var("API_URL"))
            .unwrap_or_else(|_| DEFAULT_BASE_URL.to_string());

        let auth_token = env::var("OXIDE_AUTH_TOKEN")
            .or_else(|_| env::var("ANTHROPIC_API_KEY"))
            .or_else(|_| env::var("API_KEY"))
            .context("未找到 OXIDE_AUTH_TOKEN、ANTHROPIC_API_KEY 或 API_KEY 环境变量")?;
        let auth_token = Secret::new(auth_token);

        let model = env::var("MODEL_NAME")
            .or_else(|_| env::var("MODEL"))
            .ok(); // 模型可选，不传则使用服务端默认

        let max_tokens = env::var("MAX_TOKENS")
            .ok()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(DEFAULT_MAX_TOKENS);

        let stream_chars_per_tick = env::var("STREAM_CHARS_PER_TICK")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .filter(|v| *v > 0)
            .unwrap_or(DEFAULT_STREAM_CHARS_PER_TICK);

        Ok(Config {
            base_url,
            auth_token,
            model,
            max_tokens,
            stream_chars_per_tick,
        })
    }

    pub fn validate(&self) -> Result<()> {
        if self.auth_token.expose_secret().is_empty() {
            anyhow::bail!("Auth Token 不能为空");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_config_validation_success() {
        let config = Config {
            base_url: DEFAULT_BASE_URL.to_string(),
            auth_token: Secret::new("test-token".to_string()),
            model: Some(DEFAULT_MODEL.to_string()),
            max_tokens: DEFAULT_MAX_TOKENS,
            stream_chars_per_tick: DEFAULT_STREAM_CHARS_PER_TICK,
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation_empty_key() {
        let config = Config {
            base_url: DEFAULT_BASE_URL.to_string(),
            auth_token: Secret::new("".to_string()),
            model: Some(DEFAULT_MODEL.to_string()),
            max_tokens: DEFAULT_MAX_TOKENS,
            stream_chars_per_tick: DEFAULT_STREAM_CHARS_PER_TICK,
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_load_from_env() {
        env::set_var("OXIDE_AUTH_TOKEN", "test-token");
        let result = Config::load();
        assert!(result.is_ok(), "Failed to load config: {:?}", result);
        let config = result.unwrap();
        assert_eq!(config.auth_token.expose_secret(), "test-token");
        env::remove_var("OXIDE_AUTH_TOKEN");
    }

    #[test]
    fn test_load_with_custom_base_url() {
        dotenv::dotenv().ok();
        env::set_var("OXIDE_AUTH_TOKEN", "test-token");
        env::set_var("OXIDE_BASE_URL", "https://api.example.com");
        let result = Config::load();
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.base_url, "https://api.example.com");
        // 验证 auth_token 不会在 Debug 输出中泄露
        let debug_output = format!("{:?}", config.auth_token);
        assert_eq!(debug_output, "***");
        env::remove_var("OXIDE_AUTH_TOKEN");
        env::remove_var("OXIDE_BASE_URL");
    }

    #[test]
    fn test_load_stream_chars_per_tick() {
        // 清理可能存在的环境变量
        env::remove_var("ANTHROPIC_API_KEY");
        env::remove_var("API_KEY");
        env::remove_var("MODEL");
        env::remove_var("MODEL_NAME");

        env::set_var("OXIDE_AUTH_TOKEN", "test-token");
        env::set_var("STREAM_CHARS_PER_TICK", "12");
        let result = Config::load();
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.stream_chars_per_tick, 12);
        env::remove_var("OXIDE_AUTH_TOKEN");
        env::remove_var("STREAM_CHARS_PER_TICK");
    }
}
