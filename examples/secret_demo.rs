//! Secret 类型使用示例
//!
//! 演示如何使用 Secret 类型保护敏感信息

use oxide::config::secret::Secret;
use zeroize::Zeroize;

fn main() {
    println!("=== Secret 类型保护演示 ===\n");

    // 创建 Secret 实例
    let api_key = Secret::new("sk-ant-api03-very-secret-key-12345".to_string());

    // 1. Debug 输出会被保护
    println!("1. Debug 输出 (应该显示 ***):");
    println!("   {:?}", api_key);

    // 2. Display 输出会被保护
    println!("\n2. Display 输出 (应该显示 ***):");
    println!("   {}", api_key);

    // 3. 通过 expose_secret() 访问真实值
    println!("\n3. 访问真实值 (仅用于实际使用):");
    println!("   实际 API Key: {}***", &api_key.expose_secret()[..20]);

    // 4. 测试 zeroize() 方法
    println!("\n4. 测试 zeroize():");
    let mut sensitive = Secret::new("sensitive-data".to_string());
    println!("   清零前: {}", sensitive.expose_secret());
    sensitive.zeroize();
    println!("   清零后: '{}'", sensitive.expose_secret());

    // 5. 测试克隆
    println!("\n5. 测试克隆:");
    let secret1 = Secret::new("my-secret".to_string());
    let secret2 = secret1.clone();
    println!("   secret1 == secret2: {}", secret1 == secret2);

    // 6. 演示在错误消息中的保护
    println!("\n6. 在结构体中使用 Secret:");
    #[derive(Debug)]
    struct Config {
        base_url: String,
        api_key: Secret<String>,
    }

    let config = Config {
        base_url: "https://api.anthropic.com".to_string(),
        api_key: Secret::new("sk-ant-secret".to_string()),
    };

    println!("   完整配置: {:?}", config);
    println!("   API Key 仍受保护: {}", config.api_key);

    println!("\n=== 演示完成 ===");
}
