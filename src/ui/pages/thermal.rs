//! thermal 页面。

use super::*;

impl AppState {
    /// 温控保护页：显示 thermal-guard 档位、温度，并支持启停。
    pub(super) fn thermal(&mut self, cx: &mut Context<Self>) -> Div {
        let theme = self.theme();
        let level = self.status.thermal_level;
        let active = self.status.thermal_active;
        let (level_text, level_color) = match level {
            0 => (self.t("正常", "Normal"), theme.green),
            1 => (self.t("降档 1", "Level 1"), theme.yellow),
            2 => (self.t("降档 2", "Level 2"), theme.peach),
            _ => (self.t("紧急锁存", "Emergency latch"), theme.red),
        };
        let cap = if self.status.thermal_cap_mhz > 0 {
            format!("{:.1} GHz", self.status.thermal_cap_mhz as f32 / 1000.0)
        } else {
            self.t("不限制", "unlimited")
        };
        let temp_color = |value: i32| {
            if value >= 97 {
                theme.red
            } else if value >= 90 {
                theme.yellow
            } else {
                theme.green
            }
        };
        let boost = if self.status.thermal_boost {
            self.t("开启", "on")
        } else {
            self.t("关闭", "off")
        };
        let mut page = div()
            .flex()
            .flex_col()
            .gap_3()
            .child(
                div()
                    .flex()
                    .gap_3()
                    .child(
                        self.card()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(self.label("Tctl".into()))
                            .child(self.metric(
                                "CPU".into(),
                                self.status.tctl.to_string(),
                                "°C".into(),
                                temp_color(self.status.tctl),
                            )),
                    )
                    .child(
                        self.card()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(self.label("Tccd1".into()))
                            .child(self.metric(
                                "CCD0".into(),
                                self.status.tccd1.to_string(),
                                "°C".into(),
                                temp_color(self.status.tccd1),
                            )),
                    )
                    .child(
                        self.card()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(self.label("Tccd2".into()))
                            .child(self.metric(
                                "CCD1".into(),
                                self.status.tccd2.to_string(),
                                "°C".into(),
                                temp_color(self.status.tccd2),
                            )),
                    )
                    .child(
                        self.card()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(self.label(self.t("当前档位", "Current level")))
                            .child(div().text_color(rgb(level_color)).child(self.label(level_text.clone())))
                            .child(self.label(format!(
                                "{} {} · {}",
                                self.t("上限", "Cap"),
                                cap,
                                boost
                            ))),
                    ),
            )
            .child(
                self.card()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(self.info_line(
                        self.t("服务", "Service").as_str(),
                        if active {
                            self.t("运行中", "running")
                        } else {
                            self.t("已停止", "stopped")
                        },
                    ))
                    .child(self.info_line(self.t("档位", "Level").as_str(), level_text))
                    .child(self.info_line(self.t("频率上限", "Frequency cap").as_str(), cap))
                    .child(self.info_line("Boost", boost))
                    .child(self.info_line(
                        self.t("CPU 风扇", "CPU fan").as_str(),
                        format!("{} RPM", self.status.cpu_fan),
                    )),
            )
            .child(
                self.card()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(self.label(self.t("档位规则", "Level rules")))
                    .child(self.label({
                        let config = &self.status.thermal_config;
                        let cap = |khz: i32| khz as f32 / 1_000_000.0;
                        if self.lang == Lang::Zh {
                            format!(
                                "0：≤{}°C → {:.1} GHz，boost 开\n1：≥{}°C → {:.1} GHz\n2：≥{}°C → {:.1} GHz，boost 关\n3：≥{}°C → {:.1} GHz，热锁存",
                                config.up1, cap(config.cap0), config.up1, cap(config.cap1),
                                config.up2, cap(config.cap2), config.up3, cap(config.cap3)
                            )
                        } else {
                            format!(
                                "0: <= {}°C -> {:.1} GHz, boost on\n1: >= {}°C -> {:.1} GHz\n2: >= {}°C -> {:.1} GHz, boost off\n3: >= {}°C -> {:.1} GHz, thermal latch",
                                config.up1, cap(config.cap0), config.up1, cap(config.cap1),
                                config.up2, cap(config.cap2), config.up3, cap(config.cap3)
                            )
                        }
                    }))
                    .child(self.label({
                        let config = &self.status.thermal_config;
                        if self.lang == Lang::Zh {
                            format!(
                                "回落 ≤{}/{}/{}°C · 停留 {} ms · 锁存 {}°C/{}s · 采样 {} ms · 安全档 {:.1} GHz",
                                config.down1, config.down2, config.down3,
                                config.dwell_ms, config.latch_temp, config.latch_secs, config.sample_ms,
                                config.base_khz as f32 / 1_000_000.0
                            )
                        } else {
                            format!(
                                "down <= {}/{}/{}°C · dwell {} ms · latch {}°C/{}s · sample {} ms · safe {:.1} GHz",
                                config.down1, config.down2, config.down3,
                                config.dwell_ms, config.latch_temp, config.latch_secs, config.sample_ms,
                                config.base_khz as f32 / 1_000_000.0
                            )
                        }
                    })),
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
                                "内置温度保护",
                                "Built-in thermal guard",
                            )))
                            .child(self.label(if active {
                                self.t("helper 内置线程运行中，关闭应用不会停止保护。", "Running in the helper thread; closing the app does not stop protection.")
                            } else {
                                self.t("已停止，存在热断电风险。", "Stopped; thermal shutdown risk.")
                            })),
                    )
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .child(self.action_button(
                                "thermal-toggle",
                                if active {
                                    self.t("停用保护", "Disable")
                                } else {
                                    self.t("启用保护", "Enable")
                                },
                                false,
                                cx,
                                |this, cx| {
                                    let on = !this.status.thermal_active;
                                    this.run_action(cx, move || ec::set_thermal_guard(on))
                                },
                            ))
                            .child(self.action_button(
                                "thermal-restart",
                                self.t("重启保护", "Restart"),
                                false,
                                cx,
                                |this, cx| this.run_action(cx, ec::restart_thermal_guard),
                            )),
                    ),
            );
        if !self.status.thermal_error.is_empty() {
            page = page.child(
                self.card()
                    .bg(rgb(theme.error_bg))
                    .text_color(rgb(theme.error_text))
                    .child(self.label(format!(
                        "{} {}",
                        self.t("温控写入错误：", "Thermal write error: "),
                        self.status.thermal_error
                    ))),
            );
        }
        if !active {
            page = page.child(
                self.card()
                    .bg(rgb(theme.error_bg))
                    .text_color(rgb(theme.error_text))
                    .child(self.label(self.t(
                        "当前未启用温度保护；长时间满载可能触发热断电。",
                        "Thermal protection is off; sustained load may trigger a thermal shutdown.",
                    ))),
            );
        }
        page
    }

}
