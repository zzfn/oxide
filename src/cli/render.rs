use anyhow::Result;
use colored::*;

use super::OxideCli;

impl OxideCli {
    pub fn show_welcome(&self) -> Result<()> {
        println!("{}", "✨ Welcome to Oxide CLI!".bright_green());
        println!(
            "{} {} | {} {}",
            "Session:".dimmed(),
            self.context_manager.session_id(),
            "cwd:".dimmed(),
            std::env::current_dir().unwrap().display()
        );
        println!();
        Ok(())
    }

    pub fn show_tips(&self) -> Result<()> {
        println!("{}", "Tips for getting started:".bright_white());
        println!();
        println!(
            "{} Ask questions, edit files, or run commands.",
            "1.".bright_white()
        );
        println!("{} Be specific for the best results.", "2.".bright_white());
        println!("{} Type /help for more information.", "3.".bright_white());
        println!();
        println!(
            "{}",
            "ctrl+c to exit, /help for commands, Tab for completion".dimmed()
        );
        println!();
        Ok(())
    }
}
