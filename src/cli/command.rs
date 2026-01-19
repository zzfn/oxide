use crate::context::{ContextManager, SerializableMessage};
use anyhow::Result;
use colored::*;

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
                // 配置信息由外部传入，这里只显示提示
                println!("{}", "⚙️  Configuration loaded from environment".bright_cyan());
                println!("  API URL: Set via API_URL or default");
                println!("  Model: Set via MODEL_NAME or default");
                println!("  Max Tokens: Set via MAX_TOKENS or default");
                println!();
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
                // 返回 true 表示需要处理用户消息（在 main.rs 中处理）
                // 这里不做任何处理，返回 Ok(true) 让主循环继续
                return Ok(true);
            }
        }
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

                // 显示内容
                if let Some(content) = &serializable.content {
                    // 限制显示长度，避免输出过长
                    let display_content = if content.chars().count() > 200 {
                        format!(
                            "{}...",
                            content.chars().take(200).collect::<String>()
                        )
                    } else {
                        content.clone()
                    };

                    // 缩进显示内容
                    for line in display_content.lines() {
                        println!("   {}", line);
                    }
                }

                // 显示工具调用
                if let Some(tool_calls) = &serializable.tool_calls {
                    for tool_call in tool_calls {
                        println!(
                            "   {} {}",
                            "🔧".bright_yellow(),
                            tool_call.function.name.bright_white()
                        );
                    }
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
        // 保存当前会话
        if !self.context_manager.get_messages().is_empty() {
            if let Err(e) = self.context_manager.save() {
                println!(
                    "{} Warning: Failed to save current session: {}",
                    "⚠️".yellow(),
                    e
                );
            }
        }

        // 切换到新会话
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

        // 创建临时上下文管理器来删除指定会话
        let storage_dir = std::path::PathBuf::from(".oxide/sessions");
        let temp_context = ContextManager::new(storage_dir, session_id.to_string())?;

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

// 为 Message 实现 SerializableMessage 转换
impl From<&crate::context::Message> for SerializableMessage {
    fn from(msg: &crate::context::Message) -> Self {
        SerializableMessage {
            role: msg.role.clone(),
            content: msg.content.clone(),
            tool_call_id: msg.tool_call_id.clone(),
            tool_calls: msg.tool_calls.clone().map(|calls| {
                calls
                    .into_iter()
                    .map(|call| crate::context::SerializableToolCall {
                        id: call.id,
                        call_type: call.call_type,
                        function: crate::context::SerializableFunctionCall {
                            name: call.function.name,
                            arguments: call.function.arguments,
                        },
                    })
                    .collect()
            }),
        }
    }
}
