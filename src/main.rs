//! 程序入口：命令行解析、单实例锁、显示后端选择。

mod app;
mod config;
mod details;
mod ec;
mod helper;
mod klc;
mod theme;
mod tray;
mod ui;

use gpui::{prelude::*, App, Bounds, QuitMode, WindowBounds, WindowOptions, px, size};

use crate::app::{AppState, Page};

/// 使用 XDG_RUNTIME_DIR 下的文件锁保证只运行一个 GUI 实例。
fn acquire_single_instance() -> Option<nix::fcntl::Flock<std::fs::File>> {
    use nix::fcntl::{Flock, FlockArg};

    let dir = std::env::var_os("XDG_RUNTIME_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    let path = dir.join(format!("ecpilot-{}.lock", nix::unistd::getuid()));
    let file = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(path)
        .ok()?;
    Flock::lock(file, FlockArg::LockExclusiveNonblock).ok()
}


/// 程序入口：处理 helper 模式、单实例锁与窗口创建。
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|arg| arg == "--dbus-helper") {
        if let Err(error) = helper::run() {
            eprintln!("{error:#}");
            std::process::exit(1);
        }
        return;
    }
    if let Some(position) = args.iter().position(|arg| arg == "--helper") {
        if let Err(error) = ec::helper_main(&args[position + 1..]) {
            eprintln!("{error:#}");
            std::process::exit(1);
        }
        return;
    }

    /* FluxDown-style launch: `--x11` (or `env -u WAYLAND_DISPLAY`) hides the
     * Wayland display so GPUI starts on X11/XWayland. The default remains
     * Wayland on niri; other Wayland desktops fall back to X11.
     */
    let x11_arg = args.iter().any(|arg| arg == "--x11");
    let force_x11 = x11_arg || std::env::var_os("MCC_FORCE_X11").is_some();
    // 默认 X11；Wayland 需显式开启（MCC_ALLOW_WAYLAND=1 或 Wayland 桌面入口）。
    let allow_wayland = std::env::var_os("MCC_ALLOW_WAYLAND").is_some();
    if std::env::var_os("WAYLAND_DISPLAY").is_some()
        && std::env::var_os("DISPLAY").is_some()
        && (force_x11 || !allow_wayland)
    {
        if let Ok(executable) = std::env::current_exe() {
            let mut command = std::process::Command::new(executable);
            command.args(std::env::args_os().skip(1));
            command.env_remove("WAYLAND_DISPLAY");
            if let Ok(status) = command.status() {
                std::process::exit(status.code().unwrap_or(1));
            }
        }
    }

    let _instance_lock = match acquire_single_instance() {
        Some(lock) => lock,
        None => {
            eprintln!("ECPilot is already running");
            std::process::exit(0);
        }
    };

    let close_to_tray = crate::config::load_close_to_tray();
    crate::app::set_close_to_tray(close_to_tray);
    let (tray_tx, tray_rx) = std::sync::mpsc::channel();
    let windows = crate::app::new_tray_windows();
    let page = match args
        .iter()
        .position(|arg| arg == "--page")
        .and_then(|position| args.get(position + 1))
        .map(String::as_str)
    {
        Some("fan") => Page::Fan,
        Some("keyboard") => Page::Keyboard,
        Some("gpu") => Page::Gpu,
        Some("telemetry") => Page::Telemetry,
        Some("thermal") => Page::Thermal,
        Some("settings") => Page::Settings,
        _ => Page::Overview,
    };

    // 没有托盘宿主时，close-to-tray 会让应用不可达，不要驻留。
    let tray_available = crate::tray::is_available();
    if tray_available {
        tray::spawn(tray_tx);
    }
    crate::app::set_close_to_tray(close_to_tray && tray_available);

    gpui_platform::application()
        .with_quit_mode(if close_to_tray {
            QuitMode::Explicit
        } else {
            QuitMode::LastWindowClosed
        })
        .run(move |cx: &mut App| {
            crate::app::spawn_tray_polling(cx, tray_rx, std::sync::Arc::clone(&windows));
            let window_global = std::sync::Arc::clone(&windows);
            let bounds = Bounds::centered(None, size(px(1180.0), px(980.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(gpui::TitlebarOptions {
                    title: Some("ECPilot".into()),
                    ..Default::default()
                }),
                app_id: Some("ecpilot".into()),
                ..Default::default()
            },
            move |window, cx| {
                *window_global.lock().unwrap() = Some(window.window_handle());
                cx.new(move |cx| AppState::new(cx, page))
            },
        )
        .unwrap();
        cx.activate(true);
    });
}
