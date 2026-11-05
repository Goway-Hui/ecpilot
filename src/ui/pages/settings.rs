//! settings 页面。

use super::*;

impl AppState {
    /// 设置页。
    pub(super) fn settings(&mut self, cx: &mut Context<Self>) -> Div {
        div()
            .flex()
            .flex_col()
            .gap_4()
            .child(
                self.card()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(self.label(self.t("语言", "Language")))
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .child(self.pill("lang-zh", "中文".into(), self.lang == Lang::Zh, cx, |this, cx| {
                                this.lang = Lang::Zh;
                                save_config(this.theme_mode, this.lang);
                                cx.notify();
                            }))
                            .child(self.pill("lang-en", "English".into(), self.lang == Lang::En, cx, |this, cx| {
                                this.lang = Lang::En;
                                save_config(this.theme_mode, this.lang);
                                cx.notify();
                            })),
                    ),
            )
            .child(
                self.card()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(self.label(self.t("主题", "Theme")))
                    .child(
                        div()
                            .flex()
                            .gap_2()
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
            )
            .child(
                self.card()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(self.label(self.t("电池充电上限", "Battery charge limit")))
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .child(self.pill(
                                "battery-60",
                                "60%".into(),
                                self.status.threshold == 60,
                                cx,
                                |this, cx| this.action(ec::set_battery_threshold(60), cx),
                            ))
                            .child(self.pill(
                                "battery-80",
                                "80%".into(),
                                self.status.threshold == 80,
                                cx,
                                |this, cx| this.action(ec::set_battery_threshold(80), cx),
                            ))
                            .child(self.pill(
                                "battery-100",
                                "100%".into(),
                                self.status.threshold == 100,
                                cx,
                                |this, cx| this.action(ec::set_battery_threshold(100), cx),
                            )),
                    )
                    .child(self.label(format!(
                        "{} {}%",
                        self.t("当前", "Current"),
                        self.status.threshold
                    ))),
            )
            .child(
                self.card()
                    .flex()
                    .justify_between()
                    .items_center()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(self.label(self.t("摄像头", "Webcam")))
                            .child(self.label(if self.status.webcam {
                                self.t("开启", "on")
                            } else {
                                self.t("关闭", "off")
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
                            this.action(ec::set_webcam(on), cx)
                        },
                    )),
            )
            .child(
                self.card()
                    .flex()
                    .justify_between()
                    .items_center()
                    .child(self.label(self.t("USB Power Share", "USB Power Share")))
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
                            this.action(ec::set_usb_power_share(on), cx)
                        },
                    )),
            )
            .child(
                self.card()
                    .flex()
                    .justify_between()
                    .items_center()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(self.label(self.t("Fn / Win 交换", "Fn / Win swap")))
                            .child(self.label(if self.status.fn_swapped {
                                self.t("已交换", "swapped")
                            } else {
                                self.t("默认", "default")
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
                            this.action(ec::set_fn_win_swap(on), cx)
                        },
                    )),
            )
            .child(
                self.card()
                    .flex()
                    .justify_between()
                    .items_center()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(self.label(self.t(
                                "关闭时最小化到托盘",
                                "Minimize to tray on close",
                            )))
                            .child(self.label(if self.close_to_tray {
                                self.t("开启", "on")
                            } else {
                                self.t("关闭", "off")
                            }))
                            .child(self.label(self.t(
                                "重启应用后生效。",
                                "Takes effect after restart.",
                            ))),
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
                            crate::app::set_close_to_tray(this.close_to_tray);
                            save_close_to_tray(this.close_to_tray);
                            cx.notify();
                        },
                    )),
            )
            .child(
                self.card()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(self.label(self.t("关于", "About")))
                    .child(self.label(format!(
                        "{} {}",
                        self.t("版本", "Version"),
                        env!("CARGO_PKG_VERSION")
                    )))
                    .child(self.label("GPUI 0.2 · Rust".into()))
                    .child(self.label(self.t(
                        "KLC 协议已内置（源自 msi-klc，MIT/Apache-2.0）。",
                        "KLC protocol is built in (ported from msi-klc, MIT/Apache-2.0).",
                    ))),
            )
    }

}
