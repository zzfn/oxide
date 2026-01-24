# Change: Add LSP Client Integration

## Why

当前 Oxide 使用基于文本 diff 的方式编辑代码（`edit_file` 工具），这种方式存在以下限制：

1. **缺乏语义理解** - 无法理解代码结构（函数、类、作用域），只能进行基于文本的替换
2. **语言单一** - 只能处理文本，无法利用特定语言的语言服务器功能
3. **无错误感知** - 无法获取实时诊断信息（错误、警告、提示）
4. **重构能力弱** - 无法执行跨文件重命名、提取函数等语义操作

通过集成 LSP（Language Server Protocol）客户端，Oxide 将能够：
- 与 20+ 语言的语言服务器通信（rust-analyzer, pyright, gopls 等）
- 基于代码语义进行精确编辑
- 实时获取并修复代码错误
- 执行跨文件重构操作

## What Changes

### 新增功能
- **LSP 客户端系统**
  - 自动检测项目类型并启动对应的语言服务器
  - 支持连接已运行的语言服务器实例（混合模式）
  - 管理语言服务器生命周期（启动、停止、重启）

- **LSP 工具集**
  - `lsp_edit` - 基于语义位置的代码编辑
  - `lsp_diagnostic` - 获取并修复代码诊断信息
  - `lsp_symbol` - 符号查询（定义、引用、类型定义）
  - `lsp_rename` - 跨文件重命名重构
  - `lsp_format` - 代码格式化

- **多语言支持**
  - Rust (rust-analyzer)
  - TypeScript/JavaScript (typescript-language-server, vls)
  - Python (pyright, pylsp)
  - Go (gopls)
  - 其他 LSP 兼容的语言服务器

### 技术实现
- 新增 `src/lsp/` 模块
- 添加依赖：`lsp-types`, `async-lsp`
- 保持向后兼容：现有 `edit_file` 工具继续可用

## Impact

### Affected specs
- `lsp-client` (NEW) - LSP 客户端核心能力
- `code-editing` (MODIFIED) - 扩展代码编辑能力，增加语义编辑选项

### Affected code
- **新增**：`src/lsp/` 目录（约 1000-1500 行新代码）
  - `client.rs` - LSP 客户端核心
  - `manager.rs` - 服务器管理器
  - `workspace.rs` - 工作区检测
  - `config.rs` - LSP 配置

- **新增**：`src/tools/lsp_*.rs`（约 600-800 行）
  - 5 个新的 LSP 工具

- **修改**：`Cargo.toml` - 添加 LSP 相关依赖

- **可选修改**：`src/tools/mod.rs` - 导出新的 LSP 工具

### User-facing changes
- 用户可以使用自然语言请求语义级别的代码操作
- AI 助手能够基于语言服务器提供更准确的代码修改
- 支持更多编程语言的智能编辑
- 更好的错误检测和修复能力

### Performance impact
- 启动语言服务器会增加初始开销（约 1-3 秒）
- 后续操作通过 IPC 通信，延迟可接受（< 100ms）
- 可配置是否自动启动或连接现有服务器

## Migration Plan

### Phase 1: 基础设施（必需）
- 实现 LSP 客户端框架
- 服务器管理器
- 基础通信协议

### Phase 2: 核心工具（必需）
- `lsp_edit` - 语义编辑
- `lsp_diagnostic` - 错误修复

### Phase 3: 高级功能（可选）
- 符号查询和导航
- 重命名和重构
- 代码格式化

### Backward Compatibility
- 现有 `edit_file` 工具保持不变
- LSP 工具为可选增强功能
- 用户可选择性启用 LSP 功能

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| LSP 服务器启动失败 | 无法使用语义编辑 | 降级到文本编辑，记录错误日志 |
| 语言服务器不兼容 | 某些语言不可用 | 提供兼容性测试列表，标记支持状态 |
| 性能问题 | 操作延迟增加 | 实现连接池、请求缓存、超时机制 |
| 协议版本不匹配 | 通信失败 | 支持多个 LSP 版本，优雅降级 |

## Open Questions

1. **是否需要支持 Windows 的语言服务器启动？**
   - 初期可以优先支持 Unix-like 系统（Linux/macOS）
   - Windows 支持可以作为 Phase 4

2. **如何处理用户未安装语言服务器的情况？**
   - 选项 A：自动下载安装（复杂，有安全风险）
   - 选项 B：提示用户手动安装（推荐，更安全）
   - 选项 C：提供一键安装脚本（折中方案）

3. **LSP 配置应该放在哪里？**
   - 项目级别：`.oxide/lsp.toml`
   - 全局级别：`~/.oxide/lsp.toml`
   - 建议：两者都支持，项目配置优先
