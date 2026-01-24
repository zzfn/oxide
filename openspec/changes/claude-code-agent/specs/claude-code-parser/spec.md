# claude-code-parser Specification

此规范描述 Claude Code 输出的解析逻辑，将 Claude Code 的原始输出转换为结构化数据。

## ADDED Requirements

### Requirement: Claude Code 输出解析
系统 SHALL 能够解析 Claude Code 的输出，提取工具调用、文件操作和命令执行结果。

#### Scenario: 解析 Read 工具调用
- **WHEN** Claude Code 调用 Read 工具
- **THEN** 系统 SHALL 识别 Read 工具调用的模式
- **AND** 系统 SHALL 提取文件路径参数
- **AND** 系统 SHALL 提取文件内容（如果输出中包含）
- **AND** 系统 SHALL 记录工具调用状态（成功/失败）

#### Scenario: 解析 Write 工具调用
- **WHEN** Claude Code 调用 Write 工具
- **THEN** 系统 SHALL 识别 Write 工具调用的模式
- **AND** 系统 SHALL 提取目标文件路径
- **AND** 系统 SHALL 提取写入的内容（或文件大小）
- **AND** 系统 SHALL 记录写入操作是否成功

#### Scenario: 解析 Edit 工具调用
- **WHEN** Claude Code 调用 Edit 工具（使用 unified diff）
- **THEN** 系统 SHALL 识别 Edit 工具调用的模式
- **AND** 系统 SHALL 提取目标文件路径
- **AND** 系统 SHALL 提取 diff 内容
- **AND** 系统 SHALL 提取变更行数统计

#### Scenario: 解析 Bash 工具调用
- **WHEN** Claude Code 调用 Bash 工具
- **THEN** 系统 SHALL 识别 Bash 工具调用的模式
- **AND** 系统 SHALL 提取执行的命令字符串
- **AND** 系统 SHALL 提取命令的退出码
- **AND** 系统 SHALL 提取命令的标准输出
- **AND** 系统 SHALL 提取命令的错误输出（如存在）

#### Scenario: 解析 Task 工具调用
- **WHEN** Claude Code 调用 Task 工具（启动子 Agent）
- **THEN** 系统 SHALL 识别 Task 工具调用的模式
- **AND** 系统 SHALL 提取子 Agent 类型（如 "code-reviewer"）
- **AND** 系统 SHALL 提取任务描述
- **AND** 系统 SHALL 记录 Task ID（如可获取）

#### Scenario: 解析 Glob 工具调用
- **WHEN** Claude Code 调用 Glob 工具
- **THEN** 系统 SHALL 识别 Glob 工具调用的模式
- **AND** 系统 SHALL 提取文件模式（pattern）
- **AND** 系统 SHALL 提取匹配的文件列表（如果输出中包含）

#### Scenario: 解析 Grep 工具调用
- **WHEN** Claude Code 调用 Grep 工具
- **THEN** 系统 SHALL 识别 Grep 工具调用的模式
- **AND** 系统 SHALL 提取搜索模式和路径
- **AND** 系统 SHALL 提取匹配结果（文件、行号、内容）

### Requirement: 模式匹配策略
系统 SHALL 使用正则表达式和状态机组合进行输出解析。

#### Scenario: 正则表达式模式
- **WHEN** 定义工具调用模式
- **THEN** 系统 SHALL 为每个工具定义正则表达式
- **AND** 模式 SHALL 匹配工具调用的开始标记
- **AND** 模式 SHALL 提取关键参数（使用捕获组）
- **AND** 模式 SHALL 兼容 Claude Code 的不同版本

#### Scenario: 状态机处理
- **WHEN** 解析跨行的工具调用
- **THEN** 系统 SHALL 使用状态机跟踪当前解析状态
- **AND** 系统 SHALL 能够处理工具调用、结果、错误等不同状态
- **AND** 系统 SHALL 能够处理嵌套的工具调用

#### Scenario: 容错处理
- **WHEN** 输出不完全匹配预期模式
- **THEN** 系统 SHALL 保留原始输出
- **AND** 系统 SHALL 记录解析警告
- **AND** 系统 SHALL 继续处理后续输出
- **AND** 系统 SHALL 不中断整体解析流程

### Requirement: ANSI 转义字符处理
系统 SHALL 能够正确处理 ANSI 转义字符，同时支持显示和解析。

#### Scenario: 保留 ANSI 用于显示
- **WHEN** 捕获 Claude Code 输出
- **THEN** 系统 SHALL 保留原始的 ANSI 转义字符
- **AND** 系统 SHALL 将带 ANSI 的输出渲染到终端
- **AND** 用户 SHALL 能够看到彩色输出和进度条

#### Scenario: 过滤 ANSI 用于解析
- **WHEN** 解析工具调用
- **THEN** 系统 SHALL 在解析时过滤 ANSI 转义字符
- **AND** 系统 SHALL 仅处理纯文本内容
- **AND** ANSI 过滤 SHALL 不影响正则表达式匹配

#### Scenario: 进度条和状态更新
- **WHEN** Claude Code 显示进度条或状态更新
- **THEN** 系统 SHALL 识别进度条模式（如 `\r` 和 `\u001b[0K`）
- **AND** 系统 SHALL 保留进度条用于显示
- **AND** 系统 SHALL 不将进度条内容解析为工具调用

### Requirement: 流式解析
系统 SHALL 支持流式解析，实时处理 Claude Code 的输出。

#### Scenario: 逐行解析
- **WHEN** Claude Code 产生输出
- **THEN** 系统 SHALL 逐行读取输出
- **AND** 系统 SHALL 立即解析每一行
- **AND** 系统 SHALL 不等待进程结束

#### Scenario: 增量构建结构化输出
- **WHEN** 解析工具调用
- **THEN** 系统 SHALL 增量地将解析结果添加到 `StructuredOutput`
- **AND** 系统 SHALL 保持结果的时序
- **AND** 系统 SHALL 支持实时导出中间结果

#### Scenario: 缓冲区管理
- **WHEN** 工具调用跨多行
- **THEN** 系统 SHALL 使用缓冲区累积部分内容
- **AND** 系统 SHALL 在工具调用完成后清空缓冲区
- **AND** 系统 SHALL 限制缓冲区大小，避免内存溢出

### Requirement: 解析结果验证
系统 SHALL 验证解析结果的正确性和完整性。

#### Scenario: 必需字段验证
- **WHEN** 解析工具调用
- **THEN** 系统 SHALL 验证必需字段存在（如工具名称、参数）
- **AND** 如果字段缺失，系统 SHALL 记录警告
- **AND** 系统 SHALL 尝试使用默认值或保留部分结果

#### Scenario: 数据类型验证
- **WHEN** 解析数值、路径等字段
- **THEN** 系统 SHALL 验证数据类型正确性
- **AND** 系统 SHALL 验证路径格式有效性
- **AND** 系统 SHALL 验证退出码是整数

#### Scenario: 一致性检查
- **WHEN** 解析多个相关工具调用
- **THEN** 系统 SHALL 检查调用之间的一致性
- **AND** 系统 SHALL 检测文件操作顺序的合理性
- **AND** 系统 SHALL 标记可疑的操作序列

### Requirement: 性能优化
解析器 SHALL 满足性能要求，不影响用户体验。

#### Scenario: 解析速度
- **WHEN** Claude Code 产生大量输出
- **THEN** 系统 SHALL 能够实时解析
- **AND** 解析速度 SHALL 不低于输出速度
- **AND** CPU 使用率 SHALL 保持合理（< 30% 单核）

#### Scenario: 正则表达式优化
- **WHEN** 执行模式匹配
- **THEN** 系统 SHALL 使用编译后的正则表达式（缓存的 Regex）
- **AND** 系统 SHALL 避免回溯和灾难性匹配
- **AND** 系统 SHALL 使用非贪婪匹配（`*?`、`+?`）

#### Scenario: 内存效率
- **WHEN** 解析长时间运行的会话
- **THEN** 系统 SHALL 定期将结果写入文件
- **AND** 系统 SHALL 限制内存中的结果数量
- **AND** 系统 SHALL 支持增量导出

### Requirement: 可扩展性
解析器 SHALL 设计为可扩展，支持 Claude Code 新增工具和格式变化。

#### Scenario: 添加新工具支持
- **WHEN** Claude Code 添加新工具
- **THEN** 系统 SHALL 能够轻松添加新的解析规则
- **AND** 系统 SHALL 不需要修改核心解析逻辑
- **AND** 系统 SHALL 支持插件式添加解析器

#### Scenario: 版本兼容性
- **WHEN** Claude Code 更新版本
- **THEN** 系统 SHALL 能够检测版本差异
- **AND** 系统 SHALL 支持多版本解析模式
- **AND** 系统 SHALL 在输出格式中包含解析器版本

#### Scenario: 自定义解析规则
- **WHEN** 用户需要自定义解析逻辑
- **THEN** 系统 SHALL 支持配置文件定义解析规则
- **AND** 系统 SHALL 支持用户自定义正则表达式
- **AND** 系统 SHALL 提供默认规则作为后备
