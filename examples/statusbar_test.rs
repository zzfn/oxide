use crossterm::terminal;
use reedline::{DefaultPrompt, Reedline, Signal};
use std::io::{stdout, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    println!("=== DECSTBM + Reedline 兼容性测试 ===\n");

    let (width, height) = terminal::size()?;
    println!("终端尺寸: {}x{}", width, height);

    if height < 5 {
        println!("错误: 终端高度不足");
        return Ok(());
    }

    println!("按 Ctrl+D 退出\n");
    thread::sleep(Duration::from_millis(500));

    // 设置滚动区域
    print!("\x1b[1;{}r", height - 1);
    stdout().flush()?;

    // 移动光标到滚动区域底部（状态栏上方）
    print!("\x1b[{};1H", height - 2);
    stdout().flush()?;

    // 状态栏数据
    let counter = Arc::new(Mutex::new(0));
    let counter_clone = counter.clone();
    let refresh_count = Arc::new(Mutex::new(0u32));
    let refresh_count_clone = refresh_count.clone();

    // 后台线程：持续刷新状态栏
    let refresh_handle = thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_millis(100));
            let count = *counter_clone.lock().unwrap();
            let mut refresh = refresh_count_clone.lock().unwrap();
            *refresh = refresh.wrapping_add(1);
            let _ = render_statusbar(width, height, count, *refresh);
        }
    });

    // Reedline 循环
    let mut line_editor = Reedline::create();
    let prompt = DefaultPrompt::default();

    loop {
        let sig = line_editor.read_line(&prompt);
        match sig {
            Ok(Signal::Success(buffer)) => {
                println!("输入: {}", buffer.trim());
                let mut count = counter.lock().unwrap();
                *count += 1;
            }
            Ok(Signal::CtrlD) | Ok(Signal::CtrlC) => {
                println!("\n退出");
                break;
            }
            _ => {}
        }
    }

    drop(refresh_handle);
    cleanup_statusbar(height)?;
    println!("测试完成");
    Ok(())
}

fn render_statusbar(width: u16, height: u16, counter: usize, refresh: u32) -> anyhow::Result<()> {
    print!("\x1b[s");
    print!("\x1b[{};1H", height);

    let info = format!(" 输入: {} | 刷新: {} | 终端: {}x{} ", counter, refresh, width, height);
    let padding = " ".repeat((width as usize).saturating_sub(info.len()));
    print!("\x1b[48;5;238m{}{}\x1b[0m", info, padding);

    print!("\x1b[u");
    stdout().flush()?;
    Ok(())
}

fn cleanup_statusbar(height: u16) -> anyhow::Result<()> {
    print!("\x1b[r");
    print!("\x1b[{};1H\x1b[2K", height);
    stdout().flush()?;
    Ok(())
}
