# external-tool Specification

此规范描述 Oxide 与外部 CLI 工具（如 Claude Code、Cursor、Copilot 等）的通用集成基础设施。

## ADDED Requirements

### Requirement: 通用 PTY 管理
系统 SHALL 提供跨平台的伪终端（PTY）管理功能，用于与外部工具交互。

#### Scenario: 创建虚拟终端
- **WHEN** 外部工具需要启动
- **THEN** 系统 SHALL 使用 `portable-pty` 创建虚拟终端
- **AND** PTY SHALL 支持读写操作
- **AND** PTY SHALL 支持终端大小调整

#### Scenario: 跨平台支持
- **WHEN** 在不同操作系统上创建 PTY
- **THEN** 系统 SHALL 在 macOS 上使用原生 PTY
- **AND** 系统 SHALL 在 Linux 上使用原生 PTY
- **AND** 系统 SHALL 在 Windows 上使用 ConPTY 或 WinPTY

#### Scenario: PTY 清理
- **WHEN** 外部工具进程结束或发生错误
- **THEN** 系统 SHALL 正确关闭 PTY
- **AND** 系统 SHALL 释放所有相关资源
- **AND** 系统 SHALL 不泄漏文件描述符

### Requirement: 通用进程管理
系统 SHALL 提供通用的外部进程管理功能，支持启动、控制和监控外部工具。

#### Scenario: 启动外部进程
- **WHEN** 需要启动外部工具（如 Claude Code）
- **THEN** 系统 SHALL 使用 `tokio::process::Command` 启动进程
- **AND** 系统 SHALL 将进程的 stdin/stdout 连接到 PTY
- **AND** 系统 SHALL 支持传递环境变量和工作目录

#### Scenario: 发送输入到进程
- **WHEN** 需要向外部工具发送输入
- **THEN** 系统 SHALL 通过 PTY 写入句柄发送数据
- **AND** 系统 SHALL 支持发送多行输入
- **AND** 系统 SHALL 正确处理特殊字符（如 Ctrl+C）

#### Scenario: 读取进程输出
- **WHEN** 外部工具产生输出
- **THEN** 系统 SHALL 异步读取进程的 stdout 和 stderr
- **AND** 系统 SHALL 保留原始的 ANSI 转义字符
- **AND** 系统 SHALL 支持流式读取，不等待进程结束

#### Scenario: 进程状态监控
- **WHEN** 外部工具进程正在运行
- **THEN** 系统 SHALL 能够检查进程是否仍在运行
- **AND** 系统 SHALL 能够获取进程的退出状态码
- **AND** 系统 SHALL 能够检测进程崩溃

#### Scenario: 终止进程
- **WHEN** 需要停止外部工具（用户请求或超时）
- **THEN** 系统 SHALL 优雅地终止进程（发送 SIGTERM）
- **AND** 如果进程未响应，系统 SHALL 强制终止（SIGKILL）
- **AND** 系统 SHALL 清理所有相关资源

### Requirement: 通用输出结构
系统 SHALL 定义通用的结构化输出格式，用于表示外部工具的执行结果。

#### Scenario: 工具调用结果结构
- **WHEN** 记录工具调用结果
- **THEN** 系统 SHALL 使用统一的 `ToolCall` 结构
- **AND** 结构 SHALL 包含工具名称、参数、结果、时间戳
- **AND** 结构 SHALL 支持 JSON 序列化

#### Scenario: 文件操作结构
- **WHEN** 记录文件操作
- **THEN** 系统 SHALL 使用统一的 `FileOperation` 结构
- **AND** 结构 SHALL 包含操作类型、文件路径、变更内容
- **AND** 结构 SHALL 支持 JSON 序列化

#### Scenario: Shell 命令结构
- **WHEN** 记录 Shell 命令执行
- **THEN** 系统 SHALL 使用统一的 `ShellCommand` 结构
- **AND** 结构 SHALL 包含命令字符串、退出码、输出
- **AND** 结构 SHALL 支持 JSON 序列化

#### Scenario: 顶层输出结构
- **WHEN** 生成完整的结构化输出
- **THEN** 系统 SHALL 使用 `StructuredOutput` 结构
- **AND** 结构 SHALL 包含版本、时间戳、会话 ID
- **AND** 结构 SHALL 包含工具调用、文件操作、Shell 命令数组
- **AND** 结构 SHALL 可选包含原始 ANSI 输出（base64 编码）

### Requirement: 输出管理器
系统 SHALL 提供输出管理功能，用于收集、序列化和导出结构化数据。

#### Scenario: 收集解析结果
- **WHEN** 解析器产生结构化数据
- **THEN** 系统 SHALL 将数据添加到 `StructuredOutput`
- **AND** 系统 SHALL 维护工具调用、文件操作、命令的列表
- **AND** 系统 SHALL 按时间顺序排序结果

#### Scenario: 生成 JSON
- **WHEN** 需要导出结构化输出
- **THEN** 系统 SHALL 将 `StructuredOutput` 序列化为 JSON 字符串
- **AND** JSON SHALL 符合预定义的 Schema
- **AND** JSON SHALL 包含所有必要的字段

#### Scenario: 保存到文件
- **WHEN** 用户请求导出或会话结束时
- **THEN** 系统 SHALL 将 JSON 保存到文件
- **AND** 文件名 SHALL 包含时间戳和会话 ID
- **AND** 文件 SHALL 保存到配置的输出目录
- **AND** 系统 SHALL 自动创建不存在的目录

### Requirement: 错误处理
系统 SHALL 提供健壮的错误处理机制，确保外部工具失败不影响 Oxide 核心功能。

#### Scenario: PTY 创建失败
- **WHEN** PTY 创建失败（权限不足、平台不支持）
- **THEN** 系统 SHALL 记录错误日志
- **AND** 系统 SHALL 返回清晰的错误消息
- **AND** 系统 SHALL 不影响 Oxide 的其他功能

#### Scenario: 进程启动失败
- **WHEN** 外部工具启动失败（未安装、路径错误）
- **THEN** 系统 SHALL 捕获启动错误
- **AND** 系统 SHALL 提供诊断信息（检查的路径、错误原因）
- **AND** 系统 SHALL 建议解决方案

#### Scenario: 进程意外终止
- **WHEN** 外部工具进程崩溃
- **THEN** 系统 SHALL 检测到进程结束
- **AND** 系统 SHALL 清理 PTY 和文件句柄
- **AND** 系统 SHALL 记录崩溃信息（退出码、核心转储路径）
- **AND** Oxide CLI SHALL 继续运行

#### Scenario: 输出读取错误
- **WHEN** 读取进程输出时发生错误
- **THEN** 系统 SHALL 记录错误但不中断
- **AND** 系统 SHALL 尝试恢复读取
- **AND** 系统 SHALL 保留已读取的数据

### Requirement: 可扩展性
基础设施 SHALL 设计为可扩展，便于添加新的外部工具支持。

#### Scenario: 添加新工具
- **WHEN** 需要添加新的外部工具（如 Cursor）
- **THEN** 系统 SHALL 能够复用 `external::process` 模块
- **AND** 系统 SHALL 能够复用 `external::pty` 模块
- **AND** 系统 SHALL 能够复用 `external::output` 结构
- **AND** 只需实现工具特定的解析器（在 `parsers/` 模块）

#### Scenario: 工具特定配置
- **WHEN** 不同工具需要不同配置
- **THEN** 系统 SHALL 支持为每个工具定义配置结构
- **AND** 系统 SHALL 支持从配置文件加载工具特定设置
- **AND** 系统 SHALL 支持环境变量覆盖

### Requirement: 性能要求
基础设施 SHALL 满足一定的性能要求，确保用户体验。

#### Scenario: 异步操作
- **WHEN** 执行进程管理、PTY 操作、输出读取
- **THEN** 所有操作 SHALL 是异步的
- **AND** 主线程 SHALL 不被阻塞
- **AND** 用户 SHALL 能够继续与 CLI 交互

#### Scenario: 内存管理
- **WHEN** 外部工具产生大量输出
- **THEN** 系统 SHALL 使用流式处理，避免一次性加载所有数据
- **AND** 系统 SHALL 限制内存缓冲区大小
- **AND** 系统 SHALL 将结果及时写入文件

#### Scenario: 资源清理
- **WHEN** 进程结束或发生错误
- **THEN** 系统 SHALL 立即释放 PTY 和文件句柄
- **AND** 系统 SHALL 不泄漏内存
- **AND** 系统 SHALL 支持长时间运行而不累积资源
