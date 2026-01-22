# Tasks: Claude Code 功能对等

## Phase 1: 核心基础设施

### 1.1 配置系统重构
- [x] 创建 `src/config/loader.rs` 模块
- [x] 实现配置层次结构（全局/项目/会话）
- [x] 添加 `toml` 依赖
- [x] 实现 `CONFIG.md` 项目指令解析
- [x] 实现 `.oxide/config.toml` 加载
- [x] 实现 `~/.oxide/config.toml` 全局配置
- [x] 环境变量覆盖逻辑
- [x] 配置验证和错误处理
- [x] 编写配置系统测试

### 1.2 Agent 类型定义
- [x] 创建 `src/agent/types.rs` 定义 AgentType 枚举
- [x] 创建 `AgentCapability` 结构体
- [x] 为每个 Agent 类型定义能力描述
- [x] 定义 Agent 系统提示词模板
- [x] 定义 Agent 工具权限映射

### 1.3 Task Manager 基础
- [x] 创建 `src/task/` 模块目录
- [x] 实现 `TaskManager` 结构体
- [x] 实现 `Task` 状态管理
- [x] 实现任务持久化存储
- [x] 实现任务 ID 生成器
- [x] 实现任务状态查询 API
- [x] 编写 Task Manager 测试

### 1.4 Glob 工具
- [x] 创建 `src/tools/glob.rs`
- [x] 添加 `glob` 依赖
- [x] 实现 `GlobTool` 结构体
- [x] 实现 `GlobInput` 反序列化
- [x] 实现文件模式匹配逻辑
- [x] 集成到 Agent 工具链
- [x] 编写 Glob 工具测试

## Phase 2: Agent System

### 2.1 Subagent Manager
- [x] 创建 `src/agent/subagent.rs`
- [x] 实现 `SubagentManager` 结构体
- [x] 实现 Agent 注册机制
- [x] 实现 Agent 切换逻辑
- [x] 实现 Agent 能力查询
- [x] 实现 Agent 生命周期管理
- [x] 编写 Subagent Manager 测试

### 2.2 Explore Agent
- [x] 实现 Explore Agent 提示词
- [x] 配置 Explore Agent 工具权限（只读）
- [x] 实现 Explore Agent 创建逻辑
- [x] 实现 Explore Agent 特定行为
- [x] 集成到 Subagent Manager
- [x] 测试 Explore Agent 功能

### 2.3 Plan Agent
- [x] 实现 Plan Agent 提示词
- [x] 配置 Plan Agent 工具权限
- [x] 实现 Plan Agent 创建逻辑
- [x] 实现计划生成和验证
- [x] 集成到 Subagent Manager
- [x] 测试 Plan Agent 功能

### 2.4 Code Reviewer Agent
- [x] 实现 Code Reviewer Agent 提示词
- [x] 配置 Code Reviewer 工具权限（只读）
- [x] 实现 Code Reviewer 创建逻辑
- [x] 实现代码审查逻辑
- [x] 集成到 Subagent Manager
- [x] 测试 Code Reviewer 功能

### 2.5 Task 工具集成
- [x] 创建 `src/tools/task.rs`
- [x] 实现 `TaskTool` 工具
- [x] 实现后台任务执行逻辑
- [x] 实现任务输出捕获
- [x] 实现 TaskOutput 工具
- [x] 集成到主 Agent
- [x] 编写 Task 工具测试

### 2.6 /agent 命令
- [x] 实现 `/agent` 斜杠命令
- [x] 实现 `/agent list` 子命令
- [x] 实现 `/agent switch <type>` 子命令
- [x] 实现 `/agent capabilities` 子命令
- [x] 添加命令帮助和补全
- [x] 测试 /agent 命令功能

## Phase 3: Advanced Tools

### 3.1 MultiEdit 工具
- [x] 创建 `src/tools/multiedit.rs`
- [x] 实现 `MultiEditInput` 结构体
- [x] 实现批量编辑逻辑
- [x] 实现编辑原子性保证
- [x] 集成到 Agent 工具链
- [x] 编写 MultiEdit 测试

### 3.2 AskUserQuestion 工具
- [x] 创建 `src/tools/ask_user_question.rs`
- [x] 实现 `QuestionInput` 结构体
- [x] 实现交互式选择器 UI（CLI 模式）
- [x] 实现交互式选择器 UI（TUI 模式）
- [x] 实现答案收集和返回
- [x] 集成到 Agent 工具链
- [x] 编写 AskUserQuestion 测试

### 3.3 NotebookEdit 工具
- [x] 创建 `src/tools/notebook_edit.rs`
- [x] 添加 Jupyter notebook 解析依赖(使用 serde_json)
- [x] 实现 `NotebookEditInput` 结构体
- [x] 实现 notebook 单元编辑逻辑
- [x] 实现 notebook 序列化
- [x] 集成到 Agent 工具链
- [x] 编写 NotebookEdit 测试

### 3.4 Git 增强
- [x] 创建 `src/tools/git_guard.rs`
- [x] 添加 `git2` 依赖
- [x] 实现 `GitGuard` 结构体
- [x] 实现分支检测逻辑
- [x] 实现远程状态检查
- [x] 实现主分支推送警告
- [x] 集成到 shell_execute 工具
- [x] 编写 Git 安全检查测试

### 3.5 Commit 规范验证
- [x] 创建 `src/tools/commit_linter.rs`
- [x] 实现 `CommitLinter` 结构体
- [x] 实现 Conventional Commits 正则验证
- [x] 实现 commit 消息格式化
- [x] 集成到 Git 工作流
- [x] 编写 Commit Linter 测试

## Phase 4: TUI 完善

### 4.1 完成未完成的 TUI 任务
- [x] 阶段 3: 实现增量 Markdown 解析器
- [x] 阶段 6: 主题系统实现
- [x] 阶段 8: 交互功能增强（帮助面板、历史管理基础）
- [ ] 阶段 9: 性能优化
- [ ] 阶段 10: 测试和文档

### 4.2 主题系统
- [x] 创建 `src/tui/theme.rs` 模块
- [x] 实现内置主题（dark/light/high_contrast）
- [x] 实现主题配置加载
- [x] 实现主题热切换（通过 `App::set_theme`）
- [ ] 添加 `/theme` 命令
- [ ] 创建主题配置示例

### 4.3 TUI 交互增强
- [x] 实现快捷键帮助屏幕（`?` 键）
- [x] 实现命令历史管理（`App::add_to_history` 等）
- [x] 实现上下键浏览历史（`↑/↓` 在输入框为空时）
- [x] 实现历史快捷键（`Ctrl+P/N`）
- [ ] 实现历史搜索（`Ctrl+R`）
- [ ] 实现多行输入模式（`Ctrl+G`）
- [ ] 实现消息搜索（`/` 键）

### 4.4 TUI 性能优化
- [x] 实现虚拟滚动
- [x] 添加渲染缓存
- [x] 实现增量渲染（通过 dirty 标志）
- [ ] 限制消息历史大小
- [ ] 添加性能监控

## Phase 5: 斜杠命令增强

### 5.1 /tasks 命令
- [x] 实现 `/tasks list` 子命令
- [x] 实现 `/tasks show <id>` 子命令
- [x] 实现 `/tasks cancel <id>` 子命令
- [x] 添加任务状态显示
- [x] 添加命令帮助

### 5.2 /config 命令增强
- [x] 实现 `/config show` 子命令
- [x] 实现 `/config edit` 子命令
- [x] 实现 `/config reload` 子命令
- [x] 添加配置验证命令

### 5.3 /help 命令增强
- [x] 显示所有 Agent 类型
- [x] 显示所有工具列表
- [x] 显示所有斜杠命令
- [x] 添加使用示例

## Phase 6: 测试

### 6.1 单元测试
- [x] Agent 类型测试
- [x] Agent 能力测试
- [x] Task Manager 测试
- [x] 所有工具测试
- [x] 配置加载测试
- [x] Git 安全检查测试
- [x] TUI 主题系统测试
- [x] TUI Markdown 解析器测试
- [x] TUI 渲染缓存和虚拟滚动测试

### 6.2 集成测试
- [x] 主题切换测试
- [x] Markdown 解析集成测试
- [ ] 完整对话流程测试
- [ ] Agent 切换测试
- [ ] 任务创建和执行测试
- [ ] 多 Agent 协作测试
- [ ] 配置优先级测试

### 6.3 手动测试
- [ ] 用户体验测试
- [ ] 性能基准测试
- [ ] 错误恢复测试
- [ ] 兼容性测试

## Phase 7: 文档

### 7.1 用户文档
- [ ] 更新 README.md
- [ ] 创建 Agent 系统使用指南
- [ ] 创建配置系统文档
- [ ] 创建工具参考文档
- [ ] 创建故障排除指南

### 7.2 开发文档
- [ ] 更新架构设计文档
- [ ] 创建 Agent 开发指南
- [ ] 创建工具开发指南
- [ ] 代码注释完善

### 7.3 API 文档
- [ ] 生成 rustdoc
- [ ] 添加示例代码
- [ ] 创建模块文档

## Phase 8: 可选功能

### 8.1 MCP 支持（可选）
- [ ] 创建 `src/mcp/` 模块
- [ ] 实现 MCP 客户端
- [ ] 实现 MCP 工具包装器
- [ ] 实现 MCP 配置加载
- [ ] 添加 `/mcp` 命令
- [ ] 编写 MCP 测试

### 8.2 多模态支持（可选）
- [ ] 添加 `image` 依赖
- [ ] 扩展 Read 工具支持图片
- [ ] 实现 AnalyzeImage 工具
- [ ] 实现图片显示（TUI）
- [ ] 编写多模态测试

### 8.3 Web 工具（可选）
- [ ] 创建 `src/tools/web.rs`
- [ ] 实现 WebFetch 工具
- [ ] 实现 WebSearch 工具
- [ ] 添加 HTML 转 Markdown
- [ ] 编写 Web 工具测试

## Phase 9: 发布

### 9.1 发布准备
- [ ] 创建 CHANGELOG
- [ ] 更新版本号
- [ ] 创建 Git tag
- [ ] 发布到 crates.io

### 9.2 发布后
- [ ] 监控用户反馈
- [ ] 修复紧急 Bug
- [ ] 收集功能请求
- [ ] 规划下一版本
