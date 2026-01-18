# Change: 初始化 CLI 工具基本功能

## Why
当前代码已经实现了基本的 CLI 功能，但需要优化 UX 交互体验，使其接近 Claude Code 的使用体验。专注于命令行交互体验，建立良好的用户界面反馈机制。

## What Changes
- 优化 CLI 启动和欢迎信息，提供清晰的上下文提示
- 增强交互式对话体验，包括颜色、格式化输出
- 改进命令提示符和输入体验
- 添加基本的斜杠命令系统（如 /exit, /help, /clear）
- 建立配置管理能力（环境变量、API Key）
- 添加测试覆盖（单元测试、集成测试）
- **BREAKING**: 无破坏性变更，仅优化现有体验
- **后续扩展**: 文件系统交互将在后续迭代中添加

## Impact
- Affected specs: 新建 `cli-core`, `config` 两个能力规范（移除 tool-system，后续单独规划）
- Affected code: `src/main.rs`
- Testing: 新增测试模块 `src/tests/`
