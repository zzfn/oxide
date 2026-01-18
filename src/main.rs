mod config;
mod tools;

#[cfg(feature = "tui")]
mod tui;

use anyhow::{Context, Result};
use clap::Parser;
use colored::Colorize;
use config::Config;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::io::Write;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser, Debug)]
#[command(name = "oxide")]
#[command(about = "Oxide CLI - DeepSeek Agent", long_about = None)]
struct Args {
    #[arg(long, help = "禁用 TUI 界面，使用简单终端模式")]
    no_tui: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Message {
    role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<ToolCall>>,
}

impl Message {
    fn user(text: &str) -> Self {
        Message {
            role: "user".to_string(),
            content: Some(text.to_string()),
            tool_call_id: None,
            tool_calls: None,
        }
    }

    fn assistant_with_text(text: &str) -> Self {
        Message {
            role: "assistant".to_string(),
            content: Some(text.to_string()),
            tool_call_id: None,
            tool_calls: None,
        }
    }

    fn assistant_with_tool_calls(tool_calls: Vec<ToolCall>) -> Self {
        Message {
            role: "assistant".to_string(),
            content: None,
            tool_call_id: None,
            tool_calls: Some(tool_calls),
        }
    }

    fn tool_result(tool_use_id: &str, content: &str) -> Self {
        Message {
            role: "tool".to_string(),
            content: Some(content.to_string()),
            tool_call_id: Some(tool_use_id.to_string()),
            tool_calls: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ToolCall {
    id: String,
    #[serde(rename = "type")]
    call_type: String,
    function: FunctionCall,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FunctionCall {
    name: String,
    arguments: String,
}

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

#[derive(Debug, Clone)]
enum Command {
    Exit,
    Help,
    Clear,
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
            "/exit" => Some(Command::Exit),
            "/help" => Some(Command::Help),
            "/clear" => Some(Command::Clear),
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
    pub messages: Vec<Message>,
    tools: Vec<Tool>,
    message_count: usize,
}

impl Agent {
    fn new(config: Config) -> Self {
        let client = Client::new();
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

        Agent {
            client,
            config,
            messages: Vec::new(),
            tools,
            message_count: 0,
        }
    }

    fn clear_history(&mut self) {
        self.messages.clear();
        self.message_count = 0;
    }

    fn display_welcome(&self) {
        println!();
        println!("{}", "=".repeat(50).cyan());
        println!(
            "{} {} - DeepSeek Agent",
            "Oxide CLI".bold().cyan(),
            VERSION.dimmed()
        );
        println!("{}", "=".repeat(50).cyan());
        println!("{} {}", "模型:".green(), self.config.model.bold());
        println!("{} 输入 {}", "提示:".yellow(), "/help".cyan().bold());
        println!("{} 输入 {} 退出", "提示:".yellow(), "/exit".cyan().bold());
        println!();
    }

    fn show_help() {
        println!();
        println!("{}", "可用命令:".bold().cyan());
        println!("  {}  - 显示此帮助信息", "/help".cyan());
        println!("  {}  - 清空对话历史", "/clear".cyan());
        println!("  {}  - 退出程序", "/exit".cyan());
        println!();
    }

    fn get_prompt(&self) -> String {
        format!("{}[{}] ", "你>".green().bold(), self.message_count)
    }

    async fn add_user_message(&mut self, text: &str) {
        self.messages.push(Message::user(text));
        self.message_count += 1;
    }

    async fn send_message(&mut self) -> Result<AssistantResponse> {
        let request_body = json!({
            "model": self.config.model,
            "max_tokens": self.config.max_tokens,
            "messages": self.messages,
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

        self.messages.push(assistant_message);

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
            "shell_execute" => {
                let command = input["command"].as_str().context("缺少 command 参数")?;
                tools::shell_execute(command)?
            }
            _ => anyhow::bail!("未知工具: {}", name),
        };

        Ok(result.to_string())
    }

    async fn run(&mut self, user_input: &str) -> Result<()> {
        self.add_user_message(user_input).await;

        loop {
            print!("{}", "思考中...".dimmed().italic());
            std::io::stdout().flush()?;
            print!("\r{}\r", " ".repeat(20));
            std::io::stdout().flush()?;

            let response = self.send_message().await?;

            match response {
                AssistantResponse::Text(text) => {
                    println!("{}", text.cyan());
                    break;
                }
                AssistantResponse::ToolCalls(tool_calls) => {
                    for tool_call in &tool_calls {
                        println!("{} {}", "[工具]".yellow(), tool_call.function.name.bold());
                        println!("{}", tool_call.function.arguments.dimmed());

                        let result = self.execute_tool(tool_call).await?;
                        self.messages
                            .push(Message::tool_result(&tool_call.id, &result));
                    }
                }
            }
        }

        Ok(())
    }

    async fn handle_command(&mut self, command: Command) -> Result<bool> {
        match command {
            Command::Exit => {
                println!("{}", "再见!".cyan());
                return Ok(true);
            }
            Command::Help => {
                Self::show_help();
            }
            Command::Clear => {
                self.clear_history();
                println!("{}", "对话历史已清空".green());
            }
            Command::Unknown(cmd) => {
                println!("{} 未知命令: {}", "错误:".red(), cmd);
                println!("{} 输入 {} 查看可用命令", "提示:".yellow(), "/help".cyan());
            }
        }
        Ok(false)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let config = Config::load().context("加载配置失败")?;

    if let Err(e) = config.validate() {
        eprintln!("{} {}", "错误:".red(), e);
        eprintln!("{} 请设置 DEEPSEEK_API_KEY 环境变量", "提示:".yellow());
        eprintln!(
            "{} 或在项目根目录创建 .env 文件并添加该变量",
            "提示:".yellow()
        );
        std::process::exit(1);
    }

    let mut agent = Agent::new(config.clone());

    if !args.no_tui {
        return run_tui_mode(agent).await;
    }

    agent.display_welcome();

    loop {
        print!("{}", agent.get_prompt());
        std::io::stdout().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        if let Some(command) = Command::parse(input) {
            let should_exit = agent.handle_command(command).await?;
            if should_exit {
                break;
            }
            continue;
        }

        if let Err(e) = agent.run(input).await {
            println!("{} {}", "错误:".red(), e);
        }

        println!();
    }

    Ok(())
}

async fn run_tui_mode(agent: Agent) -> Result<()> {
    use crossterm::{
        cursor::Hide,
        event::{DisableMouseCapture, EnableMouseCapture},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    };
    use ratatui::{backend::CrosstermBackend, Terminal};
    use std::sync::Arc;
    use tokio::sync::RwLock;

    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture, Hide)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let (tui_tx, mut tui_rx) = tokio::sync::mpsc::unbounded_channel();

    let app = Arc::new(RwLock::new(crate::tui::App::new(
        agent.config.model.to_string(),
    )));
    app.write().await.set_event_sender(tui_tx.clone());

    let agent_shared = Arc::new(RwLock::new(agent));

    let mut event_handler = crate::tui::EventHandler::new(250);

    let result = loop {
        {
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
                    Some(crate::tui::Event::Resize(_, _)) => {
                        let app_ref = app.read().await;
                        terminal.draw(|f| crate::tui::render(f, &*app_ref)).unwrap();
                    }
                    Some(crate::tui::Event::Tick) => {
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
                                _ => {
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
                                        app_clone.write().await.add_message(
                                            crate::tui::MessageType::Assistant,
                                            text.clone()
                                        );
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
                                                    agent.messages.push(
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
                            app_clone.write().await.set_state(crate::tui::AppState::Normal);
                        });
                    }
                    crate::tui::TuiEvent::NavigateUp => {
                        app.write().await.scroll_up(5);
                    }
                    crate::tui::TuiEvent::NavigateDown => {
                        app.write().await.scroll_down(5);
                    }
                    crate::tui::TuiEvent::PageUp => {
                        app.write().await.scroll_up(20);
                    }
                    crate::tui::TuiEvent::PageDown => {
                        app.write().await.scroll_down(20);
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
                        let app_ref = app.read().await;
                        terminal.draw(|f| crate::tui::render(f, &*app_ref)).unwrap();
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
        LeaveAlternateScreen,
        DisableMouseCapture,
        crossterm::cursor::Show
    )?;
    terminal.show_cursor()?;

    result
}
