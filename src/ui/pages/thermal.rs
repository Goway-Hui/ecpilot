//! thermal 页面。

use super::*;

impl AppState {
    /// 温控保护页：显示档位、温度，并支持启停。
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
        let sensor = |label: &str, value: i32| {
            self.card()
                .flex_1()
                .flex()
                .flex_col()
                .gap(px(4.0))
                .child(self.card_title(label, label))
                .child(
                    div()
                        .text_size(px(30.0))
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(rgb(temp_color(value)))
                        .child(value.to_string()),
                )
                .child(self.hint("°C".into()))
        };
        let tctl = sensor("Tctl", self.status.tctl);
        let tccd1 = sensor("Tccd1", self.status.tccd1);
        let tccd2 = sensor("Tccd2", self.status.tccd2);

        let level_card = self
            .card()
            .flex_1()
            .flex()
            .flex_col()
            .gap(px(8.0))
            .child(self.card_title("当前档位", "Current level"))
            .child(
                div()
                    .text_size(px(22.0))
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(rgb(level_color))
                    .child(level_text.clone()),
            )
            .child(self.level_bar(level, level_color))
            .child(self.hint(format!(
                "{} {} · boost {}",
                self.t("上限", "cap"),
                cap,
                if self.status.thermal_boost {
                    self.t("开", "on")
                } else {
                    self.t("关", "off")
                }
            )));

        let rules = self
            .card()
            .flex_1()
            .flex()
            .flex_col()
            .gap(px(6.0))
            .child(self.card_title("档位规则", "Level rules"))
            .child({
                let config = &self.status.thermal_config;
                let cap = |khz: i32| khz as f32 / 1_000_000.0;
                self.hint(if self.lang == Lang::Zh {
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
                })
            })
            .child({
                let config = &self.status.thermal_config;
                self.hint(if self.lang == Lang::Zh {
                    format!(
                        "回落 ≤{}/{}/{}°C · 停留 {} ms\n锁存 {}°C/{}s · 采样 {} ms · 安全档 {:.1} GHz",
                        config.down1, config.down2, config.down3,
                        config.dwell_ms, config.latch_temp, config.latch_secs, config.sample_ms,
                        config.base_khz as f32 / 1_000_000.0
                    )
                } else {
                    format!(
                        "down <= {}/{}/{}°C · dwell {} ms\nlatch {}°C/{}s · sample {} ms · safe {:.1} GHz",
                        config.down1, config.down2, config.down3,
                        config.dwell_ms, config.latch_temp, config.latch_secs, config.sample_ms,
                        config.base_khz as f32 / 1_000_000.0
                    )
                })
            });

        let toggle: AnyElement = if active {
            self.action_button(
                "thermal-toggle",
                self.t("停用保护", "Disable"),
                true,
                cx,
                |this, cx| {
                    let on = !this.status.thermal_active;
                    this.run_action(cx, move || ec::set_thermal_guard(on))
                },
            )
            .into_any_element()
        } else {
            self.primary_button(
                "thermal-toggle",
                self.t("启用保护", "Enable"),
                cx,
                |this, cx| {
                    let on = !this.status.thermal_active;
                    this.run_action(cx, move || ec::set_thermal_guard(on))
                },
            )
            .into_any_element()
        };

        let control = self
            .card()
            .flex_1()
            .flex()
            .flex_col()
            .gap(px(10.0))
            .child(self.card_title("保护控制", "Guard control"))
            .child(self.info_row(
                self.t("服务", "Service").as_str(),
                if active {
                    self.t("运行中", "running")
                } else {
                    self.t("已停止", "stopped")
                },
                if active { theme.green } else { theme.red },
            ))
            .child(self.info_row(
                self.t("档位", "Level").as_str(),
                level_text.clone(),
                level_color,
            ))
            .child(self.info_row(
                self.t("频率上限", "Frequency cap").as_str(),
                cap.clone(),
                theme.accent,
            ))
            .child(self.info_row(
                "Boost",
                if self.status.thermal_boost {
                    self.t("开", "on")
                } else {
                    self.t("关", "off")
                },
                if self.status.thermal_boost {
                    theme.green
                } else {
                    theme.faint
                },
            ))
            .child(
                div().flex().gap(px(8.0)).pt(px(4.0)).child(toggle).child(
                    self.action_button(
                        "thermal-restart",
                        self.t("重启保护", "Restart"),
                        false,
                        cx,
                        |this, cx| this.run_action(cx, ec::restart_thermal_guard),
                    ),
                ),
            )
            .child(self.hint(self.t(
                "helper 内置线程运行；关闭应用不会停止保护。",
                "Runs in the helper; closing the app does not stop protection.",
            )));

        let sensors = if self.compact {
            div()
                .flex()
                .flex_col()
                .gap(px(12.0))
                .child(div().flex().gap(px(12.0)).child(tctl).child(tccd1))
                .child(div().flex().gap(px(12.0)).child(tccd2).child(level_card))
        } else {
            div()
                .flex()
                .gap(px(14.0))
                .child(tctl)
                .child(tccd1)
                .child(tccd2)
                .child(level_card)
        };
        let mut page = div()
            .flex()
            .flex_col()
            .gap(px(14.0))
            .child(sensors)
            .child(
                div()
                    .flex()
                    .gap(px(14.0))
                    .items_stretch()
                    .child(rules)
                    .child(control),
            );

        if !self.status.thermal_error.is_empty() {
            page = page.child(
                self.card()
                    .bg(rgb(theme.error_bg))
                    .border_color(rgb(theme.red))
                    .child(
                        div()
                            .text_size(px(12.5))
                            .text_color(rgb(theme.error_text))
                            .child(format!(
                                "{} {}",
                                self.t("温控写入错误：", "Thermal write error: "),
                                self.status.thermal_error
                            )),
                    ),
            );
        }
        if !active {
            page = page.child(
                self.card()
                    .bg(rgb(theme.error_bg))
                    .border_color(rgb(theme.red))
                    .child(
                        div()
                            .text_size(px(12.5))
                            .text_color(rgb(theme.error_text))
                            .child(self.t(
                                "当前未启用温度保护；长时间满载可能触发热断电。",
                                "Thermal protection is off; sustained load may trigger a thermal shutdown.",
                            )),
                    ),
            );
        }
        page
    }
}
