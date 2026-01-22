# config-system Specification

## Purpose
定义 Oxide 的配置管理系统，支持多层级配置、项目指令和灵活的配置覆盖机制。

## ADDED Requirements

### Requirement: 配置层次结构
系统 SHALL 支持三层配置结构：全局配置、项目配置和运行时配置。

#### Scenario: 全局默认配置
- **GIVEN** ~/.oxide/config.toml 文件存在
- **WHEN** Oxide 启动
- **THEN** 加载全局配置作为默认值
- **AND** 配置应用于所有 Oxide 会话

#### Scenario: 项目特定配置
- **GIVEN** .oxide/config.toml 文件存在
- **WHEN** 在项目目录中启动 Oxide
- **THEN** 项目配置覆盖全局配置
- **AND** 配置只应用于当前项目

#### Scenario: 运行时配置覆盖
- **GIVEN** 全局和项目配置
- **WHEN** 设置环境变量
- **THEN** 环境变量优先级最高
- **AND** 运行时配置立即生效

#### Scenario: 配置优先级
- **GIVEN** 同一个配置项在三层都存在
- **WHEN** 查询配置值
- **THEN** 环境变量 > 项目配置 > 全局配置
- **AND** 返回优先级最高的值

### Requirement: 配置文件格式
配置文件 SHALL 使用 TOML 格式。

#### Scenario: 基础配置结构
- **GIVEN** config.toml 文件
- **WHEN** 解析配置
- **THEN** 支持以下节：
  - [default] - 默认模型和 API 设置
  - [agent.*] - Agent 特定配置
  - [theme] - UI 主题设置
  - [tui] - TUI 特定设置
  - [features] - 功能开关

#### Scenario: 模型配置
- **GIVEN** [default] 节配置
- **WHEN** 设置模型参数
- **THEN** 支持以下字段：
  - model = "claude-sonnet-4-20250514"
  - max_tokens = 4096
  - temperature = 0.7
  - base_url = "https://api.anthropic.com"

#### Scenario: Agent 配置覆盖
- **GIVEN** [agent.explore] 节配置
- **WHEN** 创建 Explore Agent
- **THEN** 使用 Agent 特定的模型设置
- **AND** 未设置的字段使用 [default] 值

### Requirement: 项目指令系统
系统 SHALL 支持项目级别的指令文档（.oxide/CONFIG.md）。

#### Scenario: 加载项目指令
- **GIVEN** .oxide/CONFIG.md 文件存在
- **WHEN** Oxide 启动
- **THEN** 读取 CONFIG.md 内容
- **AND** 内容被添加到系统提示词

#### Scenario: 指令格式支持
- **GIVEN** CONFIG.md 包含 Markdown
- **WHEN** 解析指令
- **THEN** 支持：
  - 标题（# ## ###）
  - 列表（无序和有序）
  - 代码块
  - 普通文本

#### Scenario: 指令应用范围
- **GIVEN** CONFIG.md 定义了项目规范
- **WHEN** Agent 生成响应
- **THEN** 遵循项目指令中的规范
- **AND** 应用项目特定的编码风格

#### Scenario: 无指令文件
- **GIVEN** 项目没有 CONFIG.md
- **WHEN** Oxide 启动
- **THEN** 使用默认系统提示词
- **AND** 不显示错误

### Requirement: 配置验证
系统 SHALL 验证配置的有效性。

#### Scenario: 有效的模型配置
- **GIVEN** 配置中指定了支持的模型
- **WHEN** 验证配置
- **THEN** 验证通过
- **AND** Agent 使用该模型

#### Scenario: 无效的模型名称
- **GIVEN** 配置中指定了不支持的模型
- **WHEN** 验证配置
- **THEN** 显示警告
- **AND** 建议使用支持的模型
- **AND** 回退到默认模型

#### Scenario: 缺少必需字段
- **GIVEN** 配置缺少必需的 API 密钥
- **WHEN** 验证配置
- **THEN** 显示清晰的错误消息
- **AND** 指引用户如何设置
- **AND** 程序退出并返回非零状态码

#### Scenario: 配置语法错误
- **GIVEN** TOML 文件有语法错误
- **WHEN** 解析配置
- **THEN** 显示语法错误位置
- **AND** 提供修复建议
- **AND** 使用默认配置继续

### Requirement: 配置热重载
系统 SHALL 支持在运行时重新加载配置。

#### Scenario: 手动重载配置
- **GIVEN** Oxide 正在运行
- **WHEN** 用户执行 `/config reload` 命令
- **THEN** 重新读取配置文件
- **AND** 应用新配置
- **AND** 显示重载确认

#### Scenario: 配置变更提示
- **GIVEN** 配置文件被外部修改
- **WHEN** 检测到变更
- **THEN** 显示配置已变更的提示
- **AND** 建议用户重载配置

#### Scenario: 无效的重载
- **GIVEN** 新配置包含错误
- **WHEN** 尝试重载配置
- **THEN** 保持当前配置
- **AND** 显示错误消息
- **AND** 不中断运行会话

### Requirement: 配置查询
系统 SHALL 提供命令查询当前配置。

#### Scenario: 显示所有配置
- **WHEN** 用户执行 `/config show`
- **THEN** 显示当前生效的配置
- **AND** 标注配置来源（全局/项目/环境）
- **AND** 隐藏敏感信息（API 密钥）

#### Scenario: 显示单个配置项
- **WHEN** 用户执行 `/config show model`
- **THEN** 显示当前模型配置
- **AND** 显示配置优先级

#### Scenario: 编辑配置
- **WHEN** 用户执行 `/config edit`
- **THEN** 打开配置文件在默认编辑器
- **AND** 编辑后提示重载

### Requirement: 环境变量支持
系统 SHALL 支持通过环境变量覆盖配置。

#### Scenario: API 密钥环境变量
- **GIVEN** OXIDE_AUTH_TOKEN 环境变量已设置
- **WHEN** Oxide 启动
- **THEN** 使用环境变量中的 API 密钥
- **AND** 忽略配置文件中的值

#### Scenario: 模型环境变量
- **GIVEN** OXIDE_MODEL 环境变量已设置
- **WHEN** Oxide 启动
- **THEN** 使用环境变量中的模型
- **AND** 覆盖所有配置文件设置

#### Scenario: 基础 URL 环境变量
- **GIVEN** OXIDE_BASE_URL 环境变量已设置
- **WHEN** Oxide 启动
- **THEN** 使用环境变量中的 URL
- **AND** 覆盖配置文件设置

### Requirement: 配置文件创建
系统 SHALL 能够创建初始配置文件。

#### Scenario: 首次运行引导
- **GIVEN** 用户首次运行 Oxide
- **WHEN** 检测到没有配置文件
- **THEN** 询问是否创建示例配置
- **AND** 在 ~/.oxide/ 创建 config.toml.example

#### Scenario: 项目配置初始化
- **GIVEN** 项目目录中没有 .oxide/
- **WHEN** 用户执行 `/config init`
- **THEN** 创建 .oxide/ 目录
- **AND** 创建 config.toml 模板
- **AND** 创建 CONFIG.md 模板

### Requirement: 配置迁移
系统 SHALL 支持配置格式的版本迁移。

#### Scenario: 旧版配置升级
- **GIVEN** 存在旧版本配置文件
- **WHEN** Oxide 启动
- **THEN** 检测配置版本
- **AND** 自动迁移到新格式
- **AND** 备份旧配置

#### Scenario: 迁移失败处理
- **GIVEN** 配置迁移遇到错误
- **WHEN** 执行迁移
- **THEN** 显示迁移错误
- **AND** 保持旧配置不变
- **AND** 提供手动迁移指引

### Requirement: 配置安全
系统 SHALL 保护配置中的敏感信息。

#### Scenario: API 密钥屏蔽
- **GIVEN** 配置包含 API 密钥
- **WHEN** 显示配置
- **THEN** API 密钥被部分屏蔽
- **AND** 只显示前 4 个字符

#### Scenario: 配置文件权限
- **GIVEN** 创建配置文件
- **WHEN** 写入文件
- **THEN** 设置文件权限为 600（仅所有者可读写）
- **AND** 在 Linux/macOS 上使用 chmod

#### Scenario: 敏感信息日志
- **GIVEN** 启用调试日志
- **WHEN** 记录配置信息
- **THEN** 不记录敏感信息
- **AND** API 密钥被替换为 "****"

### Requirement: 配置验证模式
系统 SHALL 提供配置验证工具。

#### Scenario: 验证当前配置
- **WHEN** 用户执行 `oxide config validate`
- **THEN** 检查所有配置文件
- **AND** 报告任何错误或警告
- **AND** 不启动 Oxide

#### Scenario: 验证特定配置文件
- **WHEN** 用户执行 `oxide config validate ~/.oxide/config.toml`
- **THEN** 只验证指定文件
- **AND** 显示详细的验证结果

### Requirement: Agent 配置继承
Agent 配置 SHALL 支持从默认配置继承。

#### Scenario: 继承默认配置
- **GIVEN** [default] 配置了 max_tokens = 4096
- **AND** [agent.explore] 没有配置 max_tokens
- **WHEN** 创建 Explore Agent
- **THEN** 使用默认的 max_tokens = 4096

#### Scenario: 覆盖特定字段
- **GIVEN** [default] 配置了 temperature = 0.7
- **AND** [agent.explore] 配置了 temperature = 0.3
- **WHEN** 创建 Explore Agent
- **THEN** 使用 Agent 特定的 temperature = 0.3
- **AND** 其他字段使用默认值

### Requirement: 主题配置
系统 SHALL 支持主题配置。

#### Scenario: 内置主题选择
- **GIVEN** [theme] 节配置
- **WHEN** 设置 mode = "dark"
- **THEN** 应用暗色主题
- **AND** 更新所有 UI 元素颜色

#### Scenario: 自定义主题路径
- **GIVEN** 自定义主题文件 ~/.oxide/custom_theme.toml
- **WHEN** 设置 custom_theme 路径
- **THEN** 加载自定义主题
- **AND** 覆盖内置主题颜色

#### Scenario: 主题切换
- **GIVEN** Oxide 正在运行
- **WHEN** 用户执行 `/theme set light`
- **THEN** 立即切换到浅色主题
- **AND** 更新配置文件
- **AND** 刷新 TUI 显示
