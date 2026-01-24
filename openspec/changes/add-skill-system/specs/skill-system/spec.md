# skill-system Specification

## Purpose
定义 Oxide CLI 的 Skill 系统，允许用户创建可重用的 prompt 模板并通过斜杠命令调用。

## ADDED Requirements

### Requirement: Skill 文件格式
Skill 文件 SHALL 使用 Markdown 格式，包含 Front matter 元数据和模板内容。

#### Scenario: 基本 Skill 文件结构
- **GIVEN** 一个有效的 Skill 文件
- **WHEN** 解析该文件
- **THEN** 文件以 `---` 开始和结束的 Front matter 块
- **AND** Front matter 包含 `name` 字段（Skill 名称）
- **AND** Front matter 包含 `description` 字段（描述）
- **AND** Front matter 包含可选的 `args` 数组（参数定义）
- **AND** Front matter 后跟随模板内容（纯文本或 Markdown）

#### Scenario: 参数定义
- **GIVEN** Skill 定义了参数
- **WHEN** 解析 `args` 数组
- **THEN** 每个参数包含 `name` 字段
- **AND** 每个参数包含 `description` 字段
- **AND** 每个参数包含可选的 `required` 布尔值（默认 false）
- **AND** 每个参数包含可选的 `default` 字符串（默认值）

### Requirement: Skill 存储位置
Skills SHALL 支持多个存储位置，按优先级加载。

#### Scenario: 本地 Skills
- **GIVEN** 项目根目录存在 `.oxide/skills/` 目录
- **WHEN** 加载 Skills
- **THEN** 扫描该目录下所有 `.md` 文件
- **AND** 本地 Skills 优先级最高
- **AND** 本地 Skill 可以覆盖同名全局 Skill

#### Scenario: 全局 Skills
- **GIVEN** 用户主目录存在 `~/.oxide/skills/` 目录
- **WHEN** 加载 Skills
- **THEN** 扫描该目录下所有 `.md` 文件
- **AND** 全局 Skills 优先级低于本地 Skills
- **AND** 全局 Skills 可在所有项目中使用

#### Scenario: 内置 Skills
- **WHEN** 加载 Skills
- **THEN** 加载内置示例 Skills（如 /commit, /compact）
- **AND** 内置 Skills 优先级最低
- **AND** 用户可覆盖内置 Skills

### Requirement: Skill 参数传递
Skills SHALL 支持命令行参数传递和模板变量替换。

#### Scenario: 简单参数传递
- **GIVEN** Skill 定义了参数 `message`
- **WHEN** 用户执行 `/commit -m "Fix bug"`
- **THEN** 解析参数 `-m` 对应到 `message`
- **AND** 将值 `"Fix bug"` 赋给 `message`
- **AND** 模板中的 `{{message}}` 替换为 `"Fix bug"`

#### Scenario: 多参数传递
- **GIVEN** Skill 定义了多个参数
- **WHEN** 用户传递多个参数（如 `-m "msg" -a "author"`）
- **THEN** 正确解析每个参数
- **AND** 替换模板中所有对应的变量

#### Scenario: 默认值使用
- **GIVEN** Skill 参数定义了默认值
- **WHEN** 用户未提供该参数
- **THEN** 使用默认值替换模板变量

#### Scenario: 缺失必需参数
- **GIVEN** Skill 参数标记为 `required: true`
- **WHEN** 用户未提供该参数
- **THEN** 显示错误提示，说明缺少哪个参数
- **AND** 不执行该 Skill
- **AND** 显示参数的使用说明

### Requirement: Skill 模板渲染
Skill 模板 SHALL 支持变量替换和基本的文本处理。

#### Scenario: 简单变量替换
- **GIVEN** 模板包含 `{{message}}` 占位符
- **WHEN** 渲染模板时 `message` 值为 `"Hello"`
- **THEN** 将所有 `{{message}}` 替换为 `"Hello"`
- **AND** 保持模板的其他内容不变

#### Scenario: 多变量替换
- **GIVEN** 模板包含多个变量（`{{title}}`, `{{body}}`, `{{author}}`）
- **WHEN** 渲染模板时提供了所有变量值
- **THEN** 正确替换所有变量
- **AND** 渲染后的内容作为用户消息发送

#### Scenario: 未替换的变量
- **GIVEN** 模板包含 `{{unknown_var}}`
- **WHEN** 渲染模板时未提供该变量值
- **THEN** 保留原始占位符 `{{unknown_var}}`
- **AND** 不报错（允许某些变量可选）

### Requirement: Skill 执行流程
Skill 执行 SHALL 遵循预定义的流程，从命令解析到 AI 响应。

#### Scenario: 成功执行
- **GIVEN** 用户输入 `/commit -m "Fix bug"`
- **AND** Skill 文件存在且格式正确
- **WHEN** 执行 Skill
- **THEN** 解析 Skill 名称和参数
- **AND** 从缓存或文件系统加载 Skill
- **AND** 验证所有必需参数
- **AND** 渲染模板内容
- **AND** 将渲染后的结果作为用户消息发送给 AI
- **AND** AI 正常处理并响应

#### Scenario: Skill 不存在
- **GIVEN** 用户输入 `/nonexistent -arg "value"`
- **WHEN** 执行命令
- **THEN** 检测到 Skill 不存在
- **AND** 显示错误提示（"Skill not found"）
- **AND** 列出可用的 Skills
- **AND** 建议相似的 Skill 名称（模糊匹配）

#### Scenario: 模板渲染错误
- **GIVEN** Skill 模板内容有效
- **WHEN** 渲染过程中发生错误
- **THEN** 显示友好的错误消息
- **AND** 说明可能的错误原因
- **AND** 不将部分渲染的内容发送给 AI

### Requirement: Skill 管理命令
CLI SHALL 提供管理 Skills 的命令。

#### Scenario: 列出所有 Skills
- **WHEN** 用户输入 `/skills` 或 `/skills list`
- **THEN** 显示所有可用的 Skills
- **AND** 按名称排序
- **AND** 标记每个 Skill 的来源（内置/全局/本地）
- **AND** 显示 Skill 描述

#### Scenario: 显示 Skill 详情
- **WHEN** 用户输入 `/skills show <name>`
- **AND** Skill 存在
- **THEN** 显示 Skill 的完整信息
- **AND** 包括名称、描述、来源
- **AND** 列出所有参数及其说明
- **AND** 显示模板内容（可选）

#### Scenario: Skill 不存在时的详情查询
- **WHEN** 用户输入 `/skills show <name>`
- **AND** Skill 不存在
- **THEN** 显示 "Skill not found" 错误
- **AND** 建议相似的 Skill 名称

### Requirement: Skill 错误处理
Skill 系统 SHALL 优雅处理各种错误情况。

#### Scenario: Front matter 解析失败
- **GIVEN** Skill 文件的 Front matter 格式错误
- **WHEN** 加载该 Skill
- **THEN** 跳过该 Skill 文件
- **AND** 在启动时显示警告（文件路径和解析错误）
- **AND** 继续加载其他 Skills

#### Scenario: 文件系统权限错误
- **GIVEN** Skills 目录不可读
- **WHEN** 扫描该目录
- **THEN** 跳过该目录
- **AND** 可选：显示警告消息
- **AND** 不影响 CLI 启动

#### Scenario: 参数解析失败
- **GIVEN** 用户提供的参数格式无效
- **WHEN** 解析参数
- **THEN** 显示友好的错误消息
- **AND** 说明正确的参数格式
- **AND** 不执行 Skill

### Requirement: Skill 缓存
CLI SHALL 缓存已加载的 Skills 以提高性能。

#### Scenario: 启动时缓存
- **WHEN** Oxide 启动
- **THEN** 加载所有可用 Skills
- **AND** 缓存到内存（使用 `once_cell::sync::Lazy`）
- **AND** 后续执行直接从缓存读取

#### Scenario: 缓存更新
- **GIVEN** 用户在 CLI 运行时修改了 Skill 文件
- **WHEN** 执行该 Skill
- **THEN** 使用缓存的旧版本
- **AND** 提示用户重启 CLI 以加载新版本
- **AND** **(未来)** 支持热重载

## Cross-References
- **cli-core**: 扩展斜杠命令系统以支持 Skills
- **config-system**: 可能需要配置 Skill 目录路径
