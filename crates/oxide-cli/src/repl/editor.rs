//! Reedline 编辑器配置
//!
//! 配置和创建 Reedline 编辑器实例。

use anyhow::Result;
use reedline::{
    Emacs, FileBackedHistory, Reedline, ReedlineMenu,
    ColumnarMenu, MenuBuilder,
};
use std::path::PathBuf;
use std::sync::Arc;

use super::completer::OxideCompleter;
use super::keybindings::create_keybindings;
use crate::commands::CommandRegistry;

/// 历史文件名
const HISTORY_FILE: &str = ".oxide_history";

/// 创建 Reedline 编辑器
pub fn create_editor(
    commands: Arc<CommandRegistry>,
    working_dir: PathBuf,
) -> Result<Reedline> {
    // 创建历史记录
    let history_path = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(HISTORY_FILE);

    let history = Box::new(
        FileBackedHistory::with_file(1000, history_path)?
    );

    // 创建补全器
    let completer = Box::new(OxideCompleter::new(commands, working_dir));

    // 创建补全菜单
    let completion_menu = Box::new(
        ColumnarMenu::default()
            .with_name("completion_menu")
            .with_columns(4)
            .with_column_padding(2)
    );

    // 创建快捷键绑定
    let keybindings = create_keybindings();
    let edit_mode = Box::new(Emacs::new(keybindings));

    // 构建编辑器
    let editor = Reedline::create()
        .with_history(history)
        .with_completer(completer)
        .with_menu(ReedlineMenu::EngineCompleter(completion_menu))
        .with_edit_mode(edit_mode);

    Ok(editor)
}

/// 更新编辑器的补全器
pub fn update_completer(
    editor: &mut Reedline,
    commands: Arc<CommandRegistry>,
    working_dir: PathBuf,
) {
    let completer = Box::new(OxideCompleter::new(commands, working_dir));
    // 注意：Reedline 不支持直接更新 completer，需要重新创建
    // 这里只是一个占位实现
    let _ = (editor, completer);
}
