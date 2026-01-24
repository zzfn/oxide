## ADDED Requirements

### Requirement: 语义感知代码编辑

Oxide SHALL provide semantic code editing capabilities using LSP, allowing precise modifications based on code structure rather than text positions.

#### Scenario: 基于范围的精确编辑

- **GIVEN** a source file with multiple functions
- **AND** the language server is available
- **WHEN** Oxide edits a specific function using `lsp_edit`
- **THEN** Oxide SHALL identify the function's AST range using LSP
- **AND** Oxide SHALL apply the edit only within that range
- **AND** Oxide SHALL preserve the function's signature and surrounding code
- **AND** Oxide SHALL verify the edit does not introduce syntax errors

#### Scenario: 跨文件重构编辑

- **GIVEN** a function used in multiple files
- **AND** the language server provides workspace-wide information
- **WHEN** Oxide modifies the function signature
- **THEN** Oxide SHALL use LSP to find all references across files
- **AND** Oxide SHALL apply consistent edits to all usage sites
- **AND** Oxide SHALL report the total number of files modified
- **AND** Oxide SHALL ensure all edits are syntactically correct

#### Scenario: 错误恢复和降级

- **GIVEN** an `lsp_edit` operation fails due to LSP unavailability
- **WHEN** the failure occurs
- **THEN** Oxide SHALL automatically fall back to `edit_file` (text-based)
- **AND** Oxide SHALL inform the user about the fallback
- **AND** Oxide SHALL attempt the text-based edit with the same parameters
- **AND** Oxide SHALL continue processing without user intervention

---

### Requirement: 实时代码诊断

Oxide SHALL provide real-time code diagnostics using LSP, including errors, warnings, and hints from the language server.

#### Scenario: 获取文件诊断信息

- **GIVEN** a source file with errors and warnings
- **AND** the language server is running
- **WHEN** Oxide runs `lsp_diagnostic` on the file
- **THEN** Oxide SHALL request diagnostics from the language server
- **AND** Oxide SHALL receive all diagnostics (errors, warnings, hints, info)
- **AND** Oxide SHALL display each diagnostic with:
  - File path and line number
  - Severity level (error/warning/hint/info)
  - Diagnostic message
  - Suggested fix (if available from code actions)

#### Scenario: 自动应用诊断修复

- **GIVEN** a file with fixable errors (e.g., missing import)
- **AND** the language server provides code actions
- **WHEN** Oxide runs `lsp_diagnostic` with `fix_errors=true`
- **THEN** Oxide SHALL request code actions for each diagnostic
- **AND** Oxide SHALL apply the available fixes automatically
- **AND** Oxide SHALL verify the fixes resolve the diagnostics
- **AND** Oxide SHALL report the number of fixes applied

#### Scenario: 诊断信息格式化输出

- **GIVEN** multiple diagnostics from the language server
- **WHEN** Oxide displays the diagnostics to the user
- **THEN** Oxide SHALL format the output with:
  - Color coding (red for errors, yellow for warnings, blue for hints)
  - Grouping by file
  - Sorted by severity and line number
  - Clear separation between diagnostics
- **AND** Oxide SHALL provide a summary (e.g., "3 errors, 5 warnings")

---

### Requirement: 符号查询和导航

Oxide SHALL provide symbol query capabilities using LSP, enabling navigation to definitions, references, and type definitions.

#### Scenario: 跳转到定义

- **GIVEN** a function call in the code
- **AND** the function is defined in another file
- **WHEN** Oxide queries the definition using `lsp_symbol`
- **THEN** Oxide SHALL send a `textDocument/definition` request
- **AND** Oxide SHALL receive the definition location (file, line, column)
- **AND** Oxide SHALL display the definition to the user
- **AND** Oxide SHALL show the relevant code snippet

#### Scenario: 查找所有引用

- **GIVEN** a function or variable in the code
- **WHEN** Oxide queries all references using `lsp_symbol`
- **THEN** Oxide SHALL send a `textDocument/references` request
- **AND** Oxide SHALL receive all reference locations
- **AND** Oxide SHALL display each reference with file path and line number
- **AND** Oxide SHALL group references by file
- **AND** Oxide SHALL provide a count of total references

#### Scenario: 类型定义查询

- **GIVEN** a variable or expression in the code
- **WHEN** Oxide queries the type definition using `lsp_symbol`
- **THEN** Oxide SHALL send a `textDocument/typeDefinition` request
- **AND** Oxide SHALL receive the type's definition location
- **AND** Oxide SHALL display the type definition to the user
- **AND** Oxide SHALL include type information in the output

---

### Requirement: 语义重命名

Oxide SHALL provide semantic rename capabilities using LSP, enabling cross-file renaming of symbols while maintaining correctness.

#### Scenario: 单文件重命名

- **GIVEN** a function named `process_data` in a file
- **AND** the function is called multiple times in the same file
- **WHEN** Oxide renames it to `handle_data` using `lsp_rename`
- **THEN** Oxide SHALL send a `textDocument/rename` request
- **AND** Oxide SHALL apply the workspace edits from the language server
- **AND** Oxide SHALL update the function definition
- **AND** Oxide SHALL update all call sites in the file
- **AND** Oxide SHALL ensure no other symbols are affected

#### Scenario: 跨文件重命名

- **GIVEN** a public function used across multiple files
- **WHEN** Oxide renames the function using `lsp_rename`
- **THEN** Oxide SHALL receive workspace edits for all affected files
- **AND** Oxide SHALL apply edits to all files atomically
- **AND** Oxide SHALL report the total number of files modified
- **AND** Oxide SHALL verify the rename is complete and consistent

#### Scenario: 重命名验证和确认

- **GIVEN** a rename operation that affects multiple files
- **WHEN** Oxide receives the workspace edits
- **THEN** Oxide SHALL display a preview of changes to the user
- **AND** Oxide SHALL request user confirmation before applying
- **AND** Oxide SHALL allow the user to cancel the operation
- **AND** Oxide SHALL apply the edits only after confirmation

---

### Requirement: 代码格式化

Oxide SHALL provide code formatting capabilities using LSP, ensuring consistent code style according to language server rules.

#### Scenario: 格式化整个文档

- **GIVEN** a source file with inconsistent formatting
- **AND** the language server supports formatting
- **WHEN** Oxide runs `lsp_format` on the file
- **THEN** Oxide SHALL send a `textDocument/formatting` request
- **AND** Oxide SHALL receive the text edits from the server
- **AND** Oxide SHALL apply all edits to the file
- **AND** Oxide SHALL preserve the file's semantic content
- **AND** Oxide SHALL report the number of formatting changes

#### Scenario: 格式化选定范围

- **GIVEN** a source file
- **AND** a specific range (e.g., a function) is selected
- **WHEN** Oxide runs `lsp_format` with the range
- **THEN** Oxide SHALL send a `textDocument/rangeFormatting` request
- **AND** Oxide SHALL apply edits only within the specified range
- **AND** Oxide SHALL leave code outside the range unchanged
- **AND** Oxide SHALL verify the formatted code is syntactically correct

#### Scenario: 格式化配置

- **GIVEN** a project with formatting configuration (e.g., `.editorconfig`, `rustfmt.toml`)
- **WHEN** Oxide formats a file in that project
- **THEN** Oxide SHALL respect the project's formatting rules
- **AND** Oxide SHALL pass the configuration to the language server
- **AND** Oxide SHALL apply the formatting according to the configuration
- **AND** Oxide SHALL not override user-configured style preferences

---

### Requirement: 工具集成和互操作

Oxide SHALL integrate LSP tools with existing tools, providing a unified interface for code operations.

#### Scenario: LSP 和文本工具的协调

- **GIVEN** both `lsp_edit` and `edit_file` tools are available
- **WHEN** the AI assistant decides which tool to use
- **THEN** Oxide SHALL prefer `lsp_edit` when LSP is available and appropriate
- **AND** Oxide SHALL fall back to `edit_file` when LSP is unavailable
- **AND** Oxide SHALL provide consistent error handling across both tools
- **AND** Oxide SHALL ensure both tools produce compatible results

#### Scenario: LSP 工具的错误处理

- **GIVEN** an LSP operation encounters an error (e.g., server crash)
- **WHEN** the error occurs
- **THEN** Oxide SHALL log detailed error information
- **AND** Oxide SHALL attempt to restart the LSP server if needed
- **AND** Oxide SHALL fall back to text-based editing if restart fails
- **AND** Oxide SHALL inform the user about the error and recovery action

#### Scenario: LSP 工具的性能优化

- **GIVEN** multiple sequential LSP operations on the same file
- **WHEN** Oxide performs these operations
- **THEN** Oxide SHALL reuse the LSP client connection
- **AND** Oxide SHALL cache document state between operations
- **AND** Oxide SHALL batch operations when possible
- **AND** Oxide SHALL complete the operations with minimal latency (< 500ms per operation)
