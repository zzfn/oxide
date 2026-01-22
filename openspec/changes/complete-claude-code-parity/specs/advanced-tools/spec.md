# advanced-tools Specification

## Purpose
定义 Oxide 的高级工具集，扩展基础文件操作能力，提供更强大的代码操作和用户交互功能。

## ADDED Requirements

### Requirement: Glob 工具
系统 SHALL 提供 Glob 工具用于文件模式匹配。

#### Scenario: 基础模式匹配
- **GIVEN** 用户请求查找所有 Rust 文件
- **WHEN** 调用 Glob 工具，pattern 为 "**/*.rs"
- **THEN** 返回所有匹配的文件路径
- **AND** 路径按修改时间排序

#### Scenario: 指定目录搜索
- **GIVEN** 用户限制搜索范围
- **WHEN** 调用 Glob 工具，pattern 为 "*.rs"，path 为 "src/"
- **THEN** 只在 src/ 目录中搜索
- **AND** 返回相对路径

#### Scenario: 复杂模式匹配
- **GIVEN** 用户需要查找测试文件
- **WHEN** 调用 Glob 工具，pattern 为 "**/*test*.rs"
- **THEN** 返回所有名称包含 test 的 Rust 文件
- **AND** 包括嵌套目录中的文件

#### Scenario: 无匹配结果
- **GIVEN** 搜索不存在的模式
- **WHEN** 调用 Glob 工具
- **THEN** 返回空列表
- **AND** 不抛出错误

### Requirement: MultiEdit 工具
系统 SHALL 提供 MultiEdit 工具用于批量编辑文件。

#### Scenario: 批量编辑同一文件
- **GIVEN** 用户需要在一个文件中做多个修改
- **WHEN** 调用 MultiEdit 工具，包含多个编辑操作
- **THEN** 所有编辑按顺序应用
- **AND** 文件被一次性更新

#### Scenario: 跨文件编辑
- **GIVEN** 用户需要修改多个文件
- **WHEN** 调用 MultiEdit 工具，包含不同文件的编辑
- **THEN** 每个文件被独立修改
- **AND** 返回所有文件的修改结果

#### Scenario: 编辑冲突处理
- **GIVEN** 两个编辑操作影响同一行
- **WHEN** 执行 MultiEdit
- **THEN** 第二个编辑操作基于第一个的结果
- **AND** 返回冲突警告

#### Scenario: 部分失败处理
- **GIVEN** 多个编辑操作
- **WHEN** 其中一个编辑失败
- **THEN** 已成功的编辑被保留
- **AND** 失败的编辑返回错误信息
- **AND** 继续执行剩余的编辑

### Requirement: NotebookEdit 工具
系统 SHALL 提供 NotebookEdit 工具用于编辑 Jupyter Notebook。

#### Scenario: 替换单元格内容
- **GIVEN** 现有的 Jupyter Notebook
- **WHEN** 调用 NotebookEdit，edit_mode 为 "replace"
- **THEN** 指定单元格的内容被替换
- **AND** Notebook 结构保持完整

#### Scenario: 添加新单元格
- **GIVEN** 现有的 Jupyter Notebook
- **WHEN** 调用 NotebookEdit，edit_mode 为 "insert"
- **THEN** 新单元格被插入到指定位置
- **AND** 后续单元格索引被更新

#### Scenario: 删除单元格
- **GIVEN** 包含多个单元格的 Notebook
- **WHEN** 调用 NotebookEdit，edit_mode 为 "delete"
- **THEN** 指定单元格被删除
- **AND** Notebook 被正确保存

#### Scenario: Notebook 不存在
- **GIVEN** 指定的 Notebook 文件不存在
- **WHEN** 调用 NotebookEdit
- **THEN** 返回文件未找到错误
- **AND** 不创建新文件

### Requirement: AskUserQuestion 工具
系统 SHALL 提供 AskUserQuestion 工具用于交互式用户确认。

#### Scenario: 单选问题
- **GIVEN** Agent 需要用户选择一个选项
- **WHEN** 调用 AskUserQuestion，multi_select 为 false
- **THEN** 显示交互式选择器
- **AND** 用户可以选择一个选项
- **AND** 返回选择的值

#### Scenario: 多选问题
- **GIVEN** Agent 需要用户选择多个选项
- **WHEN** 调用 AskUserQuestion，multi_select 为 true
- **THEN** 显示多选选择器
- **AND** 用户可以选择多个选项
- **AND** 返回所有选择的值

#### Scenario: 带描述的选项
- **GIVEN** 复杂的选择场景
- **WHEN** 选项包含描述信息
- **THEN** 选择器显示选项标签和描述
- **AND** 用户基于描述做出选择

#### Scenario: CLI 模式交互
- **GIVEN** Oxide 运行在 CLI 模式
- **WHEN** 调用 AskUserQuestion
- **THEN** 使用 dialoguer 库显示交互界面
- **AND** 正确处理用户输入

#### Scenario: TUI 模式交互
- **GIVEN** Oxide 运行在 TUI 模式
- **WHEN** 调用 AskUserQuestion
- **THEN** 暂停 TUI 渲染
- **AND** 显示原生交互界面
- **AND** 用户选择后恢复 TUI

#### Scenario: 用户取消操作
- **GIVEN** 显示问题选择器
- **WHEN** 用户按 Esc 或 Ctrl+C
- **THEN** 返回取消状态
- **AND** Agent 可以处理取消情况

### Requirement: Git 集成增强
系统 SHALL 提供 Git 安全检查和工作流增强。

#### Scenario: 推送到主分支警告
- **GIVEN** 当前在 main/master 分支
- **WHEN** 执行 git push 命令
- **THEN** 显示警告消息
- **AND** 要求用户确认推送

#### Scenario: 强制推送保护
- **GIVEN** 用户尝试 git push --force
- **WHEN** 目标是主分支
- **THEN** 拒绝强制推送
- **AND** 显示安全警告
- **AND** 建议使用 --force-with-lease

#### Scenario: 未提交更改检查
- **GIVEN** 工作目录有未提交的更改
- **WHEN** 执行可能影响版本的操作
- **THEN** 显示未提交文件列表
- **AND** 提示用户先提交更改

#### Scenario: 远程分支状态检查
- **GIVEN** 准备推送代码
- **WHEN** 执行推送前检查
- **THEN** 显示本地和远程的差异
- **AND** 提示是否需要先拉取远程更改

### Requirement: Commit 规范验证
系统 SHALL 验证 Git commit 消息格式。

#### Scenario: 有效的 Conventional Commit
- **GIVEN** 格式正确的 commit 消息："feat: add user authentication"
- **WHEN** 验证 commit 消息
- **THEN** 验证通过
- **AND** 允许创建 commit

#### Scenario: 无效的 commit 格式
- **GIVEN** 不符合规范的 commit 消息："add new feature"
- **WHEN** 验证 commit 消息
- **THEN** 验证失败
- **AND** 显示格式要求
- **AND** 建议正确格式

#### Scenario: 带 scope 的 commit
- **GIVEN** commit 消息："feat(auth): add login support"
- **WHEN** 验证 commit 消息
- **THEN** 验证通过
- **AND** 正确解析 type 和 scope

#### Scenario: 详细描述的 commit
- **GIVEN** 多行 commit 消息
- **WHEN** 验证 commit 消息
- **THEN** 验证标题行格式
- **AND** 允许详细的正文描述

### Requirement: TaskOutput 工具
系统 SHALL 提供 TaskOutput 工具用于查询后台任务输出。

#### Scenario: 查询任务输出
- **GIVEN** 后台任务正在运行
- **WHEN** 调用 TaskOutput，block 为 true
- **THEN** 等待任务完成
- **AND** 返回完整输出

#### Scenario: 非阻塞查询
- **GIVEN** 后台任务正在运行
- **WHEN** 调用 TaskOutput，block 为 false
- **THEN** 立即返回当前状态
- **AND** 包含已产生的输出

#### Scenario: 任务已完成
- **GIVEN** 后台任务已完成
- **WHEN** 调用 TaskOutput
- **THEN** 返回完整任务输出
- **AND** 包含最终状态

#### Scenario: 超时处理
- **GIVEN** 长时间运行的任务
- **WHEN** 调用 TaskOutput，设置 timeout
- **THEN** 超时后返回部分输出
- **AND** 标记任务为超时状态

### Requirement: Task 工具
系统 SHALL 提供 Task 工具用于创建和管理后台任务。

#### Scenario: 创建后台任务
- **GIVEN** Agent 需要执行长时间任务
- **WHEN** 调用 Task 工具，run_in_background 为 true
- **THEN** 任务在后台启动
- **AND** 立即返回任务 ID
- **AND** 主进程继续执行

#### Scenario: 创建同步任务
- **GIVEN** Agent 需要立即获得结果
- **WHEN** 调用 Task 工具，run_in_background 为 false
- **THEN** 任务在前台执行
- **AND** 等待完成后返回结果

#### Scenario: 指定 Subagent 类型
- **GIVEN** 需要特定 Agent 执行任务
- **WHEN** 调用 Task 工具，指定 subagent_type
- **THEN** 使用指定的 Agent 执行任务
- **AND** 返回该 Agent 的执行结果

#### Scenario: 任务状态追踪
- **GIVEN** 多个后台任务
- **WHEN** 调用 Task 工具创建任务
- **THEN** 任务被添加到任务管理器
- **AND** 可以通过 task_id 查询状态

### Requirement: 工具错误处理
所有高级工具 SHALL 提供清晰的错误处理。

#### Scenario: 文件操作错误
- **GIVEN** 工具尝试操作不存在的文件
- **WHEN** 执行工具操作
- **THEN** 返回友好的错误消息
- **AND** 包含文件路径和错误原因

#### Scenario: 权限错误
- **GIVEN** 工具尝试无权限操作
- **WHEN** 执行工具操作
- **THEN** 返回权限错误
- **AND** 建议解决方法

#### Scenario: 无效输入
- **GIVEN** 工具接收到无效参数
- **WHEN** 执行工具操作
- **THEN** 返回输入验证错误
- **AND** 显示参数要求

### Requirement: 工具性能
高级工具 SHALL 优化性能以处理大型代码库。

#### Scenario: Glob 大型代码库
- **GIVEN** 包含数千文件的代码库
- **WHEN** 执行 Glob 操作
- **THEN** 在合理时间内完成（< 5秒）
- **AND** 内存使用可控

#### Scenario: MultiEdit 批量操作
- **GIVEN** 需要编辑多个文件
- **WHEN** 执行 MultiEdit
- **THEN** 操作原子性执行
- **AND** 避免重复文件读取

#### Scenario: Notebook 大文件处理
- **GIVEN** 包含数百单元格的 Notebook
- **WHEN** 执行 NotebookEdit
- **THEN** 高效解析和修改
- **AND** 保持 Notebook 结构完整性
