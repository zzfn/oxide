mod config;
mod context;
mod tools;

#[cfg(feature = "cli")]
mod cli;

#[cfg(feature = "tui")]
mod tui;

use anyhow::{Context, Result};
use colored::Colorize;
use config::Config;
use context::{ContextManager, FunctionCall, Message, ToolCall};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Tool {
    #[serde(rename = "type")]
    tool_type: String,
    function: FunctionDefinition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FunctionDefinition {
    name: String,
    description: String,
    parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ToolCallResponse {
    id: String,
    #[serde(rename = "type")]
    call_type: String,
    function: FunctionCall,
}

impl From<ToolCallResponse> for ToolCall {
    fn from(value: ToolCallResponse) -> Self {
        ToolCall {
            id: value.id,
            call_type: value.call_type,
            function: value.function,
        }
    }
}

#[derive(Debug, Clone)]
enum Command {
    Exit,
    Clear,
    Help,
    Config,
    ToggleTools,
    Unknown(String),
}

impl Command {
    fn parse(input: &str) -> Option<Self> {
        if !input.starts_with('/') {
            return None;
        }

        let parts: Vec<&str> = input.trim().split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }

        match parts[0] {
            "/exit" | "/quit" => Some(Command::Exit),
            "/clear" => Some(Command::Clear),
            "/help" => Some(Command::Help),
            "/config" => Some(Command::Config),
            "/toggle-tools" => Some(Command::ToggleTools),
            _ => Some(Command::Unknown(parts[0].to_string())),
        }
    }
}

#[derive(Debug, Clone)]
enum AssistantResponse {
    Text(String),
    ToolCalls(Vec<ToolCall>),
}

struct Agent {
    client: Client,
    pub config: Config,
    context_manager: ContextManager,
    tools: Vec<Tool>,
}

impl Agent {
    fn new(config: Config) -> Result<Self> {
        let client = Client::new();
        let storage_dir = std::path::PathBuf::from(".oxide/sessions");
        let session_id = names::Generator::default().next().unwrap();
        let context_manager = ContextManager::new(storage_dir, session_id)?;

        let tools = vec![
            Tool {
                tool_type: "function".to_string(),
                function: FunctionDefinition {
                    name: "read_file".to_string(),
                    description: "读取文件内容".to_string(),
                    parameters: json!({
                        "type": "object",
                        "properties": {
                            "path": {
                                "type": "string",
                                "description": "文件路径"
                            }
                        },
                        "required": ["path"]
                    }),
                },
            },
            Tool {
                tool_type: "function".to_string(),
                function: FunctionDefinition {
                    name: "write_file".to_string(),
                    description: "写入文件内容，如果目录不存在会自动创建".to_string(),
                    parameters: json!({
                        "type": "object",
                        "properties": {
                            "path": {
                                "type": "string",
                                "description": "文件路径"
                            },
                            "content": {
                                "type": "string",
                                "description": "文件内容"
                            }
                        },
                        "required": ["path", "content"]
                    }),
                },
            },
            Tool {
                tool_type: "function".to_string(),
                function: FunctionDefinition {
                    name: "edit_file".to_string(),
                    description: "使用 unified diff patch 编辑文件（适用于小范围修改）".to_string(),
                    parameters: json!({
                        "type": "object",
                        "properties": {
                            "file_path": {
                                "type": "string",
                                "description": "文件路径"
                            },
                            "patch": {
                                "type": "string",
                                "description": "Unified diff patch 字符串"
                            }
                        },
                        "required": ["file_path", "patch"]
                    }),
                },
            },
            Tool {
                tool_type: "function".to_string(),
                function: FunctionDefinition {
                    name: "create_directory".to_string(),
                    description: "创建目录（包括父目录）".to_string(),
                    parameters: json!({
                        "type": "object",
                        "properties": {
                            "path": {
                                "type": "string",
                                "description": "目录路径"
                            }
                        },
                        "required": ["path"]
                    }),
                },
            },
            Tool {
                tool_type: "function".to_string(),
                function: FunctionDefinition {
                    name: "delete_file".to_string(),
                    description: "删除文件或目录".to_string(),
                    parameters: json!({
                        "type": "object",
                        "properties": {
                            "path": {
                                "type": "string",
                                "description": "文件或目录路径"
                            }
                        },
                        "required": ["path"]
                    }),
                },
            },
            Tool {
                tool_type: "function".to_string(),
                function: FunctionDefinition {
                    name: "grep_search".to_string(),
                    description: "使用正则表达式搜索文件中的文本".to_string(),
                    parameters: json!({
                        "type": "object",
                        "properties": {
                            "root_path": {
                                "type": "string",
                                "description": "搜索的根目录"
                            },
                            "query": {
                                "type": "string",
                                "description": "正则表达式搜索模式"
                            },
                            "max_results": {
                                "type": "integer",
                                "description": "最大结果数（默认 100）"
                            }
                        },
                        "required": ["root_path", "query"]
                    }),
                },
            },
            Tool {
                tool_type: "function".to_string(),
                function: FunctionDefinition {
                    name: "scan_codebase".to_string(),
                    description: "扫描并显示代码库目录结构".to_string(),
                    parameters: json!({
                        "type": "object",
                        "properties": {
                            "path": {
                                "type": "string",
                                "description": "扫描的目录路径"
                            },
                            "max_depth": {
                                "type": "integer",
                                "description": "最大深度（默认 10）"
                            }
                        },
                        "required": ["path"]
                    }),
                },
            },
            Tool {
                tool_type: "function".to_string(),
                function: FunctionDefinition {
                    name: "shell_execute".to_string(),
                    description: "在终端执行 Shell 命令".to_string(),
                    parameters: json!({
                        "type": "object",
                        "properties": {
                            "command": {
                                "type": "string",
                                "description": "要执行的命令"
                            }
                        },
                        "required": ["command"]
                    }),
                },
            },
        ];

        Ok(Agent {
            client,
            config,
            context_manager,
            tools,
        })
    }

    async fn add_user_message(&mut self, text: &str) {
        self.context_manager.add_message(Message::user(text));
    }

    async fn send_message(&mut self) -> Result<AssistantResponse> {
        let request_body = json!({
            "model": self.config.model,
            "max_tokens": self.config.max_tokens,
            "messages": self.context_manager.get_messages(),
            "tools": self.tools
        });

        let response = self
            .client
            .post(&self.config.api_url)
            .header("Authorization", format!("Bearer {}", &self.config.api_key))
            .header("content-type", "application/json")
            .json(&request_body)
            .send()
            .await
            .context("发送请求失败")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            anyhow::bail!("API 请求失败 ({}): {}", status, error_text);
        }

        let response_json: serde_json::Value = response.json().await?;
        let assistant_msg = &response_json["choices"][0]["message"];

        let assistant_message =
            if let Some(content) = assistant_msg.get("content").and_then(|c| c.as_str()) {
                Message::assistant_with_text(content)
            } else if let Some(tool_calls) =
                assistant_msg.get("tool_calls").and_then(|tc| tc.as_array())
            {
                let calls: Vec<ToolCall> = tool_calls
                    .iter()
                    .filter_map(|tc| serde_json::from_value(tc.clone()).ok())
                    .collect();
                Message::assistant_with_tool_calls(calls)
            } else {
                anyhow::bail!("无效的 AI 响应格式");
            };

        self.context_manager.add_message(assistant_message);

        let response = if let Some(content) = assistant_msg.get("content").and_then(|c| c.as_str())
        {
            AssistantResponse::Text(content.to_string())
        } else if let Some(tool_calls) =
            assistant_msg.get("tool_calls").and_then(|tc| tc.as_array())
        {
            let calls: Vec<ToolCall> = tool_calls
                .iter()
                .filter_map(|tc| serde_json::from_value(tc.clone()).ok())
                .collect();
            AssistantResponse::ToolCalls(calls)
        } else {
            anyhow::bail!("无效的 AI 响应格式");
        };

        Ok(response)
    }

    async fn execute_tool(&self, tool_call: &ToolCall) -> Result<String> {
        let name = &tool_call.function.name;
        let input: serde_json::Value = serde_json::from_str(&tool_call.function.arguments)
            .unwrap_or_else(|_| serde_json::Value::String(tool_call.function.arguments.clone()));

        let result = match name.as_str() {
            "read_file" => {
                let path = input["path"].as_str().context("缺少 path 参数")?;
                tools::read_file(path)?
            }
            "write_file" => {
                let path = input["path"].as_str().context("缺少 path 参数")?;
                let content = input["content"].as_str().context("缺少 content 参数")?;
                tools::write_file(path, content)?
            }
            "edit_file" => {
                let file_path = input["file_path"].as_str().context("缺少 file_path 参数")?;
                let patch = input["patch"].as_str().context("缺少 patch 参数")?;
                tools::edit_file(file_path, patch)?
            }
            "create_directory" => {
                let path = input["path"].as_str().context("缺少 path 参数")?;
                tools::create_directory(path)?
            }
            "delete_file" => {
                let path = input["path"].as_str().context("缺少 path 参数")?;
                tools::delete_file(path)?
            }
            "grep_search" => {
                let root_path = input["root_path"].as_str().context("缺少 root_path 参数")?;
                let query = input["query"].as_str().context("缺少 query 参数")?;
                let max_results = match input.get("max_results") {
                    Some(v) => Some(v.as_u64().context("max_results 格式错误")? as usize),
                    None => None,
                };
                tools::grep_search(root_path, query, max_results)?
            }
            "scan_codebase" => {
                let path = input["path"].as_str().context("缺少 path 参数")?;
                let max_depth = match input.get("max_depth") {
                    Some(v) => Some(v.as_u64().context("max_depth 格式错误")? as usize),
                    None => None,
                };
                tools::scan_codebase(path, max_depth)?
            }
            "shell_execute" => {
                let command = input["command"].as_str().context("缺少 command 参数")?;
                tools::shell_execute(command)?
            }
            _ => anyhow::bail!("未知工具: {}", name),
        };

        Ok(result.to_string())
    }

}

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::load().context("加载配置失败")?;

    if let Err(e) = config.validate() {
        eprintln!("错误: {}", e);
        eprintln!("提示: 请设置 DEEPSEEK_API_KEY 环境变量");
        eprintln!("提示: 或在项目根目录创建 .env 文件并添加该变量");
        std::process::exit(1);
    }

    let agent = Agent::new(config.clone()).context("创建 agent 失败")?;

    // 根据 feature 选择运行模式
    #[cfg(feature = "cli")]
    {
        run_cli_mode(agent).await
    }

    #[cfg(all(feature = "tui", not(feature = "cli")))]
    {
        run_tui_mode(agent).await
    }

    #[cfg(not(any(feature = "cli", feature = "tui")))]
    {
        eprintln!("错误: 请启用至少一个功能特性 (cli 或 tui)");
        eprintln!("提示: 使用 --features cli 或 --features tui");
        std::process::exit(1);
    }
}

#[cfg(feature = "cli")]
async fn run_cli_mode(mut agent: Agent) -> Result<()> {
    // 创建 CLI 实例，传入 context_manager
    let context_manager = agent.context_manager.clone();
    let mut cli = cli::OxideCli::new(context_manager);

    // 显示 LOGO 和欢迎横幅
    println!("{}", cli::LOGO);
    cli.show_welcome()?;
    cli.show_tips()?;

    // 主循环
    use rustyline::error::ReadlineError;
    use rustyline::Editor;

    let mut rl = Editor::new()?;
    rl.set_helper(Some(cli::OxideHelper::default()));

    loop {
        cli.print_separator()?;
        let readline = rl.readline("❯ ");

        match readline {
            Ok(line) => {
                let input = line.trim();
                if input.is_empty() {
                    continue;
                }

                let _ = rl.add_history_entry(input);
                cli.print_separator()?;

                // 处理命令
                let should_continue = cli.handle_command(input).await?;
                if !should_continue {
                    break;
                }

                // 如果不是命令，则是用户消息
                if !input.starts_with('/') {
                    // 添加用户消息到上下文
                    agent.add_user_message(input).await;

                    println!("{}", "🧠 Thinking...".yellow());
                    println!("{}", "● oxide:".blue());

                    // 处理 AI 响应（可能包含工具调用）
                    loop {
                        match agent.send_message().await? {
                            crate::AssistantResponse::Text(text) => {
                                println!("{}", text);
                                break;
                            }
                            crate::AssistantResponse::ToolCalls(tool_calls) => {
                                for tool_call in &tool_calls {
                                    println!(
                                        "{} {}",
                                        "🔧".bright_yellow(),
                                        tool_call.function.name.bright_white()
                                    );
                                    println!(
                                        "{}",
                                        tool_call.function.arguments.dimmed()
                                    );

                                    let result = agent.execute_tool(tool_call).await?;
                                    agent.context_manager.add_message(Message::tool_result(
                                        &tool_call.id,
                                        &result,
                                    ));
                                    println!(
                                        "{} {}",
                                        "✓".bright_green(),
                                        "Success".bright_green()
                                    );
                                }
                            }
                        }
                    }

                    println!();

                    // 保存会话
                    if let Err(e) = agent.context_manager.save() {
                        println!("{} Failed to save context: {}", "⚠️".yellow(), e);
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("{}", "^C".dimmed());
                break;
            }
            Err(ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                println!("{} {:?}", "Error:".red(), err);
                break;
            }
        }
    }

    println!("\n{}", "👋 Goodbye!".bright_cyan());
    Ok(())
}

#[cfg(feature = "tui")]
async fn run_tui_mode(agent: Agent) -> Result<()> {
    use crossterm::{
        cursor::Hide,
        event::{DisableMouseCapture, EnableMouseCapture},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
    };
    use ratatui::{
        backend::CrosstermBackend,
        Terminal,
        TerminalOptions,
        Viewport,
    };
    use std::sync::Arc;
    use tokio::sync::RwLock;

    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnableMouseCapture, Hide)?;
    execute!(stdout, Clear(ClearType::All))?;  // 清屏
    let backend = CrosstermBackend::new(stdout);

    // 明确使用全屏 viewport
    let mut terminal = Terminal::with_options(
        backend,
        TerminalOptions {
            viewport: Viewport::Fullscreen,
        },
    )?;

    let (tui_tx, mut tui_rx) = tokio::sync::mpsc::unbounded_channel();

    let app = Arc::new(RwLock::new(crate::tui::App::new(
        agent.config.model.to_string(),
        agent.context_manager.session_id().to_string(),
        agent.config.stream_chars_per_tick,
    )));
    app.write().await.set_event_sender(tui_tx.clone());

    // 保存配置信息用于命令显示
    let api_url = agent.config.api_url.clone();
    let model_name = agent.config.model.clone();
    let max_tokens = agent.config.max_tokens;

    let agent_shared = Arc::new(RwLock::new(agent));

    let mut event_handler = crate::tui::EventHandler::new(250);

    let result = loop {
        let should_draw = {
            let mut app_guard = app.write().await;
            app_guard.take_dirty()
        };

        if should_draw {
            let app_ref = app.read().await;
            terminal.draw(|f| crate::tui::render(f, &*app_ref)).unwrap();
        }

        tokio::select! {
            event = event_handler.receiver.recv() => {
                match event {
                    Some(crate::tui::Event::Input(key)) => {
                        if key.code == crossterm::event::KeyCode::Char('c') &&
                           key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
                            break Ok(());
                        }

                        let should_exit = crate::tui::handle_key_event(key, &tui_tx)?;
                        if should_exit {
                            break Ok(());
                        }
                    }
                    Some(crate::tui::Event::Mouse(mouse)) => {
                        crate::tui::handle_mouse_event(mouse, &tui_tx)?;
                    }
                    Some(crate::tui::Event::Resize(_, _)) => {
                        app.write().await.mark_dirty();
                    }
                    Some(crate::tui::Event::Tick) => {
                        app.write().await.tick();
                    }
                    _ => {}
                }
            }
            Some(tui_event) = tui_rx.recv() => {
                match tui_event {
                    crate::tui::TuiEvent::Exit => {
                        break Ok(());
                    }
                    crate::tui::TuiEvent::Input(input) => {
                        app.write().await.append_input(input.chars().next().unwrap());
                    }
                    crate::tui::TuiEvent::Backspace => {
                        app.write().await.remove_last_char();
                    }
                    crate::tui::TuiEvent::SendMessage => {
                        let input = app.read().await.get_input();
                        if input.is_empty() {
                            continue;
                        }

                        if let Some(command) = Command::parse(&input) {
                            app.write().await.clear_input();
                            match command {
                                Command::Exit => {
                                    break Ok(());
                                }
                                Command::Clear => {
                                    app.write().await.clear_messages();
                                    continue;
                                }
                                Command::Help => {
                                    let help_text = r#"
📚 Available Commands:

  /exit或/quit   - Exit the application
  /clear         - Clear all messages
  /config        - Show current model configuration
  /toggle-tools  - Toggle tool panel
  /help          - Show this help message

💡 You can also type any message to chat with the AI!
"#;
                                    app.write().await.add_message(
                                        crate::tui::MessageType::Assistant,
                                        help_text.to_string()
                                    );
                                    continue;
                                }
                                Command::Config => {
                                    let config_text = format!(
                                        "⚙️  Current Configuration:\n  API URL: {}\n  Model: {}\n  Max Tokens: {}",
                                        api_url, model_name, max_tokens
                                    );
                                    app.write().await.add_message(
                                        crate::tui::MessageType::Assistant,
                                        config_text
                                    );
                                    continue;
                                }
                                Command::ToggleTools => {
                                    app.write().await.toggle_tool_panel();
                                    continue;
                                }
                                Command::Unknown(cmd) => {
                                    app.write().await.add_message(
                                        crate::tui::MessageType::Assistant,
                                        format!("❌ Unknown command: {}\n💡 Type /help for available commands", cmd)
                                    );
                                    continue;
                                }
                            }
                        }

                        app.write().await.add_message(crate::tui::MessageType::User, input.clone());
                        app.write().await.clear_input();
                        app.write().await.set_state(crate::tui::AppState::Processing);

                        let input_clone = input.clone();
                        let agent_shared_clone = agent_shared.clone();
                        let app_clone = app.clone();

                        tokio::spawn(async move {
                            let mut agent = agent_shared_clone.write().await;
                            agent.add_user_message(&input_clone).await;

                            loop {
                                let response = agent.send_message().await;
                                match response {
                                    Ok(AssistantResponse::Text(text)) => {
                                        app_clone
                                            .write()
                                            .await
                                            .start_streaming_message(text.clone());
                                        break;
                                    }
                                    Ok(AssistantResponse::ToolCalls(tool_calls)) => {
                                        for tool_call in &tool_calls {
                                            app_clone.write().await.update_tool_status(
                                                tool_call.function.name.clone(),
                                                "执行中...".to_string()
                                            );
                                            app_clone.write().await.add_tool_message(
                                                &tool_call.function.name,
                                                tool_call.function.arguments.clone()
                                            );

                                            let result = agent.execute_tool(tool_call).await;
                                            match result {
                                                Ok(result_str) => {
                                                    agent.context_manager.add_message(
                                                        Message::tool_result(&tool_call.id, &result_str)
                                                    );
                                                    app_clone.write().await.update_tool_status(
                                                        tool_call.function.name.clone(),
                                                        "成功".to_string()
                                                    );
                                                }
                                                Err(e) => {
                                                    app_clone.write().await.update_tool_status(
                                                        tool_call.function.name.clone(),
                                                        format!("失败: {}", e)
                                                    );
                                                }
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        app_clone.write().await.set_state(
                                            crate::tui::AppState::Error(e.to_string())
                                        );
                                        break;
                                    }
                                }
                            }
                            let mut app_guard = app_clone.write().await;
                            if !app_guard.has_active_streaming() {
                                app_guard.set_state(crate::tui::AppState::Normal);
                            }
                        });
                    }
                    crate::tui::TuiEvent::NavigateUp => {
                        app.write().await.scroll_up(1);
                    }
                    crate::tui::TuiEvent::NavigateDown => {
                        app.write().await.scroll_down(1);
                    }
                    crate::tui::TuiEvent::PageUp => {
                        app.write().await.scroll_up(10);
                    }
                    crate::tui::TuiEvent::PageDown => {
                        app.write().await.scroll_down(10);
                    }
                    crate::tui::TuiEvent::ScrollToTop => {
                        app.write().await.scroll_to_top();
                    }
                    crate::tui::TuiEvent::ScrollToBottom => {
                        app.write().await.scroll_to_bottom();
                    }
                    crate::tui::TuiEvent::Command(cmd) => {
                        if cmd == "/toggle-tools" {
                            app.write().await.toggle_tool_panel();
                        }
                    }
                    crate::tui::TuiEvent::Resize(_, _) => {
                        app.write().await.mark_dirty();
                    }
                    crate::tui::TuiEvent::Tick => {
                    }
                }
            }
        }
    };

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        DisableMouseCapture,
        crossterm::cursor::Show
    )?;
    terminal.show_cursor()?;

    result
}
