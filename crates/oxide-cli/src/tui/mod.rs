pub mod app_event;
pub mod chat_widget;
pub mod bottom_pane;
pub mod history_cell;
pub mod markdown_render;
pub mod diff_render;
pub mod exec_cell;
pub mod approval_overlay;
pub mod event_loop;
pub mod style;

pub use app_event::{AppEvent, AppEventSender};
pub use event_loop::run_tui;
