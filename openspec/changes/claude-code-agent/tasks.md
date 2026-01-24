# Implementation Tasks

## 1. 项目设置和依赖

- [ ] 1.1 在 `Cargo.toml` 中添加 `portable-pty` 依赖
- [ ] 1.2 在 `Cargo.toml` 中添加 `async-trait` 依赖（如需要）
- [ ] 1.3 创建 `src/external/` 模块目录（通用基础设施）
- [ ] 1.4 创建 `src/parsers/` 模块目录（各工具的解析器）
- [ ] 1.5 在 `src/lib.rs` 或 `src/main.rs` 中声明 `external` 和 `parsers` 模块

## 2. PTY 管理器（通用）

- [ ] 2.1 创建 `src/external/pty.rs`，实现 `PtyManager` 结构体
- [ ] 2.2 实现 `PtyManager::new()` - 创建虚拟终端
- [ ] 2.3 实现 `PtyManager::writer()` - 获取写入句柄
- [ ] 2.4 实现 `PtyManager::reader()` - 获取读取句柄
- [ ] 2.5 实现 `PtyManager::resize()` - 终端大小调整
- [ ] 2.6 添加 PTY 创建失败的错误处理
- [ ] 2.7 编写 PTY 管理器的单元测试

## 3. 通用进程管理器

- [ ] 3.1 创建 `src/external/process.rs`，实现 `ExternalProcess` 结构体
- [ ] 3.2 实现 `ExternalProcess::new()` - 使用 `tokio::process::Command` 启动进程
- [ ] 3.3 实现 `ExternalProcess::start()` - 启动外部工具并连接到 PTY
- [ ] 3.4 实现 `ExternalProcess::send_input()` - 发送用户输入
- [ ] 3.5 实现 `ExternalProcess::read_output()` - 读取进程输出（异步流）
- [ ] 3.6 实现 `ExternalProcess::wait()` - 等待进程结束
- [ ] 3.7 实现 `ExternalProcess::is_running()` - 检查进程状态
- [ ] 3.8 实现 `ExternalProcess::terminate()` - 终止进程
- [ ] 3.9 添加进程崩溃和超时的错误处理
- [ ] 3.10 编写进程管理器的集成测试（使用真实的简单进程测试）

## 4. 通用输出结构

- [ ] 4.1 创建 `src/external/output.rs`，定义通用输出结构
- [ ] 4.2 定义 `ToolCall`、`FileOperation`、`ShellOutput` 等结构
- [ ] 4.3 定义 `StructuredOutput` 顶层结构体
- [ ] 4.4 实现 `OutputManager` - 收集和管理解析结果
- [ ] 4.5 实现 JSON 序列化和反序列化
- [ ] 4.6 编写输出结构的单元测试

## 5. Claude Code 解析器

- [ ] 5.1 创建 `src/parsers/claude_code.rs`，实现 ClaudeCodeParser
- [ ] 5.2 实现 `ClaudeCodeParser::parse()` - 主解析入口
- [ ] 5.3 实现工具调用的正则表达式模式匹配
- [ ] 5.4 实现文件操作解析（Read、Write、Edit）
- [ ] 5.5 实现 Shell 命令输出解析
- [ ] 5.6 实现 Task 工具调用解析
- [ ] 5.7 实现进度条和状态更新解析（保留原始 ANSI）
- [ ] 5.8 添加解析失败时的容错处理
- [ ] 5.9 编写解析器的单元测试（使用模拟输出）

## 6. Claude Code Agent 实现

- [ ] 6.1 创建 `src/agent/claude_code.rs`，实现 `ClaudeCodeAgent` 结构体
- [ ] 6.2 实现 `ClaudeCodeAgent::new()` - 创建 Agent 实例
- [ ] 6.3 实现 `ClaudeCodeAgent::chat()` - 发送消息并获取响应
- [ ] 6.4 集成 `external::process::ExternalProcess` 管理进程
- [ ] 6.5 集成 `parsers::claude_code::ClaudeCodeParser` 解析输出
- [ ] 6.6 实现输出捕获和同时显示/解析
- [ ] 6.7 实现会话管理
- [ ] 6.8 编写 ClaudeCodeAgent 的单元测试

## 7. Agent 系统扩展

- [ ] 7.1 在 `src/agent/types.rs` 中添加 `ClaudeCode` Agent 类型
- [ ] 7.2 在 `src/agent/builder.rs` 中实现 `build_claude_code()` 方法
- [ ] 7.3 为 `ClaudeCode` Agent 设计系统提示词
- [ ] 7.4 在 `AgentEnum` 中添加 `ClaudeCode(ClaudeCodeAgent)` 变体
- [ ] 7.5 实现 `AgentEnum::type_name()` 对新类型的支持，返回 "claude-code"
- [ ] 7.6 编写 Agent 构建的单元测试

## 8. CLI 集成

- [ ] 8.1 在 `src/cli/command.rs` 中添加 `/bridge [on|off]` 命令
- [ ] 8.2 在 `src/cli/command.rs` 中添加 `/export <format>` 命令
- [ ] 8.3 实现桥接模式的状态管理
- [ ] 8.4 在 Prompt 中显示当前桥接模式状态
- [ ] 8.5 实现桥接模式和普通模式的切换逻辑
- [ ] 8.6 添加导出功能，支持 JSON 和 Markdown 格式
- [ ] 8.7 更新 `/help` 命令，显示新增的命令
- [ ] 8.8 编写 CLI 命令的集成测试

## 9. 配置管理

- [ ] 9.1 在 `.env.example` 中添加 `CLAUDE_CODE_PATH` 配置项
- [ ] 9.2 在 `.env.example` 中添加 `BRIDGE_ENABLED` 配置项
- [ ] 9.3 在 `.env.example` 中添加 `BRIDGE_OUTPUT_DIR` 配置项
- [ ] 9.4 在 `src/config.rs` 中添加 Bridge 配置结构体
- [ ] 9.5 实现 Claude Code 路径的自动检测逻辑
- [ ] 9.6 实现配置加载和验证
- [ ] 9.7 编写配置管理的单元测试

## 10. 错误处理和恢复

- [ ] 10.1 定义桥接相关的错误类型（使用 `thiserror`）
- [ ] 10.2 实现 Claude Code 未安装时的优雅降级
- [ ] 10.3 实现 PTY 创建失败时的回退逻辑
- [ ] 10.4 实现进程崩溃时的资源清理
- [ ] 10.5 添加详细的错误日志和用户提示
- [ ] 10.6 编写错误处理的测试用例

## 11. 文档

- [ ] 11.1 更新 `README.md`，添加桥接功能说明
- [ ] 11.2 创建 `docs/claude-code-bridge.md`，详细说明桥接功能
- [ ] 11.3 在 `USAGE.md` 中添加 `/bridge` 和 `/export` 命令文档
- [ ] 11.4 更新 `docs/architecture.md`，添加桥接架构图
- [ ] 11.5 添加使用示例和常见问题

## 12. 测试和验证

- [ ] 12.1 在 macOS 上测试完整的桥接流程
- [ ] 12.2 在 Linux 上测试完整的桥接流程
- [ ] 12.3 在 Windows 上测试完整的桥接流程
- [ ] 12.4 测试不同 Claude Code 版本的兼容性
- [ ] 12.5 性能测试，确保解析不影响响应速度
- [ ] 12.6 编写端到端测试，验证完整工作流

## 13. 优化和打磨

- [ ] 13.1 优化正则表达式性能
- [ ] 13.2 优化输出缓冲策略
- [ ] 13.3 添加进度指示器（如适用）
- [ ] 13.4 改进错误消息的用户体验
- [ ] 13.5 添加配置验证和提示
- [ ] 13.6 代码审查和重构

## 依赖关系

- 任务 2 (PTY) 必须在任务 3 (进程管理) 之前完成
- 任务 4 (输出捕获) 必须在任务 5 (解析器) 之前完成
- 任务 5 (解析器) 必须在任务 6 (导出) 之前完成
- 任务 7 (Agent) 可以在任务 1-6 完成后并行开始
- 任务 8 (CLI) 和任务 9 (配置) 可以在任务 7 完成后并行进行
- 任务 10 (错误处理) 应该贯穿整个开发过程
- 任务 11 (文档) 和任务 12 (测试) 在所有功能完成后进行
- 任务 13 (优化) 在所有测试通过后进行

## 可并行的工作项

以下任务可以并行开发：
- 任务 2 (PTY) 和任务 3 的部分内容（进程结构定义）
- 任务 5 (解析器) 和任务 6 (导出) 的结构定义
- 任务 8 (CLI) 和任务 9 (配置) 的部分内容
