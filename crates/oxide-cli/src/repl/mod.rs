//! REPL 模块
//!
//! 提供交互式命令行界面，包括编辑器、补全、提示符和主循环。

pub mod completer;
pub mod editor;
pub mod keybindings;
pub mod prompt;

pub use completer::OxideCompleter;
pub use editor::create_editor;
pub use keybindings::create_keybindings;
pub use prompt::OxidePrompt;

use anyhow::Result;
use reedline::Signal;
use std::sync::Arc;

use crate::app::SharedAppState;
use crate::commands::{CommandRegistry, CommandResult};
use crate::render::Renderer;

/// REPL 主循环
pub struct Repl {
    /// 共享应用状态
    state: SharedAppState,
    /// 命令注册表
    commands: Arc<CommandRegistry>,
    /// 渲染器
    renderer: Renderer,
}

impl Repl {
    /// 创建新的 REPL
    pub fn new(state: SharedAppState, commands: Arc<CommandRegistry>) -> Self {
        Self {
            state,
            commands,
            renderer: Renderer::new(),
        }
    }

    /// 运行 REPL 主循环
    pub async fn run(&mut self) -> Result<()> {
        // 显示欢迎信息
        self.renderer.welcome();

        // 获取工作目录
        let working_dir = {
            let state = self.state.read().await;
            state.working_dir.clone()
        };

        // 创建编辑器
        let mut editor = create_editor(self.commands.clone(), working_dir)?;

        loop {
            // 获取当前模式并创建提示符
            let mode = {
                let state = self.state.read().await;
                state.mode
            };
            let prompt = OxidePrompt::new(mode);

            // 读取用户输入
            match editor.read_line(&prompt) {
                Ok(Signal::Success(line)) => {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }

                    // 重置 Ctrl+C 计数
                    {
                        let mut state = self.state.write().await;
                        state.reset_ctrl_c();
                    }

                    // 处理输入
                    if CommandRegistry::is_command(line) {
                        // 执行命令
                        match self.commands.execute(line, self.state.clone()).await {
                            Ok(CommandResult::Exit) => {
                                self.renderer.info("再见！");
                                break;
                            }
                            Ok(CommandResult::Message(msg)) => {
                                self.renderer.markdown(&msg);
                            }
                            Ok(CommandResult::Continue) => {}
                            Err(e) => {
                                self.renderer.error(&format!("命令执行失败: {}", e));
                            }
                        }
                    } else {
                        // 处理普通输入（发送给 AI）
                        self.handle_user_input(line).await?;
                    }
                }
                Ok(Signal::CtrlC) => {
                    let should_exit = {
                        let mut state = self.state.write().await;
                        if state.is_processing {
                            // 如果正在处理，取消当前操作
                            state.end_processing();
                            self.renderer.warning("操作已取消");
                            false
                        } else {
                            // 否则检查是否应该退出
                            state.increment_ctrl_c()
                        }
                    };

                    if should_exit {
                        self.renderer.info("再见！");
                        break;
                    } else {
                        self.renderer.info("再按一次 Ctrl+C 退出");
                    }
                }
                Ok(Signal::CtrlD) => {
                    self.renderer.info("再见！");
                    break;
                }
                Err(e) => {
                    self.renderer.error(&format!("输入错误: {}", e));
                }
            }
        }

        Ok(())
    }

    /// 处理用户输入（发送给 AI）
    async fn handle_user_input(&mut self, input: &str) -> Result<()> {
        // 标记开始处理
        {
            let mut state = self.state.write().await;
            state.start_processing();
        }

        // 检查是否有 Provider
        let provider = {
            let state = self.state.read().await;
            state.provider.clone()
        };

        let Some(provider) = provider else {
            self.renderer.error("AI Provider 未初始化。请设置 OXIDE_AUTH_TOKEN 或 ANTHROPIC_API_KEY 环境变量。");
            let mut state = self.state.write().await;
            state.end_processing();
            return Ok(());
        };

        // 添加用户消息到会话
        {
            let mut state = self.state.write().await;
            state.conversation.add_message(oxide_core::types::Message::text(
                oxide_core::types::Role::User,
                input,
            ));
        }

        // 获取会话消息
        let messages = {
            let state = self.state.read().await;
            state.conversation.messages.clone()
        };

        // 调用 AI（流式响应）
        self.renderer.assistant_header();

        match provider
            .complete_stream(
                &messages,
                Box::new(|block| {
                    if let oxide_core::types::ContentBlock::Text { text } = block {
                        print!("{}", text);
                        use std::io::Write;
                        std::io::stdout().flush().unwrap();
                    }
                }),
            )
            .await
        {
            Ok(response) => {
                println!(); // 换行

                // 添加 AI 响应到会话
                {
                    let mut state = self.state.write().await;
                    state.conversation.add_message(response);
                    // TODO: 从 API 响应中获取实际的 token 使用量
                    state.update_token_usage(input.len() as u64, 100, 0);
                    state.end_processing();
                }
            }
            Err(e) => {
                println!(); // 换行
                self.renderer.error(&format!("AI 请求失败: {}", e));

                // 移除失败的用户消息
                {
                    let mut state = self.state.write().await;
                    state.conversation.messages.pop();
                    state.end_processing();
                }
            }
        }

        Ok(())
    }
}
