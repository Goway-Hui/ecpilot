//! settings 页面。

use super::*;

impl AppState {
    /// 设置页。
    pub(super) fn settings(&mut self, cx: &mut Context<Self>) -> Div {
        let theme = self.theme();

        let appearance = self
            .card()
            .flex()
            .flex_col()
            .gap(px(14.0))
            .child(
                div()
                    .flex()
                    .justify_between()
                    .items_center()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(2.0))
                            .child(self.label(self.t("界面语言", "Language")))
                            .child(self.hint(self.t(
                                "切换后立即生效并保存。",
                                "Applies immediately and is saved.",
                            ))),
                    )
                    .child(
                        div()
                            .flex()
                            .gap(px(6.0))
                            .child(self.pill(
                                "lang-zh",
                                "中文".into(),
                                self.lang == Lang::Zh,
                                cx,
                                |this, cx| {
                                    this.lang = Lang::Zh;
                                    save_config(this.theme_mode, this.lang);
                                    cx.notify();
                                },
                            ))
                            .child(self.pill(
                                "lang-en",
                                "English".into(),
                                self.lang == Lang::En,
                                cx,
                                |this, cx| {
                                    this.lang = Lang::En;
                                    save_config(this.theme_mode, this.lang);
                                    cx.notify();
                                },
                            )),
                    ),
            )
            .child(div().h(px(1.0)).w_full().bg(rgb(theme.border)))
            .child(
                div()
                    .flex()
                    .justify_between()
                    .items_center()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(2.0))
                            .child(self.label(self.t("主题", "Theme")))
                            .child(self.hint(self.t(
                                "深色适合夜间使用；浅色为暖纸底。",
                                "Dark for night use; light is warm paper.",
                            ))),
                    )
                    .child(
                        div()
                            .flex()
                            .gap(px(6.0))
                            .child(self.pill(
                                "theme-dark",
                                self.t("深色", "Dark"),
                                self.theme_mode == ThemeMode::Dark,
                                cx,
                                |this, cx| {
                                    this.theme_mode = ThemeMode::Dark;
                                    save_config(this.theme_mode, this.lang);
                                    cx.notify();
                                },
                            ))
                            .child(self.pill(
                                "theme-light",
                                self.t("浅色", "Light"),
                                self.theme_mode == ThemeMode::Light,
                                cx,
                                |this, cx| {
                                    this.theme_mode = ThemeMode::Light;
                                    save_config(this.theme_mode, this.lang);
                                    cx.notify();
                                },
                            )),
                    ),
            );

        let power = self
            .card()
            .flex()
            .flex_col()
            .gap(px(10.0))
            .child(
                div()
                    .flex()
                    .justify_between()
                    .items_center()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(2.0))
                            .child(self.label(self.t("电池充电上限", "Battery charge limit")))
                            .child(self.hint(format!(
                                "{} {}%",
                                self.t("当前", "current"),
                                self.status.threshold
                            ))),
                    )
                    .child(
                        div()
                            .flex()
                            .gap(px(6.0))
                            .child(self.pill(
                                "battery-60",
                                "60%".into(),
                                self.status.threshold == 60,
                                cx,
                                |this, cx| {
                                    this.run_action(cx, || ec::set_battery_threshold(60))
                                },
                            ))
                            .child(self.pill(
                                "battery-80",
                                "80%".into(),
                                self.status.threshold == 80,
                                cx,
                                |this, cx| {
                                    this.run_action(cx, || ec::set_battery_threshold(80))
                                },
                            ))
                            .child(self.pill(
                                "battery-100",
                                "100%".into(),
                                self.status.threshold == 100,
                                cx,
                                |this, cx| {
                                    this.run_action(cx, || ec::set_battery_threshold(100))
                                },
                            )),
                    ),
            )
            .child(div().h(px(1.0)).w_full().bg(rgb(theme.border)))
            .child(
                div()
                    .flex()
                    .justify_between()
                    .items_center()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(2.0))
                            .child(self.label(self.t("摄像头", "Webcam")))
                            .child(self.hint(if self.status.webcam {
                                self.t("摄像头已启用。", "Camera enabled.")
                            } else {
                                self.t("摄像头已禁用。", "Camera disabled.")
                            })),
                    )
                    .child(self.action_button(
                        "webcam-toggle",
                        if self.status.webcam {
                            self.t("关闭", "Turn off")
                        } else {
                            self.t("开启", "Turn on")
                        },
                        false,
                        cx,
                        |this, cx| {
                            let on = !this.status.webcam;
                            this.run_action(cx, move || ec::set_webcam(on))
                        },
                    )),
            )
            .child(div().h(px(1.0)).w_full().bg(rgb(theme.border)))
            .child(
                div()
                    .flex()
                    .justify_between()
                    .items_center()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(2.0))
                            .child(self.label("USB Power Share".into()))
                            .child(self.hint(if self.status.usb_power_share {
                                self.t("关机后仍为 USB 供电。", "USB stays powered when off.")
                            } else {
                                self.t("关机后停止 USB 供电。", "USB powers off with the laptop.")
                            })),
                    )
                    .child(self.action_button(
                        "usb-share-toggle",
                        if self.status.usb_power_share {
                            self.t("关闭", "Turn off")
                        } else {
                            self.t("开启", "Turn on")
                        },
                        false,
                        cx,
                        |this, cx| {
                            let on = !this.status.usb_power_share;
                            this.run_action(cx, move || ec::set_usb_power_share(on))
                        },
                    )),
            )
            .child(div().h(px(1.0)).w_full().bg(rgb(theme.border)))
            .child(
                div()
                    .flex()
                    .justify_between()
                    .items_center()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(2.0))
                            .child(self.label(self.t("Fn / Win 交换", "Fn / Win swap")))
                            .child(self.hint(if self.status.fn_swapped {
                                self.t("已交换 Fn 与 Win。", "Fn and Win are swapped.")
                            } else {
                                self.t("使用默认键位。", "Default key order.")
                            })),
                    )
                    .child(self.action_button(
                        "fn-win-toggle",
                        if self.status.fn_swapped {
                            self.t("恢复", "Restore")
                        } else {
                            self.t("交换", "Swap")
                        },
                        false,
                        cx,
                        |this, cx| {
                            let on = !this.status.fn_swapped;
                            this.run_action(cx, move || ec::set_fn_win_swap(on))
                        },
                    )),
            );

        let notifications = self
            .card()
            .flex()
            .justify_between()
            .items_center()
            .child(
                div()
                    .flex_1()
                    .min_w(px(0.0))
                    .flex()
                    .flex_col()
                    .gap(px(2.0))
                    .child(self.label(self.t("温控消息提醒", "Thermal notifications")))
                    .child(self.hint(self.t(
                        "换档、紧急锁存和高温提醒通过桌面通知发送。",
                        "Level changes, the emergency latch and high-temperature alerts are sent as desktop notifications.",
                    ))),
            )
            .child(self.switch(
                "thermal-notify-toggle",
                self.thermal_notify,
                cx,
                |this, cx| {
                    this.thermal_notify = !this.thermal_notify;
                    save_thermal_notify(this.thermal_notify);
                    cx.notify();
                },
            ));

        let window = self
            .card()
            .flex()
            .justify_between()
            .items_center()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(2.0))
                    .child(self.label(self.t(
                        "关闭时最小化到托盘",
                        "Minimize to tray on close",
                    )))
                    .child(self.hint(if self.close_to_tray {
                        self.t("关闭窗口后保留在托盘。", "Keeps running in the tray.")
                    } else {
                        self.t("关闭窗口即退出应用。", "Closing the window quits.")
                    })),
            )
            .child(self.action_button(
                "close-to-tray-toggle",
                if self.close_to_tray {
                    self.t("关闭", "Turn off")
                } else {
                    self.t("开启", "Turn on")
                },
                false,
                cx,
                |this, cx| {
                    this.close_to_tray = !this.close_to_tray;
                    save_close_to_tray(this.close_to_tray);
                    cx.notify();
                },
            ));

        let about = self
            .card()
            .flex()
            .flex_col()
            .gap(px(8.0))
            .child(self.label(format!(
                "ECPilot {} · MSI Alpha 17 C7VF",
                env!("CARGO_PKG_VERSION")
            )))
            .child(self.hint("GPUI (gpui-fast) · Rust".into()))
            .child(self.hint(self.t(
                "KLC 协议已内置（源自 msi-klc，MIT/Apache-2.0）。",
                "KLC protocol is built in (ported from msi-klc, MIT/Apache-2.0).",
            )));

        div()
            .flex()
            .flex_col()
            .gap(px(8.0))
            .child(self.page_section("外观", "Appearance"))
            .child(appearance)
            .child(self.page_section("电源与设备", "Power & devices"))
            .child(power)
            .child(self.page_section("通知", "Notifications"))
            .child(notifications)
            .child(self.page_section("窗口", "Window"))
            .child(window)
            .child(self.page_section("关于", "About"))
            .child(about)
    }
}
