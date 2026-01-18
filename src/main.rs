mod tools;

use anyhow::{Context, Result};
use colored::Colorize;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::env;

const API_URL: &str = "https://api.anthropic.com/v1/messages";
const MODEL: &str = "claude-3-5-sonnet-20241022";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
enum ContentBlock {
    Text { text: String },
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    ToolResult {
        tool_use_id: String,
        content: String,
        is_error: Option<bool>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Message {
    role: String,
    content: Vec<ContentBlock>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Tool {
    name: String,
    description: String,
    input_schema: serde_json::Value,
}

struct Agent {
    client: Client,
    api_key: String,
    messages: Vec<Message>,
    tools: Vec<Tool>,
}

impl Agent {
    fn new(api_key: String) -> Self {
        let client = Client::new();
        let tools = vec![
            Tool {
                name: "read_file".to_string(),
                description: "读取文件内容".to_string(),
                input_schema: json!({
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
            Tool {
                name: "write_file".to_string(),
                description: "写入文件内容，如果目录不存在会自动创建".to_string(),
                input_schema: json!({
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
            Tool {
                name: "shell_execute".to_string(),
                description: "在终端执行 Shell 命令".to_string(),
                input_schema: json!({
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
        ];

        Agent {
            client,
            api_key,
            messages: Vec::new(),
            tools,
        }
    }

    async fn add_user_message(&mut self, text: &str) {
        self.messages.push(Message {
            role: "user".to_string(),
            content: vec![ContentBlock::Text {
                text: text.to_string(),
            }],
        });
    }

    async fn send_message(&mut self) -> Result<Vec<ContentBlock>> {
        let request_body = json!({
            "model": MODEL,
            "max_tokens": 4096,
            "messages": self.messages,
            "tools": self.tools
        });

        let response = self
            .client
            .post(API_URL)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request_body)
            .send()
            .await
            .context("发送请求失败")?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("API 请求失败: {}", error_text);
        }

        let response_json: serde_json::Value = response.json().await?;
        let assistant_content: Vec<ContentBlock> =
            serde_json::from_value(response_json["content"].clone())?;

        let assistant_message = Message {
            role: "assistant".to_string(),
            content: assistant_content.clone(),
        };
        self.messages.push(assistant_message);

        Ok(assistant_content)
    }

    async fn execute_tool(&self, tool_use: &ContentBlock) -> Result<ContentBlock> {
        match tool_use {
            ContentBlock::ToolUse { id, name, input } => {
                let result = match name.as_str() {
                    "read_file" => {
                        let path = input["path"]
                            .as_str()
                            .context("缺少 path 参数")?;
                        tools::read_file(path)?
                    }
                    "write_file" => {
                        let path = input["path"]
                            .as_str()
                            .context("缺少 path 参数")?;
                        let content = input["content"]
                            .as_str()
                            .context("缺少 content 参数")?;
                        tools::write_file(path, content)?
                    }
                    "shell_execute" => {
                        let command = input["command"]
                            .as_str()
                            .context("缺少 command 参数")?;
                        tools::shell_execute(command)?
                    }
                    _ => anyhow::bail!("未知工具: {}", name),
                };

                Ok(ContentBlock::ToolResult {
                    tool_use_id: id.clone(),
                    content: result.to_string(),
                    is_error: None,
                })
            }
            _ => anyhow::bail!("无效的工具使用块"),
        }
    }

    async fn run(&mut self, user_input: &str) -> Result<()> {
        self.add_user_message(user_input).await;

        loop {
            let content_blocks = self.send_message().await?;

            let mut tool_results = Vec::new();
            let mut has_tool_use = false;

            for block in &content_blocks {
                match block {
                    ContentBlock::Text { text } => {
                        println!("{}", text.cyan());
                    }
                    ContentBlock::ToolUse { name, input, .. } => {
                        has_tool_use = true;
                        println!(
                            "{} {}",
                            "[Tool]".yellow(),
                            name.bold()
                        );
                        println!("{}", input.to_string().dimmed());

                        let result = self.execute_tool(block).await?;
                        tool_results.push(result);
                    }
                    _ => {}
                }
            }

            if has_tool_use {
                self.messages.push(Message {
                    role: "user".to_string(),
                    content: tool_results,
                });
            } else {
                break;
            }
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    let api_key = env::var("ANTHROPIC_API_KEY")
        .context("未设置 ANTHROPIC_API_KEY 环境变量")?;

    let mut agent = Agent::new(api_key);

    println!("{} Oxide CLI - Claude 3.5 Sonnet Agent", "=".repeat(40).cyan());
    println!("{} 输入 /exit 退出\n", "提示:".yellow());

    loop {
        print!("{} ", "用户>".green().bold());
        use std::io::Write;
        std::io::stdout().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        if input == "/exit" {
            println!("{}", "再见!".cyan());
            break;
        }

        if let Err(e) = agent.run(input).await {
            eprintln!("{} {}", "错误:".red(), e);
        }

        println!();
    }

    Ok(())
}
