use std::sync::OnceLock;

/// Token 分词器缓存
static CL100K_BASE: OnceLock<tiktoken_rs::CoreBPE> = OnceLock::new();

/// 获取 GPT-4/GPT-3.5 使用的 cl100k_base 分词器
fn get_cl100k_base() -> &'static tiktoken_rs::CoreBPE {
    CL100K_BASE.get_or_init(|| {
        tiktoken_rs::cl100k_base().expect("Failed to load cl100k_base tokenizer")
    })
}

/// 估算文本的 token 数量
///
/// # 参数
/// - `text`: 要计算的文本
///
/// # 返回
/// token 数量
#[allow(dead_code)]
pub fn count_tokens(text: &str) -> usize {
    let bpe = get_cl100k_base();
    bpe.encode_with_special_tokens(text).len()
}

/// Token 使用统计
#[derive(Debug, Clone, Default)]
pub struct TokenUsage {
    /// 输入 token 数
    pub input_tokens: usize,
    /// 输出 token 数（预估）
    pub output_tokens: usize,
    /// 总 token 数
    #[allow(dead_code)]
    pub total_tokens: usize,
}

impl TokenUsage {
    /// 创建新的 token 统计
    pub fn new(input_tokens: usize, output_tokens: usize) -> Self {
        Self {
            input_tokens,
            output_tokens,
            total_tokens: input_tokens + output_tokens,
        }
    }

    /// 计算预估成本（美元）
    ///
    /// 基于 GPT-4 定价（输入 $0.03/1K tokens, 输出 $0.06/1K tokens）
    pub fn estimated_cost(&self) -> f64 {
        const INPUT_COST_PER_1K: f64 = 0.03;
        const OUTPUT_COST_PER_1K: f64 = 0.06;

        let input_cost = (self.input_tokens as f64 / 1000.0) * INPUT_COST_PER_1K;
        let output_cost = (self.output_tokens as f64 / 1000.0) * OUTPUT_COST_PER_1K;
        input_cost + output_cost
    }
}

/// 计算消息列表的 token 数量
pub fn count_messages_tokens(messages: &[(String, String)]) -> usize {
    let bpe = get_cl100k_base();

    // 每条消息的开销（格式化 tokens）
    // 参考: https://github.com/openai/openai-cookbook/blob/main/examples/How_to_count_tokens_with_tiktoken.ipynb
    let mut total = 3; // 每个回复的 primtokens

    for (_role, content) in messages {
        // 每条消息: `<|start|>{role}<|message|>\n{content}<|end|>`
        total += bpe.encode_with_special_tokens(_role).len();
        total += bpe.encode_with_special_tokens(content).len();
        total += 4; // <|start|>, <|message|>, \n, <|end|>
    }

    total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_tokens() {
        let text = "Hello, world!";
        let count = count_tokens(text);
        assert!(count > 0);
        println!("'{}' uses {} tokens", text, count);
    }

    #[test]
    fn test_count_empty() {
        assert_eq!(count_tokens(""), 0);
    }

    #[test]
    fn test_token_usage() {
        let usage = TokenUsage::new(1000, 500);
        assert_eq!(usage.input_tokens, 1000);
        assert_eq!(usage.output_tokens, 500);
        assert_eq!(usage.total_tokens, 1500);

        let cost = usage.estimated_cost();
        assert!(cost > 0.0);
        println!("Estimated cost: ${:.6}", cost);
    }

    #[test]
    fn test_count_messages() {
        let messages = vec![
            ("system".to_string(), "You are a helpful assistant.".to_string()),
            ("user".to_string(), "Hello!".to_string()),
        ];
        let count = count_messages_tokens(&messages);
        assert!(count > 0);
        println!("Messages use {} tokens", count);
    }
}
