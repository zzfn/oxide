## ADDED Requirements

### Requirement: LSP 客户端核心

Oxide MUST provide a Language Server Protocol client that can communicate with language servers to enable semantic code understanding and editing capabilities.

#### Scenario: 自动检测并启动语言服务器

- **GIVEN** a project directory with `Cargo.toml` (Rust project)
- **WHEN** Oxide needs to perform a code operation
- **THEN** Oxide SHALL automatically detect the project type as Rust
- **AND** Oxide SHALL start the `rust-analyzer` language server
- **AND** Oxide SHALL establish communication via stdio
- **AND** the startup process SHALL complete within 3 seconds

#### Scenario: 连接到已运行的语言服务器

- **GIVEN** a language server is already running on a TCP port
- **AND** configuration is set to `mode = "connect"`
- **WHEN** Oxide needs to perform a code operation
- **THEN** Oxide SHALL connect to the existing server instance
- **AND** Oxide SHALL NOT start a new server process
- **AND** Oxide SHALL reuse the connection for subsequent operations

#### Scenario: 优雅降级到文本编辑

- **GIVEN** language server is not installed or fails to start
- **WHEN** Oxide attempts to perform an LSP operation
- **THEN** Oxide SHALL log a warning message
- **AND** Oxide SHALL fall back to text-based editing
- **AND** Oxide SHALL inform the user about the fallback
- **AND** the operation SHALL still complete successfully

#### Scenario: LSP 服务器生命周期管理

- **GIVEN** Oxide has started a language server
- **WHEN** Oxide shuts down or the operation completes
- **THEN** Oxide SHALL send a `shutdown` request to the server
- **AND** Oxide SHALL send an `exit` notification
- **AND** Oxide SHALL wait for the server process to terminate
- **AND** Oxide SHALL clean up all resources (stdio handles, temp files)

---

### Requirement: 多语言服务器支持

The LSP client MUST support multiple language servers and automatically select the appropriate server based on the project type.

#### Scenario: Rust 项目支持

- **GIVEN** a project directory with `Cargo.toml` or `.rs` files
- **WHEN** Oxide detects a Rust project
- **THEN** Oxide SHALL use `rust-analyzer` as the language server
- **AND** Oxide SHALL configure the server with appropriate initialization options
- **AND** Oxide SHALL support Rust-specific features (macros, traits, lifetimes)

#### Scenario: TypeScript/JavaScript 项目支持

- **GIVEN** a project directory with `package.json` and `.ts` or `.tsx` files
- **WHEN** Oxide detects a TypeScript project
- **THEN** Oxide SHALL use `typescript-language-server` as the language server
- **AND** Oxide SHALL support JSX and TSX syntax
- **AND** Oxide SHALL respect `tsconfig.json` configuration

#### Scenario: Python 项目支持

- **GIVEN** a project directory with `.py` files or `pyproject.toml`
- **WHEN** Oxide detects a Python project
- **THEN** Oxide SHALL use `pyright` as the language server
- **AND** Oxide SHALL support Python type hints
- **AND** Oxide SHALL respect virtual environment configuration

#### Scenario: 扩展新语言服务器

- **GIVEN** a new language server is added to the configuration
- **WHEN** Oxide encounters a project of that language
- **THEN** Oxide SHALL load the server configuration from the config file
- **AND** Oxide SHALL start the server with the specified command and arguments
- **AND** Oxide SHALL use the server for code operations

---

### Requirement: LSP 协议实现

The LSP client MUST implement the core LSP protocol including initialization, document synchronization, and text document operations.

#### Scenario: 初始化握手

- **GIVEN** a language server process is started
- **WHEN** Oxide sends an `initialize` request
- **THEN** Oxide SHALL include client capabilities in the request
- **AND** Oxide SHALL specify the workspace root URI
- **AND** Oxide SHALL wait for the server's `InitializeResult`
- **AND** Oxide SHALL store the server's capabilities for later use
- **AND** Oxide SHALL send an `initialized` notification after successful initialization

#### Scenario: 文档同步

- **GIVEN** a file is opened or modified
- **WHEN** Oxide needs to sync the document state
- **THEN** Oxide SHALL send a `textDocument/didOpen` notification with the document's URI and content
- **AND** Oxide SHALL send `textDocument/didChange` notifications for subsequent edits
- **AND** Oxide SHALL include the correct version number in change notifications
- **AND** Oxide SHALL use full document sync (incremental sync is optional)

#### Scenario: 服务器请求超时处理

- **GIVEN** a request is sent to the language server
- **AND** the server does not respond within the timeout period (default 5 seconds)
- **WHEN** the timeout occurs
- **THEN** Oxide SHALL cancel the request
- **AND** Oxide SHALL log a timeout error
- **AND** Oxide SHALL fall back to text-based editing
- **AND** Oxide SHALL mark the server as potentially unhealthy

---

### Requirement: LSP 配置管理

Oxide MUST provide a flexible configuration system for LSP servers, supporting project-level and global-level configuration.

#### Scenario: 全局配置加载

- **GIVEN** a global configuration file at `~/.oxide/lsp.toml`
- **AND** the file contains default server settings
- **WHEN** Oxide starts
- **THEN** Oxide SHALL load the global configuration
- **AND** Oxide SHALL use these settings as defaults for all projects

#### Scenario: 项目配置覆盖

- **GIVEN** a project configuration file at `.oxide/lsp.toml`
- **AND** the project config overrides global settings
- **WHEN** Oxide operates in that project
- **THEN** Oxide SHALL load both global and project configurations
- **AND** Oxide SHALL merge the configurations with project config taking precedence
- **AND** Oxide SHALL use the merged configuration for that project

#### Scenario: 环境变量配置

- **GIVEN** an environment variable `OXIDE_LSP_AUTO_START=false`
- **WHEN** Oxide attempts to auto-start an LSP server
- **THEN** Oxide SHALL read the environment variable
- **AND** Oxide SHALL skip auto-starting the server
- **AND** Oxide SHALL attempt to connect to an existing server if configured

#### Scenario: 配置验证

- **GIVEN** a configuration file with invalid settings (e.g., non-existent command)
- **WHEN** Oxide loads the configuration
- **THEN** Oxide SHALL validate the configuration structure
- **AND** Oxide SHALL report a clear error message for invalid settings
- **AND** Oxide SHALL continue with default or fallback configuration
