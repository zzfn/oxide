mod tools;

use anyhow::{Context, Result};
use colored::Colorize;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;

const API_URL: &str = "https://api.deepseek.com/v1/chat/completions";
const MODEL: &str = "deepseek-chat";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum Content {
    Text(String),
    Array(Vec<ContentBlock>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
enum ContentBlock {
    Text {
        text: String,
    },
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
    content: Content,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<ToolCall>>,
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
            api_key,
            messages: Vec::new(),
            tools,
        }
    }

    async fn add_user_message(&mut self, text: &str) {
        self.messages.push(Message {
            role: "user".to_string(),
            content: Content::Text(text.to_string()),
            tool_call_id: None,
            tool_calls: None,
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
            .header("Authorization", format!("Bearer {}", &self.api_key))
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
        let assistant_msg = &response_json["choices"][0]["message"];

        let mut content_blocks = Vec::new();

        if let Some(content) = assistant_msg.get("content").and_then(|c| c.as_str()) {
            content_blocks.push(ContentBlock::Text {
                text: content.to_string(),
            });
        }

        if let Some(tool_calls) = assistant_msg.get("tool_calls").and_then(|tc| tc.as_array()) {
            for tool_call in tool_calls {
                if let (Some(id), Some(func)) = (
                    tool_call.get("id").and_then(|i| i.as_str()),
                    tool_call.get("function"),
                ) {
                    if let (Some(name), Some(args)) = (
                        func.get("name").and_then(|n| n.as_str()),
                        func.get("arguments").and_then(|a| a.as_str()),
                    ) {
                        let input: serde_json::Value = serde_json::from_str(args)
                            .unwrap_or_else(|_| serde_json::Value::String(args.to_string()));
                        content_blocks.push(ContentBlock::ToolUse {
                            id: id.to_string(),
                            name: name.to_string(),
                            input,
                        });
                    }
                }
            }
        }

        let assistant_message = Message {
            role: "assistant".to_string(),
            content: Content::Array(content_blocks.clone()),
            tool_call_id: None,
            tool_calls: assistant_msg
                .get("tool_calls")
                .and_then(|tc| serde_json::from_value(tc.clone()).ok()),
        };
        self.messages.push(assistant_message);

        Ok(content_blocks)
    }

    async fn execute_tool(&self, tool_use: &ContentBlock) -> Result<ContentBlock> {
        match tool_use {
            ContentBlock::ToolUse { id, name, input } => {
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
                        println!("{} {}", "[Tool]".yellow(), name.bold());
                        println!("{}", input.to_string().dimmed());

                        let result = self.execute_tool(block).await?;
                        tool_results.push(result);
                    }
                    _ => {}
                }
            }

            if has_tool_use {
                for result in &tool_results {
                    if let ContentBlock::ToolResult {
                        tool_use_id,
                        content,
                        is_error: _,
                    } = result
                    {
                        self.messages.push(Message {
                            role: "tool".to_string(),
                            content: Content::Text(content.clone()),
                            tool_call_id: Some(tool_use_id.clone()),
                            tool_calls: None,
                        });
                    }
                }
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

    let api_key = env::var("DEEPSEEK_API_KEY").context("未设置 DEEPSEEK_API_KEY 环境变量")?;

    let mut agent = Agent::new(api_key);

    println!("{} Oxide CLI - DeepSeek Agent", "=".repeat(40).cyan());
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
