//! 最小化的 StatusNotifierItem 托盘。
//!
//! GPUI 没有托盘 API，因此用 ksni 在独立 D-Bus 线程中运行。托盘只负责发送
//! `TrayCommand`，窗口的显示/退出由 GPUI 主线程处理。

use std::sync::mpsc::Sender;

use ksni::menu::StandardItem;

/// 托盘发给 GPUI 主线程的命令。
#[derive(Debug)]
pub(crate) enum TrayCommand {
    /// 显示并激活主窗口。
    Show,
    /// 退出整个应用。
    Quit,
}

struct Tray {
    tx: Sender<TrayCommand>,
}

impl Tray {
    fn send(&self, command: TrayCommand) {
        let _ = self.tx.send(command);
    }
}

impl ksni::Tray for Tray {
    fn id(&self) -> String {
        "ecpilot".into()
    }

    fn title(&self) -> String {
        "ECPilot".into()
    }

    fn icon_name(&self) -> String {
        "applications-system".into()
    }

    /// 左键点击托盘图标：显示主窗口。
    fn activate(&mut self, _x: i32, _y: i32) {
        self.send(TrayCommand::Show);
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        let show_tx = self.tx.clone();
        let quit_tx = self.tx.clone();
        vec![
            StandardItem {
                label: "显示窗口 / Show Window".into(),
                activate: Box::new(move |_| {
                    let _ = show_tx.send(TrayCommand::Show);
                }),
                ..Default::default()
            }
            .into(),
            ksni::menu::MenuItem::Separator,
            StandardItem {
                label: "退出 / Quit".into(),
                activate: Box::new(move |_| {
                    let _ = quit_tx.send(TrayCommand::Quit);
                }),
                ..Default::default()
            }
            .into(),
        ]
    }
}

/// 询问会话总线上是否有 StatusNotifier 宿主；没有时不应创建托盘。
pub fn is_available() -> bool {
    use zbus::blocking::Connection;
    let Ok(conn) = Connection::session() else { return false };
    let Ok(proxy) = zbus::blocking::Proxy::new(
        &conn,
        "org.freedesktop.DBus",
        "/org/freedesktop/DBus",
        "org.freedesktop.DBus",
    ) else {
        return false;
    };
    proxy
        .call::<_, _, bool>("NameHasOwner", &("org.kde.StatusNotifierWatcher",))
        .is_ok_and(|available| available)
}

/// 启动托盘线程；命令通过 `tx` 发送给 GPUI。
pub fn spawn(tx: Sender<TrayCommand>) {
    let service = ksni::TrayService::new(Tray { tx });
    service.spawn();
}
