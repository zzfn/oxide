use anyhow::Result;
use rig::{
    agent::Agent,
    client::CompletionClient,
    providers::{anthropic, openai::{self, responses_api}},
};

use crate::tools::{
    WrappedCreateDirectoryTool, WrappedDeleteFileTool, WrappedEditFileTool,
    WrappedShellExecuteTool, WrappedGrepSearchTool, WrappedReadFileTool,
    WrappedScanCodebaseTool, WrappedWriteFileTool,
};

macro_rules! build_agent {
    ($client_expr:expr, $model_name:expr, $preamble:expr, $tools:expr, $variant:ident) => {{
        let client = $client_expr?;
        let agent = client
            .agent($model_name)
            .preamble($preamble)
            .max_tokens(4096)
            .tool($tools.read_file)
            .tool($tools.write_file)
            .tool($tools.edit_file)
            .tool($tools.delete_file)
            .tool($tools.shell_execute)
            .tool($tools.scan_codebase)
            .tool($tools.make_dir)
            .tool($tools.grep_find)
            .build();
        Ok(AgentType::$variant(agent))
    }};
}

// Agent enum - 支持多种客户端
pub enum AgentType {
    Anthropic(Agent<anthropic::completion::CompletionModel>),
    OpenAI(Agent<responses_api::ResponsesCompletionModel>),
}

pub struct AgentBuilder {
    base_url: String,
    auth_token: String,
    model: Option<String>,
}

impl AgentBuilder {
    pub fn new(base_url: String, auth_token: String, model: Option<String>) -> Result<Self> {
        Ok(Self {
            base_url,
            auth_token,
            model,
        })
    }

    pub fn build(self) -> Result<AgentType> {
        let tools = self.create_tools();
        let preamble = self.get_preamble();

        // 使用默认模型或配置的模型
        let model_name = self.model.unwrap_or_else(|| "claude-sonnet-4-20250514".to_string());

        // 判断使用哪个客户端
        // 优先检查 URL 路径是否包含 /anthropic，其次检查域名是否包含 anthropic.com
        if self.base_url.contains("/anthropic") || self.base_url.contains("anthropic.com") {
            // Anthropic API 或兼容端点
            build_agent!(
                anthropic::Client::builder()
                    .api_key(&self.auth_token)
                    .base_url(&self.base_url)
                    .build(),
                &model_name,
                &preamble,
                tools,
                Anthropic
            )
        } else {
            // OpenAI 兼容服务
            build_agent!(
                openai::Client::builder()
                    .api_key(&self.auth_token)
                    .base_url(&self.base_url)
                    .build(),
                &model_name,
                &preamble,
                tools,
                OpenAI
            )
        }
    }

    fn create_tools(&self) -> AgentTools {
        AgentTools {
            read_file: WrappedReadFileTool::new(),
            write_file: WrappedWriteFileTool::new(),
            edit_file: WrappedEditFileTool::new(),
            delete_file: WrappedDeleteFileTool::new(),
            shell_execute: WrappedShellExecuteTool::new(),
            scan_codebase: WrappedScanCodebaseTool::new(),
            make_dir: WrappedCreateDirectoryTool::new(),
            grep_find: WrappedGrepSearchTool::new(),
        }
    }

    fn get_preamble(&self) -> String {
        r#"
        Your name is Oxide. You are a helpful AI code assistant with comprehensive file system and command execution access. 
        You can read, write, edit (with patches), and delete files, execute bash commands, scan codebase structures, search text in the codebase and create directories. 
        Use the edit_file tool for making small, targeted changes to existing files - it's more efficient than rewriting entire files.
        Please provide clear and concise responses and be careful when modifying files or executing commands."#.to_string()
    }
}

struct AgentTools {
    read_file: WrappedReadFileTool,
    write_file: WrappedWriteFileTool,
    edit_file: WrappedEditFileTool,
    delete_file: WrappedDeleteFileTool,
    shell_execute: WrappedShellExecuteTool,
    scan_codebase: WrappedScanCodebaseTool,
    make_dir: WrappedCreateDirectoryTool,
    grep_find: WrappedGrepSearchTool,
}

// Convenience function for creating an agent
pub fn create_agent(base_url: String, auth_token: String, model: Option<String>) -> Result<AgentType> {
    AgentBuilder::new(base_url, auth_token, model)?.build()
}
