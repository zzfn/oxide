//! Markdown 渲染
//!
//! 使用 termimad 将 Markdown 文本渲染为终端格式。

use termimad::{MadSkin, crossterm::style::Color};

/// Markdown 渲染器
pub struct MarkdownRenderer {
    skin: MadSkin,
}

impl MarkdownRenderer {
    /// 创建新的 Markdown 渲染器
    pub fn new() -> Self {
        let mut skin = MadSkin::default();

        // 配置标题样式
        skin.headers[0].set_fg(Color::Cyan);
        skin.headers[1].set_fg(Color::Blue);
        skin.headers[2].set_fg(Color::Magenta);

        // 配置代码块样式
        skin.code_block.set_bg(Color::Rgb { r: 40, g: 40, b: 40 });
        skin.inline_code.set_fg(Color::Yellow);

        // 配置引用样式
        skin.quote_mark.set_fg(Color::DarkGrey);

        // 配置粗体和斜体
        skin.bold.set_fg(Color::White);
        skin.italic.set_fg(Color::Grey);

        Self { skin }
    }

    /// 渲染 Markdown 文本
    pub fn render(&self, markdown: &str) -> String {
        self.skin.term_text(markdown).to_string()
    }

    /// 渲染 Markdown 文本到指定宽度
    pub fn render_with_width(&self, markdown: &str, width: usize) -> String {
        self.skin.text(markdown, Some(width)).to_string()
    }

    /// 直接打印 Markdown 文本
    pub fn print(&self, markdown: &str) {
        self.skin.print_text(markdown);
    }

    /// 获取内部 skin 的可变引用（用于自定义配置）
    pub fn skin_mut(&mut self) -> &mut MadSkin {
        &mut self.skin
    }
}

impl Default for MarkdownRenderer {
    fn default() -> Self {
        Self::new()
    }
}
