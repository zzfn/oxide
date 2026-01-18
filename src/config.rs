use anyhow::{Context, Result};
use std::env;

const DEFAULT_API_URL: &str = "https://api.deepseek.com/v1/chat/completions";
const DEFAULT_MODEL: &str = "deepseek-chat";
const DEFAULT_MAX_TOKENS: u32 = 4096;

#[derive(Debug, Clone)]
pub struct Config {
    pub api_key: String,
    pub api_url: String,
    pub model: String,
    pub max_tokens: u32,
}

impl Config {
    pub fn load() -> Result<Self> {
        dotenv::dotenv().ok();

        let api_key = env::var("API_KEY")
            .or_else(|_| env::var("DEEPSEEK_API_KEY"))
            .context("未找到 API_KEY 或 DEEPSEEK_API_KEY 环境变量，请设置该变量后再运行程序")?;

        let api_url = env::var("API_URL").unwrap_or_else(|_| DEFAULT_API_URL.to_string());

        let model = env::var("MODEL_NAME")
            .or_else(|_| env::var("MODEL"))
            .unwrap_or_else(|_| DEFAULT_MODEL.to_string());

        let max_tokens = env::var("MAX_TOKENS")
            .ok()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(DEFAULT_MAX_TOKENS);

        Ok(Config {
            api_key,
            api_url,
            model,
            max_tokens,
        })
    }

    pub fn validate(&self) -> Result<()> {
        if self.api_key.is_empty() {
            anyhow::bail!("API Key 不能为空");
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
            api_key: "test-key".to_string(),
            api_url: DEFAULT_API_URL.to_string(),
            model: DEFAULT_MODEL.to_string(),
            max_tokens: DEFAULT_MAX_TOKENS,
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation_empty_key() {
        let config = Config {
            api_key: "".to_string(),
            api_url: DEFAULT_API_URL.to_string(),
            model: DEFAULT_MODEL.to_string(),
            max_tokens: DEFAULT_MAX_TOKENS,
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_load_from_env() {
        env::set_var("API_KEY", "test-key");
        let result = Config::load();
        assert!(result.is_ok(), "Failed to load config: {:?}", result);
        let config = result.unwrap();
        assert_eq!(config.api_key, "test-key");
        env::remove_var("API_KEY");
        env::remove_var("DEEPSEEK_API_KEY");
    }

    #[test]
    fn test_load_with_custom_model() {
        dotenv::dotenv().ok();
        env::set_var("API_KEY", "test-key");
        env::set_var("MODEL", "custom-model");
        let result = Config::load();
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.model, "custom-model");
        env::remove_var("API_KEY");
        env::remove_var("MODEL");
    }
}
