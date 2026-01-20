use anyhow::{Context, Result};
use std::env;

const DEFAULT_BASE_URL: &str = "https://api.anthropic.com";
const DEFAULT_MODEL: &str = "claude-sonnet-4-20250514";
const DEFAULT_MAX_TOKENS: u32 = 4096;
const DEFAULT_STREAM_CHARS_PER_TICK: usize = 8;

#[derive(Debug, Clone)]
pub struct Config {
    pub base_url: String,
    pub auth_token: String,
    pub model: Option<String>,
    pub max_tokens: u32,
    pub stream_chars_per_tick: usize,
}

impl Config {
    pub fn load() -> Result<Self> {
        dotenv::dotenv().ok();

        let base_url = env::var("OXIDE_BASE_URL")
            .or_else(|_| env::var("API_URL"))
            .unwrap_or_else(|_| DEFAULT_BASE_URL.to_string());

        let auth_token = env::var("OXIDE_AUTH_TOKEN")
            .or_else(|_| env::var("ANTHROPIC_API_KEY"))
            .or_else(|_| env::var("API_KEY"))
            .context("未找到 OXIDE_AUTH_TOKEN、ANTHROPIC_API_KEY 或 API_KEY 环境变量")?;

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
        if self.auth_token.is_empty() {
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
            auth_token: "test-token".to_string(),
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
            auth_token: "".to_string(),
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
        assert_eq!(config.auth_token, "test-token");
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
        env::remove_var("OXIDE_AUTH_TOKEN");
        env::remove_var("OXIDE_BASE_URL");
    }

    #[test]
    fn test_load_stream_chars_per_tick() {
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
