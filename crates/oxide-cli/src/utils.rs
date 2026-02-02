//! 工具函数模块
//!
//! 提供 token 计数等辅助功能。

use once_cell::sync::Lazy;

/// Claude 使用的 tokenizer (cl100k_base)
static TOKENIZER: Lazy<tiktoken_rs::CoreBPE> = Lazy::new(|| {
    tiktoken_rs::cl100k_base().expect("Failed to load cl100k_base tokenizer")
});

/// 计算文本的 token 数量
///
/// 使用 cl100k_base tokenizer（Claude 使用相同的 tokenizer）
pub fn count_tokens(text: &str) -> usize {
    TOKENIZER.encode_with_special_tokens(text).len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_tokens() {
        // 英文单词通常 1 个单词 ≈ 0.75-1 个 token
        let text = "Hello, world!";
        let tokens = count_tokens(text);
        assert!(tokens > 0);

        // 中文通常 1 个汉字 ≈ 2-3 个 token
        let text = "你好，世界！";
        let tokens = count_tokens(text);
        assert!(tokens > 0);
    }

    #[test]
    fn test_empty_string() {
        assert_eq!(count_tokens(""), 0);
    }
}
