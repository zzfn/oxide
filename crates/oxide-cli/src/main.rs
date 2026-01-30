//! Oxide CLI 入口
//!
//! AI 编程助手命令行界面。

use anyhow::Result;
use clap::Parser;
use oxide_core::Env;
use oxide_provider::RigAnthropicProvider;
use std::path::PathBuf;

use oxide_cli::{commands, create_shared_state, repl::Repl};

/// Oxide - AI 编程助手
#[derive(Parser, Debug)]
#[command(name = "oxide")]
#[command(author, version, about, long_about = None)]
struct Args {
    /// 运行模式: normal, fast, plan
    #[arg(short, long, default_value = "normal")]
    mode: String,

    /// 工作目录
    #[arg(short, long)]
    dir: Option<String>,

    /// 直接执行的提示（非交互模式）
    #[arg(short, long)]
    prompt: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // 创建共享状态
    let state = create_shared_state();

    // 初始化工作目录
    let working_dir = args.dir.clone().map(PathBuf::from).unwrap_or_else(|| {
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    });

    // 初始化 LLM Provider
    {
        let mut state = state.write().await;

        // 设置工作目录
        state.working_dir = working_dir;

        // 从环境变量获取配置
        match Env::api_key() {
            Ok(api_key) => {
                let base_url = Env::base_url();
                let model = Env::model_override();

                let provider = if let Some(url) = base_url {
                    RigAnthropicProvider::with_base_url(api_key, url, model)
                } else {
                    RigAnthropicProvider::new(api_key, model)
                };

                // 使用新的 rig provider
                state.set_rig_provider(provider);
                eprintln!("✅ LLM Provider 已初始化 (rig-core)");
            }
            Err(e) => {
                eprintln!("⚠️  警告: {}", e);
                eprintln!("   AI 功能将不可用。请设置 ANTHROPIC_API_KEY 环境变量。");
            }
        }

        // 设置运行模式
        match args.mode.to_lowercase().as_str() {
            "fast" | "f" => state.set_mode(oxide_cli::CliMode::Fast),
            "plan" | "p" => state.set_mode(oxide_cli::CliMode::Plan),
            _ => {} // 默认 Normal
        }
    }

    // 创建命令注册表
    let commands = commands::create_registry();

    // 如果提供了 prompt，直接执行并退出
    if let Some(prompt) = args.prompt {
        // TODO: 实现非交互模式
        println!("非交互模式尚未实现。提示: {}", prompt);
        return Ok(());
    }

    // 启动 REPL
    let mut repl = Repl::new(state, commands);
    repl.run().await?;

    Ok(())
}
