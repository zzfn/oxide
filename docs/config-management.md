# 配置管理实现详解

## 目录

- [系统概述](#系统概述)
- [配置结构](#配置结构)
- [加载优先级](#加载优先级)
- [配置验证](#配置验证)
- [环境变量](#环境变量)
- [LLM 配置](#llm-配置)
- [Agent 配置](#agent-配置)
- [主题配置](#主题配置)
- [功能开关](#功能开关)
- [使用指南](#使用指南)

## 系统概述

Oxide 的配置管理系统采用分层架构，支持多层级配置加载、环境变量覆盖和完善的验证机制。系统设计既保证了灵活性，又确保了配置的安全性和一致性。

### 核心特性

- **分层加载**: 支持全局、项目、环境变量四个配置层级
- **优先级管理**: 清晰的配置覆盖规则
- **类型安全**: 使用 Rust 类型系统确保配置正确性
- **验证机制**: 完善的配置验证和默认值处理
- **热更新**: 支持运行时重载配置（部分功能）
- **安全性**: 敏感信息保护和文件权限控制

## 配置结构

### 配置文件格式

Oxide 使用 **TOML** 格式的配置文件：

```toml
# .oxide/config.toml 或 ~/.oxide/config.toml

[default]
# API 认证
base_url = "https://api.anthropic.com"
model = "claude-sonnet-4-20250514"
auth_token = "sk-ant-..."

# 模型参数
max_tokens = 4096
temperature = 0.7
top_p = 0.9
stream_chars_per_tick = 8

# Agent 配置
[agent.main]
model = "claude-sonnet-4-20250514"
temperature = 0.7

[agent.explore]
model = "claude-3-haiku-20250307"
temperature = 0.3
max_tokens = 2048

[agent.plan]
model = "claude-opus-4-20250514"
temperature = 0.5
max_tokens = 8192

[agent.code_reviewer]
model = "claude-opus-4-20250514"
temperature = 0.2

[agent.frontend_developer]
model = "claude-sonnet-4-20250514"
temperature = 0.7

# 主题配置
[theme]
mode = "dark"  # dark | light | auto
custom_theme = "/path/to/theme.toml"

# 功能开关
[features]
enable_mcp = false
enable_multimodal = false
enable_task_system = true
```

### 数据结构

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub default: DefaultConfig,
    pub agent: Option<AgentConfigs>,
    pub theme: Option<ThemeConfig>,
    pub features: Option<FeaturesConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultConfig {
    pub base_url: String,
    pub model: String,
    pub auth_token: String,
    pub max_tokens: u32,
    pub temperature: f32,
    pub top_p: Option<f32>,
    pub stream_chars_per_tick: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfigs {
    pub main: Option<AgentConfig>,
    pub explore: Option<AgentConfig>,
    pub plan: Option<AgentConfig>,
    pub code_reviewer: Option<AgentConfig>,
    pub frontend_developer: Option<AgentConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub model: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
}
```

## 加载优先级

### 四层配置系统

配置加载遵循以下优先级（从高到低）：

```
1. 环境变量 (Environment Variables)
       ↓ 覆盖
2. 项目配置 (Project Config: .oxide/config.toml)
       ↓ 覆盖
3. 全局配置 (Global Config: ~/.oxide/config.toml)
       ↓ 覆盖
4. 默认值 (Default Values)
```

### 加载流程

```rust
impl ConfigManager {
    pub fn load() -> Result<Config> {
        // 1. 加载默认值
        let mut config = Self::load_defaults();

        // 2. 加载全局配置（如果存在）
        if let Some(global_config) = Self::load_global_config()? {
            config = Self::merge_configs(config, global_config)?;
        }

        // 3. 加载项目配置（如果存在）
        if let Some(project_config) = Self::load_project_config()? {
            config = Self::merge_configs(config, project_config)?;
        }

        // 4. 应用环境变量覆盖
        config = Self::apply_env_overrides(config)?;

        // 5. 验证配置
        Self::validate(&config)?;

        Ok(config)
    }

    fn merge_configs(base: Config, override_: Config) -> Result<Config> {
        Ok(Config {
            default: if override_.default.base_url != DEFAULT_BASE_URL {
                override_.default
            } else {
                base.default
            },
            agent: Self::merge_agent_configs(base.agent, override_.agent),
            // ... 合并其他字段
        })
    }
}
```

### 环境变量覆盖

支持以下环境变量：

| 环境变量 | 说明 | 示例 |
|---------|------|------|
| `API_KEY` | API 密钥 | `sk-ant-...` |
| `ANTHROPIC_API_KEY` | Anthropic API 密钥 | `sk-ant-...` |
| `OXIDE_AUTH_TOKEN` | Oxide 认证令牌 | `sk-ant-...` |
| `API_URL` | API 端点 URL | `https://api.anthropic.com` |
| `OXIDE_BASE_URL` | Oxide 基础 URL | `https://api.anthropic.com` |
| `MODEL` | 模型名称 | `claude-sonnet-4-20250514` |
| `MODEL_NAME` | 模型名称（别名） | `claude-sonnet-4-20250514` |
| `MAX_TOKENS` | 最大 tokens | `4096` |
| `TEMPERATURE` | 温度参数 | `0.7` |
| `STREAM_CHARS_PER_TICK` | 流式输出速度 | `8` |

```rust
impl ConfigManager {
    fn apply_env_overrides(mut config: Config) -> Result<Config> {
        // API 密钥（多个备选）
        if let Ok(token) = std::env::var("OXIDE_AUTH_TOKEN") {
            config.default.auth_token = token;
        } else if let Ok(token) = std::env::var("ANTHROPIC_API_KEY") {
            config.default.auth_token = token;
        } else if let Ok(token) = std::env::var("API_KEY") {
            config.default.auth_token = token;
        }

        // API URL
        if let Ok(url) = std::env::var("OXIDE_BASE_URL") {
            config.default.base_url = url;
        } else if let Ok(url) = std::env::var("API_URL") {
            config.default.base_url = url;
        }

        // 模型名称
        if let Ok(model) = std::env::var("MODEL") {
            config.default.model = model;
        } else if let Ok(model) = std::env::var("MODEL_NAME") {
            config.default.model = model;
        }

        // 数值参数
        if let Ok(max_tokens) = std::env::var("MAX_TOKENS") {
            config.default.max_tokens = max_tokens.parse()?;
        }

        if let Ok(temperature) = std::env::var("TEMPERATURE") {
            config.default.temperature = temperature.parse()?;
        }

        Ok(config)
    }
}
```

## 配置验证

### 验证规则

```rust
impl ConfigManager {
    fn validate(config: &Config) -> Result<()> {
        // 验证 API 密钥
        if config.default.auth_token.is_empty() {
            bail!("API key cannot be empty. Please set API_KEY environment variable.");
        }

        // 验证 URL 格式
        if !config.default.base_url.starts_with("http://") &&
           !config.default.base_url.starts_with("https://") {
            bail!("Invalid base_url: must start with http:// or https://");
        }

        // 验证数值范围
        if config.default.temperature < 0.0 || config.default.temperature > 1.0 {
            bail!("Temperature must be between 0.0 and 1.0");
        }

        if config.default.max_tokens == 0 {
            bail!("Max tokens cannot be zero");
        }

        // 验证模型名称
        if !is_valid_model_name(&config.default.model) {
            bail!("Invalid model name: {}", config.default.model);
        }

        // Agent 特定验证
        if let Some(agent_configs) = &config.agent {
            if let Some(explore) = &agent_configs.explore {
                if let Some(ref model) = explore.model {
                    if !is_valid_model_name(model) {
                        bail!("Invalid model name for explore agent: {}", model);
                    }
                }
            }
        }

        Ok(())
    }

    fn is_valid_model_name(name: &str) -> bool {
        // 常见模型名称验证
        let valid_patterns = [
            "claude-", "gpt-", "deepseek-",
        ];

        valid_patterns.iter().any(|pattern| name.starts_with(pattern)) ||
        name.contains("-")  // 至少包含连字符
    }
}
```

### 默认值

```rust
impl ConfigManager {
    fn load_defaults() -> Config {
        Config {
            default: DefaultConfig {
                base_url: DEFAULT_BASE_URL.to_string(),
                model: DEFAULT_MODEL.to_string(),
                auth_token: String::new(),  // 必须由用户提供
                max_tokens: DEFAULT_MAX_TOKENS,
                temperature: DEFAULT_TEMPERATURE,
                top_p: Some(DEFAULT_TOP_P),
                stream_chars_per_tick: DEFAULT_STREAM_CHARS_PER_TICK,
            },
            agent: None,
            theme: None,
            features: None,
        }
    }
}

const DEFAULT_BASE_URL: &str = "https://api.anthropic.com";
const DEFAULT_MODEL: &str = "claude-sonnet-4-20250514";
const DEFAULT_MAX_TOKENS: u32 = 4096;
const DEFAULT_TEMPERATURE: f32 = 0.7;
const DEFAULT_TOP_P: f32 = 0.9;
const DEFAULT_STREAM_CHARS_PER_TICK: usize = 8;
```

## 环境变量

### .env 文件支持

Oxide 支持 `.env` 文件自动加载：

```env
# .env
ANTHROPIC_API_KEY=sk-ant-api03-...
API_URL=https://api.anthropic.com
MODEL=claude-sonnet-4-20250514
MAX_TOKENS=4096
TEMPERATURE=0.7
```

**注意**:
- `.env` 文件应添加到 `.gitignore`
- 不要提交包含真实密钥的 `.env` 文件
- 使用 `.env.example` 提供模板

### .env.example

```env
# .env.example
# API 配置
API_KEY=your_api_key_here
API_URL=https://api.anthropic.com
MODEL_NAME=claude-sonnet-4-20250514

# 模型参数
MAX_TOKENS=4096
TEMPERATURE=0.7
```

## LLM 配置

### 提供商配置

Oxide 支持多个 LLM 提供商：

#### Anthropic Claude

```toml
[default]
base_url = "https://api.anthropic.com"
model = "claude-sonnet-4-20250514"
auth_token = "sk-ant-..."
```

**支持的模型**:
- `claude-opus-4-20250514` - 最强模型
- `claude-sonnet-4-20250514` - 平衡性能
- `claude-3-haiku-20250307` - 快速模型

#### OpenAI 兼容

```toml
[default]
base_url = "https://api.openai.com/v1"
model = "gpt-4o"
auth_token = "sk-..."
```

**支持的模型**:
- `gpt-4o` - 最新 GPT-4
- `gpt-4o-mini` - 轻量版
- `gpt-4-turbo` - GPT-4 Turbo
- `gpt-3.5-turbo` - 经济选择

#### DeepSeek

```toml
[default]
base_url = "https://api.deepseek.com/v1"
model = "deepseek-chat"
auth_token = "sk-..."
```

**支持的模型**:
- `deepseek-chat` - 通用对话
- `deepseek-coder` - 代码专用

### 模型参数

```toml
[default]
# Token 限制
max_tokens = 4096  # 输出最大 tokens

# 采样参数
temperature = 0.7    # 创造性 (0.0 - 1.0)
top_p = 0.9         # 核采样 (0.0 - 1.0)

# 流式输出
stream_chars_per_tick = 8  # 每次输出的字符数
```

**参数说明**:

- **temperature**:
  - `0.0 - 0.3`: 确定性输出（代码生成、精确回答）
  - `0.4 - 0.7`: 平衡（日常对话）
  - `0.8 - 1.0`: 创造性（创意写作、头脑风暴）

- **max_tokens**:
  - 短回答: `512 - 2048`
  - 中等回答: `2048 - 4096`
  - 长篇文档: `8192 - 16384`

## Agent 配置

### Agent 特定参数

每个 Agent 可以有独立的配置：

```toml
[agent.explore]
model = "claude-3-haiku-20250307"  # 使用快速模型
temperature = 0.3                   # 低温度，确定性高
max_tokens = 2048                   # 较短输出

[agent.plan]
model = "claude-opus-4-20250514"    # 使用最强模型
temperature = 0.5                   # 中等温度
max_tokens = 8192                   # 长篇规划

[agent.code_reviewer]
model = "claude-opus-4-20250514"    # 使用最强模型
temperature = 0.2                   # 极低温度，严格审查
```

### 配置继承

Agent 配置继承 `[default]` 节的基础配置：

```rust
impl ConfigManager {
    fn get_agent_config(&self, agent_type: AgentType) -> AgentConfig {
        let default = &self.default;

        match agent_type {
            AgentType::Main => {
                let agent_config = self.agent.as_ref()
                    .and_then(|a| a.main.as_ref());

                AgentConfig {
                    model: agent_config.and_then(|a| a.model.clone())
                        .unwrap_or_else(|| default.model.clone()),
                    temperature: agent_config.and_then(|a| a.temperature)
                        .unwrap_or(default.temperature),
                    max_tokens: agent_config.and_then(|a| a.max_tokens)
                        .unwrap_or(default.max_tokens),
                }
            }
            // ... 其他 Agent 类型
        }
    }
}
```

## 主题配置

### 内置主题

```toml
[theme]
mode = "dark"  # dark | light | auto
```

### 自定义主题

```toml
[theme]
mode = "custom"
custom_theme = "/path/to/theme.toml"
```

**主题文件格式**:

```toml
# theme.toml
[colors]
primary = "#00ff00"
secondary = "#008800"
background = "#1a1a1a"
foreground = "#ffffff"

[ui]
border_style = "rounded"
padding = 2
spacing = 1
```

## 功能开关

### 功能标志

```toml
[features]
# MCP (Model Context Protocol) 支持
enable_mcp = false

# 多模态支持（图像、音频等）
enable_multimodal = false

# 任务系统（后台任务管理）
enable_task_system = true

# 调试模式
debug = false
```

### 功能检测

```rust
impl Config {
    pub fn is_feature_enabled(&self, feature: &str) -> bool {
        match feature {
            "mcp" => self.features
                .as_ref()
                .map(|f| f.enable_mcp)
                .unwrap_or(false),
            "multimodal" => self.features
                .as_ref()
                .map(|f| f.enable_multimodal)
                .unwrap_or(false),
            "task_system" => self.features
                .as_ref()
                .map(|f| f.enable_task_system)
                .unwrap_or(true),
            _ => false,
        }
    }
}
```

## 使用指南

### 初始化配置

```bash
# 创建配置文件
oxide config init

# 编辑配置
oxide config edit

# 验证配置
oxide config validate
```

### 查看当前配置

```bash
# 显示完整配置
/config show

# 显示特定部分
/config show agent
/config show features
```

### 配置文件位置

| 类型 | 位置 | 说明 |
|-----|------|------|
| 全局配置 | `~/.oxide/config.toml` | 所有项目共享 |
| 项目配置 | `.oxide/config.toml` | 项目特定 |
| 环境变量 | `.env` | 不提交到版本控制 |
| 示例配置 | `.env.example` | 配置模板 |

### 安全实践

1. **API 密钥保护**:
```bash
# ✅ 好的做法：使用环境变量
export ANTHROPIC_API_KEY="sk-ant-..."

# ❌ 不好的做法：硬编码在配置文件中
# [default]
# auth_token = "sk-ant-..."  # 不要这样做！
```

2. **文件权限**:
```bash
# 设置配置文件权限（仅所有者可读写）
chmod 600 ~/.oxide/config.toml
chmod 600 .env
```

3. **Git 忽略**:
```gitignore
# .gitignore
.env
.oxide/config.toml
.oxide/sessions/
```

### 配置模板

创建 `.env.example` 作为模板：

```env
# .env.example
# API 密钥（必需）
API_KEY=

# API 端点（可选）
API_URL=https://api.anthropic.com

# 模型名称（可选）
MODEL_NAME=claude-sonnet-4-20250514

# 模型参数（可选）
MAX_TOKENS=4096
TEMPERATURE=0.7
```

## 最佳实践

### 配置组织

1. **全局配置** - 放置通用的、跨项目的设置
```toml
# ~/.oxide/config.toml
[default]
model = "claude-sonnet-4-20250514"
temperature = 0.7

[agent.explore]
model = "claude-3-haiku-20250307"  # 全局使用快速模型探索
```

2. **项目配置** - 覆盖特定项目的需求
```toml
# .oxide/config.toml
[default]
model = "claude-opus-4-20250514"  # 重要项目使用最强模型

[agent.plan]
max_tokens = 16384  # 需要长篇规划
```

### 环境特定配置

使用不同环境变量管理开发/生产环境：

```bash
# 开发环境
export OXIDE_ENV="development"
export MODEL="claude-3-haiku-20250307"

# 生产环境
export OXIDE_ENV="production"
export MODEL="claude-opus-4-20250514"
```

### 配置验证

在 CI/CD 中验证配置：

```bash
# 验证配置文件
oxide config validate

# 检查必需的环境变量
if [ -z "$API_KEY" ]; then
    echo "Error: API_KEY not set"
    exit 1
fi
```

## 相关文档

- [Agent 系统](./agent-system.md) - Agent 配置和使用
- [工具系统](./tool-system.md) - 工具权限配置
- [整体架构](./architecture.md) - 项目架构总览
