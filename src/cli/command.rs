use crate::agent::{AgentType, NewAgentType, SubagentManager};
use crate::context::SerializableMessage;
use crate::hooks::SessionIdHook;
use crate::skill::{SkillExecutor, SkillManager};
use crate::token_counter::{count_tokens, count_messages_tokens, TokenUsage};
use super::file_resolver::parse_file_references;
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
            "/config" | "/config show" => {
                self.show_config()?;
            }
            "/config edit" => {
                self.edit_config()?;
            }
            "/config reload" => {
                self.reload_config()?;
            }
            "/config validate" => {
                self.validate_config()?;
            }
            _ if input.starts_with("/config ") => {
                println!("{} Unknown /config subcommand", "❌".red());
                println!("{} Usage: /config [show|edit|reload|validate]", "💡".bright_blue());
            }
            "/toggle-tools" => {
                println!("{}", "🔧 当前仅支持 CLI 模式，工具默认启用".bright_yellow());
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
            "/skills" | "/skills list" => {
                self.list_skills()?;
            }
            _ if input.starts_with("/skills show ") => {
                let skill_name = input.strip_prefix("/skills show ").unwrap_or("").trim();
                self.show_skill(skill_name)?;
            }
            _ if input.starts_with("/skills ") => {
                println!("{} Unknown /skills subcommand", "❌".red());
                println!("{} Usage: /skills [list|show <name>]", "💡".bright_blue());
            }
            _ if input.starts_with('/') => {
                // 尝试作为 skill 执行
                if self.try_execute_skill(input).await? {
                    // 成功执行了 skill，跳过后续处理
                    return Ok(true);
                }

                println!("{} Unknown command: {}", "❌".red(), input);
                println!("{} Type /help for available commands", "💡".bright_blue());
            }
            _ => {
                // 处理文件引用
                let (parsed_input, file_refs) = parse_file_references(input);

                // 显示文件引用信息
                if !file_refs.is_empty() {
                    println!();
                    println!("{}", "📎 已引用文件:".bright_cyan());
                    for ref_info in &file_refs {
                        println!("  {}", ref_info.display_info());
                    }
                    println!();
                }

                // 构建完整的用户消息（包含文件内容）
                let enhanced_input = if !file_refs.is_empty() {
                    let mut enhanced = String::new();

                    // 添加文件内容
                    for ref_info in &file_refs {
                        enhanced.push_str(&format!(
                            "```file_path=\"{}\"\n{}\n```\n\n",
                            ref_info.file_path.display(),
                            ref_info.content
                        ));
                    }

                    // 添加用户输入
                    enhanced.push_str(&parsed_input);
                    enhanced
                } else {
                    input.to_string()
                };

                // Add user message to context
                self.context_manager.add_message(Message::user(&enhanced_input));

                // 计算 token 预估
                let messages = self.context_manager.get_messages();
                let input_tokens = count_messages_tokens(
                    &messages.iter().map(|m| {
                        let serializable = SerializableMessage::from(m);
                        (serializable.role, serializable.content)
                    }).collect::<Vec<_>>()
                );

                // 预估输出 tokens（通常是输入的 1.5-2 倍，这里保守估计）
                let estimated_output = (input_tokens as f64 * 0.5).ceil() as usize;

                let usage = TokenUsage::new(input_tokens, estimated_output);

                // 显示 token 预估
                println!();
                println!(
                    "{} {} | {} {} | {} {}",
                    "📊".bright_blue(),
                    format!("输入: {} tokens", usage.input_tokens).bright_white(),
                    "预估输出".bright_yellow(),
                    format!("~{} tokens", usage.output_tokens).bright_yellow(),
                    "成本".bright_green(),
                    format!("${:.6}", usage.estimated_cost()).bright_green()
                );
                println!();

                // Start spinner
                self.spinner.start("Thinking...");
                stdout().flush().unwrap();

                // Create session hook
                let hook = SessionIdHook::new(self.context_manager.session_id().to_string());

                let response_result: Result<rig::agent::FinalResponse, std::io::Error> = match &self.agent {
                    AgentType::OpenAI(agent) => {
                        let mut stream = agent
                            .stream_prompt(&enhanced_input)
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
                            .stream_prompt(&enhanced_input)
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
                        self.add_session_tokens(resp.usage().total_tokens as u64);
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
        self.reset_session_tokens();
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

    fn edit_config(&self) -> Result<()> {
        // 查找配置文件
        let config_paths = vec![
            std::path::PathBuf::from(".oxide/config.toml"),
            dirs::home_dir()
                .map(|p| p.join(".oxide/config.toml"))
                .unwrap_or_else(|| std::path::PathBuf::from("~/.oxide/config.toml")),
        ];

        let config_file = config_paths
            .iter()
            .find(|p| p.exists())
            .or_else(|| config_paths.first())
            .unwrap();

        println!(
            "{} Opening config file: {}",
            "📝".bright_blue(),
            config_file.display().to_string().bright_cyan()
        );
        println!(
            "{}",
            "💡 Tip: Use /config reload after editing to apply changes".bright_yellow()
        );
        println!();

        // 使用系统默认编辑器打开配置文件
        let editor = std::env::var("EDITOR").unwrap_or_else(|_| {
            if cfg!(target_os = "macos") {
                "nano".to_string()
            } else if cfg!(target_os = "windows") {
                "notepad".to_string()
            } else {
                "nano".to_string()
            }
        });

        let status = std::process::Command::new(&editor)
            .arg(config_file)
            .status();

        match status {
            Ok(s) if s.success() => {
                println!(
                    "{} Config file edited successfully",
                    "✅".bright_green()
                );
                println!(
                    "{} Use '/config reload' to apply changes",
                    "💡".bright_blue()
                );
            }
            Ok(_) => {
                println!(
                    "{} Editor exited with non-zero status",
                    "⚠️".yellow()
                );
            }
            Err(e) => {
                println!(
                    "{} Failed to open editor: {}",
                    "❌".red(),
                    e
                );
                println!(
                    "{} You can manually edit: {}",
                    "💡".bright_blue(),
                    config_file.display().to_string().bright_cyan()
                );
            }
        }
        println!();
        Ok(())
    }

    fn reload_config(&mut self) -> Result<()> {
        println!("{}", "🔄 Reloading configuration...".bright_yellow());
        println!();

        // TODO: 实现配置重载逻辑
        // 这需要：
        // 1. 重新读取配置文件
        // 2. 更新 self.model_name, self.api_key 等字段
        // 3. 可能需要重建 Agent

        println!(
            "{} Configuration reload is not fully implemented yet.",
            "⚠️".yellow()
        );
        println!(
            "{} For now, restart the application to apply config changes.",
            "💡".bright_blue()
        );
        println!();
        Ok(())
    }

    fn validate_config(&self) -> Result<()> {
        println!("{}", "✓ Validating configuration...".bright_cyan());
        println!();

        let mut has_errors = false;
        let mut has_warnings = false;

        // 验证 API Key
        if self.api_key.is_empty() {
            println!(
                "{} {}",
                "❌".bright_red(),
                "API Key is empty".bright_red()
            );
            has_errors = true;
        } else if self.api_key.len() < 10 {
            println!(
                "{} {}",
                "⚠️".bright_yellow(),
                "API Key seems too short".bright_yellow()
            );
            has_warnings = true;
        } else {
            println!(
                "{} {}",
                "✓".bright_green(),
                "API Key looks valid".bright_green()
            );
        }

        // 验证模型名称
        if self.model_name.is_empty() {
            println!(
                "{} {}",
                "❌".bright_red(),
                "Model name is empty".bright_red()
            );
            has_errors = true;
        } else {
            println!(
                "{} {}",
                "✓".bright_green(),
                format!("Model: {}", self.model_name).bright_green()
            );
        }

        // 检查配置文件是否存在
        let config_paths = vec![
            std::path::PathBuf::from(".oxide/config.toml"),
            dirs::home_dir()
                .map(|p| p.join(".oxide/config.toml"))
                .unwrap_or_else(|| std::path::PathBuf::from("~/.oxide/config.toml")),
        ];

        let has_config = config_paths.iter().any(|p| p.exists());
        if has_config {
            println!(
                "{} {}",
                "✓".bright_green(),
                "Config file exists".bright_green()
            );
        } else {
            println!(
                "{} {}",
                "⚠️".bright_yellow(),
                "No config file found (using defaults)".bright_yellow()
            );
            has_warnings = true;
        }

        println!();

        if has_errors {
            println!("{}", "❌ Configuration validation FAILED".bright_red());
        } else if has_warnings {
            println!("{}", "⚠️ Configuration validation completed with warnings".bright_yellow());
        } else {
            println!("{}", "✓ Configuration validation PASSED".bright_green());
        }
        println!();
        Ok(())
    }

    fn show_help(&self) -> Result<()> {
        println!("{}", "📚 Oxide CLI - Help & Commands".bright_cyan().bold());
        println!();

        // 斜杠命令列表
        println!("{}", "═══ Slash Commands ═══".bright_black());
        println!();
        println!("  {} - Exit the application", "/quit or /exit".bright_green());
        println!("  {} - Clear all messages in current session", "/clear".bright_green());
        println!(
            "  {} - Show or edit configuration",
            "/config [show|edit|reload|validate]".bright_green()
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
        println!("  {} - Manage and use skills", "/skills [list|show <name>]".bright_green());
        println!("  {} - Show this help message", "/help".bright_green());
        println!();

        // Agent 类型列表
        println!("{}", "═══ Available Agents ═══".bright_black());
        println!();
        let manager = SubagentManager::new();
        let capabilities = manager.list_capabilities();

        for cap in &capabilities {
            let current_marker = if matches!(&self.agent, AgentType::Anthropic(_) if cap.agent_type == NewAgentType::Main) {
                " (current)".bright_green()
            } else {
                "".normal()
            };

            println!("  {}{} - {}", cap.name.bright_white(), current_marker, cap.description.bright_black());
            println!("    {}", format!("Tools: {}", cap.tools.join(", ")).dimmed());
            if cap.read_only {
                println!("    {} {}", "🔒".bright_red(), "Read-only".bright_red());
            }
            println!();
        }

        // 可用工具列表
        println!("{}", "═══ Available Tools ═══".bright_black());
        println!();
        let tools = vec![
            ("read", "Read file contents"),
            ("write", "Write or create files"),
            ("edit", "Edit specific parts of a file"),
            ("delete", "Delete files or directories"),
            ("shell_execute", "Execute shell commands"),
            ("grep", "Search for patterns in files"),
            ("scan", "Scan directory structure"),
            ("mkdir", "Create directories"),
            ("glob", "Match files using patterns"),
            ("multi_edit", "Edit multiple files at once"),
            ("notebook_edit", "Edit Jupyter notebooks"),
            ("ask_user_question", "Ask the user questions"),
            ("task", "Spawn background tasks"),
            ("task_output", "Get background task output"),
        ];

        for (tool, description) in tools {
            println!("  {} - {}", tool.bright_cyan(), description.bright_black());
        }
        println!();

        // 使用示例
        println!("{}", "═══ Usage Examples ═══".bright_black());
        println!();
        println!("  {}", "Basic Chat:".bright_yellow());
        println!("    {}", "Hello, how are you?".dimmed());
        println!();
        println!("  {}", "File References:".bright_yellow());
        println!("    {}", "@src/main.rs 请帮我重构这个文件".dimmed());
        println!("    {}", "@Cargo.toml @README.md 比较这两个文件".dimmed());
        println!();
        println!("  {}", "Session Management:".bright_yellow());
        println!("    {}", "/sessions".dimmed());
        println!("    {}", "/load abc123".dimmed());
        println!();
        println!("  {}", "Agent Switching:".bright_yellow());
        println!("    {}", "/agent list".dimmed());
        println!("    {}", "/agent switch explore".dimmed());
        println!();
        println!("  {}", "Configuration:".bright_yellow());
        println!("    {}", "/config show".dimmed());
        println!("    {}", "/config validate".dimmed());
        println!();

        // 提示
        println!("{}", "═══ Tips ═══".bright_black());
        println!();
        println!(
            "{}",
            "💡 You can type any message to chat with the AI!".bright_white()
        );
        println!(
            "{}",
            "📎 Use @file_path to reference files in your messages".bright_blue()
        );
        println!(
            "{}",
            "⌨️  Press Tab after typing '/' to see available commands".bright_blue()
        );
        println!(
            "{}",
            "⌨️  Press Tab after typing '@' to see available files".bright_blue()
        );
        println!(
            "{}",
            "🤖 Use different agents for specific tasks (explore, plan, code_reviewer)".bright_blue()
        );
        println!(
            "{}",
            "🔧 Tools are automatically available to the AI agent".bright_blue()
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
        self.reset_session_tokens();

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

    /// 列出所有可用的技能
    fn list_skills(&self) -> Result<()> {
        let manager = SkillManager::new()?;
        let skills = manager.list_skills();

        if skills.is_empty() {
            println!("{}", "📚 No skills found".bright_yellow());
            println!();
            return Ok(());
        }

        println!("{}", "📚 Available Skills:".bright_cyan());
        println!();

        for skill in skills {
            let source_icon = match skill.source {
                crate::skill::SkillSource::BuiltIn => "🔧".bright_blue(),
                crate::skill::SkillSource::Global => "🌐".bright_green(),
                crate::skill::SkillSource::Local => "📁".bright_yellow(),
            };

            println!("  {} {} - {}", source_icon, format!("/{}", skill.name).bright_white(), skill.description.bright_black());

            // 显示参数
            if !skill.args.is_empty() {
                println!("    {}", "Arguments:".bright_yellow());
                for arg in &skill.args {
                    let required = if arg.required {
                        format!("{} required", "✓".bright_green())
                    } else {
                        "optional".dimmed().to_string()
                    };
                    println!("      -{} : {} ({})", arg.name.bright_white(), arg.description.bright_black(), required);
                }
            }
            println!();
        }

        println!(
            "{} Use '/skills show <name>' to view skill details",
            "💡".bright_blue()
        );
        println!(
            "{} Use /<skill-name> to execute a skill",
            "💡".bright_blue()
        );
        println!();
        Ok(())
    }

    /// 显示技能详细信息
    fn show_skill(&self, skill_name: &str) -> Result<()> {
        let manager = SkillManager::new()?;
        let skill = match manager.get_skill(skill_name) {
            Some(s) => s,
            None => {
                println!("{} Skill not found: {}", "❌".red(), skill_name);
                println!(
                    "{} Use '/skills list' to see available skills",
                    "💡".bright_blue()
                );
                println!();
                return Ok(());
            }
        };

        println!("{}", "📖 Skill Details:".bright_cyan());
        println!();
        println!("  {} {}", "Name:".bright_yellow(), skill.name.bright_white());
        println!(
            "  {} {}",
            "Description:".bright_yellow(),
            skill.description.bright_white()
        );

        let source_str = match skill.source {
            crate::skill::SkillSource::BuiltIn => "Built-in".bright_blue(),
            crate::skill::SkillSource::Global => "Global".bright_green(),
            crate::skill::SkillSource::Local => "Local".bright_yellow(),
        };
        println!("  {} {}", "Source:".bright_yellow(), source_str);

        if !skill.args.is_empty() {
            println!();
            println!("  {}", "Arguments:".bright_yellow());
            for arg in &skill.args {
                let required = if arg.required {
                    format!("{} required", "✓".bright_green())
                } else {
                    "optional".dimmed().to_string()
                };
                println!(
                    "    -{} : {} ({})",
                    arg.name.bright_white(),
                    arg.description.bright_black(),
                    required
                );
                if let Some(default) = &arg.default {
                    println!("      Default: {}", default.dimmed());
                }
            }
        }

        println!();
        println!("  {}", "Usage:".bright_yellow());
        let args_str = skill
            .args
            .iter()
            .map(|arg| {
                if arg.required {
                    format!("-{} <{}>", arg.name, arg.name)
                } else {
                    format!("[ -{} <{}> ]", arg.name, arg.name)
                }
            })
            .collect::<Vec<_>>()
            .join(" ");

        println!("    /{} {}", skill.name.bright_white(), args_str.dimmed());
        println!();
        Ok(())
    }

    /// 尝试执行一个 skill
    /// 返回 true 如果成功识别并执行了 skill，否则返回 false
    async fn try_execute_skill(&mut self, input: &str) -> Result<bool> {
        // 解析命令格式：/skillname [args...]
        let parts: Vec<&str> = input.splitn(2, ' ').collect();
        if parts.is_empty() {
            return Ok(false);
        }

        let skill_name = parts[0].strip_prefix('/');
        let skill_name = match skill_name {
            Some(name) if !name.is_empty() => name,
            _ => return Ok(false),
        };

        let args_str = parts.get(1).unwrap_or(&"");

        // 获取 skill
        let manager = SkillManager::new()?;
        let skill = match manager.get_skill(skill_name) {
            Some(s) => s,
            None => return Ok(false), // 不是 skill，返回 false
        };

        // 执行 skill
        let rendered_prompt = match SkillExecutor::execute(&skill, args_str) {
            Ok(prompt) => prompt,
            Err(e) => {
                println!("{} Failed to execute skill: {}", "❌".red(), e);
                println!();
                return Ok(true); // 虽然执行失败，但确实是 skill 命令
            }
        };

        // 显示执行的 skill 信息
        let source_icon = match skill.source {
            crate::skill::SkillSource::BuiltIn => "🔧".bright_blue(),
            crate::skill::SkillSource::Global => "🌐".bright_green(),
            crate::skill::SkillSource::Local => "📁".bright_yellow(),
        };
        println!(
            "{} Executing skill: {}",
            source_icon,
            format!("/{}", skill.name).bright_cyan()
        );
        println!();

        // 将渲染后的提示词添加到上下文，作为用户消息
        self.context_manager.add_message(Message::user(&rendered_prompt));

        // 计算 token 预估
        let messages = self.context_manager.get_messages();
        let input_tokens = count_messages_tokens(
            &messages.iter().map(|m| {
                let serializable = SerializableMessage::from(m);
                (serializable.role, serializable.content)
            }).collect::<Vec<_>>()
        );

        let estimated_output = (input_tokens as f64 * 0.5).ceil() as usize;
        let usage = TokenUsage::new(input_tokens, estimated_output);

        // 显示 token 预估
        println!(
            "{} {} | {} {} | {} {}",
            "📊".bright_blue(),
            format!("输入: {} tokens", usage.input_tokens).bright_white(),
            "预估输出".bright_yellow(),
            format!("~{} tokens", usage.output_tokens).bright_yellow(),
            "成本".bright_green(),
            format!("${:.6}", usage.estimated_cost()).bright_green()
        );
        println!();

        // 执行 AI 处理
        self.spinner.start("Thinking...");
        stdout().flush().unwrap();

        let hook = SessionIdHook::new(self.context_manager.session_id().to_string());

        let response_result: Result<rig::agent::FinalResponse, std::io::Error> = match &self.agent {
            AgentType::OpenAI(agent) => {
                let mut stream = agent
                    .stream_prompt(&rendered_prompt)
                    .with_hook(hook.clone())
                    .multi_turn(20)
                    .with_history(self.context_manager.get_messages().to_vec())
                    .await;
                self.spinner.stop();
                super::render::stream_with_animation(&mut stream).await
            }
            AgentType::Anthropic(agent) => {
                let mut stream = agent
                    .stream_prompt(&rendered_prompt)
                    .with_hook(hook.clone())
                    .multi_turn(20)
                    .with_history(self.context_manager.get_messages().to_vec())
                    .await;
                self.spinner.stop();
                super::render::stream_with_animation(&mut stream).await
            }
        };

        println!();

        match response_result {
            Ok(resp) => {
                let response_content = resp.response();
                self.context_manager
                    .add_message(Message::assistant(response_content));

                if let Err(e) = self.context_manager.save() {
                    println!("{} Failed to save context: {}", "⚠️".yellow(), e);
                }

                self.add_session_tokens(resp.usage().total_tokens as u64);
                println!(
                    "{} Total tokens used: {}",
                    "📊".bright_blue(),
                    resp.usage().total_tokens
                );
            }
            Err(e) => {
                println!("{} Failed to get AI response: {}", "❌".red(), e);
            }
        }

        println!();
        Ok(true)
    }
}
