mod agent;
mod config;
mod context;
mod hooks;
mod skill;
mod tools;
mod task;

#[cfg(feature = "cli")]
mod cli;

// TUI module is temporarily disabled during refactor
// #[cfg(feature = "tui")]
// mod tui;

use anyhow::{Context, Result};
use config::Config;
use crate::agent::AgentBuilder;
use crate::cli::OxideCli;
use crate::context::ContextManager;
use crate::skill::SkillManager;
use names::Generator;

#[tokio::main]
async fn main() -> Result<()> {
    // Load config
    let config = Config::load().context("Failed to load configuration")?;

    if let Err(e) = config.validate() {
        eprintln!("Error: {}", e);
        eprintln!("Tip: Please set OXIDE_AUTH_TOKEN environment variable");
        eprintln!("Tip: Or create .env file in project root");
        std::process::exit(1);
    }

    // Create Agent using AgentBuilder
    let builder = AgentBuilder::new(
        config.base_url.clone(),
        config.auth_token.clone(),
        config.model.clone(),
    );
    let agent = builder.build_main().context("Failed to create agent")?;

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

        // Initialize SkillManager
        let skill_manager = SkillManager::new()?;
        skill_manager.init()?;

        // Initialize and run CLI
        let mut cli = OxideCli::new(
            config.auth_token,
            config.model.unwrap_or_else(|| "claude-sonnet-4-20250514".to_string()),
            agent,
            context_manager,
        );

        cli.run().await?;
    }

    #[cfg(not(feature = "cli"))]
    {
        println!("Please run with --features cli");
    }

    Ok(())
}
