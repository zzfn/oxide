//! 快捷键说明
//!
//! 定义可用的快捷键列表（实际处理在 input.rs 的 LineEditor 中）。

/// 快捷键描述
pub struct KeyBinding {
    pub keys: &'static str,
    pub description: &'static str,
}

/// 获取所有快捷键描述
pub fn keybinding_help() -> Vec<KeyBinding> {
    vec![
        KeyBinding { keys: "Ctrl+A", description: "移动到行首" },
        KeyBinding { keys: "Ctrl+E", description: "移动到行尾" },
        KeyBinding { keys: "Ctrl+U", description: "清除到行首" },
        KeyBinding { keys: "Ctrl+K", description: "清除到行尾" },
        KeyBinding { keys: "Ctrl+W", description: "删除前一个词" },
        KeyBinding { keys: "Alt+F", description: "向前移动一个词" },
        KeyBinding { keys: "Alt+B", description: "向后移动一个词" },
        KeyBinding { keys: "Tab", description: "触发补全" },
        KeyBinding { keys: "Up/Down", description: "浏览历史" },
        KeyBinding { keys: "Ctrl+C x2", description: "退出" },
        KeyBinding { keys: "Ctrl+D", description: "退出（空行时）" },
    ]
}
