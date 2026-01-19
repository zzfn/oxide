mod agent;
mod config;
mod context;
mod hooks;
mod tools;

#[cfg(feature = "cli")]
mod cli;

// TUI module is temporarily disabled during refactor
// #[cfg(feature = "tui")]
// mod tui;

use anyhow::{Context, Result};
use config::Config;
use crate::agent::create_agent;
use crate::cli::OxideCli;
use crate::context::ContextManager;
use names::Generator;

#[tokio::main]
async fn main() -> Result<()> {
    // Load config
    let config = Config::load().context("Failed to load configuration")?;

    if let Err(e) = config.validate() {
        eprintln!("Error: {}", e);
        eprintln!("Tip: Please set DEEPSEEK_API_KEY environment variable");
        eprintln!("Tip: Or create .env file in project root");
        std::process::exit(1);
    }

    // Create Agent using rig-core
    let agent = create_agent(config.api_key.clone(), config.model.clone())
        .context("Failed to create agent")?;

    #[cfg(feature = "cli")]
    {
        // Generate session ID
        let session_id = {
            let mut generator = Generator::default();
            generator.next().unwrap_or_else(|| "unknown-session".to_string())
        };

        // Create ContextManager
        let storage_dir = std::path::PathBuf::from(".oxide/sessions");
        let context_manager = ContextManager::new(storage_dir, session_id)?;

        // Initialize and run CLI
        let mut cli = OxideCli::new(
            config.api_key,
            config.api_url, // Note: config might need to support api_base if distinct from full URL
            config.model,
            agent,
            context_manager
        );
        
        cli.run().await?;
    }

    #[cfg(not(feature = "cli"))]
    {
        println!("Please run with --features cli");
    }

    Ok(())
}
