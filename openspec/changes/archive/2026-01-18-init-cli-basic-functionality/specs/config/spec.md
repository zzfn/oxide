## ADDED Requirements

### Requirement: API Key 配置
系统 SHALL 通过环境变量加载 API Key。

#### Scenario: 从环境变量加载
- **WHEN** 设置 DEEPSEEK_API_KEY 环境变量
- **THEN** 成功读取该值用于 API 认证
- **AND** 在所有 API 请求中使用 Bearer Token 格式

#### Scenario: 从 .env 文件加载
- **WHEN** 项目根目录存在 .env 文件
- **AND** .env 文件包含 DEEPSEEK_API_KEY
- **THEN** 自动加载该环境变量
- **AND** 优先级低于系统环境变量

### Requirement: API 配置
系统 SHALL 使用预定义的 API 端点和模型配置。

#### Scenario: API 端点配置
- **WHEN** 发送 API 请求
- **THEN** 使用端点 https://api.deepseek.com/v1/chat/completions
- **AND** 设置 Content-Type 为 application/json

#### Scenario: 模型配置
- **WHEN** 发送聊天请求
- **THEN** 使用 deepseek-chat 模型
- **AND** 设置 max_tokens 为 4096

### Requirement: 工具定义
系统 SHALL 定义可用的工具供 AI 调用。

#### Scenario: 工具列表
- **WHEN** 初始化 Agent
- **THEN** 定义 read_file, write_file, shell_execute 三个工具
- **AND** 每个工具包含名称、描述和参数定义
- **AND** 在 API 请求中包含 tools 数组

#### Scenario: 工具参数定义
- **WHEN** 定义 read_file 工具
- **THEN** 参数包含 path（必需，字符串类型）
- **WHEN** 定义 write_file 工具
- **THEN** 参数包含 path 和 content（均为必需）
- **WHEN** 定义 shell_execute 工具
- **THEN** 参数包含 command（必需，字符串类型）
