## 1. Phase 1: 基础设施实现
- [ ] 1.1 添加依赖到 `Cargo.toml`
  - [ ] 1.1.1 添加 `lsp-types = "0.95"`
  - [ ] 1.1.2 添加 `async-lsp = "0.6"`
  - [ ] 1.1.3 添加 `tokio-process` 或相关进程管理依赖
- [ ] 1.2 创建 `src/lsp/` 模块结构
  - [ ] 1.2.1 创建 `src/lsp/mod.rs` - 模块入口
  - [ ] 1.2.2 创建 `src/lsp/error.rs` - 错误类型定义
- [ ] 1.3 实现 LSP 客户端核心 (`src/lsp/client.rs`)
  - [ ] 1.3.1 定义 `LspClient` 结构体
  - [ ] 1.3.2 实现连接建立（stdio 和 TCP）
  - [ ] 1.3.3 实现 initialize 协议握手
  - [ ] 1.3.4 实现 shutdown 和 exit 协议
  - [ ] 1.3.5 实现基础请求/响应处理
- [ ] 1.4 实现服务器管理器 (`src/lsp/manager.rs`)
  - [ ] 1.4.1 定义 `LspServerManager` 结构体
  - [ ] 1.4.2 实现服务器启动逻辑（进程 spawning）
  - [ ] 1.4.3 实现服务器停止和清理
  - [ ] 1.4.4 实现服务器健康检查
  - [ ] 1.4.5 实现服务器实例池（支持多服务器）
- [ ] 1.5 实现工作区检测 (`src/lsp/workspace.rs`)
  - [ ] 1.5.1 检测项目根目录（查找 package.json, Cargo.toml, go.mod 等）
  - [ ] 1.5.2 根据项目类型确定语言服务器
  - [ ] 1.5.3 实现语言服务器配置加载
- [ ] 1.6 实现 LSP 配置 (`src/lsp/config.rs`)
  - [ ] 1.6.1 定义配置结构体
  - [ ] 1.6.2 实现配置文件加载（TOML 格式）
  - [ ] 1.6.3 实现默认配置
  - [ ] 1.6.4 支持环境变量覆盖
- [ ] 1.7 添加语言服务器映射表
  - [ ] 1.7.1 Rust → rust-analyzer
  - [ ] 1.7.2 TypeScript/JavaScript → typescript-language-server
  - [ ] 1.7.3 Python → pyright
  - [ ] 1.7.4 Go → gopls
  - [ ] 1.7.5 预留扩展接口

## 2. Phase 2: 核心工具实现
- [ ] 2.1 实现 `lsp_edit` 工具 (`src/tools/lsp_edit.rs`)
  - [ ] 2.1.1 定义 `LspEditTool` 结构体
  - [ ] 2.1.2 实现 Tool trait
  - [ ] 2.1.3 定义 `LspEditArgs`（文件路径、range、新文本）
  - [ ] 2.1.4 调用 `textDocument/didChange`
  - [ ] 2.1.5 处理编辑结果和错误
  - [ ] 2.1.6 编写单元测试
- [ ] 2.2 实现 `lsp_diagnostic` 工具 (`src/tools/lsp_diagnostic.rs`)
  - [ ] 2.2.1 定义 `LspDiagnosticTool` 结构体
  - [ ] 2.2.2 实现 Tool trait
  - [ ] 2.2.3 定义 `LspDiagnosticArgs`（文件路径、修复选项）
  - [ ] 2.2.4 调用 `textDocument/publishDiagnostics`
  - [ ] 2.2.5 获取并应用 code actions
  - [ ] 2.2.6 编写单元测试
- [ ] 2.3 注册新工具到 Agent
  - [ ] 2.3.1 在 `src/tools/mod.rs` 中导出新工具
  - [ ] 2.3.2 在 Agent 初始化时注册 LSP 工具
  - [ ] 2.3.3 更新工具定义文档

## 3. Phase 3: 高级功能实现
- [ ] 3.1 实现 `lsp_symbol` 工具 (`src/tools/lsp_symbol.rs`)
  - [ ] 3.1.1 定义 `LspSymbolTool` 结构体
  - [ ] 3.1.2 实现定义查询 (`textDocument/definition`)
  - [ ] 3.1.3 实现引用查询 (`textDocument/references`)
  - [ ] 3.1.4 实现类型定义查询 (`textDocument/typeDefinition`)
  - [ ] 3.1.5 编写单元测试
- [ ] 3.2 实现 `lsp_rename` 工具 (`src/tools/lsp_rename.rs`)
  - [ ] 3.2.1 定义 `LspRenameTool` 结构体
  - [ ] 3.2.2 实现 Tool trait
  - [ ] 3.2.3 定义 `LspRenameArgs`（文件路径、位置、新名称）
  - [ ] 3.2.4 调用 `textDocument/rename`
  - [ ] 3.2.5 应用跨文件修改（workspace edits）
  - [ ] 3.2.6 编写单元测试
- [ ] 3.3 实现 `lsp_format` 工具 (`src/tools/lsp_format.rs`)
  - [ ] 3.3.1 定义 `LspFormatTool` 结构体
  - [ ] 3.3.2 实现文档格式化 (`textDocument/formatting`)
  - [ ] 3.3.3 实现范围格式化 (`textDocument/rangeFormatting`)
  - [ ] 3.3.4 编写单元测试

## 4. Phase 4: 集成和测试
- [ ] 4.1 编写集成测试
  - [ ] 4.1.1 测试 rust-analyzer 集成
  - [ ] 4.1.2 测试 pyright 集成
  - [ ] 4.1.3 测试多文件编辑场景
  - [ ] 4.1.4 测试错误处理和恢复
- [ ] 4.2 添加示例和文档
  - [ ] 4.2.1 创建 LSP 使用示例
  - [ ] 4.2.2 编写 README 章节
  - [ ] 4.2.3 添加配置文件示例
- [ ] 4.3 性能优化
  - [ ] 4.3.1 实现连接池和复用
  - [ ] 4.3.2 添加请求缓存
  - [ ] 4.3.3 实现超时和取消机制
- [ ] 4.4 错误处理增强
  - [ ] 4.4.1 实现优雅降级到文本编辑
  - [ ] 4.4.2 添加详细的错误日志
  - [ ] 4.4.3 提供用户友好的错误提示

## 5. Phase 5: 可选增强
- [ ] 5.1 Windows 支持
  - [ ] 5.1.1 适配 Windows 进程启动
  - [ ] 5.1.2 测试 Windows 语言服务器
- [ ] 5.2 自动安装功能
  - [ ] 5.2.1 提供语言服务器安装脚本
  - [ ] 5.2.2 实现安装检测和提示
- [ ] 5.3 高级 LSP 特性
  - [ ] 5.3.1 代码补全 (`textDocument/completion`)
  - [ ] 5.3.2 代码片段 (`completionItem/resolve`)
  - [ ] 5.3.3 语义高亮 (`textDocument/semanticTokens`)
- [ ] 5.4 交互式配置
  - [ ] 5.4.1 CLI 命令配置 LSP 服务器
  - [ ] 5.4.2 动态启用/禁用 LSP 功能

## 总计
- **Phase 1**: 23 个任务（基础设施）
- **Phase 2**: 11 个任务（核心工具）
- **Phase 3**: 14 个任务（高级功能）
- **Phase 4**: 12 个任务（集成测试）
- **Phase 5**: 12 个任务（可选增强）
- **总计**: 72 个任务

## MVP 定义（最小可行产品）
完成 **Phase 1 + Phase 2**（共 34 个任务）后，Oxide 将具备：
- 基本的 LSP 客户端功能
- 语义编辑能力
- 错误诊断和修复
- 支持 Rust 和至少一种其他语言

这已经能够显著提升代码编辑的智能程度。
