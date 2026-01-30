//! 测试 Anthropic API 集成
//!
//! 运行方式:
//! ```bash
//! export OXIDE_AUTH_TOKEN=your_api_key
//! export OXIDE_BASE_URL=https://api.anthropic.com  # 可选
//! cargo run --example test_api --package oxide-provider
//! ```

use oxide_core::types::{ContentBlock, Message, Role};
use oxide_provider::{AnthropicProvider, LLMProvider};
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 从环境变量读取配置
    let api_key = env::var("OXIDE_AUTH_TOKEN")
        .or_else(|_| env::var("ANTHROPIC_API_KEY"))
        .expect("请设置 OXIDE_AUTH_TOKEN 或 ANTHROPIC_API_KEY 环境变量");

    let base_url = env::var("OXIDE_BASE_URL").ok();
    let model = env::var("OXIDE_MODEL").ok();

    // 创建 Provider
    let mut provider = AnthropicProvider::new(api_key, model);
    if let Some(url) = base_url {
        provider = provider.with_base_url(url);
    }

    println!("🚀 测试 Anthropic API 集成\n");

    // 测试 1: 简单对话
    println!("📝 测试 1: 简单对话");
    let messages = vec![Message::text(Role::User, "你好！请用一句话介绍你自己。")];

    match provider.complete(&messages).await {
        Ok(response) => {
            println!("✅ 响应成功:");
            for block in &response.content {
                if let ContentBlock::Text { text } = block {
                    println!("   {}", text);
                }
            }
        }
        Err(e) => {
            println!("❌ 请求失败: {}", e);
            return Err(e);
        }
    }

    println!("\n---\n");

    // 测试 2: 流式响应
    println!("📝 测试 2: 流式响应");
    let messages = vec![Message::text(
        Role::User,
        "请用三个词描述 Rust 编程语言。",
    )];

    print!("✅ 流式输出: ");
    match provider
        .complete_stream(
            &messages,
            Box::new(|block| {
                if let ContentBlock::Text { text } = block {
                    print!("{}", text);
                    use std::io::Write;
                    std::io::stdout().flush().unwrap();
                }
            }),
        )
        .await
    {
        Ok(_) => {
            println!("\n✅ 流式响应完成");
        }
        Err(e) => {
            println!("\n❌ 流式请求失败: {}", e);
            return Err(e);
        }
    }

    println!("\n🎉 所有测试通过！");
    Ok(())
}
