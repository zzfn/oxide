//! 快捷键绑定
//!
//! 配置 Reedline 的快捷键，使用 Emacs 模式并添加自定义绑定。

use reedline::{
    default_emacs_keybindings, EditCommand, KeyCode, KeyModifiers, Keybindings, ReedlineEvent,
};

/// 创建自定义快捷键绑定
pub fn create_keybindings() -> Keybindings {
    let mut keybindings = default_emacs_keybindings();

    // Ctrl+L: 清屏
    keybindings.add_binding(
        KeyModifiers::CONTROL,
        KeyCode::Char('l'),
        ReedlineEvent::ClearScreen,
    );

    // Ctrl+U: 清除当前行
    keybindings.add_binding(
        KeyModifiers::CONTROL,
        KeyCode::Char('u'),
        ReedlineEvent::Edit(vec![EditCommand::CutFromStart]),
    );

    // Ctrl+K: 删除到行尾
    keybindings.add_binding(
        KeyModifiers::CONTROL,
        KeyCode::Char('k'),
        ReedlineEvent::Edit(vec![EditCommand::CutToEnd]),
    );

    // Ctrl+W: 删除前一个词
    keybindings.add_binding(
        KeyModifiers::CONTROL,
        KeyCode::Char('w'),
        ReedlineEvent::Edit(vec![EditCommand::CutWordLeft]),
    );

    // Alt+Backspace: 删除前一个词（备选）
    keybindings.add_binding(
        KeyModifiers::ALT,
        KeyCode::Backspace,
        ReedlineEvent::Edit(vec![EditCommand::CutWordLeft]),
    );

    // Ctrl+Y: 粘贴
    keybindings.add_binding(
        KeyModifiers::CONTROL,
        KeyCode::Char('y'),
        ReedlineEvent::Edit(vec![EditCommand::PasteCutBufferBefore]),
    );

    // Ctrl+A: 移动到行首
    keybindings.add_binding(
        KeyModifiers::CONTROL,
        KeyCode::Char('a'),
        ReedlineEvent::Edit(vec![EditCommand::MoveToStart { select: false }]),
    );

    // Ctrl+E: 移动到行尾
    keybindings.add_binding(
        KeyModifiers::CONTROL,
        KeyCode::Char('e'),
        ReedlineEvent::Edit(vec![EditCommand::MoveToEnd { select: false }]),
    );

    // Alt+F: 向前移动一个词
    keybindings.add_binding(
        KeyModifiers::ALT,
        KeyCode::Char('f'),
        ReedlineEvent::Edit(vec![EditCommand::MoveWordRight { select: false }]),
    );

    // Alt+B: 向后移动一个词
    keybindings.add_binding(
        KeyModifiers::ALT,
        KeyCode::Char('b'),
        ReedlineEvent::Edit(vec![EditCommand::MoveWordLeft { select: false }]),
    );

    keybindings
}
