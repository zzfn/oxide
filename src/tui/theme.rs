//! TUI 主题系统
//!
//! 提供可配置的颜色主题，支持内置主题和自定义主题。

use ratatui::style::{Color, Style};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// 主题模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    #[default]
    Dark,
    Light,
    HighContrast,
}

/// 主题颜色配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
    /// 主色调（品牌色）
    pub primary: Color,
    /// 次要色调
    pub secondary: Color,
    /// 用户消息颜色
    pub user_message: Color,
    /// 助手消息颜色
    pub assistant_message: Color,
    /// 工具消息颜色
    pub tool_message: Color,
    /// 成功状态颜色
    pub success: Color,
    /// 警告状态颜色
    pub warning: Color,
    /// 错误状态颜色
    pub error: Color,
    /// 边框颜色
    pub border: Color,
    /// 背景色
    pub background: Color,
    /// 前景色（文本）
    pub foreground: Color,
    /// 帮助文本颜色
    pub help_text: Color,
    /// 输入提示颜色
    pub input_prompt: Color,
    /// 命令高亮颜色
    pub command_highlight: Color,
}

/// 主题配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    /// 主题名称
    pub name: String,
    /// 颜色配置
    pub colors: ThemeColors,
}

impl Theme {
    /// 创建暗色主题
    pub fn dark() -> Self {
        Theme {
            name: "dark".to_string(),
            colors: ThemeColors {
                primary: Color::Blue,
                secondary: Color::Cyan,
                user_message: Color::Cyan,
                assistant_message: Color::Blue,
                tool_message: Color::Yellow,
                success: Color::Green,
                warning: Color::Yellow,
                error: Color::Red,
                border: Color::DarkGray,
                background: Color::Black,
                foreground: Color::White,
                help_text: Color::DarkGray,
                input_prompt: Color::Green,
                command_highlight: Color::Green,
            },
        }
    }

    /// 创建亮色主题
    pub fn light() -> Self {
        Theme {
            name: "light".to_string(),
            colors: ThemeColors {
                primary: Color::Blue,
                secondary: Color::Cyan,
                user_message: Color::Blue,
                assistant_message: Color::Cyan,
                tool_message: Color::Yellow,
                success: Color::Green,
                warning: Color::Yellow,
                error: Color::Red,
                border: Color::Gray,
                background: Color::White,
                foreground: Color::Black,
                help_text: Color::Gray,
                input_prompt: Color::Blue,
                command_highlight: Color::Blue,
            },
        }
    }

    /// 创建高对比度主题
    pub fn high_contrast() -> Self {
        Theme {
            name: "high_contrast".to_string(),
            colors: ThemeColors {
                primary: Color::White,
                secondary: Color::White,
                user_message: Color::White,
                assistant_message: Color::Cyan,
                tool_message: Color::Yellow,
                success: Color::Green,
                warning: Color::Yellow,
                error: Color::Red,
                border: Color::White,
                background: Color::Black,
                foreground: Color::White,
                help_text: Color::White,
                input_prompt: Color::White,
                command_highlight: Color::Yellow,
            },
        }
    }

    /// 从模式创建主题
    pub fn from_mode(mode: ThemeMode) -> Self {
        match mode {
            ThemeMode::Dark => Self::dark(),
            ThemeMode::Light => Self::light(),
            ThemeMode::HighContrast => Self::high_contrast(),
        }
    }

    /// 从文件加载主题
    pub fn from_file(path: &PathBuf) -> anyhow::Result<Self> {
        let content = fs::read_to_string(path)?;
        let theme: Theme = toml::from_str(&content)?;
        Ok(theme)
    }

    /// 保存主题到文件
    pub fn to_file(&self, path: &PathBuf) -> anyhow::Result<()> {
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// 获取主色调样式
    pub fn primary_style(&self) -> Style {
        Style::default().fg(self.colors.primary)
    }

    /// 获取次要色调样式
    pub fn secondary_style(&self) -> Style {
        Style::default().fg(self.colors.secondary)
    }

    /// 获取用户消息样式
    pub fn user_message_style(&self) -> Style {
        Style::default().fg(self.colors.user_message)
    }

    /// 获取助手消息样式
    pub fn assistant_message_style(&self) -> Style {
        Style::default().fg(self.colors.assistant_message)
    }

    /// 获取工具消息样式
    pub fn tool_message_style(&self) -> Style {
        Style::default().fg(self.colors.tool_message)
    }

    /// 获取成功状态样式
    pub fn success_style(&self) -> Style {
        Style::default().fg(self.colors.success)
    }

    /// 获取警告状态样式
    pub fn warning_style(&self) -> Style {
        Style::default().fg(self.colors.warning)
    }

    /// 获取错误状态样式
    pub fn error_style(&self) -> Style {
        Style::default().fg(self.colors.error)
    }

    /// 获取边框样式
    pub fn border_style(&self) -> Style {
        Style::default().fg(self.colors.border)
    }

    /// 获取背景样式
    pub fn background_style(&self) -> Style {
        Style::default().fg(self.colors.foreground).bg(self.colors.background)
    }

    /// 获取帮助文本样式
    pub fn help_text_style(&self) -> Style {
        Style::default().fg(self.colors.help_text)
    }

    /// 获取输入提示样式
    pub fn input_prompt_style(&self) -> Style {
        Style::default().fg(self.colors.input_prompt)
    }

    /// 获取命令高亮样式
    pub fn command_highlight_style(&self) -> Style {
        Style::default().fg(self.colors.command_highlight)
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

/// Color 序列化和反序列化辅助函数
mod color_serde {
    use ratatui::style::Color;
    use serde::{Deserialize, Deserializer, Serializer};
    use std::str::FromStr;

    pub fn serialize<S>(color: &Color, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let color_str = match color {
            Color::Black => "black",
            Color::Red => "red",
            Color::Green => "green",
            Color::Yellow => "yellow",
            Color::Blue => "blue",
            Color::Magenta => "magenta",
            Color::Cyan => "cyan",
            Color::White => "white",
            Color::Gray => "gray",
            Color::DarkGray => "dark_gray",
            Color::LightRed => "light_red",
            Color::LightGreen => "light_green",
            Color::LightYellow => "light_yellow",
            Color::LightBlue => "light_blue",
            Color::LightMagenta => "light_magenta",
            Color::LightCyan => "light_cyan",
            Color::Rgb(r, g, b) => &format!("rgb({},{},{})", r, g, b),
            _ => "black",
        };
        serializer.serialize_str(color_str)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Color, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let color = match s.to_lowercase().as_str() {
            "black" => Color::Black,
            "red" => Color::Red,
            "green" => Color::Green,
            "yellow" => Color::Yellow,
            "blue" => Color::Blue,
            "magenta" => Color::Magenta,
            "cyan" => Color::Cyan,
            "white" => Color::White,
            "gray" => Color::Gray,
            "dark_gray" => Color::DarkGray,
            "light_red" => Color::LightRed,
            "light_green" => Color::LightGreen,
            "light_yellow" => Color::LightYellow,
            "light_blue" => Color::LightBlue,
            "light_magenta" => Color::LightMagenta,
            "light_cyan" => Color::LightCyan,
            s if s.starts_with("rgb(") => {
                // 解析 rgb(r,g,b)
                let parts = s
                    .trim_start_matches("rgb(")
                    .trim_end_matches(')')
                    .split(',');
                let parts: Vec<u8> = parts
                    .filter_map(|p| p.trim().parse().ok())
                    .collect();
                if parts.len() == 3 {
                    Color::Rgb(parts[0], parts[1], parts[2])
                } else {
                    Color::Black
                }
            }
            _ => Color::Black,
        };
        Ok(color)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_modes() {
        let dark = Theme::from_mode(ThemeMode::Dark);
        assert_eq!(dark.name, "dark");

        let light = Theme::from_mode(ThemeMode::Light);
        assert_eq!(light.name, "light");

        let high_contrast = Theme::from_mode(ThemeMode::HighContrast);
        assert_eq!(high_contrast.name, "high_contrast");
    }

    #[test]
    fn test_theme_styles() {
        let theme = Theme::dark();
        let primary_style = theme.primary_style();
        assert_eq!(primary_style.fg, Some(Color::Blue));
    }

    #[test]
    fn test_default_theme() {
        let theme = Theme::default();
        assert_eq!(theme.name, "dark");
    }
}
