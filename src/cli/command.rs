use crate::agent::AgentType;
use crate::context::SerializableMessage;
use crate::hooks::SessionIdHook;
use anyhow::Result;
use colored::*;
use rig::completion::Message;
use rig::streaming::StreamingPrompt;
use std::io::{stdout, Write};

use super::render::stream_with_animation;
use super::OxideCli;

impl OxideCli {
    pub async fn handle_command(&mut self, input: &str) -> Result<bool> {
        match input {
            "/quit" | "/exit" => {
                return Ok(false);
            }
            "/clear" => {
                self.clear_context()?;
            }
            "/config" => {
                self.show_config()?;
            }
            "/toggle-tools" => {
                println!("{}", "🔧 Tool toggle is available in TUI mode".bright_yellow());
                println!("   In CLI mode, tools are always enabled");
                println!();
            }
            "/help" => {
                self.show_help()?;
            }
            "/history" => {
                self.show_history()?;
            }
            _ if input.starts_with("/load ") => {
                let session_id = input.strip_prefix("/load ").unwrap_or("").trim();
                self.load_session(session_id)?;
            }
            _ if input.starts_with("/sessions") => {
                self.list_sessions()?;
            }
            _ if input.starts_with("/delete ") => {
                let session_id = input.strip_prefix("/delete ").unwrap_or("").trim();
                if !session_id.is_empty() {
                    self.delete_session(session_id)?;
                } else {
                    println!("{} Usage: /delete <session_id>", "❌".red());
                }
            }
            _ if input.starts_with('/') => {
                println!("{} Unknown command: {}", "❌".red(), input);
                println!("{} Type /help for available commands", "💡".bright_blue());
            }
            _ => {
                // Add user message to context
                self.context_manager.add_message(Message::user(input));

                // Start spinner
                self.spinner.start("Thinking...");
                stdout().flush().unwrap();

                // Create session hook
                let hook = SessionIdHook::new(self.context_manager.session_id().to_string());

                let response_result: Result<rig::agent::FinalResponse, std::io::Error> = match &self.agent {
                    AgentType::OpenAI(agent) => {
                        let mut stream = agent
                            .stream_prompt(input)
                            .with_hook(hook.clone())
                            .multi_turn(20)
                            .with_history(self.context_manager.get_messages().to_vec())
                            .await;
                        // Stop spinner before response starts
                        self.spinner.stop();
                        stream_with_animation(&mut stream).await
                    }
                    AgentType::Anthropic(agent) => {
                        let mut stream = agent
                            .stream_prompt(input)
                            .with_hook(hook.clone())
                            .multi_turn(20)
                            .with_history(self.context_manager.get_messages().to_vec())
                            .await;
                        self.spinner.stop();
                        stream_with_animation(&mut stream).await
                    }
                    AgentType::Cohere(agent) => {
                        let mut stream = agent
                            .stream_prompt(input)
                            .with_hook(hook.clone())
                            .multi_turn(20)
                            .with_history(self.context_manager.get_messages().to_vec())
                            .await;
                        self.spinner.stop();
                        stream_with_animation(&mut stream).await
                    }
                    AgentType::DeepSeek(agent) => {
                        let mut stream = agent
                            .stream_prompt(input)
                            .with_hook(hook.clone())
                            .multi_turn(20)
                            .with_history(self.context_manager.get_messages().to_vec())
                            .await;
                        self.spinner.stop();
                        stream_with_animation(&mut stream).await
                    }
                    AgentType::Ollama(agent) => {
                        let mut stream = agent
                            .stream_prompt(input)
                            .with_hook(hook.clone())
                            .multi_turn(20)
                            .with_history(self.context_manager.get_messages().to_vec())
                            .await;
                        self.spinner.stop();
                        stream_with_animation(&mut stream).await
                    }
                };

                println!();

                match response_result {
                    Ok(resp) => {
                        // Get response content and add to context
                        let response_content = resp.response();
                        self.context_manager
                            .add_message(Message::assistant(response_content));

                        // Auto-save context
                        if let Err(e) = self.context_manager.save() {
                            println!("{} Failed to save context: {}", "⚠️".yellow(), e);
                        }

                        // We can't easily get token usage from the stream response in rig currently without more complex handling,
                        // or if stream_to_stdout returns it.
                        // rig 0.28 stream_to_stdout returns Result<StreamingResponse> which has a usage method? 
                        // Let's assume it works.
                         println!(
                            "{} Total tokens used: {}",
                            "📊".bright_blue(),
                            resp.usage().total_tokens
                        );
                    }
                    Err(e) => {
                        println!("{} Failed to get AI response: {}", "❌".red(), e);
                        println!(
                            "{} Please check your API key and network connection",
                            "💡".bright_blue()
                        );
                    }
                }
            }
        }
        println!(); 
        Ok(true)
    }

    fn clear_context(&mut self) -> Result<()> {
        self.context_manager.clear();
        println!(
            "{} Context cleared. Current session: {}",
            "✅".bright_green(),
            self.context_manager.session_id().bright_cyan()
        );
        println!();
        Ok(())
    }

    fn show_config(&self) -> Result<()> {
        println!("{}", "⚙️  Current Configuration:".bright_cyan());
        println!("  {} {}", "API Base:".bright_white(), self.api_base);
        println!("  {} {}", "Model:".bright_white(), self.model_name);
        println!(
            "  {} {}",
            "API Key:".bright_white(),
            "*".repeat(self.api_key.len().min(8))
        );
        println!();
        Ok(())
    }

    fn show_help(&self) -> Result<()> {
        println!("{}", "📚 Available Commands:".bright_cyan());
        println!();
        println!("  {} - Exit the application", "/quit or /exit".bright_green());
        println!("  {} - Clear all messages in current session", "/clear".bright_green());
        println!(
            "  {} - Show current model configuration",
            "/config".bright_green()
        );
        println!(
            "  {} - Show conversation history",
            "/history".bright_green()
        );
        println!(
            "  {} - Load specific session",
            "/load <session_id>".bright_green()
        );
        println!("  {} - List all sessions", "/sessions".bright_green());
        println!(
            "  {} - Delete a specific session",
            "/delete <session_id>".bright_green()
        );
        println!("  {} - Show this help message", "/help".bright_green());
        println!();
        println!(
            "{}",
            "💡 You can also type any message to chat with the AI!".bright_white()
        );
        println!(
            "{}",
            "⌨️  Press Tab after typing '/' to see available commands".bright_blue()
        );
        println!();
        Ok(())
    }

    fn show_history(&self) -> Result<()> {
        let messages = self.context_manager.get_messages();
        if messages.is_empty() {
            println!(
                "{} No conversation history in current session",
                "📝".bright_blue()
            );
            println!(
                "  Current session: {}",
                self.context_manager.session_id().bright_white()
            );
        } else {
            println!(
                "{} Conversation History (Session: {})",
                "📝".bright_blue(),
                self.context_manager.session_id().bright_white()
            );
            println!();

            for (i, message) in messages.iter().enumerate() {
                let serializable = SerializableMessage::from(message);
                let role_color = match serializable.role.as_str() {
                    "user" => "👤 User".bright_cyan(),
                    "assistant" => "🤖 Assistant".bright_green(),
                    "tool" => "🔧 Tool".bright_yellow(),
                    _ => "❓ Unknown".bright_yellow(),
                };

                println!("{}. {}", (i + 1).to_string().bright_white(), role_color);

                // Display content
                let content = if serializable.content.chars().count() > 200 {
                    format!(
                        "{}...",
                        serializable.content.chars().take(200).collect::<String>()
                    )
                } else {
                    serializable.content
                };

                for line in content.lines() {
                    println!("   {}", line);
                }
                println!();
            }

            println!("{} Total messages: {}", "📊".bright_blue(), messages.len());
        }
        println!();
        Ok(())
    }

    fn list_sessions(&self) -> Result<()> {
        match self.context_manager.list_sessions() {
            Ok(sessions) => {
                if sessions.is_empty() {
                    println!("{} No saved sessions found", "📁".bright_blue());
                } else {
                    println!("{} Available Sessions:", "📁".bright_blue());
                    println!();

                    for (i, session) in sessions.iter().enumerate() {
                        let current_marker = if session.session_id == self.context_manager.session_id() {
                            " (current)".bright_green()
                        } else {
                            "".normal()
                        };

                        println!(
                            "{}. {} - {} messages{}",
                            (i + 1).to_string().bright_white(),
                            session.session_id.bright_cyan(),
                            session.message_count.to_string().bright_yellow(),
                            current_marker
                        );
                        println!("   Last updated: {}", session.last_updated.dimmed());
                    }

                    println!();
                    println!(
                        "{} Use '/load <session_id>' to load a session",
                        "💡".bright_blue()
                    );
                }
            }
            Err(e) => {
                println!("{} Failed to list sessions: {}", "❌".red(), e);
            }
        }
        println!();
        Ok(())
    }

    fn load_session(&mut self, session_id: &str) -> Result<()> {
        // Save current session
        if !self.context_manager.get_messages().is_empty() {
            if let Err(e) = self.context_manager.save() {
                println!(
                    "{} Warning: Failed to save current session: {}",
                    "⚠️".yellow(),
                    e
                );
            }
        }

        // Switch
        self.context_manager.switch_session(session_id.to_string());

        match self.context_manager.load() {
            Ok(true) => {
                println!(
                    "{} Successfully loaded session: {}",
                    "✅".bright_green(),
                    session_id.bright_cyan()
                );
                println!(
                    "   Messages loaded: {}",
                    self.context_manager
                        .get_messages()
                        .len()
                        .to_string()
                        .bright_yellow()
                    );
            }
            Ok(false) => {
                println!(
                    "{} Session '{}' not found, created new session",
                    "📝".bright_blue(),
                    session_id.bright_cyan()
                );
            }
            Err(e) => {
                println!(
                    "{} Failed to load session '{}': {}",
                    "❌".red(),
                    session_id.bright_cyan(),
                    e
                );
            }
        }
        println!();
        Ok(())
    }

    fn delete_session(&mut self, session_id: &str) -> Result<()> {
        if session_id == self.context_manager.session_id() {
            println!("{} Cannot delete current active session", "❌".red());
            println!("   Switch to another session first using '/load <session_id>'",);
            println!();
            return Ok(());
        }

        // Create temp context manager
        let storage_dir = std::path::PathBuf::from(".oxide/sessions");
        let temp_context = crate::context::ContextManager::new(storage_dir, session_id.to_string())?;

        match temp_context.delete_session() {
            Ok(true) => {
                println!(
                    "{} Successfully deleted session: {}",
                    "✅".bright_green(),
                    session_id.bright_cyan()
                );
            }
            Ok(false) => {
                println!(
                    "{} Session '{}' not found",
                    "❌".red(),
                    session_id.bright_cyan()
                );
            }
            Err(e) => {
                println!(
                    "{} Failed to delete session '{}': {}",
                    "❌".red(),
                    session_id.bright_cyan(),
                    e
                );
            }
        }
        println!();
        Ok(())
    }
}
