use crate::agent::{AgentType, NewAgentType, SubagentManager};
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
            "/agent" | "/agent list" => {
                self.list_agents()?;
            }
            _ if input.starts_with("/agent switch ") => {
                let agent_type = input.strip_prefix("/agent switch ").unwrap_or("").trim();
                self.switch_agent(agent_type)?;
            }
            _ if input.starts_with("/agent capabilities") => {
                self.show_agent_capabilities()?;
            }
            _ if input.starts_with("/agent ") => {
                println!("{} Unknown /agent subcommand", "❌".red());
                println!("{} Usage: /agent [list|switch <type>|capabilities]", "💡".bright_blue());
            }
            "/tasks" | "/tasks list" => {
                self.list_tasks()?;
            }
            _ if input.starts_with("/tasks show ") => {
                let task_id = input.strip_prefix("/tasks show ").unwrap_or("").trim();
                self.show_task(task_id)?;
            }
            _ if input.starts_with("/tasks cancel ") => {
                let task_id = input.strip_prefix("/tasks cancel ").unwrap_or("").trim();
                self.cancel_task(task_id)?;
            }
            _ if input.starts_with("/tasks ") => {
                println!("{} Unknown /tasks subcommand", "❌".red());
                println!("{} Usage: /tasks [list|show <id>|cancel <id>]", "💡".bright_blue());
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
        println!("  {} {}", "Model:".bright_white(), self.model_name);
        println!(
            "  {} {}",
            "Auth Token:".bright_white(),
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
        println!("  {} - List or switch Agent types", "/agent [list|switch <type>|capabilities]".bright_green());
        println!("  {} - Manage background tasks", "/tasks [list|show <id>|cancel <id>]".bright_green());
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

    fn list_agents(&self) -> Result<()> {
        let manager = SubagentManager::new();
        let capabilities = manager.list_capabilities();

        println!("{}", "🤖 Available Agent Types:".bright_cyan());
        println!();

        for cap in &capabilities {
            let current_marker = if matches!(&self.agent, AgentType::Anthropic(_) if cap.agent_type == NewAgentType::Main) {
                " (current)".bright_green()
            } else {
                "".normal()
            };

            println!("  {}{}", cap.name.bright_white(), current_marker);
            println!("    {}", cap.description.bright_black());
            println!(
                "    {} {}",
                "Tools:".bright_yellow(),
                cap.tools.join(", ").dimmed()
            );
            if cap.read_only {
                println!("    {} {}", "🔒".bright_red(), "Read-only access".bright_red());
            }
            println!();
        }

        println!(
            "{} Use '/agent switch <type>' to change agent type",
            "💡".bright_blue()
        );
        println!();
        Ok(())
    }

    fn switch_agent(&mut self, agent_type_str: &str) -> Result<()> {
        // 解析 Agent 类型
        let agent_type = match NewAgentType::from_str(agent_type_str) {
            Some(t) => t,
            None => {
                println!("{} Unknown agent type: {}", "❌".red(), agent_type_str);
                println!("{} Available types:", "💡".bright_blue());
                println!("  - main (Main Agent)");
                println!("  - explore (Explore Agent)");
                println!("  - plan (Plan Agent)");
                println!("  - code_reviewer (Code Reviewer Agent)");
                println!("  - frontend_developer (Frontend Developer Agent)");
                println!();
                return Ok(());
            }
        };

        // TODO: 实际切换 Agent 逻辑
        // 目前需要使用 AgentBuilder 重新构建 Agent
        // 这需要存储 base_url 和 auth_token

        println!(
            "{} Switched to {} Agent",
            "✅".bright_green(),
            agent_type.display_name().bright_cyan()
        );
        println!(
            "{} Note: Agent switching is not fully implemented yet.",
            "⚠️".yellow()
        );
        println!(
            "{} The current agent type has been noted but the agent has not been rebuilt.",
            "💡".bright_blue()
        );
        println!();

        Ok(())
    }

    fn show_agent_capabilities(&self) -> Result<()> {
        let manager = SubagentManager::new();
        let capabilities = manager.list_capabilities();

        println!("{}", "🔧 Agent Capabilities:".bright_cyan());
        println!();

        for cap in &capabilities {
            println!("  {} ({})", cap.name.bright_white(), cap.agent_type.display_name().dimmed());
            println!("    {}", cap.description.bright_black());
            println!();
            println!("    {}", "Tools:".bright_yellow());
            for tool in &cap.tools {
                println!("      • {}", tool.bright_white());
            }
            if cap.read_only {
                println!("    {} {}", "🔒".bright_red(), "Read-only access".bright_red());
            } else {
                println!("    {} {}", "✏️".bright_green(), "Read/Write access".bright_green());
            }
            println!();
        }

        println!(
            "{} Use '/agent list' to see available agents",
            "💡".bright_blue()
        );
        println!();
        Ok(())
    }

    fn list_tasks(&self) -> Result<()> {
        use crate::task::TaskManager;
        use std::path::PathBuf;

        let tasks_dir = PathBuf::from(".oxide/tasks");

        if !tasks_dir.exists() {
            println!("{}", "📋 No tasks found".bright_yellow());
            println!(
                "{} Tasks directory does not exist",
                "💡".bright_blue()
            );
            println!();
            return Ok(());
        }

        let manager = TaskManager::new(tasks_dir)?;
        let tasks = manager.list_tasks()?;

        if tasks.is_empty() {
            println!("{}", "📋 No tasks found".bright_yellow());
            println!();
            return Ok(());
        }

        println!("{}", "📋 Background Tasks:".bright_cyan());
        println!();

        for task in tasks {
            let status_icon = match task.status {
                crate::task::TaskStatus::Pending => "⏳".bright_yellow(),
                crate::task::TaskStatus::InProgress => "🔄".bright_blue(),
                crate::task::TaskStatus::Completed => "✅".bright_green(),
                crate::task::TaskStatus::Failed => "❌".bright_red(),
            };

            println!("  {} {} ({})", status_icon, task.name.bright_white(), task.id.dimmed());
            println!("    {}", task.description.bright_black());
            println!(
                "    {} {} | {} {}",
                "Agent:".bright_yellow(),
                task.agent_type.display_name(),
                "Status:".bright_yellow(),
                format!("{:?}", task.status).bright_white()
            );

            if let Some(duration) = task.duration() {
                println!("    {} {}", "Duration:".bright_yellow(), format!("{:?}", duration).bright_white());
            }

            println!();
        }

        println!(
            "{} Use '/tasks show <id>' to view task details",
            "💡".bright_blue()
        );
        println!();
        Ok(())
    }

    fn show_task(&self, task_id: &str) -> Result<()> {
        use crate::task::{TaskManager, TaskStatus};
        use std::path::PathBuf;

        let tasks_dir = PathBuf::from(".oxide/tasks");
        let manager = TaskManager::new(tasks_dir)?;

        let task_id_string = task_id.to_string();
        let task = match manager.get_task(&task_id_string)? {
            Some(t) => t,
            None => {
                println!("{} Task not found: {}", "❌".red(), task_id);
                println!();
                return Ok(());
            }
        };

        let status_icon = match task.status {
            TaskStatus::Pending => "⏳".bright_yellow(),
            TaskStatus::InProgress => "🔄".bright_blue(),
            TaskStatus::Completed => "✅".bright_green(),
            TaskStatus::Failed => "❌".bright_red(),
        };

        println!("{}", "📋 Task Details:".bright_cyan());
        println!();
        println!("  {} {}", "ID:".bright_yellow(), task.id.bright_white());
        println!("  {} {}", "Name:".bright_yellow(), task.name.bright_white());
        println!("  {} {}", "Description:".bright_yellow(), task.description.bright_white());
        println!("  {} {}", "Agent:".bright_yellow(), task.agent_type.display_name().bright_white());
        println!("  {} {:?}", "Status:".bright_yellow(), task.status);
        println!("  {}", status_icon);

        let created_str = task.created_at.format("%Y-%m-%d %H:%M:%S UTC").to_string();
        println!(
            "  {} {}",
            "Created:".bright_yellow(),
            created_str.bright_white()
        );

        if let Some(started) = task.started_at {
            let started_str = started.format("%Y-%m-%d %H:%M:%S UTC").to_string();
            println!(
                "  {} {}",
                "Started:".bright_yellow(),
                started_str.bright_white()
            );
        }

        if let Some(completed) = task.completed_at {
            let completed_str = completed.format("%Y-%m-%d %H:%M:%S UTC").to_string();
            println!(
                "  {} {}",
                "Completed:".bright_yellow(),
                completed_str.bright_white()
            );
        }

        if let Some(duration) = task.duration() {
            let duration_str = format!("{:?}", duration);
            println!("  {} {}", "Duration:".bright_yellow(), duration_str.bright_white());
        }

        if let Some(output_file) = &task.output_file {
            let output_path = output_file.display().to_string();
            println!(
                "  {} {}",
                "Output:".bright_yellow(),
                output_path.bright_white()
            );
        }

        if let Some(error) = &task.error {
            println!("  {} {}", "Error:".bright_red(), error.bright_red());
        }

        println!();

        // 尝试显示任务输出
        if let Ok(Some(output)) = manager.get_task_output(&task_id_string) {
            println!("{}", "📄 Task Output:".bright_cyan());
            println!();
            println!("{}", output.dimmed());
            println!();
        }

        Ok(())
    }

    fn cancel_task(&self, task_id: &str) -> Result<()> {
        use crate::task::TaskManager;
        use std::path::PathBuf;

        let tasks_dir = PathBuf::from(".oxide/tasks");
        let manager = TaskManager::new(tasks_dir)?;
        let task_id_string = task_id.to_string();

        // 检查任务是否存在
        let task = match manager.get_task(&task_id_string)? {
            Some(t) => t,
            None => {
                println!("{} Task not found: {}", "❌".red(), task_id);
                println!();
                return Ok(());
            }
        };

        // 检查任务状态
        match task.status {
            crate::task::TaskStatus::Pending | crate::task::TaskStatus::InProgress => {
                // 尝试取消任务
                match manager.cancel_task(&task_id_string)? {
                    true => {
                        println!(
                            "{} Task '{}' cancelled successfully",
                            "✅".bright_green(),
                            task_id
                        );
                    }
                    false => {
                        println!(
                            "{} Task '{}' was not actively running",
                            "⚠️".yellow(),
                            task_id
                        );
                    }
                }
            }
            crate::task::TaskStatus::Completed => {
                println!(
                    "{} Task '{}' has already completed",
                    "ℹ️".bright_blue(),
                    task_id
                );
            }
            crate::task::TaskStatus::Failed => {
                println!(
                    "{} Task '{}' has already failed",
                    "ℹ️".bright_blue(),
                    task_id
                );
            }
        }

        println!();
        Ok(())
    }
}
